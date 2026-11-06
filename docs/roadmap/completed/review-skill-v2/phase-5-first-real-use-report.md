# Phase 5 Report: First Real Use

**Phase:** [phase-5-first-real-use.md](phase-5-first-real-use.md)
**Start Commit:** `docs(review-skill-v2): close phase 4` of 2026-11-04

---

## Work Log

### 2026-11-04

Scope set with `scope.py roadmap` on `audio-image-backends`. Its first run ended the scope at
`docs(roadmap): describe each roadmap in an overview of them` of 2026-11-02, 59 commits past the roadmap's closure at `docs(audio-image-backends): close the roadmap without phase 4` of 2026-10-30, and took this roadmap's own scripts
for the reviewed code: fixed in `scope.py`, a roadmap's end now the last commit touching one of its
phase reports, its test watched failing, then green.

The scope by hunk, `docs(audio-image-backends): create the roadmap` of 2026-10-19 to `docs(audio-image-backends): close the roadmap without phase 4` of 2026-10-30: 341 hunks in 32 code files, 7,126 lines handed to the
reviewers, against 26,201 for a scope by file of the same files, 27%. `Cargo.toml` and
`Cargo.lock` changed, so the dependency axis runs. Asked, the session told the user that this
review is the one `audio-image-backends` closed without.

The go-ahead proposed `medium`, eight agents, from step 2's one rule, the `reading:` line. The user
asked why not `high`, and noted «tu as pris le choix par défault sans annalyser le cas réel»: this
review is the only one on every axis of that code, the closing pass running `review-security`
alone, and the code holds the API key's patches and the downloads. `high`, thirteen agents, each
reviewer with its verifier, was estimated at 94% or more of a five-hour window and 13% of the
seven-day one. Step 2 fixed: the tier proposed from the case, the tier above named with its line.
Traced to phase 0's tier "proposed at the go-ahead by the diff's size"; on the user's yes, a
design-feedback note went to the agent setup, answered in skill-tooling 0.4.2 by a case on the other
side of a default, so `cost-gate` now guards that a commit touching no guarded surface gets `low`
or `medium`.

The seven-day window was at 94%, its reset on 2026-11-06, and the six reviewers alone at 70 to 96%
of a five-hour window: the reviewers go in two batches, the verifiers after them. Two more failures
fixed before any start: no write to the repository while an agent runs, `guard.py` hashing the
review's folder at each stop; and `given_to`'s value with no mutant missed, `null`.

At the user's word, «on peut lancer les premiers runs», the review started at tier `high`. Every
check green, 828 tests; `cargo deny check` clean but for three duplicates; the mutants on the
scope's diff, 198 in 582 s, none missed. The first batch, `review-security`,
`review-correctness`, `review-structure`, `review-performance` and `review-dependencies`: each
answer accepted at its first submission, no stop blocked, 29 findings and three probes for the user.

### 2026-11-05

The second batch, `review-tests` and `review-cleanup`: 17 and 13 findings, each answer accepted at
its first submission; the session's own answer holds none. Three merges, `write_file`'s check
removed rather than hardened since no input reaches it: 55 findings, 35 above the floor; of the 20
below it, 12 `fix at once` in batch 1 and 8 registered.

Two failures fixed: `review-tests`' new tests, in a scratch folder step 9 removes when a review
stops before its verifiers, now kept in `repro/review-tests/`; and step 2 naming `usage.py`'s
threshold as a figure, which moved from 90% to 97% during the batch. No verifier started in that
session, at the user's word: «on va attendre le passage sur une session neuve».

A fresh session, on another account at 0% of its seven-day window, ran the five verifiers, each
answer accepted at its first submission: of the 35 findings above the floor, 33 confirmed, finding
13 refuted and finding 32 fallen below the floor. No answer was refused in this review. Phase 4's
check: 1 and 29 are `high` at «content ai-ctl fetches […] runs code or gets the key», each
confirmed by a run; 2, 3, 4 and 30 `low` at «a control weakened with no damage shown»; each severity
meets its anchor, though no agent names one. A verifier's probe writing to its own scratch folder:
fixed. The triage gave sixteen batches, the tests alone first, and three hard cases, 1, 4 and 26.

The user took the recommendation on 1 and 4. On 26 the user asked twice what the bound counts, the
models running at once or those of the library, and the list of the matrix's sets did not answer
it. A later session answered by what the bound limits, the library's small models alone, each
speech model multiplying the sets where a chat or image model adds one; then by why llama-swap
needs a matrix at all, since it accounts for no VRAM and runs one model at a time without one. The
user chose 20,000.

Answering, the session found `docs/stack.md:313` still saying that one model runs at a time with
the shipped inventory: `review-structure` had read that document by its hunks alone, `stack.md`
being the LLM stack's, and `findings.py add` refused the session's finding once its own answer was
in. Both fixed, `findings.py`'s test watched failing on the refusal; the defect is finding 56,
confirmed by the sixth verifier the go-ahead counted, in batch 15.

The closure's checks found the study repository's check, `sid.py check .`, red since the review's
first record: its `checks` copied that command with its path through `study/`, a link the check
forbids in a tracked file.
At the user's word, «une correction propre plutôt qu'un patch», rather than `--skip docs/reviews`:
`findings.py` now refuses a check naming a path another clone lacks, one git ignores or one a link
leads out of the repository, its tests watched failing, and step 3 says to name such a program by
its file name; the review records `sid.py check .`, and the check is green. `scripts/audit.py`
clean after each fix of the phase.

---

## Decisions

- **The tier `high`** (the user, 2026-11-04: «oui on part sur high»), started the same day at the
  user's word, in batches that each fit a five-hour window at their
  high end, the reviewers in two, then the verifiers: the only review on every axis this code
  gets, its security surfaces among it, each axis's findings refuted by a verifier of its own.
- **The surface wins over the size** (stated to the user, 2026-11-04): a scope the session could
  analyse alone, at `low`, that touches a surface `docs/security.md` guards with no later security
  pass over it, takes `high`. `low` leaves the session that knows the code as its only reader,
  the look an independent security pass exists to give, and nothing later covers it; on a small
  scope, each agent reads little.
- **The reviewers' second batch forced** (the user, 2026-11-05: «tu peux lancer», the session
  having recommended waiting for the reset): `review-tests` and `review-cleanup` started with
  `--force` at 91% of the seven-day window, past the 90% `usage.py` then refused, and took it to
  95%. The verifiers keep a start of their own, which the user decides.
- **Five verifiers, not six** (stated to the user, 2026-11-05): one per reviewer with a finding above
  the floor, since step 6 gives none below it and `review-cleanup` has none above. The sixth run the
  go-ahead counted verified finding 56, the session's, later.
- **The three probes not run** (stated to the user, 2026-11-05): their answers change no fix, 13
  being refuted and 9's fix refusing before any request, and each starts the stack.
- **The verifier's word over the reviewer's** (stated to the user, 2026-11-05) on 30's and 37's
  fixes and 32's effect, each shown by a run.
- **Finding 2's document alone** (stated to the user, 2026-11-05): a patch refusing host paths,
  kept at each release, is out of proportion with its effect.
- **Finding 1: llama-server's web UI off** (the user, 2026-11-05): the pinned archive's SHA-256
  has no source but the bucket itself, and one flag brings the UI back the day a user needs it.
- **Finding 4: the OpenWebUI image now running, checked and pinned** (the user, 2026-11-05):
  its volume is already migrated by it, which an older image may not open.
- **Finding 26: an inventory refused past 20,000 sets** (the user, 2026-11-05: «20 000 semble
  être un bon compromis»), by `doctor` and `up` before any start, the error saying to take models
  out of the inventory: 10,000 differs by about two small models, and a warning alone leaves `up`
  failing on its wait with no cause. The user sees the library's growth handled later by models
  switched on and off, only those on entering the matrix; out of this roadmap.
- **No review of the phase's hunks** (stated to the user, 2026-11-05): `scope.py range` from
  `docs(review-skill-v2): close phase 4` of 2026-11-04 finds no code hunk, the phase having changed the skill's scripts, each fix by a test watched
  failing, its text, its agents and the review's records.

---

## Deviations

- `scope.py roadmap` ended a closed roadmap's scope at a later roadmap's commit touching its
  folder: fixed in this phase, its end now the last commit touching a phase report.
- The go-ahead proposed the tier from the scope's size alone and named no other: fixed in this
  phase, at step 2's line.
- Steps 3 and 4 had the session write to the review's folder while agents ran, which the guard
  blocks at their stop: fixed in this phase, no write to the repository while an agent runs.
- `given_to` had no stated value with no mutant missed: fixed in this phase, `null`.
- A review stopped before its verifiers lost what an answer cites in scratch: fixed in this phase,
  at step 9.
- Step 2 held `usage.py`'s threshold as a figure, which moved during this use: fixed in this phase.
- A verifier's probe wrote to its own scratch folder, which step 9 removes: fixed in this phase, at
  `review-verifier`'s step 3.
- `review-structure` read only the documents of the scope's domains and missed a statement of
  another that the scope made false: fixed in this phase, at its section on the CLI's contract.
- `findings.py add` refused a defect the session saw after its own answer: fixed in this phase,
  the session answering again in a run of its own.
- The review's record of the checks cited a path of the study repository, which turned
  `sid.py check` red from 2026-11-04 to the closure: fixed in this phase, `findings.py` refusing a
  check naming a path another clone lacks.

---

## Assessment

The v2 reviewed `audio-image-backends`' code at tier `high` on every axis, the dependencies
included: 56 findings, 46 to fix in sixteen batches, 34 of them confirmed by a verifier, 12
below the floor in batch 1, nine registered and one refuted. Every agent's answer passed
`findings.py add` at its first submission and the guard blocked no stop, so the agents' format
and rules held in real use. The security findings meet their anchors, though no agent names one.
The review is committed at stage `fixing`, the three hard cases answered.

The use found ten failures of the skill, each recorded in `failures_in_use` and fixed. Most sit
on the session's side of the review rather than in the agents' answers: the scope's end, the
tier's choice, writes while agents run, what a stopped review keeps, a moving threshold, a
finding seen late, a record citing a path no other clone has, which only the closure's checks
showed. One sits with an agent, `review-structure`'s reading confined to the scope's domains. The cost sets the pace: the review took three sessions, two accounts and a forced batch,
each step held for the user's word.

Phase 6 needs to know first that `findings.py tasks` prints its batches, each confirmed fix's
reproduction is in `repro/<n>/` written against `docs(audio-image-backends): close the roadmap without phase 4` of 2026-10-30, and the hard cases' answers are in
their findings' notes: 1 turns llama-server's web UI off, 4 pins OpenWebUI's running image, and
26 refuses an inventory past 20,000 sets, its early stop in `largest_sets` taking mutants no test
without timing kills, to justify in that phase's report.
