# Phase 1: Native Builds

---

## Status

**Current Status:** 🟢 Done (100% — 14/14)
**Started:** 2026-10-28
**Completed:** 2026-10-28
**Blocked By:** —

---

## Before Starting This Phase

Read `phase-0-probes-and-decisions.md` and `phase-0-probes-and-decisions-report.md` in full before
touching anything here: the decisions already taken, the problems and
deviations recorded, and the changes made to this phase.

---

## While Working

Keep `phase-1-native-builds-report.md` current as the work happens — after each significant
step, and before every commit, pause, or end of session.

---

## Objective

`ai-ctl build` builds whisper.cpp, stable-diffusion.cpp and audio.cpp as it builds llama.cpp: one
recipe each, versioned builds with a `current` link per project, and `build list | use | prune`
across the four projects.

---

## Overview

### Why This Phase Matters
The builder was written for every ggml project, but only llama.cpp has a recipe, and
`TagPattern` only reads llama.cpp's `bNNNN` tags. A server the gateway runs must come from a
build ai-ctl made and keeps, with the rollback and the pruning rules llama.cpp's builds have.

### What It Enables
Phase 2 points each new kind's command at its project's current build.

### Out of Scope
The inventory's kinds and the gateway's configuration. Container images of these projects.

---

## Tasks

### Recipes
- [x] Read each project's release tags with a tag pattern for its form as phase 0 captured it, beside `Numbered`, tested on the captured tag lists
- [x] Ship the whisper.cpp recipe: repository, tags, CMake options, the `whisper-server` target, its artifact and its device listing, if any, as phase 0's report records its hand build
- [x] Ship the stable-diffusion.cpp recipe, its `ggml` submodule only, `SD_SERVER_BUILD_FRONTEND`, `SD_WEBP` and `SD_WEBM` off, so that no build runs `pnpm install`
- [x] Ship the audio.cpp recipe, with `AUDIOCPP_DEPLOYMENT_BUILD=ON` so that the binary carries the model specs of its commit, and without its SSH submodule
- [x] A recipe names the submodules the builder initialises, fetched as the clone is
- [x] A recipe's smoke test and device listing: `whisper-server` has neither `--version` (an unknown argument exits 0) nor a device listing; `sd-server`'s `CUDA0<TAB>…` and `audiocpp_server`'s `CUDA:0 "…" [GPU]` read as llama-server's `CUDA0:` lines are, each tested on phase 0's captures
- [x] A recipe's patches: the builder applies them after the checkout, refuses a tag they no longer apply to, and records them in `build.json`; the three key patches phase 0 prototyped (`LLAMA_API_KEY` required to start and on every route but the health check), each tested on its built server; `sd-server`'s patch also ends a failed `listen` with a message and exit 1, where it now exits 0 without a word

### Lifecycle
- [x] `build list`, `use` and `prune` across several projects: each project keeps its `current`, and `prune` never removes a project's current build or the build the gateway serves
- [x] `doctor` reports each project's current build

### Live
- [x] Build each project's latest release on this machine with `ai-ctl build <project>`, smoke-tested
- [x] `build use` back to an older build of one project, which leaves the others as they were

### Documentation
- [x] `docs/commands.md`'s `build` and README's build section name the four projects, and `docs/security.md` §7 says what each patch changes in its server and why

### Binding
- [x] `whisper-server` and `sd-server` bind as `llama-server` does, `SO_REUSEADDR` alone: a `reuse-address.patch` in each recipe, so that a second server on a live port exits 1 and a TIME-WAIT left by a server of the stack no longer stops a start; probed on each built server

### Submodules
- [x] A recipe names a submodule of a submodule by its whole path, checked out within its parent: stable-diffusion.cpp's names its ggml fork's CUTLASS, `ggml/src/ggml-cuda/vendors/cutlass`, and `ai-ctl build stable-diffusion.cpp` builds `master-1024-77b72a2` or later, smoke-tested and probed

---

## Technical Details

### Files to Modify
```
src/domain/recipe.rs
src/domain/build.rs
src/application/build.rs
src/application/doctor.rs
src/infrastructure/build_store.rs
src/presentation/build.rs
tests/
docs/commands.md
README.md
```

### Dependencies
Phase 0: each project's build by hand, its tag list and its device listing, captured, and the key
patch each server was built and probed with, recorded in its report.

### Constraints
- Nothing in the builder is specific to one project outside its recipe.
- A patch is the smallest change that makes its server require the key; it lives in the
  repository beside its recipe, and a build records it.
- A recipe's facts come from phase 0's captures; a tag pattern is tested on the captured list.
- One build at a time, under the existing lock; a build of one project never touches another's
  builds.
- Each commit's code diff goes through `cargo mutants --in-diff`; a mutant it misses is killed or
  justified in the report.

---

## Acceptance Criteria

- [x] `ai-ctl build whisper.cpp`, `ai-ctl build stable-diffusion.cpp` and `ai-ctl build audio.cpp`, with no ref, build the latest release, smoke-tested
- [x] `build use` and `build prune` act on one project without touching the others
- [x] The five checks pass
