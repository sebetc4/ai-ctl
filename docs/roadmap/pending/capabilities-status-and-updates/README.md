# Roadmap: capabilities-status-and-updates

Writes each model's capabilities into the generated llama-swap configuration, adds `status --json`, and a command that finds the backends' builds behind their newest release and builds them, on llama-swap v270, toward 0.3.0.

---

## Status Indicators

- 🔴 Not Started
- 🟡 In Progress
- 🟢 Done
- ⏸️ Blocked
- ⚠️ Needs Review

---

## Overall Progress

```
Phase 0  Decisions And Probes         🔴 ░░░░░░░░░░░░░░░░░░░░   0%  (0/5)
Phase 1  llama-swap v270              🔴 ░░░░░░░░░░░░░░░░░░░░   0%  (0/7)
Phase 2  Capabilities                 🔴 ░░░░░░░░░░░░░░░░░░░░   0%  (0/5)
Phase 3  status --json                🔴 ░░░░░░░░░░░░░░░░░░░░   0%  (0/5)
Phase 4  Builds Behind Their Release  🔴 ░░░░░░░░░░░░░░░░░░░░   0%  (0/8)
Phase 5  Closing Pass                 🔴 ░░░░░░░░░░░░░░░░░░░░   0%  (0/3)
TOTAL                                    ░░░░░░░░░░░░░░░░░░░░   0%  (0/33)
```

**Current Phase:** —
**Blocked By:** —

---

## Why This Roadmap Exists

A client that delegates tasks to the local models should pick a model by what it can do, with no
list of models of its own. llama-swap v263 publishes `capabilities` per model in `/v1/models`, but
discovers them from llama-server only after a model's first load, keeps them in memory alone, and
reads nothing from the image, speech and transcription servers; ai-ctl writes none. The inventory
knows each model's kind and its `mmproj`, which is enough to declare them before any load.

`status` renders text only, while `doctor` already has `--json`: a script or a client reading the
gateway's state has to parse prose, or settle for the exit code.

`ai-ctl build <project>` builds the newest release by default, keeps the build it replaces and
refuses a release one of its patches does not apply to; but nothing tells which current builds are
behind. The closing pass of roadmap `fresh-history-and-ci`, on 2026-11-06, found llama.cpp at
b12111 behind b12177, stable-diffusion.cpp 19 tags behind, audio.cpp v0.9.3 behind v0.9.4, and
llama-swap v263 behind v270, no release note read. llama-swap comes first: the capabilities are
probed and written for the release ai-ctl ships, and v263 cannot reach a backend on a unix socket,
the way out of the high finding `docs/security.md` §2 records, so whether v270 can is worth knowing.

The same summary left `Cargo.toml`'s `rust-version`, 1.89, checked by no build.

---

## Decisions Taken At Opening

Taken with the user on 2026-11-06:

- **Name:** `capabilities-status-and-updates`. **Purpose:** ai-ctl's part of the agents' client —
  the capabilities —, `status --json`, and the builds kept at their newest release, on llama-swap
  v270, released as 0.3.0.
- **llama-swap first:** its pin moves to v270 before the capabilities, so that they are probed and
  written once, for the release shipped.
- **`build outdated`:** read-only, over the network, for the projects the inventory uses: each
  current build beside its project's newest release, a build its recipe no longer matches named,
  and an exit code of its own when one is behind. It stays out of `doctor`, which remains local
  and fast. Its exact output and code are phase 0's.
- **`build update [PROJECT]`:** builds the newest release of each project behind, one after the
  other; a failure, such as a patch that no longer applies, is reported and the others go on. Like
  `build`, the new build becomes the current one, the running gateway keeps its builds until
  `down` then `up`, and the build it replaces stays for `build use`. Set aside: building without
  switching, the switch made by hand after a `bench compare`.
- **llama-swap in `build outdated`, never in `build update`:** its archive is pinned by SHA-256 in
  ai-ctl's code, renewed by a release of ai-ctl once its notes are read
  ([security.md §7](../../../security.md#llama-swap-releases)).
- **0.3.0:** the entry opens in phase 1 with `Cargo.toml` at 0.3.0; the release follows the
  roadmap's integration, a tag being made on `main`, as 0.2.0's did.
- **Reviews:** each phase that changes code ends with the review of its hunks; phase 5 is the
  closing pass, and no review after it.
- **Branch:** `roadmap/capabilities-status-and-updates`, made at the first opening.

---

## Deliberately Out Of Scope

- A new version of the review skill: a roadmap of its own after this one, once this roadmap's
  reviews have given it failures to answer.
- Models installed but switched off in the inventory.
- A desktop panel reading `status --json`.
- The agents' client itself, a separate project: ai-ctl stays no client.
- Moving the backends onto unix sockets, even if v270 reaches one: phase 0 records the fact, a
  later roadmap the change.
- An update run by a timer or a unit: the user runs `build update`.
- Another platform than Fedora 44.

---

## Phases

- [Phase 0: Decisions And Probes](phase-0-decisions-and-probes.md)
- [Phase 1: llama-swap v270](phase-1-llama-swap-v270.md)
- [Phase 2: Capabilities](phase-2-capabilities.md)
- [Phase 3: status --json](phase-3-status-json.md)
- [Phase 4: Builds Behind Their Release](phase-4-builds-behind-their-release.md)
- [Phase 5: Closing Pass](phase-5-closing-pass.md)

---

## Dependencies

- Each phase builds on the previous one; phases 2 to 4 on phase 0's Design.
- The agents' client, a separate project, needs phase 2's capabilities before its own second
  phase.
- The reviews start paid sessions, each batch after the user's yes; phase 4's builds take the GPU
  and several minutes each.

---

## Related Documentation

- [docs/gpu/llama-swap.md](../../../gpu/llama-swap.md) — the gateway, its clients, the backends
- [docs/commands.md](../../../commands.md) — `build`, `gateway`, `status`
- [docs/security.md](../../../security.md) — what listens, the key, the pinned downloads
- [docs/stack.md](../../../stack.md) — the server and gateway arguments, §6.4
- roadmap `fresh-history-and-ci`: `summary.md` — the lag of llama-swap and the backends, `rust-version`

---

## Metadata

**Roadmap Status:** 🔴 Not Started
**Created:** 2026-11-06
