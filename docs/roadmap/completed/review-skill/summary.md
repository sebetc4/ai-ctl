# Summary — Review Skill

---

## Where We Started

On 2026-10-12, the v2 rebuild had just closed and left ai-ctl with about 14,000 production and
11,600 test lines that nobody had reviewed as a whole. The user wanted a review after each
large session or roadmap, so that technical debt would not build up. They also wanted the work
traced well enough to be done again on other repositories.

What existed was an analysis of eleven review plugins, in English and French, outside the
repository, with thirteen decisions left open. There was no review
skill. Neither `cargo-audit` nor `cargo-mutants` was installed, and `.claude/` was ignored by git.
authoring-skills, the tool meant to build and prove the skill, could not evaluate a skill that
starts agents: its runs denied the Agent tool.

---

## Where We Landed

`reviewing-delivered-code` lives in ai-ctl's versioned `.claude/`. It holds the skill, a report
template, and `scripts/scope.py`, which sets a review's scope from a roadmap's Start Commits
and has fifteen tests. Seven agents carry twelve axes: six reviewers and a verifier that tries
to refute each finding. The skill shows its estimate and waits for the user's yes before any
agent starts. It stops again, with the cost spent so far, before each later wave of agents. Its
rates are measured on two scopes, a single commit and the whole v2 code. Three eval cases and
22 trigger queries go with it, and `test_fixtures.py` joined the repository's checks.

The skill's first real review is `docs/reviews/delivered-code/2026-10-15-v2-rebuild.md`. In five
passes it raised 107 findings on the v2 code. A verifier and the user confirmed every one: 105
to fix and 2 kept, in a fix plan of 55 batches the user approved. Beside the plan sit the defects
found outside the review: phase 1's four missed bugs, and five defects found by checking its
leads and the files it read only in part. No code of ai-ctl changed.

[workflow.md](workflow.md) records the workflow for the next repositories. It gives each step,
its cost, what depended on ai-ctl, and eleven changes to make next time.

---

## What Each Phase Delivered

**Phase 0, Analysis and Decisions.** The analysis became final and moved beside the roadmap as
`analysis.md`, in English only. Performance joined the axes at the user's request. Every open
decision was taken:
- one entry-point skill, `reviewing-delivered-code`, of kind task;
- one read-only agent per axis, plus a verifier;
- reports in `docs/reviews/delivered-code/`;
- effort tiers, with an estimate shown before the review starts;
- `cargo-audit` and `cargo-mutants` to install.

The harness's block on agents went to the agent setup as a proposal, decided the same day.

**Phase 1, Evals and Baselines.** Two cases were pinned in fixtures at `docs: close the v2 rebuild roadmap` of 2026-10-12: the v2 roadmap,
and a session with eleven planted defects. Each ran without the skill, and twelve assertions per
case came from their quoted failures. The baselines reviewed what they read well. What they
lacked was the frame: no report file, no scope from the Start Commits, no test read, no coverage
stated, no verification, no fix plan. The built-in `/code-review` found six bugs, the v2 baseline
found four, and the two found eight bugs between them. The user's usage window of about $20
showed up here. From then on, each paid run took a session of its own.

**Phase 2, Skill Build.** The skill was written from those failures. The user set six reviewers
and a verifier on cost, since no baseline had started an agent. A review runs in passes when its
estimate passes a window. `scope.py`'s tests were written first. `cargo-audit` 0.22.2 and
`cargo-mutants` 27.1.0 were installed and measured. `skill-auditor` passed the skill on its
second pass.

**Phase 3, Evaluation.** With the skill, the post-session case passed 9 of 12 assertions,
against 3 of 12 without it, and the description triggered on 22 of 22 queries. The phase paused
a day while the validation was fixed at its source. It then stopped on its first iteration, by
the agent setup's stop rule: real use was to be the main proof.

**Phase 4, First Review of ai-ctl.** The skill reviewed the whole v2 code in four passes for
about $51. It confirmed 100 findings and found five of phase 1's eight bugs. The verifiers
refuted four findings and corrected about forty fixes, facts and consequences. Sixteen changes
came from that use; thirteen were made, and three new axes were added. The roadmap's
end point changed: instead of one standard review skill, the goal is now a skill that builds
review skills.

**Phase 5, Workflow Record.** The record, `workflow.md`, gives each step, its cost and what was
specific to ai-ctl, and its claims were checked against the reports. The phase also took on
what pass 5 showed:
- the user's cost gates;
- eight fixes from use;
- an estimate recalibrated so that a one-commit review fits one pass.

The `cost-gate` case held at 7 of 7 in each run of its last iteration.

---

## What We Learned

- **A review skill adds the frame, not the reading.** An agent without the skill found ten of
  eleven planted defects for $1.29. What it never produced unprompted was the frame: the scope,
  the coverage, verification by another agent, and a fix plan the user can approve.
- **Verification pays for itself.** On the v2 code, the verifiers refuted four findings. They
  caught fixes that would have added a bug or would not have caught their mutant, and corrected
  about forty facts and consequences.
- **Reviewers should run what they can run.** Two of the four refuted findings came from
  reasoning about a program that was installed and could have been run safely.
- **Read costs from the records Claude Code writes when a session ends.** Mid-session readings
  and default output figures misled more than once: `usage.py`'s default was 6.4 times too high
  for the review agents. A verifier's cost follows its findings, and each pass pays for its own
  session.
- **Fit the rates on more than one scope.** A rate fitted on the v2 code alone overrated a
  verifier on one commit three to four times, and split a review that fits one pass into two.
- **Plan paid work by the usage window, not the session.** A new session is not a fresh window,
  and a request that accepts the cost beforehand is not the user's yes. Cost gates belong in the
  skill that starts the agents, not in the agent's memory.
- **Check the eval harness against the skill's shape first.** Agents that did not exist yet cost
  two proposals and two sessions. Runs that take one turn can show a gate only up to its first
  stop.
- **When validation fails, fix it at its source.** A failed eval is classed before anything
  changes. A skill or an assertion bent around a gap in the tooling is reverted, and the gap goes
  to the tooling's owner.
- **Real use is the main proof.** One iteration against the baseline, then a real review, gave
  more evidence than more eval runs would have, at a fraction of their cost.
- **A repository's specifics weigh more than expected.** The commands safe to run, the checks,
  the threat model and the reviewers' methods were all ai-ctl's own. Only the orchestration and
  the reviewers' disciplines would hold anywhere.

---

## What We Are Leaving Open

- **The fix roadmap** for the review's 105 findings and the defects found outside it, from the
  review's report. It has not been created yet.
- **The stop before a later wave has no eval run.** A run takes one turn, so the eval shows only
  the first gate, and phase 5's acceptance criterion 4 holds only in part. The whole-review cases
  also end at step 2. A design feedback to the agent setup proposes scripted replies.
- **The skill's changes from use are unevaluated beyond the trigger eval and `cost-gate`.** Those
  changes include the twelve axes, the fixes to `review-correctness` for its three missed bugs,
  and pass 5's eight fixes. The next real review is their proof. Two of the three new axes answer
  the user's request, not a defect anyone observed.
- **The `cost-gate` assertion "One pass" is weak on the low side.** It sets a $15 ceiling and no
  floor. This is put to the user.
- **Changes owed by the agent setup:** the `failures_in_use` key, and `usage.py`'s output default
  for subagent calls. Both are handled in its `lean-workflow` roadmap and not yet installed.
- **The other repositories and the skill that builds review skills.** Both are out of scope.
  `workflow.md` is their input.
