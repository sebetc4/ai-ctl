# Architecture

ai-ctl is a single Rust crate split into four layers with a strict dependency rule. The layering
keeps each concern testable on its own: the use cases that touch the GPU, processes, the gateway
and the network run in tests against doubles of their ports.

ai-ctl sits beside the gateway, not in front of it: llama-swap serves every request and starts or
stops the backends; ai-ctl builds the backends, generates the gateway's configuration and runs the
gateway ([gpu/llama-swap.md](gpu/llama-swap.md)).

---

## Layers

```
presentation ───┐
                ├──► application ──► domain
infrastructure ─┘

main.rs (composition root) ──► every layer
```

| Layer | Location | Holds | May depend on |
|---|---|---|---|
| Domain | `src/domain/` | Entities and rules of the stack: values, states, validations. No I/O. | nothing in the crate |
| Application | `src/application/` | Use cases, and the ports (traits) they need for what lies outside the process | `domain` |
| Infrastructure | `src/infrastructure/` | Adapters implementing the ports: NVML, external commands, files, HTTP | `application`, `domain` |
| Presentation | `src/presentation/` | Command-line parsing, dispatch, rendering | `application`, `domain` |

`main.rs` is the **composition root**: the only place that knows every layer. It builds the
infrastructure adapters into `presentation::Services`, and passes the parsed command line to
`presentation::run` with the function that builds them, which `run` calls only for a command that
needs an adapter: `completions` works on any machine. Each command's handler then assembles its
use case from `Services`, which holds each adapter as its port, and the clocks;
`Services::configurator` builds the configuration use case that `up`, `down`, `status`, `logs`,
`bench run` and `gateway config` share. Presentation never imports infrastructure.

---

## Ports

A port is a trait defined in `application` and implemented in `infrastructure`. One exists only
when it earns its place:

- **two implementations exist**; or
- **a use case needs a test double** for something that does I/O, such as the GPU, the gateway's
  API or an external command.

Every port so far exists for the second reason: each has one implementation in `infrastructure`
and its doubles in the tests. A choice between two kinds, such as a backend's source, a native
build or a container image, is an enum of the domain, not a port.

Otherwise a use case calls a plain function or struct. No trait is written "in case".

Naming: a port is named after its role (`GpuProbe`, `Compiler`, `GatewayApi`), an adapter after its
technology (`NvmlProbe`, `Cmake`, `LlamaSwapApi`). A port stays small: one responsibility, so that a
use case holds only the ports it calls.

---

## Path of a Command

```
argv ──► clap (presentation::cli) ──► presentation::run ──► Services, built by main.rs
     ──► handler assembles its use case ──► use case (application) ──► ports ──► adapters
     ◄── domain values ◄──────────────────────────────────────────────────┘
     ──► handler renders text or JSON and picks the exit code
```

| Command | Use case |
|---|---|
| `init` | `application::init::init` |
| `doctor` | `Doctor` |
| `build` | `Builder` |
| `build list`, `build use`, `build prune` | `list`, `use_build` and `prune` of `application::build` |
| `gateway install` | `Installer` |
| `gateway config` | `Configurator` |
| `up`, `down`, `status`, `logs` | `Lifecycle` |
| `bench run` | `Bench` |
| `bench list`, `bench compare` | `saved` and `load` of `application::bench` |
| `completions` | none: it stays inside presentation |

`build`, `gateway install`, `up`, `down`, `status` and `bench run` report their steps through a
callback the presentation renders as it goes.

Presentation picks the exit code: `1` when `doctor` finds a failed check, `3` when `status` finds
no gateway running, `0` otherwise (`docs/commands.md`). A command that fails returns its error to
`main.rs`, which prints it and exits with `1`.

---

## Errors

- **Domain**: `thiserror` enums, where a caller has to branch on the error.
- **Application and infrastructure**: `anyhow::Result`, with `.context(...)` at each boundary that
  adds meaning.
- **Presentation** returns a command's error; **`main.rs`** prints it as `error: {err:#}` (the
  whole context chain) and exits with `1`. No `.unwrap()` outside tests.

---

## Concurrency

The crate is synchronous. Where one task must run beside another, a `std::thread` does it inside
the adapter: the bench reads the GPU every 250 ms on a thread of its own, stopped through a channel
(`NvmlMonitor` in `src/infrastructure/nvml.rs`). `tokio` arrives only if a phase needs many
concurrent I/O operations, with `#[async_trait]` only where a port must be async.

---

## Testing

| Layer | How |
|---|---|
| Domain | Unit tests in the module |
| Application | Use cases run against test doubles of their ports |
| Infrastructure | Parsers and adapters tested on outputs captured on this machine, in `tests/fixtures/`, each listed with its capture in `tests/fixtures/README.md`: `nvcc`, `podman`, `/proc`, llama-swap's `/running` and `/logs/stream`, llama-server's `/props`, `/tokenize`, `/apply-template` and `/metrics`…; integration tests where the real dependency is cheap. The GPU is read through NVML, which no unit test calls; the binary's tests of `doctor` and `status` load it, and assert nothing that depends on it |
| Presentation | Rendering tests in the module; command-line parsing and the binary's commands in `tests/` |

The tests follow four conventions:

- A double fails through a field named after its method, `<method>_fails`, with an error worded
  like the real adapter's: a `bool`, or what picks the call that fails, such as the id whose
  removal fails. A state that a later call changes, such as a `current` link that names nothing
  until `set_current` replaces it, keeps a name of its own.
- `Cmake` and `Podman` hold the program they run as a field, and `SystemHost` the search path
  it finds `nvcc`, `g++`, `rpm` and `busctl` through. Their tests run a script that
  `command::script` writes under `command::exclusive()`, which keeps the other tests from forking
  while the script is written; the script checks the arguments it is given. The other adapters run the real program on what
  the test builds: `git` on a scratch repository, `curl` on local files, `sha256sum` and `tar`
  on an archive of the test's, `kill` on a child of the test; `dnf`, `nvidia-ctk` and the tools'
  `--version` run from `PATH`, and their parsers are tested on captures. A failure the real
  program cannot be made to give, such as a refused `kill`, is reached through the use case's
  test double.
- `ScriptedGpu` gives one GPU reading per call, the last one for good, for a use case that reads
  the GPU more than once, such as `down` waiting for the GPU's memory.
- A test written for a path no test covered is proven by its mutant: it passes on the code, and
  fails once the code is changed the way the test guards against.

The gateway configuration generated for the shipped inventory is compared with snapshots in
`tests/snapshots/`, since llama-swap ignores a key it does not know: after a deliberate change,
`UPDATE_SNAPSHOTS=1 cargo test --test gateway_config` rewrites them, and the diff is read before
it is committed.

`tests/gateway_lifecycle.rs` runs `up`, a chat request through the gateway and `down` with the
installed llama-swap and the real process, API and inventory adapters, in a scratch state
directory; it checks that nothing but `/health` answers without the API key, and that the model's
own server holds the key too. It has no libtest harness (`harness = false` in `Cargo.toml`): llama-swap runs the test
binary itself as the model's llama-server, recognised by its `--alias` argument, and a fake GPU
lists those fake llama-servers alive, so `down` waits for them as it waits for real ones. It is
skipped, with the reason printed, when no llama-swap is installed or when port 5800, llama-swap's
first model port, is taken.

No test depends on wall-clock timing, on a GPU, or on the network: the gateway test waits on real
processes only within the bounds `up` and `down` give them, and on the loopback.

---

## Repository Layout

```
.
├── .claude/
│   ├── agents/          The reviewer and verifier agents of the review skill
│   └── skills/          The code review skill, reviewing-delivered-code, with its evals
├── .github/
│   ├── scripts/         check_release.py, the release's guard, with its tests
│   └── workflows/       release.yml: the checks and the build at each tag, the release
│                        (docs/release.md)
├── Cargo.toml
├── Makefile             Shortcuts for the stack's daily commands; `make` lists them
├── assets/bench/        The bench's corpus, compiled into the binary
├── deploy/              Reference files of the stack: the shipped inventory, opencode.json
├── docs/                The documentation, by domain; docs/README.md is its index
├── rust-toolchain.toml  The Rust toolchain that checks and builds ai-ctl, here and in the CI
├── src/
│   ├── main.rs          Composition root
│   ├── lib.rs
│   ├── domain/
│   ├── application/
│   ├── infrastructure/
│   └── presentation/
└── tests/               Integration and command-line tests
    ├── fixtures/        Outputs captured on this machine; README.md lists each with its capture
    └── snapshots/       The gateway configurations generated for the shipped inventory
```
