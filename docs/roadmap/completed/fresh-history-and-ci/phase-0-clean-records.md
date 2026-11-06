# Phase 0: Clean Records

---

## Status

**Current Status:** 🟢 Done (100% — 6/6)
**Started:** 2026-11-06
**Completed:** 2026-11-06
**Blocked By:** —

---

## Objective

Make the review skill's evals work without ai-ctl's history, rewrite each citation of that history
in the tracked files, and tell the agent setup what this reset changes for its tools, so that the
reset of phase 1 removes nothing the repository needs.

### Out of Scope
The reset, phase 1's. The citations this roadmap's own records gain until the reset, which phase 1
rewrites.

---

## Tasks

### The Agent Setup
- [x] An argued design-feedback note to the agent setup: `scope.py` reads the report template's `**Start Commit:**` line, which the setup's own reset plans to remove; its planned rule that a tracked file cites no commit meets the review's records, `base`, `end` and `fixed_in`, which stay valid in a history never rewritten
  Proof: low, review — the note read by the user

### Fixtures
- [x] The evals' fixtures built without ai-ctl's history: the base tree of 2026-10-30 tracked among the fixtures, less what `LEFT_OUT` leaves out, and its author a constant; `fixtures.py` runs neither `git archive` nor `git log` in the repository
  Proof: high, test — a fixture built in a repository whose history lacks the base commit, watched failing on its assertion then passing; each fixture's commits of the same hashes as before the change; the tracked tree passing `sid.py check`

### Records
- [x] Each citation of the old history rewritten in place — the 551 lines citing a commit in 44 files at the roadmap's creation, the closed roadmaps' `**Start Commit:**` and `**Branch:**` lines among them, the reviews' `base`, `end` and `fixed_in` with the reports `findings.py` writes from them, the comment of `src/domain/prompt.rs` on the bench's corpus, and the 9 lines citing a session's id or its transcript or scratch path —, each replaced by the work's date and name, or dropped
  Proof: medium, check — a command resolving each hexadecimal word of the tracked files against the history and searching the session ids and paths finds none outside the fixtures; `findings.py report` clean on each rewritten review; the diff read

### Rule
- [x] The rule, among `CLAUDE.md`'s conventions, that a tracked file cites no session's id and no path into a session's transcript or scratch folder, a fact's source named by its date and its work; a captured payload among the fixtures is no citation
  Proof: low, review — the diff read, the records task's command finding none

### Closing
- [x] The documents this phase's changes describe at their current state — the review skill's evals on their fixtures, `CLAUDE.md` if a convention changes —, each named in the report
  Proof: medium, review — each document changed, named in the report
- [x] The review of the phase's hunks by `reviewing-delivered-code`, before the reset removes the commits it reads, its findings fixed in the phase
  Proof: medium, review — the review's report, each finding fixed or registered

---

## Technical Details

### Files to Modify
```
.claude/skills/reviewing-delivered-code/evals/fixtures.py
.claude/skills/reviewing-delivered-code/evals/fixtures/
.claude/skills/reviewing-delivered-code/evals/test_fixtures.py
.claude/skills/reviewing-delivered-code/evals/evals.json
src/domain/prompt.rs
docs/roadmap/completed/
docs/reviews/delivered-code/
```

### Dependencies
No other roadmap open.

### Constraints
Code and scripts test-first; `cargo mutants --in-diff` on each commit's code diff. The base tree is
a fixture frozen as it was: the rewrite leaves its content as it is, and `sid.py check` passes on it
before it is tracked. A closed record keeps its meaning: a citation is replaced by what lets a reader
find the work without the hash — the date, the roadmap, the phase, the commit's subject —, never
by a paraphrase of what the commit did. The agent setup's files are never edited from here.
