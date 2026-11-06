# Phase 2: Dead Code & Tests

---

## Status

**Current Status:** 🟢 Done (100% — 17/17)
**Started:** 2026-10-16
**Completed:** 2026-10-16
**Blocked By:** —

---

## Before Starting This Phase

Read `phase-1-bugs-and-error-handling.md` and `phase-1-bugs-and-error-handling-report.md` in full before
touching anything here: the decisions already taken, the problems and
deviations recorded, and the changes made to this phase.

---

## While Working

Keep `phase-2-dead-code-and-tests-report.md` current as the work happens — after each significant
step, and before every commit, pause, or end of session.

---

## Objective

- The API that only tests call is removed, and the power-profile probe asks for one service
  name.
- The review's test gaps are closed: each new test passes on the code and fails under the
  mutant its finding cites.
- Doctor reads the GPU once and reports what it cannot judge (findings 103, 109).
- L1, the first lead of pass 5, is verified.

---

## Overview

### Why This Phase Matters
Dead code goes first, so that no later batch tests or moves code that is about to be removed.
The test gaps then form the net under phases 3 and 4, which rewrite adapters and split ports.
The review wrote 52 mutants that every current test lets pass.

### What It Enables
Phases 3 and 4 change code that tests catch the review's mutants on. Batches 18, 19 and 21 add
failure switches to the fakes, which batches 40 and 41 then split with their ports.

### Out of Scope
- `cargo mutants` over the whole crate: 1,907 mutants, about 86 minutes. The review's mutants
  are this phase's measure.
- Batch 26, with P3, which moved up to phase 1.

---

## Tasks

### Dead And Legacy Code
- [x] Batch 14 (findings 36–45): remove the API that only tests call, and rewrite each test on what production reads. The items: `Inventory::model_files`, `Build::cuda_architectures`, `Summary::relative_spread`, `Summary.count` and `.max`, `GpuSummary.readings`, `Toolchain::nvcc` and `::c_compiler`, `Sha256::as_str`, `Corpus::len` and `::is_empty`, `Timestamp::unix`, `CommandLine::words`
- [x] Batch 15 (finding 46): the power-profile probe asks `org.freedesktop.UPower.PowerProfiles` alone. `is_service_absent` and its `Probed::Missing` arm stay, now tested. The assertion of "not provided by any .service files" is rewritten with the UPower name, or the wording is dropped

### Gateway Process And Store Tests
- [x] Batch 16 (findings 66, 67, 68): the tests send `Signal::Kill` to a `sleep 60`, show that a running gateway whose ELF binary was deleted reads as `Running`, and check the child's process group through `/proc/$$/stat`. Finding 67's test exists since phase 1's batch 6, `a_gateway_run_through_a_linked_directory_runs_until_it_exits`, and fails under the mutant `liveness_deleted_binary_not_running`; the batch adds none for it
- [x] Batch 17 (finding 65): `builds()` is asserted while a `current` link exists

### Lifecycle And Doctor Tests
- [x] Batch 18 (findings 72, 69, 70, 71): failure switches on `FakeProcesses` and `FakeContainers`. The tests assert the warnings, `up` stopping before `start()` when a pull fails, a log longer than one `LOG_CHUNK`, a health error in the timeout message, and a GPU lost during the release wait
- [x] A record that cannot be removed once its gateway is gone is a warning, not the command's error: `down` after the stop goes on to wait for the GPU's memory, and a stale record found by `running()` reads as no gateway. Found while writing batch 18, approved by the user on 2026-10-16. Two tests on `forget_fails`, watched failing first
- [x] Batch 19 (findings 73, 98, 74, 75, 88, 89): failure switches on the read-only fakes, and one table-driven `Probed::Failed` test per doctor and toolchain section. The tests also cover the remedy for a model without `repo`, doctor's exit status through `tests/cli.rs`, and the 64 MiB and minimum-version boundaries
- [x] Batch 53 (findings 103, 109):
  - `Doctor::run` reads one `Probed<Gpu>` and passes it to `inspect`/`observe` and to `prepare`;
  - `Serving.current` carries its reason, and `gateway_check` warns on `Unknown` with `ai-ctl status` as the remedy;
  - the lifecycle's warnings are collected and shown, and the false comment is removed
- [x] L1: verify with a test whether doctor reports every check `Ok` while `up` fails, given an inventory that `generate` refuses (`${HOME}` in a flag) and no gateway running. If confirmed, a check reports the refusal. If refuted, the report records it

### Build Tests
- [x] Batch 20 (findings 77, 76, 79, 92, 93): failure switches on the builder's store and cmake fakes. The tests cover:
  - the discard and clean warnings, and "is installed but not current";
  - no replacement without `--force`, and `--jobs` reaching cmake;
  - the exact refusal for a three-step remedy, and `summary()` on seven lines and on none;
  - a failing `--list-devices`
- [x] Batch 21 (finding 78): `gateway install` is tested with `publish`, `discard` and `set_current` failing
- [x] Batch 22 (findings 80, 34): the cmake program becomes a field, and `compile()` is tested with a script. A Makefile-generator build is captured as `tests/fixtures/make-build-excerpt.txt` for `parse_progress`
- [x] Batch 23 (findings 95, 96, 81, E4): `SystemHost::cuda_toolkit` is run on a temporary `cuda` link. A `g++` that links to `g++-N` counts once. An older default `g++` comes before an accepted `g++-N`. `parse_cdi_driver`'s hand-written JSON case is either captured on this machine or dropped

### Bench And Inventory Tests
- [x] Batch 24 (findings 82, 83, 91, 94):
  - `DraftCheck::agrees` with only the accepted drafts differing;
  - a hardware slowdown through the summary, the run store and `render_run`;
  - runs at limits of different sizes compared;
  - a fake tokenizer that overshoots every attempt
- [x] Batch 25 (finding 84): the request bodies and paths of `pieces()` and `count()` are asserted on the tokenize fixtures
- [x] Batch 27 (finding 87): a matrix input fits the budget exactly: `[("a", 20000), ("b", 3051), ("c", 5000)]` gives `["a & b", "b & c"]`

### Podman
- [x] Batch 28 (findings 97, 33): the podman program becomes a field built in `main.rs`. The adapter's methods are tested with scripts that exit 1 silently, exit 1 with stderr, or are absent. `parse_digest` and `parse_pids` are tested on outputs captured on this machine

---

## Technical Details

### Files to Modify
```
src/domain/inventory.rs
src/domain/build.rs
src/domain/stats.rs
src/domain/bench.rs
src/domain/gateway.rs
src/domain/prompt.rs
src/domain/timestamp.rs
src/domain/gateway_config.rs
src/domain/machine.rs
src/application/lifecycle.rs
src/application/doctor.rs
src/application/toolchain.rs
src/application/build.rs
src/application/gateway.rs
src/application/bench.rs
src/application/gateway_config.rs
src/application/fakes.rs
src/infrastructure/host.rs
src/infrastructure/gateway_process.rs
src/infrastructure/build_store.rs
src/infrastructure/gateway_store.rs
src/infrastructure/inventory_file.rs
src/infrastructure/cmake.rs
src/infrastructure/server_api.rs
src/infrastructure/run_store.rs
src/infrastructure/podman.rs
src/presentation/lifecycle.rs
src/presentation/compare.rs
src/presentation/bench.rs
src/main.rs
tests/cli.rs
tests/fixtures/
```

### Dependencies
Phase 1 closed. Batch 53 and L1 use the failure switches of batches 18 and 19. Batch 24's
slowdown tests take the `policy` field that batch 54 added in phase 1.

### Constraints
- Each test finding has a mutant in the review's Mutants list (`file`, `old`, `new`). Its new
  test passes on the code and fails under that mutant, applied by exact replacement in a
  scratch copy of the tree.
- When an earlier batch changed a mutant's code, the mutant is applied to the new code, and the
  report records how `old` was adapted.
- Batch 14 removes the code of the mutant `relative_spread_signed`. After batch 15,
  `power_profile_no_fallback` tests the `Probed::Missing` arm.
- No test depends on timing, a GPU or the network. Finding 66's test kills a plain `sleep`
  without a trap. Finding 67's test copies a real ELF binary under `command::exclusive()`.
- New fixtures are captured on this machine, and the report records the command for each.
- The failure switches of batches 18, 19 and 21 are written so that batches 40 and 41 can split
  them with their ports.

---

## Acceptance Criteria

- [x] `cargo test` fails under each mutant that the phase's findings cite, run in a scratch copy, except the one whose code batch 14 removes
- [x] Finding 36's compiler check reports none of the items that batch 14 removes. The check runs on a copy with every `pub` made `pub(crate)`
- [x] Doctor reads the GPU once per run: a test with a `ScriptedGpu` whose second reading differs still gives an `Ok` Idle VRAM
- [x] L1 is either confirmed and fixed, or refuted, and the report records which
- [x] The five checks of CLAUDE.md are green
