# Phase 3 Report: Gateway

**Phase:** [phase-3-gateway.md](phase-3-gateway.md)
**Start Commit:** `docs: close phase 2` of 2026-10-09

---

## Work Log

### 2026-10-09

Phase opened right after the closure of phase 2 (`docs: close phase 2`); the opening edits to the README and
the phase file are not committed yet. Read the phase 2 file and report in full before opening; it
left no pending restructuring.

State of the machine at opening: `b11538` is the current llama.cpp build; llama-swap is not
installed; no inventory is installed (`ai-ctl init` never ran); and neither model of the shipped
inventory is downloaded — no `.gguf` file under `/home`, `/data`, `/mnt` or `/srv`. The first two
acceptance criteria (a chat request loading the main model, a swap to the other one) and the
OpenCode and OpenWebUI check need both models on disk and an installed inventory; the phase file's
Dependencies name neither.

The user gave `/ai/models/gguf` as the models directory and asked for a better layout of `/ai`
if one was worth it. `/ai` is a btrfs subvolume (`compress=zstd:1`, `noatime`) on the same LUKS
device as `/home`, 102 GB used. It held flat GGUF files under `models/gguf/` (Qwen3.5-9B Q4 and
Q5, Qwen3.5-27B Q5, Qwen3-Coder-30B-A3B Q5: 52 GB, none of them in the inventory), three AWQ
snapshots for vLLM under `models/awq/` (45 GB), Voxtral-4B-TTS under `models/tts/`, `HF_HOME` at
`models/huggingface/` (exported in `~/.bashrc`), and a v1-era `vllm/config/models.yml`. Its
directories mixed two axes, format (`gguf`, `awq`) and use (`tts`); flat files cannot hold a
model's several files, and every Qwen repository names its projector `mmproj-F16.gguf`; and the
inventory has a single `models_dir` that later roadmaps' whisper.cpp, Voxtral and
stable-diffusion.cpp entries share, while it refuses a path leaving that directory.

Proposed grouping by use under one root; the user chose it, and to archive what the inventory does
not use. Done by renames on the same filesystem: `/ai/models/llm/` created; `models/gguf`,
`models/awq` and `/ai/vllm` moved to `/ai/archive/`; `models/huggingface` moved to
`/ai/cache/huggingface`, and `HF_HOME` in `~/.bashrc` follows. Voxtral stays at
`/ai/models/tts/Voxtral-4B-TTS-2603`, its name unchanged: `scripts/serve-voxtral.sh` of
`/code/python/polarsteps-tts` defaults to that path. `/code/docs/ai/scripts/vllm` still names
`/ai/vllm`, the abandoned vLLM setup, now under `/ai/archive/vllm`.

In the repository, the shipped inventory's model directories became `llm/qwen3.8-27b` and
`llm/qwen3.6-35b-a3b`, with the layout explained in its comment; `docs/stack.md` §8 downloads into
`$MODELS_DIR/llm/…` and gives this machine's `MODELS_DIR` and `HF_HOME`; the README's inventory
example follows; one test of `src/infrastructure/inventory_file.rs` expected the old paths. The
three checks pass. `ai-ctl init --models-dir /ai/models` installed the inventory.

A dry run of the first `hf download` was stopped by the user: the models are downloaded in a new
session. The user set the rule for model files: GGUF from Unsloth, preferred for quality, and
each download verified against tampering. Checked two facts before writing it down: the Hub
exposes each LFS file's SHA-256 (`lfs.sha256` from `HfApi.get_paths_info`), and a GGUF's embedded
chat template can carry a backdoor that leaves the weights untouched and passed Hugging Face's
scans (Pillar Security, July 2025; arXiv 2602.04653). Wrote the rule into `docs/stack.md` §5,
"Where Models Come From", with its sources, and added the models to this phase's Dependencies.

The user asked to record the work and commit; the work goes on in a new session. No task of the
phase is done: 0 of 13.

Where the next session starts: download both models into `/ai/models/llm/` with `--revision`
pinned, check each file's SHA-256 against the Hub and read each embedded chat template, then
write the commands that worked into `docs/stack.md` §5. Whether ai-ctl should hold the revision
and hashes in the inventory and check them (a `doctor` option, say) is the user's call: the
roadmap leaves Hugging Face downloads out of scope, and verifying files is not decided. Then the
phase's first task, installing a pinned llama-swap release.

### 2026-10-10

Resumed in a new session; the user asked for the commands to download the models in the
background, to run them themselves. Read the Hub's metadata (`/api/models/<repo>/revision/main?blobs=true`)
to pin each repository and get the LFS hashes the check needs:

| Repository | Commit (last modified) | File | Size | SHA-256 |
|---|---|---|---|---|
| `unsloth/Qwen3.8-27B-GGUF` | `4ca720788d1e01f1bff70c033e0d0028fd02e502` (2026-08-20) | `Qwen3.8-27B-UD-Q4_K_M.gguf` | 16 464 440 224 | `322e194ff79741c7baa497c240f677f54b201b0efab44ca8e50f122b39123482` |
| | | `mmproj-F16.gguf` | 927 607 488 | `cbb841a9ee0636b2ec172f5bb8df2ea8dfeb01e90fe7c6126581d662a0b4e43e` |
| | | `MTP/mtp-Qwen3.8-27B-Q4_0.gguf` | 1 369 590 656 | `50d9ce5a6da381bbcfb31061cf73df94a90e6faf8efeddee379a9cb8f1501c6e` |
| `unsloth/Qwen3.6-35B-A3B-GGUF` | `a483e9e6cbd595906af30beda3187c2663a1118c` (2026-04-20) | `Qwen3.6-35B-A3B-UD-IQ4_XS.gguf` | 17 730 509 792 | `649d7508507b84638732c4f52c24c8b15843c6dca2f3ff793ae07c14a67ebbb3` |
| | | `mmproj-F16.gguf` | 899 283 680 | `8971ee4f331ff0a4c609374f32984b3d4e6dc086c0aa35f1d637fad1829e887f` |

37.4 GB in all. `hf` is 1.14.0 and takes `--revision`. Gave the user one background job that
downloads both models in turn with these commits pinned, then a `sha256sum -c` against the table.
`/ai/models/llm/qwen3.8-27b/` already held an empty `.cache/huggingface/` and `MTP/` from an
earlier attempt; `hf download` reuses them.

The user reported the downloads done in `/ai/models/llm/`. The five files had the table's sizes,
with no `.incomplete` file left and no `hf` process running; `sha256sum -c` against the table
passed for all five.

Read each GGUF's `tokenizer.chat_template` with a standard-library reader first, then with
llama.cpp's `gguf-dump` from the clone at `b11538` (`uvx --from …/gguf-py`), which gave the same
bytes; that reader is what `docs/stack.md` documents. Compared both with `chat_template.jinja` of
`Qwen/Qwen3.8-27B` (`1d4bf0f`, 2026-08-14) and `Qwen/Qwen3.6-35B-A3B` (`995ad96`, 2026-04-24).
Both differ, in control flow only: every string literal they add is a role name, an error
message or `'high'`; no URL, no character outside ASCII; each ends with the comment
`Unsloth fixes - developer role, [merged system messages,] tool calling`. The Unsloth model cards
at the pinned commits, and the Unsloth guides, document the `developer` role and the tool-calling
change. Not documented there: merging the leading system/developer messages (the 35B merges two
at most and silently drops a later system message, where the 27B raises), accepting a
conversation without a user message, and, in the 27B, reading `reasoning_effort: "high"` as
`xhigh` — Qwen's own template rejects `high`, which `docs/stack.md` §7.1 tells clients to send.
None of them adds text to the prompt, so none can carry the injected content the rule guards
against. The 27B template raises on tool-call arguments given as a JSON string; llama-server at
`b11538` turns them into an object when its probe finds a template that takes one
(`common/chat.cpp:1337`), which the OpenCode criterion will confirm.

Wrote the commands into `docs/stack.md`: §5 gets the check (commit from the Hub API, hashes at
that commit piped into `sha256sum -c`, template extracted and diffed with the base model's) and
the record of this check; §8 pins both downloads with `--revision`; two sources added.

The user accepted the templates as shipped, to be tested through the gateway. Committed the
check (`docs(stack): pin the models and record their check`); next, the phase's first task, installing a pinned llama-swap release.

llama-swap publishes about one release a week; v263 came out that morning (2026-10-10), with a
goreleaser `checksums.txt` and a per-asset digest in the GitHub API, which agree. Its notes carry
PR #1211, `disableKeepAlives` against 502 `proxy error: EOF` when llama-server closes a pooled
connection after a streamed response: reason enough to pin v263. Its Linux archive holds
`llama-swap` (43 MB, static Go binary) with `README.md`, `LICENSE.md` and `CHANGELOG.md`;
`llama-swap -version` prints `version: v263 (e33e97b), built at …` on stdout; `-validate` checks a
configuration and exits, which the configuration tasks can use. Captured the checksums file and
the version output as fixtures.

Wrote `gateway install [--version N]`: the release and digest rules in `src/domain/gateway.rs`,
two ports (`Downloads`, `GatewayStore`), the use case `application::gateway::Installer`, adapters
`Curl` and `FsGatewayStore`, and the command. The build store's lock, `current` link replacement
and leftover cleanup moved to `src/infrastructure/files.rs`, which both stores now use; its tests
are unchanged and pass. The three checks pass (280 tests).

Ran it on this machine: v263 downloaded, verified against the pinned digest, run and made current
in `~/.local/share/ai-ctl/llama-swap/`; a second run reports it current already; `--version 99999`
fails on the 404 of its checksums file before any download. Installed v262 to run the
checksums-file path for real (verified, current, `was v263`), went back to v263 without a download,
and removed `v262` by hand. Committed as `feat(gateway): install a pinned llama-swap release`.

The user ended the session here; the work goes on in a new one. 1 task of 13 done.

Where the next session starts: the task "Generate one llama-swap model entry per inventory model",
with the native build's `llama-server`, the model's flags on `${PORT}`, its health check and
`ttl`. Points to carry into it:

- `llama-swap -validate -config <file>` checks a generated configuration without starting
  anything: the snapshot tests and `up` can lean on it.
- v263's per-model `disableKeepAlives` (PR #1211) answers 502 `proxy error: EOF` after a streamed
  response; whether to set it for llama-server entries is open, to decide on what the first real
  requests show.
- The OpenCode acceptance criterion also confirms that llama-server passes tool-call arguments as
  an object to the 27B's template, which raises an error on a JSON string.
- Still the user's call, from 2026-10-09: whether ai-ctl should hold each model's revision and
  hashes in the inventory and check them. The commands and the pinned commits live in
  `docs/stack.md` §5 and §8 meanwhile.

Resumed in a third session the same day. Read llama-swap's configuration code at v263 (a clone
of the tag) before writing a generator for it, and found four constraints the phase file does not
state:

- `cmd` is split by `billziss-gh/golib/shlex` in its POSIX dialect, after every line starting
  with `#` is dropped and a trailing `\` joins two lines; single quotes keep a word literal.
- Every string of the file is searched for macros: `${env.NAME}` in the raw text before the YAML
  is parsed, then `${name}` in each value, and an unknown name fails the load. Nothing escapes a
  `$`, so a path or a flag value containing `${` cannot be written at all.
- The decoder is not strict: an unknown or misspelled key is ignored, so `llama-swap -validate`
  cannot catch one. Only the snapshot tests can.
- `matrix` now sits under `routing.router.settings`, with `routing.router.use: matrix`; the
  top-level `matrix:` key is kept for compatibility.

Read llama-swap #1205 behind `disableKeepAlives`: llama-server advertises keep-alive on a
streamed response, then closes the connection; a request sent within ~30 ms reuses the closed
connection and gets 502 `proxy error: EOF` (3 in 156 streamed requests in the report). The cause
is llama.cpp #29680, still open today. An agent sends requests back to back, so the user chose to
set `disableKeepAlives: true` on every llama-server entry now rather than wait to observe it,
with the reason written where the setting is, so it can be removed once #29680 is fixed. The
user also chose a `gateway config` command that prints the generated configuration, and
eviction costs derived from `vram_mib` for the `matrix`.

Wrote the generator as a domain module, `src/domain/gateway_config.rs`: from the inventory and
the program each backend runs, one entry per model, in the inventory's order — the native build's
`llama-server` with `--host 127.0.0.1 --port ${PORT} --alias <id> --metrics`, the model's files
(`model` first), `--ctx-size`, then the kind's defaults overlaid with the model's flags, `false`
leaving a flag out — with `proxy` on `127.0.0.1:${PORT}`, `checkEndpoint: /health`, the model's
`ttl`, `disableKeepAlives: true` under a comment giving its reason, and the preloaded models
under `hooks.on_startup.preload`. The YAML is written by hand: every string double-quoted, the
command as a literal block with one option per line and each word quoted for the POSIX splitter
only when it needs it, so no YAML crate is needed. The inventory now reserves `alias` and
`metrics`, which ai-ctl sets. The use case `application::gateway_config::Configurator` resolves
the current build of each backend a model uses, and only those: the shipped inventory's image
backends, which no model uses, need nothing; a model on an image backend fails, "not supported
yet", until the next task. `ai-ctl gateway config` prints the result. The inventory fakes of the
doctor's tests moved to `application/fakes.rs`, shared with the new tests.

Run on this machine, `gateway config` gives both models on `b11538`, and
`llama-swap -validate` accepts it (`config is valid: 2 model(s)`). To check the quoting against
llama-swap's own splitter, ran v263 on a scratch configuration whose `cmd` was a script printing
its arguments, with the words a model ID or a path can hold — quote, double quote, `#`, space,
backslash, empty word, `é` — and sent it one request: every word reached the program intact.
`llama-swap -h` shows that it listens on `:8080`, every interface, unless `-listen` says
otherwise: `up` must pass the inventory's address.

`cargo test` then failed once in `infrastructure::host` and passed on the next run. Looped the
suite: at `docs: record the phase 3 session and where the next one starts`, before any change of this session, the lock test of `files.rs` failed 9
times in 30 runs; alone, 0 in 40. Cause: the tests run in parallel threads, and a child process
forked by one test shares every file the others hold open until it runs its program — a lock
released by `drop` stays held, and a script just written is busy (`ETXTBSY`, the `host.rs`
failure, about 1 in 60). Every child of ai-ctl now starts through `command::spawn`; in the
crate's tests it takes a shared lock until the child runs its program, and the three tests that
release a lock or write a script take it exclusively meanwhile (`command::exclusive`). The four
spawns written directly in tests go through it too. 0 failures in 60 runs afterwards. The three
checks pass (297 tests).

The user asked to commit in two commits and stop here: the test fix, then the generator with its
documentation and this report. 2 tasks of 13 done.

Where the next session starts: the task "Generate image-based entries for the comparison
runtime". `Configurator::runtime` refuses an image backend ("not supported yet") and
`Runtime` has only `Native`: both grow there. To settle with the user before writing it:

- how the read-only mount of `/ai/models` gets past SELinux: `:z` relabels the models directory,
  `--security-opt label=disable` leaves it alone;
- the network: `-p 127.0.0.1:${PORT}:8080`, or `--network host` with the same
  `--host 127.0.0.1 --port ${PORT}` as the native build;
- pulling: `podman run` pulls a missing image within llama-swap's 120 s `healthCheckTimeout`;
  `--pull=never` with an explicit pull beforehand is the alternative;
- `cmdStop`: `podman stop` of a container named after the model, which `--rm` then removes.

Then the `matrix` task, with eviction costs from `vram_mib` (decided), under
`routing.router.settings.matrix` with `routing.router.use: matrix`. For `up`: llama-swap listens
on `:8080`, every interface, unless `-listen` gives the inventory's address. Still the user's
call, from 2026-10-09: whether ai-ctl should hold each model's revision and hashes and check them.

Resumed in a fourth session the same day, on the image entries. Checked the four open points
before putting them to the user: SELinux is enforcing and `/ai/models` is `unlabeled_t`; the
image's `HEALTHCHECK` curls `localhost:8080` (`.devops/cuda.Dockerfile` at `b11538`); llama-server
stops on SIGTERM (`tools/server/server.cpp:515`); llama-swap v263 documents `cmdStop` with
`podman stop` for containers, and its default `unloadTimeout`, 10 s, is `podman stop`'s. The user
chose `--security-opt label=disable`, `--network host`, and `--pull=never` with `up` pulling a
missing image; `--rm --replace --name ai-ctl-<id>` with `cmdStop: podman stop ai-ctl-<id>` went
as proposed, with `--no-healthcheck` added, since on the host's network the image's check probes
the gateway.

`Runtime::Image` joins `Runtime::Native`: the runtime gives the program — the build's binary, or
`podman run` with its options and the image — and the backend's kind gives the arguments, so an
image runs the same llama-server command as a native build. A model on an image is refused when
its ID cannot name a container (letters, digits, `_`, `.`, `-`) or the models directory contains
`:`, which a volume cannot hold. The use case resolves an image backend without a build. The
doctor's CDI check now names the generator's device constant, `GPU_DEVICE`.

Checked on this machine: with the 35B moved to `llama-cuda13` in a scratch copy of the inventory,
`gateway config` gives the container entry and `llama-swap -validate` accepts it. No llama.cpp
image is pulled yet, so the podman options ran on `node:22-bookworm-slim`, already in local
storage: the container read a GGUF's magic, listed the GPU and could not write to `/ai/models`.
Without `label=disable` the same container could neither read the models nor initialise NVML
("Insufficient Permissions"), so the option is needed for the GPU as much as for the models. No
host label changed. Not checked: an image serving a request through llama-swap, and
`podman stop` run by llama-swap; both need an image pulled, which `up` will do. Wrote the command
and the reason for each option into `docs/stack.md` §4.4. The three checks pass (301 tests).
3 tasks of 13 done.

Went on with the `matrix`. Read its code at v263: a set is an expression (`a & b`, `|`, `+ref`)
whose terms take letters, digits, `_`, `-` and `.`; a set allows any subset of itself; a model in
no set runs alone; `evict_costs` are positive integers, 1 for a model left out; a matrix needs at
least one set. The inventory accepted any ID without whitespace, which a term cannot always hold,
nor a container name: model and backend IDs are now a letter or a digit followed by ASCII letters,
digits, `.`, `_` and `-`, checked with the inventory's other rules, and the container-name error
added earlier the same session went. The shipped inventory complies.

The budget needed the GPU's memory: NVML gives 24463 MiB, of which the driver reserves 388, which
no process can allocate; nvml-wrapper 0.13 already reads `reserved`, so the probe's `Gpu` now
carries it and gives `usable_mib()`, 24075 here. For the margin, the user chose an inventory field,
`[gateway] vram_margin_mib`, 1024 by default: the shipped `fit-target`, so a model started beside
another finds the memory its `--fit` keeps free and moves no layer to the CPU. `listen` became
optional within `[gateway]`, so either setting can be given alone.

`domain::gateway_config` now writes one set per largest combination whose `vram_mib` fit in the
usable memory less the margin, plus a set of its own for a model too large for it, ordered by the
inventory's order, each model's `vram_mib` as its eviction cost, under
`routing.router.settings.matrix` with a comment giving the figures; no set, no `routing` section.
The search walks the combinations in index order and takes all the remaining models at once when
they all fit. `Configurator` reads the GPU through the `GpuProbe` port and fails, with `doctor` as
the remedy, when NVML cannot answer.

On this machine, the 27B and the 35B need 43827 MiB together: `gateway config` gives one set
each, and `llama-swap -validate` accepts it. With two small models added to a scratch inventory,
the sets came out as computed by hand (`qwen3.6-35b-a3b & tiny-a`, `qwen3.6-35b-a3b & tiny-b`,
`tiny-a & tiny-b`, the 27B alone) and validated. Since llama-swap ignores unknown keys, checked
that it reads this block: an unknown model in a set and an unknown engine in `use` are both
refused. Documented the matrix in `docs/stack.md` §6.1 and §6.4, the new field and the ID rule in
the README and the shipped inventory; the README's list of flags ai-ctl sets was missing `alias`
and `metrics`, reserved earlier in this phase. The three checks pass (308 tests). 4 tasks of 13
done.

The user asked to record the work and commit, and ended the session here; the work goes on in a
new one. Both tasks went into one commit with this report: the matrix work rewrote code of the
image task (the container-name check went with the stricter IDs), so the state between them was
never kept.

Where the next session starts: the task "Snapshot tests of the generated configuration for the
shipped inventory" — the generator is pure, so the shipped inventory with a fixed build path and
24075 MiB gives a stable file to compare, and `llama-swap -validate` cannot catch a misspelled
key. Then `up`, which pulls missing images first, passes `-listen` with the inventory's address
(llama-swap listens on every interface otherwise), and regenerates the file. Still the user's
call, from 2026-10-09: whether ai-ctl should hold each model's revision and hashes and check them.

Resumed in a fifth session the same day, on the snapshot tests. `tests/gateway_config.rs` renders
the shipped inventory with `/ai/models`, gives every build backend a fixed `llama-server` path laid
out as the build store lays it out and every image backend its image, generates with 24075 MiB,
and compares `to_yaml()` with a file of `tests/snapshots/`: `gateway-shipped.yaml` for the
inventory as shipped, and `gateway-shipped-image.yaml` with the 35B moved to `llama-cuda13`, so the
container entry and its `cmdStop` are pinned too. `UPDATE_SNAPSHOTS=1` rewrites them instead of
comparing; no crate added. On a difference, the test names the file, the first differing line and
both versions of it. Comparing with `lines()` first left a missing final newline with
`<end of file>` on both sides; it splits on every `\n` now, and shows `""` against `<end of file>`.
Checked both messages by hand-editing a copy of the snapshot (`checkEndpoint` misspelled, last
newline removed).

On this machine, `ai-ctl gateway config` differs from the first snapshot by the build path only,
on its two lines, and `llama-swap -validate` (v263) accepts both snapshots. The snapshots and the
command to rewrite them are documented under Testing in `docs/architecture.md`. The three checks
pass (310 tests). 5 tasks of 13 done.

Went on with `up`. Read llama-swap's `main` and server at v263 for what `up` and `down` lean on:
SIGINT and SIGTERM shut it down gracefully within 30 s, then it forces its exit 5 s later; SIGHUP
reloads the configuration, and a reload shuts the old server down, stopping its models, while the
new one already takes requests. Each upstream runs in its own process group (`Setpgid`), with no
parent-death signal: a llama-swap killed with SIGKILL leaves its llama-server processes running,
holding their VRAM. `/health` answers 200 `OK` as soon as the server listens, whatever the
models; `/running` gives `{"running": [{"model", "state", …}]}`. By default (`logToStdout:
proxy`) its standard output carries only its own messages; a model's output stays in a buffer in
memory, served by `/logs/stream/<model>` and lost when the gateway stops.

Put four choices to the user, who took the recommendation each time: a small HTTP/1.1 client on
`std::net::TcpStream` for llama-swap's API; llama-swap detached in its own process group
(`process_group(0)`) rather than a new session; `up` on a running gateway does nothing when the
configuration is unchanged and refuses, naming `down` then `up`, when it changed; the generated
file sets `logToStdout: "proxy,upstream"`.

Ran llama-swap v263 on a scratch configuration to capture what `up` reads: `/health` answers
`HTTP/1.0 200` with `Content-Length: 2` and `OK`, `/running` gives `{"running":[]}`; both are
fixtures now. Two findings: a configuration whose `proxy` names `${PORT}` without `cmd` naming it
is refused at load, exit 1 with one log line; and llama-swap logs `listening on …` before it
binds, then exits 1 on `address already in use`, so its log cannot tell `up` that it listens.

The generated file sets `logToStdout: "proxy,upstream"` under a comment; the snapshots were
rewritten and `llama-swap -validate` still accepts them. `Configurator::prepare` gives the
configuration with the inventory's address and the images the models run, deduplicated;
`generate` returns its configuration. Three ports: `GatewayProcesses` (the record, liveness, the
port, starting, signals, the log), `GatewayApi` (`healthy` for now) and `Images`. The use case
`application::lifecycle::Lifecycle::up` takes the gateway's lock, prepares the configuration, finds
the current llama-swap, then: a recorded gateway that runs on the same configuration, address and
release is reported as running; one that runs on another is refused; a record whose PID has
exited or now runs another program is removed, with an event. Then it refuses a taken address,
pulls each missing image, starts llama-swap and polls `/health` every 100 ms for 30 s. A gateway
that exits first is forgotten, and the error quotes the last 15 lines of its log; one that never
answers gets SIGTERM, then SIGKILL after llama-swap's own 35 s, and is forgotten too. The polls are
counted, and the pause between them is a function the tests replace, so no test waits.

Adapters: `LocalGateway` keeps `gateway.json` and `llama-swap.yaml` in
`~/.local/state/ai-ctl/gateway/` (written by a rename) and `gateway.log` in `logs/`, the previous
one kept as `gateway.log.1`; it starts llama-swap with `process_group(0)`, standard input on
`/dev/null` and both outputs in the log, in its state directory; liveness reads
`/proc/<pid>/exe`, a removed binary (`… (deleted)`) still counting as the gateway; the port is free
when a `TcpListener` can bind it; signals go through `kill -s`. `LlamaSwapApi` sends HTTP/1.0
requests over `TcpStream`, with the response parser tested on the fixtures. `Podman` runs `podman
image exists` and `podman pull --quiet`. `ai-ctl up` prints its steps on stderr and the address,
the PID, the models loading in the background and the log on stdout.

The full suite then failed once in 1: `a_taken_port_is_not_free` found the port still bound after
its listener was dropped — the fork problem fixed earlier this phase, for a socket this time: a
child forked by a parallel test holds the listener until it runs its program. The test releasing a
socket now holds `command::exclusive()`, as the nothing-listening test of the client does; 0
failures in 20 runs of the suite afterwards.

Checked on this machine with a scratch copy of the inventory (port 18080, nothing preloaded) and a
scratch state directory: `up` returned in 0.13 s, llama-swap running in its own process group and
reparented once ai-ctl exited; a second `up` reported it running; with a model's `ttl` changed it
refused; after a `kill -9` of the gateway it removed the record and started a new one; with
another program on the port it refused, and on port 80 it reported the permission error. The
first attempt at the taken-port case tested nothing: Python could not bind the port, left in
TIME_WAIT by the health checks, without `SO_REUSEADDR`, and the port-80 case met the running
gateway; both were run again with the gateway stopped. `podman image exists` answers 0 for a local
image, 1 with nothing on stderr for a missing one, 125 with a message for a malformed name, as
the adapter reads them. Not run for real: a pull (no model of the inventory is on an image) and a
gateway exiting before it answers. The three checks pass (339 tests). 8 tasks of 13 done: `up`,
the detection of a stale record, and the regeneration of the configuration, which `up` writes
afresh each time it starts the gateway and compares with the running one otherwise.

The user asked to commit: the snapshot tests and `up` go into one commit with this report.

Where the work goes on: `down` — `POST /api/models/unload`, SIGTERM, SIGKILL after a timeout
(`Lifecycle::stop_started` already does the signals for a gateway `up` started; `down` stops one
it did not start, through the record and `signal`), then the VRAM. Open, put to the user: what
counts as "the VRAM released". Proposed: the GPU processes NVML lists for the gateway's models
before the stop, waited for until none is left, so the display or another program on the GPU
cannot hold `down`; the alternative is no compute process at all in NVML's list. `/running` and
`/api/models/unload` need fixtures captured with a model loaded. Still the user's call, from
2026-10-09: whether ai-ctl should hold each model's revision and hashes and check them.

Committed as `feat(gateway): start the gateway with up, pinned by snapshot tests`. The user chose the proposal for `down`: the gateway's GPU processes,
noted before the stop, waited for until none is left. Read llama-swap's unload at v263 first:
`POST /api/models/unload` blocks until every model is stopped — SIGTERM to its process group, or
its `cmdStop`, then SIGKILL to the group after its `unloadTimeout` — and does not wait for the
requests in flight, so a `down` cuts a generation short. The NVML probe already lists each GPU
process with its PID and memory. The gateway's GPU processes are then the descendants of
llama-swap, its native models, and the processes of the `ai-ctl-*` containers, which run under
conmon and descend from no process of the gateway. Checked with a throwaway container of a local
image named `ai-ctl-probe`: `podman ps --filter name=^ai-ctl- --format {{.Pid}}` gives the host
PID of its process, whose parent is conmon, whose parent is the user's systemd; nothing when no
name matches. Captured the unload answer, `{"msg":"ok"}`, as a fixture.

`Lifecycle::down`: takes the lock; no record, or a record whose process is gone (removed, with
the same event as `up`), is "no gateway running". Otherwise it reads the GPU before anything
stops and keeps the processes on it that are the gateway's: llama-swap's descendants, read from
`/proc/*/stat`, and the containers' PIDs from podman with theirs. Then `POST /api/models/unload` —
a failure is a warning, since SIGTERM stops the models too — SIGTERM, SIGKILL after 40 s, the
record removed, and the GPU polled every 100 ms until none of those processes is on it, for 30 s
at most. A process still there is an error naming its PID and `kill <PID>`: ai-ctl stops no
backend itself. A GPU that cannot be read before the stop is a warning, and `down` then waits for
nothing. The stop shared by `up` and `down` takes the test of whether the gateway has ended:
`up` waits on its child, `down` reads `/proc/<pid>/exe`. The port `Images` became `Containers`,
with the PIDs. `ai-ctl down` prints the steps on stderr and the memory released on stdout.

Checked on this machine with the scratch inventory on port 18080: `up`, `down` (0.08 s, no model
loaded), and `down` again, "No gateway running". Then, the GPU idle (gnome-shell, 4 MiB), `up`,
a chat request naming `qwen3.6-35b-a3b` — loaded and answered in 6.8 s, 172 tokens/s in
generation, the model holding 19876 MiB as PID 144121 — and `down`: unload, SIGTERM, the wait for
PID 144121, "19876 MiB of GPU memory released" in 0.53 s, and NVML back to 15 MiB used. The log
held the model's output, from loading to its timings, as `logToStdout` was meant to. Captured
`/running` with the model loaded as a fixture for `status`. Not run for real: a container's
process on the GPU, a llama-swap outliving SIGTERM, a process left on the GPU. The three checks
pass (354 tests). 9 tasks of 13 done.

The user asked to record the work and commit, and ended the session here; the work goes on in a
new one. Nothing runs: no llama-swap, the GPU back to 15 MiB. The scratch inventory and state
directory of the checks were under the session's scratchpad, not in the user's directories; the
real `~/.config/ai-ctl` and `~/.local/state/ai-ctl` were not touched.

Where the next session starts: `status` — gateway process, address, models from `/running` (a
fixture with a model loaded is in `tests/fixtures/llama-swap-263-running.http`), VRAM used — which
reuses `Lifecycle`'s record and liveness, cleaning a stale record as `up` and `down` do; then
`logs [-f] [model]`, where the gateway's log already holds the models' output and
`/logs/stream/<model>` streams one model (the HTTP/1.0 client reads until the server closes, so
`-f` reads the body as it comes). Then the integration test with llama-swap and a fake upstream,
and `doctor`, `deploy/opencode.json`, `docs/stack.md` and the README for the commands. The
acceptance criteria are still to check on the real address, `127.0.0.1:8080`, with the main
model preloaded: the swap between the two models, OpenCode (its tool calls through the 27B's
template) and OpenWebUI. Still the user's call, from 2026-10-09: whether ai-ctl should hold each
model's revision and hashes and check them.

After the commit (`feat(gateway): stop the gateway with down, once its GPU memory is released`), the user settled that question: not now. A model library managed
by ai-ctl may be worth a roadmap of its own later, not planned yet; the user asked to record it in
the documentation and in a note, so it is not lost among this report's decisions. Wrote
`docs/notes/model-library.md`, a new kind of document (a direction considered, not a decision):
where it comes from, what a library could cover (pins, downloads, checks, template review,
updates, archiving), and what a roadmap must settle first — v1's dropped model registry and the
roadmap's exclusion of Hugging Face downloads, llama-server's own `-hf`, where the pins live,
formats other than GGUF, disk space. `docs/stack.md` §5 now says ai-ctl neither downloads nor
verifies models and points at the note; `docs/architecture.md` lists `docs/notes/`. CLAUDE.md
names `docs/notes/` under "Where Things Are"; that file is ignored by git, so the line stays on
this machine. No phase of this roadmap changes: its scope already leaves downloads out.

### 2026-10-11

Resumed in a new session, on `status` and `logs`. Read llama-swap's code at v263 for what they
lean on. `/running` lists every model that is not stopped, in the state `starting`, `ready` or
`stopping`. `/logs/stream/<model>` writes the model's history — a 100 KB ring buffer in memory,
kept since the gateway started, across the model's loads — then the live output, until the
client goes or the gateway shuts down; no endpoint gives a model's history without the stream.
The handler sets `Transfer-Encoding: chunked` itself, which Go's server drops for an HTTP/1.0
request, closing the connection at the end instead, so ai-ctl's client can read the stream; to
confirm on a running gateway. In `gateway.log` a model's lines are llama-server's own, with no
prefix (read in the log of the 2026-10-10 checks): the file cannot be filtered by model.

Raised with the user: a gateway killed with SIGKILL leaves its llama-server processes on the GPU,
and an `up` after it removes the stale record and starts a new gateway, whose model then finds
less memory than its `vram_mib` and is moved partly to the CPU by `--fit`, with no error. Put
three choices to the user, who took the recommendation each time: `logs <model>` is always a
stream, history then live, `-f` applying to the gateway's log only; `status` lists the processes
an earlier gateway left on the GPU, with `kill <PID>`, and `up` refuses to start while there are
any; `status` exits 0 when the gateway runs and 3 when it does not, as `systemctl status` does.
`status` takes the gateway's lock, as `up` and `down` do, to remove a stale record without racing
them.

Before writing the stream client, ran llama-swap v263 on a scratch configuration whose model was a
fake upstream (a Python script answering `/health` and `/v1/chat/completions` and printing a line
per request), and read `/logs/stream/fake` and `/logs/stream/nope` with raw HTTP/1.0 requests. The
first answers `HTTP/1.0 200` with no `Transfer-Encoding` and no `Content-Length`, the history,
then the live lines; the second `400` with a JSON error, "invalid logger. Use 'proxy',
'upstream', 'http' or a model's ID". On SIGTERM llama-swap closed the open stream and stopped the
upstream. Both answers are fixtures now. A first attempt at the SIGTERM ran `pkill -f` with a
pattern that also matched the shell running it, which ended with it; llama-swap got the signal
all the same, and the check was read from the files it left.

Ports: `GatewayProcesses` gains `program` (`/proc/<pid>/exe`, `None` for a process gone or of
another user) and `read_log` (a bounded read from an offset, `None` when the log is shorter than
the offset: replaced); `GatewayApi` gains `running` and `stream_log`; `BuildStore` gains
`contains`, true for any path under the store, a removed build's `… (deleted)` included. The HTTP
client's header parsing became its own function, shared by the whole-response parser and
`copy_stream`, which copies a body to its writer as it arrives, flushing each piece. The domain's
ID rule is public, `is_valid_id`, so `logs <model>` refuses a word that is not an ID before it
reaches the request line.

`Lifecycle` gains `status`, `log` and `model_log`. The GPU processes are now sorted out once, in
`owners`: the running gateway's (llama-swap, its descendants and the containers' processes), and
the containers' alone. `held` keeps the gateway's, as `down` waited for before; `leftovers` keeps
the programs of the build store the running gateway does not own and, when none runs, the
containers' processes. `up` reads the GPU after checking the port and refuses while there are
leftovers, naming them and `kill`. `status` also says when the running gateway's configuration,
address or release is no longer what `up` would give it — the comparison `up` makes, now shared
— and names `down` then `up`; a part it cannot read (the API, the GPU, the configuration) is a
warning, and the rest is reported. `log` copies the file in pieces of 1 MiB; following, it checks
the gateway, then reads, every 100 ms, so the lines written before it stopped are read, and ends
when it stops or a shorter log took its place. `ai-ctl status` prints aligned lines and exits 3
when no gateway runs; `ai-ctl logs` writes to standard output, ends quietly when the reader goes
(`closed_pipe` finds a broken pipe through the error's context), and says on stderr why a follow
ended.

Checked on this machine with the scratch inventory on port 18080 (models not preloaded) and a
scratch state directory: `status` with nothing (exit 3), `logs` with no log (exit 1, naming
`up`); `up`; `status` with no model. Then a chat request naming `qwen3.6-35b-a3b`, with
`logs -f` and `logs qwen3.6-35b-a3b` running meanwhile: `status` read the model `starting` at
16662 MiB, then `ready` at 19876 MiB, all of it the gateway's. With a model's `ttl` changed in the
inventory, `status` gave the note; restored. `down` (0.30 s, 19876 MiB released) ended both
streams, "The gateway stopped" and "The gateway ended the stream", and what `logs -f` printed was
the log file byte for byte. Then `up`, the model loaded, and `kill -9` of llama-swap: `status`
removed the record, reported the gateway stopped, exit 3, and the llama-server left on the GPU,
PID 379737, 19876 MiB, with `kill 379737`; `up` refused with the same PID; `logs <model>` named
`ai-ctl logs`. After `kill 379737` — a process this check had started — `up` started,
`logs | head -2` exited 0, and `down` left the GPU at 15 MiB with no llama process. Not run for
real: a container left on the GPU, and a reader going in the middle of a long log (the broken-pipe
test is a unit test). The three checks pass (386 tests), and the suite passed 20 runs in a row.
11 tasks of 13 done.

Where the work goes on: the integration test of `up`, a proxied request and `down` with
llama-swap and a fake upstream — what the fake upstream is written in is open (the Python script
of this session's capture, or the test binary run again as the upstream); then `doctor`,
`deploy/opencode.json`, `docs/stack.md` and the README for `up`, `down`, `status` and `logs`, none
of which the README documents yet. The acceptance criteria are still to check on
`127.0.0.1:8080` with the main model preloaded: the swap between the two models, OpenCode (its
tool calls through the 27B's template) and OpenWebUI.

The user asked to commit (`feat(gateway): read the gateway with status and logs`) and chose the test binary, run again, as the fake upstream of
the integration test. The generated command puts llama-server's arguments after the binary,
which libtest would refuse as unknown options, so `tests/gateway_lifecycle.rs` runs without the
harness (`harness = false`): started with `--alias`, it is a fake llama-server answering
`/health`, `/v1/models` and `/v1/chat/completions` on its `--port`, one connection at a time,
printing a line per request; otherwise it runs the scenario, and answers `--list` and a filter as
a test binary does. The scenario drives `Lifecycle` with the real `LocalGateway`, `LlamaSwapApi`
and `InventoryFile` (a TOML file of one model, `fake`, in a scratch directory with the gateway's
state), the installed llama-swap read through `FsGatewayStore`, and three doubles: the current
llama.cpp build is the test binary, no container runs, and the GPU lists every process running
the test binary with `--alias`, holding 1024 MiB each, so `down` waits for the fake llama-server
to exit as it waits for a real one. The build double's `contains` is false, so another run's fake
servers are never this run's leftovers. It checks `up`, `status` before a request, the request
answered through the gateway, `status` with the model ready and its process held, the model's
lines in the gateway's log, `down` releasing that process with both processes gone, `status`
after it, and no warning on the way. A guard sends SIGTERM to the gateway when the scenario fails
before `down`. Skipped, with the reason, when no llama-swap is installed or port 5800 — the port
llama-swap gives its first model, `startPort` — is taken: a real gateway's model there would
answer in the fake's place.

It passed in 0.6 s. Broke one expectation on purpose: the failure quoted the gateway's log, and
the guard left no process. Run with an empty `HOME` it was skipped (llama-swap not installed), and
with a listener on 5800 skipped too. The ports llama-swap's health checks leave in TIME-WAIT
held Python's bind, which sets no `SO_REUSEADDR`; Rust's `TcpListener` and Go's set it, so the
test's own check and the next run are not held. Documented in `docs/architecture.md` under
Testing. The three checks pass (365 tests in the crate, 21 in `tests/`, and the scenario), and the
whole suite passed 20 runs in a row with no skip. 12 tasks of 13 done.

Where the work goes on: the last task. `deploy/opencode.json` already points at
`http://127.0.0.1:8080/v1` and `docs/stack.md` already describes the gateway, so it is mostly
`doctor` and the README; whether the model IDs in `opencode.json` match the inventory's is to
check. For `doctor`, read-only, to settle with the user: a Gateway section with llama-swap
installed (a failure without it, since `up` needs it) and its release against the pinned one;
the gateway running or stopped, a stopped one being no failure, and a stale record reported, not
removed; a running gateway out of date; the inventory's address taken by another program; the
processes an earlier gateway left on the GPU — which of these fail and which warn. Then the
acceptance criteria on `127.0.0.1:8080`.

The user asked to commit the test (`test(gateway): run up, a proxied request and down against llama-swap`) and approved the proposed `doctor` checks: llama-swap
missing fails, another release than the pinned one warns; a stopped gateway is ok, an out-of-date
one or a stale record warns, the record left for `status` to remove; the inventory's address taken
by another program fails; processes an earlier gateway left on the GPU fail, with `kill`.

`Lifecycle` gains `inspect`, what `status` reads without the lock and without removing a stale
record, which it returns with what runs under its PID; `status` and `inspect` share the reading.
`Doctor` gains the four gateway ports and builds a `Lifecycle` on them; its Gateway section, after
Inventory, has four checks: `llama-swap`, `Gateway`, `Gateway address` and `GPU leftovers`. The
warnings `inspect` emits are left out of the report: each is either reported by a check of its
own (the GPU, the gateway's API, the inventory, podman) or comes from `/proc`. Running `doctor`
with a model loaded showed a gap: "Idle VRAM" would have warned about the gateway's own model,
with the remedy of moving the display off the NVIDIA GPU, so it now leaves out the processes the
Gateway section reports, the gateway's and the leftovers, and names the memory they hold. The
gateway doubles of the lifecycle's tests moved to `application/fakes.rs`, shared with the
doctor's; `FakeReleases` became a struct, to give a current release whose binary is missing.

Checked on this machine: `ai-ctl doctor` on the real configuration, 22 checks ok, the gateway
stopped, `127.0.0.1:8080` free. On the scratch inventory with `qwen3.6-35b-a3b` loaded, the Gateway
check named the model `ready`, the address was the gateway's, and "Idle VRAM" stayed ok with the
model's 19876 MiB set apart. After a `kill -9` of llama-swap, `doctor` exited 1: the stale record
warned, `GPU leftovers` failed naming the llama-server and `kill`, and `gateway.json` was still
there afterwards. The leftover, started by the check, was ended; `status` then removed the record,
and `doctor` came back to 22 ok with the GPU at 15 MiB.

The documents: `deploy/opencode.json` already pointed at `http://127.0.0.1:8080/v1`, with the
inventory's model IDs and limits within each `ctx_size`; only its provider's name, "llama-server
(local)", still named the old setup, now "Local gateway (llama-swap)". `docs/stack.md` gains step 7
of the machine setup, the gateway's commands, and §7.2 says why OpenCode's model IDs are the
inventory's and gives the 35B's limits. The README documents `up`, `status`, `logs` and `down`
under "Run the gateway", with the gateway's files, and the doctor's table gains the Gateway row.
The three checks pass (374 tests in the crate, 21 in `tests/`, and the scenario), and the suite
passed 10 runs in a row. 13 tasks of 13 done; the phase stays open until the acceptance criteria
are checked.

Where the work goes on: the acceptance criteria, on the real configuration — `127.0.0.1:8080`, the
27B preloaded: `up` and a chat request to the 27B; a request to the 35B swapping them, with never
both resident; `down` returning once NVML shows the VRAM free; `status` after a gateway killed
outside ai-ctl; then OpenCode, its tool calls through the 27B's template included, and OpenWebUI,
which the user runs. Then the phase's closure.

The user asked to record the session and commit, and ended it here: the next session starts with
acceptance criteria 1 to 4 on the real gateway, which loads the 27B (about 22 GB of VRAM) while
they run. Nothing runs: no llama-swap, the GPU at 15 MiB. The checks of this session ran on a
scratch inventory and state directory under the session's scratchpad; the real
`~/.config/ai-ctl` and `~/.local/state/ai-ctl` were only read, by `doctor`.

Resumed in a new session the same day to close the phase, on the real configuration:
`~/.config/ai-ctl/inventory.toml`, `127.0.0.1:8080`, the 27B preloaded, run with the release
binary. The three checks passed first (374 tests in the crate, 21 in `tests/`, and the scenario).
`llama-swap -validate` accepted the output of `gateway config`; the GPU was idle (28 MiB) and
nothing listened on 8080. An `nvidia-smi` loop sampled the GPU's compute processes every 200 ms
through acceptance criteria 1 to 4:

- `up` returned in 0.17 s, the 27B loading; a chat request naming `qwen3.8-27b` succeeded, the
  model loaded in 11.9 s by llama-server's log: 63 tokens/s in generation, MTP drafting (91 of
  148 draft tokens accepted), 21302 MiB held, under its `vram_mib` of 22733.
- A request naming `qwen3.6-35b-a3b` answered in 9.9 s, the swap included. llama-swap logged the
  matrix's choice, `set=fit-2 … evict=[qwen3.8-27b]`; of 239 samples with a process on the GPU,
  none had two: the 27B was last seen at 10:52:45.5, the 35B first at 10:52:46.6. 158 tokens/s in
  generation, 19876 MiB.
- `down` returned in 0.34 s, after waiting for the 35B's PID to release its 19876 MiB; NVML read
  28 MiB right after, and no llama process was left. `status` then exited 3.
- `up` again, the 27B loaded by a request, then `kill -9` of llama-swap: `status` removed the
  record, reported the gateway stopped (exit 3) and named the llama-server left on the GPU with
  `kill <PID>`; `up` refused, naming the same PID; `doctor` failed on "GPU leftovers" while "Idle
  VRAM" set that memory apart. The leftover, started by this check, was ended with `kill`, and
  the GPU came back to 28 MiB.

llama-swap gave the 35B port 5800 and the 27B port 5801: it numbers the models in the order of
their IDs, not the inventory's. The integration test's skip on port 5800 still holds, since its
configuration has one model.

The fifth criterion could not be checked with the clients themselves: OpenCode is not installed
on this machine (`~/.opencode/bin`, on the `PATH`, does not exist), its configuration in
`~/.config/opencode/opencode.json` is still v1's (an `llm-ctl` provider on port 9000 with v1's
models), and the `openwebui` container has been stopped for five months. Checked instead, with a
Python script through the gateway and for each model, what both clients send: `/v1/models` lists
both models; a request offering a `read` tool got a tool call whose arguments are a JSON string;
a second turn, streamed, sending that call back with its arguments as a string and the reasoning
in `reasoning_content`, as OpenCode does, with the tool's result, was answered from that result.
No template error, and none in the gateway's log: llama-server hands the 27B's template the
arguments as an object, as `common/chat.cpp:1337` led to expect. `down` left the GPU at 28 MiB.

The user chose to close the phase with the fifth criterion open, and to move the check with the
real clients to phase 4 as a task.

---

## Decisions

- **Models are grouped by use under one root, `/ai/models`**, the inventory's `models_dir`: one
  directory per model, `llm/<id>/` now, `tts/`, `stt/` and `image/` for the later roadmaps. Every
  backend shares the root, and an image entry mounts it once. Taken with the user.
- **What the inventory does not use is archived, not deleted**, under `/ai/archive/`; the user
  removes it later. `HF_HOME` is `/ai/cache/huggingface`: a cache and a token, not a model.
- **GGUF files come from Unsloth, and every download is verified before use**: repository
  `unsloth/…`, the commit recorded, each file's SHA-256 equal to the Hub's LFS hash, the embedded
  chat template read and compared with the base model's (user's decision; `docs/stack.md` §5).
- **The models are pinned at the commits checked on 2026-10-10**, written in `docs/stack.md` §8
  (`unsloth/Qwen3.8-27B-GGUF` at `4ca7207`, `unsloth/Qwen3.6-35B-A3B-GGUF` at `a483e9e`); a newer
  commit replaces them only after the same check.
- **The Unsloth chat templates are used as shipped**, the three changes Unsloth does not document
  included: none adds text to the prompt, and the 27B's `high` alias is what lets clients send
  `reasoning_effort: "high"`. The gateway's acceptance criteria test them, OpenCode's tool calls
  first (user's decision).
- **llama-swap is pinned at v263**, with its archive's SHA-256 in `src/domain/gateway.rs`; the
  pin moves with ai-ctl, and a test checks the digest against the release's fixture checksums.
  `--version N` installs another release, checked against the `checksums.txt` published with it.
  Linux x86_64 only.
- **Downloading, hashing and unpacking run `curl`, `sha256sum` and `tar`**, like git and CMake:
  no crate added for TLS, hashing or archives. `curl` is held to HTTPS for the URL and its
  redirects, retries transient errors and gives up on a transfer stalled for a minute; its tests
  read `file://` URLs, so none needs the network.
- **Releases sit side by side under `~/.local/share/ai-ctl/llama-swap/v<N>/`, with a `current`
  link**, as builds do, staged out of sight and installed by a rename. Installing a release that
  is there already makes it current without a download; there is no `--force` and no prune, since
  a release is one 43 MB file. The smoke test requires `llama-swap -version` to name the release.
- **Every llama-server entry sets `disableKeepAlives: true`**, against the 502 `proxy error: EOF`
  of llama-swap #1205, caused by llama.cpp #29680. The generated file carries the reason as a
  comment beside the setting, and it goes once a build in use includes a fix for #29680 (user's
  decision).
- **`ai-ctl gateway config` prints the configuration `up` will write**, without starting
  anything, so it can be read and passed to `llama-swap -validate` (user's decision).
- **The `matrix` eviction cost of a model derives from its `vram_mib`**: loading time follows the
  bytes read, which the weights dominate; no field to fill in, no I/O, a stable snapshot (user's
  decision).
- **A value containing `${` is refused when the configuration is generated**: llama-swap would
  read it as a macro, and offers no escape.
- **The generated file is llama-swap's YAML written by hand, in the domain**
  (`GatewayConfig::to_yaml`): double-quoted strings, the command as a literal block of
  POSIX-quoted words, one option per line. Like the checksums and `-version` parsers of
  `domain/gateway.rs`, it is pure text in a tool's format; no YAML crate.
- **A llama-server backend runs with `--alias <model id>` and `--metrics`**, both reserved in the
  inventory: llama-server names the model as clients do, and `/metrics` feeds the MTP acceptance
  rate and the bench.
- **The configuration names the current build's own directory**, `builds/llama.cpp/<id>/`, not
  the `current` link: a running gateway keeps the build it was configured with, and `up`
  regenerates the file when the current build changes.
- **An image entry runs `podman run --rm --replace --name ai-ctl-<id> --pull=never
  --no-healthcheck --network host --security-opt label=disable --device nvidia.com/gpu=all
  --volume <models_dir>:<models_dir>:ro <image>`**, then the arguments a native build gets, and
  llama-swap stops it with `podman stop ai-ctl-<id>`. SELinux separation, the network and the pull
  are the user's choices; the reasons are in `docs/stack.md` §4.4. The image runtime is not
  confined by SELinux: a confined container reaches neither the models nor NVML on this machine.
- **`up` pulls the missing images of the backends the models use**; the generated command never
  pulls. Added to the `up` task.
- **A model on an image needs an ID a container can be named after, and a models directory
  without `:`**: both refused when the configuration is generated, not when llama-swap loads it.
- **The `matrix` lists every largest combination of models whose `vram_mib` fit together in the
  GPU's usable memory less `[gateway] vram_margin_mib`**, and a model too large for it alone. The
  usable memory is NVML's total less what the driver reserves, read when the configuration is
  generated; the margin defaults to 1024 MiB, the shipped `fit-target` (user's choice of an
  inventory field). Evicting a model costs its `vram_mib`.
- **Model and backend IDs are a letter or a digit, then ASCII letters, digits, `.`, `_` and `-`**:
  an ID names a container and a term of the `matrix`, which take no other character. Refused with
  the inventory's other rules, so the generator assumes it.
- **The generated configuration is pinned by snapshot files, `tests/snapshots/*.yaml`, compared
  in full**: the shipped inventory, and the same with the 35B on an image. A deliberate change
  rewrites them with `UPDATE_SNAPSHOTS=1` and shows in the commit's diff. Written with the
  standard library; no snapshot crate.
- **ai-ctl talks to llama-swap's API with its own HTTP/1.0 client on `std::net::TcpStream`**: one
  request per connection, so no chunked body; a refused connection reads as "nothing listens".
  The gateway is on the loopback in plain HTTP, so no crate (user's choice).
- **llama-swap runs in its own process group, `process_group(0)`, not a new session**: std offers
  no `setsid` without `unsafe`, and the group keeps Ctrl-C and the end of the terminal's session
  from it; SIGHUP would only reload its configuration (user's choice).
- **`up` never restarts or reloads a running gateway**: on the same configuration, address and
  release it reports it running; on another, it refuses and names `down`, then `up`. A reload by
  SIGHUP stops the loaded models while the new server already takes requests (user's choice).
- **The gateway's log holds the models' output, `logToStdout: "proxy,upstream"`**: it is the only
  record left after a crash or a `down`; each `up` keeps the previous log as `gateway.log.1`
  (user's choice).
- **The gateway's files live in the state directory**: `gateway/gateway.json` (PID, binary,
  address) and `gateway/llama-swap.yaml`, written by a rename, with the lock `gateway/.lock`;
  `logs/gateway.log`. A recorded gateway runs while `/proc/<pid>/exe` names its binary.
- **`up` waits 30 s for `/health`, polling every 100 ms**; a gateway that never answers gets
  SIGTERM, then SIGKILL after 40 s. Signals go through `kill -s`, the only way std has to signal a
  process ai-ctl did not start, which `down` needs too.
- **ai-ctl holds no revision or hash of the models, and checks none, for now**: the check of
  `docs/stack.md` §5 stays manual. A model library managed by ai-ctl is a direction for a later
  roadmap, recorded in `docs/notes/model-library.md`; nothing in this roadmap prepares it (user's
  decision, 2026-10-10).
- **`down` waits for the gateway's own GPU processes, not for an idle GPU**: llama-swap's
  descendants and the `ai-ctl-*` containers' processes, noted before the stop, until NVML lists
  none of them; the display or another program on the GPU cannot hold `down` (user's choice).
- **ai-ctl never ends a model process itself, even after `down`**: a process of the gateway still
  on the GPU 30 s after llama-swap stopped is an error naming its PID and `kill <PID>`.
- **`down` stops even when the models cannot be unloaded or the GPU cannot be read**, with a
  warning: SIGTERM makes llama-swap stop its models, and stopping matters more than measuring.
- **`down` cuts short a generation in flight**: llama-swap's unload does not wait for requests,
  and `down` is the user's explicit stop.
- **Every child process starts through `command::spawn`**, which serialises it, in the crate's
  tests only, against tests that hold a file open; a test that releases a lock or writes an
  executable holds `command::exclusive()` meanwhile.
- **`ai-ctl logs <model>` is always a stream**: the model's history in llama-swap, then its live
  output, until Ctrl-C or the gateway stops; `-f` follows the gateway's log only. llama-swap
  serves a model's output only as a stream, and ending on a pause would depend on timing (user's
  choice).
- **A process an earlier gateway left on the GPU blocks `up`**: a program of ai-ctl's build store,
  or of an `ai-ctl-*` container while no gateway runs, that no running gateway owns. `status`
  lists them with `kill <PID>`, and `up` refuses until they are gone, since the new model would
  be moved to the CPU; ai-ctl never ends them itself (user's choice).
- **`ai-ctl status` exits 0 when the gateway runs, 3 when it does not**, as `systemctl status`
  does; 1 on an error (user's choice).
- **`status` says when the running gateway is out of date**: the comparison `up` makes with the
  configuration, address and release it would start the gateway with, which `up` answers by
  refusing; `status` names `down`, then `up`.
- **`status` reports what it can read**: the API, the GPU or the configuration failing is a
  warning on stderr, and the line it would give is left out; the gateway's state and the exit
  status come from the record and `/proc` alone.
- **`logs -f` ends when the gateway it follows stops**, the log read to its end after the last
  check; `up` replaces the log only when no gateway runs, so a log shorter than what was read
  means a new one, and the follow ends there too.
- **`doctor` reads the gateway through `Lifecycle::inspect`, read-only**: no lock, a stale record
  reported, not removed. llama-swap missing, the inventory's address taken and GPU leftovers fail,
  since `up` would fail on each; another llama-swap release, an out-of-date gateway and a stale
  record warn; a stopped gateway is ok (user's approval of the proposed checks).
- **"Idle VRAM" leaves out the processes the Gateway section reports**: a running gateway's models
  are not a display on the NVIDIA GPU, and leftovers have their own failure and remedy.
- **The gateway's integration test runs the test binary itself as the model's llama-server**,
  without libtest's harness, against the installed llama-swap, with a GPU double that lists the
  fake llama-servers alive (user's choice of a Rust fake over a Python one). It is skipped, not
  failed, when llama-swap is not installed or port 5800 is taken.
- **The phase closes with its fifth acceptance criterion open**: OpenCode and OpenWebUI were not
  run against the gateway, only what they send; the check with the real clients is a task of
  phase 4 (user's decision).

---

## Files Changed

Computed against the start commit `docs: close phase 2` of 2026-10-09 with `git diff -M --name-status` and the untracked
files (none). The phase's thirteen commits, `chore: open phase 3 and group the models by use` of 2026-10-09 to `feat(doctor): report the gateway, and document its commands`, and this closure are counted.

**Added**
- `docs/notes/model-library.md`
- `docs/roadmap/on-progress/v2-rebuild/phase-3-gateway-report.md`
- `src/application/gateway.rs`
- `src/application/gateway_config.rs`
- `src/application/lifecycle.rs`
- `src/domain/gateway.rs`
- `src/domain/gateway_config.rs`
- `src/infrastructure/download.rs`
- `src/infrastructure/gateway_api.rs`
- `src/infrastructure/gateway_process.rs`
- `src/infrastructure/gateway_store.rs`
- `src/infrastructure/podman.rs`
- `src/presentation/gateway.rs`
- `src/presentation/lifecycle.rs`
- `tests/fixtures/llama-swap-263-checksums.txt`
- `tests/fixtures/llama-swap-263-health.http`
- `tests/fixtures/llama-swap-263-logs-stream-unknown.http`
- `tests/fixtures/llama-swap-263-logs-stream.http`
- `tests/fixtures/llama-swap-263-running-empty.http`
- `tests/fixtures/llama-swap-263-running.http`
- `tests/fixtures/llama-swap-263-unload.http`
- `tests/fixtures/llama-swap-263-version.txt`
- `tests/gateway_config.rs`
- `tests/gateway_lifecycle.rs`
- `tests/snapshots/gateway-shipped-image.yaml`
- `tests/snapshots/gateway-shipped.yaml`

**Modified**
- `Cargo.toml`
- `README.md`
- `deploy/inventory.toml`
- `deploy/opencode.json`
- `docs/architecture.md`
- `docs/roadmap/on-progress/v2-rebuild/README.md`
- `docs/roadmap/on-progress/v2-rebuild/phase-3-gateway.md`
- `docs/roadmap/on-progress/v2-rebuild/phase-4-bench.md`
- `docs/stack.md`
- `src/application/build.rs`
- `src/application/doctor.rs`
- `src/application/fakes.rs`
- `src/application/mod.rs`
- `src/application/ports.rs`
- `src/domain/inventory.rs`
- `src/domain/machine.rs`
- `src/domain/mod.rs`
- `src/infrastructure/build_store.rs`
- `src/infrastructure/cmake.rs`
- `src/infrastructure/command.rs`
- `src/infrastructure/files.rs`
- `src/infrastructure/git.rs`
- `src/infrastructure/host.rs`
- `src/infrastructure/inventory_file.rs`
- `src/infrastructure/mod.rs`
- `src/infrastructure/nvml.rs`
- `src/infrastructure/paths.rs`
- `src/main.rs`
- `src/presentation/cli.rs`
- `src/presentation/mod.rs`
- `tests/cli.rs`

---

## Problems And Deviations

- **The phase file's Dependencies did not name the models**, which three acceptance criteria need;
  added. Both are downloaded and checked since 2026-10-10.
- **Three template changes are not documented by Unsloth** (merged system messages, no user
  message required, `high` read as `xhigh` in the 27B), where the rule asks for a documented fix.
  They add no text to the prompt; the user accepted them (see Decisions).
- **The shipped inventory changed in this phase** (a phase 1 deliverable): model directories moved
  under `llm/`, with the test that read them.
- **`cargo test` was flaky before this session** (lock test of `files.rs` 9 failures in 30 runs at
  `docs: record the phase 3 session and where the next one starts` of 2026-10-10, a script test of `host.rs` about 1 in 60): child processes forked by parallel tests
  held other tests' files. Fixed with `command::spawn` and `command::exclusive`; 0 in 60
  afterwards. The "three checks pass" of the earlier entries was true of the runs made.
- **The inventory reserves two more flags**, `alias` and `metrics` (a phase 1 rule), which the
  generated command sets.
- **`gateway config` was not in the task list**: added at the user's request with the first
  configuration task.
- **The `up` task grew**: it pulls the missing images first, the consequence of `--pull=never`.
- **No image entry has run end to end yet**: no llama.cpp image is pulled. The podman options were
  checked on a local image; the first real run comes with `up`, which pulls.
- **Two more changes to phase 1 deliverables**: the inventory's ID rule is stricter (any ID
  without whitespace before), and it gains `[gateway] vram_margin_mib`, with `listen` now optional
  within the section; the GPU probe reads the reserved memory.
- **The README's list of flags ai-ctl sets lacked `alias` and `metrics`**, reserved earlier in
  this phase; fixed.
- **The `up` task said "in its own session"**; llama-swap runs in its own process group instead
  (see Decisions), and the task's wording follows.
- **The fork problem reached sockets**: a test releasing a port found it still bound, held by a
  child of a parallel test; such tests hold `command::exclusive()` now, and its comment says so.
- **Not run for real yet**: pulling an image, and a llama-swap exiting before it answers; both are
  covered by the use case's tests, and the pull runs the first time a model is put on an image.
  Likewise for `down`: a container's process on the GPU, a llama-swap outliving SIGTERM, and a
  process left on the GPU.
- **`up` and `status` grew beyond their tasks**: `up` refuses while an earlier gateway's processes
  hold GPU memory, and `status` lists them and says when the running gateway is out of date. A
  gateway killed with SIGKILL made the first necessary: the next `up` would have started a model
  short of memory, moved partly to the CPU without an error.
- **The roadmap's README progress block** is kept current during the phase, as the earlier
  sessions did, besides being recomputed at closure.
- **The integration test also checks `status` and the gateway's log**, beyond the task's `up`, a
  proxied request and `down`: they cost one call each on the gateway the test already runs.
- **The doctor task changed a phase 1 check**: "Idle VRAM" no longer counts the gateway's
  processes and the leftovers, which a running gateway would otherwise have turned into a warning.
- **The last task was mostly done already for two of its documents**: `deploy/opencode.json` and
  `docs/stack.md` pointed at the gateway already — `opencode.json` since the v2 restart
  (`chore: restart llm-ctl v2 on a clean base` of 2026-10-09); only the provider's name, OpenCode's section and the setup steps changed.
- **With 13 tasks of 13 ticked, `progress.py` gives the open phase 20 filled cells and 100%**,
  where the progress invariant caps an unfinished phase at 19; the README carries the script's
  output, as the skill asks, and the phase stays 🟡 until its acceptance criteria are checked.
- **The integration test waits on real processes**, within the bounds `up` and `down` give them
  (30 s for `/health`, 40 s for a stop); the "no test depends on timing" rule of
  `docs/architecture.md` now says so, rather than claiming no test waits at all.
- **The fifth acceptance criterion is not met**: OpenCode and OpenWebUI were not run against the
  gateway. OpenCode is not installed, and OpenWebUI's container is stopped. What both send — tool
  calls with string arguments, streamed answers with `reasoning_content`, `/v1/models` — was
  checked on both models through the gateway. Moved to phase 4 as a task (user's choice).
- **`~/.config/opencode/opencode.json` on this machine is v1's**: an `llm-ctl` provider on port
  9000. `deploy/opencode.json` has to replace it before OpenCode can use the gateway; the phase 4
  task says so.

---

## Changes To Later Phases

- `phase-4-bench.md`: added under Baseline the task "Check OpenCode and OpenWebUI against
  `127.0.0.1:8080`", phase 3's fifth acceptance criterion, moved at the user's choice; the phase
  goes from 12 to 13 tasks.

---

## Assessment

Phase 3 delivered its objective: llama-swap v263, installed and pinned by ai-ctl, is the single
entry point on `127.0.0.1:8080` and the only process that starts or stops a model. `ai-ctl
gateway install | config`, `up`, `down`, `status` and `logs` run it on demand; its configuration
is generated from the inventory, the `matrix` from the VRAM budgets and NVML's usable memory; and
`doctor` reports it. On the real configuration, `up` returns in under 0.2 s, a swap between the
two models takes about 10 s with never both on the GPU, and `down` returns once NVML shows the
memory released. Only the check with OpenCode and OpenWebUI themselves is left, moved to phase 4;
what they send works on both models.

The phase grew beyond its tasks where llama-swap's real behaviour required it. A gateway killed
with SIGKILL leaves its models on the GPU, so `up` refuses to start while an earlier gateway's
processes hold memory, and `status` and `doctor` name them. Reading llama-swap's code at v263
before each step found constraints its documentation does not state — no escape for `$`, a
decoder that ignores unknown keys, the `matrix` under `routing`, a model's output served only as
a stream — and each became a decision or a test. The phase also changed phase 1 deliverables
(stricter IDs, `vram_margin_mib`, two reserved flags, the GPU's reserved memory, "Idle VRAM"),
and fixed a flaky test suite whose cause was child processes inheriting other tests' files.

What phase 4 needs to know first:

- **The official images have never run end to end.** No llama.cpp image is pulled; the podman
  options were checked on another image. The baseline on the official image is the first real
  pull by `up` and the first container served through llama-swap: expect to debug there.
- The bench reaches models through the gateway only: OpenAI endpoints directly, `/tokenize` and
  `/metrics` through `/upstream/<model>/`; llama-server runs with `--metrics` and `--alias <id>`.
- A model's first request pays its load, about 12 s for the 27B; a request to the other model
  pays the swap too. The matrix never keeps both models: each runs alone.
- The figures of this closure (63 and 158 tokens/s, 61% MTP acceptance) come from one short
  request each: they are not a baseline.
- `disableKeepAlives: true` stays on every llama-server entry until a build includes the fix of
  llama.cpp #29680; a bench sending requests back to back is where its absence would show.
- The integration test needs llama-swap installed and port 5800 free; it is skipped otherwise.
- OpenCode is to be installed, with `deploy/opencode.json` replacing v1's configuration, before
  the client task can run; OpenWebUI's container has to be started and pointed at the gateway.
