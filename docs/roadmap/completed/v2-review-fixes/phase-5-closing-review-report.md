# Phase 5 Report: Closing Review

**Phase:** [phase-5-closing-review.md](phase-5-closing-review.md)
**Start Commit:** `docs(v2-review-fixes): close phase 4, structure and refactoring` of 2026-10-17

---

## Work Log

### 2026-10-17

Phase opened at `docs(v2-review-fixes): close phase 4, structure and refactoring`, the commit that closed phase 4, in the session that closed it. Read
before working: phase 4's file and report in full. No restructuring was pending. What carries
into this phase:
- the review runs on the roadmap's whole diff, phases 0 to 4, through `scope.py roadmap` on
  this folder, and its agents start only after the user's go-ahead on its estimate;
- phase 4's Assessment names three leads for the review: `GatewayProcess` still holds ten
  methods, of which `Bench` calls three; the bench's setup reads the GPU twice, where phase 2's
  rule has the other commands read it once; batches 43 to 48 changed error and status texts;
- `docs/commands.md` and the `Makefile`, the user's, entered the repository during phase 4;
- the fixes follow phases 0 to 4's rules: one batch per commit with the five checks green,
  test-first, and a test finding's test killing its mutant.

Resumed in a new session. Read before working: this phase's file and report, and phase 4's
report in full. Loaded `reviewing-delivered-code` for the first task, and ran its step 1:
`scope.py roadmap docs/roadmap/on-progress/v2-review-fixes --files` exited 0, with base
`docs(v2-review-fixes): create the roadmap` of 2026-10-16, phase 0's start commit, and end `docs(v2-review-fixes): open phase 5`, the HEAD; 87 files (production 48,
tests 15, other 24, binary 0); lines production 15,218, tests 14,560, other 6,188; uncommitted
none. Nearly every production file of the crate is in the scope.

Step 2, the estimate at the skill's rates: a production reviewer $3.75, `review-tests` $7.10,
a verifier $2.33, the session $3.50 a pass and $1 more for the last; about $55 at `high` in
four passes, against about $51 measured for the first review on a code of the same size, and
about $50 at `medium`. The session had spent $0.63 by `usage.py`, nothing estimated. The user
approved `high` and the four passes ("je valide"), and asked that the agents start when the
next session resumes. No check ran: the scratch folder, the checks and the review's report,
steps 3 to 5, wait for the next session.

**Where the next session starts:**
1. Continue `reviewing-delivered-code` at step 3, tier `high`, the passes as approved (under
   Decisions): the scratch folder, the five checks, `cargo audit`, and `cargo mutants --list
   --in-diff` on `git diff` from `docs(v2-review-fixes): create the roadmap` of 2026-10-16 to `docs(v2-review-fixes): open phase 5`, run only at 300 mutants or fewer; then the
   report, `docs/reviews/delivered-code/<date>-<scope>.md` from `assets/report.md`, with
   Scope, Checks and Tier filled from this entry.
2. Start pass 1's reviewers at effort `high`: `review-security`, given `cargo audit`'s
   result, and `review-correctness`. Then their verification, priced by its findings.
- Before the wave, the skill asks a yes given in the session that starts it, with the cost
  spent so far by `usage.py` and the wave's estimate, about $7.50: the user's go-ahead of
  2026-10-17 settles the tier and the passes, and asked the agents started at resume, so one
  line and one yes.
- Nothing uncommitted.

Resumed in a third session. Read before working: this phase's file and report, and the
roadmap's README. Continued `reviewing-delivered-code` at step 3. `scope.py` gave the same
scope, its end now `docs(v2-review-fixes): record the closing review's scope and tier`, the commit that recorded the scope and tier, which touches this
report only; other lines 6,227. The diff covers 48 of the crate's 54 production files, and
`src/presentation/doctor.rs` is among the six it leaves out. The five checks were green:
`cargo test` passed 664 tests, and `up_a_proxied_request_and_down` ran. `tests/gateway_lifecycle.rs`
runs without libtest's harness, so its summary line reads 0 passed. `cargo audit` was green, with
1,338 advisories and 66 crates. `cargo mutants --list --in-diff` counted 551 mutants, over the
limit of 300, so the mutants were not run. Wrote the review's report,
`docs/reviews/delivered-code/2026-10-17-v2-review-fixes.md`, with Scope, Checks and Tier filled.
Phase 4's three leads are in the report's Scope, for the reviewers of their axes. Cost so far
by `usage.py`: $0.86, nothing estimated. Pass 1's reviewers wait for the user's yes.

The user said yes ("oui"), and pass 1's two reviewers ran at `high`:
- `review-security` gave 5 findings: 1 high, 1 medium and 3 low. The high one: a process that
  squats a backend's predictable loopback port, from 5800, receives the client's key and
  prompts, which the reviewer reproduced with llama-swap v263 alone in scratch.
- `review-correctness` gave 16 findings: 1 medium, 13 low and 2 without a severity. Its summary
  counted 17. Its F1 has the same cause as security's finding 4, flag names spelled with `_`, so
  the two were merged under number 4 and number 6 left unused.

That leaves 20 findings under the review's To Verify, numbers 1 to 5 and 7 to 21. No process
was left behind, and nothing changed in the repository besides the review's report and this
report. By `usage.py`, the session stood at $20.21, of which $9.07 was estimated:
`review-security` cost $7.65 and `review-correctness` $10.69, against $3.75 each at the skill's
rate.

Put to the user: two verifiers at `high`, about $2.50 at the skill's rate and more likely $5 to
$8 given pass 1's overrun. The user chose to run them in the next session: "on lance dans la
prochaine session". The review's report now has `Status` at pass 2 of 5, the verification of
pass 1's findings, with its Tier table renumbered. The scratch folder was
removed by the skill's step 10. The reproductions that the findings cite there are gone: the
squatter's scripts, `tests/review_scratch.rs` in the reviewer's clone, and the XDG states
`xdg1`, `xdg2`, `future-run` and `future-build`. Each finding's evidence says how it was built.
Committed: the review's report and this report.

**Where the next session starts:**
1. On lance dans la prochaine session: the verification of pass 1's 20 findings, pass 2 of the
   review. Continue `reviewing-delivered-code` at step 7:
   - create a new scratch folder (step 3);
   - take the checks from the review's report, after `git diff` from `docs(v2-review-fixes): record the closing review's scope and tier` on `src`, `tests`, `Cargo.toml`, `Cargo.lock` and `deploy` shows the code unchanged;
   - after the user's yes, with the cost spent shown, start two `review-verifier` at `high`.
     The first gets findings 1 to 5: security, with 4 merged with correctness's F1, both blocks
     given. The second gets findings 7 to 21: bugs, error handling, interruption and
     concurrency, and it gives findings 20 and 21 a severity.
2. Step 8: each verdict moves its finding out of To Verify, and each `LEADS` line goes under
   To Verify. Then `Status` goes to pass 3, `review-tests` and its verifier, given the
   reviewers' mutants since `cargo mutants` did not run. Pass 4 is `review-cleanup` and
   `review-structure`, with phase 4's leads from the review's Scope. Pass 5 is
   `review-performance`, then the report's finish.
- The user's yes is still needed before the verifiers start.
- Nothing uncommitted. The scratch folder is removed.

Resumed in a fourth session. Read before working: this phase's file and report, and the
review's report. `git diff` from `docs(v2-review-fixes): record the closing review's scope and tier` on `src`, `tests`, `Cargo.toml`, `Cargo.lock` and `deploy` was empty, so the
checks were taken from the review's report; `scope.py` now ends at `docs(v2-review-fixes): record pass 1 of the closing review`, which recorded
pass 1 and touches docs only. The review's Coverage table
still named the passes as they stood before the split into five; renumbered to the Tier
table's. Cost so far by `usage.py`: $0.54, nothing estimated. The user said yes ("oui"), and two
`review-verifier` started at `high`: one on findings 1 to 5, finding 4 with both its blocks,
and one on findings 7 to 21, which gives 20 and 21 a severity.

Both verifiers answered in their format, at `high`:
- findings 7 to 21: all `CONFIRMED`, all low, 20 and 21 included. The verifier corrected one
  point each of 16 (`bench compare <id> <id>` shows a single saved run), 17 (the sweep comes
  after the new compile, not before), 19 (the temporary file is removed on every path; only a
  kill leaves it) and 20 (no shared helper fits yet: add `files::replace_file`). Its one lead,
  `build list` printing "current build b11700 is not installed" above its own "not a build"
  line for the same directory, has finding 9's cause and fix, and was merged into it.
- findings 1 to 5: 2, 3, 4 and 5 `CONFIRMED`, 4 at medium with the methods of both axes, and
  1 `UNCERTAIN`, lowered from high to medium. The cause of finding 1 holds, but llama-swap's
  first health check comes 0.25 to 0.3 s after it starts the command, and llama-server without
  the GPU fails to bind in 0.09 to 0.12 s: the reviewer's 0.5 s backend overstated the race.
  What settles it, llama-server's time to fail to bind with the GPU visible, needs a CUDA
  context, which the review's rules forbid. The verifier also found that this machine has one
  login account, `seb`, which bears on every local-user finding. No lead.

The fixes the verifiers settle: finding 4 refuses `_` in `check_flag_name` rather than
normalising at four comparisons; finding 5 shares the escaping that `inventory_file.rs:94` and
`Text::checked` already hold; finding 3 drops the second `exited()` check, which would pass.

Step 8 moved the 20 findings under the review's Findings, finding 1 with the action "to
decide", and removed To Verify; the reviewers' FILES READ and LEFT OUT went under Coverage,
whose rows for pass 1's axes now name what rests on reading. `Status` is at pass 3,
`review-tests` and its verifier. The security verifier left three logs of its llama-server and
podman runs, `ll-b11538.log`, `ll-b11700.log` and `pod.log`, in the repository's root instead
of its scratch folder; they were moved to scratch before the commit. No process or port was
left (`pgrep`, `ss`). Cost by `usage.py`: verifiers $6.12 and $6.86, the session $15.35 in all,
$10.91 of it estimated. Committed: the review's report and this report. The scratch folder
was removed by the skill's step 10, with the verifiers' copies, tests and
scripts that the findings' Verification lines cite; each line says how its run was built.

**Where the next session starts:**
1. Pass 3 of the review: continue `reviewing-delivered-code` at step 6 with `review-tests` at
   `high`. Create a new scratch folder (step 3), take the checks from the review's report
   after `git diff` from `docs(v2-review-fixes): record the closing review's scope and tier` on `src`, `tests`, `Cargo.toml`, `Cargo.lock` and `deploy` shows the code
   unchanged, and give `review-tests` the scope and, since `cargo mutants` did not run, no
   missed mutants. Then its verifier, priced by its findings at step 7.
- Before the wave, the user's yes with the cost spent shown. Pass 1's reviewers cost two to
  three times the rate and pass 2's verifiers about five times: `review-tests`, $7.10 at the
  rate on production and test lines, may alone take $15 or more; say so with the estimate.
- Finding 1 stays "to decide" until the user's verdicts. The measurement that settles it,
  llama-server's time to fail to bind a taken port with the GPU visible, natively and as the
  image with its device, needs a CUDA context: only the user can allow it, outside the
  review's rules, or run it.
- Nothing uncommitted. The scratch folder is removed.

After that commit, the user asked to record two things: the review resumes at pass 3, and the
CUDA context that finding 1's measurement needs is allowed, against the review's rules (under
Decisions). The review's finding 1 says so. The next session therefore starts as above, with
one step added before pass 3's wave:

**Where the next session starts:**
1. Finding 1's measurement, run in the session itself, no agent, with the user's allowance:
   llama-server b11700, with the GPU visible, started natively on a loopback port that a
   listener in scratch already holds, timed from start to exit over several runs; then the
   official image the same way, with its GPU device (`podman run --rm --pull=never`, a
   container name without ai-ctl's prefix `ai-ctl-`). First check with `nvidia-smi` and
   `ai-ctl status` that no model is loaded and the gateway is down, and with `pgrep` after
   that nothing is left. Compare the times with llama-swap's first health check, 0.25 to 0.3 s
   after the start, and write the result and the verdict into finding 1's Verification.
2. Pass 3, as the block above says: `review-tests` at `high`, after the user's yes on its
   estimate, then its verifier.
- Nothing uncommitted.

The user then gave a first review of the review workflow's tools, asked for as many points
and proposals as possible, to be asked again once the roadmap closes. The session's account and
the user's answer to each point are recorded in `docs/notes/review-workflow-tooling.md`. The
direction agreed: what the agents produce becomes artifacts used again, not prose the next
session reads again ("un exemple réel vaut bien mieux que de la prose"). The changes go to
`reviewing-delivered-code` after this roadmap, per the phase's Out of Scope.

On the cost, the user asked for a written method, the same for every request, its variants
identified, so it can be improved, and fixed at the source rather than in a memory: "quand nous
avons un problème il faut le faire remonter et le traiter à la source". While gathering the
evidence for the agent setup's note, the session compared `usage.py`'s counts with the
`cost-state` records of the first review's five sessions. That showed this phase's cost
figures overstated two to three times; they are corrected under Problems And Deviations, and in
the review's Tier section. The note is
the agent setup's `notes/design-feedback/2026-10-17-cost-method-written-and-reproducible.md`,
with the data of the 21 runs of review agents. The calibrated recount was run in the session's
scratchpad, `usage.py` with `usage.toml`'s default set to 635, 845 and 872 tokens. The session
had given the user "two to three times" and "five times" in its first answer; it gave the
correction in the next.

For the next session, the block above stands, with one change to its gate: give the wave's
estimate in one line, without the method or a running total, as the user asked on
2026-10-17. `usage.py`'s figure for a review agent without a `cost-state` record overstates two
to three times.
- Nothing uncommitted after this entry's commit.

Session closed at the user's request. Since the entry above, committed as `docs: record the first feedback on the review workflow's tools`, two
things happened outside the repository. The agent's memory records the user's principle that a
tool's defect is fixed at its source, never by a memory instruction. The agent setup's cost note
is written in that repository's `notes/design-feedback/`, which git does not track; it waits for
the agent setup to process it. Nothing of the review moved: its `Status` is pass 3, finding 1
"to decide".

**Where the next session starts:**
1. Finding 1's measurement, with the user's allowance of the CUDA context, as the
   second-to-last resume point above sets it out. Write its result and verdict into the
   review's finding 1.
2. Pass 3 of the review: `reviewing-delivered-code` at step 6, `review-tests` at `high`, then
   its verifier.
- Before pass 3's wave: the user's yes, the wave's estimate given in one line, without the
  method or a running total.
- Nothing uncommitted in ai-ctl. The agent setup's note
  `notes/design-feedback/2026-10-17-cost-method-written-and-reproducible.md` waits there.

### 2026-10-18

Resumed in a fifth session. Read before working: this phase's file and report, and the
review's report. `git diff` from `docs(v2-review-fixes): record the closing review's scope and tier` on `src`, `tests`, `Cargo.toml`, `Cargo.lock` and `deploy` was empty. The
installed `ai-ctl` is not on `PATH`; `cargo build --release` built `target/release/ai-ctl`,
whose `status` gave the gateway stopped and 14 MiB in use. The scratch folder was
kept for pass 3.

Finding 1's measurement ran in the session, no agent, under the user's allowance of a CUDA
context. A script in scratch held a loopback port and timed llama-server b11700, GPU visible,
with the `qwen3.8-27b` command of the generated configuration: it exited at 0.40 to 0.87 s, five
runs, against 0.09 to 0.12 s without the GPU in pass 2. The first image run failed before
llama-server started: crun could not stat `/usr/lib64/libnvidia-egl-wayland.so.1.1.23`, which
the live CDI specification mounts. The cause is on the machine: the specification was written
at boot, 10:55, and dnf installed egl-wayland 1.1.24 at 11:25; `nvidia-cdi-refresh.path` does
not watch that library. `ai-ctl doctor` still says `ok CDI specification`, exit 0. A
specification generated in scratch with `nvidia-ctk cdi generate` differs by that line only;
with it, through `podman --cdi-spec-dir`, the image exited at 0.76 to 1.39 s. End to end,
llama-swap v263 alone with the real command as its model's and a squatter on the backend's
port: three rounds out of three for each runtime, the squatter received the health check at
0.25 s, then the bearer key and the prompt, and the client got its answer. Finding 1 is
`CONFIRMED`, action fix, severity back to high, the reviewer's: the verifier had lowered it for
the race's doubt alone. The fix's line on `docs/security.md` no longer names a race window: the
squatter has no race to win. Nothing was left: `pgrep`, `podman ps`, 15 MiB on the GPU.

The doctor gap went under the review's To Verify as lead 22, bugs, low, in the reviewers'
format, for pass 3's verifier. The live CDI specification on this machine stays stale until
the next boot or `sudo systemctl start nvidia-cdi-refresh.service`, the user's to run: until
then no image backend starts.

Committed as `docs(v2-review-fixes): settle finding 1 of the closing review by its measurement`. The user said yes to pass 3's reviewer ("oui"), and `review-tests`
started at `high`. Its prompt carried the scope as an unexpanded `$(cat …)` instead of the
scope's output; a follow-up message pointed it at `scope.txt` in scratch, which it read. It
answered in its format: 26 findings, all on the tests axis, numbered 23 to 48 after lead 22.
It wrote 28 mutants by hand, since `cargo mutants` did not run, and each one survived the whole
suite, 663 passed; its proposed tests, one per finding, pass on the code and each kills its
mutants. None shares a cause with another finding, so nothing was merged; 24, 34 and 39 sit
next to findings 18, 17 and 21, whose fixes they will go with. Its patch of proposed tests,
842 lines, stays in scratch, `review-tests/mutants/review-tests.patch`. Nothing changed in the
repository and no process was left. The 26 findings and lead 22 wait under To Verify for one
`review-verifier`, after the user's yes.

Committed as `docs(v2-review-fixes): record pass 3's tests review of the closing review`. Two questions were put to the user: the go-ahead for one
`review-verifier` at `high` on findings 23 to 48 and lead 22, about $3.50 at step 7's price
(27 findings at about $0.126; pass 3 then stays under $15 at the rates, so the verification
belongs to this pass); and whether to copy `review-tests`' patch out of scratch, for example to
`~/.local/share/ai-ctl/review-scratch/`, before step 10 removes the scratch folder, since it
holds a tested test for each finding the fixes will need. The user closed the session and
asked to be put the questions again when the next one opens: "tu me repose la question a
l'ouverture et on y réponds". The scratch folder was therefore not removed: step 10 would have
decided the second question. It is 1.4 GB, on a tmpfs, so a reboot
loses it: `review-tests/mutants/` holds the patch, its add and apply scripts, `run.sh`,
`kill.sh` and the 56 logs of the mutants' survival and kill; `review-tests/copy/` and
`review-tests/target/` are the reviewer's clone and build; `finding-1/` holds the
measurement's scripts, logs and the CDI specification generated in scratch.

**Where the next session starts:**
1. "tu me repose la question a l'ouverture et on y réponds": put the two questions to the user
   again, before anything else, and act on the answers.
   - The verifier: one `review-verifier` at `high` on findings 23 to 48, the tests axis, and
     lead 22, bugs, about $3.50 at step 7's price, given in one line without the method or a
     running total. Continue `reviewing-delivered-code` at step 7: take the checks from the
     review's report after `git diff` from `docs(v2-review-fixes): record the closing review's scope and tier` on `src`, `tests`, `Cargo.toml`, `Cargo.lock` and `deploy` shows
     the code unchanged; give the verifier the scope (`scope.py`'s output, pasted in the
     prompt, not as a shell substitution) and its folder `<scratch>/verifier-tests`, and point
     it at `review-tests/mutants/` when the scratch folder still exists.
   - The patch: copy `review-tests/mutants/` out of scratch, or not. If
     the machine rebooted, the folder is gone and the question is moot: say so, create a new
     scratch folder (step 3), and the verifier rebuilds what it needs.
2. Step 8 on the verdicts: each `CONFIRMED` under Findings, `UNCERTAIN` with "to decide", the
   mutants per finding under Checks; then step 10 removes the scratch folder, and `Status`
   goes to pass 4, `review-cleanup` and `review-structure` at `high`, with phase 4's leads from
   the review's Scope. Pass 5 is `review-performance`, then the report's finish.
- The live CDI specification `/var/run/cdi/nvidia.yaml` is stale (lead 22): no image backend
  starts until `sudo systemctl start nvidia-cdi-refresh.service`, the user's to run, or the
  next boot.
- Nothing uncommitted in ai-ctl. The scratch folder is kept on purpose,
  outside the repository.

Resumed in a sixth session. Read before working: this phase's file and report, and the
review's report up to its Findings. The machine had not rebooted since 10:55, so the scratch
folder was intact, and the live CDI specification still stale.
`git diff` from `docs(v2-review-fixes): record the closing review's scope and tier` on `src`, `tests`, `Cargo.toml`, `Cargo.lock` and `deploy` was empty. The two questions
were put again, as asked, and the user answered both with yes:
- the patch: `review-tests/mutants/`, 62 files and 2.1 MB with `review-tests.patch`, copied to
  `~/.local/share/ai-ctl/review-scratch/2026-10-18-v2-review-fixes-review-tests/`, outside
  the repository; ai-ctl reads only its named folders under its data directory, so it ignores
  this one;
- the verifier: one `review-verifier` at `high` on lead 22 and findings 23 to 48, about $3.50,
  given in one line. Its folder is `<scratch>/verifier-tests`, and it reuses the reviewer's
  clone and build in `review-tests/`. `scope.py` now ends at `docs(v2-review-fixes): record the session close of the closing review's pass 3`, with no code changed.
  The allowance of a CUDA context was not passed on to it: it checks lead 22 by the live
  specification's `hostPath` entries and `ai-ctl doctor`, without podman and the GPU device.

The verifier answered in its format, at `high`: the 27 findings `CONFIRMED`, lead 22 at low,
kept. It ran M1 to M28 again from its own definitions, in two clones of `docs(v2-review-fixes): record the session close of the closing review's pass 3`: each
survived the whole suite without the proposed tests, and failed its own test, and only it,
with them; with the patch, `cargo test` passed 689, fmt and clippy green. The reviewer's 663
left out `gateway_lifecycle`: the suite is 664. What it adds to the fixes, written into each
finding: every proposed test drops its `review_` prefix and takes the module's blank line
before `#[test]`; 28's test is written on `FakeApi` rather than a `SlowApi` of its own; 33's
test is not fit as is, since the handler's output escapes libtest's capture, and its fix also
updates `docs/architecture.md:115` and the comment at `fakes.rs:1`; 25 and 26 share one failing
writer; 27 asserts at least 35 s; 22's fix tests its parser on a captured specification and
corrects `CDI_REMEDY`'s comment. 24, 34 and 39 are causes apart from 18, 17 and 21, each the
error path of its neighbour's window. One lead: M29, `status`'s lock, which survives and no
proposed test kills; it went under To Verify as finding 49, tests, for pass 4's verifiers, not
a verifier of its own.

Step 8 moved findings 22 to 48 under the review's Findings, wrote the 29 mutants with their
findings under Checks, the verifier's FILES READ and LEFT OUT under Coverage, and pass 3's
cost under Tier: counted at 845 output tokens a call, `review-tests` about $7.67 and the
verifier about $2.19, against $7.10 and $3.50; `usage.py` as shipped gives the verifier $6.83.
`Status` is at pass 4. Before step 10 removed the scratch folder, the verifier's
`tools/mutants.py`, its two scripts and its two summaries were copied beside the patch, into
`verifier/` of the folder above: the fixes need each mutant to prove that its test kills it.
`pgrep` showed nothing left, and the scratch folder is removed.

**Where the next session starts:**
1. Pass 4 of the review: continue `reviewing-delivered-code` at step 6 with `review-cleanup`
   and `review-structure` at `high`. Create a new scratch folder (step 3), take the checks from
   the review's report after `git diff` from `docs(v2-review-fixes): record the closing review's scope and tier` on `src`, `tests`, `Cargo.toml`, `Cargo.lock` and `deploy`
   shows the code unchanged, and give each reviewer `scope.py`'s output pasted in its prompt
   and phase 4's leads from the review's Scope: `GatewayProcess`'s ten methods, of which
   `Bench` calls three (both); batches 43 to 48's error and status texts (`review-structure`,
   the CLI's contract).
2. Then the verification, priced by its findings at step 7, with finding 49 given to the
   verifier of the tests axis's nearest reviewer, or to either one.
- Before the wave, the user's yes, the wave's estimate given in one line, without the method or
  a running total: two reviewers at $3.75 each at the skill's rate.
- The proposed tests and the mutants' definitions for findings 23 to 49 are in
  `~/.local/share/ai-ctl/review-scratch/2026-10-18-v2-review-fixes-review-tests/`, outside the
  repository: the fixes start from them.
- The live CDI specification `/var/run/cdi/nvidia.yaml` is still stale (finding 22): no image
  backend starts until `sudo systemctl start nvidia-cdi-refresh.service` or the next boot.

Resumed in a seventh session. Read before working: this phase's file and report, and the
review's report: its Status, Scope, Checks, Tier, To Verify and Coverage. `git diff` from `docs(v2-review-fixes): record the closing review's scope and tier` on `src`, `tests`, `Cargo.toml`, `Cargo.lock` and `deploy` was empty, so the checks were taken from the review's
report; `scope.py` now ends at `docs(v2-review-fixes): record pass 3's verification of the closing review`, with other files 26 and other lines 8,190, the
review's and this report's growth. The machine had not rebooted, and the live CDI
specification still named `libnvidia-egl-wayland.so.1.1.23`. The user then ran
`sudo systemctl start nvidia-cdi-refresh.service`: the specification, rewritten at 15:01, names
1.1.24 only, so image backends can start again; finding 22's gap in doctor stands. Pass 4's
wave was put in one line, about $8 to $12 for the two reviewers. The user said yes, with one
condition: the reviewers run one after the other, with the user's go-ahead between them, not in
parallel (under Decisions). `review-cleanup` started at
`high`, given the scope pasted in its prompt, phase 4's lead on `GatewayProcess`, and the order
to write and run its commands from its scratch folder only.

`review-cleanup` answered in its format, 122 tool calls in about 16 minutes: 11 findings,
numbered 50 to 60, 2 on dead and legacy code and 9 on refactoring, none with a severity, which
the axis's format does not ask. No dead production item: it built a copy of the crate as a
binary, every module private, and rustc warned of nothing. On phase 4's lead it found no
finding of its axes: splitting `GatewayProcess` into a read part removes no code, and what the
split gains, the narrower dependency of `Bench`, prune and Doctor, it left to
`review-structure`. The findings with most weight: 51, `Doctor::pause` and `now` that nothing
calls, which it proved with a clone where both panic and doctor still passes; 55, `Bench::setup`
loading the inventory and resolving the build again after `Configurator::prepare`, which also
lets a run record another build than the one it compared; 50, the run store's read path for
formats 1 and 2 with no `// LEGACY:` comment, though the seven runs saved on this machine are
all of those formats. 50 touches finding 10, and 55 the performance lead on the bench's two GPU
reads: the merges wait for step 7, once `review-structure` has answered. It wrote only in its
scratch folder, ran only `doctor` of ai-ctl's commands, and left no process (`pgrep`). Its
findings went under the review's To Verify, its FILES READ and LEFT OUT under Coverage.
Committed as `docs(v2-review-fixes): record pass 4's cleanup review of the closing review`.

The user gave the second yes ("oui feu vert"), and `review-structure` started at `high`, given
the scope, phase 4's two leads for its axes, `review-cleanup`'s words on splitting
`GatewayProcess`, and findings 50, 51 and 55 to settle against.

`review-structure` answered in its format, 113 tool calls in about 16 minutes: 19 findings,
numbered 61 to 79, none high. On the two leads: splitting `GatewayProcess` breaks no rule while
`Bench` stands alone, and is worth it only with finding 51's fix, since Doctor, which only
reads, holds `start`, `signal` and `forget` and builds a whole `Lifecycle` in the application
layer; `docs/commands.md` quotes none of batches 43 to 48's texts, and two of them are pinned by
no test: up's "not looking for processes left on the GPU", which can never be printed (69), and
bench's "cannot record the run's GPU". The rest: the CLI's contract against `docs/commands.md`
and the README, with `build list`'s dangling `current` link (62, 63), prune's lock under
`up`'s busy message (64), the GPU that `gateway config` needs (65), and three promises of the
documents the code does not keep (66 to 68); conventions that batch 50 recorded and the code
does not follow (72 to 75), a test on a hand-written error body (71), and three stale texts
(76 to 78); and the inventory's reader, a third reader with finding 10's cause (79). It ran
`build list`, `doctor`, `status` and llama-swap alone in scratch, and left no process.

Step 7 merged three findings by cause: 61 into 51, the same Doctor built on a whole `Lifecycle`,
with the port's split as the fix's second part; 70 into 55, `Prepared` keeping none of what
`prepare` read, the GPU reading included; 79 into finding 10, already verified, whose verifier
checks only the inventory. Kept apart: 64 from 13, since 13's fix removes `status` from the lock
and leaves the wording that 64 is about; 62 and 63 from 9 and from each other, three causes in
`build list`'s lines on the `current` link; 50 from 10, a missing comment beside a parse order.
29 findings wait under To Verify: 49, 50 to 60, 62 to 69 and 71 to 79. Priced at step 7's
$0.126 a finding, about $3.66; with the two reviewers and the session at step 2's rates, the
pass stays under $15, so the verification belongs to this pass.

Committed as `docs(v2-review-fixes): record pass 4's structure review of the closing review`. Put to the user: two `review-verifier` at `high`, one after the other
like the reviewers, the first on 49 to 60, the second on 62 to 79, about $4 to $6 in one line.
The user closed the session without answering. Observed in the pass: `review-cleanup`'s check
for dead items, the crate built as a binary with private modules, finds unused items only, not
an unreachable branch such as finding 69's `warn_unread`, which `review-structure` found by
reading and `review-tests` had seen in pass 3 and left out as not a test gap. The scratch folder, 2.7 GB, held the two reviewers' clones, builds and scratch states: the
skill's step 10 removed it, and each finding's evidence says how its run was built.

**Where the next session starts:**
1. Put the verification question to the user again, in one line: two `review-verifier` at
   `high`, one after the other unless the user says otherwise, about $4 to $6. On the yes,
   continue `reviewing-delivered-code` at step 7: create a new scratch folder (step 3), take
   the checks from the review's report after `git diff` from `docs(v2-review-fixes): record the closing review's scope and tier` on `src`, `tests`, `Cargo.toml`, `Cargo.lock` and `deploy` shows the code unchanged, and give each verifier `scope.py`'s output
   pasted in its prompt, its folder `<scratch>/verifier-<batch>`, and the order to write and
   run its commands from that folder only.
   - The first verifier: findings 49 to 60, 51 with `review-structure`'s merged block (the
     port's split) and 55 with its merged block (bench's second GPU read).
   - The second: findings 62 to 69 and 71 to 79; for 79, only what it adds to finding 10,
     the inventory's reader.
2. Step 8 on the verdicts: each `CONFIRMED` under Findings, `UNCERTAIN` with "to decide";
   61 and 70 stay unused, and 79, if confirmed, goes into finding 10 and stays unused; each
   `LEADS` line under To Verify; pass 4's rows under Coverage (architecture, dead and legacy
   code, refactoring, conventions, on-disk formats, the CLI's contract). Then step 10, and
   `Status` to pass 5: `review-performance` at `high`, told that phase 4's lead on the bench's
   two GPU reads is now part of finding 55, then its verifier, then step 9, the report's
   finish and its fix plan.
- Open with the user: the go-ahead for the two verifiers, and whether they run one after the
  other like the reviewers.
- Nothing uncommitted after this entry's commit. The scratch folder is removed.

Resumed in an eighth session. Read before working: this phase's file and report, and the
review's Status, Scope, Checks, Tier, To Verify and Coverage. `git diff` from `docs(v2-review-fixes): record the closing review's scope and tier` on `src`, `tests`, `Cargo.toml`, `Cargo.lock` and `deploy` was empty, so the checks were taken from the review's report;
`scope.py` now ends at `docs(v2-review-fixes): record the session close of the closing review's pass 4`, with other files 26 and other lines 8,801. The verification
was put in one line, two `review-verifier` at `high`, one after the other, about $4 to $6, and
the user said yes ("oui"). Each verifier's findings were
given as a file in its folder, `findings.md`, the blocks of To Verify copied as they stand, and
`scope.py`'s output pasted in its prompt with the per-file list in `scope.txt`.

The first verifier, on findings 49 to 60, answered in its format, 81 tool calls in about 11
minutes: the 12 findings `CONFIRMED`, and no lead. Finding 51's merged block, architecture, is
refuted as a defect of its own: `docs/architecture.md:53-54` speaks of whole ports, the port's
shape is the first review's approved fix of its finding 25, and Doctor's read-only contract
holds in behaviour; the `GatewayState` supertrait stays an optional rider of 51's fix, and 51
keeps its number with the dead-code axis. Finding 55's merge holds, and its fix changes: Bench
reads the GPU once and passes the reading to `prepare_on`, as Doctor does, and `Prepared` drops
`Eq` when it carries the `Inventory`. Finding 60's fix, as the reviewer wrote it, does not
compile; the verifier corrected it. Smaller corrections: 50's policy share prints "–", not
"0 %"; 51's dead lines are six, not eight; 57 needs no renamed import; 52's fix narrows
`api_key.rs`'s imports. The verifier ran M29 again and wrote finding 49's test, which kills it,
and wrote M30, the text of bench's "cannot record the run's GPU", which survives. It applied
the fixes of 52, 56, 57 and 60 in clones, each green. It wrote only in its folder, ran `doctor`
on a stubbed build and `bench list` on a scratch copy of the runs, and left no process.

Step 8 moved findings 49 to 60 under the review's Findings, wrote M30 under Checks, the
verifier's cost under Tier, about $3.12 at 845 output tokens a call against $1.52 at step 7's
price, the rows of dead and legacy code and refactoring under Coverage, and its FILES READ and
LEFT OUT. The second verifier, on findings 62 to 69 and 71 to 79, waits for the user's yes, as
pass 4's Decision on the reviewers asks between agents.

Committed as `docs(v2-review-fixes): record pass 4's first verification of the closing review`. The user said yes ("oui"), and the second verifier started at `high`.
It answered in its format, 70 tool calls in about 9 minutes: the 17 findings `CONFIRMED`, and
four leads. The splits hold: 64 is what finding 13's fix leaves, prune still a holder that
neither starts nor stops; 62 and 63 are causes apart from finding 9 and from each other, though
63's better fix, a warning sent from `application::build::list`, also fixes 62, so the three
are planned together. Finding 79 holds, and finding 10's fix as written, JSON and `format` for
the two stores, did not cover the inventory, TOML and `version`: 79 went into finding 10, whose
fix now reads the version key first in each of the three readers. Corrections: 71's title
holds, but the parser is tested on real output elsewhere (`server_api.rs:419-430`,
`chat_api.rs:429-437`), so its fix only swaps the hand-written bodies for the fixtures; 72 is
three binary tests, not two, `gateway config`'s included; 73's `bad_link` is state, which
`current_fails` would hide; 68's "gigabytes" is not measured. The verifier wrote M31, a proof
that up's `warn_unread` branch cannot be reached. It ran `build list`, `doctor` and `status` on
scratch states, llama-swap alone on a loopback port, and left no process.

Step 8 moved findings 62 to 69 and 71 to 78 under the review's Findings, and merged 79 into
finding 10. Of the four leads, the one on `docs/commands.md`'s `bench run` section names the
cause of finding 65 and went into it; the three others are findings 80 to 82 under To Verify,
two on error handling, doctor's remedy for an inventory of a newer version (80) and the GPU
read before the inventory is checked (82), one on tests, a fixture captured from another
request than the test makes (81): they go to pass 5's verifier. M31 went under Checks, the
second verifier's cost under Tier, about $2.57 against $2.14 at step 7's price, pass 4's rows
of architecture, conventions, on-disk formats and the CLI's contract under Coverage, and its
FILES READ and LEFT OUT. `Status` is at pass 5.

Before step 10, the verifiers' work that the fixes will use was copied out of scratch, as pass
3's was at the user's word: the diffs of their clones (finding 49's test; M29, M30 and M31; the
fixes they applied for findings 52, 56, 57 and 60), the first verifier's logs and tools, into
`~/.local/share/ai-ctl/review-scratch/2026-10-18-v2-review-fixes-pass-4/`, 248 KB. `pgrep`
showed nothing left, and the scratch folder, 8.2 GB, is removed.

**Where the next session starts:**
1. Pass 5 of the review: continue `reviewing-delivered-code` at step 6 with
   `review-performance` at `high`. Create a new scratch folder (step 3), take the checks from
   the review's report after `git diff` from `docs(v2-review-fixes): record the closing review's scope and tier` on `src`, `tests`, `Cargo.toml`, `Cargo.lock` and `deploy` shows
   the code unchanged, and give the reviewer `scope.py`'s output pasted in its prompt, its
   folder, the order to write and run its commands from that folder only, and phase 4's lead
   on the bench's two GPU reads as part of finding 55, already verified.
2. Then one `review-verifier` at `high` on `review-performance`'s findings and on findings 80
   to 82, priced at step 7; then step 8, and step 9: the report's finish, ordering the findings
   by action, Coverage's last row and the files no agent opened, and the fix plan, every
   finding in a numbered batch. Then the user's verdicts, batch by batch.
- Before each wave, the user's yes, the wave's estimate given in one line, without the method
  or a running total; one agent after the other, with the user's go-ahead between them.
- Nothing uncommitted after this entry's commit. The scratch folder is removed.

Resumed in a ninth session. Read before working: this phase's file and report, and the
review's Status, Scope, Checks, Tier and To Verify. `git diff` from `docs(v2-review-fixes): record the closing review's scope and tier` on `src`, `tests`, `Cargo.toml`, `Cargo.lock` and `deploy` was empty, so the checks were taken from the review's report; `scope.py` now
ends at `docs(v2-review-fixes): record pass 4's second verification of the closing review`, with other files 26 and other lines 9,227. Pass 5's reviewer was put in one
line, `review-performance` at `high`, about $4 to $6, and the user said yes ("oui"). `review-performance` started at `high`, given `scope.py`'s output
pasted in its prompt, its folder `<scratch>/review-performance`, the order to run every command
from it with any build and XDG state inside it, and finding 55 as the place of phase 4's lead on
the bench's two GPU reads: a measurement of that read's cost goes into 55.

`review-performance` answered in its format, 69 tool calls in about 14 minutes: six findings,
numbered 83 to 88, all on performance, none with a severity, which the axis's format does not
ask. The two with most weight are on `doctor`, 263 to 267 ms on this machine: 83, the toolkit
probe's `rpm -qa 'cuda-toolkit*'`, which loads every header of the database, 87 to 108 ms
against 7 ms for a query by exact names, about a third of the command; 84, its fifteen
independent probes run one after the other, 230 to 258 ms against about 100 ms run at once. The
others: 85, the bench's session scenario counting the agent's bare prompt a second time, two
requests; 87, `down` sleeping a full 100 ms poll for a llama-swap gone 1 to 2 ms after SIGTERM;
88, `build --ref <installed tag>` refused only after a fetch and a checkout, 1.46 s measured by
phase 3. 86 measures finding 55's second pass at about 0.12 ms a run, NVML's 24 to 26 ms
initialisation paid by the first read only: 55 has no performance reason, its reasons stay the
race and the error paths. Measured with no finding: the monitor's 250 ms sampling at 0.023 ms of
CPU a reading, `bench list`, `status`, `build list`, `up`'s health poll. It wrote only in its
scratch folder, ran `doctor`, `status` and `build list` of ai-ctl's commands on a scratch XDG
copy, llama-swap alone on a loopback port, and left no process (`pgrep`, `nvidia-smi`).
Counted at 845 output tokens a call, it cost about $4.45, against $3.75 at the skill's rate.

Step 7: 86 merged into finding 55, its number unused, its verifier to check only the
measurement. No other finding of the pass shares a cause with one already in the report: 87's
fix, a deadline on the injected clock in place of `STOP_POLLS` and `KILL_POLLS`, sits beside
finding 27, whose test asserts the fake clock moved at least 35 s and holds under that fix, so
the two are planned together; 88 and finding 67 both touch `Builder::build`, apart. The
findings, the Tier line and its FILES READ and LEFT OUT went into the review's report, and
`Status` names the reviewer as answered. Nine findings wait for one `review-verifier`: 80 to 85,
87, 88 and 86's measurement, about $1.10 at step 7's price; the pass stays under $15 at step
2's rates, so the verification belongs to this pass.

Committed as `docs(v2-review-fixes): record pass 5's performance review of the closing review`. The verifier was put in one line, about $2 to $3, and the user said yes
("ok"). One `review-verifier` started at `high`, its folder `<scratch>/verifier-performance`,
the nine blocks given as `findings.md` there with finding 55's entry for 86, the scope pasted
in its prompt, the order to build in a copy of its own rather than the reviewer's clone, and
the fixes of 80, 81 and 82 to set; it was told where 87, 82 and 84 touch findings 27, 55 and
51.

The verifier answered in its format, 64 tool calls in about 12 minutes: the nine `CONFIRMED`,
and one lead. It set the three fixes: 80, a typed `UnsupportedVersion` that doctor's Inventory
check downcasts for a remedy of its own, with or after finding 10; 81, a capture of `GET
/upstream/nope/props`, whose 404 "model not found" llama-swap v263 gives for every
`/upstream` request of an unknown model, planned with finding 71; 82, `prepare_on` split into
`resolve()` and `size()` so that `prepare` reads the GPU after the inventory, planned with
finding 55, since `resolve()` is where 55's `Prepared` takes its inventory and builds. Its
corrections: 84's GPU reading is not independent, and the fix's four groups give 154 to 156 ms,
not about 100, unless 83 lands first, 72 to 73 ms then, so 51 and 83 come before 84; 86's second
pass is 0.09 ms rather than 0.12; 88's 1.20 s fetch was measured apart from the 1.46 s refusal,
and the pre-check is limited to a release tag. 87's fix holds against finding 27's test, which
lands first. The lead: `nvml.rs:33` says NVML's initialisation costs 10 to 20 ms; both agents
measured 24 to 26 ms.

Step 8 moved findings 80 to 85, 87 and 88 under the review's Findings, folded 86's measurement
into finding 55, its number unused, and wrote the lead under To Verify as finding 89,
conventions. The verifier's cost went under Tier, about $2.54 at 845 output tokens a call
against $1.13 at step 7's price, its FILES READ and LEFT OUT and the pass's rows under
Coverage. Before step 10, the verifier's diffs of its clones (81's test and capture, 82's probe
and split, 85's fix and probe, 88's probe), its timing scripts and harness, and the reviewer's
scripts, wrappers and harnesses were copied to
`~/.local/share/ai-ctl/review-scratch/2026-10-18-v2-review-fixes-pass-5/`, 132 KB, as passes
3 and 4's were. `Status` names finding 89 as the last step before the report's finish.

Committed as `docs(v2-review-fixes): record pass 5's verification of the closing review`. The user said yes ("oui") to one `review-verifier` at `high` on finding
89, about $1. It answered in its format, 14 tool calls in about 2 minutes: `CONFIRMED`, and the
figure is wrong whichever quantity it names: `Nvml::init` alone costs 7 to 9 ms, the first
read, which also attaches the device, 23 to 28 ms, a later read under 0.1 ms; the figure, from
`perf(gpu): initialise NVML once per process` of 2026-10-16, matched neither, and the first read drifted from 15 to 20 ms to 23 to 28 ms in two
days on the same driver and persistence mode. Its fix: an order of magnitude in the comment, the
measured values in the fix batch's report. About $0.43 at 845 output tokens a call. Its one
lead, left while verifying a lead, stays under Coverage unverified: `docs/toolchain.md`'s
record lacks two rebuilds of `kmod-nvidia` for the 7.2.11 kernels of 2026-10-17, the user's to
complete.

Step 9 finished the review's report. The 84 findings are ordered by action, all fix: security
and bugs by severity (1 high; 4 and 2 medium; 3, 5, 7, 8, 9, 11 and 22 low), then error
handling, interruption and concurrency, performance, the CLI's contract, conventions, dead and
legacy code, refactoring and tests, each keeping its number. Coverage has its twelve rows, and
names the files no agent opened: nine captures and two snapshots of `tests/`, the `Makefile`,
and `docs/notes/review-workflow-tooling.md`. The Fix Plan puts every finding in one of 59
batches, in the first review's order, three of them placed early because a later batch needs
them (15 for 46, 28 for 43, 39 for 40), and six holding a choice for the user (1, 2, 22, 24,
26, 48); a script checked that each finding sits in exactly one batch. `Status` is awaiting the
user's verdicts. Step 10 removed the scratch folder, 2.6 GB; `pgrep`
showed nothing left. This phase's first task is ticked, and the README's progress recomputed.

Committed as `docs(v2-review-fixes): finish the closing review's report and its fix plan`. The session put the Security section to the user, batch by batch. The
user answered that the findings had been confirmed before the roadmap's creation: those were
the first review's, `2026-10-15-v2-rebuild.md`; the 84 of the closing review are new and had no
verdict. The session also found the review's `Status` line carrying the end of its previous
version, and fixed it (`docs(v2-review-fixes): drop the stale end of the closing review's status line`). The user then delegated the verdicts: the session settles
the clear cases and asks only the hardest ("Serait t'il possible de te laisser choisir les
confirmation et que tu ne me demande que pour les cas les plus durs à trancer ? On aurait du
faire ça dès le début"). The session confirmed 83, rejected 84, settled four of the plan's six
choices, and put three questions: finding 1's transport, finding 18's `libc`, and the scope.
The user answered: a probe for 1 before deciding, 18 kept and documented, the whole plan here.

The user then asked whether fixing as the roadmap went would have been simpler: a first full
review, a second one, and a third pass to fix. The session's account: the second review had two
reasons, the fixes' check and a second use of the skill, the second being the skill's need; it
found a dozen debts the roadmap itself added (69, 55, 71 to 75, 77, 78), which a review of each
phase's diff at its close would have caught fresh, 28 test gaps that `cargo mutants` on each
batch's smaller diff would have shown at the commit, and a few that only a new reading finds (1, 4,
83); and a full review that reports everything with no floor makes each fix roadmap call for a
review that calls for a roadmap. The user agreed with the proposals, kept a security pass as
essential, and chose to fix everything now (under Decisions). The proposals went to
`docs/notes/review-workflow-tooling.md`, for the skill's change after this roadmap.

The review's report records each finding's verdict, the settled choices in its batches and the
approval; its `Status` is complete. This phase's second task is ticked; the generic fixing task
became the 58 batches kept, 41 rejected, and a security pass closes the phase: 61 tasks, the
README's totals recomputed. The constraints add `cargo mutants --in-diff` on each batch's diff.

Committed as `docs(v2-review-fixes): record the closing review's verdicts and the fix scope`. Batch 1 started with its probe, in the session's scratchpad: llama-swap
v263 alone on a free loopback port, its one model a Python HTTP server on a unix socket, for
three proxy forms, `unix://<path>`, `http://unix:<path>` and `unix:<path>`; `-validate`
accepts all three, since it only parses the URL. A first run failed for another cause: the
scratchpad's socket path, about 130 characters, is over the 108 bytes of `sun_path`, and the
backend exited; under `$XDG_RUNTIME_DIR` it ran. Then, in the three forms, llama-swap's health
check never reached the backend and timed out, HTTP 500 to the client, nothing received; with
`logLevel: debug`, `unix://` logs `health check not ready: unsupported protocol scheme "unix"`
30 times. The control, the same backend on TCP at `http://127.0.0.1:${PORT}`, passed its health
check on 5800 and received the client's `Authorization: Bearer` key and the prompt, as finding
1 states. Nothing was left (`ps`), and each socket directory was removed. llama-swap v263
cannot reach a backend over a unix socket, so the transport stays TCP unless llama-swap gains
it; the choice goes back to the user, as the Decision on finding 1 asks.

The user chose the documents alone. Batch 1, finding 1: `docs/security.md` §2 describes a
backend's port taken first, the measured times, the key and prompts handed over and why ai-ctl
cannot close it, §1's table names the exception, and its Sources the closing review;
`docs/stack.md` §6.4 records that llama-swap takes any 200 on `/health` and reaches a backend
over TCP only, with the probe; `docs/notes/backends-on-unix-sockets.md` keeps the switch for the
day llama-swap allows it, with what a roadmap must settle. `common/arg.cpp:3335` at `b11700`,
`--host` taking a `.sock` path, was checked in the sources clone. No code, so no test and no
mutant; the five checks green, `up_a_proxied_request_and_down ... ok`.

Committed as `docs(security): record the squat of a backend's port that llama-swap cannot prevent`. Batch 2, finding 2: `up`'s refusal of a taken address adds that an
unknown program may have taken the API key from a client while the gateway was down, and to
change the key as `docs/security.md` §3 says; the application layer does not know the key's
path, so the error names the document rather than the file. Test-first:
`up_refuses_a_taken_address` expected the new text and failed on its assertion, then passed.
`docs/security.md` §3 names the squat of 8080 under What It Does Not Guard, why the key is not
changed at each `up`, and the rotation with the gateway down; §1's table names it; the `up`
section of `docs/commands.md` says what the error adds. `cargo mutants --in-diff` on the batch's
diff: one mutant, `up` replaced by `Ok(Default::default())`, unviable since `Up` has no
`Default`. The five checks green, 663 in libtest and `up_a_proxied_request_and_down ... ok`.

Committed as `fix(up): point to the key's rotation when the gateway's address is taken`. Batch 3, finding 3: `PortCheck` gains `listens(pid, listen)`, whether
a process holds the socket listening on an address. `TcpPorts` reads `/proc/net/tcp` or `tcp6`,
keeps the rows in state `0A` whose local address, as the kernel prints it, is `listen`, and
looks for their inodes among the links of `/proc/<pid>/fd`; a process that is gone holds
nothing. `up`'s health wait counts a 200 only when the started llama-swap holds the socket: an
answer from another program is not the gateway's, and when llama-swap then exits, failing to
bind, the error says another program answers there, with the advice of batch 2, now one
constant for both messages; a check that cannot be made stops llama-swap with "cannot tell
whether the answer … comes from the llama-swap ai-ctl started". The parser is tested on two
captures of 2026-10-18, a Python listener on `127.0.0.1` and `[::1]` with one connection
accepted, `tests/fixtures/proc-net-tcp-excerpt.txt` and `proc-net-tcp6-excerpt.txt`, listed in
`tests/fixtures/README.md`. Test-first: the four tests failed on the stubs, the two of `up` on
`unwrap_err` since `up` succeeded. `cargo mutants --in-diff` missed five at first: three in
`read_dir`'s guard, killed by a test of a gone PID and of `/proc/1/fd`, root's; two in
`read_link`'s guard, a descriptor closed since the listing, which only other tests' threads
provoked, by chance, so that a third mutant was caught in one run and missed in the next. The
links' reading moved into `holds_socket`, tested with a built `NotFound`: 24 mutants caught, 1
unviable. `docs/commands.md` and `docs/security.md` §3 say what `up` counts. The five checks
green, 669 in libtest and `up_a_proxied_request_and_down ... ok`, which runs `listens` on a real
llama-swap.

The session's line on the files no agent opened, in the review's Coverage, said that
`test_fixtures.py` checks the captures against their tools: it tests the skill's own eval
fixtures. Corrected there; pass 3's reviewer had written the same, which stays as it wrote it.

Committed as `fix(up): take an answer only from the socket of the started llama-swap`. Batch 4, finding 4: `check_flag_name` refuses a flag with `_`, with
`InventoryError::UnderscoredFlag`, whose message gives the dashed spelling, "must be written
`ctx-size`: llama-server reads each `_` of a flag as `-`". The one check covers the defaults,
the flags and the file keys, so `api_key`, `api_key_file`, `ctx_size`, `model_draft` beside a
`model-draft` file and a default `batch_size` are all refused before they reach the command.
Test-first: the inventory test failed on `expect_err`, the inventory being accepted, then
passed. `ai-ctl doctor` on the installed inventory still gives `ok Inventory`, its
`ctx_size` and `vram_mib` being the model's fields, not flags. The shipped inventory's header
and the README's inventory table say the spelling. `cargo mutants --in-diff`: one mutant,
`check_flag_name` emptied, caught. The five checks green, 671 in libtest.

Committed as `fix(inventory): refuse flag names spelled with an underscore`. Batch 5, finding 5: a new domain module, `domain::text`, holds the one
rule, a control character or U+2028 or U+2029, with `escape_controls`, which writes each as its
`\uXXXX`. NVML's process names go through `process_name`, the program name with its controls
escaped, so doctor's and `up`'s and `down`'s texts can no longer carry another user's escape
sequence; `Text::checked` refuses by the same rule, and `toml_escape` escapes by it, now U+2028
and U+2029 too. Test-first: the three tests failed on the stubs' assertions, then passed.
`cargo mutants --in-diff` missed two at first: `toml_escape`'s guard, which no test exercised
before this batch either, killed by a test that renders a models directory with ESC and U+2028
and parses it back; and `read_gpu` replaced by `Ok(None)`, left: it needs NVML and a GPU, which
no unit test may use (CLAUDE.md), and the line this batch changed in it only calls
`process_name`, tested on its own. 15 caught, 1 unviable. The five checks green, 675 in libtest.

While writing that test, the session's edit tool decoded a `\u2028` it was given into the
character itself, which the test caught, and did it again in this entry; the session then
wrote such text through Python only. Searching for others found one in `docs/security.md` §7,
from phase 0's `docs: add docs/security.md, gathered from phase 0` of 2026-10-16: the TOML escape it names was the raw U+2028 between backticks, where
`\u2028` was meant. Fixed in this batch, outside the review's findings.

Committed as `fix(gpu): escape the control characters of GPU process names`; the Security section, batches 1 to 5, is done. Batch 6, finding 7:
`check_flags` refuses a text value that starts with `-`, with `InventoryError::OptionValue`,
naming the flag and the value: after a flag that takes none, llama-server would read it as the
next option, `jinja = "--api-key"` giving a backend key. Defaults and models alike. Test-first:
the test failed on `expect_err`, then passed. The README's inventory table says it. `cargo
mutants --in-diff`: two mutants, one caught, one unviable. The five checks green, 676 in
libtest.

Session closed at the user's request after batch 6 ("tu peux finir la tache en cours et clore,
on reprendra ds une nouvelle session"). How each batch ran, which the next session keeps:
test-first, the failure watched on an assertion; `git diff HEAD -- src` written to a file and
`cargo mutants --in-diff <file>` run with `TMPDIR` and `-o` in the session's scratchpad, every
missed mutant killed by a test of the batch or justified in this report; the five checks; then
the batch's task ticked and the Current Status and README lines rewritten from
`progress.py`'s output, and one commit with its report entry. The two helper scripts that did
the checks and the ticks lived in this session's scratchpad and are gone with it.

**Where the next session starts:**
1. On reprendra la phase 5 aux corrections : batch 7, finding 11 (non-finite float flags
   refused in `flag_values`, `inventory_file.rs:219`), then batches 8 to 59 in the Fix Plan's
   order of `docs/reviews/delivered-code/2026-10-17-v2-review-fixes.md`, batch 41 rejected;
   one batch per commit, as above.
2. After the 58 batches, the security pass: `review-security` and one `review-verifier` on the
   fixes' diff, from `docs(v2-review-fixes): record the closing review's verdicts and the fix scope`, the commit before batch 1, after the user's go-ahead; then fix
   what they confirm, and close the phase.
- The proposed tests and mutants of findings 23 to 49, and passes 4 and 5's verifiers'
  patches, are in `~/.local/share/ai-ctl/review-scratch/2026-10-18-v2-review-fixes-*/`: the
  test batches start from them.
- Text holding `\u2028` or `\u2029` is written through Python, never the Edit tool (Problems
  And Deviations); `grep -rnP '[\x{2028}\x{2029}]' docs src tests` finds none today.
- Nothing uncommitted after this entry's commit.

Resumed in a tenth session. Read before working: this phase's file and report, and the
review's finding 11 and Fix Plan. Nothing uncommitted, HEAD `fix(inventory): refuse a text flag value that starts with a dash`. Batch 7, finding 11:
`check_flags` refuses a float that is not finite, with `InventoryError::NonFiniteValue`, naming
the flag: TOML reads `nan` and `inf` as floats, and a model's `temp = nan` reached the command
and, after a whole `bench run`, failed the run's save. Defaults and models alike. The check sits
in the domain's `check_flags`, beside batch 6's, rather than in `flag_values`, where the fix
put it (Problems And Deviations). Test-first: the test failed on `expect_err`, the inventory
being accepted, then passed. The README's inventory table says it. `cargo mutants --in-diff`:
three mutants, two caught, one unviable. The five checks green, 677 in libtest and
`up_a_proxied_request_and_down ... ok`. The ticks and progress lines now come from a script in
this session's scratchpad, `tick.py`, which rewrites the task, the Current Status and the
README's two lines from `progress.py`'s output, then runs its `--check`.

Committed as `fix(inventory): refuse a flag value that is not a finite number`. Batch 8, finding 8: `Builder` gains the `GatewayProcess` port, and a
build of an installed id with `--force` is refused while the running gateway serves that build:
once after the checkout, so that a refusal does not wait for the compile, and again under the
gateway's lock, held only from that check through `publish`, since a gateway can start during
the compile. `refuse_served` now takes the command that would take the build away, prune's text
unchanged; the build's refusal names `ai-ctl down`, and says nothing is installed. A build that
replaces nothing takes no lock of the gateway's, so `up` stays free during an ordinary build.
Test-first: three of the four tests failed on `unwrap_err`, the build succeeding; the fourth, a
build the gateway does not serve, passed before and after. `cargo mutants --in-diff`: six
mutants, four caught, two unviable; it wrote none for the three guards in `build`, so each was
removed by hand in turn, and each failed its own test, and only it. `docs/commands.md` says it
in `build`, and the Concurrency table adds `build --force` to the gateway's lock. The five
checks green, 681 in libtest and `up_a_proxied_request_and_down ... ok`.

Committed as `fix(build): refuse --force over the build the running gateway serves`. Batch 9, findings 9, 62 and 63: `Current` gains `Unreadable { id,
reason }`, for a `current` link that names a directory that is not a readable build. `up`,
`gateway config` and `bench run` say "cannot be read: <reason>; run `ai-ctl build <project>
--ref <id> --force`", and doctor the same, with that remedy. `build` refuses an unreadable
directory of the id it would build as in the way, naming it and its reason, with `--force`:
before, it said "already built", and offered `build use`, which refuses an unreadable build.
`application::build::list` now sends a current build it cannot show, gone or unreadable, as a
warning, with the text of `Current::installed`; the renderer's stdout line goes. That fixes 62,
the line that "no build yet" skipped, and 63, the channel. Test-first: the six tests failed on
their assertions, the variant stubbed as `Missing`, then passed. Probed with the debug binary on
two scratch XDG states: `build list` with only `current -> b99999` warns on stderr above "no
build yet", exit 0; with `b11700/build.json` lacking `project`, it warns with the reason and
`--force` above its "not a build" line, and doctor gives the same detail and remedy.
`docs/commands.md` says it in `build` and `build list`. `cargo mutants --in-diff`: 16 mutants,
9 caught, 7 unviable. The five checks green, 683 in libtest and `up_a_proxied_request_and_down
... ok`.

Committed as `fix(build): tell an unreadable current build from a missing one`. Batch 10, finding 22: `CdiSpec` gains `missing`, the host paths it
mounts that are not on disk. `parse_cdi_host_paths` collects the `hostPath` values, YAML and
JSON, and the probe keeps those whose `fs::metadata` gives `NotFound`, a dangling link
included; a path it cannot look at is not counted. Doctor's CDI check warns after the driver
check, which says more when a driver change is the cause: "<spec> mounts <path>, which is not
on disk: podman starts no container with the GPU", or the count and the first path, with the
remedy `sudo systemctl start nvidia-cdi-refresh.service`. A specification that names no driver
is checked all the same. `CDI_REMEDY`'s comment now names what the path unit watches. The
parser is tested on a new capture, `tests/fixtures/nvidia-ctk-1.20.1-cdi-spec.yaml`, the live
specification the refresh wrote at 15:01, whole, 60 `hostPath` entries, the GPU's UUID zeroed,
listed in `tests/fixtures/README.md`; the stale one was gone with that refresh. The existing
test of the two directories now compares paths and drivers only, since which of the captures'
host paths exist depends on the machine. Test-first: the three tests failed on their
assertions, the parser stubbed empty; then one failed on its own index, the test's error, 30
for 15. `cargo mutants --in-diff`: 17 mutants, 8 caught, 9 unviable. It mutates no `let ...
else`, so the case of a specification without a driver was added, and moving the missing
check below the driver's guard by hand failed it. Doctor on this machine: `ok CDI
specification`, the live one being current. `docs/stack.md` §7 records the unit's watched
paths and the 2026-10-18 failure, with the review as source. The five checks green, 686 in
libtest and `up_a_proxied_request_and_down ... ok`.

Committed as `fix(doctor): warn of a CDI specification that mounts a path gone`. Batch 11, findings 10 and 79: each of the three readers reads its
version key alone before the strict parse. The run store and the build store share
`files::json_format`, a struct holding only `format`, whose module's doc now says it holds file
format helpers too; the inventory's `parse` reads a `VersionDto` holding only `version`. A run
of format 4 with a field `notes` and no `started`, a `build.json` of format 2 with `lto`, and an
inventory of version 2 with `vram_mb` or `models_root` now read as their unsupported format or
version, not as malformed. `FileDto` keeps its `version` field for the strict parse to accept
the key, under `#[expect(dead_code)]` with its reason, the crate's first. Test-first: the three
tests failed on their assertions, serde's "unknown field", then passed. Probed with the debug
binary on a scratch XDG state: `build list` names "build metadata format 2 is not supported",
and doctor "unsupported inventory version 2", still with the remedy that finding 80, the next
batch, replaces. `cargo mutants --in-diff`: five mutants, two caught, three unviable. The five
checks green, 686 in libtest and `up_a_proxied_request_and_down ... ok`.

Committed as `fix(store): read a file's format alone before its strict parse`. Batch 12, finding 80: `domain::inventory::UnsupportedVersion { found,
reads }`, beside `InvalidInventory`, keeps the version's text, and `inventory_file::parse`
returns it in place of the `ensure!`. Doctor's Inventory check downcasts it, as
`config_check` does `ConfigError`, and when `found` is above `reads` gives "<file> is written
for a newer ai-ctl: install that one, or move the file away and run `ai-ctl init --models-dir
<dir>`"; a version below, which no ai-ctl writes, keeps the remedy to fix the file.
`FakeInventory`'s error is now a `LoadFailure`, a message or the typed version wrapped in the
real store's context, so that the doctor test downcasts through it; its two callers' `"…".into()`
are unchanged. Test-first: both tests failed on their assertions, the type stubbed and unused,
then passed. Probed with the debug binary on a scratch inventory of version 2 with
`models_root`: the new remedy. `cargo mutants --in-diff`: nine mutants, six caught, two
unviable, one missed and left: `>` for `>=` in the guard, which differs only when `found`
equals `reads`, a value `parse` never returns, since it returns the error only when the
version differs. The five checks green, 687 in libtest and `up_a_proxied_request_and_down ...
ok`.

Committed as `fix(doctor): give an inventory of a newer version its own remedy`. Batch 13, finding 12: on a recorded gateway that runs on the current
configuration, `up` makes one health check, `CHECK`'s 500 ms, before "up already". A silent
gateway, or a check that fails, is an error: "the gateway (PID <pid>) runs but does not answer
on <listen>[ (<error>)]; run `ai-ctl down`, then `ai-ctl up`", with the log's last lines; the
gateway is left running and recorded, for `down`. Test-first: the left-alone test now needs one
health call and the new test an error, and both failed on their assertions. `cargo mutants
--in-diff`: one mutant, unviable, `up` replaced by `Ok(Default::default())`; it mutates no
`match` arm, and the three, an answer, silence and an error, are each asserted.
`docs/commands.md` says it under `up`. The five checks green, 688 in libtest and
`up_a_proxied_request_and_down ... ok`.

Committed as `fix(up): check a recorded gateway's health before "up already"`. Batch 14, finding 14: `http::send` and `open` return a `Connection`, the
socket with its read timeout, whose `Read` turns `WouldBlock` and `TimedOut` into "no answer
within <N>", `300 s` or `500 ms`. One place covers `request`, `read_head` and `Body`, so the
health wait, `running` and the chat stream all say it; `open`'s two callers read it as a
`dyn Read`, unchanged. The fake of a hanging health check now gives the same text, and the
lifecycle test that quotes it follows. Test-first: a silent server, one that accepts and never
answers, gave "Resource temporarily unavailable (os error 11)" against the expected text, then
passed; the timeouts are 50 ms, and the formatter is tested on its own rather than through a
second-long read. `cargo mutants --in-diff`: eight mutants, five caught, one unviable, and two
timeouts, both detected: `read` replaced by `Ok(1)` never ends, and `send` replaced by
`Ok(None)` makes the real `up` wait its 30 s. The five checks green, 689 in libtest and
`up_a_proxied_request_and_down ... ok`.

Committed as `fix(http): tell a read timeout as no answer within its time`. Batch 15, finding 82: `prepare_on`'s body is split into a private
`resolve()`, the inventory loaded and the runtimes resolved, and `size()`, the configuration
sized to a reading; `prepare` reads the GPU between the two, so that `gateway config` without
an inventory, or with a backend whose build is missing, never loads NVML. The verifier's patch,
`review-scratch/2026-10-18-v2-review-fixes-pass-5/verifier/repo82.patch`, applied as it was;
`resolve()` returns the pair batch 46 will make `Prepared`'s. Test-first: `ScriptedGpu`'s
`reads` counted one read without an inventory against none expected, then the three cases
passed: no inventory, no build, both there and one read. `cargo mutants --in-diff`: six
mutants, all unviable, each a body replaced by a `Default` that `Prepared` and `Inventory` do
not have, as the logs' E0277 shows. The five checks green, 690 in libtest and
`up_a_proxied_request_and_down ... ok`.

Committed as `fix(gateway): read the GPU after the inventory in gateway config`. Batch 16, finding 20: `files::replace_file` writes a temporary named
by `temporary_for`, renames it over the path, and removes it when the write or the rename
fails; `LocalGateway::write` now goes through it for the record and the configuration. The
temporary's name changes from `.<name>.tmp` to `.<name>.<pid>.tmp`, the one batch 23 sweeps; no
document or test named the old one. It keeps `fs::write`'s mode and does not sync, as before.
Test-first: with the helper written without its removal, a directory in place of the file left
`.gateway.json.<pid>.tmp`, then the test passed. `cargo mutants --in-diff`: two mutants, both
caught. The five checks green, 691 in libtest and `up_a_proxied_request_and_down ... ok`.

Committed as `fix(gateway): remove the temporary file of a failed write`. Batch 17, findings 21 and 39: `prune`'s error on a failed removal now
reads "cannot remove <project> <id>, after removing <the builds gone already>", the plain text
when nothing was removed before it. The test `FakeStore` gains `remove_fails`, which holds the
id whose removal fails rather than a switch, since 21's test needs a removal that succeeds
before one that fails; the name keeps batch 47's `<method>_fails`. Finding 39's test, from the
kept `review-tests.patch` with its `review_` prefix dropped and the module's blank line before
`#[test]`, passed before the fix, as a test gap's does; 21's failed on its assertion, then
passed. `cargo mutants --in-diff`: two mutants, one caught, one unviable; it does not mutate the
`?` on `store.remove`, so M8, the removal's result ignored, was applied by hand: both tests
fail under it. `docs/commands.md` says it under `build prune`. The five checks green, 693 in
libtest and `up_a_proxied_request_and_down ... ok`.

Committed as `fix(build): name the builds a failed prune removed before it`; batches 6 to 17, bugs and error handling, are done. Batch 18, findings
23, 49 and 13: `status` no longer takes the gateway's lock to observe. It reads the record and
its liveness; only a record whose process is gone makes it take the lock, and then it calls
`running()`, which reads the record again under the lock, since an `up` may have recorded
another gateway in between, and removes it. While another ai-ctl holds the lock, the stale
record is left with a warning naming the lock's error. Finding 23's test, from the kept patch,
`up_and_down_wait_for_no_one`, passed before the fix, as a test gap's does. Finding 49's test,
the verifier's `t49.patch`, asserted that `status` fails under the lock; 13's fix changes that
behaviour, so the test now asserts the new one with the same setup: `status` succeeds, warns,
and leaves the record (Problems And Deviations). A third test, `status` observing a running
gateway while the lock is held, is 13's. Test-first: the two `status` tests failed on `unwrap`,
`status` refusing with the busy message, then passed. `cargo mutants --in-diff`: four mutants,
three caught, one unviable; the lock's removals are no mutants of its, so four were applied by
hand: M22 and M23, `up`'s and `down`'s lock removed, fail 23's test; `status` forgetting
without the lock, M29's counterpart, fails 49's; `status` locking always fails both new tests.
`docs/commands.md`'s Concurrency table and `status` section say when `status` takes the lock.
The five checks green, 696 in libtest and `up_a_proxied_request_and_down ... ok`.

Committed as `fix(status): observe without the gateway's lock`; its first commit left out this report's entry under Problems And
Deviations, a check of the session's script failing after the Work Log's write, and was
amended. Batch 19, findings 16 and 33: `Recorded` gains `failure`, the request that ended the
run. A run with scenarios completed before the failure is saved and returned with it; with none
completed, the failure is the error, as before. Presentation's new `report` prints the run to a
writer, then "Saved as", then fails with the request's error, or with the save's after the run
is printed, or with both; `print` now goes through `print_to`, the same closed-pipe rule for
any writer. Finding 33's test is on `report` with a `Vec<u8>` rather than on a test `Services`
of the fakes: the verifier left that choice, and a writer keeps the output out of libtest's
with `fakes` private to `application`, so `docs/architecture.md:115` and `fakes.rs:1` stay
true and unchanged (Problems And Deviations). The test `World::run` returns a partial run's
failure as its error, so that the tests that read it as a run stay strict. Test-first: three
tests failed on their assertions, 33's passed before the fix, as a test gap's does. `cargo
mutants --in-diff`: five mutants, four caught, one unviable; it does not mutate `report`'s arms,
so three were applied by hand: M28, an unsaved run that succeeds, fails 33's test; a saved
partial run that succeeds and a double failure that loses the request's fail 16's.
`docs/commands.md` says it under `bench run`. The five checks green, 699 in libtest and
`up_a_proxied_request_and_down ... ok`.

Committed as `fix(bench): print and save the scenarios a failed request leaves`. Batch 20, finding 15: `RunStore` gains `replace`, which writes a run in
place of the file its `save` wrote, through batch 16's `files::replace_file`, whole or not at
all. `Bench::run` saves the run after each completed scenario: the first time by `save`, which
still refuses a run saved before under that ID, then by `replace`; the final save, with the
GPU's readings and the drafts, and batch 19's partial save go the same way, else they would be
refused as already saved. A save that fails during the run warns once, "the run cannot be saved
as it is measured (…), so a stop before its end loses it", and the end's save reports its own
failure as before. A Ctrl-C now loses only the scenario it interrupts; the run saved before it
holds no GPU readings and no drafts, which only the end gathers. The test `FakeRuns` logs each
write and refuses a `replace` of a run never saved. Test-first: four tests failed on their
assertions, `replace` stubbed as `save`, then passed once a test's own filter left out the
World's other warning. `cargo mutants --in-diff`: six mutants, five caught, one unviable.
`docs/commands.md` says it under `bench run`. The five checks green, 700 in libtest and
`up_a_proxied_request_and_down ... ok`.

Committed as `fix(bench): save the run after each scenario`. Batch 21, findings 17 and 34: before the build store sweeps a project's
hidden entries, `put_back_replaced` renames each `.replaced-<id>-<pid>` back to `<id>` when
nothing is there: a kill between `publish`'s two renames left it the only copy of the build. A
hidden copy of a build in place is a leftover, swept as before. Finding 34's test, from the
kept patch, passed before the fix, as a test gap's does; 17's, the state a kill leaves built by
hand, lost b1 to the sweep, then passed. `cargo mutants --in-diff`: five mutants at first, one
missed: the guard on `read_dir`'s `NotFound` replaced by `true`, equivalent, since the sweep
that follows reads the same directory and reports the same "cannot read" error. The function
now leaves an unreadable directory to that sweep, which removes the guard: two mutants, both
caught. M17, the put-back of `publish`'s failure path removed, applied by hand, fails 34's
test. The five checks green, 702 in libtest and `up_a_proxied_request_and_down ... ok`.

Committed as `fix(build): put back a build an interrupted replacement left hidden`. Batch 22, findings 18 and 24, under the user's verdict: no `libc` and
no unsafe code. `docs/commands.md`'s `up` section records the window, a write and a rename long,
between llama-swap's start and its record: an interrupted `up` there leaves an llama-swap that
`down` and `status` do not see and that the next `up` refuses as another program; `pgrep -af
llama-swap` names it, ai-ctl's own release, `kill <PID>` ends it, and the key needs no change
for it, which tempers batch 2's advice in that one case. The `down` section points there. Finding
24's test, from the kept patch, makes the record's rename fail on a directory after the spawn,
scans `/proc` for the script's command line, kills what it finds, and asserts nothing was left;
it passed before, as a test gap's does. M6, the kill and the wait removed, applied by hand,
fails it with "left running: [<pid>]", and the test's own cleanup ended that process: nothing
ran as a `llama-swap` script afterwards (`ps`). `cargo mutants --in-diff` had nothing to mutate,
the diff's code being a test's. The five checks green, 703 in libtest and
`up_a_proxied_request_and_down ... ok`.

Committed as `docs(up): document the window before the gateway's record`. Batch 23, finding 19: `files::write_new` and batch 16's `replace_file`
first sweep their directory of the temporaries a killed writer left: `.<name>.<pid>.tmp`, as
`temporary_for` names them, whose `/proc/<pid>` is gone. A reused PID keeps its file, a cautious
miss, and a file that cannot be removed stays: both are clutter, never a wrong file. The sweep
covers every call site at once, `bench/`, the inventory's and the key's directories, and the
gateway's, each read once per write. Test-first: with the sweep stubbed empty, two temporaries of
a gone PID, `u32::MAX`, which no process has, stayed beside this process's own, a hidden name
without a PID and a visible one; then they went, for `write_new` and `replace_file` alike.
`cargo mutants --in-diff`: nine mutants, all caught. The five checks green, 704 in libtest and
`up_a_proxied_request_and_down ... ok`.

Committed as `fix(files): sweep the temporaries of writers that no longer run`. The tenth session ended there, its context window overflowing, before it
wrote a closing entry; nothing was uncommitted.

Resumed in an eleventh session. Read before working: this phase's file and report, and the
review's finding 51 and Fix Plan. Nothing uncommitted, HEAD `fix(files): sweep the temporaries of writers that no longer run`. The tenth session's
`tick.py` was still in its scratchpad, and was copied into this one's.

Batch 24, finding 51, under the verdict without the `GatewayState` supertrait: `Inspector`,
beside `Lifecycle`, holds the ports the inspection reads, `configurator`, `releases`, `gateway`,
`processes`, `api` and `containers`, and the eight methods that only read: `inspect`,
`observe`, `target`, `is_current`, `owners`, `ancestry`, `in_container` and `leftovers`. Doctor
builds an `Inspector` alone, its `lifecycle()` replaced by `inspector()`, and `config_check`
calls a `Configurator` of its own; `Doctor::pause` and `now` go, with the six lines that set
them in presentation and in doctor's two test constructions. `Lifecycle` keeps its ports and
builds its `Inspector` from them for the six calls that read, rather than holding it as the fix
wrote (Problems And Deviations); `Configurator` derives `Copy` for that. No new test for the
move: nothing observable changes, and the existing tests are the net, as for phase 4's batch
37. `cargo mutants --in-diff`: 44 mutants, the moved methods' first run; 31 caught, 11
unviable, one timeout, `||` for `&&` in `ancestry`'s loop, which then walks a loop of parents
for ever, and one missed, `>=` for `>` in its guard, which reads PID 1 too. PID 1 is never the
gateway, a conmon or a container's root, so that mutant changes no placement, but the walk is
documented to end below PID 1: `the_walk_ends_below_pid_1` kills it, the mutant applied by hand
failing that test and only it. A second run, on the diff with the test, was interrupted at 143
s, with no process left; run again: 44 mutants, 32 caught, 11 unviable, the same timeout, none
missed. The five checks green, 705 in libtest and
`up_a_proxied_request_and_down ... ok`.

Committed as `refactor(doctor): inspect the gateway without the lifecycle's waits`. Batch 25, finding 69: `up` unwraps the GPU's reading once after
`target`, which sized the configuration to it, and its `warn_unread`, "not looking for processes
left on the GPU", goes with the `if let` around the leftovers' check: the reading of an `up`
that reaches that check is always found, as M31 showed. `found_gpu`'s `?` stands in its place,
never failing, rather than an `expect`. `down`'s and `status`'s `warn_unread` can be reached and
stay. No new test: the branch removed could not run, and `up`'s tests of the leftovers are the
net. `cargo mutants --in-diff`: two mutants, one caught, one unviable. The five checks green,
705 in libtest and `up_a_proxied_request_and_down ... ok`.

Committed as `` refactor(up): drop the GPU warning no `up` can reach ``. Batch 26, finding 50, under the verdict without a migration of the saved
runs: `// LEGACY:` comments at the seven places of the run store's read path for formats 1 and
2, `READS`, `BUILD_JSON_FORMAT`, `BuildJson::format` and its check in `build()`, the
`#[serde(default)]` of `policy` and of `clock_events`, and the domain's
`GpuReading::clock_events`. The one at `READS` states the obligation, the runs saved before
format 3 kept readable for `bench list` and `bench compare`, and when the path goes: once no
such run is kept, or once they are migrated; the others point to it. Comments only, so no test:
`grep -rn LEGACY src` names the seven, and the debug binary's `bench list` on this machine's
runs, six of format 1 and one of format 2, lists the seven, exit 0. `cargo mutants --in-diff`
found nothing to mutate. The five checks green, 705 in libtest and
`up_a_proxied_request_and_down ... ok`.

Committed as `docs(bench): state why the run store reads formats 1 and 2`; batches 24 to 26, the dead code, are done. Batch 27, findings 25 and
26: one failing writer, `ClosedPipe`, and `broken_pipe`, whether an error's chain holds its
`BrokenPipe`, in `application::fakes`, which becomes `pub(crate)` under `cfg(test)` so that
`http.rs`'s tests reach it: application cannot import infrastructure, and infrastructure may
import application. `fakes.rs`'s module doc says so; `docs/architecture.md` says nothing of the
module's reach. The two tests come from the kept `review-tests.patch`, their `review_` prefix
dropped and their own `ClosedPipe` replaced by the shared one: `log(true, …)` on a closed pipe
fails with "cannot write the log" and follows nothing, and `copy_stream` with "cannot write the
stream". Both passed on the code, as a test gap's do; M4 and M26, from the verifier's
`mutants.py`, applied by hand one at a time, each fail their own test and only it. `cargo
mutants --in-diff` found nothing to mutate, the diff being tests'. The five checks green, 707
in libtest and `up_a_proxied_request_and_down ... ok`.

Committed as `test(logs): a reader that has gone ends the log and the model's stream`. Batch 28, finding 27: `a_gateway_that_outlives_sigterm_is_killed_by_down`,
whose setup is the one the fix names, `serving(0)` with a stubborn gateway, now also asserts that
the fake clock moved at least 35 s during `down`, llama-swap's own stop, rather than the
constant's 40 s, as the verifier preferred; the kept test, a new one with the same setup and the
exact 40 s, was not added. It passed on the code, as a test gap's does; M1, `STOP_POLLS` at 4,
applied by hand, fails it and only it. Batch 43's deadline keeps the assertion. `cargo mutants
--in-diff` found nothing to mutate. The five checks green, 707 in libtest and
`up_a_proxied_request_and_down ... ok`.

Committed as `test(down): llama-swap gets its 35 s between SIGTERM and SIGKILL`. Batch 29, finding 28: `FakeApi` gains `first_check`, how long the first
health check takes on the test's clock, within its timeout, before it fails; the later checks
keep `hangs` or fail at once, as `silent()` gives. `a_start_never_waits_past_its_30_s`, the kept
test's two cases written on `FakeApi` rather than its `SlowApi`, as the verifier asked: a first
check of 250 ms, then checks that hang, and one of 30 ms, then checks that fail at once; in
both, the failed `up` ends exactly 30 s after it started. It passed on the code, as a test
gap's does; M2 and M3 applied by hand, one at a time, fail it alone, at 30.25 s and 30.03 s.
`cargo mutants --in-diff` found nothing to mutate, the diff being tests'. The five checks
green, 708 in libtest and `up_a_proxied_request_and_down ... ok`.

Committed as `test(up): a start never waits past its 30 s`. Batch 30, findings 29 and 47: `FakeGateway` gains `record_fails`, with
the real adapter's kind of error, "cannot read /state/gateway/gateway.json: Permission denied
(os error 13)". Two doctor tests from the kept patch, their `review_` prefix dropped: an
unreadable record fails the Gateway check with that error and gives the report a failure; a
running gateway with no model loaded reads "running on http://127.0.0.1:8080 (PID 1234), no
model loaded". Both passed on the code, as a test gap's do; M5 and M13 applied by hand, one at
a time, each fail their own test and only it. `cargo mutants --in-diff` found nothing to
mutate. The five checks green, 710 in libtest and `up_a_proxied_request_and_down ... ok`.

Committed as `test(doctor): an unreadable gateway record, and a gateway with no model`. Batch 31, findings 30, 31 and 46, three tests from the kept patch, their
`review_` prefix dropped: the overhead scenario, with another model listed first on
`127.0.0.1:5800`, sends nothing there and its 50 direct requests to the measured model's own
server; in `tests/cli.rs`, each `--scenario` value maps to the scenario of its name, through
`ScenarioArg::value_variants()` and `Scenario::from`; and `FakeMonitor` gains `finish_fails`,
under which a run succeeds with no GPU readings and warns "the GPU's reading failed". The three
passed on the code, as a test gap's do; M9, M10 and M15 applied by hand, one at a time, each
fail their own test and only it in its test binary. `cargo mutants --in-diff` found nothing to
mutate. The five checks green, 713 in libtest and `up_a_proxied_request_and_down ... ok`.

Committed as `` test(bench): the overhead's own server, `--scenario`'s mapping, the monitor's failure ``. Batch 32, finding 32: `status_with_a_running_gateway_exits_0`, in
`tests/cli.rs`, from the kept patch with the verifier's remark taken: the `sleep 60` that plays
the recorded llama-swap, through a link to `/usr/bin/sleep`, is held by `Running`, which kills
and waits for it on drop, so that a panic in `ai_ctl()` leaves nothing behind. The record names
its PID, the link and a port nothing listens on, which only makes `status` warn that it cannot
list the models; the test asserts exit 0 and the "Gateway     running on http://…" line. It
passed on the code, as a test gap's does; M16, `status` always exiting 3, applied by hand,
fails it alone with `--no-fail-fast`, and the guard ended the `sleep` through that panic
(`pgrep`). No `src` change, so nothing to mutate. The five checks green, 713 in libtest, 27 in
`tests/cli.rs`, and `up_a_proxied_request_and_down ... ok`.

Committed as `test(status): exit 0 while a gateway runs`. Batch 33, findings 35, 36, 37 and 44, four adapter tests from the kept
patch, their `review_` prefix dropped, each at the end of its module: a second fetch from an
upstream that is gone fails with "cannot fetch <url>" (35); a cmake that fails its configure
gives "cmake configure failed (exit status: 1)", ends with its stderr line, and never runs
`--build` (36); a g++ that exits 127 with a loader error is a failed probe naming it, not a
missing compiler (37); and a branch `b1` beside the tag `b1` checks out the tag's commit (44).
Every one runs on a local upstream or a script in a temporary directory, no network. The four
passed on the code, as a test gap's do; M18, M19, M20 and M12 applied by hand, one at a time,
each fail their own test and only it, with `--no-fail-fast`. `cargo mutants --in-diff` found
nothing to mutate. The five checks green, 717 in libtest and `up_a_proxied_request_and_down ...
ok`.

Committed as `test(build): failing fetch, configure and g++, and a tag over a branch`. Batch 34, findings 38 and 41: `up_refuses_an_address_it_cannot_bind`,
from the kept patch, on `FakePorts::is_free_fails`, which only doctor's tests set: `up` fails
with the bind error and starts nothing. `FakeGateway` gains `signal_fails`, and
`a_signal_that_cannot_be_sent_ends_down_at_once`: `down` fails with the kill's error at once,
the fake clock unmoved, the record kept. The fake's text follows the real adapter's, `kill -s
<signal> <pid>: <kill's stderr>`, with util-linux's own refusal, captured here by `/usr/bin/kill
-s 0 1` as an unprivileged user, a signal that only checks the permission: "kill: sending
signal to 1 failed: Operation not permitted"; the kept patch's "kill: Operation not permitted"
was written from memory. Both passed on the code, as a test gap's do; M21 and M14 applied by
hand, one at a time, each fail their own test and only it. `cargo mutants --in-diff` found
nothing to mutate. The five checks green, 719 in libtest and `up_a_proxied_request_and_down ...
ok`.

Committed as `test(lifecycle): an address up cannot bind, a signal down cannot send`. Batch 35, findings 40 and 45, two tests from the kept patch, their
`review_` prefix dropped; the patch's prune test and `remove_fails` beside them landed with
batch 17. `use_waits_for_no_one`: on a locked build store, `use_build` fails with "another
ai-ctl is changing the builds", asserted whole rather than the patch's `contains`, and leaves
`current` on b2. `the_current_release_is_not_linked_again`: the pinned release installed and
current, with `set_current_fails`, installs as `Unchanged`. Both passed on the code, as a test
gap's do; M24 and M7 applied by hand, one at a time, each fail their own test and only it.
`cargo mutants --in-diff` found nothing to mutate. The five checks green, 721 in libtest and
`up_a_proxied_request_and_down ... ok`.

Committed as `` test(build): `build use` under the builds' lock, and the current llama-swap reinstalled ``. Batch 36, findings 42 and 43, two tests from the kept patch, their
`review_` prefix dropped: a native program path with a Latin-1 byte, `/home/s\xe9b/…`, is
refused with `ConfigError::NonUtf8Path` naming the model and the path, and `render` of the
shipped inventory with `/data/mod\xe8les` fails with "… is not valid UTF-8". The blocks were
copied from the patch by a script, and the test's `\u{FFFD}` escape checked to stay an escape:
`grep -rnP '[\x{2028}\x{2029}\x{FFFD}]' docs src tests`, the reviews aside, finds none. Both
passed on the code, as a test gap's do; M11 and M27, taken from the verifier's `mutants.py`
and applied one at a time, each fail their own test and only it. `cargo mutants --in-diff`
found nothing to mutate. The five checks green, 723 in libtest and
`up_a_proxied_request_and_down ... ok`.

Committed as `test(config): program and models paths that are not UTF-8 are refused`. Batch 37, finding 48: `a_change_from_zero_is_not_told`, from the kept
patch, its `review_` prefix dropped: a relative change from a first mean of 0 prints "–", and
the reverse "-100.0%", a case of the same rule that the verifier kept though it does not guard
the mutant. It passed on the code, as a test gap's does; M25, the `first.mean != 0.0` guard
dropped, applied by hand, fails it and only it. `cargo mutants --in-diff` found nothing to
mutate. The five checks green, 724 in libtest and `up_a_proxied_request_and_down ... ok`.

Committed as `test(bench): a change from zero is not told`, amended: its first commit said the test section ended with batch 37,
though batch 38 belongs to it. Batch 38, findings 71 and 81: the verifier's capture of `GET
/upstream/nope/props`, kept as `props-nope.http`, copied byte for byte as
`tests/fixtures/llama-swap-263-upstream-props-unknown-model.http`, its 404 "model not found",
121 bytes of body, and its row in `tests/fixtures/README.md`'s llama-swap section, whose header
now names the closing review as the source of the rows marked (R). Served by
`an_error_answer_names_the_endpoint`, in place of the chat request's capture, and by
`only_a_200_response_gives_its_body`'s `GET /x`, in place of a body written from memory;
`an_error_body_gives_its_message` reads the body of the chat capture it had been cut down from.
Test-first for 81: the expected "model not found" failed on its assertion against the old
capture, then passed on the new one; the two others pass on their captures. `grep -rn 'no such
model' src tests` finds nothing. `cargo mutants --in-diff` found nothing to mutate. The five
checks green, 724 in libtest and `up_a_proxied_request_and_down ... ok`; batches 27 to 38, the
tests, are done.

Committed as `test(http): error answers on captures of their own request`. Batch 39, finding 53: `domain::machine::toolkit_package`, beside its
inverse `versioned_toolkit`, names a release's meta-package, `cuda-toolkit-<major>-<minor>`, and
`Setup::package`, `newer_toolkit_remedy` and `package_check` call it with `Version::release()`;
the remedy keeps its `<major>-<minor>` text for the `grep -E` pattern. `grep -rn
'cuda-toolkit-{' src` finds the domain's alone. Test-first: the round trip through
`versioned_toolkit` failed on its assertion against a stub returning nothing, then passed.
`cargo mutants --in-diff`: ten mutants, eight caught, one unviable, one missed: the guard on
`package_check`'s `keep`, set to `false`, which the tests could not see since the healthy
toolkit is at the driver's release. `an_unversioned_package_keeps_the_installed_toolkits_release`,
a toolkit 13.1 under a CUDA 13.4 driver, kills it, the mutant applied by hand failing it alone;
run again, nine caught, one unviable. The five checks green, 726 in libtest and
`up_a_proxied_request_and_down ... ok`.

Committed as `refactor(doctor): format the toolkit package name once`. Batch 40, finding 83: the toolkit-package probe asks rpm for exact
names, which go through its index, in place of `-qa 'cuda-toolkit*'`, which loads every header.
`ToolchainProbe::toolkit_packages` takes the releases to ask about: `toolchain::inspect` gives
the driver's CUDA release and the found toolkit's, and `SystemHost` adds those of the
`/usr/local/cuda-<major>.<minor>` directories, through a new field `toolkits`, `/usr/local` but
a temporary directory in its tests. `domain::machine::toolkit_meta_packages`, beside batch 39's
`toolkit_package`, names them: `cuda-toolkit`, and for each release `cuda-toolkit-<major>` and
`cuda-toolkit-<major>-<minor>`, sorted, each once. `rpm -q` prints a missing name's line on
stdout and adds 1 to its exit status, so `parse_installed` keeps only the lines equal to a name
asked, whatever rpm's wording, as the verifier refined, and a nonzero exit with nothing on
stderr is not a failure; rpm failing, with stderr, still is. The new capture,
`tests/fixtures/rpm-q-cuda-toolkit-packages.txt`, taken here under `LC_ALL=C` with rpm 6.0.2:
three names not installed and `cuda-toolkit-13-4`, exit status 3, stderr empty; its row in
`tests/fixtures/README.md` marks this report as its source (5). The old capture of `rpm -qa`,
`rpm-cuda-toolkit-packages.txt`, which no command ai-ctl runs prints any more, is removed with
its row and its use in `package_names_are_sorted_once_each`. `FakeToolchain` records the
releases it is asked about, and its lists drop the `-config-common` packages an exact query
never returns. Measured here, three runs each: the pattern 0.08 s, the exact names 0.00 s. The
limit the finding states holds: a meta-package installed without its `/usr/local` directory, of
a release neither the driver's nor the toolkit's, goes unseen. Test-first: the domain's names,
the parser on the capture, the probe's exact arguments with a scratch `cuda-13.2` beside the
`cuda-13` and `cuda` links, and the releases `inspect` asks about, each failed on its
assertion against stubs, then passed. The release binary's `doctor` on this machine: `ok
Toolkit packages cuda-toolkit-13-4`, 0.19 to 0.29 s over five runs, measured while `cargo
mutants` ran, so not a figure to set against the review's 259 to 282 ms. `cargo mutants --in-diff`: 33 mutants, 21 caught, 11 unviable,
one missed: `||` for `&&` in `parse_toolkit_dir`'s check that a number is all digits, which
differs only for a name such as `cuda-+13.2`, since `u32`'s parse already refuses the rest.
The check went, a directory's name needing no more than the parse; run again, 31 mutants, 20
caught, 11 unviable. The five
checks green, 730 in libtest and `up_a_proxied_request_and_down ... ok`.

Session closed at the user's request after batch 40: "tu peux finir les opération en cours et
consigner". This eleventh session fixed batches 24 to 40: the dead code (24 to 26), the tests
(27 to 38) and the first two of performance (39, 40), one commit each, by the method of the
tenth session's last entry. Each test gap's test passed on the code, and the mutant its finding
names, taken from the verifier's `mutants.py`, was applied by hand and failed that test alone;
`cargo mutants --in-diff` then found nothing to mutate in a diff of tests. Two of its runs on
code found what no earlier run had seen, on lines the batch moved or touched: `ancestry`'s PID
guard (batch 24) and `package_check`'s `keep` guard (batch 39), each killed by a test of the
batch. The kept tests were taken with their `review_` prefix dropped and adapted where the code
had moved since: batch 28 put its assertion in the existing test of the same setup, batch 32
added a guard that kills its `sleep` on a panic, batch 34 worded the kill's error after a
capture of util-linux's own. `tick.py`, which ticks a batch and rewrites the phase's Current
Status and the README's two lines from `progress.py`'s output, then runs its `--check`, was
copied from the tenth session's scratchpad into this one's, on
a tmpfs that a reboot clears.

**Where the next session starts:**
1. On reprendra la phase 5 aux corrections : batch 42, finding 85 (the session scenario's bare
   prompt counted once, with the verifier's `repo85.patch` and `repo85-probe.patch` in
   `~/.local/share/ai-ctl/review-scratch/2026-10-18-v2-review-fixes-pass-5/verifier/`), then
   batches 43 to 59 in the Fix Plan's order of
   `docs/reviews/delivered-code/2026-10-17-v2-review-fixes.md`; batch 41 was rejected and has
   no task. One batch per commit, as above.
2. After batch 59, the security pass: `review-security` and one `review-verifier` on the fixes'
   diff from `docs(v2-review-fixes): record the closing review's verdicts and the fix scope`, after the user's go-ahead; then fix what they confirm, and close the
   phase.
- `tick.py` is in this session's scratchpad above, if the machine has not rebooted; otherwise
  write it again from the description above.
- Text holding `\u2028`, `\u2029` or `\u{FFFD}` is written through Python, never the Edit
  tool.
- Nothing uncommitted after this entry's commit.

### 2026-10-19

Resumed in a twelfth session. Read before working: this phase's file and report, and the
review's finding 85 and Fix Plan. Nothing uncommitted, HEAD `perf(doctor): ask rpm for the toolkit meta-packages by exact name` of 2026-10-18. The machine had rebooted
at 08:43, so the eleventh session's `tick.py` was gone; it was written again from its
description, beside a `checks.sh` that runs the five checks, both in this session's scratchpad.
The kept patches under `~/.local/share/ai-ctl/review-scratch/` are intact.

Batch 42, finding 85: `turn_extra` subtracts `self.own`, the bare prompt's count that
`Prompts::load` stored, rather than counting it again; `opening` goes with it. Test-first:
`a_session_counts_the_agents_bare_prompt_once`, from the verifier's `repo85-probe.patch` made an
assertion, pins a session run at three templates and three counts: the bare prompt at load, the
opening, exact at the first attempt with the fake's tokenizer, and the opening followed by a
turn. It failed on its assertion, (4, 4), then passed. `cargo mutants --in-diff`: two mutants,
both caught. The five checks green, 731 in libtest and `up_a_proxied_request_and_down ... ok`.

Committed as `perf(bench): count the agent's bare prompt once in a session`. Batch 43, finding 87: `stop` checks the gateway at once after each
signal, then after pauses that start at 5 ms and double up to `POLL`, 100 ms, each clamped to
the time left, until a deadline on the injected clock: `STOP_WAIT`, 40 s after SIGTERM, and
`KILL_WAIT`, 5 s after SIGKILL, in place of `STOP_POLLS` and `KILL_POLLS`. A `down` whose
llama-swap ends a millisecond after SIGTERM now waits 5 ms, not 100. `FakeGateway` gains
`lingers`, how many checks a gateway the signals ended still runs. Test-first:
`a_stopping_gateway_is_checked_soon_then_every_100_ms`, seven checks lingering, failed on its
assertion, 700 ms against 355, then passed; `the_waits_after_each_signal_end_on_their_deadlines`,
an immortal gateway ending `down` exactly 45 s after it started, passed on the old code, whose
fixed polls divide both waits, and guards the clamp, which the growing pauses need. Batch 28's
test, at least 35 s before SIGKILL, holds. `cargo mutants --in-diff`: five mutants, two caught,
two unviable, one timeout, detected: `* 2` for `/ 2`, which brings the pause to zero and the
fake clock to a stop. It mutates neither the clamp, the cap nor the constants, so four were
applied by hand, one at a time: the clamp removed fails the 45 s test, the cap removed and a
first pause of 100 ms fail the 355 ms test, and a `STOP_WAIT` of 5 s fails batch 28's test and
the 45 s one. Not measured live: it needs the gateway up; `up_a_proxied_request_and_down` runs
the new `stop` on a real llama-swap. The README and `docs/commands.md` say "SIGKILL after 40 s",
still true. The five checks green, 733 in libtest and `up_a_proxied_request_and_down ... ok`.

Committed as `perf(down): check a stopping gateway soon, then every 100 ms, to a deadline`. Batch 44, finding 88: `build --ref <tag>` without `--force`, when the
tag is a release by the recipe's pattern, checks whether that id is installed, or has an
unreadable directory in the way, before the gate and the fetch, and refuses with the same
messages; the checks, moved into `Builder::taken`, run again after the checkout as before, for
every other reference. The fix said before the fetch; the gate comes before it too, since the
answer needs no GPU and no toolkit (Problems And Deviations). Test-first: a named installed
release, with a gate that would refuse, and an unreadable directory of the release's name,
failed on their assertions, the gate's refusal and the fetch made, then passed. Two tests guard
what stays: a short commit naming an installed build goes through the fetch and the checkout,
and `--force` rebuilds a named release; and a commit built under its short id before its
release was tagged, `--ref 79e2e74eb1`, builds under the tag, the fake's checkout accepting that
id. `cargo mutants --in-diff`: 11 mutants, eight caught, three unviable. It does not mutate the
release guard, so `true` in its place was applied by hand: the last test fails, and only it.
Measured here with the debug binary: `build llama.cpp --ref b11700`, the current build, refused
in 0.00 s, three runs, against 1.46 s in phase 3, with no fetch. `docs/commands.md` says when
the refusal comes. The five checks green, 737 in libtest and `up_a_proxied_request_and_down ...
ok`.

Committed as `perf(build): refuse an installed release before the gate and the fetch`; batches 42 to 44 close performance. Batch 45, finding 64, by the fix's
second form: `files::lock` writes into `.lock`, once it holds it, what the holder is doing,
truncated first, and a process refused reads it: "another ai-ctl is <what the holder wrote>
(<path> is locked)", or "another ai-ctl holds <path>" when the file is empty, a holder that has
not written yet or an ai-ctl from before this batch. The first form, the activity passed to
`GatewayProcess::lock`, could not work: the refused process knows only its own activity, not
the holder's (Problems And Deviations). `GatewayProcess::lock` takes `doing`, and each holder
says its own: `up` "starting the gateway", `down` "stopping the gateway", `status` "removing the
record of a gateway that stopped", `build --force` "replacing an installed build", `build prune`
"removing builds". The builds' and the releases' locks pass their fixed texts as before, true of
each of their holders. `FakeGateway` records each `doing` in `locks`, and its `locked` now
refuses as a holder that stops the gateway, a text a holder writes, in place of the old fixed
one: six assertions follow. Test-first: with the port's `doing` passed the old text everywhere,
the four tests, the files' and the three of each holder's text, failed on their assertions,
then passed. `cargo mutants --in-diff`: 11 mutants, four caught, seven unviable. It does not
remove the truncation, so the files' test was given a previous holder with a longer text, and
the truncation removed by hand fails it with "removing buildscord of a gateway that stopped".
Probed with the debug binary on a scratch state: a `flock` holding the lock with "removing
builds" in its file makes `down` say so, an empty file "another ai-ctl holds …", and once free,
"No gateway running", leaving "stopping the gateway" in the file. `docs/commands.md`'s
Concurrency and `docs/security.md` §8 say what the file holds. The five checks green, 741 in
libtest and `up_a_proxied_request_and_down ... ok`.

Committed as `fix(lock): a refused command says what the holder of its lock is doing`. Batch 46, findings 55, 70 and 86: `Prepared` carries what the
configuration was made of, the `Inventory`, the current `Build` of each project a model runs,
and the `Gpu` the models were sized to, and drops `PartialEq` and `Eq`, which nothing compared.
Batch 15's `resolve()` returns a `Resolved` holding the inventory, the runtimes and the builds,
each project resolved once through the map's entry, and `size`, now a free function, moves them
into `Prepared`. `Bench::setup` calls `prepare()` once and takes from it the model and its
command, paired by the domain's rule of one entry per model in the inventory's order, the
backend and the build; `machine` takes the prepared `Gpu`. The second load and its two errors,
"the inventory is gone" and "the inventory changed during the run", the second `current`, the
second GPU read and M30's "cannot record the run's GPU" go: `grep` finds none in `src`, `tests`
and the documents. A `build use` between the comparison and the record can no longer make the
run record another build than the one it compared. The fix had Bench read the GPU first and call
`prepare_on`; it calls `prepare`, which keeps batch 15's order (Problems And Deviations). The
build's lookup in `Prepared` keeps a context, "no build of <project> was resolved", for an
invariant: the configuration resolved the build of every backend a model uses. Bench's test world
takes a `ScriptedGpu`. Test-first: with `Prepared`'s builds left empty, the preparation's test
failed on its assertion, and `a_run_reads_the_gpu_once`, a GPU lost after the first reading,
failed on its count, 2 against 1, then both passed. `cargo mutants --in-diff`: eight mutants, one
caught, seven unviable, each a body replaced by a `Default` the types do not have. The five
checks green, 743 in libtest and `up_a_proxied_request_and_down ... ok`.

Committed as `refactor(bench): set up a run on one preparation of the configuration`. Batch 47, finding 73: the five switches take their methods' names,
`is_installed_fails`, `log_tail_fails`, `in_container_fails`, `healthy_fails` and
`image_exists_fails`, 19 places in `fakes.rs`, `doctor.rs` and `lifecycle.rs`, renamed on word
boundaries; each sits in its own method. The two `bad_link` keep their name, with a comment: a
state that `set_current` clears, not a `current_fails`. `docs/architecture.md`'s convention is
narrowed to fit: a field named `<method>_fails`, a `bool` or what picks the call that fails,
such as batch 17's id whose removal fails, which settles that deviation; a state a later call
changes keeps a name of its own. Renames and comments in test code: no new test, and `cargo
mutants --in-diff` found nothing to mutate. The five checks green, 743 in libtest and
`up_a_proxied_request_and_down ... ok`.

Committed as `test: name the failure switches after their methods`. Batch 48, finding 74, under the verdict that narrows the document and
leaves the adapters: `docs/architecture.md`'s convention now names the adapters that hold
their program, `Cmake` and `Podman`, and `SystemHost`, which holds the search path it finds
`nvcc`, `g++`, `rpm` and `busctl` through, and the route of the others, read in their tests:
`git` on a scratch repository, `curl` on local files through its `file` protocol, `sha256sum`
and `tar` on an archive of the test's, `kill` on a child of the test; `dnf`, `nvidia-ctk` and
the tools' `--version` from `PATH`, with their parsers on captures. A failure the real program
cannot be made to give, finding 41's refused `kill`, goes through the use case's double. The
document alone: no code, no test, no mutant. The five checks green, 743 in libtest and
`up_a_proxied_request_and_down ... ok`.

Committed as `docs(architecture): name the adapters that hold their program`; batches 45 to 48 close structure. Batch 49, finding 52: the verifier's
`ref.patch` of pass 4, its hunks for this finding taken alone, with the import line of
`gateway_process.rs` updated for batch 16's `replace_file`: `files::read_if_present` beside
`read_link_if_present`, and the six readers through it, `KeyFile::read`, `LocalGateway::record`
and `config`, `Procfs::stat` and `in_container`, `InventoryFile::load`; `api_key.rs`'s imports
narrow to `File` and `Read`. The patch went in before the helper's test, so the test was
watched failing afterwards: `a_file_is_read_and_a_missing_one_is_none` failed on a body that
reads a directory as none and on one that reads a file as none, then passed. `cargo mutants
--in-diff`: 20 mutants, 16 caught, three unviable, one missed and left: `in_container` replaced
by `Ok(false)`, which only a process in a container tells apart. No test runs one, which needs
podman and an image; `names_a_container` is tested on the libpod capture, and the use cases'
container case on `FakeProcesses`. The mutant was there before this batch, on lines it moved.
The five checks green, 744 in libtest and `up_a_proxied_request_and_down ... ok`.

Committed as `refactor(files): read an optional file through one helper`. Batch 50, finding 54: `build::current_or_none` takes the link's read,
`Result<Option<T>>`, in place of the store and the project, and is `pub(super)`; its three
callers in `build.rs` pass `store.current(project)`, and `Installer::install` calls it on
`self.store.current()` in place of its own `unwrap_or_else`. The fix named it `lenient`; it
keeps its name, which says at each call what it reads. The four warnings' texts are unchanged,
each pinned by its test (`build.rs:1522`, `:1543`, `:1554`, `gateway.rs:373`), which are the
net: no new test, nothing observable changes. `cargo mutants --in-diff`: eight mutants, three
caught, five unviable. The five checks green, 744 in libtest and
`up_a_proxied_request_and_down ... ok`.

Committed as `refactor: read a build's and a release's current link by one rule`. Batch 51, finding 56: `find_nvcc` is `self.find("nvcc")`, else the
fallback when it is a file, as the verifier's `ref.patch` wrote it. `cargo mutants --in-diff`
caught its two mutants, but the fallback removed by hand failed no test: it was an absolute
path, `/usr/local/cuda/bin/nvcc`, which no test could point elsewhere, before this batch as
after. `NVCC_FALLBACK` is now `cuda/bin/nvcc` under batch 40's `toolkits` field, `/usr/local`
in production, the same path. Test-first for that part:
`nvcc_off_the_path_is_found_under_the_toolkits_cuda_link`, a search path without `nvcc` and a
scratch `cuda -> cuda-13.2`, failed on its assertion, the old fallback finding this machine's
13.4.92 against the scratch's 13.2.86, then passed; the fallback removed fails it alone. The
release binary's `doctor` with `PATH=/usr/bin:/bin`: `ok CUDA toolkit 13.4.92
(/usr/local/cuda-13.4/bin/nvcc)`, through the fallback. The five checks green, 745 in libtest
and `up_a_proxied_request_and_down ... ok`.

Committed as `refactor(doctor): find nvcc through the host's search, then under the toolkits`. Batch 52, finding 57: doctor's Idle VRAM check gives the memory of the
processes the Gateway section reports as `lifecycle::mib(accounted)`, through the module's path
beside doctor's own `mib(bytes)`, and its `gateway` local goes. The text is unchanged, pinned
by two tests (`doctor.rs:1799`, `:1838`), the net: no new test. `cargo mutants --in-diff`: two
mutants, one caught, one unviable. The five checks green, 745 in libtest and
`up_a_proxied_request_and_down ... ok`.

Committed as `refactor(doctor): sum the gateway's VRAM through lifecycle::mib`. Batch 53, finding 58: `command::script(path, body)`, under
`#[cfg(test)]` beside `exclusive`, writes the executable `#!/bin/sh` script under the guard
itself. `cmake.rs`'s and `host.rs`'s copies go, `gateway_process.rs`'s three callers write
`llama-swap` in the test's directory through it, and podman's helper passes its arguments'
check as the body's first line. The archived scripts of `gateway_store.rs` and `build_store.rs`
stay, never run. Three imports the move left unused went. `docs/architecture.md`'s convention
names the helper. Test code alone: the tests that run the scripts are the net, and `cargo
mutants --in-diff` found nothing to mutate. The five checks green, 745 in libtest and
`up_a_proxied_request_and_down ... ok`.

Committed as `test: write an executable script through one helper`. Batch 54, finding 59: the four `proc-*` captures of `gateway_process`'s
tests are `include_str!` constants, as podman's are, and its `fixture` reader goes; `host.rs`
keeps its own, with its many uses. Test code alone, the two tests reading the same bytes: no
new test, nothing to mutate. The five checks green, 745 in libtest and
`up_a_proxied_request_and_down ... ok`.

Committed as `test(gateway): embed the /proc captures as the other modules do`. Batch 55, finding 60: the verifier's corrected `f60.patch`, applied as
it was: the key's, the build store's and the release store's tests read a mode through
`files::mode`, and `api_key.rs`'s tests drop their `PermissionsExt` import; `grep -rn
'permissions().mode()' src` finds `files::mode` alone. Test code alone: nothing to mutate. The
five checks green, 745 in libtest and `up_a_proxied_request_and_down ... ok`; batches 49 to 55,
refactoring, are done.

Committed as `test: read a file's mode through files::mode`. Batch 56, findings 65 to 68, in `docs/commands.md` and the README, each
statement read against the code first. 65: `gateway config`, `up` and `bench run` fail when the
GPU cannot be read, since the models are sized to its memory; after batch 46, bench reads it
once, in the preparation. 66: each request measured is printed as it is measured, but the
overhead's, every ten requests each way (`render_event`'s `request % 10`), and the request that
sets the context aside not at all. 67: a failed compile's error quotes the last lines of the
build log; the other steps' errors give their own cause. 68: a row for `work/<project>/`, the
CMake tree, removed after a build that installs (`clean`) and kept after one that fails until
the next build, which removes it first; no size given; the README's tree lists it too. Two
README paragraphs were rewrapped. `check_links.py` on both files: 20 links, none broken. The
documents alone: no test, no mutant. The five checks green, 745 in libtest and
`up_a_proxied_request_and_down ... ok`.

Committed as `docs: say what the commands do where the code does otherwise`. Batch 57, findings 72 and 75, in `docs/architecture.md`. 72 measured
again first, since batch 15 changed what `gateway config` reads: `tests/cli.rs`'s `ai_ctl()`
given `LD_DEBUG=libs` and an output file per command, then put back with `git checkout`; `cargo
test --test cli`, 27 passed, and `libnvidia-ml` was loaded by `doctor --json` and the two
`status` runs, batch 32's included, and no longer by `gateway config`, which fails on the
inventory or the build before it reads the GPU. The Testing table now says that no unit test
calls NVML, and that the binary's tests of `doctor` and `status` load it and assert nothing
that depends on it; `tests/gateway_lifecycle.rs` reads a fake GPU. 75: the Ports section's
example goes; every port so far exists for a test double, one implementation each in
`infrastructure`, counted again over the 22 traits of `ports.rs`, and a choice between two
kinds, such as a backend's source, is an enum of the domain. The document alone: no test, no
mutant. The five checks green, 745 in libtest and `up_a_proxied_request_and_down ... ok`.

Committed as `docs(architecture): which tests load NVML, and why every port exists`. Batch 58, findings 76, 77 and 89, three doc comments, each read against
the code. 76: `Services::configurator` is shared by `up`, `down`, `status`, `logs`, `bench run`
and `gateway config`, the three presentation modules that call it, as `docs/architecture.md`
lists them. 77: the domain's `gateway` module says the order `expected_sha256` follows, the
digest given with `--sha256`, else the pinned release's own, and any other release's published
checksums. 89: NVML's comment names the quantity and its order of magnitude, the first read,
which loads the library, initialises NVML and attaches the device, tens of milliseconds, a later
one about a tenth of one; the measured values stay in the review, pass 5's verifier's
`Nvml::init` 7.0 to 8.6 ms and first read 23.0 to 27.6 ms. Comments alone: no test, no mutant.
The five checks green, 745 in libtest and `up_a_proxied_request_and_down ... ok`.

Committed as `docs: three doc comments said less or other than the code`. Batch 59, finding 78, written with the roadmap's state as it lands: the
README's Status says that security, bugs, tests, performance and structure are done, and the
fixes the closing review asked for, and that a security pass over those fixes comes last. Its
link still names `on-progress/`; the roadmap's closure moves the folder and must update it.
`check_links.py`: the README's links resolve. The five checks green, 745 in libtest and
`up_a_proxied_request_and_down ... ok`. The 58 batches kept are fixed; batches 56 to 59 close
the documents.

Committed as `docs(readme): the status as the fix roadmap stands`. Session closed at the user's request after the last batch. This twelfth
session fixed batches 42 to 59, one commit each, `perf(bench): count the agent's bare prompt once in a session` to `docs(readme): the status as the fix roadmap stands`: performance (42 to
44), structure (45 to 48), refactoring (49 to 55) and the documents (56 to 59). The phase
stands at 60 of 61 tasks; only the security pass is left. The session's choices that differ
from the Fix Plan's wording are under Problems And Deviations, batches 44, 45 and 46; batch 51
also moved the `nvcc` fallback under `toolkits` to test it, and batch 57 measured that
`gateway config` no longer loads NVML. One missed mutant is left, justified under batch 49:
`in_container` as `Ok(false)`. Put to the user, with no answer yet: `review-security` at `high`
on the fixes' diff from `docs(v2-review-fixes): record the closing review's verdicts and the fix scope` of 2026-10-18, 45 files and about 5,100 changed lines, about $3 to $5, then
its verifier, about $2, after a second yes.

**Where the next session starts:**
1. Put the security pass to the user again, in one line: `review-security` at `high` on the
   fixes' diff, `git diff` from `docs(v2-review-fixes): record the closing review's verdicts and the fix scope` of 2026-10-18 to `HEAD`, about $3 to $5. On the yes, load
   `reviewing-delivered-code`, run it on that diff, then, after a second yes, one
   `review-verifier` on its findings, about $2, one agent after the other.
2. Fix what the verifier confirms, one batch per commit by the same method; then close the
   phase and the roadmap with the roadmap skill. At the roadmap's closure, the README's Status
   link to `on-progress/v2-review-fixes/` moves with the folder.
- Open with the user: the go-ahead for the security pass.
- `tick.py` and `checks.sh` are in this session's scratchpad, on a
  tmpfs a reboot clears; otherwise write them again from the tenth session's description.
- Text holding `\u2028`, `\u2029` or `\u{FFFD}` is written through Python, never the Edit tool.
- Nothing uncommitted after this entry's commit.

Resumed in a thirteenth session. Before the security pass, the user asked to study an outside
security audit skill, read in its local clone
at its commit `c1c8a8c`, and to compare it with `review-security` and `review-verifier`. What
it offers, a gate on each candidate that names the attacker's starting capability, severity
anchors, a `needs_validation` verdict with an exact blocker, hardening apart from findings,
rejected findings kept under a stable fingerprint, a structured answer checked by a schema, a
budget counted in agents, went to a note on that skill's lessons in `docs/notes/`, which
`docs/notes/review-workflow-tooling.md` now names. The user chose to run the pass with the skill
as it stands (under Decisions). `scope.py range` from `docs(v2-review-fixes): record the closing review's verdicts and the fix scope` of 2026-10-18 to `HEAD`: 54 files, 36 of them production,
+5,385 and -1,005 lines; 14,261 production lines for the reviewer to read.

On the user's yes, the pass started under `reviewing-delivered-code`, its report
`docs/reviews/delivered-code/2026-10-19-v2-review-fixes-security-pass.md`, scope from `docs(v2-review-fixes): record the closing review's verdicts and the fix scope` of 2026-10-18 to `HEAD`.
The five checks green, 745 tests and `up_a_proxied_request_and_down ... ok`; `cargo audit` clean on
66 crates; the diff's 284 mutants, under the skill's limit of 300, run in the background. The
caller's prompt told `review-security` to work from its scratch folder and check `git status`
before answering, and to read `docs/security.md` §1 and §2 before setting a severity, the two
problems the closing review had with it. It answered two low findings, both on fixes that do not
fully hold: finding 3's listener check misses llama-swap's dual-stack socket on `0.0.0.0`, so an
exposed gateway on `0.0.0.0` never starts, reproduced in a network namespace; finding 5's escape
lets Unicode bidi and format characters through. It found that findings 4 and 7 hold, and the
documents of 1 and 2. It left a lead of the bugs axis on `put_back_replaced`. The repository was
clean after it.

The pass's report holds the two findings and the lead under To Verify, as the reviewer gave
them: 1, the `0.0.0.0` listener; 2, the bidi and format characters; 3, the lead on
`put_back_replaced`, which the reviewer read and did not reproduce. Its `Status` names the
verification as next. The verifier was put to the user, one `review-verifier` at `high` on 1, 2
and 3, about $2; the user closed the session instead, asking that the two findings be recorded
and dealt with on resuming. The diff's mutants were still running at the close: 230 of 284 done,
165 caught, 59 unviable, 4 timeouts, 2 missed, `src/application/doctor.rs:428`, `>` for `>=` in
`Doctor::inventory_checks`, and `in_container` as `Ok(false)`, the one batch 49 justified.

**Where the next session starts:**
1. The user: "enregistre bien les deux findings trouvé par review-security, on les traite à la
   reprise". Findings 1 and 2 of `docs/reviews/delivered-code/2026-10-19-v2-review-fixes-security-pass.md`,
   with lead 3, are under its To Verify. Load `reviewing-delivered-code`, continue the review
   from its `Status` at step 7, and put the verifier to the user again: one `review-verifier` at
   `high` on 1, 2 and 3, about $2.
2. Then the verdicts, the session giving the clear ones; the fix plan; the fixes, one batch per
   commit, test-first, with `cargo mutants --in-diff`; then the phase's last task ticked, and the
   phase and the roadmap closed with the roadmap skill, the README's Status link moving with the
   folder.
- Open with the user: the verifier's go-ahead.
- The review's scratch folder is on a tmpfs a reboot clears. It holds
  the reviewer's probes, `review-security/copy` and the network-namespace run of finding 1, and
  the mutants' output, `mutants.out/`; copy its `missed.txt` into the review report's Checks,
  whose row still says running, or run the 284 mutants again, about 13 minutes. Step 10 removes
  the folder at the review's end.
- Nothing uncommitted after this entry's commit.

Resumed in a fourteenth session. Read before working: this phase's file and report, and the
security pass's report. The machine had not rebooted since 08:43, so the scratch folder
was intact, and `git diff` from the pass's end on `src`, `tests`, `Cargo.toml`, `Cargo.lock` and
`deploy` was empty. The pass's mutants had been recorded by `docs(v2-review-fixes): the security pass's mutants, finished after the close`: the three missed, each
justified already, `doctor.rs:428`'s `>` for `>=` by batch 12, `in_container` by batch 49 and
`read_gpu` by batch 5. The user said yes ("oui") to one `review-verifier` at `high` on findings
1 and 2 and lead 3, its folder `<scratch>/verifier-security`, the three blocks given there as
`findings.md` and `scope.py`'s output pasted in its prompt.

It answered in its format, 43 tool calls in about 8 minutes: the three `CONFIRMED`, each low.
Finding 1 reproduced apart in a network namespace; it holds a second case of the same cause, a
v4-mapped address bound by Go as AF_INET and looked up in `tcp6`, merged into it; and `[::]`
opens nothing more than `0.0.0.0`, which already answers on `[::1]`. Finding 2 holds on the
code; the display escape gets a wider predicate of its own, while `Text::checked` and
`toml_escape` keep their rule, since llama-swap v263 loads a configuration holding those
characters and runs the command with their exact bytes. Lead 3 is finding 3, bugs: `publish`
removes the old build while hidden, a kill leaves it half deleted, and `put_back_replaced`
brings it back after `prune --keep 0` or a manual removal of the rebuilt one; the fix puts back
only beside its staging directory. Two leads remain, findings 4 and 5 under To Verify: `up`'s
message at the start wait's deadline drops "another program answers", and `docs/security.md`
§4 does not say that `0.0.0.0` also serves IPv6. The verifier saw `docs/notes/review-costs-through-usage-py.md`
appear untracked during its run, written by the agent setup's roadmap `cost-method`; committed
apart at the user's word. Its captures, probes and the diff of its copy were copied to
`~/.local/share/ai-ctl/review-scratch/2026-10-19-v2-review-fixes-security-pass/verifier/`, as
the closing review's were.

Committed as `docs(v2-review-fixes): the security pass's findings 1 to 3 verified, two leads to verify`. The user said yes ("oui") to one `review-verifier` at `high` on
findings 4 and 5, about $1. It answered in its format, 33 tool calls in about 7 minutes: both
`CONFIRMED`, low. Finding 4 stands apart from finding 1, its cause being the message: with a
correct finding 1, every bind conflict it tried made llama-swap exit, the branch that names
another program, so the clause shared at the deadline lands after batch S1. It corrected finding
1's fix: `[::ffff:0.0.0.0]` is one dual-stack socket in `/proc/net/tcp6`, so the mapping
canonicalises first, which a trial in its copy showed for six addresses, the suite green. Finding
5 is more than a gap of the documents, §1's row saying "The LAN" on the decision `expose` asks
for; this machine has a global IPv6 address. Its two leads, left while verifying leads, were not
given to a verifier: `is_free` binding AF_INET only for `0.0.0.0`, reproduced, went into finding
1, the same cause, and podman publishing on `*:port` went into finding 5, whose fix names it.

Step 9 finished the security pass's report: the five findings ordered security and bugs first,
then error handling, its Coverage with each axis's row and the files no agent opened, and a fix
plan of five batches, S1 to S5 in this phase, the order being 1, 4, 5, 2, 3. Under the user's
delegation of 2026-10-18, the session confirmed the five, none being a hard case, and the
report's `Status` is complete. The phase's security task is split into the review, ticked, and
the five batches: 66 tasks, the README's totals recomputed. Before step 10, the probes, captures
and copies' diffs of the reviewer and both verifiers, and the pass's missed mutants, were kept in
`~/.local/share/ai-ctl/review-scratch/2026-10-19-v2-review-fixes-security-pass/`, 456 KB; the
scratch folder is removed, and `pgrep` shows nothing left.

Committed as `docs(v2-review-fixes): the security pass verified and planned, five batches to fix`. Batch S1, finding 1: `go_listener` maps `listen` to the sockets Go opens
for it, the address canonical and, for an unspecified IPv4 address, the dual-stack `[::]` as
well. `listens` reads the table of each, `tcp6` read only when present, a kernel without IPv6
having none; `is_free` binds each, and a dual-stack bind that fails otherwise than on a taken
port is that kernel, where Go listens on IPv4 alone. The parser is unchanged: it matched `[::]`
already, and its test now reads the first verifier's capture of llama-swap v263 on `0.0.0.0`,
`tests/fixtures/proc-net-tcp6-dual-stack.txt`, listed in `tests/fixtures/README.md` with the
namespace's uid of 0. Test-first: the mapping, `[::ffff:0.0.0.0]` and `[::ffff:127.0.0.1]` among
its six cases, a dual-stack listener of the test's own on `[::]:0` that `listens` must see for
`0.0.0.0`, and a `[::1]` listener that must make `0.0.0.0`'s port taken and leave `127.0.0.1`'s
free, failed on their assertions against a stub, then passed. `cargo mutants --in-diff`: 13
mutants, one missed, the bind's `AddrInUse` guard set to `true`, older than this batch, which
no test made fail otherwise; `an_address_of_no_interface_is_an_error_not_a_taken_port`, on
TEST-NET-1's `192.0.2.1`, kills it, the mutant applied by hand failing it alone; run again, 11
caught, 2 unviable. `docs/security.md` §3 names `tcp6` and the dual-stack socket. The five checks
green, 719 in libtest and `up_a_proxied_request_and_down ... ok`.

Committed as `fix(up): the listener check and the free-port check follow Go's dual-stack socket`. Batch S2, finding 4: `another_answers`, a function of `another`, gives
the clause ", where another program answers: …" with batch 2's advice on the key, and both of
`up`'s failed starts add it: llama-swap's exit, as before, and now the start wait's deadline,
after the last error. Test-first: `a_start_timeout_names_another_program_that_answers`, the
second verifier's, `held_by_another` and a gateway that never exits, failed on its assertion,
the message without the clause, then passed. `cargo mutants --in-diff`: three mutants, all
caught. `docs/commands.md` quotes neither message. The five checks green, 720 in libtest and
`up_a_proxied_request_and_down ... ok`.

Committed as `fix(up): a start that times out names another program that answered`. Batch S3, finding 5, in `docs/security.md`: §1's row names the network,
the LAN and over IPv6 whoever routes to the machine's global address, what `expose` opens on
`listen`'s addresses, and the firewall; §4 says that `0.0.0.0` and `[::]` open the same
dual-stack socket, every IPv4 and IPv6 address, global ones included, with the second
verifier's probe, that this machine has a global IPv6 address, and that a firewalld port opens
both families; §6 says podman publishes `-p 18000:8080` on `*:18000`; its Sources name the
security pass. `grep` finds no other `0.0.0.0` in `docs/stack.md`, `docs/commands.md`, the README,
`docs/architecture.md` or `deploy/`. `check_links.py`: 10 links, none broken. The document
alone: no test, no mutant. The five checks green, 720 in libtest and
`up_a_proxied_request_and_down ... ok`.

Committed as `docs(security): what 0.0.0.0 opens, over IPv6 too`. Batch S4, finding 2: `domain::text` gains `is_invisible_format`, a
fixed list, since the standard library has no lookup of Unicode's categories: U+061C, U+200B
to U+200F, U+202A to U+202E, U+2060 to U+2064, U+2066 to U+2069 and U+FEFF. `escape_controls`,
whose one caller is NVML's `process_name`, escapes them beside the controls and line
separators; `is_control_or_separator`, the rule `Text::checked` and `toml_escape` share, is
unchanged, and its test still asserts that U+202A passes it. Test-first: a process name holding
U+202E, U+2066 and U+FEFF came back unescaped against the stub, then was escaped; the test pins
each range's bounds and the characters beside them. The test's characters were written as Rust
escapes through Python, and `grep -cP` finds none raw in `src/domain/text.rs` or
`src/infrastructure/nvml.rs`, whose comment now names the format characters. `cargo mutants
--in-diff`: five mutants, all caught. The five checks green, 721 in libtest and
`up_a_proxied_request_and_down ... ok`.

Committed as `fix(nvml): GPU process names shown with Unicode's format characters escaped`. Batch S5, finding 3: `put_back_replaced` renames a
`.replaced-<id>-<pid>` back to `<id>` only when `.staging-<id>-<pid>`, the same PID's, is still
there, the mark of a kill between `publish`'s two renames; once the replacement is in place,
the old build is a leftover, maybe half removed, and the sweep that follows removes it, even
after the new one was removed. The second part of the fix, `publish` removing the old build
through `remove`'s rename to `.removing-…`, was not done (Problems And Deviations). Test-first:
`a_build_whose_replacement_was_installed_is_not_put_back`, the state the second verifier built,
the old b1 aside and half removed beside the new one, the new one removed as `prune --keep 0`
does, then a build of b2, gave `b1` and `b2` against `b2` alone, then passed;
`a_build_an_interrupted_replacement_left_hidden_is_put_back`, which creates the staging
directory, still puts b1 back. `cargo mutants --in-diff`: two mutants, both caught. The five
checks green, 722 in libtest and `up_a_proxied_request_and_down ... ok`. The five batches of the
security pass are fixed.

Committed as `fix(build): a replaced build put back only before its replacement is installed`. Every task of the phase is ticked, 66 of 66, and the README's line
reads 100% while the phase is still 🟡: its closure sets 🟢. The session put the two closures to
the user, the phase's and the roadmap's, with a `roadmap-auditor` each, about $1 an audit, and
recommended the phase's now and the roadmap's in a fresh session, since `summary.md` is written
from the six phase reports read in full. The user chose both in a fresh session: "oui on
cloture dans une session neuve". The tick and checks scripts are in this session's scratchpad, on a tmpfs
a reboot clears; the closures need neither.

**Where the next session starts:**
1. Oui on clôture dans une session neuve: close phase 5 with the roadmap skill's
   `close-phase.md`, its Start Commit `docs(v2-review-fixes): close phase 4, structure and refactoring` of 2026-10-17: the five checks, the acceptance criteria
   ticked, `**Completed:**`, the Assessment, `## Changes To Later Phases` (none: no phase
   follows), the README's block from `progress.py`, its version, changelog and fields, Files
   Changed from `git diff -M --name-status` from the Start Commit, the Final Verification, a
   `roadmap-auditor`, then one commit.
2. No phase follows, so close the roadmap with `close-roadmap.md`: `summary.md` from the six
   phase reports read in full, with what is left open (the review skill's defects under phase
   5's Problems And Deviations, `docs/notes/review-workflow-tooling.md`,
   the security skill's lessons and `review-costs-through-usage-py.md`, and the missed
   mutant `in_container` as `Ok(false)` that batch 49 justified); the README; the folder moved
   to `completed/` with `git mv`; every reference to `on-progress/v2-review-fixes` repaired, the
   repository's README Status link and `CLAUDE.md`'s "Open:" line among them; a second
   `roadmap-auditor`; one commit.
- Before each audit, the user's go-ahead in one line, about $1.
- Nothing uncommitted after this entry's commit.

Resumed in a fifteenth session, to close the phase and the roadmap: "on clôture la phase 5 et la
roadmap". Read before working: this phase's file and report in full, the five other phase
reports, and the roadmap's README. The five checks green: 722 in the library, 27 in
`tests/cli.rs`, 3 in `tests/gateway_config.rs`, `up_a_proxied_request_and_down ... ok`, and the
skill's two evals.

Beside the closures, the user asked for a full report on the review workflow, its agents and
its skills, as the first one of 2026-10-17 was: what works and is efficient, what can be
improved or removed, what is missing or handled badly, and the loop already seen, "review de
code -> correction -> review de code après correction -> nouvelle correction -> ...". It is
`docs/notes/review-workflow-second-assessment.md`, which `review-workflow-tooling.md` now
names. For it, the origin of each of the closing review's 84 findings was measured by
`git blame` at `docs(v2-review-fixes): record the closing review's scope and tier` of 2026-10-17, on the lines its Where field cites, against phase 0's start commit
`docs(v2-review-fixes): create the roadmap` of 2026-10-16, by a script in the session's scratchpad: 32 cite mostly lines the roadmap wrote, 52
mostly older lines, 20 of these test gaps. The fixes measured +7,034 −1,993 lines in `src/` and
`tests/` for phases 0 to 4, +4,213 −888 for the 58 batches, and +231 −31 for S1 to S5. While the
note was written, the Write tool decoded the line separator's escape into the character itself,
as the Edit tool did in batch 5; rewritten through Python, and
`grep -cP '[\x{2028}\x{2029}\x{FFFD}]'` finds none in it.

The Final Verification's `cargo test` failed once, on
`a_port_taken_on_ipv6_is_not_free_for_the_wildcard_ipv4_address`, batch S1's test, which had
passed at the session's start. Put to the user, who chose to fix it before the closure.
Measured: it failed in 6 of 150 runs of `gateway_process`'s tests, and in none of 150 alone or
beside any other module; at a failure, `ss` showed only the test's own `[::1]` listener on the
port. The cause: a child forked by another test of the module while `is_free` held its socket on
`0.0.0.0` kept that socket until it ran its program, and the next bind of `127.0.0.1` on the
port was refused, the hazard `command::exclusive()` exists for. A second cause, rarer: the
kernel may give `[::1]` a port that a client of the machine holds on `127.0.0.1`, 3 binds in
20,000 without load, and a forced collision failed the test's assertion. The test now holds
`command::exclusive()` and takes a `[::1]` port that `127.0.0.1` leaves free: none failed in
400 runs of the module and 30 of the suite, and `is_free`'s 7 mutants are caught. Committed
apart as `fix(test): the dual-stack port test holds the spawn guard`.

---

## Decisions

- **The closing review runs at `high`, in four passes.** `high` is the first review's tier,
  so the two reviews compare; approved by the user on 2026-10-17 at about $55. The passes:
  1. `review-security` and `review-correctness`, with a verifier each;
  2. `review-tests`, with its verifier;
  3. `review-cleanup` and `review-structure`, with a verifier each;
  4. `review-performance`, with its verifier, then the report's finish.

  The verification of passes 1 and 3 is priced by its findings at step 7, $0.13 a finding;
  past about 31 findings it would take the pass over $15, and moves to the next pass.
- **Pass 1's verification is a pass of its own: the review runs in five passes.** The user
  decided this on 2026-10-17, once the two reviewers had cost $18.34 against $7.50 at the rate,
  by `usage.py`'s figure, which overstated them: recomputed, the two cost about $10.69
  together (under Problems And Deviations).
  The verification of pass 1's 20 findings is now pass 2. The reviewers of the former passes 2
  to 4 move to passes 3 to 5, unchanged, as the review's Tier table shows.
- **The CUDA context that finding 1's measurement needs is allowed, against the review's
  rules.** The user, on 2026-10-17, in answer to the question that pass 2 left on finding 1:
  "j'autorise à créer un context CUDA contre les règles de la review". It covers the
  measurement that settles finding 1, llama-server's time to fail to bind a taken port with
  the GPU visible, natively and as the image; the review's other rules stand. Finding 1 takes
  its verdict from that measurement.
- **Pass 4's reviewers run one after the other, with the user's go-ahead between them.** The
  user, on 2026-10-18, in the yes to the wave: "on lance l'un après l'autre avec validation
  entre les deux, pas en parallèle". `review-cleanup` runs first, then `review-structure` after
  a second yes; each answer is written under To Verify before the next starts.
- **The session gives the verdicts, the user the hard cases.** The user, on 2026-10-18: "Serait
  t'il possible de te laisser choisir les confirmation et que tu ne me demande que pour les cas
  les plus durs à trancer ?". The session confirmed 83 findings and rejected 84, the threads
  for doctor's probes: about 75 ms on an interactive diagnostic does not pay the `Sync` bounds
  and the 24 fake fields rewritten. Its choices: batch 2, the documents without a rotation at
  each `up`; 24, no `GatewayState` supertrait; 26, `// LEGACY:` comments without a migration;
  48, `docs/architecture.md` narrowed rather than nine adapters rewritten. The user may
  overturn any of them.
- **Finding 1's transport waits for a probe.** The user's choice: batch 1 starts with a probe of
  llama-swap v263 on a unix socket, whose result and impact (the generated configuration, the
  bench's overhead scenario, which calls the backend directly, decision 0001) go to the user
  before the transport changes; `docs/security.md` §1 and §2 are written either way.
- **Finding 18 is kept and documented.** The user's choice: no `libc` and no unsafe code for a
  window of microseconds whose outcome the next `up` shows; finding 24's test stays.
- **This roadmap fixes the whole plan, then closes on a security pass.** The user's choice of
  2026-10-18: the 58 batches kept are this phase's tasks, none left open. A full review after
  each fix roadmap would be a loop with no end: "faire une review complète a la fin de chaque
  roadmap de correction reviens à une boucle sans fin"; the security pass stays: "garder une
  passe sécurisé me semble primordiable". The pass runs `review-security` and its verifier on
  the fixes' diff, after the user's go-ahead.
- **The security pass runs `review-security` as the skill stands; the lessons of the outside
  security skill wait for the skill's change.** The user, on 2026-10-19, between running the pass as
  the phase's Out of Scope says and changing `review-security` first, outside the roadmap:
  "A". The study is in `docs/notes/`, its note on the security skill's lessons.
- **The security pass prices its waves at the skill's rates; skill-tooling 0.2.0 waits for the
  phase's release.** The user, on 2026-10-19, on `docs/notes/review-costs-through-usage-py.md`:
  its method needs `usage.py estimate` from skill-tooling 0.2.0, not yet published, and a wave
  of about $1 stays under its threshold anyway; the note is taken up with the skill's change
  after this roadmap.
- **Each batch's diff runs `cargo mutants --in-diff` before its commit.** Agreed with the user
  with the proposals: the 28 test findings of the closing review are mutants that no run of
  `cargo mutants` saw, the roadmap's diff holding 551, over the limit of 300. A missed mutant
  is killed by a test in the batch or justified in this report.

---

## Files Changed

**Added**
- `docs/notes/backends-on-unix-sockets.md`
- `docs/notes/review-costs-through-usage-py.md`, written by the agent setup's roadmap `cost-method` during the phase and committed apart at the user's word
- `docs/notes/review-workflow-second-assessment.md`
- `docs/notes/review-workflow-tooling.md`
- `docs/notes/`, the note on the security skill's lessons
- `docs/reviews/delivered-code/2026-10-17-v2-review-fixes.md`
- `docs/reviews/delivered-code/2026-10-19-v2-review-fixes-security-pass.md`
- `docs/roadmap/on-progress/v2-review-fixes/phase-5-closing-review-report.md`
- `src/domain/text.rs`
- `tests/fixtures/llama-swap-263-upstream-props-unknown-model.http`
- `tests/fixtures/nvidia-ctk-1.20.1-cdi-spec.yaml`
- `tests/fixtures/proc-net-tcp-excerpt.txt`
- `tests/fixtures/proc-net-tcp6-dual-stack.txt`
- `tests/fixtures/proc-net-tcp6-excerpt.txt`
- `tests/fixtures/rpm-q-cuda-toolkit-packages.txt`

**Modified**
- `CLAUDE.md`
- `README.md`
- `deploy/inventory.toml`
- `docs/architecture.md`
- `docs/commands.md`
- `docs/roadmap/on-progress/v2-review-fixes/README.md`
- `docs/roadmap/on-progress/v2-review-fixes/phase-5-closing-review.md`
- `docs/security.md`
- `docs/stack.md`
- `src/application/bench.rs`
- `src/application/build.rs`
- `src/application/doctor.rs`
- `src/application/fakes.rs`
- `src/application/gateway.rs`
- `src/application/gateway_config.rs`
- `src/application/lifecycle.rs`
- `src/application/mod.rs`
- `src/application/ports.rs`
- `src/application/toolchain.rs`
- `src/domain/bench.rs`
- `src/domain/gateway.rs`
- `src/domain/gateway_config.rs`
- `src/domain/inventory.rs`
- `src/domain/machine.rs`
- `src/domain/mod.rs`
- `src/infrastructure/api_key.rs`
- `src/infrastructure/build_store.rs`
- `src/infrastructure/cmake.rs`
- `src/infrastructure/command.rs`
- `src/infrastructure/files.rs`
- `src/infrastructure/gateway_process.rs`
- `src/infrastructure/gateway_store.rs`
- `src/infrastructure/git.rs`
- `src/infrastructure/host.rs`
- `src/infrastructure/http.rs`
- `src/infrastructure/inventory_file.rs`
- `src/infrastructure/nvml.rs`
- `src/infrastructure/podman.rs`
- `src/infrastructure/run_store.rs`
- `src/infrastructure/server_api.rs`
- `src/presentation/bench.rs`
- `src/presentation/build.rs`
- `src/presentation/compare.rs`
- `src/presentation/mod.rs`
- `tests/cli.rs`
- `tests/fixtures/README.md`

**Deleted**
- `tests/fixtures/rpm-cuda-toolkit-packages.txt`

---

## Problems And Deviations

Defects that the review shows in `reviewing-delivered-code`. Per the phase's Out of Scope, they
are recorded here for a change to the skill outside this roadmap:
- **The cost figures this phase recorded for its agents were overstated two to three times by
  the measuring tool, and the overruns drawn from them were wrong.** Found on 2026-10-17, after
  pass 2. `usage.py` gives every subagent call that lacks its final usage 5,400 output tokens,
  and `usage.toml` holds no figure of its own for the review agents. Against the `cost-state`
  records of the first review's five sessions, it overstates the agents' cost 1.9 to 3.3 times;
  their true output is about 635 to 870 tokens a call. The first review had measured 845 and
  priced its later passes with it, but the figure stayed in its report. Recomputed at 845:
  - pass 1's reviewers cost about $4.37 (`review-security`) and $6.32 (`review-correctness`),
    not $7.65 and $10.69: 1.2 to 1.7 times the $3.75 of the skill's rate, not two to three
    times. They reproduced their findings in scratch: 65 and 89 tool calls.
  - pass 2's verifiers cost about $1.74 and $2.03, not $6.12 and $6.86: about 1.5 times the
    $2.50 of step 7's price together, not five times. Most of a verifier's cost is fixed: five
    findings cost $1.74, fifteen $2.03. They reproduced rather than read: 48 and 53 calls.

  The split into five passes rested partly on the overstated figure (under Decisions). The
  cost method goes to the agent setup:
  its `notes/design-feedback/2026-10-17-cost-method-written-and-reproducible.md`.
- **The list of commands a reviewer may run leaves out `bench list`.** The skill allows
  `doctor`, `status` and `build list`, yet `bench list` only reads as well. `review-correctness`
  ran it once, on a scratch XDG directory, to reproduce finding 10, and reported the breach
  itself.
- **A reviewer's answer did not always follow its format.** `review-correctness` counted 17
  findings for 16 blocks, and two of its blocks, findings 20 and 21, have no severity. Its
  verifier gives one.
- **A verifier wrote outside its scratch folder.** The security verifier left three logs in
  the repository's root, though its rules allow writes in scratch only; nothing tracked
  changed. Its prompt gave the scratch folder, but its commands ran from the repository's root
  and wrote their logs to the working directory.
- **The review's rules leave a security finding unsettled.** The measurement that decides
  finding 1 needs llama-server to start with the GPU visible, which creates a CUDA context, and
  the skill forbids one for any agent. The skill names no way to obtain such a measurement,
  such as asking the user to run it.
- **The skill has no place for a defect the session finds itself at `high`.** Finding 1's
  measurement showed lead 22, a gap of doctor's CDI check. At `high` only reviewers and
  verifiers bring findings; the session wrote it under To Verify in the reviewers' format, as
  a verifier's lead goes, so that a verifier checks it before it counts.
- **The skill's step 10 deletes what the fixes will need.** `review-tests` and its verifier
  produce tested tests, one per finding, and mutants defined as exact replacements; the skill
  keeps nothing of scratch, and the report holds only their description. The session copied
  them, at the user's word, outside the repository. The skill names no place for an agent's
  output that a later step uses again, which the direction of
  `docs/notes/review-workflow-tooling.md` asks for.
- **`review-cleanup`'s search for dead code misses an unreachable branch.** It builds the crate
  as a binary with private modules, where rustc reports unused items; a call that can never
  act, such as finding 69's `warn_unread` after a `target` that already failed on the same
  reading, passes. `review-structure` found it by reading; `review-tests` had seen it in pass 3
  and left it out as not a test gap, so no axis owned it. The agent's file names no check for
  a branch that an earlier guard makes unreachable.
- **The skill asks the user a verdict on every finding.** With 84 findings, the session began to
  put them batch by batch; the user asked to delegate the clear cases and be asked only the
  hard ones, "On aurait du faire ça dès le début". The skill names no such triage.
- **A full review at the end of a fix roadmap starts a loop.** It reports every finding, with no
  floor of effect, so each fix roadmap calls for another; the closing review took nine
  sessions. The phase's design, a full closing review, came from the skill's practice and from
  the skill's need of a second real use. The user's direction: per-batch mutants, a review of
  each phase's diff at its close, a floor, and a security pass only at the end
  (`docs/notes/review-workflow-tooling.md`).
- **The session's edit tool decodes `\u2028` and `\u2029` into the characters themselves.**
  Found in batch 5: text given to the Edit tool with that escape reached the file as the raw
  U+2028, in a test and in this report; a phase 0 line of `docs/security.md` had the same
  defect. In this repository, whose rules are about those characters, such text is written
  through Python, and `grep -rnP '[\x{2028}\x{2029}]'` checks that none is left. The
  closure's session found the Write tool doing the same, in
  `docs/notes/review-workflow-second-assessment.md`, and rewrote the line through Python.
- **`cargo mutants` ran on no diff of this roadmap.** The closing review's 551 mutants passed
  the limit of 300, so 28 test gaps were found by mutants written by hand; a batch's diff holds
  a fraction of the roadmap's mutants, though no batch's count was measured.
- **Finding 49's test asserts the behaviour that finding 13's fix gives, not the one it was
  written for.** The verifier wrote it against the code under review: `status` failing under
  the lock, with no event and the record left. Batch 18 makes `status` observe without the
  lock, so the same setup now succeeds, warns, and leaves the record; the test keeps its point,
  that the record is removed only under the lock, and kills M29's counterpart.
- **Finding 33's test is on a function that takes a writer, not on a test `Services`.** The
  Fix Plan's batch 19 named a test `Services` of the fakes, with `fakes` made `pub(crate)`,
  `docs/architecture.md:115` and the comment at `fakes.rs:1` updated; the verifier's own note
  allowed a function that takes a writer instead, which needs none of the three and keeps the
  run's rendering out of libtest's output. It kills M28 all the same.
- **Batch 17's `remove_fails` holds an id, where `docs/architecture.md`'s convention says a
  `bool`.** A test of a prune that fails midway needs a removal that succeeds before one that
  fails. Batch 47, the failure switches' names, or batch 57, the document's statements, settles
  whether the convention admits it.
- **Batch 7's check is in the domain, not in `flag_values`.** The review's fix names the
  inventory file's reader; the domain's `check_flags` already holds batch 6's rule on a flag's
  value, covers defaults and models, and reports every broken rule together, as doctor shows
  them. The run store's own refusal at `run_store.rs:539` stays, for a setup built otherwise.
- **Batch 24's `Lifecycle` builds its `Inspector` rather than holding it.** The fix has
  `Lifecycle` hold the read-only struct and add `ports`, `pause` and `now`. Without the
  `GatewayState` supertrait, which the verdict left out, that routes `up`'s and `down`'s
  `lock`, `start`, `signal` and `forget` through `self.inspector.gateway`: 36 accesses through a
  struct whose doc says it changes nothing, tried and reverted. `Lifecycle` keeps its ports, and
  a private `inspector()` builds the struct for the six calls that read, as Doctor's does. The
  finding's point holds: Doctor builds the read-only part alone, without `pause` and `now`.
- **Batch 44's refusal comes before the gate, not only before the fetch.** The fix placed it
  before `Fetching`; whether the named release is installed needs neither the GPU nor the
  toolkit, and the gate costs about 0.2 s. A build refused as installed with a broken toolchain
  now names the build first; with `--force`, the gate then refuses as before.
- **Batch 45 takes the fix's second form, the holder's activity in the lock's file.** The
  first, prune's activity passed to `GatewayProcess::lock`, would have given prune's text only
  to a prune refused by another holder: the text a refused process prints must come from the
  holder, which only a channel between the two carries.
- **Batch S5 leaves `publish`'s removal of the old build as it was.** The security pass's fix
  plan also had `publish` rename the old build to `.removing-…` before removing it, through one
  helper with `remove`. Once `put_back_replaced` asks for the staging directory, that rename
  changes nothing: `publish` removes the old build only after its second rename, when the
  staging directory is gone, so a `.replaced-…` that a kill leaves half removed is never put back,
  and the sweep removes it like a `.removing-…`.
- **Batch S1's dual-stack test was flaky, about once in 30 runs of the suite.** The closure's
  Final Verification found it. It released sockets without `command::exclusive()`, which
  `command.rs` asks of every test that releases a lock or a socket, so a child forked meanwhile
  by another test held the port; one `cargo test` per batch could not show it. Fixed at the
  user's word before the closure, as `fix(test): the dual-stack port test holds the spawn guard`.
- **Batch 46's Bench calls `prepare`, not `prepare_on` on a reading of its own.** The verifier
  of pass 4 set the fix before batch 15 made `prepare` read the GPU after the inventory and the
  builds, so that a command failing on them never loads NVML. A reading taken first would undo
  that for `bench run`, and `machine` would keep a `found_gpu` that cannot fail. `Prepared`
  carries the `Gpu` it was sized to instead, which the verifier had replaced with `prepare_on`
  when the fix carried the whole reading.

---

## Changes To Later Phases

None: no phase follows, and the roadmap closes after this phase.

---

## Assessment

The phase did what it set out to do, on a scale no one could size at its opening: it grew from
3 tasks to 66. The closing review, `docs/reviews/delivered-code/2026-10-17-v2-review-fixes.md`,
raised 84 findings in five passes and nine sessions, every one `CONFIRMED`: 1 high, 2 medium,
18 low and 63 without a severity. Under the user's delegation the session confirmed 83 and
rejected 1, finding 84, and the user settled three hard cases. The 58 batches kept are fixed,
one commit each, test-first, each diff through `cargo mutants --in-diff`; batch 41 alone was
rejected, so no batch is left open. The security pass over the fixes' diff,
`docs/reviews/delivered-code/2026-10-19-v2-review-fixes-security-pass.md`, raised 5 low
findings, all on what the fixes wrote, and S1 to S5 fixed them. Every acceptance criterion
holds; the five checks are green, with 722 tests in the library.

What the code gained, among the 63 batches:
- the most serious finding of the three reviews, a backend's predictable loopback port that a
  local process can take first to receive the key and the prompts, is documented, since
  llama-swap v263 reaches a backend over TCP only (`docs/notes/backends-on-unix-sockets.md`);
- `up` counts an answer on `listen` only from the llama-swap it started, for IPv4, IPv6 and the
  dual-stack socket of `0.0.0.0`;
- the inventory refuses underscored flags, option values and non-finite floats;
- `build --force` spares the build the gateway serves;
- `bench run` saves its run after each scenario, and prints a partial run before its error;
- the stores sweep what a kill leaves, and put back only what an interrupted replacement hid;
- doctor warns of a CDI specification that mounts a library no longer on disk, and lost the
  third of its time that rpm took;
- three missed mutants stay, each justified at its batch: `doctor.rs:428`'s `>` for `>=`
  (batch 12), `in_container` as `Ok(false)` (batch 49) and `read_gpu` as `Ok(None)` (batch 5).

What the phase showed about the workflow: the review of a fix roadmap's diff, scoped by file,
was a full review of 48 of the crate's 54 production files; half of its findings sit in code
older than the fixes; with no floor, each finding took a batch; and the review took nine
sessions where its fixes took three and a half. Mutants on each batch's diff, a security pass
alone at the end, verdicts given by the session and no pass after the security pass's fixes
broke the loop in practice; the skill holds none of them yet. The defects the phase showed in
`reviewing-delivered-code` are under Problems And Deviations, and
`docs/notes/review-workflow-second-assessment.md` sets out what to keep, change, remove and add.

What the roadmap's closure needs first: no phase follows. Its summary leaves open the skill's
defects recorded here, the four notes on the review workflow
(`review-workflow-tooling.md`, `review-workflow-second-assessment.md`,
the security skill's lessons, `review-costs-through-usage-py.md`), the three justified
mutants, and the review's kept work in `~/.local/share/ai-ctl/review-scratch/`, 3 MB outside
the repository, which the fixes no longer need.
