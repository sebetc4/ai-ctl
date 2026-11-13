#!/usr/bin/env python3
"""Unit tests of scripts/scope.py, run on throwaway git repositories.

Usage: test_scope.py [unittest options]
"""

import os
import re
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

HERE = Path(__file__).resolve().parent
SCRIPT = HERE.parent / "scripts" / "scope.py"
sys.dont_write_bytecode = True
sys.path[:0] = [str(SCRIPT.parent), str(HERE)]

import fixtures  # noqa: E402
import scope  # noqa: E402

GIT_ENV = {**os.environ, "GIT_CONFIG_GLOBAL": os.devnull, "GIT_CONFIG_NOSYSTEM": "1",
           "GIT_AUTHOR_NAME": "Test", "GIT_AUTHOR_EMAIL": "test@example.com",
           "GIT_COMMITTER_NAME": "Test", "GIT_COMMITTER_EMAIL": "test@example.com",
           "GIT_AUTHOR_DATE": "2026-10-01T10:00:00+00:00", "GIT_COMMITTER_DATE": "2026-10-01T10:00:00+00:00"}
RUST = "fn main() {}\nfn helper() {}\n#[cfg(test)]\nmod tests {\n    #[test]\n    fn t() {}\n}\n"


class Repo:
    def __init__(self, folder):
        self.folder = Path(folder)
        self.git("init", "-q", "-b", "main")

    def git(self, *args):
        done = subprocess.run(["git", *args], cwd=self.folder, check=True, capture_output=True, text=True,
                              env=GIT_ENV)
        return done.stdout.strip()

    def write(self, path, text):
        target = self.folder / path
        target.parent.mkdir(parents=True, exist_ok=True)
        if isinstance(text, bytes):
            target.write_bytes(text)
        else:
            target.write_text(text, encoding="utf-8")

    def commit(self, message, files=None, remove=()):
        for path, text in (files or {}).items():
            self.write(path, text)
        for path in remove:
            self.git("rm", "-q", path)
        self.git("add", "-A")
        self.git("commit", "-q", "--allow-empty", "-m", message)
        return self.git("rev-parse", "--short", "HEAD")

    def report(self, roadmap, number, start):
        line = f"**Start Commit:** {start}\n" if start else ""
        return {f"{roadmap}/phase-{number}-work-report.md": f"# Phase {number} Report\n\n{line}\n## Work Log\n"}


def run(folder, *args):
    return subprocess.run([str(SCRIPT), "--repo", str(folder), *args], capture_output=True, text=True,
                          env=GIT_ENV)


def lines(output, key):
    return [line[len(key) + 2:] for line in output.splitlines() if line.startswith(key + ": ")]


class ScopeTest(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.repo = Repo(self.tmp.name)

    def tearDown(self):
        self.tmp.cleanup()


class RoadmapTest(ScopeTest):
    ROADMAP = "docs/roadmap/completed/demo"

    def test_base_is_the_first_phase_start_commit_and_end_the_last_commit_touching_the_folder(self):
        root = self.repo.commit("root", {"README.md": "demo\n"})
        phase0 = self.repo.report(self.ROADMAP, 0, root)
        first = self.repo.commit("phase 0", {"src/main.rs": RUST, **phase0})
        phase1 = self.repo.report(self.ROADMAP, 1, first)
        closing = self.repo.commit("phase 1", {"tests/cli.rs": "#[test]\nfn cli() {}\n", **phase0, **phase1})
        self.repo.commit("later work", {"src/later.rs": "fn later() {}\n"})

        done = run(self.repo.folder, "roadmap", self.ROADMAP)

        self.assertEqual(done.returncode, 0, done.stderr)
        self.assertEqual(lines(done.stdout, "scope"), [f"roadmap {self.ROADMAP}"])
        self.assertEqual(lines(done.stdout, "start commit"), [f"phase 0 {root} the root commit", f"phase 1 {first}"])
        self.assertEqual(lines(done.stdout, "base"), [f"{root}, the start commit of phase 0"])
        self.assertEqual(lines(done.stdout, "end"), [f"{closing}, the last commit touching a phase report of {self.ROADMAP}"])
        self.assertEqual(lines(done.stdout, "diff"), [f"git diff {root} {closing}"])
        self.assertEqual(lines(done.stdout, "files"),
                         ["4: code 2, dependencies 0, documentation 0, captures 0, other 2"])
        self.assertEqual(lines(done.stdout, "hunks"), ["2 in 2 code files, +9 -0"])

    def test_a_later_commit_touching_the_folder_but_no_report_is_left_out(self):
        root = self.repo.commit("root", {"README.md": "demo\n"})
        closing = self.repo.commit("phase 0", {"src/main.rs": RUST, **self.repo.report(self.ROADMAP, 0, root)})
        self.repo.commit("another roadmap", {f"{self.ROADMAP}/README.md": "# Demo\n\nWhat it does.\n",
                                             "src/later.rs": "fn later() {}\n"})

        done = run(self.repo.folder, "roadmap", self.ROADMAP)

        self.assertEqual(done.returncode, 0, done.stderr)
        self.assertEqual(lines(done.stdout, "end"),
                         [f"{closing}, the last commit touching a phase report of {self.ROADMAP}"])
        self.assertEqual(lines(done.stdout, "diff"), [f"git diff {root} {closing}"])

    def test_a_first_start_commit_absent_from_the_history_takes_the_whole_history(self):
        root = self.repo.commit("root", {"src/main.rs": RUST, "README.md": "demo\n"})
        reports = {**self.repo.report(self.ROADMAP, 0, "1a49576"), **self.repo.report(self.ROADMAP, 1, root)}
        closing = self.repo.commit("phase 1", {"src/lib.rs": "pub fn lib() {}\n", **reports})

        done = run(self.repo.folder, "roadmap", self.ROADMAP)

        self.assertEqual(done.returncode, 0, done.stderr)
        self.assertEqual(lines(done.stdout, "start commit"),
                         ["phase 0 1a49576 absent from the history", f"phase 1 {root} the root commit"])
        self.assertEqual(lines(done.stdout, "base"),
                         [f"none: the start commit of phase 0 is absent from the history, so the scope is the "
                          f"whole history up to the end, from the root commit {root}"])
        empty_tree = self.repo.git("hash-object", "-t", "tree", "/dev/null")
        self.assertEqual(lines(done.stdout, "diff"), [f"git diff {empty_tree} {closing}"])
        self.assertEqual(lines(done.stdout, "files"),
                         ["5: code 2, dependencies 0, documentation 1, captures 0, other 2"])

    def test_phases_are_ordered_by_number_not_by_name(self):
        root = self.repo.commit("root", {"README.md": "demo\n"})
        second = self.repo.commit("phase 2", self.repo.report(self.ROADMAP, 2, root))
        self.repo.commit("phase 10", self.repo.report(self.ROADMAP, 10, second))

        done = run(self.repo.folder, "roadmap", self.ROADMAP)

        self.assertEqual(done.returncode, 0, done.stderr)
        self.assertEqual(lines(done.stdout, "start commit"), [f"phase 2 {root} the root commit",
                                                              f"phase 10 {second}"])
        self.assertEqual(lines(done.stdout, "base"), [f"{root}, the start commit of phase 2"])

    def test_a_report_without_a_start_commit_is_named(self):
        root = self.repo.commit("root", {"README.md": "demo\n"})
        self.repo.commit("phases", {**self.repo.report(self.ROADMAP, 0, root), **self.repo.report(self.ROADMAP, 1, None)})

        done = run(self.repo.folder, "roadmap", self.ROADMAP)

        self.assertEqual(done.returncode, 0, done.stderr)
        self.assertEqual(lines(done.stdout, "start commit"), [f"phase 0 {root} the root commit",
                                                              "phase 1 none in phase-1-work-report.md"])

    def test_a_folder_without_any_start_commit_fails(self):
        self.repo.commit("root", {"README.md": "demo\n", **self.repo.report(self.ROADMAP, 0, None)})

        done = run(self.repo.folder, "roadmap", self.ROADMAP)

        self.assertEqual(done.returncode, 1)
        self.assertIn(f"no phase report in {self.ROADMAP} gives a Start Commit", done.stderr)

    def test_a_missing_folder_fails(self):
        self.repo.commit("root", {"README.md": "demo\n"})

        done = run(self.repo.folder, "roadmap", "docs/roadmap/completed/none")

        self.assertEqual(done.returncode, 1)
        self.assertIn("docs/roadmap/completed/none is not a folder", done.stderr)

    def test_to_sets_the_end(self):
        root = self.repo.commit("root", {"README.md": "demo\n"})
        self.repo.commit("phase 0", self.repo.report(self.ROADMAP, 0, root))
        later = self.repo.commit("later work", {"src/later.rs": "fn later() {}\n"})

        done = run(self.repo.folder, "roadmap", self.ROADMAP, "--to", later)

        self.assertEqual(done.returncode, 0, done.stderr)
        self.assertEqual(lines(done.stdout, "end"), [f"{later}, given by --to"])


class PrivateRootRoadmapTest(ScopeTest):
    """A roadmap under the private root, which the project ignores and its own repository commits."""

    ROADMAP = "private/roadmap/completed/demo"
    REPORT = "roadmap/completed/demo/phase-0-work-report.md"

    def setUp(self):
        super().setUp()
        self.repo.write(".gitignore", "/private\n")
        (self.repo.folder / "private").mkdir()
        self.private = Repo(self.repo.folder / "private")

    def at(self, repo, hour, message, files):
        for path, text in files.items():
            repo.write(path, text)
        date = f"2026-10-01T{hour:02}:00:00+00:00"
        env = {**GIT_ENV, "GIT_AUTHOR_DATE": date, "GIT_COMMITTER_DATE": date}
        subprocess.run(["git", "add", "-A"], cwd=repo.folder, check=True, env=env)
        subprocess.run(["git", "commit", "-q", "-m", message], cwd=repo.folder, check=True, env=env)
        return repo.git("rev-parse", "--short", "HEAD")

    def report(self, start, more=""):
        return {self.REPORT: f"# Phase 0 Report\n\n**Start Commit:** {start}\n\n## Work Log\n{more}"}

    def test_the_end_is_the_project_commit_dated_at_most_the_last_report_commit_of_the_private_root(self):
        root = self.at(self.repo, 10, "root", {"README.md": "demo\n"})
        self.at(self.private, 11, "the opening", self.report(root))
        closing = self.at(self.repo, 12, "phase 0", {"src/main.rs": RUST})
        self.at(self.private, 13, "the closure", self.report(root, "Closed.\n"))
        self.at(self.repo, 14, "later work", {"src/later.rs": "fn later() {}\n"})
        self.at(self.private, 15, "no report", {"roadmap/completed/demo/README.md": "# Demo\n"})

        done = run(self.repo.folder, "roadmap", self.ROADMAP)

        self.assertEqual(done.returncode, 0, done.stderr)
        self.assertEqual(lines(done.stdout, "end"),
                         [f"{closing}, the project's last commit dated at most the last commit of its private root "
                          f"touching a phase report of {self.ROADMAP}"])
        self.assertEqual(lines(done.stdout, "diff"), [f"git diff {root} {closing}"])

    def test_to_sets_the_end(self):
        root = self.at(self.repo, 10, "root", {"README.md": "demo\n"})
        self.at(self.private, 11, "the opening", self.report(root))
        later = self.at(self.repo, 12, "later work", {"src/later.rs": "fn later() {}\n"})

        done = run(self.repo.folder, "roadmap", self.ROADMAP, "--to", later)

        self.assertEqual(done.returncode, 0, done.stderr)
        self.assertEqual(lines(done.stdout, "end"), [f"{later}, given by --to"])

    def test_a_report_no_commit_of_the_private_root_touches_is_named(self):
        self.at(self.private, 9, "the private root", {"roadmap/README.md": "# Roadmaps\n"})
        self.private.write(self.REPORT, "**Start Commit:** 1a49576\n")
        self.at(self.repo, 10, "root", {"README.md": "demo\n"})

        done = run(self.repo.folder, "roadmap", self.ROADMAP)

        self.assertEqual(done.returncode, 1)
        self.assertIn(f"no commit of the private root touches a phase report of {self.ROADMAP}", done.stderr)

    def test_a_report_older_than_every_project_commit_is_named(self):
        self.at(self.private, 9, "the opening", self.report("1a49576"))
        self.at(self.repo, 10, "root", {"README.md": "demo\n"})

        done = run(self.repo.folder, "roadmap", self.ROADMAP)

        self.assertEqual(done.returncode, 1)
        self.assertIn("no project commit is dated at most", done.stderr)


class RangeTest(ScopeTest):
    def test_counts_production_and_tests_at_the_end_and_names_deleted_files(self):
        base = self.repo.commit("base", {"README.md": "demo\n", "src/old.rs": "fn old() {}\n"})
        end = self.repo.commit("session", {"src/main.rs": RUST, "tests/it.rs": "#[test]\nfn it() {}\n",
                                           "docs/note.md": "one\ntwo\n", "logo.png": b"\x89PNG\x00\x01"},
                               remove=["src/old.rs"])

        done = run(self.repo.folder, "range", base, end)

        self.assertEqual(done.returncode, 0, done.stderr)
        self.assertEqual(lines(done.stdout, "scope"), [f"range {base}..{end}"])
        self.assertEqual(lines(done.stdout, "base"), [base])
        self.assertEqual(lines(done.stdout, "end"), [end])
        self.assertEqual(lines(done.stdout, "files"),
                         ["4: code 2, dependencies 0, documentation 1, captures 0, other 1"])
        self.assertEqual(lines(done.stdout, "changed"), ["+11 -1"])
        self.assertEqual(lines(done.stdout, "hunks"), ["2 in 2 code files, +9 -0"])
        self.assertEqual(lines(done.stdout, "deleted"), ["src/old.rs"])

    def test_end_defaults_to_head(self):
        base = self.repo.commit("base", {"README.md": "demo\n"})
        head = self.repo.commit("session", {"src/main.rs": RUST})

        done = run(self.repo.folder, "range", base)

        self.assertEqual(done.returncode, 0, done.stderr)
        self.assertEqual(lines(done.stdout, "end"), [head])

    def test_uncommitted_changes_are_named_outside_the_scope(self):
        base = self.repo.commit("base", {"README.md": "demo\n"})
        self.repo.write("README.md", "changed\n")
        self.repo.write("notes.txt", "new\n")

        done = run(self.repo.folder, "range", base)

        self.assertEqual(done.returncode, 0, done.stderr)
        self.assertEqual(lines(done.stdout, "uncommitted"),
                         ["1 tracked file changed, 1 untracked file, outside the scope"])

    def test_a_clean_work_tree_says_none(self):
        base = self.repo.commit("base", {"README.md": "demo\n"})

        done = run(self.repo.folder, "range", base)

        self.assertEqual(lines(done.stdout, "uncommitted"), ["none"])

    def test_a_base_that_is_not_an_ancestor_of_the_end_fails(self):
        self.repo.commit("root", {"README.md": "demo\n"})
        self.repo.git("checkout", "-q", "-b", "side")
        side = self.repo.commit("side", {"side.txt": "side\n"})
        self.repo.git("checkout", "-q", "main")
        main = self.repo.commit("main", {"main.txt": "main\n"})

        done = run(self.repo.folder, "range", side, main)

        self.assertEqual(done.returncode, 1)
        self.assertIn(f"{side} is not an ancestor of {main}", done.stderr)

    def test_an_unknown_commit_fails(self):
        self.repo.commit("root", {"README.md": "demo\n"})

        done = run(self.repo.folder, "range", "deadbee")

        self.assertEqual(done.returncode, 1)
        self.assertIn("deadbee is not a commit of the repository", done.stderr)


class HunkTest(ScopeTest):
    """The scope's code by hunk, each with the items that hold it."""

    BEFORE = """use std::fmt;

pub struct Store {
    names: Vec<String>,
}

impl Store {
    pub fn count(&self) -> usize {
        self.names.len()
    }

    pub fn first(&self) -> Option<&String> {
        let first = self.names.first();
        debug_assert!(first.is_some() || self.names.is_empty());
        first
    }
}
"""
    AFTER = """use std::fmt;
use std::path::Path;

pub struct Store {
    names: Vec<String>,
}

impl Store {
    pub fn count(&self) -> usize {
        self.names.iter().count()
    }

    pub fn first(&self) -> Option<&String> {
        let first = self.names.first();
        first
    }

    pub fn last(&self) -> Option<&String> {
        self.names.last()
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn counts() {}
}
"""

    def test_each_hunk_names_the_items_holding_it_and_each_item_is_given_whole(self):
        base = self.repo.commit("base", {"src/lib.rs": self.BEFORE})
        self.repo.commit("session", {"src/lib.rs": self.AFTER})

        done = run(self.repo.folder, "range", base)

        self.assertEqual(done.returncode, 0, done.stderr)
        self.assertEqual(lines(done.stdout, "hunks"), ["4 in 1 code file, +12 -2"])
        # The fourth hunk holds the impl's closing brace, which git's alignment moves: a line
        # of closing delimiters names no item, as a blank line.
        self.assertEqual(lines(done.stdout, "hunk"), [
            "src/lib.rs:2 (top level) +1 -0",
            "src/lib.rs:10 (Store::count) +1 -1",
            "src/lib.rs:14 (Store::first) +0 -1",
            "src/lib.rs:17-26 (Store::last, top level, tests::counts) +10 -0",
        ])
        self.assertEqual(lines(done.stdout, "item"), [
            "src/lib.rs:9-11 (Store::count)",
            "src/lib.rs:13-16 (Store::first)",
            "src/lib.rs:18-20 (Store::last)",
            "src/lib.rs:25-26 (tests::counts)",
        ])
        self.assertEqual(lines(done.stdout, "reading"),
                         ["15 lines: production 11, tests 4; 4 items, 3 top-level lines"])

    def test_files_outside_the_code_are_routed_with_their_counts(self):
        base = self.repo.commit("base", {"README.md": "demo\n"})
        self.repo.commit("session", {
            "src/main.rs": "fn main() {}\n",
            "Cargo.toml": "[package]\nname = \"demo\"\n", "Cargo.lock": "version = 4\n",
            "README.md": "demo\nmore\n", "docs/commands.md": "one\n",
            "tests/fixtures/capture.txt": "{\n", "tests/fixtures/sample.rs": "fn sample() {}\n",
            "docs/roadmap/demo/README.md": "# Demo\n",
            "docs/reviews/delivered-code/register.json": "[]\n", "logo.png": b"\x89PNG\x00\x01"})

        done = run(self.repo.folder, "range", base)

        self.assertEqual(done.returncode, 0, done.stderr)
        self.assertEqual(lines(done.stdout, "files"),
                         ["10: code 1, dependencies 2, documentation 2, captures 2, other 3"])
        self.assertEqual(lines(done.stdout, "hunks"), ["1 in 1 code file, +1 -0"])
        self.assertEqual(lines(done.stdout, "dependencies"), ["Cargo.lock +1 -0", "Cargo.toml +2 -0"])
        self.assertEqual(lines(done.stdout, "documentation"), ["README.md +1 -0", "docs/commands.md +1 -0"])
        self.assertEqual(lines(done.stdout, "captures"),
                         ["tests/fixtures/capture.txt +1 -0", "tests/fixtures/sample.rs +1 -0"])
        self.assertEqual(lines(done.stdout, "other"), ["docs/reviews/delivered-code/register.json +1 -0",
                                                       "docs/roadmap/demo/README.md +1 -0", "logo.png binary"])

    def test_the_files_the_production_code_embeds_are_code(self):
        rust = ('pub const CORPUS: &str = include_str!(\n    "../assets/corpus.txt"\n);\n'
                'pub const PATCH: &[u8] = include_bytes!("patches/a.patch");\n\n'
                '#[cfg(test)]\nmod tests {\n    const SAMPLE: &str = include_str!("../data/sample.txt");\n}\n')
        base = self.repo.commit("base", {"src/lib.rs": rust, "assets/corpus.txt": "one\n",
                                         "src/patches/a.patch": "line one\n", "data/sample.txt": "one\n"})
        self.repo.commit("session", {"assets/corpus.txt": "one\ntwo\n",
                                     "src/patches/a.patch": "line one\nline two\nline three\n",
                                     "data/sample.txt": "one\ntwo\n"})

        done = run(self.repo.folder, "range", base)

        self.assertEqual(done.returncode, 0, done.stderr)
        self.assertEqual(lines(done.stdout, "files"),
                         ["3: code 2, dependencies 0, documentation 0, captures 0, other 1"])
        self.assertEqual(lines(done.stdout, "hunk"), ["assets/corpus.txt:2 (top level) +1 -0",
                                                      "src/patches/a.patch:2-3 (top level) +2 -0"])
        self.assertEqual(lines(done.stdout, "reading"),
                         ["3 lines: production 3, tests 0; 0 items, 3 top-level lines"])
        self.assertEqual(lines(done.stdout, "other"), ["data/sample.txt +1 -0"])


class OriginTest(ScopeTest):
    """Each line's origin against the scope's base, by git blame, as the report writer asks it."""

    def origin(self, base, *places):
        return scope.origin(scope.repository(self.repo.folder), base, "HEAD", places)

    def session(self):
        self.repo.commit("root", {"README.md": "demo\n"})
        base = self.repo.commit("base", {"src/lib.rs": HunkTest.BEFORE})
        self.repo.commit("session", {"src/lib.rs": HunkTest.AFTER})
        return base

    def test_a_line_changed_after_the_base_is_new_and_one_of_the_base_older(self):
        base = self.session()

        self.assertEqual(self.origin(base, ("src/lib.rs", 10, 10)), "new")
        self.assertEqual(self.origin(base, ("src/lib.rs", 13, 16)), "older")
        self.assertEqual(self.origin(base, ("src/lib.rs", 9, 11)), "mixed")
        self.assertEqual(self.origin(base, ("src/lib.rs", 2, 2), ("src/lib.rs", 14, 14)), "mixed")

    def test_a_closing_delimiter_git_aligns_with_older_code_counts_for_nothing(self):
        base = self.session()

        # git blame gives the test module's closing brace, line 27, to the impl's.
        self.assertEqual(self.origin(base, ("src/lib.rs", 23, 27)), "new")

    def test_a_line_only_reindented_keeps_its_origin(self):
        base = self.repo.commit("base", {"src/lib.rs": "fn run() {\n    work();\n}\n"})
        self.repo.commit("session", {"src/lib.rs": "fn run() {\n        work();\n}\n"})

        self.assertEqual(self.origin(base, ("src/lib.rs", 2, 2)), "older")

    def test_a_file_created_in_the_scope_is_new(self):
        base = self.repo.commit("base", {"README.md": "demo\n"})
        self.repo.commit("session", {"src/new.rs": "fn new() {\n    made();\n}\n"})

        self.assertEqual(self.origin(base, ("src/new.rs", 1, 3)), "new")

    def test_without_a_base_every_line_is_new(self):
        self.repo.commit("root", {"src/lib.rs": HunkTest.BEFORE})

        self.assertEqual(self.origin(None, ("src/lib.rs", 1, 17)), "new")

    def test_a_line_beyond_the_file_or_a_file_absent_at_the_end_fails(self):
        base = self.session()

        with self.assertRaisesRegex(scope.ScopeError, "src/lib.rs has 27 lines, not 28-30"):
            self.origin(base, ("src/lib.rs", 28, 30))
        with self.assertRaisesRegex(scope.ScopeError, "src/gone.rs"):
            self.origin(base, ("src/gone.rs", 1, 1))


def at(text, needle):
    """The number of the one line of `text` that holds `needle`."""
    found = [number for number, line in enumerate(text.splitlines(), 1) if needle in line]
    if len(found) != 1:
        raise AssertionError(f"{needle!r} is on lines {found}")
    return found[0]


class ItemsTest(unittest.TestCase):
    """Items found by their keywords and matched braces, whatever strings and comments hold."""

    SOURCE = r"""//! A module { with a brace in its doc.

use std::fmt::{self, Display};

/// A constant with braces in its value.
pub const BRACES: &str = "{ not a block }";
static TABLE: [u8; 2] = [1, 2];

/// Kinds.
#[derive(Debug)]
pub(crate) enum Kind {
    A,
    B { x: u8 },
} // end Kind

// A plain comment above an item.
pub struct Unit;

// A comment kept apart by a blank line.

pub struct Pair(u8, u8);

type Alias = Vec<u8>;

pub trait Store {
    /// The names.
    fn names(&self) -> Vec<String>;
    fn count(&self) -> usize {
        self.names().len()
    } // end count
} // end Store

impl<'a> Holder<'a> {
    pub const fn new() -> Self {
        Holder { x: '{' }
    } // end new

    async unsafe fn tricky(&self) -> &'a str {
        let _ = '}';
        let _ = '\'';
        let _ = b'{';
        let _ = r#"a "}" raw"#;
        let _ = b"{";
        let _ = "an escaped \" quote {";
        /* a { block /* nested } */ comment } */
        // a { line comment
        let f = |x: u8| { x };
        fn inner() {
        }
        "}"
    } // end tricky
} // end Holder

impl Display for Kind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{{}}")
    } // end fmt
} // end Display

impl<F: Fn() -> u8> Runner<F>
where
    F: Copy,
{
    fn run(&self) {}
} // end Runner

macro_rules! twice {
    ($e:expr) => {{ $e; $e }};
} // end twice

thread_local! {
    static COUNT: u8 = 0;
}

extern "C" fn callback() {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn works() {
        assert_eq!(twice!(1), 1);
    } // end works
}
"""

    def test_each_item_from_its_attributes_and_comments_to_its_end(self):
        s = self.SOURCE
        expected = [
            ("BRACES", at(s, "/// A constant with braces"), at(s, "pub const BRACES")),
            ("TABLE", at(s, "static TABLE"), at(s, "static TABLE")),
            ("Kind", at(s, "/// Kinds."), at(s, "} // end Kind")),
            ("Unit", at(s, "// A plain comment above"), at(s, "pub struct Unit;")),
            ("Pair", at(s, "pub struct Pair"), at(s, "pub struct Pair")),
            ("Alias", at(s, "type Alias"), at(s, "type Alias")),
            ("trait Store", at(s, "pub trait Store"), at(s, "} // end Store")),
            ("Store::names", at(s, "/// The names."), at(s, "fn names(&self)")),
            ("Store::count", at(s, "fn count(&self)"), at(s, "} // end count")),
            ("impl Holder", at(s, "impl<'a> Holder<'a>"), at(s, "} // end Holder")),
            ("Holder::new", at(s, "pub const fn new"), at(s, "} // end new")),
            ("Holder::tricky", at(s, "async unsafe fn tricky"), at(s, "} // end tricky")),
            ("impl Display for Kind", at(s, "impl Display for Kind"), at(s, "} // end Display")),
            ("Kind::fmt", at(s, "fn fmt("), at(s, "} // end fmt")),
            ("impl Runner", at(s, "impl<F: Fn() -> u8> Runner<F>"), at(s, "} // end Runner")),
            ("Runner::run", at(s, "fn run(&self) {}"), at(s, "fn run(&self) {}")),
            ("twice!", at(s, "macro_rules! twice"), at(s, "} // end twice")),
            ("callback", at(s, 'extern "C" fn callback'), at(s, 'extern "C" fn callback')),
            ("tests::works", at(s, "#[test]"), at(s, "} // end works")),
        ]

        found = [(item.symbol, item.first, item.last) for item in scope.items(s)]

        self.assertEqual(found, expected)


ITEM_LINE = re.compile(r'^\s*(pub(\([^)]*\))?\s+)?((const|async|unsafe|default)\s+)*(extern\s+"[^"]*"\s+)?'
                       r"(fn|struct|enum|union|trait|impl|type|const|static|macro_rules!)\b")
MACRO_CALL = re.compile(r"^(\s*)[\w:]+! \{$")


def macro_bodies(source):
    """The lines inside each macro invocation that opens a block, such as `thread_local! {`."""
    inside = set()
    for number, line in enumerate(source, 1):
        opened = MACRO_CALL.match(line)
        if opened:
            close = opened.group(1) + "}"
            end = next(n for n in range(number + 1, len(source) + 1) if source[n - 1].rstrip() == close)
            inside.update(range(number + 1, end))
    return inside


class CapturedFilesTest(unittest.TestCase):
    """Every Rust file of ai-ctl as the fixtures build it, against what rustfmt guarantees."""

    @classmethod
    def setUpClass(cls):
        cls.tmp = tempfile.TemporaryDirectory()
        cls.folder = Path(cls.tmp.name)
        fixtures.phase_close(cls.folder)
        cls.files = sorted(cls.folder.glob("**/*.rs"))

    @classmethod
    def tearDownClass(cls):
        cls.tmp.cleanup()

    def test_an_item_closed_by_a_brace_ends_on_the_first_brace_at_its_indentation(self):
        self.assertGreater(len(self.files), 50)
        checked = 0
        for path in self.files:
            text = path.read_text(encoding="utf-8")
            source = text.splitlines()
            for item in scope.items(text):
                checked += 1
                head = source[item.head - 1]
                close = source[item.last - 1].rstrip()
                indent = head[:len(head) - len(head.lstrip())]
                with self.subTest(file=str(path.relative_to(self.folder)), item=item.symbol):
                    if item.last > item.head and close.strip() in ("}", "};"):
                        self.assertIn(close, (indent + "}", indent + "};"))
                        between = source[item.head:item.last - 1]
                        self.assertNotIn(indent + "}", [line.rstrip() for line in between])
                    else:
                        self.assertTrue(close.endswith((";", "}")), close)
        # 2,004 lines open a function in these files.
        self.assertGreater(checked, 2000)

    def test_no_item_line_is_missed_outside_a_function(self):
        for path in self.files:
            text = path.read_text(encoding="utf-8")
            found = scope.items(text)
            heads = {item.head for item in found}
            bodies = [(item.head, item.last) for item in found if item.kind == "fn"]
            # A macro invocation holds no item: its lines are top level.
            heads |= macro_bodies(text.splitlines())
            for number, line in enumerate(text.splitlines(), 1):
                if ITEM_LINE.match(line) and number not in heads:
                    with self.subTest(file=str(path.relative_to(self.folder)), line=number):
                        self.assertTrue(any(head < number <= last for head, last in bodies), line)


class FixtureTest(unittest.TestCase):
    """The figures the evals' fixture history was measured at."""

    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.folder = Path(self.tmp.name)

    def tearDown(self):
        self.tmp.cleanup()

    def test_phase_1_of_gateway_releases(self):
        fixtures.phase_close(self.folder)

        done = run(self.folder, "roadmap", "docs/roadmap/on-progress/gateway-releases")

        self.assertEqual(done.returncode, 0, done.stderr)
        self.assertEqual(lines(done.stdout, "files"),
                         ["14: code 8, dependencies 0, documentation 1, captures 0, other 5"])
        self.assertEqual(lines(done.stdout, "hunks"), ["18 in 8 code files, +210 -6"])
        self.assertEqual(lines(done.stdout, "hunk"), [
            "src/application/fakes.rs:554-562 (FakeReleases::releases, FakeReleases::size, "
            "FakeReleases::remove) +9 -0",
            "src/application/gateway.rs:2 (top level) +1 -1",
            "src/application/gateway.rs:8-10 (top level) +3 -1",
            "src/application/gateway.rs:219-246 (PrunedReleases, prune) +28 -0",
            "src/application/gateway.rs:391-400 (tests::FakeStore::releases, tests::FakeStore::size, "
            "tests::FakeStore::remove) +10 -0",
            "src/application/gateway.rs:767-778 (tests::prune_removes_old_releases) +12 -0",
            "src/application/ports.rs:306-311 (GatewayStore::releases, GatewayStore::size, "
            "GatewayStore::remove) +6 -0",
            "src/domain/gateway.rs:132-153 (releases_to_prune, newest) +22 -0",
            "src/domain/gateway.rs:158-171 (tests::prune_keeps_the_newest_releases_and_the_current_one) +14 -0",
            "src/infrastructure/gateway_store.rs:175-218 (FsGatewayStore::releases, FsGatewayStore::size, "
            "FsGatewayStore::remove) +44 -0",
            "src/presentation/cli.rs:131-138 (GatewayAction) +8 -0",
            "src/presentation/gateway.rs:1-2 (top level) +2 -2",
            "src/presentation/gateway.rs:5 (top level) +1 -0",
            "src/presentation/gateway.rs:10-14 (top level) +5 -1",
            "src/presentation/gateway.rs:36-39 (run) +4 -0",
            "src/presentation/gateway.rs:80-97 (render_pruned) +18 -0",
            "src/presentation/gateway.rs:101-114 (top level, tests::prune_says_what_it_removed) +14 -1",
            "tests/cli.rs:248-256 (gateway_prune_needs_a_count) +9 -0",
        ])
        items = lines(done.stdout, "item")
        self.assertEqual(len(items), 23)
        self.assertIn("src/presentation/cli.rs:116-139 (GatewayAction)", items)
        self.assertIn("src/infrastructure/gateway_store.rs:176-198 (FsGatewayStore::releases)", items)
        self.assertIn("src/application/gateway.rs:228-245 (prune)", items)
        self.assertEqual(lines(done.stdout, "reading"),
                         ["233 lines: production 178, tests 55; 23 items, 11 top-level lines"])
        self.assertEqual(lines(done.stdout, "documentation"), ["README.md +1 -0"])

    def test_phase_2_of_gateway_releases(self):
        fixtures.roadmap_close(self.folder)
        report = self.folder / "docs/roadmap/on-progress/gateway-releases/phase-2-install-from-an-archive-report.md"
        start = re.search(r"^\*\*Start Commit:\*\* (\S+)$", report.read_text(), re.M).group(1)
        end = subprocess.run(["git", "log", "-1", "--format=%h", "--grep", "record phase 2's work"],
                             cwd=self.folder, check=True, capture_output=True, text=True).stdout.strip()

        done = run(self.folder, "range", start, end)

        self.assertEqual(done.returncode, 0, done.stderr)
        self.assertEqual(lines(done.stdout, "hunks"), ["29 in 7 code files, +213 -36"])
        self.assertIn("src/application/gateway.rs:142-149 (Installer::install) +8 -0", lines(done.stdout, "hunk"))

    def test_the_backends_patches_and_the_shipped_inventory_are_code(self):
        fixtures.phase_close(self.folder)
        changed = ["deploy/inventory.toml", "src/domain/patches/whisper.cpp/api-key.patch",
                   "tests/fixtures/llama-server-version.txt", "README.md"]
        for path in changed:
            with open(self.folder / path, "a", encoding="utf-8") as file:
                file.write("one more line\n")
        subprocess.run(["git", "commit", "-q", "-am", "session"], cwd=self.folder, check=True, env=GIT_ENV)

        done = run(self.folder, "range", "HEAD~1")

        self.assertEqual(done.returncode, 0, done.stderr)
        self.assertEqual(lines(done.stdout, "files"),
                         ["4: code 2, dependencies 0, documentation 1, captures 1, other 0"])
        self.assertEqual([hunk.split(":")[0] for hunk in lines(done.stdout, "hunk")], changed[:2])
        self.assertEqual(lines(done.stdout, "captures"), ["tests/fixtures/llama-server-version.txt +1 -0"])


class RepositoryTest(unittest.TestCase):
    def test_a_folder_outside_git_fails(self):
        with tempfile.TemporaryDirectory() as folder:
            done = run(folder, "range", "HEAD~1")

        self.assertEqual(done.returncode, 1)
        self.assertIn("is not a git repository", done.stderr)


if __name__ == "__main__":
    unittest.main()
