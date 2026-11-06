# Phase 4 Report: Bench

**Phase:** [phase-4-bench.md](phase-4-bench.md)
**Start Commit:** `docs: close phase 3` of 2026-10-11

---

## Work Log

### 2026-10-11

Phase opened right after the closure of phase 3 (`docs: close phase 3`), in the same session. The phase 3
file and report were read in full, the report having been finalized in this session; it left no
pending restructuring. This phase gained one task at that closure: checking OpenCode and OpenWebUI
against the gateway, phase 3's fifth acceptance criterion, under Baseline (13 tasks now).

State of the machine at opening: llama-swap v263 installed and current; `b11538` the current
llama.cpp build; the inventory installed in `~/.config/ai-ctl/inventory.toml` (`/ai/models`,
`127.0.0.1:8080`, the 27B preloaded); both models downloaded and checked. No gateway running, the
GPU at 28 MiB. No llama.cpp image pulled. OpenCode is not installed, and
`~/.config/opencode/opencode.json` is still v1's; the `openwebui` container is stopped.

The user asked to record the session and commit, and ended it here: no task of the phase is
started, 0 of 13.

Where the next session starts: the Domain tasks — scenarios and results, then v1's aggregates,
read from the bundle at `/code/rust/projects/llm-ctl-v1-archive/llm-ctl-v1.bundle` and ported
deliberately. Points to carry into the work, from phase 3's Assessment:

- The official images have never run end to end: the baseline on the image is the first real
  pull by `up` and the first container served through llama-swap.
- The bench talks to the gateway only; `/tokenize` and `/metrics` go through
  `/upstream/<model>/`, and llama-server runs with `--metrics` and `--alias <id>`.
- The streaming client is to settle with the user before writing it: ai-ctl already speaks
  HTTP/1.0 over `TcpStream` to llama-swap (`src/infrastructure/gateway_api.rs`, with
  `copy_stream`, used by `logs <model>`), and llama-swap, a Go server, answers an HTTP/1.0 client
  without chunked encoding; `serde_json` is already a dependency for the SSE payloads.
- A model's first request pays its load (about 12 s for the 27B), and a request to the other
  model pays the swap: the matrix keeps one model at a time.
- `disableKeepAlives: true` is on every llama-server entry, against llama.cpp #29680.

Resumed in a second session the same day. The user asked to open the phase, already open, and
what the bench would measure; `docs/stack.md` §2, §4, §6 and §9 and the phase 3 report were read
to answer. The answer set the five scenarios against the levers of a single-request machine, and
named what the phase did not cover — quality, load, the one-hour stability run and the `--fit`
logs of §9 — and four points to settle before writing code.

The user settled them, and raised four questions, answered the same way:

- **Depths up to the entry's limit.** Why stop at 80K when the models support more? Their native
  context is 262K, but the server refuses a prompt beyond the entry's `ctx_size`, 96K for the 27B
  and 128K for the 35B; within that, a session near OpenCode's limit is the real case. The depths
  become 8K, 32K, 64K and the entry's limit. Going further is an inventory variant to bench — the
  27B without MTP reaches 160K, already commented out in `deploy/inventory.toml`; 262K on the 27B
  needs about 8.5 GiB of q8_0 KV beside 15.4 GiB of weights *(calculation)*, so it does not fit
  without moving layers to the CPU, or a lighter KV type.
- **vLLM.** The user proposed to keep this roadmap on llama.cpp, and to follow if now were the time
  to test vLLM. It is not: the criterion of `docs/stack.md` §4.2 is the prefill and cache figures
  this phase produces, and Qwen3.8-27B does not fit in NVFP4. Recorded in a working note on vLLM.
- **llama-server's APIs and the gateway's cost.** Chat completions, completions, Responses and
  Anthropic Messages share one engine and one slot: a speed bench across them would measure
  JSON. What may differ is whether they work through llama-swap, so the client check gains
  `/v1/messages` and `/v1/responses`. The gateway's own cost is worth measuring: `/running` gives
  each model's `proxy` address (`http://127.0.0.1:5800` in the phase 3 fixture), so the bench can
  send the same request straight to llama-server without starting anything. It shows whether
  llama-swap delays the first token of a stream, and what `disableKeepAlives` costs. A new
  scenario task.
- **A protocol lighter than HTTP.** llama-server and llama-swap speak only HTTP; llama.cpp's RPC
  protocol distributes a backend's compute across machines and is not a client API. On loopback,
  HTTP costs a few hundred bytes per request and some per chunk, against tens of milliseconds per
  generated token; the gateway-overhead scenario gives the figure.

And one direction for later: a second bench of models larger than the VRAM (70B, 120B), for
quality tasks run in the background. Recorded in a working note on models larger than the VRAM, with the estimate
that sets its first choice: in RAM, a dense 70B generates a few tokens per second, a ~120B MoE
with ~5B active parameters an order of magnitude more.

The phase file changed accordingly, from 13 to 14 tasks: the client task names the HTTP/1.0
client and records inter-token latency; the depth task reaches the entry's limit on a warm cache;
the multi-turn task sends the reasoning back as OpenCode does; the VRAM task also samples
temperature, SM clock and power; the gateway-overhead task is new; the client check covers the
two other APIs. The Objective names the gateway's overhead; the Out of Scope names the stability
run, the APIs' speeds and the two notes; two constraints are added, on repetitions and on comparing
runtimes. The README's progress block was regenerated.

Committed as `docs: settle the bench's scenarios`, then on to the Domain tasks. v1's bench, read from the bundle (`main`;
`archive/v1-wip` holds the same files), kept everything in `src/domain/benchmark.rs`: a prompt
suite, per-request metrics measured by the client alone (TTFT, latency, tokens counted per SSE
delta), and `PromptResult::from_metrics` with a mean, a median and a p99 interpolated linearly, and
a population standard deviation — all zeros for an empty series. Before modelling the server's
side, llama.cpp's source at `b11538` (`~/.local/share/ai-ctl/sources/llama.cpp`) was read:
`server_slot_stats::to_json` in `tools/server/server-common.cpp` writes `cache_n`, `prompt_n`,
`prompt_ms`, `predicted_n`, `predicted_ms` and their per-second rates, plus `draft_n` and
`draft_n_accepted` whenever speculative decoding proposed a token; a streamed chat completion puts
`timings` on its last chunk, and `usage` on an extra chunk only with `stream_options.include_usage`
(`tools/server/server-task.cpp`). So each request reports its own MTP acceptance, beside the
`/metrics` counters the MTP task reads.

Two modules came of it. `src/domain/stats.rs` ports v1's aggregates into a `Summary` (count, mean,
median, min, max, standard deviation, p99, relative spread). `src/domain/bench.rs` holds the
scenarios (depth, prefill, session, overhead), the `Plan` a model's context size gives — the depths
up to its limit, the prefill prompt, the session's shape, the repetitions — and the results: a
`Sample` per request joining the client's `ClientTimings` (TTFT, total, the gaps between chunks)
with llama-server's `ServerTimings` and the `usage` object, and one result type per scenario with
its summaries. With the 27B's 98304 tokens, the depths are 8192, 32768, 65536 and 97792. 25 tests,
the three checks green.

The user settled the two points left: the plan takes the server's real `n_ctx` from
`/upstream/<model>/props` rather than the inventory's `ctx_size`, and the MTP task reads each
request's draft counts, the `/metrics` counters as a check. Committed as `feat(bench): model the scenarios, results and aggregates`.

The client next. `ai-ctl up` started the gateway, the 27B on port 5801, and curl captured over
HTTP/1.0 a streamed reply through llama-swap (24 reasoning tokens; `draft_n` 18 and
`draft_n_accepted` 13 on the last event), the same request straight to llama-server, a 404 for an
unknown model, and a 400 from llama-server through llama-swap. The direct reply showed what the
phase 3 client could not read: llama-server answers an HTTP/1.0 request in HTTP/1.1, with
`Transfer-Encoding: chunked` and `Keep-Alive`. It closed the connection right after its last chunk
all the same, with or without `Connection: close`: both were timed, `[DONE]` and the close within
a millisecond. So the HTTP/1.0 client moved out of `gateway_api.rs` into
`src/infrastructure/http.rs`, which now sends `Connection: close` and a JSON body, and reads a body
as it arrives — to its `Content-Length`, through a chunk decoder, or to the close; `parse` still
refuses chunks in llama-swap's whole responses. `src/infrastructure/chat_api.rs` is the client,
`OpenAiChat` behind the `ChatClient` port: v1's SSE parser, rewritten to split events as the
format does (comments such as llama-server's pings skipped, `data` lines joined), the clock
started before the connection opens, TTFT at the first content or reasoning delta, the end at
`[DONE]`, an error body or an error event turned into its message. The four captures are fixtures.

Last, the premise of the latency measure was checked live with a short script, 128 tokens with
`ignore_eos`, twice each way: llama-swap does not buffer the stream — first tokens and arrivals
alike through it and direct — but tokens arrive in bursts, 83 of 127 gaps under a millisecond on
both paths, the drafts MTP accepts being sent together. A median gap between tokens is then 0 and
says nothing, so the domain's `inter_token_latency` became `arrival_gaps`, which leaves gaps under
1 ms out. The three checks green, the gateway integration test included.

Committed as `feat(bench): stream and time chat completions`, the gateway stopped. Then the prompts. Asked what the prompts would hold,
the user proposed a realistic filler, a fictional request on fictional code, for more realistic
figures. Realism changes neither the prefill, nor the cache, nor generation without MTP, but it
sets what the model generates, hence MTP's acceptance: an agent reuses the names and lines it has
read. Real code frozen at one commit was preferred to generated code, which would be repetitive and
inflate the acceptance: ai-ctl's own sources at `feat(bench): stream and time chat completions`, built into `assets/bench/corpus.txt`
with `git show` file by file, each in a `<file path="…">` block, 636 KB.

The gateway was started again to capture `/props` (`n_ctx` under `default_generation_settings`),
`/apply-template` and `/tokenize`, with and without pieces, all through `/upstream/qwen3.8-27b/`.
The props showed the 27B's template keeping past reasoning (`preserve_thinking` undefined or true),
so a session's prefix stays stable from turn to turn. A Python script then checked the approach
before any code: the corpus makes 175419 tokens, tokenized with its pieces in 0.65 s; a text cut
at a token boundary re-tokenizes to the same count from 1K to 97K tokens; the template adds 59 or
60 tokens around it.

`src/domain/prompt.rs` holds the corpus, the agent's texts (a system prompt describing ai-ctl's
layers, rules and tools; a request to add `--quiet` to `ai-ctl up`; the instruction to write the
change) and the `Tag` that opens a fresh prompt; `src/application/bench.rs` the `Prompts` that
load the corpus through the model's tokenizer and fit an agent's prompt to a number of tokens;
`src/infrastructure/server_api.rs` the `ModelServer` port's adapter for `/props`,
`/apply-template` and `/tokenize`, through llama-swap. `http::request` takes a JSON body, and the
message of an error body is read in one place.

A temporary test then ran the real adapters on the 27B: prompts fitted to 8192 and 97792 tokens
were counted 8192 and 97792 by llama-server (`prompt_n`, with `cache_n` 0), and 97792 with 512
tokens generated fit in its 98304. One try each, not a baseline: prefill at 1358 and 990 tokens/s
(TTFT 100 s at 97792, within the client's 300 s), generation at 69 and 42 tokens/s, MTP accepting
89 % and 84 % of drafts. But at 97792 the model answered with calls to its `read` tool for
`src/main.rs`: the corpus, sorted by path, held none of the files the request needs before the
infrastructure ones, so what the model generated changed with the depth. The corpus was rebuilt
with those files first — `src/presentation/cli.rs`, `src/main.rs`, `src/presentation/mod.rs`,
`src/presentation/lifecycle.rs`, `src/application/lifecycle.rs`, about 7.7K tokens before the
use case — then the rest by path. Run again at 8192 and 32768 with 1024 tokens free to end, the
model analysed the request at both depths, its 1024 tokens all reasoning at the server's `medium`
effort. At 32768 the fit stopped at 32767: where the code was cut, adding one token's text added
two tokens, and the closest prompt under the target is kept. The temporary test was removed.

Committed as `feat(bench): fit an agent's prompts to exact token counts`. The user ended the session here, to resume in a fresh one: 4 of 14 tasks
done, the four commits of the session being `docs: settle the bench's scenarios` (the scenarios settled), `feat(bench): model the scenarios, results and aggregates` (domain),
`feat(bench): stream and time chat completions` (client) and `feat(bench): fit an agent's prompts to exact token counts` (prompts). State of the machine at the end: the gateway stopped
by `ai-ctl down`, the GPU at 14 MiB; nothing else changed on it.

Where the next session starts: the Scenarios tasks, assembled into `ai-ctl bench run` — the
depth, prefill and session scenarios first, then MTP, VRAM and the gateway's overhead, then the
JSON results and `bench compare`. What is ready for it, and what is still open:

- **Ready**: `Plan::new(n_ctx, repeat)` and the result types with their summaries
  (`src/domain/bench.rs`); `Prompts::load`, `agent` and `turn` (`src/application/bench.rs`); the
  `ChatClient` and `ModelServer` ports with `OpenAiChat` and `LlamaServerApi`, `context_size`
  giving the plan's `n_ctx`. `FakeModelServer` exists in `src/application/fakes.rs`; a fake
  `ChatClient` is still to write, with the use case.
- **The session's turns**: turn 1 is `Prompts::agent` at the plan's opening; each later turn
  appends the reply as `Message::assistant(content, reasoning)` and then `Prompts::turn(from,
  1024)`, `from` where the code stopped. At the server's `medium` effort, a 512-token reply is all
  reasoning, its content empty: whether session replies use `ignore_eos`, or end on their own, is
  to settle.
- **The overhead scenario** needs each model's `proxy` address: `parse_running`
  (`src/infrastructure/gateway_api.rs`) reads only the ID and state of `/running` today.
- **Sampling NVML during a run** needs a second thread. `docs/architecture.md` brings in `tokio`
  for concurrent I/O; a `std::thread` sampling at a fixed period may be enough and adds no crate:
  to settle before the VRAM task.
- **A fresh prompt's `cache_n`** is 0 in the live checks; a check that a prefill sample really
  missed the cache can allow the few template tokens a tag leaves shared.
- **Duration** *(estimate)*: with the default plan on the 27B, about 15 to 20 minutes per run, the
  97792-token fill alone taking 100 s.

Resumed in a third session the same day, from `docs: record where phase 4 stops`. The phase file, this report and the bench
code were read again. The user settled the two points left open: session replies are generated
with `ignore_eos`, 512 tokens each, and NVML is sampled on a `std::thread`.

The scenarios were written together, as `Bench::run` in `src/application/bench.rs`, since they
share the run's machinery: the running gateway found through its record, the model's `n_ctx` read
first (the gateway loads the model on that request), the plan, the corpus tokenized, then a warm-up
request — the agent's prompt without code, 16 tokens — left out of the measures. Each request asks
`ignore_eos`, its exact length and seed 42; a failed request ends the run, its error naming the
step (`repetition 3 at 32768 tokens: …`). The overhead scenario was written with the others rather
than after MTP and VRAM, so that `--scenario` never offers an unfinished one: `RunningModel` gained
the `proxy` address of `/running`, read before the run so that a model without one fails at once;
its prompt is the warm-up's, in the cache, 16 tokens generated, sent 50 times each way in turn,
after one unmeasured request that primes the cache. A `FakeChat` simulates a llama-server with one
slot — the prompt's common prefix with the last prompt and reply comes from the cache — so the use
case's tests check that the depths repeat the fill's prompt, each prefill is fresh, a session sends
its history back and its cache grows, and the overhead alternates the two addresses.
`ai-ctl bench run <model> [--repeat N] [--scenario S]...` prints each request on stderr and the
run's tables on stdout (`src/presentation/bench.rs`).

The first live run, `--repeat 1` on the 27B, went through end to end in about seven minutes
(figures of one try each, not a baseline): generation at 80.9, 51.7, 50.9 and 43.1 tokens/s at
8192, 32768, 65536 and 97792 tokens, with a TTFT of 141 to 450 ms on the filled context; the fills
at 1603 down to 1006 tokens/s, the 97792-token one taking 98.7 s; the fresh 40960-token prefill at
1380 tokens/s, one token generated, which streams its reasoning token as the first delta, as
hoped; the session's `cache_n` growing at every turn, 0, 8703, 10266 … 21191, about 1050 tokens
evaluated per turn and a TTFT of about 1 s from turn 2 on, so the 27B's template keeps the
reasoning sent back and the hybrid model finds its checkpoint; the gateway adding 0.6 ms to the
TTFT and nothing measurable to the total (117.6 against 117.0 ms, 294.1 against 294.4 ms). A TTFT of
999.6 ms printed as `1000 ms`: durations now switch to seconds from what rounds to a second.

MTP next. llama.cpp's source at `b11538` showed that `/metrics` adds up, request by request, the
same slot counters `timings` reports (`metrics_on_prediction` in `tools/server/server-context.cpp`):
`spec_decode_num_draft_tokens_total`, `…_accepted_tokens_total`, `…_drafts_total` (verification
steps) and `…_accepted_tokens_per_pos_total{position="i"}`, written as doubles with six
significant digits. So the run reads them after the warm-up and at its end, and adds up the drafts
of every request in between, the overhead's primer included: the two agree to the counters'
rounding unless another client used the model. A server without `--metrics`, or `/metrics` that
fails, gives a warning and the run goes on. The `/metrics` captured after the first run counted
10849 predicted tokens, exactly the run's, and 6883 of 7669 drafts accepted over 3838 steps, two
positions, 2.79 tokens per step.

Then the GPU: `NvmlMonitor` reads memory, temperature, SM clock, power and NVML's throttle reasons
(power cap, thermal, hardware) every 250 ms on a thread of its own, started after a first reading
that shows NVML answers, each reading labelled with the scenario under way; a GPU that cannot be
read gives a warning and the run goes on. The run's tables gain a drafts column per depth, the
session's acceptance, the drafts section with acceptance per position, and a GPU table per scenario
and over the run. Committed as `feat(bench): run the scenarios, check the drafts and read the GPU`, the four scenario tasks ticked; MTP and VRAM waited for
a live run.

The second live run, `--repeat 1`, showed both: the requests accounted for every draft the server
counted, 89.8 % of 7652 accepted, 2.80 tokens per decoding step, 93.4 % and 86.1 % at the two draft
positions; the GPU read 1631 times, peak memory 21352 MiB, 74 °C at most, the SM clock at a median
of 1987 MHz and 1507 at least, 149 W on average, slowed by the power limit in 93 % of the readings
and never by heat. It also showed two problems.

At the context's limit, the repetition generated 511 tokens, not 512. Without context shift,
llama-server ends a generation once the context holds all but one token
(`prompt.n_tokens() + 1 >= n_ctx` in `tools/server/server-context.cpp`), and whether the last
step's drafts are already in the cache then decides between 511 and 512. Every request now leaves
one token free beyond those it generates: the 27B's limit depth is 97791, the session needs 22529
tokens of context.

And the generation at the limit fell from 43.1 tokens/s in the first run to 37.4, its drafts'
acceptance to 65.2 % against 84 % at the other depths. The two runs had not sent the same prompts:
the tag that kept a fresh prompt out of the cache held the run's start, so each run's prompts, and
the replies the model generated to them, were its own — and what the model generates sets MTP's
acceptance, hence the speed. Between runs, the content would have varied with the runtime under
test. llama-server's `cache_prompt: false` evaluates a prompt whole, whatever its cache holds
(`n_past = 0`); checked live through llama-swap: two requests with it evaluated their 743 tokens
twice, `cache_n` 0, and a third without it took 739 from the cache, the three replies the same.
So the tag went: the prompts are the same in every run, and the requests that must miss the cache
— each depth's fill, the prefills, a session's first turn — ask `cache_prompt: false`. Within a
run, the five prefills and the five sessions are now the same requests, so their spread is the
machine's alone, as for the depths' repetitions.

Then the results. A run now records its setup before its first request: the model's inventory
entry, the command line of its llama-swap entry, one line per option, its source — the current
build from the build store, or the image with the digest `podman image inspect` gives — and the
machine, the GPU from NVML and the power profile and AC adapter `doctor` reads. A run refuses to
start unless the gateway runs the configuration the inventory and the builds give now, the YAML
`up` wrote compared with a fresh generation, as `status` does: otherwise the run would record
another model, source or flags than the gateway serves. `FsRunStore`
(`src/infrastructure/run_store.rs`) saves a run as JSON in `~/.local/share/ai-ctl/bench/`, named
after its start and model, every sample kept, durations in microseconds; written beside its name,
then hard-linked under it, a saved run is complete and never replaced, and `build.json` is reused
for a native source. `bench run` saves after printing its tables, so a failed save loses nothing
on screen. `bench list` lists the saved runs, and `bench compare <run> <run>...` sets them side by
side: the command lines and machines that differ, then one table per measure — generation, TTFT
and the fill's prefill by depth, the prefill, the session's TTFT by turn, the gateway's overhead,
token arrivals, the drafts and the GPU — with a change column for two runs, in percent, or in
points for shares. `bench` is documented in the README, and `docs/stack.md` §9 says what the
bench measures of the protocol, with the 64K depth and the context's limit in place of 80K.

The third live run, `--repeat 1`, the first with the same prompts in every run, was saved as
`20261011T155903Z-qwen3.8-27b` (580 KB), its setup recorded: build `b11538`, the 27 lines of the
model's command, its flags (`spec-draft-n-max` 2, a `q8_0` KV cache, `medium` reasoning), the
`performance` profile on AC power. At the limit depth, 97791, the repetition generated its 512
tokens. A fourth run of the depths alone, `20261011T160553Z-qwen3.8-27b`, then checked
`bench compare` and the change of prompts: generation within 0.5 % at every depth (76.6 and 76.2,
62.0, 52.9, 44.5 tokens/s), and the drafts accepted at the depths 85.4 % in both runs — the same
replies. That comparison also showed that the tables over a whole run mixed scenarios: the
gaps between token arrivals pooled the depths with the sessions the first run had and the second
had not, +14.7 % on the median gap, and the server's counters and the GPU covered different
scenarios. The gaps are now compared scenario by scenario, the server's counters only between runs
of the same scenarios, and the GPU over the scenarios every run ran: the depths' median gap then
differs by 0.2 %, and the GPU over the depths not at all.

Committed as `feat(bench): run the scenarios, check the drafts and read the GPU` and `feat(bench): save runs with their setup and compare them`: 12 of 14 tasks done. State of the machine at the end: the
gateway stopped by `ai-ctl down`, the GPU at 14 MiB; two runs saved in
`~/.local/share/ai-ctl/bench/`, which are trials of one repetition, not the baseline. Nothing else
changed on the machine.

Where the next session starts: the two Baseline tasks, which change the user's environment and
wait for their agreement. The baseline: a run of five repetitions on the native build, about 20
minutes; then the first pull of the official image, an edit of the installed inventory to run the
27B on it, `ai-ctl down` and `ai-ctl up`, the same run, `bench compare`, and the inventory set back.
The client check: installing OpenCode, `deploy/opencode.json` in place of v1's
`~/.config/opencode/opencode.json`, the `openwebui` container started, and `/v1/messages` and
`/v1/responses` streamed through the gateway.

The user agreed to the baseline. The image had to be chosen first: the installed inventory's
`llama-cuda13` backend names the floating tag `server-cuda13`, whose build is whatever was published
last, and the comparison is between runtimes, not llama.cpp versions. ghcr.io's 12403 tags hold no
image of `b11538`, the native build: an image is not published for every build. The nearest after
it is `server-cuda13-b11541`; the three commits between them, read in the clone
(`git log b11538..b11541`), touch OpenCL kernels, SYCL kernels and the vendored JSON parser,
nothing CUDA builds or llama-server's inference runs. So the baseline compares the native
`b11538` with `server-cuda13-b11541`, 2.17 GiB compressed in 15 layers.

The image was pulled before the native run, so that no download would load the machine during a
measure. The first pull failed after several minutes: name resolution vanished during the download
(`lookup ghcr.io: no such host`), and podman kept nothing. The network back, a second pull ran at
about 5 MiB/s; the user then chose to stop and pull the image from a better connection, at the
start of a fresh session. The second pull was stopped, and its 219 MB of partial blobs, left in
`/var/tmp/container_images_storage…`, removed: podman does not resume from them.

State of the machine at the end of the session: the gateway stopped, the GPU at 14 MiB; the
image not in local storage; the installed inventory untouched, the 27B on `llama-native`; the two
trial runs of this session in `~/.local/share/ai-ctl/bench/`.

Where the next session starts: `podman pull ghcr.io/ggml-org/llama.cpp:server-cuda13-b11541`, on a
good connection. Then, on a quiet machine, the native baseline: `ai-ctl up`, then
`ai-ctl bench run qwen3.8-27b`, five repetitions, about 20 minutes. Then the installed inventory
copied aside, and edited for the image: `llama-cuda13`'s `source` set to the pinned tag, and the
27B's `backend` to `llama-cuda13`; `ai-ctl down`, `ai-ctl up` — the first container served through
llama-swap — and the same run. Then `ai-ctl bench compare` of the two runs, the figures recorded
in this report, and the inventory set back from its copy, `down` and `up` again. The client check
follows.

### 2026-10-12

Resumed in a fourth session, from `docs: record the baseline's image and where phase 4 resumes` of 2026-10-11, the user having pulled
`ghcr.io/ggml-org/llama.cpp:server-cuda13-b11541` from a better connection: digest
`sha256:42e80867b03cc22e8cf744c07e0b3194f078838ab6c476095ecdc1621271ec95`, 4.1 GB unpacked, built
2026-10-10. The machine was quiet, the GPU at 15 MiB and 43 °C, the gateway stopped, the release
binary current with `feat(bench): save runs with their setup and compare them` of 2026-10-11. `ai-ctl up` started the gateway on the native build, then the
native baseline started: every scenario, five repetitions.

While it ran, the container's path to the GPU was read, without touching the GPU: `podman run`
asks the CDI device `nvidia.com/gpu=all`, defined twice on this machine. `/etc/cdi/nvidia.yaml`,
generated in January, names driver 580.119.02's libraries; `/var/run/cdi/nvidia.yaml`, regenerated
at boot this morning by the `nvidia-cdi-refresh` unit of the container toolkit, names 615.71.09's,
the driver installed. CDI resolves a device defined in two spec directories to the later one,
`/var/run/cdi`, so the container should get the current libraries; the image run will tell. Doctor
checks only that `nvidia.com/gpu=all` is listed, and its remedy writes `/etc/cdi/nvidia.yaml`,
the static spec that went stale here.

The native run, `20261012T105446Z-qwen3.8-27b`, took 16 minutes, every repetition close to the
others: generation at 75.4, 61.6, 52.8 and 44.6 tokens/s at 8192, 32768, 65536 and 97791 tokens,
± 0.1 at each; the fresh prefill at 1386.1 ± 3.8 tokens/s; the cache growing at every turn of the
five sessions; 93.0 % of 27474 drafts accepted, all accounted for by the requests; the gateway
adding 1.5 ms to the TTFT. The GPU peaked at 21352 MiB and 76 °C, slowed by its power limit in
96 % of the readings, never by heat.

Two first repetitions stood out: the first prefill's TTFT, 31.28 s against 29.6 s for the four
others, and the first turn of the first two sessions, 6.07 s and 7.81 s against 5.22 s for the
three others, the server's prefill speed being the same for all. The server's log placed the
difference between the slot's selection and the task's launch: 0.92 s before the first session,
2.57 s before the second, under a millisecond before the others. llama.cpp's source at `b11538`
gave the cause: with `cache-ram`, a slot that takes a new prompt first saves the state of the one
it holds — the target's and the draft's — into the RAM prompt cache, unless that prompt is already
there (`get_available_slot` in `tools/server/server-context.cpp`, `server_prompt_cache::alloc` in
`tools/server/server-task.cpp`). The first repetition after another scenario saves that scenario's
context; the next ones, the same requests, find theirs saved. Prefill speeds come from the
server's `prompt_ms` and are untouched; a first repetition's TTFT counts the save, which widens
the spread of the session's first turn to ± 1.13 s. Both runtimes run the scenarios in the same
order, so the comparison holds.

For the image, `ai-ctl down`, the installed inventory copied to
`~/.config/ai-ctl/inventory.native.toml`, then two lines changed: `llama-cuda13`'s source to
`ghcr.io/ggml-org/llama.cpp:server-cuda13-b11541`, the 27B's backend to `llama-cuda13`. `ai-ctl up`
found the image in local storage and served the 27B in its container, the first served through
llama-swap: the GPU reached through CDI, the model and its MTP head loaded in 3.8 s, the page cache
warm, against 6.0 s for the native start before it. The server is build 11541 (`f2918cabb`), CUDA
runtime 13.4.1, GCC 14.2; the image sets `LLAMA_ARG_HOST` and `LLAMA_ARG_PORT`, which the command
line overrides with a warning, and no other `LLAMA_ARG_*`. Then the same run.

The image run, `20261012T111211Z-qwen3.8-27b`, took 16 minutes as well, its source recorded with
the image's digest. `bench compare` of the two runs, the native build first:

| Measure | Native `b11538` | Image `server-cuda13-b11541` | Change |
|---|---|---|---|
| Generation at 8192 tokens | 75.4 ± 0.1 t/s | 76.1 ± 0.1 t/s | +0.9 % |
| Generation at 32768 tokens | 61.6 ± 0.1 t/s | 61.7 ± 0.2 t/s | +0.3 % |
| Generation at 65536 tokens | 52.8 ± 0.1 t/s | 52.8 ± 0.2 t/s | +0.1 % |
| Generation at 97791 tokens (limit) | 44.6 ± 0.1 t/s | 44.6 ± 0.1 t/s | +0.0 % |
| TTFT on the filled context, 65536 | 350 ± 19 ms | 379 ± 16 ms | +8.3 % |
| TTFT on the filled context, 97791 | 449 ± 40 ms | 480 ± 5 ms | +6.8 % |
| Fill's prefill, 8192 → 97791 | 1625.9 → 1007.6 t/s | 1592.1 → 1003.2 t/s | −2.1 → −0.4 % |
| Fresh prefill, 40960 tokens | 1386.1 ± 3.8 t/s | 1381.4 ± 4.1 t/s | −0.3 % |
| Session TTFT, turns 2 to 10 | 945 ms → 1.10 s | 1.00 → 1.18 s | +5.4 to +7.4 % |
| Gateway overhead on the TTFT | +1.5 ms | +0.2 ms | — |
| Median gap between arrivals, depths | 43.7 ms | 44.0 ms | +0.5 % |
| Drafts accepted, run | 93.0 % | 93.0 % | +0.0 pt |
| GPU: peak memory, max temperature | 21352 MiB, 76 °C | 21352 MiB, 75 °C | — |
| GPU: median SM clock, mean power | 1747 MHz, 145 W | 1762 MHz, 145 W | +0.9 %, −0.2 % |

The two runtimes generated the same tokens — every draft count is identical — and generate, fill
and prefill at the same speed, within 1 % but for the 8192-token fill, a single measure. The image
is slower in one place: the TTFT of a short prompt evaluated on a long context. The saved samples
split each TTFT into the server's `prompt_ms` and the rest, and the difference is all in
`prompt_ms`: the 4 tokens a depth's repetition evaluates take 278 against 309 ms at 65536 tokens
and 340 against 390 ms at 97791; a session's 1050 tokens 926 against 978 ms at turn 2 and 1074
against 1157 ms at turn 10; the overhead's 4 tokens 110 against 122 ms, on both paths. At 8192
and 32768 tokens the two are equal. The cause was looked for, not found: both binaries hold
`sm_120a` code (`cuobjdump`), both CUDA 13.4 (13.4.92 native, 13.4.1 in the image), both with a
memlock limit of 8192 KiB. What differs still is the host compiler (GCC 16.2 against 14.2), the
linking (static against shared libraries), and the CPU backend (`GGML_NATIVE` against the image's
variants loaded at run time).

For an agent, the image costs 50 to 80 ms on a turn's first token, about 6 %, and nothing on the
rest: the native build stays the default runtime on this measure, and the image is a fair
fallback. The baseline answers the phase's question; it is not a reason to change the inventory.

The installed inventory was set back from its copy, byte for byte, the copy removed; `ai-ctl down`
and `ai-ctl up` served the 27B on the native `b11538` again, and the gateway was left running.
The baseline task ticked, 13 of 14.

What is left: the client check, which installs OpenCode, replaces v1's
`~/.config/opencode/opencode.json` with `deploy/opencode.json`, starts the `openwebui` container
and streams `/v1/messages` and `/v1/responses` through the gateway, and waits for the user's
agreement; and the two points left open for the user, the first repetitions' cache save and
doctor's CDI remedy.

The user left the first point to the approach judged best, and asked to fix the second at once.

For the first, llama.cpp's source settled when the save happens: `get_available_slot` sets a slot's
context aside when the new prompt keeps less than half of it (`f_keep < 0.5`), or when no slot
matches; `server_prompt_cache::alloc` skips a context already saved. Hence the three cases of the
native run: the first prefill set the 97791-token context aside, the first session the prefill's,
the second session the first one's end, new to the cache; each later session ended where the one
before had, already saved. So a fresh request now comes after an unmeasured request of the
warm-up's short prompt, `Step::SetAside`, which `Runner::send` sends before every fresh step: that
request keeps little of the slot's context and pays its save, and the fresh request finds the short
prompt in the slot, whatever ran before. The overhead's own first request, `Step::Prime`, was the
same request: it became a `SetAside` too. A set-aside request that fails names the step it came
before (`before the fill at 8192 tokens: the request that sets the context aside: …`). Its drafts
count with the others' in the check against `/metrics`. The use case's tests follow the new
request order, and a new one checks that every fresh request, and the overhead, comes after one.

For the second, the toolkit installed, 1.20.1, ships `nvidia-cdi-refresh.path` and
`nvidia-cdi-refresh.service`, both enabled here: they write `/var/run/cdi/nvidia.yaml` at boot and
at each driver change, so `/etc/cdi/nvidia.yaml` was a leftover of a manual `cdi generate`.
`HostProbe::cdi_specs` reads the NVIDIA specifications of `/etc/cdi` then `/var/run/cdi`, with the
driver of the `libcuda.so.<version>` each mounts (`parse_cdi_driver`, on an excerpt of this
machine's specification captured as `tests/fixtures/nvidia-ctk-1.20.1-cdi-spec-excerpt.yaml`).
Doctor's "CDI specification" check now warns when the specification in use, the last, mounts
another driver's libraries, and when one it shadows names another driver, with its removal as the
remedy; its remedy for a missing or incomplete specification enables the toolkit's two units
instead of writing `/etc/cdi/nvidia.yaml`. On this machine, doctor now warns about
`/etc/cdi/nvidia.yaml` and gives `sudo rm /etc/cdi/nvidia.yaml`; the file is left for the user to
remove. `docs/stack.md` (§2's failure modes, the container tooling's setup, §9's protocol) and the
README follow. The three checks green, 492 tests.

A live check of the set-aside request, `--repeat 2` of the prefill and the sessions
(`20261012T114848Z-qwen3.8-27b`), showed the big save gone but not all the cache's work: each fresh
request's TTFT still held 185 to 211 ms beyond the server's `prompt_ms`, and the second prefill
385 ms, against 11 to 42 ms for the clean repetitions of the native baseline. The source gave the
reason: with the short prompt in the slot, a 40960-token prompt shares less than 10 % of itself
with it, under `--slot-prompt-similarity`, so llama-server took the slot as the least recently
used, saved the short prompt, and looked in the RAM cache for a better start, loading the previous
prefill's context — which `cache_prompt: false` then evaluated again whole. So the request before
a fresh one now sends the same prompt cut to a quarter (`LEAD_FRACTION`): the fresh prompt shares
a quarter with it, more than the 10 %, and keeps all of it, more than the half, so the server
selects the slot by similarity and neither saves nor loads anything. When a quarter cannot hold
the agent's own prompt, the request sends that prompt without code, which shares all but the
instruction. The overhead keeps the warm-up's prompt, which is its own. A second live check,
`--repeat 3` (`20261012T115603Z-qwen3.8-27b`), measured every fresh request as the clean ones: 38
to 44 ms beyond `prompt_ms` on the three prefills, 10 to 11 ms on the three sessions' first turn,
their TTFTs 29.29, 29.45, 29.54 s and 5.12, 5.10, 5.11 s; the requests accounted for every draft.
The use case's test of the rule checks the two thresholds on each fresh request.

What is left: the client check, waiting for the user's agreement; and `/etc/cdi/nvidia.yaml`, for
the user to remove. The gateway runs the native `b11538`; the two live checks are saved beside the
baseline, trials of the fix, not measures to compare.

The user asked to commit and record the session, and to resume with the client check in a fresh
one. Committed in three commits: `fix(doctor): check the CDI specification against the loaded driver`, doctor's CDI check; `fix(bench): set the slot's context aside before each fresh request`, the bench's set-aside
request; then this record with the baseline. 13 of 14 tasks done.

State of the machine at the end of the session: the gateway running on `127.0.0.1:8080`, the 27B
on the native `b11538`, holding 21.3 GB of VRAM until its 30-minute TTL; the installed inventory
as at the session's start, its copy removed; the image `server-cuda13-b11541` in podman's storage,
no container left; four runs saved in `~/.local/share/ai-ctl/bench/` this session — the baseline's
`20261012T105446Z-qwen3.8-27b` (native) and `20261012T111211Z-qwen3.8-27b` (image), and the two
trials of the fix; `/etc/cdi/nvidia.yaml` still there, stale, for the user to remove.

Where the next session starts: the client check, the phase's last task. OpenCode installed, v1's
`~/.config/opencode/opencode.json` saved aside and replaced with `deploy/opencode.json`, OpenCode
run against `127.0.0.1:8080` with tool calls through the 27B's template; the `openwebui` container
started and pointed at the gateway; `/v1/messages` and `/v1/responses` streamed through the
gateway. Then the phase's closure: the acceptance criteria, the three checks, and the report's
Files Changed and Assessment.

Resumed in a fifth session, from `docs: record phase 4's baseline, native build against official image`, for the client check. The gateway was still running,
the 27B ready on port 5801; the user had removed `/etc/cdi/nvidia.yaml`. OpenCode was not
installed: the `PATH` line of `~/.bashrc` for `~/.opencode/bin` shows it came from the official
install script, and `~/.config/opencode/` still held v1's configuration beside a plugin package at
1.14.29. The `openwebui` container, stopped since April, runs on
`slirp4netns:allow_host_loopback=true` with port 18000; its saved settings point at an Ollama
connection on `http://10.0.2.2:9000`, v1's gateway, and at api.openai.com. So `docs/stack.md`
§7.3's `http://127.0.0.1:8080/v1` is wrong for it: inside the container, that address is the
container, and the host's loopback is `10.0.2.2`.

The two other APIs first, streamed through llama-swap and, for comparison, straight to the
model's `proxy` address, with a short script timing each event. Both stream through the gateway
unbuffered: 221 events of `/v1/messages` and 223 of `/v1/responses` spread over 2.5 s for 400
tokens, as direct. Both make tool calls: offered a `read` tool and asked for `src/main.rs`, the
model answered with a `tool_use` block, its input `{"path":"src/main.rs"}` streamed as
`input_json_delta` and the stop reason `tool_use`, and with a `function_call` whose arguments came
in seven deltas. In the Messages stream, llama-server opens the text block before it closes the
thinking block, whose `content_block_stop` and empty `signature_delta` come at the end: harmless
for the clients checked, but not Anthropic's order.

Then OpenCode. The install script (`https://opencode.ai/install`, read before use) downloads the
latest release from `github.com/anomalyco/opencode` and checks no digest; so the release archive,
`opencode-linux-x64.tar.gz` of v1.19.0 (published 2026-10-11), was downloaded apart, its SHA-256
checked against the digest GitHub publishes for the asset (`7bb9e61a…`), and installed with the
script's `--binary` and `--no-modify-path` into `~/.opencode/bin`, already on the `PATH`. v1's
configuration was moved to `~/.config/opencode/opencode.json.v1-backup`, and `deploy/opencode.json`
copied in its place; `opencode models llamacpp` listed the inventory's two models.

OpenCode was run with `opencode run --format json` in a scratch git repository holding a few of
ai-ctl's files, three times. Asked which file defines `Summary` and its fields, it called `grep`,
then `read`, and answered with the seven fields as they are. Asked to add a `range` method, it
called `read`, `edit` — multi-line `oldString` and `newString` holding Rust code — `bash` twice
(`cargo check`, `git status`), and the diff was the method asked for and nothing else. Continuing
that session with `--continue`, it ran a `grep` through `bash` and showed its output. No error on
either side; every request reached the 27B, the session's title included, generated by the
`small_model` on the gateway. The server's log shows the reasoning sent back as the configuration
asks: the turn after a tool call kept the whole context (`f_keep = 1.000`) and evaluated only the
tool's result, and the continued session evaluated 41 tokens, the rest from the cache. So
llama-server hands OpenCode's tool calls to the 27B's template without the error a JSON string
would raise, which phase 3 had checked only with a script.

Then OpenWebUI. The user chose to update its image, five months old, and to set its connection
in the interface themselves. Its data, `webui.db` with its WAL, `vector_db` and `uploads`, was
saved first to `~/.local/share/open-webui-backup-2026-10-12.tar.gz`, since a newer OpenWebUI
migrates its database for good. The pull of `ghcr.io/open-webui/open-webui:main` was started, then
stopped at about 450 MB at the user's request, to run it from their own terminal; its partial
blobs in `/var/tmp` were removed. The image in use is still `sha256:a1916b8d…` of 2026-04-24.

The user pulled the image and recreated the container with the same options and volume: OpenWebUI
v0.12.0, `sha256:2e3549a5…`, built 2026-10-10 (`f8ae8a6`). It started cleanly, after migrating the
database through some thirty Alembic revisions, its configuration reshaped into one row per key.
From inside the container, `http://127.0.0.1:8080/v1/models` answers OpenWebUI's own page, and
`http://10.0.2.2:8080/v1/models` the gateway's two models: the address §7.3 has to give.

The user could not sign in at first: the form asks an email, and the account's login is
`seb@mail.com`, `seb` being its display name; the user reset its password with a bcrypt hash
written into the `auth` table from inside the container, the recovery OpenWebUI documents, typed
by the user. A first guess, an interface cached in the browser, was wrong: the server had simply
received no sign-in until then. The user then added the OpenAI connection
`http://10.0.2.2:8080/v1`, key `sk-local`, and turned the Ollama API off. OpenWebUI 0.12 created
the connection with `api_type: "responses"`: it talks to the gateway through `/v1/responses`. Its
task model was left empty, which this version reads as "the current chat model": what §7.3 asks
for, and better than naming the 27B, which a title on the 35B would swap in. Autocomplete was
already off; title, tags and follow-up generation on.

A chat on the 27B then streamed its reasoning and its answer, and got its title, "Cache KV", and
four follow-up questions. The gateway's log shows a first chat on the 35B, the default model,
deleted by the user, then the swap to the 27B, loaded in 10 s, and four requests on it. The answer
itself was the largest, 7134 tokens in for a one-line question (`usage` in the saved message):
OpenWebUI 0.12 calls tools natively and sends its built-in tools — memory, notes, chats, and so on
— with every chat. A 275-token request, sent with the chat and served before it, held the answer
2.7 s in llama-server's single slot; title and follow-ups came after the answer, 364 and 310
tokens in, 422 and 247 generated, mostly reasoning at the server's `medium` effort.

A second message in the same chat made two requests: the answer, 7269 tokens in of which 7130
came from the cache — the slot held the follow-ups' prompt, and llama-server restored the chat's
context from its RAM cache — and the follow-ups. The 275-token request did not come back: a task
of a chat's first message only, most likely its title, since tags gave nothing and the
follow-ups come after the answer; OpenWebUI's middleware starts title generation after the
answer, so where this request comes from was not settled.

`docs/stack.md` follows: §7.1 says both other APIs stream through llama-swap, tool calls included,
and how llama-server orders a Messages stream's blocks; §7.2 how to install OpenCode from a
checked release, and what was checked with it; §7.3 the container's command, the connection at
`10.0.2.2`, the task model left empty, the Responses API, the built-in tools' cost and the task
request ahead of a first answer; §11's checklist the empty task model. The command in §7.3
publishes the port on `127.0.0.1` only, where v1's container, recreated as it was, publishes it on
every interface.

The client task ticked: 14 of 14.

The user asked to close the phase and, since it is the roadmap's last, the roadmap. The three
checks passed before any closing edit — `cargo fmt --check`, `cargo clippy --all-targets -- -D
warnings`, and `cargo test` with 492, 23 and 2 tests — and the five acceptance criteria hold.

State of the machine at the closure: the gateway running on `127.0.0.1:8080`, the 27B loaded for
the OpenWebUI check; OpenCode 1.19.0 in `~/.opencode/bin` with `deploy/opencode.json` installed and
v1's configuration in `~/.config/opencode/opencode.json.v1-backup`; the `openwebui` container
running OpenWebUI 0.12.0 on port 18000, connected to the gateway, its April data in
`~/.local/share/open-webui-backup-2026-10-12.tar.gz`; the image `server-cuda13-b11541` in podman's
storage; the baseline's two runs and four trials in `~/.local/share/ai-ctl/bench/`.

---

## Decisions

- **The streaming client is the HTTP/1.0 `TcpStream` client ai-ctl already uses for llama-swap**
  (`src/infrastructure/gateway_api.rs`), with no new crate, as decided for the gateway. llama-swap
  answers HTTP/1.0 without chunked encoding, and one connection per request matches
  `disableKeepAlives`. Taken with the user.
- **Repetitions over duration.** Each measure is repeated, 5 times by default and more where it
  narrows the spread; a run's duration is not a constraint. The real limit is the laptop's heat,
  so NVML samples temperature, SM clock and power with the VRAM, and a throttled measure shows. A
  p99 is reported only over enough samples: inter-token latencies, not five requests. Taken with
  the user.
- **The multi-turn session reproduces a real agent**, reasoning sent back in the history as
  OpenCode does. If the template drops past reasoning, the prefix changes and the hybrid 27B
  restarts from its nearest checkpoint: the scenario shows it rather than working around it. Taken
  with the user.
- **Runtimes and settings are compared through the inventory.** An entry has one backend: native
  build against official image, or one `spec-draft-n-max` against another, is an edit of the entry
  and `ai-ctl up`, one run each, then `bench compare`. The bench rewrites nothing. Taken with the
  user.
- **Depths reach the entry's limit**: 8K, 32K, 64K and `ctx_size` less the tokens to generate, a
  depth beyond the limit skipped; the context is prefilled once, so repetitions measure generation
  only. Proposed by the user.
- **The gateway's overhead is measured** against llama-server's own address from `/running`; the
  bench reads that address and starts nothing, so decision 0001 holds. Proposed by the user.
- **This roadmap stays on llama.cpp.** vLLM and models larger than the VRAM are directions for
  later roadmaps, in a working note on vLLM and a working note on models larger than the VRAM. Taken with the user.
- **Aggregates differ from v1's in three ways**: an empty series has no summary rather than zeros,
  which read as a speed of 0; the standard deviation is a sample's (divided by n − 1), five
  repetitions being a sample of a setting; and the p99 needs 100 values, below which it is the
  maximum or an interpolation toward it. Count, minimum, maximum and the relative spread are added,
  the last to tell when more repetitions are needed.
- **The domain derives speeds from counts and milliseconds** (`prompt_n` over `prompt_ms`), as
  llama-server computes its `*_per_second` fields, rather than storing both. Without `timings`, a
  server's generation speed comes from the client: the tokens after the first over the time after
  it. A prefill speed has no such fallback.
- **The plan's defaults**: 512 tokens generated per depth and session request; a 40K prefill
  prompt, shrunk to the context when it is smaller, generating one token; sessions of 10 turns
  opening on 8K tokens and adding 1K per turn, 22528 tokens at most, a model whose context cannot
  hold it being refused rather than given a smaller session; 50 overhead requests each way, 16
  tokens each; 5 repetitions. Constants in `src/domain/bench.rs`, to revise after the baseline.
- **The domain's bench types carry no serde derive**, as the rest of the domain: the result store
  maps them to a format of its own, as the build store maps a build to `build.json`
  (`src/infrastructure/build_store.rs`).
- **The plan's context is the server's `n_ctx`**, read from `/upstream/<model>/props`, rather than
  the inventory's `ctx_size`: the size in force is what limits a prompt. Taken with the user.
- **MTP acceptance is read per request** from `timings.draft_n` and `draft_n_accepted`; the
  `/metrics` counters read before and after a run check it. Taken with the user.
- **One HTTP/1.0 client for the loopback**, `src/infrastructure/http.rs`, shared by llama-swap's API
  and the chat client. It decodes chunks, since llama-server chunks a stream even to an HTTP/1.0
  request, and sends `Connection: close` on every request. A streamed request ends at
  `data: [DONE]`, not at the close.
- **The client waits 300 s at most for a byte**: a model loads within llama-swap's 120 s health
  check before its first request is answered, and llama-server pings every 30 s during a long
  prefill.
- **Arrival gaps replace inter-token latency.** With MTP, tokens arrive in bursts; a gap under 1 ms
  continues a burst and is left out of `arrival_gaps`, whose median is then a decoding step and
  whose p99 its stalls. The client keeps every gap, zeros included.
- **The prompts are an agent's at work on ai-ctl's own code**: a fictional request on real code
  frozen at `feat(bench): stream and time chat completions` of 2026-10-11, the files the request needs first, so the model generates what an agent
  does at every depth. Generated code was set aside, its repetition inflating MTP's acceptance.
  Proposed by the user, the frozen real code taken with the user. The corpus changes only by a
  deliberate rebuild, which makes runs before and after it incomparable.
- **A fresh prompt is tagged at its head**, `Session <run>/<scenario>/<index>.` opening the system
  prompt; only the template's first tokens are then shared with an earlier prompt. Superseded the
  same day by `cache_prompt: false`, below.
- **A prompt is fitted with the model's own template and tokenizer**: the corpus tokenized once
  with its pieces, the agent's own tokens counted, then up to four attempts. When none is exact,
  the longest prompt under the target is kept, and the run records the tokens actually sent.
- **Session replies have a fixed length**: 512 tokens with `ignore_eos`, so a session's history
  has the same size from one run, build or setting to the next, and its turns compare at equal
  size. At the server's `medium` effort, 512 tokens are all reasoning anyway: the reply is the
  model's, cut. Taken with the user.
- **NVML is sampled on a `std::thread`**, at a fixed period and stopped through a channel, inside
  the adapter behind a port: the crate stays synchronous and adds no crate, against the `tokio`
  that `docs/architecture.md` foresaw for this case. Taken with the user.
- **A run measures one model, its scenarios in a fixed order**: depth, prefill, session,
  overhead, whatever the order of `--scenario`, after an unmeasured warm-up — the agent's prompt
  without code, 16 tokens. Every request asks `ignore_eos`, its exact length, and seed 42, so two
  runs generate alike.
- **What a run cannot do without fails it before the first request; what only enriches it gives
  a warning.** A model without a `proxy` address fails a run that includes the overhead, before
  anything is sent; `/metrics` or NVML that cannot be read leave the drafts check or the GPU
  readings out, and the run goes on. A failed request ends the run.
- **The drafts are checked against the server's counters**, read after the warm-up and at the end:
  the requests in between, measured or not, must account for every draft, to the rounding of
  counters written with six significant digits. A difference means another client used the model
  during the run.
- **The GPU is read every 250 ms, each reading labelled with the scenario under way**, and
  summarised per scenario and over the run: peak memory, highest temperature, median and lowest SM
  clock, power, and the share of readings slowed by the power cap, by heat, or by the hardware.
  The power cap is a laptop GPU's usual state under load; heat is the one that spoils a measure.
- **Two runs send the same prompts; a request that must miss the cache asks `cache_prompt:
  false`** — each depth's fill, the prefills, a session's first turn. A tag that set a run's
  prompts apart changed what the model generated, hence MTP's acceptance and the speed, from one
  run to the next: the runtime under test would not have been the only difference.
- **Every request leaves one token of the context free** beyond those it generates (`FREE` in
  `src/domain/bench.rs`), since llama-server ends a generation once the context holds all but one
  token: the 27B's limit depth is 97791.
- **A run records what it measured, and refuses to start when it could not**: the model's
  inventory entry, its command line, its source — the build, or the image and its digest — and the
  machine, taken from the configuration the inventory and the builds give, which the gateway must
  be running. A digest podman cannot give leaves a warning.
- **A run is saved whole**, as JSON in `~/.local/share/ai-ctl/bench/<start>-<model>.json`, every
  request's timings kept, gaps between tokens included, so that a later ai-ctl can summarise them
  otherwise; a saved run is never replaced. It is saved after its tables are printed.
- **The baseline compares the native `b11538` with the official `server-cuda13-b11541`**, the
  image of the nearest published build: the three commits between them touch no code CUDA builds
  or llama-server runs. The inventory names the image by this pinned tag for the baseline, not by
  the floating `server-cuda13`, whose build moves.
- **`bench compare` takes the first run as the reference**: with two runs, a column gives the
  second's change, in percent, or in points for shares; with more, the columns alone. The
  gateway's overhead has no change column, a difference of differences saying little.
- **The native build stays the default runtime, the official image a fallback.** The baseline
  found them equal in generation and prefill, the image 50 to 80 ms slower on an agent turn's
  first token: nothing in it calls for a change of the inventory.
- **A fresh request comes after the context is set aside**: an unmeasured request of the same
  prompt cut to a quarter, sent before each fill, prefill and session, pays llama-server's save of
  the slot's context into its RAM cache, and leaves a prompt the fresh one selects by similarity
  and keeps whole, so that no save or load of the cache falls in a measure and a scenario's first
  repetition is measured as its others. The overhead's own first request became the same kind,
  with the warm-up's prompt. Chosen over leaving the save in the measure, since it is the previous
  scenario's cost, not the measured request's; an agent's own switch between sessions pays it,
  which the bench does not measure. The warm-up's prompt alone was tried first, and left 0.2 to
  0.4 s of the cache's work in each fresh request. Left to the approach judged best by the user.
- **The CDI specification is the toolkit's**, which `nvidia-cdi-refresh` keeps in
  `/var/run/cdi/nvidia.yaml`; doctor checks that the one in use mounts the loaded driver's
  libraries and that none it shadows names another driver. Asked by the user.
- **OpenCode is installed from a checked release**: the archive's SHA-256 against the digest
  GitHub publishes, then the install script's `--binary`, since the script itself checks nothing.
- **OpenWebUI's task model is left empty**, the current chat model, rather than set to the 27B:
  `docs/stack.md` asked for "the chat model", and with two models only the empty setting gives it
  without a swap.

---

## Files Changed

Computed against the start commit `docs: close phase 3` of 2026-10-11 with `git diff -M --name-status` and the untracked
files (none). The phase's thirteen commits, `docs: open phase 4` of 2026-10-11 to `docs: record phase 4's baseline, native build against official image`, and this closure are counted.

**Added**
- `assets/bench/corpus.txt`
- a working note on models larger than the VRAM
- a working note on vLLM
- `docs/roadmap/on-progress/v2-rebuild/phase-4-bench-report.md`
- `src/application/bench.rs`
- `src/domain/bench.rs`
- `src/domain/prompt.rs`
- `src/domain/stats.rs`
- `src/infrastructure/chat_api.rs`
- `src/infrastructure/http.rs`
- `src/infrastructure/run_store.rs`
- `src/infrastructure/server_api.rs`
- `src/presentation/bench.rs`
- `src/presentation/compare.rs`
- `tests/fixtures/llama-server-b11538-apply-template.http`
- `tests/fixtures/llama-server-b11538-chat-stream.http`
- `tests/fixtures/llama-server-b11538-metrics.http`
- `tests/fixtures/llama-server-b11538-props.http`
- `tests/fixtures/llama-server-b11538-tokenize-pieces.http`
- `tests/fixtures/llama-server-b11538-tokenize.http`
- `tests/fixtures/llama-swap-263-chat-bad-request.http`
- `tests/fixtures/llama-swap-263-chat-stream.http`
- `tests/fixtures/llama-swap-263-chat-unknown-model.http`
- `tests/fixtures/nvidia-ctk-1.20.1-cdi-spec-excerpt.yaml`

**Modified**
- `README.md`
- `docs/architecture.md`
- `docs/roadmap/on-progress/v2-rebuild/README.md`
- `docs/roadmap/on-progress/v2-rebuild/phase-4-bench.md`
- `docs/stack.md`
- `src/application/doctor.rs`
- `src/application/fakes.rs`
- `src/application/lifecycle.rs`
- `src/application/mod.rs`
- `src/application/ports.rs`
- `src/domain/machine.rs`
- `src/domain/mod.rs`
- `src/infrastructure/gateway_api.rs`
- `src/infrastructure/host.rs`
- `src/infrastructure/mod.rs`
- `src/infrastructure/nvml.rs`
- `src/infrastructure/paths.rs`
- `src/infrastructure/podman.rs`
- `src/main.rs`
- `src/presentation/cli.rs`
- `src/presentation/lifecycle.rs`
- `src/presentation/mod.rs`
- `tests/cli.rs`
- `tests/gateway_lifecycle.rs`

---

## Problems And Deviations

- **The tag of fresh prompts made runs incomparable.** Each run's prompts held its start, so the
  model generated other replies, and MTP accepted another share of drafts, from one run to the
  next: 65.2 % against 84 % at the limit depth in the second live run. Fixed: the prompts are the
  same in every run, `cache_prompt: false` keeps a fresh request out of the cache, and the phase's
  constraint on prompts says so.
- **A request at the context's limit could generate one token fewer than asked**, llama-server
  stopping once the context holds all but one token. Fixed: every request leaves one token free.
- **The overhead scenario was written with the other scenarios**, before MTP and VRAM rather than
  after them, so that `--scenario` never offered an unfinished one.
- **A scenario's first repetition pays the save of the previous scenario's context.** With
  `cache-ram`, llama-server saves the context a slot holds into the RAM prompt cache before it
  takes a new prompt, unless that prompt is already there: 0.9 to 2.6 s on the first prefill and
  the first two sessions' first turn, nothing on the identical repetitions after them. The
  server's speeds are untouched, a first repetition's TTFT is not, and the session's first turn
  shows ± 1.1 s. Fixed: every fresh request comes after an unmeasured request of its prompt cut
  to a quarter, which pays the save; checked live, a first repetition now measures as the others.
  The baseline's two runs were made before the fix; they compare with each other, and a later run
  compares with them except on the TTFT of a scenario's first repetition.
- **The CDI specification doctor recommended goes stale.** Doctor's remedy wrote
  `/etc/cdi/nvidia.yaml`, which still named driver 580.119.02 here; the container worked because
  the toolkit's `nvidia-cdi-refresh` unit regenerates `/var/run/cdi/nvidia.yaml` at each driver
  change, and CDI prefers it. Doctor checked only that `nvidia.com/gpu=all` is listed. Fixed:
  doctor compares the specifications' driver with the loaded one, and its remedy enables the
  toolkit's units. The stale file is still on the machine, for the user to remove.
- **The image's slower short prompts on a long context were not explained**: 5 to 15 % more
  `prompt_ms`, with the CUDA architectures, CUDA's version and memlock ruled out. Left open; it
  does not change the default runtime.
- **`docs/stack.md` gave OpenWebUI the gateway's address as seen from the host**,
  `127.0.0.1:8080`, which inside its container is the container itself; v1 had found `10.0.2.2` and
  recorded it in a note that the v2 restart deleted. Fixed in §7.3.
- **OpenWebUI's account was not reachable at first**: its login is an email the user did not
  remember, and the password was reset in the database. No consequence for ai-ctl.
- **OpenWebUI 0.12 costs a local model more than its settings show**: 7.1K tokens of built-in
  tools in every chat, a task request served before a chat's first answer, and reasoning in every
  task request. Recorded in `docs/stack.md` §7.3; not changed, the tools being the user's to keep
  or turn off per model.
- **The origin of the 275-token request before a first answer was not settled**: most likely the
  title, by elimination, though OpenWebUI's middleware starts title generation after the answer.
  Left open.

---

## Changes To Later Phases

None: this phase is the roadmap's last, and no later phase exists to change.

---

## Assessment

Phase 4 delivered its objective. `ai-ctl bench run` measures a model through the gateway with the
protocol of `docs/stack.md` §9: generation at 8K, 32K and 64K tokens and at the context's limit,
the prefill of a fresh 40K-token prompt, ten-turn sessions sending their reasoning back, MTP's
acceptance per request checked against the server's counters, the GPU's memory, temperature, clock,
power and throttling read through NVML, and the gateway's own cost against llama-server's address.
A run records the inventory entry, command line, build or image digest and machine it measured,
refuses to start when the gateway runs another configuration, and is saved whole as JSON;
`bench list` and `bench compare` read the saved runs. The baseline found the native `b11538` and
the official `server-cuda13-b11541` equal in generation and prefill, within 1 %, and the image 5 to
15 % slower on short prompts over a long context, 50 to 80 ms on an agent turn's first token: the
native build stays the default. The gateway adds a millisecond or two to a first token. OpenCode,
OpenWebUI, `/v1/messages` and `/v1/responses` work through the gateway, OpenCode's tool calls
through the 27B's template included, which closes phase 3's last criterion.

Most of the phase's work went into making a measure mean what it says. Three traps of
llama-server came out of live runs, each read in its source at `b11538` before it was fixed: the
generated tokens set MTP's acceptance, so prompts that differed between runs made runs
incomparable; a generation stops one token short of a full context; and with `cache-ram`, a
scenario's first repetition paid the save of the previous scenario's context. The prompts are now
the same in every run with `cache_prompt: false` where a request must miss the cache, every request
leaves a token free, and an unmeasured request of the same prompt cut to a quarter sets the context
aside before each fresh one. The image run also showed `doctor` recommending a static CDI
specification that goes stale at the next driver; doctor now checks the specification against the
loaded driver. The phase grew by one scenario, the gateway's overhead, and by `bench list`.

What a later roadmap needs to know first:

- **The bench is how a choice is made**: an edit of the inventory entry, `ai-ctl up`, one run each,
  `ai-ctl bench compare`. The bench rewrites nothing. Runs made before `fix(bench): set the slot's context aside before each fresh request` differ from later
  ones on the time to a scenario's first token, the baseline's two runs included.
- **The corpus is frozen at `feat(bench): stream and time chat completions` of 2026-10-11**; rebuilding it makes every earlier run incomparable.
- **The 27B on this machine**: 75 down to 45 tokens/s from 8K to 97K tokens of context, a 40K
  prefill at about 1390 tokens/s, 93 % of MTP drafts accepted, 21352 MiB at peak, the GPU slowed by
  its power limit in nearly every reading under load and never by heat. These are the prefill
  figures `docs/stack.md` §4.2 sets as vLLM's criterion.
- **Clients cost more than their settings show**: OpenWebUI 0.12 sends 7.1K tokens of built-in
  tools with every chat and a task request ahead of a first answer; in a Messages stream,
  llama-server's order of blocks is not Anthropic's, which a strict orchestrator client may refuse.
- **Left open**: why the image is slower on short prompts over a long context, and where
  OpenWebUI's request ahead of a first answer comes from.

