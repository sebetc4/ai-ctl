//! Installed builds on disk.
//!
//! ```text
//! builds/
//!   .lock                     held by every change to the builds
//!   llama.cpp/
//!     b11538/                 llama-server, build.json, build.log
//!     current -> b11538
//!     .staging-b11540-4242/   a build being made, out of sight
//! ```
//!
//! A build appears under its id by a rename, so an installed build is
//! always complete; the `current` link is replaced by a rename too. Hidden
//! directories left by an interrupted run are removed by the next build,
//! once a build moved aside and never replaced is put back.

use std::any::Any;
use std::collections::BTreeMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result, bail, ensure};
use serde::{Deserialize, Serialize};

use crate::application::ports::{BuildStore, Staged, Unreadable};
use crate::domain::build::{Build, BuildId, CURRENT, CommitId};
use crate::domain::machine::{HostCompiler, Version};
use crate::domain::recipe::Patch;
use crate::domain::timestamp::Timestamp;

use super::files::{
    json_format, lock, read_link_if_present, remove_dir_if_present, remove_hidden, replace_link,
};

/// The metadata file in each build's directory.
pub const METADATA: &str = "build.json";
/// The build log, copied beside the binary.
const LOG: &str = "build.log";
/// The directory beside the binary that holds a copy of each patch applied
/// to the build's sources.
const PATCHES: &str = "patches";
/// The metadata format this ai-ctl writes: format 2 added `patches`. It
/// reads formats 1 to this one.
// LEGACY: the builds installed before format 2 must stay readable, so
// format 1 is read too, and `patches` defaults below. Format 1 goes once no
// such build is kept or a rebuild has replaced them.
const FORMAT: u32 = 2;

/// `build.json`, as written to disk.
#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Metadata {
    format: u32,
    project: String,
    id: String,
    commit: String,
    toolkit: String,
    host_compiler: Compiler,
    cmake_flags: BTreeMap<String, String>,
    /// Absent from format 1, whose builds have no patch.
    // LEGACY: format 1 must stay readable (see `FORMAT`).
    #[serde(default)]
    patches: Vec<String>,
    built_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Compiler {
    command: String,
    path: PathBuf,
    version: String,
}

/// Serializes `build` as the content of its `build.json`. Fails only on a
/// compiler path that is not valid UTF-8.
fn to_json(build: &Build) -> Result<String> {
    let metadata = Metadata {
        format: FORMAT,
        project: build.project.clone(),
        id: build.id.to_string(),
        commit: build.commit.to_string(),
        toolkit: build.toolkit.to_string(),
        host_compiler: Compiler {
            command: build.host_compiler.command.clone(),
            path: build.host_compiler.path.clone(),
            version: build.host_compiler.version.to_string(),
        },
        cmake_flags: build.cmake_flags.clone(),
        patches: build.patches.clone(),
        built_at: build.built_at.to_string(),
    };
    let mut text =
        serde_json::to_string_pretty(&metadata).context("cannot serialize the build metadata")?;
    text.push('\n');
    Ok(text)
}

/// Reads the content of a `build.json`.
fn from_json(text: &str) -> Result<Build> {
    let format = json_format(text).context("malformed build metadata")?;
    ensure!(
        (1..=FORMAT).contains(&format),
        "build metadata format {format} is not supported (this ai-ctl reads formats 1 and {FORMAT})"
    );
    let metadata: Metadata = serde_json::from_str(text).context("malformed build metadata")?;
    let version = |what: &str, text: &str| {
        Version::parse(text).with_context(|| format!("{what} `{text}` is not a version"))
    };
    Ok(Build {
        project: metadata.project,
        id: BuildId::parse(&metadata.id)?,
        commit: CommitId::parse(&metadata.commit)?,
        toolkit: version("toolkit", &metadata.toolkit)?,
        host_compiler: HostCompiler {
            command: metadata.host_compiler.command,
            path: metadata.host_compiler.path,
            version: version("host compiler version", &metadata.host_compiler.version)?,
        },
        cmake_flags: metadata.cmake_flags,
        patches: metadata.patches,
        built_at: Timestamp::parse(&metadata.built_at).with_context(|| {
            format!(
                "built_at `{}` is not a UTC timestamp such as 2026-10-09T18:02:11Z",
                metadata.built_at
            )
        })?,
    })
}

/// The builds under `root`, one directory per project.
pub struct FsBuildStore {
    root: PathBuf,
}

impl FsBuildStore {
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }

    fn project_dir(&self, project: &str) -> PathBuf {
        self.root.join(project)
    }

    /// A hidden name in the project's directory, unique to this process.
    fn hidden(&self, project: &str, kind: &str, id: &BuildId) -> PathBuf {
        self.project_dir(project)
            .join(format!(".{kind}-{id}-{}", std::process::id()))
    }

    /// Removes what an interrupted run left: staged, replaced or half-removed
    /// builds, and a link not yet renamed into place. A build a kill left
    /// moved aside between `publish`'s two renames is put back first: it is
    /// the only copy of that build.
    fn remove_leftovers(&self, project: &str) -> Result<()> {
        let dir = self.project_dir(project);
        put_back_replaced(&dir)?;
        remove_hidden(&dir)
    }
}

/// Renames each `.replaced-<id>-<pid>` of `dir` back to `<id>` when nothing
/// is there and `.staging-<id>-<pid>` still is: the process was killed
/// between `publish`'s two renames. Once the replacement is in place, the
/// old build is a leftover, maybe half removed, even when the new one was
/// removed since. A `dir` that cannot be read is left to the sweep that
/// follows, which reports it.
fn put_back_replaced(dir: &Path) -> Result<()> {
    let Ok(entries) = fs::read_dir(dir) else {
        return Ok(());
    };
    for entry in entries.flatten() {
        let name = entry.file_name();
        let Some((id, pid)) = name
            .to_str()
            .and_then(|name| name.strip_prefix(".replaced-"))
            .and_then(|rest| rest.rsplit_once('-'))
        else {
            continue;
        };
        let Ok(id) = BuildId::parse(id) else {
            continue;
        };
        let target = dir.join(id.as_str());
        let staged = dir.join(format!(".staging-{id}-{pid}"));
        if target.symlink_metadata().is_err() && staged.symlink_metadata().is_ok() {
            fs::rename(entry.path(), &target)
                .with_context(|| format!("cannot put {} back", target.display()))?;
        }
    }
    Ok(())
}

impl BuildStore for FsBuildStore {
    fn lock(&self) -> Result<Box<dyn Any>> {
        lock(&self.root, "building or changing the builds")
    }

    fn builds(&self, project: &str) -> Result<(Vec<Build>, Vec<Unreadable>)> {
        let dir = self.project_dir(project);
        let entries = match fs::read_dir(&dir) {
            Ok(entries) => entries,
            Err(err) if err.kind() == io::ErrorKind::NotFound => return Ok((vec![], vec![])),
            Err(err) => return Err(err).with_context(|| format!("cannot read {}", dir.display())),
        };
        let mut builds = Vec::new();
        let mut unreadable = Vec::new();
        for entry in entries {
            let entry = entry.with_context(|| format!("cannot read {}", dir.display()))?;
            let name = entry.file_name().to_string_lossy().into_owned();
            let path = entry.path();
            if name.starts_with('.') || name == CURRENT || !path.is_dir() {
                continue;
            }
            match read_build(&path, project, &name) {
                Ok(build) => builds.push(build),
                Err(err) => unreadable.push(Unreadable {
                    dir: path,
                    reason: format!("{err:#}"),
                }),
            }
        }
        Ok((builds, unreadable))
    }

    fn current(&self, project: &str) -> Result<Option<BuildId>> {
        let link = self.project_dir(project).join(CURRENT);
        let Some(target) = read_link_if_present(&link)? else {
            return Ok(None);
        };
        let name = target
            .to_str()
            .filter(|name| !name.contains('/'))
            .with_context(|| {
                format!(
                    "{} points at {}, not at a build of its directory",
                    link.display(),
                    target.display()
                )
            })?;
        Ok(Some(BuildId::parse(name)?))
    }

    fn binary(&self, project: &str, id: &BuildId, name: &str) -> PathBuf {
        self.project_dir(project).join(id.as_str()).join(name)
    }

    fn stage(
        &self,
        build: &Build,
        artifact: &Path,
        log: &Path,
        patches: &[Patch],
    ) -> Result<Staged> {
        self.remove_leftovers(&build.project)?;
        let dir = self.hidden(&build.project, "staging", &build.id);
        let staged = (|| {
            fs::create_dir_all(&dir).with_context(|| format!("cannot create {}", dir.display()))?;
            let name = artifact
                .file_name()
                .with_context(|| format!("{} has no file name", artifact.display()))?;
            let binary = dir.join(name);
            fs::copy(artifact, &binary).with_context(|| {
                format!("cannot copy {} to {}", artifact.display(), binary.display())
            })?;
            fs::copy(log, dir.join(LOG))
                .with_context(|| format!("cannot copy {}", log.display()))?;
            if !patches.is_empty() {
                let copies = dir.join(PATCHES);
                fs::create_dir(&copies)
                    .with_context(|| format!("cannot create {}", copies.display()))?;
                for patch in patches {
                    let copy = copies.join(patch.name);
                    fs::write(&copy, patch.diff)
                        .with_context(|| format!("cannot write {}", copy.display()))?;
                }
            }
            let metadata = dir.join(METADATA);
            fs::write(&metadata, to_json(build)?)
                .with_context(|| format!("cannot write {}", metadata.display()))?;
            Ok(binary)
        })();
        match staged {
            Ok(binary) => Ok(Staged {
                dir,
                binary,
                build: build.clone(),
            }),
            Err(err) => {
                // Best effort: the error above is what matters.
                let _ = remove_dir_if_present(&dir);
                Err(err)
            }
        }
    }

    fn publish(&self, staged: &Staged, replace: bool) -> Result<()> {
        let project = &staged.build.project;
        let target = self.project_dir(project).join(staged.build.id.as_str());
        if target.symlink_metadata().is_err() {
            return fs::rename(&staged.dir, &target)
                .with_context(|| format!("cannot install {}", target.display()));
        }
        ensure!(replace, "{} already exists", target.display());
        let old = self.hidden(project, "replaced", &staged.build.id);
        fs::rename(&target, &old)
            .with_context(|| format!("cannot move {} aside", target.display()))?;
        if let Err(err) = fs::rename(&staged.dir, &target) {
            // Put the old build back; failing that, the next build removes it.
            let _ = fs::rename(&old, &target);
            return Err(err).with_context(|| format!("cannot install {}", target.display()));
        }
        // The new build is in place; a leftover is removed by the next build.
        let _ = remove_dir_if_present(&old);
        Ok(())
    }

    fn discard(&self, staged: &Staged) -> Result<()> {
        remove_dir_if_present(&staged.dir)
    }

    fn set_current(&self, project: &str, id: &BuildId) -> Result<()> {
        let dir = self.project_dir(project);
        ensure!(
            dir.join(id.as_str()).is_dir(),
            "{project} has no build {id} in {}",
            dir.display()
        );
        replace_link(
            &dir.join(CURRENT),
            id.as_str(),
            &self.hidden(project, "current", id),
        )
    }

    fn remove(&self, project: &str, id: &BuildId) -> Result<()> {
        if self.current(project)?.as_ref() == Some(id) {
            bail!("{project} {id} is the current build");
        }
        let target = self.project_dir(project).join(id.as_str());
        let doomed = self.hidden(project, "removing", id);
        // Out of sight first: an interrupted removal leaves no half build.
        fs::rename(&target, &doomed)
            .with_context(|| format!("cannot remove {}", target.display()))?;
        remove_dir_if_present(&doomed)
    }

    fn contains(&self, program: &Path) -> bool {
        // The kernel names a program by its real path, and a removed binary
        // as `<path> (deleted)`, still under the root.
        let root = fs::canonicalize(&self.root).unwrap_or_else(|_| self.root.clone());
        program.starts_with(root)
    }
}

/// The build in `dir`, which must be the build its name and project say.
fn read_build(dir: &Path, project: &str, name: &str) -> Result<Build> {
    let path = dir.join(METADATA);
    let text =
        fs::read_to_string(&path).with_context(|| format!("cannot read {}", path.display()))?;
    let build = from_json(&text).with_context(|| format!("invalid {}", path.display()))?;
    ensure!(
        build.project == project && build.id.as_str() == name,
        "{} describes {} {}",
        path.display(),
        build.project,
        build.id
    );
    Ok(build)
}

/// The current time, for a build's date.
pub fn now() -> Timestamp {
    let seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |elapsed| {
            i64::try_from(elapsed.as_secs()).unwrap_or(i64::MAX)
        });
    Timestamp::from_unix(seconds).unwrap_or(Timestamp::EPOCH)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn build() -> Build {
        Build {
            project: "llama.cpp".into(),
            id: BuildId::parse("b11538").unwrap(),
            commit: CommitId::parse("79e2e74eb11022c1ba2e438df7f0ca2d4c10f8b6").unwrap(),
            toolkit: Version::new(13, 4, 92),
            host_compiler: HostCompiler {
                command: "g++".into(),
                path: PathBuf::from("/usr/bin/g++"),
                version: Version::new(16, 2, 1),
            },
            cmake_flags: BTreeMap::from([
                ("CMAKE_CUDA_ARCHITECTURES".into(), "120a-real".into()),
                ("GGML_CUDA".into(), "ON".into()),
            ]),
            patches: vec![],
            built_at: Timestamp::parse("2026-10-09T18:02:11Z").unwrap(),
        }
    }

    #[test]
    fn metadata_round_trips() {
        let build = build();
        assert_eq!(from_json(&to_json(&build).unwrap()).unwrap(), build);
        let patched = Build {
            patches: vec!["api-key.patch".into(), "listen-failure.patch".into()],
            ..build
        };
        assert_eq!(from_json(&to_json(&patched).unwrap()).unwrap(), patched);
    }

    #[test]
    fn a_build_of_format_1_has_no_patch() {
        let format_1 = edited(|value| {
            value["format"] = 1.into();
            value.as_object_mut().unwrap().remove("patches");
        });
        assert_eq!(from_json(&format_1).unwrap(), build());
    }

    #[test]
    fn metadata_is_readable_json() {
        let text = to_json(&build()).unwrap();
        let value: serde_json::Value = serde_json::from_str(&text).unwrap();
        assert_eq!(value["format"], 2);
        assert_eq!(value["id"], "b11538");
        assert_eq!(value["patches"], serde_json::json!([]));
        assert_eq!(value["toolkit"], "13.4.92");
        assert_eq!(value["host_compiler"]["path"], "/usr/bin/g++");
        assert_eq!(value["cmake_flags"]["GGML_CUDA"], "ON");
        assert_eq!(value["built_at"], "2026-10-09T18:02:11Z");
        assert!(text.ends_with("}\n"));
    }

    fn edited(edit: impl FnOnce(&mut serde_json::Value)) -> String {
        let mut value: serde_json::Value =
            serde_json::from_str(&to_json(&build()).unwrap()).unwrap();
        edit(&mut value);
        value.to_string()
    }

    fn error(text: &str) -> String {
        format!("{:#}", from_json(text).unwrap_err())
    }

    #[test]
    fn another_format_is_refused() {
        let later = "build metadata format 3 is not supported (this ai-ctl reads formats 1 and 2)";
        assert_eq!(error(&edited(|value| value["format"] = 3.into())), later);
        let lto = edited(|value| {
            value["format"] = 3.into();
            value["lto"] = true.into();
        });
        assert_eq!(error(&lto), later);
        let earlier =
            "build metadata format 0 is not supported (this ai-ctl reads formats 1 and 2)";
        assert_eq!(error(&edited(|value| value["format"] = 0.into())), earlier);
    }

    #[test]
    fn invalid_fields_are_refused_with_their_value() {
        let cases: [(&str, serde_json::Value, &str); 5] = [
            ("id", "../b1".into(), "`../b1` is not a build id"),
            (
                "commit",
                "79e2e74".into(),
                "`79e2e74` is not a full commit hash",
            ),
            ("toolkit", "unknown".into(), "toolkit `unknown`"),
            ("built_at", "2026-10-09".into(), "built_at `2026-10-09`"),
            ("format", "1".into(), "malformed"),
        ];
        for (field, value, expected) in cases {
            let message = error(&edited(|metadata| metadata[field] = value));
            assert!(message.contains(expected), "{field}: {message}");
        }
    }

    #[test]
    fn unknown_and_missing_fields_are_refused() {
        let extra = edited(|value| value["extra"] = true.into());
        assert!(error(&extra).contains("unknown field `extra`"));
        let missing = edited(|value| {
            value.as_object_mut().unwrap().remove("commit");
        });
        assert!(error(&missing).contains("missing field `commit`"));
        assert!(error("").contains("malformed"));
    }
}

#[cfg(test)]
mod store_tests {
    use super::*;

    struct Fixture {
        _root: tempfile::TempDir,
        store: FsBuildStore,
        artifact: PathBuf,
        log: PathBuf,
    }

    fn fixture() -> Fixture {
        let root = tempfile::tempdir().unwrap();
        let work = root.path().join("work/bin");
        fs::create_dir_all(&work).unwrap();
        let artifact = work.join("llama-server");
        fs::write(&artifact, "binary").unwrap();
        let log = root.path().join("build-llama.cpp.log");
        fs::write(&log, "[1/1] done\n").unwrap();
        Fixture {
            store: FsBuildStore::new(root.path().join("builds")),
            _root: root,
            artifact,
            log,
        }
    }

    fn build(id: &str, at: i64) -> Build {
        Build {
            project: "llama.cpp".into(),
            id: BuildId::parse(id).unwrap(),
            commit: CommitId::parse("79e2e74eb11022c1ba2e438df7f0ca2d4c10f8b6").unwrap(),
            toolkit: Version::new(13, 4, 92),
            host_compiler: HostCompiler {
                command: "g++".into(),
                path: PathBuf::from("/usr/bin/g++"),
                version: Version::new(16, 2, 1),
            },
            cmake_flags: BTreeMap::new(),
            patches: vec![],
            built_at: Timestamp::from_unix(at).unwrap(),
        }
    }

    impl Fixture {
        fn install(&self, id: &str, at: i64) -> Build {
            let build = build(id, at);
            let staged = self
                .store
                .stage(&build, &self.artifact, &self.log, &[])
                .unwrap();
            self.store.publish(&staged, false).unwrap();
            build
        }

        fn project_entries(&self) -> Vec<String> {
            let mut names: Vec<String> = fs::read_dir(self.store.project_dir("llama.cpp"))
                .unwrap()
                .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
                .collect();
            names.sort();
            names
        }
    }

    fn id(text: &str) -> BuildId {
        BuildId::parse(text).unwrap()
    }

    #[test]
    fn the_store_contains_its_builds_binaries_removed_or_not() {
        let f = fixture();
        let binary = f.store.binary("llama.cpp", &id("b11538"), "llama-server");
        assert!(f.store.contains(&binary));
        let removed = format!("{} (deleted)", binary.display());
        assert!(f.store.contains(Path::new(&removed)));
        assert!(!f.store.contains(Path::new("/usr/bin/llama-server")));
        assert!(!f.store.contains(&f.artifact));
    }

    #[test]
    fn a_store_reached_through_a_link_contains_its_builds_by_their_real_path() {
        let root = tempfile::tempdir().unwrap();
        let real = fs::canonicalize(root.path()).unwrap().join("real");
        fs::create_dir_all(real.join("builds")).unwrap();
        let data = root.path().join("data");
        std::os::unix::fs::symlink(&real, &data).unwrap();
        let store = FsBuildStore::new(data.join("builds"));

        // The kernel names a running program by its real path.
        let binary = real.join("builds/llama.cpp/b11538/llama-server");
        assert!(store.contains(&binary));
        let removed = format!("{} (deleted)", binary.display());
        assert!(store.contains(Path::new(&removed)));
        assert!(!store.contains(&real.join("llama-server")));
    }

    #[test]
    fn a_build_keeps_the_patches_applied_to_its_sources() {
        let f = fixture();
        let patches = [
            Patch {
                name: "api-key.patch",
                diff: "diff --git a/server.cpp b/server.cpp\n",
            },
            Patch {
                name: "listen-failure.patch",
                diff: "diff --git a/main.cpp b/main.cpp\n",
            },
        ];
        let build = Build {
            patches: vec!["api-key.patch".into(), "listen-failure.patch".into()],
            ..build("b2", 20)
        };
        let staged = f
            .store
            .stage(&build, &f.artifact, &f.log, &patches)
            .unwrap();
        f.store.publish(&staged, false).unwrap();
        let dir = f.store.project_dir("llama.cpp").join("b2/patches");
        for patch in patches {
            assert_eq!(
                fs::read_to_string(dir.join(patch.name)).unwrap(),
                patch.diff
            );
        }
        assert_eq!(f.store.builds("llama.cpp").unwrap().0, [build]);
    }

    #[test]
    fn a_patch_that_cannot_be_kept_leaves_nothing_staged() {
        let f = fixture();
        let patches = [Patch {
            name: "no/such/directory.patch",
            diff: "diff --git a/server.cpp b/server.cpp\n",
        }];
        let message = format!(
            "{:#}",
            f.store
                .stage(&build("b2", 20), &f.artifact, &f.log, &patches)
                .err()
                .unwrap()
        );
        assert!(message.starts_with("cannot write "), "{message}");
        assert!(f.project_entries().is_empty(), "{:?}", f.project_entries());
    }

    #[test]
    fn a_staged_build_is_out_of_sight_until_published() {
        let f = fixture();
        let build = build("b2", 20);
        let staged = f.store.stage(&build, &f.artifact, &f.log, &[]).unwrap();
        assert!(
            staged
                .dir
                .file_name()
                .unwrap()
                .to_string_lossy()
                .starts_with(".staging-b2-")
        );
        assert_eq!(fs::read_to_string(&staged.binary).unwrap(), "binary");
        assert_eq!(f.store.builds("llama.cpp").unwrap(), (vec![], vec![]));

        f.store.publish(&staged, false).unwrap();
        let (builds, unreadable) = f.store.builds("llama.cpp").unwrap();
        assert_eq!(builds, [build]);
        assert!(unreadable.is_empty());
        let binary = f.store.binary("llama.cpp", &id("b2"), "llama-server");
        assert_eq!(fs::read_to_string(binary).unwrap(), "binary");
        let dir = f.store.project_dir("llama.cpp").join("b2");
        assert_eq!(fs::read_to_string(dir.join(LOG)).unwrap(), "[1/1] done\n");
        assert_eq!(f.project_entries(), ["b2"]);
    }

    #[test]
    fn the_binary_keeps_its_permissions() {
        use std::os::unix::fs::PermissionsExt;
        let f = fixture();
        fs::set_permissions(&f.artifact, fs::Permissions::from_mode(0o755)).unwrap();
        f.install("b1", 10);
        let binary = f.store.binary("llama.cpp", &id("b1"), "llama-server");
        assert_eq!(crate::infrastructure::files::mode(&binary), 0o755);
    }

    #[test]
    fn an_installed_build_is_replaced_only_on_request() {
        let f = fixture();
        f.install("b1", 10);
        let again = build("b1", 99);
        let staged = f.store.stage(&again, &f.artifact, &f.log, &[]).unwrap();
        let message = f.store.publish(&staged, false).unwrap_err().to_string();
        assert!(message.ends_with("b1 already exists"), "{message}");
        f.store.publish(&staged, true).unwrap();
        let (builds, _) = f.store.builds("llama.cpp").unwrap();
        assert_eq!(builds, [again]);
        assert_eq!(f.project_entries(), ["b1"]);
    }

    #[test]
    fn a_discarded_build_leaves_nothing() {
        let f = fixture();
        let staged = f
            .store
            .stage(&build("b1", 10), &f.artifact, &f.log, &[])
            .unwrap();
        f.store.discard(&staged).unwrap();
        assert!(f.project_entries().is_empty());
    }

    #[test]
    fn leftovers_of_an_interrupted_run_go_with_the_next_build() {
        let f = fixture();
        f.install("b1", 10);
        let dir = f.store.project_dir("llama.cpp");
        fs::create_dir_all(dir.join(".staging-b2-1/sub")).unwrap();
        fs::create_dir(dir.join(".removing-b0-1")).unwrap();
        std::os::unix::fs::symlink("b1", dir.join(".current-b1-1")).unwrap();
        assert_eq!(f.store.builds("llama.cpp").unwrap().0.len(), 1);
        f.install("b2", 20);
        assert_eq!(f.project_entries(), ["b1", "b2"]);
    }

    #[test]
    fn a_build_an_interrupted_replacement_left_hidden_is_put_back() {
        let f = fixture();
        let old = f.install("b1", 10);
        let dir = f.store.project_dir("llama.cpp");
        // Killed between `publish`'s two renames: b1 aside, its
        // replacement still staged.
        fs::rename(dir.join("b1"), dir.join(".replaced-b1-4242")).unwrap();
        fs::create_dir(dir.join(".staging-b1-4242")).unwrap();
        f.install("b2", 20);
        assert_eq!(f.project_entries(), ["b1", "b2"]);
        assert!(f.store.builds("llama.cpp").unwrap().0.contains(&old));

        // A hidden copy of a build in place is a leftover.
        fs::create_dir(dir.join(".replaced-b2-4242")).unwrap();
        f.install("b3", 30);
        assert_eq!(f.project_entries(), ["b1", "b2", "b3"]);
    }

    #[test]
    fn a_build_whose_replacement_was_installed_is_not_put_back() {
        let f = fixture();
        f.install("b1", 10);
        let dir = f.store.project_dir("llama.cpp");
        // Killed after `publish`'s two renames, while removing the old b1:
        // the new b1 in place, the old one aside and half removed.
        let staged = f
            .store
            .stage(&build("b1", 99), &f.artifact, &f.log, &[])
            .unwrap();
        fs::rename(dir.join("b1"), dir.join(".replaced-b1-4242")).unwrap();
        fs::rename(&staged.dir, dir.join("b1")).unwrap();
        fs::remove_file(dir.join(".replaced-b1-4242/llama-server")).unwrap();
        // The new b1 removed, as `build prune --keep 0` does.
        f.store.remove("llama.cpp", &id("b1")).unwrap();
        f.install("b2", 20);
        assert_eq!(f.project_entries(), ["b2"]);
    }

    #[test]
    fn a_replacement_that_cannot_be_installed_puts_the_old_build_back() {
        let f = fixture();
        let old = f.install("b1", 10);
        let staged = f
            .store
            .stage(&build("b1", 99), &f.artifact, &f.log, &[])
            .unwrap();
        // The staged build is gone: its rename into place fails.
        fs::remove_dir_all(&staged.dir).unwrap();
        let message = format!("{:#}", f.store.publish(&staged, true).unwrap_err());
        let target = f.store.project_dir("llama.cpp").join("b1");
        assert!(
            message.starts_with(&format!("cannot install {}", target.display())),
            "{message}"
        );
        assert_eq!(f.store.builds("llama.cpp").unwrap(), (vec![old], vec![]));
        assert_eq!(f.project_entries(), ["b1"]);
    }

    #[test]
    fn the_current_link_switches_between_installed_builds() {
        let f = fixture();
        assert_eq!(f.store.current("llama.cpp").unwrap(), None);
        f.install("b1", 10);
        f.install("b2", 20);
        f.store.set_current("llama.cpp", &id("b2")).unwrap();
        assert_eq!(f.store.current("llama.cpp").unwrap(), Some(id("b2")));
        f.store.set_current("llama.cpp", &id("b1")).unwrap();
        assert_eq!(f.store.current("llama.cpp").unwrap(), Some(id("b1")));
        let link = f.store.project_dir("llama.cpp").join(CURRENT);
        assert_eq!(fs::read_link(&link).unwrap(), Path::new("b1"));
        assert_eq!(
            fs::read_to_string(link.join("llama-server")).unwrap(),
            "binary"
        );
        assert!(f.store.set_current("llama.cpp", &id("b9")).is_err());
        assert_eq!(f.project_entries(), ["b1", "b2", "current"]);
        // The link names a build directory; it is not listed as one.
        let (builds, unreadable) = f.store.builds("llama.cpp").unwrap();
        let mut ids: Vec<String> = builds.iter().map(|build| build.id.to_string()).collect();
        ids.sort();
        assert_eq!(ids, ["b1", "b2"]);
        assert!(unreadable.is_empty(), "{unreadable:?}");
    }

    #[test]
    fn a_link_pointing_elsewhere_is_reported() {
        let f = fixture();
        f.install("b1", 10);
        let link = f.store.project_dir("llama.cpp").join(CURRENT);
        std::os::unix::fs::symlink("/usr/bin", &link).unwrap();
        let message = f.store.current("llama.cpp").unwrap_err().to_string();
        assert!(
            message.contains("not at a build of its directory"),
            "{message}"
        );
    }

    #[test]
    fn removing_a_build_spares_the_current_one() {
        let f = fixture();
        f.install("b1", 10);
        f.install("b2", 20);
        f.store.set_current("llama.cpp", &id("b2")).unwrap();
        f.store.remove("llama.cpp", &id("b1")).unwrap();
        let message = f
            .store
            .remove("llama.cpp", &id("b2"))
            .unwrap_err()
            .to_string();
        assert!(message.contains("is the current build"), "{message}");
        assert_eq!(f.project_entries(), ["b2", "current"]);
    }

    #[test]
    fn foreign_directories_are_unreadable_builds() {
        let f = fixture();
        f.install("b1", 10);
        let dir = f.store.project_dir("llama.cpp");
        fs::create_dir(dir.join("notes")).unwrap();
        fs::create_dir(dir.join("b7")).unwrap();
        fs::write(
            dir.join("b7").join(METADATA),
            to_json(&build("b8", 1)).unwrap(),
        )
        .unwrap();
        fs::write(dir.join("README"), "a file").unwrap();
        let (builds, mut unreadable) = f.store.builds("llama.cpp").unwrap();
        assert_eq!(builds.len(), 1);
        unreadable.sort_by(|a, b| a.dir.cmp(&b.dir));
        assert_eq!(unreadable.len(), 2);
        assert!(
            unreadable[0].reason.contains("describes llama.cpp b8"),
            "{:?}",
            unreadable[0]
        );
        assert!(
            unreadable[1].reason.contains("cannot read"),
            "{:?}",
            unreadable[1]
        );
    }

    #[test]
    fn a_second_lock_is_refused_until_the_first_is_dropped() {
        let f = fixture();
        let _exclusive = crate::infrastructure::command::exclusive();
        let first = f.store.lock().unwrap();
        let message = f.store.lock().unwrap_err().to_string();
        assert!(message.contains("another ai-ctl"), "{message}");
        drop(first);
        f.store.lock().unwrap();
    }

    #[test]
    fn now_is_after_this_code_was_written() {
        assert!(now() > Timestamp::parse("2026-10-09T00:00:00Z").unwrap());
    }
}
