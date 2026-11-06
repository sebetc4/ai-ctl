# Phase 0: Decisions And Probes

---

## Status

**Current Status:** 🔴 Not Started (0% — 0/5)
**Started:** {{START_DATE}}
**Completed:** {{COMPLETION_DATE}}
**Blocked By:** —

---

## Objective

The facts and the contracts the later phases build on: what llama-swap v264 to v270 change for
ai-ctl, probed on this machine, and the capabilities, `status --json`'s document and the two
`build` commands settled with the user and written as this phase's Design.

### Out of Scope
Any change to ai-ctl's code or to the pinned release: phase 1 makes them.

---

## Tasks

### llama-swap
- [ ] The release notes of llama-swap v264 to v270 read: each change to the configuration ai-ctl writes, to the API it calls (`/health`, `/running`, `/api/models/unload`, `/logs/stream/{model}`), to `capabilities` and to `proxy`'s schemes, listed with its release
  Proof: medium, review — the list in the report, each item with its release
- [ ] v270 probed in a scratch folder with the shipped inventory's generated configuration: it starts, `-version` names it, the `capabilities` a model's entry declares appear in `/v1/models` before any load, and whether `proxy` reaches a llama-server listening on a unix socket; each capture kept for phase 1's fixtures
  Proof: high, check — the probe's commands and outputs in the report

### The Contracts
- [ ] The capabilities each kind of model declares from the inventory alone — chat with or without `mmproj`, tools, context, speech, transcription, images in and out — settled with the user
  Proof: medium, review — the table in the Design, the user's yes in the report
- [ ] `status --json`'s document — its fields from what `status` shows, a gateway stopped included, the exit codes unchanged — settled with the user
  Proof: medium, review — the document in the Design, the user's yes in the report
- [ ] `build outdated` and `build update`'s contract — the projects they cover, how the newest release is read (the sources clone's fetch or `git ls-remote`), the exit code of a project behind, the order and the failures of an update, llama-swap's line — settled with the user
  Proof: medium, review — the contract in the Design, the user's yes in the report

---

## Technical Details

### Files to Modify
```
docs/roadmap/on-progress/capabilities-status-and-updates/phase-0-decisions-and-probes.md   its Design
docs/roadmap/on-progress/capabilities-status-and-updates/phase-0-decisions-and-probes-report.md
```

### Dependencies
None.

### Constraints
`docs/gpu/llama-swap.md` and `docs/security.md` read first. The probe downloads v270's archive into
a scratch folder, checks it against the release's `checksums.txt`, and runs it on a port of its
own with a scratch key, the installed gateway and its state untouched; the GPU's models are not
loaded unless a capability can only be seen after a load.
