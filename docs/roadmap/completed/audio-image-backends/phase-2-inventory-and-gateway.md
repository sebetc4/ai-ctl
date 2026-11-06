# Phase 2: Inventory And Gateway

---

## Status

**Current Status:** 🟢 Done (100% — 13/13)
**Started:** 2026-10-28
**Completed:** 2026-10-29
**Blocked By:** —

---

## Before Starting This Phase

Read `phase-1-native-builds.md` and `phase-1-native-builds-report.md` in full before
touching anything here: the decisions already taken, the problems and
deviations recorded, and the changes made to this phase.

---

## While Working

Keep `phase-2-inventory-and-gateway-report.md` current as the work happens — after each significant
step, and before every commit, pause, or end of session.

---

## Objective

The inventory declares whisper.cpp, audio.cpp and stable-diffusion.cpp backends and their models,
and `ai-ctl gateway config` turns each model into a llama-swap entry: its server's command on
the assigned port, its readiness check, ai-ctl's key, and its place in the matrix.

---

## Overview

### Why This Phase Matters
`BackendKind` knows `llama-server` only, and the gateway's configuration passes the key through
`LLAMA_API_KEY`, which only llama-server reads. Every check the inventory applies, written after
a value could make llama-swap run a command on the host, must hold for the new kinds too.

### What It Enables
Phase 3 ships models of every kind in the inventory and serves them through the gateway.

### Out of Scope
The shipped inventory's entries and the live use, in phase 3. Bench scenarios for these kinds.

---

## Tasks

### Inventory
- [x] One backend kind per server, named as phase 0 settles, each with the files its models need, its reserved flags and a `[defaults.<kind>]` table
- [x] Every inventory check holds for every kind: control characters and line separators, underscored flags, an option as a value, non-finite floats, reserved flags; tested per kind
- [x] A backend's `source = { build = "<project>" }` checked against its kind: a whisper.cpp kind built from whisper.cpp

### Gateway Configuration
- [x] Each kind's command: its project's current build, the loopback host, `${PORT}`, the model's files, the kind's defaults and the model's flags; `sd-server` with `--eager-load`, so that ready means its weights are loaded; `whisper-server` with `--inference-path /v1/audio/transcriptions` and `-l auto`, since its default `en` decodes a request without `language` as English
- [x] Each kind's `checkEndpoint` is the `/health` its patch leaves open (`sd-server`'s comes from the patch), and ai-ctl's key reaches its server as `LLAMA_API_KEY` through the entry's `env:`, as for llama-server, never written in the file
- [x] audio.cpp's model configuration, the JSON `audiocpp_server --config` reads: one model per entry, its `id` equal to the gateway's model id, its family, path and task, its `session_options` (Qwen3-TTS's `weight_type`, which makes one set of files two entries), its voices, the UI off, loaded at start
- [x] Each model's port across a restart: probe how llama-swap v263 assigns `${PORT}` (one log of phase 0 suggests by sorted model id, from 5800), then whether each server binds a port another server left in TIME-WAIT, every server binding with `SO_REUSEADDR` since phase 1; only if one cannot, keep a change of the inventory from handing a model's port to another server within TIME-WAIT's 60 s
- [x] `sd-server`'s model routes since `master-1017` (`/sdcpp/v1/models/load`, `/unload`, `/refresh`, `/status`, `/sdcpp/v1/model-files`, `/sdcpp/v1/recipes`): what a client with the key reaches of them through the gateway, so that none loads a model outside llama-swap's arbitration of the VRAM
- [x] The matrix sizes every kind's models by `vram_mib`, so a small model runs beside another when both fit; a snapshot of the configuration for an inventory with several models of each kind

### Integration
- [x] `doctor` checks each model's files and its backend's build for every kind
- [x] `status` and `logs` show the models of every kind
- [x] `doctor` reads a model's safetensors index, such as Fibo's `diffusion-model`, and checks each shard it names
- [x] `docs/commands.md` (the inventory, `gateway config`, `doctor`) and `docs/security.md` (what listens, the key of each server) describe the new kinds

---

## Technical Details

### Files to Modify
```
src/domain/inventory.rs
src/domain/gateway_config.rs
src/infrastructure/inventory_file.rs
src/application/doctor.rs
src/application/gateway_config.rs
src/application/lifecycle.rs
tests/
tests/snapshots/
docs/commands.md
docs/security.md
```

### Dependencies
Phase 0: each server's options, readiness endpoint and key support, captured. Phase 1: each
project's builds, patched to require the key.

### Constraints
- A kind's command, readiness check and key come from phase 0's captures and its run behind
  llama-swap, never from a server's documentation alone.
- No key, in the configuration file or on a command line; llama-swap passes it through the
  environment, as it does for llama-server.
- A value the inventory accepts for a new kind is checked as strictly as a llama-server one.
- A kind accepts any model its server loads: ai-ctl keeps no list of models, nor of audio.cpp's
  families, which the server checks at load, so that a new model needs an inventory entry and
  no change to the code (phase 0's report, Decisions, 2026-10-28).
- Each commit's code diff goes through `cargo mutants --in-diff`; a new test that spawns a
  process, binds a port or takes a lock runs 30 times before its commit.

---

## Acceptance Criteria

- [x] `ai-ctl gateway config`, on an inventory with several models of each kind, writes a configuration the installed llama-swap loads, with no key in it
- [x] An inventory that breaks a rule of a new kind is refused by `doctor` and by `up`, with every broken rule listed
- [x] The five checks pass
