# Phase 0 Report: Decisions And Design

**Phase:** [phase-0-decisions-and-design.md](phase-0-decisions-and-design.md)
**Start Commit:** `docs(review-skill-v2): create the roadmap` of 2026-10-31

---

## Work Log

### 2026-10-31

Phase opened. The work goes by blocks, each a proposal resting on the reviews' figures, then the
user's decision, written in the Design section and committed: the workflow and the gates; the
findings and the report; the scope and the mutants; the agents; the dependency axis; the shares.

Block 1, the workflow and the gates. The sizes a review at a phase's close would meet, measured
on `audio-image-backends`' phases: from +29 to +2,477 lines in `src/`, +4,261 −271 over the
roadmap, against 11,573 production and 17,001 test lines in its scope by file. The user accepted
the four proposals; the Design section holds them.

Block 2, the findings and the report. The 2026-10-17 review had already recognized two kept
findings of the first by their symbols, lines moved; all 84 of its findings were fixed, so the
fingerprint's proof took two fixed findings, 3 and 84, whose every symbol is still at `HEAD`. One
finding of each report rewritten in the new format left no field empty; the timing finding 84
needs `test: none:` and a reason, since `CLAUDE.md` forbids a test that depends on timing. The
kept work of that review's `review-tests` wrote its mutants as exact replacements, not patches,
which the recorded mutants now follow. The user accepted the five proposals: "ok on verra a
l'utilisation si les décisions sont solides ou demande à être amélioré".

Block 3, the scope and the mutants. A script in the session's scratchpad counted, rather than a
count by hand, the functions holding a change and their callers by name: a third of the scope
by file for `v2-review-fixes`, a sixth for `audio-image-backends`, the callers by name an upper
bound. `cargo mutants --list --in-diff` refused the working tree, its diff no longer matching
`doctor.rs`, and listed 198 mutants in a `git archive` copy of `docs(audio-image-backends): close the roadmap without phase 4` of 2026-10-30. The user accepted both
proposals, and widened the context: new code is analysed in its context, even when that analyses
the same code again in later reviews.

Block 4, the agents, first half. The sub-agents page gave three mechanisms: `skills:`, which
injects a skill's whole content into an agent; hooks in an agent's frontmatter; and
`isolation: worktree`, branched from the default branch. The user accepted the proposals and the
probe. Three `claude -p` probes followed: the injection held; the frontmatter hooks ran in
neither a scratch folder nor this repository; the project settings' hooks did, naming the agent
in `agent_type`. The guards move to `.claude/settings.json`.

The check of the closed rule, one of the contract's checks, failed on six lines of this roadmap's
files naming one of the user's repositories by its name, which it passed at the creation while
the files were still untracked; they now say "the agent setup", as the earlier roadmaps do.

The security skill's name, in prose, escaped that check: the consumer's glossary of neutral
terms showed it, and the files now say "the security skill" and "outside the repository". The
branch's commits before `docs(review-skill-v2): use the neutral term for the security skill`, and the creation commit on `main`, keep both names in their
content; nothing is pushed.

Block 4b, `review-security`'s method. The security skill's lessons, read outside the repository
with the user's leave, hold ten points; six are already decided in blocks 2 to 4 (the rejected
findings kept, the fingerprint, the structured answer and the generated report, the agents
counted, the floor as its hardening notes, `PROBES` as its `needs_validation`). Proposal 10, put
to the user, its four remaining points, the threat context taken from `docs/security.md` §1 and
§2 (one login account):

- **A gate on every candidate:** a security finding names who attacks and their starting
  capability, a row of §1; the input accepted; the control that should stop it; the boundary
  crossed; the resource reached; the result observed. With no boundary crossed it is no security
  finding, "Self-harm within the same account is not a boundary violation", and may stay a
  finding of another axis.
- **Severity anchors of ai-ctl's own:** critical, the network gets the key or runs code with no
  action of the user; high, a web page through DNS rebinding, a container, or content ai-ctl
  fetches (a release, a source, an image, a model) gets the key, runs code or reads the prompts;
  medium, another local account gets the key or the prompts; low, a control weakened with no
  damage shown, or a defect that fails closed. Between two, the lessons' rule: does the result
  fully defeat an explicit control for an action with real consequences, or only weaken it, and
  with no concrete damage stated, the lower one. By these anchors, the 2026-10-17 review's
  finding 1, a backend's port taken by another local account, would be medium, not high.
- **The hunting of sibling paths:** each input followed through the legacy, batch, retry,
  cancellation and error paths to the same effect; sibling controls compared for equivalence,
  not only presence; a root cause's variants searched in the scope; a line of search stopped once
  its invariant is settled. S1 and S4 of the 2026-10-19 pass are such misses.
- **ai-ctl's attack classes,** written in the agent: injection through the CLI (commands, paths,
  environment); files (symlinks, TOCTOU, archive extraction, temporary files); secrets, the key in
  logs, URLs and errors; logic on partial failure, defaults and fallbacks; loopback listeners
  with no peer authentication, and the lifecycle of ports, locks and restarts; the supply chain
  (downloads, digests, a mutable tag); containers (podman, devices, CDI, host paths). Left out:
  the web and multi-tenant classes, and memory safety, `src/` having no `unsafe`.
- **Left out of the lessons:** their weight, their sandbox for hostile target code, their cost.
- **The bar for `CONFIRMED`,** which the lessons left to decide: a security finding medium or
  worse confirmed only on an observed result, a run; shown by reading alone, it stays `UNCERTAIN`
  with its `PROBE`; a low one may be confirmed by reading.

The user, at the session's close: "on valide dans la prochaine session". Task 9 stays unticked
until then.

### 2026-11-01

Resumed with the phase file uncommitted: the user's editor had removed the line and paragraph
separators that the Write tool had put in place of their escapes on 2026-10-31, in the Shares
task and in phase 3's Constraints; both lines rewritten through Python (Deviations). The user
validated proposal 10 as a whole, "on peut revalider l'ensemble des choix": task 9 ticked.

Block 5, the dependency axis. Each criterion's source queried once (Design, The Dependency Axis):
nothing to fix today, `nvml-wrapper` the one crate outside the large projects, two duplicates,
llama-swap six releases and llama.cpp 332 builds behind. The user accepted the scope, the script
and its agent, and the routing, then asked for the session's proposal on the tools: `cargo-deny`
in place of `cargo audit`, accepted.

Block 6, the shares. Each point of the assessment checked in the installed tools: two fixed or
answered since, the progress bar and the report's rules; four sent as notes; the agent setup's
open roadmap `fixes-and-overview` designs a documentation step that the closing review can share.
The user accepted the three proposals; the four notes written, phases 1, 2, 4 and 6 adjusted.
Every task of the phase is ticked.

---

## Decisions

- **The workflow in four moments and its stopping rule** (Design, The Workflow And Its Stopping
  Rule): at each fix batch, at a phase's close, at a roadmap's close, and a full review only on
  request; no pass after the roadmap's closing pass. The user: "Oui très bien, on étale les
  reviews, on rend le tout plus dynamique". The dependency axis runs at a roadmap's close. Commits
  phase 1's cases and phase 3's `SKILL.md` to it.
- **The trigger of the review at a phase's close:** `CLAUDE.md` from phase 3 on, then
  `.agent-conventions.toml` once the roadmap skill reads a closing step there. The user:
  "CLAUDE.md ou .agent-conventions.toml". `CLAUDE.md` first because it is tracked and read at each
  session start, while this repository's `.agent-conventions.toml` is ignored by git and no tool
  reads such a key yet. Commits phase 3 to a task, added, and this phase's Shares to a note for
  the roadmap skill.
- **A request that accepts the cost beforehand is the go-ahead.** The user: "oui je suis d'accord
  aussi". `usage.py estimate --start` now guards the window, refusing a wave past 90%, which the
  skill's current rule, written before it, could not know. Commits phase 1 to rewrite the case
  `cost-gate`, which asserts the opposite.
- **One go-ahead per review, the session's triage and its hard cases** (Design, The Gates). The
  user: "Ok, si besoin de compléter on sera à temps de le faire petit à petit": the list of hard
  cases grows with use.
- **The structured finding** (Design, The Structured Finding): `where` with its symbol, an
  `effect` of six values, a `test` field; a test gap takes the effect of the defect it would let
  through; the origin computed by the report writer. Commits phase 2's report writer and phase
  3's agents to these fields.
- **Below the floor, the scope's new code with a local fix is fixed at once**, in one commit with
  the others of its kind; the rest goes to the register. It refines block 1's row for a phase's
  close: the chain of a finding, not its fix, was the cost of a stale comment.
- **The fingerprint and the register** (Design, The Debt Register And The Fingerprint): the
  axis, the paths and the symbols; a match is a candidate the session compares; one
  `register.json` for the repository. Commits phase 2's register.
- **A folder per review, `findings.json` its source and `report.md` written from it, by
  `findings.py`** (Design, The Report Writer). Commits phase 2's report writer and phase 3's
  `assets/report.md`.
- **`repro/` per review, the verifier's red test and settled fix, the recorded mutants in
  `mutants.json`, `repro/` removed in the fixes' last commit** (Design, Reproductions, The
  Verifier, The Regression Run). Commits phase 2's regression run and phase 3's
  `review-verifier`.

  The user, on block 2's five: "ok on verra a l'utilisation si les décisions sont solides ou
  demande à être amélioré": the decisions are tried by the first real use, phase 5.
- **The scope by hunk, new code analysed in its context** (Design, The Scope By Hunk): the
  holding item whole, its callers and callees one level away and the sibling paths, analysed and
  not only read, even when a later review analyses them again. The user: "le nouveau code doit
  être analysé dans son context (code environent même si celà oblige a réanaliser plusieur fois
  le même code lors des diffèrentes review, je pense que c'est la solution la plus sur)". The
  agents find the callers. Commits phase 2's `scope.py` and phase 3's reviewers to it.
- **The mutants in the background from the review's start, in a copy, with no limit**, and the
  kinds a fix applies by hand (Design, The Mutants). The user: "validé". Commits phase 3's steps
  and fix method to it.
- **The agents' shared rules injected by `skills:`, mechanical guards by hooks, no worktree,
  `PROBES`, the owner of unreachable branches, and the directions read from the roadmaps**
  (Design, The Agents). The user: "validé", and allowed the probe and the reading of the study.
  Commits phase 2 to the guards' script and its tests, phase 3 to the rules file and the agents,
  phase 4 to the rules skill's trigger rate.
- **`review-security`'s method** (Design, The Method Of `review-security`): the gate on every
  candidate, ai-ctl's severity anchors with another local account at medium, the hunting of
  sibling paths, ai-ctl's attack classes, and a run to confirm a security finding medium or
  worse. Proposed on 2026-10-31; the user, on 2026-11-01: "on peut revalider l'ensemble des
  choix". Commits phase 3's `review-security` and `review-verifier` to it.
- **The dependency axis** (Design, The Dependency Axis): the crates on every criterion, the
  backend sources and llama-swap for their lag alone, the images left out; `cargo-deny` in place
  of `cargo audit` for the mechanical part, under a tracked `deny.toml`; a script for the
  updates, the maintenance and the reliability, with no personal data in its `User-Agent`;
  `review-dependencies` judging, at a roadmap's close, at a phase's close when the Cargo files
  change, and alone on request; the routing of its findings. The user, 2026-11-01: "I OK", "K
  OK", "L ok", and on `cargo-deny`, after asking for the session's proposal: "Ok validé on suit
  ta recomendation". Commits phase 2 to `cargo-deny`'s install, `deny.toml` and the script,
  phase 3 to the agent, phase 6 to `cargo deny check` as its proof.
- **The shares** (Design, The Shares): four design-feedback notes for the agent setup — the raw
  separators, a closing step a repository declares, the frontmatter hooks in eval runs, a tick
  command —, none for the report's size or the progress bar; a check against raw separators in
  phase 2; `cargo deny check` in the CI/CD note; the split between ai-ctl and every repository,
  written as `shared-and-specific.md` in phase 6. The user, 2026-11-01: "ok validé". Commits phase
  2 to the check and phase 6 to the file.

---

## Deviations

- `phase-3-skill-and-agents.md`: a task added under The Repository, `CLAUDE.md`'s convention for
  the review at a phase's and a roadmap's close, which the creation left out; it lands with the
  v2, since the skill as it stands would review the whole crate at each phase's close.
- The recorded mutants are kept as exact replacements in `mutants.json`, where the proposal the
  user accepted said patches in a `mutants/` folder: the 2026-10-17 review's kept tooling wrote
  its 25 mutants that way, and a replacement survives the lines moving, where a patch's context
  does not. Put to the user in the session's message after the commit.
- The guards the user accepted as hooks in the agents' frontmatter are hooks of the project's
  `.claude/settings.json` instead: the probe showed frontmatter hooks skipped in this
  repository's `claude -p` sessions, which eval runs are, and the settings' hooks running with the
  agent named in their input.
- `phase-2-scripts.md`: a task added under Guards, the hook script of the review agents' guards
  and its wiring in `.claude/settings.json`, by block 4's decision.
- The phase file's Shares task and phase 3's Constraints held the line and paragraph separators
  themselves where they meant their escapes: the Write tool of 2026-10-31 decoded `\u2028` and
  `\u2029`, the defect phase 3's constraint names, and the user's editor then removed them from
  the phase file, uncommitted. Both rewritten through Python on 2026-11-01, no raw separator left
  in the roadmap's files.
- `phase-2-scripts.md`: the dependency facts' task rewritten for what `cargo-deny` does not check,
  and a task added for `cargo-deny`'s install, its toolchain entry and `deny.toml`;
  `phase-5-first-real-use.md` and `phase-6-fixes-and-closing-pass.md`: `cargo-deny` in place of
  `cargo audit`; the README's Dependencies likewise. By block 5's decision.
- The session sent the user's email address in the `User-Agent` of about thirty requests to
  crates.io during block 5's queries, a contact crates.io's policy asks for but the user had not
  given for it; told to the user at once, the later requests sent a neutral `User-Agent`, and the
  script's carries no personal data.
- By block 6's adjustment of the later phases: `phase-1-evals-and-baseline.md`, the case
  `cost-gate` named among the cases to rewrite; `phase-2-scripts.md`, the scope's task without the
  callers, the regression run on `mutants.json`'s replacements, and a task added under Checks
  for the raw separators; `phase-4-evaluation.md`, the shared rules skill's trigger rate;
  `phase-6-fixes-and-closing-pass.md`, the record named `shared-and-specific.md`.

---

## Assessment

Phase 0 settled, with the user, every point the 2026-10-19 assessment left open, and the
dependency axis the user asked for: twelve decisions in six blocks, each a proposal resting on
the three reviews' figures or a probe, each written in the phase file's Design section before
its commit. The v2 has its shape: a review at each phase's close on the phase's hunks in their
context, a security pass, a dependency check and a regression run at a roadmap's close, then no
pass; one go-ahead per review and the session's triage; findings with an effect, an origin, a
floor and a fingerprint, kept in a register; a report generated from `findings.json`; agents
sharing one rules skill, guarded by the settings' hooks; `review-security` with a gate and
ai-ctl's own anchors; `cargo-deny` and a script for the crates.

What the probes found that the plan did not foresee: an agent's frontmatter hooks do not run in
`claude -p` sessions, where the project settings' hooks do, naming the agent; `isolation:
worktree` branches from the default branch; `cargo mutants --in-diff` refuses a tree that differs
from its diff; the shared rules skill shows in the main session's list. The closed rule's check
catches a registered name, not a studied source named in prose, which the glossary caught. The
Write tool's decoding of U+2028 and U+2029 struck this phase's own files. The phase grew the
roadmap from 54 to 57 tasks; no task moved, no restructuring waits.

Phase 1 needs first: the baseline is the skill as committed today, unchanged by this phase,
passed to `scripts/workspace.py --baseline`; the evals' cases come before any other file of the
skill; the case `cost-gate` asserts the opposite of the new gate; `evals/fixtures/claude.md`
predates the 2026-10-19 reorganization; and the runs are paid, each batch after the user's yes
on its count and an order of magnitude in one line.
