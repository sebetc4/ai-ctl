# Phase 1: Evals And Baseline

---

## Status

**Current Status:** 🟢 Done (100% — 6/6)
**Started:** 2026-11-01
**Completed:** 2026-11-02
**Blocked By:** —

---

## Objective

Write the cases the v2 must pass, run them on the skill as it stands, and turn each failure the
runs show into an assertion, so that phase 3 writes only what answers them.

### Out of Scope
Any change to `SKILL.md`, its agents or its scripts.

---

## Tasks

### Cases
- [x] Rewrite `evals/evals.json`'s cases for phase 0's decisions: a roadmap's diff scoped by hunk on a fixture repository; one go-ahead and the session's triage, the case `cost-gate` rewritten since a request that accepts the cost beforehand is now the go-ahead; the floor and the debt register; the dependency axis on a `Cargo.lock` holding a crate with an advisory and one behind its newest release; a review continued from its report; the closing security pass with the regression run — each with the result it should produce
  Proof: `scripts/workspace.py` validates `evals/evals.json`
- [x] Bring the fixtures to the cases: `evals/fixtures/claude.md` to today's `CLAUDE.md`, the dependency case's lock file, each built by `evals/fixtures.py`
  Proof: `test_fixtures.py`'s new tests watched failing on their assertion, then passing
- [x] Write the trigger queries of the new requests — a check of the crates' maintenance and updates, a review at a phase's close, a security pass alone — and the near requests that must not trigger
  Proof: `scripts/workspace.py` validates the triggers

### Baseline
- [x] Run the cases on the commit before the change, after the user's yes on the batch
  Proof: every run of the iteration finished, by `scripts/run.py --status`
- [x] Grade the runs, class each failure, quote it, and write the assertions from the failures, each named after what it checks
  Proof: `scripts/grade.py` and `scripts/benchmark.py` on the iteration, the classes in `classes.json`
- [x] Measure the current description's trigger rates on the new queries, after the user's yes
  Proof: `scripts/triggers.py`' rates recorded in the report

---

## Technical Details

### Files to Modify
```
.claude/skills/reviewing-delivered-code/evals/evals.json
.claude/skills/reviewing-delivered-code/evals/fixtures/
.claude/skills/reviewing-delivered-code/evals/fixtures.py
.claude/skills/reviewing-delivered-code/evals/test_fixtures.py
```

### Dependencies
Phase 0's Design section. `authoring-skills`' scripts, run by their path in its folder.

### Constraints
- The baseline is the committed skill, passed to `scripts/workspace.py --baseline`.
- A failure is classed before any assertion is written from it.
- The runs' workspace stays under `.eval-runs/`, ignored by git.
