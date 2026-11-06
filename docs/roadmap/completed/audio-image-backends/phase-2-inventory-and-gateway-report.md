# Phase 2 Report: Inventory And Gateway

**Phase:** [phase-2-inventory-and-gateway.md](phase-2-inventory-and-gateway.md)
**Start Commit:** `docs(audio-image-backends): close phase 1` of 2026-10-28

---

## Work Log

### 2026-10-28

Before task 1, how each server reads its options, at the builds phase 1 made, so that each
inventory check is written for the parser it guards:
- `whisper-server` (`v1.9.5`, `examples/server/server.cpp:225–270`) compares each argument to
  its short and long names exactly. An unknown one prints "error: unknown argument" and exits 0.
- `sd-server` (`master-1024-77b72a2`, `parse_options`, `examples/common/common.cpp:262`)
  compares exactly too. Several of its long names hold `_`: `--clip_l`, `--clip_g`,
  `--clip_vision`, `--llm_vision`, `--qwen2vl_vision` (`--help` at `master-975`).
- `audiocpp_server` (`v0.9.3`, `app/server/main.cpp:35–50`) looks each option up by scanning
  the whole command line for the first word equal to its name. It ignores unknown words, and
  `--ui` anywhere turns the UI on.

So the rule against `_` in a flag is llama-server's alone, which reads each `_` as `-`: for the
three new servers, a flag's name reaches them as written, and the reserved names are checked as
written. The rule against a text value starting with `-` holds for all three.

Tasks 1 and 3, and task 2's inventory side, in `src/domain/inventory.rs`. Each new test was
watched failing on its assertion first, against kinds added with empty rules:
- `BackendKind` gains `WhisperServer`, `SdServer` and `AudiocppServer`, named
  after their binaries, and `project()`, whose recipe's binary is the kind's name (tested for
  the four).
- The reserved flags:

  | Kind | Reserved |
  |---|---|
  | `whisper-server` | `host`, `port`, `model`, `inference-path`, `request-path` (which would move `/health`) |
  | `sd-server` | `listen-ip`, `listen-port`, `eager-load`, `model`, `diffusion-model` |
  | `audiocpp_server` | `config`, `host`, `port`, `no-ui`, `ui`, `ui-management` |

- The files: a `model` for `whisper-server`. For `sd-server`, a `model` or a
  `diffusion-model` (`MissingFile` now names the files it wants one of). For
  `audiocpp_server`, none: it may name `model` alone (`UnusedFile` otherwise), and its files
  are not flags.
- `ctx_size` is refused for the three (`NotForKind`).
- `UnderscoredFlag` is llama-server's alone, so `sd-server`'s `clip_l` passes.
- `OptionValue` names the server that would misread the value.
- Task 3: a `build` source must be the project whose server the kind is (`SourceKind`), which
  also refuses a `llama-server` backend built from `whisper.cpp`, accepted until now. A project
  with no recipe is left to `doctor`. An `image` source is refused for the new kinds
  (`ImageKind`).
- The gateway's configuration refuses the three kinds (`ConfigError::NotServed`) until tasks 4
  to 6 give each its command. Without that refusal, a `whisper-server` model got llama-server's
  command and `[defaults.llama-server]`: the new test, watched failing, shows it.
- Not done in task 2 yet: the control characters, checked when the configuration is written,
  wait for each kind's command.
- `README.md`'s inventory table: a `build` names the project that builds the kind's server. The
  rest of the table, and `docs/commands.md`, follow once the gateway serves the kinds (task 12).
- fmt, clippy and the tests green (761, 27 and 3). `cargo mutants --in-diff`: 35 mutants, 32
  caught, 3 unviable. A test added afterwards, `a_number_value_must_be_finite_for_every_server`,
  was watched failing under a hand-made mutation that kept the finite check to `llama-server`.

Committed as `feat(inventory): add the whisper, sd and audio.cpp server kinds`.

Tasks 4 and 5 for `whisper-server` and `sd-server`, in `src/domain/gateway_config.rs`, from
phase 0's hand-written configuration (`$P/swap/config.yaml`), which ran both behind llama-swap
v263. The four new tests were watched failing on `NotServed` first:
- `whisper-server`: `--host 127.0.0.1 --port ${PORT} --inference-path /v1/audio/transcriptions`,
  its files, its weights first, then its flags. The flags are ai-ctl's own defaults,
  `language = "auto"` (Decisions), overlaid with `[defaults.whisper-server]`, then the model's.
- `sd-server`: `--listen-ip 127.0.0.1 --listen-port ${PORT} --eager-load`, its weights first,
  `model` or `diffusion-model`, then its other files and flags. Each kind takes its own
  defaults: llama-server's never reach it (tested).
- Both: `checkEndpoint: /health`, the key as `LLAMA_API_KEY` through `env:`, and no
  `disableKeepAlives`, which works around llama-server alone. Phase 0 ran both with keep-alive.
- `llama_server` now shares `files` and `flags` with them; its command is unchanged, its tests
  untouched.
- Task 2's control characters: a whisper flag holding a line feed and a sd file holding U+2028
  are refused when the configuration is written.
- `NotServed` stays for `audiocpp_server` until task 6.
- fmt, clippy and the tests green (765, 27 and 3). `cargo mutants --in-diff`: 28 mutants, 27
  caught, 1 unviable.

Committed as `feat(gateway): serve whisper-server and sd-server models`.

Task 6 starts with what audio.cpp reads, at `v0.9.3` (`load_server_config`,
`app/server/config.cpp:236–390`):
- a model needs `id`, `family` and `path`; `task` and `mode` have defaults;
- `session_options` is an object whose values become strings (`json_options_map`): strings,
  numbers and booleans alike;
- `voice_dir` belongs to the server. Without it, and with the UI off, the server has no voice
  library, so a request naming a voice fails; Voxtral's presets are in its GGUF.

The two choices this asked for are under Decisions: `voices` on the model, and the JSON in the
YAML as a comment.

The model's fields, first part of task 6:
- `Model` gains `audiocpp: AudiocppFields`: `family`, `task`, `mode`, `session_options` (typed
  as flags are) and `voices`.
- The inventory file reads them as the model's own keys, and `[models.session_options]` as a
  table. A nested value is refused as a flag's is.
- Checks:
  - an `audiocpp_server` model needs a non-empty `family` (`MissingField`);
  - another kind takes none of the five fields (`NotForKind`);
  - `voices` stays inside `models_dir`, as `dir` does;
  - a session option is a finite number, which JSON can write.
- Each new test was watched failing on its assertion first.
- fmt, clippy and the tests green (770, 27 and 3). `cargo mutants --in-diff`: 8 mutants,
  5 caught, 3 unviable.

Committed as `feat(inventory): read an audio.cpp model's family, options and voices`.

Task 6's second part, which ends tasks 2, 4 and 5 for `audiocpp_server`:
- Domain:
  - `ModelEntry` gains `file: Option<ConfigFile>`, a path and its content.
  - `generate` takes the gateway's directory and puts each `audiocpp_server` model's file in
    `<dir>/audiocpp/<model>.json`.
  - The command: `--config <file> --no-ui --host 127.0.0.1 --port ${PORT}`, then the flags.
  - The JSON, written with `serde_json`, which does no I/O:
    - `lazy_load: false`, so that ready means loaded, and `ui: false`;
    - `voice_dir` when the model names `voices`;
    - one model: its `id`, `family`, `path` (its `model` file, or its directory), `task`,
      `mode` and `session_options`, typed.
  - Every text in it goes through the YAML's checks, since the YAML carries the file as a
    comment above the model's entry (Decisions).
  - `NotServed` is gone.
- Application:
  - `GatewayProcess` gains `files_dir()`, the gateway's directory, and `start` takes the files.
  - `Configurator` takes `files_dir`, from each use case's gateway port.
  - `up` passes `config.files()`.
- Infrastructure: `LocalGateway::start` writes each file by a rename before the configuration,
  in a 0700 directory, and refuses a path outside its directory.
- Tests:
  - the four domain tests were watched failing on `NotServed` first;
  - the adapter's two tests and the `up` test were watched failing under hand-made mutations:
    no file written, no directory check, no file passed;
  - the two adapter tests spawn a process: 30 runs, no failure.
- fmt, clippy and the tests green (776, 27 and 3). `cargo mutants --in-diff`: 21 mutants,
  6 caught, 15 unviable.

Run on this machine, from the working tree, with `XDG_CONFIG_HOME` and `XDG_STATE_HOME` in a
scratch folder. The inventory held `whisper-large-v3-turbo`, `flux-3-image-klein`,
`qwen3-tts-q8` (`weight_type` `q8_0`, no voices), `voxtral-4b-tts` and `qwen3-asr-1.7b`, on
`127.0.0.1:18080`:
- `ai-ctl gateway config`, then llama-swap v263's `-validate`: "Config is valid: 5 model(s)".
  No key in the file.
  - `-validate` refuses the file unless `AI_CTL_API_KEY` is set, "environment variable
    'AI_CTL_API_KEY' is not set": `docs/commands.md`'s example lacked it since the key went to
    the environment. Fixed.
- `ai-ctl up` wrote the three JSON files under `gateway/audiocpp/`, mode 0700 directory, and
  started the gateway. Through it, with the key:

  | Model | Request | Answer |
  |---|---|---|
  | `whisper-large-v3-turbo` | `jfk.wav` | 200 in 1.50 s, the sentence word for word |
  | `voxtral-4b-tts` | French sentence, `fr_female` | 200 in 5.34 s, a 24 kHz mono WAV |
  | `qwen3-asr-1.7b` | that WAV | 200 in 3.65 s, the sentence word for word |
  | `qwen3-tts-q8` | `voice` `jfk` | 500, "Qwen3 base TTS requires voice clone reference audio": loaded, with no `voice_dir` |
  | `flux-3-image-klein` | 512×512 | 200 in 8.31 s, a PNG; llama-swap stopped the three audio models first |

  Without the key, 401 from llama-swap. `ai-ctl down` stopped it, 15368 MiB released, no
  process left.
- Documentation:
  - `README.md`'s inventory table: the four kinds, their files, the `audiocpp_server` fields
    and each kind's reserved flags;
  - `docs/commands.md`: `gateway config`, the validation example and the files table;
  - `docs/security.md` §2: the new servers on loopback and their reserved flags. Its inventory
    section: the configuration's texts are checked as the YAML's are.
  - Task 12 waits for `doctor`'s part.

Committed as `feat(gateway): serve audio.cpp models from a configuration up writes`. Session closed there, nothing uncommitted.

### 2026-10-29

Outside this phase's tasks: Fibo 2.0 Dev, Bria's image model, probed as a third image model at
the user's request. Its facts and the decision are in `docs/image/stable-diffusion-cpp.md` §4;
the commands and outputs here.
- sd.cpp supports it since `2e2ba3f` (2026-10-21, PR #2180), in `master-1024-77b72a2`, the build
  ai-ctl uses. PRs #2216 (text drawn as glyph images) and #2215 (masked editing) were open.
- The user downloaded `briaai/Fibo-2.0-dev` at `72415bd0` (a first try failed on an unset
  `$MODELS_DIR`, nothing written). The six files checked against
  `POST /api/models/briaai/Fibo-2.0-dev/paths-info/72415bd0…` with the token: the two shards and
  the VAE by SHA-256 (`a7c0121b…`, `0edf2f2d…`, `d64f3a68…`), the two `config.json` and the index
  by git blob SHA-1, all equal. FLUX.3 Image Klein's `Qwen3VL-4B-Instruct-Q8_0.gguf` hard-linked
  into `image/fibo-2.0-dev/` (inode 4570, two links).
- Cold start with `bin/probe-ready.sh` and `bin/req-image.sh` of `~/.cache/ai-ctl-probes/`, after
  `POSIX_FADV_DONTNEED` on each file: `sd-server --eager-load --diffusion-model
  …/transformer/diffusion_pytorch_model.safetensors.index.json --vae
  …/vae/diffusion_pytorch_model.safetensors --llm …/Qwen3VL-4B-Instruct-Q8_0.gguf --cfg-scale 5.0
  --sampling-method euler --steps 50 --negative-prompt-file negative-t2i.json` (Bria's
  `NEGATIVE_T2I`). The log: "load … using safetensors index format", "Version: Fibo-2.0",
  `--auto-fit` placing all three parts on `CUDA0`, 19750.59 MB of weights. Ready in 6.77 s at
  20060 MiB, peak 23520 MiB; the fox in 114.6 s, clean, "ai-ctl" legible.
- `compare-fibo.sh` (kept with its outputs in `~/.cache/ai-ctl-probes/compare-2026-10-29-fibo/`,
  with a page beside them): the four prompts of 2026-10-21 and two counted ones, apples and cups,
  at 50 and 20 steps; Qwen-Image 2.1 and FLUX.3 Image Klein on the two counted ones. Every request
  200. Times: Fibo 114.9–115.3 s at 50 steps, 46.4–46.7 s at 20; Qwen-Image 29.0 s; FLUX
  9.8–10.1 s. Counts: Fibo 6 apples and 7 cups, both in two rows; Qwen-Image and FLUX 4 apples,
  5 cups in a row. Fibo's poster: "Vint-quatre"; its lighthouse signed "Leighhouse".
- The user's verdict by eye: Fibo's rendering above the two others', at 50 steps more than at 20.
- Sizes, 4 steps, one server per size: with the encoder on the GPU, 1024×1024 23520 MiB,
  1248×832 23488, 1424×800 23808; 1536×1024 aborts in the VAE's decode, `[E] ggml: CUDA error:
  out of memory` in `cuMemCreate`, exit 134, twice; 1536×1536 passed once at 23600. With
  `--vae-tiling`, 1536×1024 at 21458 MiB and 2048×2048 at 23168, but the tiled 1024×1024 is
  darker and warmer than the whole one (PSNR 25.3 dB). With `--backend te=cpu` (the log:
  `--params-backend "te=cpu"`, 4076.43 MB in RAM): 1024×1024 19440 MiB, the encoding 3.99 s;
  1536×1024 21104; 1536×1536 23600; 2048×2048 aborts. The portrait at 50 steps with the encoder on
  the CPU: 118.8 s, the same image as on the GPU but for its details.

Task 7, the ports:
- llama-swap v263's source (`internal/config/macros.go` at `e33e97b`): when the configuration
  loads, each model whose `cmd` holds `${PORT}` gets the next port from `startPort`, 5800 by
  default, the ids sorted in byte order. Phase 0's log agrees. ai-ctl passes no
  `-watch-config`, so the ports change at an `up` only.
- `probe-timewait-cross.sh <leaver> <starter> <port> [keepalive|close]`, in
  `~/.cache/ai-ctl-probes/bin/` (untracked): a server answers one request, then is stopped with
  the connection still open. A server of another kind then starts on the port.
  - Its first version read only the headers of the answer. `whisper-server` sends its body
    apart, so the client's close sent a RST and left no TIME-WAIT. Three runs gave no result and
    were run again.

  | Left by | Started | TIME-WAIT on the port | Result |
  |---|---|---|---|
  | `llama-server` | `whisper-server` | 23, 12 | ready 0.8 and 0.6 s after the stop |
  | `whisper-server` | `sd-server` | 1, 1 | ready 0.6 s after |
  | `sd-server` | `audiocpp_server` | 1, 1 | ready 0.2 s after |
  | `audiocpp_server` | `llama-server` | 3, 3 | ready 3.1 s after |
  | `audiocpp_server`, `whisper-server`, `llama-server` | phase 0's `whisper-server` (key patch alone) | 3, 1, 13 | exit 1, "couldn't bind to server socket" |

  Each pair was run twice: once with a keep-alive connection left idle, once with a
  `Connection: close` request.
- Through ai-ctl, with `XDG_CONFIG_HOME` and `XDG_STATE_HOME` in the session's scratch folder
  and the gateway on `127.0.0.1:18080`:
  - Inventory A, by the gateway's log: `qwen3-asr-1.7b` on 5800, `qwen3.6-35b-a3b` on 5801,
    `whisper-large-v3-turbo` on 5802. Each was requested and answered 200; then `down`.
  - Inventory B is A plus `flux-3-image-klein`, whose id sorts first: it got 5800,
    `qwen3-asr-1.7b` 5801, `qwen3.6-35b-a3b` 5802.
  - `up` started 0.4 s after `down`, then:

  | Model | Its port left by | TIME-WAIT before the request | Answer |
  |---|---|---|---|
  | `qwen3.6-35b-a3b` | `whisper-server` | 2 | 200 in 3.5 s |
  | `flux-3-image-klein` | `audiocpp_server` | 5 | 200 in 8.3 s |
  | `qwen3-asr-1.7b` | `llama-server` | 6 | 200 in 2.0 s |

- Every server started on a port another server had left, so the inventory keeps no guard, as
  the task allows when none fails. No code change.
- Documentation:
  - `docs/gpu/llama-swap.md` now has the facts about the ports;
  - `docs/security.md` §2 said a model starts "on the next port from 5800". Its port is fixed
    by the configuration: corrected.

Committed as `docs: record how ports move across a restart and bind over TIME-WAIT`.

Task 8, `sd-server`'s model routes, read at `master-1024-77b72a2`
(`examples/server/routes_models.cpp`, `model_catalog.cpp`), then probed through ai-ctl's
gateway with `flux-3-image-klein` alone in the inventory:
- llama-swap answers 404 to `POST /sdcpp/v1/models/load` and to
  `GET /sdcpp/v1/models/status?model=…`: a client reaches these routes by `/upstream/<model>/…`
  only.
- A load names a recipe, and files of the catalog for its slots. The catalog holds the files
  under `--model-dir`, by default `models` in the current directory. That directory is
  llama-swap's, the gateway's, which holds no `models`:
  - `model-files` answered `"files":[]`;
  - a load naming Qwen-Image's path: "unknown model or incorrect category";
  - each of the 108 builtin recipes: 84 "missing required model: diffusion_model" and 24 "…:
    model";
  - FLUX still made an image after them.
- `unload` answered 200, `"state":"empty"`. VRAM went from 15392 to 320 MiB. `/health` still
  answered 200 and `/running` still said ready, but the next image got 503
  `model_not_loaded`, and no load can bring the model back. The VRAM stays under what the matrix
  counts. Left open to a key holder, as llama-swap's own unload is: `docs/security.md` §3 says
  so.
- `model-dir` is now reserved for `sd-server`: with it, a load could bring any model of that
  directory into the server, in place of the inventory's and outside the matrix. The test was
  watched failing on its assertion first: the inventory accepted the flag. `lora-model-dir`
  stays allowed: a LoRA's weights add to the model's, and the inventory's author counts them in
  `vram_mib`.
- Documentation:
  - `docs/image/stable-diffusion-cpp.md` §2: the routes behind the gateway;
  - `docs/security.md` §3: the unload, under what a key holder does;
  - `README.md`'s inventory table: the new reserved flag.
- fmt, clippy and the tests green (776, 27 and 3). `cargo mutants --in-diff`: 3 mutants,
  3 caught.

Committed as `fix(inventory): reserve sd-server's model-dir flag`. The user, told that the unload was left open, asked to go on to task 9.

Task 9, the matrix and the snapshot:
- `matrix` reads no kind: it already sizes every model by `vram_mib`. So no code changed:
  - a domain test, `the_matrix_sizes_the_models_of_every_kind`, puts whisper beside FLUX and
    beside two TTS models, and leaves the 27B alone. It passes on the code, and was watched
    failing on its assertion under a hand-made mutation that kept only the `llama-server`
    models in the matrix (`["qwen"]`);
  - `tests/gateway_config.rs`: each project gets its own build path; until now, every build was
    llama.cpp's. The shipped snapshots are unchanged.
- `gateway-every-kind.yaml`, read once:
  - the inventory is the shipped one plus 10 models: two whisper, five audio.cpp (Qwen3-TTS
    twice, by `weight_type`), and FLUX, Qwen-Image and Fibo, at their measured peaks;
  - the matrix has 27 sets;
  - Fibo's `--backend te=cpu` arrives as written;
  - the `voice_dir`, the JSON paths and `--threads 4` are as `up` writes them.
- Live:
  - The same inventory with this machine's builds: `ai-ctl gateway config`, then llama-swap
    v263's `-validate`: "config is valid: 12 model(s)". No key in the file.
  - Task 7's inventory B through the gateway:

    | Requests | `/running` | VRAM |
    |---|---|---|
    | FLUX, then whisper | both (`fit-3`) | 17587 MiB |
    | then Qwen3-ASR | Qwen3-ASR and whisper (`fit-2`): FLUX evicted | 7095 MiB |

- fmt, clippy and the tests green (777, 27 and 4). `cargo mutants --in-diff`: no mutant, the
  diff holding tests only.

Committed as `test(gateway): snapshot an inventory with several models of each kind`.

Task 10, `doctor` for every kind:
- The build checks have been per project since phase 1: each recipe's smoke test, failing for
  a project the inventory runs. Nothing to change. On this machine, the four builds are ok.
- The model check read only `files`. That passed an `audiocpp_server` model that names none,
  whose directory audio.cpp reads whole, and its `voices`. Now:
  - a model that names no file needs its directory: missing, it fails with
    `hf download <repo> --revision <commit> --local-dir <dir>`, or "copy the model's files into
    <dir>" without a repository;
  - a model's `voices` needs its directory: missing, it fails with how audio.cpp reads a voice,
    `<voice>.wav` and its line in `prompt_text`;
  - several missing items are listed in one check, separated by `;`.
- `ToolchainProbe` gains `is_dir`, beside `is_file`.
- Tests:
  - the three new doctor tests were watched failing on their assertion first;
  - `cargo mutants --in-diff` missed 3 mutants: the no-repository directory's remedy, and
    `is_dir` returning `true` or `false`. Killed by two tests: the remedy, and a host test that
    tells a file from a directory, `is_file` included.
  - Second run: 12 mutants, 11 caught, 1 unviable.
- Live, task 9's inventory: Qwen3-ASR passed as "the directory /ai/models/stt/qwen3-asr-1.7b".
  The three models with `voices = "voices"` failed, since `/ai/models/voices` does not exist
  here.
- Seen, left open: Fibo's `diffusion-model` is a safetensors index. `doctor` checks the index
  file, not the two shards it names, so a missing shard passes `doctor` and fails at the
  server's load.
- `README.md`'s doctor table: the Inventory row.
- fmt, clippy and the tests green (782, 27 and 4).

Committed as `feat(doctor): check a model's directory and its voices`. The session closed before task 11 began. Nothing is left
uncommitted. The probes of the day are in `~/.cache/ai-ctl-probes/bin/probe-timewait-cross.sh`
(untracked).

Session resumed. `status` and `logs` read no kind: the gateway's processes are the GPU's
processes that descend from llama-swap, the leftovers the programs under the build store, every
project's, the models those of `/running`, and `logs <model>` streams `/logs/stream/<model>`.
The user chose that `doctor` read a safetensors index in this phase (Decisions): a task added
after task 11.

Task 11, `status` and `logs` for every kind:
- No code changed. Two application tests put a server of each kind on the GPU, from its
  project's build:
  - `status_counts_a_server_of_every_kind_among_the_gateways_processes`: children of the
    gateway, all four in `held`;
  - `status_lists_a_server_of_every_kind_a_killed_gateway_left`: children of `systemd --user`,
    all four in the leftovers.
  - Both pass on the code. Each was watched failing on its assertion under a hand-made
    mutation that kept only the processes named `llama-server`: one in `held`, one in the
    leftovers.
- Live, with `probe-status-logs.sh` (in `~/.cache/ai-ctl-probes/bin/`, untracked; its outputs in
  `~/.cache/ai-ctl-probes/logs/status-logs-2026-10-29/`). The state was in the session's
  scratch folder, and the gateway ran on `127.0.0.1:18080`. The inventory was the installed
  one, `preload` off, plus `whisper-large-v3-turbo`, `qwen3-asr-1.7b`, `voxtral-4b-tts` and
  `flux-3-image-klein`. One request per kind, each prompt holding a marker and an ESC, every
  answer 200:

  | After | `status`'s models | GPU memory, the models' share |
  |---|---|---|
  | whisper, Qwen3-ASR, Voxtral | the three, ready | 15535 MiB, 15502 by the models |
  | FLUX | FLUX and whisper | 17586, 17558 |
  | Qwen3.6-35B-A3B | it alone | 19899, 19876 |

  `down` released 19876 MiB; `status` then said stopped, exit 3, no leftovers.
- `logs <model>` streamed each kind's output, and `logs nope` failed with "the gateway serves no
  model". The gateway's log holds no prompt, answer or transcript, of any kind, and no marker.
  What each server writes there:
  - `whisper-server`: its load, its timings, and the uploaded file's name ("processing
    'jfk.wav'"), which the client chooses;
  - `audiocpp_server`: its device, its CUDA graph warm-ups, its address;
  - `sd-server`: its load, its auto-fit plan, each request's size, seed and timings, and progress
    bars drawn with `\r` and `ESC[K`, the log's only control characters.
- A file name holding an ESC never reached the log. Through the gateway, llama-swap cannot read
  the form: 404 "no model id could be identified", for whisper and audio.cpp alike. Through
  `/upstream/<model>/…`, `whisper-server` answers 400 "Invalid request". A plain name is logged.
  These facts go to `docs/security.md` §8 in task 12.
- fmt, clippy and the tests green (784, 27 and 4). `cargo mutants --in-diff`: no mutant, the
  diff holding tests only.

Committed as `test(lifecycle): show a server of every kind in status`.

The added task, `doctor` and a safetensors index:
- sd.cpp at `77b72a2` (`src/model_loader.cpp`, `src/model_io/safetensors_io.cpp`) reads every
  `.json` model file as a safetensors index:
  - a JSON object whose `weight_map` maps each tensor to its shard's path, string values, one
    shard at least, each read once;
  - a shard resolved from the index's directory, an absolute one as it is.
- Now `doctor`, for an `sd-server` model:
  - reads each `.json` file of the model as an index, and checks each shard where sd.cpp reads
    it;
  - a missing shard joins the missing files and the remedy's `hf download`, by its path in
    the model's directory;
  - an index that cannot be read, that is not an index, or that names a shard outside the
    model's directory fails the check, with the `hf download` that fetches the index again.
- A shard name holding a control or invisible character makes the index invalid, since
  `doctor` prints the name. The ok line counts the shards: "3 files and 2 shards in …".
- New code:
  - `domain::safetensors`: the parser and the shard's path;
  - `BackendKind::reads_as_index`;
  - `ToolchainProbe::read_text`, which reads at most 16 MiB on the host, the index being 72 KiB.
- The fixture `fibo-2.0-dev-72415bd0-diffusion_pytorch_model.safetensors.index.json` is Fibo's
  index, its git blob SHA-1 the Hub's (`e8669c03…`), listed in `tests/fixtures/README.md`.
- Tests:
  - each new test was watched failing on its assertion first, against stubs;
  - one doctor test, a missing index and another kind's `.json` that is not read, passes on the
    old code and guards it.
  - My expectation for a shard outside the directory was wrong (`../../` from `transformer/`
    leaves `image/`, not `/models`): the test was corrected, not the code.
  - `cargo mutants --in-diff` missed 3 mutants: `read_text` returning a constant, and the
    16 MiB cap shifted to 0. Killed by a test that reads the fixture through `SystemHost`,
    watched failing under the cap's mutation. Second run: 26 mutants, 25 caught, 1 unviable.
- Live, on this machine:
  - Fibo's directory: "3 files and 2 shards in /ai/models/image/fibo-2.0-dev";
  - a mirror of it made of links in the session's scratch folder, without the second shard:
    fail, the shard named with its `hf download`, exit 1.
- `README.md`'s doctor table: the Inventory row.
- fmt, clippy and the tests green (794, 27 and 4).

Committed as `feat(doctor): check the shards a safetensors index names`.

Task 12, the documentation of the new kinds:
- `docs/commands.md`:
  - `init`: the models' files are GGUF, safetensors or ggml;
  - `gateway config`: what ai-ctl adds to each kind's command, its `/health`, the key in the
    environment, the matrix for every kind, a link to each server's document;
  - `doctor`: Builds and Inventory cover every kind, their checks in the README's table.
  - The inventory's reference is the README's, which already describes the new kinds.
- `docs/security.md`:
  - §2: the audio and image servers before a squatter on their port. Each exits 1 on a taken
    port and binds once its model is loaded;
  - §3: the audio and image requests and their servers' routes are guarded;
  - §8: what each server writes in the log, from task 11's probe.
  - What listens and each server's key were already in §2 and §7.
- Found while writing §2: `docs/gpu/llama-swap.md`, written in task 7, said the three servers
  other than `llama-server` bind with `SO_REUSEADDR` by ai-ctl's `reuse-address.patch`. audio.cpp
  has no such patch: `audiocpp_server` sets it in its own code (`app/server/http.cpp:754` at
  `v0.9.3`), and raises an error on a failed bind, which its `main` turns into exit 1.
  Corrected.
- Links: 87, none broken.

Committed as `docs: describe the audio and image kinds in commands and security`.

The closure, begun:
- The six checks green at its start: fmt, clippy, the tests (794, 27 and 4), `test_scope.py`
  (15), `test_fixtures.py` (1), `sid.py check`.
- Criterion 1 was met live in task 9.
- Criterion 2, live, with the state in the session's scratch folder:
  - An inventory breaking nine rules of the three new kinds: a whisper backend built from
    stable-diffusion.cpp, `session_options` and `ctx_size` on a whisper model, its reserved
    `inference-path`, an sd model without its weights, its reserved `model-dir`, an audio.cpp
    model without `family`, with a `vae` file and the reserved `no-ui`. `doctor` (exit 1) and
    `up` (exit 1, no gateway started, `status` 3) list the nine.
  - The same inventory with a U+2028 in an sd flag: not among the nine. That check, like `${`
    and the control characters, runs when the configuration is generated, once the inventory
    is valid.
  - A valid inventory with `${` in a whisper flag, a U+2028 in an sd flag and an ESC in an
    audio.cpp `session_options` value: `doctor`'s Gateway configuration and `up` name only
    the first, `${`. Generation stops at its first error (`?` in
    `domain::gateway_config::generate` and in each entry), for every kind, `llama-server`
    included. Criterion 2 holds for the inventory's rules, not for the configuration's.

The user chose to fix the gap in this phase (Decisions).

The fix, in `domain::gateway_config`:
- `generate` returns `InvalidConfig`, every `ConfigError` in the inventory's order, shown as one
  message joined by `;`, as `InvalidInventory` is.
- `Text`, which checks each value, records each rule a value breaks and returns the value.
  Every builder of an entry no longer fails. A model whose backend has no program records it
  and is skipped. A volume path holding `:` is recorded.
- A value holding both `${` and a control character is listed under both.
- `doctor` downcasts `InvalidConfig`.
- Tests:
  - First the type, the old behaviour kept: every test green.
  - Then `every_value_the_configuration_cannot_hold_is_listed`, five violations across three
    models of the new kinds, two of them in one value. It was watched failing on its
    assertion: one listed.
  - Then the change: 795, 27 and 4 green, the single-error tests unchanged.
  - `cargo mutants --in-diff`: 44 mutants, 37 caught, 7 unviable.
- Live, the inventory of the three violations: `doctor`'s Gateway configuration and `up` (exit
  1, `status` 3) list the three, the ESC and the U+2028 shown escaped.
- `docs/commands.md` (`gateway config`: the control characters, every value listed) and
  `README.md`'s doctor table (the Gateway row).

Committed as `fix(gateway): list every value the configuration cannot hold`. The phase closed the same day.

---

## Decisions

- **The three kinds are named after their servers' binaries (the user, 2026-10-28):**
  `whisper-server`, `sd-server` and `audiocpp_server`, as `llama-server` is, in `kind = "…"` and
  `[defaults.<kind>]`. Set aside: `audiocpp-server`, uniform with the other names but no longer
  the binary's.
- **An audio.cpp model's fields are the model's own (the user, 2026-10-28):** `family`, `task`
  and `mode` beside its other fields, and a `[models.session_options]` table, taken for an
  `audiocpp_server` model only and refused for the other kinds. The model's `path` in the
  `--config` file ai-ctl writes is its `files.model`, or its `dir` when the model is a directory
  of safetensors; its `[models.flags]` are the server's options. Set aside: a `[models.audiocpp]`
  table holding them, one level more.
- **`whisper-server` gets `--language auto` from ai-ctl (the user, 2026-10-28)**, unless the
  kind's defaults or the model set `language`, so that an inventory written by hand does not
  decode French as English without a word. Set aside: `language = "auto"` in the shipped
  inventory's `[defaults.whisper-server]` alone, which an inventory that leaves it out loses.
- **An audio.cpp model's voices are a field of the model (the user, 2026-10-28):** `voices`, a
  directory relative to `models_dir`, checked as `dir` is, written as the configuration's
  `voice_dir`; models may share one. Without it, the configuration has no `voice_dir`, which
  Voxtral's embedded presets do not need. Set aside: one `voices_dir` for every audio.cpp
  model; a fixed `<models_dir>/voices/`.
- **`up` writes audio.cpp's configuration, and the YAML carries it (the user, 2026-10-28):**
  each `audiocpp_server` model gets `~/.local/state/ai-ctl/gateway/audiocpp/<model>.json`,
  written by `up`, and the YAML holds that file's content as a comment above the model's entry.
  A change to the JSON alone then changes the YAML, which `up` compares to tell a current
  gateway, and `gateway config` shows what each server reads. Set aside: the file's SHA-256
  alone in the YAML, shorter but hiding the content.
- **Fibo 2.0 Dev joins the image models (the user, 2026-10-29)**, for its rendering, which the
  user judged above Qwen-Image 2.1's and FLUX.3 Image Klein's on the comparison's prompts, at
  50 steps. It is the slowest of the three and the weakest on counts, positions and text with a
  plain prompt, so each keeps its use. Its text encoder runs on the CPU (`--backend te=cpu`),
  4 s more an image for 4.1 GB of VRAM, after the size probe of the day: on the GPU, 1536×1024
  aborts the server. Set aside: waiting for more reviews or a distilled
  text-to-image version, none announced by Bria; Comfy-Org's INT8 files, which `docs/stack.md`
  §5 does not admit since Bria publishes a format sd.cpp reads. The decision and its measures
  are in `docs/image/stable-diffusion-cpp.md` §4.
- **`doctor` reads a safetensors index and checks each shard it names (the user, 2026-10-29)**,
  in this phase, since task 10's check passed Fibo's `diffusion-model`, an index, with a shard
  missing, which only the server's load would tell. Set aside: moving it to phase 3, which ships
  Fibo; leaving the check at the index file.
- **The configuration's checks list every violation (the user, 2026-10-29)**, in this phase,
  since criterion 2 asks every broken rule listed, and generation stopped at its first error, for
  every kind. Set aside: moving the fix to phase 3; one error at a time.

---

## Deviations

- **Phase 3 gains Fibo 2.0 Dev** (2026-10-29): its first task adds the model to the shipped
  inventory beside phase 0's, and a new Live task checks its two-minute requests through the
  gateway and OpenWebUI, and its largest size. The probe was made outside this phase's tasks,
  in the session recorded under 2026-10-29.
- **A task added to this phase** (2026-10-29, the user's decision): `doctor` reads a
  safetensors index and checks each shard it names, under Integration in
  `phase-2-inventory-and-gateway.md`, after task 11.
- **Criterion 2 held for the inventory's rules only** (2026-10-29, found at the closure): the
  configuration's checks, `${`, control characters and line separators, stopped at the first
  value, for every kind. Fixed in this phase at the user's choice (Decisions), in `fix(gateway): list every value the configuration cannot hold`.
- **`docs/gpu/llama-swap.md` was wrong on audio.cpp's binding** (written in task 7, found in task
  12): it said every server but `llama-server` binds with `SO_REUSEADDR` by ai-ctl's patch.
  `audiocpp_server` does it in its own code. Corrected in `docs: describe the audio and image kinds in commands and security`.
- Tasks 7, 9 and 11 changed no code: each found the code already held for every kind, and
  added the tests or the probes that show it.

---

## Assessment

The inventory declares the three new kinds, `whisper-server`, `sd-server` and
`audiocpp_server`, with their fields, reserved flags and `[defaults.<kind>]`. Every check holds
for each kind:
- the inventory's rules are listed together by `doctor` and `up`;
- since the closure, the configuration's checks are listed together too.

`ai-ctl gateway config` writes each kind's entry:
- its server from the current build, on loopback and `${PORT}`, `/health` as its readiness
  check, the key through `env:`;
- `--eager-load` for `sd-server`, and `--inference-path` and `--language auto` for
  `whisper-server`;
- the JSON audio.cpp reads, written by `up`, its content carried in the YAML.

The matrix sizes every kind by `vram_mib`. Live, small models ran together and a large one
evicted them: FLUX beside whisper, then Qwen3-ASR in FLUX's place. `status` and `logs` show
every kind. `doctor` checks each model's directory, its voices, and the shards of a safetensors
index.

Most of the phase was verification. A model's port is fixed by its sorted id at each `up`, and
every server binds a port another left in TIME-WAIT. A client with the key cannot load a model
through `sd-server`'s routes, but can unload it (`docs/security.md` §3). No server writes a
prompt or an answer to the log.

Two defects were found by checking the phase's own claims, not by its tasks:
- the configuration's errors came one at a time;
- `docs/gpu/llama-swap.md` was wrong on how audio.cpp binds.

Seen, left open: `doctor`'s ok line says "1 files" for a model of one file.

What phase 3 needs to know first:
- The three models with `voices = "voices"` fail `doctor` on this machine until
  `/ai/models/voices/` exists, each voice a `<voice>.wav` with its line in `prompt_text`.
  Voxtral needs none.
- Fibo 2.0 Dev's entry is the one in `tests/gateway_config.rs`, `--backend te=cpu` included.
  `doctor` now checks its two shards.
- A model's port moves at an `up` when a model whose id sorts before it is added or removed:
  harmless, as probed.
- `ai-ctl logs <model>` for an `sd-server` model shows its progress bars, drawn with `\r` and
  `ESC[K`.
