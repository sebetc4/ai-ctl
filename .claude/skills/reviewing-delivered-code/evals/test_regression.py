#!/usr/bin/env python3
"""Unit tests of scripts/regression.py, run on a throwaway crate and on libtest's captured outputs.

Usage: test_regression.py [unittest options]
"""

import json
import os
import signal
import subprocess
import sys
import tempfile
import time
import unittest
from pathlib import Path

HERE = Path(__file__).resolve().parent
SCRIPT = HERE.parent / "scripts" / "regression.py"
CAPTURES = HERE.parents[3] / "tests" / "fixtures"
sys.dont_write_bytecode = True
sys.path[:0] = [str(SCRIPT.parent), str(HERE)]

import regression  # noqa: E402
from test_scope import GIT_ENV, Repo  # noqa: E402

MUTANTS = "docs/reviews/delivered-code/mutants.json"
MANIFEST = '[package]\nname = "store"\nversion = "0.1.0"\nedition = "2024"\n'
LIB = """/// The newest release.
pub fn newest(releases: &[u32]) -> Option<u32> {
    releases.iter().copied().max()
}

/// The `count` newest releases, newest first.
pub fn keep(releases: &[u32], count: usize) -> Vec<u32> {
    let mut sorted = releases.to_vec();
    sorted.sort_unstable_by(|a, b| b.cmp(a));
    sorted.truncate(count);
    sorted
}

/// The oldest release.
pub fn oldest(releases: &[u32]) -> Option<u32> {
    releases.iter().copied().min()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_newest_is_the_highest() {
        assert_eq!(newest(&[99, 100, 263]), Some(263));
    }

    #[test]
    fn keep_keeps_some() {
        assert!(!keep(&[1, 2, 3], 2).is_empty());
    }

    #[test]
    fn the_oldest_is_the_lowest() {
        assert_eq!(oldest(&[99, 100, 263]), Some(100));
    }

    #[test]
    fn waits_for_ever() {
        loop {}
    }

    #[test]
    #[ignore = "needs a GPU"]
    fn needs_a_gpu() {
        assert_eq!(newest(&[1]), Some(1));
    }
}
"""
INTEGRATION = """#[test]
fn keeps_the_count() {
    assert_eq!(store::keep(&[1, 2, 3], 2), vec![3, 2]);
}
"""
NEWEST = "tests::the_newest_is_the_highest"


def mutant(number, text, replacement, test, file="src/lib.rs"):
    return {"id": f"M{number}", "kind": "a guard forced", "file": file, "text": text, "replacement": replacement,
            "test": test, "review": "2026-11-02-store-phase-1", "finding": number}


ALL = [
    mutant(1, "releases.iter().copied().max()", "releases.iter().copied().min()", NEWEST),
    mutant(2, "sorted.truncate(count);", "sorted.truncate(count + 1);", "keeps_the_count"),
    mutant(3, "    sorted.truncate(count);\n", "", "tests::keep_keeps_some"),
    mutant(4, "sorted.sort_unstable_by(|a, b| b.cmp(a));", "", "tests::keep_sorts_newest_first"),
    mutant(5, "releases.iter().rev()", "releases.iter()", NEWEST),
    mutant(6, "releases.iter().copied()", "releases.iter().copied().skip(1)", NEWEST),
    mutant(7, "releases.iter().copied().max()", "releases.iter().copied().maximum()", NEWEST),
    mutant(8, "releases.iter().copied().min()", "None", "tests::the_oldest_is_the_lowest"),
    mutant(9, "releases.iter().copied().max()", "loop {}", NEWEST),
    mutant(10, "pub fn keep", "fn keep", "keeps_the_count", file="src/gone.rs"),
    mutant(11, "sorted.sort_unstable_by(|a, b| b.cmp(a));", "sorted.sort_unstable();", "tests::waits_for_ever"),
    mutant(12, "releases.iter().copied().max()", "None", "tests::needs_a_gpu"),
]
TIMEOUT = 5
HANG = 600


def recorded(mutants):
    return json.dumps({"mutants": mutants}, indent=2) + "\n"


def run(folder, *args, env=None):
    """The script's run, failed past HANG seconds, a hang, its process group killed."""
    script = subprocess.Popen([str(SCRIPT), "--repo", str(folder), *map(str, args)], stdout=subprocess.PIPE,
                              stderr=subprocess.PIPE, text=True, env=env or GIT_ENV, start_new_session=True)
    try:
        out, err = script.communicate(timeout=HANG)
    except subprocess.TimeoutExpired:
        os.killpg(script.pid, signal.SIGKILL)
        script.communicate()
        raise AssertionError(f"regression.py still runs after {HANG} s") from None
    return subprocess.CompletedProcess(script.args, script.returncode, out, err)


def kill_left(folder):
    """Kills the processes a failed run left under `folder`, which no assertion would stop."""
    for pid in processes_in(folder):
        try:
            os.kill(pid, signal.SIGKILL)
        except ProcessLookupError:
            pass


class Crate:
    """A crate in a throwaway repository, its Cargo.lock written by cargo."""

    def __init__(self, folder):
        folder.mkdir()
        self.repo = Repo(folder)
        self.repo.write("Cargo.toml", MANIFEST)
        self.repo.write("src/lib.rs", LIB)
        self.repo.write("tests/keep.rs", INTEGRATION)
        self.repo.write(".gitignore", "/target/\n")
        subprocess.run(["cargo", "generate-lockfile", "--offline"], cwd=folder, check=True, capture_output=True)

    def commit(self, message, mutants=None, files=None):
        files = dict(files or {})
        if mutants is not None:
            files[MUTANTS] = recorded(mutants)
        return self.repo.commit(message, files)


class RunTest(unittest.TestCase):
    """One run of every mutant, of each outcome, with the copies in a folder of the test's own."""

    @classmethod
    def setUpClass(cls):
        cls.tmp = tempfile.TemporaryDirectory()
        root = Path(cls.tmp.name)
        cls.copies, cls.target = root / "copies", root / "target"
        cls.copies.mkdir()
        cls.crate = Crate(root / "repo")
        cls.head = cls.crate.commit("store", ALL)
        env = {**GIT_ENV, "TMPDIR": str(cls.copies), "CARGO_TARGET_DIR": str(cls.target)}
        cls.done = run(cls.crate.repo.folder, "--timeout", TIMEOUT, env=env)

    @classmethod
    def tearDownClass(cls):
        kill_left(cls.copies)
        cls.tmp.cleanup()

    def test_each_mutant_is_reported_with_its_outcome(self):
        self.assertEqual(self.done.returncode, 1, self.done.stderr)
        self.assertEqual(self.done.stdout.splitlines(), [
            "missed: M4 src/lib.rs: no test is named tests::keep_sorts_newest_first",
            "gone: M5 src/lib.rs: its text no longer occurs",
            "ambiguous: M6 src/lib.rs: its text occurs 2 times",
            "failing: M8 src/lib.rs: tests::the_oldest_is_the_lowest fails without the mutant",
            "gone: M10 src/gone.rs: its file no longer exists",
            f"failing: M11 src/lib.rs: tests::waits_for_ever runs past {TIMEOUT} s without the mutant",
            "missed: M12 src/lib.rs: tests::needs_a_gpu does not run: ignored, needs a GPU",
            "caught: M1 src/lib.rs: tests::the_newest_is_the_highest fails with the mutant",
            "caught: M2 src/lib.rs: keeps_the_count fails with the mutant",
            "missed: M3 src/lib.rs: tests::keep_keeps_some passes with the mutant",
            "unviable: M7 src/lib.rs: the mutant does not build",
            f"caught: M9 src/lib.rs: tests::the_newest_is_the_highest runs past {TIMEOUT} s with the mutant",
            f"summary: 12 mutants at {self.head}: 3 caught, 3 missed, 2 gone, 1 ambiguous, 1 unviable, 2 failing",
        ])

    def test_the_run_leaves_no_copy_no_process_and_the_repository_as_it_was(self):
        self.assertEqual(list(self.copies.iterdir()), [])
        self.assertFalse(self.target.exists())
        self.assertEqual(self.crate.repo.git("status", "--porcelain"), "")
        self.assertFalse((self.crate.repo.folder / "target").exists())
        self.assertEqual(processes_in(self.copies), [])


def processes_in(folder):
    """The processes whose working folder or command line is under `folder`."""
    found = []
    for proc in Path("/proc").iterdir():
        if not proc.name.isdigit():
            continue
        try:
            cwd = os.readlink(proc / "cwd")
            line = (proc / "cmdline").read_bytes().decode(errors="replace")
        except OSError:
            continue
        if cwd.startswith(str(folder)) or str(folder) in line:
            found.append(int(proc.name))
    return found


class CrateTest(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        root = Path(self.tmp.name)
        self.copies = root / "copies"
        self.copies.mkdir()
        self.crate = Crate(root / "repo")
        self.env = {**GIT_ENV, "TMPDIR": str(self.copies)}

    def tearDown(self):
        kill_left(self.copies)
        self.tmp.cleanup()

    def run_script(self, *args):
        return run(self.crate.repo.folder, *args, env=self.env)


class ChoiceTest(CrateTest):
    def test_the_mutants_named_run_alone(self):
        head = self.crate.commit("store", ALL)
        done = self.run_script("M3", "M1")
        self.assertEqual(done.returncode, 1, done.stderr)
        self.assertEqual(done.stdout.splitlines(), [
            "caught: M1 src/lib.rs: tests::the_newest_is_the_highest fails with the mutant",
            "missed: M3 src/lib.rs: tests::keep_keeps_some passes with the mutant",
            f"summary: 2 mutants at {head}: 1 caught, 1 missed, 0 gone, 0 ambiguous, 0 unviable, 0 failing",
        ])

    def test_the_mutants_are_those_of_the_commit_given(self):
        older = self.crate.commit("store", ALL[:2])
        self.crate.commit("more mutants", ALL)
        done = self.run_script("--at", older)
        self.assertEqual(done.returncode, 0, done.stderr)
        self.assertEqual(done.stdout.splitlines()[-1:],
                         [f"summary: 2 mutants at {older}: 2 caught, 0 missed, 0 gone, 0 ambiguous, 0 unviable, 0 failing"])

    def test_no_mutant_recorded_is_no_fault(self):
        head = self.crate.commit("store")
        done = self.run_script()
        self.assertEqual((done.returncode, done.stdout, done.stderr), (0, f"summary: no mutant recorded at {head}\n", ""))

    def test_a_mutant_not_recorded_is_refused_before_any_build(self):
        self.crate.commit("store", ALL[:2])
        done = self.run_script("M1", "M12", "M14")
        self.assertEqual(done.returncode, 3)
        self.assertEqual(done.stderr, "regression.py: M12 and M14 are not in mutants.json\n")
        done = self.run_script("M14")
        self.assertEqual((done.returncode, done.stderr), (3, "regression.py: M14 is not in mutants.json\n"))
        self.assertEqual(list(self.copies.iterdir()), [])

    def test_a_timeout_under_a_second_is_refused(self):
        self.crate.commit("store", ALL[:1])
        done = self.run_script("--timeout", 0)
        self.assertEqual(done.returncode, 2)
        self.assertIn("--timeout takes 1 second or more", done.stderr)
        self.assertEqual(list(self.copies.iterdir()), [])


class RefusalTest(CrateTest):
    def test_a_malformed_mutants_file_is_refused_with_each_field_named(self):
        broken = [dict(ALL[0], file="../outside.rs"), dict(ALL[1], text=""), dict(ALL[2], finding="3"),
                  dict(ALL[3], file="/etc/passwd"), {k: v for k, v in ALL[4].items() if k != "test"},
                  dict(ALL[5], id="M1"), dict(ALL[6], extra=1), dict(ALL[7], replacement=None),
                  dict(ALL[8], text="loop {}", replacement="loop {}"), dict(ALL[9], review=" "),
                  dict(ALL[10], finding=0), dict(ALL[11], kind=" ")]
        self.crate.commit("store", broken)
        done = self.run_script()
        self.assertEqual(done.returncode, 3)
        self.assertEqual(done.stderr.splitlines(), [
            "regression.py: mutants.json: M1: `file` is no relative path inside the repository",
            "regression.py: mutants.json: M2: `text` is no text",
            "regression.py: mutants.json: M3: `finding` is no number of 1 or more",
            "regression.py: mutants.json: M4: `file` is no relative path inside the repository",
            "regression.py: mutants.json: M5: `test` is missing",
            "regression.py: mutants.json: M7: `extra` is no key of it",
            "regression.py: mutants.json: M8: `replacement` is no text",
            "regression.py: mutants.json: M9: `replacement` is its `text`",
            "regression.py: mutants.json: M10: `review` is no text",
            "regression.py: mutants.json: M11: `finding` is no number of 1 or more",
            "regression.py: mutants.json: M12: `kind` is no text",
            "regression.py: mutants.json: M1 is listed twice",
        ])
        self.assertEqual(done.stdout, "")
        self.assertEqual(list(self.copies.iterdir()), [])

    def test_a_mutants_file_that_is_no_json_is_refused(self):
        self.crate.repo.commit("store", {MUTANTS: "{\"mutants\": [\n"})
        done = self.run_script()
        self.assertEqual(done.returncode, 3)
        self.assertTrue(done.stderr.startswith("regression.py: mutants.json: "), done.stderr)
        self.crate.repo.commit("a key misspelt", {MUTANTS: "{\"mutant\": []}\n"})
        done = self.run_script()
        self.assertEqual((done.returncode, done.stderr),
                         (3, "regression.py: mutants.json: it is no mapping of `mutants` to a list\n"))

    def test_a_link_out_of_the_copy_stops_the_run_before_any_write(self):
        (self.crate.repo.folder / "outside").symlink_to(Path(self.tmp.name) / "repo-outside")
        self.crate.commit("store", ALL[:1])
        done = self.run_script()
        self.assertEqual(done.returncode, 3)
        self.assertIn("regression.py: the copy of ", done.stderr)
        self.assertIn("cannot be made", done.stderr)
        self.assertFalse((Path(self.tmp.name) / "repo-outside").exists())
        self.assertEqual(list(self.copies.iterdir()), [])

    def test_a_copy_that_does_not_build_stops_the_run(self):
        self.crate.commit("store", ALL[:1], {"src/lib.rs": LIB.replace(".max()", ".maximum()", 1)})
        done = self.run_script()
        self.assertEqual(done.returncode, 3)
        self.assertEqual(done.stdout, "")
        self.assertIn("regression.py: the copy of ", done.stderr)
        self.assertIn("does not build", done.stderr)
        self.assertIn("maximum", done.stderr)
        self.assertEqual(list(self.copies.iterdir()), [])

    def test_a_commit_not_in_the_history_is_refused(self):
        self.crate.commit("store", ALL[:1])
        done = self.run_script("--at", "deadbeef")
        self.assertEqual(done.returncode, 3)
        self.assertEqual(done.stderr, "regression.py: deadbeef is not a commit of the repository\n")


class InterruptionTest(CrateTest):
    def test_an_interrupted_run_leaves_no_copy_and_no_process(self):
        self.crate.commit("store", [ALL[8]])
        script = subprocess.Popen([str(SCRIPT), "--repo", str(self.crate.repo.folder), "--timeout", "600"],
                                  stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True, env=self.env)
        try:
            deadline = time.monotonic() + 300
            while not hung(self.copies):
                self.assertIsNone(script.poll(), script.stderr.read() if script.poll() is not None else "")
                self.assertLess(time.monotonic(), deadline, "the hung test never started")
                time.sleep(0.1)
            script.send_signal(signal.SIGINT)
            script.communicate(timeout=60)
        finally:
            if script.poll() is None:
                script.kill()
                script.communicate()
        self.assertEqual(script.returncode, 130)
        self.assertEqual(list(self.copies.iterdir()), [])
        self.assertEqual(processes_in(self.copies), [])


def hung(copies):
    """Whether the mutant is applied in the copy and its test is running, hung."""
    copy = next(copies.iterdir(), None)
    try:
        applied = copy is not None and "loop {}" in (copy / "src" / "lib.rs").read_text(encoding="utf-8")
    except OSError:
        return False
    return applied and any("/deps/store-" in cmdline(pid) for pid in processes_in(copies))


def cmdline(pid):
    try:
        return Path(f"/proc/{pid}/cmdline").read_bytes().decode(errors="replace")
    except OSError:
        return ""


class ParseTest(unittest.TestCase):
    def results(self, capture):
        return regression.results((CAPTURES / capture).read_text(encoding="utf-8"))

    def test_each_test_run_is_read_with_its_status_over_the_test_binaries(self):
        self.assertEqual(self.results("cargo-test-exact-one.txt"),
                         {"domain::gateway::tests::a_release_is_a_number_with_or_without_its_v": "ok"})
        self.assertEqual(self.results("cargo-test-exact-none.txt"), {})

    def test_a_harness_of_its_own_is_read_by_its_test_line(self):
        self.assertEqual(self.results("cargo-test-exact-own-harness.txt"), {"up_a_proxied_request_and_down": "ok"})


if __name__ == "__main__":
    unittest.main()
