# Phase 0: Security

---

## Status

**Current Status:** 🟢 Done (100% — 6/6)
**Started:** 2026-10-16
**Completed:** 2026-10-16
**Blocked By:** —

---

## While Working

Keep `phase-0-security-report.md` current as the work happens — after each significant
step, and before every commit, pause, or end of session.

---

## Objective

The plan's five security batches are fixed:
- the gateway answers only a client that holds ai-ctl's API key, and listens on loopback
  unless the inventory opts in;
- no inventory text can end the configuration's `cmd` block;
- the check of a non-pinned llama-swap release says what it proves;
- doctor's download remedy pins a revision;
- image backends can be pinned by digest.

---

## Overview

### Why This Phase Matters
Security is the review's most important axis and the first section of its plan. Finding 1 is
the review's most serious finding. Any local process, or any web page open in the user's
browser, can load or evict a model, read the models' output in the log stream and use the GPU.
Finding 7 opens the same door to the LAN with one edited line. Finding 3 lets an inventory
someone else wrote make llama-swap run a command on the host.

### What It Enables
The gateway is closed to callers without the key, and the later phases change code whose
inputs are trusted. Batch 43's HTTP helpers in phase 4 build on the `Authorization` header that
this phase adds to the three HTTP adapters.

### Out of Scope
- llama-swap's `allowedOrigins`, which finding 1 leaves unprobed. Without the key, a page gets
  a 401 anyway.
- Downloading or verifying models in ai-ctl. `model_check` stays as it is, per the decision of
  2026-10-10 recorded in `docs/stack.md`.
- Checking a pinned image digest before `up`: for comparison benches, a warning on a bare tag
  is enough (finding 5).

---

## Tasks

### Decision
- [x] Run the installed llama-swap v263 alone on a loopback port with `apiKeys` set, and probe whether it forwards the client's `Authorization` header to an upstream. Then decide with the user whether the key also guards the llama-server backends (`--api-key`) or only the gateway, and how a client reads the key. Record the decision in the report and in `docs/stack.md` §7

### Gateway Access
- [x] Batch 1 (findings 1, 7):
  - ai-ctl creates one API key once, keeps it in its state across `up` runs, writes it as llama-swap's `apiKeys`, and `gateway_api`, `server_api` and `chat_api` send it as `Authorization: Bearer`; `/health` stays open;
  - a non-loopback `gateway.listen` is refused unless the inventory opts in, and the opt-in is tied to the key;
  - `deploy/opencode.json` and `docs/stack.md` §7 give clients the key

### Inventory Text
- [x] Batch 2 (finding 3): `Text::checked` refuses U+2028 and U+2029 beside `char::is_control`. A test with U+2028 in a flag value is watched failing first

### Downloads
- [x] Batch 3 (finding 4): for a non-pinned release, `gateway install --version` says that the release's own checksums detect corruption only, instead of "SHA-256 verified" and "tampered with". An optional `--sha256` checks the archive against a digest the user gives
- [x] Batch 4 (finding 6): doctor's remedy for a missing model adds `--revision <commit>` to `hf download` and points to "Where Models Come From" in `docs/stack.md` for the hash and template checks. `model_check` is unchanged
- [x] Batch 5 (finding 5): image references with `@sha256:` are accepted, the shipped inventory pins its images by digest, and a bare tag gets a warning

---

## Technical Details

### Files to Modify
```
src/domain/gateway_config.rs
src/infrastructure/inventory_file.rs
src/infrastructure/paths.rs
src/infrastructure/http.rs
src/infrastructure/gateway_api.rs
src/infrastructure/server_api.rs
src/infrastructure/chat_api.rs
src/infrastructure/gateway_process.rs
src/infrastructure/podman.rs
src/application/gateway.rs
src/application/gateway_config.rs
src/application/lifecycle.rs
src/application/doctor.rs
src/presentation/cli.rs
src/presentation/gateway.rs
tests/snapshots/
deploy/inventory.toml
deploy/opencode.json
docs/stack.md
README.md
```

### Dependencies
None from an earlier phase. The probe of the first task uses the installed llama-swap v263,
as finding 1's verification did.

### Constraints
- Each batch applies its findings' Fix as verified, the verifiers' corrections included, with
  the tests those findings name. A test is watched failing on its assertion before the fix.
- Only the user can read the key: the file that keeps it and the generated configuration that
  carries it are not readable by other users.
- The snapshot tests of the generated configuration are updated for `apiKeys`, not bypassed.

---

## Acceptance Criteria

- [x] Against the gateway started by `up`, `/v1/models`, `/running`, `/logs` and `POST /api/models/unload` answer 401 without the key, whatever the origin, and `/health` answers 200. The check repeats finding 1's verification
- [x] `up`, `status`, `logs` and `down` work with the key on this machine, and OpenCode and OpenWebUI work through the gateway
- [x] A non-loopback `listen` without the opt-in is refused, with a message that names the opt-in
- [x] With finding 3's flag, the inventory is refused: the reproduction adds no model
- [x] The five checks of CLAUDE.md are green
