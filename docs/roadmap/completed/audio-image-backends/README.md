# Roadmap: Audio And Image Backends

Adds whisper.cpp, audio.cpp and stable-diffusion.cpp beside llama.cpp, built natively and
served through llama-swap, with their models in the inventory.

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
Phase 0  Probes And Decisions       🟢 ████████████████████ 100%  (12/12)
Phase 1  Native Builds              🟢 ████████████████████ 100%  (14/14)
Phase 2  Inventory And Gateway      🟢 ████████████████████ 100%  (13/13)
Phase 3  End To End                 🟢 ████████████████████ 100%  (12/12)
TOTAL                                  ████████████████████ 100%  (51/51)
```

**Current Phase:** —
**Blocked By:** —

---

## Why This Roadmap Exists

`docs/gpu/llama-swap.md` made llama-swap the only arbiter of the GPU for every backend, so that
audio and image backends could join the LLM without a controller of their own. It plans
whisper.cpp, audio.cpp and stable-diffusion.cpp, ggml projects built with the same CMake and CUDA
recipe as llama.cpp. The v2 rebuild delivered the LLM domain only, and `v2-review-fixes` reviewed
and fixed that code so these backends build on reviewed code.

The code is ready for them in its structure, not yet in its content. At the opening:
- `BackendKind` knows `llama-server` only;
- `RECIPES` holds llama.cpp only, and `TagPattern` reads its `bNNNN` tags only;
- the gateway's configuration passes the key through `LLAMA_API_KEY`, which only llama-server
  reads;
- the bench speaks the chat API only.

This roadmap adds the three servers with the guarantees llama-server has: a native build ai-ctl
keeps, an inventory that checks every value, the gateway's key, and `doctor`. Its code is also
the first real use of the next version of `reviewing-delivered-code`, which waits for it.

---

## Decisions Taken At Opening

- **The three ggml servers:** whisper.cpp's `whisper-server`, audio.cpp's `audiocpp_server` and
  stable-diffusion.cpp's `sd-server`, built natively by the existing builder (the user,
  2026-10-19).
- **Measured by probes and by real use:** each model's VRAM and time to ready in phase 0, one real
  request per server through the gateway in phase 3. The bench stays with the LLM.
- **OpenWebUI is wired in phase 3**, for speech-to-text, text-to-speech and image generation
  through the gateway.
- **Every server requires ai-ctl's key**, as `docs/security.md` guarantees for llama-server. A
  server that cannot require one is not served until the user decides how it is guarded, on
  phase 0's probe. None of the three can: each recipe applies a patch that makes its server
  require the key in `LLAMA_API_KEY` on every route but its health check (the user,
  2026-10-20).
- **Facts are probed, never written from memory.** Each server's options, endpoints and readiness
  come from captures on this machine, listed in `tests/fixtures/README.md`. The facts go to the
  documents of their domains, `docs/audio/` and `docs/image/`, with their sources.
- **Several models per use**, served side by side so that the one suited to a need can be
  chosen and new ones tried (the user, 2026-10-20, replacing "one model per use"): the
  candidates phase 0 compares and keeps, each pinned and verified under the provenance rule of
  `docs/stack.md` §5, written with the user for models other than LLM GGUF files, and downloaded
  into `/ai/models/<use>/<id>/`.
- **Phase 0's closure adjusts phases 1 to 3** to what the probes found, with the user: a server
  may need an option, a patch or a guard that this plan does not foresee.
- **Tests and commits follow CLAUDE.md:** test-first; each commit's code diff through
  `cargo mutants --in-diff`, a missed mutant killed or justified in the report; a new test that
  spawns a process, binds a port or takes a lock run 30 times before its commit; one Conventional
  Commit per unit of work, with the five checks green.
- **No closing review.** This roadmap does not run `reviewing-delivered-code` as it stands, which
  scopes a review by file and would review the whole crate again. The skill's next version,
  built after this roadmap, reviews its code as its first real use.
- **Machine at opening:** Fedora 44, NVIDIA driver 615.78.08, CUDA 13.4, llama-swap v263, native
  build `b11700` of llama.cpp, podman 5.8.9, git 2.56.0, CMake 4.3.0, rustc 1.97.1.

---

## Deliberately Out Of Scope

- Voxtral through vLLM-Omni and Fish Speech: Python backends need a new kind of source, a
  command run on the host from a virtual environment, left to a later roadmap.
- Official container images of the new servers: native builds only.
- Bench scenarios for audio and image.
- sd.cpp's own web UI, for the image editing OpenWebUI lacks: added as phase 4 on 2026-10-30,
  then removed by the user the same day, image editing not being a priority.
- The change to `reviewing-delivered-code`.
- The orchestrator, a separate project; a bridge between the agents and ai-ctl; a model library
  managed by ai-ctl; vLLM as a second LLM engine; models larger than the VRAM; backends on unix
  sockets.
- The open items of the earlier roadmaps' summaries.

---

## Phases

- 🟢 [Phase 0: Probes And Decisions](phase-0-probes-and-decisions.md) — each server built by hand, run behind llama-swap and measured; its models chosen and verified; the key and audio.cpp decided with the user
- 🟢 [Phase 1: Native Builds](phase-1-native-builds.md) — a recipe per project, its tag pattern, and `build list | use | prune` across four projects
- 🟢 [Phase 2: Inventory And Gateway](phase-2-inventory-and-gateway.md) — a backend kind per server, the inventory's checks for each, and its llama-swap entry with readiness, key and matrix
- 🟢 [Phase 3: End To End](phase-3-end-to-end.md) — the shipped inventory served live through the gateway, swaps against the 27B, OpenWebUI wired, documents updated

---

## Dependencies

- llama-swap routes the audio and image endpoints and starts any server with `${PORT}`
  (`docs/gpu/llama-swap.md`, checked 2026-10-08 and 2026-10-09); phase 0 checks it on the
  installed v263.
- The CUDA toolkit and host compilers of `docs/toolchain.md`.

---

## Related Documentation

- [docs/gpu/llama-swap.md](../../../gpu/llama-swap.md) — the GPU's arbitration and the backends it plans
- [docs/security.md](../../../security.md) — the key, the loopback, what ai-ctl runs
- [docs/stack.md](../../../stack.md) — the LLM stack, the model provenance rule (§5), OpenWebUI (§7.3)
- [docs/toolchain.md](../../../toolchain.md) — the build toolchain
- [docs/commands.md](../../../commands.md) — every command
- [docs/architecture.md](../../../architecture.md) — layers, ports, testing
- [v2-review-fixes summary](../../completed/v2-review-fixes/summary.md) — what the previous roadmap left open

---

## Metadata

**Roadmap Status:** 🟢 Done
**Created:** 2026-10-19

