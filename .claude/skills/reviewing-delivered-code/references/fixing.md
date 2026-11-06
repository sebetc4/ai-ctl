# Fixing A Review's Findings

How the session fixes the batches the triage gave: the method each fix follows, then each
batch's steps. A fix that holds for the finding's example alone, or guards one path among
several, is the defect the next review finds.

## The Method

1. **Sibling paths.** Before writing the fix, name every other path to the same effect —
   the legacy, batch, retry, cancellation and error paths, and the other commands that do
   the same thing, as a Ctrl-C beside a failed request — and guard each one the same
   way, or write why not in the finding's `note`. Compare sibling controls for
   equivalence, not only for presence: a check on IPv4 alone beside one on both stacks is
   two controls. A fix to what the code makes — a recipe, a file's format, a default —
   leaves what it made before, such as a build already installed: name that too, and
   guard it on each part of how it was made, not the part the finding names alone.
2. **Input classes.** Name the class of inputs the defect belongs to, and make the tests
   cover the class, not the finding's example: every form of an address, the boundary
   numbers and their digit counts, a path holding a space, a quote or bytes that are not
   UTF-8, the empty input.
3. **A document's statements are claims.** A fix that writes or keeps a statement — in
   `docs/`, a doc comment, a help text or a message — checks each one against the code or
   by a run before the commit, as a test checks the code.
4. **Mutants the tool does not write.** `cargo mutants` writes none for most of what a fix
   hinges on. Apply each of these by hand to the fix's own code: an `if` guard forced true
   or false, a `let … else` taking its `else`, a `?` swallowing its error, a match arm
   removed, a lock not taken, a clamp removed, a truncation removed.
5. **New tests stable.** Run each new test that binds a port, takes a lock or runs two
   processes at once 30 times before its commit: one run cannot show a flaky test. A test
   that only runs a command in a scratch folder runs once.

## Each Batch

In the batches' order, one commit each:

1. For each finding of the batch with a `repro/<n>/test.patch`, apply it with `git apply`
   and run its test: it must fail on its assertion, not on a build error. A finding
   whose `test` is `none:` has none.
2. Fix it by the method, starting from `repro/<n>/fix.patch` when there is one: the
   verifier's settled fix, not yet checked against the sibling paths and the input class.
3. Run the checks of `.agent-conventions.toml`; each one passes.
4. Write the batch's diff, `git diff HEAD > <scratch>/batch.diff`, and run
   `TMPDIR=<scratch> cargo mutants --in-diff <scratch>/batch.diff`. Kill each mutant it
   missed with a test, or write in the finding's `note` why no test can.
5. Apply each mutant of the method's fourth point in turn, watch its test fail, and undo
   it. Record it in `docs/reviews/delivered-code/mutants.json`, made with
   `{"mutants": []}` when missing: its `id`, the next `M<n>`; its `kind`; its `file`; its
   `text`, which must occur once in the file, and its `replacement`; its `test`, the
   libtest path that kills it; the `review` and `finding` it comes from. A recorded
   mutant whose `text` the batch rewrote is recorded again on the new text, its `id`
   kept.
6. Commit through the `committing-changes` skill, then write the commit into each finding:
   `scripts/findings.py set <folder> <n>,<n> fixed_in=<hash>`. Run the whole record at
   that commit, `TMPDIR=<scratch> scripts/regression.py`, not the batch's mutants alone,
   since a batch that rewrites an earlier mutant's line or renames its test leaves it
   `gone` or `missed` unseen; record each such mutant again on its new line or test. Any
   other `missed` is a finding of the session's.

The batch of the findings below the floor, `fix at once`, is one commit by the same steps,
with no red test to apply. An update within `Cargo.toml`'s requirements is one commit too:
`cargo update -p <crate> ...`, the checks passing.

The last fix's commit removes the review's `repro/` with `git rm -r`: its tests then live in
`tests/` and the `#[cfg(test)]` modules, and git keeps the rest.
