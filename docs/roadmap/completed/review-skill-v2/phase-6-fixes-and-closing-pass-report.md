# Phase 6 Report: Fixes And Closing Pass

**Phase:** [phase-6-fixes-and-closing-pass.md](phase-6-fixes-and-closing-pass.md)
**Start Commit:** `docs(review-skill-v2): close phase 5` of 2026-11-05

---

## Work Log

### 2026-11-05

Batch 1 begins with its update within the requirements, finding 28, in a commit of its own as
`references/fixing.md` asks: `toml` 1.1.8 → 1.2.0 and `toml_writer` 1.1.3 → 1.2.0. The checks
green, 828 tests; `cargo deny check` clean, its three duplicates as before. Committed as
`build(deps): update toml to 1.2.0`, written into finding 28's `fixed_in`.

The batches go in their numbers' order, batch 1 first: its eleven other findings, below the
floor, make one commit with no red test to apply, then batch 2, the tests alone. None of the
eleven started; finding 22's code, `src/domain/safetensors.rs` and `Doctor::index_shards`, was
read: the enum must keep each message as the tests pin it, «not JSON: <serde's message>» among
them, and the domain's other errors carry no `#[source]`.

Batch 1's `cargo mutants --in-diff` ended at exit 4, no mutant tested: cargo-mutants builds
its copy without `.git`, and `no_tracked_text_file_holds_a_raw_separator`, which phase 2 added
in `tests/repository.rs` after the review's end commit, runs `git ls-files` there.
`.cargo/mutants.toml` now skips that test in the mutants' runs (Decisions): the baseline
passes with it alone filtered out.

Batch 1's eleven other findings fixed in one commit, each by its finding's `fix`: the
safetensors parser on a `thiserror` enum of the domain, its messages unchanged; whisper's
`-l auto` written in `docs/audio/whisper-cpp.md` §2 as a decision, in the «Decided:» form its
other decisions take, with what was set aside and the short dictation detected as Icelandic on
2026-10-30; finding 35's test taken from `repro/review-tests/tests.diff`, without the test of
`..` that finding 45's removal makes moot. The checks green, 828 tests, finding 45's removed
test and 35's new one cancelling out. `cargo mutants --in-diff` on the batch's diff: 24
mutants, 22 caught, 2 unviable, none missed. Ten mutants by hand, each watched failing its
test in a copy, recorded as M1 to M10 in `docs/reviews/delivered-code/mutants.json`, which
this batch creates: three on `shards`' guards and filter, three on `check_finite` and its two
calls, and the four changes to `update_submodules`, `parent_of` and `update_within` that
finding 35 named, which its test now kills. Fixing finding 24 showed its sibling,
`FakeApi::running` failing through `models: None`: older code below the floor, added as the
session's finding 57 and registered.

Batch 1's commit written into its eleven findings' `fixed_in`; `scripts/regression.py` on M1
to M10 at that commit: ten caught.

The mutants' exit 4 is the fix method's first failure in this use, recorded in the evals'
`failures_in_use`: step 3 of `SKILL.md` read exit 4 as a red check alone, where here every
check was green and the copy lacked `.git`. The line now names that case, the file that
skips such a test and the baseline's log; `audit.py` clean. Steps of the fix method this
batch exercised: the sibling paths (finding 57), the mutants on the batch's diff, the
mutants by hand and their record; neither a reproduction seen failing first nor the input
class, its findings having no red test.

Batch 2, the tests alone, findings 29 to 41, from each finding's `repro/<n>/test.patch`, five
of them applied by `patch` with fuzz, their anchors moved by batch 1 or by each other. Each
test passes on the code, and each of the fifteen changes the findings' `mutant-*.patch` name
fails its test on its assertion, applied by hand in a copy: recorded as M11 to M25, finding
39's on `text::joined`, where batch 1 moved the join. `cargo mutants --in-diff` writes no
mutant on a diff of tests alone. The sibling paths, each tried as a mutant against the whole
suite: voice_dir's path was held already; the write of `llama-swap.yaml`, older code beside
finding 41's file, was not, and takes a test of its own, killing M26; `submodule sync` and
`clean`, beside finding 36's update, stay untested, their notes saying why. The two tests
that write a script under `command::exclusive()` and the build store's ran 30 times, green.
The checks green, 841 tests.

Batch 2's commit written into its findings' `fixed_in`; `scripts/regression.py` on M11 to M26
at that commit: sixteen caught.

Batch 3, finding 1, as the user answered it: `LLAMA_USE_PREBUILT_UI` off in llama.cpp's recipe,
the verifier's test red on its assertion first, then green with the settled fix. Every
statement written was checked against b12111's sources and `build.log`: the bucket, the
fallback to `latest`, the digest from the same place, and the fresh build tree that leaves no
stale UI to embed. The sibling recipes: stable-diffusion.cpp's frontend off already, and no
download in the build logs of whisper.cpp v1.9.5 and audio.cpp v0.9.3. Whether an official
image embeds the UI was left unsaid, a look at b11541's image settling nothing:
`docs/security.md` §2 now names the files of the UI a llama-server embeds. One mutant by
hand, the flag removed, recorded as M27; `cargo mutants` writes none on a recipe's table. The
checks green, 841 tests; `CHANGELOG.md` and `docs/stack.md` say it too.

Batch 4, finding 3: doctor and the configuration refuse a current build without a patch its
recipe applies now, through one `missing_patches`. Both of the verifier's tests red on their
assertions first; with the fix, two tests of batch 2 failed too, their builds holding no patch,
and now carry their recipes'. Every current build on this machine holds its recipe's patches,
so nothing the user runs is refused. A unit test covers the input class: none missing, two
missing beside a patch the recipe dropped, a recipe with none. `cargo mutants --in-diff`: 8,
6 caught, 2 unviable; the two `if let` guards taking their else, by hand, recorded as M28 and
M29. `docs/security.md`, `docs/commands.md` (`gateway config`, `up`), the README and
`CHANGELOG.md` say it. The checks green, 844 tests.

Batch 4's commit written into finding 3's `fixed_in`; M28 and M29 caught at it.

Batch 5, finding 5: `reads_as_index` takes the flag, and sd-server's `tokenizer` is no index.
The verifier's test red on its assertion, doctor failing the model, then green. The sibling
flags, read in stable-diffusion.cpp's sources: every other file it loads goes through its
model loader, which reads a `.json` as an index. `cargo mutants --in-diff`: 9, 8 caught, 1
unviable; the flag's clause removed by hand, M30. The README's doctor row and `CHANGELOG.md`
say it. The checks green, 845 tests.

Batch 5's commit written into finding 5's `fixed_in`; M30 caught at it.

Batch 6, finding 6: the index's directory made normal before the shards are compared with it,
by one `safetensors::normal` that `shard_path` shares, without the arm for `.` batch 1
removed, `models_dir` being absolute by the inventory's rule. The verifier's test red on its
assertion, then green. `cargo mutants --in-diff`: 4, all caught; by hand, the comparison with
the directory as written and the arm for `..` removed, M31 and M32. One case left in the
finding's note: a symbolic link before `models_dir`'s `..`. The checks green, 846 tests.

Batch 6's commit written into finding 6's `fixed_in`; M31 and M32 caught at it.

Batch 7, finding 30: doctor refuses a shard whose path in the model's directory starts with
`-`, checked after batch 6's normalisation, as its verifier settled. The test red on its
assertion, the option in the printed command, then green for the three spellings. `cargo
mutants --in-diff`: 2, both caught; the guard forced false by hand, M33. The checks green, 847
tests.

Batch 7's commit written into finding 30's `fixed_in`; M33 caught at it.

Batch 8, findings 7 and 8, one change to `build_checks`: the projects in use taken from the
models' backends, as `up` resolves them, and an inventory that could not be read said so. Each
verifier's fix was written beside its own test alone; merged by hand. Both tests red on their
assertions, then green. `cargo mutants --in-diff`: 11, 9 caught, 2 unviable; by hand, the
declared backends taken again and the arm of an unused project removed, M34 and M35. The
README's Builds row, `docs/commands.md` and `CHANGELOG.md` say it. The checks green, 849
tests.

Batch 8's commit written into findings 7 and 8's `fixed_in`; M34 and M35 caught at it.

Batch 9, finding 9: `bench run` refuses a model of another server than llama-server before
any request. The verifier's test, its whisper build given the patches batch 4 now asks, red
on its assertion, the bench having loaded the model, then green. `cargo mutants --in-diff`
wrote one mutant, unviable, none inside `ensure!`; the condition forced true by hand, M36. The
README, `docs/commands.md`, its table of commands, `bench run`'s help and `CHANGELOG.md` say
it. The checks green, 850 tests.

Batch 9's commit written into finding 9's `fixed_in`; M36 caught at it.

Batch 10, finding 10: a patch git refused is a `PatchRefused` of the domain, and only that
refusal sends to another release. The verifier's test red on its assertion, then green; git's
own test of a patch that does not apply now asserts the refusal's type, which no double can.
`cargo mutants --in-diff`: 3, all caught; by hand, the build's guard forced each way and the
refusal's type lost in git, M37 to M39. The checks green, 851 tests.

Batch 10's commit written into finding 10's `fixed_in`; M37 to M39 caught at it.

Batch 11, finding 11: a submodule a kill left without a commit during its first fetch is
removed and cloned again. The verifier's test red on its assertion, git's «Unable to find
current revision», then green. By the method, two additions: `.gitmodules`' mapping read with
git's `--null`, so that a name may hold a space, and a name from upstream that climbs out of
`modules` or is absolute left out, since the removal follows it; both tested on a capture,
`tests/fixtures/git-config-null-submodule-paths.txt`, listed in the fixtures' README. `cargo
mutants --in-diff`: 16, 14 caught, 2 missed, the two `||` of the removal's guard: two tests
kill them, an unnamed submodule and one with commits each kept. By hand, those two, the retry
forced off and the name's filter removed, M40 to M43. The new tests run `git` in scratch
folders alone, once each. `docs/commands.md` and `CHANGELOG.md` say it. The checks green, 855
tests.

Batch 11's commit written into finding 11's `fixed_in`; M40 to M43 caught at it.

Batch 12, finding 12: an `audiocpp_server` model's directory holding nothing but hf's `.cache`
fails, through a new `ToolchainProbe::entries`. The verifier's test red on its assertion,
then green. The input class caught a defect of the settled fix: a directory that cannot be
read gave the check an empty remedy, `Some("")`, every earlier detail having come with one; it
now gives none, tested through a new `entries_fails` of the double, and `SystemHost::entries`
is tested on a scratch directory, a name not UTF-8 among its entries. `cargo mutants
--in-diff`: 9, 8 caught, 1 unviable; by hand, four, M44 to M47. The README's Inventory row and
`CHANGELOG.md` say it. The checks green, 858 tests.

Batch 12's commit written into finding 12's `fixed_in`; M44 to M47 caught at it. The
review's status note brought to batch 13. The session closed there, at the user's word, «fini
la tache en cours et on cloture».

### 2026-11-06

Batch 13, finding 42: `read_capped` checks the bytes' count before decoding them. The
verifier's test red on its assertion, then green; the input class added characters of two to
four bytes under each limit that cuts them, and an empty file. The sibling reads decode lossily
or once whole. `cargo mutants --in-diff`: 4, all caught; by hand, M48 to M50. The checks green,
859 tests.

Batch 13's commit written into finding 42's `fixed_in`; M48 to M50 caught at it.

Batch 14, finding 26, as the user decided it: past 20000 sets, a `TooManySets` says how many
models can share the GPU and to take some out, refused by `doctor`, `gateway config` and `up`.
The verifier's test red on its assertion, 184756 sets, then green; the input class added the
bound itself, 20000 sets passing and one more refused. `cargo mutants --in-diff`: 16, 13
caught, 2 unviable, 1 missed, `j != i` of the sharing count, killed by a model that fits beside
others but not beside its own size; by hand, M51 to M53. The early stop in `largest_sets`,
forced off, changes no result, only the time, 0.74 s against 0.05 s: no test without timing
kills it. The decision and the probe's figures in `docs/stack.md` §6.4. The checks green, 861
tests.

Batch 14's commit written into finding 26's `fixed_in`; M51 to M53 caught at it.

Batch 15, the documents, findings 2, 14 to 21 and 56. Each statement was checked against the
sources, the code or a run: audio.cpp v0.9.3's and whisper.cpp v1.9.5's host-path routes in the
clones, llama-swap v263's `-validate` without and with `AI_CTL_API_KEY`, `init --help`, the
audiocpp files' modes on this machine, the shipped inventory and snapshot. `cargo mutants`
lists none, the one code change a help text; `check_docs.py` as before the batch. The checks
green, 861 tests.

Batch 15's commit written into its findings' `fixed_in`.

Batch 16, finding 4, as the user decided it: OpenWebUI 0.13.0, running here since 2026-11-04,
pinned by its linux/amd64 manifest's digest as §7 pins the shipped images. «Checked with the
page again» was done without the interface, both images read from podman's store against
0.12.3, the tested one: the backend's requests to the gateway unchanged, the page's 42 labels in
both frontends, its French ones in both locales. No change to ai-ctl, so no changelog entry.
The checks green, 861 tests; `cargo deny check` clean. The commit removes the review's `repro/`.

The three Fixes tasks ticked: the sixteen batches above; the one dependency finding, 28, in
batch 1, no crate beyond the requirements; the changelog beside each batch.

Finding 4's commit written into its `fixed_in`, and the review's stage set `complete`.

The user rebuilt b12111 with `target/release/ai-ctl`, built on 2026-10-30, before batch 3: its
recipe still left `LLAMA_USE_PREBUILT_UI` on, and the build downloaded `latest`'s UI again, its
`build.json` without the flag, which neither `doctor` nor `up` noticed. After `make bin`, the
user's second rebuild holds the flag off in `build.json`, and its `build.log` downloads nothing,
«no assets available - building without an embedded UI», «embedded 0 assets»; `build prune
--keep 1 llama.cpp` removed b11700 and b11538, and `doctor`'s Builds check passes on b12111
alone. The incident is the defect of Deviations, for the closing pass.

The closing pass approved at `high`, «oui»: `review-security`, its verifier and
`review-dependencies` at `high`, on the roadmap's diff, from `docs(review-skill-v2): create the roadmap` of 2026-10-31 to `docs(review-skill-v2): record the session's close`, 138 hunks in 20 code
files, the sixteen batches' code.

The pass, `docs/reviews/delivered-code/2026-11-06-review-skill-v2-closing-pass/`: the checks
green, 861 tests, `cargo deny check` clean; `cargo mutants --in-diff` on the roadmap's diff, 100
mutants, 91 caught, 9 unviable, none missed; `scripts/regression.py` on M1 to M53, 52 caught and
M18 gone, batch 11 having reworded its line while each batch ran its own mutants alone. Three
findings. `review-security`'s, merged with the session's defect of Deviations: the stale-build
check compares patch names alone, so a recipe's CMake option added since reaches no older build;
confirmed `low`, a control weakened with no damage shown, every current build here holding its
recipe's options; its other half, a patch's content, refuted by the verifier, a rebased patch
keeping its name and its change; batch 2, its settled fix in `repro/1/`. Below the floor, batch
1: `winnow` 1.0.4 → 1.0.5 within the requirements, and M18 recorded again. `review-dependencies`
raised no finding on the lag of llama-swap (v263, newest v270) or the backends, their release
notes out of its reach.

Batch 1 of the pass, its update first, finding 2: `winnow` 1.0.4 → 1.0.5. The checks green, 861
tests; `cargo deny check` clean.

Batch 1's other finding, 3: M18 recorded again on the line batch 11 left, its test unchanged,
caught in a scratch clone before the commit. The checks green, 861 tests. M18 is the fix
method's second failure in this use: step 5 of `references/fixing.md` runs the batch's own
mutants alone, so a later batch that rewrites an earlier mutant's line leaves it gone unseen.

Batch 1's commit written into finding 3's `fixed_in`; `scripts/regression.py` on the whole
record at that commit: 53 caught, in 123 s. The closing pass's task ticked. Batch 2 waits for
the user's answer to its hard case, the session closed at the user's word, «je te laisse
travailler en autonomie […] une fois la tache terminée tu peux clore la session nous
reprendrons avec les résultats dans une session neuve».

The user chose the verifier's fix for finding 1 (Decisions).

Batch 2 of the closing pass, finding 1: `unlike_recipe` compares a current build's patches'
names and its recipe's CMake options. The verifier's three tests red on their assertions, then
green. Its settled fix left four statements of the patch-only check, in the README and
`docs/commands.md`, now naming the options. The input class added several options unset beside
one at another value, a dropped option, a build recording none, and a patch and an option
missing together; a new test holds that the builder overrides no recipe's option, which the
prefix test let through for `GGML_NATIVE`. `cargo mutants --in-diff`: 11, 9 caught, 2
unviable; by hand, M54 to M58. The rename left M28 and M29 gone, recorded again: M18's failure
once more, caught here only by reading the record for the batch's files. `doctor` and `gateway
config` pass on this machine's four builds. The checks green, 866 tests. The commit removes the
review's `repro/`.

Batch 2's commit written into finding 1's `fixed_in`, and the review set `complete`;
`scripts/regression.py` on the whole record at that commit: 58 caught, in 133 s. The pass's
three findings fixed, none registered, and no pass after: the closing pass's fixes task ticked.

The fix method's two later failures recorded in the evals' `failures_in_use` and fixed in
`references/fixing.md`. The builds batch 3's recipe had made before, which neither its sibling
paths nor batch 4's input class named: the sibling paths now take what a fix to what the code
makes leaves made, guarded on each part of how it was made. The recorded mutants gone unseen,
M18 then M28 and M29: each batch now runs the whole record after its commit, two minutes for 58
mutants, and records again one its change left `gone` or `missed`. `audit.py` clean after each.
The steps this use exercised, as phase 4 left them to it: the reproduction seen failing first,
in each batch holding a `test.patch`; the mutants on each batch's diff, failing once on exit 4,
and by hand, M1 to M58; the sibling paths, which found finding 57 and the four statements the
pass's settled fix left, and missed the builds made before; the input class, which caught a
defect of batch 12's settled fix and missed the recipe's options in batch 4; the stopping rule,
one security pass on the fixes' diff, its findings fixed and no pass after.

`shared-and-specific.md` written from phase 0's draft and the cycle: the line between every
repository and ai-ctl, part by part, with where each part lives; the six lines the use moved,
the scripts holding both sides among them; what another repository gives the skill. The user
approved it unchanged, «parfait valider»: its task ticked.

---

## Decisions

- **The mutants' runs skip the tracked-text check**, by `.cargo/mutants.toml`: the test reads
  the tree's text through git, which no mutant changes, and neither the copy of a review's step
  3, made by `git archive`, nor cargo-mutants' own copy holds `.git`. Set aside: `copy_vcs`,
  which mends the batches' runs alone. The closing pass's mutants run takes the same file,
  which `git archive` carries.
- **A current build is compared with its recipe by its patches' names and its recipe's CMake
  options**, the user's choice on 2026-11-06 for the closing pass's finding 1, the verifier's
  settled fix: a patch rebased onto a newer release keeps its name and its change, one whose
  change differs takes a new name, a rule `docs/security.md` states. Set aside: comparing each
  patch's content too, which refuses a v0.9.2 audio.cpp build made as intended, the rebased
  `api-key.patch` not applying to it. Binds every later change to a recipe's patch.

---

## Deviations

- Finding 57, the session's, seen while fixing finding 24: older code below the floor,
  registered, in no batch.
- `.cargo/mutants.toml`, which no task planned, added for the batches' mutants runs
  (Decisions).
- Seen on 2026-11-06, after the fixes: `doctor` and `up` compare a current build's patches
  with its recipe's (batch 4, finding 3) but not its CMake options. A build made by an older
  recipe, before batch 3 or by a binary built before it, passes both with no word, the UI of
  finding 1 embedded: that fix reaches only the builds made after it. Left open, for the
  closing pass as the session's finding: it became the pass's finding 1, batch 2.
- M18 gone, found by the closing pass's regression run: recorded again in batch 1.
- M28 and M29 gone with the rename of the pass's batch 2, found by reading the record for the
  batch's files: recorded again in that batch, and the fix method's step changed so that each
  batch runs the whole record.
- The settled fix of the pass's finding 1 left four statements of the patch-only check, in the
  README and `docs/commands.md`: fixed in its batch.

---

## Assessment

The phase fixed the 46 findings phase 5's review gave it, in sixteen batches of one commit
each, the session's finding 57 registered beside them, with one update within the
requirements and each change to what ai-ctl does in `CHANGELOG.md`'s 0.2.0 entry. The closing
pass, `review-security`, its verifier and the dependency axis on the roadmap's diff, raised
three findings: below the floor, `winnow`'s update and M18 gone; above it, a current build
compared with its recipe by its patches alone, which the user settled on the patches' names and
the recipe's CMake options. All three are fixed, none registered, and no pass followed: the
cycle ended by its stopping rule, the 58 recorded mutants caught at the last fix and 866 tests
green.

The fix method failed three times in use: the mutants' exit 4 on a test needing `.git`, the
builds a recipe had made before its change, and the recorded mutants a later batch left gone.
Each is in `failures_in_use` and fixed at its line. Its other steps held: the input class caught
a defect of a settled fix, and the sibling paths four statements another one left.
`shared-and-specific.md` records what every repository shares, approved by the user.

What comes next needs to know first that 0.2.0 stays unreleased until CI/CD runs at each tag,
and that each fix batch now runs the whole record of mutants after its commit, two minutes at 58
mutants.
