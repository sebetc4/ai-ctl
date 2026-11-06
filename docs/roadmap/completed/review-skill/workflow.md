# Workflow: A Review Skill For One Repository

How the review-skill roadmap took ai-ctl from an analysis of review plugins to a review
skill used for real, written so that the same work can be done again on another repository.
Each step gives its inputs, actions, outputs and decisions, and links the report that holds
the details. Lines marked **ai-ctl** held for this repository only; the rest would hold
anywhere. The record states what the reports say. It does not design the skill that will
build review skills.

---

## The Steps

### 1. Analyse The Review Tools, Take The Open Decisions

[Phase 0 report](phase-0-analysis-and-decisions-report.md), `analysis.md`, removed on 2026-10-27.

- **Inputs:** the user's axes; copies of eleven review plugins, outside git; the
  repository's conventions.
- **Actions:**
  - wrote an analysis of the plugins against the repository, which the user read for its
    main lines and held final;
  - gathered the facts each open decision depends on: licences, git remotes, the plugin
    policy of the agent setup, the install model, what the eval harness can run, the tools
    installed;
  - put the thirteen decisions to the user in groups.
- **Outputs:** `analysis.md`, in English only, in the roadmap folder; the `[skills]` table;
  the skill's intent in the user's words.
- **Decisions:**
  - one entry skill, one read-only agent per axis and a verifier;
  - the skill and its agents in the repository's versioned `.claude/`;
  - a gerund name, `reviewing-delivered-code`;
  - one report per review in `docs/reviews/delivered-code/`, ending with a fix plan;
  - effort tiers, each with an estimate shown before the review;
  - kept legacy code marked with a `// LEGACY:` comment.
- **ai-ctl:**
  - the tools chosen, `cargo-audit` and `cargo-mutants`;
  - the security method rewritten from the vulnerability-scanning plugin rather than copied: its licence
    forbids distributing a modified version, and ai-ctl has no remote;
  - one comparison run of the built-in `/code-review` for the bugs axis.

### 2. Write The Cases, Run Them Without The Skill

[Phase 1 report](phase-1-evals-and-baselines-report.md).

- **Inputs:** the intent; authoring-skills' eval files; the repository at the commit that
  closed the roadmap under review.
- **Actions:**
  - built each case's folder with `evals/fixtures.py`: the commit fetched alone, its
    history kept, no later document, a pinned `CLAUDE.md`;
  - wrote two cases: the whole roadmap's code, and one session's commit with eleven
    defects planted across the axes, its checks left green;
  - ran one baseline per case at the user's real model and effort, one paid run per usage
    window;
  - ran the built-in `/code-review` once on the code;
  - quoted each failure, wrote twelve assertions per case from them, and recorded what the
    baselines did right so that the skill would not teach it.
- **Outputs:** `evals.json`, failures F1 to F10 and F1 to F7, and the eight bugs the runs
  found together, the measure of the bugs agent.
- **Decisions:**
  - the baselines get the Agent tool; this took two changes to authoring-skills, proposed
    from here;
  - the planted session is a commit, so the run's own changes stay apart from it;
  - each prompt names a tier and accepts the cost, so that a run would not stop at its
    estimate. Step 6 voids this.
- **ai-ctl:** the planted defects are Rust (a text sort, an unquoted `sh -c`, an
  `.unwrap()`); the checks are `cargo fmt`, `clippy` and `cargo test`.

### 3. Build The Skill

[Phase 2 report](phase-2-skill-build-report.md).

- **Inputs:** the quoted failures; the user's statements; the eight bugs.
- **Actions:**
  - wrote `scripts/scope.py` test first, fifteen tests watched failing;
  - installed the two tools and measured them on the planted session;
  - wrote `SKILL.md`, the report template and seven agents, each block tied to a failure
    or a statement, then cut what traced to neither;
  - one `skill-auditor` pass, FAIL then PASS.
- **Outputs:** the skill, its template, `scope.py`, six reviewers and a verifier.
- **Decisions:**
  - six reviewers, not nine, set on the cost the usage window allows, since no baseline
    had started an agent;
  - a review whose estimate passes a window runs in passes.
- **ai-ctl:**
  - `scope.py` counts production and test lines by the `#[cfg(test)]` modules;
  - the commands safe to run (`doctor`, `status`, `build list`; `cargo test` with its
    fake llama-server);
  - the threshold of 300 mutants, from a probe.

### 4. Evaluate The Skill Against Its Baseline

[Phase 3 report](phase-3-evaluation-report.md).

- **Inputs:** the cases and their baselines, reused.
- **Actions:**
  - one run with the skill on the planted session, graded, benchmarked, read by the user;
  - the trigger eval;
  - changes from the run: findings merged by cause before verification, the estimate
    recalibrated, a step for `review-tests`;
  - a pause of a day, to fix the validation at its source rather than bend the skill
    around it.
- **Outputs:** 9 of 12 assertions with the skill against 3 of 12 without; 22 of 22
  trigger queries.
- **Decisions:**
  - the phase stopped after one iteration, by the stop rule of the agent setup's decision
    of 2026-10-15, the default regime: real use is the main proof;
  - the whole-roadmap case never ran with the skill, and the real review stood in for it.

### 5. Review The Repository For Real

[Phase 4 report](phase-4-first-review-report.md), [the review's
report](../../../reviews/delivered-code/2026-10-15-v2-rebuild.md).

- **Inputs:** the v2 code, 14,016 production and 11,591 test lines.
- **Actions:**
  - four passes, each in its own session: two reviewers and their verifiers in passes 1
    and 4, two reviewers in pass 2, and their verifiers in pass 3;
  - the user's verdict on each finding;
  - five checks of the leads and of the files read in part;
  - sixteen changes to the skill proposed from use, thirteen made, and three axes added;
  - after a reboot, pass 5 on the GPU paths.
- **Outputs:**
  - 107 findings, 105 to fix and 2 kept, in a fix plan of 55 batches;
  - five defects found outside the review;
  - the skill on twelve axes;
  - the review's measure: 5 of the 8 bugs.
- **Decisions:**
  - each reviewer's findings go into the report as soon as it answers, so a cut session
    loses only the agents running;
  - verification is priced from the findings;
  - the end point is a skill that builds review skills, not one standard skill.
- **ai-ctl:**
  - the verifiers ran llama-swap alone on a loopback port, podman and copies of the crate;
  - the GPU paths waited for a reboot, because the NVML driver and its library had
    different versions.

### 6. Fix The Skill From Use, Gate Its Cost

This phase's [report](phase-5-workflow-record-report.md).

- **Inputs:** pass 5's transcript; the user's decision on its 95% usage window.
- **Actions:**
  - eight fixes from use, each quoted from the transcript;
  - the cost gates: no agent before the user's yes in the session, and a stop before each
    wave of agents;
  - `audit.py` and one `skill-auditor` pass;
  - one eval case for the first gate, three runs per iteration;
  - the estimate recalibrated on both scopes measured: a verifier priced from the scope's
    lines at step 2, and from its findings and the lines at step 7.
- **Outputs:**
  - the gate held in 3 of 3 runs, then in 1 of 3 once the estimate fitted one pass, then
    in 3 of 3 once step 2's stop stated its own reason;
  - the review in one pass in 3 of 3 runs once step 7 priced the verification from its
    findings; at a rate fitted on the v2 code alone, one run of 3 had split it into two;
  - a design feedback to the agent setup.
- **Decisions:**
  - a cost accepted in the request no longer counts as the user's yes;
  - since a run takes one turn, a run with the skill ends at the first gate, and the
    whole-review cases cannot run past step 2 until the harness can answer a run.

---

## Cost Of Each Phase

The figures the reports give. The exact ones come from the `cost-state` records, which
Claude Code writes when a session ends, and sometimes hours later. The others come from
`usage.py`.

| Phase | Dates | What was paid | Cost |
|---|---|---|---|
| 0 | 2026-10-13 | no paid run | not recorded |
| 1 | 2026-10-13 | 2 baselines, 1 stopped run, 2 runs of `/code-review` | $1.29, $4.51, $0.68; $2.82 and $3.13 (exact) |
| 2 | 2026-10-13 | no paid run; 2 `skill-auditor` passes | not recorded |
| 3 | 2026-10-13 to 2026-10-15, paused on 2026-10-14 | 1 run with the skill (13 min), 3 gradings, the trigger eval | $8.31, $1.13, $3.26 |
| 4 | 2026-10-15 to 2026-10-16 | 4 review passes; the verdicts; the closing session and its trigger eval | $51.61 (exact: $11.89, $14.41, $9.62, $15.69); $7.17 (exact); $10.30 (exact) and $3.27 |
| Pass 5, after phase 4 | 2026-10-16 | 2 reviewers, 3 verifiers | $12.11 (exact) |
| 5 | 2026-10-16 | 1 `skill-auditor` pass; 4 iterations of 3 eval runs, with their grading; 3 sessions | $1.54, $2.25, $1.15, $1.17; $11.21 and $1.64 (exact), the closing one $1.77 by `usage.py` when the closures began |

The sessions' own cost is recorded for phase 4 and later only, where it held $3.14 to
$4.27 per pass. Tokens are recorded for a few runs only: pass 1, 20.5 million cache reads
and 188,453 output tokens; the two `/code-review` runs, 38,948 and 29,424 output tokens.
Each review agent ran 4.7 to 14.9 minutes.

---

## What Depended On ai-ctl

- **ai-ctl only:**
  - the commands safe to run, and the reasons for them: `src/presentation/cli.rs`, a stack
    on the GPU;
  - the checks, `cargo audit` and `cargo mutants`, and the threshold of 300 mutants;
  - `scope.py`'s count of test lines by `#[cfg(test)]`;
  - the attackers of a local, single-user tool;
  - the reviewers' methods, written against Rust and ai-ctl's layers;
  - what the verifiers ran: llama-swap on a loopback port, podman, read-only NVML,
    copies of the crate;
  - the planted defects in Rust;
  - the machine: one GPU, and a driver update that blocked the GPU paths until a reboot.
- **The user's, across repositories:**
  - a usage window of about $20 per plan;
  - the roadmap skill's Start Commits, from which the scope is read;
  - the legacy rule;
  - French in conversation, English in files.
- **Any repository:**
  - a scope from commits, tiers, an estimate and passes within a window;
  - findings under To Verify as they arrive;
  - a verifier that tries to refute each finding, and leads sent to a verifier;
  - the user's verdict per finding, and a fix plan in batches;
  - the cost gates;
  - the reviewers' disciplines: run what can run, try a proposed test against its mutant,
    compare each site before raising a copy.

---

## What To Change For The Next Repository

From the problems and deviations the reports recorded:

1. Load authoring-skills before any decision about a skill, its name included: the first
   three names broke its naming rule (phase 0).
2. Check, before writing the cases, that the eval harness can run the skill's shape:
   agents, agents not yet written, and stops for the user. The first two cost two
   proposals and two sessions (phases 0 and 1). The third has no answer yet (step 6).
3. Plan paid runs by the usage window, not by the session: one paid run per window, the
   user asked what the window has left, and a stop before each wave of agents (phases 1,
   4 and 5).
4. Read costs from the `cost-state` records once they land. `usage.py`'s default output
   per subagent call ran 6.4 times too high for the review agents (phases 2 and 4).
5. Calibrate the estimate from measures, and price the verification from the findings
   once they exist. Flat rates miss at both ends: a commit was overrated 1.7 times, and
   on the v2 scope the sessions were counted once for four passes (phases 3, 4 and 5).
6. Do not write the user's yes into an eval's prompt. With the gates, a run checks a
   skill up to its first stop and no further (phase 5).
7. Have each reviewer run what it can run, try each proposed test against its mutant, and
   compare each site before raising a copy. Two of the four refuted findings, and the
   four fixes that would not have worked, came from not doing so (phase 4).
8. Give the report places for findings waiting for a verifier, leads, the user's verdicts
   and later passes. The first review improvised each one (phases 4 and 5).
9. Number each finding once and never renumber it: a script renumbered 100 findings and
   still missed two references (phase 4).
10. When an eval fails, class the failure first, and never bend the skill or an assertion
    around a gap of the validation: record it, pause, and fix it at its source (phase 3).
11. Pin the cases at a commit before the roadmap's own documents, so that no run reads the
    skill's design (phase 1).
