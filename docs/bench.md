# Measuring the LLM Stack

How ai-ctl's LLM stack is measured: what a comparison holds fixed, the measures and how
`ai-ctl bench run` takes them through the gateway, and how its GPU and memory readings read. The
engine, the models and their settings are in [stack.md](stack.md); the command's arguments, its
output and its saved runs in [commands.md](commands.md#bench).

---

## What Is Compared

Compare **engine + model + quantization** pairs, never engines alone: a Q4_K_M GGUF and an NVFP4
checkpoint are neither the same weights nor the same quality. The same goes for runtimes: compare a
native build and an image on the same model and flags.

## The Measures

| Measure | How | Why |
|---|---|---|
| Generation, 1 request | tok/s at 8K, 32K and 64K of context and at the context's limit | The real sequential case |
| Prefill | tok/s on a fresh 40K prompt | The cost of a new agent |
| Cache effect | TTFT at turn 1 against turns 5–10 of one session | Checks the reuse (`timings.cache_n`) |
| MTP | Throughput with `spec-draft-n-max` from 1 to 4, acceptance rate | Find the best setting |
| VRAM | Peak from NVML + `--fit` logs | Nothing may be moved to the CPU |
| Stability | 1 h of continuous agentic session | Laptop thermals, leaks, crashes |
| Quality | Success rate on 10–20 real tasks, % of valid tool calls and JSON | The only criterion that counts in the end |

`/metrics` needs llama-server to run with `--metrics`, which ai-ctl adds to every entry. Through the
gateway, it is reached at `/upstream/<model>/metrics`, like `/tokenize` and `/slots`.

## What the Bench Takes

`ai-ctl bench run` measures the first five through the gateway, the MTP setting being an edit of
the inventory and one run per value; the `--fit` logs are read in the gateway's log. A request that
must miss the prompt cache sends `cache_prompt: false`, which makes llama-server evaluate the prompt
whole (`n_past = 0` in `tools/server/server-context.cpp`), so that two runs can send the same
prompts. Such a request comes after an unmeasured one, the same prompt cut to a quarter. With
`--cache-ram`, a slot given a prompt that keeps less than half of its context first saves that
context into the RAM cache, unless it is there already, and one given a prompt that shares no more
than 10 % with its own (`--slot-prompt-similarity`) also saves it and may load a cached context in
its place (`get_available_slot` in the same file). Left in the measure, the save cost 0.9 to 2.6 s
to the first repetition of a scenario on the 27B and nothing to the next ones. The quarter pays it,
and leaves in the slot a prompt the fresh one shares a quarter with and keeps whole, so that the
fresh request neither saves nor loads a context. Stability and quality are outside the bench.

## Reading the GPU

The GPU's readings keep NVML's clock-event reasons whole. On this machine (driver 615.78.08,
run `20261016T194820Z-qwen3.8-27b`, 1,742 readings), every reading holds one reason: SwPowerCap
(0x4) or Reliability (0x400), never both, and never BoardLimit (0x200), heat or the hardware.
Reliability is what NVML reports when the power limit does not hold the clocks: at idle (4.5 W,
no gateway), between requests (below 80 W), and for about half a second at a time under load,
at 150 to 171 W where the capped readings reach 173 W, the SM clock then at 0.98 of the capped
readings around it (0.83 to 1.07). So "slowed by" leaves it out, and the "policy limit" column
gives its share apart: 3 to 8 % of the depth, prefill and session scenarios, 35 % of the
overhead scenario, whose short requests leave the GPU waiting.

The memory read is the device's, every process on the GPU included. A browser whose GPU process
runs on the NVIDIA GPU adds its own: on 2026-11-14, Firefox's started during a run of the 27B and
took 93 to 250 MiB, the run's peak 21669 MiB where the 27B alone held 21422. `nvidia-smi` lists
the processes; a model's budget is its peak measured with no other process on the GPU but the
desktop's few MiB.
