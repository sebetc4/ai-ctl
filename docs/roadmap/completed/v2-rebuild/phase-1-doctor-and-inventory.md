# Phase 1: Doctor & Inventory

---

## Status

**Current Status:** 🟢 Done (100% — 16/16)
**Started:** 2026-10-09
**Completed:** 2026-10-09
**Blocked By:** —

---

## Before Starting This Phase

Read `phase-0-reset-and-skeleton.md` and `phase-0-reset-and-skeleton-report.md` in full before
touching anything here: the decisions already taken, the problems and
deviations recorded, and the changes made to this phase.

---

## While Working

Keep `phase-1-doctor-and-inventory-report.md` current as the work happens — after each significant
step, and before every commit, pause, or end of session.

---

## Objective

`ai-ctl init` installs the inventory — the declarative list of backends and models with their VRAM
budgets — and `ai-ctl doctor` diagnoses, read-only, everything the stack needs on this machine:
GPU and driver, CUDA toolkit and host compiler, build tools, container tooling, laptop hygiene, and
the inventory's model files, printing a remedy for each problem.

---

## Overview

### Why This Phase Matters
The inventory is the single source every later phase reads: the builder knows which project to
compile, the gateway configuration is generated from it, and the bench names its runs after its
entries (decision `docs/decisions/0001-gpu-arbitration-through-llama-swap.md`). The machine
pitfalls the stack study found — a CUDA 13.2 toolkit, the display on the NVIDIA GPU, a laptop on
battery, a missing GGUF file — are facts no upstream tool checks, and the build of phase 2 refuses
to start on the same toolchain failures.

### What It Enables
Phase 2 gates the build on the toolchain probes; phase 3 generates the gateway configuration from
the inventory; phases 3 and 4 reuse the GPU probe to read VRAM.

### Out of Scope
Installing anything: doctor prints commands, it never runs `sudo`. Downloading models. Audio and
image inventory entries, which come with a later roadmap (decision 0001). Checks of builds and of
the gateway, which phases 2 and 3 add to doctor.

---

## Tasks

### Foundations
- [x] Define the paths module: config, data and state directories with XDG defaults
- [x] Model diagnostics in the domain: a check with a status (ok, warn, fail, skip), a detail and a remedy, grouped in sections with summary counts, unit-tested

### GPU And Driver
- [x] NVML probe: GPU name, compute capability, total and used VRAM, driver version, CUDA driver API version
- [x] Flag processes holding VRAM while idle, with the remedy of moving the display to the iGPU

### Toolchain
- [x] CUDA toolkit probe: parse `nvcc --version`, fail on 13.2 and on a toolkit newer than the driver's CUDA version
- [x] Host compiler probe: the GCC versions installed and the newest one the installed `nvcc` accepts, read from the toolkit's `crt/host_config.h`
- [x] CMake and git probes with their minimum versions
- [x] Container probes, reported as warnings since only image-based inventory entries need them: podman, `nvidia-ctk`, and a CDI specification listing `nvidia.com/gpu=all`

### Laptop Hygiene
- [x] Power probes: AC adapter online, and the power profile set to `performance`, read over D-Bus (`ActiveProfile`, served by `power-profiles-daemon` and by `tuned-ppd`)

### Inventory
- [x] Model the inventory in the domain: backends (a native build or an official image), models (id, backend, files, server flags, context size, VRAM budget, idle TTL) and the gateway's listen address, with validation rules unit-tested (unique ids, known backends, positive budgets)
- [x] Replace `deploy/models.ini` with `deploy/inventory.toml`, carrying the same two models and flags, and parse it
- [x] Add `ai-ctl init --models-dir <dir>`, which installs the embedded inventory into the config directory with the models directory substituted, never overwriting an existing file
- [x] Check that every file the inventory references exists, with an `hf download` remedy

### Command
- [x] Render `ai-ctl doctor`: one line per check, remedies, a non-zero exit code on any failure, and a `--json` output
- [x] Unit tests for every parser on captured outputs: `nvcc`, `gcc`, `host_config.h`, `busctl`, the inventory, and a machine without NVML

### Documentation
- [x] Document `init`, `doctor` and the inventory format in the README

---

## Technical Details

### Files to Modify
```
Cargo.toml
README.md
deploy/models.ini           (deleted)
deploy/inventory.toml       (new)
src/domain/                 (diagnostics, inventory)
src/application/            (doctor use case, init use case)
src/infrastructure/         (NVML, command probes, sysfs, inventory loader, paths)
src/presentation/           (doctor and init commands, rendering)
tests/
```

### Dependencies
Phase 0: the skeleton, the checks, and `deploy/models.ini` as the source of the first inventory.

### Constraints
- Read-only: doctor never writes and never escalates privileges.
- A machine without NVML or without `nvcc` gets failed checks, not a crash.
- The models directory is given to `init`; no default path is guessed.
- The inventory format must hold audio and image backends later without a breaking change: a model names its backend kind, and the backend kind decides which fields apply.
- VRAM budgets are declared per model; they feed the gateway's `matrix` in phase 3.

---

## Acceptance Criteria

- [x] On this machine, `ai-ctl doctor` reports the installed CUDA 13.2 toolkit as a failure, with the upgrade remedy
- [x] `ai-ctl doctor --json` produces the same checks as the text output
- [x] `ai-ctl init --models-dir <dir>` writes the inventory once and refuses to overwrite it
- [x] A model whose file is missing is reported with the `hf download` command that fetches it
- [x] The three checks pass
