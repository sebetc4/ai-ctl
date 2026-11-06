//! Installed llama-swap releases on disk.
//!
//! ```text
//! llama-swap/
//!   .lock                     held by every change to the releases
//!   v263/llama-swap
//!   current -> v263
//!   .staging-v264-4242/       a release being installed, out of sight
//! ```
//!
//! A release appears under its tag by a rename, so an installed release is
//! always complete and verified; the `current` link is replaced by a rename
//! too. Hidden entries left by an interrupted run go with the next install.

use std::any::Any;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

use anyhow::{Context, Result, bail, ensure};

use crate::application::ports::{GatewayStore, StagedRelease};
use crate::domain::build::CURRENT;
use crate::domain::gateway::{BINARY, Release, Sha256};

use super::command::{checked, execute};
use super::files::{
    lock, read_link_if_present, remove_dir_if_present, remove_hidden, replace_link,
};

/// `sha256sum` and `tar` work on a local archive of a few dozen MB.
const TOOL_TIMEOUT: Duration = Duration::from_secs(60);

/// The releases under `root`.
pub struct FsGatewayStore {
    root: PathBuf,
}

impl FsGatewayStore {
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }

    fn release_dir(&self, release: Release) -> PathBuf {
        self.root.join(release.to_string())
    }

    /// A hidden name in the root, unique to this process.
    fn hidden(&self, kind: &str, release: Release) -> PathBuf {
        self.root
            .join(format!(".{kind}-{release}-{}", std::process::id()))
    }
}

/// Runs a tool on a local file and returns what it printed.
fn tool(command: Command) -> Result<String> {
    let program = command.get_program().to_string_lossy().into_owned();
    let missing = format!("{program} is not installed");
    let output = checked(execute(command, Some(TOOL_TIMEOUT)), &missing)?;
    ensure!(output.success, "{program} failed: {}", output.failure());
    Ok(output.stdout)
}

impl GatewayStore for FsGatewayStore {
    fn lock(&self) -> Result<Box<dyn Any>> {
        lock(&self.root, "installing llama-swap")
    }

    fn is_installed(&self, release: Release) -> Result<bool> {
        let binary = self.binary(release);
        match fs::metadata(&binary) {
            Ok(metadata) => Ok(metadata.is_file()),
            Err(err) if err.kind() == io::ErrorKind::NotFound => Ok(false),
            Err(err) => Err(err).with_context(|| format!("cannot read {}", binary.display())),
        }
    }

    fn current(&self) -> Result<Option<Release>> {
        let link = self.root.join(CURRENT);
        let Some(target) = read_link_if_present(&link)? else {
            return Ok(None);
        };
        let release = target
            .to_str()
            .and_then(|name| Release::parse(name).ok())
            .filter(|release| release.to_string() == target.as_os_str().to_string_lossy());
        match release {
            Some(release) => Ok(Some(release)),
            None => bail!(
                "{} points at {}, not at a release of its directory",
                link.display(),
                target.display()
            ),
        }
    }

    fn binary(&self, release: Release) -> PathBuf {
        self.release_dir(release).join(BINARY)
    }

    fn stage(&self, release: Release) -> Result<StagedRelease> {
        remove_hidden(&self.root)?;
        let dir = self.hidden("staging", release);
        fs::create_dir_all(&dir).with_context(|| format!("cannot create {}", dir.display()))?;
        Ok(StagedRelease {
            release,
            archive: dir.join(release.archive()),
            dir,
        })
    }

    fn sha256(&self, file: &Path) -> Result<Sha256> {
        let mut command = Command::new("sha256sum");
        command.arg("--").arg(file);
        let output = tool(command)?;
        let digest = output.split_whitespace().next().unwrap_or_default();
        Sha256::parse(digest)
            .with_context(|| format!("unexpected output of sha256sum for {}", file.display()))
    }

    fn unpack(&self, staged: &StagedRelease) -> Result<PathBuf> {
        let mut command = Command::new("tar");
        command
            .args(["--extract", "--gzip", "--no-same-owner", "--file"])
            .arg(&staged.archive)
            .arg("--directory")
            .arg(&staged.dir)
            .args(["--", BINARY]);
        tool(command).with_context(|| format!("cannot unpack {}", staged.archive.display()))?;
        fs::remove_file(&staged.archive)
            .with_context(|| format!("cannot remove {}", staged.archive.display()))?;
        let binary = staged.dir.join(BINARY);
        let metadata = fs::symlink_metadata(&binary)
            .with_context(|| format!("cannot read {}", binary.display()))?;
        ensure!(
            metadata.is_file(),
            "{BINARY} in {} is not a file",
            staged.archive.display()
        );
        Ok(binary)
    }

    fn publish(&self, staged: &StagedRelease) -> Result<()> {
        let target = self.release_dir(staged.release);
        ensure!(
            !self.is_installed(staged.release)?,
            "{} already exists; remove it to install {} again",
            target.display(),
            staged.release
        );
        // What is left of a release whose binary is gone is no release.
        remove_dir_if_present(&target)?;
        fs::rename(&staged.dir, &target)
            .with_context(|| format!("cannot install {}", target.display()))
    }

    fn discard(&self, staged: &StagedRelease) -> Result<()> {
        remove_dir_if_present(&staged.dir)
    }

    fn set_current(&self, release: Release) -> Result<()> {
        ensure!(
            self.is_installed(release)?,
            "llama-swap {release} is not installed in {}",
            self.root.display()
        );
        replace_link(
            &self.root.join(CURRENT),
            &release.to_string(),
            &self.hidden("current", release),
        )
    }
}

#[cfg(test)]
mod tests {
    use std::os::unix::fs::PermissionsExt;

    use super::*;
    use crate::infrastructure::command;

    struct Fixture {
        root: tempfile::TempDir,
        store: FsGatewayStore,
    }

    fn fixture() -> Fixture {
        let root = tempfile::tempdir().unwrap();
        Fixture {
            store: FsGatewayStore::new(root.path().join("llama-swap")),
            root,
        }
    }

    fn release(text: &str) -> Release {
        Release::parse(text).unwrap()
    }

    impl Fixture {
        /// Stages `release` with an archive holding a `llama-swap` script and
        /// a README, as a release archive does.
        fn stage_archive(&self, release: Release) -> StagedRelease {
            let content = self.root.path().join("content");
            fs::create_dir_all(&content).unwrap();
            let binary = content.join(BINARY);
            fs::write(&binary, "#!/bin/sh\necho version: v263\n").unwrap();
            fs::set_permissions(&binary, fs::Permissions::from_mode(0o755)).unwrap();
            fs::write(content.join("README.md"), "readme").unwrap();
            let staged = self.store.stage(release).unwrap();
            let mut tar = Command::new("tar");
            tar.arg("--create")
                .arg("--gzip")
                .arg("--file")
                .arg(&staged.archive)
                .arg("--directory")
                .arg(&content)
                .args([BINARY, "README.md"]);
            let status = command::spawn(&mut tar).unwrap().wait().unwrap();
            assert!(status.success());
            staged
        }

        fn install(&self, release: Release) {
            let staged = self.stage_archive(release);
            self.store.unpack(&staged).unwrap();
            self.store.publish(&staged).unwrap();
        }

        fn entries(&self) -> Vec<String> {
            let mut names: Vec<String> = fs::read_dir(&self.store.root)
                .unwrap()
                .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
                .collect();
            names.sort();
            names
        }
    }

    #[test]
    fn a_staged_release_is_out_of_sight_until_published() {
        let f = fixture();
        let staged = f.stage_archive(release("263"));
        let name = staged
            .dir
            .file_name()
            .unwrap()
            .to_string_lossy()
            .into_owned();
        assert!(name.starts_with(".staging-v263-"), "{name}");
        assert_eq!(
            staged.archive.file_name().unwrap(),
            "llama-swap_263_linux_amd64.tar.gz"
        );
        assert!(!f.store.is_installed(release("263")).unwrap());

        let binary = f.store.unpack(&staged).unwrap();
        assert_eq!(binary, staged.dir.join(BINARY));
        assert!(!staged.archive.exists());
        assert!(!staged.dir.join("README.md").exists());
        f.store.publish(&staged).unwrap();
        assert!(f.store.is_installed(release("263")).unwrap());
        let installed = f.store.binary(release("263"));
        assert_eq!(installed, f.store.root.join("v263/llama-swap"));
        assert_eq!(crate::infrastructure::files::mode(&installed), 0o755);
        assert_eq!(f.entries(), ["v263"]);
    }

    #[test]
    fn the_digest_is_the_one_sha256sum_computes() {
        let f = fixture();
        let file = f.root.path().join("abc");
        fs::write(&file, "abc").unwrap();
        assert_eq!(
            f.store.sha256(&file).unwrap().to_string(),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
        let message = format!(
            "{:#}",
            f.store.sha256(&f.root.path().join("absent")).unwrap_err()
        );
        assert!(
            message.starts_with("sha256sum failed: sha256sum:"),
            "{message}"
        );
    }

    #[test]
    fn an_archive_without_the_binary_is_refused() {
        let f = fixture();
        let staged = f.store.stage(release("263")).unwrap();
        fs::write(staged.dir.join("README.md"), "readme").unwrap();
        let mut tar = Command::new("tar");
        tar.arg("--create")
            .arg("--gzip")
            .arg("--file")
            .arg(&staged.archive)
            .arg("--directory")
            .arg(&staged.dir)
            .arg("README.md");
        let status = command::spawn(&mut tar).unwrap().wait().unwrap();
        assert!(status.success());
        let message = format!("{:#}", f.store.unpack(&staged).unwrap_err());
        assert!(message.starts_with("cannot unpack"), "{message}");
        assert!(message.contains("tar failed"), "{message}");

        fs::write(&staged.archive, "not an archive").unwrap();
        assert!(f.store.unpack(&staged).is_err());
    }

    #[test]
    fn an_installed_release_is_not_overwritten() {
        let f = fixture();
        f.install(release("263"));
        let staged = f.stage_archive(release("263"));
        f.store.unpack(&staged).unwrap();
        let message = f.store.publish(&staged).unwrap_err().to_string();
        assert!(message.contains("v263 already exists"), "{message}");
        f.store.discard(&staged).unwrap();
        assert_eq!(f.entries(), ["v263"]);
    }

    #[test]
    fn a_release_directory_without_its_binary_gives_way() {
        // Doctor's remedy for a current release whose binary is missing
        // installs that release again.
        let f = fixture();
        f.install(release("263"));
        fs::remove_file(f.store.binary(release("263"))).unwrap();
        assert!(!f.store.is_installed(release("263")).unwrap());
        let staged = f.stage_archive(release("263"));
        f.store.unpack(&staged).unwrap();
        let published = f.store.publish(&staged);
        assert!(published.is_ok(), "{published:?}");
        assert!(f.store.is_installed(release("263")).unwrap());
        assert_eq!(f.entries(), ["v263"]);
    }

    #[test]
    fn leftovers_of_an_interrupted_run_go_with_the_next_install() {
        let f = fixture();
        f.install(release("262"));
        fs::create_dir_all(f.store.root.join(".staging-v263-1/sub")).unwrap();
        std::os::unix::fs::symlink("v262", f.store.root.join(".current-v262-1")).unwrap();
        let _guard = f.store.lock().unwrap();
        f.install(release("263"));
        assert_eq!(f.entries(), [".lock", "v262", "v263"]);
    }

    #[test]
    fn the_current_link_switches_between_installed_releases() {
        let f = fixture();
        assert_eq!(f.store.current().unwrap(), None);
        f.install(release("262"));
        f.install(release("263"));
        f.store.set_current(release("263")).unwrap();
        assert_eq!(f.store.current().unwrap(), Some(release("263")));
        f.store.set_current(release("262")).unwrap();
        assert_eq!(f.store.current().unwrap(), Some(release("262")));
        let link = f.store.root.join(CURRENT);
        assert_eq!(fs::read_link(&link).unwrap(), Path::new("v262"));
        assert!(link.join(BINARY).is_file());
        let message = f.store.set_current(release("9")).unwrap_err().to_string();
        assert!(
            message.starts_with("llama-swap v9 is not installed"),
            "{message}"
        );
        assert_eq!(f.entries(), ["current", "v262", "v263"]);
    }

    #[test]
    fn a_link_pointing_elsewhere_is_reported() {
        let f = fixture();
        f.install(release("263"));
        for target in ["/usr/bin", "263", "v0263"] {
            let link = f.store.root.join(CURRENT);
            let _ = fs::remove_file(&link);
            std::os::unix::fs::symlink(target, &link).unwrap();
            let message = f.store.current().unwrap_err().to_string();
            assert!(
                message.contains("not at a release of its directory"),
                "{target}: {message}"
            );
        }
    }

    #[test]
    fn a_directory_without_its_binary_is_not_installed() {
        let f = fixture();
        fs::create_dir_all(f.store.root.join("v263")).unwrap();
        assert!(!f.store.is_installed(release("263")).unwrap());
        assert!(f.store.set_current(release("263")).is_err());
    }

    #[test]
    fn a_second_lock_is_refused() {
        let f = fixture();
        let _first = f.store.lock().unwrap();
        let message = f.store.lock().unwrap_err().to_string();
        assert!(
            message.starts_with("another ai-ctl is installing llama-swap"),
            "{message}"
        );
    }
}
