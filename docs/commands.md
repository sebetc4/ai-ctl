# Command Reference

Every command of ai-ctl, with its arguments, options, defaults, output and exit status. The
[README](../README.md) walks through the same commands in the order of a first install;
`ai-ctl <command> --help` prints each one's options.

| Command | What it does |
|---|---|
| [`init`](#init) | Install the inventory of backends and models into the config directory |
| [`doctor`](#doctor) | Check everything the stack needs on this machine, read-only |
| [`build`](#build) | Build a backend from its sources |
| [`build outdated`](#build-outdated) | Tell where each current build stands against its project's newest release |
| [`build update`](#build-update) | Build what `build outdated` finds, each project at its newest release |
| [`build list`](#build-list) | List the installed builds |
| [`build use`](#build-use) | Make an installed build the current one, without compiling |
| [`build prune`](#build-prune) | Remove all but the newest builds |
| [`gateway install`](#gateway-install) | Download the llama-swap release ai-ctl pins, verify it and make it current |
| [`gateway config`](#gateway-config) | Print the configuration `up` gives llama-swap |
| [`up`](#up) | Start the gateway, and return once it answers |
| [`down`](#down) | Stop the gateway and its models, and return once their GPU memory is released |
| [`status`](#status) | Show whether the gateway runs, its models and the GPU memory in use |
| [`logs`](#logs) | Print the gateway's log, or stream one model's output |
| [`bench run`](#bench-run) | Measure a llama-server model the running gateway serves |
| [`bench list`](#bench-list) | List the saved runs |
| [`bench compare`](#bench-compare) | Set saved runs side by side |
| [`completions`](#completions) | Print a shell completion script |

## Conventions

### Global Options

| Option | Effect |
|---|---|
| `-h`, `--help` | Print the help of ai-ctl or of the command it follows |
| `-V`, `--version` | Print ai-ctl's version; top level only |

`ai-ctl help <command>` prints the same help as `ai-ctl <command> --help`.

### Output

A command prints its result on stdout, and its progress, warnings (`warning: …`) and errors
(`error: …`) on stderr: `ai-ctl gateway config > swap.yaml` or `ai-ctl doctor --json | jq` capture
the result alone. `doctor` colours its output only on a terminal, and never when `NO_COLOR` is set
to a non-empty value.

### Exit Status

| Code | Meaning |
|---|---|
| `0` | Success |
| `1` | An error, printed on stderr as `error: …` with its causes; for `doctor`, at least one check failed; for `build outdated`, a project's releases could not be read, and for `build update`, a project failed, each with its cause on stdout and the other projects still done |
| `2` | The command line is invalid: an unknown command or option, a missing argument, a value out of range |
| `3` | `status` only: no gateway runs |
| `4` | `build outdated` only: a project has something to build |

### Environment

| Variable | Use |
|---|---|
| `XDG_CONFIG_HOME` | Base of the configuration directory, `$XDG_CONFIG_HOME/ai-ctl`; default `~/.config` |
| `XDG_DATA_HOME` | Base of the data directory, `$XDG_DATA_HOME/ai-ctl`; default `~/.local/share` |
| `XDG_STATE_HOME` | Base of the state directory, `$XDG_STATE_HOME/ai-ctl`; default `~/.local/state` |
| `HOME` | The defaults above; an `XDG_*` variable counts only when it holds an absolute path |
| `NO_COLOR` | Non-empty: `doctor` prints without colours |
| `PATH` | Where the programs ai-ctl runs are found, such as `nvcc`, `cmake`, `git`, `curl` and `podman` |

Every command but `completions` fails when neither the `XDG_*` variable nor `HOME` is an absolute
path.

### Files

| Path | Written by | Content |
|---|---|---|
| `~/.config/ai-ctl/inventory.toml` | `init` | The inventory: backends, models, flags, the gateway's address |
| `~/.local/share/ai-ctl/sources/<project>/` | `build` | The project's git clone |
| `~/.local/share/ai-ctl/work/<project>/` | `build` | The CMake build tree: removed after a build that installs, kept after one that fails, until the next build |
| `~/.local/share/ai-ctl/builds/<project>/<id>/` | `build` | One build: its binary, `build.json`, `build.log`, and `patches/`, a copy of each patch its recipe applied to the sources |
| `~/.local/share/ai-ctl/builds/<project>/current` | `build`, `build use` | Link to the current build |
| `~/.local/share/ai-ctl/llama-swap/v<N>/llama-swap` | `gateway install` | One llama-swap release |
| `~/.local/share/ai-ctl/llama-swap/current` | `gateway install` | Link to the current release |
| `~/.local/share/ai-ctl/bench/<run>.json` | `bench run` | One saved run |
| `~/.local/state/ai-ctl/api-key` | `up` | The API key every client of the gateway sends, readable by the user only |
| `~/.local/state/ai-ctl/gateway/gateway.json` | `up` | The running gateway's record: PID, binary, address |
| `~/.local/state/ai-ctl/gateway/llama-swap.yaml` | `up` | The configuration the gateway runs |
| `~/.local/state/ai-ctl/gateway/audiocpp/<model>.json` | `up` | The configuration an `audiocpp_server` model's server reads |
| `~/.local/state/ai-ctl/logs/gateway.log` | `up` | The gateway's output and its models'; the previous run's is `gateway.log.1` |
| `~/.local/state/ai-ctl/logs/build-<project>.log` | `build` | The output of CMake and the compilers of the last build |

The state directory and every directory under it are created at mode 0700.

### Concurrency

Three locks keep two ai-ctl processes apart. A command that finds its lock taken fails at once,
saying what the other ai-ctl is doing, as that one wrote it in the lock's file; it never waits.

| Lock | Taken by |
|---|---|
| The builds' | `build`, `build update` during each build, `build use`, `build prune` |
| The gateway's | `up`, `down`, `build prune`, `build --force` and `build update` while they install over a build, and `status` while it removes a stale record |
| llama-swap's releases' | `gateway install` |

## init

```text
ai-ctl init --models-dir <DIR>
```

Writes the shipped inventory to `~/.config/ai-ctl/inventory.toml`, with `models_dir` set to the
absolute path of `DIR`, then prints the inventory's path and the next step.

| Option | Value | Default | Meaning |
|---|---|---|---|
| `--models-dir` | `DIR`, a path | required | Directory holding the models' files, GGUF, safetensors or ggml, one subdirectory per model; it must exist |

`init` never overwrites an installed inventory: it refuses when the file exists, and names the
target of a link that points nowhere. Edit the installed inventory, or move it away to install the
shipped one again.

```bash
ai-ctl init --models-dir /ai/models
```

## doctor

```text
ai-ctl doctor [--json]
```

Checks everything the stack needs on this machine and prints one line per check with its status,
`ok`, `warn`, `fail`, or `skip` when something it depends on is absent, and the command that fixes
each problem. `doctor` reads only: it never installs anything and never runs `sudo`. The sections
are GPU and driver, Toolchain, Containers (image backends only), Power, Builds, Inventory and
Gateway; the
[README](../README.md#diagnose-the-machine) lists each section's checks. Builds and Inventory
cover every backend kind: the build of each project a model runs on, and each model's files as
its server reads them.

| Option | Default | Meaning |
|---|---|---|
| `--json` | off | Print the checks as JSON instead of text |

The JSON object holds `healthy`, `true` when no check fails; `summary`, the count of each status
(`ok`, `warn`, `fail`, `skip`); and `sections`, each with its `name` and `checks`, each check with
its `name`, `status`, `detail` and `remedy`.

Exit status: `1` when any check fails; warnings leave it at `0`.

```bash
ai-ctl doctor
ai-ctl doctor --json | jq '.sections[].checks[] | select(.status == "fail")'
```

## build

```text
ai-ctl build [--ref <REF>] [--force] [--jobs <N>] <PROJECT>
ai-ctl build <outdated|update|list|use|prune> …
```

Compiles a project from its sources for the RTX 5090's architecture, `120a-real`, and installs the
result as a new build that becomes the current one. The progress goes to stderr, rewritten in
place on a terminal; the build's id, commit, CUDA toolkit, host compiler and duration to stdout,
with the patches applied, the current binary's path and the build it replaces.

| Argument | Meaning |
|---|---|
| `PROJECT` | Project to build: one with a recipe in ai-ctl, in the table below |

| Option | Value | Default | Meaning |
|---|---|---|---|
| `--ref` | `REF` | `latest` | Git tag, branch or commit to build; `latest` is the newest release tag, in the form the table below gives |
| `--force` | — | off | Build again a build that is already installed |
| `--jobs` | `N`, above zero | one per processor | Parallel compile jobs |

Each project's recipe:

| Project | Repository, and the submodules built | Release tags | Server | Smoke test | Device listing | Patches |
|---|---|---|---|---|---|---|
| `llama.cpp` | `ggml-org/llama.cpp` | `b<N>` | `llama-server` | `--version` prints `version: ` | `--list-devices` | — |
| `whisper.cpp` | `ggml-org/whisper.cpp` | `vX.Y.Z` | `whisper-server` | `--help` names `--inference-path` | — | `api-key.patch`, `reuse-address.patch` |
| `stable-diffusion.cpp` | `leejet/stable-diffusion.cpp`, with `ggml` and CUTLASS, `thirdparty/cutlass` or, in an older release, `ggml`'s `src/ggml-cuda/vendors/cutlass` | `master-<N>-<hash>` | `sd-server` | `--version` prints `stable-diffusion.cpp version` | `--list-devices` | `api-key.patch`, `listen-failure.patch`, `reuse-address.patch` |
| `audio.cpp` | `0xShug0/audio.cpp` | `vX.Y.Z` | `audiocpp_server` | `--version` prints `audio.cpp ` | `--list-devices` | `api-key.patch` |

A build passes `doctor`'s GPU and Toolchain checks or is refused with their remedies, then clones
the repository once (`--filter=blob:none`), checks out the commit with the submodules its recipe
names, fetched the same way, a submodule's own within it and none the commit does not hold, a submodule a kill left without a commit during its first fetch cloned again, applies the recipe's patches, configures a fresh CMake tree with every
compiler named, builds the target, runs the new binary's smoke test, which must exit 0 and print
its text, and its device listing, which must list a CUDA device, and installs it under its id: the
release tag, or the commit's first ten digits. Nothing is installed when a step fails: a release one of the patches does not apply to is refused before the
compile, naming the patch; a failed compile's error quotes the last lines of
`logs/build-<project>.log`, the other steps' errors their own cause, such as the new binary's
output, and the build tree stays in `work/<project>/` until the next build. A build whose id is
installed already, or whose directory is there but not a readable build, is refused without
`--force`: a release tag given to `--ref` is refused at once, before the checks and the fetch, any
other reference once checked out, since the checkout decides its id. With `--force`, the new build
takes the installed one's place under the same path, so it is refused while the running gateway
serves that build, before the compile and again before the install: the error names `ai-ctl down`.

```bash
ai-ctl build llama.cpp                  # the newest release
ai-ctl build llama.cpp --ref 79e2e74    # a tag, a branch or a commit
ai-ctl build llama.cpp --ref b11538 --force --jobs 8
```

### build outdated

```text
ai-ctl build outdated [PROJECT]
```

Tells where each current build stands, one line per project on stdout: its current build, its
project's newest release and a verdict, then llama-swap's line. It reads and changes nothing, takes
no lock and needs no clone; `doctor`, which stays local and fast, does not make this check. The
newest release comes from `git ls-remote --tags --refs` on the recipe's repository, the tags of the
recipe's form ordered by number, as `build` chooses `latest`.

```text
llama.cpp             b12111               newest b12226               behind
whisper.cpp           v1.9.5               newest v1.9.5               current
stable-diffusion.cpp  master-1024-77b72a2  newest master-1068-fc350be  behind
audio.cpp             v0.9.3               newest v0.9.4               behind
llama-swap            v271                 pinned v271                 current
```

| Argument | Default | Meaning |
|---|---|---|
| `PROJECT` | each project a backend of the inventory builds, in the recipes' order | Only this project, built by the inventory or not; the inventory is not read |

| Verdict | Meaning |
|---|---|
| `current` | At the newest release, or past it, as after a release its project withdrew |
| `behind` | At an older release |
| `not built` | No current build; one a `current` link names but that cannot be read is warned of on stderr, and counts as none |
| `built from a commit` | Built through a `--ref` that no release names |
| `off its recipe: …` | At the newest release or past it, but built before its recipe applied a patch or set a CMake option, as `doctor` reports it |
| `cannot read its releases: …` | The repository could not be read or holds no release, or ai-ctl has no recipe for the project; the other projects are still told |

llama-swap's line sets its current release beside the release this ai-ctl pins, and never beside
its project's newest: a newer llama-swap reaches a machine through a release of ai-ctl
([security.md §7](security.md#llama-swap-releases)). Behind the pin, or not installed, it names
`ai-ctl gateway install`, and says when `up` refuses the release for its age, older than v267;
ahead of the pin, the same command, which goes back to it. Whether the release was checked against
a pinned digest is told by `doctor`'s llama-swap check and `up`, never by this line.

Exit status: `0` when each project is `current`; `4` when one is `behind`, `not built`, `built from
a commit` or `off its recipe`; `1` when a project cannot be read, the others printed first, over
`4`. llama-swap's line never changes it.

```bash
ai-ctl build outdated
ai-ctl build outdated llama.cpp
```

### build update

```text
ai-ctl build update [--jobs <N>] [PROJECT]
```

Builds what [`build outdated`](#build-outdated) finds, each project in turn and each as `ai-ctl
build <project>` builds its newest release: the checks, the fetch, the patches, the smoke test.
Each new build becomes the current one, and the build it replaces stays for `build use`, but for a
build off its recipe, which its new build of the same release replaces in place: no other build is
removed. The progress goes to stderr; stdout gets one line per project, built with the build it
replaces, already current, or failed with its cause, then what is left to do.

```text
llama.cpp             built b12226 (was b12111)
whisper.cpp           already current
stable-diffusion.cpp  failed: stable-diffusion.cpp master-1068-fc350be cannot be built: its patch listen-failure.patch does not apply; `--ref` builds a release it applies to: …
audio.cpp             built v0.9.4 (was v0.9.3)
The builds replaced stay installed: `ai-ctl build use <project> <id>` goes back to one.
The running gateway keeps its builds until `ai-ctl down` then `ai-ctl up`.
```

| Argument | Default | Meaning |
|---|---|---|
| `PROJECT` | each project a backend of the inventory builds | Only this project, built by the inventory or not; the inventory is not read |

| Option | Value | Default | Meaning |
|---|---|---|---|
| `--jobs` | `N`, above zero | one per processor | Parallel compile jobs |

A failure stops its project only: a patch that no longer applies, a compile, a smoke test or a
repository that cannot be read is reported and the next project goes on. A project `behind` or
`built from a commit` whose newest release is installed already, which `build use` went back from,
is refused as `build` refuses it, naming `build use`. A project `off its recipe` is built again at
its own release, never an older one, as `--force` would, so refused while the running gateway
serves that build, the error naming `ai-ctl down`; past the newest release, whose tag the project
withdrew and the fetch drops, it fails, naming `ai-ctl build <project>` for the newest release.
llama-swap is never built or installed here: `ai-ctl gateway install` does it, at the release this
ai-ctl pins.

Exit status: `0` when nothing failed, `1` otherwise.

```bash
ai-ctl build update
ai-ctl build update --jobs 8 llama.cpp
```

### build list

```text
ai-ctl build list [PROJECT]
```

Lists the installed builds of each project, newest first, one line per build: its id, its date,
the commit's first ten digits, the CUDA toolkit and the host compiler; `*` marks the current one.
A directory that is not a readable build is listed as such. A `current` link that names no
installed build, whether the build is gone or its directory is not a readable build, is a warning
on stderr with its remedy.

| Argument | Default | Meaning |
|---|---|---|
| `PROJECT` | every project | Only this project's builds |

### build use

```text
ai-ctl build use <PROJECT> <ID>
```

Makes an installed build the current one, without compiling, and names the build it replaces. The
running gateway keeps its build until `ai-ctl down` then `ai-ctl up`.

| Argument | Meaning |
|---|---|
| `PROJECT` | Project, such as `llama.cpp` |
| `ID` | Build id, as `build list` shows it; it must be installed |

```bash
ai-ctl build use llama.cpp b11400
```

### build prune

```text
ai-ctl build prune --keep <N> [PROJECT]
```

Removes every build but the `N` newest and the current one, which is always kept, and prints what
it removed and kept. Nothing is removed when the running gateway serves one of the builds to
remove: the error names `ai-ctl down` then `ai-ctl up`, which restart it on the current build. A
removal that fails stops the prune, and its error names the builds removed before it.

| Argument | Default | Meaning |
|---|---|---|
| `PROJECT` | every project | Only this project's builds |

| Option | Value | Default | Meaning |
|---|---|---|---|
| `--keep` | `N`, zero or more | required | How many of the newest builds to keep, besides the current one |

```bash
ai-ctl build prune --keep 3
ai-ctl build prune --keep 0 llama.cpp   # only the current build is left
```

## gateway

```text
ai-ctl gateway <install|config>
```

### gateway install

```text
ai-ctl gateway install
```

Installs the llama-swap release this ai-ctl pins, `v271`, and no other: llama-swap is updated only
through ai-ctl's pin, whatever release is current, behind or ahead of it. Downloads the release's
Linux x86_64 archive from GitHub with `curl`, HTTPS only, checks its SHA-256 against the digest
ai-ctl carries for it, the only source of trust, unpacks the `llama-swap` binary, runs it
(`-version` must name the release), installs it and makes it current. The steps go to stderr; the
release installed to stdout. Nothing is installed when a step fails. Each release's directory
records the digest its archive was checked against, `archive.sha256`: the pinned release installed
already against the pinned digest becomes current again without a download; one with no record or
another digest, as an older ai-ctl left it, is downloaded and checked again. The output names the
current binary and the release it replaces. It takes no option.

```bash
ai-ctl gateway install
```

### gateway config

```text
ai-ctl gateway config
```

Prints, without starting anything, the YAML configuration `up` gives llama-swap, generated from the
inventory. Each model gets one entry. Its command runs the model's server from the current build
of its backend's project, or from the backend's image for `llama-server`. The server listens on
`127.0.0.1` and the port llama-swap gives it, `${PORT}`, with the model's files, its kind's
defaults and its flags. ai-ctl adds to each kind's command:

| Kind | Added |
|---|---|
| `llama-server` | `--alias <model>`, `--metrics`, and the model's `--ctx-size` |
| `whisper-server` | `--inference-path /v1/audio/transcriptions`, the route llama-swap forwards; `--language auto`, unless a table sets `language` |
| `sd-server` | `--eager-load`, so that its health check answers once its weights are loaded |
| `audiocpp_server` | `--config <file>` and `--no-ui`: the file is the JSON `up` writes, `~/.local/state/ai-ctl/gateway/audiocpp/<model>.json`, with the model's `family`, `task`, `mode`, path, `session_options` and voices, loaded at start; the entry carries its content as a comment |

Each entry's readiness check is its server's `/health`. The key reaches each server as
`LLAMA_API_KEY` in its environment, never in the file
([security.md §3](security.md#3-the-api-key)). Each entry takes its model's `vram_mib` from the
`capacity` router's one pool, the GPU's usable memory less the inventory's `vram_margin_mib`, and
llama-swap runs models of any kind together while their budgets fit in it; a model whose budget
exceeds the pool takes all of it, and runs alone
([stack.md §6.4](stack.md#64-server-and-gateway-arguments)). Each entry declares what its model
can do, its `capabilities` — what it reads and writes, whether it calls tools, the context of one
request —, from its kind and its entry, which llama-swap shows clients in `/v1/models` before any
model is loaded ([gpu/llama-swap.md](gpu/llama-swap.md#what-each-model-can-do)). Each server's options and routes are in its document:
[whisper-cpp.md](audio/whisper-cpp.md), [stable-diffusion-cpp.md](image/stable-diffusion-cpp.md),
[audio-cpp.md](audio/audio-cpp.md).

It fails when the inventory is missing or invalid, when a backend in use has no current build or
one made without a patch its recipe now applies or one of its CMake options at its value, with
the commands that build it again, at its release or at the newest when its project withdrew it,
when a path or value contains `${`, which llama-swap would read as a macro, or a control character or a
line separator, when the GPU cannot be read, since the models are sized to its memory, or when
the inventory's `vram_margin_mib` leaves the pool nothing of it. The error lists every value the
configuration cannot hold, of every model.

```bash
ai-ctl gateway config > /tmp/swap.yaml
# The configuration reads the key from this variable; any value validates.
AI_CTL_API_KEY=x ~/.local/share/ai-ctl/llama-swap/current/llama-swap -validate -config /tmp/swap.yaml
```

## up

```text
ai-ctl up
```

Writes the generated configuration and starts the current llama-swap on the inventory's `listen`
address, `127.0.0.1:8080` by default, detached in its own process group, then returns once its
`/health` answers. It prints the address, the release, the PID, the models with `preload = true`
the gateway loads in the background, and the log's path. A llama-server model loaded that way
leaves `failed to preload model <id>: status 404` in the log, its server built without the web
UI the preload asks for, and runs all the same ([gpu/llama-swap.md](gpu/llama-swap.md#what-llama-swap-offers)). The first `up` creates the API key.

Before starting, `up` pulls the images the models run on, and refuses when:

- the inventory, a build or llama-swap is missing, or the current llama-swap is older than v267,
  the first to read the `capacity` router the configuration runs the models on;
- the current llama-swap is the pinned release but its directory records no check against the
  pinned digest, or another release whose directory records no check by an ai-ctl that pins it,
  as an older ai-ctl installed it, naming `ai-ctl gateway install`;
- a backend in use runs a build made without a patch its recipe now applies or one of its CMake
  options at its value, with the commands that build it again, at its release or at the newest
  when its project withdrew it;
- the GPU cannot be read, since the models are sized to its memory;
- the inventory gives a configuration `gateway config` refuses, such as a `${` in a flag or a
  margin that leaves the models none of the GPU's memory;
- the address is taken; the error says to change the API key when the program that holds it is
  unknown, since it may have taken the key from a client while the gateway was down
  ([security.md §3](security.md#3-the-api-key));
- processes an earlier gateway left hold GPU memory; `status` names them, with the `kill` command
  that ends them.

On a running gateway, `up` does nothing when the configuration, address and release are unchanged,
its binary not replaced since it started, and its `/health` answers, and refuses otherwise: `down`,
then `up`, applies a changed inventory, build or release, runs the binary `gateway install` checked
again, and restarts a recorded gateway that does not answer; that error quotes the
log's last lines. A gateway that never answers is stopped and its record removed; one that
outlives the stop stays recorded. An answer on the address counts only from the socket of the
llama-swap `up` started: another program that took the address meanwhile is not taken for the
gateway.

`up` records the gateway, `gateway.json`, right after it starts llama-swap; a record it cannot
write stops that llama-swap. A Ctrl-C or a kill of `up` in between, the time of one file's write
and rename, leaves an llama-swap that ai-ctl does not know: `down` and `status` say no gateway
runs, and the next `up` refuses the address as taken by another program. `pgrep -af llama-swap`
names it, ai-ctl's own release under `~/.local/share/ai-ctl/llama-swap/`, and `kill <PID>` ends
it; the key needs no change for it, since that llama-swap is the gateway the key was made for.

## down

```text
ai-ctl down
```

Asks llama-swap to unload every model, which cuts short a generation in flight, stops it with
SIGTERM, then SIGKILL after 40 s, and returns once the gateway's processes are gone from the GPU.
It prints the PID stopped and the GPU memory released, or `No gateway running`, which is not an
error. A process still on the GPU after that is an error that names it: ai-ctl never stops a
backend itself. `down` stops only the recorded gateway: one an interrupted `up` left without its
record is ended by hand, as the `up` section says.

## status

```text
ai-ctl status [--json]
```

Prints whether the gateway runs, its address and PID, its models and their state, and the GPU
memory in use, with the share the gateway's models hold. It also says when the inventory, a build
or llama-swap changed since the gateway started, and lists the processes an earlier gateway left
on the GPU with the `kill` command that ends them.

| Option | Default | Meaning |
|---|---|---|
| `--json` | off | Print the same reading as one JSON document instead of text |

Exit status: `0` when the gateway runs, `3` when it does not, as text or as JSON; `1` on an error,
with nothing on stdout.

The JSON document, with the gateway running:

```json
{
  "running": true,
  "gateway": {
    "pid": 4242,
    "listen": "127.0.0.1:8080",
    "url": "http://127.0.0.1:8080",
    "models": [{ "id": "qwen3.8-27b", "state": "ready" }],
    "configuration": { "state": "current", "reason": null },
    "gpu_mib": 21877
  },
  "gpu": { "used_mib": 22015, "usable_mib": 24075 },
  "leftovers": [{ "pid": 991, "name": "llama-server", "mib": 21000 }]
}
```

| Field | Value |
|---|---|
| `running` | Whether the gateway runs, as the exit status says |
| `gateway` | The running gateway; `null` when it is stopped |
| `gateway.pid` | llama-swap's PID, the process that listens on `gateway.listen` once it has bound it |
| `gateway.listen` | The address the gateway listens on |
| `gateway.url` | That address as a URL |
| `gateway.models` | Each model it runs, its `id` and its `state` as llama-swap gives it, such as `starting`, `ready` or `stopping`; `[]` when none is loaded, `null` when the gateway's API did not answer |
| `gateway.configuration` | Its `state`: `current`; `outdated` when the inventory, a build or llama-swap changed since the gateway started; `unknown` when that cannot be worked out, with why in `reason`, `null` otherwise |
| `gateway.gpu_mib` | The GPU memory the gateway's processes hold, `0` when they hold none; `null` when the GPU or the driver does not tell |
| `gpu` | `used_mib` and `usable_mib`; `null` when the GPU cannot be read |
| `leftovers` | The processes an earlier gateway left on the GPU, each with its `pid`, its `name` and its `mib`, `null` when the driver does not tell |

A client can rely on the exit status, `running`, `gateway.pid`, `gateway.listen` and `gateway.url`.
`status` reads the gateway's record and whether its PID still runs llama-swap, and does not look at
the socket: the agents' client checks from these fields that the gateway's PID holds the listening
socket before it sends the key. A field is added, never renamed or removed without a version of ai-ctl that says so
in `CHANGELOG.md`. Warnings, and the removal of a stale record, stay on stderr.

`up`, `down` and `status` remove the record of a gateway that has exited, and say so on stderr.
`status` observes without the gateway's lock, so it never makes an `up` or a `down` wait; it takes
the lock only to remove such a record, and leaves the record, with a warning, while another ai-ctl
holds it.

## logs

```text
ai-ctl logs [-f] [MODEL]
```

Without `MODEL`, prints the gateway's log, `logs/gateway.log`, which holds the output of every
model since the last `up`. With `MODEL`, streams that model's output from the running gateway:
what the gateway kept of it in memory, then its live output, until Ctrl-C or the gateway stops.

| Argument | Default | Meaning |
|---|---|---|
| `MODEL` | none | A model ID of the inventory; needs a running gateway |

| Option | Default | Meaning |
|---|---|---|
| `-f`, `--follow` | off | Keep printing the gateway's log as it grows, until the gateway stops; a `MODEL` stream follows always |

`logs` fails when the gateway has never run, since there is no log yet. A follow ends with a line
on stderr: the gateway stopped, no gateway ran, or a new `up` replaced the log, which
`ai-ctl logs -f` follows again. A reader that leaves, as in `ai-ctl logs | head`, ends `logs`
without an error.

```bash
ai-ctl logs -f
ai-ctl logs qwen3.8-27b
```

## bench

```text
ai-ctl bench <run|list|compare>
```

### bench run

```text
ai-ctl bench run [--repeat <N>] [--scenario <SCENARIO>]… <MODEL>
```

Measures a llama-server model the running gateway serves, through the gateway, with the protocol
of [docs/stack.md](stack.md) §9, and saves the run. Each request measured is printed on stderr as it
is measured, but the overhead's, printed every ten requests each way; the request that sets the
context aside before a fresh prompt is not. The run's tables go on stdout, then the saved run's id
and path.

| Argument | Meaning |
|---|---|
| `MODEL` | Model ID, as the inventory names it |

| Option | Value | Default | Meaning |
|---|---|---|---|
| `--repeat` | `N`, 1 or more | `5` | How many times each measure is made |
| `--scenario` | `SCENARIO`; repeatable | every scenario | Run only this scenario; repeat the option for several |

| Scenario | Measures |
|---|---|
| `depth` | Generation at 8K, 32K and 64K tokens of context and at the context's limit |
| `prefill` | The prefill of a fresh 40K-token prompt and the time to its first token |
| `session` | Sessions of 10 turns, the history sent back at each: the time to the first token and the tokens taken from the cache, turn by turn |
| `overhead` | The same short request through the gateway and straight to the model's server, 50 times each way |

A run refuses to start unless the gateway runs the configuration the inventory and the current
builds give now, which needs the GPU read, since the models are sized to its memory. A model whose
context cannot hold a session's last turn, about 23K tokens, is
refused the `session` scenario; `--scenario` runs the others. A request that fails ends the run:
the scenarios completed before it are printed and saved as the run, then the command fails with
the request's error. A run that cannot be saved is printed, then fails the command. The run is
saved after each scenario as well, so a run stopped by Ctrl-C keeps the scenarios it completed.
Every run also reads the GPU every 250 ms and checks speculative decoding's drafts against
llama-server's `/metrics`.

```bash
ai-ctl bench run qwen3.8-27b
ai-ctl bench run qwen3.8-27b --repeat 10 --scenario depth --scenario prefill
```

### bench list

```text
ai-ctl bench list
```

Lists the saved runs, oldest first, with each one's source (the build, or the image), repetitions
and scenarios. A run that cannot be read is listed with the reason.

### bench compare

```text
ai-ctl bench compare <RUN> <RUN>…
```

Sets saved runs side by side, one table per measure. With two runs, a last column gives the
second's change from the first.

| Argument | Meaning |
|---|---|
| `RUN` | Two or more saved runs, as `bench list` names them; the first is the reference |

```bash
ai-ctl bench compare 20261011T154305Z-qwen3.8-27b 20261011T171210Z-qwen3.8-27b
```

## completions

```text
ai-ctl completions <SHELL>
```

Prints a completion script for a shell. It needs no inventory, build or environment, and works on
any machine.

| Argument | Values |
|---|---|
| `SHELL` | `bash`, `elvish`, `fish`, `powershell`, `zsh` |

```bash
ai-ctl completions bash > ~/.local/share/bash-completion/completions/ai-ctl
ai-ctl completions zsh  > "${fpath[1]}/_ai-ctl"
ai-ctl completions fish > ~/.config/fish/completions/ai-ctl.fish
```
