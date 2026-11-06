# Phase 4 Report: Structure & Refactoring

**Phase:** [phase-4-structure-and-refactoring.md](phase-4-structure-and-refactoring.md)
**Start Commit:** `docs(v2-review-fixes): close phase 3, performance` of 2026-10-17

---

## Work Log

### 2026-10-17

Phase opened at `docs(v2-review-fixes): close phase 3, performance`, the commit that closed phase 3, in the session that closed it. Read
before working: phase 3's file and report in full. No restructuring was pending. What carries
into this phase:
- each batch is committed once the five checks are green;
- phase 3's Assessment names what batches 40, 42, 49 and 50 meet: `GatewayProcesses`'s process
  table is now `signal`, `stat`, `in_container` and `program`, with the fakes' process tree;
  `command.rs`'s `execute` was rewritten by batch 32; the `CheckedOut` line cuts a commit;
  `Sources` is `fetch`, `release_tags` and `checkout`; the use cases' waits take an injected
  `now` clock;
- `docs/commands.md`, the Makefile, and one line each of `CLAUDE.md` and `README.md` hold the
  user's uncommitted work, kept out of this phase's commits; batch 50 lists the first two in
  the Repository Layout once the user has committed them.

Resumed in a new session. Read before working: this phase's file and report, phase 3's
Decisions, Problems, Changes and Assessment, and the review's findings 30 and 59 and Fix Plan.

Batch 36 (findings 30, 59). Four domain tests, watched failing on their assertions against
stubs that returned nothing: `Run::scenarios` on the full test run and on one without depth
and session; `common_scenarios` and `Run::gpu_over` on a run of every scenario beside one of
depth alone, and over depth and session without the idle reading before the first scenario;
`comparable_counters` on two runs of the same scenarios, one without counters, then on runs
of different scenarios; `OverheadResult::medians` on both ways and on one way empty.

The fix: `domain/bench.rs` gains `Run::scenarios`, `Run::gpu_over`, which filters readings on
`Scenario` values, and two free functions beside `arrival_gaps` and `drafts`:
`common_scenarios` and `comparable_counters`, which gives each run's `server()` counters, or
`None` when the runs' scenarios differ. `compare.rs` keeps the tables and the formats: its
`scenarios` helper became `names`, the drafts table reads the counters once per run instead of
once per row, and the GPU table reads `gpu_over(&common_scenarios(runs))`. `added` is the
difference of `medians`; `render_run` takes its gateway and direct columns from `medians` and
its added column from `added`, so the subtraction is written once. A misplaced doc comment,
`gpu`'s sitting above `holds`, went back to `gpu`. `medians` gives `None` when one way is
empty, where `render_run` showed the other way's median alone: no run holds that, since the
overhead scenario sends `OVERHEAD_REQUESTS` requests each way and a failed scenario keeps
nothing. The comparison's rendered tests hold unchanged. The five checks green, 652 tests.
Committed as `refactor(bench): move the rules for comparing runs into the domain`.

Batch 37 (findings 61, 29). No new test: nothing observable changes, since inspecting never
pauses, and the existing tests are the net. `Services::configurator` builds the `Configurator`
that `up`, `down`, `status`, `logs`, `bench run` and `gateway config` use. `Doctor` gains
`pause` and `now` fields, set from `Services` by presentation and to the fakes' `advance` and
`now` by its tests; its `lifecycle()` still builds its own `Configurator` from its ports, the
one finding 61 says a `Services` helper cannot reach. `std::thread` no longer appears in
`src/application` outside the tests. The five checks green, 652 tests. Committed as `refactor: build the configurator in Services and inject doctor's pause`.

Batch 38 (finding 31). Names only, so no new test. The port is `Compiler`, after its role, and
its adapter `Cmake`, after its technology, as `Podman` and `Curl` are; `Builder::compiler`,
`Services::compiler`, `main.rs` and the tests' `FakeCompiler` follow, and the adapter's tests
call their value `compiler`. The contract stays CMake-shaped (`CompileJob`, `-D` flags), and
the port's doc still says it compiles a CMake project. No document named the old types;
`FakeHost::cmake` and `Tool::Cmake`, CMake's version, are not the port and keep their names.
The five checks green, 652 tests. Committed as `refactor(build): name the compile port after its role`.

Batch 39 (finding 32), option (a) as the review settled it. The two tests that downcast
`Refused` were rewritten first to assert on the message, and passed on the code as it stood,
so the message is proven unchanged: the CUDA 13.2 test now checks that one check line is
indented under the first line, and that it is the toolkit's, quoted whole as the run printed
it after a first guess of mine had cut it short; the missing driver's test checks that the
GPU's line follows the first. Then the type went: the gate does `bail!(refusal(&refused))`,
`refusal` a private function that writes the same text, and the test of the layout calls it.
No document named the type. The five checks green, 652 tests. Committed as `refactor(build): refuse a build with an anyhow error`.

Batch 40 (finding 25). No new test: the methods and their doubles move, nothing else, and the
existing tests are the net. `GatewayProcesses` became three ports: `GatewayProcess` (`lock`,
`record`, `liveness`, `forget`, `config`, `start`, `signal`, `log`, `log_tail`, `read_log`),
`ProcessTable` (`stat`, `in_container`, `program`) and `PortCheck` (`is_free`). The first
name was to be `Gateway`, after `docs/architecture.md`'s example, until the domain's
`inventory::Gateway`, the inventory's gateway section, turned up in doctor's tests; the port
takes its module's name instead. `LocalGateway` implements `GatewayProcess`; the `/proc` reads
and the port check, which touch none of its files, went into two stateless adapters in the
same module, `Procfs` and `TcpPorts`, so `main.rs` builds each once. `Bench` and `build
prune` hold `GatewayProcess` alone; `Lifecycle` and `Doctor` hold all three. `Services` names
the release store `releases`, as the use cases did, since `gateway` now holds the
`GatewayProcess`. `FakeProcesses` was split into `FakeGateway`, the process tree's
`FakeProcesses` and `FakePorts`; the tests' accesses moved to the fake that owns each field,
mapped by name, the three sets of names being disjoint. The adapter's tests call `Procfs` and
`TcpPorts`. No document named the old port. The five checks green, 652 tests. Committed as
`refactor(gateway): split GatewayProcesses by role`.

Batch 41 (finding 26). No new test, for the same reason. `HostProbe` became `ToolchainProbe`
(`cuda_toolkit`, `host_compilers`, `tool_version`, `is_file`, `fedora_release`,
`cuda_repositories`, `toolkit_packages`), `CdiProbe` (`cdi_devices`, `cdi_specs`) and
`PowerProbe` (`ac_power`, `power_profile`). `SystemHost` implements all three, as the plan
says: unlike batch 40's, its state, the `PATH` and the sysfs and CDI paths, serves every
role, so `main.rs` builds one per port. `Builder` holds `toolchain`, `Bench` `power` alone,
and `Doctor` the three, its container checks reading `tool_version` and its inventory check
`is_file` from the toolchain probe, as the review placed them. `FakeHost` became
`FakeToolchain`, which keeps `healthy` and `before_the_upgrade`, `FakeCdi` and `FakePower`.
Doctor's tests hand one machine to `diagnose` some thirty times, so they gained a test-local
`Host` of the three fakes, `healthy_host()` building it; their accesses were mapped by field,
and the two `cdi: &host.cdi` fields the mapping caught were put back. No document named the
old port. The five checks green, 652 tests. Committed as `refactor(doctor): split HostProbe by role`.

E2. This machine holds seven saved runs, five of format 1 and one of format 2 on a native
build, each embedding `build.json` with its `format: 1`, and one of format 1 on an image.
Before any change, `bench list` and `bench compare` of all seven were saved as the baseline.
Tests first: a run is written in format 3, its build without `format`; a run of format 4 is
refused with the formats now read; an embedded build of any `format` but 1 is refused with a
message of the run store's own. Watched failing on their assertions: the format read 2, the
refusals gave the old texts. The test that formats 1 and 2 read the `build.json` they
embedded passed already; it guards the change.

The fix: `run_store.rs` has its own `BuildJson`, the fields of `build.json` with an optional
`format`, written in none and read in runs of formats 1 and 2, where it must be 1. The run's
format went to 3, read with 1 and 2. `build_store::to_json` and `from_json` lost their last
outside caller and became private. After the change, `bench list` and `bench compare` of the
seven runs gave the baseline's output byte for byte, both exiting 0. The five checks green,
653 tests. Committed as `feat(bench): keep the saved run's own copy of the build`.

Batch 42 (findings 47, 48). Two tests first, watched failing on stubs that returned empty
texts: `Tool::install` for each of the four tools, the nvidia-ctk note kept; and
`command::checked` and `command::not_found`, the missing program's message, a failure's own,
and a command that ran handed back whatever its exit. The fix: `checked`, podman's, moved to
`command.rs` taking the missing program's message, and `not_found(tool)` writes "<command>
not found; run `<install>`". Git, the gateway store's tools, curl, `kill`, `SystemPrograms`
and podman go through `checked`, each keeping its own handling of a command that ran;
download keeps its text under `{:#}` with `with_context`. CMake's spawn error, the toolchain's
CMake and Git checks, and doctor's Podman and toolkit remedies take `Tool::install`;
`minimum_check` lost its remedy parameter. Podman's tests assert against the message as a
literal of their own, as the task said. Two `RunError` matches stay, by design: `pids`
without podman is no container, and `host.rs` maps to `Probed`. No text changed. The five
checks green, 655 tests.

Batch 42 was committed as `refactor: map command errors and name install commands once`. Batch 43 (finding 49). One test first, watched failing on stubs: `request_ok` gives a 200's
body, quotes the error body's message for another status, and fails when nothing listens;
`open` fails when nothing listens. The fix: `http.rs` gains `request_ok`, `open`, the
`send` that fails when nothing listens, and a public `unexpected` that writes the non-200
error through `error_message`. `server_api::call`, `gateway_api::unload_all` and `running`
take `request_ok`; `stream_log` and `chat_api::stream` take `open`, `stream_log` keeping its
own 400 and `unexpected` for the rest; `healthy` keeps `request` and its `Option`. One text
changes, as the finding asked: a gateway API answer other than 200 quotes the error body's
message where it quoted the raw body. `chat_api`'s "answered" stays its own: it reads the
head of a streamed reply, and its caller adds the method and URL. The five checks green, 656
tests.

Session closed at the user's request, once batch 43 was finished. The session resumed the
phase opened at `docs(v2-review-fixes): open phase 4` and did the Structure section whole, then batches 42 and 43, each
committed once the five checks were green; the user's uncommitted `CLAUDE.md`, `README.md`,
`Makefile` and `docs/commands.md` stayed out of every commit. Batch 40 departs from the
plan's wording, `LocalGateway` implementing one port and `signal` staying with the gateway,
recorded under Decisions and Problems And Deviations for the user to see.

**Where the next session starts:**
1. Batch 44 (finding 50): one lookup of a project's current build in `application::build`,
   with the three-way outcome (`None`, `Missing`, `Installed`), for doctor, bench and the
   configurator.
2. Batches 45 to 49 in their order, then batch 50 last, then the phase's closure.
- Batch 50 lists `docs/commands.md` and the `Makefile` in the Repository Layout once the user
  has committed them, and changes CLAUDE.md's Layers line, where the user holds uncommitted
  changes: ask before batch 50.
- Uncommitted on purpose, the user's work: `CLAUDE.md`, `README.md`, `Makefile`,
  `docs/commands.md`.

Resumed in a new session. Read before working: this phase's file and report, and the review's
findings 50 to 64 and Fix Plan.

Batch 44 (finding 50). Two tests first, watched failing on their assertions against stubs:
`current` on `FakeBuilds` gives `Installed`, `Missing` and `None`, and fails with the store;
`Current::installed` gives the build, or the texts with their remedy for the other two. The
fix: `application::build` gains `Current` and `current`, which reads the `current` link and
looks its id up among the installed builds. Doctor matches the three outcomes, keeping its
statuses and remedies; bench and the configurator call `installed`, so they share one text,
the configurator's, with its remedy, where bench's had none. `known_recipe` became
`pub(super)`, and the configurator takes its text for a backend without a recipe, which
lists the projects ai-ctl builds; the one test that pinned the old text follows it. Bench's
`current_build` went. The five checks green, 658 tests. Committed as `refactor(build): look up a project's current build once`.

Batch 45 (finding 51). The test of `found_gpu` first, and the five tests that pin the sites'
texts moved to the new ones; all six watched failing on their assertions against a stub that
gave an empty error. The fix: `found_gpu`, beside `GpuProbe` in `ports.rs`, takes a reading
and gives its GPU or the base text, "no NVIDIA GPU found; run `ai-ctl doctor`" or "cannot
read the GPU: <error>". Each site adds what it needed the GPU for as context: the
configurator "cannot size the models to the GPU's memory", the bench "cannot record the run's
GPU", `down`'s wait "cannot wait for the GPU's memory to be released", and `warn_unread` the
consequence it was given, which now names the GPU since it opens the warning ("not waiting
for the GPU's memory: cannot read the GPU: …"). The texts change, as the finding's common
base implies; the presentation test that renders a configuration's reason follows them. The
bench's setup still reads the GPU twice, once in `prepare` and once for the machine, as it
did. The five checks green, 659 tests. Committed as `refactor: turn a GPU reading into a GPU or an error once`.

Batch 46 (finding 52). No new test: nothing observable changes, and the tests of `logs
--follow`, `logs <model>` and the bench without a gateway are the net. `log`, `model_log` and
`Bench::listen` call `live_record`, which `build prune` already used, with their texts
unchanged. The other reads of `Liveness::Running` stay: `running`, which the finding leaves
out since it needs the other livenesses; `observe`, which reports a stale record; and the
follow loop and `down`'s wait, which watch a record they already hold. The five checks green,
659 tests. Committed as `refactor(gateway): read the live gateway record in one place`.

Batch 47 (finding 53). The test of `check_id` first, and the two use cases' tests moved to
the inventory's text; the four, the inventory's own id test among them, watched failing on
their assertions against a stub that accepted every id. The fix: `domain::inventory::check_id`
is public and returns `InventoryError::InvalidId`, which the inventory collects and `logs
<model>` and `bench run` return with `?`. The use cases' text becomes the inventory's: "model
id `<id>` must start with a letter or a digit and hold only ASCII letters, digits, `.`, `_`
and `-`", where they quoted the id with `{:?}`. `is_valid_id` lost its outside callers and
became private. The five checks green, 660 tests. Committed as `refactor(inventory): check a model id with the inventory's rule and text`.

Batch 48 (findings 56, 57, 58). Three tests first, of `Liveness::describe`, of
`Serving::running_on` with `models_list`, and of `named` with `kill_command`, and the five
tests that pin a site's list moved to the one format; the eight watched failing on their
assertions against stubs that gave empty texts. The fix: `Liveness::describe`, beside
`Liveness` in `ports.rs`, says what became of a recorded gateway, for doctor's Gateway check
and the `Forgotten` event. In `application::lifecycle`, `Serving::running_on` gives "running
on http://<listen> (PID <pid>)" and `models_list` the "<id> (<state>)" list, which doctor and
`status` use, each keeping its own words for no model and for an API that does not answer.
`named` became public and gives one format, "PID <pid> (<name>, <n> MiB)", the memory left
out when the driver reports none; `kill_command` gives the `kill <pids>` the four sites
quote. `up`, `down`, `status` and doctor's GPU leftovers and Idle VRAM use them, and doctor's
`process_list` went. The texts that change: doctor's lists, which read "<name> (pid <pid>,
<n> MiB)", and `up`'s and `down`'s, which gain each process's memory. The five checks green,
663 tests. Committed as `refactor(gateway): describe the gateway's state in one place`.

Batch 49 (findings 54, 55, 60, 63, 64). No new test: no text and no order changes, and the
tests of `build list`, `prune`, the build's progress and summary lines and `gateway install`
are the net. `domain::build::newest_first` is the comparator `to_prune` and `list` sort by;
`recipes` gives one project's recipe or every one, for `list` and `prune`;
`CommitId::short` slices `SHORT_COMMIT` digits, for `from_commit` and presentation's three
lines; toolchain's `fix` is `pub(super)` and doctor's copy went; `current_line`, in
`presentation/mod.rs`, writes "Current build: …" and "Current gateway: …" with their "(was
…)" tail. A search for each copy finds one definition. The five checks green, 663 tests.
Committed as `refactor: share five small helpers`.

Before batch 50, asked the user about their uncommitted `CLAUDE.md`, `README.md`, `Makefile`
and `docs/commands.md`, which the batch lists or edits: "tu peux tout commit". Committed as
they stood, apart from this phase's work, as `docs: add the command reference and a Makefile`.

Batch 50 (findings 28, 35). Documents, so the proof is a reading against the code, not a
test. `docs/architecture.md` now says that `main.rs` builds the adapters into `Services` and
`run` asks for them only when a command needs one, that each handler assembles its use case
from `Services`, `Services::configurator` once for the six commands that share it; names
`GpuProbe`, `Compiler` and `GatewayApi` with `NvmlProbe`, `Cmake` and `LlamaSwapApi` as its
examples; gives every command's use case in a table; says that presentation picks the exit
code, 1 for a failed doctor check and 3 for `status` without a gateway, and that `main.rs`
prints an error and exits with 1; cites the fixtures of `tests/fixtures/` and their index,
where it cited `nvidia-smi` and `GET /models`, and says NVML is called by no test. Its Testing
section gains the four conventions phases 1 and 2 settled, and its Repository Layout
`.claude/` with its agents and skill, the `Makefile`, `docs/commands.md`, `docs/reviews/`,
`docs/security.md`, `tests/fixtures/` and `tests/snapshots/`, `docs/` in alphabetical
order. `main.rs`'s module doc and CLAUDE.md's Layers line say the same split. Each claim was
checked against the code: the seven agents, the podman scripts' `[ "$*" = … ]`, NVML's tests,
which call none of it. A search for `nvidia-smi`, `/models`, `LlamaSwap`, "hands them" and
"sets the exit" finds nothing in the three files. The five checks green, 663 tests.

---

## Decisions

- **Doctor takes the lifecycle's `pause` and `now` from `Services`, not a whole `Lifecycle`.**
  A `Lifecycle` handed in would hold the same ports as `Doctor`'s own fields, two references
  a test would have to keep equal. Phase 3's rule holds: a use case's waits take the injected
  clock.
- **`GatewayProcesses` is split into `GatewayProcess`, `ProcessTable` and `PortCheck`, and
  `signal` belongs to `GatewayProcess`.** Every signal goes to the gateway's PID (`Signal` is
  "a signal for a gateway to stop"), and the fake gateway's liveness follows the signals it
  received: in the process table, the two fakes would share their state. The process table is
  read-only. Batch 41 splits `HostProbe` the same way, one port per role.
- **The `/proc` table and the port check are stateless adapters, `Procfs` and `TcpPorts`,
  beside `LocalGateway`.** They read none of its files, so one instance of each does, where a
  `LocalGateway` behind each port would have been built three times.
- **One list of GPU processes, always with the memory each holds.** The finding had `named`
  take the memory as an option; with one format and no option, `up` and `down` name each
  process's memory beside the total `up` gives, and every site reads the same. The `kill`
  command is a function of its own, `kill_command`, since doctor's Idle VRAM lists processes
  it gives no command for.
- **A saved run keeps its own copy of the build's fields, in format 3, without `build.json`'s
  `format`.** The run's format versions its copy; a run of formats 1 or 2 holds `format: 1`,
  the only `build.json` format there was. A new `build.json` format now leaves the runs alone,
  and a change to the run's copy takes a run format of its own.

---

## Files Changed

**Added**
- `Makefile` — the user's work, committed at their request in `docs: add the command reference and a Makefile`
- `docs/commands.md` — the user's work, committed at their request in `docs: add the command reference and a Makefile`
- `docs/roadmap/on-progress/v2-review-fixes/phase-4-structure-and-refactoring-report.md`

**Modified**
- `CLAUDE.md` — the Layers line (batch 50) and the phase's closure in Current work; its Commands line is the user's, committed in `docs: add the command reference and a Makefile`
- `README.md` — the user's `docs/commands.md` line, committed in `docs: add the command reference and a Makefile`
- `docs/architecture.md`
- `docs/roadmap/on-progress/v2-review-fixes/README.md`
- `docs/roadmap/on-progress/v2-review-fixes/phase-4-structure-and-refactoring.md`
- `src/application/bench.rs`
- `src/application/build.rs`
- `src/application/doctor.rs`
- `src/application/fakes.rs`
- `src/application/gateway_config.rs`
- `src/application/lifecycle.rs`
- `src/application/ports.rs`
- `src/application/toolchain.rs`
- `src/domain/bench.rs`
- `src/domain/build.rs`
- `src/domain/inventory.rs`
- `src/domain/machine.rs`
- `src/infrastructure/build_store.rs`
- `src/infrastructure/chat_api.rs`
- `src/infrastructure/cmake.rs`
- `src/infrastructure/command.rs`
- `src/infrastructure/download.rs`
- `src/infrastructure/gateway_api.rs`
- `src/infrastructure/gateway_process.rs`
- `src/infrastructure/gateway_store.rs`
- `src/infrastructure/git.rs`
- `src/infrastructure/host.rs`
- `src/infrastructure/http.rs`
- `src/infrastructure/podman.rs`
- `src/infrastructure/run_store.rs`
- `src/infrastructure/server_api.rs`
- `src/main.rs`
- `src/presentation/bench.rs`
- `src/presentation/build.rs`
- `src/presentation/compare.rs`
- `src/presentation/gateway.rs`
- `src/presentation/lifecycle.rs`
- `src/presentation/mod.rs`
- `tests/gateway_lifecycle.rs`

---

## Problems And Deviations

- **Batch 40: `LocalGateway` implements one port, not three, and `signal` is not in the process
  table.** The plan's task and the review's fix said `LocalGateway` implements all three and
  listed `signal` with the process table; the reasons are under Decisions. The roles are those
  of the task: the gateway's state, the process table, the port check.
- **Batch 48: `Serving::address` is `Serving::running_on`, and `Serving::models_list` a free
  function, `models_list`.** The text the first gives, "running on http://… (PID …)", is not
  an address; the second lists a non-empty slice of models, which a `Serving` holds as an
  `Option` its callers match first. `named` takes no option, per Decisions.
- **Batches 43, 44, 45, 47 and 48 change texts, where the phase's Out of Scope allows batch
  48's alone.** Each finding asks for one text where there were two or more, so one site at
  least changes its words: a gateway API answer other than 200 (43), the configurator's for a
  backend without a recipe and bench's for a build not installed (44), every GPU reading's
  (45), `logs <model>`'s and `bench run`'s for an invalid id (47). No exit status and no
  behaviour changes.
- **No task moved, and every acceptance criterion holds.** At closure, `bench list` and `bench
  compare` of this machine's seven saved runs both exited 0.

---

## Changes To Later Phases

None: this phase changed no later phase's file.

---

## Assessment

The phase did what it set out to do: its 16 tasks fixed the review's structure, refactoring
and documents findings 25, 26, 28 to 32, 35, 47 to 61, 63 and 64, and the defect E2, and every
acceptance criterion holds. The two wide ports are split by role, `Bench` holding the gateway's
port and the power probe alone; the rules for comparing runs live in the domain; presentation
builds the configurator once, from `Services`; the compile port is `Compiler` and its adapter
`Cmake`; the build gate refuses with `bail!`; a saved run keeps its own copy of the build, in
format 3, and the seven runs of this machine still read. Each copy the refactoring findings
cite has one definition, and `docs/architecture.md`, `main.rs` and CLAUDE.md describe the
structure the code has. The five checks are green, with 663 tests.

Two departures from the plan's wording are recorded above: batch 40 keeps `signal` with the
gateway's port, and batch 48 names its methods `running_on` and `models_list` and lists every
GPU process with its memory. Batches 43 to 48 changed error and status texts wherever a
finding asked for one text in place of several.

What phase 5 needs to know first:
- the review's scope is the roadmap's whole diff, phases 0 to 4; `docs/commands.md` and the
  `Makefile`, the user's, entered the repository in this phase's `docs: add the command reference and a Makefile`;
- `GatewayProcess` still holds ten methods, of which `Bench` calls `record`, `liveness` and
  `config`, and `build prune` those and `lock`: one role, the gateway's process and its files,
  but a wide port;
- the bench's setup reads the GPU twice, in `prepare` and for the run's machine, where phase
  2's rule has `up`, `down`, `status` and doctor read it once;
- the texts that batches 43 to 48 changed are pinned by their tests; no document quoted them.

