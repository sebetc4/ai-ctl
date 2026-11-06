#!/usr/bin/env python3
"""Unit tests of scripts/findings.py, run on throwaway git repositories and on the fixtures' reviews.

Usage: test_findings.py [unittest options]
"""

import json
import shutil
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

HERE = Path(__file__).resolve().parent
SCRIPT = HERE.parent / "scripts" / "findings.py"
TEMPLATE = HERE.parent / "assets" / "report.md"
sys.dont_write_bytecode = True
sys.path[:0] = [str(SCRIPT.parent), str(HERE)]

import findings  # noqa: E402
import fixtures  # noqa: E402
import scope  # noqa: E402
from test_scope import GIT_ENV, Repo  # noqa: E402

REVIEW = "docs/reviews/delivered-code/2026-11-02-store-phase-1"
BASE_STORE = ("pub struct Store {\n    root: String,\n}\n\nimpl Store {\n"
              "    pub fn lock(&self) -> String {\n        format!(\"installing {}\", self.root)\n    }\n}\n")
END_STORE = ("pub struct Store {\n    root: String,\n}\n\nimpl Store {\n"
             "    pub fn lock(&self) -> String {\n        format!(\"installing {}\", self.root)\n    }\n\n"
             "    pub fn releases(&self) -> Vec<String> {\n        let mut names = vec![self.root.clone()];\n"
             "        names.sort();\n        names\n    }\n\n"
             "    pub fn remove(&self, name: &str) {\n"
             "        let _ = std::process::Command::new(\"sh\").arg(\"-c\").arg(format!(\"rm -rf {name}\")).status();\n"
             "    }\n}\n")
COMMANDS = "# Commands\n\n| Command | What |\n|---|---|\n| up | starts the stack |\n"
CLI = "pub enum Action {\n    Prune,\n}\n"
AGENTS = ("review-security", "review-correctness", "review-tests", "review-cleanup", "review-structure",
          "review-performance", "review-verifier")

ANSWERS = {
    "review-correctness": """FINDINGS: 2

FINDING
axis: bugs
severity: medium
where: src/store.rs:12 (Store::releases)
title: the releases are sorted as text
cause: releases sorts the names as strings
effect: wrong action
consequence: v100 comes before v99, so prune removes v100
fix: releases sorts by number
test: releases_come_by_number
evidence: read: the sort

FINDING
axis: error handling
severity: low
where: src/store.rs:7 (Store::lock)
title: the lock says installing whoever holds it
cause: lock writes the activity `installing`
effect: misleading
consequence: an install during a prune says another install runs
fix: the holder's activity passed to lock
test: none: a message
evidence: read: lock

FILES READ
in full: src/store.rs
in part: src/cli.rs (lines 1-3)

LEFT OUT
docs/: no claim to check
""",
    "review-security": """Two paths followed; one finding.

FINDINGS: 1

```
FINDING
axis: security, bugs
severity: high
where: src/store.rs:17 (Store::remove)
title: a path with a space makes remove delete other directories
cause: remove runs sh -c with the path unquoted
effect: data lost
consequence: with the root at `/home/me/My Data`, rm -rf removes /home/me/My
fix: the directory removed with remove_dir_all
test: a_removal_under_a_path_with_a_space_removes_the_release_alone
evidence: reproduced: in a copy, the directory beside removed
```

PROBE
command: ln -s /tmp/target /tmp/root/v1
risk: removes /tmp/target if remove follows the link
settles: whether remove follows a symbolic link
finding: a path with a space makes remove delete other directories

FILES READ
in full: src/store.rs

LEFT OUT
none
""",
    "review-tests": """FINDINGS: 1

FINDING
axis: tests
severity: medium
where: src/store.rs:10 (Store::releases), src/store.rs:6 (Store::lock)
title: no test orders releases of two and three digits
cause: releases has no test
effect: wrong action
consequence: the text order passes every test
fix: a test on v99, v100 and v263
test: releases_come_by_number
evidence: read: the tests

FILES READ
in full: src/store.rs
""",
    "review-cleanup": """FINDINGS: 1

FINDING
axis: refactoring
where: src/store.rs:6 (Store::lock), src/store.rs:16 (Store::remove)
title: lock and remove each format the root
cause: two formats of one path
effect: maintenance
consequence: a change to the path is made twice
fix: one helper
test: none: no behaviour changes
evidence: read: both bodies

FILES READ
in full: src/store.rs
""",
    "review-structure": """FINDINGS: 1

FINDING
axis: the CLI's contract
severity: low
where: src/cli.rs:2 (Action::Prune), docs/commands.md:5 (command table)
title: prune is missing from docs/commands.md
cause: the command was added without its row
effect: misleading
consequence: the reference lists no prune
fix: the row in docs/commands.md
test: none: a document
evidence: read: docs/commands.md

FILES READ
in full: src/cli.rs, docs/commands.md
""",
    "review-performance": """FINDINGS: 1

FINDING
axis: performance
where: src/store.rs:11 (Store::releases)
title: releases clones the root
cause: a clone where a borrow would do
effect: slow
consequence: one allocation per call,
  too small to notice
fix: a borrow
test: none: a test depending on timing is forbidden
evidence: read: the line

FILES READ
in part: src/store.rs (lines 10-14)
""",
}
ORDER = ("review-correctness", "review-security", "review-tests", "review-cleanup", "review-structure",
         "review-performance")
ONE = ANSWERS["review-tests"].replace("src/store.rs:10 (Store::releases), src/store.rs:6 (Store::lock)",
                                      "src/store.rs:12 (Store::releases)")
# The phase's store two lines down, under a module comment: every method's lines move.
MOVED_STORE = "//! The releases on disk.\n\n" + END_STORE
LATER = """FINDINGS: 4

FINDING
axis: security
severity: high
where: src/store.rs:19 (Store::remove), src/store.rs:13 (Store::releases)
title: remove runs a name releases lists through a shell
cause: remove passes the name to sh -c unquoted, and releases lists the names on disk
effect: data lost
consequence: a release named `x; rm -rf ~` removes the home directory
fix: the directory removed with remove_dir_all
test: a_release_named_with_a_semicolon_is_removed_alone
evidence: read: remove and releases

FINDING
axis: bugs
severity: medium
where: src/store.rs:19 (Store::remove)
title: remove reports nothing when rm fails
cause: remove drops the command's status
effect: misleading
consequence: a failed removal is counted as freed
fix: the status checked
test: a_failed_removal_is_reported
evidence: read: remove

FINDING
axis: security
severity: low
where: src/store.rs:13 (Store::releases)
title: releases lists entries another account made
cause: releases reads the root's entries with no owner check
effect: wrong action
consequence: a release planted by another account is listed
fix: the entries of another account skipped
test: a_release_of_another_account_is_skipped
evidence: read: releases

FINDING
axis: error handling
severity: low
where: src/store.rs:9 (Store::lock)
title: lock names the wrong activity
cause: lock writes `installing` for every holder
effect: misleading
consequence: a prune reads as an install
fix: the holder's activity passed to lock
test: none: a message
evidence: read: lock

FILES READ
in full: src/store.rs
"""


def run(*args):
    return subprocess.run([str(SCRIPT), *map(str, args)], capture_output=True, text=True, env=GIT_ENV)


class ReviewTest(unittest.TestCase):
    """A repository whose phase adds two methods and a command, and its review just started."""

    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.repo = Repo(self.tmp.name)
        self.base = self.repo.commit("base", {"src/store.rs": BASE_STORE, "docs/commands.md": COMMANDS})
        self.end = self.repo.commit("phase 1", {"src/store.rs": END_STORE, "src/cli.rs": CLI})
        self.folder = Path(self.tmp.name) / REVIEW
        agents = [arg for agent in AGENTS for arg in ("--agent", f"{agent}:medium")]
        done = run("new", self.folder, "--base", self.base, "--roadmap", "store", "--phase", "1",
                   "--by", "the request", "--text", "fais la review", "--tier", "medium", *agents)
        self.assertEqual(done.returncode, 0, done.stderr)
        self.assertTrue((self.folder / "findings.json").is_file())

    def tearDown(self):
        self.tmp.cleanup()

    def review(self):
        return json.loads((self.folder / "findings.json").read_text(encoding="utf-8"))

    def report(self):
        return (self.folder / "report.md").read_text(encoding="utf-8")

    def add(self, agent, text=None):
        answer = Path(self.tmp.name) / f"{agent}.txt"
        answer.write_text(ANSWERS[agent] if text is None else text, encoding="utf-8")
        return run("add", self.folder, agent, answer)

    def add_all(self):
        for agent in ORDER:
            done = self.add(agent)
            self.assertEqual(done.returncode, 0, done.stderr)

    def files(self):
        return [(self.folder / name).read_bytes() for name in ("findings.json", "report.md")]


class NewTest(ReviewTest):
    def test_the_review_starts_with_its_scope_by_hunk_and_its_agents(self):
        review = self.review()
        self.assertEqual(review["review"], "2026-11-02-store-phase-1")
        self.assertEqual(review["scope"], {"kind": "phase", "roadmap": "store", "phase": 1, "base": self.base,
                                           "end": self.end, "hunks": 2, "lines": {"added": 13, "removed": 0}})
        self.assertEqual(review["go_ahead"], {"by": "the request", "text": "fais la review", "tier": "medium"})
        self.assertEqual([(a["type"], a["effort"], a["state"]) for a in review["agents"]],
                         [(agent, "medium", "not started") for agent in AGENTS])
        self.assertEqual((review["checks"], review["mutants"], review["findings"]), ([], None, []))
        self.assertEqual(review["status"]["stage"], "reviewing")
        self.assertIn("## Summary\n\nNo finding.\n", self.report())
        self.assertNotIn("## Findings", self.report())

    def test_a_review_already_started_is_not_started_again(self):
        before = self.files()
        done = run("new", self.folder, "--base", self.base, "--by", "the request", "--text", "again",
                   "--tier", "medium", "--agent", "review-tests:medium")
        self.assertEqual(done.returncode, 1)
        self.assertIn("findings.json", done.stderr)
        self.assertEqual(self.files(), before)


class AnswerTest(ReviewTest):
    def test_an_answer_from_each_reviewer_is_numbered_with_its_origin_floor_and_fingerprint(self):
        self.add_all()
        review = self.review()
        got = [(f["number"], f["reviewers"], f["axes"], f["origin"], f["floor"], f["fingerprint"])
               for f in review["findings"]]
        self.assertEqual(got, [
            (1, ["review-correctness"], ["bugs"], "new", "above", "bugs | src/store.rs | Store::releases"),
            (2, ["review-correctness"], ["error handling"], "older", "above",
             "error handling | src/store.rs | Store::lock"),
            (3, ["review-security"], ["security", "bugs"], "new", "above", "security | src/store.rs | Store::remove"),
            (4, ["review-tests"], ["tests"], "new", "above", "tests | src/store.rs | Store::lock + Store::releases"),
            (5, ["review-cleanup"], ["refactoring"], "older", "below",
             "refactoring | src/store.rs | Store::lock + Store::remove"),
            (6, ["review-structure"], ["the CLI's contract"], "new", "above",
             "the CLI's contract | docs/commands.md + src/cli.rs | Action::Prune + command table"),
            (7, ["review-performance"], ["performance"], "new", "below", "performance | src/store.rs | Store::releases"),
        ])
        first = review["findings"][0]
        self.assertEqual({key: first[key] for key in ("where", "severity", "test", "verdict", "action", "fixed_in")},
                         {"where": ["src/store.rs:12 (Store::releases)"], "severity": "medium",
                          "test": "releases_come_by_number", "verdict": None, "action": None, "fixed_in": None})
        self.assertEqual(review["findings"][6]["consequence"], "one allocation per call, too small to notice")
        agents = {agent["type"]: agent for agent in review["agents"]}
        self.assertEqual(agents["review-correctness"]["state"], "answered")
        self.assertEqual(agents["review-correctness"]["read"],
                         {"in full": ["src/store.rs"], "in part": ["src/cli.rs (lines 1-3)"]})
        self.assertEqual(agents["review-correctness"]["left_out"], ["docs/: no claim to check"])
        self.assertEqual(agents["review-security"]["probes"], [
            {"command": "ln -s /tmp/target /tmp/root/v1", "risk": "removes /tmp/target if remove follows the link",
             "settles": "whether remove follows a symbolic link", "finding": 3}])
        self.assertEqual(agents["review-verifier"]["state"], "not started")
        report = self.report()
        self.assertIn("| 1 | the releases are sorted as text | bugs | wrong action | medium | new | to verify | — | — |",
                      report)
        self.assertIn("| 5 | lock and remove each format the root | refactoring | maintenance | — | older | — | — | — |",
                      report)
        self.assertIn("- `review-correctness`: in full: src/store.rs; in part: src/cli.rs (lines 1-3); "
                      "left out: docs/: no claim to check\n", report)
        self.assertIn("- `review-tests`: in full: src/store.rs; in part: none; left out: none\n", report)
        self.assertIn("- Finding 3, from `review-security`: `ln -s /tmp/target /tmp/root/v1`. Risk: removes "
                      "/tmp/target if remove follows the link. Settles: whether remove follows a symbolic link.\n",
                      report)

    def test_a_malformed_answer_is_refused_with_its_field_named_and_nothing_written(self):
        cases = [
            ("effect: wrong action\n", "", "`effect`"),
            ("effect: wrong action", "effect: cosmetic", "`effect`"),
            ("evidence: read: the tests\n", "evidence: read: the tests\nrule: CLAUDE.md:12\n", "`rule`"),
            ("evidence: read: the tests\n", "evidence: read: the tests\ntitle: again\n", "`title`"),
            ("severity: medium\n", "", "`severity`"),
            ("effect: wrong action", "effect: maintenance", "`severity`"),
            ("test: releases_come_by_number", "test: none", "`test`"),
            ("axis: tests", "axis: test", "`axis`"),
            ("cause: releases has no test", "cause: ", "`cause`"),
            ("cause: releases has no test\n", "cause: releases has no test\nnot even one\n", "no field"),
            ("src/store.rs:12 (Store::releases)", "src/store.rs:12", "`where`"),
            ("src/store.rs:12 (Store::releases)", "src/store.rs:120 (Store::releases)", "`where`"),
            ("src/store.rs:12 (Store::releases)", "src/gone.rs:1 (gone)", "`where`"),
            ("FINDINGS: 1", "FINDINGS: 2", "FINDINGS"),
            ("FINDINGS: 1\n", "", "FINDINGS"),
            ("FILES READ\nin full: src/store.rs\n", "", "FILES READ"),
        ]
        before = self.files()
        for old, new, named in cases:
            with self.subTest(old=old, new=new):
                self.assertEqual(ONE.count(old), 1)
                done = self.add("review-tests", ONE.replace(old, new))
                self.assertEqual(done.returncode, 1, done.stdout)
                self.assertIn(named, done.stderr)
                self.assertEqual(self.files(), before)

    def test_an_answer_naming_a_path_no_other_clone_can_open_is_refused_and_nothing_written(self):
        session = "/tmp/claude-1000/-code-x/00000000-0000-0000-0000-000000000000"
        cases = [
            ("in full: src/store.rs\n", "in full: src/store.rs, /tmp/tmp.aB3dE5gH7j/scope.txt\n",
             "/tmp/tmp.aB3dE5gH7j/scope.txt"),
            ("in full: src/store.rs\n", f"in full: {session}/scratchpad/tmp.kL9mN0pQ1r/deny.txt\n",
             f"{session}/scratchpad/tmp.kL9mN0pQ1r/deny.txt"),
            ("in full: src/store.rs\n", f"in full: {session}/tasks/b1.output\n", f"{session}/tasks/b1.output"),
            ("in full: src/store.rs\n", "in full: `/run/user/1000/agent/scratchpad/deny.txt`\n",
             "/run/user/1000/agent/scratchpad/deny.txt"),
            ("evidence: read: the tests\n", "evidence: reproduced: in /tmp/tmp.sT2uV4wX6y/review-tests, red\n",
             "/tmp/tmp.sT2uV4wX6y/review-tests"),
            ("evidence: read: the tests\n", "evidence: read: /home/u/.claude/projects/-code-x/1.jsonl:4\n",
             "/home/u/.claude/projects/-code-x/1.jsonl:4"),
        ]
        before = self.files()
        for old, new, named in cases:
            with self.subTest(new=new):
                self.assertEqual(ONE.count(old), 1)
                done = self.add("review-tests", ONE.replace(old, new))
                self.assertEqual(done.returncode, 1, done.stdout)
                self.assertIn(f"`{named}`", done.stderr)
                self.assertEqual(self.files(), before)

    def test_a_scratch_file_named_by_its_place_in_scratch_and_a_temporary_path_of_the_code_are_kept(self):
        evidence = ("evidence: reproduced: review-tests/tests.diff, in the session's scratchpad, red on "
                    "hf download --cache-dir=/tmp/x and ${TMPDIR:-/tmp}/tmp.*\n")
        done = self.add("review-tests", ONE.replace("evidence: read: the tests\n", evidence))
        self.assertEqual(done.returncode, 0, done.stderr)
        self.assertIn("--cache-dir=/tmp/x", self.review()["findings"][0]["evidence"])

    def test_an_answer_with_no_finding_is_counted(self):
        done = self.add("review-tests", "FINDINGS: 0\n\nFILES READ\nin full: src/store.rs\n")
        self.assertEqual(done.returncode, 0, done.stderr)
        review = self.review()
        self.assertEqual(review["findings"], [])
        self.assertEqual(next(a for a in review["agents"] if a["type"] == "review-tests")["state"], "answered")

    def test_an_agent_answers_once_and_only_an_agent_of_the_review(self):
        self.assertEqual(self.add("review-tests").returncode, 0)
        before = self.files()
        again = self.add("review-tests")
        self.assertEqual(again.returncode, 1)
        self.assertIn("answered", again.stderr)
        stranger = run("add", self.folder, "review-dependencies", Path(self.tmp.name) / "review-tests.txt")
        self.assertEqual(stranger.returncode, 1)
        self.assertIn("review-dependencies", stranger.stderr)
        self.assertEqual(self.files(), before)

    def test_each_run_of_an_agent_listed_twice_answers_once(self):
        folder = Path(self.tmp.name) / "2026-11-02-store-high"
        done = run("new", folder, "--base", self.base, "--by", "the request", "--text", "high", "--tier", "high",
                   "--agent", "review-tests:high", "--agent", "review-verifier:high", "--agent", "review-verifier:high")
        self.assertEqual(done.returncode, 0, done.stderr)
        answer = Path(self.tmp.name) / "verifier.txt"
        answer.write_text(ONE, encoding="utf-8")
        self.assertEqual(run("add", folder, "review-verifier", answer).returncode, 0)
        review = json.loads((folder / "findings.json").read_text(encoding="utf-8"))
        self.assertEqual([a["state"] for a in review["agents"]], ["not started", "answered", "not started"])
        self.assertEqual(run("add", folder, "review-verifier", answer).returncode, 0)
        third = run("add", folder, "review-verifier", answer)
        self.assertEqual(third.returncode, 1)
        self.assertIn("answered", third.stderr)
        review = json.loads((folder / "findings.json").read_text(encoding="utf-8"))
        self.assertEqual([a["state"] for a in review["agents"]], ["not started", "answered", "answered"])
        self.assertEqual([f["number"] for f in review["findings"]], [1, 2])

    def test_the_session_answers_again_for_a_defect_it_sees_after_its_answer(self):
        folder = Path(self.tmp.name) / "2026-11-02-store-session"
        done = run("new", folder, "--base", self.base, "--by", "the request", "--text", "high", "--tier", "high",
                   "--agent", "review-tests:high", "--agent", "session:high")
        self.assertEqual(done.returncode, 0, done.stderr)
        none, later = Path(self.tmp.name) / "none.txt", Path(self.tmp.name) / "later.txt"
        none.write_text("FINDINGS: 0\n\nFILES READ\nin full: src/store.rs\n", encoding="utf-8")
        later.write_text(ONE, encoding="utf-8")
        self.assertEqual(run("add", folder, "session", none).returncode, 0)
        done = run("add", folder, "session", later)
        self.assertEqual(done.returncode, 0, done.stderr)
        review = json.loads((folder / "findings.json").read_text(encoding="utf-8"))
        self.assertEqual([(a["type"], a["effort"], a["state"]) for a in review["agents"]],
                         [("review-tests", "high", "not started"), ("session", "high", "answered"),
                          ("session", "high", "answered")])
        self.assertEqual([(f["number"], f["reviewers"]) for f in review["findings"]], [(1, ["session"])])

    def test_a_verifiers_leads_are_its_findings_and_its_verdicts_are_left_to_the_session(self):
        text = ONE.replace("FINDINGS: 1\n", "FINDINGS: 1\n\nVERDICT 4\nverdict: CONFIRMED\n"
                                            "evidence: reproduced: the red test in my copy\n"
                                            "repro: repro/4/test.patch, repro/4/fix.patch\n")
        self.assertIn("VERDICT 4", text)
        done = self.add("review-verifier", text)
        self.assertEqual(done.returncode, 0, done.stderr)
        review = self.review()
        self.assertEqual([(f["number"], f["reviewers"], f["verdict"]) for f in review["findings"]],
                         [(1, ["review-verifier"], None)])

    def test_the_facts_of_the_dependency_axis_are_kept_and_reported_after_its_findings(self):
        folder = Path(self.tmp.name) / "2026-11-02-crates"
        done = run("new", folder, "--base", self.end, "--by", "the request", "--text", "nos crates", "--tier", "medium",
                   "--agent", "review-dependencies:medium")
        self.assertEqual(done.returncode, 0, done.stderr)
        answer = Path(self.tmp.name) / "dependencies.txt"
        answer.write_text("FINDINGS: 0\n\nFILES READ\nin full: Cargo.lock\n\nLEFT OUT\nnone\n\nFACTS\n"
                          "thiserror 1.0.69 beside 2.0.21, through nvml-wrapper\n"
                          "llama-swap v263 pinned, v269 the newest\n", encoding="utf-8")
        done = run("add", folder, "review-dependencies", answer)
        self.assertEqual(done.returncode, 0, done.stderr)
        agent = json.loads((folder / "findings.json").read_text(encoding="utf-8"))["agents"][0]
        self.assertEqual(agent.get("facts"), ["thiserror 1.0.69 beside 2.0.21, through nvml-wrapper",
                                          "llama-swap v263 pinned, v269 the newest"])
        report = (folder / "report.md").read_text(encoding="utf-8")
        self.assertIn("## Facts\n\n- `review-dependencies`: thiserror 1.0.69 beside 2.0.21, through nvml-wrapper\n"
                      "- `review-dependencies`: llama-swap v263 pinned, v269 the newest\n", report)
        self.assertIn("- `review-dependencies`: in full: Cargo.lock; in part: none; left out: none", report)
        self.assertNotIn("## Facts", self.report())

    def test_a_probe_names_a_finding_of_its_answer(self):
        text = ANSWERS["review-security"].replace("finding: a path with a space", "finding: a path with a tab")
        done = self.add("review-security", text)
        self.assertEqual(done.returncode, 1)
        self.assertIn("`finding`", done.stderr)

    def test_the_origin_is_the_first_places(self):
        reversed_places = ONE.replace("src/store.rs:12 (Store::releases)",
                                      "src/store.rs:7 (Store::lock), src/store.rs:12 (Store::releases)")
        self.assertEqual(self.add("review-tests", reversed_places).returncode, 0)
        self.assertEqual(self.add("review-correctness").returncode, 0)
        self.assertEqual([f["origin"] for f in self.review()["findings"]], ["older", "new", "older"])


class MergeTest(ReviewTest):
    def setUp(self):
        super().setUp()
        for agent in ("review-correctness", "review-security", "review-cleanup"):
            self.assertEqual(self.add(agent).returncode, 0)

    def test_a_merge_unites_axes_reviewers_and_places_and_numbers_again(self):
        done = run("merge", self.folder, 1, 3)
        self.assertEqual(done.returncode, 0, done.stderr)
        merged = self.review()["findings"]
        self.assertEqual([f["number"] for f in merged], [1, 2, 3])
        self.assertEqual((merged[0]["axes"], merged[0]["reviewers"], merged[0]["where"], merged[0]["fingerprint"]),
                         (["bugs", "security"], ["review-correctness", "review-security"],
                          ["src/store.rs:12 (Store::releases)", "src/store.rs:17 (Store::remove)"],
                          "bugs | src/store.rs | Store::releases + Store::remove"))
        self.assertEqual(merged[2]["title"], "lock and remove each format the root")
        self.assertIn("### 3. lock and remove each format the root\n", self.report())

    def test_a_merge_is_refused_once_the_triage_began_or_on_a_number_not_there(self):
        self.assertEqual(run("set", self.folder, 2, "verdict=CONFIRMED").returncode, 0)
        before = self.files()
        for numbers, named in (((1, 3), "triage"), ((1, 9), "9")):
            with self.subTest(numbers=numbers):
                done = run("merge", self.folder, *numbers)
                self.assertEqual(done.returncode, 1)
                self.assertIn(named, done.stderr)
                self.assertEqual(self.files(), before)


class SetTest(ReviewTest):
    def setUp(self):
        super().setUp()
        self.add_all()

    def test_the_triage_is_written_and_the_report_with_it(self):
        done = run("set", self.folder, "1,4", "verdict=CONFIRMED", "action=fix", "batch=1")
        self.assertEqual(done.returncode, 0, done.stderr)
        self.assertEqual(run("set", self.folder, 5, "action=fix at once", "batch=2",
                             "note=below the floor, new code, a local fix").returncode, 0)
        self.assertEqual(run("set", self.folder, 7, "severity=low").returncode, 0)
        found = self.review()["findings"]
        self.assertEqual([(f["verdict"], f["action"], f["batch"]) for f in (found[0], found[3], found[4])],
                         [("CONFIRMED", "fix", 1), ("CONFIRMED", "fix", 1), (None, "fix at once", 2)])
        self.assertEqual(found[6]["floor"], "above")
        report = self.report()
        self.assertIn("| 1 | the releases are sorted as text | bugs | wrong action | medium | new | CONFIRMED | fix | 1 |",
                      report)
        self.assertIn("Verdict: CONFIRMED. Action: fix, batch 1.\n", report)
        self.assertIn("Verdict: none, below the floor. Action: fix at once, batch 2. "
                      "below the floor, new code, a local fix.\n", report)
        self.assertIn("## Fix Plan\n\n- Batch 1: findings 1, 4\n- Batch 2: findings 5\n", report)
        self.assertFalse((self.folder.parent / "register.json").exists())

    def test_a_value_the_rules_forbid_is_refused_and_nothing_written(self):
        cases = [
            ("2", ["verdict=MAYBE"], "`verdict`"),
            ("5", ["verdict=CONFIRMED"], "floor"),
            ("1", ["action=fix"], "`verdict`"),
            ("1", ["verdict=CONFIRMED", "action=fix"], "`batch`"),
            ("1", ["verdict=CONFIRMED", "action=register"], "`note`"),
            ("1", ["verdict=CONFIRMED", "action=rejected", "note=no"], "`action`"),
            ("1", ["fixed_in=fix: x"], "`fixed_in`"),
            ("1", ["severity="], "`severity`"),
            ("5", ["severity=low"], "`severity`"),
            ("1", ["colour=red"], "colour"),
            ("9", ["verdict=CONFIRMED"], "9"),
        ]
        before = self.files()
        for numbers, values, named in cases:
            with self.subTest(numbers=numbers, values=values):
                done = run("set", self.folder, numbers, *values)
                self.assertEqual(done.returncode, 1, done.stdout)
                self.assertIn(named, done.stderr)
                self.assertEqual(self.files(), before)

    def test_the_tasks_of_a_batch_carry_their_proof(self):
        for values in (["1", "verdict=CONFIRMED", "action=fix", "batch=1"],
                       ["3", "verdict=CONFIRMED", "action=fix", "batch=1", "fixed_in=fix(store): remove the release alone"],
                       ["5", "action=fix at once", "batch=2", "note=below the floor, new code, a local fix"]):
            self.assertEqual(run("set", self.folder, *values).returncode, 0)
        done = run("tasks", self.folder)
        self.assertEqual(done.returncode, 0, done.stderr)
        self.assertEqual(done.stdout, (
            "### Batch 1\n"
            "- [ ] Finding 1: releases sorts by number\n"
            "  Proof: releases_come_by_number, watched failing, then passing\n"
            "- [x] Finding 3: the directory removed with remove_dir_all\n"
            "  Proof: a_removal_under_a_path_with_a_space_removes_the_release_alone, watched failing, then passing\n"
            "\n"
            "### Batch 2\n"
            "- [ ] Finding 5: one helper\n"
            "  Proof: none: no behaviour changes\n"))
        self.assertTrue(run("tasks", self.folder, 2).stdout.startswith("### Batch 2\n"))
        missing = run("tasks", self.folder, 4)
        self.assertEqual(missing.returncode, 1)
        self.assertIn("batch 4", missing.stderr)


class ChecksTest(ReviewTest):
    def setUp(self):
        super().setUp()
        root = Path(self.tmp.name)
        self.repo.commit("ignore", {".gitignore": "/local\n"})
        self.repo.write("local/tool.py", "")
        outside = tempfile.TemporaryDirectory()
        self.addCleanup(outside.cleanup)
        (root / "link").symlink_to(outside.name)

    def record(self, *commands):
        review = self.review()
        review["checks"] = [{"command": command, "result": "exit 0"} for command in commands]
        (self.folder / "findings.json").write_text(json.dumps(review, indent=2), encoding="utf-8")
        return run("report", self.folder)

    def test_a_check_naming_a_path_another_clone_lacks_is_refused(self):
        report = self.report()
        for command, path in (("local/tool.py check .", "local/tool.py"), ("python3 link/tool.py", "link/tool.py")):
            with self.subTest(command=command):
                done = self.record("cargo test", command)
                self.assertEqual(done.returncode, 1)
                self.assertIn(f"check 2: `{path}`", done.stderr)
                self.assertEqual(self.report(), report)

    def test_a_check_naming_its_program_by_name_or_a_path_of_the_clone_is_recorded(self):
        done = self.record("tool.py check .", "python3 src/store.rs --check", "cargo test -- --nocapture")
        self.assertEqual(done.returncode, 0, done.stderr)
        self.assertIn("- `tool.py check .`: exit 0\n- `python3 src/store.rs --check`: exit 0\n", self.report())


def entry(number, title, fingerprint, effect, state, reason, review="2026-11-02-store-phase-1"):
    return {"fingerprint": fingerprint, "axis": fingerprint.split(" | ")[0], "effect": effect, "title": title,
            "state": state, "review": review, "finding": number, "reason": reason}


class RegisterTest(ReviewTest):
    """The review's findings registered or rejected, in register.json beside its folder."""

    def setUp(self):
        super().setUp()
        for agent in ORDER[:-1]:
            self.assertEqual(self.add(agent).returncode, 0)
        kept = ANSWERS["review-performance"].replace("fix: a borrow", "fix: none: one allocation is worth no lifetime")
        self.assertEqual(self.add("review-performance", kept).returncode, 0)
        self.path = self.folder.parent / "register.json"
        self.other = entry(2, "prune takes no argument", "bugs | src/cli.rs | Action::Prune", "wrong action", "open",
                           "older code", review="2026-11-20-cli")

    def register(self):
        return json.loads(self.path.read_text(encoding="utf-8")) if self.path.exists() else None

    def test_a_finding_registered_or_rejected_enters_the_register_with_its_state_and_leaves_it(self):
        self.path.write_text(json.dumps({"entries": [self.other]}, indent=2) + "\n", encoding="utf-8")
        for values in (["2", "verdict=CONFIRMED", "action=register", "note=older code: a change of the port"],
                       ["3", "verdict=REFUTED", "action=rejected", "note=the root is the user's own"],
                       ["5", "action=register", "note=below the floor; the helper reaches older code"],
                       ["7", "action=register", "note=below the floor"]):
            done = run("set", self.folder, *values)
            self.assertEqual(done.returncode, 0, done.stderr)
        lock = entry(2, "the lock says installing whoever holds it", "error handling | src/store.rs | Store::lock",
                     "misleading", "open", "older code: a change of the port")
        remove = entry(3, "a path with a space makes remove delete other directories",
                       "security | src/store.rs | Store::remove", "data lost", "rejected", "the root is the user's own")
        helper = entry(5, "lock and remove each format the root",
                       "refactoring | src/store.rs | Store::lock + Store::remove", "maintenance", "open",
                       "below the floor; the helper reaches older code")
        clone = entry(7, "releases clones the root", "performance | src/store.rs | Store::releases", "slow", "kept",
                      "below the floor")
        self.assertEqual(self.register(), {"entries": [lock, remove, helper, clone, self.other]})
        self.assertEqual(run("set", self.folder, 5, "action=fix at once", "batch=1",
                             "note=below the floor, new code, a local fix").returncode, 0)
        self.assertEqual(run("set", self.folder, 2, "note=older code, low").returncode, 0)
        self.assertEqual(self.register(),
                         {"entries": [{**lock, "reason": "older code, low"}, remove, clone, self.other]})

    def test_a_malformed_register_is_refused_and_nothing_written(self):
        cases = [("[]\n", "`entries`"), ("{", "register.json")]
        for change, named in (({"state": "closed"}, "`state`"), ({"finding": "2"}, "`finding`"),
                              ({"effect": "cosmetic"}, "`effect`"), ({"axis": "colour"}, "`axis`"),
                              ({"fingerprint": "security | src/cli.rs | Action::Prune"}, "`fingerprint`")):
            cases.append((json.dumps({"entries": [{**self.other, **change}]}), named))
        cases.append((json.dumps({"entries": [self.other, self.other]}), "twice"))
        for text, named in cases:
            with self.subTest(named=named):
                self.path.write_text(text, encoding="utf-8")
                before = self.files()
                for done in (run("set", self.folder, 2, "verdict=CONFIRMED", "action=register", "note=older code"),
                             self.add("review-verifier", ONE)):
                    self.assertEqual(done.returncode, 1, done.stdout)
                    self.assertIn("register.json", done.stderr)
                    self.assertIn(named, done.stderr)
                    self.assertEqual(self.files(), before)
                    self.assertEqual(self.path.read_text(encoding="utf-8"), text)


class LaterReviewTest(ReviewTest):
    """A first review registers a finding and rejects another; a later review meets them with their lines moved."""

    def setUp(self):
        super().setUp()
        for agent in ("review-correctness", "review-security"):
            self.assertEqual(self.add(agent).returncode, 0)
        for values in (["2", "verdict=CONFIRMED", "action=register", "note=older code: a change of the port"],
                       ["3", "verdict=REFUTED", "action=rejected", "note=the root is the user's own"]):
            self.assertEqual(run("set", self.folder, *values).returncode, 0)
        self.path = self.folder.parent / "register.json"
        self.registered = self.path.read_text(encoding="utf-8") if self.path.exists() else None
        moved = self.repo.commit("phase 2", {"src/store.rs": MOVED_STORE})
        self.later = self.folder.parent / "2026-11-03-store-range"
        done = run("new", self.later, "--base", self.end, "--end", moved, "--by", "the request", "--text", "review",
                   "--tier", "small", "--agent", "review-security:medium")
        self.assertEqual(done.returncode, 0, done.stderr)
        answer = Path(self.tmp.name) / "later.txt"
        answer.write_text(LATER, encoding="utf-8")
        self.added = run("add", self.later, "review-security", answer)
        self.remove = {"review": "2026-11-02-store-phase-1", "finding": 3, "state": "rejected",
                       "title": "a path with a space makes remove delete other directories"}
        self.lock = {"review": "2026-11-02-store-phase-1", "finding": 2, "state": "open",
                     "title": "the lock says installing whoever holds it"}

    def later_review(self):
        return json.loads((self.later / "findings.json").read_text(encoding="utf-8"))

    def later_files(self):
        return [(self.later / name).read_bytes() for name in ("findings.json", "report.md")]

    def test_a_later_review_recognizes_a_registered_finding_whose_lines_moved_and_no_other(self):
        self.assertEqual(self.added.returncode, 0, self.added.stderr)
        self.assertEqual([f.get("matches") for f in self.later_review()["findings"]],
                         [[self.remove], [], [], [self.lock]])
        self.assertEqual(self.added.stdout,
                         "finding 1: the register holds 2026-11-02-store-phase-1 finding 3, rejected: "
                         "a path with a space makes remove delete other directories\n"
                         "finding 4: the register holds 2026-11-02-store-phase-1 finding 2, open: "
                         "the lock says installing whoever holds it\n")
        self.assertIn("Register: `2026-11-02-store-phase-1` finding 3, rejected: "
                      "a path with a space makes remove delete other directories.\n",
                      (self.later / "report.md").read_text(encoding="utf-8"))
        own = self.add("review-verifier", ANSWERS["review-security"])
        self.assertEqual(own.returncode, 0, own.stderr)
        self.assertEqual(self.review()["findings"][-1].get("matches"), [])

    def test_a_finding_the_register_holds_is_known_with_no_verification(self):
        before = self.later_files()
        for numbers, values, named in (("3", ["action=known", "note=the same root"], "`matches`"),
                                       ("1", ["action=known"], "`note`"),
                                       ("1", ["action=known", "note=x", "batch=1"], "`batch`")):
            with self.subTest(numbers=numbers, values=values):
                done = run("set", self.later, numbers, *values)
                self.assertEqual(done.returncode, 1, done.stdout)
                self.assertIn(named, done.stderr)
                self.assertEqual(self.later_files(), before)
        done = run("set", self.later, 1, "action=known", "note=the register's 2026-11-02-store-phase-1 finding 3")
        self.assertEqual(done.returncode, 0, done.stderr)
        first = self.later_review()["findings"][0]
        self.assertEqual((first["verdict"], first["action"]), (None, "known"))
        self.assertEqual(self.path.read_text(encoding="utf-8"), self.registered)
        report = (self.later / "report.md").read_text(encoding="utf-8")
        self.assertIn("| 1 | remove runs a name releases lists through a shell | security | data lost | high | older "
                      "| — | known | — |", report)
        self.assertIn("Verdict: none, in the register. Action: known. "
                      "the register's 2026-11-02-store-phase-1 finding 3.\n", report)

    def test_a_match_edited_by_hand_is_written_back_in_its_form(self):
        before = self.later_files()
        review = self.later_review()
        review["findings"][0]["matches"] = [dict(reversed(match.items())) for match in review["findings"][0]["matches"]]
        (self.later / "findings.json").write_text(json.dumps(review, indent=2), encoding="utf-8")
        done = run("report", self.later)
        self.assertEqual(done.returncode, 0, done.stderr)
        self.assertEqual(self.later_files(), before)

    def test_a_merge_unites_the_matches(self):
        done = run("merge", self.later, 1, 4)
        self.assertEqual(done.returncode, 0, done.stderr)
        self.assertEqual(self.later_review()["findings"][0].get("matches"), [self.remove, self.lock])


class FixtureTest(unittest.TestCase):
    """The reviews of the fixtures, written by hand from the design, are what the writer reads and writes."""

    @classmethod
    def setUpClass(cls):
        cls.tmp = tempfile.TemporaryDirectory()
        cls.continued = Path(cls.tmp.name) / "continued-review"
        cls.closed = Path(cls.tmp.name) / "roadmap-close"
        for folder, build in ((cls.continued, fixtures.continued_review), (cls.closed, fixtures.roadmap_close)):
            folder.mkdir()
            build(folder)

    @classmethod
    def tearDownClass(cls):
        cls.tmp.cleanup()

    def states(self):
        """(name, repository, findings.json's text, report.md's text) of each review state of the fixtures."""
        reviews = "docs/reviews/delivered-code"
        folder = self.continued / reviews / "2026-10-31-gateway-releases-phase-1"
        yield ("continued-review", self.continued, (folder / "findings.json").read_text(encoding="utf-8"),
               (folder / "report.md").read_text(encoding="utf-8"))
        for message, phase in (("docs(reviews): review phase 1", 1), ("refactor(cli): remove the clean alias", 1),
                               ("docs(reviews): review phase 2", 2), ("docs: show gateway install --from", 2)):
            commit = fixtures.git(self.closed, "log", "-1", "--format=%h", "--fixed-strings", "--grep", message)
            path = f"{reviews}/2026-10-31-gateway-releases-phase-{phase}"
            yield (message, self.closed, fixtures.git(self.closed, "show", f"{commit}:{path}/findings.json") + "\n",
                   fixtures.git(self.closed, "show", f"{commit}:{path}/report.md") + "\n")

    def test_each_report_of_the_fixtures_is_written_from_its_findings(self):
        template = TEMPLATE.read_text(encoding="utf-8")
        for name, _, source, report in self.states():
            with self.subTest(state=name):
                review = json.loads(source)
                self.assertEqual(findings.problems(review), [])
                self.assertEqual(findings.dump(review), source)
                self.assertEqual(findings.render(review, template), report)

    def test_the_origin_floor_and_fingerprint_computed_agree_with_the_fixtures(self):
        for name, repository, source, _ in self.states():
            git = scope.repository(repository)
            review = json.loads(source)
            for finding in review["findings"]:
                with self.subTest(state=name, finding=finding["number"]):
                    self.assertEqual(findings.origin(git, review["scope"], finding["where"]), finding["origin"])
                    self.assertEqual(findings.floor(finding["effect"], finding["severity"]), finding["floor"])
                    self.assertEqual(findings.fingerprint(finding["axes"], finding["where"]), finding["fingerprint"])

    def test_the_register_holds_the_reviews_findings_registered_or_rejected(self):
        reviews = "docs/reviews/delivered-code"
        for message in ("docs(reviews): review phase 1", "docs(reviews): review phase 2", "docs: show gateway install"):
            with self.subTest(state=message):
                commit = fixtures.git(self.closed, "log", "-1", "--format=%h", "--fixed-strings", "--grep", message)
                entries = []
                for folder in fixtures.git(self.closed, "ls-tree", "-d", "--name-only", f"{commit}:{reviews}").split():
                    review = json.loads(fixtures.git(self.closed, "show", f"{commit}:{reviews}/{folder}/findings.json"))
                    entries += filter(None, (findings.entry(folder, f) for f in review["findings"]))
                self.assertEqual(findings.dump_register({"entries": entries}),
                                 fixtures.git(self.closed, "show", f"{commit}:{reviews}/register.json") + "\n")

    def test_the_closing_pass_finding_meets_the_rejected_entry_of_its_site(self):
        register = json.loads((self.closed / "docs/reviews/delivered-code/register.json").read_text(encoding="utf-8"))
        where = ["src/infrastructure/gateway_store.rs:114 (FsGatewayStore::stage_from)",
                 "src/infrastructure/gateway_store.rs:103 (FsGatewayStore::stage)"]
        self.assertEqual(findings.matches(register["entries"], "2026-11-01-gateway-releases", ["security"], where),
                         [{"review": "2026-10-31-gateway-releases-phase-2", "finding": 3, "state": "rejected",
                           "title": "an archive given as a symbolic link is copied from its target"}])
        elsewhere = ["src/application/gateway.rs:222 (FsGatewayStore::stage_from)"]
        self.assertEqual(findings.matches(register["entries"], "2026-11-01-gateway-releases", ["security"], elsewhere),
                         [])

    def review_to_verify(self):
        _, _, source, _ = next(self.states())
        return json.loads(source)

    def test_a_review_that_breaks_a_rule_is_refused_with_the_rule(self):
        def broken(change):
            review = self.review_to_verify()
            change(review)
            return " ".join(findings.problems(review))

        self.assertIn("colour", broken(lambda r: r.update(colour="red")))
        self.assertIn("number", broken(lambda r: r["findings"][1].update(number=5)))
        self.assertIn("floor", broken(lambda r: r["findings"][4].update(floor="above")))
        self.assertIn("fingerprint", broken(lambda r: r["findings"][0]["where"].pop()))
        self.assertIn("matches", broken(lambda r: r["findings"][0].update(matches=[{"review": "x"}])))
        self.assertIn("floor", broken(lambda r: r["findings"][4].update(verdict="CONFIRMED")))
        self.assertIn("stage", broken(lambda r: r["status"].update(stage="done")))
        self.assertIn("complete", broken(lambda r: r["status"].update(stage="complete")))
        self.assertIn("`/tmp/tmp.aB3dE5gH7j/scope.txt`",
                      broken(lambda r: r["status"].update(note="read /tmp/tmp.aB3dE5gH7j/scope.txt")))
        self.assertIn("`/tmp/tmp.aB3dE5gH7j/x`", broken(lambda r: r["findings"][2].update(note="/tmp/tmp.aB3dE5gH7j/x")))
        self.assertEqual(broken(lambda r: None), "")

    def test_the_template_holds_each_part_once(self):
        review = self.review_to_verify()
        template = TEMPLATE.read_text(encoding="utf-8")
        for bad, named in ((template.replace("{{findings}}", ""), "{{findings}}"),
                           (template + "\n{{colour}}\n", "{{colour}}")):
            with self.subTest(named=named):
                with self.assertRaises(findings.FindingsError) as raised:
                    findings.render(review, bad)
                self.assertIn(named, str(raised.exception))

    def test_a_folder_holds_the_review_it_names(self):
        source = self.continued / "docs/reviews/delivered-code/2026-10-31-gateway-releases-phase-1"
        with tempfile.TemporaryDirectory() as tmp:
            copy = Path(tmp) / "2026-11-01-another"
            shutil.copytree(source, copy)
            done = run("report", copy)
            self.assertEqual(done.returncode, 1)
            self.assertIn("2026-10-31-gateway-releases-phase-1", done.stderr)


if __name__ == "__main__":
    unittest.main()
