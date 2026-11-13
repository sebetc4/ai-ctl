# Claude Agent Guide: ai-ctl

A Rust CLI for local AI backends sharing one GPU: it diagnoses the machine, builds the ggml backends
natively, runs llama-swap as the single gateway and VRAM arbiter, and benchmarks the stack. It is a
production tool: robust error handling, predictable behaviour on failure, and tests that cover the
failure paths come before anything else. The project was called llm-ctl until 2026-10-09.

## Where Things Are

- **Current work**: the roadmaps under `private/` (contract in `.agent-conventions.toml`).
  Open: `history-reset`, which started the history again on 2026-11-13 from one root commit,
  the 0.3.0 release, and closes once the remote is published. 0.3.0 is the only release on
  GitHub, its entry holding 0.2.0's lines, its tag checked, built and published by the CI
  (`docs/release.md`).
  Each closed roadmap's `summary.md`, under `completed/`, says what it left open.
  When a roadmap is open, read its open phase file and report before working.
- **Documentation**: `docs/README.md` indexes it by domain, with its rules. Read a domain's
  document before changing that domain: `docs/gpu/llama-swap.md` before anything about backends,
  the gateway or VRAM; `docs/security.md` before the key, the gateway's address, the inventory's
  checks, a download or an image; `docs/release.md` before the workflow, one of its pins or a
  release. Change `docs/commands.md` with the command, and add an entry to
  `docs/toolchain.md`'s record for each change to the toolchain.
- **Working notes**: under `private/`, by domain, with the session notes, stay
  outside git; no tracked file cites them. Read a domain's notes before planning work on it.
- **Reference files**: `deploy/` — the shipped inventory, `inventory.toml` (`${MODELS_DIR}`
  placeholder, filled by `ai-ctl init`), and `opencode.json`.

## Constraints

- One RTX 5090 Laptop (24 GB VRAM) shared by every backend.
- Fedora 44 with an NVIDIA GPU is the only target platform.
- llama-swap is the only process that starts or stops a GPU backend; ai-ctl never launches a backend
  itself. ai-ctl runs llama-swap between `up` and `down`; no systemd unit.
- Native builds are the default runtime; official container images are entries for comparison.
- Never reimplement what llama-server or llama-swap do natively (model switching, caching,
  speculative decoding, APIs, routing). Before ai-ctl works around, patches or bounds a behavior
  of llama-swap or a backend, read that project's release notes since the version ai-ctl pins
  or builds: a newer release may do it natively.
- The agents' client, the command-line tool through which Claude Code's agents delegate tasks to
  the models, is a separate project; ai-ctl is not a client: it declares what each model can do,
  and never chooses one for a client.

## Conventions

- **Language**: all code, comments, documentation and commit messages in English.
- **Sources**: a tracked file cites no session's id and no path into a session's transcript or
  scratch folder, which no one but the user can open, on this machine alone: a fact's source is
  named by its date and its work, a commit by its hash. A payload captured among the fixtures is
  no citation.
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
  by a test or justified in the phase report; a script, which `cargo mutants` does not
  reach, takes mutants by hand only on logic whose failure is silent or irreversible, such
  as the review agents' guard. A new test that binds a port, takes a lock or runs two
  processes at once runs 30 times before its commit: one run cannot show a flaky test; one
  that only runs a command in a scratch folder, such as `git` in a scratch repository, runs
  once.
- **Reviews**: a roadmap phase that changes code ends with the review of its hunks by
  `reviewing-delivered-code`, its findings fixed in the phase before its closure. A roadmap that
  changed code ends with that skill's closing pass — the security pass, the dependency axis, the
  recorded mutants run again —, its findings fixed in its last phase, and no review after it. A
  full review runs only when the user asks for one.
- **Commits**: through `committing-changes`, by the `[git]` table of `.agent-conventions.toml`.
- **Releases**: by `docs/release.md`. Production runs the release's binary; `make bin` builds the
  working tree, for development.

## Checks

Every phase closes with the `checks` of `.agent-conventions.toml` green.
