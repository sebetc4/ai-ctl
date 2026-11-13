//! `ai-ctl build`: runs a build with its progress on stderr, and renders the
//! installed builds.

use std::fmt::Write as _;
use std::io::{self, IsTerminal, Write};
use std::process::ExitCode;
use std::time::{Duration, Instant};

use anyhow::{Context, Result};

use crate::application::build::{
    self, Builder, Built, Event, Listing, Outdated, Pruned, Request, UpdateOutcome, Updated,
};
use crate::application::ports::Progress;
use crate::domain::build::{BuildId, Standing};
use crate::domain::gateway::{OLDEST, SwapStanding};

use super::cli::{BuildAction, BuildCommand};
use super::{Services, current_line, table};

/// `build outdated`'s exit status when `build update` has something to build.
const OUTDATED: u8 = 4;

/// Runs `ai-ctl build` and its subcommands.
pub fn run(command: BuildCommand, services: &Services) -> Result<ExitCode> {
    let store = services.builds.as_ref();
    let warn = &mut |warning: String| {
        let _ = writeln!(io::stderr(), "warning: {warning}");
    };
    let output = match command.action {
        Some(BuildAction::Outdated { project }) => {
            let outdated = read_outdated(services, project.as_deref(), warn)?;
            super::print(&render_outdated(&outdated))?;
            return Ok(ExitCode::from(outdated_status(&outdated)));
        }
        Some(BuildAction::Update { jobs, project }) => {
            let outdated = read_outdated(services, project.as_deref(), warn)?;
            let mut reporter = Reporter::new(io::stderr(), io::stderr().is_terminal());
            let updated = builder(services).update(
                &outdated.projects,
                jobs.map(std::num::NonZero::get),
                &mut |event| reporter.event(event),
            );
            reporter.end_line();
            super::print(&render_updated(&updated))?;
            return Ok(ExitCode::from(update_status(&updated)));
        }
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
            let request = Request {
                project: &project,
                reference: Some(&command.reference),
                force: command.force,
                jobs: command.jobs.map(std::num::NonZero::get),
            };
            let started = Instant::now();
            let mut reporter = Reporter::new(io::stderr(), io::stderr().is_terminal());
            let built = builder(services).build(&request, &mut |event| reporter.event(event));
            reporter.end_line();
            render_built(&built?, started.elapsed())
        }
    };
    super::print(&output)?;
    Ok(ExitCode::SUCCESS)
}

/// Where each project stands, `project` alone when named, on the
/// composition root's adapters.
fn read_outdated(
    services: &Services,
    project: Option<&str>,
    warn: &mut dyn FnMut(String),
) -> Result<Outdated> {
    build::outdated(
        services.sources.as_ref(),
        services.builds.as_ref(),
        services.inventory.as_ref(),
        services.releases.as_ref(),
        project,
        warn,
    )
}

/// The build use case, on the composition root's adapters.
fn builder(services: &Services) -> Builder<'_> {
    Builder {
        gpu: services.gpu.as_ref(),
        toolchain: services.toolchain.as_ref(),
        sources: services.sources.as_ref(),
        compiler: services.compiler.as_ref(),
        store: services.builds.as_ref(),
        gateway: services.gateway.as_ref(),
        programs: services.programs.as_ref(),
        clock: &services.clock,
    }
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
            Event::Updating { project } => format!("Updating {project}"),
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

/// `build outdated`'s exit status: 0 when each project is current, 4 when
/// `build update` has something to build, 1, over 4, when a project could
/// not be read. llama-swap never changes it: `build update` cannot act on it.
pub fn outdated_status(outdated: &Outdated) -> u8 {
    if outdated
        .projects
        .iter()
        .any(|project| project.found.is_err())
    {
        return 1;
    }
    let to_build = outdated
        .projects
        .iter()
        .any(|project| matches!(&project.found, Ok(found) if found.standing != Standing::Current));
    if to_build { OUTDATED } else { 0 }
}

/// One line per project, its current build, its newest release and its
/// verdict, in columns, and llama-swap's last.
pub fn render_outdated(outdated: &Outdated) -> String {
    let shown = |current: Option<String>| current.unwrap_or_else(|| "-".to_owned());
    let mut rows: Vec<Vec<String>> = outdated
        .projects
        .iter()
        .map(|project| {
            let (newest, verdict) = match &project.found {
                Ok(found) => (
                    format!("newest {}", found.newest),
                    standing_verdict(&found.standing),
                ),
                Err(cause) => (
                    "newest unknown".to_owned(),
                    format!("cannot read its releases: {cause}"),
                ),
            };
            let current = project.current.as_ref().map(BuildId::to_string);
            vec![project.project.clone(), shown(current), newest, verdict]
        })
        .collect();
    rows.push(vec![
        "llama-swap".to_owned(),
        shown(outdated.swap.map(|(release, _)| release.to_string())),
        format!("pinned {}", outdated.pinned),
        swap_verdict(outdated.swap.map(|(_, standing)| standing)),
    ]);
    table(&rows, "")
}

fn standing_verdict(standing: &Standing) -> String {
    match standing {
        Standing::Current => "current".to_owned(),
        Standing::Behind => "behind".to_owned(),
        Standing::NotBuilt => "not built".to_owned(),
        Standing::FromCommit => "built from a commit".to_owned(),
        Standing::OffRecipe(how) => format!("off its recipe: {how}"),
    }
}

/// llama-swap's verdict against the pin alone: ai-ctl installs no other
/// release.
fn swap_verdict(standing: Option<SwapStanding>) -> String {
    match standing {
        None => "not installed; ai-ctl gateway install".to_owned(),
        Some(SwapStanding::TooOld) => {
            format!("behind, and up needs {OLDEST} or later; ai-ctl gateway install")
        }
        Some(SwapStanding::Behind) => "behind; ai-ctl gateway install".to_owned(),
        Some(SwapStanding::Ahead) => {
            "ahead of the pin; ai-ctl gateway install goes back to it".to_owned()
        }
        Some(SwapStanding::Pinned) => "current".to_owned(),
    }
}

/// `build update`'s exit status: 0 when nothing failed, 1 otherwise.
pub fn update_status(updated: &Updated) -> u8 {
    let failed = |outcome: &UpdateOutcome| matches!(outcome, UpdateOutcome::Failed(_));
    u8::from(
        updated
            .projects
            .iter()
            .any(|project| failed(&project.outcome)),
    )
}

/// One line per project, what became of it, then what the new builds leave
/// to do.
pub fn render_updated(updated: &Updated) -> String {
    let rows: Vec<Vec<String>> = updated
        .projects
        .iter()
        .map(|project| vec![project.project.clone(), updated_what(&project.outcome)])
        .collect();
    let mut out = table(&rows, "");
    let built: Vec<&Option<BuildId>> = updated
        .projects
        .iter()
        .filter_map(|project| match &project.outcome {
            UpdateOutcome::Built { previous, .. } => Some(previous),
            _ => None,
        })
        .collect();
    if built.iter().any(|previous| previous.is_some()) {
        out.push_str(
            "The builds replaced stay installed: `ai-ctl build use <project> <id>` goes back to one.\n",
        );
    }
    if updated.gateway_runs && !built.is_empty() {
        out.push_str(
            "The running gateway keeps its builds until `ai-ctl down` then `ai-ctl up`.\n",
        );
    }
    out
}

/// What became of a project `build update` tried.
fn updated_what(outcome: &UpdateOutcome) -> String {
    match outcome {
        UpdateOutcome::Built {
            id,
            previous: Some(previous),
        } => format!("built {id} (was {previous})"),
        UpdateOutcome::Built { id, previous: None } => format!("built {id}"),
        UpdateOutcome::AlreadyCurrent => "already current".to_owned(),
        UpdateOutcome::Failed(cause) => format!("failed: {cause}"),
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
    use crate::application::build::{Found, ProjectStanding, ProjectUpdate};
    use crate::application::ports::Unreadable;
    use crate::domain::build::{Build, CommitId};
    use crate::domain::gateway::{PINNED, Release, swap_standing};
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

    fn found(project: &str, current: &str, newest: &str, standing: Standing) -> ProjectStanding {
        ProjectStanding {
            project: project.into(),
            current: (!current.is_empty()).then(|| id(current)),
            found: Ok(Found {
                newest: newest.into(),
                standing,
            }),
        }
    }

    fn outdated(projects: Vec<ProjectStanding>, swap: Option<&str>) -> Outdated {
        Outdated {
            projects,
            swap: swap.map(|release| {
                let release = Release::parse(release).unwrap();
                (release, swap_standing(release))
            }),
            pinned: Release::parse("v271").unwrap(),
        }
    }

    #[test]
    fn outdated_sets_each_project_and_llama_swap_in_columns() {
        let projects = vec![
            found("llama.cpp", "b12111", "b12177", Standing::Behind),
            found("whisper.cpp", "v1.9.5", "v1.9.5", Standing::Current),
            found(
                "stable-diffusion.cpp",
                "master-1024-77b72a2",
                "master-1043-0a1b2c3",
                Standing::Behind,
            ),
            found("audio.cpp", "", "v0.9.4", Standing::NotBuilt),
        ];
        assert_eq!(
            render_outdated(&outdated(projects, Some("v270"))),
            "llama.cpp             b12111               newest b12177               behind\n\
             whisper.cpp           v1.9.5               newest v1.9.5               current\n\
             stable-diffusion.cpp  master-1024-77b72a2  newest master-1043-0a1b2c3  behind\n\
             audio.cpp             -                    newest v0.9.4               not built\n\
             llama-swap            v270                 pinned v271                 behind; ai-ctl gateway install\n"
        );
    }

    #[test]
    fn outdated_names_a_commit_build_and_a_build_off_its_recipe() {
        let projects = vec![
            found("llama.cpp", "79e2e74eb1", "b12177", Standing::FromCommit),
            found(
                "whisper.cpp",
                "v1.9.5",
                "v1.9.5",
                Standing::OffRecipe("v1.9.5 was built without reuse-address.patch".into()),
            ),
        ];
        assert_eq!(
            render_outdated(&outdated(projects, Some("v271"))),
            "llama.cpp    79e2e74eb1  newest b12177  built from a commit\n\
             whisper.cpp  v1.9.5      newest v1.9.5  off its recipe: v1.9.5 was built without reuse-address.patch\n\
             llama-swap   v271        pinned v271    current\n"
        );
    }

    #[test]
    fn outdated_says_a_project_it_could_not_read_and_the_others() {
        let projects = vec![
            ProjectStanding {
                project: "llama.cpp".into(),
                current: Some(id("b12111")),
                found: Err("cannot read the tags: no route".into()),
            },
            found("whisper.cpp", "v1.9.5", "v1.9.5", Standing::Current),
        ];
        assert_eq!(
            render_outdated(&outdated(projects, Some("v271"))),
            "llama.cpp    b12111  newest unknown  cannot read its releases: cannot read the tags: no route\n\
             whisper.cpp  v1.9.5  newest v1.9.5   current\n\
             llama-swap   v271    pinned v271     current\n"
        );
    }

    #[test]
    fn llama_swap_is_told_against_the_pin_with_the_command_that_installs_it() {
        let line = |swap| {
            let text = render_outdated(&outdated(vec![], swap));
            text.trim_end().to_owned()
        };
        assert_eq!(line(Some("v271")), "llama-swap  v271  pinned v271  current");
        assert_eq!(
            line(Some("v270")),
            "llama-swap  v270  pinned v271  behind; ai-ctl gateway install"
        );
        assert_eq!(
            line(Some("v272")),
            "llama-swap  v272  pinned v271  ahead of the pin; ai-ctl gateway install goes back to it"
        );
        assert_eq!(
            line(Some("v266")),
            "llama-swap  v266  pinned v271  behind, and up needs v267 or later; ai-ctl gateway install"
        );
        assert_eq!(
            line(None),
            "llama-swap  -  pinned v271  not installed; ai-ctl gateway install"
        );
        assert_eq!(PINNED.release, Release::parse("v271").unwrap());
    }

    #[test]
    fn outdated_exits_0_when_each_project_is_current_whatever_llama_swap_is() {
        let current = || vec![found("whisper.cpp", "v1.9.5", "v1.9.5", Standing::Current)];
        assert_eq!(outdated_status(&outdated(current(), Some("v271"))), 0);
        assert_eq!(outdated_status(&outdated(current(), Some("v260"))), 0);
        assert_eq!(outdated_status(&outdated(current(), None)), 0);
        assert_eq!(outdated_status(&outdated(vec![], None)), 0);
    }

    #[test]
    fn outdated_exits_4_for_each_standing_update_would_build() {
        for standing in [
            Standing::Behind,
            Standing::NotBuilt,
            Standing::FromCommit,
            Standing::OffRecipe("how".into()),
        ] {
            let projects = vec![
                found("whisper.cpp", "v1.9.5", "v1.9.5", Standing::Current),
                found("llama.cpp", "b1", "b2", standing.clone()),
            ];
            assert_eq!(
                outdated_status(&outdated(projects, Some("v271"))),
                4,
                "{standing:?}"
            );
        }
    }

    #[test]
    fn outdated_exits_1_over_4_when_a_project_could_not_be_read() {
        let projects = vec![
            found("llama.cpp", "b1", "b2", Standing::Behind),
            ProjectStanding {
                project: "whisper.cpp".into(),
                current: None,
                found: Err("no route".into()),
            },
        ];
        assert_eq!(outdated_status(&outdated(projects, Some("v271"))), 1);
    }

    fn updated(project: &str, outcome: UpdateOutcome) -> ProjectUpdate {
        ProjectUpdate {
            project: project.into(),
            outcome,
        }
    }

    fn built_project(project: &str, built: &str, previous: Option<&str>) -> ProjectUpdate {
        updated(
            project,
            UpdateOutcome::Built {
                id: id(built),
                previous: previous.map(id),
            },
        )
    }

    fn summary(projects: Vec<ProjectUpdate>, gateway_runs: bool) -> Updated {
        Updated {
            projects,
            gateway_runs,
        }
    }

    #[test]
    fn the_summary_names_each_project_then_build_use_and_the_restart() {
        let projects = vec![
            built_project("llama.cpp", "b12177", Some("b12111")),
            updated("whisper.cpp", UpdateOutcome::AlreadyCurrent),
            updated(
                "stable-diffusion.cpp",
                UpdateOutcome::Failed("its patch api-key.patch does not apply".into()),
            ),
            built_project("audio.cpp", "v0.9.4", None),
        ];
        assert_eq!(
            render_updated(&summary(projects, true)),
            "llama.cpp             built b12177 (was b12111)\n\
             whisper.cpp           already current\n\
             stable-diffusion.cpp  failed: its patch api-key.patch does not apply\n\
             audio.cpp             built v0.9.4\n\
             The builds replaced stay installed: `ai-ctl build use <project> <id>` goes back to one.\n\
             The running gateway keeps its builds until `ai-ctl down` then `ai-ctl up`.\n"
        );
    }

    #[test]
    fn the_summary_says_nothing_of_a_gateway_that_does_not_run() {
        let projects = vec![built_project("llama.cpp", "b12177", Some("b12111"))];
        assert_eq!(
            render_updated(&summary(projects, false)),
            "llama.cpp  built b12177 (was b12111)\n\
             The builds replaced stay installed: `ai-ctl build use <project> <id>` goes back to one.\n"
        );
    }

    #[test]
    fn a_new_build_replacing_none_needs_no_way_back() {
        let projects = vec![built_project("llama.cpp", "b12177", None)];
        assert_eq!(
            render_updated(&summary(projects, true)),
            "llama.cpp  built b12177\n\
             The running gateway keeps its builds until `ai-ctl down` then `ai-ctl up`.\n"
        );
    }

    #[test]
    fn with_nothing_built_the_summary_has_nothing_to_add() {
        let projects = vec![
            updated("whisper.cpp", UpdateOutcome::AlreadyCurrent),
            updated("llama.cpp", UpdateOutcome::Failed("no route".into())),
        ];
        assert_eq!(
            render_updated(&summary(projects, true)),
            "whisper.cpp  already current\n\
             llama.cpp    failed: no route\n"
        );
    }

    #[test]
    fn update_exits_1_when_a_project_failed_and_0_otherwise() {
        let good = || {
            vec![
                built_project("llama.cpp", "b2", Some("b1")),
                updated("whisper.cpp", UpdateOutcome::AlreadyCurrent),
            ]
        };
        assert_eq!(update_status(&summary(good(), false)), 0);
        assert_eq!(update_status(&summary(vec![], false)), 0);
        let mut failed = good();
        failed.push(updated("audio.cpp", UpdateOutcome::Failed("x".into())));
        assert_eq!(update_status(&summary(failed, false)), 1);
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
                Event::Updating {
                    project: "audio.cpp".into(),
                },
            ],
        );
        assert_eq!(
            text,
            "Fetching the sources\nChecked out b11538 (79e2e74eb1) as build b11538\n\
             Applied api-key.patch\nwarning: cannot remove /work\nUpdating audio.cpp\n"
        );
    }
}
