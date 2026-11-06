# Phase 1 Report: Evals And Baseline

**Phase:** [phase-1-evals-and-baseline.md](phase-1-evals-and-baseline.md)
**Start Commit:** `docs(review-skill-v2): close phase 0` of 2026-11-01

---

## Work Log

### 2026-11-01

Phase opened. The cases proposed to the user before any file: five cases on phase 0's themes,
their fixtures, the base they start from and the baseline's runs; the user: "oui parfait".

The fixtures first, as the cases' answer keys. The `gateway prune` session of the earlier evals,
ported to `main`'s code, its eleven defects kept and a twelfth added, becomes phase 1 of a
fictitious roadmap, `gateway-releases`, of 17 commits, each commit of code passing the three
checks; phase 2, `gateway install --from`, plants the security defect, drops the test of M2 and
rewrites M3's text. The figures the review states were measured on that history: phase 1's
diff, 18 hunks and 25 mutants, 15 missed, in 53 s; phase 2's, 29 hunks and 14 mutants, one
missed, in 38 s. The lock file's two crates come from the local registry cache, so the
dependency case builds offline.

`fixtures.py` builds each case from `docs: give the command that installs a release from its tag` of 2026-10-30, squashed, test-first: `test_fixtures.py`'s eleven
new tests watched failing on their assertions, then passing, 30 runs green. `evals/evals.json`
holds the five cases, no assertion before the baseline's failures; `workspace.py --baseline
<base> --baseline-only --runs 1`, `<base>` being `docs(review-skill-v2): open phase 1`, validated it and prepared `iteration-7`. Tasks 1 and 2
committed as `test(reviewing-delivered-code): five cases and fixtures for the v2`.

Next session. The triggers grow from 22 to 36, 18 of each: seven requests the v2 must take —
three checks of the dependencies, two reviews at a phase's close, the roadmap's closing
security pass and a security pass alone —, and seven near misses beside them, each a
neighbouring skill's request or a task done without a review. `workspace.py --assertions
iteration-7` validated them and left the iteration's cases unchanged.

The plan for tasks 4 and 6, the 108 trigger sessions, then the five runs one at a time, each
started only under 90% of the window; the user: "oui parfait".

Task 6, the committed description on the 36 queries, three sessions each, `claude-opus-5-5` at
`xhigh`: 32 passed — the 18 near misses at 0; at 1.0, the 11 earlier requests, the two reviews
at a phase's close and the security pass alone. Failed: the three checks of the dependencies, at
0, 0 and 0.33, their sessions running `cargo audit` and `cargo update --dry-run` themselves; the
roadmap's closing security pass, at 0, two sessions in three loading the roadmap skill at their
first call. Phase 3's description has to name the dependency check and the roadmap's closing
pass, the second against the roadmap skill's pull.

Task 4, the five runs, `claude-sonnet-5-5` at `xhigh`, all five complete by `run.py --status`,
from 53 s to 5 min 40 s. Every run stops at the old skill's step 2,
and the four whose request accepts the cost refuse it as the go-ahead; every scope is set by
file. `continued-review` runs `cargo mutants` again, sends the findings below the floor to the
verifier and rewrites `report.md` by hand; `roadmap-close` takes the closing pass for a full
review; `dependency-check` sets a code review's scope over the whole history and leaves updates,
maintenance and reliability out. Three failures belong to the runs' world: the hook refuses
`~/.claude`, so no run shows `usage.py`'s line; the session sees its $5 ceiling and weighs the
tier against it; `cargo-deny` is not installed.

Reading the runs showed a defect of the `continued-review` fixture: findings 1, 2 and 14 cited a
verifier's copy, which a state stopped before its verifiers cannot hold; they now cite
`review-correctness`'s. `test_fixtures.py` green.

Task 5's assertions written from the failures, each named after what it checks, the failures
quoted under `failures_with_skill`, with three measures and two guards, the second, no personal
data in a request, from a twelfth failure in use, phase 0's `User-Agent`.

The user: "validé", for the grading and for a design-feedback note to the agent setup on
`run.py`'s record of the agents' copy. The grader passed `cost-gate`'s guard "No file changed",
reading the seven agents as the harness's copy, and named `roadmap-close`'s "No rejected finding
raised again" weak, since a run reporting nothing passes it: it now requires the security pass
to have run, and that run waits for its grading again. The graders' claim checks: the call the
runs' hook refused was an `ls` of the folder holding `usage.py`; the $5 ceiling the sessions
give appears in no file a run's record keeps.

### 2026-11-02

The user's yes on the grading left: `roadmap-close` at 2/14, the rewritten check now failing.
The grader still names it weak for checking one finding only; the planted one is the case's
assertion "The security finding", so it stays. Its failure classed `skill`, the 57 in
`classes.json` all `skill`. `benchmark.py` on the five runs: 23% ± 13% — `cost-gate` 40%,
`phase-close` 7%, `continued-review` 25%, `roadmap-close` 14%, `dependency-check` 27% —; the
three measures at 0/4, 0/16 and 0/3. Task 5 done.

The phase closed at the user's word, "tu peux cloturer la phase 1 et la session en même temps",
the six checks green before it. Left outside the phase: the design-feedback note on `run.py`'s
`changes.json`, among the agent setup's notes, untracked; the user's own change to
`.gitignore`, `/notes` to `/notes/`, uncommitted.

---

## Decisions

- **Five cases** in place of the three: `cost-gate` rewritten, a review without the cost
  accepted that stops on one go-ahead; `phase-close`, a fictitious roadmap whose phase 1
  delivered `gateway prune` with planted defects, the cost accepted in the request: the scope by
  hunk with its context, the session's triage with one hard case, the floor, the register and
  the fixes in the phase; `continued-review`, the same review stopped by the window's guard
  before its verifiers; `roadmap-close`, the same roadmap at its last phase, a security defect
  planted in phase 2, a register and recorded mutants from phase 1's fixes: the security pass
  alone, the regression run and no pass after its fixes; `dependency-check`, `anyhow` 1.0.102
  under RUSTSEC-2026-0190 and `clap_complete` 4.6.2 behind 4.6.11 in the lock file. The
  `phase-close` case is also phase 4's case on a fix, `roadmap-close` its security case,
  `dependency-check` its dependency case.
- **`review-v2-roadmap` and `post-session-review` removed:** the first is a full review of
  24,000 lines, about $50 a run, a request apart in the v2; the second's patch becomes
  `phase-close`'s phase 1.
- **The fixtures start from `main` before this roadmap, `docs: give the command that installs a release from its tag` of 2026-10-30, squashed into one commit,**
  without `.claude/` and roadmap `review-skill`, which hold the old skill and the cases' answer
  keys: in a run's folder the old skill would sit beside the arm's copy, and the history would
  give the removed files back. `CLAUDE.md` is today's, without the two checks of the evals.
- **The states of `continued-review` and `roadmap-close` are written by hand** by the Design's
  fields — the review's folder with `findings.json`, `register.json`, `mutants.json` —, no run
  having left one: they become the formats phase 2's scripts read.
- **The baseline: one run per case on the committed skill, replies for `cost-gate` alone.** The
  old skill stops at its go-ahead, $0.25 to $1.30 a run in the iterations of 2026-10-13 to
  2026-10-16, so the runs show the front of a review; the back — a verdict asked per finding, no
  floor, `repro/` deleted, the report written by hand — was seen in real use and goes to
  `failures_in_use` with its source.
- **The near miss of a phase's closure names a phase that changed only documentation:** from
  phase 3 on, `CLAUDE.md` ends a phase that changes code with its review, so the earlier query,
  phase 3 of roadmap `review-skill`, which changed the skill's folder, would no longer be a clear
  near miss. The earlier 21 queries stay as they were, for phase 4 to compare on the same set.
- **The assertions on a review's front come from the baseline's runs, those on its back from
  `failures_in_use`:** every run stopped at its go-ahead, so none shows the floor, the triage,
  `repro/`, the fixes or the report; each case's `pending` says which.

---

## Deviations

- `audit.py`'s ten warnings on `SKILL.md`, its citations of a roadmap, left: the phase changes
  no file of the skill but its evals, and phase 3's second task removes them.
- What phase 4 meets in its runs, found while writing the cases: a run of 2026-10-16 says a hook
  refuses `~/.claude` in the runs, where `usage.py` sits; the runs have no `committing-changes`,
  so a run commits with git alone; the dependency case's fixture takes `deny.toml` once phase 2
  writes it, and that case's baseline runs again.
- No trigger query measures the review that `CLAUDE.md`'s rule starts when a phase is closed
  without one being asked: `triggers.py` stops a session at its third call, and a closure's first
  calls are the roadmap skill's. Left open: the first phase this roadmap closes under the rule
  shows it.
- The trigger sessions' copy holds `CLAUDE.md`, which sends a session to the open roadmap's phase
  file before any work, and this roadmap's files, which describe the v2: most sessions of the
  failed queries spent their first calls reading `docs/roadmap/on-progress/review-skill-v2/`,
  within `triggers.py`'s three. `triggers.py` takes no list of paths to leave out of the copy.
  Left open for phase 4, whose measure meets the same copy.
- What phase 4 meets in its runs, found in the baseline's: the session sees its run's ceiling,
  $5, and gives it as a reason to stop, where a review with its fixes costs more; `changes.json`
  lists the arm's seven agents, which the runs' folder gets under `.claude/agents/`, as added,
  since the fixtures hold no `.claude/`, so `cost-gate`'s guard "No file changed" fails in every
  run, a defect of `run.py`'s record; and the `continued-review` fixture's change makes
  `--reuse` run that case's baseline again.

---

## Assessment

The phase delivered what phase 3 writes against: five cases on phase 0's decisions, their
fixtures built offline and test-first by `fixtures.py`, 36 trigger queries, and a baseline on
the committed skill. The baseline's runs pass 23% of their assertions, and none of the three
measures scores: every run stops at the old skill's go-ahead, refuses a cost accepted in the
request, and sets its scope by file. The assertions on a review's back — the floor, the triage,
`repro/`, the fixes, the report — therefore rest on `failures_in_use`, and phase 4 is their
first run. The description triggers on 32 of the 36 queries; it misses the three checks of the
dependencies and the roadmap's closing pass, which phase 3's description has to name.

Four things the runs' world adds, left for phase 4: the hook that refuses `~/.claude`, the
session's $5 ceiling, `run.py`'s `changes.json` counting the arm's agents as added, and the
triggers' copy holding `CLAUDE.md` and this roadmap's files.

What phase 2 needs first: the states of `continued-review` and `roadmap-close`, written by hand
by phase 0's Design — `findings.json`, `register.json`, `mutants.json` —, are the formats its
scripts read, and the fixture history's figures — the hunks, the mutants missed, `running !=
release` in `Installer::install` — are what `scope.py` and the regression run must give back.
Once phase 2 writes `deny.toml`, the dependency case's fixture takes it and that case's
baseline runs again.
