# Phase 6: Fixes And Closing Pass

---

## Status

**Current Status:** 🟢 Done (100% — 7/7)
**Started:** 2026-11-05
**Completed:** 2026-11-06
**Blocked By:** —

---

## Objective

Fix phase 5's findings by the v2's fix method, then end the cycle by its stopping rule: one
security pass on the fixes' diff, the recorded mutants run again, that pass's findings fixed or
registered, and no pass after. Record, from this first full cycle, what is ai-ctl's and what every
repository shares.

### Out of Scope
A further review after the closing pass's fixes. The findings in the debt register.

---

## Tasks

### Fixes
- [x] Fix the findings above the floor, one batch per commit: the reproduction from `repro/` seen failing first, `cargo mutants --in-diff` on the batch's diff with the kinds applied by hand, the sibling paths and the input class named, a new test that binds a port, takes a lock or runs two processes at once run 30 times
  Proof: each batch's entry in the report: the red test, its mutants, the checks green
- [x] Apply the dependency findings: an update within the requirements with the checks, a crate changed beyond them or replaced only on the user's yes
  Proof: the checks green after each update; `cargo deny check` clean
- [x] Write each fix that changes what ai-ctl does, and each dependency changed, in `CHANGELOG.md`'s 0.2.0 entry, and in `docs/commands.md` when a command changes
  Proof: the entries beside their batches

### Closing Pass
- [x] Run the security pass on the fixes' diff, and the recorded mutants again with the regression run
  Proof: the pass's report; the regression run's output, every recorded mutant caught
- [x] Fix the pass's findings that bear on the fixes or are medium or worse; register the rest; run no pass after
  Proof: each fix's batch entry; the register's entries

### The Skill In Use
- [x] Record each failure of the skill's fix method seen in these fixes — the reproduction seen failing first, the mutants on the batch's diff, the sibling paths and the input class, the stopping rule — in its evals' `failures_in_use`, with its source, and fix it at the line that let it through
  Proof: `scripts/audit.py` clean after each fix; the report naming the fix method's steps this use exercised, as phase 4 left them to it

### Record
- [x] Write `shared-and-specific.md` in this roadmap's folder: what is ai-ctl's and what every repository shares, from phase 0's draft and this cycle's use
  Proof: the user's review of the record

---

## Technical Details

### Files to Modify
```
src/  tests/                                   (the fixes)
Cargo.toml  Cargo.lock                         (the dependency findings)
CHANGELOG.md                                   (0.2.0 — unreleased)
docs/commands.md                               (when a command changes)
docs/reviews/delivered-code/                   (the closing pass, the debt register)
```

### Dependencies
Phase 5's report and fix plan.

### Constraints
- `CLAUDE.md`'s tests rules: test-first, a bug fix starts with the test that reproduces it, each
  commit's code diff through `cargo mutants --in-diff`.
- 0.2.0 stays unreleased: the release comes after this roadmap and CI/CD.
