# Phase 1: Bugs & Error Handling

---

## Status

**Current Status:** 🟢 Done (100% — 16/16)
**Started:** 2026-10-16
**Completed:** 2026-10-16
**Blocked By:** —

---

## Before Starting This Phase

Read `phase-0-security.md` and `phase-0-security-report.md` in full before
touching anything here: the decisions already taken, the problems and
deviations recorded, and the changes made to this phase.

---

## While Working

Keep `phase-1-bugs-and-error-handling-report.md` current as the work happens — after each significant
step, and before every commit, pause, or end of session.

---

## Objective

The plan's bug and error-handling batches are fixed. With them come P1 to P4, the defects found
outside the review that join them, and L2, the second lead of pass 5, once verified.

---

## Overview

### Why This Phase Matters
In these findings, ai-ctl does the wrong thing or misleads the user:
- it loses track of a gateway that still runs (2, 10, P4);
- it deletes the build the gateway serves (9);
- it loses measured runs (11, 12, 15);
- it refuses a bench that fits (8), and accepts one that overflows (P2);
- doctor's CUDA remedies contradict each other (101, 102);
- the bench hides throttling and lost GPU readings (104, 105).

### What It Enables
Phase 2 writes its tests against the corrected behaviour. Three pieces are ready for later
batches: batch 10's `FakeRuns`, finding 62's `read_link_if_present`, and batch 13's publish
helper.

### Out of Scope
- Findings 103 and 109, and L1, the first lead of pass 5. They go to phase 2 with batch 53,
  since their tests use the failure switches of batches 18 and 19.
- The bench's plan defaults, left open by the v2 rebuild.

---

## Tasks

### Security Document
- [x] Write `docs/security.md` from phase 0's report. It covers:
  - who can reach the gateway and the backends: web pages, containers, other users, the LAN;
  - what the API key, the loopback address and `expose` guard, and what they do not: code that runs as the user, and a key the browser stored from llama-swap's Basic prompt;
  - why CORS is left as llama-swap sets it: `allowedOrigins` is not access control;
  - the inventory's text, the downloads and the images of batches 2 to 5;
  - the clients that hold the key: why OpenWebUI's container is published on loopback only, and the `unauthorized` lines llama-swap's own requests leave in the log at each model load.

  `CLAUDE.md` and `docs/stack.md` §7 link to it

### State Directory
- [x] ai-ctl creates its state directory, and each directory under it, at mode 0700, so that the gateway's log and configuration stay the user's whatever the home directory's mode. One helper in `files.rs`, and a test for each place that creates one: the gateway's lock, record and log, the build log, the API key. This machine's existing directory is narrowed by hand, and `docs/security.md` §8 says what the code guarantees

### Gateway Records And Builds
- [x] Batch 6 (finding 2): `liveness_of` compares `/proc/<pid>/exe` with the canonicalized binary. For a removed file, the parent is canonicalized, then the file name and " (deleted)" are added. `FsBuildStore::contains` compares with the canonicalized root. The record is unchanged. A test uses a symlinked data directory
- [x] Batch 8 (finding 9): `build prune` refuses to remove a build whose binary the running gateway's configuration names, and names `ai-ctl down`; a test
- [x] Batch 9 (finding 10, P4): `up` forgets a gateway that never answered only when `stop` returned `Ok` or `started.exited()` reports an exit. An error from `started.exited()?` no longer returns before the stop. The test asserts the kept record and the whole message

### Bench
- [x] Batch 7 (findings 8, 90, P2): `Plan::new` takes the selected scenarios, so `--scenario depth` runs on a 16K context. `SessionPlan::peak` counts the `TURN` preamble and the template's tokens. Finding 90's boundary test is written against the new plan and the new peak
- [x] Batch 10 (findings 27, 11, 12): saving moves into the bench use case, through `RunStore` with a `FakeRuns`. When a scenario fails, the run keeps the completed scenarios, is saved, and the error names the saved id. The run is saved before it is printed. Tests cover the save, `saved` with an unreadable run, and `load`
- [x] Batch 54 (finding 104): `GpuReading` keeps NVML's raw clock-event mask, and `Slowdown` gains a `policy` field with its own column. Runs saved before still read. BoardLimit and Reliability count as a slowdown in the summary only once a probe under load on this machine has shown what they report
- [x] Batch 55 (finding 105): `finish()` returns the number of failed GPU readings and the last error, `Bench::run` emits a `Warning`, and `render_gpu` shows a scenario without readings as such. A test uses a `FakeMonitoring` that fails

### Stores, Links And Remedies
- [x] Batch 11 (finding 13): `init` names a dangling `inventory.toml` link and its target when it refuses
- [x] Batch 12 (findings 62, 14): `read_link_if_present` in `files.rs` serves both stores. `build`, `build use`, `build list` and `gateway install` read a bad `current` link leniently, with a warning, and replace it. `prune` and `remove` stay strict
- [x] Batch 13 (finding 15): the run store publishes a run with `files::write_new`, the helper batch 1 moved from the inventory store into `files.rs`, with a mode. The helper writes, makes a hard link, always removes the temporary file, and maps AlreadyExists
- [x] P1: doctor's remedy for a current llama-swap release whose binary is missing ends in a working install, not in `publish`'s refusal (`doctor.rs:156-160`, `gateway_store.rs:148`)

### Doctor's Toolkit Remedies
- [x] Batch 52 (findings 101, 102): a toolkit newer than the driver gets a remedy of its own: install the driver's toolkit if it is missing, then remove the newer one so that `alternatives` falls back. The packages check pins the driver's release whenever the toolkit check refuses the installed one. Tests use toolkits 13.2 and 13.5

### Inventory Checks
- [x] Batch 26 (findings 85, 86, P3): a dashed key in `files` is refused, and each reserved flag is tested. A model's file keys are checked against the reserved flags, and the defaults against the model's file flags: `files = { port = "x" }`, and a default `mmproj` beside `files.mmproj`

### Command Errors
- [x] L2: capture podman's stderr under a storage driver it refuses (`CONTAINERS_STORAGE_CONF`), and verify whether `Output::failure()` shows podman's logrus line instead of its `Error:` line. If confirmed: a fixture, a test, and the fix at `command.rs:33-39`. If refuted: recorded in the report

---

## Technical Details

### Files to Modify
```
src/domain/bench.rs
src/domain/prompt.rs
src/domain/build.rs
src/domain/inventory.rs
src/application/lifecycle.rs
src/application/build.rs
src/application/bench.rs
src/application/gateway.rs
src/application/doctor.rs
src/application/toolchain.rs
src/application/init.rs
src/application/ports.rs
src/application/fakes.rs
src/infrastructure/gateway_process.rs
src/infrastructure/build_store.rs
src/infrastructure/gateway_store.rs
src/infrastructure/files.rs
src/infrastructure/inventory_file.rs
src/infrastructure/run_store.rs
src/infrastructure/nvml.rs
src/infrastructure/command.rs
src/presentation/bench.rs
src/presentation/build.rs
tests/fixtures/
```

### Dependencies
Phase 0 closed. Within the phase, batch 12 builds on finding 62's `read_link_if_present`, and
P1 comes after batch 13, per the plan.

### Constraints
- A bug fix starts with the test that reproduces it, watched failing on its assertion.
- Batch 10 saves in the application layer through the `RunStore` port. Presentation imports no
  infrastructure (CLAUDE.md, Layers).
- A failed request still ends `bench run`, as the v2 rebuild decided (its phase 4 report). The
  run saves only the completed scenarios.
- Batch 54's probe under load is a `bench run` on this machine's GPU. The report records what
  NVML reported during it.

---

## Acceptance Criteria

- [x] Each batch of the phase has a test that failed on its assertion before its fix
- [x] `bench list` and `bench compare` still read every saved run of this machine
- [x] With toolkits 13.2 and 13.5, the "CUDA toolkit" and "Toolkit packages" remedies agree, in tests
- [x] L2 is either confirmed and fixed, or refuted, and the report records which
- [x] The five checks of CLAUDE.md are green
