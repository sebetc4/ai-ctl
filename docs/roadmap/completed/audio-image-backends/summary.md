# Summary — Audio And Image Backends

---

## Where We Started

On 2026-10-19, ai-ctl served LLMs only. `docs/gpu/llama-swap.md` had made llama-swap the one
arbiter of the GPU so that audio and image backends could join without a controller of their
own, and the `v2-review-fixes` roadmap had left reviewed code to build on. The code was ready in
its structure, not in its content: `BackendKind` knew `llama-server` alone, the builder held
llama.cpp's recipe alone, the key reached the backends through a variable only llama-server read,
and the bench spoke the chat API only.

The roadmap was to add whisper.cpp's `whisper-server`, audio.cpp's `audiocpp_server` and
stable-diffusion.cpp's `sd-server` with llama-server's guarantees: a native build ai-ctl keeps,
an inventory that checks every value, the gateway's key, and `doctor`, each fact probed on this
machine.

---

## Where We Landed

The shipped inventory serves 13 models on four backends: two chat models on `llama-server`;
whisper large-v3 and large-v3-turbo on `whisper-server`; Qwen3-ASR and five speech entries,
Qwen3-TTS and Fish S2 Pro at two precisions each and Voxtral, on `audiocpp_server`; Qwen-Image 2.1, FLUX.3
Image Klein and Fibo 2.0 Dev on `sd-server`. Each is pinned, verified and measured, and served
side by side with the others of its use, so that the one suited to a need is chosen.

`ai-ctl build` builds the four projects with one builder, each recipe carrying its tag pattern,
submodules, smoke test and patches. No server could require a key: each gets a patch of ai-ctl's
own, applied to every build, which makes it refuse every route but its health check without the
key and drop the CORS headers that let any page read its answers. `whisper-server` and
`sd-server` bind as `llama-server` does, and `sd-server` fails loudly on a port it cannot bind.

Every model answers through the gateway with the key and gets llama-swap's 401 without it. The
matrix sizes every kind by its measured budget, the swaps follow it, and `down` leaves no
process and no GPU memory. OpenWebUI transcribes, reads aloud, makes and edits images through the
gateway, and `docs/openwebui.md` says how to set it. The image models keep their weights in RAM,
so that every size up to `sd-server`'s 4096 a side passes, each budgeted at its largest
request.

---

## What Each Phase Delivered

**Phase 0, Probes And Decisions.** Each server built by hand for `sm_120a`, run alone and behind
llama-swap v263, its options, routes, readiness and key written to `docs/audio/` and
`docs/image/`, its captures to `tests/fixtures/`. The provenance rule for models other than LLM
GGUF files went to `docs/stack.md` §5. The user replaced "one model per use" with several per
use, and the comparisons on the user's own recordings and prompts dropped no model. The key
patches were prototyped, and audio.cpp kept whole.

**Phase 1, Native Builds.** A recipe per project, three tag patterns, submodules by path, the
patches embedded in the binary and recorded in `build.json` format 2, and
`build list | use | prune` and `doctor` across the four projects. stable-diffusion.cpp moved from
`master-975` to `master-1024` within the phase: the first move broke the patches, the second the
configure, on the CUTLASS submodule its ggml fork added, which the builder now fetches.

**Phase 2, Inventory And Gateway.** The three kinds, named after their binaries, with their
fields, reserved flags and defaults; each kind's llama-swap entry with `/health` as readiness and
the key through `env:`; audio.cpp's configuration written by `up` and carried in the YAML; every
check of the inventory and of the configuration listed together. Fibo 2.0 Dev joined the image
models, and `doctor` checks the shards of a safetensors index.

**Phase 3, End To End.** The shipped inventory served live, the swaps against the 27B, OpenWebUI
wired for the three uses, and `docs/openwebui.md`. The live runs corrected the budgets the
doubles could not test: the cloning models measured again with a 40 s reference, the image models'
weights moved to RAM, Qwen-Image budgeted at its edit's peak. Image editing was added through the
gateway and from OpenWebUI.

**Phase 4, Image Web UI**, added on 2026-10-30 for the editing OpenWebUI lacks (a mask, init and
reference images, the seed, the steps, the strength), was opened and its sources read the same
day, then removed by the user, image editing not being a priority. Its reading is recorded at
commit `docs(audio-image-backends): record the reading of sdcpp-webui` of 2026-10-30: through the gateway, a browser could load the page only by typing the key into
llama-swap's Basic prompt, and the UI's model selector, polling every 2 s, would start the image
model again after each swap.

---

## What We Learned

- **No server of the three could require a key**, upstream included. A patch of ai-ctl's own per
  server, checked on its port and behind the gateway, was the price of the guarantee
  `docs/security.md` gives; it also removed CORS headers that let any page read the answers.
- **Upstream's pace is the builder's main risk.** Two releases of stable-diffusion.cpp in one
  phase broke a patch, then the configure. A patch that no longer applies stops the build before
  the compile and keeps the current one.
- **Budgets measured on a short input are wrong on a real one.** Fish S2 Pro failed beside two
  other speech models with the user's 40 s reference, after phase 0 measured it with 11 s; the
  image models' budgets changed three times as the sizes clients ask became known.
- **A probe settles what the sources suggest.** `sd-server` exiting 0 on a port it could not bind,
  cpp-httplib's `SO_REUSEPORT` sharing a live port, llama-swap's routing by `model` and its 404
  without one: each came from a run on this machine, not from reading.
- **The first real client finds what no probe did**: whisper guessing the language of a short
  dictation, a model id typed with a leading space, the chat models reading an attached image on
  the CPU for minutes because the image model evicted them.
- **One client's need goes to its own setting first.** OpenWebUI's image size and attachment
  compression stayed in OpenWebUI rather than in the shared inventory.

---

## What We Are Leaving Open

- **The review of this roadmap's code**, by the next version of `reviewing-delivered-code`, as its
  first real use: the next work.
- **sd.cpp's own web UI**, phase 4, postponed by the user on 2026-10-30, its plan and reading at
  commit `docs(audio-image-backends): record the reading of sdcpp-webui` of 2026-10-30.
- **4K through `sd-server`'s ESRGAN upscale**, asked on 2026-10-29 and not probed.
- **The edits' quality**: FLUX.3's edits unrealistic, Qwen-Image's colours shifted, a matter of
  the models.
- **`laure-fr`'s transcript**, for the user to correct in `/ai/models/voices/prompt_text`.
- **Phase 0's open items**: Qwen3-TTS with an English reference, Fish S2 Pro BF16 with its inline
  tags, Qwen3-ASR from OpenWebUI while audio.cpp transcribes WAV only.
- **`doctor`'s ok line** says "1 files" for a model of one file.
- **Out of this roadmap's scope**: a bench of audio and image; Voxtral through vLLM-Omni and Fish
  Speech, Python backends that need a new kind of source; official container images of the new
  servers.
