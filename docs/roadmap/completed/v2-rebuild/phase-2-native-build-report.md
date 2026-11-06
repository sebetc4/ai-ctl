# Phase 2 Report: Native Build

**Phase:** [phase-2-native-build.md](phase-2-native-build.md)
**Start Commit:** `feat(doctor): add doctor, init and the inventory` of 2026-10-09

---

## Work Log

### 2026-10-09

Phase opened right after the closure of phase 1 (`feat(doctor): add doctor, init and the inventory`); the opening edits to the README and
the phase file are not committed yet. Resumed in a new session: read the phase file, the phase 1
file and report in full, and the roadmap README. No restructuring was pending. The dependency the
phase file names is still unsettled: `nvcc` is 13.2.86 and `/usr/local/cuda` points at
`/usr/local/cuda-13.2`, with no other toolkit under `/usr/local`.

The user ran `sudo dnf install cuda-toolkit-13-4` alone, which found no package: the only CUDA
repository configured is still `cuda-fedora43`. `doctor`'s remedy prints the `addrepo` command and
the install command on two lines with nothing saying both are needed, in that order.

Before removing 13.2, the user asked whether anything still needs it. The package they installed is
the unversioned `cuda-toolkit` meta-package, which follows NVIDIA's newest release and could pull a
toolkit newer than the driver once the `fedora44` repository is added. The loader finds the CUDA
libraries through `/etc/ld.so.conf.d/`, which lists `/usr/local/cuda/targets/x86_64-linux/lib` and
`/usr/local/cuda-13/...`, both alternatives links. v1's leftover `libggml-cuda.so` needs
`libcudart.so.13`, `libcublas.so.13` and `libcublasLt.so.13` with no RUNPATH into a toolkit, so a
dynamically linked build follows whichever 13.x toolkit the link points at, and breaks only when
no 13.x is left. This is the input for the `GGML_STATIC` evaluation.

The user installed `cuda-toolkit-13-4` and removed `cuda-toolkit` and `cuda-toolkit-13-2`:
`doctor` reports `nvcc` 13.4.92 under `/usr/local/cuda-13.4`, and the host compiler is now
`g++` 16.2.1, since 13.4's `host_config.h` accepts GCC 16. Three 13.2 packages remain
(`cuda-opencl-13-2`, `cuda-driver-devel-13-2`, `cuda-toolkit-13-2-config-common`), required by
nothing, and `/usr/local/cuda-13.2` stays registered as an alternative of lower priority; harmless.
`cuda-fedora43` is disabled through a dnf override.

That override also excluded NVIDIA's driver packages on `cuda-fedora43`; `cuda-fedora44` has no
exclude. It carries a whole driver stack (`nvidia-driver*`, `nvidia-settings`, `nvidia-modprobe`,
`nvidia-persistenced`, `kmod-nvidia-*-dkms`, `xorg-x11-nvidia`, …) at the same epoch and version as
the RPM Fusion driver installed here (615.71.09). Nothing was replaced: `dnf history` shows the
13.4 install pulled no driver package, `nvidia-settings` still comes from `rpmfusion-nonfree-updates`,
and `dnf check-upgrade` offers only the two `config-common` packages. The risk is the next driver
NVIDIA publishes before RPM Fusion. `doctor`'s remedy, which adds the repository, carries no
exclude either, and hard-codes `fedora44` in its URL (`src/application/doctor.rs:35`).

The user applied the exclusion on `cuda-fedora44` (`dnf repoquery` now finds no driver package
there) and removed the three 13.2 leftovers; `/usr/local/cuda-13.2` is gone from the
alternatives. A `dnf upgrade` in between moved only the two `config-common` packages to 13.4.92.
`g++` on the `PATH` is ccache's wrapper, `/usr/lib64/ccache/g++`: the compile tasks must pass the
real compiler as `CMAKE_CUDA_HOST_COMPILER`.

The user asked that the commands run be recorded and that the repository allow a fresh install of
the build tools, or a version change, without friction. Added three tasks to this phase (see
Problems And Deviations) and wrote `docs/toolchain.md`: requirements, a fresh install on Fedora 44
(RPM Fusion driver with Secure Boot, NVIDIA's repository with the driver packages excluded, a
versioned toolkit package, build tools), a CUDA version change with rollback through the
`alternatives` links and removal of the leftovers, a Fedora release change, and a dated record of
the commands of this session, rebuilt from `dnf history` 330 to 333 and the dnf override file.
Checked the commands against RPM Fusion's Configuration, NVIDIA and Secure Boot pages and NVIDIA's
installation guide, which confirmed that only `cuda-toolkit-X-Y` stays on its version and corrected
the Secure Boot package list (`kmodtool`). Linked it from `docs/stack.md` §8, the README,
`docs/architecture.md` and `CLAUDE.md`.

`cargo test` then failed in two `init` tests of `tests/cli.rs`: the repository moved from
`/code/rust/projects/ai-ctl` to `/code/ai/ai-ctl`, and the cached test binary still held the old
`CARGO_BIN_EXE_ai-ctl` path. `cargo clean -p ai-ctl` and a rebuild made all 118 tests green; no
code change. The v1 archive stays at `/code/rust/projects/llm-ctl-v1-archive/`, where `CLAUDE.md`
points.

The user asked to record the progress and the decisions, then to commit and end the session; the
work goes on in a new one. The user did not ask to remove the third added task (the `doctor`
warnings), which stays. `CLAUDE.md` is ignored by git (`.gitignore`), so its rule about the
toolchain record stays local; `docs/toolchain.md` states the same rule at the head of its record.
The three checks are green. Committed the phase opening, the three added tasks, the toolchain
document and its links, and this report together as `docs: open phase 2 and document the build
toolchain`. No code of this phase is written yet: 1 of 15 tasks is done.

Where the next session starts: the Domain tasks — the build model with its JSON metadata, the
recipe with the llama.cpp recipe, and the toolkit rule moved from the private functions of
`src/application/doctor.rs` into the domain, which the build gate and `doctor` then share. The
`doctor` remedy and warning tasks build on that rule. The machine is ready for a real build:
`nvcc` 13.4.92, `g++` 16.2.1 accepted, CMake 4.3.0, git 2.56.0; the host compiler to pass is the
real `g++`, not ccache's wrapper. `GGML_STATIC` is still to be evaluated.

Resumed in a new session: read the phase file, this report, the phase 1 report and the roadmap
README, then the code `doctor` is made of and v1's builder from the archive bundle. v1 drove
`git2` and plain `cmake` with a hard-coded GCC limit and a `120` target; nothing worth porting
beyond the shape of the CMake calls.

Read llama.cpp's build files at `79e2e74` (2026-10-09), shallow-cloned into the session
scratchpad, before modelling anything:

- `BUILD_SHARED_LIBS` defaults to `ON`, so a default build loads `libllama.so` and `libggml*.so`
  from its build tree; `OFF` links them into the binary.
- `GGML_STATIC=ON` adds `-static` to every link (`ggml/src/CMakeLists.txt`) and links
  `cudart_static`, `cublas_static` and `cublasLt_static`. ggml also links the driver's `libcuda`
  unless `GGML_CUDA_NO_VMM`, and the driver ships no static `libcuda`: a two-line C program
  calling `cuInit`, linked with `-static -lcuda` against the toolkit's stubs, fails with `ld` asking
  for "the static version of the cuda library". `libstdc++-static` is not installed either.
  `GGML_STATIC` is not an option for a CUDA build on Linux: builds stay linked to the CUDA
  libraries by soname (see Decisions).
- `GGML_CCACHE` is `ON` by default and uses `ccache` when it is found: a fresh build tree still
  reuses the objects of the previous build.
- The server is in `tools/`, added when `LLAMA_BUILD_TOOLS` and `LLAMA_BUILD_SERVER` are on;
  `LLAMA_BUILD_APP` adds a unified `llama` binary. Building the `llama-server` target alone
  compiles nothing else.
- `LLAMA_USE_PREBUILT_UI` (default `ON`) downloads the web UI from a Hugging Face bucket at build
  time; the build already needs the network for the sources.
- Release tags are lightweight and `b<number>`; the newest is `b11538`. `git ls-remote` with the
  glob `b*` also returns `backup/…` tags, and `b[0-9]*` still returns `b10964-b10964-b29c606`.

On this machine, `g++` on the `PATH` resolves to `/usr/lib64/ccache/g++`, a link to
`/usr/bin/ccache`, because `/usr/lib64/ccache` comes before `/usr/bin`. The `HostProbe` reports
that wrapper as `g++` today; the compile tasks must skip it.

Moved the toolkit rule into the domain: `toolkit_problem` in `src/domain/machine.rs` returns
`Broken`, `TooOld` or `NewerThanDriver`, and `doctor` formats it; the CMake and git minimums
moved with it as `Tool::minimum`. The `doctor` tests pass unchanged. Then wrote the build model:
`BuildId` (a release tag or ten digits of the commit, a safe directory name, never `current`),
`CommitId`, `Build` with the toolkit version, the host compiler, the CMake flags by name (the CUDA
architectures read from `CMAKE_CUDA_ARCHITECTURES`) and a `Timestamp`, a new UTC type written in
RFC 3339 without a date crate. The JSON metadata, `build.json`, lives in
`src/infrastructure/build_store.rs`, which keeps serde out of the domain as the inventory does; its
tests round-trip a build and refuse another format, unknown or missing fields and invalid values.
The recipe, `src/domain/recipe.rs`, holds the project, repository, tag pattern, the project's own
CMake options, the target, the artifact path and the device listing; `LLAMA_CPP` is the only one.
The three checks are green with 146 tests; the two Domain tasks are done.

Before writing the compile code, ran the build ai-ctl would run by hand, on the scratchpad clone:
the planned CMake options, Ninja, `-j 24`, with the available memory sampled every 2 s. Configure
took 3 s and the build 250 s for 356 steps; the available memory fell from 47.3 to 38.7 GiB at
worst, so 24 parallel jobs need about 9 GiB and the job count can default to the processor count.
The binary is 68 MB, the build tree 307 MB; `ldd` shows `libcudart.so.13`, `libcublas.so.13` and
`libcublasLt.so.13` resolved under `/usr/local/cuda-13.4` through `ld.so.conf`, and nothing else
from the toolkit. `llama-server --version` prints to stderr, `--list-devices` to stdout
(`CUDA0: NVIDIA GeForce RTX 5090 Laptop GPU (24075 MiB, 23798 MiB free)`), both captured as
fixtures with an excerpt of Ninja's output. The shallow clone made llama.cpp report `build 1`:
llama.cpp numbers a build by counting commits, which settles the clone as a full-history one. The
compilers wrote their messages in French: every command ai-ctl runs gets `LC_ALL=C`.

Fixed the host probe to skip a `g++` that resolves to `ccache` or `sccache`, so the real
`/usr/bin/g++` is found behind the wrapper. Moved `doctor`'s Toolchain checks into
`src/application/toolchain.rs`, whose `inspect` returns the checks and, when none fails, the
`Toolchain` a build compiles with: `nvcc`, the GCC `nvcc` accepts, and the `gcc` beside it, since
the same GCC compiles C, C++ and the CUDA host code. The probe fakes moved to
`src/application/fakes.rs`, shared by the use cases' tests. The domain gained the CMake options
(`cmake_flags`: the recipe's, then the builder's, every compiler named, `120a-real`,
`BUILD_SHARED_LIBS=OFF`, `GGML_NATIVE=ON`) and the prune rule.

Wrote the ports (`Sources`, `Cmake`, `BuildStore`, `Programs`) and the adapters:

- `src/infrastructure/git.rs` clones with `--filter=blob:none` into a hidden directory renamed when
  complete, fetches branches and moved tags, resolves a reference as a tag, a branch of the remote,
  then a commit, and fetches a full hash no branch holds; it checks out with `--force --detach`,
  cleans the tree with `git clean -ffdx`, and lists the tags of the commit. Its tests run against a
  repository made in a temporary directory and reached by `file://`.
- `src/infrastructure/cmake.rs` removes and recreates the build tree, runs configure and build with
  stdout and stderr merged into one pipe, writes every line to `~/.local/state/ai-ctl/logs/
  build-<project>.log`, reads Ninja's `[n/total]` or Make's `[ n%]` as the progress, and quotes the
  last 20 lines of the log when a step fails.
- `src/infrastructure/build_store.rs` keeps `builds/<project>/<id>/` with the binary, `build.json`
  and the log; a build is staged in a hidden directory and installed by a rename, `current` is a
  relative link replaced by a rename, a removed build is renamed out of sight before it is deleted,
  and the next build removes what an interrupted one left. `builds/.lock`, taken with
  `File::try_lock`, refuses a second `build`, `use` or `prune` at once.

The use case, `src/application/build.rs`, gates the build on `doctor`'s GPU check and Toolchain
checks — a failed check refuses it with its remedy, a warning does not — resolves `latest`, checks
out, names the build after the release tag pointing at the commit (so `--ref` with the hash of a
release gives the release's id), refuses an installed id without `--force`, compiles, stages,
smoke-tests (`--version` runs; `--list-devices` lists a CUDA device), installs, removes the build
tree, and makes the build current; the staged build is discarded when the smoke test fails.
`list`, `use` and `prune` work on the store alone. The presentation adds `ai-ctl build <project>
[--ref] [--force] [--jobs]` and the `list`, `use` and `prune` subcommands, with one line per step
and a progress line rewritten in place on a terminal.

First real run, `ai-ctl build llama.cpp`: it resolved `latest` to `b11538`, cloned 242 MB, built in
4 min 22 s with `/usr/bin/g++` 16.2.1, passed the smoke test, and installed
`~/.local/share/ai-ctl/builds/llama.cpp/b11538/` as current; the work tree was removed. The
installed binary reports `build 11538`.

Then the Toolchain Setup tasks. The machine exposes what they need cheaply: `/etc/os-release`
gives the release, `dnf repolist --enabled` the repositories, and `dnf repoquery --cacheonly
--repo <id>` answers from the cache in 0.1 s. With the exclusion in place it lists no driver
package of `cuda-fedora44-x86_64`; with `--setopt=disable_excludes=*` it lists 28 of them, so the
query sees exactly what `dnf install` would. dnf5 has no `--disableexcludes`. `rpm -qa
'cuda-toolkit*'` gives the installed meta-packages. The toolkit remedy is now numbered steps for the
running release: the repository (`addrepo`) and the exclusion (`setopt …exclude=`) are left out
when they are already there, and the install names `cuda-toolkit-<major>-<minor>` for the driver's
CUDA version. Two checks joined the Toolchain section: "CUDA repositories" warns about an enabled
`cuda-*` repository that offers driver packages, with the `setopt` command; "Toolkit packages" warns
about `cuda-toolkit` or `cuda-toolkit-<major>`, with the versioned install then the removal. A test
fake reproduces this machine before the upgrade (`FakeHost::before_the_upgrade`): the remedy is
then the three steps the user needed on 2026-10-09. `doctor` on this machine runs in 0.6 s, every
Toolchain check `ok`.

`doctor` gained a Builds section: for each recipe, the current build runs `--version`, which also
fails when its CUDA libraries are gone; a project the inventory runs without a build fails, and a
broken build the inventory does not run only warns. On this machine: `b11538, built 2026-10-09
20:48 UTC with CUDA 13.4.92`. The README documents `build`, and `docs/toolchain.md` lists Ninja and
ccache as optional build tools. `Cargo.toml` declares `rust-version = "1.89"`, for
`File::try_lock`.

Verified the acceptance criteria on this machine. `ai-ctl build llama.cpp --ref
1460533e0e159dd04e64b2b1f6a1e582b180e208`, the head of the `pr/29600-gg` branch, five commits ahead
of master and carrying no tag, built as `1460533e0e` in 1 min 9 s: ccache served 302 of the 341
compilations, since the source and build paths do not change between builds. It became current
(`was b11538`); `ai-ctl build use llama.cpp b11538` switched back in 2 ms, and `build prune --keep
0` removed the branch build, which is not current. The failure paths answer with one line each:
an unknown reference, an unknown project, `use` of a build that is not installed, and `build
llama.cpp` when `b11538` is already built — whose message suggested `build use` for a build that
was already current, now `is already built and current; --force builds it again`. Two forced
rebuilds of `b11538` took 13 and 19 s and replaced it in place.

Interrupted a forced rebuild with SIGINT to its process group, as Ctrl-C would; the signal landed
during the smoke test. No process was left, the lock was free, the installed `b11538` was intact and
current, and the staged build stayed hidden as `.staging-b11538-<pid>`, which `build list` and
`doctor` ignore; the next build removed it with the work tree. The first two attempts at this test
missed their process: one build ended in 13 s, before the signal, and `setsid` forked away from the
PID that was recorded, so the signal went nowhere while the build held the lock — a concurrent
`build use` was refused with `another ai-ctl is building or changing the builds`.

Updated `docs/architecture.md` (a line still said only `completions` existed) and `docs/stack.md`
§8 (a sixth setup step, the native build). Ticked the fifteen tasks and the six acceptance
criteria; the three checks are green with 244 tests and the relative links resolve. The README's
progress block is left to the closure.

The user asked to record the progress and commit, and to go on in a new session; the phase is not
closed. Committed the phase's code, fixtures and documents together with this report as
`feat(build): add native builds with list, use and prune`.

Where the next session starts: the closure of this phase (`close-phase`), once the user asks for
it. Two behaviours were chosen without the user and are open to reversal before then: a
successful build becomes current, and an installed id is rebuilt only with `--force`. The machine
holds the sources under `~/.local/share/ai-ctl/sources/llama.cpp` and one build, `b11538`, current;
no inventory is installed (`ai-ctl init` not run), so `doctor` fails on it alone. Phase 3 points the
gateway at `builds/llama.cpp/current/llama-server`.

Resumed in a new session to close the phase, at the user's request: read the phase file, this
report, the roadmap README and the phase 3 and 4 files. Before closing, asked the user about the
two behaviours chosen without them; the user kept both — a successful build becomes current, and an
installed id is rebuilt only with `--force`. The three checks were green before any edit (244
tests). Phase 2 changed no later phase file: what phase 3 inherits is in the Assessment.

---

## Decisions

- **The build toolchain has its own document, `docs/toolchain.md`**, with a dated record of every
  command run on this machine, newest first; each change to the driver, the toolkit, the compilers
  or their repositories adds an entry (stated in the document, and in the local `CLAUDE.md`). The
  repository must allow a fresh install of the build tools, or a version change, without
  friction. `doctor`'s remedies give the same commands as the document.
- **`doctor`'s toolkit remedy changes in this phase** (approved by the user): numbered steps, the
  NVIDIA repository of the running Fedora release read from `/etc/os-release` instead of a
  hard-coded `fedora44`, the driver exclusion, and the versioned toolkit package. `doctor` also
  warns about an enabled CUDA repository offering driver packages and about an unversioned toolkit
  meta-package.
- **The toolkit is installed by its versioned package**, `cuda-toolkit-<major>-<minor>`, never by
  the `cuda-toolkit` or `cuda-toolkit-<major>` meta-packages, which follow NVIDIA's newest release.
- **NVIDIA's CUDA repository is used with its driver packages excluded**
  (`nvidia-*,kmod-nvidia*,xorg-x11-*nvidia*,libnvidia-*,dnf-plugin-nvidia`); the driver comes from
  RPM Fusion only.
- **Builds pass the real compiler as `CMAKE_CUDA_HOST_COMPILER`**, never ccache's wrapper that
  `g++` resolves to on the `PATH` (`/usr/lib64/ccache/g++`).
- **This machine's toolchain at the end of the session**: CUDA 13.4.92 alone, from `cuda-fedora44`
  with the driver exclusion; `cuda-fedora43` disabled; driver 615.71.09 from RPM Fusion; `g++`
  16.2.1, which `nvcc` 13.4 accepts, with `g++-15` still installed.
- **`GGML_STATIC` is not used.** It links the whole executable with `-static`, and the driver has
  no static `libcuda` to link. Builds link `BUILD_SHARED_LIBS=OFF`, so llama.cpp and ggml are inside
  the binary, and load the CUDA libraries by soname (`libcudart.so.13`, `libcublas.so.13`) through
  `/usr/local/cuda`: a build survives a minor toolkit change and breaks when no toolkit of its
  major version is left. Each build records its toolkit version; `doctor`'s build checks (task
  "Add the current builds to `doctor`") report a current build whose libraries no longer resolve.
- **The toolkit rule is `toolkit_problem` in the domain**, shared by `doctor` and the build gate;
  the build tools' minimums are `Tool::minimum`.
- **Recipes are domain constants** (`src/domain/recipe.rs`), not infrastructure as the phase file's
  list of files suggested: a recipe is data, and the CMake flags generated from it are a rule
  tested without I/O. A later recipe is one more constant in `RECIPES`.
- **A build id is the release tag, or the first ten digits of the commit**; ids are directory
  names, so `/`, a leading `.` or `-`, and `current` are refused. Release tags are recognised by
  the recipe's pattern (`b` and digits only), never by git's glob alone.
- **`build.json` is versioned (`"format": 1`) and strict**: unknown fields are refused, like the
  inventory's unknown keys. Timestamps are UTC, RFC 3339 to the second.
- **A successful build becomes current; an installed id is rebuilt only with `--force`.** Taken by
  default, without asking the user, from the acceptance criterion that `build use` switches *back*
  to an older build; the user confirmed both at the closure.
- **One GCC compiles everything**: the newest `g++` `nvcc` accepts, passed as
  `CMAKE_CXX_COMPILER` and `CMAKE_CUDA_HOST_COMPILER`, and the `gcc` beside it as
  `CMAKE_C_COMPILER`; `CMAKE_CUDA_COMPILER` names the `nvcc` `doctor` checked. ccache is still used
  through ggml's `GGML_CCACHE`, never as a compiler.
- **The gate is `doctor`'s verdict**: the GPU check and the Toolchain checks; a failure refuses
  the build with the same remedies, a warning does not.
- **Sources are one blobless clone per project** under `~/.local/share/ai-ctl/sources/`, with the
  full history, which llama.cpp's build number needs; the file contents are fetched at checkout.
- **Builds use one job per processor** by default (`--jobs N` otherwise): 24 jobs peaked at about
  9 GiB on this machine.
- **The build tree is fresh for each build** (`~/.local/share/ai-ctl/work/<project>`), removed
  after a success and kept after a failure until the next build; ccache makes a rebuild cheap. The
  log of the last attempt is `~/.local/state/ai-ctl/logs/build-<project>.log`, and each build keeps
  its own copy.
- **The smoke test** runs the staged binary with `--version`, then with the recipe's device listing
  (`--list-devices` for llama.cpp), whose stdout and stderr must name a `CUDA<n>` device.
- **`doctor` reads dnf's cache only** (`--cacheonly`), staying fast and offline; its toolkit remedy
  depends on the repositories it finds, and with the repositories unreadable it gives every step.
- **`doctor`'s Builds section** runs each current build's `--version`; the remedy for a build that
  no longer runs rebuilds the same reference with `--force`.

---

## Files Changed

Computed against the start commit `feat(doctor): add doctor, init and the inventory` with `git diff -M --name-status` and the untracked
files (none). The phase's two commits, `docs: open phase 2 and document the build toolchain` and `feat(build): add native builds with list, use and prune`, and this closure are counted.

**Added**
- `docs/roadmap/on-progress/v2-rebuild/phase-2-native-build-report.md`
- `docs/toolchain.md`
- `src/application/build.rs`
- `src/application/fakes.rs`
- `src/application/toolchain.rs`
- `src/domain/build.rs`
- `src/domain/recipe.rs`
- `src/domain/timestamp.rs`
- `src/infrastructure/build_store.rs`
- `src/infrastructure/cmake.rs`
- `src/infrastructure/files.rs`
- `src/infrastructure/git.rs`
- `src/presentation/build.rs`
- `tests/fixtures/dnf-repolist-enabled.txt`
- `tests/fixtures/dnf-repoquery-cuda-driver-packages.txt`
- `tests/fixtures/git-ls-remote-tags.txt`
- `tests/fixtures/llama-server-list-devices.txt`
- `tests/fixtures/llama-server-version.txt`
- `tests/fixtures/ninja-build-excerpt.txt`
- `tests/fixtures/os-release-fedora44.txt`
- `tests/fixtures/rpm-cuda-toolkit-packages.txt`

**Modified**
- `Cargo.toml`
- `README.md`
- `docs/architecture.md`
- `docs/roadmap/on-progress/v2-rebuild/README.md`
- `docs/roadmap/on-progress/v2-rebuild/phase-2-native-build.md`
- `docs/stack.md`
- `src/application/doctor.rs`
- `src/application/mod.rs`
- `src/application/ports.rs`
- `src/domain/machine.rs`
- `src/domain/mod.rs`
- `src/infrastructure/command.rs`
- `src/infrastructure/host.rs`
- `src/infrastructure/mod.rs`
- `src/infrastructure/paths.rs`
- `src/main.rs`
- `src/presentation/cli.rs`
- `src/presentation/mod.rs`
- `tests/cli.rs`

---

## Problems And Deviations

- **The toolkit dependency is settled**: CUDA 13.4 installed on 2026-10-09; the user's first
  attempt failed because `doctor`'s two-line remedy did not say both lines were needed.
- **Three tasks added to this phase**, under Toolchain Setup: the `docs/toolchain.md` document and
  `doctor`'s toolkit remedy aligned with it (numbered steps, the repository of the running Fedora
  release with the exclusion, a versioned package), both at the user's request; and `doctor`
  warnings for a CUDA repository offering driver packages and for an unversioned toolkit
  meta-package, proposed after today's trap and kept by the user. One acceptance criterion added to
  match. 12 → 15 tasks.
- **The repository moved to `/code/ai/ai-ctl`**, which left a stale test binary pointing at the old
  path; fixed by rebuilding, no code change.
- **Recipes live in the domain**, not in `src/infrastructure/` as the phase file's list of files
  said (see Decisions).
- **The refusal of a build with CUDA 13.2 cannot be shown on this machine** any more, since 13.2 was
  removed: it is shown by the use-case test against the machine as it was
  (`FakeHost::before_the_upgrade`), and the gate runs the very checks `doctor` runs.
- **The first device-listing test used text written by hand**; replaced by the output captured from
  the first build.

---

## Changes To Later Phases

None: phase 2 changed no later phase file. What phase 3 inherits is listed in the Assessment.

---

## Assessment

The phase delivered what it promised and three tasks more. Most of its first day went to the
machine rather than the code: installing CUDA 13.4 exposed a two-line `doctor` remedy that did not
say both lines were needed, an unversioned toolkit meta-package, and an NVIDIA repository offering
a driver stack beside RPM Fusion's. Those became `docs/toolchain.md` and three `doctor` checks, so
the next toolkit change is a documented procedure instead of a session. The builder itself landed
in one pass once llama.cpp's build files had been read and a build run by hand: that reading
settled `GGML_STATIC` (impossible, no static `libcuda`), the full-history clone (llama.cpp numbers
builds by commit count), the release-tag pattern (`b*` also matches `backup/…`), and the ccache
wrapper that `g++` resolves to. Every failure path was run on the machine — refused references,
an installed id, Ctrl-C during a build — and left no partial version behind.

What phase 3 needs to know first:

- **The binary to run is `~/.local/share/ai-ctl/builds/llama.cpp/current/llama-server`**, where
  `current` is a relative link that `build use` replaces by a rename. A llama-swap `cmd` naming
  the link picks up a `build use` at the next model load with no regeneration; one naming the
  resolved build makes the "regenerate on `up` when the current build changed" task detect the
  switch. Phase 3 chooses, and the store's `current` port already answers which build is current.
- **No inventory is installed on this machine** (`ai-ctl init` never ran), so `doctor` fails on it
  alone and the configuration has nothing to generate from until it runs.
- **Builds load `libcudart.so.13` and the cuBLAS libraries by soname** through `ld.so.conf`, which
  every child of llama-swap inherits; `doctor`'s Builds section already reports a current build
  that no longer runs, and the gateway state is one more section beside it.
- **Patterns phase 3 can reuse:** `src/infrastructure/command.rs` runs every command with
  `LC_ALL=C`; `builds/.lock`, taken with `File::try_lock`, refuses concurrent changes; staging in a
  hidden directory then renaming leaves no partial state after an interruption.

