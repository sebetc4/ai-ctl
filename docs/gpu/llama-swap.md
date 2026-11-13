# GPU Arbitration: llama-swap

One RTX 5090 Laptop with 24 GB of VRAM serves every local AI workload, and llama-swap is the only
component that starts or stops a GPU backend. This document says how the GPU is shared, why
llama-swap holds that role, and what it means for the backends to come, so the analysis does not
have to be redone when they arrive. Decided on 2026-10-09; the facts about llama-swap were checked
on 2026-10-08 and 2026-10-09.

---

## How the GPU Is Shared

Three pieces, each with one responsibility:

```
Claude's agents ──► agents' client ──┐
OpenCode, OpenWebUI ─────────────────┼──► llama-swap (single entry point, no VRAM of its own)
                                     │      ├─ llama-server     ├─ whisper-server
ai-ctl ──────────────────────────────┘      └─ audiocpp_server  └─ sd-server
 (doctor, builds, inventory,
  gateway config, up/down, bench)
```

1. **llama-swap is the only arbiter.** It is the only component that starts or stops a GPU backend,
   and every client goes through it.
2. **ai-ctl** does what llama-swap does not: diagnostics of the machine, native builds of the ggml
   projects, a declarative **inventory** of backends and models with their VRAM budgets, generation
   of the llama-swap configuration from it — each model's budget included, which llama-swap's
   `capacity` router counts —, `up` and `down` of the gateway, and benchmarks. Nothing runs between `down` and `up`.
3. **The agents' client** is a separate project: a command-line tool through which Claude Code's
   agents delegate tasks to the models the gateway serves, files in and a file out. It knows
   tasks, not VRAM: it only calls the gateway, picks a model by what `/v1/models` says it can do
   unless its setting names one, and groups tasks by domain to limit swaps.

What follows from it:

- **llama-server's router mode is not used.** llama-swap is the only layer that swaps. Each model
  runs as a plain `llama-server` with its own flags, so the `capacity` router counts each model's
  VRAM, and an idle `ttl` stops the process, freeing everything including the CUDA context.
- **Clients see one endpoint.** OpenWebUI and OpenCode point at llama-swap on the host's
  `127.0.0.1:8080`, with the key ai-ctl creates
  ([security.md §6](../security.md#6-clients-that-hold-the-key)): `/v1/models` lists the
  inventory's models with what each can do ([below](#what-each-model-can-do)), the chat,
  Anthropic, speech and image endpoints are proxied, and a model switch restarts a process. Every client uses the same model IDs
  ([stack.md §7](../stack.md#7-client-configuration)).
- **A resident process exists**, holding no VRAM: llama-swap, between `ai-ctl up` and
  `ai-ctl down`.
- **Most switches are swaps.** Each chat model and each image model needs 21 to 23 GB and runs
  alone; only the speech models share the GPU, with one another. A sentence read aloud or an
  image asked from a chat evicts the chat model, which loads again for the next message
  ([openwebui.md](../openwebui.md)). The agents' client batches work by domain.

---

## The Backends

The shipped inventory, [`deploy/inventory.toml`](../../deploy/inventory.toml), serves four
backends, each with several models; each model's `vram_mib` is its peak measured on this machine:

| Domain | Backend | What it is | GPU notes |
|---|---|---|---|
| LLM | llama.cpp (`llama-server`), [stack.md](../stack.md) | GGUF inference, OpenAI and Anthropic APIs | 21094 and 22733 MiB for the two chat models: each runs alone |
| Speech-to-text | whisper.cpp (`whisper-server`), [audio/whisper-cpp.md](../audio/whisper-cpp.md) | ggml; `/v1/audio/transcriptions` | 2213 and 3895 MiB for large-v3-turbo and large-v3 |
| Speech, all tasks | audio.cpp (`audiocpp_server`), [audio/audio-cpp.md](../audio/audio-cpp.md) | ggml; TTS, voice cloning, ASR (110+ model families); `/v1/audio/speech`, `/v1/audio/transcriptions`, `/v1/tasks/run` | 5548 MiB for Qwen3-ASR, 5542 to 12768 MiB for the five TTS models, the cloning ones measured with the voice library's longest reference |
| Image | stable-diffusion.cpp (`sd-server`), [image/stable-diffusion-cpp.md](../image/stable-diffusion-cpp.md) | ggml; `/v1/images/generations`, `/v1/images/edits` | 22773 to 23514 MiB for the three models, each the peak of the largest request it serves, their weights in RAM (16 to 21 GiB): each runs alone |

The clients are OpenCode, OpenWebUI, scripts, and Claude Code's agents through the agents'
client. Several backends default to port 8080; behind llama-swap each gets its own (below).

**Several models per use** (user's decision, 2026-10-20). Speech-to-text, text-to-speech and image
generation may each have several models, on one backend or several. Each is an entry of the
inventory, and llama-swap serves them side by side, so that a client picks by the request's
`model` the one suited to its need, and a new model or backend can be tried beside the others. A
comparison drops only a model that fails. Set aside: one model per use, chosen once.

### To Come

- **Python backends**: Voxtral through vLLM-Omni and Fish Speech, each a command run from its own
  virtual environment, a kind of source ai-ctl does not have yet. audio.cpp serves Voxtral 4B TTS
  and Fish S2 Pro meanwhile.
- **stable-diffusion.cpp's own web UI**, served by `sd-server` through the gateway with the key,
  for the editing OpenWebUI lacks: a mask, the seed, the steps, the strength.
- **The agents' client**, in its own repository: a command-line tool with a skill that says when
  delegating pays, its first task image generation, calling only the gateway; an MCP adapter on
  the same core later, if a client without a shell needs one.

---

## What Each Model Can Do

The configuration ai-ctl generates declares each model's `capabilities`, from the inventory alone
and by its backend's kind, and llama-swap shows them in `/v1/models` before any model is loaded:
a client picks a model by what it can do, with no list of models of its own. ai-ctl declares
what each model can do, and never chooses one for a client (user's decision, 2026-11-07, phase 0
of `capabilities-status-and-updates`).

| Model | `in` | `out` | `tools` | `context` |
|---|---|---|---|---|
| `llama-server`, no `mmproj` file | text | text | true | the window of one request |
| `llama-server`, an `mmproj` file | text, image | text | true | the window of one request |
| `whisper-server` | audio | text | — | — |
| `audiocpp_server`, `task = "asr"` | audio | text | — | — |
| `audiocpp_server`, `task = "tts"` | text | audio | — | — |
| `audiocpp_server`, another task or none | nothing declared | | | |
| `sd-server` | text, image | image | — | — |

- **What a client reads.** llama-swap renders them as `architecture.input_modalities` and
  `output_modalities`, a `capabilities` map derived from them — `vision` for an image in,
  `audio_transcriptions`, `audio_speech`, `image_generation` for text in and an image out,
  `image_to_image`, `function_calling` for `tools` — and `context_length` from `context`. An
  image model takes reference images (`POST /v1/images/edits`), so it carries `vision` too: a
  client tells an image model from a chat model by `output_modalities` or `image_generation`,
  never by `vision`.
- **`context`, the window of one request**: `ctx_size`, divided by `parallel` when the slots do
  not share a unified KV cache (`kv-unified`), and at most `kv-unified-per-slot` when set. With no
  `parallel`, llama-server takes four slots and unifies their cache, which leaves each request the
  whole `ctx_size` (`tools/server/server.cpp` at llama.cpp b12111). llama-server pads its context
  to 256 tokens, which the declared window leaves out, so that it never exceeds a slot's context.
  It also caps a slot at the model's training context, which the inventory does not read: within
  it, llama-server refuses no prompt shorter than the window (`n_ctx_slot` in
  `tools/server/server-context.cpp`).
- **`tools` for every llama-server model.** Every chat model the inventory serves calls tools, for
  OpenCode ([stack.md §7.2](../stack.md#72-opencode)). A model that cannot, such as an embedding
  model, gets a field of its own in the inventory when one enters it.
- **The declared values are the only ones.** After a model starts, llama-swap asks its backend's
  `/v1/models` what it supports, without the key; every backend ai-ctl runs answers 401, so on
  ai-ctl's gateway that discovery never completes, whatever the release (probed on v270 on
  2026-11-07). `disableAuto` is not written: it would only stop using a result that never comes.
- **No voices.** `capabilities.voices` would answer `GET /v1/audio/voices` without starting the
  model, but ai-ctl would read the voice library at each `up`, and cannot list Voxtral's presets,
  embedded in its GGUF: the server answers, once its model has started.
- **Out of the inventory's reach**: an `mmproj` that reads audio, which llama-server would serve,
  is declared as image; an `mmproj` given as a flag rather than a file is not counted.

Checked through the gateway on v271 on 2026-11-09, the twelve models unloaded and the 27B
starting: each `sd-server` model showed `image` in `output_modalities` and `image_generation`,
each chat model `vision` and `function_calling` with its window, and no `image_generation`.

---

## Why llama-swap

The question was whether to keep ai-ctl for LLMs only and create one controller per domain (audio,
image, video…). A controller that does not know the others' state may start a backend and load a
model while the VRAM is already taken. Four options were weighed.

### A. One independent controller per domain — rejected

The domain-specific part is small: a backend's command line, its health check, its model format.
The common part is large: four of the six backends are ggml projects built with the same CMake/CUDA
recipe, and lifecycle, VRAM accounting, ports, diagnostics and benchmarks are shared. Three
controllers mean three copies of that, plus a coordination protocol to invent — and without it, two
controllers load models into the same VRAM.

### B. Controllers coordinating through a shared lock and VRAM budgets — rejected

A file lock with declared budgets serialises the controllers, but it fails on two points:

- **It does not see the traffic.** Clients talk to the backends directly, so the lock cannot tell
  whether a backend is in the middle of a generation before stopping it.
- **Backends load on their own.** llama-server in router mode reloads its model on the first request
  after `--sleep-idle-seconds`: 22 GB arrive while the lock gave the VRAM to another backend.

The arbiter has to sit on the path of the requests.

### C. A gateway written for ai-ctl — rejected

It would reimplement llama-swap, which already covers every backend listed above.

### D. llama-swap as the single entry point — accepted

---

## What llama-swap Offers

Checked on 2026-10-08 and 2026-10-09:

- **llama-swap** (Go, MIT) proxies any OpenAI- or Anthropic-compatible server and names llama.cpp,
  vLLM, stable-diffusion.cpp, audio.cpp and ComfyUI; its Docker image also builds whisper.cpp.
- **Endpoints:** `/v1/chat/completions`, `/v1/completions`, `/v1/responses`, `/v1/embeddings`,
  `/v1/models`, `/v1/audio/speech`, `/v1/audio/transcriptions`, `/v1/audio/voices`,
  `/v1/images/generations`, `/v1/images/edits`, `/v1/messages`, rerank and infill endpoints,
  `/sdapi/v1/*` for stable-diffusion.cpp, `/audioapi/v1/tasks/run` for audio.cpp.
- **`/upstream/:model_id`** gives direct access to a backend, for the endpoints llama-swap does not
  route (`/tokenize`, `/metrics`, `/slots`…).
- **The `capacity` router**, since v267, read in v270's source on 2026-11-07: pools with a limit
  each, in units the administrator chooses, and each model's `capacities` in them. Models whose
  requirements fit every pool run together; to start one that does not fit, llama-swap evicts,
  among the running models, those whose `evict_cost` adds up to the least, then the fewest, then
  by ID, and past 100000 steps of that search keeps a plan that fits. **It measures no VRAM**:
  the requirements are the administrator's, and a model that declares none takes nothing. It
  replaces `groups` and the `matrix`, whose sets list the models allowed together.
- Per-model `ttl`, `POST /api/models/unload` and `/unload/:model_id`, `/running`, `${PORT}` for
  dynamic ports, `cmd` + `cmdStop` for containers, log streams per model, `/health`.
- **`hooks.on_startup.preload`** starts the models the inventory marks `preload = true` once the
  gateway runs, by sending each a `GET /` and taking a status of 400 or more for a failure (v271's
  `startPreload`, read on 2026-11-12). A llama-server registers `/` only with its web UI, which
  ai-ctl's build leaves out ([security.md §7](../security.md#build-sources)),
  so it answers 404 once its model is loaded: the gateway's log reads `failed to preload model
  <id>: status 404` beside `model loaded`, and the model answers.

The audio and image routes, probed on v263 on 2026-10-20 with a fake upstream that logs each
request, then with each server behind the gateway:

- **Routing by the request's `model`.** llama-swap reads it from a JSON body or a multipart form
  alike. It forwards each of these requests under its own path: `POST /v1/audio/transcriptions`,
  `POST /v1/audio/speech`, `GET /v1/audio/voices?model=…`, `POST /v1/images/generations`,
  `POST /v1/images/edits` and `POST /sdapi/v1/txt2img`.
- **`POST /audioapi/v1/tasks/run`** is forwarded as `/v1/tasks/run`.
- **A request that names no model** cannot be routed. `GET /sdapi/v1/sd-models` gets llama-swap's
  404 "404 page not found", even with `?model=`.
- **`/upstream/<model>/<path>`** forwards any path.
- **Headers.** Each forwarded request carries the client's `Authorization: Bearer <key>`
  unchanged, its `Host`, and an added `X-Forwarded-For: 127.0.0.1`. llama-swap's own health
  check and its `GET /v1/models` after a start carry no key.

The models' ports, read in v263's source (`internal/config/macros.go`) and probed on 2026-10-29:

- **A port per model, fixed when the configuration loads.** llama-swap gives each model whose
  `cmd` holds `${PORT}` the next port from `startPort`, 5800 by default, in the byte order of the
  model ids, and the model's server binds it whenever the model starts. Every model ai-ctl writes
  holds `${PORT}`, and ai-ctl sets no `startPort`. A model keeps its port while the configuration
  does not change; at the next `up`, a model added or removed whose id sorts before another's
  moves that one to another port, which another server may just have left.
- **A port another server left in TIME-WAIT binds.** A backend that llama-swap stops closes its
  connections first, which leaves TIME-WAIT sockets on its port for 60 s. Every server ai-ctl
  runs binds with `SO_REUSEADDR`: `llama-server` and `audiocpp_server` by their own code
  (audio.cpp's `app/server/http.cpp` at `v0.9.3`), `whisper-server` and `sd-server` by ai-ctl's
  `reuse-address.patch`
  ([security.md §7](../security.md#build-sources)). Each started on
  such a port, whatever server left it. Behind ai-ctl, an inventory of three models, then the
  same with `flux-3-image-klein`, whose id sorts first: after `down` and `up`, `sd-server`,
  `audiocpp_server` and `llama-server` each got the port of a server of another kind, 2 to 6
  TIME-WAIT sockets on it, and answered 200. By hand, each of the four servers started on a port
  another kind had left in TIME-WAIT, while a `whisper-server` built without the patch, binding
  with cpp-httplib's `SO_REUSEPORT`, exited 1 there, "couldn't bind to server socket".

---

## Sources

- [llama-swap](https://github.com/mostlygeek/llama-swap)
- [llama-swap #1242 — the `capacity` router](https://github.com/mostlygeek/llama-swap/pull/1242)
- [audio.cpp](https://github.com/0xShug0/audio.cpp)
- [whisper.cpp](https://github.com/ggml-org/whisper.cpp)
- [miscripts — image generation through stable-diffusion.cpp](https://github.com/aaaaadrien/miscripts/tree/main/ia/image-generation)
