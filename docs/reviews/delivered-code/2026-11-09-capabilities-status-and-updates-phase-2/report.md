# Review: 2026-11-09-capabilities-status-and-updates-phase-2

**Status:** complete — every finding fixed: batch 1 in `fix(inventory): split the window when no-kv-unified follows kv-unified` of 2026-11-09, batch 2 in `docs(gateway): bound the declared window by the training context` of 2026-11-09, batch 3 in `refactor(inventory): merge a model's flags over its defaults once` of 2026-11-09; the recorded mutants at `refactor(inventory): merge a model's flags over its defaults once` of 2026-11-09, 64 caught, none missed or gone
**Scope:** roadmap `capabilities-status-and-updates`, phase 2, `chore: remove the study line of .gitignore, private covers it` of 2026-11-08..`docs: describe the models' capabilities and the agents' client` of 2026-11-09: 13 hunks, +301 −5 lines of code
**Go-ahead:** the user, tier `low`

## Summary

| # | Title | Axis | Effect | Severity | Origin | Verdict | Action | Batch |
|---|---|---|---|---|---|---|---|---|
| 1 | the window of one request ignores no-kv-unified | bugs | misleading | low | new | CONFIRMED | fix | 1 |
| 2 | the declared window says a client keeping to it is never refused, past the model's training context too | bugs, the CLI's contract | misleading | low | new | CONFIRMED | fix | 2 |
| 3 | two places merge a model's flags over its kind's defaults | refactoring | maintenance | — | new | — | fix at once | 3 |

## Checks

- `cargo fmt --check`: exit 0
- `cargo clippy --all-targets -- -D warnings`: exit 0
- `cargo test`: exit 0: 863 passed, 0 failed, 0 ignored; no test skipped itself, the gateway's test end to end run with llama-swap v271
- `python3 .claude/skills/reviewing-delivered-code/evals/test_scope.py`: exit 0: 34 tests
- `python3 .claude/skills/reviewing-delivered-code/evals/test_fixtures.py`: exit 0: 14 tests
- `python3 .claude/skills/reviewing-delivered-code/evals/test_findings.py`: exit 0: 34 tests
- `python3 .claude/skills/reviewing-delivered-code/evals/test_regression.py`: exit 0: 15 tests
- `python3 .claude/skills/reviewing-delivered-code/evals/test_dependencies.py`: exit 0: 25 tests
- `python3 .claude/skills/reviewing-delivered-code/evals/test_guard.py`: exit 0: 37 tests
- `python3 .github/scripts/test_check_release.py`: exit 0: 15 tests
- `sid.py check .`: exit 0: cites 0, names 0, other 0
- `cargo mutants --in-diff` on the scope's diff: 24 mutants in 89 s, 0 missed, 22 caught, 2 unviable

## Agents

- `review-verifier`, effort `low`: answered
- `session`, effort `low`: answered

## Findings

### 1. the window of one request ignores no-kv-unified

```
FINDING
axis: bugs
severity: low
where: src/domain/inventory.rs:496 (Inventory::request_window), src/domain/gateway_config.rs:501-516 (flags)
title: the window of one request ignores no-kv-unified
cause: request_window takes the cache as unified whenever kv-unified is true, but flags writes both kv-unified and no-kv-unified when the defaults set one and the model the other, in key order, and llama-server keeps the last, no-kv-unified
effect: misleading
consequence: with parallel = 2 and kv-unified = true in the defaults and no-kv-unified = true in a model, /v1/models declares the whole ctx_size while each of the two slots holds half of it, so a client that keeps to the declared context is refused past half
fix: unified only when kv-unified is true and no-kv-unified is not, the later of the two on the command line
test: a case of a_chat_models_context_is_the_window_of_one_request: defaults parallel 2 and kv-unified true, the model no-kv-unified true, window 49152
evidence: read: common/arg.cpp:1675-1679 at llama.cpp b12111, one handler for --kv-unified and --no-kv-unified setting kv_unified, the last argument winning; src/domain/gateway_config.rs:501-516, the flags written in BTreeMap order, kv-unified before no-kv-unified
```

Reviewers: session. Origin: new. Floor: above. Fingerprint: `bugs | src/domain/gateway_config.rs + src/domain/inventory.rs | Inventory::request_window + flags`.
Verdict: CONFIRMED. Action: fix, batch 1.
Fixed in `fix(inventory): split the window when no-kv-unified follows kv-unified`.

### 2. the declared window says a client keeping to it is never refused, past the model's training context too

```
FINDING
axis: bugs, the CLI's contract
severity: low
where: src/domain/inventory.rs:487 (Inventory::request_window), docs/gpu/llama-swap.md:118-119 (top level)
title: the declared window says a client keeping to it is never refused, past the model's training context too
cause: llama-server caps each slot at the model's training context as well (n_ctx_slot in tools/server/server-context.cpp at b12111), which the inventory does not read, while the doc comment and docs/gpu/llama-swap.md promise that a client keeping to the window is never refused
effect: misleading
consequence: a model given a ctx_size above its training context declares that ctx_size, and a request above the training context is refused, against what the document says
fix: none in the code, the training context being out of the inventory's reach: the doc comment and the document say the window holds within the model's training context, which llama-server also caps a slot at
test: none: a statement of a document and a comment
evidence: read: tools/server/server-context.cpp:5066-5073 at b12111, n_ctx_slot capped by llama_model_n_ctx_train; :4083, a request above the slot's context refused; docs/gpu/llama-swap.md, "a client that keeps to it is never refused"
```

Reviewers: session. Origin: new. Floor: above. Fingerprint: `bugs | docs/gpu/llama-swap.md + src/domain/inventory.rs | Inventory::request_window + top level`.
Verdict: CONFIRMED. Action: fix, batch 2.
Fixed in `docs(gateway): bound the declared window by the training context`.

### 3. two places merge a model's flags over its kind's defaults

```
FINDING
axis: refactoring
where: src/domain/inventory.rs:489-495 (Inventory::request_window), src/domain/gateway_config.rs:503-504 (flags)
title: two places merge a model's flags over its kind's defaults
cause: request_window looks a flag up in the model's flags then in the defaults, while flags extends the defaults with the model's flags: the precedence is written twice, in two forms
effect: maintenance
consequence: a change to how a model's flags combine with the defaults, made in one place, leaves the declared window computed from flags other than those the server runs with
fix: Inventory::flags(kind, model), the defaults overlaid with the model's own, which gateway_config's flags extends its built-in defaults with and request_window reads
test: none: a refactoring the existing tests of both cover
evidence: read: src/domain/inventory.rs:489-495, the lookup model then defaults; src/domain/gateway_config.rs:503-504, the extend of defaults then model flags
```

Reviewers: session. Origin: new. Floor: below. Fingerprint: `refactoring | src/domain/gateway_config.rs + src/domain/inventory.rs | Inventory::request_window + flags`.
Verdict: none, below the floor. Action: fix at once, batch 3.
Fixed in `refactor(inventory): merge a model's flags over its defaults once`.

## Coverage

- `review-verifier`: in full: none; in part: src/domain/inventory.rs (lines 163, 440-516, 1157-1202), src/domain/gateway_config.rs (lines 490-520), docs/gpu/llama-swap.md (lines 110-122), /home/seb/.local/share/ai-ctl/sources/llama.cpp/common/arg.cpp (lines 1670-1682), /home/seb/.local/share/ai-ctl/sources/llama.cpp/tools/server/server-context.cpp (lines 4078-4088, 5060-5076); left out: llama-server run on a real model: GPU in use, both findings settled by reading and the unit test
- `session`: in full: src/domain/inventory.rs (lines 1-520 and the new tests), src/domain/gateway_config.rs (lines 1-720), tests/gateway_config.rs (lines 1-130); in part: src/domain/gateway_config.rs (tests, lines 720-860, 1300-1420, 1576-1840), tests/snapshots/gateway-shipped.yaml (lines 1-80, 340-356), docs/gpu/llama-swap.md, docs/stack.md (§2, §6.1, §6.4, §6.5, §7.1), docs/security.md (§3, §6), docs/commands.md (gateway config), README.md (lines 180-225), CHANGELOG.md (0.3.0), llama.cpp b12111 common/arg.cpp (lines 1345-1360, 1580-1610, 1670-1685, 2490-2512, 2525-2555), tools/server/server.cpp (lines 118-146), tools/server/server-context.cpp (lines 1435-1470, 4080-4086, 5060-5075), src/llama-context.cpp (lines 405-440), common/fit.cpp (lines 195-260), tools/server/server-models.cpp (lines 355-372); left out: docs/reviews/delivered-code/: the reviews' own records, not code

## Fix Plan

- Batch 1: findings 1
- Batch 2: findings 2
- Batch 3: findings 3
