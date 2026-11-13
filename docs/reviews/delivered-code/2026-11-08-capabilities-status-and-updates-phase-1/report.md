# Review: 2026-11-08-capabilities-status-and-updates-phase-1

**Status:** complete — finding 1 confirmed by its verifier and fixed in batch 1, the user's strict pin on gateway install's side; the recorded mutants run at the fix: 58 caught, none missed, M51 to M53 gone with the matrix sets that phase 1 removed, taken out of the record
**Scope:** roadmap `capabilities-status-and-updates`, phase 1, `docs(capabilities-status-and-updates): close phase 0` of 2026-11-07..`test(chat): read the gateway's chat captured through llama-swap v270` of 2026-11-08: 97 hunks, +334 −359 lines of code
**Go-ahead:** the user, tier `low`

## Summary

| # | Title | Axis | Effect | Severity | Origin | Verdict | Action | Batch |
|---|---|---|---|---|---|---|---|---|
| 1 | gateway install makes current a llama-swap older than v267 without a word, as its documented example does | the CLI's contract, error handling | misleading | low | new | CONFIRMED | fix | 1 |

## Checks

- `cargo fmt --check`: exit 0
- `cargo clippy --all-targets -- -D warnings`: exit 0
- `cargo test`: exit 0: 864 passed, 0 failed, 0 ignored; no test skipped itself, the gateway's test end to end run with llama-swap v270
- `python3 .claude/skills/reviewing-delivered-code/evals/test_scope.py`: exit 0: 30 tests
- `python3 .claude/skills/reviewing-delivered-code/evals/test_fixtures.py`: exit 0: 14 tests
- `python3 .claude/skills/reviewing-delivered-code/evals/test_findings.py`: exit 0: 34 tests
- `python3 .claude/skills/reviewing-delivered-code/evals/test_regression.py`: exit 0: 15 tests
- `python3 .claude/skills/reviewing-delivered-code/evals/test_dependencies.py`: exit 0: 25 tests
- `python3 .claude/skills/reviewing-delivered-code/evals/test_guard.py`: exit 0: 37 tests
- `python3 .github/scripts/test_check_release.py`: exit 0: 15 tests
- `sid.py check .`: exit 0: cites 0, names 0
- `cargo deny check`: exit 0: advisories, bans, licenses and sources ok
- `cargo mutants --in-diff` on the scope's diff: 25 mutants in 84 s, 0 missed, 21 caught, 4 unviable

## Agents

- `session`, effort `low`: answered
- `review-verifier`, effort `low`: answered
- `review-dependencies`, effort `low`: answered

## Findings

### 1. gateway install makes current a llama-swap older than v267 without a word, as its documented example does

```
FINDING
axis: the CLI's contract, error handling
severity: low
where: src/domain/gateway.rs:26-28 (OLDEST), src/application/gateway.rs:79-129 (Installer::install), docs/commands.md:290 (top level)
title: gateway install makes current a llama-swap older than v267 without a word, as its documented example does
cause: OLDEST bounds the current release in up and doctor alone; install takes any release above zero, downloads it, makes it current and reports it installed, and docs/commands.md's example installs v264
effect: misleading
consequence: `ai-ctl gateway install --version 264 --sha256 <hex>`, the documented example, downloads v264 and makes it current with no warning; the next `up` refuses it, telling to run `ai-ctl gateway install`, which replaces it with v270
fix: install refuses a release older than OLDEST before any download or change of `current`, with up's reason; the documented example names a release from v267 on
test: install of release 266 refused, nothing downloaded and the current release unchanged; 267 installed and made current
evidence: read: src/application/gateway.rs:89-129 checks the release only against PINNED, for its digest; src/application/lifecycle.rs:775-779 refuses it at up; docs/commands.md:290
```

Reviewers: session. Origin: new. Floor: above. Fingerprint: `the CLI's contract | docs/commands.md + src/application/gateway.rs + src/domain/gateway.rs | Installer::install + OLDEST + top level`.
Verdict: CONFIRMED. Action: fix, batch 1. the fix the user chose on 2026-11-08: llama-swap updated only through ai-ctl's pin, gateway install installing the pinned release alone, --version and --sha256 removed.
Fixed in `fix(gateway): install the pinned llama-swap release alone`.

## Facts

- `review-dependencies`: cargo deny check: advisories, bans, licences and sources ok; duplicates thiserror 1.0.69/2.0.21 and thiserror-impl 1.0.69/2.0.21 (Cargo.lock:45-48), 1.x pulled by nvml-wrapper 0.13.0
- `review-dependencies`: dependencies.json: 65 crates, 9 direct, 0 updates within the requirements, 0 yanked, 0 archived
- `review-dependencies`: behind beyond the requirements, all transitive, none fixing an advisory: darling* 0.20.11 (0.24.1), libloading 0.8.9 (0.9.0), r-efi 6.0.0 (7.1.0), syn 2.0.119 (3.0.9), thiserror* 1.0.69 (2.0.21), windows-link 0.2.1 (0.100.0)
- `review-dependencies`: old releases: fnv 1.0.7 (2020), ident_case 1.0.1 (2019), static_assertions 1.1.0 (2019), and 9 others over a year; no RustSec warning, no archived repository
- `review-dependencies`: moved repositories: nvml-wrapper and wrapcenum-derive to rust-nvml/, static_assertions renamed; none archived
- `review-dependencies`: llama-swap: PINNED v270 (OLDEST v267); newest tag v271, one release between
- `review-dependencies`: llama.cpp: current build b12111; newest tag b12201, 90 releases between
- `review-dependencies`: whisper.cpp: current build v1.9.5; newest tag v1.9.5, no lag
- `review-dependencies`: stable-diffusion.cpp: current build master-1024-77b72a2; newest tag master-1057-5f8c19f, 33 releases between
- `review-dependencies`: audio.cpp: current build v0.9.3; newest tag v0.9.4, one release between

## Coverage

- `session`: in full: none; in part: src/domain/gateway_config.rs (lines 1-306, 570-690 and the tests of the phase's hunks), src/domain/gateway.rs (lines 1-60, 120-210), src/application/doctor.rs (lines 150-200, 660-690, 2620-2700), src/application/lifecycle.rs (lines 225-262, 730-800, 1160-1200), src/application/gateway.rs (lines 74-135), src/presentation/lifecycle.rs (lines 185-205), src/infrastructure/chat_api.rs (lines 1-110, 290-380), docs/commands.md (lines 265-300 and the phase's hunks), README.md, docs/stack.md, docs/security.md and docs/gpu/llama-swap.md (the phase's hunks), CHANGELOG.md (0.3.0's entry); left out: src/infrastructure/http.rs, src/infrastructure/gateway_api.rs, src/infrastructure/server_api.rs, src/application/fakes.rs: their hunks are captures' names and expected values moved from v263 to v270, read in the scope, their code unchanged
- `review-verifier`: in full: src/application/gateway.rs, src/presentation/gateway.rs, src/infrastructure/download.rs; in part: src/domain/gateway.rs (lines 1-160), src/presentation/cli.rs (lines 105-195), src/application/ports.rs (lines 260-275), src/application/lifecycle.rs (lines 765-782), tests/cli.rs (lines 1-250), docs/commands.md (lines 1-110, 255-310), docs/security.md (lines 276-287, by grep); left out: src/application/doctor.rs, src/application/lifecycle.rs: their side of the pin decision belongs to the phase's task, by instruction
- `review-dependencies`: in full: dependencies.out (dependencies.py's output); in part: deny.txt (cargo deny check's output: its end, the duplicates and the summary), src/domain/gateway.rs (lines 11-28), src/domain/recipe.rs (lines 160-280, repositories and tag patterns); left out: background task a81294ccc1ac9af17 and the running llama-swap and llama-server processes: left, none started by this agent, which ran no background task and no process outside git ls-remote and ai-ctl build list; release notes of llama-swap v271 and of the backends' releases between: not read, no network but git ls-remote; the lag stays a fact with no defect of ai-ctl named against it

## Fix Plan

- Batch 1: findings 1
