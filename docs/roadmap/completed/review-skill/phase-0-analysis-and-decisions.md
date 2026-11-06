# Phase 0: Analysis and Decisions

---

## Status

**Current Status:** 🟢 Done (100% — 10/10)
**Started:** 2026-10-13
**Completed:** 2026-10-13
**Blocked By:** —

---

## While Working

Keep `phase-0-analysis-and-decisions-report.md` current as the work happens — after each significant
step, and before every commit, pause, or end of session.

---

## Objective

The analysis of the studied review plugins is final, every decision it leaves open is taken and
recorded with its reason, and the repository is ready for skill work under authoring-skills'
conventions.

---

## Overview

### Why This Phase Matters
The analysis report is still open: the user may add, change or remove points, as was done for
security and tests. The choices it prepares (organisation, axes, placement, name, budget, tools)
shape every later phase, so they are taken here, once the report is final, and not at the
roadmap's creation.

### What It Enables
The cases of phase 1 are written against a settled intent; the build of phase 2 follows recorded
decisions instead of re-deciding them.

### Out of Scope
Writing any file of the skill. Running any eval. Reviewing ai-ctl's code.

---

## Tasks

### Analysis
- [x] Collect the user's feedback on the analysis report and apply each addition, change or removal to `report.md` and `report.fr.md`, keeping both versions in agreement
- [x] Record the studied plugins with the versions their folder names give, so the analysis can be reproduced
- [x] Settle with the user where the analysis report lives in the versioned trace, since the folder holding it is ignored by git, and put it there

### Decisions
- [x] Take with the user each decision the final report leaves open, and record the choice and its reason in this phase's report
- [x] List the proposals beyond the skill itself (scripts, guards, tool installs) with their cost, and record the user's answer for each
- [x] Write the skill's intent as authoring-skills asks: what it lets the agent do, when it applies in the user's words, what a good result is, the edge cases and dependencies that matter
- [x] Choose the skill's kind (task, reference or discipline) and record why

### Repository Setup
- [x] Write the `[skills]` table of `.agent-conventions.toml` with the user, per authoring-skills' `references/conventions.md`: the folders that hold skills, the evals folder, the eval workspace, the excluded features
- [x] Check that authoring-skills' `scripts/conventions.py skills` returns `status: ok`
- [x] Record the budget rule for paid runs decided with the user (cap per batch, effort tier), which phases 1, 3 and 4 apply

---

## Technical Details

### Files to Modify
```
the analysis report
its French translation
.agent-conventions.toml
docs/roadmap/completed/review-skill/phase-0-analysis-and-decisions-report.md
```

### Dependencies
The user's reading of the analysis report.

### Constraints
- The English report is the reference; the French one is its translation and changes with it.
- A decision is recorded with its reason and the alternatives set aside, so the next repository's
  work can tell a choice tied to ai-ctl from a general one.

---

## Acceptance Criteria

- [ ] The analysis report holds the user's feedback, and its English and French versions agree
- [x] Every decision the report leaves open has a recorded choice and reason
- [x] The skill's intent and kind are written down
- [x] `scripts/conventions.py skills` returns `status: ok`
