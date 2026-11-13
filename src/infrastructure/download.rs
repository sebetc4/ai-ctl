//! Downloads over HTTPS with `curl`, which every Fedora installs.

use std::path::Path;
use std::process::Command;

use anyhow::{Context, Result, ensure};

use crate::application::ports::Downloads;

use super::command::{checked, execute};

/// Fetches files with `curl`: HTTPS only, redirects included, retried on
/// transient errors, and abandoned when the transfer stalls.
pub struct Curl {
    /// The protocols `curl` may use, for the URL and its redirects.
    protocols: &'static str,
}

impl Curl {
    pub fn new() -> Self {
        Self {
            protocols: "=https",
        }
    }
}

impl Default for Curl {
    fn default() -> Self {
        Self::new()
    }
}

impl Downloads for Curl {
    fn download(&self, url: &str, to: &Path) -> Result<()> {
        let mut command = Command::new("curl");
        command
            .args(["--fail", "--silent", "--show-error", "--location"])
            .args(["--proto", self.protocols, "--proto-redir", self.protocols])
            .args(["--retry", "3", "--connect-timeout", "30"])
            // Slower than 1 KiB/s for a minute: the transfer is stalled.
            .args(["--speed-limit", "1024", "--speed-time", "60"])
            .arg("--output")
            .arg(to)
            .arg("--")
            .arg(url);
        let output = checked(execute(command, None), "curl is not installed")
            .with_context(|| format!("cannot download {url}"))?;
        ensure!(
            output.success,
            "cannot download {url}: {}",
            output.failure()
        );
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;

    /// `curl` allowed to read local files, so no test needs the network.
    fn local() -> Curl {
        Curl {
            protocols: "=https,file",
        }
    }

    #[test]
    fn a_file_is_saved_where_asked() {
        let root = tempfile::tempdir().unwrap();
        let source = root.path().join("archive.tar.gz");
        fs::write(&source, "archive").unwrap();
        let target = root.path().join("staging/archive.tar.gz");
        fs::create_dir(root.path().join("staging")).unwrap();
        local()
            .download(&format!("file://{}", source.display()), &target)
            .unwrap();
        assert_eq!(fs::read_to_string(target).unwrap(), "archive");
    }

    #[test]
    fn a_missing_file_is_an_error_naming_the_url() {
        let root = tempfile::tempdir().unwrap();
        let url = format!("file://{}/absent.txt", root.path().display());
        let target = root.path().join("file");
        let message = local().download(&url, &target).unwrap_err().to_string();
        assert!(
            message.starts_with(&format!("cannot download {url}: curl: (37)")),
            "{message}"
        );
    }

    #[test]
    fn only_https_is_allowed() {
        let root = tempfile::tempdir().unwrap();
        let target = root.path().join("file");
        for url in ["http://127.0.0.1:9/file", "file:///etc/hostname"] {
            let message = Curl::new().download(url, &target).unwrap_err().to_string();
            assert!(
                message.starts_with(&format!("cannot download {url}: curl: (1)")),
                "{message}"
            );
        }
        assert!(!target.exists());
    }
}
