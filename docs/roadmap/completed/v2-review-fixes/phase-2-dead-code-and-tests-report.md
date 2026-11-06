# Phase 2 Report: Dead Code & Tests

**Phase:** [phase-2-dead-code-and-tests.md](phase-2-dead-code-and-tests.md)
**Start Commit:** `docs(v2-review-fixes): close phase 1, bugs and error handling` of 2026-10-16

---

## Work Log

### 2026-10-16

Phase opened at `docs(v2-review-fixes): close phase 1, bugs and error handling`, the commit that closed phase 1. Read before working: phase 1's file
and report in full. No restructuring was pending. What carries into this phase:
- each batch is committed as soon as the five checks are green;
- the tests are written against phase 1's code. The fakes already hold `FakeRuns`,
  `FakeMonitor::lost_from`, the started gateway's `wait_failures`, `FakeModelServer::ctx_size`
  and the test stores' broken `current` links;
- batch 24's slowdown tests take `Slowdown::policy` and `GpuReading::clock_events`;
- finding 67's test exists since batch 6 and kills its mutant, so batch 16 adds none for it;
- `CLAUDE.md`, `README.md`, `Makefile` and `docs/commands.md` hold the user's uncommitted
  work, kept out of this phase's commits.

Batch 14 (findings 36–45). Finding 36's compiler check was reproduced first, as a script in
the session's scratch directory:
- copy `src/`, `assets/`, `deploy/` and the manifest;
- make the crate binary-only: `main.rs` declares the four layers, `ai_ctl::` becomes `crate::`,
  `lib.rs` and the `[[test]]` target go;
- turn every `pub` item and field into `pub(crate)`;
- run `cargo check --bin ai-ctl`.

The first attempt reported nothing because the copy did not compile: the module declarations
came before `main.rs`'s inner doc comment, `use crate::presentation::{self, …}` clashed with
the module, and `include_str!` needed `assets/` and `deploy/`. The script now stops on any
error instead of reading silence as success.

On `docs(v2-review-fixes): close phase 1, bugs and error handling`, the check named the eight methods of findings 36 to 38 and 41 to 45. A second
pass, without the `PartialEq` derive of `Summary`, `GpuSummary` and `TurnSummary`, named the
fields of findings 39 and 40: `count` and `max`, and `readings`. `TurnSummary` had to lose its
derive as well, since it compares `Option<Summary>`.

The removal, each test rewritten on what production reads:
- `model_files`: its test keeps the `model_dir` assertion, renamed; the inventory file's test
  asserts `main.files` and `model_dir`;
- `cuda_architectures`: the build test reads `cmake_flags[CUDA_ARCHITECTURES]`;
- `nvcc()` and `c_compiler()`: the toolchain tests read `cmake_flags(&LLAMA_CPP, …)`;
- `Sha256::as_str`: `to_string()` in two tests, `gateway_store.rs` and `gateway.rs`, the
  second missed by the finding;
- `Corpus::len` and `::is_empty`: the tests read `text`, which wraps, so `text(0, 6)` of five
  tokens ends with the first again;
- `Timestamp::unix`: the tests compare timestamps, and the leap-cycle test starts from
  946,684,800, checked against `2000-01-01T00:00:00Z`;
- `CommandLine::words`: three tests flatten `lines()`;
- `Summary.count` and `.max`: five assertions, three of them in `domain/bench.rs` tests the
  finding did not list, now read `mean`, `min` or `stddev`;
- `relative_spread` and its test, with the mutant `relative_spread_signed`'s code;
- `GpuSummary.readings`: the run's summary asserts the thermal share, one reading in four.

559 tests pass, two fewer: `relative_spread`'s and `cuda_architectures`'s. Both checks now
report nothing. The five checks are green. Committed as `refactor(domain): remove the API only tests call`, with the phase's opening.

Batch 15 (finding 46), test-first. To reach the `Probed::Missing` arm, the probe had to run a
`busctl` a test controls. `SystemHost` already finds `nvcc` and `g++` through its own `PATH`,
so `busctl` is found the same way, by a new `find`; no `busctl` on it reads `Missing`, as the
`NotFound` error did. The test, written first: a `busctl` script on the host's `PATH` checks
it is asked the UPower name alone, then answers in three ways:
- a profile: `Found`;
- the fixture's "not activatable" on stderr: `Missing`;
- "Access denied": `Failed`.

It failed on its first assertion, `Found("performance")` for `Found("balanced")`: the probe ran
this machine's `busctl`.

The fix: one name, no loop. The `Missing` arm returns at once, and any other failure is
`Failed`. The `net.hadess` assertion of `is_service_absent` is rewritten with the UPower name,
since dbus-daemon gives that wording.

The mutant `power_profile_no_fallback` is adapted to the new code: its `old`, the arm returning
`Probed::Missing`, becomes an arm returning `Probed::Failed(output.failure())`. Under it the
test fails, `Failed("… not activatable")` for `Missing`. Mutants are now run by a script in
the scratch directory, which copies the tree, applies `old` → `new` once, and runs `cargo test`.

On this machine, `ai-ctl doctor` reads "Power profile: performance"; `busctl --system list`
shows tuned-ppd (PID 1731) serving both names. The five checks are green, with 560 tests.
Committed as `fix(host): ask the power profile of the UPower service alone`.

Batch 16 (findings 66, 67, 68). These are test findings: each new test passed on the code at
once and was then run under its mutant.
- Finding 66: `kill_stops_a_process` sends `Signal::Kill` to a plain `sleep 60`, without a
  trap, and asserts "signal: 9 (SIGKILL)". Under `signal_kill_sends_term` it reads
  "signal: 15 (SIGTERM)".
- Finding 68: the start test's script writes field 5 of `/proc/$$/stat`, its process group,
  to the log, and the log must hold the child's PID. Under `no_process_group` the group is the
  test runner's.
- Finding 67: no new test. Batch 6's test fails under `liveness_deleted_binary_not_running`,
  run again here with the other two.

The five checks are green, with 561 tests. Committed as `test(gateway): send SIGKILL and check the gateway's process group`.

Batch 17 (finding 65), a test finding. `the_current_link_switches_between_installed_builds`
now lists the builds once the link exists: `b1` and `b2`, sorted since `read_dir` gives no
order, and no unreadable entry. It passed at once. Under `builds_reads_current_link` it fails
on the last assertion: the link is read as a build directory, and its `build.json`, which
describes `b1`, is reported unreadable under the name `current`. The five checks are green.
Committed as `test(build): list the builds while a current link exists`.

Batch 18 (findings 72, 69, 70, 71). Failure switches first, one `bool` a method, named as
`FakeApi::unload_fails` already was, so that batch 40 can move each with its port:
- `FakeProcesses`: `forget_fails`, `descendants_fails`, `program_fails`, `tail_fails`;
- `FakeContainers`: `exists_fails`, `pull_fails`, `pids_fails`;
- `FakeApi`: `health_fails`.

Each error reads like the real adapter's. Then ten tests, all passing at once on the code:
- `up` starts nothing and records nothing when a pull fails, after the `Pulling` event, or
  when an image cannot be looked for;
- a start timeout ends "(last error: …)" with the health check's error;
- an unreadable log ends the timeout message "; cannot read /state/logs/gateway.log: …";
- a record that cannot be removed is a warning beside the error, and stays;
- `down` fails when the GPU is lost while its memory is released, `Missing` and `Failed`;
- containers or processes that cannot be listed are left out with their warnings;
- a program that cannot be read is left out with a warning, once for each process that is not
  a container's;
- a log of `LOG_CHUNK` + 10 bytes is printed whole.

The five mutants are killed: `copy_log_one_chunk`, `healthy_timeout_drops_last_error`,
`wait_for_release_gpu_gone_is_ok`, `owners_pids_error_silent` and `up_pull_error_ignored`. The
second needed adapting: `rustfmt` had split its line, so `old` became the line's second half,
`last_error.map_or_else(…)`, and `new` became `{ let _ = last_error; String::new() }`. The five
checks are green, with 571 tests. Committed as `test(lifecycle): exercise the gateway's degraded paths`.

Two questions put to the user: the session plan's 12-token margin, from phase 1, and `down`'s
failed removal of a record, from batch 18. The user asked for a proposal: leave the first open,
fix the second now. The user agreed, and asked that the first be recorded where it can be found
and decided later (see Decisions). Written as a working note on the bench session plan's margin, in the
format of the other notes:
- the question;
- the plan's formula;
- the probe's figures;
- when it bites and what a failure leaves;
- four options, and what a roadmap must settle.

A task was added after batch 18 for the second, test-first. Two tests on `forget_fails`, each
watched failing on its assertion:
- `down` on a serving world returned `Err("cannot remove /state/gateway/gateway.json: …")`
  where it must stop the gateway, warn, wait for the GPU and return `Stopped`;
- `status` with a stale record returned the same error where it must read no gateway and warn.

The fix: in `down`, a failed removal after the stop is a warning, "the gateway stopped, but its
record cannot be removed (…)". In `running()`, a stale record that cannot be removed gives a
warning naming its PID instead of `Forgotten`, and reads as no gateway. No `forget()?` is left
in the lifecycle. `docs/commands.md`, the user's uncommitted file, says `up`, `down` and
`status` remove an exited gateway's record; still true, so it is left to the user. The five
checks are green, with 573 tests. Committed as `docs(notes): keep the bench session plan's margin as an open question` (the note) and `fix(lifecycle): warn of a record that cannot be removed once the gateway is gone` (the fix).

Batch 19 (findings 73, 98, 74, 75, 88, 89). Failure switches on the read-only fakes, named as
batch 18's:
- `FakeReleases`: `current_fails`, `installed_fails`;
- `FakeBuilds`: `current_fails`, `builds_fails`;
- `FakePrograms`: `run_fails`;
- `FakeProcesses`: `is_free_fails`.

`FakeBuilds` gained a `Default`, and its five literals in the tests of `doctor` and
`gateway_config` use it. The tests, all passing at once on the code:
- doctor, through a new `outcome` helper giving status, detail and remedy:
  - llama-swap's current or installed release unreadable, and the address check failing;
  - the build check with `current`, `builds` or the run failing;
  - podman, nvidia-ctk, the CDI listing failing, and an empty listing;
  - AC power and the power profile failing;
  - a model without `repo`: "copy … into …";
  - 64 MiB held, 63 MiB background;
- toolchain:
  - one table where every probe fails, each check's status, detail and remedy, and no
    toolchain;
  - `cuda-toolkit` with no toolkit keeps the driver's release;
  - three driver packages named without an ellipsis;
  - CMake and Git at their minimums, taken from `Tool::minimum`, so that batch 33's new Git
    minimum carries into the test;
- `tests/cli.rs`: `doctor --json` in an empty `HOME` exits 1 with `"healthy": false`. No
  inventory and no llama-swap make it fail whatever the machine.

All thirteen mutants are killed, each by the test written for it. One was adapted:
`x_keep_without_toolkit`'s `None => setup.package()` became `_ => setup.package()` with batch
52, so `old` and `new` take `_`. The five checks are green, with 583 unit tests and 25 CLI
tests. Committed as `test(doctor): check every failing probe and store, and doctor's exit status`.

Batch 53 (findings 103, 109), test-first. The types changed first, the behaviour unchanged:
- `Serving.current: Option<bool>` became `configuration: Configuration`, one of `Current`,
  `Outdated` or `Unknown(reason)`;
- `Lifecycle::inspect` takes a reading of the GPU;
- `FakeProcesses` gained `config_fails`;
- doctor's `GatewaySide` gained an inventory and `report_on(gpu)`.

Then six tests failed on their assertions:
- doctor on a `ScriptedGpu` whose second reading lacks the model, then fails: Idle VRAM warned
  "held by llama-server (pid 5000, 21094 MiB)" with the display remedy, finding 103's
  reproduction;
- a gateway whose configuration cannot be worked out, from an inventory that gained
  `${HOME}/t.jinja` or a `config()` that fails: the Gateway check said `Ok`, finding 109's;
- `podman ps` failing: no check reported it;
- `status` read the GPU twice;
- `status` with the GPU and the API unreadable still emitted "cannot tell whether…" as a
  third warning;
- `render_status` showed nothing for an unknown configuration.

The fix:
- `up`, `down`, `status` and `inspect` read the GPU once. The lifecycle's `target` and the
  configurator's new `prepare_on` take that reading, and `read_gpu` became `warn_unread`;
- `inspect` warns of nothing about the GPU, which doctor's GPU check reports from the same
  reading;
- doctor collects the lifecycle's other warnings and shows them as a "Gateway inspection"
  check, there only when something could not be read;
- the Gateway check warns on `Unknown` with its reason and `ai-ctl status`, and `status`
  prints the reason;
- doctor's false comment is gone.

On this machine, `doctor` gives its usual 22 `ok`. Under `CONTAINERS_STORAGE_CONF` naming a
bogus driver, as for L2, it adds "warn Gateway inspection: cannot list ai-ctl's containers
(podman ps: Error: configure storage: … driver not supported); leaving their processes out",
where the review found the output identical to a normal run. The five checks are green, with
586 tests. Committed as `fix(doctor): judge one GPU reading, and report what the inspection could not read`, and the README's Gateway row as `docs(readme): name doctor's warnings for an unknown configuration and an incomplete inspection`.

L1, test-first. The test gives doctor's gateway side an inventory with `${HOME}/t.jinja` in a
flag and no gateway running, runs `up` on the same fakes, and checks that `up` refuses
("contains `${`") while doctor's report holds a failure. It failed on that assertion, "every
check passed": L1 is confirmed.

The fix: a "Gateway configuration" check, second in the Gateway section, runs
`Configurator::prepare_on` on doctor's one reading of the GPU:
- `Ok` with the number of models and the address;
- `Fail` on a `ConfigError`, `generate`'s own refusals, with the refusal and "fix <inventory>"
  as the remedy;
- `Skip` on any other error, "needs the inventory, its builds and the GPU": no inventory, no
  build or no GPU fails a check of its own.

A first version skipped with `prepare_on`'s text, which ends "run `ai-ctl doctor`", absurd
inside doctor; the reason is now fixed. The stopped-gateway test lists the new check, and the
tests without a GPU or an inventory assert its skip. Watched failing again in a scratch copy
without the check. On this machine, `doctor` gives 23 `ok`, the new one "2 models, on
127.0.0.1:8080". The README's Gateway row names the check; `docs/commands.md` sends to the
README for the checks, so it needs no change. The five checks are green, with 587 tests.
Committed as `fix(doctor): fail a check on an inventory that up refuses to configure`.

Batch 20 (findings 77, 76, 79, 92, 93). Failure switches on the builder's test fakes, named as
batch 18's, each error worded like the real adapter's:
- `FakeStore`: `publish_fails`, `discard_fails`, `set_current_fails`;
- `FakeCmake`: `clean_fails`, and `fail` renamed `compile_fails`. It now records `job.jobs`,
  and gained a `Default`; the builder's `World` gained the `--jobs` asked for.

The tests, all passing at once on the code:
- the newest-release build asserts that nothing was replaced and that no `--jobs` reached
  cmake, and a new test builds with four and finds them in the compile job;
- a build that cannot be installed nor discarded returns the install's error, warns of the
  discard, keeps the current build and leaves the build tree;
- a build tree that cannot be removed is a warning beside a successful build;
- a build that cannot be made current fails "llama.cpp b11538 is installed but not current:
  …", installed and the old build still current;
- the refusal's exact text, now with a three-step remedy whose later steps align under the
  first;
- `summary()` on seven lines, stderr first, keeps the last five, and on blank output gives
  "it printed nothing";
- a `--list-devices` that prints b11538's CUDA line, then exits with a failure, is refused
  and discarded.

All seven mutants are killed, each by the test written for it, none adapted:
`build_publish_always_replace`, `build_discard_failure_silent`,
`build_set_current_failure_unreported`, `build_jobs_dropped`,
`refused_every_line_labelled_fix`, `summary_keeps_every_line` and
`list_devices_failure_ignored`. The five checks are green, with 593 tests. Committed as
`test(build): fail the store and cmake after compiling, and check --jobs and the error texts`.

Batch 21 (finding 78). The installer's test `FakeStore` gained `publish_fails`,
`discard_fails` and `set_current_fails`, worded like the real store's errors. Two tests, both
passing at once on the code:
- a release that can be neither installed nor discarded returns the install's error and warns
  of the discard, with nothing installed and nothing current;
- a release that cannot be made current fails "llama-swap v263 is installed but not current:
  …", once downloaded and once installed already, the old release still current.

The mutant `gateway_install_set_current_failure_unreported` is killed by the second, not
adapted. The doc comment of `FakeLlamaSwap`, which sat above an unrelated test, is moved back
onto it. The five checks are green, with 595 tests. Committed as `test(gateway): fail the store's publish, discard and set_current in gateway install`.

Batch 22 (findings 80, 34). `CmakeBuilder` gained a `program` field, `cmake` from `new`, which
both steps run. The new test runs `compile()` on a script, written under
`command::exclusive()`, that logs its arguments and, on `--build <tree>`, prints one Ninja
step and creates `bin/llama-server`. It asserts:
- the build tree is fresh: a `CMakeCache.txt` left in it is gone;
- the configure and build arguments, `--target` and `--parallel 3` among them. The generator
  is taken from `generator()`, so that the test holds with or without ninja on `PATH`;
- the log, both command lines and the step;
- the progress, `Configuring` then 1 of 1;
- with a script that builds nothing, "the build succeeded but … is missing; see <log>".

It passed at once. The mutant `cmake_missing_artifact_ignored` is killed by it, not adapted.

`tests/fixtures/make-build-excerpt.txt` was captured on this machine, cmake 4.3.0 and GNU Make
4.4.1, from a project of two library sources and one executable in the session's scratch
directory:

```bash
LC_ALL=C cmake -S proj -B build -G "Unix Makefiles"
LC_ALL=C cmake --build build --target app --parallel 2 > make-build.txt 2>&1
```

Its seven lines are the whole build. `make_percentages_give_the_progress` reads them in place
of the line written from memory: 40, 40, 60, 60, 80, 100 and 100 of 100.
The five checks are green, with 596 tests. Committed as `test(build): run CmakeBuilder::compile on a script, and read a captured Make build`.

Batch 23 (findings 95, 96, 81, E4). Test findings, each new test passing at once on the code:
- finding 95: `SystemHost::cuda_toolkit` runs on a temporary `cuda-13.2/bin/nvcc`, a script
  printing `nvcc-13.2.txt`, beside `host_config-13.2-excerpt.h` and a `cuda` link to it, with
  `cuda/bin` alone on the host's `PATH`. The toolkit is 13.2.86, `nvcc` is reported at
  `cuda-13.2/bin/nvcc` and GCC 15 is the limit;
- finding 96: a `g++` linked to `g++-15` gives one compiler, `g++`;
- finding 81: `[g++ 14.3.0, g++-15 15.3.1]`, in the probe's order, with 15 as the limit picks
  `g++-15`.

The mutants `nvcc_root_not_canonical`, `host_compilers_no_link_dedup` and
`pick_host_compiler_first_accepted` are each killed by the test written for it, none adapted.

E4: the hand-written JSON case of `parse_cdi_driver` is replaced with a capture. On this
machine, nvidia-ctk 1.20.1 and driver 615.78.08:

```bash
nvidia-ctk cdi generate --format json --quiet > nvidia.json
```

It prints one line of 16 KB, the GPU's UUID among the device names.
`tests/fixtures/nvidia-ctk-1.20.1-cdi-spec-excerpt.json` keeps, in the capture's order and
spelling, `cdiVersion`, `kind`, the `all` device's name, and the container edits' `env` and
libcuda mount. The mount and the `env` were checked byte for byte against the capture. The
hooks are left out. Their `libcuda.so.615.78.08::…` link would come first and end at `:`, as
the YAML excerpt's does, whereas the case replaced held a version that ends at a JSON quote.
`parse_cdi_driver` reads 615.78.08 from it, and the probe's test of both CDI directories now
reads it as `/etc/cdi/nvidia.json`, in place of the YAML excerpt with an edited version.
The five checks are green, with 599 tests. Committed as `test(doctor): run the nvcc probe through a cuda link, and capture a JSON CDI specification`.

Batch 24 (findings 82, 83, 91, 94). Test findings:
- finding 82: the draft check with 3 more accepted drafts on the server alone disagrees;
- finding 83: the session's reading in `testing::run()` is now `BRAKE`, the power limit and
  the hardware's power brake. A new domain test reads its share, 100% of the session and 25%
  of the run. `render_run`'s table shows "power 100%, hardware 100%" and "power 75%,
  thermal 25%, hardware 25%"; its expected text is updated, nothing else changed. The run
  store's round trip already compared the whole run;
- finding 91: a run against a copy of a 65,536-token context, its limit at 65,023, gives one
  row labelled `limit`;
- finding 94: `FakeModelServer` gained `overshoot`, a target: every prompt holding code, told
  by the corpus's `<file path="src/`, counts that target and 3 more. The system prompt cites
  `<file path="…">`, so the bare prompt keeps its own count. Fitting 5,000 tokens fails "no
  prompt of 5000 tokens could be fitted: the shortest attempt made 5003". A fake that ignores
  the target cannot reach this error: the loop corrects a constant offset at the second
  attempt, and a faster count drives the code to none, which fits.

Each new test passed at once. The six mutants are killed, none adapted:
- `agrees_ignores_accepted`, by the draft check's test;
- `gpu_summary_hardware_zero`, by the new domain test and `render_run`'s;
- `run_store_hardware_false`, by the round trip and the store's other tests on the test run;
- `render_gpu_hardware_dropped`, by `render_run`'s test;
- `compare_several_limits_unlabelled` and `prompts_no_fit_error_text`, by the tests written
  for them.

The five checks are green, with 602 tests. Committed as `test(bench): cover accepted drafts, a hardware slowdown, limits of two sizes and a prompt that never fits`.

Batch 25 (finding 84), a test finding. One test serves the two tokenize fixtures in turn:
- `pieces()` gets back the fixture's text, and its request is a `POST` to
  `/upstream/qwen3.8-27b/tokenize` whose body is the content with `"with_pieces":true`;
- `count()` counts 5 tokens, and its body is the content alone.

It passed at once. Under `pieces_without_with_pieces` the fixture still answers with its
pieces, and the body's assertion alone fails, as the finding expected. Not adapted. The five
checks are green, with 603 tests. Committed as `test(bench): check the tokenize requests of pieces() and count()`.

Batch 27 (finding 87), a test finding. `[("a", 20000), ("b", 3051), ("c", 5000)]` gives
`["a & b", "b & c"]`: `a` and `b` fill the test GPU's 24,075 MiB less the default 1,024 MiB
margin to the MiB, which the test asserts first. It passed at once. The mutant
`matrix_exact_fit_refused` is killed by it, not adapted. The finding's other comparison,
`>=` in place of `>` in `is_largest`, has no named mutant; applied by hand in the scratch copy,
it fails the test too, with `["a", "a & b", "b & c"]`. The five checks are green, with
604 tests. Committed as `test(gateway): let two models that fill the VRAM budget exactly run together`.

Batch 28 (findings 97, 33). `Podman` became a struct with a `program` field, built in
`main.rs` as `Podman::new("podman".into())`; its four commands run it. Captured on this
machine, podman 5.8.7:

```bash
podman image inspect --format '{{.Digest}}' ghcr.io/ggml-org/llama.cpp:server-cuda13-b11541 \
    > tests/fixtures/podman-5.8.7-image-inspect-digest.txt
podman ps --format '{{.Pid}}' > tests/fixtures/podman-5.8.7-ps-pid.txt
```

The second lists the one container running here, Open WebUI's. Two more probes, kept in the
tests rather than as files:
- `podman ps --filter 'name=^ai-ctl-' --format '{{.Pid}}'` prints nothing at all;
- `podman image exists` of a tag podman does not hold exits 1 and prints nothing, the
  adapter's reading of a missing image.

The parsers' tests read the captures: the digest; the PID, alone and repeated as two
containers' lines; and the empty listing. Their refusals keep the hand-written invalid
inputs. Each of the four methods is run on a script that checks its arguments, exits 2 on
any others, and answers:
- in podman's way, with the captures;
- with exit 1 and nothing printed: a missing image for `image exists`, and "exited with an
  error and printed nothing" for `ps` and `image inspect`;
- with batch 53's broken-storage capture on stderr and exit 125: the error, named after the
  command;
- by being absent: "podman not found; run `sudo dnf install podman`", and no PID for `ps`.

All passed at once. The mutant `podman_image_error_as_missing` is killed by the
`image exists` test, not adapted. The five checks are green, with 608 tests. Committed as
`test(containers): run the Podman adapter on scripts, and parse captured podman outputs`.

Closure. The five checks are green, with 608 unit tests and 25 CLI tests. The acceptance
criteria, checked again on the final code:
- the mutants: the phase's findings cite 45. `relative_spread_signed`'s code is gone with
  batch 14, and the 44 others ran once more in one scratch copy, by a script that applies each
  `old` → `new` and runs `cargo test --lib`, or `cargo test` for
  `doctor_exit_code_always_success`, which `tests/cli.rs` kills. Three took their adapted
  `old` and `new`, as recorded above: `power_profile_no_fallback`,
  `healthy_timeout_drops_last_error` and `x_keep_without_toolkit`. All 44 are killed, none by
  a compile error;
- finding 36's check ran at batch 14 and reported nothing. Since then, none of the items it
  removed is back: no `model_files`, `cuda_architectures`, `relative_spread`, `nvcc()`,
  `c_compiler()`, `Sha256::as_str`, `Corpus::len` or `::is_empty`, `Timestamp::unix`,
  `CommandLine::words`, `Summary.count` or `.max`, nor `GpuSummary.readings`;
- `the_gpu_is_read_once_for_every_check` reads a `ScriptedGpu` whose second reading lacks
  the model, then fails, and gets an `Ok` Idle VRAM from one read;
- L1 is confirmed and fixed.

---

## Decisions

- **The session plan's margin stays open, in a note** (the user, 2026-10-16). The plan holds by
  12 tokens on the probe's run, and only a context within some 30 tokens above 22,763 could
  overflow, with a clean, named failure since batch 10. A margin would be a constant of the
  kind batch 7 refused for `turn_extra`. a working note on the bench session plan's margin holds the
  question, the figures and the options, for a later roadmap to decide.
- **A record that cannot be removed once its gateway is gone is a warning** (the user,
  2026-10-16). The gateway being stopped or gone is what the command is for; a record left
  behind reads as no gateway to the next command. `down` goes on to wait for the GPU's memory.
  This adds a task to this phase, after batch 18.
- **A command reads the GPU once, and passes the reading on** (batch 53). `up`, `down`,
  `status` and `doctor` judge one reading. `Configurator::prepare_on` and the lifecycle's
  `target` take it; `wait_for_release` alone reads again, since it polls. Later batches that
  touch the GPU keep this.
- **Whether a gateway runs on the current configuration is a `Configuration`, with the
  reason when it cannot be told**, not an event. `status` prints the reason, and doctor warns
  with it.
- **What doctor's inspection could not read is a check of its own, "Gateway inspection"**,
  in the Gateway section and only when something is missing. Its warnings are the
  lifecycle's own texts; the GPU's are left to the GPU check, which reads the same reading.
- **`up`'s refusals of the inventory are doctor's "Gateway configuration" check** (L1). It
  fails on `generate`'s refusals alone; what fails elsewhere, the inventory, a build or the
  GPU, skips it, so that one cause gives one failure. While a gateway serves, the Gateway
  check still warns that its configuration cannot be told, with the same reason: one says
  what `up` would do, the other what runs.
- **A failure switch is a `bool` named after its method, `<method>_fails`**, on the shared
  fakes (batches 18, 19) and on the test modules' own (batches 20, 21), each error worded like
  the real adapter's. Batches 40 and 41 move each switch with its method.
- **An adapter that runs a program holds it as a field**: `CmakeBuilder` sets `cmake` in
  `new`, and `main.rs` builds `Podman` with `podman` (batches 22, 28). Its tests run it on a
  script written under `command::exclusive()`, which checks its arguments. Later adapters
  follow this rather than a program found by name.
- **A fixture is an output captured on this machine, its command in the report.** An excerpt
  keeps the capture's order and spelling, and leaves out what identifies the machine, such as
  the GPU's UUID (E4). Parsers' refusals keep hand-written invalid inputs, as elsewhere in the
  crate. A repeated capture stands for several lines of the same output (`parse_pids`).
- **`FakeModelServer::overshoot` knows the target** (batch 24). A fake tokenizer that ignores
  it cannot reach the no-fit error: the fitting loop corrects a constant offset at its second
  attempt, and a faster count drives the code to none, which fits.

---

## Files Changed

Computed against the start commit `docs(v2-review-fixes): close phase 1, bugs and error handling`, working tree included.

**Added**
- a working note on the bench session plan's margin
- `docs/roadmap/on-progress/v2-review-fixes/phase-2-dead-code-and-tests-report.md`
- `tests/fixtures/make-build-excerpt.txt`
- `tests/fixtures/nvidia-ctk-1.20.1-cdi-spec-excerpt.json`
- `tests/fixtures/podman-5.8.7-image-inspect-digest.txt`
- `tests/fixtures/podman-5.8.7-ps-pid.txt`
- `Makefile` — the user's uncommitted work, not this phase's
- `docs/commands.md` — the user's uncommitted work, not this phase's

**Modified**
- `CLAUDE.md` — this closure's current work line; also holds the user's uncommitted
  Commands line
- `README.md` — doctor's Gateway row; also holds the user's uncommitted link to
  `docs/commands.md`
- `docs/roadmap/on-progress/v2-review-fixes/README.md`
- `docs/roadmap/on-progress/v2-review-fixes/phase-2-dead-code-and-tests.md`
- `docs/roadmap/on-progress/v2-review-fixes/phase-4-structure-and-refactoring.md`
- `src/application/bench.rs`
- `src/application/build.rs`
- `src/application/doctor.rs`
- `src/application/fakes.rs`
- `src/application/gateway.rs`
- `src/application/gateway_config.rs`
- `src/application/lifecycle.rs`
- `src/application/toolchain.rs`
- `src/domain/bench.rs`
- `src/domain/build.rs`
- `src/domain/gateway.rs`
- `src/domain/gateway_config.rs`
- `src/domain/inventory.rs`
- `src/domain/machine.rs`
- `src/domain/prompt.rs`
- `src/domain/stats.rs`
- `src/domain/timestamp.rs`
- `src/infrastructure/build_store.rs`
- `src/infrastructure/cmake.rs`
- `src/infrastructure/gateway_process.rs`
- `src/infrastructure/gateway_store.rs`
- `src/infrastructure/host.rs`
- `src/infrastructure/inventory_file.rs`
- `src/infrastructure/podman.rs`
- `src/infrastructure/server_api.rs`
- `src/main.rs`
- `src/presentation/bench.rs`
- `src/presentation/compare.rs`
- `src/presentation/lifecycle.rs`
- `tests/cli.rs`
- `tests/gateway_lifecycle.rs`

---

## Problems And Deviations

- **`down` reports a record it cannot remove as its only error.** Seen while writing batch
  18's tests. After stopping the gateway, `down` calls `forget()?`. When that fails, the error
  is the removal's alone: it does not say the gateway stopped, and the wait for the GPU's
  memory is skipped. The next `up` or `status` removes the stale record. No finding covers it.
  Put to the user, who approved a fix in this phase: the task after batch 18, done. The same
  held for a stale record found by `running()`, fixed with it.
- **The phase grew from 16 tasks to 17** with that task.
- **L1 is confirmed**: doctor reported every check `Ok` while `up` refused an inventory with
  `${` in a flag. Fixed in this phase by the "Gateway configuration" check.
- **Batch 22's fixture is a whole build, not an excerpt.** A Makefile-generator build of a
  three-file project prints seven lines; the file keeps the name the task gave,
  `make-build-excerpt.txt`.
- **Two small changes outside the tasks**: the doc comment of `FakeLlamaSwap`, which sat above
  an unrelated test, moved back onto it (batch 21), and the CDI probe's test reads the JSON
  excerpt from `/etc/cdi` instead of the YAML one with an edited version (E4).

---

## Changes To Later Phases

- `phase-4-structure-and-refactoring.md`, batch 45: notes that batch 53 replaced `read_gpu`
  with `warn_unread` and added `Configurator::prepare_on`, both taking a reading of the GPU.
- `phase-4-structure-and-refactoring.md`, batch 42: notes that batch 28's tests compare each
  podman method without podman with `MISSING`, so that those assertions follow the message
  `Tool::install` gives.

---

## Assessment

The phase did what it set out to do. The API only tests called is gone, and the review's test
gaps are closed: each of the 44 mutants its findings cite is killed on the final code, which
the review found every test let pass. Doctor reads the GPU once and says what it cannot judge,
and L1 proved real: doctor passed an inventory that `up` refused, and now fails a check on
it. The work found one defect no finding covered, a record `down` could not remove reported as
its only error, fixed with the user's approval. It left one question open, the bench session
plan's 12-token margin, in a working note on the bench session plan's margin.

Most batches were test findings: their tests passed at once, and the mutant was the proof. The
failure switches on the fakes are what made the degraded paths reachable. Two adapters,
`CmakeBuilder` and `Podman`, now hold their program as a field and run on scripts; the
fixtures added are all captures of this machine, their commands recorded above.

What phase 3 needs to know first: its batches rewrite the code these tests now hold.
- Batch 29's deadline replaces `START_POLLS`: the timeout test of batch 18 asserts the health
  check's last error, not the count of polls.
- Batch 32's `execute` runs `podman pull` without a timeout, which batch 28's script test
  covers.
- Batch 34's `parent` replaces `descendants`, which `FakeProcesses::descendants_fails` and the
  owners' tests use.
- Batch 33's git minimum carries into finding 89's boundary test through `Tool::minimum`.
