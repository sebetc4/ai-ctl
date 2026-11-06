# Command Reference

Every command of ai-ctl, with its arguments, options, defaults, output and exit status. The
[README](../README.md) walks through the same commands in the order of a first install;
`ai-ctl <command> --help` prints each one's options.

| Command | What it does |
|---|---|
| [`init`](#init) | Install the inventory of backends and models into the config directory |
| [`doctor`](#doctor) | Check everything the stack needs on this machine, read-only |
| [`build`](#build) | Build a backend from its sources |
| [`build list`](#build-list) | List the installed builds |
| [`build use`](#build-use) | Make an installed build the current one, without compiling |
| [`build prune`](#build-prune) | Remove all but the newest builds |
| [`gateway install`](#gateway-install) | Download a llama-swap release, verify it and make it current |
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
| `1` | An error, printed on stderr as `error: …` with its causes; for `doctor`, at least one check failed |
| `2` | The command line is invalid: an unknown command or option, a missing argument, a value out of range |
| `3` | `status` only: no gateway runs |

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
| The builds' | `build`, `build use`, `build prune` |
| The gateway's | `up`, `down`, `build prune`, `build --force` while it installs over a build, and `status` while it removes a stale record |
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
ai-ctl build <list|use|prune> …
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
| `stable-diffusion.cpp` | `leejet/stable-diffusion.cpp`, with `ggml` and its `src/ggml-cuda/vendors/cutlass` | `master-<N>-<hash>` | `sd-server` | `--version` prints `stable-diffusion.cpp version` | `--list-devices` | `api-key.patch`, `listen-failure.patch`, `reuse-address.patch` |
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
ai-ctl gateway install [--version <N>] [--sha256 <HEX>]
```

Downloads a llama-swap release's Linux x86_64 archive from GitHub with `curl`, HTTPS only, checks
its SHA-256, unpacks the `llama-swap` binary, runs it (`-version` must name the release), installs
it and makes it current. The steps go to stderr; the release installed to stdout. Nothing is
installed when a step fails. A release installed already becomes current again without a
download, and its digest is not checked again. The output names the current binary and the
release it replaces.

| Option | Value | Default | Meaning |
|---|---|---|---|
| `--version` | `N`: a release number above zero, with or without its `v` | `263`, the release ai-ctl pins | Release to install |
| `--sha256` | `HEX`: 64 lower-case hexadecimal digits | see below | The archive's SHA-256, from a source you trust |

The archive is checked against the first of: the digest given with `--sha256`; the digest ai-ctl
carries for the pinned release; the `checksums.txt` published with the release, which sits on the
same page as the archive and so detects a corrupt download, not a replaced one.

```bash
ai-ctl gateway install
ai-ctl gateway install --version 264 --sha256 <hex>
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
([security.md §3](security.md#3-the-api-key)). The `matrix` lets models of any kind run together
when their `vram_mib` fit the GPU's memory. Each server's options and routes are in its document:
[whisper-cpp.md](audio/whisper-cpp.md), [stable-diffusion-cpp.md](image/stable-diffusion-cpp.md),
[audio-cpp.md](audio/audio-cpp.md).

It fails when the inventory is missing or invalid, when a backend in use has no current build or
one made without a patch its recipe now applies or one of its CMake options at its value, with
the command that builds it again, when a
path or value contains `${`, which llama-swap would read as a macro, or a control character or a
line separator, or when the GPU cannot be read, since the models are sized to its memory. It
fails too when the models that fit beside another make more than 20000 sets in the `matrix`,
which llama-swap would be slow to read
([stack.md §6.4](stack.md#64-server-and-gateway-arguments)). The error lists every value the
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
the gateway loads in the background, and the log's path. The first `up` creates the API key.

Before starting, `up` pulls the images the models run on, and refuses when:

- the inventory, a build or llama-swap is missing;
- a backend in use runs a build made without a patch its recipe now applies or one of its CMake
  options at its value, with the command that builds it again;
- the GPU cannot be read, since the models are sized to its memory;
- the inventory gives a configuration `gateway config` refuses, such as a `${` in a flag or more
  than 20000 sets of models that run together;
- the address is taken; the error says to change the API key when the program that holds it is
  unknown, since it may have taken the key from a client while the gateway was down
  ([security.md §3](security.md#3-the-api-key));
- processes an earlier gateway left hold GPU memory; `status` names them, with the `kill` command
  that ends them.

On a running gateway, `up` does nothing when the configuration, address and release are unchanged
and its `/health` answers, and refuses otherwise: `down`, then `up`, applies a changed inventory,
build or release, and restarts a recorded gateway that does not answer; that error quotes the
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
ai-ctl status
```

Prints whether the gateway runs, its address and PID, its models and their state, and the GPU
memory in use, with the share the gateway's models hold. It also says when the inventory, a build
or llama-swap changed since the gateway started, and lists the processes an earlier gateway left
on the GPU with the `kill` command that ends them.

Exit status: `0` when the gateway runs, `3` when it does not.

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
