# Phase 2: Capabilities

---

## Status

**Current Status:** 🔴 Not Started (0% — 0/5)
**Started:** {{START_DATE}}
**Completed:** {{COMPLETION_DATE}}
**Blocked By:** —

---

## Objective

The generated llama-swap configuration declares each model's capabilities, computed from the
inventory by phase 0's table, so that a client picks a model by what it can do from `/v1/models`,
before any load, with no list of models of its own.

### Out of Scope
A field of the inventory for a capability the kind does not tell, unless phase 0's table needs
one. The client's choice between two models with the same capabilities.

---

## Tasks

### The Configuration
- [ ] Each model's capabilities computed in the domain from its kind and its entry, by phase 0's table, test-first on each kind of the inventory
  Proof: medium, test — a test per kind watched failing then passing
- [ ] The capabilities written into each model of the generated configuration, the shipped configuration's snapshot updated
  Proof: medium, test — the snapshot test watched failing then passing
- [ ] On this machine, `/v1/models` through the gateway lists each shipped model with its capabilities, before any model is loaded
  Proof: medium, use — the response in the report

### Closing
- [ ] The documents this phase's changes describe at their current state: `docs/gpu/llama-swap.md` — a client picks a model by its capabilities, and the agents' client described as a separate project's command-line tool in place of an MCP server —, `docs/commands.md`'s `gateway config`, `docs/security.md` if what `/v1/models` tells changes; the phase's line in 0.3.0's entry
  Proof: medium, review — each document changed, named in the report
- [ ] The review of the phase's hunks by `reviewing-delivered-code`, its findings fixed in the phase
  Proof: medium, review — the review's report, each finding fixed or registered

---

## Technical Details

### Files to Modify
```
src/domain/gateway_config.rs
src/domain/inventory.rs
tests/snapshots/gateway-shipped.yaml
CHANGELOG.md
docs/gpu/llama-swap.md
docs/commands.md
```

### Dependencies
Phase 1 closed: the capabilities are written for v270.

### Constraints
`docs/gpu/llama-swap.md` read first. ai-ctl stays no client: it declares what each model can do,
and never chooses for a client. Code test-first; `cargo mutants --in-diff` on each commit's code.
