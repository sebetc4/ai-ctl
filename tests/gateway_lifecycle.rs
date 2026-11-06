//! The gateway's lifecycle end to end: `up`, a chat request the gateway
//! proxies to a model, and `down`, with the installed llama-swap and this
//! binary as the model's llama-server.
//!
//! The binary runs without libtest's harness. Started by llama-swap with a
//! llama-server's arguments (`--alias`), it is a fake llama-server that
//! answers `/health` and `/v1/chat/completions`, with the API key it finds
//! in `LLAMA_API_KEY` as llama-server does; otherwise it runs the scenario. No GPU is involved: the fake GPU lists the fake llama-servers
//! alive, as NVML lists a real one, so `down` waits for them to exit.
//! Skipped when no llama-swap is installed, or when llama-swap's first model
//! port is taken.

use std::any::Any;
use std::cell::RefCell;
use std::env;
use std::fs;
use std::io::{self, Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};
use std::time::{Duration, Instant};

use ai_ctl::application::gateway_config::Configurator;
use ai_ctl::application::lifecycle::{Configuration, Down, Event, Lifecycle, Up};
use ai_ctl::application::ports::{
    BuildStore, Containers, GatewayProcess, GatewayStore, GpuProbe, RunningModel, Staged,
    Unreadable,
};
use ai_ctl::domain::build::{Build, BuildId, CommitId};
use ai_ctl::domain::machine::{Gpu, GpuProcess, HostCompiler, Probed, Version};
use ai_ctl::domain::recipe::LLAMA_CPP;
use ai_ctl::domain::recipe::Patch;
use ai_ctl::domain::timestamp::Timestamp;
use ai_ctl::infrastructure::api_key::KeyFile;
use ai_ctl::infrastructure::gateway_api::LlamaSwapApi;
use ai_ctl::infrastructure::gateway_process::{LocalGateway, Procfs, TcpPorts};
use ai_ctl::infrastructure::gateway_store::FsGatewayStore;
use ai_ctl::infrastructure::http;
use ai_ctl::infrastructure::inventory_file::InventoryFile;
use ai_ctl::infrastructure::paths::Paths;
use anyhow::{Context, Result, bail, ensure};

const NAME: &str = "up_a_proxied_request_and_down";
/// The port llama-swap gives its first model.
const FIRST_MODEL_PORT: u16 = 5800;
/// What the fake GPU says each fake llama-server holds.
const HELD_MIB: u64 = 1024;

fn main() -> ExitCode {
    let args: Vec<String> = env::args().skip(1).collect();
    if args.iter().any(|arg| arg == "--alias") {
        return match fake_llama_server(&args) {
            Ok(()) => ExitCode::SUCCESS,
            Err(err) => {
                eprintln!("fake llama-server: {err:#}");
                ExitCode::FAILURE
            }
        };
    }
    if args.iter().any(|arg| arg == "--list") {
        println!("{NAME}: test");
        return ExitCode::SUCCESS;
    }
    // `cargo test <filter>` passes the filter to every test binary.
    let filters: Vec<&String> = args.iter().filter(|arg| !arg.starts_with('-')).collect();
    if !filters.is_empty() && !filters.iter().any(|filter| NAME.contains(filter.as_str())) {
        return ExitCode::SUCCESS;
    }
    match scenario() {
        Ok(None) => println!("test {NAME} ... ok"),
        Ok(Some(reason)) => println!("test {NAME} ... skipped: {reason}"),
        Err(err) => {
            println!("test {NAME} ... FAILED");
            eprintln!("{err:#}");
            return ExitCode::FAILURE;
        }
    }
    ExitCode::SUCCESS
}

/// The value following `flag` in `args`.
fn value<'a>(args: &'a [String], flag: &str) -> Option<&'a str> {
    let at = args.iter().position(|arg| arg == flag)?;
    args.get(at + 1).map(String::as_str)
}

/// Serves `/health` and `/v1/chat/completions` on llama-server's `--port`,
/// one connection at a time, until llama-swap stops it with SIGTERM.
fn fake_llama_server(args: &[String]) -> Result<()> {
    let alias = value(args, "--alias").context("no --alias")?;
    let port: u16 = value(args, "--port")
        .context("no --port")?
        .parse()
        .context("--port is not a port")?;
    let listener = TcpListener::bind(("127.0.0.1", port))
        .with_context(|| format!("cannot listen on 127.0.0.1:{port}"))?;
    println!("fake llama-server {alias} listening on 127.0.0.1:{port}");
    for stream in listener.incoming() {
        if let Err(err) = stream.and_then(|stream| answer(stream, alias)) {
            println!("fake llama-server {alias}: {err}");
        }
    }
    Ok(())
}

fn answer(mut stream: TcpStream, alias: &str) -> io::Result<()> {
    let mut received = Vec::new();
    let mut buffer = [0; 4096];
    let end = loop {
        if let Some(end) = received.windows(4).position(|w| w == b"\r\n\r\n") {
            break end;
        }
        let read = stream.read(&mut buffer)?;
        if read == 0 {
            return Ok(());
        }
        received.extend_from_slice(&buffer[..read]);
    };
    let head = String::from_utf8_lossy(&received[..end]).into_owned();
    let length = head
        .lines()
        .filter_map(|line| line.split_once(':'))
        .find(|(name, _)| name.eq_ignore_ascii_case("content-length"))
        .and_then(|(_, value)| value.trim().parse::<usize>().ok())
        .unwrap_or(0);
    while received.len() < end + 4 + length {
        let read = stream.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        received.extend_from_slice(&buffer[..read]);
    }
    let mut words = head.split(' ');
    let (method, path) = (words.next().unwrap_or(""), words.next().unwrap_or(""));
    // llama-server's check of `--api-key`: `Authorization`, else `X-Api-Key`.
    let header = |name: &str| {
        head.lines()
            .filter_map(|line| line.split_once(':'))
            .find(|(found, _)| found.eq_ignore_ascii_case(name))
            .map(|(_, value)| value.trim())
    };
    let presented = header("authorization")
        .map(|value| value.strip_prefix("Bearer ").unwrap_or(value))
        .or_else(|| header("x-api-key"));
    let refused = path != "/health"
        && env::var("LLAMA_API_KEY").is_ok_and(|key| presented != Some(key.as_str()));
    let (status, body) = match (method, path) {
        _ if refused => (
            "401 Unauthorized",
            r#"{"error":{"message":"Invalid API Key"}}"#.to_owned(),
        ),
        ("GET", "/health") => ("200 OK", r#"{"status":"ok"}"#.to_owned()),
        ("GET", "/v1/models") => ("200 OK", r#"{"object":"list","data":[]}"#.to_owned()),
        ("POST", "/v1/chat/completions") => (
            "200 OK",
            format!(
                r#"{{"object":"chat.completion","model":"{alias}","choices":[{{"index":0,"finish_reason":"stop","message":{{"role":"assistant","content":"pong"}}}}]}}"#
            ),
        ),
        _ => ("404 Not Found", r#"{"error":"not found"}"#.to_owned()),
    };
    println!("fake llama-server {alias}: {method} {path} {status}");
    write!(
        stream,
        "HTTP/1.1 {status}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    )
}

/// `Some(reason)` when the scenario cannot run here.
fn scenario() -> Result<Option<String>> {
    let Ok(paths) = Paths::from_env() else {
        return Ok(Some("HOME is not set".into()));
    };
    let releases = FsGatewayStore::new(paths.gateway_dir());
    if releases.current()?.is_none() {
        return Ok(Some(
            "llama-swap is not installed; run `ai-ctl gateway install`".into(),
        ));
    }
    if TcpListener::bind(("127.0.0.1", FIRST_MODEL_PORT)).is_err() {
        return Ok(Some(format!(
            "port {FIRST_MODEL_PORT}, llama-swap's first model port, is taken"
        )));
    }

    let root = tempfile::tempdir().context("cannot create a scratch directory")?;
    let listen = free_port()?;
    let inventory = root.path().join("inventory.toml");
    fs::write(
        &inventory,
        inventory_toml(&root.path().join("models"), listen),
    )?;
    let inventory = InventoryFile::new(inventory);
    let builds = ThisBinary(env::current_exe()?);
    let gpu = FakeGpu(env::current_exe()?);
    let key = KeyFile::new(root.path().join("api-key"));
    let gateway = LocalGateway::new(
        root.path().join("gateway"),
        root.path().join("logs"),
        key.clone(),
    );
    let api = LlamaSwapApi::new(key.clone());
    let lifecycle = Lifecycle {
        configurator: Configurator {
            inventory: &inventory,
            builds: &builds,
            gpu: &gpu,
            files_dir: gateway.files_dir(),
        },
        releases: &releases,
        gateway: &gateway,
        processes: &Procfs,
        ports: &TcpPorts,
        api: &api,
        containers: &NoContainers,
        pause: std::thread::sleep,
        now: Instant::now,
    };
    let events = RefCell::new(Vec::new());
    let mut record = |event: Event| events.borrow_mut().push(event);

    let Up::Started { pid, .. } = lifecycle.up(&mut record)? else {
        bail!("`up` found a gateway running in a fresh state directory");
    };
    // Stops the gateway should the scenario fail before `down`.
    let mut guard = Guard(Some(pid));
    ensure!(
        matches!(events.borrow().as_slice(), [Event::Starting { listen: at, .. }] if *at == listen),
        "`up` reported {:?}",
        events.borrow()
    );

    let status = lifecycle.status(&mut record)?;
    let gateway = status.gateway.context("`status` reports no gateway")?;
    ensure!(
        gateway.pid == pid
            && gateway.models == Some(vec![])
            && gateway.configuration == Configuration::Current,
        "`status` before any request reported {gateway:?}"
    );

    // Nothing but the health check answers without the key.
    for (method, path) in [
        ("GET", "/v1/models"),
        ("GET", "/running"),
        ("GET", "/logs"),
        ("POST", "/api/models/unload"),
    ] {
        let status = request(listen, method, path, None, None)?.status;
        ensure!(
            status == 401,
            "{method} {path} without the key answered {status}"
        );
    }
    let status = request(listen, "GET", "/health", None, None)?.status;
    ensure!(status == 200, "/health without the key answered {status}");
    let chat = r#"{"model":"fake","messages":[{"role":"user","content":"ping"}]}"#;
    let status = request(listen, "POST", "/v1/chat/completions", Some(chat), None)?.status;
    ensure!(
        status == 401,
        "a chat request without the key answered {status}"
    );

    let key = key.read()?.context("`up` created no API key")?;
    let response = request(
        listen,
        "POST",
        "/v1/chat/completions",
        Some(chat),
        Some(&key),
    )?;
    let body = String::from_utf8_lossy(&response.body);
    ensure!(
        response.status == 200 && body.contains(r#""content":"pong""#),
        "the proxied request answered {}: {body}",
        response.status
    );
    // The model's own server holds the gateway's key too.
    let backend = SocketAddr::from(([127, 0, 0, 1], FIRST_MODEL_PORT));
    let path = "/v1/chat/completions";
    let without = request(backend, "POST", path, Some(chat), None)?.status;
    let with = request(backend, "POST", path, Some(chat), Some(&key))?.status;
    ensure!(
        (without, with) == (401, 200),
        "the model's server answered {without} without the key and {with} with it"
    );

    let status = lifecycle.status(&mut record)?;
    let gateway = status.gateway.context("`status` reports no gateway")?;
    let ready = vec![RunningModel {
        id: "fake".into(),
        state: "ready".into(),
        proxy: Some(SocketAddr::from(([127, 0, 0, 1], FIRST_MODEL_PORT))),
    }];
    ensure!(
        gateway.models == Some(ready) && gateway.held.len() == 1 && status.leftovers.is_empty(),
        "`status` with the model loaded reported {gateway:?}"
    );
    let model = gateway.held[0].pid;

    let mut log = Vec::new();
    lifecycle.log(false, &mut log)?;
    let log = String::from_utf8_lossy(&log);
    ensure!(
        log.contains("fake llama-server fake: POST /v1/chat/completions 200 OK"),
        "the gateway's log lacks the model's output:\n{log}"
    );

    let down = lifecycle.down(&mut record)?;
    guard.0 = None;
    let released: Vec<u32> = match &down {
        Down::Stopped {
            pid: stopped,
            released,
        } if *stopped == pid => released.iter().map(|process| process.pid).collect(),
        other => bail!("`down` reported {other:?}"),
    };
    ensure!(
        released == [model] && !runs(model) && !runs(pid),
        "`down` released {released:?}, the model being PID {model}"
    );
    let status = lifecycle.status(&mut record)?;
    ensure!(
        status.gateway.is_none(),
        "`status` after `down` reported {:?}",
        status.gateway
    );
    let warnings: Vec<String> = events
        .borrow()
        .iter()
        .filter_map(|event| match event {
            Event::Warning(warning) => Some(warning.clone()),
            _ => None,
        })
        .collect();
    ensure!(warnings.is_empty(), "warnings: {warnings:?}");
    Ok(None)
}

/// A port of the loopback no server listens on now.
fn free_port() -> Result<SocketAddr> {
    let listener = TcpListener::bind("127.0.0.1:0").context("cannot find a free port")?;
    Ok(listener.local_addr()?)
}

fn inventory_toml(models_dir: &Path, listen: SocketAddr) -> String {
    format!(
        r#"version = 1
models_dir = "{}"

[gateway]
listen = "{listen}"

[[backends]]
id = "llama-native"
kind = "llama-server"
source = {{ build = "llama.cpp" }}

[[models]]
id = "fake"
backend = "llama-native"
dir = "llm/fake"
ctx_size = 4096
vram_mib = {HELD_MIB}
ttl = 60

[models.files]
model = "fake.gguf"
"#,
        models_dir.display()
    )
}

/// Sends `method path` to `addr`, with `json` as its body and `key` as a
/// bearer token.
fn request(
    addr: SocketAddr,
    method: &str,
    path: &str,
    json: Option<&str>,
    key: Option<&str>,
) -> Result<http::Response> {
    let timeout = Duration::from_secs(60);
    http::request(addr, method, path, json.map(str::as_bytes), timeout, key)?
        .with_context(|| format!("nothing listens on {addr}"))
}

/// Whether `pid` runs a program: a process gone, or a zombie, does not.
fn runs(pid: u32) -> bool {
    fs::read_link(format!("/proc/{pid}/exe")).is_ok()
}

/// Sends SIGTERM to a gateway left running by a failed scenario, and waits
/// for it to stop its model and exit.
struct Guard(Option<u32>);

impl Drop for Guard {
    fn drop(&mut self) {
        let Some(pid) = self.0 else { return };
        let _ = Command::new("kill")
            .args(["-s", "TERM", &pid.to_string()])
            .status();
        let deadline = Instant::now() + Duration::from_secs(40);
        while runs(pid) && Instant::now() < deadline {
            std::thread::sleep(Duration::from_millis(100));
        }
    }
}

/// The current llama.cpp build, whose `llama-server` is this binary.
struct ThisBinary(PathBuf);

fn build_b1() -> Build {
    Build {
        project: "llama.cpp".into(),
        id: BuildId::parse("b1").unwrap(),
        commit: CommitId::parse("79e2e74eb11022c1ba2e438df7f0ca2d4c10f8b6").unwrap(),
        toolkit: Version::new(13, 4, 92),
        host_compiler: HostCompiler {
            command: "g++".into(),
            path: "/usr/bin/g++".into(),
            version: Version::new(16, 2, 1),
        },
        cmake_flags: LLAMA_CPP
            .cmake_flags
            .iter()
            .map(|(name, value)| ((*name).to_owned(), (*value).to_owned()))
            .collect(),
        patches: vec![],
        built_at: Timestamp::parse("2026-10-11T00:00:00Z").unwrap(),
    }
}

impl BuildStore for ThisBinary {
    fn lock(&self) -> Result<Box<dyn Any>> {
        unreachable!("read-only")
    }
    fn builds(&self, _: &str) -> Result<(Vec<Build>, Vec<Unreadable>)> {
        Ok((vec![build_b1()], vec![]))
    }
    fn current(&self, _: &str) -> Result<Option<BuildId>> {
        Ok(Some(build_b1().id))
    }
    fn binary(&self, _: &str, _: &BuildId, _: &str) -> PathBuf {
        self.0.clone()
    }
    fn stage(&self, _: &Build, _: &Path, _: &Path, _: &[Patch]) -> Result<Staged> {
        unreachable!("read-only")
    }
    fn publish(&self, _: &Staged, _: bool) -> Result<()> {
        unreachable!("read-only")
    }
    fn discard(&self, _: &Staged) -> Result<()> {
        unreachable!("read-only")
    }
    fn set_current(&self, _: &str, _: &BuildId) -> Result<()> {
        unreachable!("read-only")
    }
    fn remove(&self, _: &str, _: &BuildId) -> Result<()> {
        unreachable!("read-only")
    }
    fn contains(&self, _: &Path) -> bool {
        // Another run's fake llama-servers are no leftover of this one.
        false
    }
}

/// A GPU on which every fake llama-server alive holds [`HELD_MIB`].
struct FakeGpu(PathBuf);

impl FakeGpu {
    fn fake_llama_servers(&self) -> Vec<u32> {
        let Ok(entries) = fs::read_dir("/proc") else {
            return Vec::new();
        };
        entries
            .flatten()
            .filter_map(|entry| entry.file_name().to_str()?.parse::<u32>().ok())
            .filter(|pid| {
                fs::read_link(format!("/proc/{pid}/exe")).is_ok_and(|exe| exe == self.0)
                    && fs::read(format!("/proc/{pid}/cmdline")).is_ok_and(|cmdline| {
                        cmdline.split(|&b| b == 0).any(|arg| arg == b"--alias")
                    })
            })
            .collect()
    }
}

impl GpuProbe for FakeGpu {
    fn gpu(&self) -> Probed<Gpu> {
        let processes: Vec<GpuProcess> = self
            .fake_llama_servers()
            .into_iter()
            .map(|pid| GpuProcess {
                pid,
                name: "llama-server".into(),
                memory: Some(HELD_MIB << 20),
            })
            .collect();
        Probed::Found(Gpu {
            name: "Fake GPU".into(),
            compute_capability: (12, 0),
            memory_total: 24463 << 20,
            memory_reserved: 388 << 20,
            memory_used: (16 + HELD_MIB * processes.len() as u64) << 20,
            driver_version: "615.71.09".into(),
            cuda_version: Version::new(13, 4, 0),
            processes,
        })
    }
}

/// No container: the model runs natively.
struct NoContainers;

impl Containers for NoContainers {
    fn image_exists(&self, _: &str) -> Result<bool> {
        unreachable!("no model runs on an image")
    }
    fn pull(&self, _: &str) -> Result<()> {
        unreachable!("no model runs on an image")
    }
    fn pids(&self, _: &str) -> Result<Vec<u32>> {
        Ok(Vec::new())
    }
    fn digest(&self, _: &str) -> Result<String> {
        unreachable!("no model runs on an image")
    }
}
