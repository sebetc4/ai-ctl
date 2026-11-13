# Review: 2026-11-09-capabilities-status-and-updates-phase-3

**Status:** complete — every finding fixed: batch 1 in `docs(status): say that a client checks the socket, not status` of 2026-11-09, batch 2 in `test(status): hold the newline that ends a JSON document` of 2026-11-09; the recorded mutants at `test(status): hold the newline that ends a JSON document` of 2026-11-09, 65 caught, none missed or gone
**Scope:** roadmap `capabilities-status-and-updates`, phase 3, `docs(review): complete the review of capabilities phase 2` of 2026-11-09..`feat(status): print the gateway's state as JSON` of 2026-11-09: 19 hunks, +248 −20 lines of code
**Go-ahead:** the user, tier `low`

## Summary

| # | Title | Axis | Effect | Severity | Origin | Verdict | Action | Batch |
|---|---|---|---|---|---|---|---|---|
| 1 | the documents have status --json say that the gateway's PID holds the socket, which status never checks | the CLI's contract | misleading | low | new | CONFIRMED | fix | 1 |
| 2 | no test holds the newline that ends a --json document | tests | none | — | new | — | fix at once | 2 |

## Checks

- `cargo fmt --check`: exit 0
- `cargo clippy --all-targets -- -D warnings`: exit 0
- `cargo test`: exit 0: 871 passed, 0 failed, 0 ignored; no test skipped itself, the gateway's test end to end run with llama-swap v271
- `python3 .claude/skills/reviewing-delivered-code/evals/test_scope.py`: exit 0: 34 tests
- `python3 .claude/skills/reviewing-delivered-code/evals/test_fixtures.py`: exit 0: 14 tests
- `python3 .claude/skills/reviewing-delivered-code/evals/test_findings.py`: exit 0: 34 tests
- `python3 .claude/skills/reviewing-delivered-code/evals/test_regression.py`: exit 0: 15 tests
- `python3 .claude/skills/reviewing-delivered-code/evals/test_dependencies.py`: exit 0: 25 tests
- `python3 .claude/skills/reviewing-delivered-code/evals/test_guard.py`: exit 0: 37 tests
- `python3 .github/scripts/test_check_release.py`: exit 0: 15 tests
- `sid.py check .`: exit 0: cites 0, names 0, other 0
- `cargo mutants --in-diff` on the scope's diff: 7 mutants in 36 s, 0 missed, 7 caught, 0 unviable

## Agents

- `review-verifier`, effort `low`: answered
- `session`, effort `low`: answered

## Findings

### 1. the documents have status --json say that the gateway's PID holds the socket, which status never checks

```
FINDING
axis: the CLI's contract
severity: low
where: docs/security.md:138 (top level), docs/security.md:216 (top level), docs/stack.md:478 (top level), src/application/lifecycle.rs:380 (Lifecycle::status), src/application/lifecycle.rs:566 (Lifecycle::wait_until_healthy)
title: the documents have status --json say that the gateway's PID holds the socket, which status never checks
cause: three statements read that the agents' client sends the key once status --json says the gateway runs at a loopback address whose listening socket its PID holds; status reads the record and whether its PID still runs llama-swap, and looks at no socket; only up checks the socket, through ports.listens, and the check of the socket is the client's own, from /proc
effect: misleading
consequence: a reader of the security document takes running: true for proof that the gateway holds the port; while up waits for a llama-swap it has recorded, status --json says running with a process that may not hold the port, so a client written from these lines, reading status --json alone, sends the key to whoever holds it
fix: reword the three statements so that status --json gives whether the gateway runs, its address and its PID, and the client itself checks from /proc that this PID holds the listening socket; say in docs/commands.md, beside the fields a client relies on, that status does not look at the socket
test: none: a document's wording, which the review's reading of the code settles
evidence: read: docs/security.md:138-139, docs/security.md:216 and docs/stack.md:478, the statements; docs/commands.md:435, which already gives the check to the client; src/application/lifecycle.rs:380-405, status reading the record and its liveness alone; git grep 'listens(' -- src, its one production call at src/application/lifecycle.rs:566, in up's wait, whose doc comment says the record stays until the wait fails
```

Reviewers: session. Origin: new. Floor: above. Fingerprint: `the CLI's contract | docs/security.md + docs/stack.md + src/application/lifecycle.rs | Lifecycle::status + Lifecycle::wait_until_healthy + top level`.
Verdict: CONFIRMED. Action: fix, batch 1. The verifier added docs/commands.md's gateway.pid row and found that no document says how the client checks; the fix names no /proc, and the row keeps that gateway.pid is the process that listens, which the agents' client relies on.
Fixed in `docs(status): say that a client checks the socket, not status`.

### 2. no test holds the newline that ends a --json document

```
FINDING
axis: tests
where: src/presentation/mod.rs:149 (json_text), tests/cli.rs:295 (status_json_without_a_gateway_exits_3_with_its_document), tests/cli.rs:278 (doctor_exits_1_when_a_check_fails)
title: no test holds the newline that ends a --json document
cause: every test of a --json output parses its stdout with serde_json, which reads a document the same with or without its last newline, so text.push('\n') in json_text can go with every test green
effect: none
consequence: with the push removed, status --json and doctor --json end without a newline, which only a shell's prompt, printed on the document's last line, shows
fix: assert in status_json_without_a_gateway_exits_3_with_its_document that stdout ends with a closing brace and a newline
test: status_json_without_a_gateway_exits_3_with_its_document, watched failing with the push removed
evidence: read: src/presentation/mod.rs:149-153 and the three tests of status --json and doctor_exits_1_when_a_check_fails in tests/cli.rs, each going through serde_json::from_slice
```

Reviewers: session. Origin: new. Floor: below. Fingerprint: `tests | src/presentation/mod.rs + tests/cli.rs | doctor_exits_1_when_a_check_fails + json_text + status_json_without_a_gateway_exits_3_with_its_document`.
Verdict: none, below the floor. Action: fix at once, batch 2. Held by a unit test of json_text, which every --json output goes through, rather than by one command's test.
Fixed in `test(status): hold the newline that ends a JSON document`.

## Coverage

- `review-verifier`: in full: none; in part: docs/security.md (lines 130-146, 212-220), docs/stack.md (lines 474-482), docs/commands.md (lines 420-445), src/application/lifecycle.rs (lines 296-335, 375-410, 535-590, 712-745), src/infrastructure/gateway_process.rs (grep of lines 151-226, 349), src/application/ports.rs (lines 421-445, by grep), src/presentation/lifecycle.rs (lines 232-264, by grep); left out: the agents' client: a separate project, outside this repository; how it makes its check was not read
- `session`: in full: src/presentation/lifecycle.rs; in part: src/application/lifecycle.rs (lines 120-230, 375-465, 535-600, 714-760, 797-830, 935-975), src/presentation/mod.rs (lines 80-140, 149-178), src/presentation/cli.rs (lines 20-80), tests/cli.rs (lines 276-392), src/application/ports.rs (lines 431-455), src/infrastructure/gateway_api.rs (lines 86-109), src/infrastructure/gateway_process.rs (lines 151-171), docs/commands.md (lines 36-51, 126-155, 383-441), docs/security.md (lines 1-146, 216), docs/stack.md (lines 475-480), docs/architecture.md (lines 1-30), CHANGELOG.md (lines 1-6), README.md (lines 197-204); left out: none

## Fix Plan

- Batch 1: findings 1
- Batch 2: findings 2
