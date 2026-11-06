# Phase 3 Report: End To End

**Phase:** [phase-3-end-to-end.md](phase-3-end-to-end.md)
**Start Commit:** `docs(audio-image-backends): close phase 2` of 2026-10-29

---

## Work Log

### 2026-10-29

Phase opened. Before task 1, the voice library the cloning models need:
- The user added three French reference recordings under `/ai/models/voices/<name>/`, one
  `reference-fr.*` each: one MP3 of 37 s with its transcript, two Opus of 39 s and 95 s
  without one.
- audio.cpp at `v0.9.3` reads a voice as `<voice_dir>/<voice>.wav`, any WAV
  (`read_wav_f32`), and its transcript as the line `<voice>|<transcript>` of
  `<voice_dir>/prompt_text`. It lists the `.wav` files at the top of `voice_dir` only, so the
  source folders do not show as voices. Qwen3-TTS and Fish S2 Pro resample the reference
  themselves.
- Each source converted with ffmpeg to 16-bit mono WAV at its own rate, in the session's
  scratch folder, and transcribed through a gateway started by ai-ctl with
  `whisper-large-v3` alone, `language` `fr`: 200 in 3.5 s, 1.2 s and 2.2 s; 401 without the
  key. The MP3's transcript and whisper's differ in three elisions. `down` released 3876 MiB,
  no process left.
- On the user's approval (Decisions), `/ai/models/voices/` now holds `<name>-fr.wav` for each
  recording and `prompt_text`, one line each: the MP3's own transcript, whisper's for the two
  others. The 95 s recording was cut at 40.6 s, in a pause after a sentence, then transcribed
  again: the cut's text holds two words the whole file's transcript had dropped.

Task 1, the shipped inventory:
- `the_shipped_inventory_is_valid_once_rendered` was given the 13 models and 6 backends first,
  and watched failing on its assertion: two models listed.
- `deploy/inventory.toml` gains three backends, `whisper-native`, `audio-native` and
  `sd-native`, and 11 models: three speech-to-text, six text-to-speech, three image. The ids
  are those phase 2's tests and documents used, plus `fish-s2-pro-bf16`. Each `vram_mib` is
  the model's measured peak, each `ttl` 1800 s as for the LLMs, `voices = "voices"` for the four
  cloning models. Fibo's entry is phase 2's, plus Bria's negative prompt (Decisions). The
  image models set no `repo`: each takes files from several repositories, and `doctor`'s
  remedy would name one for all.
- `gateway-shipped.yaml` and `gateway-shipped-image.yaml` regenerated; read once:
  - each kind's command as phase 2's tests have it; Fibo's negative prompt one single-quoted
    word;
  - the matrix has 37 sets. The three image models each run beside `whisper-large-v3-turbo`,
    and FLUX beside `whisper-large-v3` too: their budgets are a 1024×1024 image's peak.
- `an_inventory_with_several_models_of_each_kind_gives_the_snapshot` and
  `gateway-every-kind.yaml` removed: its inventory was the shipped one plus the models now
  shipped, which the shipped snapshot covers; the `[defaults.audiocpp_server]` it added is
  covered by `an_audiocpp_server_model_reads_the_configuration_ai_ctl_writes`.
- Live, the shipped inventory installed by `ai-ctl init --models-dir /ai/models` in the
  session's scratch folder: `doctor` 37 ok, every model's files and the voices found;
  `gateway config`, then llama-swap v263's `-validate`: "config is valid: 13 model(s)". No
  key in the file.
- fmt, clippy and the tests green (795, 27 and 3). `cargo mutants --in-diff`: no mutant, the
  diff holding a test, the inventory and a snapshot.

Committed as `feat(inventory): ship the speech and image models`.

The image budgets, after the user chose the largest size each model makes (Decisions):
- `~/.config/ai-ctl/inventory.toml`, the shipped inventory of before phase 2 with no setting of
  its own, moved to `inventory.toml.old` at the user's word; `ai-ctl init --models-dir
  /ai/models` installed the new one, and `doctor` gave 37 ok.
- FLUX.3 Image Klein and Qwen-Image 2.1 probed by hand at four sizes, one `sd-server` per size,
  4 steps, with `probe-ready.sh` and `req-image.sh` (`~/.cache/ai-ctl-probes/bin/`, untracked;
  the script and outputs in the session's scratch folder). Their peaks, in
  `docs/image/stable-diffusion-cpp.md` §4: 1536×1536 passes for both, at 22754 and 23804 MiB,
  and 2048×2048 aborts both, as it does Fibo. Their 1024×1024 peaks, 18594 and 20404 MiB, agree
  with phase 0's.
- The three image budgets are now their 1536×1536 peaks, 22754, 23804 and 23600 MiB. The
  snapshot test was watched failing on the first, then regenerated: 36 sets, each image model
  alone in its own.

Committed as `fix(inventory): budget each image model at its largest size`.

Task 2, live, with `ai-ctl up` from the working tree on `127.0.0.1:8080` and the installed
inventory. `requests.sh` (the session's scratch folder) sends each model one request without the
key, then one with it, and reads `/running` after it: a chat, JFK's 11 s, a French sentence with
`fr_female` or `micode-fr`, or a 1024×1024 image.
- First run: every request without the key got llama-swap's 401, every one with it 200, but
  `fish-s2-pro-q8`: 500 "failed to allocate Fish DAC codec encode graph", loaded beside
  `qwen3-tts` and `voxtral-4b-tts`, a set its budgets allowed (Deviations).
- Each cloning model's peak measured again behind the gateway, alone, with the library's
  longest reference (40.6 s), and Qwen3-ASR's on 60 and 180 s of French: the figures are in
  `docs/audio/audio-cpp.md` §5. Fish S2 Pro Q8_0 peaks at 10310 MiB against phase 0's 7953,
  BF16 at 12768 against 11199, Qwen3-TTS at 6914 and 5542 against 6025 and 4759, Qwen3-ASR at
  5548 against 4903; Voxtral and whisper stay at their budgets.
- The five budgets corrected; the snapshot test watched failing on the first, then regenerated:
  32 sets. Fish Q8_0 no longer runs beside Qwen3-TTS and Voxtral together.
- The installed inventory was still the one of `feat(inventory): ship the speech and image models`, with none of the new budgets, the images'
  included: `init` installs a copy. Untouched since its install, it was moved to the session's
  scratch folder and installed again.
- Second run, every model:

  | Model | Without the key | With it | Answer | Loaded beside it after |
  |---|---|---|---|---|
  | `qwen3.8-27b` | 401 | 200 in 0.35 s, preloaded | "Bonjour" | — |
  | `qwen3.6-35b-a3b` | 401 | 200 in 8.84 s | "Bonjour" | — |
  | `whisper-large-v3-turbo` | 401 | 200 in 1.98 s | JFK word for word | — |
  | `whisper-large-v3` | 401 | 200 in 1.60 s | JFK word for word | turbo |
  | `qwen3-asr-1.7b` | 401 | 200 in 3.87 s | JFK word for word | the two whisper |
  | `voxtral-4b-tts` | 401 | 200 in 6.35 s | 6.6 s at 24 kHz | the three readers |
  | `qwen3-tts` | 401 | 200 in 14.15 s | 6.4 s at 24 kHz | Voxtral, the two whisper |
  | `qwen3-tts-q8` | 401 | 200 in 16.04 s | 5.7 s at 24 kHz | Qwen3-TTS, Voxtral |
  | `fish-s2-pro-q8` | 401 | 200 in 11.52 s | 6.0 s at 44.1 kHz | the two Qwen3-TTS |
  | `fish-s2-pro-bf16` | 401 | 200 in 19.37 s | 6.5 s at 44.1 kHz | Qwen3-TTS |
  | `flux-3-image-klein` | 401 | 200 in 16.91 s | a PNG | alone |
  | `qwen-image-2.1` | 401 | 200 in 33.93 s | a PNG | alone |
  | `fibo-2.0-dev` | 401 | 200 in 128.44 s | a PNG | alone |

  Each time includes the model's start. The 401s took under 2 ms: llama-swap refuses before it
  starts a server.

Committed as `fix(inventory): budget the speech models with a 40 s reference`.

Task 3, the swaps, with `requests.sh` and the gateway's `matrix:` lines:

| Request | Answer | Set | Evicted | Running after |
|---|---|---|---|---|
| a chat on `qwen3.8-27b` | 200 in 11.56 s | `fit-1` | `fibo-2.0-dev` | the 27B, 21316 MiB |
| an image on `flux-3-image-klein` | 200 in 17.05 s | `fit-30` | the 27B | FLUX, 15778 MiB |
| an image on `qwen-image-2.1` | 200 in 33.81 s | `fit-31` | FLUX | Qwen-Image, 16660 MiB |
| a chat on `qwen3.8-27b` | 200 in 5.99 s | `fit-1` | Qwen-Image | the 27B, 21316 MiB |

Task 4, each server's own port, with `ports.sh` (the session's scratch folder): the port read
from `/running`, then each route of the kind without the key, then one `GET` with a wrong key
and with ai-ctl's, the key on curl's standard input. As phase 0 decided, every route but the
health check answers 401 without the key, for every kind:

| Server, model | Routes answering 401 without the key | Open |
|---|---|---|
| `llama-server`, the 27B | `/v1/models`, `/props`, `/slots`, `/metrics`, `/v1/chat/completions`, `/completion`, `/tokenize` | `/health`, `/v1/health` 200; `/` 415, the web UI's files |
| `whisper-server`, turbo | `/`, `/v1/audio/transcriptions`, `/inference`, `/load` | `/health` |
| `audiocpp_server`, Voxtral | `/`, `/v1/models`, `/v1/audio/voices`, `/v1/audio/speech`, `/v1/audio/transcriptions`, `/v1/tasks/run`, `/v1/tasks/unload_all_models`, `/v1/ui/upload`, `/v1/ui/path-status` | `/health` |
| `sd-server`, FLUX | `/`, `/sdapi/v1/sd-models`, `/sdapi/v1/txt2img`, `/v1/images/generations`, `/sdcpp/v1/model-files`, `/sdcpp/v1/recipes`, `/sdcpp/v1/models/status`, `/sdcpp/v1/models/unload` | `/health` |

A wrong key got 401, ai-ctl's 200, on each.

Committed as `docs(audio-image-backends): record the swaps and the servers' ports`.

Task 5, Fibo 2.0 Dev's long requests:
- Through the gateway, one image at 1536×1536, 50 steps: 200 in 300.6 s, a 1536×1536 PNG, a
  peak of 23600 MiB, its budget. Its server then still ran, `ready` in `/running`. llama-swap
  sets no timeout on a request.
- OpenWebUI 0.12.3, the image installed (`7fb550a5`, the version `docs/audio/whisper-cpp.md` §5
  read): its requests go through one aiohttp session whose total timeout is
  `AIOHTTP_CLIENT_TIMEOUT`, none when the variable is unset (`open_webui/env.py:614–618`,
  `utils/session_pool.py:43`), as in the container. The container started with
  `podman start openwebui`, on `127.0.0.1:18000`.

The voice library changed at the user's word:
- The two voice-message recordings removed, their WAV and their lines; their sources had been
  removed by the user. Their copies in the session's scratch folder deleted too.
- Six French MP3s added by the user, 32 to 67 s, each a `<name>-fr.wav` and a line of
  `prompt_text`, transcribed by whisper large-v3 through the gateway. The three longer than
  40.6 s, the reference the budgets were measured with, were cut in the gap after a sentence,
  at 32.75 to 36.9 s, with a 30 ms fade, then transcribed again: each cut's text ends on that
  sentence.
- `GET /v1/audio/voices?model=qwen3-tts` lists the seven voices without a restart, and
  Qwen3-TTS spoke a sentence with two of them, one whose name holds an `é`: 200 each.
- The two voice messages, re-added by the user since no copy was left (the scratch copies had
  gone with the voices), converted again, then their Opus sources deleted at the user's word.
  The 95 s one is cut at 31.18 s this time: at 40.6 s, whisper large-v3 and Qwen3-ASR disagreed
  on its last words, "voilà" and "et" heard by Qwen3-ASR alone. At 31.18 s, after a sentence,
  both end on the same words. The other keeps whisper's transcript, whose doubtful passages
  Qwen3-ASR reads otherwise. Qwen3-TTS spoke a sentence with each, 200. The library holds nine
  voices, five of them heard through the gateway so far.

OpenWebUI's settings were given to the user (engines `openai`, base `http://10.0.2.2:8080/v1`,
the key its chat connection holds, `whisper-large-v3-turbo`, `qwen3-tts` with a voice of the
library, `flux-3-image-klein` at `1536x1536`, then `fibo-2.0-dev`); the user has not used them
yet.

Larger images, at the user's request ("on peut ajouter maintenant!", asking for the largest
size supported), added as a Live task (`docs(audio-image-backends): add a task for larger images`):
- sd-server refuses a width or height above 4096 (`k_max_generation_dimension`,
  `examples/server/runtime.h:95` at `77b72a2`), so 4096×4096 is its ceiling. It also serves
  `POST /sdcpp/v1/upscale`, an ESRGAN upscale up to 8192×8192 with a model under
  `--hires-upscalers-dir` (`routes_sdcpp.cpp:509`), not probed, and a highres fix per request,
  which samples again at the target size.
- `probe-sizes-vae-cpu.sh` (`~/.cache/ai-ctl-probes/bin/`, untracked): one `sd-server` per size,
  `--backend vae=cpu` (Fibo `te=cpu,vae=cpu`), 4 steps, each server in a user scope capped at
  32 GiB of RAM, sizes 1536², 2048², 2560², 3840×2160, 3072², 4096² until one fails. The
  gateway was stopped for it.
- FLUX.3 Image Klein, 1536×1536: 200 in 168.1 s, of which the VAE's decode on the CPU took
  161.65 s (about 1 s on the GPU); VRAM peak 17038 MiB against 22754 with the VAE on the GPU;
  RSS peak 16.4 GiB.
- FLUX.3 Image Klein, 2048×2048: the sampling passed on the GPU, 4 steps in 12.71 s, VRAM peak
  18526 MiB; the probe was stopped at the session's close while the CPU decoded. So the
  diffusion fits at 2048×2048 with the VAE off the GPU; the decode's time and RAM there are
  not measured.
- The outputs are in `~/.cache/ai-ctl-probes/logs/sizes-vae-cpu-2026-10-29/`. The session's
  other scripts were kept beside it in `bin/`: `gateway-requests.sh` (one request per model,
  without then with the key), `gateway-ports.sh` (each server's own port), `gateway-tts-peaks.sh`
  (a speech model's peak, every model unloaded), `gateway-fibo-1536.sh`, `probe-sizes.sh`.
- At the close: the gateway stopped, no server process, the GPU at 16 MiB. Nothing
  uncommitted but this record.

### 2026-10-30

Resumed on the hand-over's first point, the image size. sd-server at `master-1024-77b72a2` read
before probing:
- With the VAE on the GPU, flash attention and direct convolution are already on (`flash
  attention auto: CUDA0 -> on`, `conv2d direct auto: CUDA0 -> on` in the 2048×2048 probe's
  log): the abort at 2048×2048 is the VAE's working set beside the DiT's and the text encoder's
  14.7 GB, which `--auto-fit` keeps in VRAM.
- `--offload-to-cpu` is `--params-backend *=cpu` (`examples/common/common.cpp:939`): the
  weights stay in RAM, the compute on the GPU.
- `--vae-tile-size` sets the tiles' size, 256 px by default: the colours set aside on
  2026-10-29 were those of 256 px tiles.

The plan, approved by the user: on FLUX.3 Image Klein, at 2048×2048 then larger, three decodes
compared: the VAE on the CPU, the reference; the weights in RAM and the VAE whole on the GPU;
tiles of 1024 px on the GPU, compared with the whole decode at 1536×1536 by PSNR and by eye.
Then Qwen-Image 2.1 and Fibo 2.0 Dev with the decodes kept.

FLUX.3 Image Klein, with `probe-vae-decode.sh` (`~/.cache/ai-ctl-probes/bin/`, untracked; the
outputs in `~/.cache/ai-ctl-probes/logs/vae-decode-2026-10-30/`): one `sd-server` per row, alone on the GPU,
`--eager-load`, 4 steps, seed 42, in a user scope capped at 32 GiB of RAM. PSNR by ImageMagick's
`compare` against the whole decode, the GPU's at 1536×1536, the weights in RAM at 2048×2048.

| Size, decode | Sampling | Decode | Peak VRAM | Peak RSS | Against the whole decode |
|---|---|---|---|---|---|
| 1536², the VAE on the GPU, as shipped | 6.06 s | 1.29 s | 22753 MiB | 3.3 GiB | — |
| 1536², tiles of 256 px | 6.08 s | 3.09 s | 17197 MiB | 3.1 GiB | 24.9 dB |
| 1536², tiles of 1024 px | 6.09 s | 2.08 s | 18593 MiB | 3.3 GiB | 33.8 dB, blue −4.5 on average |
| 1536², the weights in RAM | 6.31 s | 1.29 s | 11695 MiB | 15.7 GiB | identical, 0 pixels differ |
| 2048², the weights in RAM | 12.83 s | 2.35 s | 19625 MiB | 15.8 GiB | — |
| 2048², tiles of 1024 px | 12.64 s | 4.50 s | 18685 MiB | 3.2 GiB | 29.7 dB, red +2.0, blue −2.4 |
| 2048², the VAE on the CPU | 12.66 s | 299.44 s | 18525 MiB | 27.9 GiB | 71.2 dB, 80 pixels differ |

With the weights in RAM (`total params memory size … VRAM 0.00MB`), sd-server streams them to the
GPU at each step and the sampling loses under 4 %; the VAE then has the GPU to itself and
decodes the image whole. Tiles of 1024 px still shift the colours. The CPU's decode gives the
image the GPU's does, in 5 minutes.

Then larger, the weights in RAM:

| Size, decode | Sampling | Decode | Peak VRAM | Peak RSS |
|---|---|---|---|---|
| 2560², whole asked | 23.99 s | 7.25 s | 15095 MiB | 15.8 GiB |
| 2560², tiles of 1024 px | 24.08 s | 7.90 s | 15095 MiB | 15.8 GiB |
| 3840×2160, tiles of 1024 px | 34.30 s | 8.91 s | 16507 MiB | 15.9 GiB |
| 3072², tiles of 1024 px | 42.06 s | 12.32 s | 17434 MiB | 15.9 GiB |
| 4096², tiles of 1024 px | 108.92 s | 24.11 s | 23384 MiB | 16.1 GiB |
| 4096², tiles of 2048 px | 108.82 s | 20.84 s | 23384 MiB | 16.1 GiB |

- At 2560², the whole decode ran out of VRAM and sd-server did not abort: "Reducing VAE decode
  tiles from 2560x2560 to 1280x1280 image pixels", "VAE decode ran out of memory; retrying with
  spatial tiling" (`src/core/backend_fit.cpp:513`, `:518`), halving the tile until one fits.
  The image differs from the 1024 px tiles' by 35.9 dB.
- At 4096², the peak is the sampling's, whatever the tiles, 1079 MiB under the GPU's
  24463 MiB; 27 s a step, so about 7 minutes at the 16 steps of the inventory.

The three models with `--offload-to-cpu` alone, the tiles left to sd-server; Fibo's text
encoder on the GPU, since its weights are in RAM too. Each cell gives the sampling at 4 steps,
the decode, and the peak VRAM; "tiles" marks the decodes sd-server retried in tiles of half the
image (1280 px at 2560², 1536 at 3072², 1920×1072 at 3840×2160, 2048 at 4096²).

| Size | FLUX.3 Image Klein | Qwen-Image 2.1 | Fibo 2.0 Dev |
|---|---|---|---|
| 1536², as shipped | 6.06 s, 1.29 s, 22753 MiB | 15.33 s, 2.62 s, 23804 MiB | 22.57 s, 1.21 s, 23600 MiB |
| 1536² | 6.31 s, 1.29 s, 11695 MiB | 15.48 s, 2.60 s, 10648 MiB | 22.99 s, 1.22 s, 17602 MiB |
| 2048² | 12.83 s, 2.35 s, 19625 MiB | 34.64 s, 4.67 s, 18210 MiB | 46.74 s, 2.29 s, 19640 MiB |
| 2560² | 23.99 s, 7.25 s tiles, 15095 MiB | 66.49 s, 15.22 s tiles, 13038 MiB | 86.13 s, 7.19 s tiles, 20438 MiB |
| 3840×2160 | 34.19 s, 9.21 s tiles, 16508 MiB | not measured | not measured |
| 3072² | 41.98 s, 10.77 s tiles, 17434 MiB | 114.87 s, 22.71 s tiles, 15336 MiB | 149.71 s, 10.73 s tiles, 22378 MiB |
| 4096² | 108.73 s, 20.73 s tiles, 23384 MiB | 293.95 s, 41.86 s tiles, 20170 MiB | 398.53 s, 20.70 s tiles, 23514 MiB |

- Every size up to sd-server's limit passed for each model, none aborting. The peak RSS is the
  weights': 16.1, 16.9 and 20.9 GiB at most, against 2.8 to 5.6 GiB as shipped.
- At 1536², Qwen-Image's image is the one shipped, 0 pixels differing, as FLUX's; Fibo's
  differs by 39.7 dB, its text encoder now on the GPU, as on 2026-10-29.
- At 4096² and full steps, the sampling alone would take about 7 minutes for FLUX (16 steps),
  25 for Qwen-Image (20) and 83 for Fibo (50).

Committed as `docs(audio-image-backends): record the decodes compared up to 4096`.

Images at full steps, the weights in RAM, the fox's prompt, Fibo with Bria's negative prompt
(`STEPS` and `NEGATIVE` of the same script; the PNGs in each run's folder, a 1024 px preview of
each in `view/`):

| Model, size, steps | Sampling | Decode | Whole request | Peak VRAM |
|---|---|---|---|---|
| FLUX.3 Image Klein, 2048², 16 | 50.31 s | 2.38 s | 53.3 s | 19626 MiB |
| Qwen-Image 2.1, 2048², 20 | 169.82 s | 4.70 s | 175.4 s | 18210 MiB |
| Fibo 2.0 Dev, 2048², 50 | 579.54 s | 2.30 s | 582.6 s | 19640 MiB |
| FLUX.3 Image Klein, 4096², 16, tiles | 434.28 s | 20.79 s | 457.2 s | 23384 MiB |

Read by the agent before the user's verdict: each image whole and coherent, one fox, no subject
repeated; at 4096², FLUX's fox fills more of the frame, and a 1024 px crop of its centre, across
the tiles' joins, shows no seam.

The user's verdict by eye, 2026-10-30: all very beautiful. Qwen-Image first, FLUX at 2048² second,
Fibo third, less beautiful, though its fox, seen from afar in the middle of the forest, looks
almost more realistic; FLUX at 4096² beautiful too, sharper when zoomed, but not worth its
time. The user proposes to cap the size at 2048×2048, and to keep a choice of sizes for a
request made from the command line. sd-server has no option bounding the size, its limit of 4096
a constant: asked how to cap, the user chose OpenWebUI at 2048×2048 and the budgets at 4096×4096
(Decisions).

Fibo at 4096², 4 steps, with its negative prompt, as it will ship: peak 23390 MiB, under the
23514 measured without it; its budget stays the larger.

The inventory changed accordingly:
- `deploy/inventory.toml`: `offload-to-cpu = true` in the three image models' flags, Fibo's
  `backend = "te=cpu"` removed, the budgets 23384, 20170 and 23514 MiB, the comments on the
  RAM, the decode and the times at 2048×2048.
- The snapshot tests watched failing on their assertion (`--offload-to-cpu` where
  `--sampling-method euler` stood), then regenerated: still 32 sets, Qwen-Image now beside
  whisper large-v3-turbo (`fit-19`).
- `docs/image/stable-diffusion-cpp.md` §4: the decodes compared, the sizes with the weights in
  RAM, the times at full steps, the user's verdict and the decision, and a request from the
  command line with its own size, the key on curl's standard input.
- fmt, clippy, the tests (795, 27, 3), the two evals and `sid.py check` green; `cargo mutants
  --in-diff`: no Rust source in the diff.

Committed as `feat(inventory): keep the image models' weights in RAM up to 4096`.

The probe's images were under `~/.cache`, hidden from the user's file manager: the four at full
steps were copied to `~/Images/ai-ctl-2026-10-30/` at the user's word.

Image editing, read before probing (sd.cpp at `77b72a2`, OpenWebUI 0.12.3):
- sd-server's `POST /v1/images/edits` takes a multipart `prompt`, `image[]` or `image`, an
  optional `mask`, `n` and `size`; each image becomes a reference image of the request
  (`examples/server/routes_openai.cpp:85–210`). llama-swap routes it by the form's `model`
  (`docs/gpu/llama-swap.md`).
- OpenWebUI sends that form to `{base}/images/edits`: `model`, `prompt`, `n`, `size`,
  `response_format: b64_json`, and `image` or `image[]` (`routers/images.py:984–1041`), under
  settings of its own: `IMAGE_EDIT_ENGINE`, `IMAGE_EDIT_MODEL`, `IMAGE_EDIT_SIZE`,
  `IMAGES_EDIT_OPENAI_API_BASE_URL` and `IMAGES_EDIT_OPENAI_API_KEY`.
- FLUX.3 Image Klein edits with its files, up to 10 references (`docs/flux3.md`). Qwen-Image 2.1
  and Fibo 2.0 Dev need their text encoder's vision projector, `--llm_vision`
  (`docs/qwen_image_2.1.md:15`, `docs/fibo2.md:47`). Fibo's masked editing is not wired; Bria's
  Fibo 2.0 Edit Turbo, distilled for editing in 4 steps, is not downloaded.

The editing task was proposed to the user with the two downloads it needs, "si tu es d'accord";
the user downloaded both and left the rest to a new session: the task is added under Live
(Deviations). Both files checked in session, their size and SHA-256 equal to the Hub's
`lfs.oid` at the pinned revisions:

| File | Repository, revision | Size | SHA-256 |
|---|---|---|---|
| `image/qwen-image-2.1/mmproj-Qwen3VL-8B-Instruct-F16.gguf` | `Qwen/Qwen3-VL-8B-Instruct-GGUF` `f982a075` | 1159029824 | `ca524100…891cf9d38` |
| `image/fibo-2.0-dev/mmproj-Qwen3VL-4B-Instruct-F16.gguf` | `Qwen/Qwen3-VL-4B-Instruct-GGUF` `1cd86afb` | 836180256 | `256f3a43…a9985331` |

At the close: no probe runs, the GPU free; nothing uncommitted but this record.

After the close, the user asked whether OpenWebUI does all of image editing or a specialised
interface is needed:
- OpenWebUI edits from the chat: an image attached and the change described, or a chat model
  calling its `edit_image` tool (`utils/middleware.py:1538`, `tools/builtin.py:444`). Its form
  holds no mask, seed, steps or strength, and its OpenAI request no negative prompt
  (`routers/images.py:854–862`, `:991–1006`); one model and one size, set by the admin.
- sd.cpp's own web UI, `leejet/sdcpp-webui`, the submodule `examples/server/frontend`, has a
  mask, an init image and reference images, the seed, the sampler, the strength, the negative
  prompt, a live preview and a cancel, over the native `/sdcpp/v1/…` routes (its README). At
  `5148144` (2026-10-28) it sends no `Authorization` header: behind the key, each request gets
  401. ai-ctl builds `sd-server` without it (`SD_SERVER_BUILD_FRONTEND=OFF`), its build running
  `pnpm install`.
- ComfyUI, InvokeAI and Krita AI Diffusion run a GPU backend of their own, outside llama-swap.

On the user's approval, phase 4 is added: sd.cpp's web UI through the gateway with the key
(Deviations).

Resumed in a new session on the editing task, at the user's word:
- The installed inventory was the one of `fix(inventory): budget the speech models with a 40 s reference` of 2026-10-29, untouched since its install: moved to the session's
  scratch folder, then `ai-ctl init --models-dir /ai/models`; `doctor` 37 ok.
- `edit.sh` (the session's scratch folder): one `POST /v1/images/edits` through the gateway as
  OpenWebUI sends it, multipart `model`, `prompt`, `n` 1, `size` `2048x2048`,
  `response_format` `b64_json` and `image`, the key on curl's standard input, the GPU sampled
  every 100 ms. The reference is FLUX's fox at 2048×2048 (`~/Images/ai-ctl-2026-10-30/`), the
  prompt "Put a red knitted wool scarf around the fox's neck, keep everything else unchanged",
  each model swapped in by the request, its load included in the time.
- `deploy/inventory.toml`: `llm_vision` in Qwen-Image 2.1's and Fibo 2.0 Dev's
  `[models.files]`, their projectors; sd-server reads the flag with its underscore, which the
  inventory allows for every kind but llama-server. The two snapshot tests watched failing on
  their assertion, then regenerated: one `--llm_vision` line per model. `init` installs the
  inventory compiled into the binary: `make bin` first, then the installed copy replaced and
  `doctor` 37 ok, 4 files for each of the two.
- `docs/image/stable-diffusion-cpp.md` §4: the two projectors and their downloads.

| Model | Answer | Generation, sampling | Peak VRAM, budget | Result |
|---|---|---|---|---|
| FLUX.3 Image Klein, 16 steps | 200 in 84.5 s | 71.85 s, 68.12 s | 21305 MiB, 23384 | the scarf, the scene unchanged |
| Qwen-Image 2.1, 20 steps | 200 in 484.8 s | 476.30 s, 464.48 s, 22.7 s a step | 22773 MiB, 20170 | the scarf, long and knitted, the scene unchanged |
| Fibo 2.0 Dev, 50 steps | 200 in 1605.0 s | 1590.20 s, 1585.15 s, 31.7 s a step | 22927 MiB, 23514 | the scarf, the scene unchanged |

- Each server logged `EDIT mode` and its preset for reference images, `flux2`, `qwen` and
  `fibo2`; Qwen-Image and Fibo loaded their projector ("loading llm vision from …"), their
  weights all in RAM (16750 and 20548 MB).
- Qwen-Image's peak is above its budget: sd.cpp tried to keep its prefix cache, the reference's
  and the prompt's keys and values, in VRAM, found 1.4 GB free where it needed 1.5 more ("model
  manager cannot make enough memory available on CUDA0 … unlimited budget"), released it and
  sampled without it ("insufficient memory for prefix caching; retrying without it"). Its
  budget lets whisper large-v3-turbo run beside it (`fit-19`): a whisper started while the cache
  holds the GPU would find no room. `sd-server --help` at `77b72a2` offers `--max-vram`, "a
  per-device budget in GiB for managed weights and runner buffers", not probed.
- An edit's step against a generation's at 2048×2048 (the 4-step table of §4): FLUX's 4.3 s
  against 3.2, Qwen-Image's 22.7 against 8.7, Fibo's 31.7 against 11.7. Fibo's preset scales a
  single reference to the output's size (`docs/fibo2.md`).
- llama-swap logs "non-200 response, recording partial metrics: status=100" for each edit:
  curl's `Expect: 100-continue` on a multipart body; the answer is 200.
- The three results copied to `~/Images/ai-ctl-2026-10-30/`, `5-` to `7-`, to be judged by the
  user, who kept `7-` alone there; the three stay in the session's scratch folder.

The user's verdict by eye: all three change the colours; Fibo's stays the closest to the
reference but its scarf is the worst made; FLUX's is the most beautiful, its colours much
changed. Measured against the reference by ImageMagick, the whole image, then its top 600 rows,
trees and snow only:

| Model | PSNR, whole | Mean R, G, B, whole | PSNR, top | Mean R, G, B, top |
|---|---|---|---|---|
| Reference | — | 116, 114, 118 | — | 94, 94, 97 |
| FLUX.3 Image Klein | 22.4 dB | 123, 116, 113, warmer | 23.4 dB | 100, 96, 92 |
| Qwen-Image 2.1 | 16.0 dB | 133, 123, 139, brighter and more magenta | 19.0 dB | 96, 95, 108 |
| Fibo 2.0 Dev | 25.5 dB | 119, 117, 121 | 31.1 dB | 97, 99, 101 |

Qwen-Image's budget, the plan approved by the user: `--max-vram`, which sd.cpp reads as a
per-device budget for "managed weights and runner buffers" (`src/core/ggml_graph_cut.cpp:222`),
probed with `maxvram.sh` (the session's scratch folder): one `sd-server` alone on the GPU, the
gateway stopped, the inventory's command for the model from the generated configuration plus
`--steps 4` and the flags probed, a throwaway key; then each request in turn, its peak from the
GPU sampled every 100 ms.

| Qwen-Image 2.1, 4 steps | Edit, 2048² | Generation, 4096² | Generation, 2048² |
|---|---|---|---|
| `--max-vram 19` | 108.0 s, 20075 MiB, the prefix cache released | 340.7 s, 22155 MiB, 21487 through the sampling | — |
| `--model-args qwen_image_2_1_prefix_cache=false` | 103.5 s, 18211 MiB | 348.8 s, 21449 MiB, through the sampling | 42.6 s, 18211 MiB, 9.27 s a step |

- `--max-vram 19` bounds the edit's prefix cache, 19 GiB plus the CUDA context's 600 MiB,
  but not the generation at 4096×4096, which holds 21.5 GB through its sampling.
- Without the prefix cache, an edit's step is the same, 22.7 s, the cache having never fitted;
  a generation's at 2048×2048 takes 9.27 s against 8.66 with it (§4's table).
- Both runs put the generation at 4096×4096 above the budget measured on 2026-10-30, 20170 MiB,
  at the same build. Without the projector (`DROP=--llm_vision`), the same generation holds
  21183 MiB through its sampling, a peak of 21209, in 338.0 s: the projector adds 240 MiB. On
  2026-10-30, `probe-vae-decode.sh` held 20154 MiB with the same weights and flags, its prompt
  shorter ("A red fox sitting in a snowy birch forest at dawn, photograph" against the sign's
  prompt here), the only difference found, not probed further: Qwen-Image's budget is a
  gigabyte short of the generation at 4096×4096 with this prompt.

On the user's choice (Decisions), Qwen-Image's budget is its edit's peak:
- `deploy/inventory.toml`: `vram_mib = 22773`, the comments on an edit's time and on the
  budget an edit sets. The snapshot tests watched failing on their assertion, then regenerated:
  still 32 sets, Qwen-Image alone in `fit-31`, whisper large-v3-turbo's set with it gone.
- `docs/image/stable-diffusion-cpp.md` §4: editing, its measures, the prefix cache, `--max-vram`,
  the decision, and an edit from the command line.
- fmt, clippy, the tests (795, 27, 3), the two evals and `sid.py check` green; `cargo mutants
  --in-diff`: no Rust source in the diff.

Committed as `feat(inventory): edit images, Qwen-Image budgeted at its edit's peak`.

Then the gateway and OpenWebUI, for the Live and Clients tasks left:
- The installed inventory, this session's first install with the projectors and the old budget,
  replaced after `make bin`; `doctor` 37 ok; `ai-ctl up`.
- §4's two commands from the command line, run as written through the gateway with FLUX.3 Image
  Klein, a prompt in place of "…": a lighthouse at `2048x1152`, a 2048×1152 PNG in 34.5 s,
  then an edit of it at the same size, a 2048×1152 PNG in 39.9 s.
- OpenWebUI 0.12.3 started (`podman start openwebui`); from the container,
  `http://10.0.2.2:8080/health` answers 200. Its edit settings (`open_webui/config.py:1534–1547`):
  `images.edit.enable`, `.engine` (`openai` by default), `.model`, `.size`, and a base URL and
  key of their own, the image generation's by default. With
  `ENABLE_OPENAI_IMAGE_EDIT_NORMALIZATION`, true by default, it rewrites a JPEG reference with an
  EXIF orientation, several frames or another mode as an upright RGB JPEG
  (`routers/images.py:119–155`), and leaves every other image as sent.
- `GET /v1/audio/voices` lists the library's nine voices for `qwen3-tts` and `fish-s2-pro-q8`,
  none for `voxtral-4b-tts`, whose voices are its own; each request loaded its model.

OpenWebUI's settings given to the user, in Admin Panel → Settings, each with the base URL
`http://10.0.2.2:8080/v1` and the key its chat connection holds, each model typed by hand:
speech-to-text `openai` with `whisper-large-v3-turbo`; text-to-speech `openai` with `qwen3-tts`
and a voice of the library; image generation `openai` with `fibo-2.0-dev` at `2048x2048`, for
task 5's long request; image editing enabled, `openai` with `flux-3-image-klein` at
`2048x2048`. The user left the uses from the interface to a new session ("on fera les essais
dans une nouvelle session").

At the close: no probe runs, the GPU at 16 MiB; the gateway of this session's `ai-ctl up` still
runs, no model loaded, and OpenWebUI's container too. Nothing uncommitted but this record.

Resumed in a new session on the hand-over's first point. The gateway of the last `up` still ran,
no model loaded, the installed inventory equal to the shipped one; OpenWebUI's container too. No
audio or image setting was saved in OpenWebUI yet.

OpenWebUI's speech, configured by the user with the agent's guidance, the labels read in the
image's English and French locales:
- Administrator → Audio: engines `openai`, base `http://10.0.2.2:8080/v1`, ai-ctl's key,
  `whisper-large-v3-turbo`, `qwen3-tts`, voice `micode-fr`. The two keys saved equal ai-ctl's,
  compared by SHA-256 in the container's database. The TTS model was saved as `" qwen3-tts"`, a
  leading space included, seen in the database before any request and retyped.
- The first dictation, 2.2 s of French, came back as Icelandic: whisper-server ran with
  `lang = auto` and detected `is` (p = 0.96). OpenWebUI sends a `language` only when the user's
  own Audio setting Language is set: the recorder passes `settings.audio.stt.language` to
  `/api/v1/audio/transcriptions`, which forwards it as the form's `language`
  (`routers/audio/__init__.py:1174`, `:910–913`). Set to `fr`, the second dictation, 3.8 s, ran
  with `lang = fr` and was transcribed right, by the user's word.
- Read aloud with `qwen3-tts` and `micode-fr`, validated by the user. Each read evicts the 27B,
  `fit-4` holding Qwen3-TTS with the whisper models only, and the next chat message evicts
  Qwen3-TTS. llama-swap logs "metrics: invalid JSON in response body path=/v1/audio/speech" for
  each sentence: it reads the audio as JSON for its metrics; the answer is the audio.
- The user's remark: the avatar menu's Settings opens the settings dialog, Administrator's
  sections included, without the Admin Panel's users.

The user asked for a configuration guide of OpenWebUI in the documentation, the chat's settings
included (Deviations):
- OpenWebUI's database holds no setting of a model of its own (`model` empty,
  `models.default_params` `{}`): the chat's settings are its connection, the default OpenAI
  connection and Ollama turned off, the empty task model and autocomplete off.
- The gateway's `/v1/models` lists the 13 models, so the chat's model selector offers the speech
  and image models too. A model turned off in Administrator → Models, the page the user chose
  over the connection's Model IDs, leaves OpenWebUI's list (`utils/models.py:198–238`); the
  speech and image routes name their model from their own settings and never read that list.
- `docs/openwebui.md` written: the container and the chat's settings moved from `docs/stack.md`
  §7.3, which now points to it, the key's five settings, speech, images, and switching models;
  `docs/security.md`, `README.md`, `docs/README.md` and the three servers' OpenWebUI sections
  link to it. The images' part and the Model IDs, written from OpenWebUI's code, are to be
  checked by the user following the guide.
- The user turned off the 11 speech and image models in Administrator → Models (the database's
  `model` rows, `is_active` 0), and set the images from §7 of the guide at `1024x1024` for the
  tests, at the user's request; the engine reads "Default (Open AI)", corrected in the guide.

Images from OpenWebUI, the GPU sampled every 0.5 s (the session's scratch folder):
- The first request, on the 27B, got an answer saying no image tool was at hand: OpenWebUI hands
  the chat model `generate_image` only when the chat's Image toggle is on, and shows the toggle
  only when the configuration loaded with the page has image generation on
  (`utils/tools.py:806–821`, the input bar's `showImageGenerationButton`). After a reload, the
  toggle on, the image came.
- That request ran on the 35B: it called the tool, Fibo 2.0 Dev evicted it (`fit-32`), loaded in
  about 30 s with its weights in RAM, generated 1024×1024 in 115.44 s at a peak of 16891 MiB
  against its 23514, then the 35B loaded again and answered; about 2 min 35 s from the question
  to the answer. The image was judged good by the user. The first request on the 27B had no
  image tool because the user had not turned the toggle on; nothing to fix.
- An edit, FLUX's fox at 2048×2048 attached (9.1 MB of PNG), the scarf asked:
  - On the 35B, the user cancelled after 1 min 40 s of waiting; llama-server kept encoding the
    image and freed the GPU about 3 min 35 s after the request, when llama-swap could load the
    27B for the next one.
  - On the 27B, the prompt's text took 4.1 s, then the image's 4096 tokens 231 s: the vision
    projector runs on the CPU (`no-mmproj-offload = true`, 4 threads, the GPU at 0 %), and
    llama-server caps a Qwen-VL image at 4096 tokens by default (`tools/mtmd/clip.cpp:1756` at
    `b11700`), about 2048×2048. The inventory's comment, "slightly slower image encoding", holds
    for a small image only.
  - The 27B then called `edit_image` with a size of its own, 2048×2048, over the configured
    1024×1024: the tool's `size` wins over the setting (`routers/images.py:913–917`). FLUX
    evicted the 27B, edited in 71.48 s at a peak of 19625 MiB against its 23384, and the 27B
    loaded again to answer. The user's verdict: the scarf is not realistic at all, as in the
    test edits.
  - Loaded again, the 27B had lost the image's tokens with its process: writing its answer after
    the tool's result, it encoded the image a second time, 221 s. An edit from the chat encodes
    the attached image twice, before the tool call and after the image model's turn.
- An edit of Fibo's lighthouse at 1024×1024, a sailing boat asked, on the 27B: the image's 1024
  tokens took 24.2 s on the CPU, against 231 s for 4096, the encoder's cost growing faster than
  its tokens; FLUX edited at 1024×1024, the configured size, in 22.23 s, then the 27B loaded
  again and encoded the image a second time, 26.8 s.
- Asked why a cap rather than an OpenWebUI setting, the user was right in substance: OpenWebUI
  compresses an attached image in the browser before its upload, under the administrator's
  `file.image_compression_width` and `_height`, Administrator → Documents → Files, or the user's
  own Image Compression (the chat input's upload handler). The image size of Administrator →
  Images is the size the image models make, not the attachments'. The user set the two to 1024.
- The next edit seemed as slow: the user had attached `1-flux-2048.png` from OpenWebUI's own file
  list, the copy uploaded at 2048×2048 before the setting (file `9d4b7402…`); the compression
  applies to an upload from the computer only. The user stopped it after 2 min 50 s.
- Uploaded again from the computer, the same image reached the 27B at 1024×1024: its prompt
  1146 tokens, read in 26.3 s; FLUX got a 1024×1024 reference and edited it in 22.09 s; the 27B,
  loaded again, read it a second time in 30.2 s. About 1 min 55 s from the question to the
  answer, against 9 minutes for the scarf at 2048×2048.

On the user's approval of the plan (Decisions), OpenWebUI's images are set at 1024×1024 and the
attachments compressed to 1024:
- `docs/openwebui.md` §7: the two sizes, the compression and why, the chat model's own size, the
  times measured; `docs/image/stable-diffusion-cpp.md` §4 the decision, §5 the tool's size;
  `docs/stack.md` §5 the projectors on the CPU and their times; the inventory's comment on
  `no-mmproj-offload`, which promised "slightly slower image encoding", gives the times.
- fmt, clippy, the tests (795, 27, 3), the two evals and `sid.py check` green; `cargo mutants
  --in-diff`: no Rust source in the diff.

Committed as `docs(openwebui): add the settings guide, images at 1024 in OpenWebUI`.

The user's ranking for their own use, after the day's images: FLUX.3 Image Klein below
Qwen-Image 2.1 and Fibo 2.0 Dev, Qwen-Image the best trade of quality for time; the faster models
still useful to iterate over several images before choosing one. The guide names no model: the
shipped settings leave the choice to the user.

The Documentation tasks, at the user's word:
- `docs/gpu/llama-swap.md`: the diagram without the Python backends, the backends table as the
  shipped inventory serves them with each budget, Qwen-Image 2.1 alone at 22773 MiB, and To Come
  with the Python backends, sd.cpp's web UI and the orchestrator.
- `README.md`: the domains served, the roadmap's status, and a section using the speech and image
  models with curl, the key on its standard input. Its three requests run through the gateway
  first: JFK's MP3 transcribed word for word by whisper-large-v3-turbo in 2.2 s, a French
  sentence spoken by `qwen3-tts` with `micode-fr` in 13.4 s, a WAV at 24 kHz, and a 1024×1024 PNG
  from `flux-3-image-klein` in 19.0 s, each load included. `docs/README.md`'s index names the
  backends served.

Committed as `docs(gpu): describe the backends served, speech and images in the README`.

`ai-ctl down`, at the user's word: FLUX.3 Image Klein was loaded, 900 MiB on the GPU with its
weights in RAM. "Gateway stopped (PID 1084724); 900 MiB of GPU memory released" in 1.7 s; then
`status` exited 3, no llama-swap, `llama-server`, `whisper-server`, `audiocpp_server` or
`sd-server` process was left, the GPU held 14 MiB and listed no compute process.

The user's word on the sizes: 2048 and 4096 were asked out of curiosity and for the tests; their
times are too long for a chat, and a larger image is a case for the command line.

---

## Decisions

- **The voice library (the user, 2026-10-29):** each reference recording becomes
  `<name>-fr.wav` in `/ai/models/voices/`, beside the folder that holds its source, with its
  line in `prompt_text`; the suffix names the reference's language, since Qwen3-TTS keeps a
  French accent in English with a French reference, and leaves room for an `-en`. A recording
  longer than a minute is cut at a pause near 40 s, the others' length, since a reference's
  length weighs on every request. Since task 2's budgets, measured with a reference of 40.6 s,
  every recording longer than that is cut at a pause before it, so that the budgets hold. Its people's names and the recordings' content stay out of
  the tracked files. Set aside: the folder's name alone; the 95 s recording whole.
- **Fibo's negative prompt is a flag of its entry:** `negative-prompt`, Bria's `NEGATIVE_T2I`
  as one line of JSON, which sd-server's OpenAI route takes as each request's default. Set
  aside: the probe's `--negative-prompt-file` (phase 2's report, 2026-10-29), a file no
  repository publishes, for which
  `doctor`'s remedy would be a wrong download.
- **The image models are budgeted at the largest size they make (the user, 2026-10-29):**
  1536×1536 for the three, each `vram_mib` its peak there, so that none runs beside another
  model. The Live tasks set OpenWebUI to that size. A larger request still aborts `sd-server`,
  which bounds no size. Set aside: the 1024×1024 peaks, which let whisper share the GPU with an
  image model and a larger image abort the server.
- **The image models keep their weights in RAM, budgeted at 4096×4096, and OpenWebUI asks for
  2048×2048 (the user, 2026-10-30):** `offload-to-cpu` in each image model's flags, so that every
  size up to sd-server's limit passes, the decode whole up to 2048×2048, in tiles above; each
  `vram_mib` is the peak at 4096×4096, so that no request a client can make aborts the server;
  Fibo's text encoder back on the GPU, its weights in RAM too. OpenWebUI's size is 2048×2048,
  the largest whose decode is whole, 4096×4096 being sharper but not worth its time; a request
  from the command line chooses its size up to 4096. It replaces the decision above for the
  budgets and OpenWebUI's size. Set aside: a patch making sd-server refuse a side above 2048,
  which would let whisper run beside an image model, a fourth patch to keep up; the VAE on the
  CPU, 5 minutes a decode at 2048×2048; tiles on the GPU with the weights in VRAM, which shift
  the colours. It costs 16 to 21 GiB of RAM while an image model is loaded.
- **Qwen-Image 2.1 budgeted at its edit's peak, alone on the GPU (the user, 2026-10-30):**
  `vram_mib = 22773`, the peak of an edit at 2048×2048 whose prefix cache sd.cpp tried to keep
  in VRAM, above its generation at 4096×4096 (21449 MiB with this session's prompt); the cache
  kept. Whisper large-v3-turbo no longer runs beside it. It replaces the decision above for
  Qwen-Image's budget. Set aside: the cache turned off and a budget of 21449 MiB, alone too, for
  7 % more time a generation's step; `--max-vram`, which bounds the cache but not the generation
  at 4096×4096.
- **OpenWebUI's images at 1024×1024, its attachments compressed to 1024 (the user,
  2026-10-30):** 2048×2048 and 4096×4096 were asked out of curiosity and for the tests, their
  minutes too long for a chat; a larger size is for the command line. Generation and editing ask
  for 1024×1024, and OpenWebUI compresses an uploaded image to 1024 a side, so that the chat
  model reads it in about 25 s on the CPU rather than 4 minutes, twice an edit. The budgets stay
  at 4096×4096, since the chat model may give the image tool a size of its own. It replaces the
  decision above for OpenWebUI's size. Set aside: llama-server's `--image-max-tokens 1024` for
  the two chat models, which would have kept the reference whole for the image model but changes
  the inventory for one client's need, OpenWebUI holding a setting of its own, as the user
  pointed out; the projectors on the GPU, 0.9 GB and a working set above budgets of 21 to
  22.7 GB.

---

## Deviations

- **Phase 0's budgets of the cloning models and of Qwen3-ASR were too low** (2026-10-29, found
  in task 2): measured with JFK's 11 s as reference and as audio, they let Fish S2 Pro Q8_0 load
  beside two other TTS models and fail its first request with the user's 40 s reference. Fixed
  in this phase: five budgets measured again, `docs/audio/audio-cpp.md` §5. A reference longer
  than the library's needs its peaks measured again; the inventory's comment says so.
- **A task added to this phase** (2026-10-29, the user's decision): each image model's largest
  size with its VAE on the CPU, up to 4096 a side, under Live in `phase-3-end-to-end.md`.
- **That task reworded** (2026-10-30, the user's approval): the VAE on the CPU alone became
  three decodes compared, the CPU's, the GPU's with the weights in RAM (`--offload-to-cpu`) and
  the GPU's by tiles of 1024 px, since the CPU's took 161.65 s at 1536×1536 and needs RAM in
  proportion to the pixels.
- **Qwen-Image and Fibo judged at 2048×2048, not at their largest size** (2026-10-30): the task
  asked an image at full steps at each model's largest size, 4096×4096 for the three; only
  FLUX's was made, the user finding 4096×4096 not worth its time, about 25 and 83 minutes for
  the two others. Nothing left open: OpenWebUI's size is 1024×1024 since the evening's decision.
- **An editing task added** (2026-10-30, the user's agreement, given by downloading the files
  it needs): image editing through the gateway and from OpenWebUI, under Live in
  `phase-3-end-to-end.md`, as the user asked on 2026-10-29 ("les fonctions d'édition").
- **Phase 4 added** (2026-10-30, the user's approval, "tu peux ajouter phase à la roadmap"):
  `phase-4-image-web-ui.md`, sd.cpp's own web UI through the gateway with ai-ctl's key, for the
  editing OpenWebUI lacks: a mask, init and reference images, the seed, the steps, the strength.
  The roadmap's scope grows by a web UI that `sd-server` serves; the GUIs with a GPU backend of
  their own stay out.
- **The Clients task's document reworded** (2026-10-30, the user's request for a configuration
  guide of OpenWebUI): OpenWebUI's whole configuration, the chat's from `docs/stack.md` §7.3
  included, goes to a document of its own, `docs/openwebui.md`, rather than §7.3 or the audio and
  image documents, whose OpenWebUI sections keep what it sends to each server.

---

## Assessment

The shipped inventory serves 13 models on four backends: two chat models on `llama-server`, two
whisper models, Qwen3-ASR and five speech models on `audiocpp_server`, three image models on
`sd-server`. Each answered through the gateway with ai-ctl's key and got llama-swap's 401
without it, before any server started; each server's own port refuses every route but its
health check, and `llama-server`'s web page, without the key. The swaps follow the matrix, and `down` leaves no process and no
GPU memory. OpenWebUI transcribes, reads aloud, makes and edits images through the gateway, and
`docs/openwebui.md` says how to set it, the chat included.

The live runs corrected the budgets the doubles could not test:
- the cloning models and Qwen3-ASR were measured again with a 40 s reference, after Fish S2 Pro
  Q8_0 failed beside two other speech models;
- the image models keep their weights in RAM, so that every size up to sd-server's 4096 passes,
  each budget the peak of its largest request; Qwen-Image 2.1's is its edit's, alone on the GPU.

OpenWebUI's first uses found what no probe had: whisper guesses the language of a short
dictation wrong unless the user's Language is set; a model typed with a leading space is sent
with it; the chat's Image toggle is needed for the image tools; the chat models read an attached
image on the CPU, minutes for a 2048×2048 one, twice an edit, since the image model evicts them;
and the chat model may give the image tools a size of its own. OpenWebUI now asks for 1024×1024
and compresses attachments to 1024; larger sizes are for the command line.

Left open: 4K by sd-server's ESRGAN upscale, asked on 2026-10-29 and not probed; `laure-fr`'s
transcript, for the user to correct in `/ai/models/voices/prompt_text`; the edits' quality, FLUX's
unrealistic and Qwen-Image's colours shifted, a matter of the models.

What phase 4 needs to know first:
- sd.cpp's web UI, `leejet/sdcpp-webui` at `5148144`, sends no `Authorization` header, and
  ai-ctl builds `sd-server` without it (`SD_SERVER_BUILD_FRONTEND=OFF`, its build running
  `pnpm install`).
- llama-swap routes a request by its `model`: a request naming none, such as
  `GET /sdapi/v1/sd-models`, gets its 404; `/upstream/<model>/<path>` forwards any path.
- An image model loaded and idle holds 900 MiB of VRAM, its weights in RAM.
