//! Entry point and composition root.
//!
//! `main` is the only place that knows every layer: it builds the
//! infrastructure adapters into `Services`, which the presentation layer
//! asks for only when a command needs an adapter, and passes it the parsed
//! command line. Each command's handler assembles its use case from
//! `Services` and picks the exit code; `main` prints the error a command
//! returns.

use std::process::ExitCode;

use ai_ctl::infrastructure::api_key::KeyFile;
use ai_ctl::infrastructure::build_store::{self, FsBuildStore};
use ai_ctl::infrastructure::chat_api::OpenAiChat;
use ai_ctl::infrastructure::cmake::Cmake;
use ai_ctl::infrastructure::command::SystemPrograms;
use ai_ctl::infrastructure::download::Curl;
use ai_ctl::infrastructure::gateway_api::LlamaSwapApi;
use ai_ctl::infrastructure::gateway_process::{LocalGateway, Procfs, TcpPorts};
use ai_ctl::infrastructure::gateway_store::FsGatewayStore;
use ai_ctl::infrastructure::git::GitSources;
use ai_ctl::infrastructure::host::SystemHost;
use ai_ctl::infrastructure::inventory_file::InventoryFile;
use ai_ctl::infrastructure::nvml::{NvmlMonitor, NvmlProbe};
use ai_ctl::infrastructure::paths::Paths;
use ai_ctl::infrastructure::podman::Podman;
use ai_ctl::infrastructure::run_store::FsRunStore;
use ai_ctl::infrastructure::server_api::LlamaServerApi;
use ai_ctl::presentation::{self, Services, cli::Cli};
use anyhow::Result;
use clap::Parser;

fn main() -> ExitCode {
    let cli = Cli::parse();

    match presentation::run(cli, services) {
        Ok(code) => code,
        Err(err) => {
            eprintln!("error: {err:#}");
            ExitCode::FAILURE
        }
    }
}

fn services() -> Result<Services> {
    let paths = Paths::from_env()?;
    Ok(Services {
        gpu: Box::new(NvmlProbe),
        toolchain: Box::new(SystemHost::new()),
        cdi: Box::new(SystemHost::new()),
        power: Box::new(SystemHost::new()),
        inventory: Box::new(InventoryFile::new(paths.inventory_file())),
        sources: Box::new(GitSources::new(paths.sources_dir())),
        compiler: Box::new(Cmake::new(paths.work_dir(), paths.logs_dir())),
        builds: Box::new(FsBuildStore::new(paths.builds_dir())),
        programs: Box::new(SystemPrograms),
        downloads: Box::new(Curl::new()),
        releases: Box::new(FsGatewayStore::new(paths.gateway_dir())),
        gateway: Box::new(LocalGateway::new(
            paths.gateway_state_dir(),
            paths.logs_dir(),
            KeyFile::new(paths.api_key_file()),
        )),
        processes: Box::new(Procfs),
        ports: Box::new(TcpPorts),
        api: Box::new(LlamaSwapApi::new(KeyFile::new(paths.api_key_file()))),
        containers: Box::new(Podman::new("podman".into())),
        server: Box::new(LlamaServerApi::new(KeyFile::new(paths.api_key_file()))),
        chat: Box::new(OpenAiChat::new(KeyFile::new(paths.api_key_file()))),
        monitor: Box::new(NvmlMonitor),
        runs: Box::new(FsRunStore::new(paths.bench_dir())),
        clock: build_store::now,
        pause: std::thread::sleep,
        now: std::time::Instant::now,
    })
}
