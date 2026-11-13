#!/usr/bin/env python3
"""Unit tests of scripts/guard.py, run on Claude Code's captured hook inputs, throwaway
repositories and processes made for each test.

Usage: test_guard.py [unittest options]
"""

import json
import os
import signal
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

HERE = Path(__file__).resolve().parent
SCRIPT = HERE.parent / "scripts" / "guard.py"
CAPTURES = HERE.parents[3] / "tests" / "fixtures"
sys.dont_write_bytecode = True
sys.path[:0] = [str(SCRIPT.parent), str(HERE)]

import guard  # noqa: E402
from test_scope import GIT_ENV, Repo  # noqa: E402

AI_CTL = '[package]\nname = "ai-ctl"\nversion = "0.2.0"\nedition = "2024"\n'
PROBE = '[package]\nname = "probe"\nversion = "0.1.0"\nedition = "2024"\n'
# Binds a loopback port, prints it, then waits: a server an agent would leave behind.
LISTEN = ("import socket, sys, time\n"
          "s = socket.socket()\n"
          "s.bind(('127.0.0.1', 0))\n"
          "s.listen()\n"
          "print('port', s.getsockname()[1], flush=True)\n"
          "time.sleep(600)\n")


def capture(name, **changes):
    data = json.loads((CAPTURES / name).read_text(encoding="utf-8"))
    data.update(changes)
    return data


def bash(command, cwd, agent="review-security"):
    data = capture("claude-hook-pre-tool-use-agent.json", cwd=str(cwd), agent_type=agent)
    data["tool_input"] = {**data["tool_input"], "command": command}
    return data


def hook(data, env):
    text = data if isinstance(data, str) else json.dumps(data)
    return subprocess.run([sys.executable, "-I", str(SCRIPT)], input=text, capture_output=True, text=True,
                          env=env, timeout=120)


class Folders(unittest.TestCase):
    """A repository of ai-ctl, a scratch folder beside it, a crate and a copy of ai-ctl in it."""

    def setUp(self):
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        root = Path(temporary.name).resolve()
        self.repo = root / "ai-ctl"
        self.scratch = root / "scratch"
        for folder, manifest in ((self.repo, AI_CTL), (self.scratch / "crate", PROBE),
                                 (self.scratch / "copy", AI_CTL)):
            folder.mkdir(parents=True)
            (folder / "Cargo.toml").write_text(manifest, encoding="utf-8")
        (self.scratch / "c-project").mkdir()
        (self.scratch / "copy" / "src").mkdir()
        (self.scratch / "broken").mkdir()
        (self.scratch / "broken" / "Cargo.toml").write_text("[package\n", encoding="utf-8")
        (self.scratch / "link").symlink_to(self.repo)
        self.sibling = root / "ai-ctl-copy"
        self.sibling.mkdir()

    def kinds(self, command, cwd=None):
        return [kind for kind, _ in guard.check(command, str(cwd or self.repo), str(self.repo))]

    def allowed(self, *commands, cwd=None):
        for command in commands:
            with self.subTest(command=command):
                self.assertEqual(self.kinds(command, cwd), [])

    def refused(self, kind, *commands, cwd=None):
        for command in commands:
            with self.subTest(command=command):
                self.assertIn(kind, self.kinds(command, cwd))


class AiCtlTest(Folders):
    def test_the_reading_commands_and_the_help_pass(self):
        self.allowed("ai-ctl doctor", "ai-ctl doctor --json", "ai-ctl status", "ai-ctl build list",
                     "ai-ctl build list llama.cpp", "ai-ctl --help", "ai-ctl -h", "ai-ctl help up",
                     "ai-ctl --version", "ai-ctl -V", "ai-ctl up --help", "ai-ctl gateway install -h", "ai-ctl",
                     "./target/release/ai-ctl status", "/home/user/.local/bin/ai-ctl doctor",
                     "cargo run -- status", "cargo run --release -q -- doctor --json", "cargo run",
                     "cargo +stable run --locked -- build list", "ai-ctl \\\n  status",
                     "ai-ctl status>/dev/null", "cat <<EOF\n\\$(ai-ctl up)\nEOF")

    def test_every_other_command_is_refused_whatever_runs_it(self):
        self.refused("ai-ctl", "ai-ctl up", "ai-ctl down", "ai-ctl build llama.cpp", "ai-ctl build use llama.cpp b1",
                     "ai-ctl build prune", "ai-ctl gateway install", "ai-ctl gateway config", "ai-ctl init --models-dir /m",
                     "ai-ctl logs", "ai-ctl bench run", "ai-ctl completions bash", "ai-ctl -- --help",
                     "~/.local/bin/ai-ctl down", "target/debug/ai-ctl up", "cargo run -- up",
                     "cargo run --release -- gateway install", "cargo r -- down", "cargo run --bin ai-ctl -- up",
                     "cargo run -p ai-ctl -- up", f"cargo run --manifest-path {self.repo}/Cargo.toml -- up")

    def test_a_command_is_found_where_the_shell_would_run_it(self):
        self.refused("ai-ctl", "ai-ctl doctor; ai-ctl down", "true && ai-ctl up", "false || ai-ctl up",
                     "ai-ctl status | ai-ctl up", "ai-ctl up &", "(ai-ctl up)", "{ ai-ctl up; }", "echo $(ai-ctl up)",
                     'echo "$(ai-ctl up)"', "echo `ai-ctl up`", "x=`ai-ctl up`", "RUST_LOG=debug ai-ctl up",
                     "env RUST_LOG=debug ai-ctl up", "env -u HOME ai-ctl up", "nohup ai-ctl up >/dev/null 2>&1 &",
                     "timeout 5 ai-ctl up", "timeout -s KILL 5 ai-ctl up", "time -p ai-ctl up",
                     "env -S 'ai-ctl up'", "cat <<A <<'B'\nx\nA\nok\nB\nai-ctl up",
                     "cat <<'A' <<B\nx\nA\n$(ai-ctl up)\nB", "sudo ai-ctl down", "sudo -u root ai-ctl down",
                     "nice -n 5 ai-ctl up", "setsid ai-ctl up", "exec ai-ctl up", "command ai-ctl up", "time ai-ctl up",
                     "bash -c 'ai-ctl up'", "bash -lc 'ai-ctl up'", "sh -c \"ai-ctl doctor && ai-ctl up\"", "eval 'ai-ctl up'",
                     "echo up | xargs ai-ctl", "xargs -I{} ai-ctl {} < list", "ai-ctl status\nai-ctl up",
                     "ai-ctl \\\n  up", "cat <<EOF\n$(ai-ctl up)\nEOF", "find . -name x -exec ai-ctl up \\;",
                     "echo x > out; ai-ctl up 2>&1 | tee log", "ai-ctl up>/dev/null", "cargo +nightly run -- up",
                     'echo "$( (true); ai-ctl up )"', "echo \"$(echo ')'; ai-ctl up)\"",
                     'echo "`echo \\`date\\`; ai-ctl up`"', "echo a#b; ai-ctl up",
                     "cat <<-EOF\n\tx\n\tEOF\nai-ctl up", "diff <(ai-ctl up) x", "env - ai-ctl up",
                     "find . -exec echo {} + -exec ai-ctl up \\;")
        self.refused("unreadable", "eval " * 20 + "ai-ctl status")

    def test_a_command_named_but_not_run_passes(self):
        self.allowed("echo 'ai-ctl up'", 'grep -rn "ai-ctl up" docs/', "ls # ai-ctl up", "man ai-ctl",
                     "cat <<'EOF' > /tmp/notes\nai-ctl up\ngit commit -m x\nmake up\nEOF",
                     "cat <<-\"END\"\n\tai-ctl down\n\tEND", "rg 'cargo run -- up' src",
                     "cat <<'A' <<B\n$(ai-ctl up)\nA\nok\nB", "sudo -S ai-ctl status", "sleep 1& ai-ctl status")

    def test_cargo_run_elsewhere_runs_another_crate(self):
        crate = self.scratch / "crate"
        self.allowed(f"cd {crate} && cargo run -- up", f"cargo run --manifest-path {crate}/Cargo.toml -- up",
                     f"env -C {crate} cargo run -- up", f"sudo -D {crate} cargo run -- up",
                     "cargo run --bin probe -- up", "cargo run --example demo -- up")
        self.allowed("cargo run -- up", cwd=crate)
        self.refused("ai-ctl", f"cd {self.scratch / 'copy'} && cargo run -- up", 'cd "$CRATE" && cargo run -- up',
                     "cargo run --manifest-path $CRATE/Cargo.toml -- up")
        self.refused("ai-ctl", "cargo run -- up", cwd=self.scratch / "copy")
        self.refused("ai-ctl", "cargo run -- up", cwd=self.scratch / "copy" / "src")
        self.refused("ai-ctl", "cargo run -- up", cwd=self.scratch / "broken")
        self.refused("ai-ctl", f"sudo -C {crate} cargo run -- up", f"doas -C {crate} cargo run -- up")

    def test_an_unreadable_command_naming_a_guarded_one_is_refused(self):
        self.refused("unreadable", "ai-ctl status 'up", 'git commit -m "x', "make up 'x", "echo $(ai-ctl up")
        self.allowed("echo 'unclosed", 'printf "%s')


class MakeTest(Folders):
    def test_the_makefile_reading_targets_pass(self):
        self.allowed("make", "make help", "make doctor", "make status", "make -s status", "make doctor status",
                     f"make -C {self.repo} doctor", "make --no-print-directory status", "make -j 4 doctor",
                     "make status V=1", "make doctor 2>&1", "make status > status.log")

    def test_its_other_targets_are_refused(self):
        self.refused("make", "make up", "make down", "make restart", "make bin", "make logs MODEL=qwen",
                     "make llama-update", "make webui-up", "make start", "make doctor up", "make -n up",
                     f"make -C {self.repo} up", f"make --directory={self.repo} up", f"make -f {self.repo}/Makefile up",
                     f"cd {self.scratch / 'copy'} && make up", 'make -C "$DIR" up', "gmake up", "echo up | xargs make")
        self.refused("make", f"make -C{self.repo} up", f"make -f {self.repo}/Makefile up", cwd=self.scratch / "c-project")

    def test_make_elsewhere_runs_another_makefile(self):
        project = self.scratch / "c-project"
        self.allowed(f"make -C {project} all", f"cd {project} && make clean all")
        self.allowed("make all install", cwd=project)


class GitTest(Folders):
    def test_the_reading_subcommands_pass_on_the_repository(self):
        self.allowed("git status", "git status --porcelain", "git log --oneline -3", "git diff a..b -- src/",
                     "git show HEAD:Cargo.toml", "git blame -w src/main.rs", "git grep -n prune",
                     "git ls-files", "git ls-tree -r HEAD", "git rev-parse --short HEAD", "git rev-list --count a..b",
                     "git cat-file -p HEAD", "git archive --format=tar HEAD | tar -x -C /tmp/x",
                     "git --no-pager log -1", f"git -C {self.repo} show HEAD", "git -c color.ui=never diff",
                     "git describe --tags", "git merge-base main HEAD", "git for-each-ref refs/tags",
                     "git shortlog -sn", "git --version", "git help log", "git", "git commit -h")

    def test_the_other_subcommands_are_refused_on_the_repository(self):
        self.refused("git-write", "git commit -m x", "git add .", "git checkout main", "git switch -c x",
                     "git stash", "git reset --hard", "git branch x", "git tag v1", "git worktree add /tmp/x",
                     "git clean -fd", "git restore src/", "git rm x", "git mv a b", "git fetch", "git pull",
                     "git push", "git merge x", "git rebase main", "git cherry-pick a", "git revert a", "git gc",
                     "git config user.name x", "git notes add", "git update-ref HEAD a", "git apply p.diff",
                     "git am p.mbox", "git init", "git clone . /tmp/copy", "git st", f"git -C {self.repo} commit",
                     f"git -C {self.repo}/src commit", f"git -C {self.scratch} -C ../ai-ctl commit",
                     f"git -C {self.scratch / 'link'} commit", f"git --git-dir={self.scratch}/x/.git checkout .",
                     f"git --work-tree={self.repo} --git-dir={self.scratch}/x/.git checkout .",
                     f"GIT_DIR={self.repo}/.git git -C {self.scratch} commit",
                     f"(cd {self.scratch} && git init) && git commit -m x", f"cd {self.scratch} & git commit",
                     f"cd {self.scratch} | git commit", f"cd {self.scratch} || git commit", "git commit -m -h",
                     "echo commit | xargs git", 'GIT_DIR="$X" git commit', f"true | cd {self.scratch}; git commit")

    def test_any_subcommand_passes_on_an_absolute_path_outside(self):
        repo = self.scratch / "repo"
        self.allowed(f"git -C {repo} init", f"git -C {repo} commit -m x", f"git -C {self.scratch} clone {self.repo} copy",
                     f"cd {self.scratch} && git init repo", f"cd {self.scratch}; git init repo",
                     f"cd {self.scratch}\ngit init repo", f"cd {self.scratch} && cd crate && git init",
                     f"git -C {self.scratch} -C crate init", f"git --git-dir={self.scratch}/x/.git --work-tree={self.scratch} add .",
                     f"cd ~ && git -C {self.scratch} init repo", f"cd {self.scratch} || exit 1; git init repo",
                     "cd && git init repo")
        self.allowed("git commit -m x", "git checkout -b x", cwd=self.scratch)
        self.allowed(f"git -C {self.sibling} init", "git init", cwd=self.sibling)

    def test_a_target_that_does_not_resolve_is_refused(self):
        self.refused("git-target", 'git -C "$REPO" commit -m x', "git -C $(mktemp -d) init",
                     "git -C `mktemp -d` init", f"cd {self.scratch}/missing && git init", "cd $TMP && git init",
                     "cd - && git init", "git -C ~/x* init", f"cd {self.scratch}/missing; git init",
                     "git -C ~no-such-user/x init", f"pushd {self.scratch} && popd && git init")
        for command in ("git init", "git -C x init"):
            with self.subTest(command=command):
                self.assertIn("git-target", [kind for kind, _ in guard.check(command, None, str(self.repo))])
        self.allowed('git -C "$REPO" log', "cd $TMP && git status", "git -C ../scratch commit -m x")

    def test_git_named_but_not_run_passes(self):
        self.allowed("echo git commit", "grep -rn 'git push' .", "ls git commit", "cat .git/HEAD")


class ListeningTest(unittest.TestCase):
    def test_each_process_gets_its_listening_sockets(self):
        sockets = guard.listening((CAPTURES / "ss-listening.txt").read_text(encoding="utf-8"))
        self.assertEqual(sockets.get(3972728), ["udp [::1]:56765", "tcp 127.0.0.1:49803"])
        self.assertEqual(sockets.get(2525034), ["tcp 127.0.0.1:41919"])
        self.assertIn("udp 192.0.2.1:3702", sockets.get(19329, []))
        self.assertIn("udp [fe80::1]%wlp130s0f0:3702", sockets.get(19329, []))
        self.assertEqual(len(sockets.get(19329, [])), 6)
        self.assertEqual(sorted(sockets), [19329, 2525034, 2525063, 2526341, 2745251, 2933369, 3972728])


class PreToolUseTest(Folders):
    def env(self):
        return {**GIT_ENV, "CLAUDE_PROJECT_DIR": str(self.repo)}

    def denied(self, data):
        done = hook(data, self.env())
        self.assertEqual(done.returncode, 0, done.stderr)
        self.assertNotEqual(done.stdout, "", "no decision")
        output = json.loads(done.stdout)["hookSpecificOutput"]
        self.assertEqual((output["hookEventName"], output["permissionDecision"]), ("PreToolUse", "deny"))
        return output["permissionDecisionReason"]

    def test_a_review_agents_call_is_refused_with_claude_codes_decision(self):
        self.assertIn("ai-ctl up", self.denied(bash("ai-ctl up", self.repo)))

    def test_every_refusal_of_a_command_is_named(self):
        reason = self.denied(bash("git commit -m x && make up", self.repo))
        self.assertIn("git commit -m x", reason)
        self.assertIn("make up", reason)

    def test_an_allowed_call_gets_no_decision(self):
        done = hook(bash("ai-ctl status && git log -1", self.repo), self.env())
        self.assertEqual((done.returncode, done.stdout), (0, ""), done.stderr)

    def test_the_session_and_other_agents_are_not_guarded(self):
        main = capture("claude-hook-pre-tool-use-main.json", cwd=str(self.repo))
        main["tool_input"] = {**main["tool_input"], "command": "git commit -m x && ai-ctl up"}
        for data in (main, bash("ai-ctl up", self.repo, agent="Explore"), bash("ai-ctl up", self.repo, agent="reviewer")):
            with self.subTest(agent=data.get("agent_type")):
                done = hook(data, self.env())
                self.assertEqual((done.returncode, done.stdout), (0, ""), done.stderr)

    def test_another_tool_is_not_guarded(self):
        data = bash("ai-ctl up", self.repo)
        data.update(tool_name="Read", tool_input={"file_path": str(self.repo / "Cargo.toml")})
        done = hook(data, self.env())
        self.assertEqual((done.returncode, done.stdout), (0, ""), done.stderr)

    def test_a_call_the_guard_fails_on_is_refused(self):
        data = bash("ai-ctl status", self.repo)
        data["tool_input"] = ["ai-ctl status"]
        self.denied(data)

    def test_a_call_with_no_command_is_refused(self):
        data = bash("ai-ctl status", self.repo)
        del data["tool_input"]["command"]
        self.denied(data)

    def test_an_unreadable_input_blocks(self):
        for text in ("not json", "[]", ""):
            with self.subTest(text=text):
                done = hook(text, self.env())
                self.assertEqual(done.returncode, 2)
                self.assertIn("guard.py", done.stderr)


class Sandbox(unittest.TestCase):
    """A repository and a state folder; this test process stands for `claude`, CLAUDE_PID."""

    def setUp(self):
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        root = Path(temporary.name).resolve()
        (root / "repo").mkdir()
        self.repo = Repo(root / "repo")
        self.repo.commit("base", {"src/main.rs": "fn main() {}\n", "README.md": "ai-ctl\n"})
        self.runtime = root / "runtime"
        self.runtime.mkdir(mode=0o700)
        self.env = {**GIT_ENV, "CLAUDE_PID": str(os.getpid()), "CLAUDE_PROJECT_DIR": str(self.repo.folder),
                    "XDG_RUNTIME_DIR": str(self.runtime)}

    def start(self, agent="review-security"):
        data = capture("claude-hook-subagent-start.json", cwd=str(self.repo.folder), agent_type=agent)
        done = hook(data, self.env)
        self.assertEqual((done.returncode, done.stdout), (0, ""), done.stderr)

    def stop(self, active=False, tasks=(), agent="review-security"):
        name = "claude-hook-subagent-stop-active.json" if active else "claude-hook-subagent-stop.json"
        data = capture(name, cwd=str(self.repo.folder), agent_type=agent, background_tasks=list(tasks))
        done = hook(data, self.env)
        self.assertEqual(done.returncode, 0, done.stderr)
        return json.loads(done.stdout) if done.stdout else None

    def states(self):
        folder = self.runtime / "ai-ctl-review-guard"
        return sorted(p.name for p in folder.iterdir()) if folder.exists() else []

    def blocked(self, **arguments):
        decision = self.stop(**arguments)
        self.assertIsNotNone(decision, "the stop is let through")
        self.assertEqual(decision["decision"], "block")
        return decision["reason"]

    def orphan(self, claude_pid=None):
        """A loopback server whose parent has exited, left as `nohup … &` leaves one: its pid and port."""
        env = {**self.env, "CLAUDE_PID": str(claude_pid or os.getpid())}
        shell = subprocess.Popen(["sh", "-c", f'"{sys.executable}" -I -c "$0" &\necho pid $!', LISTEN],
                                 stdout=subprocess.PIPE, text=True, env=env)
        found = dict(shell.stdout.readline().split() for _ in range(2))
        self.addCleanup(kill, int(found["pid"]))
        shell.stdout.close()
        shell.wait()
        return int(found["pid"]), int(found["port"])

    def child(self):
        """A process of this one, as a review agent's running command is `claude`'s descendant."""
        process = subprocess.Popen([sys.executable, "-I", "-c", LISTEN], stdout=subprocess.PIPE, text=True,
                                   env=self.env)
        self.addCleanup(process.wait)
        self.addCleanup(process.kill)
        self.assertEqual(process.stdout.readline().split()[0], "port")
        return process.pid


def kill(pid):
    try:
        os.kill(pid, signal.SIGKILL)
    except ProcessLookupError:
        pass


class StopTest(Sandbox):
    def test_a_clean_agent_stops_and_its_baseline_goes(self):
        self.start()
        self.assertEqual(len(self.states()), 1)
        self.assertIsNone(self.stop())
        self.assertEqual(self.states(), [])

    def test_a_change_to_the_repository_blocks_the_first_stop_only(self):
        self.start()
        self.repo.write("notes.txt", "left\n")
        self.repo.write("src/main.rs", "fn main() { println!(); }\n")
        reason = self.blocked()
        self.assertIn("?? notes.txt", reason)
        self.assertIn(" M src/main.rs", reason)
        self.assertIsNone(self.stop(active=True))
        self.assertEqual(self.states(), [])

    def test_a_file_changed_before_the_start_counts_only_when_it_changes_again(self):
        self.repo.write("README.md", "the session's change\n")
        self.start()
        self.assertIsNone(self.stop())
        self.start()
        self.repo.write("README.md", "the agent's change\n")
        self.assertIn(" M README.md", self.blocked())

    def test_a_file_added_to_a_folder_already_untracked_counts(self):
        self.repo.write("scratch/a.txt", "the session's\n")
        self.start()
        self.repo.write("scratch/b.txt", "the agent's\n")
        self.assertIn("?? scratch/b.txt", self.blocked())

    def test_a_change_undone_counts(self):
        self.repo.write("README.md", "the session's change\n")
        self.start()
        self.repo.git("checkout", "README.md")
        self.assertIn("README.md", self.blocked())

    def test_a_process_left_behind_blocks_with_its_ports(self):
        self.start()
        pid, port = self.orphan()
        reason = self.blocked()
        self.assertIn(str(pid), reason)
        self.assertIn(f"tcp 127.0.0.1:{port}", reason)

    def test_a_running_command_an_older_process_and_another_sessions_are_not_left_behind(self):
        older, _ = self.orphan()
        self.start()
        running = self.child()
        other, _ = self.orphan(claude_pid=1)
        self.assertIsNone(self.stop(), (older, running, other))

    def test_a_background_task_still_running_blocks(self):
        self.start()
        tasks = [{"id": "blbw1ovj6", "type": "shell", "status": "running", "description": "Sleep",
                  "command": "sleep 97"},
                 {"id": "b2", "type": "shell", "status": "completed", "description": "Done", "command": "true"}]
        reason = self.blocked(tasks=tasks)
        self.assertIn("blbw1ovj6", reason)
        self.assertNotIn("b2", reason)

    def test_a_stop_with_no_baseline_is_blocked_once(self):
        self.blocked()
        self.assertIsNone(self.stop(active=True))

    def test_other_agents_are_not_guarded(self):
        self.start(agent="Explore")
        self.assertEqual(self.states(), [])
        self.repo.write("notes.txt", "left\n")
        self.assertIsNone(self.stop(agent="Explore"))

    def test_a_project_that_is_no_repository_blocks_with_why(self):
        self.env["CLAUDE_PROJECT_DIR"] = str(self.runtime)
        self.start()
        self.assertIn("git status", self.blocked())
        self.assertIsNone(self.stop(active=True))

    def test_a_state_folder_another_user_could_reach_is_refused(self):
        folder = self.runtime / "ai-ctl-review-guard"
        folder.mkdir(mode=0o777)
        folder.chmod(0o777)
        self.start()
        self.assertEqual(self.states(), [])
        self.blocked()

    def test_an_agent_named_as_a_path_writes_no_state(self):
        data = capture("claude-hook-subagent-start.json", cwd=str(self.repo.folder), agent_type="review-tests",
                       session_id="../../escaped")
        hook(data, self.env)
        self.assertEqual(list(self.runtime.parent.glob("**/escaped*")), [])
        data.update(hook_event_name="SubagentStop", stop_hook_active=False)
        done = hook(data, self.env)
        self.assertEqual(json.loads(done.stdout or "{}").get("decision"), "block")

    def test_without_a_runtime_folder_the_state_goes_to_the_temporary_one(self):
        del self.env["XDG_RUNTIME_DIR"]
        self.env["TMPDIR"] = str(self.runtime)
        self.start()
        self.assertEqual(len(list(self.runtime.glob(f"ai-ctl-review-guard-{os.getuid()}/*"))), 1)
        self.assertIsNone(self.stop())
        self.assertEqual(list(self.runtime.glob(f"ai-ctl-review-guard-{os.getuid()}/*")), [])


if __name__ == "__main__":
    unittest.main()
