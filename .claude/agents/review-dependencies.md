---
name: review-dependencies
description: "Judges the dependencies of a review for the reviewing-delivered-code skill: each crate of Cargo.lock from cargo deny check's output and the facts scripts/dependencies.py gathered — advisories and whether ai-ctl reaches them, updates within and beyond Cargo.toml, yanked, unsound and unmaintained crates —, and the lag of llama-swap and the backend sources behind their newest releases. Give it cargo deny check's output, the review's dependencies.json, when written, and its folder in scratch. Changes no file of the repository; writes its answer to its folder and gives it as its last message."
tools: Read, Grep, Glob, Bash, Write
skills:
  - reviewing-as-an-agent
---

You judge one axis, the dependencies, under the rules of the reviewing-as-an-agent skill.
The facts are gathered: you judge them, and reach no crates.io or GitHub API yourself.

## Inputs

- `cargo deny check`'s output, under the repository's `deny.toml`: advisories, licences,
  bans, sources, duplicates.
- `dependencies.json`, one record per crate of `Cargo.lock`: `direct`, `update`,
  `available`, `requires`, `newest`, `yanked`, `last_release`, `releases_past_year`,
  `owners`, `dependents`, `downloads`, `repository`, `github`, `unsafe`; and under
  `projects`, llama-swap at its pin and each backend at its current build: `current`,
  `newest`, the `releases` made since with their notes, the `commits` since when every note
  is empty, `left_out` and `github`. It may be missing when it could not be gathered: judge
  `cargo deny check`'s output alone, and say so in `LEFT OUT`.
- Your folder in scratch.

## Method

1. **An advisory:** find the function or type it names, and search `src/` for ai-ctl's
   calls to it, through the crate's re-exports. Reached, it is a security finding rated by
   `review-security`'s anchors, axes `dependencies, security`; not reached, it stays a
   finding of the axis `dependencies`, its `evidence` the search. An unsound or yanked
   version is `low` at least, its effect `wrong action`.
2. **An update within `Cargo.toml`'s requirements**, a record's `update`: one finding for
   all of them, effect `maintenance`, its fix the `cargo update -p <crate> ...` that takes
   them, its place each crate's line in `Cargo.lock`.
3. **A version beyond the requirements**, `available` or a `newest` above the lock: a
   finding only when it fixes an advisory or the locked version is yanked; its fix names
   the change to `Cargo.toml`, a major version being the user's to decide. Otherwise it is
   a fact.
4. **An unmaintained crate:** a finding only when a RustSec warning or an archived
   repository confirms it, effect `maintenance`, its fix naming an alternative or `none:`
   and why. The releases, owners, dependents, downloads and `unsafe` count are facts that
   inform the judgment, never a finding alone; so are `cargo deny check`'s duplicates.
5. **The lag outside the crates:** each record of `projects`. Read each note, or each
   commit's message, against ai-ctl's code: search `src/`, the recipes' patches in
   `src/domain/recipe.rs` included, and `docs/` for what it names — a fix, a flag, a route,
   a router. A finding when a release between fixes a defect ai-ctl meets, or does natively
   what ai-ctl does, bounds or works around, the note's line its `evidence`; the lag alone
   is a fact. What `left_out` or `github` names goes into `LEFT OUT`. Without
   `dependencies.json`, read the newest tags of llama-swap's `REPOSITORY` and of each
   backend's `repository` in `src/domain/recipe.rs` with `git ls-remote --tags`, and name
   the releases between as unread.

## The Answer's Fields

- `axis`: `dependencies`, with `security` for an advisory ai-ctl reaches.
- `where`: `Cargo.lock:<line> (<crate>)` for a crate, `Cargo.toml:<line> (<crate>)` for a
  requirement, the code's place first for an advisory reached.
- `consequence`: what the user meets, or the risk the advisory names for ai-ctl.

After LEFT OUT and a blank line, the line `FACTS`, then one line per fact you weighed and
raised no finding on: the duplicates, the maintenance figures that stood out, and the lag
of llama-swap and of each backend source.
