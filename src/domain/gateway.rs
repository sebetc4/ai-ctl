//! The gateway, llama-swap: the releases ai-ctl installs.
//!
//! ai-ctl ships with one pinned release, whose archive's SHA-256 is written
//! here: llama-swap is updated only through this pin, and its digest is the
//! only one an archive is checked against.

use std::cmp::Ordering;
use std::fmt;

/// llama-swap's repository, which publishes the release archives.
pub const REPOSITORY: &str = "https://github.com/mostlygeek/llama-swap";
/// The executable in a release archive, and once installed.
pub const BINARY: &str = "llama-swap";
/// The platform of the archives ai-ctl installs.
const PLATFORM: &str = "linux_amd64";

/// The release `gateway install` installs, the only one, with the
/// SHA-256 of its archive for [`PLATFORM`]. v267 brings the `capacity`
/// router the generated configuration uses, on top of `disableKeepAlives`
/// (v263).
pub const PINNED: Pinned = Pinned {
    release: Release(271),
    sha256: "58d5c897e14fa0f8c11d48b477b0b9b3da6f9f9f79d733957c06baf12f40c68c",
};

/// The oldest release that reads the configuration ai-ctl writes: v267
/// brings the `capacity` router it runs the models on.
pub const OLDEST: Release = Release(267);

/// Where an installed llama-swap release stands against [`OLDEST`] and the
/// release this ai-ctl pins, the only one `gateway install` installs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SwapStanding {
    /// Older than [`OLDEST`]: it cannot read the configuration ai-ctl writes.
    TooOld,
    /// Behind the pin, from [`OLDEST`] on.
    Behind,
    /// The pinned release.
    Pinned,
    /// Ahead of the pin, as a newer ai-ctl or a hand install leaves it.
    Ahead,
}

/// Where `release` stands against [`OLDEST`] and [`PINNED`].
pub fn swap_standing(release: Release) -> SwapStanding {
    if release < OLDEST {
        return SwapStanding::TooOld;
    }
    match release.cmp(&PINNED.release) {
        Ordering::Less => SwapStanding::Behind,
        Ordering::Equal => SwapStanding::Pinned,
        Ordering::Greater => SwapStanding::Ahead,
    }
}

/// A release ai-ctl knows the archive's SHA-256 of.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Pinned {
    pub release: Release,
    pub sha256: &'static str,
}

/// A llama-swap release, numbered: its tag is `v263`, and its directory once
/// installed too.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Release(u32);

impl Release {
    /// Reads a release as its tag names it, and its directory once
    /// installed: `v` then the number, with no leading zero, so never 0;
    /// `None` when `text` is not one.
    pub fn parse(text: &str) -> Option<Self> {
        let digits = text.strip_prefix('v')?;
        match digits.parse() {
            Ok(number)
                if !digits.starts_with('0') && digits.bytes().all(|b| b.is_ascii_digit()) =>
            {
                Some(Self(number))
            }
            _ => None,
        }
    }

    /// The archive of this release for the platform ai-ctl runs on.
    pub fn archive(self) -> String {
        format!("{BINARY}_{}_{PLATFORM}.tar.gz", self.0)
    }

    /// Where a file of the release is downloaded from.
    pub fn url(self, file: &str) -> String {
        format!("{REPOSITORY}/releases/download/{self}/{file}")
    }
}

impl fmt::Display for Release {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "v{}", self.0)
    }
}

/// A SHA-256 digest: 64 lower-case hexadecimal digits.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Sha256(String);

/// Why a text is not a SHA-256 digest.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("`{0}` is not a SHA-256 digest: 64 lower-case hexadecimal digits")]
pub struct InvalidSha256(pub String);

impl Sha256 {
    pub fn parse(text: &str) -> Result<Self, InvalidSha256> {
        if text.len() == 64
            && text
                .bytes()
                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
        {
            Ok(Self(text.to_owned()))
        } else {
            Err(InvalidSha256(text.to_owned()))
        }
    }
}

impl fmt::Display for Sha256 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// Whether the output of `llama-swap -version` names `release`, as in
/// `version: v271 (06de675), built at 2026-11-08T05:18:35Z`.
pub fn reports(version_output: &str, release: Release) -> bool {
    version_output
        .trim_start()
        .strip_prefix("version:")
        .and_then(|rest| rest.split_whitespace().next())
        .is_some_and(|tag| tag == release.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The checksums file of v271.
    const CHECKSUMS: &str = include_str!("../../tests/fixtures/llama-swap-271-checksums.txt");

    #[test]
    fn a_release_is_its_tag() {
        assert_eq!(Release::parse("v263"), Some(Release(263)));
        assert_eq!(Release(263).to_string(), "v263");
        for text in [
            "", "v", "0", "v0", "263", "v0263", "v+263", "+263", "26 3", "vv263", "263.1",
            "latest", "V263",
        ] {
            assert!(Release::parse(text).is_none(), "{text}");
        }
    }

    #[test]
    fn a_release_stands_against_the_oldest_and_the_pin() {
        let pinned = PINNED.release.0;
        let oldest = OLDEST.0;
        assert!(oldest < pinned);
        for (release, standing) in [
            (1, SwapStanding::TooOld),
            (oldest - 1, SwapStanding::TooOld),
            (oldest, SwapStanding::Behind),
            (pinned - 1, SwapStanding::Behind),
            (pinned, SwapStanding::Pinned),
            (pinned + 1, SwapStanding::Ahead),
            (u32::MAX, SwapStanding::Ahead),
        ] {
            assert_eq!(swap_standing(Release(release)), standing, "v{release}");
        }
    }

    #[test]
    fn a_release_names_its_archive_and_urls() {
        let release = Release(263);
        assert_eq!(release.archive(), "llama-swap_263_linux_amd64.tar.gz");
        assert_eq!(
            release.url(&release.archive()),
            "https://github.com/mostlygeek/llama-swap/releases/download/v263/llama-swap_263_linux_amd64.tar.gz"
        );
    }

    #[test]
    fn the_pinned_digest_is_the_one_its_release_publishes() {
        // `sha256sum`'s line for the pinned archive, as the release's
        // checksums file holds it.
        let line = format!("{}  {}", PINNED.sha256, PINNED.release.archive());
        assert!(CHECKSUMS.lines().any(|held| held == line), "{line}");
        assert!(Sha256::parse(PINNED.sha256).is_ok());
    }

    #[test]
    fn a_digest_is_64_lower_case_hexadecimal_digits() {
        let digest = "9f561442a43ed71043105891a10a41019efdc8c023f5a3c807bcf12fbd2a048d";
        assert!(Sha256::parse(digest).is_ok());
        assert!(Sha256::parse(&digest.to_uppercase()).is_err());
        assert!(Sha256::parse(&digest[1..]).is_err());
        assert!(Sha256::parse(&format!("{}g", &digest[1..])).is_err());
    }

    #[test]
    fn the_version_output_must_name_the_release() {
        let output = include_str!("../../tests/fixtures/llama-swap-271-version.txt");
        assert!(reports(output, Release(271)));
        assert!(!reports(output, Release(27)));
        assert!(!reports(output, Release(2710)));
        assert!(!reports(output, Release(270)));
        assert!(!reports("", Release(271)));
        assert!(!reports("v271\n", Release(271)));
    }
}
