//! Test doubles of the ports, shared by the use cases' tests, and a writer
//! whose reader has gone, which the adapters' tests share too.

use std::any::Any;
use std::cell::{Cell, RefCell};
use std::collections::{BTreeMap, HashSet, VecDeque};
use std::io::Write;
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use anyhow::{Result, anyhow, bail, ensure};

use crate::domain::bench::{
    ClientTimings, Drafts, GpuReading, Message, Run, Sample, Scenario, ServerTimings, Slowdown,
    SpecCounters, Usage,
};
use crate::domain::build::{Build, BuildId, CommitId};
use crate::domain::gateway::{Release, Sha256};
use crate::domain::gateway_config::{CONTAINER_PREFIX, ConfigFile};
use crate::domain::inventory::{
    Backend, BackendKind, BackendSource, Gateway, Inventory, InventoryParts, Model,
    UnsupportedVersion,
};
use crate::domain::machine::{
    CdiSpec, CudaRepository, CudaToolkit, Gpu, GpuProcess, HostCompiler, Probed, Tool, Version,
};

use crate::domain::recipe::{LLAMA_CPP, Patch, Recipe};
use crate::domain::timestamp::Timestamp;

use super::ports::{
    BuildStore, CdiProbe, ChatClient, ChatRequest, Containers, GatewayApi, GatewayProcess,
    GatewayRecord, GatewayStore, GpuMonitor, GpuProbe, Installed, InventoryStore, Liveness,
    ModelServer, Monitoring, PortCheck, PowerProbe, ProcessStat, ProcessTable, ProgramOutput,
    Programs, Readings, Reply, RunStore, RunningModel, Signal, Staged, StagedRelease,
    StartedGateway, ToolchainProbe, Unreadable,
};

thread_local! {
    static ORIGIN: Instant = Instant::now();
    static ELAPSED: Cell<Duration> = const { Cell::new(Duration::ZERO) };
}

/// The test's clock: it moves only when [`advance`] moves it, so a test
/// reads its differences, never its value.
pub fn now() -> Instant {
    ORIGIN.with(|origin| *origin) + ELAPSED.with(Cell::get)
}

/// Moves the test's clock on by `by`: the fake `pause`, and the time a
/// fake port's call takes.
pub fn advance(by: Duration) {
    ELAPSED.with(|elapsed| elapsed.set(elapsed.get() + by));
}

pub struct FakeGpu(pub Probed<Gpu>);

impl GpuProbe for FakeGpu {
    fn gpu(&self) -> Probed<Gpu> {
        self.0.clone()
    }
}

pub struct FakeToolchain {
    pub toolkit: Probed<CudaToolkit>,
    pub compilers: Probed<Vec<HostCompiler>>,
    pub cmake: Probed<Version>,
    pub git: Probed<Version>,
    pub podman: Probed<Version>,
    pub nvidia_ctk: Probed<Version>,
    pub files: HashSet<PathBuf>,
    pub dirs: HashSet<PathBuf>,
    /// The text of each file `read_text` reads.
    pub texts: BTreeMap<PathBuf, String>,
    pub read_text_fails: Option<PathBuf>,
    pub entries_fails: bool,
    pub fedora: Probed<u32>,
    pub repositories: Probed<Vec<CudaRepository>>,
    pub toolkit_packages: Probed<Vec<String>>,
    /// The releases `toolkit_packages` was asked about.
    pub asked: RefCell<Vec<(u32, u32)>>,
}

impl FakeToolchain {
    /// This machine, but with a toolkit accepting GCC 15 at most: every
    /// check passes, and a build compiles with `g++-15`.
    pub fn healthy() -> Self {
        Self {
            toolkit: Probed::Found(toolkit(Version::new(13, 4, 92))),
            compilers: Probed::Found(vec![
                gxx("g++", Version::new(16, 2, 1)),
                gxx("g++-15", Version::new(15, 3, 1)),
            ]),
            cmake: Probed::Found(Version::new(4, 3, 0)),
            git: Probed::Found(Version::new(2, 56, 0)),
            podman: Probed::Found(Version::new(5, 8, 7)),
            nvidia_ctk: Probed::Found(Version::new(1, 20, 1)),
            files: ["/usr/bin/gcc", "/usr/bin/gcc-15"]
                .into_iter()
                .map(PathBuf::from)
                .collect(),
            dirs: HashSet::new(),
            texts: BTreeMap::new(),
            read_text_fails: None,
            entries_fails: false,
            fedora: Probed::Found(44),
            repositories: Probed::Found(vec![CudaRepository {
                id: "cuda-fedora44-x86_64".into(),
                driver_packages: vec![],
            }]),
            toolkit_packages: Probed::Found(vec!["cuda-toolkit-13-4".into()]),
            asked: RefCell::new(vec![]),
        }
    }

    /// This machine on 2026-10-09, before the upgrade: CUDA 13.2 from the
    /// `cuda-fedora43` repository, the only CUDA repository configured.
    pub fn before_the_upgrade() -> Self {
        Self {
            toolkit: Probed::Found(toolkit(Version::new(13, 2, 86))),
            repositories: Probed::Found(vec![CudaRepository {
                id: "cuda-fedora43-x86_64".into(),
                driver_packages: vec![],
            }]),
            toolkit_packages: Probed::Found(vec![
                "cuda-toolkit".into(),
                "cuda-toolkit-13-2".into(),
            ]),
            ..Self::healthy()
        }
    }
}

impl ToolchainProbe for FakeToolchain {
    fn cuda_toolkit(&self) -> Probed<CudaToolkit> {
        self.toolkit.clone()
    }
    fn host_compilers(&self) -> Probed<Vec<HostCompiler>> {
        self.compilers.clone()
    }
    fn tool_version(&self, tool: Tool) -> Probed<Version> {
        match tool {
            Tool::Cmake => self.cmake.clone(),
            Tool::Git => self.git.clone(),
            Tool::Podman => self.podman.clone(),
            Tool::NvidiaCtk => self.nvidia_ctk.clone(),
        }
    }
    fn is_file(&self, path: &Path) -> bool {
        self.files.contains(path)
    }
    fn is_dir(&self, path: &Path) -> bool {
        self.dirs.contains(path)
    }
    fn entries(&self, path: &Path) -> Result<Vec<String>> {
        ensure!(
            !self.entries_fails,
            "cannot read {}: Permission denied (os error 13)",
            path.display()
        );
        Ok(self
            .files
            .iter()
            .chain(&self.dirs)
            .filter(|entry| entry.parent() == Some(path))
            .filter_map(|entry| entry.file_name())
            .map(|name| name.to_string_lossy().into_owned())
            .collect())
    }
    fn read_text(&self, path: &Path) -> Result<String> {
        ensure!(
            self.read_text_fails.as_deref() != Some(path),
            "cannot read {}: Permission denied (os error 13)",
            path.display()
        );
        Ok(self
            .texts
            .get(path)
            .cloned()
            .unwrap_or_else(|| panic!("the test gives no text for {}", path.display())))
    }
    fn fedora_release(&self) -> Probed<u32> {
        self.fedora.clone()
    }
    fn cuda_repositories(&self) -> Probed<Vec<CudaRepository>> {
        self.repositories.clone()
    }
    fn toolkit_packages(&self, releases: &[(u32, u32)]) -> Probed<Vec<String>> {
        self.asked.borrow_mut().extend_from_slice(releases);
        self.toolkit_packages.clone()
    }
}

/// CDI as the toolkit sets it up; `listed` tells whether `nvidia-ctk cdi
/// list` was run.
pub struct FakeCdi {
    pub devices: Probed<Vec<String>>,
    pub listed: Cell<bool>,
    pub specs: Probed<Vec<CdiSpec>>,
}

impl FakeCdi {
    /// The devices and the specification of this machine's driver.
    pub fn healthy() -> Self {
        Self {
            devices: Probed::Found(vec!["nvidia.com/gpu=0".into(), "nvidia.com/gpu=all".into()]),
            listed: Cell::new(false),
            specs: Probed::Found(vec![CdiSpec {
                path: "/var/run/cdi/nvidia.yaml".into(),
                driver: Some("615.71.09".into()),
                missing: vec![],
            }]),
        }
    }
}

impl CdiProbe for FakeCdi {
    fn cdi_devices(&self) -> Probed<Vec<String>> {
        self.listed.set(true);
        self.devices.clone()
    }
    fn cdi_specs(&self) -> Probed<Vec<CdiSpec>> {
        self.specs.clone()
    }
}

pub struct FakePower {
    pub ac: Probed<bool>,
    pub profile: Probed<String>,
}

impl FakePower {
    /// On AC power, in the performance profile.
    pub fn healthy() -> Self {
        Self {
            ac: Probed::Found(true),
            profile: Probed::Found("performance".into()),
        }
    }
}

impl PowerProbe for FakePower {
    fn ac_power(&self) -> Probed<bool> {
        self.ac.clone()
    }
    fn power_profile(&self) -> Probed<String> {
        self.profile.clone()
    }
}

/// The RTX 5090 Laptop GPU with driver 615.71.09 (CUDA 13.4), idle.
pub fn gpu() -> Gpu {
    Gpu {
        name: "NVIDIA GeForce RTX 5090 Laptop GPU".into(),
        compute_capability: (12, 0),
        memory_total: 24463 << 20,
        memory_reserved: 388 << 20,
        memory_used: 16 << 20,
        driver_version: "615.71.09".into(),
        cuda_version: Version::new(13, 4, 0),
        processes: vec![],
    }
}

/// A toolkit under `/usr/local/cuda-<major>.<minor>` that accepts GCC 15.
pub fn toolkit(version: Version) -> CudaToolkit {
    CudaToolkit {
        version,
        nvcc: format!(
            "/usr/local/cuda-{}.{}/bin/nvcc",
            version.major, version.minor
        )
        .into(),
        max_gcc_major: Probed::Found(15),
    }
}

pub fn gxx(command: &str, version: Version) -> HostCompiler {
    HostCompiler {
        command: command.into(),
        path: Path::new("/usr/bin").join(command),
        version,
    }
}

pub fn printed(success: bool, stdout: &str, stderr: &str) -> ProgramOutput {
    ProgramOutput {
        success,
        stdout: stdout.into(),
        stderr: stderr.into(),
    }
}

/// A llama-server build answering `--version` and `--list-devices` as
/// b11538 does on this machine, and `--help` as whisper-server v1.9.5 does;
/// with `run_fails`, it cannot be run.
pub struct FakePrograms {
    pub version: ProgramOutput,
    pub devices: ProgramOutput,
    pub help: ProgramOutput,
    pub run_fails: bool,
    /// The program of each run, in turn.
    pub ran: RefCell<Vec<PathBuf>>,
}

impl Default for FakePrograms {
    fn default() -> Self {
        Self {
            version: printed(
                true,
                "",
                include_str!("../../tests/fixtures/llama-server-version.txt"),
            ),
            devices: printed(
                true,
                include_str!("../../tests/fixtures/llama-server-list-devices.txt"),
                "",
            ),
            help: printed(
                true,
                "",
                include_str!("../../tests/fixtures/whisper-server-v1.9.5-help.txt"),
            ),
            run_fails: false,
            ran: RefCell::new(vec![]),
        }
    }
}

impl Programs for FakePrograms {
    fn run(&self, program: &Path, args: &[&str]) -> Result<ProgramOutput> {
        self.ran.borrow_mut().push(program.to_owned());
        ensure!(
            !self.run_fails,
            "cannot run {}: Permission denied (os error 13)",
            program.display()
        );
        Ok(match args {
            ["--version"] => self.version.clone(),
            ["--list-devices"] => self.devices.clone(),
            ["--help"] => self.help.clone(),
            other => panic!("unexpected arguments {other:?}"),
        })
    }
}

/// The CMake options `recipe` sets, as a build of it records them.
pub fn recipe_flags(recipe: &Recipe) -> BTreeMap<String, String> {
    recipe
        .cmake_flags
        .iter()
        .map(|(name, value)| ((*name).to_owned(), (*value).to_owned()))
        .collect()
}

/// llama.cpp build b11538, as its recipe builds it now.
pub fn b11538() -> Build {
    Build {
        project: "llama.cpp".into(),
        id: BuildId::parse("b11538").unwrap(),
        commit: CommitId::parse("79e2e74eb11022c1ba2e438df7f0ca2d4c10f8b6").unwrap(),
        toolkit: Version::new(13, 4, 92),
        host_compiler: gxx("g++", Version::new(16, 2, 1)),
        cmake_flags: recipe_flags(&LLAMA_CPP),
        patches: vec![],
        built_at: Timestamp::parse("2026-10-09T20:48:47Z").unwrap(),
    }
}

/// Installed builds, each project's apart, read-only: `doctor` never
/// changes them. Each `…_fails` makes its method fail.
#[derive(Default)]
pub struct FakeBuilds {
    pub builds: Vec<Build>,
    pub unreadable: Vec<Unreadable>,
    /// llama.cpp's `current` link.
    pub current: Option<BuildId>,
    /// The other projects' `current` links.
    pub other_currents: BTreeMap<String, BuildId>,
    pub current_fails: bool,
    pub builds_fails: bool,
}

impl FakeBuilds {
    /// b11538, current.
    pub fn current_b11538() -> Self {
        Self {
            builds: vec![b11538()],
            current: Some(BuildId::parse("b11538").unwrap()),
            ..Self::default()
        }
    }
}

impl BuildStore for FakeBuilds {
    fn lock(&self) -> Result<Box<dyn Any>> {
        unreachable!("read-only")
    }
    fn builds(&self, project: &str) -> Result<(Vec<Build>, Vec<Unreadable>)> {
        ensure!(
            !self.builds_fails,
            "cannot read /data/builds/{project}: Permission denied (os error 13)"
        );
        let dir = Path::new("/data/builds").join(project);
        let builds = self.builds.iter().filter(|build| build.project == project);
        let unreadable = self
            .unreadable
            .iter()
            .filter(|entry| entry.dir.starts_with(&dir));
        Ok((builds.cloned().collect(), unreadable.cloned().collect()))
    }
    fn current(&self, project: &str) -> Result<Option<BuildId>> {
        ensure!(
            !self.current_fails,
            "cannot read /data/builds/{project}/current: Permission denied (os error 13)"
        );
        if project == "llama.cpp" {
            Ok(self.current.clone())
        } else {
            Ok(self.other_currents.get(project).cloned())
        }
    }
    fn binary(&self, project: &str, id: &BuildId, name: &str) -> PathBuf {
        Path::new("/data/builds")
            .join(project)
            .join(id.as_str())
            .join(name)
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
    fn contains(&self, program: &Path) -> bool {
        program.starts_with("/data/builds")
    }
}

/// The installed inventory, read-only, or the error reading it gives.
pub struct FakeInventory(pub Result<Option<Inventory>, LoadFailure>);

/// Why [`FakeInventory`] cannot load the inventory.
#[derive(Debug, Clone)]
pub enum LoadFailure {
    Message(String),
    /// The file is of another version, as the real store tells it.
    Version(UnsupportedVersion),
}

impl From<&str> for LoadFailure {
    fn from(message: &str) -> Self {
        Self::Message(message.to_owned())
    }
}

impl InventoryStore for FakeInventory {
    fn location(&self) -> &Path {
        Path::new("/home/user/.config/ai-ctl/inventory.toml")
    }
    fn load(&self) -> Result<Option<Inventory>> {
        self.0.clone().map_err(|failure| match failure {
            LoadFailure::Message(message) => anyhow!(message),
            LoadFailure::Version(version) => anyhow::Error::new(version)
                .context("invalid inventory /home/user/.config/ai-ctl/inventory.toml"),
        })
    }
    fn install(&self, _: &Path) -> Result<Installed> {
        unreachable!("read-only")
    }
}

/// The main model, served by the native build.
pub fn model() -> Model {
    Model {
        id: "qwen3.8-27b".into(),
        backend: "llama-native".into(),
        repo: Some("unsloth/Qwen3.8-27B-GGUF".into()),
        dir: "qwen3.8-27b".into(),
        files: BTreeMap::from([
            ("model".into(), "Qwen3.8-27B-UD-Q4_K_M.gguf".into()),
            ("mmproj".into(), "mmproj-F16.gguf".into()),
            ("model-draft".into(), "MTP/mtp-Qwen3.8-27B-Q4_0.gguf".into()),
        ]),
        flags: BTreeMap::new(),
        ctx_size: Some(98304),
        vram_mib: 22733,
        ttl: Some(1800),
        preload: true,
        audiocpp: Default::default(),
    }
}

/// An inventory of [`model`] alone, on the native build of llama.cpp.
pub fn inventory_parts(models_dir: &str) -> InventoryParts {
    InventoryParts {
        models_dir: models_dir.into(),
        gateway: Gateway::default(),
        defaults: BTreeMap::new(),
        backends: vec![Backend {
            id: "llama-native".into(),
            kind: BackendKind::LlamaServer,
            source: BackendSource::Build("llama.cpp".into()),
        }],
        models: vec![model()],
    }
}

pub fn inventory(models_dir: &str) -> Inventory {
    Inventory::new(inventory_parts(models_dir)).unwrap()
}

/// The installed llama-swap releases, read-only: the current one, if any,
/// whose binary is `missing` when its link names a removed release. Each
/// `…_fails` makes its method fail.
pub struct FakeReleases {
    pub current: Option<Release>,
    pub missing: bool,
    pub current_fails: bool,
    pub is_installed_fails: bool,
}

impl FakeReleases {
    pub fn current(release: &str) -> Self {
        Self {
            current: Some(Release::parse(release).unwrap()),
            ..Self::none()
        }
    }

    pub fn none() -> Self {
        Self {
            current: None,
            missing: false,
            current_fails: false,
            is_installed_fails: false,
        }
    }
}

impl GatewayStore for FakeReleases {
    fn lock(&self) -> Result<Box<dyn Any>> {
        unreachable!("read-only")
    }
    fn is_installed(&self, release: Release) -> Result<bool> {
        ensure!(
            !self.is_installed_fails,
            "cannot read /data/llama-swap/{release}: Permission denied (os error 13)"
        );
        Ok(self.current == Some(release) && !self.missing)
    }
    fn current(&self) -> Result<Option<Release>> {
        ensure!(
            !self.current_fails,
            "cannot read /data/llama-swap/current: Permission denied (os error 13)"
        );
        Ok(self.current)
    }
    fn binary(&self, release: Release) -> PathBuf {
        PathBuf::from(format!("/data/llama-swap/{release}/llama-swap"))
    }
    fn stage(&self, _: Release) -> Result<StagedRelease> {
        unreachable!("read-only")
    }
    fn sha256(&self, _: &Path) -> Result<Sha256> {
        unreachable!("read-only")
    }
    fn unpack(&self, _: &StagedRelease) -> Result<PathBuf> {
        unreachable!("read-only")
    }
    fn publish(&self, _: &StagedRelease) -> Result<()> {
        unreachable!("read-only")
    }
    fn discard(&self, _: &StagedRelease) -> Result<()> {
        unreachable!("read-only")
    }
    fn set_current(&self, _: Release) -> Result<()> {
        unreachable!("read-only")
    }
}

/// The address of the recorded gateway.
pub const LISTEN: &str = "127.0.0.1:8080";
/// The binary of [`FakeReleases::current`]`("263")`.
pub const SWAP_BINARY: &str = "/data/llama-swap/v263/llama-swap";

pub fn listen() -> SocketAddr {
    LISTEN.parse().unwrap()
}

pub fn record(pid: u32) -> GatewayRecord {
    GatewayRecord {
        pid,
        binary: SWAP_BINARY.into(),
        listen: listen(),
    }
}

/// A change a running gateway makes to its log.
pub enum Logged {
    Appends(&'static str),
    /// A new log takes the place of the old one.
    Replaces(&'static str),
}

/// The gateway in memory. `exits_after` is how many checks a started
/// gateway survives; `None` keeps it running, `stubborn` through SIGTERM,
/// `immortal` through SIGKILL too; `lingers` is how many checks a gateway
/// the signals ended still runs. With `writing`, each check of the
/// gateway changes its log, and it exits with the last change. `locked` is
/// another ai-ctl holding the lock, stopping the gateway; `locks` is what
/// each lock taken said its holder does; `wait_failures` is how many of a
/// started gateway's first checks fail. Each `…_fails` makes its method
/// fail, with the real adapter's kind of error.
#[derive(Default)]
pub struct FakeGateway {
    pub locked: bool,
    pub locks: RefCell<Vec<String>>,
    pub record: RefCell<Option<GatewayRecord>>,
    pub liveness: Option<Liveness>,
    pub config: RefCell<Option<String>>,
    /// The files the last start wrote.
    pub files: RefCell<Vec<ConfigFile>>,
    pub exits_after: Option<u32>,
    pub wait_failures: Cell<u32>,
    pub stubborn: bool,
    pub immortal: bool,
    pub lingers: Cell<u32>,
    pub started: Cell<u32>,
    pub signals: RefCell<Vec<Signal>>,
    pub tail: String,
    pub log: RefCell<Option<Vec<u8>>>,
    pub writing: RefCell<VecDeque<Logged>>,
    pub writes: bool,
    pub forget_fails: bool,
    pub config_fails: bool,
    pub log_tail_fails: bool,
    pub record_fails: bool,
    pub signal_fails: bool,
}

/// The fake gateway's directory.
pub const GATEWAY_DIR: &str = "/state/gateway";

impl FakeGateway {
    /// Whether the signals sent so far ended the gateway.
    pub fn stopped(&self) -> bool {
        let signals = self.signals.borrow();
        !self.immortal
            && (signals.contains(&Signal::Kill)
                || (signals.contains(&Signal::Terminate) && !self.stubborn))
    }
}

pub struct FakeStarted<'a> {
    pub gateway: &'a FakeGateway,
    pub checks: u32,
}

impl StartedGateway for FakeStarted<'_> {
    fn pid(&self) -> u32 {
        4242
    }
    fn exited(&mut self) -> Result<Option<String>> {
        let failures = self.gateway.wait_failures.get();
        if failures > 0 {
            self.gateway.wait_failures.set(failures - 1);
            bail!("cannot wait for llama-swap: No child processes (os error 10)");
        }
        self.checks += 1;
        let killed = self.gateway.stopped();
        let crashed = self
            .gateway
            .exits_after
            .is_some_and(|checks| self.checks > checks);
        Ok((killed || crashed).then(|| "exit status: 1".to_owned()))
    }
}

impl GatewayProcess for FakeGateway {
    fn lock(&self, doing: &str) -> Result<Box<dyn Any>> {
        ensure!(!self.locked, "another ai-ctl is stopping the gateway");
        self.locks.borrow_mut().push(doing.to_owned());
        Ok(Box::new(()))
    }
    fn record(&self) -> Result<Option<GatewayRecord>> {
        ensure!(
            !self.record_fails,
            "cannot read /state/gateway/gateway.json: Permission denied (os error 13)"
        );
        Ok(self.record.borrow().clone())
    }
    fn liveness(&self, _: &GatewayRecord) -> Result<Liveness> {
        if self.stopped() {
            let lingers = self.lingers.get();
            if lingers == 0 {
                return Ok(Liveness::Exited);
            }
            self.lingers.set(lingers - 1);
            return Ok(Liveness::Running);
        }
        if self.writes {
            let mut writing = self.writing.borrow_mut();
            let mut log = self.log.borrow_mut();
            match writing.pop_front() {
                Some(Logged::Appends(text)) => log
                    .get_or_insert_default()
                    .extend_from_slice(text.as_bytes()),
                Some(Logged::Replaces(text)) => *log = Some(text.as_bytes().to_vec()),
                None => {}
            }
            if writing.is_empty() {
                return Ok(Liveness::Exited);
            }
        }
        Ok(self.liveness.clone().unwrap_or(Liveness::Running))
    }
    fn forget(&self) -> Result<()> {
        ensure!(
            !self.forget_fails,
            "cannot remove /state/gateway/gateway.json: Read-only file system (os error 30)"
        );
        *self.record.borrow_mut() = None;
        Ok(())
    }
    fn config(&self) -> Result<Option<String>> {
        ensure!(
            !self.config_fails,
            "cannot read /state/gateway/llama-swap.yaml: Permission denied (os error 13)"
        );
        Ok(self.config.borrow().clone())
    }
    fn files_dir(&self) -> &Path {
        Path::new(GATEWAY_DIR)
    }

    fn start(
        &self,
        binary: &Path,
        config: &str,
        files: &[ConfigFile],
        listen: SocketAddr,
    ) -> Result<Box<dyn StartedGateway + '_>> {
        self.started.set(self.started.get() + 1);
        *self.config.borrow_mut() = Some(config.to_owned());
        *self.files.borrow_mut() = files.to_vec();
        *self.record.borrow_mut() = Some(GatewayRecord {
            pid: 4242,
            binary: binary.to_owned(),
            listen,
        });
        Ok(Box::new(FakeStarted {
            gateway: self,
            checks: 0,
        }))
    }
    fn signal(&self, pid: u32, signal: Signal) -> Result<()> {
        let name = match signal {
            Signal::Terminate => "TERM",
            Signal::Kill => "KILL",
        };
        ensure!(
            !self.signal_fails,
            "kill -s {name} {pid}: kill: sending signal to {pid} failed: Operation not permitted"
        );
        self.signals.borrow_mut().push(signal);
        Ok(())
    }
    fn log(&self) -> PathBuf {
        "/state/logs/gateway.log".into()
    }
    fn log_tail(&self, _: usize) -> Result<String> {
        ensure!(!self.log_tail_fails, "Permission denied (os error 13)");
        Ok(self.tail.clone())
    }
    fn read_log(&self, offset: u64, max: usize) -> Result<Option<Vec<u8>>> {
        let log = self.log.borrow();
        let Some(log) = log.as_ref() else {
            return Ok(None);
        };
        let offset = usize::try_from(offset).unwrap();
        Ok(log
            .get(offset..)
            .map(|rest| rest[..rest.len().min(max)].to_vec()))
    }
}

/// The process table in memory: each process's program name and parent,
/// any other process being gone. Each `…_fails` makes its method fail,
/// with the real adapter's kind of error.
#[derive(Default)]
pub struct FakeProcesses {
    pub stats: BTreeMap<u32, ProcessStat>,
    /// The processes whose cgroup names a podman container.
    pub contained: Vec<u32>,
    pub programs: BTreeMap<u32, PathBuf>,
    pub stat_fails: bool,
    pub in_container_fails: bool,
    pub program_fails: bool,
}

impl FakeProcesses {
    /// Adds `(pid, name, parent)` processes to [`FakeProcesses::stats`].
    pub fn spawned(&mut self, processes: &[(u32, &str, u32)]) {
        for &(pid, name, parent) in processes {
            let name = name.to_owned();
            self.stats.insert(pid, ProcessStat { name, parent });
        }
    }
}

impl ProcessTable for FakeProcesses {
    fn stat(&self, pid: u32) -> Result<Option<ProcessStat>> {
        ensure!(
            !self.stat_fails,
            "cannot read /proc/{pid}/stat: Too many open files (os error 24)"
        );
        Ok(self.stats.get(&pid).cloned())
    }
    fn in_container(&self, pid: u32) -> Result<bool> {
        ensure!(
            !self.in_container_fails,
            "cannot read /proc/{pid}/cgroup: Too many open files (os error 24)"
        );
        Ok(self.contained.contains(&pid))
    }
    fn program(&self, pid: u32) -> Result<Option<PathBuf>> {
        ensure!(
            !self.program_fails,
            "cannot read /proc/{pid}/exe: Too many open files (os error 24)"
        );
        Ok(self.programs.get(&pid).cloned())
    }
}

/// The port check in memory: `taken` is another program listening on the
/// address; `is_free_fails` an address that cannot be bound.
#[derive(Default)]
pub struct FakePorts {
    pub taken: bool,
    pub is_free_fails: bool,
    /// Another program than the started gateway listens on the address.
    pub held_by_another: bool,
    pub listens_fails: bool,
}

impl PortCheck for FakePorts {
    fn is_free(&self, listen: SocketAddr) -> Result<bool> {
        ensure!(
            !self.is_free_fails,
            "cannot bind {listen}: Cannot assign requested address (os error 99)"
        );
        Ok(!self.taken)
    }
    fn listens(&self, _: u32, _: SocketAddr) -> Result<bool> {
        ensure!(
            !self.listens_fails,
            "cannot read /proc/net/tcp: Permission denied (os error 13)"
        );
        Ok(!self.held_by_another)
    }
}

/// Answers the health check from the `answers_after`-th call on, or fails
/// it with `healthy_fails`; with `hangs`, each check waits its whole timeout
/// on the test's clock, then fails, as against a gateway that accepts and
/// never answers; with `first_check`, the first one takes that long, within
/// its timeout, then fails. Lists `models`, or fails to when `None`.
pub struct FakeApi {
    pub answers_after: Option<u32>,
    pub healthy_fails: bool,
    pub hangs: bool,
    pub first_check: Option<Duration>,
    pub calls: Cell<u32>,
    pub unload_fails: bool,
    pub unloads: Cell<u32>,
    pub models: Option<Vec<RunningModel>>,
    pub streamed: RefCell<Vec<(SocketAddr, String)>>,
}

impl FakeApi {
    pub fn answering_after(calls: u32) -> Self {
        Self {
            answers_after: Some(calls),
            healthy_fails: false,
            hangs: false,
            first_check: None,
            calls: Cell::new(0),
            unload_fails: false,
            unloads: Cell::new(0),
            models: Some(vec![]),
            streamed: RefCell::new(vec![]),
        }
    }
    pub fn silent() -> Self {
        Self {
            answers_after: None,
            ..Self::answering_after(0)
        }
    }
}

impl GatewayApi for FakeApi {
    fn healthy(&self, _: SocketAddr, timeout: Duration) -> Result<bool> {
        self.calls.set(self.calls.get() + 1);
        if let (1, Some(first)) = (self.calls.get(), self.first_check) {
            advance(first.min(timeout));
            return Ok(false);
        }
        if self.hangs {
            advance(timeout);
            bail!(
                "GET http://127.0.0.1:8080/health: no answer within {} ms",
                timeout.as_millis()
            );
        }
        ensure!(
            !self.healthy_fails,
            "GET http://127.0.0.1:8080/health: connection reset by peer"
        );
        Ok(self.answers_after.is_some_and(|n| self.calls.get() > n))
    }
    fn unload_all(&self, _: SocketAddr) -> Result<()> {
        self.unloads.set(self.unloads.get() + 1);
        ensure!(!self.unload_fails, "connection reset");
        Ok(())
    }
    fn running(&self, _: SocketAddr) -> Result<Vec<RunningModel>> {
        self.models
            .clone()
            .ok_or_else(|| anyhow!("GET http://127.0.0.1:8080/running: timed out"))
    }
    fn stream_log(&self, listen: SocketAddr, model: &str, out: &mut dyn Write) -> Result<()> {
        self.streamed.borrow_mut().push((listen, model.to_owned()));
        out.write_all(b"srv  load_model: loading model\n")?;
        Ok(())
    }
}

/// Podman in memory: `local` images, and the PIDs of the `running`
/// containers. Each `…_fails` makes its method fail.
#[derive(Default)]
pub struct FakeContainers {
    pub local: Vec<String>,
    pub pulled: RefCell<Vec<String>>,
    pub running: Vec<u32>,
    /// How many times the running containers were listed.
    pub listed: Cell<u32>,
    pub image_exists_fails: bool,
    pub pull_fails: bool,
    pub pids_fails: bool,
}

impl Containers for FakeContainers {
    fn image_exists(&self, image: &str) -> Result<bool> {
        ensure!(
            !self.image_exists_fails,
            "podman image exists {image}: cannot connect to the storage"
        );
        Ok(self.local.iter().any(|local| local == image))
    }
    fn pull(&self, image: &str) -> Result<()> {
        self.pulled.borrow_mut().push(image.to_owned());
        ensure!(!self.pull_fails, "podman pull {image}: manifest unknown");
        Ok(())
    }
    fn pids(&self, prefix: &str) -> Result<Vec<u32>> {
        assert_eq!(prefix, CONTAINER_PREFIX);
        self.listed.set(self.listed.get() + 1);
        ensure!(
            !self.pids_fails,
            "podman ps --filter name={prefix}: cannot connect to the storage"
        );
        Ok(self.running.clone())
    }
    fn digest(&self, image: &str) -> Result<String> {
        ensure!(
            self.local.iter().any(|local| local == image),
            "podman image inspect {image}: no such image"
        );
        Ok(format!("sha256:{}", "ab".repeat(32)))
    }
}

/// The GPU read again on each call: each reading is given once, the last
/// one for good, and takes `read_time` on the test's clock.
pub struct ScriptedGpu {
    pub readings: RefCell<Vec<Probed<Gpu>>>,
    pub reads: Cell<u32>,
    pub read_time: Duration,
}

impl ScriptedGpu {
    pub fn new(readings: Vec<Probed<Gpu>>) -> Self {
        Self {
            readings: RefCell::new(readings),
            reads: Cell::new(0),
            read_time: Duration::ZERO,
        }
    }
}

impl GpuProbe for ScriptedGpu {
    fn gpu(&self) -> Probed<Gpu> {
        self.reads.set(self.reads.get() + 1);
        advance(self.read_time);
        let mut readings = self.readings.borrow_mut();
        if readings.len() > 1 {
            readings.remove(0)
        } else {
            readings[0].clone()
        }
    }
}

/// The GPU with these processes on it.
pub fn gpu_running(processes: &[(u32, &str, u64)]) -> Probed<Gpu> {
    let mut gpu = gpu();
    gpu.processes = processes
        .iter()
        .map(|&(pid, name, mib)| GpuProcess {
            pid,
            name: name.into(),
            memory: Some(mib << 20),
        })
        .collect();
    Probed::Found(gpu)
}

/// A model server whose tokenizer makes one token of each character, and
/// whose template marks each message with its role.
#[derive(Default)]
pub struct FakeModelServer {
    pub calls: RefCell<Vec<String>>,
    /// The model's context; 98,304 tokens when `None`.
    pub ctx_size: Option<u32>,
    /// The code's last character and the line break after it make one
    /// token, as a real tokenizer may merge them.
    pub boundary_merge: bool,
    /// Every count is rounded up to an even number.
    pub even: bool,
    /// Every prompt holding code counts this target and 3 more tokens, as
    /// a tokenizer whose merges no attempt gets under.
    pub overshoot: Option<usize>,
    pub fails: bool,
    /// What `/metrics` gives at each reading; none when it is disabled.
    pub counters: RefCell<Vec<SpecCounters>>,
}

impl FakeModelServer {
    fn call(&self, name: &str, model: &str, listen: SocketAddr) -> Result<()> {
        self.calls
            .borrow_mut()
            .push(format!("{name} {model} {listen}"));
        ensure!(!self.fails, "connection refused");
        Ok(())
    }
}

impl ModelServer for FakeModelServer {
    fn context_size(&self, listen: SocketAddr, model: &str) -> Result<u32> {
        self.call("props", model, listen)?;
        Ok(self.ctx_size.unwrap_or(98_304))
    }

    fn template(&self, listen: SocketAddr, model: &str, messages: &[Message]) -> Result<String> {
        self.call("template", model, listen)?;
        Ok(messages
            .iter()
            .map(|message| format!("<{}>{}", message.role.name(), message.content))
            .collect())
    }

    fn pieces(&self, listen: SocketAddr, model: &str, text: &str) -> Result<Vec<Vec<u8>>> {
        self.call("pieces", model, listen)?;
        Ok(text.chars().map(|c| c.to_string().into_bytes()).collect())
    }

    fn count(&self, listen: SocketAddr, model: &str, text: &str) -> Result<usize> {
        self.call("count", model, listen)?;
        if let Some(target) = self.overshoot
            && text.contains("<file path=\"src/")
        {
            return Ok(target + 3);
        }
        let mut count = text.chars().count();
        if self.boundary_merge
            && let Some(at) = text.find("\n\nYou have read")
            && !text[..at].ends_with("\n\n")
        {
            count -= 1;
        }
        if self.even {
            count += count % 2;
        }
        Ok(count)
    }

    fn spec_counters(&self, listen: SocketAddr, model: &str) -> Result<SpecCounters> {
        self.call("metrics", model, listen)?;
        let mut counters = self.counters.borrow_mut();
        ensure!(!counters.is_empty(), "answered 501: metrics are disabled");
        Ok(counters.remove(0))
    }
}

/// A llama-server with one slot, whose tokenizer makes one token of each
/// character. A prompt is its messages marked with their roles, each
/// reasoning before its content; what it shares with the slot's last prompt
/// and reply comes from the cache, unless the request asks for none. A reply is `max_tokens` characters of
/// reasoning: 100 ms to the first, 20 ms between the others, each of these
/// drafted and every other draft accepted.
#[derive(Default)]
pub struct FakeChat {
    pub requests: RefCell<Vec<(SocketAddr, ChatRequest)>>,
    /// The request, from 0, that fails.
    pub fails_at: Option<usize>,
    slot: RefCell<String>,
}

impl FakeChat {
    /// The drafts of a reply of `max_tokens` tokens.
    pub fn drafts(max_tokens: u32) -> Drafts {
        let proposed = u64::from(max_tokens.saturating_sub(1));
        Drafts {
            proposed,
            accepted: proposed / 2,
        }
    }

    /// The prompt `messages` make.
    pub fn prompt(messages: &[Message]) -> String {
        messages
            .iter()
            .map(|message| {
                let reasoning = message.reasoning.as_deref().unwrap_or("");
                format!("<{}>{reasoning}{}", message.role.name(), message.content)
            })
            .collect()
    }
}

impl ChatClient for FakeChat {
    fn stream(&self, addr: SocketAddr, request: &ChatRequest) -> Result<Reply> {
        let index = {
            let mut requests = self.requests.borrow_mut();
            requests.push((addr, request.clone()));
            requests.len() - 1
        };
        ensure!(
            self.fails_at != Some(index),
            "answered 500: the slot is busy"
        );
        let prompt = Self::prompt(&request.messages);
        let tokens = prompt.chars().count() as u64;
        let mut slot = self.slot.borrow_mut();
        let shared = if request.cache_prompt {
            slot.chars()
                .zip(prompt.chars())
                .take_while(|(cached, sent)| cached == sent)
                .count() as u64
        } else {
            0
        };
        // The last token of a prompt is always evaluated.
        let cache_n = shared.min(tokens.saturating_sub(1));
        let generated = u64::from(request.max_tokens);
        let reasoning = "r".repeat(request.max_tokens as usize);
        *slot = format!("{prompt}<assistant>{reasoning}");
        let gap = Duration::from_millis(20);
        let ttft = Duration::from_millis(100);
        Ok(Reply {
            sample: Sample {
                client: ClientTimings {
                    ttft,
                    total: ttft + gap * request.max_tokens.saturating_sub(1),
                    gaps: vec![gap; request.max_tokens.saturating_sub(1) as usize],
                },
                server: Some(ServerTimings {
                    cache_n,
                    prompt_n: tokens - cache_n,
                    prompt_ms: (tokens - cache_n) as f64 / 2.0,
                    predicted_n: generated,
                    predicted_ms: generated as f64 * 20.0,
                    drafts: (generated > 1).then(|| FakeChat::drafts(request.max_tokens)),
                }),
                usage: Some(Usage {
                    prompt_tokens: tokens,
                    completion_tokens: generated,
                }),
            },
            content: String::new(),
            reasoning,
        })
    }
}

/// Reads the GPU once as it starts and once as each scenario starts: the
/// memory used grows by 1000 MiB at each reading, from 1000. From the
/// scenario `lost_from` on, every reading fails.
#[derive(Default)]
pub struct FakeMonitor {
    pub fails: bool,
    pub lost_from: Option<Scenario>,
    pub finished: Cell<bool>,
    pub finish_fails: bool,
}

pub struct FakeMonitoring<'a> {
    monitor: &'a FakeMonitor,
    readings: RefCell<Vec<GpuReading>>,
    /// The readings that failed.
    lost: Cell<usize>,
}

impl FakeMonitoring<'_> {
    fn read(&self, scenario: Option<Scenario>) {
        let lost_from = self.monitor.lost_from;
        if self.lost.get() > 0 || (lost_from.is_some() && scenario == lost_from) {
            self.lost.set(self.lost.get() + 1);
            return;
        }
        let mut readings = self.readings.borrow_mut();
        let memory_used_mib = 1000 * (readings.len() as u64 + 1);
        readings.push(GpuReading {
            scenario,
            memory_used_mib,
            temperature_c: 70,
            sm_clock_mhz: 2000,
            power_w: 100.0,
            slowdown: Slowdown::default(),
            clock_events: Some(0),
        });
    }
}

impl GpuMonitor for FakeMonitor {
    fn start(&self, _: Duration) -> Result<Box<dyn Monitoring + '_>> {
        ensure!(!self.fails, "NVML: driver not loaded");
        let monitoring = FakeMonitoring {
            monitor: self,
            readings: RefCell::new(Vec::new()),
            lost: Cell::new(0),
        };
        monitoring.read(None);
        Ok(Box::new(monitoring))
    }
}

impl Monitoring for FakeMonitoring<'_> {
    fn enter(&self, scenario: Scenario) {
        self.read(Some(scenario));
    }

    fn finish(self: Box<Self>) -> Result<Readings> {
        self.monitor.finished.set(true);
        ensure!(!self.monitor.finish_fails, "the GPU's reading failed");
        let lost = self.lost.get();
        Ok(Readings {
            taken: self.readings.take(),
            failed: (lost > 0).then(|| (lost, "NVML: the GPU is lost".to_owned())),
        })
    }
}

/// The saved runs in memory, by ID. A run of `unreadable` fails to load
/// with its reason; with `full`, every save fails.
#[derive(Default)]
pub struct FakeRuns {
    pub runs: RefCell<BTreeMap<String, Run>>,
    pub unreadable: BTreeMap<String, String>,
    pub full: bool,
    /// Each run saved or replaced, in turn.
    pub writes: RefCell<Vec<Run>>,
}

impl FakeRuns {
    fn path(id: &str) -> PathBuf {
        PathBuf::from(format!("/data/bench/{id}.json"))
    }
}

impl RunStore for FakeRuns {
    fn save(&self, run: &Run) -> Result<PathBuf> {
        let id = run.id();
        let path = Self::path(&id);
        ensure!(
            !self.full,
            "cannot write {}: No space left on device (os error 28)",
            path.display()
        );
        let mut runs = self.runs.borrow_mut();
        ensure!(
            !runs.contains_key(&id) && !self.unreadable.contains_key(&id),
            "a run is already saved as {}",
            path.display()
        );
        runs.insert(id, run.clone());
        self.writes.borrow_mut().push(run.clone());
        Ok(path)
    }
    fn replace(&self, run: &Run) -> Result<PathBuf> {
        let id = run.id();
        let path = Self::path(&id);
        ensure!(
            !self.full,
            "cannot write {}: No space left on device (os error 28)",
            path.display()
        );
        let mut runs = self.runs.borrow_mut();
        assert!(runs.contains_key(&id), "a run is replaced once saved only");
        runs.insert(id, run.clone());
        self.writes.borrow_mut().push(run.clone());
        Ok(path)
    }
    fn ids(&self) -> Result<Vec<String>> {
        let runs = self.runs.borrow();
        let mut ids: Vec<String> = runs.keys().chain(self.unreadable.keys()).cloned().collect();
        ids.sort();
        Ok(ids)
    }
    fn load(&self, id: &str) -> Result<Run> {
        if let Some(reason) = self.unreadable.get(id) {
            bail!("cannot read /data/bench/{id}.json: {reason}");
        }
        self.runs
            .borrow()
            .get(id)
            .cloned()
            .ok_or_else(|| anyhow!("no run is saved as {id}; `ai-ctl bench list` lists them"))
    }
}

/// A writer whose reader has gone, as `ai-ctl logs -f | head` leaves it.
pub struct ClosedPipe;

impl Write for ClosedPipe {
    fn write(&mut self, _: &[u8]) -> std::io::Result<usize> {
        Err(std::io::ErrorKind::BrokenPipe.into())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

/// Whether `err` comes from a write to a [`ClosedPipe`].
pub fn broken_pipe(err: &anyhow::Error) -> bool {
    err.chain().any(|cause| {
        cause
            .downcast_ref::<std::io::Error>()
            .is_some_and(|err| err.kind() == std::io::ErrorKind::BrokenPipe)
    })
}
