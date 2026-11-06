# Review: 2026-11-04-audio-image-backends

**Status:** complete — the batches 1 to 16 fixed, each finding's fixed_in written, their mutants by hand recorded as M1 to M53 in docs/reviews/delivered-code/mutants.json; ten findings registered, one rejected; repro/ removed with the last fix
**Scope:** roadmap `audio-image-backends`, `docs(audio-image-backends): create the roadmap` of 2026-10-19..`docs(audio-image-backends): close the roadmap without phase 4` of 2026-10-30: 341 hunks, +4549 −294 lines of code
**Go-ahead:** the user, tier `high`

## Summary

| # | Title | Axis | Effect | Severity | Origin | Verdict | Action | Batch |
|---|---|---|---|---|---|---|---|---|
| 1 | a llama.cpp build compiles in a web UI archive downloaded with no pin, whose file names become native code in llama-server | security | wrong action | high | mixed | CONFIRMED | fix | 3 |
| 2 | the audio servers let a key holder name files on the host, which security.md's key-holder list does not say | security | misleading | low | mixed | CONFIRMED | fix | 15 |
| 3 | neither up nor doctor notices a current build that lacks a patch its recipe now holds | security | wrong action | low | mixed | CONFIRMED | fix | 4 |
| 4 | the documented OpenWebUI command runs a mutable image tag that then holds the key | security | misleading | low | new | CONFIRMED | fix | 16 |
| 5 | doctor fails an sd-server model whose tokenizer is a tokenizer.json | bugs | misleading | low | new | CONFIRMED | fix | 5 |
| 6 | doctor fails every safetensors index when models_dir holds `..` | bugs | misleading | low | new | CONFIRMED | fix | 6 |
| 7 | doctor demands a build for a backend no model uses | bugs | misleading | low | older | CONFIRMED | fix | 8 |
| 8 | doctor says the inventory runs no build when it could not read the inventory | bugs, error handling | misleading | low | older | CONFIRMED | fix | 8 |
| 9 | bench run takes a model of a server it cannot measure | bugs, error handling, the CLI's contract, tests | wrong action | low | older | CONFIRMED | fix | 9 |
| 10 | a build blames its patch for any failure to apply it | error handling | misleading | low | new | CONFIRMED | fix | 10 |
| 11 | a kill during a submodule's first fetch stops every later build of the project | interruption and concurrency, error handling | misleading | low | new | CONFIRMED | fix | 11 |
| 12 | doctor passes a model directory that is empty or half downloaded | bugs, interruption and concurrency | misleading | low | new | CONFIRMED | fix | 12 |
| 13 | a killed gateway's sd-server may be no leftover while its weights sit in RAM | bugs, interruption and concurrency | misleading | low | older | REFUTED | rejected | — |
| 14 | the README's check of the gateway's configuration always fails on the missing key variable | the CLI's contract | misleading | low | older | CONFIRMED | fix | 15 |
| 15 | init's help still says the models directory holds GGUF files | the CLI's contract | misleading | low | older | CONFIRMED | fix | 15 |
| 16 | the README's first install builds llama.cpp alone, and up refuses the inventory it installed | the CLI's contract | misleading | low | older | CONFIRMED | fix | 15 |
| 17 | gateway config's document leaves `mode` out of the file it writes for audiocpp_server | the CLI's contract | misleading | low | new | CONFIRMED | fix | 15 |
| 18 | two of the three lists of the state directory's files leave out the audiocpp configurations | the CLI's contract | misleading | low | older | CONFIRMED | fix | 15 |
| 19 | each server's document still says that ai-ctl does not serve it yet | conventions | misleading | low | new | CONFIRMED | fix | 15 |
| 20 | Fibo 2.0 Dev's settings in its document are not those the inventory runs | conventions | misleading | low | new | CONFIRMED | fix | 15 |
| 21 | the fixtures' list says no code reads a capture that a recipe test reads | conventions | misleading | low | new | CONFIRMED | fix | 15 |
| 22 | the domain's safetensors parser returns anyhow's errors, the only domain module that does | architecture, conventions | maintenance | — | new | — | fix at once | 1 |
| 23 | the toolchain's port now reads the models' directories and index files | architecture | maintenance | — | new | — | register | — |
| 24 | FakeToolchain's read_text fails through the absence of a text, not through a field named after it | conventions | maintenance | — | new | — | fix at once | 1 |
| 25 | the decision that whisper-server detects the language is dated in a code comment and absent from its document | conventions | maintenance | — | new | — | fix at once | 1 |
| 26 | The matrix grows combinatorially with small models, and past about 33 of them llama-swap outlasts up's wait | performance | slow | low | older | CONFIRMED | fix | 14 |
| 27 | A build of the latest release already installed checks out the sources and submodules before refusing | performance | slow | — | older | — | register | — |
| 28 | toml and toml_writer lag one release within Cargo.toml's requirements | dependencies | maintenance | — | older | — | fix at once | 1 |
| 29 | no test holds the quoting of a shard's name in the download command doctor prints | tests, security | wrong action | high | new | CONFIRMED | fix | 2 |
| 30 | a shard name starting with a dash becomes an option of `hf download`, and every test index avoids that shape | tests, security | wrong action | low | new | CONFIRMED | fix | 7 |
| 31 | the smoke test of a server printing on stdout has no test, though sd-server and audiocpp_server do | tests | wrong action | medium | new | CONFIRMED | fix | 2 |
| 32 | a device listed on stderr has no test | tests | maintenance | — | older | — | register | — |
| 33 | the checks of an audiocpp_server model's two paths have no test | tests | wrong action | low | new | CONFIRMED | fix | 2 |
| 34 | no test says which binary a smoke test runs, since FakePrograms drops its program | tests | wrong action | medium | older | CONFIRMED | fix | 2 |
| 35 | the submodule tests' recipes are two levels deep, sorted, and share no name's prefix | tests | maintenance | — | new | — | fix at once | 1 |
| 36 | a submodule that cannot be fetched has no test | tests | misleading | low | new | CONFIRMED | fix | 2 |
| 37 | the three "built as phase 0 built it by hand" tests assert neither the CMake options nor the repository | tests | wrong action | medium | new | CONFIRMED | fix | 2 |
| 38 | doctor's use of the gateway's directory has no test, since no doctor test serves a model whose configuration names it | tests | misleading | medium | new | CONFIRMED | fix | 2 |
| 39 | the message of several refusals has no test | tests | misleading | low | new | CONFIRMED | fix | 2 |
| 40 | a patch that cannot be kept with its build has no test | tests | misleading | low | new | CONFIRMED | fix | 2 |
| 41 | a model's file that cannot be written has no test | tests | wrong action | low | new | CONFIRMED | fix | 2 |
| 42 | a limit that cuts a character reports invalid UTF-8, and the test's text is ASCII alone | tests, error handling | misleading | low | new | CONFIRMED | fix | 13 |
| 43 | shard_path's arm for `.` can never run | dead and legacy code | maintenance | — | new | — | fix at once | 1 |
| 44 | write_model guards against an env that is never empty | dead and legacy code | maintenance | — | older | — | register | — |
| 45 | write_file refuses a path no production code can give it | dead and legacy code, tests | maintenance | — | new | — | fix at once | 1 |
| 46 | A model's ID is checked for characters the inventory has already refused | dead and legacy code | maintenance | — | new | — | fix at once | 1 |
| 47 | The missing-backend and missing-runtime branches run only in tests | dead and legacy code | none | — | new | — | register | — |
| 48 | The read of build.json format 1 is legacy code without its LEGACY comment | dead and legacy code | maintenance | — | new | — | fix at once | 1 |
| 49 | The saved runs' patches keep the old shape without a LEGACY comment | dead and legacy code | maintenance | — | mixed | — | fix at once | 1 |
| 50 | gateway_config's list copies inventory's, its element type aside | refactoring | maintenance | — | new | — | fix at once | 1 |
| 51 | recipe's new number repeats the digits-only rule of machine.rs and host.rs | refactoring | maintenance | — | new | — | register | — |
| 52 | check_audiocpp copies check_flags' finite-number rule | refactoring | maintenance | — | new | — | fix at once | 1 |
| 53 | Seven test modules each write a whole Build | refactoring | maintenance | — | mixed | — | register | — |
| 54 | The new kinds' test models and backends are written whole at each site | refactoring | maintenance | — | new | — | register | — |
| 55 | Ten test sites write the same Configurator over the fakes | refactoring | maintenance | — | mixed | — | register | — |
| 56 | stack.md still says one model runs at a time with the shipped inventory | conventions | misleading | low | older | CONFIRMED | fix | 15 |
| 57 | FakeApi's running fails through models set to None, not through a field named after it | conventions | maintenance | — | older | — | register | — |

## Checks

- `cargo fmt --check`: exit 0
- `cargo clippy --all-targets -- -D warnings`: exit 0
- `cargo test -- --nocapture`: exit 0: 828 passed, 0 failed, 0 ignored; no test skipped itself
- `python3 .claude/skills/reviewing-delivered-code/evals/test_scope.py`: exit 0
- `python3 .claude/skills/reviewing-delivered-code/evals/test_fixtures.py`: exit 0
- `python3 .claude/skills/reviewing-delivered-code/evals/test_findings.py`: exit 0
- `python3 .claude/skills/reviewing-delivered-code/evals/test_regression.py`: exit 0
- `python3 .claude/skills/reviewing-delivered-code/evals/test_dependencies.py`: exit 0
- `python3 .claude/skills/reviewing-delivered-code/evals/test_guard.py`: exit 0
- `sid.py check .`: exit 0
- `cargo deny check`: exit 0: advisories ok, bans ok, licenses ok, sources ok
- `.claude/skills/reviewing-delivered-code/scripts/dependencies.py docs/reviews/delivered-code/2026-11-04-audio-image-backends`: exit 0: 65 crates, 9 direct; 2 updates within the requirements, 9 behind the newest, 0 yanked, 11 last released over a year ago; 3 moved
- `cargo mutants --in-diff` on the scope's diff: 198 mutants in 582 s, 0 missed, 168 caught, 30 unviable

## Agents

- `review-security`, effort `high`: answered
- `review-correctness`, effort `high`: answered
- `review-structure`, effort `high`: answered
- `review-performance`, effort `high`: answered
- `review-tests`, effort `high`: answered
- `review-cleanup`, effort `high`: answered
- `review-dependencies`, effort `high`: answered
- `review-verifier`, effort `high`: answered
- `review-verifier`, effort `high`: answered
- `review-verifier`, effort `high`: answered
- `review-verifier`, effort `high`: answered
- `review-verifier`, effort `high`: answered
- `review-verifier`, effort `high`: answered
- `session`, effort `high`: answered
- `session`, effort `high`: answered
- `session`, effort `high`: answered

## Findings

### 1. a llama.cpp build compiles in a web UI archive downloaded with no pin, whose file names become native code in llama-server

```
FINDING
axis: security
severity: high
where: src/domain/recipe.rs:156-180 (LLAMA_CPP), src/domain/recipe.rs:242-248 (STABLE_DIFFUSION_CPP), docs/security.md:276-290 (top level), docs/security.md:34-39 (top level)
title: a llama.cpp build compiles in a web UI archive downloaded with no pin, whose file names become native code in llama-server
cause: source: the Hugging Face bucket ggml-org/llama-ui. LLAMA_CPP's cmake_flags leave LLAMA_USE_PREBUILT_UI at its default, ON, so the llama-server target runs scripts/ui-assets.cmake at compile time. It downloads dist.tar.gz, checks it only against dist.tar.gz.sha256 from the same place, and falls back from the tag to the mutable `latest` when the tag's files are missing. Path: file(ARCHIVE_EXTRACT) into the build tree, then emit_files, which writes each member's name unescaped into a C++ string of ui.cpp, `{ "${f}", asset_N, ... }`, and reads each member's content, following a symbolic link. Sink: the llama-server binary, which ai-ctl installs and llama-swap runs with the key in its environment. Nothing of it is in the commit build.json records. The sibling recipe closes the same door: STABLE_DIFFUSION_CPP sets SD_SERVER_BUILD_FRONTEND OFF because the frontend would fetch at build time; LLAMA_CPP has no such flag.
effect: wrong action
consequence: who attacks: whoever writes that bucket, row 6 of security.md §1 (a release or source ai-ctl runs), a writer the commit pin was meant to keep out. Input: an archive published under `latest` or a tag. Control crossed: §7's Build Sources, "`--ref <commit>` builds exactly the commit given" with build.json recording it; no digest of ai-ctl's or of the commit covers the archive. Resource reached and result: at the next `ai-ctl build llama.cpp`, a member named `a" ? (::system("..."), "b") : "c` runs a command at every llama-server start, in a process holding LLAMA_API_KEY and serving the prompts. A symbolic link member such as `leak.js -> ../../../../../../../state/ai-ctl/api-key` embeds the key file in the binary as a UI asset, and §2 lists the UI's files among the routes that answer without the key. It already takes the fallback: b12111, built 2026-11-04, embeds `latest`, not the assets of its own tag.
fix: add ("LLAMA_USE_PREBUILT_UI", "OFF") to LLAMA_CPP's cmake_flags, upstream's branch that builds without an embedded UI, rebuild the current build, and say in security.md §2 and §7 that llama-server has no web UI. If the UI is wanted, put tools/ui/dist in place from an archive whose SHA-256 the recipe carries, and refuse member names outside a plain charset and every link member before the compile.
test: llama_cpp_is_the_shipped_recipe asserts that recipe.cmake_flags holds ("LLAMA_USE_PREBUILT_UI", "OFF"); it fails today, the flag being absent
evidence: reproduced: in review-security/ui-probe, upstream's scripts/ui-assets.cmake and ui templates at b12111, no network: a tar.gz holding the eight required assets and a member named `a" ? (::system("touch pwned"), "b") : "c`, extracted by file(ARCHIVE_EXTRACT), then the script run with HF_ENABLED=OFF, which reaches the same emit_files("${DIST_DIR}") a download reaches. ui.cpp line 17 reads `{ "a" ? (::system("touch pwned"), "b") : "c", asset_0, ...`; g++ compiled it, and a one-line main linked with it created the file `pwned` at start. In link-probe, a member `leak.js -> ../../secret-outside` was extracted as a link, `zcat ui-gzip/_gzip/leak.js` printed the outside file's marker, and ui.cpp's table names "leak.js". In tar-probe, file(ARCHIVE_EXTRACT) refused `../dotdot-file` and `out/through-link`: extraction alone writes nothing outside, the name and the link are the way through. ~/.local/share/ai-ctl/builds/llama.cpp/b12111/build.log lines 413-417 show the download from b12111 failing on its sha256, then `UI: downloading from latest`, `archive verified and extracted`, `resolved: latest`. The real bucket and a whole llama-server compile were not run: a bucket cannot be impersonated from here, and the compile is `ai-ctl build`'s.
```

Reviewers: review-security. Origin: mixed. Floor: above. Fingerprint: `security | docs/security.md + src/domain/recipe.rs | LLAMA_CPP + STABLE_DIFFUSION_CPP + top level`.
Verdict: CONFIRMED. Action: fix, batch 3. the user, 2026-11-05: llama-server's web UI off, LLAMA_USE_PREBUILT_UI OFF, and the current b12111 rebuilt at the user's yes; the pinned archive set aside, its SHA-256 having no source but the bucket itself, and reversible by one flag the day a user needs the UI. Verifier: reproduced with upstream's ui-assets.cmake on a local bucket, anchor high (content ai-ctl fetches runs code or gets the key); the fallback to latest follows any failure of the tag; --no-ui at run time would not stop it, the fix being at the build; domain::build's cmake_flags test takes the new flag. Siblings: stable-diffusion.cpp's frontend is off already, and the build logs of whisper.cpp v1.9.5 and audio.cpp v0.9.3 show no download; the kept builds b11538 and b11700 embed their own tags' UI until pruned.
Fixed in `fix(build): build llama-server without its web UI`.

### 2. the audio servers let a key holder name files on the host, which security.md's key-holder list does not say

```
FINDING
axis: security
severity: low
where: docs/security.md:141-144 (top level), src/domain/recipe.rs:266-289 (AUDIO_CPP), src/domain/recipe.rs:187-213 (WHISPER_CPP), docs/security.md:17-23 (top level)
title: the audio servers let a key holder name files on the host, which security.md's key-holder list does not say
cause: source: a request through /upstream/<model>/ with the key. audio.cpp's build_openai_transcription_request reads `audio_path` or `file` as a host path into read_audio_buffer (app/server/server_protocol.cpp:1282-1298), a voice reference of type `path` does the same (:186-189), and whisper-server's POST /load loads a model from a host path. Sink: the file read as the user by a server on the host. ai-ctl's patches guard these routes with the key and leave them served; docs/audio/audio-cpp.md and whisper-cpp.md note them in passing, but §3 "What a key holder does" lists only loading, evicting, the log stream and emptying an sd-server.
effect: misleading
consequence: who attacks: code in a container that holds the key, OpenWebUI's (§1 row 3, §6), reaching the gateway at 10.0.2.2:8080. Input: JSON naming a path. Control weakened: the container's mounts, its one volume, as the limit of what it reads of the host. Resource reached: any WAV the user can read, returned as a transcript, and the existence of any path by the error; POST /load of a file that fails also ends whisper-server. No key, prompt or code is reached, so the damage stays that narrow.
fix: add these routes to §3's "What a key holder does", with OpenWebUI's container named as such a holder; refusing a host path in audio.cpp's patch is the stronger answer, if the user wants the container off those files
test: none: a document's text takes no test; a patch that refuses the path would take a probe of the built server
evidence: read: ~/.local/share/ai-ctl/sources/audio.cpp/app/server/server_protocol.cpp:1282-1298 and :186-189 at v0.9.3, request_dispatch.h's route table, docs/audio/audio-cpp.md "A transcription request may also name a WAV path on the host", docs/audio/whisper-cpp.md:52-53, docs/security.md:141-144
```

Reviewers: review-security. Origin: mixed. Floor: above. Fingerprint: `security | docs/security.md + src/domain/recipe.rs | AUDIO_CPP + WHISPER_CPP + top level`.
Verdict: CONFIRMED. Action: fix, batch 15. verifier: confirmed by reading, anchor low (a control weakened with no damage shown); the document's fix kept, a patch refusing host paths in audio.cpp set aside, its upkeep at each release out of proportion with WAV files and a path's existence. The routes read again in audio.cpp v0.9.3's sources, a JSON transcription's audio, audio_path or file and a voice_ref by path, and whisper.cpp v1.9.5's /load, which says model not found! and exits 1 on a failed load; written where the routes are, audio-cpp.md §3, and security.md §3 says what a key holder gets, linking to both.
Fixed in `docs: bring the documents in line with the audio and image servers`.

### 3. neither up nor doctor notices a current build that lacks a patch its recipe now holds

```
FINDING
axis: security
severity: low
where: src/application/doctor.rs:347-400 (Doctor::build_check), src/application/gateway_config.rs:111-129 (Configurator::runtime), src/application/build.rs:180-184 (Builder::build), docs/security.md:288-290 (top level)
title: neither up nor doctor notices a current build that lacks a patch its recipe now holds
cause: source: build.json's `patches`, written by Builder::build. Nothing reads it to decide: Configurator::runtime resolves the current build by its id and binary, and Doctor::build_check runs its smoke test alone. Sink: the llama-swap entry that runs that binary. A patch added to a recipe by a later ai-ctl reaches only the builds made after it; `reuse-address.patch` itself arrived after whisper.cpp's and stable-diffusion.cpp's first patch (`fix(build): bind whisper-server and sd-server as llama-server does` of 2026-10-28).
effect: wrong action
consequence: who attacks: another account of the machine or a web page, §1 rows 1 and 2, on whatever hole the next patch closes. Control weakened: §7's "no server is built without what its patch adds", which holds at the build and not at the run. Result: `up` serves the older build without a word, and doctor says it is healthy. No damage today: each current build's patches/ holds all of its recipe's.
fix: doctor fails a project the inventory runs when its current build's patches lack one of the recipe's, with `ai-ctl build <project> --ref <id> --force` as the remedy; the configuration refuses that build with the same message
test: a doctor test with a current whisper.cpp build whose patches lack `reuse-address.patch`, which passes today and must fail; the same build refused by Configurator::prepare
evidence: read: `grep -n patches src/application/*.rs` finds only the recording in build.rs:180-189 and the apply at build.rs:278 outside tests; doctor.rs:386-397 and gateway_config.rs:115-125 never read build.patches
```

Reviewers: review-security. Origin: mixed. Floor: above. Fingerprint: `security | docs/security.md + src/application/build.rs + src/application/doctor.rs + src/application/gateway_config.rs | Builder::build + Configurator::runtime + Doctor::build_check + top level`.
Verdict: CONFIRMED. Action: fix, batch 4. verifier: anchor low, both tests red at end; the fix as one shared function, application::build::missing_patches, used by Doctor::build_check and Configurator::runtime, so up, gateway config and doctor refuse such a build with the remedy ai-ctl build <project> --ref <id> --force; two existing tests take their recipe's patches, and two of batch 2's. Siblings: `build use` makes such a build current without a word, which doctor and up then report; bench runs a build only through the gateway up started. The input class, by a_build_lacks_each_patch_its_recipe_applies_now: none missing, two missing beside a patch the recipe dropped, a recipe with none and a build of format 1.
Fixed in `fix(build): refuse a current build without its recipe's patches`.

### 4. the documented OpenWebUI command runs a mutable image tag that then holds the key

```
FINDING
axis: security
severity: low
where: docs/openwebui.md:21-23 (top level), docs/openwebui.md:6 (top level), docs/security.md:334-337 (top level)
title: the documented OpenWebUI command runs a mutable image tag that then holds the key
cause: source: the tag ghcr.io/open-webui/open-webui:main, which docs/openwebui.md §1 gives to `podman run`, on a page that records the tested digest cut short, `sha256:4860ba00…`. Sink: a container into which the same page has the user paste the key, in five settings, its network leading to the host's loopback. §7 pins every image the inventory ships by digest, and `up` warns of a bare tag; this one has neither, ai-ctl not being what runs it.
effect: misleading
consequence: who attacks: whoever can move that tag, §1 row 6. Input: the image behind `:main` on a new machine or a pruned store. Control weakened: the digest pin §7 relies on for images. Result: an image other than the one the page was tested with holds the key and reaches the gateway and the backends. ai-ctl fetches nothing here and the user runs the command, so it stays a document's weakness.
fix: give the command as `ghcr.io/open-webui/open-webui:main@sha256:<the whole digest>`, the one the page was tested with
test: none: a document's command; the check is `podman image inspect` of the digest it names
evidence: read: docs/openwebui.md:6 and :21-23, docs/security.md:334-337
```

Reviewers: review-security. Origin: new. Floor: above. Fingerprint: `security | docs/openwebui.md + docs/security.md | top level`.
Verdict: CONFIRMED. Action: fix, batch 16. the user, 2026-11-05: the image now running, bbb14ede, checked with the page again and its whole digest pinned, its volume already migrated by it; the tested 4860ba00 set aside, an older image possibly unable to open that volume. Verifier: anchor low; podman shows :main already moved on this machine, the container openwebui running bbb14ede, created 2026-11-03, the tested 4860ba00 untagged. Pinned by the digest of its linux/amd64 manifest, 5defa464, as security.md §7 pins the shipped images, the index being 975e3965; checked with the page by comparing 0.13.0 with 0.12.3 from podman's store: the backend's requests to the gateway unchanged, the audio router by a comment, the image router by a header on fetching a reference image given by URL, the OpenAI router in the Responses API and the embeddings' prefix; the page's 42 labels in both frontends and its French ones in both locales. No other document names an image by its tag alone.
Fixed in `docs(openwebui): pin the image the guide was checked with by digest`.

### 5. doctor fails an sd-server model whose tokenizer is a tokenizer.json

```
FINDING
axis: bugs
severity: low
where: src/domain/inventory.rs:111-113 (BackendKind::reads_as_index), src/application/doctor.rs:473 (Doctor::model_check), src/application/doctor.rs:543-550 (Doctor::index_shards)
title: doctor fails an sd-server model whose tokenizer is a tokenizer.json
cause: reads_as_index judges a file by its extension alone and model_check drops the flag it belongs to, while sd-server's --tokenizer takes a tokenizer.json its model loader never reads as an index
effect: misleading
consequence: with `tokenizer = "tokenizer.json"` under [models.files], which sd.cpp's PiD, Lens, LLaDA, Ming and MOSS recipes need, doctor fails the model as "not a safetensors index: no `weight_map` object" and tells the user to download the same file again, which changes nothing; `up` serves the model
fix: give reads_as_index the flag as well as the file, and leave out `tokenizer`, the one file flag that does not go through sd.cpp's model loader
test: an sd-server model naming tokenizer.json under `tokenizer` passes model_check, its JSON read as no index
evidence: reproduced: an ad-hoc test in my copy, `cargo test --offline --lib adhoc_tokenizer_json`, gave (Fail, "/models/image/fibo/tokenizer.json is not a safetensors index: no `weight_map` object", the hf download of tokenizer.json); read: tests/fixtures/sd-server-master-975-help.txt, `--tokenizer`, "tokenizer.json path … required for PiD and Lens"
```

Reviewers: review-correctness. Origin: new. Floor: above. Fingerprint: `bugs | src/application/doctor.rs + src/domain/inventory.rs | BackendKind::reads_as_index + Doctor::index_shards + Doctor::model_check`.
Verdict: CONFIRMED. Action: fix, batch 5. siblings, read in stable-diffusion.cpp master-1024-77b72a2: the diffusion models, the VAEs, the text encoders, the upscaler, the detailer and the embeddings load through ModelLoader::init_from_file, which reads a .json as an index (model_loader.cpp:218); --tokenizer alone goes to the conditioner's own tokenizer. Its other form, main=FILE,clip-l=FILE,clip-g=FILE, is no path a model's files can name, out of this fix.
Fixed in `fix(doctor): read no sd-server tokenizer as a safetensors index`.

### 6. doctor fails every safetensors index when models_dir holds `..`

```
FINDING
axis: bugs
severity: low
where: src/application/doctor.rs:554-555 (Doctor::index_shards), src/domain/safetensors.rs:42-56 (shard_path), src/infrastructure/inventory_file.rs:53 (InventoryFile::install)
title: doctor fails every safetensors index when models_dir holds `..`
cause: shard_path resolves `..` in the whole path, then index_shards strips from it a model directory that keeps its `..`; `ai-ctl init --models-dir ../models` writes such a models_dir, since std::path::absolute keeps `..`
effect: misleading
consequence: with `ai-ctl init --models-dir ../models`, or the same path written by hand, doctor fails fibo-2.0-dev with "names /ai/models/image/fibo/…, outside /ai/x/../models/image/fibo" and gives a download of the index that cannot fix it, while sd-server, which normalises the path the same way, loads the shards
fix: normalise `dir` with the same function before strip_prefix, or compare the shard with the normalised index's directory
test: an index under a models_dir holding `..` whose shards are all present gives Status::Ok and counts them
evidence: reproduced: an ad-hoc test in my copy, `cargo test --offline --lib adhoc_dotdot_models_dir`, gave (Fail, "transformer/diffusion_pytorch_model.safetensors.index.json names /ai/models/image/fibo/transformer/…, outside /ai/x/../models/image/fibo", the hf download of the index), and std::path::absolute("../models") printed ".../copy/../models"; read: sd.cpp src/model_io/safetensors_io.cpp:28-34 in the local clone, lexically_normal
```

Reviewers: review-correctness. Origin: new. Floor: above. Fingerprint: `bugs | src/application/doctor.rs + src/domain/safetensors.rs + src/infrastructure/inventory_file.rs | Doctor::index_shards + InventoryFile::install + shard_path`.
Verdict: CONFIRMED. Action: fix, batch 6. verifier: the normalisation moves into a public safetensors::normal, shared by shard_path and index_shards, written without batch 1's arm for `.`, models_dir being absolute. Siblings: the inventory checks a model's paths and voices relative to models_dir, which no `..` of models_dir moves. Left: a symbolic link before models_dir's `..` makes the kernel's path, which doctor's file checks read, differ from the lexical one sd-server reads its shards from.
Fixed in `fix(doctor): count an index's shards under a models_dir holding ".."`.

### 7. doctor demands a build for a backend no model uses

```
FINDING
axis: bugs
severity: low
where: src/application/doctor.rs:322-332 (Doctor::build_checks), src/application/doctor.rs:372-377 (Doctor::build_check), src/application/gateway_config.rs:90-101 (Configurator::resolve)
title: doctor demands a build for a backend no model uses
cause: build_checks takes the projects the inventory runs from every declared backend, where Configurator::resolve, which `up` runs, takes only the backends a model uses
effect: misleading
consequence: an inventory with the shipped sd-native backend kept and its image models removed makes doctor fail "stable-diffusion.cpp: no build, and the inventory runs one" and fail the command, sending the user to compile a project nothing runs, while the same report says "Gateway configuration" is ok and `up` starts
fix: build `used` from the backends of inventory.models(), as resolve does
test: an inventory declaring sd-native with no model of it, and no build of stable-diffusion.cpp, skips that project's check
evidence: reproduced: an ad-hoc test in my copy, `cargo test --offline --lib adhoc_backend_no_model`, gave UNUSED (Fail, "no build, and the inventory runs one", "ai-ctl build stable-diffusion.cpp") and CONFIG (Ok, "1 models, on 127.0.0.1:8080")
```

Reviewers: review-correctness. Origin: older. Floor: above. Fingerprint: `bugs | src/application/doctor.rs + src/application/gateway_config.rs | Configurator::resolve + Doctor::build_check + Doctor::build_checks`.
Verdict: CONFIRMED. Action: fix, batch 8. one change to build_checks: the backends in use taken from the models, as resolve takes them, and an unread inventory said as such. The two settled fixes, each written beside its own test, merged by hand into one. Sibling: the failure of a build of a project with no recipe now holds for a backend in use alone, as up refuses only such a backend.
Fixed in `fix(doctor): need a build only for the projects a model runs on`.

### 8. doctor says the inventory runs no build when it could not read the inventory

```
FINDING
axis: bugs, error handling
severity: low
where: src/application/doctor.rs:378-380 (Doctor::build_check), src/application/doctor.rs:331 (Doctor::build_checks)
title: doctor says the inventory runs no build when it could not read the inventory
cause: build_checks reads a missing or invalid inventory as an empty set of projects, and build_check words that empty set as a fact about the inventory
effect: misleading
consequence: on a machine with no inventory or an invalid one, each project without a build reads "no build; the inventory does not run one", on a report whose Inventory check fails, when nothing is known of what the inventory runs
fix: tell build_check whether the inventory was read, and word the skip "no build; the inventory could not be read"
test: a broken inventory and no build give the skip's other wording
evidence: read: src/application/doctor.rs:322-332, the `_ => BTreeSet::new()` arm, and 378-380
```

Reviewers: review-correctness. Origin: older. Floor: above. Fingerprint: `bugs | src/application/doctor.rs | Doctor::build_check + Doctor::build_checks`.
Verdict: CONFIRMED. Action: fix, batch 8. one change to build_checks, with finding 7's. The input class, by its test: an invalid inventory and none installed.
Fixed in `fix(doctor): need a build only for the projects a model runs on`.

### 9. bench run takes a model of a server it cannot measure

```
FINDING
axis: bugs, error handling, the CLI's contract, tests
severity: low
where: src/application/bench.rs:314-337 (Bench::setup), src/application/bench.rs:168-186 (Bench::run), src/infrastructure/run_store.rs:536 (setup), src/infrastructure/server_api.rs:66-69 (LlamaServerApi::context_size), src/application/bench.rs:314-372 (Bench::setup), src/application/bench.rs:178-181 (Bench::run), docs/commands.md:437 (top level), src/presentation/cli.rs:136-137 (BenchAction), README.md:242 (top level), src/infrastructure/run_store.rs:536-537 (setup), src/infrastructure/run_store.rs:537 (setup), src/application/bench.rs:1927 (tests::a_model_the_inventory_lacks_is_refused)
title: bench run takes a model of a server it cannot measure
cause: the bench only speaks llama-server's /props, /tokenize and /apply-template, as run_store's comment assumes, but setup refuses no other kind, and docs/commands.md promises any model the gateway serves
effect: wrong action
consequence: `ai-ctl bench run whisper-large-v3`, or any audio or image model, passes setup, whose GET /upstream/<model>/props makes the gateway load that model and evict what the matrix cannot keep beside it, up to a chat model, then stops with "cannot read the context size" and an HTTP error that does not say the bench measures llama-server models only
fix: refuse in setup, before any request, a model whose backend kind is not llama-server, naming the kind; say so in docs/commands.md
test: bench run on a whisper-server model fails naming its kind, with no request sent to the server
evidence: read: src/application/bench.rs:314-337 and 168-186; no use of `kind` in bench.rs outside its tests
```

Reviewers: review-correctness, review-structure, review-tests. Origin: older. Floor: above. Fingerprint: `bugs | README.md + docs/commands.md + src/application/bench.rs + src/infrastructure/run_store.rs + src/infrastructure/server_api.rs + src/presentation/cli.rs | Bench::run + Bench::setup + BenchAction + LlamaServerApi::context_size + setup + tests::a_model_the_inventory_lacks_is_refused + top level`.
Verdict: CONFIRMED. Action: fix, batch 9. its verifier's test and fix in repro/9: setup refuses a backend other than llama-server before any request; README.md, docs/commands.md and bench run's help say a llama-server model, as review-structure asked, and commands.md's table of commands too. The test's whisper build takes its recipe's patches, which batch 4 asks. Siblings: logs and status speak to llama-swap, not to a server's API; a llama-server image backend is of the same kind and passes.
Fixed in `fix(bench): refuse a model of another server than llama-server`.

### 10. a build blames its patch for any failure to apply it

```
FINDING
axis: error handling
severity: low
where: src/application/build.rs:279-284 (Builder::apply_patches), src/infrastructure/git.rs:174 (GitSources::apply)
title: a build blames its patch for any failure to apply it
cause: apply_patches wraps every error of Sources::apply in "its patch does not apply", while apply also fails on writing its temporary file and on running git
effect: misleading
consequence: on a full disk, `ai-ctl build whisper.cpp` opens with "whisper.cpp v1.9.5 cannot be built: its patch api-key.patch does not apply; `--ref` builds a release it applies to", then gives the real cause at the end of the chain; the user is sent to another release that fails the same way
fix: let GitSources::apply tell a patch git refused from a failure to try it, for instance a domain error for the refusal, and give the `--ref` advice on that one alone
test: Sources::apply failing with an error other than the refusal gives no `--ref` advice
evidence: read: src/application/build.rs:279-284 and src/infrastructure/git.rs:170-181
```

Reviewers: review-correctness. Origin: new. Floor: above. Fingerprint: `error handling | src/application/build.rs + src/infrastructure/git.rs | Builder::apply_patches + GitSources::apply`.
Verdict: CONFIRMED. Action: fix, batch 10. its verifier's fix: a domain PatchRefused, which GitSources::apply returns when git ran and refused the patch, and on which alone apply_patches gives the --ref advice. git's own test now asserts the refusal's type, which the build's tests, on a double, cannot. Siblings: apply_patches is Sources::apply's only caller; git missing reads as a patch that could not be tried.
Fixed in `fix(build): blame a patch only when git refused it`.

### 11. a kill during a submodule's first fetch stops every later build of the project

```
FINDING
axis: interruption and concurrency, error handling
severity: low
where: src/infrastructure/git.rs:267-272 (update_within), src/infrastructure/git.rs:39-64 (GitSources::clone)
title: a kill during a submodule's first fetch stops every later build of the project
cause: the clone is made beside its place and renamed so that an interruption leaves nothing half made, but a submodule is cloned in place, under .git/modules, and git takes the half-made one as cloned
effect: misleading
consequence: `kill -9`, an OOM kill or a power cut during the first `ai-ctl build stable-diffusion.cpp`, while ggml or CUTLASS is fetched, leaves a submodule with no commit; from then on each build fails with "cannot check out the submodules ggml" and git's fatal line, saying nothing of the remedy, which is to remove the clone. Ctrl-C does not reach this state, git cleaning up on the signal
fix: on that failure, remove the submodule's .git/modules directory and its path and update once more; failing that, end the message as fetch's does, "remove <clone> and ai-ctl clones it again"
test: in a scratch superproject, a submodule whose git directory holds no object or ref is checked out on the next checkout
evidence: reproduced: in a scratch clone, after a clean `git submodule update --init --force -- ggml`, I emptied .git/modules/ggml's objects and refs, kept its config, left a tmp_pack file and removed the work file; the same update then exited 128, "fatal: Unable to find current revision in submodule path 'ggml'" (printed in French). The state was made by hand, not by a real kill
```

Reviewers: review-correctness. Origin: new. Floor: above. Fingerprint: `interruption and concurrency | src/infrastructure/git.rs | GitSources::clone + update_within`.
Verdict: CONFIRMED. Action: fix, batch 11. its verifier corrected the consequence: a kill -9 of ai-ctl alone leaves git running to the clone's end; the state takes a kill of the whole process group, an OOM kill of git or a power cut. The fix removes only <git dir>/modules/<name> holding a HEAD and no commit, found by .gitmodules' mapping, then updates once more. Added to it: the mapping read with git's --null, so that a name may hold a space, a name from upstream that climbs out of modules or is absolute left out, both on a capture listed in tests/fixtures/README.md; and two tests for the guards cargo mutants found untested, an unnamed submodule and one with commits both kept.
Fixed in `fix(build): clone again a submodule a kill left without a commit`.

### 12. doctor passes a model directory that is empty or half downloaded

```
FINDING
axis: bugs, interruption and concurrency
severity: low
where: src/application/doctor.rs:489 (Doctor::model_check), src/application/doctor.rs:530-531 (Doctor::model_check)
title: doctor passes a model directory that is empty or half downloaded
cause: for an audiocpp_server model that names no file, the check is is_dir alone
effect: misleading
consequence: after `mkdir`, or an `hf download` of qwen3-asr-1.7b or qwen3-tts stopped early, doctor says "the directory …" is ok, and `up` starts; the model fails only on the first request, in llama-swap's log
fix: require at least one entry other than hf's `.cache`, and report a stopped download's `.incomplete` files if hf's layout, to be checked by a probe, leaves them there
test: an existing empty directory of a model naming no file fails with the download command
evidence: read: src/application/doctor.rs:489 and 530-531; src/infrastructure/host.rs:202-204
```

Reviewers: review-correctness. Origin: new. Floor: above. Fingerprint: `bugs | src/application/doctor.rs | Doctor::model_check`.
Verdict: CONFIRMED. Action: fix, batch 12. its verifier corrected the consequence: hf 1.24.0 unlinks its .incomplete file on any failure, so only a download stopped before its first file is caught; the fix drops the .incomplete part and adds ToolchainProbe::entries. The input class added a directory that cannot be read: the settled fix then printed an empty remedy, now none, with FakeToolchain::entries_fails and SystemHost::entries tested on a scratch directory. Sibling left: an empty voices' directory, which a model of embedded voices does not need.
Fixed in `fix(doctor): fail a model directory holding nothing but hf's cache`.

### 13. a killed gateway's sd-server may be no leftover while its weights sit in RAM

```
FINDING
axis: bugs, interruption and concurrency
severity: low
where: src/application/lifecycle.rs:874-898 (Inspector::leftovers), src/application/lifecycle.rs:284-296 (Lifecycle::up), src/domain/gateway_config.rs:428-437 (sd_server)
title: a killed gateway's sd-server may be no leftover while its weights sit in RAM
cause: owners and leftovers look only at the GPU's process list, while each shipped image model now loads with --offload-to-cpu, "VRAM 0.00MB" at ready; the probe that put every kind in the leftovers ran on doubles, before that decision, and no phase re-ran it
effect: misleading
consequence: if sd-server at ready holds no CUDA context, a SIGKILL to llama-swap leaves it holding 16 to 21 GiB of RAM and its port, with `status` silent and `up` starting a new gateway, whose same model is then served by the orphan through its port. A server run with `no-gpu` or `backend = cpu` has this shape for certain. Whether sd-server does is what the probe below settles
fix: if the probe confirms it, also take as leftovers the processes running a build store's program, from /proc rather than from the GPU's list
test: a server of a build outside the GPU's list, no gateway running, is listed as a leftover and `up` refuses
evidence: read: src/application/lifecycle.rs:874-898, `gpu.processes`; docs/image/stable-diffusion-cpp.md:353-354 and 391; phase-2-inventory-and-gateway-report.md:354-362, the doubles and no live run. Not reproduced
```

Reviewers: review-correctness. Origin: older. Floor: above. Fingerprint: `bugs | src/application/lifecycle.rs + src/domain/gateway_config.rs | Inspector::leftovers + Lifecycle::up + sd_server`.
Verdict: REFUTED. Action: rejected. refuted by its verifier: a ready sd-server holds its CUDA context on the GPU whether or not its params sit there, 277 MiB measured at ready in docs/image/stable-diffusion-cpp.md:280-288, so Inspector::leftovers takes it by its build-store program; the VRAM 0.00MB the finding leaned on is sd.cpp's count of the params' buffer.

### 14. the README's check of the gateway's configuration always fails on the missing key variable

```
FINDING
axis: the CLI's contract
severity: low
where: README.md:160 (top level), docs/commands.md:326-327 (top level)
title: the README's check of the gateway's configuration always fails on the missing key variable
cause: docs/commands.md gained `AI_CTL_API_KEY=x` in front of `llama-swap -validate` and the same command in the README kept none
effect: misleading
consequence: whoever copies the README's second line reads "config validation failed" on a sound configuration, and looks for a fault in the inventory
fix: give README.md:160 the variable as docs/commands.md:326-327 gives it, or link to that section
test: none: a document's command; the run below is its proof
evidence: reproduced: llama-swap v263 `-validate -config tests/snapshots/gateway-shipped.yaml` with no variable prints "config validation failed: ... environment variable 'AI_CTL_API_KEY' is not set"; with AI_CTL_API_KEY=x it prints "config is valid: 13 model(s), 0 peer(s)"
```

Reviewers: review-structure. Origin: older. Floor: above. Fingerprint: `the CLI's contract | README.md + docs/commands.md | top level`.
Verdict: CONFIRMED. Action: fix, batch 15.
Fixed in `docs: bring the documents in line with the audio and image servers`.

### 15. init's help still says the models directory holds GGUF files

```
FINDING
axis: the CLI's contract
severity: low
where: src/presentation/cli.rs:25 (Command), docs/commands.md:110 (top level)
title: init's help still says the models directory holds GGUF files
cause: the option's table in docs/commands.md was changed to "GGUF, safetensors or ggml" and the option's doc comment, which clap prints, was not
effect: misleading
consequence: `ai-ctl init --help` describes a directory of GGUF files, where the inventory it installs reads safetensors and ggml files too
fix: the same words in the doc comment of `models_dir`
test: none: a help text; a test would read the text back, which CLAUDE.md's Tests excludes
evidence: reproduced: `ai-ctl init --help` at the scope's end prints "Directory holding the model files (GGUF), one subdirectory per model"; docs/commands.md:110 says "GGUF, safetensors or ggml"
```

Reviewers: review-structure. Origin: older. Floor: above. Fingerprint: `the CLI's contract | docs/commands.md + src/presentation/cli.rs | Command + top level`.
Verdict: CONFIRMED. Action: fix, batch 15.
Fixed in `docs: bring the documents in line with the audio and image servers`.

### 16. the README's first install builds llama.cpp alone, and up refuses the inventory it installed

```
FINDING
axis: the CLI's contract
severity: low
where: README.md:79-83 (top level), README.md:173 (top level), README.md:184-188 (top level), src/application/gateway_config.rs:90-101 (Configurator::resolve), deploy/inventory.toml:110-123 (top level), docs/commands.md:4 (top level)
title: the README's first install builds llama.cpp alone, and up refuses the inventory it installed
cause: the shipped inventory now runs four projects' builds and Configurator::resolve needs the current build of each project a model uses, but the README's walk still shows `ai-ctl build llama.cpp` then `ai-ctl up`, and its list of up's refusals leaves out a missing build
effect: misleading
consequence: a reader following the order docs/commands.md:4 announces gets `whisper.cpp has no build`, then the next project's error after building that one, and learns nowhere that serving only the LLMs means editing the inventory
fix: say after the build examples that the shipped inventory runs all four projects, list the four commands, say that a model removed from the installed inventory needs no build, and add a missing build to the refusals of README.md:184-188
test: none: a document; the_shipped_inventory_is_valid_once_rendered and the configurator's tests hold the behaviour
evidence: read: docs/commands.md:4, the README walks in the order of a first install; README.md:79-83, the build examples; src/application/gateway_config.rs:97-99 and src/application/build.rs:373, the error for each project; deploy/inventory.toml:219-221, :358, the models that use the other builds
```

Reviewers: review-structure. Origin: older. Floor: above. Fingerprint: `the CLI's contract | README.md + deploy/inventory.toml + docs/commands.md + src/application/gateway_config.rs | Configurator::resolve + top level`.
Verdict: CONFIRMED. Action: fix, batch 15. its verifier narrowed the consequence: the README walks doctor first, whose Builds check lists every missing build; the fix adds a missing build to up's refusals at README.md:184-188 and says that the shipped inventory runs builds of the four projects. The README lists the four builds after its examples, says a project no model of the installed inventory runs on needs none, and gives up's refusal of a missing build.
Fixed in `docs: bring the documents in line with the audio and image servers`.

### 17. gateway config's document leaves `mode` out of the file it writes for audiocpp_server

```
FINDING
axis: the CLI's contract
severity: low
where: docs/commands.md:310 (top level), src/domain/gateway_config.rs:482-486 (audiocpp_config)
title: gateway config's document leaves `mode` out of the file it writes for audiocpp_server
cause: the table lists the model's family, task, path, session_options and voices, where audiocpp_config also writes `mode`, which README.md:346 documents as a field of the model
effect: misleading
consequence: whoever reads the command's document for what reaches the server concludes that an inventory's `mode` is dropped
fix: add `mode` to the list in docs/commands.md:310
test: none: an_audiocpp_server_model_reads_the_configuration_ai_ctl_writes already tests the file; the gap is the document's
evidence: read: CLAUDE.md:18, a command's document changes with it; docs/commands.md:310; src/domain/gateway_config.rs:482
```

Reviewers: review-structure. Origin: new. Floor: above. Fingerprint: `the CLI's contract | docs/commands.md + src/domain/gateway_config.rs | audiocpp_config + top level`.
Verdict: CONFIRMED. Action: fix, batch 15.
Fixed in `docs: bring the documents in line with the audio and image servers`.

### 18. two of the three lists of the state directory's files leave out the audiocpp configurations

```
FINDING
axis: the CLI's contract
severity: low
where: README.md:203-209 (top level), docs/security.md:348-357 (top level), docs/commands.md:82 (top level), src/infrastructure/gateway_process.rs:65-75 (LocalGateway::write_file)
title: two of the three lists of the state directory's files leave out the audiocpp configurations
cause: the new `gateway/audiocpp/<model>.json` entered docs/commands.md's table and neither README.md's tree of the state directory nor docs/security.md §8, "Files ai-ctl keeps", which gives each file's mode and whether it holds a secret
effect: misleading
consequence: whoever checks from §8 what the state directory holds and how it is guarded finds an unlisted file there, with the models' paths and voices in it
fix: a row in docs/security.md §8, mode 0644 inside a 0700 directory, no secret; a line in the README's tree
test: none: documents; start_writes_the_files_the_models_read_before_the_gateway_starts tests the file and its directory's mode
evidence: read: docs/README.md:43, a document says what is true now; docs/commands.md:82, the row; docs/security.md:348-357 and README.md:203-209, no such file
```

Reviewers: review-structure. Origin: older. Floor: above. Fingerprint: `the CLI's contract | README.md + docs/commands.md + docs/security.md + src/infrastructure/gateway_process.rs | LocalGateway::write_file + top level`.
Verdict: CONFIRMED. Action: fix, batch 15.
Fixed in `docs: bring the documents in line with the audio and image servers`.

### 19. each server's document still says that ai-ctl does not serve it yet

```
FINDING
axis: conventions
severity: low
where: docs/audio/whisper-cpp.md:4 (top level), docs/audio/audio-cpp.md:5 (top level), docs/image/stable-diffusion-cpp.md:5 (top level), README.md:25-31 (top level), docs/gpu/llama-swap.md:55-63 (top level)
title: each server's document still says that ai-ctl does not serve it yet
cause: phase 1 wrote "and does not serve it yet" in the three documents, and phases 2 and 3, which made ai-ctl serve them, left the sentence
effect: misleading
consequence: the first lines of the document CLAUDE.md sends a reader to before touching a server contradict the README and docs/gpu/llama-swap.md, and each document's own "Behind llama-swap" section
fix: replace the clause in the three documents with what holds now: ai-ctl builds the server and the gateway serves it
test: none: documents
evidence: read: docs/README.md:43, a document says what is true now, a change to the code changing its document in the same commit; docs/audio/whisper-cpp.md:4, docs/audio/audio-cpp.md:5, docs/image/stable-diffusion-cpp.md:5, the sentence; deploy/inventory.toml:220, :247, :358, the shipped models that run them
```

Reviewers: review-structure. Origin: new. Floor: above. Fingerprint: `conventions | README.md + docs/audio/audio-cpp.md + docs/audio/whisper-cpp.md + docs/gpu/llama-swap.md + docs/image/stable-diffusion-cpp.md | top level`.
Verdict: CONFIRMED. Action: fix, batch 15.
Fixed in `docs: bring the documents in line with the audio and image servers`.

### 20. Fibo 2.0 Dev's settings in its document are not those the inventory runs

```
FINDING
axis: conventions
severity: low
where: docs/image/stable-diffusion-cpp.md:240-242 (top level), docs/image/stable-diffusion-cpp.md:312-315 (top level), deploy/inventory.toml:412-417 (top level)
title: Fibo 2.0 Dev's settings in its document are not those the inventory runs
cause: the settings still give the negative prompt "through `--negative-prompt-file`" and `--backend te=cpu`, where the shipped inventory passes `negative-prompt` inline and no `backend`, the same document saying further down that the encoder went back to the GPU on 2026-10-30
effect: misleading
consequence: whoever sets the model up by hand from this list measures another placement of the text encoder, 4.1 GB of VRAM away from the budget the inventory gives it
fix: state the inventory's settings in that list, `--negative-prompt` inline and `--offload-to-cpu` with the encoder on the GPU, leaving the te=cpu period to the dated paragraph at line 312
test: none: a document
evidence: read: docs/README.md:43, a document says what is true now; docs/image/stable-diffusion-cpp.md:240-242, the settings; docs/image/stable-diffusion-cpp.md:312-315, "the encoder is back on the GPU"; deploy/inventory.toml:412-417, the flags
```

Reviewers: review-structure. Origin: new. Floor: above. Fingerprint: `conventions | deploy/inventory.toml + docs/image/stable-diffusion-cpp.md | top level`.
Verdict: CONFIRMED. Action: fix, batch 15.
Fixed in `docs: bring the documents in line with the audio and image servers`.

### 21. the fixtures' list says no code reads a capture that a recipe test reads

```
FINDING
axis: conventions
severity: low
where: tests/fixtures/README.md:55-58 (top level), src/domain/recipe.rs:370 (tests::audio_cpp_is_built_as_phase_0_built_it_by_hand)
title: the fixtures' list says no code reads a capture that a recipe test reads
cause: the section names the tag lists, the two device listings, whisper-server's help and sd-server's version as read, and says "no code reads the others yet"; audiocpp_server's version, one of the others, has been read by the recipe's test since phase 1
effect: misleading
consequence: whoever updates the audio.cpp recipe or prunes the captures takes a capture that a smoke test's text depends on for an unused one
fix: name audiocpp_server's version among those read; the unread ones left are sd-server's help and audiocpp_server's help
test: none: a document
evidence: read: CLAUDE.md:53, each capture listed with its capture; tests/fixtures/README.md:55-58; src/domain/recipe.rs:370, the include_str! of audiocpp_server-v0.9.2-version.txt
```

Reviewers: review-structure. Origin: new. Floor: above. Fingerprint: `conventions | src/domain/recipe.rs + tests/fixtures/README.md | tests::audio_cpp_is_built_as_phase_0_built_it_by_hand + top level`.
Verdict: CONFIRMED. Action: fix, batch 15. The readers found by their include_str!: src/application/build.rs reads the two versions too; sd-server's and audiocpp_server's helps are the ones no code reads.
Fixed in `docs: bring the documents in line with the audio and image servers`.

### 22. the domain's safetensors parser returns anyhow's errors, the only domain module that does

```
FINDING
axis: architecture, conventions
where: src/domain/safetensors.rs:8 (top level), src/domain/safetensors.rs:17-36 (shards), src/application/doctor.rs:549-550 (Doctor::index_shards)
title: the domain's safetensors parser returns anyhow's errors, the only domain module that does
cause: shards was written with anyhow's Result, Context, bail and ensure, where every other domain module has errors of its own and the documented split gives anyhow to the application and the infrastructure
effect: maintenance
consequence: the domain now depends on anyhow, so its next error may do the same, and a caller that comes to branch on an index's fault has only text to match
fix: a thiserror enum in the domain for shards' refusals, with the same messages, which Doctor::index_shards keeps formatting
test: none: a change of type; an_index_without_a_weight_map_of_shards_is_refused and a_shard_that_would_drive_the_terminal_is_refused keep the messages
evidence: read: docs/architecture.md:94-96, thiserror in the domain, anyhow in the application and the infrastructure; CLAUDE.md:49; src/domain/safetensors.rs:8, the import, no other domain file importing anyhow by a search of src/domain
```

Reviewers: review-structure. Origin: new. Floor: below. Fingerprint: `architecture | src/application/doctor.rs + src/domain/safetensors.rs | Doctor::index_shards + shards + top level`.
Verdict: none, below the floor. Action: fix at once, batch 1.
Fixed in `refactor: apply batch 1 of audio-image-backends' review fixes`.

### 23. the toolchain's port now reads the models' directories and index files

```
FINDING
axis: architecture
where: src/application/ports.rs:71-74 (ToolchainProbe::is_dir, ToolchainProbe::read_text), src/application/ports.rs:62-64 (ToolchainProbe), src/application/doctor.rs:489-492 (Doctor::model_check), src/application/doctor.rs:545-548 (Doctor::index_shards), src/application/build.rs:87 (Builder)
title: the toolchain's port now reads the models' directories and index files
cause: doctor's model checks needed two more reads of the disk and they went into ToolchainProbe, documented as reading "the host's build toolchain", whose only caller of these reads, Doctor, uses them on models alone
effect: maintenance
consequence: the port holds two responsibilities, and Builder, which holds it for the toolkit and the compilers, carries reads of model files it never calls; every double of the toolchain grows with the models' needs
fix: a port of its own for the models' files, is_dir and read_text with doctor's use of is_file, implemented by SystemHost, held by Doctor alone
test: none: a move of methods; doctor's model tests stay and exercise the new port's double
evidence: read: docs/architecture.md:56-57, a port stays small, one responsibility, so that a use case holds only the ports it calls; src/application/ports.rs:62-63 and :72, the port's role and "A model's text file"; src/application/doctor.rs:489, :492 and :547, the only callers
```

Reviewers: review-structure. Origin: new. Floor: below. Fingerprint: `architecture | src/application/build.rs + src/application/doctor.rs + src/application/ports.rs | Builder + Doctor::index_shards + Doctor::model_check + ToolchainProbe + ToolchainProbe::is_dir, ToolchainProbe::read_text`.
Verdict: none, below the floor. Action: register. no local fix: a new port touches Doctor, SystemHost and every double, for two reads with one caller.

### 24. FakeToolchain's read_text fails through the absence of a text, not through a field named after it

```
FINDING
axis: conventions
where: src/application/fakes.rs:74-75 (FakeToolchain), src/application/fakes.rs:152-159 (FakeToolchain::read_text), src/application/doctor.rs:2115 (tests::an_index_that_cannot_be_read_fails_with_the_command_that_fetches_it_again)
title: FakeToolchain's read_text fails through the absence of a text, not through a field named after it
cause: the double fails for any path missing from `texts`, whose comment calls such a file "missing" while its error says "Permission denied", and a test makes it fail by clearing that map
effect: maintenance
consequence: a test that forgets a text gets a permission error it did not ask for, and the next reader cannot see from the double's fields that the call can be made to fail
fix: `read_text_fails: Option<PathBuf>`, what picks the call that fails, with a missing text a panic of the double; the comment dropped
test: none: a test double; an_index_that_cannot_be_read_fails_with_the_command_that_fetches_it_again sets the new field
evidence: read: docs/architecture.md:122-125, a double fails through a field named after its method, `<method>_fails`, with an error worded like the real adapter's; src/application/fakes.rs:74, the comment; src/application/fakes.rs:153-158, the error
```

Reviewers: review-structure. Origin: new. Floor: below. Fingerprint: `conventions | src/application/doctor.rs + src/application/fakes.rs | FakeToolchain + FakeToolchain::read_text + tests::an_index_that_cannot_be_read_fails_with_the_command_that_fetches_it_again`.
Verdict: none, below the floor. Action: fix at once, batch 1. its sibling FakeApi::running, failing through models set to None, is older code: finding 57, registered.
Fixed in `refactor: apply batch 1 of audio-image-backends' review fixes`.

### 25. the decision that whisper-server detects the language is dated in a code comment and absent from its document

```
FINDING
axis: conventions
where: docs/audio/whisper-cpp.md:55-59 (top level), docs/audio/whisper-cpp.md:193-194 (top level), src/domain/gateway_config.rs:526-536 (builtin_flags)
title: the decision that whisper-server detects the language is dated in a code comment and absent from its document
cause: builtin_flags cites "the user's decision, 2026-10-28", and docs/audio/whisper-cpp.md gives the facts behind it with neither the decision, its date nor what was set aside, where its §2 records its other decision as decided and dated
effect: maintenance
consequence: the next change to the default, such as for the short dictations the roadmap's summary says it misreads, has no record of what was weighed
fix: a "Decided" entry in docs/audio/whisper-cpp.md §2 with the date and the options set aside, the code comment linking to it
test: none: a document
evidence: read: docs/README.md:41-42, a decision is written in its domain's document with its date, the options set aside, the facts and their sources; src/domain/gateway_config.rs:527-528, the comment; docs/audio/whisper-cpp.md:55-59, the facts alone; docs/audio/whisper-cpp.md:72-74, the form of its other decision
```

Reviewers: review-structure. Origin: new. Floor: below. Fingerprint: `conventions | docs/audio/whisper-cpp.md + src/domain/gateway_config.rs | builtin_flags + top level`.
Verdict: none, below the floor. Action: fix at once, batch 1.
Fixed in `refactor: apply batch 1 of audio-image-backends' review fixes`.

### 26. The matrix grows combinatorially with small models, and past about 33 of them llama-swap outlasts up's wait

```
FINDING
axis: performance
severity: low
where: src/domain/gateway_config.rs:260 (largest_sets), src/domain/gateway_config.rs:199-226 (generate), src/domain/gateway_config.rs:228 (matrix), src/application/lifecycle.rs:32 (START_WAIT), src/application/doctor.rs:194 (Doctor::config_check)
title: The matrix grows combinatorially with small models, and past about 33 of them llama-swap outlasts up's wait
cause: largest_sets writes every largest combination of models that fit in the budget, one `fit-N` line each, with no bound; llama-swap's load time rises faster than the number of sets. This scope's speech models, which can share the GPU, start the growth: the inventory went from 2 models to 13, and 32 sets.
effect: slow
consequence: measured, shipped budgets plus models of 900 to 9000 MiB: 13 models 32 sets 100 ms to /health; 21 models 862 sets 122 ms; 23 models 2657 sets 170 ms; 27 models 10816 sets 652 ms; 33 models 62194 sets 22.7 s and 297 MB of RSS; 35 models 153480 sets 99.3 s. No cost on the shipped inventory. At 35 models, up's 30 s START_WAIT ends first: wait_until_healthy then reports that llama-swap did not answer and stops the gateway. doctor says nothing beforehand: its Gateway configuration check passed at 33 models.
fix: a bound on the sets in largest_sets, about 10000, where llama-swap still starts within a second, and a new ConfigError past it naming the model count, so that doctor's configuration check and up both refuse with the cause before any start.
test: generate on an inventory of 35 models with small budgets returns the new ConfigError, and the shipped inventory's 32 sets still pass; no timing in either.
evidence: reproduced: llama-swap v263 alone on 127.0.0.1:18765 with each model's cmd a touch, sets from a Python port of largest_sets whose count gives the snapshot's 32 on the shipped budgets, /health polled from the start, giving the times above. `ai-ctl doctor` under a scratch XDG_CONFIG_HOME with 33 models: 0.56 s, `ok Gateway configuration 33 models`, against 0.45 s at 13, so ai-ctl's own generation costs little. read: src/application/lifecycle.rs:538-600 for the wait and its failure; `ai-ctl up` itself was not run.
```

Reviewers: review-performance. Origin: older. Floor: above. Fingerprint: `performance | src/application/doctor.rs + src/application/lifecycle.rs + src/domain/gateway_config.rs | Doctor::config_check + START_WAIT + generate + largest_sets + matrix`.
Verdict: CONFIRMED. Action: fix, batch 14. the user, 2026-11-05: a bound of 20,000 sets in largest_sets, its ConfigError naming the sets and the models that share the GPU and saying to take models out of the inventory, refused by doctor's configuration check and by up before any start; 10,000 set aside, the difference being about two small models, and a warning alone, which leaves up to fail on its wait with no cause past about 53,000 sets. Measured: 10,816 sets start in 0.65 s, 53,000 in 19.5 s; 20,000, about 2 s by that curve, is not measured. Its verifier corrected the figure: the start outlasts up's 30 s between 53,000 sets (19.5 s) and 84,000 (53.5 s) on v263, 35 to 39 models by their budgets; the shipped 32 sets answer in 0.12 s. The early stop in largest_sets takes mutants no test without timing kills, to justify in the phase report. Fixed as decided, with how many models can share the GPU in the error. The input class: the bound itself, 19900 pairs, a model beside none and 99 over the budget making 20000 sets, passes, one more set is refused; a model that fits beside others but not beside its own size counts as sharing, which killed the one mutant cargo mutants missed. The sibling paths: doctor's configuration check, gateway config, up, status and bench reach generate through the Configurator alone. The early stop in largest_sets, forced off, leaves every result the same and the test slower, 0.74 s against 0.05 s: no test without timing kills it.
Fixed in `fix(gateway): refuse an inventory past 20000 sets of models together`.

### 27. A build of the latest release already installed checks out the sources and submodules before refusing

```
FINDING
axis: performance
where: src/application/build.rs:146 (Builder::build), src/application/build.rs:132-135 (Builder::build), src/application/build.rs:111-116 (Builder::build), src/infrastructure/git.rs:157 (GitSources::checkout), src/infrastructure/git.rs:226 (update_within)
title: A build of the latest release already installed checks out the sources and submodules before refusing
cause: latest_release names the tag at line 132, which is the build's id, but `taken` runs only at line 146, after checkout. The early refusal at lines 111-116 covers a release named with `--ref` only. This scope adds the submodule updates to that checkout.
effect: slow
consequence: measured for stable-diffusion.cpp at its current commit: 11 git processes, 1.1 to 1.3 s on tmpfs, spent before the refusal, on top of the fetch the answer does need. The wait is not noticeable beside the fetch. When the clone is at another commit, the checkout and the submodule updates can also download blobs over the network before refusing: the clone is partial, from the code; that case was not measured.
fix: when `reference` comes from latest_release and force is off, call `self.taken(recipe, &BuildId::parse(&reference)?, current.as_ref(), false)?` right after line 134: the newest release is also the newest tag of its commit, so this is the id line 136 would find, and the message stays the same.
test: with the newest release installed and no force, build fails with `is already built` and FakeSources records no checkout.
evidence: reproduced: on a scratch copy of ~/.local/share/ai-ctl/sources/stable-diffusion.cpp, with GIT_NO_LAZY_FETCH=1 and GIT_ALLOW_PROTOCOL=file, the sequence of checkout and update_within at its HEAD, timed twice: 1278 ms and 1135 ms, of which `git checkout` 679 and 429 ms and the cutlass `submodule update --force` 437 and 511 ms. read: src/application/build.rs:129-146 for the order.
```

Reviewers: review-performance. Origin: older. Floor: below. Fingerprint: `performance | src/application/build.rs + src/infrastructure/git.rs | Builder::build + GitSources::checkout + update_within`.
Verdict: none, below the floor. Action: register. older code: 1.1 to 1.3 s measured beside the fetch the answer needs; the case of a clone at another commit was not measured.

### 28. toml and toml_writer lag one release within Cargo.toml's requirements

```
FINDING
axis: dependencies
where: Cargo.lock:497 (toml), Cargo.lock:530 (toml_writer), Cargo.toml:21 (toml)
title: toml and toml_writer lag one release within Cargo.toml's requirements
cause: toml 1.2.0 and toml_writer 1.2.0 were published on 2026-11-04, after the lock took toml 1.1.8 and toml_writer 1.1.3; toml's requirement, "1.1.8", admits 1.2.0, and the requirement of toml_writer comes through toml
effect: maintenance
consequence: no effect today, and no advisory touches either version; the lock stays one minor release behind on the inventory's parser until it is taken
fix: cargo update -p toml -p toml_writer
test: none: an update of the lock changes no code of ai-ctl; the inventory's parsing tests and cargo deny check run on the new lock
evidence: read: dependencies.json: toml 1.1.8+spec-1.1.0 has update 1.2.0+spec-1.1.0 and requires null; toml_writer 1.1.3+spec-1.1.0 has update 1.2.0+spec-1.1.0 and requires null; no other record of the 65 has an update
```

Reviewers: review-dependencies. Origin: older. Floor: below. Fingerprint: `dependencies | Cargo.lock + Cargo.toml | toml + toml_writer`.
Verdict: none, below the floor. Action: fix at once, batch 1. verified by the session: dependencies.json gives update 1.2.0 for toml and toml_writer, within toml's requirement, neither yanked; cargo deny check exit 0.
Fixed in `build(deps): update toml to 1.2.0`.

### 29. no test holds the quoting of a shard's name in the download command doctor prints

```
FINDING
axis: tests, security
severity: high
where: src/application/doctor.rs:575 (download), src/application/doctor.rs:574 (download), src/domain/safetensors.rs:27-31 (shards), src/application/doctor.rs:2181 (tests::the_download_command_quotes_unusual_paths)
title: no test holds the quoting of a shard's name in the download command doctor prints
cause: the_download_command_quotes_unusual_paths uses a `--local-dir` with a space; every file, shard and repository name of every doctor test is plain, so nothing shows `shell_quote` applied to them
effect: wrong action
consequence: `words.extend(names.iter().map(|name| Cow::Borrowed(name.as_str())))` passes all 828 tests. Stands for a model's index, content the user fetches, naming the shard `a $(touch pwned).safetensors`: `safetensors::shards` refuses control characters only, so the name reaches the remedy doctor prints to be pasted, and the shell runs it. The same holds for the repository at line 574
fix: add the two tests below
test: a_shard_name_the_shell_would_read_is_quoted_in_the_download_command and a_repository_the_shell_would_read_is_quoted_in_the_download_command: pass on the code, each fails on its unquoted change
evidence: reproduced: copy at the scope's end, each change applied, `cargo test` all green; the new tests in review-tests/tests.diff fail on it at doctor.rs and pass on the code
```

Reviewers: review-tests. Origin: new. Floor: above. Fingerprint: `tests | src/application/doctor.rs + src/domain/safetensors.rs | download + shards + tests::the_download_command_quotes_unusual_paths`.
Verdict: CONFIRMED. Action: fix, batch 2. a test gap: its verifier's test in repro/29/test.patch, red on each of its mutant-*.patch. The command's third word, the directory, is held by the_download_command_quotes_unusual_paths.
Fixed in `test: cover the gaps audio-image-backends' review found`.

### 30. a shard name starting with a dash becomes an option of `hf download`, and every test index avoids that shape

```
FINDING
axis: tests, security
severity: low
where: src/domain/safetensors.rs:23-33 (shards), src/application/doctor.rs:571-586 (download), src/application/doctor.rs:2010 (tests::FIBO_INDEX)
title: a shard name starting with a dash becomes an option of `hf download`, and every test index avoids that shape
cause: Fibo's fixture and every test index sit in `transformer/`, so each shard's path starts with a directory; an index at the root of a model's directory gives the shard's name as it is, and neither `shards` nor `download` refuses a leading `-`
effect: wrong action
consequence: an index at the model's root naming `--cache-dir=/tmp/x` makes doctor print `hf download briaai/Fibo-2.0-dev --cache-dir=/tmp/x --revision <commit> --local-dir ...`; `hf download --help` lists `--cache-dir`, `--include`, `--exclude` and `--type`, so the index changes what the pasted command fetches and where. No code runs
fix: refuse in `safetensors::shards` a shard that starts with `-`, as it refuses a control character
test: a_shard_hf_would_read_as_an_option_never_reaches_the_download_command: fails on the code with the command above, passes with that refusal added
evidence: reproduced: the test in tests.diff on a copy at the scope's end prints the command above; `HF_HUB_OFFLINE=1 hf download --help` lists the options
```

Reviewers: review-tests. Origin: new. Floor: above. Fingerprint: `tests | src/application/doctor.rs + src/domain/safetensors.rs | download + shards + tests::FIBO_INDEX`.
Verdict: CONFIRMED. Action: fix, batch 7. its verifier corrected the fix: a refusal in safetensors::shards misses ./--x and the absolute spelling, which reach download as --x; the check goes in Doctor::index_shards, on the path relative to the model's directory, after batch 6's normalisation. The input class, by its test: the bare name, ./ and the absolute spelling. Siblings: a model's files and repo, which the inventory's author writes and which may start with `-` too, are left: that author decides what runs already (security.md §1), where an index comes with a model's download.
Fixed in `fix(doctor): refuse a shard hf download would read as an option`.

### 31. the smoke test of a server printing on stdout has no test, though sd-server and audiocpp_server do

```
FINDING
axis: tests
severity: medium
where: src/application/build.rs:328 (smoke_test_failure), src/application/fakes.rs:285-299 (FakePrograms::default), tests/fixtures/README.md:67-71 (top level)
title: the smoke test of a server printing on stdout has no test, though sd-server and audiocpp_server do
cause: FakePrograms prints llama-server's version and whisper-server's help on stderr, and every test's override prints on stderr too; recipe.rs reads the two other fixtures for their text, not through `smoke_test_failure`
effect: wrong action
consequence: `let printed = output.stderr.contains(test.prints);` passes all 828 tests, where it discards every build of stable-diffusion.cpp and audio.cpp after its compile, and fails their doctor check. cargo-mutants' `||` to `&&` dies only because stdout is empty
fix: add the test below
test: a_server_that_prints_on_stdout_passes_its_smoke_test, on the two captured versions: passes on the code, fails on the change
evidence: reproduced: change applied on a copy, `cargo test` green; the new test fails at build.rs in tests.diff
```

Reviewers: review-tests. Origin: new. Floor: above. Fingerprint: `tests | src/application/build.rs + src/application/fakes.rs + tests/fixtures/README.md | FakePrograms::default + smoke_test_failure + top level`.
Verdict: CONFIRMED. Action: fix, batch 2. a test gap: its verifier's test in repro/<n>/test.patch, red on each change in repro/<n>/mutant-*.patch.
Fixed in `test: cover the gaps audio-image-backends' review found`.

### 32. a device listed on stderr has no test

```
FINDING
axis: tests
where: src/application/build.rs:306 (Builder::smoke_test), src/application/build.rs:1363 (tests::a_build_without_a_cuda_device_is_discarded), src/application/fakes.rs:290-294 (FakePrograms::default)
title: a device listed on stderr has no test
cause: every listing a test gives is on stdout
effect: maintenance
consequence: `let devices = output.stdout.clone();` passes all 828 tests, and refuses a server that lists its devices on stderr; the captures of sd-server and audiocpp_server hold their devices on stdout, so no shipped server is known to meet it today
fix: add the test below
test: a_device_listed_on_stderr_counts: passes on the code, fails on the change
evidence: reproduced: change applied on a copy, `cargo test` green; the new test fails at build.rs in tests.diff
```

Reviewers: review-tests. Origin: older. Floor: below. Fingerprint: `tests | src/application/build.rs + src/application/fakes.rs | Builder::smoke_test + FakePrograms::default + tests::a_build_without_a_cuda_device_is_discarded`.
Verdict: none, below the floor. Action: register. its verifier corrected the effect to maintenance: no server ai-ctl builds lists its devices on stderr, so the change refuses no build today; its test waits in repro/32 for a recipe that does.

### 33. the checks of an audiocpp_server model's two paths have no test

```
FINDING
axis: tests
severity: low
where: src/domain/gateway_config.rs:481 (audiocpp_config), src/domain/gateway_config.rs:453 (audiocpp_server), src/domain/gateway_config.rs:1442 (tests::an_audiocpp_value_the_yaml_cannot_hold_is_refused)
title: the checks of an audiocpp_server model's two paths have no test
cause: an_audiocpp_value_the_yaml_cannot_hold_is_refused changes the family, mode, task, voices and session options, never `dir`, the `model` file or the gateway's directory; `files` is not called for this kind, so line 481 is the only check of `dir`
effect: wrong action
consequence: replacing `text.path(&weights)` or `text.path(&path)` with `to_string_lossy` passes all 828 tests. Then a `dir` holding U+2028 reaches the comment above the model in the YAML, whose parser ends a line there as the code's comment at line 579 says of the `cmd` block, and `${` in the state directory reaches `--config` as a macro. Not run through llama-swap
fix: add the test below
test: an_audiocpp_path_the_yaml_cannot_hold_is_refused: passes on the code, fails on each of the two changes
evidence: reproduced: each change applied on a copy, `cargo test` green; the new test fails at gateway_config.rs in tests.diff
```

Reviewers: review-tests. Origin: new. Floor: above. Fingerprint: `tests | src/domain/gateway_config.rs | audiocpp_config + audiocpp_server + tests::an_audiocpp_value_the_yaml_cannot_hold_is_refused`.
Verdict: CONFIRMED. Action: fix, batch 2. a test gap: its verifier's test in repro/33/test.patch, red on each of its mutant-*.patch. Its sibling, voice_dir's path, is held already by an_audiocpp_value_the_yaml_cannot_hold_is_refused.
Fixed in `test: cover the gaps audio-image-backends' review found`.

### 34. no test says which binary a smoke test runs, since FakePrograms drops its program

```
FINDING
axis: tests
severity: medium
where: src/application/build.rs:191 (Builder::build), src/application/doctor.rs:385 (Doctor::build_check), src/application/fakes.rs:306-318 (FakePrograms::run)
title: no test says which binary a smoke test runs, since FakePrograms drops its program
cause: `FakePrograms::run` answers by the arguments alone and keeps no record of the path
effect: wrong action
consequence: `self.smoke_test(recipe, &self.store.binary(recipe.project, &id, recipe.binary()))` at build.rs:191 passes all 828 tests: `build --force` then tests the installed binary and replaces it with the new one, untested, which may not start. `self.builds.binary(project, current, "llama-server")` at doctor.rs:385 passes too, and fails doctor on every other project's build
fix: FakePrograms records each program in `ran`; add the two tests below
test: the_smoke_test_runs_the_staged_binary_not_the_installed_one and each_projects_current_build_is_run_from_its_own_binary: pass on the code, each fails on its change
evidence: reproduced: each change applied on a copy, `cargo test` green; the new tests fail on it, fakes.rs change in tests.diff
```

Reviewers: review-tests. Origin: older. Floor: above. Fingerprint: `tests | src/application/build.rs + src/application/doctor.rs + src/application/fakes.rs | Builder::build + Doctor::build_check + FakePrograms::run`.
Verdict: CONFIRMED. Action: fix, batch 2. a test gap: its verifier's tests in repro/34/test.patch, red on each of its mutant-*.patch. The device listing, the build's other run, is held by the same test, which asserts both runs.
Fixed in `test: cover the gaps audio-image-backends' review found`.

### 35. the submodule tests' recipes are two levels deep, sorted, and share no name's prefix

```
FINDING
axis: tests
where: src/infrastructure/git.rs:195 (update_submodules), src/infrastructure/git.rs:218-220 (parent_of), src/infrastructure/git.rs:252 (update_within), src/infrastructure/git.rs:757 (tests::a_submodule_of_a_submodule_is_checked_out_within_it)
title: the submodule tests' recipes are two levels deep, sorted, and share no name's prefix
cause: every test's `submodules` is `["ggml"]`, `["ggml", "webm"]` or `["ggml", "ggml/vendors/cutlass"]`, as the shipped recipe is, and none names a tracked plain file
effect: maintenance
consequence: each of these passes all 828 tests: removing the sort at line 195, which skips a three-level recipe listed deepest first; `.next()` for `.max_by_key` at line 220, which takes `a` as the parent of `a/b/c`; `!rest.is_empty()` at line 218, which takes `a` as the parent of `a-extra`; dropping the mode check at line 252, which fails a recipe naming a plain file. Each leaves a named submodule empty or fails the checkout, on a recipe no shipped project has yet
fix: add the test below, over a fixture of nested submodules and in two orders
test: named_submodules_are_checked_out_at_any_depth_in_any_order: passes on the code, fails on each of the four changes; it runs `git` in scratch folders only
evidence: reproduced: each change applied on a copy, `cargo test` green; the new test fails on it at git.rs in tests.diff
```

Reviewers: review-tests. Origin: new. Floor: below. Fingerprint: `tests | src/infrastructure/git.rs | parent_of + tests::a_submodule_of_a_submodule_is_checked_out_within_it + update_submodules + update_within`.
Verdict: none, below the floor. Action: fix at once, batch 1. its test: repro/review-tests/tests.diff, by name, against the end commit.
Fixed in `refactor: apply batch 1 of audio-image-backends' review fixes`.

### 36. a submodule that cannot be fetched has no test

```
FINDING
axis: tests
severity: low
where: src/infrastructure/git.rs:268-272 (update_within), src/infrastructure/git.rs:646 (tests::a_recipes_submodules_are_checked_out_with_the_sources)
title: a submodule that cannot be fetched has no test
cause: every submodule test's upstream stays in place, so `git submodule update` always succeeds
effect: misleading
consequence: replacing the `?` of the update with `.ok()` passes all 828 tests: the checkout then succeeds with an empty `ggml`, and the user reads CMake's error about missing files in place of `cannot check out the submodules ggml`
fix: add the test below
test: a_submodule_that_cannot_be_fetched_fails_the_checkout, removing upstream's ggml after the fetch: passes on the code, fails on the change
evidence: reproduced: change applied on a copy, `cargo test` green; the new test fails at git.rs in tests.diff
```

Reviewers: review-tests. Origin: new. Floor: above. Fingerprint: `tests | src/infrastructure/git.rs | tests::a_recipes_submodules_are_checked_out_with_the_sources + update_within`.
Verdict: CONFIRMED. Action: fix, batch 2. a test gap: its verifier's test in repro/36/test.patch, red on its mutant-1.patch. Its siblings in update_within are left without a test: `submodule sync` fails only on a .gitmodules git cannot parse, which fails the update right after with git's same message, so swallowing it changes the message's first words alone; `clean` has no failure a test can provoke after a fresh update.
Fixed in `test: cover the gaps audio-image-backends' review found`.

### 37. the three "built as phase 0 built it by hand" tests assert neither the CMake options nor the repository

```
FINDING
axis: tests
severity: medium
where: src/domain/recipe.rs:202-205 (WHISPER_CPP), src/domain/recipe.rs:242-248 (STABLE_DIFFUSION_CPP), src/domain/recipe.rs:275-279 (AUDIO_CPP), src/domain/recipe.rs:363 (tests::audio_cpp_is_built_as_phase_0_built_it_by_hand)
title: the three "built as phase 0 built it by hand" tests assert neither the CMake options nor the repository
cause: the tests hold the binary, tags, submodules, smoke test, listing and patches; `recipes_leave_toolchain_and_cuda_options_to_the_builder` only reads the names' prefixes
effect: wrong action
consequence: emptying the three `cmake_flags` and pointing the three `repository` at another owner pass all 828 tests. Without `AUDIOCPP_DEPLOYMENT_BUILD`, whose absence the recipe's comment says keeps a model from a publisher's safetensors from starting, the build and its smoke test pass and the model fails on demand. Without `SD_SERVER_BUILD_FRONTEND=OFF`, the build runs `pnpm install`
fix: add the test below, or these assertions to each of the three tests
test: each_new_recipe_keeps_the_repository_and_options_it_is_built_with: passes on the code, fails on each change
evidence: reproduced: both changes applied on a copy, `cargo test` green; the new test fails at recipe.rs in tests.diff
```

Reviewers: review-tests. Origin: new. Floor: above. Fingerprint: `tests | src/domain/recipe.rs | AUDIO_CPP + STABLE_DIFFUSION_CPP + WHISPER_CPP + tests::audio_cpp_is_built_as_phase_0_built_it_by_hand`.
Verdict: CONFIRMED. Action: fix, batch 2. a test gap: its verifier's test in repro/37/test.patch, red on each change in its mutant-*.patch; review-tests' version fails clippy's type_complexity.
Fixed in `test: cover the gaps audio-image-backends' review found`.

### 38. doctor's use of the gateway's directory has no test, since no doctor test serves a model whose configuration names it

```
FINDING
axis: tests
severity: medium
where: src/application/doctor.rs:114 (Doctor::configurator), src/application/doctor.rs:965-981 (tests::GatewaySide::serving), src/domain/gateway_config.rs:449-453 (audiocpp_server)
title: doctor's use of the gateway's directory has no test, since no doctor test serves a model whose configuration names it
cause: every doctor test that runs a gateway has llama-server models alone, whose YAML holds no path under `files_dir`
effect: misleading
consequence: `files_dir: Path::new("/elsewhere")` at line 114 passes all 828 tests; on a gateway serving an audiocpp_server model, doctor then warns `on another configuration, address or release than the inventory, the builds and llama-swap give now`, where nothing changed
fix: add the test below
test: a_gateway_serving_an_audiocpp_model_runs_the_configuration_doctor_generates: passes on the code, fails on the change with that warning
evidence: reproduced: change applied on a copy, `cargo test` green; the new test fails at doctor.rs with status Warn and that detail
```

Reviewers: review-tests. Origin: new. Floor: above. Fingerprint: `tests | src/application/doctor.rs + src/domain/gateway_config.rs | Doctor::configurator + audiocpp_server + tests::GatewaySide::serving`.
Verdict: CONFIRMED. Action: fix, batch 2. a test gap: its verifier's test in repro/<n>/test.patch, red on each change in repro/<n>/mutant-*.patch.
Fixed in `test: cover the gaps audio-image-backends' review found`.

### 39. the message of several refusals has no test

```
FINDING
axis: tests
severity: low
where: src/domain/gateway_config.rs:186-192 (list), src/domain/gateway_config.rs:1486 (tests::every_value_the_configuration_cannot_hold_is_listed)
title: the message of several refusals has no test
cause: every_value_the_configuration_cannot_hold_is_listed compares the vector; the tests that read the message have one error, or read its end
effect: misleading
consequence: `.join(" ")` for `.join("; ")` passes all 828 tests, and runs the sentences of `up`'s and doctor's message together
fix: add the test below
test: the_message_of_several_errors_sets_them_apart: passes on the code, fails on the change
evidence: reproduced: change applied on a copy, `cargo test` green; the new test fails at gateway_config.rs in tests.diff
```

Reviewers: review-tests. Origin: new. Floor: above. Fingerprint: `tests | src/domain/gateway_config.rs | list + tests::every_value_the_configuration_cannot_hold_is_listed`.
Verdict: CONFIRMED. Action: fix, batch 2. a test gap: its verifier's test in repro/<n>/test.patch, red on each change in repro/<n>/mutant-*.patch.
Fixed in `test: cover the gaps audio-image-backends' review found`.

### 40. a patch that cannot be kept with its build has no test

```
FINDING
axis: tests
severity: low
where: src/infrastructure/build_store.rs:272-273 (FsBuildStore::stage), src/infrastructure/build_store.rs:592 (store_tests::a_build_keeps_the_patches_applied_to_its_sources)
title: a patch that cannot be kept with its build has no test
cause: the only test with patches has names that can be written
effect: misleading
consequence: `let _ = fs::write(&copy, patch.diff);` passes all 828 tests: the build's `build.json` then lists a patch its `patches/` directory lacks
fix: add the test below
test: a_patch_that_cannot_be_kept_leaves_nothing_staged, with a name in a directory that does not exist: passes on the code, fails on the change
evidence: reproduced: change applied on a copy, `cargo test` green; the new test fails in tests.diff
```

Reviewers: review-tests. Origin: new. Floor: above. Fingerprint: `tests | src/infrastructure/build_store.rs | FsBuildStore::stage + store_tests::a_build_keeps_the_patches_applied_to_its_sources`.
Verdict: CONFIRMED. Action: fix, batch 2. a test gap: its verifier's test in repro/<n>/test.patch, red on each change in repro/<n>/mutant-*.patch.
Fixed in `test: cover the gaps audio-image-backends' review found`.

### 41. a model's file that cannot be written has no test

```
FINDING
axis: tests
severity: low
where: src/infrastructure/gateway_process.rs:74 (LocalGateway::write_file), src/infrastructure/gateway_process.rs:987 (tests::a_file_outside_the_gateways_directory_is_refused)
title: a model's file that cannot be written has no test
cause: the failing path of `write_file` is tested only on the refusal of a path outside the directory
effect: wrong action
consequence: `let _ = replace_file(path, &file.content); Ok(())` passes all 828 tests: `up` then starts the gateway on the file an earlier `up` wrote, or on none, and the model fails or runs the old configuration
fix: add the test below
test: a_file_that_cannot_be_written_starts_nothing, a directory in the file's place: passes on the code, fails on the change
evidence: reproduced: change applied on a copy, `cargo test` green; the new test fails in tests.diff
```

Reviewers: review-tests. Origin: new. Floor: above. Fingerprint: `tests | src/infrastructure/gateway_process.rs | LocalGateway::write_file + tests::a_file_outside_the_gateways_directory_is_refused`.
Verdict: CONFIRMED. Action: fix, batch 2. a test gap: its verifier's test in repro/41/test.patch, red on its mutant-1.patch. Its sibling, the write of llama-swap.yaml in LocalGateway::start, older code, takes the same test, a_configuration_that_cannot_be_written_starts_nothing, killing M26.
Fixed in `test: cover the gaps audio-image-backends' review found`.

### 42. a limit that cuts a character reports invalid UTF-8, and the test's text is ASCII alone

```
FINDING
axis: tests, error handling
severity: low
where: src/infrastructure/host.rs:360-371 (read_capped), src/infrastructure/host.rs:771 (tests::a_text_file_is_read_up_to_its_limit)
title: a limit that cuts a character reports invalid UTF-8, and the test's text is ASCII alone
cause: `take(limit + 1).read_to_string` fails on the cut character before the size check runs; the test's over-limit input is ASCII
effect: misleading
consequence: an index over 16 MiB with a multi-byte character across the limit says `stream did not contain valid UTF-8` in place of `holds more than 16777216 bytes`, and doctor's remedy fetches it again
fix: read the bytes, check their count, then decode them
test: a_text_over_its_limit_is_told_so_when_the_limit_cuts_a_character: fails on the code, passes with that fix
evidence: reproduced: the test in tests.diff gives `cannot read .../model.safetensors.index.json: stream did not contain valid UTF-8` on `read_capped(&path, 1)` of `aé`
```

Reviewers: review-tests. Origin: new. Floor: above. Fingerprint: `tests | src/infrastructure/host.rs | read_capped + tests::a_text_file_is_read_up_to_its_limit`.
Verdict: CONFIRMED. Action: fix, batch 13. its test: repro/review-tests/tests.diff, by name, against the end commit. The input class added: a character of two, three and four bytes under each limit that cuts it, read whole at its size, and an empty file under a limit of 0. The sibling reads checked: http's head is measured before it is decoded, error bodies and commands' output are decoded lossily, an event only once whole; read_capped was the only one.
Fixed in `fix(doctor): say an index is over its limit when it cuts a character`.

### 43. shard_path's arm for `.` can never run

```
FINDING
axis: dead and legacy code
where: src/domain/safetensors.rs:47 (shard_path)
title: shard_path's arm for `.` can never run
cause: Path::components drops every `.` but a leading one of a relative path, and the path shard_path walks is absolute, the index being absolute and a shard either absolute or joined onto it
effect: maintenance
consequence: an arm and a clause of the doc comment describe a case that does not exist; the test's `./shards/..` reads as if the arm served it
fix: remove `Component::CurDir => {}` and say `..` alone in the doc comment; the test keeps its `./` input
test: none: no behaviour changes, and a_shard_is_read_from_the_indexs_directory_unless_absolute passes without the arm
evidence: reproduced: a small rustc program printed curdir=0 for "./shards/../x", "a/./b", "/./a/./b", "." and "./" joined onto /m/f/t; with the arm deleted in a copy of the scope's end, `cargo test --offline --lib domain::safetensors` gave 4 passed
```

Reviewers: review-cleanup. Origin: new. Floor: below. Fingerprint: `dead and legacy code | src/domain/safetensors.rs | shard_path`.
Verdict: none, below the floor. Action: fix at once, batch 1.
Fixed in `refactor: apply batch 1 of audio-image-backends' review fixes`.

### 44. write_model guards against an env that is never empty

```
FINDING
axis: dead and legacy code
where: src/domain/gateway_config.rs:684 (write_model), src/domain/gateway_config.rs:349 (entry)
title: write_model guards against an env that is never empty
cause: entry, the only constructor of ModelEntry, always sets `env: vec![…]` with the key's variable, so `if !model.env.is_empty()` has no false branch
effect: maintenance
consequence: the guard suggests a model can run without the key's variable, which every server of this roadmap now reads
fix: remove the guard and write `env:` and its items unconditionally; `env` can stay a Vec
test: none: no reachable behaviour changes, and the snapshots pin the output
evidence: read: `grep -rn 'ModelEntry {' src tests` finds the struct and gateway_config.rs:340 only; line 349 sets the one element
```

Reviewers: review-cleanup. Origin: older. Floor: below. Fingerprint: `dead and legacy code | src/domain/gateway_config.rs | entry + write_model`.
Verdict: none, below the floor. Action: register. older code, no behaviour changes: goes when write_model next changes.

### 45. write_file refuses a path no production code can give it

```
FINDING
axis: dead and legacy code, tests
where: src/infrastructure/gateway_process.rs:69-73 (LocalGateway::write_file), src/infrastructure/gateway_process.rs:986-1015 (tests::a_file_outside_the_gateways_directory_is_refused), src/domain/gateway_config.rs:449-451 (audiocpp_server), src/presentation/mod.rs:71 (Services::configurator), src/infrastructure/gateway_process.rs:71 (LocalGateway::write_file), src/infrastructure/gateway_process.rs:987-1015 (tests::a_file_outside_the_gateways_directory_is_refused)
title: write_file refuses a path no production code can give it
cause: the only ConfigFile outside tests is `files_dir/audiocpp/<id>.json`, files_dir being LocalGateway::files_dir, its own self.dir, and the id holding no `/`, so the `parent.starts_with(&self.dir)` check and its error are reached by its test alone
effect: maintenance
consequence: a branch and a test to keep for a case that cannot occur; the check is not a containment either, since starts_with passes a path that climbs out with `..`
fix: remove the filter and its test; take the parent with a plain error for a path without one, and keep create_private_dir and replace_file
test: none: the test that goes is the only one that reaches the branch
evidence: read: `grep -rn 'ConfigFile {' src tests` finds gateway_config.rs:460 and two tests of gateway_process.rs; `grep 'files_dir:'` shows every production Configurator takes gateway.files_dir(); is_valid_id at inventory.rs:490 keeps `/` out of an id; no document names this check
```

Reviewers: review-cleanup, review-tests. Origin: new. Floor: below. Fingerprint: `dead and legacy code | src/domain/gateway_config.rs + src/infrastructure/gateway_process.rs + src/presentation/mod.rs | LocalGateway::write_file + Services::configurator + audiocpp_server + tests::a_file_outside_the_gateways_directory_is_refused`.
Verdict: none, below the floor. Action: fix at once, batch 1. opposite fixes: review-cleanup removes the check, review-tests refuses `..`; removal kept: the only ConfigFile outside the tests is built at gateway_config.rs:460 from files_dir and an ID without `/`, so no input reaches it. It also stops a wrong files_dir in Services::configurator, whose one line no test holds. The `?` on a path without a directory has no test: no ConfigFile path lacks one.
Fixed in `refactor: apply batch 1 of audio-image-backends' review fixes`.

### 46. A model's ID is checked for characters the inventory has already refused

```
FINDING
axis: dead and legacy code
where: src/domain/gateway_config.rs:478 (audiocpp_config), src/domain/gateway_config.rs:317 (entry), src/domain/gateway_config.rs:399 (llama_server), src/domain/inventory.rs:490 (is_valid_id)
title: A model's ID is checked for characters the inventory has already refused
cause: is_valid_id admits ASCII letters, digits, `.`, `_` and `-` only, and Inventory::new is the only way to an Inventory, so neither of Text::checked's refusals can fire on model.id; the scope adds a third such call to the two it kept
effect: maintenance
consequence: each call reads as if an ID could carry a macro or a control character, and audiocpp_config's comment says every text in it needs the check, which leads the next server to copy it
fix: use `model.id.clone()` at the three sites, and limit audiocpp_config's comment to the texts the inventory does not constrain
test: none: no reachable behaviour changes
evidence: read: inventory.rs:490-493 and 444; the three calls by `grep -n 'checked(&model.id)' src/domain/gateway_config.rs`; the scope's base had the calls of lines 317 and 399 only
```

Reviewers: review-cleanup, review-tests. Origin: new. Floor: below. Fingerprint: `dead and legacy code | src/domain/gateway_config.rs + src/domain/inventory.rs | audiocpp_config + entry + is_valid_id + llama_server`.
Verdict: none, below the floor. Action: fix at once, batch 1.
Fixed in `refactor: apply batch 1 of audio-image-backends' review fixes`.

### 47. The missing-backend and missing-runtime branches run only in tests

```
FINDING
axis: dead and legacy code
where: src/domain/gateway_config.rs:304-312 (entry), src/domain/gateway_config.rs:163-164 (ConfigError::Unresolved), src/application/gateway_config.rs:90-101 (Configurator::resolve), src/application/doctor.rs:456-458 (Doctor::model_check)
title: The missing-backend and missing-runtime branches run only in tests
cause: check_model refuses a model with an undeclared backend and resolve fills or fails a runtime for every model's backend before generate, so entry's else branch, ConfigError::Unresolved and the None in model_check's `kind` are reached by no production path
effect: none
consequence: none today: an error variant and a None case no user can meet
fix: none: Inventory::backend and the runtimes map return an Option by their types, and removing the branches takes an expect the conventions forbid or an Inventory that resolves each model's backend, which costs more than these few lines; generate is also the domain's public entry, with a test on a map lacking a runtime
test: none: a_backend_without_a_runtime_is_an_error already covers the branch
evidence: read: inventory.rs:528-534 returns UnknownBackend; application/gateway_config.rs:90-101 inserts a runtime per model's backend or returns the error; `size` passes those runtimes to generate
```

Reviewers: review-cleanup. Origin: new. Floor: below. Fingerprint: `dead and legacy code | src/application/doctor.rs + src/application/gateway_config.rs + src/domain/gateway_config.rs | ConfigError::Unresolved + Configurator::resolve + Doctor::model_check + entry`.
Verdict: none, below the floor. Action: register. kept: the types give an Option, and removing the branches takes an expect the conventions forbid.

### 48. The read of build.json format 1 is legacy code without its LEGACY comment

```
FINDING
axis: dead and legacy code
where: src/infrastructure/build_store.rs:44-46 (FORMAT), src/infrastructure/build_store.rs:60-61 (Metadata), src/infrastructure/build_store.rs:101-104 (from_json), src/infrastructure/run_store.rs:43-46 (READS)
title: The read of build.json format 1 is legacy code without its LEGACY comment
cause: the roadmap moves build.json to format 2 and keeps reading format 1 through the range check and `#[serde(default)]` on patches, with a doc comment but no `// LEGACY:` stating the obligation or when it ends, as run_store's reads of old formats have
effect: maintenance
consequence: a grep for LEGACY does not find this read, so nothing says what would let it go; the obligation holds today, b11538 and b11700 on this machine being format 1
fix: add `// LEGACY:` on FORMAT's range and on `patches`' default: the builds installed before format 2 must stay readable, and format 1 goes once none is kept or a rebuild replaces them
test: none: a comment changes no behaviour, and a_build_of_format_1_has_no_patch covers the read
evidence: reproduced: `grep -m1 '"format"'` on each build.json under ~/.local/share/ai-ctl/builds gives format 1 for llama.cpp b11538 and b11700 and format 2 for the other five; `grep -rn LEGACY src tests` finds run_store.rs and domain/bench.rs only
```

Reviewers: review-cleanup. Origin: new. Floor: below. Fingerprint: `dead and legacy code | src/infrastructure/build_store.rs + src/infrastructure/run_store.rs | FORMAT + Metadata + READS + from_json`.
Verdict: none, below the floor. Action: fix at once, batch 1.
Fixed in `refactor: apply batch 1 of audio-image-backends' review fixes`.

### 49. The saved runs' patches keep the old shape without a LEGACY comment

```
FINDING
axis: dead and legacy code
where: src/infrastructure/run_store.rs:192-194 (BuildJson), src/infrastructure/run_store.rs:326-328 (policy), src/infrastructure/run_store.rs:43-46 (READS)
title: The saved runs' patches keep the old shape without a LEGACY comment
cause: `#[serde(default, skip_serializing_if = "Vec::is_empty")]` lets the runs of formats 1 and 2, which have no patches key, be read and keeps a run's JSON as before patches existed, with no format change; READS says the reads marked LEGACY go with those formats, and this one is not marked
effect: maintenance
consequence: whoever retires formats 1 and 2 is not led to this field, and nothing says that the omission is what keeps format 3's shape; the same choice was set aside for build.json because an older ai-ctl would refuse the unknown field, which a run with patches would meet here
fix: mark the field `// LEGACY:` as `policy` is: absent from the runs saved before builds had patches, which must stay readable, and left out when empty so that format 3 keeps its shape; a format 4 that always writes the key is the alternative if the omission has no obligation
test: none: a comment changes no behaviour, and a_run_is_readable_json pins the omission
evidence: reproduced: a read-only count over ~/.local/share/ai-ctl/bench gives 7 runs, 6 of format 1 and 1 of format 2, none with a patches key, so the obligation holds; read: docs/roadmap/completed/audio-image-backends/phase-1-native-builds-report.md:338-343 for the choice set aside; `grep -n LEGACY src/infrastructure/run_store.rs` has no line at the field
```

Reviewers: review-cleanup. Origin: mixed. Floor: below. Fingerprint: `dead and legacy code | src/infrastructure/run_store.rs | BuildJson + READS + policy`.
Verdict: none, below the floor. Action: fix at once, batch 1.
Fixed in `refactor: apply batch 1 of audio-image-backends' review fixes`.

### 50. gateway_config's list copies inventory's, its element type aside

```
FINDING
axis: refactoring
where: src/domain/gateway_config.rs:186-192 (list), src/domain/inventory.rs:367-373 (list)
title: gateway_config's list copies inventory's, its element type aside
cause: the scope adds InvalidConfig with a private `list` whose body is that of inventory::list: each error's to_string, joined by "; "
effect: maintenance
consequence: the two messages can drift apart: how several refusals read is written twice
fix: one `pub fn joined<E: Display>(errors: &[E]) -> String` in src/domain/text.rs, called by both `#[error("{}", …)]` attributes; each keeps its own error type
test: none: no behaviour changes; every_value_the_configuration_cannot_hold_is_listed and the inventory's tests pin both messages
evidence: read: the two bodies are the same line for line; `grep -rnE 'fn list\b|join\("; "\)' src` finds toolchain.rs:263 and doctor.rs:151 and 528 too, which join texts and not errors, and application/build.rs:446, another function
```

Reviewers: review-cleanup. Origin: new. Floor: below. Fingerprint: `refactoring | src/domain/gateway_config.rs + src/domain/inventory.rs | list`.
Verdict: none, below the floor. Action: fix at once, batch 1.
Fixed in `refactor: apply batch 1 of audio-image-backends' review fixes`.

### 51. recipe's new number repeats the digits-only rule of machine.rs and host.rs

```
FINDING
axis: refactoring
where: src/domain/recipe.rs:113-116 (number), src/domain/machine.rs:215-219 (versioned_toolkit), src/domain/machine.rs:213 (is_unversioned_toolkit), src/infrastructure/host.rs:566 (is_gxx)
title: recipe's new number repeats the digits-only rule of machine.rs and host.rs
cause: the scope makes `number` a function of its own, the same work as the `number` closure in versioned_toolkit, and is_unversioned_toolkit and is_gxx hold the same `!is_empty && all digits` test
effect: maintenance
consequence: four writings of the one rule that keeps `+1` from parsing as a number
fix: in the domain, `is_digits(text) -> bool` for is_unversioned_toolkit and is_gxx, and `parse_digits<T: FromStr>(text) -> Option<T>` on it for number, u64, and versioned_toolkit, u32; Release::parse keeps its own form, which accepts no zero and reports its own error, and Sha256::parse with TagPattern's hash stay apart, one line each with different lengths
test: none: no behaviour changes; versions_are_compared_number_by_number and the toolkit and g++ tests pin the sites
evidence: read: `grep -rnE 'is_ascii_digit' src` finds these four with the same condition; gateway.rs:48 tests digits after its parse and allows a `v` before them; timestamp.rs:36 and recipe.rs:147 scan by position, which is other work
```

Reviewers: review-cleanup. Origin: new. Floor: below. Fingerprint: `refactoring | src/domain/machine.rs + src/domain/recipe.rs + src/infrastructure/host.rs | is_gxx + is_unversioned_toolkit + number + versioned_toolkit`.
Verdict: none, below the floor. Action: register. no local fix: it reaches older code in machine.rs and host.rs, for a rule that holds at each site.

### 52. check_audiocpp copies check_flags' finite-number rule

```
FINDING
axis: refactoring
where: src/domain/inventory.rs:611-620 (check_audiocpp), src/domain/inventory.rs:679-686 (check_flags)
title: check_audiocpp copies check_flags' finite-number rule
cause: both match FlagValue::Float with a non-finite number and push NonFiniteValue, differing only by the scope text
effect: maintenance
consequence: a later change to what a number flag may hold has two places to change
fix: `fn check_finite(scope: &str, flag: &str, value: &FlagValue, errors: &mut Vec<InventoryError>)`, called by both, each passing its own scope; check_audiocpp keeps its loop, its other checks being check_flags' no
test: none: no behaviour changes; a_session_option_must_be_finite and a_number_value_must_be_finite_for_every_server pin both sites
evidence: read: the two blocks; `grep -n NonFiniteValue src/domain/inventory.rs` finds the variant and these two sites alone
```

Reviewers: review-cleanup. Origin: new. Floor: below. Fingerprint: `refactoring | src/domain/inventory.rs | check_audiocpp + check_flags`.
Verdict: none, below the floor. Action: fix at once, batch 1.
Fixed in `refactor: apply batch 1 of audio-image-backends' review fixes`.

### 53. Seven test modules each write a whole Build

```
FINDING
axis: refactoring
where: src/domain/build.rs:394-405 (tests::built), src/application/fakes.rs:321-333 (b11538), src/infrastructure/build_store.rs:384-402 (tests::build), src/infrastructure/build_store.rs:521-536 (store_tests::build), src/infrastructure/run_store.rs:755-770 (tests::build), src/presentation/build.rs:251-266 (tests::build), src/application/build.rs:965-976 (tests::installed), src/domain/bench.rs:889-890 (testing)
title: Seven test modules each write a whole Build
cause: each test module has its own Build literal, the same project, commit and toolkit; the scope added `patches: vec![]` to every one, and to the eighth in tests/gateway_lifecycle.rs
effect: maintenance
consequence: each new field of Build means eight edits, as this roadmap's one did, two of them in the same file
fix: a `#[cfg(test)] pub mod testing` in src/domain/build.rs with `build(id, at)`, as domain::bench::testing does for runs; each site overrides what it differs by, host compiler path, cmake_flags or a timestamp from text, with struct update as doctor.rs:1075 and lifecycle.rs:1140 already do on b11538; tests/gateway_lifecycle.rs:414 keeps its own, an integration test not seeing cfg(test) code
test: none: test code only, and the tests still pin the values they read
evidence: read: `grep -n 'Build {$' -r src tests` and the seven literals; they differ only in host_compiler's command or path, cmake_flags, and built_at's input
```

Reviewers: review-cleanup. Origin: mixed. Floor: below. Fingerprint: `refactoring | src/application/build.rs + src/application/fakes.rs + src/domain/bench.rs + src/domain/build.rs + src/infrastructure/build_store.rs + src/infrastructure/run_store.rs + src/presentation/build.rs | b11538 + store_tests::build + testing + tests::build + tests::built + tests::installed`.
Verdict: none, below the floor. Action: register. no local fix: seven test modules, test code alone; worth its cost when Build next gains a field.

### 54. The new kinds' test models and backends are written whole at each site

```
FINDING
axis: refactoring
where: src/domain/inventory.rs:934-949 (tests::bare), src/domain/gateway_config.rs:1097-1116 (tests::served), src/application/doctor.rs:1870-1901 (tests::with_an_audiocpp_model), src/application/doctor.rs:2012-2042 (tests::with_fibo), src/application/lifecycle.rs:1113-1156 (tests::up_writes_the_files_its_models_read), src/domain/inventory.rs:925-932 (tests::backend), src/domain/gateway_config.rs:1127 (tests::audio_and_image)
title: The new kinds' test models and backends are written whole at each site
cause: the scope adds bare and served, the same model with no optional field apart from served's fixed budget and ttl, and three more full Model and Backend literals, while tests::backend's form is repeated as a literal in gateway_config, doctor and lifecycle
effect: maintenance
consequence: each new field of Model means editing all of them, as `audiocpp` did for this roadmap
fix: a `#[cfg(test)] pub mod testing` in src/domain/inventory.rs with bare and backend, as they stand; served becomes `Model { files, flags, vram_mib: 2213, ttl: Some(600), ..bare(…) }`, and the doctor and lifecycle fixtures use them with struct update; the two audio_and_image stay apart, holding different models for different assertions, and tests::model and fakes::model stay, the shipped LLM with its files and flags
test: none: test code only
evidence: read: `grep -nE 'Model \{|Backend \{$' -r src tests` and the sites named; the two audio_and_image were compared, one holding a fish_audio model and runtimes the other lacks
```

Reviewers: review-cleanup. Origin: new. Floor: below. Fingerprint: `refactoring | src/application/doctor.rs + src/application/lifecycle.rs + src/domain/gateway_config.rs + src/domain/inventory.rs | tests::audio_and_image + tests::backend + tests::bare + tests::served + tests::up_writes_the_files_its_models_read + tests::with_an_audiocpp_model + tests::with_fibo`.
Verdict: none, below the floor. Action: register. no local fix: four test modules, test code alone; worth its cost when Model next gains a field.

### 55. Ten test sites write the same Configurator over the fakes

```
FINDING
axis: refactoring
where: src/application/build.rs:2050-2055 (tests::gateway_on), src/application/doctor.rs:970-975 (tests::GatewaySide::serving), src/application/doctor.rs:2381-2386 (tests::an_inventory_up_would_refuse_fails_a_check_without_a_gateway), src/application/lifecycle.rs:1020-1025 (tests::World::lifecycle), src/application/lifecycle.rs:1064-1069 (tests::World::config), src/application/bench.rs:1059-1064 (tests::World::configurator), src/application/gateway_config.rs:177-182 (tests::on_gpu), src/application/gateway_config.rs:252-257 (tests::the_gpu_is_read_once_the_inventory_and_its_builds_are), src/application/gateway_config.rs:356-361 (tests::the_images_to_pull_are_those_the_models_run), src/application/gateway_config.rs:373-378 (tests::a_preparation_keeps_the_inventory_builds_and_gpu_it_was_made_of)
title: Ten test sites write the same Configurator over the fakes
cause: each test builds the struct itself, so the scope's files_dir added the line `files_dir: Path::new(crate::application::fakes::GATEWAY_DIR)` ten times; lifecycle's World holds it twice
effect: maintenance
consequence: the next field of Configurator means ten edits, none of them to what a test checks
fix: `pub fn configurator<'a>(inventory: &'a dyn InventoryStore, builds: &'a dyn BuildStore, gpu: &'a dyn GpuProbe) -> Configurator<'a>` in src/application/fakes.rs, taking GATEWAY_DIR itself, called at the ten sites; Services::configurator and Doctor::configurator keep their own, on the real gateway and on differently held ports
test: none: test code only
evidence: read: `grep -rn 'files_dir:' src tests` finds these ten lines with the same value, and three others taking gateway.files_dir(), in presentation/mod.rs, doctor.rs:114 and tests/gateway_lifecycle.rs:209
```

Reviewers: review-cleanup. Origin: mixed. Floor: below. Fingerprint: `refactoring | src/application/bench.rs + src/application/build.rs + src/application/doctor.rs + src/application/gateway_config.rs + src/application/lifecycle.rs | tests::GatewaySide::serving + tests::World::config + tests::World::configurator + tests::World::lifecycle + tests::a_preparation_keeps_the_inventory_builds_and_gpu_it_was_made_of + tests::an_inventory_up_would_refuse_fails_a_check_without_a_gateway + tests::gateway_on + tests::on_gpu + tests::the_gpu_is_read_once_the_inventory_and_its_builds_are + tests::the_images_to_pull_are_those_the_models_run`.
Verdict: none, below the floor. Action: register. no local fix: ten test sites in five modules, test code alone; worth its cost when Configurator next gains a field.

### 56. stack.md still says one model runs at a time with the shipped inventory

```
FINDING
axis: conventions
severity: low
where: docs/stack.md:313-314 (top level), deploy/inventory.toml:219-337 (top level), src/domain/gateway_config.rs:228 (matrix)
title: stack.md still says one model runs at a time with the shipped inventory
cause: the scope added speech models whose budgets fit together, and matrix writes sets of several of them, while stack.md's description of the gateway, written before, was not updated
effect: misleading
consequence: a reader of the LLM stack's document expects every switch to evict the running model, where the shipped matrix lets up to four speech models run together
fix: the sentence says that each chat and image model runs alone and that the speech models share the GPU, as docs/gpu/llama-swap.md:46-47 says
test: none: a document's sentence
evidence: reproduced: tests/snapshots/gateway-shipped.yaml:345-351, the shipped configuration's sets, "fit-3": "whisper-large-v3-turbo & whisper-large-v3 & qwen3-asr-1.7b & voxtral-4b-tts" among them; read: docs/stack.md:313-314, «with the shipped inventory, one model runs at a time (§6.4)»
```

Reviewers: session. Origin: older. Floor: above. Fingerprint: `conventions | deploy/inventory.toml + docs/stack.md + src/domain/gateway_config.rs | matrix + top level`.
Verdict: CONFIRMED. Action: fix, batch 15. Verifier: the sentence unchanged since the base, line 289, while the shipped matrix's fit-3 to fit-15 hold two to four speech models each; severity low holds; its fix in repro/56/fix.patch, docs/stack.md §6.1 said as docs/gpu/llama-swap.md:46-47 says.
Fixed in `docs: bring the documents in line with the audio and image servers`.

### 57. FakeApi's running fails through models set to None, not through a field named after it

```
FINDING
axis: conventions
where: src/application/fakes.rs:907-911 (FakeApi::running), src/application/fakes.rs:856 (FakeApi), src/application/doctor.rs:2318 (tests), src/application/bench.rs:1442 (tests), src/application/lifecycle.rs:2274 (tests)
title: FakeApi's running fails through models set to None, not through a field named after it
cause: the double holds `models: Option<Vec<RunningModel>>` and fails `running` when it is None, where docs/architecture.md has a double fail through `<method>_fails`; finding 24's sibling, seen while fixing it, older than the scope
effect: maintenance
consequence: a test that leaves models at None by mistake gets a timeout it did not ask for, and the double's fields do not show that the call can be made to fail
fix: `models: Vec<RunningModel>` and `running_fails: bool`, the three tests that set None setting the new field
test: none: a test double; the three tests that make running fail set the new field
evidence: read: src/application/fakes.rs:907-911, the error on None; docs/architecture.md:122-125, the rule; `git blame` dates the field to `feat(doctor): report the gateway, and document its commands`, 2026-10-11, before the scope's base
```

Reviewers: session. Origin: older. Floor: below. Fingerprint: `conventions | src/application/bench.rs + src/application/doctor.rs + src/application/fakes.rs + src/application/lifecycle.rs | FakeApi + FakeApi::running + tests`.
Verdict: none, below the floor. Action: register. older code below the floor, seen while fixing finding 24: goes when FakeApi next changes.

## Probes

- Finding 9, from `review-correctness`: `ai-ctl up; ai-ctl bench run --repeat 1 --scenario depth whisper-large-v3`. Risk: starts the gateway, loads whisper-large-v3 on the GPU and evicts what the matrix cannot keep beside it; writes no saved run if it fails as read. Settles: the exact message the user meets, and that the model is loaded before the failure.
- Finding 13, from `review-correctness`: `ai-ctl up; start flux-3-image-klein through the gateway with the key, by GET /upstream/flux-3-image-klein/health; at ready, nvidia-smi --query-compute-apps=pid,used_memory --format=csv; kill -9 the recorded llama-swap PID; ai-ctl status; ai-ctl up; then end the orphan sd-server by its PID and run ai-ctl down`. Risk: loads an image model, 16 to 21 GiB of RAM, on the GPU; kills the gateway and leaves an orphan until it is ended by hand. Settles: whether an sd-server loaded with --offload-to-cpu shows on the GPU's list before its first image, so whether status and up see it once its gateway is killed.
- Finding 9, from `review-structure`: `with a gateway up on the shipped inventory, `ai-ctl bench run whisper-large-v3-turbo --repeat 1`, then `ai-ctl status``. Risk: starts whisper-server through the gateway, evicting a model its matrix does not let beside it; the model loads again on its next request. Settles: the error a speech model gets from bench run, and whether the model was loaded before it.

## Facts

- `review-dependencies`: cargo deny check: advisories ok, bans ok, licenses ok, sources ok, under unmaintained = "all" and unsound = "all", so no advisory, unsound or unmaintained warning stands on any crate; no record of dependencies.json is yanked
- `review-dependencies`: duplicates: syn 2.0.119 and 3.0.9, thiserror 1.0.69 and 2.0.21, thiserror-impl 1.0.69 and 2.0.21; the older of each comes only through nvml-wrapper 0.13.0, by thiserror 1 and by wrapcenum-derive 0.4.1, then darling 0.20.11, then syn 2; both are their crates' newest releases, so no update removes the duplicates
- `review-dependencies`: versions beyond the requirements, all pulled by another crate's requirement, none fixing an advisory: darling, darling_core and darling_macro 0.20.11 (newest 0.24.1), libloading 0.8.9 (0.9.0), syn 2.0.119 (3.0.9), thiserror and thiserror-impl 1.0.69 (2.0.21), and off the target platform r-efi 6.0.0 (7.1.0) and windows-link 0.2.1 (0.100.0); available is null on every record
- `review-dependencies`: wrapcenum-derive 0.4.1: last release 2024-02-10, none in the past year, 1 dependent, nvml-wrapper's own helper, owned by two of its three owners, repository moved to rust-nvml, last pushed 2024-02-10, not archived, no RustSec warning
- `review-dependencies`: old releases with no RustSec warning and no archived repository: ident_case 1.0.1 (2019-03-18, pushed 2026-08-18), static_assertions 1.1.0 (2019-11-03, pushed 2023-11-18, through nvml-wrapper), fnv 1.0.7 (2020-05-14, pushed 2024-03-20), heck 0.5.0 (2024-03-12, pushed 2025-08-09), strsim 0.11.1 (2024-04-02, pushed 2025-11-27), utf8parse 0.2.2 (2024-06-08, pushed 2026-10-25); no record of dependencies.json has an archived repository
- `review-dependencies`: nvml-wrapper 0.13.0: the direct dependency with the fewest dependents, 143, and downloads, 7.5 million; last release 2026-08-31, 3 in the past year, 3 owners, 317 unsafe, nvml-wrapper-sys 794, the expected cost of binding a C library, used from src/infrastructure/nvml.rs alone
- `review-dependencies`: unsafe counts that stand out: linux-raw-sys 6930 and rustix 1610, on the lock only through tempfile, a dev-dependency; windows-sys 12530, off the target platform; libc 800
- `review-dependencies`: single owner on direct dependencies: anyhow and thiserror, dtolnay; tempfile, Stebalien; each very widely used and released within the year
- `review-dependencies`: llama-swap: PINNED and the installed current are v263, newest tag v269, six releases between: v264, v265, v266, v267, v268, v269; their notes unread, see LEFT OUT
- `review-dependencies`: llama.cpp: current build b12111, newest tag b12111, no lag
- `review-dependencies`: whisper.cpp: current build v1.9.5, newest tag v1.9.5, no lag
- `review-dependencies`: audio.cpp: current build v0.9.3, newest tag v0.9.3, no lag
- `review-dependencies`: stable-diffusion.cpp: current build master-1024-77b72a2, newest tag master-1047-984eb06, fourteen releases between: master-1025-8ee08e3, master-1026-997478a, master-1027-e7b7ec6, master-1028-37bc88b, master-1030-0a06712, master-1031-c6dfa9d, master-1037-5d33b1b, master-1038-133087d, master-1039-290148d, master-1043-3be2378, master-1044-f8ea2e1, master-1045-6225bf8, master-1046-d308544, master-1047-984eb06; their notes unread, see LEFT OUT

## Coverage

- `review-security`: in full: docs/security.md, src/domain/safetensors.rs, src/domain/patches/audio.cpp/api-key.patch, src/domain/patches/stable-diffusion.cpp/api-key.patch, src/domain/patches/stable-diffusion.cpp/listen-failure.patch, src/domain/patches/stable-diffusion.cpp/reuse-address.patch, src/domain/patches/whisper.cpp/api-key.patch, src/domain/patches/whisper.cpp/reuse-address.patch, tests/fixtures/audiocpp_server-v0.9.2-help.txt, tests/fixtures/whisper-server-v1.9.5-help.txt; in part: src/domain/gateway_config.rs (lines 1-780), src/domain/inventory.rs (lines 1-725), src/domain/recipe.rs (lines 1-315), src/domain/build.rs (lines 1-135), src/domain/text.rs (lines 1-40), src/infrastructure/git.rs (lines 1-340), src/infrastructure/gateway_process.rs (lines 1-480), src/infrastructure/build_store.rs (lines 1-385), src/infrastructure/files.rs (lines 1-230), src/infrastructure/inventory_file.rs (lines 120-314), src/infrastructure/host.rs (lines 355-372), src/application/build.rs (lines 1-345), src/application/doctor.rs (lines 100-600, 840-860), src/application/gateway_config.rs (lines 1-170), src/application/lifecycle.rs (lines 225-350), deploy/inventory.toml (lines 1-20, 205-420), docs/openwebui.md (lines 15-36), docs/audio/audio-cpp.md (lines 86-206), docs/audio/whisper-cpp.md (lines 44-96), docs/image/stable-diffusion-cpp.md (lines 79-100), tests/fixtures/sd-server-master-975-help.txt (lines 1-120), docs/roadmap/completed/v2-rebuild/phase-2-native-build-report.md (lines 100-118), ~/.local/share/ai-ctl/sources/llama.cpp/scripts/ui-assets.cmake (lines 1-592), ~/.local/share/ai-ctl/sources/audio.cpp/app/server/main.cpp (lines 170-350), ~/.local/share/ai-ctl/sources/audio.cpp/app/server/request_dispatch.h (lines 60-219), ~/.local/share/ai-ctl/sources/audio.cpp/app/server/runtime.cpp (lines 112-230, 313-332, 405-445, 2276-2300), ~/.local/share/ai-ctl/sources/audio.cpp/app/server/server_protocol.cpp (lines 180-195, 410-430, 1278-1310), ~/.local/share/ai-ctl/sources/audio.cpp/app/server/server_ui.cpp (lines 60-90), ~/.local/share/ai-ctl/sources/audio.cpp/tools/cmake/espeak_static.cmake (lines 1-40); left out: src/application/fakes.rs, src/application/bench.rs, src/domain/bench.rs: test doubles and test setup, no sink of this axis; src/infrastructure/run_store.rs, src/presentation/build.rs, src/presentation/mod.rs: seen by search only; they copy a build's metadata and print the recipe's static patch names, no sink of this axis; the `tests` modules of the files above, tests/cli.rs, tests/gateway_config.rs, tests/gateway_lifecycle.rs: not analysed one by one; this axis followed the production sinks, and a control's test was read only through the search for its name; docs/commands.md, docs/stack.md, docs/gpu/llama-swap.md, README.md: searched for the security claims above, not read whole; a whole llama-server compile against the crafted archive, and the live bucket: the compile is `ai-ctl build`'s and no fake bucket can stand in for the script's fixed URL; the script and the compile of its ui.cpp were run instead; the audio.cpp and stable-diffusion.cpp servers were read and not run: their key checks are recorded as probed in their documents, and no finding here depends on a run of them
- `review-correctness`: in full: scope.txt, src/domain/patches/audio.cpp/api-key.patch, src/domain/patches/stable-diffusion.cpp/api-key.patch, src/domain/patches/stable-diffusion.cpp/listen-failure.patch, src/domain/patches/stable-diffusion.cpp/reuse-address.patch, src/domain/patches/whisper.cpp/api-key.patch, src/domain/patches/whisper.cpp/reuse-address.patch, docs/roadmap/completed/audio-image-backends/summary.md, tests/fixtures/audiocpp_server-v0.9.2-help.txt, tests/fixtures/audiocpp_server-v0.9.2-list-devices.txt, tests/fixtures/whisper-server-v1.9.5-help.txt; in part: src/application/build.rs (lines 1-640), src/application/doctor.rs (lines 100-599, 1903-1906, 2012-2103), src/application/gateway_config.rs (lines 1-170), src/application/lifecycle.rs (lines 100-960), src/application/bench.rs (lines 130-450), src/domain/inventory.rs (lines 1-725), src/domain/gateway_config.rs (lines 1-770), src/domain/recipe.rs (lines 1-300), src/domain/safetensors.rs (lines 1-60), src/domain/build.rs (lines 1-240), src/infrastructure/git.rs (lines 1-340), src/infrastructure/build_store.rs (lines 1-380), src/infrastructure/gateway_process.rs (lines 1-300), src/infrastructure/files.rs (lines 1-240), src/infrastructure/inventory_file.rs (lines 1-325), src/infrastructure/host.rs (lines 20-39, 190-219, 350-379), src/infrastructure/run_store.rs (lines 175-200, 470-560), src/infrastructure/server_api.rs (lines 1-110), src/infrastructure/cmake.rs (lines 60-175), src/infrastructure/command.rs (lines 199-240), src/infrastructure/paths.rs (lines 25-50), src/presentation/build.rs (lines 85-110, 135-152), src/application/fakes.rs (lines 65-82, 469-481), deploy/inventory.toml (lines 1-30, 60-72, 100-430), tests/fixtures/sd-server-master-975-help.txt (lines 1-120), docs/security.md (lines 1-80), docs/commands.md (lines 102-118, 335-345, 431-470), docs/image/stable-diffusion-cpp.md (lines 353-395), docs/roadmap/completed/audio-image-backends/phase-2-inventory-and-gateway-report.md (lines 354-362), the base's src/application/doctor.rs (lines 318-385), and in the local clones of sd.cpp src/model_io/safetensors_io.cpp (lines 28-34, 395-450), src/model_loader.cpp (lines 212-224), examples/common/common.cpp (lines 425-440), and of audio.cpp app/server/main.cpp (lines 25-62); left out: test code of the hunks: read only where a test shows what an item is meant to do, about 3961 lines of the scope's reading not read one by one; the tests axis is another reviewer's; src/application/ports.rs, src/application/fakes.rs, tests/gateway_lifecycle.rs, tests/gateway_config.rs, tests/cli.rs: signatures and doubles, followed only through their callers; tests/snapshots/ and the documentation paths other than the lines listed: the CLI's contract is another axis, and I did not compare them line by line; the shipped patches' C++ against upstream: read, not rebuilt; phase 3's live runs are recorded for them, and running a server with the key or the GPU is outside these rules; live runs of `up`, `down`, `build` and `bench`: forbidden to this agent; the interruptions were followed by reading, but for the submodule state made in scratch
- `review-structure`: in full: CLAUDE.md, README.md, deploy/inventory.toml, docs/README.md, docs/architecture.md, docs/commands.md, docs/audio/audio-cpp.md, docs/audio/whisper-cpp.md, docs/gpu/llama-swap.md, docs/image/stable-diffusion-cpp.md, docs/security.md, docs/roadmap/completed/audio-image-backends/summary.md, tests/fixtures/README.md, src/domain/safetensors.rs, src/presentation/cli.rs, src/presentation/mod.rs; in part: docs/openwebui.md (lines 1-80), docs/stack.md (the scope's hunks, lines 628-640), docs/roadmap/completed/audio-image-backends/README.md (lines 40-100), docs/toolchain.md (its last headings), src/application/bench.rs (lines 120-372), src/application/build.rs (lines 1-345, 368-383, 417-425, 654-712), src/application/doctor.rs (lines 90-690, 2105-2125), src/application/fakes.rs (lines 60-404, 595-625, 705-735), src/application/gateway_config.rs (lines 1-165), src/application/lifecycle.rs (lines 235-339), src/application/mod.rs (lines 1-17), src/application/ports.rs (lines 1-420), src/domain/gateway_config.rs (lines 1-790), src/domain/inventory.rs (lines 1-730), src/domain/recipe.rs (lines 1-305), src/infrastructure/build_store.rs (lines 1-300), src/infrastructure/files.rs (lines 53-71), src/infrastructure/gateway_process.rs (lines 1-300, 960-985), src/infrastructure/git.rs (lines 120-319), src/infrastructure/host.rs (lines 195-215, 355-375), src/infrastructure/inventory_file.rs (lines 1-365, 395-415), src/infrastructure/run_store.rs (lines 1-620), src/infrastructure/server_api.rs (its function list), src/presentation/build.rs (lines 1-250), tests/snapshots/gateway-shipped.yaml (run through llama-swap -validate, not read); left out: src/domain/patches/: their content is the security axis's; their names were checked against the recipe and the documents; tests/, and the tests modules of the hunks: read only where a finding or a document's statement cites one, the tests axis being another agent's; docs/openwebui.md lines 81-246 and the rest of docs/stack.md: settings of clients and the LLM stack, checked only for their links, which all resolve; docs/roadmap/completed/audio-image-backends/ phase files and reports: searched for the toolchain's changes and the mutants and 30-run rules, not read; ai-ctl up, gateway config and bench run: forbidden to this agent; their behaviour was read, with one PROBE above
- `review-performance`: in full: src/main.rs, src/infrastructure/command.rs, deploy/inventory.toml, scope.txt; in part: src/application/doctor.rs (lines 1-600), src/application/build.rs (lines 1-345, 385-415, 498-528), src/application/gateway_config.rs (lines 1-165), src/application/lifecycle.rs (lines 20-480, 538-600, 678-810), src/domain/gateway_config.rs (lines 1-700), src/domain/inventory.rs (lines 415-714), src/domain/recipe.rs (lines 1-295), src/domain/safetensors.rs (lines 1-60), src/domain/build.rs (lines 205-234), src/infrastructure/git.rs (lines 1-320), src/infrastructure/host.rs (lines 1-385), src/infrastructure/build_store.rs (lines 1-385), src/infrastructure/gateway_process.rs (lines 50-249), src/infrastructure/inventory_file.rs (lines 60-220), src/infrastructure/cmake.rs (lines 1-200), src/infrastructure/files.rs (lines 129-164), tests/snapshots/gateway-shipped.yaml (lines 320-382), docs/security.md (lines 1-80), docs/gpu/llama-swap.md (lines 128-137), tests/fixtures/whisper-server-v1.9.5-help.txt (matching lines), tests/fixtures/sd-server-master-975-help.txt (matching lines), tests/fixtures/audiocpp_server-v0.9.2-help.txt (matching lines); left out: src/application/bench.rs, src/domain/bench.rs: their hunks are in test code only, so the GPU sampling's weight was not measured; items under tests:: and store_tests::, tests/*.rs, src/application/fakes.rs: not read for this axis; they cost the user no wait, and the test suite's 795 unit tests took 1.74 s in checks/test.log; ai-ctl up and ai-ctl build: forbidden here, so their waits come from the code and from the timings above; a running gateway: none ran, so doctor and status were timed only with the gateway stopped; src/presentation/build.rs, src/presentation/mod.rs, src/infrastructure/run_store.rs, src/domain/patches/: hunks not opened; from the scope's listing they add fields, messages and patch text, with no process, wait or file read; models' load and generation times: not measured, as that needs the GPU; the inventory's figures were taken as documented
- `review-tests`: in full: src/domain/recipe.rs, src/domain/safetensors.rs, src/domain/gateway_config.rs, src/domain/inventory.rs, src/application/gateway_config.rs, src/infrastructure/git.rs, tests/gateway_config.rs, tests/fixtures/audiocpp_server-v0.9.2-help.txt, tests/fixtures/whisper-server-v1.9.5-help.txt; in part: src/application/doctor.rs (lines 90-599, 840-858, 860-1329, 1780-2199, 2376-2410), src/application/build.rs (lines 1-345, 640-1544, 1756-1772, 1808-1888, 2046-2066), src/application/fakes.rs (lines 1-480, 578-740), src/application/lifecycle.rs (lines 240-262, 310-325, 1105-1160, 2335-2405), src/application/bench.rs (lines 230-379, 1015-1085, 1925-1945), src/infrastructure/build_store.rs (lines 25-324, 375-674), src/infrastructure/gateway_process.rs (lines 40-249, 860-1079), src/infrastructure/host.rs (lines 29-32, 195-215, 350-380, 755-800, 1140-1160), src/infrastructure/inventory_file.rs (lines 120-215, 285-362, 480-540), src/infrastructure/run_store.rs (lines 175-200, 465-560, 750-830), src/infrastructure/files.rs (lines 53-59, 129-154), src/presentation/build.rs (lines 85-155, 245-300, 420-450), src/presentation/mod.rs (lines 30-80, 187-199), src/domain/build.rs (lines 40-68, 247-260), tests/cli.rs (lines 170-200), tests/gateway_lifecycle.rs (lines 170-215, 410-450), tests/fixtures/README.md (lines 1-80), tests/fixtures/sd-server-master-975-help.txt (lines 1-40), tests/fixtures/sd-server-master-975-list-devices.txt (whole), tests/fixtures/sd-server-master-975-version.txt (whole), tests/fixtures/audiocpp_server-v0.9.2-version.txt (lines 1-5), tests/fixtures/llama-server-list-devices.txt (whole), src/domain/patches/audio.cpp/api-key.patch (lines 1-30), docs/security.md (lines 1-60), docs/commands.md (searched lines only), docs/roadmap/completed/audio-image-backends/phase-3-end-to-end-report.md (searched lines only), docs/roadmap/completed/audio-image-backends/summary.md (searched lines only), docs/image/stable-diffusion-cpp.md (searched lines only); left out: src/domain/patches/: the patches' text; `shipped_patches_are_diffs_under_plain_names` reads its first and last characters, so a no-op diff under a patch's name passes every test. No test can reach it without the upstream sources: the proof is the 401s per server in phase 3's report, which I read as recorded and did not run again, a build and a GPU being needed; src/presentation/mod.rs:71 (Services::configurator): `files_dir: Path::new("/elsewhere")` passes every test, and `up` then stops loudly with `is not under`. A test needs either a GPU, through `ai-ctl gateway config`, or about twenty fakes this repository lacks, so I raised none; src/infrastructure/git.rs:262-266 (update_within): the failure of `git submodule sync` survives being ignored, but I found no input that makes it fail: a release without `.gitmodules` fails the update after it instead; tests/gateway_lifecycle.rs: it ran, but its inventory has llama-server alone, so no audio or image kind goes through a real llama-swap in `cargo test`; phase 3's report records that run by hand; deploy/inventory.toml: the snapshots hold what reaches llama-swap, not each model's `repo`, which doctor's remedy alone prints; CLAUDE.md rules out a test that reads a configuration's text back, so this is no finding; reads_as_index on `.JSON`: comparing without case passes every test, but sd-server's source is not at hand to say which is right, so I raised nothing; new tests: they are in review-tests/tests.diff, against the scope's end; `cargo fmt` and clippy were not run on them, and none binds a port, takes a lock or runs two processes at once, so each ran as often as its checks above and not 30 times
- `review-cleanup`: in full: none; in part: src/domain/recipe.rs (lines 1-300), src/domain/safetensors.rs (lines 1-60, 117-135), src/domain/text.rs (lines 1-50), src/domain/inventory.rs (lines 1-745, 925-1001), src/domain/gateway_config.rs (lines 1-760, 784-805, 1097-1234), src/domain/machine.rs (lines 205-232), src/domain/gateway.rs (lines 42-56, 84-100), src/domain/build.rs (lines 395-405), src/domain/bench.rs (lines 888-930), src/application/build.rs (lines 1-345, 749-862, 965-976, 1774-1779, 2046-2066), src/application/doctor.rs (lines 1-600, 963-982, 1072-1082, 1870-1905, 2012-2063, 2378-2388), src/application/gateway_config.rs (lines 1-165, 172-184), src/application/lifecycle.rs (lines 225-340, 1018-1036, 1062-1074, 1115-1128, 1138-1146), src/application/ports.rs (lines 1-420), src/application/fakes.rs (lines 272-420, 447-466), src/application/bench.rs (lines 300-360, 1056-1066), src/infrastructure/build_store.rs (lines 30-135, 240-300, 384-402, 521-536), src/infrastructure/run_store.rs (lines 25-60, 170-205, 322-335, 465-560, 755-770), src/infrastructure/gateway_process.rs (lines 55-80, 185-245), src/infrastructure/git.rs (lines 120-310), src/infrastructure/host.rs (lines 25-35, 198-212, 350-375, 560-572), src/infrastructure/inventory_file.rs (lines 120-220, 283-312), src/presentation/mod.rs (lines 60-76), src/presentation/build.rs (lines 251-266), tests/gateway_lifecycle.rs (lines 205-214, 412-430), docs/security.md (lines 1-60), docs/roadmap/completed/audio-image-backends/summary.md (What We Are Leaving Open), docs/roadmap/completed/audio-image-backends/phase-1-native-builds-report.md (lines 335-345), and the scope's diffs of tests/cli.rs, tests/gateway_config.rs, tests/gateway_lifecycle.rs, src/infrastructure/run_store.rs, src/domain/build.rs, src/domain/bench.rs, src/application/bench.rs, src/presentation/build.rs and src/presentation/mod.rs; left out: src/domain/patches/: unified diffs against four different source trees, read by name only; a shared patch cannot apply across them; tests in scope other than the fixtures named: searched by helper and read where a helper looked copied; test-only dead code is left to clippy with -D warnings, since those modules are private; src/domain/gateway_config.rs (KEEP_ALIVE_REASON): the workaround ends when llama.cpp #29680 is fixed, which only GitHub can tell, and no request goes there; the `// LEGACY:` comments at run_store.rs:43, 48, 185, 326, 330, 489 and domain/bench.rs:705: no finding, the 7 saved runs on this machine being of formats 1 and 2
- `review-dependencies`: in full: checks/deny.log, docs/reviews/delivered-code/2026-11-04-audio-image-backends/dependencies.json, Cargo.toml, deny.toml; in part: Cargo.lock (lines 490-540, the rest by search), docs/security.md (lines 1-80), src/domain/gateway.rs (lines 1-40), src/domain/recipe.rs (lines 58-110, 215-260, the rest by search), a working note on sd.cpp's web UI (lines 45-65), .claude/skills/reviewing-delivered-code/scripts/findings.py (lines 284-312, 404-410, the rest by search), scope.txt (lines 1-12, 668-669); left out: release notes of llama-swap v264 to v269 and of stable-diffusion.cpp master-1025 to master-1047: GitHub is out of reach and git ls-remote gives tags only, and the local clone stops at master-1024-77b72a2, so whether one of them fixes a defect ai-ctl meets is unjudged, and the lag raises no finding for want of a release note's line; to settle it, the session or the user reads those notes; whether ai-ctl's three patches to stable-diffusion.cpp still apply at master-1047-984eb06: not tried, its sources cannot be fetched under these rules, and a move of the build is a decision the lag alone does not call for
- `review-verifier`: in full: /home/seb/.local/share/ai-ctl/sources/llama.cpp/scripts/ui-assets.cmake, /home/seb/.local/share/ai-ctl/sources/llama.cpp/tools/ui/CMakeLists.txt, /home/seb/.local/share/ai-ctl/sources/llama.cpp/tools/ui/ui.h.in, /home/seb/.local/share/ai-ctl/sources/llama.cpp/tools/ui/ui.cpp.in, /home/seb/.local/share/ai-ctl/builds/llama.cpp/b12111/build.json; in part: src/domain/recipe.rs (lines 1-330), src/domain/build.rs (lines 110-135, 350-395), src/application/build.rs (lines 120-251), src/application/doctor.rs (lines 340-402, 1020-1125), src/application/gateway_config.rs (lines 1-130, 160-260, 375-420), src/application/fakes.rs (lines 315-400, 469-485), src/application/lifecycle.rs (lines 1115-1155), src/infrastructure/cmake.rs (lines 50-130), src/domain/gateway_config.rs (lines 395-460), docs/security.md (lines 1-60, 75-150, 186-345), docs/openwebui.md (lines 1-40), ~/.local/share/ai-ctl/builds/llama.cpp/b12111/build.log (lines 400-425), ~/.local/share/ai-ctl/sources/llama.cpp/CMakeLists.txt (lines 130-142), ~/.local/share/ai-ctl/sources/llama.cpp/tools/server/server-http.cpp (lines 225-262, 375-470), ~/.local/share/ai-ctl/sources/audio.cpp/app/server/server_protocol.cpp (lines 175-195, 1270-1305), ~/.local/share/ai-ctl/sources/whisper.cpp/examples/server/server.cpp (lines 1190-1240); left out: the real Hugging Face bucket and a whole llama-server compile: the bucket cannot be written from here and the compile is ai-ctl build's; the probe runs upstream's provisioning script and templates unchanged but for the base URL; audio.cpp and whisper-server path reads (finding 2): not run against a built server, the finding being a document's and low, confirmed by reading
- `review-verifier`: in full: verifier-correctness/findings.md; in part: src/domain/safetensors.rs (lines 1-80), scope.txt (its head and the hunks of the files the findings name), src/application/doctor.rs (lines 85-110, 300-600, 880-1300, 1780-1800, 1870-2200), src/domain/inventory.rs (lines 30-130, 380-420, 1220-1242), src/application/gateway_config.rs (lines 60-140), src/application/bench.rs (lines 150-200, 300-440, 866-880, 1000-1110, 1860-1960), src/infrastructure/run_store.rs (lines 528-540), src/infrastructure/server_api.rs (lines 55-75), src/presentation/cli.rs (lines 130-140), docs/commands.md (lines 128-140, 425-445), README.md (lines 60-72, 238-246, 345-346), src/application/build.rs (lines 260-300, 640-720, 982-990, 1440-1500), src/infrastructure/git.rs (lines 20-110, 160-340, 341-412, 575-705), src/infrastructure/inventory_file.rs (lines 30-80), src/infrastructure/host.rs (lines 1-30, 190-215), src/infrastructure/files.rs (lines 24-31), src/infrastructure/command.rs (lines 38-50), src/application/ports.rs (lines 60-80, 150-155), src/application/fakes.rs (lines 60-160, 322-360, 440-485, 1007-1015), src/application/lifecycle.rs (lines 860-905), src/domain/gateway_config.rs (lines 420-440), src/domain/recipe.rs (lines 45-60), docs/image/stable-diffusion-cpp.md (lines 280-300, 345-395), deploy/inventory.toml (backends and model ids, lines 245-255), tests/fixtures/sd-server-master-975-help.txt (path flags), sd.cpp 95b3090 src/model_io/safetensors_io.cpp (lines 28-34) and src/tokenizers/tokenizer_config.cpp (lines 30-66), huggingface_hub 1.24.0 _local_folder.py (lines 14-94, 431-440) and file_download.py (lines 1896-1950); left out: finding 9's eviction: llama-swap loading a model on /upstream/<model>/props, taken from the finding's reading, not run: the fix refuses before any request, so it does not depend on it; finding 13's no-GPU server case: no shipped model has that shape; not probed
- `review-verifier`: in full: verifier-structure/findings.md; in part: scope.txt (lines 1-50), README.md (lines 1-215, 335-360), docs/commands.md (lines 1-10, 75-90, 100-115, 300-330), docs/security.md (lines 335-365 and its headings), docs/README.md (lines 38-46), docs/audio/whisper-cpp.md (lines 1-8), docs/audio/audio-cpp.md (lines 1-8), docs/image/stable-diffusion-cpp.md (lines 1-8, 228-250, 300-320), deploy/inventory.toml (lines 395-417 and a grep for file types), src/presentation/cli.rs (lines 15-35), src/application/gateway_config.rs (lines 80-110), src/application/build.rs (lines 365-380), src/application/doctor.rs (lines 340-395), src/domain/gateway_config.rs (lines 455-512), src/infrastructure/gateway_process.rs (lines 55-90), src/domain/recipe.rs (lines 362-378 and its include_str! lines), tests/fixtures/README.md (lines 40-62), tests/snapshots/gateway-shipped.yaml (grep for AI_CTL_API_KEY); left out: repro/<n>/test.patch and fix.patch: every finding's test is none:, documents and a help text; repro/14 and repro/15 hold the runs that showed them
- `review-verifier`: in full: none; in part: src/domain/gateway_config.rs (lines 140-300, 600-760, 760-900, 1636-1715), src/application/lifecycle.rs (lines 32, 530-605), src/application/doctor.rs (lines 190-225), src/application/gateway_config.rs (lines 60-100), src/domain/inventory.rs (lines 206-218), docs/gpu/llama-swap.md (lines 24-40, 120-145), docs/stack.md (lines 370-386), deploy/inventory.toml (id and vram_mib lines), tests/gateway_config.rs (SHIPPED uses), src/infrastructure/inventory_file.rs (SHIPPED uses); left out: ai-ctl up: not run, the rules forbid it; how up fails once the wait runs out was read in lifecycle.rs:548-605; llama-swap's per-request solver cost on a large matrix: not measured; only the start was timed; tests/gateway_lifecycle.rs: it ran 0 tests in my copy, because llama-swap is not on PATH
- `review-verifier`: in full: verifier-tests/findings.md, verifier-tests/review-tests-tests.diff; in part: scope.txt (lines 1-30), src/application/doctor.rs (lines 1-30, 100-125, 370-400, 453-600, 847-862), src/domain/safetensors.rs (lines 1-60), src/application/build.rs (lines 150-200, 290-340), src/application/fakes.rs (lines 270-325, 388-396), src/domain/recipe.rs (lines 137-300, 300-400), src/domain/gateway_config.rs (lines 170-195, 430-500, 594-606, 615-670), src/infrastructure/git.rs (lines 225-300), src/infrastructure/build_store.rs (lines 246-300, 549-556), src/infrastructure/gateway_process.rs (lines 55-95, 270-290), src/infrastructure/host.rs (lines 195-215, 350-375, 765-797), tests/fixtures/README.md (lines 40-80), docs/security.md (lines 1-72, 234-327), ~/.local/share/ai-ctl/sources/llama.cpp common/arg.cpp (line 1140, by git grep), .claude/skills/reviewing-delivered-code/SKILL.md (lines 170-200), .claude/skills/reviewing-delivered-code/references/fixing.md (lines 25-60); left out: review-tests-tests.diff's a_model_of_another_kind_than_llama_server_is_refused, a_file_that_climbs_out_of_the_gateways_directory_is_refused and named_submodules_are_checked_out_at_any_depth_in_any_order: they serve findings outside 29-34 and 36-42; the first two fail on the code, and I did not verify them; llama-swap: no finding of this batch depends on its behaviour
- `review-verifier`: in full: none; in part: docs/stack.md at the scope's end (lines 295-330, 339-382, and a search for "at a time", "runs alone" and "evict"), docs/stack.md at the scope's base (a search for the sentence, line 289), docs/gpu/llama-swap.md at the scope's end (lines 35-60), tests/snapshots/gateway-shipped.yaml at the scope's end (lines 335-360), git diff from the scope's base to its end -- docs/stack.md (first 150 lines), scope.txt (header and the lines for the paths this finding names); left out: deploy/inventory.toml:219-337: not read; the generated snapshot of the shipped configuration already shows its sets; src/domain/gateway_config.rs:228 (matrix): not read; its output, the snapshot, is enough for a document finding
- `session`: in full: none; in part: src/infrastructure/gateway_process.rs (lines 55-85); left out: none
- `session`: in full: none; in part: docs/stack.md (lines 305-395), docs/gpu/llama-swap.md (lines 26-63, 129-135), tests/snapshots/gateway-shipped.yaml (lines 340-380), src/domain/gateway_config.rs (lines 180-290); left out: none
- `session`: in full: none; in part: src/application/fakes.rs (lines 848-911), docs/architecture.md (lines 110-125); left out: none

## Fix Plan

- Batch 1: findings 22, 24, 25, 28, 35, 43, 45, 46, 48, 49, 50, 52
- Batch 2: findings 29, 31, 33, 34, 36, 37, 38, 39, 40, 41
- Batch 3: findings 1
- Batch 4: findings 3
- Batch 5: findings 5
- Batch 6: findings 6
- Batch 7: findings 30
- Batch 8: findings 7, 8
- Batch 9: findings 9
- Batch 10: findings 10
- Batch 11: findings 11
- Batch 12: findings 12
- Batch 13: findings 42
- Batch 14: findings 26
- Batch 15: findings 2, 14, 15, 16, 17, 18, 19, 20, 21, 56
- Batch 16: findings 4
