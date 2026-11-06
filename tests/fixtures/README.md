# Test Fixtures

Outputs captured on this machine, for the parsers and adapters to be tested on what the tools
really print (CLAUDE.md, Tests). None is written from memory. Commands ran with `LC_ALL=C`, as ai-ctl
runs them: `gcc` and the compilers answer in French otherwise. An `.http` file is a whole HTTP
response, status line and headers included, as `infrastructure::http::parse` reads it.

"Captured from" gives the command or request. Where the report recorded no exact command line, it
is the one the adapter runs. Each section names the report that records the capture. An excerpt keeps the capture's order and spelling, and says what it leaves out.

When a tool's output changes with a new version, capture it again and update its row.

## Host and Toolchain

Captured on 2026-10-09 for `doctor`'s probes, read by `src/infrastructure/host.rs`. Report:
`docs/roadmap/completed/v2-rebuild/phase-1-doctor-and-inventory-report.md`, or
`docs/roadmap/completed/v2-rebuild/phase-2-native-build-report.md` where marked (2), or
`docs/roadmap/completed/v2-review-fixes/phase-5-closing-review-report.md` where marked (5).

| Fixture | Captured from |
|---|---|
| `busctl-active-profile.txt` | `busctl get-property org.freedesktop.UPower.PowerProfiles /org/freedesktop/UPower/PowerProfiles org.freedesktop.UPower.PowerProfiles ActiveProfile` |
| `busctl-not-activatable.txt` | The same request for a name no daemon serves: busctl's error on stderr |
| `cmake-version.txt` | `cmake --version`, CMake 4.3.0 |
| `git-version.txt` | `git --version`, git 2.56.0 |
| `podman-version.txt` | `podman --version`, podman 5.8.7 |
| `nvidia-ctk-version.txt` | `nvidia-ctk --version`, toolkit 1.20.1 |
| `nvidia-ctk-cdi-list.txt` | `nvidia-ctk cdi list`; the GPU's UUID replaced with zeros |
| `gcc-15-dumpfullversion.txt` | `g++-15 -dumpfullversion`, GCC 15.3.1 |
| `nvcc-13.2.txt` | `nvcc --version`, CUDA 13.2.86, since removed |
| `host_config-13.2-excerpt.h` | Excerpt of CUDA 13.2's `targets/x86_64-linux/include/crt/host_config.h`: the compiler checks, `#if __GNUC__ > 15` among them |
| `os-release-fedora44.txt` | `/etc/os-release`, Fedora 44 Workstation (2) |
| `dnf-repolist-enabled.txt` | `dnf repolist --enabled` (2) |
| `dnf-repoquery-cuda-driver-packages.txt` | `dnf repoquery --cacheonly --quiet --repo <cuda repository> --qf '%{name}\n'` with ai-ctl's driver patterns, on a repository that offers driver packages (2) |
| `rpm-q-cuda-toolkit-packages.txt` | `rpm -q --nodigest --nosignature --qf '%{NAME}\n' cuda-toolkit cuda-toolkit-13 cuda-toolkit-13-2 cuda-toolkit-13-4`, rpm 6.0.2, on 2026-10-18: one line per name, three of them not installed, exit status 3, nothing on stderr (5) |

## Native Build

Captured on 2026-10-09 while building llama.cpp `b11538`. Report:
`docs/roadmap/completed/v2-rebuild/phase-2-native-build-report.md`, or
`docs/roadmap/completed/v2-review-fixes/phase-2-dead-code-and-tests-report.md` where marked (3).

| Fixture | Captured from | Read by |
|---|---|---|
| `ninja-build-excerpt.txt` | Excerpt of the Ninja build of `b11538`: steps 1–3, 178–179 and 354–356 of 356 | `src/infrastructure/cmake.rs` |
| `make-build-excerpt.txt` | The whole Makefile-generator build of a three-file project, cmake 4.3.0 and GNU Make 4.4.1, on 2026-10-16: `cmake -S proj -B build -G "Unix Makefiles"`, then `cmake --build build --target app --parallel 2` (3) | `src/infrastructure/cmake.rs` |
| `llama-server-version.txt` | `llama-server --version` of the build; it says `build 1`, from a shallow clone at the time | `src/application/fakes.rs` |
| `llama-server-list-devices.txt` | `llama-server --list-devices` of the build | `src/domain/recipe.rs`, `src/application/fakes.rs` |

## Build Sources

Captured on 2026-11-05 with git 2.56.0, for the fixes of `audio-image-backends`' review. Report:
`phase-6-fixes-and-closing-pass-report.md` of the `review-skill-v2` roadmap.

| Fixture | Captured from | Read by |
|---|---|---|
| `git-config-null-submodule-paths.gitmodules` | A `.gitmodules` written for the capture: `ggml`, a name holding a space, a name that climbs out with `..` and an absolute one | — |
| `git-config-null-submodule-paths.txt` | `git config --null --file .gitmodules --get-regexp '^submodule\..*\.path$'` on it: entries `<key>\n<value>\0` | `src/infrastructure/git.rs` |

## Audio and Image Servers

Captured on 2026-10-20 from hand builds for sm_120a of whisper.cpp `v1.9.5`, stable-diffusion.cpp
`master-975-95b3090` and audio.cpp `v0.9.2`, nvcc 13.4.92 and g++ 16.2.1. Each tag list is
`git tag --list` in a clone made as the builder makes it, the day of the capture. Where a server
prints on both streams, the fixture holds them as `2>&1` interleaves them. The tag lists, the two
device listings, `whisper-server`'s help and the two versions are read by `src/domain/recipe.rs`,
`whisper-server`'s help also by `src/application/fakes.rs`, and the two versions also by
`src/application/build.rs`; no code reads the two other helps, `sd-server`'s and
`audiocpp_server`'s, yet.
Report: phase 0's, `phase-0-probes-and-decisions-report.md`, of the `audio-image-backends` roadmap.

| Fixture | Captured from |
|---|---|
| `whisper.cpp-git-tag-list.txt` | `git tag --list` of `https://github.com/ggml-org/whisper.cpp.git`: 95 tags, releases `vX.Y.Z` among `b<N>` and older forms |
| `whisper-server-v1.9.5-help.txt` | `whisper-server --help`, run as `build/whisper.cpp/bin/whisper-server`: all on stderr, exit 0. The server has no `--version` and no device listing |
| `stable-diffusion.cpp-git-tag-list.txt` | `git tag --list` of `https://github.com/leejet/stable-diffusion.cpp.git`: 747 tags, releases `master-<N>-<hash>` |
| `sd-server-master-975-help.txt` | `sd-server --help`: stdout, exit 0 |
| `sd-server-master-975-version.txt` | `sd-server --version`: stdout, exit 0 |
| `sd-server-master-975-list-devices.txt` | `sd-server --list-devices`, exit 0: ggml's CUDA banner on stderr, then one `name<TAB>description` line per device on stdout |
| `audio.cpp-git-tag-list.txt` | `git tag --list` of `https://github.com/0xShug0/audio.cpp.git`: 28 tags, releases `vX.Y.Z` beside suffixed ones |
| `audiocpp_server-v0.9.2-help.txt` | `audiocpp_server --help`: stdout, exit 0 |
| `audiocpp_server-v0.9.2-version.txt` | `audiocpp_server --version`: stdout, exit 0; it says `audio.cpp dev` on the release tag |
| `audiocpp_server-v0.9.2-list-devices.txt` | `audiocpp_server --list-devices`, exit 0: ggml's CUDA banner on stderr, then `available_devices=2`, one `CUDA:0 "…" [GPU]`-style line per device and a hint, on stdout |

## llama-swap v263

Captured on 2026-10-10 and 2026-10-11 from llama-swap v263, on a scratch configuration or the
running gateway. Report: `docs/roadmap/completed/v2-rebuild/phase-3-gateway-report.md`, or
`docs/roadmap/completed/v2-rebuild/phase-4-bench-report.md` where marked (4), or, captured on
2026-10-18 by the closing review's verifier, finding 81 of
`docs/reviews/delivered-code/2026-10-17-v2-review-fixes.md` where marked (R). Read by
`src/infrastructure/gateway_api.rs` and `src/infrastructure/http.rs`, the chat files by
`src/infrastructure/chat_api.rs`, the upstream's 404 by `src/infrastructure/server_api.rs` too.

| Fixture | Captured from |
|---|---|
| `llama-swap-263-version.txt` | `llama-swap -version` |
| `llama-swap-263-checksums.txt` | `llama-swap_263_checksums.txt` of the v263 GitHub release |
| `llama-swap-263-health.http` | `GET /health` |
| `llama-swap-263-running-empty.http` | `GET /running`, no model loaded |
| `llama-swap-263-running.http` | `GET /running`, a model loaded |
| `llama-swap-263-unload.http` | `POST /api/models/unload` |
| `llama-swap-263-logs-stream.http` | `GET /logs/stream/<model>` over raw HTTP/1.0, a fake upstream's lines |
| `llama-swap-263-logs-stream-unknown.http` | `GET /logs/stream/nope`: the 400 for an unknown logger |
| `llama-swap-263-chat-stream.http` | `POST /v1/chat/completions` streamed through llama-swap, curl over HTTP/1.0, the 27B on `b11538` (4) |
| `llama-swap-263-chat-unknown-model.http` | The same request for a model llama-swap does not know: its 404 (4) |
| `llama-swap-263-chat-bad-request.http` | A request llama-server refuses, through llama-swap: its 400 (4) |
| `llama-swap-263-upstream-props-unknown-model.http` | `GET /upstream/nope/props` over raw HTTP/1.0 with the key, a model llama-swap does not know: its 404, the same as `POST /upstream/nope/tokenize` gives (R) |

## llama-server b11538

Captured on 2026-10-11 and 2026-10-12 from the 27B on the native `b11538`, through
`/upstream/qwen3.8-27b/`. Report: `docs/roadmap/completed/v2-rebuild/phase-4-bench-report.md`. Read by
`src/infrastructure/server_api.rs`, the stream by `src/infrastructure/chat_api.rs`.

| Fixture | Captured from |
|---|---|
| `llama-server-b11538-chat-stream.http` | `POST /v1/chat/completions` straight to llama-server: HTTP/1.1, chunked, whatever the request's version |
| `llama-server-b11538-props.http` | `GET /props` |
| `llama-server-b11538-apply-template.http` | `POST /apply-template` of a system message, `Session x.`, and a user message, `Hello` |
| `llama-server-b11538-tokenize.http` | `POST /tokenize` of `<\|im_start\|>user\nHello<\|im_end\|>`, special tokens parsed |
| `llama-server-b11538-tokenize-pieces.http` | `POST /tokenize` with `"with_pieces": true` of `fn main() { println!("héllo — ×"); }` |
| `llama-server-b11538-metrics.http` | `GET /metrics` after a bench run of one repetition, MTP drafting two tokens |

## Containers and CDI

The CDI specifications are read by `src/infrastructure/host.rs`, podman's outputs by
`src/infrastructure/podman.rs` and `src/infrastructure/command.rs`.

| Fixture | Captured from | Report |
|---|---|---|
| `nvidia-ctk-1.20.1-cdi-spec-excerpt.yaml` | Excerpt of `/var/run/cdi/nvidia.yaml`, driver 615.71.09, 2026-10-12: the device `all`, the env, the libcuda links and mount | `docs/roadmap/completed/v2-rebuild/phase-4-bench-report.md` |
| `nvidia-ctk-1.20.1-cdi-spec.yaml` | `/var/run/cdi/nvidia.yaml`, whole, written by `nvidia-cdi-refresh.service` at 15:01 on 2026-10-18, driver 615.78.08: 60 `hostPath` entries; the GPU's UUID replaced with zeros | `docs/roadmap/completed/v2-review-fixes/phase-5-closing-review-report.md` |
| `nvidia-ctk-1.20.1-cdi-spec-excerpt.json` | Excerpt of `nvidia-ctk cdi generate --format json --quiet`, driver 615.78.08, 2026-10-16: the device `all`'s name, the env and the libcuda mount; the GPU's UUID left out | `docs/roadmap/completed/v2-review-fixes/phase-2-dead-code-and-tests-report.md` |
| `podman-5.8.7-ps-bogus-storage-driver.txt` | `podman ps` under a `CONTAINERS_STORAGE_CONF` naming the driver `bogus`, 2026-10-16: stderr, scratch path included | `docs/roadmap/completed/v2-review-fixes/phase-1-bugs-and-error-handling-report.md` |
| `podman-5.8.7-image-inspect-digest.txt` | `podman image inspect --format '{{.Digest}}' ghcr.io/ggml-org/llama.cpp:server-cuda13-b11541`, 2026-10-16 | `docs/roadmap/completed/v2-review-fixes/phase-2-dead-code-and-tests-report.md` |
| `podman-5.8.7-ps-pid.txt` | `podman ps --format '{{.Pid}}'`, one container running, 2026-10-16 | `docs/roadmap/completed/v2-review-fixes/phase-2-dead-code-and-tests-report.md` |

## Processes

`/proc` files read by `src/infrastructure/gateway_process.rs`, captured on 2026-10-17, kernel
7.2.10, podman 5.8.9, while a container of `docker.io/library/node:22-bookworm-slim` ran `sleep`
(`podman run -d --rm --pull=never`). Report:
`docs/roadmap/completed/v2-review-fixes/phase-3-performance-report.md`.

| Fixture | Captured from |
|---|---|
| `proc-stat-conmon.txt` | `/proc/<pid>/stat` of the container's `conmon`, the parent of its `sleep` |
| `proc-stat-odd-name.txt` | `/proc/<pid>/stat` of `sleep` copied to a file named `x) R 1 (y` and run |
| `proc-cgroup-podman-container.txt` | `/proc/<pid>/cgroup` of the container's `sleep` |
| `proc-cgroup-gnome-shell.txt` | `/proc/<pid>/cgroup` of `gnome-shell`, which keeps a context on the dGPU |

The socket tables, captured on 2026-10-18, kernel 7.2.11, while a Python process listened on
`127.0.0.1:34545` and `[::1]:53305` and held one connection to the first, accepted. Each excerpt
keeps the header and the rows whose addresses name the listener's port, and leaves out the
machine's other sockets. Report:
`docs/roadmap/completed/v2-review-fixes/phase-5-closing-review-report.md`.

| Fixture | Captured from |
|---|---|
| `proc-net-tcp-excerpt.txt` | `/proc/net/tcp`: the listener (state `0A`, inode 5130254), the client's socket and the accepted one (state `01`) |
| `proc-net-tcp6-excerpt.txt` | `/proc/net/tcp6`: the listener on `[::1]` (state `0A`, inode 5130257) |

The dual-stack socket, captured on 2026-10-19, kernel 7.2.11, while llama-swap v263 ran with
`-listen 0.0.0.0:19200` alone in a network namespace (`unshare -rn`), whose user mapping shows
its uid as 0. Report: `docs/reviews/delivered-code/2026-10-19-v2-review-fixes-security-pass.md`,
finding 1.

| Fixture | Captured from |
|---|---|
| `proc-net-tcp6-dual-stack.txt` | `/proc/net/tcp6`: the header and llama-swap's one socket, `[::]:19200` (state `0A`, inode 1948879), which Go opens for `0.0.0.0` |

## Models

A model's file that ai-ctl reads, read by `src/domain/safetensors.rs` and by `doctor`'s tests in
`src/application/doctor.rs`. Report: phase 2's, `phase-2-inventory-and-gateway-report.md`, of
the `audio-image-backends` roadmap.

| Fixture | Captured from |
|---|---|
| `fibo-2.0-dev-72415bd0-diffusion_pytorch_model.safetensors.index.json` | `transformer/diffusion_pytorch_model.safetensors.index.json` of `briaai/Fibo-2.0-dev` at `72415bd0`, downloaded on 2026-10-29, its git blob SHA-1 equal to the Hub's (`e8669c03…`): 783 tensors in two shards |

## Test Runs

The stdout of `cargo test` run on one test by name, read by the `reviewing-delivered-code`
skill's `scripts/regression.py`, which reads each test's `test <name> ... <status>` line, and
tested by its `evals/test_regression.py`. Captured on 2026-11-02 on this repository at
`7822811`, cargo 1.97.1; cargo's `Finished` and `Running` lines go to stderr, left out. Report:
phase 2's, `phase-2-scripts-report.md`, of the `review-skill-v2` roadmap.

| Fixture | Captured from |
|---|---|
| `cargo-test-exact-one.txt` | `cargo test --locked -- --exact domain::gateway::tests::a_release_is_a_number_with_or_without_its_v`: the test passed in the library's binary, none run in the binary's, two of `tests/` and the doc-tests; `gateway_lifecycle`'s own harness prints nothing |
| `cargo-test-exact-none.txt` | `cargo test --locked -- --exact domain::gateway::tests::no_such_test`: no test run |
| `cargo-test-exact-own-harness.txt` | `cargo test --locked -- --exact up_a_proxied_request_and_down`: `gateway_lifecycle`'s harness, `harness = false`, prints its test's line and no `test result:` line |

## Dependencies

The outputs of cargo, crates.io and GitHub read by the `reviewing-delivered-code` skill's
`scripts/dependencies.py`, and tested by its `evals/test_dependencies.py`. Captured on 2026-11-02,
cargo 1.97.1. The probe crate, `facts`, made in scratch: `equivalent = "1.0"` locked at 1.0.1,
`heck = { version = "0.4", features = ["unicode"] }`, its `unicode-segmentation` locked at 1.13.1,
a yanked version, `heck5 = { package = "heck", version = "0.5" }`, `fnv = "1"` as a
dev-dependency, `itoa` from its git repository at the tag 1.0.18, and a path crate, `inner`,
outside its folder, using `static_assertions = "1"`; rust-version 1.89. The HTTP
answers are `curl -i`'s, with the User-Agent `ai-ctl-review-dependencies (reviewing-delivered-code
skill)`; one GitHub answer holds its redirect, the 301, then the 200. Report: phase 2's,
`phase-2-scripts-report.md`, of the `review-skill-v2` roadmap.

| Fixture | Captured from |
|---|---|
| `cargo-metadata-dependencies-probe.json` | `cargo metadata --locked --format-version 1` on the probe crate: 9 packages, 6 from crates.io, `heck` at 0.4.1 and 0.5.0, `itoa` from git |
| `cargo-update-dry-run-probe.txt` | `cargo update --dry-run --verbose`'s stderr on the probe crate: the git repository's line, `equivalent` and `unicode-segmentation` updated within their requirements, `heck` 0.4.1 unchanged, 0.5.0 available |
| `cargo-update-dry-run-unchanged.txt` | The same on a crate made in scratch, edition 2024, rust-version 1.85, with `thiserror = "1"`, `anyhow = "=1.0.80"`, `toml = "0.8"` and `clap = "4.0"` locked at 4.0.32: updates, crates unchanged with a version available, packages added and removed |
| `cargo-update-dry-run-rust-version.txt` | The same on a crate made in scratch, edition 2021, rust-version 1.70, resolver 3, with `clap = "4.0"` locked at 4.0.32: an update held back by the Rust its newest requires, and a locked version requiring a newer Rust |
| `cargo-update-dry-run-none.txt` | The same on this repository at `ba2457e`: nothing to update |
| `crates-io-equivalent.http` | `GET https://crates.io/api/v1/crates/equivalent` |
| `crates-io-equivalent-owners.http` | `GET https://crates.io/api/v1/crates/equivalent/owners` |
| `crates-io-equivalent-reverse-dependencies.http` | `GET https://crates.io/api/v1/crates/equivalent/reverse_dependencies?per_page=1` |
| `crates-io-fnv.http` | `GET https://crates.io/api/v1/crates/fnv` |
| `crates-io-fnv-owners.http` | `GET https://crates.io/api/v1/crates/fnv/owners`: a user and a team |
| `crates-io-fnv-reverse-dependencies.http` | `GET https://crates.io/api/v1/crates/fnv/reverse_dependencies?per_page=1` |
| `crates-io-heck.http` | `GET https://crates.io/api/v1/crates/heck` |
| `crates-io-heck-owners.http` | `GET https://crates.io/api/v1/crates/heck/owners` |
| `crates-io-heck-reverse-dependencies.http` | `GET https://crates.io/api/v1/crates/heck/reverse_dependencies?per_page=1` |
| `crates-io-static_assertions.http` | `GET https://crates.io/api/v1/crates/static_assertions` |
| `crates-io-static_assertions-owners.http` | `GET https://crates.io/api/v1/crates/static_assertions/owners` |
| `crates-io-static_assertions-reverse-dependencies.http` | `GET https://crates.io/api/v1/crates/static_assertions/reverse_dependencies?per_page=1` |
| `crates-io-unicode-segmentation.http` | `GET https://crates.io/api/v1/crates/unicode-segmentation`: two versions yanked |
| `crates-io-unicode-segmentation-owners.http` | `GET https://crates.io/api/v1/crates/unicode-segmentation/owners` |
| `crates-io-unicode-segmentation-reverse-dependencies.http` | `GET https://crates.io/api/v1/crates/unicode-segmentation/reverse_dependencies?per_page=1` |
| `github-indexmap-rs-equivalent.http` | `GET https://api.github.com/repos/indexmap-rs/equivalent`, `Accept: application/vnd.github+json`, unauthenticated, as the next three |
| `github-servo-rust-fnv.http` | `GET https://api.github.com/repos/servo/rust-fnv` |
| `github-withoutboats-heck.http` | `GET https://api.github.com/repos/withoutboats/heck` |
| `github-nvzqz-static-assertions-rs.http` | `GET https://api.github.com/repos/nvzqz/static-assertions-rs`, redirects followed: the 301 to `/repositories/100129252`, then the 200 of `nvzqz/static-assertions` |
| `github-rate-limit.http` | `GET https://api.github.com/repos/ai-ctl-probe/no-such-repository`, unauthenticated, once the address's 60 requests of the hour were spent: the 403 and `x-ratelimit-remaining: 0`; the machine's public IPv4 address, which its message names, replaced with `192.0.2.0` |

## Claude Code Hooks

The hook inputs and the listening sockets read by the `reviewing-delivered-code` skill's
`scripts/guard.py`, and tested by its `evals/test_guard.py`. Captured on 2026-11-02, Claude Code
2.1.312: one `claude -p` session, `claude-haiku-5-5` at effort `low`, under
`--setting-sources project,local`, in a repository made in scratch, whose
`.claude/settings.json` wired a hook recording its standard input on `PreToolUse` matched on
`Bash`, and on `SubagentStart` and `SubagentStop` matched on `review-probe`, an agent running
nine commands with Bash. Each input is one line, as the hook read it; the repository's path is
replaced with `/home/user/project`, the home folder with `/home/user`. Report: phase 2's,
`phase-2-scripts-report.md`, of the `review-skill-v2` roadmap.

| Fixture | Captured from |
|---|---|
| `claude-hook-pre-tool-use-main.json` | `PreToolUse` on the main session's `git status --porcelain`: no `agent_id` and no `agent_type` |
| `claude-hook-pre-tool-use-agent.json` | `PreToolUse` on the agent's `git log --oneline -1` |
| `claude-hook-subagent-start.json` | `SubagentStart` of the agent |
| `claude-hook-subagent-stop.json` | The agent's first `SubagentStop`, `stop_hook_active` false, its `sleep 97` run in the background listed `running` in `background_tasks` |
| `claude-hook-subagent-stop-active.json` | Its second, `stop_hook_active` true, after the hook blocked the first |
| `ss-listening.txt` | `LC_ALL=C ss -Hltnup`, iproute2 6.17.0, among the machine's sockets those of a process named `probe server` by `prctl`: a loopback UDP socket, and a loopback TCP server held on two of its descriptors; the machine's LAN address replaced with `192.0.2.1`, its link-local address with `fe80::1` |
