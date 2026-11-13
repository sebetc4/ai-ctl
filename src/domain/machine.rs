//! Facts about the machine, as the probes report them.

use std::fmt;
use std::path::PathBuf;

/// A dotted version number such as `13.2.86`; missing components are zero.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Version {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}

impl Version {
    pub const fn new(major: u32, minor: u32, patch: u32) -> Self {
        Self {
            major,
            minor,
            patch,
        }
    }

    /// Parses `13`, `13.2` or `13.2.86`, ignoring a non-numeric suffix on the
    /// last component (`2.45.0-rc1`). Returns `None` without a numeric major.
    pub fn parse(text: &str) -> Option<Self> {
        let mut parts = text.trim().splitn(4, '.');
        let major = leading_number(parts.next()?)?;
        let minor = parts.next().and_then(leading_number).unwrap_or(0);
        let patch = parts.next().and_then(leading_number).unwrap_or(0);
        Some(Self::new(major, minor, patch))
    }

    /// The `major.minor` pair, for comparisons that ignore the patch level.
    pub fn release(self) -> (u32, u32) {
        (self.major, self.minor)
    }
}

fn leading_number(text: &str) -> Option<u32> {
    let end = text
        .find(|c: char| !c.is_ascii_digit())
        .unwrap_or(text.len());
    text[..end].parse().ok()
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

/// What a probe found.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Probed<T> {
    Found(T),
    /// The program, library or service is not installed.
    Missing,
    /// It is installed but could not be read; the message says why.
    Failed(String),
}

impl<T> Probed<T> {
    pub fn found(&self) -> Option<&T> {
        match self {
            Probed::Found(value) => Some(value),
            _ => None,
        }
    }
}

/// The NVIDIA GPU and its driver. Memory is in bytes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Gpu {
    pub name: String,
    /// Compute capability, `(12, 0)` for Blackwell consumer GPUs.
    pub compute_capability: (u32, u32),
    pub memory_total: u64,
    /// Set aside by the driver, out of `memory_total`; no process can
    /// allocate it.
    pub memory_reserved: u64,
    pub memory_used: u64,
    pub driver_version: String,
    /// Newest CUDA version the driver supports.
    pub cuda_version: Version,
    /// Processes holding GPU memory.
    pub processes: Vec<GpuProcess>,
}

impl Gpu {
    /// The memory processes can allocate, in MiB.
    pub fn usable_mib(&self) -> u64 {
        self.memory_total.saturating_sub(self.memory_reserved) >> 20
    }
}

/// A process holding GPU memory.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GpuProcess {
    pub pid: u32,
    pub name: String,
    /// Bytes, when the driver reports them.
    pub memory: Option<u64>,
}

/// The CUDA toolkit `nvcc` belongs to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CudaToolkit {
    pub version: Version,
    pub nvcc: PathBuf,
    /// Newest GCC major version `nvcc` accepts, read from the toolkit's
    /// `crt/host_config.h`.
    pub max_gcc_major: Probed<u32>,
}

/// CUDA 13.2 is reported to produce wrong output in llama.cpp builds
/// (llama.cpp issue #21255); see `docs/stack.md` §8.
pub const BROKEN_TOOLKIT: (u32, u32) = (13, 2);
/// ggml adds the `120a-real` architecture from CUDA 12.8 on.
pub const MIN_TOOLKIT: (u32, u32) = (12, 8);

/// Why a CUDA toolkit cannot build the backends on this machine.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum ToolkitProblem {
    #[error(
        "CUDA {}.{} is reported to produce wrong output in llama.cpp builds (llama.cpp issue #21255)",
        BROKEN_TOOLKIT.0,
        BROKEN_TOOLKIT.1
    )]
    Broken,
    #[error(
        "building for sm_120a needs CUDA {}.{} or newer",
        MIN_TOOLKIT.0,
        MIN_TOOLKIT.1
    )]
    TooOld,
    #[error(
        "newer than CUDA {}.{}, the newest the driver supports",
        driver.major,
        driver.minor
    )]
    NewerThanDriver { driver: Version },
}

/// The toolkit rule `doctor` and the build gate share: CUDA 13.2 is
/// refused, 12.8 is the minimum, and the toolkit cannot be newer than the
/// driver's CUDA version. An unknown driver is not compared.
pub fn toolkit_problem(toolkit: Version, driver_cuda: Option<Version>) -> Option<ToolkitProblem> {
    let release = toolkit.release();
    if release == BROKEN_TOOLKIT {
        return Some(ToolkitProblem::Broken);
    }
    if release < MIN_TOOLKIT {
        return Some(ToolkitProblem::TooOld);
    }
    match driver_cuda {
        Some(driver) if release > driver.release() => {
            Some(ToolkitProblem::NewerThanDriver { driver })
        }
        _ => None,
    }
}

/// The packages of NVIDIA's CUDA repository that make up a driver, which
/// comes from RPM Fusion only; `docs/toolchain.md` §2 sets this exclusion.
pub const DRIVER_PACKAGES: [&str; 5] = [
    "nvidia-*",
    "kmod-nvidia*",
    "xorg-x11-*nvidia*",
    "libnvidia-*",
    "dnf-plugin-nvidia",
];

/// An enabled NVIDIA CUDA repository, such as `cuda-fedora44-x86_64`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CudaRepository {
    pub id: String,
    /// The driver packages it offers despite its exclusions.
    pub driver_packages: Vec<String>,
}

/// A CDI specification of NVIDIA's GPUs, as the container toolkit writes
/// it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CdiSpec {
    pub path: PathBuf,
    /// The driver whose `libcuda.so` it mounts, such as `615.71.09`; `None`
    /// when it names none.
    pub driver: Option<String>,
    /// The host paths it mounts that are not on disk: podman starts no
    /// container with it.
    pub missing: Vec<PathBuf>,
}

/// The id dnf gives NVIDIA's CUDA repository for a Fedora release.
pub fn cuda_repository_id(fedora: u32) -> String {
    format!("cuda-fedora{fedora}-x86_64")
}

/// The repository file of NVIDIA's CUDA repository for a Fedora release.
pub fn cuda_repository_url(fedora: u32) -> String {
    format!(
        "https://developer.download.nvidia.com/compute/cuda/repos/fedora{fedora}/x86_64/cuda-fedora{fedora}.repo"
    )
}

/// True for the toolkit meta-packages that follow NVIDIA's releases:
/// `cuda-toolkit` follows the newest, `cuda-toolkit-13` the newest 13.x.
/// Only `cuda-toolkit-<major>-<minor>` stays on its version.
pub fn is_unversioned_toolkit(package: &str) -> bool {
    package == "cuda-toolkit"
        || package
            .strip_prefix("cuda-toolkit-")
            .is_some_and(|major| !major.is_empty() && major.bytes().all(|b| b.is_ascii_digit()))
}

/// The `(major, minor)` of a versioned toolkit meta-package such as
/// `cuda-toolkit-13-4`, which stays on its version.
pub fn versioned_toolkit(package: &str) -> Option<(u32, u32)> {
    let (major, minor) = package.strip_prefix("cuda-toolkit-")?.split_once('-')?;
    let number = |text: &str| {
        (!text.is_empty() && text.bytes().all(|b| b.is_ascii_digit()))
            .then(|| text.parse().ok())
            .flatten()
    };
    Some((number(major)?, number(minor)?))
}

/// The versioned toolkit meta-package of `release`, such as
/// `cuda-toolkit-13-4`; [`versioned_toolkit`] reads it back.
pub fn toolkit_package((major, minor): (u32, u32)) -> String {
    format!("cuda-toolkit-{major}-{minor}")
}

/// The toolkit meta-packages that may be installed for `releases`:
/// `cuda-toolkit`, and for each release `cuda-toolkit-<major>` and
/// [`toolkit_package`]'s name, sorted, each once.
pub fn toolkit_meta_packages(releases: &[(u32, u32)]) -> Vec<String> {
    let mut names = vec!["cuda-toolkit".to_owned()];
    for &(major, minor) in releases {
        names.push(format!("cuda-toolkit-{major}"));
        names.push(toolkit_package((major, minor)));
    }
    names.sort();
    names.dedup();
    names
}

/// A GNU C++ compiler found on the `PATH`, such as `g++` or `g++-15`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HostCompiler {
    pub command: String,
    pub path: PathBuf,
    pub version: Version,
}

/// The compiler a CUDA build should use: the newest one whose major version
/// `nvcc` accepts. On equal versions, the earlier one in `compilers` wins, so
/// the default `g++` is kept over an identical `g++-N`.
pub fn pick_host_compiler(compilers: &[HostCompiler], max_major: u32) -> Option<&HostCompiler> {
    compilers
        .iter()
        .filter(|compiler| compiler.version.major <= max_major)
        .fold(None, |best: Option<&HostCompiler>, compiler| match best {
            Some(best) if best.version >= compiler.version => Some(best),
            _ => Some(compiler),
        })
}

/// A command-line tool whose version `doctor` checks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tool {
    Cmake,
    Git,
    Podman,
    NvidiaCtk,
}

impl Tool {
    /// The command that runs the tool.
    pub fn command(self) -> &'static str {
        match self {
            Tool::Cmake => "cmake",
            Tool::Git => "git",
            Tool::Podman => "podman",
            Tool::NvidiaCtk => "nvidia-ctk",
        }
    }

    /// The command that installs the tool on Fedora.
    pub fn install(self) -> &'static str {
        match self {
            Tool::Cmake => "sudo dnf install cmake",
            Tool::Git => "sudo dnf install git",
            Tool::Podman => "sudo dnf install podman",
            Tool::NvidiaCtk => {
                "sudo dnf install nvidia-container-toolkit   # from NVIDIA's repository"
            }
        }
    }

    /// The oldest version a native build accepts, for the build tools.
    pub fn minimum(self) -> Option<Version> {
        match self {
            // ggml's CUDA backend: `cmake_minimum_required(VERSION 3.18)`.
            Tool::Cmake => Some(Version::new(3, 18, 0)),
            // A partial clone (`--filter=blob:none`) whose checkout and
            // `rev-parse` fetch missing objects lazily: 2.19.0 made clone
            // check what it receives, and 2.19.2 fixed the lazy fetch.
            Tool::Git => Some(Version::new(2, 19, 2)),
            Tool::Podman | Tool::NvidiaCtk => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn each_tool_names_its_install_command() {
        assert_eq!(Tool::Cmake.install(), "sudo dnf install cmake");
        assert_eq!(Tool::Git.install(), "sudo dnf install git");
        assert_eq!(Tool::Podman.install(), "sudo dnf install podman");
        assert_eq!(
            Tool::NvidiaCtk.install(),
            "sudo dnf install nvidia-container-toolkit   # from NVIDIA's repository"
        );
    }

    #[test]
    fn version_parses_one_to_three_components() {
        assert_eq!(Version::parse("13"), Some(Version::new(13, 0, 0)));
        assert_eq!(Version::parse("13.2"), Some(Version::new(13, 2, 0)));
        assert_eq!(Version::parse("13.2.86"), Some(Version::new(13, 2, 86)));
        assert_eq!(Version::parse(" 4.3.0\n"), Some(Version::new(4, 3, 0)));
    }

    #[test]
    fn version_ignores_suffixes_and_extra_components() {
        assert_eq!(Version::parse("2.45.0-rc1"), Some(Version::new(2, 45, 0)));
        assert_eq!(Version::parse("1.2.3.4"), Some(Version::new(1, 2, 3)));
    }

    #[test]
    fn version_needs_a_numeric_major() {
        assert_eq!(Version::parse(""), None);
        assert_eq!(Version::parse("V13.2"), None);
        assert_eq!(Version::parse("unknown"), None);
    }

    #[test]
    fn versions_order_numerically() {
        assert!(Version::new(13, 10, 0) > Version::new(13, 9, 99));
        assert_eq!(Version::new(13, 2, 86).release(), (13, 2));
        assert_eq!(Version::new(13, 2, 86).to_string(), "13.2.86");
    }

    fn gxx(command: &str, version: Version) -> HostCompiler {
        HostCompiler {
            command: command.to_owned(),
            path: PathBuf::from("/usr/bin").join(command),
            version,
        }
    }

    #[test]
    fn host_compiler_skips_versions_nvcc_rejects() {
        let compilers = [
            gxx("g++", Version::new(16, 2, 1)),
            gxx("g++-15", Version::new(15, 3, 1)),
            gxx("g++-14", Version::new(14, 3, 0)),
        ];
        let picked = pick_host_compiler(&compilers, 15).map(|c| c.command.as_str());
        assert_eq!(picked, Some("g++-15"));
    }

    #[test]
    fn host_compiler_keeps_the_default_when_accepted() {
        let compilers = [
            gxx("g++", Version::new(15, 3, 1)),
            gxx("g++-15", Version::new(15, 3, 1)),
            gxx("g++-14", Version::new(14, 3, 0)),
        ];
        let picked = pick_host_compiler(&compilers, 15).map(|c| c.command.as_str());
        assert_eq!(picked, Some("g++"));
    }

    #[test]
    fn host_compiler_prefers_a_newer_gxx_n_to_an_older_default() {
        // In the order the host probe gives: `g++` first.
        let compilers = [
            gxx("g++", Version::new(14, 3, 0)),
            gxx("g++-15", Version::new(15, 3, 1)),
        ];
        let picked = pick_host_compiler(&compilers, 15).map(|c| c.command.as_str());
        assert_eq!(picked, Some("g++-15"));
    }

    #[test]
    fn host_compiler_is_none_when_every_version_is_too_new() {
        let compilers = [gxx("g++", Version::new(16, 2, 1))];
        assert_eq!(pick_host_compiler(&compilers, 15), None);
        assert_eq!(pick_host_compiler(&[], 15), None);
    }

    #[test]
    fn cuda_13_2_is_refused_whatever_the_driver() {
        let driver = Some(Version::new(13, 4, 0));
        assert_eq!(
            toolkit_problem(Version::new(13, 2, 86), driver),
            Some(ToolkitProblem::Broken)
        );
        assert_eq!(
            toolkit_problem(Version::new(13, 2, 0), None),
            Some(ToolkitProblem::Broken)
        );
    }

    #[test]
    fn a_toolkit_older_than_12_8_is_refused() {
        assert_eq!(
            toolkit_problem(Version::new(12, 6, 85), Some(Version::new(13, 4, 0))),
            Some(ToolkitProblem::TooOld)
        );
        assert_eq!(toolkit_problem(Version::new(12, 8, 0), None), None);
    }

    #[test]
    fn a_toolkit_newer_than_the_driver_is_refused() {
        let driver = Version::new(13, 4, 0);
        assert_eq!(
            toolkit_problem(Version::new(13, 5, 10), Some(driver)),
            Some(ToolkitProblem::NewerThanDriver { driver })
        );
        assert_eq!(
            ToolkitProblem::NewerThanDriver { driver }.to_string(),
            "newer than CUDA 13.4, the newest the driver supports"
        );
    }

    #[test]
    fn a_toolkit_up_to_the_driver_is_accepted() {
        let driver = Some(Version::new(13, 4, 0));
        assert_eq!(toolkit_problem(Version::new(13, 4, 92), driver), None);
        assert_eq!(toolkit_problem(Version::new(13, 3, 0), driver), None);
        // An unknown driver is not compared.
        assert_eq!(toolkit_problem(Version::new(13, 9, 0), None), None);
    }

    #[test]
    fn only_meta_packages_without_a_minor_version_are_unversioned() {
        assert!(is_unversioned_toolkit("cuda-toolkit"));
        assert!(is_unversioned_toolkit("cuda-toolkit-13"));
        for package in [
            "cuda-toolkit-13-4",
            "cuda-toolkit-13-4-config-common",
            "cuda-toolkit-13-config-common",
            "cuda-toolkit-config-common",
            "cuda-toolkit-",
        ] {
            assert!(!is_unversioned_toolkit(package), "{package}");
        }
    }

    #[test]
    fn versioned_meta_packages_name_their_release() {
        assert_eq!(versioned_toolkit("cuda-toolkit-13-4"), Some((13, 4)));
        for package in [
            "cuda-toolkit",
            "cuda-toolkit-13",
            "cuda-toolkit-13-4-config-common",
            "cuda-toolkit-13-config-common",
            "cuda-toolkit-config-common",
        ] {
            assert_eq!(versioned_toolkit(package), None, "{package}");
        }
    }

    #[test]
    fn a_release_names_its_versioned_meta_package() {
        assert_eq!(toolkit_package((13, 4)), "cuda-toolkit-13-4");
        assert_eq!(versioned_toolkit(&toolkit_package((13, 4))), Some((13, 4)));
    }

    #[test]
    fn the_meta_packages_of_releases_are_named_once_each() {
        assert_eq!(
            toolkit_meta_packages(&[(13, 4), (13, 2), (13, 4)]),
            [
                "cuda-toolkit",
                "cuda-toolkit-13",
                "cuda-toolkit-13-2",
                "cuda-toolkit-13-4"
            ]
        );
        assert_eq!(toolkit_meta_packages(&[]), ["cuda-toolkit"]);
    }

    #[test]
    fn the_cuda_repository_follows_the_fedora_release() {
        assert_eq!(cuda_repository_id(44), "cuda-fedora44-x86_64");
        assert_eq!(
            cuda_repository_url(45),
            "https://developer.download.nvidia.com/compute/cuda/repos/fedora45/x86_64/cuda-fedora45.repo"
        );
    }

    #[test]
    fn only_build_tools_have_a_minimum() {
        assert_eq!(Tool::Cmake.minimum(), Some(Version::new(3, 18, 0)));
        assert_eq!(Tool::Git.minimum(), Some(Version::new(2, 19, 2)));
        assert_eq!(Tool::Podman.minimum(), None);
        assert_eq!(Tool::NvidiaCtk.minimum(), None);
    }

    #[test]
    fn probed_exposes_only_found_values() {
        assert_eq!(Probed::Found(3).found(), Some(&3));
        assert_eq!(Probed::<u32>::Missing.found(), None);
        assert_eq!(Probed::<u32>::Failed("boom".into()).found(), None);
    }
}
