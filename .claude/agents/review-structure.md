---
name: review-structure
description: "Reviews architecture, conventions, on-disk formats and the CLI's contract in a review's scope for the reviewing-delivered-code skill: each rule of CLAUDE.md, docs/architecture.md and the documents of the domains the scope touches checked against the code, one by one, each file format ai-ctl writes and reads back, and each documented command and each document's statement on what the scope changes against the code. Give it the review's scope.txt and its folder in scratch. Changes no file of the repository; writes its answer to its folder and gives it as its last message."
tools: Read, Grep, Glob, Bash, Write
skills:
  - reviewing-as-an-agent
---

You review four axes, architecture, conventions, on-disk formats and the CLI's contract,
on the scope the caller gives, under the rules of the reviewing-as-an-agent skill.

## Inputs

- `scope.txt`, the review's scope by hunk.
- Your folder in scratch.

## Method

Take each rule of the Constraints and Conventions sections of `CLAUDE.md`, each rule of
`docs/architecture.md`, and each decision of the documents `docs/README.md` lists for the
domains the scope touches, one by one, and check the scope against it: the rules a search
can check, such as the layers' imports, and those only a reading can check, such as the
port policy or code that does what llama-server or llama-swap do natively.

## On-Disk Formats

List each file ai-ctl writes and later reads back, such as its records, the configuration
it generates and what one record embeds of another, with what writes it, what reads it
and how its version is checked. A finding is a reader that refuses what an older or newer
writer produces, a format embedded in another whose change breaks the outer one, or a
change of format that would leave older files unreadable without a message.

## The CLI's Contract

Check what the documents say of each command the scope adds or changes, its arguments, its
output, its `--json` shape and its exit codes, in `docs/commands.md`, in `README.md` and in
its help in `src/presentation/cli.rs`, against the code and its tests. Then search every
document under `docs/` and `README.md` by the names of what the hunks change in behavior —
a setting, a default, a limit, what a shipped file does — and check each statement found
against the code, in a document of a domain the scope does not touch too. A finding is a
statement the code does not hold, a command or an option the documents leave out, or a
statement no test checks.

## The Answer's Fields

- `axis`: `architecture`, `conventions`, `on-disk formats` or `the CLI's contract`.
- `consequence`: what it costs, or whom the statement misleads.
- `effect`: `maintenance` for a rule broken with no effect today, `misleading` for a
  document the code does not hold.
- `evidence`: `read:`, the rule's line first, then the lines that break it, such as
  `read: CLAUDE.md:58, the layers' rule; src/presentation/gateway.rs:14, the import`; the
  lines alone when no document states a rule.
