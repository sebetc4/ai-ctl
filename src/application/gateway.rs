//! `gateway install`: installs the llama-swap release ai-ctl pins, verified
//! against the digest it carries, and makes it current.

use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail, ensure};

use crate::domain::gateway::{BINARY, PINNED, Release, Sha256, reports};

use super::build::{current_or_none, summary};
use super::ports::{Downloads, GatewayStore, Programs, StagedRelease};

/// A step of an installation, for the user to follow.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Event {
    Downloading {
        url: String,
    },
    /// The archive's SHA-256 matches the digest ai-ctl pins.
    Verified {
        sha256: Sha256,
    },
    /// Running the new binary.
    Testing,
    /// Something went wrong that does not fail the installation.
    Warning(String),
}

/// What an installation did.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Outcome {
    /// Downloaded, verified and installed.
    Downloaded,
    /// Installed already; made current again.
    Reused,
    /// Installed already, and current.
    Unchanged,
}

/// A release installed and current.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Installation {
    pub release: Release,
    pub binary: PathBuf,
    pub outcome: Outcome,
    /// The release that was current before, if another one.
    pub previous: Option<Release>,
}

/// The installation use case, with the ports it drives.
pub struct Installer<'a> {
    pub downloads: &'a dyn Downloads,
    pub store: &'a dyn GatewayStore,
    pub programs: &'a dyn Programs,
}

impl Installer<'_> {
    /// Installs the pinned release and makes it current: llama-swap is
    /// updated only through ai-ctl's pin. A release not installed yet is
    /// downloaded, its archive's SHA-256 checked against the pinned digest,
    /// the only one trusted, and its binary run; nothing is installed when a
    /// step fails.
    pub fn install(&self, events: &mut dyn FnMut(Event)) -> Result<Installation> {
        ensure!(
            cfg!(all(target_os = "linux", target_arch = "x86_64")),
            "ai-ctl installs llama-swap for Linux on x86_64 only"
        );
        let release = PINNED.release;
        let _lock = self.store.lock()?;
        // A `current` link that names no release is replaced below.
        let previous = current_or_none(
            self.store.current(),
            "the installed release",
            &mut |warning| events(Event::Warning(warning)),
        );

        let expected = Sha256::parse(PINNED.sha256)?;
        // A release an older ai-ctl installed against another digest, or
        // one installed by hand, is downloaded and checked again.
        let checked = self.store.is_installed(release)?
            && self.store.digest(release)?.as_ref() == Some(&expected);
        let outcome = if !checked {
            let staged = self.store.stage(release)?;
            let installed = self
                .fetch(&staged, &expected, events)
                .and_then(|()| self.store.publish(&staged, &expected));
            if let Err(err) = installed {
                if let Err(discard) = self.store.discard(&staged) {
                    events(Event::Warning(format!("{discard:#}")));
                }
                return Err(err);
            }
            Outcome::Downloaded
        } else if previous == Some(release) {
            Outcome::Unchanged
        } else {
            Outcome::Reused
        };
        if outcome != Outcome::Unchanged {
            self.store
                .set_current(release)
                .with_context(|| format!("llama-swap {release} is installed but not current"))?;
        }
        Ok(Installation {
            release,
            binary: self.store.binary(release),
            outcome,
            previous: previous.filter(|previous| *previous != release),
        })
    }

    /// Downloads the archive into the staged directory, checks it against
    /// `expected`, unpacks the binary and runs it.
    fn fetch(
        &self,
        staged: &StagedRelease,
        expected: &Sha256,
        events: &mut dyn FnMut(Event),
    ) -> Result<()> {
        let release = staged.release;
        let archive = release.archive();
        let url = release.url(&archive);
        events(Event::Downloading { url: url.clone() });
        self.downloads.download(&url, &staged.archive)?;
        let actual = self.store.sha256(&staged.archive)?;
        if actual != *expected {
            bail!(
                "{archive} has the SHA-256 {actual}, not the expected {expected}: the download \
                 is corrupt or was tampered with"
            );
        }
        events(Event::Verified { sha256: actual });
        let binary = self.store.unpack(staged)?;
        events(Event::Testing);
        self.smoke_test(&binary, release)
    }

    /// The binary runs, and is the release it was downloaded as.
    fn smoke_test(&self, binary: &Path, release: Release) -> Result<()> {
        let output = self.programs.run(binary, &["-version"])?;
        ensure!(
            output.success,
            "the new {BINARY} does not run: `{BINARY} -version` failed: {}",
            summary(&output)
        );
        ensure!(
            reports(&output.stdout, release),
            "the new {BINARY} is not {release}: `{BINARY} -version` printed {}",
            summary(&output)
        );
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::any::Any;
    use std::cell::{Cell, RefCell};

    use anyhow::bail;

    use super::*;
    use crate::application::fakes::printed;
    use crate::application::ports::ProgramOutput;

    const V262_SHA256: &str = "871b3ed7891f8ce057428f5a34d38298e06c36c6848c1136e05308eb4303af50";

    /// The release server: every download succeeds, unless `fail`.
    #[derive(Default)]
    struct FakeDownloads {
        downloads: RefCell<Vec<(String, PathBuf)>>,
        fail: bool,
    }

    impl Downloads for FakeDownloads {
        fn download(&self, url: &str, to: &Path) -> Result<()> {
            if self.fail {
                bail!("cannot download {url}: curl: (6) Could not resolve host: github.com");
            }
            self.downloads
                .borrow_mut()
                .push((url.to_owned(), to.to_owned()));
            Ok(())
        }
    }

    /// Releases in memory; `locked` stands for another ai-ctl holding the
    /// lock, `digest` is what any downloaded archive hashes to, and each
    /// `_fails` a failure of that method.
    struct FakeStore {
        locked: bool,
        /// The `current` link names no release, until it is replaced: a
        /// state that `set_current` clears, not a `current_fails`.
        bad_link: Cell<bool>,
        installed: RefCell<Vec<Release>>,
        /// Installed releases whose directory records no digest, as an
        /// older ai-ctl's `--version` left them.
        unrecorded: RefCell<Vec<Release>>,
        current: RefCell<Option<Release>>,
        digest: Sha256,
        staged: RefCell<Vec<Release>>,
        discarded: Cell<usize>,
        publish_fails: bool,
        discard_fails: bool,
        set_current_fails: bool,
    }

    impl FakeStore {
        fn new() -> Self {
            Self {
                locked: false,
                bad_link: Cell::new(false),
                installed: RefCell::new(vec![]),
                unrecorded: RefCell::new(vec![]),
                current: RefCell::new(None),
                digest: Sha256::parse(PINNED.sha256).unwrap(),
                staged: RefCell::new(vec![]),
                discarded: Cell::new(0),
                publish_fails: false,
                discard_fails: false,
                set_current_fails: false,
            }
        }
    }

    impl GatewayStore for FakeStore {
        fn lock(&self) -> Result<Box<dyn Any>> {
            ensure!(!self.locked, "another ai-ctl is installing llama-swap");
            Ok(Box::new(()))
        }
        fn is_installed(&self, release: Release) -> Result<bool> {
            Ok(self.installed.borrow().contains(&release))
        }
        fn digest(&self, release: Release) -> Result<Option<Sha256>> {
            let recorded = self.installed.borrow().contains(&release)
                && !self.unrecorded.borrow().contains(&release);
            Ok(recorded.then(|| Sha256::parse(PINNED.sha256).unwrap()))
        }
        fn current(&self) -> Result<Option<Release>> {
            ensure!(
                !self.bad_link.get(),
                "/data/llama-swap/current points at ./v263, not at a release of its directory"
            );
            Ok(*self.current.borrow())
        }
        fn binary(&self, release: Release) -> PathBuf {
            PathBuf::from(format!("/data/llama-swap/{release}/llama-swap"))
        }
        fn stage(&self, release: Release) -> Result<StagedRelease> {
            self.staged.borrow_mut().push(release);
            Ok(StagedRelease {
                release,
                dir: "/data/llama-swap/.staging".into(),
                archive: format!("/data/llama-swap/.staging/{}", release.archive()).into(),
            })
        }
        fn sha256(&self, _: &Path) -> Result<Sha256> {
            Ok(self.digest.clone())
        }
        fn unpack(&self, _: &StagedRelease) -> Result<PathBuf> {
            Ok("/data/llama-swap/.staging/llama-swap".into())
        }
        fn publish(&self, staged: &StagedRelease, _: &Sha256) -> Result<()> {
            ensure!(
                !self.publish_fails,
                "cannot install /data/llama-swap/{}: Permission denied (os error 13)",
                staged.release
            );
            self.unrecorded
                .borrow_mut()
                .retain(|release| *release != staged.release);
            if !self.installed.borrow().contains(&staged.release) {
                self.installed.borrow_mut().push(staged.release);
            }
            Ok(())
        }
        fn discard(&self, _: &StagedRelease) -> Result<()> {
            self.discarded.set(self.discarded.get() + 1);
            ensure!(
                !self.discard_fails,
                "cannot remove /data/llama-swap/.staging: Permission denied (os error 13)"
            );
            Ok(())
        }
        fn set_current(&self, release: Release) -> Result<()> {
            ensure!(
                !self.set_current_fails,
                "cannot update /data/llama-swap/current: Permission denied (os error 13)"
            );
            self.bad_link.set(false);
            *self.current.borrow_mut() = Some(release);
            Ok(())
        }
    }

    #[test]
    fn install_replaces_a_current_link_that_names_no_release() {
        let world = World::new();
        world.store.bad_link.set(true);
        let (installation, events) = world.install();
        let installation = installation.unwrap();
        assert_eq!(installation.previous, None);
        assert_eq!(*world.store.current.borrow(), Some(PINNED.release));
        assert!(
            events.contains(&Event::Warning(
                "/data/llama-swap/current points at ./v263, not at a release of its directory; the installed release replaces it".into()
            )),
            "{events:?}"
        );
    }

    /// A llama-swap binary answering `-version`.
    struct FakeLlamaSwap(ProgramOutput);

    impl FakeLlamaSwap {
        fn of(release: Release) -> Self {
            Self(printed(
                true,
                &format!("version: {release} (e33e97b), built at 2026-10-10T06:17:54Z\n"),
                "",
            ))
        }
    }

    impl Programs for FakeLlamaSwap {
        fn run(&self, _: &Path, args: &[&str]) -> Result<ProgramOutput> {
            assert_eq!(args, ["-version"]);
            Ok(self.0.clone())
        }
    }

    struct World {
        downloads: FakeDownloads,
        store: FakeStore,
        programs: FakeLlamaSwap,
    }

    impl World {
        fn new() -> Self {
            Self {
                downloads: FakeDownloads::default(),
                store: FakeStore::new(),
                programs: FakeLlamaSwap::of(PINNED.release),
            }
        }

        fn install(&self) -> (Result<Installation>, Vec<Event>) {
            let installer = Installer {
                downloads: &self.downloads,
                store: &self.store,
                programs: &self.programs,
            };
            let mut events = Vec::new();
            let result = installer.install(&mut |event| events.push(event));
            (result, events)
        }
    }

    fn release_of(text: &str) -> Release {
        Release::parse(text).unwrap()
    }

    fn message(result: Result<Installation>) -> String {
        format!("{:#}", result.unwrap_err())
    }

    #[test]
    fn the_pinned_release_is_installed_against_its_own_digest() {
        let world = World::new();
        let (result, events) = world.install();
        let installation = result.unwrap();
        assert_eq!(
            installation,
            Installation {
                release: PINNED.release,
                binary: "/data/llama-swap/v271/llama-swap".into(),
                outcome: Outcome::Downloaded,
                previous: None,
            }
        );
        let url = "https://github.com/mostlygeek/llama-swap/releases/download/v271/llama-swap_271_linux_amd64.tar.gz";
        assert_eq!(
            events,
            [
                Event::Downloading { url: url.into() },
                Event::Verified {
                    sha256: Sha256::parse(PINNED.sha256).unwrap(),
                },
                Event::Testing,
            ]
        );
        assert_eq!(
            *world.downloads.downloads.borrow(),
            [(
                url.to_owned(),
                "/data/llama-swap/.staging/llama-swap_271_linux_amd64.tar.gz".into()
            )]
        );
        assert_eq!(*world.store.current.borrow(), Some(PINNED.release));
    }

    #[test]
    fn the_pinned_release_replaces_any_other_current_one() {
        for other in ["v262", "v272"] {
            let world = World::new();
            *world.store.installed.borrow_mut() = vec![release_of(other)];
            *world.store.current.borrow_mut() = Some(release_of(other));
            let installation = world.install().0.unwrap();
            assert_eq!(installation.release, PINNED.release);
            assert_eq!(installation.outcome, Outcome::Downloaded);
            assert_eq!(installation.previous, Some(release_of(other)));
            assert_eq!(*world.store.current.borrow(), Some(PINNED.release));
        }
    }

    #[test]
    fn a_pinned_release_never_checked_against_the_pin_is_downloaded_and_checked_again() {
        let world = World::new();
        *world.store.installed.borrow_mut() = vec![PINNED.release];
        *world.store.unrecorded.borrow_mut() = vec![PINNED.release];
        *world.store.current.borrow_mut() = Some(PINNED.release);
        let (result, events) = world.install();
        let installation = result.unwrap();
        assert_eq!(installation.outcome, Outcome::Downloaded);
        assert_eq!(world.downloads.downloads.borrow().len(), 1);
        assert!(
            events.contains(&Event::Verified {
                sha256: Sha256::parse(PINNED.sha256).unwrap(),
            }),
            "{events:?}"
        );
        assert!(world.store.unrecorded.borrow().is_empty());
    }

    #[test]
    fn a_digest_mismatch_installs_nothing() {
        let mut world = World::new();
        world.store.digest = Sha256::parse(V262_SHA256).unwrap();
        let (result, events) = world.install();
        let message = message(result);
        assert!(
            message.contains(&format!(
                "has the SHA-256 {V262_SHA256}, not the expected {}",
                PINNED.sha256
            )),
            "{message}"
        );
        assert!(message.contains("tampered with"), "{message}");
        assert!(!events.contains(&Event::Testing));
        assert!(world.store.installed.borrow().is_empty());
        assert_eq!(world.store.discarded.get(), 1);
        assert_eq!(*world.store.current.borrow(), None);
    }

    #[test]
    fn a_failed_download_installs_nothing() {
        let mut world = World::new();
        world.downloads.fail = true;
        let message = message(world.install().0);
        assert!(message.contains("Could not resolve host"), "{message}");
        assert_eq!(world.store.discarded.get(), 1);
        assert!(world.store.installed.borrow().is_empty());
    }

    #[test]
    fn a_release_that_cannot_be_installed_or_discarded_warns_of_the_discard() {
        let mut world = World::new();
        world.store.publish_fails = true;
        world.store.discard_fails = true;
        let (result, events) = world.install();
        assert_eq!(
            message(result),
            "cannot install /data/llama-swap/v271: Permission denied (os error 13)"
        );
        assert_eq!(
            events.last(),
            Some(&Event::Warning(
                "cannot remove /data/llama-swap/.staging: Permission denied (os error 13)".into()
            ))
        );
        assert_eq!(world.store.discarded.get(), 1);
        assert!(world.store.installed.borrow().is_empty());
        assert_eq!(*world.store.current.borrow(), None);
    }

    #[test]
    fn a_release_that_cannot_be_made_current_says_it_is_installed() {
        let mut world = World::new();
        *world.store.installed.borrow_mut() = vec![release_of("v262")];
        *world.store.current.borrow_mut() = Some(release_of("v262"));
        world.store.set_current_fails = true;
        let refusal = "llama-swap v271 is installed but not current: cannot update \
                       /data/llama-swap/current: Permission denied (os error 13)";
        // Downloaded, then installed already.
        for installed in [
            vec![release_of("v262")],
            vec![release_of("v262"), PINNED.release],
        ] {
            assert_eq!(*world.store.installed.borrow(), installed);
            assert_eq!(message(world.install().0), refusal);
            assert_eq!(*world.store.current.borrow(), Some(release_of("v262")));
        }
        assert_eq!(world.store.discarded.get(), 0);
    }

    #[test]
    fn a_binary_that_does_not_run_is_discarded() {
        let mut world = World::new();
        world.programs = FakeLlamaSwap(printed(false, "", "exec format error\n"));
        let message = message(world.install().0);
        assert_eq!(
            message,
            "the new llama-swap does not run: `llama-swap -version` failed: exec format error"
        );
        assert_eq!(world.store.discarded.get(), 1);
        assert_eq!(*world.store.current.borrow(), None);
    }

    #[test]
    fn a_binary_of_another_release_is_discarded() {
        let mut world = World::new();
        world.programs = FakeLlamaSwap::of(release_of("v262"));
        let message = message(world.install().0);
        assert!(
            message.starts_with(
                "the new llama-swap is not v271: `llama-swap -version` printed version: v262"
            ),
            "{message}"
        );
        assert_eq!(world.store.discarded.get(), 1);
    }

    #[test]
    fn an_installed_release_is_made_current_without_a_download() {
        let world = World::new();
        *world.store.installed.borrow_mut() = vec![release_of("v262"), PINNED.release];
        *world.store.current.borrow_mut() = Some(release_of("v262"));
        let (result, events) = world.install();
        let installation = result.unwrap();
        assert_eq!(installation.outcome, Outcome::Reused);
        assert_eq!(installation.previous, Some(release_of("v262")));
        assert!(events.is_empty());
        assert!(world.store.staged.borrow().is_empty());
        assert_eq!(*world.store.current.borrow(), Some(PINNED.release));
    }

    #[test]
    fn the_current_release_is_left_as_it_is() {
        let world = World::new();
        *world.store.installed.borrow_mut() = vec![PINNED.release];
        *world.store.current.borrow_mut() = Some(PINNED.release);
        let installation = world.install().0.unwrap();
        assert_eq!(installation.outcome, Outcome::Unchanged);
        assert_eq!(installation.previous, None);
        assert!(world.store.staged.borrow().is_empty());
    }

    #[test]
    fn an_installation_waits_for_no_one() {
        let mut world = World::new();
        world.store.locked = true;
        let message = message(world.install().0);
        assert_eq!(message, "another ai-ctl is installing llama-swap");
        assert!(world.store.staged.borrow().is_empty());
    }

    #[test]
    fn the_current_release_is_not_linked_again() {
        let mut world = World::new();
        *world.store.installed.borrow_mut() = vec![PINNED.release];
        *world.store.current.borrow_mut() = Some(PINNED.release);
        world.store.set_current_fails = true;
        assert_eq!(world.install().0.unwrap().outcome, Outcome::Unchanged);
    }
}
