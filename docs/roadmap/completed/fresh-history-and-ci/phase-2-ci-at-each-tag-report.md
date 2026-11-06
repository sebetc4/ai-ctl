# Phase 2 Report: CI At Each Tag

**Phase:** [phase-2-ci-at-each-tag.md](phase-2-ci-at-each-tag.md)
**Start Commit:** `docs(fresh-history-and-ci): close phase 1` of 2026-11-06

---

## Work Log

### 2026-11-06

The toolchain pinned by `rust-toolchain.toml` at 1.99.1, rustup's newest stable, where this
machine's `stable` was at 1.97.1: rustup installed it in 5 s at the first command run in the
repository, and updated itself from 1.29.0 to 1.29.3 at `rustup toolchain install`, both in
`docs/toolchain.md`'s record. Here, rustc 1.99.1 (58cf8f9c3 2026-10-13). Clippy 1.99 added one
lint the code broke, `chunks_exact_to_as_chunks`, on `kernel_address`'s split of an IP into
32-bit words, rewritten with `as_chunks`, stable since 1.88, below the crate's `rust-version`
1.89; the tests on IPv4 and IPv6 tables keep it, and `cargo mutants --in-diff` caught its 2
mutants. The ten checks green on 1.99.1. The CI's side of the proof waits for the container's run.

The release's guard, `.github/scripts/check_release.py`, written test-first: its 15 tests watched
failing on their assertions against a stub, then passing. It refuses a tag not of the form
`vX.Y.Z`, one that is not Cargo.toml's `[package]` version, and a version whose `CHANGELOG.md`
entry is missing, given twice or not dated `YYYY-MM-DD`, every problem at once, and exits 1; on
the tree as it stands, `v0.2.0` is refused for its entry `unreleased`. Of 15 mutants made by
hand, one survived, an entry of a longer version, `0.2.01`, taken for `0.2.0`'s, which a test let
through by counting the problems without naming them; the test names it now and kills it. Its
tests join the checks, in `CLAUDE.md` and the contract.

The CI's steps run in the pinned image, `registry.fedoraproject.org/fedora:44` by the digest of its
linux/amd64 manifest, `sha256:b22ccd18…`, built on 2026-11-06, under rootless podman 5.8.9: no
GPU, no git configuration, no study link, the repository cloned from a read-only mount as the
checkout would fetch it. The packages from Fedora's repositories, `git-core`, `gcc`, `python3`,
`tar`, `gzip` and `iproute`; rustup-init 1.29.3 and cargo-deny 0.20.2's musl release, each
checked against its SHA-256; then `rustup toolchain install`, which read `rust-toolchain.toml`:
rustc 1.99.1 (58cf8f9c3 2026-10-13), as here. The first run, at the toolchain's commit, found
two evals red and one test skipped, none for the code. `test_regression.py` runs cargo in a
scratch crate outside the repository, where rustup had no default, installed with
`--default-toolchain none`: the CI makes the pinned toolchain the default. `test_guard.py`
lists ports with `ss`, which the image lacks: `iproute` joins the packages. The gateway's test
end to end skipped itself for want of llama-swap: the CI installs it with the binary it built,
`ai-ctl gateway install`, at the digest ai-ctl pins, and the test runs. The second run, at the
guard's commit, green from end to end: the release build in 32 s, llama-swap v263, fmt, clippy,
the 867 tests, the gateway's test end to end among them, the six evals, the guard's tests and
`cargo deny check`; `ai-ctl --version` gives `ai-ctl 0.2.0`, its SHA-256
`760808cde058007cd8868825a48fd24e4963749a43404e67ab474b591b58e2ff`, the same as the first run's,
two builds of the same sources in two fresh containers. No test needed a fix; the closed rule's
check, `sid.py check`, stays the pre-push hook's, as decided at the opening.

The workflow, `.github/workflows/release.yml`, takes these steps as they ran. `actionlint`
1.7.12, its image pinned by digest and run without a network, finds nothing in it.

How production installs a release settled with the user: the release's binary through `gh`,
which the user installed from Fedora's `updates`, 2.97.0, and logged in. The run by hand was
first refused, `HTTP 404: workflow release.yml not found on the default branch`: GitHub runs by
hand only a workflow `main` holds, and the remote's `main` is the root commit. With the user's
yes, the remote's default branch was set to the roadmap's branch for the dispatch and back to
`main` right after it, the run taking the workflow of its ref. The run, at `test(ci): leave no bytecode cache from the release guard's tests`,
green from end to end in 3 min 50 s: rustc 1.99.1 (58cf8f9c3 2026-10-13) as here, the release
build in 1 min 09 s, llama-swap v263 checked against ai-ctl's digest, fmt, clippy, the 867
tests with the gateway's test end to end run, the six evals, the guard's 15 tests and
`cargo deny check`; the release job skipped. Its artifact downloaded by the install's command,
`gh run download` in place of `gh release download`, into a scratch folder in place of
`~/.local/bin` so as not to install a branch's build as production: `sha256sum -c` passed,
`ai-ctl --version` gave `ai-ctl 0.2.0`, `ai-ctl doctor` ran on this machine, and the binary kept
its mode through the artifact. Its SHA-256, `101221b4…`, is not the podman runs' `760808cd…`:
the binary holds 55 paths under `/github/home/.cargo`, the runner's cargo home. `git ls-remote
--tags origin` lists nothing and the remote holds no release. The install's command is in
`README.md`'s Build, and `docs/release.md` gains how a release is installed, the dispatch's
refusal and the runner's sum.

The documents at the phase's state: `docs/release.md`, written with the workflow, now with how
production installs a release, and its line in `docs/README.md`; `docs/security.md`, a
subsection "ai-ctl Itself" in §7 for what the CI runs and what production installs, the
pusher of a tag in §1's table; `CLAUDE.md`'s Releases, production installing the release's
binary by `gh`, and its Checks, the guard's tests, since the guard's commit;
`docs/architecture.md`'s Repository Layout for `.github/`, since the workflow's commit.

The phase's review, `docs/reviews/delivered-code/2026-11-06-fresh-history-and-ci-phase-2/`, at
tier `low` on the user's yes, the scope from `docs(fresh-history-and-ci): close phase 1` to `docs: describe what the CI runs and what production installs` holding one hunk of code, the
`as_chunks` rewrite, whose 2 mutants are caught. `scope.py` routes the workflow and the guard
outside the code's reading; the session read them on the same axes. One finding, the
session's, confirmed by a verifier at effort low: `docs/security.md` gave `gh`'s token the scope
`repo` alone, where `gh auth status` lists `repo`, `read:org`, `gist` and `admin:public_key`;
fixed in `docs(security): name every scope of gh's token`, the phase's decision corrected in place. The 58 recorded mutants run again
at the fix, all caught, in 82 min.

The roadmap's closing pass,
`docs/reviews/delivered-code/2026-11-06-fresh-history-and-ci-closing-pass/`, at tier `high` on
the user's yes, scoped from `chore: start the history again from the tree of 2026-11-06` to `docs(reviews): review phase 2 of fresh-history-and-ci` by `scope.py range`, as phase 1's Assessment says:
`review-security`, `review-dependencies` and one verifier, each at effort high, told to read
the workflow and the guard as the release's trust boundary. The checks green, `cargo deny
check` passing with the duplicates `nvml-wrapper` 0.13.0 holds, the scope's 2 mutants caught.
Two findings, both below the floor once verified. `review-security`'s: `rustup toolchain
install` self-updates rustup, undoing the pin of rustup-init. The verifier reproduced it
against a fake update root on loopback, the newest rustup-init downloaded with no checksum
and run; but rustup 1.29.3 skips it while `CI` is set, and the run by hand's Rust step, its
container given `CI`, prints no self-update line. So no effect on GitHub today, the podman
runs of the steps exposed, and the pin resting on a heuristic nothing stated: fixed in `fix(ci): keep the pinned rustup from updating itself`
by `--no-self-update`, proven in the pinned image under podman with `CI` unset, no self-update
line with the flag and `checking for self-update (current version: 1.29.3)` without it;
actionlint finds nothing. `review-dependencies`': `serde_spanned`, `toml` and `toml_datetime`
a patch behind within the requirements, released that day, updated in `build(deps): update the toml crates within their requirements`, the checks and
`cargo deny check` green on the new lock. The verifier's answer was refused once by
`findings.py add`, its PROBE naming the finding it verified, which no FINDING of its own
answer holds; given again without the PROBE, the session having settled it from the run's
log. The recorded mutants, run again at `build(deps): update the toml crates within their requirements`, the last fix: 58 caught, 0 missed, 0 gone. No
review follows the pass.

---

## Decisions

- The toolchain pinned at the newest stable, 1.99.1 of 2026-10-13, by `rust-toolchain.toml`:
  the release binary built by the current compiler, the pin changed by one line when a newer one
  is adopted. Set aside: 1.97.1, this machine's `stable`, older only because it was not updated.
  The file asks for the minimal profile with `clippy` and `rustfmt`, what the checks run; a
  developer's own components, such as `rust-src` for an editor, stay outside it.
- The release's guard is a Python script on the standard library, as the review skill's scripts
  are, under `.github/scripts/` beside the workflow it serves, its tests among the checks: the
  rule a tag must meet stays out of ai-ctl's code and is tested here as in the CI. It reads
  `Cargo.toml` with `tomllib`, where `committing-changes` reads its first `version = ` line.
- cargo-deny comes into the CI as its musl release, 0.20.2 as here, checked against its SHA-256:
  one download in seconds where `cargo install` would build it in a minute at each run of a
  private repository's minutes. Set aside: the build from source, as this machine has it, which
  trusts the same publisher through its `Cargo.lock`.
- The packages come from Fedora 44's repositories at each run, signed by Fedora and unpinned:
  their `updates` keep the newest build alone, so a pinned version would break at the next
  update. The image is pinned by digest; the packages are those of the target itself.
- A release is published on a pushed tag alone: the guard and the release job run on
  `github.event_name == 'push'` and a tag, so a run by hand, from a branch or a tag, checks and
  builds and leaves its binary as an artifact kept 14 days. The build job reads the repository;
  the release job alone writes, `contents: write`, and runs no code of the repository, only `gh`
  on the artifact, its sum checked again.
- Production installs the release's binary, downloaded by `gh release download`, its
  `SHA256SUMS` checked, into `~/.local/bin` (the user, 2026-11-06): it runs the binary the CI
  checked, with no Rust toolchain on a production machine, and `gh` also starts the run by hand
  and downloads its artifact. `gh` comes from Fedora 44's repositories, 2.97.0, logged in by
  `gh auth login` in the browser, its token, of scopes `repo`, `read:org`, `gist` and
  `admin:public_key`, in the GNOME keyring: the SSH key in the agent already pushes to every
  repository of the account. `admin:public_key`, which the login added with SSH as git
  protocol, reaches further, adding a key to the account; the user keeps it or drops it with
  `gh auth refresh --remove-scopes admin:public_key`. Set aside: `cargo install`
  from the tag over SSH, which needs nothing new but runs a binary built here, not the CI's,
  and needs Rust and gcc on each production machine; the binary by `curl` and a token, the same
  result with more script; a fine-grained token on ai-ctl alone, narrower but renewed at each
  expiry.

---

## Deviations

- The run by hand needed the remote's default branch switched for its dispatch, GitHub running
  by hand only a workflow `main` holds; `main` is the default again. Once the roadmap is
  integrated, `main` holds the workflow and a run by hand starts from any branch without it.
- The closing pass's fix to the workflow proven in its pinned image under podman, not by a
  second run on GitHub, which the constraints allowed after a fix: the flag changes one command
  whose output the podman run shows with and without it, and 0.2.0's tag runs the workflow on
  GitHub next.
- The last task's proof, a red test seen failing before each fix, taken by the check that fits
  each artifact, as `CLAUDE.md`'s Tests convention gives for what is not code: neither fix
  changes code a test could fail on. The workflow step's red is its run in the pinned image
  under podman as it stood, `rustup toolchain install` printing `checking for self-update
  (current version: 1.29.3)`, and its green the same run with `--no-self-update`, no such
  line. The lock's red is `dependencies.json`'s three updates within the requirements,
  `serde_spanned` 1.1.2 → 1.1.3, `toml` 1.2.0 → 1.2.1 and `toml_datetime` 1.1.2 → 1.1.3, and its
  green `cargo update --dry-run -p serde_spanned -p toml -p toml_datetime` at `build(deps): update the toml crates within their requirements`, locking 0
  packages, the checks and `cargo deny check` passing. The register gains no entry: both
  findings are fixed.

---

## Assessment

The phase leaves ai-ctl a release made by its CI. `rust-toolchain.toml` pins Rust 1.99.1 here
and in the workflow; `.github/workflows/release.yml`, at a pushed tag `v*`, refuses a tag that is
not `Cargo.toml`'s version with a dated `CHANGELOG.md` entry, checks and builds ai-ctl in Fedora
44's image pinned by digest, and publishes the binary with its `SHA256SUMS` on the tag's
release, its one writing job running no code of the repository; by hand it checks and builds
alone. Its one run on GitHub, by hand on the roadmap's branch, was green from end to end, and its
binary, installed here by the command `README.md`'s Build gives, ran. Production installs a
release with `gh release download`, its sum checked; `docs/release.md` holds the workflow, its
pins, its trust and the install, `docs/security.md` what the CI runs and what production
installs. The phase's review found one statement of `docs/security.md` wrong, the closing pass a
self-update the CI escaped only by rustup's own heuristic and a patch update of the toml crates;
all three are fixed, and every recorded mutant is caught at the last fix.

What 0.2.0's release needs first: the roadmap's branch integrated into `main` and pushed, which
also lets a run by hand start from any branch, `main` then holding the workflow; then
`CHANGELOG.md`'s 0.2.0 entry dated, through `committing-changes`, whose tag's push is the
workflow's first run on a tag. `gh`'s token holds `admin:public_key`, which the login added and
the user keeps or drops. The dependency axis left llama-swap at v263 behind v270, llama.cpp's
build b12111 behind b12177, stable-diffusion.cpp's 19 tags behind and audio.cpp's v0.9.3 behind
v0.9.4, no release note read; and `findings.py add` refuses a verifier's PROBE on the finding it
verifies, which a verifier then cannot ask for a run it needs.
