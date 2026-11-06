# Phase 1: Evals and Baselines

---

## Status

**Current Status:** 🟢 Done (100% — 9/9)
**Started:** 2026-10-13
**Completed:** 2026-10-13
**Blocked By:** —

---

## Before Starting This Phase

Read `phase-0-analysis-and-decisions.md` and `phase-0-analysis-and-decisions-report.md` in full before
touching anything here: the decisions already taken, the problems and
deviations recorded, and the changes made to this phase.

---

## While Working

Keep `phase-1-evals-and-baselines-report.md` current as the work happens — after each significant
step, and before every commit, pause, or end of session.

---

## Objective

The skill's evals exist and have run without the skill; each failure the runs show is quoted, and
the assertions are written from those failures.

---

## Overview

### Why This Phase Matters
authoring-skills writes only what answers an observed failure. Without runs that show what an
agent does wrong when asked for this review, the skill would carry guidance nobody needed and
miss what agents actually get wrong.

### What It Enables
The build of phase 2 against a list of quoted failures, and the comparison of phase 3 against a
baseline.

### Out of Scope
Any skill file other than its evals. Running the skill itself.

---

## Tasks

### Cases
- [x] Write two or three realistic review requests in the skill's `evals/evals.json`, worded as the user would word them, each with the result it should produce
- [x] Build the fixtures the cases need, as decided in phase 0, and pin each to a commit
- [x] Validate the evals file and prepare the first iteration with authoring-skills' `scripts/workspace.py`

### Baselines
- [x] Show the user the count and estimated cost of the baseline runs with `scripts/run.py`, and start them only after the user's yes
- [x] Run each case without the skill, one run per case
- [x] Run the built-in `/code-review` once on the v2 code, as phase 0 decided, and record what it finds and what it misses, for comparison

### Failures
- [x] Read each run's transcript and output in full, and quote each failure as the run shows it
- [x] Write the assertions from the failures, each named after what it checks
- [x] Record what the baselines already did right, so the skill does not repeat it

---

## Technical Details

### Files to Modify
```
<skill folder>/evals/evals.json          (path set by the [skills] table)
<skill folder>/evals/fixtures.py         (and the files it builds the fixtures from)
<eval workspace>/                        (runs, set by the [skills] table)
docs/roadmap/completed/review-skill/phase-1-evals-and-baselines-report.md
```

### Dependencies
The `[skills]` table and the decisions of phase 0. The baseline runs need the Agent tool:
authoring-skills gives it to a skill whose evals say `starts_agents: true` before its agents'
files exist (the agent setup, `docs/decisions/2026-10-13-eval-runs-with-agents.md`, § Revised
The Same Day, commit `fe4f87e`), and this skill's evals say so.

### Constraints
- The evals folder is the only part of the skill written in this phase: `evals/evals.json`,
  and `evals/fixtures.py` with the files it builds the fixtures from.
- Paid runs start only after the user's yes on their count and cost, within the budget rule of
  phase 0.
- No case is played by hand in place of a fresh agent's run.

---

## Acceptance Criteria

- [x] `evals/evals.json` validates and holds the cases and their assertions
- [x] Every case has a baseline run, read in full
- [x] Every assertion traces to a quoted failure or to a requirement the user stated
