//! The inventory as a TOML file: parsing, and installing the shipped one.

use std::collections::BTreeMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, anyhow, bail, ensure};
use serde::Deserialize;

use crate::application::ports::{Installed, InventoryStore};

use super::files;
use crate::domain::inventory::{
    AudiocppFields, Backend, BackendKind, BackendSource, FlagValue, Flags, Gateway, Inventory,
    InventoryParts, Model, UnsupportedVersion,
};
use crate::domain::text::{is_control_or_separator, unicode_escape};

/// The inventory shipped with ai-ctl, `deploy/inventory.toml`.
pub const SHIPPED: &str = include_str!("../../deploy/inventory.toml");
/// Replaced by the models directory when the shipped inventory is installed.
const PLACEHOLDER: &str = "${MODELS_DIR}";
/// The format version this ai-ctl reads.
const VERSION: u32 = 1;

/// The installed inventory, at a fixed path.
pub struct InventoryFile {
    path: PathBuf,
}

impl InventoryFile {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }
}

impl InventoryStore for InventoryFile {
    fn location(&self) -> &Path {
        &self.path
    }

    fn load(&self) -> Result<Option<Inventory>> {
        let Some(text) = files::read_if_present(&self.path)? else {
            return Ok(None);
        };
        parse(&text)
            .map(Some)
            .with_context(|| format!("invalid inventory {}", self.path.display()))
    }

    fn install(&self, models_dir: &Path) -> Result<Installed> {
        let models_dir = std::path::absolute(models_dir)
            .with_context(|| format!("cannot resolve {}", models_dir.display()))?;
        match fs::metadata(&models_dir) {
            Ok(metadata) if metadata.is_dir() => {}
            Ok(_) => bail!("{} is not a directory", models_dir.display()),
            Err(err) if err.kind() == io::ErrorKind::NotFound => bail!(
                "{} does not exist; create it first (models are downloaded into it)",
                models_dir.display()
            ),
            Err(err) => {
                return Err(err).with_context(|| format!("cannot read {}", models_dir.display()));
            }
        }
        let text = render(SHIPPED, &models_dir)?;
        parse(&text).context("the shipped inventory is invalid")?;
        write_new(&self.path, &text)
    }
}

/// Fills the models directory into `template`, escaped for a TOML string.
pub fn render(template: &str, models_dir: &Path) -> Result<String> {
    let dir = models_dir
        .to_str()
        .with_context(|| format!("{} is not valid UTF-8", models_dir.display()))?;
    ensure!(
        template.contains(PLACEHOLDER),
        "the inventory template has no {PLACEHOLDER} placeholder"
    );
    Ok(template.replace(PLACEHOLDER, &toml_escape(dir)))
}

/// Escapes `text` for the inside of a TOML basic string.
fn toml_escape(text: &str) -> String {
    let mut escaped = String::with_capacity(text.len());
    for c in text.chars() {
        match c {
            '\\' => escaped.push_str(r"\\"),
            '"' => escaped.push_str("\\\""),
            c if is_control_or_separator(c) => escaped.push_str(&unicode_escape(c)),
            c => escaped.push(c),
        }
    }
    escaped
}

/// Writes `text` to `path` unless something exists there already, as
/// [`files::write_new`] does; a link whose target is missing is told
/// apart.
fn write_new(path: &Path, text: &str) -> Result<Installed> {
    if let Some(target) = dangling_target(path)? {
        return Ok(Installed::DanglingLink { target });
    }
    Ok(if files::write_new(path, text, 0o666)? {
        Installed::Written
    } else {
        Installed::AlreadyExists
    })
}

/// The target of `path` when it is a symbolic link to nothing.
fn dangling_target(path: &Path) -> Result<Option<PathBuf>> {
    let is_link = path
        .symlink_metadata()
        .is_ok_and(|metadata| metadata.file_type().is_symlink());
    let missing = matches!(fs::metadata(path), Err(err) if err.kind() == io::ErrorKind::NotFound);
    if !is_link || !missing {
        return Ok(None);
    }
    let target = fs::read_link(path).with_context(|| format!("cannot read {}", path.display()))?;
    Ok(Some(target))
}

/// Parses and validates an inventory.
pub fn parse(text: &str) -> Result<Inventory> {
    // The version alone first: a later one that adds or renames keys is
    // told by its number, not refused by the strict parse of this one.
    #[derive(Deserialize)]
    struct VersionDto {
        version: u32,
    }
    let version = toml::from_str::<VersionDto>(text)?.version;
    if version != VERSION {
        return Err(UnsupportedVersion {
            found: version,
            reads: VERSION,
        }
        .into());
    }
    let file: FileDto = toml::from_str(text)?;

    let gateway = file.gateway.unwrap_or_default();
    let default = Gateway::default();
    let gateway = Gateway {
        listen: match gateway.listen {
            Some(listen) => listen.parse().map_err(|_| {
                anyhow!("gateway.listen `{listen}` is not an address such as 127.0.0.1:8080")
            })?,
            None => default.listen,
        },
        vram_margin_mib: gateway.vram_margin_mib.unwrap_or(default.vram_margin_mib),
        expose: gateway.expose.unwrap_or(default.expose),
    };

    let mut defaults = BTreeMap::new();
    for (kind, flags) in file.defaults {
        let kind = backend_kind(&kind).with_context(|| format!("[defaults.{kind}]"))?;
        defaults.insert(kind, flag_values(flags, &format!("defaults.{kind}"))?);
    }

    let backends = file
        .backends
        .into_iter()
        .map(|backend| {
            let kind =
                backend_kind(&backend.kind).with_context(|| format!("backend `{}`", backend.id))?;
            Ok(Backend {
                id: backend.id,
                kind,
                source: match backend.source {
                    SourceDto::Build(project) => BackendSource::Build(project),
                    SourceDto::Image(reference) => BackendSource::Image(reference),
                },
            })
        })
        .collect::<Result<Vec<_>>>()?;

    let models = file
        .models
        .into_iter()
        .map(|model| {
            let flags = flag_values(model.flags, &format!("model `{}`", model.id))?;
            let scope = format!("model `{}` session_options", model.id);
            let session_options = flag_values(model.session_options, &scope)?;
            Ok(Model {
                id: model.id,
                backend: model.backend,
                repo: model.repo,
                dir: model.dir,
                files: model.files,
                flags,
                ctx_size: model.ctx_size,
                vram_mib: model.vram_mib,
                ttl: model.ttl,
                preload: model.preload,
                audiocpp: AudiocppFields {
                    family: model.family,
                    task: model.task,
                    mode: model.mode,
                    session_options,
                    voices: model.voices,
                },
            })
        })
        .collect::<Result<Vec<_>>>()?;

    Ok(Inventory::new(InventoryParts {
        models_dir: file.models_dir,
        gateway,
        defaults,
        backends,
        models,
    })?)
}

fn backend_kind(name: &str) -> Result<BackendKind> {
    BackendKind::from_name(name).ok_or_else(|| {
        let known: Vec<&str> = BackendKind::ALL.iter().map(|kind| kind.name()).collect();
        anyhow!(
            "unknown backend kind `{name}` (known: {})",
            known.join(", ")
        )
    })
}

fn flag_values(flags: BTreeMap<String, toml::Value>, scope: &str) -> Result<Flags> {
    flags
        .into_iter()
        .map(|(name, value)| {
            let value = match value {
                toml::Value::Boolean(value) => FlagValue::Bool(value),
                toml::Value::Integer(value) => FlagValue::Int(value),
                toml::Value::Float(value) => FlagValue::Float(value),
                toml::Value::String(value) => FlagValue::Text(value),
                other => bail!(
                    "{scope}: flag `{name}` is a {}; a flag takes a string, a number or a boolean",
                    other.type_str()
                ),
            };
            Ok((name, value))
        })
        .collect()
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct FileDto {
    #[expect(
        dead_code,
        reason = "`parse` reads the version alone first; the strict parse only accepts the key"
    )]
    version: u32,
    models_dir: PathBuf,
    gateway: Option<GatewayDto>,
    #[serde(default)]
    defaults: BTreeMap<String, BTreeMap<String, toml::Value>>,
    #[serde(default)]
    backends: Vec<BackendDto>,
    #[serde(default)]
    models: Vec<ModelDto>,
}

#[derive(Deserialize, Default)]
#[serde(deny_unknown_fields)]
struct GatewayDto {
    listen: Option<String>,
    vram_margin_mib: Option<u32>,
    expose: Option<bool>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct BackendDto {
    id: String,
    kind: String,
    source: SourceDto,
}

#[derive(Deserialize)]
#[serde(rename_all = "lowercase", deny_unknown_fields)]
enum SourceDto {
    Build(String),
    Image(String),
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ModelDto {
    id: String,
    backend: String,
    repo: Option<String>,
    dir: PathBuf,
    #[serde(default)]
    files: BTreeMap<String, PathBuf>,
    #[serde(default)]
    flags: BTreeMap<String, toml::Value>,
    ctx_size: Option<u32>,
    vram_mib: u32,
    ttl: Option<u32>,
    #[serde(default)]
    preload: bool,
    family: Option<String>,
    task: Option<String>,
    mode: Option<String>,
    #[serde(default)]
    session_options: BTreeMap<String, toml::Value>,
    voices: Option<PathBuf>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn shipped(models_dir: &str) -> Inventory {
        parse(&render(SHIPPED, Path::new(models_dir)).unwrap()).unwrap()
    }

    #[test]
    fn the_shipped_inventory_has_one_placeholder() {
        assert_eq!(SHIPPED.matches(PLACEHOLDER).count(), 1);
    }

    #[test]
    fn the_shipped_inventory_is_valid_once_rendered() {
        let inventory = shipped("/data/models");
        assert_eq!(inventory.models_dir(), Path::new("/data/models"));
        assert_eq!(
            inventory.gateway().listen,
            "127.0.0.1:8080".parse().unwrap()
        );
        let ids: Vec<&str> = inventory.models().iter().map(|m| m.id.as_str()).collect();
        assert_eq!(
            ids,
            [
                "qwen3.8-27b",
                "qwen3.6-35b-a3b",
                "whisper-large-v3-turbo",
                "whisper-large-v3",
                "qwen3-asr-1.7b",
                "voxtral-4b-tts",
                "qwen3-tts",
                "qwen3-tts-q8",
                "fish-s2-pro-q8",
                "fish-s2-pro-bf16",
                "flux-3-image-klein",
                "qwen-image-2.1",
                "fibo-2.0-dev",
            ]
        );
        let backends: Vec<&str> = inventory.backends().iter().map(|b| b.id.as_str()).collect();
        assert_eq!(
            backends,
            [
                "llama-native",
                "llama-cuda12",
                "llama-cuda13",
                "whisper-native",
                "audio-native",
                "sd-native",
            ]
        );
    }

    #[test]
    fn the_shipped_inventory_carries_the_presets() {
        let inventory = shipped("/data/models");
        let defaults = inventory.defaults(BackendKind::LlamaServer).unwrap();
        assert_eq!(defaults["flash-attn"], FlagValue::Text("on".into()));
        assert_eq!(defaults["cache-ram"], FlagValue::Int(16384));
        assert_eq!(defaults["no-mmproj-offload"], FlagValue::Bool(true));
        assert_eq!(defaults["temp"], FlagValue::Float(0.6));
        assert_eq!(defaults.len(), 13);

        let main = &inventory.models()[0];
        assert_eq!(main.backend, "llama-native");
        assert_eq!(main.ctx_size, Some(98304));
        assert_eq!(main.ttl, Some(1800));
        assert!(main.preload);
        assert_eq!(main.flags["spec-type"], FlagValue::Text("draft-mtp".into()));
        assert_eq!(
            inventory.model_dir(main),
            PathBuf::from("/data/models/llm/qwen3.8-27b")
        );
        assert_eq!(
            main.files,
            BTreeMap::from([
                ("mmproj".into(), PathBuf::from("mmproj-F16.gguf")),
                ("model".into(), PathBuf::from("Qwen3.8-27B-UD-Q4_K_M.gguf")),
                (
                    "model-draft".into(),
                    PathBuf::from("MTP/mtp-Qwen3.8-27B-Q4_0.gguf")
                ),
            ])
        );
        let fast = &inventory.models()[1];
        assert!(!fast.preload);
        assert!(fast.flags.is_empty());
    }

    #[test]
    fn the_readme_example_is_valid() {
        let readme = include_str!("../../README.md");
        let section = &readme[readme.find("## The Inventory").unwrap()..];
        let start = section.find("```toml\n").unwrap() + "```toml\n".len();
        let end = start + section[start..].find("```").unwrap();
        let inventory = parse(&section[start..end]).unwrap();
        assert_eq!(inventory.models()[0].id, "qwen3.8-27b");
        assert_eq!(inventory.backends().len(), 2);
    }

    #[test]
    fn the_unrendered_template_is_rejected() {
        let message = format!("{:#}", parse(SHIPPED).unwrap_err());
        assert!(message.contains("ai-ctl init"), "{message}");
    }

    #[test]
    fn rendering_escapes_the_directory() {
        let rendered = render(SHIPPED, Path::new(r#"/data/odd "dir"\x"#)).unwrap();
        let inventory = parse(&rendered).unwrap();
        assert_eq!(inventory.models_dir(), Path::new(r#"/data/odd "dir"\x"#));
    }

    #[test]
    fn rendering_escapes_the_directory_s_control_characters() {
        let dir = "/data/a\u{1b}b\u{2028}c";
        let rendered = render(SHIPPED, Path::new(dir)).unwrap();
        assert!(rendered.contains(r#"models_dir = "/data/a\u001Bb\u2028c""#));
        assert_eq!(parse(&rendered).unwrap().models_dir(), Path::new(dir));
    }

    #[test]
    fn rendering_needs_a_placeholder() {
        assert!(render("version = 1\n", Path::new("/data")).is_err());
    }

    const MINIMAL: &str = r#"
        version = 1
        models_dir = "/models"

        [[backends]]
        id = "native"
        kind = "llama-server"
        source = { build = "llama.cpp" }

        [[models]]
        id = "small"
        backend = "native"
        dir = "small"
        ctx_size = 4096
        vram_mib = 2048
        files = { model = "small.gguf" }
    "#;

    fn error(text: &str) -> String {
        format!("{:#}", parse(text).unwrap_err())
    }

    #[test]
    fn optional_sections_have_defaults() {
        let inventory = parse(MINIMAL).unwrap();
        assert_eq!(inventory.gateway(), Gateway::default());
        assert_eq!(inventory.defaults(BackendKind::LlamaServer), None);
        let model = &inventory.models()[0];
        assert_eq!(
            (model.repo.as_deref(), model.ttl, model.preload),
            (None, None, false)
        );
    }

    #[test]
    fn image_backends_parse() {
        let text = MINIMAL.replace(
            r#"source = { build = "llama.cpp" }"#,
            r#"source = { image = "ghcr.io/ggml-org/llama.cpp:server-cuda13" }"#,
        );
        let inventory = parse(&text).unwrap();
        assert_eq!(
            inventory.backends()[0].source,
            BackendSource::Image("ghcr.io/ggml-org/llama.cpp:server-cuda13".into())
        );
    }

    #[test]
    fn an_audiocpp_server_model_takes_its_own_fields() {
        let text = r#"
            version = 1
            models_dir = "/models"

            [[backends]]
            id = "audio-native"
            kind = "audiocpp_server"
            source = { build = "audio.cpp" }

            [[models]]
            id = "qwen3-tts-q8"
            backend = "audio-native"
            dir = "tts/qwen3-tts-1.7b-base"
            family = "qwen3_tts"
            task = "tts"
            mode = "offline"
            voices = "voices"
            vram_mib = 6200

            [models.session_options]
            "qwen3_tts.weight_type" = "q8_0"
            "qwen3_tts.seed" = 7
        "#;
        let inventory = parse(text).unwrap();
        assert_eq!(
            inventory.models()[0].audiocpp,
            AudiocppFields {
                family: Some("qwen3_tts".into()),
                task: Some("tts".into()),
                mode: Some("offline".into()),
                session_options: Flags::from([
                    ("qwen3_tts.seed".into(), FlagValue::Int(7)),
                    (
                        "qwen3_tts.weight_type".into(),
                        FlagValue::Text("q8_0".into())
                    ),
                ]),
                voices: Some("voices".into()),
            }
        );
        let nested = text.replace(
            r#""qwen3_tts.seed" = 7"#,
            r#""qwen3_tts.seed" = { value = 7 }"#,
        );
        let message = error(&nested);
        assert!(
            message
                .contains("model `qwen3-tts-q8` session_options: flag `qwen3_tts.seed` is a table"),
            "{message}"
        );
    }

    #[test]
    fn unknown_fields_are_rejected() {
        let message = error(&MINIMAL.replace("vram_mib", "vram_mb"));
        assert!(message.contains("vram_mb"), "{message}");
    }

    #[test]
    fn unknown_versions_are_rejected() {
        let later = "unsupported inventory version 2 (this ai-ctl reads version 1)";
        let version_2 = MINIMAL.replace("version = 1", "version = 2");
        assert_eq!(error(&version_2), later);
        assert_eq!(
            parse(&version_2)
                .unwrap_err()
                .downcast_ref::<UnsupportedVersion>(),
            Some(&UnsupportedVersion { found: 2, reads: 1 })
        );
        // A later version's new or renamed keys do not hide its version.
        assert_eq!(error(&version_2.replace("vram_mib", "vram_mb")), later);
        assert_eq!(
            error(&version_2.replace("models_dir", "models_root")),
            later
        );
    }

    #[test]
    fn unknown_backend_kinds_are_rejected() {
        let message = error(&MINIMAL.replace(r#"kind = "llama-server""#, r#"kind = "whisper""#));
        assert!(
            message.contains("unknown backend kind `whisper`"),
            "{message}"
        );
    }

    #[test]
    fn a_source_is_a_build_or_an_image() {
        let message = error(&MINIMAL.replace("{ build =", "{ binary ="));
        assert!(message.contains("binary"), "{message}");
    }

    #[test]
    fn the_gateway_section_takes_each_setting_alone() {
        let text = format!("{MINIMAL}\n[gateway]\nvram_margin_mib = 2048\n");
        let gateway = parse(&text).unwrap().gateway();
        assert_eq!(gateway.vram_margin_mib, 2048);
        assert_eq!(gateway.listen, Gateway::default().listen);
    }

    #[test]
    fn expose_lets_the_gateway_listen_beyond_loopback() {
        let text = format!("{MINIMAL}\n[gateway]\nexpose = true\n");
        assert!(parse(&text).unwrap().gateway().expose);

        let exposed = format!("{MINIMAL}\n[gateway]\nlisten = \"0.0.0.0:8080\"\n");
        let message = error(&exposed);
        assert!(
            message.contains("set `expose = true` under [gateway]"),
            "{message}"
        );
        let gateway = parse(&format!("{exposed}expose = true\n"))
            .unwrap()
            .gateway();
        assert_eq!(gateway.listen, "0.0.0.0:8080".parse().unwrap());
    }

    #[test]
    fn listen_must_be_an_address() {
        let text = format!("{MINIMAL}\n[gateway]\nlisten = \"localhost\"\n");
        assert!(error(&text).contains("gateway.listen `localhost`"));
    }

    #[test]
    fn flags_take_scalars_only() {
        let text = format!("{MINIMAL}\n[models.flags]\nlora = [\"a\", \"b\"]\n");
        let message = error(&text);
        assert!(message.contains("flag `lora` is a array"), "{message}");
    }

    #[test]
    fn validation_errors_are_reported() {
        let text = MINIMAL.replace("vram_mib = 2048", "vram_mib = 0");
        assert!(error(&text).contains("VRAM budget above zero"));
    }

    #[test]
    fn syntax_errors_are_reported() {
        assert!(parse("version = ").is_err());
    }

    #[test]
    fn install_writes_once_and_never_overwrites() {
        let root = tempfile::tempdir().unwrap();
        let models = root.path().join("models");
        fs::create_dir(&models).unwrap();
        let path = root.path().join("config/ai-ctl/inventory.toml");
        let store = InventoryFile::new(path.clone());

        assert_eq!(store.load().unwrap(), None);
        assert_eq!(store.install(&models).unwrap(), Installed::Written);
        let installed = store.load().unwrap().unwrap();
        assert_eq!(installed.models_dir(), models);

        fs::write(&path, "edited by hand").unwrap();
        assert_eq!(store.install(&models).unwrap(), Installed::AlreadyExists);
        assert_eq!(fs::read_to_string(&path).unwrap(), "edited by hand");

        let leftovers: Vec<_> = fs::read_dir(path.parent().unwrap())
            .unwrap()
            .map(|entry| entry.unwrap().file_name())
            .collect();
        assert_eq!(leftovers, ["inventory.toml"]);
    }

    #[test]
    fn install_tells_a_link_to_a_missing_file_and_leaves_it() {
        let root = tempfile::tempdir().unwrap();
        let models = root.path().join("models");
        fs::create_dir(&models).unwrap();
        let path = root.path().join("inventory.toml");
        let target = root.path().join("dotfiles/inventory.toml");
        std::os::unix::fs::symlink(&target, &path).unwrap();
        let store = InventoryFile::new(path.clone());
        assert_eq!(
            store.install(&models).unwrap(),
            Installed::DanglingLink {
                target: target.clone()
            }
        );
        assert_eq!(fs::read_link(&path).unwrap(), target);
    }

    #[test]
    fn install_needs_an_existing_models_directory() {
        let root = tempfile::tempdir().unwrap();
        let store = InventoryFile::new(root.path().join("inventory.toml"));
        let message = store
            .install(&root.path().join("absent"))
            .unwrap_err()
            .to_string();
        assert!(message.contains("does not exist"), "{message}");

        let file = root.path().join("file");
        fs::write(&file, "").unwrap();
        let message = store.install(&file).unwrap_err().to_string();
        assert!(message.contains("is not a directory"), "{message}");
        assert!(!root.path().join("inventory.toml").exists());
    }

    #[test]
    fn install_makes_a_relative_models_directory_absolute() {
        let root = tempfile::tempdir().unwrap();
        let store = InventoryFile::new(root.path().join("inventory.toml"));
        store.install(Path::new(".")).unwrap();
        let installed = store.load().unwrap().unwrap();
        assert!(installed.models_dir().is_absolute());
    }

    #[test]
    fn an_unreadable_inventory_names_its_path() {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("inventory.toml");
        fs::write(&path, "version = 1\n").unwrap();
        let message = format!("{:#}", InventoryFile::new(path).load().unwrap_err());
        assert!(message.starts_with("invalid inventory "), "{message}");
        assert!(message.contains("models_dir"), "{message}");
    }

    #[test]
    fn a_models_directory_that_is_not_utf8_is_refused() {
        use std::os::unix::ffi::OsStrExt;
        let dir = Path::new(std::ffi::OsStr::from_bytes(b"/data/mod\xe8les"));
        assert_eq!(
            render(SHIPPED, dir).unwrap_err().to_string(),
            "/data/mod\u{FFFD}les is not valid UTF-8"
        );
    }
}
