# Test Fixtures

Outputs captured on this machine, for the parsers and adapters to be tested on what the tools
really print. None is written from memory. Commands ran with `LC_ALL=C`, as ai-ctl
runs them: `gcc` and the compilers answer in French otherwise. An `.http` file is a whole HTTP
response, status line and headers included, as `infrastructure::http::parse` reads it.

"Captured from" gives the command or request. Where the report recorded no exact command line, it
is the one the adapter runs. Each section names the report that records the capture. An excerpt keeps the capture's order and spelling, and says what it leaves out.

When a tool's output changes with a new version, capture it again and update its row.

## Host and Toolchain

Captured on 2026-10-09 for `doctor`'s probes, read by `src/infrastructure/host.rs`. Report:
roadmap `v2-rebuild`: `phase-1-doctor-and-inventory-report.md`, or
roadmap `v2-rebuild`: `phase-2-native-build-report.md` where marked (2), or
roadmap `v2-review-fixes`: `phase-5-closing-review-report.md` where marked (5).

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
roadmap `v2-rebuild`: `phase-2-native-build-report.md`, or
roadmap `v2-review-fixes`: `phase-2-dead-code-and-tests-report.md` where marked (3).

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

## Each Project's Remote Tags

Captured on 2026-11-09 by `LC_ALL=C git ls-remote --tags --refs -- <repository> <glob>`, each
project's repository and its recipe's glob, git 2.56.0: one `<hash><TAB>refs/tags/<tag>` per tag,
sorted by name. Read by `src/infrastructure/git.rs`. Report: roadmap
`capabilities-status-and-updates`: `phase-4-builds-behind-their-release-report.md`, or roadmap
`release-0-3-1`: `phase-0-llama-cpp-releases-report.md` where marked (r), on 2026-11-14.

| Fixture | Captured from |
|---|---|
| `llama.cpp-git-ls-remote.txt` | `https://github.com/ggml-org/llama.cpp.git`, `v[0-9]*`: 14 tags, the 13 releases `v0.1.0` to `v1.0.0`, annotated, so that each line holds its tag object's hash, and `v1.0.0-b12359-b1b770d`, which names the commit of `b12359` and `v1.0.0` (r) |
| `whisper.cpp-git-ls-remote.txt` | `https://github.com/ggml-org/whisper.cpp.git`, `v[0-9]*`: 44 tags, 40 releases `vX.Y.Z` |
| `stable-diffusion.cpp-git-ls-remote.txt` | `https://github.com/leejet/stable-diffusion.cpp.git`, `master-[0-9]*`: 733 tags, 603 releases `master-<N>-<hash>` beside the older `master-<hash>` |
| `audio.cpp-git-ls-remote.txt` | `https://github.com/0xShug0/audio.cpp.git`, `v[0-9]*`: 18 tags, 13 releases `vX.Y.Z` beside suffixed ones |

## llama-swap v273

Captured on 2026-11-17 from llama-swap v273, its archive downloaded over HTTPS and found to have
the SHA-256 the release's checksums file gives it. Report: roadmap `release-0-3-1`:
`phase-2-llama-swap-newest-release-report.md`. Read by `src/domain/gateway.rs`, the checksums
file to tie the pinned digest to the release's.

| Fixture | Captured from |
|---|---|
| `llama-swap-273-version.txt` | `llama-swap -version` |
| `llama-swap-273-checksums.txt` | `llama-swap_273_checksums.txt` of the v273 GitHub release |

v271 and v273 changed none of the routes below, their handlers the same as v270's
(`git diff v270 v271`, `git diff v271 v273`) but for `GET /props`, which ai-ctl does not call, so
their captures stay v270's.

## llama-swap v270

Captured on 2026-11-07 from llama-swap v270, its archive checked against the release's checksums
file, on a scratch configuration of one model whose command is a fake upstream answering 200 to
every request and logging each one, shaped as ai-ctl writes a model (`ttl`, `checkEndpoint`, the
key in `env`); each request by `curl -s -i --http1.0`, with the key but for `/health`. Report:
roadmap `capabilities-status-and-updates`: `phase-0-decisions-and-probes-report.md`. The answers by `src/infrastructure/gateway_api.rs` and `src/infrastructure/http.rs`, the unknown
model's 404 by `src/infrastructure/chat_api.rs` too, the upstream's 404 by
`src/infrastructure/server_api.rs` too.

| Fixture | Captured from |
|---|---|
| `llama-swap-270-health.http` | `GET /health` |
| `llama-swap-270-running-empty.http` | `GET /running`, no model loaded |
| `llama-swap-270-running.http` | `GET /running`, the fake model loaded |
| `llama-swap-270-unload.http` | `POST /api/models/unload` |
| `llama-swap-270-logs-stream.http` | `GET /logs/stream/fake`, cut after 2 s, the fake upstream's lines |
| `llama-swap-270-logs-stream-unknown.http` | `GET /logs/stream/nope`: the 400 for an unknown logger |
| `llama-swap-270-chat-unknown-model.http` | `POST /v1/chat/completions` for a model llama-swap does not know: its 404 |
| `llama-swap-270-upstream-props-unknown-model.http` | `GET /upstream/nope/props`, a model llama-swap does not know: its 404, the same as `POST /upstream/nope/tokenize` gives on v263 |

## llama-swap v270 in Front of the 27B

Captured on 2026-11-08 from llama-swap v270 started by `ai-ctl up` with the shipped inventory, in
front of the 27B on `b12111`, each request by `curl -s -i --http1.0` with the key. Report: roadmap
`capabilities-status-and-updates`: `phase-1-llama-swap-v270-report.md`. Read by
`src/infrastructure/chat_api.rs`.

| Fixture | Captured from |
|---|---|
| `llama-swap-270-chat-stream.http` | `POST /v1/chat/completions` streamed through llama-swap, the body the bench's client writes for the test's request: 24 tokens with `ignore_eos`, seed 42, its usage asked |
| `llama-swap-270-chat-bad-request.http` | `POST /v1/chat/completions` whose `messages` is a string, which llama-server refuses: its 400 through llama-swap |

## llama-server b11538

Captured on 2026-10-11 and 2026-10-12 from the 27B on the native `b11538`, through
`/upstream/qwen3.8-27b/`. Report: roadmap `v2-rebuild`: `phase-4-bench-report.md`. Read by
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
| `nvidia-ctk-1.20.1-cdi-spec-excerpt.yaml` | Excerpt of `/var/run/cdi/nvidia.yaml`, driver 615.71.09, 2026-10-12: the device `all`, the env, the libcuda links and mount | roadmap `v2-rebuild`: `phase-4-bench-report.md` |
| `nvidia-ctk-1.20.1-cdi-spec.yaml` | `/var/run/cdi/nvidia.yaml`, whole, written by `nvidia-cdi-refresh.service` at 15:01 on 2026-10-18, driver 615.78.08: 60 `hostPath` entries; the GPU's UUID replaced with zeros | roadmap `v2-review-fixes`: `phase-5-closing-review-report.md` |
| `nvidia-ctk-1.20.1-cdi-spec-excerpt.json` | Excerpt of `nvidia-ctk cdi generate --format json --quiet`, driver 615.78.08, 2026-10-16: the device `all`'s name, the env and the libcuda mount; the GPU's UUID left out | roadmap `v2-review-fixes`: `phase-2-dead-code-and-tests-report.md` |
| `podman-5.8.7-ps-bogus-storage-driver.txt` | `podman ps` under a `CONTAINERS_STORAGE_CONF` naming the driver `bogus`, 2026-10-16: stderr, scratch path included | roadmap `v2-review-fixes`: `phase-1-bugs-and-error-handling-report.md` |
| `podman-5.8.7-image-inspect-digest.txt` | `podman image inspect --format '{{.Digest}}' ghcr.io/ggml-org/llama.cpp:server-cuda13-b11541`, 2026-10-16 | roadmap `v2-review-fixes`: `phase-2-dead-code-and-tests-report.md` |
| `podman-5.8.7-ps-pid.txt` | `podman ps --format '{{.Pid}}'`, one container running, 2026-10-16 | roadmap `v2-review-fixes`: `phase-2-dead-code-and-tests-report.md` |

## Processes

`/proc` files read by `src/infrastructure/gateway_process.rs`, captured on 2026-10-17, kernel
7.2.10, podman 5.8.9, while a container of `docker.io/library/node:22-bookworm-slim` ran `sleep`
(`podman run -d --rm --pull=never`). Report:
roadmap `v2-review-fixes`: `phase-3-performance-report.md`.

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
roadmap `v2-review-fixes`: `phase-5-closing-review-report.md`.

| Fixture | Captured from |
|---|---|
| `proc-net-tcp-excerpt.txt` | `/proc/net/tcp`: the listener (state `0A`, inode 5130254), the client's socket and the accepted one (state `01`) |
| `proc-net-tcp6-excerpt.txt` | `/proc/net/tcp6`: the listener on `[::1]` (state `0A`, inode 5130257) |

The dual-stack socket, captured on 2026-10-19, kernel 7.2.11, while llama-swap v263 ran with
`-listen 0.0.0.0:19200` alone in a network namespace (`unshare -rn`), whose user mapping shows
its uid as 0. Report: the security pass over roadmap `v2-review-fixes`'s fixes, 2026-10-19,
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
