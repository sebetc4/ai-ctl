#!/usr/bin/env python3
"""Build the folder a fixture case of reviewing-delivered-code's evals starts in.

Usage: fixtures.py <case> <folder>

scripts/workspace.py of authoring-skills runs it once per fixture case when it prepares an
iteration. Every case starts from fixtures/base.tar.gz, ai-ctl's tree as of 2026-10-30,
before the review skill's change, in one commit: it leaves out `.claude/` and roadmap
`review-skill`, which hold the skill and the cases' answer keys, since a run's folder gets
its arm's copy of the skill and agents, and it needs no history of this repository. CLAUDE.md
comes from the pinned copy beside this script, without the checks of these evals; deny.toml,
cargo-deny's policy, which the base predates, from its own; .agent-conventions.toml, which
ai-ctl's git ignores, from its own.

  cost-gate          the base, then one commit on main: a session that added
                     `ai-ctl gateway prune`, with defects planted across the axes
  phase-close        the base, then roadmap `gateway-releases` on its branch: phase 1,
                     the same prune, done and waiting for its review
  continued-review   phase-close, with phase 1's review stopped before its verifiers,
                     its folder uncommitted
  roadmap-close      the roadmap's last phase done and reviewed: phase 1's review and
                     fixes, its register and recorded mutants, phase 2 with a security
                     defect planted, reviewed and fixed but for it
  dependency-check   the base, its lock file holding anyhow 1.0.102, under
                     RUSTSEC-2026-0190, and clap_complete 4.6.2, behind 4.6.11

The roadmap's commits are fixtures/gateway-releases/series.json, each a patch whose
`{{commit:N}}` stands for the short hash of commit N, the base being 0.
"""

import json
import os
import re
import shutil
import subprocess
import sys
import tarfile
from pathlib import Path

HERE = Path(__file__).resolve().parent
FIXTURES = HERE / "fixtures"
SERIES = FIXTURES / "gateway-releases"
# ai-ctl's tree before the review skill's change, 2026-10-30, by the author of its commits.
BASE = FIXTURES / "base.tar.gz"
BASE_DATE = "2026-10-30T23:20:36+01:00"
BASE_MESSAGE = "chore: ai-ctl as of 2026-10-30"
AUTHOR = ("sebetc4", "contact@sebastien-etcheto.com")
BRANCH = "roadmap/gateway-releases"
REVIEWS = Path("docs/reviews/delivered-code")
# The user's git configuration stays out: no signing, hook or template reaches the fixture.
GIT_ENV = {**os.environ, "GIT_CONFIG_GLOBAL": os.devnull, "GIT_CONFIG_NOSYSTEM": "1"}


def git(folder, *args, env=None, data=None):
    done = subprocess.run(["git", *args], cwd=folder, check=True, capture_output=True,
                          input=data, env=env or GIT_ENV)
    return done.stdout.decode().strip()


def committer(folder, date):
    """The environment of a commit at `date`, by the base's author."""
    name, email = AUTHOR
    return {**GIT_ENV, "GIT_AUTHOR_NAME": name, "GIT_AUTHOR_EMAIL": email,
            "GIT_COMMITTER_NAME": name, "GIT_COMMITTER_EMAIL": email,
            "GIT_AUTHOR_DATE": date, "GIT_COMMITTER_DATE": date}


def base(folder, lock_patch=None):
    """The base commit on main, with `lock_patch` applied to it when given."""
    with tarfile.open(BASE) as tar:
        tar.extractall(folder, filter="tar")
    shutil.copy2(FIXTURES / "claude.md", folder / "CLAUDE.md")
    shutil.copy2(FIXTURES / "deny.toml", folder / "deny.toml")
    git(folder, "init", "-q", "-b", "main")
    if lock_patch:
        git(folder, "apply", str(lock_patch))
    git(folder, "add", "-A")
    git(folder, "commit", "-q", "-m", BASE_MESSAGE, env=committer(folder, BASE_DATE))
    shutil.copy2(FIXTURES / "agent-conventions.toml", folder / ".agent-conventions.toml")


def filled(text, hashes):
    """`text` with each `{{commit:N}}` replaced by commit N's short hash."""
    return re.sub(r"\{\{commit:(\d+)\}\}", lambda m: hashes[int(m.group(1))], text)


def short(folder):
    return git(folder, "rev-parse", "--short=7", "HEAD")


def roadmap(folder, last):
    """The base, then the roadmap's commits 1 to `last` on its branch; their hashes."""
    base(folder)
    hashes = [short(folder)]
    git(folder, "checkout", "-q", "-b", BRANCH)
    series = json.loads((SERIES / "series.json").read_text(encoding="utf-8"))
    for commit in series[:last]:
        patch = filled((SERIES / commit["patch"]).read_text(encoding="utf-8"), hashes)
        git(folder, "apply", "--whitespace=nowarn", data=patch.encode())
        git(folder, "add", "-A")
        git(folder, "commit", "-q", "-m", commit["message"], env=committer(folder, commit["date"]))
        hashes.append(short(folder))
    return hashes


def cost_gate(folder):
    base(folder)
    series = json.loads((SERIES / "series.json").read_text(encoding="utf-8"))
    for commit in series[1:3]:
        git(folder, "apply", "--whitespace=nowarn", str(SERIES / commit["patch"]))
    git(folder, "add", "-A")
    git(folder, "commit", "-q", "-m",
        "feat(gateway): remove old llama-swap releases with gateway prune",
        env=committer(folder, series[2]["date"]))


def phase_close(folder):
    roadmap(folder, 4)


def continued_review(folder):
    hashes = roadmap(folder, 4)
    for path in (SERIES / "continued-review").rglob("*"):
        if path.is_file():
            target = folder / REVIEWS / path.relative_to(SERIES / "continued-review")
            target.parent.mkdir(parents=True, exist_ok=True)
            target.write_text(filled(path.read_text(encoding="utf-8"), hashes), encoding="utf-8")


def roadmap_close(folder):
    roadmap(folder, len(json.loads((SERIES / "series.json").read_text(encoding="utf-8"))))


def dependency_check(folder):
    base(folder, FIXTURES / "dependency-check.patch")


CASES = {"cost-gate": cost_gate, "phase-close": phase_close,
         "continued-review": continued_review, "roadmap-close": roadmap_close,
         "dependency-check": dependency_check}


if __name__ == "__main__":
    if len(sys.argv) != 3:
        sys.exit(__doc__.splitlines()[2])
    if sys.argv[1] not in CASES:
        sys.exit(f"no fixture named {sys.argv[1]}: {', '.join(CASES)}")
    CASES[sys.argv[1]](Path(sys.argv[2]).resolve())
