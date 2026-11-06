# Phase 1: llama-swap v270

---

## Status

**Current Status:** 🔴 Not Started (0% — 0/7)
**Started:** {{START_DATE}}
**Completed:** {{COMPLETION_DATE}}
**Blocked By:** —

---

## Objective

ai-ctl pins llama-swap v270, each change of phase 0's list that bears on it carried into the code,
the gateway proven end to end on this machine; `Cargo.toml`'s `rust-version` checked by a build;
0.3.0's entry opened.

### Out of Scope
The capabilities, phase 2. The backends on unix sockets.

---

## Tasks

### The Pin
- [ ] ai-ctl pins v270: its archive's SHA-256, its checksums file and its `-version` output, each parser tested first on v270's captured output, listed in `tests/fixtures/README.md`
  Proof: high, test — the pin's tests watched failing on v263's values then passing; `gateway install` run on this machine
- [ ] Each change of phase 0's list that bears on the configuration ai-ctl writes or the API it calls carried into the code test-first, the shipped configuration's snapshot updated; `tests/gateway_lifecycle.rs` green against v270
  Proof: high, test — each change's test watched failing then passing; the lifecycle test run, not skipped
- [ ] The gateway end to end on this machine on v270 with the shipped inventory: `up`, a chat, a transcription, a speech, an image, `status`, `logs`, `down`
  Proof: high, use — each step's output in the report

### Rust
- [ ] `Cargo.toml`'s `rust-version` checked by a build: set to the toolchain `rust-toolchain.toml` pins, or checked by the CI at the version it names, chosen and argued in the report
  Proof: low, check — the build that checks it, green

### Closing
- [ ] `CHANGELOG.md`'s entry `## 0.3.0 — unreleased` opened with the phase's line, `Cargo.toml` at 0.3.0
  Proof: low, check — the entry and the version
- [ ] The documents this phase's changes describe at their current state: `docs/security.md` §7, `docs/gpu/llama-swap.md`, `docs/stack.md` §6.4, `docs/commands.md`'s `gateway install`, `docs/release.md` if the CI's install of llama-swap changes
  Proof: medium, review — each document changed, named in the report
- [ ] The review of the phase's hunks by `reviewing-delivered-code`, its findings fixed in the phase
  Proof: medium, review — the review's report, each finding fixed or registered

---

## Technical Details

### Files to Modify
```
src/domain/gateway.rs
src/domain/gateway_config.rs
src/infrastructure/gateway_api.rs
tests/fixtures/
tests/snapshots/gateway-shipped.yaml
tests/gateway_lifecycle.rs
Cargo.toml
CHANGELOG.md
docs/security.md
docs/gpu/llama-swap.md
docs/stack.md
docs/commands.md
```

### Dependencies
Phase 0 closed: its list of v264 to v270's changes and its captures.

### Constraints
`docs/security.md` read before the pin changes: the archive checked by SHA-256 from a source named
in the report. Code test-first; each parser on an output a probe captured; no test depends on the
network, a GPU or timing; `cargo mutants --in-diff` on each commit's code.
