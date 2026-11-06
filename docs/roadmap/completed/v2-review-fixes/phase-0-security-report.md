# Phase 0 Report: Security

**Phase:** [phase-0-security.md](phase-0-security.md)
**Start Commit:** `docs(v2-review-fixes): create the roadmap` of 2026-10-16

---

## Work Log

### 2026-10-16

Phase opened. It is the roadmap's first phase: no previous report to read and no pending
restructuring to settle. Read before working: the roadmap's README, this phase file, and in
`docs/reviews/delivered-code/2026-10-15-v2-rebuild.md` findings 1 to 7 and the Fix Plan's
security batches 1 to 5; `docs/stack.md` §7 and `deploy/opencode.json`, which the key
decision changes.

What the first task starts from:
- finding 1's verification already ran the installed llama-swap v263 alone on
  `127.0.0.1:18999`: with `apiKeys`, every endpoint but `/health` answers 401 without the key.
  It did not check what llama-swap sends upstream, which the probe settles;
- the llama-server backends listen on `127.0.0.1:${PORT}` and stay open to local processes
  whatever the gateway does;
- the clients hold a fixed key today: OpenCode `"apiKey": "sk-local"`, and "any key" for the
  orchestrator and OpenWebUI in `docs/stack.md` §7. OpenWebUI reaches the gateway from a
  container, through `10.0.2.2`.

Probed the installed llama-swap v263 alone on `127.0.0.1:18999` with `apiKeys: ["k-secret"]`,
in front of a fake upstream in Python that logs each request's headers, no GPU:
- llama-swap takes the key as `Authorization: Bearer` or as `x-api-key`; a wrong or missing
  key gets 401 on `/v1/models`, `/v1/chat/completions` and `/upstream/<model>/…`; `/health`
  answers 200 without it;
- it forwards the client's `Authorization` or `x-api-key` header to the upstream unchanged,
  on `/v1/chat/completions`, `/v1/messages` and `/upstream/<model>/props` alike;
- its own requests to the upstream carry no key: `GET /health`, then `GET /v1/models`;
- in front of an upstream that checks the key as llama-server b11538 does (the same two
  headers, `/health` and `/v1/health` public: `tools/server/server-http.cpp:251-290`),
  every keyed request passes; llama-swap's own `/v1/models` gets 401, which it ignores
  without a log line, and the model shows `ready`; the upstream's port, 5800, refuses a
  request without the key;
- the CORS preflight answers `Access-Control-Allow-Origin: *` and allows the
  `Authorization` header.

llama-server b11538 reads the key from `--api-key`, from `LLAMA_API_KEY`, or from
`--api-key-file` (`common/arg.cpp:3512-3538`). Without a key its CORS allows every origin,
and it warns about it at start (`tools/server/server.cpp:340`). llama-swap v263's
configuration has a per-model `env` and a `cors` section (`strings` on the binary).

OpenCode 1.19.0 resolves `"apiKey": "{file:<absolute path>}"` and sends the file's content
as `Authorization: Bearer`, checked against the fake upstream with `opencode run`.

Put the probe's results and three proposals to the user, who approved all three; recorded
under Decisions. The probe's facts went to `docs/stack.md` §7. The key's path and the
clients' settings go there with batch 1, which creates the key: written now, they would
describe a file that does not exist yet.

Batch 1 started. The user approved two choices for it: `gateway config` hides the key, and a
non-loopback `listen` needs `expose = true` under `[gateway]`. Reading the code for it,
`gateway config` prints the configuration `up` writes, so the key would sit in that file and
in the comparison `status` makes with it. Probed whether llama-swap could read the key from
its environment instead, with the fake upstream:
- `apiKeys: ["${env.PROBE_GATEWAY_KEY}"]` with the variable set: 200 with the key, 401
  without it or with the literal macro. Without the variable, llama-swap refuses to start:
  "environment variable 'PROBE_GATEWAY_KEY' is not set";
- a model's `env: ["LLAMA_API_KEY=${env.AI_CTL_API_KEY}"]` gives the backend the gateway's
  key: through the gateway 200, on port 5800 directly 401 without the key and 200 with it;
- `podman run --env LLAMA_API_KEY`, with no value, takes the variable from podman's
  environment into the container, and sets nothing when it is unset (checked with the
  `server-cuda13-b11541` image, entrypoint `/usr/bin/env`, no network);
- llama-server takes `LLAMA_API_KEY` as `--api-key` (`common/arg.cpp:3511-3521`), a
  comma-separated list.

OpenCode 1.19.0 expands `~` in `{file:~/…}` and drops the file's final newline, checked the
same way as the absolute path.

The user asked why CORS is not restricted as well, quoting an analysis that recommends
`security.cors.allowedOrigins` for llama-swap and `--cors-origins localhost` for
llama-server. llama-swap v263 embeds its own guides, extracted from the binary:
- *CORS and browser access*: "`allowedOrigins` is not access control". A request from an
  unlisted origin "is still handled normally", only without CORS headers. A plain
  cross-origin `GET` has no preflight, "so the work happens and the result is thrown
  away". llama-swap strips the CORS headers an upstream such as llama-server sets on
  proxied responses. An empty `allowedOrigins` is refused at startup;
- *API keys*: `apiKeys` covers the web UI and everything under `/api/`. Only
  `/favicon.ico`, the UI's manifest and its two icons stay open. A client may present the
  key as Bearer, `x-api-key` or HTTP Basic. The guide recommends `${env.VAR}` macros to keep
  keys out of the file.

A probe with the fake upstream confirmed them. Without the key, `/ui/`, `/api/events`,
`/api/version` and `/metrics` answer 401 with `WWW-Authenticate: Basic realm="llama-swap"`,
and `/favicon.ico` answers 200. With the key as Basic credentials, a form `POST
/api/models/unload` carrying `Origin: https://attacker.example` answers 200. So a browser
that has stored the key from llama-swap's Basic prompt could attach it to a request a page
triggers, and no CORS setting would stop that request from running. Reported to the user
with the recommendation to leave CORS out of scope, as the phase file has it, and to add
that caveat to `docs/stack.md` §7. The user agreed, and asked for a security document
once the phase is done, so that none of this is lost: added to phase 1.

Batch 1 written test-first. Each new test failed on its assertion before the code was written:
- **Domain.**
  - The configuration writes `apiKeys: ["${env.AI_CTL_API_KEY}"]` after `logToStdout`, each
    llama-server entry `env: ["LLAMA_API_KEY=${env.AI_CTL_API_KEY}"]`, and a container's
    `podman run` `--env LLAMA_API_KEY`.
  - `api-key` and `api-key-file` are reserved flags.
  - `Gateway.expose` is new. A non-loopback `listen` without it is
    `InventoryError::ExposedListen`, for IPv4 and IPv6 alike; `127.0.0.2` and `[::1]` are
    loopback.
- **The key file.** `write_new` moved from the inventory store to `files.rs`, with a mode,
  and publishes the key file at 0600. `KeyFile` either reads the key, trimmed, or creates 32
  random bytes in hexadecimal. Read refuses, with the remedy, an empty key, whitespace, a
  comma, non-ASCII, and a link to nothing in the key's place.
- **Starting llama-swap.** `LocalGateway::start` reads or creates the key before it writes
  anything, then runs llama-swap with `AI_CTL_API_KEY`. An unusable key starts nothing and
  records nothing.
- **The clients.** `http::send` and `request` take the key as a bearer token.
  `LlamaSwapApi`, `LlamaServerApi` and `OpenAiChat` read it from the file at each call;
  `healthy` sends none. With no file they send no header, which suits a gateway an older
  ai-ctl started.
- **End to end.** `tests/gateway_lifecycle.rs` runs against the installed llama-swap v263:
  - without the key, `/v1/models`, `/running`, `/logs`, `POST /api/models/unload` and a chat
    request answer 401, and `/health` 200;
  - with the key, the chat request answers 200;
  - the model's port, 5800, answers 401 without the key and 200 with it.

  Two mutants were applied in place and restored: one removes the `apiKeys` line, the other
  empties the backends' `env`. Each fails the test, and the message names the step that
  broke.
- **Snapshots.** Rewritten, and the diff read: `apiKeys`, each entry's `env`, and the image's
  `--env`. `llama-swap -validate` accepts both files with the variable set, and refuses them
  without it: "environment variable 'AI_CTL_API_KEY' is not set".
- **Documents.** `deploy/opencode.json` reads the key with
  `{file:~/.local/state/ai-ctl/api-key}`. `deploy/inventory.toml`, `README.md`,
  `docs/stack.md` §6.4 and §7, and `docs/architecture.md` are updated.

The five checks are green. The phase's live checks, `up` on this machine, OpenCode and
OpenWebUI, wait for the user's go-ahead: they create the real key, and OpenWebUI needs it
pasted. The user agreed to batches 2 to 5 first.

Batch 2: `Text::checked` refuses U+2028 and U+2029 beside `char::is_control`, under the same
error, whose message now reads "contains a control character or a line separator". The domain
test, finding 3's flag with either separator and one in a file name, failed first on the
generated configuration that held the flag. `tests/gateway_config.rs` adds the reproduction
end to end: the shipped inventory with the flag written as a TOML `\u2028` escape in
`[defaults.llama-server]`, through the real parser and generator. It passes, and fails
under the mutant that drops the check of the separators.

Batch 3: `Installer::install` takes an optional `Sha256`, given by `gateway install --sha256`.
`Event::Verified` names its `DigestSource`, one of `Pinned`, `Given` or `Published`, and each
renders on its own:
- `Pinned`: "SHA-256 verified against the digest ai-ctl pins";
- `Given`: "SHA-256 verified against the digest given";
- `Published`: "SHA-256 equal to the release's own checksums, which detect a corrupt
  download only; --sha256 checks it against a digest you trust".

A mismatch with the published checksums says "the download is corrupt". A mismatch with the
pinned or the given digest keeps "corrupt or was tampered with". A digest given replaces the
checksums file, which is then not read. A release installed already gets a warning that the
digest given is not checked. The four use-case tests and the rendering test failed first on
their assertions. The CLI test parses `--sha256` and refuses an upper-case digest with
`Sha256`'s message. The README gives the option and says what the published checksums prove.

Batch 4: doctor's remedy for a missing model file of a model with a `repo` is now `hf download
<repo> <files> --revision <commit> --local-dir <dir>, at a commit you pin, then check the files
as docs/stack.md says under "Where Models Come From"`. `<commit>` stays a placeholder: the
inventory records no revision, and a shell refuses the line as it stands, since `<commit` reads
as a redirection from a missing file. `model_check` is unchanged. The two remedy tests failed
first on the old text. The README's doctor table says the same.

Batch 5. Probed podman 5.8.7 first, with the local `server-cuda13-b11541` image, no network
and no GPU:
- `image exists` and `run --pull=never` take `<image>:<tag>@sha256:<digest>`, the digest alone,
  the index's digest, and a tag the image no longer has: the digest decides;
- an unknown digest gives "image not known".

`skopeo inspect --raw` read the two indexes from GHCR. The linux/amd64 manifest of
`server-cuda13-b11541` is `sha256:42e80867…`, the digest the bench recorded, and its index
`sha256:222d8eee…`. The linux/amd64 manifest of `server-cuda-b11541` is `sha256:e1483fa3…`;
that digest was read from the index, not pulled.

The code:
- `domain::inventory::is_pinned` accepts a reference that ends in `@sha256:` and a digest
  `Sha256::parse` accepts;
- `up` emits a warning for each image a model runs that is named by its tag only: "the image
  <image> is named by its tag, which its registry can move to other content; pin it in the
  inventory as <image>@sha256:<digest>";
- the shipped inventory names both images by `server-cuda[13]-b11541@sha256:<amd64 digest>`,
  with a comment on how to pin another build;
- `docs/stack.md` §4.4 and the README say the same.

The `is_pinned` and warning tests failed first on their assertions. The test that a pinned
reference reaches the configuration as it is passed on the old code, since nothing refused
`@`: it guards the acceptance. The image snapshot changed on that line only, and `llama-swap
-validate` accepts it.

All six tasks of the phase are ticked. The acceptance criteria that need this machine wait for
the user: `up`, the 401s against the real gateway, OpenCode and OpenWebUI.

Three criteria were checked with the debug binary's `gateway config`, the user's installed
inventory copied into a scratch `XDG_CONFIG_HOME`, nothing started or written in the state:
- `listen = "0.0.0.0:8080"`: exit 1, "gateway.listen `0.0.0.0:8080` is not a loopback address,
  so the gateway would serve the network; set `expose = true` under [gateway] …";
- with `expose = true` as well: exit 0, the configuration printed with its `apiKeys`;
- finding 3's flag in `[defaults.llama-server]`: exit 1, "cannot configure the gateway: model
  `qwen3.8-27b`: … contains a control character or a line separator".

The five checks are green at `feat(gateway): pin container images by digest and warn of a bare tag`. The installed inventory still names its images by
tag. `init` never overwrites it, so updating it is the user's call; no model runs on an image
now.

Session closed at `docs(v2-review-fixes): record phase 0's acceptance checks made without a gateway`, with the tree clean. Two points of the session are not recorded
above:
- at the user's request, the session wrote the agent setup's
  `notes/design-feedback/2026-10-16-first-opening-leaves-roadmap-pending.md`. It says that a
  first opening should move the folder to `on-progress/`, set `**Roadmap Status:**`, and
  update the instruction file, which the skill's opening does not do today. The roadmap
  therefore stays in `pending/` until this phase's closure moves it;
- no ai-ctl is installed on the PATH. The user runs `target/release/ai-ctl`, which predates
  this phase's commits. The installed inventory preloads a model (`preload = true`), so `up`
  takes about 22 GB of VRAM at once.

**Where the next session starts:**
1. With the user's go-ahead, the live acceptance checks:
   - `cargo build --release`, then `target/release/ai-ctl up`, which creates the real key in
     `~/.local/state/ai-ctl/api-key`;
   - `curl` with and without an `Origin` header: `/v1/models`, `/running`, `/logs` and
     `POST /api/models/unload` answer 401 without the key, and `/health` answers 200;
   - `ai-ctl status` and `ai-ctl logs`;
   - OpenCode with `"apiKey": "{file:~/.local/state/ai-ctl/api-key}"`, then OpenWebUI with
     the key pasted into its connection, a chat in each;
   - `ai-ctl down`.

   Then tick the two remaining acceptance criteria.
2. Close phase 0 with the roadmap skill. Every task is ticked. The closure moves the folder
   to `on-progress/`.
3. Open phase 1. Its first task is `docs/security.md`, from this report.
- Put to the user: whether the session edits `~/.config/opencode/opencode.json` itself or the
  user copies `deploy/opencode.json`; whether to pin the images in the installed inventory.
- Pending: nothing uncommitted.

Session resumed. The user approved the live checks, and that the session edits
`~/.config/opencode/opencode.json` and pins the installed inventory's images itself. Both files
were backed up in the session's scratch folder first:
- `~/.config/opencode/opencode.json`: `"apiKey": "sk-local"` became
  `"{file:~/.local/state/ai-ctl/api-key}"`, its only difference from `deploy/opencode.json`;
- `~/.config/ai-ctl/inventory.toml`: the two image backends, named `server-cuda` and
  `server-cuda13` without a build, now carry the shipped inventory's references and comment,
  `server-cuda[13]-b11541@sha256:<amd64 digest>`. Only the CUDA 13 image is pulled; no model
  runs on either. The installed inventory still lacks the shipped `vram_margin_mib = 1024`,
  left as it is.

`cargo build --release` at `docs(v2-review-fixes): record where phase 0 resumes`, then `target/release/ai-ctl up`: llama-swap v263 on
`127.0.0.1:8080`, `qwen3.8-27b` preloaded, no image warning since no model runs on an image.
`up` created `~/.local/state/ai-ctl/api-key`: mode 0600, 64 hexadecimal characters and a newline.
The key is in llama-swap's `/proc/<pid>/environ` (mode 0400) and on no command line.

Against that gateway, with `curl`:
- without the key, with no `Origin` and with `Origin: https://attacker.example` alike:
  `/v1/models`, `/running`, `/logs` and `POST /api/models/unload` answer 401, and `/health` 200;
- the old OpenCode key, `sk-local`, gets 401; the real key gets 200 and the model list;
- the backend's port, 5801: `/v1/models`, `/slots`, `/props` and `/metrics` answer 401 without
  the key and 200 with it, `/health` 200 both ways.

`ai-ctl status` (exit 0) shows the gateway, `qwen3.8-27b (ready)` and 21305 MiB in use.
`ai-ctl logs` shows the log; neither log file holds the key. Right after the backend's health
check, its log has three lines `unauthorized: Invalid API Key`, a second apart. These are
llama-swap's own `GET /v1/models`, sent without a key, as the probe of the first task saw. They are
harmless, the model being `ready`, and every model load adds them.

`opencode run -m llamacpp/qwen3.8-27b` with OpenCode 1.19.0 answered through the gateway, with
the key read from the file: `sk-local` would have got 401.

OpenWebUI's container `openwebui` exists and is stopped. It was created with `-p 18000:8080`,
so podman publishes it on `0.0.0.0:18000`, where `docs/stack.md` §7.3 gives
`-p 127.0.0.1:18000:8080`. The firewall's active zone, `FedoraWorkstation` on the Wi-Fi
interface, opens no port and allows only `dhcpv6-client`, so the LAN does not reach it today. But
once the key is pasted in, anyone who reaches OpenWebUI and logs in uses the gateway. Put to the
user before starting it.

The user chose to recreate it as §7.3 says. `podman rm openwebui`, then §7.3's `podman run` with
`--pull=never`: the same image (`24162421…`), the volume `open-webui` kept. Port 18000 now listens
on `127.0.0.1` only. From inside the container, `http://10.0.2.2:8080/v1/models` answers 401
without a key and with `sk-local`. The key is not shown in the conversation: the user copies it
with `wl-copy < ~/.local/state/ai-ctl/api-key` into OpenWebUI's connection.

The user pasted the key into OpenWebUI's connection and chatted. The gateway's log shows the
chat on `qwen3.8-27b`: a short task request, then the answer with an 8137-token prompt. Then the
user picked `qwen3.6-35b-a3b`, and llama-swap swapped the models (`evict=[qwen3.8-27b]`). The
new backend, on port 5800, served three requests with the key, and its log has two
`unauthorized` lines right after its health check, llama-swap's own. The user stopped the last
answer from OpenWebUI. `/running` showed `qwen3.6-35b-a3b` ready with an idle slot.

`target/release/ai-ctl down`: the models unloaded, llama-swap stopped, and 19892 MiB of GPU
memory released in 0.5 s. `status` then exits 3, "Gateway stopped", 15 MiB in use. The key file
stays, mode 0600, for the next `up`.

Every acceptance criterion holds. Closing the phase: the five checks are green.

---

## Decisions

- **One API key guards the gateway and every backend** (approved by the user,
  2026-10-16). llama-swap forwards the client's header unchanged, so a backend started with
  the same key accepts every request that passed the gateway, `/upstream/<model>/…`
  included. A key on the gateway alone would leave each loaded backend open on
  `127.0.0.1:5800` and up, where llama-server's CORS allows every origin: a web page could
  use the model, `/slots`, `/props` and `/metrics` without the key. There is one key, not one
  per backend: llama-swap cannot rewrite the header for each model, and a client holds one
  key. A backend reads it with `--api-key-file`, never with `--api-key`, which would show the
  key in `ps` to every user. An image backend gets the file mounted read-only. Batch 1
  implements it.
- **The key is the file `api-key` in ai-ctl's state directory** (`~/.local/state/ai-ctl/`
  by default), mode 0600 (approved by the user, 2026-10-16). `up` creates it when it is
  missing and keeps it afterwards. Removing the file changes the key at the next `up`.
- **Clients read the key from that file** (approved by the user, 2026-10-16):
  - OpenCode through `"apiKey": "{file:…}"` in `deploy/opencode.json`. Whether OpenCode
    expands `~` in `{file:~/…}` is unchecked, and batch 1 checks it before writing the
    shipped file;
  - OpenWebUI by pasting it once into its connection;
  - the orchestrator from the file.

  No new command. `docs/stack.md` §7 documents the path.
- **The key reaches llama-swap and the backends through the environment, not through a
  file or the configuration**. This replaces the `--api-key-file` of the first decision and
  makes the hiding moot, for the same aims: no key on a command line, and none in a file
  other users can read. `up` starts llama-swap with `AI_CTL_API_KEY` set to the key. The
  configuration says `apiKeys: ["${env.AI_CTL_API_KEY}"]`, and each llama-server entry
  `env: ["LLAMA_API_KEY=${env.AI_CTL_API_KEY}"]`, with `--env LLAMA_API_KEY` in a
  container's `podman run`. Why:
  - the configuration holds no secret, so `gateway config` prints it as it is, and
    `status`'s comparison with the written file is unchanged;
  - the gateway and its backends always hold the key `up` started them with, where a
    backend reading a file at its own start could hold another one;
  - no file to mount into a container;
  - llama-swap refuses to start without the variable, so a gateway never runs open by
    mistake.

  `/proc/<pid>/environ` is readable by its owner only (mode 0400). Removing the key file
  changes the key at the next `up` that starts the gateway. Taken while writing batch 1,
  and reported to the user with it.
- **`gateway.expose`, and `listen` on loopback otherwise** (approved by the user,
  2026-10-16). A non-loopback `listen` is refused unless `[gateway]` sets `expose = true`.
  The key guards every request either way.
- **CORS stays as llama-swap and llama-server set it** (approved by the user, 2026-10-16).
  The phase file already has it out of scope, and llama-swap's own guide now backs that:
  `allowedOrigins` only hides responses from pages; it lets requests run. The key is what
  refuses a page's request. `docs/stack.md` §7 says, with batch 1, never to type the key
  into llama-swap's Basic prompt in a browser used for browsing.
- **A security document follows this phase** (asked by the user, 2026-10-16). It gathers
  what this report records about who reaches the gateway and what guards it. It is the
  first task of phase 1.

---

## Files Changed

**Added**
- `docs/roadmap/on-progress/v2-review-fixes/phase-0-security-report.md`
- `src/infrastructure/api_key.rs`

**Modified**
- `CLAUDE.md`
- `README.md`
- `deploy/inventory.toml`
- `deploy/opencode.json`
- `docs/architecture.md`
- `docs/stack.md`
- `src/application/doctor.rs`
- `src/application/gateway.rs`
- `src/application/lifecycle.rs`
- `src/domain/gateway_config.rs`
- `src/domain/inventory.rs`
- `src/infrastructure/chat_api.rs`
- `src/infrastructure/files.rs`
- `src/infrastructure/gateway_api.rs`
- `src/infrastructure/gateway_process.rs`
- `src/infrastructure/http.rs`
- `src/infrastructure/inventory_file.rs`
- `src/infrastructure/mod.rs`
- `src/infrastructure/paths.rs`
- `src/infrastructure/server_api.rs`
- `src/main.rs`
- `src/presentation/cli.rs`
- `src/presentation/gateway.rs`
- `tests/cli.rs`
- `tests/gateway_config.rs`
- `tests/gateway_lifecycle.rs`
- `tests/snapshots/gateway-shipped-image.yaml`
- `tests/snapshots/gateway-shipped.yaml`

**Renamed**
- `docs/roadmap/pending/v2-review-fixes/README.md → docs/roadmap/on-progress/v2-review-fixes/README.md`, modified as well
- `docs/roadmap/pending/v2-review-fixes/phase-0-security.md → docs/roadmap/on-progress/v2-review-fixes/phase-0-security.md`, modified as well
- `docs/roadmap/pending/v2-review-fixes/phase-1-bugs-and-error-handling.md → docs/roadmap/on-progress/v2-review-fixes/phase-1-bugs-and-error-handling.md`, modified as well
- `docs/roadmap/pending/v2-review-fixes/phase-2-dead-code-and-tests.md → docs/roadmap/on-progress/v2-review-fixes/phase-2-dead-code-and-tests.md`
- `docs/roadmap/pending/v2-review-fixes/phase-3-performance.md → docs/roadmap/on-progress/v2-review-fixes/phase-3-performance.md`
- `docs/roadmap/pending/v2-review-fixes/phase-4-structure-and-refactoring.md → docs/roadmap/on-progress/v2-review-fixes/phase-4-structure-and-refactoring.md`
- `docs/roadmap/pending/v2-review-fixes/phase-5-closing-review.md → docs/roadmap/on-progress/v2-review-fixes/phase-5-closing-review.md`

---

## Problems And Deviations

- **Batch 1 goes beyond finding 1's Fix in two points**, both serving the decision that the
  key guards the backends:
  - the reserved flags `api-key` and `api-key-file`, which would give a backend a second key,
    on its command line;
  - `write_new` moved to `files.rs`, which batch 13 was to do; phase 1 is adjusted.
- **The key reaches llama-swap and the backends through the environment**, where the first
  decision named `--api-key-file` and the user approved hiding the key in `gateway config`.
  Both are replaced; see Decisions.
- **The README shows phase 0 at 20 cells while it is still open.** The skill's Progress bar
  invariant caps an unfinished phase at 19, but `progress.py` prints 20 for 6/6, and
  `progress.py --check` refuses the capped line. The README keeps the script's output, so the
  check stays green. This is the defect ai-ctl reported in the agent setup's
  `notes/design-feedback/2026-10-16-progress-bar-full-while-in-progress.md`, whose fix
  lean-workflow's Phase 2 plans.
- **OpenWebUI's container was published on every interface**, outside ai-ctl. It was created with
  `-p 18000:8080`, where `docs/stack.md` §7.3 gives `-p 127.0.0.1:18000:8080`. The firewall kept
  the LAN out, but once it holds the key, OpenWebUI lets anyone who reaches it and logs in use the
  gateway. Fixed on this machine: the container was recreated as §7.3 says, with the same image and
  volume. The security document of phase 1 says why the loopback binding matters.
- **Every model load leaves `unauthorized: Invalid API Key` lines in the gateway's log**: three
  for `qwen3.8-27b`, two for `qwen3.6-35b-a3b`. They are llama-swap's own `GET /v1/models`, which
  carries no key and whose 401 llama-swap ignores. They are harmless but look like an attack.
  Left as llama-swap does it; the security document of phase 1 says what they are.
- **The installed files were edited by the session**, with the user's approval:
  `~/.config/opencode/opencode.json` (the key read from its file) and
  `~/.config/ai-ctl/inventory.toml` (the images pinned by digest). Neither is in the repository.

---

## Changes To Later Phases

- `phase-1-bugs-and-error-handling.md`:
  - added the task "Write `docs/security.md`" under a new section, Security Document,
    first in its tasks, at the user's request;
  - reworded batch 13. Batch 1 moved `write_new` from the inventory store into
    `files.rs`, with a mode argument, for the API key's file, so batch 13 makes only the
    run store use it.

  Phase 1 has 15 tasks.
- `phase-1-bugs-and-error-handling.md`, at closure: the task "Write `docs/security.md`" also
  covers the clients that hold the key, OpenWebUI's container on loopback first, and the
  `unauthorized` lines llama-swap's own requests leave in the log at each model load. Phase 1
  still has 15 tasks.

---

## Assessment

The phase delivered its five batches and the decision that frames them. The gateway answers
only a client that holds ai-ctl's key, and so do its backends: llama-swap forwards the client's
header unchanged, and each backend gets the same key from the environment. The key lives in one
file, mode 0600, that `up` creates. No configuration, command line or log holds it, and llama-swap
refuses to start without it. A non-loopback `listen` needs `expose = true`. Inventory text cannot
carry a line separator into the configuration. A non-pinned release's check says it detects
corruption only, and `--sha256` checks it against a trusted digest. Doctor's download remedy pins
a revision, and the images are pinned by digest.

Two decisions changed along the way, both after a probe: the backends' key moved from
`--api-key-file` to the environment, and CORS stayed out after llama-swap's own guide showed that
`allowedOrigins` lets requests run. Live on this machine, the key closed every endpoint but
`/health`, on the gateway and the backend alike. OpenCode and OpenWebUI work with it, a model
swap included, and `down` freed the VRAM.

What phase 1 needs first: its first task, `docs/security.md`, gathers this report's account of
who reaches the gateway and what guards it, with the two observations of the live check:
OpenWebUI's container must stay on loopback, and the `unauthorized` lines at each load are
llama-swap's own. Batch 13 now only moves the run store onto `files.rs`'s `write_new`. Batch 43
builds on the bearer token that `http::send` and `request` now take.
