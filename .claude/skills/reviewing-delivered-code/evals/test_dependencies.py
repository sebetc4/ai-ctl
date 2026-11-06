#!/usr/bin/env python3
"""Unit tests of scripts/dependencies.py, run on the outputs of cargo, crates.io and GitHub captured by a probe.

Usage: test_dependencies.py [unittest options]
"""

import io
import json
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
    return "github-" + url.removeprefix("https://api.github.com/repos/").replace("/", "-") + ".http"


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


def probe(crates_io=None, github=None):
    metadata = json.loads((CAPTURES / "cargo-metadata-dependencies-probe.json").read_text(encoding="utf-8"))
    update = (CAPTURES / "cargo-update-dry-run-probe.txt").read_text(encoding="utf-8")
    return dependencies.gather(metadata, update, crates_io or Captured(), github or Captured(), TODAY,
                               lambda package: UNSAFE[package["name"]])


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


class GatherTest(unittest.TestCase):
    def test_each_crate_of_crates_io_gets_its_record(self):
        github = Captured()
        found = probe(github=github)
        self.assertEqual(found, {"gathered": "2026-11-02", "rust_version": "1.89", "crates": [
            EQUIVALENT, FNV, HECK, HECK_5, STATIC_ASSERTIONS, UNICODE_SEGMENTATION], "left_out": [ITOA]})

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
            "4 last released over a year ago; GitHub answered for 5 crates, 0 archived, 1 moved",
        ])

    def test_a_year_ago_to_the_day_is_old_and_no_release_older_still(self):
        since = "2025-11-02"
        crates = [record("aged", "1.0.0", newest="1.0.0", yanked=False, last_release=since, releases_past_year=0,
                         repository="https://github.com/o/aged"),
                  record("gone", "1.0.0", newest=None, yanked=True, last_release=None, repository="https://github.com/o/gone")]
        github = Captured({"https://api.github.com/repos/o/aged": NONE, "https://api.github.com/repos/o/gone": NONE})
        dependencies.ask_github(crates, github, since)
        self.assertEqual([crate["github"] for crate in crates], ["HTTP 404", "HTTP 404"])
        self.assertEqual(dependencies.lines({"gathered": "2026-11-02", "crates": crates, "left_out": []})[:-1], [
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
                                             "left_out": ["mine 0.1.0 git+https://example.org/mine"]}), [
            "update: clap 4.0.32 -> 4.4.18",
            "behind: clap 4.0.32, newest 4.6.7",
            "requires: clap 4.0.32, Rust 1.85 for 4.6.7",
            "archived: clap, clap-rs/clap",
            "requires: windows-link 0.2.1, Rust 1.71",
            "yanked: windows-link 0.2.1",
            "github: windows-link, not asked: GitHub answered 403 earlier",
            "left out: mine 0.1.0 git+https://example.org/mine, not from crates.io",
            "summary: 3 crates, 1 direct; 1 update within the requirements, 1 behind the newest, 1 yanked, "
            "0 last released over a year ago; GitHub answered for 2 crates, 1 archived, 0 moved",
        ])


class MainTest(unittest.TestCase):
    def test_the_facts_are_written_whole_and_their_lines_printed(self):
        with tempfile.TemporaryDirectory() as tmp:
            (Path(tmp) / "repo").mkdir()
            repo = Repo(Path(tmp) / "repo")
            head = repo.commit("a crate", {"Cargo.toml": "[package]\nname = \"facts\"\n"})
            review = Path(tmp) / "reviews" / "2026-11-02-dependencies"
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
                  redirect_stdout(out)):
                code = dependencies.main([str(review), "--repo", str(repo.folder)])
            written = review / "dependencies.json"
            self.assertEqual(code, 0)
            self.assertEqual((len(paced), sum(url.startswith("https://api.github.com/") for url in paced)), (19, 4))
            self.assertEqual(ran, [(repo.folder.resolve(), ("metadata", "--locked", "--format-version", "1")),
                                   (repo.folder.resolve(), ("update", "--dry-run", "--verbose"))])
            self.assertEqual(json.loads(written.read_text(encoding="utf-8")), {"commit": head, **probe()})
            self.assertEqual(list(json.loads(written.read_text(encoding="utf-8"))),
                             ["commit", "gathered", "rust_version", "crates", "left_out"])
            self.assertEqual(out.getvalue().splitlines(), dependencies.lines(probe()) + [f"written: {written}"])
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
                      mock.patch.object(dependencies, "unsafe_count", lambda folder: 0), redirect_stderr(err),
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
