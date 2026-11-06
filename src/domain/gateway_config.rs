//! The gateway's configuration: what llama-swap runs, generated from the
//! inventory.
//!
//! Each model becomes one llama-swap entry whose command starts its backend
//! on the port llama-swap assigns, from a native build or in a container.
//! The file is written in the dialect
//! llama-swap reads: a command is split like a POSIX shell line once the
//! lines starting with `#` are dropped, and every `${…}` in any value is a
//! macro. Nothing escapes a `$`, so a value containing `${` is refused.
//! llama-swap and each backend take ai-ctl's API key from their environment,
//! so the file holds none.

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::path::{Path, PathBuf};

use super::inventory::{BackendKind, FlagValue, Flags, Inventory, Model};
use super::text::{is_control_or_separator, joined};

/// The macro llama-swap replaces with the port it assigns to a model.
pub const PORT: &str = "${PORT}";
/// The address a backend listens on: loopback, behind the gateway.
const HOST: &str = "127.0.0.1";
/// Every server's readiness endpoint, which ai-ctl's patches leave open to
/// the gateway's check, sent without the key: llama-server answers 503
/// while the model loads, the others answer once it is loaded.
const HEALTH: &str = "/health";
/// The route whisper-server transcribes on, OpenAI's, which the gateway
/// forwards under its own path.
const TRANSCRIPTIONS: &str = "/v1/audio/transcriptions";
/// The directory of the gateway's under which audiocpp_server's
/// configurations go, one per model.
const AUDIOCPP_DIR: &str = "audiocpp";
/// What a model's container is named after its ID, which the inventory
/// keeps to the characters a container name takes.
pub const CONTAINER_PREFIX: &str = "ai-ctl-";
/// The CDI device that gives a container every GPU.
pub const GPU_DEVICE: &str = "nvidia.com/gpu=all";
/// The variable of llama-swap's environment that holds ai-ctl's API key,
/// set by `up`; llama-swap refuses to start without it.
pub const API_KEY_VARIABLE: &str = "AI_CTL_API_KEY";
/// The variable llama-server reads its API key from, as `--api-key`, and
/// ai-ctl's patches make the other servers read.
const KEY_VARIABLE: &str = "LLAMA_API_KEY";

/// What llama-swap writes to its standard output, which `up` sends to the
/// gateway's log.
const LOG_TO_STDOUT: &str = "# The gateway's log keeps the models' output too: llama-swap holds it in\n\
                             # memory only otherwise, and loses it when it stops.\n\
                             logToStdout: \"proxy,upstream\"\n\n";

/// Why llama-swap answers only a client that holds ai-ctl's API key,
/// written beside its `apiKeys`.
const API_KEYS_REASON: &[&str] = &[
    "Every request but /health needs ai-ctl's API key, which `ai-ctl up` passes to",
    "llama-swap in its environment, and llama-swap to each backend in its own.",
];

/// The key as the configuration reads it from llama-swap's environment.
fn key_macro() -> String {
    format!("${{env.{API_KEY_VARIABLE}}}")
}

/// Why llama-server entries open a new connection for every request,
/// written beside the setting. llama-server advertises keep-alive on a
/// streamed response, then closes the connection; llama-swap reuses it for
/// the next request, which fails. Drop the setting once the builds in use
/// include a fix for llama.cpp #29680.
const KEEP_ALIVE_REASON: &[&str] = &[
    "llama-server advertises keep-alive on a streamed response, then closes the",
    "connection: a request sent right after reuses it and fails with 502",
    "\"proxy error: EOF\" (llama-swap #1205). Needed until llama.cpp fixes #29680.",
];

/// Where a backend's program comes from on this machine.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Runtime {
    /// The binary of an installed native build.
    Native(PathBuf),
    /// A container image whose entrypoint is the backend's server, run with
    /// podman.
    Image(String),
}

/// A command line, its words grouped by line: the program, then one option
/// with its value per line. The grouping only makes the file readable;
/// llama-swap joins the lines.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandLine(Vec<Vec<String>>);

impl CommandLine {
    pub fn lines(&self) -> &[Vec<String>] {
        &self.0
    }
}

/// One model as llama-swap serves it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelEntry {
    /// The ID clients request.
    pub id: String,
    /// The command that starts the backend on `${PORT}`.
    pub cmd: CommandLine,
    /// The command that stops it; llama-swap sends the command SIGTERM when
    /// absent.
    pub cmd_stop: Option<Vec<String>>,
    /// Where llama-swap forwards the model's requests.
    pub proxy: String,
    /// The path that answers 200 once the backend is ready.
    pub check_endpoint: &'static str,
    /// Seconds without a request before llama-swap stops the backend; never
    /// when absent.
    pub ttl: Option<u32>,
    /// A new connection to the backend for every request; see
    /// [`KEEP_ALIVE_REASON`].
    pub disable_keep_alives: bool,
    /// The variables llama-swap adds to the backend's environment, each
    /// `NAME=value`.
    pub env: Vec<String>,
    /// The configuration file the backend's server reads its model from,
    /// which `up` writes before it starts the gateway.
    pub file: Option<ConfigFile>,
}

/// A file a model's server reads, written beside the gateway's
/// configuration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigFile {
    pub path: PathBuf,
    pub content: String,
}

/// The models llama-swap may run together, from their VRAM budgets.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Matrix {
    /// The GPU memory processes can allocate, in MiB.
    pub gpu_mib: u64,
    /// What models running together leave free of it, in MiB.
    pub margin_mib: u32,
    /// Every largest combination of models whose budgets fit together, and
    /// each model too large to share the GPU, alone; ordered by the
    /// inventory's order of their models.
    pub sets: Vec<Vec<String>>,
    /// What evicting each model costs: its VRAM budget, since loading it
    /// again reads back about as many bytes.
    pub evict_costs: Vec<(String, u32)>,
}

/// The whole configuration, ready to be written.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GatewayConfig {
    /// One entry per model, in the inventory's order.
    pub models: Vec<ModelEntry>,
    pub matrix: Matrix,
    /// The models loaded when the gateway starts.
    pub preload: Vec<String>,
}

/// Why the inventory cannot be written as a configuration.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ConfigError {
    #[error("backend `{0}` has no program resolved for it")]
    Unresolved(String),
    #[error(
        "model `{model}`: `{text}` contains `${{`, which llama-swap reads as a macro and cannot escape"
    )]
    Macro { model: String, text: String },
    #[error("model `{model}`: {text:?} contains a control character or a line separator")]
    ControlCharacter { model: String, text: String },
    #[error("model `{model}`: path {path:?} is not valid UTF-8")]
    NonUtf8Path { model: String, path: PathBuf },
    #[error(
        "model `{model}` runs in a container, which cannot mount {path:?}: a volume path cannot contain `:`"
    )]
    VolumePath { model: String, path: PathBuf },
    #[error(
        "{sharing} of the inventory's {models} models can share the GPU and run together in more \
         than {MAX_SETS} sets, the most ai-ctl writes, since llama-swap's start slows faster than \
         its sets grow: take models out of the inventory"
    )]
    TooManySets { sharing: usize, models: usize },
}

/// The most sets the matrix holds. llama-swap v263's start slows faster
/// than its sets grow: it answers in 0.65 s on 10816 sets, 19.5 s on 53000
/// and 53.5 s on 84000, past the 30 s `up` waits for it.
pub const MAX_SETS: usize = 20_000;

/// Every reason the inventory cannot be written as a configuration, in the
/// inventory's order.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("{}", joined(.0))]
pub struct InvalidConfig(pub Vec<ConfigError>);

/// Generates the configuration of `inventory` for a GPU of `gpu_mib` usable
/// MiB, each backend running the program `runtimes` gives for its ID. Only
/// the backends a model uses need one. The files a server reads go in
/// `files_dir`. Every value the configuration cannot hold is listed in one
/// error.
pub fn generate(
    inventory: &Inventory,
    runtimes: &BTreeMap<String, Runtime>,
    gpu_mib: u64,
    files_dir: &Path,
) -> Result<GatewayConfig, InvalidConfig> {
    let errors = RefCell::new(Vec::new());
    let models = inventory
        .models()
        .iter()
        .filter_map(|model| entry(inventory, model, runtimes, files_dir, &errors))
        .collect();
    let mut errors = errors.into_inner();
    let matrix = match matrix(inventory, gpu_mib) {
        Ok(matrix) if errors.is_empty() => matrix,
        Ok(_) => return Err(InvalidConfig(errors)),
        Err(err) => {
            errors.push(err);
            return Err(InvalidConfig(errors));
        }
    };
    let preload = inventory
        .models()
        .iter()
        .filter(|model| model.preload)
        .map(|model| model.id.clone())
        .collect();
    Ok(GatewayConfig {
        models,
        matrix,
        preload,
    })
}

/// The matrix of `inventory`, refused past [`MAX_SETS`] sets.
fn matrix(inventory: &Inventory, gpu_mib: u64) -> Result<Matrix, ConfigError> {
    let margin_mib = inventory.gateway().vram_margin_mib;
    let budget = gpu_mib.saturating_sub(u64::from(margin_mib));
    let models = inventory.models();
    let sizes: Vec<u64> = models.iter().map(|m| u64::from(m.vram_mib)).collect();

    let mut sets = Vec::new();
    largest_sets(&sizes, budget, 0, &mut Vec::new(), &mut sets);
    sets.extend(
        (0..sizes.len())
            .filter(|&i| sizes[i] > budget)
            .map(|i| vec![i]),
    );
    if sets.len() > MAX_SETS {
        // The models that fit beside another one make the sets grow.
        let sharing = (0..sizes.len())
            .filter(|&i| (0..sizes.len()).any(|j| j != i && sizes[i] + sizes[j] <= budget))
            .count();
        return Err(ConfigError::TooManySets {
            sharing,
            models: models.len(),
        });
    }
    sets.sort();

    let id = |i: usize| models[i].id.clone();
    Ok(Matrix {
        gpu_mib,
        margin_mib,
        sets: sets
            .into_iter()
            .map(|set| set.into_iter().map(id).collect())
            .collect(),
        evict_costs: models
            .iter()
            .map(|model| (model.id.clone(), model.vram_mib))
            .collect(),
    })
}

/// Adds to `sets` every largest combination of models, by index, that fits
/// in `budget` and extends `chosen` with models from `next` on; stops once
/// `sets` holds more than [`MAX_SETS`].
fn largest_sets(
    sizes: &[u64],
    budget: u64,
    next: usize,
    chosen: &mut Vec<usize>,
    sets: &mut Vec<Vec<usize>>,
) {
    let used: u64 = chosen.iter().map(|&i| sizes[i]).sum();
    let rest: u64 = sizes[next..].iter().sum();
    if used + rest <= budget {
        // Every model left fits: taking them all is the only largest set.
        let set: Vec<usize> = chosen.iter().copied().chain(next..sizes.len()).collect();
        if !set.is_empty() && is_largest(sizes, budget, &set) {
            sets.push(set);
        }
        return;
    }
    for i in next..sizes.len() {
        if sets.len() > MAX_SETS {
            return;
        }
        if used + sizes[i] <= budget {
            chosen.push(i);
            largest_sets(sizes, budget, i + 1, chosen, sets);
            chosen.pop();
        }
    }
    if !chosen.is_empty() && is_largest(sizes, budget, chosen) {
        sets.push(chosen.clone());
    }
}

/// No model outside `set` fits beside it.
fn is_largest(sizes: &[u64], budget: u64, set: &[usize]) -> bool {
    let used: u64 = set.iter().map(|&i| sizes[i]).sum();
    (0..sizes.len()).all(|i| set.contains(&i) || used + sizes[i] > budget)
}

/// The entry of `model`, each error found in it added to `errors`; `None`
/// when its backend has no program, which leaves nothing to check.
fn entry(
    inventory: &Inventory,
    model: &Model,
    runtimes: &BTreeMap<String, Runtime>,
    files_dir: &Path,
    errors: &RefCell<Vec<ConfigError>>,
) -> Option<ModelEntry> {
    let Some((backend, runtime)) = inventory
        .backend(&model.backend)
        .and_then(|backend| Some((backend, runtimes.get(&backend.id)?)))
    else {
        errors
            .borrow_mut()
            .push(ConfigError::Unresolved(model.backend.clone()));
        return None;
    };
    let text = Text {
        model: &model.id,
        errors,
    };
    let id = model.id.clone();
    let kind = backend.kind;
    let (arguments, file) = match kind {
        BackendKind::LlamaServer => (llama_server(inventory, model, text), None),
        BackendKind::WhisperServer => (whisper_server(inventory, model, text), None),
        BackendKind::SdServer => (sd_server(inventory, model, text), None),
        BackendKind::AudiocppServer => {
            let (arguments, file) = audiocpp_server(inventory, model, text, files_dir);
            (arguments, Some(file))
        }
    };
    let (mut lines, cmd_stop) = match runtime {
        Runtime::Native(binary) => (vec![vec![text.path(binary)]], None),
        Runtime::Image(image) => {
            let name = format!("{CONTAINER_PREFIX}{}", model.id);
            let lines = container(inventory.models_dir(), image, &name, text);
            (
                lines,
                Some(vec!["podman".to_owned(), "stop".to_owned(), name]),
            )
        }
    };
    lines.extend(arguments);
    Some(ModelEntry {
        id,
        cmd: CommandLine(lines),
        cmd_stop,
        proxy: format!("http://{HOST}:{PORT}"),
        check_endpoint: HEALTH,
        ttl: model.ttl,
        disable_keep_alives: kind == BackendKind::LlamaServer,
        // The backend's server reads the gateway's key from its environment.
        env: vec![format!("{KEY_VARIABLE}={}", key_macro())],
        file,
    })
}

/// `podman run` of `image`, whose entrypoint is the server, in the
/// foreground so that llama-swap holds its process.
///
/// The container is removed when it stops, and replaces one of the same name
/// a crash left behind. It never pulls: a pull of several GB would outlast
/// llama-swap's health check, so `up` pulls a missing image first. It shares
/// the host's network, so the server binds loopback as a native build does,
/// with no forwarder on the path the bench measures; the image's own health
/// check, on port 8080, would probe the gateway, and is turned off. It runs
/// outside SELinux separation, so the models directory needs no relabelling;
/// it is mounted read-only at its host path, which keeps the models' paths.
/// The key's variable passes from llama-swap's environment into the
/// container's.
fn container(models_dir: &Path, image: &str, name: &str, text: Text) -> Vec<Vec<String>> {
    let dir = text.path(models_dir);
    if dir.contains(':') {
        text.refuse(ConfigError::VolumePath {
            model: text.model.to_owned(),
            path: models_dir.to_owned(),
        });
    }
    let words = |words: &[&str]| words.iter().map(|&word| word.to_owned()).collect();
    vec![
        words(&["podman", "run"]),
        words(&["--rm"]),
        words(&["--replace"]),
        words(&["--name", name]),
        words(&["--pull=never"]),
        words(&["--no-healthcheck"]),
        words(&["--network", "host"]),
        words(&["--security-opt", "label=disable"]),
        words(&["--device", GPU_DEVICE]),
        words(&["--env", KEY_VARIABLE]),
        words(&["--volume", &format!("{dir}:{dir}:ro")]),
        vec![text.checked(image)],
    ]
}

/// llama-server's arguments: loopback and `${PORT}`, named after the model,
/// with `/metrics`, the model's files, its context size, and the kind's
/// default flags overlaid with its own.
fn llama_server(inventory: &Inventory, model: &Model, text: Text) -> Vec<Vec<String>> {
    let mut lines = vec![
        vec!["--host".to_owned(), HOST.to_owned()],
        vec!["--port".to_owned(), PORT.to_owned()],
        vec!["--alias".to_owned(), model.id.clone()],
        vec!["--metrics".to_owned()],
    ];
    lines.extend(files(inventory, BackendKind::LlamaServer, model, text));
    if let Some(ctx_size) = model.ctx_size {
        lines.push(vec!["--ctx-size".to_owned(), ctx_size.to_string()]);
    }
    lines.extend(flags(inventory, BackendKind::LlamaServer, model, text));
    lines
}

/// whisper-server's arguments: loopback and `${PORT}`, transcribing on
/// OpenAI's route, with the model's files and the kind's default flags
/// overlaid with its own.
fn whisper_server(inventory: &Inventory, model: &Model, text: Text) -> Vec<Vec<String>> {
    let mut lines = vec![
        vec!["--host".to_owned(), HOST.to_owned()],
        vec!["--port".to_owned(), PORT.to_owned()],
        vec!["--inference-path".to_owned(), TRANSCRIPTIONS.to_owned()],
    ];
    lines.extend(files(inventory, BackendKind::WhisperServer, model, text));
    lines.extend(flags(inventory, BackendKind::WhisperServer, model, text));
    lines
}

/// sd-server's arguments: loopback and `${PORT}`, its weights loaded before
/// it answers, so that ready means loaded and the VRAM llama-swap sees is
/// the model's; then the model's files and the kind's default flags
/// overlaid with its own.
fn sd_server(inventory: &Inventory, model: &Model, text: Text) -> Vec<Vec<String>> {
    let mut lines = vec![
        vec!["--listen-ip".to_owned(), HOST.to_owned()],
        vec!["--listen-port".to_owned(), PORT.to_owned()],
        vec!["--eager-load".to_owned()],
    ];
    lines.extend(files(inventory, BackendKind::SdServer, model, text));
    lines.extend(flags(inventory, BackendKind::SdServer, model, text));
    lines
}

/// audiocpp_server's arguments: the configuration ai-ctl writes for the
/// model under `files_dir`, the UI off, loopback and `${PORT}`, then the
/// kind's default flags overlaid with the model's own; and that
/// configuration.
fn audiocpp_server(
    inventory: &Inventory,
    model: &Model,
    text: Text,
    files_dir: &Path,
) -> (Vec<Vec<String>>, ConfigFile) {
    let path = files_dir
        .join(AUDIOCPP_DIR)
        .join(format!("{}.json", model.id));
    let mut lines = vec![
        vec!["--config".to_owned(), text.path(&path)],
        vec!["--no-ui".to_owned()],
        vec!["--host".to_owned(), HOST.to_owned()],
        vec!["--port".to_owned(), PORT.to_owned()],
    ];
    lines.extend(flags(inventory, BackendKind::AudiocppServer, model, text));
    let content = audiocpp_config(inventory, model, text);
    (lines, ConfigFile { path, content })
}

/// The configuration audiocpp_server reads: the model alone, its weights
/// its `model` file or its directory, loaded before the server answers so
/// that ready means loaded; the UI off; and its voices when it has some.
/// The command line gives the address. The YAML carries this file as a
/// comment, so each text the inventory leaves free is checked as the YAML's
/// are; the model's ID, which the inventory limits to ASCII letters,
/// digits, `.`, `_` and `-`, needs no check.
fn audiocpp_config(inventory: &Inventory, model: &Model, text: Text) -> String {
    use serde_json::{Map, Value};

    let fields = &model.audiocpp;
    let dir = inventory.model_dir(model);
    let weights = match model.files.get("model") {
        Some(file) => dir.join(file),
        None => dir,
    };
    let mut entry = Map::new();
    entry.insert("id".into(), Value::from(model.id.clone()));
    let family = fields.family.as_deref().unwrap_or_default();
    entry.insert("family".into(), Value::from(text.checked(family)));
    entry.insert("path".into(), Value::from(text.path(&weights)));
    for (key, field) in [("task", &fields.task), ("mode", &fields.mode)] {
        if let Some(field) = field {
            entry.insert(key.into(), Value::from(text.checked(field)));
        }
    }
    if !fields.session_options.is_empty() {
        let mut options = Map::new();
        for (option, value) in &fields.session_options {
            let value = match value {
                FlagValue::Bool(value) => Value::from(*value),
                FlagValue::Int(number) => Value::from(*number),
                FlagValue::Float(number) => Value::from(*number),
                FlagValue::Text(value) => Value::from(text.checked(value)),
            };
            options.insert(text.checked(option), value);
        }
        entry.insert("session_options".into(), Value::Object(options));
    }

    let mut config = Map::new();
    config.insert("lazy_load".into(), Value::from(false));
    config.insert("ui".into(), Value::from(false));
    if let Some(voices) = &fields.voices {
        let voices = inventory.models_dir().join(voices);
        config.insert("voice_dir".into(), Value::from(text.path(&voices)));
    }
    config.insert("models".into(), Value::Array(vec![Value::Object(entry)]));
    format!("{:#}\n", Value::Object(config))
}

/// The model's files, each its flag and its path, its weights first.
fn files(inventory: &Inventory, kind: BackendKind, model: &Model, text: Text) -> Vec<Vec<String>> {
    let dir = inventory.model_dir(model);
    let (first, others): (Vec<_>, Vec<_>) = model
        .files
        .iter()
        .partition(|(flag, _)| kind.model_files().contains(&flag.as_str()));
    first
        .into_iter()
        .chain(others)
        .map(|(flag, file)| vec![text.option(flag), text.path(&dir.join(file))])
        .collect()
}

/// ai-ctl's own defaults for a kind, which the inventory's override.
/// whisper-server decodes a request that names no language as English
/// otherwise, as docs/audio/whisper-cpp.md §2 records under "Language".
fn builtin_flags(kind: BackendKind) -> Flags {
    match kind {
        BackendKind::WhisperServer => {
            Flags::from([("language".to_owned(), FlagValue::Text("auto".to_owned()))])
        }
        _ => Flags::new(),
    }
}

/// The kind's flags: ai-ctl's defaults, overlaid with the inventory's, then
/// with the model's own; `false` leaves a flag out.
fn flags(inventory: &Inventory, kind: BackendKind, model: &Model, text: Text) -> Vec<Vec<String>> {
    let mut flags = builtin_flags(kind);
    flags.extend(inventory.defaults(kind).cloned().unwrap_or_default());
    flags.extend(model.flags.clone());
    let mut lines = Vec::new();
    for (flag, value) in &flags {
        let value = match value {
            FlagValue::Bool(false) => continue,
            FlagValue::Bool(true) => None,
            FlagValue::Int(number) => Some(number.to_string()),
            FlagValue::Float(number) => Some(number.to_string()),
            FlagValue::Text(value) => Some(text.checked(value)),
        };
        lines.push(std::iter::once(text.option(flag)).chain(value).collect());
    }
    lines
}

/// Checks the text an inventory puts in the configuration, on behalf of one
/// model: each text comes back as it is, and each rule it breaks goes to
/// `errors`, so that one generation lists them all.
#[derive(Clone, Copy)]
struct Text<'a> {
    model: &'a str,
    errors: &'a RefCell<Vec<ConfigError>>,
}

impl Text<'_> {
    fn refuse(self, error: ConfigError) {
        self.errors.borrow_mut().push(error);
    }

    fn checked(self, text: &str) -> String {
        if text.contains("${") {
            self.refuse(ConfigError::Macro {
                model: self.model.to_owned(),
                text: text.to_owned(),
            });
        }
        // llama-swap's YAML parser also ends a line at U+2028 and U+2029,
        // which would end the `cmd` block.
        if text.chars().any(is_control_or_separator) {
            self.refuse(ConfigError::ControlCharacter {
                model: self.model.to_owned(),
                text: text.to_owned(),
            });
        }
        text.to_owned()
    }

    fn option(self, flag: &str) -> String {
        self.checked(&format!("--{flag}"))
    }

    fn path(self, path: &Path) -> String {
        match path.to_str() {
            Some(text) => self.checked(text),
            None => {
                self.refuse(ConfigError::NonUtf8Path {
                    model: self.model.to_owned(),
                    path: path.to_owned(),
                });
                path.to_string_lossy().into_owned()
            }
        }
    }
}

impl GatewayConfig {
    /// The files its models' servers read, which must be written before
    /// the gateway starts.
    pub fn files(&self) -> Vec<ConfigFile> {
        self.models
            .iter()
            .filter_map(|model| model.file.clone())
            .collect()
    }

    /// The configuration as llama-swap reads it.
    pub fn to_yaml(&self) -> String {
        let mut out = String::from(
            "# llama-swap's configuration, generated by ai-ctl from its inventory.\n\
             # `ai-ctl up` writes it again: change the inventory instead.\n\n",
        );
        out.push_str(LOG_TO_STDOUT);
        for line in API_KEYS_REASON {
            let _ = writeln!(out, "# {line}");
        }
        let _ = writeln!(out, "apiKeys: [{}]\n", quoted(&key_macro()));
        if self.models.is_empty() {
            out.push_str("models: {}\n");
        } else {
            out.push_str("models:\n");
        }
        for model in &self.models {
            write_model(&mut out, model);
        }
        if !self.matrix.sets.is_empty() {
            write_matrix(&mut out, &self.matrix);
        }
        if !self.preload.is_empty() {
            out.push_str("\nhooks:\n  on_startup:\n    preload:\n");
            for id in &self.preload {
                let _ = writeln!(out, "      - {}", quoted(id));
            }
        }
        out
    }
}

/// Whether `config`, a configuration [`GatewayConfig::to_yaml`] wrote,
/// starts `program` for one of its models: a native model's command holds
/// its program alone on its first line.
pub fn starts_program(config: &str, program: &Path) -> bool {
    let Some(program) = program.to_str() else {
        return false;
    };
    let word = shell_word(program);
    config.lines().any(|line| line.trim() == word)
}

fn write_model(out: &mut String, model: &ModelEntry) {
    let _ = writeln!(out, "  {}:", quoted(&model.id));
    if let Some(file) = &model.file {
        let path = file.path.display();
        let _ = writeln!(
            out,
            "    # The server reads {path}, which `ai-ctl up` writes:"
        );
        for line in file.content.lines() {
            let _ = writeln!(out, "    #   {line}");
        }
    }
    out.push_str("    cmd: |\n");
    for line in model.cmd.lines() {
        let words: Vec<String> = line.iter().map(|word| shell_word(word)).collect();
        let _ = writeln!(out, "      {}", words.join(" "));
    }
    if let Some(cmd_stop) = &model.cmd_stop {
        let words: Vec<String> = cmd_stop.iter().map(|word| shell_word(word)).collect();
        let _ = writeln!(out, "    cmdStop: {}", quoted(&words.join(" ")));
    }
    let _ = writeln!(out, "    proxy: {}", quoted(&model.proxy));
    let _ = writeln!(out, "    checkEndpoint: {}", quoted(model.check_endpoint));
    if !model.env.is_empty() {
        out.push_str("    env:\n");
        for variable in &model.env {
            let _ = writeln!(out, "      - {}", quoted(variable));
        }
    }
    if let Some(ttl) = model.ttl {
        let _ = writeln!(out, "    ttl: {ttl}");
    }
    if model.disable_keep_alives {
        for line in KEEP_ALIVE_REASON {
            let _ = writeln!(out, "    # {line}");
        }
        out.push_str("    disableKeepAlives: true\n");
    }
}

/// The `matrix` swap engine, with every model's eviction cost and its sets.
fn write_matrix(out: &mut String, matrix: &Matrix) {
    let (gpu, margin) = (matrix.gpu_mib, matrix.margin_mib);
    let budget = gpu.saturating_sub(u64::from(margin));
    out.push_str("\nrouting:\n  router:\n    use: \"matrix\"\n    settings:\n");
    let _ = writeln!(
        out,
        "      # Models run side by side only when their VRAM budgets fit in {budget} MiB,"
    );
    let _ = writeln!(
        out,
        "      # the GPU's {gpu} usable MiB less the inventory's margin of {margin} MiB."
    );
    out.push_str("      # Evicting a model costs its budget, which loading it again reads back.\n");
    out.push_str("      matrix:\n        evict_costs:\n");
    for (id, cost) in &matrix.evict_costs {
        let _ = writeln!(out, "          {}: {cost}", quoted(id));
    }
    out.push_str("        sets:\n");
    for (n, set) in matrix.sets.iter().enumerate() {
        let name = format!("fit-{}", n + 1);
        let _ = writeln!(
            out,
            "          {}: {}",
            quoted(&name),
            quoted(&set.join(" & "))
        );
    }
}

/// A YAML double-quoted scalar. The text holds no control character and no
/// line separator.
fn quoted(text: &str) -> String {
    let mut out = String::with_capacity(text.len() + 2);
    out.push('"');
    for c in text.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

/// A word as llama-swap's POSIX splitter reads it back: bare when made of
/// characters it treats as plain, single-quoted otherwise. `#` is always
/// quoted, since a line starting with it is dropped as a comment.
fn shell_word(word: &str) -> String {
    let plain = |c: char| c.is_ascii_alphanumeric() || "_@%+=:,./-${}".contains(c);
    if !word.is_empty() && word.chars().all(plain) {
        word.to_owned()
    } else {
        format!("'{}'", word.replace('\'', r"'\''"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::inventory::{
        AudiocppFields, Backend, BackendSource, Gateway, InventoryParts,
    };

    fn native() -> Backend {
        Backend {
            id: "llama-native".into(),
            kind: BackendKind::LlamaServer,
            source: BackendSource::Build("llama.cpp".into()),
        }
    }

    fn cuda13() -> Backend {
        Backend {
            id: "llama-cuda13".into(),
            kind: BackendKind::LlamaServer,
            source: BackendSource::Image(CUDA13.into()),
        }
    }

    const CUDA13: &str = "ghcr.io/ggml-org/llama.cpp:server-cuda13";

    fn model(id: &str) -> Model {
        Model {
            id: id.into(),
            backend: "llama-native".into(),
            repo: None,
            dir: "llm/qwen".into(),
            files: BTreeMap::from([
                ("mmproj".into(), "mmproj-F16.gguf".into()),
                ("model".into(), "Qwen-Q4_K_M.gguf".into()),
            ]),
            flags: Flags::from([
                ("temp".into(), FlagValue::Float(0.7)),
                ("no-mmproj-offload".into(), FlagValue::Bool(false)),
                ("cache-type-k".into(), FlagValue::Text("q8_0".into())),
            ]),
            ctx_size: Some(98304),
            vram_mib: 22733,
            ttl: Some(1800),
            preload: false,
            audiocpp: Default::default(),
        }
    }

    fn parts() -> InventoryParts {
        InventoryParts {
            models_dir: "/ai/models".into(),
            gateway: Gateway::default(),
            defaults: BTreeMap::from([(
                BackendKind::LlamaServer,
                Flags::from([
                    ("flash-attn".into(), FlagValue::Text("on".into())),
                    ("parallel".into(), FlagValue::Int(1)),
                    ("no-mmproj-offload".into(), FlagValue::Bool(true)),
                    ("temp".into(), FlagValue::Float(0.6)),
                ]),
            )]),
            backends: vec![native()],
            models: vec![model("qwen")],
        }
    }

    fn runtimes() -> BTreeMap<String, Runtime> {
        BTreeMap::from([
            (
                "llama-native".into(),
                Runtime::Native("/data/builds/llama.cpp/b11538/llama-server".into()),
            ),
            ("llama-cuda13".into(), Runtime::Image(CUDA13.into())),
        ])
    }

    /// The inventory of [`parts`], its model running in the CUDA 13 image.
    fn in_a_container() -> InventoryParts {
        let mut parts = parts();
        parts.backends.push(cuda13());
        parts.models[0].backend = "llama-cuda13".into();
        parts
    }

    /// The usable memory of the RTX 5090 Laptop GPU: 24463 MiB less 388
    /// reserved by the driver.
    const GPU_MIB: u64 = 24075;

    /// Where the files the servers read go.
    const FILES: &str = "/state/gateway/files";

    fn generated(parts: InventoryParts) -> Result<GatewayConfig, Vec<ConfigError>> {
        generate(
            &Inventory::new(parts).unwrap(),
            &runtimes(),
            GPU_MIB,
            Path::new(FILES),
        )
        .map_err(|invalid| invalid.0)
    }

    /// The inventory of [`parts`] with one model per `(id, vram_mib)`.
    fn sized(models: &[(&str, u32)]) -> InventoryParts {
        let mut parts = parts();
        parts.models = models
            .iter()
            .map(|&(id, vram_mib)| Model {
                vram_mib,
                ..model(id)
            })
            .collect();
        parts
    }

    fn sets(parts: InventoryParts) -> Vec<String> {
        let matrix = generated(parts).unwrap().matrix;
        matrix.sets.iter().map(|set| set.join(" & ")).collect()
    }

    fn lines(entry: &ModelEntry) -> Vec<String> {
        entry
            .cmd
            .lines()
            .iter()
            .map(|line| line.join(" "))
            .collect()
    }

    #[test]
    fn a_llama_server_model_runs_the_build_on_the_assigned_port() {
        let config = generated(parts()).unwrap();
        let entry = &config.models[0];
        assert_eq!(entry.id, "qwen");
        assert_eq!(
            lines(entry),
            [
                "/data/builds/llama.cpp/b11538/llama-server",
                "--host 127.0.0.1",
                "--port ${PORT}",
                "--alias qwen",
                "--metrics",
                "--model /ai/models/llm/qwen/Qwen-Q4_K_M.gguf",
                "--mmproj /ai/models/llm/qwen/mmproj-F16.gguf",
                "--ctx-size 98304",
                "--cache-type-k q8_0",
                "--flash-attn on",
                "--parallel 1",
                "--temp 0.7",
            ]
        );
        assert_eq!(entry.cmd_stop, None);
        assert_eq!(entry.proxy, "http://127.0.0.1:${PORT}");
        assert_eq!(entry.check_endpoint, "/health");
        assert_eq!(entry.ttl, Some(1800));
        assert!(entry.disable_keep_alives);
        assert_eq!(entry.env, ["LLAMA_API_KEY=${env.AI_CTL_API_KEY}"]);
    }

    #[test]
    fn an_image_model_runs_the_same_server_in_a_named_container() {
        let config = generated(in_a_container()).unwrap();
        let entry = &config.models[0];
        assert_eq!(
            lines(entry),
            [
                "podman run",
                "--rm",
                "--replace",
                "--name ai-ctl-qwen",
                "--pull=never",
                "--no-healthcheck",
                "--network host",
                "--security-opt label=disable",
                "--device nvidia.com/gpu=all",
                "--env LLAMA_API_KEY",
                "--volume /ai/models:/ai/models:ro",
                "ghcr.io/ggml-org/llama.cpp:server-cuda13",
                "--host 127.0.0.1",
                "--port ${PORT}",
                "--alias qwen",
                "--metrics",
                "--model /ai/models/llm/qwen/Qwen-Q4_K_M.gguf",
                "--mmproj /ai/models/llm/qwen/mmproj-F16.gguf",
                "--ctx-size 98304",
                "--cache-type-k q8_0",
                "--flash-attn on",
                "--parallel 1",
                "--temp 0.7",
            ]
        );
        assert_eq!(
            entry.cmd_stop.as_deref(),
            Some(
                ["podman", "stop", "ai-ctl-qwen"]
                    .map(String::from)
                    .as_slice()
            )
        );
        assert_eq!(entry.proxy, "http://127.0.0.1:${PORT}");
        assert_eq!(entry.check_endpoint, "/health");
        assert!(entry.disable_keep_alives);
        assert_eq!(entry.env, ["LLAMA_API_KEY=${env.AI_CTL_API_KEY}"]);
    }

    #[test]
    fn a_pinned_image_is_written_as_it_is() {
        let pinned = "ghcr.io/ggml-org/llama.cpp:server-cuda13-b11541@sha256:42e80867b03cc22e8cf744c07e0b3194f078838ab6c476095ecdc1621271ec95";
        let mut parts = in_a_container();
        parts.backends[1].source = BackendSource::Image(pinned.into());
        let mut runtimes = runtimes();
        runtimes.insert("llama-cuda13".into(), Runtime::Image(pinned.into()));
        let config = generate(
            &Inventory::new(parts).unwrap(),
            &runtimes,
            GPU_MIB,
            Path::new(FILES),
        )
        .unwrap();
        assert!(lines(&config.models[0]).contains(&pinned.to_owned()));
        assert!(config.to_yaml().contains(&format!("\n      {pinned}\n")));
    }

    #[test]
    fn a_container_needs_a_directory_a_volume_can_mount() {
        let mut colon = in_a_container();
        colon.models_dir = "/ai/a:b".into();
        let message = InvalidConfig(generated(colon).unwrap_err()).to_string();
        assert_eq!(
            message,
            "model `qwen` runs in a container, which cannot mount \"/ai/a:b\": a volume path cannot contain `:`"
        );

        let mut native = parts();
        native.models_dir = "/ai/a:b".into();
        assert!(generated(native).is_ok());
    }

    #[test]
    fn a_model_flag_overrides_the_default_and_false_leaves_it_out() {
        let config = generated(parts()).unwrap();
        let words: Vec<&str> = config.models[0]
            .cmd
            .lines()
            .iter()
            .flatten()
            .map(String::as_str)
            .collect();
        assert!(!words.contains(&"--no-mmproj-offload"));
        assert!(!words.contains(&"0.6"));

        let mut parts = parts();
        parts.models[0].flags.clear();
        let config = generated(parts).unwrap();
        let lines = lines(&config.models[0]);
        assert!(
            lines.contains(&"--no-mmproj-offload".to_owned()),
            "{lines:?}"
        );
        assert!(lines.contains(&"--temp 0.6".to_owned()), "{lines:?}");
    }

    #[test]
    fn models_keep_their_order_and_preloaded_ones_are_listed() {
        let mut parts = parts();
        parts.models[0].preload = true;
        let mut second = model("fast");
        second.ttl = None;
        parts.models.push(second);
        let config = generated(parts).unwrap();
        let ids: Vec<&str> = config.models.iter().map(|m| m.id.as_str()).collect();
        assert_eq!(ids, ["qwen", "fast"]);
        assert_eq!(config.preload, ["qwen"]);
        assert_eq!(config.models[1].ttl, None);
    }

    #[test]
    fn a_value_llama_swap_would_read_as_a_macro_is_refused() {
        let mut in_a_flag = parts();
        in_a_flag.models[0].flags.insert(
            "chat-template-file".into(),
            FlagValue::Text("${HOME}/t.jinja".into()),
        );
        assert_eq!(
            generated(in_a_flag),
            Err(vec![ConfigError::Macro {
                model: "qwen".into(),
                text: "${HOME}/t.jinja".into()
            }])
        );

        let mut in_a_path = parts();
        in_a_path.models_dir = "/ai/${env.USER}".into();
        let message = InvalidConfig(generated(in_a_path).unwrap_err()).to_string();
        assert!(message.contains("/ai/${env.USER}/llm/qwen"), "{message}");
        assert!(message.contains("cannot escape"), "{message}");
    }

    #[test]
    fn a_control_character_is_refused() {
        let mut parts = parts();
        parts.models[0]
            .flags
            .insert("system-prompt".into(), FlagValue::Text("one\ntwo".into()));
        assert!(matches!(
            generated(parts).unwrap_err()[..],
            [ConfigError::ControlCharacter { .. }]
        ));
    }

    #[test]
    fn a_unicode_line_separator_is_refused() {
        // Finding 3's flag: llama-swap's YAML parser ends the `cmd` block at
        // U+2028, and reads what follows as a model of its own.
        let injected = "x\u{2028}  evil:\u{2028}    cmd: \"/usr/bin/touch MARKER \\x24{PORT}\" #";
        for separator in ["\u{2028}", "\u{2029}"] {
            let mut parts = parts();
            let value = injected.replace('\u{2028}', separator);
            parts.models[0]
                .flags
                .insert("zzz".into(), FlagValue::Text(value.clone()));
            assert_eq!(
                generated(parts),
                Err(vec![ConfigError::ControlCharacter {
                    model: "qwen".into(),
                    text: value
                }])
            );
        }
        let mut in_a_path = parts();
        in_a_path.models[0]
            .files
            .insert("model".into(), "a\u{2028}b.gguf".into());
        assert!(matches!(
            generated(in_a_path).unwrap_err()[..],
            [ConfigError::ControlCharacter { .. }]
        ));
    }

    /// A model of `backend` in `dir`, with `files` and `flags` and no other
    /// field set.
    fn served(id: &str, backend: &str, dir: &str, files: &[(&str, &str)], flags: Flags) -> Model {
        Model {
            id: id.into(),
            backend: backend.into(),
            repo: None,
            dir: dir.into(),
            files: files
                .iter()
                .map(|&(flag, file)| (flag.into(), file.into()))
                .collect(),
            flags,
            ctx_size: None,
            vram_mib: 2213,
            ttl: Some(600),
            preload: false,
            audiocpp: Default::default(),
        }
    }

    /// The inventory of [`parts`] with a whisper-server and a sd-server
    /// model, and the runtimes of their builds.
    fn audio_and_image() -> (InventoryParts, BTreeMap<String, Runtime>) {
        let mut parts = parts();
        for (id, kind) in [
            ("whisper-native", BackendKind::WhisperServer),
            ("sd-native", BackendKind::SdServer),
            ("audio-native", BackendKind::AudiocppServer),
        ] {
            parts.backends.push(Backend {
                id: id.into(),
                kind,
                source: BackendSource::Build(kind.project().into()),
            });
        }
        parts.models = vec![
            served(
                "whisper-large-v3-turbo",
                "whisper-native",
                "stt/whisper-large-v3-turbo",
                &[("model", "ggml-large-v3-turbo.bin")],
                Flags::new(),
            ),
            served(
                "flux-3-image-klein",
                "sd-native",
                "image/flux-3-image-klein",
                &[
                    ("vae", "ae.safetensors"),
                    ("llm", "Qwen3VL-4B-Instruct-Q8_0.gguf"),
                    ("diffusion-model", "model.safetensors"),
                ],
                Flags::from([
                    ("cfg-scale".into(), FlagValue::Float(1.0)),
                    ("sampling-method".into(), FlagValue::Text("euler".into())),
                    ("steps".into(), FlagValue::Int(16)),
                ]),
            ),
            // A directory of safetensors, quantized at load, with voices.
            Model {
                audiocpp: AudiocppFields {
                    family: Some("qwen3_tts".into()),
                    task: Some("tts".into()),
                    mode: Some("offline".into()),
                    session_options: Flags::from([(
                        "qwen3_tts.weight_type".into(),
                        FlagValue::Text("q8_0".into()),
                    )]),
                    voices: Some("voices".into()),
                },
                ..served(
                    "qwen3-tts-q8",
                    "audio-native",
                    "tts/qwen3-tts-1.7b-base",
                    &[],
                    Flags::new(),
                )
            },
            // One GGUF of a directory, with typed options and no voices.
            Model {
                audiocpp: AudiocppFields {
                    family: Some("fish_audio".into()),
                    session_options: Flags::from([
                        ("fish_audio.seed".into(), FlagValue::Int(7)),
                        ("fish_audio.stream".into(), FlagValue::Bool(true)),
                        ("fish_audio.temperature".into(), FlagValue::Float(0.7)),
                    ]),
                    ..AudiocppFields::default()
                },
                ..served(
                    "fish-s2-pro-q8",
                    "audio-native",
                    "tts/fish-audio-s2-pro",
                    &[("model", "GGUF/fish-audio-s2-pro-q8_0.gguf")],
                    Flags::new(),
                )
            },
        ];
        parts.defaults.insert(
            BackendKind::WhisperServer,
            Flags::from([("threads".into(), FlagValue::Int(8))]),
        );
        parts.defaults.insert(
            BackendKind::AudiocppServer,
            Flags::from([("threads".into(), FlagValue::Int(4))]),
        );
        let mut runtimes = runtimes();
        runtimes.extend([
            (
                "whisper-native".into(),
                Runtime::Native("/data/builds/whisper.cpp/v1.9.5/whisper-server".into()),
            ),
            (
                "sd-native".into(),
                Runtime::Native(
                    "/data/builds/stable-diffusion.cpp/master-1024-77b72a2/sd-server".into(),
                ),
            ),
            (
                "audio-native".into(),
                Runtime::Native("/data/builds/audio.cpp/v0.9.3/audiocpp_server".into()),
            ),
        ]);
        (parts, runtimes)
    }

    fn generated_with(
        (parts, runtimes): (InventoryParts, BTreeMap<String, Runtime>),
    ) -> Result<GatewayConfig, Vec<ConfigError>> {
        generate(
            &Inventory::new(parts).unwrap(),
            &runtimes,
            GPU_MIB,
            Path::new(FILES),
        )
        .map_err(|invalid| invalid.0)
    }

    #[test]
    fn a_whisper_server_model_transcribes_on_the_assigned_port() {
        let config = generated_with(audio_and_image()).unwrap();
        let entry = &config.models[0];
        assert_eq!(entry.id, "whisper-large-v3-turbo");
        assert_eq!(
            lines(entry),
            [
                "/data/builds/whisper.cpp/v1.9.5/whisper-server",
                "--host 127.0.0.1",
                "--port ${PORT}",
                "--inference-path /v1/audio/transcriptions",
                "--model /ai/models/stt/whisper-large-v3-turbo/ggml-large-v3-turbo.bin",
                "--language auto",
                "--threads 8",
            ]
        );
        assert_eq!(entry.cmd_stop, None);
        assert_eq!(entry.proxy, "http://127.0.0.1:${PORT}");
        assert_eq!(entry.check_endpoint, "/health");
        assert_eq!(entry.ttl, Some(600));
        assert!(!entry.disable_keep_alives);
        assert_eq!(entry.env, ["LLAMA_API_KEY=${env.AI_CTL_API_KEY}"]);
        assert!(!config.to_yaml().contains("disableKeepAlives"));
    }

    #[test]
    fn a_whisper_model_or_its_defaults_set_its_language() {
        let language = |config: &GatewayConfig| -> Vec<String> {
            lines(&config.models[0])
                .into_iter()
                .filter(|line| line.starts_with("--language"))
                .collect()
        };
        let (mut parts, runtimes) = audio_and_image();
        parts
            .defaults
            .get_mut(&BackendKind::WhisperServer)
            .unwrap()
            .insert("language".into(), FlagValue::Text("en".into()));
        let config = generated_with((parts.clone(), runtimes.clone())).unwrap();
        assert_eq!(language(&config), ["--language en"]);

        parts.models[0]
            .flags
            .insert("language".into(), FlagValue::Text("fr".into()));
        let config = generated_with((parts, runtimes)).unwrap();
        assert_eq!(language(&config), ["--language fr"]);
    }

    #[test]
    fn an_sd_server_model_loads_its_weights_before_it_is_ready() {
        let config = generated_with(audio_and_image()).unwrap();
        let entry = &config.models[1];
        assert_eq!(entry.id, "flux-3-image-klein");
        assert_eq!(
            lines(entry),
            [
                "/data/builds/stable-diffusion.cpp/master-1024-77b72a2/sd-server",
                "--listen-ip 127.0.0.1",
                "--listen-port ${PORT}",
                "--eager-load",
                "--diffusion-model /ai/models/image/flux-3-image-klein/model.safetensors",
                "--llm /ai/models/image/flux-3-image-klein/Qwen3VL-4B-Instruct-Q8_0.gguf",
                "--vae /ai/models/image/flux-3-image-klein/ae.safetensors",
                "--cfg-scale 1",
                "--sampling-method euler",
                "--steps 16",
            ]
        );
        assert_eq!(entry.check_endpoint, "/health");
        assert!(!entry.disable_keep_alives);
        assert_eq!(entry.env, ["LLAMA_API_KEY=${env.AI_CTL_API_KEY}"]);
        // A whole checkpoint comes first too, and the defaults of llama-server
        // reach no other kind.
        let (mut parts, runtimes) = audio_and_image();
        parts.models[1].files.remove("diffusion-model");
        parts.models[1]
            .files
            .insert("model".into(), "sd.safetensors".into());
        let config = generated_with((parts, runtimes)).unwrap();
        let lines = lines(&config.models[1]);
        assert_eq!(
            lines[4],
            "--model /ai/models/image/flux-3-image-klein/sd.safetensors"
        );
        assert!(!lines.iter().any(|line| line.starts_with("--flash-attn")));
    }

    #[test]
    fn a_control_character_is_refused_for_every_server() {
        let (mut parts, runtimes) = audio_and_image();
        parts.models[0]
            .flags
            .insert("prompt".into(), FlagValue::Text("one\ntwo".into()));
        assert!(matches!(
            &generated_with((parts, runtimes)).unwrap_err()[..],
            [ConfigError::ControlCharacter { model, .. }] if model == "whisper-large-v3-turbo"
        ));
        let (mut parts, runtimes) = audio_and_image();
        parts.models[1]
            .files
            .insert("vae".into(), "ae\u{2028}.safetensors".into());
        assert!(matches!(
            &generated_with((parts, runtimes)).unwrap_err()[..],
            [ConfigError::ControlCharacter { model, .. }] if model == "flux-3-image-klein"
        ));
    }

    /// The configuration file of an entry, read as JSON.
    fn json(entry: &ModelEntry) -> serde_json::Value {
        let file = entry.file.as_ref().expect("the entry has a file");
        serde_json::from_str(&file.content).unwrap()
    }

    #[test]
    fn an_audiocpp_server_model_reads_the_configuration_ai_ctl_writes() {
        let config = generated_with(audio_and_image()).unwrap();
        let entry = &config.models[2];
        let path = "/state/gateway/files/audiocpp/qwen3-tts-q8.json";
        assert_eq!(
            lines(entry),
            [
                "/data/builds/audio.cpp/v0.9.3/audiocpp_server".to_owned(),
                format!("--config {path}"),
                "--no-ui".to_owned(),
                "--host 127.0.0.1".to_owned(),
                "--port ${PORT}".to_owned(),
                "--threads 4".to_owned(),
            ]
        );
        assert_eq!(entry.check_endpoint, "/health");
        assert!(!entry.disable_keep_alives);
        assert_eq!(entry.env, ["LLAMA_API_KEY=${env.AI_CTL_API_KEY}"]);
        assert_eq!(
            entry.file.as_ref().map(|file| file.path.clone()),
            Some(PathBuf::from(path))
        );
        assert_eq!(
            json(entry),
            serde_json::json!({
                "lazy_load": false,
                "ui": false,
                "voice_dir": "/ai/models/voices",
                "models": [{
                    "id": "qwen3-tts-q8",
                    "family": "qwen3_tts",
                    "path": "/ai/models/tts/qwen3-tts-1.7b-base",
                    "task": "tts",
                    "mode": "offline",
                    "session_options": {"qwen3_tts.weight_type": "q8_0"},
                }],
            })
        );
        // The other kinds read no file.
        let files: Vec<PathBuf> = config.files().into_iter().map(|file| file.path).collect();
        assert_eq!(
            files,
            [
                PathBuf::from(path),
                PathBuf::from("/state/gateway/files/audiocpp/fish-s2-pro-q8.json")
            ]
        );
    }

    #[test]
    fn an_audiocpp_server_model_is_its_file_and_its_options_keep_their_type() {
        let config = generated_with(audio_and_image()).unwrap();
        assert_eq!(
            json(&config.models[3]),
            serde_json::json!({
                "lazy_load": false,
                "ui": false,
                "models": [{
                    "id": "fish-s2-pro-q8",
                    "family": "fish_audio",
                    "path": "/ai/models/tts/fish-audio-s2-pro/GGUF/fish-audio-s2-pro-q8_0.gguf",
                    "session_options": {
                        "fish_audio.seed": 7,
                        "fish_audio.stream": true,
                        "fish_audio.temperature": 0.7,
                    },
                }],
            })
        );
    }

    #[test]
    fn the_yaml_carries_each_configuration_file_above_its_model() {
        let config = generated_with(audio_and_image()).unwrap();
        let file = config.models[2].file.clone().unwrap();
        let comment: String = file
            .content
            .lines()
            .map(|line| format!("    #   {line}\n"))
            .collect();
        let expected = format!(
            "  \"qwen3-tts-q8\":\n    # The server reads {}, which `ai-ctl up` writes:\n{comment}    cmd: |\n",
            file.path.display()
        );
        let yaml = config.to_yaml();
        assert!(yaml.contains(&expected), "{yaml}");
        assert!(file.content.ends_with("}\n"), "{}", file.content);
    }

    #[test]
    fn an_audiocpp_value_the_yaml_cannot_hold_is_refused() {
        let refused = |change: fn(&mut Model)| {
            let (mut parts, runtimes) = audio_and_image();
            change(&mut parts.models[2]);
            generated_with((parts, runtimes)).unwrap_err()
        };
        // Each change breaks one rule, which is the only one listed.
        assert!(matches!(
            refused(|model| model.audiocpp.family = Some("qwen3_tts${x}".into()))[..],
            [ConfigError::Macro { .. }]
        ));
        assert!(matches!(
            refused(|model| model.audiocpp.mode = Some("off\nline".into()))[..],
            [ConfigError::ControlCharacter { .. }]
        ));
        assert!(matches!(
            refused(|model| {
                model.audiocpp.session_options.insert(
                    "qwen3_tts.style".into(),
                    FlagValue::Text("a\u{2028}b".into()),
                );
            })[..],
            [ConfigError::ControlCharacter { .. }]
        ));
        assert!(matches!(
            refused(|model| model.audiocpp.task = Some("t\u{7}".into()))[..],
            [ConfigError::ControlCharacter { .. }]
        ));
        assert!(matches!(
            refused(|model| model.audiocpp.voices = Some("voices${y}".into()))[..],
            [ConfigError::Macro { .. }]
        ));
        assert!(matches!(
            refused(|model| {
                model
                    .audiocpp
                    .session_options
                    .insert("qwen3_tts.${z}".into(), FlagValue::Bool(true));
            })[..],
            [ConfigError::Macro { .. }]
        ));
    }

    #[test]
    fn an_audiocpp_path_the_yaml_cannot_hold_is_refused() {
        // Its directory reaches the configuration through its `path` alone.
        let (mut parts, runtimes) = audio_and_image();
        parts.models[2].dir = "tts/a\u{2028}  evil:".into();
        assert!(matches!(
            &generated_with((parts, runtimes)).unwrap_err()[..],
            [ConfigError::ControlCharacter { model, .. }] if model == "qwen3-tts-q8"
        ));
        // And the gateway's directory through `--config`, for each model.
        let (parts, runtimes) = audio_and_image();
        let refused = generate(
            &Inventory::new(parts).unwrap(),
            &runtimes,
            GPU_MIB,
            Path::new("/state/${x}"),
        )
        .unwrap_err()
        .0;
        assert!(
            matches!(
                &refused[..],
                [ConfigError::Macro { .. }, ConfigError::Macro { .. }]
            ),
            "{refused:?}"
        );
    }

    #[test]
    fn the_message_of_several_errors_sets_them_apart() {
        let error = |text: &str| ConfigError::Macro {
            model: "m".into(),
            text: text.into(),
        };
        assert_eq!(
            InvalidConfig(vec![error("a"), error("b")]).to_string(),
            format!("{}; {}", error("a"), error("b"))
        );
    }

    #[test]
    fn every_value_the_configuration_cannot_hold_is_listed() {
        let (mut parts, runtimes) = audio_and_image();
        let flags = &mut parts.models[0].flags;
        flags.insert("prompt".into(), FlagValue::Text("${HOME}".into()));
        flags.insert("suppress-regex".into(), FlagValue::Text("${a}\nb".into()));
        parts.models[1]
            .files
            .insert("vae".into(), "ae\u{2028}.safetensors".into());
        parts.models[2].audiocpp.family = Some("qwen3_tts${x}".into());
        let macro_in = |model: &str, text: &str| ConfigError::Macro {
            model: model.into(),
            text: text.into(),
        };
        let control_in = |model: &str, text: &str| ConfigError::ControlCharacter {
            model: model.into(),
            text: text.into(),
        };
        assert_eq!(
            generated_with((parts, runtimes)),
            Err(vec![
                macro_in("whisper-large-v3-turbo", "${HOME}"),
                macro_in("whisper-large-v3-turbo", "${a}\nb"),
                control_in("whisper-large-v3-turbo", "${a}\nb"),
                control_in(
                    "flux-3-image-klein",
                    "/ai/models/image/flux-3-image-klein/ae\u{2028}.safetensors"
                ),
                macro_in("qwen3-tts-q8", "qwen3_tts${x}"),
            ])
        );
    }

    #[test]
    fn a_backend_without_a_runtime_is_an_error() {
        let inventory = Inventory::new(parts()).unwrap();
        assert_eq!(
            generate(&inventory, &BTreeMap::new(), GPU_MIB, Path::new(FILES)),
            Err(InvalidConfig(vec![ConfigError::Unresolved(
                "llama-native".into()
            )]))
        );
    }

    #[test]
    fn the_yaml_writes_each_option_on_its_own_line() {
        let mut parts = parts();
        parts.models[0].preload = true;
        parts.defaults.clear();
        parts.models[0].flags = Flags::from([
            (
                "chat-template-kwargs".into(),
                FlagValue::Text(r#"{"a":"b c"}"#.into()),
            ),
            ("jinja".into(), FlagValue::Bool(true)),
        ]);
        let mut second = model("fast-1.0");
        second.vram_mib = 21094;
        second.ttl = None;
        second.flags.clear();
        second.files = BTreeMap::from([("model".into(), "My Model #1.gguf".into())]);
        second.ctx_size = Some(4096);
        parts.models.push(second);
        let yaml = generated(parts).unwrap().to_yaml();
        assert_eq!(
            yaml,
            r#"# llama-swap's configuration, generated by ai-ctl from its inventory.
# `ai-ctl up` writes it again: change the inventory instead.

# The gateway's log keeps the models' output too: llama-swap holds it in
# memory only otherwise, and loses it when it stops.
logToStdout: "proxy,upstream"

# Every request but /health needs ai-ctl's API key, which `ai-ctl up` passes to
# llama-swap in its environment, and llama-swap to each backend in its own.
apiKeys: ["${env.AI_CTL_API_KEY}"]

models:
  "qwen":
    cmd: |
      /data/builds/llama.cpp/b11538/llama-server
      --host 127.0.0.1
      --port ${PORT}
      --alias qwen
      --metrics
      --model /ai/models/llm/qwen/Qwen-Q4_K_M.gguf
      --mmproj /ai/models/llm/qwen/mmproj-F16.gguf
      --ctx-size 98304
      --chat-template-kwargs '{"a":"b c"}'
      --jinja
    proxy: "http://127.0.0.1:${PORT}"
    checkEndpoint: "/health"
    env:
      - "LLAMA_API_KEY=${env.AI_CTL_API_KEY}"
    ttl: 1800
    # llama-server advertises keep-alive on a streamed response, then closes the
    # connection: a request sent right after reuses it and fails with 502
    # "proxy error: EOF" (llama-swap #1205). Needed until llama.cpp fixes #29680.
    disableKeepAlives: true
  "fast-1.0":
    cmd: |
      /data/builds/llama.cpp/b11538/llama-server
      --host 127.0.0.1
      --port ${PORT}
      --alias fast-1.0
      --metrics
      --model '/ai/models/llm/qwen/My Model #1.gguf'
      --ctx-size 4096
    proxy: "http://127.0.0.1:${PORT}"
    checkEndpoint: "/health"
    env:
      - "LLAMA_API_KEY=${env.AI_CTL_API_KEY}"
    # llama-server advertises keep-alive on a streamed response, then closes the
    # connection: a request sent right after reuses it and fails with 502
    # "proxy error: EOF" (llama-swap #1205). Needed until llama.cpp fixes #29680.
    disableKeepAlives: true

routing:
  router:
    use: "matrix"
    settings:
      # Models run side by side only when their VRAM budgets fit in 23051 MiB,
      # the GPU's 24075 usable MiB less the inventory's margin of 1024 MiB.
      # Evicting a model costs its budget, which loading it again reads back.
      matrix:
        evict_costs:
          "qwen": 22733
          "fast-1.0": 21094
        sets:
          "fit-1": "qwen"
          "fit-2": "fast-1.0"

hooks:
  on_startup:
    preload:
      - "qwen"
"#
        );
    }

    #[test]
    fn the_yaml_stops_a_container_by_its_name() {
        let yaml = generated(in_a_container()).unwrap().to_yaml();
        assert!(
            yaml.contains(
                "      ghcr.io/ggml-org/llama.cpp:server-cuda13\n      --host 127.0.0.1\n"
            ),
            "{yaml}"
        );
        assert!(
            yaml.contains(
                "      --temp 0.7\n    cmdStop: \"podman stop ai-ctl-qwen\"\n    proxy: "
            ),
            "{yaml}"
        );
    }

    #[test]
    fn the_matrix_lists_the_largest_combinations_that_fit() {
        // 23051 MiB to share: the fast model and whisper miss by 43 MiB.
        let models = [
            ("qwen", 22733),
            ("fast", 21094),
            ("whisper", 2000),
            ("tts", 1000),
        ];
        assert_eq!(
            sets(sized(&models)),
            ["qwen", "fast & tts", "whisper & tts"]
        );

        let mut no_margin = sized(&models);
        no_margin.gateway.vram_margin_mib = 0;
        assert_eq!(
            sets(no_margin),
            [
                "qwen & tts",
                "fast & whisper",
                "fast & tts",
                "whisper & tts"
            ]
        );
    }

    #[test]
    fn the_matrix_sizes_the_models_of_every_kind() {
        let (mut parts, runtimes) = audio_and_image();
        for (model, vram_mib) in parts.models.iter_mut().zip([2213, 18591, 4759, 7953]) {
            model.vram_mib = vram_mib;
        }
        parts.models.push(model("qwen"));
        let matrix = generated_with((parts, runtimes)).unwrap().matrix;
        let sets: Vec<String> = matrix.sets.iter().map(|set| set.join(" & ")).collect();
        assert_eq!(
            sets,
            [
                "whisper-large-v3-turbo & flux-3-image-klein",
                "whisper-large-v3-turbo & qwen3-tts-q8 & fish-s2-pro-q8",
                "qwen",
            ]
        );
        assert_eq!(
            matrix.evict_costs,
            [
                ("whisper-large-v3-turbo".to_owned(), 2213),
                ("flux-3-image-klein".to_owned(), 18591),
                ("qwen3-tts-q8".to_owned(), 4759),
                ("fish-s2-pro-q8".to_owned(), 7953),
                ("qwen".to_owned(), 22733),
            ]
        );
    }

    #[test]
    fn models_that_all_fit_make_one_set() {
        let models = [("a", 4000), ("b", 3000), ("c", 2000)];
        assert_eq!(sets(sized(&models)), ["a & b & c"]);
    }

    /// The number of sets of `parts`' matrix, or its refusals' messages.
    fn set_count(parts: InventoryParts) -> Result<usize, Vec<String>> {
        generated(parts)
            .map(|config| config.matrix.sets.len())
            .map_err(|errors| errors.iter().map(ToString::to_string).collect())
    }

    #[test]
    fn models_that_share_the_gpu_in_too_many_sets_are_refused() {
        // 23051 MiB to share: any 10 of these 20 models fit together, and no
        // 11, which makes 184756 largest sets. The last one, of 12000 MiB,
        // fits beside any other but not beside a model of its own size.
        let ids: Vec<String> = (0..21).map(|i| format!("m{i}")).collect();
        let sizes = (0..21).map(|i| if i < 20 { 2300 } else { 12000 });
        let models: Vec<(&str, u32)> = ids.iter().map(String::as_str).zip(sizes).collect();
        let too_many = "21 of the inventory's 21 models can share the GPU and run together in \
                        more than 20000 sets, the most ai-ctl writes, since llama-swap's start \
                        slows faster than its sets grow: take models out of the inventory";
        assert_eq!(set_count(sized(&models)), Err(vec![too_many.to_owned()]));

        // Listed after the models' own refusals.
        let mut templated = sized(&models);
        templated.models[3].flags.insert(
            "chat-template-file".into(),
            FlagValue::Text("${HOME}/t.jinja".into()),
        );
        assert_eq!(
            set_count(templated),
            Err(vec![
                "model `m3`: `${HOME}/t.jinja` contains `${`, which llama-swap reads as a macro \
                 and cannot escape"
                    .to_owned(),
                too_many.to_owned(),
            ])
        );
    }

    #[test]
    fn the_sets_of_models_that_run_alone_count_toward_the_bound() {
        // Any 2 of 200 models of 10000 MiB fit together, and no 3: 19900
        // sets. One of 20000 MiB fits beside none, and each of 30000 MiB
        // exceeds the budget: a set each.
        let ids: Vec<String> = (0..301).map(|i| format!("m{i}")).collect();
        let sizes = (0..301).map(|i| match i {
            0..200 => 10000,
            200 => 20000,
            _ => 30000,
        });
        let models: Vec<(&str, u32)> = ids.iter().map(String::as_str).zip(sizes).collect();
        assert_eq!(set_count(sized(&models[..300])), Ok(20_000));
        assert_eq!(
            set_count(sized(&models)),
            Err(vec![
                "200 of the inventory's 301 models can share the GPU and run together in more \
                 than 20000 sets, the most ai-ctl writes, since llama-swap's start slows faster \
                 than its sets grow: take models out of the inventory"
                    .to_owned()
            ])
        );
    }

    #[test]
    fn models_that_fill_the_budget_exactly_run_together() {
        assert_eq!(
            GPU_MIB - u64::from(Gateway::DEFAULT_VRAM_MARGIN_MIB),
            20_000 + 3_051
        );
        let models = [("a", 20000), ("b", 3051), ("c", 5000)];
        assert_eq!(sets(sized(&models)), ["a & b", "b & c"]);
    }

    #[test]
    fn a_model_too_large_for_the_gpu_runs_alone() {
        let models = [("small", 1000), ("huge", 30000), ("tiny", 500)];
        assert_eq!(sets(sized(&models)), ["small & tiny", "huge"]);

        let mut no_room = sized(&models);
        no_room.gateway.vram_margin_mib = 30000;
        assert_eq!(sets(no_room), ["small", "huge", "tiny"]);
    }

    #[test]
    fn evicting_a_model_costs_its_vram_budget() {
        let matrix = generated(sized(&[("a", 4000), ("b", 3000)]))
            .unwrap()
            .matrix;
        assert_eq!(
            matrix.evict_costs,
            [("a".to_owned(), 4000), ("b".to_owned(), 3000)]
        );
        assert_eq!((matrix.gpu_mib, matrix.margin_mib), (GPU_MIB, 1024));
    }

    #[test]
    fn a_configuration_starts_the_program_of_each_native_model() {
        let yaml = generated(parts()).unwrap().to_yaml();
        let starts = |program: &str| starts_program(&yaml, Path::new(program));
        assert!(starts("/data/builds/llama.cpp/b11538/llama-server"));
        assert!(!starts("/data/builds/llama.cpp/b1153/llama-server"));
        assert!(!starts("/data/builds/llama.cpp/b11538"));
        assert!(!starts("/ai/models/llm/qwen/Qwen-Q4_K_M.gguf"));

        let spaced = "/home/me/AI data/builds/llama.cpp/b11538/llama-server";
        let mut runtimes = runtimes();
        runtimes.insert("llama-native".into(), Runtime::Native(spaced.into()));
        let config = generate(
            &Inventory::new(parts()).unwrap(),
            &runtimes,
            GPU_MIB,
            Path::new(FILES),
        )
        .unwrap();
        assert!(starts_program(&config.to_yaml(), Path::new(spaced)));
    }

    #[test]
    fn an_inventory_without_models_gives_an_empty_map() {
        let mut parts = parts();
        parts.models.clear();
        assert_eq!(
            generated(parts).unwrap().to_yaml().lines().last(),
            Some("models: {}")
        );
    }

    #[test]
    fn words_are_quoted_only_when_the_splitter_needs_it() {
        assert_eq!(shell_word("--port"), "--port");
        assert_eq!(shell_word("${PORT}"), "${PORT}");
        assert_eq!(
            shell_word("127.0.0.1:${PORT}:8080"),
            "127.0.0.1:${PORT}:8080"
        );
        assert_eq!(shell_word("nvidia.com/gpu=all"), "nvidia.com/gpu=all");
        assert_eq!(shell_word(""), "''");
        assert_eq!(shell_word("a b"), "'a b'");
        assert_eq!(shell_word("#1"), "'#1'");
        assert_eq!(shell_word("it's"), r"'it'\''s'");
        assert_eq!(shell_word(r"C:\x"), r"'C:\x'");
        assert_eq!(shell_word("é"), "'é'");
    }

    #[test]
    fn a_double_quoted_scalar_escapes_quotes_and_backslashes() {
        assert_eq!(quoted("qwen"), r#""qwen""#);
        assert_eq!(quoted(r#"a"b\c"#), r#""a\"b\\c""#);
    }

    #[test]
    fn a_program_path_that_is_not_utf8_is_refused() {
        use std::os::unix::ffi::OsStrExt;
        let odd = PathBuf::from(std::ffi::OsStr::from_bytes(
            b"/home/s\xe9b/.local/share/ai-ctl/builds/llama.cpp/b11538/llama-server",
        ));
        let mut runtimes = runtimes();
        runtimes.insert("llama-native".into(), Runtime::Native(odd.clone()));
        assert_eq!(
            generate(
                &Inventory::new(parts()).unwrap(),
                &runtimes,
                GPU_MIB,
                Path::new(FILES)
            ),
            Err(InvalidConfig(vec![ConfigError::NonUtf8Path {
                model: "qwen".into(),
                path: odd
            }]))
        );
    }
}
