# Phase 5: Closing Review

---

## Status

**Current Status:** 🟢 Done (100% — 66/66)
**Started:** 2026-10-17
**Completed:** 2026-10-19
**Blocked By:** —

---

## Before Starting This Phase

Read `phase-4-structure-and-refactoring.md` and `phase-4-structure-and-refactoring-report.md` in full before
touching anything here: the decisions already taken, the problems and
deviations recorded, and the changes made to this phase.

---

## While Working

Keep `phase-5-closing-review-report.md` current as the work happens — after each significant
step, and before every commit, pause, or end of session.

---

## Objective

`reviewing-delivered-code` reviews the code this roadmap delivered and writes its report to
`docs/reviews/delivered-code/`. The findings the user keeps for this roadmap are fixed, and the
others are recorded as left open.

---

## Overview

### Why This Phase Matters
The roadmap rewrites error paths, adapters and ports across the whole crate. The repository
reviews the code after each roadmap, and this review shows whether the fixes hold and added no
debt. It is also the second real use of `reviewing-delivered-code`, which its roadmap's summary
left as the proof of the changes made since the first review.

### What It Enables
The roadmap closes on reviewed code. The findings not fixed here go into the summary, as left
open.

### Out of Scope
- Changing `reviewing-delivered-code`. A defect the review shows in the skill is recorded in
  the report and goes to a change of the skill outside this roadmap.
- Code outside the scope that `scope.py` sets for this roadmap.

---

## Tasks

### Review
- [x] Run `reviewing-delivered-code` on this roadmap's scope (`scope.py roadmap` on its folder), after the user's go-ahead on its estimate. Its report goes to `docs/reviews/delivered-code/`
- [x] Get the user's verdict on each finding and their approval of the fix plan, then record which batches this roadmap fixes and which are left open

### Fixes
- [x] Batch 1: The backends off TCP loopback, or their squat recorded (finding 1)
- [x] Batch 2: The gateway's port squat documented, and up's refusal pointing to the key's rotation (finding 2)
- [x] Batch 3: `up` checks that the listener on `listen` is the process it started (finding 3)
- [x] Batch 4: Underscored flag names refused (finding 4)
- [x] Batch 5: GPU process names escaped where NVML gives them (finding 5)
- [x] Batch 6: A text flag value that is an option refused (finding 7)
- [x] Batch 7: Non-finite float flags refused (finding 11)
- [x] Batch 8: `build --force` spares the build the gateway serves (finding 8)
- [x] Batch 9: An unreadable current build, and `build list`'s `current` link, told as they are (findings 9, 62, 63)
- [x] Batch 10: doctor's CDI check reads the specification's host paths (finding 22)
- [x] Batch 11: A later format read as unsupported (finding 10, 79)
- [x] Batch 12: A remedy of its own for an inventory of a newer version (finding 80)
- [x] Batch 13: `up` checks a recorded gateway's health before "up already" (finding 12)
- [x] Batch 14: Read timeouts named (finding 14)
- [x] Batch 15: `gateway config` reads the GPU after the inventory (finding 82)
- [x] Batch 16: `LocalGateway::write` through `files::replace_file` (finding 20)
- [x] Batch 17: `prune`'s error names the builds already removed (findings 21, 39)
- [x] Batch 18: The start/stop lock tested, and `status` observing without it (findings 23, 49, 13)
- [x] Batch 19: A failed bench run keeps its completed scenarios (findings 16, 33)
- [x] Batch 20: A bench run saved after each scenario (finding 15)
- [x] Batch 21: `publish`'s hidden copies put back before the sweep (findings 17, 34)
- [x] Batch 22: The window between the spawn and the record's write documented (findings 18, 24)
- [x] Batch 23: `write_new`'s leftovers swept (finding 19)
- [x] Batch 24: Doctor on a read-only inspection, without `pause` and `now` (finding 51)
- [x] Batch 25: up's unreachable GPU warning removed (finding 69)
- [x] Batch 26: The run store's formats 1 and 2 under `// LEGACY:` (finding 50)
- [x] Batch 27: The write errors of `logs` and of a streamed model log (findings 25, 26)
- [x] Batch 28: The grace between SIGTERM and SIGKILL (finding 27)
- [x] Batch 29: The clamps of `up`'s start wait (finding 28)
- [x] Batch 30: doctor's Gateway check: an unreadable record, a gateway without a model (findings 29, 47)
- [x] Batch 31: The bench's overhead address, `--scenario`'s mapping and the monitor's failure (findings 30, 31, 46)
- [x] Batch 32: `status`'s exit 0 with a running gateway (finding 32)
- [x] Batch 33: The adapters' failing commands, and a tag over a branch (findings 35, 36, 37, 44)
- [x] Batch 34: `up` on an address it cannot bind, `down` with a signal it cannot send (findings 38, 41)
- [x] Batch 35: `build use` under the builds' lock, and the current llama-swap reinstalled (findings 40, 45)
- [x] Batch 36: Paths that are not UTF-8 (findings 42, 43)
- [x] Batch 37: The compare table's change from zero (finding 48)
- [x] Batch 38: Error answers tested on captures of their own request (findings 71, 81)
- [x] Batch 39: The toolkit package name formatted once (finding 53)
- [x] Batch 40: rpm queried by exact names (finding 83)
- [x] Batch 42: The session scenario's bare prompt counted once (finding 85)
- [x] Batch 43: `down`'s poll growing from 5 ms to a deadline (finding 87)
- [x] Batch 44: An installed release refused before the fetch (finding 88)
- [x] Batch 45: `prune`'s own busy text on the gateway's lock (finding 64)
- [x] Batch 46: `Bench::setup` on one preparation (finding 55, 70, 86)
- [x] Batch 47: The failure switches named `<method>_fails` (finding 73)
- [x] Batch 48: The rule of the program as a field narrowed (finding 74)
- [x] Batch 49: `files::read_if_present` (finding 52)
- [x] Batch 50: One lenient read of a `current` link (finding 54)
- [x] Batch 51: `find_nvcc` through `SystemHost::find` (finding 56)
- [x] Batch 52: Doctor's Idle VRAM through `lifecycle::mib` (finding 57)
- [x] Batch 53: One test helper for an executable script (finding 58)
- [x] Batch 54: `gateway_process`'s fixtures embedded (finding 59)
- [x] Batch 55: `files::mode` in the three tests (finding 60)
- [x] Batch 56: `docs/commands.md` and the README say what the commands do (findings 65, 66, 67, 68)
- [x] Batch 57: `docs/architecture.md`'s statements match the code (findings 72, 75)
- [x] Batch 58: The doc comments match the code (findings 76, 77, 89)
- [x] Batch 59: README's Status (finding 78)

### Security Pass
- [x] Run `review-security` and its `review-verifier` on the fixes' diff, after the user's go-ahead. Its report: `docs/reviews/delivered-code/2026-10-19-v2-review-fixes-security-pass.md`
- [x] Batch S1: One mapping of `listen` to the sockets llama-swap opens, for `listens` and `is_free` (finding 1)
- [x] Batch S2: `up`'s timeout names another program that answers (finding 4)
- [x] Batch S3: `docs/security.md` says what `0.0.0.0` opens (finding 5)
- [x] Batch S4: Display escapes Unicode's format characters (finding 2)
- [x] Batch S5: A replaced build put back only before its replacement (finding 3)

---

## Technical Details

### Files to Modify
```
docs/reviews/delivered-code/
src/
tests/
```

### Dependencies
Phase 4 closed: the review starts from the roadmap's final code.

### Constraints
- The review runs the skill as it stands, with its gates. The user's go-ahead comes before each
  wave of agents.
- The phase's task count grows when the fix plan is approved, and the README's totals are
  recalculated.
- The fixes follow the same rules as phases 0 to 4: one batch per commit, test-first, and a
  test finding's test killing its mutant.
- Each batch's diff runs `cargo mutants --in-diff` before its commit; a mutant it misses is
  killed by a test in the batch or justified in the report.
- The roadmap closes on a security pass over the fixes' diff, not on a full review, which
  would call for another fix roadmap.

---

## Acceptance Criteria

- [x] The review's report is complete: every finding verified and given the user's verdict, and the fix plan approved
- [x] Every batch kept for this roadmap is fixed, and the others are listed as left open for the summary
- [x] The security pass over the fixes' diff is run and what it confirms is fixed
- [x] The five checks of CLAUDE.md are green
