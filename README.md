# ai-ctl

A command-line tool for local AI backends sharing a single NVIDIA GPU: diagnose the machine, build
the backends natively, serve them through one gateway that arbitrates the VRAM, and benchmark them.

ai-ctl does not reimplement what the backends or the gateway already do — model switching, prompt
caching, speculative decoding, OpenAI and Anthropic APIs, request routing. It covers what sits
around them. It serves chat, speech-to-text, text-to-speech and images: the LLM stack is described
in [docs/stack.md](docs/stack.md), the speech servers in [docs/audio/](docs/audio/), the image
server in [docs/image/](docs/image/), the gateway's arbitration of the GPU in
[docs/gpu/llama-swap.md](docs/gpu/llama-swap.md), and OpenWebUI's use of them all in
[docs/openwebui.md](docs/openwebui.md).

## Status

**v2 rebuild done.** The repository was restarted from a clean base in October 2026 and rebuilt
for the LLM stack through the v2 rebuild roadmap, completed on 2026-10-12. The first review of
that code was fixed through the v2-review-fixes roadmap, completed on 2026-10-19: security, bugs,
tests, performance and structure, then the fixes a closing review of that work asked for and a
security pass over them.

**Speech and images done.** The audio-image-backends roadmap, completed
on 2026-10-30, added three servers built natively and served by the gateway behind its key:
whisper.cpp and audio.cpp for speech-to-text and text-to-speech, voice cloning included, and
stable-diffusion.cpp to make and edit images. The shipped inventory serves 13 models on four
backends, used from OpenWebUI.

## Build

The working tree, for development, with the Rust toolchain `rust-toolchain.toml` pins, which
rustup installs at the first `cargo` command; the Makefile's targets run this build:

```bash
cargo build --release
./target/release/ai-ctl --help
```

A release, what production runs: the binary the release workflow built and checked at the tag,
downloaded from the tag's release on the public repository `sebetc4/ai-ctl` with GitHub's CLI
(`sudo dnf install gh`, then `gh auth login`), its `SHA256SUMS` checked, and installed as
`~/.local/bin/ai-ctl` ([docs/release.md](docs/release.md)). The binary holds the shipped
inventory and the patches, so it runs from anywhere.

```bash
dir=$(mktemp -d)
gh release download vX.Y.Z -R sebetc4/ai-ctl -D "$dir"
(cd "$dir" && sha256sum -c SHA256SUMS) && install -Dm 0755 "$dir/ai-ctl" ~/.local/bin/ai-ctl
ai-ctl --version
```

## Usage

### Install the inventory

```bash
ai-ctl init --models-dir ~/models
```

Writes the shipped inventory to `~/.config/ai-ctl/inventory.toml` (`$XDG_CONFIG_HOME` is honoured),
with the models directory filled in. The directory must exist. `init` never overwrites an installed
inventory: edit it, or move it away to install the shipped one again.

### Diagnose the machine

```bash
ai-ctl doctor
ai-ctl doctor --json
```

A read-only check of everything the stack needs, one line per check with a status — `ok`, `warn`,
`fail`, or `skip` when something it depends on is absent — and the command that fixes each
problem. `doctor` never installs anything and never runs `sudo`; it prints the commands instead.

| Section | Checks |
|---|---|
| GPU and driver | The NVIDIA GPU through NVML (fails without a driver, warns off Blackwell); the driver and the newest CUDA version it supports; processes other than the gateway's holding more than 64 MiB of VRAM, a sign the display runs on the NVIDIA GPU |
| Toolchain | `nvcc`, failing on CUDA 13.2, below 12.8, or newer than the driver, with the install steps of [docs/toolchain.md](docs/toolchain.md) for the running Fedora release; a warning for an enabled NVIDIA CUDA repository that offers driver packages, and for an unversioned `cuda-toolkit` meta-package; the GCC `nvcc` accepts, read from the toolkit's `crt/host_config.h`, and the one a build would use; CMake 3.18 and git 2.19.2 at least |
| Containers | Podman, the NVIDIA Container Toolkit, and a CDI specification listing `nvidia.com/gpu=all` that mounts the loaded driver's libraries; warnings only, since only image backends need them |
| Power | The AC adapter, and the `performance` power profile, read over D-Bus (`power-profiles-daemon` or `tuned-ppd`) |
| Builds | Each project's current build holds each patch its recipe applies now and each CMake option it sets, at its value, and passes its recipe's smoke test (`--version` for llama.cpp), which also catches CUDA libraries removed with a toolkit; a project a model of the inventory runs on without a build fails, one no model runs on is skipped, and so is each when the inventory cannot be read |
| Inventory | The inventory is installed and valid, and every model file it names exists, an `audiocpp_server` model's directory, holding more than `hf download`'s `.cache`, when it names no file, each shard an `sd-server` model's safetensors index names (a `.json` file but its `tokenizer`), within the model's directory, and a model's voices' directory, with the `hf download` command that fetches the missing ones at a pinned commit, to check before use |
| Gateway | llama-swap is installed (fails without it), at the release ai-ctl pins (warns on one behind it, with `ai-ctl gateway install`, and on one ahead of it, which this ai-ctl was not tested with; fails on one older than v267, which cannot read the configuration, on the pinned release when its directory records no check against the pinned digest, and on another release when its directory records no check by the ai-ctl that pins it, as an older ai-ctl's `--version` installed them); the inventory gives a configuration `up` can write (fails on each value llama-swap cannot read, such as `${` in a flag, or a path a container cannot mount, and on a margin that leaves the models none of the GPU's memory, all listed in one check); the gateway runs or is stopped, with a warning when it runs on an outdated configuration, when whether it does cannot be told (with the reason), or when its record outlived it; the inventory's address is free for `up`; no process an earlier gateway left holds GPU memory (fails otherwise, since `up` refuses to start); a "Gateway inspection" warning names what could not be read, such as ai-ctl's containers when podman fails |

The exit code is 1 when any check fails; warnings do not change it. `--json` prints the same checks
with a summary and a `healthy` verdict.

### Build a backend

```bash
ai-ctl build llama.cpp                    # the newest release, such as b11538
ai-ctl build llama.cpp --ref 79e2e74      # a tag, a branch or a commit
ai-ctl build outdated                     # where each current build stands against its newest release
ai-ctl build update                       # build what outdated finds, at the newest release
ai-ctl build list                         # installed builds; * marks the current one
ai-ctl build use llama.cpp b11400         # back to an older build, without compiling
ai-ctl build prune --keep 3               # remove all but the 3 newest and the current one
```

The shipped inventory runs models on builds of the four projects, and `up` needs the current build
of each project a model runs on:

```bash
ai-ctl build llama.cpp
ai-ctl build whisper.cpp
ai-ctl build stable-diffusion.cpp
ai-ctl build audio.cpp
```

A project none of the installed inventory's models runs on needs no build: remove the models you do
not use from the inventory before building; `doctor`'s Builds check names each build its models
still need.

`build` compiles a project from its sources for the RTX 5090's architecture, `120a-real`, and
installs the result as a new build that becomes the current one. Each project has a recipe in ai-ctl:
llama.cpp, its `llama-server` target; whisper.cpp, `whisper-server`; stable-diffusion.cpp,
`sd-server`; and audio.cpp, `audiocpp_server`. [docs/commands.md](docs/commands.md#build) gives
each recipe. A build:

1. passes the GPU and Toolchain checks of `doctor`, or is refused with their remedies;
2. clones the repository once without file contents (`git clone --filter=blob:none`), or fetches
   it, resolves `latest` to the clone's newest release tag (`b<number>` for llama.cpp) and checks
   out the commit, with the submodules its recipe names;
3. applies the recipe's patches to the sources, such as those that make `whisper-server`,
   `sd-server` and `audiocpp_server` require the gateway's key, or refuses a release one of them
   does not apply to;
4. configures a fresh CMake tree with every compiler named — `nvcc`, and the GCC `nvcc` accepts for
   C, C++ and as its host compiler, never ccache's wrapper — and builds the target only, with Ninja
   when it is installed and one job per processor (`--jobs N` to change it);
5. runs the new binary as its recipe says (for llama.cpp, `--version`, which must print
   `version: `, and `--list-devices`, which must list a CUDA device);
6. installs it under its id — the release tag, or the first ten digits of the commit — and points
   the project's `current` link at it.

Nothing is installed when a step fails. The output of CMake and the compilers is in
`~/.local/state/ai-ctl/logs/build-<project>.log`, whose last lines a failed compile's error
quotes; the other steps' errors give their own cause. A build that
exists already is refused unless `--force` is given. One command changes the builds at a time: a
second `build`, `use` or `prune` is refused while one runs, and so is each build of `build update`;
`build update` holds the lock during each project's build only, so another command may run
between two of its projects.

`prune` removes nothing while the running gateway serves one of the builds it would remove: a new
build becomes the current one, but the gateway runs on the build it started with until
`ai-ctl down` then `ai-ctl up` restart it.

```text
~/.local/share/ai-ctl/
  sources/llama.cpp/               the git clone
  work/llama.cpp/                  the CMake tree, kept after a failed build until the next one
  builds/llama.cpp/b11538/         llama-server, build.json, build.log
  builds/llama.cpp/current -> b11538
  builds/whisper.cpp/v1.9.5/       whisper-server, build.json, build.log, patches/
```

`build.json` records the commit, the CUDA toolkit, the compiler, every CMake option, the patches
its recipe applied to the sources, kept in the build's `patches/`, and the date.
A build loads the CUDA libraries of its toolkit's major version at run time (`libcudart.so.13`):
it keeps working after a minor toolkit change and stops when no toolkit of that major version is
left, which `doctor` reports.

### Install the gateway

```bash
ai-ctl gateway install                    # the llama-swap release this ai-ctl pins, v271
```

llama-swap is the gateway every client talks to, and the only process that starts or stops a
backend. Each release of ai-ctl pins one release of llama-swap, the one its tests ran on, and
llama-swap is updated only through that pin: `gateway install` installs the pinned release and no
other, whatever release is current. It downloads the release's Linux x86_64 archive from GitHub with
`curl` (HTTPS only), checks its SHA-256 against the digest ai-ctl carries for it, unpacks the
`llama-swap` binary, runs it (`-version` must name the release), installs it and points the
`current` link at it. Nothing is installed when a step fails. The pinned release installed already
becomes current again without a download, its digest not checked again, and one installation runs
at a time.

```text
~/.local/share/ai-ctl/
  llama-swap/v271/llama-swap
  llama-swap/current -> v271
```

### See the gateway's configuration

```bash
ai-ctl gateway config                     # print llama-swap's configuration
ai-ctl gateway config > /tmp/swap.yaml
# The configuration reads the key from this variable; any value validates.
AI_CTL_API_KEY=x ~/.local/share/ai-ctl/llama-swap/current/llama-swap -validate -config /tmp/swap.yaml
```

ai-ctl generates llama-swap's configuration from the inventory: one entry per model, whose command
runs the current build of its backend on the port llama-swap assigns, with the model's files,
context size and flags. `gateway config` prints it without starting anything; it fails when the
inventory is missing, a backend in use has no current build or one made without a patch its
recipe now applies or one of its CMake options at its value, a path or value contains `${`, which llama-swap would read as a macro, the
inventory's `vram_margin_mib` leaves the models none of the GPU's memory, or the GPU cannot be
read, since the models are sized to its memory. Each model takes its VRAM budget from one pool,
which llama-swap's `capacity` router shares among the models it runs together.

### Run the gateway

```bash
ai-ctl up                   # start llama-swap; returns once it answers
ai-ctl status               # the gateway, its models and the GPU memory; exits 3 when stopped
ai-ctl status --json        # the same as one JSON document, for scripts and clients
ai-ctl logs                 # the gateway's log, every model's output included
ai-ctl logs -f              # follow it until the gateway stops
ai-ctl logs qwen3.8-27b     # one model's output, then live, until Ctrl-C or the gateway stops
ai-ctl down                 # unload the models and stop llama-swap; returns once their VRAM is free
```

`up` writes the generated configuration and starts llama-swap on the inventory's `listen` address,
`127.0.0.1:8080` by default, detached from the terminal in its own process group, then returns
once its `/health` answers: llama-swap is the only process left, and it loads a model when a
request names it, or at once for a model with `preload = true`. Before that, `up` pulls the images
the models run on, and refuses when a project a model runs on has no current build, when the GPU
cannot be read, when the address is taken, or when
processes an earlier gateway left hold GPU memory — a gateway killed with SIGKILL leaves its
models running, and a new model would find less memory than its `vram_mib`. `status` lists them
with the `kill` command that ends them: ai-ctl never stops a backend itself. On a running gateway,
`up` does nothing when the configuration is unchanged and refuses otherwise; `down`, then `up`,
applies a changed inventory, build or llama-swap release.

Every request to the gateway but `/health` needs ai-ctl's API key, which the first `up` creates in
`~/.local/state/ai-ctl/api-key`, readable by you only; the models' own servers hold it too.
[docs/stack.md](docs/stack.md) §7 says how OpenCode and the agents' client read it,
[docs/openwebui.md](docs/openwebui.md) where OpenWebUI holds it.

`down` asks llama-swap to unload every model — a generation in flight is cut short — stops it
with SIGTERM, then SIGKILL after 40 s, and returns once the gateway's processes are gone from the
GPU. `up`, `down` and `status` remove the record of a gateway that has exited. The gateway's log keeps
every model's output on disk; llama-swap also keeps each model's recent output in memory while it
runs, which `logs <model>` streams.

```text
~/.local/state/ai-ctl/
  api-key                        the API key every client of the gateway sends
  gateway/gateway.json           the running gateway: PID, binary, address
  gateway/llama-swap.yaml        the configuration it runs
  gateway/audiocpp/<model>.json  the configuration an audiocpp_server model's server reads
  logs/gateway.log               its output and its models'; the previous run's is gateway.log.1
```

### Use the speech and image models

The gateway serves them at OpenAI's routes, each request naming its model; the key goes to curl
on its standard input, so that no command line shows it:

```bash
key() { printf 'Authorization: Bearer %s\n' "$(cat ~/.local/state/ai-ctl/api-key)"; }

key | curl -s -H @- http://127.0.0.1:8080/v1/audio/transcriptions \
  -F model=whisper-large-v3-turbo -F file=@speech.mp3

key | curl -s -H @- -H 'Content-Type: application/json' http://127.0.0.1:8080/v1/audio/speech \
  -d '{"model": "qwen3-tts", "input": "Bonjour.", "voice": "<voice>"}' > speech.wav

key | curl -s -H @- -H 'Content-Type: application/json' http://127.0.0.1:8080/v1/images/generations \
  -d '{"model": "flux-3-image-klein", "prompt": "…", "size": "1024x1024", "response_format": "b64_json"}' |
  jq -r '.data[0].b64_json' | base64 -d > image.png
```

A cloning model's `voice` names a recording of the inventory's voice library; the models, their
voices, sizes and times are in [docs/audio/](docs/audio/) and [docs/image/](docs/image/).

### Benchmark a model

```bash
ai-ctl bench run qwen3.8-27b                          # every scenario, each measure made 5 times
ai-ctl bench run qwen3.8-27b --repeat 10 --scenario session
ai-ctl bench list                                     # the saved runs, oldest first
ai-ctl bench compare 20261011T154305Z-qwen3.8-27b 20261011T171210Z-qwen3.8-27b
```

`bench run` measures a llama-server model the running gateway serves, through the gateway, with
the protocol of [docs/stack.md](docs/stack.md) §9. The prompts are a coding agent's at work on
ai-ctl's own code, frozen at one commit, fitted to the token with the model's own template and
tokenizer: two runs send the same tokens, and a request that must miss the prompt cache asks `cache_prompt: false`,
after an unmeasured request of the same prompt cut to a quarter, which pays llama-server's saving
of the previous context into its RAM cache.

| Scenario | Measures |
|---|---|
| `depth` | Generation at 8K, 32K and 64K tokens of context and at the context's limit, the context filled once, then 512 tokens generated on it at each repetition |
| `prefill` | The prefill of a fresh 40K-token prompt and the time to its first token |
| `session` | Sessions of 10 turns opening on 8K tokens, each turn sending the history back — the reasoning included, as OpenCode does — and 1K tokens more: the time to the first token and the tokens taken from the cache, turn by turn |
| `overhead` | The same short request sent in turn through llama-swap and straight to the model's own server, 50 times each way: what the gateway adds |

A session needs a context that holds its last turn: about 23K tokens, counted with the model's own
template before the run starts. A model with less is refused the `session` scenario, and `--scenario`
runs the others.

Every run also checks the drafts of speculative decoding each request reports against llama-server's
`/metrics` counters, and reads the GPU every 250 ms: memory, temperature, SM clock, power, and
what slowed the clocks down — the power limit, heat or the hardware. A column of its own gives the
readings NVML puts down to a board or reliability policy: on this machine, those the power limit
does not hold ([docs/stack.md](docs/stack.md) §9). Each request measured is printed on stderr as
it is measured, the overhead's every ten requests each way, the run's tables on stdout. On the
27B, a run of 5 repetitions takes about 20 minutes. A request that fails ends the run; the
scenarios it completed before are saved, and the error names the saved run.

A run refuses to start unless the gateway runs the configuration the inventory and the current
builds give now, so that it records what it measures: the model's inventory entry, its source —
the build, or the image and its digest —, its command line, and the machine. Comparing a native
build with an image, or one setting with another, is an edit of the inventory, `ai-ctl down` and
`ai-ctl up`, then one run each. `bench compare` sets saved runs side by side, one table per
measure; with two runs, a last column gives the second's change from the first.

```text
~/.local/share/ai-ctl/bench/
  20261011T154305Z-qwen3.8-27b.json    one run: its setup, its plan, every request's timings
```

### Shell completions

```bash
ai-ctl completions bash > ~/.local/share/bash-completion/completions/ai-ctl
ai-ctl completions zsh  > "${fpath[1]}/_ai-ctl"
ai-ctl completions fish > ~/.config/fish/completions/ai-ctl.fish
```

## The Inventory

The declarative list of what the gateway serves: backends, and the models they run, with their
flags and VRAM budgets. ai-ctl generates the gateway configuration from it. The shipped one,
[deploy/inventory.toml](deploy/inventory.toml), explains each setting in its comments.

```toml
version = 1
models_dir = "/home/me/models"     # model directories are relative to it

[gateway]
listen = "127.0.0.1:8080"          # optional; this is the default
vram_margin_mib = 1024             # optional; GPU memory models running together leave free
expose = false                     # optional; true allows a listen address other machines reach

[defaults.llama-server]            # flags every llama-server model gets
flash-attn = "on"
parallel = 1
no-mmproj-offload = true           # true passes the flag alone; false leaves it out

[[backends]]
id = "llama-native"
kind = "llama-server"
source = { build = "llama.cpp" }   # the current native build

[[backends]]
id = "llama-cuda13"
kind = "llama-server"
source = { image = "ghcr.io/ggml-org/llama.cpp:server-cuda13-b11541@sha256:42e80867b03cc22e8cf744c07e0b3194f078838ab6c476095ecdc1621271ec95" }

[[models]]
id = "qwen3.8-27b"                 # the `model` clients request
backend = "llama-native"
repo = "unsloth/Qwen3.8-27B-GGUF"  # for the `hf download` remedy
dir = "llm/qwen3.8-27b"            # relative to models_dir
ctx_size = 98304
vram_mib = 22733
ttl = 1800                         # seconds idle before the gateway stops it
preload = true

[models.files]                     # llama-server flag -> file in `dir`
model = "Qwen3.8-27B-UD-Q4_K_M.gguf"
mmproj = "mmproj-F16.gguf"

[models.flags]                     # added to the defaults, overriding them
cache-type-k = "q8_0"
```

| Key | Rule |
|---|---|
| `version` | `1` |
| `models_dir` | An absolute path |
| `[defaults.<kind>]` | Flags shared by every model whose backend is of that kind, so native builds and images run with the same settings |
| `[gateway]` | `listen`, the address of the gateway, on the loopback unless `expose = true`; `vram_margin_mib`, the GPU memory left free when the gateway lets models run together |
| `backends` | A unique `id`, a `kind`, named after its server: `llama-server`, `whisper-server`, `sd-server` or `audiocpp_server`; and a `source`: `build` names the project that builds the kind's server, `llama.cpp`, `whisper.cpp`, `stable-diffusion.cpp` or `audio.cpp`; `image`, for `llama-server` only, a container image, best pinned by digest (`<image>:<tag>@sha256:<digest>`), since `up` warns of one named by its tag only |
| IDs | A letter or a digit, then ASCII letters, digits, `.`, `_` and `-`: an ID names a container |
| `models` | A unique `id`; a declared `backend`; a `dir` and `files` relative to the models directory, never leaving it; the weights: a `model` file for `llama-server` and `whisper-server`, a `model` or a `diffusion-model` for `sd-server`, a `model` file or else the `dir` itself for `audiocpp_server`, which takes no other file; `ctx_size` above zero, for `llama-server` only; `vram_mib` above zero; `ttl` above zero, or absent to keep the model loaded; `repo` and `preload` optional |
| `audiocpp_server` models | `family`, required, such as `qwen3_tts`; `task` and `mode`, optional; `[models.session_options]`, strings, finite numbers or booleans, such as `"qwen3_tts.weight_type" = "q8_0"`; `voices`, a directory of reference voices relative to the models directory. ai-ctl writes them into the configuration the server reads; the other kinds refuse them |
| Flags | Long names without the leading dashes, as the server spells them: `llama-server`'s with `-`, never `_`, since it reads `_` as `-`; a string, which cannot start with `-`, a finite number, never `nan` or `inf`, or a boolean. A file flag cannot be repeated in `flags`, and the flags ai-ctl sets are refused: `host`, `port`, `alias`, `metrics`, `ctx-size`, `model`, `api-key` and `api-key-file` for `llama-server`; `host`, `port`, `model`, `inference-path` and `request-path` for `whisper-server`, which gets `language = "auto"` unless a table sets it; `listen-ip`, `listen-port`, `eager-load`, `model`, `diffusion-model` and `model-dir`, which would give its model routes other files to load, for `sd-server`; `config`, `host`, `port`, `no-ui`, `ui` and `ui-management` for `audiocpp_server` |

Unknown keys are errors. The rules are all checked together, and `doctor` reports every one the
inventory breaks. A backend kind decides which model fields apply.

## Documentation

[docs/README.md](docs/README.md) indexes the documentation by domain: the commands, the
architecture, how the GPU is shared, security, the LLM stack, the speech and image servers,
OpenWebUI and the build toolchain.
[deploy/](deploy/) holds the shipped inventory and the OpenCode configuration.

## License

MIT, in [LICENSE](LICENSE).
