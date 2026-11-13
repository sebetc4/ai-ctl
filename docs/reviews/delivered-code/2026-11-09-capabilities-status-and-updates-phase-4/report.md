# Review: 2026-11-09-capabilities-status-and-updates-phase-4

**Status:** complete — every finding fixed: batches 1 to 5, the last in `refactor(build): read what is outdated once for both commands` of 2026-11-09; finding 7, first registered, fixed by the roadmap's closing pass in `fix(build): drop the tags a project withdrew at each fetch` of 2026-11-09; the recorded mutants at `refactor(build): read what is outdated once for both commands` of 2026-11-09, 67 caught, none missed or gone
**Scope:** roadmap `capabilities-status-and-updates`, phase 4, `docs(review): complete the review of capabilities phase 3` of 2026-11-09..`docs: describe the builds kept at their newest release` of 2026-11-09: 59 hunks, +1917 −57 lines of code
**Go-ahead:** the user, tier `low`

## Summary

| # | Title | Axis | Effect | Severity | Origin | Verdict | Action | Batch |
|---|---|---|---|---|---|---|---|---|
| 1 | build update sends a build past the newest release back to that older release when it is off its recipe | bugs | wrong action | low | new | CONFIRMED | fix | 1 |
| 2 | build outdated reads the tags under the configuration of the repository the user stands in | bugs | misleading | low | new | CONFIRMED | fix | 2 |
| 3 | build update says no build is removed, where a build off its recipe is replaced in place | the CLI's contract | misleading | low | new | CONFIRMED | fix | 3 |
| 4 | a refusal reached through build update names --force, which build update does not take | bugs, the CLI's contract | misleading | low | older | CONFIRMED | fix | 4 |
| 5 | the verdict of a llama-swap ahead of the pin is left out of build outdated's document | the CLI's contract | none | — | new | — | fix at once | 5 |
| 6 | two arms of run repeat the same six-argument call of build::outdated | refactoring | maintenance | — | new | — | fix at once | 5 |
| 7 | build update builds a release the project withdrew, where build outdated announced the newest the remote lists | bugs | wrong action | low | older | CONFIRMED | fix | 6 |

## Checks

- `cargo fmt --check`: exit 0
- `cargo clippy --all-targets -- -D warnings`: exit 0
- `cargo test`: exit 0: 925 passed, 0 failed, 0 ignored; no test skipped itself, the gateway's test end to end run with llama-swap v271
- `python3 .claude/skills/reviewing-delivered-code/evals/test_scope.py`: exit 0: 34 tests
- `python3 .claude/skills/reviewing-delivered-code/evals/test_fixtures.py`: exit 0: 14 tests
- `python3 .claude/skills/reviewing-delivered-code/evals/test_findings.py`: exit 0: 34 tests
- `python3 .claude/skills/reviewing-delivered-code/evals/test_regression.py`: exit 0: 15 tests
- `python3 .claude/skills/reviewing-delivered-code/evals/test_dependencies.py`: exit 0: 25 tests
- `python3 .claude/skills/reviewing-delivered-code/evals/test_guard.py`: exit 0: 37 tests
- `python3 .github/scripts/test_check_release.py`: exit 0: 15 tests
- `sid.py check .`: exit 0: cites 0, names 0, other 0
- `cargo mutants --in-diff` on the scope's diff: 56 mutants in 180 s, 0 missed, 45 caught, 11 unviable

## Agents

- `review-verifier`, effort `low`: answered
- `session`, effort `low`: answered

## Findings

### 1. build update sends a build past the newest release back to that older release when it is off its recipe

```
FINDING
axis: bugs
severity: low
where: src/domain/build.rs:187 (standing), src/application/build.rs:287 (Builder::update_project), docs/commands.md:231 (top level)
title: build update sends a build past the newest release back to that older release when it is off its recipe
cause: standing compares with `release < newest` alone, so a release past the newest reaches `unlike` and becomes OffRecipe, where its comment says a difference from the recipe matters at the newest release alone
effect: wrong action
consequence: the project withdraws the tag of the current build and ai-ctl's recipe has since gained a patch or an option: build outdated says `off its recipe` and exits 4, and build update builds the older newest release with force and makes it current, which docs/commands.md:231 rules out for a build past the newest
fix: compare by Ordering: Less is Behind, Greater is Current, and `unlike` counts at Equal alone
test: a_build_past_the_newest_release_is_never_sent_back with `unlike` set expects Standing::Current, and gets OffRecipe on the code
evidence: read: src/domain/build.rs:187-190, the comparison; src/domain/build.rs:552-562, its test covers at and behind, never past; src/application/build.rs:287, force from OffRecipe
```

Reviewers: session. Origin: new. Floor: above. Fingerprint: `bugs | docs/commands.md + src/application/build.rs + src/domain/build.rs | Builder::update_project + standing + top level`.
Verdict: CONFIRMED. Action: fix, batch 1. The verifier narrowed the consequence and replaced the fix: it holds on a clone made since the withdrawal, a clone that kept the tag rebuilding its own release. Ordering would say current of a build up refuses, so OffRecipe stays past the newest and update_project rebuilds the current build's own release.
Fixed in `fix(build): rebuild a build off its recipe at its own release`.

### 2. build outdated reads the tags under the configuration of the repository the user stands in

```
FINDING
axis: bugs
severity: low
where: src/infrastructure/git.rs:131 (GitSources::remote_tags), src/infrastructure/git.rs:390 (git)
title: build outdated reads the tags under the configuration of the repository the user stands in
cause: remote_tags runs git ls-remote with no -C, so in the command's working directory, and ls-remote loads the configuration of the repository around it; the adapter's other commands run in the clone, and git clone reads no such repository
effect: misleading
consequence: run from a repository whose .git/config rewrites the recipe's URL by url.<base>.insteadOf, the listing is that other repository's, on which build outdated gives its verdict and build update decides; run from one whose configuration git refuses, each project reads `cannot read its releases` and the command exits 1
fix: run the listing in a directory that is no repository, git(Some(Path::new("/")), ...)
test: build outdated run from a directory holding .git, with git a script that fails when its working directory holds one
evidence: reproduced: git 2.56.0, from a repository with url.file:///<other>.insteadOf set to the llama.cpp URL, `git ls-remote --tags --refs -- https://github.com/ggml-org/llama.cpp.git 'b*'` printed the other repository's b999, and with -C / the real b12226; from a repository with a broken .git/config, exit 128 and `fatal: bad config line 6`, and with -C / exit 0
```

Reviewers: session. Origin: new. Floor: above. Fingerprint: `bugs | src/infrastructure/git.rs | GitSources::remote_tags + git`.
Verdict: CONFIRMED. Action: fix, batch 2. Sibling path: git clone, the adapter's other command with no -C, was run from the same repository and is not steered by its configuration.
Fixed in `fix(build): read the remote's tags outside the user's repository`.

### 3. build update says no build is removed, where a build off its recipe is replaced in place

```
FINDING
axis: the CLI's contract
severity: low
where: src/presentation/cli.rs:103 (BuildAction), docs/commands.md:260 (top level), src/application/build.rs:287 (Builder::update_project)
title: build update says no build is removed, where a build off its recipe is replaced in place
cause: its help and docs/commands.md:260 say that no build is removed and that the build replaced stays for build use, while update_project passes force for OffRecipe and the store replaces the installed build of that id
effect: misleading
consequence: a user counting on build use to go back to a build off its recipe finds it replaced, the summary printing `built b12177` with no way back
fix: say that a build off its recipe is replaced by its new build of the same release and that no other is removed, in the help and in docs/commands.md
test: none: a text, and a_build_off_its_recipe_at_the_newest_release_is_built_again holds the behaviour
evidence: read: src/presentation/cli.rs:103; docs/commands.md:260-261; src/application/build.rs:287 and 210; src/application/build.rs:3352, the test asserting the replacement
```

Reviewers: session. Origin: new. Floor: above. Fingerprint: `the CLI's contract | docs/commands.md + src/application/build.rs + src/presentation/cli.rs | BuildAction + Builder::update_project + top level`.
Verdict: CONFIRMED. Action: fix, batch 3.
Fixed in `docs(build): say that build update replaces a build off its recipe`.

### 4. a refusal reached through build update names --force, which build update does not take

```
FINDING
axis: bugs, the CLI's contract
severity: low
where: src/application/build.rs:324 (Builder::taken), src/application/build.rs:315 (Builder::taken), src/application/build.rs:295 (Builder::update_project)
title: a refusal reached through build update names --force, which build update does not take
cause: the messages of taken name the option alone, written for build; update_project prints the same error in build update's summary, and BuildAction::Update has no --force
effect: misleading
consequence: for a project behind whose newest build is installed, the summary says `--force builds it again`; `ai-ctl build update --force`, on the command that printed it, stops at the command line with exit 2
fix: name the whole command as Current::installed does, `ai-ctl build <project> --ref <id> --force`, in both messages of taken that reach build update
test: a_project_behind_is_not_built_again_over_its_newest_build asserts the whole command in the failure
evidence: read: src/application/build.rs:315 and 324, the bare option; src/application/build.rs:539-540, the whole command; src/presentation/cli.rs:102-110, no --force on Update; src/application/build.rs:3396-3399, the message expected
```

Reviewers: session. Origin: older. Floor: above. Fingerprint: `bugs | src/application/build.rs | Builder::taken + Builder::update_project`.
Verdict: CONFIRMED. Action: fix, batch 4. Origin older by the blame of the message, right for build; it misleads only through build update, the phase's code, so the phase fixes it.
Fixed in `fix(build): name the whole command in a refused build's remedy`.

### 5. the verdict of a llama-swap ahead of the pin is left out of build outdated's document

```
FINDING
axis: the CLI's contract
where: docs/commands.md:240 (top level), src/presentation/build.rs:254 (swap_verdict)
title: the verdict of a llama-swap ahead of the pin is left out of build outdated's document
cause: the paragraph on llama-swap's line says what is named behind the pin or not installed; swap_verdict also names ai-ctl gateway install ahead of it
effect: none
consequence: none beyond the omission: the line says itself what to run
fix: add that ahead of the pin the line names the same command, which goes back to it
test: none: a text, and llama_swap_is_told_against_the_pin_with_the_command_that_installs_it holds the verdict
evidence: read: docs/commands.md:240-241; src/presentation/build.rs:254-256; src/application/gateway.rs:77-99, which installs or reuses the pinned release and makes it current
```

Reviewers: session. Origin: new. Floor: below. Fingerprint: `the CLI's contract | docs/commands.md + src/presentation/build.rs | swap_verdict + top level`.
Verdict: none, below the floor. Action: fix at once, batch 5.
Fixed in `refactor(build): read what is outdated once for both commands`.

### 6. two arms of run repeat the same six-argument call of build::outdated

```
FINDING
axis: refactoring
where: src/presentation/build.rs:44 (run), src/presentation/build.rs:32 (run)
title: two arms of run repeat the same six-argument call of build::outdated
cause: the Outdated and Update arms each spell the call out, beside the helper builder(services) that the same file holds for the other use case
effect: maintenance
consequence: a port added to outdated changes both arms
fix: one helper outdated(services, project, warn) beside builder, called by both arms
test: none: the behaviour stays, and tests/cli.rs goes through both arms
evidence: read: src/presentation/build.rs:32-39 and 44-51; src/presentation/build.rs:95-106, builder
```

Reviewers: session. Origin: new. Floor: below. Fingerprint: `refactoring | src/presentation/build.rs | run`.
Verdict: none, below the floor. Action: fix at once, batch 5.
Fixed in `refactor(build): read what is outdated once for both commands`.

### 7. build update builds a release the project withdrew, where build outdated announced the newest the remote lists

```
FINDING
axis: bugs
severity: low
where: src/infrastructure/git.rs:79 (GitSources::update), src/application/build.rs:331 (Builder::latest_release), src/application/build.rs:142 (Builder::build), src/application/build.rs:820 (project_standing)
title: build update builds a release the project withdrew, where build outdated announced the newest the remote lists
cause: fetch runs --prune --tags, which does not prune tags fetched through --tags, so a withdrawn tag stays in the clone; build takes its newest release from the clone, while build outdated takes it from the remote
effect: wrong action
consequence: a project behind whose newest tag was withdrawn after the clone fetched it: build outdated says newest b12177, and build update builds and makes current the withdrawn b12200, which outdated then calls current
fix: add --prune-tags to the fetch of GitSources::update, with finding 1's fix so that a build past the newest is not sent back when its tag goes
test: after the upstream deletes a tag, a second fetch leaves it out of release_tags
evidence: reproduced: git 2.56.0, a clone of a repository holding b1 and b2, `tag -d b2` upstream, then `git fetch --quiet --prune --tags --force origin` in the clone: `tag --list` still gives b1 and b2, `ls-remote --tags --refs origin` gives b1 alone; read: src/application/build.rs:142 and 331-332, the build's release from the clone's tags
```

Reviewers: review-verifier. Origin: older. Floor: above. Fingerprint: `bugs | src/application/build.rs + src/infrastructure/git.rs | Builder::build + Builder::latest_release + GitSources::update + project_standing`.
Verdict: CONFIRMED. Action: fix, batch 6. Registered at this review's close as older code and low; fixed in the closing pass of the roadmap, whose rule fixes what bears on its code, build update and build outdated among it: --prune-tags on the fetch of GitSources::update, with a test on a local upstream that withdraws a release and a tag that is not one.
Fixed in `fix(build): drop the tags a project withdrew at each fetch`.

## Coverage

- `review-verifier`: in full: none; in part: src/domain/build.rs (lines 150-194, 505-564), src/application/build.rs (lines 110-335, 528-545, 818-860, 942-1022, 1610-1625, 1662-1675, 3058-3171, 3329-3405), src/presentation/cli.rs (lines 93-112), docs/commands.md (lines 222-271), src/infrastructure/git.rs (lines 30-235, 380-450), src/infrastructure/build_store.rs (lines 305-330), tests/cli.rs (lines 205-265); left out: build update run against the real stack: outside what a review agent may run; its paths are held by the tests with the repository's doubles
- `session`: in full: none; in part: src/application/build.rs (lines 1-1024, 2674-3442), src/application/gateway.rs (lines 63-106), src/application/lifecycle.rs (lines 909-914), src/application/gateway_config.rs (the diff, lines 57-66 and 89-112), src/application/ports.rs (the diff, lines 150-152), src/domain/build.rs (lines 1-300, 465-564), src/domain/recipe.rs (lines 1-110, 215-285, the diff), src/domain/inventory.rs (lines 560-582), src/domain/patches/audio.cpp/api-key.patch (lines 1-60), src/infrastructure/git.rs (lines 1-602), src/presentation/build.rs (lines 1-330, 344-384, 420-690), src/presentation/cli.rs (lines 68-110), tests/cli.rs (lines 118-138, 212-302), docs/commands.md (the diff), docs/security.md (lines 1-75, the diff), docs/stack.md (the diff), docs/audio/audio-cpp.md (the diff), docs/image/stable-diffusion-cpp.md (the diff), README.md (the diff), CHANGELOG.md (the diff), tests/fixtures/README.md (the diff), CLAUDE.md; left out: tests/fixtures/*-git-ls-remote*.txt: captures, outside the reading, parsed by the tests read; build update on this machine: run by the phase's use on 2026-11-09, not again, as it builds and changes the machine

## Fix Plan

- Batch 1: findings 1
- Batch 2: findings 2
- Batch 3: findings 3
- Batch 4: findings 4
- Batch 5: findings 5, 6
- Batch 6: findings 7
