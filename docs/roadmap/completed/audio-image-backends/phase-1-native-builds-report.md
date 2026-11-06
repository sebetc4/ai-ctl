# Phase 1 Report: Native Builds

**Phase:** [phase-1-native-builds.md](phase-1-native-builds.md)
**Start Commit:** `docs(audio-image-backends): close phase 0` of 2026-10-28

---

## Work Log

### 2026-10-28

Phase opened at `docs(audio-image-backends): close phase 0`.

Task 1, each project's release tags. `TagPattern` gains two forms beside `Numbered`:
`NumberedWithCommit`, stable-diffusion.cpp's `master-<N>-<hash>`, ordered by `<N>`, the hash
lower-case hexadecimal; and `Version`, `vX.Y.Z` with nothing after it, compared number by number.
`release` now gives the numbers a release is ordered by. Tested on phase 0's three tag lists:

| Project | Releases | Newest |
|---|---|---|
| whisper.cpp | 40 | `v1.9.5` |
| audio.cpp | 11 | `v0.9.2` |
| stable-diffusion.cpp | 539 | `master-975-95b3090` |

Each count is a `grep -E` of the list. sd.cpp's older `master-<hash>` tags are not releases,
`master-0739361` among them, whose hash is all digits. `cargo mutants --in-diff`: 17 mutants,
17 caught.

Task 5, the submodules a recipe names (`Recipe::submodules`, by path; none for llama.cpp). After
the checkout and its `clean`, `git submodule sync`, then `update --init --force
--filter=blob:none`, then `clean -ffdxq` in each, all limited to the recipe's paths: the
command phase 0 ran by hand, plus what keeps a clone reused across builds exact.
- `sync` follows a release whose `.gitmodules` moves a submodule to another repository, which
  `update` alone would fetch from the old URL;
- `--force` resets a submodule already at the commit, which `update` otherwise leaves as it is;
- with no path, `update --init` would take every submodule: a recipe that names none runs no
  submodule command.

Tested on a local repository with two submodules, one a fork moves in a third release.
Production git refuses to clone a local repository for a submodule (`transport 'file' not
allowed`, probed with git 2.56.0, the clone's own configuration not lifting it), so the
adapter's `git` adds `-c protocol.file.allow=always` under `#[cfg(test)]` only, as
`command::spawn` holds its test guard. Each of `sync`, `--force`, the `clean` and the filter,
removed by hand, fails the test. The test ran 30 times, then the library's tests 30 times, no
failure. `cargo mutants --in-diff`: 3 mutants, 1 caught, 2 unviable.

When the stable-diffusion.cpp recipe ships, `docs/security.md`'s "Build Sources" section
must name its `ggml` submodule's repository, `leejet/ggml`, whose URL `.gitmodules` gives at each
release (done with task 3).

Task 6, the smoke test and the device listing. Probed first on phase 0's patched builds, without
`LLAMA_API_KEY` (the patches check it only once the arguments are read):

| Command | Exit | Output |
|---|---|---|
| `whisper-server --version` | 0 | `error: unknown argument: --version`, then the whole usage, on stderr |
| `whisper-server --no-such-option` | 0 | The same |
| `sd-server --version` | 0 | `stable-diffusion.cpp version master-975-95b3090, commit 95b3090` |
| `sd-server --no-such-option` | 1 | `[E] error: unknown argument` |
| `audiocpp_server --version` | 0 | `audio.cpp dev`, its commit, its build, `backends: cpu,cuda` |
| `audiocpp_server --no-such-option` | 1 | `missing required --config argument` |

A recipe's smoke test (`SmokeTest`) is now arguments and a text the output, either stream, must
hold: an exit status of 0 proves only that a whisper-server starts. llama.cpp's is `--version`
printing `version: `, as `llama-server-version.txt` does; the three recipe tasks give theirs.
`cuda_devices` reads the three forms, `CUDA<n>:`, `CUDA<n><TAB>` and `CUDA:<n> `, tested on
`sd-server`'s and `audiocpp_server`'s captures. A recipe without a device listing, as
whisper-server's will be, is tested on its output alone: no check of its CUDA build, which
CMake's `GGML_CUDA=ON` requires anyway. `cargo mutants --in-diff`: 8 mutants, 8 caught.

`doctor` still runs `--version` on each project's current build: task 9 runs the recipe's smoke
test there.

Task 7, the recipes' patches: the mechanism, the user agreeing on 2026-10-28 to where the patches
live and how a build records them (Decisions).
- `Recipe::patches`, each a name and a diff embedded with `include_str!`, applied in order by
  `Sources::apply`: `git apply` of a file in the clone's `.git/`, removed after. `git apply` changes
  nothing when any hunk fails, and the next checkout undoes a patch. A patch that does not apply
  stops the build before the compile: "<project> <id> cannot be built: its patch <name> does not
  apply; `--ref` builds a release it applies to", then git's line.
- `build.json` format 2 adds `patches`, the names; format 1 reads as no patch. The build's
  `patches/` holds a copy of each. A bench run's copy of the build holds `patches` only when there
  are some, so the runs of llama.cpp builds stay as they were.
- The output names each patch as it applies, and the result lists them.

The use case's patch step is tested through a recipe of the test's own, not through `build`, which
reads the shipped recipes, none with a patch yet: the whisper.cpp recipe's task tests it through
`build`. The git test ran 30 times, no failure. `cargo mutants --in-diff`: 14 mutants, 9 caught,
5 unviable.

The patch files go into the repository with their recipes, under `src/domain/patches/<project>/`.
`sd-server`'s change to `listen` is a patch of its own, `listen-failure.patch`, applied after
`api-key.patch`: not part of the key, it keeps that patch the smallest that makes the server
require it. Probed on phase 0's `sd-server`, rebuilt by hand with both, FLUX.3 Image Klein loaded:

| Case | Result |
|---|---|
| Port held by another process | Exit 1 after 5 s, `[E] cannot listen on 127.0.0.1:18977` |
| No `LLAMA_API_KEY` | Exit 1, `[E] LLAMA_API_KEY is not set` |
| `/health`, without a key | 200 |
| `/v1/models`, no key or a wrong one | 401 |
| `/v1/models`, `Bearer` or `X-Api-Key` | 200 |

Both patches are in `~/.cache/ai-ctl-probes/patches/` (untracked) until their recipe ships them.
The task stays open until each server, built by ai-ctl with its patches, is probed the same way
(task 10). The installed llama.cpp builds, of format 1, list as before.

Task 2, the whisper.cpp recipe, from phase 0's hand build: `vX.Y.Z` tags, no submodule,
`WHISPER_BUILD_TESTS=OFF` and `WHISPER_BUILD_EXAMPLES=ON`, the `whisper-server` target at
`bin/whisper-server`, `--help` printing `--inference-path` as its smoke test, no device listing,
and `api-key.patch`, phase 0's patch as it was probed, checked to apply to `v1.9.5` with
`git apply --check`. Tested on the captures: `v1.9.5` the newest release, the smoke test's text in
the help. A shipped patch has a plain name and is a diff. Through `build`, now that a shipped
recipe has a patch: the patch applies before CMake configures, the build records it and the store
gets it; a patch that fails stops the build before the compile, nothing staged.

A second recipe broke six tests that took `whisper.cpp` for a project without one, or `list` of
every project for llama.cpp's alone. The unknown project is now `nope.cpp`; the `list` tests ask
for llama.cpp, since the fake store holds the same builds whatever the project: task 8 tests
several projects. `cargo mutants --in-diff`: no mutant, the diff holding the recipe and tests only.

`docs/security.md`'s "llama.cpp Sources" became "Build Sources", with whisper.cpp's repository and
what its patch changes; `docs/audio/whisper-cpp.md` says ai-ctl builds it and applies the patch.

Task 3, the stable-diffusion.cpp recipe: `master-<N>-<hash>` tags, the `ggml` submodule alone,
`SD_CUDA=ON`, `SD_BUILD_EXAMPLES=ON`, `SD_SERVER_BUILD_FRONTEND`, `SD_WEBP` and `SD_WEBM` off, the
`sd-server` target, `--version` printing `stable-diffusion.cpp version`, `--list-devices`, and
`api-key.patch` then `listen-failure.patch`, both checked to apply in that order to
`master-975-95b3090`. Tested on the captures: the newest release, the smoke test's text in
`--version`'s output, one CUDA device in the listing. `cargo mutants --in-diff`: no mutant.

With three projects, `docs/commands.md`'s `build` gives each recipe in a table: repository and
submodules, release tags, server, smoke test, device listing, patches. `docs/security.md`'s Build
Sources names the sd.cpp repository, its `ggml` submodule's, `leejet/ggml`, and what both patches
change; `docs/image/stable-diffusion-cpp.md` says ai-ctl builds it and applies them.

Task 4, the audio.cpp recipe: `vX.Y.Z` tags, no submodule, its one being SSH-only, the options of
phase 0's hand build, `ENGINE_ENABLE_CUDA=ON` and `ENGINE_ENABLE_NATIVE_CPU=ON`, plus
`AUDIOCPP_DEPLOYMENT_BUILD=ON`, the `audiocpp_server` target, `--version` printing `audio.cpp `,
`--list-devices`, and `api-key.patch`, checked to apply to `v0.9.2`. Tested on the captures.
`AUDIOCPP_DEPLOYMENT_BUILD` was read in the sources by phase 0, not probed: task 10's build of
audio.cpp checks that a Qwen3-TTS from Qwen's safetensors starts without `model_spec_override`.
`cargo mutants --in-diff`: no mutant. The four recipes are shipped; the recipe module's doc no
longer says llama.cpp's is the only one.

Task 8, `list`, `use` and `prune` across several projects. The code needed no change: each loops
over the recipes, every read and write naming its project, and the store keeps a directory and a
`current` link per project. The use case's fake store did not: it gave every project the same
builds and one `current`, so that with four recipes the prune tests pruned four projects sharing
llama.cpp's builds. It now keeps each project's builds and `current` apart, and the listing tests
of task 2 ask for every project again. Five tests on llama.cpp's and whisper.cpp's builds, each
project with its own current, older than its newest:
- `list` shows each project with its own current, in the recipes' order;
- `use` changes its own project's current only, and refuses another project's build;
- `prune` keeps each project's current and newest, a prune of one project leaves the others, and
  a gateway serving one project's build stops the whole prune.

They passed at once, the behaviour being there since the v2 rebuild. Each of the six reads and
writes that name the project in `list`, `use` and `prune`, made to name another one by hand, fails
one of them, or for `use`'s read, the existing `use` tests. `cargo mutants --in-diff`: no mutant,
the diff holding tests only.

Task 9, `doctor`'s check of each project's current build. It already checked each recipe's
project, but ran `--version` on every binary, which a whisper-server passes with "unknown
argument". It now runs the recipe's smoke test, and the builder and `doctor` share what is wrong
with its output (`smoke_test_failure`): "<build> does not run: `<command>` failed: …", or "<build>
is not the server its recipe builds: `<command>` printed no `<text>`: …". `FakeBuilds`, shared by
the use cases' tests, now keeps each project's builds and `current` apart, as task 8 made the
build tests' store: the healthy machine's report had passed whisper.cpp, sd.cpp and audio.cpp on
llama.cpp's build, and now skips them, the inventory running none. `cargo mutants --in-diff`: 8
mutants, 7 caught, 1 unviable.

Task 10, the live builds, run with `target/debug/ai-ctl` built from the working tree, so that
`target/release/ai-ctl`, which the Makefile runs, stays as it was.
- `ai-ctl build whisper.cpp`: `v1.9.5`, still the newest release, built in 3 min 38 s, its patch
  applied, its smoke test passed. Probed with `~/.cache/ai-ctl-probes/bin/probe-built.sh`
  (untracked), large-v3-turbo loaded: without `LLAMA_API_KEY` it exits 1, "LLAMA_API_KEY is not
  set"; ready in 3 s; `/health` 200 without a key; `GET /`, the transcription `POST`, an `OPTIONS`
  preflight and a `text/plain` `POST` from another origin 401 without a key or with a wrong one;
  `Bearer` and `X-Api-Key` let it through; no `Access-Control-*` header.
- `ai-ctl build stable-diffusion.cpp` and `ai-ctl build audio.cpp` were refused before the
  compile, as task 7 meant: the newest releases are now `master-1017-be48649` and `v0.9.3`, and
  "its patch api-key.patch does not apply", on `examples/server/main.cpp:82` and
  `app/server/request_dispatch.h:7`. Upstream changed the lines around the patches, not what they
  guard: sd.cpp logs `svr_params` at debug level and registers model routes before `listen`;
  audio.cpp adds includes and an `additional_routes` argument to `RequestDispatcher::handle`,
  which both runtimes still call for every request.
- The three patches rebased by hand on scratch clones, each with the same added and removed lines
  as the probed one, and checked with `git apply --check` to apply in order to the new release.
  They replace the files under `src/domain/patches/`, uncommitted. Built again after the close:
  `stable-diffusion.cpp master-1017-be48649` in 4 min 39 s and `audio.cpp v0.9.3` in 6 min 35 s,
  their patches applied, their smoke tests and device listings passed.

The two servers built with the rebased patches, probed with `probe-built.sh`, and with
`probe-sd-routes.sh` and `probe-audio-tts.sh` beside it (untracked):
- `sd-server` `master-1017-be48649`, FLUX.3 Image Klein loaded: without `LLAMA_API_KEY` it exits
  1, "LLAMA_API_KEY is not set"; ready in 5 s; `/health` 200 without a key; `/v1/models`, the
  generation `POST`, an `OPTIONS` preflight and a `text/plain` `POST` from another origin 401
  without a key or with a wrong one; `Bearer` and `X-Api-Key` let it through; no
  `Access-Control-*` header. The six model routes of `master-1017`, three `GET` and three `POST`,
  401 without a key; the `GET`s 200 with it. On a port a Python listener holds, it exits 1 after
  5 s, `[E] cannot listen on 127.0.0.1:18983`.
- `audiocpp_server` `v0.9.3`, Qwen3-TTS from Qwen's safetensors with no `model_spec_override`:
  without the key it exits 1; ready in 3 s; the same checks on `/v1/audio/voices` and
  `/v1/audio/speech` pass. It spoke an English and a French sentence, two 24 kHz mono WAVs, in
  2.9 s and 1.7 s: the spec comes from the binary, the proof `AUDIOCPP_DEPLOYMENT_BUILD=ON`
  lacked. On a port another `audiocpp_server` holds, it exits 1 after 1 s, "could not bind".

Tasks 10 and 7 done: the three servers ai-ctl built require the key, `sd-server` ends a failed
`listen` with exit 1. The checks green; `cargo mutants --in-diff`: no mutant, the diff holding
patches only.

A second server on the port of a live one of its kind, with `probe-reuseport.sh` (untracked):
- `whisper-server` and `sd-server` bind it: `ss` lists both processes listening on the port, and
  the kernel shares the connections between them, without a word. Their cpp-httplib's
  `default_socket_options` sets `SO_REUSEPORT`, which lets a socket of the same user carrying it
  bind a port already bound.
- `llama-server` `b11700` does not: `tools/server/server-http.cpp:218` replaces cpp-httplib's
  options with `SO_REUSEADDR`, `SO_REUSEPORT` only under `--reuse-port`, "default: disabled" in its
  help. On a port a Python listener holds with `SO_REUSEPORT`, it exits 1, "couldn't bind HTTP
  server socket"; with `--reuse-port`, it binds beside it.
- `audiocpp_server` sets `SO_REUSEADDR` alone (`app/server/http.cpp:754`): exit 1, above.

Task 13, `reuse-address.patch` for whisper.cpp and stable-diffusion.cpp. The recipe tests' patch
lists first, both failing on their assertion; then each patch, a `set_socket_options` that sets
`SO_REUSEADDR` alone through `setsockopt`, cpp-httplib's own helper being in its `detail`
namespace, before `bind_to_port` in `whisper-server` and before `listen` in `sd-server`, applied
after the patches already there. The build test that took whisper.cpp's one patch now takes both.
Rebuilt with `--force` at `v1.9.5` and `master-1017-be48649`, the three or two patches applied,
the smoke tests passed. Probed with `probe-reuseport.sh` and `probe-timewait.sh` (untracked):

| Case | `whisper-server` | `sd-server` |
|---|---|---|
| A second server on the port of a live one | Exit 1, "couldn't bind to server socket" | Exit 1, `[E] cannot listen on 127.0.0.1:18985` |
| A port in TIME-WAIT after `python3 -m http.server` | Ready | Ready |
| A port in TIME-WAIT after a stopped patched `sd-server` | Ready | — |

The control, phase 0's `whisper-server`, built with the key patch alone, exits 1 on both
TIME-WAIT cases, "couldn't bind to server socket". `docs/commands.md`'s recipes,
`docs/security.md`'s Build Sources and the two domain documents name the patch, and
`docs/image/stable-diffusion-cpp.md` no longer counts llama.cpp's server among those binding with
`SO_REUSEPORT`. The checks green; `cargo mutants --in-diff`: no mutant, the diff holding the
recipes' data and tests.

Task 11, `build use` back to an older build: `target/debug/ai-ctl build use llama.cpp b11538`,
"llama.cpp: current build is now b11538 (was b11700)". Each project's builds and `current`, listed
before and after, differ only in llama.cpp's `current`; `build list` marks `b11538` and the other
three projects' builds as they were. `build use llama.cpp b11700` put back the build the
gateway's configuration expects, the listing then the same as before.

Task 12, the documentation. `docs/commands.md`'s `build` and README's build section name the
four projects, and `docs/security.md`'s Build Sources what each patch changes and why, since
tasks 2 to 4 and 13. Checked against the builds made:
- README's build tree gave `whisper.cpp`'s `patches/` one patch: it names the folder now;
- `docs/security.md` gives `ggml`'s repository at `master-1017-be48649`, the same URL; its
  `reuse-address.patch` bullet, put by task 13 between sd.cpp's and audio.cpp's, made audio.cpp's
  "does the same" read as a binding change: moved after it, and linked to the domain documents;
- each domain document says which release its facts hold, `master-975-95b3090`, `v0.9.2` and
  `v1.9.5`, unless dated otherwise. To them are added, dated: sd.cpp's model routes at
  `master-1017-be48649` and its key patch rebased and probed there; audio.cpp's key patch
  rebased on `v0.9.3` and probed, and its deployment build probed with Qwen3-TTS.

The closure, at the user's "oui". Its checks, before any file: the six green (748, 27 and 3
tests; `test_scope.py` 15, `test_fixtures.py` 1; `sid.py` clean). The first acceptance criterion
stopped it. `ai-ctl build stable-diffusion.cpp`, no ref, took the newest release,
`master-1024-77b72a2`, to which the three patches apply in order (`git apply` on a scratch
clone), and CMake refused it: "CUTLASS headers not found at
…/ggml/src/ggml-cuda/vendors/cutlass".
- The fork of ggml at `643e8d97` adds NVIDIA's CUTLASS as a submodule of its own (`05586dc7`,
  "build(cuda): add optional CUTLASS submodule support"), and `GGML_CUDA_CUTLASS_PATH` defaults to
  that folder: an empty one stops the configure.
- CUTLASS builds the INT8 GEMM kernels (`mul-mat-i8.cu`) and a CuTe FlashAttention
  (`fattn-cute.cu`) that `ggml_cuda_flash_attn_ext` prefers when it is supported, FLUX's
  attention among them (measured below, after task 14).
- Nothing was installed: the current build stays `master-1017-be48649`. whisper.cpp `v1.9.5` and
  audio.cpp `v0.9.3` are still the newest releases (`git ls-remote`).

Task 14, added on the user's choice (Decisions), stopped midway, uncommitted:
- `src/infrastructure/git.rs`: a recipe names a submodule of a submodule by its whole path, and
  `update_submodules` checks it out within its parent once the parent is: git takes from a
  repository only its own submodules ("pathspec 'ggml/vendors/cutlass' did not match any file(s)
  known to git" from the top, the new test's first failure). A named submodule a release does not
  hold, no `160000` entry in `ls-files --stage -z`, is skipped, so that `--ref master-1017` still
  builds; a top-level one a release lacks is skipped too, where the build stopped before.
- The test, `a_submodule_of_a_submodule_is_checked_out_within_it`, on local repositories, ggml
  gaining `vendors/cutlass` between `b1` and `b2`: not taken unless named, checked out fetched as
  the clone is, cleaned, absent at `b1`, back at `b2`. Watched failing on its assertion, then
  green; 30 runs, no failure.
- The sd.cpp recipe names `ggml/src/ggml-cuda/vendors/cutlass`; its test watched failing first.
- fmt, clippy and the tests green. `cargo mutants --in-diff`: 13 mutants, 12 caught, 1 missed:
  `&&` → `||` in `update_within`'s `160000` check (`src/infrastructure/git.rs:252`), which
  takes any submodule the listing returns for every path named beside it.

Task 14 finished in the next session:
- The missed mutant killed: `a_recipes_submodules_are_checked_out_with_the_sources` now checks
  out a recipe naming `ggml` and `webm`, which no release holds, at `b2`. Under the mutant, the
  `ggml` entry counts `webm` as held and the checkout fails on its assertion, "cannot set the
  submodules' URLs: error: pathspec 'webm' did not match any file(s) known to git"; restored,
  green; 30 runs, no failure. `cargo mutants --in-diff` on the whole code diff: 13 mutants,
  13 caught.
- `target/debug/ai-ctl build stable-diffusion.cpp`, no ref: `master-1024-77b72a2`, the newest
  release (`git ls-remote`), checked out with `ggml` at `643e8d97` and its CUTLASS at
  `ad7b2f5e`, NVIDIA's `v3.9.2`, 143 MB checked out; the three patches applied; 307 steps,
  `mul-mat-i8.cu` and `fattn-cute.cu` among its 150 CUDA objects, in 2 min 47 s; the smoke test
  and device listing passed; current build, `master-1017-be48649` kept.
- Probed, FLUX.3 Image Klein loaded, as `master-1017` was:
  - `probe-built.sh`: without `LLAMA_API_KEY` exit 1, "LLAMA_API_KEY is not set"; ready in
    5 s; `/health` 200 without a key; `/v1/models`, the generation `POST`, an `OPTIONS`
    preflight and a `text/plain` `POST` 401 without a key or with a wrong one; `Bearer` and
    `X-Api-Key` let it through; no `Access-Control-*` header.
  - `probe-reuseport.sh`: a second server on the live port exits 1, `[E] cannot listen on
    127.0.0.1:18092`, the first alone listening. `probe-timewait.sh … python`: one TIME-WAIT
    on the port, then ready. Its `server` mode left no TIME-WAIT this time, no result.
- Two 1024×1024 images through `/v1/images/generations`, `req-image.sh`'s fox, 16 steps at
  CFG 1.0: 10.0 and 9.8 s per request, sampling 9.2 and 9.1 s, 15778 MiB of VRAM in use after them; the
  sign reads "ai-ctl". With `GGML_CUDA_DISABLE_CUTE_FATTN=1`, the same two: sampling 10.0 and
  10.0 s. CuTe FlashAttention takes FLUX's attention, head size 128 over 4096 image tokens and
  more, about 8 % of the sampling; sd.cpp's log line, `flash attention auto: CUDA0 -> on (CUDA
  MMA)`, comes from its own probe and does not name ggml's kernel.
- `docs/commands.md`'s recipe table and build steps, `docs/security.md`'s Build Sources
  (`NVIDIA/cutlass`) and `docs/image/stable-diffusion-cpp.md` §1 (CUTLASS, the build and the
  measure) brought to it. Committed as `feat(build): check out a submodule a recipe's submodule holds`, the six checks green (749, 27 and 3 tests;
  `test_scope.py` 15, `test_fixtures.py` 1; `sid.py` clean).

The closure resumed at its step 2. The acceptance criteria hold:
- `ai-ctl build <project>` with no ref built each project's newest release, smoke-tested:
  whisper.cpp `v1.9.5` and audio.cpp `v0.9.3` (task 10), stable-diffusion.cpp
  `master-1024-77b72a2` (task 14), each still the newest by `git ls-remote` at the closure;
- `build use` and `build prune` act on one project alone: task 8's tests, and task 11's live
  `use` back to `b11538`, which left the other projects' builds as they were;
- the checks green, as above.

---

## Decisions

- **Where the patches live and how a build records them (the user, 2026-10-28, "oui"):** in
  `src/domain/patches/<project>/<name>.patch`, embedded in the binary with `include_str!`, since a
  production binary built at a release tag has no repository beside it; `build.json` goes to
  format 2 with the names of the patches applied, format 1 read as no patch, and the build keeps a
  copy of each in its `patches/`, so that the record is exact without a hashing crate. Set aside:
  `build.json` left at format 1 with an optional field, which an older ai-ctl would refuse as an
  unknown field rather than as a format it does not read.
- **`whisper-server` and `sd-server` bind as `llama-server` does (the user, 2026-10-28, "Patch
  bind"):** a `reuse-address.patch` in each of the two recipes, which sets `SO_REUSEADDR` alone,
  `llama-server`'s default. A second server on a live port then fails to bind and exits 1,
  `whisper-server` by its own check, `sd-server` by `listen-failure.patch`, and a TIME-WAIT left
  by a server of the stack no longer stops a start. The cost: one patch more to rebase in each
  project. Set aside: leaving both cases to phase 2's port task without a patch, and recording
  the finding only.
- **sd.cpp's CUTLASS is fetched, not disabled (the user, 2026-10-28, "Récupérer CUTLASS"):** the
  builder checks out a submodule of a submodule a recipe names, and the sd.cpp recipe names
  `ggml/src/ggml-cuda/vendors/cutlass`, so that the build stays the one upstream makes by
  default. Set aside: `GGML_CUDA_CUTLASS_PATH` empty, which drops the INT8 and CuTe
  FlashAttention kernels unmeasured; closing with the first criterion unmet and the fix moved to
  phase 2.

---

## Deviations

- **Phase 0's patches no longer applied to the newest releases** of stable-diffusion.cpp
  (`master-1017-be48649`) and audio.cpp (`v0.9.3`), eight days later. Found by the live builds
  of task 10, which refused both before the compile. Consequence: the patches were rebased on
  those releases (Work Log, 2026-10-28), so that a recipe's patches follow its newest release;
  they are not expected to apply to `master-975-95b3090` or `v0.9.2` any more, not checked.
- **`sd-server` at `master-1017` serves model routes** that `master-975` did not:
  `/sdcpp/v1/model-files`, `/sdcpp/v1/models/load`, `/unload`, `/refresh`, `/status`, and
  `/sdcpp/v1/recipes`. A client with the key could load a model through them, outside llama-swap's
  arbitration of the VRAM; the key patch's check guards them like every route, probed.
  Consequence: left to phase 2, whose kinds and gateway entries decide what a client reaches; a
  task added there on 2026-10-28.
- **`whisper-server` and `sd-server` share a port with a live server of their kind**, where
  phase 0 saw only a TIME-WAIT stop them. Found probing `listen-failure.patch` with a second
  `sd-server` on the first's port: it stayed up 120 s, both listening. Consequence: task 13
  added to this phase, a `reuse-address.patch` in each (Decisions), last so that the Work Log's
  task numbers hold.
- **Phase 0 counted `llama-server` among the servers that bind with `SO_REUSEPORT` alone.** Its
  cpp-httplib's default does, but `llama-server` sets its own options (Work Log). Consequence:
  phase 2's port task reworded: it probes whether a server binds a port another left in
  TIME-WAIT, every server binding with `SO_REUSEADDR` once task 13 is done, and keeps the inventory
  from handing a port on within 60 s only if one cannot.
- **stable-diffusion.cpp's `master-1024-77b72a2` cannot be built by task 3's recipe**: its fork
  of ggml needs CUTLASS, a submodule of its own. Found by the closure's first acceptance
  criterion. Consequence: task 14 added to this phase (Decisions), the closure paused at its
  step 2, then resumed once it was done.

---

## Assessment

`ai-ctl build` now builds whisper.cpp, stable-diffusion.cpp and audio.cpp as it builds
llama.cpp, nothing in the builder specific to one project:
- each recipe carries its tag pattern (`Numbered`, `NumberedWithCommit`, `Version`), its
  submodules by path, a submodule's own included, and its smoke test and device listing;
- each recipe carries its patches, embedded in the binary. A release one no longer applies to
  is refused before the compile, and `build.json` format 2 records them, with a copy kept.

`build list | use | prune` and `doctor` cover the four projects. The newest release of each,
built live, gives a server that requires the key on every route but `/health` and sends no CORS
header. `whisper-server` and `sd-server` bind as `llama-server` does, and `sd-server` exits 1
when it cannot listen.

What cost the most was upstream's pace. stable-diffusion.cpp's newest release moved from
`master-975` to `master-1017`, then to `master-1024`, within the phase. The first move broke its
patches, rebased with audio.cpp's. The second broke the configure, on a CUTLASS submodule its
fork of ggml added. The builder now takes that submodule, and its FlashAttention takes about
8 % off FLUX's sampling. A future release can break a patch or the configure again. Either
stops the build and keeps the current one: the patch before the compile, the configure
after it.

What phase 2 needs to know first:
- The servers are at `~/.local/share/ai-ctl/builds/<project>/current/`: `whisper-server`,
  `sd-server` (`master-1024-77b72a2`) and `audiocpp_server`. Each refuses to start without
  `LLAMA_API_KEY`, so each gateway entry passes it through `env:`.
- audio.cpp's deployment build carries its models' specs: no `model_spec_override`.
- The port task starts from task 13's probes. A second server on a live port exits 1, and both
  servers start on a port left in TIME-WAIT.
- `sd-server`'s model routes still wait for phase 2's task.
