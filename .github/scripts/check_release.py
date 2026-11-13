"""Refuse a release whose tag is not the version Cargo.toml holds, or whose CHANGELOG.md entry
for that version is not dated.

Usage: check_release.py <tag> [<repository root>]

The release workflow runs it on a pushed tag before anything else. A release is
`vX.Y.Z` on a commit where Cargo.toml's [package] holds `version = "X.Y.Z"` and CHANGELOG.md
heads its entry `## X.Y.Z — YYYY-MM-DD`; `## X.Y.Z — unreleased` is not released yet. Prints
`release <tag>: …` and exits 0, or one `refused: …` line per problem and exits 1; 2 on a
wrong use.
"""

import datetime
import re
import sys
import tomllib
from pathlib import Path

TAG_RE = re.compile(r"v(\d+\.\d+\.\d+)")
ENTRY_RE = re.compile(r"^## (\d+\.\d+\.\d+) — (.*?)\s*$", re.M)
DATE_RE = re.compile(r"\d{4}-\d{2}-\d{2}")


def cargo_version(cargo_toml):
    """The version Cargo.toml's [package] holds, and a problem when there is none."""
    try:
        manifest = tomllib.loads(cargo_toml)
    except tomllib.TOMLDecodeError as error:
        return None, f"Cargo.toml is not valid TOML: {error}"
    version = manifest.get("package", {}).get("version")
    if not isinstance(version, str):
        return None, "Cargo.toml's [package] has no version"
    return version, None


def entries(changelog, version):
    """What follows the dash in each heading `## <version> — …` of the changelog."""
    return [m.group(2) for m in ENTRY_RE.finditer(changelog) if m.group(1) == version]


def dated(text):
    """Whether text is a date, YYYY-MM-DD, that exists."""
    if not DATE_RE.fullmatch(text):
        return False
    try:
        datetime.date.fromisoformat(text)
    except ValueError:
        return False
    return True


def problems(tag, cargo_toml, changelog):
    """Every reason to refuse releasing tag from these Cargo.toml and CHANGELOG.md texts."""
    found = []
    tagged = TAG_RE.fullmatch(tag)
    if not tagged:
        found.append(f"tag `{tag}` is not of the form vX.Y.Z")
    version, problem = cargo_version(cargo_toml)
    if problem:
        found.append(problem)
    elif tagged and tagged.group(1) != version:
        found.append(f"tag `{tag}` is not Cargo.toml's version {version}")
    version = version or (tagged and tagged.group(1))
    if not version:
        return found
    heads = entries(changelog, version)
    if not heads:
        found.append(f"CHANGELOG.md has no entry for {version}, `## {version} — YYYY-MM-DD`")
    elif len(heads) > 1:
        found.append(f"CHANGELOG.md has {len(heads)} entries for {version}")
    elif not dated(heads[0]):
        found.append(f"CHANGELOG.md's entry for {version} is not dated: `{heads[0]}`, "
                     f"where a release reads `## {version} — YYYY-MM-DD`")
    return found


def read(path):
    """The file's text, and a problem when it cannot be read."""
    try:
        return path.read_text(encoding="utf-8"), None
    except (OSError, UnicodeDecodeError) as error:
        return None, f"cannot read {path}: {error}"


def main(argv):
    if len(argv) not in (1, 2):
        print("usage: check_release.py <tag> [<repository root>]", file=sys.stderr)
        return 2
    tag, root = argv[0], Path(argv[1] if len(argv) == 2 else ".")
    cargo_toml, cargo_problem = read(root / "Cargo.toml")
    changelog, changelog_problem = read(root / "CHANGELOG.md")
    found = [p for p in (cargo_problem, changelog_problem) if p]
    if not found:
        found = problems(tag, cargo_toml, changelog)
    for problem in found:
        print(f"refused: {problem}")
    if found:
        return 1
    version = TAG_RE.fullmatch(tag).group(1)
    print(f"release {tag}: Cargo.toml {version}, CHANGELOG.md dated {entries(changelog, version)[0]}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
