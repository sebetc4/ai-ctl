# Phase 3: Gateway

---

## Status

**Current Status:** 🟢 Done (100% — 13/13)
**Started:** 2026-10-09
**Completed:** 2026-10-11
**Blocked By:** —

---

## Before Starting This Phase

Read `phase-2-native-build.md` and `phase-2-native-build-report.md` in full before
touching anything here: the decisions already taken, the problems and
deviations recorded, and the changes made to this phase.

---

## While Working

Keep `phase-3-gateway-report.md` current as the work happens — after each significant
step, and before every commit, pause, or end of session.

---

## Objective

llama-swap becomes the single entry point and the only process that starts or stops GPU backends:
`ai-ctl` installs it, generates its configuration from the inventory — `matrix` included, computed
from the VRAM budgets — and `ai-ctl up | down | status | logs` run it on demand, `down` returning
only once the VRAM is free.

---

## Overview

### Why This Phase Matters
One GPU needs one arbiter, and the arbiter must sit on the path of the requests to never stop a
backend in the middle of a generation (decision `docs/decisions/0001-gpu-arbitration-through-llama-swap.md`).
llama-swap already does that for every backend planned, but it does no VRAM accounting: ai-ctl
supplies the combinations, computed from the inventory, and keeps the user in control of when
anything runs.

### What It Enables
Every client — OpenCode, OpenWebUI, scripts, the future orchestrator — talks to one endpoint,
`127.0.0.1:8080`, whatever backend serves the request. Phase 4 benchmarks through it. A later
roadmap adds audio and image backends as inventory entries, with no change to the gateway code.

### Out of Scope
llama-server's router mode, which decision 0001 drops. Writing a gateway. Restarting a crashed
gateway, and starting at boot. Quadlets and systemd units. The orchestrator.

---

## Tasks

### Installation
- [x] Install a pinned llama-swap release binary into the data directory, verifying its SHA-256 against the release checksums; `gateway install [--version]`

### Configuration
- [x] Generate one llama-swap model entry per inventory model: `cmd` running the current native build of `llama-server` with the model's flags on `${PORT}`, health check, idle `ttl`
- [x] Generate image-based entries for the comparison runtime: `cmd` running `podman run` of an official `server-cuda` or `server-cuda13` tag with the CDI GPU device and the models mounted read-only at their host paths, and `cmdStop`
- [x] Compute the `matrix` sets from the VRAM budgets: the combinations whose sum fits the GPU memory minus a margin, with eviction costs from the backends' load times
- [x] Snapshot tests of the generated configuration for the shipped inventory

### Lifecycle
- [x] `up`: pull the missing images of the backends the models use, start llama-swap detached in its own process group on the inventory's listen address, its PID and log in the state directory, and wait for `/health`; refuse when the port is taken
- [x] `down`: unload every model through `POST /api/models/unload`, stop llama-swap (SIGTERM, then SIGKILL after a timeout), and wait until NVML shows the VRAM released
- [x] Detect stale state: a dead PID, or a PID now held by another executable (`/proc/<pid>/exe`)
- [x] Regenerate the configuration on `up` when the inventory or the current build changed

### Commands
- [x] `status`: gateway process, listen address, running models from `/running`, VRAM used
- [x] `logs [-f] [model]`: the gateway's log, or one model's stream from `/logs/stream/<model>`

### Tests And Docs
- [x] Integration test of `up`, a proxied request and `down`, with llama-swap running a fake upstream that serves `/health` and `/v1/chat/completions` (skipped when the llama-swap binary is absent)
- [x] Add the gateway state to `doctor`, point `deploy/opencode.json` and `docs/stack.md` at the gateway, and document the commands in the README

---

## Technical Details

### Files to Modify
```
Cargo.toml
README.md
deploy/opencode.json
docs/stack.md
src/domain/                 (gateway state, matrix computation)
src/application/            (install, up, down, status use cases)
src/infrastructure/         (llama-swap installer, config writer, process spawner, state file, gateway API client)
src/presentation/           (gateway, up, down, status, logs commands)
tests/
```

### Dependencies
Phase 2: a current llama.cpp build to run. Phase 1: the inventory, the NVML probe and the container
probes.

Both models of the inventory, downloaded under `/ai/models/llm/` and checked as `docs/stack.md` §5
("Where Models Come From") says, before the acceptance criteria can be checked.

### Constraints
- The gateway listens on `127.0.0.1:8080` unless the inventory says otherwise, so the clients' configuration does not change.
- `ai-ctl` exits once the gateway is up: the only resident process is llama-swap, which holds no VRAM.
- llama-server endpoints that llama-swap does not route (`/tokenize`, `/metrics`, `/slots`) are reached through `/upstream/<model>/`.
- A configuration uses `matrix`, never `groups` (llama-swap accepts only one of the two).

---

## Acceptance Criteria

- [x] `ai-ctl up` returns once the gateway answers, and a chat request naming the main model loads it and succeeds
- [x] Requesting the other model swaps them, with no two models resident beyond what the `matrix` allows
- [x] `ai-ctl down` returns only after NVML reports the VRAM freed
- [x] `ai-ctl status` reports a gateway killed outside ai-ctl as stopped, and cleans its state
- [ ] OpenCode and OpenWebUI work unchanged against `127.0.0.1:8080`
- [x] The three checks pass
