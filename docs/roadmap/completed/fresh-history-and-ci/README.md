# Roadmap: fresh-history-and-ci

Starts ai-ctl's history again from one root commit on a private remote, its records citing nothing
of the old history, with a CI that checks and builds each pushed tag on Fedora 44.

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
Phase 0  Clean Records              🟢 ████████████████████ 100%  (6/6)
Phase 1  The Reset And The Remote   🟢 ████████████████████ 100%  (5/5)
Phase 2  CI At Each Tag             🟢 ████████████████████ 100%  (9/9)
TOTAL                                  ████████████████████ 100%  (20/20)
```

**Current Phase:** —
**Blocked By:** —

---

## Why This Roadmap Exists

0.2.0 waits on a CI that builds each tag, and a CI needs a remote, which ai-ctl has never had:
`git remote -v` is empty and the `[git]` table pushes nothing. Pushing the history as it stands
would publish, at this roadmap's creation on 2026-11-06, 480 commits since 2026-10-09, 268 of them
with an agent's `Co-Authored-By` trailer, the last on 2026-10-23, and the commits of 2026-10-31
that opened roadmap `review-skill-v2`, which hold names the closed rule keeps out of the tracked
files; the tree no longer holds them.

Starting the history again leaves the records citing it: 551 lines in 44 tracked files cite one of
its commits, 41 of those files in the closed roadmaps and the reviews, and 9 lines in four files
cite a session's id or a path into a session's transcript or scratch folder. Two tools read that
history: the review skill's evals build their fixtures by `git archive` of `main` as of
2026-10-30, so `test_fixtures.py`, `test_findings.py` and `test_scope.py` fail once it is gone; and
`scope.py` scopes a phase's or a roadmap's review from its reports' `**Start Commit:**` lines.

---

## Decisions Taken At Opening

Taken with the user on 2026-11-06:

- **Name:** `fresh-history-and-ci`. **Purpose:** a history starting at one root commit, a private
  remote, and a CI building each tag, so that 0.2.0 is released through it.
- **Order:** ai-ctl's reset now, before the agent setup's, which plans the same reset and has not
  started it: ai-ctl's CI and 0.2.0 wait on it, and ai-ctl already started its history again once,
  from a verified bundle (roadmap `v2-rebuild`, phase 0). What this reset meets goes to the agent
  setup as argued design-feedback notes.
- **The reset:** one root commit holding the tree as it is; the old history archived outside the
  repository as a `git bundle --all` beside v1's, verified before anything is removed. Set aside:
  rewriting each of the 480 commits, their messages, trailers and trees.
- **The records rewritten in place:** each citation of the old history — a commit's hash, a
  session's id, a path into a session's transcript or scratch folder — replaced by the work's date
  and name, or dropped; an exception, for this rewrite alone, to the rule that a closed record is
  never rewritten. Set aside: the records kept as they are, with one line saying where the old
  history lives.
- **Commit hashes of the new history stay allowed:** the review's scope reads a phase report's
  `**Start Commit:**`, its records keep `base`, `end` and `fixed_in`, and the new history is never
  rewritten. Set aside: a rule that a tracked file cites no commit, which the agent setup plans
  for itself.
- **No session cited from now on:** a convention of `CLAUDE.md` that a tracked file cites no
  session's id and no path into a session's transcript or scratch folder, which no one but the
  user on this machine can open, private repository or not.
- **The remote:** a private GitHub repository named `ai-ctl`, created empty by the user, `gh` not
  being installed here; `origin` over SSH, and `push = "auto"` from the reset on. Private until
  the user decides otherwise.
- **The CI:** GitHub Actions, run by a pushed tag `v*` and by hand, in a Fedora 44 container pinned
  by digest, Fedora 44 being the only target: the checks of `CLAUDE.md` and `cargo deny check`,
  then the release build, published with its SHA-256 on the tag's release. The closed rule's check,
  `sid.py check`, stays the pre-push hook's, which runs on every push.
- **0.2.0's release follows this roadmap:** a tag is made on `main`, and the roadmap lives on its
  branch until its closure, so the release comes after the integration, as the CI's first tag.
- **Reviews:** each phase that changes code ends with the review of its hunks, phase 0's before the
  reset removes the commits it reads; the closing pass is scoped from the root commit, since a scope
  reaching back past it would be the whole tree.
- **Branch:** `roadmap/fresh-history-and-ci`, made at the first opening and made again from the
  root commit at the reset.

---

## Deliberately Out Of Scope

- 0.2.0's release, right after this roadmap's integration.
- Making the repository public: the user's decision.
- A rule against commit hashes in the tracked files, and a check of one.
- A CI run on every push or pull request: the CI runs at tags and by hand.
- A script for the reset, which runs once: the procedure lives in phase 1's report.
- Any direct change to the agent setup's tools, its own reset among them.
- Another platform than Fedora 44.
- The Makefile running a release binary: it builds the working tree until the architecture is
  stable (the user, 2026-10-30).

---

## Phases

- [Phase 0: Clean Records](phase-0-clean-records.md)
- [Phase 1: The Reset And The Remote](phase-1-the-reset-and-the-remote.md)
- [Phase 2: CI At Each Tag](phase-2-ci-at-each-tag.md)

---

## Dependencies

- Each phase builds on the previous one.
- The roadmap skill's report template keeps its `**Start Commit:**` line, which `scope.py` reads: a
  phase opened under a template without it has no scope for its review. The agent setup kept it,
  on phase 0's note (2026-11-06).
- Phase 1 waits for the user to create the remote on GitHub.
- The reviews start paid sessions, each batch after the user's yes; the CI's runs spend GitHub
  Actions' minutes of a private repository.

---

## Related Documentation

- [docs/security.md](../../../security.md) — what ai-ctl pins and how: the downloads, the images
- [docs/toolchain.md](../../../toolchain.md) — the toolchain, with the record of each change
- [CLAUDE.md](../../../../CLAUDE.md) — the archive of v1, Releases, Reviews, Checks
- [reviewing-delivered-code](../../../../.claude/skills/reviewing-delivered-code/SKILL.md) — `scope.py` and the evals' fixtures
- roadmap `v2-rebuild`: `phase-0-reset-and-skeleton.md` — ai-ctl's first restart from a bundle

---

## Metadata

**Roadmap Status:** 🟢 Done
**Created:** 2026-11-06
**Branch:** roadmap/fresh-history-and-ci, from main at `chore: start the history again from the tree of 2026-11-06` of 2026-11-06
