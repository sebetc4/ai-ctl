# Roadmap: Review Skill

Builds the `reviewing-delivered-code` skill, which reviews the code a session or a roadmap
delivered on twelve axes with reviewer agents and a verifier.

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
Phase 0  Analysis and Decisions     🟢 ████████████████████ 100%  (10/10)
Phase 1  Evals and Baselines        🟢 ████████████████████ 100%  (9/9)
Phase 2  Skill Build                🟢 ████████████████████ 100%  (8/8)
Phase 3  Evaluation                 🟢 ████████████████████ 100%  (6/6)
Phase 4  First Review of ai-ctl     🟢 ████████████████████ 100%  (10/10)
Phase 5  Workflow Record            🟢 ████████████████████ 100%  (10/10)
TOTAL                                  ████████████████████ 100%  (53/53)
```

**Current Phase:** —
**Blocked By:** —
**Next Milestone:** —

---

## Why This Roadmap Exists

Large coding sessions and roadmaps, such as the v2 rebuild closed on 2026-10-12, leave code that
nobody has reviewed as a whole. The user wants a review after each such session so that technical
debt does not build up: architecture, dead and legacy code, refactoring, conventions, bugs, error
handling, security, tests and performance (added in phase 0), then on-disk formats, interruption
and concurrency, and the CLI's contract (added in phase 4), on the code written and the code
around it.

This roadmap takes that review from the analysis of existing review plugins to a skill tailored to
ai-ctl, with one report per phase. The reports keep a complete trace of the workflow, so it can be
analysed and done again on other repositories (Node, Python, Bash…), each getting a review skill
of its own. Changed in phase 4: comparing those runs will later produce a skill that builds review
skills, from the part every repository shares and the construction of the parts each one needs,
rather than one standard skill for every repository.

---

## Decisions Taken At Opening

- **Name:** `review-skill`. The skill's own name is a decision of phase 0.
- **Purpose:** trace the whole workflow, from analysis to skill, so it can be reproduced on other
  repositories.
- **Scope:** up to a first real review of ai-ctl's v2 code, report only; the fixes it calls for
  go to a separate roadmap.
- **The analysis stays open:** the roadmap does not fix the content of the analysis report
  and its French translation. The user may still
  add, change or remove points; the choices the report prepares are made in phase 0. Changed in
  phase 0: the user held the report final, and it now lives in this folder as
  `analysis.md`, removed on 2026-10-27.
- **Method:** authoring-skills' procedure. The evals come before any other file of the skill; the
  cases run first without the skill; the skill holds only what answers an observed failure or a
  requirement the user stated. Paid runs start only after the user's yes on their count and cost.
- **Language:** exchanges with the user in French; roadmap files in English; the analysis report
  kept in English (reference) and French. Changed in phase 0: the French translation is dropped,
  and the report is kept in English only, as `CLAUDE.md` asks of every document.
- **Contract:** `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test`
  must pass to close a phase.

---

## Deliberately Out Of Scope

- Fixing the findings of the first review: a separate roadmap.
- Any change to ai-ctl's code or configuration, new lints included.
- The work on other repositories, and the skill that builds review skills from the comparison.

---

## Phases

- 🟢 [Phase 0: Analysis and Decisions](phase-0-analysis-and-decisions.md) — the analysis report made final with the user's feedback, its open decisions taken, the skill's intent and kind written, the `[skills]` table set
- 🟢 [Phase 1: Evals and Baselines](phase-1-evals-and-baselines.md) — realistic cases, run without the skill; each failure quoted and turned into an assertion
- 🟢 [Phase 2: Skill Build](phase-2-skill-build.md) — the skill and its parts written from the failures and the user's statements, scripts tested first, audit passed
- 🟢 [Phase 3: Evaluation](phase-3-evaluation.md) — runs with the skill graded against the baseline, iterations until the failures are gone, description trigger rates measured
- 🟢 [Phase 4: First Review of ai-ctl](phase-4-first-review.md) — a real review of the v2 code, every finding judged with the user, confirmed findings listed for a fix roadmap, no code changed
- 🟢 [Phase 5: Workflow Record](phase-5-workflow-record.md) — the workflow written so it can be done again on another repository, with what was specific to ai-ctl marked

---

## Dependencies

- Each phase builds on the previous one.
- Phase 0 needs the user's reading of the analysis report.
- Phases 1, 3 and 4 need the user's yes on the cost of each batch of paid runs.

---

## Related Documentation

- [docs/roadmap/completed/v2-rebuild/summary.md](../../completed/v2-rebuild/summary.md) — what the v2 rebuild delivered: the code the first review covers
- `analysis.md` — the analysis of the review plugins, held final in phase 0, removed on 2026-10-27

---

## Metadata

**Roadmap Status:** 🟢 Done
**Location:** `docs/roadmap/completed/review-skill/`
**Version:** 2.0.0
**Created:** 2026-10-12
**Last Updated:** 2026-10-16

---

## Changelog

### 2.0.0 (2026-10-16)

Roadmap closed, every phase done. [summary.md](summary.md) records where it started, the
skill and the first review it arrived at, what each phase delivered, what holds beyond ai-ctl,
and what is left open: the fix roadmap from the review's report first, then the stop before a
later wave, which no eval run shows yet. The folder moved from `on-progress/` to
`completed/`. No restructuring was pending.

### 1.6.0 (2026-10-16)

Phase 5 closed, the last phase: the workflow is recorded in [workflow.md](workflow.md),
beside `analysis.md`. Its six steps, from the analysis of the plugins to the fixes from
use, each give their inputs, actions, outputs, decisions and report. It marks what
depended on ai-ctl, gives each phase's cost, and lists eleven changes for the next
repository.

Delivered with it: the cost gates the user asked for after pass 5. No agent starts before
the user's yes in the session, and the review stops before each wave of agents. Eight
fixes from use came from pass 5's transcript. The estimate was recalibrated on both scopes
measured. Case 3, `cost-gate`, holds at 7 of 7 in each run of `iteration-6`.

Found: a run takes one turn, so no eval run shows the stop before a later wave, and the
whole-review cases end at step 2; the design feedback to the agent setup proposes scripted
replies. The exact costs phase 4's report called lost had only landed late.

### 1.5.0 (2026-10-16)

Phase 4 closed: the skill's first real review, on the whole v2 code (14,016 production and
11,591 test lines), in four passes and about $51 against an estimate of $39.

Delivered:
- the review's report, `docs/reviews/delivered-code/2026-10-15-v2-rebuild.md`: 100
  findings, each confirmed by a verifier and by the user, 98 to fix and 2 kept, in a fix
  plan of 51 batches, with the defects found outside the review beside it;
- the skill corrected from use: the measured rates, a review that keeps each reviewer's
  findings as they arrive so that a cut session loses only the agents running, leads sent
  to a verifier, the user's verdict in each finding, and reviewers that run what they can
  run; the trigger eval holds at 22 of 22.

Found: the review finds five of the eight bugs phase 1's runs had found; four of its
findings were refuted, and about forty fixes, facts or consequences corrected by the
verifiers.

Changed with the user: the review has twelve axes, on-disk formats, interruption and
concurrency, and the CLI's contract joining the nine; and the roadmap's end point is a
skill that builds review skills, from a common part and the construction of each
repository's own, rather than one standard skill. Phase 5's overview follows.

### 1.4.0 (2026-10-15)

Phase 3 closed on its first iteration with the skill, by the stop rule of the agent setup's
decision of 2026-10-15: the proof follows the tool's risk, and real use is the main loop.

Measured:
- the post-session case passes 9 of 12 assertions with the skill, against 3 of 12 without;
- the description triggers on 22 of 22 queries, 14 of 14 in training and 8 of 8 held out.

Each failure left keeps its class:
- defect 8: `assertion`;
- defect 10 and the fix plan: `skill`, fixed and unevaluated;
- Claims hold: `tool`, fixed in skill-tooling 0.1.2.

Changed on 2026-10-14 and unevaluated: the findings are merged by cause before verification,
the estimate is recalibrated, and `review-tests` gains a step.

The phase was paused from 2026-10-14 to 2026-10-15 for the update of the validation, at its
source.

Moved to phase 4: the count of the eight bugs, which phase 4's real review of the v2 code
measures in place of the v2 case's run; one `skill-auditor` pass before that review; the
estimate set against its per-agent costs.

Cut with the user's approval: the rerun of the whole set and the blind comparison. Phase 5's
record is to be short.

Found: `audit.py`'s T1 flags the axis name `legacy`, reported to the agent setup.

### 1.3.0 (2026-10-13)

Phase 2 closed. The skill `reviewing-delivered-code` is written in `.claude/skills/`: ten
steps, each answering a failure of the baselines or a statement of the user, a report
template, and `scripts/scope.py`, which sets a review's scope from a roadmap's Start Commits
and counts its production and test lines, with fifteen unit tests written first. Seven agents
in `.claude/agents/` carry the nine axes: six reviewers (security, correctness, tests,
cleanup, structure, performance) and a verifier that tries to refute each finding. Their
number was set with the user on cost, since no baseline had started an agent; a review whose
estimate passes one usage window runs in passes, the report holding its state.
`cargo-audit` 0.22.2 and `cargo-mutants` 27.1.0 are installed; on the planted session's diff,
cargo-mutants' 25 mutants ran in 68 s and named the adapter's untested functions and the
dead `newest`. The audit reports no
error and `skill-auditor` passed the skill on its second pass. Found: the v2 case at `high`
needs several passes, so phase 3 rewords its prompt or records a partial run; phase 3 also
gains a task counting the eight bugs of phase 1 in the runs with the skill.

### 1.2.0 (2026-10-13)

Phase 1 closed. The skill's evals hold two cases, each run in a fixture of ai-ctl at
`docs: close the v2 rebuild roadmap` of 2026-10-12: a review of the v2 roadmap's code, and a session with eleven planted defects. Both
ran without the skill, at Opus 5.5 `xhigh`, with the Agent tool once the agent setup gave it to
a skill whose agents are not written yet (`starts_agents: true`). Their failures are quoted and
turned into twelve assertions per case: no report file, the scope not set from the Start
Commits, the test code not read, no axis or coverage stated, the tier ignored, no verification
by another pass, no fix plan, claims of reading that do not hold, scratch left in `/tmp`. What
both baselines did right is recorded so the skill does not teach it. The built-in
`/code-review` ran once on `src/` at `high`, after a first run on the wrong scope: six bugs,
two shared with the v2 baseline's four, no test read, nothing verified. Found: the user's plan
caps spend at about $20 per usage window, so each paid run takes a session of its own; phases
2 to 4 carry the constraint. Phase 2's Dependencies name the built-in tool's gaps for the bugs
agent.

### 1.1.0 (2026-10-13)

Phase 0 closed. The analysis of the review plugins is final and lives beside the roadmap as
`analysis.md`, in English only, the French translation dropped; it records each plugin's
version and source, and gained a ninth axis, performance, at the user's request. Every
decision it left open is taken: one entry-point skill, `reviewing-delivered-code`, of kind
task, with one read-only agent per axis and a verifier, their number set after the baselines;
the skill and its agents in ai-ctl's `.claude/`, now versioned; reports in
`docs/reviews/delivered-code/`; effort tiers with a cost estimate before each review;
`cargo-audit` and `cargo-mutants` to install in phase 2. The `[skills]` table is set. Found:
authoring-skills cannot evaluate a skill that starts agents; the change is proposed in
the agent setup's note on eval runs with agents, of 2026-10-13, and phase 1's paid runs are
blocked until it lands. Phases 1, 3 and 4 were adjusted to the decisions taken.

### 1.0.0 (2026-10-12)

Roadmap created with six phases: Analysis and Decisions, Evals and Baselines, Skill Build,
Evaluation, First Review of ai-ctl, and Workflow Record. The content of the analysis report stays
open; the choices it prepares are made in phase 0. The fixes of the first review, any change to
ai-ctl's code, and the work on other repositories are out of scope.
