#!/usr/bin/env python3
"""Unit tests of evals/fixtures.py against the cases of evals/evals.json.

Usage: test_fixtures.py [unittest options]

Each fixture case is built once, into a temporary folder, from the files of evals/fixtures/.
"""

import json
import re
import shutil
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.dont_write_bytecode = True
sys.path.insert(0, str(HERE))

import fixtures  # noqa: E402

BUILT = {}
SCRATCH = tempfile.TemporaryDirectory()
AUTHOR = {**fixtures.GIT_ENV, "GIT_AUTHOR_NAME": "test", "GIT_AUTHOR_EMAIL": "test@example.com",
          "GIT_COMMITTER_NAME": "test", "GIT_COMMITTER_EMAIL": "test@example.com"}


def built(case):
    """The folder `case` builds, built on first use."""
    if case not in BUILT:
        folder = Path(SCRATCH.name) / case
        folder.mkdir()
        fixtures.CASES[case](folder)
        BUILT[case] = folder
    return BUILT[case]


def git(folder, *args):
    return subprocess.run(["git", *args], cwd=folder, check=True, capture_output=True,
                          text=True).stdout.strip()


def tracked_text(folder):
    """Each tracked file's text, by path, for the files that are text."""
    texts = {}
    for path in git(folder, "ls-files").splitlines():
        try:
            texts[path] = (folder / path).read_text(encoding="utf-8")
        except (UnicodeDecodeError, IsADirectoryError):
            continue
    return texts


def start_commit(folder, report):
    text = (folder / "docs/roadmap/on-progress/gateway-releases" / report).read_text()
    return re.search(r"^\*\*Start Commit:\*\* (\S+)$", text, re.M).group(1)


def mutants(folder):
    return json.loads((folder / "docs/reviews/delivered-code/mutants.json").read_text())["mutants"]


class FixtureCases(unittest.TestCase):
    def test_every_fixture_case_has_a_builder(self):
        cases = json.loads((HERE / "evals.json").read_text(encoding="utf-8"))["evals"]
        names = [case["name"] for case in cases if case["setup"] == "fixture"]
        self.assertEqual([name for name in names if name not in fixtures.CASES], [])


class TheBase(unittest.TestCase):
    """Every case starts from ai-ctl before the review skill's change, in one commit."""

    def test_no_case_holds_the_review_skill_or_its_answer_keys(self):
        for case in fixtures.CASES:
            folder = built(case)
            with self.subTest(case=case):
                self.assertFalse((folder / ".claude").exists())
                self.assertFalse((folder / "docs/roadmap/completed/review-skill").exists())
                self.assertEqual(git(folder, "log", "--all", "--format=%h", "--", ".claude"), "")
                self.assertNotIn("reviewing-delivered-code", (folder / "CLAUDE.md").read_text())

    def test_the_base_is_one_commit_on_main(self):
        folder = built("dependency-check")
        self.assertEqual(git(folder, "rev-list", "--count", "main"), "1")

    def test_every_case_tracks_the_policy_of_cargo_deny_from_its_base(self):
        for case in fixtures.CASES:
            folder = built(case)
            with self.subTest(case=case):
                root = git(folder, "rev-list", "--max-parents=0", "HEAD")
                self.assertIn("deny.toml", git(folder, "ls-tree", "--name-only", root).splitlines())
                self.assertEqual(git(folder, "show", f"{root}:deny.toml") + "\n",
                                 (fixtures.FIXTURES / "deny.toml").read_text(encoding="utf-8"))

    def test_the_conventions_file_is_there_and_ignored(self):
        folder = built("phase-close")
        self.assertTrue((folder / ".agent-conventions.toml").is_file())
        self.assertEqual(git(folder, "check-ignore", ".agent-conventions.toml"),
                         ".agent-conventions.toml")

    def test_a_case_is_built_the_same_twice(self):
        again = Path(SCRATCH.name) / "phase-close-again"
        again.mkdir()
        fixtures.CASES["phase-close"](again)
        self.assertEqual(git(again, "rev-parse", "HEAD"), git(built("phase-close"), "rev-parse", "HEAD"))

    def test_a_case_is_built_in_a_repository_whose_history_lacks_the_base(self):
        scratch = Path(SCRATCH.name) / "elsewhere"
        copy = scratch / "evals"
        shutil.copytree(HERE, copy, ignore=shutil.ignore_patterns("__pycache__"))
        fixtures.git(scratch, "init", "-q", "-b", "main")
        fixtures.git(scratch, "add", "-A")
        fixtures.git(scratch, "commit", "-q", "-m", "root", env=AUTHOR)
        folder = scratch / "case"
        folder.mkdir()
        done = subprocess.run([sys.executable, str(copy / "fixtures.py"), "phase-close", str(folder)],
                              capture_output=True, text=True)
        self.assertEqual((done.returncode, done.stderr), (0, ""))
        self.assertEqual(git(folder, "rev-parse", "HEAD"), git(built("phase-close"), "rev-parse", "HEAD"))


class ThePostSessionCommit(unittest.TestCase):
    def test_the_last_commit_adds_gateway_prune_on_the_base(self):
        folder = built("cost-gate")
        self.assertEqual(git(folder, "log", "-1", "--format=%s"),
                         "feat(gateway): remove old llama-swap releases with gateway prune")
        self.assertEqual(git(folder, "rev-parse", "HEAD~1"),
                         git(folder, "rev-list", "--max-parents=0", "HEAD"))
        self.assertIn("Prune {", (folder / "src/presentation/cli.rs").read_text())


class TheRoadmap(unittest.TestCase):
    def test_the_phase_to_close_starts_at_the_base_on_its_branch(self):
        folder = built("phase-close")
        self.assertEqual(git(folder, "branch", "--show-current"), "roadmap/gateway-releases")
        self.assertEqual(git(folder, "rev-list", "--count", "main..HEAD"), "4")
        self.assertEqual(start_commit(folder, "phase-1-prune-report.md"),
                         git(folder, "rev-parse", "--short=7", "main"))
        self.assertEqual(git(folder, "status", "--porcelain"), "")

    def test_no_placeholder_is_left(self):
        for case in ("phase-close", "continued-review", "roadmap-close"):
            with self.subTest(case=case):
                for path, text in tracked_text(built(case)).items():
                    self.assertNotIn("{{commit:", text, path)

    def test_the_review_to_continue_waits_for_its_verification(self):
        folder = built("continued-review")
        self.assertEqual(git(folder, "rev-parse", "HEAD"), git(built("phase-close"), "rev-parse", "HEAD"))
        review = folder / "docs/reviews/delivered-code/2026-10-31-gateway-releases-phase-1"
        findings = json.loads((review / "findings.json").read_text())
        self.assertEqual(findings["status"]["stage"], "verification")
        self.assertTrue((review / "report.md").is_file())
        self.assertEqual(git(folder, "status", "--porcelain"),
                         "?? docs/reviews/delivered-code/2026-10-31-gateway-releases-phase-1/")

    def test_the_roadmap_to_close_has_its_last_phase_reviewed(self):
        folder = built("roadmap-close")
        self.assertEqual(git(folder, "status", "--porcelain"), "")
        opening = git(folder, "log", "-1", "--format=%H", "--grep", "open phase 2")
        self.assertEqual(start_commit(folder, "phase-2-install-from-an-archive-report.md"),
                         git(folder, "rev-parse", "--short=7", f"{opening}~1"))
        self.assertFalse(list((folder / "docs/reviews/delivered-code").glob("*/repro")))
        register = json.loads((folder / "docs/reviews/delivered-code/register.json").read_text())
        self.assertIn("rejected", {entry["state"] for entry in register["entries"]})

    def test_the_recorded_mutants_hold_one_of_each_outcome(self):
        folder = built("roadmap-close")
        counts = {}
        for mutant in mutants(folder):
            counts[mutant["id"]] = (folder / mutant["file"]).read_text().count(mutant["text"])
        self.assertEqual(counts, {"M1": 1, "M2": 1, "M3": 0})
        tests = git(folder, "grep", "-h", "-o", r"fn [a-z_]*()", "--", "src").replace("()", "")
        names = {mutant["id"]: mutant["test"].rsplit("::", 1)[1] for mutant in mutants(folder)}
        self.assertIn(f"fn {names['M1']}", tests)
        self.assertNotIn(f"fn {names['M2']}", tests)


class TheLockFile(unittest.TestCase):
    def test_two_crates_are_behind_one_under_an_advisory(self):
        folder = built("dependency-check")
        lock = (folder / "Cargo.lock").read_text()
        self.assertIn('name = "anyhow"\nversion = "1.0.102"', lock)
        self.assertIn('name = "clap_complete"\nversion = "4.6.2"', lock)
        self.assertEqual(git(folder, "status", "--porcelain"), "")
        manifest = (folder / "Cargo.toml").read_text()
        self.assertIn('anyhow = "1.0"', manifest)
        self.assertIn('clap_complete = "4.6"', manifest)


if __name__ == "__main__":
    try:
        unittest.main()
    finally:
        SCRATCH.cleanup()
