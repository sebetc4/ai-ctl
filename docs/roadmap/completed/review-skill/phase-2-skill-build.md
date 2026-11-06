# Phase 2: Skill Build

---

## Status

**Current Status:** 🟢 Done (100% — 8/8)
**Started:** 2026-10-13
**Completed:** 2026-10-13
**Blocked By:** —

---

## Before Starting This Phase

Read `phase-1-evals-and-baselines.md` and `phase-1-evals-and-baselines-report.md` in full before
touching anything here: the decisions already taken, the problems and
deviations recorded, and the changes made to this phase.

---

## While Working

Keep `phase-2-skill-build-report.md` current as the work happens — after each significant
step, and before every commit, pause, or end of session.

---

## Objective

The skill and every part phase 0 decided for it are written from the failures of phase 1 and the
user's statements, and pass the audit.

---

## Overview

### Why This Phase Matters
This is where the analysis becomes a tool. Writing only what the failures and the user's
statements call for keeps the skill small enough to read and to maintain.

### What It Enables
The evaluation of phase 3 with the skill in place.

### Out of Scope
Running the evals with the skill. Changing ai-ctl's code or configuration, new lints included:
those belong with the fixes of the first review.

---

## Tasks

### Skill
- [x] Start `SKILL.md` from authoring-skills' template for the kind chosen in phase 0
- [x] Write the guidance that answers each failure quoted in phase 1, and the requirements the user stated, nothing more
- [x] Write the agents, references and other parts the organisation decided in phase 0 calls for

### Scripts and Tools
- [x] Write the unit tests of each script first and watch them fail, then write the script
- [x] Install the tools the user approved in phase 0, and record each with its version in this phase's report

### Audit
- [x] Run authoring-skills' `scripts/audit.py` on the skill and fix every error
- [x] Have the `skill-auditor` agent judge the skill against authoring-skills' writing guide, and fix what it reports
- [x] Check that every block of guidance traces to a failure of phase 1 or a requirement of the user, and remove what does not

---

## Technical Details

### Files to Modify
```
<skill folder>/SKILL.md
<skill folder>/references/
<skill folder>/scripts/                  (with their tests)
<agents folder>/                         (if phase 0 decided agents)
docs/roadmap/completed/review-skill/phase-2-skill-build-report.md
```

### Dependencies
The quoted failures and assertions of phase 1; the decisions of phase 0. The bugs agent answers
the gaps of the built-in `/code-review` that phase 1's report records, run on `src/` at `high`:
no test read, no check run, no finding verified, ten findings at most; it found six of the eight
bugs the two runs give together, the baseline four.

### Constraints
- `scripts/audit.py` runs after every write to the skill, whatever tool made it.
- No change to ai-ctl's source, tests or configuration.
- The user's plan caps spend at about $20 per usage window (2026-10-13): the tiers and their
  cost estimate (phase 0, decision 11) account for it, a review of the whole v2 code at
  Opus 5.5 `xhigh` being estimated above one window.

---

## Acceptance Criteria

- [x] `scripts/audit.py` reports no error
- [x] The `skill-auditor` agent answers `VERDICT: PASS`
- [x] Every script has unit tests, and they pass
