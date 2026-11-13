//! The inventory: the backends and models the gateway serves.
//!
//! A model names a backend; the backend's kind decides which of the model's
//! fields apply, so audio and image kinds can join `llama-server` without a
//! breaking change to the format. An [`Inventory`] value is always valid:
//! [`Inventory::new`] checks every rule and reports every violation at once.

use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::fmt;
use std::net::SocketAddr;
use std::path::{Component, Path, PathBuf};

use super::gateway::Sha256;
use super::recipe::recipe;
use super::text::joined;

/// The program a backend runs, named after its binary.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum BackendKind {
    LlamaServer,
    WhisperServer,
    SdServer,
    AudiocppServer,
}

impl BackendKind {
    pub const ALL: [BackendKind; 4] = [
        BackendKind::LlamaServer,
        BackendKind::WhisperServer,
        BackendKind::SdServer,
        BackendKind::AudiocppServer,
    ];

    /// The name used in the inventory file.
    pub fn name(self) -> &'static str {
        match self {
            BackendKind::LlamaServer => "llama-server",
            BackendKind::WhisperServer => "whisper-server",
            BackendKind::SdServer => "sd-server",
            BackendKind::AudiocppServer => "audiocpp_server",
        }
    }

    pub fn from_name(name: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|kind| kind.name() == name)
    }

    /// The project whose build serves this kind.
    pub fn project(self) -> &'static str {
        match self {
            BackendKind::LlamaServer => "llama.cpp",
            BackendKind::WhisperServer => "whisper.cpp",
            BackendKind::SdServer => "stable-diffusion.cpp",
            BackendKind::AudiocppServer => "audio.cpp",
        }
    }

    /// Flags ai-ctl or the gateway sets itself, which no flag table may set.
    /// The gateway gives llama-server its API key; another one, given by a
    /// flag, would open it to a second key and show on its command line. The
    /// other servers take the key from their environment alone, by ai-ctl's
    /// patches.
    fn reserved_flags(self) -> &'static [&'static str] {
        match self {
            BackendKind::LlamaServer => &[
                "host",
                "port",
                "alias",
                "metrics",
                "ctx-size",
                "model",
                "api-key",
                "api-key-file",
            ],
            // `request-path` would move every route, `/health` included.
            BackendKind::WhisperServer => {
                &["host", "port", "model", "inference-path", "request-path"]
            }
            // Without `--eager-load`, the server is ready before its weights
            // are loaded. `--model-dir` would give its model routes files to
            // load in place of the model's, outside its VRAM budget.
            BackendKind::SdServer => &[
                "listen-ip",
                "listen-port",
                "eager-load",
                "model",
                "diffusion-model",
                "model-dir",
            ],
            // The server takes its models from the configuration ai-ctl
            // writes, its UI off, and reads `--ui` wherever it stands.
            BackendKind::AudiocppServer => {
                &["config", "host", "port", "no-ui", "ui", "ui-management"]
            }
        }
    }

    /// The files every model of this kind names one of at least: its
    /// weights, which sd-server takes whole or as a diffusion model beside
    /// its other parts. An audiocpp_server model is its directory when it
    /// names none.
    pub fn model_files(self) -> &'static [&'static str] {
        match self {
            BackendKind::LlamaServer | BackendKind::WhisperServer => &["model"],
            BackendKind::SdServer => &["model", "diffusion-model"],
            BackendKind::AudiocppServer => &[],
        }
    }

    /// Whether this kind's server reads the model's `file`, given to its
    /// flag `flag`, as a safetensors index, and the shards it names:
    /// sd-server's model loader reads every `.json` file so, and
    /// `--tokenizer`, which goes to its tokenizer, never does.
    pub fn reads_as_index(self, flag: &str, file: &Path) -> bool {
        self == BackendKind::SdServer
            && flag != "tokenizer"
            && file.extension().is_some_and(|ext| ext == "json")
    }

    /// The files a model of this kind may name, when they are not flags of
    /// its server: audiocpp_server reads its model from its configuration.
    fn only_files(self) -> Option<&'static [&'static str]> {
        match self {
            BackendKind::AudiocppServer => Some(&["model"]),
            _ => None,
        }
    }
}

impl fmt::Display for BackendKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

/// Where a backend's program comes from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BackendSource {
    /// The current native build of a project, such as `llama.cpp`.
    Build(String),
    /// An official container image reference.
    Image(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Backend {
    pub id: String,
    pub kind: BackendKind,
    pub source: BackendSource,
}

/// The projects `backends` run a build of, each once, by name; a backend
/// that runs an image builds none.
pub fn built_projects<'a>(backends: impl IntoIterator<Item = &'a Backend>) -> BTreeSet<&'a str> {
    backends
        .into_iter()
        .filter_map(|backend| match &backend.source {
            BackendSource::Build(project) => Some(project.as_str()),
            BackendSource::Image(_) => None,
        })
        .collect()
}

/// The value of a server flag. `Bool(true)` passes the flag alone,
/// `Bool(false)` leaves it out.
#[derive(Debug, Clone, PartialEq)]
pub enum FlagValue {
    Bool(bool),
    Int(i64),
    Float(f64),
    Text(String),
}

/// Server flags by long name, without the leading dashes.
pub type Flags = BTreeMap<String, FlagValue>;

#[derive(Debug, Clone, PartialEq)]
pub struct Model {
    /// The ID clients put in a request's `model` field.
    pub id: String,
    /// The ID of the backend that serves it.
    pub backend: String,
    /// The Hugging Face repository its files come from.
    pub repo: Option<String>,
    /// Its directory, relative to the models directory.
    pub dir: PathBuf,
    /// Its files by server flag (`model`, `mmproj`…), relative to `dir`.
    pub files: BTreeMap<String, PathBuf>,
    /// Flags added to, or overriding, the kind's defaults.
    pub flags: Flags,
    /// Context size in tokens; required by `llama-server`.
    pub ctx_size: Option<u32>,
    /// VRAM the model needs once loaded, in MiB.
    pub vram_mib: u32,
    /// Seconds of inactivity before the gateway stops it; never when absent.
    pub ttl: Option<u32>,
    /// Loaded when the gateway starts.
    pub preload: bool,
    /// What an audiocpp_server model's configuration says of it.
    pub audiocpp: AudiocppFields,
}

/// The fields of an audiocpp_server model beside its files, which its
/// server reads from the configuration ai-ctl writes; none applies to
/// another kind. ai-ctl checks no value against a list: the server knows
/// its families, and checks them when it loads the model.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct AudiocppFields {
    /// The model's family, such as `qwen3_tts`; every audiocpp_server
    /// model names one.
    pub family: Option<String>,
    /// What the model does, such as `tts` or `asr`.
    pub task: Option<String>,
    /// How it runs, such as `offline`.
    pub mode: Option<String>,
    /// Options of the model's session, such as `qwen3_tts.weight_type`.
    pub session_options: Flags,
    /// Its voice library, relative to the models directory.
    pub voices: Option<PathBuf>,
}

/// What a model reads or writes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Modality {
    Text,
    Image,
    Audio,
}

impl Modality {
    /// Its name in llama-swap's configuration.
    pub fn name(self) -> &'static str {
        match self {
            Modality::Text => "text",
            Modality::Image => "image",
            Modality::Audio => "audio",
        }
    }
}

/// What a model can do, told by the inventory alone, so that a client can
/// pick a model by it before any model is loaded.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Capabilities {
    /// What its requests carry.
    pub input: &'static [Modality],
    /// What it answers with.
    pub output: &'static [Modality],
    /// It calls the tools a request describes.
    pub tools: bool,
    /// The tokens one request may hold, prompt and answer together.
    pub context: Option<u32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Gateway {
    pub listen: SocketAddr,
    /// GPU memory, in MiB, that the models running together leave free.
    pub vram_margin_mib: u32,
    /// Whether `listen` may be an address other machines reach.
    pub expose: bool,
}

impl Gateway {
    /// llama-server's `fit-target` in the shipped defaults: a model started
    /// beside another finds the memory its `--fit` keeps free, and moves no
    /// layer to the CPU.
    pub const DEFAULT_VRAM_MARGIN_MIB: u32 = 1024;
}

impl Default for Gateway {
    fn default() -> Self {
        Self {
            listen: SocketAddr::from(([127, 0, 0, 1], 8080)),
            vram_margin_mib: Self::DEFAULT_VRAM_MARGIN_MIB,
            expose: false,
        }
    }
}

/// The raw content of an inventory, before validation.
#[derive(Debug, Clone, PartialEq)]
pub struct InventoryParts {
    pub models_dir: PathBuf,
    pub gateway: Gateway,
    pub defaults: BTreeMap<BackendKind, Flags>,
    pub backends: Vec<Backend>,
    pub models: Vec<Model>,
}

/// A validated inventory.
#[derive(Debug, Clone, PartialEq)]
pub struct Inventory {
    parts: InventoryParts,
}

/// A rule an inventory breaks.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum InventoryError {
    #[error(
        "models_dir `{0}` is not an absolute path (install the inventory with `ai-ctl init --models-dir <dir>`)"
    )]
    RelativeModelsDir(PathBuf),
    #[error(
        "{what} id `{id}` must start with a letter or a digit and hold only ASCII letters, digits, `.`, `_` and `-`"
    )]
    InvalidId { what: &'static str, id: String },
    #[error("backend `{0}` is declared more than once")]
    DuplicateBackend(String),
    #[error("model `{0}` is declared more than once")]
    DuplicateModel(String),
    #[error("model `{model}` names backend `{backend}`, which is not declared")]
    UnknownBackend { model: String, backend: String },
    #[error("model `{0}` needs a VRAM budget above zero (vram_mib)")]
    ZeroVram(String),
    #[error("model `{0}` needs a context size above zero (ctx_size)")]
    MissingContextSize(String),
    #[error("model `{0}`: a ttl of zero is not allowed; leave ttl out to keep the model loaded")]
    ZeroTtl(String),
    #[error("model `{model}` has no {} file, which every {kind} model needs", one_of(.files))]
    MissingFile {
        model: String,
        kind: BackendKind,
        files: &'static [&'static str],
    },
    #[error(
        "model `{model}` names a `{file}` file, but {kind} models take only {}",
        one_of(.allowed)
    )]
    UnusedFile {
        model: String,
        kind: BackendKind,
        file: String,
        allowed: &'static [&'static str],
    },
    #[error("model `{model}` has no `{field}`, which every {kind} model needs")]
    MissingField {
        model: String,
        kind: BackendKind,
        field: &'static str,
    },
    #[error("model `{model}`: `{field}` does not apply to {kind} models")]
    NotForKind {
        model: String,
        field: &'static str,
        kind: BackendKind,
    },
    #[error("backend `{backend}` runs {kind}, but a build of {project} serves {server}")]
    SourceKind {
        backend: String,
        kind: BackendKind,
        project: String,
        server: &'static str,
    },
    #[error(
        "backend `{backend}`: ai-ctl runs {kind} from a build of {project}, \
         `source = {{ build = \"{project}\" }}`; an image serves llama-server only"
    )]
    ImageKind {
        backend: String,
        kind: BackendKind,
        project: &'static str,
    },
    #[error("model `{model}`: path `{path}` must be relative and stay inside its directory")]
    UnsafePath { model: String, path: PathBuf },
    #[error("{scope}: flag `{flag}` is set by ai-ctl and cannot be overridden")]
    ReservedFlag { scope: String, flag: String },
    #[error("{scope}: flag `{flag}` must be written without leading dashes")]
    DashedFlag { scope: String, flag: String },
    #[error(
        "{scope}: flag `{flag}` must be written `{}`: llama-server reads each `_` of a flag as `-`",
        .flag.replace('_', "-")
    )]
    UnderscoredFlag { scope: String, flag: String },
    #[error(
        "{scope}: flag `{flag}` has the value `{value}`, which {kind} would read as an \
         option; a text value cannot start with `-`"
    )]
    OptionValue {
        scope: String,
        kind: BackendKind,
        flag: String,
        value: String,
    },
    #[error("{scope}: flag `{flag}` is not a finite number; a number flag takes a finite value")]
    NonFiniteValue { scope: String, flag: String },
    #[error(
        "gateway.listen `{0}` is not a loopback address, so the gateway would serve the network; \
         set `expose = true` under [gateway] to allow it, every request but /health then needing \
         ai-ctl's API key"
    )]
    ExposedListen(SocketAddr),
}

/// Every rule an inventory breaks.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("{}", joined(.0))]
pub struct InvalidInventory(pub Vec<InventoryError>);

/// An inventory file of a version this ai-ctl does not read.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("unsupported inventory version {found} (this ai-ctl reads version {reads})")]
pub struct UnsupportedVersion {
    pub found: u32,
    pub reads: u32,
}

/// `files` as a message names them: "`model`", "`model` or `vae`".
fn one_of(files: &[&str]) -> String {
    files
        .iter()
        .map(|file| format!("`{file}`"))
        .collect::<Vec<_>>()
        .join(" or ")
}

impl Inventory {
    /// Validates `parts`, reporting every rule broken.
    pub fn new(parts: InventoryParts) -> Result<Self, InvalidInventory> {
        let errors = validate(&parts);
        if errors.is_empty() {
            Ok(Self { parts })
        } else {
            Err(InvalidInventory(errors))
        }
    }

    pub fn models_dir(&self) -> &Path {
        &self.parts.models_dir
    }

    pub fn gateway(&self) -> Gateway {
        self.parts.gateway
    }

    /// Flags shared by every model of `kind`.
    pub fn defaults(&self, kind: BackendKind) -> Option<&Flags> {
        self.parts.defaults.get(&kind)
    }

    /// The flags `model`, served by a backend of `kind`, runs with: the
    /// kind's defaults, overlaid with the model's own.
    pub fn flags(&self, kind: BackendKind, model: &Model) -> Flags {
        let mut flags = self.defaults(kind).cloned().unwrap_or_default();
        flags.extend(model.flags.clone());
        flags
    }

    pub fn backends(&self) -> &[Backend] {
        &self.parts.backends
    }

    pub fn models(&self) -> &[Model] {
        &self.parts.models
    }

    pub fn backend(&self, id: &str) -> Option<&Backend> {
        self.parts.backends.iter().find(|backend| backend.id == id)
    }

    /// The absolute directory holding `model`'s files.
    pub fn model_dir(&self, model: &Model) -> PathBuf {
        self.parts.models_dir.join(&model.dir)
    }

    /// What `model`, served by a backend of `kind`, can do: every
    /// llama-server model calls tools, and reads images when it has an
    /// `mmproj` file; sd-server takes reference images as well as a prompt.
    /// `None` for an audiocpp_server model whose task is neither `asr` nor
    /// `tts`, which the inventory cannot tell more of.
    pub fn capabilities(&self, kind: BackendKind, model: &Model) -> Option<Capabilities> {
        use Modality::{Audio, Image, Text};
        let declared = |input, output| Capabilities {
            input,
            output,
            tools: false,
            context: None,
        };
        let capabilities = match kind {
            BackendKind::LlamaServer => Capabilities {
                input: if model.files.contains_key("mmproj") {
                    &[Text, Image]
                } else {
                    &[Text]
                },
                output: &[Text],
                tools: true,
                context: model.ctx_size.map(|ctx| self.request_window(model, ctx)),
            },
            BackendKind::WhisperServer => declared(&[Audio], &[Text]),
            BackendKind::SdServer => declared(&[Text, Image], &[Image]),
            BackendKind::AudiocppServer => match model.audiocpp.task.as_deref() {
                Some("asr") => declared(&[Audio], &[Text]),
                Some("tts") => declared(&[Text], &[Audio]),
                _ => return None,
            },
        };
        Some(capabilities)
    }

    /// The tokens one request to llama-server may hold, of its context of
    /// `ctx_size`: its slots share the whole context when their KV cache is
    /// unified, and split it evenly otherwise, each up to
    /// `kv-unified-per-slot` when set. With no `parallel`, llama-server
    /// picks four slots and unifies their cache (`tools/server/server.cpp`
    /// at b12111). It pads the context to 256 tokens, which this window
    /// leaves out, so that it never exceeds a slot's context. It also caps
    /// a slot at the model's training context, which the inventory does not
    /// read: within it, llama-server refuses no prompt shorter than the
    /// window (`n_ctx_slot` in `tools/server/server-context.cpp`).
    fn request_window(&self, model: &Model, ctx_size: u32) -> u32 {
        let flags = self.flags(BackendKind::LlamaServer, model);
        let flag = |name: &str| flags.get(name);
        // flags writes kv-unified before no-kv-unified, and llama-server
        // keeps the last of the two.
        let set = |name: &str| flag(name) == Some(&FlagValue::Bool(true));
        let unified = set("kv-unified") && !set("no-kv-unified");
        let window = match flag("parallel").and_then(count) {
            Some(slots) if !unified => ctx_size.checked_div(slots).unwrap_or(ctx_size),
            _ => ctx_size,
        };
        match flag("kv-unified-per-slot").and_then(count) {
            Some(cap) if cap > 0 => window.min(cap),
            _ => window,
        }
    }
}

/// A number of slots or tokens a flag gives llama-server, which reads a
/// negative one as its own choice.
fn count(value: &FlagValue) -> Option<u32> {
    match value {
        FlagValue::Int(number) => u32::try_from(*number).ok(),
        FlagValue::Text(text) => text.parse().ok(),
        FlagValue::Bool(_) | FlagValue::Float(_) => None,
    }
}

fn validate(parts: &InventoryParts) -> Vec<InventoryError> {
    let mut errors = Vec::new();

    if !parts.models_dir.is_absolute() {
        errors.push(InventoryError::RelativeModelsDir(parts.models_dir.clone()));
    }

    let gateway = parts.gateway;
    if !gateway.listen.ip().is_loopback() && !gateway.expose {
        errors.push(InventoryError::ExposedListen(gateway.listen));
    }

    for (kind, flags) in &parts.defaults {
        check_flags(&format!("defaults.{kind}"), *kind, flags, &[], &mut errors);
    }

    let mut backend_ids = HashSet::new();
    for backend in &parts.backends {
        errors.extend(check_id("backend", &backend.id).err());
        if !backend_ids.insert(backend.id.as_str()) {
            errors.push(InventoryError::DuplicateBackend(backend.id.clone()));
        }
        errors.extend(check_source(backend));
    }

    let mut model_ids = HashSet::new();
    for model in &parts.models {
        errors.extend(check_id("model", &model.id).err());
        if !model_ids.insert(model.id.as_str()) {
            errors.push(InventoryError::DuplicateModel(model.id.clone()));
        }
        check_model(parts, model, &mut errors);
    }

    errors
}

/// That `backend` runs its kind's server: a build of the project that
/// serves it, or for llama-server an image too. A project with no recipe is
/// left to the checks of its build.
fn check_source(backend: &Backend) -> Option<InventoryError> {
    let kind = backend.kind;
    match &backend.source {
        BackendSource::Build(project) => {
            let server = recipe(project)?.binary();
            (server != kind.name()).then(|| InventoryError::SourceKind {
                backend: backend.id.clone(),
                kind,
                project: project.clone(),
                server,
            })
        }
        BackendSource::Image(_) => {
            (kind != BackendKind::LlamaServer).then(|| InventoryError::ImageKind {
                backend: backend.id.clone(),
                kind,
                project: kind.project(),
            })
        }
    }
}

/// Whether an image reference names its content by digest,
/// `…@sha256:<64 lower-case hex digits>`, which no registry can move as it
/// moves a tag.
pub fn is_pinned(image: &str) -> bool {
    image
        .rsplit_once("@sha256:")
        .is_some_and(|(_, hex)| Sha256::parse(hex).is_ok())
}

/// Whether `id` can name a model or a backend: an ID names a container,
/// which takes these characters only.
fn is_valid_id(id: &str) -> bool {
    let valid = |c: char| c.is_ascii_alphanumeric() || "._-".contains(c);
    id.starts_with(|c: char| c.is_ascii_alphanumeric()) && id.chars().all(valid)
}

/// That `id` can name a `what`, a model or a backend, per [`is_valid_id`].
pub fn check_id(what: &'static str, id: &str) -> Result<(), InventoryError> {
    if is_valid_id(id) {
        Ok(())
    } else {
        Err(InventoryError::InvalidId {
            what,
            id: id.to_owned(),
        })
    }
}

fn check_model(parts: &InventoryParts, model: &Model, errors: &mut Vec<InventoryError>) {
    let id = &model.id;
    if model.vram_mib == 0 {
        errors.push(InventoryError::ZeroVram(id.clone()));
    }
    if model.ttl == Some(0) {
        errors.push(InventoryError::ZeroTtl(id.clone()));
    }
    let voices = model.audiocpp.voices.iter();
    for path in std::iter::once(&model.dir)
        .chain(model.files.values())
        .chain(voices)
    {
        if !is_contained(path) {
            errors.push(InventoryError::UnsafePath {
                model: id.clone(),
                path: path.clone(),
            });
        }
    }

    let Some(backend) = parts.backends.iter().find(|b| b.id == model.backend) else {
        errors.push(InventoryError::UnknownBackend {
            model: id.clone(),
            backend: model.backend.clone(),
        });
        return;
    };

    let kind = backend.kind;
    check_audiocpp(model, kind, errors);
    match (kind, model.ctx_size) {
        (BackendKind::LlamaServer, None | Some(0)) => {
            errors.push(InventoryError::MissingContextSize(id.clone()));
        }
        (BackendKind::LlamaServer, Some(_)) | (_, None) => {}
        (_, Some(_)) => errors.push(InventoryError::NotForKind {
            model: id.clone(),
            field: "ctx_size",
            kind,
        }),
    }
    let required = kind.model_files();
    if !required.is_empty() && !required.iter().any(|file| model.files.contains_key(*file)) {
        errors.push(InventoryError::MissingFile {
            model: id.clone(),
            kind,
            files: required,
        });
    }
    if let Some(allowed) = kind.only_files() {
        for file in model.files.keys() {
            if !allowed.contains(&file.as_str()) {
                errors.push(InventoryError::UnusedFile {
                    model: id.clone(),
                    kind,
                    file: file.clone(),
                    allowed,
                });
            }
        }
        check_flags(&format!("model `{id}`"), kind, &model.flags, &[], errors);
        return;
    }
    // A file sets its flag: none of those ai-ctl sets, but the one it takes
    // the required file from.
    let scope = format!("model `{id}` files");
    for flag in model.files.keys() {
        check_flag_name(&scope, kind, flag, errors);
        if !required.contains(&flag.as_str()) && kind.reserved_flags().contains(&flag.as_str()) {
            errors.push(InventoryError::ReservedFlag {
                scope: scope.clone(),
                flag: flag.clone(),
            });
        }
    }
    // A default ai-ctl sets is refused with the defaults already.
    if let Some(defaults) = parts.defaults.get(&kind) {
        let set_by_a_file = |flag: &&String| {
            model.files.contains_key(*flag) && !kind.reserved_flags().contains(&flag.as_str())
        };
        for flag in defaults.keys().filter(set_by_a_file) {
            errors.push(InventoryError::ReservedFlag {
                scope: format!("defaults.{kind}, with model `{id}`'s files"),
                flag: flag.clone(),
            });
        }
    }
    let file_flags: Vec<&str> = model.files.keys().map(String::as_str).collect();
    check_flags(
        &format!("model `{id}`"),
        kind,
        &model.flags,
        &file_flags,
        errors,
    );
}

/// An audiocpp_server model names its family, and its session options are
/// finite numbers, which JSON cannot write otherwise; another kind takes
/// none of these fields.
fn check_audiocpp(model: &Model, kind: BackendKind, errors: &mut Vec<InventoryError>) {
    let id = &model.id;
    let fields = &model.audiocpp;
    let scope = format!("model `{id}` session_options");
    for (option, value) in &fields.session_options {
        check_finite(&scope, option, value, errors);
    }
    if kind == BackendKind::AudiocppServer {
        if fields.family.as_deref().is_none_or(str::is_empty) {
            errors.push(InventoryError::MissingField {
                model: id.clone(),
                kind,
                field: "family",
            });
        }
        return;
    }
    let set = [
        ("family", fields.family.is_some()),
        ("task", fields.task.is_some()),
        ("mode", fields.mode.is_some()),
        ("session_options", !fields.session_options.is_empty()),
        ("voices", fields.voices.is_some()),
    ];
    for (field, _) in set.into_iter().filter(|&(_, set)| set) {
        errors.push(InventoryError::NotForKind {
            model: id.clone(),
            field,
            kind,
        });
    }
}

fn check_flags(
    scope: &str,
    kind: BackendKind,
    flags: &Flags,
    also_reserved: &[&str],
    errors: &mut Vec<InventoryError>,
) {
    for (flag, value) in flags {
        check_flag_name(scope, kind, flag, errors);
        if kind.reserved_flags().contains(&flag.as_str()) || also_reserved.contains(&flag.as_str())
        {
            errors.push(InventoryError::ReservedFlag {
                scope: scope.to_owned(),
                flag: flag.clone(),
            });
        }
        // A text is a word of its own after its flag: after a flag that takes
        // no value, the server would read one that starts with `-` as the
        // next option, a reserved one included; audiocpp_server reads any
        // word equal to an option's name as that option.
        if let FlagValue::Text(text) = value
            && text.starts_with('-')
        {
            errors.push(InventoryError::OptionValue {
                scope: scope.to_owned(),
                kind,
                flag: flag.clone(),
                value: text.clone(),
            });
        }
        check_finite(scope, flag, value, errors);
    }
}

/// That a number `value` of `flag` is finite. TOML reads `nan` and `inf` as
/// floats: llama-server would take them, the run store could not save the
/// run's setup after the run, and JSON cannot write them.
fn check_finite(scope: &str, flag: &str, value: &FlagValue, errors: &mut Vec<InventoryError>) {
    if let FlagValue::Float(number) = value
        && !number.is_finite()
    {
        errors.push(InventoryError::NonFiniteValue {
            scope: scope.to_owned(),
            flag: flag.to_owned(),
        });
    }
}

/// A flag is written as its server's long name without its dashes. For
/// llama-server, an underscore would pass the checks against the reserved
/// and the file flags, then reach the server, which reads it as a dash; the
/// other servers read a name as written, sd-server's own holding `_`.
fn check_flag_name(scope: &str, kind: BackendKind, flag: &str, errors: &mut Vec<InventoryError>) {
    if flag.starts_with('-') {
        errors.push(InventoryError::DashedFlag {
            scope: scope.to_owned(),
            flag: flag.to_owned(),
        });
    } else if kind == BackendKind::LlamaServer && flag.contains('_') {
        errors.push(InventoryError::UnderscoredFlag {
            scope: scope.to_owned(),
            flag: flag.to_owned(),
        });
    }
}

/// True for a non-empty relative path made of plain components only.
fn is_contained(path: &Path) -> bool {
    path.components().next().is_some()
        && path
            .components()
            .all(|component| matches!(component, Component::Normal(_)))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn native() -> Backend {
        Backend {
            id: "llama-native".into(),
            kind: BackendKind::LlamaServer,
            source: BackendSource::Build("llama.cpp".into()),
        }
    }

    fn model(id: &str) -> Model {
        Model {
            id: id.into(),
            backend: "llama-native".into(),
            repo: Some("unsloth/Qwen3.8-27B-GGUF".into()),
            dir: "qwen3.8-27b".into(),
            files: BTreeMap::from([
                ("model".into(), "Qwen3.8-27B-UD-Q4_K_M.gguf".into()),
                ("mmproj".into(), "mmproj-F16.gguf".into()),
            ]),
            flags: Flags::from([("cache-type-k".into(), FlagValue::Text("q8_0".into()))]),
            ctx_size: Some(98304),
            vram_mib: 22733,
            ttl: Some(1800),
            preload: true,
            audiocpp: Default::default(),
        }
    }

    fn parts() -> InventoryParts {
        InventoryParts {
            models_dir: "/models".into(),
            gateway: Gateway::default(),
            defaults: BTreeMap::from([(
                BackendKind::LlamaServer,
                Flags::from([("flash-attn".into(), FlagValue::Text("on".into()))]),
            )]),
            backends: vec![native()],
            models: vec![model("qwen3.8-27b")],
        }
    }

    fn errors(parts: InventoryParts) -> Vec<InventoryError> {
        Inventory::new(parts)
            .expect_err("the inventory is invalid")
            .0
    }

    #[test]
    fn an_image_is_pinned_by_a_sha256_digest_only() {
        let hex = "42e80867b03cc22e8cf744c07e0b3194f078838ab6c476095ecdc1621271ec95";
        for image in [
            format!("ghcr.io/ggml-org/llama.cpp:server-cuda13-b11541@sha256:{hex}"),
            format!("ghcr.io/ggml-org/llama.cpp@sha256:{hex}"),
        ] {
            assert!(is_pinned(&image), "{image}");
        }
        for image in [
            "ghcr.io/ggml-org/llama.cpp:server-cuda13".to_owned(),
            "ghcr.io/ggml-org/llama.cpp".to_owned(),
            "ghcr.io/ggml-org/llama.cpp@sha256:42e8".to_owned(),
            format!("ghcr.io/ggml-org/llama.cpp@sha256:{}", hex.to_uppercase()),
            format!("ghcr.io/ggml-org/llama.cpp@sha512:{hex}"),
        ] {
            assert!(!is_pinned(&image), "{image}");
        }
    }

    #[test]
    fn a_valid_inventory_is_accepted() {
        let inventory = Inventory::new(parts()).unwrap();
        assert_eq!(inventory.models().len(), 1);
        assert_eq!(inventory.backend("llama-native"), Some(&native()));
        assert_eq!(
            inventory.gateway().listen,
            "127.0.0.1:8080".parse().unwrap()
        );
        assert!(inventory.defaults(BackendKind::LlamaServer).is_some());
    }

    #[test]
    fn a_models_directory_is_under_the_models_directory() {
        let inventory = Inventory::new(parts()).unwrap();
        let model = &inventory.models()[0];
        assert_eq!(
            inventory.model_dir(model),
            PathBuf::from("/models/qwen3.8-27b")
        );
    }

    #[test]
    fn model_ids_must_be_unique() {
        let mut parts = parts();
        parts.models.push(model("qwen3.8-27b"));
        assert_eq!(
            errors(parts),
            [InventoryError::DuplicateModel("qwen3.8-27b".into())]
        );
    }

    #[test]
    fn backend_ids_must_be_unique() {
        let mut parts = parts();
        parts.backends.push(native());
        assert_eq!(
            errors(parts),
            [InventoryError::DuplicateBackend("llama-native".into())]
        );
    }

    #[test]
    fn an_id_check_names_what_the_id_is_for() {
        assert_eq!(check_id("model", "Qwen_3.8-27b"), Ok(()));
        let invalid = check_id("model", "../health").unwrap_err();
        assert_eq!(
            invalid,
            InventoryError::InvalidId {
                what: "model",
                id: "../health".into()
            }
        );
        assert_eq!(
            invalid.to_string(),
            "model id `../health` must start with a letter or a digit and hold only ASCII \
             letters, digits, `.`, `_` and `-`"
        );
    }

    #[test]
    fn ids_hold_letters_digits_dots_underscores_and_dashes() {
        let mut valid = parts();
        valid.models[0].id = "Qwen_3.8-27b".into();
        assert!(Inventory::new(valid).is_ok());

        for id in [
            "", "qwen 27b", "qwen/27b", "qwen:27b", "-qwen", ".qwen", "qwén",
        ] {
            let mut parts = parts();
            parts.models[0].id = id.into();
            parts.backends.push(Backend {
                id: id.into(),
                ..native()
            });
            assert_eq!(
                errors(parts),
                [
                    InventoryError::InvalidId {
                        what: "backend",
                        id: id.into()
                    },
                    InventoryError::InvalidId {
                        what: "model",
                        id: id.into()
                    },
                ],
                "{id:?}"
            );
        }
    }

    #[test]
    fn a_model_must_name_a_declared_backend() {
        let mut parts = parts();
        parts.models[0].backend = "llama-cuda13".into();
        assert_eq!(
            errors(parts),
            [InventoryError::UnknownBackend {
                model: "qwen3.8-27b".into(),
                backend: "llama-cuda13".into()
            }]
        );
    }

    #[test]
    fn budgets_and_context_sizes_must_be_positive() {
        let mut parts = parts();
        parts.models[0].vram_mib = 0;
        parts.models[0].ctx_size = Some(0);
        parts.models[0].ttl = Some(0);
        let mut second = model("qwen3.6-35b-a3b");
        second.ctx_size = None;
        parts.models.push(second);
        assert_eq!(
            errors(parts),
            [
                InventoryError::ZeroVram("qwen3.8-27b".into()),
                InventoryError::ZeroTtl("qwen3.8-27b".into()),
                InventoryError::MissingContextSize("qwen3.8-27b".into()),
                InventoryError::MissingContextSize("qwen3.6-35b-a3b".into()),
            ]
        );
    }

    #[test]
    fn a_llama_server_model_needs_a_model_file() {
        let mut parts = parts();
        parts.models[0].files.remove("model");
        assert_eq!(
            errors(parts),
            [InventoryError::MissingFile {
                model: "qwen3.8-27b".into(),
                kind: BackendKind::LlamaServer,
                files: &["model"]
            }]
        );
    }

    #[test]
    fn the_projects_built_are_those_of_the_build_backends_each_once() {
        let image = Backend {
            id: "llama-image".into(),
            kind: BackendKind::LlamaServer,
            source: BackendSource::Image("ghcr.io/ggml-org/llama.cpp:server-cuda".into()),
        };
        let backends = [
            backend("whisper", BackendKind::WhisperServer),
            native(),
            image,
            backend("llama-other", BackendKind::LlamaServer),
        ];
        assert_eq!(
            built_projects(&backends),
            BTreeSet::from(["llama.cpp", "whisper.cpp"])
        );
        assert!(built_projects(&backends[2..3]).is_empty());
        assert!(built_projects([]).is_empty());
    }

    /// A backend of `kind`, from a build of its project.
    fn backend(id: &str, kind: BackendKind) -> Backend {
        Backend {
            id: id.into(),
            kind,
            source: BackendSource::Build(kind.project().into()),
        }
    }

    /// A model of `backend`, with no flag and no file.
    fn bare(id: &str, backend: &str, dir: &str, vram_mib: u32) -> Model {
        Model {
            id: id.into(),
            backend: backend.into(),
            repo: None,
            dir: dir.into(),
            files: BTreeMap::new(),
            flags: Flags::new(),
            ctx_size: None,
            vram_mib,
            ttl: None,
            preload: false,
            audiocpp: Default::default(),
        }
    }

    /// One model of each audio and image kind, beside the LLM.
    fn audio_and_image() -> InventoryParts {
        let mut parts = parts();
        parts.backends.extend([
            backend("whisper-native", BackendKind::WhisperServer),
            backend("sd-native", BackendKind::SdServer),
            backend("audio-native", BackendKind::AudiocppServer),
        ]);
        let mut whisper = bare(
            "whisper-large-v3-turbo",
            "whisper-native",
            "stt/whisper-large-v3-turbo",
            2213,
        );
        whisper
            .files
            .insert("model".into(), "ggml-large-v3-turbo.bin".into());
        let mut flux = bare(
            "flux-3-image-klein",
            "sd-native",
            "image/flux-3-image-klein",
            18591,
        );
        flux.files = BTreeMap::from([
            ("diffusion-model".into(), "model.safetensors".into()),
            ("vae".into(), "ae.safetensors".into()),
            ("llm".into(), "Qwen3VL-4B-Instruct-Q8_0.gguf".into()),
        ]);
        flux.flags = Flags::from([
            ("cfg-scale".into(), FlagValue::Float(1.0)),
            ("steps".into(), FlagValue::Int(16)),
        ]);
        // A model that is a directory of safetensors names no file.
        let mut qwen3_tts = bare("qwen3-tts", "audio-native", "tts/qwen3-tts-1.7b-base", 6200);
        qwen3_tts.audiocpp = AudiocppFields {
            family: Some("qwen3_tts".into()),
            task: Some("tts".into()),
            mode: Some("offline".into()),
            session_options: Flags::from([(
                "qwen3_tts.weight_type".into(),
                FlagValue::Text("q8_0".into()),
            )]),
            voices: Some("voices".into()),
        };
        parts.models.extend([whisper, flux, qwen3_tts]);
        parts.defaults.insert(
            BackendKind::WhisperServer,
            Flags::from([("threads".into(), FlagValue::Int(8))]),
        );
        parts
    }

    #[test]
    fn the_audio_and_image_kinds_are_accepted() {
        let inventory = Inventory::new(audio_and_image()).unwrap();
        assert_eq!(inventory.models().len(), 4);
        assert!(inventory.defaults(BackendKind::WhisperServer).is_some());
    }

    /// What the model `id` of `inventory` can do.
    fn capabilities_of(inventory: &Inventory, id: &str) -> Option<Capabilities> {
        let model = inventory.models().iter().find(|model| model.id == id);
        let model = model.expect("the model is in the inventory");
        let kind = inventory.backend(&model.backend).unwrap().kind;
        inventory.capabilities(kind, model)
    }

    /// The capabilities of a model that takes `input`, answers `output`,
    /// calls no tool and has no context.
    fn declared(input: &'static [Modality], output: &'static [Modality]) -> Option<Capabilities> {
        Some(Capabilities {
            input,
            output,
            tools: false,
            context: None,
        })
    }

    #[test]
    fn a_chat_model_calls_tools_and_reads_images_with_an_mmproj_file() {
        let mut parts = parts();
        let mut text_only = model("qwen3.6-35b-a3b");
        text_only.files.remove("mmproj");
        text_only.ctx_size = Some(131072);
        parts.models.push(text_only);
        let inventory = Inventory::new(parts).unwrap();
        assert_eq!(
            capabilities_of(&inventory, "qwen3.8-27b"),
            Some(Capabilities {
                input: &[Modality::Text, Modality::Image],
                output: &[Modality::Text],
                tools: true,
                context: Some(98304),
            })
        );
        assert_eq!(
            capabilities_of(&inventory, "qwen3.6-35b-a3b"),
            Some(Capabilities {
                input: &[Modality::Text],
                output: &[Modality::Text],
                tools: true,
                context: Some(131072),
            })
        );
    }

    #[test]
    fn a_chat_models_context_is_the_window_of_one_request() {
        use FlagValue::{Bool, Int, Text};
        type Set<'a> = &'a [(&'a str, FlagValue)];
        // The defaults' flags, the model's own, and the window of a context
        // of 98304 tokens.
        let cases: [(Set, Set, u32); 16] = [
            // llama-server picks four slots, which share the KV cache.
            (&[], &[], 98304),
            (&[("parallel", Int(-1))], &[], 98304),
            (&[("parallel", Int(1))], &[], 98304),
            (&[("parallel", Int(4))], &[], 24576),
            (&[("parallel", Int(1))], &[("parallel", Int(2))], 49152),
            (&[], &[("parallel", Text("2".into()))], 49152),
            // llama-server refuses no slot, which leaves nothing to split.
            (&[("parallel", Int(0))], &[], 98304),
            (
                &[("parallel", Int(2)), ("kv-unified", Bool(true))],
                &[],
                98304,
            ),
            (
                &[("parallel", Int(2)), ("kv-unified", Bool(true))],
                &[("kv-unified", Bool(false))],
                49152,
            ),
            (
                &[("parallel", Int(2)), ("kv-unified", Bool(true))],
                &[("no-kv-unified", Bool(true))],
                49152,
            ),
            // llama-server keeps the later of the two, whichever sets which.
            (
                &[("parallel", Int(2)), ("no-kv-unified", Bool(true))],
                &[("kv-unified", Bool(true))],
                49152,
            ),
            (
                &[("parallel", Int(2)), ("kv-unified", Bool(true))],
                &[("no-kv-unified", Bool(false))],
                98304,
            ),
            // With no `parallel`, llama-server unifies the cache whatever
            // the flags say.
            (&[("no-kv-unified", Bool(true))], &[], 98304),
            (&[], &[("kv-unified-per-slot", Int(65536))], 65536),
            (
                &[("parallel", Int(2))],
                &[("kv-unified-per-slot", Int(65536))],
                49152,
            ),
            (&[("kv-unified-per-slot", Int(0))], &[], 98304),
        ];
        for (defaults, flags, window) in cases {
            let mut parts = parts();
            let shared = parts.defaults.get_mut(&BackendKind::LlamaServer).unwrap();
            shared.extend(defaults.iter().map(|(f, v)| ((*f).to_owned(), v.clone())));
            let own = flags.iter().map(|(f, v)| ((*f).to_owned(), v.clone()));
            parts.models[0].flags.extend(own);
            let inventory = Inventory::new(parts).unwrap();
            let capabilities = capabilities_of(&inventory, "qwen3.8-27b");
            let context = capabilities.and_then(|capabilities| capabilities.context);
            assert_eq!(context, Some(window), "{defaults:?} {flags:?}");
        }
    }

    #[test]
    fn a_whisper_server_model_transcribes() {
        let inventory = Inventory::new(audio_and_image()).unwrap();
        assert_eq!(
            capabilities_of(&inventory, "whisper-large-v3-turbo"),
            declared(&[Modality::Audio], &[Modality::Text])
        );
    }

    #[test]
    fn an_audiocpp_server_model_transcribes_or_speaks_by_its_task() {
        let mut parts = audio_and_image();
        for (id, task) in [
            ("qwen3-asr", Some("asr")),
            ("voice-change", Some("vc")),
            ("untold", None),
        ] {
            let mut model = bare(id, "audio-native", id, 5548);
            model.audiocpp = AudiocppFields {
                family: Some("family".into()),
                task: task.map(str::to_owned),
                ..Default::default()
            };
            parts.models.push(model);
        }
        let inventory = Inventory::new(parts).unwrap();
        assert_eq!(
            capabilities_of(&inventory, "qwen3-asr"),
            declared(&[Modality::Audio], &[Modality::Text])
        );
        assert_eq!(
            capabilities_of(&inventory, "qwen3-tts"),
            declared(&[Modality::Text], &[Modality::Audio])
        );
        assert_eq!(capabilities_of(&inventory, "voice-change"), None);
        assert_eq!(capabilities_of(&inventory, "untold"), None);
    }

    #[test]
    fn an_sd_server_model_makes_images_from_text_and_images() {
        let inventory = Inventory::new(audio_and_image()).unwrap();
        assert_eq!(
            capabilities_of(&inventory, "flux-3-image-klein"),
            declared(&[Modality::Text, Modality::Image], &[Modality::Image])
        );
    }

    #[test]
    fn each_kind_refuses_the_flags_ai_ctl_sets() {
        let reserved = [
            (
                BackendKind::WhisperServer,
                &["host", "port", "model", "inference-path", "request-path"][..],
            ),
            (
                BackendKind::SdServer,
                &[
                    "listen-ip",
                    "listen-port",
                    "eager-load",
                    "model",
                    "diffusion-model",
                    "model-dir",
                ],
            ),
            (
                BackendKind::AudiocppServer,
                &["config", "host", "port", "no-ui", "ui", "ui-management"],
            ),
        ];
        for (kind, flags) in reserved {
            for flag in flags {
                let mut parts = audio_and_image();
                parts
                    .defaults
                    .entry(kind)
                    .or_default()
                    .insert((*flag).into(), FlagValue::Bool(true));
                assert_eq!(
                    errors(parts),
                    [InventoryError::ReservedFlag {
                        scope: format!("defaults.{kind}"),
                        flag: (*flag).into()
                    }],
                    "{kind} {flag}"
                );
            }
        }
    }

    #[test]
    fn a_model_names_the_files_its_kind_needs() {
        let mut parts = audio_and_image();
        // A whole checkpoint, in place of a diffusion model.
        parts.models[2].files = BTreeMap::from([("model".into(), "sd.safetensors".into())]);
        parts.models[3]
            .files
            .insert("model".into(), "qwen3-tts.gguf".into());
        assert!(Inventory::new(parts.clone()).is_ok());

        parts.models[1].files.clear();
        parts.models[2].files = BTreeMap::from([("vae".into(), "ae.safetensors".into())]);
        let missing = |model: &str, kind, files| InventoryError::MissingFile {
            model: model.into(),
            kind,
            files,
        };
        assert_eq!(
            errors(parts),
            [
                missing(
                    "whisper-large-v3-turbo",
                    BackendKind::WhisperServer,
                    &["model"]
                ),
                missing(
                    "flux-3-image-klein",
                    BackendKind::SdServer,
                    &["model", "diffusion-model"]
                ),
            ]
        );
        assert_eq!(
            missing("m", BackendKind::SdServer, &["model", "diffusion-model"]).to_string(),
            "model `m` has no `model` or `diffusion-model` file, which every sd-server model needs"
        );
    }

    #[test]
    fn an_audiocpp_server_model_takes_its_model_file_alone() {
        let mut parts = audio_and_image();
        parts.models[3]
            .files
            .insert("vae".into(), "vae.safetensors".into());
        let unused = InventoryError::UnusedFile {
            model: "qwen3-tts".into(),
            kind: BackendKind::AudiocppServer,
            file: "vae".into(),
            allowed: &["model"],
        };
        assert_eq!(errors(parts), std::slice::from_ref(&unused));
        assert_eq!(
            unused.to_string(),
            "model `qwen3-tts` names a `vae` file, but audiocpp_server models take only `model`"
        );
    }

    #[test]
    fn an_audiocpp_server_model_names_its_family() {
        let mut parts = audio_and_image();
        // The task and the mode have defaults in the server.
        parts.models[3].audiocpp.task = None;
        parts.models[3].audiocpp.mode = None;
        assert!(Inventory::new(parts.clone()).is_ok());

        let missing = InventoryError::MissingField {
            model: "qwen3-tts".into(),
            kind: BackendKind::AudiocppServer,
            field: "family",
        };
        for family in [None, Some(String::new())] {
            parts.models[3].audiocpp.family = family;
            assert_eq!(errors(parts.clone()), std::slice::from_ref(&missing));
        }
        assert_eq!(
            missing.to_string(),
            "model `qwen3-tts` has no `family`, which every audiocpp_server model needs"
        );
    }

    #[test]
    fn audiocpp_fields_apply_to_audiocpp_server_alone() {
        let mut parts = audio_and_image();
        parts.models[1].audiocpp = parts.models[3].audiocpp.clone();
        assert_eq!(
            errors(parts),
            ["family", "task", "mode", "session_options", "voices"].map(|field| {
                InventoryError::NotForKind {
                    model: "whisper-large-v3-turbo".into(),
                    field,
                    kind: BackendKind::WhisperServer,
                }
            })
        );
    }

    #[test]
    fn voices_stay_inside_the_models_directory() {
        let mut parts = audio_and_image();
        parts.models[3].audiocpp.voices = Some("../voices".into());
        assert_eq!(
            errors(parts),
            [InventoryError::UnsafePath {
                model: "qwen3-tts".into(),
                path: "../voices".into()
            }]
        );
    }

    #[test]
    fn a_session_option_must_be_finite() {
        let mut parts = audio_and_image();
        parts.models[3]
            .audiocpp
            .session_options
            .insert("qwen3_tts.temperature".into(), FlagValue::Float(f64::NAN));
        assert_eq!(
            errors(parts),
            [InventoryError::NonFiniteValue {
                scope: "model `qwen3-tts` session_options".into(),
                flag: "qwen3_tts.temperature".into()
            }]
        );
    }

    #[test]
    fn an_underscore_reaches_the_other_servers_as_written() {
        let mut parts = audio_and_image();
        // sd-server's own spelling of its CLIP and vision encoders.
        for file in ["clip_l", "clip_g", "clip_vision"] {
            parts.models[2]
                .files
                .insert(file.into(), format!("{file}.safetensors").into());
        }
        assert!(Inventory::new(parts).is_ok());
    }

    #[test]
    fn a_context_size_is_llama_servers_alone() {
        let mut parts = audio_and_image();
        for model in &mut parts.models[1..] {
            model.ctx_size = Some(4096);
        }
        let not_for = |model: &str, kind| InventoryError::NotForKind {
            model: model.into(),
            field: "ctx_size",
            kind,
        };
        assert_eq!(
            errors(parts),
            [
                not_for("whisper-large-v3-turbo", BackendKind::WhisperServer),
                not_for("flux-3-image-klein", BackendKind::SdServer),
                not_for("qwen3-tts", BackendKind::AudiocppServer),
            ]
        );
        assert_eq!(
            not_for("m", BackendKind::SdServer).to_string(),
            "model `m`: `ctx_size` does not apply to sd-server models"
        );
    }

    #[test]
    fn each_kind_is_served_by_its_projects_build() {
        for kind in BackendKind::ALL {
            assert_eq!(
                recipe(kind.project()).map(|recipe| recipe.binary()),
                Some(kind.name()),
                "{kind}"
            );
        }
    }

    #[test]
    fn sd_server_alone_reads_a_json_file_as_a_safetensors_index() {
        let index = Path::new("transformer/diffusion_pytorch_model.safetensors.index.json");
        let sd = BackendKind::SdServer;
        assert!(sd.reads_as_index("diffusion-model", index));
        assert!(sd.reads_as_index("model", Path::new("weights.json")));
        assert!(!sd.reads_as_index("model", Path::new("model.safetensors")));
        assert!(!sd.reads_as_index("model", Path::new("json")));
        assert!(!sd.reads_as_index("tokenizer", Path::new("tokenizer.json")));
        for kind in [
            BackendKind::LlamaServer,
            BackendKind::WhisperServer,
            BackendKind::AudiocppServer,
        ] {
            assert!(!kind.reads_as_index("model", index), "{kind}");
        }
    }

    #[test]
    fn a_build_must_serve_its_backends_kind() {
        let mut parts = audio_and_image();
        parts.backends[0].source = BackendSource::Build("whisper.cpp".into());
        parts.backends[1].source = BackendSource::Build("stable-diffusion.cpp".into());
        // A project with no recipe is doctor's to report.
        parts.backends[2].source = BackendSource::Build("nope.cpp".into());
        let mismatch = |backend: &str, kind, project: &str, server| InventoryError::SourceKind {
            backend: backend.into(),
            kind,
            project: project.into(),
            server,
        };
        assert_eq!(
            errors(parts),
            [
                mismatch(
                    "llama-native",
                    BackendKind::LlamaServer,
                    "whisper.cpp",
                    "whisper-server"
                ),
                mismatch(
                    "whisper-native",
                    BackendKind::WhisperServer,
                    "stable-diffusion.cpp",
                    "sd-server"
                ),
            ]
        );
        assert_eq!(
            mismatch(
                "whisper-native",
                BackendKind::WhisperServer,
                "stable-diffusion.cpp",
                "sd-server"
            )
            .to_string(),
            "backend `whisper-native` runs whisper-server, but a build of stable-diffusion.cpp \
             serves sd-server"
        );
    }

    #[test]
    fn an_image_serves_llama_server_alone() {
        let mut parts = audio_and_image();
        let image = "ghcr.io/ggml-org/whisper.cpp:main-cuda".to_owned();
        for backend in &mut parts.backends[1..] {
            backend.source = BackendSource::Image(image.clone());
        }
        let image_kind = |backend: &str, kind, project| InventoryError::ImageKind {
            backend: backend.into(),
            kind,
            project,
        };
        assert_eq!(
            errors(parts),
            [
                image_kind("whisper-native", BackendKind::WhisperServer, "whisper.cpp"),
                image_kind("sd-native", BackendKind::SdServer, "stable-diffusion.cpp"),
                image_kind("audio-native", BackendKind::AudiocppServer, "audio.cpp"),
            ]
        );
        assert_eq!(
            image_kind("whisper-native", BackendKind::WhisperServer, "whisper.cpp").to_string(),
            "backend `whisper-native`: ai-ctl runs whisper-server from a build of whisper.cpp, \
             `source = { build = \"whisper.cpp\" }`; an image serves llama-server only"
        );
    }

    #[test]
    fn paths_must_stay_inside_the_models_directory() {
        let mut parts = parts();
        parts.models[0].dir = "../elsewhere".into();
        parts.models[0]
            .files
            .insert("model-draft".into(), "/etc/passwd".into());
        parts.models[0].files.insert("lora".into(), "".into());
        let found = errors(parts);
        assert_eq!(found.len(), 3, "{found:?}");
        assert!(
            found
                .iter()
                .all(|e| matches!(e, InventoryError::UnsafePath { .. }))
        );
    }

    #[test]
    fn models_dir_must_be_absolute() {
        let mut parts = parts();
        parts.models_dir = "${MODELS_DIR}".into();
        assert_eq!(
            errors(parts),
            [InventoryError::RelativeModelsDir("${MODELS_DIR}".into())]
        );
    }

    #[test]
    fn flags_cannot_override_what_ai_ctl_sets() {
        let mut parts = parts();
        let flags = parts.defaults.get_mut(&BackendKind::LlamaServer).unwrap();
        flags.insert("port".into(), FlagValue::Int(9000));
        parts.models[0]
            .flags
            .insert("ctx-size".into(), FlagValue::Int(4096));
        parts.models[0]
            .flags
            .insert("mmproj".into(), FlagValue::Text("x.gguf".into()));
        assert_eq!(
            errors(parts),
            [
                InventoryError::ReservedFlag {
                    scope: "defaults.llama-server".into(),
                    flag: "port".into()
                },
                InventoryError::ReservedFlag {
                    scope: "model `qwen3.8-27b`".into(),
                    flag: "ctx-size".into()
                },
                InventoryError::ReservedFlag {
                    scope: "model `qwen3.8-27b`".into(),
                    flag: "mmproj".into()
                },
            ]
        );
    }

    #[test]
    fn every_flag_ai_ctl_sets_is_refused_in_the_defaults() {
        for flag in ["host", "port", "alias", "metrics", "ctx-size", "model"] {
            let mut parts = parts();
            let flags = parts.defaults.get_mut(&BackendKind::LlamaServer).unwrap();
            flags.insert(flag.into(), FlagValue::Text("x".into()));
            assert_eq!(
                errors(parts),
                [InventoryError::ReservedFlag {
                    scope: "defaults.llama-server".into(),
                    flag: flag.into()
                }],
                "{flag}"
            );
        }
    }

    #[test]
    fn a_file_cannot_set_a_flag_ai_ctl_sets() {
        let mut parts = parts();
        parts.models[0].files.insert("port".into(), "x.gguf".into());
        assert_eq!(
            errors(parts),
            [InventoryError::ReservedFlag {
                scope: "model `qwen3.8-27b` files".into(),
                flag: "port".into()
            }]
        );
    }

    #[test]
    fn a_default_cannot_set_a_flag_a_models_file_sets() {
        let mut parts = parts();
        let flags = parts.defaults.get_mut(&BackendKind::LlamaServer).unwrap();
        flags.insert("mmproj".into(), FlagValue::Text("x.gguf".into()));
        assert_eq!(
            errors(parts),
            [InventoryError::ReservedFlag {
                scope: "defaults.llama-server, with model `qwen3.8-27b`'s files".into(),
                flag: "mmproj".into()
            }]
        );
    }

    #[test]
    fn flags_cannot_give_llama_server_another_api_key() {
        let mut parts = parts();
        for flag in ["api-key", "api-key-file"] {
            parts.models[0]
                .flags
                .insert(flag.into(), FlagValue::Text("k".into()));
        }
        assert_eq!(
            errors(parts),
            ["api-key", "api-key-file"].map(|flag| InventoryError::ReservedFlag {
                scope: "model `qwen3.8-27b`".into(),
                flag: flag.into()
            })
        );
    }

    #[test]
    fn the_gateway_listens_on_loopback_unless_the_inventory_exposes_it() {
        for listen in ["0.0.0.0:8080", "192.168.1.20:8080", "[::]:8080"] {
            let mut parts = parts();
            parts.gateway.listen = listen.parse().unwrap();
            assert_eq!(
                errors(parts.clone()),
                [InventoryError::ExposedListen(listen.parse().unwrap())],
                "{listen}"
            );
            parts.gateway.expose = true;
            assert!(Inventory::new(parts).is_ok(), "{listen}");
        }
        for listen in ["127.0.0.2:9090", "[::1]:8080"] {
            let mut parts = parts();
            parts.gateway.listen = listen.parse().unwrap();
            assert!(Inventory::new(parts).is_ok(), "{listen}");
        }
    }

    #[test]
    fn an_exposed_listen_names_the_opt_in() {
        let mut parts = parts();
        parts.gateway.listen = "0.0.0.0:8080".parse().unwrap();
        assert_eq!(
            Inventory::new(parts).unwrap_err().to_string(),
            "gateway.listen `0.0.0.0:8080` is not a loopback address, so the gateway would \
             serve the network; set `expose = true` under [gateway] to allow it, every \
             request but /health then needing ai-ctl's API key"
        );
    }

    #[test]
    fn flags_are_written_without_dashes() {
        let mut parts = parts();
        parts.models[0]
            .flags
            .insert("--temp".into(), FlagValue::Float(0.6));
        assert_eq!(
            errors(parts),
            [InventoryError::DashedFlag {
                scope: "model `qwen3.8-27b`".into(),
                flag: "--temp".into()
            }]
        );
    }

    #[test]
    fn flags_are_written_with_dashes_since_llama_server_reads_underscores_as_dashes() {
        let mut parts = parts();
        let model = &mut parts.models[0];
        model
            .flags
            .insert("api_key".into(), FlagValue::Text("second".into()));
        model.flags.insert("ctx_size".into(), FlagValue::Int(4096));
        model
            .flags
            .insert("model_draft".into(), FlagValue::Text("other.gguf".into()));
        model.files.insert("api_key_file".into(), "key.txt".into());
        parts
            .defaults
            .entry(BackendKind::LlamaServer)
            .or_default()
            .insert("batch_size".into(), FlagValue::Int(512));
        let underscored = |scope: &str, flag: &str| InventoryError::UnderscoredFlag {
            scope: scope.into(),
            flag: flag.into(),
        };
        assert_eq!(
            errors(parts),
            [
                underscored("defaults.llama-server", "batch_size"),
                underscored("model `qwen3.8-27b` files", "api_key_file"),
                underscored("model `qwen3.8-27b`", "api_key"),
                underscored("model `qwen3.8-27b`", "ctx_size"),
                underscored("model `qwen3.8-27b`", "model_draft"),
            ]
        );
    }

    #[test]
    fn a_text_value_cannot_be_an_option() {
        let mut parts = parts();
        parts.models[0]
            .flags
            .insert("jinja".into(), FlagValue::Text("--api-key".into()));
        parts
            .defaults
            .entry(BackendKind::LlamaServer)
            .or_default()
            .insert("lora".into(), FlagValue::Text("-x".into()));
        let option = |scope: &str, flag: &str, value: &str| InventoryError::OptionValue {
            scope: scope.into(),
            kind: BackendKind::LlamaServer,
            flag: flag.into(),
            value: value.into(),
        };
        assert_eq!(
            errors(parts),
            [
                option("defaults.llama-server", "lora", "-x"),
                option("model `qwen3.8-27b`", "jinja", "--api-key"),
            ]
        );
        assert_eq!(
            option("model `m`", "jinja", "--api-key").to_string(),
            "model `m`: flag `jinja` has the value `--api-key`, which llama-server would read \
             as an option; a text value cannot start with `-`"
        );
    }

    #[test]
    fn a_text_value_cannot_be_an_option_to_any_server() {
        let mut parts = audio_and_image();
        let kinds = [
            BackendKind::WhisperServer,
            BackendKind::SdServer,
            BackendKind::AudiocppServer,
        ];
        for model in &mut parts.models[1..] {
            model
                .flags
                .insert("prompt".into(), FlagValue::Text("--port".into()));
        }
        let found = errors(parts);
        let kinds_named: Vec<BackendKind> = found
            .iter()
            .map(|error| match error {
                InventoryError::OptionValue { kind, .. } => *kind,
                other => panic!("{other}"),
            })
            .collect();
        assert_eq!(kinds_named, kinds);
        assert!(
            found[1]
                .to_string()
                .contains("`--port`, which sd-server would read as an option"),
            "{}",
            found[1]
        );
    }

    #[test]
    fn a_number_value_must_be_finite() {
        let mut parts = parts();
        parts.models[0]
            .flags
            .insert("temp".into(), FlagValue::Float(f64::NAN));
        parts
            .defaults
            .entry(BackendKind::LlamaServer)
            .or_default()
            .insert("top-p".into(), FlagValue::Float(f64::NEG_INFINITY));
        let non_finite = |scope: &str, flag: &str| InventoryError::NonFiniteValue {
            scope: scope.into(),
            flag: flag.into(),
        };
        assert_eq!(
            errors(parts),
            [
                non_finite("defaults.llama-server", "top-p"),
                non_finite("model `qwen3.8-27b`", "temp"),
            ]
        );
        assert_eq!(
            non_finite("model `m`", "temp").to_string(),
            "model `m`: flag `temp` is not a finite number; a number flag takes a finite value"
        );
    }

    #[test]
    fn a_number_value_must_be_finite_for_every_server() {
        let mut parts = audio_and_image();
        for model in &mut parts.models[1..] {
            model
                .flags
                .insert("cfg-scale".into(), FlagValue::Float(f64::INFINITY));
        }
        let non_finite = |model: &str| InventoryError::NonFiniteValue {
            scope: format!("model `{model}`"),
            flag: "cfg-scale".into(),
        };
        assert_eq!(
            errors(parts),
            [
                non_finite("whisper-large-v3-turbo"),
                non_finite("flux-3-image-klein"),
                non_finite("qwen3-tts"),
            ]
        );
    }

    #[test]
    fn an_underscored_flag_names_its_dashed_spelling() {
        let error = InventoryError::UnderscoredFlag {
            scope: "model `m`".into(),
            flag: "ctx_size".into(),
        };
        assert_eq!(
            error.to_string(),
            "model `m`: flag `ctx_size` must be written `ctx-size`: llama-server reads each `_` \
             of a flag as `-`"
        );
    }

    #[test]
    fn file_keys_are_written_without_dashes() {
        let mut parts = parts();
        parts.models[0]
            .files
            .insert("--mmproj".into(), "x.gguf".into());
        assert_eq!(
            errors(parts),
            [InventoryError::DashedFlag {
                scope: "model `qwen3.8-27b` files".into(),
                flag: "--mmproj".into()
            }]
        );
    }

    #[test]
    fn every_error_is_reported_at_once() {
        let mut parts = parts();
        parts.models_dir = "models".into();
        parts.models[0].vram_mib = 0;
        parts.models.push(model("qwen3.8-27b"));
        let message = Inventory::new(parts).unwrap_err().to_string();
        assert!(message.contains("models_dir `models`"), "{message}");
        assert!(message.contains("VRAM budget"), "{message}");
        assert!(message.contains("more than once"), "{message}");
    }

    #[test]
    fn backend_kinds_round_trip_through_their_names() {
        // Each kind is named after its server's binary.
        assert_eq!(
            BackendKind::ALL.map(BackendKind::name),
            [
                "llama-server",
                "whisper-server",
                "sd-server",
                "audiocpp_server"
            ]
        );
        for kind in BackendKind::ALL {
            assert_eq!(BackendKind::from_name(kind.name()), Some(kind));
        }
        assert_eq!(BackendKind::from_name("whisper"), None);
    }
}
