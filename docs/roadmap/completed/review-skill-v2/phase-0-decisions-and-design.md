# Phase 0: Decisions And Design

---

## Status

**Current Status:** 🟢 Done (100% — 12/12)
**Started:** 2026-10-31
**Completed:** 2026-11-01
**Blocked By:** —

---

## Objective

Settle with the user each point the assessment of 2026-10-19 left open, and the dependency axis,
in a Design section that phases 1 to 6 build on; send the agent setup its share as design-feedback
notes. The user decides; the session proposes, each proposal resting on the three reviews'
figures or on a probe.

### Out of Scope
Any change to the skill, its agents, its scripts or its evals. Any paid run.

---

## Tasks

### The Workflow
- [x] Write the review-and-fix workflow and its stopping rule: at each fix batch, test-first, `cargo mutants --in-diff` and the fix's sibling paths and input classes; at a phase's close, a review of the phase's hunks and their callers, fixed in the phase; at a roadmap's close, a security pass on the roadmap's diff and the recorded mutants run again; that pass's findings fixed in the same phase when they bear on the roadmap's fixes or are medium or worse, the rest to the debt register, and no pass after; a full review only at the user's request. Decide who triggers the review at a phase's close: the roadmap skill or this repository's conventions
  Proof: the user's review of the Design section
- [x] Decide the gates: one go-ahead per review, its agents counted and an order of magnitude from `usage.py estimate`; the session's triage of the verdicts, the user asked only the hard cases; the per-wave yes, the rates and the plan's figures removed
  Proof: the user's review of the Design section

### Findings And Report
- [x] Design the structured finding: the fields every reviewer gives (axis, place by symbol and `file:line`, cause, effect — data lost, wrong action, misleading, cosmetic, none —, severity, evidence, proposed fix and its test), its origin, new or older code, from `git blame` against the scope's base, and the floor that sends a finding with no effect for the user to the debt register with no verification
  Proof: the user's review; one finding of each of the three reports rewritten in the format, no field left empty
- [x] Design the debt register: where it lives, a fingerprint free of line numbers, how a later review recognizes a finding registered or rejected
  Proof: the user's review; two findings of the 2026-10-17 report whose lines the fixes moved keep their fingerprint at `HEAD`
- [x] Design the report writer: the answers it reads, the checks of their fields and counts, the numbering and the merges, the report with a summary table at its top, the fix plan's frame and the fix roadmap's tasks; and its name by `authoring-skills`' rules
  Proof: the user's review of the Design section
- [x] Design the `repro/` folder of a review (each finding's failing test, patch and probe script), the verifier's product (the red test and the settled fix, the refutation a by-product, its independence kept), and the regression run of the recorded mutants at a roadmap's close
  Proof: the user's review; one reproduction of the 2026-10-17 closing review laid out in the format

### Scope And Mutants
- [x] Specify the scope by hunk, the diff's hunks and the functions that call them, each file's rest read only to follow a path, and the origin by blame
  Proof: the lines a scope by hunk hands the reviewers, counted by hand on `v2-review-fixes`' range, against the 15,218 of the scope by file
- [x] Decide the mutants: in the background from the review's start, split with `--shard` past an hour, the limit of 300 removed; per batch at the fix, with the kinds `cargo mutants` does not write and a fix applies by hand (an `if` guard, a `let … else`, a `?`, a match arm, a lock, a clamp, a truncation)
  Proof: `cargo mutants --list --in-diff` on `audio-image-backends`' diff, its count recorded

### The Agents
- [x] Design the agents' shared rules: one file for what may run, the sandbox and its check after each wave (`git status --porcelain`, `pgrep`, `ss`), the threat context of `docs/security.md` read before any severity; the `PROBES` channel for a probe a rule forbids; an owner for a branch an earlier guard makes unreachable; how `review-cleanup` learns a direction under discussion now that no tracked file cites a note; whether `isolation: "worktree"` gives each agent its copy; `review-security`'s method from the lessons of the security skill read on 2026-10-19 — the candidate gate, the severity anchors, `needs_validation`, the hunting of sibling paths
  Proof: the user's review; a probe of the mechanism by which an agent loads the shared file, after the user's yes

### The Dependency Axis
- [x] Settle the dependency axis with the user: its scope (the crates of `Cargo.lock`, direct and transitive; whether the backend sources, llama-swap and the container images join it); its criteria (advisories, unmaintained and yanked releases; the newer versions within and beyond `Cargo.toml`'s requirements; the last release, the release rhythm, the owners, an archived repository; downloads and reverse dependencies, a single maintainer, `unsafe` and FFI; licences, sources and duplicates; a new crate against `CLAUDE.md`'s Dependencies rule); its tools (`cargo audit`, and whether `cargo-deny`, `cargo-outdated` and crates.io's API join it); its network access; whether a script gathers the facts for an agent to judge; which agent owns it; whether it runs alone on request, as the security pass did
  Proof: the user's review; each criterion's source queried once on this machine, its output kept as a capture

### Shares
- [x] Write the agent setup's share as design-feedback notes, each claim checked first in the tool's current version: the phase report's size, a row per batch; a command that ticks a task in `progress.py`; the progress bar of a finished phase not yet closed; the Edit and Write tools decoding `\u2028` and `\u2029`; the review at a phase's close, if the roadmap skill triggers it
  Proof: each note among the agent setup's design-feedback notes, untracked, its claims each checked against the tool's code
- [x] Decide what is ai-ctl's and what every repository shares, for the skill that builds review skills — the orchestration, the axes, each language's tools —, and where the record lives; adjust phases 1 to 6 to the decisions
  Proof: the user's review of the Design section and of the phase files

---

## Design

### The Workflow And Its Stopping Rule

| Moment | What runs | Its findings |
|---|---|---|
| Each fix batch, each commit of code | Test-first; `cargo mutants --in-diff` on the batch's diff, a missed mutant killed or justified; a new test that spawns a process, binds a port or takes a lock run 30 times (`CLAUDE.md`, Tests); the fix method — sibling paths, input classes, the mutants a fix applies by hand. No agent | — |
| A phase's close, when the phase changed code | A review of the phase's hunks and their callers on every axis, its tier proposed at the go-ahead by the diff's size, the session alone for a small one | Fixed in the phase: on the phase's code, above the floor; on older code, medium or worse. Below the floor, on the scope's new code and with a local fix, fixed at once in one commit. The rest to the debt register |
| A roadmap's close, when the roadmap changed code | A security pass on the roadmap's diff by hunk; the dependency axis on `Cargo.lock`, its crates added or updated and the advisories published since; the recorded mutants run again by a script, with no agent | Fixed in the last phase when they bear on the roadmap's code or are medium or worse. The rest to the register |
| A full review | Only at the user's request, such as after a rebuild; never as a roadmap's last step | Its fixes end by the roadmap's close rule |

- **The stopping rule:** no pass after the roadmap's closing pass. Its fixes rest on their tests,
  their mutants and the regression run.
- **A phase with no code change has no review.** `audio-image-backends`' phase 0 added only
  captures.
- **The trigger:** `CLAUDE.md` states it from phase 3 on, once the v2 exists: a phase that changes
  code ends with the review of its hunks, its findings fixed before its closure. A design-feedback
  note proposes that the roadmap skill read a closing step a repository declares in
  `.agent-conventions.toml`'s `[roadmap]` table; the rule moves there once the skill reads it.

The sizes a review at a phase's close would have met, `audio-image-backends`' code by phase,
measured on 2026-10-31 with `git diff --shortstat` between the phases' start commits:

| Phase | `src/` | `tests/` without `tests/fixtures/` |
|---|---|---|
| 0 | none: captures only | none |
| 1 | +1,755 −97 | +9 −4 |
| 2 | +2,477 −172 | +494 −11 |
| 3 | +29 −2 | +560 −454 |
| The roadmap | +4,261 −271 | +609 −15 |

The scope by file of the same roadmap, by `scope.py roadmap`, hands the reviewers 11,573
production and 17,001 test lines.

### The Gates

- **One go-ahead per review**, at its start: the tier, the agents counted — the reviewers, the
  verifiers by the tier's rule —, and one line of `usage.py estimate`. It covers every wave of the
  review, across sessions.
- **A request that accepts the cost beforehand is the go-ahead:** the line is shown and the review
  starts.
- **The window's guard is `usage.py`:** `usage.py estimate --start` refuses a wave past 90% of a
  window. The review then stops with its state in the report's `Status`, and resumes in a later
  window with no new go-ahead.
- **Removed:** the yes per wave and the yes given in the session that starts it, the session's
  running total, the rates, the plan's figures and the passes of $15.
- **The session triages.** A confirmed finding is fixed or registered by the floor; a refuted one
  leaves the report and enters the register as rejected, with its fingerprint, so that no later
  review raises it again. The user is asked only the hard cases, which grow with use:
  - a finding still `UNCERTAIN` after its verification;
  - a fix that changes a behaviour the user sees, a documented contract (`docs/commands.md`, an
    on-disk format) or a guarantee of `docs/security.md`;
  - a fix that removes a feature, or changes a crate beyond `Cargo.toml`'s requirements or
    replaces it;
  - a reviewer and its verifier disagreeing on the severity or the fix;
  - a fix the session finds out of proportion with its effect, with its reasons.

### The Structured Finding

Every reviewer answers in one block per cause. The fields of today's `FINDING` block stay; three
are added or changed:

- **`where`** gives each place's symbol: `path:line (symbol)`. The symbol finds the code again
  once the lines move.
- **`effect`**, what the user meets, one of six:

  | Effect | Meaning |
  |---|---|
  | `data lost` | data the user loses |
  | `wrong action` | a wrong result, a wrong process stopped, a failure |
  | `misleading` | a message, a document or an output that misleads |
  | `slow` | a wait the user notices |
  | `maintenance` | no effect today, a cost to the next change: dead code, a copy, a stale comment, a convention |
  | `none` | no effect |

  A test gap takes the effect of the defect it would let through, so that test findings sort
  like the others: review 2's 28 test gaps had no severity.
- **`test`**, the test that fails before the fix, or `none:` and why.

`severity`, `critical`, `high`, `medium` or `low`, is required above the floor, by anchors common
to every axis that `review-security` refines (The Agents). The **origin**, `new`, `older` or
`mixed`, is no reviewer's field: the report writer computes it by `git blame` on the places'
lines against the scope's base.

**The floor:** `maintenance`, `none`, and a `slow` too small to notice. A finding below it gets
no verification, no verdict and no batch: on the scope's new code with a local fix, the session
fixes it at once, with the others of its kind in one commit; otherwise it goes to the debt
register.

One finding of each report rewritten in the format, its places at `HEAD`:

```
FINDING                                   (2026-10-15, finding 99)
axis: refactoring
where: src/infrastructure/gateway_process.rs:103 (liveness_of), src/infrastructure/gateway_process.rs:309 (Procfs::program)
title: /proc/<pid>/exe is read and classified twice
cause: liveness_of and Procfs::program each read the link and map its errors
effect: maintenance
consequence: two readers of one kernel interface in one file
fix: none: the readers follow two port contracts; a shared reader costs what it saves
test: none: no behaviour changes
evidence: read: both bodies
```

```
FINDING                                   (2026-10-17, finding 84)
axis: performance
severity: low
where: src/application/doctor.rs:69 (Doctor::run), src/application/toolchain.rs:29 (toolchain::inspect)
title: doctor runs its independent probes one after another
cause: Doctor::run and toolchain::inspect call each probe in turn
effect: slow
consequence: doctor waits 224 to 258 ms where its four independent groups at once take 154 to 156 ms
fix: the four groups on std::thread::scope threads, + Sync on the ports concerned
test: none: a test depending on timing is forbidden (CLAUDE.md, Tests)
evidence: reproduced: probes.sh, the commands doctor runs, in turn then at once
```

```
FINDING                                   (2026-10-19, finding 1)
axis: security, bugs, error handling
severity: low
where: src/infrastructure/gateway_process.rs:350 (TcpPorts::listens), src/infrastructure/gateway_process.rs:332 (TcpPorts::is_free)
title: up never recognises llama-swap's socket on 0.0.0.0
cause: listens reads /proc/net/tcp for an IPv4 address, where Go's wildcard listener is one dual-stack socket in /proc/net/tcp6
effect: wrong action
consequence: with expose = true and listen = "0.0.0.0:8080", up stops the gateway after 30 s and blames llama-swap
fix: one pure function mapping listen to the sockets Go opens, shared by listens and is_free
test: listens true on the capture of llama-swap v263 listening on 0.0.0.0
evidence: reproduced: llama-swap v263 in unshare -rn, its socket in /proc/net/tcp6 only, listens false
```

### The Debt Register And The Fingerprint

- **The fingerprint** is the axis, the paths and the symbols, with no line:
  `refactoring | src/infrastructure/gateway_process.rs | Procfs::program + liveness_of`.
- **A match is a candidate:** two defects can share a symbol and an axis, so the report writer
  shows the session each finding whose fingerprint the register holds, and the session compares
  their titles. The worst case is one comparison more, never a finding lost.
- **The register** is one tracked file for the repository,
  `docs/reviews/delivered-code/register.json`: each entry's fingerprint, axis, effect and title,
  its state — kept, rejected or open —, the review and finding it came from, and its reason.

Measured on 2026-10-31, against the code at `HEAD`:

- The 2026-10-17 review's `review-cleanup` recognized the 2026-10-15 findings 99 and 100 by their
  symbols, "`liveness_of` against `Procfs::program` (first review's 99, kept)", while
  `Procfs::program` had moved from line 244 to 309.
- The 2026-10-17 findings 3 and 84 keep their fingerprints at `HEAD`, every symbol they cite still
  there: `Lifecycle::up` from line 224 to 244, `wait_until_healthy` from 695 to 538,
  `GatewayApi::healthy` at 74 and `start` from 187 to 193 for finding 3; `Doctor::run` from 71 to
  69 and `toolchain::inspect` at 29 for finding 84.

### The Report Writer

- **A folder per review**, `docs/reviews/delivered-code/<date>-<scope>/`: `findings.json`, the
  source; `report.md`, written from it; `repro/`. The three reports before the v2 stay as they
  are.
- **`findings.py`**, on that folder:
  - reads the agents' answers and checks each block: its fields, their allowed values, the count
    an answer gives against its blocks; a malformed block is refused with its field named;
  - numbers the findings, and merges them on request, `merge 12 15`;
  - computes each finding's origin and fingerprint, and matches it against the register;
  - writes `report.md`.
- **The report** opens on a summary table, one row per finding: number, title, axis, effect,
  severity, origin, verdict, action, batch. The findings in full follow, then Coverage from the
  agents' `FILES READ` lists, the fix plan's frame, and the fix roadmap's tasks, each with its
  `Proof:` line.

### Reproductions, The Verifier, The Regression Run

- **`repro/<n>/`** holds, as the finding needs, `test.patch`, the red test; `fix.patch`, the
  settled fix; `probe.sh`. A fix batch applies `test.patch`, sees it fail, then fixes. The
  copies kept by hand outside the repository end.
- **The verifier's product** is the red test, seen failing in its own copy, and the settled fix.
  The verdict stays; the refutation becomes a by-product, and its independence holds: it builds
  its own reproduction.
- **The recorded mutants** are the ones a fix applies by hand, such as an `if` guard, a `?` or a
  lock, which `cargo mutants` does not write. Each is kept as an exact replacement — its file, the
  text replaced, which must occur once, its replacement, and the test that kills it — in
  `docs/reviews/delivered-code/mutants.json`. The 2026-10-17 review's `review-tests` applied its
  M1 to M25 that way (a file, an old and a new text, a filter of the tests), each with a log of
  its run and of its kill; a replacement survives the lines moving, where a patch's context does
  not. A mutant whose text no longer occurs is reported, then retired with its code.
- **The regression run at a roadmap's close:** `cargo mutants --in-diff` on the roadmap's diff in
  the background, for the mutants the tool writes, and a script that applies each recorded
  mutant in a copy and runs its test, for the others.
- **`repro/` is removed in the fixes' last commit:** its tests then live in `tests/` and the
  `#[cfg(test)]` modules, and git keeps the rest.

One reproduction of the 2026-10-17 review laid out in the format: its `review-tests` wrote one
patch of 18 files for every proposed test, and mutants such as M17, `publish-restore`, by a file,
an old and a new text. In the v2, each finding's tests become `repro/<n>/test.patch`, and M17
an entry of `mutants.json` with its file, its two texts and its killing test, which its kill log
shows failing alone, "20 passed; 1 failed", under a test filter.

### The Scope By Hunk

- **`scope.py` gives the hunks** of each file and the item that holds each, a function, an
  `impl`, a type or a constant, as `path:first-last (symbol)`, with their counts. The list of
  files read whole goes.
- **New code is analysed in its context** (the user, 2026-10-31): the item that holds the hunk,
  whole; its callers one level up and its callees one level down; the sibling paths to the same
  effect. The context is analysed, not only read to follow a path, even when a later review
  analyses the same code again: "je pense que c'est la solution la plus sûre". A finding in the
  context has the origin `older` and follows the floor's routing; the register keeps a later
  review from raising it again.
- **The agents find the callers and callees**, by searching for the qualified symbol, as their
  method already asks, rather than the script by name, which counts too much.
- **Hunks outside the code:** `Cargo.toml` and `Cargo.lock` go to the dependency axis, `docs/` to
  the CLI's contract; captures under `tests/fixtures/` stay in the diff, outside the reading.
- **Items are found** with the standard library, by `fn` and `impl` lines and matched braces,
  under tests on captured files, braces inside strings and comments included.

The production lines a scope hands the reviewers, measured on 2026-10-31 by a script that sums
the functions holding a change, then those and the functions that call them by name, an upper
bound since common names such as `new`, `run` or `parse` match unrelated functions:

| Roadmap | By file | Functions holding a change | With their callers by name |
|---|---|---|---|
| `v2-review-fixes`, from `docs(v2-review-fixes): create the roadmap` of 2026-10-16 to `docs(v2-review-fixes): record the closing review's scope and tier` of 2026-10-17 | 15,218 | 5,135 | 7,846 |
| `audio-image-backends`, from `docs(audio-image-backends): create the roadmap` of 2026-10-19 to `docs(audio-image-backends): close the roadmap without phase 4` of 2026-10-30 | 11,573 | 1,974 | 5,016 |

### The Mutants

- **At the review:** `cargo mutants --in-diff` on the scope's diff, started in the background at
  the review's start, before the agents, in a `git archive` copy of the scope's end. The copy
  because `--in-diff` refuses a tree that differs from the diff: on 2026-10-31 it refused the
  working tree, `doctor.rs` having changed since `docs(audio-image-backends): close the roadmap without phase 4` of 2026-10-30; and the session may go on working
  while it runs. No limit; split with `--shard` when the run would pass an hour, about 1,200
  mutants at 284 in 14 minutes, measured on 2026-10-19. The missed mutants go to `review-tests` as
  leads.
- **`audio-image-backends`' diff lists 198 mutants** (cargo-mutants 27.1.0, 2026-10-31), about
  10 minutes at that rate.
- **At the fix:** `--in-diff` on each batch (`CLAUDE.md`, Tests), and by hand the kinds
  `cargo mutants` does not write, each recorded in `mutants.json`: an `if` guard forced true or
  false, a `let … else` taking its `else`, a `?` swallowing its error, a match arm removed, a
  lock not taken, a clamp removed, a truncation removed.

### The Agents

- **One file of shared rules, injected into every review agent** by the `skills:` field of its
  definition: what may run, the sandbox, the threat context — `docs/security.md` §1 and §2 read
  before any severity —, the `PROBES` channel, the common fields of the `FINDING` block and the
  severity anchors. The seven copies of the rules on what may run go.
- **Mechanical guards rather than rules in prose**, as hooks of the project's
  `.claude/settings.json`, each acting only when the hook's input names a review agent in
  `agent_type`: `PreToolUse` on Bash refuses ai-ctl's commands that start the stack or change the
  machine (`up`, `down`, `build`, `gateway install`, `init`) and the git commands that write to the
  repository; `SubagentStop` checks the sandbox — the repository's `git status --porcelain`
  unchanged since the wave started, no process or port the agent started left — and blocks the
  stop with what is left, so that the agent cleans up itself.
- **No `isolation: worktree`:** the worktree branches by default from the default branch, not
  from the session's `HEAD`, so on `roadmap/<name>` an agent would read `main`'s code. The agents
  keep their own `git archive` copy of the scope's end in scratch.
- **`PROBES`:** a `PROBE` block in an agent's answer gives the command, what it changes or risks,
  what it settles, and the finding it serves. The session gathers them and puts them to the user
  as hard cases, then runs those the user accepts.
- **A branch an earlier guard makes unreachable** is dead code, `review-cleanup`'s. A mutant
  `review-tests` sees surviving on such a branch goes to dead code as a lead, not as "not a test
  gap": the 2026-10-17 review's finding 69.
- **A direction under discussion**, for `review-cleanup`: the roadmaps under `pending/` and
  `on-progress/` and the closed roadmaps' What We Are Leaving Open, all tracked, instead of
  `docs/notes/`.

### The Method Of `review-security`

From the lessons of the security skill read on 2026-10-31, outside the repository; six of its
ten points are decided above (the register of rejected findings, the fingerprint, the
structured answer and the generated report, the agents counted, the floor as its hardening
notes, `PROBES` as its `needs_validation`). The threat context is `docs/security.md` §1, who
reaches what, and §2, this machine's one login account.

- **A gate on every candidate.** A security finding names who attacks and their starting
  capability, a row of §1; the input accepted; the control that should stop it; the boundary
  crossed; the resource reached; the result observed. With no boundary crossed it is no security
  finding — "Self-harm within the same account is not a boundary violation" — and may stay a
  finding of another axis.
- **Severity anchors of ai-ctl's own:**

  | Severity | Anchor |
  |---|---|
  | critical | the network gets the key or runs code, with no action of the user |
  | high | a web page through DNS rebinding, a container, or content ai-ctl fetches — a release, a source, an image, a model — gets the key, runs code or reads the prompts |
  | medium | another local account gets the key or the prompts |
  | low | a control weakened with no damage shown, or a defect that fails closed |

  Between two, the lessons' rule: does the result fully defeat an explicit control for an action
  with real consequences, or only weaken it? With no concrete damage stated, the lower one. By
  these anchors, the 2026-10-17 review's finding 1, a backend's port taken by another local
  account, is medium where it was rated high.
- **The hunting of sibling paths:** each input followed through the legacy, batch, retry,
  cancellation and error paths that reach the same effect; sibling controls compared for
  equivalence, not only for presence; a root cause's variants searched in the scope; a line of
  search stopped once its invariant is settled. The 2026-10-19 pass's S1, IPv4 alone beside the
  dual-stack socket, and S4, the format characters beside the controls, are such misses.
- **ai-ctl's attack classes,** written in the agent: injection through the CLI (commands, paths,
  environment); files (symlinks, TOCTOU, archive extraction, temporary files); secrets, and the
  key in logs, URLs and errors; logic on partial failure, defaults and fallbacks; loopback
  listeners with no peer authentication, and the lifecycle of ports, locks and restarts; the
  supply chain (downloads, digests, a mutable tag); containers (podman, devices, CDI, host
  paths). Left out: the web and multi-tenant classes, and memory safety, `src/` holding no
  `unsafe`.
- **The bar for `CONFIRMED`:** a security finding medium or worse is confirmed only on an
  observed result, a run; shown by reading alone, it stays `UNCERTAIN` with its `PROBE`. A low one
  may be confirmed by reading.
- **Left out of the lessons:** their weight, the same procedure written three times; their
  sandbox for hostile target code; their cost, four reconnaissance agents in their lightest
  profile.

Probed on 2026-10-31, Claude Code 2.1.312, three `claude -p` sessions under
`--setting-sources project,local`, `claude-haiku-5-5` at effort `low`
(https://code.claude.com/docs/en/sub-agents, Preload skills into subagents and Define hooks for
subagents):

- An agent limited to the `Glob` tool, a skill named in its `skills:` field, gave the skill's rule
  word and said it came from the skill's content loaded at its start: the injection holds with no
  read.
- That skill is in the main session's list of skills too: a trigger risk, measured in phase 4.
- The `PreToolUse` and `Stop` hooks of an agent's frontmatter ran neither in a scratch folder nor
  in this repository, though the agent called the matched tool; the page says frontmatter hooks
  are skipped until the folder is trusted.
- The `PreToolUse` and `SubagentStop` hooks of a project's `.claude/settings.json` ran on the
  agent's calls, their input holding `agent_type` and `agent_id`, `SubagentStop` matched by the
  agent's name.

### The Dependency Axis

- **Its scope:** the crates of `Cargo.lock`, direct and transitive, on every criterion below; the
  backend sources — llama.cpp, whisper.cpp, stable-diffusion.cpp, audio.cpp — and llama-swap for
  their lag behind the newest release alone, large and active projects whose maintenance it does
  not judge; the container images, entries for comparison, left out for now.
- **Its tools:** `cargo-deny` for the mechanical part, in place of `cargo audit`: one
  deterministic command, `cargo deny check`, under a tracked `deny.toml` — the advisories, from
  RustSec as `cargo audit`'s; the licences against an explicit list; duplicates and banned
  crates; the sources, crates.io alone. The policy is then written, not buried in a script, and
  the CI/CD that follows this roadmap runs the same command at each tag. Installed by
  `cargo install --locked cargo-deny@0.20.2`, the newest, of 2026-07-09, needing Rust 1.88,
  pinned as `cargo-audit` was installed; Fedora's updates carry 0.19.9. `cargo-outdated` is not
  needed.
- **A script gathers what `cargo-deny` does not,** one JSON record per crate: the updates, from
  `cargo update --dry-run --verbose` and crates.io's `max_stable_version`; the maintenance, from
  crates.io's versions, their dates, and owners, and GitHub's `archived` and `pushed_at`; the
  reliability, from the reverse dependencies, the downloads and the count of `unsafe` in the
  crate's sources. crates.io takes one request a second; GitHub, unauthenticated, 60 an hour, so
  it is asked for the direct crates and the crates a criterion flags; redirects are followed. Its
  `User-Agent` names the tool and carries no personal data.
- **`review-dependencies` judges the records,** at a roadmap's close (The Workflow And Its
  Stopping Rule), at a phase's close when `Cargo.toml` or `Cargo.lock` changed, and alone on
  request, as the security pass.
- **Its findings' routing:** an update within `Cargo.toml`'s requirements applied at once,
  `cargo update` with the checks green, a local fix of a maintenance effect; a major version or a
  crate replaced, a hard case for the user (The Gates); an advisory, a security finding by
  `review-security`'s anchors, once the agent has shown the vulnerable code reachable from
  ai-ctl; a yanked or unsound crate, low at least; an unmaintained crate, to the register with an
  alternative named. The maintenance figures are facts for a decision, a finding only when a
  RustSec warning or an archived repository confirms them.
- **For the CI/CD after this roadmap,** `cargo deny check` at each tag, written in the shares
  (task 12).

Each criterion's source queried once on 2026-11-01, on this repository at `docs(review-skill-v2): decide the method of review-security` of 2026-11-01, the outputs
in the session's scratchpad:

| Criterion | Source | Result |
|---|---|---|
| Advisories, unmaintained, yanked, unsound | `cargo audit --json`, 1,395 advisories | none over 66 dependencies |
| Updates within the requirements | `cargo update --dry-run --verbose` | none; resolution bound to Rust 1.89, `rust-version` |
| Versions beyond the requirements | crates.io's `max_stable_version` | the 10 direct crates at their newest |
| Maintenance | crates.io's versions and owners, GitHub's repository | `nvml-wrapper`: 3 releases in 12 months, 3 owners, repository moved to `rust-nvml/nvml-wrapper` behind a 301, not archived, last push 2026-08-31; `tempfile`: last release 2026-03-11, one owner, last push 2026-10-30 |
| Reliability | reverse dependencies, downloads, `unsafe` in the sources | `nvml-wrapper` 141 dependents against 3,903 to 136,344 for the other direct crates; 322 `unsafe`, 794 in `nvml-wrapper-sys`, a C binding's |
| Licences, sources, duplicates | `cargo metadata --locked` | 65 crates from crates.io's registry; ten licence expressions, all permissive; `thiserror` 1.0.69, through `nvml-wrapper`, beside 2.0.21, and `syn` 2.0.119 beside 3.0.9, through `clap_derive` |
| Outside the crates | GitHub's latest release, `git ls-remote --tags` | llama-swap v269, released 2026-11-01, against the installed v263; llama.cpp `b12032` against the `b11700` build |

### The Shares

The agent setup's share, each point of the 2026-10-19 assessment checked on 2026-11-01 against
the installed roadmap 3.2.0 and skill-tooling 0.2.8 and the agent setup's open roadmaps; four
design-feedback notes written among its notes, untracked:

| Point | Its state | Sent |
|---|---|---|
| The Write and Edit tools decoding the escapes of U+2028 and U+2029 | reproduced on 2026-10-31 in this roadmap's files | a note: a `PostToolUse` hook on Write and Edit that reports a raw separator; the cause reported to Claude Code |
| The review at a phase's close | no place in `close-phase.md`, whose `checks` are commands | a note: a closing step a repository declares, designed with the documentation step that the agent setup's `fixes-and-overview` designs in its phase 1 |
| Hooks in an agent's frontmatter | skipped in `claude -p` sessions, probed on 2026-10-31 | a note for the eval runs: the fact in the harness's probe log, an audit warning |
| A command that ticks a task | `progress.py` has `--write` and `--check` alone; ticking by a script worked in this phase | a note, low priority |
| The phase report's size | the report's rules now forbid the diff retold file by file | none: phase 6's report is watched |
| The progress bar of a finished phase not closed | fixed: invariant 1 of roadmap 3.2.0 | none |

ai-ctl's own share: a check of the tracked files against a raw U+2028 or U+2029 (phase 2), and
`cargo deny check` at each tag in the CI/CD note.

What is ai-ctl's and what every repository shares, a draft that phase 6 writes in this roadmap's
folder as `shared-and-specific.md`, after the first real use, as roadmap `review-skill` left its
`workflow.md`:

| Every repository | ai-ctl's |
|---|---|
| The workflow and its stopping rule, the gates, the triage and its hard cases | The hard cases that cite `docs/commands.md` and `docs/security.md` |
| The structured finding, the effect, the floor, the fingerprint, the register | The severity anchors and the attack classes, from `docs/security.md` |
| The report writer, `repro/`, the verifier's product, the regression run | The thirteen axes' content: the CLI's contract, the on-disk formats |
| The scope by hunk, new code analysed in its context | The items of Rust found: `fn`, `impl` |
| The rules injected by `skills:`, the guards by the settings' hooks, `PROBES` | What may run: ai-ctl's commands, llama-swap, podman, NVML |
| The dependency axis's frame: a script gathers, an agent judges, the routing | The tools: `cargo-mutants`, `cargo-deny`, crates.io; the kinds of Rust mutants applied by hand, a `?`, a `let … else` |

Another language's tools, such as mutmut and pip-audit for Python or Stryker and npm audit for
Node, are each repository's to check.

---

## Technical Details

### Files to Modify
```
phase-0-decisions-and-design.md      (its Design section)
phase-1-*.md … phase-6-*.md          (adjusted at the closure)
tests/fixtures/                      (the dependency axis's captures)
tests/fixtures/README.md
```

### Dependencies
The three reviews' reports in `docs/reviews/delivered-code/`, roadmap `v2-review-fixes`:
`phase-5-closing-review-report.md` and `summary.md`, roadmap `review-skill`: `workflow.md`.
Network access to crates.io for the dependency axis's captures.

### Constraints
- Every figure the Design section gives names what was measured and its date.
- No paid session without the user's yes.
- A note stays outside the tracked files: what a note gathered and the phase keeps is written in
  the Design section.
