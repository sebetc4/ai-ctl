//! Ports: what the use cases need from outside the process.

use std::any::Any;
use std::collections::BTreeMap;
use std::io::Write;
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::time::Duration;

use anyhow::{Result, bail};

use crate::domain::bench::{GpuReading, Message, Run, Sample, Scenario, SpecCounters};
use crate::domain::build::{Build, BuildId, CommitId};
use crate::domain::gateway::{Release, Sha256};
use crate::domain::gateway_config::ConfigFile;
use crate::domain::inventory::Inventory;
use crate::domain::machine::{
    CdiSpec, CudaRepository, CudaToolkit, Gpu, HostCompiler, Probed, Tool, Version,
};
use crate::domain::recipe::{Patch, Recipe};

/// Reads the NVIDIA GPU and its driver.
pub trait GpuProbe {
    fn gpu(&self) -> Probed<Gpu>;
}

/// The GPU `reading` found, or why it found none; each caller adds what it
/// needed the GPU for.
pub fn found_gpu(reading: &Probed<Gpu>) -> Result<&Gpu> {
    match reading {
        Probed::Found(gpu) => Ok(gpu),
        Probed::Missing => bail!("no NVIDIA GPU found; run `ai-ctl doctor`"),
        Probed::Failed(err) => bail!("cannot read the GPU: {err}"),
    }
}

/// Reads the GPU at a fixed period while a bench runs.
pub trait GpuMonitor {
    /// Starts reading the GPU every `period`, on a thread of its own, after
    /// a first reading that shows it can be read.
    fn start(&self, period: Duration) -> Result<Box<dyn Monitoring + '_>>;
}

/// What reading the GPU through a run gave.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Readings {
    /// The readings taken, in order.
    pub taken: Vec<GpuReading>,
    /// How many readings failed, with the last one's error; `None` when
    /// none did.
    pub failed: Option<(usize, String)>,
}

/// The GPU being read; dropped, the reading stops.
pub trait Monitoring {
    /// Labels the readings to come with `scenario`.
    fn enter(&self, scenario: Scenario);
    /// Stops reading, and returns what it gave.
    fn finish(self: Box<Self>) -> Result<Readings>;
}

/// Reads the host's build toolchain, read-only: the CUDA toolkit, the
/// compilers and tools, and the packages and repositories they come from.
pub trait ToolchainProbe {
    /// The toolkit of the `nvcc` a build would use.
    fn cuda_toolkit(&self) -> Probed<CudaToolkit>;
    /// The GNU C++ compilers on the `PATH`, the default `g++` first.
    fn host_compilers(&self) -> Probed<Vec<HostCompiler>>;
    fn tool_version(&self, tool: Tool) -> Probed<Version>;
    fn is_file(&self, path: &Path) -> bool;
    fn is_dir(&self, path: &Path) -> bool;
    /// The names of the entries of the directory `path`.
    fn entries(&self, path: &Path) -> Result<Vec<String>>;
    /// A model's text file, such as a safetensors index, read whole: one
    /// too large for such a file is refused, as one that is not UTF-8.
    fn read_text(&self, path: &Path) -> Result<String>;
    /// The Fedora release, from `/etc/os-release`; `Missing` on another
    /// distribution.
    fn fedora_release(&self) -> Probed<u32>;
    /// The enabled NVIDIA CUDA repositories, read from dnf's cache.
    fn cuda_repositories(&self) -> Probed<Vec<CudaRepository>>;
    /// The installed toolkit meta-packages of `releases` and of the
    /// toolkits installed under `/usr/local`, as
    /// `domain::machine::toolkit_meta_packages` names them.
    fn toolkit_packages(&self, releases: &[(u32, u32)]) -> Probed<Vec<String>>;
}

/// Reads how containers reach the GPU, read-only: CDI's devices and
/// specifications.
pub trait CdiProbe {
    /// The CDI device names `nvidia-ctk` lists, such as `nvidia.com/gpu=all`.
    fn cdi_devices(&self) -> Probed<Vec<String>>;
    /// The specifications of NVIDIA's GPUs in CDI's directories, `/etc/cdi`
    /// then `/var/run/cdi`, each by name: of two that define a device,
    /// podman uses the later.
    fn cdi_specs(&self) -> Probed<Vec<CdiSpec>>;
}

/// Reads the machine's power state, read-only.
pub trait PowerProbe {
    /// Whether an external power supply is online; `Missing` when the machine
    /// reports none.
    fn ac_power(&self) -> Probed<bool>;
    /// The active power profile, such as `performance`.
    fn power_profile(&self) -> Probed<String>;
}

/// What [`InventoryStore::install`] did.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Installed {
    Written,
    /// An inventory was already there; it was left untouched.
    AlreadyExists,
    /// A symbolic link is there, to `target`, which does not exist; it was
    /// left untouched.
    DanglingLink {
        target: PathBuf,
    },
}

/// The installed inventory file.
pub trait InventoryStore {
    /// Where the inventory lives.
    fn location(&self) -> &Path;
    /// The installed inventory, or `None` when none is installed.
    fn load(&self) -> Result<Option<Inventory>>;
    /// Installs the shipped inventory with `models_dir` substituted, unless
    /// one is already installed.
    fn install(&self, models_dir: &Path) -> Result<Installed>;
}

/// What [`Sources::checkout`] checked out.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Checkout {
    /// The working tree, at the commit.
    pub dir: PathBuf,
    pub commit: CommitId,
    /// The tags pointing at the commit.
    pub tags: Vec<String>,
}

/// A project's sources, from a clone of its git repository.
pub trait Sources {
    /// Clones the project's repository, or fetches it when it is already
    /// cloned.
    fn fetch(&self, recipe: &Recipe) -> Result<()>;
    /// The tags of the project's clone that may be releases; the recipe's
    /// pattern sorts them out.
    fn release_tags(&self, recipe: &Recipe) -> Result<Vec<String>>;
    /// Checks out `reference` in the project's clone, as the last
    /// [`Sources::fetch`] left it: a tag, a branch or a commit, with the
    /// recipe's submodules.
    fn checkout(&self, recipe: &Recipe, reference: &str) -> Result<Checkout>;
    /// Applies `patch` to the checked-out tree, or changes nothing when any
    /// of it does not apply; the next checkout undoes it.
    fn apply(&self, checkout: &Checkout, patch: &Patch) -> Result<()>;
}

/// One CMake build: a fresh build tree for `source`, configured with
/// `flags`, building `target` only.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CompileJob<'a> {
    pub project: &'a str,
    pub source: &'a Path,
    pub flags: &'a BTreeMap<String, String>,
    pub target: &'a str,
    /// Where the target's binary lands, relative to the build tree.
    pub artifact: &'a str,
    /// Parallel compile jobs; `None` for one per processor.
    pub jobs: Option<usize>,
}

/// What a successful [`Compiler::compile`] produced.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Compiled {
    /// The target's binary, in the build tree.
    pub artifact: PathBuf,
    /// The complete output of CMake and the compilers.
    pub log: PathBuf,
}

/// How far a compilation is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Progress {
    Configuring,
    /// `done` of `total` build steps finished.
    Building {
        done: u32,
        total: u32,
    },
}

/// Compiles a CMake project.
pub trait Compiler {
    /// Configures a fresh build tree and builds the job's target, writing
    /// every line of output to the build log. A failure names the log and
    /// quotes its last lines.
    fn compile(&self, job: &CompileJob, progress: &mut dyn FnMut(Progress)) -> Result<Compiled>;
    /// Removes the project's build tree.
    fn clean(&self, project: &str) -> Result<()>;
}

/// A build made but not yet installed: its directory, out of sight of the
/// installed builds, holds the binary and the metadata.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Staged {
    pub dir: PathBuf,
    pub binary: PathBuf,
    pub build: Build,
}

/// A directory of the store that is not a readable build.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Unreadable {
    pub dir: PathBuf,
    pub reason: String,
}

/// The installed builds of every project, with each project's current one.
pub trait BuildStore {
    /// Takes the lock every change to the builds holds, released when the
    /// returned guard is dropped; fails at once when another ai-ctl holds it.
    fn lock(&self) -> Result<Box<dyn Any>>;
    /// The project's installed builds, and the directories that are not.
    fn builds(&self, project: &str) -> Result<(Vec<Build>, Vec<Unreadable>)>;
    /// The build the project's `current` link names, if any.
    fn current(&self, project: &str) -> Result<Option<BuildId>>;
    /// The installed binary of a build.
    fn binary(&self, project: &str, id: &BuildId, name: &str) -> PathBuf;
    /// Copies `artifact` and `log` beside the build's metadata, with the
    /// patches applied to its sources, out of sight.
    fn stage(
        &self,
        build: &Build,
        artifact: &Path,
        log: &Path,
        patches: &[Patch],
    ) -> Result<Staged>;
    /// Installs a staged build under its id; with `replace`, an installed
    /// build of the same id gives way to it.
    fn publish(&self, staged: &Staged, replace: bool) -> Result<()>;
    /// Removes a staged build.
    fn discard(&self, staged: &Staged) -> Result<()>;
    /// Points the project's `current` link at an installed build.
    fn set_current(&self, project: &str, id: &BuildId) -> Result<()>;
    /// Removes an installed build.
    fn remove(&self, project: &str, id: &BuildId) -> Result<()>;
    /// Whether `program` lies in the store: the binary of a build, installed
    /// or removed since.
    fn contains(&self, program: &Path) -> bool;
}

/// What a program printed, and whether it succeeded.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProgramOutput {
    pub success: bool,
    pub stdout: String,
    pub stderr: String,
}

/// Runs a program briefly, such as a new build's smoke test.
pub trait Programs {
    fn run(&self, program: &Path, args: &[&str]) -> Result<ProgramOutput>;
}

/// Files fetched over HTTPS.
pub trait Downloads {
    /// Saves the file at `url` as `to`.
    fn download(&self, url: &str, to: &Path) -> Result<()>;
    /// The content of the text file at `url`.
    fn text(&self, url: &str) -> Result<String>;
}

/// A gateway release being installed: a directory out of sight of the
/// installed releases, where its archive is downloaded and unpacked.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StagedRelease {
    pub release: Release,
    pub dir: PathBuf,
    pub archive: PathBuf,
}

/// The installed releases of the gateway, with the current one.
pub trait GatewayStore {
    /// Takes the lock every change to the releases holds, released when the
    /// returned guard is dropped; fails at once when another ai-ctl holds it.
    fn lock(&self) -> Result<Box<dyn Any>>;
    /// Whether the release's binary is installed.
    fn is_installed(&self, release: Release) -> Result<bool>;
    /// The release the `current` link names, if any.
    fn current(&self) -> Result<Option<Release>>;
    /// The installed binary of a release.
    fn binary(&self, release: Release) -> PathBuf;
    /// An empty directory for the release, out of sight.
    fn stage(&self, release: Release) -> Result<StagedRelease>;
    /// The SHA-256 of a file.
    fn sha256(&self, file: &Path) -> Result<Sha256>;
    /// Extracts the binary from the staged archive and removes the archive;
    /// returns the binary.
    fn unpack(&self, staged: &StagedRelease) -> Result<PathBuf>;
    /// Installs a staged release; fails when the release is installed
    /// already. A directory of the release without its binary gives way.
    fn publish(&self, staged: &StagedRelease) -> Result<()>;
    /// Removes a staged release.
    fn discard(&self, staged: &StagedRelease) -> Result<()>;
    /// Points the `current` link at an installed release.
    fn set_current(&self, release: Release) -> Result<()>;
}

/// The gateway ai-ctl started, as its state directory records it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GatewayRecord {
    pub pid: u32,
    /// The llama-swap binary it runs.
    pub binary: PathBuf,
    pub listen: SocketAddr,
}

/// What runs under a recorded gateway's PID now.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Liveness {
    /// The recorded binary: the gateway runs.
    Running,
    /// Nothing: the gateway has exited.
    Exited,
    /// Another program, named when it can be read: the PID was reused.
    Reused(Option<PathBuf>),
}

impl Liveness {
    /// What became of a recorded gateway that no longer runs, for a message
    /// that names it by `pid`: `has exited`, or `is gone; PID 1234 now runs …`.
    pub fn describe(&self, pid: u32) -> String {
        match self {
            Liveness::Exited => "has exited".to_owned(),
            Liveness::Reused(Some(program)) => {
                format!("is gone; PID {pid} now runs {}", program.display())
            }
            Liveness::Reused(None) | Liveness::Running => {
                format!("is gone; PID {pid} now runs another user's program")
            }
        }
    }
}

/// A signal for a gateway to stop.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Signal {
    /// SIGTERM: llama-swap stops its models, then exits.
    Terminate,
    /// SIGKILL: the process ends at once, its models left running.
    Kill,
}

/// A gateway just started, still a child of this process.
pub trait StartedGateway {
    fn pid(&self) -> u32;
    /// Its exit status, once it has exited.
    fn exited(&mut self) -> Result<Option<String>>;
}

/// What `/proc/<pid>/stat` gives of a process.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcessStat {
    /// The program's name, which the kernel cuts at 15 bytes.
    pub name: String,
    pub parent: u32,
}

/// The gateway ai-ctl runs on this machine: its process, and the files it
/// keeps, its record, its configuration and its log.
pub trait GatewayProcess {
    /// Takes the lock that starting and stopping the gateway hold, released
    /// when the returned guard is dropped, saying the holder is `doing` it;
    /// fails at once when another ai-ctl holds it, saying what that one is
    /// doing.
    fn lock(&self, doing: &str) -> Result<Box<dyn Any>>;
    /// The recorded gateway, if any.
    fn record(&self) -> Result<Option<GatewayRecord>>;
    fn liveness(&self, record: &GatewayRecord) -> Result<Liveness>;
    /// Removes the record.
    fn forget(&self) -> Result<()>;
    /// The configuration the gateway was last started with.
    fn config(&self) -> Result<Option<String>>;
    /// The gateway's directory, under which go the files its models read.
    fn files_dir(&self) -> &Path;
    /// Writes `config` and `files`, then starts `binary` on it, listening on
    /// `listen`, detached from the terminal, its output in a new log; and
    /// records it.
    fn start(
        &self,
        binary: &Path,
        config: &str,
        files: &[ConfigFile],
        listen: SocketAddr,
    ) -> Result<Box<dyn StartedGateway + '_>>;
    /// Sends `signal` to the gateway's process `pid`; a process already
    /// gone is not an error.
    fn signal(&self, pid: u32, signal: Signal) -> Result<()>;
    /// The gateway's log.
    fn log(&self) -> PathBuf;
    /// The last `lines` lines of the log.
    fn log_tail(&self, lines: usize) -> Result<String>;
    /// At most `max` bytes of the log from `offset` on, empty at its end;
    /// `None` when there is no log, or one shorter than `offset`: a new log
    /// replaced it.
    fn read_log(&self, offset: u64, max: usize) -> Result<Option<Vec<u8>>>;
}

/// The processes of this machine, any of them, as `/proc` shows them.
pub trait ProcessTable {
    /// The program name and the parent of the process `pid`; `None` when
    /// it is gone.
    fn stat(&self, pid: u32) -> Result<Option<ProcessStat>>;
    /// Whether the process `pid` runs in a podman container, its cgroup
    /// naming a `libpod-` scope; `false` when it is gone.
    fn in_container(&self, pid: u32) -> Result<bool>;
    /// The program the process `pid` runs, as `/proc/<pid>/exe` names it;
    /// `None` when the process is gone or belongs to another user.
    fn program(&self, pid: u32) -> Result<Option<PathBuf>>;
}

/// Whether an address is free to listen on, and who listens on it.
pub trait PortCheck {
    /// Whether a server could listen on `listen` now.
    fn is_free(&self, listen: SocketAddr) -> Result<bool>;
    /// Whether the process `pid` holds the socket that listens on `listen`.
    fn listens(&self, pid: u32, listen: SocketAddr) -> Result<bool>;
}

/// A model the gateway runs, as llama-swap reports it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunningModel {
    pub id: String,
    /// `starting`, `ready` or `stopping`.
    pub state: String,
    /// The address of the model's own server, to which llama-swap passes
    /// its requests on; `None` when llama-swap gives none, or a URL that is
    /// not `http://<ip>:<port>`.
    pub proxy: Option<SocketAddr>,
}

/// llama-swap's HTTP API.
pub trait GatewayApi {
    /// Whether a gateway listening on `listen` answers its health check,
    /// waiting `timeout` at most to connect and as much to read; `false`
    /// while nothing listens there.
    fn healthy(&self, listen: SocketAddr, timeout: Duration) -> Result<bool>;
    /// Stops every model the gateway runs, and returns once they are
    /// stopped.
    fn unload_all(&self, listen: SocketAddr) -> Result<()>;
    /// The models the gateway runs, loading or stopping included.
    fn running(&self, listen: SocketAddr) -> Result<Vec<RunningModel>>;
    /// Copies to `out` the output of `model` the gateway kept, then its live
    /// output as it comes, until the gateway ends the stream.
    fn stream_log(&self, listen: SocketAddr, model: &str, out: &mut dyn Write) -> Result<()>;
}

/// A chat completion request, streamed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChatRequest {
    /// The model's ID: llama-swap routes on it, and llama-server answers to
    /// it.
    pub model: String,
    pub messages: Vec<Message>,
    /// Tokens to generate at most.
    pub max_tokens: u32,
    /// Whether to generate `max_tokens` exactly, the end of sequence ignored.
    pub ignore_eos: bool,
    /// The sampling seed, so two runs generate the same tokens.
    pub seed: u32,
    /// Whether the server may take the prompt's beginning from its cache;
    /// `false` evaluates it whole.
    pub cache_prompt: bool,
}

/// A streamed reply, timed.
#[derive(Debug, Clone, PartialEq)]
pub struct Reply {
    pub sample: Sample,
    /// The reply's text and reasoning, which a session sends back.
    pub content: String,
    pub reasoning: String,
}

/// Chat completions, streamed: the gateway's, or a model's own server's.
pub trait ChatClient {
    /// Sends `request` to the server listening on `addr`, and times the
    /// stream of its reply.
    fn stream(&self, addr: SocketAddr, request: &ChatRequest) -> Result<Reply>;
}

/// llama-server's own endpoints, which the gateway passes through at
/// `/upstream/<model>/`, loading the model first when it must.
pub trait ModelServer {
    /// The context size the server runs with (`n_ctx`).
    fn context_size(&self, listen: SocketAddr, model: &str) -> Result<u32>;
    /// The prompt the model's chat template makes of `messages`, ready for
    /// the reply.
    fn template(&self, listen: SocketAddr, model: &str, messages: &[Message]) -> Result<String>;
    /// The tokens of `text`, each as the bytes it stands for.
    fn pieces(&self, listen: SocketAddr, model: &str, text: &str) -> Result<Vec<Vec<u8>>>;
    /// How many tokens `text` makes.
    fn count(&self, listen: SocketAddr, model: &str, text: &str) -> Result<usize>;
    /// The counters of speculative decoding since the server started, from
    /// its `/metrics`.
    fn spec_counters(&self, listen: SocketAddr, model: &str) -> Result<SpecCounters>;
}

/// The bench's saved runs.
pub trait RunStore {
    /// Saves `run` under its ID, and returns where; fails when a run is
    /// saved under that ID already.
    fn save(&self, run: &Run) -> Result<PathBuf>;
    /// Saves `run` in place of the file a [`Self::save`] of the same run
    /// wrote, whole or not at all, and returns where.
    fn replace(&self, run: &Run) -> Result<PathBuf>;
    /// The IDs of the saved runs, oldest first.
    fn ids(&self) -> Result<Vec<String>>;
    /// The run saved under `id`.
    fn load(&self, id: &str) -> Result<Run>;
}

/// Containers and their images, in podman's local storage.
pub trait Containers {
    fn image_exists(&self, image: &str) -> Result<bool>;
    /// Pulls `image` from its registry.
    fn pull(&self, image: &str) -> Result<()>;
    /// The host PIDs of the running containers whose name starts with
    /// `prefix`.
    fn pids(&self, prefix: &str) -> Result<Vec<u32>>;
    /// The digest of a local image, such as `sha256:3f1c…`.
    fn digest(&self, image: &str) -> Result<String>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::fakes::gpu;

    #[test]
    fn a_gone_gateway_is_described_by_what_runs_under_its_pid() {
        assert_eq!(Liveness::Exited.describe(1234), "has exited");
        assert_eq!(
            Liveness::Reused(Some("/usr/bin/vim".into())).describe(1234),
            "is gone; PID 1234 now runs /usr/bin/vim"
        );
        assert_eq!(
            Liveness::Reused(None).describe(1234),
            "is gone; PID 1234 now runs another user's program"
        );
    }

    #[test]
    fn a_reading_without_a_gpu_says_why() {
        assert_eq!(found_gpu(&Probed::Found(gpu())).unwrap(), &gpu());
        let missing = found_gpu(&Probed::Missing).unwrap_err();
        assert_eq!(
            format!("{missing:#}"),
            "no NVIDIA GPU found; run `ai-ctl doctor`"
        );
        let failed = found_gpu(&Probed::Failed("NVML: Unknown Error".into())).unwrap_err();
        assert_eq!(
            format!("{failed:#}"),
            "cannot read the GPU: NVML: Unknown Error"
        );
    }
}
