//! `gateway config`: the gateway's configuration, generated from the
//! installed inventory, the current builds and the GPU's memory.

use std::collections::btree_map::Entry;
use std::collections::{BTreeMap, BTreeSet};
use std::net::SocketAddr;
use std::path::Path;

use anyhow::{Context, Result, anyhow, bail};

use crate::domain::build::Build;
use crate::domain::gateway_config::{self, GatewayConfig, Runtime};
use crate::domain::inventory::{Backend, BackendSource, Inventory};
use crate::domain::machine::{Gpu, Probed};

use super::build::{current, known_recipe, unlike_recipe};
use super::ports::{BuildStore, GpuProbe, InventoryStore, found_gpu};

/// A configuration, with what starting a gateway on it takes besides, and
/// what it was made of.
#[derive(Debug, Clone)]
pub struct Prepared {
    pub config: GatewayConfig,
    /// The inventory's address for the gateway.
    pub listen: SocketAddr,
    /// The images the models run, which must be in local storage: the
    /// configuration never pulls one.
    pub images: Vec<String>,
    /// The inventory the configuration was made of.
    pub inventory: Inventory,
    /// The current build of each project a model runs, by project.
    pub builds: BTreeMap<String, Build>,
    /// The GPU the models were sized to.
    pub gpu: Gpu,
}

/// What a configuration is made of, but the GPU.
struct Resolved {
    inventory: Inventory,
    /// The runtime of every backend a model uses, by backend.
    runtimes: BTreeMap<String, Runtime>,
    /// The current build of each project a model runs, by project.
    builds: BTreeMap<String, Build>,
}

/// The generation use case, with the ports it reads.
#[derive(Clone, Copy)]
pub struct Configurator<'a> {
    pub inventory: &'a dyn InventoryStore,
    pub builds: &'a dyn BuildStore,
    pub gpu: &'a dyn GpuProbe,
    /// Where the files the models' servers read go: the gateway's
    /// directory.
    pub files_dir: &'a Path,
}

impl Configurator<'_> {
    /// The configuration of the installed inventory, each backend a model
    /// uses running from where it is installed now, the models sharing the
    /// GPU's usable memory.
    pub fn generate(&self) -> Result<GatewayConfig> {
        Ok(self.prepare()?.config)
    }

    /// The configuration, with the address and the images it needs. The GPU
    /// is read once the inventory and its builds are: a command that fails
    /// on them never loads NVML.
    pub fn prepare(&self) -> Result<Prepared> {
        let resolved = self.resolve()?;
        size(resolved, &self.gpu.gpu(), self.files_dir)
    }

    /// [`Self::prepare`], on a reading of the GPU the caller has taken.
    pub fn prepare_on(&self, gpu: &Probed<Gpu>) -> Result<Prepared> {
        let resolved = self.resolve()?;
        size(resolved, gpu, self.files_dir)
    }

    /// The installed inventory, and the runtime of every backend a model
    /// uses.
    fn resolve(&self) -> Result<Resolved> {
        let inventory = self.inventory.load()?.ok_or_else(|| {
            anyhow!(
                "no inventory at {}; install it with `ai-ctl init --models-dir <dir>`",
                self.inventory.location().display()
            )
        })?;
        let mut runtimes = BTreeMap::new();
        let mut builds = BTreeMap::new();
        for model in inventory.models() {
            if runtimes.contains_key(&model.backend) {
                continue;
            }
            let backend = inventory
                .backend(&model.backend)
                .ok_or_else(|| anyhow!("backend `{}` is not declared", model.backend))?;
            let runtime = self
                .runtime(backend, &mut builds)
                .with_context(|| format!("backend `{}`, which `{}` uses", backend.id, model.id))?;
            runtimes.insert(backend.id.clone(), runtime);
        }
        Ok(Resolved {
            inventory,
            runtimes,
            builds,
        })
    }

    /// Where `backend` runs from; a build's project resolved once into
    /// `builds`.
    fn runtime(&self, backend: &Backend, builds: &mut BTreeMap<String, Build>) -> Result<Runtime> {
        match &backend.source {
            BackendSource::Build(project) => {
                let recipe = known_recipe(project)?;
                let build = match builds.entry(project.clone()) {
                    Entry::Occupied(entry) => entry.into_mut(),
                    Entry::Vacant(entry) => {
                        let build = current(self.builds, project)?.installed(project)?;
                        if let Some(unlike) = unlike_recipe(recipe, &build) {
                            bail!(
                                "{project} {unlike}; build it again with \
                                 `ai-ctl build {project} --ref {} --force`",
                                build.id
                            );
                        }
                        entry.insert(build)
                    }
                };
                Ok(Runtime::Native(self.builds.binary(
                    project,
                    &build.id,
                    recipe.binary(),
                )))
            }
            BackendSource::Image(reference) => Ok(Runtime::Image(reference.clone())),
        }
    }
}

/// The configuration of what `resolved` holds, sized to `gpu`.
fn size(resolved: Resolved, gpu: &Probed<Gpu>, files_dir: &Path) -> Result<Prepared> {
    let Resolved {
        inventory,
        runtimes,
        builds,
    } = resolved;
    let gpu = found_gpu(gpu).context("cannot size the models to the GPU's memory")?;
    let config = gateway_config::generate(&inventory, &runtimes, gpu.usable_mib(), files_dir)
        .context("cannot configure the gateway")?;
    let images: BTreeSet<&String> = runtimes
        .values()
        .filter_map(|runtime| match runtime {
            Runtime::Image(image) => Some(image),
            Runtime::Native(_) => None,
        })
        .collect();
    Ok(Prepared {
        listen: inventory.gateway().listen,
        images: images.into_iter().cloned().collect(),
        config,
        inventory,
        builds,
        gpu: gpu.clone(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::fakes::{
        FakeBuilds, FakeGpu, FakeInventory, ScriptedGpu, b11538, gpu, inventory, inventory_parts,
    };
    use crate::domain::build::BuildId;
    use crate::domain::inventory::BackendKind;

    fn generate(inventory: FakeInventory, builds: &FakeBuilds) -> Result<GatewayConfig> {
        on_gpu(inventory, builds, Probed::Found(gpu()))
    }

    fn on_gpu(
        inventory: FakeInventory,
        builds: &FakeBuilds,
        gpu: Probed<crate::domain::machine::Gpu>,
    ) -> Result<GatewayConfig> {
        Configurator {
            inventory: &inventory,
            builds,
            gpu: &FakeGpu(gpu),
            files_dir: Path::new(crate::application::fakes::GATEWAY_DIR),
        }
        .generate()
    }

    fn message(result: Result<GatewayConfig>) -> String {
        format!("{:#}", result.unwrap_err())
    }

    #[test]
    fn a_model_runs_the_current_build() {
        let config = generate(
            FakeInventory(Ok(Some(inventory("/models")))),
            &FakeBuilds::current_b11538(),
        )
        .unwrap();
        assert_eq!(
            config.models[0].cmd.lines()[0][0],
            "/data/builds/llama.cpp/b11538/llama-server"
        );
    }

    #[test]
    fn the_models_share_the_memory_the_driver_does_not_reserve() {
        let config = generate(
            FakeInventory(Ok(Some(inventory("/models")))),
            &FakeBuilds::current_b11538(),
        )
        .unwrap();
        assert_eq!(config.matrix.gpu_mib, 24463 - 388);
    }

    #[test]
    fn without_a_readable_gpu_there_is_no_matrix_to_compute() {
        let missing = message(on_gpu(
            FakeInventory(Ok(Some(inventory("/models")))),
            &FakeBuilds::current_b11538(),
            Probed::Missing,
        ));
        assert_eq!(
            missing,
            "cannot size the models to the GPU's memory: no NVIDIA GPU found; run `ai-ctl doctor`"
        );
        let failed = message(on_gpu(
            FakeInventory(Ok(Some(inventory("/models")))),
            &FakeBuilds::current_b11538(),
            Probed::Failed("NVML: driver/library version mismatch".into()),
        ));
        assert_eq!(
            failed,
            "cannot size the models to the GPU's memory: cannot read the GPU: \
             NVML: driver/library version mismatch"
        );
    }

    #[test]
    fn without_an_inventory_init_is_the_remedy() {
        let message = message(generate(
            FakeInventory(Ok(None)),
            &FakeBuilds::current_b11538(),
        ));
        assert_eq!(
            message,
            "no inventory at /home/user/.config/ai-ctl/inventory.toml; install it with `ai-ctl init --models-dir <dir>`"
        );
    }

    #[test]
    fn the_gpu_is_read_once_the_inventory_and_its_builds_are() {
        let reads = |inventory: FakeInventory, builds: &FakeBuilds| {
            let gpu = ScriptedGpu::new(vec![Probed::Found(gpu())]);
            let configurator = Configurator {
                inventory: &inventory,
                builds,
                gpu: &gpu,
                files_dir: Path::new(crate::application::fakes::GATEWAY_DIR),
            };
            let _ = configurator.prepare();
            gpu.reads.get()
        };
        let installed = || FakeInventory(Ok(Some(inventory("/models"))));
        assert_eq!(
            reads(FakeInventory(Ok(None)), &FakeBuilds::current_b11538()),
            0
        );
        assert_eq!(reads(installed(), &FakeBuilds::default()), 0);
        assert_eq!(reads(installed(), &FakeBuilds::current_b11538()), 1);
    }

    #[test]
    fn an_unreadable_inventory_is_an_error() {
        let message = message(generate(
            FakeInventory(Err("invalid TOML".into())),
            &FakeBuilds::current_b11538(),
        ));
        assert_eq!(message, "invalid TOML");
    }

    #[test]
    fn without_a_current_build_the_remedy_is_to_build() {
        let builds = FakeBuilds::default();
        let message = message(generate(
            FakeInventory(Ok(Some(inventory("/models")))),
            &builds,
        ));
        assert_eq!(
            message,
            "backend `llama-native`, which `qwen3.8-27b` uses: llama.cpp has no build; run `ai-ctl build llama.cpp`"
        );
    }

    #[test]
    fn a_current_link_to_a_missing_build_is_an_error() {
        let builds = FakeBuilds {
            current: Some(BuildId::parse("b9").unwrap()),
            ..FakeBuilds::default()
        };
        let message = message(generate(
            FakeInventory(Ok(Some(inventory("/models")))),
            &builds,
        ));
        assert!(
            message.ends_with(
                "the current build of llama.cpp, b9, is not installed; run `ai-ctl build llama.cpp`"
            ),
            "{message}"
        );
    }

    #[test]
    fn a_model_on_an_image_needs_no_build() {
        let mut parts = inventory_parts("/models");
        parts.backends.push(Backend {
            id: "llama-cuda13".into(),
            kind: BackendKind::LlamaServer,
            source: BackendSource::Image("ghcr.io/ggml-org/llama.cpp:server-cuda13".into()),
        });
        parts.models[0].backend = "llama-cuda13".into();
        let installed = Inventory::new(parts).unwrap();
        let builds = FakeBuilds::default();
        let config = generate(FakeInventory(Ok(Some(installed))), &builds).unwrap();
        let words: Vec<&str> = config.models[0]
            .cmd
            .lines()
            .iter()
            .flatten()
            .map(String::as_str)
            .collect();
        assert_eq!(words[..2], ["podman", "run"]);
        assert!(words.contains(&"ghcr.io/ggml-org/llama.cpp:server-cuda13"));
    }

    #[test]
    fn the_images_to_pull_are_those_the_models_run() {
        let mut parts = inventory_parts("/models");
        for (id, image) in [
            ("llama-cuda12", "ghcr.io/ggml-org/llama.cpp:server-cuda"),
            ("llama-cuda13", "ghcr.io/ggml-org/llama.cpp:server-cuda13"),
            (
                "llama-cuda13-bis",
                "ghcr.io/ggml-org/llama.cpp:server-cuda13",
            ),
        ] {
            parts.backends.push(Backend {
                id: id.into(),
                kind: BackendKind::LlamaServer,
                source: BackendSource::Image(image.into()),
            });
        }
        let mut second = crate::application::fakes::model();
        second.id = "second".into();
        second.backend = "llama-cuda13-bis".into();
        parts.models[0].backend = "llama-cuda13".into();
        parts.models.push(second);
        parts.gateway.listen = "127.0.0.1:9090".parse().unwrap();
        let prepared = Configurator {
            inventory: &FakeInventory(Ok(Some(Inventory::new(parts).unwrap()))),
            builds: &FakeBuilds::current_b11538(),
            gpu: &FakeGpu(Probed::Found(gpu())),
            files_dir: Path::new(crate::application::fakes::GATEWAY_DIR),
        }
        .prepare()
        .unwrap();
        assert_eq!(
            prepared.images,
            ["ghcr.io/ggml-org/llama.cpp:server-cuda13"]
        );
        assert_eq!(prepared.listen, "127.0.0.1:9090".parse().unwrap());
    }

    #[test]
    fn a_preparation_keeps_the_inventory_builds_and_gpu_it_was_made_of() {
        let prepared = Configurator {
            inventory: &FakeInventory(Ok(Some(inventory("/models")))),
            builds: &FakeBuilds::current_b11538(),
            gpu: &FakeGpu(Probed::Found(gpu())),
            files_dir: Path::new(crate::application::fakes::GATEWAY_DIR),
        }
        .prepare()
        .unwrap();
        assert_eq!(prepared.inventory, inventory("/models"));
        assert_eq!(
            prepared.builds,
            BTreeMap::from([("llama.cpp".to_owned(), b11538())])
        );
        assert_eq!(prepared.gpu, gpu());
    }

    #[test]
    fn a_current_build_without_one_of_its_recipes_patches_is_refused() {
        let mut parts = inventory_parts("/models");
        parts.backends = vec![Backend {
            id: "whisper".into(),
            kind: BackendKind::WhisperServer,
            source: BackendSource::Build("whisper.cpp".into()),
        }];
        parts.models[0].backend = "whisper".into();
        parts.models[0].ctx_size = None;
        let installed = Inventory::new(parts).unwrap();
        let v1_9_5 = BuildId::parse("v1.9.5").unwrap();
        let built = |patches: &[&str]| FakeBuilds {
            builds: vec![Build {
                project: "whisper.cpp".into(),
                cmake_flags: crate::application::fakes::recipe_flags(
                    &crate::domain::recipe::WHISPER_CPP,
                ),
                id: v1_9_5.clone(),
                patches: patches.iter().map(|&name| name.to_owned()).collect(),
                ..b11538()
            }],
            other_currents: BTreeMap::from([("whisper.cpp".to_owned(), v1_9_5.clone())]),
            ..FakeBuilds::default()
        };
        let message = message(generate(
            FakeInventory(Ok(Some(installed.clone()))),
            &built(&["api-key.patch"]),
        ));
        assert_eq!(
            message,
            "backend `whisper`, which `qwen3.8-27b` uses: whisper.cpp v1.9.5 was built without \
             reuse-address.patch, which its recipe applies; build it again with \
             `ai-ctl build whisper.cpp --ref v1.9.5 --force`"
        );
        assert!(
            generate(
                FakeInventory(Ok(Some(installed))),
                &built(&["api-key.patch", "reuse-address.patch"]),
            )
            .is_ok()
        );
    }

    #[test]
    fn a_current_build_without_one_of_its_recipes_cmake_options_is_refused() {
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
        assert_eq!(
            message(generate(
                FakeInventory(Ok(Some(inventory("/models")))),
                &builds
            )),
            "backend `llama-native`, which `qwen3.8-27b` uses: llama.cpp b11538 was built \
             without LLAMA_USE_PREBUILT_UI=OFF, which its recipe sets; build it again with \
             `ai-ctl build llama.cpp --ref b11538 --force`"
        );
    }

    #[test]
    fn only_the_backends_a_model_uses_are_resolved() {
        let mut parts = inventory_parts("/models");
        parts.backends.push(Backend {
            id: "llama-cuda13".into(),
            kind: BackendKind::LlamaServer,
            source: BackendSource::Image("ghcr.io/ggml-org/llama.cpp:server-cuda13".into()),
        });
        let installed = Inventory::new(parts.clone()).unwrap();
        assert!(
            generate(
                FakeInventory(Ok(Some(installed))),
                &FakeBuilds::current_b11538()
            )
            .is_ok()
        );

        parts.backends.push(Backend {
            id: "whisper".into(),
            kind: BackendKind::LlamaServer,
            source: BackendSource::Build("nope.cpp".into()),
        });
        parts.models[0].backend = "whisper".into();
        let installed = Inventory::new(parts).unwrap();
        let message = message(generate(
            FakeInventory(Ok(Some(installed))),
            &FakeBuilds::current_b11538(),
        ));
        assert_eq!(
            message,
            "backend `whisper`, which `qwen3.8-27b` uses: no recipe for `nope.cpp`; ai-ctl builds llama.cpp, whisper.cpp, stable-diffusion.cpp, audio.cpp"
        );
    }
}
