# Phase 2 Report: Skill Build

**Phase:** [phase-2-skill-build.md](phase-2-skill-build.md)
**Start Commit:** `docs: close phase 1 of the review-skill roadmap` of 2026-10-13

---

## Work Log

### 2026-10-13

Phase opened at the end of phase 1's closure (`docs: close phase 1 of the review-skill roadmap`), in the session that closed it, after
reading phase 1's file and report in full; no restructuring was pending. No work on the
phase's tasks yet.

The user then closed the two `/code-review` sessions of phase 1, and their `cost-state`
records give the real cost that phase 1's report left open: $2.82 for the run on `docs: close the v2 rebuild roadmap` of 2026-10-12
(38,948 output tokens, 26,117 of them thinking), $3.13 for the run on `src/` at `high`
(29,424 output, 21,036 thinking). `usage.py` had estimated $9.11 and $7.13: its default of
5,400 output tokens for a subagent call without its final record is eight to nine times the
600 to 700 these calls made. The cost figures of phase 1's Assessment, "$4.51 to about $7" for
a whole-repository review in one session, read $3.13 to $4.51.

At the user's invitation, added these measures to the agent setup's
`notes/2026-10-13-run-estimate-per-case.md`, in a section "Measured Later The Same Day": the
input side and the prices exact, the whole gap in the output estimate, and two observations,
`usage.py`'s command line counting only the transcripts it is given (a session whose work a
slash command forked reads $0) and the `cost-state` record landing only when a session ends.
Recommended there: settle the subagents' output from `cost-state` when a session has one, and a
`--session` option. Nothing was committed in the agent setup, whose `notes/` git ignores. The
user had `/tmp/v2-code-review` and the sessions' scratch folder
(1.1 GB) deleted.

The user asked to record the state and resume in a new session. Where the next session starts:
read phase 1's file and report in full, as this phase's file asks, then this report, then
start the phase's first task: `SKILL.md` from authoring-skills' template for the kind phase 0
chose (task), loading authoring-skills first. The guidance answers the failures F1 to F10 and
F1 to F7 quoted in `evals.json`, and the bugs agent the gaps that phase 2's Dependencies name.

Resumed in a new session. Read this phase's file and report, phase 1's file and report in
full, phase 0's Decisions, `evals.json`, sections 3 and 5 to 10 of `analysis.md`, and
authoring-skills' `SKILL.md`, `references/create-and-edit.md`, `references/writing-guide.md`,
the task template and the `agents` keys of `references/eval-files.md`. `cargo-audit` and
`cargo-mutants` are not installed (`~/.cargo/bin` holds the rustup tools alone);
`docs/toolchain.md` records the driver, CUDA and compilers only, so the two installs go in
this report, as the phase's task says.

Reported to the user what carries into the session, and put three points no phase had
settled. Phase 0 set the agents' number "after the baselines", and neither baseline started
an agent, so the runs say nothing of which agents overlap; what phase 1 added is the plan's
cap of about $20 per usage window, against a pass over `src/` measured at $3.13 to $4.51.
Nine reviewers over the whole repository pass one window, and phase 3's v2 case, at `high`
under an $18 ceiling, would be cut. The user chose six reviewers and a verifier, a review
split into passes when its estimate passes a window, and `scripts/scope.py` as the only
script. Settled without a question, as the intent already rules: a reproduction may run
`cargo test` and its integration test, which starts the installed llama-swap with a fake
llama-server and no GPU; never `ai-ctl up`, `down`, `build`, `gateway install` or `init`.

Installed the two tools phase 0 approved, with `cargo install --locked cargo-audit
cargo-mutants`: **cargo-audit 0.22.2** (`cargo audit --version` prints `cargo-audit-audit
0.22.2`) and **cargo-mutants 27.1.0**, both in `~/.cargo/bin`. Read the agents page of
Claude Code (https://code.claude.com/docs/en/sub-agents): an agent's `effort` field is
overridden by the `effort` parameter of the call, an agent loads `CLAUDE.md` unless
`omitClaudeMd` is set, and one whose `tools` leave out `Agent` cannot start agents of its own.

Wrote `evals/test_scope.py` first, fifteen tests on throwaway git repositories, and watched
all fifteen fail on the missing script (`audit.py` puts every `test_*.py` in `evals/`, its
rule C2); then `scripts/scope.py`, and the fifteen pass. On ai-ctl, `scope.py roadmap
docs/roadmap/completed/v2-rebuild` gives what the v2 case's expected output says: phase 0's
`1a49576` absent, `chore: restart llm-ctl v2 on a clean base` of 2026-10-09 the root commit, so the whole history up to `docs: close the v2 rebuild roadmap` of 2026-10-12, the last
commit touching the folder; 120 files, 14,016 production lines and 11,591 test lines.

Measured the two tools on the post-session fixture, built in this session's scratch folder
by `evals/fixtures.py`. `cargo audit` took 7.4 s, scanned 66 dependencies against 1,310
advisories, found none, and fetched its database to `~/.cargo/advisory-db` (49 MB), the
tool's cache. `cargo mutants --list --in-diff` gives 25 mutants for the session's diff and
1,931 for the whole crate; the 25 ran in 68 s, `-o` and `TMPDIR` in the scratch folder, none
left in `TMPDIR`: 16 missed, 5 caught, 4 unviable, exit code 2 (mutants missed). The missed
ones fall on planted defects 7 (`newest` replaced by `None`) and 10 (the adapter's
`releases`, `size` and `remove`, and the use case's arithmetic, which a test of `is_ok()`
alone cannot catch). At that rate the whole crate takes about 90 minutes. `cargo test` prints
`test gateway_lifecycle ... skipped: <reason>` when that test skips itself
(`tests/gateway_lifecycle.rs:68`).

Wrote `SKILL.md` from the task template, `assets/report.md`, and the seven agents in
`.claude/agents/`, then added their paths to `evals.json` under `agents`; `workspace.validate`
reports no problem. The steps answer the failures one by one: the scope from `scope.py`, tests
included (F2, F3 of the v2 case); one scratch folder, removed and named in the account (F10);
`cargo audit` and `cargo mutants` with the threshold measured above (F8; F7 of the
post-session case); the tier table, the estimate and the passes (F6, the user's choices); a
verifier per tier (F7); findings merged by root cause and split when two causes need two fixes
(F5 of the post-session case); a coverage row per axis, files named as read only from the
agents' lists (F4, F5); a fix plan in numbered batches (F9; F2 of the post-session case); the
report as the only file written (F1). The agents carry the axes: the tests agent reads every
test in full and names the change each test lets pass (F3; F4 of the post-session case), the
cleanup agent shares a copied function rather than copy it again (F3 of the post-session
case), the correctness agent reads the tests and follows a failure to what it leaves behind,
which the v2 baseline missed where `/code-review` found it, with no cap on findings; the
security agent carries the vulnerability-scanning plugin's method in the skill's own words, no text copied, on
the threat model of a local single-user tool.

Then checked each block against the failures and the user's statements, and cut what traced to
neither: `review-correctness` named second in the passes, the reviewers started "all in one
message", the list of refuted findings in the report, one roadmap phase per batch, "code more
complex than the work it does", the error and test rules `CLAUDE.md` already gives every agent
(an agent loads it, per the agents page), the swallowed result and the unhelpful message (both
baselines found them unprompted), the performance agent's paragraph on how to give a cost, and
"the axis the user ranks first" in the security agent. `audit.py` reports no error and three
warnings, all on the word `legacy`: it names the user's axis and rule, not compatibility, and
stays.

Gave the skill, its seven agents and the audit's output to the `skill-auditor` agent. It
answered `VERDICT: FAIL` with four problems, each holding: the mutants figures, their 300
threshold and the meaning of exit code 2 given with no source; the $20 window with no source;
the scratch folder removed in prose rather than by an exact command; and the verifier's "Do not
invent a defense", a prohibition no failure called for. Fixed all four: the probe cited with its
date and version, the threshold derived from it (about 15 minutes), the exit codes linked
(https://mutants.rs/exit-codes.html), the window's source named, `rm -rf <scratch>` on the path
`mktemp -d` printed, the prohibition cut. Found while fixing, a defect of my own: a pass that is
not the last stopped at step 8 and never reached step 10, leaving its scratch folder, and a
later pass would have lost the missed mutants that lived there. A pass now ends at step 10, a
continued review creates its own scratch folder, and the missed mutants are copied into the
report's Checks section, a new slot of `assets/report.md`.

Sent the fixes back to the same `skill-auditor`, which answered `VERDICT: PASS`. `audit.py`
reports no error; `evals/test_scope.py` passes its fifteen tests. The security agent's method
is taken from `analysis.md`'s account of the vulnerability-scanning plugin (sections 1.3 and 2.7), already in this
roadmap's words; the plugin's own files, at 0.11.0, were not read again, so no text
of theirs could pass into the agent. The seven agents cannot be started from this session: its
`.claude/agents/` folder did not exist when the session began, and Claude Code watches only
the folders that did (agents page); phase 3's runs are new sessions and see them.

Adjusted phase 3 for what this phase decided: a constraint on the v2 case, which at `high` is
estimated above the $15 of one pass, and a task counting, in the v2 case's runs with the skill,
which of the eight bugs of phase 1 are found again. Ticked the eight tasks, and with them the
three acceptance criteria, all three met: a replacement meant for the tasks reached them too,
in this phase's file alone.

Reported to the user; they asked to close the phase and to start phase 3 in a new session.
Two statements above proved wrong before the closure. The seven agents became available to
this session after all: the harness listed them later in the session, so its watcher picked
up the new `.claude/agents/` folder. And `cargo test` does not print `test gateway_lifecycle
... skipped`: the harness prints its test's name, `up_a_proxied_request_and_down`
(`tests/gateway_lifecycle.rs:40`, `:68`), found by running `cargo test --test
gateway_lifecycle` during the closure's checks. Fixed in `SKILL.md`, step 3; `audit.py` still
reports no error. The three checks are green: `cargo fmt --check` and `cargo clippy
--all-targets -- -D warnings` clean, `cargo test` 492, 23 and 2 tests passed, the lifecycle
test run (llama-swap is installed).

---

## Decisions

- **Six reviewer agents and one verifier, the nine axes grouped by the reading they need.**
  `review-structure` (architecture, conventions), `review-cleanup` (dead and legacy code, code
  for later, refactoring: each searches the whole repository for callers and copies),
  `review-correctness` (bugs, error handling), `review-security`, `review-tests`,
  `review-performance`, and `review-verifier`, which tries to refute a finding. The reason is
  the cost the user's plan allows, not an overlap the runs showed: nine reviewers over the
  whole repository cost about $27 to $36. Phase 3 splits an axis back out when its runs show
  it thin. Set aside: nine reviewers (the analysis's section 7.5) and three (briefs so wide
  that the axes get lost, failures F3 and F5).
- **A review whose estimate passes a usage window runs in passes, one per window.** Each
  verified finding is written into the report as it comes, and the next pass continues from
  the report. Set aside: tiers sized to fit one window (`high` covering less on a large scope)
  and the estimate alone (a session cut by the limit loses its agents' work). Phase 3's v2
  case, at `high` under an $18 ceiling, runs one pass: its prompt is reworded there, or its
  run recorded as a partial observation.
- **`scripts/scope.py` is the skill's only script.** It reads a roadmap's Start Commits,
  names those the history lacks, gives the range, and counts the scope's production and test
  lines, the base of the estimate; failure F2 of the v2 case becomes a command's output.
  Set aside: a script that lists `// LEGACY:` markers (a grep does it, and no run needed it)
  and no script (F2 left to the agent's reading).
- **A reproduction may run `cargo test`, its integration test included**, which starts the
  installed llama-swap with a fake llama-server and no GPU, as the v2 baseline did; it never
  runs `ai-ctl up`, `down`, `build`, `gateway install` or `init`, which phase 0's intent
  rules out.

---

## Files Changed

**Added**

- `.claude/agents/review-cleanup.md`
- `.claude/agents/review-correctness.md`
- `.claude/agents/review-performance.md`
- `.claude/agents/review-security.md`
- `.claude/agents/review-structure.md`
- `.claude/agents/review-tests.md`
- `.claude/agents/review-verifier.md`
- `.claude/skills/reviewing-delivered-code/SKILL.md`
- `.claude/skills/reviewing-delivered-code/assets/report.md`
- `.claude/skills/reviewing-delivered-code/evals/test_scope.py`
- `.claude/skills/reviewing-delivered-code/scripts/scope.py`
- `docs/roadmap/on-progress/review-skill/phase-2-skill-build-report.md`

**Modified**

- `.claude/skills/reviewing-delivered-code/evals/evals.json`
- `docs/roadmap/on-progress/review-skill/README.md`
- `docs/roadmap/on-progress/review-skill/phase-2-skill-build.md`
- `docs/roadmap/on-progress/review-skill/phase-3-evaluation.md`

---

## Problems And Deviations

- **The agents' number could not be set from the baselines, as phase 0 decided**: neither
  baseline started an agent, so the runs show no overlap between axes. Set with the user on
  the cost the plan allows instead (Decisions); phase 3 splits an axis back out if its runs
  show it thin.
- **The v2 case cannot run whole at `high` within one window.** Its estimate passes one pass,
  so phase 3's run under an $18 ceiling covers the first pass alone. Moved to phase 3 as a
  constraint: reword the prompt with the user, or record a partial observation.
- **A pass that was not the last left its scratch folder** in the first draft of `SKILL.md`,
  and lost the missed mutants a later pass needs. Found while fixing the auditor's problems,
  fixed before its second verdict.
- **The acceptance criteria were ticked with the tasks**, by one replacement over the phase
  file, against the roadmap skill's rule of one exact replacement each. All three are met; no
  other file was touched by it.
- **`SKILL.md` quoted a line `cargo test` never prints**, `test gateway_lifecycle ...
  skipped`, written from the harness's code without running it, and passed both audits.
  Found by running the test during the closure; fixed in step 3 before the commit.
- **No run with the skill yet**: the phase builds the skill, and its proof, the runs against
  the baselines, belongs to phase 3, as planned. The verifier's cost, counted as half a
  reviewer in the estimate, has no measure until then.

---

## Changes To Later Phases

- `phase-3-evaluation.md`: a constraint added, the v2 case at `high` running one pass under its
  $18 ceiling, with the two ways out (a reworded prompt, which loses its baseline for
  `--reuse`, or a partial observation); the reason is the passes decided in this phase.
- `phase-3-evaluation.md`: a task added, counting which of the eight bugs phase 1's two runs
  found in the v2 code the runs with the skill find again, the measure phase 2's Dependencies
  set for the bugs agent; the task count goes from 8 to 9.
- `phase-3-evaluation.md`: its Dependencies name the agents in `.claude/agents/` and the
  `agents` key of `evals.json`.

---

## Assessment

The phase delivered what it promised: `reviewing-delivered-code` in `.claude/skills/`, with
its report template and `scripts/scope.py`, whose fifteen tests were written first and seen
failing; seven agents in `.claude/agents/`, listed in `evals.json`; `cargo-audit` 0.22.2 and
`cargo-mutants` 27.1.0 installed and measured on the post-session fixture. `audit.py` reports
no error, and `skill-auditor` answered `VERDICT: PASS` on its second pass. Each step of the
skill answers a quoted failure or a statement of the user, and what traced to neither was cut.

Three points were settled with the user, since phase 0 had left them to the baselines and the
baselines had started no agent: six reviewers and a verifier, a review in passes when its
estimate passes one usage window, and `scope.py` as the only script. The measures say where
the skill's weight lies: `cargo mutants` on the planted session's diff names, for nothing in
tokens, the adapter's untested functions, the use case's unchecked arithmetic and the dead
`newest`, though not the domain test built on three-digit releases, which stays the tests
agent's to find; a pass over the whole
repository costs about $4.50 per reviewer, so the v2 case at `high` needs several windows.

What phase 3 needs to know first: the skill has never run. The v2 case at `high` does one
pass under an $18 ceiling, so its prompt is reworded with the user or its run recorded as a
partial observation, before the paid runs are announced. The post-session case fits one pass
and is the first measure of the whole procedure. The estimate's verifier figure is a guess
until those runs, and the eight bugs of phase 1 are the measure of `review-correctness`.
