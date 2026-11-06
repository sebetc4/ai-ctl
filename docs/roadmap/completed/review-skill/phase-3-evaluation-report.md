# Phase 3 Report: Evaluation

**Phase:** [phase-3-evaluation.md](phase-3-evaluation.md)
**Start Commit:** `feat(review-skill): build reviewing-delivered-code and close phase 2` of 2026-10-13

---

## Work Log

### 2026-10-13

Phase opened at the end of phase 2's closure (`feat(review-skill): build reviewing-delivered-code and close phase 2`), in the session that closed it, after
writing and reading phase 2's file and report in full; no restructuring was pending. No work on
the phase's tasks: the user starts it in a new session.

Where the next session starts: read phase 2's file and report in full, as this phase's file
asks, then this report. Before announcing any paid run, settle with the user the v2 case at
`high`, which the skill splits into passes above one usage window: reword its prompt (a new
digest, no `--reuse` of its baseline) or accept its run as a partial observation, per this
phase's Constraints. The post-session case fits one pass and is the first measure of the whole
procedure. Each paid run starts in its own fresh window, from a new session, with `run.py
--case` and `--run`; `iteration-1` holds the baselines in `.eval-runs/`, which git ignores.

Resumed in a new session; the opening above is not committed yet. Read phase 2's file and
report in full, this phase's file and report, and phase 1's Decisions and Assessment. Worked
out the skill's own estimate for the v2 case at `high` (step 4 of `SKILL.md`, scope 14,016
production and 11,591 test lines): five reviewers at $4.50, `review-tests` at $7.40,
six verifiers at half a reviewer, $14.96, and $2 for the session, about $47, four passes of
$15. The first pass holds `review-security` and its verifier ($6.76) with the session's $2;
a second reviewer and its verifier would bring it to $15.52. So a run of the case as it
stands reviews security alone, and the count of phase 1's eight bugs, which needs
`review-correctness`, cannot come from it. Put to the user, who chose to keep the prompt,
reuse the baseline and record the run as a partial observation, then measure the eight bugs by
continuing the review from the run's report in a later window.

Loaded authoring-skills and read its `references/evaluate.md`. Prepared `iteration-2` with
`workspace.py --reuse iteration-1 --runs 1 --model claude-opus-5-5 --effort xhigh --budget
18`: one run with the skill per case, the two baselines reused, the cases' digests unchanged.
`run.py` estimates $2.90 a run, the mean of the baselines, which started no agent: it says
nothing of a run with six reviewers. Computed the skill's own estimate for the post-session
case instead: `scope.py range` from `docs: close the v2 rebuild roadmap` of 2026-10-12 to the fixture's session commit, `01b2813`, gives 2,211 production and 1,152
test lines, so five reviewers at $1.55, `review-tests` at $1.84, six verifiers at $4.80 and
$2 for the session, $16.41. That passes $15, so the skill splits the case into two passes:
the first with five reviewers ($14.08), the second with `review-performance` alone. Phase 2's
Assessment said the case fits one pass; by the skill's own formula it does not. Run as it
stands, the case ends after its first pass, with no merge of findings, no fix plan, a partial
Coverage and no performance review, so assertions 1, 2, 7 and 9 would fail by construction.
The $1 a reviewer the formula starts from has no measure; the one agent measured, reading
14,016 lines at `high`, cost $3.13 where the formula gives $4.50. Put to the user, who chose
to run the case as it stands and to recalibrate the formula from the run's measures.

Where the next session starts: in a fresh usage window, from a new session, start the
post-session run with `run.py .eval-runs/skills/reviewing-delivered-code/iteration-2 --case
post-session-review --start` (the user's yes is given; the baselines took 5 and 8 minutes, well
within the default 120-minute timeout). Then read its `run.json`, whose `agents` key gives each
agent's real cost, and its outputs, before grading.

The post-session run ran from 19:49 to 20:02 UTC, status complete, $8.31 against the $14.08
the skill estimated for its first pass. Resumed in a new session, which read this phase's file
and report, phase 2's file and report in full, authoring-skills' `references/evaluate.md`, the
run's `run.json`, its closing account and its report. The run did what the skill says for a
pass that is not the last: scope from `scope.py range HEAD~1`, the five checks (`cargo test`
522 passed, `cargo audit` clean, `cargo mutants` 25 mutants, 16 missed, copied into the
report), the tier and its two passes, five reviewers and five verifiers at `high`, 41
findings all `CONFIRMED`, most reproduced by both the reviewer and the verifier, a Coverage
row per axis with performance left to pass 2, `Status` naming pass 2, the scratch folder
removed, and the report as the only file changed. No merge, no ordering by severity and no
fix plan, which the skill keeps for the last pass, as recorded before the run.

Measured per agent, against the skill's formula: `review-security` $0.32, `review-correctness`
$0.42, `review-cleanup` $0.48, `review-structure` $0.85 (formula $1.55 each), `review-tests`
$0.88 (formula $1.84), the five verifiers $2.43 together, $0.49 each on average (formula
$0.78 to $0.92), and the session itself about $2.92 (formula $2). The formula overrates the
pass 1.7 times, and its $1 base per reviewer dominates on a small scope. Three observations
for the iteration, before any grading: the run says pass 2 would have fit in its session and
stopped anyway, since the skill asks for one pass per session; the same cause comes from up to
three reviewers and is verified up to three times (the unquoted `rm -rf` is findings 1, 11 and
21, with severities medium and high), so 41 findings hold about 23 causes; and the domain test
on three-digit releases of planted defect 10 is not named as such, the run putting the gap on
the adapter's `releases` (finding 34) and a domain test with v99, v263 and v1000 in finding
20's fix.

`grade.py` on `iteration-2` lists three runs to grade, the two reused baselines and the
post-session run with the skill (the v2 run, not started, left out): three sessions of
`skill-grader` at Sonnet `xhigh`, estimated $1.35, $0.45 a session from `skill-auditor`'s
runs, under a default cap of $1 a session. No grading exists in `iteration-1`. The run with
the skill holds ten agents' transcripts (2.3 MB, 1.3 MB for its own), which the assertions
on claims, scratch and stack commands make the grader read; a grader stopped at its cap is
recorded `stopped` and started again by the next `--start`, its spend lost. Put to the user
with `--budget 3` (at most $9, about $2 to $4 expected) and the window already holding the
run's $8.31: the user chose to wait for the next usage window.

Where the next session starts: in a fresh usage window, from a new session, start the grading
with `grade.py .eval-runs/skills/reviewing-delivered-code/iteration-2 --start --budget 3`
(the user's yes is given), then `benchmark.py` and `viewer.py` on the same iteration. Before
reading the gradings, note this report's reading of the run (the three observations above,
and assertions 1 and 9 expected to fail). `run.json` gives `duration_s` 120.6 and `--status`
2m00s for this 13-minute run: take its duration from `started` and `ended`.

### 2026-10-14

Resumed in a new session at 10:15, twelve hours after the last commit, so in a fresh usage
window. Read this phase's file and report, phase 2's file and report in full, and
authoring-skills' `references/evaluate.md`. `grade.py` listed the same three runs (the v2 run
with the skill, not started, left out), estimated at $1.35; started with `--start --budget 3`,
the user's yes given in the previous session.

While the graders ran, wrote the trigger queries under `triggers` in `evals.json`: eleven that
should trigger, in the user's words (the v2 roadmap, a post-session review, technical debt, a
review in progress, one in English, tiers named or not), and eleven near misses, each with its
`near`: a session's summary and "relis ce que cette session a livré", as phase 0 decided, the
built-in `/code-review` on a working diff, `/security-review`, `/simplify`, a skill's audit, a
phase's closure, `tool-review`, reading code to understand it, debugging a test the last
session broke, and the checks alone. `audit.py` reports no error and the same three warnings
on `legacy`. `triggers.py` lists 66 sessions (22 queries, three each) at Opus 5.5 `xhigh`,
estimated $3.37, under a $0.50 ceiling a session: not started, waiting for the user's yes.

The two baselines' gradings came first, both `complete` at Sonnet `xhigh`, about $0.25 each:
the post-session baseline passes 3 of 12 (claims, scratch, no stack command), the v2 baseline
1 of 12 (no stack command), each failure matching the failures quoted in `evals.json`. The
grader of the v2 baseline names two assertions weak, though both failed there: Axes, which a
report with no finding would pass, and Claims hold, which a run claiming few reads would pass
without having opened what a review needs.

The post-session run with the skill passes 9 of 12, its grading $0.67, the three gradings
$1.13 together against $1.35 estimated. Against this report's reading before the gradings
(assertions 1, 2, 7 and 9 expected to fail), two predictions held and two did not. Assertion
2 passes: `?` without context (finding 24) and `size` reading whole files (finding 3) are two
findings. Assertion 7 passes: the Coverage section has a row per axis, performance "not yet:
pass 2". Assertion 9 fails by construction, the Fix Plan saying it is written in pass 2.
Assertion 1 fails on nine of eleven defects, but not where expected: the performance defect
(11) is found, by `review-security`, filed under security with `metadata()` as its fix. The
two defects missing a part are 8, the `clean` alias found without saying it lacks a `//
LEGACY:` comment, and 10, the domain test on three-digit releases never named as the reason
the ordering bug passed (finding 32 criticises that test on another ground, finding 20 only
proposes a v99, v263 and v1000 test), the same gap as the baseline's F4.

Assertion 10 fails on one claim: the Coverage row says `review-structure` read `CLAUDE.md` in
full, while its only call on the file is `sed -n 35,60p`. Checked before the skill, as
`evaluate.md` asks: the agent's transcript holds `CLAUDE.md` whole, in an attachment of type
`instructions` at its fifth line, loaded by the harness (each of the ten agents has it). The
claim holds; the assertion's measure, "opened by a call", does not count what the harness
loads, and the rendered `agents/*.md` the grader reads does not show that attachment.

`benchmark.py`: pass rate 75% with the skill against 17% without (+58 points), $8.31 against
$2.90 a run (+$5.42). Assertions 11 and 12 pass in every run and do not tell the
configurations apart, as do 1 and 9, failed in every run. The benchmark's duration for the
run with the skill, 120.6 s, is the `run.json` figure already noted as wrong (13 minutes from
`started` and `ended`); its output counts 123 agent calls at 5,400 tokens each, the estimate
phase 2 measured as eight to nine times too high. `viewer.py` serves the three runs at
`http://127.0.0.1:37709/` for the user's feedback.

Put the trigger eval to the user with its ceiling: `trigger_stop.py` stops a session at the
skill's first use, so only the 33 sessions of near misses answer in full, at most $0.50 each,
about $17 at worst against $3.37 estimated, with about $3 already spent in this window. The
user chose to run it now and to read the review page meanwhile; `triggers.py --start` started.

The trigger eval passed every query: the eleven that should trigger at 3 of 3, the eleven near
misses at 0 of 3, the session's summary and "relis ce que cette session a livré" included;
66 sessions, $3.26 against $3.37 estimated (`triggers/2026-10-14T084523Z.json`). `tuning.py
split` (seed 42) put 14 queries in training and 8 held out, four of each kind in the held-out
set; `score` gives the current description, 544 characters, 14 of 14 on training and 8 of 8
held out, and the loop stops there, nothing left to rewrite from. The description stays as it
is. Ticked the two Triggering tasks.

The user's feedback on the review page, given in the conversation: the report is relevant and
shows the skill doing the expected work where the run without it does not. The case's two
`review` items, the order by severity and the batches of the fix plan, cannot be judged on
this run: pass 1 orders nothing and writes no plan.

Asked the user one question whose answer changes the iteration: merge the findings sharing a
cause in every pass, before verification, or keep the merge for the last pass. The user chose
the merge in every pass, each merged finding keeping the axes and reviewers that raised it.
On defect 8, explained to the user what the run did: `review-cleanup` cloned the v1 bundle
`CLAUDE.md` names, found with `git grep` on both branches that llm-ctl never had a `clean`
command, and so showed the fixture's own pretext false, which its verifier confirmed; the
finding reaches the expected conclusion, no obligation and the alias removed, and the missing
`// LEGACY:` comment, owed only by legacy code that stays, is moot for code to remove.

Read authoring-skills' Edit section, then changed the skill for iteration 3, the previous
version being commit `docs(review-skill): record the post-session run and prepare its grading` of 2026-10-13:

- `SKILL.md` step 7 merges the reviewers' findings by root cause before any verification,
  each merged finding keeping every place, axis and reviewer and each axis's consequence; a
  later pass merges into the report's findings and verifies only what it adds. Step 9 keeps
  the order and numbers the findings in it; the tier table's `high` shares the merged
  findings among one verifier per reviewer, by axis. `assets/report.md` gives `Axes: AXES —
  raised by REVIEWERS`, and `review-verifier` takes the method of each axis a finding names.
  This answers F2 of the run, 41 findings for about 23 causes.
- Step 4's estimate is recalibrated from the run's measures and phase 1's: a reviewer $0.25
  plus $0.20 per 1,000 lines, a verifier three quarters of its reviewers, the session $3.
  The post-session case now estimates at about $10.66, one pass; the v2 case at `high` at
  about $39, three passes, its first holding `review-security` and one more reviewer with
  their verifiers. This answers F1, the case split by an estimate 1.7 times too high.
- `review-tests` gets a step 4: for each test, what its inputs all share and which defect only
  an input outside that shape would show, named as the reason the defect passes. This answers
  F3, defect 10's domain test on three-digit releases, missed by both the baseline (its F4)
  and the run with the skill.

Changed the evals with the user's agreement on the reasoning: assertion 1 counts defect 8
when the finding says no obligation justifies the alias and proposes removing it, and the
expected output says the alias is removed rather than marked; Claims hold, in both cases,
counts a file the harness loaded whole into an agent's context, as `CLAUDE.md`, an
`instructions` attachment in the agent's transcript under `subagents/`. Both measures apply
to both configurations: the reused baselines are graded again in iteration 3. Recorded the
three failures with the skill under `failures_with_skill`, and the case's `pending` now names
the order by severity and the fix plan, which no run has shown. `audit.py` reports no error,
the same three warnings.

Prepared `iteration-3` with `workspace.py --reuse iteration-2 --runs 1 --model
claude-opus-5-5 --effort xhigh --budget 18`: one run with the skill per case, the two
baselines reused (copied without their gradings). `run.py` lists two runs at $3.67 each, the
mean of seven runs whose five baselines started no agent; the skill's own estimate is the
figure to read. The window already holds this session's spend, the gradings' $1.13 and the
trigger eval's $3.26: the post-session run waits for a fresh window.

The user questioned the defect 8 change: does the agent propose to keep the alias, and is
the tool or the validation at fault? Answered that the agent removes the alias and the fault
lay in the assertion, which counted a part of the expected output without substance; no
instruction was added to the skill for it. Reviewed the session's other changes by the same
test: "Claims hold" was likewise changed in the assertion alone; the merge and the estimate
answer failures in use; `review-tests`' new step is the one instruction aimed at a failed
assertion, defect 10, whose gap is real and general, its risk being noise.

The user then pointed to the agent setup's
design-feedback note on close-session's validation, of 2026-10-14, which records the same
pattern in the `session-close` roadmap: work bending the skill under evaluation around the
gaps of authoring-skills' validation. The user proposed to pause this roadmap, fix the
validation at its source, then resume, with no compromise or workaround. Read the note, then
checked in skill-tooling 0.1.2's code each point before writing ai-ctl's answer,
the agent setup's design-feedback note on reviewing-delivered-code's validation, of 2026-10-14.
What the checks showed:

- `harness.render` (`harness.py:288-308`) keeps only `user` and `assistant` records, so the
  grader never sees the `instructions` attachments; the "Claims hold" change of this session
  is a workaround of that gap, not a fix.
- `duration_s` comes from the json result's `duration_ms` (`harness.py:206-208`): 120.6 s
  for a 13.5-minute run, so the benchmark's "-260.3 s a run" has the wrong sign.
- `run.py`'s `counted` (`:284-312`) settles the reported output over the agents' calls by
  call count: the per-agent costs this session recalibrated the estimate from hold exact
  input but an estimated output share, which nothing in `run.json` marks.
- The benchmark's footnote on estimated output ignores `output_source` (`benchmark.py:142`,
  `:330-333`), and its note "sharpen it or drop it" (`:218`, `evaluate.md:101-102`) falls on
  guard assertions too.
- The three source citations of `SKILL.md` point into this roadmap, which the fixture keeps
  out of every run (`evals/fixtures.py:7-8`): no run can open them, and one was rewritten in
  this session by the same rule.

The note lists what ai-ctl carries because of the validation, to revert once the tool is
updated, and what stands on failures in use. Phase 3 pauses here, at the user's request.

At the user's request and with their leave, set the phase's status to ⏸️ Blocked and its
`**Blocked By:**` to the validation's update, in this file and in the README, whose progress
block comes from `progress.py`. Reverted the "Claims hold" clause on harness-loaded files in
both cases of `evals.json`: a workaround stays out of the tree, and the note lists it. Stopped
the review page. Committed the skill's changes for iteration 3 apart from the pause's record.

Where the next session starts, once authoring-skills is updated: read this phase's file and
report in full, then ai-ctl's note in the agent setup and the update's account (its changelog
and the decisions it took on the two notes). Then, with the user, go through the note's list
of what ai-ctl carries because of the validation, one item at a time: the source citations of
`SKILL.md`, the estimate's rates measured again from exact per-agent figures, the v2 case's
partial observation, the Phase 2 cuts drawn from the study. Revert or redo each by what the
updated tool allows, and apply its new rules to the cases (kinds of assertion, failures
classified, several sessions). `iteration-3` was prepared with the "Claims hold" clause since
reverted and under skill-tooling 0.1.2: prepare a new iteration rather than run it. The
skill's changes of this session (the merge before verification, the estimate, `review-tests`'
step 4) are committed and unevaluated; the next run with the skill is their proof. Clear the
`**Blocked By:**` and set the status back to 🟡 when work resumes.

### 2026-10-15

Resumed after skill-tooling 0.1.2's release. The resume followed the agent setup's
design-feedback note on resuming the review skill, of 2026-10-15, and its decision
`docs/decisions/2026-10-15-validation-weight.md`: the proof follows the tool's risk, real use
is the main loop, and this skill falls under the default regime. Read this phase's file and
report and phases 4 and 5's files. `audit.py`: 0 errors and the three T1 warnings on
`legacy`, which here names the axis "dead and legacy code", not wording about earlier
versions. That is a false positive of the rule. It is reported in the agent setup's design-feedback note on the audit's T1 and the legacy axis, of 2026-10-15, and the skill is unchanged.

Put to the user in one yes:
- lift the pause;
- close the phase on iteration 2 by the stop rule;
- one `skill-auditor` pass before phase 4's run.

The user approved all three. Deleted `iteration-3`: it was prepared under the reverted
"Claims hold" clause, and no run had started in it. The v2 case's `pending` in `evals.json`
now names phase 4's review in place of its run. Checks green: fmt, clippy, `cargo test` 517
passed, `test_scope.py` 15 passed.

---

## Decisions

- **The v2 case keeps its prompt and its baseline, and its run is a partial observation.**
  The skill splits it into four passes of about $15, the first holding `review-security`
  alone; that run measures the scoping, the checks, the estimate, the split and the first
  pass. Phase 1's eight bugs are then counted by continuing the review from the run's report,
  in a later window, through the pass that holds `review-correctness`; that continuation also
  exercises the skill's path for a review in progress. Set aside: the partial run alone (the
  eight bugs left unmeasured) and a narrower prompt (a new baseline, one more paid window, and
  eight bugs found over the whole of `src/` no longer comparable).
- **One run with the skill per case in `iteration-2`, at Opus 5.5 `xhigh` under an $18
  ceiling**, as the baselines ran. Three runs per case would take three windows per case;
  `workspace.py --extend` raises the count later if a result calls for it.
- **The post-session case runs as it stands, its split into two passes included.** Its
  failures by construction (assertions 1, 2, 7 and 9) are recorded before the run, and the
  run's per-agent costs recalibrate the estimate, whose $1 a reviewer has no measure. Set
  aside: changing the formula before any measure (a run the estimate underrates is stopped at
  $18 and lost) and raising the $15 threshold to fit this case (no margin left for an
  estimate's error).
- **The gradings run with `--budget 3`, in a usage window of their own.** The default cap of
  $1 a session was set from `skill-auditor`'s runs, and a grader of a run with ten agents that
  stops at it is lost and paid again. Set aside: the default cap (a likely restart) and
  grading in the window that holds the run's $8.31 (at most $9 more, with this session's own
  spend, passes the $20 of a window).
- **The description stays as written.** It passed every trigger query, 14 of 14 in training
  and 8 of 8 held out; `tuning.py` has nothing to rewrite from.
- **Findings sharing a root cause are merged in every pass, before verification**, each
  keeping the axes and reviewers that raised it (the user). A cause is verified once, and the
  report a pass leaves is readable on its own. Set aside: the merge kept for the last pass,
  which left 41 findings for about 23 causes and verified one cause up to three times.
- **The estimate's rates come from measures**: $0.25 plus $0.20 per 1,000 lines a reviewer,
  three quarters of that a verifier, $3 the session. Two measures support them, the first
  pass on a small scope and phase 1's `/code-review` over `src/`; the next runs check them.
  Set aside: the former $1 base per reviewer, which split a case that fits one pass.
- **Defect 8 counts when the finding removes the alias for want of an obligation.** A
  `// LEGACY:` comment is owed by legacy code that stays (phase 0), not by code to remove.
- **The phase closes on iteration 2, by the stop rule** of the 2026-10-15 decision (the
  user). The skill beats its baseline clearly: 9 of 12 against 3 of 12 on the post-session
  case, and 22 of 22 trigger queries. The failures left carry no risk. Set aside: iteration 3,
  since real use gives the same proof at a fraction of its cost.
- **The v2 case's run with the skill is dropped.** Phase 4's real review of the same code
  stands in for it, the count of the eight bugs included. This replaces the decision of
  2026-10-13 on a partial observation continued by hand.
- **Each failure left keeps its class**, recorded here without a second paid grading
  (`grade.py --again`):

  | Failure | Class | Fix |
  |---|---|---|
  | Assertion 1, defect 8 | `assertion` | The expected output asked a `// LEGACY:` comment of code to remove; fixed in the assertion on 2026-10-14 |
  | Assertion 1, defect 10 | `skill` | `review-tests`' step 4 on inputs that share one shape; real use is its proof |
  | Assertion 9, the fix plan | `skill` | The estimate, 1.7 times too high, split a case that fits one pass; recalibrated on 2026-10-14 |
  | Assertion 10, Claims hold | `tool` | The grader's rendering dropped the agents' `instructions` attachments; fixed in skill-tooling 0.1.2, the workaround reverted at `docs(review-skill): record iteration 2's grading and pause phase 3` of 2026-10-14 |
- **`SKILL.md`'s three source citations and phase 2's cuts drawn from the study stay.**
  the agent setup's roadmap `lean-workflow` decides their place in its Phase 1.

---

## Files Changed

**Added**
- `docs/roadmap/on-progress/review-skill/phase-3-evaluation-report.md`

**Modified**
- `.claude/agents/review-tests.md`
- `.claude/agents/review-verifier.md`
- `.claude/skills/reviewing-delivered-code/SKILL.md`
- `.claude/skills/reviewing-delivered-code/assets/report.md`
- `.claude/skills/reviewing-delivered-code/evals/evals.json`
- `CLAUDE.md`: outside this phase's work, the test-first convention and the phase checks
  (`docs: make code and scripts test-first, other artifacts take their own proof` of 2026-10-14, `docs: run the review skill's script tests among the phase checks` of 2026-10-14), committed during the pause
- `docs/roadmap/on-progress/review-skill/README.md`
- `docs/roadmap/on-progress/review-skill/phase-3-evaluation.md`
- `docs/roadmap/on-progress/review-skill/phase-4-first-review.md`
- `docs/roadmap/on-progress/review-skill/phase-5-workflow-record.md`

---

## Problems And Deviations

- **Task "Count which of the eight bugs … are found again" not done.** The v2 case's run
  with the skill was dropped. Moved to phase 4.
- **Tasks "Fix each failure that remains, rerun its case, then run the whole set" and
  "Compare the last two versions blind" removed** by the stop rule, with the user's
  approval. For the same reason, the acceptance criterion "The whole set passed once after
  the last fix" does not hold. The changes of 2026-10-14 are unevaluated: the merge before
  verification, the estimate and `review-tests`' step 4. Phase 4's run is their proof.
- **The estimate's rates rest on per-agent costs whose output share was settled by call
  count** (R3 of ai-ctl's note of 2026-10-14). Moved to phase 4, set against the real run's
  per-agent costs.
- **T1 flags the axis name `legacy` three times.** Left open in the agent setup:
  the agent setup's design-feedback note on the audit's T1 and the legacy axis, of 2026-10-15.
- **Phase paused from 2026-10-14 to 2026-10-15** for the update of the validation; lifted by
  the user.

---

## Changes To Later Phases

- **Pending approval**, approved by the user on 2026-10-15 and applied: phase 3 stops on
  iteration 2 by the stop rule. Its tasks "Fix each failure that remains…" and "Compare the
  last two versions blind…" leave the roadmap, and phase 4's review replaces the v2 case's
  run with the skill.
- `phase-4-first-review.md`, 8 tasks → 10:
  - added "Run one `skill-auditor` pass on the skill as phase 3 left it…", the default
    regime's pass, never run on the changes of 2026-10-14;
  - added "Count which of the eight bugs…", moved from this phase;
  - reworded "Record the run's cost…" so that it sets step 4's rates against the per-agent
    costs;
  - reworded the last task to a fix from use;
  - added to Constraints the estimate (about $39, three passes) and the default regime.
- `phase-5-workflow-record.md`: added to Constraints that the record is short.

---

## Assessment

With the skill, the post-session case passes 9 of 12 assertions, against 3 of 12 without it.
Its description triggers on 22 of 22 queries. After the pause, the phase stopped on its first
iteration with the skill, by the stop rule. The v2 case never ran with the skill.

The changes of 2026-10-14 are unevaluated: the merge before verification, the recalibrated
estimate and `review-tests`' step 4.

What phase 4 needs first:
- one `skill-auditor` pass on the skill as it stands;
- then the real review of the v2 code at `high`, about $39 in three passes, each pass in a
  fresh window from a new session, after the user's yes on the estimate;
- its per-agent costs set the estimate again, and its count of the eight bugs measures
  `review-correctness`.
