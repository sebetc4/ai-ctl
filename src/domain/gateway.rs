//! The gateway, llama-swap: the releases ai-ctl installs.
//!
//! ai-ctl ships with one pinned release, whose archive's SHA-256 is written
//! here. An archive is checked against the digest given with `--sha256`;
//! else the pinned release's against this one, and any other release's
//! against the checksums file published with it.

use std::fmt;

/// llama-swap's repository, which publishes the release archives.
pub const REPOSITORY: &str = "https://github.com/mostlygeek/llama-swap";
/// The executable in a release archive, and once installed.
pub const BINARY: &str = "llama-swap";
/// The platform of the archives ai-ctl installs.
const PLATFORM: &str = "linux_amd64";

/// The release `gateway install` installs unless told otherwise, with the
/// SHA-256 of its archive for [`PLATFORM`]. v263 brings `disableKeepAlives`
/// against the 502 errors of a reused connection that llama-server closed
/// (llama-swap #1211).
pub const PINNED: Pinned = Pinned {
    release: Release(263),
    sha256: "9f561442a43ed71043105891a10a41019efdc8c023f5a3c807bcf12fbd2a048d",
};

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

/// Why a text is not a release.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("`{0}` is not a llama-swap release: a number such as 263, or a tag such as v263")]
pub struct InvalidRelease(pub String);

impl Release {
    /// Reads a release number, with or without the tag's `v`.
    pub fn parse(text: &str) -> Result<Self, InvalidRelease> {
        let digits = text.strip_prefix('v').unwrap_or(text);
        match digits.parse() {
            Ok(number) if number > 0 && digits.bytes().all(|b| b.is_ascii_digit()) => {
                Ok(Self(number))
            }
            _ => Err(InvalidRelease(text.to_owned())),
        }
    }

    /// The archive of this release for the platform ai-ctl runs on.
    pub fn archive(self) -> String {
        format!("{BINARY}_{}_{PLATFORM}.tar.gz", self.0)
    }

    /// The checksums file published with the release.
    pub fn checksums(self) -> String {
        format!("{BINARY}_{}_checksums.txt", self.0)
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

/// The digest a checksums file gives `file`, in the format `sha256sum`
/// writes (`<digest>  <file>`, or `<digest> *<file>`); `None` when the file
/// is not listed or its digest is malformed.
pub fn checksum(checksums: &str, file: &str) -> Option<Sha256> {
    checksums.lines().find_map(|line| {
        let (digest, name) = line.trim().split_once(char::is_whitespace)?;
        let name = name.trim_start();
        let name = name.strip_prefix('*').unwrap_or(name);
        if name == file {
            Sha256::parse(digest).ok()
        } else {
            None
        }
    })
}

/// Whether the output of `llama-swap -version` names `release`, as in
/// `version: v263 (e33e97b), built at 2026-10-10T06:17:54Z`.
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

    /// The checksums file of v263.
    const CHECKSUMS: &str = include_str!("../../tests/fixtures/llama-swap-263-checksums.txt");

    #[test]
    fn a_release_is_a_number_with_or_without_its_v() {
        assert_eq!(Release::parse("263"), Ok(Release(263)));
        assert_eq!(Release::parse("v263"), Ok(Release(263)));
        assert_eq!(Release(263).to_string(), "v263");
        for text in [
            "", "v", "0", "v0", "+263", "26 3", "vv263", "263.1", "latest", "V263",
        ] {
            assert!(Release::parse(text).is_err(), "{text}");
        }
    }

    #[test]
    fn a_release_names_its_archive_checksums_and_urls() {
        let release = Release(263);
        assert_eq!(release.archive(), "llama-swap_263_linux_amd64.tar.gz");
        assert_eq!(release.checksums(), "llama-swap_263_checksums.txt");
        assert_eq!(
            release.url(&release.archive()),
            "https://github.com/mostlygeek/llama-swap/releases/download/v263/llama-swap_263_linux_amd64.tar.gz"
        );
    }

    #[test]
    fn the_pinned_digest_is_the_one_its_release_publishes() {
        let published = checksum(CHECKSUMS, &PINNED.release.archive()).unwrap();
        assert_eq!(Sha256::parse(PINNED.sha256), Ok(published));
    }

    #[test]
    fn a_checksums_file_gives_the_digest_of_one_file() {
        assert_eq!(
            checksum(CHECKSUMS, "llama-swap_263_linux_arm64.tar.gz")
                .unwrap()
                .to_string(),
            "eb2a67d5212b688e32038e3d9c4b96dc2c64d5a16208ba008e30b4af4a7becb0"
        );
        assert_eq!(
            checksum(CHECKSUMS, "llama-swap_264_linux_amd64.tar.gz"),
            None
        );
        assert_eq!(checksum(CHECKSUMS, "linux_amd64.tar.gz"), None);
        let digest = "9f561442a43ed71043105891a10a41019efdc8c023f5a3c807bcf12fbd2a048d";
        let binary_mode = format!("{digest} *llama-swap_263_linux_amd64.tar.gz\n");
        assert!(checksum(&binary_mode, "llama-swap_263_linux_amd64.tar.gz").is_some());
        let malformed = "9F56  llama-swap_263_linux_amd64.tar.gz\n";
        assert_eq!(
            checksum(malformed, "llama-swap_263_linux_amd64.tar.gz"),
            None
        );
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
        let output = include_str!("../../tests/fixtures/llama-swap-263-version.txt");
        assert!(reports(output, Release(263)));
        assert!(!reports(output, Release(26)));
        assert!(!reports(output, Release(2630)));
        assert!(!reports("", Release(263)));
        assert!(!reports("v263\n", Release(263)));
    }
}
