# Phase 2: Scripts

---

## Status

**Current Status:** 🟢 Done (100% — 11/11)
**Started:** 2026-11-02
**Completed:** 2026-11-02
**Blocked By:** —

---

## Objective

Write the scripts phase 0 designed, each test-first, so that the scope, the report, the debt
register, the regression run and the dependency facts are computed rather than transcribed.

### Out of Scope
`SKILL.md` and the agents, which phase 3 changes to use these scripts.

---

## Tasks

### Scope
- [x] `scope.py`: the scope by hunk, each hunk with the item that holds it, a function, an `impl`, a type or a constant, as `path:first-last (symbol)`, and the lines handed to the reviewers counted; the callers left to the agents
  Proof: `test_scope.py`'s new tests watched failing on their assertion, then passing
- [x] `scope.py`: each line's origin, new or older than the scope's base, by `git blame`
  Proof: tests on a repository made in scratch, watched failing, then passing

### Findings
- [x] The report writer: the agents' answers read and checked, a malformed block refused with its field named, the findings numbered and merged on request, the report written from `assets/report.md` with its summary table, the fix plan's frame and the fix roadmap's tasks
  Proof: tests on answers in the agents' format, watched failing, then passing
- [x] The debt register: each finding's fingerprint, and a finding already registered or rejected recognized by a later review
  Proof: tests, a finding whose lines moved recognized and a different one not, watched failing, then passing

### Regression And Dependencies
- [x] The regression run: each mutant of `mutants.json`, an exact replacement, applied in a copy and its killing test run, each one no test catches named and each one whose text no longer occurs reported
  Proof: tests on a fixture repository, one mutant caught and one missed, watched failing, then passing
- [x] The dependency facts: each crate's record of what `cargo-deny` does not check — the updates, the maintenance, the reliability — from `cargo update --dry-run --verbose`, crates.io and GitHub, parsed from captures listed in `tests/fixtures/README.md`
  Proof: tests on the captures, watched failing, then passing; no test reaches the network
- [x] Install `cargo-deny` 0.20.2 by `cargo install --locked`, with its entry in `docs/toolchain.md`'s record, and write `deny.toml` with the user's list of licences, the sources limited to crates.io and the duplicates as warnings
  Proof: `cargo deny check` on the crate, its output recorded; the user's review of the licence list
- [x] The evals' fixtures take `deny.toml`, a pinned copy beside `claude.md` in every case's base commit
  Proof: `test_fixtures.py`'s test watched failing, then passing; `cargo deny check` on the built `dependency-check` fixture reporting RUSTSEC-2026-0190

### Guards
- [x] The review agents' guards: a hook script wired in the project's `.claude/settings.json`, acting only on a review agent's `agent_type`: `PreToolUse` on Bash refusing the commands phase 0 lists, `SubagentStop` blocking on a changed `git status --porcelain` or a process or port the agent left
  Proof: tests on hook inputs in the form phase 0's probe captured, watched failing, then passing; one probe session with the hooks wired, after the user's yes

### Checks
- [x] A check of the repository that refuses a raw U+2028 or U+2029 in the tracked text files outside `docs/reviews/`, added to the checks
  Proof: the check fails on a file holding one and passes on the tree, watched failing first
- [x] Each new test file added to the checks of `CLAUDE.md` and `.agent-conventions.toml`
  Proof: every check runs green

---

## Technical Details

### Files to Modify
```
.claude/skills/reviewing-delivered-code/scripts/scope.py
.claude/skills/reviewing-delivered-code/scripts/          (the report writer, the register, the regression run, the dependency facts)
.claude/skills/reviewing-delivered-code/evals/test_*.py
tests/fixtures/README.md
deny.toml                                                 (cargo-deny's policy)
docs/toolchain.md                                         (cargo-deny's install)
.claude/settings.json                                     (the guards' hooks)
CLAUDE.md                                                 (Checks)
.agent-conventions.toml                                   (untracked: [roadmap] and [skills] checks)
```

### Dependencies
Phase 0's Design section; phase 1's cases, which name the outputs these scripts must give.

### Constraints
- The standard library only, as `scope.py` does today.
- No test depends on timing, a GPU or the network.
- After each write to the skill's folder, `authoring-skills`' `scripts/audit.py` runs clean.
