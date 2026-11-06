# Phase 3: status --json

---

## Status

**Current Status:** 🔴 Not Started (0% — 0/5)
**Started:** {{START_DATE}}
**Completed:** {{COMPLETION_DATE}}
**Blocked By:** —

---

## Objective

`ai-ctl status --json` prints phase 0's document for scripts and clients, as `doctor --json` does
for the diagnosis; the text and the exit codes unchanged.

### Out of Scope
A JSON output for the other commands. A desktop panel reading it.

---

## Tasks

### The Document
- [ ] The document rendered from `status`'s result in the presentation layer, test-first, a gateway running with its models and a gateway stopped
  Proof: medium, test — each case's test watched failing then passing
- [ ] `--json` on `status`, its exit codes those of the text: 0 while the gateway runs, 3 when it does not
  Proof: medium, test — the stopped gateway's exit code and document tested

### The Use
- [ ] On this machine, `status --json` with the gateway up then down, each output read by `jq`
  Proof: low, use — both outputs in the report

### Closing
- [ ] The documents this phase's changes describe at their current state: `docs/commands.md`'s `status`, with the document's fields; the phase's line in 0.3.0's entry
  Proof: medium, review — each document changed, named in the report
- [ ] The review of the phase's hunks by `reviewing-delivered-code`, its findings fixed in the phase
  Proof: medium, review — the review's report, each finding fixed or registered

---

## Technical Details

### Files to Modify
```
src/presentation/cli.rs
src/presentation/lifecycle.rs
CHANGELOG.md
docs/commands.md
```

### Dependencies
Phase 0's Design: the document. Phase 2 closed, so that the roadmap's phases close in order.

### Constraints
Presentation never imports infrastructure. Code test-first; `cargo mutants --in-diff` on each
commit's code.
