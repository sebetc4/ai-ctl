# Phase 4: Bench

---

## Status

**Current Status:** 🟢 Done (100% — 14/14)
**Started:** 2026-10-11
**Completed:** 2026-10-12
**Blocked By:** —

---

## Before Starting This Phase

Read `phase-3-gateway.md` and `phase-3-gateway-report.md` in full before
touching anything here: the decisions already taken, the problems and
deviations recorded, and the changes made to this phase.

---

## While Working

Keep `phase-4-bench-report.md` current as the work happens — after each significant
step, and before every commit, pause, or end of session.

---

## Objective

`ai-ctl bench run` measures a model served through the gateway with the protocol of
`docs/stack.md` — generation speed by context depth, fresh prefill, prompt-cache reuse across
turns, MTP acceptance, peak VRAM — plus the gateway's own overhead, and `ai-ctl bench compare` sets saved runs side by side.

---

## Overview

### Why This Phase Matters
llama-benchy and GuideLLM already measure depths and prefill on any endpoint. What none of them
covers for this setup is the agent's real pattern: multi-turn sessions measured through
`timings.cache_n`, MTP acceptance read from `/metrics`, and the build or image each run used. The
bench is also what decides between builds, official images, presets and, one day, vLLM.

### What It Enables
Inventory choices (`spec-draft-n-max`, KV cache type, context size, VRAM budgets) and build choices
backed by numbers, and the prefill measurement that would justify a vLLM roadmap.

### Out of Scope
Model quality evaluation (task success rate, valid tool calls). Automatic sweeps that rewrite the
inventory. Concurrency and load testing. Audio and image benchmarks. A one-hour stability run.
Speed differences between llama-server's APIs (same engine, same slot). vLLM and
models larger than the VRAM: directions for later roadmaps.

---

## Tasks

### Domain
- [x] Model scenarios (context depth, fresh prefill, multi-turn session) and results that combine client timings (TTFT, total) with the server's `timings` (`prompt_n`, `cache_n`, `prompt_per_second`, `predicted_per_second`)
- [x] Port v1's aggregates (mean, median, p99, standard deviation) from the archive and extend them, with unit tests

### Client
- [x] Streaming chat client through the gateway, over the HTTP/1.0 `TcpStream` client ai-ctl already uses for llama-swap, that records TTFT at the first content or reasoning token, the time of every chunk after it (inter-token latency), and keeps the final `timings` and `usage` objects, porting v1's SSE parser
- [x] Build prompts of exact token depths through `/upstream/<model>/tokenize`, with a unique prefix when a run must not hit the prompt cache

### Scenarios
- [x] Generation speed at 8K, 32K and 64K tokens of context and at the context's limit — the server's `n_ctx`, read from `/upstream/<model>/props`, less the tokens to generate — skipping a depth beyond the limit; the context is prefilled once, so the repetitions measure generation only
- [x] Prefill speed on a fresh 40K-token prompt
- [x] Multi-turn session of 10 turns, sending each turn's reasoning back in the history as OpenCode does, with TTFT and `cache_n` per turn
- [x] MTP acceptance rate per request from `timings.draft_n` and `draft_n_accepted`, checked against the `/upstream/<model>/metrics` counters read before and after a run
- [x] Peak VRAM, with the GPU's temperature, SM clock and power, sampled through NVML during a run, so a measure taken on a throttled GPU shows
- [x] Gateway overhead: the same short request sent alternately through llama-swap and straight to the model's `proxy` address read from `/running`, with the difference in TTFT and total time

### Results
- [x] Save each run as JSON in the data directory, with the inventory entry, its runtime (build id, or image tag and digest), its flags and the machine facts
- [x] `bench compare`: one table per scenario across saved runs

### Baseline
- [x] Run a baseline with the main model (Qwen3.8-27B) on the native build and on the official image, record both in the phase report, and document `bench` in the README
- [x] Check OpenCode and OpenWebUI against `127.0.0.1:8080`, OpenCode's tool calls through the 27B's template included, with `deploy/opencode.json` installed in place of the v1 configuration (phase 3's fifth acceptance criterion, moved here); and check that `/v1/messages` and `/v1/responses` stream through the gateway, as `docs/stack.md` §7.1 tells the orchestrator

---

## Technical Details

### Files to Modify
```
Cargo.toml
README.md
assets/bench/               (the prompts' corpus)
src/domain/                 (scenarios, results, aggregates)
src/application/            (bench run, bench compare)
src/infrastructure/         (streaming client, metrics scraper, result store)
src/presentation/           (bench commands, tables)
tests/
```

### Dependencies
Phase 3: a gateway started by ai-ctl, so a run knows the inventory entry and runtime it measured.
The baseline also needs the inventory's models downloaded.

### Constraints
- The bench talks to the gateway: OpenAI-compatible endpoints directly, llama-server-only endpoints (`/tokenize`, `/metrics`) through `/upstream/<model>/`; a scenario that needs a llama-server-only field marks it as optional. Only the gateway-overhead scenario also sends to the address llama-swap gave the model, which it reads from `/running` and never starts.
- Prompts are an agent's at work on ai-ctl's own code, frozen at `feat(bench): stream and time chat completions` of 2026-10-11 in `assets/bench/corpus.txt` (`src/domain/prompt.rs`), and are generated deterministically: two runs of the same scenario send the same tokens; a request that must miss the prompt cache asks `cache_prompt: false`.
- The first request of a run pays the model's load: it is excluded from the measurements.
- Each measure is repeated, 5 times by default and more where it narrows the spread; a run's duration is not a constraint. A p99 is reported only over enough samples, inter-token latencies first.
- Native build against official image, or one setting against another, is a change of the inventory entry and `ai-ctl up`, then one run each and `bench compare`: the bench rewrites nothing.

---

## Acceptance Criteria

- [x] `ai-ctl bench run` runs every scenario through the gateway and saves one JSON file
- [x] The multi-turn scenario shows `cache_n` growing from the second turn on
- [x] `ai-ctl bench compare` sets two saved runs side by side
- [x] The baseline compares the native build with the official image and is recorded in the phase report
- [x] The three checks pass
