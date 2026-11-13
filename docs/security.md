# Security

Who can reach the gateway and the backends ai-ctl runs, what guards them and what does not, and
what ai-ctl runs on the user's behalf. Every fact below was checked on this machine on 2026-10-16
unless dated otherwise, with llama-swap v263, llama-server `b11538`, podman 5.8.7 and OpenCode
1.19.0. The behaviour of llama-swap and llama-server is recorded in
[stack.md §7](stack.md#7-client-configuration); this document says what follows from it.

The boundary is the user's account. ai-ctl guards the gateway and the backends against other
users, web pages, containers and the network. It does not guard them against code that runs as
the user, which can read everything ai-ctl reads.

---

## 1. At a Glance

| Who | Reaches | What stops them |
|---|---|---|
| Another user of the machine | The gateway's port and every backend's port, on loopback | The API key, in a file and in process environments only the user can read; not on a backend's port taken before the backend binds it (§2), nor on the gateway's port taken while the gateway is down (§3) |
| A web page open in the user's browser | The same ports, through the requests the browser sends for it | The API key. CORS does not (§5) |
| A container | The host's loopback, when its network leads there | The API key |
| The network: the LAN, and over IPv6 whoever routes to the machine's global address | Nothing while `listen` is a loopback address. With `expose = true`, the gateway on `listen`'s addresses, every IPv4 and IPv6 address of the machine for `0.0.0.0` (§4) | Loopback. With `expose = true`, the key alone, sent in clear, once the firewall lets the port in (§4) |
| Code that runs as the user | Everything | Nothing: it can read the key (§3) |
| Whoever writes the inventory, a release, a source tree, a model or an image ai-ctl runs, or pushes a tag of ai-ctl | Code run as the user | The checks of §7, which narrow who can change what runs, not what it does |

---

## 2. What Listens

- **The gateway**, llama-swap, on `gateway.listen`: `127.0.0.1:8080` by default. Every route
  answers 401 without the key, its web UI and everything under `/api/` included, except
  `/health` and `/wol-health`, the same handler, which tell only whether a gateway runs,
  `/favicon.ico`, the web UI's manifest and two icons, the redirects of `GET /` and `/ui` to
  `/ui/` and of `GET /upstream` to the UI's list of models, and the CORS preflight, an `OPTIONS`
  on any route answered 204 before the key is asked: each tells only that a gateway runs, as
  `/health` does. Those three redirects and the preflight read in v271's route registrations and
  probed on v271 on 2026-11-12: `GET /` 302, `GET /ui` 308, `GET /upstream` 302,
  `OPTIONS /v1/chat/completions` 204, `/ui/` itself 401. Probed again on v270 on 2026-11-07,
  the routes v264 to v270 added among them: `/audiocpp/`, `/v1/realtime`, `/api/mcp`,
  `/api/events`, `/api/models/pause` and `/api/models/resume` answer 401 without the key;
  `/wol-health` answers 200, on v263 as well. On v271, probed on 2026-11-08, `/v1/realtime` also
  takes a websocket with no `?model=`, its handshake answered by llama-swap itself behind the
  key until a `session.update` names the model: 401 without the key, with `?model=`,
  `?intent=transcription` or neither; the routes above answer as on v270.
- **The backends**, llama-server, on `127.0.0.1:${PORT}`, at ports llama-swap assigns from
  5800. Every route answers 401 without the key except `/health`, `/v1/health` and the files of
  the web UI a llama-server embeds, which ai-ctl's build leaves out (§7). ai-ctl passes
  `--host 127.0.0.1` and reserves the flags `host` and
  `port`, so no inventory moves a backend off loopback, `expose` or not. An image backend runs
  with `--network host` and binds the same address
  ([stack.md §4.4](stack.md#44-native-build-or-official-image)).
- **The audio and image backends**, `whisper-server`, `sd-server` and `audiocpp_server`, on the
  same address and ports, from ai-ctl's builds only: an inventory cannot run them from an image.
  Each requires the key by ai-ctl's patch (§7), every route but `/health` answering 401 without
  it. ai-ctl passes the loopback address and reserves the flags that move it or the routes:
  `host`, `port`, `inference-path` and `request-path` for `whisper-server`; `listen-ip` and
  `listen-port` for `sd-server`; `host`, `port`, `config` and the UI's flags for
  `audiocpp_server`, whose configuration ai-ctl writes under
  `~/.local/state/ai-ctl/gateway/audiocpp/`, in a directory only the user can read.

Loopback keeps the network out, not the machine's other users: every local process reaches it.

**A backend's port taken first.** Each model has a port from 5800, fixed when the gateway starts
([gpu/llama-swap.md](gpu/llama-swap.md#what-llama-swap-offers)); its backend binds it when the
model starts on demand, and frees it when its `ttl` unloads it. A process that binds that port
before the backend makes llama-server fail to bind and exit, but only after llama-swap's first
health check: on 2026-10-18, llama-server `b11700` with the GPU visible exited 0.40 to 0.87 s
after its start on a taken port, the official image 0.76 to 1.39 s, and llama-swap checks
0.25 s after the start. llama-swap takes the
squatter's 200 for the backend and forwards it every request for the model, the client's
`Authorization` header included: the squatter gets the key and the prompts, and writes the
answers, tool calls for OpenCode's agents among them. There is no race to win: the squatter won
three rounds out of three for each runtime, end to end. ai-ctl cannot close this while llama-swap
reaches its backends over TCP only and takes any 200 on `/health` for the backend
([stack.md §6.4](stack.md#64-server-and-gateway-arguments)). A `startPort` in the configuration
would not help: a configured port is as easy to find as the default, by reading the configuration
or by scanning loopback. The audio and image servers are exposed the same way. Each exits 1 with
a message on a taken port: `whisper-server` and `sd-server` by ai-ctl's patches (§7),
`audiocpp_server` by its own code. Each binds only once its model is loaded, which gives a
squatter the whole load. What protects the backends is who else can run a process on the machine:
this one has one login account.

---

## 3. The API Key

### Where It Lives

The first `ai-ctl up` that starts the gateway creates the key: 32 random bytes in hexadecimal, in
`~/.local/state/ai-ctl/api-key` (under `$XDG_STATE_HOME` when that is set), mode 0600, written
whole or not at all. Later runs keep it. `up` passes it to llama-swap in the variable
`AI_CTL_API_KEY`. The configuration reads it as `apiKeys: ["${env.AI_CTL_API_KEY}"]` and gives it
to each backend as `LLAMA_API_KEY`, with `--env LLAMA_API_KEY` for a container. Therefore:

- no configuration, command line or log holds the key: `gateway config` prints the configuration
  as `up` writes it, and `ps` shows no key;
- a process's environment, `/proc/<pid>/environ`, is readable by its owner only (mode 0400);
- llama-swap refuses to start without the variable, so a gateway never runs open by mistake;
- the gateway and its backends keep the key `up` started them with, whatever happens to the file
  afterwards.

To change the key, remove the file, then run `ai-ctl down` and `ai-ctl up`; with the gateway down,
remove the file before `ai-ctl up`. Clients that read the file get the new key the next time they
read it; OpenWebUI needs it pasted again in its five settings (§6).

**One key guards the gateway and every backend.** llama-swap forwards the client's
`Authorization` or `x-api-key` header to the backend unchanged, so a backend started with the
gateway's key accepts every request that passed the gateway, `/upstream/<model>/…` included. A
key on the gateway alone would leave each loaded backend open on its port, where llama-server's
CORS allows every origin. There is one key, not one per backend: llama-swap cannot rewrite the
header for each model, and a client holds one key. The flags `api-key` and `api-key-file` are
reserved: a second key would open a backend to it, and `--api-key` shows on the command line to
every user.

### What It Guards

Every request to the gateway or to a backend except the health checks and the static files of §2:
chat and completion requests, the audio and image requests, `/upstream/<model>/…`, `/running`,
`/logs`, `/api/models/unload`, the pause and resume of every model, the event stream, the Help
agent's `/api/mcp`, the realtime websockets, the web UI, `/metrics`, `/slots` and `/props`, and
the audio and image servers' own routes, `sd-server`'s `/sdcpp/v1/…` among them. Checked against the gateway `ai-ctl up` started,
without the key, with no `Origin` header and with `Origin: https://attacker.example` alike, and on
the backend's port.

Before the key, any local process or web page could load a model, evict the one in use, read the
log stream and use the GPU (finding 1 of the review of 2026-10-15).

### What It Does Not Guard

- **Code that runs as the user.** It can read the key file, llama-swap's environment and
  OpenWebUI's volume, and needs none of them to stop the gateway or kill a backend. An agent's
  shell tool, a build script and a package's install hook all run as the user. ai-ctl has no
  defence there.
- **A browser that stored the key from llama-swap's login prompt.** llama-swap also takes the
  key as HTTP Basic credentials, and asks for them with `WWW-Authenticate: Basic` on every 401. A
  browser that keeps them attaches them to the requests a page sends to the gateway: a form posted
  to `/api/models/unload` with `Origin: https://attacker.example` and the key as Basic credentials
  answered 200. **Never type the key into that prompt in a browser used for browsing.** ai-ctl
  and the clients of §6 never need it.
- **The gateway's port while the gateway is down.** Between `ai-ctl down` and `ai-ctl up`,
  `127.0.0.1:8080` is free, and any account can bind it, the port being above 1024. The clients
  keep sending the key there: OpenCode reads the file and sends it as a bearer token, and
  OpenWebUI does the same; the agents' client holds it back until `ai-ctl status --json` says the
  gateway runs at a loopback address and the client has itself checked that the PID it gives,
  `gateway.pid`, holds the listening socket, which `status` does not look at. A process
  that holds the port gets the key, which
  stays valid after the next `up`, and the prompts sent while it holds it, and writes their
  answers. It needs no race: the gateway is down in normal use. `up` refuses a taken address, and
  its error says to change the key when the program that holds it is unknown. A program that
  takes the address between that check and llama-swap's bind is not taken for the gateway
  either: `up` counts an answer only from the socket of the llama-swap it started, found in
  `/proc/net/tcp` or `tcp6` and the process's descriptors; on `0.0.0.0`, that socket is the
  dual-stack `[::]` Go opens in its place, and `up` checks that the port is free on both. ai-ctl
  does not change the key at each `up`: OpenWebUI would need it pasted again each time.
- **The network.** With `expose = true`, the key and every prompt and answer cross the network in
  plain HTTP: ai-ctl configures no TLS. Whoever can watch the traffic takes the key.
- **What a key holder does.** There is one key and no roles: every client that holds it can load
  and evict models and read the log stream. It can pause the gateway, `POST /api/models/pause`
  (v268): every model unloaded and every inference request refused with 503 until a resume or
  the gateway's restart. It can read the running configuration through `/api/mcp`, each model's
  command and paths, with the variables named like a key redacted. It can also empty an `sd-server` of its model through
  `/upstream/<model>/sdcpp/v1/models/unload`, and the model fails until llama-swap stops it
  ([stable-diffusion-cpp.md §2](image/stable-diffusion-cpp.md#2-serving)). It can have the
  speech servers read files of the host by their path, as the user: `audiocpp_server` reads the
  audio a JSON transcription request or a `voice_ref` names, and answers its transcription or
  the speech in that voice
  ([audio-cpp.md §3](audio/audio-cpp.md#3-serving)); `whisper-server`'s `POST /load` loads a
  model from a path, says when nothing is there, and ends when the file fails to load
  ([whisper-cpp.md §2](audio/whisper-cpp.md#2-serving)). OpenWebUI's container holds the key
  (§6): what runs in it can send these requests.

---

## 4. Loopback and `expose`

- A non-loopback `gateway.listen` is refused unless `[gateway]` sets `expose = true`.
  `listen = "0.0.0.0:8080"` alone fails with "gateway.listen `0.0.0.0:8080` is not a loopback
  address, so the gateway would serve the network; set `expose = true` under [gateway] …". Any
  address of `127.0.0.0/8` and `[::1]` count as loopback.
- `expose` opens the gateway only; the backends stay on loopback (§2).
- `0.0.0.0` and `[::]` open the same: llama-swap, a Go program, listens on either through one
  dual-stack socket, `*:8080` in `ss -ltn`, which serves every IPv4 and every IPv6 address of the
  machine, its global IPv6 addresses included (llama-swap v263, checked on 2026-10-19 in a network
  namespace). A global IPv6 address may be reached from beyond the LAN where the router lets IPv6
  in, and this machine has one on its Wi-Fi interface. To serve one network only, give that
  interface's own address, such as `192.168.1.20:8080`, which nothing else answers on.
- An exposed gateway still needs the key on each request, sent in clear (§3).
- ai-ctl does not touch the firewall. On this machine, the active zone, `FedoraWorkstation` on the
  Wi-Fi interface, allows only `dhcpv6-client`: an exposed gateway would also need a port opened,
  which firewalld opens to IPv4 and IPv6 alike.
- A client that holds the key and listens on the network opens the gateway to whoever reaches the
  client, whatever `listen` says: see OpenWebUI in §6.

---

## 5. CORS

llama-swap answers every origin with `Access-Control-Allow-Origin: *`, and its preflight allows
the `Authorization` header. llama-server, started without a key, allows every origin and warns
about it at start. ai-ctl leaves both as they are, on purpose:

- **`allowedOrigins` is not access control**, as llama-swap's own guide *CORS and browser access*,
  embedded in v263, puts it. A request from an origin it does not list "is still handled
  normally", only without CORS headers. A plain cross-origin `GET` sends no preflight, "so the
  work happens and the result is thrown away"; so does a form `POST`. Restricting origins would
  hide the answers from pages, not stop the requests.
- llama-swap strips the CORS headers an upstream sets on proxied responses, so a backend's own
  CORS setting changes nothing through the gateway. An empty `allowedOrigins` is refused at
  startup.
- **The key is what refuses a page's request**: without it, a page gets 401, whatever its origin.
  DNS rebinding changes nothing either. llama-swap checks no `Host` header, but a rebound page
  still lacks the key.

---

## 6. Clients That Hold the Key

| Client | Gets the key | Where the key sits |
|---|---|---|
| ai-ctl | Reads the file at each request; its health checks send none | ai-ctl's file |
| OpenCode | `"apiKey": "{file:~/.local/state/ai-ctl/api-key}"` in `opencode.json` ([stack.md §7.2](stack.md#72-opencode)) | ai-ctl's file; OpenCode's configuration holds no key |
| The agents' client | Reads the file at each request, and sends the key once `ai-ctl status --json` says the gateway runs at a loopback address and the client has itself checked that the PID it gives holds the listening socket ([stack.md §7.1](stack.md#71-the-agents-client)) | ai-ctl's file |
| OpenWebUI | Pasted into five settings: its OpenAI connection, speech to text, text to speech, image generation and image editing ([openwebui.md §3](openwebui.md#3-the-key)) | Its volume, `open-webui` |

A client that holds the key passes the gateway on to whoever uses the client. OpenCode's tools
and the agents that run the agents' client run as the user already (§3). OpenWebUI serves a web page: anyone
who reaches it and logs in uses the gateway through it.

### OpenWebUI on Loopback

Publish OpenWebUI's container on loopback only, `-p 127.0.0.1:18000:8080`, as
[openwebui.md §1](openwebui.md#1-the-container) gives it. podman's `-p 18000:8080` publishes it on every
IPv4 and IPv6 address, `*:18000` in `ss -ltn` (rootless podman 5.8.7 with pasta, checked on
2026-10-19): that gives the network a door to the gateway past `listen` and `expose`,
with OpenWebUI's own accounts as the only guard. On 2026-10-16, the container on this machine was
found created that way; the firewall kept the LAN out, and the container was recreated on
loopback with the same image and volume. `ss -ltn` shows the address port 18000 listens on.

The container reaches the gateway at `http://10.0.2.2:8080/v1` through slirp4netns's
`allow_host_loopback`, which opens the host's whole loopback to it: the gateway, the backends and
every other local service. The key guards the first two.

### The `unauthorized` Lines in the Log

Each model load leaves two or three warnings in the gateway's log, right after the backend's
health check:

```
W srv    operator(): unauthorized: Invalid API Key
```

They are llama-swap's own `GET /v1/models` to the backend, which carries no key. llama-swap
ignores the 401, and the model shows `ready`. They are harmless, and left as llama-swap does it.
A client's request without the right key gets its 401 from llama-swap and never reaches a
backend, so such a line at any other moment than a model load comes from a request sent straight
to a backend's port: worth a look.

---

## 7. What ai-ctl Runs

Everything below runs as the user, on the host or in a rootless container. Each check narrows who
can change what runs; none makes untrusted content safe to run.

### The Inventory

The inventory decides what llama-swap runs: each model becomes a `cmd` that llama-swap runs on the
host when a request or a preload starts the model. ai-ctl writes each word of that command quoted for llama-swap's
shell-like split, inside a YAML literal block, and refuses a value that contains:

- `${`, which llama-swap reads as a macro, with no escape;
- a control character;
- U+2028 or U+2029, which llama-swap's YAML parser reads as line ends. Such a character in a flag
  value, invisible or written as a TOML `\u2028` escape, ended the `cmd` block and added a model
  of the value's own, whose command llama-swap ran on the host (finding 3 of the review,
  reproduced end to end).

With these refused, a value cannot leave its word. The same holds for every text an
`audiocpp_server` model's configuration carries, which the YAML holds too. What the inventory
decides by design stays: the flags each server gets, among them flags that name files it reads,
such as `chat-template-file`, and the images the models run. Read an inventory someone else wrote as a
script before running it. `ai-ctl init` never overwrites the installed one.

### llama-swap Releases

`ai-ctl gateway install` installs the release ai-ctl pins, v271, and no other: llama-swap is
updated only through ai-ctl's pin. It downloads the release's archive from llama-swap's GitHub
releases, over HTTPS only, redirects included, and checks its SHA-256 against the digest ai-ctl
carries for that release, the only source of trust. That digest was read in the release's
published checksums when it was pinned, and found on the archive downloaded: trusted on first use,
then frozen. A mismatch says the archive is "corrupt or was tampered with". The release's
directory records the digest its archive was checked against, `archive.sha256`: the pinned
release installed already against the pinned digest becomes current again without a download;
one with no record or another digest, as an older ai-ctl's `--version` left it, is downloaded and
checked again, and `up` refuses to run it until then. A gateway started from it before keeps
running the binary replaced: `status` shows its configuration outdated, and `doctor` and `up` name
`ai-ctl down`, then `ai-ctl up`. `up` refuses as well, and `doctor` fails on,
a release behind the pin or ahead of it whose directory records no check: only an ai-ctl that pins
a release records its check, so one an older ai-ctl's `--version` installed against the release's
own checksums or a digest the user gave is refused, and one ahead of the pin runs once the ai-ctl
that pins it installed it. The binary runs at once (`-version`), then as the gateway.

### Build Sources

`ai-ctl build` clones a project's repository, `https://github.com/ggml-org/llama.cpp.git`,
`https://github.com/ggml-org/whisper.cpp.git`, `https://github.com/leejet/stable-diffusion.cpp.git`
or `https://github.com/0xShug0/audio.cpp.git`, fetches its tags with `--force`, drops those the
repository no longer holds, such as a release its project withdrew, resolves `latest` to the
newest release tag and compiles that commit. stable-diffusion.cpp's `ggml`
submodule comes from the repository its `.gitmodules` names at that commit,
`https://github.com/leejet/ggml.git` at `master-1071-cfdf3cf`, at the commit its tree records, and
so does NVIDIA's CUTLASS, `https://github.com/NVIDIA/cutlass.git` (`v3.9.2`), whose headers are
compiled into `sd-server`'s CUDA kernels: a submodule of the project itself, `thirdparty/cutlass`,
since 2026-11-01, and before that of the fork, `src/ggml-cuda/vendors/cutlass`. No signature is checked; the build's `build.json` records the commit. A tag names whatever the
repository's owners make it name, and `--ref <commit>` builds exactly the commit given. The build
runs CMake and the compilers as the user, and its server serves every model.

`ai-ctl build outdated` reads each repository's tags with `git ls-remote`, and runs and writes
nothing. `ai-ctl build update` builds each project behind at the newest release tag, by the steps
above, one project after the other with no question between them: whoever can move a project's
newest tag chooses what it compiles, as with `build`, and it adds no check of its own. The user
runs it; no timer or unit does. llama-swap is not among what it builds.

Before the compile, the recipe's patches change the sources: `build.json` names them, and the
build keeps a copy of each in its `patches/`. A release one of them no longer applies to is
refused, so that no server is built without what its patch adds. A build made before its recipe
gained a patch, or set one of its CMake options or changed its value, such as llama.cpp's
`LLAMA_USE_PREBUILT_UI=OFF`, is not built as its recipe says: `doctor` reports such a current
build, failing when the inventory runs its project, and `up` and `gateway config` refuse it when
a model runs on it, each with `ai-ctl build <project> --ref <id> --force`, or `ai-ctl build
<project>` at the newest release when the project withdrew `<id>`. The options ai-ctl
sets for every project, such as the compilers' paths, are not compared, nor an option a recipe
no longer sets. A patch is matched by its name: one rebased onto a newer release keeps its name
and its change, and one whose change differs takes a new name.

Decided: a build compared with its recipe by its patches' names and its recipe's CMake options
(the user, 2026-11-06, in the closing pass of `review-skill-v2`). Set aside: each patch's
content compared with the build's copy, which refuses a build of an older release made as
intended, the patch rebased for a newer one no longer applying to it.

llama.cpp is built with `LLAMA_USE_PREBUILT_UI=OFF`, so `llama-server` has no web UI. Left on,
its build downloads the UI from the Hugging Face bucket `ggml-org/llama-ui`, falling back to the
bucket's mutable `latest` when the release's files are missing, checks it only against a digest
from the same place, and compiles each file's name into the server's code and each file's
content, a symbolic link followed, into the binary: content outside the commit `build.json`
records. b12111, built on 2026-11-04, took `latest` that way (its `build.log`). No other
recipe downloads at build time: stable-diffusion.cpp's frontend is off, and the builds of
whisper.cpp and audio.cpp fetch nothing.

Decided: no web UI in llama-server (the user, 2026-11-05, in the fixes of
`audio-image-backends`' review). Set aside: the UI from an archive whose SHA-256 the recipe
pins, that digest having no source but the bucket itself. The flag brings the UI back the day
a user needs it.

- whisper.cpp's `api-key.patch` makes `whisper-server` require the gateway's key, which it cannot
  do on its own ([whisper-cpp.md §3](audio/whisper-cpp.md#3-the-key)). The server refuses to
  start without `LLAMA_API_KEY`. It answers 401 to every request but its health check that sends
  neither `Authorization: Bearer <key>` nor `X-Api-Key: <key>`, and it no longer sends
  `Access-Control-Allow-Origin: *`, which let any page read its answers.
- stable-diffusion.cpp's `api-key.patch` does the same for `sd-server`
  ([stable-diffusion-cpp.md §3](image/stable-diffusion-cpp.md#3-the-key)), whose check replaces the
  handler that echoed any `Origin` with credentials allowed; its health check is a `GET /health` the
  patch adds, so that `/v1/models` stays guarded. Its `listen-failure.patch` makes a port it cannot
  bind end the server with 1 and a message, where it exited 0 without a word, so that llama-swap
  sees the failure.
- audio.cpp's `api-key.patch` does the same for `audiocpp_server`
  ([audio-cpp.md §4](audio/audio-cpp.md#4-the-key)), at the top of the dispatch both its runtimes
  share, `GET /health` exempt. The server ran a `text/plain` `POST`, which a page sends without a
  preflight; without the key, such a request now gets 401.
- whisper.cpp's and stable-diffusion.cpp's `reuse-address.patch` make their server bind with
  `SO_REUSEADDR` alone, as llama-server does, in place of their cpp-httplib's `SO_REUSEPORT`
  ([whisper-cpp.md §2](audio/whisper-cpp.md#2-serving),
  [stable-diffusion-cpp.md §2](image/stable-diffusion-cpp.md#2-serving)). That option let a
  second server of the user bind a port the first still listened on and take a share of its
  connections, the key with them, without a word. A port another process listens on now stops
  the server with 1, and a port in TIME-WAIT no longer does.

### Models

ai-ctl neither downloads nor verifies models (user's decision, 2026-10-10, in
[stack.md §5](stack.md#where-models-come-from)). A GGUF carries the chat template llama-server
applies to every request, and a poisoned template changes the answers while the weights stay
untouched. Doctor's remedy for a missing model file is
`hf download <repo> <files> --revision <commit> --local-dir <dir>`. `<commit>` stays a placeholder
for a commit the user pins: a shell refuses the line as it stands. The remedy points to "Where
Models Come From" in `docs/stack.md` for the SHA-256 and template checks. The same section holds
the rule for models other than LLM GGUF files: from their publisher, failing that from their
engine project's conversions.

### Container Images

An image backend runs under rootless `podman run` with `--network host`,
`--security-opt label=disable`, the GPU, the models directory mounted read-only, and
`LLAMA_API_KEY` in its environment ([stack.md §4.4](stack.md#44-native-build-or-official-image)).
An image is therefore code that holds the key and reaches every service on the host's loopback.

- An image reference may carry a digest, `<image>:<tag>@sha256:<digest>`. podman then runs the
  digest's content, whatever the tag names now. The shipped inventory pins both images,
  `server-cuda-b11541` and `server-cuda13-b11541`, by the digest of their linux/amd64 manifest.
- `ai-ctl up` warns of each image a model runs that is named by its tag only.
- `up` pulls a missing image before it starts the gateway, which runs the image with
  `--pull=never`: a tag is resolved once, at the first pull.

ai-ctl checks no digest itself: for a comparison bench, the warning on a bare tag is enough. Every
shipped model runs the native build, so the shipped inventory pulls nothing.

OpenWebUI's container, which ai-ctl does not run, holds the key too (§6):
[openwebui.md §1](openwebui.md#1-the-container) gives its command with the image pinned the same
way, by the digest of its linux/amd64 manifest.

### ai-ctl Itself

Since 2026-11-06, production runs the binary of a release, which the workflow
`.github/workflows/release.yml` built and checked at its tag, installed by `gh release download`
with its `SHA256SUMS` checked ([release.md §5](release.md#5-installing-a-release)). Whoever can
push to the public repository `sebetc4/ai-ctl` a tag the release's guard accepts publishes a
binary that runs as the user: the guard checks that the tag names `Cargo.toml`'s version and a
dated entry, not who pushed it. The `SHA256SUMS` comes from the same release: it detects a
corrupt download, not a replaced release. `gh`'s token lies in the GNOME keyring, which code
run as the user reads, as it uses the SSH key in the agent. Its scopes are `repo`, `read:org`
and `gist`, `gh`'s minimum, and `admin:public_key`, which the login added with SSH as git
protocol (2026-11-06): that scope adds an SSH key to the account, a way in that outlives the
agent's key, and `gh auth refresh --remove-scopes admin:public_key` drops it; `gh auth status`
lists them.

The CI runs the repository's code, its tests and the review skill's evals on a GitHub-hosted
runner, in Fedora 44's image pinned by digest, with a token that reads the repository alone. The
job that publishes runs no code of the repository, only `gh` on the build's artifact, its sum
checked again ([release.md §4](release.md#4-what-it-trusts)).

---

## 8. Files ai-ctl Keeps

| File | Mode | Holds |
|---|---|---|
| `~/.local/state/ai-ctl/` | 0700 | The state: every file below but the inventory |
| `~/.local/state/ai-ctl/api-key` | 0600 | The key |
| `~/.local/state/ai-ctl/gateway/gateway.json` | 0644 | The running gateway's record: PID, binary, address, no secret; removed when it stops |
| `~/.local/state/ai-ctl/gateway/.lock` | 0644 | The lock `up`, `down`, `status`, `build prune` and `build --force` take, and what its last holder did, no secret |
| `~/.local/state/ai-ctl/gateway/llama-swap.yaml` | 0644 | The configuration `up` wrote, no secret |
| `~/.local/state/ai-ctl/gateway/audiocpp/<model>.json` | 0644, in a 0700 directory | The configuration an `audiocpp_server` model's server reads, written by `up`: its family, task, mode, path, session options and voices, no secret |
| `~/.local/state/ai-ctl/logs/gateway.log`, `gateway.log.1` | 0644 | The output of llama-swap and its backends, for this start and the previous one |
| `~/.local/state/ai-ctl/logs/build-<project>.log` | 0644 | The last build's output |
| `~/.config/ai-ctl/inventory.toml` | 0644 | The inventory |

ai-ctl creates the state directory, and each directory under it, at mode 0700, so only the user
reads the logs and the configuration, whatever the home directory's or `XDG_STATE_HOME`'s mode.
The files in it take the mode the umask leaves (0644 here), the key's 0600 aside. A state
directory already there keeps its mode: one made by an older ai-ctl, at 0755, is narrowed by hand
with `chmod 700` on it and its `gateway` and `logs` directories (done on this machine on
2026-10-16). At llama-server's default verbosity, the log holds model loads, file paths, timings
and token counts, not prompts or answers. The audio and image servers, as ai-ctl runs them, write
no prompt, answer or transcript either (probed on 2026-10-29, a request of each kind, each prompt
with a marker):

- `whisper-server` writes its load, its timings, and the name of each uploaded file, which the
  client chooses;
- `audiocpp_server` its device, its CUDA graph warm-ups and its address;
- `sd-server` its load, its placement on the GPU, each request's size, seed and timings, and
  progress bars drawn with `\r` and `ESC[K`, which `ai-ctl logs` passes to the terminal as they
  are.

A file name holding a control character does not reach the log. llama-swap cannot read such a
form and answers 404, "no model id could be identified", and `whisper-server` answers 400 to it
through `/upstream/<model>/…`.

The data directory, `~/.local/share/ai-ctl/` (sources, builds, llama-swap releases, bench runs),
and the configuration directory keep the umask's mode: they hold no secret.

---

## Sources

- The review `docs/reviews/delivered-code/2026-10-15-v2-rebuild.md`, findings 1 to 7, and their
  verification runs.
- The review `docs/reviews/delivered-code/2026-10-17-v2-review-fixes.md`, finding 1: a squatter on
  a backend's port, end to end with llama-swap v263 and llama-server `b11700`, natively and as the
  image, measured on 2026-10-18 with the GPU visible.
- The review `docs/reviews/delivered-code/2026-10-19-v2-review-fixes-security-pass.md`, findings 1
  and 5: llama-swap v263's dual-stack socket on `0.0.0.0`, its reach over IPv6, and podman's
  publishing, probed in network namespaces on 2026-10-19.
- Phase 0 of the `v2-review-fixes` roadmap, whose report records each probe and the live check of
  2026-10-16: the gateway `ai-ctl up` started, OpenCode and OpenWebUI through it.
- [stack.md](stack.md) §4.4, §5, §6.4 and §7.
- llama-swap v263's embedded guides *CORS and browser access* and *API keys*, extracted from the
  binary.
- llama-server `b11538`: `tools/server/server-http.cpp` for the public routes, `common/arg.cpp`
  for where the key is read from, `tools/server/server.cpp` for the CORS warning.
- [release.md](release.md), for ai-ctl's own release: the workflow's run by hand on GitHub on
  2026-11-06 and its artifact installed with `gh` 2.97.0 (roadmap `fresh-history-and-ci`, phase 2).
