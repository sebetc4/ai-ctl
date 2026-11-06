# Local Inference Stack

The reference for the LLM stack ai-ctl serves: which engine, which models, which settings, which
clients, and how to measure them. It rewrites the October 2026 study of local inference on this
machine, corrected by the research of 2026-10-08 and adapted to ai-ctl v2: native builds, one
gateway — llama-swap — that arbitrates the GPU, and nothing running between `ai-ctl up` and
`ai-ctl down` ([gpu/llama-swap.md](gpu/llama-swap.md)).

Values marked *(estimated)* are calculations still to be confirmed on the machine.

The model settings live in the inventory, [`deploy/inventory.toml`](../deploy/inventory.toml),
which `ai-ctl init` installs; that file is the source of truth for every value quoted in section 6.

---

## 1. Decisions at a Glance

| Topic | Decision |
|---|---|
| Engine | **llama.cpp** (`llama-server`), one process per model, the only LLM engine |
| Runtime | **Native build** from source for sm_120a by default; the official `server-cuda` and `server-cuda13` images as a second runtime, for comparison |
| Gateway | **llama-swap**, the single entry point: it starts and stops the backends by the request's `model` field; **ai-ctl** runs it between `up` and `down`; no systemd unit |
| Second engine | **vLLM**, only for a targeted NVFP4 test. **SGLang abandoned** on this machine |
| Main model | **Qwen3.8-27B**, GGUF UD-Q4_K_M + MTP head |
| Fast model | **Qwen3.6-35B-A3B**, GGUF UD-IQ4_XS |
| Concurrency | **1 slot**: one request at a time, with the whole context |
| Context | 96K tokens with MTP; ~160K without MTP *(estimated)* |
| Endpoint | The gateway: `http://127.0.0.1:8080/v1` (OpenAI) and `/v1/messages` (Anthropic) |

---

## 2. Hardware and Usage Profile

| Component | Value |
|---|---|
| GPU | RTX 5090 Laptop, 24 GB GDDR7, ~896 GB/s, Blackwell sm_120 |
| CPU | Intel Core Ultra 9 275HX |
| RAM | 64 GB DDR5 |
| OS | Fedora 44 |
| Driver | NVIDIA 615.78.08, CUDA driver API 13.4 (2026-10-15) |

Uses, by priority:

1. **Sub-agent of an orchestrator**: code, text, tasks within its reach. Sub-agents run **one after
   the other**. A global task may take longer than with five agents in parallel; in exchange, each
   request gets a larger context and a higher throughput.
2. **OpenCode** in agentic mode, for code.
3. **OpenWebUI** and its plugins, for chat.

### What Matters with One Request at a Time

With a single active request, generation is bound by **memory bandwidth**: every token reads all
the model's weights again. An engine's multi-user scheduler plays almost no role. The real levers,
in order:

1. the **size of the weights** (the quantization);
2. the **prefill speed** on long agent prompts;
3. the **reuse of the prompt cache** between two turns of the same agent;
4. **speculative decoding** (MTP).

Order of magnitude *(estimated)*: 896 GB/s ÷ ~16 GB of weights gives a ceiling of about 55 tok/s
for a dense 27B, before MTP.

---

## 3. The OCTO Article of May 2026

[The article](https://blog.octo.com/llama.cpp-sglang-vllm--quel-framework-d%27inference-llm-choisir-pour-votre-assistant-de-code)
compares llama.cpp, vLLM and SGLang for a coding assistant, with Devstral-Small-2-24B.

**Useful for the concepts**: the role of the KV cache, prefill versus decode, each engine's
philosophy.

**Useless for choosing on this laptop**:

- The hardware is datacenter-grade: an L40S 48 GB for llama.cpp, two H100 80 GB for vLLM and
  SGLang, for teams of 2 to 100 users.
- llama.cpp runs neither on the same GPU nor with the same quantization as the other two. **Only
  the vLLM / SGLang comparison is fair.**
- The article states that vLLM recomputes the common prefix on every request. **That is wrong**:
  automatic prefix caching is on by default in vLLM's V1 engine.
- llama.cpp is underrated there: it has continuous batching, parallel slots and a prompt cache in
  RAM.

**Verdict**: a three-engine comparison is not worth it for this use case.

---

## 4. Engine and Runtime

### 4.1 llama.cpp as the Main Engine

| Argument | Detail |
|---|---|
| VRAM | GGUF quantizes everything, including the large embedding table (248K-token vocabulary). Qwen3.8-27B fits in 16.5 GB in UD-Q4_K_M. vLLM's recipe for the same model in NVFP4 announces 24.6 GiB (weights + FP8 KV) on one GPU: above 24 GB. |
| Preallocation | vLLM and SGLang reserve VRAM up front. The April 2026 bench on Qwen3.5-9B showed it: 7.1 GB for llama-server in Q5_K_M against 23.6 GB for SGLang in AWQ. |
| RAM put to use | Prompt cache in RAM (`--cache-ram`), MoE expert offload (`--n-cpu-moe`): the 64 GB serve a purpose. |
| Router mode | Several models behind one endpoint, one in VRAM, switched by the request's `model` field. |
| MTP | With one request, about 1.4× announced on dense models with 2 draft tokens. |
| Startup | Loading in ~10–20 s *(estimated)*, against minutes for vLLM and SGLang (compilation, preallocation). |

### 4.2 vLLM: a Second Engine, Tested on a Criterion

**What would make it interesting:**

- **NVFP4 W4A4** on Blackwell really uses the FP4 tensor cores. It speeds up **prefill** above all,
  the weak point of a laptop GPU facing 40K-token agent prompts.
- NVIDIA publishes `Qwen3.6-27B-NVFP4` (June 2026) and Unsloth an NVFP4 Qwen3.6-27B with MTP.

**What holds it back here:**

- Qwen3.8-27B in NVFP4 does not fit in 24 GB: it would mean going back to Qwen3.6-27B, clearly
  weaker at agentic work (section 5).
- The community figures for "one RTX 5090" (~80 tok/s, 218K context with NVFP4 and MTP) come from
  the **desktop** card, which has 32 GB. They do not transfer to this 24 GB laptop.
- Prefix caching for hybrid models (Gated DeltaNet) is still **experimental**: `align` mode, block
  boundaries of 784 tokens on the 27B in LMCache's recipe. A July 2026 change added
  `--enable-mamba-fine-grained-prefix-cache`, which can reuse a prefix ending mid-block, but only
  under conditions (align mode, MTP or EAGLE speculative decoding); it is off by default.
- One vLLM process serves one model, and startup is slow.
- With sequential sub-agents, its main strength — the multi-request scheduler — goes unused.

**Criterion to run the test**: **prefill dominates** the tasks' time — for example every call sends
more than 30–40K new tokens and the prompt cache reuses little. Then compare "Qwen3.6-27B NVFP4 +
vLLM" with "Qwen3.8-27B + llama.cpp" using the protocol of section 9.

### 4.3 SGLang: Abandoned for This Machine

- Same constraints as vLLM (everything in VRAM, Hugging Face formats), with no advantage of its own
  here.
- RadixAttention shines when **many concurrent requests** share a prefix. Sequentially, llama.cpp's
  prompt cache already covers the case of an agent sending the same system prompt 20 times.
- Few reliable reports on consumer RTX 50 cards. LMCache does not validate it for the hybrid Qwen
  models.
- **Only exception**: an orchestrator written with SGLang's frontend language.

### 4.4 Native Build or Official Image

Facts checked on 2026-10-08 in the llama.cpp repository:

| | Native build | Official image |
|---|---|---|
| Freshness | Any commit, including an unmerged pull request | Rebuilt **once a day** (`cron: '12 4 * * *'`, "rather than on every push because it is expensive"); a failed run leaves it one or two days behind. Releases (`b####`) come out about ten times a day. |
| Blackwell kernels | `CMAKE_CUDA_ARCHITECTURES=120a-real` | `server-cuda`: CUDA 12.8.1, default architecture list including `120a-real`. `server-cuda13`: CUDA 13.4.1. |
| Host needs | NVIDIA driver, CUDA toolkit, a host compiler `nvcc` accepts, CMake, git | NVIDIA driver, podman, NVIDIA Container Toolkit, a CDI specification |
| Rollback | Versioned builds kept side by side | Image tag (`server-cuda-b####`), pinned by digest |
| Failure modes | Toolkit bugs (CUDA 13.2), compiler too new for `nvcc` | A CDI specification left on an older driver, SELinux labels, mounted paths |

The `a` in `120a` matters: it enables the instructions specific to sm_120, such as the FP4 tensor
cores. Compiling for plain `120`, as llm-ctl v1 did, leaves them out.

Neither side is faster by construction: both run native Blackwell kernels, and the remaining
difference — a generic CPU backend in the image, a CPU-tuned one in a native build — does not count
when the whole model sits in VRAM. ai-ctl builds natively by default for the exact commit and the
control over flags, and keeps the images as a second runtime so the bench can settle it.

The gateway runs an image with the same llama-server arguments as a native build, after
`podman run --rm --replace --name ai-ctl-<model> --pull=never --no-healthcheck --network host
--security-opt label=disable --device nvidia.com/gpu=all --volume <models_dir>:<models_dir>:ro
<image>`, and stops it with `podman stop ai-ctl-<model>`. The shipped inventory names each image
by tag and digest, `<image>:<tag>@sha256:<digest>`: podman runs the content of the digest, which
no registry can move as it moves a tag, and `ai-ctl up` warns of an image named by its tag only. Checked on 2026-10-10 with podman 5.8.7
and SELinux enforcing:

- `--security-opt label=disable`: a confined container could neither read the models (`/ai` is
  `unlabeled_t`) nor initialise NVML through the CDI device ("Insufficient Permissions"). Without
  SELinux separation it does both, and no label on the host changes; the container stays rootless
  and the mount read-only.
- `--network host`: the server binds `127.0.0.1:${PORT}` as a native build does, with no port
  forwarder between the gateway and the server the bench measures.
- `--no-healthcheck`: the image's `HEALTHCHECK` probes `localhost:8080`, which on the host's
  network is the gateway. llama-swap runs its own check.
- `--pull=never`: a first pull, several GB, would outlast llama-swap's 120 s health check, so
  `ai-ctl up` pulls a missing image before starting the gateway.
- `--replace`: a container left behind by a crash does not block the next start. llama-server
  stops on SIGTERM (`tools/server/server.cpp`), which `podman stop` sends first.

---

## 5. Models

| | Qwen3.8-27B | Qwen3.6-35B-A3B |
|---|---|---|
| Role | Main: quality | Fast: simple tasks in volume |
| Architecture | Dense 27B, hybrid attention | MoE 35B, ~3B active, hybrid attention |
| Files | UD-Q4_K_M (16.5 GB) + MTP Q4_0 (1.37 GB) | UD-IQ4_XS (17.7 GB) |
| Vision | Yes, `mmproj-F16` (~0.9 GB, kept on the CPU) | Yes, `mmproj-F16` |
| Native context | 262K | 262K |
| Licence | Apache 2.0 | Apache 2.0 |

**Vision on the CPU.** Both projectors stay on the CPU (`no-mmproj-offload`), which saves their
0.9 GB of VRAM but makes an image slow to read. llama-server caps a Qwen-VL image at 4096 tokens
by default (`tools/mtmd/clip.cpp:1756` at `b11700`), a 2048×2048 image or a phone's photo; the
27B read one in 231 s, and a 1024×1024 image, 1024 tokens, in 24 to 30 s (2026-10-30, through
OpenWebUI). OpenWebUI compresses the images it sends to 1024 a side
([openwebui.md §7](openwebui.md#7-images)). On the GPU, the 27B's projector would add its 0.9 GB
and its working set to a budget of 22733 MiB.

Every file name above exists in the Unsloth repositories (checked 2026-10-08). Unsloth re-uploaded
the Qwen3.8-27B GGUFs on 2026-08-19/20 with its Dynamic V3.0 quantization (about 10 % better
accuracy at the same size): sizes measured before that date should be checked again.

### Where Models Come From

- **GGUF files come from Unsloth** (`unsloth/<model>-GGUF`), preferred for quality, with its
  Dynamic (`UD-*`) quantizations; never from a re-upload by another account, even one claiming to
  be identical (user's decision, 2026-10-09).
- **The other models come from their publisher, in a format their engine reads** — speech-to-text,
  text-to-speech, image generation; failing that, from the engine project's own conversions
  (`ggerganov/whisper.cpp`, `audio-cpp/audio.cpp-gguf`, `leejet/…`). A repackaging by a third
  party, such as Comfy-Org, is admitted only for a file the publisher does not offer in a format
  the engine reads, named file by file; none is admitted today. An LLM used as an image model's
  text encoder comes as a GGUF from its publisher or from Unsloth (user's decision, 2026-10-20).
  Set aside: no third party at all, which seemed to rule out Qwen-Image 2.1 until
  stable-diffusion.cpp was found to load Qwen's own diffusers VAE (probed 2026-10-20 at
  `master-975-95b3090`); Comfy-Org as a general source. A publisher's format the engine does not
  read is, for instance, Mistral's `consolidated.safetensors` with voices as `.pt` pickles for
  Voxtral, or Fish Audio's `codec.pth`: audio.cpp reads its own GGUF of them.
- **Every download is checked before its first use**, since a model file can be tampered with:
  1. the repository is one the rules above admit, and the commit downloaded is recorded
     (`hf download --revision`);
  2. the SHA-256 of each file equals the Hub's LFS hash at that commit (`lfs.sha256` returned by
     `HfApi.get_paths_info` in `huggingface_hub`); a small file the Hub keeps outside LFS, such as
     a `config.json`, is checked by its git blob SHA-1 against the Hub's `oid`. A whisper model is
     also checked against the SHA-1 that whisper.cpp's `models/README.md` gives at the tag built;
  3. for an LLM's GGUF, the chat template embedded in it (`tokenizer.chat_template`), which
     llama-server applies to every request, is read and compared with the base model's. A
     poisoned template changes answers or inserts links while the weights stay untouched, and such
     files passed Hugging Face's automated scans (Pillar Security, July 2025; arXiv 2602.04653): a
     difference must be explained by a fix Unsloth documents. The other engines need no such
     check today: stable-diffusion.cpp reads no `chat_template` from a text encoder's GGUF and
     builds its prompts itself (no match in its `src/` or `examples/` at `master-975-95b3090`),
     and audio.cpp reads one only for its families `outetts`, `ace_step` and `higgs_audio_tts`
     (at `v0.9.2`): a model of those families would need it.
- **The commands**, here for the 27B (the other model differs only by its names); the download
  itself is in §8. The hashes are read at the pinned commit, and the template with llama.cpp's own
  reader from the clone `ai-ctl build` keeps (`uvx` fetches its numpy):

  ```bash
  REPO=unsloth/Qwen3.8-27B-GGUF BASE=Qwen/Qwen3.8-27B DIR="$MODELS_DIR/llm/qwen3.8-27b"
  FILES=(Qwen3.8-27B-UD-Q4_K_M.gguf mmproj-F16.gguf MTP/mtp-Qwen3.8-27B-Q4_0.gguf)
  REV=$(curl -sf "https://huggingface.co/api/models/$REPO/revision/main" | jq -r .sha)
  # after the download at $REV: one OK line per file
  curl -sf "https://huggingface.co/api/models/$REPO/revision/$REV?blobs=true" \
    | jq -r --args '.siblings[] | select(.rfilename | IN($ARGS.positional[]))
                    | "\(.lfs.sha256)  \(.rfilename)"' "${FILES[@]}" \
    | (cd "$DIR" && sha256sum -c -)
  # the base model is general.base_model.0.repo_url in the same dump
  uvx --from ~/.local/share/ai-ctl/sources/llama.cpp/gguf-py gguf-dump --no-tensors --json \
      "$DIR/${FILES[0]}" | jq -j '.metadata["tokenizer.chat_template"].value' > gguf.jinja
  curl -sfL "https://huggingface.co/$BASE/raw/main/chat_template.jinja" -o base.jinja
  diff -u base.jinja gguf.jinja
  ```

- **Checked on 2026-10-10**, at the commits §8 pins: every SHA-256 equal to the Hub's. Both
  templates differ from Qwen's (`Qwen/Qwen3.8-27B` at `1d4bf0f`, `Qwen/Qwen3.6-35B-A3B` at
  `995ad96`) in control flow only: they add no text to the prompt, no link and no character
  outside ASCII, and end with a comment naming Unsloth's fixes. Documented by Unsloth on both
  model cards: the `developer` role treated as `system`, and tool calling (nested arguments
  rendered as JSON; arguments taken only as an object, so a JSON string raises an error in the 27B
  and renders no parameter in the 35B). Not documented there: the leading
  system and developer messages merged into one (named in the 27B template's comment; the 35B
  merges two at most and drops a later system message silently, where the 27B raises an error), a
  conversation with no user message accepted, and, in the 27B only, `reasoning_effort: "high"`
  read as `xhigh` — Qwen's template rejects `high`, which §7.1 passes on. The user accepted the
  templates as shipped, these three included, to be tested through the gateway.
- **ai-ctl neither downloads nor verifies models**: the commands above are run by hand, and the
  inventory holds no revision or hash (user's decision, 2026-10-10). A model library managed by
  ai-ctl — pins, downloads, checks, template review — is a direction for a later roadmap.

### Why Qwen3.8-27B

- Released on **2026-08-14**. It replaces Qwen3.6-27B at the same size, so the same memory footprint.
- A clear step up in agentic work according to Alibaba: Terminal Bench 2.1 from 63.4 to 73.0,
  SWE-bench Pro from 53.5 to 61.7.
- **Hybrid architecture**: 16 blocks of (3 Gated DeltaNet layers + 1 attention layer). Only
  **16 layers of 64** keep a KV cache, with 4 KV heads of dimension 256. That makes about
  **64 KiB per token in f16** and **~34 KiB in q8_0** *(calculation)*. The other 48 layers keep a
  fixed-size state.
- **Adjustable thinking**: `xhigh` by default, `medium` and `low` available, and it can be turned
  off.
- It needs a recent llama.cpp: older CUDA builds produced corrupted output on its DeltaNet layers
  (llama.cpp discussion #27164).

### Recommended Sampling (Qwen, Unsloth)

| Parameter | Thinking, general | Thinking, precise code | No thinking |
|---|---|---|---|
| temperature | 1.0 | 0.6 | 0.7 |
| top_p | 0.95 | 0.95 | 0.80 |
| top_k | 20 | 20 | 20 |
| min_p | 0.0 | 0.0 | 0.0 |
| presence_penalty | 0.0 | 0.0 | 1.5 |

The server defaults to "precise code". Clients override per request.

---

## 6. Server Configuration

### 6.1 Architecture

```
Orchestrator ─┐
OpenCode ─────┼──► llama-swap 127.0.0.1:8080 ──► llama-server (one per model, on ${PORT})
OpenWebUI ────┘        │                           native build (default) or official image
                       └─ configuration generated by ai-ctl from the inventory

ai-ctl up | down ──► starts and stops llama-swap
```

llama-swap reads the `model` field of each request, stops the running backend when the requested
model needs its place, starts the right one and forwards the request. Its `matrix` decides which
models may run together; with the shipped inventory, each chat model and each image model runs
alone, and only the speech models share the GPU, with one another (§6.4). llama-server's router
mode is not used: llama-swap is the only layer that swaps ([gpu/llama-swap.md](gpu/llama-swap.md)).

### 6.2 VRAM Budget of the 27B *(estimated)*

| Item | GiB |
|---|---|
| UD-Q4_K_M weights | 15.4 |
| MTP head + draft context | ~2.0 |
| KV q8_0, 96K tokens | 3.2 |
| DeltaNet state (1 slot) | ~0.15 |
| Compute buffers (ubatch 1024) | ~1.5 |
| **Total** | **~22.2** |

About 1.5 GiB of margin, provided the display runs on the iGPU. The variant without MTP reaches
~160K tokens within the same budget; it is in `deploy/inventory.toml`, commented out.

### 6.3 Model Settings

[`deploy/inventory.toml`](../deploy/inventory.toml) holds the settings with their reasons in
comments: a `[defaults.llama-server]` table that every llama-server model gets whatever its backend
(flash attention, batch sizes, `fit-target`, one slot, a 16 GiB prompt cache in RAM, the projector
on the CPU, sampling), the backends — the native build and the two official images — and one
`[[models]]` entry per model ID with its files, flags, context size, VRAM budget and idle `ttl`.

### 6.4 Server and Gateway Arguments

For each model, ai-ctl generates a llama-swap entry whose command runs the current build — or the
backend's image, under `podman run` (§4.4) — with the model's files, context size and flags plus:

```
llama-server --host 127.0.0.1 --port ${PORT} --alias <model id> --metrics <model flags>
```

`ai-ctl gateway config` prints the whole configuration. llama-swap assigns `${PORT}` and listens on
`127.0.0.1:8080` itself; it forwards to `http://127.0.0.1:${PORT}` once `/health` answers 200.
Any 200: llama-swap checks neither which process holds the port nor anything only the backend
knows ([security.md §2](security.md#2-what-listens) says what follows). And it reaches a backend
over TCP only. A probe of 2026-10-18 ran llama-swap v263 with one model served on a unix socket,
its `proxy` written `unix://<path>`, `http://unix:<path>` or `unix:<path>`: `-validate` accepts all
three, since it only parses the URL, but the health check never reached the backend, and with
`logLevel: debug` llama-swap logs `health check not ready: unsupported protocol scheme "unix"`.
The same backend on TCP passed.
Each entry also sets `env: ["LLAMA_API_KEY=${env.AI_CTL_API_KEY}"]`, and a `podman run` passes
`--env LLAMA_API_KEY` into its container: every backend holds the gateway's API key (§7). An
idle `ttl` of 1800 s stops the backend after 30 minutes without requests, which frees the VRAM
entirely, CUDA context included. `--alias` makes llama-server name the model as clients do.

Every llama-server entry sets **`disableKeepAlives: true`**: llama-server advertises keep-alive on a
streamed response, then closes the connection, and llama-swap writes the next request to that
closed connection when it arrives within a few tens of milliseconds: 502 `proxy error: EOF`, 3 in
156 streamed requests in llama-swap #1205. An agent sends its requests back to back. The cost is
one loopback connection per request. The cause, llama.cpp #29680, is open on 2026-10-10: once a
build in use includes its fix, the setting goes.

llama-swap reads every `${…}` in its file as a macro, with no escape, and splits `cmd` like a POSIX
shell line after dropping the lines that start with `#`: ai-ctl quotes each word for it and refuses
a path or flag value containing `${`.

The configuration uses llama-swap's **`matrix`** engine (`routing.router.use: matrix`), whose sets
list the models allowed to run together; a set also allows any subset of itself. ai-ctl writes one
set per largest combination of models whose `vram_mib` fit together in the GPU's usable memory —
NVML's total less the memory the driver reserves, 24075 MiB here — minus the inventory's
`vram_margin_mib`, 1024 by default, the same as `fit-target`: a model started beside another finds
the memory its `--fit` keeps free, and moves no layer to the CPU. A model too large for that budget
gets a set of its own. Evicting a model costs its `vram_mib`: llama-swap makes room by evicting the
running models whose budgets add up to the least, which are the quickest to load again. The 27B and
the 35B need 43827 MiB together, so each has a set of its own.

The sets grow combinatorially with the models that fit beside another, and llama-swap's start
slows faster than they do. Probed on v263 alone on a loopback port, each model's command a
`touch`, `/health` polled from its start: the shipped inventory's 32 sets answer in about 0.1 s,
10816 in 0.65 s, 53000 in 19.5 s and 84000 in 53.5 s, past the 30 s `up` waits for it before it
says that llama-swap did not answer and stops it. The shipped models and small ones of 900 to
9000 MiB, about 35 in all, reach that point.

Decided: ai-ctl writes 20000 sets at most, and refuses an inventory past them, saying how many
of its models can share the GPU and to take models out of it, so that `doctor`'s configuration
check, `gateway config` and `up` refuse it before any start (the user, 2026-11-05, in the fixes
of `audio-image-backends`' review). Set aside: 10000, the difference being about two small
models, and a warning alone, which leaves `up` to fail on its wait with no cause. 20000 sets is
not measured: about 2 s by the curve above.

### 6.5 Settings Not to Miss

- **`parallel = 1`**: requests arriving during a generation wait their turn. To chat in OpenWebUI
  while an agent runs, set `parallel = 2` **and** `kv-unified = true`. Without `kv-unified`, a fixed
  `parallel` splits the context into equal parts. MTP is less effective when two slots generate at
  once.
- **`reasoning-effort = medium`**: Qwen3.8's default `xhigh` is too long on a laptop. If the option
  has no effect, use `chat-template-kwargs = {"reasoning_effort":"medium"}`.
- **`--fit`** is on by default, with a 1024 MiB margin (`--fit-target`). When VRAM runs short it
  moves layers to the CPU instead of failing. **Check the logs** on the first start.
- **KV in q8_0**: if outputs turn incoherent or generation becomes very slow, go back to f16 and
  lower `ctx-size` to 49152.
- **MTP**: check in the logs that `draft-mtp` is loaded. The acceptance rate comes from `/metrics`
  (`llamacpp:spec_decode_num_accepted_tokens_total` against
  `llamacpp:spec_decode_num_draft_tokens_total`). Try `spec-draft-n-max` from 1 to 4 (the server's
  default is 3); beyond 2, acceptance drops sharply according to Unsloth.
- **`cache-ram = 16384`** (the default is 8192): essential when sequential. When the orchestrator
  goes back from agent B to agent A, A's context is still in RAM and its prefill is not redone.
- **Hybrid models and the cache**: llama.cpp resumes the state from checkpoints
  (`--ctx-checkpoints`, 32 per slot by default). When a client edits the beginning of the history
  (OpenCode compaction, an edit in OpenWebUI), it restarts from the nearest checkpoint.
- **One model ID at a time** for every client. Asking for the other model triggers a reload of a
  few tens of seconds.
- **Idle unload**: the gateway's `ttl` stops an idle backend after 30 minutes; the next request
  starts it again. llama-server's own `--sleep-idle-seconds` is not used: it keeps the process and
  would reload the model on the next request, whatever else holds the VRAM by then.

---

## 7. Client Configuration

How llama-swap and llama-server handle an API key was checked on 2026-10-16, with llama-swap
v263 and llama-server `b11538` in front of a fake upstream that logged each request's headers:

- llama-swap with `apiKeys` takes a key as `Authorization: Bearer <key>` or as
  `x-api-key: <key>`. It answers 401 without one on every route but `/health`,
  `/upstream/<model>/…` included.
- It forwards the client's header to the backend unchanged, so a backend started with the same
  key accepts every request that passed the gateway. Its own requests carry no key: `/health`,
  which is public in llama-server, and `/v1/models`, whose 401 it ignores.
- llama-server checks the same two headers. Only `/health`, `/v1/health` and its web UI's files
  are public (`tools/server/server-http.cpp`); ai-ctl builds it without that UI
  ([security.md §7](security.md#build-sources)). It reads the key from `--api-key`, from
  `LLAMA_API_KEY` or from `--api-key-file`; a key given on the command line shows in `ps` to
  every user.
- Without keys, any web page can call both. llama-swap answers every origin with
  `Access-Control-Allow-Origin: *`, llama-server's CORS allows every origin by default, and the
  backends listen on predictable ports from 5800.

**ai-ctl's key.** The first `ai-ctl up` that starts the gateway creates one key in
`~/.local/state/ai-ctl/api-key` (under `$XDG_STATE_HOME` when that is set), readable by the user
only, and keeps it. `up` passes it to llama-swap in the variable `AI_CTL_API_KEY`. The
configuration reads it from there, `apiKeys: ["${env.AI_CTL_API_KEY}"]`, and passes it to each
backend as `LLAMA_API_KEY` (§6.4), so neither the configuration nor a command line holds the key,
and llama-swap refuses to start without it. Every client sends it as `Authorization: Bearer
<key>`; `/health` needs none. To change it, remove the file, then run `ai-ctl down` and `ai-ctl
up`.

CORS is left as llama-swap and llama-server set it. llama-swap's own guide says it plainly:
`allowedOrigins` is not access control. A request from an origin it does not list still runs; the
browser only hides the answer from the page. The key is what refuses a page's request.

**Never type the key into the login prompt a browser shows for llama-swap's web UI**, in a browser
used for browsing. llama-swap also takes the key as HTTP Basic credentials, and asks for them with
`WWW-Authenticate: Basic` on every 401. A browser that keeps them can attach them to a request a
page sends to the gateway, such as a form posted to `/api/models/unload`, which llama-swap runs
whatever the page's origin (checked on 2026-10-16). ai-ctl and the clients below never need that
prompt.

[security.md](security.md) says what follows from these facts: who can reach the gateway and the
backends, what the key guards and what it does not, and the clients that hold it.

### 7.1 Orchestrator

- Base URL `http://127.0.0.1:8080/v1`, model `qwen3.8-27b`, the API key read from ai-ctl's file.
- If the orchestrator speaks the Anthropic protocol, `/v1/messages` is available; the Responses
  API is at `/v1/responses`. Both stream through llama-swap unbuffered, tool calls included
  (checked on 2026-10-12). In a Messages stream, llama-server opens the text block before it
  closes the thinking block, which Anthropic's API never does: a strict client may refuse it.
- **Per request**:
  - `reasoning_effort`: `none` turns thinking off; `low`, `medium`, `high` go to the template. Use
    `none` or `low` for simple tasks.
  - `response_format` with a `json_schema` for structured, guaranteed-valid output.
  - `reasoning_control: true` lets a client cut an overlong reasoning short, through
    `POST /v1/chat/completions/control` with the action `reasoning_end`. llama-swap does not route
    that endpoint: call it at `/upstream/<model>/v1/chat/completions/control`.
  - `chat_template_kwargs` for any other template parameter.
- **Maximise cache hits**: keep one stable system prompt per kind of sub-agent and put the variable
  content at the end. Each response's `timings.cache_n` tells how many tokens were reused.

### 7.2 OpenCode

[`deploy/opencode.json`](../deploy/opencode.json) goes to `~/.config/opencode/opencode.json`:

- the `@ai-sdk/openai-compatible` provider pointing at `http://127.0.0.1:8080/v1`, the gateway,
  with `"apiKey": "{file:~/.local/state/ai-ctl/api-key}"`: OpenCode reads ai-ctl's key from its
  file, `~` expanded and the final newline dropped (checked with OpenCode 1.19.0 on 2026-10-16);
- the inventory's model IDs: llama-swap routes a request on its `model` field, and llama-server
  answers to the same name (`--alias`);
- limits of 73,728 context + 24,576 output = 96K for the 27B, and 106,496 + 24,576 = 128K for the
  35B, to stay within each model's `ctx_size`;
- `"reasoning": true` and `"interleaved": {"field": "reasoning_content"}` to send the reasoning back
  in the history;
- **`small_model` pointing at the local model.** Otherwise OpenCode uses a remote model by default
  (gpt-5-nano through Zen) to title sessions, and data leaves the machine;
- `"share": "disabled"`.

OpenCode's install script (`https://opencode.ai/install`) downloads the latest release from
`github.com/anomalyco/opencode` without checking it. Download the release's
`opencode-linux-x64.tar.gz` instead, check its SHA-256 against the digest GitHub publishes for the
asset, then install the binary with the script's `--binary <path> --no-modify-path` into
`~/.opencode/bin`.

Checked with OpenCode 1.19.0 on 2026-10-12, through the gateway on the 27B: `grep`, `read`,
`edit` with multi-line code, and `bash` calls, a continued session, and the session title from
`small_model`, with no template error. The turn after a tool call reuses the whole context, the
reasoning sent back included.

### 7.3 OpenWebUI

OpenWebUI, in a podman container, reaches the gateway at `http://10.0.2.2:8080/v1`.
[openwebui.md](openwebui.md) says how to run it and set its chat, speech and images: the
connection, the empty task model and autocomplete turned off among them.

---

## 8. Machine Setup (Fedora 44)

1. **NVIDIA driver** from RPM Fusion (`akmod-nvidia`, `xorg-x11-drv-nvidia-cuda`). With Secure
   Boot, enroll the MOK key.
2. **CUDA toolkit**, for native builds. **Avoid CUDA 13.2**: it is reported to produce gibberish
   with some quantizations when llama.cpp is compiled from source (llama.cpp issue #21255, user
   reports of a fix in 13.3); the evidence is thin but the cost of avoiding it is nil. The driver
   above supports up to 13.4, installed here since 2026-10-09; `nvcc` 13.4 accepts Fedora 44's
   GCC 16. The procedure — NVIDIA's repository with its driver packages excluded, a versioned
   toolkit package, version and release changes — is in [toolchain.md](toolchain.md).
3. **Container tooling**, only for the official-image runtime: the NVIDIA Container Toolkit, whose
   `nvidia-cdi-refresh` units (read in toolkit 1.20.1) write the CDI specification to
   `/var/run/cdi/nvidia.yaml` at boot and at each driver change:
   `sudo systemctl enable --now nvidia-cdi-refresh.path nvidia-cdi-refresh.service`. The path
   unit watches `modules.dep`, `modules.dep.bin` and `/usr/bin/nvidia-ctk` only, so an upgrade of
   another library the specification mounts leaves it stale: on 2026-10-18, dnf replaced
   egl-wayland 1.1.23 by 1.1.24 after the boot, and podman refused every container with the GPU,
   "cannot stat `/usr/lib64/libnvidia-egl-wayland.so.1.1.23`", until
   `sudo systemctl start nvidia-cdi-refresh.service` wrote it again (finding 22 of
   [the closing review of v2-review-fixes](reviews/delivered-code/2026-10-17-v2-review-fixes.md)). A
   specification generated by hand into `/etc/cdi/nvidia.yaml` stays on the driver it was made
   for; podman prefers `/var/run/cdi` for a device both define, and uses the one in `/etc/cdi`
   whenever `/var/run/cdi` is empty. On this machine one written in January named driver 580.119.02 on
   2026-10-12, under 615.71.09; `ai-ctl doctor` checks that the specification in use mounts the
   loaded driver's libraries and only paths on disk, and that no other names an older one.
4. **Models**, with `hf download` (about 37 GB), one directory per model under `$MODELS_DIR`,
   grouped by use (`llm/`, later `tts/`, `stt/`, `image/`); on this machine `MODELS_DIR` is
   `/ai/models` and `HF_HOME` is `/ai/cache/huggingface`. Each download is checked before its
   first use, as "Where Models Come From" (§5) says. The revisions are the commits downloaded and
   checked on 2026-10-10; a newer one is checked the same way before it replaces them:

   ```bash
   hf download unsloth/Qwen3.8-27B-GGUF \
     Qwen3.8-27B-UD-Q4_K_M.gguf mmproj-F16.gguf MTP/mtp-Qwen3.8-27B-Q4_0.gguf \
     --revision 4ca720788d1e01f1bff70c033e0d0028fd02e502 \
     --local-dir "$MODELS_DIR/llm/qwen3.8-27b"
   hf download unsloth/Qwen3.6-35B-A3B-GGUF \
     Qwen3.6-35B-A3B-UD-IQ4_XS.gguf mmproj-F16.gguf \
     --revision a483e9e6cbd595906af30beda3187c2663a1118c \
     --local-dir "$MODELS_DIR/llm/qwen3.6-35b-a3b"
   ```

5. **Inventory and check**: `ai-ctl init --models-dir "$MODELS_DIR"` installs the inventory, then
   `ai-ctl doctor` checks every point of this section and prints a remedy for each problem.
6. **Native build**: `ai-ctl build llama.cpp` compiles the newest release for `120a-real` and makes
   it the current build; `ai-ctl build use` returns to an older one (see the README).
7. **Gateway**: `ai-ctl gateway install` installs the pinned llama-swap release; `ai-ctl up` starts
   it on `127.0.0.1:8080`, `ai-ctl status` shows it, its models and the VRAM, `ai-ctl logs` prints
   its log, and `ai-ctl down` stops it once the models' VRAM is free (see the README). Nothing runs
   outside `up` and `down`.

### Laptop Hygiene

- On mains power, profile `performance`. Fedora 44 ships `tuned-ppd`, which serves the power
  profiles interface without `powerprofilesctl`: choose the profile in the system menu, or set it
  with the `busctl` command `ai-ctl doctor` prints.
- Display on the iGPU (hybrid mode), to keep the whole VRAM.
- Monitoring: `nvidia-smi`, `curl -s localhost:8080/models`.

---

## 9. Measurement Protocol

Compare **engine + model + quantization** pairs, never engines alone: a Q4_K_M GGUF and an NVFP4
checkpoint are neither the same weights nor the same quality. The same goes for runtimes: compare a
native build and an image on the same model and flags.

| Measure | How | Why |
|---|---|---|
| Generation, 1 request | tok/s at 8K, 32K and 64K of context and at the context's limit | The real sequential case |
| Prefill | tok/s on a fresh 40K prompt | The cost of a new agent |
| Cache effect | TTFT at turn 1 against turns 5–10 of one session | Checks the reuse (`timings.cache_n`) |
| MTP | Throughput with `spec-draft-n-max` from 1 to 4, acceptance rate | Find the best setting |
| VRAM | Peak from NVML + `--fit` logs | Nothing may be moved to the CPU |
| Stability | 1 h of continuous agentic session | Laptop thermals, leaks, crashes |
| Quality | Success rate on 10–20 real tasks, % of valid tool calls and JSON | The only criterion that counts in the end |

`/metrics` needs llama-server to run with `--metrics`, which ai-ctl adds to every entry. Through the
gateway, it is reached at `/upstream/<model>/metrics`, like `/tokenize` and `/slots`.

`ai-ctl bench run` measures the first five through the gateway, the MTP setting being an edit of
the inventory and one run per value; the `--fit` logs are read in the gateway's log. A request that
must miss the prompt cache sends `cache_prompt: false`, which makes llama-server evaluate the prompt
whole (`n_past = 0` in `tools/server/server-context.cpp`), so that two runs can send the same
prompts. Such a request comes after an unmeasured one, the same prompt cut to a quarter. With
`--cache-ram`, a slot given a prompt that keeps less than half of its context first saves that
context into the RAM cache, unless it is there already, and one given a prompt that shares no more
than 10 % with its own (`--slot-prompt-similarity`) also saves it and may load a cached context in
its place (`get_available_slot` in the same file). Left in the measure, the save cost 0.9 to 2.6 s
to the first repetition of a scenario on the 27B and nothing to the next ones. The quarter pays it,
and leaves in the slot a prompt the fresh one shares a quarter with and keeps whole, so that the
fresh request neither saves nor loads a context. Stability and quality are outside the bench.

The GPU's readings keep NVML's clock-event reasons whole. On this machine (driver 615.78.08,
run `20261016T194820Z-qwen3.8-27b`, 1,742 readings), every reading holds one reason: SwPowerCap
(0x4) or Reliability (0x400), never both, and never BoardLimit (0x200), heat or the hardware.
Reliability is what NVML reports when the power limit does not hold the clocks: at idle (4.5 W,
no gateway), between requests (below 80 W), and for about half a second at a time under load,
at 150 to 171 W where the capped readings reach 173 W, the SM clock then at 0.98 of the capped
readings around it (0.83 to 1.07). So "slowed by" leaves it out, and the "policy limit" column
gives its share apart: 3 to 8 % of the depth, prefill and session scenarios, 35 % of the
overhead scenario, whose short requests leave the GPU waiting.

---

## 10. Upstream and ai-ctl's Place

### 10.1 What llama-server Now Does

| llm-ctl v1 feature | Native equivalent (October 2026) |
|---|---|
| `model list/info/switch` | INI presets, `GET /models`, `POST /models/load` and `/unload`, automatic switch with `--models-max 1` |
| Readiness | `/health` (503 while loading), model states in `GET /models` |
| `--lora` | `--lora`, `--lora-scaled`, `lora` per request on `/completion` |
| `--draft` | `--spec-type` (`draft-mtp`, `draft-simple`, `draft-eagle3`, `ngram-*`…), `--spec-draft-model` |
| `--warm` (prefix warming) | Automatic prompt cache in RAM; `--slot-save-path` with `POST /slots/{id}?action=save\|restore` |
| Fitting into VRAM | `--fit` (on by default), `--fit-target` |
| Freeing VRAM when idle | `--sleep-idle-seconds` |
| OpenAI gateway | Chat completions, completions, Responses API, Anthropic Messages, routing by `model`. No Ollama API. |
| Model registry and downloads | `-hf`, local cache, downloads through the router |

### 10.2 Neighbouring Tools

- **llama-swap**: a Go proxy, one binary and one YAML file, that starts and stops any
  OpenAI-compatible server (llama.cpp, vLLM, a container) according to the `model` field, with idle
  TTLs, a `matrix` of models allowed to run together, health checks, unload endpoints, `/running`,
  `/upstream/<model>` passthrough, a web UI and logs. It also fronts whisper.cpp, audio.cpp,
  stable-diffusion.cpp and ComfyUI. **Chosen as the gateway** ([gpu/llama-swap.md](gpu/llama-swap.md)); it does no VRAM
  accounting, which ai-ctl supplies.
- **llama-benchy**: llama-bench-style measurements against any OpenAI-compatible endpoint — prompt
  and generation speeds by context depth, TTFT, prefix caching.
- **GuideLLM** (vLLM project): load profiles and TTFT/ITL distributions against any
  OpenAI-compatible server; no multi-turn mode found.

### 10.3 What ai-ctl Keeps

| Piece | Why upstream does not cover it |
|---|---|
| `doctor` | Driver, toolkit and compiler versions, CDI, power profile, display on the iGPU, model files: outside llama.cpp's scope |
| Native builds | Exact commit, `120a` kernels, versioned builds with rollback, the CUDA 13.2 guard |
| Gateway | llama-swap's configuration generated from the inventory, with a `matrix` computed from VRAM budgets; the gateway run between `up` and `down`, `down` returning once the VRAM is free |
| Bench | Multi-turn agent sessions measured through `timings.cache_n`, MTP acceptance from `/metrics`, and the runtime of each run — which neither llama-benchy nor GuideLLM records |

### 10.4 Corrections to the March 2026 Analysis

| March 2026 claim | October 2026 |
|---|---|
| llama.cpp: "basic per-session" KV, no sharing between requests | Wrong: prompt cache in RAM, slots, unified KV pool |
| llama-server: sequential processing, flat throughput whatever the concurrency | Wrong: continuous batching on by default |
| vLLM excluded because SGLang is faster on one GPU | On consumer Blackwell with hybrid Qwen models, vLLM is the most credible second engine (NVFP4 W4A4, MTP, Unsloth recipes); SGLang the least |
| RadixAttention computes a system prompt sent 20 times only once | vLLM (prefix caching) and llama.cpp (prompt cache) do it too; sequentially, the advantage disappears |
| Prebuilt llama.cpp binaries target sm_89, so compile for `120` | The official images ship `120a-real` kernels; a native build must target `120a` to match them |

---

## 11. Checklist

- [ ] Display on the iGPU, laptop on mains power, profile `performance`
- [ ] CUDA toolkit other than 13.2 for native builds
- [ ] For the image runtime: `nvidia-ctk cdi list` contains `nvidia.com/gpu=all`
- [ ] Logs: model entirely on the GPU, nothing moved by `--fit`
- [ ] Logs: `draft-mtp` loaded, acceptance rate sound in `/metrics`
- [ ] Same model ID in the orchestrator and OpenCode (`model` **and** `small_model`); OpenWebUI's task model left empty
- [ ] OpenWebUI autocomplete off
- [ ] `timings.cache_n` high from an agent's second turn on

---

## Sources

- [OCTO Talks! — Llama.cpp, SGLang, vLLM: quel framework pour votre assistant de code ?](https://blog.octo.com/llama.cpp-sglang-vllm--quel-framework-d%27inference-llm-choisir-pour-votre-assistant-de-code)
- [llama.cpp — llama-server README (options, router mode, API)](https://github.com/ggml-org/llama.cpp/blob/master/tools/server/README.md)
- [llama.cpp — CUDA Dockerfile](https://github.com/ggml-org/llama.cpp/blob/master/.devops/cuda.Dockerfile), [image workflow](https://github.com/ggml-org/llama.cpp/blob/master/.github/workflows/docker.yml), [CUDA architectures](https://github.com/ggml-org/llama.cpp/blob/master/ggml/src/ggml-cuda/CMakeLists.txt)
- [llama.cpp — Docker images](https://github.com/ggml-org/llama.cpp/blob/master/docs/docker.md)
- [llama.cpp PR #23269 — MTP and parallel decoding](https://github.com/ggml-org/llama.cpp/pull/23269)
- [llama.cpp issue #21255 — CUDA 13.2 and IQ3_S](https://github.com/ggml-org/llama.cpp/issues/21255)
- [llama.cpp discussion #27164 — Qwen3.8 corrupted output on older CUDA builds](https://github.com/ggml-org/llama.cpp/discussions/27164)
- [llama.cpp issue #22364 — presets and `load-on-startup`](https://github.com/ggml-org/llama.cpp/issues/22364)
- [Llama-server router mode (glukhov.org)](https://glukhov.org/llm-hosting/llama-cpp/llama-server-router-mode/)
- [Unsloth — Qwen3.8-27B GGUF](https://huggingface.co/unsloth/Qwen3.8-27B-GGUF)
- [Unsloth — Qwen3.6-35B-A3B GGUF](https://huggingface.co/unsloth/Qwen3.6-35B-A3B-GGUF/tree/main)
- [Unsloth — Qwen3.6 guide (MTP, NVFP4, sampling)](https://unsloth.ai/docs/models/qwen3.6)
- [Unsloth — Qwen3.8 guide](https://unsloth.ai/docs/models/qwen3.8)
- [Unsloth — MTP guide](https://unsloth.ai/docs/models/mtp)
- [Qwen — Qwen3.8-27B chat template](https://huggingface.co/Qwen/Qwen3.8-27B/blob/main/chat_template.jinja), [Qwen3.6-35B-A3B chat template](https://huggingface.co/Qwen/Qwen3.6-35B-A3B/blob/main/chat_template.jinja)
- [llama.cpp — `gguf-dump`](https://github.com/ggml-org/llama.cpp/blob/master/gguf-py/gguf/scripts/gguf_dump.py)
- [Pillar Security — LLM backdoors at the inference level: poisoned templates](https://www.pillar.security/blog/llm-backdoors-at-the-inference-level-the-threat-of-poisoned-templates), [chat template backdoors across 18 models and 4 engines](https://www.pillar.security/blog/from-discovery-to-large-scale-validation-chat-template-backdoors-across-18-models-and-4-engines), [arXiv 2602.04653](https://arxiv.org/pdf/2602.04653v3)
- [Atomic Chat — Qwen 3.8 27B locally (architecture, benchmarks)](https://atomic.chat/blog/guides/how-to-run-qwen-3-8-locally)
- [OrcaRouter — Qwen3.8-27B with vLLM](https://www.orcarouter.ai/de/blog/qwen-3-8-27b-vllm)
- [NVIDIA — Qwen3.6-27B-NVFP4](https://huggingface.co/nvidia/Qwen3.6-27B-NVFP4)
- [vLLM issue #40696 — prefix caching of hybrid models](https://github.com/vllm-project/vllm/issues/40696)
- [vLLM PR #46384 — prefix matching for hybrid models](https://github.com/vllm-project/vllm/pull/46384)
- [LMCache — Qwen3.5 / 3.6 / 3.8 recipe](https://docs.lmcache.ai/recipes/qwen3_5.html)
- [packet.ai — vLLM prefix caching on by default](https://packet.ai/blog/vllm-prefix-caching)
- [llama-swap](https://github.com/mostlygeek/llama-swap), [`matrix` (#643)](https://github.com/mostlygeek/llama-swap/issues/643), [quickstart](https://www.glukhov.org/llm-hosting/llama-swap/), [configuration reference](https://github.com/mostlygeek/llama-swap/blob/v263/docs/config.example.yaml)
- [llama-swap issue #1205 — 502 `proxy error: EOF` after a streamed response](https://github.com/mostlygeek/llama-swap/issues/1205), [PR #1211 — `disableKeepAlives`](https://github.com/mostlygeek/llama-swap/pull/1211)
- [llama.cpp issue #29680 — keep-alive advertised, connection closed after a stream](https://github.com/ggml-org/llama.cpp/issues/29680)
- [llama-benchy (NVIDIA forums)](https://forums.developer.nvidia.com/t/new-tool-llama-benchy-llama-bench-style-benchmarking-for-any-llm-backend-vllm-sglang-llama-cpp-etc/356698)
- [GuideLLM](https://github.com/vllm-project/guidellm)
- [OpenCode — provider configuration](https://opencode.ai/docs/providers/)
