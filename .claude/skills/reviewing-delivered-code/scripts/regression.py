#!/usr/bin/env python3
"""Run the recorded mutants: each applied in a copy of a commit, and its killing test run.

Usage:
  regression.py [<id> ...] [--at <commit>] [--timeout <seconds>] [--repo <folder>]

The mutants are those of docs/reviews/delivered-code/mutants.json at <commit>, HEAD by
default, or those named. Each is an exact replacement: its file, the text replaced, which must
occur once, its replacement, and the test that kills it, a libtest path. The copy is the
commit's `git archive`, in the temporary folder, TMPDIR's, and built there with its own
target folder; it is removed at the end, interrupted or not, and the repository is only read.

Each mutant's test runs first without it, then with it, `cargo test --locked -- --exact`, and
is stopped past --timeout, 300 s by default. One line per mutant, `<outcome>: <id> <file>:
<why>`, then a summary line:

caught     its test fails with the mutant, or runs past the timeout;
missed     its test passes with the mutant, does not run, ignored or skipped, or no test
           has its name;
gone       its file or its text no longer occurs: record it again, or retire it;
ambiguous  its text occurs more than once;
unviable   the mutant does not build;
failing    its test fails, or runs past the timeout, without the mutant.

Exit codes: 0 every mutant caught, or none recorded; 1 a mutant reported; 2 wrong arguments;
3 the run could not be made, the reason on stderr: a commit, mutants.json or a mutant named
wrong, or the copy not made or not building.
"""

import argparse
import json
import os
import posixpath
import re
import signal
import subprocess
import sys
import tarfile
import tempfile
from pathlib import Path

sys.dont_write_bytecode = True
import scope  # noqa: E402

MUTANTS = "docs/reviews/delivered-code/mutants.json"
KEYS = ("id", "kind", "file", "text", "replacement", "test", "review", "finding")
OUTCOMES = ("caught", "missed", "gone", "ambiguous", "unviable", "failing")
TEST_RE = re.compile(r"^test (\S+) \.\.\. (.+)$", re.M)


class RegressionError(Exception):
    pass


def results(output):
    """{test: its status} from the `test <name> ... <status>` lines of `cargo test`'s stdout, over its test
    binaries: libtest's, and those of a harness of its own, which prints no `test result:` line."""
    return dict(TEST_RE.findall(output))


def mutant_faults(index, mutant):
    if not isinstance(mutant, dict):
        return [f"mutants[{index}] is no mapping"]
    name = mutant["id"] if isinstance(mutant.get("id"), str) and mutant["id"].strip() else f"mutants[{index}]"
    faults = [f"{name}: `{key}` is missing" for key in KEYS if key not in mutant]
    faults += [f"{name}: `{key}` is no key of it" for key in mutant if key not in KEYS]
    for key in ("id", "kind", "file", "text", "test", "review"):
        if key in mutant and (not isinstance(mutant[key], str) or not mutant[key].strip()):
            faults.append(f"{name}: `{key}` is no text")
    if "replacement" in mutant and not isinstance(mutant["replacement"], str):
        faults.append(f"{name}: `replacement` is no text")
    elif "replacement" in mutant and mutant["replacement"] == mutant.get("text"):
        faults.append(f"{name}: `replacement` is its `text`")
    finding = mutant.get("finding")
    if "finding" in mutant and (not isinstance(finding, int) or isinstance(finding, bool) or finding < 1):
        faults.append(f"{name}: `finding` is no number of 1 or more")
    path = mutant.get("file")
    if isinstance(path, str) and path.strip() and (
            posixpath.isabs(path) or posixpath.normpath(path).split("/")[0] == ".."):
        faults.append(f"{name}: `file` is no relative path inside the repository")
    return faults


def recorded(git, commit):
    """The mutants of mutants.json at `commit`, checked whole; none when it is not there."""
    done = git.run("show", f"{commit}:{MUTANTS}", check=False)
    if done.returncode != 0:
        if git.run("cat-file", "-e", f"{commit}:{MUTANTS}", check=False).returncode != 0:
            return []
        raise RegressionError(f"mutants.json: {done.stderr.decode(errors='replace').strip()}")
    try:
        mutants = json.loads(done.stdout.decode("utf-8"))
    except (UnicodeDecodeError, json.JSONDecodeError) as error:
        raise RegressionError(f"mutants.json: {error}") from None
    if not isinstance(mutants, dict) or set(mutants) != {"mutants"} or not isinstance(mutants["mutants"], list):
        raise RegressionError("mutants.json: it is no mapping of `mutants` to a list")
    faults = [fault for k, mutant in enumerate(mutants["mutants"]) for fault in mutant_faults(k, mutant)]
    ids = [mutant.get("id") for mutant in mutants["mutants"] if isinstance(mutant, dict)]
    faults += [f"{name} is listed twice" for name in sorted({i for i in ids if isinstance(i, str) and ids.count(i) > 1})]
    if faults:
        raise RegressionError("\n".join(f"mutants.json: {fault}" for fault in faults))
    return mutants["mutants"]


def chosen(mutants, names):
    if not names:
        return mutants
    ids = {mutant["id"] for mutant in mutants}
    unknown = [name for name in names if name not in ids]
    if unknown:
        listed = " and ".join([", ".join(unknown[:-1]), unknown[-1]] if len(unknown) > 1 else unknown)
        raise RegressionError(f"{listed} {'are' if len(unknown) > 1 else 'is'} not in mutants.json")
    return [mutant for mutant in mutants if mutant["id"] in names]


def extract(git, commit, copy):
    """The commit's files in `copy`, by `git archive`."""
    archive = subprocess.Popen(["git", "-C", str(git.folder), "archive", "--format=tar", commit],
                               stdout=subprocess.PIPE, stderr=subprocess.PIPE)
    try:
        with tarfile.open(fileobj=archive.stdout, mode="r|") as tar:
            tar.extractall(copy, filter="data")
    except tarfile.TarError as error:
        raise RegressionError(f"the copy of {commit} cannot be made: {error}") from None
    finally:
        archive.stdout.close()
        error = archive.stderr.read().decode(errors="replace").strip()
        archive.wait()
    if archive.returncode != 0:
        raise RegressionError(f"git archive {commit}: {error}")


class Cargo:
    """cargo in the copy, its target folder there, each command in a process group of its own."""

    def __init__(self, copy):
        self.copy = copy
        self.env = {**os.environ, "CARGO_TARGET_DIR": str(copy / "target"), "LC_ALL": "C",
                    "CARGO_TERM_COLOR": "never"}

    def run(self, args, timeout=None):
        """(exit code, stdout, stderr), the exit code None past `timeout`."""
        process = subprocess.Popen(["cargo", *args], cwd=self.copy, env=self.env, stdout=subprocess.PIPE,
                                   stderr=subprocess.PIPE, text=True, errors="replace", start_new_session=True)
        try:
            out, err = process.communicate(timeout=timeout)
            return process.returncode, out, err
        except subprocess.TimeoutExpired:
            stop(process)
            return None, "", ""
        except BaseException:
            stop(process)
            raise

    def build(self):
        code, _, err = self.run(["test", "--locked", "--no-run"])
        return code == 0, err

    def test(self, name, timeout):
        return self.run(["test", "--locked", "--", "--exact", name], timeout)


def stop(process):
    """Kills the process and every process of its group, then reaps it."""
    try:
        os.killpg(process.pid, signal.SIGKILL)
    except ProcessLookupError:
        pass
    process.communicate()


def occurrences(copy, mutant):
    """(the file's text, the count of the mutant's text in it), or (None, why it is gone)."""
    path = copy / mutant["file"]
    try:
        text = path.read_text(encoding="utf-8")
    except FileNotFoundError:
        return None, "its file no longer exists"
    except (OSError, UnicodeDecodeError) as error:
        return None, f"its file cannot be read: {error}"
    return text, text.count(mutant["text"])


def baseline(cargo, copy, mutant, timeout):
    """The outcome and why for a mutant that cannot run, None for one whose test passes without it."""
    text, count = occurrences(copy, mutant)
    if text is None:
        return "gone", count
    if count == 0:
        return "gone", "its text no longer occurs"
    if count > 1:
        return "ambiguous", f"its text occurs {count} times"
    code, out, _ = cargo.test(mutant["test"], timeout)
    if code is None:
        return "failing", f"{mutant['test']} runs past {timeout} s without the mutant"
    if code != 0:
        return "failing", f"{mutant['test']} fails without the mutant"
    status = results(out).get(mutant["test"])
    if status is None:
        return "missed", f"no test is named {mutant['test']}"
    if status != "ok":
        return "missed", f"{mutant['test']} does not run: {status}"
    return None


def mutated(cargo, copy, mutant, timeout):
    """The outcome and why of the mutant applied, the file given back after."""
    path = copy / mutant["file"]
    original = path.read_text(encoding="utf-8")
    path.write_text(original.replace(mutant["text"], mutant["replacement"], 1), encoding="utf-8")
    try:
        built, _ = cargo.build()
        if not built:
            return "unviable", "the mutant does not build"
        code, _, _ = cargo.test(mutant["test"], timeout)
    finally:
        path.write_text(original, encoding="utf-8")
    if code is None:
        return "caught", f"{mutant['test']} runs past {timeout} s with the mutant"
    if code != 0:
        return "caught", f"{mutant['test']} fails with the mutant"
    return "missed", f"{mutant['test']} passes with the mutant"


def regression(git, commit, mutants, timeout):
    """Runs the mutants in a copy of `commit`, printing each outcome; the count of each outcome."""
    counts = dict.fromkeys(OUTCOMES, 0)

    def report(mutant, outcome, why):
        counts[outcome] += 1
        print(f"{outcome}: {mutant['id']} {mutant['file']}: {why}", flush=True)

    with tempfile.TemporaryDirectory(prefix="regression-") as folder:
        copy = Path(folder)
        extract(git, commit, copy)
        cargo = Cargo(copy)
        built, err = cargo.build()
        if not built:
            raise RegressionError(f"the copy of {commit} does not build:\n{err.strip()}")
        runnable = []
        for mutant in mutants:
            found = baseline(cargo, copy, mutant, timeout)
            if found:
                report(mutant, *found)
            else:
                runnable.append(mutant)
        for mutant in runnable:
            report(mutant, *mutated(cargo, copy, mutant, timeout))
    return counts


def main(argv):
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("ids", nargs="*", help="the mutants to run, every one by default")
    parser.add_argument("--at", default="HEAD", help="the commit copied, HEAD by default")
    parser.add_argument("--timeout", type=int, default=300, help="the seconds a test may run, 300 by default")
    parser.add_argument("--repo", default=".", help="the repository, the current folder by default")
    args = parser.parse_args(argv)
    if args.timeout < 1:
        parser.error("--timeout takes 1 second or more")
    try:
        git = scope.repository(Path(args.repo))
        commit = scope.resolve(git, args.at)
        mutants = chosen(recorded(git, commit), args.ids)
        if not mutants:
            print(f"summary: no mutant recorded at {commit}")
            return 0
        counts = regression(git, commit, mutants, args.timeout)
    except (RegressionError, scope.ScopeError) as error:
        for line in str(error).splitlines():
            print(f"regression.py: {line}", file=sys.stderr)
        return 3
    except KeyboardInterrupt:
        print("regression.py: interrupted; the copy is removed", file=sys.stderr)
        return 130
    total = sum(counts.values())
    print(f"summary: {scope.plural(total, 'mutant')} at {commit}: "
          + ", ".join(f"{counts[outcome]} {outcome}" for outcome in OUTCOMES))
    return 0 if counts["caught"] == total else 1


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
