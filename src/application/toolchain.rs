//! The build toolchain: the checks of `doctor`'s Toolchain section, which
//! also gate a build, and the toolchain they settle. The remedies give the
//! commands of `docs/toolchain.md`.

use crate::domain::build::{Toolchain, c_compiler_beside};
use crate::domain::diagnostics::{Check, Status};
use crate::domain::machine::{
    BROKEN_TOOLKIT, CudaRepository, CudaToolkit, DRIVER_PACKAGES, Gpu, HostCompiler, Probed, Tool,
    ToolkitProblem, Version, cuda_repository_id, cuda_repository_url, is_unversioned_toolkit,
    pick_host_compiler, toolkit_package, toolkit_problem, versioned_toolkit,
};

use super::ports::ToolchainProbe;

/// Driver packages named in a warning before the count stands for the rest.
const NAMED_PACKAGES: usize = 3;

/// The Toolchain checks, and the toolchain a build compiles with when none
/// of them failed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Inspection {
    pub checks: Vec<Check>,
    pub toolchain: Option<Toolchain>,
}

/// Checks the CUDA toolkit against `gpu`'s driver, NVIDIA's repositories
/// and the installed toolkit packages, the host compiler against the
/// toolkit, and the build tools against their minimums.
pub fn inspect(host: &dyn ToolchainProbe, gpu: Option<&Gpu>) -> Inspection {
    let toolkit = host.cuda_toolkit();
    let repositories = host.cuda_repositories();
    let driver_cuda = gpu.map(|gpu| gpu.cuda_version);
    // The releases whose meta-packages may be installed: the driver's and
    // the toolkit's; the probe adds those of the toolkits it finds.
    let releases: Vec<(u32, u32)> = driver_cuda
        .into_iter()
        .chain(toolkit.found().map(|toolkit| toolkit.version))
        .map(Version::release)
        .collect();
    let packages = host.toolkit_packages(&releases);
    let setup = Setup {
        driver_cuda,
        fedora: host.fedora_release().found().copied(),
        repositories: repositories.found().map(Vec::as_slice),
        packages: packages.found().map(Vec::as_slice),
    };
    let (compiler, picked) = compiler_check(host, &toolkit);
    let mut checks = vec![
        toolkit_check(&toolkit, &setup),
        repository_check(&repositories),
        package_check(&packages, &toolkit, &setup),
        compiler,
        minimum_check(host, "CMake", Tool::Cmake),
        minimum_check(host, "Git", Tool::Git),
    ];
    let failed = checks.iter().any(|check| check.status == Status::Fail);
    let toolchain = match (failed, toolkit.found(), picked) {
        (false, Some(toolkit), Some((compiler, c_compiler))) => {
            match Toolchain::new(toolkit.nvcc.clone(), toolkit.version, compiler, c_compiler) {
                Ok(toolchain) => Some(toolchain),
                Err(err) => {
                    checks.push(Check::fail("Toolchain paths", err.to_string(), None));
                    None
                }
            }
        }
        _ => None,
    };
    Inspection { checks, toolchain }
}

/// What the toolkit remedy depends on.
struct Setup<'a> {
    driver_cuda: Option<Version>,
    fedora: Option<u32>,
    /// `None` when the repositories could not be read.
    repositories: Option<&'a [CudaRepository]>,
    /// The installed toolkit packages; `None` when they could not be read.
    packages: Option<&'a [String]>,
}

impl Setup<'_> {
    /// The versioned toolkit package matching the driver.
    fn package(&self) -> String {
        match self.driver_cuda {
            Some(cuda) => toolkit_package(cuda.release()),
            None => "cuda-toolkit-<major>-<minor>".to_owned(),
        }
    }

    /// Installs the toolkit matching the driver from NVIDIA's repository for
    /// the running Fedora release, adding the repository and its driver
    /// exclusion first when they are missing (`docs/toolchain.md` §2).
    fn toolkit_remedy(&self) -> Option<String> {
        if self.driver_is_broken() {
            return fix(DRIVER_UPDATE);
        }
        fix(numbered(&self.install_steps()))
    }

    /// For a toolkit newer than the driver, `newer`: installs the driver's
    /// toolkit when it is missing, then removes the newer one, so that
    /// `/usr/local/cuda`, an `alternatives` link whose priority is the
    /// version, falls back to the driver's (`docs/toolchain.md` §3). Pinning
    /// the link instead leaves it in manual mode: the remedy offers it last.
    fn newer_toolkit_remedy(&self, newer: Version) -> Option<String> {
        let Some(driver) = self.driver_cuda.filter(|_| !self.driver_is_broken()) else {
            return self.toolkit_remedy();
        };
        let installed = self.packages.is_some_and(|packages| {
            packages
                .iter()
                .any(|package| versioned_toolkit(package) == Some(driver.release()))
        });
        let mut steps = if installed {
            Vec::new()
        } else {
            self.install_steps()
        };
        steps.push(format!(
            "sudo dnf remove {}",
            toolkit_package(newer.release())
        ));
        let version = format!("{}-{}", newer.major, newer.minor);
        steps.push(format!(
            "sudo dnf remove $(rpm -qa --qf '%{{name}}\\n' | grep -E -- '-{version}(-|$)')"
        ));
        let target = format!("/usr/local/cuda-{}.{}", driver.major, driver.minor);
        fix(format!(
            "{}\nto keep CUDA {}.{} instead: sudo alternatives --set cuda {target} && \
             sudo alternatives --set cuda-{} {target}",
            numbered(&steps),
            newer.major,
            newer.minor,
            driver.major
        ))
    }

    fn driver_is_broken(&self) -> bool {
        self.driver_cuda
            .is_some_and(|cuda| cuda.release() == BROKEN_TOOLKIT)
    }

    /// The steps that install the driver's toolkit, from NVIDIA's
    /// repository, added with its driver exclusion when missing.
    fn install_steps(&self) -> Vec<String> {
        let install = format!("sudo dnf install {}", self.package());
        let Some(fedora) = self.fedora else {
            return vec![format!(
                "add NVIDIA's CUDA repository for your Fedora release with its driver packages excluded (docs/toolchain.md §2), then: {install}"
            )];
        };
        let id = cuda_repository_id(fedora);
        let repository = self
            .repositories
            .and_then(|repositories| repositories.iter().find(|repo| repo.id == id));
        let mut steps = Vec::new();
        if repository.is_none() {
            steps.push(format!(
                "sudo dnf config-manager addrepo --from-repofile={}",
                cuda_repository_url(fedora)
            ));
        }
        if repository.is_none_or(|repo| !repo.driver_packages.is_empty()) {
            steps.push(exclude_command(&id));
        }
        steps.push(install);
        steps
    }
}

/// The remedy for a driver whose CUDA version is the broken toolkit's.
const DRIVER_UPDATE: &str =
    "update the NVIDIA driver to one supporting CUDA 13.3 or newer, then install that toolkit";

/// Excludes the driver packages from a CUDA repository.
fn exclude_command(id: &str) -> String {
    format!(
        "sudo dnf config-manager setopt {id}.exclude='{}'",
        DRIVER_PACKAGES.join(",")
    )
}

/// Steps one per line, numbered when there are several.
fn numbered(steps: &[String]) -> String {
    if steps.len() == 1 {
        return steps[0].clone();
    }
    steps
        .iter()
        .enumerate()
        .map(|(i, step)| format!("{}. {step}", i + 1))
        .collect::<Vec<_>>()
        .join("\n")
}

fn toolkit_check(toolkit: &Probed<CudaToolkit>, setup: &Setup) -> Check {
    const NAME: &str = "CUDA toolkit";
    let toolkit = match toolkit {
        Probed::Found(toolkit) => toolkit,
        Probed::Missing => {
            return Check::fail(
                NAME,
                "nvcc not found on the PATH nor in /usr/local/cuda/bin",
                setup.toolkit_remedy(),
            );
        }
        Probed::Failed(err) => {
            return Check::fail(NAME, format!("nvcc: {err}"), setup.toolkit_remedy());
        }
    };

    let found = format!("{} ({})", toolkit.version, toolkit.nvcc.display());
    match toolkit_problem(toolkit.version, setup.driver_cuda) {
        Some(problem @ ToolkitProblem::NewerThanDriver { .. }) => Check::fail(
            NAME,
            format!("{found}: {problem}"),
            setup.newer_toolkit_remedy(toolkit.version),
        ),
        Some(problem) => Check::fail(NAME, format!("{found}: {problem}"), setup.toolkit_remedy()),
        None => Check::ok(NAME, found),
    }
}

/// An enabled CUDA repository offering driver packages would replace the
/// RPM Fusion driver with the first driver NVIDIA publishes before it.
fn repository_check(repositories: &Probed<Vec<CudaRepository>>) -> Check {
    const NAME: &str = "CUDA repositories";
    let repositories = match repositories {
        Probed::Found(repositories) => repositories,
        Probed::Missing => return Check::skip(NAME, "dnf not found"),
        Probed::Failed(err) => return Check::warn(NAME, format!("dnf: {err}"), None),
    };
    if repositories.is_empty() {
        return Check::ok(NAME, "none enabled");
    }
    let offering: Vec<&CudaRepository> = repositories
        .iter()
        .filter(|repo| !repo.driver_packages.is_empty())
        .collect();
    if offering.is_empty() {
        let ids: Vec<&str> = repositories.iter().map(|repo| repo.id.as_str()).collect();
        return Check::ok(
            NAME,
            format!("{}: driver packages excluded", ids.join(", ")),
        );
    }
    let detail = offering
        .iter()
        .map(|repo| {
            let packages = &repo.driver_packages;
            let mut named = packages[..packages.len().min(NAMED_PACKAGES)].join(", ");
            if packages.len() > NAMED_PACKAGES {
                named.push_str(", …");
            }
            format!(
                "{} offers {} driver packages ({named})",
                repo.id,
                packages.len()
            )
        })
        .collect::<Vec<_>>()
        .join("; ");
    let remedy: Vec<String> = offering
        .iter()
        .map(|repo| exclude_command(&repo.id))
        .collect();
    Check::warn(
        NAME,
        format!("{detail}: an update could replace the RPM Fusion driver"),
        Some(remedy.join("\n")),
    )
}

/// An unversioned toolkit meta-package follows NVIDIA's newest release, so
/// an update can install a toolkit newer than the driver.
fn package_check(
    packages: &Probed<Vec<String>>,
    toolkit: &Probed<CudaToolkit>,
    setup: &Setup,
) -> Check {
    const NAME: &str = "Toolkit packages";
    let packages = match packages {
        Probed::Found(packages) => packages,
        Probed::Missing => return Check::skip(NAME, "rpm not found"),
        Probed::Failed(err) => return Check::warn(NAME, format!("rpm: {err}"), None),
    };
    let unversioned: Vec<&str> = packages
        .iter()
        .map(String::as_str)
        .filter(|package| is_unversioned_toolkit(package))
        .collect();
    if unversioned.is_empty() {
        let versioned: Vec<&str> = packages
            .iter()
            .map(String::as_str)
            .filter(|package| versioned_toolkit(package).is_some())
            .collect();
        let detail = if versioned.is_empty() {
            "no toolkit meta-package installed".to_owned()
        } else {
            versioned.join(", ")
        };
        return Check::ok(NAME, detail);
    }
    // Keep the installed toolkit's release when the toolkit check accepts
    // it, or take the driver's.
    let keep = match toolkit.found() {
        Some(toolkit) if toolkit_problem(toolkit.version, setup.driver_cuda).is_none() => {
            toolkit_package(toolkit.version.release())
        }
        _ => setup.package(),
    };
    Check::warn(
        NAME,
        format!(
            "{} installed: it follows NVIDIA's newest release, which can outrun the driver",
            unversioned.join(", ")
        ),
        fix(numbered(&[
            format!("sudo dnf install {keep}"),
            format!("sudo dnf remove {}", unversioned.join(" ")),
        ])),
    )
}

/// The host compiler check, and the C++ and C compilers a build uses: the
/// newest GCC `nvcc` accepts, or the default `g++` when its limit is
/// unknown.
fn compiler_check(
    host: &dyn ToolchainProbe,
    toolkit: &Probed<CudaToolkit>,
) -> (Check, Option<(HostCompiler, std::path::PathBuf)>) {
    const NAME: &str = "Host compiler";
    let compilers = match host.host_compilers() {
        Probed::Found(compilers) if !compilers.is_empty() => compilers,
        Probed::Found(_) | Probed::Missing => {
            let check = Check::fail(
                NAME,
                "no g++ found on the PATH",
                fix("sudo dnf install gcc-c++"),
            );
            return (check, None);
        }
        Probed::Failed(err) => return (Check::fail(NAME, format!("g++: {err}"), None), None),
    };
    let found = compiler_list(&compilers);
    let Some(toolkit) = toolkit.found() else {
        let check = Check::skip(
            NAME,
            format!("{found}; the GCC limit of nvcc needs the CUDA toolkit"),
        );
        return (check, None);
    };
    let (check, picked) = match &toolkit.max_gcc_major {
        Probed::Found(max) => match pick_host_compiler(&compilers, *max) {
            Some(picked) => (
                Check::ok(
                    NAME,
                    format!(
                        "{} {}; nvcc {} accepts GCC {max} at most (found {found})",
                        picked.command, picked.version, toolkit.version
                    ),
                ),
                Some(picked),
            ),
            None => (
                Check::fail(
                    NAME,
                    format!(
                        "nvcc {} accepts GCC {max} at most (found {found})",
                        toolkit.version
                    ),
                    fix(format!("sudo dnf install gcc{max}-c++")),
                ),
                None,
            ),
        },
        Probed::Missing => (
            Check::warn(
                NAME,
                format!(
                    "{found}; no host_config.h in the toolkit, so the GCC limit of nvcc is unknown"
                ),
                None,
            ),
            compilers.first(),
        ),
        Probed::Failed(err) => (
            Check::warn(
                NAME,
                format!("{found}; cannot read the GCC limit of nvcc: {err}"),
                None,
            ),
            compilers.first(),
        ),
    };
    let Some(picked) = picked else {
        return (check, None);
    };
    match c_compiler_beside(&picked.path).filter(|c| host.is_file(c)) {
        Some(c_compiler) => (check, Some((picked.clone(), c_compiler))),
        None => (
            Check::fail(
                NAME,
                format!(
                    "no C compiler beside {} ({})",
                    picked.command,
                    picked.path.display()
                ),
                fix(format!(
                    "sudo dnf install {}",
                    picked.command.replacen("g++", "gcc", 1).replace('-', "")
                )),
            ),
            None,
        ),
    }
}

fn minimum_check(host: &dyn ToolchainProbe, name: &str, tool: Tool) -> Check {
    let minimum = tool.minimum().unwrap_or(Version::new(0, 0, 0));
    let remedy = tool.install();
    match host.tool_version(tool) {
        Probed::Found(version) if version >= minimum => Check::ok(name, version.to_string()),
        Probed::Found(version) => Check::fail(
            name,
            format!("{version} is older than the {minimum} required"),
            fix(remedy),
        ),
        Probed::Missing => Check::fail(name, format!("{} not found", tool.command()), fix(remedy)),
        Probed::Failed(err) => Check::fail(name, format!("{}: {err}", tool.command()), None),
    }
}

fn compiler_list(compilers: &[HostCompiler]) -> String {
    compilers
        .iter()
        .map(|compiler| format!("{} {}", compiler.command, compiler.version))
        .collect::<Vec<_>>()
        .join(", ")
}

pub(super) fn fix(remedy: impl Into<String>) -> Option<String> {
    Some(remedy.into())
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use super::*;
    use crate::application::fakes::{FakeToolchain, gpu, gxx, toolkit};
    use crate::domain::build::{BuildId, cmake_flags};
    use crate::domain::machine::CudaRepository;
    use crate::domain::recipe::LLAMA_CPP;

    fn check<'a>(inspection: &'a Inspection, name: &str) -> &'a Check {
        inspection
            .checks
            .iter()
            .find(|check| check.name == name)
            .unwrap_or_else(|| panic!("no check named {name}"))
    }

    fn toolkit_remedy(host: &FakeToolchain, gpu: Option<&Gpu>) -> String {
        let inspection = inspect(host, gpu);
        check(&inspection, "CUDA toolkit")
            .remedy
            .clone()
            .expect("a remedy")
    }

    const EXCLUDE: &str = "sudo dnf config-manager setopt cuda-fedora44-x86_64.exclude='nvidia-*,kmod-nvidia*,xorg-x11-*nvidia*,libnvidia-*,dnf-plugin-nvidia'";

    #[test]
    fn the_remedy_adds_the_repository_of_the_running_release_with_its_exclusion() {
        let remedy = toolkit_remedy(&FakeToolchain::before_the_upgrade(), Some(&gpu()));
        assert_eq!(
            remedy,
            format!(
                "1. sudo dnf config-manager addrepo --from-repofile=https://developer.download.nvidia.com/compute/cuda/repos/fedora44/x86_64/cuda-fedora44.repo\n\
                 2. {EXCLUDE}\n\
                 3. sudo dnf install cuda-toolkit-13-4"
            )
        );
    }

    #[test]
    fn the_remedy_skips_what_is_already_set_up() {
        let mut host = FakeToolchain::healthy();
        host.toolkit = Probed::Missing;
        assert_eq!(
            toolkit_remedy(&host, Some(&gpu())),
            "sudo dnf install cuda-toolkit-13-4"
        );
        host.repositories = Probed::Found(vec![CudaRepository {
            id: "cuda-fedora44-x86_64".into(),
            driver_packages: vec!["nvidia-driver".into()],
        }]);
        assert_eq!(
            toolkit_remedy(&host, Some(&gpu())),
            format!("1. {EXCLUDE}\n2. sudo dnf install cuda-toolkit-13-4")
        );
    }

    #[test]
    fn the_remedy_follows_the_fedora_release() {
        let mut host = FakeToolchain::before_the_upgrade();
        host.fedora = Probed::Found(45);
        let remedy = toolkit_remedy(&host, Some(&gpu()));
        assert!(
            remedy.contains("/fedora45/x86_64/cuda-fedora45.repo"),
            "{remedy}"
        );
        assert!(
            remedy.contains("setopt cuda-fedora45-x86_64.exclude="),
            "{remedy}"
        );
    }

    #[test]
    fn unreadable_repositories_give_every_step() {
        let mut host = FakeToolchain::before_the_upgrade();
        host.repositories = Probed::Failed("dnf crashed".into());
        assert!(
            toolkit_remedy(&host, Some(&gpu())).starts_with("1. sudo dnf config-manager addrepo")
        );
    }

    #[test]
    fn without_fedora_or_driver_the_remedy_points_at_the_document() {
        let mut host = FakeToolchain::before_the_upgrade();
        host.fedora = Probed::Missing;
        let remedy = toolkit_remedy(&host, Some(&gpu()));
        assert!(remedy.contains("docs/toolchain.md"), "{remedy}");
        assert!(
            remedy.ends_with("sudo dnf install cuda-toolkit-13-4"),
            "{remedy}"
        );

        let remedy = toolkit_remedy(&FakeToolchain::before_the_upgrade(), None);
        assert!(
            remedy.ends_with("3. sudo dnf install cuda-toolkit-<major>-<minor>"),
            "{remedy}"
        );
    }

    #[test]
    fn a_driver_stuck_at_cuda_13_2_must_be_updated_first() {
        let mut stuck = gpu();
        stuck.cuda_version = Version::new(13, 2, 0);
        let remedy = toolkit_remedy(&FakeToolchain::before_the_upgrade(), Some(&stuck));
        assert!(remedy.starts_with("update the NVIDIA driver"), "{remedy}");
        // A toolkit newer than such a driver: its removal would leave none.
        let remedy = toolkit_remedy(&FakeToolchain::healthy(), Some(&stuck));
        assert!(remedy.starts_with("update the NVIDIA driver"), "{remedy}");
    }

    /// This machine once an update brought toolkit 13.5 beside a driver
    /// supporting CUDA 13.4, with the toolkit packages `packages`.
    fn newer_than_the_driver(packages: &[&str]) -> FakeToolchain {
        FakeToolchain {
            toolkit: Probed::Found(toolkit(Version::new(13, 5, 10))),
            toolkit_packages: Probed::Found(packages.iter().map(|&p| p.to_owned()).collect()),
            ..FakeToolchain::healthy()
        }
    }

    const REMOVE_13_5: &str = "sudo dnf remove cuda-toolkit-13-5\n\
         3. sudo dnf remove $(rpm -qa --qf '%{name}\\n' | grep -E -- '-13-5(-|$)')";
    const KEEP_13_5: &str = "to keep CUDA 13.5 instead: \
         sudo alternatives --set cuda /usr/local/cuda-13.4 && \
         sudo alternatives --set cuda-13 /usr/local/cuda-13.4";

    #[test]
    fn a_toolkit_newer_than_the_driver_gives_way_to_the_drivers() {
        let host = newer_than_the_driver(&[
            "cuda-toolkit-13-5",
            "cuda-toolkit-13-5-config-common",
            "cuda-toolkit-config-common",
        ]);
        assert_eq!(
            toolkit_remedy(&host, Some(&gpu())),
            format!("1. sudo dnf install cuda-toolkit-13-4\n2. {REMOVE_13_5}\n{KEEP_13_5}")
        );
        // The driver's toolkit is there already: `/usr/local/cuda` follows
        // the newer one, whose removal is enough.
        let host = newer_than_the_driver(&[
            "cuda-toolkit-13-4",
            "cuda-toolkit-13-5",
            "cuda-toolkit-config-common",
        ]);
        let remedy = toolkit_remedy(&host, Some(&gpu()));
        assert_eq!(
            remedy,
            format!("1. {}\n{KEEP_13_5}", REMOVE_13_5.replace("\n3. ", "\n2. "))
        );
    }

    #[test]
    fn both_toolkit_remedies_name_the_drivers_toolkit() {
        // 13.2, refused as broken, and 13.5, newer than the driver, each
        // brought by the unversioned meta-package.
        for host in [
            FakeToolchain::before_the_upgrade(),
            newer_than_the_driver(&["cuda-toolkit", "cuda-toolkit-13-5"]),
        ] {
            let inspection = inspect(&host, Some(&gpu()));
            assert_eq!(
                check(&inspection, "Toolkit packages").remedy.as_deref(),
                Some("1. sudo dnf install cuda-toolkit-13-4\n2. sudo dnf remove cuda-toolkit")
            );
            let toolkit = check(&inspection, "CUDA toolkit").remedy.clone().unwrap();
            assert!(
                toolkit.contains("sudo dnf install cuda-toolkit-13-4"),
                "{toolkit}"
            );
        }
    }

    #[test]
    fn a_repository_offering_driver_packages_warns() {
        let mut host = FakeToolchain::healthy();
        host.repositories = Probed::Found(vec![CudaRepository {
            id: "cuda-fedora44-x86_64".into(),
            driver_packages: vec![
                "kmod-nvidia-open-dkms".into(),
                "nvidia-driver".into(),
                "nvidia-settings".into(),
                "xorg-x11-nvidia".into(),
            ],
        }]);
        let inspection = inspect(&host, Some(&gpu()));
        let repositories = check(&inspection, "CUDA repositories");
        assert_eq!(repositories.status, Status::Warn);
        assert_eq!(
            repositories.detail,
            "cuda-fedora44-x86_64 offers 4 driver packages (kmod-nvidia-open-dkms, nvidia-driver, nvidia-settings, …): an update could replace the RPM Fusion driver"
        );
        assert_eq!(repositories.remedy.as_deref(), Some(EXCLUDE));
        assert!(
            inspection.toolchain.is_some(),
            "a warning does not refuse a build"
        );
    }

    #[test]
    fn three_driver_packages_are_all_named() {
        let mut host = FakeToolchain::healthy();
        host.repositories = Probed::Found(vec![CudaRepository {
            id: "cuda-fedora44-x86_64".into(),
            driver_packages: vec![
                "kmod-nvidia-open-dkms".into(),
                "nvidia-driver".into(),
                "nvidia-settings".into(),
            ],
        }]);
        let inspection = inspect(&host, Some(&gpu()));
        assert_eq!(
            check(&inspection, "CUDA repositories").detail,
            "cuda-fedora44-x86_64 offers 3 driver packages (kmod-nvidia-open-dkms, nvidia-driver, \
             nvidia-settings): an update could replace the RPM Fusion driver"
        );
    }

    #[test]
    fn repositories_without_driver_packages_pass() {
        let inspection = inspect(&FakeToolchain::healthy(), Some(&gpu()));
        assert_eq!(
            check(&inspection, "CUDA repositories").detail,
            "cuda-fedora44-x86_64: driver packages excluded"
        );
        let mut host = FakeToolchain::healthy();
        host.repositories = Probed::Found(vec![]);
        let inspection = inspect(&host, Some(&gpu()));
        assert_eq!(
            check(&inspection, "CUDA repositories").detail,
            "none enabled"
        );
        host.repositories = Probed::Missing;
        let inspection = inspect(&host, Some(&gpu()));
        assert_eq!(check(&inspection, "CUDA repositories").status, Status::Skip);
    }

    #[test]
    fn an_unversioned_toolkit_package_warns_with_its_replacement() {
        let mut host = FakeToolchain::healthy();
        host.toolkit_packages = Probed::Found(vec![
            "cuda-toolkit".into(),
            "cuda-toolkit-13".into(),
            "cuda-toolkit-13-4".into(),
        ]);
        let inspection = inspect(&host, Some(&gpu()));
        let packages = check(&inspection, "Toolkit packages");
        assert_eq!(packages.status, Status::Warn);
        assert!(
            packages
                .detail
                .starts_with("cuda-toolkit, cuda-toolkit-13 installed")
        );
        assert_eq!(
            packages.remedy.as_deref(),
            Some(
                "1. sudo dnf install cuda-toolkit-13-4\n2. sudo dnf remove cuda-toolkit cuda-toolkit-13"
            )
        );
    }

    #[test]
    fn an_unversioned_package_without_a_toolkit_keeps_the_drivers_release() {
        let mut host = FakeToolchain::healthy();
        host.toolkit = Probed::Missing;
        host.toolkit_packages = Probed::Found(vec!["cuda-toolkit".into()]);
        let inspection = inspect(&host, Some(&gpu()));
        assert_eq!(
            check(&inspection, "Toolkit packages").remedy.as_deref(),
            Some("1. sudo dnf install cuda-toolkit-13-4\n2. sudo dnf remove cuda-toolkit")
        );
    }

    #[test]
    fn an_unversioned_package_keeps_the_installed_toolkits_release() {
        let mut host = FakeToolchain::healthy();
        host.toolkit = Probed::Found(toolkit(Version::new(13, 1, 80)));
        host.toolkit_packages = Probed::Found(vec!["cuda-toolkit".into()]);
        let inspection = inspect(&host, Some(&gpu()));
        assert_eq!(
            check(&inspection, "Toolkit packages").remedy.as_deref(),
            Some("1. sudo dnf install cuda-toolkit-13-1\n2. sudo dnf remove cuda-toolkit")
        );
    }

    #[test]
    fn the_packages_of_the_drivers_and_the_toolkits_releases_are_asked() {
        let mut host = FakeToolchain::healthy();
        host.toolkit = Probed::Found(toolkit(Version::new(13, 1, 80)));
        inspect(&host, Some(&gpu()));
        assert_eq!(*host.asked.borrow(), [(13, 4), (13, 1)]);
    }

    #[test]
    fn every_probe_that_fails_is_named_in_its_check() {
        let mut host = FakeToolchain::healthy();
        host.toolkit = Probed::Failed("x".into());
        host.repositories = Probed::Failed("x".into());
        host.toolkit_packages = Probed::Failed("x".into());
        host.compilers = Probed::Failed("x".into());
        host.cmake = Probed::Failed("x".into());
        host.git = Probed::Failed("x".into());
        let inspection = inspect(&host, Some(&gpu()));
        let outcomes: Vec<(&str, Status, &str, Option<&str>)> = inspection
            .checks
            .iter()
            .map(|check| {
                (
                    check.name.as_str(),
                    check.status,
                    check.detail.as_str(),
                    check.remedy.as_deref(),
                )
            })
            .collect();
        let install = format!(
            "1. sudo dnf config-manager addrepo --from-repofile=https://developer.download.nvidia.com/compute/cuda/repos/fedora44/x86_64/cuda-fedora44.repo\n\
             2. {EXCLUDE}\n\
             3. sudo dnf install cuda-toolkit-13-4"
        );
        assert_eq!(
            outcomes,
            [
                (
                    "CUDA toolkit",
                    Status::Fail,
                    "nvcc: x",
                    Some(install.as_str())
                ),
                ("CUDA repositories", Status::Warn, "dnf: x", None),
                ("Toolkit packages", Status::Warn, "rpm: x", None),
                ("Host compiler", Status::Fail, "g++: x", None),
                ("CMake", Status::Fail, "cmake: x", None),
                ("Git", Status::Fail, "git: x", None),
            ]
        );
        assert_eq!(inspection.toolchain, None);
    }

    #[test]
    fn build_tools_at_their_minimum_pass() {
        let mut host = FakeToolchain::healthy();
        let (cmake, git) = (Tool::Cmake.minimum().unwrap(), Tool::Git.minimum().unwrap());
        host.cmake = Probed::Found(cmake);
        host.git = Probed::Found(git);
        let inspection = inspect(&host, Some(&gpu()));
        for (name, minimum) in [("CMake", cmake), ("Git", git)] {
            let check = check(&inspection, name);
            assert_eq!(
                (check.status, check.detail.as_str()),
                (Status::Ok, minimum.to_string().as_str()),
                "{name}"
            );
        }
    }

    #[test]
    fn versioned_toolkit_packages_pass() {
        let inspection = inspect(&FakeToolchain::healthy(), Some(&gpu()));
        let packages = check(&inspection, "Toolkit packages");
        assert_eq!(
            (packages.status, packages.detail.as_str()),
            (Status::Ok, "cuda-toolkit-13-4")
        );
    }

    #[test]
    fn a_healthy_machine_settles_the_toolchain() {
        let inspection = inspect(&FakeToolchain::healthy(), Some(&gpu()));
        assert!(
            inspection.checks.iter().all(|c| c.status == Status::Ok),
            "{:#?}",
            inspection.checks
        );
        let toolchain = inspection.toolchain.unwrap();
        let flags = cmake_flags(&LLAMA_CPP, &toolchain, &BuildId::parse("v1.0.0").unwrap());
        assert_eq!(
            flags["CMAKE_CUDA_COMPILER"],
            "/usr/local/cuda-13.4/bin/nvcc"
        );
        assert_eq!(toolchain.toolkit(), Version::new(13, 4, 92));
        assert_eq!(toolchain.compiler().command, "g++-15");
        assert_eq!(flags["CMAKE_C_COMPILER"], "/usr/bin/gcc-15");
    }

    #[test]
    fn any_failed_check_leaves_no_toolchain() {
        let mut broken = FakeToolchain::healthy();
        broken.toolkit = Probed::Found(toolkit(Version::new(13, 2, 86)));
        let mut old_git = FakeToolchain::healthy();
        old_git.git = Probed::Found(Version::new(2, 17, 0));
        let mut no_cmake = FakeToolchain::healthy();
        no_cmake.cmake = Probed::Missing;
        for host in [broken, old_git, no_cmake] {
            let inspection = inspect(&host, Some(&gpu()));
            assert!(inspection.checks.iter().any(|c| c.status == Status::Fail));
            assert_eq!(inspection.toolchain, None);
        }
    }

    #[test]
    fn a_missing_c_compiler_fails_the_compiler_check() {
        let mut host = FakeToolchain::healthy();
        host.files.remove(Path::new("/usr/bin/gcc-15"));
        let inspection = inspect(&host, Some(&gpu()));
        let compiler = check(&inspection, "Host compiler");
        assert_eq!(compiler.status, Status::Fail);
        assert_eq!(
            compiler.detail,
            "no C compiler beside g++-15 (/usr/bin/g++-15)"
        );
        assert_eq!(compiler.remedy.as_deref(), Some("sudo dnf install gcc15"));
        assert_eq!(inspection.toolchain, None);
    }

    #[test]
    fn an_unknown_gcc_limit_builds_with_the_default_compiler() {
        let mut host = FakeToolchain::healthy();
        let mut kit = toolkit(Version::new(13, 4, 92));
        kit.max_gcc_major = Probed::Failed("unreadable".into());
        host.toolkit = Probed::Found(kit);
        let inspection = inspect(&host, Some(&gpu()));
        assert_eq!(check(&inspection, "Host compiler").status, Status::Warn);
        let toolchain = inspection.toolchain.unwrap();
        assert_eq!(toolchain.compiler().command, "g++");
        assert_eq!(
            cmake_flags(&LLAMA_CPP, &toolchain, &BuildId::parse("v1.0.0").unwrap())["CMAKE_C_COMPILER"],
            "/usr/bin/gcc"
        );
    }

    #[test]
    fn a_non_utf8_toolkit_path_is_reported() {
        use std::ffi::OsStr;
        use std::os::unix::ffi::OsStrExt;
        let mut host = FakeToolchain::healthy();
        let mut kit = toolkit(Version::new(13, 4, 92));
        kit.nvcc = PathBuf::from(OsStr::from_bytes(b"/opt/cuda-\xff/bin/nvcc"));
        host.toolkit = Probed::Found(kit);
        let inspection = inspect(&host, Some(&gpu()));
        let last = inspection.checks.last().unwrap();
        assert_eq!(
            (last.name.as_str(), last.status),
            ("Toolchain paths", Status::Fail)
        );
        assert_eq!(inspection.toolchain, None);
    }

    #[test]
    fn the_compiler_list_names_every_compiler() {
        let compilers = [
            gxx("g++", Version::new(16, 2, 1)),
            gxx("g++-15", Version::new(15, 3, 1)),
        ];
        assert_eq!(compiler_list(&compilers), "g++ 16.2.1, g++-15 15.3.1");
    }
}
