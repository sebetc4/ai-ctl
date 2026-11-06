---
name: reviewing-delivered-code
description: Reviews the code a phase, a roadmap or a session delivered, hunk by hunk on thirteen axes from architecture, bugs and security to tests, performance and the dependencies, into a report under docs/reviews/delivered-code/ whose findings are fixed or kept in a debt register. Use at the close of a phase that changed code, for a roadmap's closing pass (its security pass, its crates, its recorded mutants), after a large coding session, when the user asks to review delivered code, for a security pass alone, to check or audit the crates (RustSec advisories, yanked or unmaintained crates, updates within and beyond Cargo.toml, maintainers, reliability, llama-swap's and the backend sources' lag), to check the technical debt (dette technique), or to continue a review in progress.
---

# Reviewing Delivered Code

A review lives in its folder, `docs/reviews/delivered-code/<YYYY-MM-DD>-<scope>/`:
`findings.json`, its state, which `scripts/findings.py` writes and checks, the session
writing by hand only its `checks`, `mutants` and `status`, which `findings.py report` then
checks; `report.md`, written from it by the same script, never by hand; `repro/`, each
confirmed finding's red test and settled fix, kept for the fixes. Beside the folders, one
for the repository: `register.json`, the debt register, kept by `findings.py set`, and
`mutants.json`, the mutants the fixes applied by hand. Run the scripts from the
repository's root; each one's `--help` gives its arguments and outputs.

## The Moments

| Request | Scope | Agents and scripts | Fixed |
|---|---|---|---|
| A phase's close, the phase having changed code | the phase's diff, from the commit it opened on | the six reviewers; the dependency axis too when `Cargo.toml` or `Cargo.lock` changed | in the phase: each confirmed finding above the floor on the phase's code, and on older code each one `medium` or worse |
| A roadmap's close, the roadmap having changed code | the roadmap's diff | `review-security` alone, the dependency axis, `scripts/regression.py` | in the roadmap's last phase: what bears on the roadmap's code or is `medium` or worse |
| A session's work, or a commit range | the range | the six reviewers | as at a phase's close, in this session |
| A security pass alone | the range given | `review-security` | as at a roadmap's close |
| The dependencies alone | none | the dependency axis | as at a roadmap's close |
| A full review, only when the user asks for one | the range the user gives | the six reviewers | in a roadmap made from `findings.py tasks`, as at a roadmap's close |

The six reviewers: `review-security` (security), `review-correctness` (bugs, error handling,
interruption and concurrency), `review-tests` (tests), `review-cleanup` (dead and legacy
code, refactoring), `review-structure` (architecture, conventions, on-disk formats, the
CLI's contract), `review-performance` (performance). The dependency axis is `cargo deny
check`, `scripts/dependencies.py` and `review-dependencies`, which judges their output. The
agents are the six reviewers, `review-dependencies` and the verifiers.

No pass follows a roadmap's closing pass: its fixes rest on their tests, their mutants and
the regression run. A phase that changed no code has no review.

## Throughout

- Of ai-ctl's commands, run only those that read: `doctor`, `status`, `build list`, and
  the help and version forms; never `up`, `down`, `build`, `gateway install` or `init`,
  which start the stack or change the machine. `scripts/guard.py`, wired in the project's
  `.claude/settings.json`, holds the review agents to the same and to read-only git.
- Reach crates.io and GitHub only through `scripts/dependencies.py`, whose `User-Agent`
  carries no personal data, and a repository's tags through `git ls-remote --tags`.
- Put the review's working files — copies, diffs, answers, outputs — in the scratch folder
  of step 2.
- While an agent runs, change no file of the repository, the review's folder included, and
  make no commit: at each agent's stop, `scripts/guard.py` compares `git status` and each
  listed file's hash with those of its start, and a change blocks that stop and makes the
  agent give its whole answer again.
- The session triages: it asks the user only the hard cases of step 7.

## Steps

1. **Set the scope** with `scripts/scope.py`:

   - a phase's close: `scripts/scope.py range <start>`, `<start>` the commit the phase
     opened on, which the roadmap skill records at the opening;
   - a roadmap's close: `scripts/scope.py roadmap <roadmap-folder>`;
   - a session, a range, a security pass, a full review:
     `scripts/scope.py range <base> [<end>]`, the last commit alone being `range HEAD~1`;
   - the dependencies alone: no scope; the review's base and end are `HEAD`.

   The scope is given by hunk, each with the items that hold it as
   `path:first-last (symbol)`, and its `reading:` line counts the lines handed to the
   reviewers. When `uncommitted:` is not `none`, name those paths at the go-ahead: the
   review covers the commits only.

   Check: the script exits 0. On exit 1, its message names the commit or the folder that
   is wrong: ask the user for the range.

2. **Ask one go-ahead** for the whole review, across its sessions. Take the tier from the
   request, or propose it from the case, giving its reasons:

   - `high` when the scope's code touches a surface `docs/security.md` guards and no later
     security pass covers that code, however small the scope: an open roadmap's closing
     pass covers every commit of its diff;
   - otherwise `low` when the session can analyse every item with its context itself, by
     the scope's `reading:` line, and `medium` when it cannot.

   Name the tier above the one proposed, with what it adds and its `usage.py` line.

   | Tier | Reviewers | Verifiers |
   |---|---|---|
   | `low` | none: the session reviews the moment's axes itself, the reviewers' files in `.claude/agents/` as checklists under the reviewing-as-an-agent skill's rules, and answers as `session` | one |
   | `medium` | the moment's reviewers, at effort `medium` | one |
   | `high`, `xhigh`, `max` | the moment's reviewers, at the tier's effort | one per reviewer, at the tier's effort |

   `review-dependencies` takes no verifier. Pass each agent the effort its tier gives, as
   the Agent call's `effort` parameter. Create the scratch folder with `mktemp -d`, and write the agents to
   `<scratch>/plan.json`:
   `{"tool": "reviewing-delivered-code", "keys": [{"key": {"variant": "subagent", "agent": "<type>", "model": "<the session's model id>", "effort": "<effort>"}, "count": <n>}]}`,
   one key per agent type and effort; then run
   `~/.claude/skills/authoring-skills/scripts/usage.py estimate <scratch>/plan.json`, of
   the authoring-skills skill.

   Write the tier, each agent counted with its effort, and the line `usage.py` prints, in
   your own sentence, in your text before the first agent starts: what a command prints
   stays out of the user's sight. Give no rate and no total spent, and announce neither a
   later stop nor the `--start` below, a step of the session's own. When the request
   accepts the cost beforehand, it is the go-ahead: write these and go on. Otherwise end
   the turn on the question, and start nothing — folder, checks or agent — before the
   user's yes; on a no, remove the scratch folder as step 9 does.

   Before each start of agents, in this session or a later one, run `usage.py estimate
   <scratch>/plan.json --start` with the agents it starts in the plan. On exit 1, a window
   is past the threshold its line names: start none, and stop as step 9 says.

3. **Start the review.**

   - Write the scope's output to `<scratch>/scope.txt`; each agent gets its folder,
     `<scratch>/<agent>`.
   - Create the review's folder: `scripts/findings.py new <folder> --base <base>
     --end <end> [--roadmap <name> [--phase <n>]] --by <the request|the user>
     --text '<the go-ahead's words>' --tier <tier> --agent <type>:<effort> ...`, once per
     agent counted, verifiers included, and `session:<tier>` for the session's own
     findings.
   - When the review has a scope, start the mutants on its diff, in the background, in a
     copy of the end:

     ```bash
     git diff <base> <end> > <scratch>/scope.diff
     mkdir <scratch>/copy && git archive <end> | tar -x -C <scratch>/copy
     TMPDIR=<scratch> cargo mutants -d <scratch>/copy --in-diff <scratch>/scope.diff -o <scratch>
     ```

     The copy, because `--in-diff` refuses a tree that differs from its diff. Set no
     limit; when `cargo mutants --list --in-diff <scratch>/scope.diff` counts more than
     about 1,200 mutants, an hour at the rate measured on ai-ctl (284 in 14 minutes,
     2026-10-19), run it as shards with `--shard <k>/<n>`. Exit 2 means some mutants were
     missed, 3 that some timed out, 4 that the copy's tests fail with no mutant
     (https://mutants.rs/exit-codes.html): a red check, or, the checks green, a test
     needing what the copy lacks, such as `.git`, for `.cargo/mutants.toml` to skip, its
     log in `<scratch>/mutants.out/log/baseline.log`. `<scratch>/mutants.out/missed.txt`
     goes to `review-tests`, or, in a moment without it, each missed mutant is a finding
     of the session's.
   - Run the checks of `.agent-conventions.toml`; at a roadmap's close and on the
     dependency axis, `cargo deny check` too. Name every test that skipped itself, such as
     a `skipped: <reason>` line of `cargo test`: it is coverage that did not run.
   - On the dependency axis, `scripts/dependencies.py <folder>` writes
     `dependencies.json` into the review's folder, over the network. On exit 3 it wrote
     nothing, its reason on stderr: run it once more, then give `review-dependencies`
     `cargo deny check`'s output alone and write the reason into the status's note.
   - At a roadmap's close, `TMPDIR=<scratch> scripts/regression.py` runs the recorded
     mutants, each in a copy; each `missed` or `gone` is a finding of the session's. On
     exit 3 the run was not made: a mutant written wrong in `mutants.json` is a finding of
     the session's, a copy that does not build a red check.

   Write each check's command and result into `findings.json`'s `checks`, a program
   another clone lacks — a path git ignores or a link leads out of the repository — by
   its file name, which `findings.py` refuses otherwise; once the mutants have ended and
   no agent runs, their counts into its `mutants`, `given_to` naming who got the missed
   ones, `null` with none missed; then run
   `scripts/findings.py report <folder>`, which checks the file and writes the report.

4. **Start the reviewers**, each with `<scratch>/scope.txt` and its folder, where it
   writes its whole answer to `answer.txt`; `review-tests` once the mutants have ended,
   with their missed list; `review-dependencies` with `dependencies.json` and
   `cargo deny check`'s output.

   Once every agent started has stopped, add each answer: `scripts/findings.py add <folder>
   <agent> <scratch>/<agent>/answer.txt`. The script numbers the findings, computes each one's
   origin, floor and fingerprint, and prints each match the register holds. When it
   refuses the answer, start the agent again once with its answer and the faults the
   script named; refused again, write it into the status's note.

   The session's own findings — its review at `low`, a red check, a recorded mutant
   `missed` or `gone`, a missed mutant no reviewer takes — go in one answer in the format
   of the reviewing-as-an-agent skill, `FINDINGS: 0` when there are none, added as
   `session` once the checks, the mutants and the regression run have ended. A defect the
   session sees later, in the triage or a fix, goes in a further answer added as `session`,
   then through steps 6 and 7.

5. **Merge** the findings that share a root cause, or ask opposite changes of one piece of
   code, once every answer is in: `scripts/findings.py merge <folder> <n> <n>...`, which
   numbers the findings again and is refused once a verdict or an action is set. Then,
   for each finding whose `matches` holds an entry of the register, compare the titles:
   the same defect is set `action=known note='<review> finding <n>'` and goes no further.

6. **Verify the findings above the floor.** Below the floor — effect `maintenance` or
   `none`, or a `slow` with no severity — a finding gets no verifier: set it
   `action='fix at once'` with one batch for all of them when it lies on the scope's new
   code with a local fix, or is the dependency axis's update within `Cargo.toml`'s
   requirements; `action=register` with its reason otherwise.

   Give each verifier its findings as `report.md` shows them, `<scratch>/scope.txt` and its
   folder. From its answer, write each VERDICT block:
   `scripts/findings.py set <folder> <n> verdict=<CONFIRMED|REFUTED|UNCERTAIN>`, with the
   `severity` or `effect` it corrected; copy each confirmed finding's `repro/<n>/` from its
   folder into the review's; then add the answer with `findings.py add`, for its leads and
   what it read. A lead below the floor is triaged as above; above it, give its verdict
   yourself, from the verifier's reproduction run again, with no further verifier. Verify
   a dependency finding yourself against `cargo deny check`'s output and
   `dependencies.json`, with no verifier.

   Check: every finding above the floor has a verdict, or the action `known`.

7. **Triage**, with `findings.py set`: a `REFUTED` finding `action=rejected` with its
   reason, which enters the register so that no later review raises it; a `CONFIRMED`
   finding `action=fix` with its batch when its moment fixes it, `action=register` with
   its reason otherwise. A batch holds the findings one change fixes, numbered in the
   order that keeps each commit green.

   Ask the user only these hard cases, each with what each answer changes:

   - a finding still `UNCERTAIN`;
   - a fix that changes a behaviour the user sees, a documented contract — a command's,
     an on-disk format — or a security guarantee of the repository;
   - a fix that removes a feature, or takes a crate beyond `Cargo.toml`'s requirements or
     replaces it;
   - a reviewer and its verifier disagreeing on the severity or the fix;
   - a fix out of proportion with its effect, with your reasons;
   - an agent's `PROBE`, with what it risks; run the ones the user accepts.

8. **Fix** where the moment says, by `references/fixing.md`, which holds the method and
   each batch's steps; read it before the first fix. In this session, fix every batch but
   the hard cases', then put the hard cases. When a session's or a full review's fixes do
   not fit in this session, `scripts/findings.py tasks <folder>` prints them as a
   roadmap's tasks, for the roadmap skill.

9. **Stop or finish.** Set `findings.json`'s `status`: its `stage`, `reviewing`,
   `verification`, `fixing` or `complete`, and its `note`, what the next session takes
   up; `complete` once every finding has its action and each fix its `fixed_in`. When the
   review stops before its verifiers, copy what an answer cites from its agent's folder,
   such as a reviewer's tests, into `repro/<agent>/` and name it in the note, for that
   agent's verifier. Run `scripts/findings.py report <folder>`. Commit the review's folder
   and the register with the fixes, or alone when the review stops. Remove the scratch
   folder with a command that refuses any other path:

   ```bash
   case '<scratch>' in "${TMPDIR:-/tmp}"/tmp.?*) rm -rf -- '<scratch>' ;; *) echo "not a mktemp folder: <scratch>" >&2; false ;; esac
   ```

   Check: `ls <scratch>` fails. Then give the report's path, the findings by action, at a
   roadmap's close that the roadmap can close, and the hard cases' questions last.

## Continuing A Review

Read its `findings.json`: `status` says where the review stands, and the agents whose
`state` is `answered`, the `checks` and the `mutants` it holds are done. The go-ahead it
holds stands: ask none, start no agent that answered and run no check or mutant it
records. Create a scratch folder and its `plan.json` for the agents still to start, then
take up the step its `stage` names: `reviewing`, step 4 for the agents still to answer;
`verification`, step 6; `fixing`, step 8.
