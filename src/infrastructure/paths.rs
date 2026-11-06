//! Where ai-ctl keeps its files, after the XDG base directory specification.

use std::ffi::OsString;
use std::path::PathBuf;

use anyhow::{Context, Result};

const APP: &str = "ai-ctl";

/// ai-ctl's directories: configuration (the inventory), data (builds) and
/// state (the gateway's PID and logs).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Paths {
    pub config_dir: PathBuf,
    pub data_dir: PathBuf,
    pub state_dir: PathBuf,
}

impl Paths {
    /// Resolves the directories from the process environment.
    pub fn from_env() -> Result<Self> {
        Self::resolve(|name| std::env::var_os(name))
    }

    /// Resolves the directories from `var`. An `XDG_*` variable counts only
    /// when it holds an absolute path, as the specification requires;
    /// otherwise the default under `HOME` applies.
    pub fn resolve(var: impl Fn(&str) -> Option<OsString>) -> Result<Self> {
        let absolute = |name: &str| {
            var(name)
                .map(PathBuf::from)
                .filter(|path| path.is_absolute())
        };
        let base = |xdg: &str, default: &str| -> Result<PathBuf> {
            match absolute(xdg) {
                Some(path) => Ok(path),
                None => absolute("HOME")
                    .map(|home| home.join(default))
                    .with_context(|| format!("neither {xdg} nor HOME is set to an absolute path")),
            }
        };
        Ok(Self {
            config_dir: base("XDG_CONFIG_HOME", ".config")?.join(APP),
            data_dir: base("XDG_DATA_HOME", ".local/share")?.join(APP),
            state_dir: base("XDG_STATE_HOME", ".local/state")?.join(APP),
        })
    }

    pub fn inventory_file(&self) -> PathBuf {
        self.config_dir.join("inventory.toml")
    }

    /// The projects' git clones.
    pub fn sources_dir(&self) -> PathBuf {
        self.data_dir.join("sources")
    }

    /// The CMake build trees, removed after each successful build.
    pub fn work_dir(&self) -> PathBuf {
        self.data_dir.join("work")
    }

    /// The installed builds.
    pub fn builds_dir(&self) -> PathBuf {
        self.data_dir.join("builds")
    }

    /// The bench's saved runs.
    pub fn bench_dir(&self) -> PathBuf {
        self.data_dir.join("bench")
    }

    /// The installed releases of the gateway, llama-swap.
    pub fn gateway_dir(&self) -> PathBuf {
        self.data_dir.join("llama-swap")
    }

    /// The running gateway's record and configuration.
    pub fn gateway_state_dir(&self) -> PathBuf {
        self.state_dir.join("gateway")
    }

    /// The build logs and the gateway's.
    pub fn logs_dir(&self) -> PathBuf {
        self.state_dir.join("logs")
    }

    /// The API key every client of the gateway sends.
    pub fn api_key_file(&self) -> PathBuf {
        self.state_dir.join("api-key")
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;

    fn resolve(vars: &[(&str, &str)]) -> Result<Paths> {
        let vars: HashMap<String, OsString> = vars
            .iter()
            .map(|(name, value)| ((*name).to_owned(), OsString::from(value)))
            .collect();
        Paths::resolve(|name| vars.get(name).cloned())
    }

    #[test]
    fn defaults_live_under_home() {
        let paths = resolve(&[("HOME", "/home/user")]).unwrap();
        assert_eq!(paths.config_dir, PathBuf::from("/home/user/.config/ai-ctl"));
        assert_eq!(
            paths.data_dir,
            PathBuf::from("/home/user/.local/share/ai-ctl")
        );
        assert_eq!(
            paths.state_dir,
            PathBuf::from("/home/user/.local/state/ai-ctl")
        );
        assert_eq!(
            paths.inventory_file(),
            PathBuf::from("/home/user/.config/ai-ctl/inventory.toml")
        );
        assert_eq!(
            paths.builds_dir(),
            PathBuf::from("/home/user/.local/share/ai-ctl/builds")
        );
        assert_eq!(
            paths.sources_dir(),
            PathBuf::from("/home/user/.local/share/ai-ctl/sources")
        );
        assert_eq!(
            paths.work_dir(),
            PathBuf::from("/home/user/.local/share/ai-ctl/work")
        );
        assert_eq!(
            paths.gateway_dir(),
            PathBuf::from("/home/user/.local/share/ai-ctl/llama-swap")
        );
        assert_eq!(
            paths.gateway_state_dir(),
            PathBuf::from("/home/user/.local/state/ai-ctl/gateway")
        );
        assert_eq!(
            paths.logs_dir(),
            PathBuf::from("/home/user/.local/state/ai-ctl/logs")
        );
        assert_eq!(
            paths.api_key_file(),
            PathBuf::from("/home/user/.local/state/ai-ctl/api-key")
        );
    }

    #[test]
    fn xdg_variables_take_precedence() {
        let paths = resolve(&[
            ("HOME", "/home/user"),
            ("XDG_CONFIG_HOME", "/cfg"),
            ("XDG_DATA_HOME", "/data"),
            ("XDG_STATE_HOME", "/state"),
        ])
        .unwrap();
        assert_eq!(paths.config_dir, PathBuf::from("/cfg/ai-ctl"));
        assert_eq!(paths.data_dir, PathBuf::from("/data/ai-ctl"));
        assert_eq!(paths.state_dir, PathBuf::from("/state/ai-ctl"));
    }

    #[test]
    fn empty_or_relative_xdg_variables_are_ignored() {
        let paths = resolve(&[
            ("HOME", "/home/user"),
            ("XDG_CONFIG_HOME", ""),
            ("XDG_DATA_HOME", "relative/data"),
        ])
        .unwrap();
        assert_eq!(paths.config_dir, PathBuf::from("/home/user/.config/ai-ctl"));
        assert_eq!(
            paths.data_dir,
            PathBuf::from("/home/user/.local/share/ai-ctl")
        );
    }

    #[test]
    fn a_missing_or_relative_home_is_an_error() {
        assert!(resolve(&[]).is_err());
        let message = resolve(&[("HOME", "home/user")]).unwrap_err().to_string();
        assert!(message.contains("XDG_CONFIG_HOME"), "{message}");
    }

    #[test]
    fn xdg_variables_alone_are_enough() {
        let paths = resolve(&[
            ("XDG_CONFIG_HOME", "/cfg"),
            ("XDG_DATA_HOME", "/data"),
            ("XDG_STATE_HOME", "/state"),
        ])
        .unwrap();
        assert_eq!(paths.config_dir, PathBuf::from("/cfg/ai-ctl"));
    }
}
