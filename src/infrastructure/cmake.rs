//! Compiles a CMake project: a fresh build tree per build, the output of
//! CMake and the compilers streamed to a build log, and the build's progress
//! read from the step counters Ninja and Make print.

use std::collections::{BTreeMap, VecDeque};
use std::fs::{self, File};
use std::io::{self, BufRead, BufReader, BufWriter, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use anyhow::{Context, Result, bail};

use crate::application::ports::{CompileJob, Compiled, Compiler, Progress};
use crate::domain::machine::Tool;

use super::command;
use super::files::{create_private_dir, remove_dir_if_present};

/// Lines of the log quoted when a step fails: the compiler's error and
/// the build tool's summary.
const TAIL: usize = 20;

/// Build trees under `work_root`, one per project; build logs under
/// `log_dir`, one per project, replaced by each build.
pub struct Cmake {
    /// The CMake program: `cmake`, found on `PATH`.
    program: PathBuf,
    work_root: PathBuf,
    log_dir: PathBuf,
}

impl Cmake {
    pub fn new(work_root: PathBuf, log_dir: PathBuf) -> Self {
        Self {
            program: "cmake".into(),
            work_root,
            log_dir,
        }
    }

    fn build_tree(&self, project: &str) -> PathBuf {
        self.work_root.join(project)
    }

    pub fn log_file(&self, project: &str) -> PathBuf {
        self.log_dir.join(format!("build-{project}.log"))
    }

    /// Creates the project's build log, empty.
    fn create_log(&self, project: &str) -> Result<File> {
        create_private_dir(&self.log_dir)?;
        let path = self.log_file(project);
        File::create(&path).with_context(|| format!("cannot create {}", path.display()))
    }
}

impl Compiler for Cmake {
    fn compile(&self, job: &CompileJob, progress: &mut dyn FnMut(Progress)) -> Result<Compiled> {
        let tree = self.build_tree(job.project);
        remove_dir_if_present(&tree)?;
        fs::create_dir_all(&tree).with_context(|| format!("cannot create {}", tree.display()))?;
        let file = self.create_log(job.project)?;
        let log_path = self.log_file(job.project);
        let mut log = Log::new(BufWriter::new(file), &log_path);

        progress(Progress::Configuring);
        let mut configure = Command::new(&self.program);
        configure
            .arg("-S")
            .arg(job.source)
            .arg("-B")
            .arg(&tree)
            .args(generator())
            .args(definitions(job.flags));
        log.stream("configure", configure, &mut |_| {})?;

        let jobs = job.jobs.unwrap_or_else(|| {
            std::thread::available_parallelism().map_or(1, std::num::NonZero::get)
        });
        let mut build = Command::new(&self.program);
        build.arg("--build").arg(&tree).args([
            "--target",
            job.target,
            "--parallel",
            &jobs.to_string(),
        ]);
        log.stream("build", build, &mut |line| {
            if let Some((done, total)) = parse_progress(line) {
                progress(Progress::Building { done, total });
            }
        })?;
        log.finish()?;

        let artifact = tree.join(job.artifact);
        if !artifact.is_file() {
            bail!(
                "the build succeeded but {} is missing; see {}",
                artifact.display(),
                log_path.display()
            );
        }
        Ok(Compiled {
            artifact,
            log: log_path,
        })
    }

    fn clean(&self, project: &str) -> Result<()> {
        remove_dir_if_present(&self.build_tree(project))
    }
}

/// Ninja when it is installed: its `[n/total]` counters give an exact
/// progress, and it schedules the CUDA compilations better than Make.
fn generator() -> Vec<&'static str> {
    let on_path = |name: &str| {
        std::env::var_os("PATH")
            .is_some_and(|path| std::env::split_paths(&path).any(|dir| dir.join(name).is_file()))
    };
    if on_path("ninja") || on_path("ninja-build") {
        vec!["-G", "Ninja"]
    } else {
        vec![]
    }
}

fn definitions(flags: &BTreeMap<String, String>) -> Vec<String> {
    flags
        .iter()
        .map(|(name, value)| format!("-D{name}={value}"))
        .collect()
}

/// The build log, and the last lines written to it.
struct Log<'a, W: Write> {
    out: W,
    path: &'a Path,
    tail: VecDeque<String>,
}

impl<'a, W: Write> Log<'a, W> {
    fn new(out: W, path: &'a Path) -> Self {
        Self {
            out,
            path,
            tail: VecDeque::with_capacity(TAIL),
        }
    }

    fn write_line(&mut self, line: &str) -> Result<()> {
        writeln!(self.out, "{line}")
            .with_context(|| format!("cannot write {}", self.path.display()))?;
        if self.tail.len() == TAIL {
            self.tail.pop_front();
        }
        self.tail.push_back(line.to_owned());
        Ok(())
    }

    /// Runs `command` with its stdout and stderr merged into the log, in
    /// the order they were printed, and hands each line to `on_line`.
    fn stream(
        &mut self,
        step: &str,
        mut command: Command,
        on_line: &mut dyn FnMut(&str),
    ) -> Result<()> {
        self.write_line(&format!("$ {}", render(&command)))?;
        let (reader, writer) = io::pipe().context("cannot create a pipe")?;
        let stderr = writer.try_clone().context("cannot create a pipe")?;
        command
            .env("LC_ALL", "C")
            .stdin(Stdio::null())
            .stdout(writer)
            .stderr(stderr);
        let mut child = command::spawn(&mut command).map_err(|err| match err.kind() {
            io::ErrorKind::NotFound => anyhow::anyhow!(command::not_found(Tool::Cmake)),
            _ => anyhow::anyhow!("cannot run cmake: {err}"),
        })?;
        // The command holds the pipe's write ends: the reader sees the end
        // of the output only once they are closed.
        drop(command);

        let mut lines = BufReader::new(reader);
        let mut buffer = Vec::new();
        let mut written = Ok(());
        loop {
            buffer.clear();
            match lines.read_until(b'\n', &mut buffer) {
                Ok(0) => break,
                Ok(_) => {
                    let text = String::from_utf8_lossy(&buffer);
                    let line = text.trim_end_matches(['\n', '\r']);
                    on_line(line);
                    // Keep reading after a write error, so the child never
                    // blocks on a full pipe; the error is reported after.
                    if written.is_ok() {
                        written = self.write_line(line);
                    }
                }
                Err(err) if err.kind() == io::ErrorKind::Interrupted => {}
                Err(err) => {
                    written = Err(err).context("cannot read the output of cmake");
                    break;
                }
            }
        }
        let status = child.wait().context("cannot wait for cmake")?;
        written?;
        if !status.success() {
            self.out.flush().ok();
            bail!(
                "cmake {step} failed ({status}); the last lines of {}:\n{}",
                self.path.display(),
                self.tail.iter().cloned().collect::<Vec<_>>().join("\n")
            );
        }
        Ok(())
    }

    fn finish(mut self) -> Result<()> {
        self.out
            .flush()
            .with_context(|| format!("cannot write {}", self.path.display()))
    }
}

/// The command line, for the log.
fn render(command: &Command) -> String {
    std::iter::once(command.get_program())
        .chain(command.get_args())
        .map(|arg| arg.to_string_lossy().into_owned())
        .collect::<Vec<_>>()
        .join(" ")
}

/// A build step counter: Ninja's `[12/356] Building CUDA object …` gives
/// 12 of 356; Make's `[ 45%] Building CXX object …` gives 45 of 100.
pub fn parse_progress(line: &str) -> Option<(u32, u32)> {
    let inside = line.strip_prefix('[')?.split_once(']')?.0;
    if let Some((done, total)) = inside.split_once('/') {
        let (done, total) = (done.trim().parse().ok()?, total.trim().parse().ok()?);
        return (total > 0 && done <= total).then_some((done, total));
    }
    let percent: u32 = inside.strip_suffix('%')?.trim().parse().ok()?;
    (percent <= 100).then_some((percent, 100))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::command::script;
    use crate::infrastructure::files;

    #[test]
    fn ninja_counters_give_the_progress() {
        let excerpt = include_str!("../../tests/fixtures/ninja-build-excerpt.txt");
        let progress: Vec<(u32, u32)> = excerpt.lines().filter_map(parse_progress).collect();
        assert_eq!(
            progress,
            [
                (1, 356),
                (2, 356),
                (3, 356),
                (178, 356),
                (179, 356),
                (354, 356),
                (355, 356),
                (356, 356)
            ]
        );
    }

    #[test]
    fn make_percentages_give_the_progress() {
        let excerpt = include_str!("../../tests/fixtures/make-build-excerpt.txt");
        let progress: Vec<(u32, u32)> = excerpt.lines().filter_map(parse_progress).collect();
        assert_eq!(
            progress,
            [
                (40, 100),
                (40, 100),
                (60, 100),
                (60, 100),
                (80, 100),
                (100, 100),
                (100, 100)
            ]
        );
    }

    #[test]
    fn other_lines_give_no_progress() {
        for line in [
            "",
            "-- Found CUDAToolkit",
            "/src/common/hf-cache.cpp:236:50: warning: [-Wdeprecated-declarations]",
            "[0/0] nothing",
            "[5/3] backwards",
            "[x/356] word",
            "[101%] too far",
            "ninja: build stopped: subcommand failed.",
        ] {
            assert_eq!(parse_progress(line), None, "{line:?}");
        }
    }

    #[test]
    fn flags_become_cmake_definitions() {
        let flags = BTreeMap::from([
            ("GGML_CUDA".to_owned(), "ON".to_owned()),
            (
                "CMAKE_CUDA_ARCHITECTURES".to_owned(),
                "120a-real".to_owned(),
            ),
        ]);
        assert_eq!(
            definitions(&flags),
            ["-DCMAKE_CUDA_ARCHITECTURES=120a-real", "-DGGML_CUDA=ON"]
        );
    }

    fn shell(script: &str) -> Command {
        let mut command = Command::new("sh");
        command.args(["-c", script]);
        command
    }

    #[test]
    fn the_log_holds_both_streams_in_order() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("build.log");
        let mut lines = Vec::new();
        {
            let mut log = Log::new(File::create(&path).unwrap(), &path);
            log.stream(
                "build",
                shell("echo '[1/2] one'; echo oops >&2; echo '[2/2] two'"),
                &mut |line| lines.push(line.to_owned()),
            )
            .unwrap();
            log.finish().unwrap();
        }
        assert_eq!(lines, ["[1/2] one", "oops", "[2/2] two"]);
        let text = fs::read_to_string(&path).unwrap();
        assert!(text.starts_with("$ sh -c "), "{text}");
        assert!(text.ends_with("[1/2] one\noops\n[2/2] two\n"), "{text}");
    }

    #[test]
    fn a_failed_step_quotes_the_end_of_the_log() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("build.log");
        let mut log = Log::new(File::create(&path).unwrap(), &path);
        let script = "for i in $(seq 1 30); do echo line $i; done; echo 'error: boom' >&2; exit 2";
        let message = log
            .stream("build", shell(script), &mut |_| {})
            .unwrap_err()
            .to_string();
        assert!(
            message.starts_with("cmake build failed (exit status: 2)"),
            "{message}"
        );
        assert!(message.contains(&path.display().to_string()), "{message}");
        assert!(message.ends_with("line 30\nerror: boom"), "{message}");
        assert!(!message.contains("line 11\n"), "{message}");
        assert!(message.contains("line 12\n"), "{message}");
    }

    #[test]
    fn a_line_without_a_newline_at_the_end_is_kept() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("build.log");
        let mut lines = Vec::new();
        let mut log = Log::new(Vec::new(), &path);
        log.stream("build", shell("printf 'a\\r\\nb'"), &mut |line| {
            lines.push(line.to_owned())
        })
        .unwrap();
        assert_eq!(lines, ["a", "b"]);
    }

    #[test]
    fn the_build_log_is_created_in_a_directory_for_the_user_only() {
        let dir = tempfile::tempdir().unwrap();
        let state = dir.path().join("state");
        let compiler = Cmake::new(dir.path().join("work"), state.join("logs"));
        compiler.create_log("llama.cpp").unwrap();
        assert!(compiler.log_file("llama.cpp").is_file());
        assert_eq!(files::mode(&state), 0o700);
        assert_eq!(files::mode(&state.join("logs")), 0o700);
    }

    #[test]
    fn compile_configures_a_fresh_tree_then_builds_the_target_into_the_log() {
        let dir = tempfile::tempdir().unwrap();
        let (cmake, calls) = (dir.path().join("cmake"), dir.path().join("calls"));
        // Builds the artifact, with one step, when asked for `--build <tree>`.
        let body = format!(
            "echo \"$*\" >> {}\nif [ \"$1\" = --build ]; then\n  \
             echo '[1/1] Linking CXX executable bin/llama-server'\n  \
             mkdir -p \"$2/bin\" && touch \"$2/bin/llama-server\"\nfi",
            calls.display()
        );
        script(&cmake, &body);
        let compiler = Cmake {
            program: cmake.clone(),
            ..Cmake::new(dir.path().join("work"), dir.path().join("logs"))
        };
        let tree = dir.path().join("work/llama.cpp");
        fs::create_dir_all(&tree).unwrap();
        fs::write(tree.join("CMakeCache.txt"), "from the last build").unwrap();
        let flags = BTreeMap::from([("GGML_CUDA".to_owned(), "ON".to_owned())]);
        let job = CompileJob {
            project: "llama.cpp",
            source: Path::new("/data/sources/llama.cpp"),
            flags: &flags,
            target: "llama-server",
            artifact: "bin/llama-server",
            jobs: Some(3),
        };

        let mut steps = Vec::new();
        let compiled = compiler
            .compile(&job, &mut |step| steps.push(step))
            .unwrap();
        let log = dir.path().join("logs/build-llama.cpp.log");
        assert_eq!(
            compiled,
            Compiled {
                artifact: tree.join("bin/llama-server"),
                log: log.clone(),
            }
        );
        assert_eq!(
            steps,
            [
                Progress::Configuring,
                Progress::Building { done: 1, total: 1 }
            ]
        );
        assert!(!tree.join("CMakeCache.txt").exists(), "a fresh build tree");
        let configure = [
            "-S",
            "/data/sources/llama.cpp",
            "-B",
            &tree.display().to_string(),
        ]
        .into_iter()
        .chain(generator())
        .chain(["-DGGML_CUDA=ON"])
        .collect::<Vec<_>>()
        .join(" ");
        let build = format!(
            "--build {} --target llama-server --parallel 3",
            tree.display()
        );
        assert_eq!(
            fs::read_to_string(&calls).unwrap(),
            format!("{configure}\n{build}\n")
        );
        assert_eq!(
            fs::read_to_string(&log).unwrap(),
            format!(
                "$ {cmake} {configure}\n$ {cmake} {build}\n\
                 [1/1] Linking CXX executable bin/llama-server\n",
                cmake = cmake.display()
            )
        );

        // A build that succeeds without its artifact.
        script(&cmake, "true");
        let message = compiler.compile(&job, &mut |_| {}).unwrap_err().to_string();
        assert_eq!(
            message,
            format!(
                "the build succeeded but {} is missing; see {}",
                tree.join("bin/llama-server").display(),
                log.display()
            )
        );
    }

    #[test]
    fn cleaning_a_missing_build_tree_is_not_an_error() {
        let dir = tempfile::tempdir().unwrap();
        let compiler = Cmake::new(dir.path().join("work"), dir.path().join("logs"));
        compiler.clean("llama.cpp").unwrap();
        fs::create_dir_all(dir.path().join("work/llama.cpp/bin")).unwrap();
        compiler.clean("llama.cpp").unwrap();
        assert!(!dir.path().join("work/llama.cpp").exists());
    }

    #[test]
    fn a_failed_configuration_builds_nothing() {
        let dir = tempfile::tempdir().unwrap();
        let (cmake, calls) = (dir.path().join("cmake"), dir.path().join("calls"));
        script(
            &cmake,
            &format!(
                "echo \"$1\" >> {}\nif [ \"$1\" != --build ]; then echo 'CMake Error: no CUDA' >&2; exit 1; fi",
                calls.display()
            ),
        );
        let compiler = Cmake {
            program: cmake,
            ..Cmake::new(dir.path().join("work"), dir.path().join("logs"))
        };
        let flags = BTreeMap::new();
        let job = CompileJob {
            project: "llama.cpp",
            source: Path::new("/data/sources/llama.cpp"),
            flags: &flags,
            target: "llama-server",
            artifact: "bin/llama-server",
            jobs: Some(1),
        };
        let message = compiler.compile(&job, &mut |_| {}).unwrap_err().to_string();
        assert!(
            message.starts_with("cmake configure failed (exit status: 1)"),
            "{message}"
        );
        assert!(message.ends_with("CMake Error: no CUDA"), "{message}");
        assert_eq!(fs::read_to_string(&calls).unwrap(), "-S\n");
    }
}
