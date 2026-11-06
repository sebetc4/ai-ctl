# Roadmap: Review Skill V2

Rebuilds the `reviewing-delivered-code` skill on scripts, a debt register, guarded agents and a
dependency axis, then reviews and fixes `audio-image-backends`' code as its first real use.

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
Phase 0  Decisions And Design       🟢 ████████████████████ 100%  (12/12)
Phase 1  Evals And Baseline         🟢 ████████████████████ 100%  (6/6)
Phase 2  Scripts                    🟢 ████████████████████ 100%  (11/11)
Phase 3  Skill And Agents           🟢 ████████████████████ 100%  (12/12)
Phase 4  Evaluation                 🟢 ████████████████████ 100%  (4/4)
Phase 5  First Real Use             🟢 ████████████████████ 100%  (7/7)
Phase 6  Fixes And Closing Pass     🟢 ████████████████████ 100%  (7/7)
TOTAL                                  ████████████████████ 100%  (59/59)
```

**Current Phase:** —
**Blocked By:** —

---

## Why This Roadmap Exists

`reviewing-delivered-code` reviewed the v2 code three times, and its cycle of reviews and fixes
did not end on its own. It scopes a review by file, so the review of `v2-review-fixes`' diff
handed its reviewers 15,218 production lines, the whole crate less six files: a second full
review, which raised 84 findings and asked for a third round. Its findings had no floor and no
effect, so a stale comment went through the chain of a lost build; its gates asked a yes per
wave; its report, 214 KB, was written by hand and slipped three times; its reproductions were
deleted at each pass's end; and it ends by opening a clean-up roadmap, which calls for the next
review.

The workflow that broke the loop was decided during `v2-review-fixes`' phase 5 and is in none of
the skill's steps: mutants on each fix batch's diff, a review of a phase's hunks at its close, a
security pass alone at a roadmap's close, the session giving the clear verdicts, and no pass after
that pass's fixes. The session's assessment at that roadmap's close, on 2026-10-19, lists what to
keep, change, remove and add; the skill also cites a roadmap nine times, which a skill never does,
and two of its agents point at folders the documentation's reorganization of 2026-10-19 removed.

The user asked on 2026-10-30, and again at this roadmap's creation, for an axis none of the twelve
covers: the dependencies, here the crates — their updates, their reliability, their maintenance.

`audio-image-backends` closed on 2026-10-30 without a closing review, on purpose: its code is the
v2's first real review.

---

## Decisions Taken At Opening

- **Name:** `review-skill-v2`. The skill keeps its name, `reviewing-delivered-code`: a rename
  breaks its invocations and every permission rule that names it.
- **Purpose:** a review-and-fix cycle that ends. The workflow above and its stopping rule written
  in the skill; the tools the user agreed to on 2026-10-17 (a report writer, a `repro/` folder per
  review, mutants in the background, a `PROBES` channel, a regression run); a thirteenth axis, the
  dependencies; proven by a first real use on `audio-image-backends`' code and its fixes.
- **The dependency axis** (the user, 2026-10-30 and 2026-10-31): each crate of `Cargo.lock`,
  direct and transitive, checked for its updates, its reliability and its maintenance — its
  advisories, unmaintained or yanked releases, the versions newer than the lock within and beyond
  `Cargo.toml`'s requirements, its release rhythm, its maintainers, its repository's activity.
  Phase 0 settles its criteria, its tools, its network access and whether it also covers the
  backend sources, llama-swap and the container images `docs/security.md` pins.
- **The fixes of the first real use are in this roadmap**, phase 6, with the closing security
  pass and the regression run: the stopping rule is proven only by a cycle that ends (the user's
  agreement to the proposal, 2026-10-31).
- **Method:** `authoring-skills`' procedure for an edit. The previous version, the commit before
  the change, is the baseline; the evals' cases come before the skill's other files and run on
  the baseline first; the skill holds only what answers an observed failure or a rule the user
  stated; scripts are written test-first; the static audit runs after every write. Paid runs start
  only after the user's yes, given on their count and an order of magnitude in one line.
- **A skill cites no roadmap, phase or report**; a repository path only where a step reads, runs
  or writes it. A user's rule is stated as the rule, a measure with what was measured and its
  date.
- **A defect of a tool of the agent setup** goes there as an argued design-feedback note, never as
  a direct edit; a failed eval is classed before any change, and no skill or assertion is bent
  around a gap of the validation.
- **Tests and commits follow `CLAUDE.md`:** test-first; each commit's code diff through
  `cargo mutants --in-diff`, a missed mutant killed or justified in the report; a new test that
  spawns a process, binds a port or takes a lock run 30 times before its commit; one Conventional
  Commit per unit of work, with the checks green.
- **Branch:** `roadmap/review-skill-v2`, made at the first opening.

---

## Deliberately Out Of Scope

- Any direct change to the agent setup's tools: `roadmap`, `authoring-skills`, `usage.py`.
- The cost method itself, which `usage.py estimate` implements.
- The skill that builds review skills, and the review skills of other repositories: this roadmap
  records only what is ai-ctl's and what every repository shares.
- A full review of the crate: the first real use reviews `audio-image-backends`' diff.
- CI/CD at each tag and the release of 0.2.0, which come after this roadmap.
- The open items of the earlier roadmaps' summaries, other than the review skill's.

---

## Phases

- [Phase 0: Decisions And Design](phase-0-decisions-and-design.md)
- [Phase 1: Evals And Baseline](phase-1-evals-and-baseline.md)
- [Phase 2: Scripts](phase-2-scripts.md)
- [Phase 3: Skill And Agents](phase-3-skill-and-agents.md)
- [Phase 4: Evaluation](phase-4-evaluation.md)
- [Phase 5: First Real Use](phase-5-first-real-use.md)
- [Phase 6: Fixes And Closing Pass](phase-6-fixes-and-closing-pass.md)

---

## Dependencies

- Each phase builds on the previous one; phase 0's closure adjusts phases 1 to 6 to its
  decisions.
- Phases 1, 4 and 5 start paid sessions, each batch after the user's yes; `usage.py estimate`, of
  skill-tooling, installed, gives the order of magnitude.
- `cargo-mutants`, installed; `cargo-deny`, chosen in phase 0 in place of `cargo audit`, installed
  in phase 2 with an entry in `docs/toolchain.md`'s record.
- The dependency axis reads crates.io over the network; no test depends on it, its parsers being
  tested on captured outputs.

---

## Related Documentation

- [reviewing-delivered-code](../../../../.claude/skills/reviewing-delivered-code/SKILL.md) — the skill as it stands, with its seven agents in `.claude/agents/review-*.md`
- [docs/reviews/delivered-code/](../../../reviews/delivered-code/) — the three reviews' reports: the v2 rebuild, the closing review, the security pass
- roadmap `v2-review-fixes`: `summary.md`, and `phase-5-closing-review-report.md`, § Problems And Deviations — the skill's defects found in use
- roadmap `review-skill`: `summary.md` and `workflow.md` — how the skill was built, the input for the other repositories' review skills
- roadmap `audio-image-backends`: `summary.md` — the code of the first real use
- [docs/security.md](../../../security.md) — the threat context and the pinning of the backend sources, llama-swap and the images
- [tests/fixtures/README.md](../../../../tests/fixtures/README.md) — the captures every parser is tested on

---

## Metadata

**Roadmap Status:** 🟢 Done
**Created:** 2026-10-31
**Branch:** roadmap/review-skill-v2, from main at `docs(review-skill-v2): create the roadmap` of 2026-10-31
