# Phase 1 Report: Doctor & Inventory

**Phase:** [phase-1-doctor-and-inventory.md](phase-1-doctor-and-inventory.md)
**Start Commit:** `chore: restart llm-ctl v2 on a clean base` of 2026-10-09

---

## Work Log

### 2026-10-09

Phase opened right after the closure of phase 0, whose phase file and report were read first. No
restructuring was pending. Carried over: llm-ctl targets production; `deploy/models.ini` holds a
`${MODELS_DIR}` placeholder for `init`; `main.rs` is the composition root; v1's system probes are
in the archive bundle; this machine still has `nvcc` 13.2, GCC 16.2.1 by default with GCC 15, and a
driver supporting CUDA 13.4.

Before any code, the user raised the future of the tool: an orchestrator called by Claude agents to
use local sub-agents, and backends for other domains (whisper.cpp, audio.cpp, Voxtral through
vLLM-Omni, Fish Speech, stable-diffusion.cpp), with the question of one controller per domain and
the risk of two controllers loading models into the same VRAM. Studied the backends (audio.cpp's
server and tasks, the image script calling stable-diffusion.cpp's `/v1/images/generations`, the
`polarsteps-tts` project's vLLM-Omni and Fish Speech servers) and llama-swap's current features,
`matrix` and `/upstream/:model_id` included. The analysis and the decision are recorded in
`docs/decisions/0001-gpu-arbitration-through-llama-swap.md`.

The user accepted the architecture, chose the name ai-ctl over local-ai-ctl (the latter reads as a
controller for the LocalAI project; both were free on crates.io), kept this roadmap to the LLM
domain, and asked whether dropping llama-server's router mode would affect OpenWebUI and OpenCode.
It does not: they point at llama-swap on the same port with the same model IDs. Restructured the
roadmap accordingly (README 1.2.0) and renamed the project in the code and the documents.

Resumed in a new session: read the phase file, this report, the phase 0 report, decision 0001,
`docs/architecture.md`, the relevant sections of `docs/stack.md`, and v1's NVML and command probes
from the archive bundle, cloned into the session scratchpad. Probed the machine read-only before
writing code. Two premises of the phase proved wrong:

- `powerprofilesctl` does not exist here: Fedora 44 installs `tuned-ppd` (tuned 2.28) instead of
  `power-profiles-daemon`. Both serve the same D-Bus interface, and
  `busctl get-property org.freedesktop.UPower.PowerProfiles /org/freedesktop/UPower/PowerProfiles
  org.freedesktop.UPower.PowerProfiles ActiveProfile` answers `s "performance"`; the probe reads it
  there.
- v1 hard-coded the newest GCC `nvcc` accepts (`MAX_CUDA_HOST_GCC = 15`). The toolkit states it
  itself: `targets/x86_64-linux/include/crt/host_config.h` holds `#if __GNUC__ > 15` followed by an
  `#error`. The probe reads that file under the toolkit `nvcc` belongs to, so no version table has
  to follow CUDA releases.

The rest matches the phase's expectations: `nvcc` 13.2.86 under `/usr/local/cuda-13.2`, `gcc` 16.2.1
with `gcc-15` beside it, CMake 4.3.0, git 2.56.0, podman 5.8.7, `nvidia-ctk` 1.20.1 with a CDI
specification listing `nvidia.com/gpu=all`, `libnvidia-ml.so` 615.71.09, no process on the GPU,
the AC adapter online, `hf` 1.14.0 installed.

The user approved committing the restructuring on its own before the phase's code, and chose the
inventory's shape (see Decisions). The restructuring went in as `refactor: rename llm-ctl to ai-ctl and adopt llama-swap as the gateway`, with the three checks
green.

Checked where a newer toolkit would come from: the machine's CUDA repository is
`cuda-fedora43`, which stops at 13.2, while NVIDIA's `fedora44` repository carries
`cuda-toolkit-13-3` and `cuda-toolkit-13-4`. `/usr/local/cuda` is an `alternatives` link in auto
mode whose priority is the version (13.2 → 132), so installing 13.4 moves it without another step;
the remedy is two `dnf` commands. Also checked ggml's CUDA `CMakeLists.txt` for the minimums:
`cmake_minimum_required(VERSION 3.18)` and `120a-real` added from CUDA 12.8.

Wrote the domain (diagnostics, machine facts with `pick_host_compiler`, the inventory and its
validation), the ports and the `doctor` and `init` use cases against test doubles, then the
adapters: XDG paths, a command runner (C locale, no stdin, 15 s timeout), the host probes, the NVML
probe and the inventory file with `deploy/inventory.toml`. Captured the machine's outputs as
fixtures under `tests/fixtures/` (`LC_ALL=C`: `gcc` answers in French otherwise; the GPU UUID in
the CDI list anonymised). Two slips on the way: `UsedGpuMemory` lives in
`nvml_wrapper::enums::device` in 0.13, and a test expected 15 shared flags where `models.ini` holds
13. The session was interrupted with 102 tests green and the presentation layer not started.

Resumed in a third session: read the phase file and this report, then the code already written,
since the previous session ended before the presentation layer. Added `ai-ctl init` and
`ai-ctl doctor [--json]`, with the adapters handed to `presentation::run` through a closure built
in `main.rs`, so `completions` needs neither `HOME` nor NVML.

First run on this machine: the CUDA 13.2 toolkit fails with the two-command remedy, exit code 1, in
0.14 s. It also warned that `gnome-shell` held 4 MiB of VRAM: the display already runs on the iGPU,
and in hybrid mode the compositor keeps a minimal context on the NVIDIA GPU. A warning that always
fires teaches the user to ignore warnings, so processes under 64 MiB are now reported as
background contexts in an `ok` line; a display on the NVIDIA GPU holds hundreds of MiB.

Ran `init` end to end in the scratchpad: a missing models directory is refused before anything is
written, a relative one is resolved, a second `init` refuses and leaves the file untouched, and
`doctor` then lists every missing model file with its `hf download` command. The JSON output holds
the same fifteen checks as the text; a closed pipe (`doctor | head`) is not an error. The binary
tests needed `HOME` besides `XDG_CONFIG_HOME`, since the paths module resolves all three
directories.

Deleted `deploy/models.ini` after comparing it with the inventory, and rewrote the references to it
in `docs/stack.md`, `docs/architecture.md`, `CLAUDE.md` and the README; the README documents `init`,
`doctor` and the inventory format, and a test parses its inventory example with the real loader.
The three checks are green with 118 tests, and all sixteen tasks are ticked.

The user asked to close the phase. The three checks passed again, and the four acceptance criteria
were verified once more on the built binary in a scratch configuration: CUDA 13.2 fails with the
upgrade remedy, the JSON output holds the fifteen checks of the text output, a second `init`
refuses, and both models' missing files come with their `hf download` command.

---

## Decisions

- **llama-swap is the single gateway and GPU arbiter** (decision 0001). ai-ctl installs it, generates its configuration and runs it between `up` and `down`.
- **The tool is renamed ai-ctl.** Crate, binary and documents follow; the repository directory and the archive keep their names until the user renames them.
- **llama-server's router mode is dropped**: one swap layer only; each model is a plain `llama-server` entry in the gateway configuration.
- **An inventory replaces `models.ini`**: backends and models with flags and VRAM budgets, extensible to audio and image backends without a breaking change.
- **This roadmap delivers the LLM domain only**; audio and image backends and the orchestrator are follow-ups recorded in decision 0001.
- **The inventory's shape** (chosen by the user): arrays of tables `[[backends]]` and `[[models]]` with an `id`, so uniqueness is a validation rule; the flags shared by every model of a backend kind live in `[defaults.<kind>]` (`[defaults.llama-server]` replaces `models.ini`'s `[*]`), so native and image runtimes run with the same flags and the bench compares runtimes, not settings; a model carries `repo` and `dir` for the `hf download` remedy, its files as a map from server flag to a path in `dir`, and `models_dir` is written once at the top as `${MODELS_DIR}`. Phase 3 generates the gateway configuration from this shape.
- **The power profile is read over D-Bus**, through `busctl`, not with `powerprofilesctl`: the interface is served both by `power-profiles-daemon` and by `tuned-ppd`, the one Fedora 44 installs.
- **The newest host GCC `nvcc` accepts is read from the toolkit's `crt/host_config.h`**, not from a table in ai-ctl. Phase 2 passes the compiler it finds as `CMAKE_CUDA_HOST_COMPILER`.
- **Crates added**: `nvml-wrapper` 0.13 (it loads `libnvidia-ml.so` at run time, so a machine without the driver gets a failed check instead of a binary that cannot start), `toml` 1.1, `serde`, `serde_json` and `thiserror`; `tempfile` for tests.
- **Toolchain floors**: CUDA 12.8 (ggml adds `120a-real` from it), CMake 3.18 (ggml's CUDA backend), git 2.18 (`git ls-remote --sort` for phase 2's release tags). CUDA 13.2 and a toolkit newer than the driver's CUDA version fail.
- **The toolkit remedy follows the driver**: NVIDIA's `fedora44` repository, then `cuda-toolkit-<major>-<minor>` for the driver's CUDA version (13.4 here).
- **Idle VRAM tolerates 64 MiB per process**; a process whose usage the driver does not report counts as holding VRAM.
- **Probe commands** run in the C locale, without input, and are killed after 15 s; a command that is absent is a missing tool, never an error of ai-ctl.
- **`doctor`'s contract**: exit code 1 when any check fails, warnings and skips do not change it; `--json` gives `healthy`, `summary` and `sections[].checks[]` with `name`, `status`, `detail` and `remedy`. Phases 2 and 3 add checks to the same report.
- **Inventory rules beyond the phase's three**: `version = 1`; `models_dir` absolute; unknown keys rejected; paths relative and contained in the models directory; `ttl = 0` rejected (absence means never unload); `host`, `port`, `ctx-size` and `model`, and any file flag, cannot be set as flags, because phase 3 sets them; every rule broken is reported together.
- **`init` writes atomically**: a temporary file published with a hard link, which fails if the inventory appeared meanwhile; the models directory must exist and is stored as an absolute path.
- **Adapters are built lazily**: `main.rs` hands `presentation::run` a closure, called only by the commands that need the probes or the inventory.

---

## Files Changed

Computed against the start commit `chore: restart llm-ctl v2 on a clean base` with `git diff -M --name-status` and the untracked
files. The restructuring commit `refactor: rename llm-ctl to ai-ctl and adopt llama-swap as the gateway` falls inside this phase and is counted.

**Added**
- `deploy/inventory.toml`
- `docs/decisions/0001-gpu-arbitration-through-llama-swap.md`
- `docs/roadmap/on-progress/v2-rebuild/phase-1-doctor-and-inventory-report.md`
- `docs/roadmap/on-progress/v2-rebuild/phase-1-doctor-and-inventory.md` — renamed from `phase-1-doctor-and-presets.md`, rewritten beyond git's rename detection
- `docs/roadmap/on-progress/v2-rebuild/phase-3-gateway.md` — renamed from `phase-3-server-lifecycle.md`, rewritten beyond git's rename detection
- `src/application/doctor.rs`
- `src/application/init.rs`
- `src/application/ports.rs`
- `src/domain/diagnostics.rs`
- `src/domain/inventory.rs`
- `src/domain/machine.rs`
- `src/infrastructure/command.rs`
- `src/infrastructure/host.rs`
- `src/infrastructure/inventory_file.rs`
- `src/infrastructure/nvml.rs`
- `src/infrastructure/paths.rs`
- `src/presentation/doctor.rs`
- `tests/fixtures/busctl-active-profile.txt`
- `tests/fixtures/busctl-not-activatable.txt`
- `tests/fixtures/cmake-version.txt`
- `tests/fixtures/gcc-15-dumpfullversion.txt`
- `tests/fixtures/git-version.txt`
- `tests/fixtures/host_config-13.2-excerpt.h`
- `tests/fixtures/nvcc-13.2.txt`
- `tests/fixtures/nvidia-ctk-cdi-list.txt`
- `tests/fixtures/nvidia-ctk-version.txt`
- `tests/fixtures/podman-version.txt`

**Modified**
- `Cargo.lock`
- `Cargo.toml`
- `README.md`
- `docs/architecture.md`
- `docs/roadmap/on-progress/v2-rebuild/README.md`
- `docs/roadmap/on-progress/v2-rebuild/phase-2-native-build.md`
- `docs/roadmap/on-progress/v2-rebuild/phase-4-bench.md`
- `docs/stack.md`
- `src/application/mod.rs`
- `src/domain/mod.rs`
- `src/infrastructure/mod.rs`
- `src/lib.rs`
- `src/main.rs`
- `src/presentation/cli.rs`
- `src/presentation/completions.rs`
- `src/presentation/mod.rs`
- `tests/cli.rs`

**Deleted**
- `deploy/models.ini`
- `docs/roadmap/on-progress/v2-rebuild/phase-1-doctor-and-presets.md` — see its rename above
- `docs/roadmap/on-progress/v2-rebuild/phase-3-server-lifecycle.md` — see its rename above

---

## Problems And Deviations

- **`powerprofilesctl` is absent on Fedora 44**, which installs `tuned-ppd`. The power task and the parser-test task now name the D-Bus property read through `busctl`; `docs/stack.md` §8 recommended the missing command and is corrected.
- **The host compiler task names its source**: `crt/host_config.h` rather than a version table; the parser-test task lists it.
- **The machine's CUDA repository stops at 13.2** (`cuda-fedora43`). Nothing to fix in ai-ctl: the remedy adds the `fedora44` repository. Installing 13.4 is still the user's step before phase 2's first real build, as its phase file states.
- **The first `Idle VRAM` check warned on the compositor's 4 MiB.** Fixed with the 64 MiB tolerance (Decisions).
- **The paths module needs `HOME` unless all three `XDG_*` variables are set.** Kept: a session without `HOME` is not a real case, and `completions` does not resolve paths.
- **The README's progress block lagged behind the ticked tasks until this closure**, which recomputed it per the Totals invariant; `progress.py --check` reported the gap in the meantime.
- **Every task is done and every acceptance criterion holds**; no task moved to a later phase.
- **Decision 0001 still names `deploy/models.ini`**, as the file the inventory replaces; a decision record is not rewritten.

---

## Changes To Later Phases

- **Restructuring approved by the user on 2026-10-09** (decision 0001), applied directly:
  - this phase: renamed Doctor & Presets → Doctor & Inventory; the presets tasks replaced by four inventory tasks (model, `deploy/inventory.toml`, `init`, file checks), the README task widened to the inventory format; 14 → 16 tasks. Files renamed with the report.
  - `phase-2-native-build.md`: the builder becomes recipe-driven for ggml projects, llama.cpp shipped first; a recipe task added and the acceptance criteria name the project; 11 → 12 tasks.
  - `phase-3-server-lifecycle.md` → `phase-3-gateway.md`: objective changed from an in-house server lifecycle over two runtimes to installing, configuring (`matrix` computed from VRAM budgets) and running llama-swap; tasks rewritten; 13 tasks.
  - `phase-4-bench.md`: measurements go through the gateway, llama-server-only endpoints through `/upstream/<model>/`; runs record the inventory entry; 12 tasks.
  - README: scope, decisions and phase list revised.
- `phase-2-native-build.md`, Dependencies: the toolkit 13.4 comes from NVIDIA's `fedora44` repository (the machine's `cuda-fedora43` stops at 13.2), and the probes and `pick_host_compiler` that phase 1 provides for the build gate are named.
- `phase-2-native-build.md`, Constraints: two added — the toolkit verdict becomes one domain rule shared by `doctor` and the build gate, and builds get a streaming runner instead of the probes' buffered one with its 15 s timeout.

---

## Assessment

The phase delivered what it promised, on a wider base than planned. The restructuring at its start
(decision 0001) replaced the presets with an inventory that every later phase reads, and renamed
the tool; the code then landed in one pass: the diagnostics and inventory in the domain, `doctor`
and `init` as use cases tested against doubles, and adapters whose parsers are tested on outputs
captured from this machine. Two premises of the phase file were wrong — `powerprofilesctl` is
absent on Fedora 44, and the GCC limit of `nvcc` need not be a table — and both were settled by
reading what the machine itself exposes. The first real run caught a noisy warning that no test
would have shown.

What phase 2 needs to know first:

- **The machine still has CUDA 13.2**, and only the user can install 13.4: `ai-ctl doctor` prints
  the two commands. No real build can be accepted before that.
- **The toolkit rules live in `src/application/doctor.rs`** (13.2 refused, 12.8 minimum, not newer
  than the driver) as private functions. The build gate needs the same verdict: move the rule into
  the domain and call it from both places, rather than copying it.
- **`pick_host_compiler` and the `HostProbe` port** already give the compiler to pass as
  `CMAKE_CUDA_HOST_COMPILER`; `g++-15` is the answer on this machine.
- **The command runner is for probes**: it buffers the output and kills a command after 15 s. A
  build needs a runner that streams to a log and has no such timeout.
- **The inventory's `source = { build = "llama.cpp" }`** names the project a recipe must match;
  `doctor`'s report takes the build checks as one more section.
