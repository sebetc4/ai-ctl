# Documentation

What ai-ctl is and why, by domain. Each document describes the code and the machine as they are
now; how they got there is in the roadmaps, the reviews and git's history.

---

## Documents

| Document | What it holds | Read it before |
|---|---|---|
| [architecture.md](architecture.md) | Layers, dependency rule, ports, errors, concurrency, testing, repository layout | Changing the structure of the code |
| [audio/audio-cpp.md](audio/audio-cpp.md) | audio.cpp's `audiocpp_server`: build, configuration, routes, readiness, the key, its text-to-speech and speech-to-text models, OpenWebUI's fit, with sources | Touching audio.cpp, its recipe, its kind or its models |
| [audio/whisper-cpp.md](audio/whisper-cpp.md) | whisper.cpp's `whisper-server`: build, options, routes, readiness, the key, its models, OpenWebUI's fit, with sources | Touching whisper.cpp, its recipe, its kind or its models |
| [commands.md](commands.md) | Every command: arguments, options, defaults, output, exit status, files | Changing a command, and change it with the command |
| [gpu/llama-swap.md](gpu/llama-swap.md) | How the GPU is shared: llama-swap as the only arbiter, ai-ctl and the clients around it, the backends served and planned, several models per use, and why | Touching a backend, the gateway or VRAM |
| [image/stable-diffusion-cpp.md](image/stable-diffusion-cpp.md) | stable-diffusion.cpp's `sd-server`: build, options, routes, readiness, the key, its image models, OpenWebUI's fit, with sources | Touching stable-diffusion.cpp, its recipe, its kind or its models |
| [openwebui.md](openwebui.md) | OpenWebUI, the client for daily use: its container, the key, and the settings of its chat, speech and images, step by step | Running or setting OpenWebUI |
| [release.md](release.md) | A release: the tag, the workflow that checks, builds and publishes it, what it pins and trusts, how each pin is renewed, and how production installs it | Touching the workflow or one of its pins, making a release or installing one |
| [security.md](security.md) | Who can reach the gateway and the backends; what the API key, loopback, `expose` and CORS guard; what ai-ctl runs and keeps | Touching the key, the gateway's address, the inventory's checks, a download or an image |
| [stack.md](stack.md) | The LLM stack: engine, models, settings, clients, measurement protocol, with sources | Changing an engine, a model, a setting or a client |
| [toolchain.md](toolchain.md) | Driver, CUDA toolkit and compilers: fresh install, version changes, and the record of every command run on this machine | Changing the toolchain, and add an entry to its record |

## Records

- [roadmap/](roadmap/) — the roadmaps, their phases and reports; `completed/` holds each closed
  one with its `summary.md`, which says what it left open.
- [reviews/delivered-code/](reviews/delivered-code/) — the reviews of delivered code, each ending
  with its fix plan.

## Outside `docs/`

- [deploy/](../deploy/) — the shipped inventory and the OpenCode configuration.
- [tests/fixtures/README.md](../tests/fixtures/README.md) — each captured output the parsers are
  tested on, with its capture.

---

## Rules

- **One domain, one place.** A fact lives in the document of its domain; the others link to it.
- **A decision lives in its domain.** It is written in the document of that domain, with its date,
  the options set aside, the facts checked and their sources, not in a folder of decisions apart.
- **A document says what is true now.** A change to the code changes its document in the same
  commit; the history stays in the roadmaps and in git.
- **Working notes are not documentation.** A question under discussion stays out of `docs/` until
  it is settled, and no document cites a working note.
- **The index follows the documents.** A new document gets its line here, in the same commit.
