# Phase 4: Evaluation

---

## Status

**Current Status:** 🟢 Done (100% — 4/4)
**Started:** 2026-11-03
**Completed:** 2026-11-04
**Blocked By:** —

---

## Objective

Show that the v2 removes the failures phase 1 observed without adding others: the same cases
graded against the baseline, iterated until each failure is gone or recorded with its reason, the
description's trigger rates measured, and the skill judged by `skill-auditor`. What phases 5 and 6
prove in real use is not run here: the fix method, and the security anchors in the rules skill.

### Out of Scope
A review of ai-ctl's code, which phase 5 makes. `phase-close`'s v2 run and `roadmap-close`'s rerun,
whose proofs phases 5 and 6 carry.

---

## Tasks

### Runs
- [x] Run the cases on the v2, `authoring-skills` among the evals' `skills`, each batch after the user's yes: one run per case and configuration, three for `cost-gate`, run first, `phase-close` left to phase 6's fixes; with the proofs they carry for phase 3: every case run for `SKILL.md`, `roadmap-close` for `review-security` and `review-verifier`, `dependency-check` for `review-dependencies`
  Proof: high, run — `cost-gate`'s guards holding in each of its runs of the v2, every run ended by `scripts/run.py --status`, and each of phase 3's proofs, its case's result, recorded in the report
- [x] Grade against the baseline, iteration 8's ended runs included, class each failure, and fix it at its source by its severity — a low fix by the static audit and a reading of the diff, a medium or high one by its case run again, once, `cost-gate` three times, or by phases 5 and 6's real use when it proves the fix —, until the v2 beats the baseline clearly and each failure left, carrying no risk, is recorded with its class
  Proof: high, measure — `scripts/benchmark.py`' table for each iteration, the classes in `classes.json`

### Description And Audit
- [x] Measure the new description's trigger rates against phase 1's, one session a query and three for a query whose sessions disagree, and the shared rules skill's, `reviewing-as-an-agent`, on the review requests, which must not trigger it, its trigger queries written first in its `evals/evals.json`
  Proof: high, measure — `scripts/triggers.py`' rates, before and after, recorded in the report
- [x] Have `skill-auditor` judge the skill and the rules skill, one pass each
  Proof: medium, review — `VERDICT: PASS` for each, or each point of a `FAIL` answered at its source, `authoring-skills`' `references/audit.md` running no second pass

---

## Technical Details

### Files to Modify
```
.claude/skills/reviewing-delivered-code/       (fixes from the iterations)
.claude/agents/review-*.md
```

### Dependencies
Phase 3's skill and agents; phase 1's baseline iteration.

### Constraints
- A failure is classed before any change: skill, run variance, grader, assertion, or a gap of the
  validation; a gap goes to the agent setup as a design-feedback note, never around it.
- Each batch of paid runs starts after the user's yes.
- GitHub answers 60 requests an hour to this machine's address without a token, and
  `dependencies.py` spends about 20 on ai-ctl's crates: the `dependency-check` case's runs share
  them, a fourth run within the hour finding its records `rate limit reached`.
- The fixtures carry no `.claude/`: a run's folder needs the project's `.claude/settings.json`, which
  wires `scripts/guard.py`, for the v2's agents to run guarded, while the baseline's skill holds no
  guard to wire.
