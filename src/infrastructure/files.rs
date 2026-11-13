//! File system and file format helpers the adapters share.

use std::any::Any;
use std::fs::{self, File, OpenOptions, TryLockError};
use std::io::{self, Write};
use std::os::unix::fs::{DirBuilderExt, OpenOptionsExt};
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, anyhow};
use serde::Deserialize;

/// The `format` of a JSON file ai-ctl writes, read alone: a later format
/// that adds or renames fields is told by its number, not refused as
/// malformed by the strict parse of the formats this ai-ctl reads.
pub fn json_format(text: &str) -> Result<u32> {
    #[derive(Deserialize)]
    struct Format {
        format: u32,
    }
    Ok(serde_json::from_str::<Format>(text)?.format)
}

/// Removes `dir` and its content; a missing `dir` is not an error.
pub fn remove_dir_if_present(dir: &Path) -> Result<()> {
    match fs::remove_dir_all(dir) {
        Err(err) if err.kind() != io::ErrorKind::NotFound => {
            Err(err).with_context(|| format!("cannot remove {}", dir.display()))
        }
        _ => Ok(()),
    }
}

/// The content of `path`; `None` when there is no file.
pub fn read_if_present(path: &Path) -> Result<Option<String>> {
    match fs::read_to_string(path) {
        Ok(text) => Ok(Some(text)),
        Err(err) if err.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(err) => Err(err).with_context(|| format!("cannot read {}", path.display())),
    }
}

/// The target of the symbolic link `link`; `None` when there is no link.
pub fn read_link_if_present(link: &Path) -> Result<Option<PathBuf>> {
    match fs::read_link(link) {
        Ok(target) => Ok(Some(target)),
        Err(err) if err.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(err) => Err(err).with_context(|| format!("cannot read {}", link.display())),
    }
}

/// Creates `dir` and its missing parents for the user only, mode 0700; a
/// directory already there keeps its mode.
pub fn create_private_dir(dir: &Path) -> Result<()> {
    fs::DirBuilder::new()
        .recursive(true)
        .mode(0o700)
        .create(dir)
        .with_context(|| format!("cannot create {}", dir.display()))
}

/// Takes the lock file `.lock` of `dir`, creating both if needed, and writes
/// in it what the holder is `doing`; the lock is released when the returned
/// guard is dropped. Fails at once when another process holds it, saying
/// what that one wrote it is doing.
pub fn lock(dir: &Path, doing: &str) -> Result<Box<dyn Any>> {
    fs::create_dir_all(dir).with_context(|| format!("cannot create {}", dir.display()))?;
    let path = dir.join(".lock");
    let mut file = File::options()
        .create(true)
        .truncate(false)
        .write(true)
        .open(&path)
        .with_context(|| format!("cannot open {}", path.display()))?;
    match file.try_lock() {
        Ok(()) => {
            // Truncated under the lock: until then the file holds what the
            // last holder did, which only a refused process reads.
            file.set_len(0)
                .and_then(|()| file.write_all(doing.as_bytes()))
                .with_context(|| format!("cannot write {}", path.display()))?;
            Ok(Box::new(file))
        }
        Err(TryLockError::WouldBlock) => {
            // A holder that wrote nothing yet, or an older ai-ctl, leaves it
            // empty.
            let held = fs::read_to_string(&path).unwrap_or_default();
            Err(match held.trim() {
                "" => anyhow!("another ai-ctl holds {}", path.display()),
                held => anyhow!("another ai-ctl is {held} ({} is locked)", path.display()),
            })
        }
        Err(TryLockError::Error(err)) => {
            Err(err).with_context(|| format!("cannot lock {}", path.display()))
        }
    }
}

/// Writes `text` to `path`, a new file with permissions `mode` (less the
/// umask), unless something is already there; `false` then. The content
/// goes to a temporary file first, then a hard link publishes it: the link
/// fails if `path` exists, so a concurrent writer or a crash never leaves a
/// partial or overwritten file.
pub fn write_new(path: &Path, text: &str, mode: u32) -> Result<bool> {
    if path.symlink_metadata().is_ok() {
        return Ok(false);
    }
    let dir = path
        .parent()
        .with_context(|| format!("{} has no parent directory", path.display()))?;
    fs::create_dir_all(dir).with_context(|| format!("cannot create {}", dir.display()))?;

    sweep_temporaries(dir);
    let temporary = temporary_for(path)?;
    let written = write_synced(&temporary, text, mode);
    let published = written.and_then(|()| fs::hard_link(&temporary, path));
    // The temporary file is never needed afterwards; failing to remove it
    // leaves a stray hidden file, not a wrong one.
    let _ = fs::remove_file(&temporary);
    match published {
        Ok(()) => Ok(true),
        Err(err) if err.kind() == io::ErrorKind::AlreadyExists => Ok(false),
        Err(err) => Err(err).with_context(|| format!("cannot write {}", path.display())),
    }
}

/// Writes `text` to `path` in place of what is there, by the rename of a
/// temporary file beside it, so that `path` holds the old text or the new;
/// the temporary file is removed when the write or the rename fails.
pub fn replace_file(path: &Path, text: &str) -> Result<()> {
    if let Some(dir) = path.parent() {
        sweep_temporaries(dir);
    }
    let temporary = temporary_for(path)?;
    let replaced = fs::write(&temporary, text).and_then(|()| fs::rename(&temporary, path));
    if replaced.is_err() {
        let _ = fs::remove_file(&temporary);
    }
    replaced.with_context(|| format!("cannot write {}", path.display()))
}

/// The hidden file beside `path` that [`write_new`] and [`replace_file`]
/// write first, named after this process.
pub fn temporary_for(path: &Path) -> Result<PathBuf> {
    let dir = path
        .parent()
        .with_context(|| format!("{} has no parent directory", path.display()))?;
    let name = path
        .file_name()
        .with_context(|| format!("{} has no file name", path.display()))?;
    Ok(dir.join(format!(
        ".{}.{}.tmp",
        name.to_string_lossy(),
        std::process::id()
    )))
}

/// Removes the temporary files of `dir` that a writer killed before it
/// removed them left behind: `.<name>.<pid>.tmp`, as [`temporary_for`]
/// names them, whose process no longer runs. One that cannot be removed
/// stays: it is clutter, never a wrong file.
fn sweep_temporaries(dir: &Path) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let name = entry.file_name();
        let Some(pid) = name.to_str().and_then(temporary_pid) else {
            continue;
        };
        if !Path::new("/proc").join(pid.to_string()).exists() {
            let _ = fs::remove_file(entry.path());
        }
    }
}

/// The PID in the name of a temporary file, `.<name>.<pid>.tmp`.
fn temporary_pid(name: &str) -> Option<u32> {
    let (file, pid) = name
        .strip_prefix('.')?
        .strip_suffix(".tmp")?
        .rsplit_once('.')?;
    (!file.is_empty()).then(|| pid.parse().ok())?
}

fn write_synced(path: &Path, text: &str, mode: u32) -> io::Result<()> {
    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .mode(mode)
        .open(path)?;
    file.write_all(text.as_bytes())?;
    file.sync_all()
}

/// Points the symbolic link `link` at `target` by renaming `temporary`, a
/// link made beside it, over it: the link is never missing.
pub fn replace_link(link: &Path, target: &str, temporary: &Path) -> Result<()> {
    let _ = fs::remove_file(temporary);
    std::os::unix::fs::symlink(target, temporary)
        .with_context(|| format!("cannot create {}", temporary.display()))?;
    if let Err(err) = fs::rename(temporary, link) {
        let _ = fs::remove_file(temporary);
        return Err(err).with_context(|| format!("cannot update {}", link.display()));
    }
    Ok(())
}

/// Removes the hidden entries of `dir` — files, links and directories whose
/// name starts with `.` — but not the lock file. A missing `dir` has none.
pub fn remove_hidden(dir: &Path) -> Result<()> {
    let entries = match fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(err) if err.kind() == io::ErrorKind::NotFound => return Ok(()),
        Err(err) => return Err(err).with_context(|| format!("cannot read {}", dir.display())),
    };
    for entry in entries {
        let entry = entry.with_context(|| format!("cannot read {}", dir.display()))?;
        let name = entry.file_name();
        if !name.to_string_lossy().starts_with('.') || name == ".lock" {
            continue;
        }
        let path = entry.path();
        if path.is_symlink() || path.is_file() {
            fs::remove_file(&path).with_context(|| format!("cannot remove {}", path.display()))?;
        } else {
            remove_dir_if_present(&path)?;
        }
    }
    Ok(())
}

#[cfg(test)]
pub fn mode(path: &Path) -> u32 {
    use std::os::unix::fs::PermissionsExt;
    fs::metadata(path).unwrap().permissions().mode() & 0o777
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_file_is_read_and_a_missing_one_is_none() {
        let root = tempfile::tempdir().unwrap();
        let file = root.path().join("gateway.json");
        assert_eq!(read_if_present(&file).unwrap(), None);
        fs::write(&file, "{}").unwrap();
        assert_eq!(read_if_present(&file).unwrap().as_deref(), Some("{}"));
        let error = format!("{:#}", read_if_present(root.path()).unwrap_err());
        assert!(
            error.starts_with(&format!("cannot read {}: ", root.path().display())),
            "{error}"
        );
    }

    #[test]
    fn a_link_is_read_and_a_missing_one_is_none() {
        let root = tempfile::tempdir().unwrap();
        let link = root.path().join("current");
        assert_eq!(read_link_if_present(&link).unwrap(), None);
        std::os::unix::fs::symlink("b11538", &link).unwrap();
        assert_eq!(
            read_link_if_present(&link).unwrap(),
            Some(PathBuf::from("b11538"))
        );
        let file = root.path().join("file");
        fs::write(&file, "").unwrap();
        let error = format!("{:#}", read_link_if_present(&file).unwrap_err());
        assert!(
            error.starts_with(&format!("cannot read {}: ", file.display())),
            "{error}"
        );
    }

    #[test]
    fn a_private_directory_and_its_missing_parents_are_the_users_only() {
        let root = tempfile::tempdir().unwrap();
        let shared = root.path().join("shared");
        fs::DirBuilder::new().mode(0o755).create(&shared).unwrap();
        create_private_dir(&shared.join("state/logs")).unwrap();
        assert_eq!(mode(&shared.join("state")), 0o700);
        assert_eq!(mode(&shared.join("state/logs")), 0o700);
        assert_eq!(mode(&shared), 0o755);
        create_private_dir(&shared.join("state/logs")).unwrap();
    }

    #[test]
    fn a_missing_directory_is_already_removed() {
        let root = tempfile::tempdir().unwrap();
        remove_dir_if_present(&root.path().join("absent")).unwrap();
        let dir = root.path().join("tree/leaf");
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("file"), "x").unwrap();
        remove_dir_if_present(&root.path().join("tree")).unwrap();
        assert!(!root.path().join("tree").exists());
    }

    #[test]
    fn a_second_lock_is_refused_until_the_first_is_dropped() {
        let root = tempfile::tempdir().unwrap();
        let dir = root.path().join("store");
        let _exclusive = crate::infrastructure::command::exclusive();
        let first = lock(&dir, "changing the store").unwrap();
        let message = lock(&dir, "changing the store").unwrap_err().to_string();
        assert!(
            message.starts_with("another ai-ctl is changing the store ("),
            "{message}"
        );
        drop(first);
        lock(&dir, "changing the store").unwrap();
    }

    #[test]
    fn a_refused_lock_says_what_its_holder_does() {
        let root = tempfile::tempdir().unwrap();
        let dir = root.path().join("gateway");
        let path = dir.join(".lock");
        let _exclusive = crate::infrastructure::command::exclusive();
        drop(lock(&dir, "removing the record of a gateway that stopped").unwrap());
        let held = lock(&dir, "removing builds").unwrap();
        assert_eq!(
            lock(&dir, "starting the gateway").unwrap_err().to_string(),
            format!(
                "another ai-ctl is removing builds ({} is locked)",
                path.display()
            )
        );
        drop(held);
        // A holder that wrote nothing, such as an older ai-ctl, is told by
        // its lock alone.
        let older = File::options().write(true).open(&path).unwrap();
        older.set_len(0).unwrap();
        older.try_lock().unwrap();
        assert_eq!(
            lock(&dir, "starting the gateway").unwrap_err().to_string(),
            format!("another ai-ctl holds {}", path.display())
        );
    }

    #[test]
    fn a_file_is_replaced_whole_and_a_failure_leaves_no_temporary() {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("llama-swap.yaml");
        fs::write(&path, "old").unwrap();
        replace_file(&path, "new").unwrap();
        assert_eq!(fs::read_to_string(&path).unwrap(), "new");

        let blocked = root.path().join("gateway.json");
        fs::create_dir(&blocked).unwrap();
        let error = replace_file(&blocked, "{}").unwrap_err();
        assert_eq!(
            format!("{error:#}"),
            format!(
                "cannot write {}: Is a directory (os error 21)",
                blocked.display()
            )
        );
        let mut names: Vec<String> = fs::read_dir(root.path())
            .unwrap()
            .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
            .collect();
        names.sort();
        assert_eq!(names, ["gateway.json", "llama-swap.yaml"]);
    }

    #[test]
    fn a_write_sweeps_the_temporaries_of_writers_that_no_longer_run() {
        let root = tempfile::tempdir().unwrap();
        // No process has the largest PID: Linux's PIDs end at 2^22.
        let gone = u32::MAX;
        let ours = std::process::id();
        let left = [
            format!(".inventory.toml.{gone}.tmp"),
            format!(".20261011T120000Z-m.json.{gone}.tmp"),
            format!(".inventory.toml.{ours}.tmp"),
            ".notes.tmp".to_owned(),
            format!("kept.{gone}.tmp"),
        ];
        let names = |dir: &Path| {
            let mut names: Vec<String> = fs::read_dir(dir)
                .unwrap()
                .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
                .collect();
            names.sort();
            names
        };
        let kept = |file: &str| {
            let mut kept = vec![
                format!(".inventory.toml.{ours}.tmp"),
                ".notes.tmp".to_owned(),
                file.to_owned(),
                format!("kept.{gone}.tmp"),
            ];
            kept.sort();
            kept
        };
        for (i, write) in [
            |path: &Path| write_new(path, "x", 0o600).map(|_| ()),
            |path: &Path| replace_file(path, "x"),
        ]
        .iter()
        .enumerate()
        {
            let dir = root.path().join(i.to_string());
            fs::create_dir(&dir).unwrap();
            for name in &left {
                fs::write(dir.join(name), "").unwrap();
            }
            write(&dir.join("api-key")).unwrap();
            assert_eq!(names(&dir), kept("api-key"));
        }
    }

    #[test]
    fn a_link_is_replaced_in_place() {
        let root = tempfile::tempdir().unwrap();
        let link = root.path().join("current");
        let temporary = root.path().join(".current-1");
        replace_link(&link, "a", &temporary).unwrap();
        replace_link(&link, "b", &temporary).unwrap();
        assert_eq!(fs::read_link(&link).unwrap(), Path::new("b"));
        assert!(temporary.symlink_metadata().is_err());
    }

    #[test]
    fn hidden_entries_go_but_the_lock_and_the_rest_stay() {
        let root = tempfile::tempdir().unwrap();
        let dir = root.path();
        fs::create_dir_all(dir.join(".staging-1/sub")).unwrap();
        fs::write(dir.join(".partial"), "x").unwrap();
        std::os::unix::fs::symlink("v1", dir.join(".current-1")).unwrap();
        fs::write(dir.join(".lock"), "").unwrap();
        fs::create_dir(dir.join("v1")).unwrap();
        remove_hidden(dir).unwrap();
        let mut names: Vec<String> = fs::read_dir(dir)
            .unwrap()
            .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
            .collect();
        names.sort();
        assert_eq!(names, [".lock", "v1"]);
        remove_hidden(&dir.join("absent")).unwrap();
    }
}
