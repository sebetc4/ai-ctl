# Phase 5: First Real Use

---

## Status

**Current Status:** 🟢 Done (100% — 7/7)
**Started:** 2026-11-04
**Completed:** 2026-11-05
**Blocked By:** —

---

## Objective

Review `audio-image-backends`' code with the v2, on its roadmap's diff by hunk and on every axis,
the dependencies included: one report written by the report writer, its findings triaged by the
session and the hard cases put to the user, a fix plan for phase 6.

### Out of Scope
The fixes, which phase 6 makes. A full review of the crate.

---

## Tasks

### Review
- [x] Set the scope with `scope.py` on `audio-image-backends`, by hunk
  Proof: the script exits 0; the lines handed to the reviewers recorded beside those of a scope by file
- [x] Put one go-ahead to the user, the agents counted and an order of magnitude, then run the checks and start the mutants in the background
  Proof: the report's Checks section, the mutants' result included
- [x] Run the reviewers, the dependency axis included, and the verification
  Proof: each agent's answer accepted by the report writer's checks
- [x] Write the report with the report writer, its summary table at the top, the findings below the floor in the debt register
  Proof: the report in `docs/reviews/delivered-code/`, and the register's entries

### Triage
- [x] Give the clear verdicts, put the hard cases to the user, and write the fix plan
  Proof: every finding with a verdict and a batch; the user's answer on each hard case

### The Skill In Use
- [x] Record each failure of the skill seen in this use in its evals' `failures_in_use`, with its source, and fix it at the line that let it through
  Proof: `scripts/audit.py` clean after each fix
- [x] Check in the agents' answers what phase 4 left to this use: `review-security` and `review-verifier` rating each security finding by the security anchors of the preloaded rules, and each agent's answer in its format at its first submission
  Proof: each security finding's severity, its anchor and the verifier's verdict on it, and each refused answer with its fault, recorded in the report

---

## Technical Details

### Files to Modify
```
docs/reviews/delivered-code/                   (the report, the repro/ folder, the debt register)
.claude/skills/reviewing-delivered-code/       (fixes from use)
```

### Dependencies
Phase 4's skill. llama-swap, `cargo-mutants`, `cargo-deny` and the dependency axis's script
installed; network access to crates.io.

### Constraints
- The agents run only what phase 3's shared rules allow; a probe they may not run goes through the
  `PROBES` channel to the user.
- No code of ai-ctl is changed in this phase.
- The session runs no ai-ctl command that writes the stack's state, such as `gateway prune`, without
  the user's yes: a v2 session ran one in an eval run, and phase 4's watch on it ended with no
  `phase-close` run.
- A request for a security review of pending changes, the built-in `/security-review`'s, triggered
  the skill in phase 4's measure: each false trigger of the skill met in this use goes to its evals'
  `failures_in_use`, and a narrower "security pass alone" is measured on every trigger query.
