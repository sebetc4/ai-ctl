//! Containers and their images, through podman driven as commands.

use std::path::PathBuf;
use std::process::Command;

use anyhow::{Result, anyhow, bail};

use crate::application::ports::Containers;
use crate::domain::machine::Tool;

use super::command::{self, Output, RunError};

/// podman, run by the user: images go to their rootless storage.
pub struct Podman {
    /// The podman program: `podman`, found on `PATH`.
    program: PathBuf,
}

impl Podman {
    pub fn new(program: PathBuf) -> Self {
        Self { program }
    }
}

impl Containers for Podman {
    fn image_exists(&self, image: &str) -> Result<bool> {
        let output = checked(command::run(&self.program, &["image", "exists", image]))?;
        // A missing image exits with an error and prints nothing.
        match (output.success, output.stderr.trim().is_empty()) {
            (true, _) => Ok(true),
            (false, true) => Ok(false),
            (false, false) => bail!("podman image exists {image}: {}", output.failure()),
        }
    }

    fn pull(&self, image: &str) -> Result<()> {
        let mut pull = Command::new(&self.program);
        pull.args(["pull", "--quiet", image]);
        // An image of several GB takes as long as the network does.
        let output = checked(command::execute(pull, None))?;
        if !output.success {
            bail!("podman pull {image}: {}", output.failure());
        }
        Ok(())
    }

    fn pids(&self, prefix: &str) -> Result<Vec<u32>> {
        let filter = format!("name=^{prefix}");
        let output = match command::run(
            &self.program,
            &["ps", "--filter", &filter, "--format", "{{.Pid}}"],
        ) {
            // Without podman, no container runs.
            Err(RunError::NotFound) => return Ok(Vec::new()),
            result => checked(result)?,
        };
        if !output.success {
            bail!("podman ps: {}", output.failure());
        }
        parse_pids(&output.stdout)
    }

    fn digest(&self, image: &str) -> Result<String> {
        let output = checked(command::run(
            &self.program,
            &["image", "inspect", "--format", "{{.Digest}}", image],
        ))?;
        if !output.success {
            bail!("podman image inspect {image}: {}", output.failure());
        }
        parse_digest(&output.stdout)
    }
}

/// The digest `podman image inspect --format {{.Digest}}` prints.
fn parse_digest(text: &str) -> Result<String> {
    let digest = text.trim();
    let hex = digest
        .strip_prefix("sha256:")
        .filter(|hex| hex.len() == 64 && hex.bytes().all(|b| b.is_ascii_hexdigit()))
        .ok_or_else(|| anyhow!("podman image inspect printed {digest:?} for a digest"))?;
    Ok(format!("sha256:{hex}"))
}

/// One PID per line, as `podman ps --format {{.Pid}}` prints them.
fn parse_pids(text: &str) -> Result<Vec<u32>> {
    text.lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(|line| {
            line.parse()
                .map_err(|_| anyhow!("podman ps printed {line:?} for a PID"))
        })
        .collect()
}

/// The output of a podman command that ran, or why it did not.
fn checked(result: Result<Output, RunError>) -> Result<Output> {
    command::checked(result, &command::not_found(Tool::Podman))
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::*;

    /// A podman method's error without podman.
    const MISSING: &str = "podman not found; run `sudo dnf install podman`";

    /// What podman 5.8.7 printed on this machine.
    const DIGEST: &str = include_str!("../../tests/fixtures/podman-5.8.7-image-inspect-digest.txt");
    const PID: &str = include_str!("../../tests/fixtures/podman-5.8.7-ps-pid.txt");
    const BROKEN_STORAGE: &str =
        include_str!("../../tests/fixtures/podman-5.8.7-ps-bogus-storage-driver.txt");
    const IMAGE: &str = "ghcr.io/ggml-org/llama.cpp:server-cuda13-b11541";

    /// Podman as a script, in `dir`, that runs `body` when given `args`
    /// and exits 2 on any other arguments.
    fn podman(dir: &Path, args: &str, body: &str) -> Podman {
        let program = dir.join("podman");
        command::script(
            &program,
            &format!("[ \"$*\" = '{args}' ] || exit 2\n{body}"),
        );
        Podman::new(program)
    }

    /// A script body that prints `text` on `fd` and exits with `status`.
    fn prints(fd: u8, text: &str, status: u8) -> String {
        format!("cat >&{fd} <<'EOF'\n{text}EOF\nexit {status}")
    }

    /// The line `failure()` takes from the broken storage's error.
    fn storage_error() -> &'static str {
        BROKEN_STORAGE
            .lines()
            .find(|line| line.starts_with("Error: "))
            .unwrap()
    }

    fn absent(dir: &Path) -> Podman {
        Podman::new(dir.join("absent"))
    }

    #[test]
    fn an_image_that_podman_does_not_find_is_missing_and_an_error_is_an_error() {
        let dir = tempfile::tempdir().unwrap();
        let args = format!("image exists {IMAGE}");
        assert!(
            podman(dir.path(), &args, "exit 0")
                .image_exists(IMAGE)
                .unwrap()
        );
        // As podman 5.8.7 answers for an image it does not hold.
        assert!(
            !podman(dir.path(), &args, "exit 1")
                .image_exists(IMAGE)
                .unwrap()
        );
        let broken = podman(dir.path(), &args, &prints(2, BROKEN_STORAGE, 125));
        assert_eq!(
            broken.image_exists(IMAGE).unwrap_err().to_string(),
            format!("podman image exists {IMAGE}: {}", storage_error())
        );
        assert_eq!(
            absent(dir.path())
                .image_exists(IMAGE)
                .unwrap_err()
                .to_string(),
            MISSING
        );
    }

    #[test]
    fn a_pull_is_quiet_and_its_failure_names_the_image() {
        let dir = tempfile::tempdir().unwrap();
        let args = format!("pull --quiet {IMAGE}");
        podman(dir.path(), &args, &prints(1, DIGEST, 0))
            .pull(IMAGE)
            .unwrap();
        let broken = podman(dir.path(), &args, &prints(2, BROKEN_STORAGE, 125));
        assert_eq!(
            broken.pull(IMAGE).unwrap_err().to_string(),
            format!("podman pull {IMAGE}: {}", storage_error())
        );
        assert_eq!(
            absent(dir.path()).pull(IMAGE).unwrap_err().to_string(),
            MISSING
        );
    }

    #[test]
    fn the_containers_pids_come_from_podman_ps_and_none_run_without_podman() {
        let dir = tempfile::tempdir().unwrap();
        let args = "ps --filter name=^ai-ctl- --format {{.Pid}}";
        let listing = podman(dir.path(), args, &prints(1, PID, 0));
        assert_eq!(listing.pids("ai-ctl-").unwrap(), [895_050]);
        let silent = podman(dir.path(), args, "exit 1");
        assert_eq!(
            silent.pids("ai-ctl-").unwrap_err().to_string(),
            "podman ps: exited with an error and printed nothing"
        );
        let broken = podman(dir.path(), args, &prints(2, BROKEN_STORAGE, 125));
        assert_eq!(
            broken.pids("ai-ctl-").unwrap_err().to_string(),
            format!("podman ps: {}", storage_error())
        );
        assert_eq!(
            absent(dir.path()).pids("ai-ctl-").unwrap(),
            Vec::<u32>::new()
        );
    }

    #[test]
    fn a_digest_comes_from_image_inspect() {
        let dir = tempfile::tempdir().unwrap();
        let args = format!("image inspect --format {{{{.Digest}}}} {IMAGE}");
        let inspect = podman(dir.path(), &args, &prints(1, DIGEST, 0));
        assert_eq!(inspect.digest(IMAGE).unwrap(), DIGEST.trim_end());
        let silent = podman(dir.path(), &args, "exit 1");
        assert_eq!(
            silent.digest(IMAGE).unwrap_err().to_string(),
            format!("podman image inspect {IMAGE}: exited with an error and printed nothing")
        );
        assert_eq!(
            absent(dir.path()).digest(IMAGE).unwrap_err().to_string(),
            MISSING
        );
    }

    #[test]
    fn a_digest_is_a_sha256() {
        assert_eq!(
            parse_digest(DIGEST).unwrap(),
            "sha256:42e80867b03cc22e8cf744c07e0b3194f078838ab6c476095ecdc1621271ec95"
        );
        assert_eq!(
            parse_digest("<no value>\n").unwrap_err().to_string(),
            "podman image inspect printed \"<no value>\" for a digest"
        );
        assert!(parse_digest("sha256:abc").is_err());
    }

    #[test]
    fn pids_are_read_one_per_line() {
        assert_eq!(parse_pids(PID).unwrap(), [895_050]);
        // Two containers, a line each.
        assert_eq!(
            parse_pids(&format!("{PID}{PID}")).unwrap(),
            [895_050, 895_050]
        );
        // What `podman ps` prints when no container matches.
        assert_eq!(parse_pids("").unwrap(), Vec::<u32>::new());
        assert_eq!(
            parse_pids("<no value>\n").unwrap_err().to_string(),
            "podman ps printed \"<no value>\" for a PID"
        );
    }
}
