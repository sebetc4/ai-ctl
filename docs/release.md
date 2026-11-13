# Release

How a release of ai-ctl is made, checked, built and published, what the workflow that does it
pins and trusts, how each pin is renewed, and how production installs a release. Every fact
below was checked on 2026-11-06, by the workflow's steps run in its image under rootless podman
5.8.9 on this machine, and by a run of the workflow on GitHub.

---

## 1. A Release

A release is a tag `vX.Y.Z` on `main`, on the commit where `Cargo.toml` holds `version = "X.Y.Z"`
and `CHANGELOG.md` heads its entry `## X.Y.Z — YYYY-MM-DD`; while in work, the entry reads
`## X.Y.Z — unreleased`. The `committing-changes` skill makes the tag on that commit and pushes
it, by the `[git]` table of `.agent-conventions.toml`; no one tags by hand.

Each release starts from llama-swap's newest release, since ai-ctl pins one release of it, the
only one `ai-ctl gateway install` installs: its notes are read since the pinned release and checked
against its source, `git diff`; each change bearing on the configuration ai-ctl writes or the API
it calls is carried, test-first; the archive's SHA-256 is read in the release's checksums file and
found on the archive downloaded, then written in `src/domain/gateway.rs`; its `-version` output and
checksums file are captured into `tests/fixtures/`; `llama-swap -validate` runs on the shipped
configuration's snapshots, and the gateway runs end to end on this machine, a chat, a
transcription, a speech and an image. v270 took that path on 2026-11-07 and 2026-11-08, v271 on
2026-11-08 (roadmap `capabilities-status-and-updates`, phase 1).

The push of the tag runs the workflow `.github/workflows/release.yml` on GitHub Actions, in the
public repository `sebetc4/ai-ctl`. It checks the tag, checks and builds ai-ctl, and publishes
the binary `ai-ctl` with its `SHA256SUMS` on the tag's release, whose notes are the tag's message,
the version's entry.

---

## 2. The Workflow

**When it runs.** At each pushed tag `v*`, and by hand (`workflow_dispatch`), from any branch or
tag, with `gh workflow run release.yml -R sebetc4/ai-ctl --ref <branch or tag>`. GitHub runs by
hand only a workflow the default branch, `main`, holds, and refuses it otherwise with a 404, "not
found on the default branch"; the run then takes the workflow of the ref it is given. A run by
hand checks and builds alone: it publishes nothing, and leaves the binary and its `SHA256SUMS`
as the run's artifact `ai-ctl`, kept 14 days.

**Where.** A GitHub-hosted `ubuntu-24.04` runner, every step of the build in a Fedora 44 container,
Fedora 44 being ai-ctl's only target: the binary is linked against the glibc it runs on.

**The build job**, which reads the repository and writes nothing:

1. The packages, from Fedora's repositories: `git-core` before the checkout, which without git
   downloads the tree with no history, `gcc` as the linker, `python3`, `tar`, `gzip` and
   `iproute`, whose `ss` the tests of the review skill's guard run.
2. The checkout, its token not kept, and the workspace marked safe for git, the container's
   root not owning it.
3. On a pushed tag, the guard, `.github/scripts/check_release.py`, before anything costly: it
   refuses a tag not of the form `vX.Y.Z`, one that is not `Cargo.toml`'s version, and a version
   whose `CHANGELOG.md` entry is missing, given twice or not dated. A refused tag stops the run.
4. rustup, then the toolchain `rust-toolchain.toml` pins, made the default too: the review
   skill's evals run cargo in folders outside the repository. The install takes
   `--no-self-update`, which keeps the pinned rustup-init: rustup's self-update downloads the
   newest rustup-init of its update root and runs it with no checksum. rustup 1.29.3 skips it
   on its own while `CI` is set, as GitHub sets it, but not in the steps run under podman
   here; the flag holds in both.
5. cargo-deny, as `docs/toolchain.md` records it here.
6. `cargo build --release --locked`; then `ai-ctl gateway install` with that binary, which
   installs llama-swap for the gateway's test end to end, `tests/gateway_lifecycle.rs`, which
   skips itself without it.
7. The checks of `CLAUDE.md` — fmt, clippy, the tests, the six evals, the guard's own tests —
   and `cargo deny check` under `deny.toml`. `sid.py check`, the closed rule's check, is left
   out: the pre-push hook runs it on every push, and the runner has no study link.
8. The binary and its `SHA256SUMS`, its `--version` printed, uploaded as the artifact.

**The release job**, on a pushed tag alone, after the build job: it downloads the artifact,
checks its sum again, reads the tag's message through GitHub's API, refusing a tag that is not
annotated, and creates the tag's release with `gh release create --verify-tag --notes-file`. It
is the only job whose token writes, `contents: write`, and it runs no code of the repository, so
it holds no clone: `gh`'s `--notes-from-tag` reads the tag from a clone, and refuses `--repo`.

---

## 3. What It Pins

| What | Pinned by | Renewed with |
|---|---|---|
| The image, `registry.fedoraproject.org/fedora:44` | The digest of its linux/amd64 manifest | `skopeo inspect --raw docker://registry.fedoraproject.org/fedora:44`, the `amd64` entry |
| Each action, `actions/checkout`, `upload-artifact`, `download-artifact` | Its commit, the version in a comment | `curl -fsSL https://api.github.com/repos/actions/<name>/git/ref/tags/<version>`, a release at least two weeks old |
| rustup-init | Its version in the URL and its SHA-256 | The `.sha256` beside it on `static.rust-lang.org/rustup/archive/<version>/` |
| The Rust toolchain | `rust-toolchain.toml`'s version; rustup checks each component against its channel's manifest | `docs/toolchain.md`, with an entry in its record |
| cargo-deny | Its musl release's version and SHA-256 | The `.sha256` published beside the release's archive, and this machine's `cargo-deny` moved to the same version |
| llama-swap | ai-ctl's own pin, `ai-ctl gateway install` ([security.md §7](security.md#llama-swap-releases)) | Its newest release, before each release of ai-ctl (§1) |
| The crates | `Cargo.lock`'s checksums, `--locked` | `cargo update`, reviewed by the review's dependency axis |

Two inputs are not pinned, on purpose. The packages are Fedora 44's at the run, signed by
Fedora: its `updates` repository keeps the newest build alone, so a pinned version would break
at its next update, and they are the target's own packages. RustSec's advisory database, which
`cargo deny check` fetches at each run, is meant to be the newest.

Two builds of the same sources, in two fresh containers of the pinned image, gave the same
binary, SHA-256 `760808cd…` on 2026-11-06. The run on GitHub gave another, `101221b4…`: the
binary holds the paths of cargo's home, `/github/home/.cargo` on the runner, so a build compares
with a build of the same home alone.

---

## 4. What It Trusts

- **Whoever can push to the repository.** A pushed tag that the guard accepts is published. The
  repository is public, its one writer the user: anyone reads it and its releases, only a writer
  pushes, and a fork's pull request runs nothing, the workflow having no `pull_request` trigger.
  The guard checks what the tag names, not who pushed it, nor that its commit is on `main`,
  which `committing-changes` alone ensures.
- **GitHub**: its runners, its Actions and the token it gives each job, scoped as §2 says.
- **The sources of §3**: Fedora's repositories, `static.rust-lang.org`, the cargo-deny and
  llama-swap releases on GitHub, crates.io. A pin detects a change to what it pins; it does not
  make the pinned content safe.
- **The artifact between the two jobs**: GitHub's artifact store, its digest checked by
  `download-artifact` and the sum checked again before the release.

A run spends no Actions minutes of the account: GitHub's standard runners are free for a public
repository.

---

## 5. Installing a Release

Production runs a release's binary, the one the workflow checked, downloaded with GitHub's CLI,
`gh`, from Fedora's repositories (`sudo dnf install gh`), logged in once by `gh auth login`, its
token kept in the GNOME keyring. The binary is linked against Fedora 44's glibc and holds the shipped
inventory and the patches; it needs no Rust toolchain.

```bash
dir=$(mktemp -d)
gh release download vX.Y.Z -R sebetc4/ai-ctl -D "$dir"
(cd "$dir" && sha256sum -c SHA256SUMS) && install -Dm 0755 "$dir/ai-ctl" ~/.local/bin/ai-ctl
ai-ctl --version
```

The `SHA256SUMS` comes from the same release as the binary: it detects a corrupt download, not a
replaced release, which only a writer of the repository can make (§4). A run by hand's binary,
to try a branch before its tag, comes the same way from the run's artifact:
`gh run download <run-id> -R sebetc4/ai-ctl -n ai-ctl -D "$dir"` in place of
`gh release download`, the artifact keeping the binary's mode.

---

## Sources

- The workflow's steps, run in its image under rootless podman 5.8.9 on 2026-11-06: two runs, the
  second green from end to end (roadmap `fresh-history-and-ci`, phase 2).
- `actionlint` 1.7.12 on the workflow, 2026-11-06.
- The run by hand on GitHub on 2026-11-06, on the roadmap's branch, green from
  end to end, and its artifact installed by §5's command with `gh` 2.97.0, its sum checked, the
  binary run on this machine.
- The first run of a tag on GitHub, v0.2.0's, on 2026-11-06: the build job green, the release
  job failed on `using --notes-from-tag with --repo is not supported`. The fixed step's two API
  calls, run on this machine against that tag the same day, gave the notes `--notes-from-tag`
  gives from a clone; `actionlint` 1.7.12, with its `shellcheck` 0.11.0, found nothing in the
  fixed workflow. Its first run on GitHub is v0.2.0's tag in the repository made again that day.
- The release notes of `actions/checkout` v7.0.1, `actions/upload-artifact` v7.0.2 and
  `actions/download-artifact` v8.0.2.
