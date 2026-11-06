//! `ai-ctl build`: runs a build with its progress on stderr, and renders the
//! installed builds.

use std::fmt::Write as _;
use std::io::{self, IsTerminal, Write};
use std::process::ExitCode;
use std::time::{Duration, Instant};

use anyhow::{Context, Result};

use crate::application::build::{self, Builder, Built, Event, Listing, Pruned, Request};
use crate::application::ports::Progress;
use crate::domain::build::BuildId;

use super::cli::{BuildAction, BuildCommand};
use super::{Services, current_line};

/// Runs `ai-ctl build` and its subcommands.
pub fn run(command: BuildCommand, services: &Services) -> Result<ExitCode> {
    let store = services.builds.as_ref();
    let warn = &mut |warning: String| {
        let _ = writeln!(io::stderr(), "warning: {warning}");
    };
    let output = match command.action {
        Some(BuildAction::List { project }) => {
            render_list(&build::list(store, project.as_deref(), warn)?)
        }
        Some(BuildAction::Use { project, id }) => {
            let previous = build::use_build(store, &project, &id, warn)?;
            render_use(&project, &id, previous.as_ref())
        }
        Some(BuildAction::Prune { keep, project }) => {
            let gateway = services.gateway.as_ref();
            render_pruned(&build::prune(store, gateway, project.as_deref(), keep)?)
        }
        None => {
            let project = command
                .project
                .context("a project to build is required, such as llama.cpp")?;
            let builder = Builder {
                gpu: services.gpu.as_ref(),
                toolchain: services.toolchain.as_ref(),
                sources: services.sources.as_ref(),
                compiler: services.compiler.as_ref(),
                store,
                gateway: services.gateway.as_ref(),
                programs: services.programs.as_ref(),
                clock: &services.clock,
            };
            let request = Request {
                project: &project,
                reference: Some(&command.reference),
                force: command.force,
                jobs: command.jobs.map(std::num::NonZero::get),
            };
            let started = Instant::now();
            let mut reporter = Reporter::new(io::stderr(), io::stderr().is_terminal());
            let built = builder.build(&request, &mut |event| reporter.event(event));
            reporter.end_line();
            render_built(&built?, started.elapsed())
        }
    };
    super::print(&output)?;
    Ok(ExitCode::SUCCESS)
}

/// Follows a build: one line per step, and on a terminal a progress line
/// rewritten in place. Write errors are ignored: the build goes on.
pub struct Reporter<W: Write> {
    out: W,
    terminal: bool,
    /// A progress line is on screen without its newline.
    open_line: bool,
    /// The last quarter reported off a terminal.
    quarter: Option<u32>,
}

impl<W: Write> Reporter<W> {
    pub fn new(out: W, terminal: bool) -> Self {
        Self {
            out,
            terminal,
            open_line: false,
            quarter: None,
        }
    }

    pub fn event(&mut self, event: Event) {
        let line = match event {
            Event::Fetching => "Fetching the sources".to_owned(),
            Event::CheckedOut {
                reference,
                id,
                commit,
            } => {
                format!("Checked out {reference} ({}) as build {id}", commit.short())
            }
            Event::Patched { patch } => format!("Applied {patch}"),
            Event::Compile(Progress::Configuring) => "Configuring".to_owned(),
            Event::Compile(Progress::Building { done, total }) => {
                self.progress(done, total);
                return;
            }
            Event::Testing => "Testing the new binary".to_owned(),
            Event::Warning(message) => format!("warning: {message}"),
        };
        self.end_line();
        let _ = writeln!(self.out, "{line}");
    }

    fn progress(&mut self, done: u32, total: u32) {
        let percent = done * 100 / total.max(1);
        if self.terminal {
            let _ = write!(self.out, "\r\x1b[2KCompiling {done}/{total} ({percent}%)");
            let _ = self.out.flush();
            self.open_line = true;
        } else {
            let quarter = percent / 25;
            if self.quarter != Some(quarter) {
                self.quarter = Some(quarter);
                let _ = writeln!(self.out, "Compiling {done}/{total} ({percent}%)");
            }
        }
    }

    /// Ends a progress line left open, before other output.
    pub fn end_line(&mut self) {
        if self.open_line {
            let _ = writeln!(self.out);
            self.open_line = false;
        }
    }
}

pub fn render_built(built: &Built, elapsed: Duration) -> String {
    let build = &built.build;
    let mut out = format!(
        "Built {} {} in {}: commit {}, CUDA {}, {} {}\n",
        build.project,
        build.id,
        duration(elapsed),
        build.commit.short(),
        build.toolkit,
        build.host_compiler.command,
        build.host_compiler.version
    );
    if !build.patches.is_empty() {
        let _ = writeln!(out, "Patches applied: {}", build.patches.join(", "));
    }
    current_line(&mut out, "build", &built.binary, built.previous.as_ref());
    out
}

/// `4 min 10 s`, or `52 s` under a minute.
fn duration(elapsed: Duration) -> String {
    let seconds = elapsed.as_secs();
    if seconds < 60 {
        format!("{seconds} s")
    } else {
        format!("{} min {} s", seconds / 60, seconds % 60)
    }
}

pub fn render_list(listings: &[Listing]) -> String {
    let mut out = String::new();
    for listing in listings {
        if listing.builds.is_empty() && listing.unreadable.is_empty() {
            let _ = writeln!(
                out,
                "{}: no build yet; `ai-ctl build {}` makes one",
                listing.project, listing.project
            );
            continue;
        }
        let _ = writeln!(out, "{}", listing.project);
        let width = listing
            .builds
            .iter()
            .map(|build| build.id.as_str().len())
            .max()
            .unwrap_or(0);
        for build in &listing.builds {
            let marker = if listing.current.as_ref() == Some(&build.id) {
                '*'
            } else {
                ' '
            };
            let _ = writeln!(
                out,
                "  {marker} {:<width$}  {}  {}  CUDA {}  {} {}",
                build.id.as_str(),
                build.built_at.minutes(),
                build.commit.short(),
                build.toolkit,
                build.host_compiler.command,
                build.host_compiler.version
            );
        }
        for entry in &listing.unreadable {
            let _ = writeln!(
                out,
                "  not a build: {} ({})",
                entry.dir.display(),
                entry.reason
            );
        }
    }
    out
}

pub fn render_use(project: &str, id: &str, previous: Option<&BuildId>) -> String {
    match previous {
        Some(previous) => format!("{project}: current build is now {id} (was {previous})\n"),
        None => format!("{project}: current build is {id}\n"),
    }
}

pub fn render_pruned(pruned: &[Pruned]) -> String {
    let mut out = String::new();
    for project in pruned {
        let removed: Vec<&str> = project.removed.iter().map(BuildId::as_str).collect();
        let what = if removed.is_empty() {
            "nothing to remove".to_owned()
        } else {
            format!("removed {}", removed.join(", "))
        };
        let _ = writeln!(out, "{}: {what}; {} kept", project.project, project.kept);
        for entry in &project.unreadable {
            let _ = writeln!(
                out,
                "  left alone, not a build: {} ({})",
                entry.dir.display(),
                entry.reason
            );
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::path::PathBuf;

    use super::*;
    use crate::application::ports::Unreadable;
    use crate::domain::build::{Build, CommitId};
    use crate::domain::machine::{HostCompiler, Version};
    use crate::domain::timestamp::Timestamp;

    fn build(id: &str, at: &str) -> Build {
        Build {
            project: "llama.cpp".into(),
            id: BuildId::parse(id).unwrap(),
            commit: CommitId::parse("79e2e74eb11022c1ba2e438df7f0ca2d4c10f8b6").unwrap(),
            toolkit: Version::new(13, 4, 92),
            host_compiler: HostCompiler {
                command: "g++".into(),
                path: PathBuf::from("/usr/bin/g++"),
                version: Version::new(16, 2, 1),
            },
            cmake_flags: BTreeMap::new(),
            patches: vec![],
            built_at: Timestamp::parse(at).unwrap(),
        }
    }

    fn id(text: &str) -> BuildId {
        BuildId::parse(text).unwrap()
    }

    #[test]
    fn a_build_reports_what_it_made_and_what_it_replaced() {
        let built = Built {
            build: build("b11538", "2026-10-09T20:36:00Z"),
            binary: "/data/builds/llama.cpp/b11538/llama-server".into(),
            previous: Some(id("b11400")),
        };
        assert_eq!(
            render_built(&built, Duration::from_secs(250)),
            "Built llama.cpp b11538 in 4 min 10 s: commit 79e2e74eb1, CUDA 13.4.92, g++ 16.2.1\n\
             Current build: /data/builds/llama.cpp/b11538/llama-server (was b11400)\n"
        );
        let first = Built {
            previous: None,
            ..built
        };
        assert!(render_built(&first, Duration::from_secs(52)).contains(" in 52 s: "));
        let mut patched = first;
        patched.build.patches = vec!["api-key.patch".into(), "listen-failure.patch".into()];
        assert_eq!(
            render_built(&patched, Duration::from_secs(52)),
            "Built llama.cpp b11538 in 52 s: commit 79e2e74eb1, CUDA 13.4.92, g++ 16.2.1\n\
             Patches applied: api-key.patch, listen-failure.patch\n\
             Current build: /data/builds/llama.cpp/b11538/llama-server\n"
        );
    }

    #[test]
    fn the_list_marks_the_current_build() {
        let listing = Listing {
            project: "llama.cpp",
            builds: vec![
                build("b11538", "2026-10-09T20:36:00Z"),
                build("79e2e74eb1", "2026-10-08T09:05:00Z"),
            ],
            current: Some(id("b11538")),
            unreadable: vec![Unreadable {
                dir: "/data/builds/llama.cpp/notes".into(),
                reason: "cannot read build.json".into(),
            }],
        };
        assert_eq!(
            render_list(&[listing]),
            "llama.cpp\n\
             \x20 * b11538      2026-10-09 20:36 UTC  79e2e74eb1  CUDA 13.4.92  g++ 16.2.1\n\
             \x20   79e2e74eb1  2026-10-08 09:05 UTC  79e2e74eb1  CUDA 13.4.92  g++ 16.2.1\n\
             \x20 not a build: /data/builds/llama.cpp/notes (cannot read build.json)\n"
        );
    }

    #[test]
    fn an_empty_list_says_how_to_build() {
        let listing = Listing {
            project: "llama.cpp",
            builds: vec![],
            current: None,
            unreadable: vec![],
        };
        assert_eq!(
            render_list(&[listing]),
            "llama.cpp: no build yet; `ai-ctl build llama.cpp` makes one\n"
        );
    }

    #[test]
    fn a_dangling_current_link_is_left_to_its_warning() {
        let listing = Listing {
            project: "llama.cpp",
            builds: vec![build("b1", "2026-10-09T20:36:00Z")],
            current: Some(id("b9")),
            unreadable: vec![],
        };
        assert_eq!(
            render_list(&[listing]),
            "llama.cpp\n\
             \x20   b1  2026-10-09 20:36 UTC  79e2e74eb1  CUDA 13.4.92  g++ 16.2.1\n"
        );
    }

    #[test]
    fn use_and_prune_say_what_changed() {
        assert_eq!(
            render_use("llama.cpp", "b1", Some(&id("b2"))),
            "llama.cpp: current build is now b1 (was b2)\n"
        );
        assert_eq!(
            render_use("llama.cpp", "b1", None),
            "llama.cpp: current build is b1\n"
        );
        let pruned = [Pruned {
            project: "llama.cpp",
            removed: vec![id("b1"), id("b2")],
            kept: 2,
            unreadable: vec![],
        }];
        assert_eq!(
            render_pruned(&pruned),
            "llama.cpp: removed b1, b2; 2 kept\n"
        );
        let nothing = [Pruned {
            project: "llama.cpp",
            removed: vec![],
            kept: 1,
            unreadable: vec![],
        }];
        assert_eq!(
            render_pruned(&nothing),
            "llama.cpp: nothing to remove; 1 kept\n"
        );
    }

    fn report(terminal: bool, events: Vec<Event>) -> String {
        let mut out = Vec::new();
        let mut reporter = Reporter::new(&mut out, terminal);
        for event in events {
            reporter.event(event);
        }
        reporter.end_line();
        String::from_utf8(out).unwrap()
    }

    fn compiling(done: u32) -> Event {
        Event::Compile(Progress::Building { done, total: 356 })
    }

    #[test]
    fn a_terminal_sees_one_progress_line_rewritten() {
        let text = report(
            true,
            vec![
                Event::Compile(Progress::Configuring),
                compiling(1),
                compiling(356),
                Event::Testing,
            ],
        );
        assert_eq!(
            text,
            "Configuring\n\r\x1b[2KCompiling 1/356 (0%)\r\x1b[2KCompiling 356/356 (100%)\nTesting the new binary\n"
        );
    }

    #[test]
    fn a_log_sees_the_progress_by_quarters() {
        let events = (1..=356).map(compiling).collect();
        let text = report(false, events);
        assert_eq!(
            text,
            "Compiling 1/356 (0%)\nCompiling 89/356 (25%)\nCompiling 178/356 (50%)\nCompiling 267/356 (75%)\nCompiling 356/356 (100%)\n"
        );
    }

    #[test]
    fn steps_name_the_reference_and_the_build() {
        let text = report(
            false,
            vec![
                Event::Fetching,
                Event::CheckedOut {
                    reference: "b11538".into(),
                    id: id("b11538"),
                    commit: CommitId::parse("79e2e74eb11022c1ba2e438df7f0ca2d4c10f8b6").unwrap(),
                },
                Event::Patched {
                    patch: "api-key.patch",
                },
                Event::Warning("cannot remove /work".into()),
            ],
        );
        assert_eq!(
            text,
            "Fetching the sources\nChecked out b11538 (79e2e74eb1) as build b11538\n\
             Applied api-key.patch\nwarning: cannot remove /work\n"
        );
    }
}
