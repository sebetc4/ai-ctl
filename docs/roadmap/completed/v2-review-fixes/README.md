# Roadmap: v2 Review Fixes

Fixes the findings of the first review of the v2 code and the defects found beside it, security
first, one batch per commit, test-first.

---

## Status Indicators

- 🔴 Not Started
- 🟡 In Progress
- 🟢 Done
- ⏸️ Blocked
- ⚠️ Needs Review

---

## Overall Progress

```
Phase 0  Security                   🟢 ████████████████████ 100%  (6/6)
Phase 1  Bugs & Error Handling      🟢 ████████████████████ 100%  (16/16)
Phase 2  Dead Code & Tests          🟢 ████████████████████ 100%  (17/17)
Phase 3  Performance                🟢 ████████████████████ 100%  (7/7)
Phase 4  Structure & Refactoring    🟢 ████████████████████ 100%  (16/16)
Phase 5  Closing Review             🟢 ████████████████████ 100%  (66/66)
TOTAL                                  ████████████████████ 100%  (128/128)
```

**Current Phase:** —
**Blocked By:** —
**Next Milestone:** —

---

## Why This Roadmap Exists

The first review by `reviewing-delivered-code` covered the whole v2 crate in five passes. It
raised 107 findings, each confirmed by a verifier and by the user: 105 to fix and 2 to keep. Its
fix plan of 55 batches was approved on 2026-10-15, and pass 5's batches on 2026-10-16. Two other
sources add work:
- nine defects found outside the review, confirmed for this roadmap: P1 to P4, from the
  baselines of the review-skill roadmap, and E1 to E5, from checking the review's leads;
- two leads that pass 5 left unverified, here L1 and L2.

The review's most serious findings concern security. The gateway is open to any local process
and to any web page (finding 1), and a value in an inventory can make llama-swap run a command
on the host (finding 3). Others lose a gateway ai-ctl still runs, delete the build it serves,
or discard measured runs. The tests let 52 written mutants pass.

This roadmap applies the approved plan, so that the next roadmaps, the audio and image backends
of decision 0001 first, build on reviewed code. It ends with a review of its own changes.

---

## Decisions Taken At Opening

- **The approved plan is the work.** Each task is one batch of the review's Fix Plan, and the
  phases follow the plan's order: security, bugs and error handling, dead code, tests,
  performance, structure and refactoring, documents. The plan's placements are kept:
  - batch 26, with P3, among the bugs;
  - batch 53 after batches 18 and 19;
  - pass 5's batches 52 to 55 where they apply;
  - each defect found outside the review with the batch it joins.

  The tasks use the report's batch numbers.
- **A batch applies its findings' Fix as verified**, the verifiers' corrections included, with
  the tests those findings name. When an earlier batch has moved the code, the finding's cause
  decides what to change, not its line numbers.
- **Tests:** a bug fix starts with the test that reproduces it, watched failing on its
  assertion. A test finding's new test passes on the code and fails under the mutant the
  finding cites. The mutant is applied by exact replacement in a scratch copy of the tree, from
  the report's Mutants list. New fixtures are captured on this machine.
- **One commit per batch**, in Conventional Commits, and each commit leaves the five checks of
  CLAUDE.md green.
- **Settled with the review's verdicts:**
  - findings 99 and 100 are kept (batch 51);
  - finding 32 takes option (a): `bail!`, and the type dropped;
  - finding 46's old D-Bus name goes with no `// LEGACY:` obligation.
- **Left to phase 0:** whether finding 1's API key also guards the llama-server backends
  (`--api-key`) or only the gateway, and how a client gets the key. The user decides after a
  probe of llama-swap v263.
- **The two leads of pass 5 are verified before any fix**, by a test or a probe. A lead
  confirmed is fixed in its phase. A lead refuted is recorded in its phase's report and dropped.
- **A defect found along the way that no finding, defect or lead covers** is recorded in the
  phase's report and put to the user, not fixed unasked.
- **A closing review:** phase 5 reviews the roadmap's code with `reviewing-delivered-code`. For
  each batch of its fix plan, the user decides whether this roadmap fixes it or leaves it open.
- **Machine at opening:** Fedora 44, NVIDIA driver and NVML 615.78.08, CUDA 13.4, llama-swap
  v263, native build `b11538`, podman 5.8.7, git 2.56.0, CMake 4.3.0, rustc 1.97.1.

---

## Deliberately Out Of Scope

- Findings 99 and 100, kept by the review.
- The open items of the v2 rebuild's summary:
  - the official image's slower short prompts;
  - OpenWebUI's request ahead of a chat's first answer;
  - `disableKeepAlives`;
  - the scenarios never run for real;
  - the bench's plan defaults;
  - the order of blocks in a Messages stream.
- `reviewing-delivered-code` itself and the open items of the review-skill roadmap. Phase 5 runs
  the skill as it stands.
- New features, the directions of `docs/notes/`, and the audio and image backends.
- `cargo mutants` over the whole crate. The review's mutants are the measure of the test fixes.

---

## Phases

- 🟢 [Phase 0: Security](phase-0-security.md) — an API key and a loopback listen address for the gateway, no line separator in inventory text, an honest check of a non-pinned release, a pinned download remedy, images by digest (batches 1–5)
- 🟢 [Phase 1: Bugs & Error Handling](phase-1-bugs-and-error-handling.md) — a security document from phase 0, the gateway's records and builds, the bench's plan, saved runs and GPU readings, the stores' links, doctor's toolkit remedies, the inventory's flag checks (batches 6–13, 26, 52, 54, 55; P1–P4; L2)
- 🟢 [Phase 2: Dead Code & Tests](phase-2-dead-code-and-tests.md) — the API only tests call removed, the review's test gaps closed against their mutants, doctor reading the GPU once (batches 14–25, 27, 28, 53; E4; L1)
- 🟢 [Phase 3: Performance](phase-3-performance.md) — deadlines for `up`'s and `down`'s waits, NVML initialised once, commands waited for without polling, the newest tag read locally, GPU owners found by walking up, prompts fitted once (batches 29–35; E1, E3, E5)
- 🟢 [Phase 4: Structure & Refactoring](phase-4-structure-and-refactoring.md) — ports split by role, comparison rules in the domain, the review's copies shared once, the documents matching the final structure (batches 36–50; E2)
- 🟢 [Phase 5: Closing Review](phase-5-closing-review.md) — `reviewing-delivered-code` on the roadmap's code, and the fixes the user keeps for this roadmap

---

## Dependencies

- Each phase builds on the previous one, in the plan's order.
- Batches 18, 19 and 21 add failure switches to the fakes. Batch 53 and L1 use them in phase 2,
  and batches 40 and 41 split them with their ports in phase 4.
- Batch 34 replaces `descendants` with `GatewayProcesses::stat` and `in_container` before batch
  40 splits the port.
- Batch 45 builds on batch 31's NVML and takes the GPU reading that batch 53 passes around.
- Batch 43's HTTP helpers build on batch 1's `Authorization` header.
- Finding 89's boundary test, written in batch 19, follows the git minimum that E5 sets in batch
  33.
- Batch 54 counts a policy limit as a slowdown only after a probe under load on this machine's
  GPU.
- Phase 5's review starts its agents only after the user's go-ahead on its estimate.

---

## Related Documentation

- [docs/reviews/delivered-code/2026-10-15-v2-rebuild.md](../../../reviews/delivered-code/2026-10-15-v2-rebuild.md) — the findings, the mutants, the fix plan, and the defects found outside the review
- [docs/roadmap/completed/review-skill/summary.md](../../completed/review-skill/summary.md) — how the review was made and what it left open
- [docs/roadmap/completed/v2-rebuild/summary.md](../../completed/v2-rebuild/summary.md) — the code under review and the v2 rebuild's open items
- [docs/architecture.md](../../../architecture.md) — layers, dependency rule, port policy and testing
- [docs/gpu/llama-swap.md](../../../gpu/llama-swap.md) — decision 0001, llama-swap as the single gateway and GPU arbiter
- [docs/stack.md](../../../stack.md) — the clients' configuration and where models come from
- [docs/toolchain.md](../../../toolchain.md) — the CUDA toolkit's packages and `alternatives`, for batch 52

---

## Metadata

**Roadmap Status:** 🟢 Done
**Location:** `docs/roadmap/completed/v2-review-fixes/`
**Version:** 2.0.0
**Created:** 2026-10-16
**Last Updated:** 2026-10-19

---

## Changelog

### 2.0.0 (2026-10-19)

Roadmap closed: its six phases are done, 128 tasks. `summary.md` records where it started and
where it landed, what each phase delivered, what holds beyond it (the loop of reviews and fixes
and what ended it, mutants on each batch's diff, the security pass at the end, sibling paths and
input classes), and what is left open: the change to `reviewing-delivered-code` with its four
notes, the backends on TCP loopback, three justified mutants, the bench session's margin, the
review's kept work outside the repository, and the agent setup's share. The folder moved from
`docs/roadmap/on-progress/` to `docs/roadmap/completed/`; the repository's README, `CLAUDE.md`
and `tests/fixtures/README.md` name the new path, and the closed reports keep the paths they
were written with. No restructuring was pending.

### 1.6.0 (2026-10-19)

Phase 5, Closing Review, closed with its 66 tasks and every acceptance criterion met:
- `reviewing-delivered-code` reviewed the roadmap's code in five passes,
  `docs/reviews/delivered-code/2026-10-17-v2-review-fixes.md`: 84 findings, each confirmed by a
  verifier, 1 high and 2 medium among them; the session gave the verdicts under the user's
  delegation, 83 confirmed and finding 84 rejected;
- the 58 batches kept are fixed, one commit each, test-first, with `cargo mutants --in-diff` on
  each batch's diff; batch 41 alone was rejected, so none is left open;
- a security pass over the fixes' diff,
  `docs/reviews/delivered-code/2026-10-19-v2-review-fixes-security-pass.md`, raised 5 low
  findings, fixed as batches S1 to S5.

The phase grew from 3 tasks to 61 when the fix plan was approved, then to 66 with the security
pass's batches. The review's high finding, a backend's port that a local process can take first,
is documented: llama-swap v263 reaches a backend over TCP only. Three missed mutants stay, each
justified at its batch. The defects the review showed in `reviewing-delivered-code` are in the
phase's report, and the second assessment of the review workflow, asked by the user at the
close, is `docs/notes/review-workflow-second-assessment.md`: the loop of reviews and fixes, and
how this phase broke it.

### 1.5.0 (2026-10-17)

Phase 4, Structure & Refactoring, closed with its 16 tasks and every acceptance criterion met:
- `GatewayProcesses` is split into `GatewayProcess`, `ProcessTable` and `PortCheck`, and
  `HostProbe` into `ToolchainProbe`, `CdiProbe` and `PowerProbe`; `Bench` holds the gateway's
  port and the power probe alone;
- the rules for comparing runs live in the domain, and `render_run` takes the overhead's
  medians from it;
- `Services::configurator` builds the configuration use case, and doctor takes its pause and
  clock from `Services`;
- the compile port is `Compiler` and its adapter `Cmake`; the build gate refuses with `bail!`;
- a saved run keeps its own copy of the build, in format 3, and reads formats 1 and 2 (E2);
- the review's copies exist once: command errors and install commands, the HTTP helpers, the
  current build, the GPU reading, the live gateway record, the model id check, the gateway's
  state, and five small helpers;
- `docs/architecture.md`, `main.rs` and CLAUDE.md describe where use cases are assembled, the
  exit codes, each command's use case, the real ports and fixtures, the testing conventions
  and the whole repository layout.

The seven saved runs of this machine still list and compare. Batch 40 keeps `signal` with the
gateway's port, and batch 48 lists every GPU process with its memory, both recorded in the
report. Batches 43 to 48 changed error and status texts where their findings asked for one
text. The user's `docs/commands.md` and `Makefile` were committed during the phase, and the
architecture lists them.

### 1.4.0 (2026-10-17)

Phase 3, Performance, closed with its 7 tasks and every acceptance criterion met:
- `up`'s and `down`'s waits end at a 30 s deadline on an injected clock, and a health check
  waits 500 ms at most;
- `rpm` lists the toolkit packages without checking their digests;
- NVML is initialised once per process;
- commands are waited for without a sleep loop, and a grandchild holding a pipe no longer
  outlives a timeout (E1);
- `build` fetches first and reads the newest release from its clone; `resolve`'s explicit
  fetch is gone (E3), and git's minimum is 2.19.2 (E5);
- GPU processes are placed by walking up their parents, and `podman ps` runs only for one
  that may be a container's;
- the bench fits its prompts before the GPU is watched, counts the agent's bare prompt once
  and cuts its leads without fitting them.

On this machine, `doctor` went from 690 to 247 ms and `status` from 41 to 20 ms, their output
unchanged. Found along the way, and fixed before its commit: batch 34's first walk lost a PID
whose stat it could not read. Batch 34's methods are `stat` and `in_container` rather than
`parent`, which phase 4's dependency line and this README follow. Batch 35 was counted
rather than timed live.

### 1.3.0 (2026-10-16)

Phase 2, Dead Code & Tests, closed with its 17 tasks and every acceptance criterion met:
- the API only tests called is removed, and the power-profile probe asks the UPower name
  alone;
- the review's test gaps are closed: the 44 mutants the phase's findings cite, run again on
  the final code, are all killed;
- failure switches on the fakes reach the degraded paths of the lifecycle, doctor, the builder
  and `gateway install`;
- `CmakeBuilder` and `Podman` hold their program as a field and run on scripts in their tests;
- new fixtures captured on this machine: a Make build, a JSON CDI specification, podman's
  digest and PID outputs;
- doctor reads the GPU once, reports what its inspection could not read, warns of a gateway
  whose configuration cannot be told, and fails a "Gateway configuration" check on an
  inventory that `up` refuses.

L1 proved real and is fixed. Found along the way: `down` reported a record it could not remove
as its only error; fixed with the user's approval, which grew the phase from 16 tasks to 17.
The session plan's 12-token margin stays open, in `docs/notes/bench-session-margin.md`. Phase
4's batches 42 and 45 now note what this phase changed under them.

### 1.2.0 (2026-10-16)

Phase 1, Bugs & Error Handling, closed with its 16 tasks and every acceptance criterion met:
- `docs/security.md`, and ai-ctl's state directory at mode 0700;
- the gateway's records and builds:
  - a gateway is followed through a linked data directory;
  - `build prune` spares the build the gateway serves;
  - `up` keeps the record of a gateway it could not stop;
- the bench:
  - the plan fits the selected scenarios and counts a session's turns whole;
  - a failed request saves the scenarios completed before it;
  - lost GPU readings are counted;
  - NVML's raw clock-event mask is kept in a run of format 2;
- `current` links read leniently by the commands that replace them;
- a run published without leftovers;
- doctor's remedies for a missing llama-swap binary and for a toolkit newer than the driver;
- the inventory refusing file keys and defaults that repeat a flag;
- a failing command's error line taken past podman's log lines.

A probe under load on this machine showed NVML's Reliability reason only where the power cap
does not hold the clocks, and BoardLimit never. Reliability stays out of "slowed by", in its own
column. The same run confirmed the session's plan with 12 tokens to spare; that margin is left
open. The phase grew from 14 tasks to 16: the security document, added at phase 0's closure,
and the state directory, added during the phase. Phase 2's batch 16 now notes that finding
67's test exists since batch 6.

### 1.1.0 (2026-10-16)

Phase 0, Security, closed with its six tasks and every acceptance criterion met:
- one API key, created by `up` in a 0600 file, guards the gateway and every backend; it reaches
  llama-swap and the backends through the environment, and no configuration, command line or log
  holds it;
- a non-loopback `listen` needs `expose = true`;
- inventory text refuses U+2028 and U+2029;
- a non-pinned release's check says it detects corruption only, and `--sha256` checks a digest
  the user trusts;
- doctor's download remedy pins a revision;
- images are pinned by digest, and `up` warns of a bare tag.

Checked live on this machine: every endpoint but `/health` answers 401 without the key, on the
gateway and the backend, and OpenCode and OpenWebUI work with it. The live check found OpenWebUI's
container published on every interface, recreated on loopback, and `unauthorized` lines that
llama-swap's own requests leave at each model load. Phase 1's security document now covers both.
The key moved from `--api-key-file` to the environment, and CORS stayed out of scope, both after
a probe. The roadmap moved from `pending/` to `on-progress/`.

### 1.0.0 (2026-10-16)

Roadmap created from the fix plan of `docs/reviews/delivered-code/2026-10-15-v2-rebuild.md`,
with six phases and 62 tasks:
- Security (6);
- Bugs & Error Handling (14);
- Dead Code & Tests (16);
- Performance (7);
- Structure & Refactoring (16), which ends with the documents;
- Closing Review (3).

Each task of phases 0 to 4 is a batch of the plan, with the defects found outside the review
that join it and the two leads of pass 5 (L1, L2). Phase 0 opens with the decision finding 1
left open: whether the API key also guards the backends. Phase 5 reviews the roadmap's own code.
