//! `build`: compiles a project at a git reference into a new installed
//! build, and manages the installed builds (`list`, `use`, `prune`).

use std::fmt::Write as _;
use std::path::PathBuf;

use anyhow::{Context, Result, anyhow, bail, ensure};

use crate::domain::build::{Build, BuildId, CommitId, cmake_flags, newest_first, to_prune};
use crate::domain::diagnostics::{Check, Status};
use crate::domain::gateway_config::starts_program;
use crate::domain::recipe::{PatchRefused, RECIPES, Recipe, cuda_devices, recipe};
use crate::domain::timestamp::Timestamp;

use super::doctor::gpu_checks;
use super::lifecycle::live_record;
use super::ports::{
    BuildStore, Checkout, CompileJob, Compiler, GatewayProcess, GpuProbe, ProgramOutput, Programs,
    Progress, Sources, ToolchainProbe, Unreadable,
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
        let flags = cmake_flags(recipe, &toolchain);
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
                    "{} is not a readable build: {}; `--force` builds {project} {id} in its place",
                    entry.dir.display(),
                    entry.reason
                );
            }
            if current == Some(id) {
                bail!("{project} {id} is already built and current; `--force` builds it again");
            }
            bail!(
                "{project} {id} is already built; `ai-ctl build use {project} {id}` makes it current, `--force` builds it again"
            );
        }
        Ok(taken)
    }

    /// The newest release tag of the project's clone.
    fn latest_release(&self, recipe: &Recipe) -> Result<String> {
        let tags = self.sources.release_tags(recipe)?;
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

/// How `build` was not built as `recipe` says now, such as `v1.9.5 was
/// built without reuse-address.patch, which its recipe applies` or `b12111
/// was built without LLAMA_USE_PREBUILT_UI=OFF, which its recipe sets`, or
/// `None` when it was built with each of its patches and each of its CMake
/// options at its value: a patch or an option a later ai-ctl adds reaches
/// only the builds made after it. The options the builder sets for every
/// project are not compared, nor an option the recipe no longer sets; a
/// patch is matched by name: one rebased under the same name adds the same
/// lines to a newer release, and one whose change differs takes a new name.
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
                "the current build of {project}, {id}, cannot be read: {reason}; run \
                 `ai-ctl build {project} --ref {id} --force`"
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
        FakeBuilds, FakeGateway, FakeGpu, FakeInventory, FakePrograms, FakeToolchain, b11538, gpu,
        inventory, printed, record,
    };
    use crate::application::gateway_config::Configurator;
    use crate::application::ports::{Compiled, Liveness, Staged};
    use crate::domain::build::CUDA_ARCHITECTURES;
    use crate::domain::machine::{Probed, Version};
    use crate::domain::recipe::{LLAMA_CPP, Patch, SmokeTest};

    const B11538: &str = "79e2e74eb11022c1ba2e438df7f0ca2d4c10f8b6";
    const FEATURE: &str = "0123456789abcdef0123456789abcdef01234567";
    /// whisper.cpp's `v1.9.5`.
    const V1_9_5: &str = "d1be6fde0123456789abcdef0123456789abcdef";
    const NOW: i64 = 1_791_568_931;

    /// `tags`, and a `feature` branch; `calls` records each call in turn,
    /// and `patched` each patch applied. The patch `apply_fails` names does
    /// not apply.
    struct FakeSources {
        tags: Vec<&'static str>,
        calls: RefCell<Vec<String>>,
        patched: RefCell<Vec<&'static str>>,
        apply_fails: Option<&'static str>,
        /// The patch whose file cannot be written, so is never tried.
        apply_errs: Option<&'static str>,
    }

    impl FakeSources {
        /// Release tags `b11537` and `b11538`, and a tag that is not one.
        fn new() -> Self {
            Self {
                tags: vec!["b11537", "b10964-b10964-b29c606", "b11538"],
                calls: RefCell::new(vec![]),
                patched: RefCell::new(vec![]),
                apply_fails: None,
                apply_errs: None,
            }
        }
    }

    impl Sources for FakeSources {
        fn fetch(&self, _: &Recipe) -> Result<()> {
            self.calls.borrow_mut().push("fetch".into());
            Ok(())
        }
        fn release_tags(&self, _: &Recipe) -> Result<Vec<String>> {
            self.calls.borrow_mut().push("release_tags".into());
            Ok(self.tags.iter().map(|&tag| tag.to_owned()).collect())
        }
        fn checkout(&self, _: &Recipe, reference: &str) -> Result<Checkout> {
            self.calls
                .borrow_mut()
                .push(format!("checkout {reference}"));
            let (commit, tags) = match reference {
                "b11538" | "79e2e74" | "79e2e74eb1" => (B11538, vec!["b11538".to_owned()]),
                "v1.9.5" => (V1_9_5, vec!["v1.9.5".to_owned()]),
                "feature" => (FEATURE, vec![]),
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
        /// b11538.
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
            let checkout = self.sources.checkout(recipe, "b11538")?;
            builder.apply_patches(recipe, &checkout, &id("b11538"), &mut |event| {
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
            commit: CommitId::parse(B11538).unwrap(),
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
            ["fetch", "release_tags", "checkout b11538"]
        );
        assert_eq!(built.build.id, id("b11538"));
        assert_eq!(built.build.commit.as_str(), B11538);
        assert_eq!(built.build.toolkit, Version::new(13, 4, 92));
        assert_eq!(built.build.host_compiler.command, "g++-15");
        assert_eq!(built.build.built_at, Timestamp::from_unix(NOW).unwrap());
        assert_eq!(built.build.cmake_flags[CUDA_ARCHITECTURES], "120a-real");
        assert_eq!(
            built.binary,
            Path::new("/data/builds/llama.cpp/b11538/llama-server")
        );
        assert_eq!(built.previous, Some(id("b11400")));
        assert_eq!(*world.store.current.borrow(), Some(id("b11538")));
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
                    reference: "b11538".into(),
                    id: id("b11538"),
                    commit: CommitId::parse(B11538).unwrap()
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
            ["fetch", "release_tags", "checkout b11538"]
        );
    }

    #[test]
    fn a_clone_without_a_release_tag_fails_with_the_pattern() {
        let mut world = World::new();
        world.sources.tags = vec!["b10964-b10964-b29c606", "master-old"];
        let text = message(world.build(None, false).0);
        assert_eq!(
            text,
            "no release tag of https://github.com/ggml-org/llama.cpp.git matches b[0-9]*"
        );
    }

    #[test]
    fn a_commit_of_a_release_is_named_after_the_release() {
        let world = World::new();
        let built = world.build(Some("79e2e74"), false).0.unwrap();
        assert_eq!(built.build.id, id("b11538"));
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
        world.store.builds.borrow_mut().push(installed("b11538", 1));
        let text = message(world.build(None, false).0);
        assert!(text.contains("already built"), "{text}");
        assert!(text.contains("ai-ctl build use llama.cpp b11538"), "{text}");
        assert!(world.compiler.flags.borrow().is_none(), "nothing compiled");

        *world.store.current.borrow_mut() = Some(id("b11538"));
        let text = message(world.build(None, false).0);
        assert!(text.contains("is already built and current"), "{text}");

        let rebuilt = world.build(None, true).0.unwrap();
        assert!(world.store.replaced.get());
        assert_eq!(rebuilt.build.built_at, Timestamp::from_unix(NOW).unwrap());
        assert_eq!(rebuilt.previous, None);
        assert_eq!(world.store.builds.borrow().len(), 1);
    }

    /// The refusal of `--force` while the running gateway serves b11538.
    const SERVED: &str = "the running gateway serves llama.cpp b11538, which `build --force` \
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
        world.store.builds.borrow_mut().push(installed("b11538", 1));
        *world.store.current.borrow_mut() = Some(id("b11538"));
        world.gateway = gateway_on(&world.store, "b11538");
        assert_eq!(message(world.build(None, true).0), SERVED);
        assert!(world.compiler.flags.borrow().is_none(), "nothing compiled");
        assert_eq!(built_at(&world), [("b11538".to_owned(), at(1))]);

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
            .extend([installed("b11400", 1), installed("b11538", 2)]);
        *world.store.current.borrow_mut() = Some(id("b11538"));
        world.gateway = gateway_on(&world.store, "b11400");
        world.build(None, true).0.unwrap();
        assert!(world.store.replaced.get());
        assert_eq!(
            built_at(&world),
            [("b11400".to_owned(), at(1)), ("b11538".to_owned(), at(NOW))]
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
        world.store.builds.borrow_mut().push(installed("b11538", 1));
        *world.store.current.borrow_mut() = Some(id("b11538"));
        let compiler = StartsGateway {
            compiler: &world.compiler,
            gateway: &world.gateway,
            config: gateway_on(&world.store, "b11538")
                .config
                .into_inner()
                .unwrap(),
        };
        assert_eq!(message(world.build_with(&compiler, None, true).0), SERVED);
        assert_eq!(world.store.discarded.get(), 1);
        assert!(!world.store.replaced.get());
        assert_eq!(built_at(&world), [("b11538".to_owned(), at(1))]);
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
        assert_eq!(built_at(&world), [("b11538".to_owned(), at(NOW))]);
    }

    #[test]
    fn an_installed_release_named_is_refused_before_the_gate_and_the_fetch() {
        let mut world = World::new();
        world.store.builds.borrow_mut().push(installed("b11538", 1));
        world.gpu = FakeGpu(Probed::Missing);
        let (result, events) = world.build(Some("b11538"), false);
        assert_eq!(
            message(result),
            "llama.cpp b11538 is already built; `ai-ctl build use llama.cpp b11538` makes it \
             current, `--force` builds it again"
        );
        assert!(events.is_empty(), "{events:?}");
        assert!(world.sources.calls.borrow().is_empty());
        assert!(message(world.build(Some("b11538"), true).0).starts_with(REFUSED));
    }

    #[test]
    fn a_commit_names_its_build_only_once_checked_out() {
        let world = World::new();
        world.store.builds.borrow_mut().push(installed("b11538", 1));
        let text = message(world.build(Some("79e2e74"), false).0);
        assert!(text.contains("llama.cpp b11538 is already built"), "{text}");
        assert_eq!(
            world.sources.calls.borrow().as_slice(),
            ["fetch", "checkout 79e2e74"]
        );
        world.build(Some("b11538"), true).0.unwrap();
        assert!(world.store.replaced.get());
    }

    #[test]
    fn a_commit_built_before_its_release_was_tagged_is_built_under_the_tag() {
        let world = World::new();
        world
            .store
            .builds
            .borrow_mut()
            .push(installed("79e2e74eb1", 1));
        let built = world.build(Some("79e2e74eb1"), false).0.unwrap();
        assert_eq!(built.build.id, id("b11538"));
    }

    #[test]
    fn an_unreadable_directory_of_a_release_named_is_in_the_way_before_the_fetch() {
        let mut world = World::new();
        world.store.unreadable.push(not_a_build("b11538"));
        assert_eq!(
            message(world.build(Some("b11538"), false).0),
            "/data/builds/llama.cpp/b11538 is not a readable build: no build.json; `--force` \
             builds llama.cpp b11538 in its place"
        );
        assert!(world.sources.calls.borrow().is_empty());
    }

    #[test]
    fn an_unreadable_directory_of_the_same_name_is_in_the_way() {
        let mut world = World::new();
        world.store.unreadable.push(not_a_build("b11538"));
        let in_the_way = "/data/builds/llama.cpp/b11538 is not a readable build: no build.json; \
                          `--force` builds llama.cpp b11538 in its place";
        assert_eq!(message(world.build(None, false).0), in_the_way);
        *world.store.current.borrow_mut() = Some(id("b11538"));
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
            "llama.cpp b11538 cannot be built: its patch listen-failure.patch does not apply; \
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
        world.store.builds.borrow_mut().push(installed("b11538", 1));
        world.build(None, true).0.unwrap();
        let staged = PathBuf::from("/data/builds/llama.cpp/.staging/llama-server");
        assert_eq!(*world.programs.ran.borrow(), [staged.clone(), staged]);
    }

    #[test]
    fn a_build_whose_device_listing_fails_is_discarded() {
        let mut world = World::new();
        // The listing b11538 prints, then a crash.
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
            "cannot install /data/builds/llama.cpp/b11538: Permission denied (os error 13)"
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
        assert_eq!(result.unwrap().build.id, id("b11538"));
        assert_eq!(*world.store.current.borrow(), Some(id("b11538")));
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
            "llama.cpp b11538 is installed but not current: cannot update \
             /data/builds/llama.cpp/current: Permission denied (os error 13)"
        );
        assert_eq!(left(&world.store), ["b11538"]);
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
             build llama.cpp --ref b9 --force`"
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
            unreadable: vec![not_a_build("b11538")],
            ..FakeStore::default()
        };
        *store.current.borrow_mut() = Some(id("b11538"));
        list(&store, None, &mut |warning| warnings.push(warning)).unwrap();
        assert_eq!(
            warnings,
            [
                "the current build of llama.cpp, b9, is not installed; run `ai-ctl build \
                 llama.cpp`",
                "the current build of llama.cpp, b11538, cannot be read: no build.json; run \
                 `ai-ctl build llama.cpp --ref b11538 --force`",
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
        assert_eq!(*world.store.current.borrow(), Some(id("b11538")));
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
        world.store.builds.borrow_mut().push(installed("b11538", 1));
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
