# Audio: audio.cpp

audio.cpp's `audiocpp_server` serves text-to-speech, voice cloning and speech-to-text on the GPU,
among the many model families of its engine. It reads its own GGUF packages or a publisher's
safetensors. ai-ctl builds it, `ai-ctl build audio.cpp`, and the gateway serves the inventory's
`audiocpp_server` models with it. This document says how it builds, runs, gets ready and answers,
what guards it, and which models it serves. Every fact below was probed on this
machine on 2026-10-20 unless dated otherwise, at release `v0.9.2` (commit `0097d365`,
2026-10-15), with nvcc 13.4.92, g++ 16.2.1 and llama-swap v263.

---

## 1. The Project

- **Repository.** `https://github.com/0xShug0/audio.cpp.git`, a personal account. The repository
  was created on 2026-06-23 and is Apache-2.0, "Copyright 2026 ShugoAI LLC", which GitHub reports
  as NOASSERTION.
- **Releases.** 28 tags. The releases are `vX.Y.Z`, beside suffixed tags
  (`v0.9.2-drama3-base-small`, `v0.8.2-audio8-perf-hotfix`), older `release-0.N` tags and test
  tags. A strict `vX.Y.Z` pattern picks the plain releases.
- **Why it stays whole** (user's decision, 2026-10-20). audio.cpp is kept for benches and
  comparisons, not only for the uses whisper.cpp does not cover. It serves the text-to-speech
  models kept (Qwen3-TTS, Fish Audio S2 Pro and Voxtral 4B TTS) and Qwen3-ASR beside whisper.cpp
  for speech-to-text ([whisper-cpp.md](whisper-cpp.md)).

---

## 2. Build

- **Sources.** ggml is vendored in `external/ggml` (version 0.12.0). The one submodule,
  `external/audio.cpp-server-frontends`, is declared with an SSH URL (`git@github.com:`), so it
  cannot be fetched without an SSH key. It holds optional modules (MP3 and FLAC decoding, an
  HTTPS listener), and the server builds without it.
- **CUDA.** CUDA is `ENGINE_ENABLE_CUDA` and native CPU code `ENGINE_ENABLE_NATIVE_CPU`.
  `CMakeLists.txt` sets `GGML_CUDA` and `GGML_NATIVE` from them with `FORCE`, yet a `GGML_CUDA`
  given on the command line becomes `ENGINE_ENABLE_CUDA`'s default (lines 60–89). Configured with
  the builder's options for llama.cpp alone, the cache holds `ENGINE_ENABLE_CUDA=ON`,
  `GGML_CUDA=ON` and `GGML_NATIVE=ON`, and the log says "Including CUDA backend".
- **`AUDIOCPP_DEPLOYMENT_BUILD=ON`** compiles every `model_specs/*.json` of the built commit into
  the binary (`CMakeLists.txt:16–40`, `docs/gguf.md`).
  - Without it, a model from a safetensors package fails at start, exit 1: "model spec not found
    for family 'qwen3_tts' (provide --model-spec-override, embed it in the GGUF, enable
    AUDIOCPP_DEPLOYMENT_BUILD, or install model_specs/qwen3_tts.json)". The server looks for
    `model_specs/` from its working directory upward.
  - An audio.cpp GGUF carries its spec in `audiocpp.model_spec.*` metadata, and its config and
    tokenizer as embedded sidecars, so one GGUF is the whole model.
  - The probes gave each safetensors model a `model_spec_override`, the source tree's spec at
    `v0.9.2`. The deployment build, which ai-ctl's recipe makes, was probed on 2026-10-28 at
    `v0.9.3`: Qwen3-TTS from Qwen's safetensors, with no `model_spec_override`, started and spoke
    an English and a French sentence.
- **The build.** `cmake --build … --target audiocpp_server --parallel 24`: 1190 steps, 143 CUDA
  objects, 343 s.
  - Warnings come from libstdc++, sentencepiece's protobuf-lite and several model sessions, and
    CMake warns that `external/sentencepiece` asks for a CMake older than 3.10.
  - The binary is `bin/audiocpp_server`, 100.6 MB. Its 142 cubins are all `sm_120a`, with no PTX.
  - Besides the CUDA libraries whisper.cpp links, it links `libcufft.so.12`, for two CUDA
    objects of its own outside ggml (`istft_cuda_runtime.cu`, `torch_random_cuda_runtime.cu`).
- **Version and devices.**
  - `--version` (exit 0) prints `audio.cpp dev`, then `git: 0097d36 2026-10-15`,
    `build: Release, gcc 16.2.1, Linux x86_64` and `backends: cpu,cuda`. It says "dev" on the
    release tag.
  - `--list-devices` (exit 0) prints ggml's CUDA banner on stderr. On stdout it prints
    `available_devices=2`, then one line per device: `CUDA:0 "NVIDIA GeForce RTX 5090 Laptop GPU"
    [GPU]` and `CPU:0 "…" [CPU]`.
  - Both are captured with `--help` in `tests/fixtures/`, as listed in its README.

---

## 3. Serving

- **Configuration.** The models come from a JSON file, `--config <server.json>`. Its keys are
  `host`, `port`, `backend`, `device`, `threads`, `lazy_load`, `max_loaded_models`, `ui`,
  `ui_management` and `voice_dir`, plus `models`. Each model has an `id`, a `family`, a `task`,
  a `mode`, a `path` (a GGUF, or a directory of safetensors), an optional
  `model_spec_override` and its `session_options`. `--host` and `--port` override the file.
  Without `--config`, `lazy_load` is forced on.
- **Options of note.**
  - `--max-loaded-models` keeps at most that many models resident, and `--idle-unload-ms` unloads
    them after a quiet spell.
  - `--min-free-memory-mb` refuses a load that would leave less free memory.
  - `--busy-timeout-ms` answers 503 after a request has been busy for 300000 ms by default.
  - `--cors-origins` is off by default.
  - `--no-ui` turns the web UI off, and `--voice-dir` sets the voice library.
- **Model ids.** A request's `model` reaches the server unchanged, so a model's id in llama-swap
  must equal its id in this file. An unknown id answers 500 "unknown model id: …", a 500 for a
  client's error.
- **Readiness.** A model that is not lazy loads in `ServerState`'s constructor, before the server
  listens (`app/server/runtime.cpp:131`), so nothing answers during the load. Then `GET /health`
  answers 200
  `{"status":"ok","backend":"cuda","models":1,"ui":false,"ui_management":false}`. SIGTERM ends it
  with 0.
- **Routes this build serves** (`--help`, `app/server/request_dispatch.h`,
  `app/server/runtime.cpp`, `app/server/main.cpp:210–240`):
  - `GET /health`, and `GET /v1/models`, which answers each model's host path.
  - `POST /v1/audio/speech` takes JSON `model`, `input` and `voice`, and answers `audio/wav`. It
    also takes `language`, `seed`, the sampling options and `instructions`
    (`app/server/server_protocol.cpp:67–89`, read on 2026-10-23), and a `voice_ref` that names
    a reference voice by a path on the host, as a string or `{"type": "path", "path": …}`,
    read as the server's user (`:176–190`, v0.9.3, read on 2026-11-06).
  - `GET /v1/audio/voices?model=<id>` answers `{"voices":["jfk"]}`, a list of strings.
  - `POST /v1/audio/transcriptions` takes a multipart `file`, `model`, `language`, `prompt` and
    `stream`. A JSON body names the audio instead by `audio`, `audio_path` or `file`, a path on
    the host, read as the server's user (`:1279–1300`, v0.9.3, read on 2026-11-06).
  - `/v1/tasks/run`, `/v1/tasks/batch` and `/v1/tasks/stream`, and the live, batch and alignment
    routes `--help` lists.
- **Routes that are not served.** `--help` lists more than this build serves. The model
  management routes (`/v1/ui/models-root`, `/v1/ui/browse-directories`, install, delete) exist
  only with `AUDIOCPP_HAS_NATIVE_MODEL_MANAGER`, which is off. `/v1/models/load` and `/unload`
  answer 403 without `--ui-management`.
- **Routes open to any client, with no gate:**
  - `/v1/ui/upload` writes the request's file under the server's upload root whenever the UI is
    on, and the UI is on by default with a `--config`;
  - `/v1/ui/path-status`;
  - `/v1/tasks/unload_models` and `/v1/tasks/unload_all_models`.
- **Transcription takes WAV only, judged by the upload's file name**
  (`is_wav_upload_filename`, `app/server/server_protocol.cpp:694`). A WAV sent as `speech-fr.bin`
  got 400 "only WAV audio uploads are currently supported for transcription; MP3 support is
  planned". A transcription request may also name a WAV path on the host. Qwen3-ASR detects the
  language when the request gives none (French and English, probed on 2026-10-21).
- **Voices.** The voice library belongs to the server, not to a family
  (`app/server/server_protocol.cpp:339–375`). A request's `voice` names `<voice_dir>/<voice>.wav`.
  Its transcript is the line `<voice>|<transcript>` of `<voice_dir>/prompt_text`. A name with a
  path in it is refused. A family with embedded presets, such as Voxtral, takes their names
  instead.
- **Behind llama-swap.** Each configuration runs as
  `audiocpp_server --config <file> --no-ui --host 127.0.0.1 --port ${PORT}`, with
  `checkEndpoint: /health`. Probed with Qwen3-TTS; the routes belong to the server, the same for
  every family:
  - `POST /v1/audio/speech` took 4.50 s cold through the gateway, the load included, and a WAV
    came back;
  - `GET /v1/audio/voices?model=<id>` reaches the server;
  - `POST /audioapi/v1/tasks/run` is forwarded as `/v1/tasks/run`;
  - `POST /upstream/<model>/v1/tasks/unload_all_models` unloads the model while llama-swap still
    counts it as running, and the next request reloads it.

---

## 4. The Key

- **No route checks an `Authorization` header**, at `v0.9.2` nor at the head of the main branch
  fetched on 2026-10-20 (`c677d5e`). CORS is off by default: a preflight from
  `https://attacker.example` gets 404. But a JSON body is parsed whatever its `Content-Type`, so
  a `POST` with `Content-Type: text/plain`, which a page sends without preflight, is executed. A
  page can thus run a synthesis on the GPU or `unload_all_models`, without reading the answer.
- **Decided: a patch of ai-ctl's own** (user's decision, 2026-10-20), the same decision as for
  whisper.cpp ([whisper-cpp.md §3](whisper-cpp.md#3-the-key)), with the same options set aside.
  ai-ctl's recipe applies it to every build: `src/domain/patches/audio.cpp/api-key.patch`.
- **The prototype**, against `v0.9.2` (`app/server/request_dispatch.h`, `app/server/main.cpp`,
  +29):
  - the check sits at the top of `RequestDispatcher::handle`, which the legacy and the parallel
    runtimes share;
  - `GET /health` is exempt;
  - header names arrive lower-cased from the HTTP layer;
  - `main.cpp` throws before the models load when `LLAMA_API_KEY` is empty;
  - the body is read before the dispatch, so a request without the key is refused only once its
    body is received, within `max_request_body_bytes`.

  Checked on the server's port with a throwaway key:
  - without the key, `/health` answers 200;
  - without the key or with a wrong one, a `GET`, the speech `POST`, an `OPTIONS` preflight, a
    `text/plain` `POST` from `Origin: https://attacker.example`, `unload_all_models` and
    `/v1/audio/voices` all get 401;
  - with `Bearer` or `X-Api-Key`, the speech answers 200;
  - no `Access-Control-*` header reaches any origin.

  Behind llama-swap, with `env: ["LLAMA_API_KEY=${env.AI_CTL_API_KEY}"]`, the health check
  passes without a key, and a speech and the voice list answer 200 with it.
- **Rebased on `v0.9.3`** (2026-10-28), whose `RequestDispatcher::handle` takes an
  `additional_routes` argument and which both runtimes still call for every request: the same
  added lines, the check still at the top. Probed on the server ai-ctl built: without the key it
  exits 1; without the key or with a wrong one, a `GET` of the voices, the speech `POST`, an
  `OPTIONS` preflight and a `text/plain` `POST` from another origin get 401; with `Bearer` or
  `X-Api-Key`, they pass; no `Access-Control-*` header.

---

## 5. Models

The rule of [stack.md §5](../stack.md#where-models-come-from) applies: the publisher's files when
audio.cpp reads their format, failing that the project's own GGUF conversions in
`audio-cpp/audio.cpp-gguf`. The project's GGUF of Qwen3-TTS is therefore not used, since audio.cpp
reads Qwen's safetensors. Fish Audio publishes `fishaudio/s2-pro` as two safetensors shards and a
`codec.pth`, a pickle, under names audio.cpp's spec does not read. Mistral publishes Voxtral as
`consolidated.safetensors` with voices as `.pt` pickles. Both come from the project's GGUF.

| Directory | Family, task | Repository and commit | Files | Licence |
|---|---|---|---|---|
| `tts/qwen3-tts-1.7b-base` | `qwen3_tts`, TTS with voice cloning | `Qwen/Qwen3-TTS-12Hz-1.7B-Base` `fd4b2543` | the package's eight files, 4.2 GiB | Apache-2.0 |
| `tts/fish-audio-s2-pro` | `fish_audio`, TTS with voice cloning | `audio-cpp/audio.cpp-gguf` `02f5de92` | Q8_0 (6.32 GB) and BF16 (10.23 GB) GGUF | Fish Audio Research License, non-commercial |
| `tts/voxtral-4b-tts` | `voxtral_tts`, TTS with 20 preset voices, no cloning | `audio-cpp/audio.cpp-gguf` `02f5de92` | BF16 GGUF (8.03 GB) | CC-BY-NC-4.0 |
| `stt/qwen3-asr-1.7b` | `qwen3_asr`, speech-to-text | `Qwen/Qwen3-ASR-1.7B-hf` `bcd2b5b7` | seven files, 4.1 GB | Apache-2.0 |

```bash
hf download Qwen/Qwen3-TTS-12Hz-1.7B-Base \
  config.json generation_config.json merges.txt model.safetensors tokenizer_config.json \
  vocab.json speech_tokenizer/config.json speech_tokenizer/model.safetensors \
  --revision fd4b254389122332181a7c3db7f27e918eec64e3 \
  --local-dir "$MODELS_DIR/tts/qwen3-tts-1.7b-base"
hf download audio-cpp/audio.cpp-gguf \
  Fish-Audio-S2-Pro-GGUF/fish-audio-s2-pro-q8_0.gguf Fish-Audio-S2-Pro-GGUF/fish-audio-s2-pro-bf16.gguf \
  --revision 02f5de92a82339ebb722ea6fdadc14b7bec43a7f \
  --local-dir "$MODELS_DIR/tts/fish-audio-s2-pro"
hf download audio-cpp/audio.cpp-gguf \
  Voxtral-4B-TTS-2603-GGUF/voxtral-4b-tts-2603-bf16.gguf \
  --revision 02f5de92a82339ebb722ea6fdadc14b7bec43a7f \
  --local-dir "$MODELS_DIR/tts/voxtral-4b-tts"
hf download Qwen/Qwen3-ASR-1.7B-hf \
  chat_template.jinja config.json generation_config.json model.safetensors processor_config.json \
  tokenizer.json tokenizer_config.json \
  --revision bcd2b5b7f32b480ab5790554cfa8347f246a14f3 \
  --local-dir "$MODELS_DIR/stt/qwen3-asr-1.7b"
```

Every file was checked on 2026-10-20 or 2026-10-21. Each LFS file's SHA-256 equalled the Hub's
`lfs.oid` at the commit above, and each small file's git blob SHA-1 equalled its `oid`.

- **Qwen3-TTS** is the eight files of audio.cpp's package `qwen3_tts_1_7b_base_safetensors`,
  exactly what Qwen publishes. It clones a voice, and a request without a reference fails with
  500 "Qwen3 base TTS requires voice clone reference audio". The session option
  `qwen3_tts.weight_type` quantizes at load (`native`, `f32`, `f16`, `bf16`, `q8_0`;
  `src/framework/model_spec/options.cpp:14`). Both `native` and `q8_0` are kept, as two entries
  over the same files. A request's `language` reaches the talker, `Auto` when the request gives
  none, and an `options.language` overrides it (`src/models/qwen3_tts/session.cpp:643–645`).
- **Fish S2 Pro and Voxtral** each run from one GGUF, with the spec embedded and no
  `model_spec_override`. Fish is kept at both precisions, Q8_0 and BF16; Voxtral in BF16 alone.
- **Fish S2 Pro's inline tags.** `model_specs/fish_audio.json` announces "inline prosody/emotion
  controls" without their syntax. The model card of `fishaudio/s2-pro` at `1de9996b` gives it:
  free-form descriptions in brackets, such as `[excited]`, `[emphasis]`, `[short pause]`,
  `[chuckle]` and `[delight]`. Sent with the request's `options.text_chunk_mode` `tag_aware`, so
  that no chunk cuts a tag, no tag was spoken as a word (2026-10-23).
- **Voxtral's 20 presets** are embedded in the GGUF (`model_specs/voxtral_tts.json`: `voice_id`,
  default `casual_male`; `fr_male` and `fr_female` among them). It speaks en, fr, es, de, it, pt,
  nl, ar and hi. With no `seed` in the request, the acoustic seed is random, so two runs of one
  sentence differ.

**Measured** with each server alone on the GPU, which held 15 MiB before. Each start was cold,
after `POSIX_FADV_DONTNEED` on the weights, with `lazy_load` false.

The text-to-speech request was one English and one French sentence:
- the cloning models took JFK's English recording as the reference voice, a placeholder until the
  user's;
- Voxtral took `neutral_female` for the English and `fr_female` for the French.

The answer is a 16-bit mono WAV, at 24 kHz for Qwen3-TTS and Voxtral and 44.1 kHz for Fish.

| Model, weights | Ready, cold | Peak VRAM | English | French |
|---|---|---|---|---|
| Qwen3-TTS, native | 2.67 s | 6025 MiB | 3.13 s for ~8 s of audio | 2.04 s for ~10 s |
| Qwen3-TTS, `weight_type` `q8_0` | 5.89 s | 4759 MiB | 2.79 s | 1.89 s |
| Fish S2 Pro, Q8_0 | 6.78 s | 7953 MiB | 4.59 s for ~9.5 s | 4.56 s for ~10 s |
| Fish S2 Pro, BF16 | 12.26 s | 11199 MiB | 6.36 s | 5.97 s |
| Voxtral 4B TTS, Q8_0 (2026-10-21), not kept | 2.36 s | 5305 MiB | 2.07 s for 11.1 s | 1.16 s for 7.8 s |
| Voxtral 4B TTS, BF16 (2026-10-21) | 3.92 s | 8459 MiB | 2.64 s for 10.5 s | 1.44 s for 6.7 s |

Qwen3-ASR 1.7B was ready in 2.35 s, cold, with a peak of 4903 MiB. It transcribed JFK's 11 s in
0.47 s and a French WAV of 9.5 s in 0.31 s, both word for word. Its comparison with whisper on
the user's recordings, and the use each reader keeps, are in
[whisper-cpp.md §4](whisper-cpp.md#4-models).

**The outputs read back** by whisper large-v3, large-v3-turbo and Qwen3-ASR, each request with its
language. This is a reading by machine of synthetic speech: it ranks the outputs, not the
readers, and the ear decides. The French sentence was "Le vif renard brun saute par-dessus le
chien paresseux. Ceci est un essai de synthèse vocale locale sur une seule carte graphique."

- Every English output was read word for word, except "A quick" for "The quick" in turbo's
  reading of Voxtral Q8_0.
- Voxtral's French, at both precisions, was read word for word by all three readers.
- Fish BF16's French was read exactly by Qwen3-ASR, and as "Les vifs renards bruns sautent…" by
  both whisper models.
- Fish Q8_0's French first sentence was misread by all three readers.
- Qwen3-TTS's French was poor at both precisions, for large-v3-turbo and Qwen3-ASR alike.

**Compared on the user's voice**, from 2026-10-21 to 2026-10-28, each server alone on the GPU. The
reference of both cloning models is the user's French recording, 37 s, converted to 16-bit mono at
44.1 kHz and taken whole, since neither family's spec sets a limit on its length. With it,
Qwen3-TTS's French is intelligible where JFK's reference made it poor. Voxtral took `fr_male` and
`fr_female` in French, `neutral_male` and `neutral_female` in English. The text was one French and
one English paragraph, each with a date and an hour. A request without `seed` takes a random one,
so each output is one draw:

| Model, weights | French: time for audio | English: time for audio |
|---|---|---|
| Qwen3-TTS, native | 14.5 s for 17.3 s | 6.7 s for 21.4 s |
| Qwen3-TTS, `q8_0` | 13.3 s for 16.3 s | 4.9 s for 18.6 s |
| Fish S2 Pro, Q8_0 | 10.9 s for 18.6 s | 10.0 s for 19.6 s |
| Fish S2 Pro, BF16 | 13.5 s for 17.3 s | 13.4 s for 19.6 s |
| Voxtral 4B TTS, Q8_0, the male voices | 3.7 s for 20.2 s | 3.1 s for 20.0 s |
| Voxtral 4B TTS, BF16, the male voices | 4.9 s for 19.0 s | 4.4 s for 19.9 s |

Qwen3-TTS's French times above are each the first request after a start. On 2026-10-23, on the
seeds 11, 22 and 33 at each precision, that first request took 8.8 to 9.6 s for 17 s of French.
After it, the French took 3.6 to 4.4 s for 17 to 18 s, and the English 3.5 to 4.6 s for 16 to
20 s: warm, Qwen3-TTS is as fast as Voxtral BF16. Fish S2 Pro Q8_0 with its tags took 9.9 to
10.9 s for 18.6 to 22.0 s.

**The peaks the inventory budgets**, measured on 2026-10-29 through ai-ctl's gateway, every model
unloaded before each request, the GPU's memory sampled every 100 ms. The peaks above were taken
with JFK's 11 s as the reference and as the audio; behind the gateway, with a reference of
40.6 s, the longest of the user's voice library, Fish S2 Pro Q8_0 beside Qwen3-TTS and Voxtral
answered 500 "failed to allocate Fish DAC codec encode graph". The French text was one
paragraph of 224 characters, then the same four times:

| Model | Request | Peak, one paragraph | Peak, four |
|---|---|---|---|
| Qwen3-TTS, native | reference of 40.6 s | 6700 MiB | 6914 MiB |
| Qwen3-TTS, `q8_0` | reference of 40.6 s | 5328 MiB | 5542 MiB |
| Fish S2 Pro, Q8_0 | reference of 40.6 s | 10310 MiB | 10310 MiB |
| Fish S2 Pro, BF16 | reference of 40.6 s | 12768 MiB | 12768 MiB |
| Voxtral 4B TTS, BF16 | `fr_female` | 8444 MiB | 8446 MiB |

The reference weighs on Fish, whose codec encodes it whole, and the text a little on Qwen3-TTS.
Qwen3-ASR reads audio in chunks of 30 s at most (`audio_chunk_duration_sec`,
`model_specs/qwen3_asr.json`): it peaked at 5548 MiB on 60 s of French as on 180 s, where
whisper large-v3 stayed at 3896 MiB. Each budget of the inventory is the larger peak.

The user's verdicts by ear, on 2026-10-23 and 2026-10-28:
- **Qwen3-TTS** is the most natural in French, strikingly realistic at its best, at both
  precisions. The seed changes the delivery more than the precision does: the best output of each
  precision came from a different seed. Every French output says "vingt et un heure" for "vingt
  et une heure". In English, with the French reference, it keeps a French accent and says "2026"
  in French in six outputs of six, `language` `en` included; an English reference was not
  probed.
- **Fish S2 Pro** copies the voice well, but its delivery is slow and empty of emotion at both
  precisions. With its tags, its English gains a real emotion and comes well above Qwen3-TTS's;
  its French stays flat.
- **Voxtral 4B TTS** has little dynamism, little emotion and less personality on its presets. Its
  BF16 is a little better than its Q8_0; read back by whisper, it renders the date and the hour,
  where the Q8_0's `fr_female` read "21 arches 45". It is the only model with voices ready
  without a reference recording.
- A reading back by machine judges the numbers badly: whisper wrote the hour "21h45" in the
  French outputs where the ear hears "vingt et un heure".

---

## 6. OpenWebUI

These facts come from the OpenWebUI image installed on this machine, version 0.12.3
(`backend/open_webui/routers/audio/__init__.py`). Its speech-to-text requests are in
[whisper-cpp.md §5](whisper-cpp.md#5-openwebui). How to set OpenWebUI is in
[openwebui.md §6](../openwebui.md#6-text-to-speech).

- **Text-to-speech, engine `openai`.** OpenWebUI sends `POST {base}/audio/speech` with JSON
  `model`, `voice`, `input` and the configured parameters, and the key as a bearer token. Any
  audio content type comes back and is transcoded to MP3.
- **Model and voice lists.** The model list (`GET {base}/audio/models`, then `{base}/models`) and
  the voice list (`GET {base}/audio/voices`, without `?model=`) are requested without the key.
  Through the gateway they get 401, and OpenWebUI falls back to `tts-1`, `tts-1-hd` and OpenAI's
  six voices, so the model and the voice are typed by hand. audio.cpp's voice list is a list of
  strings, where OpenWebUI reads `voices[].id` and `voices[].name`.
- **Speech-to-text does not fit.** OpenWebUI sends MP3 or WebM, and audio.cpp's transcription
  takes WAV only (§3). Qwen3-ASR cannot serve OpenWebUI as the server stands; whisper-server can.

---

## Sources

- Phase 0 of the `audio-image-backends` roadmap, whose report records each command, probe and
  measure, with its output.
- audio.cpp at `v0.9.2`: `CMakeLists.txt`, `docs/gguf.md`, `docs/model_licenses.md`,
  `model_specs/qwen3_tts.json`, `model_specs/fish_audio.json`, `model_specs/voxtral_tts.json`,
  `model_specs/qwen3_asr.json`, `app/server/main.cpp`, `app/server/request_dispatch.h`,
  `app/server/runtime.cpp`, `app/server/server_protocol.cpp` and
  `src/models/qwen3_tts/session.cpp`; and the main branch at `c677d5e`, fetched on 2026-10-20,
  for the key.
- The model card of `fishaudio/s2-pro` at `1de9996b`, for Fish's inline tags, read on
  2026-10-23.
- The captures of [tests/fixtures/README.md](../../tests/fixtures/README.md), "Audio and Image
  Servers".
- The Hugging Face API, `/api/models/<repo>` and `POST /api/models/<repo>/paths-info/<commit>`,
  read on 2026-10-20 and 2026-10-21, with each repository's model card for its licence.
- OpenWebUI 0.12.3, read in its image (digest `sha256:4860ba00…`).
