#!/usr/bin/env python3
"""The review agents' guards: the hook of the project's settings on Bash, SubagentStart and
SubagentStop.

Usage, as the command of the hooks, the hook's input on standard input:
  guard.py

It acts only on an agent whose `agent_type` starts with `review-`:

PreToolUse     on Bash, refuses the command, by Claude Code's `deny` decision and its reason,
               when it runs ai-ctl other than `doctor`, `status`, `build list` and the help
               and version forms, through `ai-ctl` at any path or `cargo run` of the ai-ctl
               crate; a target of ai-ctl's Makefile other than `help`, `doctor` and `status`;
               or a git subcommand that does not only read, on the repository, or on a target
               that does not resolve to a path. It follows the shell: `;`, `&&`, `||`, pipes,
               subshells, substitutions, heredocs, `cd`, `bash -c`, `eval`, `find -exec`, and
               the wrappers such as `env`, `timeout` or `xargs`. A command it cannot read is
               refused when it names ai-ctl, make, git or cargo.
SubagentStart  takes the agent's baseline: the repository's `git status`, each file it lists
               hashed, and the session's processes left behind.
SubagentStop   blocks the agent's first stop, by the decision `block` and its reason, when the
               repository's status changed, a process of the session was left behind, or a
               background task of the agent still runs, for the agent to clean what is its
               own and give its whole answer again; it lets the next stop through.

The repository is CLAUDE_PROJECT_DIR. A process of the session carries `claude`'s pid,
CLAUDE_PID, in its environment; one left behind is no longer `claude`'s descendant, as a
command run with `nohup … &` once its shell has exited. The baselines are kept in
$XDG_RUNTIME_DIR/ai-ctl-review-guard/, or the temporary folder's ai-ctl-review-guard-<uid>/,
a folder only its user may reach.

Exit codes: 0, the decision on standard output, nothing when the call or the stop passes;
2 an input that is no hook's, the reason on stderr, which blocks the call.
"""

import hashlib
import json
import os
import re
import stat
import subprocess
import sys
import tempfile
import tomllib
from pathlib import Path

AGENTS = "review-"
STATE = "ai-ctl-review-guard"
IDENTIFIER = re.compile(r"[A-Za-z0-9_-]{1,128}")
GUARDED = re.compile(r"(?<![\w.-])(ai-ctl|git|g?make|cargo)(?![\w-])")
PLACEHOLDER = re.compile("\x00(\\d+)\x00")
ASSIGNMENT = re.compile(r"[A-Za-z_][A-Za-z0-9_]*=")
DEPTH = 16

OPERATORS = sorted(["&&", "||", ";;&", ";;", ";&", "|&", "&>>", "&>", ">>", "<<<", "<<-", "<<", ">&", "<&", ">|",
                    "<>", "&", "|", ";", "(", ")", "\n", ">", "<"], key=len, reverse=True)
REDIRECTS = {"&>>", "&>", ">>", "<<<", "<<-", "<<", ">&", "<&", ">|", "<>", ">", "<"}
PIPES = {"|", "|&"}
RESERVED = {"{", "}", "!", "if", "then", "else", "elif", "fi", "do", "done", "while", "until"}
STAY = object()

AI_CTL_HELP = {"-h", "--help", "-V", "--version"}
MAKE_TARGETS = {"help", "doctor", "status"}
GIT_READS = {"status", "diff", "diff-tree", "diff-index", "diff-files", "range-diff", "log", "shortlog", "show",
             "show-ref", "show-branch", "whatchanged", "blame", "annotate", "grep", "ls-files", "ls-tree",
             "ls-remote", "rev-parse", "rev-list", "cat-file", "archive", "describe", "name-rev", "merge-base",
             "for-each-ref", "cherry", "count-objects", "check-ignore", "check-attr", "check-ref-format", "var",
             "version", "help", "verify-commit", "verify-tag"}
GIT_VALUES = {"-C", "-c", "--git-dir", "--work-tree", "--namespace", "--config-env", "--super-prefix"}
CARGO_VALUES = {"--color", "--config", "-Z", "-C", "--explain"}
RUN_VALUES = {"--bin", "--example", "-p", "--package", "--manifest-path", "--target-dir", "--features", "-F", "-j",
              "--jobs", "--profile", "--target", "--color", "--config", "-Z", "--message-format", "--lockfile-path",
              "--artifact-dir"}
MAKE_VALUES = {"-I", "--include-dir", "-o", "--old-file", "--assume-old", "-W", "--what-if", "--new-file",
               "--assume-new", "-E", "--eval"}
SHELLS = {"bash", "sh", "dash", "zsh", "ksh"}
# Each wrapper: its options that take a value, and the operands before the command it runs.
WRAPPERS = {
    "env": ({"-u", "--unset", "-C", "--chdir", "-S", "--split-string"}, 0),
    "sudo": ({"-u", "--user", "-g", "--group", "-h", "--host", "-p", "--prompt", "-C", "--close-from", "-D",
              "--chdir", "-r", "--role", "-t", "--type", "-U", "--other-user", "-T", "--command-timeout"}, 0),
    "doas": ({"-u", "-C"}, 0),
    "nohup": (set(), 0),
    "setsid": (set(), 0),
    "exec": ({"-a"}, 0),
    "command": (set(), 0),
    "builtin": (set(), 0),
    "time": ({"-f", "--format", "-o", "--output"}, 0),
    "nice": ({"-n", "--adjustment"}, 0),
    "ionice": ({"-c", "--class", "-n", "--classdata"}, 0),
    "stdbuf": ({"-i", "--input", "-o", "--output", "-e", "--error"}, 0),
    "timeout": ({"-s", "--signal", "-k", "--kill-after"}, 1),
    "xargs": ({"-a", "--arg-file", "-d", "--delimiter", "-E", "-I", "-L", "-n", "--max-args", "-P",
               "--max-procs", "-s", "--max-chars", "--process-slot-var"}, 0),
}
CHDIR = {"env": {"-C", "--chdir"}, "sudo": {"-D", "--chdir"}}

REASONS = {
    "ai-ctl": "of ai-ctl, a review agent runs only `doctor`, `status`, `build list` and the help and version "
              "forms, through `ai-ctl` or `cargo run`",
    "make": "of ai-ctl's Makefile, a review agent runs only `make help`, `make doctor` and `make status`",
    "git-write": "on the repository, a review agent runs only the git subcommands that read, such as status, "
                 "diff, log, show or blame; another repository is named by its absolute path, "
                 "`git -C <absolute path>`",
    "git-target": "its target does not resolve to a path: a variable, a substitution or a folder that does not "
                  "exist yet; name it by its absolute path, `git -C <absolute path>`",
    "unreadable": "the guard cannot read this command, which names ai-ctl, make, git or cargo; write it plainly",
}
AGAIN = ("Then give your whole answer again, every block of it, since the session receives only your last "
         "message, and say in it what you leave and why.")


class GuardError(Exception):
    pass


# The shell's words.

def closing(text, start):
    """The index of the `)` closing the substitution whose command starts at `start`."""
    depth, i = 1, start
    while i < len(text):
        c = text[i]
        if c == "\\":
            i += 2
            continue
        if c == "'":
            end = text.find("'", i + 1)
            if end < 0:
                break
            i = end + 1
            continue
        if c == '"':
            i += 1
            while i < len(text) and text[i] != '"':
                i += 2 if text[i] == "\\" else 1
            i += 1
            continue
        if c == "(":
            depth += 1
        elif c == ")":
            depth -= 1
            if depth == 0:
                return i
        i += 1
    raise ValueError("a substitution is not closed")


def backquoted(text, start):
    """The index of the backquote closing the one before `start`."""
    i = start
    while i < len(text):
        if text[i] == "\\":
            i += 2
            continue
        if text[i] == "`":
            return i
        i += 1
    raise ValueError("a backquote is not closed")


def substitution(text, i, subs):
    """Reads the substitution at `i`, `$(…)`, `<(…)`, `>(…)` or `` `…` ``: its placeholder and the
    index after it."""
    if text[i] == "`":
        end = backquoted(text, i + 1)
        subs.append(text[i + 1:end].replace("\\`", "`"))
    else:
        end = closing(text, i + 2)
        subs.append(text[i + 2:end])
    return f"\x00{len(subs) - 1}\x00", end + 1


def body_substitutions(body, subs):
    """The placeholders of the substitutions a heredoc's body runs, its delimiter unquoted."""
    found, i = [], 0
    while i < len(body):
        if body[i] == "\\":
            i += 2
        elif body.startswith("$(", i) or body[i] == "`":
            placeholder, i = substitution(body, i, subs)
            found.append(placeholder)
        else:
            i += 1
    return found


def lex(text):
    """Bash's tokens of `text`, ("word", its text unquoted) or ("op", an operator), and the
    commands of its substitutions, each standing in its word as a placeholder. A heredoc's body
    is left out, but for the substitutions it runs, set as words after its delimiter."""
    tokens, subs, heredocs = [], [], []
    word, quoted, delimiter = None, False, None
    i = 0

    def end_word():
        nonlocal word, quoted, delimiter
        if word is None:
            return
        tokens.append(("word", "".join(word)))
        if delimiter is not None:
            heredocs.append(("".join(word), delimiter, quoted, len(tokens)))
            delimiter = None
        word, quoted = None, False

    while i < len(text):
        c = text[i]
        if c == "\\":
            if text.startswith("\\\n", i):
                i += 2
                continue
            word = (word or []) + [text[i + 1:i + 2]]
            quoted = True
            i += 2
        elif c == "'":
            end = text.find("'", i + 1)
            if end < 0:
                raise ValueError("a quotation is not closed")
            word = (word or []) + [text[i + 1:end]]
            quoted = True
            i = end + 1
        elif c == '"':
            word, quoted, i = (word or []), True, i + 1
            while True:
                if i >= len(text):
                    raise ValueError("a quotation is not closed")
                if text[i] == '"':
                    i += 1
                    break
                if text[i] == "\\" and text[i + 1:i + 2] in ("$", "`", '"', "\\", "\n"):
                    word.append(text[i + 1] if text[i + 1] != "\n" else "")
                    i += 2
                elif text.startswith("$(", i) or text[i] == "`":
                    placeholder, i = substitution(text, i, subs)
                    word.append(placeholder)
                else:
                    word.append(text[i])
                    i += 1
        elif text.startswith("$(", i) or c == "`":
            placeholder, i = substitution(text, i, subs)
            word = (word or []) + [placeholder]
        elif c in "<>" and text.startswith("(", i + 1):
            end_word()
            placeholder, i = substitution(text, i, subs)
            tokens.append(("word", placeholder))
        elif c in " \t\r":
            end_word()
            i += 1
        elif c == "#" and word is None:
            end = text.find("\n", i)
            i = len(text) if end < 0 else end
        elif c in ";&|()<>\n":
            if c in "<>" and word is not None and not quoted and "".join(word).isdigit():
                word = None
            end_word()
            op = next(op for op in OPERATORS if text.startswith(op, i))
            tokens.append(("op", op))
            i += len(op)
            if op in ("<<", "<<-"):
                delimiter = op
            elif op == "\n" and heredocs:
                i = bodies(text, i, heredocs, tokens, subs)
                heredocs = []
        else:
            word = (word or []) + [c]
            i += 1
    end_word()
    return tokens, subs


def bodies(text, i, heredocs, tokens, subs):
    """Skips the bodies of the heredocs opened on the line ending at `i`, in their order; the index
    after them."""
    found = []
    for delimiter, op, quoted, index in heredocs:
        start = i
        while i < len(text):
            end = text.find("\n", i)
            end = len(text) if end < 0 else end
            line = text[i:end]
            i = end + 1
            if (line.lstrip("\t") if op == "<<-" else line) == delimiter:
                break
        if not quoted:
            found.append((index, body_substitutions(text[start:i], subs)))
    for index, placeholders in reversed(found):
        tokens[index:index] = [("word", placeholder) for placeholder in placeholders]
    return i


def literal(word):
    return word is not None and not any(c in word for c in "$`*?[{\x00")


def resolve(base, path):
    """`path`'s real absolute path from `base`, None when either does not resolve."""
    if not literal(path):
        return None
    path = os.path.expanduser(path)
    if path.startswith("~"):
        return None
    if not os.path.isabs(path):
        if base is None:
            return None
        path = os.path.join(base, path)
    return os.path.realpath(path)


def crate_name(folder):
    """The name of the package whose Cargo.toml holds `folder`, "" with none, None when unknown."""
    if folder is None:
        return None
    for candidate in (Path(folder), *Path(folder).parents):
        manifest = candidate / "Cargo.toml"
        if manifest.is_file():
            try:
                return tomllib.loads(manifest.read_text(encoding="utf-8")).get("package", {}).get("name", "")
            except (OSError, UnicodeDecodeError, tomllib.TOMLDecodeError):
                return None
    return ""


def shown(words):
    return PLACEHOLDER.sub("$(…)", " ".join(words))


def option_value(args, i, values):
    """(the option's name, its value or None, the index after it) for the option at `i`."""
    name, eq, value = args[i].partition("=")
    if eq and name in values:
        return name, value, i + 1
    if args[i] in values:
        return args[i], (args[i + 1] if i + 1 < len(args) else None), i + 2
    return args[i], None, i + 1


# The commands.

def ai_ctl_allowed(args, appended):
    options = args[:args.index("--")] if "--" in args else args
    if AI_CTL_HELP.intersection(options):
        return True
    if not args:
        return not appended
    return args[0] in ("help", "doctor", "status") or args[:2] == ["build", "list"]


def cargo_run(args, cwd):
    """(whether `cargo` runs ai-ctl, the arguments it gets), or None when it runs nothing."""
    i = 1 if args and args[0].startswith("+") else 0
    base = cwd
    while i < len(args) and args[i].startswith("-"):
        name, value, i = option_value(args, i, CARGO_VALUES)
        if name == "-C":
            base = resolve(base, value)
    if i >= len(args) or args[i] not in ("run", "r"):
        return None
    rest = args[i + 1:]
    options, program = (rest[:rest.index("--")], rest[rest.index("--") + 1:]) if "--" in rest else (rest, [])
    values, j = {}, 0
    while j < len(options):
        name, value, j = option_value(options, j, RUN_VALUES)
        if value is not None:
            values[name] = value
    if "--example" in values:
        return False, program
    for key in ("--bin", "-p", "--package"):
        if key in values:
            return values[key] == "ai-ctl" or not literal(values[key]), program
    if "--manifest-path" in values:
        manifest = resolve(base, values["--manifest-path"])
        base = os.path.dirname(manifest) if manifest else None
    return crate_name(base) in ("ai-ctl", None), program


def make_run(args, cwd):
    """(the folder `make` runs in, its targets)."""
    folder, makefile, targets, i = cwd, None, [], 0
    while i < len(args):
        arg = args[i]
        if arg.startswith("-C") and len(arg) > 2 and not arg.startswith("--"):
            folder, i = resolve(folder, arg[2:]), i + 1
        elif arg.startswith("-"):
            name, value, after = option_value(args, i, {"-C", "--directory", "-f", "--file", "--makefile"})
            if name in ("-C", "--directory"):
                folder, i = resolve(folder, value), after
            elif name in ("-f", "--file", "--makefile"):
                makefile, i = value, after
            elif arg in ("-j", "--jobs", "-l", "--load-average") and args[i + 1:i + 2] and args[i + 1].isdigit():
                i += 2
            else:
                i = option_value(args, i, MAKE_VALUES)[2]
        elif "=" in arg:
            i += 1
        else:
            targets.append(arg)
            i += 1
    if makefile is not None:
        path = resolve(folder, makefile)
        folder = os.path.dirname(path) if path else None
    return folder, targets


def git_run(args, cwd, env):
    """(git's subcommand or None, its arguments, the paths it acts on, None for one unknown)."""
    base, git_dir, work_tree, i = cwd, env.get("GIT_DIR"), env.get("GIT_WORK_TREE"), 0
    while i < len(args) and args[i].startswith("-"):
        name, value, i = option_value(args, i, GIT_VALUES)
        if name == "-C":
            base = resolve(base, value)
        elif name == "--git-dir":
            git_dir = value
        elif name == "--work-tree":
            work_tree = value
    targets = [resolve(base, path) for path in (git_dir, work_tree) if path is not None]
    if git_dir is None or work_tree is None:
        targets.append(base)
    return (args[i] if i < len(args) else None), args[i + 1:], targets


def cd_target(args, cwd):
    args = [arg for arg in args if arg not in ("-L", "-P", "-e", "-@")]
    if args[:1] == ["--"]:
        args = args[1:]
    target = resolve(cwd, args[0] if args else "~")
    return target if target and os.path.isdir(target) else None


class Checker:
    def __init__(self, repository):
        self.repository = os.path.realpath(repository)

    def inside(self, path):
        return path == self.repository or path.startswith(self.repository + os.sep)

    def analyze(self, text, cwd, depth=0):
        """The refusals of a command line run in `cwd`, None when unknown: (kind, the command)."""
        try:
            if depth > DEPTH:
                raise ValueError("its commands nest too deep")
            tokens, subs = lex(text)
        except ValueError as error:
            return [("unreadable", f"{shown(text.split())[:200]} ({error})")] if GUARDED.search(text) else []
        found, stack, words, redirect, before, override = [], [], [], False, None, STAY
        for kind, value in tokens + [("op", None)]:
            if kind == "word":
                here = cwd if override is STAY else override
                for number in PLACEHOLDER.findall(value):
                    found += self.analyze(subs[int(number)], here, depth + 1)
                if redirect:
                    redirect = False
                else:
                    words.append(value)
                continue
            if value in REDIRECTS:
                redirect = True
                continue
            if words:
                here = cwd if override is STAY else override
                override = STAY
                refusals, moved = self.command(words, here, depth)
                found += refusals
                if moved is not STAY and before not in PIPES and value not in PIPES and value != "&":
                    if value == "||":
                        override = here
                    cwd = moved
                words = []
            if value == "(":
                stack.append(cwd)
            elif value == ")" and stack:
                cwd = stack.pop()
            before = value
        return found

    def command(self, words, cwd, depth, appended=False, env=None):
        """The refusals of a simple command, and the folder it moves to, STAY for none."""
        env = dict(env or {})
        words = list(words)
        while words and (words[0] in RESERVED or ASSIGNMENT.match(words[0])):
            name, eq, value = words.pop(0).partition("=")
            if eq:
                env[name] = value
        if not words:
            return [], STAY
        name, args, text = os.path.basename(words[0]), words[1:], shown(words)
        if name in WRAPPERS:
            return self.wrapped(name, args, cwd, depth, appended, env), STAY
        if name in SHELLS:
            script = next((args[i + 1] for i, arg in enumerate(args[:-1])
                           if arg.startswith("-") and not arg.startswith("--") and "c" in arg), None)
            return (self.analyze(script, cwd, depth + 1) if script is not None else []), STAY
        if name == "eval":
            return self.analyze(" ".join(args), cwd, depth + 1), STAY
        if name == "find":
            return self.executed(args, cwd, depth, env), STAY
        if name in ("cd", "pushd"):
            return [], cd_target(args, cwd)
        if name == "popd":
            return [], None
        if name == "ai-ctl":
            return ([] if ai_ctl_allowed(args, appended) else [("ai-ctl", text)]), STAY
        if name == "cargo":
            run = cargo_run(args, cwd)
            refused = run is not None and run[0] and not ai_ctl_allowed(run[1], appended)
            return ([("ai-ctl", text)] if refused else []), STAY
        if name in ("make", "gmake"):
            folder, targets = make_run(args, cwd)
            refused = crate_name(folder) in ("ai-ctl", None) and (appended or not MAKE_TARGETS.issuperset(targets))
            return ([("make", text)] if refused else []), STAY
        if name == "git":
            return self.git(args, cwd, appended, env, text), STAY
        return [], STAY

    def wrapped(self, name, args, cwd, depth, appended, env):
        values, operands = WRAPPERS[name]
        i = 0
        while i < len(args):
            arg = args[i]
            if arg == "--":
                i += 1
                break
            if name == "env" and (arg == "-" or ASSIGNMENT.match(arg)):
                key, _, value = arg.partition("=")
                env[key] = value
                i += 1
                continue
            if not arg.startswith("-") or arg == "-":
                break
            option, value, i = option_value(args, i, values)
            if option in CHDIR.get(name, ()):
                cwd = resolve(cwd, value)
            elif name == "env" and option in ("-S", "--split-string"):
                return self.analyze(" ".join([value or "", *args[i:]]), cwd, depth + 1)
        rest = args[i + operands:]
        if not rest:
            return []
        return self.command(rest, cwd, depth, appended or name == "xargs", env)[0]

    def executed(self, args, cwd, depth, env):
        found, i = [], 0
        while i < len(args):
            if args[i] in ("-exec", "-execdir", "-ok", "-okdir"):
                end = next((j for j in range(i + 1, len(args)) if args[j] in (";", "+")), len(args))
                if end > i + 1:
                    found += self.command(args[i + 1:end], cwd, depth, False, env)[0]
                i = end
            i += 1
        return found

    def git(self, args, cwd, appended, env, text):
        subcommand, rest, targets = git_run(args, cwd, env)
        if subcommand is None and not appended:
            return []
        if subcommand in GIT_READS or rest in (["-h"], ["--help"]):
            return []
        if any(target is not None and self.inside(target) for target in targets):
            return [("git-write", text)]
        if None in targets:
            return [("git-target", text)]
        return []


def check(command, cwd, repository):
    """The refusals of `command`, a review agent's call run in `cwd` on `repository`."""
    return Checker(repository).analyze(command, os.path.realpath(cwd) if cwd else None)


def listening(text):
    """{pid: its sockets, "tcp 127.0.0.1:8080"} from the lines of `ss -Hltnup`."""
    sockets = {}
    for line in text.splitlines():
        fields = line.split()
        if len(fields) < 7 or not fields[6].startswith("users:"):
            continue
        for pid in map(int, re.findall(r"pid=(\d+)", " ".join(fields[6:]))):
            entry = f"{fields[0]} {fields[4]}"
            if entry not in sockets.setdefault(pid, []):
                sockets[pid].append(entry)
    return sockets


# The sandbox.

def status(repository):
    """{`XY path` of `git status`: the file's hash} for the repository."""
    done = subprocess.run(["git", "-C", repository, "status", "--porcelain=v1", "-z", "--untracked-files=all"],
                          capture_output=True, env={**os.environ, "GIT_OPTIONAL_LOCKS": "0"})
    if done.returncode != 0:
        raise GuardError(f"`git status` failed on {repository}: "
                         + done.stderr.decode("utf-8", "replace").strip())
    found, entries = {}, iter(done.stdout.split(b"\0"))
    for entry in entries:
        if not entry:
            continue
        code, path = entry[:2].decode("ascii", "replace"), entry[3:]
        if "R" in code or "C" in code:
            next(entries, None)
        found[f"{code} {path.decode('utf-8', 'backslashreplace')}"] = digest(os.path.join(repository.encode(), path))
    return found


def digest(path):
    try:
        info = os.lstat(path)
        if stat.S_ISLNK(info.st_mode):
            return "link " + os.fsdecode(os.readlink(path))
        if not stat.S_ISREG(info.st_mode):
            return "other"
        with open(path, "rb") as file:
            return hashlib.file_digest(file, "sha256").hexdigest()
    except FileNotFoundError:
        return "missing"


def left_behind(claude):
    """{"pid start": its command line} of the user's processes carrying CLAUDE_PID=`claude` that
    are not its descendants."""
    marker, uid, parents, candidates = f"CLAUDE_PID={claude}".encode(), os.getuid(), {}, []
    for entry in os.scandir("/proc"):
        if not entry.name.isdigit():
            continue
        pid = int(entry.name)
        try:
            with open(f"/proc/{pid}/stat", "rb") as file:
                fields = file.read().rpartition(b")")[2].split()
            parents[pid] = int(fields[1])
            if entry.stat().st_uid != uid:
                continue
            with open(f"/proc/{pid}/environ", "rb") as file:
                if marker in file.read().split(b"\0"):
                    candidates.append((pid, int(fields[19])))
        except (OSError, ValueError, IndexError):
            continue
    found = {}
    for pid, start in candidates:
        ancestor, seen = pid, set()
        while ancestor not in (claude, None, 0, 1) and ancestor not in seen:
            seen.add(ancestor)
            ancestor = parents.get(ancestor)
        if ancestor != claude:
            found[f"{pid} {start}"] = command_line(pid)
    return found


def command_line(pid):
    try:
        with open(f"/proc/{pid}/cmdline", "rb") as file:
            return file.read().rstrip(b"\0").replace(b"\0", b" ").decode("utf-8", "replace")
    except OSError:
        return "?"


def state_path(data):
    runtime = os.environ.get("XDG_RUNTIME_DIR")
    folder = Path(runtime) / STATE if runtime else Path(tempfile.gettempdir()) / f"{STATE}-{os.getuid()}"
    try:
        folder.mkdir(mode=0o700)
    except FileExistsError:
        pass
    except OSError as error:
        raise GuardError(f"{folder} cannot be made: {error.strerror}") from None
    info = os.lstat(folder)
    if not stat.S_ISDIR(info.st_mode) or info.st_uid != os.getuid() or info.st_mode & 0o077:
        raise GuardError(f"{folder} is no folder of this user's alone")
    names = [data.get("session_id"), data.get("agent_id")]
    if not all(isinstance(name, str) and IDENTIFIER.fullmatch(name) for name in names):
        raise GuardError("the hook's input names no session or agent")
    return folder / f"{names[0]}-{names[1]}.json"


def write(path, state):
    temporary = path.with_suffix(".tmp")
    temporary.write_text(json.dumps(state), encoding="utf-8")
    os.replace(temporary, path)


def claude_pid():
    value = os.environ.get("CLAUDE_PID", "")
    if not value.isdigit():
        raise GuardError("CLAUDE_PID, `claude`'s pid, is not set")
    return int(value)


def subagent_start(data, repository):
    path = state_path(data)
    try:
        state = {"status": status(repository), "processes": left_behind(claude_pid()), "blocked": False}
    except (GuardError, OSError) as error:
        state = {"error": str(error), "blocked": False}
    write(path, state)


def subagent_stop(data, repository):
    try:
        path = state_path(data)
        state = json.loads(path.read_text(encoding="utf-8")) if path.exists() else None
    except (GuardError, OSError, ValueError) as error:
        path, state = None, {"error": str(error), "blocked": False}
    if state is None:
        state = {"error": "no baseline was taken at its start", "blocked": False}
    if state.get("blocked") or (data.get("stop_hook_active") and "status" not in state):
        if path is not None:
            path.unlink(missing_ok=True)
        return None
    if "status" not in state:
        return {"decision": "block",
                "reason": f"The review guard cannot tell what this agent leaves: {state['error']}. Say in your "
                          "answer which processes you started and stopped, and which files of the repository you "
                          f"changed. {AGAIN}"}
    left = leftovers(data, repository, state)
    if not left:
        path.unlink(missing_ok=True)
        return None
    try:
        write(path, {**state, "blocked": True})
    except OSError:
        path.unlink(missing_ok=True)
    return {"decision": "block",
            "reason": "The review guard found what may be left since this agent started:\n"
                      + "\n".join(f"- {line}" for line in left)
                      + "\nStop the processes you started and undo your changes to the repository; leave what "
                        f"another agent or the session made. {AGAIN}"}


def leftovers(data, repository, state):
    now, before = status(repository), state["status"]
    found = [f"the repository: `{line}`" + ("" if line in now else ", no longer listed")
             for line in sorted(set(now) | set(before)) if now.get(line) != before.get(line)]
    processes = {key: line for key, line in left_behind(claude_pid()).items() if key not in state["processes"]}
    if processes:
        done = subprocess.run(["ss", "-Hltnup"], capture_output=True, text=True, env={**os.environ, "LC_ALL": "C"})
        sockets = listening(done.stdout) if done.returncode == 0 else {}
        for key, line in sorted(processes.items()):
            pid = int(key.split()[0])
            ports = ", listening on " + ", ".join(sockets[pid]) if pid in sockets else ""
            found.append(f"process {pid}: `{line}`{ports}")
        if done.returncode != 0:
            found.append(f"their sockets unread: `ss` failed, {done.stderr.strip()}")
    for task in data.get("background_tasks") or []:
        if isinstance(task, dict) and task.get("status") == "running":
            found.append(f"background task {task.get('id')}: `{task.get('command')}`")
    return found


def pre_tool_use(data, repository):
    if data.get("tool_name") != "Bash":
        return None
    command = (data.get("tool_input") or {}).get("command")
    if not isinstance(command, str):
        return deny("Refused by the review guard: the call holds no command it can read.")
    refusals = check(command, data.get("cwd"), repository)
    if not refusals:
        return None
    return deny("Refused by the review guard:\n" + "\n".join(f"- `{text}`: {REASONS[kind]}" for kind, text in refusals)
                + "\nA command a finding needs goes in a PROBE block of your answer, for the session to run.")


def deny(reason):
    return {"hookSpecificOutput": {"hookEventName": "PreToolUse", "permissionDecision": "deny",
                                   "permissionDecisionReason": reason}}


def main():
    try:
        data = json.loads(sys.stdin.read())
        if not isinstance(data, dict):
            raise ValueError("it is no object")
    except ValueError as error:
        print(f"guard.py: the hook's input is unreadable: {error}", file=sys.stderr)
        return 2
    agent = data.get("agent_type")
    if not isinstance(agent, str) or not agent.startswith(AGENTS):
        return 0
    event = data.get("hook_event_name")
    repository = os.environ.get("CLAUDE_PROJECT_DIR") or data.get("cwd") or "."
    decision = None
    try:
        if event == "PreToolUse":
            decision = pre_tool_use(data, repository)
        elif event == "SubagentStart":
            subagent_start(data, repository)
        elif event == "SubagentStop":
            decision = subagent_stop(data, repository)
    except Exception as error:  # noqa: BLE001 — a guard that fails lets nothing through
        if event == "PreToolUse":
            decision = deny(f"The review guard failed on this call: {error!r}")
        elif event == "SubagentStop" and not data.get("stop_hook_active"):
            decision = {"decision": "block", "reason": f"The review guard failed at this stop: {error!r}. {AGAIN}"}
        else:
            print(f"guard.py: {error!r}", file=sys.stderr)
    if decision is not None:
        print(json.dumps(decision))
    return 0


if __name__ == "__main__":
    sys.exit(main())
