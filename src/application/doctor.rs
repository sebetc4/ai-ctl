//! `doctor`: a read-only diagnosis of everything the stack needs, with a
//! remedy for each problem. It never installs anything.

use std::borrow::Cow;
use std::fmt::Write as _;
use std::path::{Path, PathBuf};

use std::collections::BTreeSet;

use anyhow::Result;

use crate::domain::diagnostics::{Check, Report, Section, Status};
use crate::domain::gateway::{OLDEST, PINNED, SwapStanding, swap_standing};
use crate::domain::gateway_config::{GPU_DEVICE, InvalidConfig};
use crate::domain::inventory::{Inventory, Model, UnsupportedVersion, built_projects};
use crate::domain::machine::{CdiSpec, Gpu, GpuProcess, Probed, Tool};
use crate::domain::recipe::{RECIPES, Recipe, recipe};
use crate::domain::safetensors;

use super::build::{self, Current, rebuild_remedy, smoke_test_failure, unlike_recipe};
use super::gateway_config::Configurator;
use super::lifecycle::{
    self, Configuration, Event, Inspection, Inspector, Serving, kill_command, models_list, named,
};
use super::ports::{
    BuildStore, CdiProbe, Containers, GatewayApi, GatewayProcess, GatewayStore, GpuProbe,
    InventoryStore, PortCheck, PowerProbe, ProcessTable, Programs, ToolchainProbe,
};
use super::toolchain::{self, fix};

/// The architecture native builds target.
const BLACKWELL: u32 = 12;
/// The device the gateway gives a model's container.
const CDI_ALL: &str = GPU_DEVICE;
/// Below this, a process holds a background context: in hybrid graphics the
/// compositor keeps about 4 MiB on the NVIDIA GPU, while a display running on
/// it holds hundreds of MiB.
const IDLE_VRAM_TOLERANCE: u64 = 64 << 20;
const POWER_PROFILE: &str = "performance";

const DRIVER_REMEDY: &str =
    "sudo dnf install akmod-nvidia xorg-x11-drv-nvidia-cuda   # from RPM Fusion, then reboot";
const DRIVER_FAILED_REMEDY: &str = "run `nvidia-smi`; after a driver update, reboot (with Secure Boot, enroll the akmod's MOK key)";
const VRAM_REMEDY: &str = "run the display on the integrated GPU (hybrid graphics) and close other GPU programs, so the models get the whole VRAM";
const CDI_REMEDY: &str = "sudo systemctl enable --now nvidia-cdi-refresh.path nvidia-cdi-refresh.service   # the toolkit then writes /var/run/cdi/nvidia.yaml at boot and when the kernel modules or nvidia-ctk change";
const CDI_REFRESH_REMEDY: &str = "sudo systemctl start nvidia-cdi-refresh.service   # writes /var/run/cdi/nvidia.yaml again from the libraries on disk";
const PROFILE_REMEDY: &str = "busctl set-property org.freedesktop.UPower.PowerProfiles /org/freedesktop/UPower/PowerProfiles org.freedesktop.UPower.PowerProfiles ActiveProfile s performance   # or choose Performance in the system menu";
const INIT_REMEDY: &str = "ai-ctl init --models-dir <dir>";
const SWAP_REMEDY: &str = "ai-ctl gateway install";

/// The doctor use case, with the probes it reads.
pub struct Doctor<'a> {
    pub gpu: &'a dyn GpuProbe,
    pub toolchain: &'a dyn ToolchainProbe,
    pub cdi: &'a dyn CdiProbe,
    pub power: &'a dyn PowerProbe,
    pub inventory: &'a dyn InventoryStore,
    pub builds: &'a dyn BuildStore,
    pub programs: &'a dyn Programs,
    pub releases: &'a dyn GatewayStore,
    pub gateway: &'a dyn GatewayProcess,
    pub processes: &'a dyn ProcessTable,
    pub ports: &'a dyn PortCheck,
    pub api: &'a dyn GatewayApi,
    pub containers: &'a dyn Containers,
}

impl Doctor<'_> {
    pub fn run(&self) -> Report {
        // Read once: every check judges the same GPU.
        let gpu = self.gpu.gpu();
        let inventory = self.inventory.load();
        // What the inspection could not read; the GPU check reports the GPU.
        let mut unread = Vec::new();
        let inspection = self.inspector().inspect(&gpu, &mut |event| {
            if let Event::Warning(warning) = event {
                unread.push(warning);
            }
        });
        let accounted: Vec<GpuProcess> = match &inspection {
            Ok(inspection) => {
                let status = &inspection.status;
                let held = status.gateway.iter().flat_map(|gateway| &gateway.held);
                held.chain(&status.leftovers).cloned().collect()
            }
            Err(_) => Vec::new(),
        };
        let gateway = self.gateway_checks(&gpu, &inventory, &inspection, &unread);
        Report {
            sections: vec![
                Section::new("GPU and driver", gpu_checks(&gpu, &accounted)),
                Section::new(
                    "Toolchain",
                    toolchain::inspect(self.toolchain, gpu.found()).checks,
                ),
                Section::new(
                    "Containers (image backends only)",
                    self.container_checks(gpu.found().map(|gpu| gpu.driver_version.as_str())),
                ),
                Section::new("Power", self.power_checks()),
                Section::new("Builds", self.build_checks(&inventory)),
                Section::new("Inventory", self.inventory_checks(inventory)),
                Section::new("Gateway", gateway),
            ],
        }
    }

    /// The generation use case on the doctor's ports.
    fn configurator(&self) -> Configurator<'_> {
        Configurator {
            inventory: self.inventory,
            builds: self.builds,
            gpu: self.gpu,
            files_dir: self.gateway.files_dir(),
        }
    }

    /// The gateway's inspection on the doctor's ports.
    fn inspector(&self) -> Inspector<'_> {
        Inspector {
            configurator: self.configurator(),
            releases: self.releases,
            gateway: self.gateway,
            processes: self.processes,
            api: self.api,
            containers: self.containers,
        }
    }

    /// llama-swap, the configuration `up` would write, the gateway's
    /// process, its address, what earlier gateways left on the GPU, and what
    /// of it all could not be read.
    fn gateway_checks(
        &self,
        gpu: &Probed<Gpu>,
        inventory: &Result<Option<Inventory>>,
        inspection: &Result<Inspection>,
        unread: &[String],
    ) -> Vec<Check> {
        let mut checks = vec![self.llama_swap_check(), self.config_check(gpu)];
        match inspection {
            Ok(inspection) => {
                checks.extend([
                    gateway_check(inspection),
                    self.address_check(inventory, inspection.status.gateway.as_ref()),
                    leftovers_check(inspection),
                ]);
                // What the checks above judged without: processes that could
                // not be listed, or the API's error.
                if !unread.is_empty() {
                    checks.push(Check::warn("Gateway inspection", unread.join("; "), None));
                }
            }
            Err(err) => checks.push(Check::fail("Gateway", format!("{err:#}"), None)),
        }
        checks
    }

    fn llama_swap_check(&self) -> Check {
        const NAME: &str = "llama-swap";
        let pinned = PINNED.release;
        let release = match self.releases.current() {
            Ok(Some(release)) => release,
            Ok(None) => {
                return Check::fail(
                    NAME,
                    "not installed; `ai-ctl up` needs it",
                    fix(SWAP_REMEDY),
                );
            }
            Err(err) => return Check::fail(NAME, format!("{err:#}"), fix(SWAP_REMEDY)),
        };
        match self.releases.is_installed(release) {
            // A release behind the pin or ahead of it: an older ai-ctl's
            // `--version`, a newer ai-ctl or a hand install may have left it.
            Ok(true) => match swap_standing(release) {
                SwapStanding::TooOld => Check::fail(
                    NAME,
                    format!(
                        "{release} is current, and cannot read the configuration this ai-ctl \
                         writes, which needs {OLDEST} or later"
                    ),
                    fix(SWAP_REMEDY),
                ),
                // `up` runs the pinned release only once checked against
                // the pinned digest, which an older ai-ctl did not record.
                SwapStanding::Pinned => match self.releases.digest(release) {
                    Ok(Some(digest)) if digest.to_string() == PINNED.sha256 => {
                        Check::ok(NAME, format!("{release}, the release this ai-ctl pins"))
                    }
                    Ok(_) => Check::fail(
                        NAME,
                        format!(
                            "{release} is current, but was never checked against the digest \
                             this ai-ctl pins for it"
                        ),
                        fix(SWAP_REMEDY),
                    ),
                    Err(err) => Check::fail(NAME, format!("{err:#}"), fix(SWAP_REMEDY)),
                },
                // `up` runs another release only once an ai-ctl that pins
                // it recorded its check, which an older ai-ctl's
                // `--version` never did.
                standing @ (SwapStanding::Behind | SwapStanding::Ahead) => {
                    match self.releases.digest(release) {
                        Ok(Some(_)) if standing == SwapStanding::Behind => Check::warn(
                            NAME,
                            format!(
                                "{release} is current, behind {pinned}, the release this ai-ctl pins"
                            ),
                            fix(SWAP_REMEDY),
                        ),
                        Ok(Some(_)) => Check::warn(
                            NAME,
                            format!(
                                "{release} is current, ahead of {pinned}, the release this ai-ctl \
                             pins and was tested with"
                            ),
                            fix(format!(
                                "{SWAP_REMEDY}   # back to {pinned}; or run an ai-ctl that pins \
                             {release}"
                            )),
                        ),
                        Ok(None) => Check::fail(
                            NAME,
                            format!(
                                "{release} is current, but no ai-ctl checked it against a digest it \
                             pins for it"
                            ),
                            fix(SWAP_REMEDY),
                        ),
                        Err(err) => Check::fail(NAME, format!("{err:#}"), fix(SWAP_REMEDY)),
                    }
                }
            },
            Ok(false) => Check::fail(
                NAME,
                format!("{release} is current, but its binary is missing"),
                fix(SWAP_REMEDY),
            ),
            Err(err) => Check::fail(NAME, format!("{err:#}"), fix(SWAP_REMEDY)),
        }
    }

    /// Whether the configuration `up` writes can be generated now. Only its
    /// own refusals fail here: a missing inventory, build or GPU fails a
    /// check of its own, and skips this one.
    fn config_check(&self, gpu: &Probed<Gpu>) -> Check {
        const NAME: &str = "Gateway configuration";
        match self.configurator().prepare_on(gpu) {
            Ok(prepared) => Check::ok(
                NAME,
                format!(
                    "{} models, on {}",
                    prepared.config.models.len(),
                    prepared.listen
                ),
            ),
            Err(err) if err.downcast_ref::<InvalidConfig>().is_some() => Check::fail(
                NAME,
                format!("{err:#}"),
                fix(format!("fix {}", self.inventory.location().display())),
            ),
            Err(_) => Check::skip(NAME, "needs the inventory, its builds and the GPU"),
        }
    }

    /// Whether `up` could listen on the inventory's address.
    fn address_check(
        &self,
        inventory: &Result<Option<Inventory>>,
        gateway: Option<&Serving>,
    ) -> Check {
        const NAME: &str = "Gateway address";
        let Ok(Some(inventory)) = inventory else {
            return Check::skip(NAME, "needs the inventory");
        };
        let listen = inventory.gateway().listen;
        if gateway.is_some_and(|gateway| gateway.listen == listen) {
            return Check::ok(NAME, format!("{listen}, where the gateway listens"));
        }
        match self.ports.is_free(listen) {
            Ok(true) => Check::ok(NAME, format!("{listen} free")),
            Ok(false) => Check::fail(
                NAME,
                format!("{listen} is taken by another program; `ai-ctl up` would refuse to start"),
                fix("stop it, or give the gateway another `listen` address in the inventory"),
            ),
            Err(err) => Check::fail(
                NAME,
                format!("{err:#}"),
                fix("give the gateway another `listen` address in the inventory"),
            ),
        }
    }

    /// Podman, the toolkit, and the CDI specification that gives a
    /// container the GPU and the libraries of `driver`, the one loaded.
    fn container_checks(&self, driver: Option<&str>) -> Vec<Check> {
        let podman = match self.toolchain.tool_version(Tool::Podman) {
            Probed::Found(version) => Check::ok("Podman", version.to_string()),
            Probed::Missing => {
                Check::warn("Podman", "podman not found", fix(Tool::Podman.install()))
            }
            Probed::Failed(err) => Check::warn("Podman", format!("podman: {err}"), None),
        };

        const CTK: &str = "NVIDIA Container Toolkit";
        let ctk = self.toolchain.tool_version(Tool::NvidiaCtk);
        let ctk_check = match &ctk {
            Probed::Found(version) => Check::ok(CTK, version.to_string()),
            Probed::Missing => {
                Check::warn(CTK, "nvidia-ctk not found", fix(Tool::NvidiaCtk.install()))
            }
            Probed::Failed(err) => Check::warn(CTK, format!("nvidia-ctk: {err}"), None),
        };

        const CDI: &str = "CDI specification";
        let cdi = match ctk {
            Probed::Found(_) => match self.cdi.cdi_devices() {
                Probed::Found(devices) if devices.iter().any(|d| d == CDI_ALL) => {
                    cdi_spec_check(CDI, self.cdi.cdi_specs(), driver)
                }
                Probed::Found(devices) if !devices.is_empty() => Check::warn(
                    CDI,
                    format!("{CDI_ALL} not listed (found {})", devices.join(", ")),
                    fix(CDI_REMEDY),
                ),
                Probed::Found(_) | Probed::Missing => {
                    Check::warn(CDI, "no CDI device listed", fix(CDI_REMEDY))
                }
                Probed::Failed(err) => {
                    Check::warn(CDI, format!("nvidia-ctk cdi list: {err}"), fix(CDI_REMEDY))
                }
            },
            _ => Check::skip(CDI, "needs nvidia-ctk"),
        };

        vec![podman, ctk_check, cdi]
    }

    fn power_checks(&self) -> Vec<Check> {
        const AC: &str = "AC power";
        let ac = match self.power.ac_power() {
            Probed::Found(true) => Check::ok(AC, "adapter online"),
            Probed::Found(false) => Check::warn(
                AC,
                "on battery: the GPU runs at a reduced power limit",
                fix("plug in the AC adapter"),
            ),
            Probed::Missing => Check::skip(AC, "no external power supply reported"),
            Probed::Failed(err) => Check::warn(AC, err, None),
        };

        const PROFILE: &str = "Power profile";
        let profile = match self.power.power_profile() {
            Probed::Found(profile) if profile == POWER_PROFILE => Check::ok(PROFILE, profile),
            Probed::Found(profile) => Check::warn(
                PROFILE,
                format!("`{profile}`; generation and benchmarks need `{POWER_PROFILE}`"),
                fix(PROFILE_REMEDY),
            ),
            Probed::Missing => Check::skip(
                PROFILE,
                "no power profile service (power-profiles-daemon or tuned-ppd)",
            ),
            Probed::Failed(err) => Check::warn(PROFILE, err, None),
        };

        vec![ac, profile]
    }

    /// Each project's current build runs. A project a model runs on needs
    /// one; the others are reported when they have one.
    fn build_checks(&self, inventory: &Result<Option<Inventory>>) -> Vec<Check> {
        // The projects the inventory runs, a model's backend being a build of
        // each, as `up` resolves them; none known when it could not be read.
        let used: Option<BTreeSet<&str>> = match inventory {
            Ok(Some(inventory)) => Some(built_projects(
                inventory
                    .models()
                    .iter()
                    .filter_map(|model| inventory.backend(&model.backend)),
            )),
            _ => None,
        };
        let mut checks: Vec<Check> = RECIPES
            .iter()
            .map(|recipe| {
                let runs = used.as_ref().map(|used| used.contains(recipe.project));
                self.build_check(recipe, runs)
            })
            .collect();
        let unknown = used
            .iter()
            .flatten()
            .filter(|project| recipe(project).is_none());
        checks.extend(unknown.map(|project| {
            Check::fail(
                *project,
                format!("the inventory runs a build of {project}, which ai-ctl has no recipe for"),
                None,
            )
        }));
        checks
    }

    /// `used`: whether the inventory runs `recipe`'s project, `None` when it
    /// could not be read.
    fn build_check(&self, recipe: &Recipe, used: Option<bool>) -> Check {
        let project = recipe.project;
        let status = if used == Some(true) {
            Status::Fail
        } else {
            Status::Warn
        };
        let problem = |detail: String, remedy: String| {
            let remedy = Some(remedy);
            match status {
                Status::Fail => Check::fail(project, detail, remedy),
                _ => Check::warn(project, detail, remedy),
            }
        };
        let build_remedy = format!("ai-ctl build {project}");
        let build = match build::current(self.builds, project) {
            Ok(Current::Installed(build)) => build,
            Ok(Current::Missing(current)) => {
                return problem(
                    format!("the current build, {current}, is not installed"),
                    format!("ai-ctl build use {project} <id>   # or: {build_remedy}"),
                );
            }
            Ok(Current::Unreadable { id, reason }) => {
                return problem(
                    format!("the current build, {id}, cannot be read: {reason}"),
                    rebuild_remedy(project, &id),
                );
            }
            Ok(Current::None) if used == Some(true) => {
                return problem(
                    "no build, and the inventory runs one".to_owned(),
                    build_remedy,
                );
            }
            Ok(Current::None) if used == Some(false) => {
                return Check::skip(project, "no build; the inventory does not run one");
            }
            Ok(Current::None) => {
                return Check::skip(project, "no build; the inventory could not be read");
            }
            Err(err) => return problem(format!("{err:#}"), build_remedy),
        };
        let current = &build.id;
        let rebuild = rebuild_remedy(project, current);
        if let Some(unlike) = unlike_recipe(recipe, &build) {
            return problem(unlike, rebuild);
        }
        let binary = self.builds.binary(project, current, recipe.binary());
        match self.programs.run(&binary, recipe.smoke_test.args) {
            Ok(output) => match smoke_test_failure(recipe, &output) {
                None => Check::ok(
                    project,
                    format!(
                        "{current}, built {} with CUDA {}",
                        build.built_at.minutes(),
                        build.toolkit
                    ),
                ),
                Some(failure) => problem(format!("{current} {failure}"), rebuild),
            },
            Err(err) => problem(format!("{current}: {err:#}"), rebuild),
        }
    }

    fn inventory_checks(&self, inventory: Result<Option<Inventory>>) -> Vec<Check> {
        const NAME: &str = "Inventory";
        let location = self.inventory.location().display();
        match inventory {
            Ok(Some(inventory)) => {
                let summary = format!(
                    "{location}: {} models, {} backends",
                    inventory.models().len(),
                    inventory.backends().len()
                );
                std::iter::once(Check::ok(NAME, summary))
                    .chain(
                        inventory
                            .models()
                            .iter()
                            .map(|model| self.model_check(&inventory, model)),
                    )
                    .collect()
            }
            Ok(None) => vec![Check::fail(
                NAME,
                format!("{location} not found"),
                fix(INIT_REMEDY),
            )],
            Err(err)
                if err
                    .downcast_ref::<UnsupportedVersion>()
                    .is_some_and(|version| version.found > version.reads) =>
            {
                vec![Check::fail(
                    NAME,
                    format!("{err:#}"),
                    fix(format!(
                        "{location} is written for a newer ai-ctl: install that one, or move \
                         the file away and run `{INIT_REMEDY}`"
                    )),
                )]
            }
            Err(err) => vec![Check::fail(
                NAME,
                format!("{err:#}"),
                fix(format!(
                    "fix {location}, or move it away and run `{INIT_REMEDY}`"
                )),
            )],
        }
    }

    /// A model's files, or its directory when it names none, which
    /// audiocpp_server reads whole and which must hold more than hf's
    /// `.cache`, the shards of each safetensors index its server reads, and
    /// its voices' directory.
    fn model_check(&self, inventory: &Inventory, model: &Model) -> Check {
        let name = format!("Model {}", model.id);
        let dir = inventory.model_dir(model);
        let kind = inventory
            .backend(&model.backend)
            .map(|backend| backend.kind);
        let voices = model
            .audiocpp
            .voices
            .as_ref()
            .map(|voices| inventory.models_dir().join(voices));
        let mut missing = Vec::new();
        // Each index that cannot be used, with why.
        let mut unusable = Vec::new();
        let mut shards = 0;
        for (flag, file) in &model.files {
            if !self.toolchain.is_file(&dir.join(file)) {
                missing.push(file.display().to_string());
                continue;
            }
            if !kind.is_some_and(|kind| kind.reads_as_index(flag, file)) {
                continue;
            }
            match self.index_shards(&dir, file) {
                Ok(named) => {
                    for shard in named {
                        if self.toolchain.is_file(&dir.join(&shard)) {
                            shards += 1;
                        } else {
                            missing.push(shard.display().to_string());
                        }
                    }
                }
                Err(why) => unusable.push((file.display().to_string(), why)),
            }
        }
        let no_dir = model.files.is_empty() && !self.toolchain.is_dir(&dir);
        let no_voices = voices
            .as_ref()
            .filter(|voices| !self.toolchain.is_dir(voices));

        let mut details = Vec::new();
        let mut remedies = Vec::new();
        if no_dir {
            details.push(format!("missing: {}", dir.display()));
            remedies.push(download(model, &dir, &[]));
        } else if model.files.is_empty() {
            // `hf download --local-dir` keeps its records in `.cache`.
            match self.toolchain.entries(&dir) {
                Ok(names) if names.iter().all(|name| name == ".cache") => {
                    details.push(format!("empty: {}", dir.display()));
                    remedies.push(download(model, &dir, &[]));
                }
                Ok(_) => {}
                Err(err) => details.push(format!("{err:#}")),
            }
        }
        if !missing.is_empty() {
            details.push(format!(
                "missing in {}: {}",
                dir.display(),
                missing.join(", ")
            ));
        }
        details.extend(unusable.iter().map(|(_, why)| why.clone()));
        // An index that cannot be used is fetched again with the files.
        let fetch: Vec<String> = missing
            .into_iter()
            .chain(unusable.into_iter().map(|(file, _)| file))
            .collect();
        if !fetch.is_empty() {
            remedies.push(download(model, &dir, &fetch));
        }
        if let Some(voices) = no_voices {
            details.push(format!(
                "missing: the voices' directory {}",
                voices.display()
            ));
            remedies.push(format!(
                "create {}, each voice a <voice>.wav with its line <voice>|<transcript> in {}",
                voices.display(),
                voices.join("prompt_text").display()
            ));
        }
        if !details.is_empty() {
            // A directory that cannot be read has no remedy ai-ctl knows.
            let remedy = (!remedies.is_empty()).then(|| remedies.join("; "));
            return Check::fail(name, details.join("; "), remedy);
        }
        let found = match (model.files.len(), shards) {
            (0, _) => format!("the directory {}", dir.display()),
            (files, 0) => format!("{} in {}", counted(files, "file"), dir.display()),
            (files, shards) => format!(
                "{} and {} in {}",
                counted(files, "file"),
                counted(shards, "shard"),
                dir.display()
            ),
        };
        match voices {
            Some(voices) => Check::ok(name, format!("{found}, voices in {}", voices.display())),
            None => Check::ok(name, found),
        }
    }

    /// The shards the safetensors index `file` of the model's `dir` names,
    /// each relative to `dir`; why the index cannot be used otherwise.
    fn index_shards(&self, dir: &Path, file: &Path) -> Result<Vec<PathBuf>, String> {
        let index = dir.join(file);
        let text = self
            .toolchain
            .read_text(&index)
            .map_err(|err| format!("{err:#}"))?;
        let names = safetensors::shards(&text)
            .map_err(|err| format!("{} is not a safetensors index: {err}", index.display()))?;
        // The shard's path is normal, as the server reads it: so is the
        // directory it is compared with.
        let normal_dir = safetensors::normal(dir);
        names
            .iter()
            .map(|name| {
                let shard = safetensors::shard_path(&index, name);
                let relative = shard.strip_prefix(&normal_dir).map_err(|_| {
                    format!(
                        "{} names {}, outside {}",
                        file.display(),
                        shard.display(),
                        dir.display()
                    )
                })?;
                // The download command names it as it is, after the repository.
                if relative.as_os_str().as_encoded_bytes().starts_with(b"-") {
                    return Err(format!(
                        "{} names {}, which hf download would read as an option",
                        file.display(),
                        relative.display()
                    ));
                }
                Ok(relative.to_path_buf())
            })
            .collect()
    }
}

/// The command that puts `model`'s files `names` into `dir`, or the whole
/// repository when `names` is empty. The download is pinned and checked as
/// docs/stack.md says; ai-ctl itself neither downloads nor verifies a model.
fn download(model: &Model, dir: &Path, names: &[String]) -> String {
    match &model.repo {
        Some(repo) => {
            let mut words = vec![shell_quote(repo)];
            words.extend(names.iter().map(|name| shell_quote(name)));
            format!(
                "hf download {} --revision <commit> --local-dir {}, at a commit you pin, \
                 then check the files as docs/stack.md says under \"Where Models Come From\"",
                words.join(" "),
                shell_quote(&dir.display().to_string())
            )
        }
        None if names.is_empty() => format!("copy the model's files into {}", dir.display()),
        None => format!("copy {} into {}", names.join(", "), dir.display()),
    }
}

/// Whether the gateway runs, and what it serves.
fn gateway_check(inspection: &Inspection) -> Check {
    const NAME: &str = "Gateway";
    let Some(gateway) = &inspection.status.gateway else {
        return match &inspection.stale {
            Some((pid, liveness)) => Check::warn(
                NAME,
                format!(
                    "stopped; the gateway recorded with PID {pid} {}",
                    liveness.describe(*pid)
                ),
                fix("ai-ctl status   # removes the record"),
            ),
            None => Check::ok(NAME, "stopped; `ai-ctl up` starts it"),
        };
    };
    let at = gateway.running_on();
    match &gateway.configuration {
        Configuration::Current => {}
        Configuration::Outdated => {
            return Check::warn(
                NAME,
                format!(
                    "{at}, on another configuration, address or release than the inventory, \
                     the builds and llama-swap give now"
                ),
                fix("ai-ctl down && ai-ctl up"),
            );
        }
        Configuration::Unknown(reason) => {
            return Check::warn(
                NAME,
                format!(
                    "{at}, but whether it runs on the configuration the inventory gives \
                     cannot be told: {reason}"
                ),
                fix("ai-ctl status"),
            );
        }
    }
    match gateway.models.as_deref() {
        None => Check::warn(
            NAME,
            format!("{at}, but its API does not answer"),
            fix("read its log: ai-ctl logs"),
        ),
        Some([]) => Check::ok(NAME, format!("{at}, no model loaded")),
        Some(models) => Check::ok(NAME, format!("{at}: {}", models_list(models))),
    }
}

/// The processes earlier gateways left on the GPU, which `up` refuses to
/// start beside.
fn leftovers_check(inspection: &Inspection) -> Check {
    const NAME: &str = "GPU leftovers";
    let status = &inspection.status;
    if status.gpu.is_none() {
        return Check::skip(NAME, "the GPU could not be read");
    }
    if status.leftovers.is_empty() {
        return Check::ok(NAME, "none");
    }
    Check::fail(
        NAME,
        format!(
            "left by an earlier gateway: {}; `ai-ctl up` refuses to start until they are gone",
            named(&status.leftovers)
        ),
        fix(format!(
            "{}   # ai-ctl never stops a backend itself",
            kill_command(&status.leftovers)
        )),
    )
}

/// The GPU and driver checks; the first one, "GPU", also gates a build.
/// `accounted` are the processes the Gateway section reports, which the
/// idle VRAM leaves out.
pub(super) fn gpu_checks(gpu: &Probed<Gpu>, accounted: &[GpuProcess]) -> Vec<Check> {
    let unavailable = |reason: &str| {
        [
            Check::skip("Driver", reason),
            Check::skip("Idle VRAM", reason),
        ]
    };
    match gpu {
        Probed::Found(gpu) => vec![
            gpu_check(gpu),
            driver_check(gpu),
            vram_check(gpu, accounted),
        ],
        Probed::Missing => {
            let mut checks = vec![Check::fail(
                "GPU",
                "NVML not found: the NVIDIA driver is not installed",
                fix(DRIVER_REMEDY),
            )];
            checks.extend(unavailable("no NVIDIA driver"));
            checks
        }
        Probed::Failed(err) => {
            let mut checks = vec![Check::fail(
                "GPU",
                format!("NVML: {err}"),
                fix(DRIVER_FAILED_REMEDY),
            )];
            checks.extend(unavailable("the GPU could not be read"));
            checks
        }
    }
}

fn gpu_check(gpu: &Gpu) -> Check {
    let (major, minor) = gpu.compute_capability;
    let detail = format!(
        "{}, compute {major}.{minor}, {}",
        gpu.name,
        gib(gpu.memory_total)
    );
    if major == BLACKWELL {
        Check::ok("GPU", detail)
    } else {
        Check::warn(
            "GPU",
            format!("{detail}; native builds target sm_120a (Blackwell)"),
            None,
        )
    }
}

fn driver_check(gpu: &Gpu) -> Check {
    Check::ok(
        "Driver",
        format!(
            "{}, CUDA {}.{}",
            gpu.driver_version, gpu.cuda_version.major, gpu.cuda_version.minor
        ),
    )
}

fn vram_check(gpu: &Gpu, accounted: &[GpuProcess]) -> Check {
    let (holders, background): (Vec<GpuProcess>, Vec<GpuProcess>) = gpu
        .processes
        .iter()
        .filter(|process| !accounted.iter().any(|other| other.pid == process.pid))
        .cloned()
        .partition(|process| {
            process
                .memory
                .is_none_or(|bytes| bytes >= IDLE_VRAM_TOLERANCE)
        });
    if holders.is_empty() {
        let mut detail = format!("{} in use", mib(gpu.memory_used));
        if !accounted.is_empty() {
            let _ = write!(
                detail,
                ", {} MiB of it by the processes the Gateway section reports",
                lifecycle::mib(accounted)
            );
        }
        if !background.is_empty() {
            let _ = write!(
                detail,
                "; under {} each: {}",
                mib(IDLE_VRAM_TOLERANCE),
                named(&background)
            );
        }
        return Check::ok("Idle VRAM", detail);
    }
    Check::warn(
        "Idle VRAM",
        format!("held by {}", named(&holders)),
        fix(VRAM_REMEDY),
    )
}

/// With `nvidia.com/gpu=all` listed: whether the specification podman uses,
/// the last of `specs`, mounts the libraries of `driver` and only paths on
/// disk, and whether one it shadows names another driver. A shadowed
/// specification comes back into use whenever the one in use goes missing:
/// `/var/run` is emptied at each boot. The toolkit writes its specification
/// again when the kernel modules change, not when dnf upgrades a library it
/// mounts.
fn cdi_spec_check(name: &str, specs: Probed<Vec<CdiSpec>>, driver: Option<&str>) -> Check {
    let listed = format!("{CDI_ALL} listed");
    let Probed::Found(specs) = specs else {
        return Check::ok(name, listed);
    };
    let Some((in_use, shadowed)) = specs.split_last() else {
        return Check::ok(name, listed);
    };
    let version = in_use.driver.as_deref();
    if let (Some(driver), Some(version)) = (driver, version)
        && version != driver
    {
        return Check::warn(
            name,
            format!(
                "{} mounts the libraries of driver {version}, but driver {driver} is loaded",
                in_use.path.display()
            ),
            fix(CDI_REMEDY),
        );
    }
    if let Some(first) = in_use.missing.first() {
        let spec = in_use.path.display();
        let first = first.display();
        let gone = match in_use.missing.len() {
            1 => format!("{spec} mounts {first}, which is not on disk"),
            n => format!("{spec} mounts {n} paths that are not on disk, {first} first"),
        };
        return Check::warn(
            name,
            format!("{gone}: podman starts no container with the GPU"),
            fix(CDI_REFRESH_REMEDY),
        );
    }
    let (Some(driver), Some(_)) = (driver, version) else {
        return Check::ok(name, listed);
    };
    let stale = shadowed.iter().find_map(|spec| {
        let version = spec.driver.as_deref()?;
        (version != driver).then_some((spec, version))
    });
    if let Some((spec, version)) = stale {
        return Check::warn(
            name,
            format!(
                "{} names driver {version}; {} shadows it now, but it comes back into use if \
                 that one goes missing",
                spec.path.display(),
                in_use.path.display()
            ),
            fix(format!(
                "sudo rm {}   # {} is the specification to keep current",
                shell_quote(&spec.path.to_string_lossy()),
                in_use.path.display()
            )),
        );
    }
    Check::ok(
        name,
        format!(
            "{listed}, from {} for driver {driver}",
            in_use.path.display()
        ),
    )
}

fn gib(bytes: u64) -> String {
    format!("{:.1} GiB", bytes as f64 / f64::from(1u32 << 30))
}

fn mib(bytes: u64) -> String {
    format!("{} MiB", bytes >> 20)
}

/// `count` of `noun`, in words: `1 file`, `3 files`.
fn counted(count: usize, noun: &str) -> String {
    if count == 1 {
        format!("1 {noun}")
    } else {
        format!("{count} {noun}s")
    }
}

/// Quotes `word` for a POSIX shell when it holds anything but safe characters.
fn shell_quote(word: &str) -> Cow<'_, str> {
    let safe = !word.is_empty()
        && word
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || "_-./:=@%+,".contains(c));
    if safe {
        Cow::Borrowed(word)
    } else {
        Cow::Owned(format!("'{}'", word.replace('\'', r"'\''")))
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::path::PathBuf;

    use super::*;
    use crate::application::fakes::{
        FakeApi, FakeBuilds, FakeCdi, FakeContainers, FakeGateway, FakeGpu,
        FakeInventory as FakeStore, FakePorts, FakePower, FakeProcesses, FakePrograms,
        FakeReleases, FakeToolchain, LoadFailure, OTHER_DIGEST, ScriptedGpu, advance, b11538, gpu,
        gxx, inventory, inventory_parts, model, now, printed, record, toolkit,
    };
    use crate::application::lifecycle::Lifecycle;
    use crate::application::ports::{Liveness, RunningModel};
    use crate::domain::build::Build;
    use crate::domain::diagnostics::Status;
    use crate::domain::inventory::{
        AudiocppFields, Backend, BackendKind, BackendSource, FlagValue, Gateway, InventoryParts,
    };
    use crate::domain::machine::Version;

    /// The host's probes, which doctor reads all of.
    struct Host {
        toolchain: FakeToolchain,
        cdi: FakeCdi,
        power: FakePower,
    }

    /// A machine where every check passes.
    fn healthy_host() -> Host {
        let dir = Path::new("/models/qwen3.8-27b");
        let mut toolchain = FakeToolchain::healthy();
        toolchain
            .files
            .extend(model().files.values().map(|file| dir.join(file)));
        Host {
            toolchain,
            cdi: FakeCdi::healthy(),
            power: FakePower::healthy(),
        }
    }

    fn diagnose(gpu: Probed<Gpu>, host: &Host, store: FakeStore) -> Report {
        diagnose_builds(
            gpu,
            host,
            store,
            &FakeBuilds::current_b11538(),
            &FakePrograms::default(),
        )
    }

    fn diagnose_builds(
        gpu: Probed<Gpu>,
        host: &Host,
        store: FakeStore,
        builds: &FakeBuilds,
        programs: &FakePrograms,
    ) -> Report {
        Doctor {
            gpu: &FakeGpu(gpu),
            toolchain: &host.toolchain,
            cdi: &host.cdi,
            power: &host.power,
            inventory: &store,
            builds,
            programs,
            releases: &FakeReleases::current("v271"),
            gateway: &FakeGateway::default(),
            processes: &FakeProcesses::default(),
            ports: &FakePorts::default(),
            api: &FakeApi::answering_after(0),
            containers: &FakeContainers::default(),
        }
        .run()
    }

    /// The gateway's side of the machine: llama-swap v271 installed, and no
    /// gateway recorded until a test records one.
    struct GatewaySide {
        gpu: Probed<Gpu>,
        inventory: FakeStore,
        releases: FakeReleases,
        gateway: FakeGateway,
        processes: FakeProcesses,
        ports: FakePorts,
        api: FakeApi,
        containers: FakeContainers,
    }

    impl GatewaySide {
        fn new() -> Self {
            Self {
                gpu: Probed::Found(gpu()),
                inventory: installed(),
                releases: FakeReleases::current("v271"),
                gateway: FakeGateway::default(),
                processes: FakeProcesses::default(),
                ports: FakePorts::default(),
                api: FakeApi::answering_after(0),
                containers: FakeContainers::default(),
            }
        }

        /// The gateway PID 1234 running the model PID 5000, on the
        /// configuration the inventory gives.
        fn serving() -> Self {
            let mut side = Self::new();
            *side.gateway.record.borrow_mut() = Some(record(1234));
            side.processes.spawned(&[(5000, "llama-server", 1234)]);
            side.gpu = with(&[(5000, "llama-server", 21094)]);
            let config = Configurator {
                inventory: &installed(),
                builds: &FakeBuilds::current_b11538(),
                gpu: &FakeGpu(side.gpu.clone()),
                files_dir: Path::new(crate::application::fakes::GATEWAY_DIR),
            }
            .generate()
            .unwrap()
            .to_yaml();
            *side.gateway.config.borrow_mut() = Some(config);
            side
        }

        fn report(&self) -> Report {
            self.report_on(&FakeGpu(self.gpu.clone()))
        }

        /// The report, the GPU read through `gpu`.
        fn report_on(&self, gpu: &dyn GpuProbe) -> Report {
            let host = healthy_host();
            Doctor {
                gpu,
                toolchain: &host.toolchain,
                cdi: &host.cdi,
                power: &host.power,
                inventory: &self.inventory,
                builds: &FakeBuilds::current_b11538(),
                programs: &FakePrograms::default(),
                releases: &self.releases,
                gateway: &self.gateway,
                processes: &self.processes,
                ports: &self.ports,
                api: &self.api,
                containers: &self.containers,
            }
            .run()
        }

        fn check(&self, name: &str) -> Check {
            check(&self.report(), name).clone()
        }
    }

    /// The idle GPU, with these processes on it.
    fn with(processes: &[(u32, &str, u64)]) -> Probed<Gpu> {
        let mut gpu = gpu();
        gpu.processes = processes
            .iter()
            .map(|&(pid, name, mib)| GpuProcess {
                pid,
                name: name.into(),
                memory: Some(mib << 20),
            })
            .collect();
        Probed::Found(gpu)
    }

    fn builds_check(builds: &FakeBuilds, programs: &FakePrograms, store: FakeStore) -> Check {
        let report = diagnose_builds(
            Probed::Found(gpu()),
            &healthy_host(),
            store,
            builds,
            programs,
        );
        check(&report, "llama.cpp").clone()
    }

    #[test]
    fn the_current_build_runs() {
        let check = builds_check(
            &FakeBuilds::current_b11538(),
            &FakePrograms::default(),
            installed(),
        );
        assert_eq!(check.status, Status::Ok);
        assert_eq!(
            check.detail,
            "b11538, built 2026-10-09 20:48 UTC with CUDA 13.4.92"
        );
    }

    #[test]
    fn a_current_build_that_prints_not_what_its_recipe_expects_fails() {
        let programs = FakePrograms {
            version: printed(true, "", "usage: llama-server [options]\n"),
            ..FakePrograms::default()
        };
        let check = builds_check(&FakeBuilds::current_b11538(), &programs, installed());
        assert_eq!(check.status, Status::Fail);
        assert_eq!(
            check.detail,
            "b11538 is not the server its recipe builds: `llama-server --version` printed no \
             `version: `: usage: llama-server [options]"
        );
        assert_eq!(
            check.remedy.as_deref(),
            Some(
                "`ai-ctl build llama.cpp --ref b11538 --force`, or `ai-ctl build llama.cpp` at the \
                 newest release if the project withdrew b11538"
            )
        );
    }

    #[test]
    fn each_projects_current_build_runs_its_own_smoke_test() {
        let mut builds = FakeBuilds::current_b11538();
        let v1_9_5 = crate::domain::build::BuildId::parse("v1.9.5").unwrap();
        builds.builds.push(Build {
            project: "whisper.cpp".into(),
            cmake_flags: crate::application::fakes::recipe_flags(
                &crate::domain::recipe::WHISPER_CPP,
            ),
            id: v1_9_5.clone(),
            patches: vec!["api-key.patch".into(), "reuse-address.patch".into()],
            ..b11538()
        });
        builds.other_currents.insert("whisper.cpp".into(), v1_9_5);
        let report = diagnose_builds(
            Probed::Found(gpu()),
            &healthy_host(),
            installed(),
            &builds,
            &FakePrograms::default(),
        );
        let whisper = check(&report, "whisper.cpp");
        assert_eq!(whisper.status, Status::Ok);
        assert_eq!(
            whisper.detail,
            "v1.9.5, built 2026-10-09 20:48 UTC with CUDA 13.4.92"
        );
        assert_eq!(check(&report, "stable-diffusion.cpp").status, Status::Skip);

        // whisper-server exits 0 whatever its arguments: its help must name
        // its option. The inventory does not run it: a warning.
        let programs = FakePrograms {
            help: printed(true, "", "error: unknown argument: --help\n"),
            ..FakePrograms::default()
        };
        let report = diagnose_builds(
            Probed::Found(gpu()),
            &healthy_host(),
            installed(),
            &builds,
            &programs,
        );
        let whisper = check(&report, "whisper.cpp");
        assert_eq!(whisper.status, Status::Warn);
        assert_eq!(
            whisper.detail,
            "v1.9.5 is not the server its recipe builds: `whisper-server --help` printed no \
             `--inference-path`: error: unknown argument: --help"
        );
        assert_eq!(
            whisper.remedy.as_deref(),
            Some(
                "`ai-ctl build whisper.cpp --ref v1.9.5 --force`, or `ai-ctl build whisper.cpp` at \
                 the newest release if the project withdrew v1.9.5"
            )
        );
        assert_eq!(check(&report, "llama.cpp").status, Status::Ok);
    }

    #[test]
    fn a_current_build_without_one_of_its_recipes_patches_is_built_again() {
        let mut builds = FakeBuilds::current_b11538();
        let v1_9_5 = crate::domain::build::BuildId::parse("v1.9.5").unwrap();
        builds.builds.push(Build {
            project: "whisper.cpp".into(),
            cmake_flags: crate::application::fakes::recipe_flags(
                &crate::domain::recipe::WHISPER_CPP,
            ),
            id: v1_9_5.clone(),
            patches: vec!["api-key.patch".into()],
            ..b11538()
        });
        builds.other_currents.insert("whisper.cpp".into(), v1_9_5);
        let report = diagnose_builds(
            Probed::Found(gpu()),
            &healthy_host(),
            installed(),
            &builds,
            &FakePrograms::default(),
        );
        let whisper = check(&report, "whisper.cpp");
        assert_eq!(whisper.status, Status::Warn);
        assert_eq!(
            whisper.detail,
            "v1.9.5 was built without reuse-address.patch, which its recipe applies"
        );
        assert_eq!(
            whisper.remedy.as_deref(),
            Some(
                "`ai-ctl build whisper.cpp --ref v1.9.5 --force`, or `ai-ctl build whisper.cpp` at \
                 the newest release if the project withdrew v1.9.5"
            )
        );
    }

    #[test]
    fn a_nightly_build_of_llama_cpp_passes_as_ai_ctl_0_3_0_built_it() {
        // `b12359`'s `build.json`, built on 2026-11-14 by ai-ctl 0.3.0, before
        // its recipe set `LLAMA_BUILD_IS_DEV=OFF` for a release.
        let flags = [
            ("BUILD_SHARED_LIBS", "OFF"),
            ("CMAKE_BUILD_TYPE", "Release"),
            ("CMAKE_CUDA_ARCHITECTURES", "120a-real"),
            ("CMAKE_CUDA_COMPILER", "/usr/local/cuda-13.5/bin/nvcc"),
            ("CMAKE_CUDA_HOST_COMPILER", "/usr/bin/g++"),
            ("CMAKE_CXX_COMPILER", "/usr/bin/g++"),
            ("CMAKE_C_COMPILER", "/usr/bin/gcc"),
            ("GGML_CUDA", "ON"),
            ("GGML_NATIVE", "ON"),
            ("LLAMA_BUILD_APP", "OFF"),
            ("LLAMA_BUILD_EXAMPLES", "OFF"),
            ("LLAMA_BUILD_SERVER", "ON"),
            ("LLAMA_BUILD_TESTS", "OFF"),
            ("LLAMA_BUILD_TOOLS", "ON"),
            ("LLAMA_USE_PREBUILT_UI", "OFF"),
        ];
        let b12359 = crate::domain::build::BuildId::parse("b12359").unwrap();
        let builds = FakeBuilds {
            builds: vec![Build {
                id: b12359.clone(),
                cmake_flags: flags
                    .map(|(name, value)| (name.to_owned(), value.to_owned()))
                    .into(),
                ..b11538()
            }],
            current: Some(b12359),
            ..FakeBuilds::default()
        };
        let check = builds_check(&builds, &FakePrograms::default(), installed());
        assert_eq!(check.status, Status::Ok, "{}", check.detail);
    }

    #[test]
    fn a_current_build_without_one_of_its_recipes_cmake_options_is_built_again() {
        // b11538 built before its recipe set LLAMA_USE_PREBUILT_UI=OFF.
        let mut flags: BTreeMap<String, String> = crate::domain::recipe::LLAMA_CPP
            .cmake_flags
            .iter()
            .map(|(name, value)| ((*name).to_owned(), (*value).to_owned()))
            .collect();
        flags.remove("LLAMA_USE_PREBUILT_UI");
        let builds = FakeBuilds {
            builds: vec![Build {
                cmake_flags: flags,
                ..b11538()
            }],
            ..FakeBuilds::current_b11538()
        };
        let check = builds_check(&builds, &FakePrograms::default(), installed());
        assert_eq!(check.status, Status::Fail);
        assert_eq!(
            check.detail,
            "b11538 was built without LLAMA_USE_PREBUILT_UI=OFF, which its recipe sets"
        );
        assert_eq!(
            check.remedy.as_deref(),
            Some(
                "`ai-ctl build llama.cpp --ref b11538 --force`, or `ai-ctl build llama.cpp` at the \
                 newest release if the project withdrew b11538"
            )
        );
    }

    #[test]
    fn a_current_build_that_no_longer_runs_fails_with_a_rebuild() {
        let programs = FakePrograms {
            version: printed(
                false,
                "",
                "llama-server: error while loading shared libraries: libcudart.so.13: cannot open shared object file: No such file or directory\n",
            ),
            ..FakePrograms::default()
        };
        let check = builds_check(&FakeBuilds::current_b11538(), &programs, installed());
        assert_eq!(check.status, Status::Fail);
        assert!(
            check.detail.starts_with("b11538 does not run: "),
            "{}",
            check.detail
        );
        assert!(check.detail.contains("libcudart.so.13"), "{}", check.detail);
        assert_eq!(
            check.remedy.as_deref(),
            Some(
                "`ai-ctl build llama.cpp --ref b11538 --force`, or `ai-ctl build llama.cpp` at the \
                 newest release if the project withdrew b11538"
            )
        );
        // Without the inventory running it, the same build only warns.
        let check = builds_check(
            &FakeBuilds::current_b11538(),
            &programs,
            FakeStore(Ok(None)),
        );
        assert_eq!(check.status, Status::Warn);
    }

    #[test]
    fn a_project_the_inventory_runs_needs_a_build() {
        let none = FakeBuilds::default();
        let check = builds_check(&none, &FakePrograms::default(), installed());
        assert_eq!(check.status, Status::Fail);
        assert_eq!(check.remedy.as_deref(), Some("ai-ctl build llama.cpp"));
        let check = builds_check(&none, &FakePrograms::default(), FakeStore(Ok(None)));
        assert_eq!(check.status, Status::Skip);
    }

    #[test]
    fn a_backend_no_model_uses_needs_no_build() {
        let mut parts = with_fibo();
        parts.models.retain(|model| model.id != "fibo");
        let report = diagnose(Probed::Found(gpu()), &healthy_host(), stored(parts));
        assert_eq!(
            outcome(check(&report, "stable-diffusion.cpp")),
            (
                Status::Skip,
                "no build; the inventory does not run one",
                None
            )
        );
    }

    #[test]
    fn no_build_under_an_unread_inventory_says_it_was_not_read() {
        let none = FakeBuilds::default();
        for store in [
            FakeStore(Err("model `x` is declared more than once".into())),
            FakeStore(Ok(None)),
        ] {
            let check = builds_check(&none, &FakePrograms::default(), store);
            assert_eq!(
                outcome(&check),
                (
                    Status::Skip,
                    "no build; the inventory could not be read",
                    None
                )
            );
        }
    }

    #[test]
    fn a_current_link_to_a_missing_build_fails() {
        let dangling = FakeBuilds {
            current: Some(crate::domain::build::BuildId::parse("b9").unwrap()),
            ..FakeBuilds::default()
        };
        let check = builds_check(&dangling, &FakePrograms::default(), installed());
        assert_eq!(check.status, Status::Fail);
        assert_eq!(check.detail, "the current build, b9, is not installed");
    }

    #[test]
    fn a_current_build_that_cannot_be_read_fails_with_its_rebuild() {
        let unreadable = FakeBuilds {
            current: Some(crate::domain::build::BuildId::parse("b9").unwrap()),
            unreadable: vec![crate::application::ports::Unreadable {
                dir: "/data/builds/llama.cpp/b9".into(),
                reason: "no build.json".into(),
            }],
            ..FakeBuilds::default()
        };
        let check = builds_check(&unreadable, &FakePrograms::default(), installed());
        assert_eq!(check.status, Status::Fail);
        assert_eq!(
            check.detail,
            "the current build, b9, cannot be read: no build.json"
        );
        assert_eq!(
            check.remedy.as_deref(),
            Some(
                "`ai-ctl build llama.cpp --ref b9 --force`, or `ai-ctl build llama.cpp` at the \
                 newest release if the project withdrew b9"
            )
        );
    }

    #[test]
    fn failing_build_stores_fail_the_build_check_with_a_remedy() {
        let build = "ai-ctl build llama.cpp";
        let rebuild = "`ai-ctl build llama.cpp --ref b11538 --force`, or `ai-ctl build llama.cpp` \
                       at the newest release if the project withdrew b11538";
        let cases = [
            (
                FakeBuilds {
                    current_fails: true,
                    ..FakeBuilds::current_b11538()
                },
                FakePrograms::default(),
                "cannot read /data/builds/llama.cpp/current: Permission denied (os error 13)",
                build,
            ),
            (
                FakeBuilds {
                    builds_fails: true,
                    ..FakeBuilds::current_b11538()
                },
                FakePrograms::default(),
                "cannot read /data/builds/llama.cpp: Permission denied (os error 13)",
                build,
            ),
            (
                FakeBuilds::current_b11538(),
                FakePrograms {
                    run_fails: true,
                    ..FakePrograms::default()
                },
                "b11538: cannot run /data/builds/llama.cpp/b11538/llama-server: \
                 Permission denied (os error 13)",
                rebuild,
            ),
        ];
        for (builds, programs, detail, remedy) in cases {
            let check = builds_check(&builds, &programs, installed());
            assert_eq!(outcome(&check), (Status::Fail, detail, Some(remedy)));
        }
    }

    #[test]
    fn a_project_without_a_recipe_fails() {
        let inventory = Inventory::new(InventoryParts {
            models_dir: "/models".into(),
            gateway: Gateway::default(),
            defaults: BTreeMap::new(),
            backends: vec![Backend {
                id: "llama-native".into(),
                kind: BackendKind::LlamaServer,
                source: BackendSource::Build("nope.cpp".into()),
            }],
            models: vec![model()],
        })
        .unwrap();
        let report = diagnose(
            Probed::Found(gpu()),
            &healthy_host(),
            FakeStore(Ok(Some(inventory))),
        );
        let unknown = check(&report, "nope.cpp");
        assert_eq!(unknown.status, Status::Fail);
        assert!(unknown.detail.contains("no recipe"), "{}", unknown.detail);
        assert_eq!(check(&report, "llama.cpp").status, Status::Ok);
    }

    fn installed() -> FakeStore {
        FakeStore(Ok(Some(inventory("/models"))))
    }

    fn check<'a>(report: &'a Report, name: &str) -> &'a Check {
        report
            .checks()
            .find(|check| check.name == name)
            .unwrap_or_else(|| panic!("no check named {name}"))
    }

    /// A check's status, detail and remedy, to compare at once.
    fn outcome(check: &Check) -> (Status, &str, Option<&str>) {
        (check.status, check.detail.as_str(), check.remedy.as_deref())
    }

    #[test]
    fn a_healthy_machine_passes_every_check() {
        let report = diagnose(Probed::Found(gpu()), &healthy_host(), installed());
        let not_ok: Vec<_> = report
            .checks()
            .filter(|check| check.status != Status::Ok)
            .map(|check| (check.name.as_str(), check.status))
            .collect();
        // The projects the inventory does not run, and that have no build.
        assert_eq!(
            not_ok,
            [
                ("whisper.cpp", Status::Skip),
                ("stable-diffusion.cpp", Status::Skip),
                ("audio.cpp", Status::Skip)
            ]
        );
        assert_eq!(
            report
                .sections
                .iter()
                .map(|s| s.name.as_str())
                .collect::<Vec<_>>(),
            [
                "GPU and driver",
                "Toolchain",
                "Containers (image backends only)",
                "Power",
                "Builds",
                "Inventory",
                "Gateway"
            ]
        );
    }

    #[test]
    fn a_machine_without_nvml_fails_the_gpu_and_goes_on() {
        let report = diagnose(Probed::Missing, &healthy_host(), installed());
        let gpu = check(&report, "GPU");
        assert_eq!(gpu.status, Status::Fail);
        assert!(gpu.remedy.as_deref().unwrap().contains("akmod-nvidia"));
        assert_eq!(check(&report, "Driver").status, Status::Skip);
        assert_eq!(check(&report, "Idle VRAM").status, Status::Skip);
        assert_eq!(check(&report, "CUDA toolkit").status, Status::Ok);
        assert_eq!(check(&report, "Model qwen3.8-27b").status, Status::Ok);
    }

    #[test]
    fn an_unreadable_gpu_fails_with_the_nvml_error() {
        let report = diagnose(
            Probed::Failed("Driver Not Loaded".into()),
            &healthy_host(),
            installed(),
        );
        let gpu = check(&report, "GPU");
        assert_eq!(gpu.status, Status::Fail);
        assert!(gpu.detail.contains("Driver Not Loaded"));
    }

    #[test]
    fn sixty_four_mib_is_held_and_less_is_background() {
        let report = diagnose(
            with(&[(5153, "gnome-shell", 64)]),
            &healthy_host(),
            installed(),
        );
        assert_eq!(check(&report, "Idle VRAM").status, Status::Warn);
        let report = diagnose(
            with(&[(5153, "gnome-shell", 63)]),
            &healthy_host(),
            installed(),
        );
        assert_eq!(check(&report, "Idle VRAM").status, Status::Ok);
    }

    #[test]
    fn a_gpu_other_than_blackwell_is_a_warning() {
        let mut ada = gpu();
        ada.compute_capability = (8, 9);
        let report = diagnose(Probed::Found(ada), &healthy_host(), installed());
        assert_eq!(check(&report, "GPU").status, Status::Warn);
    }

    #[test]
    fn processes_holding_vram_are_named() {
        let mut busy = gpu();
        busy.processes = vec![
            GpuProcess {
                pid: 2210,
                name: "gnome-shell".into(),
                memory: Some(312 << 20),
            },
            GpuProcess {
                pid: 2390,
                name: "Xwayland".into(),
                memory: None,
            },
        ];
        let report = diagnose(Probed::Found(busy), &healthy_host(), installed());
        let vram = check(&report, "Idle VRAM");
        assert_eq!(vram.status, Status::Warn);
        assert_eq!(
            vram.detail,
            "held by PID 2210 (gnome-shell, 312 MiB), PID 2390 (Xwayland)"
        );
        assert!(vram.remedy.as_deref().unwrap().contains("integrated GPU"));
    }

    #[test]
    fn a_background_context_is_tolerated() {
        let mut hybrid = gpu();
        hybrid.processes = vec![GpuProcess {
            pid: 5153,
            name: "gnome-shell".into(),
            memory: Some(4 << 20),
        }];
        let report = diagnose(Probed::Found(hybrid), &healthy_host(), installed());
        let vram = check(&report, "Idle VRAM");
        assert_eq!(vram.status, Status::Ok);
        assert_eq!(
            vram.detail,
            "16 MiB in use; under 64 MiB each: PID 5153 (gnome-shell, 4 MiB)"
        );
    }

    #[test]
    fn cuda_13_2_fails_with_the_upgrade_matching_the_driver() {
        let mut host = healthy_host();
        host.toolchain = FakeToolchain {
            files: host.toolchain.files,
            ..FakeToolchain::before_the_upgrade()
        };
        let report = diagnose(Probed::Found(gpu()), &host, installed());
        let cuda = check(&report, "CUDA toolkit");
        assert_eq!(cuda.status, Status::Fail);
        assert!(cuda.detail.contains("#21255"), "{}", cuda.detail);
        let remedy = cuda.remedy.as_deref().unwrap();
        assert!(
            remedy.starts_with("1. sudo dnf config-manager addrepo"),
            "{remedy}"
        );
        assert!(remedy.contains("cuda-fedora44.repo"), "{remedy}");
        assert!(
            remedy.ends_with("3. sudo dnf install cuda-toolkit-13-4"),
            "{remedy}"
        );
        assert_eq!(check(&report, "Toolkit packages").status, Status::Warn);
        assert!(report.has_failures());
    }

    #[test]
    fn a_toolkit_newer_than_the_driver_fails() {
        let mut host = healthy_host();
        host.toolchain.toolkit = Probed::Found(toolkit(Version::new(13, 5, 10)));
        let report = diagnose(Probed::Found(gpu()), &host, installed());
        let cuda = check(&report, "CUDA toolkit");
        assert_eq!(cuda.status, Status::Fail);
        assert!(
            cuda.detail.contains("newer than CUDA 13.4"),
            "{}",
            cuda.detail
        );
    }

    #[test]
    fn a_toolkit_without_blackwell_support_fails() {
        let mut host = healthy_host();
        host.toolchain.toolkit = Probed::Found(toolkit(Version::new(12, 6, 85)));
        let report = diagnose(Probed::Found(gpu()), &host, installed());
        assert_eq!(check(&report, "CUDA toolkit").status, Status::Fail);
    }

    #[test]
    fn the_toolkit_is_not_compared_with_an_unknown_driver() {
        let mut host = healthy_host();
        host.toolchain.toolkit = Probed::Found(toolkit(Version::new(13, 5, 10)));
        let report = diagnose(Probed::Missing, &host, installed());
        assert_eq!(check(&report, "CUDA toolkit").status, Status::Ok);
    }

    #[test]
    fn a_missing_toolkit_fails_and_skips_the_compiler_limit() {
        let mut host = healthy_host();
        host.toolchain.toolkit = Probed::Missing;
        let report = diagnose(Probed::Found(gpu()), &host, installed());
        let cuda = check(&report, "CUDA toolkit");
        assert_eq!(cuda.status, Status::Fail);
        assert!(
            cuda.remedy
                .as_deref()
                .unwrap()
                .contains("cuda-toolkit-13-4")
        );
        assert_eq!(check(&report, "Host compiler").status, Status::Skip);
    }

    #[test]
    fn the_compiler_check_names_the_compiler_a_build_uses() {
        let report = diagnose(Probed::Found(gpu()), &healthy_host(), installed());
        let compiler = check(&report, "Host compiler");
        assert_eq!(compiler.status, Status::Ok);
        assert!(
            compiler
                .detail
                .starts_with("g++-15 15.3.1; nvcc 13.4.92 accepts GCC 15"),
            "{}",
            compiler.detail
        );
    }

    #[test]
    fn compilers_all_too_new_fail_with_the_compat_package() {
        let mut host = healthy_host();
        host.toolchain.compilers = Probed::Found(vec![gxx("g++", Version::new(16, 2, 1))]);
        let report = diagnose(Probed::Found(gpu()), &host, installed());
        let compiler = check(&report, "Host compiler");
        assert_eq!(compiler.status, Status::Fail);
        assert_eq!(
            compiler.remedy.as_deref(),
            Some("sudo dnf install gcc15-c++")
        );
    }

    #[test]
    fn no_compiler_at_all_fails() {
        let mut host = healthy_host();
        host.toolchain.compilers = Probed::Missing;
        let report = diagnose(Probed::Found(gpu()), &host, installed());
        assert_eq!(
            check(&report, "Host compiler").remedy.as_deref(),
            Some("sudo dnf install gcc-c++")
        );
    }

    #[test]
    fn an_unknown_gcc_limit_is_a_warning() {
        let mut host = healthy_host();
        let mut kit = toolkit(Version::new(13, 4, 92));
        kit.max_gcc_major = Probed::Missing;
        host.toolchain.toolkit = Probed::Found(kit);
        let report = diagnose(Probed::Found(gpu()), &host, installed());
        assert_eq!(check(&report, "Host compiler").status, Status::Warn);
    }

    #[test]
    fn build_tools_below_their_minimum_fail() {
        let mut host = healthy_host();
        host.toolchain.cmake = Probed::Found(Version::new(3, 16, 3));
        host.toolchain.git = Probed::Missing;
        let report = diagnose(Probed::Found(gpu()), &host, installed());
        let cmake = check(&report, "CMake");
        assert_eq!(cmake.status, Status::Fail);
        assert_eq!(cmake.detail, "3.16.3 is older than the 3.18.0 required");
        let git = check(&report, "Git");
        assert_eq!(git.status, Status::Fail);
        assert_eq!(git.remedy.as_deref(), Some("sudo dnf install git"));
    }

    #[test]
    fn missing_container_tooling_only_warns() {
        let mut host = healthy_host();
        host.toolchain.podman = Probed::Missing;
        host.toolchain.nvidia_ctk = Probed::Missing;
        let report = diagnose(Probed::Found(gpu()), &host, installed());
        assert_eq!(check(&report, "Podman").status, Status::Warn);
        assert_eq!(
            check(&report, "NVIDIA Container Toolkit").status,
            Status::Warn
        );
        assert_eq!(check(&report, "CDI specification").status, Status::Skip);
        assert!(
            !host.cdi.listed.get(),
            "CDI is not listed without nvidia-ctk"
        );
        assert!(!report.has_failures());
    }

    #[test]
    fn container_probes_that_fail_warn_with_what_they_said() {
        const CTK: &str = "NVIDIA Container Toolkit";
        const CDI: &str = "CDI specification";
        let mut failing = healthy_host();
        failing.toolchain.podman = Probed::Failed("x".into());
        failing.toolchain.nvidia_ctk = Probed::Failed("x".into());
        let mut unlisted = healthy_host();
        unlisted.cdi.devices = Probed::Failed("x".into());
        let mut empty = healthy_host();
        empty.cdi.devices = Probed::Found(vec![]);
        let cases = [
            (&failing, "Podman", (Status::Warn, "podman: x", None)),
            (&failing, CTK, (Status::Warn, "nvidia-ctk: x", None)),
            (&failing, CDI, (Status::Skip, "needs nvidia-ctk", None)),
            (
                &unlisted,
                CDI,
                (Status::Warn, "nvidia-ctk cdi list: x", Some(CDI_REMEDY)),
            ),
            (
                &empty,
                CDI,
                (Status::Warn, "no CDI device listed", Some(CDI_REMEDY)),
            ),
        ];
        for (host, name, expected) in cases {
            let report = diagnose(Probed::Found(gpu()), host, installed());
            assert_eq!(outcome(check(&report, name)), expected, "{name}");
        }
    }

    #[test]
    fn a_cdi_specification_without_all_gpus_warns_with_the_toolkits_refresh() {
        let mut host = healthy_host();
        host.cdi.devices = Probed::Found(vec!["nvidia.com/gpu=0".into()]);
        let report = diagnose(Probed::Found(gpu()), &host, installed());
        let cdi = check(&report, "CDI specification");
        assert_eq!(cdi.status, Status::Warn);
        assert_eq!(cdi.remedy.as_deref(), Some(CDI_REMEDY));
        assert!(CDI_REMEDY.starts_with("sudo systemctl enable --now nvidia-cdi-refresh.path"));
    }

    fn spec(path: &str, driver: Option<&str>) -> CdiSpec {
        CdiSpec {
            path: path.into(),
            driver: driver.map(str::to_owned),
            missing: vec![],
        }
    }

    #[test]
    fn the_cdi_specification_in_use_names_the_loaded_driver() {
        let report = diagnose(Probed::Found(gpu()), &healthy_host(), installed());
        let cdi = check(&report, "CDI specification");
        assert_eq!(cdi.status, Status::Ok);
        assert_eq!(
            cdi.detail,
            "nvidia.com/gpu=all listed, from /var/run/cdi/nvidia.yaml for driver 615.71.09"
        );
    }

    #[test]
    fn a_cdi_specification_of_another_driver_warns() {
        let mut host = healthy_host();
        host.cdi.specs = Probed::Found(vec![spec("/etc/cdi/nvidia.yaml", Some("580.119.02"))]);
        let report = diagnose(Probed::Found(gpu()), &host, installed());
        let cdi = check(&report, "CDI specification");
        assert_eq!(cdi.status, Status::Warn);
        assert_eq!(
            cdi.detail,
            "/etc/cdi/nvidia.yaml mounts the libraries of driver 580.119.02, but driver 615.71.09 \
             is loaded"
        );
        assert_eq!(cdi.remedy.as_deref(), Some(CDI_REMEDY));
    }

    #[test]
    fn a_cdi_specification_that_mounts_a_path_gone_warns_with_its_refresh() {
        let gone = |path: &str, driver: &str, missing: &[&str]| {
            let mut spec = spec(path, Some(driver));
            spec.missing = missing.iter().map(PathBuf::from).collect();
            Probed::Found(vec![spec])
        };
        let cdi = |specs: Probed<Vec<CdiSpec>>| {
            let mut host = healthy_host();
            host.cdi.specs = specs;
            let report = diagnose(Probed::Found(gpu()), &host, installed());
            check(&report, "CDI specification").clone()
        };
        let wayland = "/usr/lib64/libnvidia-egl-wayland.so.1.1.23";

        let one = cdi(gone("/var/run/cdi/nvidia.yaml", "615.71.09", &[wayland]));
        assert_eq!(one.status, Status::Warn);
        assert_eq!(
            one.detail,
            "/var/run/cdi/nvidia.yaml mounts /usr/lib64/libnvidia-egl-wayland.so.1.1.23, which \
             is not on disk: podman starts no container with the GPU"
        );
        assert_eq!(one.remedy.as_deref(), Some(CDI_REFRESH_REMEDY));

        let three = cdi(gone(
            "/var/run/cdi/nvidia.yaml",
            "615.71.09",
            &["/a", "/b", "/c"],
        ));
        assert_eq!(
            three.detail,
            "/var/run/cdi/nvidia.yaml mounts 3 paths that are not on disk, /a first: podman \
             starts no container with the GPU"
        );

        // A specification that names no driver is checked all the same.
        let mut unnamed = spec("/var/run/cdi/nvidia.yaml", None);
        unnamed.missing = vec![PathBuf::from(wayland)];
        assert_eq!(
            cdi(Probed::Found(vec![unnamed])).remedy.as_deref(),
            Some(CDI_REFRESH_REMEDY)
        );

        // Another driver's specification misses its libraries: the driver says why.
        let other = cdi(gone("/etc/cdi/nvidia.yaml", "580.119.02", &[wayland]));
        assert!(
            other.detail.contains("driver 580.119.02"),
            "{}",
            other.detail
        );
        assert_eq!(other.remedy.as_deref(), Some(CDI_REMEDY));
    }

    #[test]
    fn a_stale_specification_the_one_in_use_shadows_warns_with_its_removal() {
        let mut host = healthy_host();
        host.cdi.specs = Probed::Found(vec![
            spec("/etc/cdi/nvidia.yaml", Some("580.119.02")),
            spec("/var/run/cdi/nvidia.yaml", Some("615.71.09")),
        ]);
        let report = diagnose(Probed::Found(gpu()), &host, installed());
        let cdi = check(&report, "CDI specification");
        assert_eq!(cdi.status, Status::Warn);
        assert_eq!(
            cdi.detail,
            "/etc/cdi/nvidia.yaml names driver 580.119.02; /var/run/cdi/nvidia.yaml shadows it \
             now, but it comes back into use if that one goes missing"
        );
        assert_eq!(
            cdi.remedy.as_deref(),
            Some(
                "sudo rm /etc/cdi/nvidia.yaml   # /var/run/cdi/nvidia.yaml is the specification \
                 to keep current"
            )
        );
    }

    #[test]
    fn without_a_driver_or_a_readable_specification_the_listing_is_enough() {
        let mut host = healthy_host();
        host.cdi.specs = Probed::Failed("/etc/cdi: permission denied".into());
        let report = diagnose(Probed::Found(gpu()), &host, installed());
        let cdi = check(&report, "CDI specification");
        assert_eq!(
            (cdi.status, cdi.detail.as_str()),
            (Status::Ok, "nvidia.com/gpu=all listed")
        );
        let mut host = healthy_host();
        host.cdi.specs = Probed::Found(vec![spec("/var/run/cdi/nvidia.yaml", None)]);
        let report = diagnose(Probed::Found(gpu()), &host, installed());
        assert_eq!(check(&report, "CDI specification").status, Status::Ok);
        let report = diagnose(Probed::Missing, &healthy_host(), installed());
        assert_eq!(check(&report, "CDI specification").status, Status::Ok);
    }

    #[test]
    fn battery_and_a_slow_profile_warn() {
        let mut host = healthy_host();
        host.power.ac = Probed::Found(false);
        host.power.profile = Probed::Found("balanced".into());
        let report = diagnose(Probed::Found(gpu()), &host, installed());
        assert_eq!(check(&report, "AC power").status, Status::Warn);
        let profile = check(&report, "Power profile");
        assert_eq!(profile.status, Status::Warn);
        assert!(
            profile
                .remedy
                .as_deref()
                .unwrap()
                .contains("ActiveProfile s performance")
        );
        assert!(!report.has_failures());
    }

    #[test]
    fn power_checks_skip_what_the_machine_does_not_report() {
        let mut host = healthy_host();
        host.power.ac = Probed::Missing;
        host.power.profile = Probed::Missing;
        let report = diagnose(Probed::Found(gpu()), &host, installed());
        assert_eq!(check(&report, "AC power").status, Status::Skip);
        assert_eq!(check(&report, "Power profile").status, Status::Skip);
    }

    #[test]
    fn power_probes_that_fail_warn_with_what_they_said() {
        let mut host = healthy_host();
        host.power.ac = Probed::Failed("x".into());
        host.power.profile = Probed::Failed("x".into());
        let report = diagnose(Probed::Found(gpu()), &host, installed());
        assert_eq!(
            outcome(check(&report, "AC power")),
            (Status::Warn, "x", None)
        );
        assert_eq!(
            outcome(check(&report, "Power profile")),
            (Status::Warn, "x", None)
        );
    }

    #[test]
    fn a_missing_inventory_fails_with_the_init_command() {
        let report = diagnose(Probed::Found(gpu()), &healthy_host(), FakeStore(Ok(None)));
        let inventory = check(&report, "Inventory");
        assert_eq!(inventory.status, Status::Fail);
        assert_eq!(inventory.remedy.as_deref(), Some(INIT_REMEDY));
        assert!(
            report
                .checks()
                .all(|check| !check.name.starts_with("Model "))
        );
    }

    #[test]
    fn an_invalid_inventory_fails_with_its_error() {
        let store = FakeStore(Err("model `x` is declared more than once".into()));
        let report = diagnose(Probed::Found(gpu()), &healthy_host(), store);
        let inventory = check(&report, "Inventory");
        assert_eq!(inventory.status, Status::Fail);
        assert!(inventory.detail.contains("more than once"));
    }

    #[test]
    fn an_inventory_of_a_newer_version_asks_for_a_newer_ai_ctl() {
        let version = |found| {
            let version = UnsupportedVersion { found, reads: 1 };
            FakeStore(Err(LoadFailure::Version(version)))
        };
        let report = diagnose(Probed::Found(gpu()), &healthy_host(), version(2));
        let inventory = check(&report, "Inventory");
        assert_eq!(inventory.status, Status::Fail);
        assert_eq!(
            inventory.detail,
            "invalid inventory /home/user/.config/ai-ctl/inventory.toml: unsupported inventory \
             version 2 (this ai-ctl reads version 1)"
        );
        assert_eq!(
            inventory.remedy.as_deref(),
            Some(
                "/home/user/.config/ai-ctl/inventory.toml is written for a newer ai-ctl: install \
                 that one, or move the file away and run `ai-ctl init --models-dir <dir>`"
            )
        );

        // No ai-ctl writes a version below the one this one reads: the file is to fix.
        let report = diagnose(Probed::Found(gpu()), &healthy_host(), version(0));
        assert_eq!(
            check(&report, "Inventory").remedy.as_deref(),
            Some(
                "fix /home/user/.config/ai-ctl/inventory.toml, or move it away and run \
                 `ai-ctl init --models-dir <dir>`"
            )
        );
    }

    #[test]
    fn missing_model_files_fail_with_the_download_command() {
        let mut host = healthy_host();
        host.toolchain
            .files
            .remove(Path::new("/models/qwen3.8-27b/Qwen3.8-27B-UD-Q4_K_M.gguf"));
        host.toolchain.files.remove(Path::new(
            "/models/qwen3.8-27b/MTP/mtp-Qwen3.8-27B-Q4_0.gguf",
        ));
        let report = diagnose(Probed::Found(gpu()), &host, installed());
        let model = check(&report, "Model qwen3.8-27b");
        assert_eq!(model.status, Status::Fail);
        assert_eq!(
            model.remedy.as_deref(),
            Some(
                "hf download unsloth/Qwen3.8-27B-GGUF Qwen3.8-27B-UD-Q4_K_M.gguf \
                 MTP/mtp-Qwen3.8-27B-Q4_0.gguf --revision <commit> \
                 --local-dir /models/qwen3.8-27b, at a commit you pin, then check the files as \
                 docs/stack.md says under \"Where Models Come From\""
            )
        );
    }

    #[test]
    fn a_model_without_a_repository_is_copied_by_hand() {
        let mut parts = inventory_parts("/models");
        parts.models[0].repo = None;
        let store = FakeStore(Ok(Some(Inventory::new(parts).unwrap())));
        let mut host = healthy_host();
        host.toolchain
            .files
            .remove(Path::new("/models/qwen3.8-27b/Qwen3.8-27B-UD-Q4_K_M.gguf"));
        let report = diagnose(Probed::Found(gpu()), &host, store);
        assert_eq!(
            outcome(check(&report, "Model qwen3.8-27b")),
            (
                Status::Fail,
                "missing in /models/qwen3.8-27b: Qwen3.8-27B-UD-Q4_K_M.gguf",
                Some("copy Qwen3.8-27B-UD-Q4_K_M.gguf into /models/qwen3.8-27b")
            )
        );
    }

    /// [`inventory_parts`] with an audiocpp_server model, naming `files` of
    /// its directory, or none when it is a directory of safetensors, and
    /// voices.
    fn with_an_audiocpp_model(files: &[(&str, &str)]) -> InventoryParts {
        let mut parts = inventory_parts("/models");
        parts.backends.push(Backend {
            id: "audio-native".into(),
            kind: BackendKind::AudiocppServer,
            source: BackendSource::Build("audio.cpp".into()),
        });
        parts.models.push(Model {
            id: "qwen3-tts".into(),
            backend: "audio-native".into(),
            repo: Some("Qwen/Qwen3-TTS-12Hz-1.7B-Base".into()),
            dir: "tts/qwen3-tts".into(),
            files: files
                .iter()
                .map(|&(flag, file)| (flag.into(), file.into()))
                .collect(),
            flags: BTreeMap::new(),
            ctx_size: None,
            vram_mib: 6025,
            ttl: None,
            preload: false,
            audiocpp: AudiocppFields {
                family: Some("qwen3_tts".into()),
                voices: Some("voices".into()),
                ..AudiocppFields::default()
            },
        });
        parts
    }

    fn stored(parts: InventoryParts) -> FakeStore {
        FakeStore(Ok(Some(Inventory::new(parts).unwrap())))
    }

    #[test]
    fn a_model_that_names_no_file_needs_its_directory_and_its_voices() {
        let mut host = healthy_host();
        host.toolchain
            .dirs
            .extend(["/models/tts/qwen3-tts", "/models/voices"].map(PathBuf::from));
        host.toolchain
            .files
            .insert(PathBuf::from("/models/tts/qwen3-tts/model.safetensors"));
        let report = diagnose(
            Probed::Found(gpu()),
            &host,
            stored(with_an_audiocpp_model(&[])),
        );
        assert_eq!(
            outcome(check(&report, "Model qwen3-tts")),
            (
                Status::Ok,
                "the directory /models/tts/qwen3-tts, voices in /models/voices",
                None
            )
        );
    }

    #[test]
    fn a_missing_model_directory_fails_with_the_download_command() {
        let mut host = healthy_host();
        host.toolchain.dirs.insert(PathBuf::from("/models/voices"));
        let report = diagnose(
            Probed::Found(gpu()),
            &host,
            stored(with_an_audiocpp_model(&[])),
        );
        assert_eq!(
            outcome(check(&report, "Model qwen3-tts")),
            (
                Status::Fail,
                "missing: /models/tts/qwen3-tts",
                Some(
                    "hf download Qwen/Qwen3-TTS-12Hz-1.7B-Base --revision <commit> \
                     --local-dir /models/tts/qwen3-tts, at a commit you pin, then check the \
                     files as docs/stack.md says under \"Where Models Come From\""
                )
            )
        );
    }

    #[test]
    fn a_model_directory_that_cannot_be_read_fails() {
        let mut host = healthy_host();
        host.toolchain
            .dirs
            .extend(["/models/tts/qwen3-tts", "/models/voices"].map(PathBuf::from));
        host.toolchain.entries_fails = true;
        let report = diagnose(
            Probed::Found(gpu()),
            &host,
            stored(with_an_audiocpp_model(&[])),
        );
        assert_eq!(
            outcome(check(&report, "Model qwen3-tts")),
            (
                Status::Fail,
                "cannot read /models/tts/qwen3-tts: Permission denied (os error 13)",
                None
            )
        );
    }

    #[test]
    fn an_empty_model_directory_fails_with_the_download_command() {
        let mut host = healthy_host();
        host.toolchain.dirs.extend(
            [
                "/models/tts/qwen3-tts",
                "/models/tts/qwen3-tts/.cache",
                "/models/voices",
            ]
            .map(PathBuf::from),
        );
        let report = diagnose(
            Probed::Found(gpu()),
            &host,
            stored(with_an_audiocpp_model(&[])),
        );
        assert_eq!(
            outcome(check(&report, "Model qwen3-tts")),
            (
                Status::Fail,
                "empty: /models/tts/qwen3-tts",
                Some(
                    "hf download Qwen/Qwen3-TTS-12Hz-1.7B-Base --revision <commit> \
                     --local-dir /models/tts/qwen3-tts, at a commit you pin, then check the \
                     files as docs/stack.md says under \"Where Models Come From\""
                )
            )
        );
    }

    #[test]
    fn a_missing_model_directory_without_a_repository_is_copied_by_hand() {
        let mut parts = with_an_audiocpp_model(&[]);
        parts.models[1].repo = None;
        let store = stored(parts);
        let mut host = healthy_host();
        host.toolchain.dirs.insert(PathBuf::from("/models/voices"));
        let report = diagnose(Probed::Found(gpu()), &host, store);
        assert_eq!(
            outcome(check(&report, "Model qwen3-tts")),
            (
                Status::Fail,
                "missing: /models/tts/qwen3-tts",
                Some("copy the model's files into /models/tts/qwen3-tts")
            )
        );
    }

    #[test]
    fn missing_voices_fail_beside_missing_files() {
        let mut host = healthy_host();
        host.toolchain
            .dirs
            .insert(PathBuf::from("/models/tts/qwen3-tts"));
        let store = stored(with_an_audiocpp_model(&[("model", "qwen3-tts.gguf")]));
        let report = diagnose(Probed::Found(gpu()), &host, store);
        assert_eq!(
            outcome(check(&report, "Model qwen3-tts")),
            (
                Status::Fail,
                "missing in /models/tts/qwen3-tts: qwen3-tts.gguf; missing: the voices' \
                 directory /models/voices",
                Some(
                    "hf download Qwen/Qwen3-TTS-12Hz-1.7B-Base qwen3-tts.gguf --revision \
                     <commit> --local-dir /models/tts/qwen3-tts, at a commit you pin, then \
                     check the files as docs/stack.md says under \"Where Models Come From\"; \
                     create /models/voices, each voice a <voice>.wav with its line \
                     <voice>|<transcript> in /models/voices/prompt_text"
                )
            )
        );

        host.toolchain
            .files
            .insert(PathBuf::from("/models/tts/qwen3-tts/qwen3-tts.gguf"));
        host.toolchain.dirs.insert(PathBuf::from("/models/voices"));
        let store = stored(with_an_audiocpp_model(&[("model", "qwen3-tts.gguf")]));
        let report = diagnose(Probed::Found(gpu()), &host, store);
        assert_eq!(
            outcome(check(&report, "Model qwen3-tts")),
            (
                Status::Ok,
                "1 file in /models/tts/qwen3-tts, voices in /models/voices",
                None
            )
        );
    }

    /// Fibo's diffusion model: a safetensors index, as Bria publishes it.
    const FIBO_INDEX: &str = "transformer/diffusion_pytorch_model.safetensors.index.json";

    /// [`inventory_parts`] with Fibo, an sd-server model whose diffusion
    /// model is a safetensors index.
    fn with_fibo() -> InventoryParts {
        let mut parts = inventory_parts("/models");
        parts.backends.push(Backend {
            id: "sd-native".into(),
            kind: BackendKind::SdServer,
            source: BackendSource::Build("stable-diffusion.cpp".into()),
        });
        parts.models.push(Model {
            id: "fibo".into(),
            backend: "sd-native".into(),
            repo: Some("briaai/Fibo-2.0-dev".into()),
            dir: "image/fibo".into(),
            files: BTreeMap::from([
                ("diffusion-model".into(), FIBO_INDEX.into()),
                (
                    "vae".into(),
                    "vae/diffusion_pytorch_model.safetensors".into(),
                ),
                ("llm".into(), "Qwen3VL-4B-Instruct-Q8_0.gguf".into()),
            ]),
            flags: BTreeMap::new(),
            ctx_size: None,
            vram_mib: 19440,
            ttl: None,
            preload: false,
            audiocpp: AudiocppFields::default(),
        });
        parts
    }

    /// A healthy host holding Fibo's three files, its index reading `index`,
    /// and its shards `shards`.
    fn fibo_host(index: &str, shards: &[&str]) -> Host {
        let dir = Path::new("/models/image/fibo");
        let mut host = healthy_host();
        host.toolchain.files.extend(
            [
                FIBO_INDEX,
                "vae/diffusion_pytorch_model.safetensors",
                "Qwen3VL-4B-Instruct-Q8_0.gguf",
            ]
            .iter()
            .chain(shards)
            .map(|file| dir.join(file)),
        );
        host.toolchain
            .texts
            .insert(dir.join(FIBO_INDEX), index.to_owned());
        host
    }

    const FIBO_SHARDS: [&str; 2] = [
        "transformer/diffusion_pytorch_model-00001-of-00002.safetensors",
        "transformer/diffusion_pytorch_model-00002-of-00002.safetensors",
    ];

    const BRIAS_INDEX: &str = include_str!(
        "../../tests/fixtures/fibo-2.0-dev-72415bd0-diffusion_pytorch_model.safetensors.index.json"
    );

    #[test]
    fn a_safetensors_index_needs_each_shard_it_names() {
        let host = fibo_host(BRIAS_INDEX, &FIBO_SHARDS[..1]);
        let report = diagnose(Probed::Found(gpu()), &host, stored(with_fibo()));
        assert_eq!(
            outcome(check(&report, "Model fibo")),
            (
                Status::Fail,
                "missing in /models/image/fibo: \
                 transformer/diffusion_pytorch_model-00002-of-00002.safetensors",
                Some(
                    "hf download briaai/Fibo-2.0-dev \
                     transformer/diffusion_pytorch_model-00002-of-00002.safetensors --revision \
                     <commit> --local-dir /models/image/fibo, at a commit you pin, then check the \
                     files as docs/stack.md says under \"Where Models Come From\""
                )
            )
        );

        let host = fibo_host(BRIAS_INDEX, &FIBO_SHARDS);
        let report = diagnose(Probed::Found(gpu()), &host, stored(with_fibo()));
        assert_eq!(
            outcome(check(&report, "Model fibo")),
            (
                Status::Ok,
                "3 files and 2 shards in /models/image/fibo",
                None
            )
        );
    }

    #[test]
    fn an_index_that_cannot_be_read_fails_with_the_command_that_fetches_it_again() {
        let fetch_again = Some(
            "hf download briaai/Fibo-2.0-dev \
             transformer/diffusion_pytorch_model.safetensors.index.json --revision <commit> \
             --local-dir /models/image/fibo, at a commit you pin, then check the files as \
             docs/stack.md says under \"Where Models Come From\"",
        );
        let mut host = fibo_host("", &FIBO_SHARDS);
        host.toolchain.read_text_fails = Some(Path::new("/models/image/fibo").join(FIBO_INDEX));
        let report = diagnose(Probed::Found(gpu()), &host, stored(with_fibo()));
        assert_eq!(
            outcome(check(&report, "Model fibo")),
            (
                Status::Fail,
                "cannot read /models/image/fibo/transformer/\
                 diffusion_pytorch_model.safetensors.index.json: Permission denied (os error 13)",
                fetch_again
            )
        );

        let host = fibo_host("{}", &FIBO_SHARDS);
        let report = diagnose(Probed::Found(gpu()), &host, stored(with_fibo()));
        assert_eq!(
            outcome(check(&report, "Model fibo")),
            (
                Status::Fail,
                "/models/image/fibo/transformer/diffusion_pytorch_model.safetensors.index.json \
                 is not a safetensors index: no `weight_map` object",
                fetch_again
            )
        );

        let outside = r#"{"weight_map": {"a.weight": "../../../other/a.safetensors"}}"#;
        let mut host = fibo_host(outside, &FIBO_SHARDS);
        host.toolchain
            .files
            .insert(PathBuf::from("/models/other/a.safetensors"));
        let report = diagnose(Probed::Found(gpu()), &host, stored(with_fibo()));
        assert_eq!(
            outcome(check(&report, "Model fibo")),
            (
                Status::Fail,
                "transformer/diffusion_pytorch_model.safetensors.index.json names \
                 /models/other/a.safetensors, outside /models/image/fibo",
                fetch_again
            )
        );
    }

    #[test]
    fn a_missing_index_is_a_missing_file_and_another_kinds_json_is_no_index() {
        let mut host = fibo_host(BRIAS_INDEX, &FIBO_SHARDS);
        host.toolchain
            .files
            .remove(Path::new("/models/image/fibo").join(FIBO_INDEX).as_path());
        let report = diagnose(Probed::Found(gpu()), &host, stored(with_fibo()));
        assert_eq!(
            check(&report, "Model fibo").detail,
            "missing in /models/image/fibo: \
             transformer/diffusion_pytorch_model.safetensors.index.json"
        );

        let mut parts = inventory_parts("/models");
        parts.models[0]
            .files
            .insert("chat-template-file".into(), "chat_template.json".into());
        let mut host = healthy_host();
        host.toolchain
            .files
            .insert(PathBuf::from("/models/qwen3.8-27b/chat_template.json"));
        let report = diagnose(Probed::Found(gpu()), &host, stored(parts));
        assert_eq!(check(&report, "Model qwen3.8-27b").status, Status::Ok);
    }

    #[test]
    fn a_shard_name_the_shell_would_read_is_quoted_in_the_download_command() {
        let index = r#"{"weight_map": {"a.weight": "a $(touch pwned).safetensors"}}"#;
        let host = fibo_host(index, &[]);
        let report = diagnose(Probed::Found(gpu()), &host, stored(with_fibo()));
        let remedy = check(&report, "Model fibo").remedy.clone().unwrap();
        assert!(
            remedy.contains(" 'transformer/a $(touch pwned).safetensors' --revision"),
            "{remedy}"
        );
    }

    #[test]
    fn a_repository_the_shell_would_read_is_quoted_in_the_download_command() {
        let mut parts = inventory_parts("/models");
        parts.models[0].repo = Some("user/my repo".into());
        let mut host = healthy_host();
        host.toolchain.files.clear();
        let report = diagnose(Probed::Found(gpu()), &host, stored(parts));
        let remedy = check(&report, "Model qwen3.8-27b").remedy.clone().unwrap();
        assert!(
            remedy.starts_with("hf download 'user/my repo' "),
            "{remedy}"
        );
    }

    #[test]
    fn each_projects_current_build_is_run_from_its_own_binary() {
        let mut builds = FakeBuilds::current_b11538();
        let v1_9_5 = crate::domain::build::BuildId::parse("v1.9.5").unwrap();
        builds.builds.push(Build {
            project: "whisper.cpp".into(),
            cmake_flags: crate::application::fakes::recipe_flags(
                &crate::domain::recipe::WHISPER_CPP,
            ),
            id: v1_9_5.clone(),
            patches: vec!["api-key.patch".into(), "reuse-address.patch".into()],
            ..b11538()
        });
        builds.other_currents.insert("whisper.cpp".into(), v1_9_5);
        let programs = FakePrograms::default();
        diagnose_builds(
            Probed::Found(gpu()),
            &healthy_host(),
            installed(),
            &builds,
            &programs,
        );
        assert!(
            programs.ran.borrow().contains(&PathBuf::from(
                "/data/builds/whisper.cpp/v1.9.5/whisper-server"
            )),
            "{:?}",
            programs.ran.borrow()
        );
    }

    #[test]
    fn a_gateway_serving_an_audiocpp_model_runs_the_configuration_doctor_generates() {
        let mut builds = FakeBuilds::current_b11538();
        let v0_9_3 = crate::domain::build::BuildId::parse("v0.9.3").unwrap();
        builds.builds.push(Build {
            project: "audio.cpp".into(),
            cmake_flags: crate::application::fakes::recipe_flags(&crate::domain::recipe::AUDIO_CPP),
            id: v0_9_3.clone(),
            patches: vec!["api-key.patch".into()],
            ..b11538()
        });
        builds.other_currents.insert("audio.cpp".into(), v0_9_3);
        let store = stored(with_an_audiocpp_model(&[]));
        let mut side = GatewaySide::new();
        *side.gateway.record.borrow_mut() = Some(record(1234));
        side.processes.spawned(&[(5000, "llama-server", 1234)]);
        side.gpu = with(&[(5000, "llama-server", 21094)]);
        let config = Configurator {
            inventory: &store,
            builds: &builds,
            gpu: &FakeGpu(side.gpu.clone()),
            files_dir: Path::new(crate::application::fakes::GATEWAY_DIR),
        }
        .generate()
        .unwrap()
        .to_yaml();
        *side.gateway.config.borrow_mut() = Some(config);
        let host = healthy_host();
        let report = Doctor {
            gpu: &FakeGpu(side.gpu.clone()),
            toolchain: &host.toolchain,
            cdi: &host.cdi,
            power: &host.power,
            inventory: &store,
            builds: &builds,
            programs: &FakePrograms::default(),
            releases: &side.releases,
            gateway: &side.gateway,
            processes: &side.processes,
            ports: &side.ports,
            api: &side.api,
            containers: &side.containers,
        }
        .run();
        let gateway = check(&report, "Gateway");
        assert_eq!(gateway.status, Status::Ok, "{}", gateway.detail);
    }

    #[test]
    fn an_sd_server_tokenizer_json_is_no_index() {
        let mut parts = with_fibo();
        parts.models[1]
            .files
            .insert("tokenizer".into(), "tokenizer.json".into());
        let mut host = fibo_host(BRIAS_INDEX, &FIBO_SHARDS);
        let tokenizer = Path::new("/models/image/fibo/tokenizer.json");
        host.toolchain.files.insert(tokenizer.to_path_buf());
        host.toolchain.texts.insert(
            tokenizer.to_path_buf(),
            r#"{"version": "1.0", "model": {}}"#.into(),
        );
        let report = diagnose(Probed::Found(gpu()), &host, stored(parts));
        assert_eq!(
            outcome(check(&report, "Model fibo")),
            (
                Status::Ok,
                "4 files and 2 shards in /models/image/fibo",
                None
            )
        );
    }

    #[test]
    fn an_index_under_a_models_dir_holding_dot_dot_counts_its_shards() {
        let mut parts = with_fibo();
        parts.models_dir = "/ai/x/../models".into();
        let dir = Path::new("/ai/x/../models/image/fibo");
        let mut host = healthy_host();
        host.toolchain.files.extend(
            [
                FIBO_INDEX,
                "vae/diffusion_pytorch_model.safetensors",
                "Qwen3VL-4B-Instruct-Q8_0.gguf",
            ]
            .iter()
            .chain(&FIBO_SHARDS)
            .map(|file| dir.join(file)),
        );
        host.toolchain
            .texts
            .insert(dir.join(FIBO_INDEX), BRIAS_INDEX.to_owned());
        let report = diagnose(Probed::Found(gpu()), &host, stored(parts));
        assert_eq!(
            outcome(check(&report, "Model fibo")),
            (
                Status::Ok,
                "3 files and 2 shards in /ai/x/../models/image/fibo",
                None
            )
        );
    }

    #[test]
    fn a_shard_hf_would_read_as_an_option_never_reaches_the_download_command() {
        let mut parts = with_fibo();
        parts.models[1].files.insert(
            "diffusion-model".into(),
            "model.safetensors.index.json".into(),
        );
        let dir = Path::new("/models/image/fibo");
        let mut host = healthy_host();
        host.toolchain.files.extend(
            [
                "model.safetensors.index.json",
                "vae/diffusion_pytorch_model.safetensors",
                "Qwen3VL-4B-Instruct-Q8_0.gguf",
            ]
            .map(|file| dir.join(file)),
        );
        // Each names the same shard, `--cache-dir=/tmp/x` in the model's directory.
        for shard in [
            "--cache-dir=/tmp/x",
            "./--cache-dir=/tmp/x",
            "/models/image/fibo/--cache-dir=/tmp/x",
        ] {
            host.toolchain.texts.insert(
                dir.join("model.safetensors.index.json"),
                format!(r#"{{"weight_map": {{"a.weight": "{shard}"}}}}"#),
            );
            let report = diagnose(Probed::Found(gpu()), &host, stored(parts.clone()));
            let model = check(&report, "Model fibo");
            let remedy = model.remedy.clone().unwrap();
            assert!(
                !remedy.contains(" --cache-dir=/tmp/x "),
                "{shard}: {remedy}"
            );
            assert!(
                model.detail.contains("hf download would read as an option"),
                "{shard}: {}",
                model.detail
            );
        }
    }

    #[test]
    fn the_download_command_quotes_unusual_paths() {
        let mut host = healthy_host();
        host.toolchain.files.clear();
        let store = FakeStore(Ok(Some(inventory("/data/my models"))));
        let report = diagnose(Probed::Found(gpu()), &host, store);
        let remedy = check(&report, "Model qwen3.8-27b").remedy.clone().unwrap();
        assert!(
            remedy.contains(" --local-dir '/data/my models/qwen3.8-27b', at a commit"),
            "{remedy}"
        );
    }

    #[test]
    fn shell_quote_escapes_single_quotes() {
        assert_eq!(shell_quote("plain/path-1.gguf"), "plain/path-1.gguf");
        assert_eq!(shell_quote("it's"), r"'it'\''s'");
        assert_eq!(shell_quote(""), "''");
    }

    #[test]
    fn a_stopped_gateway_with_the_pinned_llama_swap_passes() {
        let side = GatewaySide::new();
        let report = side.report();
        let gateway: Vec<(&str, Status, &str)> = report
            .sections
            .iter()
            .find(|section| section.name == "Gateway")
            .unwrap()
            .checks
            .iter()
            .map(|check| (check.name.as_str(), check.status, check.detail.as_str()))
            .collect();
        assert_eq!(
            gateway,
            [
                (
                    "llama-swap",
                    Status::Ok,
                    "v271, the release this ai-ctl pins"
                ),
                (
                    "Gateway configuration",
                    Status::Ok,
                    "1 models, on 127.0.0.1:8080"
                ),
                ("Gateway", Status::Ok, "stopped; `ai-ctl up` starts it"),
                ("Gateway address", Status::Ok, "127.0.0.1:8080 free"),
                ("GPU leftovers", Status::Ok, "none"),
            ]
        );
    }

    #[test]
    fn llama_swap_missing_fails_and_a_release_behind_or_ahead_of_the_pin_warns() {
        let mut side = GatewaySide::new();
        side.releases = FakeReleases::none();
        let swap = side.check("llama-swap");
        assert_eq!(
            (swap.status, swap.detail.as_str(), swap.remedy.as_deref()),
            (
                Status::Fail,
                "not installed; `ai-ctl up` needs it",
                Some("ai-ctl gateway install")
            )
        );
        for release in ["v267", "v270"] {
            side.releases = FakeReleases::current(release);
            let swap = side.check("llama-swap");
            assert_eq!(
                (swap.status, swap.detail.as_str(), swap.remedy.as_deref()),
                (
                    Status::Warn,
                    format!("{release} is current, behind v271, the release this ai-ctl pins")
                        .as_str(),
                    Some("ai-ctl gateway install")
                )
            );
        }
        for release in ["v272", "v300"] {
            side.releases = FakeReleases::current(release);
            let swap = side.check("llama-swap");
            assert_eq!(
                (swap.status, swap.detail.as_str(), swap.remedy.as_deref()),
                (
                    Status::Warn,
                    format!(
                        "{release} is current, ahead of v271, the release this ai-ctl pins \
                         and was tested with"
                    )
                    .as_str(),
                    Some(
                        format!(
                            "ai-ctl gateway install   # back to v271; or run an ai-ctl that \
                             pins {release}"
                        )
                        .as_str()
                    )
                )
            );
        }
        side.releases = FakeReleases::current("v266");
        let swap = side.check("llama-swap");
        assert_eq!(
            (swap.status, swap.detail.as_str(), swap.remedy.as_deref()),
            (
                Status::Fail,
                "v266 is current, and cannot read the configuration this ai-ctl writes, which \
                 needs v267 or later",
                Some("ai-ctl gateway install")
            )
        );
        side.releases = FakeReleases::current("v262");
        side.releases.missing = true;
        let swap = side.check("llama-swap");
        assert_eq!(
            (swap.status, swap.remedy.as_deref()),
            (Status::Fail, Some("ai-ctl gateway install"))
        );
    }

    #[test]
    fn a_llama_swap_off_the_pin_that_no_ai_ctl_checked_fails() {
        let mut side = GatewaySide::new();
        for release in ["v267", "v270", "v272", "v300"] {
            side.releases = FakeReleases::current(release);
            side.releases.record = None;
            let swap = side.check("llama-swap");
            assert_eq!(
                (swap.status, swap.detail.as_str(), swap.remedy.as_deref()),
                (
                    Status::Fail,
                    format!(
                        "{release} is current, but no ai-ctl checked it against a digest it \
                         pins for it"
                    )
                    .as_str(),
                    Some("ai-ctl gateway install")
                ),
            );
        }
    }

    #[test]
    fn a_llama_swap_whose_record_cannot_be_read_fails() {
        let mut side = GatewaySide::new();
        for release in ["v270", "v271", "v272"] {
            side.releases = FakeReleases::current(release);
            side.releases.digest_fails = true;
            let swap = side.check("llama-swap");
            assert_eq!(
                (swap.status, swap.detail.as_str(), swap.remedy.as_deref()),
                (
                    Status::Fail,
                    format!(
                        "cannot read /data/llama-swap/{release}/archive.sha256: Permission \
                         denied (os error 13)"
                    )
                    .as_str(),
                    Some("ai-ctl gateway install")
                ),
            );
        }
    }

    #[test]
    fn the_pinned_llama_swap_never_checked_against_its_pin_fails() {
        // An older ai-ctl installed it against its release's own checksums:
        // `up` refuses it until `gateway install` checks it again.
        let mut side = GatewaySide::new();
        for record in [None, Some(OTHER_DIGEST)] {
            side.releases.record = record;
            let swap = side.check("llama-swap");
            assert_eq!(
                (swap.status, swap.detail.as_str(), swap.remedy.as_deref()),
                (
                    Status::Fail,
                    "v271 is current, but was never checked against the digest this ai-ctl \
                     pins for it",
                    Some("ai-ctl gateway install")
                ),
                "{record:?}"
            );
        }
    }

    #[test]
    fn a_running_gateway_names_its_models_and_leaves_the_idle_vram_alone() {
        let mut side = GatewaySide::serving();
        side.api.models = Some(vec![RunningModel {
            id: "qwen3.6-35b-a3b".into(),
            state: "ready".into(),
            proxy: None,
        }]);
        let report = side.report();
        let gateway = check(&report, "Gateway");
        assert_eq!(
            (gateway.status, gateway.detail.as_str()),
            (
                Status::Ok,
                "running on http://127.0.0.1:8080 (PID 1234): qwen3.6-35b-a3b (ready)"
            )
        );
        assert_eq!(
            check(&report, "Gateway address").detail,
            "127.0.0.1:8080, where the gateway listens"
        );
        let vram = check(&report, "Idle VRAM");
        assert_eq!(
            (vram.status, vram.detail.as_str()),
            (
                Status::Ok,
                "16 MiB in use, 21094 MiB of it by the processes the Gateway section reports"
            )
        );
        // Read-only: nothing was signalled or removed.
        assert!(side.gateway.signals.borrow().is_empty());
        assert_eq!(side.gateway.record.borrow().as_ref(), Some(&record(1234)));
    }

    #[test]
    fn a_running_gateway_out_of_date_or_without_an_api_warns() {
        let side = GatewaySide::serving();
        *side.gateway.config.borrow_mut() = Some("models: {}\n".into());
        let gateway = side.check("Gateway");
        assert_eq!(gateway.status, Status::Warn);
        assert!(gateway.detail.ends_with("on another configuration, address or release than the inventory, the builds and llama-swap give now"));
        assert_eq!(gateway.remedy.as_deref(), Some("ai-ctl down && ai-ctl up"));

        let mut side = GatewaySide::serving();
        side.api.models = None;
        let gateway = side.check("Gateway");
        assert_eq!(
            (gateway.status, gateway.detail.as_str()),
            (
                Status::Warn,
                "running on http://127.0.0.1:8080 (PID 1234), but its API does not answer"
            )
        );
    }

    #[test]
    fn the_gpu_is_read_once_for_every_check() {
        let side = GatewaySide::serving();
        for second in [with(&[]), Probed::Failed("Unknown Error".into())] {
            let gpu = ScriptedGpu::new(vec![side.gpu.clone(), second]);
            let report = side.report_on(&gpu);
            assert_eq!(
                outcome(check(&report, "Idle VRAM")),
                (
                    Status::Ok,
                    "16 MiB in use, 21094 MiB of it by the processes the Gateway section reports",
                    None
                )
            );
            assert_eq!(gpu.reads.get(), 1);
        }
    }

    #[test]
    fn a_gateway_whose_configuration_cannot_be_worked_out_warns_with_the_reason() {
        const AT: &str = "running on http://127.0.0.1:8080 (PID 1234), but whether it runs on \
                          the configuration the inventory gives cannot be told: ";
        // The inventory gained a value llama-swap cannot read since `up`.
        let mut side = GatewaySide::serving();
        side.inventory = templated();
        let gateway = side.check("Gateway");
        assert_eq!(
            (gateway.status, gateway.remedy.as_deref()),
            (Status::Warn, Some("ai-ctl status"))
        );
        assert!(
            gateway.detail.starts_with(AT) && gateway.detail.contains("contains `${`"),
            "{}",
            gateway.detail
        );

        let mut side = GatewaySide::serving();
        side.gateway.config_fails = true;
        assert_eq!(
            outcome(&side.check("Gateway")),
            (
                Status::Warn,
                format!(
                    "{AT}cannot read /state/gateway/llama-swap.yaml: Permission denied (os error 13)"
                )
                .as_str(),
                Some("ai-ctl status")
            )
        );
    }

    /// The installed inventory, with a value llama-swap reads as a macro.
    fn templated() -> FakeStore {
        let mut parts = inventory_parts("/models");
        parts.models[0].flags.insert(
            "chat-template-file".into(),
            FlagValue::Text("${HOME}/t.jinja".into()),
        );
        FakeStore(Ok(Some(Inventory::new(parts).unwrap())))
    }

    #[test]
    fn an_inventory_up_would_refuse_fails_a_check_without_a_gateway() {
        let mut side = GatewaySide::new();
        side.inventory = templated();
        let up = Lifecycle {
            configurator: Configurator {
                inventory: &side.inventory,
                builds: &FakeBuilds::current_b11538(),
                gpu: &FakeGpu(side.gpu.clone()),
                files_dir: Path::new(crate::application::fakes::GATEWAY_DIR),
            },
            releases: &side.releases,
            gateway: &side.gateway,
            processes: &side.processes,
            ports: &side.ports,
            api: &side.api,
            containers: &side.containers,
            pause: advance,
            now,
        }
        .up(&mut |_| {});
        let refusal = format!("{:#}", up.err().unwrap());
        assert!(refusal.contains("contains `${`"), "{refusal}");

        let report = side.report();
        assert!(report.has_failures(), "every check passed: {report:#?}");
        assert_eq!(
            outcome(check(&report, "Gateway configuration")),
            (
                Status::Fail,
                refusal.as_str(),
                Some("fix /home/user/.config/ai-ctl/inventory.toml")
            )
        );
    }

    #[test]
    fn what_the_gateway_inspection_could_not_read_is_reported() {
        let mut side = GatewaySide::serving();
        side.containers.pids_fails = true;
        // A container's process, under its conmon.
        side.processes
            .spawned(&[(6000, "llama-server", 5990), (5990, "conmon", 1)]);
        side.gpu = with(&[(5000, "llama-server", 21094), (6000, "llama-server", 512)]);
        assert_eq!(
            outcome(&side.check("Gateway inspection")),
            (
                Status::Warn,
                "cannot list ai-ctl's containers (podman ps --filter name=ai-ctl-: cannot \
                 connect to the storage); leaving their processes out",
                None
            )
        );
        // An unreadable GPU is the GPU check's to report.
        let mut side = GatewaySide::serving();
        side.gpu = Probed::Failed("Unknown Error".into());
        let report = side.report();
        assert!(
            report
                .checks()
                .all(|check| check.name != "Gateway inspection"),
            "{report:#?}"
        );
    }

    #[test]
    fn a_stale_record_is_reported_and_left_for_status_to_remove() {
        let mut side = GatewaySide::new();
        *side.gateway.record.borrow_mut() = Some(record(1234));
        side.gateway.liveness = Some(Liveness::Exited);
        let gateway = side.check("Gateway");
        assert_eq!(
            (
                gateway.status,
                gateway.detail.as_str(),
                gateway.remedy.as_deref()
            ),
            (
                Status::Warn,
                "stopped; the gateway recorded with PID 1234 has exited",
                Some("ai-ctl status   # removes the record")
            )
        );
        assert_eq!(side.gateway.record.borrow().as_ref(), Some(&record(1234)));
    }

    #[test]
    fn a_taken_address_fails() {
        let mut side = GatewaySide::new();
        side.ports.taken = true;
        let address = side.check("Gateway address");
        assert_eq!(
            (address.status, address.detail.as_str()),
            (
                Status::Fail,
                "127.0.0.1:8080 is taken by another program; `ai-ctl up` would refuse to start"
            )
        );
    }

    #[test]
    fn failing_gateway_stores_fail_their_checks_with_a_remedy() {
        let install = Some("ai-ctl gateway install");
        let mut side = GatewaySide::new();
        side.releases.current_fails = true;
        assert_eq!(
            outcome(&side.check("llama-swap")),
            (
                Status::Fail,
                "cannot read /data/llama-swap/current: Permission denied (os error 13)",
                install
            )
        );
        let mut side = GatewaySide::new();
        side.releases.is_installed_fails = true;
        assert_eq!(
            outcome(&side.check("llama-swap")),
            (
                Status::Fail,
                "cannot read /data/llama-swap/v271: Permission denied (os error 13)",
                install
            )
        );
        let mut side = GatewaySide::new();
        side.ports.is_free_fails = true;
        assert_eq!(
            outcome(&side.check("Gateway address")),
            (
                Status::Fail,
                "cannot bind 127.0.0.1:8080: Cannot assign requested address (os error 99)",
                Some("give the gateway another `listen` address in the inventory")
            )
        );
    }

    #[test]
    fn leftovers_fail_with_kill_and_leave_the_idle_vram_alone() {
        let mut side = GatewaySide::new();
        side.processes
            .programs
            .insert(5000, "/data/builds/llama.cpp/b11538/llama-server".into());
        side.gpu = with(&[(5000, "llama-server", 21094)]);
        let report = side.report();
        let leftovers = check(&report, "GPU leftovers");
        assert_eq!(
            (
                leftovers.status,
                leftovers.detail.as_str(),
                leftovers.remedy.as_deref()
            ),
            (
                Status::Fail,
                "left by an earlier gateway: PID 5000 (llama-server, 21094 MiB); \
                 `ai-ctl up` refuses to start until they are gone",
                Some("kill 5000   # ai-ctl never stops a backend itself")
            )
        );
        assert_eq!(check(&report, "Idle VRAM").status, Status::Ok);
    }

    #[test]
    fn without_a_gpu_or_an_inventory_the_gateway_checks_that_need_them_are_skipped() {
        let mut side = GatewaySide::new();
        side.gpu = Probed::Missing;
        assert_eq!(side.check("GPU leftovers").status, Status::Skip);
        // The GPU and the inventory fail checks of their own.
        assert_eq!(
            outcome(&side.check("Gateway configuration")),
            (
                Status::Skip,
                "needs the inventory, its builds and the GPU",
                None
            )
        );
        let report = diagnose(Probed::Found(gpu()), &healthy_host(), FakeStore(Ok(None)));
        assert_eq!(check(&report, "Gateway address").status, Status::Skip);
        assert_eq!(check(&report, "Gateway").status, Status::Ok);
        assert_eq!(check(&report, "Gateway configuration").status, Status::Skip);
    }

    #[test]
    fn a_gateway_record_that_cannot_be_read_fails_the_gateway_check() {
        let mut side = GatewaySide::new();
        side.gateway.record_fails = true;
        let report = side.report();
        assert_eq!(
            outcome(check(&report, "Gateway")),
            (
                Status::Fail,
                "cannot read /state/gateway/gateway.json: Permission denied (os error 13)",
                None
            )
        );
        assert!(report.has_failures());
    }

    #[test]
    fn a_running_gateway_without_a_model_says_so() {
        let side = GatewaySide::serving();
        assert_eq!(
            outcome(&side.check("Gateway")),
            (
                Status::Ok,
                "running on http://127.0.0.1:8080 (PID 1234), no model loaded",
                None
            )
        );
    }
}
