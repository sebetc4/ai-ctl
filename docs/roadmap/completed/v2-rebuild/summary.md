# Summary — ai-ctl v2 Rebuild

---

## Where We Started

On 2026-10-09 the tool was llm-ctl v1: a Rust CLI that started one backend process per model,
served its own gateway with OpenAI and Ollama routes, kept a model registry, drove an SGLang
backend beside llama.cpp, and carried LoRA, draft and warm flags. By then llama-server did most of
this natively — presets, model switching, `--fit`, idle sleep, MTP speculative decoding, OpenAI
and Anthropic APIs — and the v2 study had settled on llama.cpp as the only LLM engine. A third of
v1 was dead weight, and its last phase sat uncommitted.

The machine was a laptop with one RTX 5090 (24 GB) shared by every backend, Fedora 44, driver
615.71.09, and a CUDA 13.2 toolkit, the release reported to produce gibberish with some quantizations in
llama.cpp builds. No
inventory existed; the models were loose GGUF files under `/ai/models/gguf/`, none of them the
ones the study chose; OpenCode and OpenWebUI pointed at v1's gateway on port 9000.

---

## Where We Landed

ai-ctl, on a new git history with v1 archived as a bundle outside the repository, is a production
CLI of ten commands. `init` installs the inventory, the one declarative file of backends,
models, flags and VRAM budgets everything else reads. `doctor` checks the GPU, driver, toolkit,
compilers, repositories, container tooling, power, model files, builds and gateway, each problem
with its remedy. `build` compiles llama.cpp natively for `sm_120a` into versioned builds with
`list`, `use` and `prune`. `gateway install | config`, `up`, `down`, `status` and `logs` run a
pinned llama-swap as the single entry point on `127.0.0.1:8080` and the only process that starts
or stops a model, its `matrix` computed from the VRAM budgets. `bench run`, `list` and `compare`
measure a model through that gateway and keep every run with the setup it measured.

On the machine: CUDA 13.4 from a versioned package with NVIDIA's driver packages excluded, the
native build `b11538`, llama-swap v263, both models pinned at checked commits and verified under
`/ai/models/llm/`, OpenCode and OpenWebUI working through the gateway, and a baseline of the main
model on the native build and on the official image. The three checks pass with 517 tests. The
documents that carry the knowledge are `docs/stack.md`, `docs/architecture.md`,
`docs/toolchain.md`, decision 0001 and three notes on directions for later roadmaps.

---

## What Each Phase Delivered

**Phase 0 — Reset & Skeleton.** v1 archived and verified, a new history from a skeleton of three
dependencies, four layers and one command, and the stack, architecture and agent guide rewritten.
The user's correction set the tone of what followed: ai-ctl is a production tool, not a learning
project.

**Phase 1 — Doctor & Inventory.** It opened on decision 0001 — llama-swap as the one gateway and
GPU arbiter, llama-server's router mode dropped, the tool renamed ai-ctl — and replaced presets
with the inventory. `doctor` and `init` landed with parsers tested on outputs captured from the
machine. Two premises proved false and were settled by reading the machine: Fedora 44 has no
`powerprofilesctl`, and the newest GCC `nvcc` accepts is written in the toolkit's own header.

**Phase 2 — Native Build.** Recipe-driven, versioned ggml builds, gated on `doctor`'s verdict,
smoke-tested, installed by rename, and safe under Ctrl-C. Installing CUDA 13.4 exposed an
unversioned meta-package and an NVIDIA repository offering a second driver stack, which became
`docs/toolchain.md` and three `doctor` checks. Reading llama.cpp's build files ruled out
`GGML_STATIC` and required full-history clones.

**Phase 3 — Gateway.** llama-swap v263 installed against a pinned digest, its configuration
generated from the inventory and pinned by snapshot tests, and run on demand. A swap between the
two models takes about 10 s with never both on the GPU, and `down` returns once NVML shows the
memory free. The phase grew where llama-swap's real behaviour required it: a gateway killed with
SIGKILL leaves its models on the GPU, so `up` refuses while such processes remain. It also fixed a
flaky test suite whose children inherited other tests' files.

**Phase 4 — Bench.** `bench run` measures generation by context depth up to the limit, a fresh
prefill, ten-turn sessions, MTP acceptance per request, the GPU's state and the gateway's cost,
and saves each run whole. The baseline found the native build and the official image equal in
generation and prefill, the image 50 to 80 ms slower on an agent turn's first token: the native
build stays the default. Most of the work went into making a measure mean what it says, against
three llama-server behaviours found in live runs. The clients were checked through the gateway,
which closed phase 3's last criterion.

---

## What We Learned

**Read the upstream at the pinned version before writing against it.** llama-swap's YAML has no
escape for `$` and ignores unknown keys; llama.cpp cannot link statically against the driver;
llama-server stops one token short of a full context and saves a slot's context into its RAM
cache before taking a new prompt. None of it is in the documentation, each was found in the
source, and each became a decision or a test.

**Ask the machine, not a table.** The GCC limit is in `host_config.h`, the power profile on D-Bus,
the usable VRAM in NVML's reserved memory, the CDI specification's driver in the libraries it
mounts. Every probe that read the source of truth survived a toolkit or driver change; every
remedy that wrote a static artefact — a hand-generated CDI specification, an unversioned toolkit
package — went stale at the next one.

**A benchmark's inputs must be identical when speculative decoding is on.** What the model
generates sets MTP's acceptance, hence its speed: prompts tagged per run made two runs of the same
build differ by 13 % at one depth. Same prompts, `cache_prompt: false` where a request must miss
the cache, and an unmeasured request that pays the server's cache housekeeping keep each measure
to what it names.

**Refuse rather than degrade silently.** A model started beside an earlier gateway's leftover
would have been moved partly to the CPU by `--fit` without an error; refusing `up` and naming the
PID cost a check, the silent case would have cost a wrong baseline.

**Check what a client sends, not what it is configured to send.** OpenWebUI 0.12 adds 7.1K tokens
of built-in tools to every chat and a task request ahead of the first answer; OpenCode sends tool
arguments as strings the model's template would refuse, saved only by llama-server's conversion.
Settings pages show none of it; the server's log does.

**A clean slate drops what only lived in the deleted files.** v1 had learned that OpenWebUI in a
podman container reaches the host at `10.0.2.2`; the reset deleted the note, and v2's stack
document gave the wrong address until a live check. Porting deliberately has to include the
operational notes, not only the code.

**Pin and verify every download; install scripts check nothing.** GGUF files against the Hub's
SHA-256 with their chat template read, llama-swap against its digest, OpenCode's release against
GitHub's: the vendors' one-line installers verify none of it.

---

## What We Are Leaving Open

- Why the official image is 5 to 15 % slower than the native build on short prompts over a long
  context; CUDA architectures, CUDA's version and memlock are ruled out, the host compiler, the
  linking and the CPU backend are not.
- Where OpenWebUI's short request ahead of a chat's first answer comes from.
- `disableKeepAlives: true` on every llama-server entry, until a build includes the fix of
  llama.cpp #29680.
- Never run for real: a llama-swap outliving SIGTERM, a container's process left on the GPU after
  `down`, and a gateway exiting before it answers; the use cases' tests cover them.
- The bench's plan defaults, meant to be revised after the baseline; and runs made before
  `fix(bench): set the slot's context aside before each fresh request` of 2026-10-12, which differ from later ones on the time to a scenario's first token.
- In a Messages stream, llama-server's order of blocks is not Anthropic's; a strict orchestrator
  client may refuse it.
- Directions recorded, not decided: a model library managed by ai-ctl
  (a working note on a model library), vLLM, whose prefill criterion now has its figures
  (a working note on vLLM), and models larger than the VRAM (a working note on models larger than the VRAM). The
  audio and image backends that decision 0001 prepares, and the orchestrator, a separate project.
