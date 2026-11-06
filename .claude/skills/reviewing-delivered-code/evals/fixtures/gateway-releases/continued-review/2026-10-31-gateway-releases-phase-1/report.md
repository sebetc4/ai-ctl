# Review: 2026-10-31-gateway-releases-phase-1

**Status:** verification — usage.py estimate --start refused the verifiers' wave at 92% of the window, 2026-10-31 12:40; the review resumes at its verification, under the go-ahead it has
**Scope:** roadmap `gateway-releases`, phase 1, {{commit:0}}..{{commit:4}}: 18 hunks, +210 −6 lines of code
**Go-ahead:** the request, tier `medium`

## Summary

| # | Title | Axis | Effect | Severity | Origin | Verdict | Action | Batch |
|---|---|---|---|---|---|---|---|---|
| 1 | prune keeps v99 and removes v100: the releases are sorted as text | bugs | wrong action | medium | new | to verify | — | — |
| 2 | a data directory whose path holds a space makes prune remove other directories | bugs | data lost | high | new | to verify | — | — |
| 3 | a failed removal is reported as done | error handling | misleading | medium | new | to verify | — | — |
| 4 | a name that is not UTF-8 makes releases panic | error handling, conventions | wrong action | low | new | to verify | — | — |
| 5 | presentation imports infrastructure::paths | architecture | maintenance | — | new | — | — | — |
| 6 | releases_to_prune copies to_prune's selection | refactoring | maintenance | — | new | — | — | — |
| 7 | newest is called nowhere | dead and legacy code | maintenance | — | new | — | — | — |
| 8 | the `clean` alias is kept for llm-ctl's habit, with no obligation | dead and legacy code | maintenance | — | new | — | — | — |
| 9 | a comment in French | conventions | maintenance | — | new | — | — | — |
| 10 | the reads of releases and size fail without their path | error handling | misleading | low | new | to verify | — | — |
| 11 | the tests let the ordering bug through | tests | wrong action | medium | new | to verify | — | — |
| 12 | size reads each file whole to count its bytes | performance | slow | — | new | — | — | — |
| 13 | gateway prune is missing from docs/commands.md | the CLI's contract | misleading | low | new | to verify | — | — |
| 14 | prune neither holds the gateway's lock nor spares the release the running gateway runs | bugs, interruption and concurrency | wrong action | medium | new | to verify | — | — |
| 15 | a failed removal does not name the releases removed before it | error handling | misleading | low | new | to verify | — | — |
| 16 | the releases' lock says `installing` while prune holds it | error handling | misleading | low | older | to verify | — | — |
| 17 | a symlinked release directory makes prune remove its target | security | data lost | medium | new | to verify | — | — |

## Checks

- `cargo fmt --check`: green
- `cargo clippy --all-targets -- -D warnings`: green
- `cargo test`: green: 798, 28 and 3 tests; gateway_lifecycle ran
- `cargo mutants --in-diff` on {{commit:0}}..{{commit:4}}: 25 mutants in 53 s, 15 missed, 6 caught, 4 unviable; the missed to review-tests

## Agents

- `review-security`, effort `medium`: answered
- `review-correctness`, effort `medium`: answered
- `review-tests`, effort `medium`: answered
- `review-cleanup`, effort `medium`: answered
- `review-structure`, effort `medium`: answered
- `review-performance`, effort `medium`: answered
- `review-verifier`, effort `medium`: not started

## Findings

### 1. prune keeps v99 and removes v100: the releases are sorted as text

```
FINDING
axis: bugs
severity: medium
where: src/infrastructure/gateway_store.rs:193 (FsGatewayStore::releases), src/domain/gateway.rs:134 (releases_to_prune)
title: prune keeps v99 and removes v100: the releases are sorted as text
cause: releases sorts the directory names as strings, and releases_to_prune trusts that order
effect: wrong action
consequence: with v99, v100 and v263 installed and v263 current, `gateway prune --keep 1` removes v100 and keeps v99
fix: releases parses each name and sorts the releases by number
test: releases_come_by_number on v99, v100 and v263; prune_goes_by_release_number
evidence: reproduced: in review-correctness's copy, releases_come_by_number and prune_goes_by_release_number red on the order
```

Reviewers: review-correctness, review-tests. Origin: new. Floor: above. Fingerprint: `bugs | src/domain/gateway.rs + src/infrastructure/gateway_store.rs | FsGatewayStore::releases + releases_to_prune`.

### 2. a data directory whose path holds a space makes prune remove other directories

```
FINDING
axis: bugs
severity: high
where: src/infrastructure/gateway_store.rs:213 (FsGatewayStore::remove)
title: a data directory whose path holds a space makes prune remove other directories
cause: remove runs `sh -c "rm -rf <dir>"` with the path unquoted
effect: data lost
consequence: with XDG_DATA_HOME at `/home/me/My Data`, prune runs `rm -rf /home/me/My` and a relative `Data/ai-ctl/llama-swap/v262`
fix: the release renamed out of sight, then removed with remove_dir_all, as FsBuildStore::remove does
test: a_removal_under_a_path_with_a_space_removes_the_release_alone
evidence: reproduced: in review-correctness's copy, a_removal_under_a_path_with_a_space_removes_the_release_alone red, the directory beside removed; review-security's gate: the variable is the user's own, no boundary crossed, so a bug and not a security finding
```

Reviewers: review-correctness, review-security. Origin: new. Floor: above. Fingerprint: `bugs | src/infrastructure/gateway_store.rs | FsGatewayStore::remove`.

### 3. a failed removal is reported as done

```
FINDING
axis: error handling
severity: medium
where: src/infrastructure/gateway_store.rs:213 (FsGatewayStore::remove)
title: a failed removal is reported as done
cause: remove ignores the command's status and returns Ok
effect: misleading
consequence: a release that stays is printed as removed, and its megabytes as freed
fix: the removal's error returned, with the path
test: a_release_that_is_not_there_is_not_removed
evidence: read: remove's body; reproduced: rm on a read-only parent, Ok returned
```

Reviewers: review-correctness. Origin: new. Floor: above. Fingerprint: `error handling | src/infrastructure/gateway_store.rs | FsGatewayStore::remove`.

### 4. a name that is not UTF-8 makes releases panic

```
FINDING
axis: error handling, conventions
severity: low
where: src/infrastructure/gateway_store.rs:186 (FsGatewayStore::releases)
title: a name that is not UTF-8 makes releases panic
cause: `.unwrap()` on into_string, outside tests
effect: wrong action
consequence: a directory named with a byte that is not UTF-8 makes `gateway prune` panic
fix: such a name skipped: it is no release of ai-ctl's
test: releases_come_by_number with a name that is not UTF-8
evidence: read: the line; CLAUDE.md, Errors
```

Reviewers: review-correctness, review-structure. Origin: new. Floor: above. Fingerprint: `error handling | src/infrastructure/gateway_store.rs | FsGatewayStore::releases`.

### 5. presentation imports infrastructure::paths

```
FINDING
axis: architecture
where: src/presentation/gateway.rs:14 (imports), src/presentation/gateway.rs:38 (run)
title: presentation imports infrastructure::paths
cause: run reads the releases' directory with Paths::from_env for its message
effect: maintenance
consequence: the layer rule broken: presentation never imports infrastructure
fix: the directory passed through Services by main.rs
test: none: no behaviour changes
evidence: read: the import; CLAUDE.md, Layers
```

Reviewers: review-structure. Origin: new. Floor: below. Fingerprint: `architecture | src/presentation/gateway.rs | imports + run`.

### 6. releases_to_prune copies to_prune's selection

```
FINDING
axis: refactoring
where: src/domain/gateway.rs:134 (releases_to_prune), src/domain/build.rs:141 (to_prune)
title: releases_to_prune copies to_prune's selection
cause: the same skip-the-newest, keep-the-current selection written twice
effect: maintenance
consequence: a change to the rule must be made twice
fix: one generic selection over an order, shared by both
test: none: the domain tests of both cover it
evidence: read: both bodies
```

Reviewers: review-cleanup. Origin: new. Floor: below. Fingerprint: `refactoring | src/domain/build.rs + src/domain/gateway.rs | releases_to_prune + to_prune`.

### 7. newest is called nowhere

```
FINDING
axis: dead and legacy code
where: src/domain/gateway.rs:150 (newest)
title: newest is called nowhere
cause: written for a later `gateway install --installed` that no phase plans
effect: maintenance
consequence: code to read and keep with no caller
fix: removed
test: none: no caller
evidence: read: a search for `newest(` finds its definition alone
```

Reviewers: review-cleanup. Origin: new. Floor: below. Fingerprint: `dead and legacy code | src/domain/gateway.rs | newest`.

### 8. the `clean` alias is kept for llm-ctl's habit, with no obligation

```
FINDING
axis: dead and legacy code
where: src/presentation/cli.rs:132 (GatewayAction::Prune)
title: the `clean` alias is kept for llm-ctl's habit, with no obligation
cause: `#[command(alias = "clean")]`, its comment naming llm-ctl
effect: maintenance
consequence: a second name for the command, documented nowhere
fix: the alias removed
test: gateway_prune_needs_a_count refuses `clean`
evidence: read: the attribute and its comment
```

Reviewers: review-cleanup. Origin: new. Floor: below. Fingerprint: `dead and legacy code | src/presentation/cli.rs | GatewayAction::Prune`.

### 9. a comment in French

```
FINDING
axis: conventions
where: src/application/gateway.rs:232 (prune)
title: a comment in French
cause: the comment on the current release
effect: maintenance
consequence: CLAUDE.md asks every comment in English
fix: the comment in English
test: none: a comment
evidence: read: the line
```

Reviewers: review-structure. Origin: new. Floor: below. Fingerprint: `conventions | src/application/gateway.rs | prune`.

### 10. the reads of releases and size fail without their path

```
FINDING
axis: error handling
severity: low
where: src/infrastructure/gateway_store.rs:186 (FsGatewayStore::releases), src/infrastructure/gateway_store.rs:202 (FsGatewayStore::size), src/infrastructure/gateway_store.rs:204 (FsGatewayStore::size)
title: the reads of releases and size fail without their path
cause: `?` without `.context()` on read_dir's entries, on size's read_dir and on its reads
effect: misleading
consequence: a release that cannot be read fails prune with `Permission denied (os error 13)` alone
fix: each read with the path in its context
test: none: the message, read
evidence: read: the three lines; CLAUDE.md, Errors
```

Reviewers: review-correctness. Origin: new. Floor: above. Fingerprint: `error handling | src/infrastructure/gateway_store.rs | FsGatewayStore::releases + FsGatewayStore::size`.

### 11. the tests let the ordering bug through

```
FINDING
axis: tests
severity: medium
where: src/application/gateway.rs:769 (tests::prune_removes_old_releases), src/domain/gateway.rs:159 (tests::prune_keeps_the_newest_releases_and_the_current_one), src/infrastructure/gateway_store.rs:176 (FsGatewayStore::releases)
title: the tests let the ordering bug through
cause: the use case's test checks is_ok alone, the domain test uses three-digit releases only, and the store's releases, size and remove have no test
effect: wrong action
consequence: finding 1 passes every test; 15 of the diff's 25 mutants are missed
fix: tests on the use case's result, on releases of one, two and three digits, and on the store's three methods
test: the tests of findings 1 to 3
evidence: cargo mutants --in-diff: 15 missed, 6 caught, 4 unviable
```

Reviewers: review-tests. Origin: new. Floor: above. Fingerprint: `tests | src/application/gateway.rs + src/domain/gateway.rs + src/infrastructure/gateway_store.rs | FsGatewayStore::releases + tests::prune_keeps_the_newest_releases_and_the_current_one + tests::prune_removes_old_releases`.

### 12. size reads each file whole to count its bytes

```
FINDING
axis: performance
where: src/infrastructure/gateway_store.rs:204 (FsGatewayStore::size)
title: size reads each file whole to count its bytes
cause: fs::read where the file's metadata gives its length
effect: slow
consequence: about 43 MB read for each release removed, a fraction of a second
fix: the length from each entry's metadata
test: a_release_measures_its_files
evidence: read: the loop
```

Reviewers: review-performance. Origin: new. Floor: below. Fingerprint: `performance | src/infrastructure/gateway_store.rs | FsGatewayStore::size`.

### 13. gateway prune is missing from docs/commands.md

```
FINDING
axis: the CLI's contract
severity: low
where: src/presentation/cli.rs:131 (GatewayAction::Prune), docs/commands.md:16 (command table), docs/commands.md:97 (Concurrency), docs/commands.md:263 (gateway)
title: gateway prune is missing from docs/commands.md
cause: the README alone was changed, though the phase's last task says both
effect: misleading
consequence: the reference lists no `gateway prune`, nor the lock it takes
fix: the command's row, section and locks in docs/commands.md
test: none: a document
evidence: read: docs/commands.md and the phase file's task
```

Reviewers: review-structure. Origin: new. Floor: above. Fingerprint: `the CLI's contract | docs/commands.md + src/presentation/cli.rs | Concurrency + GatewayAction::Prune + command table + gateway`.

### 14. prune neither holds the gateway's lock nor spares the release the running gateway runs

```
FINDING
axis: bugs, interruption and concurrency
severity: medium
where: src/application/gateway.rs:229 (prune), src/application/build.rs:522 (prune)
title: prune neither holds the gateway's lock nor spares the release the running gateway runs
cause: build prune takes the gateway's lock and refuses a build the gateway serves; gateway prune does neither
effect: wrong action
consequence: prune removes the binary a running gateway runs, and a gateway can start on a release being removed
fix: prune takes the gateway's lock and fails, removing nothing, when the running gateway runs a release to remove
test: prune_spares_the_release_the_gateway_runs; prune_waits_for_no_gateway_start
evidence: reproduced: in review-correctness's copy, prune removes v262 under a FakeGateway whose record runs it
```

Reviewers: review-correctness. Origin: new. Floor: above. Fingerprint: `bugs | src/application/build.rs + src/application/gateway.rs | prune`.

### 15. a failed removal does not name the releases removed before it

```
FINDING
axis: error handling
severity: low
where: src/application/gateway.rs:236 (prune), src/application/build.rs:557 (prune)
title: a failed removal does not name the releases removed before it
cause: prune returns the removal's error as it is, where build prune's names what is gone
effect: misleading
consequence: after a failure the user cannot tell which releases are gone
fix: the error names the releases removed before it, as build prune's does
test: prune_stops_at_a_failed_removal
evidence: read: both loops
```

Reviewers: review-correctness. Origin: new. Floor: above. Fingerprint: `error handling | src/application/build.rs + src/application/gateway.rs | prune`.

### 16. the releases' lock says `installing` while prune holds it

```
FINDING
axis: error handling
severity: low
where: src/infrastructure/gateway_store.rs:67 (FsGatewayStore::lock)
title: the releases' lock says `installing` while prune holds it
cause: lock writes the activity `installing llama-swap` whoever takes it
effect: misleading
consequence: an install during a prune fails with `another ai-ctl is installing llama-swap`
fix: the holder's activity passed to lock, a change of the port
test: none: a message
evidence: read: lock and its two callers
```

Reviewers: review-correctness. Origin: older. Floor: above. Fingerprint: `error handling | src/infrastructure/gateway_store.rs | FsGatewayStore::lock`.

### 17. a symlinked release directory makes prune remove its target

```
FINDING
axis: security
severity: medium
where: src/infrastructure/gateway_store.rs:176 (FsGatewayStore::releases), src/infrastructure/gateway_store.rs:209 (FsGatewayStore::remove)
title: a symlinked release directory makes prune remove its target
cause: releases lists a link named like a release, and remove runs rm -rf on it
effect: data lost
consequence: a link `v100 -> ~/work` would have prune remove ~/work
fix: links skipped by releases
test: none: settled by the verification
evidence: read: releases and remove
```

Reviewers: review-security. Origin: new. Floor: above. Fingerprint: `security | src/infrastructure/gateway_store.rs | FsGatewayStore::releases + FsGatewayStore::remove`.
