# Phase 4: First Review of ai-ctl

---

## Status

**Current Status:** 🟢 Done (100% — 10/10)
**Started:** 2026-10-15
**Completed:** 2026-10-16
**Blocked By:** —

---

## Before Starting This Phase

Read `phase-3-evaluation.md` and `phase-3-evaluation-report.md` in full before
touching anything here: the decisions already taken, the problems and
deviations recorded, and the changes made to this phase.

---

## While Working

Keep `phase-4-first-review-report.md` current as the work happens — after each significant
step, and before every commit, pause, or end of session.

---

## Objective

The skill reviews the code of the v2-rebuild roadmap for real; its report is kept, every finding
gets a verdict from the user, and no code of ai-ctl changes.

---

## Overview

### Why This Phase Matters
Evals cover a few cases; a real review covers the whole code the skill was made for. It shows the
skill's cost, its false positives and its misses in real conditions, and gives the input of the
fixes.

### What It Enables
A separate roadmap that fixes the confirmed findings, and a skill corrected by what real use
showed.

### Out of Scope
Fixing any finding in ai-ctl's code: a separate roadmap does it.

---

## Tasks

### Before the run
- [x] Run one `skill-auditor` pass on the skill as phase 3 left it, and fix each of its points at its source

### Run
- [x] Run the skill on the scope of the v2-rebuild roadmap, at the effort and within the budget decided in phase 0, after the user's yes on its estimated cost
- [x] Record the run's cost per agent, duration and coverage, and set step 4's estimate rates against the per-agent costs
- [x] Keep the review's report where phase 0 decided

### Findings
- [x] Go through every finding with the user and record a verdict for each: confirmed, rejected or to decide, with its evidence
- [x] Count which of the eight bugs of the v2 code that phase 1's two runs found together the review finds again: the measure of `review-correctness`
- [x] Record the false positives, and the misses the user knows of, as input for the skill
- [x] List the confirmed findings to fix, grouped by root cause, as the input of a later fix roadmap

### Skill
- [x] Record the changes to the skill the real run calls for
- [x] Make the changes the user approves as fixes from use: each failure quoted with its transcript, fixed at its source, `audit.py`, then one `skill-auditor` pass

---

## Technical Details

### Files to Modify
```
docs/reviews/delivered-code/<date>-<scope>.md   (decided in phase 0)
.claude/skills/reviewing-delivered-code/       (approved changes only)
.claude/agents/                                (approved changes only)
docs/roadmap/completed/review-skill/phase-4-first-review-report.md
```

### Dependencies
The skill evaluated in phase 3.

### Constraints
- Read-only on ai-ctl: `git diff` on `src/`, `tests/`, `Cargo.toml` and `deploy/` stays empty.
- Commands that start processes on the GPU or change the machine are not run during the review.
- The user's plan caps spend at about $20 per usage window (2026-10-13): the review spans
  windows as the skill's tiers provide. The skill estimates the v2 scope at `high` at about
  $39, three passes, each in a fresh window from a new session.
- Real use is the skill's proof, under the default regime of the agent setup's
  `docs/decisions/2026-10-15-validation-weight.md`: no further eval iteration before this
  review; a failure seen in it is fixed from use.

---

## Acceptance Criteria

- [x] The review report exists, covers the v2-rebuild scope, and holds its coverage record
- [x] Every finding has a verdict from the user
- [x] The confirmed findings are listed for a later fix roadmap
- [x] No code or configuration of ai-ctl changed in this phase
