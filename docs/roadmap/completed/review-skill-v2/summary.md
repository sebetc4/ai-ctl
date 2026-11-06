# Summary — Review Skill V2

---

## Where We Started

`reviewing-delivered-code` had reviewed the v2 code three times, and its cycle of reviews and
fixes did not end on its own. It scoped a review by file, so `v2-review-fixes`' diff handed its
reviewers 15,218 production lines; its findings had no floor and no effect, so a second full
review raised 84 and asked for a third; its gates asked a yes per wave; its report, 214 KB, was
written by hand; its reproductions were deleted at each pass's end. The workflow that broke the
loop, decided at `v2-review-fixes`' close, lived in none of its steps: mutants on each fix
batch's diff, a review of a phase's hunks, a security pass alone at a roadmap's close, the
session giving the clear verdicts, and no pass after that pass's fixes. The user had asked for an
axis none of the twelve covered, the dependencies. `audio-image-backends` had closed without a
review, on purpose, to be the v2's first real use.

---

## Where We Landed

The v2 runs a review by moment: a phase's close on the phase's hunks, a roadmap's closing pass
with `review-security` alone, the dependency axis and the regression run, a session or a range, a
security pass, the dependencies alone, a full review only on request. One go-ahead covers a
review, its tier taken from the surface `docs/security.md` guards before the scope's size, its
estimate shown before any agent starts. Five scripts carry what the session wrote by hand:
`scope.py` gives the scope by hunk with the items holding it, `findings.py` keeps
`findings.json`, writes the report and the debt register, `regression.py` runs the recorded
mutants, `dependencies.py` gathers the crates' facts, and `guard.py` holds the agents to what
they may run and checks the tree at each stop. Six reviewers, a verifier per reviewer and
`review-dependencies` share one rules skill, `reviewing-as-an-agent`, with ai-ctl's severity
anchors and one answer format. A finding has an effect, an origin, a floor and a fingerprint;
below the floor it is fixed at once or registered, above it a verifier leaves its red test and
its settled fix in `repro/`.

The cycle ended. The first real use reviewed `audio-image-backends` at tier `high`: 56 findings,
46 fixed in sixteen batches by `references/fixing.md`, nine registered and one refuted. The
closing pass raised three, all fixed, and no pass followed; the 58 mutants recorded by hand are
each caught at the last fix. ai-ctl gained, among the fixes, a `llama-server` built without its
web UI, a current build compared with its recipe's patches and CMake options by `doctor`, `up`
and `gateway config`, and an inventory refused past 20,000 sets of models that run together;
`CHANGELOG.md`'s 0.2.0 entry lists them, unreleased.

---

## What Each Phase Delivered

**Phase 0, Decisions And Design.** Twelve decisions with the user, each resting on the three
reviews' figures or a probe: the moments and their stopping rule, one go-ahead, the floor, the
register, the report written from `findings.json`, the agents' shared rules injected by
`skills:` and guarded by the settings' hooks, `review-security` with a gate and ai-ctl's own
anchors, `cargo-deny` and a script for the crates. Its probes found that an agent's frontmatter
hooks do not run in `claude -p` sessions and that `cargo mutants --in-diff` refuses a tree that
differs from its diff.

**Phase 1, Evals And Baseline.** Five cases on those decisions, their fixtures built offline by
`fixtures.py`, 36 trigger queries, and a baseline on the old skill: 23% of the assertions, every
run stopping at its go-ahead and scoping by file.

**Phase 2, Scripts.** The five scripts, test-first with their mutants applied by hand,
`cargo-deny` under a `deny.toml`, and a check against raw line separators in the tracked files.
Three of phase 0's rules changed on evidence: a finding's origin from its first place, a match by
axis, path and symbol, and an allowlist of ai-ctl's commands for the guard.

**Phase 3, Skill And Agents.** `SKILL.md` by moment on those scripts, `references/fixing.md`, the
rules skill, the seven agents rewritten and `review-dependencies` added; `CLAUDE.md` asks the
reviews at a phase's and a roadmap's close.

**Phase 4, Evaluation.** The v2 passed 84%, 95% and 95% of its assertions over three iterations,
against 41%, 41% and 26% for the baseline; its description triggers on every review request.
`phase-close` never ran on the v2: with $58 of the roadmap's $188 spent in that phase, the user
chose to prove the fix method in real use.

**Phase 5, First Real Use.** The review of `audio-image-backends` at `high`, over three sessions
and two accounts. Every agent's answer passed `findings.py add` at its first submission and the
guard blocked no stop; ten failures of the skill, nearly all on the session's side, were recorded
and fixed.

**Phase 6, Fixes And Closing Pass.** The sixteen batches, the closing pass and its two batches,
three more failures of the fix method fixed at their lines, and `shared-and-specific.md`, the
line between what every repository shares and what is ai-ctl's.

---

## What We Learned

A review's cycle ends when its last pass is narrow and its fixes carry their own proof: one
security pass on the roadmap's diff, three findings, closed what the old full reviews kept open,
the fixes resting on red tests, mutants and a record that runs again. The evals proved the
workflow's shape at a cost; the fix method's failures showed only in real use, on real code: a
test the mutation tool's copy cannot run, a fix that reaches only what is made after it, and a
record of mutants that decays between batches unless each batch runs it whole. The agents held;
the session's side failed most, in its scope, its writes while agents ran, its records and its
reading of a step. The pace is set by the usage windows: a `high` review took three sessions and
a forced batch, each step held for the user's word.

---

## What We Are Leaving Open

- CI/CD at each tag, then the release of 0.2.0, which waits for it.
- The lag the dependency axis reported without a finding: llama-swap pinned at v263 with v270
  out, llama.cpp, stable-diffusion.cpp and audio.cpp behind their newest releases, whose release
  notes no agent may read.
- The debt register's eleven entries, each with its reason.
- `phase-close` on the v2, never run as an eval; the first real phase close that changes code
  is its proof.
- The description's trigger on a request for the built-in security review of pending changes,
  which no request in use has tested.
- The review skills of other repositories, and the skill that builds them, outside this
  repository: `shared-and-specific.md` is their input.
