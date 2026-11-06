# Review: 2026-11-06-review-skill-v2-closing-pass

**Status:** complete — batch 1 fixed, findings 2 and 3, and the record at `test(reviews): record mutant M18 again on its moved line`: 53 mutants caught; batch 2, finding 1, fixed at `fix(build): compare a current build's CMake options with its recipe's` by the verifier's settled fix, the user's choice: the record of 58 mutants caught at that commit
**Scope:** roadmap `review-skill-v2`, `docs(review-skill-v2): create the roadmap` of 2026-10-31..`docs(review-skill-v2): record the session's close`: 138 hunks, +1464 −170 lines of code
**Go-ahead:** the user, tier `high`

## Summary

| # | Title | Axis | Effect | Severity | Origin | Verdict | Action | Batch |
|---|---|---|---|---|---|---|---|---|
| 1 | the stale-build check compares only patch names, so a llama.cpp build compiled with the bucket's web UI before LLAMA_USE_PREBUILT_UI=OFF stays current with no word | security, bugs | wrong action | low | new | CONFIRMED | fix | 2 |
| 2 | winnow is locked at 1.0.4 where Cargo.toml's requirements take 1.0.5 | dependencies | maintenance | — | older | — | fix at once | 1 |
| 3 | the recorded mutant M18 is gone: batch 11 reworded the line it replaces, and no run of the whole record followed | tests | maintenance | — | new | — | fix at once | 1 |

## Checks

- `cargo fmt --check`: exit 0
- `cargo clippy --all-targets -- -D warnings`: exit 0
- `cargo test`: exit 0: 861 passed, 0 failed, 0 ignored; no test skipped itself
- `python3 .claude/skills/reviewing-delivered-code/evals/test_scope.py`: exit 0
- `python3 .claude/skills/reviewing-delivered-code/evals/test_fixtures.py`: exit 0
- `python3 .claude/skills/reviewing-delivered-code/evals/test_findings.py`: exit 0
- `python3 .claude/skills/reviewing-delivered-code/evals/test_regression.py`: exit 0
- `python3 .claude/skills/reviewing-delivered-code/evals/test_dependencies.py`: exit 0
- `python3 .claude/skills/reviewing-delivered-code/evals/test_guard.py`: exit 0
- `sid.py check .`: exit 0
- `cargo deny check`: exit 0: advisories ok, bans ok, licenses ok, sources ok
- `.claude/skills/reviewing-delivered-code/scripts/dependencies.py docs/reviews/delivered-code/2026-11-06-review-skill-v2-closing-pass`: exit 0: 65 crates, 9 direct; 1 update within the requirements, 9 behind the newest, 0 yanked, 12 last released over a year ago; 3 moved
- `.claude/skills/reviewing-delivered-code/scripts/regression.py`: exit 1: 53 mutants at the scope's end: 52 caught, 0 missed, 1 gone (M18), 0 ambiguous, 0 unviable, 0 failing
- `cargo mutants --in-diff` on the scope's diff: 100 mutants in 312 s, 0 missed, 91 caught, 9 unviable

## Agents

- `review-security`, effort `high`: answered
- `review-verifier`, effort `high`: answered
- `review-dependencies`, effort `high`: answered
- `session`, effort `high`: answered

## Findings

### 1. the stale-build check compares only patch names, so a llama.cpp build compiled with the bucket's web UI before LLAMA_USE_PREBUILT_UI=OFF stays current with no word

```
FINDING
axis: security, bugs
severity: low
where: src/application/build.rs:324-342 (missing_patches), src/application/doctor.rs:404-408 (Doctor::build_check), src/application/gateway_config.rs:118-126 (Configurator::runtime), src/domain/recipe.rs:177-180 (LLAMA_CPP), src/domain/build.rs:208-215 (cmake_flags), docs/security.md:296-315 (top level), src/application/build.rs:328-339 (missing_patches), src/application/doctor.rs:361-420 (Doctor::build_check), src/application/gateway_config.rs:111-130 (Configurator::runtime), src/domain/recipe.rs:169-181 (LLAMA_CPP), docs/security.md:296-308 (top level)
title: the stale-build check compares only patch names, so a llama.cpp build compiled with the bucket's web UI before LLAMA_USE_PREBUILT_UI=OFF stays current with no word
cause: source: a build installed by an earlier ai-ctl, its build.json recording the recipe's CMake flags of that time and the names of its patches. Path: Doctor::build_check and Configurator::runtime call missing_patches, the one check that a current build is still built as its recipe says; missing_patches only looks for each name of recipe.patches in build.patches. It never compares recipe.cmake_flags with build.cmake_flags, where cmake_flags (domain/build.rs:208) copies the recipe's flags, so a pre-fix llama.cpp build has no LLAMA_USE_PREBUILT_UI key and was built with upstream's default, ON; nor a patch's content with the copy the build keeps in its patches/, so a patch rewritten under the same name (`fix(build): rebase the sd.cpp and audio.cpp patches on their releases` of 2026-10-28 rebased them that way) passes too. Sink: doctor's Ok on that build, and up and gateway config writing it into llama-swap's configuration as each llama-server model's cmd, with LLAMA_API_KEY in its environment.
effect: wrong action
consequence: who attacks: whoever writes the Hugging Face bucket ggml-org/llama-ui, row 6 of security.md §1 (a release or source ai-ctl runs). Input: a dist.tar.gz under the tag or `latest`, compiled into a llama.cpp build any ai-ctl before this one made; b12111 of 2026-11-04 took `latest` that way. Control that should stop it: finding 1 of audio-image-backends' review fixed the recipe and said "rebuild the current build"; the roadmap added a check for builds made before a recipe change, but only for patches, and the CHANGELOG entry tells no one to rebuild llama.cpp. Boundary and resource: on any other install, or after ai-ctl is updated, the build with content outside the commit build.json records keeps serving every llama-server model with the key in its environment, doctor says Ok and up starts it. Result: a control weakened with no damage shown; this machine's b12111 was rebuilt by hand on 2026-11-06 (its build.json has LLAMA_USE_PREBUILT_UI=OFF, its build.log says "UI: embedded 0 assets"), and every installed build's patch copies match the recipe's today.
fix: extend missing_patches into one check that a build is built as its recipe says: each (name, value) of recipe.cmake_flags present and equal in build.cmake_flags, and each patch's diff equal to the copy in the build's patches/ (or a digest recorded in build.json), the message naming what differs. doctor and Configurator::runtime already use it, with the same rebuild command.
test: a doctor test with a current llama.cpp build whose cmake_flags lack LLAMA_USE_PREBUILT_UI expects the warning and `ai-ctl build llama.cpp --ref b11538 --force`, and a Configurator test expects the refusal; both fail today, missing_patches(&LLAMA_CPP, &b11538()) being None while b11538's cmake_flags are empty (build.rs test a_build_lacks_each_patch_its_recipe_applies_now, line 1576).
evidence: read: build.rs:328-342 reads only recipe.patches and build.patches; git grep finds no read of build.cmake_flags in src/application but the write at build.rs:179; recipe.rs:177-180 adds the flag; ~/.local/share/ai-ctl/builds/llama.cpp/b12111/build.json now holds OFF only because of the hand rebuild of 2026-11-06T06:27:16Z; CHANGELOG.md line 8 gives no rebuild step.
```

Reviewers: review-security, session. Origin: new. Floor: above. Fingerprint: `security | docs/security.md + src/application/build.rs + src/application/doctor.rs + src/application/gateway_config.rs + src/domain/build.rs + src/domain/recipe.rs | Configurator::runtime + Doctor::build_check + LLAMA_CPP + cmake_flags + missing_patches + top level`.
Verdict: CONFIRMED. Action: fix, batch 2. verifier: the CMake-option half reproduced, three tests red on their assertions at the end (missing_patches None, doctor Ok, Configurator::generate Ok on b11538 without LLAMA_USE_PREBUILT_UI); severity low kept, a control weakened with no damage shown, every current build on this machine holding its recipe's options and patches. The patch-content half refuted: the rebase of audio.cpp's api-key.patch in `fix(build): rebase the sd.cpp and audio.cpp patches on their releases` of 2026-10-28 keeps its added lines, and the rebased patch does not apply to v0.9.2, so a content comparison would refuse a v0.9.2 build made as intended and its own remedy; a changed patch takes a new name, as reuse-address.patch did. Settled fix in repro/1/: missing_patches becomes unlike_recipe, each recipe option absent or at another value named beside each missing patch, the builder's options and a dropped recipe option left alone, security.md saying a patch is matched by its name; the fixtures standing for a build as its recipe makes it carry the recipe's options. Hard case for the user: the reviewer's fix compared each patch's content too, the verifier's does not. Left to the fix: a mutant on the join of a patch part and an option part; recipes_leave_toolchain_and_cuda_options_to_the_builder extended to every builder name. Fix: the verifier's, the user's choice on 2026-11-06. Sibling paths: doctor's Builds check and Configurator::runtime, which gateway config and up use, are the only readers of a current build, both guarded; README.md (twice) and docs/commands.md (twice) stated the patch-only check, now the options too. Input class: several options unset with one at another value, an option the recipe no longer sets, a build recording no option, a patch and an option missing together; the builder overriding no option of any recipe, which the prefix test let through for GGML_NATIVE. Mutants: 11 by cargo mutants, 9 caught, 2 unviable; by hand M54 to M58; M28 and M29, gone with the rename, recorded again.
Fixed in `fix(build): compare a current build's CMake options with its recipe's`.

### 2. winnow is locked at 1.0.4 where Cargo.toml's requirements take 1.0.5

```
FINDING
axis: dependencies
where: Cargo.lock:563 (winnow)
title: winnow is locked at 1.0.4 where Cargo.toml's requirements take 1.0.5
cause: winnow 1.0.5, released 2026-11-05, falls within the requirement toml_parser 1.1.5 puts on winnow (toml_parser comes in through toml = "1.1.8"), and Cargo.lock was not updated after it came out
effect: maintenance
consequence: no effect today, since no advisory names winnow 1.0.4; the lock stays one patch release behind what the requirements allow, and toml's parser misses that release's fixes until someone runs cargo update
fix: cargo update -p winnow
test: none: this only bumps the lock within the requirements; cargo test and cargo deny check run again after it
evidence: read: dependencies.json, winnow: version 1.0.4, update 1.0.5, newest 1.0.5, last_release 2026-11-05, yanked false; Cargo.lock:562-566 [[package]] name = "winnow" version = "1.0.4"; Cargo.lock:521-527 toml_parser depends on winnow; dependencies.out: "update: winnow 1.0.4 -> 1.0.5"
```

Reviewers: review-dependencies. Origin: older. Floor: below. Fingerprint: `dependencies | Cargo.lock | winnow`.
Verdict: none, below the floor. Action: fix at once, batch 1.
Fixed in `build(deps): update winnow to 1.0.5`.

### 3. the recorded mutant M18 is gone: batch 11 reworded the line it replaces, and no run of the whole record followed

```
FINDING
axis: tests
where: docs/reviews/delivered-code/mutants.json:173-182 (top level), src/infrastructure/git.rs:293 (update_within)
title: the recorded mutant M18 is gone: batch 11 reworded the line it replaces, and no run of the whole record followed
cause: M18, recorded by batch 2 on finding 36, replaces `    .with_context(|| format!("cannot check out the submodules {}", held.join(", ")))?;` in update_within. Batch 11 (`fix(build): clone again a submodule a kill left without a commit` of 2026-11-05, finding 11) split that statement into `updated.with_context(...)?;`, and each batch ran scripts/regression.py on its own mutants alone, so the text left the file unseen until this pass
effect: maintenance
consequence: no effect today: a_submodule_that_cannot_be_fetched_fails_the_checkout still kills the same mutant on the new line; the regression run no longer guards that `?` until M18 is recorded again
fix: record M18 again on the line as batch 11 left it, `    updated.with_context(|| format!("cannot check out the submodules {}", held.join(", ")))?;` replaced by the same call ending in `.ok();`, its test unchanged
test: none: the record's own run, scripts/regression.py M18, reports it caught
evidence: reproduced: `TMPDIR=<scratch> scripts/regression.py` at the scope's end, `gone: M18 src/infrastructure/git.rs: its text no longer occurs`, 52 of 53 caught; in a scratch clone with M18 recorded again as the fix says, `regression.py M18 --repo <clone>`: caught, the test failing with the mutant
```

Reviewers: session. Origin: new. Floor: below. Fingerprint: `tests | docs/reviews/delivered-code/mutants.json + src/infrastructure/git.rs | top level + update_within`.
Verdict: none, below the floor. Action: fix at once, batch 1.
Fixed in `test(reviews): record mutant M18 again on its moved line`.

## Facts

- `review-dependencies`: advisories: none reported by cargo deny check (exit 0); licences, bans and sources ok; 0 of 65 crates yanked
- `review-dependencies`: duplicates: syn 2.0.119 (Cargo.lock:422) and 3.0.9 (Cargo.lock:433), thiserror 1.0.69 (Cargo.lock:457) and 2.0.21 (Cargo.lock:466), thiserror-impl 1.0.69 (Cargo.lock:475) and 2.0.21 (Cargo.lock:486). The older versions all come from nvml-wrapper 0.13.0, which is its newest release, through wrapcenum-derive -> darling 0.20 and its own thiserror 1. ai-ctl cannot remove them; they cost only compile time
- `review-dependencies`: behind the newest, outside the requirements, with no advisory and none yanked: darling, darling_core and darling_macro 0.20.11 (newest 0.24.1, Cargo.lock:144, 154, 168), syn 2.0.119 (3.0.9), thiserror and thiserror-impl 1.0.69 (2.0.21), libloading 0.8.9 (0.9.0, Cargo.lock:264). All are pinned by nvml-wrapper 0.13.0 or nvml-wrapper-sys 0.10.0. r-efi 6.0.0 (7.1.0, Cargo.lock:339, through getrandom <- tempfile, dev only, UEFI only) and windows-link 0.2.1 (0.100.0, Cargo.lock:548, Windows only) are not built for the only target, x86_64-unknown-linux-gnu
- `review-dependencies`: no release in over a year, no RustSec warning, repository not archived: ident_case 1.0.1 (2019-03-18, Cargo.lock:230), static_assertions 1.1.0 (2019-11-03, Cargo.lock:410, repository last pushed 2023-11-18), fnv 1.0.7 (2020-05-14, Cargo.lock:201, repository last pushed 2024-03-20), wrapcenum-derive 0.4.1 (2024-02-10, Cargo.lock:569, 1 dependent, repository last pushed 2024-02-10), heck 0.5.0 (2024-03-12), strsim 0.11.1 (2024-04-02), utf8parse 0.2.2 (2024-06-08), errno 0.3.14 (2025-09-09, dev only through rustix <- tempfile), windows-sys 0.61.2 (2025-10-06, Windows only), is_terminal_polyfill and once_cell_polyfill 1.70.2 (2025-10-21), libloading 0.8.9 (2025-11-05). They are small and finished, or proc-macro helpers of nvml-wrapper's build; ident_case, fnv, static_assertions and wrapcenum-derive all come in through nvml-wrapper
- `review-dependencies`: moved repositories: nvml-wrapper and wrapcenum-derive moved from Cldfire to rust-nvml, and static_assertions from nvzqz/static-assertions-rs to nvzqz/static-assertions. None is archived, and nvml-wrapper had a release on 2026-08-31
- `review-dependencies`: nvml-wrapper 0.13.0 (direct, Cargo.toml:17): 145 dependents, 3 owners, 3 releases in the past year. It brings the most unsafe code of the direct dependencies (317, plus 794 in nvml-wrapper-sys and 93 in libloading), which is expected of the NVML FFI it exists to wrap
- `review-dependencies`: largest unsafe counts in the rest of the lock: windows-sys 12530 (not built on this target), linux-raw-sys 6930 and rustix 1610 (dev only, through tempfile), libc 800, hashbrown 455
- `review-dependencies`: llama-swap: pinned v263 (src/domain/gateway.rs:21, PINNED); newest tag of https://github.com/mostlygeek/llama-swap is v270; 7 releases between: v264, v265, v266, v267, v268, v269, v270
- `review-dependencies`: llama.cpp: current build b12111 (built 2026-11-06); newest b<number> tag of https://github.com/ggml-org/llama.cpp.git is b12164; 34 releases between: b12114 to b12152, then b12164
- `review-dependencies`: whisper.cpp: current build v1.9.5; newest vX.Y.Z tag of https://github.com/ggml-org/whisper.cpp.git is v1.9.5; no lag
- `review-dependencies`: stable-diffusion.cpp: current build master-1024-77b72a2; newest master-<N>-<hash> tag of https://github.com/leejet/stable-diffusion.cpp.git is master-1052-f5df5f8; 18 releases between: master-1025-8ee08e3 to master-1052-f5df5f8
- `review-dependencies`: audio.cpp: current build v0.9.3; newest vX.Y.Z tag of https://github.com/0xShug0/audio.cpp.git is v0.9.4; 1 release between: v0.9.4

## Coverage

- `review-security`: in full: src/domain/safetensors.rs (production part), src/infrastructure/git.rs (lines 1-460), docs/security.md; in part: src/application/doctor.rs (lines 316-425, 440-640, 905-920), src/application/build.rs (lines 160-360), src/application/gateway_config.rs (lines 1-150), src/application/bench.rs (lines 300-385), src/domain/gateway_config.rs (lines 160-560), src/domain/inventory.rs (lines 100-120, 340-420, 480-500, 595-690), src/domain/recipe.rs (diff), src/domain/build.rs (lines 205-225 and diff), src/domain/text.rs (diff), src/infrastructure/gateway_process.rs (diff, lines 55-75), src/infrastructure/host.rs (diff), src/infrastructure/files.rs (lines 20-40), src/infrastructure/build_store.rs (diff), src/infrastructure/run_store.rs (diff), src/presentation/cli.rs (diff), src/application/ports.rs (diff), src/application/lifecycle.rs (diff), src/application/fakes.rs (diff), docs/openwebui.md (diff), CHANGELOG.md (lines 1-10), docs/reviews/delivered-code/2026-11-04-audio-image-backends/findings.json (titles, finding 1), llama.cpp 52e53a76 tools/ui/CMakeLists.txt and scripts/ui-assets.cmake (lines 1-120), audio.cpp 41c1f8ff tools/cmake/espeak_static.cmake, CMakeLists.txt (lines 2865-2880), external/cpp-httplib/CMakeLists.txt (lines 30-50), the installed builds' build.json, build.log and patches/; left out: tests/repository.rs: a test over tracked files, no sink of the surface; the test hunks of doctor.rs, build.rs, git.rs, gateway_process.rs, host.rs, gateway_config.rs, recipe.rs, build_store.rs: read for what they hold, not for their own security; Cargo.lock: the dependency axis's
- `review-verifier`: in full: none; in part: src/application/build.rs (lines 1-30, 140-380, 1000-1040, 1545-1580, 2195-2235), src/application/doctor.rs (lines 300-425, 1090-1220), src/application/gateway_config.rs (lines 1-20, 95-135, 165-215, 395-440), src/domain/recipe.rs (lines 15-60, 150-330, 470-500), src/domain/build.rs (lines 180-240), src/infrastructure/build_store.rs (lines 40-200), src/application/fakes.rs (lines 1-30, 340-395), tests/gateway_lifecycle.rs (lines 1-30, 410-435), docs/security.md (lines 285-320), CHANGELOG.md (lines 1-10), ~/.local/share/ai-ctl/builds/*/*/build.json (cmake_flags and patches), git show on `fix(build): rebase the sd.cpp and audio.cpp patches on their releases` of 2026-10-28 (src/domain/patches), src/domain/patches/audio.cpp/api-key.patch at `feat(build): ship the audio.cpp recipe and its key patch` of 2026-10-28; left out: cargo mutants --in-diff on fix.patch: not run; the message's joins of a patch part and an option part (", and ") have no test of the two together, a mutant left to the phase that applies the fix; stable-diffusion.cpp's rebased patches against their older release: not checked; audio.cpp's case settles the point
- `review-dependencies`: in full: deny.txt, dependencies.out, docs/reviews/delivered-code/2026-11-06-review-skill-v2-closing-pass/dependencies.json, Cargo.toml, deny.toml; in part: Cargo.lock (the name lines of each crate cited, lines 521-527, 562-566), src/domain/gateway.rs (lines 1-70), src/domain/recipe.rs (lines 1-130, 160-305 by grep for project, repository and tags); left out: advisories: cargo deny check reported "advisories ok" under deny.toml's unmaintained = "all", unsound = "all", yanked = "warn", and no record of dependencies.json is yanked, so I had no function or type to search for in src/; release notes of llama-swap v264-v270, llama.cpp b12114-b12164, stable-diffusion.cpp master-1025 to master-1052 and audio.cpp v0.9.4: not read. git ls-remote gives only tags, and these rules allow no other request to GitHub, so I could not check whether a release in between fixes a defect ai-ctl meets. I raised no finding on the lag. Reading those notes is the only thing that could turn the lag into a finding; ai-ctl build list: ran target/debug/ai-ctl, built 2026-11-06 07:13 after `docs: bring the documents in line with the audio and image servers`, the last commit to touch src/; the scope's end changes only docs
- `session`: in full: none; in part: src/application/build.rs (lines 270-345), src/application/doctor.rs (lines 380-420), src/application/gateway_config.rs (lines 100-135), src/domain/build.rs (lines 105-135, 200-235), src/domain/recipe.rs (lines 165-190), docs/security.md (lines 295-320), src/infrastructure/git.rs (line 293), docs/reviews/delivered-code/mutants.json (lines 173-182); left out: none

## Fix Plan

- Batch 1: findings 2, 3
- Batch 2: findings 1
