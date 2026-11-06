# Summary — v2 Review Fixes

---

## Where We Started

On 2026-10-16, the first review by `reviewing-delivered-code`,
`docs/reviews/delivered-code/2026-10-15-v2-rebuild.md`, had covered the whole v2 crate and raised
107 findings: 105 to fix and 2 to keep, in a fix plan of 55 batches the user had approved. Nine
defects found outside the review, P1 to P4 and E1 to E5, and two leads it left unverified, L1
and L2, joined them.

The most serious findings were about security. The gateway answered any local process and any
web page, and a value in the inventory could make llama-swap run a command on the host. Others
lost a gateway ai-ctl still ran, deleted the build it served, or discarded measured runs. The
tests let 52 written mutants pass. The roadmap was to apply the plan, so that the next roadmaps,
the audio and image backends first, build on reviewed code, and to end with a review of its own
changes.

---

## Where We Landed

Every batch of both fix plans is fixed, one commit each, test-first, with the five checks green.
The gateway and every backend answer only a client that holds ai-ctl's key, kept in a file at
mode 0600 and passed to llama-swap and the backends through the environment. A non-loopback
`listen` needs `expose = true`, and `up` counts an answer on `listen` only from the llama-swap it
started. The inventory refuses a line separator, an underscored flag, an option as a value and a
non-finite float; downloads and images are pinned. `docs/security.md` says who reaches what and
what guards it.

The builds, the gateway's records and the bench's runs survive the failures and interruptions
the reviews found: a served build is never pruned or replaced, a run is saved after each
scenario, and the stores sweep what a kill leaves. `doctor` went from 690 to 247 ms, then lost
the third of its time that rpm took; `status` from 41 to 20 ms. The ports are split by role,
the comparison rules live in the domain, and `docs/architecture.md` describes the code as it is.

The closing review, `docs/reviews/delivered-code/2026-10-17-v2-review-fixes.md`, raised 84
findings, 1 high, all fixed but one rejected; a security pass over those fixes,
`docs/reviews/delivered-code/2026-10-19-v2-review-fixes-security-pass.md`, raised 5, all fixed.
The library holds 722 tests, against 635 when the closing review began. Three missed mutants
stay, each justified at its batch. The review's high finding, a backend's predictable port that
a local process can take first, is documented rather than fixed: llama-swap v263 reaches a
backend over TCP only.

---

## What Each Phase Delivered

**Phase 0, Security.** One API key, created by `up` in a 0600 file, guards the gateway and every
backend; no configuration, command line or log holds it, and llama-swap refuses to start without
it. Two decisions changed after a probe: the key moved from `--api-key-file` to the environment,
and CORS stayed out once llama-swap's own guide showed that `allowedOrigins` lets requests run.
The live check closed every endpoint but `/health`, and found OpenWebUI's container published on
every interface, recreated on loopback.

**Phase 1, Bugs & Error Handling.** `docs/security.md`, ai-ctl's state directory at 0700, and
the review's bugs: a gateway followed through a linked data directory, `build prune` sparing the
served build under the gateway's lock, a failed bench request saving the scenarios before it,
lost GPU readings counted, `current` links read leniently by the commands that replace them. A
probe under load showed NVML's Reliability reason to be where the power cap does not hold the
clocks, so it stays out of "slowed by". The session plan's 12-token margin was left open.

**Phase 2, Dead Code & Tests.** The API only tests called is gone, and the 44 mutants the phase's
findings cite are all killed on the final code. Failure switches on the fakes reach the degraded
paths; two adapters hold their program as a field and run on scripts; every new fixture is a
capture of this machine. Doctor reads the GPU once, and L1 proved real: doctor now fails a check
on an inventory that `up` refuses.

**Phase 3, Performance.** The waits of `up` and `down` end at a 30 s deadline on an injected
clock, a grandchild no longer holds a command past its timeout (E1), NVML loads once per process,
`build` reads the newest release from its clone (E3, with git's minimum at 2.19.2, E5), GPU
processes are placed by walking up their parents, and the bench fits its prompts before the GPU
is watched.

**Phase 4, Structure & Refactoring.** The two wide ports are split by role, the rules for
comparing runs live in the domain, presentation builds the configurator once, the compile port
is `Compiler`, a saved run keeps its own copy of the build in format 3 (E2), and each copy the
review cited has one definition. The documents describe the structure the code has.

**Phase 5, Closing Review.** It grew from 3 tasks to 66. The closing review ran in five passes
and nine sessions; the session gave the verdicts under the user's delegation, the user the hard
cases. Its 58 batches were fixed in three and a half sessions, each diff through
`cargo mutants --in-diff`. A security pass alone closed the roadmap: its 5 findings, two of them
fixes that did not fully hold, became batches S1 to S5, with no pass after them. The closure
found and fixed one flaky test of S1. At the user's request, the review workflow was assessed a
second time, in `docs/notes/review-workflow-second-assessment.md`.

---

## What We Learned

- **A full review at the end of a fix roadmap starts a loop.** The closing review of a diff,
  scoped by file, read 48 of the crate's 54 production files: a full review. Half of its findings
  sit in code older than the fixes, and with no floor every finding takes a batch, whose lines
  the next review reads. Mutants on each batch's diff, a security pass alone at the end, verdicts
  given by the session and no pass after the security pass's fixes ended it here.
- **Mutants on each batch's diff close the tests axis at the commit.** The closing review needed
  28 test findings from mutants written by hand; on the diff of the 58 batches that followed the
  per-batch rule, `cargo mutants` missed 3, all justified.
- **Closing one hole shows the next.** Once phase 0 put a key on the gateway, a backend's
  predictable port became the most serious finding of the three reviews. The security pass at
  the end found what the fixes broke or left half done.
- **A fix misses the paths beside the one it guards, and the inputs beside its examples.**
  `build --force` lacked the guard phase 1 gave `prune`; a Ctrl-C lacked the save a failed
  request got; S1 and S4 held for the finding's examples and missed their class.
- **What an agent produces is worth more as an artifact than as prose.** The proposed tests,
  mutants and patches kept from the review went into about twenty batches nearly as they were.
- **A probe on this machine settles what reading cannot**: NVML's Reliability reason, llama-swap's
  key forwarding and its refusal of unix sockets, llama-server's time to fail on a taken port, a
  CDI specification gone stale after an update. Each fact went to `docs/stack.md` with its source.
- **Finding costs more than fixing.** Nine sessions found the closing review's findings; three
  and a half fixed them. A gate per wave and cost figures overstated two to three times stretched
  the review; costs are read from the records Claude Code writes, not from defaults.
- **A test that releases a socket holds the spawn guard**, or a child forked by a parallel test
  keeps it; one run per batch cannot show such a flaky test.
- **A tool's defect is fixed at its source.** The cost method went to the agent setup, the review
  skill's defects to its own change, never to a memory instruction.

---

## What We Are Leaving Open

- **The change to `reviewing-delivered-code`.** Its input is the defects under phase 5's
  Problems And Deviations and four notes: `review-workflow-tooling.md`,
  `review-workflow-second-assessment.md`, the security skill's lessons and
  `review-costs-through-usage-py.md`. The workflow that broke the loop is in none of the skill's
  steps yet.
- **The backends on TCP loopback.** The closing review's finding 1 stays documented in
  `docs/security.md` §2 until llama-swap reaches a backend over a unix socket;
  `docs/notes/backends-on-unix-sockets.md` holds the switch.
- **Three missed mutants**, each justified at its batch: `doctor.rs:428`'s `>` for `>=`,
  `in_container` as `Ok(false)`, `read_gpu` as `Ok(None)`.
- **The bench session plan's 12-token margin**, in `docs/notes/bench-session-margin.md`, and batch
  35 of phase 3, counted on the fake server and never benchmarked live.
- **The review's kept work** in `~/.local/share/ai-ctl/review-scratch/`, 3 MB outside the
  repository, which the fixes no longer need: to remove, or to seed a `repro/` folder.
- **`docs/toolchain.md`'s record** lacks the two rebuilds of `kmod-nvidia` for the 7.2.11 kernels
  of 2026-10-17, the user's to complete.
- **The agent setup's share**: the progress bar of a finished phase not yet closed, a first opening
  that leaves the roadmap in `pending/`, the phase report's size, a tick in `progress.py`, the
  Edit and Write tools decoding the line separator's escape, and skill-tooling 0.2.0's
  `usage.py estimate`.
- **The v2 rebuild's open items**, out of this roadmap's scope, stay as its summary lists them.
