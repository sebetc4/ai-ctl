# Phase 4 Report: First Review of ai-ctl

**Phase:** [phase-4-first-review.md](phase-4-first-review.md)
**Start Commit:** `docs(review-skill): close phase 3 on iteration 2 by the stop rule` of 2026-10-15

---

## Work Log

### 2026-10-15

Phase opened at the end of phase 3's closure (`docs(review-skill): close phase 3 on iteration 2 by the stop rule`), in the session that closed it.
Phase 3's file and report were written and read in full in that session.

One `skill-auditor` pass on the skill as phase 3 left it: `VERDICT: FAIL`, five points, each
fixed in `SKILL.md`.

| Point | Fix |
|---|---|
| Read-only commands, no source | Cited `src/presentation/cli.rs`, each command's help |
| `cargo test` safe, no source | Cited `tests/gateway_lifecycle.rs`'s header: fake llama-server, no GPU, skipped without llama-swap |
| `cargo audit` and its database, no source | A probe of `cargo audit --help`, cargo-audit 0.22.2, `--db` default and `--no-fetch` |
| "checks", "review tools", "tools' output" for one table | "check" throughout |
| `rm -rf <scratch>` unguarded | The command refuses any path outside `${TMPDIR:-/tmp}/tmp.*`; probed on a `mktemp -d` folder (removed) and on a path outside it (refused, exit 1) |

`audit.py`: 0 errors, the same three T1 warnings. Checks green.

Where the next session starts: in a fresh usage window, from a new session, read this
phase's file and report, then run the skill on the v2-rebuild roadmap at `high`. The skill
shows its estimate and its passes (about $39, three, `review-security` in the first) and
waits for the user's yes. Each pass takes a session of its own. After each pass, take the
per-agent costs from the session's transcript with authoring-skills' `scripts/usage.py`
(the transcript and its `subagents/` files as arguments), and record here any failure the
skill shows, quoted with its transcript.

New session, pass 1 of the review. `scope.py roadmap docs/roadmap/completed/v2-rebuild
--files` exited 0: phase 0's start commit `1a49576` is absent from the history, so the scope
is the whole history from the root commit `chore: restart llm-ctl v2 on a clean base` of 2026-10-09 to `docs: close the v2 rebuild roadmap` of 2026-10-12, 53 production files of
14,016 lines and 41 test files of 11,591 lines; no code changed since `docs: close the v2 rebuild roadmap` of 2026-10-12. The user
asked to see the estimate and the passes before anything started, so they were shown after
step 1 and before the checks of step 3, which the skill orders first; the estimate depends on
the scope only.

The skill's rates gave $39.12: six reviewers $20.63 (five at $3.05, `review-tests` at $5.37
on 25,607 lines), six verifiers $15.47, the session $3. Split into three passes of $15 at
most: security and correctness ($10.69 with their verifiers), tests and cleanup ($14.74),
structure and performance ($10.69). `review-correctness` joined `review-security` in pass 1
so that the bugs, and the count of phase 1's eight, come first. User's yes: "tu peux lancer".

- **Failure from use, candidate: the estimate counts the session once.** The skill's step 4
  says "yourself: $3", and its three passes take three sessions, each one paying its own
  orchestration; the estimate shown to the user was $39 by the rule and about $45 with the
  session counted per pass. To settle against the measured cost of each pass's session.

Checks, all green: `cargo fmt --check`, `cargo clippy`, `cargo test` (518 passed;
`up_a_proxied_request_and_down ... ok`, not skipped), `test_scope.py` (15), `cargo audit`
(66 dependencies, no advisory). `cargo mutants --list --in-diff` counted 1,907 mutants, over
the limit of 300, so it did not run and `review-tests` will have no missed mutants as leads.
The review's report was created at `docs/reviews/delivered-code/2026-10-15-v2-rebuild.md`,
and `review-security` and `review-correctness` started at effort `high`.

`review-security` answered in its format after 4.7 minutes (145k tokens, 30 tool calls):
seven findings, one medium (no `apiKeys` nor origin limit in the generated llama-swap
configuration), six low, none reproduced. `review-correctness` answered after 9.5 minutes
(298k tokens, 53 tool calls): ten findings, one medium (the symlink in the data path), two
reproduced by a throwaway crate in scratch. Its `LEFT OUT` names the timing of `up`'s poll
loops as unchecked, and reports an NVML driver/library mismatch on the machine (driver
updated, no reboot) that kept every GPU path out of reach. No finding is shared between the
two axes; the two `current`-link findings of `review-correctness` share one cause and were
merged, leaving 16 findings, given to two `review-verifier` at `high`, one per reviewer.

Against phase 1's eight bugs, before verification: the reviewer raised the symlink, `build
prune`, `bench run` printing before saving and `Plan::new`'s context; it missed `doctor`'s
remedy ending at `publish`'s refusal, `SessionPlan::peak`'s undercount and the duplicated
flags from a model's file keys, and left `up`'s 300 polls out on purpose.

The two verifiers answered in their format. `review-correctness`'s verifier (4.7 minutes, 43
tool calls) confirmed all nine, reran both reproductions, added two failing probe tests in
a copy and a third binary, and corrected four of them: the reviewer's first fix for the
symlink, canonicalizing the recorded binary, would have made `up` refuse a running gateway
through `is_current`; three consequences were overstated (runs of 16 to 20 minutes, not 30
to 60; `init`'s own remedy gets out of the dangling link; a re-run of `build` fails before
"already built"); and the partial-file fix should share `write_new`'s code. `review-security`'s
verifier (5.8 minutes, 33 tool calls) ran the installed llama-swap v263 alone on
`127.0.0.1:18999`, with configurations of its own and a `touch` as the model's command: it
confirmed six, one stronger than raised (CORS `*` by default, so a web page needs no DNS
rebinding) and reproduced end to end (a U+2028 in an inventory flag makes llama-swap run a
command on the host), narrowed two (the shipped inventory pulls no image; `model_check`
follows a recorded decision), and refuted one with podman 5.8.7 (an image reference
starting with `-` reaches no container). The review's report holds 15 findings, all
CONFIRMED, its Status naming pass 2.

Against phase 1's eight bugs, verified: four found again (the symlink, `build prune`, `bench
run` printing before saving, `Plan::new`'s context), all four CONFIRMED.

The pass's cost, from `usage.py` on the session's transcript and its four `subagents/`
files, on 2026-10-15 before the session ended:

| Agent | Calls | Cost | Of which estimated | Step 4's estimate |
|---|---|---|---|---|
| `review-security` | 28 | $4.08 | $2.81 | $3.05 |
| `review-correctness` | 51 | $6.99 | $3.67 | $3.05 |
| `review-verifier`, security | 33 | $4.00 | $3.46 | $2.29 |
| `review-verifier`, correctness | 42 | $5.44 | $4.54 | $2.29 |
| The session, so far | 40 | $2.54 | $0.00 | $3.00 |
| **Pass 1** | 194 | $23.05 | $14.47 | $13.69 |

134 of the subagents' 154 calls lack their final usage in the transcript, so `usage.py`
gives each one the default 5,400 output tokens of `usage.toml`, measured on writing agents;
no figure exists for the review agents. The exact total is the `cost-state` record Claude
Code writes into the session's transcript when the session ends
(the agent setup's `docs/decisions/2026-10-06-token-costs.md`): to be read at the start of
the next session, before step 4's rates are set.

- **Failure from use: step 4's rates are too low on a 14,016-line scope.** Even the exact
  part alone, $8.58 for the four agents and the session, reaches 63% of the pass's
  estimate; with the default output, the reviewers cost 1.3 and 2.3 times their estimate
  and the verifiers 1.7 and 2.4 times. The pass meant to stay under $15 measured $23 at the
  default, over the $20 window. Pass 2's $14.74 estimate is not safe to run as split: to
  settle with the exact total before pass 2 starts.
- **False positive of `review-security`:** "The image reference is not validated, so a
  leading `-` is read as a podman option", refuted by running podman. The reviewer reasoned
  on the argument order and did not try podman's parser, which it could have run.
- **A reviewer's fix that would have added a bug**, caught by the verifier (above): the
  verification step earned its cost on this finding alone.
- **A verifier ran llama-swap.** The skill forbids ai-ctl's commands that start the stack
  and anything that starts a process on the GPU or changes the machine; a verifier running
  the installed llama-swap on another port with a harmless command falls outside both, and
  gave the pass its strongest evidence. Recorded as use, not as a failure: whether the skill
  should say so is for the skill changes task.

`.gitignore` gained `/notes`, and an empty `notes/` folder appeared, at 18:02 during the
pass; neither came from the review (no agent and no command of this session wrote them).
The user asked for the `.gitignore` line to be committed with this session's record. The
scratch folder was removed by step 10's guarded command (`ls` fails on it), and `git diff`
on `src/`, `tests/`, `Cargo.toml`, `Cargo.lock` and `deploy/` is empty. The session ended
at the user's request, after pass 1, to resume in a fresh session.

**Where the next session starts:**
1. In a fresh usage window, from a new session: read this phase's file, this report and the
   review's report, `docs/reviews/delivered-code/2026-10-15-v2-rebuild.md`.
2. Read the `cost-state` record Claude Code wrote at the end of this session's transcript: the
   exact cost of pass 1, subagents included. Set it against the table above, and split it
   among the four agents.
3. From that figure, set step 4's rates (the task "Record the run's cost per agent…") and
   recompute the remaining passes; show the user the new split before pass 2 starts.
4. Continue the review at step 6 with the pass its Status names, `review-tests` and
   `review-cleanup` unless the new split changes it.
- To put to the user: how the remaining four reviewers are split, if pass 2 as estimated
  ($14.74) does not fit a window at the measured rates; the recommendation is one reviewer
  and its verifier per session.
- Pending: nothing uncommitted; no run in progress.

New session, for pass 2. Pass 1's transcript holds one `cost-state` record (line 612):
**$11.89**, all on Claude Opus 5.5 but $0.0001 of Haiku. Its input, cache reads (20,479,730)
and cache writes (718,759 = 576,679 at 5 minutes + 142,080 at 1 hour) equal `usage.py`'s to
the token; only the output differs: 188,453 tokens, against `usage.py`'s 798,879. The 77
calls with their final usage hold 75,279 of them, so the 134 calls without it produced
113,174, **845 a call**, not `usage.toml`'s default 5,400. The `$23.05` above, and the
failure "step 4's rates are too low" drawn from it, came from that default, not from the
review.

Pass 1, the exact total split by agent: each agent's input and cache tokens as `usage.py`
counts them, its exact output, and 845 tokens for each of its calls without a final usage
(the total is exact; the split assumes the same output on every such call). Durations from
the agents' answers; the session's from the `cost-state` record (`totalDuration`).

| Agent | Calls | Without usage | Cost | Step 4's estimate | Ratio | Duration |
|---|---|---|---|---|---|---|
| `review-security` | 28 | 26 | $1.71 | $3.05 | 0.56 | 4.7 min |
| `review-correctness` | 51 | 34 | $3.89 | $3.05 | 1.28 | 9.5 min |
| `review-verifier`, security (7 findings) | 33 | 32 | $1.09 | $2.29 | 0.48 | 5.8 min |
| `review-verifier`, correctness (10 findings) | 42 | 42 | $1.61 | $2.29 | 0.70 | 4.7 min |
| The session, at `xhigh`, closing record included | 57 | 0 | $3.59 | $3.00 | 1.20 | 28 min |
| **Pass 1** | 211 | 134 | **$11.89** | $13.69 | 0.87 | |

Against step 4's rates: the reviewers average $2.80 against $3.05, but differ by 2.3 times
on the same lines, so a reviewer's cost follows what its axis makes it open and run more
than the lines in scope (`review-correctness` read 8.6M cache tokens, `review-security`
2.6M). Each verifier cost 0.64 and 0.41 of its reviewer, below the rule's three quarters.
The session cost $3.59, and each pass pays it again.

- **Failure of a tool, not of the skill:** `usage.toml`'s default output of 5,400 tokens,
  measured on writing agents, is 6.4 times what the review agents produced at `high`
  (845). An estimate `review-agents.high = { tokens = 845 }` belongs in authoring-skills'
  `usage.toml`, which the agent setup owns: a proposal there, not an edit.

At the measured ratios, pass 2 as planned costs about $15: $10 with every agent at its
lowest ratio, $21 at its highest. The user chose pass 2 as planned, with a guard (below).
`scope.py` gave the same scope, exit 0; no code changed
since `docs: close the v2 rebuild roadmap` of 2026-10-12. `review-tests` and `review-cleanup` started at effort `high`, with no missed
mutants for `review-tests`.

`review-tests` answered in its format after 9.3 minutes (50 calls, 73 tool uses): 38
findings. With no `cargo mutants` run, it wrote 49 mutants of its own as old/new pairs and
ran each against `cargo test --lib --test cli --test gateway_config` in two copies of the
tree in scratch. Every finding but one cites a mutant that survives; the last is read.
It left `tests/gateway_lifecycle.rs` out of the runs, since that test runs `up` and `down`
against the installed llama-swap. It also dropped one surviving mutant without a word,
reading `--list-devices` from stdout alone. Checked: llama-server prints the list on stdout
(v2-rebuild `phase-2-native-build-report.md:136`), so the mutant is equivalent, and leaving
it out was right. `review-cleanup` answered after 12.1 minutes (92 calls, 100 tool uses): 11
dead-code findings and 21 refactoring findings. The dead code was found by a compiler check
on a copy with every `pub` made `pub(crate)`, then a second copy without the derives that
hide unread fields. The refactoring findings came from a repeated-window detector and greps.
The two reviewers clash twice: `review-tests` asks for a test of `relative_spread` and of the
`net.hadess` fallback, and `review-cleanup` asks to remove both.

The guard, measured with `usage.py` at 845 output tokens a call: `review-tests` $5.68 (19 of
50 calls estimated), `review-cleanup` $5.27 (91 of 92 estimated), the session $1.69, so
$12.64, under the $13 limit. But a verifier's cost follows its findings, not its reviewer:
about $0.16 a finding in pass 1 ($1.09 for 7, $1.61 for 10). Seventy findings put the two
verifiers near $11 and the pass near $26. The user moved both verifications to the next
session.

Step 7 ran in this session. Three of `review-tests`' findings merged into `review-cleanup`'s:
`relative_spread` and the `net.hadess` fallback into their removal, and doctor's untested
text for a reused PID into C21's shared description. T6 split in two, the failed stop at a
start timeout joining finding 4, whose fix adds that test. That leaves 36 findings in batch
T and 32 in batch C. They are written, in the reviewers' format with the 45 mutants they
cite, under a section `## To Verify` of the review's report, ahead of Coverage. The scratch
folder does not outlast a session, and `/tmp` is a tmpfs here. The report's Status names
pass 3 of 4, the two verifiers; pass 4 is `review-structure` and `review-performance`.
Coverage holds the three axes of pass 2 and what each reviewer read.

Pass 2's cost at the end of its work, before the session's record: the two reviewers
$10.95 and the session $2.88 (45 calls), so $13.83. The exact figure is the `cost-state`
record at the end of this session's transcript.

- **Failure from use: step 4 estimates a verifier from its reviewers' cost** ("three
  quarters of the reviewers whose findings it checks"). Its cost follows the number of
  findings. Pass 2's reviewers cost as estimated, yet their 70 findings would have taken
  their verifiers to about 1.4 times step 4's $6.32, past the window.
  Step 4 cannot know the finding count beforehand. Step 8 can: after merging, it could
  price the verification and split the pass there, as this session did by hand.
- **Failure from use: no place for findings that wait for a verifier across sessions.** The
  skill keeps everything but the report in a scratch folder that step 10 removes, and has a
  verifier run in its reviewer's pass. The `## To Verify` section is this session's answer,
  outside the template.
- **Two reviewers' findings contradict each other** (a test asked for code that another
  reviewer asks to remove). Step 7's merge resolved both; the skill's merge rule covers it
  only as "share a root cause".

The scratch folder was removed by step 10's guarded command (`ls` fails on it); `git diff`
on `src/`, `tests/`, `Cargo.toml`, `Cargo.lock` and `deploy/` is empty.

**Where the next session starts:**
1. In a fresh usage window, from a new session: read this phase's file, this report and the
   review's report, `docs/reviews/delivered-code/2026-10-15-v2-rebuild.md`, whose
   `## To Verify` section holds batch T (36 findings) and batch C (32).
2. Read the `cost-state` record at the end of this session's transcript, and
   split it as pass 1's was: the exact output of the 110 estimated calls of the two
   reviewers gives a second measure of the 845 tokens a call.
3. Pass 3: create a scratch folder (step 2), give batch T to one `review-verifier` and batch
   C to another, at `high`, with the scope and the scratch folder; the mutants under To
   Verify let batch T's verifier rerun them in a copy rather than rewrite them. Write each
   verdict under Findings (step 8), T6b into finding 4, then remove the section. Show the
   user the measured cost before pass 4 starts.
- Pending: nothing uncommitted, the session's record committed with the review's report
  at its close; no run in progress.

New session, for pass 3. Pass 2's transcript holds no `cost-state` record: its session
did not end before this one started, so the exact figure is lost. `usage.py` gives its
own 53 calls exactly, **$3.41**, all final; with the two reviewers' $10.95 at 845 output
tokens a call, pass 2 cost about **$14.36**. No second measure of the 845 tokens a call.

`scope.py` gave the same scope, exit 0; no code
changed since `docs: close the v2 rebuild roadmap` of 2026-10-12. Batch T, batch C and the 45 mutants were copied from the review's
report into scratch files, and each verifier given its file rather than the findings in its
prompt. Two `review-verifier` started at `high`, in the background, each in its own copy of
the repository: batch T's to rerun the reviewer's mutants, batch C's to rerun the compiler
check on a copy with every `pub` made `pub(crate)`.

Batch C's verifier answered in its format after 7.9 minutes (58 tool uses, 136k tokens):
31 CONFIRMED, 1 REFUTED. Both compiler checks, rerun, gave the reviewer's ten items exactly;
the three merged tests mutants survive, and the removals of C3 and C11 close their gaps, C21's
sharing too (the same change in presentation fails a test). It corrected nine fixes: a
helper narrowed to what the copies share (C12, C14, C17, C19), an existing error reused
rather than a new one (C18), a three-way outcome where the reviewer's `Option` lost a case
doctor separates (C15), each site's context kept (C16), a text change named as such (C23),
a test kept that the removal would have dropped (C11). It kept two copies as they are
(C28, C29), the shared code costing what it saves, and refuted C31: the two quoting
functions follow two rules, the shell's and llama-swap's splitter's. Written as findings 16
to 46 of the review's report; batch C removed from `## To Verify`.

- **False positive of `review-cleanup`:** C31, two rules that look alike read as one copy,
  from a repeated-window detector. Recorded for the task on false positives.
- **`review-cleanup`'s refactoring fixes were often too broad**: of its 21 refactoring
  findings, eight fixes corrected, two dropped (C28, C29) and one finding refuted (C31):
  the reviewer proposed the helper from the detector's window, without checking each site's
  differences. The verifier's method, "the shared code would do the same work", caught them.

Batch T's verifier answered in its format after 14.9 minutes (91 tool uses, 175k tokens):
35 CONFIRMED, T6b included, 1 REFUTED; its own summary line said "31 confirmed", against its
36 blocks, which are the record. It reran all 41 mutants the batch cites, every one
surviving; wrote 8 of its own, six of them for T38's parts; and ran trial tests for eight
findings against the code and the mutant. Those trials found three of the reviewer's fixes
that would not work: T3's script makes `/proc/<pid>/exe` name the interpreter, so the test
fails on the code itself; T25's two models fit at once, so the test lets the mutant pass;
T2's `trap` races with the signal, bringing timing into a test. It corrected T18's input
to the order `host_compilers` produces, and narrowed seven consequences. It refuted T21:
the clone is partial, so `rev_parse` fetches a missing commit itself, and the mutant is
equivalent; its lead, that the explicit fetch may be redundant, is recorded in the review's
report as a lead, not a finding. T6b joins finding 4: its test must assert the whole message,
not only the kept record. Written as findings 47 to 80; findings 11 to 15, which the first
insertion had pushed after 80, moved back.

The review's report then lost its `## To Verify` section. The 45 mutants, with the
verifier's seven that findings 69 and 80 cite, moved under Checks: the fix roadmap checks
each new test against its mutant. Status: pass 4 of 4.

- **Failure from use: `review-tests` proposed tests it had not run.** Three of its 36 fixes
  would not have caught their mutant, or not without timing; only the verifier's trials
  showed it. The reviewer had the mutants and the copies to run each proposed test against
  them.
- **False positive of `review-tests`:** T21, a mutant that survives because it is
  equivalent, not because a test is missing; the reviewer did not check why it survived.
- **A verifier's summary line disagreed with its blocks** (31 against 35): the blocks were
  counted, not the line.

Pass 3's cost, from `usage.py` with 845 output tokens for each call without its final usage,
before the session's close:

| Agent | Calls | Without usage | Cost | Findings | Per finding | Duration |
|---|---|---|---|---|---|---|
| `review-verifier`, batch T | 87 | 86 | $3.96 | 36 | $0.11 | 14.9 min |
| `review-verifier`, batch C | 57 | 56 | $2.41 | 32 | $0.08 | 7.9 min |
| The session, so far | 35 | 0 | $3.14 | | | |
| **Pass 3, so far** | 179 | 142 | **$9.51** | 68 | | |

The default of 5,400 tokens would give $17.85: the 845 holds on review agents only, and
batch T's verifier wrote trial tests and scripts, so its share is the least sure. The
`cost-state` record at the end of this session's transcript settles it. Against pass 2's
forecast of about $11 for the two verifiers at $0.16 a finding, they measured $6.37.

The scratch folder was removed by step 10's guarded command (`ls` fails on it); `git diff`
from `docs: close the v2 rebuild roadmap` of 2026-10-12 on `src/`, `tests/`, `Cargo.toml`, `Cargo.lock` and `deploy/` is empty.

**Where the next session starts:**
1. In a fresh usage window, from a new session: read this phase's file, this report and the
   review's report, `docs/reviews/delivered-code/2026-10-15-v2-rebuild.md`, Status pass 4.
2. Read the `cost-state` record at the end of this session's transcript, if the
   session ended cleanly, and split it as pass 1's was: a second measure of the 845 tokens.
3. Pass 4: `review-structure` and `review-performance` at `high`, then their verifiers.
   At the measured rates, about $14: reviewers $3.40 to $11.40 for the two (pass 1 and 2's
   four at $1.71 to $5.68), verifiers $0.08 to $0.16 a finding, the session about $3.50.
   Keep pass 2's guard: if the reviewers and the session pass about $13, or their findings
   price the verification past the window, move it to a session of its own.
4. Pass 4 is the last: step 9 orders and renumbers the findings, fills Coverage and writes
   the fix plan; then the phase's tasks on the user's verdicts, the eight bugs, the false
   positives and the skill's changes.
- Pending: nothing uncommitted, the session's record committed with the review's report
  at its close; no run in progress.

New session, for pass 4. Pass 3's transcript holds no `cost-state` record either: like pass
2's, its session had not ended when this one started, so its exact cost is lost; about
$9.51 from `usage.py`. Only pass 1 is measured exactly. `scope.py` gave the same scope, exit 0; no code changed since
`docs: close the v2 rebuild roadmap` of 2026-10-12. `review-structure` and `review-performance` started at effort `high`, in the
background, each told that the review's report holds 80 findings and to raise a cause
already there only for what its axis adds; `review-performance` was given the timing of
`up`'s poll loops, which pass 1's `review-correctness` left out.

`review-structure` answered in its format after 9.9 minutes (71 tool uses, 241k tokens):
12 findings, 7 on architecture (two ports of five and three roles, `RunStore` a port with no
double, use cases assembled in presentation against `docs/architecture.md`, doctor wiring
the real `sleep`, compare's rules in presentation, the matrix's explicit singletons) and 5
on conventions (the `Cmake` port named after its technology, the hand-written `Refused`,
Podman's and Make's parsers tested on invented outputs, `docs/architecture.md` as of phase
2). It ran no command and wrote nothing in scratch; its checks found the layer imports, the
`.unwrap()` rule, the commit subjects, the language and the crates' phases clean.
`review-performance` answered after 10.6 minutes (59 tool uses, 182k tokens): 9 findings,
6 reproduced by measure: `up`'s 30 s wait is 300 polls with no deadline, about 630 s
against a silent gateway; `rpm -qa` checks every header's digest, 400 ms of `doctor`'s 650;
NVML loaded and shut down at each GPU read; a 10 ms sleep after every command, 87 ms of
`doctor`; `ls-remote` before a fetch that brings the same tags; `podman ps` on an idle GPU.
Three on the bench's prompt fitting are read only (no server here).

Step 7 merged nothing: each new finding that names an existing one (#5, #6, #31, #39, #41,
#52, #74, #79) gives a cause of its own; the verifiers are asked to check it. The guard,
by `usage.py` with 845 output tokens for each of the 113 calls without a final usage: the
two reviewers about $7.15, the session $1.75, so about $8.90, under the $13 limit; 21
findings at $0.08 to $0.16 a finding keep the verification in this session. Two
`review-verifier` started at `high`, one per reviewer, each given its batch as a scratch
file, and the performance verifier the reviewer's examples and builds to rerun.

Batch S's verifier answered after 5.1 minutes (40 tool uses, 111k tokens): 11 CONFIRMED, 1
REFUTED. It ran llama-swap v263 alone on `127.0.0.1:18080` with `python3 -m http.server`
as each model, and captured podman 5.8.7's digest output and a Makefile-generator build
of a small project. It refuted S10: the reading of llama-swap holds (a model in no set
runs alone, and both configurations validate and behave the same), but writing a set
reimplements nothing, the singletons are a recorded choice, and the consequence is
reversed: with no VRAM accounting in the matrix, the explicit singleton keeps a model too
large to share alone if a later llama-swap changes its rule. It narrowed two
consequences by probe (S8: podman's real outputs match the invented ones; S9: the parser
reads real Make output, only the fixture is missing), corrected three causes (S1: doctor
needs four more methods than `is_free`; S2: doctor calls `tool_version` and `is_file`;
S12: the document was updated in phases 3 and 4, four statements lag), and dropped S4's
fallback fix, a concrete `FsRunStore`, which would make presentation or application import
infrastructure. Written as findings 81 to 91 of the review's report.

- **False positive of `review-structure`:** S10, a deliberate, recorded choice read as a
  reimplementation of llama-swap; the reviewer did not run llama-swap, which the verifier
  did. Recorded for the task on false positives.

Batch P's verifier answered after 11 minutes (54 tool uses, 114k tokens): 9 CONFIRMED. It
rebuilt the crate rather than trust the reviewer's `target/`, since the reviewer's `tree/`
already carried the 1 ms `POLL` of its trial, and reran every measure: `doctor` 649 ms
against 563 ms with a 1 ms poll, 660 ms against 337 ms without rpm's digest check, the
same output; `ls-remote` against a local listing after the fetch in a copy of the sources
clone, the same 7,265 tags. It bounded P7's effect from the six saved runs (0.8 % of
prefill's mean power, possibly nil), corrected P9's band (the "50 %" of the comment is
llama-server's `f_keep`, no upper bound), P8's count (34 requests, not 24), P2's share
(about half of `doctor`, not two thirds) and P3's (libcuda loads once), and completed four
fixes: P1's clock needs state a plain `fn() -> Instant` cannot carry; P4's wait after EOF
must stay bounded; P5 changes the `Sources` port, an event and a snapshot. It left one lead,
not verified: `collect()` joins the drain threads with no bound, so a grandchild holding a
pipe outlives a probe's 15 s timeout. Written as findings 92 to 100, then step 9.

Step 9: the 100 findings ordered by action (98 fix, then the 2 kept), security and bugs
first by severity, then error handling, performance, architecture, conventions, dead code,
refactoring and tests, and renumbered by a script that rewrote every cross-reference; two
references split across a line break were fixed by hand. Old to new: 1→2, 2→8, 3→9, 4→10,
5→11, 6→12, 7→13, 8→14, 9→15, 10→1, 11→3, 12→4, 13→5, 14→6, 15→7, 16 to 42→36 to 62,
43→99, 44→100, 45→63, 46→64, 47 to 80→65 to 98, 81→25, 82→26, 83→31, 84→27, 85→28,
86→29, 87→32, 88→33, 89→34, 90→30, 91→35, 92 to 100→16 to 24. This report's entries
above keep the numbers each pass gave. Coverage filled: the three axes of pass 4, the
files no agent opened (34 of 37 fixtures, the two snapshots, the corpus, `Cargo.lock`,
`.gitignore`, `docs/toolchain.md`), what pass 4 read, S10 refuted, and two leads (the
unbounded join, and saved runs embedding `build.json`'s format). The fix plan: 51 batches
in the order to apply them, each finding in one; checked by script, each finding with its
axes, `file:line` and verdict. Status: complete, the plan awaiting the user's approval.

Pass 4's cost, by `usage.py` with 845 output tokens for each call without its final usage,
before the session's close:

| Agent | Calls | Without usage | Cost | Findings | Per finding | Duration |
|---|---|---|---|---|---|---|
| `review-structure` | 69 | 58 | $4.12 | 12 | | 9.9 min |
| `review-performance` | 58 | 55 | $3.04 | 9 | | 10.6 min |
| `review-verifier`, batch S | 40 | 39 | $1.75 | 12 | $0.15 | 5.1 min |
| `review-verifier`, batch P | 50 | 49 | $1.96 | 9 | $0.22 | 11.0 min |
| The session, so far | 42 | 0 | $4.27 | | | |
| **Pass 4, so far** | 259 | 201 | **$15.14** | 21 | | |

The review as a whole: about $51 over four passes (pass 1 exact, $11.89; passes 2 to 4 by
`usage.py`, $14.36, $9.51, $15.14), against the skill's $39 estimate for three passes.
The session's share grew with step 9: renumbering 100 findings, filling Coverage and
writing the plan.

- **Failure from use: step 9 renumbers by hand.** On a review of several passes the
  findings cite each other by number (twenty-odd references here); the session wrote a
  script for it, and the script still missed two references split across a line break. A
  candidate for a bundled script.
- **Failure from use: no place in the template for leads.** Pass 3 and pass 4 each left
  leads that no reviewer raised nor verifier checked (the explicit fetch of `git.rs`, the
  unbounded join, the embedded build format); both went under Coverage, outside the
  template.
- **An agent's scratch artefacts can mislead the next agent:** the reviewer left its
  `tree/` with a trial change in it; the verifier noticed and rebuilt. The skill shares one
  scratch folder among the agents without saying which artefacts are pristine.

Against phase 1's eight bugs, pass 4 adds one: `up`'s 300 polls, which pass 1's
`review-correctness` left out, came back from `review-performance` as finding 16. The
review finds five of the eight; it misses `doctor`'s remedy ending at `publish`'s refusal,
`SessionPlan::peak`'s undercount and the duplicated flags from a model's file keys.

The scratch folder was removed by step 10's guarded command (`ls` fails on it); `git diff`
from `docs: close the v2 rebuild roadmap` of 2026-10-12 on `src/`, `tests/`, `Cargo.toml`, `Cargo.lock` and `deploy/` is empty.

Two tasks ticked, the run and the review's report kept: the phase at 3/10, the README's
block with it (`progress.py --check` clean). The session ended at the user's request after
the review, its record committed with the review's report.

**Where the next session starts:**
1. Read this phase's file, this report and the review's report,
   `docs/reviews/delivered-code/2026-10-15-v2-rebuild.md`, Status complete.
2. The user's verdict on each of the 100 findings and on the fix plan's 51 batches; then
   the phase's tasks: the eight bugs (five found, above), the false positives (four
   refuted findings: the image reference of pass 1, C31 and T21 of pass 3, S10 of pass 4;
   and the facts the verifiers corrected), the confirmed findings listed for a fix roadmap,
   step 4's rates against the four passes' costs, and the changes to the skill this record
   calls for.
3. If the session ended cleanly, the `cost-state` record at the end of this session's
   transcript gives pass 4's exact cost and a second measure of the 845 tokens a call.
- To put to the user: the approval of the whole fix plan, striking what is rejected, and a
  verdict on each finding; the fixes in a separate clean-up roadmap (recommended: this
  phase changes no code of ai-ctl, and 51 batches do not fit one session).
- Pending: nothing uncommitted; no run in progress.

New session, for the user's verdicts. Pass 4's transcript holds no `cost-state` record
either: its last line is the Stop hook's summary at 19:33. Only pass 1's cost is exact.

The findings are put to the user section by section of the fix plan, each with its cause,
fix and verification, and the user gives a verdict on each.

Section 1, security (batches 1 to 5): findings 1, 3, 4, 5, 6 and 7 confirmed. Finding 1's
open choice, an API key on the llama-server backends too or on the gateway only, is left to
the fix roadmap.

- **Failure from use, candidate: the review's report has no place for the user's
  verdicts.** The template's findings carry the verifier's verdict only, and step 9 asks
  for the approval of the whole plan; a verdict per finding, which a fix roadmap starts
  from, has no field.

Section 2, bugs and error handling (batches 6 to 13): findings 2, 8, 90, 9, 10, 27, 11, 12,
13, 62, 14 and 15 confirmed.

Section 3, dead and legacy code (batches 14 and 15): findings 36 to 46 confirmed, by
CLAUDE.md's rule against code written for later. The user asked whether finding 46 needs a
`// LEGACY:` obligation rather than a removal. Removal: neither provider on Fedora 44
serves the old name alone. tuned-ppd 2.28, installed here, serves both (pass 3's
verifier). power-profiles-daemon 0.30, the other provider, which pass 3 could not check,
installs both names' `.service` and `.conf` files: the Arch package 0.30-1's file list,
built by upstream's install; Fedora's spec could not be read (src.fedoraproject.org and
gitlab.freedesktop.org refuse automated requests). The `Missing` arm stays, so a machine
with neither daemon still reads as Missing.

Section 4, tests (batches 16 to 28): findings 65 to 89, 91 to 98, 33 and 34 confirmed.

Section 5, performance (batches 29 to 35): findings 16 to 24 confirmed, finding 20 included
although its gain (1 to 1.6 s a `build`) is the smallest for the change it makes (the
`Sources` port, its fake, an event and a snapshot).

Sections 6 to 9 (batches 36 to 51): structure, findings 30, 59, 61, 29, 31, 32 (its option
(a), `bail!`), 25 and 26 confirmed; refactoring, findings 47 to 58, 60, 63 and 64
confirmed, 55 and 64 although of low gain; documents, findings 28 and 35 confirmed; findings
99 and 100 confirmed as kept. Every finding of the review has the user's verdict: 98
confirmed to fix, 2 confirmed as kept, none rejected, none to decide.

The verdicts went into the review's report: its Status names the plan approved, and the
approval request at the end of the fix plan gave way to the approval, with the three choices
settled (finding 1's key left to the fix roadmap, finding 32's option (a), finding 46 without
`// LEGACY:`). Task ticked: the phase at 4/10.

The eight bugs of phase 1's two runs, against the review: it finds five. `review-correctness`
found four, the symlink (finding 2), `build prune` (9), `bench run` printing before saving
(12) and `Plan::new`'s context (8); it left out `up`'s 300 polls, which `review-performance`
raised in pass 4 (16). No agent found the other three:

- `doctor`'s remedy for a current llama-swap release without its binary, `gateway install
  --version vN`, ends at `publish`'s refusal, since the release's directory exists
  (`doctor.rs:156-160`, `gateway_store.rs:148`).
- `SessionPlan::peak` counts the turns' corpus tokens and the replies, not the `TURN`
  preamble and the template's tokens each message adds (`domain/bench.rs:119`,
  `domain/prompt.rs:128`): a context a few hundred tokens above 22,529 passes the plan and
  overflows at the last turn.
- A model's file keys are checked for a leading dash only, not against the reserved flags,
  and the defaults are checked without the model's file flags (`domain/inventory.rs:332`,
  `:252`): `files = { port = "x" }` gives two `--port`. Findings 85 and 86 test the dash
  and reserved-flag checks of the same code: `review-tests` read beside the bug.

Also missed, the second half of phase 1's bug 4: an error from `started.exited()?` returns
before the stop and leaves the started gateway running and recorded (`lifecycle.rs:612`).
The measure of `review-correctness`: 4 of 8; the review's, 5 of 8. The three missed bugs
hold on the code as it stands: no file of `src/` changed since `docs: close the v2 rebuild roadmap` of 2026-10-12, the code phase 1
read them against. Task ticked: the phase at 5/10.

The user's answers: the three missed bugs and the second half of bug 4 join the list for the
fix roadmap, marked as phase 1's, not as findings of the review; the user knows of no other
miss, having made no full review of the v2 code.

The user then asked which areas the review left unexplored. From its Coverage: every path
through the GPU (NVML's driver/library mismatch: module 615.71.09 loaded, library 615.78.08,
the 615.78 kmod for the running kernel built by akmod after the 12:50 boot, so a reboot
settles it); the three leads no one verified; 34 of 37 fixtures never opened; `deploy/`,
`README.md` and `docs/stack.md` read only at the cited passages, `docs/toolchain.md` not
opened. And three subjects no axis of the skill takes: on-disk formats and their evolution,
interruption and concurrency, the CLI's contract (exit codes, `--json`). The user chose to
check the leads, the fixtures and the shipped files in this session, by reading and probes
without an agent; the GPU paths wait for the reboot.

- **Lead `collect()`: confirmed by reproduction.** In a scratch copy (`git archive HEAD`),
  a test ran `sh -c 'sleep 4 & echo started'`: `execute` with a 1 s timeout returned after
  4.00 s, `SystemPrograms::run` too; the same with the grandchild's output sent to
  `/dev/null`, 10.66 ms. The timeout bounds the direct child only: `collect()`
  (`command.rs:158-162`) joins the drain threads until every writer closes the pipe, so a
  grandchild that keeps it, a daemon say, holds a probe or a smoke test for its whole life.
- **Lead on saved runs: confirmed by reading.** A run of a native build embeds its
  `build.json` (`run_store.rs:404-406`) and is read back through `build_store::from_json`
  (`run_store.rs:446-447`), which refuses any `format` but its own and any unknown field
  (`build_store.rs:39-41`, `:84-89`): a new `build.json` format makes every saved run of a
  native build unreadable, the run's own format unchanged.
- **Lead on `git.rs`'s explicit fetch: confirmed by a probe**, git 2.56.0, a bare repository
  with a commit on a deleted branch: `rev-parse --verify --quiet <hash>^{commit}` finds the
  commit in a `--filter=blob:none` clone, by the promisor's lazy fetch, and not in a full
  clone, where only the explicit fetch brings it; a hash the server lacks fails both ways
  ("not our ref"). In the clones ai-ctl makes, always partial, `resolve`'s explicit fetch
  (`git.rs:98-105`) runs only after the lazy fetch failed, and repeats the same request.
- **Fixtures: captured.** The v2 reports record the captures of all of them: phase 1 the
  machine's outputs (`LC_ALL=C`, the GPU UUID anonymised), phase 2 llama-server's and
  Ninja's, phase 3 llama-swap's (checksums, health, running, unload, logs), phase 4 the
  streams, props, template and tokenize answers; the `.http` files keep their real `Date`
  headers. Tests also give parsers inline text, as empty or malformed inputs beside a
  fixture's case, except, beyond findings 33 and 34, one positive case written by hand: a
  CDI specification in JSON (`host.rs:591`), a format no probe captured.
- **Shipped files: consistent.** `deploy/opencode.json` names the inventory's two model ids
  and address, each context being `ctx_size` less the output (73,728 and 106,496);
  `README.md`'s commands match `cli.rs`, and its exit codes match the code (`doctor` 1,
  untested: finding 75; `status` 3, tested at `tests/cli.rs:281`); `docs/toolchain.md`'s
  install commands match the remedies. One stale reason: Git's minimum, 2.18.0, cites
  `git ls-remote --sort` (`machine.rs:271`), which no code has used since `feat(build): add native builds with list, use and prune` of 2026-10-09; the
  code relies on a partial clone instead, and after finding 20 and the explicit fetch's
  removal, on `git tag --list` and the lazy fetch. The version they need is not checked
  here: git's release notes are the source.

Five items outside the review's findings, put to the user for a verdict. On the side: `/etc/cdi`
is empty and `/var/run/cdi` absent, so `doctor`'s CDI check fails until the reboot; a
machine state, not a finding.

The user confirmed the five: E1 the unbounded join, E2 the embedded `build.json` format, E3
the explicit fetch, E4 the hand-written CDI JSON case, E5 Git's stale minimum.

The input of the fix roadmap is one file, the review's report: its fix plan, 51 batches,
each the findings one change fixes, and a new section after it, `## Found Outside the
Review`, with phase 1's four missed bugs (P1 to P4) and E1 to E5, each beside the batch it
joins, kept apart from the review's findings so that the skill's measure stays its own.
Task ticked.

The false positives and the misses, as input for the skill's changes (task ticked):

| Kind | Agent | Count | Instances |
|---|---|---|---|
| Refuted finding | `review-security` | 1 | the image reference starting with `-`: podman's parser, which the reviewer could have run, refuses it |
| Refuted finding | `review-cleanup` | 1 | C31: two quoting rules, the shell's and llama-swap's splitter's, read as one copy from a repeated-window detector |
| Refuted finding | `review-tests` | 1 | T21: an equivalent mutant read as a missing test; the reviewer did not ask why it survived |
| Refuted finding | `review-structure` | 1 | S10: a recorded choice read as a reimplementation of llama-swap; the reviewer did not run llama-swap |
| A fix that would not work | `review-correctness` | 1 | finding 2's first fix would have broken `is_current` |
| A fix that would not work | `review-tests` | 3 | T2's `trap` brings timing, T3's script makes `/proc/<pid>/exe` name the interpreter, T25's two models fit at once |
| A fix wider than its copies | `review-cleanup` | 8 + 2 | eight helpers narrowed or replaced by existing code (C11, C12, C14 to C19, C23); two copies kept, the shared code costing what it saves (99, 100) |
| A fix incomplete or against a rule | `review-structure`, `review-performance` | 4 | S4's fallback would import infrastructure; P1's clock, P4's wait after EOF, P5's port changes completed |
| A consequence or fact overstated | all six | about 20 | pass 1: 3 consequences, 2 security reaches; pass 3: 7 consequences, T18's input; pass 4: 3 causes, 2 consequences, 5 measures |
| Missed bug | `review-correctness` | 3 + 1 | P1 to P3, and P4, the half of a bug whose other half it left out |
| Lead left without an owner | verifiers, `review-structure` | 3 | E1 (the unbounded join), E2 (the embedded format), E3 (the explicit fetch): seen, written under Coverage, never raised nor verified |
| Missed by a rule's check | `review-structure` | 2 | E4, a parser input written by hand beside its checked fixtures; E5, a comment whose reason no code holds |

What the table points at: a reviewer that does not run what it can run (two of four refuted
findings), a fix proposed without being tried (four that would not work), a consequence
written beyond what was checked (about twenty), and a lead with no axis to go to (three, all
real). `review-correctness`'s three misses sit in domain arithmetic and validation
(`SessionPlan::peak`, the inventory's checks) and in a remedy's path through another
command, code it read in part, before the test modules.

The run's cost per agent, the four passes together (task "Record the run's cost per
agent…"). Pass 1 is split from its exact `cost-state` record; passes 2 to 4 come from
`usage.py` with 845 output tokens for each call without its final usage, their sessions'
records lacking. Step 4's estimate is the skill's as run: a reviewer \$0.25 plus \$0.20 per
1,000 lines, a verifier three quarters of its reviewers, the session \$3 once.

| Agent | Pass | Calls | Cost | Duration | Findings | Step 4's estimate |
|---|---|---|---|---|---|---|
| `review-security` | 1 | 28 | $1.71 | 4.7 min | 7 | $3.05 |
| `review-correctness` | 1 | 51 | $3.89 | 9.5 min | 10 | $3.05 |
| `review-tests` | 2 | 50 | $5.68 | 9.3 min | 38 | $5.37 |
| `review-cleanup` | 2 | 92 | $5.27 | 12.1 min | 32 | $3.05 |
| `review-structure` | 4 | 69 | $4.12 | 9.9 min | 12 | $3.05 |
| `review-performance` | 4 | 58 | $3.04 | 10.6 min | 9 | $3.05 |
| `review-verifier`, security | 1 | 33 | $1.09 | 5.8 min | 7 | $2.29 |
| `review-verifier`, correctness | 1 | 42 | $1.61 | 4.7 min | 10 | $2.29 |
| `review-verifier`, batch T | 3 | 87 | $3.96 | 14.9 min | 36 | $4.03 |
| `review-verifier`, batch C | 3 | 57 | $2.41 | 7.9 min | 32 | $2.29 |
| `review-verifier`, batch S | 4 | 40 | $1.75 | 5.1 min | 12 | $2.29 |
| `review-verifier`, batch P | 4 | 50 | $1.96 | 11.0 min | 9 | $2.29 |
| The sessions | 1 to 4 | | $3.59, $3.41, $3.14, $4.27 | | | $3.00 |
| **The review** | | | **$50.90** | | 106 verified | **$39.10** |

By role: the reviewers $23.71 against $20.63, the verifiers $12.78 against $15.47, the
sessions $14.41 against $3. The rates miss on the sessions, counted once for four, and on
the verifiers' basis: a verifier's cost follows its findings (\$1.09 for 7, \$3.96 for 36),
not its reviewers'. The reviewers' total holds within 15 %, each one between 0.56 and 1.73
times its estimate. The coverage is the review's `## Coverage`, one row per axis.

New rates proposed to the user, not yet approved, to be applied with the skill's other
changes under one `audit.py` and one `skill-auditor` pass: a reviewer \$0.25 plus \$0.23 per
1,000 lines (\$23.5 for the six against \$23.71); a verifier \$2 at step 4, before any
finding exists (the six averaged \$2.13), then priced again at step 8 once the findings are
merged, \$1 plus \$0.07 a finding (a fit on the six), its verification moved to the next
session when the pass would pass the window; each pass's session \$3.50, the last \$1 more
for step 9. On the v2 scope: about \$50 in four passes, against \$50.90 measured. The task
"Record the run's cost…" stays open until the user's yes. This session cost about \$7 by
`usage.py`, no agent started. The session ended at the user's request.

**Where the next session starts:**
1. Read this phase's file and this report from the entry "New session, for the user's
   verdicts". Seven tasks of ten are ticked.
2. Ask for the user's yes on the proposed rates, above; then tick "Record the run's cost…".
3. Put to the user the skill's changes the real run calls for (task "Record the changes to
   the skill…"), from the failures this report records: step 4's rates, with the session
   counted per pass; step 8 pricing the verification and moving it to the next session,
   with a `## To Verify` section in the template; places in the template for the user's
   verdicts, for leads and for defects found outside the review; a route from a lead to an
   axis (E1 to E3 were real); step 9's renumbering by a bundled script; reviewers that
   run what they can run (podman, llama-swap alone on a loopback port), and say a verifier
   may; a proposed test tried against its mutant, a helper checked against each site,
   before they are raised; a merge rule for two reviewers that contradict each other;
   which scratch artefacts are pristine; a verifier's blocks counted, not its summary
   line; `review-correctness`'s misses in domain arithmetic and validation; and the three
   subjects no axis takes (on-disk formats, interruption and concurrency, the CLI's
   contract).
4. Make the changes the user approves as fixes from use: each failure quoted with its
   transcript, fixed at its source, `audit.py`, one `skill-auditor` pass. Then all ten
   tasks are ticked: propose the phase's closure.
- To put to the user: the fixes in a separate clean-up roadmap (recommended), and when to
  create it; the GPU paths, after a reboot, as a targeted pass (about \$5 to \$8) or as
  the fix roadmap's first phase; the proposal to the agent setup on `usage.toml`'s default
  output (845 tokens a call for the review agents), written in the agent setup's `notes/`.
- Pending: nothing uncommitted, no run in progress. The machine needs a reboot: NVML's
  driver/library mismatch, `/etc/cdi` empty and `/var/run/cdi` absent.

### 2026-10-16

New session, for the skill's changes. The proposed rates were put to the user with sixteen
changes, each with a recommendation; the user accepted the rates and every
recommendation: changes 1 to 6 and 8 to 14 made, 7 (a section for defects found outside
the review, specific to this roadmap's measure), 15 (a verifier's summary line, outside its
format already) and 16 (overstated consequences, the verifier's job) left out. On the three
subjects no axis takes, the user wants all three followed although only one has shown a
defect: the review is at its first real run, and the axes may change as they are used.
They join existing reviewers, each the one whose method already holds them:
interruption and concurrency `review-correctness`, whose error handling names what a
failure leaves; on-disk formats and the CLI's contract `review-structure`, which checks
documents against the code. A seventh reviewer stays possible if Coverage shows them
diluted. Task "Record the run's cost…" ticked on the rates' yes.

The user asked whether a session cut during a pass loses the pass's work. As the skill
stands, yes for everything not yet verified: step 8 writes a finding into the report only
with its verdict, the reviewers' answers stay in the session, and the resume path starts
the pass again at step 6, paying its reviewers twice. The answers of the agents that
finished do survive in the cut session's transcript, under `subagents/`, but the skill
does not look there, and an agent running at the cut has none. Change 2 extended: each
reviewer's findings go under `## To Verify` as soon as it answers, `Status` names the
reviewers that answered, and a verdict moves its finding to Findings; a cut then loses
only the agents running.

The changes, made as fixes from use, each quoted from the session of a pass, or from one of its agents.

| # | Failure from use, quoted | Source | Fixed in |
|---|---|---|---|
| 1 | "about $45 with the session counted in each of the three passes" | pass 1's session | SKILL step 2: the measured rates, $3.50 a pass |
| 2 | "Seventy findings put the two verifiers near $11 and the pass near $26" | pass 2's session | step 7 prices the verification and can move it to the next pass; `## To Verify` in the template; step 6 writes each reviewer's findings there as it answers |
| 3 | The user: "Montre-moi l'estimation et les passes avant de démarrer" | pass 1's session | the tier and estimate are step 2, before the scratch folder and the checks |
| 4 | "the review's report has no place for the user's verdicts" | the verdicts' session | `User's verdict` in each finding; the verdicts asked batch by batch after step 10 |
| 5 | "A side benefit: today `collect()` … joins the drain threads with no bound"; "The explicit fetch is redundant, which belongs to the dead-code axis" | an agent of pass 4, an agent of pass 3 | `review-verifier`'s `LEADS` block; step 7 sends each lead to a verifier |
| 6 | "Remove To Verify and move the mutants under Checks" | pass 3's session | the template's "Mutants the agents wrote"; step 8 |
| 8 | "Let the reference regex span line breaks and retest" | pass 4's session | step 6 numbers each finding once; steps 8 and 9 never renumber |
| 9 | "I ran podman 5.8.7 on option-like image words" (REFUTED); "With v263 alone, configs … both pass `-validate`, and they behave the same" (REFUTED) | an agent of pass 1, an agent of pass 4 | a rule in SKILL and in the seven agents: run the program when it needs no GPU and changes nothing outside scratch |
| 10 | "the trap races with the signal"; T21 "the mutant is equivalent" | an agent of pass 3 | `review-tests`: a test tried on the code and its mutant before it is raised; an equivalent mutant is no finding |
| 11 | C31 "these are two rules that look alike, not one rule copied" | an agent of pass 3 | `review-cleanup`: each site compared before a copy is raised, and the shared code limited to what every site does |
| 12 | "No agent found the other three" | the verdicts' session | `review-correctness`: every source of a checked value, each bound against what is built, each printed remedy followed |
| 13 | "The two reviewers clash twice" | pass 2's session | step 7 merges two findings asking opposite changes of one piece of code |
| 14 | "The reviewer's `tree/` already has `POLL` = 1 ms, so I rebuilt" | an agent of pass 4 | step 3 gives each agent its own folder; `review-verifier` builds its own copies |
| 17 | The user, this session: all three subjects to follow | this session | twelve axes: `review-correctness` interruption and concurrency, `review-structure` on-disk formats and the CLI's contract; the verifier's method for each; the template's Coverage; the description; the evals' axis assertions |

`audit.py`: 0 errors, 0 warnings, the three T1 warnings of phase 3 gone. One
`skill-auditor` pass: `VERDICT: FAIL`, two points, both on lines this session did not
write, each fixed at its source: the Agent call's `effort` parameter now cites a probe
(pass 1's Agent calls pass `"effort":"high"`, and each subagent's `.meta.json` records
it); the user's ranking of security cites phase 0's report, "security, the axis that
matters most". `audit.py` again 0 and 0. Checks green: `cargo fmt --check`,
`cargo clippy`, `cargo test` (518, `up_a_proxied_request_and_down ... ok`),
`test_scope.py` (15).

- **The fix-from-use regime's `failures_in_use` has no place yet.** The agent setup's
  decision of 2026-10-15 adds each such failure to the skill's `failures_in_use`, a key
  that authoring-skills' `references/eval-files.md` does not define and `workspace.py`
  would reject; the table above is the record until `lean-workflow` adds it.
- **The description changed** (twelve axes): authoring-skills' proof for a description
  is its trigger queries, run before and after. The before is phase 3's 22 of 22;
  `triggers.py` prices the after at $3.26 (66 sessions), put to the user.

Tasks "Record the changes to the skill…" and "Make the changes the user approves…" ticked:
the phase at 10/10.

The user's answers: the trigger eval runs ($3.26), then the phase closes; the fix roadmap,
the GPU paths after a reboot and the proposal on `usage.toml` come after the closure. On
the rates, the user notes that with a pass's work no longer lost to a cut, they matter
less, though they still guide the split.

The user also doubts the roadmap's end point, one standard review skill for every
repository: the further the work goes, the more each repository's specifics, its
execution context and its language, weigh. This review bears it out. What held only for
ai-ctl: the commands safe to run and the reasons (`src/presentation/cli.rs`, the GPU),
the checks (`cargo audit`, `cargo mutants`), the attackers of a local tool on one
machine, the reviewers' methods written against Rust and ai-ctl's layers, and most of
what the verifiers ran (llama-swap on a loopback port, podman, a crate's copies). What
held for any repository: the orchestration (a scope from commits, tiers, passes within a
usage window, findings under To Verify, a verifier that tries to refute, leads, the
user's verdicts, a fix plan), and the reviewers' disciplines (run what can run, try a
proposed test against its mutant, compare each site before raising a copy). Recorded
under Changes To Later Phases as a restructuring pending approval.

The trigger eval on the new description: 22 of 22, the 11 that should trigger 3 of 3
each, the 11 that should not 0 of 3, $3.27 for 66 sessions
(`.eval-runs/skills/reviewing-delivered-code/triggers/2026-10-16T092800Z.json`); three
calls named the skill's source and were not counted. The same as phase 3's.

The user approved the restructuring, and named the end point: a skill that builds review
skills, from the common part and the part that builds what each repository needs.
Applied to the README, whose scope also gains the three axes of this phase, and to phase 5.
Checks green again before the closure.

---

## Decisions

- **Phase 3's restructuring, approved by the user on 2026-10-15 and applied before this
  phase opened.** Phase 3 stopped on iteration 2 by the stop rule, and this phase's real
  review of the v2 code replaces the v2 case's run with the skill. This phase gained two
  tasks: one `skill-auditor` pass before the run, and the count of the eight bugs. It also
  sets the estimate against the run's per-agent costs, and treats each failure in use as a
  fix from use.
- **One `skill-auditor` pass, not two.** The default regime of the agent setup's decision of
  2026-10-15 applies; `authoring-skills`' `references/audit.md` still describes two passes
  until `lean-workflow` rewrites it. No further pass hunts for a clean sheet.
- **The scratch folder is guarded in the command, not stored in a variable** as the
  auditor proposed: a shell variable does not last from one Bash call to the next, so
  step 2's `SCRATCH=$(mktemp -d)` would be empty by step 10.
- **Pass 1 pairs `review-security` with `review-correctness`.** The skill puts security
  first; bugs come next because they rank with it in the report's order and because the
  count of phase 1's eight bugs depends on them. Passes 2 and 3 were planned as tests with
  cleanup, then structure with performance: the next session may split them again by the
  measured cost.
- **Pass 2 stays as planned, with a guard** (the user, 2026-10-15): `review-tests` and
  `review-cleanup`, then their verifiers. Once both reviewers have answered, their cost
  is measured with `usage.py` at 845 output tokens a call. If they and the session pass
  about $13, the second reviewer's verification moves to the next session, against the
  skill's "each verifier runs in its reviewer's pass". Pass 3 remains `review-structure`
  and `review-performance`, about $12.
- **Pass 2's verification moves to a session of its own** (the user, 2026-10-15). The
  guard's limit held ($12.64), but 70 findings put their verifiers near $11. The review
  becomes four passes: verification of pass 2 (batch T, batch C, about $11 plus the
  session), then `review-structure` and `review-performance` with their verifiers. The
  merged findings wait in the review's report, under `## To Verify`, which the next
  session empties.
- **The review's mutants stay in its report, under Checks** (pass 3). With no
  `cargo mutants` run, the 52 mutants the reviewers and verifiers wrote are the only record
  of what each test finding claims; the fix roadmap checks each new test fails under the
  mutant its finding names.
- **The fix roadmap reads one file, the review's report** (2026-10-15). The user's
  verdicts replace the approval request at the end of its fix plan, and the defects found
  outside the review, phase 1's missed bugs and the checks of the leads, go after it under
  `## Found Outside the Review`, each beside the batch it joins, never among the review's
  findings, so that the skill's measure stays its own.
- **Step 2's rates are the first review's** (the user, 2026-10-16): a reviewer \$0.25 plus
  \$0.23 per 1,000 lines, a verifier \$2 until it is priced from its findings at \$1 plus
  \$0.07 a finding, \$3.50 for each pass's session and \$1 more for the last.
- **The review has twelve axes** (the user, 2026-10-16): on-disk formats, interruption
  and concurrency, and the CLI's contract join the nine, carried by the existing
  reviewers; the axes may change again with use, and a seventh reviewer stays possible.
- **A reviewer's findings reach the report as soon as it answers**, under `## To Verify`,
  and keep their number for the rest of the review: a session cut mid-pass loses only
  the agents running, and no step renumbers.

---

## Files Changed

**Added**
- `docs/reviews/delivered-code/2026-10-15-v2-rebuild.md`
- `docs/roadmap/on-progress/review-skill/phase-4-first-review-report.md`

**Modified**
- `.claude/agents/review-cleanup.md`
- `.claude/agents/review-correctness.md`
- `.claude/agents/review-performance.md`
- `.claude/agents/review-security.md`
- `.claude/agents/review-structure.md`
- `.claude/agents/review-tests.md`
- `.claude/agents/review-verifier.md`
- `.claude/skills/reviewing-delivered-code/SKILL.md`
- `.claude/skills/reviewing-delivered-code/assets/report.md`
- `.claude/skills/reviewing-delivered-code/evals/evals.json`
- `.gitignore` — the line `/notes`, at the user's request (Work Log, pass 1)
- `docs/roadmap/on-progress/review-skill/README.md`
- `docs/roadmap/on-progress/review-skill/phase-4-first-review.md`
- `docs/roadmap/on-progress/review-skill/phase-5-workflow-record.md` — the approved
  restructuring, its overview

---

## Problems And Deviations

- **The estimate and the passes were shown before the checks.** The skill runs step 3's
  checks before step 4's estimate; the user asked to see the estimate before anything
  started, and it depends on the scope alone. No consequence on the review.
- **Step 4's estimate counts the session once**, while a review split into passes takes
  one session per pass. Left open, for the task on the skill's changes.
- **Step 4's rates were recorded as too low, a premise the exact cost refuted:** pass 1
  measured $23.05 by `usage.py`, $14.47 of it on the default output of 5,400 tokens a
  call; its `cost-state` record gives $11.89 against $13.69 estimated. The rates hold on
  the pass's total, but not agent by agent (0.48 to 1.28 times). The task on the run's
  cost sets them once the three passes are measured. `usage.toml`'s default, 6.4 times
  the review agents' output, goes to the agent setup as a proposal.
- **One false positive of `review-security`**, refuted by running podman (the image
  reference starting with `-`). Recorded for the task on false positives.
- **Pass 2's verifiers did not run in their reviewers' pass**, against step 8: the user moved
  them to the next session, since a verifier's cost follows its findings, and step 4 prices
  it by its reviewers' cost. The skill has no place for findings awaiting verification
  between sessions; the review's report holds them under `## To Verify`, outside the
  template. Both left open for the task on the skill's changes.
- **`review-correctness` missed three of phase 1's eight bugs** (`doctor`'s remedy ending
  at `publish`'s refusal, `SessionPlan::peak`'s undercount, the duplicated flags from a
  model's file keys) and left out a fourth (`up`'s 300 polls). Recorded for the task on
  the eight bugs.
- **Pass 2's session wrote no `cost-state` record**: it had not ended when pass 3's
  session started, so its exact cost is lost; about $14.36 from `usage.py`. The next
  session reads this one's record only if it ended cleanly.
- **Two more false positives** (pass 3): `review-cleanup`'s C31, two quoting rules read as
  one copy; `review-tests`' T21, an equivalent mutant read as a missing test. Recorded for
  the task on false positives.
- **`review-tests` proposed tests it had not run** (pass 3): three of its fixes would not
  have caught their mutant, or not without timing; and `review-cleanup` proposed helpers
  wider than its copies share (eight fixes narrowed, two dropped). Left open for the task
  on the skill's changes.
- **Pass 3's session wrote no `cost-state` record either**: only pass 1's cost is exact;
  passes 2 to 4 rest on `usage.py` with 845 output tokens for each call without its final
  usage.
- **One more false positive** (pass 4): `review-structure`'s S10, a recorded choice read as
  a reimplementation of llama-swap. And both pass 4 reviewers stated facts the verifiers
  corrected (doctor's use of two ports, a prompt band, three costs). Recorded for the task
  on false positives.
- **Step 9's renumbering and the review's leads fall outside the skill**: a script written
  in the session renumbered the findings; the leads went under Coverage. Left open for the
  task on the skill's changes.
- **Pass 4's session wrote no `cost-state` record either**: only pass 1's cost is exact.
- **The three leads the review left without an owner were real defects** (E1 to E3), and
  checking the files it read in part found two more (E4, E5); the review's report has no
  place for the user's verdicts nor for defects found outside it. Left open for the task
  on the skill's changes.
- **The skill's changes of 2026-10-16 are unevaluated.** They answer failures of this
  review, quoted, but no run checked them: the next real review is their proof, per the
  fix-from-use regime. Two of the three new axes, interruption and concurrency and the
  CLI's contract, answer the user's statement rather than an observed defect.
- **The closing `skill-auditor` pass found two unsourced claims the opening pass had
  passed over** (the `effort` parameter, the ranking of security), on lines unchanged
  since: the auditor's reading varies between passes. Both fixed at their source.
- **`failures_in_use` does not exist yet** in authoring-skills: the fix-from-use failures
  are recorded in this report's Work Log, 2026-10-16, until `lean-workflow` adds the key.
- **`.gitignore` changed**, outside the phase's Files to Modify: the line `/notes`, at the
  user's request (Work Log, pass 1). No code or configuration of ai-ctl changed: `git diff`
  from `docs(review-skill): close phase 3 on iteration 2 by the stop rule` of 2026-10-15 on `src/`, `tests/`, `Cargo.toml`, `Cargo.lock` and `deploy/` is empty.

---

## Changes To Later Phases

- **Approved by the user on 2026-10-16 and applied** — `README.md` and
  `phase-5-workflow-record.md`: the end point "one standard skill" built from the
  comparison gives way to a skill that builds review skills, from the part every
  repository shares and the construction of the parts each one needs; each repository
  gets a review skill of its own, built by the workflow phase 5 records. In the README,
  the sentence on the comparison and the out-of-scope line on the standard skill; in
  phase 5, "Why This Phase Matters", `### Out of Scope` and the constraint on the
  standard skill. Phase 5's Objective stands: a record that lets the work be done again
  on another repository, marking what was specific to ai-ctl. The user's doubt and this
  review's evidence: Work Log, 2026-10-16.
- `README.md`, its scope: the axes gain on-disk formats, interruption and concurrency,
  and the CLI's contract, the user's decision of 2026-10-16.

---

## Assessment

The skill reviewed the whole v2 code, 14,016 production and 11,591 test lines, in four
passes and about \$51: 100 findings, all confirmed by a verifier and by the user, 98 to
fix and 2 kept, ordered in a fix plan of 51 batches. Four findings were refuted, and
about forty fixes, facts or consequences corrected by the verifiers: verification earned its
cost. The review found five of the eight bugs phase 1's runs had found; its three misses
and three unowned leads, real defects all, pointed at changes to the skill. Those
changes are made from use, unevaluated but for the trigger eval (22 of 22): the review
now has twelve axes, keeps every reviewer's findings in its report as they arrive, and
prices verification from the findings.

What the next phase needs first: this report's Work Log of 2026-10-16 splits what held
for ai-ctl alone from what would hold anywhere, the input of phase 5's record; the
roadmap's end point is now a skill that builds review skills. After this closure, at the
user's word: the fix roadmap from the review's report, a targeted pass on the GPU paths
after a reboot, and the proposal to the agent setup on `usage.toml`'s default output.
