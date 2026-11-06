# Phase 2: CI At Each Tag

---

## Status

**Current Status:** 🟢 Done (100% — 9/9)
**Started:** 2026-11-06
**Completed:** 2026-11-06
**Blocked By:** —

---

## Objective

A GitHub Actions workflow that, at each pushed tag `v*` and by hand, runs ai-ctl's checks in a
Fedora 44 container pinned by digest, builds the release binary, and publishes it with its SHA-256
on the tag's release; proven on this machine in the same container, then on the remote by a run
that tags nothing; then the roadmap's closing pass.

### Out of Scope
0.2.0's tag, after the roadmap's integration. A run on each push or pull request.

---

## Tasks

### The Build
- [x] One Rust toolchain here and in the CI, pinned by the repository, so that `clippy -D warnings` judges the code the same way in both; its entry in `docs/toolchain.md`'s record
  Proof: medium, check — the same `rustc --version` in both runs, recorded
- [x] The checks of `CLAUDE.md`, `cargo deny check` and `cargo build --release --locked` green in the pinned Fedora 44 image under rootless podman on this machine, without the GPU, the user's git configuration or the study link; each test that needed one of them fixed test-first, or each check left out of the CI named and argued
  Proof: high, check — the run's output, each check green; the binary run and its SHA-256 written

### The Workflow
- [x] The workflow in `.github/workflows/`, run by a pushed tag `v*` and by `workflow_dispatch`: the image pinned by digest, each action by its commit, the token's permissions the fewest; the checks, the build, and on a tag a release refused unless the tag's version is `Cargo.toml`'s and `CHANGELOG.md`'s entry for it is dated, the binary and its `SHA256SUMS` attached to it; by hand, the same run ending with no tag
  Proof: high, test — the guard written test-first, a tag that disagrees with `Cargo.toml` and an undated entry refused, watched failing then passing
- [x] How production installs a release from the private remote — the release's binary with its `SHA256SUMS` checked, or `cargo install` from the tag over SSH — settled with the user, and written in `README.md`'s Build with its command
  Proof: medium, use — the command run on this machine on the run by hand's binary
- [x] A run by hand on the remote, on the roadmap's branch: every check green, the binary and its sum downloaded here by the install's command, the sum checked, the binary run; no tag and no release left on the remote
  Proof: high, check — the run's outcome in the report, the sum checked here, `git ls-remote --tags origin` empty

### Closing
- [x] The documents this phase's changes describe at their current state — a document of the release's domain under `docs/`, with its line in `docs/README.md`: how a tag is checked, built, published and installed, and what the workflow trusts; `docs/security.md` for what the CI runs and what production installs; `CLAUDE.md`'s Releases and Checks; `docs/architecture.md`'s Repository Layout for `.github/`
  Proof: medium, review — each document changed, named in the report
- [x] The review of the phase's hunks by `reviewing-delivered-code`, its findings fixed in the phase
  Proof: medium, review — the review's report, each finding fixed or registered
- [x] The roadmap's closing pass — the security pass, the dependency axis, the recorded mutants run again — scoped from the root commit, phase 0's hunks having been reviewed before the reset
  Proof: high, review — the pass's report and the regression run's output
- [x] The closing pass's findings that bear on this roadmap's changes or are medium or worse fixed in this phase, the others registered; no review after it
  Proof: high, test — each fix's red test seen failing first; the register's entries

---

## Technical Details

### Files to Modify
```
.github/workflows/
rust-toolchain.toml
README.md
CLAUDE.md
docs/README.md
docs/security.md
docs/toolchain.md
docs/architecture.md
docs/reviews/delivered-code/
```

### Dependencies
Phase 1 closed: the remote exists and holds the new history.

### Constraints
`docs/security.md` read before the image, the actions and each download are chosen: each pinned,
by digest, by commit or by SHA-256. Code and scripts test-first; no test depends on timing, a GPU or
the network. A run by hand spends GitHub Actions' minutes: one run proves the workflow, a second
only after a fix. The release's token writes the release alone, on a tag.
