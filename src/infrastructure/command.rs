//! Runs an external command and collects its output: C locale, no input.
//! Probes run with a bounded time; other commands take as long as their work.

use std::ffi::OsStr;
use std::io::{self, Read};
use std::path::Path;
use std::process::{Child, Command, ExitStatus, Stdio};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError};
use std::thread;
use std::time::{Duration, Instant};

use anyhow::{Result, anyhow};

use crate::application::ports::{ProgramOutput, Programs};
use crate::domain::machine::Tool;

/// A probe command that has not exited after this long is killed.
const TIMEOUT: Duration = Duration::from_secs(15);
/// A program run by a use case, such as a new build's smoke test, gets more
/// time: a CUDA program initialises the driver first.
const PROGRAM_TIMEOUT: Duration = Duration::from_secs(60);
/// How often a child that closed its output is checked for its exit: once
/// its pipes close, it is exiting, or runs on without them.
const EXIT_POLL: Duration = Duration::from_millis(1);

/// What a command printed, and whether it succeeded.
#[derive(Debug)]
pub struct Output {
    pub success: bool,
    pub stdout: String,
    pub stderr: String,
}

impl Output {
    /// The first non-empty line of stderr that is not a log line, or the
    /// first log line, or a generic message: why the command failed, in one
    /// line.
    pub fn failure(&self) -> String {
        let lines: Vec<&str> = self
            .stderr
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty())
            .collect();
        lines
            .iter()
            .find(|line| !is_log_line(line))
            .or(lines.first())
            .copied()
            .unwrap_or("exited with an error and printed nothing")
            .to_owned()
    }
}

/// Whether `line` is one of the log lines podman and nvidia-ctk print with
/// logrus before their error, such as `time="…" level=error msg="…"`.
fn is_log_line(line: &str) -> bool {
    line.starts_with("time=\"") && line.contains(" level=")
}

#[derive(Debug)]
pub enum RunError {
    /// The program is not installed.
    NotFound,
    Failed(String),
}

/// The output of a command that ran, or an error: `missing` when the
/// program is not installed, the reason it could not run otherwise.
pub fn checked(result: Result<Output, RunError>, missing: &str) -> Result<Output> {
    result.map_err(|err| match err {
        RunError::NotFound => anyhow!("{missing}"),
        RunError::Failed(message) => anyhow!(message),
    })
}

/// The error of a missing `tool`, with its install command.
pub fn not_found(tool: Tool) -> String {
    format!("{} not found; run `{}`", tool.command(), tool.install())
}

/// Runs a probe: `program` with `args`, killed if it has not exited within
/// 15 s.
pub fn run(program: impl AsRef<OsStr>, args: &[&str]) -> Result<Output, RunError> {
    let mut command = Command::new(program);
    command.args(args);
    execute(command, Some(TIMEOUT))
}

/// Runs `command`, prepared by the caller with its arguments, directory and
/// environment, and collects its output; `timeout` bounds its run time and
/// the time a process it leaves keeps its output open. The C locale keeps
/// the output parseable whatever the user's language.
pub fn execute(mut command: Command, timeout: Option<Duration>) -> Result<Output, RunError> {
    let program = command.get_program().to_string_lossy().into_owned();
    command
        .env("LC_ALL", "C")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let mut child = spawn(&mut command).map_err(|err| match err.kind() {
        io::ErrorKind::NotFound => RunError::NotFound,
        _ => RunError::Failed(format!("{program}: {err}")),
    })?;

    let stdout = child.stdout.take().map(drain);
    let stderr = child.stderr.take().map(drain);

    let Some(timeout) = timeout else {
        let status = child
            .wait()
            .map_err(|err| RunError::Failed(err.to_string()))?;
        return Ok(Output {
            success: status.success(),
            stdout: collect(stdout, None).unwrap_or_default(),
            stderr: collect(stderr, None).unwrap_or_default(),
        });
    };
    let deadline = Instant::now() + timeout;
    let finished = match (
        collect(stdout, Some(deadline)),
        collect(stderr, Some(deadline)),
    ) {
        (Some(stdout), Some(stderr)) => exit_by(&mut child, deadline)
            .map_err(|err| RunError::Failed(err.to_string()))?
            .map(|status| Output {
                success: status.success(),
                stdout,
                stderr,
            }),
        _ => None,
    };
    finished.ok_or_else(|| {
        // Best effort: the error is what matters. A process the child left
        // keeps its output open, and the thread reading it, until it ends.
        let _ = child.kill();
        let _ = child.wait();
        RunError::Failed(format!(
            "{program} did not answer within {} s",
            timeout.as_secs()
        ))
    })
}

/// The exit status of `child`, once its output is closed; `None` when it
/// still runs at `deadline`.
fn exit_by(child: &mut Child, deadline: Instant) -> io::Result<Option<ExitStatus>> {
    loop {
        if let Some(status) = child.try_wait()? {
            return Ok(Some(status));
        }
        if Instant::now() >= deadline {
            return Ok(None);
        }
        thread::sleep(EXIT_POLL);
    }
}

/// Starts `command`; every child process of ai-ctl starts here. In the
/// crate's tests, a start waits while a test holds [`exclusive`].
pub fn spawn(command: &mut Command) -> io::Result<Child> {
    #[cfg(test)]
    let _spawning = SPAWNS
        .read()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    command.spawn()
}

/// Held for reading while a child starts, until it runs its program.
#[cfg(test)]
static SPAWNS: std::sync::RwLock<()> = std::sync::RwLock::new(());

/// For the crate's tests: holds every [`spawn`] back until the guard is
/// dropped. Tests run in parallel threads, and a child forked by one shares
/// every file another holds open until it runs its program: a script being
/// written is then busy (`ETXTBSY`), and a lock or a port just released is
/// still held. A test writing an executable, or releasing a lock or a socket,
/// holds this guard.
#[cfg(test)]
pub fn exclusive() -> std::sync::RwLockWriteGuard<'static, ()> {
    SPAWNS
        .write()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

/// For the crate's tests: writes the executable script `path`, `#!/bin/sh`
/// then `body`, under [`exclusive`].
#[cfg(test)]
pub fn script(path: &Path, body: &str) {
    use std::os::unix::fs::PermissionsExt;
    let _exclusive = exclusive();
    std::fs::write(path, format!("#!/bin/sh\n{body}\n")).unwrap();
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o755)).unwrap();
}

/// Runs programs for the use cases, with a bounded time.
pub struct SystemPrograms;

impl Programs for SystemPrograms {
    fn run(&self, program: &Path, args: &[&str]) -> Result<ProgramOutput> {
        let mut command = Command::new(program);
        command.args(args);
        let missing = format!("{} not found", program.display());
        let output = checked(execute(command, Some(PROGRAM_TIMEOUT)), &missing)?;
        Ok(ProgramOutput {
            success: output.success,
            stdout: output.stdout,
            stderr: output.stderr,
        })
    }
}

/// Reads `pipe` to its end on a thread of its own, which sends the text.
fn drain(mut pipe: impl Read + Send + 'static) -> Receiver<String> {
    let (text, received) = mpsc::channel();
    thread::spawn(move || {
        let mut bytes = Vec::new();
        // A read error leaves what was read so far, which the parsers reject.
        let _ = pipe.read_to_end(&mut bytes);
        let _ = text.send(String::from_utf8_lossy(&bytes).into_owned());
    });
    received
}

/// What `reader` read, once its pipe is closed: waited for until
/// `deadline`, `None` when it is still open then, or for as long as it
/// takes without one.
fn collect(reader: Option<Receiver<String>>, deadline: Option<Instant>) -> Option<String> {
    let Some(reader) = reader else {
        return Some(String::new());
    };
    let received = match deadline {
        Some(deadline) => {
            match reader.recv_timeout(deadline.saturating_duration_since(Instant::now())) {
                Err(RecvTimeoutError::Timeout) => return None,
                received => received.ok(),
            }
        }
        None => reader.recv().ok(),
    };
    Some(received.unwrap_or_default())
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;

    #[test]
    fn a_command_error_becomes_the_callers_message() {
        assert_eq!(
            not_found(Tool::Git),
            "git not found; run `sudo dnf install git`"
        );
        let missing = checked(Err(RunError::NotFound), "git not found").unwrap_err();
        assert_eq!(missing.to_string(), "git not found");
        let failed = RunError::Failed("git: Permission denied (os error 13)".into());
        let failed = checked(Err(failed), "git not found").unwrap_err();
        assert_eq!(failed.to_string(), "git: Permission denied (os error 13)");
        let ran = Output {
            success: false,
            stdout: String::new(),
            stderr: "fatal: not a git repository".into(),
        };
        // A command that ran is the caller's to judge, failed or not.
        assert!(!checked(Ok(ran), "git not found").unwrap().success);
    }

    #[test]
    fn a_missing_program_is_not_found() {
        assert!(matches!(
            run("ai-ctl-no-such-program", &[]),
            Err(RunError::NotFound)
        ));
    }

    #[test]
    fn output_and_status_are_collected() {
        let output = run("sh", &["-c", "echo out; echo err >&2; exit 3"]).unwrap();
        assert!(!output.success);
        assert_eq!(output.stdout, "out\n");
        assert_eq!(output.failure(), "err");
    }

    #[test]
    fn the_locale_is_c() {
        let output = run("sh", &["-c", "echo $LC_ALL"]).unwrap();
        assert_eq!(output.stdout.trim(), "C");
    }

    #[test]
    fn a_prepared_command_keeps_its_directory_and_environment() {
        let dir = tempfile::tempdir().unwrap();
        let mut command = Command::new("sh");
        command
            .args(["-c", "pwd; echo $GREETING $LC_ALL"])
            .current_dir(dir.path())
            .env("GREETING", "hello");
        let output = execute(command, None).unwrap();
        let canonical = dir.path().canonicalize().unwrap();
        assert_eq!(output.stdout, format!("{}\nhello C\n", canonical.display()));
    }

    #[test]
    fn a_command_past_its_timeout_is_killed() {
        let mut command = Command::new("sleep");
        command.arg("5");
        match execute(command, Some(Duration::ZERO)) {
            Err(RunError::Failed(message)) => {
                assert!(message.contains("did not answer"), "{message}")
            }
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn a_grandchild_holding_the_pipe_does_not_outlive_the_timeout() {
        let dir = tempfile::tempdir().unwrap();
        let pid = dir.path().join("pid");
        // The shell exits at once; the `sleep` it leaves keeps its stdout.
        let mut command = Command::new("sh");
        command.args(["-c", "sleep 4 & echo $! > \"$0\"", &pid.to_string_lossy()]);
        let result = execute(command, Some(Duration::from_secs(1)));
        let _ = Command::new("kill")
            .arg(fs::read_to_string(&pid).unwrap().trim())
            .status();
        match result {
            Err(RunError::Failed(message)) => {
                assert_eq!(message, "sh did not answer within 1 s");
            }
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn a_child_that_closes_its_output_and_runs_on_is_killed_at_its_timeout() {
        let mut command = Command::new("sh");
        command.args(["-c", "exec >&- 2>&-; exec sleep 4"]);
        match execute(command, Some(Duration::from_secs(1))) {
            Err(RunError::Failed(message)) => {
                assert_eq!(message, "sh did not answer within 1 s");
            }
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn programs_report_their_output_and_absence() {
        let output = SystemPrograms
            .run(Path::new("sh"), &["-c", "echo out; exit 4"])
            .unwrap();
        assert!(!output.success);
        assert_eq!(output.stdout, "out\n");
        let message = SystemPrograms
            .run(Path::new("/nonexistent/llama-server"), &["--version"])
            .unwrap_err()
            .to_string();
        assert_eq!(message, "/nonexistent/llama-server not found");
    }

    fn failed(stderr: &str) -> Output {
        Output {
            success: false,
            stdout: String::new(),
            stderr: stderr.to_owned(),
        }
    }

    #[test]
    fn a_failure_is_the_error_after_the_log_lines() {
        // podman logs with logrus, then states the error.
        let stderr = include_str!("../../tests/fixtures/podman-5.8.7-ps-bogus-storage-driver.txt");
        let failure = failed(stderr).failure();
        assert!(
            failure.starts_with("Error: configure storage: failed to GetDriver graph bogus /"),
            "{failure}"
        );
        assert!(
            failure.ends_with("/graph: driver not supported"),
            "{failure}"
        );
        // Log lines alone: the first one.
        let logged = "time=\"2026-10-16T21:26:42+02:00\" level=error msg=\"no driver\"\n";
        assert_eq!(failed(logged).failure(), logged.trim());
    }

    #[test]
    fn a_silent_failure_still_explains_itself() {
        let output = run("sh", &["-c", "exit 1"]).unwrap();
        assert_eq!(output.failure(), "exited with an error and printed nothing");
    }
}
