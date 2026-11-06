# Review: the security pass over the fixes of v2-review-fixes

**Status:** complete — every finding verified and given its verdict, the fix plan approved; the fixes are the last task of `v2-review-fixes`' phase 5
**Started:** 2026-10-19

## Scope

| Fact | Value |
|---|---|
| Scope | range from `docs(v2-review-fixes): record the closing review's verdicts and the fix scope` of 2026-10-18 to `HEAD` |
| Start commits | — |
| Base | `docs(v2-review-fixes): record the closing review's verdicts and the fix scope` of 2026-10-18, the commit before batch 1 of the closing review's fix plan |
| End | `HEAD` when the pass started |
| Files | 56: production 36, tests 7, other 13, binary 0 |
| Lines | production 14,261, tests 14,720, other 7,589 |
| Uncommitted | none |

Set with `scripts/scope.py range <base> HEAD --files`, `<base>` the base above, exit 0; the diff is
`git diff <base> HEAD`, +5,550 and -1,005. It holds batches 1 to 59 of the fix plan of
`2026-10-17-v2-review-fixes.md`, one commit each, and the documents recorded with them; the last
commit adds a note and touches no code. Deleted in the diff:
`tests/fixtures/rpm-cuda-toolkit-packages.txt`.

The pass is the security pass that closes the `v2-review-fixes` roadmap (its phase 5,
Decisions): `review-security` alone on the fixes' diff, then one `review-verifier` on its
findings, after the user's go-ahead for each. The other axes are not reviewed: each batch ran
`cargo mutants --in-diff` before its commit, and the roadmap closes on no full review.

## Checks

| Command | Result |
|---|---|
| `cargo fmt --check` | green |
| `cargo clippy --all-targets -- -D warnings` | green |
| `cargo test` | green, 745 passed: 715 in the library, 27 in `tests/cli.rs`, 3 in `tests/gateway_config.rs`, and `up_a_proxied_request_and_down ... ok` in `tests/gateway_lifecycle.rs`; none skipped itself |
| `python3 .claude/skills/reviewing-delivered-code/evals/test_scope.py` | green |
| `python3 .claude/skills/reviewing-delivered-code/evals/test_fixtures.py` | green |
| `cargo audit` | green: 1,338 advisories loaded, 66 crate dependencies scanned, no vulnerability and no warning |
| `cargo mutants --in-diff` | exit 3: 284 mutants tested in 14 minutes, 204 caught, 71 unviable, 6 timeouts, 3 missed (cargo-mutants 27.1.0) |

**Missed mutants:** leads for the tests axis, which this pass does not review:

- `src/application/doctor.rs:428:58`: replace `>` with `>=` in `Doctor<'_>::inventory_checks`
- `src/infrastructure/gateway_process.rs:285:9`: replace `<impl ProcessTable for Procfs>::in_container -> Result<bool>` with `Ok(false)`, the one batch 49 justified
- `src/infrastructure/nvml.rs:72:5`: replace `read_gpu -> Result<Option<Gpu>, NvmlError>` with `Ok(None)`

The 6 timeouts, which `cargo mutants` counts as detected: `src/application/lifecycle.rs:453:60` and
`:462:20` in `copy_log`, `:618:9` `time_left`, `:647:30` in `stop`; `src/infrastructure/http.rs:53:9`
`Connection::read` as `Ok(1)`, `:88:5` `send` as `Ok(None)`.

**Mutants the agents wrote:** none: `cargo mutants` ran on the scope's diff.

## Tier

**Tier:** `high`, security alone — **Estimate:** about $3 to $5 for the reviewer, about $2 for
the verifier

| Pass | Agent | Axes | Effort |
|---|---|---|---|
| 1 | `review-security` | security | high |
| 1 | `review-verifier` | findings 1 to 3 | high |
| 1 | `review-verifier` | findings 4 and 5, the first verifier's leads | high |

## Findings

### 1. The finding 3 fix never recognises llama-swap's socket on 0.0.0.0, so `up` always stops an exposed gateway listening on 0.0.0.0

- **Axes:** security, bugs, error handling — raised by `review-security`; the v4-mapped case and
  `is_free`'s half by the two `review-verifier`
- **Action:** fix
- **Severity:** low
- **Where:** `src/infrastructure/gateway_process.rs:320-326`, `src/infrastructure/gateway_process.rs:310-316`,
  `src/application/lifecycle.rs:278-281`, `src/application/lifecycle.rs:566-569`,
  `src/application/lifecycle.rs:590-596`
- **Cause:** `TcpPorts::listens` picks the kernel's table by the address family of `listen`:
  `/proc/net/tcp` for IPv4, `/proc/net/tcp6` otherwise. llama-swap is a Go program, and Go
  listens on a wildcard IPv4 address such as `0.0.0.0` through one dual-stack AF_INET6 socket,
  `[::]:port`, which the kernel lists in `/proc/net/tcp6` only. The reverse holds for a
  v4-mapped address, `[::ffff:a.b.c.d]:port`: Go binds it as AF_INET, listed in
  `/proc/net/tcp`, where ai-ctl does not look; but `[::ffff:0.0.0.0]:port`, the wildcard
  written v4-mapped, is again one dual-stack socket in `/proc/net/tcp6`. In each case llama-swap
  answers `/health`, and `listens` says false on every poll. `TcpPorts::is_free` has the same
  cause on the other side: for `0.0.0.0:port` it binds AF_INET only, where llama-swap binds the
  dual-stack `[::]:port`, so a program on `[::1]:port`, or on `[::]:port` with `IPV6_V6ONLY`,
  passes the check.
- **Consequence:** no attacker is involved, and it fails closed. A user who sets `expose = true`
  with `listen = "0.0.0.0:8080"`, the example of `docs/security.md` §4, or with a v4-mapped
  address, which the inventory accepts only with `expose`, can never start the gateway: after
  30 s, `up` stops it with "llama-swap did not answer on 0.0.0.0:8080 within 30 s", which blames
  llama-swap for an answer it gave (finding 4). The workaround, `[::]:port`, opens nothing
  more than `0.0.0.0` does (Verification). On `is_free`'s side, a program already on
  `[::1]:8080` lets `up` start llama-swap on `0.0.0.0:8080`, which fails to bind and exits:
  `up` says "llama-swap exited (exit status: 1) before answering on 0.0.0.0:8080" with the
  log's tail, without batch 2's advice on the key, though a client that reaches `localhost`
  through `::1` would have reached that program.
- **Fix:** keep the parser `listening_inodes`, which matches `[::]` already. Map `listen` to
  the sockets Go opens for it, in one small pure function that `listens` and `is_free` share:
  canonicalise first (`ip.to_canonical()`), then an unspecified IPv4 is the dual-stack `[::]:p`
  or, on a kernel without IPv6, `0.0.0.0:p`, so `listens` reads both tables and `is_free`
  binds `[::]:p` dual-stack as Go does; any other IPv4 keeps `/proc/net/tcp`, any other IPv6
  `/proc/net/tcp6`. The socket must still be one of the started PID's descriptors, so the check
  does not weaken. Tests: the mapping, as a unit test, `[::ffff:0.0.0.0]:p` and
  `[::ffff:127.0.0.1]:p` among its cases; `listening_inodes` on a capture of llama-swap v263
  listening on `0.0.0.0`, the first verifier's `capture-tcp6-0.0.0.0.txt`, whose uid reads 0
  under the namespace's mapping, which its row in `tests/fixtures/README.md` says; `is_free` of
  `0.0.0.0:p` false while the test holds `[::1]:p`.
- **Evidence:** reproduced by the reviewer: llama-swap v263 alone in a network namespace
  (`unshare -rn`, only `lo`), a model whose command is `touch`. For `-listen 0.0.0.0:18990`,
  `/proc/net/tcp` has no LISTEN row and `/proc/net/tcp6` has
  `00000000000000000000000000000000:4A2E`, inode 1145630, llama-swap's fd 7. A probe test in a
  copy of the repository called `TcpPorts.listens` on that llama-swap: `127.0.0.1` true,
  `0.0.0.0` false, `[::]` true, each with health 200.
- **Verification:** CONFIRMED — reproduced again apart, in the same kind of namespace: `0.0.0.0`
  false with its one socket in `/proc/net/tcp6`, `[::]` and `127.0.0.1` true; a probe test
  with `held_by_another` gave `up`'s "did not answer … within 30 s; stopped it". The verifier
  added the v4-mapped case, `[::ffff:127.0.0.1]:19202` false, its row `0100007F:4A9E` in
  `/proc/net/tcp`, a lead of the bugs axis with the same cause, merged here; and corrected the
  consequence: llama-swap on `0.0.0.0` already answers on `[::1]`, so `[::]` opens no more.
  The second verifier, on finding 4, corrected the fix: a v4-mapped wildcard,
  `[::ffff:0.0.0.0]:19700`, is one socket in `/proc/net/tcp6`, so the mapping canonicalises
  first; a trial mapping in a copy gave `listens` true with health 200 for `[::ffff:0.0.0.0]`,
  `0.0.0.0`, `[::ffff:127.0.0.1]`, `[::]`, `127.0.0.1` and `[::1]`, and the whole suite passed.
  It also reproduced `is_free`'s half, a lead it left: a program on `[::1]:p` or a
  `V6ONLY` `[::]:p` passed `is_free` for `0.0.0.0:p`, and llama-swap exited on "bind: address
  already in use". Merged here by the session, its cause being this finding's, and not given
  to a verifier of its own.
- **User's verdict:** confirmed, by the session under the user's delegation

### 2. GPU process names from other users still reach the terminal with Unicode bidi and invisible formatting characters

- **Axes:** security — raised by `review-security`
- **Action:** fix
- **Severity:** low
- **Where:** `src/domain/text.rs:6-8`, `src/infrastructure/nvml.rs:274-279`,
  `src/application/doctor.rs:653-666`
- **Cause:** the finding 5 fix of the closing review sends NVML's process name through
  `escape_controls`, which escapes `char::is_control()`, C0, DEL and C1, and U+2028 and U+2029
  only. Unicode's format characters pass: U+061C, U+200B to U+200F, U+202A to U+202E, U+2060 to
  U+2069 and U+FEFF. The name is chosen by whoever runs the GPU process, other users included,
  and reaches doctor's Idle VRAM detail through `named()`, and `up`'s and `down`'s errors;
  presentation prints it as it is. The comment at `nvml.rs:274-276` says the name is "made safe
  for a terminal".
- **Consequence:** another local account runs a process on the GPU whose name holds U+202E or
  an isolate (the GPU's device nodes are 0666, `/proc` has no `hidepid`). A terminal that
  applies the bidirectional algorithm, as VTE does, shows the rest of the line reordered: the
  PIDs, the MiB and the following processes in a misleading order. These characters cannot move
  the cursor or rewrite other lines, so the gain is one misleading line. This machine has one
  login account.
- **Fix:** keep `is_control_or_separator` as the structural rule that `Text::checked` and
  `toml_escape` share, and give display a wider predicate of its own, used by `escape_controls`
  alone: that rule, or a character of a fixed list, U+061C, U+200B to U+200F, U+202A to U+202E,
  U+2060 to U+2064, U+2066 to U+2069, U+FEFF (the standard library has no category lookup).
  Update `escape_controls`' doc and `nvml.rs:274-276`. Test U+202E and U+2066 through
  `escape_controls`, and keep the test's assertion that `is_control_or_separator` leaves U+202A,
  so that the split stays visible. Not printable ASCII only, which would escape legitimate
  names such as the test's `llama-server é`.
- **Evidence:** reproduced by the reviewer: a probe test ran `escape_controls` on a text holding
  each of U+202E, U+2066 to U+2069, U+200F, U+061C and U+200B, and got it back unchanged. That
  NVML gives the name's bytes as they are was shown by the closing review's verification of its
  finding 5.
- **Verification:** CONFIRMED, on the code — the module's own test asserts that U+202A passes;
  the path from `nvml.rs` to `presentation/doctor.rs:38` escapes nothing else. Not checked: a
  terminal's rendering, and NVML naming another user's process, which needs a second account.
  The verifier settled the rule's reach: llama-swap v263 loaded a configuration whose `cmd`
  held U+FEFF, U+202E, U+2066, U+200B or an emoji, answered 200 and ran the command with the
  exact bytes, so these characters break neither its YAML nor TOML; `Text::checked` and
  `toml_escape` guard the structure of files written from the user's own inventory, inside the
  user's trust boundary (`docs/security.md` §1), and keep their rule.
- **User's verdict:** confirmed, by the session under the user's delegation

### 3. `put_back_replaced` restores a replaced build without checking that its replacement finished

- **Axes:** bugs — raised by `review-security`, as a lead under its LEFT OUT
- **Action:** fix
- **Severity:** low
- **Where:** `src/infrastructure/build_store.rs:150-176`, `src/infrastructure/build_store.rs:281-301`
- **Cause:** `publish` removes the old build while it is hidden as `.replaced-<id>-<pid>`, after
  the new one is in place, rather than renaming it out of sight first as `remove` does; a kill
  during that removal leaves a half-deleted `.replaced-<id>-<pid>` beside a complete `<id>`.
  The next build's `put_back_replaced`, the closing review's batch 21, renames any
  `.replaced-<id>-<pid>` back to `<id>` when nothing is there, without checking that
  `publish`'s second rename had not happened, the one case its doc names.
- **Consequence:** after such a kill, `<id>` removed by `build prune --keep 0` or by hand, then
  any build, brings the old build back: with its binary gone, a readable build with the
  metadata of before the rebuild, that `build list` lists without a warning and `build use`
  accepts, doctor then giving "llama-server not found"; with its `build.json` gone, an
  unreadable build. Even a complete put-back is wrong: a build the user removed comes back. A
  plain `build prune` does not reach it, since the rebuilt `<id>` is the newest and `--keep N`
  of 1 or more keeps it. The window is the removal of a build's three files.
- **Fix:** in `put_back_replaced`, put `.replaced-<id>-<pid>` back only when
  `.staging-<id>-<pid>`, the same PID, still exists: the mark that `publish`'s second rename
  has not happened. Otherwise the sweep that follows removes it, as the comment at
  `build_store.rs:293` intends. Give `publish` the order of `remove`, a rename to
  `.removing-…` then the removal, through one helper with `remove`. Test first: the state above,
  `<id>` removed after a kill past the second rename, then the next build: no `<id>`, the
  leftover gone. The existing `a_build_an_interrupted_replacement_left_hidden_is_put_back`
  already creates the staging directory, and keeps passing.
- **Evidence:** read by the reviewer, not reproduced.
- **Verification:** CONFIRMED — a probe test in a copy: b1 and b2 installed, b2 current; b1
  staged again, `publish`'s two renames done, one file of `.replaced-b1-4242` deleted;
  `store.remove(b1)`; b3 installed. The entries were `b1`, `b2`, `b3` and `current`: with
  `llama-server` deleted, b1 came back readable with its old `built_at` and no binary; with
  `build.json` deleted, unreadable. On a scratch XDG state with that b1 made current by hand,
  `build list` exited 0 with no warning, and `doctor` gave `warn llama.cpp b11538:
  …/llama-server not found`. The verifier corrected the lead's trigger: `--keep 0` or a manual
  removal, not a plain `build prune`.
- **User's verdict:** confirmed, by the session under the user's delegation

### 5. `docs/security.md` says `expose` opens the gateway to the LAN, where `0.0.0.0` also serves every IPv6 address

- **Axes:** security — raised by `review-verifier`, a lead while verifying finding 1
- **Action:** fix
- **Severity:** low
- **Where:** `docs/security.md:22`, `docs/security.md:131-140`, `docs/security.md:181-183`
- **Cause:** llama-swap, a Go program, listens on `0.0.0.0` or `[::]` through one dual-stack
  socket, `*:port` in `ss -ltn`, which serves every IPv4 and every IPv6 address of the
  machine. `docs/security.md` frames `expose` as reach of the LAN: §1's row "The LAN", §4's
  bullets, which never name IPv6. §6 says podman's `-p 18000:8080` publishes on
  `0.0.0.0:18000`, where rootless podman 5.8.7 with pasta listens on `*:18000`, IPv6 included.
- **Consequence:** a reader who opens a firewalld port "for the LAN" opens it to both families,
  firewalld's ports applying to IPv4 and IPv6 alike, and a global IPv6 address may be reachable
  from beyond the LAN where the router lets IPv6 in. This machine has one, on its Wi-Fi
  interface, and `net.ipv6.bindv6only = 0`. The key still guards every route but `/health` and
  the static files (§2), the firewall blocks by default, and `expose` is opt-in.
- **Fix:** the documents alone. §4: `0.0.0.0` and `[::]` open the same, one dual-stack socket
  serving every IPv4 and IPv6 address, global IPv6 ones included, checked with llama-swap v263;
  a firewalld port opens both families; to serve one network only, give that interface's own
  address. §1's row: the network, the LAN and over IPv6 whoever routes to the machine's global
  address. §6: podman's `-p 18000:8080` publishes on every IPv4 and IPv6 address, `*:18000` in
  `ss -ltn`. `docs/commands.md`, the README and `deploy/inventory.toml` speak of "an address
  other machines reach", which stays true.
- **Evidence:** the first verifier: in a network namespace, llama-swap v263 on
  `-listen 0.0.0.0:19400` answered 200 on `[::1]`.
- **Verification:** CONFIRMED — the second verifier, in a network namespace with a dummy
  interface holding `192.0.2.10/24` and `2001:db8::10/64`: `-listen 0.0.0.0:19900` and
  `[::]:19902` showed `*:port` and answered 200 on `127.0.0.1`, `192.0.2.10`, `[::1]` and
  `[2001:db8::10]`; `-listen 192.0.2.10:19901` answered on that address alone. On this machine,
  read only: a global `2a02:8440:…` address on `wlp130s0f0`, `bindv6only` 0, and the
  `FedoraWorkstation` zone with `dhcpv6-client` and no port, as §4 says. podman 5.8.7, a
  container `-p 19950:80` without ai-ctl's prefix, showed `*:19950`; the verifier's lead, merged
  here. More than a gap of the documents: §1 understates who is reached, on the very decision
  `expose` asks for.
- **User's verdict:** confirmed, by the session under the user's delegation

### 4. At the start wait's deadline, `up` blames llama-swap when another program answered

- **Axes:** error handling — raised by `review-verifier`, a lead while verifying finding 1
- **Action:** fix
- **Severity:** low, the verifier's, though the axis's format asks none: it fails closed, with
  the wrong blame
- **Where:** `src/application/lifecycle.rs:590-596`, `src/application/lifecycle.rs:549-552`
- **Cause:** `wait_until_healthy` sets `another` when a 200 comes from a socket the started
  llama-swap does not hold, and reads it in the branch where llama-swap exits only. At the
  deadline, with llama-swap still running, the message is built from `last_error` alone:
  "llama-swap did not answer on <listen> within 30 s".
- **Consequence:** the user is told that llama-swap did not answer when an answer came, from
  another program, and is not given batch 2's advice on the key. With finding 1's fix, a
  squatter cannot hold that answer while llama-swap stays alive: every bind conflict the
  verifier tried made llama-swap exit, the branch that says it; the state remains reachable
  through a mapping that misses a socket, such as finding 1's, which the clause would then
  misstate.
- **Fix:** build the clause ", where another program answers: {ANOTHER_PROGRAM}" once in
  `wait_until_healthy`, a small function of `another`, and use it in the exited message and
  after the timeout's `{cause}`. Test first: `a_start_timeout_names_another_program_that_answers`,
  `FakePorts` `held_by_another` and a gateway that never exits. A commit of its own, after
  finding 1's: until that fix, the clause would call llama-swap's own answer on `0.0.0.0`
  another program's.
- **Evidence:** the first verifier: a probe test with `held_by_another = true` gave "llama-swap
  did not answer … within 30 s; stopped it".
- **Verification:** CONFIRMED — the second verifier's test failed on its assertion, then passed
  with the shared clause, the 83 lifecycle tests and clippy green. Its probes with llama-swap
  v263 in a network namespace and a squatter: `SO_REUSEPORT` on `127.0.0.1` or `0.0.0.0`, the
  same port on another address of the same family, and the families crossed, each gave
  `is_free` false and a refused `up`, or llama-swap exiting on its bind, or each program
  answering on its own address; v263 binds within about 1.5 s.
- **User's verdict:** confirmed, by the session under the user's delegation

## Coverage

| Axis | Reviewed by | Left out, and why |
|---|---|---|
| Architecture | — | not reviewed: a security pass |
| Dead and legacy code | — | not reviewed: a security pass |
| Refactoring | — | not reviewed: a security pass |
| Conventions | — | not reviewed: a security pass |
| Bugs | — | not reviewed: a security pass; findings 1 and 3, the reviewer's lead, verified on that axis |
| Error handling | — | not reviewed: a security pass; finding 4, a verifier's lead, verified on that axis |
| Security | `review-security`, high, and two `review-verifier`, high | read in full: `docs/security.md`, `src/domain/text.rs`, `src/domain/inventory.rs`, `src/infrastructure/gateway_process.rs`, `src/infrastructure/files.rs`; the others in part or by their diff (below). Left out by the reviewer: `src/presentation/bench.rs`, `compare.rs` and `build.rs`, `src/domain/bench.rs` and most of `src/infrastructure/run_store.rs`, which print or store the user's own runs and builds, with no sink fed from outside the account; `src/application/fakes.rs`, `tests/cli.rs` and the captures, not run in production; the backend's squat, closing finding 1, recorded as residual risk in `docs/security.md` §2 |
| Tests | — | not reviewed: a security pass; each batch ran `cargo mutants --in-diff` |
| Performance | — | not reviewed: a security pass |
| On-disk formats | — | not reviewed: a security pass |
| Interruption and concurrency | — | not reviewed: a security pass |
| The CLI's contract | — | not reviewed: a security pass |

The verifier of findings 1 to 3, 43 tool calls in about 8 minutes, read
`src/infrastructure/gateway_process.rs` (250-443, 630-720), `src/application/lifecycle.rs`
(520-620, 939-953, 1355-1420), `src/application/fakes.rs` (755-790), `src/domain/text.rs`,
`src/infrastructure/nvml.rs` (71-92, 265-289), `src/application/doctor.rs` (365-400, 640-666),
`src/domain/gateway_config.rs` (396-406), `src/infrastructure/inventory_file.rs` (85-96),
`src/infrastructure/build_store.rs` (its production code, tests 445-660),
`src/infrastructure/files.rs` (24-31, 211-231), `src/application/build.rs` (96-215, 317-345,
395-519), `src/domain/build.rs` (133-147), the two `proc-net-tcp*` captures, and
`docs/security.md` (1-80, 125-145). It left out the scope's other files, which no finding
names, a terminal's bidirectional rendering, with no display, and an NVML read of another
user's process, with one account. It ran `build list` and `doctor` on a scratch XDG state, and
llama-swap in a network namespace; nothing was left (`pgrep`). Its captures, probes and the
diff of its copy are kept outside the repository, in
`~/.local/share/ai-ctl/review-scratch/2026-10-19-v2-review-fixes-security-pass/verifier/`.

`review-security`, in full: `docs/security.md`, `src/domain/text.rs`, `src/domain/inventory.rs`,
`src/infrastructure/gateway_process.rs`, `src/infrastructure/files.rs`. In part:
`src/application/lifecycle.rs` (1-990), `src/domain/gateway_config.rs` (1-560),
`src/application/build.rs` (1-600), `src/infrastructure/build_store.rs` (1-360),
`src/infrastructure/git.rs` (1-225), `src/infrastructure/server_api.rs` (1-300),
`src/infrastructure/gateway_api.rs` (1-150), `src/infrastructure/chat_api.rs` (1-80),
`src/infrastructure/api_key.rs` (1-100), `src/infrastructure/run_store.rs` (55-100),
`src/application/bench.rs` (295-312, and each use of the listen address),
`src/application/doctor.rs` (650-668, and its diff), `src/infrastructure/http.rs` (176-205,
and its diff), `src/domain/build.rs` (41-67), the closing review's report (182-511). By their
diff alone: `src/infrastructure/host.rs`, `nvml.rs`, `inventory_file.rs`, `gateway_store.rs`,
`cmake.rs`, `command.rs`, `podman.rs`; `src/domain/machine.rs`, `gateway.rs`;
`src/application/gateway.rs`, `gateway_config.rs`, `toolchain.rs`, `ports.rs`;
`src/presentation/mod.rs`.

The verifier of findings 4 and 5, 33 tool calls in about 7 minutes, read
`src/application/lifecycle.rs` (40-59, 240-320, 480-640, 1340-1420), `src/application/fakes.rs`
(760-790), `src/infrastructure/gateway_process.rs` (170-215, 260-399),
`src/infrastructure/gateway_api.rs` (1-130), `src/infrastructure/http.rs` (110-142),
`src/domain/inventory.rs` (270-310), `docs/security.md` (1-200), `README.md` (250-262),
`deploy/inventory.toml` (18-30), and searched `docs/commands.md`, `docs/stack.md`,
`docs/architecture.md`, `docs/notes/` and `docs/decisions/` for `0.0.0.0`, IPv6 and
dual-stack. It ran llama-swap and a podman container in or beside a network namespace, read
the machine's addresses and firewall zone, and left nothing (`pgrep`, `podman ps -a`). Its
probes, kept with the first verifier's, are in
`~/.local/share/ai-ctl/review-scratch/2026-10-19-v2-review-fixes-security-pass/verifier-leads/`.

**Files of the scope no agent opened:** `src/application/mod.rs` and `src/domain/mod.rs`,
module lists; `src/domain/bench.rs`, `src/presentation/bench.rs`, `build.rs` and `compare.rs`,
left out by the reviewer as above; `tests/cli.rs` and the captures
`llama-swap-263-upstream-props-unknown-model.http`, `nvidia-ctk-1.20.1-cdi-spec.yaml` and
`rpm-q-cuda-toolkit-packages.txt`, not run in production; of the documents, the notes, the
roadmap's files and `docs/stack.md`, outside the security axis but for the searches above.

## Fix Plan

Approved under the user's delegation of 2026-10-18: the session gives the verdicts and the plan,
the user the hard cases (`v2-review-fixes`' phase 5 report, Decisions). Each batch is one commit,
test-first, with `cargo mutants --in-diff` on its diff.

1. **One mapping of `listen` to the sockets llama-swap opens** — finding 1: a pure function,
   `listen` canonicalised, then an unspecified IPv4 as the dual-stack `[::]` or `0.0.0.0`;
   `listens` reads the tables it names and `is_free` binds as Go does; the parser tested on the
   verifier's capture of `0.0.0.0`, listed in `tests/fixtures/README.md`.
2. **`up`'s timeout names another program that answers** — finding 4: the clause shared by the
   exited and the timeout messages; after batch 1.
3. **`docs/security.md` says what `0.0.0.0` opens** — finding 5: §1's row, §4's bullet on the
   dual-stack socket and the firewall, §6's podman publishing.
4. **Display escapes Unicode's format characters** — finding 2: a predicate of its own for
   `escape_controls`, a fixed list beside `is_control_or_separator`, which `Text::checked` and
   `toml_escape` keep.
5. **A replaced build put back only before its replacement** — finding 3: `put_back_replaced`
   only beside the same PID's `.staging-<id>-<pid>`, and `publish` removing the old build
   through `remove`'s order, a rename to `.removing-…` first.
