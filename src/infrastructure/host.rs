//! The host probes — its toolchain, CDI and power state — from external
//! commands, sysfs and D-Bus, all read-only.

use std::collections::HashSet;
use std::ffi::OsString;
use std::fs;
use std::io::{self, Read};
use std::path::{Path, PathBuf};

use anyhow::{Context, ensure};

use crate::application::ports::{CdiProbe, PowerProbe, ToolchainProbe};
use crate::domain::machine::{
    CdiSpec, CudaRepository, CudaToolkit, DRIVER_PACKAGES, HostCompiler, Probed, Tool, Version,
    toolkit_meta_packages,
};

use super::command::{self, RunError};

/// Where the CUDA packages install each toolkit, as `cuda-<major>.<minor>`.
const TOOLKITS: &str = "/usr/local";
/// `nvcc` under [`TOOLKITS`], through the link to the default toolkit,
/// where it is looked for when it is not on the `PATH`.
const NVCC_FALLBACK: &str = "cuda/bin/nvcc";
/// `host_config.h` relative to a toolkit root, newest layout first.
const HOST_CONFIG: [&str; 2] = [
    "targets/x86_64-linux/include/crt/host_config.h",
    "include/crt/host_config.h",
];
/// The most of a model's text file read: a safetensors index of thousands
/// of tensors holds a few hundred KiB.
const MAX_TEXT: u64 = 16 << 20;
const POWER_SUPPLIES: &str = "/sys/class/power_supply";
const OS_RELEASE: &str = "/etc/os-release";
/// CDI's specification directories, by rising priority: a hand-written
/// specification goes in the first, the toolkit's `nvidia-cdi-refresh`
/// writes its own in the second.
const CDI_DIRS: [&str; 2] = ["/etc/cdi", "/var/run/cdi"];
/// The power profiles D-Bus service, object and interface, which both
/// `power-profiles-daemon` and `tuned-ppd` serve.
const POWER_PROFILES: [&str; 3] = [
    "org.freedesktop.UPower.PowerProfiles",
    "/org/freedesktop/UPower/PowerProfiles",
    "org.freedesktop.UPower.PowerProfiles",
];

/// Probes this machine.
pub struct SystemHost {
    path: Option<OsString>,
    power_supplies: PathBuf,
    os_release: PathBuf,
    cdi_dirs: Vec<PathBuf>,
    toolkits: PathBuf,
}

impl SystemHost {
    pub fn new() -> Self {
        Self {
            path: std::env::var_os("PATH"),
            power_supplies: PathBuf::from(POWER_SUPPLIES),
            os_release: PathBuf::from(OS_RELEASE),
            cdi_dirs: CDI_DIRS.iter().map(PathBuf::from).collect(),
            toolkits: PathBuf::from(TOOLKITS),
        }
    }

    fn path_dirs(&self) -> Vec<PathBuf> {
        self.path
            .as_deref()
            .map(|path| std::env::split_paths(path).collect())
            .unwrap_or_default()
    }

    /// `program` in the first directory of `PATH` that holds it.
    fn find(&self, program: &str) -> Option<PathBuf> {
        self.path_dirs()
            .into_iter()
            .map(|dir| dir.join(program))
            .find(|candidate| candidate.is_file())
    }

    /// The releases of the toolkits under [`TOOLKITS`], from their
    /// directories' names; none when it cannot be read.
    fn installed_toolkits(&self) -> Vec<(u32, u32)> {
        let Ok(entries) = fs::read_dir(&self.toolkits) else {
            return Vec::new();
        };
        entries
            .filter_map(|entry| parse_toolkit_dir(entry.ok()?.file_name().to_str()?))
            .collect()
    }

    fn find_nvcc(&self) -> Option<PathBuf> {
        self.find("nvcc")
            .or_else(|| Some(self.toolkits.join(NVCC_FALLBACK)).filter(|path| path.is_file()))
    }
}

impl Default for SystemHost {
    fn default() -> Self {
        Self::new()
    }
}

impl ToolchainProbe for SystemHost {
    fn cuda_toolkit(&self) -> Probed<CudaToolkit> {
        let Some(nvcc) = self.find_nvcc() else {
            return Probed::Missing;
        };
        let output = match command::run(&nvcc, &["--version"]) {
            Ok(output) if output.success => output,
            Ok(output) => return Probed::Failed(output.failure()),
            Err(err) => return failed_run(err),
        };
        let Some(version) = parse_nvcc_version(&output.stdout) else {
            return Probed::Failed("unexpected `nvcc --version` output".to_owned());
        };
        // `/usr/local/cuda/bin/nvcc` resolves to `/usr/local/cuda-13.2/bin/nvcc`,
        // whose grandparent is the toolkit root.
        let nvcc = fs::canonicalize(&nvcc).unwrap_or(nvcc);
        let max_gcc_major = match nvcc.parent().and_then(Path::parent) {
            Some(root) => read_max_gcc_major(root),
            None => Probed::Missing,
        };
        Probed::Found(CudaToolkit {
            version,
            nvcc,
            max_gcc_major,
        })
    }

    fn host_compilers(&self) -> Probed<Vec<HostCompiler>> {
        let mut candidates: Vec<(String, PathBuf)> = Vec::new();
        for dir in self.path_dirs() {
            let Ok(entries) = fs::read_dir(&dir) else {
                continue;
            };
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().into_owned();
                // ccache's `g++` comes first on Fedora's `PATH`
                // (`/usr/lib64/ccache`): the compiler is the one behind it.
                if is_gxx(&name)
                    && !is_compiler_cache(&entry.path())
                    && !candidates.iter().any(|(known, _)| *known == name)
                {
                    candidates.push((name, entry.path()));
                }
            }
        }
        candidates.sort_by_key(|(name, _)| gxx_order(name));

        let mut compilers = Vec::new();
        let mut seen = HashSet::new();
        let mut first_error = None;
        for (command, path) in candidates {
            // `g++` may be a link to `g++-15`: report the binary once.
            if !seen.insert(fs::canonicalize(&path).unwrap_or_else(|_| path.clone())) {
                continue;
            }
            match command::run(&path, &["-dumpfullversion"]) {
                Ok(output) if output.success => match Version::parse(&output.stdout) {
                    Some(version) => compilers.push(HostCompiler {
                        command,
                        path,
                        version,
                    }),
                    None => {
                        first_error.get_or_insert(format!("{command}: unexpected version output"));
                    }
                },
                Ok(output) => {
                    first_error.get_or_insert(format!("{command}: {}", output.failure()));
                }
                Err(err) => {
                    first_error.get_or_insert(format!("{command}: {}", describe(err)));
                }
            }
        }

        match (compilers.is_empty(), first_error) {
            (false, _) => Probed::Found(compilers),
            (true, Some(error)) => Probed::Failed(error),
            (true, None) => Probed::Missing,
        }
    }

    fn tool_version(&self, tool: Tool) -> Probed<Version> {
        match command::run(tool.command(), &["--version"]) {
            Ok(output) if output.success => match parse_version_output(&output.stdout) {
                Some(version) => Probed::Found(version),
                None => Probed::Failed(format!("unexpected `{} --version` output", tool.command())),
            },
            Ok(output) => Probed::Failed(output.failure()),
            Err(err) => failed_run(err),
        }
    }

    fn is_file(&self, path: &Path) -> bool {
        path.is_file()
    }

    fn is_dir(&self, path: &Path) -> bool {
        path.is_dir()
    }

    fn entries(&self, path: &Path) -> anyhow::Result<Vec<String>> {
        let read = || -> io::Result<Vec<String>> {
            fs::read_dir(path)?
                .map(|entry| Ok(entry?.file_name().to_string_lossy().into_owned()))
                .collect()
        };
        read().with_context(|| format!("cannot read {}", path.display()))
    }

    fn read_text(&self, path: &Path) -> anyhow::Result<String> {
        read_capped(path, MAX_TEXT)
    }

    fn fedora_release(&self) -> Probed<u32> {
        match fs::read_to_string(&self.os_release) {
            Ok(text) => parse_fedora_release(&text).map_or(Probed::Missing, Probed::Found),
            Err(err) if err.kind() == io::ErrorKind::NotFound => Probed::Missing,
            Err(err) => Probed::Failed(format!("{}: {err}", self.os_release.display())),
        }
    }

    fn cuda_repositories(&self) -> Probed<Vec<CudaRepository>> {
        let ids = match command::run("dnf", &["repolist", "--enabled"]) {
            Ok(output) if output.success => parse_repolist(&output.stdout),
            Ok(output) => return Probed::Failed(output.failure()),
            Err(err) => return failed_run(err),
        };
        let mut repositories = Vec::new();
        for id in ids.into_iter().filter(|id| id.starts_with("cuda-")) {
            // The cache only: doctor stays fast and offline. The query sees
            // the repository's exclusions, as `dnf install` would.
            let mut args = vec![
                "repoquery",
                "--cacheonly",
                "--quiet",
                "--repo",
                &id,
                "--qf",
                "%{name}\\n",
            ];
            args.extend(DRIVER_PACKAGES);
            match command::run("dnf", &args) {
                Ok(output) if output.success => repositories.push(CudaRepository {
                    driver_packages: parse_package_names(&output.stdout),
                    id,
                }),
                Ok(output) => return Probed::Failed(format!("{id}: {}", output.failure())),
                Err(err) => return failed_run(err),
            }
        }
        Probed::Found(repositories)
    }

    fn toolkit_packages(&self, releases: &[(u32, u32)]) -> Probed<Vec<String>> {
        let Some(rpm) = self.find("rpm") else {
            return Probed::Missing;
        };
        let mut releases = releases.to_vec();
        releases.extend(self.installed_toolkits());
        let names = toolkit_meta_packages(&releases);
        // Exact names go through rpm's index, where a pattern loads every
        // header; a name listing needs no header's digest or signature
        // checked. Each name not installed adds 1 to the exit status.
        let mut args = vec!["-q", "--nodigest", "--nosignature", "--qf", "%{NAME}\\n"];
        args.extend(names.iter().map(String::as_str));
        match command::run(&rpm, &args) {
            Ok(output) if output.success || output.stderr.trim().is_empty() => {
                Probed::Found(parse_installed(&output.stdout, &names))
            }
            Ok(output) => Probed::Failed(output.failure()),
            Err(err) => failed_run(err),
        }
    }
}

impl CdiProbe for SystemHost {
    fn cdi_devices(&self) -> Probed<Vec<String>> {
        match command::run("nvidia-ctk", &["cdi", "list"]) {
            Ok(output) if output.success => Probed::Found(parse_cdi_list(&output.stdout)),
            Ok(output) => Probed::Failed(output.failure()),
            Err(err) => failed_run(err),
        }
    }

    fn cdi_specs(&self) -> Probed<Vec<CdiSpec>> {
        let mut specs = Vec::new();
        for dir in &self.cdi_dirs {
            let entries = match fs::read_dir(dir) {
                Ok(entries) => entries,
                Err(err) if err.kind() == io::ErrorKind::NotFound => continue,
                Err(err) => return Probed::Failed(format!("{}: {err}", dir.display())),
            };
            // CDI reads `.yaml` and `.json` files.
            let mut paths: Vec<PathBuf> = entries
                .filter_map(|entry| entry.ok().map(|entry| entry.path()))
                .filter(|path| {
                    path.extension()
                        .is_some_and(|ext| ext == "yaml" || ext == "json")
                })
                .collect();
            paths.sort();
            for path in paths {
                match fs::read_to_string(&path) {
                    Ok(text) if text.contains(NVIDIA_GPU_KIND) => specs.push(CdiSpec {
                        driver: parse_cdi_driver(&text),
                        missing: parse_cdi_host_paths(&text)
                            .into_iter()
                            .filter(|path| is_gone(path))
                            .collect(),
                        path,
                    }),
                    Ok(_) => {}
                    Err(err) => return Probed::Failed(format!("{}: {err}", path.display())),
                }
            }
        }
        if specs.is_empty() {
            Probed::Missing
        } else {
            Probed::Found(specs)
        }
    }
}

impl PowerProbe for SystemHost {
    fn ac_power(&self) -> Probed<bool> {
        ac_online(&self.power_supplies)
    }

    fn power_profile(&self) -> Probed<String> {
        let Some(busctl) = self.find("busctl") else {
            return Probed::Missing;
        };
        let [service, object, interface] = POWER_PROFILES;
        let args = ["get-property", service, object, interface, "ActiveProfile"];
        match command::run(&busctl, &args) {
            Ok(output) if output.success => match parse_busctl_string(&output.stdout) {
                Some(profile) => Probed::Found(profile),
                None => Probed::Failed("unexpected busctl output".to_owned()),
            },
            // No daemon serves power profiles: nothing to report, not a failure.
            Ok(output) if is_service_absent(&output.stderr) => Probed::Missing,
            Ok(output) => Probed::Failed(output.failure()),
            Err(err) => failed_run(err),
        }
    }
}

fn failed_run<T>(err: RunError) -> Probed<T> {
    match err {
        RunError::NotFound => Probed::Missing,
        RunError::Failed(message) => Probed::Failed(message),
    }
}

fn describe(err: RunError) -> String {
    match err {
        RunError::NotFound => "not found".to_owned(),
        RunError::Failed(message) => message,
    }
}

/// The text of `path`, refused when it holds more than `limit` bytes.
fn read_capped(path: &Path, limit: u64) -> anyhow::Result<String> {
    let mut bytes = Vec::new();
    fs::File::open(path)
        .and_then(|file| file.take(limit + 1).read_to_end(&mut bytes))
        .with_context(|| format!("cannot read {}", path.display()))?;
    // The size before the text: the limit may cut a character in two.
    ensure!(
        bytes.len() as u64 <= limit,
        "{} holds more than {limit} bytes",
        path.display()
    );
    String::from_utf8(bytes).with_context(|| format!("cannot read {}", path.display()))
}

/// Reads the newest GCC major version the toolkit accepts.
fn read_max_gcc_major(root: &Path) -> Probed<u32> {
    let Some(header) = HOST_CONFIG
        .iter()
        .map(|relative| root.join(relative))
        .find(|path| path.is_file())
    else {
        return Probed::Missing;
    };
    match fs::read_to_string(&header) {
        Ok(text) => match parse_max_gcc_major(&text) {
            Some(major) => Probed::Found(major),
            None => Probed::Failed(format!("no GCC limit in {}", header.display())),
        },
        Err(err) => Probed::Failed(format!("{}: {err}", header.display())),
    }
}

/// `Cuda compilation tools, release 13.2, V13.2.86` → 13.2.86.
pub fn parse_nvcc_version(output: &str) -> Option<Version> {
    let line = output.lines().find(|line| line.contains("release"))?;
    let full = line
        .split_whitespace()
        .find_map(|word| word.strip_prefix('V'))
        .and_then(Version::parse);
    full.or_else(|| {
        let release = line.split("release").nth(1)?.split_whitespace().next()?;
        Version::parse(release.trim_end_matches(','))
    })
}

/// The `N` of the first `#if __GNUC__ > N` in `crt/host_config.h`: nvcc
/// rejects any GCC whose major version is above it.
pub fn parse_max_gcc_major(header: &str) -> Option<u32> {
    header.lines().find_map(|line| {
        let words: Vec<&str> = line.split_whitespace().collect();
        match words.as_slice() {
            ["#if", "__GNUC__", ">", major] => major.parse().ok(),
            _ => None,
        }
    })
}

/// The version after the word `version` on the first line that has one, as
/// printed by `cmake`, `git`, `podman` and `nvidia-ctk`.
pub fn parse_version_output(output: &str) -> Option<Version> {
    output.lines().find_map(|line| {
        let mut words = line.split_whitespace();
        words.find(|word| *word == "version")?;
        Version::parse(words.next()?)
    })
}

/// Whether nothing is at `path`, a link included that names nothing, as
/// podman finds it when it mounts the path; a path that cannot be looked at
/// is not counted.
fn is_gone(path: &Path) -> bool {
    matches!(fs::metadata(path), Err(err) if err.kind() == io::ErrorKind::NotFound)
}

/// CDI device names, one per line, such as `nvidia.com/gpu=all`.
pub fn parse_cdi_list(output: &str) -> Vec<String> {
    output
        .lines()
        .map(str::trim)
        .filter(|line| {
            line.split_once('=')
                .is_some_and(|(kind, _)| kind.contains('/') && !kind.contains(' '))
        })
        .map(str::to_owned)
        .collect()
}

/// The kind of the CDI devices of NVIDIA's GPUs.
const NVIDIA_GPU_KIND: &str = "nvidia.com/gpu";

/// The driver version of the `libcuda.so.<version>` a CDI specification
/// mounts, such as `615.71.09`; the `libcuda.so.1` link names none.
pub fn parse_cdi_driver(text: &str) -> Option<String> {
    const LIBCUDA: &str = "libcuda.so.";
    text.match_indices(LIBCUDA).find_map(|(at, _)| {
        let rest = &text[at + LIBCUDA.len()..];
        let end = rest
            .find(|c: char| !c.is_ascii_digit() && c != '.')
            .unwrap_or(rest.len());
        let version = rest[..end].trim_end_matches('.');
        version.contains('.').then(|| version.to_owned())
    })
}

/// The host paths a CDI specification mounts, its `hostPath` values, in
/// YAML as `nvidia-ctk` writes it or in JSON.
pub fn parse_cdi_host_paths(text: &str) -> Vec<PathBuf> {
    const KEY: &str = "hostPath";
    text.match_indices(KEY)
        .filter_map(|(at, _)| {
            let rest = &text[at + KEY.len()..];
            let rest = rest.strip_prefix('"').unwrap_or(rest);
            let value = rest.trim_start().strip_prefix(':')?.trim_start();
            let value = match value.strip_prefix('"') {
                Some(quoted) => &quoted[..quoted.find('"')?],
                None => value.split_whitespace().next()?,
            };
            (!value.is_empty()).then(|| PathBuf::from(value))
        })
        .collect()
}

/// The `VERSION_ID` of `/etc/os-release` when its `ID` is `fedora`.
pub fn parse_fedora_release(text: &str) -> Option<u32> {
    let field = |name: &str| {
        text.lines().find_map(|line| {
            let value = line.strip_prefix(name)?.strip_prefix('=')?;
            Some(value.trim().trim_matches(['"', '\'']))
        })
    };
    if field("ID")? != "fedora" {
        return None;
    }
    field("VERSION_ID")?.parse().ok()
}

/// The repository ids of `dnf repolist`: the first word of each line after
/// the header.
pub fn parse_repolist(output: &str) -> Vec<String> {
    output
        .lines()
        .skip_while(|line| !line.starts_with("repo id"))
        .skip(1)
        .filter_map(|line| line.split_whitespace().next())
        .map(str::to_owned)
        .collect()
}

/// Package names, one per line, sorted and without duplicates: a package
/// available in several versions is listed once.
/// The names of `asked` that `rpm -q` printed alone on a line, as it
/// prints an installed package's name with `--qf '%{NAME}\\n'`; the line of
/// a package not installed is left out, whatever its wording.
pub fn parse_installed(output: &str, asked: &[String]) -> Vec<String> {
    output
        .lines()
        .map(str::trim)
        .filter(|line| asked.iter().any(|name| name == line))
        .map(str::to_owned)
        .collect()
}

/// `cuda-13.2` → `(13, 2)`; `None` for a name with no minor, such as the
/// `cuda-13` link.
fn parse_toolkit_dir(name: &str) -> Option<(u32, u32)> {
    let (major, minor) = name.strip_prefix("cuda-")?.split_once('.')?;
    Some((major.parse().ok()?, minor.parse().ok()?))
}

pub fn parse_package_names(output: &str) -> Vec<String> {
    let mut names: Vec<String> = output
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(str::to_owned)
        .collect();
    names.sort();
    names.dedup();
    names
}

/// `s "performance"` → `performance`.
pub fn parse_busctl_string(output: &str) -> Option<String> {
    let value = output.trim().strip_prefix("s ")?;
    let value = value.strip_prefix('"')?.strip_suffix('"')?;
    Some(value.to_owned())
}

/// True when busctl failed because no service provides the name.
fn is_service_absent(stderr: &str) -> bool {
    stderr.contains("not activatable") || stderr.contains("not provided by any .service")
}

/// True when `path` resolves to a compiler cache such as ccache, which
/// stands in for a compiler under the compiler's name.
fn is_compiler_cache(path: &Path) -> bool {
    fs::canonicalize(path).is_ok_and(|target| {
        target
            .file_name()
            .is_some_and(|name| name == "ccache" || name == "sccache")
    })
}

fn is_gxx(name: &str) -> bool {
    name == "g++"
        || name
            .strip_prefix("g++-")
            .is_some_and(|major| !major.is_empty() && major.bytes().all(|b| b.is_ascii_digit()))
}

/// `g++` first, then `g++-N` from the newest major down.
fn gxx_order(name: &str) -> (bool, std::cmp::Reverse<u32>) {
    let major = name
        .strip_prefix("g++-")
        .and_then(|major| major.parse().ok())
        .unwrap_or(0);
    (name != "g++", std::cmp::Reverse(major))
}

/// Whether an external supply (mains, USB-C…) is online, from the
/// `type` and `online` attributes under `/sys/class/power_supply`.
pub fn ac_online(root: &Path) -> Probed<bool> {
    let entries = match fs::read_dir(root) {
        Ok(entries) => entries,
        Err(err) if err.kind() == io::ErrorKind::NotFound => return Probed::Missing,
        Err(err) => return Probed::Failed(format!("{}: {err}", root.display())),
    };
    let mut external = Vec::new();
    for entry in entries.flatten() {
        let read = |attribute: &str| fs::read_to_string(entry.path().join(attribute));
        let Ok(kind) = read("type") else { continue };
        if kind.trim() == "Battery" {
            continue;
        }
        if let Ok(online) = read("online") {
            external.push(online.trim() == "1");
        }
    }
    if external.is_empty() {
        Probed::Missing
    } else {
        Probed::Found(external.contains(&true))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::command::script;

    fn fixture(name: &str) -> String {
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures")
            .join(name);
        fs::read_to_string(&path).unwrap_or_else(|err| panic!("{}: {err}", path.display()))
    }

    #[test]
    fn nvcc_version_comes_from_the_release_line() {
        assert_eq!(
            parse_nvcc_version(&fixture("nvcc-13.2.txt")),
            Some(Version::new(13, 2, 86))
        );
    }

    #[test]
    fn nvcc_version_falls_back_to_the_release_number() {
        let output = "Cuda compilation tools, release 13.4\n";
        assert_eq!(parse_nvcc_version(output), Some(Version::new(13, 4, 0)));
    }

    #[test]
    fn nvcc_version_rejects_other_output() {
        assert_eq!(parse_nvcc_version(""), None);
        assert_eq!(parse_nvcc_version("nvcc fatal : No input files"), None);
    }

    #[test]
    fn gcc_limit_comes_from_host_config() {
        assert_eq!(
            parse_max_gcc_major(&fixture("host_config-13.2-excerpt.h")),
            Some(15)
        );
        assert_eq!(parse_max_gcc_major("#if defined(__GNUC__)\n#endif\n"), None);
    }

    #[test]
    fn gcc_dumpfullversion_parses() {
        assert_eq!(
            Version::parse(&fixture("gcc-15-dumpfullversion.txt")),
            Some(Version::new(15, 3, 1))
        );
    }

    #[test]
    fn tool_versions_parse() {
        let cases = [
            ("cmake-version.txt", Version::new(4, 3, 0)),
            ("git-version.txt", Version::new(2, 56, 0)),
            ("podman-version.txt", Version::new(5, 8, 7)),
            ("nvidia-ctk-version.txt", Version::new(1, 20, 1)),
        ];
        for (name, expected) in cases {
            assert_eq!(
                parse_version_output(&fixture(name)),
                Some(expected),
                "{name}"
            );
        }
        assert_eq!(parse_version_output("no number here"), None);
        assert_eq!(parse_version_output("version"), None);
    }

    #[test]
    fn cdi_list_keeps_device_names_only() {
        let mut output = fixture("nvidia-ctk-cdi-list.txt");
        output.insert_str(
            0,
            "time=\"2026-10-09T10:46:19+02:00\" level=info msg=\"Found 3 CDI devices\"\n",
        );
        assert_eq!(
            parse_cdi_list(&output),
            [
                "nvidia.com/gpu=0",
                "nvidia.com/gpu=GPU-00000000-0000-0000-0000-000000000000",
                "nvidia.com/gpu=all",
            ]
        );
        assert!(parse_cdi_list("").is_empty());
    }

    #[test]
    fn a_cdi_specification_names_the_driver_of_its_libcuda() {
        assert_eq!(
            parse_cdi_driver(&fixture("nvidia-ctk-1.20.1-cdi-spec-excerpt.yaml")).as_deref(),
            Some("615.71.09")
        );
        assert_eq!(
            parse_cdi_driver(&fixture("nvidia-ctk-1.20.1-cdi-spec-excerpt.json")).as_deref(),
            Some("615.78.08")
        );
        assert_eq!(
            parse_cdi_driver("- libcuda.so.1::/usr/lib64/libcuda.so"),
            None
        );
        assert_eq!(parse_cdi_driver("libcuda.so."), None);
        assert_eq!(parse_cdi_driver(""), None);
    }

    #[test]
    fn a_cdi_specification_lists_the_host_paths_it_mounts() {
        let paths = parse_cdi_host_paths(&fixture("nvidia-ctk-1.20.1-cdi-spec.yaml"));
        assert_eq!(paths.len(), 60);
        assert_eq!(paths[0], Path::new("/usr/bin/nvidia-cuda-mps-control"));
        assert_eq!(
            paths[15],
            Path::new("/usr/lib64/libnvidia-egl-wayland.so.1.1.24")
        );
        assert_eq!(
            paths[59],
            Path::new("/usr/share/glvnd/egl_vendor.d/10_nvidia.json")
        );
        assert_eq!(
            parse_cdi_host_paths(&fixture("nvidia-ctk-1.20.1-cdi-spec-excerpt.json")),
            [Path::new("/usr/lib64/libcuda.so.615.78.08")]
        );
        assert!(parse_cdi_host_paths("hostPath:\n").is_empty());
        assert!(parse_cdi_host_paths("").is_empty());
    }

    #[test]
    fn a_cdi_specification_names_the_host_paths_gone_from_disk() {
        let root = tempfile::tempdir().unwrap();
        let run = root.path().join("run");
        fs::create_dir_all(&run).unwrap();
        let present = root.path().join("libcuda.so.615.78.08");
        fs::write(&present, "").unwrap();
        let gone = root.path().join("libnvidia-egl-wayland.so.1.1.23");
        let dangling = root.path().join("libGLX_nvidia.so.0");
        std::os::unix::fs::symlink(root.path().join("nowhere"), &dangling).unwrap();
        let mounts: String = [&present, &gone, &dangling]
            .iter()
            .map(|path| format!("        - hostPath: {}\n", path.display()))
            .collect();
        fs::write(
            run.join("nvidia.yaml"),
            format!("kind: nvidia.com/gpu\ncontainerEdits:\n    mounts:\n{mounts}"),
        )
        .unwrap();
        let host = SystemHost {
            path: None,
            power_supplies: root.path().join("power"),
            os_release: root.path().join("os-release"),
            cdi_dirs: vec![run.clone()],
            toolkits: root.path().join("local"),
        };
        let Probed::Found(specs) = host.cdi_specs() else {
            panic!("one specification")
        };
        assert_eq!(specs[0].missing, [gone, dangling]);
    }

    #[test]
    fn a_models_index_is_read_whole() {
        let name = "fibo-2.0-dev-72415bd0-diffusion_pytorch_model.safetensors.index.json";
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures")
            .join(name);
        assert_eq!(SystemHost::new().read_text(&path).unwrap(), fixture(name));
    }

    #[test]
    fn a_directorys_entries_are_named_files_and_directories_alike() {
        use std::os::unix::ffi::OsStrExt;
        let root = tempfile::tempdir().unwrap();
        let host = SystemHost::new();
        assert_eq!(host.entries(root.path()).unwrap(), Vec::<String>::new());
        fs::create_dir(root.path().join(".cache")).unwrap();
        fs::write(root.path().join("model one.safetensors"), "").unwrap();
        fs::write(root.path().join(std::ffi::OsStr::from_bytes(b"x\xff")), "").unwrap();
        let mut names = host.entries(root.path()).unwrap();
        names.sort();
        assert_eq!(names, [".cache", "model one.safetensors", "x\u{fffd}"]);
        let missing = root.path().join("gone");
        assert!(
            format!("{:#}", host.entries(&missing).unwrap_err())
                .starts_with(&format!("cannot read {}: ", missing.display()))
        );
    }

    #[test]
    fn a_text_file_is_read_up_to_its_limit() {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("model.safetensors.index.json");
        fs::write(&path, "{\"weight_map\": {}}").unwrap();
        assert_eq!(read_capped(&path, 18).unwrap(), "{\"weight_map\": {}}");
        assert_eq!(
            format!("{:#}", read_capped(&path, 17).unwrap_err()),
            format!("{} holds more than 17 bytes", path.display())
        );

        fs::write(&path, b"\xff\xfe").unwrap();
        let invalid = format!("{:#}", read_capped(&path, 18).unwrap_err());
        assert!(
            invalid.starts_with(&format!("cannot read {}: ", path.display())),
            "{invalid}"
        );
        let missing = root.path().join("nope.json");
        assert_eq!(
            format!("{:#}", read_capped(&missing, 18).unwrap_err()),
            format!(
                "cannot read {}: No such file or directory (os error 2)",
                missing.display()
            )
        );
    }

    #[test]
    fn a_text_over_its_limit_is_told_so_when_the_limit_cuts_a_character() {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("model.safetensors.index.json");
        // Each width of a character, the limit after each of its bytes.
        for character in ["é", "€", "🦀"] {
            let text = format!("a{character}");
            fs::write(&path, &text).unwrap();
            let size = text.len() as u64;
            for limit in 1..size {
                assert_eq!(
                    format!("{:#}", read_capped(&path, limit).unwrap_err()),
                    format!("{} holds more than {limit} bytes", path.display()),
                    "{character} under a limit of {limit}"
                );
            }
            assert_eq!(read_capped(&path, size).unwrap(), text);
        }

        fs::write(&path, "").unwrap();
        assert_eq!(read_capped(&path, 0).unwrap(), "");
    }

    #[test]
    fn cdi_specifications_are_read_from_both_directories_in_order() {
        let root = tempfile::tempdir().unwrap();
        let (etc, run) = (root.path().join("etc"), root.path().join("run"));
        fs::create_dir_all(&etc).unwrap();
        fs::create_dir_all(&run).unwrap();
        let spec = fixture("nvidia-ctk-1.20.1-cdi-spec-excerpt.yaml");
        fs::write(run.join("nvidia.yaml"), &spec).unwrap();
        fs::write(
            etc.join("nvidia.json"),
            fixture("nvidia-ctk-1.20.1-cdi-spec-excerpt.json"),
        )
        .unwrap();
        fs::write(etc.join("other.yaml"), "kind: vendor.com/device\n").unwrap();
        fs::write(etc.join("nvidia.yaml.bak"), &spec).unwrap();
        let host = SystemHost {
            path: None,
            power_supplies: root.path().join("power"),
            os_release: root.path().join("os-release"),
            cdi_dirs: vec![etc.clone(), run.clone(), root.path().join("missing")],
            toolkits: root.path().join("local"),
        };
        // Which of their host paths are on disk depends on the machine.
        let Probed::Found(specs) = host.cdi_specs() else {
            panic!("two specifications")
        };
        let found: Vec<(&Path, Option<&str>)> = specs
            .iter()
            .map(|spec| (spec.path.as_path(), spec.driver.as_deref()))
            .collect();
        assert_eq!(
            found,
            [
                (etc.join("nvidia.json").as_path(), Some("615.78.08")),
                (run.join("nvidia.yaml").as_path(), Some("615.71.09")),
            ]
        );
        let host = SystemHost {
            cdi_dirs: vec![root.path().join("missing")],
            toolkits: root.path().join("local"),
            ..host
        };
        assert_eq!(host.cdi_specs(), Probed::Missing);
    }

    #[test]
    fn busctl_strings_parse() {
        assert_eq!(
            parse_busctl_string(&fixture("busctl-active-profile.txt")).as_deref(),
            Some("performance")
        );
        assert_eq!(parse_busctl_string("u 3"), None);
        assert_eq!(parse_busctl_string(""), None);
    }

    #[test]
    fn an_absent_power_service_is_recognised() {
        assert!(is_service_absent(&fixture("busctl-not-activatable.txt")));
        // dbus-daemon's wording; dbus-broker, Fedora's, answers "not activatable".
        assert!(is_service_absent(
            "Failed to get property ActiveProfile: The name org.freedesktop.UPower.PowerProfiles was not provided by any .service files"
        ));
        assert!(!is_service_absent("Access denied"));
    }

    #[test]
    fn the_power_profile_is_asked_of_the_upower_service_alone() {
        let root = tempfile::tempdir().unwrap();
        let host = SystemHost {
            path: Some(root.path().as_os_str().to_owned()),
            power_supplies: root.path().join("power"),
            os_release: root.path().join("os-release"),
            cdi_dirs: Vec::new(),
            toolkits: root.path().join("local"),
        };
        let busctl = |answer: &str| {
            script(
                &root.path().join("busctl"),
                &format!(
                    "[ \"$*\" = 'get-property org.freedesktop.UPower.PowerProfiles \
                     /org/freedesktop/UPower/PowerProfiles org.freedesktop.UPower.PowerProfiles \
                     ActiveProfile' ] || {{ echo \"unexpected: $*\" >&2; exit 2; }}\n{answer}"
                ),
            );
        };

        busctl("echo 's \"balanced\"'");
        assert_eq!(host.power_profile(), Probed::Found("balanced".into()));
        busctl(&format!(
            "cat <<'EOF' >&2\n{}EOF\nexit 1",
            fixture("busctl-not-activatable.txt")
        ));
        assert_eq!(host.power_profile(), Probed::Missing);
        busctl("echo 'Access denied' >&2; exit 1");
        assert_eq!(host.power_profile(), Probed::Failed("Access denied".into()));
    }

    #[test]
    fn the_toolkit_packages_are_queried_by_exact_names() {
        let root = tempfile::tempdir().unwrap();
        let host = SystemHost {
            path: Some(root.path().as_os_str().to_owned()),
            power_supplies: root.path().join("power"),
            os_release: root.path().join("os-release"),
            cdi_dirs: Vec::new(),
            toolkits: root.path().join("local"),
        };
        assert_eq!(host.toolkit_packages(&[(13, 4)]), Probed::Missing);
        // A toolkit installed without its meta-package, as CUDA 13.2 is
        // here, beside links that name no release.
        for dir in ["cuda-13.2", "cuda-13", "cuda"] {
            fs::create_dir_all(root.path().join("local").join(dir)).unwrap();
        }
        script(
            &root.path().join("rpm"),
            &format!(
                "[ \"$*\" = '-q --nodigest --nosignature --qf %{{NAME}}\\n cuda-toolkit \
                 cuda-toolkit-13 cuda-toolkit-13-2 cuda-toolkit-13-4' ] \
                 || {{ echo \"unexpected: $*\" >&2; exit 2; }}\ncat <<'EOF'\n{}EOF\nexit 3",
                fixture("rpm-q-cuda-toolkit-packages.txt")
            ),
        );
        assert_eq!(
            host.toolkit_packages(&[(13, 4)]),
            Probed::Found(vec!["cuda-toolkit-13-4".into()])
        );
        // rpm failing is told from names not installed.
        script(
            &root.path().join("rpm"),
            "echo 'the database is locked' >&2; exit 1",
        );
        assert_eq!(
            host.toolkit_packages(&[(13, 4)]),
            Probed::Failed("the database is locked".into())
        );
    }

    #[test]
    fn rpm_prints_the_names_installed_and_words_the_others() {
        let asked: Vec<String> = [
            "cuda-toolkit",
            "cuda-toolkit-13",
            "cuda-toolkit-13-2",
            "cuda-toolkit-13-4",
        ]
        .map(String::from)
        .to_vec();
        assert_eq!(
            parse_installed(&fixture("rpm-q-cuda-toolkit-packages.txt"), &asked),
            ["cuda-toolkit-13-4"]
        );
    }

    #[test]
    fn compiler_cache_wrappers_are_skipped_for_the_compiler_behind_them() {
        let root = tempfile::tempdir().unwrap();
        let (wrappers, bin) = (root.path().join("ccache"), root.path().join("bin"));
        fs::create_dir_all(&wrappers).unwrap();
        fs::create_dir_all(&bin).unwrap();
        script(&bin.join("ccache"), "echo 16.2.1");
        std::os::unix::fs::symlink(bin.join("ccache"), wrappers.join("g++")).unwrap();
        script(&bin.join("g++"), "echo 16.2.1");
        script(&bin.join("g++-15"), "echo 15.3.1");

        let host = SystemHost {
            path: Some(std::env::join_paths([&wrappers, &bin]).unwrap()),
            power_supplies: root.path().join("power"),
            os_release: root.path().join("os-release"),
            cdi_dirs: Vec::new(),
            toolkits: root.path().join("local"),
        };
        let Probed::Found(compilers) = host.host_compilers() else {
            panic!("no compiler found");
        };
        let found: Vec<_> = compilers
            .iter()
            .map(|c| (c.command.as_str(), c.path.clone(), c.version))
            .collect();
        assert_eq!(
            found,
            [
                ("g++", bin.join("g++"), Version::new(16, 2, 1)),
                ("g++-15", bin.join("g++-15"), Version::new(15, 3, 1)),
            ]
        );
    }

    #[test]
    fn a_gxx_that_links_to_a_gxx_n_is_one_compiler() {
        let root = tempfile::tempdir().unwrap();
        script(&root.path().join("g++-15"), "echo 15.3.1");
        std::os::unix::fs::symlink("g++-15", root.path().join("g++")).unwrap();
        let host = SystemHost {
            path: Some(root.path().into()),
            power_supplies: root.path().join("power"),
            os_release: root.path().join("os-release"),
            cdi_dirs: Vec::new(),
            toolkits: root.path().join("local"),
        };
        assert_eq!(
            host.host_compilers(),
            Probed::Found(vec![HostCompiler {
                command: "g++".into(),
                path: root.path().join("g++"),
                version: Version::new(15, 3, 1),
            }])
        );
    }

    #[test]
    fn nvcc_is_found_through_the_cuda_link_and_reported_at_its_versioned_root() {
        let root = tempfile::tempdir().unwrap();
        let toolkit = fs::canonicalize(root.path()).unwrap().join("cuda-13.2");
        fs::create_dir_all(toolkit.join("bin")).unwrap();
        let header = toolkit.join(HOST_CONFIG[0]);
        fs::create_dir_all(header.parent().unwrap()).unwrap();
        fs::write(&header, fixture("host_config-13.2-excerpt.h")).unwrap();
        script(
            &toolkit.join("bin/nvcc"),
            &format!("cat <<'EOF'\n{}EOF", fixture("nvcc-13.2.txt")),
        );
        std::os::unix::fs::symlink("cuda-13.2", root.path().join("cuda")).unwrap();
        let host = SystemHost {
            path: Some(root.path().join("cuda/bin").into()),
            power_supplies: root.path().join("power"),
            os_release: root.path().join("os-release"),
            cdi_dirs: Vec::new(),
            toolkits: root.path().join("local"),
        };
        assert_eq!(
            host.cuda_toolkit(),
            Probed::Found(CudaToolkit {
                version: Version::new(13, 2, 86),
                nvcc: toolkit.join("bin/nvcc"),
                max_gcc_major: Probed::Found(15),
            })
        );
    }

    #[test]
    fn nvcc_off_the_path_is_found_under_the_toolkits_cuda_link() {
        let root = tempfile::tempdir().unwrap();
        let local = fs::canonicalize(root.path()).unwrap().join("local");
        let toolkit = local.join("cuda-13.2");
        fs::create_dir_all(toolkit.join("bin")).unwrap();
        script(
            &toolkit.join("bin/nvcc"),
            &format!("cat <<'EOF'\n{}EOF", fixture("nvcc-13.2.txt")),
        );
        std::os::unix::fs::symlink("cuda-13.2", local.join("cuda")).unwrap();
        fs::create_dir(root.path().join("bin")).unwrap();
        let host = SystemHost {
            path: Some(root.path().join("bin").into()),
            power_supplies: root.path().join("power"),
            os_release: root.path().join("os-release"),
            cdi_dirs: Vec::new(),
            toolkits: local,
        };
        let Probed::Found(found) = host.cuda_toolkit() else {
            panic!("no toolkit found");
        };
        assert_eq!(
            (found.version, found.nvcc),
            (Version::new(13, 2, 86), toolkit.join("bin/nvcc"))
        );
    }

    #[test]
    fn the_fedora_release_comes_from_os_release() {
        assert_eq!(
            parse_fedora_release(&fixture("os-release-fedora44.txt")),
            Some(44)
        );
        assert_eq!(
            parse_fedora_release("ID=\"fedora\"\nVERSION_ID='45'\n"),
            Some(45)
        );
        assert_eq!(parse_fedora_release("ID=ubuntu\nVERSION_ID=24.04\n"), None);
        assert_eq!(
            parse_fedora_release("ID=fedora\nVERSION_ID=rawhide\n"),
            None
        );
        assert_eq!(parse_fedora_release(""), None);
    }

    #[test]
    fn os_release_is_read_from_its_file() {
        let root = tempfile::tempdir().unwrap();
        let host = SystemHost {
            path: None,
            power_supplies: root.path().join("power"),
            os_release: root.path().join("os-release"),
            cdi_dirs: Vec::new(),
            toolkits: root.path().join("local"),
        };
        assert_eq!(host.fedora_release(), Probed::Missing);
        fs::write(
            root.path().join("os-release"),
            fixture("os-release-fedora44.txt"),
        )
        .unwrap();
        assert_eq!(host.fedora_release(), Probed::Found(44));
    }

    #[test]
    fn repolist_gives_the_repository_ids() {
        let ids = parse_repolist(&fixture("dnf-repolist-enabled.txt"));
        assert_eq!(ids.len(), 14);
        assert_eq!(ids[1], "copr:copr.fedorainfracloud.org:phracek:PyCharm");
        let cuda: Vec<&String> = ids.iter().filter(|id| id.starts_with("cuda-")).collect();
        assert_eq!(cuda, ["cuda-fedora44-x86_64"]);
        assert!(parse_repolist("").is_empty());
    }

    #[test]
    fn package_names_are_sorted_once_each() {
        let names = parse_package_names(&fixture("dnf-repoquery-cuda-driver-packages.txt"));
        assert_eq!(names.len(), 12);
        assert_eq!(names[0], "kmod-nvidia-latest-dkms");
        assert_eq!(parse_package_names("b\na\n\nb\n"), ["a", "b"]);
    }

    #[test]
    fn gxx_names_are_recognised_and_ordered() {
        let mut names = vec![
            "g++-14",
            "g++",
            "g++-15",
            "g++-x",
            "gcc",
            "g++-",
            "x86_64-redhat-linux-g++-15",
        ];
        names.retain(|name| is_gxx(name));
        names.sort_by_key(|name| gxx_order(name));
        assert_eq!(names, ["g++", "g++-15", "g++-14"]);
    }

    fn supply(root: &Path, name: &str, kind: &str, online: Option<&str>) {
        let dir = root.join(name);
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("type"), format!("{kind}\n")).unwrap();
        if let Some(online) = online {
            fs::write(dir.join("online"), format!("{online}\n")).unwrap();
        }
    }

    #[test]
    fn a_file_and_a_directory_are_told_apart() {
        let root = tempfile::tempdir().unwrap();
        let file = root.path().join("model.gguf");
        fs::write(&file, "").unwrap();
        let host = SystemHost::new();
        assert!(host.is_dir(root.path()));
        assert!(!host.is_dir(&file));
        assert!(!host.is_dir(&root.path().join("absent")));
        assert!(host.is_file(&file));
        assert!(!host.is_file(root.path()));
    }

    #[test]
    fn ac_is_online_when_any_external_supply_is() {
        let root = tempfile::tempdir().unwrap();
        supply(root.path(), "ADP0", "Mains", Some("0"));
        supply(root.path(), "BAT0", "Battery", None);
        supply(root.path(), "ucsi-source-psy-USBC000:001", "USB", Some("1"));
        assert_eq!(ac_online(root.path()), Probed::Found(true));
    }

    #[test]
    fn ac_is_offline_on_battery() {
        let root = tempfile::tempdir().unwrap();
        supply(root.path(), "ADP0", "Mains", Some("0"));
        supply(root.path(), "BAT0", "Battery", Some("1"));
        assert_eq!(ac_online(root.path()), Probed::Found(false));
    }

    #[test]
    fn ac_is_unknown_without_external_supplies() {
        let root = tempfile::tempdir().unwrap();
        supply(root.path(), "BAT0", "Battery", None);
        assert_eq!(ac_online(root.path()), Probed::Missing);
        assert_eq!(ac_online(&root.path().join("absent")), Probed::Missing);
    }

    #[test]
    fn host_config_is_found_in_either_layout() {
        let root = tempfile::tempdir().unwrap();
        assert_eq!(read_max_gcc_major(root.path()), Probed::Missing);
        let header = root.path().join("include/crt/host_config.h");
        fs::create_dir_all(header.parent().unwrap()).unwrap();
        fs::write(&header, "#if __GNUC__ > 14\n").unwrap();
        assert_eq!(read_max_gcc_major(root.path()), Probed::Found(14));
        fs::write(&header, "nothing\n").unwrap();
        assert!(matches!(read_max_gcc_major(root.path()), Probed::Failed(_)));
    }

    #[test]
    fn a_gxx_that_cannot_tell_its_version_is_a_failed_probe() {
        let root = tempfile::tempdir().unwrap();
        script(
            &root.path().join("g++"),
            "echo 'g++: error while loading shared libraries: libisl.so.15' >&2; exit 127",
        );
        let host = SystemHost {
            path: Some(root.path().into()),
            power_supplies: root.path().join("power"),
            os_release: root.path().join("os-release"),
            cdi_dirs: Vec::new(),
            toolkits: root.path().join("local"),
        };
        assert_eq!(
            host.host_compilers(),
            Probed::Failed("g++: g++: error while loading shared libraries: libisl.so.15".into())
        );
    }
}
