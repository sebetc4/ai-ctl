# Roadmap: ai-ctl v2 Rebuild

Rebuilds llm-ctl v1 as ai-ctl v2, a CLI that checks the machine, builds llama.cpp natively and
runs llama-swap as the single gateway and VRAM arbiter.

---

## Status Indicators

- 🔴 Not Started
- 🟡 In Progress
- 🟢 Done
- ⏸️ Blocked
- ⚠️ Needs Review

---

## Overall Progress

```
Phase 0  Reset & Skeleton           🟢 ████████████████████ 100%  (14/14)
Phase 1  Doctor & Inventory         🟢 ████████████████████ 100%  (16/16)
Phase 2  Native Build               🟢 ████████████████████ 100%  (15/15)
Phase 3  Gateway                    🟢 ████████████████████ 100%  (13/13)
Phase 4  Bench                      🟢 ████████████████████ 100%  (14/14)
TOTAL                                  ████████████████████ 100%  (72/72)
```

**Current Phase:** —
**Blocked By:** —
**Next Milestone:** —

---

## Why This Roadmap Exists

By October 2026, llama-server natively covers most of what llm-ctl v1 built around it: model
presets, model switching, `--fit`, idle sleep, MTP speculative decoding, OpenAI and Anthropic APIs.
The v2 study settled on llama.cpp as the single LLM engine and abandoned SGLang. The gateway, the
SGLang backend, the model registry and the LoRA, draft and warm flags — roughly a third of v1 —
became dead weight.

This roadmap restarts the tool from a clean repository with a new history, keeping only what
upstream does not do. Renamed ai-ctl, since audio and image backends will follow the LLM stack, it
diagnoses the machine, builds the ggml backends natively in versioned builds, runs llama-swap as the
single gateway and GPU arbiter, and benchmarks the stack. This roadmap delivers the LLM domain;
ai-ctl is a production tool.

---

## Decisions Taken At Opening

Revised on 2026-10-09 during phase 1, after decision
0001, now [docs/gpu/llama-swap.md](../../../gpu/llama-swap.md); the set taken at opening is
in the 1.0.0 changelog entry and in the git history.

- **One gateway, one arbiter:** llama-swap is the single entry point of every client and the only process that starts or stops a GPU backend. ai-ctl installs it, generates its configuration and runs it between `up` and `down`; it holds no VRAM. No systemd unit.
- **Single LLM engine, no router mode:** each model runs as a plain `llama-server` behind llama-swap, which does all the swapping. SGLang is abandoned; vLLM would come through a future roadmap, only if the prefill criterion of the stack study is met.
- **An inventory instead of presets:** a declarative list of backends and models with their flags and VRAM budgets, from which ai-ctl generates the llama-swap configuration and its `matrix`. No model registry.
- **Native build by default:** compiled from source at a chosen ref, for `120a` (Blackwell-specific instructions), in versioned builds with rollback, by a builder driven by per-project recipes (llama.cpp first; whisper.cpp, stable-diffusion.cpp and audio.cpp later).
- **Official images as a second runtime:** `server-cuda` and `server-cuda13` entries in the gateway configuration, so the bench can compare them with native builds.
- **Clean Architecture kept** (domain, application, infrastructure, presentation) for testability — the use cases touching the GPU, processes and the network run against test doubles — with a port only where a second implementation or a test needs it.
- **New git history:** the v1 history and the uncommitted Phase 6-S work are archived as a git bundle outside the repository, in `/code/rust/projects/llm-ctl-v1-archive/`.
- **Contract:** documentation in English; `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test` must pass to close a phase.
- **Machine at opening:** Fedora 44, NVIDIA driver 615.71.09 (CUDA driver API 13.4), CUDA toolkit 13.2 installed, GCC 16.2.1 by default with GCC 15 available, CMake 4.3.0.

---

## Deliberately Out Of Scope

- Audio and image backends (whisper.cpp, audio.cpp, Voxtral through vLLM-Omni, Fish Speech, stable-diffusion.cpp): a later roadmap, prepared by decision 0001.
- The orchestrator that exposes local tasks to the agents: a separate project.
- A gateway or an arbiter written for ai-ctl: llama-swap is used.
- The SGLang backend; a vLLM backend.
- A model registry, Hugging Face downloads, and LoRA, draft and warm flags: the inventory and `hf download` cover them.
- Quadlets and systemd units; deploying OpenWebUI.
- A dashboard TUI.
- Platforms other than Fedora with an NVIDIA GPU.
- Evaluating model quality, and automatic sweeps of the inventory.

---

## Phases

- 🟢 [Phase 0: Reset & Skeleton](phase-0-reset-and-skeleton.md) — archive v1, clean slate, new git history, layered skeleton, rewritten documentation
- 🟢 [Phase 1: Doctor & Inventory](phase-1-doctor-and-inventory.md) — `init` installs the inventory; `doctor` checks GPU, driver, toolkit, compiler, container tooling, power and model files
- 🟢 [Phase 2: Native Build](phase-2-native-build.md) — recipe-driven, versioned ggml builds for sm_120a, llama.cpp first, with `list`, `use` and `prune`
- 🟢 [Phase 3: Gateway](phase-3-gateway.md) — llama-swap installed, configured from the inventory with a VRAM-computed `matrix`, and run with `up`, `down`, `status`, `logs`
- 🟢 [Phase 4: Bench](phase-4-bench.md) — the stack study's measurement protocol through the gateway, saved runs and `compare`

---

## Dependencies

- Each phase builds on the previous one.
- Phase 2 needs a CUDA toolkit other than 13.2 before its first real build: 13.3 or 13.4, which the driver supports. Settled on 2026-10-09: CUDA 13.4 is installed.
- The Phase 4 baseline needs the inventory's models downloaded. Settled on 2026-10-10: both models are downloaded and checked.

---

## Related Documentation

- [docs/gpu/llama-swap.md](../../../gpu/llama-swap.md) — decision 0001, why one gateway arbitrates the GPU, the options rejected, the backends planned
- [docs/stack.md](../../../stack.md) — the local inference stack: engine, models, settings, clients, measurement protocol
- [docs/architecture.md](../../../architecture.md) — layers, dependency rule and port policy

---

## Metadata

**Roadmap Status:** 🟢 Done
**Location:** `docs/roadmap/completed/v2-rebuild/`
**Version:** 2.0.0
**Created:** 2026-10-09
**Last Updated:** 2026-10-12

---

## Changelog

### 2.0.0 (2026-10-12)

Roadmap closed: the five phases are done, 72 tasks of 72. ai-ctl diagnoses the machine, builds
llama.cpp natively in versioned builds, runs llama-swap as the single gateway and GPU arbiter on
`127.0.0.1:8080`, and benchmarks the stack through it; OpenCode and OpenWebUI work through the
gateway. [summary.md](summary.md) records where the roadmap started and landed, what each phase
delivered, what was learned beyond it — read the upstream at the pinned version, ask the machine
rather than a table, keep a benchmark's inputs identical under speculative decoding — and what is
left open: the official image's slower short prompts, `disableKeepAlives` until llama.cpp #29680,
and the directions of `docs/notes/`. No restructuring was pending. The folder moved from
`docs/roadmap/on-progress/v2-rebuild/` to `docs/roadmap/completed/v2-rebuild/`.

### 1.6.0 (2026-10-12)

Phase 4 closed. `ai-ctl bench run <model>` measures a model through the gateway with the protocol
of `docs/stack.md` §9 — generation by context depth up to the context's limit, a fresh 40K
prefill, ten-turn sessions sending their reasoning back, MTP's acceptance per request checked
against `/metrics`, the GPU read through NVML, and the gateway's overhead — and saves each run as
JSON with the inventory entry, command line, build or image digest and machine it measured;
`bench list` and `bench compare` read the saved runs. The baseline found the native `b11538` and
the official `server-cuda13-b11541` equal in generation and prefill, the image 50 to 80 ms slower
on an agent turn's first token: the native build stays the default. Found along the way: prompts
that differed between runs changed MTP's acceptance, a generation stops one token short of a full
context, and `cache-ram`'s save of the slot's context fell into a scenario's first repetition —
each fixed — and doctor's CDI remedy wrote a specification that goes stale; doctor now checks it
against the loaded driver. OpenCode, OpenWebUI, `/v1/messages` and `/v1/responses` work through
the gateway, phase 3's last criterion; `docs/stack.md` §7 gives OpenWebUI the right address and
records what both clients cost. Every phase is done: the roadmap is ready to close.

### 1.5.0 (2026-10-11)

Phase 3 closed. llama-swap v263 is the single gateway on `127.0.0.1:8080`. `ai-ctl gateway
install` installs a pinned release verified against its SHA-256; `gateway config` prints the
configuration generated from the inventory — one llama-server entry per model, on the native build
or an official image under podman, and a `matrix` computed from the VRAM budgets and the GPU's
usable memory less `[gateway] vram_margin_mib`; `up`, `down`, `status` and `logs` run it on
demand, `down` returning once NVML shows the models' memory released; `doctor` gains a Gateway
section. Checked on this machine with both models: a swap takes about 10 s, never with both on
the GPU. Found along the way: a gateway killed with SIGKILL leaves its models on the GPU, so `up`
refuses while such processes remain and `status` names them; and the test suite was flaky, child
processes forked by parallel tests holding other tests' files. The models live under
`/ai/models/llm/`, pinned and verified by hand (`docs/stack.md` §5); a model library managed by
ai-ctl is recorded as a direction in `docs/notes/model-library.md`. The fifth acceptance
criterion, OpenCode and OpenWebUI themselves, moved to phase 4 as a task: phase 4 has 13 tasks.

### 1.4.0 (2026-10-09)

Phase 2 closed. `ai-ctl build <project> [--ref] [--force] [--jobs]` compiles a ggml project from a
blobless clone at a release tag, a branch or a commit, for `120a-real` with the real GCC `nvcc`
accepts, gated on `doctor`'s GPU and Toolchain checks; it smoke-tests the binary, installs it as
`builds/<project>/<id>/` with `build.json` and its log, and makes it current. `build list`, `use`
and `prune --keep N` manage the versions; the llama.cpp recipe is the only one. `b11538` is built
and current on this machine. Found along the way: `GGML_STATIC` cannot link a CUDA build (no
static `libcuda`), so builds load the CUDA libraries by soname; `g++` on the `PATH` is ccache's
wrapper. The phase grew from 12 to 15 tasks with `docs/toolchain.md` and three `doctor` changes:
numbered toolkit remedy for the running Fedora release, and warnings about a CUDA repository
offering driver packages and an unversioned toolkit meta-package. Nothing moved to a later phase.

### 1.3.0 (2026-10-09)

Phase 1 closed. `ai-ctl init --models-dir <dir>` installs the inventory, `deploy/inventory.toml`,
which replaces `deploy/models.ini` with the same two models and flags, plus their VRAM budgets and
idle TTLs; `ai-ctl doctor [--json]` checks the GPU and driver, the CUDA toolkit and host compiler,
CMake and git, the container tooling, power, and the inventory's model files, with a remedy for
each problem. On this machine it fails on the CUDA 13.2 toolkit. Found along the way:
`powerprofilesctl` is absent on Fedora 44 (`tuned-ppd`), so the profile is read over D-Bus, and the
GCC limit of `nvcc` is read from the toolkit's `host_config.h`. Nothing moved to a later phase;
phase 2 gained a dependency note and two constraints.

### 1.2.0 (2026-10-09)

Restructured during phase 1, with the user's approval, after decision 0001 (one GPU arbiter:
llama-swap as the gateway). The tool is renamed ai-ctl and the roadmap delivers the LLM domain
only. Phase 1 becomes Doctor & Inventory (16 tasks): an inventory of backends and models with VRAM
budgets replaces `models.ini`. Phase 2 builds ggml projects through recipes, llama.cpp first
(12 tasks). Phase 3, Server Lifecycle, becomes Gateway (13 tasks): llama-swap installed, configured
from the inventory with a computed `matrix`, and run with `up` and `down`; llama-server's router
mode is dropped. Phase 4 measures through the gateway. Files renamed:
`phase-1-doctor-and-presets.md` → `phase-1-doctor-and-inventory.md` (and its report),
`phase-3-server-lifecycle.md` → `phase-3-gateway.md`.

### 1.1.0 (2026-10-09)

Phase 0 closed. v1 is archived as a verified git bundle outside the repository, the repository
restarts with a new history, and the tree holds a compiling four-layer skeleton with one command
(`completions`), the presets and OpenCode configuration under `deploy/`, and rewritten
documentation (`docs/stack.md`, `docs/architecture.md`, README). Nothing moved to a later phase.
The roadmap folder moved from `docs/roadmap/pending/v2-rebuild/` to
`docs/roadmap/on-progress/v2-rebuild/`.

### 1.0.1 (2026-10-09)

llm-ctl is a production tool, not a learning project: the purpose no longer mentions learning, and
Clean Architecture is kept for testability.

### 1.0.0 (2026-10-09)

Roadmap created with five phases: Reset & Skeleton, Doctor & Presets, Native Build, Server
Lifecycle, and Bench. Native builds are the default runtime; the official container images are kept
as a second runtime for comparison benches.
