//! The repository's text: no tracked text file outside `docs/reviews/` holds a raw U+2028
//! or U+2029. The agents' Write and Edit tools leave one where an escape was meant, and
//! llama-swap's YAML reads one as a line end (docs/security.md); `\u2028` is how a file
//! writes it. A review quotes what it found as it found it, hence `docs/reviews/`.

use std::fs;
use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};
use std::process::Command;

/// U+2028 and U+2029 in UTF-8.
const SEPARATORS: [[u8; 3]; 2] = [[0xe2, 0x80, 0xa8], [0xe2, 0x80, 0xa9]];
/// A file with a NUL byte in its first 8,000 bytes is binary, as git counts it.
const BINARY_PROBE: usize = 8000;

/// The places, `path:line`, of the raw separators in `files`, read under `root`; a binary
/// file, a file missing from the working tree, a symbolic link and a submodule are skipped.
fn raw_separators(root: &Path, files: &[PathBuf]) -> Vec<String> {
    let mut found = Vec::new();
    for file in files {
        let path = root.join(file);
        match fs::symlink_metadata(&path) {
            Ok(metadata) if metadata.is_file() => {}
            Ok(_) => continue,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
            Err(error) => panic!("{}: {error}", file.display()),
        }
        let bytes = fs::read(&path).unwrap_or_else(|error| panic!("{}: {error}", file.display()));
        if bytes[..bytes.len().min(BINARY_PROBE)].contains(&0) {
            continue;
        }
        for (number, line) in bytes.split(|byte| *byte == b'\n').enumerate() {
            if line
                .windows(3)
                .any(|window| SEPARATORS.iter().any(|s| window == s))
            {
                found.push(format!("{}:{}", file.display(), number + 1));
            }
        }
    }
    found
}

/// The paths `git ls-files -z` printed, but for the reviews'.
fn checked(listed: &[u8]) -> Vec<PathBuf> {
    listed
        .split(|byte| *byte == 0)
        .filter(|path| !path.is_empty() && !path.starts_with(b"docs/reviews/"))
        .map(|path| PathBuf::from(std::ffi::OsStr::from_bytes(path)))
        .collect()
}

#[test]
fn the_reviews_are_left_out() {
    let listed = b"README.md\0docs/reviews/delivered-code/report.md\0docs/reviewsx.md\0";
    assert_eq!(
        checked(listed),
        ["README.md", "docs/reviewsx.md"].map(PathBuf::from)
    );
}

#[test]
fn a_raw_separator_is_found_at_its_line() {
    let folder = tempfile::tempdir().unwrap();
    fs::write(
        folder.path().join("a.md"),
        "one\ntwo \u{2028} three\u{2028}\n",
    )
    .unwrap();
    fs::write(folder.path().join("b.md"), "one\u{2029}\n").unwrap();
    fs::write(folder.path().join("c.md"), "an escape: \\u2028\n").unwrap();
    fs::write(folder.path().join("d.bin"), b"\0\xe2\x80\xa8").unwrap();
    std::os::unix::fs::symlink("a.md", folder.path().join("e.md")).unwrap();
    let files = ["a.md", "b.md", "c.md", "d.bin", "e.md", "gone.md"].map(PathBuf::from);
    assert_eq!(raw_separators(folder.path(), &files), ["a.md:2", "b.md:1"]);
}

#[test]
fn no_tracked_text_file_holds_a_raw_separator() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let output = Command::new("git")
        .args(["ls-files", "-z"])
        .current_dir(root)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "git ls-files failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        raw_separators(root, &checked(&output.stdout)),
        Vec::<String>::new(),
        "write each separator as its escape, \\u2028 or \\u2029"
    );
}
