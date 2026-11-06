# Phase 1: The Reset And The Remote

---

## Status

**Current Status:** 🟢 Done (100% — 5/5)
**Started:** 2026-11-06
**Completed:** 2026-11-06
**Blocked By:** —

---

## Objective

Start the history again from one root commit holding the tree as it is, the old history archived
and verified outside the repository, and push it to a private remote.

### Out of Scope
The CI, phase 2's. Making the remote public.

---

## Tasks

### Before
- [x] No other roadmap open, phase 0 closed, the checks green; the old history written to a `git bundle --all` beside v1's, then verified: `git bundle verify`, and a clone of it holding as many commits and the same refs as the repository, its `main` at the repository's
  Proof: high, check — the counts and the refs equal, recorded in the report

### Reset
- [x] The citations of the old history this roadmap's records gained since phase 0 — its reports' `**Start Commit:**` lines, the README's `**Branch:**` line, phase 0's review — rewritten as phase 0 did, on the tree the root commit takes
  Proof: medium, check — phase 0's command finding none on that tree
- [x] The root commit on an orphan branch, holding that tree; `main` replaced by it and the roadmap's branch made again from it; every other ref, the reflog and the old objects removed from the clone once the bundle is verified; through `committing-changes` where its scripts allow, each git command run outside them named in the report
  Proof: high, check — one commit reachable from every ref, `git cat-file --batch-all-objects` listing no commit but the root, the checks green on the root commit
- [x] The remote: a private GitHub repository `ai-ctl`, created empty by the user, set as `origin` over SSH; `push = "auto"` in the `[git]` table; `main` and the roadmap's branch pushed through `committing-changes`, the pre-push hook's check passing
  Proof: high, check — `git ls-remote origin` giving each branch at the new history alone; the hook's output in the report

### Closing
- [x] The documents this phase's changes describe at their current state — `CLAUDE.md`'s archive of v1 naming both bundles and the day the history starts —, and a design-feedback note to the agent setup on what this reset met, its procedure step by step, for the setup's own reset
  Proof: medium, review — each document named in the report; the note read by the user

---

## Technical Details

### Files to Modify
```
CLAUDE.md
.agent-conventions.toml    (untracked)
roadmap fresh-history-and-ci: its README and its reports
```

### Dependencies
Phase 0 closed, its review's findings fixed. The remote created by the user.

### Constraints
The reset is irreversible once the clone's old objects are gone: each step's check passes before
the next one starts, and nothing is removed before the bundle has been cloned and counted. The
working tree's untracked and ignored files, `notes/` among them, stay as they are. The first push
publishes the root commit's whole tree: the closed rule's check runs on it before.
