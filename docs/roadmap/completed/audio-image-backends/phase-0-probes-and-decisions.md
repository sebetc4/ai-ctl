# Phase 0: Probes And Decisions

---

## Status

**Current Status:** 🟢 Done (100% — 12/12)
**Started:** 2026-10-20
**Completed:** 2026-10-28
**Blocked By:** —

---

## While Working

Keep `phase-0-probes-and-decisions-report.md` current as the work happens — after each significant
step, and before every commit, pause, or end of session.

---

## Objective

Settle on this machine, for whisper.cpp, audio.cpp and stable-diffusion.cpp, what the next phases
build on: how each server builds natively for sm_120a, how it runs, answers and gets ready, what
it needs from llama-swap, and which model it serves. Where a probe leaves a choice, the user
decides before phase 1 opens.

---

## Overview

### Why This Phase Matters
The facts about these servers are not in the repository yet: `docs/gpu/llama-swap.md` lists them
from their documentation of 2026-10-08 and 2026-10-09, without a build, a run or a measure on
this machine. The code's own rules ask for probes: a parser is tested on captured outputs only,
and every backend must require ai-ctl's key, which a server may not support. Building on a guess
would put the error in the recipes, the inventory's kinds and the gateway's configuration at
once.

### What It Enables
Phase 1 writes each recipe from a build that worked by hand. Phase 2 writes each kind's command,
readiness check and key from a server that ran behind llama-swap. Phase 3 ships models already
chosen, verified and measured.

### Out of Scope
Any change to ai-ctl's code. Voxtral, Fish Speech and other Python backends. Container images.

---

## Tasks

### Builds
- [x] Build whisper.cpp's `whisper-server` by hand for sm_120a, starting from the llama.cpp recipe's CMake options: its repository, its release tags, the options and target that build the server alone, where the binary lands, and whether it lists its CUDA device; each command and its output recorded in the report
- [x] Build stable-diffusion.cpp's `sd-server` the same way
- [x] Build audio.cpp's `audiocpp_server` the same way, its CUDA source build on Linux included

### Servers
- [x] Capture each server's `--help` and `git tag` list as fixtures, listed in `tests/fixtures/README.md`, and record for each: the host, port and model options, the readiness endpoint and what it answers while the model loads, and whether it can require an API key, by option or by environment variable
- [x] Run each server behind a hand-written entry of the installed llama-swap: which of its endpoints the gateway routes (`/v1/audio/transcriptions`, `/v1/audio/speech`, `/v1/images/generations`, `/audioapi/v1/tasks/run`, `/sdapi/v1/*`), how a request names its model, and whether the gateway's key reaches the server
- [x] Find what OpenWebUI needs to use speech-to-text, text-to-speech and image generation through an OpenAI-compatible endpoint, and whether each server's API gives it

### Models
- [x] Compare with the user the candidates of each use — speech-to-text (whisper.cpp, audio.cpp), text-to-speech and voice cloning (audio.cpp), image generation (stable-diffusion.cpp) — and keep those that pass, each with its repository, pinned revision and files
- [x] Write with the user the provenance rule for models other than LLM GGUF files, beside `docs/stack.md` §5's: which organizations, the pinned revision, the SHA-256 against the Hub's LFS hash; then download the chosen models into `/ai/models/<use>/<id>/` and verify them
- [x] Measure each model's peak VRAM and its time to ready, its server alone on the GPU, and one request's time

### Decisions
- [x] Put to the user, for each server that cannot require a key, the ways the probe leaves to guard it, with what `docs/security.md` would then say
- [x] Put to the user whether audio.cpp stays, on the probes: its build, its API, its VRAM, and its overlap with whisper.cpp for speech-to-text
- [x] Record the facts in the documents of their domains, `docs/audio/` and `docs/image/`, one document per server, with their sources and dates, each listed in `docs/README.md`; adjust phases 1 to 3 to what the probes found

---

## Technical Details

### Files to Modify
```
docs/audio/                 (to be created: one document per server)
docs/image/                 (to be created)
docs/README.md              (index)
docs/stack.md               (§5, the provenance rule, or its domain's document)
tests/fixtures/             (each server's --help and tag list)
tests/fixtures/README.md
```

### Dependencies
The CUDA toolkit 13.4 and the host compilers of `docs/toolchain.md`. llama-swap v263, installed
by `ai-ctl gateway install`. The gateway must be down while a server is probed alone on the GPU.

### Constraints
- Hand builds go to a scratch folder, never into `builds/`, which `ai-ctl build` manages.
- A hand-written llama-swap configuration runs only on the loopback, with ai-ctl's key in its
  environment, never in the file.
- Models are downloaded by hand, as `docs/stack.md` §5 does for the LLM: ai-ctl neither
  downloads nor verifies models.
- Every fact recorded names its source: the command and its output, the file and commit read,
  or the page and the date it was read.

---

## Acceptance Criteria

- [x] Each of the three servers has been built by hand for sm_120a and has answered one request through the installed llama-swap
- [x] Each server's options, endpoints, readiness and key support are recorded with their sources, and its captures listed in `tests/fixtures/README.md`
- [x] The models kept for each use are chosen, downloaded under the written provenance rule, verified and measured
- [x] The user has decided the guard of every server that cannot require a key, and whether audio.cpp stays
- [x] Phases 1 to 3 match what the probes found
