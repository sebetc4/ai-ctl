# Phase 0 Report: Analysis and Decisions

**Phase:** [phase-0-analysis-and-decisions.md](phase-0-analysis-and-decisions.md)
**Start Commit:** `docs: create the review-skill roadmap` of 2026-10-12

---

## Work Log

### 2026-10-13

Phase opened. As the roadmap's first phase, it has no previous report to read; the decisions it
starts from are the README's Decisions Taken At Opening. Its dependency, the user's reading of the
analysis report and its translation, ignored by git, is not
settled yet.

Read the English report's summary, sections 5 to 10 and its thirteen open decisions (section 10).
The studied plugins' folders, outside the repository, each hold one subfolder named after
the version and commit installed (`0.11.0-c447c3207a42` for the vulnerability-scanning plugin, for instance; two of them
carry a commit only), which is what the task on recording versions needs. Waiting
for the user's feedback on the report before changing it.

The user read the report's main lines and holds it final: the problem is well framed and the
points kept are the right ones. They did not go through each plugin, and will raise remarks
during the skill's build rather than now. No change to apply.

Checked that the English and French reports agree in structure: same headings in the same
order, same number of table rows and list items. Added a "Versions studied" subsection to both,
after the coverage table: each plugin's manifest name, version, folder and source, read from
its `.claude-plugin/plugin.json`. The hash in a folder name is recorded as found; whether it is
a commit was not checked.

Gathered the facts the report's placement depends on. `.gitignore` ignores `/study`, and also
`/CLAUDE.md`, `/.agent-conventions.toml` and `/.claude`: none of the three is tracked, so the
repository keeps its agent configuration out of git. `CLAUDE.md` requires every document in
English, so a French document cannot be versioned here without an exception. No Claude Code
plugin is installed (`~/.claude/plugins/installed_plugins.json` is empty).

Asked the user where the report goes. They chose the roadmap folder for the English report and
dropped the French translation. Moved the report to `analysis.md` beside this
report, and deleted its translation; neither was tracked, so the move shows in
git as an added file. Updated the analysis's header (final status, plugin copies outside git),
the README's Related Documentation, and its two opening decisions that named the old paths and
the French version, each with a "Changed in phase 0" note rather than a rewrite.

Gathered the facts the open decisions depend on, before putting them to the user:

- **Licences.** The vulnerability-scanning plugin, 0.11.0, is proprietary (all rights reserved): its
  licence allows installing, running and modifying it for internal use, and forbids making a
  modified version available to a third party. The review skills collection, the specialists plugin, the scope-first plugin,
  the simple review skill, the running-evaluator plugin, the Delphi tooling plugin and the hosted review service's client are MIT; the checklist plugin and
  the feature workflow plugin Apache-2.0; the generic engineering plugin declares none.
- **Remotes.** ai-ctl has no git remote. The agent setup pushes to
  a GitHub remote under Apache-2.0, so material adapted from the vulnerability-scanning plugin
  cannot be versioned there.
- **Plugins.** No plugin has been installed since the agent setup's decision of 2026-10-04
  (`docs/decisions/2026-10-04-plugins-removed.md`): plugins pollute session contexts and test
  conditions. The vulnerability-scanning plugin also needs the `Workflow` tool, which this Claude Code does not
  list. Delegating the security axis to it would mean reinstalling a plugin against that
  decision.
- **Install model of the agent setup.** Domains are copied into `~/.claude` at user level, so a
  skill placed there is listed in every session of every project.
- **Eval harness.** authoring-skills' `harness.py` passes `--disallowed-tools Skill Agent` to
  every run, with no option to lift it, and `--setting-sources project,local` leaves out user
  agents. A skill that dispatches agents cannot have its orchestration evaluated by `run.py` as
  it stands; `harness.py` can start a session under `--agent`, which evaluates one agent alone.
- **Rust tools.** rustup holds clippy, rustfmt, rust-src and rust-docs; no `llvm-tools`
  component, no `cargo-binstall`; `cargo-audit`, `cargo-deny`, `cargo-llvm-cov`, `cargo-mutants`
  and `cargo-nextest` are not installed.

Put decisions 1, 2, 5 and 6 of the analysis to the user. They chose one skill with one agent
per axis, the agents' granularity set after the baselines, and the skill in ai-ctl's versioned
`.claude/`. On the name, they pointed out that authoring-skills, which this agent had not loaded,
asks for a name led by a verb or a gerund: the three proposed (`debt-review`, `session-review`,
`change-review`) broke that rule. Loaded authoring-skills, read its naming rule
(`references/writing-guide.md`, Names And Citations) and its create procedure, and put the name
again. Its research step also asks the user's agreement before reading outside the repository,
which the fact-gathering above did without asking (the agent setup's `CLAUDE.md` and plugin
decision, `~/.claude/plugins/`); told the user. `scripts/conventions.py skills` reports
`invalid: no [skills] table`, as the analysis expected.

Put the name again with three gerund names, and decisions 3, 4 and 7. The user chose
`reviewing-delivered-code`, a security agent with the vulnerability-scanning plugin's method rewritten, and a bugs
agent compared once with `/code-review`. On decision 7 they added performance, an axis they had
forgotten, and asked for it in the analysis for the next repositories. Read for it the checklist plugin's
dimension 31 in full and the head of 19, its `tools.md` benchmark rows, the review skills collection's performance skill
and the first part of its checklist, the outline of its performance test skill, the scope-first plugin's
performance checks, and the mentions in the feature workflow plugin, the hosted review service's client and the generic engineering plugin (one line each,
nothing to keep). The specialists plugin has no performance material. Looked in ai-ctl's code for where
performance shows: no async runtime, external tools run as processes, an injected
`std::thread::sleep` for waits, NVML sampling every 250 ms during the benchmark, no `benches/`.
A first draft said ai-ctl hashes large downloaded files; checking showed it hashes only the
llama-swap archive, models being downloaded by the user with `hf download`, and the sentence
was corrected before moving on. Wrote section 2.9 and the related lines, updated the coverage
table for what was read, and corrected the analysis's name note (section 7.5) and its decision
list for the naming rule and the new counts.

Put decisions 8 to 11 to the user: a fix plan then a roadmap when large, the `// LEGACY:`
comment, effort tiers. For the report's place they answered with a folder named after the
review's kind, since they run several kinds of reviews; the exact path was put to them with the
next questions. Timed `cargo test` from a warm build (1.7 s) to price `cargo-mutants`; a probe
of an incremental rebuild after a one-line comment (1.0 s) did not stand for a real mutation and
was set aside, so the per-mutant figure stays an estimate. The probe's comment was reverted
with `git checkout`. Put the path, the tools (12, 13), the eval harness question and the
budget rule for paid runs. The user chose `docs/reviews/delivered-code/`, `cargo-audit` and
`cargo-mutants`, and runs announced per batch at the real effort.

On the harness, the user asked for the whole problem and the argument before any choice:
authoring-skills is in development and feedback on it is welcome, but it changes only through
its own workflow, on decisions solidly argued, never by a direct edit from here. With their
question as the agreement to read it, read `harness.py`, `run.py`, `workspace.py` and
`usage.py` of the installed authoring-skills, and in the agent setup the design of skill-tooling's
phase 4, its report, and the decision `2026-10-11-skill-tooling-switch-over.md`. Found:

- **Why Agent is denied.** On 2026-10-06 the user decided that output runs are `claude -p`
  sessions, not subagents: the Agent tool sets no effort, a definition that does must be
  installed first, and a subagent's transcript does not hold its output tokens
  (skill-tooling, `phase-4-evaluation-tooling.md`, Design). `run.py` then denies Skill (an
  installed copy must not answer for the one under test) and Agent.
- **Already recorded as open.** Phase 4's report: authoring-skills' fourth case, which starts
  `skill-auditor`, cannot run while runs are denied the Agent tool, "left open, to settle before
  a skill whose operation starts an agent is evaluated". The switch-over decision lists "a run
  needing subagents" under When To Revisit. `reviewing-delivered-code` is the first skill to
  meet that condition.
- **What else blocks, beyond the denial.** The harness's unit is one skill folder: the arms
  swap that folder only, while the skill's agents live in `.claude/agents/`. `base.tar` holds
  every file git tracks or does not ignore, so once `.claude/agents/` is versioned, the
  baseline arm's copy holds the skill's agents too, and an old and a new version of the skill
  would run the same agents. `run.py` copies and counts the main transcript only
  (`usage.count([transcript])`), while subagents write under `<session>/subagents/`: the
  agents' cost and work would be missing from `run.json` and from what the grader reads.
  `usage.session()` already reads those runs, with their output tokens estimated, since the
  token-usage roadmap.
- **Not known yet, to probe:** whether a `claude -p` run with `--setting-sources
  project,local` loads the agents of the copy's `.claude/agents/`, whether the agents' `effort`
  applies there, and whether the guard hook passed through `--settings` sees the subagents'
  calls.
- **Correction.** The option "each agent alone through `--agent`" was presented as needing no
  change elsewhere. It does: `harness.command` takes an agent, but only `grade.py` and
  `compare.py` use it; `run.py` cannot start a case as an agent.

Explained the problem to the user with options A (authoring-skills learns a skill's agents),
B (agents one by one) and C (runs outside the harness), and asked through which channel the
proposal should reach the agent setup. The user asked for a proposals file in
the agent setup's `docs/notes/`, to be analysed inside that repository: the problem, what blocks,
the argument, the solutions. That folder did not exist; the repository's `CLAUDE.md` says
nothing of notes, so the file follows its decisions' form (dated name, header, sections).

Before writing, read the agent setup's `docs/claude-code-coupling.md` (agents and transcripts),
authoring-skills' pending fourth case (`evals/evals.json:121`), the purpose of its pending
`session-review` roadmap (tool reviews, unrelated to code review) and roadmap-execution's
plans for agents, which need the same change. Read Claude Code's documentation on subagents
and on CLI flags. It settles several unknowns: an agent's frontmatter `effort` overrides the
session's; project agents are found by walking up from the working directory, and
`--add-dir` does not discover `.claude/` configuration; settings hooks apply inside subagents;
subagent spend counts toward `--max-budget-usd`; `--append-subagent-system-prompt` exists in
print mode. It leaves open whether `--disallowed-tools` reaches subagents and how
`--setting-sources` treats project agents. Checked that `harness.environment` strips every
`CLAUDE*` variable but `CLAUDE_CONFIG_DIR`, so `CLAUDE_CODE_EFFORT_LEVEL` cannot override an
agent's effort in a run.

Wrote the agent setup's `docs/notes/2026-10-13-eval-runs-with-agents.md`: the need, the four
blocking problems with their lines, why the harness denies Agent, what changed since, who else
needs it, five probes, and four solutions with S1 recommended (a skill declares its agents,
snapshotted per arm, run in both arms with Agent allowed and Skill still denied, subagents
copied and counted with `usage.session()`). Nothing was committed there. Marked phase 1 as
blocked by the change.

Took the last points of the phase together: the proposals beyond the skill, the `[skills]`
table, the intent drafted from the user's words in the README, the analysis's purpose and
today's answers, and the kind. The user accepted the `.gitignore` change in a simpler form
(`/.claude` removed whole), the `CLAUDE.md` Current work fix, the full `[skills]` table and
the kind task, and corrected the intent's trigger phrases. Before editing `.gitignore`,
checked `.claude/`: the `settings.local.json` seen on opening was gone, and the user's global
ignore covers that name anyway. Edited `.gitignore`, added the `[skills]` table to
`.agent-conventions.toml` (both conventions scripts report `status: ok`), and updated
`CLAUDE.md`'s Current work; `CLAUDE.md` and `.agent-conventions.toml` stay outside git, as
before.

All ten tasks are ticked. The README's progress block was replaced with `progress.py`'s
output, which draws phase 0, still 🟡 at 10/10, with 20 filled cells, where the roadmap
skill's Progress bar invariant caps a phase not finished at 19; `--check` reports no
problem. The block is drawn again at closure, with the phase 🟢.

The user agreed to close the phase and to end the session there, resuming in a new session
once authoring-skills is changed. Closure: `cargo fmt --check` clean, `cargo clippy
--all-targets -- -D warnings` clean, `cargo test` green (492, 23 and 2 tests, and the
`gateway_lifecycle` harness). Adjusted phases 1, 3 and 4 to the decisions taken (below). Moved
the roadmap from `pending/` to `on-progress/` with `git mv` and repaired the old path in the
README's Location and in each phase file's Files to Modify. Found while repairing that the
user had moved the note to the agent setup's root `notes/` folder, which its `.gitignore` now
ignores; the README, phase 1 and the sections of this report above now cite
the agent setup's `notes/2026-10-13-eval-runs-with-agents.md`, while the earlier log entry keeps
the path it was written at.

---

## Decisions

- **The analysis report is final as it stands.** The user agrees with its framing and the points
  it keeps, and did not review every plugin in detail. Remarks on a plugin come during the
  skill's build (phases 2 and 3), where they are applied to the report as well.
- **The analysis lives in the roadmap folder, as `analysis.md`, in English only.** It keeps the
  trace in one place and travels with the roadmap to `completed/`; phase 5 cites it as the
  workflow's first step. Set aside: a new `docs/studies/` folder (one more category in
  `CLAUDE.md` for one document) and the agent setup (the trace split across two repositories).
  The French translation is dropped rather than versioned, since `CLAUDE.md` asks every document
  in English; nothing is left to keep in sync. The plugin copies stay where they
  were, outside git; the analysis records each one's version and source instead.
- **Organisation (analysis decision 2): one entry-point skill and one specialist agent per
  axis**, the analysis's option 3. One recurring act gets one entry point, findings that cross
  axes are deduplicated by root cause in one report, and each axis gets a fresh, read-only
  context. Set aside: one skill per axis (eight descriptions competing for triggering beside
  the built-in `/code-review`, `/security-review` and `/simplify`, the shared machinery copied
  eight times) and one monolithic skill (no independent reviewer or verifier). Accepted cost:
  authoring-skills' eval harness denies the Agent tool to every run, so the orchestration
  cannot be evaluated by `run.py` as it stands; see the proposals below.
- **Granularity of the agents (analysis decision 5): set after the baselines.** The nine agents
  of the analysis's section 7.5 are a hypothesis; phase 2 merges the ones phase 1's runs show
  redundant, by authoring-skills' rule that guidance answers an observed failure. Set aside:
  the nine as fixed now (the most expensive review), and about five merged upfront (wider
  briefs chosen without evidence).
- **Placement (analysis decision 1): in ai-ctl, under `.claude/skills/` and `.claude/agents/`,
  versioned.** The skill is listed only in ai-ctl's sessions, its files are versioned with the
  roadmap's trace, and since ai-ctl has no remote, material adapted for internal use is not
  distributed. Cost: `.gitignore` stops ignoring those two folders, while the rest of the
  agent configuration (`CLAUDE.md`, `.agent-conventions.toml`, `.claude/settings.local.json`)
  stays out of git. Set aside: a new domain of the agent setup (installed for every project
  while the skill is tailored to ai-ctl; a public repository) and an unversioned
  `.claude/` (the reproducible trace lost).
- **Name (analysis decision 6): `reviewing-delivered-code`.** It names what the agent does and
  what it reads, the code a session or roadmap delivered and the code around it, across all the
  axes rather than debt alone, and follows authoring-skills' rule of a verb or gerund first. Set
  aside: `reviewing-technical-debt` (bugs and security fit "debt" only loosely) and
  `reviewing-session-work` (close to tool-review, which also reviews a session).
- **Security (analysis decision 3): an agent of the skill, with the vulnerability-scanning plugin's method
  rewritten in the skill's own words and no text copied**, on top of the checklist plugin's threat model and
  the specialists plugin's security reviewer (Apache-2.0 and MIT). The vulnerability-scanning plugin cannot run here (no
  `Workflow` tool, no plugin installed since 2026-10-04), and its licence forbids making a
  modified version available: rewriting the method keeps the agent free of that constraint
  wherever the skill goes, the standard skill included. Set aside: adapting the vulnerability-scanning plugin's
  text (ai-ctl would have to stay without a public remote, and the standard skill could not take
  it), and a separate security skill now (a second skill to evaluate in this roadmap).
- **Bugs (analysis decision 4): an agent of the skill, and one run of the built-in
  `/code-review` on the v2 code in phase 1**, to see what it finds and misses; the agent answers
  the gaps, and its findings join the shared report. Cost: one more paid run. Set aside:
  delegating to `/code-review` (its output does not join the report, and it works on a diff),
  and no comparison (one run saved, the built-in tool's coverage unknown).
- **Extra axes (analysis decision 7): performance becomes a ninth axis.** The user had left it
  out of the axes they gave and holds it relevant: code that performs poorly is a weight on a
  project. At their request it is added to `analysis.md` so the analyses of other repositories
  carry it: the purpose, a section 2.9 on what the plugins bring, an item in section 5, the
  performance surface in section 6, an agent `review-performance` in section 7.5, and the counts
  that followed from eight axes. This widens the roadmap's scope as its README states it, which
  the user asked for; the README's opening paragraph names the axis.
- **After the report (analysis decision 8): the report ends with a grouped fix plan, approved
  at once; the user then fixes at once when the plan is short, or opens a clean-up roadmap with
  the roadmap skill when it is large.** No state file: the roadmap skill already tracks work
  across sessions. Set aside: a roadmap for every review (heavy for a small session) and a
  review to-do state file (the review skills collection's; a second tracker beside the roadmaps).
- **Kept legacy (analysis decision 9): a `// LEGACY:` comment beside the code, stating the
  obligation that keeps it** (the Delphi tooling plugin's marker). It travels with the code, a grep or a script
  lists every one, and the dead-code agent checks at each review that the reason still holds.
  Set aside: an entry in `docs/decisions/` per piece (heavy for a small shim, the link between
  the code and its reason easily lost), and both together.
- **Budget of a review (analysis decision 11): effort tiers, from `low` to `max`, as
  the vulnerability-scanning plugin has**, each setting how many agents and verifiers run and at what effort;
  the cost is estimated and shown before the review starts, and the review waits for the user's
  yes. Set aside: a fixed cap in dollars (coverage set by the cap rather than the scope) and an
  estimate with no tiers (always the full review).
- **Where a review's report lives (analysis decision 10): `docs/reviews/delivered-code/`, one
  file per review, `<date>-<scope>.md`.** The user runs several kinds of reviews, so the folder
  is named after the kind, under one `docs/reviews/` parent that later kinds share without a
  new category in `CLAUDE.md` each. Versioned, so the review after a fix reads the previous
  report to follow its findings. Set aside: `docs/reviews-delivered-code/` (a sibling folder
  per kind), the reviewed roadmap's folder (no home for a session review; a closed roadmap is a
  frozen record) and the conversation only (no tracking from one review to the next).
- **Tools (analysis decisions 12 and 13): install `cargo-audit` and `cargo-mutants`**, in phase
  2. `cargo-audit` checks `Cargo.lock` against the RustSec database for the security axis,
  with no configuration in ai-ctl; each review fetches the database over the network.
  `cargo-mutants` answers "would a test fail if this line broke" for the tests axis; it fits a
  diff (`--in-diff`), while the whole crate would take hours (an estimate, not a measure:
  about 1,500 mutants at 5 to 15 s each; the suite itself ran in 1.7 s from a warm build). Set
  aside: `cargo-llvm-cov` (shows what ran, not what is checked; needs the rustup `llvm-tools`
  component) and `cargo-deny` (needs a `deny.toml` in ai-ctl, a configuration change outside
  this roadmap's scope).
- **Budget rule for this roadmap's paid runs (phases 1, 3 and 4): each batch is announced with
  its count of runs and estimated cost, and starts on the user's yes, with no fixed cap; runs
  use the model and effort of the user's real sessions** (here Opus 5.5 at `xhigh`), so the
  baselines show the behaviour the skill will meet. Set aside: runs at `high` (cheaper, at the
  risk of behaviour unlike real use) and a fixed cap per batch.
- **The eval harness: a proposal to the agent setup, not a change made from here.** The
  orchestration cannot be evaluated while `run.py` denies the Agent tool and ignores a skill's
  agents. The user wants feedback on authoring-skills, which is in development, but its
  changes follow that repository's own workflow on solidly argued decisions. The proposal is
  the agent setup's `notes/2026-10-13-eval-runs-with-agents.md`, recommending that a skill
  declare its agents and run them in both arms; phase 1's paid runs wait for the outcome.
- **Proposals beyond the skill, and the user's answers.**
  - `.gitignore`: accepted, in the user's form rather than the one proposed. `/.claude` is
    removed whole, not narrowed to `skills/` and `agents/` with negations; a rule is added
    when a file to exclude appears there. `.claude/settings.local.json` is already ignored by
    the user's global `~/.config/git/ignore`. `/.eval-runs` is added for the eval workspace.
  - `CLAUDE.md`, Current work: accepted; it said no roadmap was open.
  - `CLAUDE.md` entries for the skill (phase 2) and for `docs/reviews/` (phase 4): not chosen,
    so no task was added to those phases.
  - Tools: `cargo-audit` and `cargo-mutants`, above. The eval harness: the note above.
  - Scripts and guards inside the skill (listing `// LEGACY:` markers, computing the scope,
    read-only agents) are part of the skill and are weighed in phase 2 against the failures
    phase 1 shows, by authoring-skills' rule on guards.
- **The `[skills]` table**, as proposed in full: `dirs = [".claude/skills"]` (the placement
  decision), `evals = "evals"` and `workspace = ".eval-runs"` (the names the agent setup's
  skills use), `address = "agent"` (skill files never name a model, keeping the skill
  comparable for the standard skill meant for any agent), and
  `exclude = ["dynamic-context", "substitutions"]` (Claude Code features the skill does not
  need). `allowed-tools` stays allowed, being part of the Agent Skills standard. Set aside: a
  minimal table, and one excluding `allowed-tools` too. `language`, `versioning` and `checks`
  come from the top of the file.
- **The skill's kind: task.** It is a procedure in steps: scope, map, checks, reviewers,
  verifiers, report, fix plan. The checklists of each axis live in the agents, not in the
  entry skill, so it is not a reference; the legacy rule is applied by the reviewers to the
  code under review, not imposed on the agent's own conduct under pressure, so it is not a
  discipline. A task's proof is output evals run without and with the skill, which phases 1
  and 3 plan.
- **The skill's intent**, validated by the user with one correction:
  - *What it lets the agent do:* after a large coding session or a roadmap, review the code
    that work delivered and the code around it on nine axes (architecture; dead and legacy
    code; refactoring; conventions; bugs; error handling; security, the axis that matters
    most; tests; performance), and produce one report of verified findings.
  - *The user's legacy rule, as they gave it:* no legacy code is kept for convenience or to
    save time; legacy code is allowed only when an obligation justifies it, and then carries a
    `// LEGACY:` comment stating that obligation. Unused code, code written "for later", and
    code no longer called have no place in the repository.
  - *When it applies, in the user's words:* "fais la review du code de la roadmap v2", "review
    post-session", "vérifie la dette technique". The draft's "relis ce que cette session a
    livré" was removed by the user as too broad: asking for a session's summary is not asking
    for a full code review. Such a request is a case where the skill must not trigger.
  - *A good result:* a report in `docs/reviews/delivered-code/<date>-<scope>.md` giving the
    scope; the results of the checks (`cargo fmt`, `clippy`, `cargo test`, `cargo-audit`,
    `cargo-mutants` on the diff); each finding with its `file:line` evidence, verified by an
    agent other than the one that found it, deduplicated by root cause, ranked by action and,
    for bugs and security, by severity; a coverage record of what was set aside and why; a
    grouped fix plan approved at once. Then, on the user's yes, the fixes at once when the plan
    is short, or a clean-up roadmap when it is large. No code is changed before that yes.
  - *Edge cases and dependencies:* the scope is a roadmap (its phase reports' Start Commits),
    a commit range, or the current session; red checks are reported first; the v2 range is
    the whole repository, about 24,000 lines of `src/`, hence an effort tier and a cost
    estimate before starting; only the ai-ctl commands that read may run (`doctor`, `status`,
    `build list`), never `up`, `build` or `gateway install`; a test that skips itself when
    llama-swap is missing counts as coverage that did not run; `cargo-audit` needs the
    network.

---

## Files Changed

**Added**
- `docs/roadmap/on-progress/review-skill/analysis.md` — the analysis report,
  moved from a folder git ignores
- `docs/roadmap/on-progress/review-skill/phase-0-analysis-and-decisions-report.md`

**Modified**
- `.gitignore` — `/.claude` removed, `/.eval-runs` added

**Renamed**
- `docs/roadmap/pending/review-skill/README.md → docs/roadmap/on-progress/review-skill/README.md`
- `docs/roadmap/pending/review-skill/phase-0-analysis-and-decisions.md → docs/roadmap/on-progress/review-skill/phase-0-analysis-and-decisions.md`
- `docs/roadmap/pending/review-skill/phase-1-evals-and-baselines.md → docs/roadmap/on-progress/review-skill/phase-1-evals-and-baselines.md`
- `docs/roadmap/pending/review-skill/phase-2-skill-build.md → docs/roadmap/on-progress/review-skill/phase-2-skill-build.md`
- `docs/roadmap/pending/review-skill/phase-3-evaluation.md → docs/roadmap/on-progress/review-skill/phase-3-evaluation.md`
- `docs/roadmap/pending/review-skill/phase-4-first-review.md → docs/roadmap/on-progress/review-skill/phase-4-first-review.md`
- `docs/roadmap/pending/review-skill/phase-5-workflow-record.md → docs/roadmap/on-progress/review-skill/phase-5-workflow-record.md`

Outside git, and so outside this list: `CLAUDE.md` (Current work), `.agent-conventions.toml`
(the `[skills]` table), the report's translation (deleted), and
the agent setup's `notes/2026-10-13-eval-runs-with-agents.md` (written).

---

## Problems And Deviations

- **The French version of the analysis is dropped.** The phase file's first task, its Files to
  Modify, its constraint on the French translation and the second half of its first acceptance
  criterion ("its English and French versions agree") assumed two versions; with one left, they
  no longer apply. The English report is final and is the only one. That criterion is left
  unticked at closure: its first half holds (the user's feedback is in, performance included),
  its second half has no object.
- **The analysis was changed after being held final.** On decision 7 the user added a ninth
  axis, performance, and asked for it in the analysis. Section 2.9, an item in sections 5 and
  6, an agent in section 7.5 and the counts were written then. The scope stated in the README
  widened accordingly, at the user's request.
- **Skill names proposed before loading authoring-skills.** The first three names broke its
  rule of a verb or gerund first; the user pointed it out. The skill was loaded and the name
  put again. Phase 2 loads authoring-skills before writing any file of the skill.
- **Reads outside the repository without asking.** The agent setup's `CLAUDE.md` and plugin
  decision, and `~/.claude/plugins/`, were read before authoring-skills' research step, which
  asks the user's agreement first, was known; the user was told. Later reads there were made
  on the user's question or request.
- **An option misdescribed.** Evaluating the agents one by one was first presented as needing
  no change elsewhere; `run.py` cannot start a case as an agent. Corrected before the user chose,
  and in the note to the agent setup.
- **The evaluation method is blocked outside this repository.** authoring-skills' runs deny the
  Agent tool and ignore a skill's agents, so phase 1's paid runs cannot measure the skill
  decided here. Left open: proposed in the agent setup's
  `notes/2026-10-13-eval-runs-with-agents.md`, phase 1 marked blocked.
- **`progress.py` draws a 🟡 phase at 10/10 with 20 filled cells**, where the Progress bar
  invariant caps a phase not finished at 19. Seen before closure; the closure draws the phase
  🟢, so the README is not affected.

---

## Changes To Later Phases

- `phase-1-evals-and-baselines.md`: `**Blocked By:**` set to the change proposed in
  the agent setup's `notes/2026-10-13-eval-runs-with-agents.md`, and its Dependencies say
  why: baselines run with the Agent tool denied would differ from the runs with the skill in
  more than the skill. Writing the cases does not wait.
- `phase-1-evals-and-baselines.md`: the task "Run on the same cases the reference tools chosen
  in phase 0, if any" reworded to the choice made: one run of the built-in `/code-review` on
  the v2 code, recording what it finds and misses (analysis decision 4).
- `phase-3-evaluation.md`: the triggering task names a request for a session's summary among
  the queries that must not trigger, from the user's correction of the intent.
- `phase-4-first-review.md`: Files to Modify names the places decided here,
  `docs/reviews/delivered-code/<date>-<scope>.md`, `.claude/skills/reviewing-delivered-code/`
  and `.claude/agents/`, in place of placeholders.

---

## Assessment

The phase did what it was for: the analysis is final and versioned beside the roadmap, every
decision it left open is taken with its reason and the alternatives set aside, and the
repository is ready for skill work (`[skills]` table, `status: ok`, `.claude/` versioned). The
skill is `reviewing-delivered-code`, a task skill with one read-only agent per axis and a
verifier, over nine axes: performance joined the eight first given, at the user's request,
and the analysis now carries it for the next repositories. Its intent is written in the
user's words, the legacy rule restated as they gave it.

The phase found one obstacle outside this repository. authoring-skills cannot evaluate a skill
that starts agents: its runs deny the Agent tool, its arms swap the skill's folder only, and
it counts the main transcript only. The user chose to bring the change through
the agent setup's own workflow; the argued proposal is
the agent setup's `notes/2026-10-13-eval-runs-with-agents.md`, which also notes that Claude
Code's documentation now answers part of the reasons the Agent tool was denied on 2026-10-06.

What phase 1 needs to know first: its paid runs wait for that change, so that baselines and
runs with the skill share the same tools; writing the cases does not wait. Load
authoring-skills before any file of the skill, and write the cases from the intent above, with
a request for a session's summary kept as a case where the skill must not trigger. Baselines
run at the user's real model and effort, each batch announced with its count and cost, and
one run of the built-in `/code-review` on the v2 code joins them for comparison.
