//! ai-ctl's API key: one key, created by the first `up` and kept in a file of
//! the state directory that only the user can read. `up` passes it to
//! llama-swap in its environment, and ai-ctl's clients send it as
//! `Authorization: Bearer`.

use std::fmt::Write as _;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

use anyhow::{Context, Result, anyhow};

use super::files;

/// The random bytes a key is made of, written in hexadecimal.
const KEY_BYTES: usize = 32;

/// The file that keeps the key.
#[derive(Debug, Clone)]
pub struct KeyFile {
    path: PathBuf,
}

impl KeyFile {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }

    /// The key; `None` while no `up` has created it.
    pub fn read(&self) -> Result<Option<String>> {
        let Some(text) = files::read_if_present(&self.path)? else {
            return Ok(None);
        };
        let key = text.trim();
        // A header carries it, and llama-server splits a list of keys at
        // commas.
        if key.is_empty() || !key.chars().all(|c| c.is_ascii_graphic() && c != ',') {
            return Err(anyhow!(
                "{} holds no API key ai-ctl can use; remove it, and the next `ai-ctl up` \
                 that starts the gateway creates a new one",
                self.path.display()
            ));
        }
        Ok(Some(key.to_owned()))
    }

    /// The key, created first when the file is missing: random bytes in
    /// hexadecimal, in a new file only the user can read. A key already
    /// there, even one written meanwhile by another ai-ctl, is kept.
    pub fn read_or_create(&self) -> Result<String> {
        if let Some(key) = self.read()? {
            return Ok(key);
        }
        let key = random_key()?;
        if let Some(dir) = self.path.parent() {
            files::create_private_dir(dir)?;
        }
        if files::write_new(&self.path, &format!("{key}\n"), 0o600)? {
            return Ok(key);
        }
        // Another ai-ctl wrote a key meanwhile, or something that is none,
        // such as a link to nothing, holds the place.
        self.read()?.ok_or_else(|| {
            anyhow!(
                "{} is in the way of a new API key but holds none; remove it, and the next \
                 `ai-ctl up` that starts the gateway creates a new one",
                self.path.display()
            )
        })
    }
}

#[cfg(test)]
impl KeyFile {
    /// The file `api-key` of `dir`, holding `key`.
    pub fn written(dir: &std::path::Path, key: &str) -> Self {
        let path = dir.join("api-key");
        std::fs::write(&path, format!("{key}\n")).unwrap();
        Self::new(path)
    }
}

/// [`KEY_BYTES`] bytes from the kernel's random source, in hexadecimal.
fn random_key() -> Result<String> {
    let mut bytes = [0; KEY_BYTES];
    File::open("/dev/urandom")
        .and_then(|mut random| random.read_exact(&mut bytes))
        .context("cannot read /dev/urandom")?;
    Ok(bytes.iter().fold(String::new(), |mut hex, byte| {
        let _ = write!(hex, "{byte:02x}");
        hex
    }))
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;

    fn key_file(dir: &tempfile::TempDir) -> KeyFile {
        KeyFile::new(dir.path().join("state").join("api-key"))
    }

    #[test]
    fn no_key_reads_as_none() {
        let dir = tempfile::tempdir().unwrap();
        assert_eq!(key_file(&dir).read().unwrap(), None);
    }

    #[test]
    fn a_key_is_created_once_readable_by_the_user_only() {
        let dir = tempfile::tempdir().unwrap();
        let keys = key_file(&dir);
        let key = keys.read_or_create().unwrap();
        assert_eq!(key.len(), 64, "{key}");
        assert!(key.chars().all(|c| c.is_ascii_hexdigit()), "{key}");

        let path = dir.path().join("state").join("api-key");
        assert_eq!(fs::read_to_string(&path).unwrap(), format!("{key}\n"));
        assert_eq!(files::mode(&path), 0o600);
        assert_eq!(keys.read_or_create().unwrap(), key);
        assert_eq!(keys.read().unwrap(), Some(key));
        let names: Vec<_> = fs::read_dir(dir.path().join("state"))
            .unwrap()
            .map(|entry| entry.unwrap().file_name())
            .collect();
        assert_eq!(names, ["api-key"]);
    }

    #[test]
    fn the_keys_directory_is_created_for_the_user_only() {
        let dir = tempfile::tempdir().unwrap();
        key_file(&dir).read_or_create().unwrap();
        assert_eq!(files::mode(&dir.path().join("state")), 0o700);
    }

    #[test]
    fn each_key_is_drawn_anew() {
        let (one, two) = (tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap());
        assert_ne!(
            key_file(&one).read_or_create().unwrap(),
            key_file(&two).read_or_create().unwrap()
        );
    }

    #[test]
    fn a_key_written_by_hand_is_kept_and_read_without_its_newline() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("state").join("api-key");
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, "sk-mine\n").unwrap();
        let keys = key_file(&dir);
        assert_eq!(keys.read_or_create().unwrap(), "sk-mine");
        assert_eq!(keys.read().unwrap().as_deref(), Some("sk-mine"));
        assert_eq!(fs::read_to_string(&path).unwrap(), "sk-mine\n");
    }

    #[test]
    fn a_link_to_nothing_in_the_keys_place_is_an_error_naming_it() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("state").join("api-key");
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::os::unix::fs::symlink(dir.path().join("gone"), &path).unwrap();
        assert_eq!(
            format!("{:#}", key_file(&dir).read_or_create().unwrap_err()),
            format!(
                "{} is in the way of a new API key but holds none; remove it, and the next \
                 `ai-ctl up` that starts the gateway creates a new one",
                path.display()
            )
        );
    }

    #[test]
    fn a_file_without_a_usable_key_is_an_error_naming_it() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("state").join("api-key");
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        // Empty, or what a header or llama-server's comma-separated list
        // would split.
        for text in ["", "\n", "two words", "one,two", "é"] {
            fs::write(&path, text).unwrap();
            for result in [
                key_file(&dir).read().map(|_| ()),
                key_file(&dir).read_or_create().map(|_| ()),
            ] {
                assert_eq!(
                    format!("{:#}", result.unwrap_err()),
                    format!(
                        "{} holds no API key ai-ctl can use; remove it, and the next \
                         `ai-ctl up` that starts the gateway creates a new one",
                        path.display()
                    ),
                    "{text:?}"
                );
            }
        }
    }
}
