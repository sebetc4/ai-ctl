# Image Generation: stable-diffusion.cpp

stable-diffusion.cpp's `sd-server` generates images on the GPU from a diffusion model, a VAE and
one or more text encoders. It answers OpenAI's image endpoints and the stable-diffusion-webui API.
ai-ctl builds it, `ai-ctl build stable-diffusion.cpp`, and the gateway serves the inventory's
`sd-server` models with it. This document says how it builds, runs, gets ready and answers, what
guards it, and which models it serves. Every fact below was probed on this machine
on 2026-10-20 unless dated otherwise, at release `master-975-95b3090` (commit `95b30906`,
2026-10-18), with nvcc 13.4.92, g++ 16.2.1 and llama-swap v263.

---

## 1. Build

- **Repository and releases.** `https://github.com/leejet/stable-diffusion.cpp.git`, a personal
  account, MIT, 747 tags. 539 are `master-<N>-<hash>` releases, published several times a day.
  The rest are 206 older `master-<hash>` tags and two `workflow-` tags. A pattern of
  `master-<N>-<hash>`, ordered by `<N>`, picks the releases.
- **Submodules.** There are four:
  - `ggml` comes from `https://github.com/leejet/ggml.git`, a fork, on its branch `sd.cpp`
    (`dc998771`, 2026-10-17, ggml 0.27.0);
  - `thirdparty/libwebp` and `thirdparty/libwebm`;
  - `examples/server/frontend` comes from `leejet/sdcpp-webui`.

  Only `ggml` is needed: `git submodule update --init --filter=blob:none -- ggml`.
- **CUTLASS.** Since `master-1024-77b72a2` (2026-10-28), the fork has a submodule of its own,
  `src/ggml-cuda/vendors/cutlass`: NVIDIA's CUTLASS, `https://github.com/NVIDIA/cutlass.git`,
  BSD-3-Clause, at `v3.9.2` (`ad7b2f5e`), 143 MB of headers checked out. ggml's
  `GGML_CUDA_CUTLASS_PATH` defaults to that folder, and CMake stops when the headers are not
  there: "CUTLASS headers not found at …". With them, ggml builds an INT8 GEMM (`mul-mat-i8.cu`)
  and a CuTe FlashAttention (`fattn-cute.cu`), which `ggml_cuda_flash_attn_ext` prefers when it
  supports the case; an empty `GGML_CUDA_CUTLASS_PATH` builds without both. Git takes from a
  repository only its own submodules, so CUTLASS is checked out from within `ggml`:
  `git -C ggml submodule update --init --filter=blob:none -- src/ggml-cuda/vendors/cutlass`.
  On 2026-11-01 (#2253) the project took CUTLASS over: `thirdparty/cutlass`, a submodule of its
  own from the same repository at the same `v3.9.2`, its path in `SD_CUDA_CUTLASS_PATH`; the fork
  no longer holds it, and CMake stops the same way without it (`master-1071-cfdf3cf`, built on
  2026-11-09). The recipe names both, and a release holds one or the other.
  `ai-ctl build stable-diffusion.cpp` built that release in 307 steps and 2 min 47 s, 150 CUDA
  objects with those two among them, into a 94.9 MB `sd-server`. FLUX.3 Image Klein made a
  1024×1024 image in 9.9 s, 16 steps at CFG 1.0. Its attention, head size 128 over 4096 tokens
  and more, takes the CuTe kernel: its sampling took 9.2 s, and 10.0 s with
  `GGML_CUDA_DISABLE_CUTE_FATTN=1`, two images each. sd.cpp's own
  `flash attention auto: CUDA0 -> on (CUDA MMA)` line says only that attention is on.
- **Options.** The builder's options for llama.cpp, plus:
  - `SD_CUDA=ON`, which sets `GGML_CUDA`;
  - `SD_BUILD_EXAMPLES=ON`;
  - `SD_SERVER_BUILD_FRONTEND=OFF`. It defaults on, and with `pnpm` on the `PATH`, as it is on
    this machine, the build would run `pnpm install` in the frontend: packages fetched from npm
    and run at build time;
  - `SD_WEBP=OFF` and `SD_WEBM=OFF`, which default on only when their submodule is there.
- **The build.** `cmake --build … --target sd-server --parallel 24`: 266 steps, 147 CUDA objects,
  251 s.
  - Warnings: nvcc's `#174-D` in `ggml/src/ggml-cuda/sageattention/attn_utils.cuh`,
    `-Wstringop-overflow` in `thirdparty/stb_image.h`, and `wstring_convert` deprecated in
    `src/core/util.cpp`.
  - The binary is `bin/sd-server`, 92.0 MB. It links the same CUDA 13.4 libraries as
    whisper-server. Its 147 cubins are all `sm_120a`, with no PTX.
- **Version and devices.**
  - `--version` (exit 0) prints `stable-diffusion.cpp version master-975-95b3090, commit 95b3090`.
  - `--list-devices` (exit 0) prints ggml's CUDA banner on stderr, then one
    `name<TAB>description` line per device on stdout:
    `CUDA0<TAB>NVIDIA GeForce RTX 5090 Laptop GPU` and `CPU`.
  - Both are captured with `--help` in `tests/fixtures/`, as listed in its README.

---

## 2. Serving

- **Options.**
  - `-l/--listen-ip` (default `127.0.0.1`), `--listen-port` (1234) and `--serve-html-path`.
  - The model's files: `-m`, `--diffusion-model`, `--vae`, `--llm` (an LLM text encoder),
    `--clip_l`, `--clip_g` and `--t5xxl`.
  - The defaults of a generation: `--steps` (20), `--cfg-scale` (7.0) and `--sampling-method`.
  - `--lora-model-dir`, and `--rpc-servers`, which sends the work to remote ggml RPC servers.
  - Memory: `--offload-to-cpu`, `--max-vram` and `--eager-load`.
- **Placement, `--auto-fit`, on by default.** Each component is placed by the free VRAM read at
  the start ("CUDA0 free 23798 MiB, budget 23286 MiB"): on the GPU first, then in RAM, then on
  disk. Here everything went to `CUDA0`. On a GPU that already holds a backend, the log and
  `--help` say it would place weights in RAM instead of failing; not probed.
- **Lazy weights.** Without `--eager-load`, weights load on first use. "Ready" then holds no
  weight: the first request carries the load, and the VRAM llama-swap sees after the start is not
  the model's.
- **Routes** (`examples/server/main.cpp`, `examples/server/routes_openai.cpp`):
  - `/`;
  - `GET /v1/models` answers one fixed model, `sd-cpp-local`;
  - `/v1/images/generations` and `/v1/images/edits`;
  - `/sdapi/v1/*`, the stable-diffusion-webui API;
  - `/sdcpp/v1/*`, for jobs, video and upscaling.

  There is no `/health`. `master-1017-be48649` (2026-10-28) adds model routes:
  `/sdcpp/v1/model-files`, `/sdcpp/v1/models/load`, `/unload`, `/refresh`, `/status`, and
  `/sdcpp/v1/recipes`. The key guards them (§3). Behind the gateway, probed on 2026-10-29 at
  `master-1024-77b72a2`:
  - A client reaches them only by `/upstream/<model>/…`: llama-swap answers 404 to
    `/sdcpp/v1/models/load` with the model in the body.
  - A load takes its files from the server's catalog, the files under `--model-dir`
    (`examples/server/model_catalog.cpp`), by default `models` in the current directory. The
    server runs in llama-swap's, the gateway's directory, which holds no `models`: the
    catalog is empty, and each of the 108 builtin recipes was refused, "missing required
    model", the model in use still answering. ai-ctl reserves `model-dir`, which would let a
    client load another model in place of the inventory's, outside llama-swap's arbitration
    of the VRAM.
  - An unload empties the server: its VRAM is freed, `/health` still answers 200 and llama-swap
    keeps the model running, so that each image request answers 503 `model_not_loaded` until
    llama-swap stops it, at its `ttl`, a swap or `down`.
- **The OpenAI route.** `POST /v1/images/generations` reads `prompt`, `n`, `size` (`WxH`),
  `output_format` (`png` by default) and `output_compression`. It ignores `model` and
  `response_format`, and always answers `data[].b64_json`. Steps, CFG and the sampler come from
  the server's command line.
- **The sdapi route.** `POST /sdapi/v1/txt2img` takes `width`, `height` and `steps` from the
  request (512², 8 steps: 3.2 s), and answers `images`, `info` and `parameters`.
- **Arguments inside the prompt.** A prompt may embed
  `<sd_cpp_extra_args>{…}</sd_cpp_extra_args>`, JSON read by
  `SDGenerationParams::from_json_str` (`examples/common/common.cpp`). It sets `width`, `height`
  and `batch_count` with no bound, and `ad_model`, a model path on the host.
- **Readiness.** The context is built before `listen`, so nothing answers while it loads. Then
  `GET /v1/models` answers 200 `{"data":[{"id":"sd-cpp-local",…}]}`. SIGTERM ends it with 143:
  it installs no handler.
- **A port it cannot bind ends it with 0, without a word** (2026-10-21). `main.cpp` logs
  "listening on" before `svr.listen`, and ignores its failure: it stops its worker and returns 0
  (`examples/server/main.cpp:157–166`). Started on a port another process holds, it exits 0 with
  that log, where `whisper-server` and `audiocpp_server` exit 1 with a message.
  - The vendored cpp-httplib sets `SO_REUSEPORT` in place of `SO_REUSEADDR` on Linux, as
    whisper.cpp's does, so the server cannot bind a port where another process's
    connection is in TIME-WAIT, for up to 60 s. Reproduced with `python3 -m http.server` on the
    port, stopped after one request: `sd-server` exited 0, and started again once the TIME-WAIT
    expired, it was ready. llama-server replaces its cpp-httplib's options with `SO_REUSEADDR`,
    `SO_REUSEPORT` only under `--reuse-port`.
  - A server binds a port a server of its own kind still listens on (2026-10-28,
    `master-1017-be48649`): both listen, and the kernel shares the connections between them.
  - Decided: ai-ctl binds it with `SO_REUSEADDR` alone, as llama-server, by a patch of its own,
    `reuse-address.patch` (§3; the user, 2026-10-28). Probed on the server ai-ctl built: a
    second `sd-server` on the first's port exits 1, `[E] cannot listen`, and a port in
    TIME-WAIT after `python3 -m http.server` no longer stops it.
  - Decided: ai-ctl ends a failed `listen` with a message and exit 1, by a patch of its own beside
    the key's, `listen-failure.patch` (§3). Probed on 2026-10-28 on a server rebuilt with both,
    FLUX.3 Image Klein loaded: on a port another process held, it exited 1 once its model was
    loaded, with `[E] cannot listen on 127.0.0.1:18977`.
- **Behind llama-swap.** Each model runs as
  `sd-server <files> <defaults> --listen-ip 127.0.0.1 --listen-port ${PORT} --eager-load`, with a
  `checkEndpoint` on a route that needs no model loaded: `/v1/models` unpatched, `/health` with the
  patch (§3).
  - `POST /v1/images/generations` with `model: qwen-image-2.1` answered 200 in 41.1 s, the load
    included, then 36.7 s.
  - Then `model: flux-3-image-klein` answered in 16.9 s: llama-swap stopped Qwen-Image, loaded
    FLUX and made one image, and `GET /running` listed FLUX alone.
  - `POST /sdapi/v1/txt2img` with `model` in the body is routed.
  - `GET /sdapi/v1/sd-models` gets llama-swap's 404, with or without `?model=`, and
    `/upstream/<model>/…` reaches any path.

---

## 3. The Key

- **No key anywhere in the sources**, at `master-975-95b3090` nor at the head of the main branch
  fetched on 2026-10-20 (`0518b34`). A pre-routing handler echoes the request's `Origin` with
  `Access-Control-Allow-Credentials: true`. On its port, a request without a key and with
  `Origin: https://attacker.example` gets 200,
  `Access-Control-Allow-Origin: https://attacker.example` and the credentials header. Any page can
  thus generate images on the GPU and read them.
- **Decided: a patch of ai-ctl's own** (user's decision, 2026-10-20), the same decision as for
  whisper.cpp ([whisper-cpp.md §3](../audio/whisper-cpp.md#3-the-key)), with the same options set
  aside. ai-ctl's recipe applies it to every build, `src/domain/patches/stable-diffusion.cpp/`
  `api-key.patch`, then `listen-failure.patch` and `reuse-address.patch` (§2).
- **The prototype**, against `master-975-95b3090` (`examples/server/main.cpp`, +20 −14):
  - the check of whisper.cpp's patch takes the place of the pre-routing handler that echoed the
    `Origin`;
  - a new `GET /health` answers `{"status":"ok"}` and is exempt, so that `/v1/models` stays
    guarded.

  Checked on the server's port with a throwaway key:
  - without the key, `/health` answers 200;
  - without the key or with a wrong one, a `GET`, the generation `POST`, an `OPTIONS` preflight
    and a `text/plain` `POST` from `Origin: https://attacker.example` all get 401;
  - with `Bearer` or `X-Api-Key`, an image comes back;
  - no `Access-Control-*` header reaches any origin.

  Behind llama-swap, with `env: ["LLAMA_API_KEY=${env.AI_CTL_API_KEY}"]` and
  `checkEndpoint: /health`, an sdapi image and `/upstream/flux-3-image-klein/v1/models` answer
  200 with the key. On FLUX's backend port, without the key, `/v1/models` gets 401 and `/health`
  200.
- **Rebased on `master-1017-be48649`** (2026-10-28), which logs its parameters at debug level and
  registers the model routes before `listen`: the same added and removed lines. Probed on the
  server ai-ctl built, FLUX.3 Image Klein loaded: the same results as the prototype's, and the
  six model routes get 401 without the key.

---

## 4. Models

The rule of [stack.md §5](../stack.md#where-models-come-from) applies: the publisher's files when
stable-diffusion.cpp reads their format, failing that the project's own conversions (`leejet/…`),
and an LLM text encoder as a GGUF from its publisher. Each model is several files in one directory,
its text encoder and VAE included.

- **Qwen-Image 2.1** (`image/qwen-image-2.1`, licence `qwen-research`).
  - The diffusion model comes from leejet's GGUF: `leejet/Qwen-Image-2.1-GGUF` `cc114339`,
    `qwen_image_2.1-Q8_0.gguf` (7.69 GB), the highest precision it offers. Qwen publishes the model
    as two diffusers shards, which `--diffusion-model` cannot take as one file.
  - The VAE is Qwen's own: `Qwen/Qwen-Image-2.1` `d26bb612`, `vae/config.json` and
    `vae/diffusion_pytorch_model.safetensors` (1.35 GB). It loads (the log: "Wan VAE", 238 f32
    tensors) through `convert_diffusers_to_original_wan_vae`, which sd.cpp uses for
    `VERSION_QWEN_IMAGE_2_1` (`src/name_conversion.cpp:1181`, `:1280`). sd.cpp's
    `docs/qwen_image_2.1.md` asks for Comfy-Org's `qwen_image_2.1_vae_bf16.safetensors`, which is
    not needed.
  - The text encoder comes from `Qwen/Qwen3-VL-8B-Instruct-GGUF` `f982a075`,
    `Qwen3VL-8B-Instruct-Q8_0.gguf` (8.71 GB, Apache-2.0). Its vision projector,
    `mmproj-Qwen3VL-8B-Instruct-F16.gguf` (1.16 GB), passed with `--llm_vision`, reads the
    reference images of an edit: sd.cpp's `docs/qwen_image_2.1.md` requires it with a GGUF text
    encoder.
  - Settings, from `docs/qwen_image_2.1.md`: `--cfg-scale 6.0 --sampling-method euler`, with the
    default 20 steps.
- **FLUX.3 Image Klein** (`image/flux-3-image-klein`, licence `flux-kommunity-license`).
  - `black-forest-labs/flux-3-image-klein` `33733762` provides `model.safetensors` (11.10 GB,
    359 bf16 tensors) and `ae.safetensors` (0.34 GB). It also holds diffusers folders and `fp8r`
    and `nvfp4` variants.
  - The repository is gated: the user accepts the licence on its Hugging Face page and downloads
    with a read token (`hf auth login`).
  - sd.cpp's `docs/flux3.md` names FLUX.2-dev's VAE, and adds that "the `ae.safetensors` included
    with FLUX.3 can also be used"; the image is clean with it.
  - The text encoder comes from `Qwen/Qwen3-VL-4B-Instruct-GGUF` `1cd86afb`,
    `Qwen3VL-4B-Instruct-Q8_0.gguf` (4.28 GB, Apache-2.0).
  - Settings, from `docs/flux3.md`: `--cfg-scale 1.0 --sampling-method euler --steps 16`.
- **Fibo 2.0 Dev** (`image/fibo-2.0-dev`, licence `bria-fibo`: CC BY-NC 4.0, non-commercial),
  Bria's 8B model, supported since sd.cpp's `2e2ba3f` (2026-10-21), `docs/fibo2.md`.
  - `briaai/Fibo-2.0-dev` `72415bd0` provides the transformer as two diffusers shards,
    `transformer/diffusion_pytorch_model-0000{1,2}-of-00002.safetensors` (9.99 and 6.26 GB, bf16),
    with their `diffusion_pytorch_model.safetensors.index.json` and `config.json`.
    `--diffusion-model` takes the index ("using safetensors index format") and loads both shards.
    The VAE, FLUX.2's, is in the same repository, `vae/diffusion_pytorch_model.safetensors`
    (0.34 GB) with its `config.json`, and loads as it is.
  - The repository is gated: the user fills its form on the Hugging Face page (organisation,
    site, Discord, intended use) and downloads with a read token.
  - The text encoder is Qwen3-VL-4B-Instruct, as Bria's model card says: FLUX.3 Image Klein's
    GGUF, hard-linked into the model's directory, so that each directory holds its model whole
    with one file on disk. Its vision projector, `mmproj-Qwen3VL-4B-Instruct-F16.gguf`
    (0.84 GB) from the same repository, passed with `--llm_vision`, reads the reference images
    of an edit (`docs/fibo2.md`); FLUX.3 Image Klein edits without it.
  - What sd.cpp leaves out of Bria's pipeline (`docs/fibo2.md`): it takes a plain prompt or a
    structured JSON caption, but does not expand a short prompt into the structured caption the
    model was trained on, which Bria's `bria-vgl` does with Gemini; it does not draw the text to
    render as glyph images (pull request #2216, open on 2026-10-29); and masked editing is not
    wired (#2215, open).
  - Settings, as the shipped inventory gives them: from Bria's model card,
    `--cfg-scale 5.0 --sampling-method euler --steps 50`, and the card's negative prompt for
    text-to-image, a JSON caption (`NEGATIVE_T2I`), inline in `--negative-prompt`; and
    `--offload-to-cpu`, the text encoder on the GPU, as for the other two models (below).

```bash
hf download leejet/Qwen-Image-2.1-GGUF qwen_image_2.1-Q8_0.gguf \
  --revision cc11433936a06e9765f7c0c0b1f0436cfd2b9856 \
  --local-dir "$MODELS_DIR/image/qwen-image-2.1"
hf download Qwen/Qwen-Image-2.1 vae/config.json vae/diffusion_pytorch_model.safetensors \
  --revision d26bb61231c349cf6b7896fa83353113880e1ba3 \
  --local-dir "$MODELS_DIR/image/qwen-image-2.1"
hf download Qwen/Qwen3-VL-8B-Instruct-GGUF \
  Qwen3VL-8B-Instruct-Q8_0.gguf mmproj-Qwen3VL-8B-Instruct-F16.gguf \
  --revision f982a07559d4a2f6c8744d840bf6fccab30eea96 \
  --local-dir "$MODELS_DIR/image/qwen-image-2.1"
hf download black-forest-labs/flux-3-image-klein model.safetensors ae.safetensors \
  --revision 337337621b8526e120f2275d250242f4a6bedd75 \
  --local-dir "$MODELS_DIR/image/flux-3-image-klein"
hf download Qwen/Qwen3-VL-4B-Instruct-GGUF Qwen3VL-4B-Instruct-Q8_0.gguf \
  --revision 1cd86afb9a95c410a6038ab3b40d8b578c892266 \
  --local-dir "$MODELS_DIR/image/flux-3-image-klein"
hf download briaai/Fibo-2.0-dev \
  transformer/config.json transformer/diffusion_pytorch_model.safetensors.index.json \
  transformer/diffusion_pytorch_model-00001-of-00002.safetensors \
  transformer/diffusion_pytorch_model-00002-of-00002.safetensors \
  vae/config.json vae/diffusion_pytorch_model.safetensors \
  --revision 72415bd021af33b2cbc63525683a114ff8ac294e \
  --local-dir "$MODELS_DIR/image/fibo-2.0-dev"
ln "$MODELS_DIR/image/flux-3-image-klein/Qwen3VL-4B-Instruct-Q8_0.gguf" \
  "$MODELS_DIR/image/fibo-2.0-dev/"
hf download Qwen/Qwen3-VL-4B-Instruct-GGUF mmproj-Qwen3VL-4B-Instruct-F16.gguf \
  --revision 1cd86afb9a95c410a6038ab3b40d8b578c892266 \
  --local-dir "$MODELS_DIR/image/fibo-2.0-dev"
```

Every file was checked on 2026-10-20, Fibo 2.0 Dev's on 2026-10-29, the two vision projectors on
2026-10-30. Each LFS file's SHA-256
equalled the Hub's `lfs.oid` at the commit above, and each small file's git blob SHA-1 (a
`config.json`, the shards' index) equalled its `oid`. No template check applies: sd.cpp reads no
`chat_template` from a text encoder's GGUF.

**Measured** with the server alone on the GPU, which held 15 MiB before. Each start was cold, after
`POSIX_FADV_DONTNEED` on each file. The request was one image as OpenWebUI sends it: `n` 1,
`size` `1024x1024`, `response_format` `b64_json`. The answer is a 1024×1024 PNG in
`data[0].b64_json`. Both images were clean, and the text on Qwen-Image's sign was legible.

| Model, load | Ready, cold | VRAM at ready | Peak VRAM | One image, 1024² |
|---|---|---|---|---|
| Qwen-Image 2.1, lazy (default) | 0.58 s | 277 MiB | 20243 MiB | 39.8 s, the weights' load included |
| Qwen-Image 2.1, `--eager-load` | 3.73 s | 15943 MiB | 20399 MiB | 36.8 s (sampling 35.4 s, 20 steps, CFG in two passes) |
| FLUX.3 Image Klein, `--eager-load` | 5.01 s | 15127 MiB | 18591 MiB | 11.3 s (16 steps, CFG 1.0) |
| Fibo 2.0 Dev, `--eager-load` | 6.77 s | 20060 MiB | 23520 MiB | 114.6 s (50 steps, CFG in two passes) |
| Fibo 2.0 Dev, `--eager-load --backend te=cpu` | 3.27 s, warm | 15980 MiB | 19440 MiB | 118.8 s, the prompt's encoding 4.15 s on the CPU against 0.13 s |

Fibo 2.0 Dev's rows were measured on 2026-10-29 at `master-1024-77b72a2`, the others at
`master-975-95b3090`; at `master-1024-77b72a2`, Qwen-Image 2.1 takes 29.0 s an image and FLUX.3
Image Klein 9.8 to 10.1 s. All peak above 18 GiB at 1024×1024 with their weights in VRAM, where
only whisper fits beside them; the inventory keeps their weights in RAM, below.

**Fibo 2.0 Dev's sizes**, measured on 2026-10-29 with 4 steps, since the peak does not depend on
their number. sd-server bounds no size a request asks for, and `--auto-fit` places the weights
once, at the start: a request that needs more VRAM than is left aborts the server (`CUDA error:
out of memory`, exit 134), and the request gets no answer.

| Size | Text encoder on the GPU | Text encoder on the CPU |
|---|---|---|
| 1024×1024 | 23520 MiB | 19440 MiB |
| 1248×832, 1424×800 | 23488 MiB, 23808 MiB | not measured |
| 1536×1024 | aborts while the VAE decodes, twice | 21104 MiB |
| 1536×1536 | 23600 MiB, once | 23600 MiB |
| 2048×2048 | not measured | aborts |

From 2026-10-29 to 2026-10-30, the text encoder ran on the CPU, `--backend te=cpu`: it freed
4.1 GB of VRAM, so that sizes up to 1536×1536 passed, for 4 s more an image. The portrait made
both ways at 50 steps is the same image, its details apart. With the weights in RAM, below, the
encoder is back on the GPU. Set aside then:
- the encoder on the GPU: at 1024×1024, a peak 943 MiB under the GPU's 24463 MiB, and an abort
  at 1536×1024;
- `--vae-tiling`, which decodes 1536×1024 in 21458 MiB and 2048×2048 in 23168 MiB, but changes
  the image: at 1024×1024, the tiled decode is darker and warmer than the whole one, with a
  pattern of tiles in their difference (PSNR 25.3 dB between the two).

**FLUX.3 Image Klein's and Qwen-Image 2.1's sizes**, measured on 2026-10-29 at
`master-1024-77b72a2` the same way: one server per size, alone on the GPU, `--eager-load` and each
model's settings above but 4 steps, one image through the OpenAI route.

| Size | FLUX.3 Image Klein | Qwen-Image 2.1 |
|---|---|---|
| 1024×1024 | 18594 MiB | 20404 MiB |
| 1536×1024 | 20258 MiB | 22564 MiB |
| 1536×1536 | 22754 MiB | 23804 MiB |
| 2048×2048 | aborts, `CUDA error: out of memory`, exit 134 | aborts, the same |

So 1536×1536 is the largest size each of the three makes with its weights in VRAM: with the DiT
and the text encoder resident, the VAE's working set at 2048×2048 does not fit beside them, flash
attention and direct convolution on (`flash attention auto: CUDA0 -> on`, `conv2d direct auto:
CUDA0 -> on` in the server's log). From 2026-10-29 to 2026-10-30, the budgets were the peaks at
1536×1536.

**The decode at larger sizes**, compared on 2026-10-30 with FLUX.3 Image Klein: one server per
row, alone on the GPU, `--eager-load`, 4 steps, seed 42, the PSNR by ImageMagick's `compare`
against the whole decode.

| Size, decode | Sampling | Decode | Peak VRAM | Peak RSS | Against the whole decode |
|---|---|---|---|---|---|
| 1536², the weights in VRAM | 6.06 s | 1.29 s | 22753 MiB | 3.3 GiB | — |
| 1536², `--vae-tiling`, 256 px tiles | 6.08 s | 3.09 s | 17197 MiB | 3.1 GiB | 24.9 dB |
| 1536², `--vae-tile-size 1024` | 6.09 s | 2.08 s | 18593 MiB | 3.3 GiB | 33.8 dB, blue −4.5 on average |
| 1536², `--offload-to-cpu` | 6.31 s | 1.29 s | 11695 MiB | 15.7 GiB | identical |
| 2048², `--offload-to-cpu` | 12.83 s | 2.35 s | 19625 MiB | 15.8 GiB | — |
| 2048², `--vae-tile-size 1024` | 12.64 s | 4.50 s | 18685 MiB | 3.2 GiB | 29.7 dB |
| 2048², `--backend vae=cpu` | 12.66 s | 299.44 s | 18525 MiB | 27.9 GiB | 71.2 dB, 80 pixels differ |

- `--offload-to-cpu` is `--params-backend *=cpu` (`examples/common/common.cpp:939`): the weights
  stay in RAM (`total params memory size … VRAM 0.00MB`) and the GPU takes them at each step,
  for under 4 % more sampling. The VAE then has the GPU to itself and decodes the image whole.
  sd.cpp's own documents run FLUX.3, Qwen-Image 2.1 and Fibo 2.0 with it.
- When the whole decode does not fit, sd-server retries in tiles of half the image rather than
  abort: "Reducing VAE decode tiles from 2560x2560 to 1280x1280 image pixels", "VAE decode ran
  out of memory; retrying with spatial tiling" (`src/core/backend_fit.cpp:513`, `:518`).
- Tiles shift the colours, less with larger tiles; the CPU's decode is the GPU's image, in 5
  minutes and 28 GiB of RAM at 2048×2048.

**The three with `--offload-to-cpu`**, measured on 2026-10-30 the same way, Fibo's text encoder
on the GPU. Each cell gives the sampling at 4 steps, the decode and the peak VRAM; "tiles" marks a
decode sd-server retried in tiles.

| Size | FLUX.3 Image Klein | Qwen-Image 2.1 | Fibo 2.0 Dev |
|---|---|---|---|
| 1536² | 6.31 s, 1.29 s, 11695 MiB | 15.48 s, 2.60 s, 10648 MiB | 22.99 s, 1.22 s, 17602 MiB |
| 2048² | 12.83 s, 2.35 s, 19625 MiB | 34.64 s, 4.67 s, 18210 MiB | 46.74 s, 2.29 s, 19640 MiB |
| 2560² | 23.99 s, 7.25 s tiles, 15095 MiB | 66.49 s, 15.22 s tiles, 13038 MiB | 86.13 s, 7.19 s tiles, 20438 MiB |
| 3840×2160 | 34.19 s, 9.21 s tiles, 16508 MiB | not measured | not measured |
| 3072² | 41.98 s, 10.77 s tiles, 17434 MiB | 114.87 s, 22.71 s tiles, 15336 MiB | 149.71 s, 10.73 s tiles, 22378 MiB |
| 4096² | 108.73 s, 20.73 s tiles, 23384 MiB | 293.95 s, 41.86 s tiles, 20170 MiB | 398.53 s, 20.70 s tiles, 23514 MiB |

Every size up to sd-server's limit of 4096 a side (`k_max_generation_dimension`,
`examples/server/runtime.h:95`) passes; the peak RSS is the weights', 16.1, 16.9 and 20.9 GiB
at most. At 1536×1536, FLUX's and Qwen-Image's images are those made with the weights in VRAM, 0
pixels differing. At full steps, the fox's prompt, Fibo with its negative prompt:

| Model, steps | 2048×2048 | 4096×4096 |
|---|---|---|
| FLUX.3 Image Klein, 16 | 53.3 s, 19626 MiB | 457.2 s, 23384 MiB, tiles |
| Qwen-Image 2.1, 20 | 175.4 s, 18210 MiB | about 25 minutes, from 4 steps |
| Fibo 2.0 Dev, 50 | 582.6 s, 19640 MiB | about 83 minutes, from 4 steps |

The user's verdict by eye, on 2026-10-30: all very beautiful, Qwen-Image first, FLUX second,
Fibo third, its fox seen from afar and almost more realistic; FLUX at 4096×4096 sharper when
zoomed, but not worth its time.

Decided on 2026-10-30 (the user): each image model runs with `--offload-to-cpu`, its `vram_mib`
its peak at 4096×4096, 23384, 20170 and 23514 MiB, so that no size a client asks aborts the
server; a request from the command line chooses its own size. It costs 16 to 21 GiB of RAM
while an image model is loaded. Set aside: a patch making sd-server refuse a side above 2048,
whose budgets at 2048×2048 would let whisper run beside an image model; the VAE on the CPU;
tiles with the weights in VRAM.

Decided on 2026-10-30 (the user), after the first uses from OpenWebUI: OpenWebUI asks for
1024×1024, to make and to edit, and compresses the images attached to 1024 a side. 2048×2048
and 4096×4096 serve the command line: their minutes are too long for a chat. The budgets stay
those of 4096×4096, since a chat model may ask its own size
([openwebui.md §7](../openwebui.md#7-images)). Set aside: 2048×2048 in OpenWebUI, the largest size
decoded whole, decided earlier the same day.

**A size from the command line.** The OpenAI route takes any `size` up to `4096x4096`; sd.cpp's
documents ask sides divisible by 32 for Qwen-Image 2.1 and by 16 for Fibo 2.0. A side above 2048
is decoded in tiles, and the time grows faster than the pixels, per the tables above. The key goes
to curl on its standard input, so that no command line shows it:

```bash
printf 'Authorization: Bearer %s\n' "$(cat ~/.local/state/ai-ctl/api-key)" |
  curl -s -H @- -H 'Content-Type: application/json' http://127.0.0.1:8080/v1/images/generations \
    -d '{"model": "flux-3-image-klein", "prompt": "…", "size": "2048x1152", "response_format": "b64_json"}' |
  jq -r '.data[0].b64_json' | base64 -d > image.png
```

**Compared on four prompts** on 2026-10-21, at 1024×1024 through the OpenAI route, with
`--eager-load` and each model's settings above: Qwen-Image 2.1 took 36.7 to 36.8 s an image, FLUX.3
Image Klein 11.5 to 12.0 s. Both wrote "ai-ctl" on the fox's sign and the poster's two French lines
exactly; FLUX added two lines of illegible small text to the poster. The user's verdicts by eye, on
2026-10-28:

| Prompt | Verdict |
|---|---|
| A fox in a snowy birch forest at dawn, a sign reading "ai-ctl", photograph | Qwen-Image's fox is very realistic, its sign less so; FLUX's fox is less realistic, its scenery and its sign more. |
| A lighthouse on a Breton coast in a storm at dusk, oil painting | Both very beautiful and not comparable: Qwen-Image's reads as a watercolour, FLUX's as brushwork. |
| An elderly fisherman holding a cup of coffee, 85mm photograph | FLUX wins by far, realistic in the hands, the face and the cup; Qwen-Image's face is wrinkled to excess, its fingers stuck together and blurred. |
| A 1970s cinema poster, "La Nuit des GPU" | Two styles not to be compared: Qwen-Image's a montage, FLUX's a classic AI generation. |

Neither wins: the two are complementary, chosen by the need, and both are kept.

**Fibo 2.0 Dev compared** on 2026-10-29, the text encoder on the GPU: the four prompts above, and
two whose result is counted rather than judged, a row of five red apples and one green pear, the
pear second from the left and a brass bell on the third apple from the right (the example of
Bria's model card), and a row of five cups, blue, yellow, blue, yellow, blue, a black cat to its
right and a red book under the leftmost cup. Fibo took 114.9 to 115.3 s an image at 50 steps and
46.4 to 46.7 s at 20.

| Prompt | Fibo 2.0 Dev, 50 and 20 steps | Qwen-Image 2.1 | FLUX.3 Image Klein |
|---|---|---|---|
| Apples | 6 apples in two rows, the bell on the rightmost | 4 apples, the pear and the bell placed right | 4 apples, the pear and the bell placed right |
| Cups | 7 cups in two rows; the cat and the book right | the row and its colours right, the book beside the cup | all right, the cat behind the last cup |
| Poster | "Vint-quatre" at 50 steps, "Vint-quzra" and a stray "La" at 20, and illegible small text | exact (2026-10-21) | exact, with two illegible lines (2026-10-21) |

Fibo also signed the lighthouse painting "Leighhouse". With a plain prompt, the counts, positions
and text its model card puts forward do not show: they rest on the structured caption sd.cpp
does not write. The user's verdict by eye, on 2026-10-29: Fibo's rendering is above the two
others', at 50 steps more than at 20.

Decided on 2026-10-29 (the user): Fibo 2.0 Dev is kept as the third image model, for its
rendering, when two minutes an image are acceptable; FLUX.3 Image Klein stays the fast one and the
one that follows a layout, Qwen-Image 2.1 the other style. Set aside: waiting for more reviews or
a distilled text-to-image version, none announced by Bria (Fibo 2.0 Edit Turbo is distilled for
editing); Comfy-Org's INT8 files (8.41 GB), which [stack.md §5](../stack.md#where-models-come-from)
does not admit since Bria publishes a format sd.cpp reads.

**Editing.** sd-server's `POST /v1/images/edits` takes a multipart form, `prompt`, `image` or
`image[]`, an optional `mask`, `n` and `size`; each image becomes a reference image of the
request (`examples/server/routes_openai.cpp:85–210`), and llama-swap routes it by the form's
`model`. Each model reads the references through its preset, `flux2`, `qwen` or `fibo2`
(sd.cpp's `docs/edit.md`); Qwen-Image 2.1 and Fibo 2.0 Dev also need their projector, above.
Measured on 2026-10-30 through the gateway, as OpenWebUI sends an edit: the fox of FLUX at
2048×2048 as reference, `size` `2048x2048`, the prompt "Put a red knitted wool scarf around the
fox's neck, keep everything else unchanged", each model's settings, its load included.

| Model | Answer | Sampling, a step | Peak VRAM | PSNR against the reference |
|---|---|---|---|---|
| FLUX.3 Image Klein, 16 steps | 84.5 s | 68.12 s, 4.3 s | 21305 MiB | 22.4 dB, warmer |
| Qwen-Image 2.1, 20 steps | 484.8 s | 464.48 s, 22.7 s | 22773 MiB | 16.0 dB, brighter and more magenta |
| Fibo 2.0 Dev, 50 steps | 1605.0 s | 1585.15 s, 31.7 s | 22927 MiB | 25.5 dB |

- Each added the scarf and kept the scene. The user's verdict by eye: all three change the
  colours; Fibo's stays the closest but its scarf is the worst made; FLUX's is the most
  beautiful, its colours much changed.
- An edit's step costs more than a generation's at the same size, 3.2, 8.7 and 11.7 s at
  2048×2048: the reference's tokens join the image's. Fibo's preset scales a single reference to
  the output's size (`docs/fibo2.md`).
- Qwen-Image 2.1 keeps the reference's and the prompt's keys and values in a prefix cache
  (`docs/qwen_image_2.1.md`). At 2048×2048, the cache did not fit: sd.cpp filled the GPU, then
  released it and sampled without it ("insufficient memory for prefix caching; retrying without
  it"). Without the cache (`--model-args qwen_image_2_1_prefix_cache=false`), the same edit
  peaks at 18211 MiB at the same speed, and a generation's step at 2048×2048 takes 9.27 s
  against 8.66.
- `--max-vram`, a "per-device budget in GiB for managed weights and runner buffers", bounds that
  cache: at 19, the edit peaks at 20075 MiB. It does not bound a generation at 4096×4096, which
  holds 21487 MiB through its sampling.
- Qwen-Image 2.1's generation at 4096×4096 holds 21183 MiB through its sampling with the fox's
  prompt and its sign, 21449 with the projector, against 20154 with the shorter prompt of the
  table above.

Decided on 2026-10-30 (the user): Qwen-Image 2.1's `vram_mib` is its edit's peak, 22773 MiB,
its prefix cache kept, so that it runs alone, as the two others do. It replaces its budget of
20170 MiB above, which let whisper large-v3-turbo run beside it. Set aside: the cache turned off
and a budget of 21449 MiB, which runs alone too, for 7 % more time a step; `--max-vram`, which
does not bound the generation.

An edit from the command line, the key on curl's standard input:

```bash
printf 'Authorization: Bearer %s\n' "$(cat ~/.local/state/ai-ctl/api-key)" |
  curl -s -H @- http://127.0.0.1:8080/v1/images/edits \
    -F model=flux-3-image-klein -F 'prompt=…' -F size=2048x2048 -F response_format=b64_json \
    -F 'image=@reference.png;type=image/png' |
  jq -r '.data[0].b64_json' | base64 -d > edit.png
```

---

## 5. OpenWebUI

These facts come from the OpenWebUI image installed on this machine, version 0.12.3
(`backend/open_webui/routers/images.py`). How to set OpenWebUI is in
[openwebui.md §7](../openwebui.md#7-images).

- **What it sends.** With image engine `openai`, OpenWebUI sends `POST {base}/images/generations`
  with the key as a bearer token. The JSON holds `model`, `prompt`, `n`, `size` (`512x512` by
  default) and `response_format: "b64_json"`, unless the model matches `^gpt-image`.
- **What it reads.** It reads `data[].b64_json`, or `data[].url`. Its model list is a fixed list
  of OpenAI's models, so the model is typed by hand.
- **The fit.** sd-server always answers `data[].b64_json`, which OpenWebUI reads. The size is
  OpenWebUI's setting, unless the chat model gives its image tool one of its own
  (`routers/images.py:614–618`, `:913–917`), and the steps come from the server's command line.

---

## Sources

- Phase 0 of the `audio-image-backends` roadmap, whose report records each command, probe and
  measure, with its output; Fibo 2.0 Dev's, in phase 2's report, under 2026-10-29.
- stable-diffusion.cpp at `master-1024-77b72a2`: `docs/fibo2.md`, `docs/edit.md`; its pull
  requests #2180, #2215 and #2216, read on 2026-10-29.
- Fibo 2.0 Dev's model card, `briaai/Fibo-2.0-dev` at `72415bd0`: the architecture, the licence,
  the settings and the negative prompt.
- stable-diffusion.cpp at `master-975-95b3090`: `CMakeLists.txt`, `.gitmodules`,
  `examples/server/main.cpp`, `examples/server/routes_openai.cpp`, `examples/common/common.cpp`,
  `src/name_conversion.cpp`, `docs/flux3.md` and `docs/qwen_image_2.1.md`; and the main branch at
  `0518b34`, fetched on 2026-10-20, for the key.
- The captures of [tests/fixtures/README.md](../../tests/fixtures/README.md), "Audio and Image
  Servers".
- The Hugging Face API, `/api/models/<repo>` and `POST /api/models/<repo>/paths-info/<commit>`,
  read on 2026-10-20 and 2026-10-29, with the token for the gated repositories.
- OpenWebUI 0.12.3, read in its image (digest `sha256:4860ba00…`).
