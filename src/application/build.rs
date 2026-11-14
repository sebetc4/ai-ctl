//! `build`: compiles a project at a git reference into a new installed
//! build, and manages the installed builds (`list`, `use`, `prune`).

use std::fmt::Write as _;
use std::path::PathBuf;

use anyhow::{Context, Result, anyhow, bail, ensure};

use crate::domain::build::{
    Build, BuildId, CommitId, Standing, cmake_flags, newest_first, standing, to_prune,
};
use crate::domain::diagnostics::{Check, Status};
use crate::domain::gateway::{PINNED, Release, SwapStanding, swap_standing};
use crate::domain::gateway_config::starts_program;
use crate::domain::inventory::built_projects;
use crate::domain::recipe::{PatchRefused, RECIPES, Recipe, cuda_devices, recipe};
use crate::domain::timestamp::Timestamp;

use super::doctor::gpu_checks;
use super::gateway_config::required_inventory;
use super::lifecycle::live_record;
use super::ports::{
    BuildStore, Checkout, CompileJob, Compiler, GatewayProcess, GatewayStore, GpuProbe,
    InventoryStore, ProgramOutput, Programs, Progress, Sources, ToolchainProbe, Unreadable,
};
use super::toolchain;

/// The reference that names a project's newest release.
pub const LATEST: &str = "latest";

/// What to build.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Request<'a> {
    pub project: &'a str,
    /// A tag, a branch or a commit; `None` or [`LATEST`] for the newest
    /// release.
    pub reference: Option<&'a str>,
    /// Rebuild a build that is already installed.
    pub force: bool,
    /// Parallel compile jobs; `None` for one per processor.
    pub jobs: Option<usize>,
}

/// A step of a build, for the user to follow.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Event {
    /// Cloning the project's repository, or fetching it.
    Fetching,
    /// `reference` is checked out at `commit`, to be built as `id`.
    CheckedOut {
        reference: String,
        id: BuildId,
        commit: CommitId,
    },
    /// The recipe's patch is applied to the sources.
    Patched {
        patch: &'static str,
    },
    Compile(Progress),
    /// Running the new binary.
    Testing,
    /// `build update` goes on to this project.
    Updating {
        project: String,
    },
    /// Something went wrong that does not fail the build.
    Warning(String),
}

/// A build installed and made current.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Built {
    pub build: Build,
    pub binary: PathBuf,
    /// The build that was current before, if another one.
    pub previous: Option<BuildId>,
}

/// The message of a build the gate refused: each check that refused it,
/// with its remedy.
fn refusal(checks: &[Check]) -> String {
    let mut text = "the build is refused; `ai-ctl doctor` shows every check".to_owned();
    for check in checks {
        let _ = write!(text, "\n  {}: {}", check.name, check.detail);
        for (i, line) in check.remedy.iter().flat_map(|r| r.lines()).enumerate() {
            let label = if i == 0 { "fix:" } else { "    " };
            let _ = write!(text, "\n    {label} {line}");
        }
    }
    text
}

/// The build use case, with the ports it drives.
pub struct Builder<'a> {
    pub gpu: &'a dyn GpuProbe,
    pub toolchain: &'a dyn ToolchainProbe,
    pub sources: &'a dyn Sources,
    pub compiler: &'a dyn Compiler,
    pub store: &'a dyn BuildStore,
    pub gateway: &'a dyn GatewayProcess,
    pub programs: &'a dyn Programs,
    pub clock: &'a dyn Fn() -> Timestamp,
}

impl Builder<'_> {
    /// Builds `request.project` at `request.reference`, smoke-tests the
    /// binary, installs it and makes it current. Nothing is installed when
    /// any step fails.
    pub fn build(&self, request: &Request, events: &mut dyn FnMut(Event)) -> Result<Built> {
        let recipe = known_recipe(request.project)?;
        let _lock = self.store.lock()?;
        let current = current_or_none(
            self.store.current(recipe.project),
            "the build, once installed,",
            &mut |warning| events(Event::Warning(warning)),
        );

        // A release tag names its build: one in place is refused before the
        // gate and the fetch, which the answer needs neither of.
        if let Some(reference) = request.reference
            && recipe.tags.release(reference).is_some()
            && !request.force
        {
            self.taken(recipe, &BuildId::parse(reference)?, current.as_ref(), false)?;
        }

        // The gate: `doctor`'s GPU and Toolchain checks.
        let gpu = self.gpu.gpu();
        let mut refused: Vec<Check> = gpu_checks(&gpu, &[]).into_iter().take(1).collect();
        let inspection = toolchain::inspect(self.toolchain, gpu.found());
        refused.extend(inspection.checks);
        refused.retain(|check| check.status == Status::Fail);
        let toolchain = match inspection.toolchain {
            Some(toolchain) if refused.is_empty() => toolchain,
            _ => bail!(refusal(&refused)),
        };

        events(Event::Fetching);
        self.sources.fetch(recipe)?;
        let reference = match request.reference {
            None | Some(LATEST) => self.latest_release(recipe)?,
            Some(reference) => reference.to_owned(),
        };
        let checkout = self.sources.checkout(recipe, &reference)?;
        let id = match recipe.tags.newest(checkout.tags.iter().map(String::as_str)) {
            Some(tag) => BuildId::parse(tag)?,
            None => BuildId::from_commit(&checkout.commit),
        };
        events(Event::CheckedOut {
            reference,
            id: id.clone(),
            commit: checkout.commit.clone(),
        });

        let taken = self.taken(recipe, &id, current.as_ref(), request.force)?;

        // A build again takes the installed one's place under the same path,
        // which a gateway serving it would run from: refused before the
        // compile, and again under the gateway's lock before the install.
        let replaced = taken.then(|| {
            let binary = self.store.binary(recipe.project, &id, recipe.binary());
            (format!("{} {id}", recipe.project), binary)
        });
        if let Some(replaced) = &replaced {
            refuse_served(self.gateway, std::slice::from_ref(replaced), &REBUILD)?;
        }

        self.apply_patches(recipe, &checkout, &id, events)?;
        let flags = cmake_flags(recipe, &toolchain, &id);
        let job = CompileJob {
            project: recipe.project,
            source: &checkout.dir,
            flags: &flags,
            target: recipe.target,
            artifact: recipe.artifact,
            jobs: request.jobs,
        };
        let compiled = self
            .compiler
            .compile(&job, &mut |progress| events(Event::Compile(progress)))?;

        let build = Build {
            project: recipe.project.to_owned(),
            id: id.clone(),
            commit: checkout.commit,
            toolkit: toolchain.toolkit(),
            host_compiler: toolchain.compiler().clone(),
            cmake_flags: flags,
            patches: recipe
                .patches
                .iter()
                .map(|patch| patch.name.to_owned())
                .collect(),
            built_at: (self.clock)(),
        };
        let staged = self
            .store
            .stage(&build, &compiled.artifact, &compiled.log, recipe.patches)?;
        events(Event::Testing);
        let installed = self.smoke_test(recipe, &staged.binary).and_then(|()| {
            let _gateway_lock = match &replaced {
                Some(replaced) => {
                    let lock = self.gateway.lock("replacing an installed build")?;
                    refuse_served(self.gateway, std::slice::from_ref(replaced), &REBUILD)?;
                    Some(lock)
                }
                None => None,
            };
            self.store.publish(&staged, request.force)
        });
        if let Err(err) = installed {
            if let Err(discard) = self.store.discard(&staged) {
                events(Event::Warning(format!("{discard:#}")));
            }
            return Err(err);
        }
        if let Err(err) = self.compiler.clean(recipe.project) {
            events(Event::Warning(format!("{err:#}")));
        }

        self.store
            .set_current(recipe.project, &id)
            .with_context(|| format!("{} {id} is installed but not current", recipe.project))?;
        Ok(Built {
            binary: self.store.binary(recipe.project, &id, recipe.binary()),
            build,
            previous: current.filter(|previous| *previous != id),
        })
    }

    /// Builds each of `projects` that `build outdated` found something to
    /// build for, at its newest release, one after the other; a failure
    /// stops that project only.
    pub fn update(
        &self,
        projects: &[ProjectStanding],
        jobs: Option<usize>,
        events: &mut dyn FnMut(Event),
    ) -> Updated {
        let projects = projects
            .iter()
            .map(|project| ProjectUpdate {
                project: project.project.clone(),
                outcome: self.update_project(project, jobs, events),
            })
            .collect();
        // A gateway that cannot be told from none is taken for one: the
        // reminder to restart it costs nothing.
        let gateway_runs = live_record(self.gateway).map_or_else(
            |err| {
                events(Event::Warning(format!("{err:#}")));
                true
            },
            |record| record.is_some(),
        );
        Updated {
            projects,
            gateway_runs,
        }
    }

    /// Builds `project`'s newest release, as `build` does, when it has
    /// something to build; a build off its recipe is built again at its own
    /// release as `--force` would, which is never an older one.
    fn update_project(
        &self,
        project: &ProjectStanding,
        jobs: Option<usize>,
        events: &mut dyn FnMut(Event),
    ) -> UpdateOutcome {
        let found = match &project.found {
            Ok(found) => found,
            Err(cause) => {
                return UpdateOutcome::Failed(format!("cannot read its releases: {cause}"));
            }
        };
        if found.standing == Standing::Current {
            return UpdateOutcome::AlreadyCurrent;
        }
        // Off its recipe past the newest release: the remote withdrew its
        // tag, which the fetch drops from the clone, so nothing builds that
        // release again, and update never goes back to an older one.
        if let (Standing::OffRecipe(_), Some(current)) = (&found.standing, &project.current)
            && current.as_str() != found.newest
        {
            return UpdateOutcome::Failed(format!(
                "{current} was withdrawn from the repository and cannot be built again; \
                 `ai-ctl build {}` builds the newest release, {}",
                project.project, found.newest
            ));
        }
        events(Event::Updating {
            project: project.project.clone(),
        });
        let (reference, force) = match (&found.standing, &project.current) {
            (Standing::OffRecipe(_), Some(current)) => (Some(current.as_str()), true),
            _ => (None, false),
        };
        let request = Request {
            project: &project.project,
            reference,
            force,
            jobs,
        };
        match self.build(&request, events) {
            Ok(built) => UpdateOutcome::Built {
                id: built.build.id,
                previous: built.previous,
            },
            Err(err) => UpdateOutcome::Failed(format!("{err:#}")),
        }
    }

    /// Whether the build `id` is installed, or a directory that is not a
    /// readable build has its name; without `force`, either is refused.
    fn taken(
        &self,
        recipe: &Recipe,
        id: &BuildId,
        current: Option<&BuildId>,
        force: bool,
    ) -> Result<bool> {
        let (installed, unreadable) = self.store.builds(recipe.project)?;
        let in_the_way = unreadable.iter().find(|entry| names(entry, id));
        let taken = in_the_way.is_some() || installed.iter().any(|build| build.id == *id);
        if taken && !force {
            let project = recipe.project;
            if let Some(entry) = in_the_way {
                bail!(
                    "{} is not a readable build: {}; `ai-ctl build {project} --ref {id} --force` builds it in its place",
                    entry.dir.display(),
                    entry.reason
                );
            }
            if current == Some(id) {
                bail!(
                    "{project} {id} is already built and current; `ai-ctl build {project} --ref {id} --force` builds it again"
                );
            }
            bail!(
                "{project} {id} is already built; `ai-ctl build use {project} {id}` makes it current, `ai-ctl build {project} --ref {id} --force` builds it again"
            );
        }
        Ok(taken)
    }

    /// The newest release tag of the project's clone.
    fn latest_release(&self, recipe: &Recipe) -> Result<String> {
        newest_release(recipe, &self.sources.release_tags(recipe)?)
    }

    /// Applies the recipe's patches to the checkout, in order; a release
    /// one of them does not apply to cannot be built.
    fn apply_patches(
        &self,
        recipe: &Recipe,
        checkout: &Checkout,
        id: &BuildId,
        events: &mut dyn FnMut(Event),
    ) -> Result<()> {
        for patch in recipe.patches {
            self.sources.apply(checkout, patch).map_err(|err| {
                // Another release may take a patch this one refuses, not one
                // that could not be tried.
                let why = if err.downcast_ref::<PatchRefused>().is_some() {
                    format!(
                        "its patch {} does not apply; `--ref` builds a release it applies to",
                        patch.name
                    )
                } else {
                    format!("cannot apply its patch {}", patch.name)
                };
                err.context(format!("{} {id} cannot be built: {why}", recipe.project))
            })?;
            events(Event::Patched { patch: patch.name });
        }
        Ok(())
    }

    /// The binary runs and prints what its recipe expects, and lists a
    /// CUDA device when the recipe says how.
    fn smoke_test(&self, recipe: &Recipe, binary: &std::path::Path) -> Result<()> {
        let name = recipe.binary();
        let output = self.programs.run(binary, recipe.smoke_test.args)?;
        if let Some(failure) = smoke_test_failure(recipe, &output) {
            bail!("the new {name} {failure}");
        }
        if let Some(listing) = recipe.device_listing {
            let command = format!("{name} {}", listing.args.join(" "));
            let output = self.programs.run(binary, listing.args)?;
            ensure!(
                output.success,
                "the new {name} failed `{command}`: {}",
                summary(&output)
            );
            let devices = format!("{}\n{}", output.stdout, output.stderr);
            ensure!(
                !cuda_devices(&devices).is_empty(),
                "the new {name} lists no CUDA device (`{command}`): {}",
                summary(&output)
            );
        }
        Ok(())
    }
}

/// What `build update` did with one project.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UpdateOutcome {
    /// Built at its newest release and made current; the build it replaces
    /// stays installed.
    Built {
        id: BuildId,
        previous: Option<BuildId>,
    },
    /// Nothing to build.
    AlreadyCurrent,
    /// Why it is left as it was.
    Failed(String),
}

/// One project of `build update`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectUpdate {
    pub project: String,
    pub outcome: UpdateOutcome,
}

/// What `build update` did.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Updated {
    pub projects: Vec<ProjectUpdate>,
    /// A gateway runs, which keeps its builds until `down` then `up`.
    pub gateway_runs: bool,
}

/// The newest release among `tags`.
fn newest_release(recipe: &Recipe, tags: &[String]) -> Result<String> {
    recipe
        .tags
        .newest(tags.iter().map(String::as_str))
        .map(str::to_owned)
        .ok_or_else(|| {
            anyhow!(
                "no release tag of {} matches {}",
                recipe.repository,
                recipe.tags.glob()
            )
        })
}

/// How `build` was not built as `recipe` says now, such as `v1.9.5 was
/// built without reuse-address.patch, which its recipe applies` or `b12111
/// was built without LLAMA_USE_PREBUILT_UI=OFF, which its recipe sets`, or
/// `None` when it was built with each of its patches and each of its CMake
/// options at its value: a patch or an option a later ai-ctl adds reaches
/// only the builds made after it. The options the builder sets for every
/// project are not compared, nor the recipe's release options, nor an option
/// the recipe no longer sets; a patch is matched by name: one rebased under
/// the same name adds the same lines to a newer release, and one whose
/// change differs takes a new name.
pub(super) fn unlike_recipe(recipe: &Recipe, build: &Build) -> Option<String> {
    let mut differences = Vec::new();
    let missing: Vec<&str> = recipe
        .patches
        .iter()
        .map(|patch| patch.name)
        .filter(|name| !build.patches.iter().any(|applied| applied == name))
        .collect();
    if !missing.is_empty() {
        differences.push(format!(
            "without {}, which its recipe applies",
            missing.join(", ")
        ));
    }
    let mut unset: Vec<String> = Vec::new();
    for (name, value) in recipe.cmake_flags {
        match build.cmake_flags.get(*name) {
            Some(built) if built == value => {}
            Some(built) => differences.push(format!(
                "with {name}={built}, where its recipe sets {value}"
            )),
            None => unset.push(format!("{name}={value}")),
        }
    }
    if !unset.is_empty() {
        differences.push(format!(
            "without {}, which its recipe sets",
            unset.join(", ")
        ));
    }
    (!differences.is_empty())
        .then(|| format!("{} was built {}", build.id, differences.join(", and ")))
}

/// What is wrong with the output of the recipe's smoke test, if anything:
/// the binary must exit 0 and print the recipe's text.
pub(super) fn smoke_test_failure(recipe: &Recipe, output: &ProgramOutput) -> Option<String> {
    let test = recipe.smoke_test;
    let command = format!("{} {}", recipe.binary(), test.args.join(" "));
    if !output.success {
        return Some(format!(
            "does not run: `{command}` failed: {}",
            summary(output)
        ));
    }
    let printed = output.stdout.contains(test.prints) || output.stderr.contains(test.prints);
    (!printed).then(|| {
        format!(
            "is not the server its recipe builds: `{command}` printed no `{}`: {}",
            test.prints,
            summary(output)
        )
    })
}

/// The last non-empty lines a program printed, stderr first.
pub(super) fn summary(output: &ProgramOutput) -> String {
    let lines: Vec<&str> = output
        .stderr
        .lines()
        .chain(output.stdout.lines())
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect();
    if lines.is_empty() {
        return "it printed nothing".to_owned();
    }
    lines[lines.len().saturating_sub(5)..].join(" | ")
}

/// A project's current build, looked up among its installed builds.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Current {
    /// No build is current.
    None,
    /// The current build is not installed.
    Missing(BuildId),
    /// The current build's directory is not a readable build.
    Unreadable {
        id: BuildId,
        reason: String,
    },
    Installed(Build),
}

/// How to build `project`'s current build `id` again: at its own release,
/// or at the newest when the project withdrew it, whose tag the fetch drops.
pub(crate) fn rebuild_remedy(project: &str, id: &BuildId) -> String {
    format!(
        "`ai-ctl build {project} --ref {id} --force`, or `ai-ctl build {project}` at the newest \
         release if the project withdrew {id}"
    )
}

impl Current {
    /// The installed build, or the error that tells how to get one.
    pub fn installed(self, project: &str) -> Result<Build> {
        match self {
            Current::Installed(build) => Ok(build),
            Current::None => bail!("{project} has no build; run `ai-ctl build {project}`"),
            Current::Missing(id) => bail!(
                "the current build of {project}, {id}, is not installed; run `ai-ctl build {project}`"
            ),
            Current::Unreadable { id, reason } => bail!(
                "the current build of {project}, {id}, cannot be read: {reason}; run {}",
                rebuild_remedy(project, &id)
            ),
        }
    }
}

/// The current build of `project`.
pub fn current(store: &dyn BuildStore, project: &str) -> Result<Current> {
    let Some(id) = store.current(project)? else {
        return Ok(Current::None);
    };
    let (builds, unreadable) = store.builds(project)?;
    Ok(lookup(id, &builds, &unreadable))
}

/// Build `id` among a project's installed builds and the directories that
/// are not.
fn lookup(id: BuildId, builds: &[Build], unreadable: &[Unreadable]) -> Current {
    if let Some(build) = builds.iter().find(|build| build.id == id) {
        return Current::Installed(build.clone());
    }
    match unreadable.iter().find(|entry| names(entry, &id)) {
        Some(entry) => Current::Unreadable {
            id,
            reason: entry.reason.clone(),
        },
        None => Current::Missing(id),
    }
}

/// Whether `entry` is the directory of build `id`.
fn names(entry: &Unreadable, id: &BuildId) -> bool {
    entry
        .dir
        .file_name()
        .is_some_and(|name| name == id.as_str())
}

pub(super) fn known_recipe(project: &str) -> Result<&'static Recipe> {
    recipe(project).ok_or_else(|| {
        let known: Vec<&str> = RECIPES.iter().map(|recipe| recipe.project).collect();
        anyhow!(
            "no recipe for `{project}`; ai-ctl builds {}",
            known.join(", ")
        )
    })
}

/// The recipe of `project`, or every recipe.
fn recipes(project: Option<&str>) -> Result<Vec<&'static Recipe>> {
    Ok(match project {
        Some(project) => vec![known_recipe(project)?],
        None => RECIPES.iter().collect(),
    })
}

/// One project's installed builds.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Listing {
    pub project: &'static str,
    /// Newest first.
    pub builds: Vec<Build>,
    pub current: Option<BuildId>,
    pub unreadable: Vec<Unreadable>,
}

/// The installed builds of `project`, or of every project with a recipe.
pub fn list(
    store: &dyn BuildStore,
    project: Option<&str>,
    warn: &mut dyn FnMut(String),
) -> Result<Vec<Listing>> {
    recipes(project)?
        .into_iter()
        .map(|recipe| {
            let (mut builds, unreadable) = store.builds(recipe.project)?;
            builds.sort_by(newest_first);
            let replaced_by = "`ai-ctl build use` or `ai-ctl build`";
            let current = current_or_none(store.current(recipe.project), replaced_by, warn);
            if let Some(id) = &current
                && let Err(err) = lookup(id.clone(), &builds, &unreadable).installed(recipe.project)
            {
                warn(format!("{err:#}"));
            }
            Ok(Listing {
                project: recipe.project,
                builds,
                current,
                unreadable,
            })
        })
        .collect()
}

/// Makes an installed build current; returns the build that was.
pub fn use_build(
    store: &dyn BuildStore,
    project: &str,
    id: &str,
    warn: &mut dyn FnMut(String),
) -> Result<Option<BuildId>> {
    let recipe = known_recipe(project)?;
    let id = BuildId::parse(id)?;
    let _lock = store.lock()?;
    let (builds, _) = store.builds(recipe.project)?;
    ensure!(
        builds.iter().any(|build| build.id == id),
        "{} has no build {id}; `ai-ctl build list {}` lists them",
        recipe.project,
        recipe.project
    );
    let previous = current_or_none(store.current(recipe.project), "`build use`", warn);
    store.set_current(recipe.project, &id)?;
    Ok(previous.filter(|previous| *previous != id))
}

/// What `read` gives of a `current` link, a build's or a release's. A link
/// that names nothing reads as none, `warn` told why and that `replaced_by`
/// replaces it; only commands that set the link read it so.
pub(super) fn current_or_none<T>(
    read: Result<Option<T>>,
    replaced_by: &str,
    warn: &mut dyn FnMut(String),
) -> Option<T> {
    read.unwrap_or_else(|err| {
        warn(format!("{err:#}; {replaced_by} replaces it"));
        None
    })
}

/// What `prune` did to one project.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pruned {
    pub project: &'static str,
    pub removed: Vec<BuildId>,
    pub kept: usize,
    /// Directories prune leaves alone, since they are not readable builds.
    pub unreadable: Vec<Unreadable>,
}

/// Removes every build but the `keep` newest and the current one, for
/// `project` or for every project with a recipe. Removes nothing when the
/// running gateway serves a build it would remove.
pub fn prune(
    store: &dyn BuildStore,
    gateway: &dyn GatewayProcess,
    project: Option<&str>,
    keep: usize,
) -> Result<Vec<Pruned>> {
    let recipes = recipes(project)?;
    let _lock = store.lock()?;
    // Held so that no gateway starts on a build being removed.
    let _gateway_lock = gateway.lock("removing builds")?;
    let mut pruned = Vec::new();
    let mut doomed_binaries = Vec::new();
    for recipe in recipes {
        let (builds, unreadable) = store.builds(recipe.project)?;
        let current = store.current(recipe.project)?;
        let doomed: Vec<BuildId> = to_prune(&builds, current.as_ref(), keep)
            .into_iter()
            .map(|build| build.id.clone())
            .collect();
        doomed_binaries.extend(doomed.iter().map(|id| {
            let binary = store.binary(recipe.project, id, recipe.binary());
            (format!("{} {id}", recipe.project), binary)
        }));
        pruned.push(Pruned {
            project: recipe.project,
            kept: builds.len() - doomed.len(),
            removed: doomed,
            unreadable,
        });
    }
    if !doomed_binaries.is_empty() {
        refuse_served(gateway, &doomed_binaries, &PRUNE)?;
    }
    // A failure stops the prune; its error names what is gone already.
    let mut removed: Vec<String> = Vec::new();
    for project in &pruned {
        for id in &project.removed {
            store.remove(project.project, id).with_context(|| {
                let build = format!("cannot remove {} {id}", project.project);
                if removed.is_empty() {
                    build
                } else {
                    format!("{build}, after removing {}", removed.join(", "))
                }
            })?;
            removed.push(format!("{} {id}", project.project));
        }
    }
    Ok(pruned)
}

/// A project's newest release, and where its current build stands against it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Found {
    pub newest: String,
    pub standing: Standing,
}

/// One project of `build outdated`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectStanding {
    pub project: String,
    /// The current build, when one is installed.
    pub current: Option<BuildId>,
    /// What was found, or why it cannot be told.
    pub found: Result<Found, String>,
}

/// What `build outdated` found.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Outdated {
    pub projects: Vec<ProjectStanding>,
    /// llama-swap's current release, when one is, and where it stands.
    pub swap: Option<(Release, SwapStanding)>,
    /// The release this ai-ctl pins, the only one llama-swap is compared with.
    pub pinned: Release,
}

/// Sets each project the inventory builds, or `project` alone, beside its
/// newest release, and llama-swap beside the release this ai-ctl pins. Reads
/// and changes nothing else, and takes no lock.
pub fn outdated(
    sources: &dyn Sources,
    store: &dyn BuildStore,
    inventory: &dyn InventoryStore,
    releases: &dyn GatewayStore,
    project: Option<&str>,
    warn: &mut dyn FnMut(String),
) -> Result<Outdated> {
    let (recipes, no_recipe) = match project {
        Some(project) => (vec![known_recipe(project)?], Vec::new()),
        None => inventory_projects(inventory)?,
    };
    let mut projects: Vec<ProjectStanding> = recipes
        .into_iter()
        .map(|recipe| project_standing(sources, store, recipe, warn))
        .collect();
    projects.extend(no_recipe.into_iter().map(|project| ProjectStanding {
        project,
        current: None,
        found: Err("ai-ctl has no recipe for it".to_owned()),
    }));
    Ok(Outdated {
        projects,
        swap: current_or_none(releases.current(), "`ai-ctl gateway install`", warn)
            .map(|release| (release, swap_standing(release))),
        pinned: PINNED.release,
    })
}

/// The recipes of the projects the inventory's backends build, in the
/// recipes' order, and the projects they build that have none. Every
/// backend counts, a model using it or not, where doctor counts those of
/// the models alone, as `up` runs them: `build update` keeps each up to date.
fn inventory_projects(
    inventory: &dyn InventoryStore,
) -> Result<(Vec<&'static Recipe>, Vec<String>)> {
    let inventory = required_inventory(inventory)?;
    let built = built_projects(inventory.backends());
    let recipes = RECIPES
        .iter()
        .filter(|recipe| built.contains(recipe.project))
        .collect();
    let no_recipe = built
        .into_iter()
        .filter(|project| recipe(project).is_none())
        .map(str::to_owned)
        .collect();
    Ok((recipes, no_recipe))
}

/// `recipe`'s current build beside the newest release of its remote. A
/// current build that cannot be read counts as none, `warn` told why: a build
/// replaces it.
fn project_standing(
    sources: &dyn Sources,
    store: &dyn BuildStore,
    recipe: &'static Recipe,
    warn: &mut dyn FnMut(String),
) -> ProjectStanding {
    let build = current(store, recipe.project)
        .and_then(|current| match current {
            Current::None => Ok(None),
            current => current.installed(recipe.project).map(Some),
        })
        .unwrap_or_else(|err| {
            warn(format!("{err:#}"));
            None
        });
    let found = sources
        .remote_tags(recipe)
        .and_then(|tags| newest_release(recipe, &tags))
        .map(|newest| Found {
            standing: standing(
                recipe.tags,
                build.as_ref(),
                &newest,
                build
                    .as_ref()
                    .and_then(|build| unlike_recipe(recipe, build)),
            ),
            newest,
        })
        .map_err(|err| format!("{err:#}"));
    ProjectStanding {
        project: recipe.project.to_owned(),
        current: build.map(|build| build.id),
        found,
    }
}

/// A command that takes installed builds away, as its refusal names it.
struct Taker {
    command: &'static str,
    /// What it does to a build.
    verb: &'static str,
    /// What the refusal leaves, and how to go on.
    remedy: &'static str,
}

const PRUNE: Taker = Taker {
    command: "prune",
    verb: "remove",
    remedy: "nothing is removed. `ai-ctl down` then `ai-ctl up` restarts the gateway on the \
             current build",
};

const REBUILD: Taker = Taker {
    command: "`build --force`",
    verb: "replace",
    remedy: "nothing is installed. `ai-ctl down` stops the gateway; build again, then \
             `ai-ctl up` starts it on the new build",
};

/// Fails when the running gateway serves one of `doomed`, each a build's
/// name and binary, or when it runs without its configuration.
fn refuse_served(
    gateway: &dyn GatewayProcess,
    doomed: &[(String, PathBuf)],
    taker: &Taker,
) -> Result<()> {
    let Taker {
        command,
        verb,
        remedy,
    } = taker;
    if live_record(gateway)?.is_none() {
        return Ok(());
    }
    let Some(config) = gateway.config()? else {
        bail!(
            "a gateway runs whose configuration is gone, so {command} cannot tell which builds \
             it serves; {remedy}"
        );
    };
    let served: Vec<&str> = doomed
        .iter()
        .filter(|(_, binary)| starts_program(&config, binary))
        .map(|(name, _)| name.as_str())
        .collect();
    ensure!(
        served.is_empty(),
        "the running gateway serves {}, which {command} would {verb}; {remedy}",
        served.join(", ")
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::any::Any;
    use std::cell::{Cell, RefCell};
    use std::collections::BTreeMap;
    use std::path::Path;

    use super::*;
    use crate::application::fakes::{
        FakeBuilds, FakeGateway, FakeGpu, FakeInventory, FakePrograms, FakeReleases, FakeToolchain,
        b11538, gpu, inventory, inventory_parts, printed, recipe_flags, record,
    };
    use crate::application::gateway_config::Configurator;
    use crate::application::ports::{Compiled, Liveness, Staged};
    use crate::domain::build::CUDA_ARCHITECTURES;
    use crate::domain::inventory::{Backend, BackendKind, BackendSource, Inventory};
    use crate::domain::machine::{Probed, Version};
    use crate::domain::recipe::{
        AUDIO_CPP, LLAMA_CPP, Patch, STABLE_DIFFUSION_CPP, SmokeTest, WHISPER_CPP,
    };

    /// llama.cpp's `v1.0.0`, which its nightly `b12359` and
    /// `v1.0.0-b12359-b1b770d` tag too.
    const V1_0_0: &str = "b1b770d90d94cad9e9ea5b993bdbc383a522fb28";
    const FEATURE: &str = "0123456789abcdef0123456789abcdef01234567";
    /// whisper.cpp's `v1.9.5`.
    const V1_9_5: &str = "d1be6fde0123456789abcdef0123456789abcdef";
    const NOW: i64 = 1_791_568_931;

    /// `tags`, and a `feature` branch; `calls` records each call in turn,
    /// and `patched` each patch applied. The patch `apply_fails` names does
    /// not apply.
    struct FakeSources {
        tags: Vec<&'static str>,
        /// The tags of each project named here, in place of `tags`.
        project_tags: BTreeMap<&'static str, Vec<&'static str>>,
        /// The project whose remote cannot be read.
        remote_fails: Option<&'static str>,
        calls: RefCell<Vec<String>>,
        patched: RefCell<Vec<&'static str>>,
        apply_fails: Option<&'static str>,
        /// The patch whose file cannot be written, so is never tried.
        apply_errs: Option<&'static str>,
    }

    impl FakeSources {
        /// Release tags `v0.9.0` and `v1.0.0`, and a tag that is not one.
        fn new() -> Self {
            Self {
                tags: vec!["v0.9.0", "v1.0.0-b12359-b1b770d", "v1.0.0"],
                project_tags: BTreeMap::new(),
                remote_fails: None,
                calls: RefCell::new(vec![]),
                patched: RefCell::new(vec![]),
                apply_fails: None,
                apply_errs: None,
            }
        }

        fn tags_of(&self, recipe: &Recipe) -> Vec<String> {
            let tags = self.project_tags.get(recipe.project).unwrap_or(&self.tags);
            tags.iter().map(|&tag| tag.to_owned()).collect()
        }
    }

    impl Sources for FakeSources {
        fn fetch(&self, _: &Recipe) -> Result<()> {
            self.calls.borrow_mut().push("fetch".into());
            Ok(())
        }
        fn release_tags(&self, recipe: &Recipe) -> Result<Vec<String>> {
            self.calls.borrow_mut().push("release_tags".into());
            Ok(self.tags_of(recipe))
        }
        fn remote_tags(&self, recipe: &Recipe) -> Result<Vec<String>> {
            self.calls
                .borrow_mut()
                .push(format!("remote_tags {}", recipe.project));
            ensure!(
                self.remote_fails != Some(recipe.project),
                "cannot read the tags of {}: Could not resolve host: github.com",
                recipe.repository
            );
            Ok(self.tags_of(recipe))
        }
        fn checkout(&self, _: &Recipe, reference: &str) -> Result<Checkout> {
            self.calls
                .borrow_mut()
                .push(format!("checkout {reference}"));
            let (commit, tags) = match reference {
                // As `git tag --points-at` lists them, by name.
                "v1.0.0" | "b12359" | "b1b770d" | "b1b770d90d" => (
                    V1_0_0,
                    ["b12359", "v1.0.0", "v1.0.0-b12359-b1b770d"]
                        .map(str::to_owned)
                        .to_vec(),
                ),
                "v1.9.5" => (V1_9_5, vec!["v1.9.5".to_owned()]),
                "feature" => (FEATURE, vec![]),
                tag if self
                    .project_tags
                    .values()
                    .flatten()
                    .any(|known| *known == tag) =>
                {
                    (FEATURE, vec![tag.to_owned()])
                }
                other => bail!("`{other}` is not a tag, a branch or a commit"),
            };
            Ok(Checkout {
                dir: "/data/sources/llama.cpp".into(),
                commit: CommitId::parse(commit)?,
                tags,
            })
        }
        fn apply(&self, _: &Checkout, patch: &Patch) -> Result<()> {
            ensure!(
                self.apply_errs != Some(patch.name),
                "cannot write /data/sources/llama.cpp/.git/ai-ctl-{}: No space left on device \
                 (os error 28)",
                patch.name
            );
            if self.apply_fails == Some(patch.name) {
                return Err(PatchRefused(
                    "error: patch failed: examples/server/main.cpp:157".into(),
                )
                .into());
            }
            self.patched.borrow_mut().push(patch.name);
            Ok(())
        }
    }

    #[derive(Default)]
    struct FakeCompiler {
        compile_fails: bool,
        clean_fails: bool,
        flags: RefCell<Option<BTreeMap<String, String>>>,
        jobs: Cell<Option<usize>>,
        cleaned: Cell<bool>,
    }

    impl Compiler for FakeCompiler {
        fn compile(
            &self,
            job: &CompileJob,
            progress: &mut dyn FnMut(Progress),
        ) -> Result<Compiled> {
            *self.flags.borrow_mut() = Some(job.flags.clone());
            self.jobs.set(job.jobs);
            progress(Progress::Configuring);
            progress(Progress::Building { done: 1, total: 2 });
            if self.compile_fails {
                bail!("cmake --build failed; the last lines of /state/build-llama.cpp.log: error");
            }
            progress(Progress::Building { done: 2, total: 2 });
            Ok(Compiled {
                artifact: Path::new("/data/work/llama.cpp").join(job.artifact),
                log: "/state/logs/build-llama.cpp.log".into(),
            })
        }
        fn clean(&self, _: &str) -> Result<()> {
            ensure!(
                !self.clean_fails,
                "cannot remove /data/work/llama.cpp: Permission denied (os error 13)"
            );
            self.cleaned.set(true);
            Ok(())
        }
    }

    /// Builds in memory, each project's apart; `locked` stands for another
    /// ai-ctl holding the lock, and each `_fails` for a failure of that
    /// method.
    #[derive(Default)]
    struct FakeStore {
        locked: bool,
        /// llama.cpp's `current` link names no build, until it is replaced:
        /// a state that `set_current` clears, not a `current_fails`.
        bad_link: Cell<bool>,
        builds: RefCell<Vec<Build>>,
        /// Directories of llama.cpp's that are not readable builds.
        unreadable: Vec<Unreadable>,
        /// llama.cpp's `current` link.
        current: RefCell<Option<BuildId>>,
        /// The other projects' `current` links.
        other_currents: RefCell<BTreeMap<String, BuildId>>,
        staged: RefCell<Vec<Build>>,
        /// The patches staged with each build, by name.
        staged_patches: RefCell<Vec<Vec<&'static str>>>,
        discarded: Cell<usize>,
        replaced: Cell<bool>,
        publish_fails: bool,
        discard_fails: bool,
        set_current_fails: bool,
        /// The build whose removal fails.
        remove_fails: Option<&'static str>,
    }

    impl BuildStore for FakeStore {
        fn lock(&self) -> Result<Box<dyn Any>> {
            ensure!(!self.locked, "another ai-ctl is changing the builds");
            Ok(Box::new(()))
        }
        fn builds(&self, project: &str) -> Result<(Vec<Build>, Vec<Unreadable>)> {
            let builds = self.builds.borrow();
            let builds = builds.iter().filter(|build| build.project == project);
            let unreadable = if project == "llama.cpp" {
                self.unreadable.clone()
            } else {
                vec![]
            };
            Ok((builds.cloned().collect(), unreadable))
        }
        fn current(&self, project: &str) -> Result<Option<BuildId>> {
            if project != "llama.cpp" {
                return Ok(self.other_currents.borrow().get(project).cloned());
            }
            ensure!(
                !self.bad_link.get(),
                "/data/builds/llama.cpp/current points at ../llama.cpp/b11538, not at a build of its directory"
            );
            Ok(self.current.borrow().clone())
        }
        fn binary(&self, project: &str, id: &BuildId, name: &str) -> PathBuf {
            Path::new("/data/builds")
                .join(project)
                .join(id.as_str())
                .join(name)
        }
        fn stage(&self, build: &Build, _: &Path, _: &Path, patches: &[Patch]) -> Result<Staged> {
            self.staged.borrow_mut().push(build.clone());
            let names = patches.iter().map(|patch| patch.name).collect();
            self.staged_patches.borrow_mut().push(names);
            Ok(Staged {
                dir: "/data/builds/llama.cpp/.staging".into(),
                binary: "/data/builds/llama.cpp/.staging/llama-server".into(),
                build: build.clone(),
            })
        }
        fn publish(&self, staged: &Staged, replace: bool) -> Result<()> {
            ensure!(
                !self.publish_fails,
                "cannot install /data/builds/llama.cpp/{}: Permission denied (os error 13)",
                staged.build.id
            );
            self.replaced.set(replace);
            let mut builds = self.builds.borrow_mut();
            builds.retain(|build| build.id != staged.build.id);
            builds.push(staged.build.clone());
            Ok(())
        }
        fn discard(&self, _: &Staged) -> Result<()> {
            self.discarded.set(self.discarded.get() + 1);
            ensure!(
                !self.discard_fails,
                "cannot remove /data/builds/llama.cpp/.staging: Permission denied (os error 13)"
            );
            Ok(())
        }
        fn set_current(&self, project: &str, id: &BuildId) -> Result<()> {
            ensure!(
                !self.set_current_fails,
                "cannot update /data/builds/{project}/current: Permission denied (os error 13)"
            );
            if project != "llama.cpp" {
                let mut currents = self.other_currents.borrow_mut();
                currents.insert(project.to_owned(), id.clone());
                return Ok(());
            }
            self.bad_link.set(false);
            *self.current.borrow_mut() = Some(id.clone());
            Ok(())
        }
        fn remove(&self, project: &str, id: &BuildId) -> Result<()> {
            ensure!(
                self.remove_fails != Some(id.as_str()),
                "cannot remove /data/builds/{project}/{id}: Permission denied (os error 13)"
            );
            let mut builds = self.builds.borrow_mut();
            builds.retain(|build| build.project != project || &build.id != id);
            Ok(())
        }
        fn contains(&self, _: &Path) -> bool {
            unreachable!("builds never look at processes")
        }
    }

    struct World {
        gpu: FakeGpu,
        toolchain: FakeToolchain,
        sources: FakeSources,
        compiler: FakeCompiler,
        store: FakeStore,
        gateway: FakeGateway,
        programs: FakePrograms,
        /// The project built.
        project: &'static str,
        /// The `--jobs` asked for.
        jobs: Option<usize>,
    }

    impl World {
        fn new() -> Self {
            Self {
                gpu: FakeGpu(Probed::Found(gpu())),
                toolchain: FakeToolchain::healthy(),
                sources: FakeSources::new(),
                compiler: FakeCompiler::default(),
                store: FakeStore::default(),
                gateway: FakeGateway::default(),
                programs: FakePrograms::default(),
                project: "llama.cpp",
                jobs: None,
            }
        }

        fn build(&self, reference: Option<&str>, force: bool) -> (Result<Built>, Vec<Event>) {
            self.build_with(&self.compiler, reference, force)
        }

        /// Builds with `compiler` in place of the world's.
        fn build_with(
            &self,
            compiler: &dyn Compiler,
            reference: Option<&str>,
            force: bool,
        ) -> (Result<Built>, Vec<Event>) {
            let clock = || Timestamp::from_unix(NOW).unwrap();
            let builder = Builder {
                gpu: &self.gpu,
                toolchain: &self.toolchain,
                sources: &self.sources,
                compiler,
                store: &self.store,
                gateway: &self.gateway,
                programs: &self.programs,
                clock: &clock,
            };
            let mut events = Vec::new();
            let request = Request {
                project: self.project,
                reference,
                force,
                jobs: self.jobs,
            };
            let result = builder.build(&request, &mut |event| events.push(event));
            (result, events)
        }

        /// Applies the patches of `recipe` to the world's checkout of
        /// v1.0.0.
        fn apply_patches(&self, recipe: &Recipe, events: &mut Vec<Event>) -> Result<()> {
            let clock = || Timestamp::from_unix(NOW).unwrap();
            let builder = Builder {
                gpu: &self.gpu,
                toolchain: &self.toolchain,
                sources: &self.sources,
                compiler: &self.compiler,
                store: &self.store,
                gateway: &self.gateway,
                programs: &self.programs,
                clock: &clock,
            };
            let checkout = self.sources.checkout(recipe, "v1.0.0")?;
            builder.apply_patches(recipe, &checkout, &id("v1.0.0"), &mut |event| {
                events.push(event)
            })
        }

        /// The smoke test of `recipe` on the world's programs.
        fn smoke_test(&self, recipe: &Recipe) -> Result<()> {
            let clock = || Timestamp::from_unix(NOW).unwrap();
            let builder = Builder {
                gpu: &self.gpu,
                toolchain: &self.toolchain,
                sources: &self.sources,
                compiler: &self.compiler,
                store: &self.store,
                gateway: &self.gateway,
                programs: &self.programs,
                clock: &clock,
            };
            builder.smoke_test(recipe, Path::new("/data/builds/.staging/bin/x"))
        }
    }

    fn installed(id: &str, at: i64) -> Build {
        Build {
            project: "llama.cpp".into(),
            id: BuildId::parse(id).unwrap(),
            commit: CommitId::parse(V1_0_0).unwrap(),
            toolkit: Version::new(13, 4, 92),
            host_compiler: crate::application::fakes::gxx("g++", Version::new(16, 2, 1)),
            cmake_flags: crate::application::fakes::recipe_flags(&LLAMA_CPP),
            patches: vec![],
            built_at: Timestamp::from_unix(at).unwrap(),
        }
    }

    fn id(text: &str) -> BuildId {
        BuildId::parse(text).unwrap()
    }

    fn message(result: Result<impl std::fmt::Debug>) -> String {
        format!("{:#}", result.unwrap_err())
    }

    /// The first line of a refused build.
    const REFUSED: &str = "the build is refused; `ai-ctl doctor` shows every check";

    #[test]
    fn latest_builds_the_newest_release_and_makes_it_current() {
        let world = World::new();
        *world.store.current.borrow_mut() = Some(id("b11400"));
        let (result, events) = world.build(None, false);
        let built = result.unwrap();

        assert_eq!(
            *world.sources.calls.borrow(),
            ["fetch", "release_tags", "checkout v1.0.0"]
        );
        assert_eq!(built.build.id, id("v1.0.0"));
        assert_eq!(built.build.commit.as_str(), V1_0_0);
        assert_eq!(built.build.toolkit, Version::new(13, 4, 92));
        assert_eq!(built.build.host_compiler.command, "g++-15");
        assert_eq!(built.build.built_at, Timestamp::from_unix(NOW).unwrap());
        assert_eq!(built.build.cmake_flags[CUDA_ARCHITECTURES], "120a-real");
        assert_eq!(
            built.binary,
            Path::new("/data/builds/llama.cpp/v1.0.0/llama-server")
        );
        assert_eq!(built.previous, Some(id("b11400")));
        assert_eq!(*world.store.current.borrow(), Some(id("v1.0.0")));
        assert!(!world.store.replaced.get(), "no build to replace");
        assert!(world.compiler.cleaned.get());
        assert_eq!(
            world.compiler.flags.borrow().as_ref().unwrap()["CMAKE_CUDA_HOST_COMPILER"],
            "/usr/bin/g++-15"
        );
        assert_eq!(world.compiler.jobs.get(), None);
        assert_eq!(
            events,
            [
                Event::Fetching,
                Event::CheckedOut {
                    reference: "v1.0.0".into(),
                    id: id("v1.0.0"),
                    commit: CommitId::parse(V1_0_0).unwrap()
                },
                Event::Compile(Progress::Configuring),
                Event::Compile(Progress::Building { done: 1, total: 2 }),
                Event::Compile(Progress::Building { done: 2, total: 2 }),
                Event::Testing,
            ]
        );
    }

    #[test]
    fn the_jobs_asked_for_reach_cmake() {
        let mut world = World::new();
        world.jobs = Some(4);
        world.build(None, false).0.unwrap();
        assert_eq!(world.compiler.jobs.get(), Some(4));
    }

    #[test]
    fn latest_can_be_named() {
        let world = World::new();
        world.build(Some(LATEST), false).0.unwrap();
        assert_eq!(
            *world.sources.calls.borrow(),
            ["fetch", "release_tags", "checkout v1.0.0"]
        );
    }

    #[test]
    fn a_clone_without_a_release_tag_fails_with_the_pattern() {
        let mut world = World::new();
        world.sources.tags = vec!["v1.0.0-b12359-b1b770d", "master-old"];
        let text = message(world.build(None, false).0);
        assert_eq!(
            text,
            "no release tag of https://github.com/ggml-org/llama.cpp.git matches v[0-9]*"
        );
    }

    #[test]
    fn a_commit_of_a_release_is_named_after_the_release() {
        let world = World::new();
        let built = world.build(Some("b1b770d"), false).0.unwrap();
        assert_eq!(built.build.id, id("v1.0.0"));
    }

    #[test]
    fn a_commit_both_forms_tag_is_built_under_its_version() {
        for reference in ["v1.0.0", "b12359"] {
            let world = World::new();
            let built = world.build(Some(reference), false).0.unwrap();
            assert_eq!(built.build.id, id("v1.0.0"), "{reference}");
            assert_eq!(built.build.commit.as_str(), V1_0_0, "{reference}");
            assert_eq!(
                *world.store.current.borrow(),
                Some(id("v1.0.0")),
                "{reference}"
            );
        }
    }

    #[test]
    fn a_release_of_llama_cpp_is_built_to_print_its_version_and_a_commit_is_not() {
        let world = World::new();
        let built = world.build(Some("b12359"), false).0.unwrap();
        let flags = world.compiler.flags.borrow().clone().unwrap();
        assert_eq!(
            flags.get("LLAMA_BUILD_IS_DEV").map(String::as_str),
            Some("OFF")
        );
        assert_eq!(built.build.cmake_flags, flags);
        // A commit no release tags is a development build, as llama.cpp's
        // own nightlies are.
        let world = World::new();
        world.build(Some("feature"), false).0.unwrap();
        let flags = world.compiler.flags.borrow().clone().unwrap();
        assert_eq!(flags.get("LLAMA_BUILD_IS_DEV"), None);
    }

    #[test]
    fn a_commit_without_a_release_is_named_after_its_hash() {
        let world = World::new();
        let (result, events) = world.build(Some("feature"), false);
        assert_eq!(result.unwrap().build.id, id("0123456789"));
        assert_eq!(*world.sources.calls.borrow(), ["fetch", "checkout feature"]);
        assert_eq!(
            events[..2],
            [
                Event::Fetching,
                Event::CheckedOut {
                    reference: "feature".into(),
                    id: id("0123456789"),
                    commit: CommitId::parse(FEATURE).unwrap()
                }
            ]
        );
    }

    #[test]
    fn cuda_13_2_refuses_the_build_with_doctors_remedy() {
        let mut world = World::new();
        world.toolchain = FakeToolchain::before_the_upgrade();
        let (result, events) = world.build(None, false);
        let text = message(result);
        // One check refused, each check's line indented by two spaces.
        let checks: Vec<&str> = text
            .lines()
            .filter(|line| line.starts_with("  ") && !line.starts_with("   "))
            .collect();
        assert_eq!(
            checks,
            [
                "  CUDA toolkit: 13.2.86 (/usr/local/cuda-13.2/bin/nvcc): CUDA 13.2 is reported to \
                 produce wrong output in llama.cpp builds (llama.cpp issue #21255)"
            ],
            "{text}"
        );
        assert!(text.starts_with(REFUSED), "{text}");
        assert!(
            text.contains("fix: 1. sudo dnf config-manager addrepo"),
            "{text}"
        );
        assert!(
            text.contains("3. sudo dnf install cuda-toolkit-13-4"),
            "{text}"
        );
        assert!(
            !text.contains("Toolkit packages"),
            "warnings do not refuse: {text}"
        );
        assert!(events.is_empty());
        assert!(world.sources.calls.borrow().is_empty());
    }

    #[test]
    fn a_machine_without_the_driver_refuses_the_build() {
        let mut world = World::new();
        world.gpu = FakeGpu(Probed::Missing);
        let text = message(world.build(None, false).0);
        assert!(
            text.starts_with(&format!(
                "{REFUSED}\n  GPU: NVML not found: the NVIDIA driver is not installed\n"
            )),
            "{text}"
        );
    }

    #[test]
    fn a_build_waits_for_no_one() {
        let mut world = World::new();
        world.store.locked = true;
        let text = message(world.build(None, false).0);
        assert!(text.contains("another ai-ctl"), "{text}");
        assert!(world.sources.calls.borrow().is_empty());
    }

    #[test]
    fn an_installed_build_is_not_rebuilt_without_force() {
        let world = World::new();
        world.store.builds.borrow_mut().push(installed("v1.0.0", 1));
        let text = message(world.build(None, false).0);
        assert!(text.contains("already built"), "{text}");
        assert!(text.contains("ai-ctl build use llama.cpp v1.0.0"), "{text}");
        assert!(world.compiler.flags.borrow().is_none(), "nothing compiled");

        *world.store.current.borrow_mut() = Some(id("v1.0.0"));
        let text = message(world.build(None, false).0);
        assert!(
            text.contains(
                "is already built and current; `ai-ctl build llama.cpp --ref v1.0.0 --force`"
            ),
            "{text}"
        );

        let rebuilt = world.build(None, true).0.unwrap();
        assert!(world.store.replaced.get());
        assert_eq!(rebuilt.build.built_at, Timestamp::from_unix(NOW).unwrap());
        assert_eq!(rebuilt.previous, None);
        assert_eq!(world.store.builds.borrow().len(), 1);
    }

    /// The refusal of `--force` while the running gateway serves v1.0.0.
    const SERVED: &str = "the running gateway serves llama.cpp v1.0.0, which `build --force` \
                          would replace; nothing is installed. `ai-ctl down` stops the \
                          gateway; build again, then `ai-ctl up` starts it on the new build";

    /// Each installed build of the world's store, with when it was built.
    fn built_at(world: &World) -> Vec<(String, Timestamp)> {
        let builds = world.store.builds.borrow();
        builds
            .iter()
            .map(|b| (b.id.to_string(), b.built_at))
            .collect()
    }

    fn at(seconds: i64) -> Timestamp {
        Timestamp::from_unix(seconds).unwrap()
    }

    #[test]
    fn force_refuses_to_replace_the_build_the_running_gateway_serves() {
        let mut world = World::new();
        world.store.builds.borrow_mut().push(installed("v1.0.0", 1));
        *world.store.current.borrow_mut() = Some(id("v1.0.0"));
        world.gateway = gateway_on(&world.store, "v1.0.0");
        assert_eq!(message(world.build(None, true).0), SERVED);
        assert!(world.compiler.flags.borrow().is_none(), "nothing compiled");
        assert_eq!(built_at(&world), [("v1.0.0".to_owned(), at(1))]);

        *world.gateway.config.borrow_mut() = None;
        assert_eq!(
            message(world.build(None, true).0),
            "a gateway runs whose configuration is gone, so `build --force` cannot tell which \
             builds it serves; nothing is installed. `ai-ctl down` stops the gateway; build \
             again, then `ai-ctl up` starts it on the new build"
        );
        assert!(world.compiler.flags.borrow().is_none(), "nothing compiled");
    }

    #[test]
    fn force_replaces_a_build_the_running_gateway_does_not_serve() {
        let mut world = World::new();
        world
            .store
            .builds
            .borrow_mut()
            .extend([installed("b11400", 1), installed("v1.0.0", 2)]);
        *world.store.current.borrow_mut() = Some(id("v1.0.0"));
        world.gateway = gateway_on(&world.store, "b11400");
        world.build(None, true).0.unwrap();
        assert!(world.store.replaced.get());
        assert_eq!(
            built_at(&world),
            [("b11400".to_owned(), at(1)), ("v1.0.0".to_owned(), at(NOW))]
        );
    }

    /// Compiles as the world's compiler does, while a gateway starts on
    /// `config`.
    struct StartsGateway<'a> {
        compiler: &'a FakeCompiler,
        gateway: &'a FakeGateway,
        config: String,
    }

    impl Compiler for StartsGateway<'_> {
        fn compile(
            &self,
            job: &CompileJob,
            progress: &mut dyn FnMut(Progress),
        ) -> Result<Compiled> {
            *self.gateway.record.borrow_mut() = Some(record(1234));
            *self.gateway.config.borrow_mut() = Some(self.config.clone());
            self.compiler.compile(job, progress)
        }
        fn clean(&self, project: &str) -> Result<()> {
            self.compiler.clean(project)
        }
    }

    #[test]
    fn force_refuses_to_install_over_a_gateway_started_during_the_compile() {
        let world = World::new();
        world.store.builds.borrow_mut().push(installed("v1.0.0", 1));
        *world.store.current.borrow_mut() = Some(id("v1.0.0"));
        let compiler = StartsGateway {
            compiler: &world.compiler,
            gateway: &world.gateway,
            config: gateway_on(&world.store, "v1.0.0")
                .config
                .into_inner()
                .unwrap(),
        };
        assert_eq!(message(world.build_with(&compiler, None, true).0), SERVED);
        assert_eq!(world.store.discarded.get(), 1);
        assert!(!world.store.replaced.get());
        assert_eq!(built_at(&world), [("v1.0.0".to_owned(), at(1))]);
    }

    #[test]
    fn force_installs_under_the_gateway_lock() {
        let mut world = World::new();
        world.gateway.locked = true;
        // A build that replaces none needs no lock of the gateway's.
        world.build(None, false).0.unwrap();
        assert_eq!(
            message(world.build(None, true).0),
            "another ai-ctl is stopping the gateway"
        );
        assert_eq!(world.store.discarded.get(), 1);
        assert_eq!(built_at(&world), [("v1.0.0".to_owned(), at(NOW))]);
    }

    #[test]
    fn an_installed_release_named_is_refused_before_the_gate_and_the_fetch() {
        let mut world = World::new();
        world.store.builds.borrow_mut().push(installed("v1.0.0", 1));
        world.gpu = FakeGpu(Probed::Missing);
        let (result, events) = world.build(Some("v1.0.0"), false);
        assert_eq!(
            message(result),
            "llama.cpp v1.0.0 is already built; `ai-ctl build use llama.cpp v1.0.0` makes it \
             current, `ai-ctl build llama.cpp --ref v1.0.0 --force` builds it again"
        );
        assert!(events.is_empty(), "{events:?}");
        assert!(world.sources.calls.borrow().is_empty());
        assert!(message(world.build(Some("v1.0.0"), true).0).starts_with(REFUSED));
    }

    #[test]
    fn a_commit_names_its_build_only_once_checked_out() {
        let world = World::new();
        world.store.builds.borrow_mut().push(installed("v1.0.0", 1));
        let text = message(world.build(Some("b1b770d"), false).0);
        assert!(text.contains("llama.cpp v1.0.0 is already built"), "{text}");
        assert_eq!(
            world.sources.calls.borrow().as_slice(),
            ["fetch", "checkout b1b770d"]
        );
        world.build(Some("v1.0.0"), true).0.unwrap();
        assert!(world.store.replaced.get());
    }

    #[test]
    fn a_commit_built_before_its_release_was_tagged_is_built_under_the_tag() {
        let world = World::new();
        world
            .store
            .builds
            .borrow_mut()
            .push(installed("b1b770d90d", 1));
        let built = world.build(Some("b1b770d90d"), false).0.unwrap();
        assert_eq!(built.build.id, id("v1.0.0"));
    }

    #[test]
    fn an_unreadable_directory_of_a_release_named_is_in_the_way_before_the_fetch() {
        let mut world = World::new();
        world.store.unreadable.push(not_a_build("v1.0.0"));
        assert_eq!(
            message(world.build(Some("v1.0.0"), false).0),
            "/data/builds/llama.cpp/v1.0.0 is not a readable build: no build.json; \
             `ai-ctl build llama.cpp --ref v1.0.0 --force` builds it in its place"
        );
        assert!(world.sources.calls.borrow().is_empty());
    }

    #[test]
    fn an_unreadable_directory_of_the_same_name_is_in_the_way() {
        let mut world = World::new();
        world.store.unreadable.push(not_a_build("v1.0.0"));
        let in_the_way = "/data/builds/llama.cpp/v1.0.0 is not a readable build: no build.json; \
                          `ai-ctl build llama.cpp --ref v1.0.0 --force` builds it in its place";
        assert_eq!(message(world.build(None, false).0), in_the_way);
        *world.store.current.borrow_mut() = Some(id("v1.0.0"));
        assert_eq!(message(world.build(None, false).0), in_the_way);
        assert!(world.compiler.flags.borrow().is_none(), "nothing compiled");
        world.build(None, true).0.unwrap();
        assert!(world.store.replaced.get());
    }

    #[test]
    fn a_failed_compilation_installs_nothing() {
        let mut world = World::new();
        world.compiler.compile_fails = true;
        let text = message(world.build(None, false).0);
        assert!(text.contains("build-llama.cpp.log"), "{text}");
        assert!(world.store.staged.borrow().is_empty());
        assert_eq!(*world.store.current.borrow(), None);
    }

    #[test]
    fn a_build_without_a_cuda_device_is_discarded() {
        let mut world = World::new();
        *world.store.current.borrow_mut() = Some(id("b11400"));
        world.programs.devices = printed(true, "Available devices:\n", "");
        let text = message(world.build(None, false).0);
        assert!(text.contains("lists no CUDA device"), "{text}");
        assert_eq!(world.store.discarded.get(), 1);
        assert!(world.store.builds.borrow().is_empty());
        assert_eq!(*world.store.current.borrow(), Some(id("b11400")));
        assert!(
            !world.compiler.cleaned.get(),
            "the build tree stays for inspection"
        );
    }

    #[test]
    fn a_binary_that_does_not_start_is_discarded() {
        let mut world = World::new();
        world.programs.version = printed(
            false,
            "",
            "llama-server: error while loading shared libraries: libcudart.so.13: cannot open shared object file\n",
        );
        let text = message(world.build(None, false).0);
        assert!(text.contains("does not run"), "{text}");
        assert!(text.contains("libcudart.so.13"), "{text}");
        assert_eq!(world.store.discarded.get(), 1);
    }

    /// llama.cpp's recipe, with two patches.
    const PATCHED: Recipe = Recipe {
        patches: &[
            Patch {
                name: "api-key.patch",
                diff: "diff --git a/server.cpp b/server.cpp\n",
            },
            Patch {
                name: "listen-failure.patch",
                diff: "diff --git a/main.cpp b/main.cpp\n",
            },
        ],
        ..LLAMA_CPP
    };

    #[test]
    fn the_recipes_patches_apply_in_order() {
        let world = World::new();
        let mut events = vec![];
        world.apply_patches(&PATCHED, &mut events).unwrap();
        assert_eq!(
            *world.sources.patched.borrow(),
            ["api-key.patch", "listen-failure.patch"]
        );
        assert_eq!(
            events,
            [
                Event::Patched {
                    patch: "api-key.patch"
                },
                Event::Patched {
                    patch: "listen-failure.patch"
                }
            ]
        );
    }

    /// A world building whisper.cpp, whose newest release is `v1.9.5`.
    fn whisper_cpp() -> World {
        let mut world = World::new();
        world.project = "whisper.cpp";
        world.sources.tags = vec!["v1.9.4", "b2250", "v1.9.5", "v1.7.4-pre-1"];
        world
    }

    #[test]
    fn a_build_applies_its_recipes_patches_and_keeps_them() {
        let world = whisper_cpp();
        let (built, events) = world.build(None, false);
        let built = built.unwrap();
        let patches = ["api-key.patch", "reuse-address.patch"];
        assert_eq!(built.build.id, id("v1.9.5"));
        assert_eq!(built.build.patches, patches);
        assert_eq!(*world.sources.patched.borrow(), patches);
        assert_eq!(*world.store.staged_patches.borrow(), [patches]);
        let patched = events.iter().position(|event| {
            *event
                == Event::Patched {
                    patch: "reuse-address.patch",
                }
        });
        let configured = events
            .iter()
            .position(|event| *event == Event::Compile(Progress::Configuring));
        assert!(patched.unwrap() < configured.unwrap(), "{events:?}");
    }

    #[test]
    fn a_release_its_patch_does_not_apply_to_is_not_compiled() {
        let mut world = whisper_cpp();
        world.sources.apply_fails = Some("api-key.patch");
        assert_eq!(
            message(world.build(None, false).0),
            "whisper.cpp v1.9.5 cannot be built: its patch api-key.patch does not apply; \
             `--ref` builds a release it applies to: \
             error: patch failed: examples/server/main.cpp:157"
        );
        assert!(world.compiler.flags.borrow().is_none(), "nothing compiled");
        assert!(world.store.staged.borrow().is_empty());
    }

    #[test]
    fn a_patch_that_cannot_be_tried_sends_to_no_other_release() {
        let mut world = whisper_cpp();
        world.sources.apply_errs = Some("api-key.patch");
        assert_eq!(
            message(world.build(None, false).0),
            "whisper.cpp v1.9.5 cannot be built: cannot apply its patch api-key.patch: \
             cannot write /data/sources/llama.cpp/.git/ai-ctl-api-key.patch: No space left on \
             device (os error 28)"
        );
        assert!(world.compiler.flags.borrow().is_none(), "nothing compiled");
    }

    #[test]
    fn a_release_a_patch_does_not_apply_to_cannot_be_built() {
        let mut world = World::new();
        world.sources.apply_fails = Some("listen-failure.patch");
        let mut events = vec![];
        assert_eq!(
            message(world.apply_patches(&PATCHED, &mut events)),
            "llama.cpp v1.0.0 cannot be built: its patch listen-failure.patch does not apply; \
             `--ref` builds a release it applies to: \
             error: patch failed: examples/server/main.cpp:157"
        );
        assert_eq!(*world.sources.patched.borrow(), ["api-key.patch"]);
        assert_eq!(
            events,
            [Event::Patched {
                patch: "api-key.patch"
            }]
        );
    }

    #[test]
    fn a_binary_that_does_not_print_what_its_recipe_expects_is_discarded() {
        let mut world = World::new();
        world.programs.version = printed(true, "", "error: unknown argument: --version\n");
        assert_eq!(
            message(world.build(None, false).0),
            "the new llama-server is not the server its recipe builds: `llama-server --version` \
             printed no `version: `: error: unknown argument: --version"
        );
        assert_eq!(world.store.discarded.get(), 1);
        assert!(world.store.builds.borrow().is_empty());
    }

    #[test]
    fn a_build_lacks_each_patch_its_recipe_applies_now() {
        let sd_cpp = recipe("stable-diffusion.cpp").unwrap();
        let built = |patches: &[&str]| Build {
            project: "stable-diffusion.cpp".into(),
            cmake_flags: crate::application::fakes::recipe_flags(
                &crate::domain::recipe::STABLE_DIFFUSION_CPP,
            ),
            patches: patches.iter().map(|&name| name.to_owned()).collect(),
            ..b11538()
        };
        let all = [
            "api-key.patch",
            "listen-failure.patch",
            "reuse-address.patch",
        ];
        assert_eq!(unlike_recipe(sd_cpp, &built(&all)), None);
        assert_eq!(
            unlike_recipe(sd_cpp, &built(&["old.patch", "api-key.patch"])).as_deref(),
            Some(
                "b11538 was built without listen-failure.patch, reuse-address.patch, which its \
                 recipe applies"
            )
        );
        // A recipe with no patch, as llama.cpp's, and a build of format 1.
        assert_eq!(unlike_recipe(&LLAMA_CPP, &b11538()), None);
    }

    #[test]
    fn a_build_lacks_each_cmake_option_its_recipe_sets_now() {
        // b11538 as an ai-ctl before LLAMA_USE_PREBUILT_UI=OFF built it:
        // upstream's default, ON, downloaded the bucket's web UI.
        let built = |change: &dyn Fn(&mut BTreeMap<String, String>)| {
            let mut flags: BTreeMap<String, String> = LLAMA_CPP
                .cmake_flags
                .iter()
                .map(|(name, value)| ((*name).to_owned(), (*value).to_owned()))
                .collect();
            // The builder's own, which follow the toolchain.
            flags.insert("GGML_CUDA".into(), "ON".into());
            flags.insert("CMAKE_CXX_COMPILER".into(), "/usr/bin/g++-15".into());
            change(&mut flags);
            Build {
                cmake_flags: flags,
                ..b11538()
            }
        };
        assert_eq!(unlike_recipe(&LLAMA_CPP, &built(&|_| {})), None);
        assert_eq!(
            unlike_recipe(
                &LLAMA_CPP,
                &built(&|flags| {
                    flags.remove("LLAMA_USE_PREBUILT_UI");
                })
            )
            .as_deref(),
            Some("b11538 was built without LLAMA_USE_PREBUILT_UI=OFF, which its recipe sets")
        );
        assert_eq!(
            unlike_recipe(
                &LLAMA_CPP,
                &built(&|flags| {
                    flags.insert("LLAMA_USE_PREBUILT_UI".into(), "ON".into());
                })
            )
            .as_deref(),
            Some("b11538 was built with LLAMA_USE_PREBUILT_UI=ON, where its recipe sets OFF")
        );
        // An option the recipe no longer sets is left alone.
        assert_eq!(
            unlike_recipe(
                &LLAMA_CPP,
                &built(&|flags| {
                    flags.insert("LLAMA_CURL".into(), "ON".into());
                })
            ),
            None
        );
        // Several options unset, and one at another value besides.
        assert_eq!(
            unlike_recipe(
                &LLAMA_CPP,
                &built(&|flags| {
                    flags.insert("LLAMA_BUILD_TESTS".into(), "ON".into());
                    flags.remove("LLAMA_BUILD_APP");
                    flags.remove("LLAMA_USE_PREBUILT_UI");
                })
            )
            .as_deref(),
            Some(
                "b11538 was built with LLAMA_BUILD_TESTS=ON, where its recipe sets OFF, and \
                 without LLAMA_BUILD_APP=OFF, LLAMA_USE_PREBUILT_UI=OFF, which its recipe sets"
            )
        );
        // No option recorded at all: each of the recipe's is unset.
        assert_eq!(
            unlike_recipe(
                &LLAMA_CPP,
                &Build {
                    cmake_flags: BTreeMap::new(),
                    ..b11538()
                }
            )
            .as_deref(),
            Some(
                "b11538 was built without LLAMA_BUILD_TOOLS=ON, LLAMA_BUILD_SERVER=ON, \
                 LLAMA_BUILD_TESTS=OFF, LLAMA_BUILD_EXAMPLES=OFF, LLAMA_BUILD_APP=OFF, \
                 LLAMA_USE_PREBUILT_UI=OFF, which its recipe sets"
            )
        );
    }

    #[test]
    fn a_build_lacking_a_patch_and_an_option_names_both() {
        let sd_cpp = recipe("stable-diffusion.cpp").unwrap();
        let mut flags = crate::application::fakes::recipe_flags(sd_cpp);
        flags.remove("SD_SERVER_BUILD_FRONTEND");
        let build = Build {
            project: "stable-diffusion.cpp".into(),
            cmake_flags: flags,
            patches: vec!["api-key.patch".into(), "reuse-address.patch".into()],
            ..b11538()
        };
        assert_eq!(
            unlike_recipe(sd_cpp, &build).as_deref(),
            Some(
                "b11538 was built without listen-failure.patch, which its recipe applies, and \
                 without SD_SERVER_BUILD_FRONTEND=OFF, which its recipe sets"
            )
        );
    }

    #[test]
    fn a_server_without_a_version_or_a_device_listing_is_tested_on_what_it_prints() {
        // whisper-server: no `--version`, an unknown argument exits 0, and
        // no device listing.
        const WHISPER_SERVER: Recipe = Recipe {
            artifact: "bin/whisper-server",
            smoke_test: SmokeTest {
                args: &["--help"],
                prints: "--inference-path",
            },
            device_listing: None,
            ..LLAMA_CPP
        };
        let mut world = World::new();
        // A device listing would fail.
        world.programs.devices.success = false;
        world.smoke_test(&WHISPER_SERVER).unwrap();

        world.programs.help = printed(true, "", "");
        assert_eq!(
            message(world.smoke_test(&WHISPER_SERVER)),
            "the new whisper-server is not the server its recipe builds: `whisper-server --help` \
             printed no `--inference-path`: it printed nothing"
        );
        world.programs.help.success = false;
        assert_eq!(
            message(world.smoke_test(&WHISPER_SERVER)),
            "the new whisper-server does not run: `whisper-server --help` failed: it printed nothing"
        );
    }

    #[test]
    fn a_server_that_prints_on_stdout_passes_its_smoke_test() {
        // sd-server and audiocpp_server print their version on stdout.
        let sd_server = printed(
            true,
            include_str!("../../tests/fixtures/sd-server-master-975-version.txt"),
            "",
        );
        assert_eq!(
            smoke_test_failure(&crate::domain::recipe::STABLE_DIFFUSION_CPP, &sd_server),
            None
        );
        let audiocpp_server = printed(
            true,
            include_str!("../../tests/fixtures/audiocpp_server-v0.9.2-version.txt"),
            "",
        );
        assert_eq!(
            smoke_test_failure(&crate::domain::recipe::AUDIO_CPP, &audiocpp_server),
            None
        );
    }

    #[test]
    fn the_smoke_test_runs_the_staged_binary_not_the_installed_one() {
        let world = World::new();
        world.store.builds.borrow_mut().push(installed("v1.0.0", 1));
        world.build(None, true).0.unwrap();
        let staged = PathBuf::from("/data/builds/llama.cpp/.staging/llama-server");
        assert_eq!(*world.programs.ran.borrow(), [staged.clone(), staged]);
    }

    #[test]
    fn a_build_whose_device_listing_fails_is_discarded() {
        let mut world = World::new();
        // The listing v1.0.0 prints, then a crash.
        world.programs.devices.success = false;
        let text = message(world.build(None, false).0);
        assert_eq!(
            text,
            "the new llama-server failed `llama-server --list-devices`: Available devices: | \
             CUDA0: NVIDIA GeForce RTX 5090 Laptop GPU (24075 MiB, 23798 MiB free)"
        );
        assert_eq!(world.store.discarded.get(), 1);
        assert!(world.store.builds.borrow().is_empty());
    }

    #[test]
    fn a_build_that_cannot_be_installed_or_discarded_warns_of_the_discard() {
        let mut world = World::new();
        *world.store.current.borrow_mut() = Some(id("b11400"));
        world.store.publish_fails = true;
        world.store.discard_fails = true;
        let (result, events) = world.build(None, false);
        assert_eq!(
            message(result),
            "cannot install /data/builds/llama.cpp/v1.0.0: Permission denied (os error 13)"
        );
        assert_eq!(
            events.last(),
            Some(&Event::Warning(
                "cannot remove /data/builds/llama.cpp/.staging: Permission denied (os error 13)"
                    .into()
            ))
        );
        assert_eq!(world.store.discarded.get(), 1);
        assert_eq!(*world.store.current.borrow(), Some(id("b11400")));
        assert!(!world.compiler.cleaned.get());
    }

    #[test]
    fn a_build_tree_that_cannot_be_removed_is_a_warning() {
        let mut world = World::new();
        world.compiler.clean_fails = true;
        let (result, events) = world.build(None, false);
        assert_eq!(result.unwrap().build.id, id("v1.0.0"));
        assert_eq!(*world.store.current.borrow(), Some(id("v1.0.0")));
        assert_eq!(
            events.last(),
            Some(&Event::Warning(
                "cannot remove /data/work/llama.cpp: Permission denied (os error 13)".into()
            ))
        );
    }

    #[test]
    fn a_build_that_cannot_be_made_current_says_it_is_installed() {
        let mut world = World::new();
        *world.store.current.borrow_mut() = Some(id("b11400"));
        world.store.set_current_fails = true;
        assert_eq!(
            message(world.build(None, false).0),
            "llama.cpp v1.0.0 is installed but not current: cannot update \
             /data/builds/llama.cpp/current: Permission denied (os error 13)"
        );
        assert_eq!(left(&world.store), ["v1.0.0"]);
        assert_eq!(*world.store.current.borrow(), Some(id("b11400")));
    }

    #[test]
    fn a_summary_keeps_the_last_five_lines_printed_stderr_first() {
        let output = printed(false, "o1\n\n  o2  \no3\n", "e1\ne2\ne3\ne4\n");
        assert_eq!(summary(&output), "e3 | e4 | o1 | o2 | o3");
        assert_eq!(summary(&printed(false, " \n", "")), "it printed nothing");
    }

    #[test]
    fn an_unknown_project_or_reference_is_refused() {
        let world = World::new();
        let clock = || Timestamp::from_unix(NOW).unwrap();
        let builder = Builder {
            gpu: &world.gpu,
            toolchain: &world.toolchain,
            sources: &world.sources,
            compiler: &world.compiler,
            store: &world.store,
            gateway: &world.gateway,
            programs: &world.programs,
            clock: &clock,
        };
        let request = Request {
            project: "nope.cpp",
            reference: None,
            force: false,
            jobs: None,
        };
        let text = message(builder.build(&request, &mut |_| {}));
        assert_eq!(
            text,
            "no recipe for `nope.cpp`; ai-ctl builds llama.cpp, whisper.cpp, stable-diffusion.cpp, \
             audio.cpp"
        );

        assert!(message(world.build(Some("b1"), false).0).contains("is not a tag"));
    }

    #[test]
    fn the_refusal_lists_each_check_with_its_remedy() {
        let refused = refusal(&[
            Check::fail(
                "CMake",
                "cmake not found",
                Some("sudo dnf install cmake".into()),
            ),
            Check::fail(
                "CUDA toolkit",
                "CUDA 13.2 is reported",
                Some(
                    "1. add NVIDIA's repository\n2. sudo dnf remove cuda-toolkit-13-2\n\
                     3. sudo dnf install cuda-toolkit-13-4"
                        .into(),
                ),
            ),
            Check::fail("Git", "git: broken", None),
        ]);
        assert_eq!(
            refused,
            "the build is refused; `ai-ctl doctor` shows every check\n  CMake: cmake not found\n    fix: sudo dnf install cmake\n  \
             CUDA toolkit: CUDA 13.2 is reported\n    fix: 1. add NVIDIA's repository\n         \
             2. sudo dnf remove cuda-toolkit-13-2\n         3. sudo dnf install cuda-toolkit-13-4\n  \
             Git: git: broken"
        );
    }

    /// `dir` of llama.cpp's builds, which is not a readable build.
    fn not_a_build(dir: &str) -> Unreadable {
        Unreadable {
            dir: Path::new("/data/builds/llama.cpp").join(dir),
            reason: "no build.json".into(),
        }
    }

    #[test]
    fn the_current_build_is_none_missing_unreadable_or_installed() {
        let mut store = FakeBuilds::current_b11538();
        assert_eq!(
            current(&store, "llama.cpp").unwrap(),
            Current::Installed(b11538())
        );
        store.current = Some(id("b9"));
        store.unreadable = vec![not_a_build("b10")];
        assert_eq!(
            current(&store, "llama.cpp").unwrap(),
            Current::Missing(id("b9"))
        );
        store.unreadable.push(not_a_build("b9"));
        assert_eq!(
            current(&store, "llama.cpp").unwrap(),
            Current::Unreadable {
                id: id("b9"),
                reason: "no build.json".into()
            }
        );
        store.current = None;
        assert_eq!(current(&store, "llama.cpp").unwrap(), Current::None);
        store.current_fails = true;
        assert!(message(current(&store, "llama.cpp")).starts_with("cannot read"));
    }

    #[test]
    fn a_current_build_not_installed_tells_how_to_build_one() {
        assert_eq!(
            Current::Installed(b11538()).installed("llama.cpp").unwrap(),
            b11538()
        );
        assert_eq!(
            message(Current::None.installed("llama.cpp")),
            "llama.cpp has no build; run `ai-ctl build llama.cpp`"
        );
        assert_eq!(
            message(Current::Missing(id("b9")).installed("llama.cpp")),
            "the current build of llama.cpp, b9, is not installed; run `ai-ctl build llama.cpp`"
        );
        let unreadable = Current::Unreadable {
            id: id("b9"),
            reason: "no build.json".into(),
        };
        assert_eq!(
            message(unreadable.installed("llama.cpp")),
            "the current build of llama.cpp, b9, cannot be read: no build.json; run `ai-ctl \
             build llama.cpp --ref b9 --force`, or `ai-ctl build llama.cpp` at the newest \
             release if the project withdrew b9"
        );
    }

    #[test]
    fn list_warns_of_a_current_build_it_cannot_show() {
        let mut warnings = Vec::new();
        // The link names a build that is gone, and no build is left.
        let store = FakeStore::default();
        *store.current.borrow_mut() = Some(id("b9"));
        let listings = list(&store, None, &mut |warning| warnings.push(warning)).unwrap();
        assert_eq!(listings[0].current, Some(id("b9")));
        // The link names a directory that is not a readable build.
        let store = FakeStore {
            unreadable: vec![not_a_build("v1.0.0")],
            ..FakeStore::default()
        };
        *store.current.borrow_mut() = Some(id("v1.0.0"));
        list(&store, None, &mut |warning| warnings.push(warning)).unwrap();
        assert_eq!(
            warnings,
            [
                "the current build of llama.cpp, b9, is not installed; run `ai-ctl build \
                 llama.cpp`",
                "the current build of llama.cpp, v1.0.0, cannot be read: no build.json; run \
                 `ai-ctl build llama.cpp --ref v1.0.0 --force`, or `ai-ctl build llama.cpp` at \
                 the newest release if the project withdrew v1.0.0",
            ]
        );

        store.builds.borrow_mut().push(installed("b2", 20));
        *store.current.borrow_mut() = Some(id("b2"));
        list(&store, None, &mut |warning| panic!("{warning}")).unwrap();
    }

    #[test]
    fn list_shows_the_newest_first_with_the_current_one() {
        let store = FakeStore::default();
        store.builds.borrow_mut().extend([
            installed("b1", 10),
            installed("b3", 30),
            installed("b2", 20),
        ]);
        *store.current.borrow_mut() = Some(id("b2"));
        let listings = list(&store, None, &mut |_| {}).unwrap();
        assert_eq!(listings[0].project, "llama.cpp");
        let ids: Vec<&str> = listings[0].builds.iter().map(|b| b.id.as_str()).collect();
        assert_eq!(ids, ["b3", "b2", "b1"]);
        assert_eq!(listings[0].current, Some(id("b2")));
        assert!(list(&store, Some("nope"), &mut |_| {}).is_err());
    }

    /// An installed build of `project`.
    fn installed_of(project: &str, id: &str, at: i64) -> Build {
        Build {
            project: project.into(),
            ..installed(id, at)
        }
    }

    /// llama.cpp's `b1` to `b3`, `b1` current; whisper.cpp's `v1.9.3` to
    /// `v1.9.5`, `v1.9.4` current; no build of the other projects.
    fn two_projects() -> FakeStore {
        let store = FakeStore::default();
        store.builds.borrow_mut().extend([
            installed("b1", 10),
            installed_of("whisper.cpp", "v1.9.3", 5),
            installed("b2", 20),
            installed_of("whisper.cpp", "v1.9.4", 15),
            installed("b3", 30),
            installed_of("whisper.cpp", "v1.9.5", 25),
        ]);
        *store.current.borrow_mut() = Some(id("b1"));
        store
            .other_currents
            .borrow_mut()
            .insert("whisper.cpp".into(), id("v1.9.4"));
        store
    }

    /// The ids of `project`'s builds left, in the store's order.
    fn left_of(store: &FakeStore, project: &str) -> Vec<String> {
        let builds = store.builds.borrow();
        let builds = builds.iter().filter(|build| build.project == project);
        builds.map(|build| build.id.to_string()).collect()
    }

    #[test]
    fn list_shows_each_project_with_its_own_current_build() {
        let store = two_projects();
        let listings = list(&store, None, &mut |warning| panic!("{warning}")).unwrap();
        let projects: Vec<&str> = listings.iter().map(|listing| listing.project).collect();
        assert_eq!(
            projects,
            [
                "llama.cpp",
                "whisper.cpp",
                "stable-diffusion.cpp",
                "audio.cpp"
            ]
        );
        let ids = |listing: &Listing| -> Vec<String> {
            listing
                .builds
                .iter()
                .map(|build| build.id.to_string())
                .collect()
        };
        assert_eq!(ids(&listings[0]), ["b3", "b2", "b1"]);
        assert_eq!(listings[0].current, Some(id("b1")));
        assert_eq!(ids(&listings[1]), ["v1.9.5", "v1.9.4", "v1.9.3"]);
        assert_eq!(listings[1].current, Some(id("v1.9.4")));
        assert!(listings[2].builds.is_empty() && listings[2].current.is_none());
        let whisper = list(&store, Some("whisper.cpp"), &mut |_| {}).unwrap();
        assert_eq!(whisper, listings[1..2]);
    }

    #[test]
    fn use_changes_its_own_projects_current_build_only() {
        let store = two_projects();
        let warn = &mut |warning| panic!("{warning}");
        let previous = use_build(&store, "whisper.cpp", "v1.9.3", warn).unwrap();
        assert_eq!(previous, Some(id("v1.9.4")));
        assert_eq!(
            store.other_currents.borrow().get("whisper.cpp"),
            Some(&id("v1.9.3"))
        );
        assert_eq!(*store.current.borrow(), Some(id("b1")));
        // Another project's build is not one of its own.
        let text = message(use_build(&store, "whisper.cpp", "b2", warn));
        assert!(text.contains("whisper.cpp has no build b2"), "{text}");
    }

    #[test]
    fn prune_keeps_each_projects_current_build_and_newest_ones() {
        let store = two_projects();
        let pruned = prune(&store, &FakeGateway::default(), None, 1).unwrap();
        assert_eq!(pruned[0].removed, [id("b2")]);
        assert_eq!(pruned[1].removed, [id("v1.9.3")]);
        assert!(pruned[2].removed.is_empty() && pruned[3].removed.is_empty());
        assert_eq!(left_of(&store, "llama.cpp"), ["b1", "b3"]);
        assert_eq!(left_of(&store, "whisper.cpp"), ["v1.9.4", "v1.9.5"]);
    }

    #[test]
    fn prune_of_one_project_leaves_the_others_alone() {
        let store = two_projects();
        let pruned = prune(&store, &FakeGateway::default(), Some("whisper.cpp"), 0).unwrap();
        assert_eq!(pruned.len(), 1);
        assert_eq!(left_of(&store, "whisper.cpp"), ["v1.9.4"]);
        assert_eq!(left_of(&store, "llama.cpp"), ["b1", "b2", "b3"]);
    }

    #[test]
    fn prune_removes_nothing_while_the_gateway_serves_a_build_of_one_project() {
        let store = two_projects();
        let gateway = gateway_on(&store, "b2");
        let text = message(prune(&store, &gateway, None, 1));
        assert!(
            text.starts_with("the running gateway serves llama.cpp b2, which prune would remove"),
            "{text}"
        );
        assert_eq!(left_of(&store, "llama.cpp"), ["b1", "b2", "b3"]);
        assert_eq!(
            left_of(&store, "whisper.cpp"),
            ["v1.9.3", "v1.9.4", "v1.9.5"]
        );
    }

    #[test]
    fn use_switches_to_an_installed_build_only() {
        let store = FakeStore::default();
        store
            .builds
            .borrow_mut()
            .extend([installed("b1", 10), installed("b2", 20)]);
        *store.current.borrow_mut() = Some(id("b2"));
        let warn = &mut |warning| panic!("{warning}");
        assert_eq!(
            use_build(&store, "llama.cpp", "b1", warn).unwrap(),
            Some(id("b2"))
        );
        assert_eq!(*store.current.borrow(), Some(id("b1")));
        assert_eq!(use_build(&store, "llama.cpp", "b1", warn).unwrap(), None);

        let text = message(use_build(&store, "llama.cpp", "b9", warn));
        assert!(text.contains("has no build b9"), "{text}");
        assert!(message(use_build(&store, "llama.cpp", "../b1", warn)).contains("not a build id"));
        assert_eq!(*store.current.borrow(), Some(id("b1")));
    }

    #[test]
    fn a_build_replaces_a_current_link_that_names_no_build() {
        let world = World::new();
        world.store.bad_link.set(true);
        let (built, events) = world.build(None, false);
        let built = built.unwrap();
        assert_eq!(built.previous, None);
        assert_eq!(*world.store.current.borrow(), Some(id("v1.0.0")));
        assert!(
            events.contains(&Event::Warning(
                "/data/builds/llama.cpp/current points at ../llama.cpp/b11538, not at a build of its directory; the build, once installed, replaces it".into()
            )),
            "{events:?}"
        );
    }

    #[test]
    fn use_and_list_read_a_current_link_that_names_no_build_as_none() {
        let store = FakeStore::default();
        store
            .builds
            .borrow_mut()
            .extend([installed("b1", 10), installed("b2", 20)]);
        store.bad_link.set(true);
        let mut warnings = Vec::new();
        let listings = list(&store, None, &mut |warning| warnings.push(warning)).unwrap();
        assert_eq!(listings[0].current, None);
        assert_eq!(listings[0].builds.len(), 2);
        assert_eq!(
            warnings,
            [
                "/data/builds/llama.cpp/current points at ../llama.cpp/b11538, not at a build of its directory; `ai-ctl build use` or `ai-ctl build` replaces it"
            ]
        );

        warnings.clear();
        let previous = use_build(&store, "llama.cpp", "b1", &mut |w| warnings.push(w)).unwrap();
        assert_eq!(previous, None);
        assert_eq!(*store.current.borrow(), Some(id("b1")));
        assert_eq!(
            warnings,
            [
                "/data/builds/llama.cpp/current points at ../llama.cpp/b11538, not at a build of its directory; `build use` replaces it"
            ]
        );
    }

    #[test]
    fn prune_refuses_a_current_link_that_names_no_build() {
        let store = FakeStore::default();
        store
            .builds
            .borrow_mut()
            .extend([installed("b1", 10), installed("b2", 20)]);
        store.bad_link.set(true);
        let text = message(prune(&store, &FakeGateway::default(), None, 0));
        assert_eq!(
            text,
            "/data/builds/llama.cpp/current points at ../llama.cpp/b11538, not at a build of its directory"
        );
        assert_eq!(store.builds.borrow().len(), 2);
    }

    #[test]
    fn prune_keeps_the_newest_and_the_current_build() {
        let store = FakeStore::default();
        store.builds.borrow_mut().extend([
            installed("b1", 10),
            installed("b2", 20),
            installed("b3", 30),
            installed("b4", 40),
        ]);
        *store.current.borrow_mut() = Some(id("b1"));
        let pruned = prune(&store, &FakeGateway::default(), Some("llama.cpp"), 2).unwrap();
        assert_eq!(pruned[0].removed, [id("b2")]);
        assert_eq!(pruned[0].kept, 3);
        assert_eq!(left(&store), ["b1", "b3", "b4"]);
    }

    #[test]
    fn prune_holds_the_gateways_lock_saying_it_removes_builds() {
        let store = FakeStore::default();
        store
            .builds
            .borrow_mut()
            .extend([installed("b1", 10), installed("b2", 20)]);
        let gateway = FakeGateway::default();
        prune(&store, &gateway, None, 1).unwrap();
        assert_eq!(*gateway.locks.borrow(), ["removing builds"]);
    }

    #[test]
    fn a_build_again_holds_the_gateways_lock_saying_it_replaces_one() {
        let world = World::new();
        world.store.builds.borrow_mut().push(installed("v1.0.0", 1));
        world.build(None, true).0.unwrap();
        assert_eq!(
            *world.gateway.locks.borrow(),
            ["replacing an installed build"]
        );
    }

    #[test]
    fn prune_waits_for_no_one() {
        let store = FakeStore {
            locked: true,
            ..FakeStore::default()
        };
        store.builds.borrow_mut().push(installed("b1", 10));
        let text = message(prune(&store, &FakeGateway::default(), None, 0));
        assert!(text.contains("another ai-ctl"), "{text}");
        assert_eq!(store.builds.borrow().len(), 1);

        let store = FakeStore::default();
        store.builds.borrow_mut().push(installed("b1", 10));
        let gateway = FakeGateway {
            locked: true,
            ..FakeGateway::default()
        };
        let text = message(prune(&store, &gateway, None, 0));
        assert_eq!(text, "another ai-ctl is stopping the gateway");
        assert_eq!(store.builds.borrow().len(), 1);
    }

    /// The ids of the builds left, in the store's order.
    fn left(store: &FakeStore) -> Vec<String> {
        store
            .builds
            .borrow()
            .iter()
            .map(|b| b.id.to_string())
            .collect()
    }

    /// A gateway running on the configuration `up` wrote while `served`
    /// was the current build in `store`.
    fn gateway_on(store: &FakeStore, served: &str) -> FakeGateway {
        let current = store.current.replace(Some(id(served)));
        let config = Configurator {
            inventory: &FakeInventory(Ok(Some(inventory("/ai/models")))),
            builds: store,
            gpu: &FakeGpu(Probed::Found(gpu())),
            files_dir: Path::new(crate::application::fakes::GATEWAY_DIR),
        }
        .generate()
        .unwrap()
        .to_yaml();
        *store.current.borrow_mut() = current;
        FakeGateway {
            record: RefCell::new(Some(record(1234))),
            config: RefCell::new(Some(config)),
            ..FakeGateway::default()
        }
    }

    /// Builds `b1` to `b4`, `b4` the newest and the current one, and a
    /// gateway running on the configuration `up` wrote while `served` was
    /// current.
    fn serving(served: &str) -> (FakeStore, FakeGateway) {
        let store = FakeStore::default();
        store.builds.borrow_mut().extend([
            installed("b1", 10),
            installed("b2", 20),
            installed("b3", 30),
            installed("b4", 40),
        ]);
        *store.current.borrow_mut() = Some(id("b4"));
        let gateway = gateway_on(&store, served);
        (store, gateway)
    }

    #[test]
    fn prune_refuses_to_remove_the_build_the_running_gateway_serves() {
        let (store, gateway) = serving("b2");
        let text = message(prune(&store, &gateway, None, 1));
        assert_eq!(
            text,
            "the running gateway serves llama.cpp b2, which prune would remove; nothing is \
             removed. `ai-ctl down` then `ai-ctl up` restarts the gateway on the current build"
        );
        assert_eq!(left(&store), ["b1", "b2", "b3", "b4"]);
    }

    #[test]
    fn prune_removes_the_builds_the_running_gateway_does_not_serve() {
        let (store, gateway) = serving("b4");
        let pruned = prune(&store, &gateway, None, 1).unwrap();
        assert_eq!(pruned[0].removed, [id("b3"), id("b2"), id("b1")]);
        assert_eq!(left(&store), ["b4"]);
    }

    #[test]
    fn prune_removes_the_build_of_a_gateway_that_no_longer_runs() {
        for liveness in [Liveness::Exited, Liveness::Reused(None)] {
            let (store, mut gateway) = serving("b2");
            gateway.liveness = Some(liveness);
            let pruned = prune(&store, &gateway, None, 1).unwrap();
            assert_eq!(pruned[0].removed, [id("b3"), id("b2"), id("b1")]);
        }
    }

    #[test]
    fn a_prune_that_cannot_remove_a_build_names_it() {
        let store = FakeStore {
            remove_fails: Some("b1"),
            ..FakeStore::default()
        };
        store
            .builds
            .borrow_mut()
            .extend([installed("b1", 10), installed("b2", 20)]);
        *store.current.borrow_mut() = Some(id("b2"));
        assert_eq!(
            message(prune(&store, &FakeGateway::default(), None, 0)),
            "cannot remove llama.cpp b1: cannot remove /data/builds/llama.cpp/b1: Permission \
             denied (os error 13)"
        );
        assert_eq!(left(&store), ["b1", "b2"]);
    }

    #[test]
    fn a_prune_that_fails_midway_names_the_builds_it_removed() {
        let store = FakeStore {
            remove_fails: Some("b1"),
            ..FakeStore::default()
        };
        store.builds.borrow_mut().extend([
            installed("b1", 10),
            installed("b2", 20),
            installed("b3", 30),
        ]);
        *store.current.borrow_mut() = Some(id("b3"));
        assert_eq!(
            message(prune(&store, &FakeGateway::default(), None, 0)),
            "cannot remove llama.cpp b1, after removing llama.cpp b2: cannot remove \
             /data/builds/llama.cpp/b1: Permission denied (os error 13)"
        );
        assert_eq!(left(&store), ["b1", "b3"]);
    }

    #[test]
    fn prune_refuses_to_remove_a_build_while_a_gateway_runs_without_its_configuration() {
        let (store, gateway) = serving("b4");
        *gateway.config.borrow_mut() = None;
        let text = message(prune(&store, &gateway, None, 1));
        assert_eq!(
            text,
            "a gateway runs whose configuration is gone, so prune cannot tell which builds it \
             serves; nothing is removed. `ai-ctl down` then `ai-ctl up` restarts the gateway \
             on the current build"
        );
        assert_eq!(left(&store), ["b1", "b2", "b3", "b4"]);
        let pruned = prune(&store, &gateway, None, 4).unwrap();
        assert!(pruned[0].removed.is_empty());
    }

    /// What `build outdated` reads: each project's remote, its builds, the
    /// inventory, and llama-swap at the pinned release.
    struct Reading {
        sources: FakeSources,
        builds: FakeBuilds,
        inventory: FakeInventory,
        releases: FakeReleases,
    }

    /// `recipe`'s build `id`, as the recipe builds it now.
    fn faithful(recipe: &Recipe, id: &str) -> Build {
        Build {
            project: recipe.project.into(),
            cmake_flags: recipe_flags(recipe),
            patches: recipe
                .patches
                .iter()
                .map(|patch| patch.name.to_owned())
                .collect(),
            ..installed(id, 10)
        }
    }

    fn native(kind: BackendKind, id: &str) -> Backend {
        Backend {
            id: id.into(),
            kind,
            source: BackendSource::Build(kind.project().into()),
        }
    }

    impl Reading {
        /// Backends of every project, listed out of the table's order, with
        /// llama.cpp behind, whisper.cpp current, and the two others behind.
        fn of_every_project() -> Self {
            let mut parts = inventory_parts("/models");
            parts.backends = vec![
                native(BackendKind::AudiocppServer, "audio-native"),
                native(BackendKind::SdServer, "sd-native"),
                native(BackendKind::WhisperServer, "whisper-native"),
                native(BackendKind::LlamaServer, "llama-native"),
                native(BackendKind::LlamaServer, "llama-second"),
                Backend {
                    id: "llama-image".into(),
                    kind: BackendKind::LlamaServer,
                    source: BackendSource::Image("ghcr.io/ggml-org/llama.cpp:server".into()),
                },
            ];
            let mut sources = FakeSources::new();
            sources.project_tags = BTreeMap::from([
                (
                    "llama.cpp",
                    vec!["v0.8.0", "v0.9.0", "v1.0.0-b12359-b1b770d"],
                ),
                ("whisper.cpp", vec!["v1.9.4", "v1.9.5", "v1.7.4-pre-1"]),
                (
                    "stable-diffusion.cpp",
                    vec!["master-1024-77b72a2", "master-1043-0a1b2c3"],
                ),
                ("audio.cpp", vec!["v0.9.3", "v0.9.4"]),
            ]);
            Self {
                sources,
                builds: FakeBuilds {
                    builds: vec![
                        faithful(&LLAMA_CPP, "v0.8.0"),
                        faithful(&WHISPER_CPP, "v1.9.5"),
                        faithful(&STABLE_DIFFUSION_CPP, "master-1024-77b72a2"),
                        faithful(&AUDIO_CPP, "v0.9.3"),
                    ],
                    current: Some(id("v0.8.0")),
                    other_currents: BTreeMap::from([
                        ("whisper.cpp".to_owned(), id("v1.9.5")),
                        ("stable-diffusion.cpp".to_owned(), id("master-1024-77b72a2")),
                        ("audio.cpp".to_owned(), id("v0.9.3")),
                    ]),
                    ..FakeBuilds::default()
                },
                inventory: FakeInventory(Ok(Some(Inventory::new(parts).unwrap()))),
                releases: FakeReleases::current(&PINNED.release.to_string()),
            }
        }

        /// One model on llama.cpp's native build, and its build at v0.9.0.
        fn of_llama_cpp() -> Self {
            let mut reading = Self::of_every_project();
            reading.inventory = FakeInventory(Ok(Some(inventory("/models"))));
            reading.builds.builds = vec![faithful(&LLAMA_CPP, "v0.9.0")];
            reading.builds.current = Some(id("v0.9.0"));
            reading
        }

        fn run(&self, project: Option<&str>) -> (Result<Outdated>, Vec<String>) {
            let mut warnings = Vec::new();
            let outdated = outdated(
                &self.sources,
                &self.builds,
                &self.inventory,
                &self.releases,
                project,
                &mut |warning| warnings.push(warning),
            );
            (outdated, warnings)
        }

        /// The one project found, without a warning.
        fn the_project(&self) -> ProjectStanding {
            let (outdated, warnings) = self.run(None);
            assert_eq!(warnings, Vec::<String>::new());
            let mut projects = outdated.unwrap().projects;
            assert_eq!(projects.len(), 1, "{projects:?}");
            projects.remove(0)
        }
    }

    /// Each project's name, current build, newest release and standing.
    fn standings(outdated: &Outdated) -> Vec<(&str, Option<&str>, &str, &Standing)> {
        outdated
            .projects
            .iter()
            .map(|project| {
                let found = project.found.as_ref().unwrap();
                (
                    project.project.as_str(),
                    project.current.as_ref().map(BuildId::as_str),
                    found.newest.as_str(),
                    &found.standing,
                )
            })
            .collect()
    }

    #[test]
    fn outdated_sets_each_project_beside_its_newest_release_in_the_tables_order() {
        let reading = Reading::of_every_project();
        let (outdated, warnings) = reading.run(None);
        let outdated = outdated.unwrap();
        assert_eq!(warnings, Vec::<String>::new());
        assert_eq!(
            standings(&outdated),
            [
                ("llama.cpp", Some("v0.8.0"), "v0.9.0", &Standing::Behind),
                ("whisper.cpp", Some("v1.9.5"), "v1.9.5", &Standing::Current),
                (
                    "stable-diffusion.cpp",
                    Some("master-1024-77b72a2"),
                    "master-1043-0a1b2c3",
                    &Standing::Behind
                ),
                ("audio.cpp", Some("v0.9.3"), "v0.9.4", &Standing::Behind),
            ]
        );
    }

    #[test]
    fn outdated_reads_each_project_once_from_its_remote_and_nothing_else() {
        let reading = Reading::of_every_project();
        reading.run(None).0.unwrap();
        // Neither a clone nor a fetch; llama-swap is compared with the pin.
        assert_eq!(
            *reading.sources.calls.borrow(),
            [
                "remote_tags llama.cpp",
                "remote_tags whisper.cpp",
                "remote_tags stable-diffusion.cpp",
                "remote_tags audio.cpp",
            ]
        );
    }

    #[test]
    fn outdated_covers_only_the_projects_the_inventory_builds() {
        let reading = Reading::of_llama_cpp();
        let outdated = reading.run(None).0.unwrap();
        assert_eq!(
            standings(&outdated),
            [("llama.cpp", Some("v0.9.0"), "v0.9.0", &Standing::Current)]
        );
        assert_eq!(*reading.sources.calls.borrow(), ["remote_tags llama.cpp"]);
    }

    #[test]
    fn a_project_named_is_read_used_or_not_without_the_inventory() {
        let mut reading = Reading::of_every_project();
        reading.inventory = FakeInventory(Err("the inventory is invalid".into()));
        let (outdated, warnings) = reading.run(Some("audio.cpp"));
        assert_eq!(warnings, Vec::<String>::new());
        assert_eq!(
            standings(&outdated.unwrap()),
            [("audio.cpp", Some("v0.9.3"), "v0.9.4", &Standing::Behind)]
        );
    }

    #[test]
    fn an_unknown_project_named_is_refused() {
        let (outdated, _) = Reading::of_llama_cpp().run(Some("nope.cpp"));
        assert_eq!(
            message(outdated),
            "no recipe for `nope.cpp`; ai-ctl builds llama.cpp, whisper.cpp, \
             stable-diffusion.cpp, audio.cpp"
        );
    }

    #[test]
    fn outdated_needs_an_inventory_when_no_project_is_named() {
        let mut reading = Reading::of_llama_cpp();
        reading.inventory = FakeInventory(Ok(None));
        assert_eq!(
            message(reading.run(None).0),
            "no inventory at /home/user/.config/ai-ctl/inventory.toml; install it with \
             `ai-ctl init --models-dir <dir>`"
        );
        reading.inventory = FakeInventory(Err("the inventory is invalid".into()));
        assert_eq!(message(reading.run(None).0), "the inventory is invalid");
        assert!(reading.sources.calls.borrow().is_empty());
    }

    #[test]
    fn a_project_the_inventory_builds_without_a_recipe_is_named_last() {
        let mut parts = inventory_parts("/models");
        parts.backends.push(Backend {
            id: "nope-native".into(),
            kind: BackendKind::WhisperServer,
            source: BackendSource::Build("nope.cpp".into()),
        });
        let mut reading = Reading::of_llama_cpp();
        reading.inventory = FakeInventory(Ok(Some(Inventory::new(parts).unwrap())));
        let projects = reading.run(None).0.unwrap().projects;
        assert_eq!(projects.len(), 2, "{projects:?}");
        assert_eq!(projects[0].project, "llama.cpp");
        assert_eq!(projects[1].project, "nope.cpp");
        assert_eq!(projects[1].current, None);
        assert_eq!(
            projects[1].found,
            Err("ai-ctl has no recipe for it".to_owned())
        );
    }

    #[test]
    fn a_project_whose_remote_fails_is_said_without_hiding_the_others() {
        let mut reading = Reading::of_every_project();
        reading.sources.remote_fails = Some("llama.cpp");
        let projects = reading.run(None).0.unwrap().projects;
        assert_eq!(projects.len(), 4, "{projects:?}");
        assert_eq!(projects[0].project, "llama.cpp");
        assert_eq!(projects[0].current, Some(id("v0.8.0")));
        assert_eq!(
            projects[0].found,
            Err(
                "cannot read the tags of https://github.com/ggml-org/llama.cpp.git: Could not \
                 resolve host: github.com"
                    .to_owned()
            )
        );
        assert!(projects[1..].iter().all(|project| project.found.is_ok()));
    }

    #[test]
    fn a_repository_without_a_release_says_so() {
        let mut reading = Reading::of_llama_cpp();
        reading
            .sources
            .project_tags
            .insert("llama.cpp", vec!["backup/old", "v1.0.0-b12359-b1b770d"]);
        assert_eq!(
            reading.the_project().found,
            Err(
                "no release tag of https://github.com/ggml-org/llama.cpp.git matches v[0-9]*"
                    .to_owned()
            )
        );
    }

    #[test]
    fn a_project_with_no_build_is_not_built() {
        let mut reading = Reading::of_llama_cpp();
        reading.builds.builds.clear();
        reading.builds.current = None;
        let project = reading.the_project();
        assert_eq!(project.current, None);
        assert_eq!(project.found.unwrap().standing, Standing::NotBuilt);
    }

    #[test]
    fn a_build_made_from_a_commit_is_told_with_the_newest_release() {
        let mut reading = Reading::of_llama_cpp();
        reading.builds.builds = vec![faithful(&LLAMA_CPP, "b1b770d90d")];
        reading.builds.current = Some(id("b1b770d90d"));
        let project = reading.the_project();
        assert_eq!(project.current, Some(id("b1b770d90d")));
        let found = project.found.unwrap();
        assert_eq!(found.newest, "v0.9.0");
        assert_eq!(found.standing, Standing::FromCommit);
    }

    #[test]
    fn a_nightly_build_of_llama_cpp_reads_as_built_from_a_commit() {
        // As ai-ctl built llama.cpp before it took its releases `vX.Y.Z`.
        let mut reading = Reading::of_llama_cpp();
        reading.sources.project_tags.insert(
            "llama.cpp",
            vec!["v0.9.0", "v1.0.0", "v1.0.0-b12359-b1b770d"],
        );
        reading.builds.builds = vec![faithful(&LLAMA_CPP, "b12359")];
        reading.builds.current = Some(id("b12359"));
        let project = reading.the_project();
        assert_eq!(project.current, Some(id("b12359")));
        assert_eq!(
            project.found,
            Ok(Found {
                newest: "v1.0.0".into(),
                standing: Standing::FromCommit,
            })
        );
    }

    #[test]
    fn a_build_at_the_newest_release_its_recipe_no_longer_matches_is_named() {
        let mut build = faithful(&LLAMA_CPP, "v0.9.0");
        build.cmake_flags.remove("LLAMA_USE_PREBUILT_UI");
        let mut reading = Reading::of_llama_cpp();
        reading.builds.builds = vec![build];
        assert_eq!(
            reading.the_project().found.unwrap().standing,
            Standing::OffRecipe(
                "v0.9.0 was built without LLAMA_USE_PREBUILT_UI=OFF, which its recipe sets"
                    .to_owned()
            )
        );
    }

    #[test]
    fn a_current_build_that_cannot_be_read_warns_and_counts_as_not_built() {
        let unreadable = |reading: &mut Reading| {
            reading.builds.unreadable = vec![Unreadable {
                dir: "/data/builds/llama.cpp/v0.9.0".into(),
                reason: "cannot read build.json".into(),
            }];
            reading.builds.builds.clear();
        };
        let missing = |reading: &mut Reading| reading.builds.builds.clear();
        let link_fails = |reading: &mut Reading| reading.builds.current_fails = true;
        let store_fails = |reading: &mut Reading| reading.builds.builds_fails = true;
        type Arrange<'a> = &'a dyn Fn(&mut Reading);
        let cases: [(Arrange, &str); 4] = [
            (
                &missing,
                "the current build of llama.cpp, v0.9.0, is not installed; run \
                 `ai-ctl build llama.cpp`",
            ),
            (
                &unreadable,
                "the current build of llama.cpp, v0.9.0, cannot be read: cannot read build.json; \
                 run `ai-ctl build llama.cpp --ref v0.9.0 --force`, or `ai-ctl build llama.cpp` \
                 at the newest release if the project withdrew v0.9.0",
            ),
            (
                &link_fails,
                "cannot read /data/builds/llama.cpp/current: Permission denied (os error 13)",
            ),
            (
                &store_fails,
                "cannot read /data/builds/llama.cpp: Permission denied (os error 13)",
            ),
        ];
        for (arrange, warned) in cases {
            let mut reading = Reading::of_llama_cpp();
            arrange(&mut reading);
            let (outdated, warnings) = reading.run(None);
            assert_eq!(warnings, [warned]);
            let project = outdated.unwrap().projects.remove(0);
            assert_eq!(project.current, None, "{warned}");
            assert_eq!(
                project.found.unwrap().standing,
                Standing::NotBuilt,
                "{warned}"
            );
        }
    }

    #[test]
    fn llama_swap_is_set_beside_the_release_this_ai_ctl_pins() {
        let mut reading = Reading::of_llama_cpp();
        reading.releases = FakeReleases::current("v270");
        let outdated = reading.run(None).0.unwrap();
        assert_eq!(
            outdated.swap,
            Some((Release::parse("v270").unwrap(), SwapStanding::Behind))
        );
        assert_eq!(outdated.pinned, PINNED.release);
        reading.releases = FakeReleases::none();
        assert_eq!(reading.run(None).0.unwrap().swap, None);
    }

    #[test]
    fn a_llama_swap_link_that_cannot_be_read_warns_and_reads_as_none() {
        let mut reading = Reading::of_llama_cpp();
        reading.releases.current_fails = true;
        let (outdated, warnings) = reading.run(None);
        assert_eq!(outdated.unwrap().swap, None);
        assert_eq!(
            warnings,
            [
                "cannot read /data/llama-swap/current: Permission denied (os error 13); \
                 `ai-ctl gateway install` replaces it"
            ]
        );
    }

    impl World {
        /// Each project's tags as [`Reading`]'s, its server answering as
        /// its recipe expects, and each project's build installed: llama.cpp
        /// behind, whisper.cpp current, and the two others behind.
        fn updating() -> Self {
            let mut world = Self::new();
            world.sources.project_tags = Reading::of_every_project().sources.project_tags;
            world.programs.version = printed(
                true,
                "version: 1\nstable-diffusion.cpp version 1\naudio.cpp 1\n",
                "",
            );
            world.store.builds.borrow_mut().extend([
                faithful(&LLAMA_CPP, "v0.8.0"),
                faithful(&WHISPER_CPP, "v1.9.5"),
                faithful(&STABLE_DIFFUSION_CPP, "master-1024-77b72a2"),
                faithful(&AUDIO_CPP, "v0.9.3"),
            ]);
            *world.store.current.borrow_mut() = Some(id("v0.8.0"));
            world.store.other_currents.borrow_mut().extend([
                ("whisper.cpp".to_owned(), id("v1.9.5")),
                ("stable-diffusion.cpp".to_owned(), id("master-1024-77b72a2")),
                ("audio.cpp".to_owned(), id("v0.9.3")),
            ]);
            world
        }

        fn update(
            &self,
            projects: &[ProjectStanding],
            jobs: Option<usize>,
        ) -> (Updated, Vec<Event>) {
            let clock = || Timestamp::from_unix(NOW).unwrap();
            let builder = Builder {
                gpu: &self.gpu,
                toolchain: &self.toolchain,
                sources: &self.sources,
                compiler: &self.compiler,
                store: &self.store,
                gateway: &self.gateway,
                programs: &self.programs,
                clock: &clock,
            };
            let mut events = Vec::new();
            let updated = builder.update(projects, jobs, &mut |event| events.push(event));
            (updated, events)
        }

        /// Each installed build of `project`, sorted.
        fn builds_of(&self, project: &str) -> Vec<String> {
            let (builds, _) = self.store.builds(project).unwrap();
            let mut ids: Vec<String> = builds.iter().map(|build| build.id.to_string()).collect();
            ids.sort();
            ids
        }

        fn current_of(&self, project: &str) -> Option<String> {
            self.store
                .current(project)
                .unwrap()
                .map(|id| id.to_string())
        }

        /// The references checked out, in turn.
        fn checkouts(&self) -> Vec<String> {
            let calls = self.sources.calls.borrow();
            let checkouts = calls
                .iter()
                .filter_map(|call| call.strip_prefix("checkout "));
            checkouts.map(str::to_owned).collect()
        }
    }

    fn standing_of(
        project: &str,
        current: Option<&str>,
        newest: &str,
        standing: Standing,
    ) -> ProjectStanding {
        ProjectStanding {
            project: project.into(),
            current: current.map(id),
            found: Ok(Found {
                newest: newest.into(),
                standing,
            }),
        }
    }

    /// The four projects as `build outdated` finds them in
    /// [`World::updating`].
    fn four_projects() -> Vec<ProjectStanding> {
        vec![
            standing_of("llama.cpp", Some("v0.8.0"), "v0.9.0", Standing::Behind),
            standing_of("whisper.cpp", Some("v1.9.5"), "v1.9.5", Standing::Current),
            standing_of(
                "stable-diffusion.cpp",
                Some("master-1024-77b72a2"),
                "master-1043-0a1b2c3",
                Standing::Behind,
            ),
            standing_of("audio.cpp", Some("v0.9.3"), "v0.9.4", Standing::Behind),
        ]
    }

    fn built_update(project: &str, built: &str, previous: Option<&str>) -> ProjectUpdate {
        ProjectUpdate {
            project: project.into(),
            outcome: UpdateOutcome::Built {
                id: id(built),
                previous: previous.map(id),
            },
        }
    }

    #[test]
    fn update_builds_each_project_behind_in_turn_and_leaves_the_current_one() {
        let world = World::updating();
        let (updated, _) = world.update(&four_projects(), None);
        assert_eq!(
            updated.projects,
            [
                built_update("llama.cpp", "v0.9.0", Some("v0.8.0")),
                ProjectUpdate {
                    project: "whisper.cpp".into(),
                    outcome: UpdateOutcome::AlreadyCurrent,
                },
                built_update(
                    "stable-diffusion.cpp",
                    "master-1043-0a1b2c3",
                    Some("master-1024-77b72a2")
                ),
                built_update("audio.cpp", "v0.9.4", Some("v0.9.3")),
            ]
        );
        assert_eq!(
            world.checkouts(),
            ["v0.9.0", "master-1043-0a1b2c3", "v0.9.4"]
        );
    }

    #[test]
    fn update_makes_each_new_build_current_and_removes_none() {
        let world = World::updating();
        world.update(&four_projects(), None);
        assert_eq!(world.builds_of("llama.cpp"), ["v0.8.0", "v0.9.0"]);
        assert_eq!(world.current_of("llama.cpp").unwrap(), "v0.9.0");
        assert_eq!(world.builds_of("whisper.cpp"), ["v1.9.5"]);
        assert_eq!(world.current_of("whisper.cpp").unwrap(), "v1.9.5");
        assert_eq!(world.builds_of("audio.cpp"), ["v0.9.3", "v0.9.4"]);
        assert_eq!(world.current_of("audio.cpp").unwrap(), "v0.9.4");
    }

    #[test]
    fn update_tells_each_project_it_goes_on_to_and_no_other() {
        let world = World::updating();
        let (_, events) = world.update(&four_projects(), None);
        let updating: Vec<&str> = events
            .iter()
            .filter_map(|event| match event {
                Event::Updating { project } => Some(project.as_str()),
                _ => None,
            })
            .collect();
        assert_eq!(updating, ["llama.cpp", "stable-diffusion.cpp", "audio.cpp"]);
        assert_eq!(
            events.first(),
            Some(&Event::Updating {
                project: "llama.cpp".into()
            })
        );
    }

    #[test]
    fn a_refused_patch_stops_that_project_only() {
        let mut world = World::updating();
        // The one patch only stable-diffusion.cpp applies.
        world.sources.apply_fails = Some("listen-failure.patch");
        let (updated, _) = world.update(&four_projects(), None);
        assert_eq!(
            updated.projects[0],
            built_update("llama.cpp", "v0.9.0", Some("v0.8.0"))
        );
        assert_eq!(
            updated.projects[2],
            ProjectUpdate {
                project: "stable-diffusion.cpp".into(),
                outcome: UpdateOutcome::Failed(
                    "stable-diffusion.cpp master-1043-0a1b2c3 cannot be built: its patch \
                     listen-failure.patch does not apply; `--ref` builds a release it applies \
                     to: error: patch failed: examples/server/main.cpp:157"
                        .into()
                ),
            }
        );
        assert_eq!(
            updated.projects[3],
            built_update("audio.cpp", "v0.9.4", Some("v0.9.3"))
        );
        // The refused project keeps its build, current as before.
        assert_eq!(
            world.builds_of("stable-diffusion.cpp"),
            ["master-1024-77b72a2"]
        );
        assert_eq!(
            world.current_of("stable-diffusion.cpp").unwrap(),
            "master-1024-77b72a2"
        );
    }

    #[test]
    fn a_project_outdated_could_not_read_fails_and_builds_nothing() {
        let world = World::updating();
        let unread = ProjectStanding {
            project: "llama.cpp".into(),
            current: Some(id("v0.8.0")),
            found: Err("cannot read the tags: no route".into()),
        };
        let (updated, events) = world.update(&[unread], None);
        assert_eq!(
            updated.projects,
            [ProjectUpdate {
                project: "llama.cpp".into(),
                outcome: UpdateOutcome::Failed(
                    "cannot read its releases: cannot read the tags: no route".into()
                ),
            }]
        );
        assert!(events.is_empty(), "{events:?}");
        assert!(world.sources.calls.borrow().is_empty());
    }

    #[test]
    fn a_project_not_built_or_built_from_a_commit_is_built_at_the_newest_release() {
        let world = World::updating();
        *world.store.current.borrow_mut() = None;
        let (updated, _) = world.update(
            &[standing_of("llama.cpp", None, "v0.9.0", Standing::NotBuilt)],
            None,
        );
        assert_eq!(
            updated.projects,
            [built_update("llama.cpp", "v0.9.0", None)]
        );

        let world = World::updating();
        *world.store.current.borrow_mut() = Some(id("b1b770d90d"));
        world
            .store
            .builds
            .borrow_mut()
            .push(faithful(&LLAMA_CPP, "b1b770d90d"));
        let (updated, _) = world.update(
            &[standing_of(
                "llama.cpp",
                Some("b1b770d90d"),
                "v0.9.0",
                Standing::FromCommit,
            )],
            None,
        );
        assert_eq!(
            updated.projects,
            [built_update("llama.cpp", "v0.9.0", Some("b1b770d90d"))]
        );
        assert_eq!(
            world.builds_of("llama.cpp"),
            ["b1b770d90d", "v0.8.0", "v0.9.0"]
        );
    }

    fn off_recipe() -> ProjectStanding {
        standing_of(
            "llama.cpp",
            Some("v0.9.0"),
            "v0.9.0",
            Standing::OffRecipe("v0.9.0 was built without a flag".into()),
        )
    }

    #[test]
    fn update_builds_the_newest_version_beside_a_nightly_build_which_use_brings_back() {
        let mut world = World::updating();
        world.sources.project_tags.insert(
            "llama.cpp",
            vec!["v0.9.0", "v1.0.0", "v1.0.0-b12359-b1b770d"],
        );
        world
            .store
            .builds
            .borrow_mut()
            .push(faithful(&LLAMA_CPP, "b12359"));
        *world.store.current.borrow_mut() = Some(id("b12359"));
        let (updated, _) = world.update(
            &[standing_of(
                "llama.cpp",
                Some("b12359"),
                "v1.0.0",
                Standing::FromCommit,
            )],
            None,
        );
        assert_eq!(
            updated.projects,
            [built_update("llama.cpp", "v1.0.0", Some("b12359"))]
        );
        assert!(world.builds_of("llama.cpp").contains(&"b12359".to_owned()));
        let previous = use_build(&world.store, "llama.cpp", "b12359", &mut |_| {}).unwrap();
        assert_eq!(previous, Some(id("v1.0.0")));
        assert_eq!(world.current_of("llama.cpp").unwrap(), "b12359");
    }

    #[test]
    fn a_build_off_its_recipe_at_the_newest_release_is_built_again() {
        let world = World::updating();
        world
            .store
            .builds
            .borrow_mut()
            .push(faithful(&LLAMA_CPP, "v0.9.0"));
        *world.store.current.borrow_mut() = Some(id("v0.9.0"));
        let (updated, _) = world.update(&[off_recipe()], None);
        assert_eq!(
            updated.projects,
            [built_update("llama.cpp", "v0.9.0", None)]
        );
        assert_eq!(world.checkouts(), ["v0.9.0"]);
        assert!(world.store.replaced.get());
        assert_eq!(world.builds_of("llama.cpp"), ["v0.8.0", "v0.9.0"]);
    }

    /// llama.cpp's current build, `v0.9.1`, off its recipe and past the
    /// newest release the remote shows, `v0.9.0`.
    fn past_the_newest() -> (World, ProjectStanding) {
        let world = World::updating();
        world
            .store
            .builds
            .borrow_mut()
            .push(faithful(&LLAMA_CPP, "v0.9.1"));
        *world.store.current.borrow_mut() = Some(id("v0.9.1"));
        let project = standing_of(
            "llama.cpp",
            Some("v0.9.1"),
            "v0.9.0",
            Standing::OffRecipe("v0.9.1 was built without a flag".into()),
        );
        (world, project)
    }

    #[test]
    fn a_build_off_its_recipe_is_never_sent_back_to_an_older_release() {
        // A clone made since the project withdrew v0.9.1 holds only v0.9.0.
        let (world, project) = past_the_newest();
        let (updated, _) = world.update(&[project], None);
        assert!(
            matches!(updated.projects[0].outcome, UpdateOutcome::Failed(_)),
            "{:?}",
            updated.projects[0]
        );
        assert_eq!(world.current_of("llama.cpp").unwrap(), "v0.9.1");
        assert_eq!(world.builds_of("llama.cpp"), ["v0.8.0", "v0.9.1"]);
    }

    #[test]
    fn a_build_off_its_recipe_whose_release_was_withdrawn_is_told_how_to_go_on() {
        // The fetch before the checkout drops the withdrawn tag: nothing
        // builds v0.9.1 again, and `ai-ctl build llama.cpp` builds v0.9.0.
        let (world, project) = past_the_newest();
        let (updated, _) = world.update(&[project], None);
        let UpdateOutcome::Failed(cause) = &updated.projects[0].outcome else {
            panic!("{:?}", updated.projects[0]);
        };
        assert!(cause.contains("withdrawn"), "{cause}");
        assert!(cause.contains("`ai-ctl build llama.cpp`"), "{cause}");
        assert!(world.checkouts().is_empty());
        assert_eq!(world.current_of("llama.cpp").unwrap(), "v0.9.1");
    }

    #[test]
    fn a_build_off_its_recipe_is_refused_while_the_running_gateway_serves_it() {
        let mut world = World::updating();
        world
            .store
            .builds
            .borrow_mut()
            .push(faithful(&LLAMA_CPP, "v0.9.0"));
        *world.store.current.borrow_mut() = Some(id("v0.9.0"));
        world.gateway = gateway_on(&world.store, "v0.9.0");
        let (updated, _) = world.update(&[off_recipe()], None);
        let UpdateOutcome::Failed(cause) = &updated.projects[0].outcome else {
            panic!("{:?}", updated.projects[0]);
        };
        assert!(cause.contains("`ai-ctl down`"), "{cause}");
        assert!(!world.store.replaced.get());
    }

    #[test]
    fn a_project_behind_is_not_built_again_over_its_newest_build() {
        // `build use` went back to v0.8.0: going forward is the user's word.
        let world = World::updating();
        world
            .store
            .builds
            .borrow_mut()
            .push(faithful(&LLAMA_CPP, "v0.9.0"));
        let (updated, _) = world.update(
            &[standing_of(
                "llama.cpp",
                Some("v0.8.0"),
                "v0.9.0",
                Standing::Behind,
            )],
            None,
        );
        assert_eq!(
            updated.projects,
            [ProjectUpdate {
                project: "llama.cpp".into(),
                outcome: UpdateOutcome::Failed(
                    "llama.cpp v0.9.0 is already built; `ai-ctl build use llama.cpp v0.9.0` \
                     makes it current, `ai-ctl build llama.cpp --ref v0.9.0 --force` builds \
                     it again"
                        .into()
                ),
            }]
        );
        assert!(!world.store.replaced.get());
        assert_eq!(world.current_of("llama.cpp").unwrap(), "v0.8.0");
    }

    #[test]
    fn the_jobs_asked_for_reach_each_compile() {
        let world = World::updating();
        world.update(&four_projects(), Some(3));
        assert_eq!(world.compiler.jobs.get(), Some(3));
    }

    #[test]
    fn update_tells_whether_a_gateway_runs() {
        let world = World::updating();
        assert!(!world.update(&[], None).0.gateway_runs);

        let mut world = World::updating();
        world.gateway.record = RefCell::new(Some(record(1234)));
        assert!(world.update(&[], None).0.gateway_runs);

        // A record whose process is gone is no gateway.
        let mut world = World::updating();
        world.gateway.record = RefCell::new(Some(record(1234)));
        world.gateway.liveness = Some(Liveness::Exited);
        assert!(!world.update(&[], None).0.gateway_runs);
    }

    #[test]
    fn a_gateway_that_cannot_be_read_is_taken_for_one_with_a_warning() {
        let mut world = World::updating();
        world.gateway.record_fails = true;
        let (updated, events) = world.update(&[], None);
        assert!(updated.gateway_runs);
        assert_eq!(
            events,
            [Event::Warning(
                "cannot read /state/gateway/gateway.json: Permission denied (os error 13)".into()
            )]
        );
    }

    #[test]
    fn use_waits_for_no_one() {
        let store = FakeStore {
            locked: true,
            ..FakeStore::default()
        };
        store
            .builds
            .borrow_mut()
            .extend([installed("b1", 10), installed("b2", 20)]);
        *store.current.borrow_mut() = Some(id("b2"));
        assert_eq!(
            message(use_build(&store, "llama.cpp", "b1", &mut |_| {})),
            "another ai-ctl is changing the builds"
        );
        assert_eq!(*store.current.borrow(), Some(id("b2")));
    }
}
