# Phase 3 Report: Skill And Agents

**Phase:** [phase-3-skill-and-agents.md](phase-3-skill-and-agents.md)
**Start Commit:** `docs(review-skill-v2): close phase 2` of 2026-11-02

---

## Work Log

### 2026-11-02

Phase opened; the user: "tu peux y aller", for `SKILL.md`'s rewrite.

`SKILL.md` rewritten against the five cases' assertions and failures: the moments in one table
with their scope, agents and fixing rule; the scope by hunk; one go-ahead with `usage.py
estimate`'s line, the request that accepts the cost taking its place, `--start` before each start
of agents; the review's folder kept by `findings.py`; the mutants in the background in a
`git archive` copy; the session's triage, the floor and the register; the fixes where the moment
says; `findings.py tasks` in place of a clean-up roadmap; continuing from `findings.json`'s
status. A reading against the cases found four gaps, fixed before the commit: the plan file left
in `/tmp` when the user declines, `cost-gate`'s guard; `--start` written for the reviewers alone;
mutants and a verifier on a review of the dependencies alone; and no taker for the missed mutants
at a roadmap's close. The fix method went to `references/fixing.md`, from the second assessment
of the review workflow: sibling paths, input classes, a document's statements, the mutants by
hand, new tests 30 times. The description names the thirteen axes, the roadmap's closing pass
and the crates' checks, the three queries phase 1 saw missed. `SKILL.md`: 211 lines, 13,812
characters, the description 950; `audit.py` clean after each write. A search of the skill's
folder and the agents finds no roadmap, phase or report cited, no rate and no plan figure.

The agents, after the user's choices of the rules skill's name and of the anchors. The rules
went to `reviewing-as-an-agent`, preloaded by each agent's `skills:`; the seven agents lost
their copies of the rules and of the answer's frame, kept their axis's method and gained a
section on the answer's fields, and moved to `scope.txt` by hunk. `review-security` took
phase 0's method; `review-verifier` its reproductions from a git copy in scratch and VERDICT
blocks; `review-dependencies` is new. Two needs of these formats went to `findings.py`,
test-first: an agent type listed once per run, which the `high` tier's verifiers need, the
test watched failing on the refusal, and an answer's FACTS kept and reported, the test
watched failing on the missing facts; a verifier's answer, its leads as FINDING blocks beside
its VERDICT blocks, was already read as the session needs. Five mutants applied by hand to
the new lines, all caught; the three new tests 30 runs green. The rules skill's example
answer and the verifier's frame parse with `findings.py`'s reader. A reading for echoes of the
evals' answer keys found four, replaced by real cases: the verifier's example, the fixing
method's sibling paths, two of the anchors' examples, and `review-security`'s run in the
attacker's place. `SKILL.md`'s step 3 gained what to do when `cargo mutants`,
`dependencies.py` or `regression.py` fails. Both skills' audit clean, every check green; a
search of the agents finds no `docs/decisions`, `docs/notes`, `--files` or `file:` line.

`CLAUDE.md`'s convention for the reviews at a phase's and a roadmap's close, put to the user as
proposed text and added as it stood (the user: "tu peux ajouter").

The proofs, task by task, at the closure. Run in this phase: `audit.py` clean on both skills
after each write, for `SKILL.md`, the fix method, the description, the shared rules,
`review-security`, `review-verifier` and `review-dependencies`. Not run: every part that reads
"phase 4's runs", "phase 4's case on a fix", "phase 4's security case", "phase 4's dependency
case" and "phase 4's trigger rates against phase 1's", the runs being phase 4's, outside this
phase's scope, and paid. The report on the dependency axis: phase 2's report writer tests,
`test_findings.py`, 29 passed on the new `assets/report.md`, the fixtures' five reports among
them written again byte for byte, the frame holding each part once, and the Facts section shown
by the new test. The structured finding: the reviewers no longer have a format each but the one
of `reviewing-as-an-agent`, whose example answer `findings.py`'s reader takes whole; the
verifier's frame, VERDICT blocks with no lead, read as an answer of none; `review-dependencies`'
answer with FACTS taken by the new test of `test_findings.py`. The rules in one file: a search
of the agents and both skills finds the rules on what may run, podman's, the `ai-ctl-` prefix and
the whole answer given again in `reviewing-as-an-agent` alone, the review skill keeping only the
session's own rule on ai-ctl's commands, and the verifier naming the frame it shares.

The closure's audit failed twice on the tasks whose proof is a run of phase 4 that no task of
phase 4 named; at the user's choice ("Nommer dans la phase 4 (Recommandé)"), phase 4's first task
now names the proofs it carries for phase 3.
A third audit failed on the same six tasks, a part of whose proof is a run of phase 4; at the
user's choice ("Réécrire les Proof (Recommandé)"), their Proof lines keep what ran in this phase
and move their runs to the phase 4 tasks that name them.

---

## Decisions

- **Verifiers above `medium`: one per reviewer at `high`, `xhigh` and `max`,** where the skill
  gave one per finding at `xhigh` and `max`: the one go-ahead counts its agents before any
  finding exists.
- **Each agent writes its whole answer to `<scratch>/<agent>/answer.txt`,** which `findings.py
  add` reads: the session copies no answer by hand. Commits the agents' rules to it.
- **The session is an agent of the review, `session`,** with one answer for its own findings —
  its review at `low`, a red check, a recorded mutant missed or gone, a missed mutant no reviewer
  takes —, `FINDINGS: 0` when none, added once the checks, the mutants and the regression run
  have ended.
- **A verifier writes its reproductions in its scratch folder,** the guard blocking a stop that
  leaves the repository changed; the session copies the confirmed findings' into the review's
  `repro/<n>/`. Commits `review-verifier`'s rewrite to it.
- **A dependency finding is verified by the session** against `cargo deny check`'s output and
  `dependencies.json`, with no verifier: the `dependency-check` case starts no agent but
  `review-dependencies`.
- **The scratch folder is made at the go-ahead,** for the plan `usage.py` reads, and removed on a
  no; `review-tests` starts once the mutants end, with their missed list; a review of the
  dependencies alone has `HEAD` for its base and end.
- **The agents' shared rules are the skill `reviewing-as-an-agent`,** marked
  `user-invocable: false` (the user, 2026-11-02: "reviewing-as-an-agent (Recommandé)"): a skill
  with `disable-model-invocation: true` cannot be preloaded by `skills:`
  (https://code.claude.com/docs/en/sub-agents, Preload skills into subagents). Phase 4 measures
  that review requests do not trigger it.
- **The severity anchors common to every axis, by what the user meets** (the user, 2026-11-02:
  "Table par effet (Recommandé)"): `critical`, data or control lost on a command's ordinary use;
  `high`, data the user cannot get back lost on an input the user may well have; `medium`, a
  wrong action or a misleading result on an ordinary path that the user can undo or redo; `low`,
  a wrong action on a rare input, a message lacking what the user needs, a misleading document.
  The lower between two, and with no concrete damage; a test gap at the severity of the defect
  it lets through; `review-security`'s table on its axis. The fixture's 17 findings keep their
  severities under them. Two of `medium`'s examples, close to two planted defects of the evals,
  became a backend left running after `down` and a status showing a model loaded that is not.
- **The examples of the skills and agents come from real reviews, never from the evals'
  fixtures:** a fixture's planted defect in a skill's text gives the runs their answer key.
- **Each agent writes its answer with the Write tool, added to its tools;** the guard's check
  at the stop catches a write outside its folder.
- **`review-verifier` answers VERDICT blocks, which the session writes with `findings.py set`,
  and its leads as FINDING blocks, which `findings.py add` reads;** a lead above the floor gets
  its verdict from the session, from the verifier's reproduction run again, with no further
  verifier, so that the go-ahead's count holds. Its reproductions come out of a git copy of the
  scope's end in its folder, tagged `end`, as `test.patch` and `fix.patch`.
- **`findings.py`: an agent type given once per run at `new`, `add` filling the first run
  still to answer; an answer's FACTS kept on its agent and shown in the report's Facts.**
- **`review-dependencies` judges and reaches no API:** an advisory by its reachability, a
  security finding when reached; the updates within the requirements one finding below the
  floor, fixed at once; a version beyond them a finding only for an advisory or a yanked lock;
  maintenance figures and duplicates facts, a finding only on a RustSec warning or an archived
  repository; llama-swap's lag from `PINNED`, each backend's from `ai-ctl build list`, against
  `git ls-remote --tags`.

---

## Deviations

- `findings.py new` refused an agent type listed twice, while the `high` tier counts a
  `review-verifier` per reviewer, and a verifier's leads had no way into `findings.json`: fixed
  in this phase, `findings.py` changed though the phase's files to modify did not name it, with
  `assets/report.md`'s Facts.
- The six reviewers called `scope.py --files` and read its `file:` lines: rewritten in this
  phase for the scope by hunk.
- The rules skill has no evals of its own: `phase-4-evaluation.md`'s trigger task now writes its
  trigger queries first, in its `evals/evals.json`, and `skill-auditor` judges both skills.
- `phase-4-evaluation.md`: its first task names the proofs it carries for phase 3 — every case
  for `SKILL.md`, `phase-close` for the fix method, `roadmap-close` for `review-security` and
  `review-verifier`, `dependency-check` for `review-dependencies` —, each recorded in its report;
  its trigger task carries the description's.
- `phase-3-skill-and-agents.md`: the Proof lines of six tasks — `SKILL.md`, the fix method, the
  description, `review-security`, `review-verifier`, the dependency agent — keep what ran in this
  phase, and their runs move to phase 4's first task and its trigger task.
- The phase closed without the review of its hunks that `CLAUDE.md`'s new convention asks of a
  phase that changes code, at the user's word ("on peut fermer sans review"): the convention
  came with this phase, its code is the skill's own Python, and the v2 it would run is first
  evaluated in phase 4.

---

## Assessment

The phase wrote the v2 of `reviewing-delivered-code` and of its agents on phase 2's scripts.
`SKILL.md` runs a review by moment — a phase's close, a roadmap's closing pass, a session, a
security pass, the dependencies, a full review on request — with one go-ahead, the scope by
hunk, the mutants in the background, the session's triage over the floor and the register,
the fixes where the moment says and no pass after a roadmap's closing pass; the fix method is
`references/fixing.md`. The agents' rules are one skill, `reviewing-as-an-agent`, preloaded by
`skills:`, with the answer's exact format and the severity anchors the user chose;
`review-security` follows phase 0's method, `review-verifier` leaves a red test and a settled
fix, and `review-dependencies` is new. Two needs of the formats went to `findings.py`: an agent
type run more than once, and an answer's facts in the report. The skill cites no roadmap and no
rate, and its texts hold no echo of the evals' answer keys, four of which a reading found and
replaced. `CLAUDE.md` now asks the reviews at a phase's and a roadmap's close.

Nothing of this has run yet: every proof that reads "phase 4's runs" waits there. Phase 4
needs first: the five cases run on the v2 after the user's yes, the baselines of
`continued-review`, `roadmap-close` and `dependency-check` again under `--reuse`; the runs'
folders given the project's `.claude/settings.json` for the guards, and the rules skill, now in
`evals.json`'s `skills`, beside `review-dependencies` in its `agents`; the rules skill's trigger
queries written first in its own `evals/evals.json`; and the world's known gaps — the hook that
refuses `~/.claude`, where `usage.py` sits, the runs' $5 ceiling, `run.py` counting the agents
as files changed, the triggers' copy holding `CLAUDE.md` and this roadmap's files.
