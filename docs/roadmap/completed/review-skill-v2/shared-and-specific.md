# Shared And Specific: The Review Skill V2

What of `reviewing-delivered-code`, its agents and `reviewing-as-an-agent` would hold in any
repository, and what is ai-ctl's, written after the first full cycle: phase 5's review of
`audio-image-backends`, phase 6's fixes and the closing pass. It takes up phase 0's draft
([Design, The Shares](phase-0-decisions-and-design.md)) and checks each line against that use,
as roadmap `review-skill` left its `workflow.md`. It states what the reports say and the files
hold; it does not design the skill that will build review skills for other repositories.

---

## The Cycle It Rests On

- **The review** ([phase 5 report](phase-5-first-real-use-report.md)): tier `high`, the six
  reviewers and the dependency axis on `audio-image-backends`' diff; 56 findings, 46 fixed in
  sixteen batches, 34 of them confirmed by a verifier, nine registered, one refuted. It took
  three sessions, two accounts and a batch forced past the usage window's threshold.
- **The fixes and the closing pass** ([phase 6 report](phase-6-fixes-and-closing-pass-report.md)):
  sixteen batches, one commit each, 53 mutants applied by hand and recorded; then
  `review-security`, its verifier and the dependency axis on the roadmap's 138 hunks: three
  findings, one above the floor, rated `low`, all three fixed, and no pass after. The record
  ended at 58 mutants, each caught.
- **The skill's failures in use**: ten in phase 5, three in phase 6, each quoted in the evals'
  `failures_in_use` and fixed at its line. Most sit on the session's side — the scope, the tier,
  its writes, its records, the fix method —; every agent's answer passed `findings.py add` at its
  first submission, and the guard blocked no stop.

---

## The Line, Part By Part

| Part | Every repository | ai-ctl's | Where it lives |
|---|---|---|---|
| The moments | A review per phase that changed code, one closing pass per roadmap, a full review only on request; the stopping rule: no pass after the closing one, its fixes resting on their tests, their mutants and the regression run | None | `SKILL.md`, The Moments |
| The go-ahead | One go-ahead per review, its tier and agents counted, the estimate before any start, a start refused past a window's threshold | None | `SKILL.md` step 2, `usage.py` of `authoring-skills` |
| The tier | The surface over the size: a scope touching a guarded surface with no later security pass takes `high` | The surfaces: those `docs/security.md` guards | `SKILL.md` step 2 |
| The scope | By hunk, each with the items holding it, the items given whole; the routes of the other paths; the end of a roadmap's scope at its last phase report | The items' parser: Rust's `fn`, `impl`, trait, type, constant, macro; the routes: Rust files, `include_str!`, `Cargo.toml`, `tests/fixtures/` | `scripts/scope.py` |
| The finding | The block, its effect, the floor, the fingerprint, the merge, the register and its match | The severity anchors and the attack classes, from `docs/security.md` §1 and §2 | `scripts/findings.py`, `reviewing-as-an-agent`, Severity |
| The records | `findings.json` and the report written from it, `repro/` until the last fix, `register.json`, `mutants.json` and its regression run | The runner: `cargo test --locked -- --exact`, libtest's paths; one report line naming `cargo mutants` | `scripts/findings.py`, `scripts/regression.py` |
| The agents' rules | Injected by `skills:`; writes in scratch alone; the guard at each stop on `git status`, file hashes, processes and background tasks; `PROBE` for a run the rules forbid | The commands that read: `doctor`, `status`, `build list`, three Makefile targets, `cargo run` of the crate; podman, llama-swap and NVML used without changing the machine | `reviewing-as-an-agent`, Where You Work; `scripts/guard.py` |
| The axes | Six reviewers and a verifier per reviewer, each refuting by its axis's method | Their content: the CLI's contract in `docs/commands.md`, the on-disk formats, the processes and waits of a local CLI, the threat model of one user on one machine | `.claude/agents/review-*.md` |
| The dependency axis | A script gathers the facts, an agent judges them, the routing of its findings; no personal data in a request | `cargo-deny`, crates.io and `Cargo.lock`; the lag of llama-swap and of the four backend sources, by `git ls-remote` | `scripts/dependencies.py`, `review-dependencies.md` |
| The triage | The session triages; the hard cases alone go to the user | The documented contracts and guarantees those cases cite: `docs/commands.md`, `docs/security.md` | `SKILL.md` step 7 |
| The fix method | Sibling paths, what a fix to what the code makes leaves made, input classes, a document's statements as claims, a red test first, the batch's mutants, the whole record after each batch | Rust's kinds among those applied by hand: a `let … else` taking its `else`, a `?` swallowing its error | `references/fixing.md` |
| The checks | Each check of the repository's contract, run once and recorded | The checks themselves, one of them run through a link git ignores | `.agent-conventions.toml` |

Another language's tools, such as mutmut and pip-audit for Python or Stryker and npm audit for
Node, are each repository's to check, as phase 0 left them.

---

## What The Use Moved

Phase 0's draft held. The use moved six lines of it, or showed a part it had not named:

- **The tier's rule is shared, its anchor is not.** Phase 5 chose `high` for a scope the session
  could read alone, because it touched surfaces `docs/security.md` guards. Every repository can
  take the rule; it needs a document saying who reaches what, or the rule has nothing to read.
- **The scripts mix both sides.** The draft put the scope by hunk with every repository and the
  Rust items with ai-ctl; `scope.py` holds both in one file, the hunk split and the routes beside a
  tokenizer of Rust. `guard.py` likewise: its mechanism is shared, its list of allowed commands
  ai-ctl's. `findings.py` is shared but for one report line.
- **A repository's own tests can stop the mutation tool.** Phase 6's first batch ended at
  cargo-mutants' exit 4, every check green: a test of the repository read `.git`, which the
  tool's copy lacks. The case is shared, a test needing what a copy lacks; the answer, a
  `.cargo/mutants.toml` skipping that test, is ai-ctl's.
- **A record may cite what another clone lacks.** The review's `checks` named a program through a
  link git ignores, and turned a check red once committed. `findings.py` now refuses
  such a path; the case is shared, the link ai-ctl's.
- **The fix method missed what a fix leaves made.** A recipe's option turned off reached no build
  installed before it, which neither the sibling paths nor the input class named; the closing
  pass found it. The rule added is shared: a fix to what the code makes names what it made
  before.
- **The record decays between batches.** Batches that rewrote a recorded mutant's line left it
  gone until the closing pass ran the whole record: now each batch runs it. Shared.

---

## What A Repository Gives The Skill

Each ai-ctl's line of the table above stands for something another repository gives in its own
form:

- a document saying who reaches what and the severity of each kind of damage, for the tier, the
  security axis and the anchors;
- its conventions and architecture, for `review-structure`, and its contract: commands, formats,
  outputs;
- a parser of its language's items for the scope, or every line read as top level;
- its test runner and mutation tool, the kinds of mutants that tool does not write, and the
  repository's own tests a tool's copy cannot run;
- its dependency tools, and the upstream projects it builds or runs whose lag counts;
- its commands that only read, and those that change the machine, for the guard;
- its checks, run once at each review.
