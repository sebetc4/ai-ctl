# Phase 5 Report: Workflow Record

**Phase:** [phase-5-workflow-record.md](phase-5-workflow-record.md)
**Start Commit:** `docs(review-skill): record pass 5 of the v2 review, on the GPU paths` of 2026-10-16

---

## Work Log

### 2026-10-16

Phase opened in a new session, in a fresh usage window (the user's word: the session
started at 11:31 UTC, four minutes after pass 5's session ended at 95 % of the window
before). Read before working: phase 4's file and report in full, the two resume notes of
2026-10-16 (the GPU pass's, then pass 5's, whose step 2 this session takes up; its step 1,
the fix roadmap, waits). Phase 4's Changes To Later Phases hold nothing pending; no other
phase is in progress.

The phase gained four tasks at its opening, set by the two resume notes: the exact cost of
each pass from the `cost-state` records, and three on the skill (the user's cost gates,
the fixes from use for pass 5's misfits and the agents' container names, one eval case
on the gates). Its total is 10.

The user, on the proof of the skill's changes: the fixes from use for the misfits, as phase
4's were, and one eval case for the cost gates, with the skill, on a small fixture.

The exact costs of the first review, from the `cost-state` records Claude Code wrote at
22:39 on 2026-10-15, after the next sessions had looked for them: pass 1
$11.89, pass 2 $14.41, pass 3 $9.62, pass 4 $15.69,
the verdicts `d93c8ff9` $7.17. Passes 1 to 4 come to $51.61, against the $50.90 phase 4's
report gives from `usage.py` at 845 output tokens a call. Phase 4's report calls the exact
costs of passes 2 to 4 lost; they were only late. The sessions of phase 4's closure
and of pass 5 had no record yet at 11:35 UTC.

The skill's changes, made as fixes from use. Quotes from pass 5's transcript:

| # | Misfit, quoted | Line | Fixed in |
|---|---|---|---|
| 1 | "Il ne prévoit pas de reprendre une revue dont le `Status` est `complete`." | 165 | SKILL: a further pass on a complete review, its steps and its checks |
| 2 | "je les autorise à lire la GPU via NVML en lecture seule, ce que le skill ne prévoit pas" | 165 | SKILL and the seven agents: `nvidia-smi` or NVML queries that read, with no CUDA context, VRAM or setting |
| 3 | "Sa Coverage a 12 axes, alors que ce rapport n'en a que 9." | 165 | SKILL: a missing axis's row added, naming the passes that did not review it |
| 4 | "« Number from the report's highest number » est ambigu : faut-il partir de 100 ou de 101 ?" | 165 | step 6: from the number after the highest, 101 after 100 |
| 5 | "Ses deux pistes corrigent le texte des constats 21 et 106 ; aucune ne signale un nouveau défaut." | 341 | `review-verifier`: a correction goes in its finding's block; step 7: a lead line that corrects goes into its finding |
| 6 | "Pass 5's batches are numbered 52 to 55, after the approved ones so that the batch numbers cited elsewhere stay true" | 463 | step 9: batches after an approved plan, and the reopened ones put to the user |
| 7 | The user: "tu as anchainé les pass, les deux relecteurs et les trois vérificateurs sans breakpoint avec confirmation de ma part" | 533 | SKILL: no agent before the user's yes in the session, a stop with the cost spent before each wave; step 2 waits whatever the request accepts |
| 8 | "Rien ne reste : ni `sleep`, ni sonde, ni conteneur `ai-ctl-perfprobe`." | 251 | SKILL and the seven agents: a container name without the prefix `ai-ctl-`, which `lifecycle.rs:500` takes for ai-ctl's |

`audit.py`: 0 errors, 0 warnings. One `skill-auditor` pass: `VERDICT: FAIL`, two points of
content, each fixed at its source: the gate's parenthesis told pass 5's story, and now
cites the decision below; the Coverage rule gave the reason "the report predates the axis",
and now states only what to do. `audit.py` again 0 and 0.

The eval case `cost-gate` (case 3) takes the post-session fixture and the prompt of case 2,
which accepts the cost beforehand. Its assertions check that no agent starts, that the
estimate is shown and that the run asks for the yes. Pass 5's transcript stands as its
baseline, under `failures_with_skill`. Test first, for the case's fixture:
`evals/test_fixtures.py` failed on its assertion (`['cost-gate'] != []`), then passed once
`fixtures.py` gave the case the post-session builder. It joined the checks, in `CLAUDE.md`
and `.agent-conventions.toml`.

- **Found: a run takes one turn, so a run with the skill now ends at step 2.** Cases 1 and
  2 can no longer reach a report, and no run reaches a second wave. Not bent around:
  both cases' `pending` say so, and the design feedback
  the agent setup's design-feedback note on runs that wait for the user, of 2026-10-16
  proposes scripted replies.

`iteration-3` prepared: `cost-gate`, the skill's arm only, 3 runs at Opus 5.5 `xhigh`,
$3 ceiling each, so $9 at most; grading about $0.38 a run. The session had spent $4.36
by `usage.py`. The user's yes: "Oui, runs puis notation".

The three runs passed 6 of 6 each (`iteration-3/benchmark.md`), $1.16 for the runs and
$0.38 for their grading; no grader named a weak assertion. Each run set the scope
(`range HEAD~1`, 2,211 production and 1,152 test lines), showed the six reviewers with
their axes and effort, two passes and their estimate, started no agent (no Agent call in
any transcript), changed no file, and ended on the request for the yes, naming the
skill's rule that a cost accepted beforehand does not count. Task ticked.

- **Seen in the runs, of class `world`:** each run tried to reach
  `~/.claude/skills/authoring-skills/scripts/usage.py`, and the runs' guard refused it,
  as it refuses any call naming the real configuration folder. Each run then gave the
  session's budget counter instead and said so. In real use, the script is reachable.
- **Seen in the runs, an estimate overrated on a small scope:** $24.82 in two passes for
  one commit, of which $12 for six verifiers at step 2's flat $2. On the same scope,
  phase 3's run measured its first pass at $8.31, against $14.08 estimated (case 2's
  `failures_with_skill` F1). The $2 rate fits the v2 scope (verifiers at $1.09 to $3.96).
  On a commit, it splits a review that would fit in one session into two. Put to the user.

The user chose the record's place: [workflow.md](workflow.md), beside `analysis.md`, whose
relative links to the reports stay true when the roadmap moves to `completed/`; the next
repositories' work reads it by its absolute path. Read before writing: the reports of
phases 0 to 3 in full, phase 4's read at opening. The record gives each step its inputs,
actions, outputs and decisions, with a link to its report. It marks the parts specific to
ai-ctl and splits the user's own conventions from what holds anywhere. It gives the cost
of each phase, as the reports give it, and eleven changes for the next repository, each
from a problem a report recorded. Two figures it cites came from the resume notes, which
git ignores, and are recorded here: phase 4's closing session, $8.80 by
`usage.py` beside its $3.27 trigger eval; pass 5, about $10.50 by `usage.py`
with 776 output tokens for each call without its final usage. Neither session had its
`cost-state` record at 12:05 UTC.

Checked the record against the reports, claim by claim. Four did not hold as first
written, and each is corrected:
- phase 4's passes did not each hold two reviewers; pass 3 held only verifiers;
- its sixteen changes from use came with three new axes;
- tokens are recorded for the `/code-review` runs as well as for pass 1;
- the renumbering of 100 findings was done by a script, not by hand.

`check_links.py`: 24 relative links, none broken.

All ten tasks ticked; checks green (`cargo fmt --check`, `cargo clippy`, `cargo test`
with `up_a_proxied_request_and_down ... ok`, `test_scope.py`, `test_fixtures.py`). The
session had spent $7.19. The user chose to close the phase, then the roadmap, and to fix
the overrated estimate first.

The estimate's fix: at step 2, a verifier is now $0.20 plus $0.14 per 1,000 production
lines, in place of a flat $2. That fits both scopes measured: $0.51 against $0.49 on one
commit (phase 3), and $2.16 against $2.13 on the v2 code. On the case's commit, the
estimate becomes $12.37 in one pass. The case gained the assertion "One pass" and its F3.
`iteration-4`, after the user's yes on about $1.50, cost $2.25 ($1.78 for the runs, $0.47
for the grading). Two runs did not stop at step 2, which made it dearer.

- **`iteration-4`: "One pass" held in 3 of 3 runs, the gate in 1 of 3.** Runs 1 and 2 ran
  steps 3 to 5 (the checks and the report) before stopping ahead of the agents. Their
  reason: « J'ai jugé que ce blocage sert à protéger le coût des agents » (run 1); « La
  skill demande aussi d'attendre le oui avant l'étape 3, mais je suis passé outre pour ces
  étapes » (run 2). Five failures, all classed `skill` in `iteration-4/classes.json`. Step
  2's stop gave no reason of its own beside a rule on agents. It now says that steps 3 on
  wait for the yes, the checks and the report included, since the user asked to see the
  estimate before anything starts (phase 4's change 3). `iteration-3` had held 3 of 3
  under the same wording.

`iteration-5`, after the user's yes: the gate held in 3 of 3 runs, which changed no file,
started no agent and asked for the yes. The runs cost $0.81 and their grading $0.34.

- **`iteration-5`: "One pass" failed in run 1, classed `skill`.** Step 7 re-prices a
  verifier at $1 plus $0.07 a finding, a fit on the v2 scope alone. On one commit, that
  price is above step 2's $0.51 and phase 3's measured $0.49. Run 1 applied it in advance
  and split the review into two passes (« La vérification passera donc dans une seconde
  passe »). Put to the user.

The user chose to fix step 7 in this session, and to leave its proof and the closures to a
fresh window. Step 7 now prices the verification by its findings: $0.05 a finding, plus
$0.005 a finding per 1,000 production lines. That is about $0.12 a finding on the v2 code
and $0.06 on one commit. It fits both totals measured: $12.72 against $12.78 for 106
findings (phase 4), and $2.42 against $2.43 for 41 (phase 3). `audit.py`: 0 errors, 0
warnings. The fix is unevaluated: `iteration-6` checks it. `test_fixtures.py` now writes
no `__pycache__` beside the fixtures, which its import of `fixtures.py` had left. The
record's step 6 and its cost row follow the iterations. At the pause, the session had
spent $8.82 by `usage.py`, and the eval iterations $4.94.

**Where the next session starts:**
1. In a fresh usage window, from a new session: read this phase's file and this report from
   the entry "All ten tasks ticked". Prepare `iteration-6` with `workspace.py
   .claude/skills/reviewing-delivered-code --skill-only --case cost-gate --runs 3 --model
   claude-opus-5-5 --budget 3`. Show its cost (about $1.20 with the grading, $9 at most),
   wait for the user's yes, then run it, grade it and benchmark it. Class each failure in
   its `classes.json` before any change.
2. Then, with the user's yes, close this phase with the roadmap skill's `close-phase`, and
   the roadmap with `close-roadmap`: it is the last phase. All ten tasks are ticked.
- Questions put to the user: none open.
- Pending: nothing uncommitted after this session's commit; no run going. The `cost-state`
  records of the sessions of phase 4's closure and of pass 5 had not landed. When they do, they give
  pass 5's exact cost and phase 4's closing session's.

- **Found while reading phase 1: its decision to write the user's yes into each prompt**
  ("a skill that waits for the yes would otherwise stop at its estimate") **is void since
  the cost gates.** The record's step 2 says so, and its sixth change for the next
  repository follows from it.

After the commit `feat(review-skill): gate each wave of agents on the user's yes`, the user asked whether the work waits for the agent setup to
adjust its measures of cost and tokens. It does not.
- The installed `usage.py` still counts 5,400 output tokens for a subagent's call without
  its final usage. Its fix is processed in `lean-workflow` but not yet installed, so only
  the next real review's gates will show an inflated cost spent, about twice as in pass 5,
  with its estimated part named.
- `run.py`'s pooled estimate and the guard's refusal of `~/.claude` touch no assertion of
  `cost-gate`, and the case's real cost is known from iterations 3 to 5.
- The single-turn runs block only cases 1 and 2, and no run of them is planned.
- The closure reads `cost-state` records, which Claude Code writes itself.

The session cost $9.77, its `skill-auditor` included, by the tool review's measure, and
the eval iterations $4.94.

**Where the next session starts:**
1. In a fresh usage window, from a new session, without waiting for the agent setup: read
   this phase's file and this report from the entry "All ten tasks ticked". Prepare
   `iteration-6` with `workspace.py .claude/skills/reviewing-delivered-code --skill-only
   --case cost-gate --runs 3 --model claude-opus-5-5 --budget 3`. Show its cost (about
   $1.20 with the grading, $9 at most), wait for the user's yes, then run it, grade it and
   benchmark it. Class each failure in its `classes.json` before any change.
2. Every task of the phase is ticked: with the user's yes, close this phase with the
   roadmap skill's `close-phase`, and then the roadmap with `close-roadmap`, since it is
   the last phase.
- Questions put to the user: none open.
- Pending: nothing uncommitted after this record's commit; no run going. The `cost-state`
  records of the sessions of phase 4's closure, of pass 5 and of this one had not landed;
  when they do, the closure takes their exact costs.

New session, from 12:27 UTC, two minutes after the previous one ended. The user
called the window fresh. The `cost-state` records of phase 4's closure, pass 5 and the session before
had still not landed at 12:28 UTC.

`iteration-6` prepared with the resume point's command. `run.py` estimated $5.07, $1.69 a
run: the mean of 14 complete runs of the skill, phase 3's whole reviews included, against
$0.25 to $0.60 for each earlier run of `cost-gate`. The user's yes: "Fenêtre neuve,
lance", the runs first, the grading after its own yes. The three runs cost $0.77 ($0.26,
$0.24, $0.27), 51 s to 60 s each. Read from their closing messages only, before any
grading: each stopped at step 2 and started no agent, gave the review in one pass at
$12.37 with six verifiers at $0.51, and asked for the yes. Run 3 states step 7's price,
about $0.06 a finding. Runs 1 and 3 again gave the session's budget counter in place of
`usage.py`, which the runs' guard refuses (class `world`, as in `iteration-3`).

The grading was proposed: 3 `skill-grader` sessions at Sonnet `xhigh`, $0.58 by
`grade.py`. The user closed the session before answering. The session had spent $1.38 by
`usage.py`.

**Where the next session starts:**
1. Grade `iteration-6`: show `grade.py .eval-runs/skills/reviewing-delivered-code/iteration-6`
   (3 sessions, about $0.58), wait for the user's yes, then `--start`; `grade.py --again`
   on any failed assertion, after its own yes; class each failure in `classes.json`
   before any change; then `benchmark.py`. "One pass" passing in 3 of 3 meets the
   criterion; one failure asks for 5 runs (`workspace.py --extend`).
2. Write the result into `workflow.md`'s step 6 and its phase 5 cost row (`iteration-6`:
   runs $0.77, plus the grading), and here.
3. Every task of the phase is ticked: with the user's yes, close this phase with the
   roadmap skill's `close-phase`, then the roadmap with `close-roadmap`, since it is the
   last phase. Both rituals were read in this session; each starts one `roadmap-auditor`,
   whose cost no report has measured.
- Questions put to the user: the yes on the grading of `iteration-6`.
- Pending: `iteration-6`'s three runs, complete and ungraded, under `.eval-runs/`, which
  git ignores. The `cost-state` records of the four sessions before for the closure's exact costs.

New session, from about 13:55 UTC. The `cost-state` records had landed:
phase 4's closure $10.30, pass 5 $12.11, the next session $11.21 and the one after
$1.64. `usage.py` run on each whole transcript gives $10.12, $4.67, $11.09 and
$1.64: on a session's own calls it matches the record within the Sonnet part, and pass
5's $4.67 leaves out its agents. The lower figures written before ($8.80, $9.77) were read
before those sessions ended; `usage.py` does not count too low.

`iteration-6` graded after the user's yes ("tu peux lancer"): 3 `skill-grader` sessions,
$0.40 against $0.58 estimated. Each run passed 7 of 7, its three guards included, and
`benchmark.py` finds the criterion met, with no failure to class. Step 7's price holds:
each run gave the review in one pass, at $12.37. `iteration-6` cost $1.17: $0.77 for the
runs and $0.40 for the grading.
- **One grader named "One pass" weak:** an estimate far too low, $3 say, would pass it as
  well. Left as it is and put to the user: the assertion answers F3, an overrated
  estimate, and no run has given one too low.

The result is in `workflow.md`, step 6 and the phase 5 cost row, with the exact costs of
phase 4's closing session and of pass 5. Checks green before the closure: `cargo fmt
--check`, `cargo clippy`, `cargo test` (492, 23 and 2 tests, `up_a_proxied_request_and_down
... ok`), `test_scope.py` (15) and `test_fixtures.py`. The session had spent $1.77 by
`usage.py` when the closure began, and its graders $0.40.

---

## Decisions

- **Phase 5 takes the skill's changes from pass 5** (the resume note of 2026-10-16, after
  pass 5): the user's cost gates and pass 5's misfits, beside the record. Its Objective
  stands; the tasks and the acceptance criterion are added.
- **The proof of the skill's changes** (the user, 2026-10-16): each misfit is a fix from
  use, under the agent setup's `docs/decisions/2026-10-15-validation-weight.md` (quoted
  with its transcript, fixed at its source, `audit.py`, one `skill-auditor` pass). The
  cost gates get one eval case as well, since a failure there spends money unasked, a
  criterion of that decision's full regime. Pass 5's transcript is the baseline: no run
  without the skill. A run with the skill stops at the first gate when the gate holds, and
  a small fixture keeps a failed run cheap.
- **No agent starts before the user's yes in the session** (the user, 2026-10-16, after
  pass 5): the review stops before each wave of agents (a pass's reviewers, their
  verifiers, a verifier for leads) with the cost spent so far and the wave's estimate,
  and a request that accepts the cost beforehand does not count as the yes.
  `reviewing-delivered-code` cites this entry. The window check (when the previous session
  ended, what it cost, what is left) is accepted for now and stays in the agent's memory,
  to revisit if broad estimates add more noise than help.
- **Read-only GPU queries are allowed to the review's agents** (fix from use, pass 5):
  `nvidia-smi` or NVML reads, with no CUDA context, no VRAM allocated and no setting
  changed. Pass 5's findings 104 and 106 rest on them.
- **The record lives in the roadmap folder as `workflow.md`** (the user, 2026-10-16),
  beside `analysis.md`. It moves with the roadmap to `completed/`, and its links to the
  reports stay true.
- **A verifier is priced from the scope at step 2 and from its findings at step 7, on both
  scopes measured** (the user, 2026-10-16: fix the estimate now): $0.20 plus $0.14 per
  1,000 production lines, then $0.05 plus $0.005 per 1,000 lines for each finding. Each
  rate replaces a fit on the v2 scope alone, which overrated a verifier on one commit three to four times.
- **The proof of step 7's rate and the closures wait for a fresh window** (the user,
  2026-10-16), the window then holding about $14.

---

## Files Changed

**Added**
- `.claude/skills/reviewing-delivered-code/evals/test_fixtures.py`
- `docs/roadmap/on-progress/review-skill/phase-5-workflow-record-report.md`
- `docs/roadmap/on-progress/review-skill/workflow.md`

**Modified**
- `.claude/agents/review-cleanup.md`
- `.claude/agents/review-correctness.md`
- `.claude/agents/review-performance.md`
- `.claude/agents/review-security.md`
- `.claude/agents/review-structure.md`
- `.claude/agents/review-tests.md`
- `.claude/agents/review-verifier.md`
- `.claude/skills/reviewing-delivered-code/SKILL.md`
- `.claude/skills/reviewing-delivered-code/evals/evals.json`
- `.claude/skills/reviewing-delivered-code/evals/fixtures.py`
- `CLAUDE.md`
- `docs/roadmap/on-progress/review-skill/README.md`
- `docs/roadmap/on-progress/review-skill/phase-5-workflow-record.md`

Outside the diff: `.agent-conventions.toml`, which git ignores, gained the check
`test_fixtures.py`.

---

## Problems And Deviations

- **The phase grew from six tasks to ten at its opening.** The two resume notes of
  2026-10-16 added the exact cost of each pass and three tasks on the skill: the cost
  gates, the fixes from use, and one eval case. All ten are done.
- **Acceptance criterion 4 does not hold in full.** The skill stops before its first agent
  and before each later wave, but the eval case shows the first stop only: a run takes
  one turn and ends there, so no run reaches a second wave (case 3's `pending`, F2). Left
  open: the design feedback of 2026-10-16 to the agent setup proposes scripted replies.
- **Cases 1 and 2 can no longer reach a report** for the same reason. Their `pending` say
  so; no run of them is planned.
- **The estimate overrated a verifier on one commit three to four times**
  (`iteration-3`), at step 2's flat $2, then at step 7's rate fitted on the v2 code alone
  (`iteration-5`, run 1 split into two passes). Fixed at both steps from the two scopes
  measured; `iteration-6` holds at 3 of 3.
- **`iteration-4`: the gate held in 1 of 3 runs.** Two runs ran the checks and wrote the
  report before stopping ahead of the agents. Fixed: step 2's stop states its own reason;
  `iteration-5` and `iteration-6` hold at 3 of 3.
- **The runs could not reach `usage.py`**, refused by the runs' guard (class `world`).
  Each run gave the session's budget counter instead and said so. Left as it is: in real
  use the script is reachable.
- **Phase 4's report calls the exact costs of passes 2 to 4 lost; they were late.** Phase
  4's report is frozen: the correction lives here and in `workflow.md`.
- **Phase 1's decision to write the user's yes into each prompt is void since the cost
  gates.** The record's step 2 says so, and its sixth change follows from it.
- **Four claims of the record did not hold as first written,** found by its check against
  the reports and corrected before the commit.
- **"One pass" is weak on the low side** (`iteration-6`'s grader). Left open and put to the
  user; see the Work Log.

---

## Changes To Later Phases

None: phase 5 is the roadmap's last phase.

---

## Assessment

The phase delivered its record, [workflow.md](workflow.md). It holds six steps, from the
analysis of the plugins to the fixes from use, each with its inputs, actions, outputs,
decisions and report. It marks the parts specific to ai-ctl, gives the cost of each phase
with the exact figures that have since landed, and lists eleven changes for the next
repository, each checked against the reports.

It also took on the skill's changes from pass 5: eight fixes from use, each quoted from the
transcript, and the cost gates the user asked for once pass 5 had reached 95 % of a usage
window. The gates held in every run of the last two iterations. The estimate is now
priced at step 2 from the scope's lines and at step 7 from the findings. It fits both
scopes measured and keeps a one-commit review in one pass. The four iterations cost $6.11;
the phase's sessions cost $11.21 and $1.64 (exact), plus this closing one.

What the next work needs first: the fix roadmap for the v2 review's 105 findings to fix,
from the fix plan in `docs/reviews/delivered-code/2026-10-15-v2-rebuild.md`; for the next
repository, `workflow.md`'s eleven changes, read before its roadmap is written. The stop
before a later wave has no eval yet: the first real review after this phase is its proof.
