#!/usr/bin/env python3
"""Gather what cargo-deny does not check about each crate of Cargo.lock, for review-dependencies to judge.

Usage:
  dependencies.py <folder> [--repo <folder>]

<folder> is the review's folder, made when missing; dependencies.json is written there, whole or
not at all. The facts come from `cargo metadata --locked`; `cargo update --dry-run --verbose`,
which changes nothing; crates.io, three requests a crate; and GitHub, 60 requests an hour for the
address asking when no token is sent, asked for the direct crates and for those last released
more than a year ago, each repository once, then for the release notes of llama-swap and of each
backend, two requests a project at most. Requests go one a second to each, their User-Agent
naming the tool alone. No request follows a refusal from GitHub, 403 or 429.

dependencies.json holds the commit, the day gathered, the workspace's rust-version, `left_out`,
the packages from neither crates.io nor a path, which cargo-deny's sources check judges,
`projects`, below, and a record per crate of crates.io, by name, then version:

direct              the dependency kinds through which a workspace member uses it, none when
                    it is transitive
update              the version `cargo update` takes, within Cargo.toml's requirements
available           a newer version cargo names, beyond the requirements or held back by the
                    rust-version
requires            the Rust version cargo says that version, or the locked one, requires
newest              crates.io's newest stable version
yanked              whether the locked version is yanked
last_release        the day of the newest version not yanked
releases_past_year  the versions not yanked published in the 365 days before the day gathered
releases            the versions not yanked
owners              its owners on crates.io, users and teams
dependents          the crates crates.io lists as depending on it
downloads           its downloads, and recent_downloads those of the last 90 days
repository          the repository crates.io names
github              what GitHub answers for that repository: the name asked, its name now,
                    archived, the day of its last push; or why it was not asked or answered
unsafe              the `unsafe` keywords of its .rs files, comments and strings left out

`projects` holds a record per program ai-ctl runs: llama-swap at the release `PINNED` names in
src/domain/gateway.rs, then each backend at the current build of ai-ctl's data folder —
builds/<project>/current/build.json, its repository from its clone's origin under
sources/<project>, read with no fetch:

project             its name
repository          its GitHub repository, owner/name
current             the release or build ai-ctl runs
anchor              the date its releases are compared with: its own release's on GitHub's page,
                    else its commit's in its clone
newest              the newest release on the page
releases            each release made after the anchor, newest first, on one page of 100: its
                    tag, the date it was made, its note and cut_from; a note leaves out the lines
                    in more than half the page's releases and in two at least, the project's
                    template, and those linking to the release's own assets
commits             when every release kept has an empty note, the commits after the anchor up
                    to the newest release, on one page of 100: hash, date, message and cut_from;
                    null otherwise
left_out            what lies past a page, or the page's releases when no anchor is found
github              null when GitHub answered; otherwise its status, or why it was not asked

A note or a message is cut at 8 KB, cut_from giving its size in bytes, null when whole.

It prints a line per fact a session reads first — an update, a crate behind the newest, a newer
Rust required, a yanked version, a last release over a year old, a repository moved or archived,
a GitHub answer missing, a package left out, a project behind its newest release and what was
left out of its notes —, then a summary line.

Exit codes: 0 written; 2 wrong arguments; 3 not gathered, nothing written, the reason on stderr:
the repository not a git one, cargo failing, crates.io or GitHub unreachable, or crates.io
answering otherwise than with a crate's facts.
"""

import argparse
import http.client
import json
import os
import re
import subprocess
import sys
import time
import urllib.error
import urllib.parse
import urllib.request
from collections import Counter, namedtuple
from datetime import date, datetime, timedelta, timezone
from pathlib import Path

# Importing findings.py and scope.py leaves no __pycache__ in the skill's folder.
sys.dont_write_bytecode = True
import findings  # noqa: E402
import scope  # noqa: E402

USER_AGENT = "ai-ctl-review-dependencies (reviewing-delivered-code skill)"
CRATES_IO = "https://crates.io/api/v1/crates/"
GITHUB = "https://api.github.com/repos/"
REGISTRIES = ("registry+https://github.com/rust-lang/crates.io-index", "sparse+https://index.crates.io/")
OUTPUT = "dependencies.json"
INTERVAL = 1.0
YEAR = timedelta(days=365)
REFUSALS = (403, 429)
GITHUB_ACCEPT = "application/vnd.github+json"
PAGE = 100
NOTE_LIMIT = 8192
GATEWAY = "src/domain/gateway.rs"
PIN_RE = re.compile(r"^pub const PINNED: Pinned = Pinned \{\s*release: Release\((\d+)\)", re.M)
REPOSITORY_RE = re.compile(r'^pub const REPOSITORY: &str = "([^"]*)";', re.M)
NEXT_RE = re.compile(r'<[^>]*>;\s*rel="next"')
BLANK_RUN_RE = re.compile(r"\n{3,}")
UPDATE_RE = re.compile(r"^\s*(?:Updating|Downgrading|Unchanged) (\S+) v(\S+)(?: -> v(\S+))?(?: \((.+)\))?\s*$", re.M)
GITHUB_RE = re.compile(r"^(?:git\+)?https?://(?:www\.)?github\.com/([^/\s]+)/([^/\s#?]+)")
ASIDE = ("not asked: transitive", "not asked: no GitHub")

Response = namedtuple("Response", "status headers body")


class DependencyError(Exception):
    pass


def updates(text):
    """{(name, locked version): its update, the version available and the Rust it requires} from
    `cargo update --dry-run --verbose`'s stderr; the packages it adds or removes are none of the lock's."""
    found = {}
    for name, version, update, aside in UPDATE_RE.findall(text):
        facts = {"update": update or None, "available": None, "requires": None}
        for part in aside.split(", ") if aside else ():
            if part.startswith("available: v"):
                facts["available"] = part.removeprefix("available: v")
            elif part.startswith("requires "):
                facts["requires"] = part.removeprefix("requires ")
        found[(name, version)] = facts
    return found


def github_repository(url):
    """`owner/name` of a GitHub repository's address, None for another host or none."""
    match = GITHUB_RE.match(url or "")
    return match and f"{match[1]}/{match[2].removesuffix('.git')}"


def unsafe_count(folder):
    """The `unsafe` keywords of the .rs files under `folder`, comments and strings left out; None when it is missing."""
    if not folder.is_dir():
        return None
    return sum(token == "unsafe" for path in sorted(folder.rglob("*.rs")) if path.is_file()
               for token, _ in scope.lex(path.read_text(encoding="utf-8", errors="replace"))[0])


def headers(answer):
    return {name.lower(): value for name, value in answer.headers.items()}


class Web:
    """GET requests naming the tool alone, redirects followed."""

    def __init__(self, timeout=30):
        self.timeout = timeout

    def get(self, url, accept="application/json"):
        request = urllib.request.Request(url, headers={"User-Agent": USER_AGENT, "Accept": accept})
        try:
            with urllib.request.urlopen(request, timeout=self.timeout) as answer:
                return Response(answer.status, headers(answer), answer.read())
        except urllib.error.HTTPError as error:
            with error:
                return Response(error.code, headers(error), error.read())
        except (urllib.error.URLError, http.client.HTTPException, OSError) as error:
            raise DependencyError(f"{url}: {getattr(error, 'reason', error)}") from None


class Paced:
    """A web whose requests start `interval` seconds apart at least: crates.io takes one a second."""

    def __init__(self, web, interval, clock=time.monotonic, sleep=time.sleep):
        self.web, self.interval, self.clock, self.sleep = web, interval, clock, sleep
        self.last = None

    def get(self, url, accept="application/json"):
        if self.last is not None:
            wait = self.last + self.interval - self.clock()
            if wait > 0:
                self.sleep(wait)
        self.last = self.clock()
        return self.web.get(url, accept)


def answer(web, url):
    """crates.io's JSON for `url`."""
    response = web.get(url)
    if response.status != 200:
        raise DependencyError(f"crates.io answered {response.status} for {url}")
    try:
        return json.loads(response.body)
    except (UnicodeDecodeError, json.JSONDecodeError):
        raise DependencyError(f"crates.io's answer for {url} is no JSON") from None


def field(value, url, *keys):
    """The value under `keys` in crates.io's answer for `url`."""
    for key in keys:
        if not isinstance(value, dict) or key not in value:
            raise DependencyError(f"crates.io's answer for {url} lacks `{key}`")
        value = value[key]
    return value


def crate_facts(web, name):
    """What crates.io knows of a crate: its versions, its newest, downloads, repository, owners and dependents."""
    url = f"{CRATES_IO}{name}"
    crate = answer(web, url)
    versions = [(field(version, url, "num"), field(version, url, "created_at")[:10], field(version, url, "yanked"))
                for version in field(crate, url, "versions")]
    facts = {key: field(crate, url, "crate", key)
             for key in ("max_stable_version", "downloads", "recent_downloads", "repository")}
    owners = f"{url}/owners"
    users = [field(user, owners, "login") for user in field(answer(web, owners), owners, "users")]
    dependents = f"{url}/reverse_dependencies?per_page=1"
    return {**facts, "versions": versions, "owners": users,
            "dependents": field(answer(web, dependents), dependents, "meta", "total")}


def repository_facts(web, name):
    """GitHub's status for a repository, and what it answers or the status alone: a refusal's message
    names the address asking."""
    response = web.get(f"{GITHUB}{name}", GITHUB_ACCEPT)
    if response.status != 200:
        limited = response.headers.get("x-ratelimit-remaining") == "0"
        return response.status, f"HTTP {response.status}" + (", rate limit reached" if limited else "")
    try:
        found = json.loads(response.body)
        return 200, {"asked": name, "repository": found["full_name"], "archived": found["archived"],
                     "pushed": found["pushed_at"][:10] if found["pushed_at"] else None}
    except (UnicodeDecodeError, json.JSONDecodeError, KeyError, TypeError):
        return 200, "GitHub's answer is no repository"


def packages_of(metadata):
    """The crates of crates.io by name, the kinds through which a workspace member uses each, the
    workspace's rust-version, and the packages from neither crates.io nor a path."""
    try:
        packages = {package["id"]: package for package in metadata["packages"]}
        members = metadata["workspace_members"]
        kinds = {}
        for node in metadata["resolve"]["nodes"]:
            if node["id"] in members:
                for dependency in node["deps"]:
                    kinds.setdefault(dependency["pkg"], set()).update(
                        kind["kind"] or "normal" for kind in dependency["dep_kinds"])
        crates = sorted((package for package in packages.values() if package["source"] in REGISTRIES),
                        key=lambda package: package["name"])
        left_out = [f"{package['name']} {package['version']} {package['source']}" for package in packages.values()
                    if package["source"] and package["source"] not in REGISTRIES]
        rust_version = packages[members[0]].get("rust_version") if members else None
    except (KeyError, TypeError, IndexError) as error:
        raise DependencyError(f"cargo metadata's output lacks {error}") from None
    return crates, {package["id"]: sorted(kinds.get(package["id"], ())) for package in crates}, rust_version, left_out


def record(package, direct, update, facts, since, unsafe):
    kept = [day for _, day, yanked in facts["versions"] if not yanked]
    return {"name": package["name"], "version": package["version"], "direct": direct,
            "update": update["update"], "available": update["available"], "requires": update["requires"],
            "newest": facts["max_stable_version"],
            "yanked": next((yanked for num, _, yanked in facts["versions"] if num == package["version"]), None),
            "last_release": max(kept, default=None), "releases_past_year": sum(day > since for day in kept),
            "releases": len(kept), "owners": facts["owners"], "dependents": facts["dependents"],
            "downloads": facts["downloads"], "recent_downloads": facts["recent_downloads"],
            "repository": facts["repository"], "github": None, "unsafe": unsafe}


def ask_github(records, web, since):
    """Each record's `github`: asked for a direct crate or one last released more than a year ago,
    each repository once, none once GitHub refused a request; the status of that refusal, None without one."""
    asked, refused = {}, None
    for crate in records:
        name = github_repository(crate["repository"])
        if name is None:
            crate["github"] = "not asked: no GitHub repository"
        elif not crate["direct"] and crate["last_release"] is not None and crate["last_release"] > since:
            crate["github"] = "not asked: transitive, released within a year"
        elif name in asked:
            crate["github"] = asked[name]
        elif refused:
            crate["github"] = f"not asked: GitHub answered {refused} earlier"
        else:
            status, crate["github"] = repository_facts(web, name)
            asked[name] = crate["github"]
            if status in REFUSALS:
                refused = status
    return refused


def project(name, repository=None, current=None, date=None, reason=None):
    """A program whose release notes are gathered: its GitHub repository, its current release or build, the date
    of that build's commit when read in its clone, or why GitHub is not asked."""
    return {"project": name, "repository": repository, "current": current, "date": date, "reason": reason}


def pinned(text):
    """llama-swap at the release `src/domain/gateway.rs`, given as `text`, pins."""
    pin, url = PIN_RE.search(text), REPOSITORY_RE.search(text)
    if not (pin and url):
        return project("llama-swap", reason=f"not asked: no pin in {GATEWAY}")
    repository = github_repository(url[1])
    return project("llama-swap", repository, f"v{pin[1]}",
                   reason=None if repository else "not asked: no GitHub repository")


def data_folder(environ):
    """ai-ctl's data folder, as it resolves it: XDG_DATA_HOME when absolute, else ~/.local/share; None when neither
    is set to an absolute path."""
    for name, under in (("XDG_DATA_HOME", ""), ("HOME", ".local/share")):
        base = Path(environ.get(name, ""))
        if base.is_absolute():
            return base / under / "ai-ctl"
    return None


def clone_git(clone, *args):
    """A git command's output in `clone`, None when it fails: a partial clone fetches nothing, and a folder that is
    no clone takes nothing from a repository around it."""
    done = subprocess.run(["git", "-C", str(clone), *args], capture_output=True, text=True, errors="replace",
                          env={**os.environ, "LC_ALL": "C", "GIT_NO_LAZY_FETCH": "1",
                               "GIT_CEILING_DIRECTORIES": str(clone.parent)})
    return done.stdout.strip() or None if done.returncode == 0 else None


def builds(data):
    """Each backend at its current build under `data`: its id and commit from build.json, its repository from its
    clone's origin, and that commit's date in its clone."""
    folder = data / "builds" if data else None
    if folder is None or not folder.is_dir():
        return []
    found = []
    for name in sorted(path.name for path in folder.iterdir() if path.is_dir() and not path.name.startswith(".")):
        record = Path("builds") / name / "current" / "build.json"
        try:
            build = json.loads((data / record).read_text(encoding="utf-8"))
            current, commit = build["id"], build["commit"]
            if not (isinstance(current, str) and isinstance(commit, str)):
                raise TypeError
        except FileNotFoundError:
            found.append(project(name, reason="not asked: no current build"))
            continue
        except (OSError, UnicodeDecodeError, json.JSONDecodeError, KeyError, TypeError):
            found.append(project(name, reason=f"not asked: {record} unreadable"))
            continue
        clone = data / "sources" / name
        url = clone_git(clone, "config", "--get", "remote.origin.url")
        if url is None:
            found.append(project(name, current=current,
                                 reason=f"not asked: no clone with an origin under sources/{name}"))
            continue
        repository = github_repository(url)
        if repository is None:
            found.append(project(name, current=current, reason="not asked: no GitHub repository"))
            continue
        date = clone_git(clone, "log", "-1", "--format=%cI", "--end-of-options", commit)
        found.append(project(name, repository, current, date))
    return found


def items_of(body):
    """GitHub's answer as a list of objects."""
    items = json.loads(body)
    if not (isinstance(items, list) and all(isinstance(item, dict) for item in items)):
        raise TypeError
    return items


def moment(text):
    """A date and time of GitHub or git, comparable whatever its offset."""
    found = datetime.fromisoformat(text)
    if found.tzinfo is None:
        raise ValueError(text)
    return found


def releases_of(body):
    """A page of GitHub's releases: each one's tag, the date it was made and its note, newest first."""
    try:
        page = [{"tag": item["tag_name"], "created": item["created_at"], "body": item["body"]}
                for item in items_of(body)]
        if not all(isinstance(release["tag"], str) and isinstance(release["body"], (str, type(None)))
                   and moment(release["created"]) for release in page):
            raise TypeError
    except (UnicodeDecodeError, json.JSONDecodeError, KeyError, TypeError, ValueError):
        raise DependencyError("GitHub's answer is no release list") from None
    return page


def commits_of(body):
    """A page of GitHub's commits: each one's hash, its committer's date and its message, newest first."""
    try:
        page = [{"sha": item["sha"], "date": item["commit"]["committer"]["date"], "message": item["commit"]["message"]}
                for item in items_of(body)]
        if not all(isinstance(commit["sha"], str) and isinstance(commit["message"], str) and moment(commit["date"])
                   for commit in page):
            raise TypeError
    except (UnicodeDecodeError, json.JSONDecodeError, KeyError, TypeError, ValueError):
        raise DependencyError("GitHub's answer is no commit list") from None
    return page


def clean(page, repository):
    """Each release's note without the project's template, the lines recurring in more than half the page and in
    two releases at least, without its links to its own assets, and with its blank runs collapsed."""
    lines = {release["tag"]: [line.rstrip() for line in (release["body"] or "").splitlines()] for release in page}
    seen = Counter()
    for kept in lines.values():
        seen.update({line.strip() for line in kept if line.strip()})
    template = {line for line, count in seen.items() if count >= 2 and count * 2 > len(page)}
    notes = {}
    for tag, kept in lines.items():
        assets = f"https://github.com/{repository}/releases/download/{tag}/".lower()
        kept = [line for line in kept if line.strip() not in template and assets not in line.lower()]
        notes[tag] = BLANK_RUN_RE.sub("\n\n", "\n".join(kept)).strip()
    return notes


def cut(text):
    """`text` within NOTE_LIMIT bytes, on a whole character, and its size in bytes when cut, None otherwise."""
    data = text.encode()
    if len(data) <= NOTE_LIMIT:
        return text, None
    return data[:NOTE_LIMIT].decode(errors="ignore"), len(data)


def listing(web, url, parse):
    """GitHub's page at `url`, parsed: its status, the items or why there are none, and whether a next page follows."""
    response = web.get(url, GITHUB_ACCEPT)
    if response.status != 200:
        limited = response.headers.get("x-ratelimit-remaining") == "0"
        return response.status, f"HTTP {response.status}" + (", rate limit reached" if limited else ""), False
    try:
        return 200, parse(response.body), NEXT_RE.search(response.headers.get("link", "")) is not None
    except DependencyError as error:
        return 200, str(error), False


def release_notes(found, web, refused):
    """The release notes of a project between its current release and its newest, on one page of GitHub's releases,
    and the commits since its build when every note is empty; with the status of a refusal from GitHub, that given
    when it was not asked."""
    record = {"project": found["project"], "repository": found["repository"], "current": found["current"],
              "anchor": None, "newest": None, "releases": [], "commits": None, "left_out": [],
              "github": found["reason"]}
    if record["github"]:
        return record, refused
    if refused:
        record["github"] = f"not asked: GitHub answered {refused} earlier"
        return record, refused
    repository = found["repository"]
    status, page, more = listing(web, f"{GITHUB}{repository}/releases?per_page={PAGE}", releases_of)
    if isinstance(page, str):
        record["github"] = page
        return record, status if status in REFUSALS else refused
    record["newest"] = page[0]["tag"] if page else None
    record["anchor"] = next((release["created"] for release in page if release["tag"] == found["current"]),
                            found["date"])
    if record["anchor"] is None:
        if page:
            record["left_out"].append(f"the page's {scope.plural(len(page), 'release')}: no date for "
                                      f"{found['current']} on the page or in a clone")
        return record, refused
    anchor = moment(record["anchor"])
    notes = clean(page, repository)
    for release in page:
        if moment(release["created"]) > anchor:
            note, cut_from = cut(notes[release["tag"]])
            record["releases"].append({"tag": release["tag"], "created": release["created"], "note": note,
                                       "cut_from": cut_from})
    if more and page and moment(page[-1]["created"]) > anchor:
        record["left_out"].append(f"the releases before {page[-1]['tag']}, past the page")
    if not record["releases"] or any(release["note"] for release in record["releases"]):
        return record, refused
    since = anchor.astimezone(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")
    query = urllib.parse.urlencode({"sha": record["releases"][0]["tag"], "since": since, "per_page": PAGE})
    status, commits, more = listing(web, f"{GITHUB}{repository}/commits?{query}", commits_of)
    if isinstance(commits, str):
        record["github"] = commits
        return record, status if status in REFUSALS else refused
    record["commits"] = []
    for commit in commits:
        if moment(commit["date"]) > anchor:
            message, cut_from = cut(commit["message"])
            record["commits"].append({"sha": commit["sha"], "date": commit["date"], "message": message,
                                      "cut_from": cut_from})
    if more and record["commits"]:
        record["left_out"].append(f"the commits before {record['commits'][-1]['sha'][:7]}, past the page")
    return record, refused


def ask_notes(projects, web, refused):
    """Each project's release notes, none asked once GitHub refused a request."""
    records = []
    for found in projects:
        record, refused = release_notes(found, web, refused)
        records.append(record)
    return records


def gather(metadata, update_text, crates_io, github, today, unsafe_of, projects):
    """The facts of each crate of crates.io in `metadata`, `cargo update`'s stderr given, crates.io and
    GitHub asked through the webs given, then the release notes of each project given."""
    crates, direct, rust_version, left_out = packages_of(metadata)
    since = (today - YEAR).isoformat()
    found = updates(update_text)
    facts = {}
    for package in crates:
        if package["name"] not in facts:
            facts[package["name"]] = crate_facts(crates_io, package["name"])
    none = {"update": None, "available": None, "requires": None}
    records = [record(package, direct[package["id"]], found.get((package["name"], package["version"]), none),
                      facts[package["name"]], since, unsafe_of(package)) for package in crates]
    refused = ask_github(records, github, since)
    return {"gathered": today.isoformat(), "rust_version": rust_version, "crates": records, "left_out": left_out,
            "projects": ask_notes(projects, github, refused)}


def lines(document):
    """The facts a session reads first, a line each, then the summary."""
    since = (date.fromisoformat(document["gathered"]) - YEAR).isoformat()
    out, counts = [], dict.fromkeys(("direct", "update", "behind", "yanked", "old", "answered", "archived",
                                     "moved"), 0)
    for crate in document["crates"]:
        label, github = f"{crate['name']} {crate['version']}", crate["github"]
        facts = {"direct": bool(crate["direct"]), "update": bool(crate["update"]),
                 "behind": bool(crate["newest"]) and crate["newest"] != (crate["update"] or crate["version"]),
                 "yanked": bool(crate["yanked"]),
                 "old": crate["last_release"] is None or crate["last_release"] <= since,
                 "answered": isinstance(github, dict)}
        facts["archived"] = facts["answered"] and github["archived"]
        facts["moved"] = facts["answered"] and github["repository"].lower() != github["asked"].lower()
        for key, seen in facts.items():
            counts[key] += seen
        if facts["update"]:
            out.append(f"update: {label} -> {crate['update']}")
        if facts["behind"]:
            out.append(f"behind: {label}, newest {crate['newest']}")
        if crate["requires"]:
            out.append(f"requires: {label}, {crate['requires']}"
                       + (f" for {crate['available']}" if crate["available"] else ""))
        if facts["yanked"]:
            out.append(f"yanked: {label}")
        if facts["old"]:
            out.append(f"old: {label}, " + (f"last release {crate['last_release']}" if crate["last_release"]
                                            else "every version yanked"))
        if facts["moved"]:
            out.append(f"moved: {crate['name']}, {github['asked']} now {github['repository']}")
        if facts["archived"]:
            out.append(f"archived: {crate['name']}, {github['repository']}")
        if not facts["answered"] and not github.startswith(ASIDE):
            out.append(f"github: {crate['name']}, {github}")
    out += [f"left out: {package}, not from crates.io" for package in document["left_out"]]
    behind = 0
    for record in document["projects"]:
        behind += record["newest"] is not None and record["newest"] != record["current"]
        if record["releases"]:
            out.append(f"notes: {record['project']} {record['current']} -> {record['newest']}, "
                       f"{scope.plural(len(record['releases']), 'release')}"
                       + (f" with empty notes, {scope.plural(len(record['commits']), 'commit')}"
                          if record["commits"] is not None else ""))
        out += [f"left out: {record['project']}, {what}" for what in record["left_out"]]
        if record["github"]:
            out.append(f"github: {record['project']}, {record['github']}")
    out.append(f"summary: {scope.plural(len(document['crates']), 'crate')}, {counts['direct']} direct; "
               f"{scope.plural(counts['update'], 'update')} within the requirements, {counts['behind']} behind the "
               f"newest, {counts['yanked']} yanked, {counts['old']} last released over a year ago; GitHub answered "
               f"for {scope.plural(counts['answered'], 'crate')}, {counts['archived']} archived, "
               f"{counts['moved']} moved; {scope.plural(len(document['projects']), 'project')}, {behind} behind "
               "their newest release")
    return out


def today():
    return date.today()


def cargo(folder, *args):
    done = subprocess.run(["cargo", *args], cwd=folder, capture_output=True, text=True, errors="replace",
                          env={**os.environ, "LC_ALL": "C", "CARGO_TERM_COLOR": "never"})
    if done.returncode != 0:
        raise DependencyError(f"cargo {args[0]}: {done.stderr.strip()}")
    return done


def main(argv):
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("folder", help="the review's folder")
    parser.add_argument("--repo", default=".", help="the repository, the current folder by default")
    args = parser.parse_args(argv)
    folder = Path(args.folder)
    try:
        git = scope.repository(Path(args.repo))
        commit = scope.resolve(git, "HEAD")
        try:
            metadata = json.loads(cargo(git.folder, "metadata", "--locked", "--format-version", "1").stdout)
        except json.JSONDecodeError as error:
            raise DependencyError(f"cargo metadata's output is no JSON: {error}") from None
        update_text = cargo(git.folder, "update", "--dry-run", "--verbose").stderr
        try:
            gateway = (git.folder / GATEWAY).read_text(encoding="utf-8")
        except (OSError, UnicodeDecodeError):
            gateway = ""
        web = Web()
        facts = gather(metadata, update_text, Paced(web, INTERVAL), Paced(web, INTERVAL), today(),
                       lambda package: unsafe_count(Path(package["manifest_path"]).parent),
                       [pinned(gateway), *builds(data_folder(os.environ))])
        document = {"commit": commit, **facts}
        folder.mkdir(parents=True, exist_ok=True)
        findings.write(folder / OUTPUT, findings.as_json(document))
    except (DependencyError, scope.ScopeError, OSError) as error:
        for line in str(error).splitlines():
            print(f"dependencies.py: {line}", file=sys.stderr)
        return 3
    except KeyboardInterrupt:
        print("dependencies.py: interrupted; nothing written", file=sys.stderr)
        return 130
    for line in lines(document):
        print(line)
    print(f"written: {folder / OUTPUT}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
