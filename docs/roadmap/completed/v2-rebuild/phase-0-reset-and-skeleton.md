# Phase 0: Reset & Skeleton

---

## Status

**Current Status:** 🟢 Done (100% — 14/14)
**Started:** 2026-10-09
**Completed:** 2026-10-09
**Blocked By:** —

---

## While Working

Keep `phase-0-reset-and-skeleton-report.md` current as the work happens — after each significant
step, and before every commit, pause, or end of session.

---

## Objective

Replace the v1 repository with a clean v2 base: v1 archived outside the repository, a new git
history, a compiling layered skeleton, the reference stack files under `deploy/`, and rewritten
documentation.

---

## Overview

### Why This Phase Matters
Every later phase builds on this base. Pruning v1 in place would carry its model of the world —
one backend process per model, an own model registry, a gateway — into v2. Starting from an empty
tree forces each piece of v1 to be brought back deliberately, by the phase that needs it.

### What It Enables
Phases 1 to 4 each add one capability onto a skeleton whose layering, checks and documentation are
already in place.

### Out of Scope
Any feature code: doctor, build, server lifecycle and bench belong to phases 1 to 4. No v1 code is
ported here; later phases port what they need from the archive.

---

## Tasks

### Archive v1
- [x] Commit the uncommitted Phase 6-S work on an `archive/v1-wip` branch and write a `git bundle --all` to `/code/rust/projects/llm-ctl-v1-archive/`
- [x] Verify the bundle: `git bundle verify`, clone it into a temporary directory, check the branches and the WIP commit

### Clean Slate
- [x] Delete v1 code, tests and tooling: `src/`, `tests/`, `scripts/`, `ollama/`, `config/`, `assets/`, `Makefile`, `test.rs`, `Cargo.lock`
- [x] Delete v1 documentation: everything under `docs/` except this roadmap's folder
- [x] Bring the v2 study into the tree: `deploy/models.ini` with host paths instead of container paths, `deploy/opencode.json`; remove `study/v1/` and `study/v2/`
- [x] Remove `.git` and initialise a new repository on branch `main`, without a remote

### Skeleton
- [x] Rewrite `Cargo.toml`: real metadata, edition 2024, only the dependencies the skeleton uses
- [x] Create the four layers (`domain`, `application`, `infrastructure`, `presentation`) with module docs stating each layer's role and dependency rule
- [x] Implement the clap entry point with `completions <shell>` as the only command, with CLI parsing tests
- [x] Rewrite `.gitignore` for the v2 tree

### Documentation
- [x] Write `docs/stack.md`: the v2 study conclusions in English, corrected by the 2026-10-08 research and adapted to native builds and an llm-ctl-managed server
- [x] Write `docs/architecture.md`: layers, dependency rule, port policy, and the path of a command through the layers
- [x] Rewrite `README.md` (purpose, status, build, usage of what exists) and `CLAUDE.md` (agent guide for v2)

### Validation
- [x] Run the three checks green and create the first commit of the new history

---

## Technical Details

### Files to Modify
```
.gitignore
Cargo.toml
Cargo.lock                  (regenerated)
README.md
CLAUDE.md                   (git-ignored)
deploy/models.ini           (new)
deploy/opencode.json        (new)
docs/stack.md               (new)
docs/architecture.md        (new)
src/main.rs                 (new)
src/lib.rs                  (new)
src/domain/mod.rs           (new)
src/application/mod.rs      (new)
src/infrastructure/mod.rs   (new)
src/presentation/           (new: cli.rs, mod.rs)
tests/cli.rs                (new)
everything else from v1     (deleted)
```

### Dependencies
None: this is the roadmap's first phase.

### Constraints
- Nothing is deleted before the bundle has been verified.
- No push: what happens to the GitHub remote `sebetc4/rust-llm-ctl` (force-push or a new repository) is decided with the user before any push.
- `study/cpp.md` is an unrelated note of the user: left in place and not committed.
- `CLAUDE.md` stays git-ignored, as v1's `.gitignore` already decided; `.agent-conventions.toml` stays at the root, git-ignored by the roadmap tooling.

---

## Acceptance Criteria

- [x] The archive bundle clones and holds every v1 branch plus the WIP commit
- [x] `git log` shows a single new history that starts with this phase's commit
- [x] The tracked tree holds only `Cargo.toml`, `Cargo.lock`, `README.md`, `.gitignore`, `deploy/`, `docs/`, `src/` and `tests/`
- [x] `llm-ctl completions bash` prints a completion script
- [x] `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test` pass
- [x] The README describes only commands that exist
