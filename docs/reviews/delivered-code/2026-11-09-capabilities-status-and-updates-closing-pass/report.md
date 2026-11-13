# Review: 2026-11-09-capabilities-status-and-updates-closing-pass

**Status:** complete — one finding, below the floor, fixed at once in `build(deps): update itoa and syn 3 within their requirements` of 2026-11-09: itoa and syn 3 updated within their requirements; review-security found nothing, so no verifier ran; finding 7 of phase 4's review, which the register held, fixed in `fix(build): drop the tags a project withdrew at each fetch` of 2026-11-09 under the roadmap's closing rule; the recorded mutants all caught at `fix(build): drop the tags a project withdrew at each fetch` of 2026-11-09; no review follows
**Scope:** roadmap `capabilities-status-and-updates`, phase 5, `docs: drop the last session and subagent ids from the records` of 2026-11-06..`docs(review): complete the review of capabilities phase 4` of 2026-11-09: 266 hunks, +3098 −880 lines of code
**Go-ahead:** the user, tier `high`

## Summary

| # | Title | Axis | Effect | Severity | Origin | Verdict | Action | Batch |
|---|---|---|---|---|---|---|---|---|
| 1 | two crates of the lock are behind a release their requirements already admit | dependencies | maintenance | — | older | — | fix at once | 1 |

## Checks

- `cargo fmt --check`: exit 0
- `cargo clippy --all-targets -- -D warnings`: exit 0
- `cargo test`: exit 0: 929 passed, 0 failed, 0 ignored; no test skipped itself, the gateway's test end to end run
- `python3 .claude/skills/reviewing-delivered-code/evals/test_scope.py`: exit 0: 34 tests
- `python3 .claude/skills/reviewing-delivered-code/evals/test_fixtures.py`: exit 0: 14 tests
- `python3 .claude/skills/reviewing-delivered-code/evals/test_findings.py`: exit 0: 34 tests
- `python3 .claude/skills/reviewing-delivered-code/evals/test_regression.py`: exit 0: 15 tests
- `python3 .claude/skills/reviewing-delivered-code/evals/test_dependencies.py`: exit 0: 25 tests
- `python3 .claude/skills/reviewing-delivered-code/evals/test_guard.py`: exit 0: 37 tests
- `python3 .github/scripts/test_check_release.py`: exit 0: 15 tests
- `sid.py check .`: exit 0: cites 0, names 0
- `cargo deny check`: exit 0: advisories ok, bans ok, licenses ok, sources ok; warnings: duplicates syn, thiserror, thiserror-impl, which nvml-wrapper 0.13.0 holds
- `scripts/dependencies.py`: exit 0: 65 crates, 9 direct; 2 updates within the requirements, 9 behind the newest, 0 yanked
- `scripts/regression.py`: exit 0 at `fix(build): drop the tags a project withdrew at each fetch` of 2026-11-09, the last fix, finding 7 of phase 4's review: 68 mutants, 68 caught, 0 missed, 0 gone; at `build(deps): update itoa and syn 3 within their requirements` of 2026-11-09, this review's fix, 67 caught; at `docs(review): complete the review of capabilities phase 4` of 2026-11-09, the scope's end, 67 caught
- `cargo mutants --in-diff` on the scope's diff: 140 mutants in 480 s, 0 missed, 120 caught, 20 unviable

## Agents

- `review-security`, effort `high`: answered
- `review-verifier`, effort `high`: not started
- `review-dependencies`, effort `high`: answered
- `session`, effort `high`: answered

## Findings

### 1. two crates of the lock are behind a release their requirements already admit

```
FINDING
axis: dependencies
where: Cargo.lock:252 (itoa), Cargo.lock:433 (syn)
title: two crates of the lock are behind a release their requirements already admit
cause: Cargo.lock holds itoa 1.0.18 and syn 3.0.9 while itoa 1.0.19 and syn 3.0.10, both released 2026-11-09, satisfy the requirements of serde_json and of clap_derive, serde_derive and thiserror-impl 2.0.21
effect: maintenance
consequence: no effect today; the lock drifts from the patch releases its requirements accept, and the next change to the lock takes them unreviewed with it
fix: cargo update -p itoa -p syn@3.0.9 (the version is needed: syn 2.0.119 is also locked, through darling and thiserror-impl 1.0.69, and stays there, its requirement being ^2)
test: none: a lock update within the requirements; cargo test, cargo clippy and cargo deny check after it are its proof
evidence: read: dependencies.json, itoa "update": "1.0.19" and syn 3.0.9 "update": "3.0.10", every other record "update": null; Cargo.lock:252-253 itoa 1.0.18, Cargo.lock:433-434 syn 3.0.9
```

Reviewers: review-dependencies. Origin: older. Floor: below. Fingerprint: `dependencies | Cargo.lock | itoa + syn`.
Verdict: none, below the floor. Action: fix at once, batch 1.
Fixed in `build(deps): update itoa and syn 3 within their requirements`.

## Facts

- `review-dependencies`: advisories, bans, licences, sources: cargo deny check exit 0, no advisory, no yanked version (dependencies.json: 0 yanked)
- `review-dependencies`: duplicates: syn 2.0.119 and 3.0.9, thiserror and thiserror-impl 1.0.69 and 2.0.21; the old majors come only through nvml-wrapper 0.13.0 (thiserror 1, wrapcenum-derive 0.4.1 -> darling 0.20.11 -> syn 2), the newest nvml-wrapper; build-time cost only
- `review-dependencies`: beyond the requirements, transitive, fixing no advisory: darling, darling_core, darling_macro 0.20.11 (newest 0.24.1), libloading 0.8.9 (0.9.0), r-efi 6.0.0 (7.1.0, UEFI only, not built for x86_64-unknown-linux-gnu), syn 2.0.119 (3.0.10), thiserror and thiserror-impl 1.0.69 (2.0.21), windows-link 0.2.1 (0.100.0, Windows only); none is ai-ctl's to raise, all held by nvml-wrapper or a platform crate
- `review-dependencies`: direct crates: anyhow, clap, clap_complete, nvml-wrapper, serde, serde_json, thiserror 2, toml, tempfile (dev) all at their newest release
- `review-dependencies`: last release over a year ago, none archived, no RustSec warning: fnv 1.0.7 (2020-05-14, repository pushed 2024-03-20), ident_case 1.0.1 (2019-03-18), static_assertions 1.1.0 (2019-11-03, repository pushed 2023-11-18), heck 0.5.0 (2024-03-12), strsim 0.11.1 (2024-04-02), utf8parse 0.2.2 (2024-06-08), wrapcenum-derive 0.4.1 (2024-02-10, 1 dependent, 2 owners), libloading 0.8.9 (2025-11-05), errno 0.3.14, is_terminal_polyfill and once_cell_polyfill 1.70.2, windows-sys 0.61.2; small, finished crates, most build-time or non-Linux
- `review-dependencies`: moved repositories: nvml-wrapper and wrapcenum-derive from Cldfire to rust-nvml, static_assertions to nvzqz/static-assertions; maintained under the new name, not archived
- `review-dependencies`: nvml-wrapper 0.13.0: 147 dependents, 7.7 M downloads, 317 unsafe (nvml-wrapper-sys 794), three owners, released 2026-08-31; the smallest community of the direct crates, the one FFI boundary, and the only path to NVML
- `review-dependencies`: unsafe counts: libc 800, rustix 1610, linux-raw-sys 6930, windows-sys 12530 (not built on Linux), hashbrown 455; standard ecosystem crates
- `review-dependencies`: llama-swap: PINNED v271 (src/domain/gateway.rs:20) is the newest v<N> tag of https://github.com/mostlygeek/llama-swap; no lag
- `review-dependencies`: llama.cpp: current build b12227 (2026-11-09), newest tag b12230; between, b12228, b12229, b12230
- `review-dependencies`: whisper.cpp: current build v1.9.5, newest release tag v1.9.5; no lag
- `review-dependencies`: stable-diffusion.cpp: current build master-1071-cfdf3cf (2026-11-09), newest tag master-1078-73fef08; between, master-1072-008b9a2, master-1073-991f825, master-1076-46da352, master-1078-73fef08
- `review-dependencies`: audio.cpp: current build v0.9.4, newest release tag v0.9.4; no lag
- `session`: the checks of .agent-conventions.toml are green at `docs(review): complete the review of capabilities phase 4` of 2026-11-09 and no test skipped itself, the gateway's test end to end run; cargo deny check passes, its warnings the duplicates syn, thiserror and thiserror-impl that nvml-wrapper 0.13.0 holds
- `session`: cargo mutants --in-diff on the scope's diff tests 140 mutants: 120 caught, 20 unviable, 0 missed, 0 timed out
- `session`: scripts/regression.py at `docs(review): complete the review of capabilities phase 4` of 2026-11-09: the 67 recorded mutants caught, none missed or gone

## Coverage

- `review-security`: in full: docs/security.md, src/domain/gateway_config.rs (lines 1-760, production part), src/application/gateway.rs (lines 1-160, production part), src/presentation/build.rs (lines 1-320, production part), src/domain/gateway.rs (diff `docs: drop the last session and subagent ids from the records` of 2026-11-06..`docs(review): complete the review of capabilities phase 4` of 2026-11-09), src/infrastructure/download.rs (diff `docs: drop the last session and subagent ids from the records` of 2026-11-06..`docs(review): complete the review of capabilities phase 4` of 2026-11-09), src/domain/patches/audio.cpp/api-key.patch (diff `docs: drop the last session and subagent ids from the records` of 2026-11-06..`docs(review): complete the review of capabilities phase 4` of 2026-11-09), tests/fixtures/llama-swap-271-checksums.txt; in part: src/infrastructure/git.rs (lines 1-445), src/application/build.rs (lines 1-460, 730-870), src/application/lifecycle.rs (lines 380-405, 740-800, 870-950), src/presentation/lifecycle.rs (lines 40-100, 200-290), src/infrastructure/gateway_process.rs (lines 100-200), src/domain/inventory.rs (diff `docs: drop the last session and subagent ids from the records` of 2026-11-06..`docs(review): complete the review of capabilities phase 4` of 2026-11-09), src/domain/recipe.rs (lines 1-140, 160-305), src/domain/build.rs (diff `docs: drop the last session and subagent ids from the records` of 2026-11-06..`docs(review): complete the review of capabilities phase 4` of 2026-11-09), src/domain/text.rs (lines 1-20), src/application/doctor.rs, src/application/gateway_config.rs, src/application/ports.rs, src/presentation/cli.rs, src/presentation/gateway.rs, src/presentation/mod.rs, src/infrastructure/gateway_store.rs, deploy/inventory.toml (their diffs `docs: drop the last session and subagent ids from the records` of 2026-11-06..`docs(review): complete the review of capabilities phase 4` of 2026-11-09), docs/gpu/llama-swap.md, docs/stack.md, docs/commands.md, docs/release.md (their diffs `docs: drop the last session and subagent ids from the records` of 2026-11-06..`docs(review): complete the review of capabilities phase 4` of 2026-11-09); left out: tests and the test modules of each file: read only where a production path needed them; they reach no surface docs/security.md guards; src/infrastructure/chat_api.rs, src/infrastructure/gateway_api.rs, src/infrastructure/http.rs, src/infrastructure/server_api.rs: their hunks only move test fixtures from v263 to v270; captures: test inputs, outside the reading; Cargo.toml, Cargo.lock: the dependency axis's; .claude/, docs/reviews/, CHANGELOG.md, CLAUDE.md, .gitignore, tests/snapshots/: the review skill's files and records, no surface of ai-ctl; docs/audio/audio-cpp.md, docs/image/stable-diffusion-cpp.md, docs/toolchain.md, docs/README.md, README.md: no claim on a guarded surface beyond what docs/security.md states
- `review-dependencies`: in full: cargo-deny.txt, deps/dependencies.json, dependencies-run.txt, Cargo.toml (at `docs(review): complete the review of capabilities phase 4` of 2026-11-09), deny.toml (at `docs(review): complete the review of capabilities phase 4` of 2026-11-09); in part: Cargo.lock (at `docs(review): complete the review of capabilities phase 4` of 2026-11-09, the name and version lines of itoa, syn, thiserror, thiserror-impl, darling, libloading, nvml-wrapper, r-efi, static_assertions, windows-link, wrapcenum-derive), src/domain/gateway.rs (lines 1-40), src/domain/recipe.rs (lines 60-120 and the repository and tags lines 160-290); left out: advisories: cargo deny check reports none, unmaintained and unsound included (deny.toml: unmaintained = "all", unsound = "all"), so no call into a crate was searched in src/; release notes of llama.cpp b12228-b12230 and stable-diffusion.cpp master-1072 to master-1078: not read, git ls-remote being the only reach of a remote and the local clones ai-ctl keeps in its data directory's sources folder not holding those tags; the lag stays a fact; ai-ctl build list: run with the installed ai-ctl 0.2.0, not HEAD's 0.3.0; it lists the builds on disk, which both versions read; cargo update --dry-run: not run, it would reach crates.io; the fix's command is from the lock's two syn versions; left running: the session's cargo mutants run (in the review's scratch folder) and its processes; not started by this agent, so not stopped
- `session`: in full: none; in part: none; left out: the scope's code: read by review-security, the one reviewer of a roadmap's close

## Fix Plan

- Batch 1: findings 1
