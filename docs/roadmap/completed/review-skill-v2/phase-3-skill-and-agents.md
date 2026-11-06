# Phase 3: Skill And Agents

---

## Status

**Current Status:** 🟢 Done (100% — 12/12)
**Started:** 2026-11-02
**Completed:** 2026-11-02
**Blocked By:** —

---

## Objective

Write the v2 of `reviewing-delivered-code` and of its agents from phase 1's failures and the
user's rules: the workflow and its stopping rule, the scripts of phase 2 in its steps, the
dependency axis, and the known defects fixed.

### Out of Scope
The runs that compare the v2 with the baseline, which phase 4 makes.

---

## Tasks

### The Skill
- [x] Rewrite `SKILL.md` from phase 1's failures and the user's rules: the workflow and its stopping rule, the scope by hunk, one go-ahead per review, mutants in the background, the session's triage, the floor and the debt register, the report writer, `repro/` kept for the fixes, "open a clean-up roadmap" replaced
  Proof: `scripts/audit.py` clean; the cases' runs moved to phase 4's first task
- [x] Remove the nine roadmap citations, the rates and the plan's figures
  Proof: a search of the skill's folder and its agents for `roadmap` and `phase` finds only the paths a step uses
- [x] Write the fix method: sibling paths, input classes, a document's statements checked as claims, new tests run until they show they are stable, the kinds of mutants a fix applies by hand
  Proof: `scripts/audit.py` clean; the `phase-close` case's run moved to phase 4's first task
- [x] Bring the report to the dependency axis: `review-dependencies`' line in Coverage and its facts, in `assets/report.md`, the report writer's frame since phase 2
  Proof: phase 2's report writer tests, run on the new template
- [x] Rewrite the description for the thirteen axes, the review at a phase's close and the security pass alone
  Proof: `scripts/audit.py` clean on the description; its trigger rates against phase 1's moved to phase 4's trigger task

### The Agents
- [x] Write the agents' shared rules file and point each agent at it, by the mechanism phase 0 probed: what may run, the sandbox, the threat context, the `PROBES` channel; what `scripts/guard.py` refuses, git on a repository in scratch as `git -C <absolute path>`, and a stop the guard blocks answered with the whole answer again
  Proof: `scripts/audit.py` clean on each agent; a search shows the rules in one file
- [x] Give every reviewer the structured finding with its effect, its `where` naming the defect's site first — for an omission, the new code that calls for it —, and the report writer's checks of their answers
  Proof: phase 2's tests on an answer from each agent's format
- [x] Rewrite `review-security` by phase 0's method
  Proof: `scripts/audit.py` clean on the skill whose evals list it; the `roadmap-close` case's run moved to phase 4's first task
- [x] Rewrite `review-verifier`'s product: the red test and the settled fix in `repro/`
  Proof: `findings.py`'s tests on a verifier's answer, its leads read and its verdicts left to the session; the `roadmap-close` case's run moved to phase 4's first task
- [x] Write the dependency axis's agent or section, by phase 0's decision, judging the records of phase 2's `dependencies.py`, and giving itself llama-swap's and the backend sources' lag behind their newest releases, from the repository's pins and `git ls-remote --tags`
  Proof: `findings.py`'s test on its answer with FACTS, watched failing, then passing; the `dependency-check` case's run moved to phase 4's first task
- [x] Fix the agents' stale references: `review-structure` reads each decision in the document of its domain that `docs/README.md` lists, not in `docs/decisions/`; `review-cleanup` learns a direction under discussion by phase 0's decision, not from `docs/notes/`; an owner for unreachable branches
  Proof: a search of the agents for `docs/decisions` and `docs/notes` finds nothing

### The Repository
- [x] Write in `CLAUDE.md` the review at a phase's close and at a roadmap's close, by phase 0's workflow
  Proof: the user's review of the convention

---

## Technical Details

### Files to Modify
```
.claude/skills/reviewing-delivered-code/SKILL.md
.claude/skills/reviewing-delivered-code/assets/report.md
.claude/agents/review-*.md
.claude/agents/                     (the shared rules file, the dependency agent if any)
CLAUDE.md                           (the review at a phase's and a roadmap's close)
```

### Dependencies
Phase 1's failures and assertions; phase 2's scripts.

### Constraints
- The skill holds only what answers a failure phase 1 observed or a rule the user stated.
- After every write, whatever the tool, `scripts/audit.py` runs and every error and warning is
  fixed.
- Text holding `\u2028` or `\u2029` is written through Python: the Edit and Write tools decode
  their escapes into the characters.
