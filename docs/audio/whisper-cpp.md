# Speech-to-Text: whisper.cpp

whisper.cpp's `whisper-server` serves OpenAI's transcription endpoint from a ggml model on the GPU.
ai-ctl builds it, `ai-ctl build whisper.cpp`, and the gateway serves the inventory's
`whisper-server` models with it. This document says how it builds, runs, gets ready and answers,
what guards it, and which models it serves. Every fact below was probed on this machine
on 2026-10-20 unless dated otherwise, at release `v1.9.5` (commit `d1be6fde`, 2026-10-06), with
nvcc 13.4.92, g++ 16.2.1 and llama-swap v263.

---

## 1. Build

- **Repository and releases.** `https://github.com/ggml-org/whisper.cpp.git`, 95 tags. Releases
  are `vX.Y.Z`. Beside them sit 41 `b<N>` tags, older forms (`1.0.3`, `0.0.6-1`),
  `v1.7.4-pre-1`-style tags and two personal test tags. GitHub's prerelease flag cannot pick the
  releases: `v1.9.3` is marked prerelease and `b4938` is not. A strict `vX.Y.Z` pattern, ordered
  numerically, does.
- **Options.** The builder's options for llama.cpp (`CMAKE_BUILD_TYPE=Release`, `GGML_CUDA=ON`,
  `GGML_NATIVE=ON`, `CMAKE_CUDA_ARCHITECTURES=120a-real`, `BUILD_SHARED_LIBS=OFF`, the CUDA and
  host compilers), plus `WHISPER_BUILD_TESTS=OFF` and `WHISPER_BUILD_EXAMPLES=ON`. The server is
  the target `whisper-server` of `examples/server`, added under `WHISPER_BUILD_EXAMPLES`.
  `WHISPER_BUILD_SERVER` is declared but read nowhere. `WHISPER_BUILD_IS_DEV`, on by default,
  only appends `-dev` to the version CMake prints. ggml 0.26.0 is vendored.
- **The build.** `cmake --build … --target whisper-server --parallel 24`: 185 steps, 144 CUDA
  objects, 194 s, no warning. The binary lands at `bin/whisper-server`, 52.8 MB. Its 144 cubins
  are all `sm_120a`, with no PTX. It links `libcudart.so.13`, `libcublas.so.13` and
  `libcublasLt.so.13` from `/usr/local/cuda-13.4`, and `libgomp`.
- **No `--version` and no device listing.** `--version` prints "error: unknown argument:
  --version" and the usage, then exits 0, so a smoke test on it passes for any binary that
  starts. Started with a missing model, the server exits 3 before ggml initialises CUDA. `--help`
  prints everything on stderr and exits 0. It is the only command the server runs without a
  model: `tests/fixtures/whisper-server-v1.9.5-help.txt`.

---

## 2. Serving

- **Options.** `--host` (default `127.0.0.1`), `--port` (8080), `-m/--model`,
  `--inference-path` (`/inference`), `--request-path` (a prefix for every route), `-l/--language`
  (`en`), `-t/--threads` (4), flash attention (on). `--public` serves a folder of static files
  at `/` (`examples/server/public`, relative to the working directory). `--convert` transcodes
  each upload to WAV by running `ffmpeg` through `system()` on a file in `--tmp-dir` (`.` by
  default).
- **Routes** (`examples/server/server.cpp`):
  - `GET /` serves a page.
  - `POST <inference-path>` takes a multipart `file` and OpenAI's fields. It ignores the request's
    `model` and answers the `response_format` values `json`, `text` and `verbose_json`.
    `verbose_json` turns on token timestamps, which make `whisper_full` wrap segments at 60
    characters on a token boundary, so its `text` cuts words ("évén" and "ements";
    `examples/server/server.cpp:627–636`, 2026-10-21). With `json`, as OpenWebUI sends, the text
    breaks only between sentences.
  - `POST /load` takes a multipart `model`, a path on the host, and loads it in place of the
    current model. A file that fails to load makes the server exit 1.
  - `GET /health`.
- **Language.** A request without `language` takes the server's `-l`, `en` by default, and
  French is then decoded as English. On 2026-10-21, large-v3-turbo read Voxtral's French
  sentence as "The vif renard brun is on the side of the chien paresseux.", and `verbose_json`
  gave `english`. With `-l auto`, the language is detected for each request (`french`, read word
  for word), and a request's `language` still wins. Detection can miss on a short dictation: on
  2026-10-30, 2.2 s of French sent by OpenWebUI without a language was detected as Icelandic
  (`is`, p = 0.96); with OpenWebUI's Language set to `fr`, 3.8 s were read right.

  Decided: ai-ctl gives `whisper-server` `-l auto` as a default of its own, which the kind's
  `[defaults.whisper-server]` and the model's flags override, so that an inventory written by
  hand does not decode French as English without a word (the user, 2026-10-28, phase 2 of
  `audio-image-backends`). Set aside: `language = "auto"` in the shipped inventory's
  `[defaults.whisper-server]` alone, which an inventory that leaves it out loses. To weigh again
  if the short dictations' misreadings make a fixed language the better default.
- **Audio.** Without `--convert`, the upload is decoded in memory (`read_audio_data`). WAV and
  MP3 give the same text (`samples/jfk.wav`, `samples/jfk.mp3`).
- **Readiness.** The model loads before the server binds its port, so nothing answers during
  the load; curl gets no connection. Then `GET /health` answers 200 `{"status":"ok"}`. Its 503
  `{"status":"loading model"}` shows only during a `/load`.
- **Binding its port.** Its cpp-httplib sets `SO_REUSEPORT` in place of `SO_REUSEADDR`, where
  llama-server sets `SO_REUSEADDR` alone (2026-10-28, `v1.9.5`):
  - a second `whisper-server` binds a port the first still listens on, and the kernel shares the
    connections between them, without a word;
  - it cannot bind a port in TIME-WAIT, after `python3 -m http.server` or a stopped `sd-server`:
    it exits 1, "couldn't bind to server socket".

  Decided: ai-ctl binds it with `SO_REUSEADDR` alone, by a patch of its own,
  `reuse-address.patch` (§3; the user, 2026-10-28). Probed on the server ai-ctl built: the second
  server exits 1 with that message, and a port in TIME-WAIT no longer stops it.
- **Behind llama-swap.** Each model runs as
  `whisper-server -m <file> --host 127.0.0.1 --port ${PORT} --inference-path /v1/audio/transcriptions -l auto`,
  with `checkEndpoint: /health`. The gateway routes `POST /v1/audio/transcriptions` by the
  multipart form's `model` field. With large-v3, the first transcription through the gateway took
  1.55 s, the load included, and the next took 0.23 s.

---

## 3. The Key

- **The server checks no key.** No option or variable sets one, neither at `v1.9.5` nor at the
  head of the main branch fetched on 2026-10-20 (`231638a0`). Every response carries
  `Access-Control-Allow-Origin: *` and allows the `authorization` header. On its port, a
  request without a key and with `Origin: https://attacker.example` gets its transcription and
  `Access-Control-Allow-Origin: *`. Any local process or web page can therefore use the GPU
  through it, or `POST /load` a path on the host, and a file that fails to load ends the server.
- **Decided: a patch of ai-ctl's own** (user's decision, 2026-10-20), as for the other two
  servers ([audio-cpp.md §4](audio-cpp.md#4-the-key),
  [stable-diffusion-cpp.md §3](../image/stable-diffusion-cpp.md#3-the-key)). It makes the
  server require the key llama-swap's `env:` hands it, as [security.md §3](../security.md#3-the-api-key)
  guarantees for llama-server. Set aside: serving it without a key as a written exception,
  which opens it to any page and any local account; waiting for upstream or for llama-swap to
  reach unix sockets; a network namespace, not probed. ai-ctl's recipe applies it to every build:
  `src/domain/patches/whisper.cpp/api-key.patch`, then `reuse-address.patch` (§2).
- **The prototype**, against `v1.9.5` (`examples/server/server.cpp`, +23 −4):
  - the server reads `LLAMA_API_KEY` before the model loads and, without it, exits 1 with
    "error: LLAMA_API_KEY is not set";
  - a pre-routing handler lets through `<request-path>/health`, and any request whose
    `Authorization` is `Bearer <key>` or whose `X-Api-Key` is the key; every other request gets
    401 with an OpenAI-style error;
  - the default CORS headers are gone;
  - the server's error handler turned every status but 400 and 500 into 404 "File Not Found";
    it now leaves 401 alone.

  Checked on the server's port with a throwaway key:
  - without the key, `/health` answers 200;
  - without the key or with a wrong one, a `GET`, the transcription `POST`, an `OPTIONS`
    preflight, a `text/plain` `POST` from `Origin: https://attacker.example` and `/load` all get
    401;
  - with `Bearer` or `X-Api-Key`, the transcription answers 200;
  - no `Access-Control-*` header reaches any origin.

  Behind llama-swap, with `env: ["LLAMA_API_KEY=${env.AI_CTL_API_KEY}"]`, the health check
  passes without a key and a transcription with the key answers 200.

---

## 4. Models

Each file comes from `ggerganov/whisper.cpp` on Hugging Face, the account of whisper.cpp's author,
which `models/download-ggml-model.sh` downloads from. Its licence is MIT. The provenance rule and
the checks are those of [stack.md §5](../stack.md#where-models-come-from). On 2026-10-20, each
file's SHA-256 equalled the Hub's LFS hash at the commit below, and its SHA-1 equalled the one
whisper.cpp's `models/README.md` gives at `v1.9.5`:

```bash
hf download ggerganov/whisper.cpp ggml-large-v3.bin \
  --revision 5359861c739e955e79d9a303bcbc70fb988958b1 \
  --local-dir "$MODELS_DIR/stt/whisper-large-v3"
hf download ggerganov/whisper.cpp ggml-large-v3-turbo.bin \
  --revision 5359861c739e955e79d9a303bcbc70fb988958b1 \
  --local-dir "$MODELS_DIR/stt/whisper-large-v3-turbo"
```

Each model was measured with its server alone on the GPU, which held 15 MiB before. Each start was
cold, after `POSIX_FADV_DONTNEED` on the model file. The request was `samples/jfk.wav`, 11 s of
English:

| Model | File | Ready, cold | Peak VRAM | One request |
|---|---|---|---|---|
| large-v3 | 3.10 GB | 1.84 s (1.15 s from the page cache) | 3895 MiB | 0.34 s |
| large-v3-turbo | 1.62 GB | 1.09 s | 2213 MiB | 0.20 s |

Both read the sample word for word. Each speech-to-text model of the GPU is also an entry of the
inventory; the policy for several models of one use is in
[gpu/llama-swap.md](../gpu/llama-swap.md#the-backends). Qwen3-ASR, under audio.cpp, is the other
speech-to-text model ([audio-cpp.md §5](audio-cpp.md#5-models)).

**Compared on the user's recordings** on 2026-10-21, the three readers each alone on the GPU, each
recording sent once with its language and once without; all 18 readings were the same both ways.
Whisper took the MP3 as OpenWebUI sends it, Qwen3-ASR a 16 kHz mono WAV. The recordings are
`reference-fr`, 37 s of French with its transcript, and `transcript-fr` and `transcript-en`, 60 s
each of spontaneous speech by a French speaker:

| Reader | `reference-fr`, words differing from the audio | One request: 37 s FR / 60 s FR / 60 s EN |
|---|---|---|
| whisper large-v3 | none | 1.17 / 2.01 / 1.54 s |
| whisper large-v3-turbo | 1, "Ce n'est" for "Ça n'est" | 0.50 / 0.93 / 0.63 s |
| Qwen3-ASR 1.7B | 3, "donnait" for "tenait", "ce spécimen" for "se spéculent" | 2.43 / 3.77 / 2.87 s |

The words were compared lower case, without punctuation, elisions and numbers in words taken as
the transcript's. The user's verdicts by ear, on 2026-10-23:
- On `reference-fr`, large-v3 and turbo differ little, and both are far ahead of Qwen3-ASR, whose
  numbers in words read less well.
- On spontaneous speech, Qwen3-ASR follows the words as spoken, hesitations and repetitions
  included, where both whisper models write the sentence the speaker meant. Qwen3-ASR alone is
  right on some words and wrong on others, and adds a trailing filler where a recording is cut
  short. All three miss "Three.js Journey" in English.
- None fails, and each keeps a use: large-v3-turbo fast and efficient, large-v3 slower and
  perhaps more accurate, Qwen3-ASR slow, with the transcript closest to the audio.

---

## 5. OpenWebUI

These facts come from the OpenWebUI image installed on this machine: `open-webui:main`, digest
`sha256:4860ba00…`, version 0.12.3, `backend/open_webui/routers/audio/__init__.py`. How to set
OpenWebUI is in [openwebui.md §5](../openwebui.md#5-speech-to-text).

- **What it sends.** With speech-to-text engine `openai`, OpenWebUI sends
  `POST {base}/audio/transcriptions` with the key as `Authorization: Bearer`. The body is
  multipart by default, with `model` (the configured one), `language` when one is set, and `file`
  under its file name.
- **How it prepares the audio.** A file whose codec is not `flac`, `m4a`, `mp3`, `mp4`, `mpeg`,
  `wav` or `webm` is first converted to MP3 by pydub. A browser's recording, Opus in WebM, is
  converted that way. Over 20 MiB, the file is re-encoded to 16 kHz mono MP3 at 32 kb/s and
  split.
- **The fit.** `whisper-server` reads MP3: through the gateway, `samples/jfk.mp3` with
  `language=en` gives the same text as the WAV. OpenWebUI sends no language unless one is set,
  so the server needs `-l auto` (§2).

---

## Sources

- Phase 0 of the `audio-image-backends` roadmap, whose report records each command, probe and
  measure, with its output; its phase 3 report, for the first dictations through OpenWebUI.
- whisper.cpp at `v1.9.5`: `CMakeLists.txt`, `examples/CMakeLists.txt`,
  `examples/server/server.cpp`, `models/README.md` and `models/download-ggml-model.sh`; and the main
  branch at `231638a0`, fetched on 2026-10-20, for the key.
- The captures of [tests/fixtures/README.md](../../tests/fixtures/README.md), "Audio and Image
  Servers".
- The Hugging Face API, `POST /api/models/ggerganov/whisper.cpp/paths-info/<commit>`, read on
  2026-10-20.
- OpenWebUI 0.12.3, read in its image as cited in §5.
