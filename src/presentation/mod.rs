//! Presentation layer: command-line parsing, dispatch and output.
//!
//! Depends on `application` and on the `domain` types it returns, never on
//! `infrastructure`: the adapters arrive already wired by `main.rs`.

pub mod bench;
pub mod build;
pub mod cli;
pub mod compare;
pub mod completions;
pub mod doctor;
pub mod gateway;
pub mod lifecycle;

use std::fmt::{self, Write as _};
use std::io::{self, IsTerminal, Write};
use std::path::Path;
use std::process::ExitCode;
use std::time::{Duration, Instant};

use anyhow::{Context, Result};

use crate::application::doctor::Doctor;
use crate::application::gateway_config::Configurator;
use crate::application::init;
use crate::application::ports::{
    BuildStore, CdiProbe, ChatClient, Compiler, Containers, Downloads, GatewayApi, GatewayProcess,
    GatewayStore, GpuMonitor, GpuProbe, InventoryStore, ModelServer, PortCheck, PowerProbe,
    ProcessTable, Programs, RunStore, Sources, ToolchainProbe,
};
use crate::domain::timestamp::Timestamp;
use cli::{Cli, Command};

/// The adapters the commands run against, built by the composition root.
pub struct Services {
    pub gpu: Box<dyn GpuProbe>,
    pub toolchain: Box<dyn ToolchainProbe>,
    pub cdi: Box<dyn CdiProbe>,
    pub power: Box<dyn PowerProbe>,
    pub inventory: Box<dyn InventoryStore>,
    pub sources: Box<dyn Sources>,
    pub compiler: Box<dyn Compiler>,
    pub builds: Box<dyn BuildStore>,
    pub programs: Box<dyn Programs>,
    pub downloads: Box<dyn Downloads>,
    /// The installed llama-swap releases.
    pub releases: Box<dyn GatewayStore>,
    pub gateway: Box<dyn GatewayProcess>,
    pub processes: Box<dyn ProcessTable>,
    pub ports: Box<dyn PortCheck>,
    pub api: Box<dyn GatewayApi>,
    pub containers: Box<dyn Containers>,
    pub server: Box<dyn ModelServer>,
    pub chat: Box<dyn ChatClient>,
    pub monitor: Box<dyn GpuMonitor>,
    pub runs: Box<dyn RunStore>,
    pub clock: fn() -> Timestamp,
    pub pause: fn(Duration),
    /// The monotonic clock the waits are bounded by.
    pub now: fn() -> Instant,
}

impl Services {
    /// The configuration use case, which `up`, `down`, `status`, `logs`,
    /// `bench run` and `gateway config` share.
    pub fn configurator(&self) -> Configurator<'_> {
        Configurator {
            inventory: self.inventory.as_ref(),
            builds: self.builds.as_ref(),
            gpu: self.gpu.as_ref(),
            files_dir: self.gateway.files_dir(),
        }
    }
}

/// Runs a parsed command line. `services` is called only by the commands
/// that need the adapters, so `completions` works on any machine.
pub fn run(cli: Cli, services: impl FnOnce() -> Result<Services>) -> Result<ExitCode> {
    match cli.command {
        Command::Init { models_dir } => {
            let services = services()?;
            init::init(services.inventory.as_ref(), &models_dir)?;
            print(&init_message(services.inventory.location(), &models_dir))?;
            Ok(ExitCode::SUCCESS)
        }
        Command::Doctor { json } => {
            let services = services()?;
            let report = Doctor {
                gpu: services.gpu.as_ref(),
                toolchain: services.toolchain.as_ref(),
                cdi: services.cdi.as_ref(),
                power: services.power.as_ref(),
                inventory: services.inventory.as_ref(),
                builds: services.builds.as_ref(),
                programs: services.programs.as_ref(),
                releases: services.releases.as_ref(),
                gateway: services.gateway.as_ref(),
                processes: services.processes.as_ref(),
                ports: services.ports.as_ref(),
                api: services.api.as_ref(),
                containers: services.containers.as_ref(),
            }
            .run();
            let output = if json {
                json_text(&doctor::render_json(&report))?
            } else {
                doctor::render_text(&report, use_color())
            };
            print(&output)?;
            Ok(if report.has_failures() {
                ExitCode::FAILURE
            } else {
                ExitCode::SUCCESS
            })
        }
        Command::Build(command) => build::run(command, &services()?),
        Command::Gateway { action } => gateway::run(action, &services()?),
        Command::Up => lifecycle::up(&services()?),
        Command::Down => lifecycle::down(&services()?),
        Command::Status { json } => lifecycle::status(&services()?, json),
        Command::Logs { follow, model } => lifecycle::logs(&services()?, follow, model.as_deref()),
        Command::Bench { action } => bench::run(action, &services()?),
        Command::Completions { shell } => {
            let mut script = Vec::new();
            completions::write(shell, &mut script);
            print(&String::from_utf8_lossy(&script))?;
            Ok(ExitCode::SUCCESS)
        }
    }
}

fn init_message(inventory: &Path, models_dir: &Path) -> String {
    let models_dir = std::path::absolute(models_dir).unwrap_or_else(|_| models_dir.to_owned());
    format!(
        "Installed {}\nModels directory: {}\nNext: download the models it lists, then run `ai-ctl doctor`.\n",
        inventory.display(),
        models_dir.display()
    )
}

/// Colors only on a terminal, and never when `NO_COLOR` is set.
fn use_color() -> bool {
    io::stdout().is_terminal() && std::env::var_os("NO_COLOR").is_none_or(|value| value.is_empty())
}

/// `value` as a `--json` option prints it: indented, ending with a newline.
fn json_text(value: &serde_json::Value) -> Result<String> {
    let mut text = serde_json::to_string_pretty(value).context("cannot serialize the document")?;
    text.push('\n');
    Ok(text)
}

/// Writes `text` to stdout. A closed pipe (`ai-ctl doctor | head`) is not an
/// error: the reader has what it wanted.
fn print(text: &str) -> Result<()> {
    print_to(&mut io::stdout().lock(), text)
}

/// [`print`] to `out`.
fn print_to(out: &mut dyn Write, text: &str) -> Result<()> {
    match out.write_all(text.as_bytes()).and_then(|()| out.flush()) {
        Err(err) if err.kind() != io::ErrorKind::BrokenPipe => {
            Err(err).context("cannot write to standard output")
        }
        _ => Ok(()),
    }
}

/// The line that names the current `what` after a change, `Current build:
/// <binary> (was <previous>)`.
fn current_line(out: &mut String, what: &str, binary: &Path, previous: Option<impl fmt::Display>) {
    let _ = write!(out, "Current {what}: {}", binary.display());
    match previous {
        Some(previous) => {
            let _ = writeln!(out, " (was {previous})");
        }
        None => out.push('\n'),
    }
}

/// `rows` in columns two spaces apart, each as wide as its widest cell, each
/// line after `indent` and with no space at its end.
fn table(rows: &[Vec<String>], indent: &str) -> String {
    let columns = rows.iter().map(Vec::len).max().unwrap_or(0);
    let widths: Vec<usize> = (0..columns)
        .map(|column| {
            rows.iter()
                .filter_map(|row| row.get(column))
                .map(|cell| cell.chars().count())
                .max()
                .unwrap_or(0)
        })
        .collect();
    let mut out = String::new();
    for row in rows {
        let mut line = String::from(indent);
        for (column, (cell, width)) in row.iter().zip(&widths).enumerate() {
            if column > 0 {
                line.push_str("  ");
            }
            let _ = write!(line, "{cell:<width$}");
        }
        out.push_str(line.trim_end());
        out.push('\n');
    }
    out
}

/// Whether `err` comes from writing to a pipe whose reader has gone.
fn closed_pipe(err: &anyhow::Error) -> bool {
    err.chain().any(|cause| {
        cause
            .downcast_ref::<io::Error>()
            .is_some_and(|err| err.kind() == io::ErrorKind::BrokenPipe)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_json_document_is_indented_and_ends_with_a_newline() {
        assert_eq!(
            json_text(&serde_json::json!({ "running": false })).unwrap(),
            "{\n  \"running\": false\n}\n"
        );
    }

    #[test]
    fn a_closed_pipe_is_told_from_other_errors_through_their_context() {
        let closed = anyhow::Error::from(io::Error::from(io::ErrorKind::BrokenPipe))
            .context("cannot write the log");
        assert!(closed_pipe(&closed));
        let refused = anyhow::Error::from(io::Error::from(io::ErrorKind::ConnectionRefused))
            .context("GET http://127.0.0.1:8080/logs/stream/x");
        assert!(!closed_pipe(&refused));
        assert!(!closed_pipe(&anyhow::anyhow!("no gateway runs")));
    }
}
