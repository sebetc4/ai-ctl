# Phase 5: Closing Pass

---

## Status

**Current Status:** 🔴 Not Started (0% — 0/3)
**Started:** {{START_DATE}}
**Completed:** {{COMPLETION_DATE}}
**Blocked By:** —

---

## Objective

The roadmap's closing pass by `reviewing-delivered-code` — the security pass, the dependency axis,
the recorded mutants run again —, its findings fixed, and 0.3.0's entry ready for the release that
follows the integration.

### Out of Scope
The release itself, made on `main` after the integration. A review after the closing pass.

---

## Tasks

### The Pass
- [ ] The closing pass of `reviewing-delivered-code`, scoped from the roadmap's creation to the end of phase 4
  Proof: high, review — the pass's report and the regression run's output
- [ ] The pass's findings that bear on this roadmap's changes or are medium or worse fixed in this phase, the others registered; no review after it
  Proof: high, test — each fix's red test seen failing first; the register's entries

### Closing
- [ ] 0.3.0's entry in `CHANGELOG.md` complete, one line per phase, read by the user, left `unreleased` for the release after the integration
  Proof: low, review — the user's yes on the entry in the report

---

## Technical Details

### Files to Modify
```
docs/reviews/delivered-code/
CHANGELOG.md
```

### Dependencies
Phase 4 closed.

### Constraints
The pass's paid sessions start after the user's yes, each with its estimate. Each fix test-first,
`cargo mutants --in-diff` on its code.
