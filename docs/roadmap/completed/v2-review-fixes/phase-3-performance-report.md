# Phase 3 Report: Performance

**Phase:** [phase-3-performance.md](phase-3-performance.md)
**Start Commit:** `docs(tests): index the fixtures with how each was captured` of 2026-10-16

---

## Work Log

### 2026-10-16

Phase opened at `docs(tests): index the fixtures with how each was captured`. Phase 2 closed at `docs(roadmap): close phase 2 of v2-review-fixes, dead code and tests`; three documentation commits followed,
outside both phases: the driver update in `docs/toolchain.md`, the gateway's record and lock in
`docs/security.md` with the README's status, and the fixtures' index,
`tests/fixtures/README.md`. Read before working: phase 2's file and report in full. No
restructuring was pending. What carries into this phase:
- each batch is committed once the five checks are green, and each change is measured on this
  machine before and after;
- a command reads the GPU once and passes the reading on (phase 2, batch 53); batch 31's NVML
  keeps that;
- the tests now hold the code these batches rewrite: batch 18's timeout test asserts the health
  check's last error, not the count of polls (batch 29); batch 28's script test runs
  `podman pull` without a timeout (batch 32); `FakeProcesses::descendants_fails` and the owners'
  tests use `descendants` (batch 34); finding 89's boundary test follows `Tool::minimum`
  (batch 33);
- a new fixture gets its row in `tests/fixtures/README.md`, with its capture;
- the llama.cpp build in use is `b11700` since 2026-10-16, and the driver 615.78.08 since
  2026-10-15: each before-and-after figure is taken on the same build and driver, and the report
  names them.

`docs/commands.md`, the Makefile, and one line each of `CLAUDE.md` and `README.md` hold the
user's uncommitted work, kept out of this phase's commits.

Work resumed in a new session: the phase file and this report read in full, then the review's
findings 16 to 24, 108, 89, E1, E3 and E5. Baseline taken on a release build of `docs(tests): index the fixtures with how each was captured`,
kept in the scratchpad so that each batch is compared with the same binary, driver 615.78.08,
llama.cpp `b11700`, gateway stopped, the dGPU `active` and held by gnome-shell, wall clock by
a script alternating the commands, output to `/dev/null`:
- `doctor`: 682 to 704 ms, median 694 (n=10), 23 ok;
- `status`: 40.9 to 42.6 ms, median 41.5 (n=20), exit 3 (gateway stopped).

Batch 29 (finding 16). First the plumbing, with the behaviour unchanged: `Lifecycle` and
`Services` take `now: fn() -> Instant`, `main.rs` gives `Instant::now`, and
`GatewayApi::healthy` takes the check's timeout, which the adapter passes to `http::request`
in place of its own 2 s. `http::send` now connects within the shorter of its 2 s and the
request's timeout; every other caller gives 5 s or more, or none, so only the health check
changes. The tests' clock is a thread-local instant in `fakes.rs`, `now()` and `advance()`, the
fake `pause` advancing it: a plain `fn() -> Instant` carries no fake's state, as the review's
verification said, but a thread-local does. The five checks green, 636 tests.

Then two tests, watched failing on their assertions with the review's own figures:
- `a_gateway_that_accepts_and_never_answers_holds_up_30_s`: `FakeApi.hangs` makes each check
  wait its whole timeout on the test's clock, then fail with the error the real adapter gives;
  `up` took 630 s;
- `a_slow_gpu_reading_does_not_stretch_the_release_wait`: `ScriptedGpu.read_time` at 28 ms,
  the cost of a read with this driver (finding 18); `down` took 38.4 s.

The fix: both loops run until a deadline 30 s after they start, `time_left` giving the rest;
each pause is cut to the time left, and each health check gets the time left or 500 ms,
whichever is shorter. `up` then fails at 30 s exactly. The release test first gave 30.056 s
against the 30.028 s expected: the 28 ms more are `down`'s own reading before the wait, which
the expected figure had left out; corrected in the test, not in the code. The release wait
reads the GPU once more at the deadline, so its last reading is never skipped.
`a_gateway_that_never_answers_is_stopped_and_forgotten` still sees 300 checks, one per 100 ms
for 30 s.

Measured, the way the finding did: a scratch crate in the session's scratchpad, depending on
the repository by path, times the real `LlamaSwapApi::healthy` against a listener that
accepts through the kernel's backlog and never answers. With 2 s, today's behaviour: one check
2.03 s, ten polls 21.16 s, the review's figures. With 500 ms: one check 0.508 s, ten polls
6.08 s. `up` against such a gateway therefore fails after 30 s instead of about 630 s, and
`down`'s release wait ends at 30 s instead of about 38 s. The error text, `Resource
temporarily unavailable (os error 11)`, is the one the probe printed, and the fake quotes it.
The five checks green, 638 tests. Committed as `fix(lifecycle): bound up's and down's waits by a deadline`.

Batch 30 (finding 17). The probe ran `rpm` from the process's `PATH`, out of a script's reach;
it now finds `rpm` on the host's `PATH`, as `busctl` is found, and an absent `rpm` is
`Missing`. The test `the_toolkit_packages_are_listed_without_checking_digests` puts a script
named `rpm` on that `PATH`, which prints the fixture only when called with the new
arguments; it failed on its assertion with `Failed("unexpected: -qa --qf %{NAME}\\n
cuda-toolkit*")`, then passed with `--nodigest --nosignature` added.

On this machine, 2,637 installed packages: both forms print byte-identical output (`cmp`).
`rpm-cuda-toolkit-packages.txt` lists the same four names in another order, rpm's database
order of the earlier capture, which the parser sorts; it stays as the fixture of the new
form too. Measured, 8 alternated runs each: the query 418 to 443 ms, median 422, against 88
to 98 ms, median 90; `doctor`, the release build of `docs(tests): index the fixtures with how each was captured` against this batch's, 740 to
774 ms, median 762, against 377 to 400 ms, median 380, the output identical. The machine ran
slower than at the baseline (694 ms then); the alternation keeps the comparison fair. The five
checks green, 639 tests. Committed as `perf(doctor): list the toolkit packages without checking rpm digests`.

Batch 31 (finding 18). Counted first, with `LD_DEBUG=files` on the release build of batch
30: `libnvidia-ml` is loaded once by `doctor`, once by `status`, never by `build list`. The
review counted two loads in `doctor`; batch 53 of phase 2 has since made it read the GPU
once. So the batch changes neither command's time; it serves the commands that read the GPU
again in one process: `down`'s release wait at each poll, `bench`, `build`.

The change: `nvml()` holds the process's `Nvml` in a `static OnceLock`, initialised by the
first read, and `NvmlProbe` and `NvmlMonitor`'s thread both read through it. A failed
initialisation stays failed for the process. The monitor's first error now crosses its
channel as text, since the shared error is borrowed; its message is unchanged. The doc comment
says that nvmlShutdown is skipped at exit and that NVML's descriptors are close-on-exec.

No unit test: no test can run NVML without a GPU (CLAUDE.md, Tests), and the fake GPU ports
stand in front of the adapter, not inside it. The proof is the review's own probes, rerun:
- a scratch crate, depending on the repository by path, reads `NvmlProbe` 40 times in one
  process. Before: 15.5 to 19.2 ms a read, median 16.7. After: the first 20.2 ms, the others
  0.112 to 0.151 ms, median 0.114. Before, no NVIDIA descriptor stayed open; after, the
  process holds 5, and a child it spawns holds none, so they are close-on-exec;
- ten runs of that crate exiting without nvmlShutdown, then `nvidia-smi` answers and a fresh
  process initialises NVML again, first read 15.4 ms, the others 0.07 ms;
- `LD_DEBUG=files` after: one load in `doctor`, one in `status`, none in `build list`;
- `doctor` 357 to 369 ms, median 358, against 357 to 368, median 358 (n=8, alternated with
  batch 30's build); `status` 40.5 to 42.2 ms, median 41.1, against 39.4 to 41.0, median
  40.2 (n=20); outputs identical.

A read costs 15 to 20 ms here, against the review's 23.7 to 34.5 ms: the same driver, the dGPU
`active` as then. So `down`'s release wait now polls at about 0.1 ms a read instead of 17,
once its first read is done. The five checks green, 639 tests.

Batch 31 committed as `perf(gpu): initialise NVML once per process`. Batch 32 (finding 19, E1), stopped midway by the plan's
usage limit, uncommitted in `src/infrastructure/command.rs`:
- `a_grandchild_holding_the_pipe_does_not_outlive_the_timeout` reproduces E1: `sh` exits at
  once and leaves a `sleep 4` holding its stdout; with a 1 s timeout, `execute` returned `Ok`
  after 4.00 s, watched failing on its assertion. The message prints whole seconds, as every
  production timeout is whole, so the test takes E1's 1 s rather than change the format.
- The fix: without a timeout, `execute` waits with `child.wait()`, no poll. With one, each
  drain thread sends its text over a channel, `collect` waits for it with `recv_timeout`
  until the deadline, then `exit_by` checks the exit with `try_wait` every 1 ms
  (`EXIT_POLL`, which replaces the 10 ms `POLL`) until the same deadline: a child whose
  pipes are closed is exiting, or runs on without them. Past the deadline the child is
  killed and waited, with the same error; a grandchild keeps its drain thread until it
  ends. The E1 test passes in 1.00 s, the 12 command tests green, fmt and clippy clean.
- `a_child_that_closes_its_output_and_runs_on_is_killed_at_its_timeout`
  (`exec >&- 2>&-; exec sleep 4`) covers the branch after EOF; meant to pass before and
  after the fix, it guards against a bare `wait()` after EOF. Written, never run.

**Where the next session starts:**
1. Run `cargo test --lib command` for the second test, never run; then the review's probe
   for finding 19, before and after, alternated with the build of `perf(gpu): initialise NVML once per process` as batches 30
   and 31 did.
2. The five checks; batch 32's Work Log entry; tick batch 32, the phase's status and the
   README's progress block (4/7); commit `src/infrastructure/command.rs` with the roadmap
   folder.
3. Batch 33 (finding 20, E3, E5), the Git section.
- Whose are `Makefile`, `docs/commands.md` and the line each of `CLAUDE.md` and `README.md`
  gives it? Not this phase's work: left out of its commits until the user says.
- Uncommitted: `src/infrastructure/command.rs`, batch 32 as above.

### 2026-10-17

Work resumed after the close above, in the same session, once the usage limit had reset: the
phase file and this report read in full, then finding 19 and E1. The close's question on
`Makefile`, `docs/commands.md` and their lines in `CLAUDE.md` and `README.md` was already
answered by the phase's opening entry: the user's work, kept out of this phase's commits.

Batch 32, finished. The second test passes; with `exit_by` replaced by a bare `child.wait()`
it fails on its assertion, `Ok` after 4.00 s, so it holds the bounded wait after EOF.

Measured as the review did: release builds of `perf(gpu): initialise NVML once per process` of 2026-10-16, from a scratch worktree since
removed, as the scratchpad of 2026-10-16 had been cleared, and of this batch; driver
615.78.08, llama.cpp `b11700`, gateway stopped, the dGPU `active`, platform profile
`performance`; alternated, output to `/dev/null`:
- `doctor`: 507.3 to 558.3 ms, median 522.0, against 433.3 to 496.6, median 460.9 (n=8); a
  second series 514.6 to 587.7, median 536.1, against 448.5 to 508.8, median 468.1; the
  output identical (`cmp`). 61 to 68 ms less, 12 to 13 %, the review's 13 %;
- `status`: 54.9 to 68.9 ms, median 59.1, against 51.3 to 61.9, median 56.8 (n=20), the
  ranges overlapping.

The machine runs slower than on 2026-10-16 (`doctor` 358 ms at batch 31); the alternation
keeps the comparison fair. No document named the poll. The five checks green, 641 tests.
Committed as `perf(command): wait for commands without a sleep loop`.

Batch 33 (finding 20, E3, E5). First the plumbing, the behaviour unchanged: `Sources` gains
`fetch`, the clone or update `checkout` ran first and still called; `Event::Fetching` names
nothing and `CheckedOut` names the reference, `Checked out b11538 (79e2e74eb1) as build
b11538`; `FakeSources` records its calls in turn and takes its tags. 641 tests green.

Then six tests, watched failing on their assertions: in `build.rs`, the calls of a build of
`latest`, implicit and named, `["release_tags", "checkout b11538"]` against `["fetch",
"release_tags", "checkout b11538"]`, a named branch built with no fetch, and a clone with no
release tag, whose error quoted `refs/tags/b[0-9]*`; in `git.rs`,
`release_tags_are_read_from_the_clone`, which drops the upstream after the fetch and failed
on `ls-remote`, and `a_checkout_needs_a_fetched_clone`, which got a clone. Git's minimum test
failed on 2.18.0 against 2.19.2. `a_commit_no_branch_holds_is_found_by_its_hash`, a commit
on a branch deleted upstream before the clone, passed before the fix: it holds E3's removal.

The fix: `build` emits `Fetching`, fetches, then resolves `latest` from `release_tags`, now
`git tag --list 'b[0-9]*'` in the clone; `Event::Resolving` is gone, since a 2 ms lookup
needs no line of its own. `checkout` no longer fetches, and refuses a missing clone. E3:
`resolve`'s explicit fetch of a full hash is removed and its doc comment corrected; the E3
test still passes, the partial clone fetching the commit by itself. `parse_ls_remote_tags`,
its two tests, and `tests/fixtures/git-ls-remote-tags.txt` with its row in the fixtures'
index are removed; `TagPattern::ls_remote_pattern` became `glob`, `b[0-9]*`. E5: git's
minimum is 2.19.2, from the release notes in `/usr/share/doc/git/RelNotes`: 2.17.0 brought
the filtered clone and fetch as a foundation; 2.19.0 made `clone` validate the objects it
receives and tolerated missing promised objects; 2.19.2, then 2.20.0, fixed "the code to
backfill objects in lazily cloned repository"; 2.23.0's partial clone entry is about
`--origin`, which the code does not use. Finding 89's boundary test reads `Tool::minimum` and
follows. `docs/toolchain.md` and `README.md` follow, the build's steps and `doctor`'s
minimum; the README's commit holds these lines, not the user's. `docs/security.md` already
put the fetch before `latest`.

Measured as the review did, in a copy of the sources clone, since deleted, after a fetch of
1.20 s; git 2.56.0, alternated, n=5: `ls-remote` 731 to 890 ms, median 758; `git tag
--list` 2.5 to 2.7 ms, median 2.5; the same 7,292 tags, newest `b11707`. A `build` without
`--ref`, its refusal of a build already made included, saves that lookup. End to end, the
release build on the real clone: `build llama.cpp --ref b11700`, the current build, printed
`Fetching the sources` and `Checked out b11700 (23f7aabc16) as build b11700`, then refused
in 1.46 s, exit 1, nothing compiled. The five checks green, 642 tests. Committed as
`perf(build): read the newest release from the clone after the fetch`.

Batch 34 (findings 21, 108). Probed first, for the parsers' fixtures: a container of
`node:22-bookworm-slim` running `sleep` (`podman run -d --rm --pull=never`, a name outside
`ai-ctl-`, stopped after). Its chain: `sleep` → `conmon` → `systemd --user` (4571) → 1; the
`sleep` in `…/libpod-<id>.scope/container`, `conmon` left in its caller's scope, the VS Code
app's. `gnome-shell`, which holds the dGPU, sits under the same `systemd --user`, in
`session.slice/org.gnome.Shell@user.service`. Captured as four fixtures with a section of
their own in `tests/fixtures/README.md`, a program named `x) R 1 (y` among them.

The adapter first, test-first: `ProcessStat { name, parent }`, and `GatewayProcesses` gains
`stat`, read from `/proc/<pid>/stat`, and `in_container`, whether `/proc/<pid>/cgroup` names
a `libpod-` scope; a process gone is `None` and `false`. The two parsers, stubs at first,
failed on their assertions over the fixtures, then passed; `process_stat` replaced
`parent_pid`. A child of the test process names it as its parent, under the name `sleep`.

Then the test worlds got their process trees beside `descendants`, the behaviour unchanged,
and five tests: three failed on their assertions, `an_idle_gpu_asks_podman_nothing` (podman
listed once against none), a stat that cannot be read (5000 still released), and a cgroup
that cannot be read (no warning); three passed already and hold the new code to the old
results: a container whose conmon is gone, found by its cgroup; another container's process
under conmon, not ai-ctl's; a loop of parents.

The fix: `owners()` takes the GPU reading and walks up from each of its processes, stopping
at PID 1, at a PID met twice, or at a stat that cannot be read, with a warning when the
process is not gone. A chain that reaches the gateway's PID is the gateway's. Only a chain
that reaches a `conmon`, or a process whose cgroup names a container, asks podman, once per
call, and its containers' PIDs are matched against the chain. `descendants`, its fake, its
adapter and its two tests are removed, which also removes finding 108's rescans. One defect
of the first form, caught by `doctor`'s tests and by
`beside_a_running_gateway_only_another_gateways_build_is_left_behind`: a chain kept only the
PIDs whose stat it read, so a gateway whose own stat the fakes did not give was never reached;
the chain now keeps every PID it reaches, with its name when read. The test of a podman list
that fails gained a container's process, since podman is asked only for one.

Measured on release builds of `perf(build): read the newest release from the clone after the fetch` and of this batch, the gateway stopped, the dGPU
`active`, the outputs identical (`cmp`). A `podman` wrapper on the `PATH` logged the calls:
`status` ran `podman ps --filter name=^ai-ctl-` before, nothing after; `doctor` ran `ps` and
`--version` before, `--version` alone after, its own Containers check. Alternated: `status`
33.9 to 37.3 ms, median 35.8, against 18.3 to 19.9, median 19.0 (n=20); `doctor` 256.2 to
263.7 ms, median 260.0, against 237.3 to 248.6, median 242.7 (n=8). The machine ran faster
than for batch 32; the alternation keeps the comparison fair. No document described how the
processes are placed. The five checks green, 648 tests. Committed as `perf(gateway): place GPU processes by walking up their parents`.

Batch 35 (findings 22, 23, 24). Two tests, watched failing on their assertions: a prompt no
attempt fits, the fake's tokenizer overshooting the prefill's 40,960 tokens, failed after the
warm-up had been sent and the GPU watched; a prefill-only run made 5 counts against the 2 it
needs, the agent's bare prompt once and the fresh prompt's one exact attempt.

The fix: `Prompts::load` counts the agent's bare prompt once, after `pieces`, and `agent`
reads it (finding 23). The lead is `prompt::agent` of the corpus's first `cut - own` tokens,
one cut and no count, the `cut <= own` branch kept (finding 24). `Runner::prepare` fits the
fresh prompts of the selected scenarios after the `Planned` event and before the warm-up, as
a `Prepared` per scenario that `measure` takes; the loop runs over them, so the readings'
labels are unchanged (finding 22). The tests on the corpus's tokenization, the boundary
merge, the leads' messages and the readings' labels hold unchanged.

Counted rather than timed, as the review left it unmeasured live: a throwaway test, run in a
worktree of `perf(gateway): place GPU processes by walking up their parents` and in this tree, then removed, counted the template and tokenize
requests of a full run on the fake server, one attempt a fit: 28 counts, 56 requests,
against 9 counts, 18 requests. A real tokenizer, which needs more attempts, saves more. No
live bench was run: it holds the GPU for minutes for each build. `docs/stack.md` still holds,
the lead being the same prompt cut to a quarter. The five checks green, 650 tests.

Batch 35 committed as `perf(bench): fit the fresh prompts before the GPU is watched`. Every task of the phase is ticked; the closure was proposed
to the user, who put it first for the next session. A live bench of batch 35, before and
after, was offered and not answered.

**Where the next session starts:**
1. cloture de phase: phase 3's closure by the roadmap skill, its five checks first.
2. Phase 4, which the closure opens last.
- Is a live bench of batch 35 wanted, before and after, though it holds the GPU for minutes
  for each build?
- Nothing of this phase waits uncommitted. `Makefile`, `docs/commands.md` and their lines in
  `CLAUDE.md` and `README.md` stay the user's, out of the phase's commits.

The phase closed in the same session, at the user's word. The five checks green, 650 tests.
Measured as at the opening, on release builds of `docs(tests): index the fixtures with how each was captured` of 2026-10-16 and of `docs(v2-review-fixes): record the session's close in phase 3`, the end of the
phase's work: driver 615.78.08, llama.cpp `b11700`, the gateway stopped, the dGPU `active`,
platform profile `performance`, alternated, output to `/dev/null`, the outputs identical
(`doctor` 23 ok in both):
- `doctor`: 679.8 to 711.7 ms, median 689.7, against 241.5 to 275.5, median 246.6 (n=10);
- `status`: 40.1 to 42.0 ms, median 41.0, against 19.0 to 21.2, median 19.8 (n=20).

The base agrees with the opening's figures, 694 and 41.5 ms. `doctor` lost 64 % of its time,
`status` 52 %. No `git ls-remote` is left in `src/`. The live bench of batch 35 was left
undone.

---

## Decisions

- **A wait is bounded by a deadline on an injected monotonic clock, `now: fn() -> Instant`;
  tests use the thread-local clock of `fakes.rs`, moved by the fake `pause` and by a fake
  port's call.** A count of polls stretches with each call's duration. Any later wait of a
  use case takes the same clock; an adapter that runs a real program, such as batch 32's
  commands, keeps `Instant::now`. `Bench.clock` stays the wall clock that timestamps runs.
- **A health check waits 500 ms at most, or the time left.** A refused port answers in
  0.1 ms and llama-swap's `/health` from memory, so a longer wait only delays the next
  check.
- **`Sources::fetch` clones or updates the clone; `release_tags` and `checkout` read it as the
  last fetch left it, and `checkout` never fetches.** One fetch per build, and the newest
  release read in 2 ms rather than 0.7 to 1.6 s from the remote. Phase 4's renaming of the
  ports keeps the three steps.
- **Git's minimum is 2.19.2**, the first release whose notes carry the partial clone's fixes the
  code relies on: `clone`'s validation of what it receives (2.19.0) and the lazy backfill
  (2.19.2). A later minimum follows the same release notes.
- **A GPU process is placed by walking up its parents, and podman is asked only for a chain
  that reaches a `conmon` or a process whose cgroup names a `libpod-` scope.** On this machine
  the dGPU always holds gnome-shell, so `status` and `doctor` no longer run `podman ps`. The
  walk misses what `descendants` missed too: orphans reparented to `systemd --user`, which
  `leftovers` still finds by their build-store program, and `podman exec` sessions.
- **The set-aside lead is cut, not fitted, and not counted.** It holds a quarter of the fresh
  prompt's tokens give or take the one where its code ends, far above the 10 % of
  `--slot-prompt-similarity`; a count would only confirm it. The review allowed one count at
  most; none is made.

---

## Files Changed

**Added**
- `docs/roadmap/on-progress/v2-review-fixes/phase-3-performance-report.md`
- `tests/fixtures/proc-cgroup-gnome-shell.txt`
- `tests/fixtures/proc-cgroup-podman-container.txt`
- `tests/fixtures/proc-stat-conmon.txt`
- `tests/fixtures/proc-stat-odd-name.txt`
- `Makefile` — the user's uncommitted work, not this phase's
- `docs/commands.md` — the user's uncommitted work, not this phase's

**Modified**
- `CLAUDE.md` — the phase's closure in Current work; its Commands line is the user's
  uncommitted work
- `README.md` — the build's steps and git's minimum (batch 33); its `docs/commands.md` line is
  the user's uncommitted work
- `docs/roadmap/on-progress/v2-review-fixes/README.md`
- `docs/roadmap/on-progress/v2-review-fixes/phase-3-performance.md`
- `docs/roadmap/on-progress/v2-review-fixes/phase-4-structure-and-refactoring.md`
- `docs/toolchain.md`
- `src/application/bench.rs`
- `src/application/build.rs`
- `src/application/doctor.rs`
- `src/application/fakes.rs`
- `src/application/lifecycle.rs`
- `src/application/ports.rs`
- `src/domain/machine.rs`
- `src/domain/recipe.rs`
- `src/infrastructure/command.rs`
- `src/infrastructure/gateway_api.rs`
- `src/infrastructure/gateway_process.rs`
- `src/infrastructure/git.rs`
- `src/infrastructure/host.rs`
- `src/infrastructure/http.rs`
- `src/infrastructure/nvml.rs`
- `src/main.rs`
- `src/presentation/build.rs`
- `src/presentation/lifecycle.rs`
- `src/presentation/mod.rs`
- `tests/fixtures/README.md`
- `tests/gateway_lifecycle.rs`

**Deleted**
- `tests/fixtures/git-ls-remote-tags.txt`

---

## Problems And Deviations

- **Batch 35 was counted, not timed.** The phase's constraint measures each change the way its
  finding did; the review left findings 22 to 24 unmeasured live, and a live bench holds the
  GPU for minutes for each build. The requests of a full run were counted on the fake server
  instead, 56 against 18. Left as it is; a live bench of both builds remains possible.
- **Batch 34's port methods are `stat` and `in_container`, not the plan's `parent`.** The
  conmon test needs each ancestor's name, which the same `/proc/<pid>/stat` read gives; the
  cgroup test is the second method the review's fix asked for. Phase 4's dependency line
  follows.
- **Batch 31 has no test of its own.** NVML runs only with a GPU, which no test may need; the
  shared initialisation is proven by the review's probes, rerun before and after, and recorded
  in the Work Log. Left as it is: the fake GPU ports cover the use cases that read through it.

---

## Changes To Later Phases

- `phase-4-structure-and-refactoring.md`, batch 50: two more points for `docs/architecture.md`,
  from the documentation pass after phase 2's closure: the Repository Layout's missing entries,
  and the testing conventions phases 1 and 2 settled. The user agreed on 2026-10-16 to fold them
  into batch 50, which rewrites the file after phase 4 renames the ports, rather than write it
  twice.
- `phase-4-structure-and-refactoring.md`, Dependencies: batch 40 splits the port after batch 34's
  `stat` and `in_container`, the names batch 34 gave the methods that replace `descendants`.

---

## Assessment

The phase did what it set out to do: its seven batches fixed the review's performance findings
16 to 24, 107 and 108, and the defects E1, E3 and E5, and every acceptance criterion holds.
On this machine `doctor` went from 690 to 247 ms and `status` from 41 to 20 ms, with the same
output: rpm's digest checks and `podman ps` were most of it, the 10 ms poll of each command
the rest. The waits of `up` and `down` end at a 30 s deadline on an injected clock, a
grandchild no longer holds a command past its timeout, NVML loads once per process, `build`
reads the newest release from its clone, and the bench fits its prompts before the GPU is
watched.

The work found one defect of its own, in batch 34's first form: a chain of parents dropped
the PIDs whose stat it could not read, so a gateway's own PID went unseen; the tests caught it
before the commit. Git's minimum rose to 2.19.2, set from git's release notes. Batch 31 has
no test of its own and batch 35 no live measure, both recorded above.

What phase 4 needs to know first:
- batch 40 splits a `GatewayProcesses` whose process table is now `signal`, `stat`,
  `in_container` and `program`; `FakeProcesses` holds the process tree (`stats`, `spawned`),
  `contained`, and `stat_fails` and `cgroup_fails`, and `FakeContainers` counts its
  `listed` calls;
- batch 42 moves podman's `checked` into a `command.rs` whose `execute` batch 32 rewrote: the
  drain threads answer over channels, and `RunError` is unchanged;
- batch 49's `CommitId::short` has one more user, the `CheckedOut` line;
- `Sources` is now `fetch`, `release_tags` and `checkout`, each reading the clone as the last
  fetch left it; a renaming keeps the three steps;
- batch 50's testing conventions gain the injected `now` clock of the use cases' waits and the
  fakes' process tree.
