# Phase 3: End To End

---

## Status

**Current Status:** 🟢 Done (100% — 12/12)
**Started:** 2026-10-29
**Completed:** 2026-10-30
**Blocked By:** —

---

## Before Starting This Phase

Read `phase-2-inventory-and-gateway.md` and `phase-2-inventory-and-gateway-report.md` in full before
touching anything here: the decisions already taken, the problems and
deviations recorded, and the changes made to this phase.

---

## While Working

Keep `phase-3-end-to-end-report.md` current as the work happens — after each significant
step, and before every commit, pause, or end of session.

---

## Objective

The shipped inventory serves speech-to-text, text-to-speech and images through the gateway on
this machine, OpenWebUI uses all three, and the documentation says how.

---

## Overview

### Why This Phase Matters
The tests run against doubles; only a live run shows that llama-swap starts each server, routes
its requests, swaps it against the 27B, and that the key guards it. OpenWebUI is the client that
will use these backends day to day.

### What It Enables
The audio and image backends in daily use, and a delivered roadmap whose code the next version
of `reviewing-delivered-code` reviews as its first real use.

### Out of Scope
Bench scenarios for audio and image. Other clients: OpenCode and the orchestrator do not use
these endpoints.

---

## Tasks

### Shipped Inventory
- [x] Add every model phase 0 keeps, and Fibo 2.0 Dev, compared and kept on 2026-10-29 (phase 2's report), several per use and each precision kept as an entry of its own, to `deploy/inventory.toml` with their backends, `vram_mib` and `ttl`, their directories under `stt/`, `tts/` and `image/`, and the snapshot of the configuration generated from it

### Live
- [x] With `ai-ctl up`, one request to each model of the shipped inventory through the gateway with ai-ctl's key, each refused without it; times recorded
- [x] Swaps: a chat on the 27B, then an image, then an image on another image model, then a chat; llama-swap stops and starts the servers as the matrix says
- [x] Each server's own port: what a local process gets without the key, as phase 0's decision says
- [x] Fibo 2.0 Dev's two minutes an image: one image through the gateway and one from OpenWebUI, neither cut short by a timeout of llama-swap or OpenWebUI; one at the largest size OpenWebUI is set to, the server alive after it (`docs/image/stable-diffusion-cpp.md` §4 gives the sizes measured)
- [x] Each image model's largest size, its VAE decoding on the CPU (`--backend vae=cpu`), on the GPU with the weights in RAM (`--offload-to-cpu`) or on the GPU by tiles of 1024 px (`--vae-tiling`), from 2048×2048 up to sd-server's limit of 4096 a side: its peak VRAM and RAM, its time, and an image at full steps judged by eye; the budgets and OpenWebUI's size follow what passes, with the user
- [x] Image editing: for each image model, one `/v1/images/edits` request through the gateway with a reference image at 2048×2048, its time, its peak VRAM against its budget and its result judged by eye, Qwen-Image 2.1 and Fibo 2.0 Dev with their vision projector (`--llm_vision`); then OpenWebUI's image editing configured and used once
- [x] After `ai-ctl down`, no server process is left and the GPU holds none of their memory

### Clients
- [x] Configure OpenWebUI's speech-to-text, text-to-speech and image generation through the gateway with ai-ctl's key, and use each once from its interface; OpenWebUI takes one model per engine, typed by hand since its model and voice lists are requested without the key: say how to switch to another model
- [x] Document that configuration with OpenWebUI's chat, moved from `docs/stack.md` §7.3, in a guide of its own, `docs/openwebui.md`, which the audio and image documents link to

### Documentation
- [x] `docs/gpu/llama-swap.md`'s backends and To Come say what is served now
- [x] README's status and usage, and `docs/README.md`'s index, cover the new backends

---

## Technical Details

### Files to Modify
```
deploy/inventory.toml
tests/snapshots/
docs/gpu/llama-swap.md
docs/stack.md
docs/audio/
docs/image/
docs/README.md
README.md
```

### Dependencies
Phases 0 to 2: the models downloaded and verified, the builds, the kinds and the gateway's
configuration.

### Constraints
- OpenWebUI's container stays published on the loopback only, as `docs/stack.md` §7.3 says.
- A live run's results go to the report with their commands; a fact about a server or llama-swap
  that a run finds goes to its domain's document with its source.

---

## Acceptance Criteria

- [x] On this machine, `ai-ctl up` serves each model of the shipped inventory through the gateway with ai-ctl's key, and refuses each without it
- [x] OpenWebUI transcribes, speaks and generates an image through the gateway
- [x] After `ai-ctl down`, no server process is left and the GPU holds none of their memory
- [x] The five checks pass
