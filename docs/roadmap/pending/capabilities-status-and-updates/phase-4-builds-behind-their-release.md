# Phase 4: Builds Behind Their Release

---

## Status

**Current Status:** 🔴 Not Started (0% — 0/8)
**Started:** {{START_DATE}}
**Completed:** {{COMPLETION_DATE}}
**Blocked By:** —

---

## Objective

`ai-ctl build outdated` tells which current builds are behind their project's newest release, and
`ai-ctl build update` builds them, by phase 0's contract; then the four backends brought to their
newest releases with it, on this machine.

### Out of Scope
Updating llama-swap: `build outdated` names its lag, a release of ai-ctl renews its pin. An update
run by a timer or a unit.

---

## Tasks

### build outdated
- [ ] Each project's newest release read and set beside its current build in the domain, test-first: the forms `b<N>`, `vX.Y.Z` and `master-<N>-<hash>`, a build made at a commit rather than a release, a project with no build; a parser of a command's output tested first on a capture listed in `tests/fixtures/README.md`
  Proof: medium, test — each case's test watched failing then passing
- [ ] `build outdated` with phase 0's output and exit code, for the projects the inventory uses, a build its recipe no longer matches named, llama-swap's pinned release beside its newest, a project the network failed for said without hiding the others
  Proof: medium, test — the command's cases through the use case's doubles, watched failing then passing

### build update
- [ ] `build update [PROJECT]`: each project behind built at its newest release, one after the other; a failure reported and the others built; the summary naming `down` then `up` and `build use`; no build removed
  Proof: high, test — through the doubles, one project's patch refused and the others built, the exit code, every earlier build still installed
- [ ] A patch of ai-ctl that no longer applies to a project's newest release refreshed test-first, or the project left at its build with the reason in the report
  Proof: high, test — the patched server refusing a request without the key, as for the release it replaces

### The Use
- [ ] On this machine, `build outdated` then `build update`: the four backends at their newest releases, each refusal named; `down`, `up`, and a model of each kind answering through the gateway
  Proof: high, use — the outputs in the report
- [ ] `bench run` on the chat model before and after the update, set side by side by `bench compare`
  Proof: medium, measure — the comparison in the report

### Closing
- [ ] The documents this phase's changes describe at their current state: `docs/commands.md`'s `build`, each backend's document for a release that changes what it says, `docs/security.md` §7 for what the update fetches and builds; the phase's line in 0.3.0's entry
  Proof: medium, review — each document changed, named in the report
- [ ] The review of the phase's hunks by `reviewing-delivered-code`, its findings fixed in the phase
  Proof: medium, review — the review's report, each finding fixed or registered

---

## Technical Details

### Files to Modify
```
src/domain/build.rs
src/domain/recipe.rs
src/application/build.rs
src/infrastructure/git.rs
src/presentation/build.rs
src/presentation/cli.rs
src/domain/patches/
tests/fixtures/
CHANGELOG.md
docs/commands.md
docs/security.md
docs/audio/whisper-cpp.md
docs/audio/audio-cpp.md
docs/image/stable-diffusion-cpp.md
docs/stack.md
```

### Dependencies
Phase 0's Design: the contract. Phase 3 closed.

### Constraints
`docs/gpu/llama-swap.md` and `docs/security.md` read first. Code test-first; no test depends on the
network, a GPU or timing; `cargo mutants --in-diff` on each commit's code. The builds of the use
take the GPU and the toolchain: run while the user does not need the gateway, `down` first if a
build must replace one the gateway serves.
