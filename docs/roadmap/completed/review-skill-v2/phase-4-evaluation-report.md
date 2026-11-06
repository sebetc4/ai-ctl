# Phase 4 Report: Evaluation

**Phase:** [phase-4-evaluation.md](phase-4-evaluation.md)
**Start Commit:** `docs(review-skill-v2): close phase 3` of 2026-11-02

---

## Work Log

### 2026-11-03

Iteration 8 prepared against the baseline at `docs(review-skill-v2): open phase 1` of 2026-11-01, three runs per case and
configuration. None of iteration 7's baseline runs is reused: every arm now carries the rules
skill, and the v2's `review-dependencies`, so each case's baseline runs again.

Phase 1's four gaps of the runs' world, before any run:

- The guard: the fixtures carry no `.claude/`, and `run.py` puts the skill's copy beside the
  run's folder, so the project's settings could not name it. The evals' `hooks` now names
  `evals/hooks.json`, the settings' three hooks with the guard at
  `$CLAUDE_PROJECT_DIR/../skills/reviewing-delivered-code/scripts/guard.py`; the baseline's
  revision lacks the file, so its arm runs without the guard its skill does not know.
- `changes.json` counting the arm's agents as added: fixed in skill-tooling 0.2.10, where a
  run's start is the tree of its files once placed; to be confirmed on the runs.
- The run's ceiling, $5, which a session gave as its reason to stop: $30 a run.
- `usage.py` refused under `~/.claude`: still open, and no case's setup can reach it, the
  evals' `skills` taking only the repository's paths and `usage.py` writing in the real
  configuration folder. Written to the agent setup as the design-feedback note
  `2026-11-03-usage-py-unreachable-in-eval-runs.md`, untracked; the assertions on the
  go-ahead's line — `cost-gate` 2, `phase-close` 1, `roadmap-close` 1, `dependency-check`
  1 — are to be classed `tool` with the refusal as evidence.

The triggers' copy holding `CLAUDE.md` and this roadmap's files is left to the trigger task.
The scripts' line for the 30 runs reads 69 to 138% of the five-hour window, estimated from
runs that stopped at the old skill's go-ahead, where the v2's runs go through their agents
and their fixes.

The rules skill's trigger queries written first, in `reviewing-as-an-agent`'s own
`evals/evals.json`: the 18 review requests of `reviewing-delivered-code`'s set, each
`should_trigger: false`. `triggers.py` refuses the file, which holds no output case, `evals`
being required; a case only to pass the validation would be written for the tool, the
skill's output being its agents' answers, which the review's cases grade. Written to
the agent setup as the note `2026-11-03-trigger-eval-for-a-preloaded-skill.md`, untracked; the
measure waits for it.

Run 1 of three cases, both configurations, one case at a time. `cost-gate`: the v2 7/10, the
baseline 4/10; the v2's session started with the guard's three hooks, and `changes.json` is
empty, the 0.2.10 fix confirmed. The v2 fails three: the scope given in totals, not hunk by
hunk; `usage.py`'s line, refused; and a later `usage.py estimate --start` announced for the
verifier. `dependency-check`: the v2 9/11, the baseline 5/11, the v2 starting
`review-dependencies` alone, judging the advisory unreached and giving the backends' lag; it
fails `usage.py`'s line and the update within the requirements, sent to the register: step 6
fixes a finding below the floor at once only on the scope's new code, and a review of the
dependencies alone has none, against phase 3's decision that such an update is fixed at once.
`continued-review`: both complete, not graded; the v2 ran 24 min with one `review-verifier`,
and its `response.md` is the turn of a wake-up it scheduled while the verifier ran, its account
being in the turn before.

The real cost against the line: `continued-review`'s two runs were estimated at 10 to 18% of
the five-hour window, about $1.18; the v2's run alone cost $5.54. The window stood at 36% used
before that case. No failure classed yet: `grade.py --again` has not run and `classes.json` is
not written.

Runs paused for the agent setup's proof by severity, installed the same day (skill-tooling
0.3.0, roadmap 3.4.0, review 0.4.12); the plan resized by it, 14 runs where iteration 8
planned 30 (Decisions).

`continued-review` graded: the v2 8/8, the baseline 3/8, stopping for a new go-ahead before
its verifiers and so failing the floor, the register and the triage. The v2's five failures
of iteration 8 classed: two `skill`, fixed in `SKILL.md` — the `--start` before the
verifier announced at the go-ahead (low: the static audit and the diff), and the update
within the requirements sent to the register, step 6 fixing at once only on the scope's new
code, which a review of the dependencies alone has not (medium: `dependency-check` in
iteration 9). Three put to the user: `usage.py`'s line, twice, `world` rather than `tool`,
the case lacking the access the installed skill has, no script misreading anything; `Scope
by hunk` `assertion`, the v2 counting the hunks' 233 lines but listing the 18 hunks only in
`scope.txt`, which the skill hands the reviewers, not the user.

The three classes approved and written; `Scope by hunk` rewritten to the scope computed by
hunk and its `reading:` count, and `continued-review`'s `Nothing run again`, flagged weak
since a run doing nothing passed it, to verifiers starting with nothing run before them.
Iteration 9 prepared against `docs(review-skill-v2): open phase 1` of 2026-11-01, `authoring-skills` among the runs' skills, 14 runs.

`cost-gate`'s runs 1 and 2 of iteration 9: the v2 10/10 in both, its guards held, the
baseline 4/10 in both. The v2's sessions ran the run's copy of `usage.py`, whose line gave
the window and no estimate for one `review-verifier`. No grader flagged a weak assertion.

`cost-gate`'s run 3: the v2 10/10 again, the baseline 4/10. Over its three runs the v2's
guards held in each — no agent started, `changes.json` empty, no scratch left, no stack
command — and its go-ahead left out every form the v2 removed, 12/12 against the baseline's
0/12: `cost-gate`'s high proof met. No failure of the v2 to class, no assertion flagged weak,
and `benchmark.md` names none that tells no configuration apart.

`dependency-check`: the v2 9/11, the baseline 5/11. The medium fix holds: the v2 classed the
two updates within the requirements `fix at once`, ran `cargo update -p anyhow -p
clap_complete` and the checks green. It made no commit, `fixing.md` committing through
`committing-changes`, which the runs' `skills` lacks: put to the user as `world`, with
`phase-close` and `roadmap-close` bound to fail the same way. Its guard on personal data
failed on `git ls-remote`, which `review-dependencies` is told to use and the session ran
too, against `SKILL.md`'s "only through `dependencies.py`": git's own User-Agent, no
credential helper or extra header in git's configuration, iteration 8's grader passing the
same calls. Put to the user as `assertion`, with `SKILL.md`'s line aligned on the agent's.
`benchmark.md` names assertions 2, 3 and 7 telling no configuration apart, as in iteration 8.

The two classes approved and written. `committing-changes` added to the evals' `skills`; the
guard reworded to no email address or user name in any request, `dependencies.py`'s naming
the tool alone, `git ls-remote`'s carrying git's own; `SKILL.md`'s line given `git ls-remote
--tags` for a repository's tags (low: the static audit clean, the diff read). Assertion 3
dropped, assertion 4 requiring the advisory, and with it the case's `pending`, which
iterations 8 and 9 settled. Iteration 9 not graded again on the reworded guard: iteration 10
runs the case again. Iteration 10 prepared against `docs(review-skill-v2): open phase 1` of 2026-11-01, the four cases left, 8 runs.

`continued-review` in iteration 10, on the user's yes: both runs complete, the v2 in 27 min
for $5.41, the baseline in 8 min for $1.02, the five-hour window going from 8% to 71%, as
its line said. The baseline graded 2/8. The v2's grading was cut when the session's process
ended: its `grader/` holds only `prompt.md`, no `grading.json`, and iteration 10 has no
`classes.json` yet. The window stood at 81% at the close.

`continued-review`'s v2 graded on the user's yes: 8/8 against the baseline's 2/8, no
failure to class, no assertion flagged weak. The grader saw two departures no assertion
covers. The session set `findings.json`'s `status` by hand, as step 9 asks and `findings.py
report` then checks, against `SKILL.md`'s opening line that `findings.py` alone writes the
file: the line aligned on the steps (low: the static audit clean, the diff read). It also ran
its fixed `gateway prune` from `target/` under `env -i`, `HOME` and the XDG folders in a
throwaway tree, beyond the commands that only read; ai-ctl's paths come from those variables
alone, so nothing of the machine was reached. Not fixed: watched in `phase-close` and
`roadmap-close`, whose fixes give the same occasion.

`dependency-check` in iteration 10, on the user's yes: both runs complete, the v2 in 17 min,
the baseline in 7 min. Their grading waits for the five-hour window's reset: at 2% when the
grader of `continued-review` started, it read 99% at these runs' end, 24 minutes later, about
a quarter of it from the grader and the two runs, and `grade.py`'s line, 106 to 109% after
it, was refused past 90%.

### 2026-11-04

`dependency-check` graded on the user's yes: the v2 10/10, the baseline 4/10, no failure to
class, no assertion flagged weak; `review-dependencies`' medium proof met. The reworded guard
on personal data now tells the configurations apart: the baseline's first crates.io request
carried the user's email address in its `User-Agent`, the v2's none. The v2 committed its
`cargo update` through `committing-changes` and ran no ai-ctl command that writes. Assertions
2 and 6 (7 before assertion 3 was dropped) tell no configuration apart, as the decision kept
them. The seven-day window went from 79% to 80%.

`roadmap-close` in iteration 10, on the user's yes: both runs complete, the v2 in 25 min, the
baseline in 2 min, stopped on a go-ahead for a full review where the request asked the
security pass alone. Its line read 1%; the window went from 80% to 85%. Not graded yet. Read
in the v2's answer, for the grading: `review-security`'s answer refused twice by
`findings.py`, its header `FINDING 1` for `FINDING`, the session fixing that line by hand in a
copy; the security finding lowered from `medium` to `low`, which assertion 4 does not accept;
a PROBE asking a real `gateway install` left unrun, so no writing ai-ctl command. The trigger
lines read, nothing started: the skill's 36 queries 0 to 1% of the seven-day window, the
rules skill's 18, already in its `evals.json`, 0 to 3%.

`roadmap-close` graded on the user's yes: the v2 12/14, the baseline 2/14, no assertion flagged
weak. The benchmark over the three cases graded: the v2 86 to 100%, the baseline 14 to 40%.
Telling no configuration apart: `roadmap-close`'s 1 and 4, failed in both, the baseline having
stopped at its go-ahead; `dependency-check`'s 2 and 6, passed in both, as the decision kept them.

Both failures stay `skill`. Assertion 1: the estimate's line stayed in the command's output,
as in `dependency-check`'s v2, where the grader's evidence named it a weakness; step 2 now asks
for them in the session's text before the first agent (low: the static audit clean, the diff
read), and `dependency-check`'s assertion takes `roadmap-close`'s wording, not graded again, no
run of it being left. Assertion 4: `review-security` found the defect at `medium`;
`review-verifier` confirmed it on a run, looked for the anchors under the skills' copies, found
none, and lowered it to `low` because the machine has one login account, which the `medium`
anchor already takes for granted. The security anchors moved into the rules skill, for a
reviewer and a verifier alike, with that ground ruled out (medium: the case runs again).

The other two points read in the v2's answer. `FINDING 1`: `review-security` never opened the
rules, an agent's `skills:` preloading nothing in a run, where `review-dependencies` and
`review-verifier` found the copy by themselves. A gap of the validation, written to the agent setup
as the note `2026-11-04-agent-skills-field-in-eval-runs.md`, untracked; neither the answer's
format nor `findings.py` changes for it, and phase 5's real use, with the preload, shows whether
it recurs. The PROBE: the guard refused `review-security`'s real `gateway install` and the session
ran none, so the watch on writing ai-ctl commands holds here, `phase-close` left to read.

Iteration 11 prepared against `docs(review-skill-v2): open phase 1` of 2026-11-01, `roadmap-close` and `phase-close`, 4 runs:
`phase-close` runs on the fixes rather than on iteration 10's snapshot, and the rules skill's
change enters the cases' digests, so no baseline is reused. Whole, its line goes past 90% of the
five-hour window: split by case. The two grader sessions of this grading took about 20 points of
that window, which the next grading has to find room for.

Phase resumed, nothing started. `roadmap-close`'s line at 12:20 local time: the five-hour window
at 45%, the two runs 46 to 47% of it, 91 to 92% after them, past 90%. It cannot be split further
(Deviations), so it waits for the window's reset at 15:50 local time, 14:50 UTC, which also holds
its grader sessions. The seven-day window stood at 74%, `phase-close`'s line taking it to 75 to
92%.

The trigger batches' lines read for the same comparison as phase 1, whose record is
`.eval-runs/skills/reviewing-delivered-code/triggers/2026-11-01T165503Z.json`: 32 of 36 queries,
three sessions each, `--model claude-opus-5-5 --effort xhigh`, which the comparison has to pass,
`triggers.py`' defaults differing. At those settings the skill's 36 queries stay under the line's
threshold and print no figure, while phase 1's cost per session suggests about 20 points of the
five-hour window, an extrapolation the batch's real cost settles. The rules skill's 18 have no
estimate and read 0 to 3% of the seven-day window: one session first, then its line read again.
`revue des dépendances…`, at 0.33 in phase 1, is the query whose sessions disagreed: three
sessions. `triggers.py` still takes no list of paths to leave out, so the sessions' copy holds
`CLAUDE.md` and this roadmap's files as in phase 1: like for like, and each missed query's calls
are read for that cause.

Proposed to the user, unanswered at the close: the trigger batches before the reset, what is
left of this window being lost otherwise and the measure depending on the runs only if a fix
changes the description; `roadmap-close` after it.

Phase resumed at 14:03 on another account than the lines above read: its seven-day window at
90% until 2026-11-05 at 07:00, its five-hour window at 1%. Every line went past 90% of the
seven-day window, the trigger batches 0 to 4%, `roadmap-close` 3%, `phase-close` 1 to 18% and up
to 268% of the five-hour window: nothing started.

The roadmap's spend, on the user's question: about $188 API-equivalent since 2026-10-31, $147 of
its 22 sessions and $41 of paid batches, about 40% of the use of those days on the user's three
accounts. This phase $58, $31 of runs and grading against $27 for the eight sessions that ran
them; 25 commits, 16 recording runs, no task ticked. Its runs found four defects of the skill,
the baseline's email address in its `User-Agent` and two defects of skill-tooling; three fixes
went to the eval harness. Iteration 11 dropped on the user's yes (Decisions). `benchmark.py`
written for iteration 8, which lacked it: the v2 84%, 95% and 95% over iterations 8 to 10, the
baseline 41%, 41% and 26%. Every run started ended `complete` by `run.py --status`, 6 in
iteration 8, 8 in 9, 6 in 10. The consumption went to the agent setup as the note
`2026-11-04-eval-cost-of-an-agent-skill.md`, untracked.

`skill-auditor`'s two passes before the trigger measure, which a fix to a description voids, on
the user's word to go on with this account, the passes about 0% of its seven-day window: the
rules skill `PASS`; the skill `FAIL` on four points, each answered in `SKILL.md`, with no pass
after, by `authoring-skills`' `references/audit.md`. The description's first clause now names the
report it produces and six of the axes, the workflow left to the body; its proof is the trigger
measure. The phase report's Start Commit became the commit the phase opened on, which the roadmap
skill records; the triage's two document paths gave way to the contracts they hold, which
`CLAUDE.md` places; the agents are named once and their effort tied to the tier. The moments by
phase and roadmap stay, being `CLAUDE.md`'s review rule. The static audit clean, the six eval
scripts green.

The trigger measure, forced past 90% on the user's word, the seven-day window from 93% to 96%,
`--model claude-opus-5-5 --effort xhigh` as in phase 1. The skill, on the audited description:
18 of 18 queries that should trigger, against 14, phase 1's four misses — the crates, the
dependencies, the closing security pass — all triggering and `revue des dépendances…` 3 of 3;
17 of 18 near misses, against 18. Query 22, `Do a security review of the pending changes on this
branch before I push`, the built-in `/security-review`'s request, triggered the skill as a
security pass alone, in its one session. Classed `skill` and left open: the skill asks its
go-ahead before any agent, so the false trigger costs a question, while a narrower wording would
need every query measured again; phase 5's use watches it. The four reads of the skill's source
in the sessions' copy came from two queries that did not trigger. The rules skill: none of the 18
review requests triggered it. The records are `triggers/2026-11-04T134133Z.json` and
`2026-11-04T134156Z.json` for the skill, `2026-11-04T133933Z.json` and `2026-11-04T134108Z.json`
for the rules skill.

Phase closed with no review of its hunks: it changed no code, only the skill's text, its agents,
its evals and this roadmap's files.

---

## Decisions

- **The proof by severity** (authoring-skills, The Severity; the user, 2026-11-03: "oui
  parfait"), in place of the full regime: `cost-gate` high, its guards keeping paid agents
  from starting and files from changing unasked, three runs per configuration and its
  guards holding in each; the four other cases medium, one run per configuration; the
  go-ahead's wording low, no run of its own. `dependency-check`'s guard on personal data
  stays medium: `dependencies.py` sets the User-Agent and `test_dependencies.py` checks it,
  and GitHub's 60 requests an hour leave no room for a third run within the hour. A fix
  reruns its own case, once, `cost-gate` three times, only when it is medium or high: the
  whole set's last run is dropped.
- **A run's ceiling at $30,** where phase 1 saw the $5 one given as a reason to stop a review
  with its fixes.
- **Iteration 8 read, then iteration 9** (the user, 2026-11-03, on the plan): iteration 8's
  ended runs graded, `continued-review`'s the last, and every failure of the v2 classed and
  fixed before any new run; then `authoring-skills` added to the evals' `skills`, which
  changes every case's digest, so iteration 9 runs every case in both configurations,
  `cost-gate` first: its runs show whether a session calls the run's copy of `usage.py` or
  the installed path `SKILL.md` names, which the guard refuses. Then `dependency-check`,
  `continued-review`, `roadmap-close` and `phase-close`, each started when the five-hour
  window holds it, `phase-close`'s v2 alone in a window.
- **`CLAUDE.md`'s two test rules** (the user, 2026-11-03): 30 runs before its commit for a
  new test that binds a port, takes a lock or runs two processes at once, one for a test
  that only runs a command in a scratch folder; mutants by hand on a script only for logic
  whose failure is silent or irreversible, such as the guard, `cargo mutants --in-diff`
  unchanged. The fix method's step 5 and phase 6's fixes task follow it.
- **`dependency-check`'s assertions 2 and 7 kept** (the user, 2026-11-03: "oui parfait, je
  valide!"), though they tell no configuration apart in iterations 8 and 9: they check that
  the v2 adds no failure, which this phase's objective asks — 2 that a dependency check
  starts no reviewer of code, a spend unasked, 7 that the duplicates raise no finding.
  Assertion 3 dropped, assertion 4 requiring the advisory it reported.
- **`gateway prune` run by the session, watched rather than fixed** (2026-11-03, stated to
  the user): `continued-review`'s v2 ran a writing ai-ctl command, isolated in a throwaway
  `HOME` and XDG tree, which no assertion catches. `phase-close`'s and `roadmap-close`'s v2
  runs are read for the same move; if it recurs, the case's "No stack command" guard is
  widened to every ai-ctl command but those that only read.
- **Iteration 11 dropped, its two proofs left to the real use** (the user, 2026-11-04: "oui
  parfait!"), by `authoring-skills`' rule that what real use will prove is not run. Phase 6
  fixes by batches and ends on the security pass, which `phase-close` and `roadmap-close`
  stand for; phase 5's review runs `review-security` and `review-verifier` with the rules
  preloaded, which no eval run does, and the anchors left `review-security.md` for that
  skill, which `review-security` did not open in iteration 10: `roadmap-close`'s rerun would
  have measured that gap more than the fix. `phase-close`'s v2 needed a five-hour window to
  itself, up to 268% of one at its ceiling. Its comparison with the baseline is given up, and
  the fix method first runs on real code, one batch a commit on this branch. Phases 5 and 6
  take the tasks and the constraint Deviations lists; phase 4 ends on `skill-auditor`'s two
  passes and the trigger measure.

---

## Deviations

- The go-ahead's line could not be measured in iteration 8: in a run, the guard of `run.py`
  refused `usage.py` under `~/.claude`. Fixed in skill-tooling 0.3.0, where the evals'
  `skills` takes `authoring-skills` by name and each run gets a copy of the usage records;
  iteration 8's assertions on that line are classed `tool`.
- The rules skill's trigger measure could not run: `triggers.py` asked for an output case,
  which a skill preloaded into agents has not. Fixed in skill-tooling 0.3.0, which measures
  an `evals.json` holding queries alone.
- Task "Run the whole set once every case passes" removed, by the user's approval of the
  proof by severity: a fix reruns its own case. The three other tasks reworded for it,
  each `Proof:` line naming its severity.
- `phase-6-fixes-and-closing-pass.md`, its fixes task: a new test runs 30 times only when it
  binds a port, takes a lock or runs two processes at once, by `CLAUDE.md`'s rule as the user
  changed it.
- `run.py` splits a batch by `--case` and `--run`, never by configuration: at one run per
  configuration, a case's two arms start together, `--case roadmap-close/with_skill` being
  refused. `roadmap-close`'s line past 90% is already its smallest part, and `phase-close`'s v2
  cannot start without its baseline.
- `phase-close`'s v2 never ran, iterations 8 to 10 leaving it for a window of its own, and
  iteration 11 dropped (Decisions). This phase's objective and Out of Scope name what phases 5
  and 6 prove instead; tasks 1 and 2 reworded for it.
- `phase-5-first-real-use.md`: a task added, the security anchors and the agents' answer format
  checked in the review's agents, and a constraint, no ai-ctl command that writes the stack's
  state without the user's yes, the watch on `gateway prune` having ended with no `phase-close`
  run. `phase-6-fixes-and-closing-pass.md`: a task added under The Skill In Use, the fix method's
  failures in use recorded and fixed.
- `skill-auditor`'s task, its proof `VERDICT: PASS` for each: the skill's pass said `FAIL`, and
  `references/audit.md` fixes each point at its source with no second pass, so the four points
  answered stand for its `PASS`; its `Proof:` line reworded so.
- The skill's trigger query 22, the built-in `/security-review`'s request, triggers the skill:
  left open, watched in phase 5's use, where a narrower "security pass alone" would be measured
  on every query: `phase-5-first-real-use.md` takes it as a constraint.
- The study registry's closed check red at the closure: the report named the agent setup's
  repository by its name five times since 2026-11-03, which a closed repository's tracked files
  never do; each line now says "the agent setup".

---

## Assessment

The v2 beats the baseline clearly on the four cases it ran: 84%, 95% and 95% of the
assertions over iterations 8 to 10, against 41%, 41% and 26%. `cost-gate` passed 10/10 in each
of its three runs, its guards holding; `dependency-check` 10/10, `continued-review` 8/8 and
`roadmap-close` 12/14, against 2 to 4 for the baseline. The audited description triggers on
every review request, phase 1's four misses included, and on one near miss, the built-in
`/security-review`'s request; the rules skill triggers on none. `skill-auditor` passed the rules
skill and failed the skill on four points, each answered. The runs found four defects of the
skill, two of skill-tooling and the baseline's personal data in its `User-Agent`, and three
fixes went to the eval harness. `phase-close` never ran on the v2: once the phase's cost was
measured, $58 of the roadmap's $188 with nearly half of it in the sessions that orchestrated the
runs, the user chose to prove the fix method and the security anchors' fix in phases 5 and 6.

Phase 5 needs to know first that its review is the first proof of the fix method and of the
anchors, with the agents' rules preloaded, which no eval run had. It checks the anchors and each
answer's format in the agents' answers, records each failure of the skill in `failures_in_use`,
watches the false trigger on a security review of pending changes, and runs no ai-ctl command
that writes the stack's state without the user's yes.
