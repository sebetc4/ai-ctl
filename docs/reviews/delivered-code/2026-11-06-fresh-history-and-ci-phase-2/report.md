# Review: 2026-11-06-fresh-history-and-ci-phase-2

**Status:** complete — one finding, the session's, confirmed and fixed: security.md named one of the four scopes of gh's token; the recorded mutants, run again at the fix, 58 caught
**Scope:** roadmap `fresh-history-and-ci`, phase 2, `docs(fresh-history-and-ci): close phase 1` of 2026-11-06..`docs: describe what the CI runs and what production installs` of 2026-11-06: 1 hunk, +4 −8 lines of code
**Go-ahead:** the user, tier `low`

## Summary

| # | Title | Axis | Effect | Severity | Origin | Verdict | Action | Batch |
|---|---|---|---|---|---|---|---|---|
| 1 | security.md understates the reach of gh's token | security | misleading | low | new | CONFIRMED | fix | 1 |

## Checks

- `cargo fmt --check`: exit 0
- `cargo clippy --all-targets -- -D warnings`: exit 0
- `cargo test`: exit 0: 867 passed, 0 failed, 0 ignored; no test skipped itself, the gateway's test end to end run with llama-swap v263
- `python3 .claude/skills/reviewing-delivered-code/evals/test_scope.py`: exit 0: 30 tests
- `python3 .claude/skills/reviewing-delivered-code/evals/test_fixtures.py`: exit 0: 14 tests
- `python3 .claude/skills/reviewing-delivered-code/evals/test_findings.py`: exit 0: 32 tests
- `python3 .claude/skills/reviewing-delivered-code/evals/test_regression.py`: exit 0: 15 tests
- `python3 .claude/skills/reviewing-delivered-code/evals/test_dependencies.py`: exit 0: 25 tests
- `python3 .claude/skills/reviewing-delivered-code/evals/test_guard.py`: exit 0: 37 tests
- `python3 .github/scripts/test_check_release.py`: exit 0: 15 tests
- `sid.py check .`: exit 0: cites 0, names 0
- `cargo mutants --in-diff` on the scope's diff: 2 mutants in 20 s, 0 missed, 2 caught, 0 unviable

## Agents

- `review-verifier`, effort `low`: answered
- `session`, effort `low`: answered

## Findings

### 1. security.md understates the reach of gh's token

```
FINDING
axis: security
severity: low
where: docs/security.md:390 (top level)
title: security.md understates the reach of gh's token
cause: the subsection "ai-ctl Itself" gives gh's token the scope `repo` alone, where `gh auth login` with SSH as git protocol granted `repo`, `read:org`, `gist` and `admin:public_key`
effect: misleading
consequence: a reader weighing what code run as the user gains from the keyring misses that the token can add an SSH key to the account, a way back in that outlives the key in the agent
fix: name the four scopes, `admin:public_key` coming from the login's offer to upload an SSH key and removable with `gh auth refresh --remove-scopes admin:public_key`, the three others gh's minimum
test: none: a document; its proof is `gh auth status`'s scopes line against the text
evidence: reproduced: `gh auth status` on 2026-11-06 prints "Token scopes: 'admin:public_key', 'gist', 'read:org', 'repo'"; `gh auth refresh --help` of gh 2.97.0 says the minimum set, `repo`, `read:org` and `gist`, cannot be removed
```

Reviewers: session. Origin: new. Floor: above. Fingerprint: `security | docs/security.md | top level`.
Verdict: CONFIRMED. Action: fix, batch 1.
Fixed in `docs(security): name every scope of gh's token`.

## Facts

- `session`: the one code hunk, src/infrastructure/gateway_process.rs:450-453 (kernel_address), keeps the behaviour: as_chunks::<4> on 4 or 16 octets leaves no remainder, as chunks_exact did, and both of cargo mutants --in-diff's mutants are caught
- `session`: the workflow passes the tag to the guard through the environment, not into the script's text; its top-level permissions are none, the build job reads, the release job alone writes and runs only gh; each action is pinned by its commit, the image by its digest, rustup-init and cargo-deny by their SHA-256
- `session`: the guard refuses a tag not of the form vX.Y.Z, another version than Cargo.toml's [package], and a version whose entry is missing, given twice or not a real YYYY-MM-DD date; committing-changes makes annotated tags, which --notes-from-tag needs
- `session`: the workflow's run by hand on GitHub on 2026-11-06, at `test(ci): leave no bytecode cache from the release guard's tests`, whose code is the scope's end, the later commits changing documents alone, was green from end to end, and no test skipped itself there or here

## Coverage

- `review-verifier`: in full: none; in part: docs/security.md (lines 370-410), scope.txt (lines 1-5), findings.md; left out: none
- `session`: in full: .github/workflows/release.yml, .github/scripts/check_release.py, rust-toolchain.toml, docs/release.md; in part: src/infrastructure/gateway_process.rs (lines 400-470), .github/scripts/test_check_release.py (its test names and imports), docs/security.md (§1's table, §7 from line 242 to 395); left out: docs/toolchain.md: its new record entry lists the commands run on 2026-11-06, checked when written; the roadmap's folder: the roadmap's own files, records outside the code's reading

## Fix Plan

- Batch 1: findings 1
