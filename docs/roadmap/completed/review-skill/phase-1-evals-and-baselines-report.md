# Phase 1 Report: Evals and Baselines

**Phase:** [phase-1-evals-and-baselines.md](phase-1-evals-and-baselines.md)
**Start Commit:** `docs: close phase 0 of the review-skill roadmap` of 2026-10-13

---

## Work Log

### 2026-10-13

Phase opened at the end of phase 0's closure, after reading phase 0's file and report, in
the session that closed it. No restructuring was pending. The phase opens blocked: its paid
runs wait for authoring-skills to evaluate a skill that starts agents, proposed in
the agent setup's `notes/2026-10-13-eval-runs-with-agents.md`. The user ended the session
here, to resume in a new session once authoring-skills is changed; no work on the phase's
tasks was done.

Resumed in a new session, the user reporting authoring-skills changed. Read this phase's
file, phase 0's file and report. Checked the change: the agent setup decided the proposal on
2026-10-13 (`docs/decisions/2026-10-13-eval-runs-with-agents.md`, commit `2f242e3`) with
its S1: an `agents` key in `evals/evals.json`, the agents snapshotted per arm, the Agent
tool allowed in both arms for a skill with agents, the agents' runs copied, rendered under
a run's `agents/` for the grader and counted, their output settled from the json result's
`modelUsage`; S2 kept for later. The installed copy under `~/.claude/skills/` is identical
to the repository's, and `scripts/conventions.py skills` reports `status: ok`.

Found a gap that phase 0's proposal missed. `workspace.py` refuses an `agents` path that
does not exist (`agent_problems`), and `run.py` allows the Agent tool only when the
iteration lists agents (`harness.disallowed`). The skill's agents are written in phase 2,
after the baselines that set their number, so this phase's evals can list none, and its
baseline runs would still be denied the Agent tool: the condition this phase's
Dependencies set out to avoid. The proposal's S1 described the arms of a comparison, never
the first observation of a new skill made before its agents exist. Phase 3's comparison is
covered: its iteration lists the agents, so its arm without the skill has the Agent tool,
and the agents enter each case's digest, so `--reuse` cannot take a baseline of this phase
run with Agent denied. Also noted for the announcement: `workspace.py` defaults to
`claude-sonnet-5-5`, where phase 0's budget rule asks the user's real model and effort,
Opus 5.5 at `xhigh`, and a run's ceiling defaults to $5. Put the choice to the user.

The user chose a new proposal to the agent setup over baselines run with Agent denied. Read
the lines it cites again (`workspace.py:191-192`, `:344`, `:396-397`, `:433`, `:439`;
`run.py:141`, `:188`, `:343`; `harness.py:102-105`): in an iteration prepared with
`--baseline-only`, the only one possible before `SKILL.md` exists, the `agents` key lifts
the denial of Agent, enters each case's digest and leaves its paths out of `base.tar`,
while the arm without the skill never receives the agents. Wrote
the agent setup's `notes/2026-10-13-agent-tool-before-agents.md`: the gap, why it matters,
four solutions with T1 recommended (a skill-level key saying that the skill starts agents,
independent of their files), and two questions. Nothing was committed there. Updated this
phase's `**Blocked By:**` and Dependencies, and the README's `**Blocked By:**`, to the new
note.

Turned to the cases, which do not wait. Read what a run receives (`workspace.py:334-355`,
`run.py:206-226`, `run.py:73-76`). A `repository` case's copy holds the files git tracks or
does not ignore, plus `.agent-conventions.toml`, then is made a new git repository with one
commit: it has no history, so no Start Commit can scope a review, and it has no `CLAUDE.md`,
which `.gitignore` ignores, so a run would lack the conventions every real session of
ai-ctl loads. A `fixture` case keeps the `.git` its `evals/fixtures.py` builds. Every run's
prompt starts with a preamble saying that nobody can answer a question, and to take the most
reasonable assumption: a skill that waits for the user's yes on a cost meets it there.
Facts for the v2 case: since `docs: close the v2 rebuild roadmap` of 2026-10-12 (the v2 roadmap's closure), only `.gitignore` and
`docs/roadmap/` changed, so a fixture at that commit holds the v2 code exactly and none of
this roadmap's documents, `analysis.md` included, which would hand a baseline the skill's
design; `src/` holds 24,074 lines of Rust; the v2 phase reports' Start Commits are
`1a49576`, absent from this repository's history (the v1 archive's), then `chore: restart llm-ctl v2 on a clean base` of 2026-10-09 (the
root commit), `feat(doctor): add doctor, init and the inventory` of 2026-10-09, `docs: close phase 2` of 2026-10-09 and `docs: close phase 3` of 2026-10-11. The task "Build the fixtures … as
decided in phase 0" cites a decision phase 0 did not take: none of its decisions names a
fixture. Put the cases' design to the user.

The user chose the fixture at `docs: close the v2 rebuild roadmap` of 2026-10-12, two cases (the v2 roadmap, and a session with
planted defects) and prompts that carry a tier and their agreement to the cost. Tested the
fetch: `git fetch` of the full hash into a new repository gives 35 commits, none after
`docs: close the v2 rebuild roadmap` of 2026-10-12, and a reflog holding the reset alone, where a clone would keep the later commits
reachable from its reflog; `.git/FETCH_HEAD` names the source path and is removed.

While designing the session case, read `run.py`'s `collect` (`run.py:234-247`): a run's
changes are the diff from the commit its folder starts at, so a session left uncommitted in
the fixture would count as the run's own work. The session is therefore a commit on top of
`docs: close the v2 rebuild roadmap` of 2026-10-12, its date and author fixed, its author taken from the pinned commit; the prompt
names it as the last commit. This differs from what was put to the user ("changes not yet
committed").

`build prune` exists already, so the session adds its counterpart for llama-swap releases,
`ai-ctl gateway prune --keep N`, across every layer: the command line, three methods on the
`GatewayStore` port, `releases_to_prune` in the domain, a `prune` use case, the adapter, the
rendering, the fakes, a test per layer and a README line. Eleven defects are planted across
the nine axes, each where a session might leave it: a text sort of `v99` and `v263`, an
unquoted `sh -c "rm -rf …"`, its result ignored, an `.unwrap()` on a file name, presentation
importing `infrastructure::paths`, the selection of `build::to_prune` repeated, a function
written for later, a `clean` alias kept for llm-ctl's habit, a French comment and `?`
without context, tests that miss the ordering and the adapter, and `size` reading whole
files. Applied it in a scratch clone at `docs: close the v2 rebuild roadmap` of 2026-10-12: `cargo fmt --check` clean,
`cargo clippy --all-targets -- -D warnings` clean, `cargo test` green (495, 24 and 2 tests,
and the `gateway_lifecycle` harness), as a session that ran its checks would leave them. A
throwaway test, reverted, confirmed the two worst: with `v99`, `v100` and `v263` installed,
`prune --keep 1` removes `v100` and keeps `v99`; under a data folder `My Data`, it removes
the sibling folder `My`, and leaves `v100` in place while reporting it removed.

Wrote the evals folder: `evals/fixtures.py` (fetch, pinned `CLAUDE.md` and
`.agent-conventions.toml`, the session's patch committed for `post-session-review`, the
user's git configuration kept out), `evals/fixtures/gateway-prune.patch`,
`evals/fixtures/claude.md` (named so that no session loads it as a `CLAUDE.md`),
`evals/fixtures/agent-conventions.toml`, and `evals/evals.json` with the two cases, their
expected output and a `pending` note; the second case's expected output lists the eleven
planted defects with their lines in the fixture. Built both fixtures: a clean working tree,
history ending at `docs: close the v2 rebuild roadmap` of 2026-10-12 or at the session's commit, whose diff equals the scratch
clone's, and no path of this machine's repositories in `CLAUDE.md`. `workspace.validate`,
called on the file, reports no problem. No iteration was prepared: one prepared now would
deny the Agent tool, and the change asked for enters each case's digest.
`scripts/audit.py` stops on a traceback for a skill holding its evals alone; added to the
note in the agent setup as a separate point.

The user asked to record the progress and commit, and to resume in a fresh session. Two
points put to them stay unanswered: the session case as a commit rather than uncommitted
changes, and the wording of the two prompts. Where the next session starts: check whether
the agent setup decided `notes/2026-10-13-agent-tool-before-agents.md`; once authoring-skills
gives the Agent tool to a skill whose agents are not written yet, add its key to
`evals.json`, prepare the first iteration with `--baseline-only --model claude-opus-5-5`
and a ceiling per run chosen with the user (the default $5 is below a review of the whole v2
code), then announce the runs' count and cost with `run.py`.

Resumed in a new session, the user reporting authoring-skills fixed. Checked the change:
the agent setup decided the second proposal the same day (commit `fe4f87e`,
`docs/decisions/2026-10-13-eval-runs-with-agents.md`, § Revised The Same Day). `evals.json`
takes `starts_agents: true`, which gives every run the Agent tool before the agents' files
exist; a case's digest records the Agent tool rather than the agents' paths, so the runs of
this phase stand for phase 3's baseline once the agents are listed; `audit.py` on a skill
holding its evals alone says it waits for `SKILL.md` and exits 0. The installed copy under
`~/.claude/skills/` matches the repository's, its `evals/` folder apart, which is not
installed. Added `starts_agents: true` to `evals.json` and reworded both `pending` notes;
`workspace.validate` reports no problem (it takes the skill's folder, the evals folder and
the root besides the data; `workspace.py` has no option that validates without preparing),
and `audit.py` exits 0 on its new message.

Put the three open points to the user. They kept the session case as a commit and the two
prompts as written. On the ceiling per run, they raised a constraint no phase knew: their
plan caps spend at about $20 per usage window, which every run of a batch shares with the
session that starts it. A single run cannot be split; the runs can, `run.py --case` starting
one case. Prepared `iteration-1` with `--baseline-only --runs 1 --model claude-opus-5-5
--effort xhigh --budget 18`: both fixtures built, clean, history ending at `docs: close the v2 rebuild roadmap` of 2026-10-12 or at
the session's commit `01b2813`; `iteration.json` records `starts_agents: true`. `run.py`
announces 2 runs at $36.00, each at its ceiling since no run at that model and effort is
complete, and `--case post-session-review` alone 1 run at $18.00. Nothing was started.

The user asked whether the $18 ceiling bridles the work. It does not change how a run works:
the preamble (`run.py:74-76`) says nothing of a budget, and `--max-budget-usd` stops the
session outright once its spend, subagents included, reaches the ceiling, keeping what was
done and losing the rest. The post-session case, estimated at $3 to $10, should stay below
it; the v2 case, estimated at $10 to $30, may be cut before its report, keeping its scoping,
plan and delegation and losing verification, merging, report and fix plan. A higher ceiling
would not help: the plan's window would stop the run anyway, and a headless run stopped by
it is lost, where an interactive session pauses and resumes. Narrowing the v2 case to one
phase of the v2 roadmap was offered as the way to a complete observation, at the cost of the
whole-repository scoping; the user has not chosen it.

The user asked to record the session's progress and state, and to resume the roadmap from a
new session in a fresh usage window for each run. Where the next session starts: in a fresh
window, announce `run.py .eval-runs/skills/reviewing-delivered-code/iteration-1 --case
post-session-review` (1 run, $18.00 at most, Opus 5.5 `xhigh`), start it on the user's yes,
then read its transcript and output in full. The v2 case follows in another fresh window, the
same way with `--case review-v2-roadmap`; if it is cut at its ceiling, record it as a partial
observation. `iteration-1` lives in `.eval-runs/`, which git ignores; preparing it again
costs nothing.

The user then had `CLAUDE.md` versioned: they had removed `/CLAUDE.md` from `.gitignore`
before the session, and asked for both to be committed. The fixtures are unaffected: they
fetch `docs: close the v2 rebuild roadmap` of 2026-10-12, where `CLAUDE.md` was not tracked, and add their own pinned copy.

Resumed in a new session, in a fresh usage window, the user asking to start run 1. Read this
phase's file and report. authoring-skills has not changed since `fe4f87e`: the agent setup's
later commits touch other domains, and the installed copy matches the repository's, compiled
files apart. `iteration-1` is as prepared (`starts_agents: true`, Opus 5.5, `xhigh`, $18 per
run), and `run.py --case post-session-review` announced 1 run at $18.00, the figures the user
had accepted. Started it on the user's go, at 11:15 UTC.

The user then asked to stop it, calling the start their own mistake. Sent SIGTERM to the
`claude -p` session alone, so that `run.py` recorded the end: `run-1` is `stopped` (`no
result, exit code 143`) after 2 min 20 s, 12 calls and $0.68, with its partial transcript;
`run.py --case post-session-review` will start it again from an empty folder. Before the
stop, the run had read the pinned `CLAUDE.md`'s pointer to the v1 archive and cloned the
bundle into `/tmp/v1-review-qjlE`, outside its copy, to check whether v1 had a `clean`
command: the trace of the alias planted as defect 8. The guard allowed it, and
`repository_paths` is empty, since the bundle lies outside ai-ctl.

The user asked to start it again, unchanged. `run.py` announced the same line, 1 run at
$18.00, and the run started again at 11:20 UTC from an emptied folder. The window now holds
the stopped run's $0.68 and this session's own spend beside the new run's ceiling, near the
plan's $20; the case was estimated at $3 to $10.

The run completed in 4 min 40 s for $1.29 (24 calls, all Opus 5.5 at `xhigh`, the json
result's cost equal to the transcripts'), no refusal, no agent started, `repository_paths`
empty and `changes.json` empty. Read its `response.md` and `transcript.md` in full; the
rendering shortens long outputs, which hid only the throwaway tests' code, whose results it
shows. What it did: read the commit's diff, ran the three checks (fmt and clippy clean,
`cargo test` 495, 24 and 2 tests, reported as 521), read the code around the diff and
`build prune` beside it, cloned the v1 bundle into `/tmp` to check the `clean` alias's
claim, read the v2 phase 3 report, then wrote six throwaway tests in a `git archive` copy
under `/tmp`, ran them, deleted its scratch folders and checked the work tree clean. Its
answer, in French, ranks thirteen findings as blocking, important and minor, each with its
`file:line`, adds process findings and a section of what it checked and found sound, and
ends with its account.

Against the case's answer key, ten of the eleven planted defects are found, the tests' one in
part, and the refactoring one missed; five are reproduced by a test, and the alias's claim
checked against the v1 history: the text sort (`gateway_store.rs:193`, `v99` kept by `prune --keep 1`), the
unquoted `sh -c` (the sibling `My` removed, the release left), the ignored result (a
read-only root reported as `removed: [v262], freed: 1000`), the `.unwrap()` (a panic on
`caf\xE9`), presentation importing `infrastructure::paths`, the dead `newest`, the `clean`
alias (absent from both v1 branches, so its comment is false), the French comment, `?`
without context, the untested adapter and `is_ok()`-only use case test, and `size` reading
the 43 MB binary. Beyond the key, it found that `releases()` takes any name `Release::parse`
reads, so a stray `263` duplicates `v263` and an empty `v264` counts as the newest
(reproduced: the real `v263` removed, then an error); the lock's message saying "installing
llama-swap"; "1 releases kept"; the commit reversing the v2 phase 3 report's "no prune, since
a release is one 43 MB file" (`phase-3-gateway-report.md:692`) without a word; and the
commit lacking a body and a `Co-Authored-By` trailer. It weighed the release a running
gateway uses and judged it not a bug, citing `gateway_process.rs:88`.

The failures, quoted:

- **No report file.** `changes.json` lists nothing; the review exists only as the session's
  answer. The run's assumption: "j'ai supposé que vous vouliez **une revue seulement** :
  aucune modification, aucun commit". Phase 0's decision 10 puts the report in
  `docs/reviews/delivered-code/<date>-<scope>.md`.
- **No fix plan in batches.** The answer ends with "Les points 1 à 8 se corrigent en
  s'alignant sur `build prune`, avec des tests sur répertoire temporaire. Je peux le faire
  dans une prochaine session.", one sentence where phase 0's decision 8 asks a grouped plan
  for the user to approve.
- **The duplicated selection is not named.** Planted defect 6, `releases_to_prune` repeating
  `domain::build::to_prune`: the run compares the two only to say the sort belongs in
  `releases_to_prune` "comme le fait `domain::build::to_prune`", which copies the pattern
  again instead of sharing it.
- **A gap of the domain test missed.** Its finding 8 names the use case's `is_ok()` and the
  untested adapter, not the domain test built on three-digit releases alone, which is why the
  sort bug passed.
- **Two causes merged into one finding.** Its finding 6 joins the errors without context and
  `size` reading the whole binary ("Par ailleurs, `size` lit tout le binaire (43 MB) en
  mémoire"): error handling and performance, two causes and two fixes, under one number.
- **No axis named, no coverage stated.** The findings carry no axis, and nothing says what was
  not reviewed; whether security or performance were looked at beyond the planted defects
  cannot be told from the answer.

No delegation, by choice: "Je n'ai pas lancé de sous-agents, puisque chaque problème était
vérifiable directement." Not counted as a failure: on a 208-line commit, one session found
ten of eleven defects and reproduced five, for $1.29, against $3 to $10 estimated. It
bears on phase 2's tiers: an agent per axis has to earn its cost against that on a small
scope, and the v2 case is where a single session should show its limit.

What it did right, for the skill not to repeat: the checks first; the sibling feature as the
reference for each fix (`FsBuildStore::remove`, `build::to_prune`, `InventoryStore::location`);
each suspected bug reproduced by a throwaway test outside the repository, the repository left
clean and the scratch folders removed; a comment's claim checked against the history it cites
(the v1 bundle); the recorded decisions read (the v2 phase 3 report); a section of what it
checked and found sound; the assumption it took stated in its account.

The first, stopped run left its clone of the v1 bundle in `/tmp/v1-review-qjlE`, since it
was stopped before cleaning up. The v2 case waits for another fresh window: this one holds
the two runs' $1.97 and this session's spend, and the case's $18 ceiling would pass it.

The user had the stopped run's leftover deleted: `/tmp/v1-review-qjlE`, a clone of the v1
bundle, which stays in its archive; the run's folder itself was emptied when it started
again. No scratch folder of either run remains in `/tmp`. The user chose to commit at the
end of the step rather than now, so this report's entries of the session stay uncommitted,
and the run's files live in `.eval-runs/`, which git ignores. Where the next session starts:
in a fresh window, announce `run.py .eval-runs/skills/reviewing-delivered-code/iteration-1
--case review-v2-roadmap` (1 run, $18.00 at most, Opus 5.5 `xhigh`), start it on the user's
yes, then read its transcript and output in full, as for the first, and record its failures
and what it did right; if it is cut at its ceiling, record it as a partial observation. Tasks
4 and 5 are ticked once it has run; tasks 7 to 9 follow on both runs. Task 6, the built-in
`/code-review` on the v2 code, is paid too and needs its own window.

Resumed in a new session, in a fresh usage window, the user asking to start the second run.
Read this phase's file and report. authoring-skills has not changed since `fe4f87e`, and the
installed copy matches the repository's. `iteration-1` is as prepared, the v2 case's `run-1`
folder empty. `run.py --case review-v2-roadmap` now announces 1 run estimated at $1.29: the
mean of the post-session run, the only complete one at that model and effort, which says
nothing of a review of the whole v2 code, still estimated at $10 to $30; the ceiling stays
the $18 the user accepted. Started it on the user's go, at 11:49 UTC.

The run completed in 8 min 02 s for $4.51 (53 calls, all Opus 5.5 at `xhigh`, the json
result's cost equal to the transcripts'), far below its ceiling: no refusal, no agent
started, `repository_paths` and `changes.json` empty. Read its `response.md` and
`transcript.md` in full, and listed from `transcript.jsonl` the files its calls opened. What
it did: surveyed the layout and `.agent-conventions.toml`, read the v2 roadmap's README and
summary, `docs/architecture.md` and decision 0001, started the three checks in the
background (fmt and clippy clean, `cargo test` 492, 23 and 2 tests, reported as 517), counted
each file's lines above its `#[cfg(test)]`, then read the production code layer by layer up
to that line. Suspecting the comparison of `/proc/<pid>/exe` with the recorded path, it
confirmed with a `sleep` copied behind a symlink that the kernel gives the resolved path,
then reproduced the bug through the project's own integration test, `XDG_DATA_HOME` pointed
at a symlink to `~/.local/share`: "`status` reports no gateway". It then checked that this
machine's data path holds no symlink, grepped for presentation importing infrastructure and
for `.unwrap()` in production code, checked the work tree clean and removed
`/tmp/review-scripts`. Its answer, in French, gives four findings ranked by severity, each
with where, cause, consequence and a proposed fix: a data path behind a symlink makes a
running gateway look like a reused PID, so `status` and `down` drop its record and leave
llama-swap on the GPU (`gateway_process.rs:90`, `build_store.rs:290`, reproduced); `build
prune` and `build --force` ignore the running gateway's configuration (`build.rs:333`); `bench
run` prints the run before saving it, so a failed write to stdout loses it
(`presentation/bench.rs:59-63`); `doctor`'s remedy for a current release without its binary
ends at `publish`'s refusal (`doctor.rs:156-160`, `gateway_store.rs:148`). Read the code behind
the last two: both hold, though `save`'s context ("the run is printed above, but cannot be
saved") shows the order was chosen, and `publish`'s refusal tells the user what to remove.
The second was not checked here.

The case has no answer key; the failures are read against its expected output and phase 0's
decisions. The failures, quoted:

- **No report file.** "Je n'ai rien modifié dans le dépôt." `changes.json` lists nothing, as
  in the first run.
- **The scope set from the history, not from the recorded Start Commits.** "j'ai considéré que
  « le code de la roadmap v2 » désignait tout le crate, puisque l'historique entier,
  d'`chore: restart llm-ctl v2 on a clean base` à `HEAD`, est la roadmap v2." The conclusion matches the expected one, but no
  phase report was opened, only grepped, and the scope it reviewed is "environ 15 000 lignes
  hors tests": the tests were taken out of the scope without a word on why.
- **The tests not reviewed.** Every file was read up to its `#[cfg(test)]` line;
  `tests/cli.rs`, `tests/gateway_config.rs` and `src/application/fakes.rs` were never opened.
  No finding concerns the tests, and the coverage it states names only the presentation "que
  survolé".
- **A claim of the account that does not hold.** "j'ai lu en entier les couches domaine,
  application et infrastructure": `src/domain/diagnostics.rs` (133 lines of production code)
  was never opened, nor `src/presentation/doctor.rs`.
- **No axis named, most axes absent.** Architecture and conventions are checked by two greps
  (layering, `.unwrap()`); nothing on dead or legacy code, refactoring, performance, or
  security beyond praising "L'échappement YAML et shell". Whether they were looked at cannot
  be told.
- **The tier ignored.** The prompt's "Effort high, je valide le coût d'avance" gets no answer:
  "Je n'ai pas utilisé de sous-agents", $4.51 spent on 24,000 lines, the presentation
  skimmed. A single session on the whole repository shows its limit as coverage left out,
  not as a run cut at its ceiling.
- **No verification by another pass.** The first finding is reproduced by the session that
  found it; the other three rest on its reading alone.
- **`cargo-audit` and `cargo-mutants` not mentioned**, where the expected output asks to say
  they are not installed.
- **No fix plan in batches.** Each finding has its "Correctif proposé"; nothing groups them
  for the user to approve, and the answer ends with "Je n'ai trouvé rien d'autre qui mérite
  d'être signalé."
- **Scratch left outside the repository.** Three `mktemp -d` folders from the symlink
  experiment remain in `/tmp` (`tmp.3tactV1FVZ`, `tmp.HRu8j58Iv3`, `tmp.WAaziDGQpM`, each a
  copy of `sleep` and a symlink); the run removed `/tmp/review-scripts` alone, and its account
  says nothing of them.

What it did right, for the skill not to repeat, beyond the first run's list: the checks
started in the background while it read; the reading planned from a count of production
lines per file; the architecture and the decision on GPU arbitration read before the code; a
suspected bug reproduced through the project's own integration test with one environment
variable, rather than new test code; whether the user's machine is exposed checked; the rules
of `CLAUDE.md` that a grep can check checked by grep; the scope's assumption and what was
only skimmed stated. Reproducing the first finding started the installed llama-swap twice,
from the integration test that `cargo test` runs anyway, with its state in a scratch folder;
`pgrep` found none left. Not counted as a failure; phase 2 says whether a reproduction may
start the stack's binaries, which the expected output rules out for `ai-ctl up` and the
like.

Tasks 4, 5 and 7 ticked: both baselines have run and are read in full, their failures
quoted. Left: task 6, the built-in `/code-review` on the v2 code, paid, in its own window;
task 8, the assertions, written from the two runs' failures; task 9, what both baselines did
right, gathered from the two lists above.

The user had the three scratch folders deleted, asked to commit the two runs' record, and to
go on with tasks 8 and 9 in this session.

Read authoring-skills' `references/eval-files.md` and step 7 of `references/create-and-edit.md`
before writing. Read the first run's answer again for what the second showed: it never
mentions the tier, `cargo audit` or `cargo mutants` either, a failure the first reading had not
listed. Wrote the assertions, twelve per case, each opening with the name of what it checks,
the quoted failures under `failures_without_skill` (F1 to F10 for the v2 case, F1 to F7 for
the post-session case, each dated), and `review` items for what only the user can judge: the
order of the findings, the batches of the fix plan, and for the v2 case, which has no answer
key, whether each finding holds against the code it cites. Each assertion traces to a failure
or to a requirement the user stated in phase 0's intent:

| Assertion | v2 case | Post-session case |
|---|---|---|
| Scope | F2 | — |
| Test code read | F3 | — |
| Planted defects | — | F3, F4 |
| One cause per finding | — | F5 |
| Report file | F1 | F1 |
| Checks | F8 | F7 |
| Tier | F6 | F7 |
| Axes | F5 | F6 |
| Coverage | F2, F3, F5 | F6 |
| Verification | F7 | the intent: "verified by an agent other than the one that found it" |
| Fix plan | F9 | F2 |
| Claims hold | F4 | F4 of the v2 case |
| No scratch left | F10 | F10 of the v2 case |
| No stack command | the intent: "never `up`, `build` or `gateway install`" | the same |

Changed both expected outputs to match: `cargo audit` and `cargo mutants` are run or the
report says why, since phase 2 installs them (phase 0, decisions 12 and 13) and "not
installed" would be false by phase 3; the tier, the axis of each finding, the verification by
another agent and the coverage section, which the post-session case's text lacked, are named
in both. Neither change enters a case's digest, so phase 3 can still reuse these runs.
`workspace.validate` reports no problem; `audit.py` says it waits for `SKILL.md` and exits 0.

What both baselines did right, gathered for task 9, is recorded under Decisions, since phase
2 builds on it. Tasks 8 and 9 ticked. Left: task 6, the built-in `/code-review` on the v2
code, paid, in its own usage window; the phase closes after it.

The user had the assertions committed (`test(review-skill): write the assertions from the baselines' failures`), and asked for a note to the agent setup on
`run.py`'s estimate, as a first feedback from real use for its roadmap `token-usage`, in
progress. Read `run.py`'s `estimate()` and the roadmap's Phase 2 before writing
the agent setup's `notes/2026-10-13-run-estimate-per-case.md`: every complete run of the skill at
the model and effort is pooled, whatever its case or configuration (`run.py:134-159`), so
phase 3's arm with the skill would be announced at the baselines' mean, $2.90, against an
$18 ceiling. It recommends an estimate per case and configuration, with the ceiling printed
beside it, and adds for `token-usage` the usage window as the unit the user plans by, and
this session's hook counted at zero runs, a defect its Phase 2 already lists. Nothing was
committed in the agent setup, whose `notes/` git ignores.

Prepared task 6 with the user. `/code-review` runs as the first message of a new session,
before anything of this roadmap is read: a session that resumes the roadmap first reads this
report, which holds the baselines' findings, and the comparison would be lost. It does not run
in ai-ctl either, whose `CLAUDE.md` sends a session to the open phase's report. It runs in a
copy of the v2 case's fixture, the environment the baseline had: extracted `fixture.tar` into
`/tmp/v2-code-review`, history ending at `docs: close the v2 rebuild roadmap` of 2026-10-12 (35 commits), work tree clean, no `.claude/`,
`docs/roadmap/` holding `completed/` alone, the pinned `CLAUDE.md` saying no roadmap is open,
`src/` and `tests/` identical to ai-ctl's. `/tmp` is a tmpfs: a reboot empties it, and the same
command on `fixture.tar` rebuilds the folder.

Where the next sessions start: in a fresh usage window, a Claude Code session opened in
`/tmp/v2-code-review`, its first message `/code-review high src/` (with `tests/` if the
command takes several paths; no `--fix`, no `ultra`). Then a session in ai-ctl resumes the
roadmap: find that session's transcript,
read it in full, record what `/code-review` found and missed against the two baselines and the
case's expected output, tick task 6, and close the phase.

Resumed in a new session, the user reporting the `/code-review` run done in `/tmp/v2-code-review`.
Read this phase's file and report, then the run: session `58a1ca7e`, its own transcript holding
the command and its output alone, the review done by one `general-purpose` subagent
(`agent-a7ace141a8bc087c7`), read in full from its rendering, thinking redacted. The command
was typed bare, `/code-review`, not `/code-review high src/`. It therefore took the level typed
last, `xhigh` (its recipe: "10 inline angles → dedup (no verify) → sweep → ≤15 findings", the
angles run in sequence in one context, subagents forbidden), and the default scope: no upstream,
a clean tree, so `git diff HEAD~1`, the docs-only commit `docs: close the v2 rebuild roadmap` of 2026-10-12 that closed the v2 roadmap.
It ran 8 min 53 s, 67 calls, all Opus 5.5 at `xhigh`. Its input side is known, $2.04 (6.27M
cache reads, 157K cache writes); the output of 65 calls lacks its final record, which
`usage.py` estimates at 5,400 tokens each, $9.11 in all, a figure likely high for calls that
are mostly short commands. The session is still open; its `cost-state` record, written when it
ends, will give the real cost.

What it did: read the diff, the roadmap skill's closing reference and summary template, the
v2 README and the top-level README, ran `cargo test` in the background into its own scratch
target folder (517 tests, 492, 23 and 2), `cargo fmt --check` and clippy (clean), the roadmap's
`progress.py --check` and `check_links.py` (no problem), then checked each claim of the new
`summary.md` against the phase reports, `docs/toolchain.md`, `docs/stack.md`, `docs/notes/vllm.md`
and the code it names (`lifecycle.rs`, `podman.rs`, `gateway_config.rs`, `bench.rs`,
`chat_api.rs`, `doctor.rs`, `tests/gateway_lifecycle.rs`). It also listed
`~/.local/share/ai-ctl/llama-swap/`, outside the fixture, to see whether the integration test
had run. Its answer gives nine findings on `summary.md`, ranked, each with a failure scenario:
the "never run for real" list missing `up`'s image pull (`summary.md:127`); the open item on
runs before `fix(bench): set the slot's context aside before each fresh request` of 2026-10-12 not naming the baseline's two runs (`:130`); "the prefill criterion now
has its figures (`docs/notes/vllm.md`)", where the note holds none (`:134`); "closed phase 3's
last criterion", where `phase-3-gateway.md:121` keeps it unticked (`:75`); a driver change
claimed where `docs/toolchain.md` says "The driver was not touched" (`:89`); OpenCode said to
point at v1's gateway when it was not installed (`:18`); "three `doctor` checks" against the
changelog's two checks and a remedy (`:59`); OpenCode blamed for the wire format every client
sends (`:104`); a 104-column line left by a reflow (`:15`). Not checked here beyond what its
transcript shows; two hold in ai-ctl, where `summary.md` is unchanged since `docs: close the v2 rebuild roadmap` of 2026-10-12: the
unticked criterion at `phase-3-gateway.md:121` and the toolchain's sentence at
`docs/toolchain.md:170`.

What it tells, and what it does not. On the v2 code, the run gives nothing: no line of `src/`
or `tests/` was in its scope, so what the built-in tool finds and misses among the bugs, which
phase 0's decision on bugs asked for, is still unknown. It does show three facts for phase 2:
the built-in tool scopes itself by a diff and, asked bare, reviews the last commit whatever it
holds, here prose; at `xhigh`, it runs one agent and forbids it subagents; and on prose it
checks each claim against its source, the behaviour the v2 baseline lacked for its own account
(F4). The task is not ticked.

Put the choice to the user: run it again on `src/`, keep this run and close, or move the run on
the code to phase 3. They chose to run it again, since phase 2 builds the bugs agent from the
built-in tool's gaps. The fixture is unchanged: a clean tree, 35 commits; the run built into its
session's scratch folder (1.1 GB), outside the copy.
Where the next sessions start: close the open session in `/tmp/v2-code-review`, so that its
`cost-state` record gives this run's real cost; in a fresh usage window, a new session in the
same folder, its first message `/code-review high src/`, the level and the target typed, since
a bare command takes the level typed last and the last commit. Then a session in ai-ctl resumes
the roadmap: read both runs' transcripts, the new one in full, record what `/code-review` found
and missed on the code against the v2 baseline and the case's expected output, tick task 6 and
close the phase.

The user reported the second run done. Session `8efc056b`, its first message
`/code-review high src/`, the review again done by one `general-purpose` subagent
(`agent-afb49ffcd6e7a410d`), read in full from its rendering, thinking redacted. Neither
session was closed, so neither has its `cost-state` record yet. Its recipe at `high`: "8 inline
angles → dedup (no verify) → ≤10 findings", three correctness angles (line by line, removed
behaviour, cross-file callers), reuse, simplification, efficiency, altitude and `CLAUDE.md`
conventions, run in sequence in one context, subagents forbidden, "Err on the side of
surfacing", at most ten findings kept, the most severe. It ran 6 min 51 s, 43 calls, all Opus
5.5 at `high`: input side $2.54 (6.68M cache reads, 241K cache writes), $7.13 with the output of
42 calls estimated by `usage.py`.

What it did: listed `src/` by size, found each file's `#[cfg(test)]` line, then read the
production code of every file up to it, layer by layer; the four files it never opened are
module declarations (`lib.rs`, `domain/mod.rs`, `infrastructure/mod.rs`,
`presentation/completions.rs`, 64 lines). It grepped `.unwrap()` above each test module and
`infrastructure` imports outside it, read the head of `application/fakes.rs` and
`application/mod.rs` to see that the fakes are test-only, and grepped the line numbers of its
findings. It ran no check, reproduced nothing, wrote no file, and read no test: the
`#[cfg(test)]` modules were skipped, `tests/` lay outside the target. Its answer says so: "there
was no verification pass, so these are unverified candidates."

Its ten findings, each read here against the code, which is the same in ai-ctl as in the
fixture (`src/` and `tests/` unchanged since `docs: close the v2 rebuild roadmap` of 2026-10-12):

1. `Plan::new` refuses a context too small for the session scenario whatever scenarios were
   chosen (`domain/bench.rs:176`): `application/bench.rs:170` builds the plan before reading
   `options.scenarios`, so `bench run --scenario depth` fails under 22,529 tokens of context.
   Holds.
2. A symlink in the data path makes a running gateway look like a reused PID
   (`gateway_process.rs:95`), and `BuildStore::contains` has the same prefix problem
   (`build_store.rs:283`). The baseline's first finding, which it reproduced; this run adds the
   build store's side by reading.
3. `build prune` removes a build the running gateway's configuration still starts
   (`application/build.rs:343`): the configuration names `builds/<project>/<id>/llama-server`
   (`build_store.rs:195`), and `prune` checks only the current build. Holds; the baseline's
   second finding, without its `build --force` half.
4. `up`'s wait counts 300 polls, not 30 s (`lifecycle.rs:619`): each `/health` may block 2 s
   (`gateway_api.rs:13`), so a llama-swap that accepts and never answers holds `up` about ten
   minutes under a message saying 30 s; an error from `started.exited()?` leaves the started
   gateway running and recorded. Holds, on a rare path.
5. `SessionPlan::peak` counts the corpus tokens of each turn and the replies, not the `TURN`
   preamble and the template tokens each message adds (`domain/bench.rs:119`,
   `domain/prompt.rs:128`): a context a few hundred tokens above 22,529 passes the plan and
   overflows at the last turn. Holds, in a narrow window.
6. A model's file keys are checked for a leading dash only, not against the reserved flags,
   and the defaults are checked without the model's file flags (`domain/inventory.rs:332`,
   `:252`): `files = { port = "x" }` gives two `--port`, a default `mmproj` beside a model's
   `files.mmproj` two `--mmproj` (`domain/gateway_config.rs:325-345`). Holds.
7. Each fresh prompt templates and tokenizes the empty agent prompt three times, where once is
   enough (`application/bench.rs:471`, `:662`): four needless requests, outside the measured
   ones. Holds, minor.
8. The description of a `Liveness` is written twice word for word (`application/doctor.rs:430`,
   `presentation/lifecycle.rs:100`), and GPU processes are listed in three formats. Holds.
9. `shell_quote` repeats `shell_word` (`application/doctor.rs:688`,
   `domain/gateway_config.rs:493`). Does not hold as stated: the two quote for different
   readers on purpose, `shell_word` leaving `${…}` bare for llama-swap's macros and
   `shell_quote` quoting `$` for a shell; at most one helper with the plain set as a parameter.
10. `render_run` computes the gateway's added time beside `OverheadResult::added`
    (`presentation/bench.rs:237`, `domain/bench.rs:647`). Holds, minor.

Against the v2 baseline, which found four: both runs found the symlink and `build prune`; the
baseline alone found `bench run` printing the run before saving it and `doctor`'s remedy ending
at `publish`'s refusal; `/code-review` alone found findings 1 and 4 to 6, four bugs, and four
cleanups. Together they give eight distinct bugs in the v2 code: the baseline found four,
`/code-review` six, neither all. `/code-review` names no axis, reads no test, runs no check, verifies nothing, writes no
report and gives no fix plan: none of that is in its contract, a JSON list of findings capped at
ten, so its misses on the case's expected output are the misses of its design, not of this run.
Of the nine axes it covers bugs, refactoring through reuse, performance through one finding,
conventions by two greps, and error handling through finding 4; nothing on architecture, dead
or legacy code, security or tests. Its cap hides how many candidates it dropped: the thinking
is redacted.

Task 6 ticked: the run is done on the code and recorded against the baseline. All nine tasks
are done; the phase closes.

---

## Decisions

- **Phase 1's baselines run with the Agent tool, and wait for authoring-skills to allow it
  before the skill's agents exist.** The user's real sessions have the tool; a baseline
  denied it cannot show how an agent without the skill delegates a review, the part the
  skill's design rests on, and its failures would lead phase 2 to answer what real use never
  shows. Set aside: baselines run now with Agent denied and recorded as a deviation (the
  failures that matter most seen only after the skill is built). Proposed to the agent setup
  in `notes/2026-10-13-agent-tool-before-agents.md`; the cases are written meanwhile.
- **The cases run in a fixture: ai-ctl at `docs: close the v2 rebuild roadmap` of 2026-10-12, its history kept, with a `CLAUDE.md`.**
  `evals/fixtures.py` fetches that commit alone into a new repository, so no later commit
  and no document of this roadmap reaches a run, and adds a pinned copy of `CLAUDE.md` whose
  Current work says no roadmap is open, as it did at that commit. A `repository` case would
  have no history to scope a review by and no `CLAUDE.md`, which every real session loads.
  Set aside: the `repository` setup with this roadmap's folder excluded.
- **Two cases: the v2 roadmap's code, and a session with planted defects.** The first is the
  real request, whole and costly; the second, a small session left uncommitted on top of the
  v2 code with defects planted across the axes, gives assertions that count what a review
  finds. Set aside: a third case on one module's debt, and the v2 case alone (no measurable
  finding).
- **Each prompt carries a review tier and the user's agreement to the cost**, after the
  user's own words. Runs are unattended and their preamble says to assume where a question
  would be asked; a skill that waits for the yes would otherwise stop at its estimate, and
  phase 3 would measure nothing of the review. Set aside: the user's words alone.
- **The session of the second case is a commit, not uncommitted changes.** A run's changes
  are the diff from the commit its folder starts at, so uncommitted changes in the fixture
  would count as the run's own work, and the check that a review changes no code would read
  them. Its date and author are fixed, so the fixture is the same at each build. The prompt
  names the session as the last commit.
- **The planted session is `ai-ctl gateway prune`, with eleven defects across the nine
  axes**, listed with their lines in the case's expected output. The session keeps the three
  checks green, as one that ran them would. The assertions of task 8 are written from the
  baseline runs; the list is the answer key, not the assertions.
- **The runs have the Agent tool through `starts_agents: true`**, the key the agent setup
  added for a skill whose agents are not written yet. Phase 2 adds the agents' files under
  `agents`, which implies the key; the cases' digests stay the same, so phase 3 can reuse
  this phase's runs as its baseline with `--reuse`.
- **One paid run per usage window, each started from a new session.** The user's plan caps
  spend at about $20 per window, shared by the session and every run it starts, and a run
  the limit stops is lost. `iteration-1` is prepared for both cases at a ceiling of $18 per
  run, the most one window leaves; `run.py --case` starts them one at a time, the
  post-session case first, the cheaper one. Set aside: a higher ceiling (the window would
  stop the run anyway), both runs in one window (their estimate, $36, passes it), and runs at
  a cheaper model or effort (against phase 0's budget rule). Phases 3 and 4 inherit the
  limit.
- **The v2 case stays whole, even if its run is cut.** A cut run keeps the opening, where
  the scoping, the plan and the delegation show, which the skill's design rests on. Narrowing
  the case to one v2 phase remains open if the user prefers a complete observation.
- **`CLAUDE.md` is versioned**, by the user's choice, reversing phase 0's decision that kept
  it out of git with `.agent-conventions.toml` and `.claude/settings.local.json`, which stay
  out. The fixtures keep their pinned copy, so the runs see the file as it stood at
  `docs: close the v2 rebuild roadmap` of 2026-10-12.
- **A failure seen in one case gives an assertion to both when it does not depend on the
  case**: the claims of the account, the scratch left outside the run's folder. The two cases
  differ in size, not in what a review owes the user. Case-bound failures stay with their case:
  the scope from the Start Commits and the test code for the v2 case, the planted defects and
  the separate causes for the post-session case.
- **The skill does not teach what both baselines did unprompted.** Phase 2 writes no guidance
  for: running the three checks first, in the background while reading; reading the project's
  recorded rules and decisions before judging (`docs/architecture.md`, decision 0001, a phase
  report a change reverses); taking the sibling feature as the reference for a fix;
  reproducing a suspected bug before reporting it, by a throwaway test in a copy outside the
  repository or through the project's own integration test, and leaving the repository clean;
  checking a claim against the history it cites; checking by grep the rules a grep can check;
  checking whether the user's machine is exposed; giving each finding its `file:line`, cause,
  consequence and fix; a section of what was checked and found sound; stating the assumption
  taken. Neither baseline started an agent, by choice: the first found ten of eleven planted
  defects for $1.29, so the tiers must show what agents add on a small scope before
  prescribing them there.

---

## Files Changed

**Added**

- `.claude/skills/reviewing-delivered-code/evals/evals.json`
- `.claude/skills/reviewing-delivered-code/evals/fixtures.py`
- `.claude/skills/reviewing-delivered-code/evals/fixtures/agent-conventions.toml`
- `.claude/skills/reviewing-delivered-code/evals/fixtures/claude.md`
- `.claude/skills/reviewing-delivered-code/evals/fixtures/gateway-prune.patch`
- `CLAUDE.md` — versioned by the user's choice, outside the phase's tasks
- `docs/roadmap/on-progress/review-skill/phase-1-evals-and-baselines-report.md`

**Modified**

- `.gitignore` — `/CLAUDE.md` removed by the user, so that `CLAUDE.md` is versioned
- `docs/roadmap/on-progress/review-skill/README.md`
- `docs/roadmap/on-progress/review-skill/phase-1-evals-and-baselines.md`
- `docs/roadmap/on-progress/review-skill/phase-2-skill-build.md`
- `docs/roadmap/on-progress/review-skill/phase-3-evaluation.md`
- `docs/roadmap/on-progress/review-skill/phase-4-first-review.md`

---

## Problems And Deviations

- **The change landed in authoring-skills does not cover this phase's baselines.** It gives
  the Agent tool to runs whose evals list existing agent files, and the skill's agents are
  written in phase 2. Phase 0's proposal described the arms of a comparison and missed the
  first observation of a new skill. Fixed by a second proposal,
  the agent setup's `notes/2026-10-13-agent-tool-before-agents.md`, decided the same day
  (commit `fe4f87e`): `starts_agents: true` in `evals.json`.
- **The fixtures' task cites a decision phase 0 did not take** ("as decided in phase 0"):
  no decision of phase 0 names a fixture. Taken in this phase with the user, above.
- **The phase's constraint on the skill's files widened.** It allowed `evals/evals.json`
  alone; the fixtures need `evals/fixtures.py` and the files it builds from. The constraint
  now names the evals folder.
- **The session case differs from what was put to the user**: changes left uncommitted
  became a commit, for the reason recorded under Decisions. Confirmed by the user in the
  next session; the prompts were kept as written too.
- **Task "Validate the evals file and prepare the first iteration" done in two sessions.**
  The file validated in the first; the iteration waited for authoring-skills' change, since
  one prepared then would have denied the Agent tool, and was prepared in the second.
- **The budget rule of phase 0 met the user's plan.** It set no cap, each batch announced
  and started on the user's yes; the plan caps spend at about $20 per usage window, below
  the two baselines together and possibly below the v2 case alone. Fixed by one run per
  window; the v2 case may yield a partial observation. Phases 2, 3 and 4 carry the
  constraint.
- **The first reading of the post-session run missed one failure**: the tier, `cargo audit`
  and `cargo mutants` unanswered, found when the second run showed the same gap. Added as its
  F7 in `evals.json`; the Work Log's earlier list stays as written.
- **`run.py` estimates a case from another case's runs.** It announced the v2 run at $1.29,
  the mean of the post-session run, for a review estimated at $10 to $30. The ceiling kept the
  commitment; the figure was corrected by hand to the user. Proposed to the agent setup in
  `notes/2026-10-13-run-estimate-per-case.md`; until it is decided, phase 3's announcements
  need the same correction while a case or an arm has no run of its own.
- **The first `/code-review` run reviewed the wrong scope.** Typed bare, the command took the
  level typed last, `xhigh`, and the last commit, the docs-only `docs: close the v2 rebuild roadmap` of 2026-10-12: nine findings on the
  v2 roadmap's `summary.md`, none on the code. Its cost is spent; the run is done again on
  `src/` in a fresh window, by the user's choice. Its findings on `summary.md` are left to the
  user; the v2 roadmap is closed.
- **The `/code-review` run on the code reviewed `src/` alone**, the target typed; `tests/` and
  the `#[cfg(test)]` modules were left out, so the comparison says nothing of the tests axis.
  Left as is: the expected output's test assertions measure the skill in phase 3.
- **The two `/code-review` runs' exact cost is unknown.** Their sessions were still open at the
  closure, so no `cost-state` record holds their output; the input side is known ($2.04 and
  $2.54) and `usage.py`'s estimate of the output gives $9.11 and $7.13, likely high. Left open:
  the records land when the sessions end.
- **`scripts/audit.py` fails on a skill holding its evals alone**, with a traceback where
  rule 3 of authoring-skills asks for its run after every write. Not fixable here; reported
  in the agent setup's `notes/2026-10-13-agent-tool-before-agents.md`.

---

## Changes To Later Phases

- `phase-2-skill-build.md`: a constraint added, the user's plan cap of about $20 per usage
  window, which the tiers and their estimate must account for: the whole-repository review
  of phase 4 may not fit in one window.
- `phase-3-evaluation.md`: a constraint added, one paid run per usage window, each started
  from a new session with `run.py --case` and `--run`.
- `phase-4-first-review.md`: a constraint added, the plan cap of about $20 per usage window,
  which the review spans as the skill's tiers provide.
- `phase-2-skill-build.md`: its Dependencies name the gaps of the built-in `/code-review` that
  the bugs agent answers, and the count of bugs each run found, so that the agent is built
  against them.

---

## Assessment

The phase delivered what it promised: two cases in `evals/evals.json`, each with its fixture
pinned at `docs: close the v2 rebuild roadmap` of 2026-10-12, a baseline run of each read in full, its failures quoted, twelve
assertions per case traced to them or to the user's intent, what the baselines did right set
aside so the skill does not teach it, and the built-in `/code-review` run once on the v2 code.
It took longer than planned: two proposals to the agent setup before a baseline could have the
Agent tool, the user's plan cap that turned every paid run into a session of its own, and a
first `/code-review` run on the wrong scope.

The baselines show that an agent without the skill reviews well what it reads, reproduces its
suspicions and leaves the repository clean, and that what it lacks is the frame: no report
file, no scope from the recorded Start Commits, no test code read, no axis named, no coverage
stated, the tier ignored, no verification by another pass, no fix plan, and claims of reading
that do not hold. Neither baseline started an agent; the post-session case found ten of eleven
planted defects for $1.29, the v2 case four bugs for $4.51. `/code-review` at `high` on `src/`
found six bugs, two shared with the baseline, for about as much, reading no test and verifying
nothing.

What phase 2 needs to know first: the failures to answer are the frame's, listed as F1 to F10
and F1 to F7 in `evals.json`, and the bugs agent has the two runs' eight bugs as its measure,
neither run having found all of them. A whole-repository review in one session cost $4.51 to
about $7, well below the window phase 2's constraint feared; what agents per axis add must be
weighed against that, and against the post-session case, where one session sufficed.
