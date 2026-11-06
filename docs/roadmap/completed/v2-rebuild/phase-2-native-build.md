# Phase 2: Native Build

---

## Status

**Current Status:** 🟢 Done (100% — 15/15)
**Started:** 2026-10-09
**Completed:** 2026-10-09
**Blocked By:** —

---

## Before Starting This Phase

Read `phase-1-doctor-and-inventory.md` and `phase-1-doctor-and-inventory-report.md` in full before
touching anything here: the decisions already taken, the problems and
deviations recorded, and the changes made to this phase.

---

## While Working

Keep `phase-2-native-build-report.md` current as the work happens — after each significant
step, and before every commit, pause, or end of session.

---

## Objective

`ai-ctl build` compiles a ggml project from its sources at a chosen ref into a versioned build for
sm_120a, and `build list | use | prune` manage the installed versions, with instant rollback. The
builder is driven by per-project recipes; this roadmap ships the llama.cpp recipe.

---

## Overview

### Why This Phase Matters
A native build runs the exact commit wanted: the official image is rebuilt once a day and can lag
one or two days behind when its build fails. Compiling for `120a` keeps the Blackwell-specific
instructions, which v1's builder lost by targeting `120`. Versioned builds give the rollback a
container tag would otherwise give. llama.cpp, whisper.cpp, stable-diffusion.cpp and audio.cpp
share the same ggml CMake/CUDA recipe, so one builder serves them all (decision 0001).

### What It Enables
Phase 3 points the gateway's model entries at the current build; phase 4 compares builds with each
other and with the official image. A later roadmap adds the whisper.cpp, stable-diffusion.cpp and
audio.cpp recipes without changing the builder.

### Out of Scope
Recipes other than llama.cpp. Prebuilt binaries and container images. Building llama.cpp tools
other than `llama-server`. Installing or upgrading the CUDA toolkit: doctor prints the commands.

---

## Tasks

### Domain
- [x] Model a build: project, id (release tag or short commit), source commit, toolkit version, host compiler, CUDA architectures, CMake flags and date, with JSON metadata round-trip tests
- [x] Model a build recipe: repository URL, release tag pattern, CMake flags, target and binary name; ship the llama.cpp recipe (`b####` tags, `llama-server` target)

### Sources
- [x] Clone or fetch a project into the data directory through the `git` command line, and check out a ref
- [x] Resolve `latest` to the newest release tag matching the recipe's pattern, with `git ls-remote --tags`

### Compile
- [x] Gate the build on the doctor toolchain probes: refuse CUDA 13.2, pick the host compiler
- [x] Generate the CMake flags from the recipe (`GGML_CUDA=ON`, `CMAKE_CUDA_ARCHITECTURES=120a-real`, static libraries, tests and examples off), unit-tested
- [x] Configure and build the recipe's target only, streaming the output to a build log behind a progress line
- [x] Install the binary into `builds/<project>/<id>/` with its metadata, leaving no partial version behind when a build fails
- [x] Smoke-test each new build: `--version` runs, and for llama.cpp `--list-devices` lists the CUDA device

### Versions
- [x] Maintain a `current` link per project, switched by `build use <project> <id>`
- [x] `build list` (project, current marker, date, commit, toolkit) and `build prune --keep N`, which never removes a current build

### Toolchain Setup
- [x] Document the build toolchain in `docs/toolchain.md`: a fresh install on Fedora, a CUDA version change, a Fedora release change, and a dated record of the commands run on this machine
- [x] Align `doctor`'s toolkit remedy with that document: numbered steps, NVIDIA's repository for the running Fedora release with its driver packages excluded, and a versioned `cuda-toolkit-<major>-<minor>` package
- [x] Warn in `doctor` when an enabled NVIDIA CUDA repository offers driver packages beside the RPM Fusion driver, and when an unversioned `cuda-toolkit` or `cuda-toolkit-<major>` meta-package is installed

### Integration
- [x] Add the current builds to `doctor`, and document `build` in the README

---

## Technical Details

### Files to Modify
```
README.md
docs/toolchain.md           (to be created)
docs/stack.md               (§8, machine setup)
src/domain/                 (build, recipe, toolkit rule)
src/application/            (build, use, list, prune use cases)
src/infrastructure/         (git and cmake runners, build store, recipes)
src/presentation/           (build commands)
tests/
```

### Dependencies
Phase 1: the toolchain probes, the paths module and the inventory.

A CUDA toolkit other than 13.2 must be installed before the first real build. At the roadmap's
opening the machine had `nvcc` 13.2 with a driver supporting CUDA 13.4, so 13.3 or 13.4 fits. Its
CUDA repository, `cuda-fedora43`, stops at 13.2: `ai-ctl doctor` prints the commands that add
NVIDIA's `fedora44` repository and install `cuda-toolkit-13-4`, after which the `/usr/local/cuda`
alternative points at 13.4 by itself.

Phase 1 provides `HostProbe::cuda_toolkit` and `host_compilers`, and `pick_host_compiler` in the
domain, for the build gate; `doctor`'s report takes the build checks as one more section.

### Constraints
- `git` and `cmake` are driven as commands; no `git2` crate.
- At the roadmap's opening, the default compiler was GCC 16.2.1 and GCC 15 was also installed: the host compiler is chosen by the probe, never hard-coded.
- The build gate and `doctor` give the same toolkit verdict (13.2 refused, 12.8 at least, not newer than the driver): the rule moves from `src/application/doctor.rs` into the domain, and both call it.
- Builds run through their own runner, which streams the output to the build log; phase 1's probe runner buffers the output and kills a command after 15 s.
- One build at a time; a second `ai-ctl build` while one runs is refused.
- Nothing in the builder is specific to llama.cpp outside its recipe.

---

## Acceptance Criteria

- [x] `ai-ctl build llama.cpp` with no ref builds the latest release tag, and the new build lists the CUDA device
- [x] `ai-ctl build llama.cpp --ref <commit>` builds an arbitrary commit
- [x] `ai-ctl build use llama.cpp <id>` switches back to an older build without recompiling
- [x] A build started with CUDA 13.2 installed is refused with doctor's remedy
- [x] `doctor`'s toolkit remedy gives the commands of `docs/toolchain.md`, and `doctor` warns about a CUDA repository offering driver packages and about an unversioned toolkit meta-package
- [x] The three checks pass

---

## Risk & Mitigation

- **A build links the toolkit's CUDA libraries dynamically**, so removing a toolkit can break older builds. The toolkit version is recorded in each build's metadata, and `GGML_STATIC` is evaluated during the phase to make builds self-contained.
