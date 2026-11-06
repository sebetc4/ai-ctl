# Review: the v2-review-fixes roadmap

**Status:** complete — 84 findings in five passes, all `CONFIRMED`; 83 confirmed and 1 rejected (84) on 2026-10-18 by the session under the user's delegation, the hard cases by the user; 58 batches to fix in this roadmap, then a security pass; numbers 6, 61, 70, 79 and 86 merged into findings 4, 51, 55, 10 and 55
**Started:** 2026-10-17

## Scope

| Fact | Value |
|---|---|
| Scope | roadmap `docs/roadmap/on-progress/v2-review-fixes` |
| Start commits | phase 0 `docs(v2-review-fixes): create the roadmap` of 2026-10-16; phase 1 `docs(v2-review-fixes): close phase 0, security` of 2026-10-16; phase 2 `docs(v2-review-fixes): close phase 1, bugs and error handling` of 2026-10-16; phase 3 `docs(tests): index the fixtures with how each was captured` of 2026-10-16; phase 4 `docs(v2-review-fixes): close phase 3, performance`; phase 5 `docs(v2-review-fixes): close phase 4, structure and refactoring` |
| Base | `docs(v2-review-fixes): create the roadmap` of 2026-10-16, the start commit of phase 0 |
| End | `docs(v2-review-fixes): record the closing review's scope and tier`, the last commit touching `docs/roadmap/on-progress/v2-review-fixes` |
| Files | 87: production 48, tests 15, other 24, binary 0 |
| Lines | production 15,218, tests 14,560, other 6,227 |
| Uncommitted | none |

Set with `scripts/scope.py roadmap docs/roadmap/on-progress/v2-review-fixes --files`, exit 0;
the diff is `git diff` from `docs(v2-review-fixes): create the roadmap` of 2026-10-16 to `docs(v2-review-fixes): record the closing review's scope and tier`, +11,790 −3,029. It holds phases 0 to 4 and phase 5's
opening: the one commit after `docs(v2-review-fixes): open phase 5`, `docs(v2-review-fixes): record the closing review's scope and tier`, touches only phase 5's report, and no file
of `src/`, `tests/`, `Cargo.toml`, `Cargo.lock` or `deploy/` changed between `docs(v2-review-fixes): record the closing review's scope and tier` and the
review's `HEAD`. The scope holds 48 of the crate's 54 production files; the six outside it are
`src/lib.rs`, `src/application/mod.rs`, `src/domain/mod.rs`, `src/domain/diagnostics.rs`,
`src/presentation/completions.rs` and `src/presentation/doctor.rs`. Deleted in the diff:
`tests/fixtures/git-ls-remote-tags.txt`, and the roadmap's files under `pending/`, moved to
`on-progress/`.

Phase 4's Assessment names three leads, given to the reviewers of their axes:
`GatewayProcess` still holds ten methods, of which `Bench` calls three (structure, cleanup);
the bench's setup reads the GPU twice, where the other commands read it once (performance);
batches 43 to 48 changed error and status texts (the CLI's contract).

## Checks

| Command | Result |
|---|---|
| `cargo fmt --check` | green |
| `cargo clippy --all-targets -- -D warnings` | green |
| `cargo test` | green, 664 passed: 635 in the library, 25 in `tests/cli.rs`, 3 in `tests/gateway_config.rs`, 1 in `tests/gateway_lifecycle.rs`; none skipped itself (`up_a_proxied_request_and_down ... ok`, llama-swap installed) |
| `python3 .claude/skills/reviewing-delivered-code/evals/test_scope.py` | green, 15 tests |
| `python3 .claude/skills/reviewing-delivered-code/evals/test_fixtures.py` | green, 1 test |
| `cargo audit` | green: 1,338 advisories loaded, 66 crate dependencies scanned, no vulnerability and no warning (cargo-audit 0.22.2) |
| `cargo mutants --in-diff` | not run: the scope's diff holds 551 mutants (`cargo mutants --list --in-diff`, cargo-mutants 27.1.0), over the limit of 300; at the probe's rate of 25 mutants in 68 s, about 25 minutes |

**Missed mutants:** none to give: `cargo mutants` did not run.

**Mutants the agents wrote:** `review-tests` wrote 28 by hand, M1 to M28, and pass 3's verifier
ran each again from its own definitions, in two clones of `docs(v2-review-fixes): record the session close of the closing review's pass 3` of 2026-10-18 in scratch. Without the
proposed tests, each survived the whole suite: 663 passed in libtest and
`up_a_proxied_request_and_down ... ok`, 664 in all; the reviewer's 663 left out
`gateway_lifecycle`, whose harness prints its own line. With the proposed tests, each failed
its own test and only it. With the patch applied, `cargo test` passed 689, and `cargo fmt
--check` and `cargo clippy --all-targets -- -D warnings` were green. Before a proposed test
lands, it drops its `review_` prefix and takes the blank line the module puts before each
`#[test]`. The verifier wrote M29, which survives and which no proposed test kills (finding
49). Pass 4's first verifier ran M29 again in a clone of `docs(v2-review-fixes): record the session close of the closing review's pass 4` of 2026-10-18: it survives, and the test
it wrote for finding 49 kills it; it wrote M30, which survives, and which finding 55's fix
removes with its text. Pass 4's second verifier wrote M31, which survives: no test can
reach that branch, and finding 69's fix removes it. Pass 4's verifiers' patches, as diffs of their
clones of `docs(v2-review-fixes): record the session close of the closing review's pass 4` of 2026-10-18 and `docs(v2-review-fixes): record pass 4's first verification of the closing review` of 2026-10-18 (finding 49's test, `t49.patch`; M29, M30 and M31; the fixes
of findings 52, 56 and 57, `ref.patch`, and 60, `f60.patch`), with the first verifier's logs
and tools, are kept beside pass 3's, in
`~/.local/share/ai-ctl/review-scratch/2026-10-18-v2-review-fixes-pass-4/`. The patch of proposed tests, the mutants' definitions as exact replacements
(`verifier/tools/mutants.py`) and the summaries of their runs are kept outside the
repository, in `~/.local/share/ai-ctl/review-scratch/2026-10-18-v2-review-fixes-review-tests/`.
Pass 5's agents wrote no mutant. Its verifier's work that the fixes will use is kept in
`~/.local/share/ai-ctl/review-scratch/2026-10-18-v2-review-fixes-pass-5/verifier/`, as diffs
of its clones of `docs(v2-review-fixes): record pass 5's performance review of the closing review` of 2026-10-18: finding 81's test and capture (`repo81.patch`, `props-nope.http`),
finding 82's probe test and split (`repo82.patch`), finding 85's fix and probe test
(`repo85.patch`, `repo85-probe.patch`, `repo-probe.patch`) and finding 88's probe test
(`repo88.patch`), with its timing scripts and harness; the reviewer's scripts, wrappers and
harnesses sit beside them, in `review-performance/`.

| Mutant | Change | Findings |
|---|---|---|
| M1 | `lifecycle.rs:37`, `STOP_POLLS` from 400 to 4 | 27 |
| M2 | `lifecycle.rs:719`, `time_left.min(CHECK)` to `CHECK` | 28 |
| M3 | `lifecycle.rs:725`, `POLL.min(time_left)` to `POLL` | 28 |
| M4 | `lifecycle.rs:484-486`, the log's write error ignored | 25 |
| M5 | `doctor.rs:154`, the record's error arm to `Err(_) => {}` | 29 |
| M6 | `gateway_process.rs:225-226`, the child's kill and wait deleted | 24 |
| M7 | `application/gateway.rs:125`, `if outcome != Outcome::Unchanged` to `if true` | 45 |
| M8 | `application/build.rs:428-430`, `store.remove`'s error ignored | 39 |
| M9 | `application/bench.rs:416`, `.find(…)` to `.next()` | 30 |
| M10 | `cli.rs:181`, `ScenarioArg::Session` to `Scenario::Prefill` | 31 |
| M11 | `gateway_config.rs:414-418`, the path through `to_string_lossy` | 42 |
| M12 | `git.rs:90-91`, the branch before the tag | 44 |
| M13 | `doctor.rs:521`, the `Some([])` arm deleted | 47 |
| M14 | `lifecycle.rs:777`, `signal`'s error ignored | 41 |
| M15 | `application/bench.rs:243`, the monitor's error arm to `Err(_) => {}` | 46 |
| M16 | `presentation/lifecycle.rs:66`, `status` always exits 3 | 32 |
| M17 | `build_store.rs:245`, the rename that puts the old build back deleted | 34 |
| M18 | `git.rs:81`, the fetch's error ignored | 35 |
| M19 | `cmake.rs:75`, the configure's error ignored | 36 |
| M20 | `host.rs:163`, `Probed::Failed` to `Probed::Missing` | 37 |
| M21 | `lifecycle.rs:245`, `is_free`'s error taken as free | 38 |
| M22 | `lifecycle.rs:225`, `up`'s lock deleted | 23 |
| M23 | `lifecycle.rs:305`, `down`'s lock deleted | 23 |
| M24 | `application/build.rs:353`, `use_build`'s lock deleted | 40 |
| M25 | `compare.rs:124`, the `first.mean != 0.0` guard dropped | 48 |
| M26 | `http.rs:440-442`, the stream's write error ignored | 26 |
| M27 | `inventory_file.rs:77-79`, the directory through `to_string_lossy` | 43 |
| M28 | `presentation/bench.rs:55-56`, an unsaved run succeeds | 33 |
| M29 | `lifecycle.rs:348`, `status`'s lock deleted; pass 3's verifier's | 49 |
| M30 | `application/bench.rs:357`, the text of "cannot record the run's GPU" replaced; pass 4's first verifier's | 55 |
| M31 | `lifecycle.rs:249-254`, up's `warn_unread` replaced by an assertion that the reading is found; pass 4's second verifier's, a proof that the branch cannot be reached | 69 |

## Tier

**Tier:** `high` — **Estimate:** about $55 by the skill's rates, in four passes: six reviewers
$25.85 (five at $3.75 on the production lines, `review-tests` at $7.10 on the production and
test lines), six verifiers $13.98 at $2.33 until step 7 prices them by their findings, the
session $15 ($3.50 a pass and $1 more for the last). Approved by the user on 2026-10-17. The
verification of passes 1 and 3 is priced by its findings, about $0.13 each: past about 31
findings, it would take the pass over $15 and moves to the next pass.

Pass 1's two reviewers measured $7.65 (`review-security`) and $10.69 (`review-correctness`) by
`usage.py`, part of it estimated, against $3.75 each at the skill's rate: the session stood at
$20.21, $9.07 of it estimated, when they had answered. Their verification, about $2.50 at the
rate for 20 findings, became a pass of its own at the user's word on 2026-10-17: five passes.

Pass 2's two verifiers measured $6.12 (findings 1 to 5, $5.18 of it estimated) and $6.86
(findings 7 to 21, $5.72 estimated) by `usage.py`, against about $2.50 together at the rate of
step 7; both ran at effort `high`, as their `.meta.json` records. The pass's session stood at
$15.35, $10.91 of it estimated, when they had answered. They gave 19 findings `CONFIRMED` and
finding 1 `UNCERTAIN`; finding 1 went from high to medium; the one lead, on `build list`,
was merged into finding 9.

**Corrected on 2026-10-17.** The agents' figures above are `usage.py`'s, which gives each call
without its final usage 5,400 output tokens: against the `cost-state` records of the first
review's five sessions, that overstates a review agent's cost 1.9 to 3.3 times. Recomputed with
845 output tokens for each such call, the output per call the first review measured from its
`cost-state` record: `review-security` about $4.37, `review-correctness` $6.32, the two
verifiers $1.74 and $2.03. The reviewers cost 1.2 to 1.7 times the skill's rate, and the
verifiers together about 1.5 times step 7's price, not two to three and five times.

Pass 3, counted with 845 output tokens for each call without its final usage: `review-tests`
about $7.67, 33 of its 56 calls estimated, against $7.10 at the skill's rate; its verifier, on
lead 22 and findings 23 to 48, about $2.19, 51 of its 52 calls estimated, against about $3.50
at step 7's price. `usage.py` as shipped gives the verifier $6.83. Both ran at effort `high`,
as their `.meta.json` records. The verifier gave the 27 findings `CONFIRMED` and left one
lead, finding 49.

Pass 4, counted with 845 output tokens for each call without its final usage: the first
verifier, on findings 49 to 60, about $3.12, its 80 calls all estimated, against about $1.52
at step 7's price; `usage.py` as shipped gives $10.40. It ran at effort `high`, as its
`.meta.json` records, and gave the 12 findings `CONFIRMED`, with 51's architecture block
refuted as a defect of its own, and no lead. The second verifier, on findings 62 to 69 and 71
to 79, about $2.57, 65 of its 68 calls estimated, against about $2.14 at step 7's price;
`usage.py` as shipped gives $8.50. It ran at effort `high`, as its `.meta.json` records, and
gave the 17 findings `CONFIRMED`, 79 merged into finding 10, and four leads: three are
findings 80 to 82, for pass 5's verifier, and the fourth went into finding 65.

Pass 5, counted with 845 output tokens for each call without its final usage:
`review-performance` about $4.45, 39 of its 57 calls estimated, against $3.75 at the skill's
rate; `usage.py` as shipped gives $8.00. It ran at effort `high`, as its `.meta.json` records,
with 69 tool calls in about 14 minutes, and gave six findings, 83 to 88, of which 86 adds a
measurement to finding 55. Its verifier, on findings 80 to 85, 87 and 88 and on what 86 adds to
55, about $2.54, 60 of its 63 calls estimated, against about $1.13 at step 7's price;
`usage.py` as shipped gives $8.01. It ran at effort `high`, as its `.meta.json` records, with
64 tool calls in about 12 minutes, gave the nine `CONFIRMED`, set the fixes of 80, 81 and 82,
corrected figures or fixes in 84, 86 and 88, and left one lead, finding 89. Finding 89's
verifier, about $0.43, its 13 calls all estimated, against about $0.13 at step 7's price;
`usage.py` as shipped gives $1.62. It ran at effort `high`, as its `.meta.json` records, with
14 tool calls in about 2 minutes, gave 89 `CONFIRMED` with its fix, and left one lead, under
Coverage.

| Pass | Agent | Axes | Effort |
|---|---|---|---|
| 1 | `review-security` | security | high |
| 1 | `review-correctness` | bugs, error handling, interruption and concurrency | high |
| 2 | `review-verifier` ×2, one per reviewer of pass 1 | pass 1's findings, by axis | high |
| 3 | `review-tests` | tests | high |
| 3 | `review-verifier` | the pass's findings | high |
| 4 | `review-cleanup` | dead and legacy code, refactoring | high |
| 4 | `review-structure` | architecture, conventions, on-disk formats, the CLI's contract | high |
| 4 | `review-verifier` ×2, one per reviewer | the pass's findings, by axis | high |
| 5 | `review-performance` | performance | high |
| 5 | `review-verifier` | the pass's findings, and findings 80 to 82 | high |
| 5 | `review-verifier` | finding 89, a lead of the pass's verifier | high |

## Findings

### 1. llama-swap trusts any process that answers /health on a backend's predictable loopback port, and sends it the client's API key and prompt

- **Axes:** security — raised by `review-security`
- **Action:** fix
- **Severity:** high, the reviewer's: the verifier had lowered it to medium for the race's doubt
  alone, which the measurement of 2026-10-18 removes
- **Where:** `src/domain/gateway_config.rs:284-285`, `src/domain/gateway_config.rs:345-346`, `src/domain/gateway_config.rs:425-453`, `docs/security.md:34-41`
- **Cause:** the configuration points each model at `proxy: "http://127.0.0.1:${PORT}"` with
  `checkEndpoint: "/health"` and writes no `startPort`, so llama-swap gives out predictable
  ports from its default of 5800. A model starts on demand, and unloads on its ttl, which frees
  its port again. llama-swap takes any 200 on that address as the backend, and never checks
  that the process it started holds the socket. A process that binds the port before the model
  loads makes llama-server fail to bind and exit, and if llama-swap's first health check comes
  before that exit, llama-swap marks the squatter ready and forwards each request to it, the
  client's `Authorization` header unchanged (`docs/security.md:65-67`).
- **Consequence:** the attacker is another local account, or a process outside the user's
  account that can bind loopback: anyone the §1 table of `docs/security.md` says the key stops.
  They get the API key, which never rotates (finding 2), every prompt sent to that model, and
  control of every answer; with OpenCode's agents, a forged answer can carry tool calls, a path
  to running code as the user. `docs/security.md` §1 and §2 claim the opposite. This machine
  has one login account, `seb`, so today's attacker would be a future account or a compromised
  service account.
- **Fix:** take the backends off TCP loopback: llama-server accepts `--host <path>.sock`
  (`common/arg.cpp:3335`), a socket in a 0700 directory under the state directory; probe first
  whether llama-swap v263 can proxy and health-check a unix socket. Meanwhile, record the
  residual risk in `docs/security.md` §1 and §2: ports from 5800, a squatter taken for the
  backend, the key and prompts handed over, and no race for the squatter to win, since
  llama-server with the GPU outlives llama-swap's first check every time (Verification). The
  verifier drops the reviewer's "`startPort` the user picks": a
  configured port is just as findable, from the configuration or by scanning loopback.
- **Evidence:** reproduced by the reviewer: llama-swap v263 alone on 127.0.0.1:18931, with
  ai-ctl's proxy, `checkEndpoint` and env lines, a model whose command exits after 0.5 s
  without binding, and a Python squatter on the backend's port. The squatter logged `GET
  /health auth=None`, then `POST /v1/chat/completions auth=Bearer test-key-123`; the client
  received the squatter's answer, and llama-swap logged "Health check passed", then "upstream
  process exited unexpectedly".
- **Verification:** UNCERTAIN in pass 2, CONFIRMED by the measurement of 2026-10-18. Pass 2:
  reproduced on 127.0.0.1:18941, backend port 18942: with a
  backend command that lives 0.3 s or more, the squatter received the health check, then the
  bearer key and the body `secret prompt`, and the client got its answer. But llama-swap's
  first health check comes 0.25 to 0.3 s after it starts the command, and catches a command
  that exits before: `touch`, `timeout 0.05` and `timeout 0.2` gave "upstream command exited
  prematurely", HTTP 500, nothing reaching the squatter; `timeout 0.25` gave 503 three times
  out of three; `timeout 0.3` to `0.45`, 200 every time. The real llama-server binds before it
  loads the model (`tools/server/server.cpp:475-488`): on the squatted port, with
  `CUDA_VISIBLE_DEVICES=`, it exited in 0.12 s (b11538) and 0.09 s (b11700), logging
  "couldn't bind HTTP server socket" at 0.035 s; the official image under `podman run --rm
  --pull=never --network host`, with no device, exited in 0.21 s. The reviewer's 0.5 s
  command overstates the race. What would settle it: how long llama-server runs, with the GPU
  visible, before it fails to bind a taken port, natively and as the image with its device;
  before binding it enumerates the CUDA devices (`common/arg.cpp:253`, `:1148`,
  `ggml_cuda_init`). That needs a CUDA context, which the review's rules forbid. The squatter
  also needs another account, and a port that `SO_REUSEPORT` does not let it share across
  UIDs. On 2026-10-17 the user allowed that CUDA context, against the review's rules ("j'autorise
  à créer un context CUDA contre les règles de la review"). Measured on 2026-10-18 in pass 3's
  session, no agent, the gateway stopped, the GPU idle at 15 MiB, persistence mode off. On a
  loopback port a listener held, llama-server b11700, natively, with the `qwen3.8-27b` command
  of the generated configuration, logged "couldn't bind" at 0.24 to 0.44 s and exited at 0.40
  to 0.87 s, five runs, the first, cold, the slowest. The official image
  `server-cuda13-b11541`, under ai-ctl's `podman run` words with `--device
  nvidia.com/gpu=all` and the container name `review-probe-llama`, failed to bind at 0.50 to
  0.95 s and exited at 0.76 to 1.39 s. The CUDA initialisation before the bind and its teardown
  after make the difference with the runs without the GPU. End to end: llama-swap v263 alone
  on 127.0.0.1:18971, then :18981, with `startPort` on the squatted port, ai-ctl's proxy,
  `checkEndpoint` and env lines, and that real command as the model's. Three rounds out of
  three for each runtime: the squatter received `GET /health` 0.25 s after the request, then
  `POST /v1/chat/completions` with `Authorization: Bearer e2e-test-key` and the body `secret
  prompt`, and the client got the squatter's answer, HTTP 200. llama-swap logged "Health check
  passed", then llama-server "couldn't bind", then "upstream process exited unexpectedly". The
  squatter has no race to win: holding the port before the model loads is enough. It ran
  under the same account without `SO_REUSEPORT`, which is how the kernel treats a squatter of
  another account. The image ran with a CDI specification generated in scratch (`podman
  --cdi-spec-dir`): the live one names a library no longer on disk (lead 22). Nothing was left:
  `pgrep`, `podman ps`, 15 MiB on the GPU.
- **User's verdict:** confirmed — the user's, on 2026-10-18; the fix's transport, unix sockets or the documents alone, put to the user again with batch 1's probe and its impact

### 4. The reserved-flag and duplicate-flag checks miss underscore spellings, which llama-server reads as dashes

- **Axes:** security, bugs — raised by `review-security` and `review-correctness` (its F1,
  number 6, merged here and left unused)
- **Action:** fix
- **Severity:** medium: low on the security side alone, since whoever writes the inventory
  already runs code as the user (`docs/security.md` §1); medium on the bugs side
- **Where:** `src/domain/inventory.rs:38-51`, `src/domain/inventory.rs:369-372`, `src/domain/inventory.rs:384-386`, `src/domain/inventory.rs:406-408`, `src/domain/inventory.rs:418`, `src/domain/inventory.rs:94`, `src/domain/gateway_config.rs:410-412`
- **Cause:** `reserved_flags`, `check_flags` and the file-flag checks compare flag names as
  written, and `option` writes `--{flag}` unchanged. llama-server rewrites every `_` to `-` in
  a `--` argument before looking it up (`common/arg.cpp:825-828` and `:1218-1221` at b11700),
  `--api-key` appends to the key list (`:3518`), and a repeated option keeps its last value.
- **Consequence:** `api_key = "…"` under flags or defaults, or `api_key_file` under `files`,
  passes validation and becomes a second key on llama-server's command line, which `ps` shows
  to every user: the case phase 0 reserved `api-key` against, and against `docs/security.md:69-72`.
  `ctx_size = 4096`, the spelling the inventory itself uses for the model's field, silently
  overrides the context that the VRAM budget and the bench's records assume, a DEPRECATED
  warning in the gateway log the only trace. A `model_draft` flag passes beside a
  `model-draft` file. And since `Flags` is a `BTreeMap` (`inventory.rs:94`) where `-` sorts
  before `_`, a default spelled `batch_size` is written after a model's `batch-size` and wins,
  against "A model's own flags override these".
- **Fix:** refuse `_` in `check_flag_name` (`inventory.rs:418`), which already runs for
  defaults (`:273`, through `check_flags`), flags (`:407`) and files (`:369`): one line covers
  the three checks and the override order, where normalising would touch four comparisons.
  Neither `deploy/inventory.toml` nor the installed inventory uses an underscored flag. Test
  with `api_key`, `api_key_file` under `files`, `ctx_size`, and a `model_draft` flag beside a
  `model-draft` file.
- **Evidence:** reproduced by both reviewers: `ai-ctl doctor` on a scratch copy of the
  installed inventory reports `fail Inventory … flag \`api-key\` is set by ai-ctl` for
  `api-key`, exit 1, and `ok Inventory` for `api_key`, exit 0; a scratch test printed
  `--ctx-size 98304`, `--api_key second-key`, `--ctx_size 4096`, `--model_draft other.gguf`;
  llama-server b11700 accepts `--api_key k --help` and fails `--zz_unknown k --help` with
  `invalid argument: --zz-unknown`.
- **Verification:** CONFIRMED, with the methods of both axes — `ai-ctl doctor` on a copy of
  the installed inventory: `api-key` fails, exit 1; `api_key`, `ctx_size = 4096` and
  `model_draft = "other.gguf"` give `ok Inventory`, `ok Gateway configuration 2 models`, exit
  0. Test `verifier_underscored_reserved_flags_reach_the_command` on the shipped inventory: the
  command reads `--model-draft …/mtp-Qwen3.8-27B-Q4_0.gguf`, `--ctx-size 98304`, `--api_key
  second-key`, `--ctx_size 4096`, `--model_draft other.gguf`. llama-server b11538 and b11700,
  `CUDA_VISIBLE_DEVICES=`, `LLAMA_API_KEY=gateway-key --api_key second-key`: `api_keys: 2 keys
  loaded`, despite a warning that the variable "will be overwritten". The `BTreeMap` order is
  the verifier's.
- **User's verdict:** confirmed — the session, under the user's delegation of 2026-10-18

### 2. The API key never changes, and clients send it to 127.0.0.1:8080 whoever listens there while the gateway is down

- **Axes:** security — raised by `review-security`
- **Action:** fix
- **Severity:** medium
- **Where:** `src/infrastructure/api_key.rs:51-74`, `deploy/opencode.json:8-9`, `src/application/lifecycle.rs:244-248`, `docs/security.md:19`, `docs/security.md:49-51`, `docs/security.md:62-63`
- **Cause:** `read_or_create` keeps the key for ever ("Later runs keep it"). The shipped
  OpenCode configuration reads `{file:~/.local/state/ai-ctl/api-key}` and sends it as a bearer
  token to `http://127.0.0.1:8080/v1`, and the orchestrator and OpenWebUI (through
  10.0.2.2:8080) do the same. Between `down` and `up`, port 8080 is free, and any local account
  can bind it, since it is above 1024.
- **Consequence:** another local account binds 8080 while the gateway is stopped, a normal
  state, so it only has to wait: no race. Each client request hands it the key, which stays
  valid after the next `up`; `up` refuses the taken port, but only until the attacker lets it
  go. With the key they can load and evict models, read the log stream and use the GPU,
  everything §3 says the key guards, and while they hold the port they answer the clients'
  prompts. `docs/security.md` §3, "What It Does Not Guard", does not name this squat, and the
  §1 table says the key stops other users. This machine has one login account, `seb`.
- **Fix:** first, document the squat in §3 and name it in §1. Rotating the key at each `up`
  cuts the reuse of a stolen key but not the prompts sent while the attacker holds the port,
  and costs pasting OpenWebUI's key again after every `up`: offer it as an option, or as a
  rotation that `up` does after it found 8080 taken. The "taken by another program" error
  (`lifecycle.rs:244-247`) should point to the rotation procedure that `docs/security.md:62-63`
  already gives, rather than add a mechanism.
- **Evidence:** read: `api_key.rs:54-57` returns the existing key unchanged;
  `deploy/opencode.json:8-9` sends it to `http://127.0.0.1:8080/v1` whatever runs there;
  `lifecycle.rs:244-248` notices a taken port at the next `up` only.
- **Verification:** CONFIRMED — read: the same lines; finding 1's run shows that a listener on
  the client's address receives the `Authorization` header as sent. Not run against OpenCode
  itself, which would write OpenCode's state outside scratch.
- **User's verdict:** confirmed — the session, under the user's delegation of 2026-10-18: the documents and the error's pointer to the rotation, no rotation at each `up`, which would make the user paste the key into OpenWebUI after every `up`

### 3. `up` takes any 200 on /health at the listen address as the llama-swap it started

- **Axes:** security — raised by `review-security`
- **Action:** fix
- **Severity:** low
- **Where:** `src/application/lifecycle.rs:244-248`, `src/application/lifecycle.rs:289-290`, `src/application/lifecycle.rs:704-723`, `src/infrastructure/gateway_api.rs:74-77`, `src/infrastructure/gateway_process.rs:215-224`
- **Cause:** `up` checks the port with a bind that it releases at once (`TcpPorts::is_free`),
  starts llama-swap, then polls `GET /health` on `listen`, checking in each loop only whether
  its own child has exited. A process that binds the port between the check and llama-swap's
  own bind answers the poll; llama-swap fails to bind and exits, but `up` may already have
  reported `Started` and written the record.
- **Consequence:** a local account watching for a new `llama-swap` process in /proc can grab
  8080 during start-up. The user is told the gateway is up, and the clients send the key and
  prompts to the squatter, as in finding 2. The window is narrow but can be scripted.
- **Fix:** before declaring the gateway healthy, check that the socket listening on `listen`
  belongs to the started PID: match the listening inode of /proc/net/tcp or tcp6 against
  /proc/<pid>/fd. The verifier drops the reviewer's alternative, calling `started.exited()`
  again after the 200: it would run while llama-swap is still alive, and pass.
- **Evidence:** read: `lifecycle.rs:705-723` polls `started.exited()`, then
  `api.healthy(listen, …)`, and returns `Ok(())` on the first 200; `gateway_api.rs:74-77`
  counts any `status == 200`.
- **Verification:** CONFIRMED — test `verifier_squatter_answering_health_is_taken_for_the_gateway`
  in a copy, with `FakeApi::answering_after(0)` and `exits_after = Some(1)`: `up()` returns
  `Ok(Up::Started{..})`, and the record is written. On a taken listen port, llama-swap v263
  takes 0.07 to 0.09 s to exit (three runs), and the first health poll comes right after the
  spawn (`gateway_process.rs:215-224`, no delay), so the squatter answers before llama-swap is
  gone.
- **User's verdict:** confirmed — the session, under the user's delegation of 2026-10-18

### 5. GPU process names from other users reach the terminal with their control characters

- **Axes:** security — raised by `review-security`
- **Action:** fix
- **Severity:** low
- **Where:** `src/infrastructure/nvml.rs:88-91`, `src/infrastructure/nvml.rs:274-279`, `src/application/lifecycle.rs:850-864`, `src/application/doctor.rs:537-547`, `src/application/doctor.rs:646-650`, `src/presentation/doctor.rs:33-39`
- **Cause:** `read_gpu` takes each compute and graphics process name from NVML's
  `sys_process_name`, keeps what follows the last `/` (`base_name`), and stores it unchanged,
  other users' processes included. `named()` puts it into doctor's "Idle VRAM" and "GPU
  leftovers" details and into `up`'s and `down`'s errors, and presentation prints them
  unescaped. A process name is chosen by whoever runs the process, and can hold ESC and other
  control bytes.
- **Consequence:** another local account running a GPU process with an escape sequence in its
  name can rewrite what `ai-ctl doctor` and `status` show: hide a warning, or show a fake
  remedy line to copy into a shell. /proc has no `hidepid`, and /dev/nvidia0, nvidiactl and
  nvidia-uvm are mode 0666, so any account can put a process on the GPU; this machine has one
  login account, `seb`.
- **Fix:** escape at the source, in nvml.rs, so every output path is covered, without a third
  escaping routine: `src/infrastructure/inventory_file.rs:94` already escapes control
  characters as `\uXXXX`, and `Text::checked` in `gateway_config.rs` tests for a control
  character or U+2028/U+2029. Move that escaping into one shared helper, extended to
  U+2028/U+2029, and call it from `read_gpu`.
- **Evidence:** read: `nvml.rs:88-91` stores the name as NVML returns it;
  `lifecycle.rs:853-860` formats `process.name` into the message; `doctor.rs:648` and
  `:540-541` print it; no escaping on the output paths.
- **Verification:** CONFIRMED — NVML read, with no CUDA context, of a process started as
  `bash -c $'exec -a "/x/\e[2K\e[1Gok    Idle VRAM fake\e[0m" sleep 20'`:
  `nvmlSystemGetProcessName` returned argv[0] with its ESC bytes. Test
  `verifier_escape_in_a_process_name_reaches_the_detail` in a copy: the Idle VRAM detail is
  `"held by PID 5153 (\u{1b}[2K\u{1b}[1Gok    fake, 900 MiB)"`.
- **User's verdict:** confirmed — the session, under the user's delegation of 2026-10-18

### 7. A text flag value that is itself an option can bring in a reserved flag

- **Axes:** bugs — raised by `review-correctness`
- **Action:** fix
- **Severity:** low
- **Where:** `src/domain/inventory.rs:406-415`, `src/domain/gateway_config.rs:374`
- **Cause:** `check_flags` checks flag names only, and a `FlagValue::Text` is written as its own
  word after its flag; after a flag that takes no value, llama-server parses that text as the
  next option.
- **Consequence:** `[models.m.flags] jinja = "--api-key"` passes validation and generates
  `--jinja --api-key`. llama-server takes the next word as the backend's key, and a
  command-line `--api-key` overrides `LLAMA_API_KEY`. The reserved list is bypassed, but the
  author cannot choose the value: the next word is always a `--flag`, since flag names cannot
  start with `-`. The backend then refuses every request or fails to start; nothing is exposed.
- **Fix:** in `check_flags`, refuse a `FlagValue::Text` that starts with `-`, naming the flag.
  llama.cpp has no `--flag=value` form, so the refusal is the fix.
- **Evidence:** reproduced: a scratch test printed the configuration line `--jinja --api-key`.
- **Verification:** CONFIRMED — `tests/verify_scratch.rs::dashed_text_value` in a copy:
  `jinja = "--api-key"` then `no-mmap = true` generate `--jinja --api-key` then `--no-mmap`, so
  the key becomes the literal `--no-mmap`. llama.cpp b11700's parser matches each word exactly
  as an option name (`common/arg.cpp:825-870`), a value-taking option takes the next word
  whatever it is, and `--api-key` overrides `LLAMA_API_KEY` (arg.cpp:842, 3522); flag names
  cannot start with `-` (`inventory.rs:419`).
- **User's verdict:** confirmed — the session, under the user's delegation of 2026-10-18

### 8. `build --force` replaces the build the running gateway serves without the guard `prune` has

- **Axes:** bugs — raised by `review-correctness`
- **Action:** fix
- **Severity:** low
- **Where:** `src/application/build.rs:177-179`, `src/infrastructure/build_store.rs:239-251`, `src/application/bench.rs:308-312`, `src/application/bench.rs:328`
- **Cause:** `Builder::build` calls `store.publish(&staged, request.force)` under the store lock
  only, and `publish(replace)` renames the installed build aside and deletes it. Only `prune`
  takes the gateway lock and calls `refuse_served` (`build.rs:402-425`, `build.rs:438-461`).
  The path stays the same, so `is_current` and bench's configuration check still match.
- **Consequence:** with the gateway serving llama.cpp b11700 and its model loaded,
  `ai-ctl build llama.cpp --ref b11700 --force`, doctor's own rebuild remedy
  (`doctor.rs:378`), puts a new binary under the same path. The loaded llama-server keeps
  running the deleted old binary until its ttl, and `status` says the gateway is current.
  `bench run` passes its check and records the new build's commit, toolkit and flags for
  measurements of the old binary. A model swap that starts llama-server between `publish`'s
  two renames finds no binary. `docs/commands.md` says that the running gateway keeps its build
  until `down` and `up` for `build use` only.
- **Fix:** when `replace` would hit an installed build, take the gateway lock and reuse
  `refuse_served` (`build.rs:438`) with that build's binary; name `ai-ctl down` in the refusal.
- **Evidence:** read: `build.rs:177-179`, `build_store.rs:239-249`, `build.rs:401-425`
  (prune's guard), `bench.rs:308-312` and `bench.rs:329`.
- **Verification:** CONFIRMED — read: the same lines; `bench.rs:308-312` compares the
  configuration text only, and `bench.rs:328` records `current(...).installed()`, the new
  `build.json`.
- **User's verdict:** confirmed — the session, under the user's delegation of 2026-10-18

### 9. A current build whose directory cannot be read is called not installed, and the remedy `ai-ctl build <project>` is refused

- **Axes:** bugs, error handling — raised by `review-correctness`; the `build list` half by
  pass 2's verifier, as a lead
- **Action:** fix
- **Severity:** low
- **Where:** `src/application/build.rs:282-291`, `src/application/build.rs:133-145`, `src/application/doctor.rs:360`, `src/presentation/build.rs` (`build list`)
- **Cause:** `build::current` looks the link up among readable builds only, so an unreadable
  `b11700/` gives `Current::Missing`. `Builder::build` counts unreadable directories as taken
  and, since `current == Some(id)`, refuses the build as already current.
- **Consequence:** take `b11700/build.json` malformed, or of a later format (finding 10), with
  `current -> b11700`, b11700 being the newest release. Doctor fails with "the current build,
  b11700, is not installed", fix `ai-ctl build use llama.cpp <id>   # or: ai-ctl build
  llama.cpp`, and `up` says the same. The build is then refused as "already built and
  current". Only `--force`, which neither message offers for this state, recovers. Neither
  doctor nor `up` names the unreadable directory or its reason, and `build list` contradicts
  itself: "current build b11700 is not installed" just above its own "not a build: …/b11700
  (…)" for the same directory.
- **Fix:** tell an unreadable directory apart from a missing one, with a
  `Current::Unreadable(id, reason)`. In doctor, `up` and `build list`, name the reason and give
  `ai-ctl build <project> --ref <id> --force` as the remedy.
- **Evidence:** reproduced: a scratch state with a bad `build.json` and `current -> b11700`;
  doctor printed the line above, and `build list` named the directory as not a build. The
  refusal was read at `build.rs:133-145`.
- **Verification:** CONFIRMED — scratch state `xdg1`, a `build.json` without `project`,
  `current -> b11700` and a native backend: `ai-ctl doctor` prints the line above and exits 1;
  `ai-ctl build list` prints both "current build b11700 is not installed" and "not a build:
  …/b11700 (invalid …: malformed build metadata: missing field `project` …)".
  `build.rs::verify_unreadable_current_is_refused_as_current` in a copy: `current()` gives
  `Missing(b11538)`, and the build fails with "llama.cpp b11538 is already built and current;
  `--force` builds it again". The verifier's lead on `build list` has this finding's cause and
  fix, and was merged into it.
- **User's verdict:** confirmed — the session, under the user's delegation of 2026-10-18

### 11. The inventory accepts a non-finite float flag that the run store then cannot save, after the whole bench run

- **Axes:** bugs — raised by `review-correctness`
- **Action:** fix
- **Severity:** low
- **Where:** `src/infrastructure/inventory_file.rs:219`, `src/infrastructure/run_store.rs:533-541`, `src/application/bench.rs:256`
- **Cause:** `flag_values` accepts TOML `nan` and `inf` as `FlagValue::Float`; `flag_json`
  refuses them when `bench run` saves the run's setup.
- **Consequence:** `[models.m.flags] temp = nan` gives `--temp NaN` in the configuration. A
  `bench run` of that model measures for its full duration, prints the results, then fails
  with "the run is printed above, but cannot be saved: flag `temp` is not a finite number".
  Only a model's own flags are serialized: a NaN under `[defaults.llama-server]` reaches the
  `command` text only, and that run saves.
- **Fix:** refuse non-finite floats in `flag_values`, for defaults and models alike, naming
  the flag.
- **Evidence:** reproduced: a scratch test parsed `temp = nan` and generated `--temp NaN`; the
  save failure was read at `run_store.rs:537-539` and `bench.rs:256`.
- **Verification:** CONFIRMED — `tests/verify_scratch.rs::nan_float` in a copy: `temp = nan`
  and `top-p = inf` parse and generate `--temp NaN` and `--top-p inf`.
  `run_store.rs::verify_nan_flag_cannot_save`: `FsRunStore::save` fails with "flag `temp` is
  not a finite number". The save comes after the run (`bench.rs:256`), and
  `presentation/bench.rs:49-61` prints the run before the error.
- **User's verdict:** confirmed — the session, under the user's delegation of 2026-10-18

### 22. doctor's CDI check passes a specification that mounts a library no longer on disk, and every image backend then fails to start

- **Axes:** bugs — raised by the session, during finding 1's measurement in pass 3
- **Action:** fix
- **Severity:** low, kept by the verifier: only image backends are hit, and they are entries for
  comparison; the next boot or the refresh service clears it; podman fails loudly at start
- **Where:** `src/application/doctor.rs:658`, `src/infrastructure/host.rs:254-274`, `src/infrastructure/host.rs:399`, `src/domain/machine.rs:181`
- **Cause:** `cdi_spec_check` compares only the driver of the `libcuda.so.<version>` that the
  specification in use mounts (`parse_cdi_driver`) with the loaded driver: `CdiSpec` holds
  only its path and driver, and nothing reads the specification's other `hostPath` entries.
  `nvidia-cdi-refresh.path` writes the specification again on a change of the kernel modules
  or of `nvidia-ctk` only, so a dnf upgrade of another library it mounts leaves it stale until
  the next boot.
- **Consequence:** on this machine on 2026-10-18, booted at 10:55, the refresh wrote
  `/var/run/cdi/nvidia.yaml` with `/usr/lib64/libnvidia-egl-wayland.so.1.1.23`, and dnf
  installed egl-wayland 1.1.24 in its place at 11:25. `ai-ctl doctor` prints "ok CDI
  specification nvidia.com/gpu=all listed, from /var/run/cdi/nvidia.yaml for driver
  615.78.08" and exits 0, yet `podman run --device nvidia.com/gpu=all` of the official image
  fails in 0.15 s with "crun: cannot stat `/usr/lib64/libnvidia-egl-wayland.so.1.1.23`: No
  such file or directory": a model on an image backend cannot load under llama-swap, and a
  bench of an image fails, with doctor green.
- **Fix:** the CDI parser also collects the specification's `hostPath` entries, tested on a
  captured specification (the stale live one qualifies, listed in `tests/fixtures/README.md`);
  `cdi_spec_check` warns, naming the first missing path, with a remedy that runs `sudo
  systemctl start nvidia-cdi-refresh.service`. `CDI_REMEDY`'s comment, "at boot and at each
  driver change", says that a library upgrade is not covered. The parser's test and the
  comment are the verifier's additions to the session's fix.
- **Evidence:** reproduced in pass 3's session: that `podman run`, with ai-ctl's words and the
  container name `review-probe-llama`, five runs, exit 127 in 0.15 s; `nvidia-ctk cdi generate
  --output=<scratch>` gives a specification whose `hostPath` entries differ from the live one
  by that line only, 1.1.24 for 1.1.23, and the image starts with it (`podman
  --cdi-spec-dir`); `rpm -q --qf '%{INSTALLTIME:date}' egl-wayland` gives 2026-10-18 11:25:28
  and `uptime -s` 10:55:32; the path unit watches `modules.dep`, `modules.dep.bin` and
  `/usr/bin/nvidia-ctk`.
- **Verification:** CONFIRMED — `test -e` on every `hostPath` and `path` of the live
  specification: only `/usr/lib64/libnvidia-egl-wayland.so.1.1.23`, its lines 389-390, is
  missing, and only `.so.1.1.24` is on disk; a specification generated in scratch differs from
  the live one by that entry only. `systemctl cat nvidia-cdi-refresh.path` shows `PathChanged`
  on the three paths above; both units are enabled, and the service is a oneshot with
  `RemainAfterExit=no`, so `CDI_REMEDY`'s `enable --now` would run it again.
  `target/release/ai-ctl doctor` exits 0 with the line above. Not run: podman with the GPU
  device, forbidden to the verifier, so crun's failure rests on the session's run.
- **User's verdict:** confirmed — the session, under the user's delegation of 2026-10-18

### 10. A run or build of a later format is reported as malformed, not as an unsupported format

- **Axes:** error handling — raised by `review-correctness`; on-disk formats — raised by
  `review-structure` in pass 4, merged at step 7 (its number, 79, is unused)
- **Action:** fix
- **Severity:** low
- **Where:** `src/infrastructure/run_store.rs:366-372`, `src/infrastructure/build_store.rs:87-93`, `src/infrastructure/inventory_file.rs:129-135`, `src/infrastructure/inventory_file.rs:231-284`
- **Cause:** `from_json` deserialises into structs with `deny_unknown_fields`, and only then
  checks `format`. A later format that adds a field fails at the first step, with serde's
  "unknown field". The inventory's reader has the same cause: `parse` deserialises `FileDto`,
  `deny_unknown_fields`, before it compares `version`.
- **Consequence:** a run saved by a newer ai-ctl, `format: 4` with a new field, reads
  "malformed run: unknown field `notes`…". A `build.json` of format 2 with a new field shows
  as "not a build: … malformed build metadata: unknown field `lto`". The user is told to
  suspect a corrupt file, and editing or deleting it loses data. A build read this way also
  feeds finding 9. An inventory of version 2 that adds or renames a key fails as "TOML parse
  error … unknown field", and doctor's remedy says to fix it, or move it away and run `ai-ctl
  init` (finding 80).
- **Fix:** in each of the three readers, read the version key alone first, refuse an
  unsupported one with the existing message, then parse strictly. The two stores, JSON with the
  key `format`, share one helper, from a struct holding only `format` or a `serde_json::Value`;
  the inventory, TOML with the key `version`, takes its own pre-read of the same pattern, from a
  struct holding only `version` or a `toml::Value`.
- **Evidence:** reproduced in scratch: the run half through `bench list`, a command outside
  those the skill allows; the build half through `build list`.
- **Verification:** CONFIRMED — `ai-ctl build list` on a scratch state: format 2 with `lto`
  reads "malformed build metadata: unknown field `lto`…", format 2 alone "build metadata format
  2 is not supported (this ai-ctl reads format 1)". `run_store.rs::verify_future_format_with_new_field`
  in a copy: format 4 with `notes` gives "malformed run: unknown field `notes`, expected one of
  …", format 4 alone "run format 4 is not supported (this ai-ctl reads formats 1, 2 and 3)".
  That `bench list` shows this text rests on reading. Pass 4, the inventory's half: CONFIRMED
  — `ai-ctl doctor --json` on scratch inventories: `version = 2` with `new_key = 1` fails with
  "TOML parse error at line 3 … unknown field `new_key`", `version = 2` with `models_dir`
  renamed `models_root` with "unknown field `models_root`", and `version = 2` alone with
  "unsupported inventory version 2 (this ai-ctl reads version 1)"; all three with the remedy
  "fix …, or move it away and run `ai-ctl init --models-dir <dir>`". The fix as first written,
  JSON and `format` for the two stores, did not cover the inventory; widened above.
- **User's verdict:** confirmed — the session, under the user's delegation of 2026-10-18

### 12. `up` reports "Gateway up already" for a recorded gateway that never answered

- **Axes:** error handling — raised by `review-correctness`
- **Action:** fix
- **Severity:** low
- **Where:** `src/application/lifecycle.rs:230-241`, `src/application/lifecycle.rs:752`
- **Cause:** when the record's process runs on the current configuration, `up` returns
  `Up::Running` without a health check. A start whose health wait failed and whose stop failed
  leaves the record, by design ("it stays recorded for `ai-ctl down`"); so does an `up`
  interrupted during its 30 s wait, since llama-swap runs in its own process group.
- **Consequence:** after `up` fails with "llama-swap did not answer on 127.0.0.1:8080 within
  30 s; could not stop it: …; it stays recorded", a second `ai-ctl up` prints "Gateway up
  already on http://127.0.0.1:8080 (PID …)" and exits 0, though nothing answers there.
  `docs/commands.md` says only that `up` does nothing on an unchanged running gateway.
- **Fix:** in the `Running` branch, call `api.healthy(listen, CHECK)`; when it fails, give an
  error naming `ai-ctl down` and the log.
- **Evidence:** read: `lifecycle.rs:230-242` returns `Up::Running` after `is_current` only;
  `lifecycle.rs:748-755` keeps the record.
- **Verification:** CONFIRMED — `lifecycle.rs::verify_silent_recorded_gateway_is_up_already` in
  a copy, with a live record, a current configuration and `FakeApi::silent()`: `up` returns
  `Up::Running { pid: 1234, … }` after 0 health calls, and `presentation/lifecycle.rs:137-138`
  prints "Gateway up already on …".
- **User's verdict:** confirmed — the session, under the user's delegation of 2026-10-18

### 14. A read timeout is reported as "Resource temporarily unavailable (os error 11)"

- **Axes:** error handling — raised by `review-correctness`
- **Action:** fix
- **Severity:** low
- **Where:** `src/infrastructure/http.rs:59-60`, `src/infrastructure/http.rs:95-97`
- **Cause:** `send` sets a socket read timeout. On Linux its expiry is `WouldBlock` (EAGAIN),
  which `request` passes on with the request as its only context.
- **Consequence:** a gateway or backend that accepts the connection but does not answer in
  time is reported as "cannot list the gateway's models (GET http://127.0.0.1:46047/running:
  Resource temporarily unavailable (os error 11))", and `up`'s health wait ends on "last
  error: … Resource temporarily unavailable". It reads as resource exhaustion, not as no answer
  within N s.
- **Fix:** map `WouldBlock` and `TimedOut` to "no answer within N s" once in `http.rs`, for
  `request` and the stream readers.
- **Evidence:** reproduced: finding 13's scratch state; `ai-ctl status` prints that warning.
- **Verification:** CONFIRMED — the same run as finding 13's: `status` warned with that text.
  The fake for `up`'s health wait already carries it (`fakes.rs:777-779`).
- **User's verdict:** confirmed — the session, under the user's delegation of 2026-10-18

### 16. On a failed request, the completed scenarios are never printed; when the save also fails, they are lost

- **Axes:** error handling — raised by `review-correctness`
- **Action:** fix
- **Severity:** low
- **Where:** `src/application/bench.rs:262-275`, `src/presentation/bench.rs:49-54`
- **Cause:** the partial path returns `Err` from `Bench::run` in both of its branches, so
  presentation never renders the run. When `self.runs.save` also fails, the error carries only
  the scenario names.
- **Consequence:** with a full disk, a late request failure gives "…; the run's completed
  scenarios, depth and prefill, cannot be saved: …": the measured results are neither saved
  nor printed, which the first review's finding 12 fixed for the success path. When the save
  succeeds, the error names the saved run, and only `bench compare <id> <id>` shows it, beside
  itself.
- **Fix:** return the partial run with its failure, for example
  `Recorded { run, saved, failure: Option<Error> }`; presentation renders it, then reports the
  error and exits non-zero.
- **Evidence:** read: `bench.rs:262-275`; `presentation/bench.rs:49-54` returns on `?` before
  `render_run`.
- **Verification:** CONFIRMED — read: the same lines. Correction to the reviewer, who had no
  command show a single saved run: `cli.rs:158` needs two ids or more, but neither it,
  `bench::load` (`bench.rs:440`) nor `render_compare` (`compare.rs:172`) refuses a duplicate,
  so `bench compare <id> <id>` shows the run beside itself; that rests on reading.
- **User's verdict:** confirmed — the session, under the user's delegation of 2026-10-18

### 20. `LocalGateway::write` leaves `.<name>.tmp` after a failed write or rename

- **Axes:** error handling — raised by `review-correctness`
- **Action:** fix
- **Severity:** low, given by the verifier
- **Where:** `src/infrastructure/gateway_process.rs:60-67`
- **Cause:** `fs::write(&temporary, text).and_then(|()| fs::rename(..))` returns on an error
  without removing the temporary file.
- **Consequence:** on a full disk, `up` fails with "cannot write …/llama-swap.yaml" and leaves
  `.llama-swap.yaml.tmp`, possibly truncated, in the state directory. The next write truncates
  it, and the configuration carries the key only as a `${env.…}` macro, so nothing secret or
  misread is left.
- **Fix:** add `files::replace_file`, a temporary named by `temporary_for`, a rename and a
  removal on error, and use it here. No helper of the crate fits: `files::write_new` refuses an
  existing path, and `replace_link` handles links only.
- **Evidence:** read: `gateway_process.rs:60-67`.
- **Verification:** CONFIRMED — `gateway_process.rs::verify_failed_write_leaves_temporary` in
  a copy: with a directory in place of `llama-swap.yaml`, `write` fails with "cannot write
  …/gateway/llama-swap.yaml: Is a directory (os error 21)", and `.llama-swap.yaml.tmp`
  remains. Correction to the reviewer's fix, which named a shared helper: none does it yet.
- **User's verdict:** confirmed — the session, under the user's delegation of 2026-10-18

### 21. When `prune` fails mid-way, its error does not say which builds it already removed

- **Axes:** error handling — raised by `review-correctness`
- **Action:** fix
- **Severity:** low, given by the verifier
- **Where:** `src/application/build.rs:427-433`, `src/application/build.rs:415-421`
- **Cause:** `prune` removes the doomed builds one by one and returns on the first failure with
  "cannot remove {project} {id}". The `Pruned` list is returned, and so rendered, on success
  only.
- **Consequence:** `build prune --keep 1` that removes b1 then fails on b2 prints only "cannot
  remove llama.cpp b2: …"; the user learns of b1's removal from `build list` only.
- **Fix:** put the builds already removed in the error's context ("…; removed before it: b1"),
  or return the partial `Pruned` with the error and render it first.
- **Evidence:** read: `build.rs:427-433`.
- **Verification:** CONFIRMED — read: `build.rs:427-433`, and `build.rs:415-421`, which builds
  the list.
- **User's verdict:** confirmed — the session, under the user's delegation of 2026-10-18

### 80. doctor's Inventory check tells the user to fix or replace an inventory whose only fault is a newer version

- **Axes:** error handling — a lead of pass 4's second `review-verifier`
- **Action:** fix
- **Severity:** —
- **Where:** `src/application/doctor.rs:421-427`, `src/infrastructure/inventory_file.rs:131-135`
- **Cause:** one `Err(err)` arm of the Inventory check (`doctor.rs:421-427`) gives the remedy
  "fix …, or move it away and run `ai-ctl init --models-dir <dir>`" for every load error. The
  version's error is an untyped `ensure!` (`inventory_file.rs:131-135`), so nothing can tell it
  apart today.
- **Consequence:** on "unsupported inventory version 2 (this ai-ctl reads version 1)", the file
  is valid and the remedy is a newer ai-ctl, yet the user is told to fix or replace it. Finding
  10's fix, which 79 extends to the inventory, makes that message the one every newer inventory
  gets.
- **Fix:** test first. In `src/domain/inventory.rs`, beside `InvalidInventory`, a `thiserror`
  type `UnsupportedVersion { found: u32 }` that keeps today's text, returned by
  `inventory_file::parse` in place of the `ensure!`. In doctor's `Err` arm,
  `err.downcast_ref::<UnsupportedVersion>()`, as `config_check` does with `ConfigError`
  (`doctor.rs:205`), gives a remedy of its own: "written for a newer ai-ctl: run that ai-ctl, or
  move the file away and run `ai-ctl init --models-dir <dir>`". Two tests: a doctor test whose
  inventory store returns the typed error, which `FakeInventory::load` needs a way to return
  (it builds an `anyhow!` from a `String`, `fakes.rs:380-382`); an `inventory_file` test that
  version 2's error downcasts. With or after finding 10: until its pre-read of `version`, a
  version-2 file that adds a key fails on "unknown field" and never reaches this arm.
- **Evidence:** reported by pass 4's second verifier from its runs of `ai-ctl doctor --json` on
  scratch inventories for finding 79.
- **Verification:** CONFIRMED — `ai-ctl doctor --json`, a release build of `docs(v2-review-fixes): record pass 5's performance review of the closing review` of 2026-10-18 in the
  verifier's clone, on a scratch XDG copy of this machine's state with `version = 2`: exit 1,
  the Inventory check's detail "invalid inventory …/inventory.toml: unsupported inventory
  version 2 (this ai-ctl reads version 1)" and its remedy "fix …/inventory.toml, or move it
  away and run `ai-ctl init --models-dir <dir>`". The fix is the verifier's.
- **User's verdict:** confirmed — the session, under the user's delegation of 2026-10-18

### 82. `Configurator::prepare` reads the GPU before it checks that an inventory exists

- **Axes:** error handling — a lead of pass 4's second `review-verifier`
- **Action:** fix
- **Severity:** —
- **Where:** `src/application/gateway_config.rs:43-53`, `src/infrastructure/nvml.rs:32-33`
- **Cause:** `prepare` is `self.prepare_on(&self.gpu.gpu())`, and `prepare_on` loads the
  inventory only afterwards (`gateway_config.rs:48-53`).
- **Consequence:** `gateway config` without an inventory still loads NVML, about 24 ms,
  against `nvml.rs:32-33`, "a command that never reads it never loads the library"; where NVML
  hangs, the "no inventory" error is never reached, which cannot be produced here. The error
  text is right either way: with no inventory and a failed GPU, `prepare_on` reports the
  inventory first. `prepare` is reached only by `gateway config` (`presentation/gateway.rs:30`)
  and `Bench::setup` (`bench.rs:282`); `up` and doctor take their own reading and call
  `prepare_on` (`lifecycle.rs:493`, `doctor.rs:196`).
- **Fix:** test first: with `ScriptedGpu` and `FakeInventory(Ok(None))`, `prepare()` fails
  with "no inventory at …" and the GPU was not read. Split `prepare_on`'s body into a private
  `resolve()`, which loads the inventory and resolves the runtimes, and `size(&inventory,
  &runtimes, gpu)`: `prepare` calls `resolve()`, then reads the GPU, then `size`; `prepare_on`
  calls `resolve()`, then `size`. After finding 55's fix Bench leaves `prepare`, so this fix
  changes only `gateway config`, and `resolve()` is where 55's `Prepared` takes its inventory
  and builds: the two are planned together.
- **Evidence:** reported by pass 4's second verifier from its `LD_DEBUG=libs` runs of
  `tests/cli.rs` for finding 72.
- **Verification:** CONFIRMED — the probe test in a copy fails on "the GPU was read before the
  inventory, left: 1 right: 0"; with the split applied in a copy, `cargo test` is green, 636 in
  the library with the probe, 25 in `tests/cli.rs` and 3 in `tests/gateway_config.rs`, and
  clippy is clean. NVML's initialisation measured 24.6 and 23.9 ms. The fix is the verifier's.
- **User's verdict:** confirmed — the session, under the user's delegation of 2026-10-18

### 13. `status` holds the start/stop lock for its whole observation, and a concurrent `up` or `down` is told another ai-ctl is starting or stopping

- **Axes:** interruption and concurrency — raised by `review-correctness`
- **Action:** fix
- **Severity:** low
- **Where:** `src/application/lifecycle.rs:347`, `src/application/lifecycle.rs:358`, `src/infrastructure/files.rs:54`, `src/infrastructure/gateway_process.rs:139-141`
- **Cause:** `status` takes `gateway.lock()`, a try-lock, for `running()` and `observe()`, and
  `observe()` makes HTTP calls with a 5 s timeout and runs `podman ps` with up to 15 s. The
  busy message is fixed: "another ai-ctl is starting or stopping the gateway".
- **Consequence:** anything that polls `ai-ctl status`, such as a status-bar script or a wait
  loop (exit code 3 is documented for scripts), makes a concurrent `up` or `down` fail at once
  with exit 1, under a message that names an activity no ai-ctl is doing.
  `docs/commands.md:88-95` lists `status` as a holder and promises that the refusal says what
  the other ai-ctl is doing. Each `status` whose API call times out holds the lock 5 s.
- **Fix:** let `status` observe without the lock, as `inspect` does (`lifecycle.rs:358`), and
  take the lock only to forget a stale record. Naming the holder in the lock file would fix
  only the wording.
- **Evidence:** reproduced: a scratch record pointing at a `sleep`, with a loopback listener
  that never answers; the first `status` took 5.3 s, and a second one started 1 s later failed
  with the busy message, exit 1.
- **Verification:** CONFIRMED — `scripts/xdg2.py` in scratch: the record points at
  `/usr/bin/sleep 60`, with a listener on 127.0.0.1:46047 that never accepts. The first
  `ai-ctl status` took 5.26 s and exited 0; a second one started 1.0 s later failed in 0.00 s
  with "error: another ai-ctl is starting or stopping the gateway (…/gateway/.lock is
  locked)", exit 1. `up` and `down` take the same lock (`lifecycle.rs:225`,
  `gateway_process.rs:139-141`).
- **User's verdict:** confirmed — the session, under the user's delegation of 2026-10-18

### 15. Ctrl-C or a kill during `bench run` discards every scenario already measured

- **Axes:** interruption and concurrency — raised by `review-correctness`
- **Action:** fix
- **Severity:** low
- **Where:** `src/application/bench.rs:221-275`
- **Cause:** the run lives in memory until the last scenario, and only a failed request
  triggers the partial save. SIGINT ends the process with nothing saved.
- **Consequence:** stopping a run 20 minutes in, during its third scenario, loses the depth and
  prefill scenarios it completed; only the per-request lines on stderr remain.
- **Fix:** save the run after each completed scenario, replacing a partial file. A signal flag
  would need a dependency or unsafe code for the same result.
- **Evidence:** read: `bench.rs:218-275` saves only at the end or in the `failure` branch; the
  crate handles no signal.
- **Verification:** CONFIRMED — read: `bench.rs:221-275`; `grep -rn
  "signal\|SIGINT\|ctrlc\|sigaction" src Cargo.toml` finds only the gateway's `Signal`
  (`gateway_process.rs:234`) and its fakes.
- **User's verdict:** confirmed — the session, under the user's delegation of 2026-10-18

### 17. A kill between `publish`'s two renames leaves the only copies of the build hidden, and the next build deletes them

- **Axes:** interruption and concurrency — raised by `review-correctness`
- **Action:** fix
- **Severity:** low
- **Where:** `src/infrastructure/build_store.rs:246-251`, `src/infrastructure/build_store.rs:140-142`, `src/infrastructure/build_store.rs:200`, `src/infrastructure/files.rs:133-152`
- **Cause:** `publish(replace)` renames the installed build to `.replaced-<id>-<pid>`, then the
  staged one into place; the put-back runs on an error only. `stage` begins with
  `remove_leftovers`, which removes every hidden entry but `.lock`.
- **Consequence:** a kill or power loss between the two renames of `build --force` leaves no
  `b11700/`, with `current -> b11700` dangling; doctor and `up` say the build is not installed.
  The next `ai-ctl build`, once it has compiled, deletes `.replaced-b11700-*` and
  `.staging-b11700-*`, the only complete copies, before staging its own. When that build is of
  another ref, b11700 is lost for good. The window is two renames wide.
- **Fix:** before sweeping in `remove_leftovers`, rename back a `.replaced-<id>-*` whose `<id>`
  is missing. `renameat2(RENAME_EXCHANGE)` would need `libc` or unsafe code.
- **Evidence:** read: `build_store.rs:240-249`, `build_store.rs:140-142` and
  `build_store.rs:200`.
- **Verification:** CONFIRMED — read: the same lines and `files.rs:133-152`. Correction to the
  reviewer, who had the sweep come before compiling again: `stage`, which sweeps, runs after
  `compile` (`build.rs:160-175`), so the sweep comes after the new compile, just before
  staging.
- **User's verdict:** confirmed — the session, under the user's delegation of 2026-10-18

### 18. The gateway is spawned before its record is written; a kill between the two leaves an llama-swap ai-ctl cannot see

- **Axes:** interruption and concurrency — raised by `review-correctness`
- **Action:** fix
- **Severity:** low
- **Where:** `src/infrastructure/gateway_process.rs:213-230`
- **Cause:** `start` spawns llama-swap in its own process group (`process_group(0)`), so it
  outlives ai-ctl, and only then serializes and writes `gateway.json`. The child is killed on
  a write error, not on a signal.
- **Consequence:** SIGINT or SIGTERM to ai-ctl in that window, a serialization, a write and a
  rename, leaves a running llama-swap with no record. `down` says "No gateway running"
  (`lifecycle.rs:515-518`), and `up` says "127.0.0.1:8080 is taken by another program; stop
  it…" (`lifecycle.rs:245-249`) of ai-ctl's own gateway; the user must find its PID by hand.
- **Fix:** block SIGINT and SIGTERM across the spawn and the record write, with `libc`'s
  `sigprocmask`, a new dependency; a record written before the PID is known is the awkward
  alternative.
- **Evidence:** read: `gateway_process.rs:199-230`.
- **Verification:** CONFIRMED — read: the same lines, `lifecycle.rs:515-518` and
  `lifecycle.rs:245-249`. The window is narrow.
- **User's verdict:** confirmed — the user's, on 2026-10-18: the window kept and documented, no `libc` and no unsafe code; finding 24's test stays

### 19. A kill leaves `write_new`'s temporary file behind, and no store sweeps it

- **Axes:** interruption and concurrency — raised by `review-correctness`
- **Action:** fix
- **Severity:** low
- **Where:** `src/infrastructure/files.rs:78-83`, `src/infrastructure/run_store.rs:84-89`
- **Cause:** `write_new` writes and syncs `.<name>.<pid>.tmp`, links it, then removes it on
  every path; a kill between the write and the removal, a window that holds the fsync, leaves
  it. No store sweeps hidden files: the run store skips them, and neither the configuration
  directory nor the key's directory is swept.
- **Consequence:** a kill during `bench run`'s save, `init` or the first `up` leaves
  `.<run>.json.<pid>.tmp` in `bench/`, or `.inventory.toml.<pid>.tmp` or `.api-key.<pid>.tmp`
  beside their files, for good. The key's leftover is a hard link to the key's own inode, mode
  0600: clutter, neither wrong data nor a new exposure.
- **Fix:** sweep the `.<name>.<pid>.tmp` files whose PID no longer runs, beside the
  `write_new` call sites, or document them.
- **Evidence:** read: `files.rs:78-89`; `run_store.rs:76-94` skips hidden names without
  removing them.
- **Verification:** CONFIRMED — read. Correction to the reviewer's cause, which had the
  removal on an error path only: `let _ = fs::remove_file(&temporary)` runs on every path,
  success included.
- **User's verdict:** confirmed — the session, under the user's delegation of 2026-10-18

### 83. The toolkit-package probe makes rpm read every package header (2,637 here) to find about four names, about a third of `doctor`'s time

- **Axes:** performance — raised by `review-performance`
- **Action:** fix
- **Severity:** —
- **Where:** `src/infrastructure/host.rs:223-242`, `src/application/toolchain.rs:32`, `src/application/build.rs:108`
- **Cause:** `toolkit_packages` runs `rpm -qa --nodigest --nosignature --qf '%{NAME}\n'
  'cuda-toolkit*'`. A glob given to `-qa` filters after rpm has loaded every header of the
  database, without the Name index. Batch 30 removed the digest checks; the full scan stayed.
- **Consequence:** 87 to 108 ms, about as long as listing every package unfiltered, against 7 ms
  for a query by exact names: about a third of `doctor`, 259 to 282 ms on this machine.
  `build`'s gate pays it again (`build.rs:108`), small next to a compile.
- **Fix:** query exact names, which go through the index: `rpm -q --nodigest --nosignature --qf
  '%{NAME}\n'` with `cuda-toolkit`, `cuda-toolkit-<M>` and `cuda-toolkit-<M>-<m>` for the
  driver's CUDA version, the found nvcc's version and each `/usr/local/cuda-<M>.<m>` directory.
  A missing name prints `package <name> is not installed` on stdout and adds 1 to the exit
  status: the parser keeps only the lines equal to a name it asked for, the verifier's
  refinement, so it does not depend on rpm's wording, and does not take a nonzero exit as a
  failure. A new fixture captured from `rpm -q` with a missing name, per the fixture rule.
  Limit: a meta-package installed without its `/usr/local/cuda-<M>.<m>` directory goes unseen.
  The names come from finding 53's `toolkit_package`, which lands first.
- **Evidence:** reproduced by the reviewer: the glob query 87/88/91 ms, unfiltered 89/90/90 ms,
  exact names 7/7/7 ms; `doctor` through logging wrappers, `rpm … 91ms` in a 292 ms run;
  `SystemHost::toolkit_packages` in a scratch crate, 107.9 and 105.7 ms; `rpm -q` of a missing
  name prints the not-installed line on stdout, rc 3.
- **Verification:** CONFIRMED — under `LC_ALL=C`: the glob 96/97/96 ms, unfiltered 98/97/97 ms,
  exact names 7/7/6 ms, `rpm -qa | wc -l` 2,637. `doctor` on a scratch XDG copy, 259 to 282 ms
  (n=8); with a PATH wrapper standing for the fix, 173 to 177 ms (n=5), with the same "Toolkit
  packages ok cuda-toolkit-13-4". Four missing names give rc 4 and one line each on stdout; in
  French the line reads "le paquet … n'est pas installé", and `command::run` sets `LC_ALL=C`
  (`command.rs:97`). This machine has `/usr/local/cuda-13.2` without `cuda-toolkit-13-2`, which
  the exact query reports rightly.
- **User's verdict:** confirmed — the session, under the user's delegation of 2026-10-18

### 84. doctor runs its about 15 independent probe programs one after another, so its time is their sum rather than the longest

- **Axes:** performance — raised by `review-performance`
- **Action:** fix
- **Severity:** —
- **Where:** `src/application/doctor.rs:71-107`, `src/application/toolchain.rs:29-47`
- **Cause:** `Doctor::run` and `toolchain::inspect` call each probe in turn: two dnf calls, rpm,
  nvcc, two g++, cmake, git, podman, nvidia-ctk twice, busctl, the current build's
  `llama-server --version`, and the first NVML read. The probes do not need each other's
  results, apart from dnf's repolist and repoquery, nvidia-ctk's version and cdi list, the
  build check's link lookup, and the GPU reading, which `toolchain::inspect`, the container
  checks and the gateway inspection take (`doctor.rs:73`, `:93-99`), the verifier's correction.
- **Consequence:** 224 to 258 ms run one after the other. The fix's four groups run at once take
  154 to 156 ms, the toolchain group alone 150 to 161 ms; with finding 83's fix, 72 to 73 ms:
  about 75 ms saved with or without 83, and a floor of NVML's 24 ms plus about 70 ms. Low: an
  interactive wait only. The change is not free: the boxed ports used across threads need
  `Sync`, and the fakes' 24 `Cell` and `RefCell` fields (10 and 14, one of them the thread-local
  clock) become `Mutex` or atomics; the real adapters need nothing, NVML being behind a
  `OnceLock` (`nvml.rs:39`).
- **Fix:** run the independent groups on `std::thread::scope` threads: the toolchain probes,
  with the dnf pair and rpm on threads of their own unless finding 83 lands first; the container
  checks; the power checks; the build checks. Take the GPU reading first, or start the build and
  power groups, which need none, beside it. `+ Sync` goes on the boxed ports concerned, on the
  read-only inspection struct of finding 51's fix, which lands first. The fake clock is
  thread-local (`fakes.rs:37-52`): a fake that calls `advance` on a worker thread moves that
  thread's clock, harmless for Doctor, which never reads the clock (finding 51), and to be
  stated in the code.
- **Evidence:** reproduced by the reviewer: a scratch `probes.sh` running the commands doctor
  runs, as logged by PATH wrappers, one after the other then with `&` and `wait`: 230 to 258 ms
  against 102 to 104 ms, 61 to 62 ms without rpm. Per probe: the dnf pair 64 ms, rpm 106 to
  108 ms, podman 13 to 16 ms, cmake 6 to 8 ms, `llama-server --version` 55 to 58 ms, which
  exits during argument parsing before any backend loads (llama.cpp's `common/arg.cpp:1454-1459`),
  the others 1 to 5 ms.
- **Verification:** CONFIRMED, with corrections — the reviewer's `probes.sh` re-run: 224 to
  231 ms one after the other, 90 to 93 ms at once, 57 to 61 ms at once without rpm. Its 102 to
  104 ms ran dnf and rpm as separate jobs, where the fix puts them in one toolchain thread: timed
  as the fix's four groups (`fns2.sh`), 154 to 156 ms, and 72 to 73 ms with 83's fix. The GPU
  reading is not independent; the fakes hold 24 such fields, not 28. 51's `pause` and `now` are
  `fn` pointers, already `Sync`, so 51 does not block this fix, but both rewrite `Doctor::run`.
  A threaded Doctor was not built: the group timings are shell measurements.
- **User's verdict:** rejected — the session, under the user's delegation of 2026-10-18: about 75 ms on an interactive diagnostic does not pay the threads, the `Sync` bounds and the 24 fake fields rewritten, where CLAUDE.md puts robust and predictable behaviour first

### 85. `turn_extra` counts the agent's bare prompt again, though `Prompts::load` has just stored that count in `own`

- **Axes:** performance — raised by `review-performance`
- **Action:** fix
- **Severity:** —
- **Where:** `src/application/bench.rs:784`, `src/application/bench.rs:831-836`
- **Cause:** `turn_extra` computes `self.count(&next)? - self.count(&opening)?` with `opening =
  prompt::agent("")`, the messages `Prompts::load` counted into `self.own` at `bench.rs:784`;
  the template and the tokenizer are deterministic. Batch 35 counted the bare prompt once and
  left this second count.
- **Consequence:** each run of the session scenario, the default, makes one count more: two
  HTTP requests, apply-template then tokenize, through llama-swap's `/upstream`, before the GPU
  is watched. Low: a few milliseconds; not timed live, which holds the GPU.
- **Fix:** `let extra = self.count(&next)?.saturating_sub(self.own);`, with the verifier's probe
  test, which pins the number of counts of a session run, as its test: no test pins it today.
- **Evidence:** read: `bench.rs:784` `prompts.own = prompts.count(&prompt::agent(""))?`, and
  `:832-835` `let opening = prompt::agent(""); … self.count(&next)?.saturating_sub(self.count(&opening)?)`;
  `phase-3-performance-report.md:270-271`, "28 counts, 56 requests, against 9 counts, 18
  requests".
- **Verification:** CONFIRMED — a probe test in two copies, `World::new()` running
  `Scenario::Session` and counting the fake server's calls: 4 counts and 4 templates on the
  code, 3 and 3 with the fix. With the fix in a copy, `cargo test` passes, 635 in the library,
  25 and 3, and clippy is clean.
- **User's verdict:** confirmed — the session, under the user's delegation of 2026-10-18

### 87. `down` waits a full 100 ms poll for a llama-swap that is gone about 1 ms after SIGTERM

- **Axes:** performance — raised by `review-performance`
- **Action:** fix
- **Severity:** —
- **Where:** `src/application/lifecycle.rs:767-786`, `src/application/lifecycle.rs:26`, `src/application/lifecycle.rs:742-745`
- **Cause:** `stop` sends the signal, checks `gone()` once at once, then pauses `POLL`, 100 ms,
  before checking again. llama-swap is still there at the first check and exits within 1 to
  2 ms; its models were already stopped by `unload_all`.
- **Consequence:** about 100 ms added to every `down`, and to an `up` that stops a gateway that
  never answered (`lifecycle.rs:742-745`). Low.
- **Fix:** start the polling short and let it grow to `POLL`, for example 5, 10, 20, 50, then
  100 ms, each pause clamped to the time left as `wait_until_healthy` does (`lifecycle.rs:725`),
  and bound it by a deadline on the injected `now`, as batch 29 did for the other waits: 40 s
  after SIGTERM and 5 s after SIGKILL, in place of `STOP_POLLS` and `KILL_POLLS`, which assume a
  fixed pause. Finding 27's test lands first, as the guard of the 40 s grace; it holds under a
  deadline, since the fake `pause` advances the clock by each pause. The growing schedule does
  not divide 40 s evenly, so the clamp runs: per finding 28, a test whose pauses sum evenly
  would not reach it.
- **Evidence:** reproduced by the reviewer: llama-swap v263 alone on a free loopback port, one
  model whose command is `touch`; `kill -s TERM <pid>` took 0.4 to 1.6 ms, the process was
  still alive at the first check in 6 of 6 runs, and gone 1.0 to 2.2 ms after the kill started.
- **Verification:** CONFIRMED — signalled by `kill -s TERM`, as the adapter does
  (`gateway_process.rs:241`): alive at the first check in 11 of 12 runs, gone 0.68 to 1.32 ms
  after the signal started; by `os.kill`, alive 8 of 8, gone 0.28 to 0.78 ms after. `down`
  waits for `liveness != Running` (`lifecycle.rs:322-326`), a readlink of `/proc/<pid>/exe`
  (`gateway_process.rs:95-113`).
- **User's verdict:** confirmed — the session, under the user's delegation of 2026-10-18

### 88. A build of a release that is already installed is refused only after a network fetch and a checkout

- **Axes:** performance — raised by `review-performance`
- **Action:** fix
- **Severity:** —
- **Where:** `src/application/build.rs:116-149`
- **Cause:** `Builder::build` fetches, resolves, checks out (`checkout --force`, `clean -ffdxq`,
  `tag --points-at`) and lists the builds before it tests whether the id is taken. For `--ref
  <tag>` naming an installed release, the answer is known before any of that.
- **Consequence:** phase 3 measured `build llama.cpp --ref b11700`, the current build, refused
  in 1.46 s end to end, and a fetch in a copy of the sources clone at 1.20 s, separately: the
  fetch's share is an inference, the verifier's correction. The refusal also needs the network
  and rewrites the sources tree; it comes after the gate (`build.rs:105-114`), about 0.2 s.
  Nothing changes for `latest`, which needs the fetch. Low.
- **Fix:** without `--force`, when `request.reference` is a release tag by the recipe's tags
  (`TagPattern::release`) and `store.builds()` holds that id, refuse with the same two messages
  before `Fetching`; fetch only to build. A branch name or a short hash naming an installed id
  still goes through the checkout, where the id is decided (`build.rs:123-126`: the newest
  release tag at the commit, else the short commit). Caveat: a commit carrying a newer release
  tag than the one named would build under that newer id, rare in llama.cpp.
- **Evidence:** read: `build.rs:116-117` (`events(Event::Fetching); self.sources.fetch(recipe)?`)
  comes before the `taken` test at `:133-149`; `phase-3-performance-report.md:204-210`.
- **Verification:** CONFIRMED — a probe test in a copy: with `installed("b11538")`,
  `world.build(Some("b11538"), false)` fails with "llama.cpp b11538 is already built; `ai-ctl
  build use llama.cpp b11538` makes it current…", and the fake sources record `["fetch",
  "checkout b11538"]` and the events `[Fetching, CheckedOut { … }]`. `build` was not run, by the
  review's rules.
- **User's verdict:** confirmed — the session, under the user's delegation of 2026-10-18

### 62. `build list` says nothing about a dangling `current` link when the project has no readable build

- **Axes:** the CLI's contract — raised by `review-structure`
- **Action:** fix
- **Severity:** —
- **Where:** `src/presentation/build.rs:162-168`, `src/presentation/build.rs:194-198`
- **Rule:** `docs/commands.md:190-191`
- **Cause:** `render_list` prints "no build yet" and `continue`s whenever `builds` and
  `unreadable` are both empty; the line "current build {id} is not installed" comes after that
  `continue`, so it never prints for a project with no readable build.
- **Consequence:** a `current -> b99999` whose target is gone, with no other build, gives only
  "llama.cpp: no build yet; `ai-ctl build llama.cpp` makes one", exit 0, while doctor fails the
  same state with "the current build, b99999, is not installed". No test covers it:
  `a_dangling_current_link_is_shown` always has one build.
- **Fix:** with finding 63's second way, the not-installed check moves into
  `application::build::list` (`build.rs:323-342`), which runs before rendering, and out of the
  renderer: that fixes both. Otherwise check `listing.current` before the "no build yet"
  shortcut. A test with `builds: vec![]` and `current: Some(..)`. Finding 9's
  `Current::Unreadable` touches the same lines: the three fixes are planned together.
- **Evidence:** reproduced by the reviewer on a scratch XDG state.
- **Verification:** CONFIRMED — on a scratch state with only `builds/llama.cpp/current ->
  b99999`, `build list` prints "no build yet", stderr empty, exit 0; with `b11000/build.json`
  added it prints "  current build b99999 is not installed"; `doctor --json` on the first state
  fails with "the current build, b99999, is not installed", exit 1. A cause apart from finding
  9, whose link names a directory that cannot be read, so that `unreadable` is not empty and
  the `continue` is not taken. Apart from 63 only if 63 is fixed in `docs/commands.md`.
- **User's verdict:** confirmed — the session, under the user's delegation of 2026-10-18

### 63. A `current` link that names a missing build is a line on stdout, not the warning commands.md describes

- **Axes:** the CLI's contract — raised by `review-structure`
- **Action:** fix
- **Severity:** —
- **Where:** `src/presentation/build.rs:194-198`, `src/application/build.rs:369-379`
- **Rule:** `docs/commands.md:190-191`
- **Cause:** only a link whose target is not a build name (`../x/y`) goes through
  `current_or_none`'s `warn`, on stderr. A link to a well-formed id that is not installed
  becomes `Listing.current`, which `render_list` prints on stdout as "  current build X is not
  installed".
- **Consequence:** commands.md promises "reported as a warning"; a script reading the list's
  stdout gets the line among the builds, and one watching stderr for warnings misses it.
- **Fix:** emit the not-installed case through `warn` from `application::build::list`, which
  also fixes finding 62; or reword `docs/commands.md:190-191` ("…listed as not installed; a
  link that names no build directory is a warning").
- **Evidence:** reproduced by the reviewer on a scratch XDG state.
- **Verification:** CONFIRMED — `current -> ../x/y` beside `b11000` prints "warning: …/current
  points at ../x/y, not at a build of its directory; …" on stderr; `current -> b99999` beside
  `b11000` prints the line on stdout, stderr empty, exit 0. Only `FsBuildStore::current`'s
  error (`build_store.rs:182-191`) reaches `warn`. A cause of its own: the channel, whatever
  the listing holds. The `warn` way is the better one.
- **User's verdict:** confirmed — the session, under the user's delegation of 2026-10-18

### 64. While `build prune` holds the gateway's lock, a concurrent `up`, `down` or `status` is told another ai-ctl is starting or stopping the gateway

- **Axes:** the CLI's contract — raised by `review-structure`
- **Action:** fix
- **Severity:** —
- **Where:** `src/infrastructure/gateway_process.rs:139-142`, `src/application/build.rs:401-403`
- **Rule:** `docs/commands.md:88-89`
- **Cause:** `prune` takes `gateway.lock()`, whose busy text is fixed at `LocalGateway::lock`
  ("starting or stopping the gateway"), and `docs/commands.md:94` lists prune as a holder.
- **Consequence:** commands.md promises that the refusal says what the other ai-ctl is doing.
  During a prune, `up` fails with exit 1 naming an activity no ai-ctl is doing. The window is
  short: prune holds the lock while it reads the record and its liveness, without HTTP, and
  removes the directories.
- **Fix:** a busy text for prune, for instance by passing the activity to
  `GatewayProcess::lock` ("removing builds"); or the holder's activity written into `.lock` and
  read on `WouldBlock`.
- **Evidence:** read by the reviewer: `build.rs:403`, `gateway_process.rs:141`.
- **Verification:** CONFIRMED — the text is fixed in `files.rs:54-57`. With `flock
  …/gateway/.lock sleep 3` holding the lock, `ai-ctl status` printed "Error: another ai-ctl is
  starting or stopping the gateway (…/.lock is locked)", exit 1, whoever held it. Apart from
  finding 13: 13's fix takes `status` off the lock and turns down the wording fix, so prune is
  still a holder that neither starts nor stops; 64 is what 13's fix leaves.
- **User's verdict:** confirmed — the session, under the user's delegation of 2026-10-18

### 65. `gateway config`, `up` and `bench run` need a readable GPU, which the documents' lists of failures leave out

- **Axes:** the CLI's contract — raised by `review-structure`; pass 4's second verifier's lead
  on `bench run`'s section, merged
- **Action:** fix
- **Severity:** —
- **Where:** `src/application/gateway_config.rs:44-46`, `src/application/gateway_config.rs:56-58`, `src/application/lifecycle.rs:226-227`, `src/application/bench.rs:282`
- **Rule:** `docs/commands.md:279-281`
- **Cause:** `Configurator::prepare` reads the GPU, and `prepare_on` sizes the matrix from its
  usable memory, failing with "cannot size the models to the GPU's memory: …" when NVML gives
  none. `gateway config` (`presentation/gateway.rs:30`), `up` (through `target`,
  `lifecycle.rs:227`) and `bench run` (`bench.rs:282`) all go through it; `bench run` reads the
  GPU a second time for its record (finding 55).
- **Consequence:** none of the lists of failures names the GPU: `docs/commands.md:279-281`
  (`gateway config`), `:299-304` (up's refusals), the `bench run` section (from `:376`), which
  says only that "every run also reads the GPU every 250 ms", and `README.md:142-144` and
  `:160-161`. "Prints, without starting anything" reads as an offline command, yet on a
  machine whose driver is broken or absent it fails. A monitor that fails lets a bench run go
  on with a warning, but a GPU unreadable at its setup ends it, and the page does not say so.
- **Fix:** add "or when the GPU cannot be read, since the models are sized to its memory" to
  `docs/commands.md:279-281`, the list at `:299-304`, the `bench run` section, and
  `README.md:142-144` and `:160-161`.
- **Evidence:** read by the reviewer: `gateway_config.rs:56-58` and
  `GatewayAction::Config => services.configurator().generate()?`.
- **Verification:** CONFIRMED — the failure is pinned by
  `without_a_readable_gpu_there_is_no_matrix_to_compute` (`gateway_config.rs:163`). Corrected:
  the tests the reviewer cited (`lifecycle.rs:1906`, `presentation/lifecycle.rs:412`) pin the
  text through `status`, not through `up` or `gateway config`; the fix also covers
  `README.md:160-161` and the `bench run` section, the latter from the verifier's lead on
  `docs/commands.md:384-388`.
- **User's verdict:** confirmed — the session, under the user's delegation of 2026-10-18

### 66. "Each request is printed on stderr as it is measured" does not hold for the set-aside requests or the overhead's 100 requests

- **Axes:** the CLI's contract — raised by `review-structure`
- **Action:** fix
- **Severity:** —
- **Where:** `src/presentation/bench.rs:80-127`
- **Rule:** `docs/commands.md:378-379`
- **Cause:** `render_event` returns `None` for `Step::SetAside`, and for every overhead request
  but each tenth direct one, which prints "  N requests each way".
- **Consequence:** `docs/commands.md:378-379` and `README.md:217-218` describe progress the run
  does not give: five lines for the overhead's 100 requests, none for the set-aside ones.
- **Fix:** reword both: each measured request is printed on stderr as it is measured, the
  overhead's every ten requests, and the set-aside requests not at all.
- **Evidence:** read by the reviewer: `bench.rs:123-127` and its comment.
- **Verification:** CONFIRMED — `Step::SetAside => return None`; an overhead request prints
  only when direct and `request % 10 == 0`, five lines for 50 each way
  (`docs/commands.md:390`). The set-aside clause of the fix is the verifier's.
- **User's verdict:** confirmed — the session, under the user's delegation of 2026-10-18

### 67. "The error quotes the last lines of `logs/build-<project>.log`" holds only for a failed compile

- **Axes:** the CLI's contract — raised by `review-structure`
- **Action:** fix
- **Severity:** —
- **Where:** `src/application/build.rs:113`, `src/application/build.rs:116-120`, `src/application/build.rs:171-188`, `src/application/build.rs:217-241`
- **Rule:** `docs/commands.md:172-173`
- **Cause:** only the `Compiler` failure quotes the log (`ports.rs:183-184`). The gate's refusal
  quotes doctor's checks, a failed fetch or checkout quotes git, a failed smoke test quotes the
  binary's own output through `summary`, and stage, publish and `set_current` give file-system
  errors.
- **Consequence:** `docs/commands.md:172-173` and `README.md:87-89` send the user to a log that
  does not hold the failure, for instance a new binary that lists no CUDA device.
- **Fix:** reword both: a failed compile quotes the last lines of `logs/build-<project>.log`;
  the other steps quote their own error.
- **Evidence:** read by the reviewer: `build.rs:220-238` and `:113`.
- **Verification:** CONFIRMED — each step's error read as listed above.
- **User's verdict:** confirmed — the session, under the user's delegation of 2026-10-18

### 68. The Files table leaves out `~/.local/share/ai-ctl/work/`, where `build` keeps its CMake trees and where a failed build leaves its tree

- **Axes:** the CLI's contract — raised by `review-structure`
- **Action:** fix
- **Severity:** —
- **Where:** `src/infrastructure/paths.rs:58-61`, `src/infrastructure/cmake.rs:58-62`, `src/application/build.rs:186-188`
- **Rule:** `docs/commands.md:67-82`
- **Cause:** `Paths::work_dir` holds each project's build tree; `compile` removes it only at the
  start of the next build, and `clean` (`cmake.rs:105-107`) runs only after a successful
  publish.
- **Consequence:** the list of what ai-ctl writes misses a directory that keeps a whole
  llama.cpp build tree after a failed compile, smoke test or publish. A user freeing disk space,
  or reading what a failure left, is not told it exists.
- **Fix:** a row for `~/.local/share/ai-ctl/work/<project>/`, written by `build`, holding the
  CMake build tree, removed after a successful build and kept after a failed one; no size
  figure.
- **Evidence:** read by the reviewer: `paths.rs:58-61`, `cmake.rs:59-61`, `build.rs:186-188`.
- **Verification:** CONFIRMED — no `work/` row in the table. Corrected: a failed smoke test or
  publish also leaves the tree, not only a failed compile; the reviewer's "gigabytes" is not
  measured, the directory being empty on this machine (`du`), so the row gives no size.
- **User's verdict:** confirmed — the session, under the user's delegation of 2026-10-18

### 69. up's warning "not looking for processes left on the GPU", whose text batch 45 changed, can never be emitted

- **Axes:** the CLI's contract — raised by `review-structure`; also dead code, which
  `review-cleanup`'s search for dead items cannot see
- **Action:** fix
- **Severity:** —
- **Where:** `src/application/lifecycle.rs:226-227`, `src/application/lifecycle.rs:249-268`
- **Cause:** `up` calls `self.target(&reading)?` at line 227, whose `prepare_on` fails on a
  reading without a GPU (`gateway_config.rs:56-58`), before `warn_unread` at line 249. So
  `warn_unread` never warns there, and `reading.found()` is always `Some`.
- **Consequence:** phase 4's Assessment says that the texts batches 43 to 48 changed are pinned
  by their tests (`phase-4-structure-and-refactoring-report.md:385`); this one is not, because
  it cannot be printed. Batch 45 (`refactor: turn a GPU reading into a GPU or an error once`) changed it from "…left on it".
- **Fix:** delete that `warn_unread` call and unwrap the reading once after `target`
  succeeds, for instance by having `target` return the `&Gpu`. The two other calls of
  `warn_unread` (`lifecycle.rs:310`, `:351`) can be reached and stay.
- **Evidence:** read by the reviewer; `grep -rn 'not looking for processes left' src tests`
  finds `lifecycle.rs:251` only.
- **Verification:** CONFIRMED — M31, lines 249-254 replaced by an assertion that the reading is
  found and an `expect`, in a clone of `docs(v2-review-fixes): record pass 4's first verification of the closing review` of 2026-10-18: the whole `cargo test` passes. The other two
  calls are reachable.
- **User's verdict:** confirmed — the session, under the user's delegation of 2026-10-18

### 71. Batch 43's test of `request_ok` checks the non-200 text on an error body written in the test, though captured llama-swap error responses sit in `tests/fixtures`

- **Axes:** conventions — raised by `review-structure`
- **Action:** fix
- **Severity:** —
- **Where:** `src/infrastructure/http.rs:701-728`, `src/infrastructure/http.rs:611-615`
- **Rule:** `CLAUDE.md:69-71`
- **Cause:** `only_a_200_response_gives_its_body` serves `{"error":{"message":"no such
  model"}}`, written from memory; `an_error_body_gives_its_message`, older than the scope, a
  cut-down copy of `llama-swap-263-chat-unknown-model.http`. Parsers are tested on captured
  outputs, never written from memory.
- **Consequence:** two tests break the convention. The parser itself is tested on real output:
  `an_error_answer_names_the_endpoint` (`server_api.rs:419-430`) goes through `request_ok`,
  `unexpected` and `error_message` on the captured file, and `chat_api.rs:429-437` checks
  `error_message` on both captured fixtures. The 401 has the same shape as the captured 404.
- **Fix:** replace the bodies at `http.rs:611-615` and `:711` with `include_bytes!` of the
  existing fixtures. Capturing the 401 is optional: it adds no parse path.
- **Evidence:** reproduced by the reviewer: llama-swap v263 alone in scratch, `GET /running`
  without a key returns 401 with an `error.message`.
- **Verification:** CONFIRMED on the title; consequence and fix corrected — the reviewer's "not
  tested on real output" does not hold, given the two tests above. llama-swap v263 alone on a
  loopback port: without a key, `GET /running` gives 401 `{"src":"llama-swap","error":{"message":"unauthorized: invalid or missing API key",…}}`;
  with it, `GET /upstream/nope/props` gives 404 `{"src":"llama-swap","error":{"message":"model not found",…}}`,
  the same shape. The non-200 answers ai-ctl can get are not the 401 alone.
- **User's verdict:** confirmed — the session, under the user's delegation of 2026-10-18

### 72. "The GPU is read through NVML, which no test calls", written by batch 50, does not hold: three binary tests load and query NVML

- **Axes:** conventions — raised by `review-structure`
- **Action:** fix
- **Severity:** —
- **Where:** `tests/cli.rs:243-272`, `tests/cli.rs:293-300`, `tests/cli.rs:303-309`
- **Rule:** `docs/architecture.md:114`
- **Cause:** `doctor_exits_1_when_a_check_fails` (`test(doctor): check every failing probe and store, and doctor's exit status` of 2026-10-16), `status_without_a_gateway_exits_3`
  and `gateway_config_needs_an_inventory_and_a_build` run the real binary, whose `NvmlProbe`
  loads libnvidia-ml and reads the GPU. Batch 50 checked only `nvml.rs`'s own tests.
- **Consequence:** the document's account of which tests touch the GPU is wrong. Their
  assertions do not depend on the GPU, so `CLAUDE.md:71` holds.
- **Fix:** reword `docs/architecture.md:114`: "no unit test calls NVML; the binary tests of
  `doctor`, `status` and `gateway config` load it, and assert nothing that depends on it".
- **Evidence:** reproduced by the reviewer: `env -i … LD_DEBUG=libs ai-ctl status` and `…
  doctor` log "calling init: /lib64/libnvidia-ml.so.1".
- **Verification:** CONFIRMED, corrected to three tests — in a clone of `docs(v2-review-fixes): record pass 4's first verification of the closing review` of 2026-10-18, `ai_ctl()`
  of `tests/cli.rs` given `LD_DEBUG=libs` and a log per command: `cargo test --test cli`, 25
  passed, and NVML was loaded by `doctor --json`, `status` and both `gateway config` runs of
  `gateway_config_needs_an_inventory_and_a_build`, the first without an inventory included; by
  no other command the tests run.
- **User's verdict:** confirmed — the session, under the user's delegation of 2026-10-18

### 73. Seven failure switches added in scope are not named `<method>_fails`, the convention batch 50 recorded

- **Axes:** conventions — raised by `review-structure`
- **Action:** fix
- **Severity:** —
- **Where:** `src/application/fakes.rs:434`, `src/application/fakes.rs:545`, `src/application/fakes.rs:680`, `src/application/fakes.rs:742`, `src/application/fakes.rs:813`, `src/application/build.rs:571`, `src/application/gateway.rs:275`
- **Rule:** `docs/architecture.md:119-120`
- **Cause:** `installed_fails` for `is_installed` (`test(doctor): check every failing probe and store, and doctor's exit status` of 2026-10-16), `tail_fails` for `log_tail`
  (`test(lifecycle): exercise the gateway's degraded paths` of 2026-10-16), `cgroup_fails` for `in_container` (`refactor(gateway): split GatewayProcesses by role`), `health_fails` for `healthy`
  (`test(lifecycle): exercise the gateway's degraded paths` of 2026-10-16), `exists_fails` for `image_exists` (`test(lifecycle): exercise the gateway's degraded paths` of 2026-10-16), and `bad_link`, twice, which makes
  `current` fail (`fix(build): replace a current link that names no build instead of failing` of 2026-10-16); all in scope.
- **Consequence:** the convention the document states as settled is broken by the phases that
  settled it; a reader looking for `healthy_fails` does not find it.
- **Fix:** rename the five `is_installed_fails`, `log_tail_fails`, `in_container_fails`,
  `healthy_fails` and `image_exists_fails`. `bad_link` is state, not a pure switch:
  `set_current` clears it (`build.rs:637`, `gateway.rs:357`), so `current_fails` would hide
  that; narrow the convention to failure switches, or keep the name with a comment.
- **Evidence:** read by the reviewer: the field declarations, with `git blame`.
- **Verification:** CONFIRMED — every other `*_fails` field matches its method. Corrected for
  `bad_link`, as the fix says.
- **User's verdict:** confirmed — the session, under the user's delegation of 2026-10-18

### 74. "An adapter that runs a program holds the program as a field", recorded by batch 50, is followed by `Cmake` and `Podman` but not by most other adapters

- **Axes:** conventions — raised by `review-structure`
- **Action:** fix
- **Severity:** —
- **Where:** `src/infrastructure/git.rs:174`, `src/infrastructure/download.rs:31`, `src/infrastructure/gateway_store.rs:115`, `src/infrastructure/gateway_store.rs:124`, `src/infrastructure/gateway_process.rs:241`, `src/infrastructure/host.rs:169`, `src/infrastructure/host.rs:192`, `src/infrastructure/host.rs:211`, `src/infrastructure/host.rs:247`
- **Rule:** `docs/architecture.md:121-123`
- **Cause:** `GitSources` hard-codes `git`, `Curl` `curl`, `FsGatewayStore` `sha256sum` and
  `tar`, `LocalGateway::signal` `kill`; `SystemHost` resolves `nvcc`, `g++`, `rpm` and `busctl`
  through its `path` field, but runs `dnf`, `nvidia-ctk` and the tool versions through the
  process's `PATH`. Only `cmake.rs:27` and `podman.rs:16` hold `program: PathBuf`.
- **Consequence:** the document states as general a convention that two adapters follow. Where
  it does not hold, a failing program cannot be scripted: a refused `kill` is reached only
  through the fake (finding 41).
- **Fix:** give these adapters their program as a field, as `Podman::new("podman")` does, or
  narrow `docs/architecture.md:121-123` to the adapters that follow it and name the route the
  others take (git on a scratch repository).
- **Evidence:** read by the reviewer: the listed lines.
- **Verification:** CONFIRMED — the hard-coded names as listed, `host.rs:211` added (`dnf`);
  `host.rs:224` and `:296` resolve `rpm` and `busctl` through `self.find`.
- **User's verdict:** confirmed — the session, under the user's delegation of 2026-10-18: `docs/architecture.md:121-123` narrowed to the adapters that follow it, the others unchanged

### 75. The Ports section's example of a port with two implementations, "the source of a backend in the gateway configuration", names no port of the code

- **Axes:** conventions — raised by `review-structure`
- **Action:** fix
- **Severity:** —
- **Where:** `docs/architecture.md:45-46`
- **Rule:** `docs/architecture.md:45-46`
- **Cause:** a backend's source is the domain's `BackendSource` and `Runtime` enums
  (`domain/gateway_config.rs:69`), matched in `Configurator::runtime`; no trait exists for it,
  and no port has two production implementations. Batch 50 left the example, written in
  `refactor: rename llm-ctl to ai-ctl and adopt llama-swap as the gateway` (2026-10-09).
- **Consequence:** the document that defines the port policy illustrates its first reason with
  code that disproves it; a reader may write a trait for a two-variant choice, which
  `CLAUDE.md:60` does not call for.
- **Fix:** replace the example with a real case, or say that every port so far exists for a test
  double.
- **Evidence:** read by the reviewer: `grep -rn 'impl .* for' src` per port.
- **Verification:** CONFIRMED — counting the non-test `impl <Port> for` of every trait in
  `ports.rs` gives exactly one production implementation each.
- **User's verdict:** confirmed — the session, under the user's delegation of 2026-10-18

### 76. `Services::configurator`'s doc says `up`, `bench` and `gateway config` share it; it serves six commands

- **Axes:** conventions — raised by `review-structure`
- **Action:** fix
- **Severity:** —
- **Where:** `src/presentation/mod.rs:64-65`
- **Rule:** `docs/architecture.md:35-36`
- **Cause:** batch 37 (`refactor: build the configurator in Services and inject doctor's pause`) wrote the doc; `down`, `status` and `logs` call it too,
  through `presentation::lifecycle::lifecycle` (`presentation/lifecycle.rs:22-34`).
- **Consequence:** the comment disagrees with `docs/architecture.md:35-36` and with the code.
- **Fix:** "which `up`, `down`, `status`, `logs`, `bench run` and `gateway config` share", the
  list of `docs/architecture.md`.
- **Evidence:** read by the reviewer, with `git blame`.
- **Verification:** CONFIRMED — the six handlers build it. `logs` builds the configurator but
  never uses it; `down` uses its GPU and builds, `status` its `prepare_on`.
- **User's verdict:** confirmed — the session, under the user's delegation of 2026-10-18

### 77. The domain module's doc says any release but the pinned one is checked against the published checksums; since `fix(gateway): say what the checksums of a non-pinned release prove` of 2026-10-16, a `--sha256` given is checked first

- **Axes:** conventions — raised by `review-structure`
- **Action:** fix
- **Severity:** —
- **Where:** `src/domain/gateway.rs:3-5`
- **Cause:** `fix(gateway): say what the checksums of a non-pinned release prove` of 2026-10-16 (phase 0, the fix of the first review's finding 4) added `--sha256` and
  `DigestSource::Given` without updating the module's doc.
- **Consequence:** the doc understates what an unpinned install is checked against, against
  `docs/commands.md:262-264`, which gives the order: given, pinned, published.
- **Fix:** "ai-ctl ships with one pinned release …; an archive is checked against the digest
  given, else the pinned one, else the checksums file published with the release."
- **Evidence:** read by the reviewer: `gateway.rs:3-5`, `application/gateway.rs:146-161`.
- **Verification:** CONFIRMED — `application/gateway.rs:146-148` returns `DigestSource::Given`
  first; `fix(gateway): say what the checksums of a non-pinned release prove` of 2026-10-16 does not touch `domain/gateway.rs`.
- **User's verdict:** confirmed — the session, under the user's delegation of 2026-10-18

### 78. README's Status says performance and structure "come next", but phases 3 and 4 of v2-review-fixes closed on 2026-10-17

- **Axes:** conventions — raised by `review-structure`
- **Action:** fix
- **Severity:** —
- **Where:** `README.md:16-18`
- **Cause:** `docs: list the gateway's record and lock, and say the review's fixes are under way` of 2026-10-16 (phase 2) wrote the line; the closures of phases 3 (`docs(v2-review-fixes): close phase 3, performance`) and 4
  (`docs(v2-review-fixes): close phase 4, structure and refactoring`) left it, while they updated CLAUDE.md's Current work.
- **Consequence:** the first status a reader sees contradicts `CLAUDE.md:10-14`.
- **Fix:** "…security, bugs, tests, performance and structure are done; a closing review is
  under way", or the roadmap's state when the fix lands.
- **Evidence:** read by the reviewer.
- **Verification:** CONFIRMED — corrected: the contradicting lines are `CLAUDE.md:10-14`.
- **User's verdict:** confirmed — the session, under the user's delegation of 2026-10-18

### 89. A comment says NVML's initialisation costs 10 to 20 ms; the first read measures 23 to 28 ms, the initialisation alone 7 to 9 ms

- **Axes:** conventions — a lead of pass 5's `review-verifier`
- **Action:** fix
- **Severity:** —
- **Where:** `src/infrastructure/nvml.rs:32-34`
- **Cause:** "an initialisation costs 10 to 20 ms" names no measured quantity: it matches the
  initialisation alone of 2026-10-15 and 16, or the whole reads of 2026-10-16 (`perf(gpu): initialise NVML once per process` of 2026-10-16,
  `phase-3-performance-report.md:113-117`; the first review's finding 18), and the cost has
  drifted since.
- **Consequence:** a wrong figure in the comment that explains why NVML is initialised once:
  `Nvml::init` alone costs 7 to 9 ms, under it; the first read, which also attaches the device,
  23 to 28 ms, over it. No behaviour depends on it. On the same driver, 615.78.08, with
  persistence off, the first read went from 15 to 20 ms to 23 to 28 ms in two days: a figure
  pinned in a comment drifts.
- **Fix:** the comment gives the order of magnitude and names the quantity: "The first read,
  which loads the library, initialises NVML and attaches the device, costs tens of
  milliseconds; a later read, about a tenth of one." The measured values go in the fix batch's
  report, not in the code. A comment only, so no test.
- **Evidence:** reported by pass 5's verifier from its harness, NVML's first read 24.6 and
  23.9 ms; the reviewer measured 24 to 26 ms.
- **Verification:** CONFIRMED — 40 fresh processes in two alternating modes, NVML only, no CUDA
  context, on a clone of `docs(v2-review-fixes): record pass 5's verification of the closing review` of 2026-10-18: `Nvml::init` 7.0 to 8.6 ms, the first `device_by_index(0)`
  10.7 to 13.4 ms, the memory and the two process lists 4.2 to 6.9 ms; `NvmlProbe::gpu`'s first
  read 23.0 to 27.6 ms, median 24.3, and a later read 0.077 to 0.096 ms, each process's median.
  `git log -S"10 to 20 ms"` gives `perf(gpu): initialise NVML once per process` of 2026-10-16. Driver 615.78.08 then and now, persistence mode
  Disabled, `nvidia-persistenced` inactive; only the kernel changed. No rule of CLAUDE.md or
  `docs/architecture.md` covers figures in comments: what fails is the comment's accuracy. The
  fix is the verifier's.
- **User's verdict:** confirmed — the session, under the user's delegation of 2026-10-18

### 50. The run store reads saved runs of formats 1 and 2 with no `// LEGACY:` comment stating the obligation behind it

- **Axes:** dead and legacy code — raised by `review-cleanup`
- **Action:** fix
- **Severity:** —
- **Where:** `src/infrastructure/run_store.rs:38-45`, `src/infrastructure/run_store.rs:173-175`, `src/infrastructure/run_store.rs:311-314`, `src/infrastructure/run_store.rs:368-372`, `src/infrastructure/run_store.rs:469-476`, `src/domain/bench.rs:703-705`
- **Cause:** E2 (phase 4) and `feat(bench): keep NVML's clock-event mask and show a policy limit apart` of 2026-10-16 (phase 1) kept a read path for older run formats:
  `READS = [1, 2, 3]`, `BUILD_JSON_FORMAT`, the optional `BuildJson::format` and its check in
  `build()`, the `#[serde(default)]` on `GpuReadingJson::policy` and `clock_events`, and the
  domain's `GpuReading::clock_events: Option<u64>`, which is `None` only in a format-1 run
  (`nvml.rs:240` always writes `Some`). Their doc comments describe the formats; none carries a
  `// LEGACY:` comment naming the obligation, the user's rule for legacy code
  (`.claude/agents/review-cleanup.md:31-35`).
- **Consequence:** the obligation is real: the seven runs saved on this machine are six of
  format 1 and one of format 2, none of format 3. The code does not state it, so nothing tells
  a reader that this path, a domain `Option` that holds only old data, and a `policy: false`
  default must stay, or when they can go. A format-1 run's policy share prints "–"
  (`presentation/bench.rs:311-315`), as for a run no policy limited: "not recorded" cannot be
  told from "none". Finding 10's fix reworks the same function and should keep or state this
  obligation.
- **Fix:** `// LEGACY:` comments at `READS`, `BUILD_JSON_FORMAT`, `BuildJson::format` and its
  check, the two `#[serde(default)]` attributes and `GpuReading::clock_events`, each stating
  that the runs saved before format 3 must stay readable for `bench list` and `bench compare`.
  The other way out, at the user's choice: a one-off migration of those runs to format 3,
  after which the path and the `Option` go.
- **Evidence:** read: `grep -rn LEGACY src tests deploy` finds nothing; `clock_events` is
  written only at `nvml.rs:240`. Reproduced: the `format` and `setup.source.build.format` of
  `~/.local/share/ai-ctl/bench/*.json` read with `python3 -I`.
- **Verification:** CONFIRMED — the seven runs read again: the format-2 run already holds
  `policy` and `clock_events`, so the two serde defaults serve only the six format-1 runs, and
  `BuildJson::format` the six runs on a native build. `ai-ctl bench list` on a scratch XDG copy
  of the seven runs exits 0 and lists all seven. Corrected: the policy share of a format-1 run
  prints "–", not "0 %", as the reviewer wrote. `bench compare` not run: that reading rests on
  the code.
- **User's verdict:** confirmed — the session, under the user's delegation of 2026-10-18: the `// LEGACY:` comments, no migration of the saved runs

### 51. `Doctor::pause` and `Doctor::now` are passed in by presentation and the tests, and never called

- **Axes:** dead and legacy code — raised by `review-cleanup`; architecture — raised by
  `review-structure`, merged at step 7 (its number, 61, is unused), refuted as a defect of its
  own by the verifier
- **Action:** fix
- **Severity:** —
- **Where:** `src/application/doctor.rs:65-67`, `src/application/doctor.rs:112-128`, `src/application/doctor.rs:194-196`, `src/presentation/mod.rs:101-102`, `src/application/doctor.rs:785-800`, `src/application/doctor.rs:858-873`; for the port: `src/application/ports.rs:350-383`
- **Cause:** batch 37 gave `Doctor` the lifecycle's `pause` and `now` so that
  `Doctor::lifecycle()` builds a whole `Lifecycle`. Doctor uses it only for `inspect`
  (`doctor.rs:77`) and for its configurator's `prepare_on` (`doctor.rs:196`), and neither
  reaches `log`, `wait_for_release`, `wait_until_healthy`, `time_left` or `stop`, the only
  readers of `pause` and `now` (`lifecycle.rs:445, 664, 678, 703, 725, 761, 782`). The field's
  doc says so: "though inspecting never waits". `review-structure` raised the same cause on the
  architecture axis: through that `Lifecycle`, Doctor holds the whole ten-method
  `GatewayProcess`, `start`, `signal` and `forget` included, where its inspection calls only
  `record`, `liveness` and `config`; so do `Bench` and prune.
- **Consequence:** two fields, and six lines that fill them (two in presentation's literal, two
  in each of doctor's two test constructions), carry values nothing calls. `doctor.rs:1763`,
  which the reviewer counted, builds a `Lifecycle` for `up`, where they are live. The port's
  width breaks no rule: `docs/architecture.md:53-54` asks a use case to hold only the ports it
  calls, and Doctor, `Bench` and prune call `GatewayProcess`.
- **Fix:** move the read-only part of `Lifecycle` (`inspect`, `observe`, `owners`, `ancestry`,
  `in_container`, `leftovers`, `target`, `is_current`) into its own struct of the ports it
  reads (`configurator`, `releases`, `gateway`, `processes`, `api`, `containers`); `Lifecycle`
  holds it and adds `ports`, `pause` and `now`. Doctor builds only that struct, keeps its own
  `ports` for `address_check` (`doctor.rs:228`), and `config_check` calls its `Configurator`
  directly; `Doctor::pause`, `Doctor::now` and the six lines go. The `up` test at
  `doctor.rs:1751` keeps its `Lifecycle`. An optional rider, at the user's choice: a read-only
  supertrait `GatewayState` (`record`, `liveness`, `config`) under `GatewayProcess`, taken by
  the inspection struct, `Bench`, `live_record` and `refuse_served`, with prune keeping
  `GatewayProcess` for `lock`; it does not follow from the rest of the fix, which removes
  `pause` and `now` with or without it.
- **Evidence:** reproduced by the reviewer: in a clone, `Doctor::lifecycle()` given a `pause`
  and a `now` that panic; `cargo test --lib --test cli` passed, and that build's `ai-ctl
  doctor` exited 0 with no panic. `review-structure`: a grep of `gateway.<method>(` per use
  case.
- **Verification:** CONFIRMED on dead and legacy code — the panicking `pause` and `now` again,
  in a clone of `docs(v2-review-fixes): record the session close of the closing review's pass 4` of 2026-10-18: the whole `cargo test` passes, and that build's `ai-ctl doctor`
  exits 0 with 23 checks ok and no panic, the gateway stopped; the code read covers the running
  case. Corrected: the dead lines are six, not eight. REFUTED as a defect of its own on
  architecture: `docs/architecture.md:53-54` speaks of whole ports; the port's shape is the
  first review's approved fix of its finding 25 (`2026-10-15-v2-rebuild.md:1213-1218`);
  Doctor's read-only contract (`docs/commands.md:126`) holds in behaviour, since `inspect`
  reports a stale record without `forget` and no path of Doctor reaches `start`, `signal` or
  `forget`; and `CLAUDE.md:57-58` holds, since presentation assembles Doctor from `Services`
  and `Doctor::lifecycle()` builds no adapter. The upcast from `&dyn GatewayProcess` to
  `&dyn GatewayState` compiles and runs on rustc 1.97.1; it is stable since 1.86, below the
  crate's `rust-version` 1.89. The merge holds on the cause; the supertrait stays a rider.
- **User's verdict:** confirmed — the session, under the user's delegation of 2026-10-18: without the `GatewayState` rider

### 52. "Read a file, `None` when it is missing, `cannot read <path>` otherwise" is written six times beside its link twin `read_link_if_present`

- **Axes:** refactoring — raised by `review-cleanup`
- **Action:** fix
- **Severity:** —
- **Where:** `src/infrastructure/api_key.rs:31-37`, `src/infrastructure/gateway_process.rs:146-150`, `src/infrastructure/gateway_process.rs:180-184`, `src/infrastructure/gateway_process.rs:286-292`, `src/infrastructure/gateway_process.rs:296-301`, `src/infrastructure/inventory_file.rs:43-49`, `src/infrastructure/files.rs:22-28`
- **Cause:** the scope adds three copies of the same `fs::read_to_string` match (`NotFound` to
  absent, other errors to `cannot read <path>`): `KeyFile::read` (`feat(gateway): require ai-ctl's API key and a loopback listen address` of 2026-10-16), `Procfs::stat`
  and `Procfs::in_container` (`perf(gateway): place GPU processes by walking up their parents`), beside three older ones, `LocalGateway::record`,
  `LocalGateway::config` and `InventoryFile::load`. The first review's finding 62 put the same
  rule for links into `files::read_link_if_present`; no file version exists.
- **Consequence:** one rule with one error text in six places across three adapters; each new
  reader of an optional state file writes it again.
- **Fix:** add `files::read_if_present(path) -> Result<Option<String>>` beside
  `read_link_if_present`, with the same context. `config` becomes that call; `record`,
  `KeyFile::read` and `InventoryFile::load` start with
  `let Some(text) = files::read_if_present(..)? else { return Ok(None) };`; `stat` maps the
  text through `process_stat` with its own "unexpected …" context, and `in_container` is
  `Ok(files::read_if_present(&path)?.is_some_and(|c| names_a_container(&c)))`. In `api_key.rs`,
  the production imports narrow to `File` and `Read`, and the test-only `KeyFile::written`
  calls `std::fs::write`. `host.rs:184` (maps to `Probed`) and `run_store.rs:102-105` (its own
  "no run is saved") stay: their outcomes differ.
- **Evidence:** read: `grep -B2 'ErrorKind::NotFound' src`; each of the six ends in
  `format!("cannot read {}", path.display())`.
- **Verification:** CONFIRMED — the six windows and their blame as stated. The fix applied in a
  clone of `docs(v2-review-fixes): record the session close of the closing review's pass 4` of 2026-10-18: `cargo fmt`, `cargo clippy --all-targets -- -D warnings` and the whole
  `cargo test` pass. The imports of `api_key.rs` are the verifier's addition.
- **User's verdict:** confirmed — the session, under the user's delegation of 2026-10-18

### 53. The versioned toolkit package name `cuda-toolkit-<major>-<minor>` is formatted in three places, while its parser lives in the domain

- **Axes:** refactoring — raised by `review-cleanup`
- **Action:** fix
- **Severity:** —
- **Where:** `src/application/toolchain.rs:78`, `src/application/toolchain.rs:112-113`, `src/application/toolchain.rs:297-302`, `src/domain/machine.rs:213-223`
- **Cause:** `Setup::package` (`feat(build): add native builds with list, use and prune` of 2026-10-09) formats the name; `fix(doctor): remove a toolkit newer than the driver, and pin the driver's` of 2026-10-16, the fix of the first
  review's findings 101 and 102, added `newer_toolkit_remedy`'s and `package_check`'s. Its
  inverse, `domain::machine::versioned_toolkit`, sits in the domain.
- **Consequence:** the naming rule is written three times in the application layer and parsed
  once in the domain: a change has four places to agree.
- **Fix:** add `domain::machine::toolkit_package(release: (u32, u32)) -> String` beside
  `versioned_toolkit`, fed by `Version::release()` (`machine.rs:34-36`); `Setup::package`,
  `package_check` and `newer_toolkit_remedy` call it. The remedy keeps its `{major}-{minor}`
  text for its `grep -E` pattern.
- **Evidence:** read: `grep -n 'cuda-toolkit-{' src`; `git blame` puts `:113` and `:300` in
  `fix(doctor): remove a toolkit newer than the driver, and pin the driver's` of 2026-10-16.
- **Verification:** CONFIRMED — the three sites and their blame as stated; the fix not
  applied.
- **User's verdict:** confirmed — the session, under the user's delegation of 2026-10-18

### 54. "A `current` link that names nothing reads as none, with a warning naming what replaces it" is written twice

- **Axes:** refactoring — raised by `review-cleanup`
- **Action:** fix
- **Severity:** —
- **Where:** `src/application/build.rs:369-379`, `src/application/gateway.rs:91-97`
- **Cause:** `fix(build): replace a current link that names no build instead of failing` of 2026-10-16, the fix of the first review's finding 14, wrote the rule twice:
  `build::current_or_none` for builds, and inline in `Installer::install` for releases; both
  turn `Err` into the warning `{err:#}; <what> replaces it` and `None`.
- **Consequence:** one rule and its message template copied between two use cases, whose
  wording can drift. The gain of the fix is one template; keeping both is defensible.
- **Fix:** generalize `current_or_none` over the read's result,
  `fn lenient<T>(read: Result<Option<T>>, replaced_by: &str, warn: &mut dyn FnMut(String)) -> Option<T>`,
  `pub(super)` in `application::build` as `summary` already is for `gateway.rs`; the texts are
  unchanged.
- **Evidence:** read: both windows; their tests pin the same shape (`build.rs:1160`, `:1181`,
  `:1192`, `gateway.rs:373`).
- **Verification:** CONFIRMED — both windows are the same `unwrap_or_else` on `BuildId` and on
  `Release`; `gateway.rs:10` already imports `super::build::summary`. The fix not applied.
- **User's verdict:** confirmed — the session, under the user's delegation of 2026-10-18

### 55. `Bench::setup` loads the inventory, looks up the backend and resolves the current build again, right after `Configurator::prepare` did

- **Axes:** refactoring — raised by `review-cleanup`; the CLI's contract — raised by
  `review-structure`, merged at step 7 (its number, 70, is unused); phase 4's performance lead,
  the bench's two GPU reads, goes with it; performance — raised by `review-performance` in pass
  5, merged at step 7 (its number, 86, is unused)
- **Action:** fix
- **Severity:** —
- **Where:** `src/application/bench.rs:281-347`, `src/application/bench.rs:354-357`, `src/application/gateway_config.rs:42-106`
- **Cause:** `setup` calls `configurator.prepare()`, which reads the GPU, loads the inventory,
  resolves each backend and calls `current(..)?.installed(..)` (`gateway_config.rs:84`, `:97`).
  `setup` then loads the inventory again ("the inventory is gone", `bench.rs:317`), finds the
  entry again ("changed during the run", `:322`), looks up the backend again (`:326`), calls
  `current(..)?.installed(..)` again (`:329`, batch 44's shared function, which re-reads the
  whole build list), and `machine()` reads the GPU again. `Prepared` keeps none of what it read
  besides the binaries.
- **Consequence:** the same lookups and an error text twice in one run, and three error paths
  that exist only for the second read: the two inventory errors and bench's "cannot record the
  run's GPU" (`bench.rs:357`), whose text batch 45 changed and no test pins. Neither `bench
  run` nor `build use` holds a lock the other takes here, so a `build use` between the
  comparison (`bench.rs:308`) and `:329` makes the run record a build other than the one it
  compared with the gateway's. The second pass costs about 0.09 to 0.12 ms a run, the GPU's
  read included, measured in pass 5: the fix has no performance reason.
- **Fix:** Bench reads the GPU once, calls `prepare_on(&reading)` as Doctor does
  (`doctor.rs:73`, `:196`), and builds its `Machine` from that same reading; `Prepared` carries
  the `Inventory` it loaded and the `Build` it resolved for each project (for example
  `builds: BTreeMap<String, Build>`, filled in `Configurator::runtime`), and `setup` takes the
  model entry, the backend and `Source::Build` from it. The second load, its two errors, the
  second backend lookup, the second `current` call and the "cannot record the run's GPU"
  context go. `Prepared` then drops its `Eq` derive, since `Inventory` is only `PartialEq`;
  its one test compares fields (`gateway_config.rs:294`). Without this fix, a test of a GPU lost
  between the two reads needs Bench's test world to take a `ScriptedGpu`, `[Found, Missing]`.
- **Evidence:** read: `grep -n 'is not declared\|installed(project)\|inventory is gone\|changed during the run'`;
  `git blame` dates `bench.rs:329` to `refactor(build): look up a project's current build once`, batch 44. `review-structure`:
  `grep -rn "cannot record the run's GPU" src tests` finds `bench.rs:357` only.
- **Verification:** CONFIRMED on both axes — the double reads as stated; the first read's
  failure stops at `gateway_config.rs:56-58`, so only a second failed read reaches
  `bench.rs:357`, and Bench's tests use `FakeGpu`, a fixed reading. M30, that text replaced, in
  a clone of `docs(v2-review-fixes): record the session close of the closing review's pass 4` of 2026-10-18: the whole `cargo test` passes. Phase 4's claim that the changed texts
  are pinned does not hold for this one (`phase-4-structure-and-refactoring-report.md:385`).
  The cost of the second GPU read was not measured. The merge holds: one cause, one fix.
  `Prepared`'s `Eq` and the single read through `prepare_on` are the verifier's corrections to
  the fix, which carried the reading in `Prepared`. Pass 5, on what finding 86 adds: CONFIRMED — the
  reviewer's harness, a scratch crate on a clone of `docs(v2-review-fixes): record pass 4's second verification of the closing review` of 2026-10-18 against a copy of this machine's
  inventory and builds, n=50: `InventoryFile::load` 0.027 ms, `current("llama.cpp")` 0.018 ms,
  a later NVML read 0.077 ms, a whole `Configurator::prepare` 0.144 ms, about 0.12 ms for the
  second pass; the verifier's, on a clone of `docs(v2-review-fixes): record pass 5's performance review of the closing review` of 2026-10-18, n=200, two runs: 0.023, 0.010, 0.051
  and 0.098 ms, the second pass timed as one block 0.086 to 0.087 ms. NVML's first read, 24 to
  26 ms, is paid once a process (`OnceLock`, `nvml.rs:39`).
- **User's verdict:** confirmed — the session, under the user's delegation of 2026-10-18

### 56. `SystemHost::find`, added by the scope, repeats the `PATH` walk of `find_nvcc` next to it

- **Axes:** refactoring — raised by `review-cleanup`
- **Action:** fix
- **Severity:** —
- **Where:** `src/infrastructure/host.rs:64-77`
- **Cause:** `fix(host): ask the power profile of the UPower service alone` of 2026-10-16, the fix of the first review's finding 46, added `find(program)`, the
  walk `find_nvcc` already ran for `nvcc` with only the `/usr/local/cuda/bin/nvcc` fallback
  added.
- **Consequence:** two copies of one lookup in one struct.
- **Fix:** `find_nvcc` becomes
  `self.find("nvcc").or_else(|| Some(PathBuf::from(NVCC_FALLBACK)).filter(|path| path.is_file()))`;
  two lines saved.
- **Evidence:** read: `host.rs:64-77`; `git log -S'fn find(&self, program'` gives `fix(host): ask the power profile of the UPower service alone` of 2026-10-16.
- **Verification:** CONFIRMED — the fix applied in a clone of `docs(v2-review-fixes): record the session close of the closing review's pass 4` of 2026-10-18, same order and same
  `is_file` test: clippy and the whole `cargo test` pass.
- **User's verdict:** confirmed — the session, under the user's delegation of 2026-10-18

### 57. Doctor's Idle VRAM check sums its processes' memory by hand; `lifecycle::mib` already does it

- **Axes:** refactoring — raised by `review-cleanup`
- **Action:** fix
- **Severity:** —
- **Where:** `src/application/doctor.rs:628-634`, `src/application/lifecycle.rs:839-846`
- **Cause:** `vram_check` computes `accounted.iter().filter_map(|p| p.memory).sum()` and
  shifts it to MiB: `lifecycle::mib(accounted)`, beside `named`, which batch 48 (`refactor(gateway): describe the gateway's state in one place`)
  made doctor import for this check. The copy predates the scope; `refactor(gateway): describe the gateway's state in one place` wrote four lines of
  `vram_check` (`doctor.rs:616`, `:620`, `:641`, `:648`).
- **Consequence:** the rule "the memory processes hold, as far as the driver reports it" is
  written twice.
- **Fix:** `use super::lifecycle::{self, …}` and `format!("{} MiB", lifecycle::mib(accounted))`;
  the `gateway` local goes. The module path avoids the clash with doctor's own `mib(bytes)`.
- **Evidence:** read: `doctor.rs:628`, `:712`, `lifecycle.rs:840-846`; `git blame`.
- **Verification:** CONFIRMED — the fix applied with the module path in a clone of `docs(v2-review-fixes): record the session close of the closing review's pass 4` of 2026-10-18:
  clippy and the whole `cargo test` pass. Corrected: four lines of `refactor(gateway): describe the gateway's state in one place`, not two, and no
  renamed import needed.
- **User's verdict:** confirmed — the session, under the user's delegation of 2026-10-18

### 58. The test helper that writes an executable script under `command::exclusive()` is copied into four test modules

- **Axes:** refactoring — raised by `review-cleanup`
- **Action:** fix
- **Severity:** —
- **Where:** `src/infrastructure/cmake.rs:393-399`, `src/infrastructure/host.rs:739-745`, `src/infrastructure/gateway_process.rs:588-594`, `src/infrastructure/podman.rs:121-132`, `src/infrastructure/command.rs:173-184`
- **Cause:** `test(build): run CmakeBuilder::compile on a script, and read a captured Make build` of 2026-10-16 added `cmake.rs`'s `script`, a copy of `host.rs`'s; `gateway_process.rs`
  and podman (`test(containers): run the Podman adapter on scripts, and parse captured podman outputs` of 2026-10-16) write the same `#!/bin/sh` file, mode 0755, under the same guard.
  `command.rs` and `docs/architecture.md:121-123` make the guard a rule for every test that
  writes an executable.
- **Consequence:** the `ETXTBSY` precaution written four times; a fifth adapter test must copy
  it again, and could miss the guard.
- **Fix:** a `#[cfg(test)] pub fn script(path: &Path, body: &str)` in `command.rs` beside
  `exclusive`, taking the guard itself; cmake and host call it, `gateway_process` with
  `dir.join("llama-swap")`, podman with its `[ "$*" = '…' ] || exit 2` prefix in the body.
  `gateway_store.rs:208` and `build_store.rs:531` stay: their scripts are archived, never run.
- **Evidence:** read: the five windows; `grep -rn '#!/bin/sh' src`.
- **Verification:** CONFIRMED — `cmake.rs`'s and `host.rs`'s helpers are identical, the other
  two write the same file under the same guard, and the two exclusions are only archived by
  `tar`.
- **User's verdict:** confirmed — the session, under the user's delegation of 2026-10-18

### 59. `gateway_process`'s tests copy `host.rs`'s runtime fixture reader, where the other modules embed fixtures with `include_str!`

- **Axes:** refactoring — raised by `review-cleanup`
- **Action:** fix
- **Severity:** —
- **Where:** `src/infrastructure/gateway_process.rs:498-505`, `src/infrastructure/host.rs:519-524`
- **Cause:** `perf(gateway): place GPU processes by walking up their parents`, batch 34, added a `fixture(name)` reader to `gateway_process`'s tests,
  a copy of `host.rs`'s; podman, command, cmake, recipe, the domain's gateway, server_api and
  fakes embed their fixtures with `include_str!` or `include_bytes!`.
- **Consequence:** a second copy of the reader, and two ways in the crate to load a capture.
  No rule asks for one way (`docs/architecture.md:114`): the value is low.
- **Fix:** the four `proc-*` fixtures of `gateway_process`'s tests become `include_str!`
  constants, as `podman.rs:113-116` does, and the helper goes; `host.rs`, with 19 uses of its
  reader, keeps it.
- **Evidence:** read: both helpers; `grep -rn 'fixtures/' src`.
- **Verification:** CONFIRMED — the two helpers do the same job, `host.rs`'s panicking with the
  path; the four fixtures are used at `gateway_process.rs:516`, `:521`, `:530` and `:532`.
- **User's verdict:** confirmed — the session, under the user's delegation of 2026-10-18

### 60. Three tests compute a file's permission bits by hand beside `files::mode`, which the scope added for that purpose

- **Axes:** refactoring — raised by `review-cleanup`
- **Action:** fix
- **Severity:** —
- **Where:** `src/infrastructure/api_key.rs:126-127`, `src/infrastructure/build_store.rs:535`, `src/infrastructure/gateway_store.rs:267`, `src/infrastructure/files.rs:155-159`
- **Cause:** `fix(state): create ai-ctl's state directory at mode 0700` of 2026-10-16 added `#[cfg(test)] files::mode`; the api_key test (`feat(gateway): require ai-ctl's API key and a loopback listen address` of 2026-10-16) still
  masks `fs::metadata(..).permissions().mode()` by hand, while its module calls `files::mode`
  at `api_key.rs:141`, and build_store's and gateway_store's tests do the same.
- **Consequence:** three hand-written copies of a one-line test helper, one in a module that
  also uses it.
- **Fix:** the three assertions call `files::mode`. Only the api_key tests drop their
  `PermissionsExt` import: build_store's and gateway_store's still call
  `Permissions::from_mode` (`build_store.rs:531`, `gateway_store.rs:209`) and name the helper
  by path (`crate::infrastructure::files::mode`, or a `use`); build_store's call takes
  `&binary`.
- **Evidence:** read: `grep -rn 'permissions().mode()' src`.
- **Verification:** CONFIRMED, with the fix corrected — the reviewer's fix as written does not
  compile in a clone of `docs(v2-review-fixes): record the session close of the closing review's pass 4` of 2026-10-18 (E0599 twice, `from_mode` without its import; E0433 twice,
  `files` not in scope); corrected as above, clippy `-D warnings` and the 635 library tests
  pass.
- **User's verdict:** confirmed — the session, under the user's delegation of 2026-10-18

### 23. No test runs `up` or `down` while another ai-ctl holds the start/stop lock

- **Axes:** tests — raised by `review-tests`
- **Action:** fix
- **Severity:** —
- **Where:** `src/application/lifecycle.rs:225`, `src/application/lifecycle.rs:305`
- **Cause:** the lifecycle tests never set `FakeGateway.locked`; only `build.rs`'s prune test
  does (`build.rs:1242`).
- **Consequence:** M22 deletes `let _lock = self.gateway.lock()?;` from `up`, and M23 from
  `down`; both pass every test. Two concurrent `ai-ctl up` could then each start a
  llama-swap, or `down` could signal a gateway that `up` is still starting.
- **Fix:** add `up_and_down_wait_for_no_one`: with `world.gateway.locked = true`, `up` fails
  with "another ai-ctl is starting or stopping the gateway" and `started == 0`; on
  `serving(0)` with the lock held, `down` fails the same way, with no signal sent and no
  unload. Finding 49, `status`'s lock, extends this test.
- **Evidence:** reproduced by the reviewer: M22 and M23 survive `cargo test`;
  `review_up_and_down_wait_for_no_one` passes on the code and fails on each.
- **Verification:** CONFIRMED — M22 and M23 re-run from the verifier's own definitions: each
  survives the whole suite and fails the proposed test alone. Fit to land.
- **User's verdict:** confirmed — the session, under the user's delegation of 2026-10-18

### 24. The kill of a llama-swap whose record cannot be written is untested

- **Axes:** tests — raised by `review-tests`
- **Action:** fix
- **Severity:** —
- **Where:** `src/infrastructure/gateway_process.rs:225-230`
- **Cause:** the start tests fail only before the spawn (an unusable key, a missing binary); no
  test makes the record write fail after the child has started.
- **Consequence:** M6 deletes `let _ = child.kill(); let _ = child.wait();`; every test
  passes, yet a started llama-swap would be left running with no record, which `down` and
  `status` cannot see.
- **Fix:** a script `exec -a "$0" sleep 30` as llama-swap, and a non-empty directory in place
  of `gateway/gateway.json`: `start` fails with "cannot write …/gateway.json"; then scan /proc
  for a command line naming the script, kill any found, and assert none was left. The
  verifier: `exec -a "$0"` is redundant, since the shebang already names the script, but
  harmless; the /proc scan is new code, not a copy of `Procfs`.
- **Evidence:** reproduced by the reviewer: M6 survives;
  `review_a_gateway_whose_record_cannot_be_written_is_stopped` passes on the code and fails on
  M6 with "left running: [465420]", and its cleanup killed that process.
- **Verification:** CONFIRMED — M6 re-run: it survives, and the test fails on it, then kills the
  process it found; `pgrep` found nothing after. No timing dependency: `spawn` returns after
  the exec, so the /proc scan sees the child at once, and on the code `kill` and `wait` are
  synchronous. A cause apart from finding 18: 18 is an interruption between the spawn and the
  record, which this cleanup cannot cover; 24 is that window's error path, which exists and is
  untested. A fix of 18 that reorders the spawn and the record may rewrite this path; the test
  asserts behaviour, so it stays. Fit to land.
- **User's verdict:** confirmed — the session, under the user's delegation of 2026-10-18

### 25. The write error of `logs` is untested, so `logs -f | head` could follow forever

- **Axes:** tests — raised by `review-tests`
- **Action:** fix
- **Severity:** —
- **Where:** `src/application/lifecycle.rs:484-486`
- **Cause:** every `log` test writes into a `Vec`, and no writer fails; presentation's
  `closed_pipe` is tested only on an error built by hand (`presentation/mod.rs:192`).
- **Consequence:** M4 ignores the result of `out.write_all(..).and_then(flush)`; every test
  passes, but `ai-ctl logs -f | head` would then keep following the gateway after its reader
  has gone.
- **Fix:** run `log(true, …)` on `writing([…])` with a writer that returns `BrokenPipe`;
  assert "cannot write the log" with `BrokenPipe` in its chain, and that the gateway's two
  pending changes were never read. The verifier: the test's `ClosedPipe` is written again in
  finding 26's test; one shared failing writer would do, for example in `application::fakes`.
- **Evidence:** reproduced by the reviewer: M4 survives;
  `review_a_log_whose_reader_has_gone_stops_at_once` passes on the code and fails on M4.
- **Verification:** CONFIRMED — M4 re-run: it survives and fails the test alone; no writer in
  `src/` fails. Fit to land.
- **User's verdict:** confirmed — the session, under the user's delegation of 2026-10-18

### 26. The write error of a streamed model log (`logs <model>`) is untested

- **Axes:** tests — raised by `review-tests`
- **Action:** fix
- **Severity:** —
- **Where:** `src/infrastructure/http.rs:440-442`
- **Cause:** `copy_stream`'s tests always copy into a `Vec`: a function and a fix apart from
  finding 25's.
- **Consequence:** M26 ignores the write result in `copy_stream`; every test passes, but
  `ai-ctl logs <model> | head` would then hold the stream open while the model runs.
- **Fix:** call `copy_stream(&mut Trickle(STREAM), &mut ClosedPipe, 1024)` and assert "cannot
  write the stream" with `BrokenPipe` in the chain, the failing writer shared with finding
  25's test.
- **Evidence:** reproduced by the reviewer: M26 survives;
  `review_a_stream_whose_reader_has_gone_ends_with_the_write_error` passes on the code and
  fails on M26.
- **Verification:** CONFIRMED — M26 re-run: it survives and fails the test alone. Fit to land.
- **User's verdict:** confirmed — the session, under the user's delegation of 2026-10-18

### 27. The time llama-swap gets between SIGTERM and SIGKILL is untested

- **Axes:** tests — raised by `review-tests`
- **Action:** fix
- **Severity:** —
- **Where:** `src/application/lifecycle.rs:37`, `src/application/lifecycle.rs:778-783`
- **Cause:** the stubborn-gateway tests (`lifecycle.rs:1359` and `:1540`) assert the order
  `[Terminate, Kill]`, never the time between the two signals.
- **Consequence:** M1 sets `STOP_POLLS` to 4, which SIGKILLs llama-swap after 0.4 s instead of
  40 s; every test passes. llama-swap would be killed while still stopping its models, which
  takes 30 s plus 5 s, leaving backends on the GPU.
- **Fix:** on `serving(0)` with `stubborn = true`, run `down` and assert the fake clock moved
  at least 35 s, the requirement (`lifecycle.rs:35`), rather than the constant's 40 s.
- **Evidence:** reproduced by the reviewer: M1 survives;
  `review_down_gives_llama_swap_40_s_after_sigterm_before_sigkill` passes on the code and
  fails on M1, measuring 400 ms.
- **Verification:** CONFIRMED — M1 re-run: it survives, and the test fails with "left: 400ms
  right: 40s". The fake clock is thread-local (`fakes.rs:37-52`), so no wall-clock
  dependency. Fit to land; the verifier prefers "at least 35 s" to the exact 40 s.
- **User's verdict:** confirmed — the session, under the user's delegation of 2026-10-18

### 28. The clamps that hold `up`'s start wait to 30 s are untested, because every test's checks divide 30 s evenly

- **Axes:** tests — raised by `review-tests`
- **Action:** fix
- **Severity:** —
- **Where:** `src/application/lifecycle.rs:719`, `src/application/lifecycle.rs:725`, `src/application/lifecycle.rs:1258`
- **Cause:** `a_gateway_that_accepts_and_never_answers_holds_up_30_s` uses checks of 500 ms and
  pauses of 100 ms, and the other start tests instant checks and pauses of 100 ms: both steps
  divide 30 s evenly, so no check or pause starts with less time left than its full length.
  Commit `fix(lifecycle): bound up's and down's waits by a deadline` of 2026-10-16 says "each health check waits the time left or 500 ms at most", and no
  input checks it.
- **Consequence:** M2 replaces `time_left.min(CHECK)` with `CHECK`, and M3 `POLL.min(time_left)`
  with `POLL`, in `wait_until_healthy`; both pass every test. `up` would then wait up to
  30.5 s, past the 30 s it promises.
- **Fix:** a first check of 250 ms, then checks that hang for their whole timeout; and a first
  check of 30 ms, then checks that fail at once; in both, the failed `up` ends exactly 30 s
  after it started. The verifier drops the reviewer's test-local `SlowApi`, which writes again
  the four `GatewayApi` methods that `FakeApi` (`fakes.rs:740-797`) has: give `FakeApi` a
  field for the first check's duration, and reuse `hangs` and `silent()` for the later checks.
- **Evidence:** reproduced by the reviewer: M2 and M3 survive; `review_a_start_never_waits_past_its_30_s`
  passes on the code and fails on M2 (30.25 s) and on M3 (30.03 s).
- **Verification:** CONFIRMED — M2 and M3 re-run: both survive, and the test measures 30.25 s
  and 30.03 s on them, with the thread-local fake clock only. Not fit to land as is: write it
  on `FakeApi`, as the fix says.
- **User's verdict:** confirmed — the session, under the user's delegation of 2026-10-18

### 29. doctor's Gateway check when the gateway's record cannot be read is untested

- **Axes:** tests — raised by `review-tests`
- **Action:** fix
- **Severity:** —
- **Where:** `src/application/doctor.rs:154`
- **Cause:** `FakeGateway::record` and `liveness` never fail, so the `Err` arm never runs.
- **Consequence:** M5 changes the arm to `Err(_) => {}`; every test passes, yet a broken
  `gateway.json` would drop the Gateway check entirely, and doctor would report healthy.
- **Fix:** `record_fails` on `FakeGateway`, with the real adapter's kind of error; on
  `GatewaySide::new()` with it set, the Gateway check is `(Fail, "cannot read
  /state/gateway/gateway.json: Permission denied (os error 13)", None)` and the report has
  failures.
- **Evidence:** reproduced by the reviewer: M5 survives;
  `review_a_gateway_record_that_cannot_be_read_fails_the_gateway_check` passes on the code and
  fails on M5.
- **Verification:** CONFIRMED — M5 re-run: it survives and fails the test. The real `record()`
  (`gateway_process.rs:144-156`) fails on a read error and on malformed JSON, so a broken
  `gateway.json` reaches the arm; the double follows the `<method>_fails` convention. Fit to
  land.
- **User's verdict:** confirmed — the session, under the user's delegation of 2026-10-18

### 30. The overhead's direct address is only tested with one model running

- **Axes:** tests — raised by `review-tests`
- **Action:** fix
- **Severity:** —
- **Where:** `src/application/bench.rs:414-417`
- **Cause:** the bench World lists only the measured model in `/running`; llama-swap lists
  every running model, and the matrix lets models share the GPU.
- **Consequence:** M9 replaces `.find(|entry| entry.id == model)` with `.next()`; every test
  passes, yet the overhead would then be measured against another model's server, without an
  error.
- **Fix:** list another model first, with proxy `127.0.0.1:5800`, then the measured model with
  `own()`; run the overhead scenario; assert that no request goes to 5800 and that 50 go to
  `own()`.
- **Evidence:** reproduced by the reviewer: M9 survives;
  `review_the_overhead_goes_to_the_measured_models_own_server` passes on the code and fails on
  M9.
- **Verification:** CONFIRMED — M9, at `bench.rs:416`, the third of three identical `.find`
  calls, re-run: it survives and fails the test. Fit to land.
- **User's verdict:** confirmed — the session, under the user's delegation of 2026-10-18

### 31. The mapping from `--scenario` to the scenario that runs is untested

- **Axes:** tests — raised by `review-tests`
- **Action:** fix
- **Severity:** —
- **Where:** `src/presentation/cli.rs:176-185`, `src/presentation/bench.rs:29`
- **Cause:** `tests/cli.rs:324` stops at `ScenarioArg`, and nothing calls `Scenario::from`.
- **Consequence:** M10 maps `ScenarioArg::Session` to `Scenario::Prefill`; every test passes,
  yet `bench run m --scenario session` would then run the prefill scenario.
- **Fix:** in `tests/cli.rs`, for each `ScenarioArg::value_variants()`, assert that
  `Scenario::from(arg).name()` equals the variant's clap name.
- **Evidence:** reproduced by the reviewer: M10 survives;
  `review_each_scenario_option_runs_the_scenario_of_its_name` passes on the code and fails on
  M10.
- **Verification:** CONFIRMED — M10, at `cli.rs:181`, re-run: it survives and fails the
  `tests/cli.rs` test. Fit to land.
- **User's verdict:** confirmed — the session, under the user's delegation of 2026-10-18

### 32. The exit status 0 of `status` with a running gateway is untested

- **Axes:** tests — raised by `review-tests`
- **Action:** fix
- **Severity:** —
- **Where:** `src/presentation/lifecycle.rs:66-70`
- **Cause:** every binary test of `tests/cli.rs` runs on an empty machine, so only the exit 3
  branch is covered; `tests/gateway_lifecycle.rs` calls the use case, not the binary.
- **Consequence:** M16 makes `status` always exit 3; every test passes, and scripts that check
  whether the gateway runs would always read "stopped".
- **Fix:** in `tests/cli.rs`, link `<tmp>/llama-swap` to `/usr/bin/sleep`, a link rather than
  a copy to avoid ETXTBSY, and run it; write `<tmp>/.local/state/ai-ctl/gateway/gateway.json`
  with its PID, the link and a free port; run `ai-ctl status`, assert exit 0 and the "Gateway
  running on http://…" line; kill the process. The verifier's remarks: if `ai_ctl()` itself
  panics, the `sleep 60` outlives the test; the port is free only by chance, which bears on the
  warning alone.
- **Evidence:** reproduced by the reviewer: M16 survives;
  `review_status_with_a_running_gateway_exits_0` passes on the code and fails on M16 (exit 3).
- **Verification:** CONFIRMED — M16 re-run: it survives, and the test fails with exit 3 on it
  and passes on the code. The process is killed and waited for before the asserts; `pgrep`
  found no `sleep` left. The exit status depends only on `status.gateway`
  (`lifecycle.rs:347-352`); the binary reads NVML, as the `status` test at `tests/cli.rs:305`
  already does, and a GPU that cannot be read gives a warning only, so the test does not
  depend on a GPU. Liveness compares the canonical `/proc/<pid>/exe`, which the link
  satisfies. Fit to land.
- **User's verdict:** confirmed — the session, under the user's delegation of 2026-10-18

### 33. `bench run` succeeding when its run cannot be saved is untested

- **Axes:** tests — raised by `review-tests`
- **Action:** fix
- **Severity:** —
- **Where:** `src/presentation/bench.rs:55-63`
- **Cause:** only the use case is tested with `FakeRuns.full`; no presentation test builds
  `Services`, since `application::fakes` is private to `application`.
- **Consequence:** M28 prints the "Saved as" line only when the save succeeded, and otherwise
  returns success; every test passes, yet a run that was not saved would exit 0 and lose the
  error.
- **Fix:** make `fakes` `pub(crate)` under `cfg(test)`; build a test `Services` from the fakes,
  with three unreachable stubs for Sources, Compiler and Downloads and `FakeRuns { full: true
  }`; run `BenchAction::Run` for the prefill scenario and assert "the run is printed above, but
  cannot be saved: cannot write …: No space left on device (os error 28)". The verifier adds:
  the handler prints through `super::print` (`presentation/mod.rs:152`) and
  `writeln!(stderr)`, which bypass libtest's capture, so the whole run's rendering spills into
  `cargo test`'s output: capture or redirect it, or test a function that takes a writer. And
  update `docs/architecture.md:115`, which lists presentation's tests as rendering tests and
  `tests/`, and the comment at `fakes.rs:1`, "shared by the use cases' tests".
- **Evidence:** reproduced by the reviewer: M28 survives;
  `review_services::review_a_run_that_cannot_be_saved_fails_the_command` passes on the code and
  fails on M28.
- **Verification:** CONFIRMED — M28, at `presentation/bench.rs:55-56`, re-run: it survives and
  fails the test. The layering holds: `docs/architecture.md:28` lets presentation depend on
  application and forbids only infrastructure. Not fit to land as is: its output spills, as
  the fix says.
- **User's verdict:** confirmed — the session, under the user's delegation of 2026-10-18

### 34. `publish` putting the old build back after a failed replacement is untested

- **Axes:** tests — raised by `review-tests`
- **Action:** fix
- **Severity:** —
- **Where:** `src/infrastructure/build_store.rs:243-246`
- **Cause:** the replace tests take the success path only.
- **Consequence:** M17 deletes `let _ = fs::rename(&old, &target);`; every test passes, yet
  the installed build would then stay hidden as `.replaced-…` until the next build deletes it.
- **Fix:** install b1; stage b1 again and delete the staged directory; call `publish(&staged,
  true)`; assert "cannot install …/b1", that `builds()` gives the old b1, and that the entries
  are `["b1"]`.
- **Evidence:** reproduced by the reviewer: M17 survives;
  `review_a_replacement_that_cannot_be_installed_puts_the_old_build_back` passes on the code
  and fails on M17.
- **Verification:** CONFIRMED — M17, at `build_store.rs:245`, re-run: it survives and fails the
  test. A cause apart from finding 17: 17 is an interruption between the two renames, 34 the
  failure path of the second rename, which exists and is untested. A fix of 17 that changes
  how `publish` swaps may remove this line; the test asserts behaviour, so it stays. Fit to
  land.
- **User's verdict:** confirmed — the session, under the user's delegation of 2026-10-18

### 35. A failed fetch of an existing clone is untested

- **Axes:** tests — raised by `review-tests`
- **Action:** fix
- **Severity:** —
- **Where:** `src/infrastructure/git.rs:77-81`
- **Cause:** `release_tags_are_read_from_the_clone` drops the upstream after the first fetch,
  but never fetches again.
- **Consequence:** M18 ignores the fetch's result; every test passes, yet with the network down
  `build llama.cpp` would build an outdated "latest" without saying so.
- **Fix:** fetch once, drop the upstream, fetch again, and assert the error starts with
  "cannot fetch <url>".
- **Evidence:** reproduced by the reviewer: M18 survives; `review_a_fetch_that_fails_is_an_error`
  passes on the code and fails on M18.
- **Verification:** CONFIRMED — M18, `git.rs:81`'s `?` replaced by `.ok()`, re-run: it survives
  and fails the test, which uses a local upstream, no network. Fit to land.
- **User's verdict:** confirmed — the session, under the user's delegation of 2026-10-18

### 36. `compile` with a failed CMake configure is untested

- **Axes:** tests — raised by `review-tests`
- **Action:** fix
- **Severity:** —
- **Where:** `src/infrastructure/cmake.rs:75`
- **Cause:** the compile test's script always configures successfully;
  `a_failed_step_quotes_the_end_of_the_log` calls `Log::stream` directly, on the build step
  only.
- **Consequence:** M19 ignores the configure result; every test passes, yet a failed configure
  would go on to the build and report another error, losing CMake's message.
- **Fix:** a cmake script that fails unless `--build` is given; assert that `compile` fails
  with "cmake configure failed (exit status: 1)", ending with the script's stderr line, and
  never called `--build`.
- **Evidence:** reproduced by the reviewer: M19 survives;
  `review_a_failed_configuration_builds_nothing` passes on the code and fails on M19.
- **Verification:** CONFIRMED — M19 re-run: it survives and fails the test, whose script is
  written under `command::exclusive()` (`cmake.rs:396`). Fit to land.
- **User's verdict:** confirmed — the session, under the user's delegation of 2026-10-18

### 37. A g++ that fails to report its version is untested

- **Axes:** tests — raised by `review-tests`
- **Action:** fix
- **Severity:** —
- **Where:** `src/infrastructure/host.rs:161-165`
- **Cause:** the `host_compilers` tests use scripts that succeed only.
- **Consequence:** M20 maps `(true, Some(error))` to `Missing`; every test passes, yet doctor
  would say "no g++ found; sudo dnf install gcc-c++" of a g++ that is installed but broken.
- **Fix:** put on PATH a g++ script that prints a loader error and exits 127; assert
  `Probed::Failed("g++: g++: error while loading shared libraries: libisl.so.15")`.
- **Evidence:** reproduced by the reviewer: M20 survives;
  `review_a_gxx_that_cannot_tell_its_version_is_a_failed_probe` passes on the code and fails on
  M20.
- **Verification:** CONFIRMED — M20, at `host.rs:163`, re-run: it survives and fails the test;
  doctor's remedy for `Missing` compilers is "sudo dnf install gcc-c++" (`doctor.rs:1276`).
  Fit to land.
- **User's verdict:** confirmed — the session, under the user's delegation of 2026-10-18

### 38. `up` on an address that cannot be bound is untested

- **Axes:** tests — raised by `review-tests`
- **Action:** fix
- **Severity:** —
- **Where:** `src/application/lifecycle.rs:245`
- **Cause:** `FakePorts.is_free_fails` is set by doctor's tests only (`doctor.rs:1870`).
- **Consequence:** M21 replaces `is_free(listen)?` with `.unwrap_or(true)`; every test passes,
  yet `up` would start llama-swap on an address it cannot bind.
- **Fix:** with `world.ports.is_free_fails = true`, assert that `up` fails with "cannot bind
  127.0.0.1:8080: Cannot assign requested address (os error 99)" and `started == 0`.
- **Evidence:** reproduced by the reviewer: M21 survives; `review_up_refuses_an_address_it_cannot_bind`
  passes on the code and fails on M21.
- **Verification:** CONFIRMED — M21 re-run: it survives and fails the test. Fit to land.
- **User's verdict:** confirmed — the session, under the user's delegation of 2026-10-18

### 39. A `prune` whose removal fails is untested

- **Axes:** tests — raised by `review-tests`
- **Action:** fix
- **Severity:** —
- **Where:** `src/application/build.rs:427-433`
- **Cause:** `build.rs`'s test `FakeStore` never fails to remove.
- **Consequence:** M8 ignores `store.remove`'s result; every test passes, yet prune would
  report builds as removed that are still there.
- **Fix:** `remove_fails` on the test `FakeStore`; with b1 and b2, b2 current, run
  `prune(keep 0)`; assert "cannot remove llama.cpp b1: cannot remove
  /data/builds/llama.cpp/b1: Permission denied (os error 13)" and that both builds are left.
  The expected text changes with finding 21's fix, so the two go together.
- **Evidence:** reproduced by the reviewer: M8 survives;
  `review_a_build_prune_cannot_remove_ends_it_with_its_name` passes on the code and fails on M8.
- **Verification:** CONFIRMED — M8, at `build.rs:428-430`, re-run: it survives and fails the
  test. A cause apart from finding 21: 21 is the wording of the error after a partial prune, 39
  that the error path never runs; only the expected text depends on 21. Fit to land.
- **User's verdict:** confirmed — the session, under the user's delegation of 2026-10-18

### 40. `build use` while another ai-ctl holds the builds' lock is untested

- **Axes:** tests — raised by `review-tests`
- **Action:** fix
- **Severity:** —
- **Where:** `src/application/build.rs:353`
- **Cause:** the `use_build` tests always run on an unlocked store; the builder and prune have
  lock tests, `use` has none.
- **Consequence:** M24 deletes `let _lock = store.lock()?;` in `use_build`; every test passes,
  yet `build use` could switch `current` during a build or a prune.
- **Fix:** with `FakeStore { locked: true, .. }`, assert that `use_build` fails with "another
  ai-ctl…" and leaves `current` unchanged.
- **Evidence:** reproduced by the reviewer: M24 survives; `review_use_waits_for_no_one` passes
  on the code and fails on M24.
- **Verification:** CONFIRMED — M24 re-run: it survives and fails the test. Fit to land.
- **User's verdict:** confirmed — the session, under the user's delegation of 2026-10-18

### 41. A signal that cannot be sent is untested

- **Axes:** tests — raised by `review-tests`
- **Action:** fix
- **Severity:** —
- **Where:** `src/application/lifecycle.rs:777`
- **Cause:** `FakeGateway::signal` never fails.
- **Consequence:** M14 ignores `signal`'s result; every test passes, yet when the kill is
  refused, `down` would poll for 45 s and end with "PID … still runs after SIGKILL" instead of
  the kill's own error.
- **Fix:** `signal_fails` on `FakeGateway`; on `serving(0)`, assert that `down` fails at once
  with the kill error, that the clock did not move, and that the record is kept.
- **Evidence:** reproduced by the reviewer: M14 survives;
  `review_a_signal_that_cannot_be_sent_ends_down_at_once` passes on the code and fails on M14.
- **Verification:** CONFIRMED — M14 re-run: it survives, and the test fails on it with "PID 1234
  still runs after SIGKILL" instead of the kill error, which confirms the consequence. Fit to
  land.
- **User's verdict:** confirmed — the session, under the user's delegation of 2026-10-18

### 42. A program path that is not UTF-8 is untested, because every input path is UTF-8

- **Axes:** tests — raised by `review-tests`
- **Action:** fix
- **Severity:** —
- **Where:** `src/domain/gateway_config.rs:414-420`
- **Cause:** the `gateway_config` tests use UTF-8 paths only, so `ConfigError::NonUtf8Path` is
  never produced. Such a path is a real input: a build's path comes from HOME or the XDG data
  directory, which can hold Latin-1 bytes.
- **Consequence:** M11 writes the path through `to_string_lossy`; every test passes, yet the
  configuration would start a U+FFFD path that does not exist.
- **Fix:** give `Runtime::Native` the path `/home/s\xe9b/…/llama-server` and assert
  `Err(NonUtf8Path { model: "qwen", path })`.
- **Evidence:** reproduced by the reviewer: M11 survives;
  `review_a_program_path_that_is_not_utf8_is_refused` passes on the code and fails on M11.
- **Verification:** CONFIRMED — M11 re-run: it survives and fails the test. Fit to land.
- **User's verdict:** confirmed — the session, under the user's delegation of 2026-10-18

### 43. `init` with a models directory that is not UTF-8 is untested

- **Axes:** tests — raised by `review-tests`
- **Action:** fix
- **Severity:** —
- **Where:** `src/infrastructure/inventory_file.rs:77-79`
- **Cause:** the render tests use UTF-8 directories only.
- **Consequence:** M27 renders the directory through `to_string_lossy`; every test passes, yet
  `init` would install a `models_dir` that does not exist.
- **Fix:** call `render(SHIPPED, "/data/mod\xe8les")` and assert the error
  "/data/mod\u{FFFD}les is not valid UTF-8".
- **Evidence:** reproduced by the reviewer: M27 survives;
  `review_a_models_directory_that_is_not_utf8_is_refused` passes on the code and fails on M27.
- **Verification:** CONFIRMED — M27 re-run: it survives and fails the test. `--models-dir` is a
  `PathBuf` (`cli.rs:27`), so such a path can arrive. Fit to land.
- **User's verdict:** confirmed — the session, under the user's delegation of 2026-10-18

### 44. The rule that a tag wins over a branch is untested, because no input gives a tag and a branch the same name

- **Axes:** tests — raised by `review-tests`
- **Action:** fix
- **Severity:** —
- **Where:** `src/infrastructure/git.rs:90-94`
- **Cause:** the upstream fixture's tags (b1, b2, b10) and branch (feature) never share a name.
- **Consequence:** M12 puts `refs/remotes/origin/{reference}` before `refs/tags/{reference}`;
  every test passes, yet `--ref b1` would build a branch named b1, and name the build after its
  hash.
- **Fix:** create a branch `b1` on the feature commit in the upstream; check out `b1` and
  assert that the commit is the tag's.
- **Evidence:** reproduced by the reviewer: M12 survives;
  `review_a_tag_wins_over_a_branch_of_the_same_name` passes on the code and fails on M12.
- **Verification:** CONFIRMED — M12 re-run: it survives and fails the test, which uses a local
  upstream. The consequence's "name the build after its hash" was not checked. Fit to land.
- **User's verdict:** confirmed — the session, under the user's delegation of 2026-10-18

### 45. Reinstalling the current llama-swap re-pointing the `current` link is untested

- **Axes:** tests — raised by `review-tests`
- **Action:** fix
- **Severity:** —
- **Where:** `src/application/gateway.rs:125`
- **Cause:** `the_current_release_is_left_as_it_is` never makes `set_current` fail, so whether
  it is called for an `Unchanged` outcome is never observed.
- **Consequence:** M7 changes the condition to `if true`; every test passes, yet `gateway
  install` of the release already current would rewrite the link, and fail on a store where it
  cannot.
- **Fix:** make the pinned release installed and current, with `set_current_fails = true`, and
  assert that the outcome is `Unchanged`.
- **Evidence:** reproduced by the reviewer: M7 survives;
  `review_the_current_release_is_not_linked_again` passes on the code and fails on M7.
- **Verification:** CONFIRMED — M7 re-run: it survives and fails the test. Fit to land.
- **User's verdict:** confirmed — the session, under the user's delegation of 2026-10-18

### 46. The warning when the GPU monitor cannot finish is untested

- **Axes:** tests — raised by `review-tests`
- **Action:** fix
- **Severity:** —
- **Where:** `src/application/bench.rs:243`
- **Cause:** `FakeMonitoring::finish` never fails.
- **Consequence:** M15 changes the arm to `Err(_) => {}`; every test passes, yet a run would
  silently record no GPU readings when the monitor thread fails.
- **Fix:** `finish_fails` on `FakeMonitor`; assert that the run succeeds with no GPU readings
  and the warning "the GPU's reading failed".
- **Evidence:** reproduced by the reviewer: M15 survives;
  `review_a_gpu_reading_that_cannot_be_finished_is_a_warning` passes on the code and fails on
  M15.
- **Verification:** CONFIRMED — M15 re-run: it survives and fails the test. Fit to land.
- **User's verdict:** confirmed — the session, under the user's delegation of 2026-10-18

### 47. doctor's text for a running gateway with no model loaded is untested

- **Axes:** tests — raised by `review-tests`
- **Action:** fix
- **Severity:** —
- **Where:** `src/application/doctor.rs:521`
- **Cause:** the running-gateway doctor tests use models, an outdated configuration or no API,
  never `Some([])`.
- **Consequence:** M13 deletes the `Some([])` arm; every test passes, yet the detail would read
  "running on http://…(PID 1234): " with nothing after it.
- **Fix:** on `GatewaySide::serving()`, assert that the Gateway check is `(Ok, "running on
  http://127.0.0.1:8080 (PID 1234), no model loaded", None)`.
- **Evidence:** reproduced by the reviewer: M13 survives;
  `review_a_running_gateway_without_a_model_says_so` passes on the code and fails on M13.
- **Verification:** CONFIRMED — M13 re-run: it survives and fails the test. Fit to land.
- **User's verdict:** confirmed — the session, under the user's delegation of 2026-10-18

### 48. The compare table's "change" when the first run's value is 0 is untested

- **Axes:** tests — raised by `review-tests`
- **Action:** fix
- **Severity:** —
- **Where:** `src/presentation/compare.rs:124`
- **Cause:** in the compare tests, no first value of a relative measure is ever 0.
- **Consequence:** M25 drops the `first.mean != 0.0` guard; every test passes, yet the change
  column would print "+inf%" or "NaN%".
- **Fix:** assert `change(Some(&Summary::of([0.0])), Some(&five), Relative) == "–"`, and that
  the reverse gives "-100.0%".
- **Evidence:** reproduced by the reviewer: M25 survives; `review_a_change_from_zero_is_not_told`
  passes on the code and fails on M25.
- **Verification:** CONFIRMED — M25 re-run: it survives and fails the test. The second assert
  does not guard the mutant; it stays as a case of the same rule. Fit to land.
- **User's verdict:** confirmed — the session, under the user's delegation of 2026-10-18

### 49. No test runs `status` while another ai-ctl holds the start/stop lock

- **Axes:** tests — a lead of pass 3's `review-verifier`
- **Action:** fix
- **Severity:** —
- **Where:** `src/application/lifecycle.rs:348`, `src/application/lifecycle.rs:515-534`
- **Cause:** `status` takes the gateway's lock because `running()` removes a record whose
  process is gone. No lifecycle test sets `FakeGateway.locked`; only prune's test does
  (`build.rs:1243`), and none of the tests proposed for finding 23 runs `status`.
- **Consequence:** M29 deletes `let _lock = self.gateway.lock()?;` from `status`, and the whole
  suite passes. A `status` without the lock could remove the record while another ai-ctl starts
  or stops the gateway. Finding 13 is about the same lock, held for `status`'s whole
  observation.
- **Fix:** extend finding 23's test to `status`, beside
  `status_reports_a_gateway_killed_outside_ai_ctl_as_stopped_and_removes_its_record`
  (`lifecycle.rs:1848`): with `world.gateway.locked = true` and a record of PID 1234 whose
  liveness is `Exited`, `status` fails with "another ai-ctl is starting or stopping the
  gateway", sends no event, and leaves the record. The gone process is what makes "leaves the
  record" observable.
- **Evidence:** reproduced by pass 3's verifier: M29 survives the whole suite in a clone of
  `docs(v2-review-fixes): record the session close of the closing review's pass 3` of 2026-10-18.
- **Verification:** CONFIRMED — M29 re-run from pass 3's definitions in a clone of `docs(v2-review-fixes): record the session close of the closing review's pass 4` of 2026-10-18:
  it survives, 663 passed and `up_a_proxied_request_and_down ... ok`. The test above, written
  by the verifier, passes on the code and fails on M29 at `lifecycle.rs:1872`, "status must
  fail". Fit to land.
- **User's verdict:** confirmed — the session, under the user's delegation of 2026-10-18

### 81. `an_error_answer_names_the_endpoint` serves the chat endpoint's capture as the answer of `/upstream/<model>/props`

- **Axes:** tests — a lead of pass 4's second `review-verifier`
- **Action:** fix
- **Severity:** —
- **Where:** `src/infrastructure/server_api.rs:419-430`, `tests/fixtures/README.md`
- **Cause:** the test serves `llama-swap-263-chat-unknown-model.http`, 404 "no router for
  requested model", the capture of a chat request, as the answer to `GET /upstream/nope/props`.
- **Consequence:** llama-swap v263 answers that request with another body, 404 "model not
  found": the fixture is not a capture of the request the test makes, against CLAUDE.md's rule
  that parsers are tested on outputs captured by a probe.
- **Fix:** add the capture of `GET /upstream/nope/props` as
  `tests/fixtures/llama-swap-263-upstream-props-unknown-model.http`, with its row in
  `tests/fixtures/README.md`'s llama-swap 263 table ("`GET /upstream/nope/props`, a model
  llama-swap does not know: its 404"); serve it in the test and expect "answered 404: model not
  found". It touches finding 71: 71 serves an existing fixture as the answer of a `GET /x` at
  `http.rs:711`, and this GET capture fits that request better than the chat capture;
  `chat_api.rs:310` keeps the chat capture rightly. The two are planned together.
- **Evidence:** reported by pass 4's second verifier from its probe of llama-swap v263 alone on
  a loopback port for finding 71.
- **Verification:** CONFIRMED — llama-swap v263 alone on a free loopback port, `touch` as the
  model's command, raw HTTP/1.0 with the key: `GET /upstream/nope/props` and `POST
  /upstream/nope/tokenize` both give 404 `{"src":"llama-swap","error":{"message":"model not
  found",…,"code":"not_found"}}`, 121 bytes; only `POST /v1/chat/completions` with
  `"model":"nope"` gives "no router for requested model", the fixture's body. With the new
  capture, the test passes in a copy. The fix is the verifier's; the capture,
  `props-nope.http`, and the copy's diff are kept outside the repository (Checks).
- **User's verdict:** confirmed — the session, under the user's delegation of 2026-10-18

## Coverage

| Axis | Reviewed by | Left out, and why |
|---|---|---|
| Architecture | pass 4, `review-structure` at `high`, with `review-cleanup`'s words on splitting `GatewayProcess`; verified in pass 4, `review-verifier` at `high` | the domain modules `bench.rs`, `build.rs`, `machine.rs`, `prompt.rs`, `recipe.rs`, `stats.rs` and `timestamp.rs` checked by search only: no I/O, no import of an outer layer, `thiserror` |
| Dead and legacy code | pass 4, `review-cleanup` at `high`; verified in pass 4, `review-verifier` at `high` | `bench compare` not run: finding 50's "–" for a format-1 run's policy share rests on the code; the dead-item search finds unused items only, not a branch an earlier guard makes unreachable (`review-structure`'s finding 69, under To Verify) |
| Refactoring | pass 4, `review-cleanup` at `high`; verified in pass 4, `review-verifier` at `high` | the fixes of findings 53, 54, 58 and 59 not applied by the verifier, read only; the cost of the bench's second GPU read (finding 55) not measured in pass 4; measured in pass 5, finding 86 merged into 55 |
| Conventions | pass 4, `review-structure` at `high`; verified in pass 4, `review-verifier` at `high`; finding 89, a lead of pass 5's verifier, verified in pass 5 | the adapters `chat_api`, `server_api`, `gateway_api`, `git`, `host`, `nvml`, `podman`, `download` and `gateway_store`, and `application/toolchain.rs`, read only where a search led, not line by line |
| Bugs | pass 1, `review-correctness` at `high`; verified in pass 2; finding 22 from pass 3's session, verified in pass 3 | `up`, `down`, `build` and `gateway install` not run, by the review's rules: finding 8 rests on reading; podman with the GPU device not run by pass 3's verifier: crun's failure in finding 22 rests on the session's run |
| Error handling | pass 1, `review-correctness` at `high`; verified in pass 2; findings 80 and 82, leads of pass 4's second verifier, verified in pass 5 | `bench list` and `bench compare` not run by the verifier: finding 10's run half and finding 16's `bench compare <id> <id>` rest on a test in a copy and on reading; NVML hanging, finding 82's worst case, cannot be produced here |
| Security | pass 1, `review-security` at `high`; verified in pass 2; finding 1 settled in pass 3's session by a measurement with a CUDA context, which the user allowed against the review's rules | finding 2 not run against OpenCode, which would write its state outside scratch |
| Tests | pass 3, `review-tests` at `high`; verified in pass 3, one `review-verifier` at `high`; finding 49 verified in pass 4, finding 81 in pass 5, both leads | `cargo mutants` not run, 551 mutants over the limit: the code no hand-written mutant touches had no mutant run; `nvml.rs`'s `read_gpu` and `monitor` need a GPU; `http.rs:53`, the connect-timeout clamp, needs an address that does not route; doctor's exit 0 through the binary cannot be built deterministically; `tests/fixtures/*` and `tests/snapshots/*` not opened, their match with their captures being `test_fixtures.py`'s job; finding 44's "names the build after its hash" not checked |
| Performance | pass 5, `review-performance` at `high`; verified in pass 5, `review-verifier` at `high` | `bench run`, `up`, `down` and `build` not run, by the review's rules: the waits measured on llama-swap alone, the bench's costs on the adapters through a harness in scratch, finding 88 on phase 3's measurement; test code, `fakes.rs` included, not read for cost; the pure domain modules and the inventory's reader not read, measured whole through `Configurator::prepare` at 0.144 ms; a threaded Doctor not built by the verifier: finding 84's gain rests on shell timings of its groups; finding 85's two requests not timed live, which holds the GPU |
| On-disk formats | pass 4, `review-structure` at `high`; verified in pass 4, `review-verifier` at `high` | `gateway.json`, `llama-swap.yaml`, `api-key` and the `current` links checked with no finding; `bench run` not run, by the review's rules: the run store's writing rests on reading and on its tests |
| Interruption and concurrency | pass 1, `review-correctness` at `high`; verified in pass 2 | no command was stopped mid-way: findings 15, 17, 18 and 19 rest on reading |
| The CLI's contract | pass 4, `review-structure` at `high`; verified in pass 4, `review-verifier` at `high` | `bench run`, `bench compare`, `up`, `down` and `logs` not run, by the review's rules: their statements rest on reading; `gateway config` run only by the binary tests, in the verifier's clone; `README.md:52`'s "more than 64 MiB" against doctor's `>=` left out, neither line changed in scope; exit status 2 for an invalid command line holds by hand, `ai-ctl status --bogus`, and no binary test checks it: not filed |

<details>
<summary>review-correctness: its note, FILES READ and LEFT OUT</summary>

Its note: it ran `ai-ctl bench list` once, outside the three commands the skill allows
(`doctor`, `status`, `build list`), on a scratch XDG directory
(`review-correctness/future-run`); `bench list` only reads, and nothing
outside scratch changed (`git status` and `pgrep` checked after it). It checked that the first
review's fixes hold for findings 9 (prune only; F3 is a path it leaves open), 10, 11 and 12
(bench run, partly), 13, 14, 15 (the error path), 16, 103 and 105, and NVML's close-on-exec
claim (`/dev/nvidiactl`, `/dev/nvidia0` and `nvidia-cap2` opened with `O_CLOEXEC`). Its summary
counts 17 findings; it gave 16 blocks, F1 to F16. `F<n>` is finding `n + 5` of this report,
F1 being merged into finding 4.

```
FILES READ
in full: src/main.rs, src/lib.rs, src/application/mod.rs, src/application/ports.rs, src/infrastructure/mod.rs, src/infrastructure/gateway_process.rs, src/infrastructure/files.rs, src/infrastructure/command.rs, src/infrastructure/build_store.rs, src/infrastructure/nvml.rs, src/infrastructure/download.rs, src/domain/prompt.rs, src/presentation/mod.rs, src/presentation/cli.rs
in part: src/application/lifecycle.rs (lines 1-880), src/application/bench.rs (lines 1-850), src/application/build.rs (lines 1-470), src/application/doctor.rs (lines 1-730), src/application/toolchain.rs (lines 1-440, plus tests around 534-564), src/application/gateway.rs (lines 1-220), src/application/gateway_config.rs (lines 1-110), src/application/init.rs (lines 1-30), src/domain/bench.rs (lines 60-890), src/domain/build.rs (lines 1-140), src/domain/gateway.rs (lines 1-135), src/domain/gateway_config.rs (lines 1-560), src/domain/inventory.rs (lines 1-440), src/domain/machine.rs (lines 30-230, by search), src/domain/recipe.rs (lines 1-130), src/domain/stats.rs (lines 1-60), src/domain/timestamp.rs (lines 1-110), src/infrastructure/run_store.rs (lines 1-725), src/infrastructure/gateway_store.rs (lines 1-180), src/infrastructure/git.rs (lines 1-225), src/infrastructure/cmake.rs (lines 1-250), src/infrastructure/http.rs (lines 1-450), src/infrastructure/gateway_api.rs (lines 1-112), src/infrastructure/server_api.rs (lines 1-215), src/infrastructure/chat_api.rs (lines 1-300), src/infrastructure/api_key.rs (lines 1-80), src/infrastructure/podman.rs (lines 1-105), src/infrastructure/inventory_file.rs (lines 1-285), src/infrastructure/paths.rs (lines 1-95), src/infrastructure/host.rs (lines 1-520), src/presentation/bench.rs (lines 1-200), src/presentation/lifecycle.rs (lines 1-240), src/presentation/build.rs (lines 1-240), src/presentation/gateway.rs (lines 1-70), src/presentation/compare.rs (lines 110-232, plus search), deploy/inventory.toml (lines 1-200), docs/roadmap/on-progress/v2-review-fixes/README.md (lines 1-150), docs/reviews/delivered-code/2026-10-15-v2-rebuild.md (lines 590-760, plus the headings), docs/toolchain.md (lines 85-190), a working note on the bench session plan's margin (lines 1-80), docs/commands.md (by search), ~/.local/share/ai-ctl/sources/llama.cpp/common/arg.cpp (lines 770-840, 1640-1650, 3510-3545, 3610-3620; outside the repository, for F1 and F2)

LEFT OUT
test modules of every src file, src/application/fakes.rs, tests/*.rs, tests/fixtures, tests/snapshots: read only through searches; the use cases' tests were not read case by case.
src/presentation/compare.rs (beyond lines 110-232), src/presentation/doctor.rs, src/presentation/completions.rs, src/domain/diagnostics.rs: rendering, skimmed for indexing and divisions only.
src/domain/gateway_config.rs (lines 560 on), src/domain/bench.rs (lines 1-60 and 890 on), src/domain/inventory.rs (lines 440 on): tests and test helpers.
Session plan margin: deliberately open in a working note on the bench session plan's margin, not reported again.
Running `up`, `down`, `build` and `gateway install`: not allowed, so F3, F4 (the build half), F6, F10-F13 and F16 rest on reading.
```

</details>

<details>
<summary>review-security: FILES READ and LEFT OUT</summary>

```
FILES READ
in full: docs/security.md, docs/roadmap/on-progress/v2-review-fixes/README.md, src/domain/gateway_config.rs, src/domain/inventory.rs, src/domain/gateway.rs, src/domain/build.rs, src/infrastructure/inventory_file.rs, src/infrastructure/api_key.rs, src/infrastructure/files.rs, src/infrastructure/paths.rs, src/infrastructure/gateway_process.rs, src/infrastructure/gateway_store.rs, src/infrastructure/download.rs, src/infrastructure/http.rs, src/infrastructure/gateway_api.rs, src/infrastructure/chat_api.rs, src/infrastructure/server_api.rs, src/infrastructure/git.rs, src/infrastructure/podman.rs, src/infrastructure/command.rs, src/application/gateway.rs, src/application/gateway_config.rs, src/main.rs
in part: src/application/lifecycle.rs (lines 1-880), src/infrastructure/build_store.rs (lines 1-320), src/infrastructure/host.rs (lines 1-320), src/application/doctor.rs (lines 420-730, plus grep), src/infrastructure/cmake.rs (lines 1-130), src/infrastructure/nvml.rs (lines 1-135), src/domain/recipe.rs (lines 1-60), src/application/bench.rs (grep for addresses and live_record), src/application/build.rs (grep for build ids), src/infrastructure/run_store.rs (grep for paths), deploy/opencode.json (lines 1-29), deploy/inventory.toml (grep), docs/stack.md (grep), llama.cpp common/arg.cpp in ~/.local/share/ai-ctl/sources/llama.cpp (lines 1200-1240, 3335-3350, 3505-3545)

LEFT OUT
src/presentation/* (bench.rs, build.rs, cli.rs, compare.rs, gateway.rs, lifecycle.rs, mod.rs): output and argument parsing; their inputs come from the user's CLI or from use cases already traced. Only the control-character path was followed into them, via grep.
src/application/toolchain.rs, init.rs, ports.rs, fakes.rs; src/domain/bench.rs, machine.rs, prompt.rs, stats.rs, timestamp.rs; src/infrastructure/mod.rs: no command, write or network sink reaches them from an attacker source, beyond what was traced through their callers. fakes.rs is a test double.
tests/* and tests/fixtures/*: test code, not attack surface.
src/application/bench.rs and build.rs beyond the grepped lines: bench only sends the key to the live record's address (bench.rs:401-405). Build ids are validated by `BuildId::parse`.
Ending the gateway's PID with `kill`: followed only through the liveness check of /proc/<pid>/exe; PID reuse between check and signal was not probed.
llama-swap's handling of `$VAR` and quoting inside `cmd`: not re-probed. Phase 0 checked finding 3 end to end, and only the inventory author controls those words.
```

</details>

<details>
<summary>review-tests: FILES READ and LEFT OUT</summary>

```
FILES READ
in full: tests/cli.rs, tests/gateway_config.rs, tests/gateway_lifecycle.rs; src/application/fakes.rs, ports.rs, lifecycle.rs, gateway_config.rs, gateway.rs, init.rs, build.rs, doctor.rs, toolchain.rs, bench.rs, mod.rs; src/domain/bench.rs, build.rs, gateway.rs, gateway_config.rs, inventory.rs, machine.rs, prompt.rs, recipe.rs, stats.rs, timestamp.rs; src/infrastructure/api_key.rs, build_store.rs, chat_api.rs, cmake.rs, command.rs, download.rs, files.rs, gateway_api.rs, gateway_process.rs, gateway_store.rs, git.rs, host.rs, http.rs, inventory_file.rs, mod.rs, nvml.rs, paths.rs, podman.rs, run_store.rs, server_api.rs; src/presentation/mod.rs, lifecycle.rs, gateway.rs, bench.rs, build.rs, compare.rs, cli.rs; src/main.rs, src/lib.rs, the scope's output
in part: src/presentation/doctor.rs (function and test names, by grep); docs/reviews/delivered-code/2026-10-17-v2-review-fixes.md (headings only, to avoid repeating findings); docs/roadmap/on-progress/v2-review-fixes/phase-5-closing-review-report.md (lines naming tests, by grep); the diff of commit "fix(lifecycle): bound up's and down's waits by a deadline" of 2026-10-16 for src/application/lifecycle.rs

LEFT OUT
tests/gateway_lifecycle.rs: it skips itself without an installed llama-swap or when port 5800 is taken; here it ran and passed on every run (`up_a_proxied_request_and_down ... ok`). Its inputs share one native model, no image and the first model port.
tests/fixtures/* and tests/snapshots/*.yaml: how the tests use them checked, the files not opened; whether they match their captures is test_fixtures.py's job, not run.
src/infrastructure/nvml.rs (`read_gpu`, `monitor`, `NvmlMonitoring`): needs a GPU and NVML; only the pure helpers are tested.
http.rs:53, the connect-timeout clamp: a test needs an address that does not route, which depends on the network and on timing.
`Lifecycle::up`'s `warn_unread` (lifecycle.rs:249): unreachable, since `target()` fails first without a readable GPU; not a test gap.
Smaller failure paths not raised: `signal` failing on something other than "No such process" (a signal to a PID the user does not own, unsafe if tests run as root); the `AlreadyExists` race in `files::write_new` (concurrent by nature); `build_store::remove` failing on its rename; non-UTF-8 event data in chat_api; the "malformed header" error in http.rs.
doctor's exit 0 on a healthy machine: cannot be built deterministically through the binary.
Out of scope and not read: src/domain/diagnostics.rs, src/presentation/completions.rs, the reviewing-delivered-code evals.
```

Its note: the scope's output reached it through a follow-up message, the session's prompt
having carried an unexpanded `$(cat …)` in its place; it read `scope.txt` in scratch. Its
patch of proposed tests and fake changes, 842 lines, and the logs of each mutant's survival and
kill are in its scratch folder, `review-tests/mutants/`.

</details>

<details>
<summary>Pass 3's review-verifier: FILES READ and LEFT OUT</summary>

```
FILES READ
- Code: src/application/{lifecycle.rs, doctor.rs, bench.rs, build.rs, gateway.rs, fakes.rs}, src/presentation/{lifecycle.rs, bench.rs, cli.rs, compare.rs, mod.rs}, src/infrastructure/{gateway_process.rs, build_store.rs, git.rs, cmake.rs, host.rs, http.rs, inventory_file.rs}, src/domain/{gateway_config.rs, machine.rs}, tests/cli.rs, tests/gateway_lifecycle.rs (head).
- Docs and the review: docs/architecture.md (Layers, Ports, Testing), the review report (head).
- The reviewer's material: run.sh, kill.sh, apply.py, review-tests.patch.
- The system: /var/run/cdi/nvidia.yaml, `systemctl cat nvidia-cdi-refresh.{path,service}`, /etc/nvidia-container-toolkit/nvidia-cdi-refresh.env.

LEFT OUT
- I did not run podman with the GPU device (forbidden), so the crun failure in 22 rests on the reviewer's run.
- I did not read the reviewer's add_tests*.py, or its mutant logs beyond one.
- I did not check consequence wording beyond the code lines named above, such as 44's "names the build after its hash".
```

Its note: it wrote nothing in the repository; `pgrep` showed nothing of its own left, and the
reviewer's clone in scratch still held exactly the patch. "The reviewer's run" in its first
LEFT OUT line is the session's run of pass 3, which wrote finding 22.

</details>

<details>
<summary>review-cleanup: FILES READ and LEFT OUT</summary>

```
FILES READ
in full: src/application/ports.rs, src/application/mod.rs, src/lib.rs, src/infrastructure/mod.rs, src/domain/mod.rs, docs/roadmap/on-progress/v2-review-fixes/phase-4-structure-and-refactoring.md, docs/roadmap/on-progress/v2-review-fixes/phase-4-structure-and-refactoring-report.md, a working note on the bench session plan's margin
in part: src/application/lifecycle.rs (lines 1-875), src/application/doctor.rs (lines 1-728, 795-800, 870-875, 1760-1765), src/application/build.rs (lines 1-463), src/application/gateway.rs (lines 1-217), src/application/gateway_config.rs (lines 1-108), src/application/init.rs (lines 1-26), src/application/toolchain.rs (lines 1-439), src/application/bench.rs (lines 1-845), src/application/fakes.rs (lines 290-312, 385-740, outline), src/infrastructure/gateway_process.rs (lines 1-367, 495-505, 585-605), src/infrastructure/files.rs (lines 1-162), src/infrastructure/run_store.rs (lines 1-723), src/infrastructure/build_store.rs (lines 1-140, 520-535), src/infrastructure/command.rs (lines 1-200), src/infrastructure/http.rs (lines 1-448), src/infrastructure/gateway_api.rs (lines 1-110), src/infrastructure/chat_api.rs (lines 1-130), src/infrastructure/server_api.rs (lines 1-100), src/infrastructure/podman.rs (lines 1-147), src/infrastructure/host.rs (lines 1-330, 440-470, 515-525, 670-676, 736-755), src/infrastructure/nvml.rs (lines 1-280), src/infrastructure/cmake.rs (lines 1-248, 390-410), src/infrastructure/git.rs (lines 1-221), src/infrastructure/gateway_store.rs (lines 1-176, 195-220), src/infrastructure/inventory_file.rs (lines 35-135), src/infrastructure/paths.rs (lines 1-93), src/infrastructure/api_key.rs (lines 25-130), src/infrastructure/download.rs (lines 1-72), src/presentation/mod.rs (lines 1-185), src/presentation/lifecycle.rs (lines 1-236), src/presentation/bench.rs (lines 1-484), src/presentation/compare.rs (lines 1-536), src/presentation/build.rs (lines 1-239), src/presentation/gateway.rs (lines 1-70), src/domain/bench.rs (lines 680-720, 770-960, 1076-1084), src/domain/gateway_config.rs (lines 40-60, 380-555), src/domain/inventory.rs (lines 290-316), src/domain/gateway.rs (lines 85-115), src/domain/machine.rs (lines 123-260), src/domain/recipe.rs (lines 1-127), src/main.rs (lines 1-40), tests/gateway_config.rs (lines 1-60), tests/gateway_lifecycle.rs (lines 160-260), docs/reviews/delivered-code/2026-10-17-v2-review-fixes.md (lines 1-140, 457-510, headings), docs/reviews/delivered-code/2026-10-15-v2-rebuild.md (findings 25, 26, 46, 50-53, 62, 99, 100, Coverage; headings), the working notes (headings)

LEFT OUT
GatewayProcess split (the lead): checked, and no finding on my axes. `Bench` calls `record`, `liveness` (through `live_record`) and `config`; `build prune` calls those and `lock`; Doctor's inspection calls `record`, `liveness` and `config`; only `Lifecycle` uses all ten. Splitting out a read-only part (say `record`, `liveness`, `config`, with `lock` for prune) removes no code and copies none. `FakeGateway`'s `liveness` reads the signals and advances the log, so both parts would stay on one fake, and one `LocalGateway`. With the read part as a supertrait, Services, `main.rs` and the fakes keep one value: `&dyn GatewayProcess` upcasts to the supertrait on rustc 1.97.1, checked in scratch, and on the crate's `rust-version` 1.89. What the split gains is the narrower declared dependency of `Bench`, prune and, after the Doctor finding above, Doctor. That is an architecture call for `review-structure`.
Dead-item search: `src/lib.rs` declares every layer `pub`, so I built a copy in scratch as a binary crate, all modules private under `main.rs`, where rustc reports unused `pub` functions, fields, variants and trait methods. It built with no warning, so no production item of the crate is dead or reached only by tests. Clippy `--all-targets` is green, which covers the private `fakes` module.
Copies seen and not raised: `liveness_of` against `Procfs::program` (first review's 99, kept). Stage/try/discard in build and gateway install (first review's 100, kept). `run_store::build`/`build_json` against `build_store::from_json`/`to_json`: a deliberate copy by phase 4's E2 decision, and sharing it would re-couple the run format to `build.json`'s. `doctor::shell_quote` against `gateway_config::shell_word`: different character sets. `chat_api::read_reply`'s non-200 branch against `http::copy_stream`: four lines, kept apart by batch 43's decision. The hand-written `kill not found; run …` against `command::not_found`: one line, and `kill` is not a `Tool`. `render_status`'s "{} of {} MiB in use" twice and `podman::parse_digest` against `Sha256::parse`: both predate the scope in code it did not change.
`tests/cli.rs`, `tests/gateway_lifecycle.rs` beyond lines 160-260, and test modules beyond the helpers cited: searched for duplicated helpers by name and pattern only. `tests/fixtures/` and `tests/snapshots/` were not opened. `deploy/` was not read. `docs/` beyond the notes and the review and roadmap passages cited was not read.
Uncommitted changes in the repository, `docs/reviews/delivered-code/2026-10-17-v2-review-fixes.md` and `docs/roadmap/on-progress/v2-review-fixes/phase-5-closing-review-report.md`, were already in place and are not mine. I wrote only under `review-cleanup` (clones `repo` and `repo2`, cargo target dirs, `callers.py`, `doctor-stubbed.txt`). Of ai-ctl's commands I ran only `doctor`, and `pgrep` finds no llama-swap, ai-ctl, cargo or rustc process left.
```

Its note: "the Doctor finding above" in its first LEFT OUT line is finding 51.

</details>

<details>
<summary>review-structure: its note, FILES READ and LEFT OUT</summary>

Its note, on phase 4's two leads. `GatewayProcess`: no rule is broken while `Bench` stands
alone; splitting out a read-only part is worth it only with finding 51's fix (its first block,
merged into 51). The texts of batches 43 to 48: `docs/commands.md` quotes none of them, so none
contradicts it, and most are pinned by a test; two are not, up's "not looking for processes
left on the GPU", which can never be printed (finding 69), and bench's "cannot record the run's
GPU" (merged into 55); batch 43's test is written on a hand-made error body (finding 71). It
ran `build list`, `doctor` and `status`, llama-swap alone in scratch, and clones in scratch;
`pgrep` found nothing left, and it changed no file of the repository.

```
FILES READ
in full: CLAUDE.md, docs/architecture.md, docs/decisions/0001-gpu-arbitration-through-llama-swap.md, docs/commands.md, README.md, .agent-conventions.toml, src/main.rs, src/presentation/mod.rs, src/presentation/cli.rs, src/application/ports.rs, src/application/mod.rs, src/lib.rs, src/domain/mod.rs, src/infrastructure/mod.rs, tests/fixtures/README.md, docs/roadmap/on-progress/v2-review-fixes/phase-4-structure-and-refactoring.md, docs/roadmap/on-progress/v2-review-fixes/phase-4-structure-and-refactoring-report.md
in part: src/application/lifecycle.rs (1-880, 1585-1610, 1890-1925, 2110-2135), src/application/doctor.rs (1-730, 1135-1170), src/application/bench.rs (1-460), src/application/build.rs (1-463), src/application/gateway.rs (1-217), src/application/gateway_config.rs (1-108), src/application/init.rs (1-28), src/application/fakes.rs (field declarations, 925-935, 1060-1080), src/presentation/lifecycle.rs (1-236), src/presentation/gateway.rs (1-70), src/presentation/build.rs (1-239, 300-345), src/presentation/bench.rs (1-200), src/presentation/compare.rs (1-360), src/infrastructure/build_store.rs (1-316), src/infrastructure/files.rs (1-154), src/infrastructure/run_store.rs (1-723), src/infrastructure/gateway_process.rs (1-367), src/infrastructure/api_key.rs (1-76), src/infrastructure/inventory_file.rs (1-285), src/infrastructure/paths.rs (1-93), src/infrastructure/http.rs (1-200, 595-735), src/infrastructure/command.rs (1-161), src/infrastructure/cmake.rs (58-100), src/domain/gateway.rs (1-130), src/domain/gateway_config.rs (60-260, 420-490), src/domain/inventory.rs (163-175, 300-322), src/domain/machine.rs (267-280), tests/cli.rs (1-12, 118-131, 290-380), Cargo.toml (1-30), docs/reviews/delivered-code/2026-10-17-v2-review-fixes.md (1-300, 566-702, 1304-1340), docs/reviews/delivered-code/2026-10-15-v2-rebuild.md (707-735, 1197-1435)

LEFT OUT
- On-disk formats with no finding beyond those above:
  - `build.json`: FsBuildStore, format 1, strict; finding 10 covers it.
  - Saved runs: format 3, reading 1 to 3; findings 10 and 50. The pre-E2 reader, run in a scratch clone at "refactor(doctor): split HostProbe by role" on a format-3 run, says "run format 3 is not supported (this ai-ctl reads formats 1 and 2)", so E2 broke no older reader.
  - `gateway.json`: no version and lenient serde; a change would read as "malformed; remove it once no llama-swap runs".
  - `llama-swap.yaml`: compared byte for byte; `starts_program` relies on the binary sitting alone on its line, as its doc states.
  - `api-key`: validated on read.
  - The `current` links: strict, with warnings.
- Domain modules bench.rs, build.rs, machine.rs, prompt.rs, recipe.rs, stats.rs and timestamp.rs: checked only by search (no I/O, no imports of the outer layers, thiserror); not read whole.
- Adapters chat_api, server_api, gateway_api, git, host, nvml, podman, download and gateway_store, and application/toolchain.rs: read only where a search led; not checked line by line for conventions.
- tests/gateway_lifecycle.rs, tests/gateway_config.rs, tests/snapshots: not read (tests axis).
- docs/security.md, docs/stack.md and docs/toolchain.md: not checked against the code (security axis, and no command contract).
- README.md:52 says doctor warns on processes holding "more than 64 MiB", but doctor.rs:624 uses `>=`. Neither line changed in scope, so I left it out.
- Exit status 2 for an invalid command line (commands.md:50) is checked by no binary test, only by `try_parse_from(..).is_err()`. It comes from clap and holds: `ai-ctl status --bogus` and `ai-ctl build list --force` both exit 2. Not filed.
- `bench run`, `bench list`, `bench compare`, `up`, `down` and `logs`: not run, per the rules; their statements were checked by reading.
```

</details>

<details>
<summary>Pass 4's first review-verifier, on findings 49 to 60: FILES READ and LEFT OUT</summary>

Its note: it worked in clones of `docs(v2-review-fixes): record the session close of the closing review's pass 4` of 2026-10-18 in its scratch folder, each with its own
`CARGO_TARGET_DIR`, after two clones sharing one had made cargo reuse a stale binary; the
affected run was done again. Of ai-ctl's commands it ran `doctor`, once, on a build whose
`pause` and `now` panic, and `bench list`, on a scratch XDG copy of the saved runs. It reused
pass 3's mutant tool, copied from outside the repository. `git status` in the repository was
clean after it, and `pgrep` found nothing left. Its one lead: none.

```
FILES READ
- src/application: lifecycle.rs (status, inspect, running, is_current, live_record, mib, tests' World and status tests), fakes.rs (FakeGateway), doctor.rs (Doctor, run, lifecycle, config_check, vram_check, mib, test constructions), bench.rs (Bench, setup, machine, test world), gateway_config.rs, build.rs (prune, refuse_served, current, current_or_none, prune test), gateway.rs (install), toolchain.rs (package, newer_toolkit_remedy, package_check), ports.rs (GatewayProcess)
- src/domain: bench.rs (GpuReading, policy), machine.rs (versioned_toolkit, release), inventory.rs (derives)
- src/infrastructure: run_store.rs, nvml.rs (gpu, clock_events), gateway_process.rs, api_key.rs, inventory_file.rs, files.rs, host.rs, cmake.rs, podman.rs, command.rs, gateway_store.rs, build_store.rs (test windows)
- src/presentation: mod.rs (Doctor literal), bench.rs (policy column)
- docs: architecture.md, CLAUDE.md, commands.md:120-130, README.md:44-50, stack.md:587, the phase-4 report (decisions, batches 44 and 45, line 385), the first review's finding 25, `.claude/agents/review-cleanup.md`
- pass 3's tooling (copied into `tools/`), the saved runs (read only), and the scope list

LEFT OUT
- Every other production file in the scope. No finding in this batch touches them.
- tests/gateway_lifecycle.rs and tests/gateway_config.rs, apart from the full suite runs.
- I did not run `bench compare`: it is not on the allowed list, so 50's "–" reading comes from the code alone.
- I did not apply 53's or 54's fix: read only.
- I did not measure the cost of the bench's second GPU read.
```

</details>

<details>
<summary>Pass 4's second review-verifier, on findings 62 to 69 and 71 to 79: FILES READ and LEFT OUT</summary>

Its note: it worked in a clone of `docs(v2-review-fixes): record pass 4's first verification of the closing review` of 2026-10-18 in its scratch folder, and in two more clones with
their own target directories, `mut69` (M31) and `mut72` (`tests/cli.rs` run with
`LD_DEBUG=libs`). Its runs of ai-ctl used a scratch XDG directory and `env -i`. llama-swap v263
ran alone on a loopback port with `touch` as its model's command, and was stopped. `pgrep`
found no llama-swap, ai-ctl or `flock` left, and `git status` in the repository was clean. Its
four leads: three are findings 80 to 82 under To Verify; the fourth, that `docs/commands.md`
does not say a GPU unreadable at `bench run`'s setup ends the run, went into finding 65.

```
FILES READ
- verifier-structure/findings.md and scope.txt.
- docs/reviews/delivered-code/2026-10-17-v2-review-fixes.md (findings 9, 10 and 13).
- src/presentation/build.rs, src/application/build.rs (Builder::build, list, use_build, current_or_none, prune, test fakes), src/infrastructure/build_store.rs (current, lock), src/infrastructure/files.rs (lock), src/infrastructure/gateway_process.rs (lock, :241).
- src/application/gateway_config.rs, src/application/lifecycle.rs (up, target, live_record, warn_unread, tests near :1894), src/application/ports.rs, src/application/bench.rs (setup, machine, tests), src/presentation/bench.rs (render_event), src/presentation/lifecycle.rs, src/presentation/mod.rs, src/presentation/gateway.rs.
- src/infrastructure/cmake.rs, src/infrastructure/paths.rs, src/infrastructure/http.rs (request_ok, unexpected, error_message, tests), chat_api.rs, gateway_api.rs and server_api.rs tests, src/infrastructure/nvml.rs, src/infrastructure/host.rs, src/infrastructure/git.rs, src/infrastructure/download.rs, src/infrastructure/gateway_store.rs, src/infrastructure/podman.rs, src/infrastructure/inventory_file.rs.
- src/application/fakes.rs (switches), src/application/gateway.rs (expected_sha256, fakes), src/domain/gateway.rs, src/main.rs.
- tests/cli.rs, the tests/fixtures llama-swap error captures, docs/commands.md (Files, Concurrency, build, gateway, up, bench), docs/architecture.md (Services, Ports, Testing), README.md, CLAUDE.md, the phase-4 report, and commits "refactor: turn a GPU reading into a GPU or an error once", "fix(gateway): say what the checksums of a non-pinned release prove" of 2026-10-16, "docs(v2-review-fixes): close phase 3, performance" and "docs(v2-review-fixes): close phase 4, structure and refactoring".

LEFT OUT
- Findings 61 and 70 (merged, as instructed).
- Finding 10's run and build halves (already verified).
- The other files in scope that no finding touches.
- `gateway config` and `bench run` were not run (not allowed), so their behaviour rests on reading and on the tests in the clones.
```

Its "not run" for `gateway config` means the command by hand: the binary tests of `tests/cli.rs`
ran it in its clone.

</details>

<details>
<summary>review-performance: FILES READ and LEFT OUT</summary>

```
FILES READ
in full: src/main.rs, src/lib.rs, Cargo.toml, src/infrastructure/chat_api.rs, src/infrastructure/http.rs, src/infrastructure/nvml.rs, src/infrastructure/gateway_process.rs, src/infrastructure/paths.rs, src/infrastructure/build_store.rs, src/infrastructure/host.rs, src/infrastructure/command.rs, src/infrastructure/server_api.rs, src/infrastructure/cmake.rs, src/infrastructure/git.rs, src/infrastructure/download.rs, src/domain/prompt.rs, src/domain/stats.rs, src/presentation/mod.rs, src/presentation/lifecycle.rs, docs/roadmap/on-progress/v2-review-fixes/phase-3-performance.md, docs/roadmap/on-progress/v2-review-fixes/phase-3-performance-report.md
in part: src/application/bench.rs (lines 1-860), src/application/lifecycle.rs (lines 1-880), src/application/doctor.rs (lines 1-740), src/application/build.rs (lines 1-470), src/application/toolchain.rs (lines 1-445), src/application/gateway_config.rs (lines 1-110), src/application/gateway.rs (lines 1-220), src/application/ports.rs (lines 37-59, 113-121), src/infrastructure/run_store.rs (lines 1-730), src/infrastructure/gateway_api.rs (lines 1-112), src/infrastructure/gateway_store.rs (lines 1-180), src/infrastructure/files.rs (lines 1-155), src/infrastructure/podman.rs (lines 26-73), src/domain/bench.rs (lines 167-207, and a grep of its functions), src/domain/machine.rs (lines 206-223), src/presentation/bench.rs (lines 1-82), src/presentation/compare.rs (lines 17-77), docs/reviews/delivered-code/2026-10-17-v2-review-fixes.md (lines 1-60, 588-620, 1544-1590, and the headings)

LEFT OUT
- tests/cli.rs, tests/gateway_config.rs, tests/gateway_lifecycle.rs, src/application/fakes.rs: test code, no production cost; fakes.rs only grepped for Cell/RefCell.
- src/domain/gateway_config.rs, inventory.rs, recipe.rs, timestamp.rs, gateway.rs, build.rs; src/infrastructure/inventory_file.rs, api_key.rs; src/application/init.rs: not read. Pure computation or one small file read; `Configurator::prepare`, inventory parse and config generation included, measured whole at 0.144 ms.
- src/presentation/build.rs, gateway.rs, cli.rs, and the rest of compare.rs: rendering only, not read.
- A live `bench run`, `up` and `down`: not run, since `up`/`down` are forbidden and a bench holds the GPU. Their waits were measured on llama-swap alone, and the bench's costs on the adapters through a harness.
- Measured, no finding:
  - the monitor's 250 ms sampling: 0.023 ms CPU a reading (87,114 readings in 2.01 s in a tight loop); at 250 ms, 40 readings in 10.00 s for 10 ms CPU, no drift;
  - `bench list`'s full parse of the 7 saved runs (5.5 MB): 4.4 ms median, 2.8 ms with the samples skipped;
  - `up`'s 100 ms health poll: llama-swap answers /health 86-90 ms after spawn (n=5), so about 14 ms lost;
  - spawning `kill` for a signal: 0.4-1.6 ms;
  - `status`: 31-35 ms, of which NVML's initialisation is 24-26 ms and unavoidable there;
  - `build list`: 1.1-1.5 ms;
  - doctor's second inventory load and extra build-list reads: about 0.05 ms;
  - `build`: ccache is used through llama.cpp's GGML_CCACHE, the build log says "ccache found", so a fresh tree per build costs no recompilation.
- Repository state: `git status` shows docs/roadmap/on-progress/v2-review-fixes/phase-5-closing-review-report.md modified. I did not write it; every file of mine is under review-performance (clone, target, harnesses, xdg copy, wrappers, swap probes). pgrep finds no llama-swap, llama-server or probe left, and nvidia-smi lists no compute process.
```

The phase report it names as modified is the session's own Work Log entry, written while it ran.

</details>

<details>
<summary>Pass 5's review-verifier, on findings 80 to 85, 87 and 88 and on 86: FILES READ and LEFT OUT</summary>

```
FILES READ
verifier-performance/findings.md; in my clone of "docs(v2-review-fixes): record pass 5's performance review of the closing review" of 2026-10-18, `verifier-performance/repo`:
- application: doctor.rs (40-135, 196-225, 320-450), gateway_config.rs (30-110, 270-300), bench.rs (278-365, 760-845, 1240-1266), build.rs (95-200, 870-912), lifecycle.rs (20-45, 300-345, 655-800), toolchain.rs (20-60, 266-317), fakes.rs (25-60, 376-386, 851-876, 930-951);
- domain: machine.rs (206-223), build.rs (41-100), recipe.rs (56-74), inventory.rs (160-215);
- infrastructure: host.rs (215-250, 439-449), inventory_file.rs (42-53, 100-150, 225-250), gateway_process.rs (95-135, 164-176, 234-246), nvml.rs (30-75), server_api.rs (380-430), http.rs (600-620, 698-730), command.rs (env lines);
- tests/fixtures/README.md (53-80); tests/gateway_lifecycle.rs (head);
- in the review, docs/reviews/delivered-code/2026-10-17-v2-review-fixes.md: findings 10, 27, 51, 55, 71, 72;
- docs/roadmap/on-progress/v2-review-fixes/phase-3-performance-report.md (198-275);
- the reviewer's scripts in review-performance (env.sh, probes.sh, wrap/rpm, swap/*.py, harness*/src/main.rs).

LEFT OUT
- Finding 55's cause, as instructed.
- The live cost of 85's two requests, which needs the GPU held.
- An NVML hang for 82, which cannot be produced here.
- An execve trace of doctor, since strace is not installed: I relied on the reviewer's wrapper list for the programs doctor runs.
- A threaded Doctor for 84 was not built; the group timings are shell measurements.
- `.claude/skills/reviewing-delivered-code/evals/test_fixtures.py` was not run on the 81 copy.
```

It wrote only in its folder: copies `repo81`, `repo82`, `repo85`, `repo88`, `repo-probe` and
`repo85-probe`, its llama-swap probe and capture, `harness86`, its timing scripts and XDG
copies. Of ai-ctl's commands it ran `doctor` and `doctor --json` only, `cargo test` in its
copies with `tests/gateway_lifecycle.rs`, and left nothing running (`pgrep -x llama-swap` rc 1,
no compute process in `nvidia-smi`).

</details>

<details>
<summary>Pass 5's review-verifier, on finding 89: FILES READ, LEFT OUT and its lead</summary>

```
FILES READ
- verifier-89/findings.md
- src/infrastructure/nvml.rs:1-80
- `git show` of "perf(gpu): initialise NVML once per process" of 2026-10-16 (commit message and the nvml.rs diff)
- docs/roadmap/on-progress/v2-review-fixes/phase-3-performance-report.md:100-135
- docs/roadmap/on-progress/v2-review-fixes/phase-5-closing-review-report.md:570-590
- docs/reviews/delivered-code/2026-10-15-v2-rebuild.md:980-1000
- docs/toolchain.md:145-170
- grep results in CLAUDE.md, docs/architecture.md, docs/stack.md and the skill's SKILL.md and assets
- verifier-performance/harness86/src/main.rs and env.sh
- review-performance/harness/src/main.rs

LEFT OUT
- The bench monitor's first reading, `NvmlMonitor::start`: the comment does not quote it.
- The cost of waking the dGPU from RTD3: the dGPU stayed in P8 and did not suspend during the runs, so the first-read figure is for an awake GPU.
- `ai-ctl doctor` and `status` were not run: the harness times the adapter directly.
- The rest of `src/`.
```

Its lead, left while verifying a lead, so it stays here unverified: `docs/toolchain.md:147-162`,
conventions. The record is to log every change to the driver, and logs the akmods rebuilds of
`kmod-nvidia` for each kernel (transactions 342 and 344); rpm shows two builds of `kmod-nvidia`
615.78.08 it does not hold, for kernel 7.2.11-200.fc44 on 2026-10-17 10:35 and for
7.2.11-200.audio.fc44 on 2026-10-17 13:05. The file is outside the code the review fixes; the
record is the user's to complete.

</details>

**Files of the scope no agent opened:** every production file of the scope was opened, the
largest in part, as each agent's FILES READ says. Not opened: the nine captures of
`tests/fixtures/` in the scope and the two snapshots `tests/snapshots/gateway-shipped.yaml` and
`gateway-shipped-image.yaml`: no check compares a capture with what its tool prints, since
`test_fixtures.py` tests the skill's own eval fixtures (corrected on 2026-10-18; the
reviewers' rows above say otherwise), and the snapshots are compared byte for byte by
`tests/gateway_config.rs`; the `Makefile`, the
user's shortcuts for daily commands and OpenWebUI's container, added in phase 4 and run by no
command of ai-ctl; a working note on the review's own
workflow.

## Fix Plan

Each batch is one change, with the tests its findings name, landed test-first in a commit of
its own with the five checks green; a test finding's new test must fail under the mutant it
cites (Checks). The proposed tests and the mutants' definitions of findings 23 to 49 start from
the patch kept outside the repository (Checks); passes 4 and 5's verifiers' patches likewise.
The order is the first review's: security, then bugs and error handling, interruption and
concurrency; the dead code, so that nothing after it tests or moves code that goes; the test
gaps, a net before the code moves; performance; structure and refactoring; the documents last,
since they describe the final structure. Three batches stand outside their section because a
later batch needs them first: 15 (finding 82) prepares the `resolve()` that batch 46 (finding
55) fills; 28 (finding 27) is the guard that batch 43 (finding 87) keeps; 39 (finding 53) gives
the names that batch 40 (finding 83) queries. Batch 41 (finding 84) comes after batches 24 and
40, which narrow Doctor and shorten its longest probe; it was rejected with the verdicts.

Six batches hold a choice for the user: 1, the backends on unix sockets if llama-swap allows
it, or the risk recorded alone; 2, a rotation of the key at each `up`, as an option, or the
documents alone; 22, `libc` as a new dependency, or the record written before the PID is
known; 24, the read-only `GatewayState` supertrait as a rider, or not; 26, `// LEGACY:`
comments or a one-off migration of the saved runs; 48, the adapters' program as a field, or
`docs/architecture.md` narrowed.

**Security**

1. **The backends off TCP loopback, or their squat recorded** — finding 1: first a probe of
   whether llama-swap v263 proxies to and health-checks a llama-server listening on a unix
   socket (`--host <path>.sock`); if it does, each backend on a socket in a 0700 directory
   under the state directory. Either way, `docs/security.md` §1 and §2 record the squat of a
   backend's port, ports from 5800, the key and the prompts handed over, with no race to win.
   The probe's result and its impact go to the user before the transport changes (verdict).
2. **The gateway's port squat documented, and up's refusal pointing to the key's rotation** —
   finding 2: `docs/security.md` §3, named in §1; the "taken by another program" error
   (`lifecycle.rs:244-247`) names the rotation procedure of `docs/security.md:62-63`. No
   rotation at each `up` (verdict).
3. **`up` checks that the listener on `listen` is the process it started** — finding 3: the
   listening socket's inode in `/proc/net/tcp` or `tcp6` matched against `/proc/<pid>/fd`.
4. **Underscored flag names refused** — finding 4: `check_flag_name` refuses `_`, which covers
   defaults, flags, files and the override order.
5. **GPU process names escaped where NVML gives them** — finding 5: one escaping helper shared
   with `inventory_file.rs:94` and `Text::checked`, extended to U+2028 and U+2029, called from
   `read_gpu`.

**Bugs and error handling**

6. **A text flag value that is an option refused** — finding 7.
7. **Non-finite float flags refused** — finding 11, for defaults and models.
8. **`build --force` spares the build the gateway serves** — finding 8: the gateway's lock and
   `refuse_served` before `replace`, naming `ai-ctl down`.
9. **An unreadable current build, and `build list`'s `current` link, told as they are** —
   findings 9, 62, 63: `Current::Unreadable(id, reason)`, with the remedy `ai-ctl build
   <project> --ref <id> --force` in doctor, `up` and `build list`; the not-installed case sent
   as a warning from `application::build::list`, which also covers a project with no readable
   build.
10. **doctor's CDI check reads the specification's host paths** — finding 22: the parser
    collects `hostPath`, tested on a captured specification; a warning naming the first
    missing path, with the refresh service as its remedy; `CDI_REMEDY`'s comment corrected.
11. **A later format read as unsupported** — finding 10, 79 merged: the version key read alone
    first in the run store, the build store and the inventory, then the strict parse.
12. **A remedy of its own for an inventory of a newer version** — finding 80, after batch 11:
    `UnsupportedVersion` in the domain, downcast by doctor's Inventory check.
13. **`up` checks a recorded gateway's health before "up already"** — finding 12.
14. **Read timeouts named** — finding 14: "no answer within N s", once in `http.rs`.
15. **`gateway config` reads the GPU after the inventory** — finding 82: `prepare_on` split
    into `resolve()` and `size()`, which batch 46 builds on.
16. **`LocalGateway::write` through `files::replace_file`** — finding 20.
17. **`prune`'s error names the builds already removed** — findings 21, 39: 39's test written
    against the new text.

**Interruption and concurrency**

18. **The start/stop lock tested, and `status` observing without it** — findings 23, 49, 13:
    23's test for `up` and `down` first, extended to `status` by 49, then `status` takes the
    lock only to forget a stale record.
19. **A failed bench run keeps its completed scenarios** — findings 16, 33: `Recorded { run,
    saved, failure }`, rendered before the error; 33's test, on a test `Services` of the fakes,
    written against it, with its output kept out of libtest's, and `docs/architecture.md:115`
    and `fakes.rs:1` updated.
20. **A bench run saved after each scenario** — finding 15, replacing the partial file.
21. **`publish`'s hidden copies put back before the sweep** — findings 17, 34.
22. **The window between the spawn and the record's write documented** — findings 18, 24: no
    `libc` (verdict); the window, and what the next `up` says of the llama-swap it leaves,
    written where `up` and `down` are documented; 24's test of the kill when the record cannot
    be written.
23. **`write_new`'s leftovers swept** — finding 19: the `.<name>.<pid>.tmp` files whose PID no
    longer runs.

**Dead and legacy code**

24. **Doctor on a read-only inspection, without `pause` and `now`** — finding 51: the read-only
    part of `Lifecycle` in a struct of its own, which Doctor builds alone; no `GatewayState`
    supertrait (verdict).
25. **up's unreachable GPU warning removed** — finding 69: `target` gives the `&Gpu`, and M31's
    branch goes.
26. **The run store's formats 1 and 2 under `// LEGACY:`** — finding 50, no migration of the
    saved runs (verdict).

**Tests**

27. **The write errors of `logs` and of a streamed model log** — findings 25, 26, one failing
    writer in `application::fakes`.
28. **The grace between SIGTERM and SIGKILL** — finding 27, at least 35 s on the fake clock;
    batch 43 keeps it.
29. **The clamps of `up`'s start wait** — finding 28, on `FakeApi` with a first check's
    duration.
30. **doctor's Gateway check: an unreadable record, a gateway without a model** — findings 29,
    47.
31. **The bench's overhead address, `--scenario`'s mapping and the monitor's failure** —
    findings 30, 31, 46.
32. **`status`'s exit 0 with a running gateway** — finding 32.
33. **The adapters' failing commands, and a tag over a branch** — findings 35, 36, 37, 44.
34. **`up` on an address it cannot bind, `down` with a signal it cannot send** — findings 38,
    41.
35. **`build use` under the builds' lock, and the current llama-swap reinstalled** — findings
    40, 45.
36. **Paths that are not UTF-8** — findings 42, 43.
37. **The compare table's change from zero** — finding 48.
38. **Error answers tested on captures of their own request** — findings 71, 81: the capture of
    `GET /upstream/nope/props` and its row in `tests/fixtures/README.md`, served by
    `server_api`'s test and by `http.rs`'s `GET /x`.

**Performance**

39. **The toolkit package name formatted once** — finding 53, here because batch 40 queries
    those names.
40. **rpm queried by exact names** — finding 83, with a fixture captured from `rpm -q` with a
    missing name.
41. **doctor's probes run at once** — finding 84, rejected (verdict): no change.
42. **The session scenario's bare prompt counted once** — finding 85, with a test that pins the
    counts of a session run.
43. **`down`'s poll growing from 5 ms to a deadline** — finding 87, under batch 28's test.
44. **An installed release refused before the fetch** — finding 88, for a release tag only.

**Structure**

45. **`prune`'s own busy text on the gateway's lock** — finding 64.
46. **`Bench::setup` on one preparation** — finding 55, 70 and 86 merged: one GPU reading
    through `prepare_on`, `Prepared` carrying the inventory and builds that batch 15's
    `resolve()` fills; the second load, its errors and M30's text go.
47. **The failure switches named `<method>_fails`** — finding 73, `bad_link` kept as state.
48. **The rule of the program as a field narrowed** — finding 74: `docs/architecture.md:121-123`
    names the adapters that follow it and the route the others take (verdict).

**Refactoring**

49. **`files::read_if_present`** — finding 52.
50. **One lenient read of a `current` link** — finding 54.
51. **`find_nvcc` through `SystemHost::find`** — finding 56.
52. **Doctor's Idle VRAM through `lifecycle::mib`** — finding 57.
53. **One test helper for an executable script** — finding 58.
54. **`gateway_process`'s fixtures embedded** — finding 59.
55. **`files::mode` in the three tests** — finding 60.

**Documents**

56. **`docs/commands.md` and the README say what the commands do** — findings 65, 66, 67, 68.
57. **`docs/architecture.md`'s statements match the code** — findings 72, 75.
58. **The doc comments match the code** — findings 76, 77, 89.
59. **README's Status** — finding 78, written with the roadmap's state when it lands.

**Approved** on 2026-10-18. The user delegated the verdicts to the session and settled the hard
cases: "Serait t'il possible de te laisser choisir les confirmation et que tu ne me demande que
pour les cas les plus durs à trancer ?". The session confirmed 83 findings and rejected one,
84; the six choices are settled in the batches above, the user's for 1 and 18, the session's
for 2, 24, 26 and 48. The user chose to fix the whole plan in this roadmap, 58 batches, none
left open, and to close it with a security pass on the fixes' diff, not a full review: a full
review after each fix roadmap would be a loop with no end ("faire une review complète a la fin
de chaque roadmap de correction reviens à une boucle sans fin"; "garder une passe sécurisé me
semble primordiable"). Each batch's diff also runs `cargo mutants --in-diff` before its commit.
