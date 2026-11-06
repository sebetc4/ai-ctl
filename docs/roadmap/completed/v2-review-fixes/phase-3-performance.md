# Phase 3: Performance

---

## Status

**Current Status:** 🟢 Done (100% — 7/7)
**Started:** 2026-10-16
**Completed:** 2026-10-17
**Blocked By:** —

---

## Before Starting This Phase

Read `phase-2-dead-code-and-tests.md` and `phase-2-dead-code-and-tests-report.md` in full before
touching anything here: the decisions already taken, the problems and
deviations recorded, and the changes made to this phase.

---

## While Working

Keep `phase-3-performance-report.md` current as the work happens — after each significant
step, and before every commit, pause, or end of session.

---

## Objective

The plan's performance batches are fixed:
- `up`'s and `down`'s waits are bounded by a deadline;
- `rpm` lists the packages without checking their digests;
- NVML is initialised once per process;
- commands are waited for without a sleep loop, and the deadline also bounds a grandchild
  (E1);
- the newest tag is read from the clone, the redundant fetch is removed, and git's minimum
  follows what the code runs (E3, E5);
- GPU owners are found by walking up the process tree;
- the bench fits its prompts once, before the GPU windows.

---

## Overview

### Why This Phase Matters
- `doctor` spends about half its time on rpm's digest checks, and 13 % on sleep loops.
- A gateway that accepts connections and never answers holds `up` for about 630 s, not 30.
- `down` loads NVML at each poll of its release wait.
- A grandchild that keeps a pipe open outlives a probe's timeout.

### What It Enables
Batch 40, in phase 4, splits `GatewayProcesses` once `parent` has replaced `descendants`. Batch
45, also in phase 4, builds on batch 31's NVML.

### Out of Scope
Three paths the review left out, under Coverage:
- the bench monitor's 250 ms sampling, measured with no finding;
- the cost of waking the dGPU from RTD3, which cannot be measured here;
- NVML calls that hang on a wedged GPU.

---

## Tasks

### Waits
- [x] Batch 29 (finding 16): a 30 s deadline from an injected clock bounds `wait_until_healthy` and `wait_for_release`. Each health check gets, for its connect and its read, the time left or a few hundred milliseconds, whichever is shorter. In the tests, the fake `pause` advances the clock
- [x] Batch 32 (finding 19, E1): without a timeout, `execute` waits with `child.wait()`. With one, the deadline bounds the drain threads' EOF and the child's exit, so a grandchild that keeps a pipe open no longer outlives the timeout. A test reproduces E1: a 1 s timeout that returned after 4 s

### Probes
- [x] Batch 30 (finding 17): `rpm -qa --nodigest --nosignature` lists the toolkit packages
- [x] Batch 31 (finding 18): a `static OnceLock` initialises the `Nvml` once per process and shares it with `NvmlMonitor`. A command that never reads the GPU loads no library. The doc comment says that nvmlShutdown is skipped at exit and that NVML's descriptors are close-on-exec
- [x] Batch 34 (findings 21, 108): GPU owners are found by walking each GPU process's parent chain through `/proc/<pid>/stat`, with a loop guard. `podman ps` runs only for a chain that reaches a `conmon` or a process in a `libpod-` cgroup. `GatewayProcesses::parent` replaces `descendants`, with `FakeProcesses` and the `owners()` tests

### Git
- [x] Batch 33 (finding 20, E3, E5):
  - `build` fetches or clones first, then reads the newest release with `git tag --list 'b[0-9]*'`; `CheckedOut` names the reference, and its snapshot is updated;
  - `resolve`'s explicit fetch of a commit hash is removed, and its doc comment corrected;
  - git's minimum and its comment follow what the code runs, checked against git's release notes, and finding 89's boundary test follows the new minimum

### Bench
- [x] Batch 35 (findings 22, 23, 24): the selected scenarios' prompts are fitted before `monitor()`. `Prompts::load` counts the agent's bare prompt once. The set-aside lead is built from one cut, with at most one count

---

## Technical Details

### Files to Modify
```
src/domain/machine.rs
src/application/lifecycle.rs
src/application/bench.rs
src/application/build.rs
src/application/ports.rs
src/application/fakes.rs
src/infrastructure/command.rs
src/infrastructure/http.rs
src/infrastructure/gateway_api.rs
src/infrastructure/host.rs
src/infrastructure/nvml.rs
src/infrastructure/git.rs
src/infrastructure/gateway_process.rs
src/infrastructure/podman.rs
src/presentation/build.rs
src/main.rs
```

### Dependencies
Phase 2 closed: its tests cover the waits, the probes and the owners that this phase rewrites.

### Constraints
- Each change is measured on this machine before and after, the way its finding measured it,
  and the report records both figures.
- No test depends on timing. The deadline tests use the injected clock, after the example of
  `Bench.clock`.
- Batch 35 sends the measured scenarios the same prompts as before: template and tokenize are
  deterministic. The set-aside prompt changes by a few tokens and is not measured, so saved runs
  stay comparable.
- Batch 34 finds every owner that `descendants` found. Orphans reparented to `systemd --user`
  are still caught by the build-store check of `leftovers`.

---

## Acceptance Criteria

- [x] With the injected clock, `up` against a gateway that accepts and never answers fails once the 30 s deadline passes
- [x] The report gives the wall time of `doctor` and `status` on this machine before and after the phase
- [x] `build` without `--ref` runs no `git ls-remote`
- [x] A command whose grandchild keeps its pipe open returns within its timeout
- [x] The five checks of CLAUDE.md are green
