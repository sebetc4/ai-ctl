# Phase 4: Structure & Refactoring

---

## Status

**Current Status:** 🟢 Done (100% — 16/16)
**Started:** 2026-10-17
**Completed:** 2026-10-17
**Blocked By:** —

---

## Before Starting This Phase

Read `phase-3-performance.md` and `phase-3-performance-report.md` in full before
touching anything here: the decisions already taken, the problems and
deviations recorded, and the changes made to this phase.

---

## While Working

Keep `phase-4-structure-and-refactoring-report.md` current as the work happens — after each significant
step, and before every commit, pause, or end of session.

---

## Objective

The plan's structure, refactoring and documents batches are done:
- the rules for comparing runs live in the domain;
- presentation builds the shared use cases from `Services`;
- the compile port is named after its role;
- the build gate's refusal is an `anyhow` error;
- `GatewayProcesses` and `HostProbe` are split by role;
- saved runs keep their own copy of the build's fields (E2);
- each copy the review found exists once;
- `docs/architecture.md`, `main.rs` and CLAUDE.md describe the final structure.

---

## Overview

### Why This Phase Matters
Two ports hold five and three responsibilities. `Bench` depends on 13 methods and calls three.
One rule lives in two or three places with drifting texts. The document that defines the layers
says main.rs wires the use cases, but presentation does. Every saved run would become unreadable
if `build.json`'s format changed.

### What It Enables
The roadmap closes on a structure that its documents describe, and phase 5 reviews it.

### Out of Scope
- Findings 99 and 100, kept by the review: their shared code would cost what it saves.
- Changes of behaviour, except the texts that batch 48 unifies.

---

## Tasks

### Structure
- [x] Batch 36 (findings 30, 59): `domain/bench.rs` gains `Run::scenarios`, the comparable counters, and the GPU summary over common scenarios, filtering on `Scenario` values. `added` and `render_run` both use `OverheadResult::medians`
- [x] Batch 37 (findings 61, 29): `Services::configurator` builds the `Configurator`, and doctor takes its `Lifecycle`, or its `pause`, from `Services`
- [x] Batch 38 (finding 31): the compile port is renamed after its role and its adapter after CMake, along with `Builder`'s field, `Services`, `main.rs` and the tests' double
- [x] Batch 39 (finding 32): the build gate builds its refusal with `bail!`. The `Refused` type goes, and its two tests assert on the message
- [x] Batch 40 (finding 25): `GatewayProcesses` is split by role into the gateway's state, the process table and the port check. `LocalGateway` implements all three, `Bench` holds one port, and the fakes are split to match
- [x] Batch 41 (finding 26): `HostProbe` is split into toolchain, CDI and power probes. `SystemHost` implements all three, `Bench` holds the power probe alone, and `FakeHost` is split to match
- [x] E2: the saved run's format keeps its own copy of the build's fields instead of embedding `build.json`'s format. Saved runs still read

### Refactoring
- [x] Batch 42 (findings 47, 48): podman's `checked` moves to `command.rs` for the `RunError` arms. `Tool::install` gives the install command of CMake, Git, Podman and nvidia-ctk, for the checks and for the adapters' messages. Phase 2's batch 28 tests each podman method without podman against `MISSING`; those assertions follow the message `Tool::install` gives
- [x] Batch 43 (finding 49): `http.rs` gains `request_ok` and a `send` variant for the three HTTP adapters. `healthy` keeps its `Option`, and every error goes through `error_message`
- [x] Batch 44 (finding 50): `application::build` looks up a project's current build in one place, with a three-way outcome (`None`, `Missing`, `Installed`) for doctor, bench and the configurator
- [x] Batch 45 (finding 51): one function turns a GPU reading (`Probed<Gpu>`) into a `Result<Gpu>` with a common base text, and each site adds its own context. Phase 2's batch 53 replaced the lifecycle's `read_gpu` with `warn_unread`, which takes a reading, and gave the configurator `prepare_on`, which takes one too
- [x] Batch 46 (finding 52): `log`, `model_log` and `Bench::listen` share `live_record`, which `build prune` already uses (phase 1, batch 8)
- [x] Batch 47 (finding 53): the inventory and both use cases call `domain::inventory::check_id`, which returns `InventoryError::InvalidId`
- [x] Batch 48 (findings 56, 57, 58): doctor and presentation share `Liveness::describe`, `Serving::address`, `Serving::models_list`, and a public `named` for the leftover GPU processes. The texts and their tests are updated
- [x] Batch 49 (findings 54, 55, 60, 63, 64): small shared helpers: `newest_first`, `recipes`, `CommitId::short`, one `fix`, and `current_line`

### Documents
- [x] Batch 50 (findings 28, 35): `docs/architecture.md`, `main.rs`'s module doc and CLAUDE.md's Layers line state the facts:
  - where the use cases are assembled;
  - that presentation picks the exit code;
  - which use case each command goes through;
  - the real port names and fixtures;
  - the Repository Layout's missing entries: `docs/security.md`, `docs/reviews/`, `tests/fixtures/` and its index, `tests/snapshots/`, `.claude/skills/`, and `docs/commands.md` and the Makefile once the user has committed them;
  - the testing conventions phases 1 and 2 settled: failure switches named `<method>_fails`, adapters run on scripts under `command::exclusive()`, `ScriptedGpu`, and a test finding's mutant as its proof

---

## Technical Details

### Files to Modify
```
src/domain/bench.rs
src/domain/build.rs
src/domain/inventory.rs
src/domain/machine.rs
src/application/ports.rs
src/application/fakes.rs
src/application/lifecycle.rs
src/application/doctor.rs
src/application/toolchain.rs
src/application/build.rs
src/application/bench.rs
src/application/gateway_config.rs
src/infrastructure/command.rs
src/infrastructure/git.rs
src/infrastructure/gateway_store.rs
src/infrastructure/download.rs
src/infrastructure/gateway_process.rs
src/infrastructure/podman.rs
src/infrastructure/cmake.rs
src/infrastructure/host.rs
src/infrastructure/http.rs
src/infrastructure/server_api.rs
src/infrastructure/gateway_api.rs
src/infrastructure/chat_api.rs
src/infrastructure/run_store.rs
src/presentation/mod.rs
src/presentation/compare.rs
src/presentation/bench.rs
src/presentation/build.rs
src/presentation/gateway.rs
src/presentation/lifecycle.rs
src/main.rs
docs/architecture.md
CLAUDE.md
```

### Dependencies
Phase 3 closed. In the plan's order:
- batch 40 splits the port after batch 34's `stat` and `in_container`;
- E2 comes after batch 41;
- batch 43 builds on phase 0's `Authorization` header;
- batch 45 builds on batch 31's NVML and takes the reading that batch 53 passes around;
- batch 50 comes last.

### Constraints
- The structure batches change no behaviour. Tests change only for the names and the doubles
  they move. Batch 48's texts are the exception, as its finding says.
- Presentation imports no infrastructure. A port exists only where two implementations exist
  or a use case needs a test double (CLAUDE.md).
- E2 keeps every saved run of this machine readable.

---

## Acceptance Criteria

- [x] Each port in `ports.rs` holds one responsibility, and `Bench` depends only on the ports it calls
- [x] A search for each copy the refactoring findings cite finds one definition
- [x] `bench list` and `bench compare` still read every saved run of this machine
- [x] `docs/architecture.md`, `main.rs`'s module doc and CLAUDE.md match the code on each statement of findings 28 and 35
- [x] The five checks of CLAUDE.md are green
