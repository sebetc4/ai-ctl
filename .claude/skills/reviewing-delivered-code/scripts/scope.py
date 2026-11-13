#!/usr/bin/env python3
"""Give the scope of a review: its commit range, and its code by hunk with the items holding it.

Usage:
  scope.py roadmap <roadmap-folder> [--to <commit>] [--repo <folder>]
  scope.py range <base> [<end>] [--repo <folder>]

roadmap  reads the **Start Commit:** line of each phase-N-*-report.md in the folder. The
         base is the start commit of the lowest phase; when the history lacks it, the
         scope is the whole history up to the end. The end is the last commit touching
         one of those reports, or --to: a later roadmap's commit may touch the folder, a
         closed report never. A roadmap under the private root, which the project ignores
         and its own repository commits, ends on the project's last commit dated at most
         the last commit of that repository touching one of those reports.
range    takes the commits after <base> up to <end>, HEAD by default.

The scope's files are those the range adds or changes, read at the end commit, each on one
route: captures, tests/fixtures/; code, the Rust files and the files the production code
embeds with include_str! or include_bytes!; dependencies, Cargo.toml and Cargo.lock;
documentation, docs/ but its roadmaps and reviews, and README.md; other, the rest.

The code is given by hunk, a run of changed lines of `git diff -U0`, each as
`path:first-last (symbols) +added -removed`: the items holding its lines, or `top level` for
a line outside any item, as is every line of a file that is not Rust. An item is a function,
an impl, a trait, a type, a constant or a macro, from its attributes and the comments above
it to its end; a line inside a function is held by the outermost function. A blank line, or one of closing delimiters alone, holds
nothing. A hunk that only removes lines is given at the line it follows. Each item holding
a hunk is then given whole, as `path:first-last (symbol)`, and the reading counts the lines
handed to the reviewers: those items whole, and the top-level lines changed. A line counts
as tests after its file's first `#[cfg(test)]` line, or in a file under a `tests` folder.

Prints one `key: value` line per fact. Uncommitted changes are named and left outside the
scope. The report writer imports `origin`: whether a finding's lines changed after the base,
by git blame.

Exit codes: 0 the scope is printed; 1 the repository, a commit or the folder is wrong,
with the reason on stderr; 2 wrong arguments.
"""

import argparse
import posixpath
import re
import subprocess
import sys
from collections import namedtuple
from pathlib import Path

REPORT_RE = re.compile(r"^phase-(\d+)-.*-report\.md$")
START_RE = re.compile(r"^\*\*Start Commit:\*\*\s*([0-9a-fA-F]{4,64})\s*$", re.M)
TEST_MODULE_RE = re.compile(r"^\s*#\[cfg\(test\)\]")
BLAME_RE = re.compile(r"^[0-9a-f]{40} \d+ \d+")
HUNK_RE = re.compile(r"^@@ -(\d+)(?:,(\d+))? \+(\d+)(?:,(\d+))? @@", re.M)
ROUTES = ("code", "dependencies", "documentation", "captures", "other")
TOP_LEVEL = "top level"

Item = namedtuple("Item", "first head last symbol kind")
OPEN, CLOSE = {"(", "[", "{"}, {")", "]", "}"}
NOT_A_NAME = {"dyn", "mut", "const", "unsafe", "impl", "where", "for", "crate", "self", "super"}


class ScopeError(Exception):
    pass


def lex(text):
    """The tokens of Rust source as (text, line), the lines holding code, and those holding comments.

    Strings, characters and comments are read whole, so that no brace inside them counts.
    """
    tokens, code, comments = [], set(), set()
    i, line, n = 0, 1, len(text)

    def take(end):
        nonlocal i, line
        tokens.append((text[i:end], line))
        last = line + text.count("\n", i, end)
        code.update(range(line, last + 1))
        i, line = end, last

    def quoted(start):
        """The end of a string whose text starts at `start`."""
        j = start
        while j < n and text[j] != '"':
            j += 2 if text[j] == "\\" else 1
        return min(j + 1, n)

    while i < n:
        c = text[i]
        if c == "\n":
            i, line = i + 1, line + 1
        elif c.isspace():
            i += 1
        elif text.startswith("//", i):
            comments.add(line)
            end = text.find("\n", i)
            i = n if end < 0 else end
        elif text.startswith("/*", i):
            depth, j = 1, i + 2
            while j < n and depth:
                if text.startswith("/*", j):
                    depth, j = depth + 1, j + 2
                elif text.startswith("*/", j):
                    depth, j = depth - 1, j + 2
                else:
                    j += 1
            last = line + text.count("\n", i, j)
            comments.update(range(line, last + 1))
            i, line = j, last
        elif c == '"':
            take(quoted(i + 1))
        elif c == "'":
            if text.startswith("\\", i + 1):
                end = text.find("'", i + 3)
                take(n if end < 0 else end + 1)
            elif text.startswith("'", i + 2):
                take(i + 3)
            else:
                j = i + 1
                while j < n and (text[j].isalnum() or text[j] == "_"):
                    j += 1
                take(j)
        elif c.isalpha() or c == "_":
            j = i + 1
            while j < n and (text[j].isalnum() or text[j] == "_"):
                j += 1
            word, k = text[i:j], j
            while k < n and text[k] == "#":
                k += 1
            if word in ("r", "br", "cr") and k < n and text[k] == '"':
                close = text.find('"' + "#" * (k - j), k + 1)
                take(n if close < 0 else close + 1 + k - j)
            elif word == "r" and k == j + 1 and k < n and (text[k].isalpha() or text[k] == "_"):
                i = k
            elif word in ("b", "c") and j < n and text[j] == '"':
                take(quoted(j + 1))
            elif word == "b" and j < n and text[j] == "'":
                i = j
                end = text.find("'", j + 3) if text.startswith("\\", j + 1) else j + 2
                take(n if end < 0 else end + 1)
            else:
                take(j)
        elif c.isdigit():
            j = i + 1
            while j < n and (text[j].isalnum() or text[j] == "_" or
                             (text[j] == "." and j + 1 < n and text[j + 1].isdigit())):
                j += 1
            take(j)
        elif text.startswith(("->", "=>", "::"), i):
            take(i + 2)
        else:
            take(i + 1)
    return tokens, code, comments


class Items:
    """The items of one Rust file, found by their keywords and matched braces."""

    def __init__(self, text):
        self.tokens, self.code, self.comments = lex(text)
        self.found = []

    def text(self, i):
        return self.tokens[i][0] if i < len(self.tokens) else ""

    def line(self, i):
        return self.tokens[min(i, len(self.tokens) - 1)][1]

    def after_group(self, i):
        """The index after the group that opens at `i`, or the end when it never closes."""
        depth = 0
        while i < len(self.tokens):
            token = self.text(i)
            if token in OPEN:
                depth += 1
            elif token in CLOSE:
                depth -= 1
                if depth == 0:
                    return i + 1
            i += 1
        return i

    def until(self, i, stops):
        """The index of the first token of `stops` from `i`, groups skipped whole."""
        while i < len(self.tokens):
            token = self.text(i)
            if token in stops or token in CLOSE:
                return i
            i = self.after_group(i) if token in OPEN else i + 1
        return i

    def record(self, start, head, end, symbol, kind):
        first = self.line(start)
        while first - 1 in self.comments and first - 1 not in self.code:
            first -= 1
        self.found.append(Item(first, self.line(head), self.line(end), symbol, kind))

    def body_or_semicolon(self, i):
        """The index of the token that ends an item whose header starts at `i`."""
        j = self.until(i, {"{", ";"})
        return self.after_group(j) - 1 if self.text(j) == "{" else j

    def block(self, i, prefix):
        """Records the items from `i` to the end of their block; the index after it."""
        while i < len(self.tokens):
            token = self.text(i)
            if token == "}":
                return i + 1
            start = i
            while token == "#":
                if self.text(i + 1) == "!" and self.text(i + 2) == "[":
                    i = start = self.after_group(i + 2)
                elif self.text(i + 1) == "[":
                    i = self.after_group(i + 1)
                else:
                    break
                token = self.text(i)
            if token == "pub":
                i += 1
                if self.text(i) == "(":
                    i = self.after_group(i)
            while True:
                token, following = self.text(i), self.text(i + 1)
                if token in ("default", "unsafe", "async", "auto") and following not in ("!", "=", ":"):
                    i += 1
                elif token == "const" and following in ("fn", "unsafe", "async", "extern"):
                    i += 1
                elif token == "extern" and following.startswith('"') and self.text(i + 2) != "{":
                    i += 2
                elif token == "extern" and following == "fn":
                    i += 1
                else:
                    break
            i = self.item(start, i, prefix)
        return i

    def item(self, start, i, prefix):
        """Records the item whose keyword is at `i`; the index after it."""
        keyword, name = self.text(i), self.text(i + 1)
        if keyword == "fn":
            end = self.body_or_semicolon(i + 2)
            self.record(start, i, end, prefix + name, "fn")
        elif keyword in ("struct", "enum", "union") and (name[:1].isalpha() or name[:1] == "_"):
            end = self.body_or_semicolon(i + 2)
            self.record(start, i, end, prefix + name, keyword)
        elif keyword in ("const", "static", "type"):
            if name == "mut":
                name = self.text(i + 2)
            end = self.until(i + 1, {";"})
            self.record(start, i, end, prefix + name, keyword)
        elif keyword == "trait":
            j = self.until(i + 2, {"{", ";"})
            end = self.block(j + 1, f"{prefix}{name}::") - 1 if self.text(j) == "{" else j
            self.record(start, i, end, f"trait {prefix}{name}", "trait")
        elif keyword == "impl":
            j = self.until(i + 1, {"{", ";"})
            trait, own = impl_names([token for token, _ in self.tokens[i + 1:j]])
            end = self.block(j + 1, f"{prefix}{own}::") - 1 if self.text(j) == "{" else j
            symbol = f"impl {trait} for {prefix}{own}" if trait else f"impl {prefix}{own}"
            self.record(start, i, end, symbol, "impl")
        elif keyword == "mod":
            if self.text(i + 2) == "{":
                return self.block(i + 3, f"{prefix}{name}::")
            return self.until(i + 1, {";"}) + 1
        elif keyword == "extern" and (name == "{" or self.text(i + 2) == "{"):
            return self.block(i + (2 if name == "{" else 3), prefix)
        elif keyword == "macro_rules" and name == "!":
            end = self.after_group(i + 3) - 1
            self.record(start, i, end, f"{prefix}{self.text(i + 2)}!", "macro")
        elif name == "!" and (keyword[:1].isalpha() or keyword[:1] == "_"):
            j = self.after_group(i + 2) if self.text(i + 2) in OPEN else i + 2
            return j + 1 if self.text(j) == ";" else j
        elif keyword in ("use", "extern"):
            return self.until(i + 1, {";"}) + 1
        elif keyword in OPEN:
            return self.after_group(i)
        elif keyword == "}":
            return i
        else:
            return i + 1
        return end + 1


def impl_names(header):
    """(trait or None, type) of an impl from the tokens between `impl` and its body."""
    k = 0
    if header[:1] == ["<"]:
        depth = 0
        for k, token in enumerate(header):
            depth += (token == "<") - (token == ">")
            if depth == 0:
                break
        k += 1
    rest = header[k:]
    if "where" in rest:
        rest = rest[:rest.index("where")]
    depth = 0
    for k, token in enumerate(rest):
        depth += (token == "<") - (token == ">")
        if token == "for" and depth == 0:
            return last_name(rest[:k]), last_name(rest[k + 1:])
    return None, last_name(rest)


def last_name(tokens):
    """The last name outside angle brackets: `Store` of `crate::store::Store<'a>`."""
    depth, name = 0, "?"
    for token in tokens:
        depth += (token == "<") - (token == ">")
        if depth == 0 and (token[:1].isalpha() or token[:1] == "_") and token not in NOT_A_NAME:
            name = token
    return name


def items(text):
    """The items of Rust source, in the order of their first lines, each before those it holds."""
    found = Items(text)
    i = 0
    while i < len(found.tokens):
        i = found.block(i, "")
    return sorted(found.found, key=lambda item: (item.first, -item.last))


class Git:
    def __init__(self, folder):
        self.folder = folder

    def run(self, *args, check=True):
        done = subprocess.run(["git", "-C", str(self.folder), *args], capture_output=True)
        if check and done.returncode != 0:
            raise ScopeError(f"git {' '.join(args)}: {done.stderr.decode(errors='replace').strip()}")
        return done

    def text(self, *args):
        return self.run(*args).stdout.decode(errors="replace").strip()

    def commit(self, name):
        """The short hash of a commit, or None when the history lacks it."""
        done = self.run("rev-parse", "--verify", "--quiet", "--short", f"{name}^{{commit}}", check=False)
        return done.stdout.decode().strip() if done.returncode == 0 else None

    def is_ancestor(self, base, end):
        return self.run("merge-base", "--is-ancestor", base, end, check=False).returncode == 0

    def roots(self, end):
        return [self.text("rev-parse", "--short", full) for full in self.text("rev-list", "--max-parents=0", end).split()]


def repository(folder):
    done = subprocess.run(["git", "-C", str(folder), "rev-parse", "--show-toplevel"], capture_output=True, text=True)
    if done.returncode != 0:
        raise ScopeError(f"{folder} is not a git repository")
    return Git(Path(done.stdout.strip()))


def resolve(git, name):
    commit = git.commit(name)
    if commit is None:
        raise ScopeError(f"{name} is not a commit of the repository")
    return commit


def start_commits(folder):
    """(phase, start commit or None, file name) for each phase report, by phase number."""
    found = []
    for path in folder.iterdir():
        match = REPORT_RE.match(path.name)
        if match and path.is_file():
            start = START_RE.search(path.read_text(encoding="utf-8", errors="replace"))
            found.append((int(match.group(1)), start.group(1) if start else None, path.name))
    return sorted(found)


def last_report_commit(git, folder_arg, folder, names):
    """The end of a roadmap's scope and its note, from the log of the repository holding the folder: the
    project's own, or the private root's, whose last report commit gives the project's by date."""
    owner = repository(folder)
    reports = [posixpath.join(folder.resolve().relative_to(owner.folder.resolve()).as_posix(), name) for name in names]
    if owner.folder.resolve() == git.folder.resolve():
        last = git.text("log", "-1", "--format=%H", "--", *reports)
        if not last:
            raise ScopeError(f"no commit touches a phase report of {folder_arg}")
        return git.text("rev-parse", "--short", last), f"the last commit touching a phase report of {folder_arg}"
    dated = owner.text("log", "-1", "--format=%cI", "--", *reports)
    if not dated:
        raise ScopeError(f"no commit of the private root touches a phase report of {folder_arg}")
    last = git.text("log", "-1", "--format=%H", f"--until={dated}")
    if not last:
        raise ScopeError(f"no project commit is dated at most {dated}, the last commit of the private root "
                         f"touching a phase report of {folder_arg}")
    return git.text("rev-parse", "--short", last), ("the project's last commit dated at most the last commit of its "
                                                    f"private root touching a phase report of {folder_arg}")


def roadmap_scope(git, folder_arg, to):
    folder = git.folder / folder_arg
    if not folder.is_dir():
        raise ScopeError(f"{folder_arg} is not a folder of the repository")
    phases = start_commits(folder)
    if not any(start for _, start, _ in phases):
        raise ScopeError(f"no phase report in {folder_arg} gives a Start Commit")
    if to:
        end, end_note = resolve(git, to), "given by --to"
    else:
        end, end_note = last_report_commit(git, folder_arg, folder, [name for _, _, name in phases])
    roots = git.roots(end)
    facts = [("scope", f"roadmap {folder_arg}")]
    for phase, start, name in phases:
        if start is None:
            facts.append(("start commit", f"phase {phase} none in {name}"))
            continue
        commit = git.commit(start)
        if commit is None:
            note = " absent from the history"
        elif commit in roots:
            note = " the root commit"
        elif not git.is_ancestor(commit, end):
            note = f" not an ancestor of {end}"
        else:
            note = ""
        facts.append(("start commit", f"phase {phase} {commit or start}{note}"))
    phase, start, _ = next(entry for entry in phases if entry[1])
    base = git.commit(start)
    if base is None:
        facts.append(("base", f"none: the start commit of phase {phase} is absent from the history, so the scope "
                              f"is the whole history up to the end, from the root commit {', '.join(roots)}"))
    else:
        if not git.is_ancestor(base, end):
            raise ScopeError(f"{base} is not an ancestor of {end}")
        facts.append(("base", f"{base}, the start commit of phase {phase}"))
    facts.append(("end", f"{end}, {end_note}"))
    return facts, base, end


def range_scope(git, base_arg, end_arg):
    base, end = resolve(git, base_arg), resolve(git, end_arg)
    if not git.is_ancestor(base, end):
        raise ScopeError(f"{base} is not an ancestor of {end}")
    return [("scope", f"range {base}..{end}"), ("base", base), ("end", end)], base, end


def embedded(git, end):
    """The files the production code embeds with include_str! or include_bytes! at `end`."""
    done = git.run("grep", "-l", "-z", "-e", "include_str!", "-e", "include_bytes!", end, "--", "*.rs", check=False)
    if done.returncode not in (0, 1):
        raise ScopeError(f"git grep: {done.stderr.decode(errors='replace').strip()}")
    found = set()
    for name in done.stdout.decode(errors="replace").split("\0"):
        path = name.removeprefix(f"{end}:")
        if not path or "tests" in Path(path).parts[:-1]:
            continue
        source = git.run("show", f"{end}:{path}").stdout.decode(errors="replace")
        split = next((n for n, line in enumerate(source.splitlines(), 1) if TEST_MODULE_RE.match(line)), None)
        lexed = lex(source)[0]
        tokens = [token for token, _ in lexed]
        for i, (token, line) in enumerate(lexed):
            argument = tokens[i + 3] if i + 3 < len(tokens) else ""
            if (token in ("include_str", "include_bytes") and tokens[i + 1:i + 3] == ["!", "("]
                    and argument.startswith('"') and (split is None or line < split)):
                found.add(posixpath.normpath(posixpath.join(posixpath.dirname(path), argument[1:-1])))
    return found


def route(path, embeds):
    parts = Path(path).parts
    if parts[:2] == ("tests", "fixtures"):
        return "captures"
    if parts[-1] in ("Cargo.toml", "Cargo.lock"):
        return "dependencies"
    if path.endswith(".rs") or path in embeds:
        return "code"
    if parts[0] == "docs":
        return "other" if parts[1:2] in (("roadmap",), ("reviews",)) else "documentation"
    return "documentation" if path == "README.md" else "other"


def plural(count, word):
    return f"{count} {word}" if count == 1 else f"{count} {word}s"


def trivial(line):
    """Whether a changed line holds nothing: blank, or closing delimiters alone."""
    return set(line.strip()) <= set("})];,")


def holder(found, first, last=None):
    """The innermost item holding the lines `first` to `last`, or None."""
    held = [item for item in found if item.first <= first and (last or first) <= item.last]
    return max(held, key=lambda item: item.first) if held else None


def hunk_facts(git, since, end, path):
    """The `hunk:` facts of one code file, its items given whole, and its reading."""
    source = git.run("show", f"{end}:{path}").stdout.decode(errors="replace")
    lines = source.splitlines()
    rust = path.endswith(".rs")
    found = items(source) if rust else []
    split = next((n for n, line in enumerate(lines, 1) if rust and TEST_MODULE_RE.match(line)), len(lines) + 1)
    in_tests = "tests" in Path(path).parts[:-1]
    diff = git.text("diff", "-U0", "--no-color", "--no-ext-diff", "--no-renames", since, end, "--", path)
    hunks, holders, top = [], [], set()
    for match in HUNK_RE.finditer(diff):
        removed = int(match.group(2) or 1)
        start, added = int(match.group(3)), int(match.group(4) or 1)
        symbols = []
        if added:
            span = f"{start}-{start + added - 1}" if added > 1 else f"{start}"
            for number in range(start, start + added):
                if number > len(lines) or trivial(lines[number - 1]):
                    continue
                item = holder(found, number)
                if item is None:
                    top.add(number)
                symbols.append(item.symbol if item else TOP_LEVEL)
                if item and item not in holders:
                    holders.append(item)
        else:
            span = f"{start}"
            item = holder(found, start, start + 1)
            symbols.append(item.symbol if item else TOP_LEVEL)
            if item and item not in holders:
                holders.append(item)
        named = ", ".join(dict.fromkeys(symbols)) or TOP_LEVEL
        hunks.append((f"{path}:{span} ({named}) +{added} -{removed}", added, removed))
    reading = set(top)
    for item in holders:
        reading.update(range(item.first, item.last + 1))
    tests = sum(1 for number in reading if in_tests or number >= split)
    givens = [f"{path}:{item.first}-{item.last} ({item.symbol})" for item in sorted(holders)]
    return hunks, givens, (len(reading) - tests, tests, len(holders), len(top))


def blamed(git, base, end, path):
    """The lines of `path` at `end`, and whether each changed after `base`, by git blame.

    A line only reindented keeps its origin; with no base, every line is new.
    """
    done = git.run("show", f"{end}:{path}", check=False)
    if done.returncode != 0:
        raise ScopeError(f"{path} is no file at {end}")
    lines = done.stdout.decode(errors="replace").splitlines()
    if base is None:
        return lines, [True] * len(lines)
    new = []
    for line in git.text("blame", "-w", "--line-porcelain", f"{base}..{end}", "--", path).splitlines():
        if BLAME_RE.match(line):
            new.append(True)
        elif line == "boundary":
            new[-1] = False
    return lines, new


def origin(git, base, end, places):
    """`new`, `older` or `mixed`: whether the lines of `places`, each (path, first, last) at
    `end`, changed after `base`.

    A blank line, or one of closing delimiters alone, counts only in a place holding nothing
    else: git blame gives it to whichever block its diff aligns it with.
    """
    files, seen = {}, set()
    for path, first, last in places:
        if path not in files:
            files[path] = blamed(git, base, end, path)
        lines, new = files[path]
        if not 1 <= first <= last <= len(lines):
            span = f"{first}-{last}" if last != first else f"{first}"
            raise ScopeError(f"{path} has {len(lines)} lines, not {span}")
        numbers = [n for n in range(first, last + 1) if not trivial(lines[n - 1])] or range(first, last + 1)
        seen.update(new[n - 1] for n in numbers)
    return "mixed" if len(seen) == 2 else "new" if True in seen else "older"


def numstat(git, since, end):
    """{path: (added, removed) or None for a binary file} of the scope's changes."""
    counts = {}
    for entry in git.run("diff", "--numstat", "-z", "--no-renames", since, end).stdout.decode(errors="replace").split("\0"):
        if entry:
            plus, minus, path = entry.split("\t", 2)
            counts[path] = None if plus == "-" else (int(plus), int(minus))
    return counts


def routes(git, since, end):
    """The files the scope changes and deletes, the counts of its changes, and its changed files by route."""
    changed, deleted = [], []
    fields = git.run("diff", "--name-status", "-z", "--no-renames", since, end).stdout.decode(errors="replace")
    fields = fields.split("\0")
    for status, path in zip(fields[0::2], fields[1::2]):
        (deleted if status == "D" else changed).append(path)
    counts = numstat(git, since, end)
    routed = {name: [] for name in ROUTES}
    embeds = embedded(git, end)
    for path in changed:
        routed[route(path, embeds)].append(path)
    return changed, deleted, counts, routed


def code_hunks(git, since, end, routed, counts):
    """The hunks of the scope's code, the items holding them, its reading, and the code files with a hunk."""
    hunks, givens, reading = [], [], [0, 0, 0, 0]
    files = 0
    for path in routed["code"]:
        if counts.get(path) is None:
            continue
        file_hunks, file_givens, file_reading = hunk_facts(git, since, end, path)
        files += bool(file_hunks)
        hunks += file_hunks
        givens += file_givens
        reading = [total + part for total, part in zip(reading, file_reading)]
    return hunks, givens, reading, files


def size_facts(git, base, end):
    since = base or git.text("hash-object", "-t", "tree", "/dev/null")
    facts = [("diff", f"git diff {since} {end}")]
    changed, deleted, counts, routed = routes(git, since, end)
    facts.append(("files", f"{len(changed)}: " + ", ".join(f"{name} {len(routed[name])}" for name in ROUTES)))
    added = sum(count[0] for count in counts.values() if count)
    removed = sum(count[1] for count in counts.values() if count)
    facts.append(("changed", f"+{added} -{removed}"))
    hunks, givens, reading, files = code_hunks(git, since, end, routed, counts)
    facts.append(("hunks", f"{len(hunks)} in {plural(files, 'code file')}, "
                           f"+{sum(h[1] for h in hunks)} -{sum(h[2] for h in hunks)}"))
    facts.extend(("hunk", text) for text, _, _ in hunks)
    facts.extend(("item", text) for text in givens)
    production, tests, held, top = reading
    facts.append(("reading", f"{production + tests} lines: production {production}, tests {tests}; "
                             f"{plural(held, 'item')}, {plural(top, 'top-level line')}"))
    for name in ROUTES[1:]:
        for path in routed[name]:
            count = counts.get(path)
            facts.append((name, f"{path} +{count[0]} -{count[1]}" if count else f"{path} binary"))
    facts.extend(("deleted", path) for path in deleted)
    return facts


def uncommitted(git):
    tracked = untracked = 0
    for line in git.text("status", "--porcelain").splitlines():
        if line.startswith("??"):
            untracked += 1
        else:
            tracked += 1
    if not tracked and not untracked:
        return [("uncommitted", "none")]
    return [("uncommitted", f"{plural(tracked, 'tracked file')} changed, {plural(untracked, 'untracked file')}, "
                            "outside the scope")]


def main(argv):
    parser = argparse.ArgumentParser(description="Give the scope of a review: its commit range and its code by hunk.")
    parser.add_argument("--repo", default=".", help="the repository, the current folder by default")
    modes = parser.add_subparsers(dest="mode", required=True)
    roadmap = modes.add_parser("roadmap", help="the scope of a roadmap, from its phase reports' Start Commits")
    roadmap.add_argument("folder", help="the roadmap's folder, relative to the repository's root")
    roadmap.add_argument("--to", help="the end commit, instead of the last commit touching a phase report")
    commits = modes.add_parser("range", help="the commits after <base> up to <end>")
    commits.add_argument("base")
    commits.add_argument("end", nargs="?", default="HEAD")
    args = parser.parse_args(argv)
    try:
        git = repository(Path(args.repo))
        if args.mode == "roadmap":
            facts, base, end = roadmap_scope(git, args.folder, args.to)
        else:
            facts, base, end = range_scope(git, args.base, args.end)
        facts += size_facts(git, base, end) + uncommitted(git)
    except ScopeError as error:
        print(f"scope.py: {error}", file=sys.stderr)
        return 1
    for key, value in facts:
        print(f"{key}: {value}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
