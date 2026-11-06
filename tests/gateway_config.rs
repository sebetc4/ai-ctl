//! Snapshots of the gateway configuration generated for the shipped
//! inventory. llama-swap ignores a key it does not know, so `-validate`
//! cannot catch a misspelled one: these files are read once, and any change
//! to the generated text shows up here.
//!
//! After a deliberate change, rewrite them with
//! `UPDATE_SNAPSHOTS=1 cargo test --test gateway_config` and read the diff.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use ai_ctl::domain::gateway_config::{self, Runtime};
use ai_ctl::domain::inventory::{BackendSource, Inventory};
use ai_ctl::infrastructure::inventory_file::{SHIPPED, parse, render};

/// The models directory of this machine.
const MODELS_DIR: &str = "/ai/models";
/// Where the current builds sit, as the build store lays them out.
const BUILDS: &str = "/home/user/.local/share/ai-ctl/builds";
/// The usable memory NVML gives for the RTX 5090 Laptop: 24463 MiB less the
/// 388 the driver reserves.
const GPU_MIB: u64 = 24075;
/// The gateway's directory, under which go the files its models read.
const GATEWAY_DIR: &str = "/home/user/.local/state/ai-ctl/gateway";

fn inventory(text: &str) -> Inventory {
    parse(&render(text, Path::new(MODELS_DIR)).unwrap()).unwrap()
}

/// The program of every backend, its project's build under [`BUILDS`], an
/// image as named.
fn runtimes(inventory: &Inventory) -> BTreeMap<String, Runtime> {
    inventory
        .backends()
        .iter()
        .map(|backend| {
            let runtime = match &backend.source {
                BackendSource::Build(project) => {
                    let tag = match project.as_str() {
                        "llama.cpp" => "b11538",
                        "whisper.cpp" => "v1.9.5",
                        "stable-diffusion.cpp" => "master-1024-77b72a2",
                        "audio.cpp" => "v0.9.3",
                        other => panic!("no build of {other}"),
                    };
                    Runtime::Native(
                        PathBuf::from(BUILDS)
                            .join(project)
                            .join(tag)
                            .join(backend.kind.to_string()),
                    )
                }
                BackendSource::Image(image) => Runtime::Image(image.clone()),
            };
            (backend.id.clone(), runtime)
        })
        .collect()
}

fn yaml(inventory: &Inventory) -> String {
    gateway_config::generate(
        inventory,
        &runtimes(inventory),
        GPU_MIB,
        Path::new(GATEWAY_DIR),
    )
    .unwrap()
    .to_yaml()
}

/// Compares `actual` with the snapshot `name`, or rewrites the snapshot when
/// `UPDATE_SNAPSHOTS` is set.
fn assert_snapshot(name: &str, actual: &str) {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/snapshots")
        .join(name);
    if std::env::var_os("UPDATE_SNAPSHOTS").is_some() {
        fs::write(&path, actual).unwrap();
        return;
    }
    let expected = fs::read_to_string(&path)
        .unwrap_or_else(|err| panic!("cannot read {}: {err}", path.display()));
    if actual == expected {
        return;
    }
    // Split on every newline, so a missing final one shows as a line.
    let actual: Vec<&str> = actual.split('\n').collect();
    let expected: Vec<&str> = expected.split('\n').collect();
    let line = actual
        .iter()
        .zip(&expected)
        .position(|(a, e)| a != e)
        .unwrap_or_else(|| actual.len().min(expected.len()));
    let shown = |lines: &[&str]| {
        lines
            .get(line)
            .map_or_else(|| "<end of file>".to_owned(), |text| format!("{text:?}"))
    };
    panic!(
        "the generated configuration differs from {} from line {}:\n  \
         expected: {}\n  actual:   {}\n\
         rewrite it with `UPDATE_SNAPSHOTS=1 cargo test --test gateway_config` if the change is meant",
        path.display(),
        line + 1,
        shown(&expected),
        shown(&actual),
    );
}

#[test]
fn the_shipped_inventory_gives_the_snapshot() {
    assert_snapshot("gateway-shipped.yaml", &yaml(&inventory(SHIPPED)));
}

#[test]
fn the_shipped_inventory_with_a_model_on_an_image_gives_the_snapshot() {
    let native = "id = \"qwen3.6-35b-a3b\"\nbackend = \"llama-native\"\n";
    assert_eq!(SHIPPED.matches(native).count(), 1, "the 35B's backend line");
    let text = SHIPPED.replace(
        native,
        "id = \"qwen3.6-35b-a3b\"\nbackend = \"llama-cuda13\"\n",
    );
    assert_snapshot("gateway-shipped-image.yaml", &yaml(&inventory(&text)));
}

#[test]
fn a_line_separator_in_a_flag_cannot_add_a_model() {
    // The flag of finding 3, a TOML escape for U+2028: once written in the
    // `cmd` block, llama-swap would read a model `evil` after it.
    let table = "[defaults.llama-server]\n";
    assert_eq!(SHIPPED.matches(table).count(), 1, "the defaults' table");
    let text = SHIPPED.replace(
        table,
        &format!(
            "{table}zzz = \"x\\u2028  evil:\\u2028    cmd: \\\"/usr/bin/touch MARKER \\\\x24{{PORT}}\\\" #\"\n"
        ),
    );
    let inventory = inventory(&text);
    let message = gateway_config::generate(
        &inventory,
        &runtimes(&inventory),
        GPU_MIB,
        Path::new(GATEWAY_DIR),
    )
    .unwrap_err()
    .to_string();
    assert!(
        message.ends_with("contains a control character or a line separator"),
        "{message}"
    );
}
