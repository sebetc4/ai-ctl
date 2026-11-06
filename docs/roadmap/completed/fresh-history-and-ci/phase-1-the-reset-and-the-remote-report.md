# Phase 1 Report: The Reset And The Remote

**Phase:** [phase-1-the-reset-and-the-remote.md](phase-1-the-reset-and-the-remote.md)
**Start Commit:** `docs(fresh-history-and-ci): close phase 0` of 2026-11-06

---

## Work Log

### 2026-11-06

The remote created empty by the user on GitHub, `sebetc4/ai-ctl`; SSH authenticates there as
`sebetc4` with this machine's key.

The phase's start: no other roadmap open, phase 0 closed, the ten checks green on the opening
commit. The old history written by `git bundle create --all` to
`/code/rust/projects/llm-ctl-v1-archive/ai-ctl-history-2026-11-06.bundle`, beside v1's: 3.5 MB,
SHA-256 `fb59a1403f25b0c3c7ce215ad84b8d419be6c1e8907b35fdb34e5fbe5311b8cb`, its `HEAD` the
opening of this phase. `git bundle verify` finds it whole, recording the complete history; a
mirror clone of it holds 490 commits, as the repository does, and its two refs, `main` and
`roadmap/fresh-history-and-ci`, at the repository's commits, the two `git for-each-ref` listings
equal; `git fsck --full` finds nothing in the clone. The repository holds no tag, no stash and no
other worktree.

This roadmap's citations of the old history rewritten as phase 0 did, on the tree the root commit
takes: the two reports' `**Start Commit:**` lines, the README's `**Branch:**` line, phase 0's
report naming its own start commit, and phase 0's review, its `base`, `end` and `mutants.diff`,
whose `report.md` `findings.py report` wrote again, exiting 0. Phase 0's two commands, run on
that tree with the old objects still in the clone, find nothing outside the fixtures but the
Hub's blob id `e8669c03…`, which the tree holds.

The reset. The tree the root commit was to hold computed first in the old repository, on a
separate index: `HEAD` and the rewrites above, 329 files. Then, `HEAD` still the bundle's and the
bundle verified again, the old `.git` deleted and `git init -b main` run on the working tree;
`info/exclude` copied back and the pre-push hook written by `sid.py hook .`, which left the 15
top-level entries of the tracked files alone untracked. `commit.py`, which takes the current
branch of a repository without a remote, committed them:
`chore: start the history again from the tree of 2026-11-06`, its tree the one computed, its
files the 329 tracked before. `branch.py create fresh-history-and-ci` made the roadmap's branch
again from it. Both refs reach that one commit, `git cat-file --batch-all-objects` lists 1
commit, 49 trees and 328 blobs, and the ten checks are green on it. Run outside the scripts:
`git read-tree`, `git add -A` and `git write-tree` on the separate index, `rm -rf .git`,
`git init -b main`.

The remote. `origin` set to `git@github.com:sebetc4/ai-ctl.git`, empty by `git ls-remote`, and
private, GitHub answering 404 to an anonymous request for its page and its API; `push = "auto"`
in the `[git]` table, its file untracked. `main` pushed by `git push -u origin main`, outside the
scripts: `commit.py` and `branch.py` refuse a repository with a remote whose `origin/HEAD` is
unset, and an empty remote has none to set. The pre-push hook's check passed, printing
`cites 0, names 0, other 7`; `git remote set-head origin --auto` then set `origin/HEAD` to
`main`. The roadmap's branch pushed by `commit.py` with the reset's record, through the same
hook. `git ls-remote origin` then gives `main` at `chore: start the history again from the tree of 2026-11-06` and the roadmap's branch at the
reset's record, no tag; both descend from `chore: start the history again from the tree of 2026-11-06`, the clone's one root commit, and
`origin/main` and the roadmap's branch together count 2 commits.

The documents this phase's changes describe: `CLAUDE.md`, whose archive of v1 became the
archived history, naming both bundles and the day the history starts; no other document
described the remote or the old history's place. The design-feedback note written in the agent
setup's `notes/design-feedback/`, untracked there: the procedure step by step, and what bears on
the setup's own reset, `commit.py` and `branch.py` refusing a remote without `origin/HEAD`, the
tags `commit.py` makes only when no push was refused, the old remote leaving the name `origin`
before the root commit, and a new repository rather than an orphan branch. The user read it
and approved it.

---

## Decisions

- Nothing is committed on the old history after the bundle: the records of the bundle and of the
  rewrite go into the root commit, so the archive holds every commit the repository held before
  the reset, and the opening commit is the last of the old history.
- The reset made in a new repository, not on an orphan branch, the user's choice: the old `.git`
  deleted once the bundle was verified, then `git init -b main` on the working tree. A new
  repository holds nothing of the old history that a `reflog expire` and a `gc --prune=now` would
  have to be trusted to clear, such as `ORIG_HEAD`, `FETCH_HEAD`, `COMMIT_EDITMSG`, the
  commit-graph and the reflogs. Carried over from the old `.git`: the ten lines of
  `info/exclude`, and the pre-push hook, written again by `sid.py hook`; its `config` held git's
  defaults alone, the author coming from the user's global configuration.

---

## Deviations

- The reset task made in a new repository instead of on an orphan branch (Decisions), its proof
  unchanged.
- `main`'s first push made by hand, `git push -u origin main`, not through `committing-changes`:
  its scripts refuse a remote without `origin/HEAD`, which an empty remote cannot give. The
  pre-push hook ran on it all the same, and every push since goes through `commit.py`.

---

## Assessment

ai-ctl's history now starts at one root commit, `chore: start the history again from the tree of 2026-11-06` of 2026-11-06, holding the tree as it
was, in a new repository whose every ref reaches it; `main` and the roadmap's branch are on the
private remote `origin` over SSH, `origin/HEAD` at `main`, and `push = "auto"` makes
`committing-changes` push each commit. The old history lies in a verified bundle beside v1's,
which `CLAUDE.md` names with the day the history starts. The agent setup holds the procedure as
it ran, and what in its own reset plan the scripts would refuse.

Phase 2 needs first two facts. Its CI runs on a private repository, so each run spends the
account's GitHub Actions minutes. The roadmap's closing pass is scoped from the root commit, as
decided at the opening, and `scope.py roadmap` would not find that base: phase 0's and phase 1's
`**Start Commit:**` lines name commits of the old history by their subjects, which it does not
read as hashes, so it would take phase 2's start commit; the closing pass takes its scope from
`scope.py range` from `chore: start the history again from the tree of 2026-11-06`.
