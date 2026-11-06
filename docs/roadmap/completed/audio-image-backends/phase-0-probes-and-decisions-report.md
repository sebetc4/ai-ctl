# Phase 0 Report: Probes And Decisions

**Phase:** [phase-0-probes-and-decisions.md](phase-0-probes-and-decisions.md)
**Start Commit:** `docs(audio-image-backends): create the roadmap` of 2026-10-19

---

## Work Log

### 2026-10-20

Phase opened. It is the roadmap's first phase: no previous report to read and no pending
restructuring to settle. Read before working: the roadmap's README, this phase file, the summary
of `v2-review-fixes`, `docs/gpu/llama-swap.md`, `docs/security.md`, `docs/stack.md` §4.4, §5,
§6.4, §7.3 and §8, `tests/fixtures/README.md`, `src/domain/recipe.rs`, the CMake options of
`src/domain/build.rs`, and the working notes on backends on unix sockets and on a model library.

What the first tasks start from:
- the builder adds to a recipe's options `CMAKE_BUILD_TYPE=Release`, `GGML_CUDA=ON`,
  `GGML_NATIVE=ON`, `CMAKE_CUDA_ARCHITECTURES=120a-real`, and the CUDA and host compilers; the
  current llama.cpp build, `b11700`, records nvcc 13.4.92, g++ 16.2.1 and
  `BUILD_SHARED_LIBS=OFF` in its `build.json`. The llama.cpp recipe adds its `LLAMA_BUILD_*`
  options, and `--list-devices` is its device listing;
- the gateway is stopped and the GPU holds 15 MiB: a server can be probed alone on it;
- llama-swap v263 is installed. The audio and image routes of `docs/gpu/llama-swap.md` come from
  its documentation of 2026-10-08 and 2026-10-09, not from a run of v263;
- llama-swap forwards the client's `Authorization` or `x-api-key` header unchanged
  (`docs/security.md` §3), so a server guards itself only if it checks that header against
  ai-ctl's key; an entry's `env:` can hand the key to a server under any variable name;
- the provenance rule of `docs/stack.md` §5 covers GGUF files from Unsloth only; whisper.cpp's and
  stable-diffusion.cpp's models come from other repositories;
- the hand builds go to `~/.cache/ai-ctl-probes/`: `/tmp` is a tmpfs wiped at each reboot, and
  the phase spans several sessions. Never into `~/.local/share/ai-ctl/builds/`.

Below, `$P` is `~/.cache/ai-ctl-probes`: `sources/`, `build/`, `logs/`, and `captures/<project>/`
for the fixtures to come. Each clone is made as the builder makes it,
`git clone --quiet --filter=blob:none --no-checkout -- <repository> <dir>`, and every command runs
with `LC_ALL=C`.

**whisper.cpp, built by hand.** `https://github.com/ggml-org/whisper.cpp.git`, cloned with 95 tags.
- Releases are `vX.Y.Z`, the newest `v1.9.5` (2026-10-06, commit `d1be6fde`). Beside them sit
  41 `b<N>` tags, older forms (`1.0.3`, `0.0.6-1`), `v1.7.4-pre-1`-style tags and two personal
  test tags. GitHub's prerelease flag cannot pick them: `v1.9.3` is marked prerelease and
  `b4938` is not (`GET /repos/ggml-org/whisper.cpp/releases`, 2026-10-20). A tag pattern of
  strict `vX.Y.Z` with numeric ordering does.
- The server is `examples/server`, target `whisper-server`, added under
  `WHISPER_BUILD_EXAMPLES` (on by default for a standalone build); `WHISPER_BUILD_SERVER` is
  declared and read nowhere at `v1.9.5`. `WHISPER_BUILD_IS_DEV`, on by default, only
  appends `-dev` to the version CMake prints (`1.9.5-dev` on the release tag).
- Configured with the builder's options plus `WHISPER_BUILD_TESTS=OFF`,
  `WHISPER_BUILD_EXAMPLES=ON`, `WHISPER_BUILD_SERVER=ON`:
  `cmake -S $P/sources/whisper.cpp -B $P/build/whisper.cpp -G Ninja -DBUILD_SHARED_LIBS=OFF
  -DCMAKE_BUILD_TYPE=Release -DCMAKE_CUDA_ARCHITECTURES=120a-real
  -DCMAKE_CUDA_COMPILER=/usr/local/cuda-13.4/bin/nvcc -DCMAKE_CUDA_HOST_COMPILER=/usr/bin/g++
  -DCMAKE_CXX_COMPILER=/usr/bin/g++ -DCMAKE_C_COMPILER=/usr/bin/gcc -DGGML_CUDA=ON -DGGML_NATIVE=ON
  -DWHISPER_BUILD_TESTS=OFF -DWHISPER_BUILD_EXAMPLES=ON -DWHISPER_BUILD_SERVER=ON`: 2.3 s, ggml
  0.26.0 vendored, "Replacing 120-real in CMAKE_CUDA_ARCHITECTURES_NATIVE with 120a-real", nvcc
  13.4.92 with GNU 16.2.1.
- `cmake --build $P/build/whisper.cpp --target whisper-server --parallel 24`: 185 steps, 144
  CUDA objects, 194 s, no warning. The binary lands at `bin/whisper-server`, 52.8 MB, linked
  against `libcudart.so.13`, `libcublas.so.13` and `libcublasLt.so.13` of
  `/usr/local/cuda-13.4`, and `libgomp`. `cuobjdump --list-elf` lists 144 cubins, all `sm_120a`,
  and no PTX.
- No `--version`: the server prints "error: unknown argument: --version" and its usage, and
  exits 0, so the builder's smoke test would pass on any binary that starts. No option lists the
  devices; started with a missing model, it fails before ggml initialises CUDA (exit 3). Its
  recipe has no device listing, and `--help` (exit 0, 61 lines, captured to
  `captures/whisper.cpp/whisper-server-help.txt`) is the only thing it runs without a model.

**whisper-server, read at `v1.9.5`** (`examples/server/server.cpp`):
- Options `--host` (127.0.0.1), `--port` (8080), `-m/--model`, `--inference-path`
  (`/inference`), `--request-path`, `--public` (`examples/server/public`, relative to the
  working directory, served as static files), `--convert` (runs `ffmpeg` through `system()` on a
  temporary file in `--tmp-dir`, `.` by default).
- **No API key**, by option or variable. Every response carries
  `Access-Control-Allow-Origin: *` and allows the `authorization` header.
- Routes: `GET /` (a page), `POST <inference-path>` (multipart `file`, OpenAI's fields; the
  request's `model` is ignored), `POST /load` (multipart `model`: a path on the host, loaded in
  place of the model; a file that fails to load makes the server `exit(1)`), `GET /health`.
  To serve llama-swap's `/v1/audio/transcriptions`, the server needs
  `--inference-path /v1/audio/transcriptions`, to be checked behind llama-swap.
- Readiness: the model loads before the server binds its port, so nothing answers during the
  load; then `/health` answers 200 `{"status":"ok"}`. Its 503 `{"status":"loading model"}` only
  shows during a `/load`.
- Without `--convert`, the audio is decoded in memory by `read_audio_data`.

**stable-diffusion.cpp and audio.cpp, cloned and read** while whisper.cpp compiled.
- `https://github.com/leejet/stable-diffusion.cpp.git`, a personal account (7735 stars, MIT):
  747 tags, of which 539 `master-<N>-<hash>` releases published several times a day (newest
  `master-975-95b3090`, 2026-10-18, commit `95b30906`), 206 older `master-<hash>` and two
  `workflow-` tags.
  - Four submodules: `ggml` from `https://github.com/leejet/ggml.git`, a fork on its branch
    `sd.cpp` (`dc998771`, 2026-10-17); `thirdparty/libwebp`, `thirdparty/libwebm`; and
    `examples/server/frontend` from `leejet/sdcpp-webui`. The builder initialises no submodule.
  - `SD_CUDA=ON` sets `GGML_CUDA`. WebP and WebM default on only when their submodule is there.
  - `SD_SERVER_BUILD_FRONTEND` defaults on: with `pnpm` on the `PATH`, as it is on this machine
    (`~/.local/share/pnpm/pnpm`), the build runs `pnpm install` in the frontend, packages fetched
    from npm and run at build time. The recipe turns it off and leaves that submodule out.
  - `sd-server` (`examples/server/main.cpp`): `listen_ip` 127.0.0.1, port 1234, **no API key**
    anywhere in the sources, and a pre-routing handler that echoes the request's `Origin` with
    `Access-Control-Allow-Credentials: true`. Routes `/`, `/v1/models`,
    `/v1/images/generations`, `/v1/images/edits`, `/sdapi/v1/*`, `/sdcpp/v1/*` (jobs, video,
    upscale); no `/health`. The model loads before `listen`.
- `https://github.com/0xShug0/audio.cpp.git`, a personal account (3649 stars, created
  2026-06-23; Apache-2.0, "Copyright 2026 ShugoAI LLC", which GitHub reports as NOASSERTION):
  28 tags, releases `vX.Y.Z` beside suffixed ones (`v0.9.2-drama3-base-small`,
  `v0.8.2-audio8-perf-hotfix`), older `release-0.N` and test tags. Newest plain release
  `v0.9.2`, 2026-10-15, commit `0097d365`.
  - ggml vendored in `external/ggml`; one submodule, `external/audio.cpp-server-frontends`,
    declared with an SSH URL (`git@github.com:`), so it cannot be fetched without an SSH key.
    It holds the optional modules: MP3/FLAC decoding, HTTPS listener.
  - CUDA is `ENGINE_ENABLE_CUDA`, native CPU `ENGINE_ENABLE_NATIVE_CPU`; `CMakeLists.txt`
    then sets `GGML_CUDA` and `GGML_NATIVE` from them with `FORCE`. Read alone, that looked as
    if the builder's `GGML_CUDA=ON` would be overwritten; it is not: a `GGML_CUDA` given on the
    command line becomes `ENGINE_ENABLE_CUDA`'s default (lines 60–89). Probed: configured with
    the builder's options only, the cache holds `ENGINE_ENABLE_CUDA=ON`, `GGML_CUDA=ON`,
    `GGML_NATIVE=ON`, and the log says "Including CUDA backend".
  - `audiocpp_server` reads its models from a JSON file, `--config`, and manages several
    itself (`lazy_load`, `max_loaded_models`, `/v1/tasks/unload_models`). The default build
    has no model downloader (`AUDIOCPP_BUILD_NATIVE_MODEL_MANAGER` off). No API key in its
    README or sources. CORS off by default. Its core audio routes take WAV only: "A browser's
    WebM recording is not a WAV". A transcription request may name a WAV path on the host.

**stable-diffusion.cpp, built by hand** at `master-975-95b3090`, with only the `ggml` submodule
initialised (`git submodule update --init --filter=blob:none -- ggml`).
- Configured with the builder's options plus `SD_CUDA=ON`, `SD_BUILD_EXAMPLES=ON`,
  `SD_SERVER_BUILD_FRONTEND=OFF`, `SD_WEBP=OFF`, `SD_WEBM=OFF`: 2.5 s, ggml 0.27.0 at
  `dc998771`, 120a-real, "Frontend disabled or directory not found".
- `cmake --build … --target sd-server --parallel 24`: 266 steps, 147 CUDA objects, 251 s.
  Warnings: nvcc's `#174-D` "expression has no effect" in
  `ggml/src/ggml-cuda/sageattention/attn_utils.cuh` (6 lines), `-Wstringop-overflow` in
  `thirdparty/stb_image.h`, `wstring_convert` deprecated in `src/core/util.cpp`.
- `bin/sd-server`, 92.0 MB, the same CUDA 13.4 libraries as whisper-server; 147 cubins, all
  `sm_120a`, no PTX.
- `--version` prints `stable-diffusion.cpp version master-975-95b3090, commit 95b3090`, exit 0.
  `--help`: 274 lines, exit 0. `--list-devices` (exit 0) prints ggml's CUDA banner, then one
  `name<TAB>description` line per device: `CUDA0<TAB>NVIDIA GeForce RTX 5090 Laptop GPU` and `CPU`.
  `recipe::cuda_devices` reads `CUDA<n>:` lines only, so it finds no device in this listing.
  Both captured to `captures/stable-diffusion.cpp/`.
- Options: `-l/--listen-ip`, `--listen-port`, `--serve-html-path`, the model's files
  (`-m`, `--diffusion-model`, `--vae`, `--clip_l`, `--clip_g`, `--t5xxl`, `--llm`…),
  `--lora-model-dir`, `--rpc-servers` (sends the work to remote ggml RPC servers),
  `--offload-to-cpu`, `--max-vram`. Weights load lazily on first use unless `--eager-load`
  is given, so the time to ready and the VRAM after the start may not show a loaded model.

**audio.cpp, built by hand** at `v0.9.2`, configured with the builder's options plus
`ENGINE_ENABLE_CUDA=ON` and `ENGINE_ENABLE_NATIVE_CPU=ON`: 29.7 s, the vendored ggml reports
version 0.12.0, 120a-real, CUDA 13.4.92 with GNU 16.2.1, OpenMP 5.2. CMake warns that
`external/sentencepiece` asks for a CMake older than 3.10.
- `cmake --build … --target audiocpp_server --parallel 24`: 1190 steps, 143 CUDA objects, 343 s.
  Warnings: `-Wnonnull` in libstdc++'s `stl_algobase.h`, `-Warray-bounds` in sentencepiece's
  protobuf-lite, `-Wmissing-field-initializers` in several model sessions, `wstring_convert`
  deprecated in Kokoro's G2P, an unused parameter.
- `bin/audiocpp_server`, 100.6 MB; 142 cubins, all `sm_120a`, no PTX. It links `libcufft.so.12`
  besides the libraries of the other two, from `/usr/local/cuda-13.4`: two CUDA objects of its
  own runtime (`istft_cuda_runtime.cu`, `torch_random_cuda_runtime.cu`) sit outside ggml.
- `--version` (exit 0) prints `audio.cpp dev`, then `git: 0097d36 2026-10-15`,
  `build: Release, gcc 16.2.1, Linux x86_64`, `backends: cpu,cuda`: "dev" on the release tag.
  `--help`: 71 lines, exit 0. `--list-devices` (exit 0) prints ggml's CUDA banner,
  `available_devices=2`, then `CUDA:0 "NVIDIA GeForce RTX 5090 Laptop GPU" [GPU]` and
  `CPU:0 "…" [CPU]`: a third form, which `recipe::cuda_devices` does not read either.
  All captured to `captures/audio.cpp/`.

**Where each project's models come from**, read in its repository at the commit built:
- whisper.cpp: `models/download-ggml-model.sh` downloads from `huggingface.co/ggerganov/whisper.cpp`,
  its author's own account; `models/README.md` gives each file's size and SHA-1
  (`large-v3` 2.9 GiB, `large-v3-turbo` 1.5 GiB).
- audio.cpp: every package of `model_specs/*.json` is a GGUF converted by the project, in one
  Hugging Face repository, `audio-cpp/audio.cpp-gguf` (456 files, licence `other`, head
  `02f5de92` on 2026-10-19), downloaded at revision `main`, never pinned. `docs/model_licenses.md`
  lists each family's upstream weights and licence; among the TTS families that clone a voice,
  Qwen3-TTS, CosyVoice3, VoxCPM2 and Chatterbox are Apache-2.0 or MIT, and Fish Audio S2 Pro,
  Higgs TTS 3 and OmniVoice are non-commercial. audio.cpp also runs Qwen3-ASR (Apache-2.0),
  speech-to-text beside whisper.cpp's.
- stable-diffusion.cpp: its `docs/` point to the publishers' safetensors (black-forest-labs, Qwen,
  krea) and to repackagings and quantizations by Comfy-Org (73 links), unsloth, leejet,
  QuantStack, city96 and others. A model is several files: diffusion model, VAE, and one or more
  text encoders, an LLM of 4 to 24B for the newest models.

**The candidates on the Hub** (`/api/models/<repo>`, read on 2026-10-20; heads to be pinned at
the download, sizes in GiB):
- speech-to-text: `ggerganov/whisper.cpp` (head `5359861c`, 2024-10-29, MIT):
  `ggml-large-v3.bin` 2.88, `ggml-large-v3-turbo.bin` 1.51.
- text-to-speech: `audio-cpp/audio.cpp-gguf` (head `02f5de92`):
  `Qwen3-TTS-12Hz-1.7B-Base-GGUF/qwen3-tts-12hz-1.7b-base-q8_0_v2.gguf` 2.51, the package
  `model_specs/qwen3_tts.json` recommends; `Fish-Audio-S2-Pro-GGUF/fish-audio-s2-pro-q8_0.gguf`
  5.88. Whether a family needs files beyond its GGUF is in its spec, not read yet for Fish.
- Qwen-Image 2.1: `leejet/Qwen-Image-2.1-GGUF` (head `cc114339`) `qwen_image_2.1-Q8_0.gguf`
  7.16, Qwen's own weights being two diffusers shards that `--diffusion-model` cannot take as
  one file; `Qwen/Qwen3-VL-8B-Instruct-GGUF` (head `f982a075`, Apache-2.0)
  `Qwen3VL-8B-Instruct-Q8_0.gguf` 8.11 (the `mmproj` is for editing only); the VAE, Qwen's
  `Qwen/Qwen-Image-2.1` (head `d26bb612`) `vae/diffusion_pytorch_model.safetensors` 1.26, or
  Comfy-Org's `Comfy-Org/Qwen-Image-2.1` (head `a8342359`) `vae/qwen_image_2.1_vae_bf16.safetensors`
  0.63.
- FLUX.3 Image Klein: `black-forest-labs/flux-3-image-klein` (head `33733762`, gated `auto`,
  licence `other`) `model.safetensors` 10.34 and `ae.safetensors` 0.31;
  `Qwen/Qwen3-VL-4B-Instruct-GGUF` (head `1cd86afb`) `Qwen3VL-4B-Instruct-Q8_0.gguf` 3.99.
  sd.cpp's `docs/flux3.md` names FLUX.2-dev's VAE, also gated; the Klein repository carries its
  own `ae.safetensors`. Which one sd.cpp needs is not settled.
- About 43 GiB in all; `/ai` has 1.8 TB free.

**A premise corrected.** The provenance decision first said sd.cpp converts diffusers names for
SD1, SDXL, SD3, Flux, LLaDA and Lumina2 only, from a listing cut at eight lines. The whole of
`src/name_conversion.cpp` also converts Krea2, PixArt, MiniMax-H3, an SD1 VAE, an SDXL
ControlNet, and `convert_diffusers_to_original_wan_vae(name, qwen_image_2_1)` (line 1181), used
for `VERSION_QWEN_IMAGE_2_1` (line 1280): Qwen's own diffusers VAE may well load. The decision
now admits Comfy-Org's VAE only if a probe shows Qwen's does not. Earlier in the session the
user was also told that audio.cpp overwrites the builder's `GGML_CUDA`; the probe above showed
it does not, and the user was told of the correction.

**The machine, for the downloads.** `hf` 1.14.0 (`~/.local/bin/hf`), `HF_HOME=/ai/cache/huggingface`,
no token in it yet. `/ai/models/` holds `llm/` and `tts/Voxtral-4B-TTS-2603/`: 7.5 GB of
Mistral-format safetensors and voice embeddings dated 2026-05-10, from an earlier project, not
ai-ctl's; left untouched. audio.cpp's `voxtral_tts` family reads a GGUF, not these files.
Directories proposed for the candidates, ids to confirm: `stt/whisper-large-v3/`,
`stt/whisper-large-v3-turbo/`, `tts/qwen3-tts-1.7b-base/`, `tts/fish-audio-s2-pro/`,
`image/qwen-image-2.1/`, `image/flux-3-image-klein/`.

**State at the session's end.** Tasks 1 to 3 are done. `~/.cache/ai-ctl-probes/` holds the three
sources at their release (stable-diffusion.cpp with its `ggml` submodule only), their build trees
with `whisper-server`, `sd-server` and `audiocpp_server`, the configure and build logs, and the
captures: each project's `git tag --list`, each server's `--help`, `sd-server`'s and
`audiocpp_server`'s `--version` and `--list-devices`. None is in `tests/fixtures/` yet (task 4).
Nothing runs; nothing has been downloaded. The provenance rule is written here only; its place
in `docs/` is still to choose, with the domain documents of task 12 or beside `docs/stack.md`
§5. The user asked to start the downloads in the next session.

**A second session, the same day**, resumed from the entry above. `hf auth whoami`: "Not logged in"; the gateway is down, the GPU
holds 15 MiB. The user chose one directory per candidate, the Qwen3-VL text encoder in the
directory of the image model it serves.

**The TTS files, read in audio.cpp's specs at `v0.9.2`.** An audio.cpp GGUF carries its package
spec in `audiocpp.model_spec.*` metadata and its config and tokenizer files as embedded
sidecars (`docs/gguf.md`), so one GGUF is the whole model. The safetensors package of
`model_specs/qwen3_tts.json`, `qwen3_tts_1_7b_base_safetensors`, names eight files of
`Qwen/Qwen3-TTS-12Hz-1.7B-Base` (head `fd4b2543`, 2026-01-23, Apache-2.0), exactly the files Qwen
publishes, 4.2 GiB with the speech tokenizer: under the provenance rule, the publisher's
files come first. The session option `weight_type` takes `native`, `f32`, `f16`, `bf16` or
`q8_0` (`src/framework/model_spec/options.cpp:14`), a quantization at load. Fish Audio publishes
`fishaudio/s2-pro` (head `1de9996b`) as two safetensors shards and a `codec.pth`, a pickle;
`model_specs/fish_audio.json`'s safetensors source reads `model_audio_cpp.safetensors.index.json`
and `codec.safetensors`, names of audio.cpp's own conversion, so the publisher's format is not
one the engine reads, and audio.cpp's GGUF is admitted. `audio-cpp/audio.cpp-gguf` offers it in
Q8_0 (5.88 GiB) and BF16 (9.53 GiB).

**FLUX.3 Image Klein's VAE, read in `docs/flux3.md` at `master-975-95b3090`:** "The
`ae.safetensors` included with FLUX.3 can also be used", beside FLUX.2-dev's. Klein's own
repository is enough; a run will confirm it. The text encoder is Qwen3-VL 4B, from
`Qwen/Qwen3-VL-4B-Instruct-GGUF`. `docs/qwen_image_2.1.md` asks for
`qwen_image_2.1_vae_bf16.safetensors`, Comfy-Org's, and warns that earlier Qwen-Image and Wan 2.2
VAEs do not fit; whether Qwen's own `vae/diffusion_pytorch_model.safetensors` loads is the probe
still to run. `leejet/Qwen-Image-2.1-GGUF` stops at Q8_0; Qwen-Image 2.1's licence on the Hub is
`other`.

**Downloads started**, each repository pinned to its head, unchanged since the
first session, with `hf` 1.14.0 into `/ai/models/<use>/<id>/`, the repository's layout kept as in
`/ai/models/llm/`: `ggerganov/whisper.cpp` at `5359861c` (`ggml-large-v3.bin`,
`ggml-large-v3-turbo.bin`); `Qwen/Qwen3-TTS-12Hz-1.7B-Base` at `fd4b2543` (the package's eight
files); `audio-cpp/audio.cpp-gguf` at `02f5de92` (Fish S2 Pro Q8_0 and BF16);
`Qwen/Qwen-Image-2.1` at `d26bb612` (`vae/config.json`, `vae/diffusion_pytorch_model.safetensors`),
`leejet/Qwen-Image-2.1-GGUF` at `cc114339` (`qwen_image_2.1-Q8_0.gguf`),
`Qwen/Qwen3-VL-8B-Instruct-GGUF` at `f982a075` (`Qwen3VL-8B-Instruct-Q8_0.gguf`). The Hub's LFS
SHA-256 of each file is read from `POST /api/models/<repo>/paths-info/<commit>`. FLUX.3 Image
Klein waits for the user's token.

**Task 4, the fixtures.** Before copying the captures, each command ran again with its streams
apart: `whisper-server --help` prints everything on stderr; `sd-server` and `audiocpp_server`
print `--help` and `--version` on stdout, and `--list-devices` prints ggml's CUDA banner on
stderr and the listing on stdout, the captures holding both as `2>&1` interleaved them. Each tag
list is `git tag --list` in its clone, byte for byte. Ten fixtures copied into `tests/fixtures/`,
each with its row in a new section of `tests/fixtures/README.md`, which cites this report by its
path under `pending/`: the path is updated when the roadmap's folder moves.

**`audiocpp_server`'s routes, read at `v0.9.2`** (`--help`, `app/server/request_dispatch.h`,
`app/server/runtime.cpp`, `app/server/main.cpp:210–240`). Options: `--host` (127.0.0.1),
`--port` (8080), `--config <server.json>` for the models; without `--config`, `lazy_load` is
forced on. `--help`'s list of endpoints is wider than what this build serves: the model
management routes (`/v1/ui/models-root`, `/v1/ui/browse-directories`, install, delete) exist
only with `AUDIOCPP_HAS_NATIVE_MODEL_MANAGER`, off here, and `/v1/models/load` and `/unload`
answer 403 without `--ui-management`. Still served to any client: `/v1/ui/upload`, which writes
the request's file under the server's upload root whenever the UI is on, and the UI is on by
default with a `--config` (`--no-ui` turns it off); `/v1/ui/path-status`; and
`/v1/tasks/unload_models` and `/v1/tasks/unload_all_models`, with no gate. No route checks an
`Authorization` header.

**whisper verified.** `ggml-large-v3.bin`: SHA-256 `64d182b4…94d1e2` and SHA-1 `ad82bf6a…ff8062`;
`ggml-large-v3-turbo.bin`: SHA-256 `1fc70f77…e2bc69` and SHA-1 `4af2b29d…07e941`. Each equals the
Hub's LFS hash at `5359861c` and the SHA-1 of `models/README.md` at `v1.9.5`.

**whisper-server measured alone on the GPU.** A probe script, kept outside the repository with
the runs under `$P/runs/`, starts the server, polls `/health` every 50 ms, samples
`nvidia-smi`'s `memory.used` every 100 ms, sends one request once `/health` answers 200, and
stops the server. Cold starts follow `posix_fadvise(POSIX_FADV_DONTNEED)` on the model file.
Request: `samples/jfk.wav` (11 s, English) to `--inference-path /v1/audio/transcriptions`,
`response_format=json`. Flash attention on by default, 4 of 24 threads.

| Model | Ready, cold | Peak VRAM (GPU at 15 MiB before) | One request |
|---|---|---|---|
| large-v3 | 1.84 s (1.15 s from the page cache) | 3895 MiB | 0.34 s |
| large-v3-turbo | 1.09 s | 2213 MiB | 0.20 s |

Until the model is loaded nothing listens: curl gets no connection (`000`), then `/health`
answers 200 `{"status":"ok"}`. Both transcribe the sample word for word.

**llama-swap v263 in front, a hand-written configuration** (`$P/swap/config.yaml`: `apiKeys` from
`${env.AI_CTL_API_KEY}`, ai-ctl's key in llama-swap's environment only, `-listen 127.0.0.1:18100`).
A fake upstream, a Python HTTP server that logs each request's method, path and headers, served as
model `echo`, shows what the gateway routes:
- by the request's `model`, read from a JSON body or a multipart form alike:
  `POST /v1/audio/transcriptions`, `POST /v1/audio/speech`, `GET /v1/audio/voices?model=…`,
  `POST /v1/images/generations`, `POST /v1/images/edits`, `POST /sdapi/v1/txt2img`, each
  forwarded under its own path; `POST /audioapi/v1/tasks/run`, forwarded as `/v1/tasks/run`;
- `GET /sdapi/v1/sd-models`, with or without `?model=`: llama-swap's 404 "404 page not found";
- `/upstream/echo/<path>` forwards any path, audio.cpp's `/v1/ui/upload` included;
- each forwarded request carries the client's `Authorization: Bearer <key>` unchanged, its
  `Host` (`127.0.0.1:18100`) and an added `X-Forwarded-For: 127.0.0.1`; llama-swap's own
  `GET /health` and `GET /v1/models` after the start carry no key;
- without the key, the gateway answers 401.

Model `whisper-large-v3` behind it (`cmd` with `--port ${PORT}`, `checkEndpoint: /health`):
the first transcription through the gateway took 1.55 s, the load included, the next 0.23 s;
`response_format` `json`, `text` and `verbose_json` all answer. The backend, on `127.0.0.1:5801`,
answers the same request without any key, and with `Origin: https://attacker.example` it answers
`Access-Control-Allow-Origin: *`: any local process or web page can transcribe through it, or
`POST /load` a path on the host, a file that fails to load ending the server.

**What OpenWebUI sends, read in the image installed on this machine** (`open-webui:main`, digest
`sha256:4860ba00…`, version 0.12.3, `routers/audio/__init__.py` and `routers/images.py`,
extracted with `podman run --network none`), each with the key as `Authorization: Bearer`:
- speech-to-text, engine `openai`: `POST {base}/audio/transcriptions`, multipart by default,
  fields `model` (the configured one), `language` when set, `file` with its filename. Before
  that, a file whose codec is not `flac`, `m4a`, `mp3`, `mp4`, `mpeg`, `wav` or `webm` is
  converted to MP3 by pydub; a browser's recording, Opus in WebM, is converted. Over 20 MiB, it
  is re-encoded to 16 kHz mono MP3 at 32 kb/s and split. `whisper-server` reads MP3:
  `samples/jfk.mp3` through the gateway, `language=en`, answers the same text as the WAV.
  audio.cpp's transcription routes take WAV only.
- text-to-speech, engine `openai`: `POST {base}/audio/speech`, JSON `model`, `voice`, `input`,
  plus the configured parameters; any audio content type comes back and is transcoded to MP3.
  The model list (`GET {base}/audio/models`, then `{base}/models`) and the voice list
  (`GET {base}/audio/voices`, without `?model=`) are requested **without** the key, so through
  the gateway they get 401 and OpenWebUI falls back to `tts-1`, `tts-1-hd` and OpenAI's six
  voices: the model and the voice are typed by hand.
- image generation, engine `openai`: `POST {base}/images/generations`, JSON `model`, `prompt`,
  `n`, `size` (`512x512` by default) and `response_format: "b64_json"` unless the model matches
  `^gpt-image`; it reads `data[].b64_json`, or `data[].url`. Its model list is a fixed list of
  OpenAI's models.

**`sd-server`'s OpenAI routes, read at `master-975-95b3090`** (`examples/server/routes_openai.cpp`):
`GET /v1/models` answers one fixed model, `sd-cpp-local`. `POST /v1/images/generations` reads
`prompt`, `n`, `size` (`WxH`), `output_format` (`png` by default) and `output_compression`,
ignores `model` and `response_format`, and always answers `data[].b64_json`, which OpenWebUI
reads. Steps, CFG and sampler come from the server's command line. A prompt may embed
`<sd_cpp_extra_args>{…}</sd_cpp_extra_args>`, JSON read by `SDGenerationParams::from_json_str`
(`examples/common/common.cpp`): `width`, `height` and `batch_count` with no bound, and
`ad_model`, a model path on the host.

**The downloads stalled.** After about two minutes each, the Xet transfers of `hf` 1.14.0 stopped
growing their `.incomplete` files while the Hub's API still answered: 0 bytes in 20 s, twice.
Killing the two `hf` processes did not stop their scripts, `set -e` notwithstanding: each went on
to its next file and printed "done" for the file killed, so those lines say nothing.
Restarted as one queue, one file at a time, with `HF_HUB_DISABLE_XET=1` and each `hf` exit status
printed: it resumed the partial files at about 3 MiB/s, where whisper's first download ran at
about 25 MB/s. `hf` warns that unauthenticated requests get lower rate limits; the user's token,
needed for FLUX.3 Image Klein, is asked for now.

**Verified as they landed**, against `POST /api/models/<repo>/paths-info/<commit>`: an LFS file's
SHA-256 against its `lfs.oid`, a small file's git blob SHA-1 against its `oid`. All equal:
Qwen3-TTS's eight files at `fd4b2543` (`model.safetensors` 3857413744 bytes, SHA-256
`38fc7fc5…`; `speech_tokenizer/model.safetensors` `836b7b35…`), `qwen_image_2.1-Q8_0.gguf` at
`cc114339` (`f8b244b0…`), and Qwen's `vae/config.json` and `vae/diffusion_pytorch_model.safetensors`
at `d26bb612` (`a07a1b7c…`). The queue took 1033 s for Qwen3-TTS's weights and 2000 s for the
7.2 GiB GGUF, still without a token.

**The user's token.** `hf auth whoami` with `HF_HOME=/ai/cache/huggingface`: `user=sebetc4`; the
token file, `/ai/cache/huggingface/token`, is mode 0644, as is the older
`~/.cache/huggingface/token`. With it, `black-forest-labs/flux-3-image-klein` answers its file list
(head `33733762`, unchanged; licence `flux-kommunity-license`) and a ranged download answers 206.
Its repository holds `model.safetensors` (10.34 GiB, SHA-256 `c3e9813c…`), `ae.safetensors`
(0.31 GiB, `868fe7b3…`), the diffusers folders, and `fp8r` and `nvfp4` variants. A second queue
downloads `ae.safetensors`, `Qwen/Qwen3-VL-4B-Instruct-GGUF`'s `Qwen3VL-4B-Instruct-Q8_0.gguf`
(head `1cd86afb`, Apache-2.0) and `model.safetensors` into `image/flux-3-image-klein/`. The first
queue's current file went from about 3 to 10 MiB/s.

**Qwen3-TTS under `audiocpp_server`, alone on the GPU.** The configuration
(`$P/audiocpp/qwen3-tts.json`): one model, `family` `qwen3_tts`, `path` the directory of Qwen's
files, `task` `tts`, `lazy_load` false, `max_loaded_models` 1, `ui` false, and a `voice_dir`
holding `jfk.wav` with its transcript in `prompt_text`, a provisional reference until the user's.
- The first start failed: "model spec not found for family 'qwen3_tts' (provide
  --model-spec-override, embed it in the GGUF, enable AUDIOCPP_DEPLOYMENT_BUILD, or install
  model_specs/qwen3_tts.json)", exit 1. A safetensors package carries no spec, and a normal build
  looks for `model_specs/` from the working directory upward. `AUDIOCPP_DEPLOYMENT_BUILD=ON`
  compiles every spec of the built commit into the binary (`CMakeLists.txt:16–40`,
  `docs/gguf.md`). The probes go on with the model's `model_spec_override` set to the source
  tree's `model_specs/qwen3_tts.json` at `v0.9.2`, the same contract.
- A model that is not lazy loads in `ServerState`'s constructor, before the server listens
  (`app/server/runtime.cpp:131`, `main.cpp`); `/health` then answers 200
  `{"status":"ok","backend":"cuda","models":1,"ui":false,"ui_management":false}`. Nothing
  answers during the load.
- Measured with the probe script, cold, a request of one English and one French sentence with
  `voice: "jfk"` to `/v1/audio/speech`; the answer is `audio/wav`, 16-bit mono at 24 kHz:

  | Weights | Ready, cold | Peak VRAM | English, ~8 s of audio | French, ~10 s |
  |---|---|---|---|---|
  | native | 2.67 s | 6025 MiB | 3.13 s | 2.04 s |
  | `qwen3_tts.weight_type` `q8_0` in `session_options` | 5.89 s | 4759 MiB | 2.79 s | 1.89 s |

  The server's `--log` trace confirms `server.model.session_options.qwen3_tts.weight_type q8_0`.
- whisper large-v3-turbo transcribes the English output word for word; the French one reads "Le
  Biff Renard, bon sauté par-dessus le champ par-dessus. Ceci est un essai de synthèse vocal
  locale sur un seul carte graphique.", with an English reference voice. The comparison by ear
  waits for the user's reference recording.

**Qwen3-TTS behind llama-swap**, model `qwen3-tts` with `--config … --no-ui --host 127.0.0.1
--port ${PORT}`: `--port` overrides the configuration's. A model's id in llama-swap must equal
its id in audio.cpp's configuration, since the body reaches the server unchanged.
- `POST /v1/audio/speech`: 4.50 s cold through the gateway, the load included; a WAV comes back.
- `GET /v1/audio/voices?model=qwen3-tts`: `{"voices":["jfk"]}`, a list of strings, where
  OpenWebUI reads `voices[].id` and `voices[].name`.
- `GET /upstream/qwen3-tts/v1/models` answers the model's host path.
- `POST /audioapi/v1/tasks/run` reaches the server: 500 "Qwen3 base TTS requires voice clone
  reference audio" for a request naming `voice` inside `request`, a shape this probe got wrong.
- `POST /upstream/qwen3-tts/v1/tasks/unload_all_models` unloads the model while llama-swap
  still counts it running; the next speech request reloads it.
- On the backend's port, `127.0.0.1:5801`, without the key: 200. A CORS preflight from
  `https://attacker.example` gets 404, but a `POST` with `Content-Type: text/plain`, which a page
  sends without preflight, is executed: a JSON body is parsed whatever its type. A page can thus
  run a synthesis on the GPU or `unload_all_models`, without reading the answer.

**The rest of the downloads.** The queues' log (`$P/logs/download-queue.log`, exit statuses all 0):
`Qwen3VL-8B-Instruct-Q8_0.gguf` at 16:54 (741 s), `Qwen3VL-4B-Instruct-Q8_0.gguf` at 17:01,
Fish S2 Pro Q8_0 at 17:06 (760 s). Before the session closed, the two queues, children of
Claude Code's process, were stopped and the two files left restarted under `setsid nohup`, with
their scripts and queue files copied to `$P/bin/` and `$P/queues/`. The user then chose to run
the downloads from their own terminal, given the commands (`HF_HUB_DISABLE_XET=1`, same
revisions, same directories). At the close: FLUX.3 Image Klein's `model.safetensors` is complete,
11097404512 bytes, the Hub's size (`$P/logs/queue-flux-model.status`: rc 0 at 17:20:53); Fish
S2 Pro BF16 is still downloading in the user's terminal, 8.08 of 10.23 GB at 16 MiB/s.
Verified so far: whisper's two files, Qwen3-TTS's eight, Qwen-Image's diffusion model and Qwen's
VAE. Not yet verified: both Qwen3-VL GGUFs, both Fish GGUFs, FLUX's `ae.safetensors` and
`model.safetensors`. The scripts in `$P/bin/` still name this session's scratch folder
(`swapreq.sh`'s `last.body`, and `$P/swap/config.yaml`'s fake upstream): adapt them before use.
Tasks 4 to 6 stay unticked: `sd-server`'s and Fish's readiness and runs behind the gateway are
still to probe.

**A third session, the same day**, resumed from the entry above. Fish S2 Pro BF16 had finished in
the user's terminal (no `.incomplete` left); the GPU holds 15 MiB and no server runs.

**Every download verified.** `$P/bin/verify-hub.py <repo> <commit> <dir> <paths>…` reads
`POST /api/models/<repo>/paths-info/<commit>`, with the token for the gated repository, and
compares each file's size and SHA-256 with its `lfs` entry (a small file's git blob SHA-1 with
its `oid`); the commits are the full ones `hf` recorded in each directory's
`.cache/huggingface/download/*.metadata`. All six equal (`$P/logs/verify-2026-10-20-s3.log`):
`Qwen3VL-8B-Instruct-Q8_0.gguf` at `f982a075` (`0d264b39…`), `Qwen3VL-4B-Instruct-Q8_0.gguf` at
`1cd86afb` (`054721f4…`), FLUX.3 Image Klein's `ae.safetensors` (`868fe7b3…`) and
`model.safetensors` (`c3e9813c…`) at `33733762`, Fish S2 Pro's Q8_0 (`4f051347…`) and BF16
(`b938ecf9…`) at `02f5de92`. With the earlier ones, every file downloaded for the phase is
verified.

**`sd-server` alone on the GPU**, measured with the probe script, cold after `POSIX_FADV_DONTNEED`
on each file (`fincore` shows 0 B resident after it; `/ai` is btrfs on LUKS), ready URL
`GET /v1/models`, one request as OpenWebUI sends it (`req-image.sh`: `model`, `prompt`, `n` 1,
`size` `1024x1024`, `response_format` `b64_json`); the answer is `data[0].b64_json`, a PNG of
1024×1024.
- Qwen-Image 2.1 with **Qwen's own diffusers VAE**: `--diffusion-model qwen_image_2.1-Q8_0.gguf
  --vae vae/diffusion_pytorch_model.safetensors --llm Qwen3VL-8B-Instruct-Q8_0.gguf --cfg-scale
  6.0 --sampling-method euler --steps 20`. It loads (the log: "Wan VAE", 238 f32 tensors) and the
  image is clean, its sign's text legible: Comfy-Org's VAE is not needed, and the provenance rule's
  only exception to the publisher goes unused.
- FLUX.3 Image Klein with its own `ae.safetensors`: `--diffusion-model model.safetensors --vae
  ae.safetensors --llm Qwen3VL-4B-Instruct-Q8_0.gguf --cfg-scale 1.0 --sampling-method euler
  --steps 16`, as `docs/flux3.md` gives them. The diffusion model loads as 359 bf16 tensors; the
  image is clean.
- `--auto-fit`, on by default, places each component by the free VRAM it reads at the start
  ("CUDA0 free 23798 MiB, budget 23286 MiB"), on the GPU first, then RAM, then disk: here all three
  went to `CUDA0`. On a GPU already holding a backend it would place weights in RAM instead of
  failing, read from the log and `--help`, not probed.

  | Model, load | Ready, cold | VRAM at ready | Peak VRAM | One image, 1024² |
  |---|---|---|---|---|
  | Qwen-Image 2.1, lazy (default) | 0.58 s | 277 MiB | 20243 MiB | 39.8 s, the weights' load included |
  | Qwen-Image 2.1, `--eager-load` | 3.73 s | 15943 MiB | 20399 MiB | 36.8 s (sampling 35.4 s, 20 steps, CFG in two passes) |
  | FLUX.3 Image Klein, `--eager-load` | 5.01 s | 15127 MiB | 18591 MiB | 11.3 s (16 steps, CFG 1.0) |

- Readiness: the context is built before `listen`, so nothing answers while it loads (`000`),
  then `GET /v1/models` answers 200 `{"data":[{"id":"sd-cpp-local",…}]}`. Without
  `--eager-load`, "ready" holds no weight: the first request carries the load, and the VRAM llama-swap
  sees after the start is not the model's. SIGTERM ends it with 143, no handler.

**Both image models behind llama-swap**, the scratch configuration's stale paths fixed first
(`swapreq.sh` and the `echo` entry named the first session's scratch folder; both now under `$P`).
Entries `qwen-image-2.1` and `flux-3-image-klein`, `cmd` as above with `--listen-port ${PORT}`
and `--eager-load`, `checkEndpoint: /v1/models`:
- `POST /v1/images/generations` with `model: qwen-image-2.1`: 200 in 41.1 s, the load included,
  then 36.7 s. Then `model: flux-3-image-klein`: 16.9 s, Qwen-Image stopped, FLUX loaded and one
  image made; `GET /running` lists FLUX alone, the GPU at 15771 MiB.
- `POST /sdapi/v1/txt2img` with `model` in the body: 200, `images`, `info`, `parameters`; this
  route takes `width`, `height` and `steps` from the request (512², 8 steps: 3.2 s), where the
  OpenAI route takes steps from the command line.
- On the backend's port, `127.0.0.1:5802`, without the key: 200. With `Origin:
  https://attacker.example`: `Access-Control-Allow-Origin: https://attacker.example` and
  `Access-Control-Allow-Credentials: true`, so any page can read the answers, images included.
- llama-swap stopped on SIGTERM, its child with it: "shutdown complete", the GPU back to 15 MiB.

**Fish S2 Pro under `audiocpp_server`, alone on the GPU.** One configuration per precision
(`$P/audiocpp/fish-s2-pro-{q8_0,bf16}.json`): `family` `fish_audio`, `path` the GGUF, `task`
`tts`, no `model_spec_override` (the spec is embedded), the same `voice_dir` as Qwen3-TTS. The
voice library is the server's, not the family's (`app/server/server_protocol.cpp:339–375`): a
request's `voice` names `<voice_dir>/<voice>.wav` and its transcript in `<voice_dir>/prompt_text`.
A first run sent `model: qwen3-tts` by mistake: 500 `unknown model id: qwen3-tts`, a 500 for a
client's error. Rerun with `fish-s2-pro`; the answer is `audio/wav`, 16-bit mono at 44.1 kHz:

| Weights | Ready, cold | Peak VRAM | English, ~9.5 s of audio | French, ~10 s |
|---|---|---|---|---|
| Q8_0 | 6.78 s | 7953 MiB | 4.59 s | 4.56 s |
| BF16 | 12.26 s | 11199 MiB | 6.36 s | 5.97 s |

SIGTERM ends it with 0.

**The TTS outputs read back by whisper large-v3-turbo**, each with its `language` (the earlier
Qwen3-TTS reading had none), the reference voice JFK's English for all:
- English: all four candidates word for word.
- French, "Le vif renard brun saute par-dessus le chien paresseux. Ceci est un essai de synthèse
  vocale locale sur une seule carte graphique.": Fish BF16 "Les vifs renards bruns sautent…", the
  second sentence exact; Fish Q8_0 the same with "rennais" for "renards"; Qwen3-TTS native "Le Biff
  Renard, bon sauté par-dessus le champ par-dessus. … synthèse vocal locale sur un seul carte
  graphique."; Qwen3-TTS `q8_0` "Le Vif Renard Brun Sauté par le sou, le chien par le sou. Si, et en
  Isse, de Sintes Bocall local…". A reading by machine, with an English voice; the ear decides.

Audio.cpp's routes behind llama-swap were probed with Qwen3-TTS: they belong to the server, the
same for every family, so Fish was not run behind the gateway.

**No key upstream either.** At each project's head, fetched on 2026-10-20 (whisper.cpp `231638a0`
of 2026-10-19, stable-diffusion.cpp `0518b34` of 2026-10-21, audio.cpp `c677d5e` of 2026-10-20),
`git grep -i -E 'api[-_]key|authorization'` over the server's sources finds only whisper-server's
CORS header allowing `authorization`: none of the three servers checks a key on its main branch.

**The decisions put to the user**, with the facts above: the guard, audio.cpp, the place of the
provenance rule, how whisper is chosen; then the scope of "bench et comparaison" and the new
candidates' directories. The answers are under Decisions. Two new candidates follow from them,
read in audio.cpp's specs at `v0.9.2` and on the Hub:
- Voxtral 4B TTS: `model_specs/voxtral_tts.json` has GGUF packages only, BF16 (default) and Q8_0,
  from `audio-cpp/audio.cpp-gguf` (7.48 and 4.40 GiB); preset voices only, "the released
  checkpoint does not contain the encoder needed for reference-audio cloning"; languages en, fr,
  es, de, it, pt, nl, ar, hi. Mistral's `mistralai/Voxtral-4B-TTS-2603` (head `b81be46c`,
  CC-BY-NC-4.0) is `consolidated.safetensors`, `params.json`, `tekken.json` and voices as `.pt`
  pickles, a format audio.cpp does not read: the engine project's conversion is admitted, as for
  Fish.
- Qwen3-ASR 1.7B: the package `qwen3_asr_1_7b_safetensors` downloads `Qwen/Qwen3-ASR-1.7B-hf`,
  Qwen's own transformers-format repository (head `bcd2b5b7`, Apache-2.0, `model.safetensors` 3.8
  GiB), which holds exactly the package's files; `Qwen/Qwen3-ASR-1.7B` is two shards without
  `tokenizer.json`. The publisher's files are admitted. Its transcription routes take WAV only,
  where OpenWebUI sends MP3 or WebM.

Downloads started with `HF_HUB_DISABLE_XET=1`: `Qwen/Qwen3-ASR-1.7B-hf` at `bcd2b5b7` into
`stt/qwen3-asr-1.7b/`, then both Voxtral GGUFs from `audio-cpp/audio.cpp-gguf` at `02f5de92`,
unchanged, into `tts/voxtral-4b-tts/` (`$P/logs/download-2026-10-20-s3.log`), at 3 to 8 MiB/s.

**The provenance rule written** in `docs/stack.md` §5, "Where Models Come From", with the checks
extended to every file and the template check limited to the engines that read one: audio.cpp
reads a chat template only for `outetts`, `ace_step` and `higgs_audio_tts` at `v0.9.2`
(`git grep chat_template src app`), none of them a candidate. `docs/security.md` §7 points to it.

**The key patch, prototyped on each server** while the downloads ran, so that phase 1 writes the
recipes from builds that worked. Each patch is a `git diff` in `$P/patches/`, made on the source
at its built tag and rebuilt incrementally (6 to 7 s each):
- `whisper.cpp-v1.9.5-api-key.patch` (`examples/server/server.cpp`, +23 −4): `LLAMA_API_KEY` read
  before the model loads, the server exits 1 with "error: LLAMA_API_KEY is not set" without it; a
  pre-routing handler lets `<request-path>/health` through and any request whose
  `Authorization` is `Bearer <key>` or whose `X-Api-Key` is the key, and answers every other one
  401 with an OpenAI-style error; the default CORS headers are dropped. whisper-server's error
  handler turned every status but 400 and 500 into 404 "File Not Found": it now leaves 401 alone.
- `stable-diffusion.cpp-master-975-95b3090-api-key.patch` (`examples/server/main.cpp`, +20 −14):
  the same check in place of the pre-routing handler that echoed the `Origin` with credentials,
  and a `GET /health` answering `{"status":"ok"}`, exempt, so that `/v1/models` stays guarded.
- `audio.cpp-v0.9.2-api-key.patch` (`app/server/request_dispatch.h`, `main.cpp`, +29): the check
  at the top of `RequestDispatcher::handle`, which the legacy and the parallel runtimes share,
  `GET /health` exempt, header names already lower-cased by its HTTP layer; `main.cpp` throws
  before the models load when the variable is empty. The body is read before the dispatch, so a
  request without the key is refused after its body is received, within
  `max_request_body_bytes`.
- Checked on each server directly with a throwaway key (`$P/bin/key-checks.sh`): health 200
  without the key; without it or with a wrong one, 401 on a GET, on the generation POST, on an
  `OPTIONS` preflight and on a `text/plain` POST from `Origin: https://attacker.example`, and on
  whisper's `/load` and audio.cpp's `unload_all_models` and `/v1/audio/voices`; with `Bearer` or
  `X-Api-Key`, 200 (a transcription, an image, a speech); no `Access-Control-*` header to any
  origin.
- Behind llama-swap v263, each entry given `env: ["LLAMA_API_KEY=${env.AI_CTL_API_KEY}"]`, as
  ai-ctl's configuration already does for llama-server, and `checkEndpoint: /health`: the health
  checks pass without a key, and a transcription (whisper-large-v3), a speech and the voice list
  (qwen3-tts), an sdapi image and `/upstream/flux-3-image-klein/v1/models` (FLUX) all answer 200
  through the gateway with the key; the gateway answers 401 without it; FLUX's backend port
  answers `/v1/models` 401 and `/health` 200 without it.
- Left for phase 1: the comparison is a plain `==`, as llama-server's own (`std::find` over its
  keys, `tools/server/server-http.cpp:260–300` at `b11700`, which reads `X-Api-Key` only when
  `Authorization` is empty and strips a `Bearer ` prefix); the variable's name `LLAMA_API_KEY`
  keeps the gateway's entries uniform but is llama.cpp's; each patch must be checked against every
  new tag.

**The adjustments to the later phases, approved.** Put to the user at the end of the work above:
phase 1 learns the recipes' patches, submodules and three device listings, with
`AUDIOCPP_DEPLOYMENT_BUILD`; phases 2 and 3 serve several models per use. The user: "oui
validé". Applied at once rather than at this phase's closure, since nothing left in this phase
changes them; listed under Deviations. Qwen3-ASR's download had finished
(`qwen3-asr rc=0` at 21:12:33); Voxtral's was running.

**Qwen3-ASR 1.7B verified and measured.** Its seven files at `bcd2b5b7` equal the Hub's
(`model.safetensors` 4076193080 bytes, SHA-256 `2db53c7d…`; `$P/logs/verify-qwen3-asr.log`).
Under the patched `audiocpp_server` (`$P/audiocpp/qwen3-asr-1.7b.json`: `family` `qwen3_asr`,
`task` `asr`, `model_spec_override` the source tree's spec, as for Qwen3-TTS), cold, the probe
key in `LLAMA_API_KEY`: ready in 2.35 s, peak VRAM 4903 MiB; JFK (11 s, English) in 0.47 s,
word for word; Fish BF16's French (9.5 s) in 0.31 s, word for word. A first run sent the French
WAV under the name `speech-fr.bin` and got 400 "only WAV audio uploads are currently supported
for transcription; MP3 support is planned": `is_wav_upload_filename`
(`app/server/server_protocol.cpp:694`, called at `runtime.cpp:1301`) judges an upload by its
file name, not its content.

**The French outputs read by the three speech-to-text candidates**, `language` `fr`, the same
WAV files: the second sentence is exact for every reader on Fish's outputs, and the first,
"Le vif renard brun saute…", splits them:

| Output | whisper large-v3 | whisper large-v3-turbo | Qwen3-ASR 1.7B |
|---|---|---|---|
| Fish Q8_0 | "Lévi-Frené Bruyn saute…" | "Les vifs rennais bruns sautent…" | "Lévy, Frédéric Brun saute…" |
| Fish BF16 | "Les vifs renards bruns sautent…" | the same | exact |
| Qwen3-TTS native | — | "Le Biff Renard, bon sauté…", second sentence altered | "Levif Renard, un soleil par dessus le champ…", "…sur un seul cartographique." |
| Qwen3-TTS `q8_0` | — | unintelligible | "Le vif, rieur brun sauté…", second sentence unintelligible |

Synthetic speech in an English voice: it ranks the TTS outputs (Fish BF16 first, Qwen3-TTS's
French poor whatever the reader), not the readers, which the user's recordings will compare.

**State at the session's end.** At the user's request ("peut tu sttoper les download je vais les
lancer depuis mon terminal"), the download queue was stopped: its script, then its `hf` process.
Voxtral's Q8_0 stopped at 2631925760 of its 4.40 GiB, in
`tts/voxtral-4b-tts/.cache/huggingface/download/Voxtral-4B-TTS-2603-GGUF/*.incomplete`, which
`hf` resumes; its BF16 had not started. The user runs both from a terminal, with the same
revision and directory. Tasks 1 to 6 and 9 to 11 are ticked; task 8 waits for Voxtral's
verification, task 7 for the user's material, task 12 for the domain documents. The probe
clones under `$P/sources/` hold the three key patches uncommitted (their `git diff` is
`$P/patches/*.patch`), so the binaries under `$P/build/` refuse to start without
`LLAMA_API_KEY`: export a throwaway key before a probe; `req-*.sh` send it as a bearer token.
`$P/swap/config.yaml` passes the key by `env:` and checks `/health`; the unpatched version is
`$P/swap/config-unpatched.yaml`. Nothing runs, the GPU holds 15 MiB.

### 2026-10-21

Resumed from the entry above. The user's download of the two Voxtral GGUFs stopped on an error
at about 75% of the BF16; no `hf` runs, the GPU holds 15 MiB.
- Voxtral Q8_0 is complete and verified (`$P/logs/verify-voxtral-q8_0.log`):
  `voxtral-4b-tts-2603-q8_0.gguf` at `02f5de92`, 4722918208 bytes, SHA-256 `46eca2bf…54972d`,
  equal to the Hub's.
- Voxtral BF16 stopped at 6710886400 of its 8030847808 bytes (84%), in its `.incomplete`, last
  written on 2026-10-20 at 22:04. The earlier queue's log shows the same failure on the Q8_0:
  "The read operation timed out", `hf`'s download timeout being 10 s by default
  (`HF_HUB_DOWNLOAD_TIMEOUT`). The user restarts it from a terminal, same revision and
  directory, with `HF_HUB_DOWNLOAD_TIMEOUT=60`; `hf` resumes the partial file.
- Voxtral BF16 complete, resumed from the partial file, and verified
  (`$P/logs/verify-voxtral-bf16.log`): `voxtral-4b-tts-2603-bf16.gguf` at `02f5de92`, 8030847808
  bytes, SHA-256 `883f53bc…60081d`, equal to the Hub's. Every file downloaded for the phase is
  verified; task 8 ticked.

**Voxtral 4B TTS under the patched `audiocpp_server`, alone on the GPU**, measured as Fish was:
the probe script, cold after `POSIX_FADV_DONTNEED` (`fincore` 0 B), a throwaway key in
`LLAMA_API_KEY`, ready URL `/health`. Configuration `$P/audiocpp/voxtral-4b-tts-{q8_0,bf16}.json`:
`family` `voxtral_tts`, `path` the GGUF, `task` `tts`, no `model_spec_override` (embedded), no
`voice_dir`. The request (`$P/bin/req-tts-voxtral.sh`) is Fish's two sentences with `voice`
`neutral_female` in English and `fr_female` in French, two of the 20 presets the GGUF embeds
(`model_specs/voxtral_tts.json` at `v0.9.2`: `voice_id`, default `casual_male`; no seed given, so
the acoustic seed is random and two runs differ). The answer is `audio/wav`, 16-bit mono at 24 kHz:

| Weights | Ready, cold | Peak VRAM | English | French |
|---|---|---|---|---|
| Q8_0 | 2.36 s | 5305 MiB | 2.07 s for 11.1 s of audio | 1.16 s for 7.8 s |
| BF16 | 3.92 s | 8459 MiB | 2.64 s for 10.5 s | 1.44 s for 6.7 s |

`/health` answers nothing until the model is loaded (`000`), then 200; SIGTERM ends it with 0.

**Voxtral's outputs read back** (`$P/bin/readback-voxtral.sh`, each reader alone on the GPU, with
its `language`; the readings in `$P/runs/voxtral-readback/`): whisper large-v3 and Qwen3-ASR 1.7B
read all four word for word, French included; whisper large-v3-turbo reads them word for word
but for "A quick brown fox" on the Q8_0's English. Where Fish BF16's French split the readers on
its first sentence and Qwen3-TTS's was poor, Voxtral's French presets read exactly at both
precisions. Voxtral has no voice cloning: it covers the preset voices, Fish and Qwen3-TTS the
cloning. The ear decides, on the user's material.

**Task 12, the domain documents, started.** Reading `whisper-server --help` again for them: its
`-l/--language` defaults to `en`. Probed with large-v3-turbo on Voxtral BF16's outputs, without
a `language` field, as OpenWebUI sends a request when no language is set
(`$P/runs/whisper-language-default/`): with the default, the French reads "The vif renard brun is
on the side of the chien paresseux. This is a local synthèse on a graph graphically.",
`verbose_json` giving `language` `english`; with `-l auto`, it is detected as `french` and read
word for word, the English unchanged. Qwen3-ASR under `audiocpp_server`, without `language`,
reads both word for word (`$P/runs/qwen3-asr-language-default/`). Phase 2's command task now
gives `whisper-server` `-l auto`.

**Task 12 done.** Three documents, one per server, each listed in `docs/README.md`:
`docs/audio/whisper-cpp.md`, `docs/audio/audio-cpp.md` and `docs/image/stable-diffusion-cpp.md`.
Each holds its server's build, serving options, routes, readiness, the key and its prototype
patch, the models with their pinned `hf download` commands and measures, and OpenWebUI's fit,
with sources and dates. Their sources name this phase's report without its path, which changes
when the roadmap's folder moves.
- OpenWebUI's speech-to-text requests are in the whisper document only, and audio.cpp's links to
  them. The guard's decision is written once in whisper's document, with the options set aside,
  and the other two link to it.
- `docs/gpu/llama-swap.md` gained the routing facts probed on v263 and the decision "Several
  models per use", which spans the three domains. Its backends table links the three documents
  and gives the measured peaks.
- Its Voxtral (vLLM-Omni) and Fish Speech rows, Python backends this roadmap leaves out, stay for
  phase 3's update of that table.
- Phase 1 still writes `docs/security.md` §7 on the patches, once ai-ctl applies them.

Task 7, the comparison on the user's material, is the phase's last task.

**State at the session's end.** Tasks 1 to 6 and 8 to 12 are ticked; task 7 waits for the
user's material, which the user sends at the next session. Every model of the phase is
downloaded and verified, and every candidate is measured alone on the GPU. `$P/bin/` gained
`readback-voxtral.sh`, which has each speech-to-text reader transcribe Voxtral's four outputs;
it takes the TTS outputs from `$P/runs/voxtral-4b-tts-{q8,bf16}-cold/`. The probe binaries
still need a throwaway `LLAMA_API_KEY`. Nothing runs, the GPU holds 15 MiB, and nothing is
uncommitted in the repository once this entry is committed.

**A second session, the same day**, resumed from the entry above. The user's material, in
`$P/inputs/`: `reference-fr.mp3` (37.1 s, 44.1 kHz stereo) with its transcript
`reference-fr.txt`, the reference voice; `transcript-fr.mp3` and `transcript-en.mp3` (60 s each),
for speech-to-text. No image prompt: four were written for the comparison. audio.cpp's
`fish_audio` and `qwen3_tts` specs at `v0.9.2` set no limit on a reference's length (other
families do, `git grep` over `src`, `app` and `model_specs`), so the whole 37 s is the reference,
converted to 16-bit mono at 44.1 kHz in `$P/voices-user/ref_fr.wav` with its `prompt_text` line.
Every output and script is under `$P/compare-2026-10-21/` and `$P/bin/compare-*.sh`; a local page,
`$P/compare-2026-10-21/index.html` (`bin/compare-page.py`), puts them side by side for the user.

**Speech-to-text on the user's recordings** (`bin/compare-stt.sh`, each server alone on the GPU,
whisper given the MP3 as OpenWebUI sends it, Qwen3-ASR a 16 kHz mono WAV under a `.wav` name;
once with the recording's `language`, once without). All 18 readings answer 200; with and
without `language` each reader gives the same text, whisper with `-l auto` detecting `french`
and `english`. The words of `reference-fr` each reader hears differently from the user's
transcript, compared lower case, without punctuation, whisper's `verbose_json` wraps joined,
elisions ("qu'au" for "que, au") and numbers in words ("sept") taken as the transcript's words:

| Reader | reference-fr, words differing from the transcript | One request: ref 37 s / FR 60 s / EN 60 s |
|---|---|---|
| whisper large-v3 | 1: "Ça n'est" | 1.17 / 2.01 / 1.54 s |
| whisper large-v3-turbo | none | 0.50 / 0.93 / 0.63 s |
| Qwen3-ASR 1.7B | 4: "Ça n'est", "donnait", "ce spécimen" | 2.43 / 3.77 / 2.87 s |

The transcript says "Ce n'est", which turbo writes; the audio says "Ça n'est" (the user,
2026-10-23). A first count, word error rates of 4.7%,
3.7% and 7.5% of 107 words, also counted the elisions, the numbers in words and a word split by
the wrap ("numé ros"); corrected on 2026-10-23 (`bin/compare-page.py`, which lists the points
where the readings differ). Qwen3-ASR's own errors are "donnait" for "tenait" and "ce spécimen"
for "se spéculent".
On the 60 s recordings the three agree but for a few words ("très qualies" or "très quali", "et
moi" or "et moins", "coûts" or "coups", "3DS" or "3GS" for "Three.js"); Qwen3-ASR keeps
repetitions and fillers ("this this", "des des", a trailing "ah", "Yeah").

**whisper-server's `verbose_json` cuts words.** Its `text` was wrapped every 60 characters on a
token boundary ("évén\nements"): at `v1.9.5`, `verbose_json` turns on token timestamps, which
make `whisper_full` wrap segments at `max_len` 60 (`examples/server/server.cpp:627–636`). With
`response_format` `json`, as OpenWebUI sends, the text breaks only between sentences, checked on
`transcript-fr`.

**Text-to-speech, cloning the user's voice and Voxtral's presets** (`bin/compare-tts.sh`, the
same French and English paragraph for all, a date and an hour in each). All 16 outputs answer
200, `audio/wav`:

| Candidate | FR: time for audio | EN: time for audio |
|---|---|---|
| Qwen3-TTS native | 14.5 s for 17.3 s | 6.7 s for 21.4 s |
| Qwen3-TTS `q8_0` | 13.3 s for 16.3 s | 4.9 s for 18.6 s |
| Fish S2 Pro Q8_0 | 10.9 s for 18.6 s | 10.0 s for 19.6 s |
| Fish S2 Pro BF16 | 13.5 s for 17.3 s | 13.4 s for 19.6 s |
| Voxtral Q8_0, `fr_male` / `neutral_male` | 3.7 s for 20.2 s | 3.1 s for 20.0 s |
| Voxtral BF16, `fr_male` / `neutral_male` | 4.9 s for 19.0 s | 4.4 s for 19.9 s |

Read back by whisper large-v3-turbo with its `language`, every output is intelligible, French
included. Fish at both precisions and Voxtral BF16 render the date and the hour in every output
(corrected on 2026-10-23: the entry first said Fish alone); Qwen3-TTS native reads "9th carquart
5 pm", Qwen3-TTS `q8_0` "October 2021", Voxtral Q8_0's `fr_female` "21 arches 45". Voxtral
changes words in English: "the short" for "this short" (Q8_0, both voices) and "costs" for
"cost" (three of its four English outputs). With the user's 37 s French reference, Qwen3-TTS's French is
intelligible where JFK's English reference made it poor (2026-10-20). The ear decides.

**Images** (`bin/compare-image.sh`, four prompts at 1024×1024 through the OpenAI route,
`--eager-load`): Qwen-Image 2.1 36.7 to 36.8 s an image, FLUX.3 Image Klein 11.5 to 12.0 s. Both
write "ai-ctl" on the fox's sign and the poster's two French lines exactly; FLUX adds two lines
of illegible small text to the poster; Qwen-Image's portrait has an over-sharpened, saturated skin,
FLUX's reads as a photograph. The eye decides.

**`sd-server` exits 0 without a word when it cannot bind its port.** Qwen-Image's first two starts
from `compare-image.sh` logged "listening on: http://127.0.0.1:18090", then ended before
answering, while seven starts by hand and a third from the script were ready in 4 to 6 s.
- `svr.listen` failing is not checked: `main.cpp` logs "listening on" before it, then stops its
  worker and returns 0 (`examples/server/main.cpp:157–166`). Started on a port that
  `python3 -m http.server` holds, `sd-server` exits 0 with that same log; `whisper-server` exits 1
  with "couldn't bind to server socket", `audiocpp_server` exits 1 with "could not bind".
- The cause: the cpp-httplib vendored by stable-diffusion.cpp, whisper.cpp and llama.cpp
  (`b11700`) sets `SO_REUSEPORT` in place of `SO_REUSEADDR` on Linux (`default_socket_options`), so
  the server cannot bind a port where another process's connection is in TIME-WAIT. Reproduced:
  `python3 -m http.server` on 18090 answers one curl and is stopped, `ss` shows
  `TIME-WAIT 127.0.0.1:18090`, and `sd-server` started then exits 0; started again once the
  TIME-WAIT has expired, it is ready. The first failed start came 48 s after the TTS run's
  `audiocpp_server` left 18090, the second about a minute after the probe's Python server, its
  start time not recorded. A server
  after its own kind binds, its TIME-WAIT sockets carrying `SO_REUSEPORT` too (the seven starts by
  hand). `audiocpp_server` sets `SO_REUSEADDR` (`app/server/http.cpp:754`).
- Behind llama-swap: v263 gave `echo` 5800, `flux-3-image-klein` 5801, `qwen3-tts` 5803 and
  `whisper-large-v3` 5804 in the scratch configuration, the order of the sorted ids
  (`$P/swap/llama-swap-4.log`), so a model keeps its port while the configuration does not change.
  A change of the inventory can hand a port to another server, which fails to start for up to 60
  s after a restart. Inferred from one log, to be probed in phase 2.
- Phase 1's patch task now makes `sd-server`'s failed `listen` exit 1 with a message; phase 2
  gained a task on the ports (Deviations).

**State at the session's end.** The findings above went to the user with the local page, as a
summary per use: speech-to-text by WER and time; text-to-speech all intelligible, Fish alone
exact on the date and hour, Voxtral the fastest at about 4 s for 20 s of audio; images by time
and text, with the observations by eye; the `sd-server` defect and the two adjustments. No
candidate is judged yet: task 7 waits for the user's verdict, model by model. Nothing runs, the
GPU holds 15 MiB. The outputs, the page and the scripts stay under `$P/`, outside the repository;
the probe binaries still need a throwaway `LLAMA_API_KEY`.

### 2026-10-23

Resumed from the entry above. `progress.py --check` found the roadmap's folder still under
`pending/` with phase 0 open, the opening of 2026-10-20 having left it there: moved to
`on-progress/`, with the README's `**Location:**` and the report's citation in
`tests/fixtures/README.md`.

The comparison page rebuilt for the scoring (`bin/compare-page.py`; the first kept as
`bin/compare-page-2026-10-21.py` and `compare-2026-10-21/index-2026-10-21.html`): whisper's texts
without `verbose_json`'s wraps; for each recording, the points where the readings differ, at the
time of turbo's token timestamps, a click playing the recording from there; for each TTS output,
its differences from the text sent. Checked against the outputs, the entry above was wrong three
times, corrected in place: whisper's error counts on `reference-fr`, which outputs render the date
and the hour, the poster's lines of small text.

The user's default browser, Floorp, is a Flatpak that reads only `~/Downloads`: it opened the page
but none of the files beside it. The page now holds its recordings (`compare-2026-10-21/inputs/`)
and is served on the loopback by `bin/serve.py`, `http://127.0.0.1:8765/`, with the byte ranges a
listening point needs to seek; over it, headless Firefox loaded the 19 recordings and outputs,
each seekable to its end, and the 8 images.

**The user's verdicts, file by file**, each reader ranked by ear against the recording:

| File | Ranking | The user's notes |
|---|---|---|
| `reference-fr`, 37 s | large-v3, turbo, Qwen3-ASR | The audio says "Ça n'est": the transcript's "Ce n'est" is wrong there, so large-v3 makes no error and turbo one, though "Ce n'est" is the more common written form. The audio links "qu'au" and "qu'aujourd'hui", which Qwen3-ASR writes "que, au" and "que, aujourd'hui"; its numbers in words read less well, the large ones most; "ce spécimen" is wholly wrong. large-v3 and turbo differ little in French, and both are far ahead of Qwen3-ASR. |
| `transcript-fr`, 60 s | none given | The gap is narrower, but Qwen3-ASR follows the audio word for word where turbo and large-v3 prefer the sense and the syntax. Qwen3-ASR alone is right on "très quali" (0:13) and "moins j'ai envie" (0:15), and alone builds the sentence of 0:08 well ("parce que pour deux raisons, déjà…"), which turbo writes word for word and large-v3 corrects lightly, its syntax still imperfect. Qwen3-ASR alone is wrong on "coups" for "coûts", a cost (0:21), on a "de" before "Deuxièmement" that the audio does not hold, a short hesitation there (0:29), and on "mais" for a faint "et" that turbo and large-v3 leave out, which changes the sense a little (0:46); it adds a trailing "ah" (0:58, where the recording is cut short: to be taken with care). It writes the hesitations as heard, "d'une" (0:33), "je, j'ai pas" (0:40), "des des" (0:57), breaking the syntax where turbo and large-v3 write the sentence the speaker meant. large-v3 adds an "à" the audio does not hold, "n'attendez pas à" (0:51). |
| `transcript-en`, 60 s | none given | To be taken with more care: the user's English may skew the notes. The same finding as on `transcript-fr`: Qwen3-ASR keeps the exact words where turbo and large-v3 seek the sense, dropping repetitions and hesitations. All three miss "Three.js Journey" ("3DS", "3GS"; 0:00), a trap, the speaker French with a French accent. Qwen3-ASR alone is wrong on "more an" for "more of an" (0:04), "award websites" for "Awwwards websites" (0:59), and a trailing "Yeah" where the recording is cut short (0:59.6), and keeps "on this on this journey" (0:16) and "this this" (0:21), which the others correct. The studio is "Immersive Garden", its "s" heard from the "so" that follows, and the speaker says "Immersive Garden Studio, a digital studio" (0:54–0:55), which Qwen3-ASR alone writes whole; turbo writes "Immersive Gardens" and leaves out the "a", large-v3 leaves out "Studio, a". |

**Mistral's Voxtral TTS paper, read before the TTS verdicts** (arXiv 2603.25551v2, 2026-04-06, at
the user's request). What bears on the comparison:
- Its headline, a 68.4% win rate over ElevenLabs Flash v2.5, is zero-shot voice cloning, which
  the open checkpoint cannot do: it lacks the codec's encoder (audio.cpp's
  `docs/models/voxtral_tts.md` at `v0.9.2`). The presets are the open model's whole use; against
  ElevenLabs Flash v2.5 the paper's flagship voices, `French-Female` among them, win 58.3%.
- French is its weakest language on the paper's own automated metrics (MiniMax-TTS: WER 3.22%,
  UTMOS 2.83, speaker similarity 0.587, against 0.63%, 4.30 and 0.786 in English) and near parity
  by ear in cloning (54.4%). The paper calls UTMOS a loose proxy across languages, and its WER is
  read by Mistral's own Voxtral Mini Transcribe v2.
- No open model is compared, only ElevenLabs v3, Flash v2.5 and Gemini 2.5 Flash TTS, which
  beats it (35.4% and 37.1% win rates for Voxtral): it settles nothing between Voxtral, Fish and
  Qwen3-TTS, as with FLUX.3's own evaluations.
- No emotion tag or instruction: emotion comes from the voice prompt or is inferred from the text.
- Inference: 8 Euler steps and acoustic CFG 1.2 in the paper, a higher CFG suggested for
  in-the-wild references; audio.cpp defaults to `num_inference_steps` 7 ("matching the released
  reference implementation") and `guidance_scale` 1.2, which `compare-tts.sh` left, with a random
  seed: each output is one draw. The paper's RTF on an H200 under vLLM-Omni, 0.103 with CUDA
  graphs and 0.258 without, frames this machine's 0.16 to 0.18 (Q8_0) and 0.22 to 0.26 (BF16)
  under audio.cpp, a whole request each.
- DPO cut hallucinated and skipped words, French most (WER 5.01% to 3.22%), and trained on
  rewrites between normalized and written text ("5 - 4", "five minus four"); nothing on Q8_0.

**State at the session's end.** A third session, the same day, took up the verdicts: the user's
notes on the three speech-to-text files in the table above, the three readers kept (Decisions),
and Voxtral's paper read before the TTS verdicts. The page's server ran for the session
(`$P/logs/serve-2026-10-23b.log`) and was stopped at its end: nothing runs, the GPU holds 15
MiB. The previous session's lines of this entry, the page's serving and the verdict on
`reference-fr`, were left uncommitted; the user restated that verdict at this session's start,
and they are committed with this one. The user's notes stay in `result.md` (untracked) at the
repository's root.

**A fourth session, the same day**, resumed from the third session's hand-over, replaced by the
one at this entry's end: the page served again (`$P/logs/serve-2026-10-23c.log`),
`http://127.0.0.1:8765/` answering 200. `result.md` was no longer at the repository's root; the
user's notes on speech-to-text are in the table above.

**The user's verdicts on the text-to-speech outputs**, Qwen3-TTS and Fish cloning the user's
voice, Voxtral on its presets:

| Output | The user's notes |
|---|---|
| Qwen3-TTS native, FR | Strikingly realistic at the start, a little more robotic over the last seconds, the date and the hour. |
| Qwen3-TTS `q8_0`, FR | The quality seems better, and it stays coherent to the end. |
| Qwen3-TTS native, EN | A French accent is clearly heard, but the voice is fairly natural; "2026" and "45 pm" are said in French. |
| Qwen3-TTS `q8_0`, EN | Of much better quality, but "2026" and "45 pm" are still said in French. |
| Fish S2 Pro Q8_0, FR | The voice well copied but of lower quality: a slow delivery, less natural ("seems asleep"), empty of emotion. |
| Fish S2 Pro BF16, FR | The same: a voice without emotion or dynamism. |
| Fish S2 Pro Q8_0, EN | A little better than its French, but below Qwen3-TTS. |
| Fish S2 Pro BF16, EN | A little better than its French, below Qwen3-TTS, still without dynamism or emotion. |
| Voxtral Q8_0, presets | Little dynamism, little emotion, less personality. |
| Voxtral BF16, presets | A little better than the Q8_0, the same remarks. |

**What the requests left out**, read in audio.cpp at `v0.9.2` after the verdicts.
`compare-tts.sh` sent `model`, `input` and `voice` only; the speech route also takes `language`
into the text input, `seed`, the sampling options and `instructions`
(`app/server/server_protocol.cpp:67–89`).
- `qwen3_tts`'s `language` defaults to `auto`, inferred "from text and voice settings"
  (`model_specs/qwen3_tts.json`). The English paragraph went with the French reference and no
  language, which may be why "2026" and "45 pm" are said in French; not probed yet. OpenWebUI
  sends its configured parameters with each request (`docs/audio/audio-cpp.md` §6), where a
  `language` could go.
- `qwen3_tts`'s `instruct` is for the VoiceDesign and CustomVoice checkpoints, not Base.
- `fish_audio` announces "inline prosody/emotion controls" (`model_specs/fish_audio.json`), but
  neither its spec nor `docs/models/fish_audio.md` gives their syntax at `v0.9.2`; the paragraph
  sent held none.
- Each output is one draw of a random seed, and all three families take a `seed`: the `q8_0`
  heard above the native weights it quantizes is one draw against one.

**Qwen3-TTS on fixed seeds, Fish with its tags** (`bin/probe-tts-2026-10-23.sh`, the user's
choices under Decisions; outputs, page and readings in `$P/compare-2026-10-21/tts-2026-10-23/`,
served at `http://127.0.0.1:8765/tts-2026-10-23/`). Each server alone on the GPU, cloning
`ref_fr`, a throwaway key; all 16 requests answer 200, `audio/wav`.
- Qwen3-TTS native and `q8_0`, seeds 11, 22 and 33: `compare-tts.sh`'s French paragraph without
  `language`, its English with `language` `en`, which `session.cpp:643–645` passes to the talker
  (the request's `language`, an `options.language` overriding it). A start's first request
  takes about 9 s for 17 s of French; after it, the French takes 3.6 to 4.4 s for 17 to 18 s and
  the English 3.5 to 4.6 s for 16 to 20 s. `compare-tts.sh`'s 14.5 s and 13.3 s for the French
  were each a start's first request, where Voxtral BF16's four outputs took 4.1 to 5.2 s for 18.9
  to 23.7 s, its first 4.9 s (`compare-2026-10-21/tts/times.tsv`): warm, Qwen3-TTS is as fast as
  Voxtral.
- Fish S2 Pro Q8_0, seeds 11 and 22: the same paragraphs with five tags from the model card of
  `fishaudio/s2-pro` at `1de9996b` (`[excited]`, `[emphasis]`, `[short pause]`, `[chuckle]`,
  `[delight]`; "free-form textual descriptions" in `[tag]` syntax), `text_chunk_mode`
  `tag_aware`: 9.9 to 10.9 s for 18.6 to 22.0 s.
- Read back by whisper large-v3-turbo with each output's language
  (`bin/readback-2026-10-23.sh`): no tag is spoken as a word, and Fish renders the date and the
  hour in its four outputs. Qwen3-TTS's French renders them in its six; `q8_0` seed 22 runs the
  question into the date ("naturelle le 21 octobre 2026 à 21h45 ?"). Its English, with
  `language` `en`, reads as English numbers but right in one output of six (native, seed 22):
  "October 20th, 2026, at 9.30 PM", "2022 at 9:05 p.m." (native), "9:35 pm" in the three
  `q8_0` outputs, one with "October 2021, 2026". Whisper forced to English may also translate
  or mishear an accent: the ear decides.

**State at the session's end.** A fourth session, the same day, took the user's verdicts on the
text-to-speech outputs (the table above): Voxtral is kept in BF16 alone, its Q8_0 deleted, and
Qwen3-TTS and Fish were probed again on the user's choices (Decisions). The probe's 16 outputs
wait for the user's ear. The speed put to the user for Voxtral was corrected in the session:
warm, Qwen3-TTS is as fast. The page's server ran for the session
(`$P/logs/serve-2026-10-23c.log`) and was stopped at its end: nothing runs, the GPU holds 15 MiB.
`$P/bin/` gained `probe-tts-2026-10-23.sh`, `readback-2026-10-23.sh` and `tts-probe-page.py`,
which writes the probe's page. `docs/audio/audio-cpp.md` still gives Voxtral's deleted Q8_0,
updated with the other model sections (point 3). Nothing is uncommitted in the repository once
this entry is committed.

### 2026-10-28

Resumed from the hand-over of 2026-10-23: the page served again
(`$P/logs/serve-2026-10-28.log`), `http://127.0.0.1:8765/` and `/tts-2026-10-23/` answering 200,
the GPU at 16 MiB, and the questionnaire put again.

**The user's verdicts on the probe of 2026-10-23**, Qwen3-TTS on fixed seeds, Fish S2 Pro Q8_0
with its tags:

| Output | The user's notes |
|---|---|
| Qwen3-TTS native, FR, `language` auto | All three say "vingt et un heure" for "vingt et une heure". Seed 11 fairly clean, a little robotic at the end; seed 22 livelier, above all on the date and the hour, where 11 turns robotic; seed 33 less natural, a little slower. |
| Qwen3-TTS `q8_0`, FR, `language` auto | All three say "vingt et un heure". Seed 11 very realistic, the best of the six; seed 22 slower, more measured, a little less natural; seed 33 drifts from the reference voice and loses expression. |
| Qwen3-TTS native, EN, `language` `en` | "2026" said in French in all three. Seed 11 breaks down badly after the numbers; seed 22 does not; seed 33 changes the voice much, but its English accent is far cleaner. |
| Qwen3-TTS `q8_0`, EN, `language` `en` | Seed 11 says "twenty un 2026"; seed 22 says "2026" and "pm" in French; seed 33 as the native, the numbers still in French. |
| Fish S2 Pro Q8_0, FR, tags | Seed 11 no more emotion; seed 22 flatter still. |
| Fish S2 Pro Q8_0, EN, tags | Seed 11 much cleaner, a real emotion felt; seed 22 a little below 11, well above Qwen3-TTS. |

What the verdicts show against the probe's readings:
- Whisper's readback wrote the hour "21h45" in the six French outputs, where the ear hears
  "vingt et un heure": a transcript in digits hides an error of agreement, so the readback
  cannot judge numbers read as words.
- `language` `en` does not make Qwen3-TTS read English numbers in English: "2026" stays French
  in six outputs of six, the reference being the French `ref_fr`. Whether an English reference
  changes it is not probed.
- No precision of Qwen3-TTS wins on three seeds: the best of each is native seed 22 and `q8_0`
  seed 11, and the seed changes the delivery more than the precision does.
- Fish's tags change its English, not its French.

The user's conclusion, under Decisions: "Je pense que le mieux est de garder les diffèrents outils
et de les intégrer, garder un bench et un classement pour mon usage perso, mais proposer la prise
en charge la plus large si ai-ctl est publié en prod. De plus il ne faut pas rester trop fermé,
les models évolue vite!" Phase 2 gained a constraint from it (Deviations).

**The user's answers** to the three questions: Qwen3-TTS at both precisions, Fish S2 Pro at both
("le choix se fera à l'usage"), and a bench of audio and image left to a later roadmap, the
user's verdicts kept in the model sections of the domain documents, "c'est déjà un test
pertinent" (Decisions).

**The user's verdicts on the images**, Qwen-Image 2.1 and FLUX.3 Image Klein on the four prompts
of `bin/compare-image.sh` (2026-10-21):

| Prompt | The user's notes |
|---|---|
| `fox`: a fox in a snowy birch forest, a sign reading "ai-ctl", photograph | Qwen-Image's fox very realistic, its sign less so; FLUX's fox less realistic, its scenery and its sign more. |
| `lighthouse`: a Breton coast in a storm at dusk, oil painting | Both very beautiful and not comparable: Qwen-Image's reads as a watercolour, FLUX's as brushwork, each with its own style. |
| `portrait`: an elderly fisherman holding a cup of coffee, 85mm photograph | FLUX wins by far, very realistic in the hands, the face and the cup. Qwen-Image pales beside it: a face wrinkled to excess, fingers stuck together and blurred. |
| `poster`: a 1970s cinema poster, "La Nuit des GPU" | Two different styles again, Qwen-Image's a montage, FLUX's a classic AI generation: not to be compared. |

"Pour moi aucun modèle ne l'emporte sur l'autre les deux son complémentaires selon le besoin":
both are kept. Every use is judged and task 7 is ticked. Speech-to-text keeps whisper large-v3,
large-v3-turbo and Qwen3-ASR; text-to-speech Qwen3-TTS at both precisions, Fish S2 Pro at both
and Voxtral in BF16; image generation Qwen-Image 2.1 and FLUX.3 Image Klein. No candidate failed,
and none is dropped.

**The domain documents updated** with the models kept, the user's verdicts, dated, and this
phase's measures:
- `docs/audio/whisper-cpp.md`: `verbose_json`'s wrap (§2); the three readers compared on the
  user's recordings, with their times, the user's verdicts and the use each keeps (§4).
- `docs/audio/audio-cpp.md`: the speech route's other fields (§3); Voxtral in BF16 alone, its
  `hf download` without the Q8_0, whose measure stays marked as not kept; Qwen3-TTS's two
  precisions and its `language`; Fish's two precisions and its tags; the comparison on the
  user's voice, Qwen3-TTS's warm times and the user's verdicts (§5).
- `docs/image/stable-diffusion-cpp.md`: `sd-server`'s silent exit on a port it cannot bind
  (§2); the four prompts, their times and the user's verdicts (§4).
- `docs/gpu/llama-swap.md` unchanged: its peaks, 4.6 to 10.9 GiB for audio.cpp, still hold
  without Voxtral's Q8_0.

**Phase closed.** The six checks of the contract green at the closure's start: `cargo fmt
--check`, `cargo clippy --all-targets -- -D warnings`, `cargo test` (752 passed, 0 failed), the
two evals of `reviewing-delivered-code` (15 and 1 tests) and the check of cited sources. Phases
2 and 3 adjusted to the precisions kept (Deviations). The page's server stopped: nothing runs,
the GPU holds 15 MiB.

---

## Decisions

- **The criteria for the models (the user, 2026-10-20):** speech-to-text and text-to-speech in
  French and English alike; text-to-speech with preset voices and voice cloning both; image
  generation chosen for quality, a heavy model alone on the GPU; licences for personal use are
  acceptable, non-commercial ones included.
- **Two candidates per use, compared on this machine before one is kept (the user,
  2026-10-20):** whisper `large-v3` and `large-v3-turbo` on a French and an English recording;
  Qwen3-TTS 1.7B Base and Fish Audio S2 Pro on the same French and English sentences and the same
  reference voice, chosen by ear; Qwen-Image 2.1 and FLUX.3 Image Klein on the same prompts,
  chosen by eye. Preset voices are reference recordings declared in audio.cpp's configuration,
  so one cloning model serves both. Rankings found on 2026-10-20 settle nothing: Qwen-Image 2.1
  led the open weights on Artificial Analysis on 2026-10-07, FLUX.3 Image Klein (2026-10-15)
  claims first place in its maker's own evaluations only, and the TTS comparisons found are
  blogs.
- **The provenance rule for models other than LLM GGUF files (the user, 2026-10-20):** a file
  comes from the model's publisher, in a format its engine reads; failing that, from the engine
  project's own conversions (`ggerganov/whisper.cpp`, `audio-cpp/audio.cpp-gguf`, `leejet/…`).
  A repackaging by Comfy-Org is admitted only for a file the publisher does not offer in a format
  the engine reads, named file by file: the Qwen-Image 2.1 VAE, if a probe shows sd.cpp cannot
  load Qwen's own (see the Work Log's closing entry of 2026-10-20). An LLM used as a text
  encoder comes as GGUF from its publisher or from Unsloth, as `docs/stack.md` §5 says. Every
  file: the commit downloaded pinned and recorded, its SHA-256 equal to the Hub's LFS hash at
  that commit, and for whisper also the SHA-1 that whisper.cpp's `models/README.md` gives at
  the built commit. No template check: sd.cpp reads no `chat_template` from a GGUF (no match in
  `src/` or `examples/` at `master-975-95b3090`) and builds its prompts itself. Set aside: no
  third party at all, then thought to drop Qwen-Image 2.1 for want of a VAE sd.cpp reads, Qwen
  publishing the model in diffusers form; Comfy-Org as a general source.
- **FLUX.3 Image Klein's gated repository:** the user accepts its licence on Hugging Face and
  provides a read token for `hf download`.
- **One directory per candidate (the user, 2026-10-20):** `stt/whisper-large-v3/`,
  `stt/whisper-large-v3-turbo/`, `tts/qwen3-tts-1.7b-base/`, `tts/fish-audio-s2-pro/`,
  `image/qwen-image-2.1/`, `image/flux-3-image-klein/`; an image model's text encoder and VAE in
  its own directory. The candidate a comparison drops goes with its directory.
- **Qwen3-TTS from Qwen's own files (the user, 2026-10-20):** the provenance rule applies as
  written, since audio.cpp reads the safetensors Qwen publishes; audio-cpp's Q8_0 GGUF is not
  admitted. A quantization, if the measures call for one, is audio.cpp's `weight_type=q8_0` at
  load.
- **Both precisions measured where a choice of precision is open (the user, 2026-10-20):** Fish S2
  Pro in Q8_0 and BF16, Qwen3-TTS in native precision and with `weight_type=q8_0`, compared by
  ear and on VRAM and time by hand in this phase; quality first if the measures do not settle it.
  ai-ctl's bench stays with the LLM, as the roadmap's scope says.
- **Each server guarded by a patch of its own (the user, 2026-10-20):** none of the three checks a
  key, upstream included. Each recipe applies a patch that checks `Authorization: Bearer` against
  a variable llama-swap's `env:` hands the server, exempts the path of its readiness check (which
  llama-swap sends without a key), and drops the permissive CORS. The guarantee is
  `docs/security.md`'s for llama-server. Commits phase 1 to a builder that applies a recipe's
  patches and fails a tag they no longer apply to. Set aside: serving them without a key as a
  written exception, which opens them to any page and any local account; waiting for upstream or
  for llama-swap to reach a unix socket; a network namespace, not probed.
- **The provenance rule goes in `docs/stack.md` §5 (the user, 2026-10-20)**, beside the GGUF rule
  under "Where Models Come From", `docs/security.md` §7 pointing to it.
- **Whisper chosen on the user's recordings (the user, 2026-10-20):** large-v3 and large-v3-turbo
  compared on a French and an English recording the user provides.
- **audio.cpp stays, whole (the user, 2026-10-20):** "je propose de l'intégrer au complet pour
  faire des bench et comparaison on regardera ce qui est le meilleur, pour la synthèse en
  français, voxtral 4b est très bon aussi". Voxtral 4B TTS joins the TTS candidates through
  audio.cpp's `voxtral_tts` family, and Qwen3-ASR joins the speech-to-text candidates beside
  whisper. The user ran Voxtral through vLLM-Omni in `/code/python/polarsteps-tts`, with its preset
  voices `fr_male` and `fr_female`.
- **Several models per use, served side by side (the user, 2026-10-20):** "un modèle par usage
  est une fausse affirmation, le but est de pouvoir choisir le backend / model le plus adapté au
  besoin (pr llm je pense à llama.cpp et vllm qui n'a pas encore était testé) le but est d'être
  ouvert au choix de permettre d'être dynamique et d'introduire des model et backend pour le
  test". The README's decision "One model per use" is replaced: every candidate kept is an entry
  of the inventory, served by llama-swap, and compared in use; the comparisons of this phase
  measure them and drop only a candidate that fails. A restructuring the user asked for; its
  changes to the README and to phases 1 to 3 are written at this phase's closure, with task 12.
  vLLM as a second LLM engine stays outside this roadmap, as its README says.
- **The models belong to ai-ctl (the user, 2026-10-20):** "les prochains projets utiliseront
  ai-ctl". `/ai/models/tts/Voxtral-4B-TTS-2603/` (Mistral's files for vLLM-Omni, from
  polarsteps-tts, 7.5 GB, a locally edited `params.json`) was deleted at the user's request,
  after checking that no process used it.
- **Directories of the new candidates (the user, 2026-10-20):** `tts/voxtral-4b-tts/` and
  `stt/qwen3-asr-1.7b/`.
- **The later phases adjusted now (the user, 2026-10-20, "oui validé"):** the restructuring the
  entry "Several models per use" announced for this phase's closure is applied before it, with the
  adjustments the probes call for in phase 1; task 12 keeps the domain documents.
- **The three speech-to-text readers kept (the user, 2026-10-23):** "je vois 3 usage diffèrent,
  turbo -> rapide efficace, large -> rapidité medium (peut être plus précis que turbo), qwen3 ->
  lent mais rend un transcript le plus proche de l'audio (garde les hésitation, les répétition)".
  None fails on the user's three recordings (the verdicts in the Work Log, 2026-10-23), so the
  decision "Several models per use" keeps them all; no use needs them yet. Set aside: dropping
  Qwen3-ASR for its errors and its time, proposed in the session against that decision.
- **Voxtral 4B TTS kept in BF16 alone (the user, 2026-10-23):** the better of its two precisions
  by ear, exact on the date and the hour where the Q8_0's `fr_female` read "21 arches 45", for
  synthesis on its presets. The Q8_0 GGUF and its download metadata were deleted from
  `tts/voxtral-4b-tts/` once no process held it. Set aside: both precisions, the Q8_0 3 GB
  lighter; dropping Voxtral, whose presets a reference recording under Qwen3-TTS could stand in
  for. The choice was put with Voxtral as the fastest, about 4 s for 20 s of audio against
  about 13 s for Qwen3-TTS; that compared Qwen3-TTS's first request after a start. Past it,
  Qwen3-TTS takes 0.20 to 0.25 s per second of audio and Voxtral BF16 0.21 to 0.26 s (Work Log,
  2026-10-23, fourth session), told to the user.
- **Qwen3-TTS and Fish S2 Pro probed again before the verdict (the user, 2026-10-23):**
  Qwen3-TTS, ahead by ear, is kept, its precision chosen on three fixed seeds per precision, and
  its English sent with `language` `en`; Fish, below Qwen3-TTS by ear at both precisions, is
  heard once more with the inline tags its model card describes, before it is kept or dropped
  (Work Log, 2026-10-23, fourth session).
- **Fish S2 Pro kept, at both precisions (the user, 2026-10-28):** with its inline tags, its
  English is "largement au dessus de qwen"; its French stays flat (Work Log, 2026-10-28). Q8_0
  and BF16 are two entries over two files of one directory, "le choix se fera à l'usage", the
  Q8_0 alone heard with the tags. Set aside: hearing the BF16 with the tags before a choice.
- **Qwen3-TTS kept, at both precisions (the user, 2026-10-28):** native and `weight_type` `q8_0`,
  two entries over the same files, the best output of each heard on a different seed (Work Log,
  2026-10-28).
- **Every tool integrated, the widest support shipped, open to new models (the user,
  2026-10-28):** "garder les diffèrents outils et de les intégrer, garder un bench et un
  classement pour mon usage perso, mais proposer la prise en charge la plus large si ai-ctl est
  publié en prod. De plus il ne faut pas rester trop fermé, les models évolue vite!" Every server
  and every model that passes becomes an entry; the ranking by use is the user's own; the
  shipped inventory offers every model kept, as phase 3 says; and a kind accepts any model its
  server loads, a constraint added to phase 2 (Deviations).
- **A bench of audio and image in a later roadmap; the user's verdicts in the domain documents
  (the user, 2026-10-28):** "Ok mais tu peux tout de même garder les retours que je t'ai fait
  comme tu l'as proposé, c'est déjà un test pertinent". This roadmap's scope is unchanged, its
  bench scenarios for audio and image out of it; the verdicts by ear and by eye go, dated, into
  the model sections of `docs/audio/` and `docs/image/`. Commits this roadmap's summary to name
  the bench as left open. Set aside: adding the bench to this roadmap.
- **Both image models kept (the user, 2026-10-28):** "aucun modèle ne l'emporte sur l'autre les
  deux son complémentaires selon le besoin". FLUX.3 Image Klein far ahead on a photograph of a
  person, by the portrait; on the other prompts, each its own style (Work Log, 2026-10-28).

---


## Deviations

- **The builder assumes llama.cpp beyond its recipe.** Found by the hand builds: it initialises
  no submodule, which stable-diffusion.cpp needs for `ggml`; its smoke test runs `--version`,
  which `whisper-server` lacks while still exiting 0, so the test would pass on any binary that
  starts; `recipe::cuda_devices` reads llama-server's `CUDA0:` lines only, not sd-server's
  `CUDA0<TAB>…` nor audiocpp_server's `CUDA:0 "…" [GPU]`, so both would fail it; and
  stable-diffusion.cpp's default `SD_SERVER_BUILD_FRONTEND=ON` would run `pnpm install` from the
  build. Consequence: phase 1's task "Move into the recipe what the builder still assumes of
  llama.cpp" covered them; it was replaced by three tasks on 2026-10-20 (below).
- **A normal audio.cpp build cannot load a safetensors package on its own.** Found running
  Qwen3-TTS from Qwen's files: without `AUDIOCPP_DEPLOYMENT_BUILD=ON`, the server looks for
  `model_specs/<family>.json` from its working directory upward and exits 1 without it.
  Consequence: audio.cpp's recipe needs `AUDIOCPP_DEPLOYMENT_BUILD=ON`, which compiles the built
  commit's specs into the binary; phase 1's audio.cpp recipe task says so since 2026-10-20.
- **None of the three servers can require a key.** Found reading each one's sources at its built
  commit and at its head, and checked on its port: whisper-server allows every origin and loads
  any host path on `/load`, sd-server echoes any `Origin` with credentials, audiocpp_server runs a
  `text/plain` POST a page sends without preflight. Consequence: the user chose a patch per
  server, prototyped in this phase; phase 1 ships them in the recipes.
- **Task 7 and an acceptance criterion of this phase reworded.** The user rejected "one model per
  use": the task now compares the candidates of each use and keeps those that pass, and the
  criterion speaks of the models kept for each use. Two candidates joined: Voxtral 4B TTS and
  Qwen3-ASR 1.7B.
- **`sd-server` cannot bind a port another server just used, and exits 0 without a word.** Found
  when Qwen-Image failed twice to start during task 7: its cpp-httplib binds with `SO_REUSEPORT`
  alone, as llama-server's and whisper-server's do, and its `main.cpp` ignores a failed `listen`.
  Consequence: phase 1's `sd-server` patch ends a failed `listen` with exit 1; phase 2 probes
  llama-swap's ports and keeps a port from passing to another server across a restart (below).
- **The opening left the roadmap's folder under `pending/`.** Found by the progress check on
  2026-10-23. Consequence: moved to `on-progress/` that day (Work Log).
- **Five links of the completed roadmaps broken since `docs: organize the documentation by domain, notes out of the repository`** (2026-10-19), which made
  decision 0001 `docs/gpu/llama-swap.md` and took the notes out of the repository. Found by the
  closure's link check. Fixed: the READMEs of `v2-rebuild` and `v2-review-fixes` link
  `docs/gpu/llama-swap.md`, and `v2-rebuild`'s `phase-4-bench.md` no longer cites the notes,
  which no tracked file cites.

Changes to later phases, applied on 2026-10-20, with the user's approval of the restructuring (Decisions):

- `README.md`: the decision "One model per use" replaced by "Several models per use", dated, and
  the decision on the key completed with the patch per server; `**Last Updated:**` 2026-10-20;
  the progress block recomputed (43 tasks).
- `phase-1-native-builds.md`: the task "Move into the recipe what the builder still assumes of
  llama.cpp" split into three — the submodules a recipe names; each server's smoke test and
  device listing, read from phase 0's captures; a recipe's patches, applied after the checkout,
  refused on a tag they no longer apply to, recorded in `build.json`, with the three key patches.
  The three recipe tasks name what phase 0 found (stable-diffusion.cpp's `ggml` submodule only and
  its frontend, WebP and WebM off; audio.cpp's `AUDIOCPP_DEPLOYMENT_BUILD=ON` and no SSH
  submodule). The documentation task adds `docs/security.md` §7 on the patches; a constraint and
  the dependencies name the patches. 10 tasks become 12.
- `phase-2-inventory-and-gateway.md`: the command task adds `--eager-load` for sd-server; the
  readiness and key task says `/health` and `LLAMA_API_KEY` through `env:`; the task "The guard
  phase 0's decision gives a server that cannot require a key" replaced by audio.cpp's
  `--config` file, which ai-ctl must provide; the matrix snapshot and the acceptance criterion
  speak of several models of each kind; the dependencies name the patched builds.
- `phase-3-end-to-end.md`: the shipped inventory holds every model kept, several per use; the live
  run sends one request to each model; the swaps add a second image model; OpenWebUI's task says
  how to switch models, typed by hand since its lists are requested without the key; the
  acceptance criterion covers each model of the shipped inventory.

Adjusted on 2026-10-21, an adjustment that needs no approval:

- `phase-2-inventory-and-gateway.md`: the command task gives `whisper-server`
  `--inference-path /v1/audio/transcriptions` and `-l auto`, its default language `en` decoding a
  French request without `language` as English (Work Log, 2026-10-21).
- `phase-1-native-builds.md`: the patches task adds that `sd-server`'s patch ends a failed
  `listen` with a message and exit 1 (Work Log, 2026-10-21, second session).
- `phase-2-inventory-and-gateway.md`: a task added under Gateway Configuration, each model's port
  across a restart: probe how llama-swap v263 assigns `${PORT}` and keep a change of the
  inventory from handing a port to another server within TIME-WAIT's 60 s. 10 tasks become 11;
  the README's progress block recomputed (44 tasks) and its `**Last Updated:**` set to
  2026-10-21.

Adjusted on 2026-10-28, an adjustment that needs no approval:

- `phase-2-inventory-and-gateway.md`: a constraint added, a kind accepting any model its server
  loads, with no list of models or of audio.cpp's families in ai-ctl (Decisions, 2026-10-28).
- `phase-2-inventory-and-gateway.md`: the audio.cpp configuration task adds each model's
  `session_options`, Qwen3-TTS's `weight_type` making one set of files two entries.
- `phase-3-end-to-end.md`: the shipped inventory task adds each precision kept as an entry of
  its own (Decisions, 2026-10-28).

---

## Assessment

The phase settled on this machine what phases 1 to 3 build on. `whisper-server` (`v1.9.5`),
`audiocpp_server` (`v0.9.2`) and `sd-server` (`master-975-95b3090`) were built by hand for
`sm_120a`, run alone and behind llama-swap v263, and each answered through it. Their options,
routes, readiness and key are in `docs/audio/whisper-cpp.md`, `docs/audio/audio-cpp.md` and
`docs/image/stable-diffusion-cpp.md`, their captures in `tests/fixtures/README.md`. The provenance
rule for models other than LLM GGUF files is in `docs/stack.md` §5.

No server can require a key, upstream included: each gets a patch of ai-ctl's own, prototyped and
checked on its port and behind the gateway. The user rejected "one model per use", and the
comparisons on the user's own material dropped no model: eight models stay, ten entries with the
precisions — whisper large-v3, large-v3-turbo and Qwen3-ASR; Qwen3-TTS native and `q8_0`, Fish
S2 Pro Q8_0 and BF16, Voxtral BF16; Qwen-Image 2.1 and FLUX.3 Image Klein — each pinned,
verified and measured.

Left open: Qwen3-TTS with an English reference, Fish's BF16 with its tags, Qwen3-ASR for
OpenWebUI (audio.cpp takes WAV only), and a bench of audio and image, for a later roadmap.

What phase 1 needs to know first: the builder assumes llama.cpp beyond its recipe (submodules,
smoke test, device listing, stable-diffusion.cpp's frontend), and its recipe and builder tasks
name each case. The key patches are in `$P/patches/` (`$P` is `~/.cache/ai-ctl-probes`, untracked), one per
server against its built tag; `sd-server`'s does not yet end a failed `listen` with exit 1. The
audio.cpp recipe needs `AUDIOCPP_DEPLOYMENT_BUILD=ON` and must skip the SSH submodule. A patched
binary refuses to start without `LLAMA_API_KEY`.
