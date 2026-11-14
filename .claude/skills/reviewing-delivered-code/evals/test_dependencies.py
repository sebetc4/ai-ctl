#!/usr/bin/env python3
"""Unit tests of scripts/dependencies.py, run on the outputs of cargo, crates.io and GitHub captured by a probe.

Usage: test_dependencies.py [unittest options]
"""

import io
import json
import os
import subprocess
import sys
import tempfile
import threading
import unittest
from contextlib import redirect_stderr, redirect_stdout
from datetime import date
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path
from unittest import mock

HERE = Path(__file__).resolve().parent
SCRIPT = HERE.parent / "scripts" / "dependencies.py"
CAPTURES = HERE.parents[3] / "tests" / "fixtures"
sys.dont_write_bytecode = True
sys.path[:0] = [str(SCRIPT.parent), str(HERE)]

import dependencies  # noqa: E402
from test_scope import GIT_ENV, Repo  # noqa: E402

TODAY = date(2026, 11, 2)
ITOA = "itoa 1.0.18 git+https://github.com/dtolnay/itoa?tag=1.0.18#af77385d0daf4d0e949e81f2588be2e44f69f086"
UNSAFE = {"equivalent": 1, "fnv": 2, "heck": 3, "static_assertions": 4, "unicode-segmentation": 5}


def captured(name):
    """The last response of a captured `.http` file, after its redirects: its status, headers and body."""
    *heads, body = (CAPTURES / name).read_bytes().split(b"\r\n\r\n")
    status, *fields = heads[-1].decode().split("\r\n")
    headers = {key.lower(): value.strip() for key, _, value in (line.partition(":") for line in fields)}
    return dependencies.Response(int(status.split()[1]), headers, body)


NONE = dependencies.Response(404, {}, b"")


def capture_of(url):
    """The capture answering `url`, by the probe's names."""
    crates = "https://crates.io/api/v1/crates/"
    if url.startswith(crates):
        name, _, rest = url[len(crates):].partition("/")
        suffix = {"": "", "owners": "-owners", "reverse_dependencies?per_page=1": "-reverse-dependencies"}[rest]
        return f"crates-io-{name}{suffix}.http"
    owner, name, *listing = url.removeprefix("https://api.github.com/repos/").partition("?")[0].split("/")
    return "-".join(["github", *listing, owner, name]) + ".http"


class Captured:
    """A web answering from the probe's captures, each request recorded, some answers replaced."""

    def __init__(self, replaced=None):
        self.asked, self.replaced = [], replaced or {}

    def get(self, url, accept=None):
        self.asked.append(url)
        answer = self.replaced.get(url)
        if isinstance(answer, Exception):
            raise answer
        return answer or captured(capture_of(url))


def probe(crates_io=None, github=None, projects=()):
    metadata = json.loads((CAPTURES / "cargo-metadata-dependencies-probe.json").read_text(encoding="utf-8"))
    update = (CAPTURES / "cargo-update-dry-run-probe.txt").read_text(encoding="utf-8")
    return dependencies.gather(metadata, update, crates_io or Captured(), github or Captured(), TODAY,
                               lambda package: UNSAFE[package["name"]], list(projects))


def project(name, repository, current, date=None, reason=None):
    """A program whose release notes are gathered, as `pinned` and `builds` find it."""
    return {"project": name, "repository": repository, "current": current, "date": date, "reason": reason}


def notes(name, repository, current, **facts):
    found = {"project": name, "repository": repository, "current": current, "anchor": None, "newest": None,
             "releases": [], "commits": None, "left_out": [], "github": None}
    return {**found, **facts}


RELEASES = "https://api.github.com/repos/{}/releases?per_page=100"
SD_COMMITS = ("https://api.github.com/repos/leejet/stable-diffusion.cpp/commits?sha=master-1079-a3c49b1"
              "&since=2026-11-09T14%3A16%3A04Z&per_page=100")
GATEWAY = ('pub const REPOSITORY: &str = "https://github.com/mostlygeek/llama-swap";\n'
           'pub const BINARY: &str = "llama-swap";\n\n'
           "pub const PINNED: Pinned = Pinned {\n    release: Release(271),\n    sha256: \"58d5c897\",\n};\n\n"
           "pub const OLDEST: Release = Release(267);\n")
LLAMA_SWAP = project("llama-swap", "mostlygeek/llama-swap", "v271")
LLAMA_CPP = project("llama.cpp", "ggml-org/llama.cpp", "b12227", "2026-11-09T14:54:27+02:00")
SD_CPP = project("stable-diffusion.cpp", "leejet/stable-diffusion.cpp", "master-1071-cfdf3cf",
                 "2026-11-09T22:16:04+08:00")
AFTER_B12227 = ["b12242", "b12241", "b12240", "b12239", "b12238", "b12237", "b12236", "b12233", "b12230", "b12229",
                "b12228"]
B12242 = ("chat : don't use lazy grammar when response_format is set with tools (#31956)\n\n"
          "- <https://github.com/ggml-org/llama.cpp/attestations/65619172>")


def record(name, version, **facts):
    found = {"name": name, "version": version, "direct": [], "update": None, "available": None, "requires": None}
    return {**found, **facts}


EQUIVALENT = record("equivalent", "1.0.1", direct=["normal"], update="1.0.3", newest="1.0.3", yanked=False,
                    last_release="2026-10-13", releases_past_year=1, releases=5, owners=["cuviper"], dependents=44,
                    downloads=998441905, recent_downloads=289863622,
                    repository="https://github.com/indexmap-rs/equivalent",
                    github={"asked": "indexmap-rs/equivalent", "repository": "indexmap-rs/equivalent",
                            "archived": False, "pushed": "2026-10-13"}, unsafe=1)
FNV = record("fnv", "1.0.7", direct=["dev"], newest="1.0.7", yanked=False, last_release="2020-05-14",
             releases_past_year=0, releases=8, owners=["SimonSapin", "github:servo:cargo-publish"], dependents=837,
             downloads=922476515, recent_downloads=210402606, repository="https://github.com/servo/rust-fnv",
             github={"asked": "servo/rust-fnv", "repository": "servo/rust-fnv", "archived": False,
                     "pushed": "2024-03-20"}, unsafe=2)
HECK = record("heck", "0.4.1", direct=["normal"], available="0.5.0", newest="0.5.0", yanked=False,
              last_release="2024-03-12", releases_past_year=0, releases=11, owners=["withoutboats", "jplatte"],
              dependents=1892, downloads=1496761002, recent_downloads=352429047,
              repository="https://github.com/withoutboats/heck",
              github={"asked": "withoutboats/heck", "repository": "withoutboats/heck", "archived": False,
                      "pushed": "2025-08-09"}, unsafe=3)
HECK_5 = {**HECK, "version": "0.5.0", "available": None}
STATIC_ASSERTIONS = record("static_assertions", "1.1.0", newest="1.1.0", yanked=False, last_release="2019-11-03",
                           releases_past_year=0, releases=15, owners=["nvzqz"], dependents=1705,
                           downloads=580578426, recent_downloads=139048614,
                           repository="https://github.com/nvzqz/static-assertions-rs",
                           github={"asked": "nvzqz/static-assertions-rs", "repository": "nvzqz/static-assertions",
                                   "archived": False, "pushed": "2023-11-18"}, unsafe=4)
UNICODE_SEGMENTATION = record("unicode-segmentation", "1.13.1", update="1.13.3", newest="1.13.3", yanked=True,
                              last_release="2026-06-01", releases_past_year=2, releases=25,
                              owners=["huonw", "SimonSapin", "Manishearth", "kwantam", "sujayakar"], dependents=2090,
                              downloads=651975133, recent_downloads=166650239,
                              repository="https://github.com/unicode-rs/unicode-segmentation",
                              github="not asked: transitive, released within a year", unsafe=5)


class UpdatesTest(unittest.TestCase):
    def updates(self, capture):
        return dependencies.updates((CAPTURES / capture).read_text(encoding="utf-8"))

    def test_an_update_within_the_requirements_and_a_version_beyond_them(self):
        self.assertEqual(self.updates("cargo-update-dry-run-probe.txt"), {
            ("equivalent", "1.0.1"): {"update": "1.0.3", "available": None, "requires": None},
            ("heck", "0.4.1"): {"update": None, "available": "0.5.0", "requires": None},
            ("unicode-segmentation", "1.13.1"): {"update": "1.13.3", "available": None, "requires": None},
        })

    def test_the_packages_added_and_removed_are_no_locked_crate(self):
        self.assertEqual(self.updates("cargo-update-dry-run-unchanged.txt"), {
            ("anyhow", "1.0.80"): {"update": None, "available": "1.0.104", "requires": None},
            ("clap", "4.0.32"): {"update": "4.6.7", "available": None, "requires": None},
            ("clap_lex", "0.3.3"): {"update": "1.1.1", "available": None, "requires": None},
            ("strsim", "0.10.0"): {"update": "0.11.1", "available": None, "requires": None},
            ("thiserror", "1.0.69"): {"update": None, "available": "2.0.21", "requires": None},
            ("toml", "0.8.23"): {"update": None, "available": "1.1.8+spec-1.1.0", "requires": None},
        })

    def test_the_rust_version_a_newer_release_requires(self):
        self.assertEqual(self.updates("cargo-update-dry-run-rust-version.txt"), {
            ("clap", "4.0.32"): {"update": "4.4.18", "available": "4.6.7", "requires": "Rust 1.85"},
            ("clap_lex", "0.3.3"): {"update": "0.6.0", "available": None, "requires": None},
            ("windows-link", "0.2.1"): {"update": None, "available": None, "requires": "Rust 1.71"},
        })

    def test_nothing_to_update(self):
        self.assertEqual(self.updates("cargo-update-dry-run-none.txt"), {})


class RepositoryTest(unittest.TestCase):
    def test_the_github_repository_of_a_crate(self):
        # The forms of `repository` crates.io gave on 2026-11-02 for anyhow, anstyle, libloading and
        # sequoia-openpgp.
        for url, found in [("https://github.com/dtolnay/anyhow", "dtolnay/anyhow"),
                           ("https://github.com/rust-cli/anstyle.git", "rust-cli/anstyle"),
                           ("https://github.com/nagisa/rust_libloading/", "nagisa/rust_libloading"),
                           ("https://gitlab.com/sequoia-pgp/sequoia", None), (None, None)]:
            self.assertEqual(dependencies.github_repository(url), found, url)


class UnsafeTest(unittest.TestCase):
    def test_unsafe_is_counted_in_the_code_of_every_rust_file_and_nowhere_else(self):
        with tempfile.TemporaryDirectory() as tmp:
            crate = Path(tmp)
            (crate / "src" / "sub").mkdir(parents=True)
            (crate / "src" / "lib.rs").write_text(
                "// unsafe in a comment\n/* an unsafe /* nested */ block */\npub unsafe fn f() {}\n"
                "pub fn g() { unsafe { f() } }\nconst S: &str = \"unsafe\";\nconst R: &str = r#\"unsafe\"#;\n")
            (crate / "src" / "sub" / "mod.rs").write_text("struct X;\nunsafe impl Send for X {}\n")
            (crate / "README.md").write_text("unsafe\n")
            self.assertEqual(dependencies.unsafe_count(crate), 3)
            self.assertIsNone(dependencies.unsafe_count(crate / "missing"))


class PacedTest(unittest.TestCase):
    def test_each_request_starts_an_interval_after_the_last(self):
        times, slept = iter([10.0, 10.2, 11.0, 13.0, 13.0]), []
        web = Captured()
        paced = dependencies.Paced(web, 1.0, clock=lambda: next(times), sleep=slept.append)
        for name in ("fnv", "heck", "equivalent"):
            paced.get(f"https://crates.io/api/v1/crates/{name}")
        self.assertEqual([round(wait, 3) for wait in slept], [0.8])
        self.assertEqual(len(web.asked), 3)


class Server(BaseHTTPRequestHandler):
    """A repository moved, one found, one missing; each answer echoes the request's headers."""

    def do_GET(self):
        if self.path == "/moved":
            self.send_response(301)
            self.send_header("Location", "/found")
            self.end_headers()
            return
        found = self.path == "/found"
        body = json.dumps({"user_agent": self.headers["User-Agent"], "accept": self.headers["Accept"]}).encode()
        self.send_response(200 if found else 404)
        self.send_header("Content-Length", str(len(body)))
        self.end_headers()
        self.wfile.write(body)

    def log_message(self, *args):
        pass


class WebTest(unittest.TestCase):
    def setUp(self):
        self.server = ThreadingHTTPServer(("127.0.0.1", 0), Server)
        self.thread = threading.Thread(target=self.server.serve_forever)
        self.thread.start()
        self.base = f"http://127.0.0.1:{self.server.server_port}"

    def tearDown(self):
        self.server.shutdown()
        self.thread.join()
        self.server.server_close()

    def test_a_request_names_the_tool_alone_and_follows_a_redirect(self):
        answer = dependencies.Web(timeout=5).get(f"{self.base}/moved", "application/vnd.github+json")
        self.assertEqual(answer.status, 200)
        self.assertEqual(json.loads(answer.body), {"user_agent": "ai-ctl-review-dependencies (reviewing-delivered-code skill)",
                                                   "accept": "application/vnd.github+json"})

    def test_an_answer_other_than_success_is_given_with_its_status(self):
        answer = dependencies.Web(timeout=5).get(f"{self.base}/missing")
        self.assertEqual((answer.status, json.loads(answer.body)["accept"]), (404, "application/json"))
        self.assertEqual(answer.headers["content-length"], str(len(answer.body)))

    def test_an_unreachable_server_is_an_error_naming_the_address(self):
        with self.assertRaises(dependencies.DependencyError) as raised:
            dependencies.Web(timeout=5).get("http://127.0.0.1:1/api")
        self.assertIn("http://127.0.0.1:1/api", str(raised.exception))


class PinnedTest(unittest.TestCase):
    def test_llama_swap_at_its_pin_from_the_gateway_s_source(self):
        self.assertEqual(dependencies.pinned(GATEWAY), LLAMA_SWAP)

    def test_the_repository_s_own_gateway_source_names_a_pin(self):
        found = dependencies.pinned((HERE.parents[3] / "src" / "domain" / "gateway.rs").read_text(encoding="utf-8"))
        self.assertEqual((found["repository"], found["reason"]), ("mostlygeek/llama-swap", None))
        self.assertRegex(found["current"], r"^v\d+$")

    def test_a_source_without_a_pin_or_a_github_repository(self):
        none = project("llama-swap", None, None, reason="not asked: no pin in src/domain/gateway.rs")
        for text in ("", GATEWAY.replace("Release(271)", "Release::new(271)"),
                     GATEWAY.replace("pub const REPOSITORY", "pub const SOURCE")):
            self.assertEqual(dependencies.pinned(text), none, text)
        self.assertEqual(dependencies.pinned(GATEWAY.replace("github.com", "gitlab.com")),
                         project("llama-swap", None, "v271", reason="not asked: no GitHub repository"))


class DataFolderTest(unittest.TestCase):
    def test_the_data_folder_as_ai_ctl_resolves_it(self):
        for environ, found in [({"XDG_DATA_HOME": "/data", "HOME": "/home/u"}, Path("/data/ai-ctl")),
                               ({"XDG_DATA_HOME": "data", "HOME": "/home/u"}, Path("/home/u/.local/share/ai-ctl")),
                               ({"HOME": "/home/u"}, Path("/home/u/.local/share/ai-ctl")),
                               ({"XDG_DATA_HOME": "data", "HOME": "home"}, None), ({}, None)]:
            self.assertEqual(dependencies.data_folder(environ), found, environ)


class BuildsTest(unittest.TestCase):
    """Runs git in scratch folders."""

    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.data = Path(self.tmp.name) / "ai-ctl"

    def tearDown(self):
        self.tmp.cleanup()

    def build(self, name, build_id, commit=None, text=None):
        folder = self.data / "builds" / name / build_id
        folder.mkdir(parents=True)
        facts = {"format": 2, "project": name, "id": build_id, "commit": commit, "toolkit": "13.4.92"}
        (folder / "build.json").write_text(json.dumps(facts) if text is None else text, encoding="utf-8")
        (folder.parent / "current").symlink_to(build_id)

    def clone(self, name, origin):
        folder = self.data / "sources" / name
        folder.mkdir(parents=True)
        repo = Repo(folder)
        repo.commit("a build", {"README.md": "a build\n"})
        repo.git("remote", "add", "origin", origin)
        return repo.git("rev-parse", "HEAD")

    def test_each_backend_at_its_current_build_with_its_commit_s_date(self):
        commit = self.clone("llama.cpp", "https://github.com/ggml-org/llama.cpp.git")
        (self.data / "builds" / "llama.cpp" / "b12111").mkdir(parents=True)
        self.build("llama.cpp", "b12227", commit)
        self.assertEqual(dependencies.builds(self.data),
                         [project("llama.cpp", "ggml-org/llama.cpp", "b12227", "2026-10-01T10:00:00Z")])

    def test_a_backend_that_cannot_be_asked_says_why(self):
        (self.data / "builds" / "audio.cpp" / "v0.9.4").mkdir(parents=True)
        self.build("llama.cpp", "b12227", "0" * 40)
        self.clone("llama.cpp", "https://github.com/ggml-org/llama.cpp")
        self.build("stable-diffusion.cpp", "master-1071-cfdf3cf", "cfdf3cf037a9e9ea74f1ce1793e9daf25670baca")
        self.build("whisper.cpp", "v1.9.5", text='{"id": "v1.9.5"}')
        self.build("x.cpp", "v1", self.clone("x.cpp", "https://gitlab.com/o/x.cpp.git"))
        self.build("y.cpp", "v1", text='{"id": "v1", "commit": null}')
        self.build(".partial", "v2", "0" * 40)
        (self.data / "builds" / "notes.txt").write_text("not a project\n", encoding="utf-8")
        self.assertEqual(dependencies.builds(self.data), [
            project("audio.cpp", None, None, reason="not asked: no current build"),
            project("llama.cpp", "ggml-org/llama.cpp", "b12227"),
            project("stable-diffusion.cpp", None, "master-1071-cfdf3cf",
                    reason="not asked: no clone with an origin under sources/stable-diffusion.cpp"),
            project("whisper.cpp", None, None, reason="not asked: builds/whisper.cpp/current/build.json unreadable"),
            project("x.cpp", None, "v1", reason="not asked: no GitHub repository"),
            project("y.cpp", None, None, reason="not asked: builds/y.cpp/current/build.json unreadable")])

    def test_no_builds_folder_or_no_data_folder(self):
        self.assertEqual(dependencies.builds(self.data), [])
        self.assertEqual(dependencies.builds(None), [])

    def test_a_sources_folder_that_is_no_clone_takes_nothing_from_a_repository_around_it(self):
        self.data.mkdir()
        around = Repo(self.data)
        around.commit("the data folder", {"README.md": "data\n"})
        around.git("remote", "add", "origin", "https://github.com/o/data.git")
        (self.data / "sources" / "llama.cpp").mkdir(parents=True)
        self.build("llama.cpp", "b12227", around.git("rev-parse", "HEAD"))
        self.assertEqual(dependencies.builds(self.data), [project(
            "llama.cpp", None, "b12227", reason="not asked: no clone with an origin under sources/llama.cpp")])

    def test_a_commit_the_partial_clone_lacks_is_not_fetched(self):
        (Path(self.tmp.name) / "origin").mkdir()
        origin = Repo(Path(self.tmp.name) / "origin")
        origin.git("config", "uploadpack.allowFilter", "true")
        origin.commit("one", {"a": "a\n"})
        clone = self.data / "sources" / "llama.cpp"
        subprocess.run(["git", "clone", "-q", "--filter=blob:none", f"file://{origin.folder}", str(clone)],
                       check=True, env=GIT_ENV)
        subprocess.run(["git", "-C", str(clone), "remote", "set-url", "origin", "https://github.com/o/llama.cpp"],
                       check=True, env=GIT_ENV)
        subprocess.run(["git", "-C", str(clone), "config", f"url.file://{origin.folder}.insteadOf",
                        "https://github.com/o/llama.cpp"], check=True, env=GIT_ENV)
        origin.commit("two", {"b": "b\n"})
        later = origin.git("rev-parse", "HEAD")
        self.build("llama.cpp", "b2", later)
        self.assertEqual(dependencies.builds(self.data), [project("llama.cpp", "o/llama.cpp", "b2")])
        lacking = subprocess.run(["git", "-C", str(clone), "cat-file", "-e", later], capture_output=True,
                                 env={**GIT_ENV, "GIT_NO_LAZY_FETCH": "1"})
        self.assertNotEqual(lacking.returncode, 0)
        fetched = subprocess.run(["git", "-C", str(clone), "log", "-1", "--format=%cI", later], capture_output=True,
                                 text=True, env=GIT_ENV)
        self.assertEqual((fetched.returncode, fetched.stdout), (0, "2026-10-01T10:00:00Z\n"))


class CleanTest(unittest.TestCase):
    def page(self, *bodies):
        return [{"tag": f"v{number}", "created": "2026-11-01T00:00:00Z", "body": body}
                for number, body in enumerate(bodies, 1)]

    def test_the_project_s_template_and_the_release_s_assets_are_dropped(self):
        page = dependencies.releases_of(captured("github-releases-ggml-org-llama.cpp.http").body)
        found = dependencies.clean(page, "ggml-org/llama.cpp")
        self.assertEqual((len(found), found["b12242"]), (12, B12242))

    def test_a_line_in_more_than_half_the_page_and_in_two_releases_is_the_template(self):
        self.assertEqual(dependencies.clean(self.page("shared\nonly here", "shared\n"), "o/p"),
                         {"v1": "only here", "v2": ""})
        self.assertEqual(dependencies.clean(self.page("alone on its page\n"), "o/p"), {"v1": "alone on its page"})
        self.assertEqual(dependencies.clean(self.page("half\none", "half\ntwo", "three", "four"), "o/p"),
                         {"v1": "half\none", "v2": "half\ntwo", "v3": "three", "v4": "four"})

    def test_the_links_to_the_release_s_own_assets_and_blank_runs(self):
        body = ("- [a](https://github.com/O/P/releases/download/v1/a.zip)\r\nfirst\n\n\n  \n"
                "- [b](https://github.com/o/p/releases/download/v12/b.zip)\n\n")
        self.assertEqual(dependencies.clean(self.page(body, None), "o/p"),
                         {"v1": "first\n\n- [b](https://github.com/o/p/releases/download/v12/b.zip)", "v2": ""})

    def test_a_note_past_8_kb_is_cut_on_a_character(self):
        self.assertEqual(dependencies.cut("é" * 5000), ("é" * 4096, 10000))
        self.assertEqual(dependencies.cut("a" * 8192), ("a" * 8192, None))


class ReleaseNotesTest(unittest.TestCase):
    def notes_of(self, found, replaced=None, refused=None):
        web = Captured(replaced)
        record, status = dependencies.release_notes(found, web, refused)
        return record, status, web.asked

    def test_the_pin_at_the_newest_release_has_no_note_to_read(self):
        self.assertEqual(self.notes_of(LLAMA_SWAP), (notes(
            "llama-swap", "mostlygeek/llama-swap", "v271", anchor="2026-11-08T05:16:44Z", newest="v271"), None,
            [RELEASES.format("mostlygeek/llama-swap")]))

    def test_the_releases_after_an_older_pin_with_their_notes(self):
        record, status, _ = self.notes_of({**LLAMA_SWAP, "current": "v263"})
        self.assertEqual([release["tag"] for release in record["releases"]], [f"v{n}" for n in range(271, 263, -1)])
        self.assertEqual((record["anchor"], record["newest"], record["commits"], record["left_out"], status),
                         ("2026-10-10T06:16:39Z", "v271", None, [], None))
        v267 = record["releases"][4]
        self.assertEqual((v267["tag"], v267["created"], v267["cut_from"]), ("v267", "2026-10-26T04:46:48Z", None))
        self.assertIn("a new capacity based router", v267["note"])

    def test_a_build_with_no_release_anchors_by_its_commit_s_date(self):
        record, _, _ = self.notes_of(LLAMA_CPP)
        self.assertEqual([release["tag"] for release in record["releases"]], AFTER_B12227)
        self.assertEqual(record["releases"][0], {"tag": "b12242", "created": "2026-11-10T00:44:07Z", "note": B12242,
                                                 "cut_from": None})
        self.assertEqual((record["anchor"], record["newest"], record["left_out"]),
                         ("2026-11-09T14:54:27+02:00", "b12242", []))

    def test_a_build_older_than_the_page_says_what_is_left_out(self):
        record, _, _ = self.notes_of({**LLAMA_CPP, "current": "b12111", "date": "2026-11-06T08:00:00+02:00"})
        self.assertEqual([release["tag"] for release in record["releases"]], AFTER_B12227 + ["b12224"])
        self.assertEqual(record["left_out"], ["the releases before b12224, past the page"])

    def test_a_project_s_pre_releases_are_no_release(self):
        # llama.cpp's nightly `b<N>`, pre-releases about its weekly `vX.Y.Z`.
        page = {RELEASES.format("ggml-org/llama.cpp"): captured("github-releases-ggml-org-llama.cpp-v1.0.0.http")}
        record, _, _ = self.notes_of({**LLAMA_CPP, "current": "v1.0.0", "date": "2026-11-13T11:09:55+02:00"}, page)
        self.assertEqual(record, notes("llama.cpp", "ggml-org/llama.cpp", "v1.0.0", anchor="2026-11-13T10:20:13Z",
                                       newest="v1.0.0"))
        record, _, _ = self.notes_of({**LLAMA_CPP, "current": "v0.9.0", "date": "2026-11-06T08:00:00+02:00"}, page)
        self.assertEqual(([release["tag"] for release in record["releases"]], record["newest"], record["left_out"]),
                         (["v1.0.0"], "v1.0.0", ["the releases before b12359, past the page"]))
        v1 = record["releases"][0]
        self.assertIn("\n\n## Overview\n\nllama.cpp v1.0.0 removes the deprecated public API", v1["note"])
        self.assertEqual(v1["cut_from"], 28800)
        # A page of pre-releases alone may hide releases past it.
        nightlies = dependencies.Response(200, {"link": '<https://api.github.com/x?page=2>; rel="next"'}, json.dumps([
            {"tag_name": "b2", "prerelease": True, "created_at": "2026-11-14T00:00:00Z", "body": "two"},
            {"tag_name": "b1", "prerelease": True, "created_at": "2026-11-13T00:00:00Z", "body": "one"}]).encode())
        record, _, _ = self.notes_of({**LLAMA_CPP, "current": "v0.9.0", "date": "2026-11-06T08:00:00+02:00"},
                                     {RELEASES.format("ggml-org/llama.cpp"): nightlies})
        self.assertEqual(record, notes("llama.cpp", "ggml-org/llama.cpp", "v0.9.0", anchor="2026-11-06T08:00:00+02:00",
                                       left_out=["the releases before b1, past the page"]))

    def test_a_build_neither_on_the_page_nor_dated_reads_no_note(self):
        self.assertEqual(self.notes_of({**LLAMA_CPP, "date": None})[0], notes(
            "llama.cpp", "ggml-org/llama.cpp", "b12227", newest="b12242",
            left_out=["the page's 12 releases: no date for b12227 on the page or in a clone"]))

    def test_empty_notes_bring_the_commits_since_the_build(self):
        record, status, asked = self.notes_of(SD_CPP)
        self.assertEqual(asked, [RELEASES.format("leejet/stable-diffusion.cpp"), SD_COMMITS])
        self.assertEqual([(release["tag"], release["note"]) for release in record["releases"]], [
            ("master-1079-a3c49b1", ""), ("master-1078-73fef08", ""), ("master-1076-46da352", ""),
            ("master-1073-991f825", ""), ("master-1072-008b9a2", "")])
        self.assertEqual([commit["sha"][:7] for commit in record["commits"]],
                         ["a3c49b1", "73fef08", "d36f72b", "46da352", "c44cbec", "fbb381d", "991f825", "008b9a2"])
        self.assertEqual(record["commits"][0], {
            "sha": "a3c49b13be5a64a0b4da2b576af3a17e8ae1ccbf", "date": "2026-11-09T17:58:04Z",
            "message": "refactor: unify ggml operation helpers around runner context (#2314)", "cut_from": None})
        self.assertEqual((record["anchor"], record["left_out"], record["github"], status),
                         ("2026-11-09T14:16:04Z", [], None, None))

    def test_the_commits_since_a_build_with_no_release_are_asked_from_its_commit_s_date_in_utc(self):
        record, _, asked = self.notes_of({**SD_CPP, "current": "master-1071-unreleased"})
        self.assertEqual(asked, [RELEASES.format("leejet/stable-diffusion.cpp"), SD_COMMITS])
        self.assertEqual((record["anchor"], len(record["releases"]), len(record["commits"])),
                         ("2026-11-09T22:16:04+08:00", 5, 8))

    def test_a_page_of_commits_past_its_end_says_what_is_left_out(self):
        page = captured("github-commits-leejet-stable-diffusion.cpp.http")
        more = dependencies.Response(200, {**page.headers, "link": '<https://api.github.com/x?page=2>; rel="next"'},
                                     page.body)
        record, _, _ = self.notes_of(SD_CPP, {SD_COMMITS: more})
        self.assertEqual((len(record["commits"]), record["left_out"]), (8, ["the commits before 008b9a2, past the page"]))

    def test_a_note_and_a_message_past_8_kb_are_cut(self):
        page = json.dumps([{"tag_name": "v2", "created_at": "2026-11-02T00:00:00Z", "body": "é" * 5000},
                           {"tag_name": "v1", "created_at": "2026-11-01T00:00:00Z", "body": "the build"}]).encode()
        record, _, _ = self.notes_of(project("p", "o/p", "v1"), {RELEASES.format("o/p"): dependencies.Response(
            200, {}, page)})
        self.assertEqual(record["releases"], [{"tag": "v2", "created": "2026-11-02T00:00:00Z", "note": "é" * 4096,
                                               "cut_from": 10000}])
        empty = json.dumps([{"tag_name": "v2", "created_at": "2026-11-02T00:00:00Z", "body": ""},
                            {"tag_name": "v1", "created_at": "2026-11-01T00:00:00Z", "body": None}]).encode()
        commits = json.dumps([{"sha": "b" * 40, "commit": {"committer": {"date": "2026-11-02T00:00:00Z"},
                                                           "message": "a" * 9000}}]).encode()
        record, _, _ = self.notes_of(project("p", "o/p", "v1"), {
            RELEASES.format("o/p"): dependencies.Response(200, {}, empty),
            "https://api.github.com/repos/o/p/commits?sha=v2&since=2026-11-01T00%3A00%3A00Z&per_page=100":
                dependencies.Response(200, {}, commits)})
        self.assertEqual(record["commits"], [{"sha": "b" * 40, "date": "2026-11-02T00:00:00Z", "message": "a" * 8192,
                                              "cut_from": 9000}])

    def test_github_refusing_or_not_asked(self):
        releases = RELEASES.format("mostlygeek/llama-swap")
        self.assertEqual(self.notes_of(LLAMA_SWAP, {releases: captured("github-rate-limit.http")}), (notes(
            "llama-swap", "mostlygeek/llama-swap", "v271", github="HTTP 403, rate limit reached"), 403, [releases]))
        self.assertEqual(self.notes_of(LLAMA_SWAP, {releases: NONE}, 429), (notes(
            "llama-swap", "mostlygeek/llama-swap", "v271", github="not asked: GitHub answered 429 earlier"), 429, []))
        self.assertEqual(self.notes_of(LLAMA_SWAP, {releases: NONE}), (notes(
            "llama-swap", "mostlygeek/llama-swap", "v271", github="HTTP 404"), None, [releases]))
        reason = "not asked: no current build"
        self.assertEqual(self.notes_of(project("audio.cpp", None, None, reason=reason)),
                         (notes("audio.cpp", None, None, github=reason), None, []))

    def test_an_answer_that_is_no_list_of_releases_or_commits(self):
        releases = RELEASES.format("mostlygeek/llama-swap")
        for body in (b'{"message": "Moved"}', b"[{\"tag_name\": 271}]", b"not json",
                     b'[{"tag_name": "v271", "created_at": "yesterday", "body": ""}]',
                     b'[{"tag_name": "v271", "created_at": "2026-11-08T05:16:44", "body": ""}]',
                     b'[{"tag_name": "v271", "created_at": "2026-11-08T05:16:44Z", "body": 5}]',
                     b'[{"tag_name": "v271", "created_at": "2026-11-08T05:16:44Z", "body": "", "prerelease": 0}]'):
            record, status, _ = self.notes_of(LLAMA_SWAP, {releases: dependencies.Response(200, {}, body)})
            self.assertEqual((record["github"], record["releases"], status), ("GitHub's answer is no release list", [],
                                                                              None), body)
        for body in (b"{}", b'[{"sha": "a"}]', b'[{"sha": "a", "commit": {"committer": {"date": "now"}, "message": ""}}]',
                     b'[{"sha": "a", "commit": {"committer": {"date": "2026-11-09T17:58:04"}, "message": ""}}]'):
            record, status, _ = self.notes_of(SD_CPP, {SD_COMMITS: dependencies.Response(200, {}, body)})
            self.assertEqual((record["github"], record["commits"], len(record["releases"]), status),
                             ("GitHub's answer is no commit list", None, 5, None), body)
        record, status, _ = self.notes_of(SD_CPP, {SD_COMMITS: dependencies.Response(429, {}, b"")})
        self.assertEqual((record["github"], record["commits"], len(record["releases"]), status),
                         ("HTTP 429", None, 5, 429))

    def test_github_unreachable_stops_the_gathering(self):
        releases = RELEASES.format("mostlygeek/llama-swap")
        with self.assertRaises(dependencies.DependencyError) as raised:
            self.notes_of(LLAMA_SWAP, {releases: dependencies.DependencyError(f"{releases}: timed out")})
        self.assertEqual(str(raised.exception), f"{releases}: timed out")


class GatherTest(unittest.TestCase):
    def test_each_crate_of_crates_io_gets_its_record(self):
        github = Captured()
        found = probe(github=github)
        self.assertEqual(found, {"gathered": "2026-11-02", "rust_version": "1.89", "crates": [
            EQUIVALENT, FNV, HECK, HECK_5, STATIC_ASSERTIONS, UNICODE_SEGMENTATION], "left_out": [ITOA],
            "projects": []})

    def test_each_project_gets_its_release_notes_after_the_crates(self):
        github = Captured()
        found = probe(github=github, projects=[LLAMA_SWAP, LLAMA_CPP])
        self.assertEqual([record["project"] for record in found["projects"]], ["llama-swap", "llama.cpp"])
        self.assertEqual(found["projects"][0], notes("llama-swap", "mostlygeek/llama-swap", "v271",
                                                     anchor="2026-11-08T05:16:44Z", newest="v271"))
        self.assertEqual(github.asked[4:], [RELEASES.format("mostlygeek/llama-swap"), RELEASES.format("ggml-org/llama.cpp")])

    def test_a_refusal_on_a_crate_s_repository_stops_the_release_notes_too(self):
        github = Captured({"https://api.github.com/repos/withoutboats/heck": captured("github-rate-limit.http")})
        found = probe(github=github, projects=[LLAMA_SWAP, SD_CPP])
        self.assertEqual([record["github"] for record in found["projects"]],
                         ["not asked: GitHub answered 403 earlier"] * 2)
        self.assertEqual(len(github.asked), 3)

    def test_each_crate_and_each_repository_is_asked_once(self):
        github = Captured()
        probe(github=github)
        self.assertEqual(github.asked, [f"https://api.github.com/repos/{name}" for name in (
            "indexmap-rs/equivalent", "servo/rust-fnv", "withoutboats/heck", "nvzqz/static-assertions-rs")])

    def test_crates_io_is_asked_three_times_a_crate(self):
        crates_io = Captured()
        probe(crates_io=crates_io)
        self.assertEqual(len(crates_io.asked), 15)
        self.assertEqual(crates_io.asked[:3], ["https://crates.io/api/v1/crates/equivalent",
                                               "https://crates.io/api/v1/crates/equivalent/owners",
                                               "https://crates.io/api/v1/crates/equivalent/reverse_dependencies?per_page=1"])

    def test_a_repository_github_does_not_find_and_its_rate_limit_reached(self):
        github = Captured({"https://api.github.com/repos/indexmap-rs/equivalent": captured("crates-io-heck-owners.http"),
                           "https://api.github.com/repos/servo/rust-fnv": NONE,
                           "https://api.github.com/repos/withoutboats/heck": captured("github-rate-limit.http")})
        found = [(crate["name"], crate["github"]) for crate in probe(github=github)["crates"]]
        self.assertEqual(found, [("equivalent", "GitHub's answer is no repository"), ("fnv", "HTTP 404"),
                                 ("heck", "HTTP 403, rate limit reached"), ("heck", "HTTP 403, rate limit reached"),
                                 ("static_assertions", "not asked: GitHub answered 403 earlier"),
                                 ("unicode-segmentation", "not asked: transitive, released within a year")])
        self.assertEqual(len(github.asked), 3)

    def test_too_many_requests_stops_the_asking_too(self):
        github = Captured({"https://api.github.com/repos/servo/rust-fnv": dependencies.Response(429, {}, b"")})
        found = [crate["github"] for crate in probe(github=github)["crates"]]
        self.assertEqual(found[1:4], ["HTTP 429"] + ["not asked: GitHub answered 429 earlier"] * 2)
        self.assertEqual(len(github.asked), 2)

    def test_github_unreachable_stops_the_gathering(self):
        url = "https://api.github.com/repos/servo/rust-fnv"
        with self.assertRaises(dependencies.DependencyError) as raised:
            probe(github=Captured({url: dependencies.DependencyError(f"{url}: timed out")}))
        self.assertEqual(str(raised.exception), f"{url}: timed out")

    def test_crates_io_failing_stops_the_gathering(self):
        owners = "https://crates.io/api/v1/crates/fnv/owners"
        for answer, message in [(dependencies.Response(500, {}, b""), f"crates.io answered 500 for {owners}"),
                                (NONE, f"crates.io answered 404 for {owners}"),
                                (dependencies.Response(200, {}, b"5"), f"crates.io's answer for {owners} lacks `users`"),
                                (dependencies.Response(200, {}, (CAPTURES / "cargo-update-dry-run-none.txt").read_bytes()),
                                 f"crates.io's answer for {owners} is no JSON"),
                                (captured("crates-io-fnv-reverse-dependencies.http"),
                                 f"crates.io's answer for {owners} lacks `users`"),
                                (dependencies.DependencyError(f"{owners}: timed out"), f"{owners}: timed out")]:
            with self.assertRaises(dependencies.DependencyError) as raised:
                probe(crates_io=Captured({owners: answer}))
            self.assertEqual(str(raised.exception), message)


class LinesTest(unittest.TestCase):
    def test_the_facts_a_session_reads_first_then_the_summary(self):
        self.assertEqual(dependencies.lines(probe()), [
            "update: equivalent 1.0.1 -> 1.0.3",
            "old: fnv 1.0.7, last release 2020-05-14",
            "behind: heck 0.4.1, newest 0.5.0",
            "old: heck 0.4.1, last release 2024-03-12",
            "old: heck 0.5.0, last release 2024-03-12",
            "old: static_assertions 1.1.0, last release 2019-11-03",
            "moved: static_assertions, nvzqz/static-assertions-rs now nvzqz/static-assertions",
            "update: unicode-segmentation 1.13.1 -> 1.13.3",
            "yanked: unicode-segmentation 1.13.1",
            f"left out: {ITOA}, not from crates.io",
            "summary: 6 crates, 4 direct; 2 updates within the requirements, 1 behind the newest, 1 yanked, "
            "4 last released over a year ago; GitHub answered for 5 crates, 0 archived, 1 moved; 0 projects, "
            "0 behind their newest release",
        ])

    def test_the_release_notes_a_session_reads_first(self):
        projects = [dependencies.release_notes(found, Captured(), None)[0] for found in (
            LLAMA_SWAP, {**LLAMA_CPP, "current": "b12111", "date": "2026-11-06T08:00:00+02:00"}, SD_CPP)]
        projects.append(notes("whisper.cpp", "ggml-org/whisper.cpp", "v1.9.5", github="HTTP 404"))
        self.assertEqual(dependencies.lines({"gathered": "2026-11-10", "crates": [], "left_out": [],
                                             "projects": projects}), [
            "notes: llama.cpp b12111 -> b12242, 12 releases",
            "left out: llama.cpp, the releases before b12224, past the page",
            "notes: stable-diffusion.cpp master-1071-cfdf3cf -> master-1079-a3c49b1, 5 releases with empty notes, "
            "8 commits",
            "github: whisper.cpp, HTTP 404",
            "summary: 0 crates, 0 direct; 0 updates within the requirements, 0 behind the newest, 0 yanked, "
            "0 last released over a year ago; GitHub answered for 0 crates, 0 archived, 0 moved; 4 projects, "
            "2 behind their newest release",
        ])

    def test_a_year_ago_to_the_day_is_old_and_no_release_older_still(self):
        since = "2025-11-02"
        crates = [record("aged", "1.0.0", newest="1.0.0", yanked=False, last_release=since, releases_past_year=0,
                         repository="https://github.com/o/aged"),
                  record("gone", "1.0.0", newest=None, yanked=True, last_release=None, repository="https://github.com/o/gone")]
        github = Captured({"https://api.github.com/repos/o/aged": NONE, "https://api.github.com/repos/o/gone": NONE})
        self.assertIsNone(dependencies.ask_github(crates, github, since))
        self.assertEqual([crate["github"] for crate in crates], ["HTTP 404", "HTTP 404"])
        self.assertEqual(dependencies.lines({"gathered": "2026-11-02", "crates": crates, "left_out": [],
                                             "projects": []})[:-1], [
            "old: aged 1.0.0, last release 2025-11-02", "github: aged, HTTP 404", "yanked: gone 1.0.0",
            "old: gone 1.0.0, every version yanked", "github: gone, HTTP 404"])
        facts = {"max_stable_version": "1.0.0", "downloads": 1, "recent_downloads": 1, "repository": None, "owners": [],
                 "dependents": 0, "versions": [("1.0.0", since, False), ("0.9.0", "2025-11-03", False)]}
        made = dependencies.record({"name": "aged", "version": "1.0.0"}, [], {"update": None, "available": None,
                                                                          "requires": None}, facts, since, 0)
        self.assertEqual((made["last_release"], made["releases_past_year"]), ("2025-11-03", 1))

    def test_yanked_archived_held_back_and_github_unanswered(self):
        crates = [
            record("clap", "4.0.32", direct=["normal"], update="4.4.18", available="4.6.7", requires="Rust 1.85",
                   newest="4.6.7", yanked=False, last_release="2026-09-14",
                   github={"asked": "clap-rs/clap", "repository": "clap-rs/clap", "archived": True,
                           "pushed": "2026-09-14"}),
            record("memchr", "2.8.3", newest="2.8.3", yanked=False, last_release="2026-07-08",
                   github={"asked": "burntsushi/memchr", "repository": "BurntSushi/memchr", "archived": False,
                           "pushed": "2026-07-08"}),
            record("windows-link", "0.2.1", requires="Rust 1.71", newest="0.2.1", yanked=True,
                   last_release="2026-09-03", github="not asked: GitHub answered 403 earlier"),
        ]
        self.assertEqual(dependencies.lines({"gathered": "2026-11-02", "rust_version": "1.70", "crates": crates,
                                             "left_out": ["mine 0.1.0 git+https://example.org/mine"],
                                             "projects": []}), [
            "update: clap 4.0.32 -> 4.4.18",
            "behind: clap 4.0.32, newest 4.6.7",
            "requires: clap 4.0.32, Rust 1.85 for 4.6.7",
            "archived: clap, clap-rs/clap",
            "requires: windows-link 0.2.1, Rust 1.71",
            "yanked: windows-link 0.2.1",
            "github: windows-link, not asked: GitHub answered 403 earlier",
            "left out: mine 0.1.0 git+https://example.org/mine, not from crates.io",
            "summary: 3 crates, 1 direct; 1 update within the requirements, 1 behind the newest, 1 yanked, "
            "0 last released over a year ago; GitHub answered for 2 crates, 1 archived, 0 moved; 0 projects, "
            "0 behind their newest release",
        ])


class MainTest(unittest.TestCase):
    def test_the_facts_are_written_whole_and_their_lines_printed(self):
        with tempfile.TemporaryDirectory() as tmp:
            (Path(tmp) / "repo").mkdir()
            repo = Repo(Path(tmp) / "repo")
            head = repo.commit("a crate", {"Cargo.toml": "[package]\nname = \"facts\"\n",
                                           "src/domain/gateway.rs": GATEWAY})
            review = Path(tmp) / "reviews" / "2026-11-02-dependencies"
            data = Path(tmp) / "data"
            (data / "ai-ctl" / "builds" / "llama.cpp" / "b12111").mkdir(parents=True)
            (data / "ai-ctl" / "sources" / "llama.cpp").mkdir(parents=True)
            clone = Repo(data / "ai-ctl" / "sources" / "llama.cpp")
            clone.commit("b12111", {"README.md": "llama.cpp\n"})
            clone.git("remote", "add", "origin", "https://github.com/ggml-org/llama.cpp.git")
            (data / "ai-ctl" / "builds" / "llama.cpp" / "b12111" / "build.json").write_text(json.dumps(
                {"id": "b12111", "commit": clone.git("rev-parse", "HEAD")}), encoding="utf-8")
            (data / "ai-ctl" / "builds" / "llama.cpp" / "current").symlink_to("b12111")
            projects = [LLAMA_SWAP, project("llama.cpp", "ggml-org/llama.cpp", "b12111", "2026-10-01T10:00:00Z")]
            ran = []

            def cargo(folder, *args):
                ran.append((Path(folder), args))
                capture = "cargo-metadata-dependencies-probe.json" if args[0] == "metadata" else "cargo-update-dry-run-probe.txt"
                text = (CAPTURES / capture).read_text(encoding="utf-8")
                return subprocess.CompletedProcess(["cargo", *args], 0, *((text, "") if args[0] == "metadata" else ("", text)))

            paced = []

            class Recorded(dependencies.Paced):
                def get(self, url, accept="application/json"):
                    paced.append(url)
                    return super().get(url, accept)

            out = io.StringIO()
            with (mock.patch.object(dependencies, "cargo", cargo), mock.patch.object(dependencies, "Web", Captured),
                  mock.patch.object(dependencies, "Paced", Recorded), mock.patch.object(dependencies, "INTERVAL", 0),
                  mock.patch.object(dependencies, "today", lambda: TODAY),
                  mock.patch.object(dependencies, "unsafe_count", lambda folder: UNSAFE[folder.name.rsplit("-", 1)[0]]),
                  mock.patch.dict(os.environ, {"XDG_DATA_HOME": str(data)}), redirect_stdout(out)):
                code = dependencies.main([str(review), "--repo", str(repo.folder)])
            written = review / "dependencies.json"
            self.assertEqual(code, 0)
            self.assertEqual((len(paced), sum(url.startswith("https://api.github.com/") for url in paced)), (21, 6))
            self.assertEqual(ran, [(repo.folder.resolve(), ("metadata", "--locked", "--format-version", "1")),
                                   (repo.folder.resolve(), ("update", "--dry-run", "--verbose"))])
            self.assertEqual(json.loads(written.read_text(encoding="utf-8")), {"commit": head, **probe(projects=projects)})
            self.assertEqual(list(json.loads(written.read_text(encoding="utf-8"))),
                             ["commit", "gathered", "rust_version", "crates", "left_out", "projects"])
            self.assertEqual(out.getvalue().splitlines(),
                             dependencies.lines(probe(projects=projects)) + [f"written: {written}"])
            self.assertEqual([path.name for path in review.iterdir()], ["dependencies.json"])

    def test_a_failure_writes_nothing(self):
        def broken_metadata(folder, *args):
            return subprocess.CompletedProcess(["cargo", *args], 0, (CAPTURES / "cargo-update-dry-run-none.txt")
                                               .read_text(encoding="utf-8"), "")

        def interrupted(folder, *args):
            raise KeyboardInterrupt

        def lacking(folder, *args):
            return subprocess.CompletedProcess(["cargo", *args], 0, "{}", "")

        with tempfile.TemporaryDirectory() as tmp:
            (Path(tmp) / "repo").mkdir()
            repo = Repo(Path(tmp) / "repo")
            repo.commit("a crate", {"Cargo.toml": "[package]\nname = \"facts\"\n"})
            (Path(tmp) / "file").write_text("a file\n")
            fine = mock.Mock(side_effect=lambda folder, *args: subprocess.CompletedProcess(
                args, 0, *(((CAPTURES / "cargo-metadata-dependencies-probe.json").read_text(encoding="utf-8"), "")
                           if args[0] == "metadata" else ("", ""))))
            for cargo, review, code, message in [
                    (broken_metadata, Path(tmp) / "review", 3, "dependencies.py: cargo metadata's output is no JSON: "),
                    (lacking, Path(tmp) / "review", 3, "dependencies.py: cargo metadata's output lacks 'packages'"),
                    (interrupted, Path(tmp) / "review", 130, "dependencies.py: interrupted; nothing written"),
                    (fine, Path(tmp) / "file" / "review", 3, "dependencies.py: [Errno 20] Not a directory: ")]:
                err = io.StringIO()
                with (mock.patch.object(dependencies, "cargo", cargo), mock.patch.object(dependencies, "Web", Captured),
                      mock.patch.object(dependencies, "INTERVAL", 0),
                      mock.patch.object(dependencies, "unsafe_count", lambda folder: 0),
                      mock.patch.dict(os.environ, {"XDG_DATA_HOME": str(Path(tmp) / "data")}), redirect_stderr(err),
                      redirect_stdout(io.StringIO())):
                    found = dependencies.main([str(review), "--repo", str(repo.folder)])
                self.assertEqual(found, code, err.getvalue())
                self.assertTrue(err.getvalue().startswith(message), err.getvalue())
                self.assertFalse((Path(tmp) / "review").exists())


class CommandTest(unittest.TestCase):
    def run_script(self, *args):
        return subprocess.run([str(SCRIPT), *map(str, args)], capture_output=True, text=True, env=GIT_ENV)

    def test_a_folder_is_needed(self):
        self.assertEqual(self.run_script().returncode, 2)

    def test_a_folder_outside_a_repository_is_refused_and_nothing_written(self):
        with tempfile.TemporaryDirectory() as tmp:
            review = Path(tmp) / "review"
            done = self.run_script(review, "--repo", tmp)
            self.assertEqual((done.returncode, done.stdout), (3, ""))
            self.assertEqual(done.stderr, f"dependencies.py: {tmp} is not a git repository\n")
            self.assertFalse(review.exists())

    def test_cargo_failing_stops_the_gathering_and_nothing_is_written(self):
        with tempfile.TemporaryDirectory() as tmp:
            (Path(tmp) / "repo").mkdir()
            repo = Repo(Path(tmp) / "repo")
            repo.commit("no crate", {"README.md": "nothing\n"})
            review = Path(tmp) / "review"
            done = self.run_script(review, "--repo", repo.folder)
            self.assertEqual((done.returncode, done.stdout), (3, ""))
            self.assertTrue(done.stderr.startswith("dependencies.py: cargo metadata: "), done.stderr)
            self.assertFalse(review.exists())


if __name__ == "__main__":
    unittest.main()
