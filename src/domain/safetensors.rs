//! A safetensors index, such as `diffusion_pytorch_model.safetensors.index.json`:
//! a model's weights split into shards, each tensor mapped to the shard that
//! holds it, read as stable-diffusion.cpp reads it.

use std::collections::BTreeSet;
use std::path::{Component, Path, PathBuf};

use serde_json::Value;

use super::text::escape_controls;

/// Why a text is not an index [`shards`] can read. A tensor or a shard is
/// kept as the index names it, and shown with its control and invisible
/// characters escaped.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum IndexError {
    #[error("not JSON: {0}")]
    NotJson(String),
    #[error("no `weight_map` object")]
    NoWeightMap,
    #[error("the tensor `{}` has no shard path", escape_controls(.0))]
    NoShardPath(String),
    #[error("the shard `{}` holds a control or invisible character", escape_controls(.0))]
    HiddenCharacter(String),
    #[error("`weight_map` names no shard")]
    NoShard,
}

/// The shards the index `text` names, each once: the values of its
/// `weight_map` object, which maps each tensor to its shard's path, one
/// shard at least. A shard holding a control or invisible character is
/// refused, since `doctor` prints it.
pub fn shards(text: &str) -> Result<BTreeSet<String>, IndexError> {
    let index: Value =
        serde_json::from_str(text).map_err(|err| IndexError::NotJson(err.to_string()))?;
    let Some(weight_map) = index.get("weight_map").and_then(Value::as_object) else {
        return Err(IndexError::NoWeightMap);
    };
    let mut shards = BTreeSet::new();
    for (tensor, shard) in weight_map {
        let Some(shard) = shard.as_str().filter(|shard| !shard.is_empty()) else {
            return Err(IndexError::NoShardPath(tensor.clone()));
        };
        if escape_controls(shard) != shard {
            return Err(IndexError::HiddenCharacter(shard.to_owned()));
        }
        shards.insert(shard.to_owned());
    }
    if shards.is_empty() {
        return Err(IndexError::NoShard);
    }
    Ok(shards)
}

/// Where stable-diffusion.cpp reads `shard`, named by the index at the
/// absolute path `index`: an absolute `shard` as it is, a relative one from
/// the index's directory, made [`normal`].
pub fn shard_path(index: &Path, shard: &str) -> PathBuf {
    normal(&index.parent().unwrap_or(index).join(shard))
}

/// The absolute `path` with each `..` resolved without following links, as
/// stable-diffusion.cpp resolves a shard's path. An absolute path holds no
/// `.`, which [`Path::components`] drops.
pub fn normal(path: &Path) -> PathBuf {
    let mut normal = PathBuf::new();
    for component in path.components() {
        match component {
            // At the root, `..` stays there, as the kernel reads it.
            Component::ParentDir => {
                normal.pop();
            }
            component => normal.push(component),
        }
    }
    normal
}

#[cfg(test)]
mod tests {
    use super::*;

    const FIBO: &str = include_str!(
        "../../tests/fixtures/fibo-2.0-dev-72415bd0-diffusion_pytorch_model.safetensors.index.json"
    );

    fn refused(text: &str) -> String {
        shards(text).unwrap_err().to_string()
    }

    #[test]
    fn fibos_index_names_its_two_shards() {
        assert_eq!(
            shards(FIBO).unwrap(),
            BTreeSet::from([
                "diffusion_pytorch_model-00001-of-00002.safetensors".to_owned(),
                "diffusion_pytorch_model-00002-of-00002.safetensors".to_owned(),
            ])
        );
    }

    #[test]
    fn an_index_without_a_weight_map_of_shards_is_refused() {
        assert!(refused("{").starts_with("not JSON: "), "{}", refused("{"));
        for text in ["[]", "{}", r#"{"weight_map": []}"#] {
            assert_eq!(refused(text), "no `weight_map` object", "{text}");
        }
        assert_eq!(
            refused(r#"{"weight_map": {}}"#),
            "`weight_map` names no shard"
        );
        assert_eq!(
            refused(r#"{"weight_map": {"a.weight": "a.safetensors", "b.weight": 2}}"#),
            "the tensor `b.weight` has no shard path"
        );
        assert_eq!(
            refused(r#"{"weight_map": {"a.weight": ""}}"#),
            "the tensor `a.weight` has no shard path"
        );
        assert_eq!(
            refused(r#"{"weight_map": {"a\u001bweight": null}}"#),
            "the tensor `a\\u001Bweight` has no shard path"
        );
    }

    #[test]
    fn a_shard_that_would_drive_the_terminal_is_refused() {
        assert_eq!(
            refused(r#"{"weight_map": {"a\u001b[2Jweight": "a\u001b[2J.safetensors"}}"#),
            "the shard `a\\u001B[2J.safetensors` holds a control or invisible character"
        );
        assert_eq!(
            refused(r#"{"weight_map": {"a.weight": "a\u202e.safetensors"}}"#),
            "the shard `a\\u202E.safetensors` holds a control or invisible character"
        );
    }

    #[test]
    fn a_shard_is_read_from_the_indexs_directory_unless_absolute() {
        let index = Path::new("/models/fibo/transformer/model.safetensors.index.json");
        let path = |shard| shard_path(index, shard);
        assert_eq!(
            path("model-00001.safetensors"),
            Path::new("/models/fibo/transformer/model-00001.safetensors")
        );
        assert_eq!(
            path("./shards/../model-00001.safetensors"),
            Path::new("/models/fibo/transformer/model-00001.safetensors")
        );
        assert_eq!(path("../../../../../etc/passwd"), Path::new("/etc/passwd"));
        assert_eq!(
            path("/data/model.safetensors"),
            Path::new("/data/model.safetensors")
        );
    }
}
