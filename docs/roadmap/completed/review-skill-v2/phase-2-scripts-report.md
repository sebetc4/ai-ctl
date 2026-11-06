# Phase 2 Report: Scripts

**Phase:** [phase-2-scripts.md](phase-2-scripts.md)
**Start Commit:** `docs(review-skill-v2): close phase 1` of 2026-11-02

---

## Work Log

### 2026-11-02

Phase opened; the user: "tu peux commencer".

Task 1, `scope.py` by hunk: its tests failed on their assertions against a stub, then passed,
30 runs green. On `audio-image-backends`' range, from `docs(audio-image-backends): create the roadmap` of 2026-10-19 to `docs(audio-image-backends): close the roadmap without phase 4` of 2026-10-30: 330 hunks in 25 code files
and 6,709 lines handed, 2,748 of production, where the scope by file gave 11,573. That range's
`other` route held files the binary embeds by `include_str!`, the backends' patches and the
inventory among them: made code at the user's yes, two tests watched failing, the lines handed
growing to 7,126. The user's `.gitignore` change, `/notes` to `/notes/`, committed at the
user's word.

Task 2, each line's origin by `git blame`: six tests on repositories made in scratch watched
failing. The first 30 runs gave 26 green, another session editing the same files in this tree;
once it stopped, 30 green. Against the `continued-review` fixture's 17 findings, the origin from
every place agreed on 13, a new defect citing an older reference coming out `mixed` and an
omission `older`; the user chose the site first. The fixture's two omissions now cite the new
code first, and the origin agrees with the key on all 37 findings of the two phase reviews.

Task 3, the report writer, after the user's choice of a command apart for the fix roadmap's
tasks. The fixtures' review states brought to one form by a script, patches 05, 10, 15 and 17
regenerated. `findings.py`'s 18 tests failed against a stub, then passed, 30 runs green; it gives
back the fixtures' five review states byte for byte. Tried on the built `roadmap-close` fixture,
it showed that a finding citing `stage` beside `stage_from` has not the register's fingerprint:
task 4's question. `findings.py` writes no bytecode, `scope.py`'s import having left
`scripts/__pycache__/`.

Task 4, the debt register: the match by an axis, a path and a symbol shared, 2 candidate pairs
of 190 on the fixture's 20 findings, and the action `known`, both at the user's word. Seven
tests and two assertions failed against stubs, then passed; the fixture's register given back
at its three states byte for byte. Twenty mutants by hand: all caught but two, one written
wrong, the other caught by a test added. 30 runs green.

Task 5, the regression run, after the user kept the state `kept`. A probe: `cargo test --
--exact` exits 0 when no test has the name. Capturing it here showed `gateway_lifecycle`'s own
harness printing no `test result:` line: the parser reads each `test <name> ... <status>` line,
on three captures. Tests on a crate made in scratch, twelve mutants of every outcome, an
interruption leaving no copy and no process. 33 mutants by hand, run again in a copy after a
previous session left one hanging in the skill's folder: 28 caught; two only by the suite
hanging, now a test failing past 600 s; three by tests added. 30 runs green.

Task 6, the dependency facts, after three answers of the user. Tests on the captures of a probe
crate made in scratch, watched failing. The first real run: 65 crates in 3 min 23 s. A GitHub
capture minutes later met the rate limit, its message naming the machine's address: the status
kept without the message, the address replaced in the capture, one request a second. 52 mutants
by hand: 47 caught, 4 by tests added, `LC_ALL` left, changing nothing. 30 runs green.

Task 7, `cargo-deny` 0.20.2, installed in session at the user's word, built in 58 s, its entry
in `docs/toolchain.md`. `writing-documentation` asked for a `[docs]` table, added at the user's
yes; its `check_docs.py` gives 9 errors predating this change, left outside the roadmap at the
user's word. cargo-deny's source scopes `unsound` to the direct crates by default: `deny.toml`
sets `all`. `cargo deny check`: ai-ctl exit 0, the duplicates warnings; the `dependency-check`
fixture RUSTSEC-2026-0190, exit 1; a scratch crate's BSD-3-Clause path crate and `file://`
source refused, exit 12. The user validated the file, and its pinned copy in the fixtures, whose
test failed on its assertion in the five cases, then passed, 30 runs green.

Task 8, the guards. The agents' own rule allows only `doctor`, `status` and `build list`, where
phase 0's list let `bench run`, `logs`, `gateway config` and `make up` through: four answers of
the user, the two probe sessions among them, "Oui, les deux (Recommandé)". The capture probe, `claude -p`, Claude Code 2.1.312, `claude-haiku-5-5` at effort
`low`: an agent's hook inputs carry `agent_type` and `agent_id`, the session's none; after a
stop blocked by `decision: block`, the session received only the agent's second answer; a
`nohup … &` server was left outside `claude`'s descendants, `CLAUDE_PID` in its environment.
Five inputs and an `ss` capture kept in `tests/fixtures/`. `guard.py`'s 32 tests failed against
a stub, then passed. Reading the code found four defects, each test watched failing: two
heredocs on one line read in reverse, `time -p` hiding the command, sudo's `-C` taken for a
folder, `-S` read for every wrapper. 100 mutants by hand: 36 missed at first, 98 caught once
cases were added and `ss` captured again with a process name holding a space and a socket on two
descriptors; two change no decision, a rename's source read as an entry of `git status`, and a
call with no command, refused by the failure path. A dead check, the process being `claude`
itself, removed. 30 runs green. The final probe, in a scratch copy where `ai-ctl` and the
Makefile only echo: the session's `./ai-ctl up` passed; the agent's `ai-ctl gateway config`,
`make up` and `git commit` were refused, `git -C <folder outside> init` passed; its first stop,
blocked on `?? left.txt` and the server with its port, led it to clean both and answer again
whole.

Task 9, `tests/repository.rs`: its unit test failed against a stub, then passed; with a raw
U+2028 in `README.md`, the tree's test failed naming `README.md:374`. Eight mutants by hand, all
caught once the exclusion of `docs/reviews/` had its own test. 30 runs green. Task 10: each test
file of the evals joined the checks with its task; every check runs green.

The roadmap skill changed during the session: `progress.py` now writes the overview
`docs/roadmap/README.md` and asks for a description under each README's title. At the user's
word, the four completed roadmaps' READMEs took theirs, and the overview was committed.

---

## Decisions

- **The scope by hunk:** a hunk is a run of changed lines of `git diff -U0`, given as
  `hunk: path:first-last (symbols) +added -removed`, with each item holding it, a function, an
  `impl`, a type or a constant, whole from its attributes and comments, `Type::method` for a
  method; `reading:` counts the lines handed, production and tests apart. A blank line, or
  closing delimiters alone, hold nothing: git's alignment moves a block's closing brace into the
  hunk that adds code at its end. Phase 3's steps and agents read these lines.
- **Five routes:** captures, `tests/fixtures/`, first; code, the Rust files and the files the
  production code embeds (the user, 2026-11-02: "oui validé"), the backends' patches being code
  the security pass must read; dependencies; documentation, `docs/` but its roadmaps and reviews,
  and `README.md`; other. `--files` is gone.
- **The origin is git blame's, `-w`, at the scope's end:** a reindented line keeps its origin, a
  moved or copied line is new, a removed line leaves nothing to cite, with no base every line is
  new.
- **A finding's origin is its first place's,** the defect's site, the later places its
  references, and an omission cites first the new code that calls for it (the user, 2026-11-02:
  "la recommandation"). It amends phase 0's rule, the origin from every place; it commits phase
  3's reviewers to citing the site first.
- **`findings.py` keeps `findings.json` and writes `report.md` from it** through
  `assets/report.md`, by six commands, `new`, `add`, `merge`, `set`, `report` and `tasks`, every
  change checked whole and refused when it breaks a rule, both files written whole or not at
  all, U+2028 and U+2029 escaped. A merge numbers the findings again, refused once a verdict or
  an action is set. The verifier's verdicts are written by `set`; phase 3 settles its product.
- **The answer's format, which phase 3's agents give:** a `FINDINGS: <n>` line counted against
  the blocks; one field a line, `name: value`, an indented line continuing it; the FINDING
  block's fields of the Design, `review-structure` citing its rule in `evidence`; a `PROBE`
  block's `command`, `risk`, `settles` and `finding`; `FILES READ`, with `in full:` and
  `in part:`; `LEFT OUT`. Any fault refuses the whole answer, named with its block and field.
- **The fix roadmap's tasks by `findings.py tasks`, outside the report** (the user,
  2026-11-02: "Commande à part (Recommandé)"), amending phase 0's Design.
- **A finding's matches, the register's entries of other reviews sharing one of its axes, a
  path and a symbol** (the user, 2026-11-02: "Recouvrement (Recommandé)"), computed at `add` and
  shown in the report for the session to compare the titles: a match on the whole fingerprint
  misses a finding citing a reference beside its site.
- **The action `known`, a defect the register holds already** (the user, 2026-11-02: "Action
  `known` (Recommandé)"): a match and a note naming the entry, no verdict, no batch, no second
  entry.
- **The register, `register.json` beside the reviews' folders, kept by `set`:** an entry per
  finding whose action is `register` or `rejected`, its state `rejected`, `kept` for a fix
  `none:`, `open` otherwise (the user, 2026-11-02: "Garder la règle (Recommandé)").
- **`regression.py`:** the recorded mutants of a commit, `HEAD` by default, applied in its
  `git archive` copy under TMPDIR, each test run first without its mutant; six outcomes, exit 0
  only when every mutant is caught; one timeout, 300 s by default, each cargo command in a
  process group of its own, killed whole past it or on an interruption, the copy removed.
- **GitHub asked for the direct crates and for each transitive crate whose last release is more
  than 12 months old** (the user, 2026-11-02: "Release > 12 mois (Recommandé)"); the records
  written to `dependencies.json` in the review's folder ("Dossier de la review (Recommandé)");
  llama-swap's and the backend sources' lag left to phase 3's `review-dependencies` ("Agent, hors
  script (Recommandé)").
- **A failure stops the gathering and writes nothing,** exit 3; GitHub's refusals are facts, a
  403 or a 429 stopping the asking, their message never kept, since it names the address asking
  and the review's folder is committed. One request a second to each host.
- **`deny.toml`** (the user, 2026-11-02: "Minimale (Recommandé)", then "Valider tel quel
  (Recommandé)"): the licences MIT, Apache-2.0, ISC and Unicode-3.0, what the lock needs, a crate
  bringing another failing the check; every advisory an error, `unsound` and `unmaintained` over
  every crate; a yanked release a warning; Linux x86_64 alone; crates.io alone; duplicates
  warnings. The fixtures take a pinned copy beside `claude.md` ("Maintenant, copie figée
  (Recommandé)").
- **The guards' allowlist for ai-ctl** (the user, 2026-11-02: "Liste blanche (Recommandé)"): a
  review agent runs `doctor`, `status`, `build list` and the help and version forms, through
  `ai-ctl` at any path or `cargo run`, and of the Makefile only `help`, `doctor` and `status`. It
  amends phase 0's five refused commands. `cargo run` and `make` are ai-ctl's when the nearest
  `Cargo.toml` names `ai-ctl` or the folder does not resolve.
- **Git read-only on the repository** (the user, 2026-11-02: "Lecture seule sur le dépôt
  (Recommandé)"): there, 36 subcommands that read, `status` to `verify-tag`; elsewhere every
  subcommand, its target resolved from `-C`, `--git-dir`, `--work-tree`, `GIT_DIR`,
  `GIT_WORK_TREE` and a `cd` before it that takes effect, to a folder that exists; a target that
  does not resolve refused, the reason asking for `git -C <absolute path>`, the form phase 3's
  rules give for a repository in scratch.
- **A command is followed through the shell** by a lexer of the standard library: separators,
  pipes, subshells, substitutions, heredocs, `bash -c`, `eval`, `find -exec` and the wrappers.
  One it cannot read is refused when it names ai-ctl, make, git or cargo.
- **A stop blocked once** (the user, 2026-11-02: "Bloquer une fois (Recommandé)"), by the guard's
  own mark, `stop_hook_active` deciding only with no baseline: the baseline, taken at
  `SubagentStart` in `$XDG_RUNTIME_DIR/ai-ctl-review-guard/`, holds the repository's
  `git status`, each file hashed, and the session's processes left behind, those carrying
  `CLAUDE_PID` that are no longer `claude`'s descendants, so that agents running together do not
  block each other. A background task still running blocks too. The reason asks for the whole
  answer again, the session receiving only the last message.
- **The guards act on an `agent_type` starting with `review-`** and refuse on their own failure:
  an input that is no hook's exits 2, a failure on a review agent's call denies it, a stop with
  no baseline is blocked once.
- **The separators' check is a test of the crate, `tests/repository.rs`,** run by `cargo test`
  over `git ls-files`, `docs/reviews/` left out, where a review quotes what it found.

---

## Deviations

- The fixture's review of phase 1 gave its diff +211 −6 lines in `src/` and `tests/`, where git
  counts +210 −6: the 211 held `README.md`'s line. Corrected in patches 05 and 10 and the
  `continued-review` state; that case's baseline runs again under `--reuse` in phase 4.
- `SKILL.md` and six review agents still call `scope.py --files` and read its `file:` lines,
  which this phase removes: phase 3 rewrites both for the scope by hunk.
- `phase-3-skill-and-agents.md`: the task giving the reviewers the structured finding now says
  that `where` names the defect's site first, an omission the new code that calls for it, by
  the decision on the origin's place.
- The fixtures' review states brought to the writer's form (Work Log): patches 05, 10, 15 and 17
  and the `continued-review` state; `roadmap-close`'s baseline runs again under `--reuse` in
  phase 4, as `continued-review`'s already does.
- `assets/report.md` rewritten in this phase as the writer's frame, a part of phase 3's task:
  `phase-3-skill-and-agents.md`'s task now brings the report to the dependency axis alone.
  `SKILL.md`'s step 5 cites `scripts/findings.py`; its later steps still write the report by
  hand until phase 3 rewrites them.
- The answer parser is tested on answers written to the format it defines, no agent giving it
  before phase 3: phase 4's runs hand `add` the first real answers.
- The fixtures' findings given `matches`, empty in each: patches 05, 10, 15 and 17 and the
  `continued-review` state regenerated, whose baselines already run again in phase 4.
- `evals.json`'s `roadmap-close` case: its register's entry now "on the same site and axis",
  shown as a match by `findings.py add`, where it said "the same fingerprint", which the
  finding citing `stage` beside `stage_from` does not have.
- `SKILL.md`'s step 4 cites `scripts/regression.py` at a roadmap's close, ahead of phase 3, which
  writes the closing pass. The regression run is tested on a crate made in scratch, the
  `gateway-releases` fixture's build needing ai-ctl's dependencies; the fixture's M1 to M3 run in
  phase 4's `roadmap-close` case.
- `phase-3-skill-and-agents.md`: the dependency axis's task now names `dependencies.py`'s records
  and gives the agent llama-swap's and the backend sources' lag, by the decision leaving it out
  of the script.
- `phase-4-evaluation.md`: a constraint added, GitHub's 60 requests an hour shared by the
  `dependency-check` case's runs, `dependencies.py` spending about 20 on ai-ctl's crates.
- `SKILL.md`'s step 5 cites `scripts/dependencies.py`, as the static audit asks (R3), ahead of
  phase 3, which writes the dependency axis.
- A task added to this phase: the evals' fixtures take `deny.toml`, which phase 1's report
  expected once this phase wrote it and no task held.
- `check_docs.py`'s 9 errors on `docs/`, folders without a README, left outside the roadmap at
  the user's word: no task of this roadmap holds them.
- `SKILL.md`'s step 6 cites `scripts/guard.py`, as the static audit asks (R3), ahead of phase 3.
- `phase-3-skill-and-agents.md`: the shared rules' task now names what the guard refuses, git in
  scratch as `git -C <absolute path>`, and the whole answer given again after a blocked stop.
- `phase-4-evaluation.md`: a constraint added, the run folders needing the project's
  `.claude/settings.json` for the v2's agents to run guarded, the fixtures carrying no
  `.claude/`.

---

## Assessment

The phase delivered the scripts phase 0 designed, each test-first, its mutants applied by
hand: the scope by hunk with each line's origin, the report writer with its debt register, the
regression run, the dependency facts, and the review agents' guard, wired in the project's
settings; with `cargo-deny` under a `deny.toml` and a check against raw separators. Three of
phase 0's rules were amended on evidence: a finding's origin from its first place, a match by
an axis, a path and a symbol rather than by the whole fingerprint, and an allowlist of ai-ctl's
commands in place of five refused ones. The probes found what no document said: a stop the guard
blocks hands the session only the agent's last answer, and a process an agent leaves behind is
no longer `claude`'s descendant.

What phase 3 needs first: `SKILL.md` and six agents still call `scope.py --files`, which is gone;
`SKILL.md` cites four scripts ahead of the steps that will use them; the agents must answer in
the format `findings.py add` parses, cite a defect's site first, and know what the guard refuses
and that a blocked stop asks for their whole answer again.
