# Phase 5: Workflow Record

---

## Status

**Current Status:** 🟢 Done (100% — 10/10)
**Started:** 2026-10-16
**Completed:** 2026-10-16
**Blocked By:** —

---

## Before Starting This Phase

Read `phase-4-first-review.md` and `phase-4-first-review-report.md` in full before
touching anything here: the decisions already taken, the problems and
deviations recorded, and the changes made to this phase.

---

## While Working

Keep `phase-5-workflow-record-report.md` current as the work happens — after each significant
step, and before every commit, pause, or end of session.

---

## Objective

A record of the workflow, from the analysis of the plugins to the first real review, lets the same
work be done on another repository, and separates what was specific to ai-ctl from what was not.

---

## Overview

### Why This Phase Matters
The same work will be done on other repositories (Node, Python, Bash…), each getting a review
skill of its own, and the results compared to build a skill that builds review skills: the part
every repository shares, and the construction of the parts each one needs. That comparison needs
each repository's workflow written the same way, with what depended on the repository marked as
such.

### What It Enables
The next repository's roadmap, written from this record, and the later comparison.

### Out of Scope
The work on other repositories. The skill that builds review skills.

---

## Tasks

### Record
- [x] Write the workflow step by step from the phase files and reports: inputs, actions, outputs, decisions, and the report that holds each step's details
- [x] Mark in it what depended on ai-ctl (Rust, a CLI, its conventions, its installed tools, its machine) and what did not
- [x] Record the cost of each phase (sessions, tokens, time), as the reports give it
- [x] List what to change in the workflow for the next repository, from the problems and deviations the reports recorded
- [x] Record the exact cost of each pass of the first review from its session's `cost-state` record, correcting phase 4's report, which calls the exact costs of passes 2 to 4 lost

### Skill
- [x] Put the user's cost gates in `reviewing-delivered-code`: the estimate shown and the user's yes awaited before the first agent, and a stop with the measured cost before each new wave of agents
- [x] Fix from use the misfits the review's pass 5 met, and the agents' container names: each quoted with its transcript, fixed at its source, `audit.py`, one `skill-auditor` pass
- [x] Check the cost gates with one eval case run with the skill on a small fixture, after the user's yes on its cost

### Handover
- [x] Put the record where the next repositories' work can read it, decided with the user
- [x] Check the record against the reports: every step it cites exists in a report

---

## Technical Details

### Files to Modify
```
<workflow record location>               (decided with the user)
.claude/skills/reviewing-delivered-code/       (the cost gates and the fixes from use)
.claude/agents/                                (the fixes from use)
docs/roadmap/completed/review-skill/phase-5-workflow-record-report.md
```

### Dependencies
The reports of phases 0 to 4. The review's pass 5, in its session's transcript
and in the resume note that closed it.

### Constraints
- The record states facts from the reports; it does not design the skill that builds review
  skills.
- The record is short: what changed, the figures and the decisions, per the agent setup's
  `docs/decisions/2026-10-15-validation-weight.md`.

---

## Acceptance Criteria

- [x] Each step of the record names its inputs, actions, outputs and decisions, and links the report that holds its details
- [x] The parts specific to ai-ctl are marked as such
- [x] The cost of each phase is recorded
- [ ] The skill waits for the user's yes before its first agent and before each new wave, and the eval case shows it
