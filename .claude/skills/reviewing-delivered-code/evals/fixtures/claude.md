# Claude Agent Guide: ai-ctl

A Rust CLI for local AI backends sharing one GPU: it diagnoses the machine, builds the ggml backends
natively, runs llama-swap as the single gateway and VRAM arbiter, and benchmarks the stack. It is a
production tool: robust error handling, predictable behaviour on failure, and tests that cover the
failure paths come before anything else. The project was called llm-ctl until 2026-10-09.

## Where Things Are

- **Current work**: the roadmaps in `docs/roadmap/` (contract in `.agent-conventions.toml`). None
  is open: `audio-image-backends`, which added the audio and image backends, closed on
  2026-10-30. Each closed roadmap's `summary.md`, under `completed/`, says what it left open.
  When a roadmap is open, read its open phase file and report before working.
- **Documentation**: `docs/README.md` indexes it by domain, with its rules. Read a domain's
  document before changing that domain: `docs/gpu/llama-swap.md` before anything about backends,
  the gateway or VRAM; `docs/security.md` before the key, the gateway's address, the inventory's
  checks, a download or an image. Change `docs/commands.md` with the command, and add an entry to
  `docs/toolchain.md`'s record for each change to the toolchain.
- **Working notes**: `notes/`, by domain, with the session notes in `notes/sessions/`, stay
  outside git; no tracked file cites them. Read a domain's notes before planning work on it.
- **Reference files**: `deploy/` — the shipped inventory, `inventory.toml` (`${MODELS_DIR}`
  placeholder, filled by `ai-ctl init`), and `opencode.json`.
- **v1 code**, for porting: the history before the v2 rebuild is archived in
  `/code/rust/projects/llm-ctl-v1-archive/llm-ctl-v1.bundle` (`main` = last v1 commit,
  `archive/v1-wip` = the uncommitted Phase 6-S work). Read it with
  `git clone <bundle> <scratch dir>`. Port deliberately; never copy v1's model of the world
  (one backend process per model managed by the tool, a model registry, a gateway of its own).

## Constraints

- One RTX 5090 Laptop (24 GB VRAM) shared by every backend.
- Fedora 44 with an NVIDIA GPU is the only target platform.
- llama-swap is the only process that starts or stops a GPU backend; ai-ctl never launches a backend
  itself. ai-ctl runs llama-swap between `up` and `down`; no systemd unit.
- Native builds are the default runtime; official container images are entries for comparison.
- Never reimplement what llama-server or llama-swap do natively (model switching, caching,
  speculative decoding, APIs, routing).
- The orchestrator for agents is a separate project; ai-ctl is not a client.

## Conventions

- **Language**: all code, comments, documentation and commit messages in English.
- **Layers**: `presentation → application → domain` and `infrastructure → application → domain`;
  `main.rs` is the only composition root: it builds the adapters into `Services`, and each
  presentation handler assembles its use case from them and picks the exit code. Presentation
  never imports infrastructure.
- **Ports**: a trait only when two implementations exist or a use case needs a test double.
- **Errors**: `thiserror` in the domain when callers branch; `anyhow` with `.context()` elsewhere;
  no `.unwrap()` outside tests.
- **Dependencies**: add a crate only in the phase that uses it; prefer the standard library. Drive
  `git`, `cmake`, `podman`, `nvcc` and llama-swap as commands or over HTTP.
- **Tests**: code and scripts test-first: a failing test, watched failing on its assertion
  (not on a compile error), then the code; a bug fix starts with the test that reproduces
  it. Other artifacts take the proof that fits them — an eval for a skill's text, a probe
  on this machine for a behavior of llama-swap, llama-server or the toolkit, the command
  that checks a configuration —, never a test that reads a text back. Parsers are tested
  on outputs captured by a probe, never written from memory, each listed with its capture
  in `tests/fixtures/README.md`; no test depends on timing, a GPU or the network. Each
  commit's code diff goes through `cargo mutants --in-diff`, and a mutant it misses is killed
  by a test or justified in the phase report. A new test that spawns a process, binds a port
  or takes a lock runs 30 times before its commit: one run cannot show a flaky test.
- **Commits**: Conventional Commits (`feat(doctor): …`, `fix(build): …`, `docs: …`), made through
  `committing-changes` by the `[git]` table of `.agent-conventions.toml`. A roadmap opened since
  2026-10-26 lives on its branch `roadmap/<name>` until its closure.
- **Releases**: the version lives in `Cargo.toml`, and `CHANGELOG.md` gives each version an entry,
  headed `## X.Y.Z — unreleased` while in work. A release dates the entry, `## X.Y.Z — YYYY-MM-DD`,
  with `Cargo.toml` at X.Y.Z, on `main`, and `committing-changes` tags that commit `vX.Y.Z`.
  Production runs a binary built at a release tag; `make bin` builds the working tree, for
  development.

## Checks

Every phase closes with these green:

```bash
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
```
