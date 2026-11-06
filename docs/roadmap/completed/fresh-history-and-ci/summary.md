# Summary — Fresh History And CI

---

## Where We Started

0.2.0 waited on a CI that built each tag, and a CI needed a remote, which ai-ctl had never had.
Pushing the history as it stood would have published 480 commits since 2026-10-09, 268 of them
with an agent's `Co-Authored-By` trailer, and the commits of 2026-10-31 that held names the
closed rule keeps out of the tracked files. Starting the history again meant rewriting the
records that cited it, 551 lines in 44 files citing one of its commits and others citing a
session's id or a path into a session's transcript, and keeping working the two tools that read
it: the review skill's evals, whose fixtures came from `git archive` of the old `main`, and
`scope.py`, which scopes a phase's review from its report's `**Start Commit:**`.

---

## Where We Landed

ai-ctl's history starts at one root commit, `chore: start the history again from the tree of 2026-11-06` of 2026-11-06, in a new repository on the
private remote `sebetc4/ai-ctl` over SSH, every commit since the first push pushed by
`committing-changes`. The old
history lies in a verified bundle beside v1's, which `CLAUDE.md` names; the records cite each of
its commits by subject and date, and no tracked file cites a session, a rule of `CLAUDE.md`'s
Sources convention.

A release is now made by the CI. `rust-toolchain.toml` pins Rust 1.99.1 here and in the
workflow. At a pushed tag `v*`, `.github/workflows/release.yml` refuses a tag that is not
`Cargo.toml`'s version with a dated `CHANGELOG.md` entry, runs every check of `CLAUDE.md` and
`cargo deny check` and builds ai-ctl in Fedora 44's image pinned by digest, each action, download
and toolchain pinned, then publishes the binary with its `SHA256SUMS` on the tag's release from a
job that runs no code of the repository. By hand it checks and builds alone. Production installs
a release with `gh release download`, its sum checked, into `~/.local/bin`; `docs/release.md`
holds the whole of it, `docs/security.md` what the CI runs and what production installs. No tag
has been made yet: 0.2.0's is the workflow's first.

---

## What Each Phase Delivered

**Phase 0, Clean Records.** The evals' fixtures built from a tarball of ai-ctl's tree of
2026-10-30, tracked as one file, so that they need no history; every citation of the old history
rewritten to a commit's subject and date, every session cited dropped; the Sources convention in
`CLAUDE.md`; and the agent setup's agreement to keep the `**Start Commit:**` line its review
scope reads. Its review, made before the reset, found nothing.

**Phase 1, The Reset And The Remote.** The old history bundled and verified, then the reset in
a new repository rather than on an orphan branch, which left nothing of the old history to
prune; the remote created by the user, `main` and the roadmap's branch pushed, `push = "auto"`
from then on, and the procedure as it ran given to the agent setup.

**Phase 2, CI At Each Tag.** The toolchain's pin, the release's guard written test-first, the
workflow proven twice in its pinned image under rootless podman on this machine, then once on
GitHub by hand, green from end to end, its binary installed here and run. The install by `gh`,
settled with the user. The phase's review found `docs/security.md` naming one of the four scopes
of `gh`'s token; the closing pass found the CI's rustup able to update itself past its pin,
which rustup skipped on GitHub only because `CI` is set, and the toml crates a patch behind.
All three were fixed, and every recorded mutant is caught at the last fix.

---

## What We Learned

GitHub runs a workflow by hand only when the default branch holds it: on a roadmap's branch, the
proof needed the default branch switched for the dispatch and back. A binary built by cargo
holds the paths of cargo's home, so two builds compare only from the same home: the podman runs
agreed with each other, not with the runner. rustup's self-update downloads the newest
rustup-init with no checksum and runs it, and rustup skips it on its own wherever `CI` is set;
the workflow now says `--no-self-update` instead of resting on that. `scope.py` routes a
workflow and a Python script outside the code's reading, so the reviews read them by an
explicit instruction.

---

## What We Are Leaving Open

- The integration of the roadmap's branch into `main`, then 0.2.0's release: its
  `CHANGELOG.md` entry dated and the tag made by `committing-changes`, the workflow's first run
  on a tag.
- `gh`'s token holds `admin:public_key`, which adds an SSH key to the account; the user keeps it
  or drops it with `gh auth refresh --remove-scopes admin:public_key`.
- The lag the dependency axis found and did not judge, no release note read: llama-swap v263
  behind v270, llama.cpp's build b12111 behind b12177, stable-diffusion.cpp 19 tags behind,
  audio.cpp v0.9.3 behind v0.9.4.
- `findings.py add` refuses a verifier's PROBE on the finding it verifies, so a verifier cannot
  ask for a run its verdict needs; `scope.py` gives a workflow and a script no code route.
- Cargo.toml's `rust-version`, 1.89, is checked by no build.
