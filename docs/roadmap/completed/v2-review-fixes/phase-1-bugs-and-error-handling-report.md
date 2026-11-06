# Phase 1 Report: Bugs & Error Handling

**Phase:** [phase-1-bugs-and-error-handling.md](phase-1-bugs-and-error-handling.md)
**Start Commit:** `docs(v2-review-fixes): close phase 0, security` of 2026-10-16

---

## Work Log

### 2026-10-16

Phase opened at `docs(v2-review-fixes): close phase 0, security`. Read before working: phase 0's file and report in full. What carries
into this phase: the decisions on the key (one key for the gateway and every backend, kept in
`~/.local/state/ai-ctl/api-key` at 0600, passed through the environment), `expose`, and CORS
left as llama-swap sets it; the security document's two additions from the live check; batch 13
reduced to the run store. No dependency left unsettled for the first task.

Security document. Read for it: findings 1 to 7 of the review, `docs/stack.md` §4.4, §5, §6.4
and §7, and the code that each statement rests on: `api_key.rs`, `Text::checked`, the reserved
flags, the `expose` check, the image warning, `download.rs`'s protocols, `git.rs`'s fetch and
the pinned release. Checked on this machine, nothing started:
- the modes of ai-ctl's files: the key 0600; the configuration, the inventory and both gateway
  logs 0644, in directories of mode 0755. `~` and `~/.local` are 0700, which alone keeps other
  users out. The files take the umask's mode (0022 here): `File::create` for the log;
- what the gateway log holds at llama-server's default verbosity, counted by line prefix
  without printing the content: model loads, file paths, timings and token counts, and ten
  `unauthorized` lines in the current log (two loads). No prompt or answer;
- the firewall: active zone `FedoraWorkstation` on the Wi-Fi interface, services
  `dhcpv6-client` only, no port. OpenWebUI listens on `127.0.0.1:18000`.

Wrote `docs/security.md`: an overview table, what listens, the key (where it lives, what it
guards, what it does not), loopback and `expose`, CORS, the clients that hold the key with
OpenWebUI on loopback and the `unauthorized` lines, what ai-ctl runs (the inventory, llama-swap
releases, llama.cpp sources, models, images), and the files ai-ctl keeps. `CLAUDE.md` lists it
under Where Things Are, and `docs/stack.md` §7 links to it after the facts it draws on.
`check_links.py` finds no broken link, and the six anchors into `stack.md` match its headings.

Put to the user: the state files' modes (see Problems And Deviations), and a commit. The user
approved both. Added the task "State Directory" to this phase, after the security document.
Committed as `docs: add docs/security.md, gathered from phase 0`.

State directory, test-first. Four places create a directory under the state, none knowing the
state's root: the gateway's lock and record (`gateway/`), its log (`logs/`), the build log
(`logs/`), and the API key (the state directory itself, through `write_new`). Each test failed
first on its assertion, 0755 for 0700:
- `files::create_private_dir` creates a directory and its missing parents at 0700 with a
  recursive `DirBuilder`; a directory already there keeps its mode. Its test, written against a
  body that still called `create_dir_all`;
- the gateway: `lock`, `write` and `new_log`, each on a fresh root, leave the root and their
  directory at 0700. `lock` creates the directory before `files::lock`, whose own
  `create_dir_all`, shared with the data stores, is unchanged;
- the build log: its creation moved from `compile` into `CmakeBuilder::create_log`, unchanged
  first, so that a test reaches it without running CMake;
- the API key: `read_or_create` creates the key's directory before `write_new`, whose own
  `create_dir_all` serves the inventory as well and is unchanged.

The five checks are green. On this machine, `~/.local/state/ai-ctl` and its `gateway` and
`logs` directories were narrowed to 0700 by hand, with no gateway running. `docs/security.md`
§8 now says what the code guarantees and how an older directory is narrowed.
Committed as `fix(state): create ai-ctl's state directory at mode 0700`.

Batch 6 (finding 2), test-first. Two tests, each watched failing on its assertion:
- `liveness_of`: a copy of `/usr/bin/sleep` installed as `real/llama-swap/v263/llama-swap`,
  run through the link `data -> real` and recorded by the link's path. It must read `Running`
  while installed, once the file is removed, and once its directory is removed too, then
  `Exited`. Before the fix, all three read `Reused`, the kernel naming
  `real/…/llama-swap` and `real/…/llama-swap (deleted)`: the reproduction of the finding;
- `FsBuildStore::contains`: a store built on `data/builds`, with `data` a link, contains a
  binary named by its real path, removed or not, and not a program beside the store.

The fix: `liveness_of` compares the link with `resolved(binary)`, and with that path followed
by " (deleted)". `resolved` canonicalizes the path, or, for a removed file, its nearest part
that still exists, then adds the removed names back. `contains` compares with the
canonicalized root, or the root as given when it does not exist. The record keeps the binary
as given, so `is_current` still compares it with the target `up` would start. The five checks
are green.

Session closed at the user's request, with batch 6 committed with its record. Not recorded
above: each commit of the session was put to the user before it was made; the user has not
said yet whether later batches may be committed as soon as their checks are green.

**Where the next session starts:**
1. Batch 8 (finding 9): `build prune` refuses to remove a build whose binary the running
   gateway's configuration names, and names `ai-ctl down`; a test watched failing first.
2. Then batch 9 (finding 10, P4), then the bench batches 7, 10, 54 and 55, in the phase's
   order; P1 comes after batch 13.
- Put to the user: whether each batch is committed as soon as its checks are green, without
  asking.
- Pending: nothing uncommitted.

Session resumed. The user answered the open question: each batch is committed as soon as the
five checks are green, without asking first.

Batch 8 (finding 9), test-first. Read first: how the configuration names a build. Each native
model's `cmd` block holds the build's binary alone on its first line, written as llama-swap's
splitter reads it back (`shell_word`); `up` writes that configuration, and `config()` reads it
back. On this machine, `~/.local/state/ai-ctl/gateway/llama-swap.yaml` holds
`/home/seb/.local/share/ai-ctl/builds/llama.cpp/b11538/llama-server` alone on the first line of
both models' commands.
- The domain: `gateway_config::starts_program(config, program)`, tested on the YAML `to_yaml`
  writes. Watched failing twice: on a stub, then on a version that compared the bare path,
  which missed a data directory with a space in its name, quoted in the file.
- The use case: `prune` takes the gateway's processes. Four tests, on builds `b1` to `b4`,
  `b4` current, and a gateway running on the configuration the configurator wrote while
  another build was current. Watched failing: `prune --keep 1` removed `b3`, `b2` and `b1` with
  the gateway on `b2`, the reproduction of the finding. The fix computes every project's builds
  to remove first, then refuses before removing any: when the running gateway's configuration
  starts one of their binaries, naming them, `ai-ctl down` and `ai-ctl up`; or when a gateway
  runs without its configuration, since prune cannot tell what it serves. A gateway that no
  longer runs (`Exited`, `Reused`) protects nothing. `prune` also holds the gateway's lock, a
  test with the lock taken.
- `live_record`, the helper finding 52 names, is added to `application::lifecycle` for
  `prune`; `log`, `model_log` and `Bench::listen` keep their copies until phase 4's batch 46.
- `build prune --help` and the README say when prune removes nothing.

The five checks are green. Committed as `fix(build): refuse to prune a build the running gateway serves`. That commit put `live_record` between
`held`'s doc comment and `held`; moved back with batch 9.

Batch 9 (finding 10, P4), test-first. The fake's started gateway gained `wait_failures`, the
number of its first checks that fail, with the real adapter's error text. Two tests, each
watched failing on its assertion:
- a gateway that never answers and outlives SIGKILL (`immortal`): the record must keep PID
  4242, and the message say that it could not be stopped and stays recorded. Before the fix the
  record was gone: the reproduction of finding 10;
- the started gateway's check failing once, then twice: `up` must stop it (SIGTERM), forget it,
  and say it cannot tell whether llama-swap runs. Before the fix `up` returned the check's error
  at once, the gateway running and recorded: the reproduction of P4. Twice, the check fails
  inside the stop as well, and the check after the stop tells the gateway exited: the case the
  task's "or `started.exited()` reports an exit" covers.

The fix: a failed check ends the wait like the timeout does, with its own first clause; the
record goes only when the stop succeeded or a last check reports an exit; otherwise the message
says the gateway stays recorded for `ai-ctl down`. `up`'s doc comment says so. The five checks
are green. Committed as `fix(gateway): keep the record of a gateway up cannot stop`.

Batch 7 (findings 8, 90, P2), test-first. Read first: how a session's prompt grows. Each later
turn adds the reply before it, as an assistant message, and a user message holding the `TURN`
preamble and 1,024 tokens of the corpus; `peak` counted the corpus tokens and the replies only.
What the preamble and the template add depends on the model's template, so it is measured
through the model's server: `Prompts::turn_extra`, the tokens `agent("")` followed by an empty
reply and an empty turn makes beyond `agent("")` alone, read only when the session is selected.
`peak` and `Plan::new` gained their parameters first, the behaviour unchanged, so that the six
tests failed on their assertions:
- the domain: `peak` with 63 extra tokens a turn; finding 90's boundary on the new peak,
  23,096 accepted and 23,095 refused with every field of the error; a 16,384 context planned
  when the session is left out;
- the use case: `turn_extra` on the fake template, the text of an empty turn and the two role
  markers, 63; `--scenario depth` runs on a 16,384 context, and with the session added the
  error names 23,096 tokens (finding 8's reproduction: the depth-only run failed on the session
  error); a session refused at 23,095, and at 23,096 its last turn's prompt and reply fill the
  context but the free token (P2's reproduction: 23,095 passed the plan).

The fake model server gained `ctx_size`. The prompts are now tokenized before the plan is made;
the plan, as a saved run holds it, is unchanged. The README says what a session needs and how
to run the other scenarios. The five checks are green.

Left for batch 54's run on this machine: the measured extra stands for a real turn when the
template drops or keeps the replies' reasoning, and a reply's tokens read back as about the
tokens generated. That run's session can check it: its last turn's prompt tokens and 512 stay
under the plan's need. Committed as `fix(bench): plan a session on the tokens its turns send, and only when selected`.

Batch 10 (findings 27, 11, 12), test-first. `Bench` takes the `RunStore`, and `run` returns a
`Recorded`: the run, and where it is saved or why it cannot be. `FakeRuns` joins the fakes: runs
in memory, an unreadable one with its reason, and a full disk. The signatures changed first, the
save still in presentation, so that four tests failed on their assertions:
- the run is saved before it is returned, under its ID; a save that fails still returns the run,
  with the error;
- the session's second turn failing after the depth and prefill scenarios: the error names the
  step, then the saved ID and the completed scenarios; the saved run holds the depths and the
  prefill, no session, no drafts check, and the GPU readings without the session's. Before the
  fix the error was the step's alone and nothing was saved: the reproduction of finding 11;
- the same failure with a full disk: the error says the completed scenarios cannot be saved,
  and why.

The fix: `Runner::measure` runs one scenario into the run, which keeps nothing of it on a
failure; the loop stops at the first failure, ends the GPU reading, and saves the completed
scenarios as a run of their own, as if they alone were selected. A failure in the first
scenario saves nothing: there is nothing to save. The failed request's drafts are unknown, so
the drafts go unchecked; `Run::drafts`' doc comment says so. Presentation prints the run once it
is saved: a print error names where the run is saved, a save error comes after the print
(finding 12). `bench::saved` with an unreadable run and `bench::load` are tested on `FakeRuns`;
both tests passed at once, the code being correct, and each failed under a mutant (`load`
reversed, `saved` dropping an unreadable run). The README says what a failed request leaves.
The five checks are green. Committed as `fix(bench): save the run in the use case, and keep completed scenarios`.

Batch 54 (finding 104), its code; the probe under load comes later. Read first: nvml-wrapper
0.13's `current_throttle_reasons` truncates the mask to its flags, and its strict form returns
the whole mask in `IncorrectBits(Bits::U64(mask))`; `nvml.h` of CUDA 13.4 gives BoardLimit
0x200 and Reliability 0x400. At idle, `nvidia-smi --query-gpu=clocks_throttle_reasons.active`
gives `0x0000000000000400` again, at 180 MHz and 4.6 W. The fields came first, the behaviour
unchanged, then nine tests failed on their assertions, the existing round trips among them
since the test run's idle reading now carries Reliability:
- the adapter: `slowdown` takes the raw mask; 0x400 and 0x200 set `policy` and nothing else,
  and with the power cap both show;
- the domain: `GpuReading::clock_events` keeps the mask, `None` in a run saved before;
  `Slowdown::policy` and `GpuSummary::policy`, a share apart from the slowdowns;
- the run store: format 2 writes each reading's `policy` and `clock_events`, and reads formats
  1 and 2; a format-1 run reads without them, and that test failed under a mutant without the
  `default`; format 3 is refused;
- `bench run`'s GPU table gains a "policy limit" column; "slowed by" is unchanged, and
  `bench compare` shows no policy row.

On this machine, `bench list` lists all six saved runs, and `bench compare` sets the first and
the last side by side. The five checks are green. The task stays open until the probe under
load shows what BoardLimit and Reliability report while the GPU works. Committed as `feat(bench): keep NVML's clock-event mask and show a policy limit apart`.

Batch 55 (finding 105), test-first. `Monitoring::finish` returns a `Readings`: those taken,
and how many failed with the last one's error. `FakeMonitor` gained `lost_from`, the scenario
from which every reading fails. The types changed first, the behaviour unchanged; then two
tests failed on their assertions:
- the use case: readings lost from the session on, then from the depth on: the run keeps the
  others, and a warning counts the lost readings, once and twice, with the last error;
- `render_gpu`: the test run's prefill and overhead hold results but no reading, and showed no
  row; they now read "no reading".

The fix: the NVML thread keeps the device of its first reading, where it asked NVML for it a
second time and returned the first reading alone when that failed; each later reading that
fails is counted with its error. `Bench::run` emits the warning. `Run::holds`, in the domain,
tells which scenarios a run holds results of, for `render_gpu` and for `compare`, whose own
copy it replaces; it has a test of its own. The NVML thread's counting is not probed: making
NVML fail is outside what this machine can do on demand, as in the review. The five checks are
green. Committed as `fix(bench): count GPU readings that fail and show scenarios without any`.

Batch 11 (finding 13), test-first. `Installed` gains `DanglingLink { target }`, and loses
`Copy`. With the variant in place and `init` treating it as `AlreadyExists`, two tests failed
on their assertions: the adapter, on a link to a missing file in a temporary directory,
answered `AlreadyExists` (the finding's reproduction), and `init` said "already exists". The
fix: the inventory file tells a link whose target `metadata` reports as not found, and leaves
it; any other error reading the target is not taken for a missing file. `init` names the link
and its target, and says to restore the target or remove the link. `load` still reads such a
link as no inventory, so `doctor` points to `init`, which now explains. The five checks are
green. Committed as `fix(init): name a link to a missing inventory and its target`.

Batch 12 (findings 62, 14), test-first.
- Finding 62: `files::read_link_if_present`, a missing link as `None` and any other error as
  "cannot read"; its test failed on a stub, then both stores' `current` used it.
- Finding 14: both test stores gained a `current` link that names nothing until `set_current`
  replaces it, with the real adapters' messages. `list` and `use_build` take a `warn` sink,
  which presentation writes to stderr as `warning: …`, as `build`'s events are. Three tests
  failed first, each command stopping on the link (the finding's reproduction): `build`;
  `build list` then `build use`; `gateway install`. A fourth, passing at once, pins `prune`'s
  refusal. The fix: `current_or_none` reads the link leniently for the commands that set it,
  warning that the build, `build use`, or `ai-ctl build use` / `ai-ctl build` replaces it;
  `gateway install` does the same with "the installed release replaces it". `build` reads it
  once, under the lock, for both its uses. `prune` and `FsBuildStore::remove` stay strict.

Probed with the binary on a scratch data directory holding `current -> ../llama.cpp/b11538`:
`build list` warned and listed, `build prune --keep 0` refused with the link's message and left
it. The five checks are green. Committed as `fix(build): replace a current link that names no build instead of failing`.

Batch 13 (finding 15), test-first. A write that fails once the file is open is made with a link
to `/dev/full` planted at the temporary's name, which gives ENOSPC like a full disk. So that the
test names the temporary without copying its format, `files::temporary_for` gives it, and
`write_new` and the run store both used it first, the run store's behaviour otherwise
unchanged (its temporary's name gained `.tmp`). The test then failed on its assertion: the save
failed with ENOSPC and left `.<id>.json.<pid>.tmp` behind, the finding's reproduction. The fix:
the run store publishes through `files::write_new` with mode 0666, less the umask, as
`fs::write` gave; `write_new` removes its temporary whatever happens, and its `false` becomes
"a run is already saved as …". It also fsyncs the run before linking it. The module's doc
comment names the helper and the new temporary. The five checks are green. Committed as
`fix(bench): publish a saved run through files::write_new`.

P1, test-first. With `v263/` left without its binary, doctor's remedy is `gateway install
--version v263`; `is_installed` says no, so the release is downloaded and staged, and
`publish` refused because `v263/` exists. The test, on the real store: install v263, remove its
binary, stage and unpack v263 again, publish. It failed on its assertion with that refusal. The
fix: `publish` refuses an installed release only, as `is_installed` tells it, and removes what
is left of one whose binary is gone before moving the staged release in, under the install
lock. The port's doc comment says so. My first expectation of the store's entries listed a
`.lock` the fixture never takes; corrected to `v263` alone, as the neighbouring test has it.
Doctor's remedy is unchanged and now ends in a working install. The five checks are green.
Committed as `fix(gateway): reinstall a release whose binary is missing`.

Batch 52 (findings 101, 102), test-first. Read first: `docs/toolchain.md` §2 and §3, whose
removal commands the remedy gives. Two tests, each failing on its assertion:
- toolkit 13.5.10 beside a driver at CUDA 13.4, without then with `cuda-toolkit-13-4`: the
  remedy was `sudo dnf install cuda-toolkit-13-4` alone, the reproduction of finding 101;
- 13.2 (`before_the_upgrade`) and 13.5, each with the unversioned `cuda-toolkit`: "Toolkit
  packages" pinned `cuda-toolkit-13-2`, the reproduction of finding 102; both checks must name
  `cuda-toolkit-13-4`.

The fix: `Setup` holds the installed toolkit packages, read once for both checks. A toolkit
newer than the driver gets `newer_toolkit_remedy`: the install steps when the driver's
versioned package is missing, then `dnf remove` of the newer meta-package and of that
version's leftover packages, then a last line, "to keep CUDA 13.5 instead", with the two
`alternatives --set` of §3. A driver at CUDA 13.2 still gets "update the NVIDIA driver", with a
13.4 toolkit too, an assertion added with the fix. "Toolkit packages" keeps the installed
release only when `toolkit_problem` accepts it. `docs/toolchain.md` §2 step 5 says what the
remedy does for a newer toolkit. The finding's "name the toolkit found when it comes from
another `PATH` entry" needed nothing: the check's detail already names `nvcc`'s path. The five
checks are green. Committed as `fix(doctor): remove a toolkit newer than the driver, and pin the driver's`.

Batch 26 (findings 85, 86, P3). Findings 85 and 86 are tests of code that is right: a dashed
`files` key, and each flag ai-ctl sets (`host`, `port`, `alias`, `metrics`, `ctx-size`,
`model`) refused in the defaults. Both passed at once, and each failed under the review's
mutant (the `files` loop removed; the list shrunk to `port` and `ctx-size`). P3, test-first:
`files = { port = "x.gguf" }` and a default `mmproj` beside `files.mmproj` were both accepted,
two `--port` and two `--mmproj` in the command; each test failed on `expect_err`. The fix: a
model's file keys are refused when ai-ctl sets them, `model` excepted, the required file ai-ctl
takes `--model` from; a default is refused when a model's files set it, with the scope
"defaults.llama-server, with model `…`'s files". The first version also reported a default
`model` a second time beside its reserved-flag error; the check now leaves out the flags the
defaults are refused for already, one error per mistake. This machine's installed inventory
still validates (`gateway config`). The five checks are green. Committed as `fix(inventory): refuse file keys and defaults that repeat a flag`.

L2, probed then fixed: **confirmed**. Podman 5.8.7, `podman ps --filter name=ai-ctl- --format
'{{.ID}}'` under `CONTAINERS_STORAGE_CONF` naming a storage with `driver = "bogus"`, its
graphroot and runroot in the session's scratch directory, so that no real storage was touched:
exit 125, and stderr holds a logrus line, `time="…" level=error msg="Failed to GetDriver graph
bogus …"`, then `Error: configure storage: failed to GetDriver graph bogus …: driver not
supported`. `Output::failure()` kept the first, which lacks the cause. The capture is the
fixture `podman-5.8.7-ps-bogus-storage-driver.txt`, verbatim, scratch path included. The test
failed on it, `failure()` returning the logrus line. The fix: `failure()` takes the first
non-empty line that is not a logrus line (`time="` and ` level=`), and falls back to the first
line when every line is one, as a second assertion pins. nvidia-ctk logs the same way: `cdi
list` writes `time="…" level=info msg="Found 3 CDI devices"` to stderr here. The 20 call sites
of `failure()` gain it at once. The five checks are green.

L2 committed as `fix(command): give a failure's error line, not the log lines before it`. Session closed at the user's request, before the probe under load: the user asked to run the
GPU probe and the bench runs at the next session. Every task of the phase is ticked but batch
54, whose code is committed (`feat(bench): keep NVML's clock-event mask and show a policy limit apart`) and whose probe remains. Nothing ran on the GPU in this
session; no gateway was started. Each batch was committed as soon as its checks were green, as
the user agreed at the start of the session.

**Where the next session starts:**
1. The probe under load, one run for two checks: `ai-ctl up`, then `ai-ctl bench run
   qwen3.8-27b --repeat 1` (every scenario, the GPU busy for some minutes), then `ai-ctl
   down`. From the saved run (format 2), read each GPU reading's `clock_events` scenario by
   scenario: whether BoardLimit (0x200) or Reliability (0x400) stays set under load, beside
   the power cap. While the gateway runs, measure the session's extra tokens a turn with the
   real template (`/upstream/qwen3.8-27b/apply-template` and `/tokenize`, as
   `Prompts::turn_extra` does), and check that the last turn's prompt tokens and 512 stay
   under 22,529 + 9 × that extra (batch 7's premise).
2. Decide, from what NVML reported, whether a policy limit counts as a slowdown in "slowed by";
   if it does, the change and its test, test-first. Record the probe's readings in this report,
   then tick batch 54.
3. Close phase 1 with the roadmap skill (Files Changed from `docs(v2-review-fixes): close phase 0, security`, the Assessment, the
   checks), then open phase 2.
- Questions put to the user: none.
- Pending: nothing uncommitted.

Session resumed for batch 54's probe. Found uncommitted at the start, outside this roadmap:
`CLAUDE.md` and `README.md` modified, `Makefile` and `docs/commands.md` new, the user's work
since the last session. Left alone, and kept out of this phase's commits.

The probe under load. `ai-ctl up` (llama-swap v263, build b11538, driver 615.78.08). With the
model loaded, the session's extra tokens a turn were measured with the real template, as
`Prompts::turn_extra` does: `apply-template` of `agent("")` with and without an empty reply and
an empty turn, then `/tokenize`. The result is 26 tokens: the template's `</think>`, its role
markers and the `TURN` line. Then `ai-ctl bench run qwen3.8-27b --repeat 1`, every scenario,
saved as `20261016T194820Z-qwen3.8-27b` in format 2 with 1,742 readings, then `ai-ctl down`.

What NVML reported, read from the saved run's `clock_events`:
- every reading holds one bit, SwPowerCap (0x4, 1,628 readings) or Reliability (0x400, 114),
  never both; BoardLimit (0x200), heat and the hardware never appear;
- Reliability came in 43 stretches, most of them one NVML update (two readings at 250 ms):
  - the first reading, before any scenario;
  - the gaps between requests, the clock at its 1,507 MHz floor and the power below 80 W;
  - 26 readings under load, at 150 to 171 W, just under the capped readings' 173 W. Their SM
    clock is 0.98 of the capped readings around them in the median (0.83 to 1.07);
- by scenario, Reliability holds 3 % of the depth readings, 8 % of the prefill, 6 % of the
  session and 35 % of the overhead, whose 16-token requests leave the GPU waiting.

So Reliability is the reason NVML gives when the power limit does not hold the clocks, not a
limit beside it. It does not count as a slowdown (see Decisions). The summary already leaves it
out of "slowed by", so the code is unchanged. `docs/stack.md` §9 records the readings, and the
README says what the "policy limit" column shows.

Batch 7's premise, checked on the same run. Each later turn added 1,558 to 1,564 tokens: 1,024
of the corpus, the reply's 512, and 22 to 28 more. The last turn's 22,238 prompt tokens and its
512 make 22,750, under the plan's peak of 22,528 + 9 × 26 = 22,762 (22,763 with the free token).
The premise holds, with 12 tokens to spare over the session, though the second turn went 2
tokens over its share (see Problems And Deviations).

`bench list` lists the seven saved runs, and `bench compare` reads all seven at once, formats 1
and 2 side by side. Batch 54 ticked. The five checks are green. Committed as `docs(bench): record what NVML's policy reasons report under load`, with
only this batch's paragraph of `README.md` staged; the user's line in its Documentation section
stays uncommitted.

Closure. Read the later phases' tasks against what this phase changed. Phase 2's batch 16 asks
for finding 67's test, a running gateway whose deleted ELF binary reads `Running`; batch 6's
test already covers it. Checked in a scratch copy of `HEAD`: under the mutant
`liveness_deleted_binary_not_running`, whose line batch 6 kept verbatim, the test fails, the
removed binary reading `Reused`. The batch's task now says so.

---

## Decisions

- **Each batch is committed as soon as the five checks are green** (the user, 2026-10-16),
  without being put to the user first.
- **`build prune` refuses the whole prune, not the served build alone.** Nothing is removed
  when the running gateway serves a build prune would remove, or runs without its
  configuration; the message names the builds and `ai-ctl down` then `ai-ctl up`. Skipping
  the served build would leave a prune that reports success while the build it was asked to
  remove stays. Later phases keep this: a refusal removes nothing.
- **`build prune` holds the gateway's lock besides the builds' lock.** `up` reads the current
  build, then may pull images for minutes before it records the gateway; a `build` and a
  `prune` in that window would remove the build `up` is about to start. With the lock, prune
  fails at once while `up`, `down` or `status` runs, as every lock of ai-ctl does.
- **A session's plan counts each later turn's own tokens, measured through the model's
  server** (`Prompts::turn_extra`): the `TURN` preamble and the template's markers vary with
  the model, so no constant would hold. `Plan::new` takes the selected scenarios and the
  measured tokens; the session's fit is checked only when the session is selected. The saved
  plan's fields are unchanged, so every saved run still reads.
- **A failed request still ends `bench run`, and the scenarios completed before it are
  saved** as a run of their own, without the failed scenario's results, GPU readings or a
  drafts check. A failure in the first scenario saves nothing. Batch 55's warning about failed
  GPU readings and batch 54's `policy` column apply to such runs as to any other.
- **The run format is 2; this ai-ctl reads 1 and 2.** Format 2 adds each GPU reading's
  `policy` and `clock_events`, optional on reading. An older ai-ctl refuses a format-2 run by
  its number rather than on an unknown field. A later change to the format bumps it again and
  keeps reading the formats before.
- **A `current` link is read leniently only by the commands that replace it** (`build`,
  `build use`, `build list`, `gateway install`): any error reading it, a target that names no
  build or release or an I/O error, becomes a warning and no current. `prune`, `remove`,
  doctor and the configurator read it strictly, since a guess there could remove or run the
  wrong build.
- **Whether a configuration starts a build is read from its text, in the domain**
  (`gateway_config::starts_program`): the binary's line, quoted as `to_yaml` quotes it. The
  configuration is ai-ctl's own output; the record is unchanged.

- **`docs/stack.md` §7 keeps the facts about llama-swap and llama-server; `docs/security.md`
  says what follows from them.** CLAUDE.md puts the facts about llama-server in `stack.md`, with
  sources. The security document repeats only what an argument needs and links to the section.
  A later change to the gateway's or the backends' behaviour updates both.
- **ai-ctl creates its state directory at mode 0700** (approved by the user, 2026-10-16). The
  directory, not each file: the gateway's log, its configuration, its record and the build log
  are then the user's whatever the home directory's mode, and no file needs a mode of its own.
  The data and configuration directories keep the umask's mode: builds, releases, bench runs and
  the inventory hold no secret. An existing directory is not narrowed by ai-ctl; this machine's
  is narrowed by hand, the only installation.
- **NVML's Reliability reason is not a slowdown, and BoardLimit stays beside it.** Under load
  on this machine, Reliability alternates with the power cap and never coexists with it. It
  marks the readings the power limit does not hold: at idle, between requests, and at dips just
  under the cap, the clock then within a few percent of the capped readings. Counting it in
  "slowed by" would show the overhead scenario 35 % slowed by its own pauses. It keeps its own
  "policy limit" column. BoardLimit never appeared, and stays in that column until a run shows
  what it reports.

---

## Files Changed

**Added**
- `Makefile` — the user's work, outside this roadmap, left uncommitted
- `docs/commands.md` — the user's work, outside this roadmap, left uncommitted
- `docs/roadmap/on-progress/v2-review-fixes/phase-1-bugs-and-error-handling-report.md`
- `docs/security.md`
- `tests/fixtures/podman-5.8.7-ps-bogus-storage-driver.txt`

**Modified**
- `CLAUDE.md` — also holds the user's uncommitted line on `docs/commands.md`, outside this
  roadmap
- `README.md` — also holds the user's uncommitted line on `docs/commands.md`, outside this
  roadmap
- `docs/roadmap/on-progress/v2-review-fixes/README.md`
- `docs/roadmap/on-progress/v2-review-fixes/phase-1-bugs-and-error-handling.md`
- `docs/roadmap/on-progress/v2-review-fixes/phase-2-dead-code-and-tests.md`
- `docs/roadmap/on-progress/v2-review-fixes/phase-4-structure-and-refactoring.md`
- `docs/stack.md`
- `docs/toolchain.md`
- `src/application/bench.rs`
- `src/application/build.rs`
- `src/application/fakes.rs`
- `src/application/gateway.rs`
- `src/application/init.rs`
- `src/application/lifecycle.rs`
- `src/application/ports.rs`
- `src/application/toolchain.rs`
- `src/domain/bench.rs`
- `src/domain/gateway_config.rs`
- `src/domain/inventory.rs`
- `src/infrastructure/api_key.rs`
- `src/infrastructure/build_store.rs`
- `src/infrastructure/cmake.rs`
- `src/infrastructure/command.rs`
- `src/infrastructure/files.rs`
- `src/infrastructure/gateway_process.rs`
- `src/infrastructure/gateway_store.rs`
- `src/infrastructure/inventory_file.rs`
- `src/infrastructure/nvml.rs`
- `src/infrastructure/run_store.rs`
- `src/presentation/bench.rs`
- `src/presentation/build.rs`
- `src/presentation/cli.rs`
- `src/presentation/compare.rs`

---

## Problems And Deviations

- **The security document covers two subjects the task does not list**: llama.cpp's sources,
  fetched from GitHub with no signature checked and tags fetched with `--force`, and the files
  ai-ctl keeps with their modes. A document on downloads that left out the source of every
  backend would mislead.
- **ai-ctl's state files rely on the home directory's mode.** The gateway's log, its
  configuration and the build log are created with the umask's mode (0644) in directories of
  mode 0755. On this machine `~` and `~/.local` are 0700, so no other user reads them. Under an
  `XDG_STATE_HOME` other users can enter, they could read the log: model loads, paths, timings,
  token counts, no prompt at the default verbosity. The key is not exposed: it is written at
  0600. Recorded in `docs/security.md` §8 and put to the user, who approved a fix in this phase:
  the task "State Directory".
- **Batch 6 resolves a removed binary from its nearest existing ancestor, not from its parent
  only**, as the task and the finding's Fix had it. With the parent only, a release directory
  removed by hand under a linked data directory would read `Reused`, and `status` would forget
  a gateway that still runs, where the same removal under a real directory reads `Running`.
  The test covers the removed directory.
- **A session's plan holds by 12 tokens, not by construction.** `turn_extra` counts a turn's
  markers. A corpus slice and a reply, tokenized again inside the history, vary by a few
  tokens: on the probe's run, one turn took 2 tokens more than its share, and the session ended
  12 under the plan. Only a model whose context lies within some 30 tokens above 22,763 could
  see its last turn overflow; the inventory's model holds 98,304. Left open and put to the
  user: no finding covers it.

---

## Changes To Later Phases

- `phase-4-structure-and-refactoring.md`, batch 46: notes that `live_record` exists, added by
  batch 8 for `build prune`; the batch moves `log`, `model_log` and `Bench::listen` onto it.
- `phase-2-dead-code-and-tests.md`, batch 16: notes that finding 67's test exists since batch 6
  and fails under the finding's mutant, so the batch adds none for it.

---

## Assessment

Phase 1 delivered its sixteen tasks:
- the security document, carried over from phase 0;
- the state directory at 0700, added once that document was written;
- the review's batches 6 to 13, 26, 52, 54 and 55;
- P1 to P4;
- L2, confirmed and fixed.

Each fix started from a test that reproduced its finding, watched failing on its assertion.
The test findings (85, 86, and batch 10's `load` and `saved`) passed at once, as the code was
right, and failed under their mutants. Every acceptance criterion holds.

Beyond the findings' letter, the phase settled six behaviours that later phases keep:
- `build prune` refuses the whole prune while the gateway serves a build it would remove, and
  holds the gateway's lock;
- a failed bench request saves the scenarios completed before it;
- the run format is 2, and ai-ctl reads formats 1 and 2;
- a `current` link is read leniently only by the commands that replace it;
- a removed binary is resolved from its nearest existing ancestor;
- NVML's Reliability reason stays out of "slowed by".

The probe under load showed Reliability to be, on this machine, the reason NVML gives where the
power cap does not hold the clocks, not a limit of its own. The same run confirmed the
session's plan, by 12 tokens; that margin is left open and put to the user.

What phase 2 needs to know first: its tests are written against this phase's code, not the
review's line numbers. The fakes already hold:
- `FakeRuns`;
- `FakeMonitor::lost_from`;
- the started gateway's `wait_failures`;
- `FakeModelServer::ctx_size`;
- the test stores' broken `current` links.

Batches 18 and 19 add their failure switches beside these. Batch 24's slowdown tests take
`Slowdown::policy` and `GpuReading::clock_events`. Finding 67's test exists already (batch 16).
