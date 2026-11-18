# Review: 2026-11-18-release-0-3-1-phase-2

**Status:** complete — no finding but the guard's test's ResourceWarning, which the register holds (2026-11-12-review-skill-v2-5-phase-2 finding 16); no finding above the floor, so no verifier ran; no mutant in the scope's diff
**Scope:** roadmap `release-0-3-1`, phase 2, cff6d8e..4fec63b: 48 hunks, +57 −57 lines of code
**Go-ahead:** the user, tier `low`

## Summary

| # | Title | Axis | Effect | Severity | Origin | Verdict | Action | Batch |
|---|---|---|---|---|---|---|---|---|
| 1 | test_guard.py's child() leaves its process's stdout open, a ResourceWarning in a green run | tests | maintenance | — | older | — | known | — |

## Checks

- `cargo fmt --check`: exit 0
- `cargo clippy --all-targets -- -D warnings`: exit 0
- `cargo test`: exit 0: 951 passed, 0 failed, 0 ignored; no test skipped itself, the gateway's test end to end run with llama-swap v273
- `python3 .claude/skills/reviewing-delivered-code/evals/test_scope.py`: exit 0: 34 tests OK
- `python3 .claude/skills/reviewing-delivered-code/evals/test_fixtures.py`: exit 0: 14 tests OK
- `python3 .claude/skills/reviewing-delivered-code/evals/test_findings.py`: exit 0: 34 tests OK
- `python3 .claude/skills/reviewing-delivered-code/evals/test_regression.py`: exit 0: 15 tests OK
- `python3 .claude/skills/reviewing-delivered-code/evals/test_dependencies.py`: exit 0: 54 tests OK
- `python3 .claude/skills/reviewing-delivered-code/evals/test_guard.py`: exit 0: 37 tests OK, a ResourceWarning, the session's finding
- `python3 .github/scripts/test_check_release.py`: exit 0: 15 tests OK
- `sid.py check .`: exit 0
- `cargo mutants --in-diff` on the scope's diff: 0 mutants in 0 s, 0 missed, 0 caught, 0 unviable

## Agents

- `session`, effort `low`: answered
- `review-verifier`, effort `low`: not started

## Findings

### 1. test_guard.py's child() leaves its process's stdout open, a ResourceWarning in a green run

```
FINDING
axis: tests
where: .claude/skills/reviewing-delivered-code/evals/test_guard.py:335 (Sandbox::child)
title: test_guard.py's child() leaves its process's stdout open, a ResourceWarning in a green run
cause: the check python3 .claude/skills/reviewing-delivered-code/evals/test_guard.py passes and prints a ResourceWarning, an unclosed file <_io.TextIOWrapper name=3>, from the child process child() starts
effect: maintenance
consequence: a green run of the skill's own test prints a warning the reader must know to be harmless
fix: close the child's stdout in child()'s cleanup
test: none: the warning itself, gone from the check's output, is the proof
evidence: reproduced: python3 .claude/skills/reviewing-delivered-code/evals/test_guard.py: 37 tests OK, ResourceWarning: unclosed file <_io.TextIOWrapper name=3 encoding='UTF-8'>
```

Reviewers: session. Origin: older. Floor: below. Fingerprint: `tests | .claude/skills/reviewing-delivered-code/evals/test_guard.py | Sandbox::child`.
Register: `2026-11-12-review-skill-v2-5-phase-2` finding 16, open: test_guard.py's child() leaves its process's stdout open, a ResourceWarning in a green run.
Verdict: none, below the floor. Action: known. 2026-11-12-review-skill-v2-5-phase-2 finding 16.

## Coverage

- `session`: in full: scope.txt, src/domain/gateway.rs, tests/fixtures/llama-swap-273-checksums.txt, tests/fixtures/llama-swap-273-version.txt; in part: src/application/doctor.rs (the scope's hunks), src/application/fakes.rs (lines 605-620), src/application/gateway.rs (lines 365-530), src/application/lifecycle.rs (lines 1035-1280), src/presentation/build.rs (lines 455-600), src/application/build.rs (lines 3170-3185), tests/gateway_lifecycle.rs (lines 1-75), tests/fixtures/README.md (lines 95-120), docs/security.md (lines 20-50, 284-300), docs/gpu/llama-swap.md (lines 125-225), docs/commands.md (lines 215-250, 366-380, 445-455), docs/release.md (lines 15-30), README.md (lines 75-85, 160-182), CHANGELOG.md (lines 1-12), llama-swap/internal/server/server.go (lines 400-545), llama-swap/internal/config/config.go (lines 180-225); left out: src/: the production change is PINNED's release and digest and a doc comment; the digest checked equal to the linux_amd64 line of llama-swap-273-checksums.txt, which the test at src/domain/gateway.rs:191 ties to PINNED; security: v273's routes registered without authMW (llama-swap/internal/server/server.go:457-497) are /health, /wol-health, GET /{$}, /ui, the manifest and its two icons, /favicon.ico and GET /upstream, the list of docs/security.md section 2; /props takes the model chain, authMW first; the pin's download and check are older than the base and unchanged, for the roadmap's closing pass; tests: cargo mutants --in-diff found no mutant in the scope's diff, constants and test code alone; every literal release in the tests moved with the pin by design, so that a bump fails them before the pin moves; performance: no code path changed; dependencies: Cargo.toml and Cargo.lock unchanged
