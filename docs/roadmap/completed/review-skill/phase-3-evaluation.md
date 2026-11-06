# Phase 3: Evaluation

---

## Status

**Current Status:** 🟢 Done (100% — 6/6)
**Started:** 2026-10-13
**Completed:** 2026-10-15
**Blocked By:** —

---

## Before Starting This Phase

Read `phase-2-skill-build.md` and `phase-2-skill-build-report.md` in full before
touching anything here: the decisions already taken, the problems and
deviations recorded, and the changes made to this phase.

---

## While Working

Keep `phase-3-evaluation-report.md` current as the work happens — after each significant
step, and before every commit, pause, or end of session.

---

## Objective

The skill passes its evals: each failure observed in phase 1 is gone, or recorded with the reason
it stays, and its description triggers when it should and only then.

---

## Overview

### Why This Phase Matters
A skill that has not been run against its baseline is a guess. This phase measures what the skill
changes, and fixes what it does not.

### What It Enables
A first real review in phase 4 with a skill whose behaviour is measured.

### Out of Scope
Reviewing ai-ctl's code for real. Extending the evals beyond the failures of phase 1, unless a new
failure appears in a run.

---

## Tasks

### Runs
- [x] Show the user the count and estimated cost of the runs with the skill, and start them only after the user's yes
- [x] Grade the runs with authoring-skills' `scripts/grade.py` and sum them up with `scripts/benchmark.py`
- [x] Review the runs with the user in `scripts/viewer.py` and record the feedback

### Iterations
- [x] Record for each iteration what changed in the skill and what the benchmark showed

### Triggering
- [x] Write the queries that should and should not trigger the skill, a request for a session's summary among those that should not (phase 0), then measure its description with `scripts/triggers.py` after the user's yes on the cost
- [x] Tune the description from the training queries only, if the measure calls for it, and record the held-out result

---

## Technical Details

### Files to Modify
```
<skill folder>/                          (fixes between iterations)
<skill folder>/evals/evals.json          (trigger queries)
<eval workspace>/                        (iterations, gradings, benchmarks)
docs/roadmap/completed/review-skill/phase-3-evaluation-report.md
```

### Dependencies
The skill of phase 2, with its agents in `.claude/agents/` listed under `agents` in
`evals.json`, and the baselines of phase 1.

### Constraints
- Paid runs start only after the user's yes, within the budget rule of phase 0.
- Each change to the skill takes the proof authoring-skills requires for its kind.
- One paid run per usage window, each started from a new session with `run.py --case` and
  `--run`: the user's plan caps spend at about $20 per window, and a run the limit stops is
  lost (phase 1 report, Decisions).
- The skill splits a review whose estimate passes $15 into passes, one per session (phase 2
  report, Decisions). The v2 case at `high` is estimated above that, so its run under an $18
  ceiling does one pass: before it runs, reword its prompt with the user, which gives the case
  a new digest and loses its baseline for `--reuse`, or record the run as a partial
  observation.

---

## Acceptance Criteria

- [x] Every assertion passes, or its failure is recorded with the reason it stays
- [ ] The whole set passed once after the last fix
- [x] The description's trigger rates are measured and recorded, on training and held-out queries
