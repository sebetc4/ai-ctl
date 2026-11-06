# Phase 0 Report: Clean Records

**Phase:** [phase-0-clean-records.md](phase-0-clean-records.md)
**Start Commit:** `docs(fresh-history-and-ci): add the rule that no session is cited` of 2026-11-06

---

## Work Log

### 2026-11-06

The design-feedback note written in the agent setup's `notes/design-feedback/`, untracked there:
the review's `scope.py` reads the report template's `**Start Commit:**` line at a phase's and a
roadmap's close, the setup's planned rule against commit hashes meets `findings.json`'s `base`,
`end` and `fixed_in` and the roadmap skill's own `**Branch:**` line, and the note recommends
keeping the line and writing the rule against sessions alone. It waits for the user's reading.

The evals' fixtures built from `evals/fixtures/base.tar.gz`, ai-ctl's tree of 2026-10-30 less
`.claude/` and roadmap `review-skill`, 188 files in 1.0 MB, without the global header in which
`git archive` names the commit; the author of the fixtures' commits is a constant of
`fixtures.py`, which no longer runs `git archive` or `git log`. The new test of `test_fixtures.py`
builds a case from a copy of `evals/` in a repository of one unrelated commit: it failed on its
assertion, the builder exiting 1 on `git archive`, then passed. Each of the five cases builds the
same commits as before the change, every hash and subject compared, and `continued-review`'s
uncommitted review the same bytes. `sid.py check` found nothing on the extracted tree before it
was tracked, by itself and with `--closed`. The six eval tests passed in a copy of the tracked
tree committed as one root commit, the history's base commit absent.

The records rewritten. Counted by resolving each hexadecimal word of 7 to 40 characters against
the history, 661 citations of a commit on 589 lines of 44 files, 269 commits; 437 were rewritten
by rule, 49 by hand — a hash inside a command, a range, a fenced block, a possessive —, and the
two reviews' `findings.json` and `dependencies.json` and `evals.json` field by field, the reviews'
`report.md` then written again by `findings.py report`, which first rewrote both unchanged and
exits 0 on both after. The sessions cited on 30 lines: the 9 holding a whole session id, and the
abbreviated ids of review-skill phase 4's table of failures, its legend, and the paths of
sessions' transcripts and scratch folders. The two commands below find nothing outside the
fixtures but this roadmap's own start commit, `docs(fresh-history-and-ci): add the rule that no session is cited`, which phase 1 rewrites, and the Hub's blob id of a
fixture file, `e8669c03…`, which the tree holds; `sid.py check . --closed` finds nothing; the
diff read whole. The five checks and the six eval tests green; the code's diff, one doc comment
of `src/domain/prompt.rs`, gives `cargo mutants --in-diff` no mutant.

```bash
git grep -I -n -E '[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}|[0-9a-f]{8}…(\.jsonl|/)|\.claude/project[s]/|/tmp/claude-[0-9]+/|scratchpa[d]/' -- . ':!**/fixtures/**' | grep -v '00000000-0000-0000-0000-000000000000'
git grep -I -h -o -w -E '[0-9a-f]{7,40}' -- . ':!**/fixtures/**' | sort -u | git cat-file --batch-check | grep -v ' missing$'
```

The rule written among `CLAUDE.md`'s conventions, as Sources: no session's id and no path into a
session's transcript or scratch folder, a fact's source named by its date and its work, a commit
by its hash, a captured payload among the fixtures no citation. The first command above finds
nothing after it.

The documents this phase's changes describe: the evals' fixtures, in `fixtures.py`'s docstring,
changed with the builder, and in `test_fixtures.py`'s, which still said the cases are built from
this repository's history; `CLAUDE.md`, its convention Sources. The skill's `SKILL.md`, its
references and `docs/` say nothing of how the fixtures are built, and stand.

The review of the phase's hunks at `low`, the user's "oui": one code hunk, the doc comment of
`src/domain/prompt.rs`, 7 lines read; no mutant on the diff, the ten checks green, no finding,
so the verifier did not start. Its record, `docs/reviews/delivered-code/2026-11-06-fresh-history-and-ci-phase-0/`,
cites this roadmap's commits, which phase 1 rewrites with the rest of its records.

The note read by the agent setup at the user's word, its verdict brought by the user: solution 1
retained, the note processed and removed there (Decisions).

---

## Decisions

- The base tree is tracked as one gzip tarball, not as its 188 files: the 2026-10-30 sources stay
  out of every tool that reads the repository's Rust and Markdown files — `scope.py`'s code route,
  a second `Cargo.toml` for cargo and rust-analyzer to find, a search for a symbol, the records'
  check of this phase —, and `fixtures.py` keeps its tar extraction. The cost: `sid.py check`
  reads no binary file, so the tarball's content was checked once, before it was tracked, and it
  is never rewritten.
- The agent setup retained the note's solution 1: its report template keeps the
  `**Start Commit:**` line and the README's `**Branch:**` line its hash, so `scope.py` changes
  nothing; its rule bans, in the tracked files and the messages a push sends, a session's id, a
  path into a session's transcript or scratch folder and the file name of one of its own tool
  reviews, never a commit's hash, a roadmap's branch reaching `main` by fast-forward alone; the
  note its reset leaves in each repository carries that rule, and the check it installs looks for
  no hash. ai-ctl's Sources convention agrees with it. Nothing changes in the installed skills
  until the setup opens its roadmap.
- A commit is cited as its subject in a code span followed by ` of <date>`, the date left out
  under a Work Log heading of that day or in a review of that date: the subject is the commit's
  name, and with the date it finds the commit in the archived history. A fenced block quotes the
  subject; a command keeps its shape, the commit named beside it or as `<base>`; in a review's
  `findings.json`, `base` and `end` hold the cited subject, `fixed_in` the subject alone, which
  `findings.py` puts in a code span, and a mention of the review's own base or end reads "the
  scope's base" or "the scope's end". The lines keep their breaks, unwrapped.

---

## Deviations

- Left as they are, being no citation of this history: v1's `1a49576`, which v1's archive holds;
  the commits of an eval's fixtures, such as `01b2813`; the Hub's blob id `e8669c03…`; a review's
  scratch folder, `/tmp/tmp.*`; "the session's scratchpad" in prose, with no path; the line
  numbers of a transcript quoted in review-skill phase 5's table, its source named.
- Three subjects dropped instead of quoted: two name the agent setup's repository and one the
  studied source of the security skill, both kept out of the tracked files by the closed rule.
  The security pass of 2026-10-19 ends at `HEAD` with no subject, and the note committed apart in
  v2-review-fixes phase 5 is named by its file alone.
- The sessions cited on 30 lines, against 9 counted at the roadmap's creation, which counted the
  lines holding a whole session id.

---

## Assessment

The phase leaves the reset nothing the repository needs from the old history. The evals' fixtures
build from a tarball of ai-ctl's tree of 2026-10-30, the same commits as before, in a repository
lacking every old commit. The records cite no commit of the old history and no session: each
commit by its subject and date, found again in the archived history; the two reviews of the v2
format still pass `findings.py report`. `CLAUDE.md` holds the Sources convention, and the agent
setup keeps the `**Start Commit:**` line that the phase reviews' scope reads.

Phase 1 needs first the two commands of this report, which it runs on the tree the root commit
takes after rewriting this roadmap's own citations: the reports' `**Start Commit:**` lines, the
README's `**Branch:**` line and this phase's review, whose `scope.base` and `scope.end` are hashes
of the old history.
