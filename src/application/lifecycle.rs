//! The gateway's lifecycle: `up` starts it on the configuration the
//! inventory gives, once the images its models run are in local storage;
//! `down` stops it, returning once the GPU memory its models held is
//! released; `status` and `logs` read it.

use std::collections::BTreeSet;
use std::io::Write;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::time::{Duration, Instant};

use anyhow::{Context, Result, anyhow, bail, ensure};

use crate::domain::gateway::{OLDEST, PINNED, Release, Sha256};
use crate::domain::gateway_config::CONTAINER_PREFIX;
use crate::domain::inventory::{check_id, is_pinned};
use crate::domain::machine::{Gpu, GpuProcess, Probed};

use super::gateway_config::{Configurator, Prepared};
use super::ports::{
    Containers, GatewayApi, GatewayProcess, GatewayRecord, GatewayStore, Liveness, PortCheck,
    ProcessTable, RunningModel, Signal, StartedGateway, found_gpu,
};

/// How often a starting or stopping gateway is checked.
const POLL: Duration = Duration::from_millis(100);
/// The first pause of a stop's checks, doubled up to `POLL`: llama-swap
/// ends a millisecond or two after SIGTERM once its models are unloaded.
const FIRST_STOP_POLL: Duration = Duration::from_millis(5);
/// llama-swap listens within a second; 30 s leaves room for a loaded
/// machine.
const START_WAIT: Duration = Duration::from_secs(30);
/// The most a health check waits to connect, and as much to read: a
/// refused port answers in a tenth of a millisecond and llama-swap's
/// `/health` in about a millisecond, so a slow check is one that will not
/// answer, and the deadline leaves room for the next.
const CHECK: Duration = Duration::from_millis(500);
/// llama-swap stops its models within 30 s of SIGTERM, then forces its own
/// exit 5 s later.
const STOP_WAIT: Duration = Duration::from_secs(40);
/// What SIGKILL leaves the process to end.
const KILL_WAIT: Duration = Duration::from_secs(5);
/// The driver releases a process's memory as it exits: 30 s is for a
/// process still exiting.
const RELEASE_WAIT: Duration = Duration::from_secs(30);
/// The lines of the log a failure quotes.
const TAIL: usize = 15;
/// What to do about another program on the gateway's address.
const ANOTHER_PROGRAM: &str = "stop it, or give the gateway another `listen` address in the \
     inventory. A program you do not know may have taken the API key from a client while the \
     gateway was down: change the key, as docs/security.md §3 says";
/// The most of the log read at once.
const LOG_CHUNK: usize = 1 << 20;

/// A step of `up` or `down`, for the user to follow.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Event {
    /// The recorded gateway no longer runs; its record was removed.
    Forgotten {
        pid: u32,
        liveness: Liveness,
    },
    Pulling {
        image: String,
    },
    Starting {
        release: Release,
        listen: SocketAddr,
    },
    /// Asking the gateway to stop its models.
    Unloading,
    /// SIGTERM sent to the gateway.
    Stopping {
        pid: u32,
    },
    /// The gateway outlived SIGTERM; SIGKILL follows.
    Killing {
        pid: u32,
    },
    /// Waiting for the gateway's processes to leave the GPU.
    Releasing {
        processes: Vec<GpuProcess>,
    },
    /// Something went wrong that does not change the outcome.
    Warning(String),
}

/// What `up` found or did.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Up {
    Started {
        pid: u32,
        release: Release,
        listen: SocketAddr,
        log: PathBuf,
        /// The models the gateway loads now that it runs.
        preload: Vec<String>,
    },
    /// The gateway ran already, on the same configuration.
    Running { pid: u32, listen: SocketAddr },
}

/// What `down` found or did.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Down {
    Stopped {
        pid: u32,
        /// The gateway's processes that held GPU memory, all gone now;
        /// empty when the GPU could not be read.
        released: Vec<GpuProcess>,
    },
    NotRunning,
}

/// The GPU's memory, in MiB.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GpuMemory {
    pub used_mib: u64,
    /// What processes can allocate: the total less what the driver reserves.
    pub usable_mib: u64,
}

/// Whether a running gateway runs on the configuration, address and
/// release `up` would start it with now.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Configuration {
    Current,
    Outdated,
    /// They cannot be worked out, for this reason.
    Unknown(String),
}

/// The gateway `status` found running.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Serving {
    pub pid: u32,
    pub listen: SocketAddr,
    /// The models it runs; `None` when its API did not answer.
    pub models: Option<Vec<RunningModel>>,
    pub configuration: Configuration,
    /// Its processes on the GPU; empty when the GPU cannot be read.
    pub held: Vec<GpuProcess>,
}

impl Serving {
    /// Where the gateway runs: `running on http://127.0.0.1:8080 (PID 1234)`.
    pub fn running_on(&self) -> String {
        format!("running on http://{} (PID {})", self.listen, self.pid)
    }
}

/// `models` for a message: `qwen3.8-27b (ready), …`. Each caller words an
/// empty list, or none, its own way.
pub fn models_list(models: &[RunningModel]) -> String {
    models
        .iter()
        .map(|model| format!("{} ({})", model.id, model.state))
        .collect::<Vec<_>>()
        .join(", ")
}

/// What `status` found.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Status {
    /// `None` when no gateway runs.
    pub gateway: Option<Serving>,
    /// `None` when the GPU cannot be read.
    pub gpu: Option<GpuMemory>,
    /// The processes earlier gateways left on the GPU.
    pub leftovers: Vec<GpuProcess>,
}

/// What `doctor` reads of the gateway, changing nothing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Inspection {
    pub status: Status,
    /// A record whose process is gone: its PID, and what runs under it now.
    pub stale: Option<(u32, Liveness)>,
}

/// How printing the gateway's log ended.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogEnd {
    /// The log is printed, not followed.
    Printed,
    /// Nothing to follow: no gateway runs.
    NoGateway,
    /// The followed gateway stopped; its log is printed to the end.
    Stopped,
    /// A new log replaced the one followed.
    Replaced,
}

/// What `up` starts the gateway with.
struct Target {
    prepared: Prepared,
    /// The configuration file.
    config: String,
    release: Release,
    binary: PathBuf,
}

/// Whom the GPU's processes belong to.
#[derive(Default)]
struct Owners {
    /// The GPU's processes the running gateway holds; empty when none runs.
    gateway: BTreeSet<u32>,
    /// The GPU's processes that run in ai-ctl's containers.
    containers: BTreeSet<u32>,
}

/// The reading of the gateway that `status` and `doctor` share: whether it
/// runs, what it serves and what the GPU holds. It changes nothing and never
/// waits.
pub struct Inspector<'a> {
    pub configurator: Configurator<'a>,
    pub releases: &'a dyn GatewayStore,
    pub gateway: &'a dyn GatewayProcess,
    pub processes: &'a dyn ProcessTable,
    pub api: &'a dyn GatewayApi,
    pub containers: &'a dyn Containers,
}

/// The lifecycle use cases, with the ports they drive.
pub struct Lifecycle<'a> {
    pub configurator: Configurator<'a>,
    pub releases: &'a dyn GatewayStore,
    pub gateway: &'a dyn GatewayProcess,
    pub processes: &'a dyn ProcessTable,
    pub ports: &'a dyn PortCheck,
    pub api: &'a dyn GatewayApi,
    pub containers: &'a dyn Containers,
    /// Waits between two checks of a starting or stopping gateway.
    pub pause: fn(Duration),
    /// The monotonic clock the waits are bounded by.
    pub now: fn() -> Instant,
}

impl Lifecycle<'_> {
    /// Starts the gateway, unless it runs already on the same configuration,
    /// address and release, and answers. A gateway that does not answer is
    /// stopped and its record removed; one that outlives the stop stays
    /// recorded.
    pub fn up(&self, events: &mut dyn FnMut(Event)) -> Result<Up> {
        let _lock = self.gateway.lock("starting the gateway")?;
        let reading = self.configurator.gpu.gpu();
        let target = self.inspector().target(&reading)?;
        let listen = target.prepared.listen;

        if let Some(record) = self.running(events)? {
            ensure!(
                self.inspector().is_current(&record, &target)?,
                "the gateway (PID {}) runs on another configuration, address or release \
                 than the inventory, the builds and llama-swap give now; \
                 run `ai-ctl down`, then `ai-ctl up`",
                record.pid
            );
            // A start whose wait and stop both failed leaves its gateway
            // recorded, and so does an `up` interrupted while it waits.
            let cause = match self.api.healthy(listen, CHECK) {
                Ok(true) => {
                    return Ok(Up::Running {
                        pid: record.pid,
                        listen,
                    });
                }
                Ok(false) => String::new(),
                Err(err) => format!(" ({err:#})"),
            };
            bail!(
                "the gateway (PID {}) runs but does not answer on {listen}{cause}; run \
                 `ai-ctl down`, then `ai-ctl up`{}",
                record.pid,
                self.tail()
            );
        }

        ensure!(
            self.ports.is_free(listen)?,
            "{listen} is taken by another program; {ANOTHER_PROGRAM}"
        );
        // Found: `target` sized the configuration to it.
        let gpu = found_gpu(&reading)?;
        let owners = self.inspector().owners(gpu, None, events);
        let left = self.inspector().leftovers(gpu, &owners, events);
        if !left.is_empty() {
            bail!(
                "processes an earlier gateway left on the GPU hold {} MiB of its memory: {}; \
                 a model started now would not find the memory the inventory gives it, \
                 and ai-ctl never stops a backend itself: end them with `{}`, \
                 then run `ai-ctl up` again",
                mib(&left),
                named(&left),
                kill_command(&left)
            );
        }
        let prepared = &target.prepared;
        for image in &prepared.images {
            if !is_pinned(image) {
                events(Event::Warning(format!(
                    "the image {image} is named by its tag, which its registry can move to other \
                     content; pin it in the inventory as {image}@sha256:<digest>"
                )));
            }
            if !self.containers.image_exists(image)? {
                events(Event::Pulling {
                    image: image.clone(),
                });
                self.containers.pull(image)?;
            }
        }

        events(Event::Starting {
            release: target.release,
            listen,
        });
        let files = target.prepared.config.files();
        let mut started = self
            .gateway
            .start(&target.binary, &target.config, &files, listen)?;
        self.wait_until_healthy(started.as_mut(), listen, events)?;
        Ok(Up::Started {
            pid: started.pid(),
            release: target.release,
            listen,
            log: self.gateway.log(),
            preload: target.prepared.config.preload,
        })
    }

    /// Stops the gateway: its models first, through its API, then the
    /// gateway itself, SIGTERM then SIGKILL; and returns once the processes
    /// it ran on the GPU are gone. ai-ctl never stops a backend itself: a
    /// process still on the GPU after that is an error naming it.
    pub fn down(&self, events: &mut dyn FnMut(Event)) -> Result<Down> {
        let _lock = self.gateway.lock("stopping the gateway")?;
        let Some(record) = self.running(events)? else {
            return Ok(Down::NotRunning);
        };
        let reading = self.configurator.gpu.gpu();
        warn_unread(&reading, "not waiting for the GPU's memory", events);
        let held = reading
            .found()
            .map(|gpu| held(gpu, &self.inspector().owners(gpu, Some(record.pid), events)));

        events(Event::Unloading);
        if let Err(err) = self.api.unload_all(record.listen) {
            events(Event::Warning(format!(
                "cannot unload the models ({err:#}); stopping the gateway stops them"
            )));
        }
        events(Event::Stopping { pid: record.pid });
        self.stop(
            record.pid,
            &mut || Ok(self.gateway.liveness(&record)? != Liveness::Running),
            events,
        )?;
        // The gateway is gone: a record left behind reads as no gateway to
        // the next command, so the wait for the GPU goes on.
        if let Err(err) = self.gateway.forget() {
            events(Event::Warning(format!(
                "the gateway stopped, but its record cannot be removed ({err:#})"
            )));
        }

        let released = match held {
            Some(held) => self.wait_for_release(held, events)?,
            None => Vec::new(),
        };
        Ok(Down::Stopped {
            pid: record.pid,
            released,
        })
    }

    /// Whether the gateway runs, with what it serves, and what the GPU
    /// holds, observed without the lock, so that a `status` never stops an
    /// `up` or a `down`. A record whose process is gone is removed under the
    /// lock, and left to the ai-ctl that holds it.
    pub fn status(&self, events: &mut dyn FnMut(Event)) -> Result<Status> {
        let record = match self.gateway.record()? {
            Some(record) if self.gateway.liveness(&record)? != Liveness::Running => {
                match self
                    .gateway
                    .lock("removing the record of a gateway that stopped")
                {
                    // Read again under the lock: an `up` may have recorded
                    // another gateway since.
                    Ok(_lock) => self.running(events)?,
                    Err(err) => {
                        events(Event::Warning(format!(
                            "the gateway recorded with PID {} no longer runs; its record is \
                             left in place ({err:#})",
                            record.pid
                        )));
                        None
                    }
                }
            }
            record => record,
        };
        let reading = self.configurator.gpu.gpu();
        warn_unread(&reading, "the GPU's memory is left out", events);
        Ok(self.inspector().observe(record, &reading, events))
    }

    /// Copies the gateway's log to `out`; with `follow`, then copies what
    /// the running gateway adds to it, until it stops.
    pub fn log(&self, follow: bool, out: &mut dyn Write) -> Result<LogEnd> {
        let Some(mut offset) = self.copy_log(0, out)? else {
            bail!(
                "no gateway log at {}: the gateway has not run yet; start it with `ai-ctl up`",
                self.gateway.log().display()
            );
        };
        if !follow {
            return Ok(LogEnd::Printed);
        }
        let Some(record) = live_record(self.gateway)? else {
            return Ok(LogEnd::NoGateway);
        };
        loop {
            (self.pause)(POLL);
            // Read after the check, the log holds all the gateway wrote
            // before it stopped.
            let running = self.gateway.liveness(&record)? == Liveness::Running;
            let Some(copied) = self.copy_log(offset, out)? else {
                return Ok(LogEnd::Replaced);
            };
            offset += copied;
            if !running {
                return Ok(LogEnd::Stopped);
            }
        }
    }

    /// Copies to `out` the output of `model` the running gateway kept, then
    /// its live output, until the gateway ends the stream.
    pub fn model_log(&self, model: &str, out: &mut dyn Write) -> Result<()> {
        check_id("model", model)?;
        let Some(record) = live_record(self.gateway)? else {
            bail!(
                "no gateway runs, and a model's output is kept by the gateway while it runs; \
                 `ai-ctl logs` prints the gateway's log, which holds the output of every model \
                 since the last `ai-ctl up`"
            );
        };
        self.api.stream_log(record.listen, model, out)
    }

    /// Copies the log from `offset` to its end; how much it copied, or
    /// `None` when there is no log or another one replaced it.
    fn copy_log(&self, offset: u64, out: &mut dyn Write) -> Result<Option<u64>> {
        let mut copied = 0;
        loop {
            let Some(bytes) = self.gateway.read_log(offset + copied, LOG_CHUNK)? else {
                return Ok(None);
            };
            if bytes.is_empty() {
                return Ok(Some(copied));
            }
            out.write_all(&bytes)
                .and_then(|()| out.flush())
                .context("cannot write the log")?;
            copied += bytes.len() as u64;
        }
    }

    /// The recorded gateway when it runs; a record whose process is gone is
    /// removed, or left with a warning when it cannot be.
    fn running(&self, events: &mut dyn FnMut(Event)) -> Result<Option<GatewayRecord>> {
        let Some(record) = self.gateway.record()? else {
            return Ok(None);
        };
        match self.gateway.liveness(&record)? {
            Liveness::Running => Ok(Some(record)),
            liveness => {
                match self.gateway.forget() {
                    Ok(()) => events(Event::Forgotten {
                        pid: record.pid,
                        liveness,
                    }),
                    Err(err) => events(Event::Warning(format!(
                        "the gateway recorded with PID {} no longer runs, but its record cannot \
                         be removed ({err:#})",
                        record.pid
                    ))),
                }
                Ok(None)
            }
        }
    }

    /// Polls the GPU until none of `held` is on it any more.
    fn wait_for_release(
        &self,
        held: Vec<GpuProcess>,
        events: &mut dyn FnMut(Event),
    ) -> Result<Vec<GpuProcess>> {
        if held.is_empty() {
            return Ok(held);
        }
        events(Event::Releasing {
            processes: held.clone(),
        });
        let deadline = (self.now)() + RELEASE_WAIT;
        let left = loop {
            let reading = self.configurator.gpu.gpu();
            let gpu =
                found_gpu(&reading).context("cannot wait for the GPU's memory to be released")?;
            let left: Vec<GpuProcess> = held
                .iter()
                .filter(|process| gpu.processes.iter().any(|on| on.pid == process.pid))
                .cloned()
                .collect();
            if left.is_empty() {
                return Ok(held);
            }
            match self.time_left(deadline) {
                Some(time_left) => (self.pause)(POLL.min(time_left)),
                None => break left,
            }
        };
        bail!(
            "the gateway stopped, but its processes still hold GPU memory after {} s: {}; \
             ai-ctl never stops a backend itself: end them with `{}`",
            RELEASE_WAIT.as_secs(),
            named(&left),
            kill_command(&left)
        )
    }

    /// Polls the started gateway until it answers, from its own socket: an
    /// answer from another program on the address is not the gateway's. When
    /// it exits first, or never answers or cannot be checked and is stopped,
    /// its record goes and the error quotes its log; a gateway that outlives
    /// the stop stays recorded.
    fn wait_until_healthy(
        &self,
        started: &mut dyn StartedGateway,
        listen: SocketAddr,
        events: &mut dyn FnMut(Event),
    ) -> Result<()> {
        let mut last_error = None;
        let mut unknown = None;
        let mut owner_unknown = None;
        let mut another = false;
        let deadline = (self.now)() + START_WAIT;
        while let Some(time_left) = self.time_left(deadline) {
            match started.exited() {
                Ok(Some(status)) => {
                    self.forget(events);
                    bail!(
                        "llama-swap exited ({status}) before answering on {listen}{}{}",
                        another_answers(another),
                        self.tail()
                    );
                }
                Ok(None) => {}
                Err(err) => {
                    unknown = Some(err);
                    break;
                }
            }
            match self.api.healthy(listen, time_left.min(CHECK)) {
                Ok(true) => match self.ports.listens(started.pid(), listen) {
                    Ok(true) => return Ok(()),
                    Ok(false) => another = true,
                    Err(err) => {
                        owner_unknown = Some(err);
                        break;
                    }
                },
                Ok(false) => {}
                Err(err) => last_error = Some(err),
            }
            if let Some(time_left) = self.time_left(deadline) {
                (self.pause)(POLL.min(time_left));
            }
        }
        let failure = match (unknown, owner_unknown) {
            (Some(err), _) => {
                format!("cannot tell whether llama-swap, started on {listen}, still runs ({err:#})")
            }
            (None, Some(err)) => format!(
                "cannot tell whether the answer on {listen} comes from the llama-swap ai-ctl \
                 started ({err:#})"
            ),
            (None, None) => {
                let cause =
                    last_error.map_or_else(String::new, |err| format!(" (last error: {err:#})"));
                format!(
                    "llama-swap did not answer on {listen} within {} s{cause}{}",
                    START_WAIT.as_secs(),
                    another_answers(another)
                )
            }
        };
        let pid = started.pid();
        let stopped = self
            .stop(pid, &mut || Ok(started.exited()?.is_some()), events)
            .or_else(|err| match started.exited() {
                Ok(Some(_)) => Ok(()),
                _ => Err(err),
            });
        let stopped = match stopped {
            Ok(()) => {
                self.forget(events);
                "stopped it".to_owned()
            }
            Err(err) => format!("could not stop it: {err:#}; it stays recorded for `ai-ctl down`"),
        };
        bail!("{failure}; {stopped}{}", self.tail())
    }

    /// The time left until `deadline`; `None` once it has passed.
    fn time_left(&self, deadline: Instant) -> Option<Duration> {
        deadline
            .checked_duration_since((self.now)())
            .filter(|time_left| !time_left.is_zero())
    }

    /// Stops the gateway `pid`: SIGTERM, then SIGKILL when it outlives
    /// llama-swap's own shutdown; `gone` tells whether it has ended, checked
    /// at once after each signal, then after pauses that grow to `POLL`.
    fn stop(
        &self,
        pid: u32,
        gone: &mut dyn FnMut() -> Result<bool>,
        events: &mut dyn FnMut(Event),
    ) -> Result<()> {
        for (signal, wait) in [(Signal::Terminate, STOP_WAIT), (Signal::Kill, KILL_WAIT)] {
            if signal == Signal::Kill {
                events(Event::Killing { pid });
            }
            self.gateway.signal(pid, signal)?;
            let deadline = (self.now)() + wait;
            let mut poll = FIRST_STOP_POLL;
            loop {
                if gone()? {
                    return Ok(());
                }
                let Some(time_left) = self.time_left(deadline) else {
                    break;
                };
                (self.pause)(poll.min(time_left));
                poll = (poll * 2).min(POLL);
            }
        }
        bail!("PID {pid} still runs after SIGKILL")
    }

    /// Removes the record of a gateway that no longer runs; a failure is
    /// reported, since the error being raised matters more.
    fn forget(&self, events: &mut dyn FnMut(Event)) {
        if let Err(err) = self.gateway.forget() {
            events(Event::Warning(format!("{err:#}")));
        }
    }

    /// The end of the log, for an error message.
    fn tail(&self) -> String {
        let log = self.gateway.log();
        match self.gateway.log_tail(TAIL) {
            Ok(tail) if !tail.trim().is_empty() => {
                format!(
                    "; the last lines of {}:\n{}",
                    log.display(),
                    tail.trim_end()
                )
            }
            Ok(_) => format!("; {} is empty", log.display()),
            Err(err) => format!("; cannot read {}: {err:#}", log.display()),
        }
    }

    /// The inspection on the lifecycle's ports.
    fn inspector(&self) -> Inspector<'_> {
        Inspector {
            configurator: self.configurator,
            releases: self.releases,
            gateway: self.gateway,
            processes: self.processes,
            api: self.api,
            containers: self.containers,
        }
    }
}

impl Inspector<'_> {
    /// What `status` reports, on a reading of the GPU the caller has taken
    /// and reports itself, and without the lock: a stale record is
    /// reported, not removed.
    pub fn inspect(
        &self,
        reading: &Probed<Gpu>,
        events: &mut dyn FnMut(Event),
    ) -> Result<Inspection> {
        let (record, stale) = match self.gateway.record()? {
            Some(record) => match self.gateway.liveness(&record)? {
                Liveness::Running => (Some(record), None),
                liveness => (None, Some((record.pid, liveness))),
            },
            None => (None, None),
        };
        Ok(Inspection {
            status: self.observe(record, reading, events),
            stale,
        })
    }

    /// The gateway of `record`, running, with what it serves, and what the
    /// GPU of `reading` holds.
    fn observe(
        &self,
        record: Option<GatewayRecord>,
        reading: &Probed<Gpu>,
        events: &mut dyn FnMut(Event),
    ) -> Status {
        let gpu = reading.found();
        let (held, leftovers) = match gpu {
            Some(gpu) => {
                let gateway = record.as_ref().map(|record| record.pid);
                let owners = self.owners(gpu, gateway, events);
                (held(gpu, &owners), self.leftovers(gpu, &owners, events))
            }
            None => (Vec::new(), Vec::new()),
        };
        let gateway = record.map(|record| {
            let models = self
                .api
                .running(record.listen)
                .map_err(|err| {
                    events(Event::Warning(format!(
                        "cannot list the gateway's models ({err:#})"
                    )));
                })
                .ok();
            let configuration = match self
                .target(reading)
                .and_then(|target| self.is_current(&record, &target))
            {
                Ok(true) => Configuration::Current,
                Ok(false) => Configuration::Outdated,
                Err(err) => Configuration::Unknown(format!("{err:#}")),
            };
            Serving {
                pid: record.pid,
                listen: record.listen,
                models,
                configuration,
                held,
            }
        });
        Status {
            gateway,
            gpu: gpu.map(|gpu| GpuMemory {
                used_mib: gpu.memory_used >> 20,
                usable_mib: gpu.usable_mib(),
            }),
            leftovers,
        }
    }

    /// What `up` would start the gateway with now, on the GPU of `reading`.
    fn target(&self, reading: &Probed<Gpu>) -> Result<Target> {
        let prepared = self.configurator.prepare_on(reading)?;
        let release = self
            .releases
            .current()?
            .ok_or_else(|| anyhow!("llama-swap is not installed; run `ai-ctl gateway install`"))?;
        ensure!(
            release >= OLDEST,
            "llama-swap {release} cannot read the configuration this ai-ctl writes, which needs \
             {OLDEST} or later; run `ai-ctl gateway install`"
        );
        // A release runs only once an ai-ctl checked it against the digest
        // it pins for it: an older ai-ctl's `--version` installed one against
        // its release's own checksums or a digest the user gave, and
        // recorded nothing. Only the pinned release's record can be checked
        // here; another one's was written by an ai-ctl that pins it.
        let record = self.releases.digest(release)?;
        if release == PINNED.release {
            ensure!(
                record == Some(Sha256::parse(PINNED.sha256)?),
                "llama-swap {release} was never checked against the digest this ai-ctl pins for \
                 it; run `ai-ctl gateway install`"
            );
        } else {
            ensure!(
                record.is_some(),
                "llama-swap {release} was never checked against a digest an ai-ctl pins for it; \
                 run `ai-ctl gateway install`"
            );
        }
        Ok(Target {
            config: prepared.config.to_yaml(),
            prepared,
            release,
            binary: self.releases.binary(release),
        })
    }

    /// Whether the gateway of `record` runs on `target`, from the binary as
    /// it is now: `gateway install` replaces a release's binary when it
    /// checks it again, and the kernel then names the one a gateway still
    /// runs `<path> (deleted)`.
    fn is_current(&self, record: &GatewayRecord, target: &Target) -> Result<bool> {
        let replaced = self.processes.program(record.pid)?.is_some_and(|program| {
            program
                .as_os_str()
                .as_encoded_bytes()
                .ends_with(b" (deleted)")
        });
        Ok(record.binary == target.binary
            && !replaced
            && record.listen == target.prepared.listen
            && self.gateway.config()?.as_deref() == Some(target.config.as_str()))
    }

    /// Which of the GPU's processes the gateway `gateway` holds — its
    /// descendants, and the processes of ai-ctl's containers, which run
    /// under conmon — and which run in ai-ctl's containers. Each process is
    /// placed by walking up its parents; podman is asked only when one may
    /// run in a container. What cannot be read is left out, with a warning.
    fn owners(&self, gpu: &Gpu, gateway: Option<u32>, events: &mut dyn FnMut(Event)) -> Owners {
        let mut owners = Owners::default();
        let mut roots: Option<Vec<u32>> = None;
        for process in &gpu.processes {
            let chain = self.ancestry(process.pid, events);
            let reaches = |pids: &[u32]| chain.iter().any(|(pid, _)| pids.contains(pid));
            if gateway.is_some_and(|gateway| reaches(&[gateway])) {
                owners.gateway.insert(process.pid);
                continue;
            }
            let candidate = chain
                .iter()
                .any(|(_, name)| name.as_deref() == Some("conmon"))
                || self.in_container(process.pid, events);
            if !candidate {
                continue;
            }
            let roots = roots.get_or_insert_with(|| {
                self.containers
                    .pids(CONTAINER_PREFIX)
                    .unwrap_or_else(|err| {
                        events(Event::Warning(format!(
                            "cannot list ai-ctl's containers ({err:#}); leaving their processes out"
                        )));
                        Vec::new()
                    })
            });
            if reaches(roots) {
                owners.containers.insert(process.pid);
                if gateway.is_some() {
                    owners.gateway.insert(process.pid);
                }
            }
        }
        owners
    }

    /// The process `pid` and its ancestors below PID 1, each with its
    /// program name when it can be read. A process whose parent cannot be
    /// read ends the walk, with a warning when it is not gone, and so does a
    /// PID met twice.
    fn ancestry(&self, pid: u32, events: &mut dyn FnMut(Event)) -> Vec<(u32, Option<String>)> {
        let mut chain: Vec<(u32, Option<String>)> = Vec::new();
        let mut current = pid;
        while current > 1 && !chain.iter().any(|(seen, _)| *seen == current) {
            match self.processes.stat(current) {
                Ok(Some(stat)) => {
                    chain.push((current, Some(stat.name)));
                    current = stat.parent;
                }
                Ok(None) => {
                    chain.push((current, None));
                    break;
                }
                Err(err) => {
                    chain.push((current, None));
                    events(Event::Warning(format!(
                        "cannot read the parent of PID {current} ({err:#}); leaving the \
                         processes above it out"
                    )));
                    break;
                }
            }
        }
        chain
    }

    /// Whether the process `pid` runs in a podman container; `false`, with a
    /// warning, when its cgroup cannot be read.
    fn in_container(&self, pid: u32, events: &mut dyn FnMut(Event)) -> bool {
        self.processes.in_container(pid).unwrap_or_else(|err| {
            events(Event::Warning(format!(
                "cannot read the cgroup of PID {pid} ({err:#}); leaving it out of ai-ctl's \
                 containers"
            )));
            false
        })
    }

    /// The processes on the GPU that a gateway left behind: the programs of
    /// the build store the running gateway does not own, and, while no
    /// gateway runs, the processes of ai-ctl's containers.
    fn leftovers(
        &self,
        gpu: &Gpu,
        owners: &Owners,
        events: &mut dyn FnMut(Event),
    ) -> Vec<GpuProcess> {
        let mut from_a_build = |pid: u32| match self.processes.program(pid) {
            Ok(program) => {
                program.is_some_and(|program| self.configurator.builds.contains(&program))
            }
            Err(err) => {
                events(Event::Warning(format!(
                    "cannot read the program of PID {pid} ({err:#}); leaving it out"
                )));
                false
            }
        };
        gpu.processes
            .iter()
            .filter(|process| {
                !owners.gateway.contains(&process.pid)
                    && (owners.containers.contains(&process.pid) || from_a_build(process.pid))
            })
            .cloned()
            .collect()
    }
}

/// The recorded gateway when it runs; a record whose process is gone is
/// left as it is.
pub fn live_record(gateway: &dyn GatewayProcess) -> Result<Option<GatewayRecord>> {
    Ok(match gateway.record()? {
        Some(record) if gateway.liveness(&record)? == Liveness::Running => Some(record),
        _ => None,
    })
}

/// Warns, the warning opening with `consequence`, when `reading` holds no
/// GPU.
fn warn_unread(reading: &Probed<Gpu>, consequence: &str, events: &mut dyn FnMut(Event)) {
    if let Err(err) = found_gpu(reading) {
        events(Event::Warning(format!("{consequence}: {err:#}")));
    }
}

/// What a failed start adds when another program answered on the address
/// while llama-swap did not.
fn another_answers(another: bool) -> String {
    if another {
        format!(", where another program answers: {ANOTHER_PROGRAM}")
    } else {
        String::new()
    }
}

/// The processes on the GPU of `gpu` that `owners` gives the gateway.
fn held(gpu: &Gpu, owners: &Owners) -> Vec<GpuProcess> {
    gpu.processes
        .iter()
        .filter(|process| owners.gateway.contains(&process.pid))
        .cloned()
        .collect()
}

/// The GPU memory `processes` hold, in MiB, as far as the driver reports it.
pub fn mib(processes: &[GpuProcess]) -> u64 {
    processes
        .iter()
        .filter_map(|process| process.memory)
        .sum::<u64>()
        >> 20
}

/// `processes` named for a message, `PID 5000 (llama-server, 21094 MiB), …`,
/// with the memory each holds when the driver reports it.
pub fn named(processes: &[GpuProcess]) -> String {
    processes
        .iter()
        .map(|process| match process.memory {
            Some(bytes) => format!(
                "PID {} ({}, {} MiB)",
                process.pid,
                process.name,
                bytes >> 20
            ),
            None => format!("PID {} ({})", process.pid, process.name),
        })
        .collect::<Vec<_>>()
        .join(", ")
}

/// The command that ends `processes`, `kill 5000 6000`, which ai-ctl never
/// runs itself.
pub fn kill_command(processes: &[GpuProcess]) -> String {
    let pids: Vec<String> = processes
        .iter()
        .map(|process| process.pid.to_string())
        .collect();
    format!("kill {}", pids.join(" "))
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::path::Path;

    use super::*;
    use crate::application::fakes::{
        ClosedPipe, FakeApi, FakeBuilds, FakeContainers, FakeGateway, FakeInventory, FakePorts,
        FakeProcesses, FakeReleases, Logged, OTHER_DIGEST, SWAP_BINARY, ScriptedGpu, advance,
        broken_pipe, gpu, gpu_running, inventory, inventory_parts, listen, now, record,
    };
    use crate::domain::build::{Build, BuildId};
    use crate::domain::inventory::{
        AudiocppFields, Backend, BackendKind, BackendSource, Inventory, Model,
    };

    struct World {
        inventory: FakeInventory,
        builds: FakeBuilds,
        gpu: ScriptedGpu,
        releases: FakeReleases,
        gateway: FakeGateway,
        processes: FakeProcesses,
        ports: FakePorts,
        api: FakeApi,
        containers: FakeContainers,
    }

    impl World {
        fn new() -> Self {
            Self {
                inventory: FakeInventory(Ok(Some(inventory("/models")))),
                builds: FakeBuilds::current_b11538(),
                gpu: ScriptedGpu::new(vec![Probed::Found(gpu())]),
                releases: FakeReleases::current("v271"),
                gateway: FakeGateway::default(),
                processes: FakeProcesses::default(),
                ports: FakePorts::default(),
                api: FakeApi::answering_after(2),
                containers: FakeContainers::default(),
            }
        }

        fn lifecycle(&self) -> Lifecycle<'_> {
            Lifecycle {
                configurator: Configurator {
                    inventory: &self.inventory,
                    builds: &self.builds,
                    gpu: &self.gpu,
                    files_dir: std::path::Path::new(crate::application::fakes::GATEWAY_DIR),
                },
                releases: &self.releases,
                gateway: &self.gateway,
                processes: &self.processes,
                ports: &self.ports,
                api: &self.api,
                containers: &self.containers,
                pause: advance,
                now,
            }
        }

        fn up(&self) -> (Result<Up>, Vec<Event>) {
            let mut events = Vec::new();
            let result = self.lifecycle().up(&mut |event| events.push(event));
            (result, events)
        }

        fn down(&self) -> (Result<Down>, Vec<Event>) {
            let mut events = Vec::new();
            let result = self.lifecycle().down(&mut |event| events.push(event));
            (result, events)
        }

        fn status(&self) -> (Result<Status>, Vec<Event>) {
            let mut events = Vec::new();
            let result = self.lifecycle().status(&mut |event| events.push(event));
            (result, events)
        }

        /// What `log` printed, and how it ended.
        fn log(&self, follow: bool) -> (Result<LogEnd>, String) {
            let mut out = Vec::new();
            let result = self.lifecycle().log(follow, &mut out);
            (result, String::from_utf8(out).unwrap())
        }

        /// The configuration `up` writes.
        fn config(&self) -> String {
            Configurator {
                inventory: &self.inventory,
                builds: &self.builds,
                gpu: &self.gpu,
                files_dir: std::path::Path::new(crate::application::fakes::GATEWAY_DIR),
            }
            .generate()
            .unwrap()
            .to_yaml()
        }
    }

    fn release() -> Release {
        Release::parse("v271").unwrap()
    }

    fn message<T: std::fmt::Debug>(result: Result<T>) -> String {
        format!("{:#}", result.unwrap_err())
    }

    #[test]
    fn up_starts_the_gateway_and_returns_once_it_answers() {
        let world = World::new();
        let (result, events) = world.up();
        assert_eq!(
            result.unwrap(),
            Up::Started {
                pid: 4242,
                release: release(),
                listen: listen(),
                log: "/state/logs/gateway.log".into(),
                preload: vec!["qwen3.8-27b".into()],
            }
        );
        assert_eq!(
            events,
            [Event::Starting {
                release: release(),
                listen: listen(),
            }]
        );
        assert_eq!(world.api.calls.get(), 3);
        assert_eq!(
            world.gateway.config.borrow().as_deref(),
            Some(world.config().as_str())
        );
        assert_eq!(world.gateway.record.borrow().as_ref(), Some(&record(4242)));
    }

    #[test]
    fn up_writes_the_files_its_models_read() {
        let mut world = World::new();
        let mut parts = inventory_parts("/models");
        parts.backends.push(Backend {
            id: "audio-native".into(),
            kind: BackendKind::AudiocppServer,
            source: BackendSource::Build("audio.cpp".into()),
        });
        parts.models.push(Model {
            id: "qwen3-tts".into(),
            backend: "audio-native".into(),
            repo: None,
            dir: "tts/qwen3-tts-1.7b-base".into(),
            files: BTreeMap::new(),
            flags: BTreeMap::new(),
            ctx_size: None,
            vram_mib: 6200,
            ttl: None,
            preload: false,
            audiocpp: AudiocppFields {
                family: Some("qwen3_tts".into()),
                ..AudiocppFields::default()
            },
        });
        world.inventory = FakeInventory(Ok(Some(Inventory::new(parts).unwrap())));
        let v0_9_3 = BuildId::parse("v0.9.3").unwrap();
        world.builds.builds.push(Build {
            project: "audio.cpp".into(),
            cmake_flags: crate::application::fakes::recipe_flags(&crate::domain::recipe::AUDIO_CPP),
            id: v0_9_3.clone(),
            patches: vec!["api-key.patch".into()],
            ..crate::application::fakes::b11538()
        });
        world
            .builds
            .other_currents
            .insert("audio.cpp".into(), v0_9_3);

        let (result, _) = world.up();
        assert!(result.is_ok(), "{result:?}");
        let files = world.gateway.files.borrow();
        let paths: Vec<&Path> = files.iter().map(|file| file.path.as_path()).collect();
        assert_eq!(paths, [Path::new("/state/gateway/audiocpp/qwen3-tts.json")]);
        assert!(files[0].content.contains("\"family\": \"qwen3_tts\""));
    }

    #[test]
    fn up_needs_an_installed_llama_swap() {
        let mut world = World::new();
        world.releases = FakeReleases::none();
        assert_eq!(
            message(world.up().0),
            "llama-swap is not installed; run `ai-ctl gateway install`"
        );
        assert_eq!(world.gateway.started.get(), 0);
    }

    #[test]
    fn up_needs_a_llama_swap_that_reads_the_capacity_router() {
        let mut world = World::new();
        world.releases = FakeReleases::current("v266");
        assert_eq!(
            message(world.up().0),
            "llama-swap v266 cannot read the configuration this ai-ctl writes, which needs v267 \
             or later; run `ai-ctl gateway install`"
        );
        assert_eq!(world.gateway.started.get(), 0);

        // Behind the pin or ahead of it, a release that reads the
        // configuration, and whose check an ai-ctl recorded, runs: `doctor`
        // names the difference.
        for release in ["v267", "v300"] {
            let mut world = World::new();
            world.releases = FakeReleases::current(release);
            world.up().0.unwrap();
            assert_eq!(world.gateway.started.get(), 1, "{release}");
        }
    }

    #[test]
    fn up_refuses_a_release_off_the_pin_that_no_ai_ctl_checked_against_its_pin() {
        // An older ai-ctl's `--version` installed it against its release's
        // own checksums, or a digest the user gave, and recorded nothing.
        for release in ["v267", "v270", "v272", "v300"] {
            let mut world = World::new();
            world.releases = FakeReleases::current(release);
            world.releases.record = None;
            assert_eq!(
                message(world.up().0),
                format!(
                    "llama-swap {release} was never checked against a digest an ai-ctl pins \
                     for it; run `ai-ctl gateway install`"
                ),
            );
            assert_eq!(world.gateway.started.get(), 0, "{release}");
        }
        // Installed by an ai-ctl that pins it, which recorded its check.
        let mut world = World::new();
        world.releases = FakeReleases::current("v272");
        world.releases.record = Some(OTHER_DIGEST);
        world.up().0.unwrap();
        assert_eq!(world.gateway.started.get(), 1);
    }

    #[test]
    fn up_refuses_a_release_whose_record_cannot_be_read() {
        for release in ["v270", "v271", "v272"] {
            let mut world = World::new();
            world.releases = FakeReleases::current(release);
            world.releases.digest_fails = true;
            assert_eq!(
                message(world.up().0),
                format!(
                    "cannot read /data/llama-swap/{release}/archive.sha256: Permission denied \
                     (os error 13)"
                ),
            );
            assert_eq!(world.gateway.started.get(), 0, "{release}");
        }
    }

    #[test]
    fn up_refuses_the_pinned_release_never_checked_against_its_pin() {
        for record in [None, Some(OTHER_DIGEST)] {
            let mut world = World::new();
            world.releases.record = record;
            assert_eq!(
                message(world.up().0),
                "llama-swap v271 was never checked against the digest this ai-ctl pins for it; \
                 run `ai-ctl gateway install`",
                "{record:?}"
            );
            assert_eq!(world.gateway.started.get(), 0);
        }
    }

    #[test]
    fn up_needs_an_inventory() {
        let mut world = World::new();
        world.inventory = FakeInventory(Ok(None));
        assert!(message(world.up().0).starts_with("no inventory at "));
        assert_eq!(world.gateway.started.get(), 0);
    }

    #[test]
    fn a_gateway_running_on_the_same_configuration_is_left_alone() {
        let mut world = World::new();
        world.api = FakeApi::answering_after(0);
        *world.gateway.record.borrow_mut() = Some(record(1234));
        *world.gateway.config.borrow_mut() = Some(world.config());
        let (result, events) = world.up();
        assert_eq!(
            result.unwrap(),
            Up::Running {
                pid: 1234,
                listen: listen(),
            }
        );
        assert!(events.is_empty());
        assert_eq!(world.api.calls.get(), 1, "one health check");
        assert_eq!(world.gateway.started.get(), 0);
    }

    #[test]
    fn a_gateway_whose_binary_was_replaced_since_it_started_is_not_current() {
        // `gateway install` checked its release again and replaced its
        // directory: the gateway still runs the binary it started from.
        let mut world = World::new();
        world.api = FakeApi::answering_after(0);
        *world.gateway.record.borrow_mut() = Some(record(1234));
        *world.gateway.config.borrow_mut() = Some(world.config());
        world
            .processes
            .programs
            .insert(1234, format!("{SWAP_BINARY} (deleted)").into());
        assert_eq!(
            message(world.up().0),
            "the gateway (PID 1234) runs on another configuration, address or release than the \
             inventory, the builds and llama-swap give now; run `ai-ctl down`, then `ai-ctl up`"
        );
        assert_eq!(world.gateway.started.get(), 0);

        let mut world = serving(0);
        *world.gateway.config.borrow_mut() = Some(world.config());
        world
            .processes
            .programs
            .insert(1234, format!("{SWAP_BINARY} (deleted)").into());
        let gateway = world.status().0.unwrap().gateway.unwrap();
        assert_eq!(gateway.configuration, Configuration::Outdated);
        world.processes.programs.insert(1234, SWAP_BINARY.into());
        let gateway = world.status().0.unwrap().gateway.unwrap();
        assert_eq!(gateway.configuration, Configuration::Current);
        // A program that cannot be read leaves it unknown.
        world.processes.program_fails = true;
        let gateway = world.status().0.unwrap().gateway.unwrap();
        assert_eq!(
            gateway.configuration,
            Configuration::Unknown(
                "cannot read /proc/1234/exe: Too many open files (os error 24)".to_owned()
            )
        );
    }

    #[test]
    fn a_recorded_gateway_that_does_not_answer_is_not_up_already() {
        let mut world = World::new();
        world.api = FakeApi::silent();
        world.gateway.tail = "ERROR models failed to start\n".into();
        *world.gateway.record.borrow_mut() = Some(record(1234));
        *world.gateway.config.borrow_mut() = Some(world.config());
        assert_eq!(
            message(world.up().0),
            "the gateway (PID 1234) runs but does not answer on 127.0.0.1:8080; run `ai-ctl \
             down`, then `ai-ctl up`; the last lines of /state/logs/gateway.log:\n\
             ERROR models failed to start"
        );
        assert_eq!(world.api.calls.get(), 1, "one health check");

        world.api.healthy_fails = true;
        let text = message(world.up().0);
        assert!(
            text.starts_with(
                "the gateway (PID 1234) runs but does not answer on 127.0.0.1:8080 (GET \
                 http://127.0.0.1:8080/health: connection reset by peer); run `ai-ctl down`"
            ),
            "{text}"
        );
        assert_eq!(world.gateway.started.get(), 0);
        assert!(world.gateway.signals.borrow().is_empty(), "left running");
        assert_eq!(world.gateway.record.borrow().as_ref(), Some(&record(1234)));
    }

    #[test]
    fn a_gateway_running_on_another_configuration_is_not_restarted() {
        let cases: [fn(&World); 3] = [
            |world| *world.gateway.config.borrow_mut() = Some("models: {}\n".into()),
            |world| {
                world.gateway.record.borrow_mut().as_mut().unwrap().listen =
                    "127.0.0.1:8081".parse().unwrap();
            },
            |world| {
                world.gateway.record.borrow_mut().as_mut().unwrap().binary =
                    "/data/llama-swap/v262/llama-swap".into();
            },
        ];
        for change in cases {
            let world = World::new();
            *world.gateway.record.borrow_mut() = Some(record(1234));
            *world.gateway.config.borrow_mut() = Some(world.config());
            change(&world);
            assert_eq!(
                message(world.up().0),
                "the gateway (PID 1234) runs on another configuration, address or release than \
                 the inventory, the builds and llama-swap give now; \
                 run `ai-ctl down`, then `ai-ctl up`"
            );
            assert_eq!(world.gateway.started.get(), 0);
            assert!(world.gateway.signals.borrow().is_empty());
        }
    }

    #[test]
    fn a_stale_record_is_removed_before_starting() {
        for liveness in [
            Liveness::Exited,
            Liveness::Reused(Some("/usr/bin/bash".into())),
            Liveness::Reused(None),
        ] {
            let mut world = World::new();
            *world.gateway.record.borrow_mut() = Some(record(1234));
            world.gateway.liveness = Some(liveness.clone());
            let (result, events) = world.up();
            assert!(matches!(result.unwrap(), Up::Started { pid: 4242, .. }));
            assert_eq!(
                events[0],
                Event::Forgotten {
                    pid: 1234,
                    liveness
                }
            );
            assert_eq!(world.gateway.started.get(), 1);
        }
    }

    #[test]
    fn up_refuses_a_taken_address() {
        let mut world = World::new();
        world.ports.taken = true;
        assert_eq!(
            message(world.up().0),
            "127.0.0.1:8080 is taken by another program; stop it, or give the gateway another \
             `listen` address in the inventory. A program you do not know may have taken the \
             API key from a client while the gateway was down: change the key, as \
             docs/security.md §3 says"
        );
        assert_eq!(world.gateway.started.get(), 0);
    }

    #[test]
    fn up_refuses_an_address_it_cannot_bind() {
        let mut world = World::new();
        world.ports.is_free_fails = true;
        assert_eq!(
            message(world.up().0),
            "cannot bind 127.0.0.1:8080: Cannot assign requested address (os error 99)"
        );
        assert_eq!(world.gateway.started.get(), 0);
    }

    /// The official CUDA 13 image the bench compared, pinned by digest.
    const PINNED_IMAGE: &str = "ghcr.io/ggml-org/llama.cpp:server-cuda13-b11541@sha256:42e80867b03cc22e8cf744c07e0b3194f078838ab6c476095ecdc1621271ec95";

    /// A world whose model runs on [`PINNED_IMAGE`], `images` in local
    /// storage.
    fn on_images(images: &[&str]) -> World {
        on_image(PINNED_IMAGE, images)
    }

    /// A world whose model runs on `image`, `images` in local storage.
    fn on_image(image: &str, images: &[&str]) -> World {
        let mut parts = inventory_parts("/models");
        parts.backends.push(Backend {
            id: "llama-cuda13".into(),
            kind: BackendKind::LlamaServer,
            source: BackendSource::Image(image.into()),
        });
        parts.models[0].backend = "llama-cuda13".into();
        let mut world = World::new();
        world.inventory = FakeInventory(Ok(Some(Inventory::new(parts).unwrap())));
        world.containers.local = images.iter().map(|&image| image.to_owned()).collect();
        world
    }

    #[test]
    fn up_pulls_a_missing_image_before_starting() {
        let world = on_images(&[]);
        let (result, events) = world.up();
        result.unwrap();
        assert_eq!(world.containers.pulled.borrow().as_slice(), [PINNED_IMAGE]);
        assert_eq!(
            events[0],
            Event::Pulling {
                image: PINNED_IMAGE.into()
            }
        );
    }

    #[test]
    fn up_leaves_a_local_image_alone() {
        let world = on_images(&[PINNED_IMAGE]);
        world.up().0.unwrap();
        assert!(world.containers.pulled.borrow().is_empty());
    }

    #[test]
    fn up_warns_of_an_image_named_by_its_tag_only() {
        let tagged = "ghcr.io/ggml-org/llama.cpp:server-cuda13";
        let world = on_image(tagged, &[tagged]);
        let (result, events) = world.up();
        result.unwrap();
        assert_eq!(
            events[0],
            Event::Warning(format!(
                "the image {tagged} is named by its tag, which its registry can move to other \
                 content; pin it in the inventory as {tagged}@sha256:<digest>"
            ))
        );

        let (result, events) = on_images(&[PINNED_IMAGE]).up();
        result.unwrap();
        assert!(
            !events
                .iter()
                .any(|event| matches!(event, Event::Warning(_))),
            "{events:?}"
        );
    }

    #[test]
    fn up_starts_nothing_when_an_image_cannot_be_pulled() {
        let mut world = on_images(&[]);
        world.containers.pull_fails = true;
        let (result, events) = world.up();
        assert_eq!(
            message(result),
            format!("podman pull {PINNED_IMAGE}: manifest unknown")
        );
        assert_eq!(
            events,
            [Event::Pulling {
                image: PINNED_IMAGE.into()
            }]
        );
        assert_eq!(world.gateway.started.get(), 0);
        assert_eq!(world.gateway.record.borrow().as_ref(), None);
    }

    #[test]
    fn up_starts_nothing_when_an_image_cannot_be_looked_for() {
        let mut world = on_images(&[PINNED_IMAGE]);
        world.containers.image_exists_fails = true;
        assert_eq!(
            message(world.up().0),
            format!("podman image exists {PINNED_IMAGE}: cannot connect to the storage")
        );
        assert!(world.containers.pulled.borrow().is_empty());
        assert_eq!(world.gateway.started.get(), 0);
    }

    #[test]
    fn a_gateway_exiting_before_it_answers_is_forgotten_and_its_log_quoted() {
        let mut world = World::new();
        world.api = FakeApi::silent();
        world.gateway.exits_after = Some(3);
        world.gateway.tail = "ERROR http server error: bind: address already in use\n".into();
        let message = message(world.up().0);
        assert_eq!(
            message,
            "llama-swap exited (exit status: 1) before answering on 127.0.0.1:8080; \
             the last lines of /state/logs/gateway.log:\n\
             ERROR http server error: bind: address already in use"
        );
        assert_eq!(world.gateway.record.borrow().as_ref(), None);
        assert!(world.gateway.signals.borrow().is_empty());
    }

    #[test]
    fn an_answer_from_another_program_is_not_taken_for_the_gateway() {
        let mut world = World::new();
        world.ports.held_by_another = true;
        world.gateway.exits_after = Some(3);
        world.gateway.tail = "ERROR http server error: bind: address already in use\n".into();
        assert_eq!(
            message(world.up().0),
            "llama-swap exited (exit status: 1) before answering on 127.0.0.1:8080, where \
             another program answers: stop it, or give the gateway another `listen` address \
             in the inventory. A program you do not know may have taken the API key from a \
             client while the gateway was down: change the key, as docs/security.md §3 says; \
             the last lines of /state/logs/gateway.log:\n\
             ERROR http server error: bind: address already in use"
        );
        assert_eq!(world.gateway.record.borrow().as_ref(), None);
    }

    #[test]
    fn a_start_timeout_names_another_program_that_answers() {
        let mut world = World::new();
        world.ports.held_by_another = true;
        assert_eq!(
            message(world.up().0),
            "llama-swap did not answer on 127.0.0.1:8080 within 30 s, where another program \
             answers: stop it, or give the gateway another `listen` address in the inventory. \
             A program you do not know may have taken the API key from a client while the \
             gateway was down: change the key, as docs/security.md §3 says; stopped it; \
             /state/logs/gateway.log is empty"
        );
    }

    #[test]
    fn a_gateway_whose_listener_cannot_be_checked_is_stopped() {
        let mut world = World::new();
        world.ports.listens_fails = true;
        assert_eq!(
            message(world.up().0),
            "cannot tell whether the answer on 127.0.0.1:8080 comes from the llama-swap \
             ai-ctl started (cannot read /proc/net/tcp: Permission denied (os error 13)); \
             stopped it; /state/logs/gateway.log is empty"
        );
        assert_eq!(
            world.gateway.signals.borrow().as_slice(),
            [Signal::Terminate]
        );
        assert_eq!(world.gateway.record.borrow().as_ref(), None);
    }

    #[test]
    fn a_gateway_that_never_answers_is_stopped_and_forgotten() {
        let mut world = World::new();
        world.api = FakeApi::silent();
        let message = message(world.up().0);
        assert_eq!(
            message,
            "llama-swap did not answer on 127.0.0.1:8080 within 30 s; stopped it; \
             /state/logs/gateway.log is empty"
        );
        assert_eq!(
            world.gateway.signals.borrow().as_slice(),
            [Signal::Terminate]
        );
        assert_eq!(world.gateway.record.borrow().as_ref(), None);
        // One check every 100 ms for 30 s.
        assert_eq!(world.api.calls.get(), 300);
    }

    #[test]
    fn a_start_timeout_gives_the_health_checks_last_error() {
        let mut world = World::new();
        world.api = FakeApi::silent();
        world.api.healthy_fails = true;
        assert_eq!(
            message(world.up().0),
            "llama-swap did not answer on 127.0.0.1:8080 within 30 s (last error: \
             GET http://127.0.0.1:8080/health: connection reset by peer); stopped it; \
             /state/logs/gateway.log is empty"
        );
    }

    #[test]
    fn a_gateway_that_accepts_and_never_answers_holds_up_30_s() {
        let mut world = World::new();
        world.api = FakeApi::silent();
        world.api.hangs = true;
        let start = now();
        let message = message(world.up().0);
        assert_eq!(now() - start, Duration::from_secs(30));
        assert_eq!(
            message,
            "llama-swap did not answer on 127.0.0.1:8080 within 30 s (last error: \
             GET http://127.0.0.1:8080/health: no answer within 500 ms); \
             stopped it; /state/logs/gateway.log is empty"
        );
    }

    #[test]
    fn a_start_never_waits_past_its_30_s() {
        // The deadline falls inside a check that hangs, then between two
        // checks that fail at once.
        for (first, hangs) in [
            (Duration::from_millis(250), true),
            (Duration::from_millis(30), false),
        ] {
            let mut world = World::new();
            world.api = FakeApi::silent();
            world.api.first_check = Some(first);
            world.api.hangs = hangs;
            let start = now();
            let message = message(world.up().0);
            assert_eq!(now() - start, Duration::from_secs(30), "{first:?}");
            assert!(
                message.starts_with("llama-swap did not answer"),
                "{message}"
            );
        }
    }

    #[test]
    fn a_log_that_cannot_be_read_is_named_in_the_error() {
        let mut world = World::new();
        world.api = FakeApi::silent();
        world.gateway.log_tail_fails = true;
        assert_eq!(
            message(world.up().0),
            "llama-swap did not answer on 127.0.0.1:8080 within 30 s; stopped it; \
             cannot read /state/logs/gateway.log: Permission denied (os error 13)"
        );
    }

    #[test]
    fn a_record_that_cannot_be_removed_is_a_warning_beside_the_error() {
        let mut world = World::new();
        world.api = FakeApi::silent();
        world.gateway.exits_after = Some(3);
        world.gateway.forget_fails = true;
        let (result, events) = world.up();
        assert_eq!(
            message(result),
            "llama-swap exited (exit status: 1) before answering on 127.0.0.1:8080; \
             /state/logs/gateway.log is empty"
        );
        assert_eq!(
            events.last(),
            Some(&Event::Warning(
                "cannot remove /state/gateway/gateway.json: Read-only file system (os error 30)"
                    .into()
            ))
        );
        assert_eq!(world.gateway.record.borrow().as_ref(), Some(&record(4242)));
    }

    #[test]
    fn a_gateway_up_cannot_stop_stays_recorded() {
        let mut world = World::new();
        world.api = FakeApi::silent();
        world.gateway.immortal = true;
        let message = message(world.up().0);
        assert_eq!(
            world
                .gateway
                .record
                .borrow()
                .as_ref()
                .map(|record| record.pid),
            Some(4242)
        );
        assert_eq!(
            message,
            "llama-swap did not answer on 127.0.0.1:8080 within 30 s; could not stop it: \
             PID 4242 still runs after SIGKILL; it stays recorded for `ai-ctl down`; \
             /state/logs/gateway.log is empty"
        );
        assert_eq!(
            world.gateway.signals.borrow().as_slice(),
            [Signal::Terminate, Signal::Kill]
        );
    }

    #[test]
    fn a_gateway_whose_exit_cannot_be_read_is_stopped_and_forgotten() {
        // Once: the check fails while the gateway starts. Twice: it fails
        // again while it stops, and the check after tells it has exited.
        for failures in [1, 2] {
            let mut world = World::new();
            world.api = FakeApi::silent();
            world.gateway.wait_failures.set(failures);
            let message = message(world.up().0);
            assert_eq!(
                message,
                "cannot tell whether llama-swap, started on 127.0.0.1:8080, still runs \
                 (cannot wait for llama-swap: No child processes (os error 10)); stopped it; \
                 /state/logs/gateway.log is empty",
                "{failures} failures"
            );
            assert_eq!(
                world.gateway.signals.borrow().as_slice(),
                [Signal::Terminate]
            );
            assert_eq!(world.gateway.record.borrow().as_ref(), None);
        }
    }

    #[test]
    fn a_gateway_that_outlives_sigterm_is_killed() {
        let mut world = World::new();
        world.api = FakeApi::silent();
        world.gateway.stubborn = true;
        assert!(message(world.up().0).contains("; stopped it;"));
        assert_eq!(
            world.gateway.signals.borrow().as_slice(),
            [Signal::Terminate, Signal::Kill]
        );
    }

    /// A world whose gateway, PID 1234, runs the model PID 5000, with Xorg
    /// beside it on the GPU; the model leaves after `stays` more readings.
    fn serving(stays: usize) -> World {
        let mut world = World::new();
        *world.gateway.record.borrow_mut() = Some(record(1234));
        world.processes.spawned(&[
            (777, "Xorg", 1),
            (5000, "llama-server", 1234),
            (1234, "llama-swap", USER_SYSTEMD),
        ]);
        let both = gpu_running(&[(777, "Xorg", 50), (5000, "llama-server", 21094)]);
        let mut readings = vec![both.clone(); stays + 1];
        readings.push(gpu_running(&[(777, "Xorg", 50)]));
        world.gpu = ScriptedGpu::new(readings);
        world
    }

    fn llama_server(pid: u32) -> GpuProcess {
        GpuProcess {
            pid,
            name: "llama-server".into(),
            memory: Some(21094 << 20),
        }
    }

    #[test]
    fn gpu_processes_are_named_with_their_memory_and_a_kill_command() {
        let processes = [
            llama_server(5000),
            GpuProcess {
                pid: 6000,
                name: "llama-server".into(),
                memory: None,
            },
        ];
        assert_eq!(
            named(&processes),
            "PID 5000 (llama-server, 21094 MiB), PID 6000 (llama-server)"
        );
        assert_eq!(kill_command(&processes), "kill 5000 6000");
    }

    #[test]
    fn a_serving_gateway_is_described_by_its_address_and_models() {
        let serving = Serving {
            pid: 1234,
            listen: listen(),
            models: None,
            configuration: Configuration::Current,
            held: vec![],
        };
        assert_eq!(
            serving.running_on(),
            "running on http://127.0.0.1:8080 (PID 1234)"
        );
        let model = |id: &str, state: &str| RunningModel {
            id: id.into(),
            state: state.into(),
            proxy: None,
        };
        assert_eq!(
            models_list(&[
                model("qwen3.8-27b", "ready"),
                model("gemma-4-31b", "starting")
            ]),
            "qwen3.8-27b (ready), gemma-4-31b (starting)"
        );
    }

    #[test]
    fn down_unloads_stops_and_waits_for_the_gateways_gpu_memory() {
        let world = serving(2);
        let (result, events) = world.down();
        assert_eq!(
            result.unwrap(),
            Down::Stopped {
                pid: 1234,
                released: vec![llama_server(5000)],
            }
        );
        assert_eq!(
            events,
            [
                Event::Unloading,
                Event::Stopping { pid: 1234 },
                Event::Releasing {
                    processes: vec![llama_server(5000)]
                },
            ]
        );
        assert_eq!(world.api.unloads.get(), 1);
        assert_eq!(
            world.gateway.signals.borrow().as_slice(),
            [Signal::Terminate]
        );
        assert_eq!(world.gateway.record.borrow().as_ref(), None);
        // One reading before the stop, then until the model is gone.
        assert_eq!(world.gpu.reads.get(), 4);
    }

    #[test]
    fn down_waits_for_the_processes_of_the_gateways_containers() {
        let mut world = World::new();
        *world.gateway.record.borrow_mut() = Some(record(1234));
        world.containers.running = vec![6000];
        world
            .processes
            .spawned(&[(6000, "llama-server", 5990), (5990, "conmon", USER_SYSTEMD)]);
        world.gpu = ScriptedGpu::new(vec![
            gpu_running(&[(777, "Xorg", 50), (6000, "llama-server", 21094)]),
            gpu_running(&[(777, "Xorg", 50)]),
        ]);
        let (result, _) = world.down();
        assert_eq!(
            result.unwrap(),
            Down::Stopped {
                pid: 1234,
                released: vec![llama_server(6000)],
            }
        );
    }

    #[test]
    fn down_without_a_gateway_does_nothing() {
        let world = World::new();
        let (result, events) = world.down();
        assert_eq!(result.unwrap(), Down::NotRunning);
        assert!(events.is_empty());
        assert_eq!(world.api.unloads.get(), 0);
        assert!(world.gateway.signals.borrow().is_empty());
    }

    #[test]
    fn down_removes_a_stale_record_and_signals_nothing() {
        let mut world = World::new();
        *world.gateway.record.borrow_mut() = Some(record(1234));
        world.gateway.liveness = Some(Liveness::Reused(Some("/usr/bin/bash".into())));
        let (result, events) = world.down();
        assert_eq!(result.unwrap(), Down::NotRunning);
        assert_eq!(
            events,
            [Event::Forgotten {
                pid: 1234,
                liveness: Liveness::Reused(Some("/usr/bin/bash".into())),
            }]
        );
        assert!(world.gateway.signals.borrow().is_empty());
        assert_eq!(world.gateway.record.borrow().as_ref(), None);
    }

    #[test]
    fn a_failed_unload_still_stops_the_gateway() {
        let mut world = serving(0);
        world.api.unload_fails = true;
        let (result, events) = world.down();
        assert!(matches!(result.unwrap(), Down::Stopped { .. }));
        assert_eq!(
            events[1],
            Event::Warning(
                "cannot unload the models (connection reset); stopping the gateway stops them"
                    .into()
            )
        );
        assert_eq!(
            world.gateway.signals.borrow().as_slice(),
            [Signal::Terminate]
        );
    }

    #[test]
    fn a_gateway_that_outlives_sigterm_is_killed_by_down() {
        let mut world = serving(0);
        world.gateway.stubborn = true;
        let start = now();
        let (result, events) = world.down();
        assert!(matches!(result.unwrap(), Down::Stopped { .. }));
        // SIGKILL comes no sooner than llama-swap's own stop ends.
        assert!(now() - start >= Duration::from_secs(35));
        assert!(events.contains(&Event::Killing { pid: 1234 }));
        assert_eq!(
            world.gateway.signals.borrow().as_slice(),
            [Signal::Terminate, Signal::Kill]
        );
    }

    #[test]
    fn a_gateway_that_outlives_sigkill_keeps_its_record() {
        let mut world = serving(0);
        world.gateway.immortal = true;
        assert_eq!(message(world.down().0), "PID 1234 still runs after SIGKILL");
        assert_eq!(world.gateway.record.borrow().as_ref(), Some(&record(1234)));
    }

    #[test]
    fn a_stopping_gateway_is_checked_soon_then_every_100_ms() {
        let world = serving(0);
        // llama-swap ends a millisecond or two after SIGTERM: the checks
        // come 5, 10, 20, 40 and 80 ms apart, then every 100 ms.
        world.gateway.lingers.set(7);
        let start = now();
        assert!(matches!(world.down().0.unwrap(), Down::Stopped { .. }));
        assert_eq!(now() - start, Duration::from_millis(355));
    }

    #[test]
    fn the_waits_after_each_signal_end_on_their_deadlines() {
        let mut world = serving(0);
        world.gateway.immortal = true;
        let start = now();
        assert_eq!(message(world.down().0), "PID 1234 still runs after SIGKILL");
        // 40 s after SIGTERM and 5 s after SIGKILL, though the checks'
        // pauses divide neither.
        assert_eq!(now() - start, Duration::from_secs(45));
    }

    #[test]
    fn a_signal_that_cannot_be_sent_ends_down_at_once() {
        let mut world = serving(0);
        world.gateway.signal_fails = true;
        let start = now();
        assert_eq!(
            message(world.down().0),
            "kill -s TERM 1234: kill: sending signal to 1234 failed: Operation not permitted"
        );
        assert_eq!(now() - start, Duration::ZERO);
        assert_eq!(world.gateway.record.borrow().as_ref(), Some(&record(1234)));
    }

    #[test]
    fn a_process_left_on_the_gpu_is_named_not_killed() {
        let world = serving(1000);
        assert_eq!(
            message(world.down().0),
            "the gateway stopped, but its processes still hold GPU memory after 30 s: \
             PID 5000 (llama-server, 21094 MiB); ai-ctl never stops a backend itself: end them \
             with `kill 5000`"
        );
        assert_eq!(
            world.gateway.signals.borrow().as_slice(),
            [Signal::Terminate]
        );
        assert_eq!(world.gateway.record.borrow().as_ref(), None);
    }

    #[test]
    fn a_slow_gpu_reading_does_not_stretch_the_release_wait() {
        let mut world = serving(1000);
        world.gpu.read_time = Duration::from_millis(28);
        let start = now();
        let message = message(world.down().0);
        // `down`'s reading before the wait, the wait, and its last reading,
        // which starts at the deadline.
        assert_eq!(now() - start, Duration::from_millis(28 + 30_000 + 28));
        assert!(message.contains("after 30 s"), "{message}");
    }

    #[test]
    fn down_stops_the_gateway_even_when_the_gpu_cannot_be_read() {
        let mut world = serving(0);
        world.gpu = ScriptedGpu::new(vec![Probed::Failed("NVML: driver not loaded".into())]);
        let (result, events) = world.down();
        assert_eq!(
            result.unwrap(),
            Down::Stopped {
                pid: 1234,
                released: vec![],
            }
        );
        assert_eq!(
            events[0],
            Event::Warning(
                "not waiting for the GPU's memory: cannot read the GPU: NVML: driver not loaded"
                    .into()
            )
        );
    }

    #[test]
    fn a_record_down_cannot_remove_is_a_warning_once_the_gateway_stopped() {
        let mut world = serving(0);
        world.gateway.forget_fails = true;
        let (result, events) = world.down();
        assert_eq!(
            result.map_err(|err| format!("{err:#}")),
            Ok(Down::Stopped {
                pid: 1234,
                released: vec![llama_server(5000)],
            })
        );
        assert!(
            events.contains(&Event::Warning(
                "the gateway stopped, but its record cannot be removed (cannot remove \
                 /state/gateway/gateway.json: Read-only file system (os error 30))"
                    .into()
            )),
            "{events:?}"
        );
        assert_eq!(world.gateway.record.borrow().as_ref(), Some(&record(1234)));
    }

    #[test]
    fn a_stale_record_that_cannot_be_removed_reads_as_no_gateway() {
        let mut world = World::new();
        *world.gateway.record.borrow_mut() = Some(record(1234));
        world.gateway.liveness = Some(Liveness::Exited);
        world.gateway.forget_fails = true;
        let (result, events) = world.status();
        assert_eq!(
            result
                .map(|status| status.gateway)
                .map_err(|err| format!("{err:#}")),
            Ok(None)
        );
        assert_eq!(
            events,
            [Event::Warning(
                "the gateway recorded with PID 1234 no longer runs, but its record cannot be \
                 removed (cannot remove /state/gateway/gateway.json: Read-only file system \
                 (os error 30))"
                    .into()
            )]
        );
    }

    #[test]
    fn down_fails_when_the_gpu_is_lost_while_its_memory_is_released() {
        let both = gpu_running(&[(777, "Xorg", 50), (5000, "llama-server", 21094)]);
        for (lost, expected) in [
            (
                Probed::Missing,
                "cannot wait for the GPU's memory to be released: no NVIDIA GPU found; \
                 run `ai-ctl doctor`",
            ),
            (
                Probed::Failed("NVML: GPU is lost".into()),
                "cannot wait for the GPU's memory to be released: cannot read the GPU: \
                 NVML: GPU is lost",
            ),
        ] {
            let mut world = serving(0);
            world.gpu = ScriptedGpu::new(vec![both.clone(), lost]);
            assert_eq!(message(world.down().0), expected);
            assert_eq!(
                world.gateway.signals.borrow().as_slice(),
                [Signal::Terminate]
            );
            assert_eq!(world.gateway.record.borrow().as_ref(), None);
        }
    }

    #[test]
    fn containers_that_cannot_be_listed_are_left_out_with_a_warning() {
        let mut world = serving(0);
        world.containers.pids_fails = true;
        // Another container's process, under its conmon.
        world
            .processes
            .spawned(&[(6000, "llama-server", 5990), (5990, "conmon", USER_SYSTEMD)]);
        let other = (6000, "llama-server", 21094);
        world.gpu = ScriptedGpu::new(vec![
            gpu_running(&[(777, "Xorg", 50), (5000, "llama-server", 21094), other]),
            gpu_running(&[(777, "Xorg", 50), other]),
        ]);
        let (result, events) = world.down();
        assert_eq!(
            result.unwrap(),
            Down::Stopped {
                pid: 1234,
                released: vec![llama_server(5000)],
            }
        );
        assert_eq!(
            events[0],
            Event::Warning(
                "cannot list ai-ctl's containers (podman ps --filter name=ai-ctl-: cannot \
                 connect to the storage); leaving their processes out"
                    .into()
            )
        );
    }

    #[test]
    fn parents_that_cannot_be_read_are_left_out_with_a_warning() {
        let mut world = serving(0);
        world.processes.stat_fails = true;
        let (result, events) = world.down();
        assert_eq!(
            result.unwrap(),
            Down::Stopped {
                pid: 1234,
                released: vec![],
            }
        );
        let warning = |pid: u32| {
            Event::Warning(format!(
                "cannot read the parent of PID {pid} (cannot read /proc/{pid}/stat: Too many \
                 open files (os error 24)); leaving the processes above it out"
            ))
        };
        assert_eq!(events[..2], [warning(777), warning(5000)]);
    }

    #[test]
    fn a_cgroup_that_cannot_be_read_is_left_out_with_a_warning() {
        let mut world = left_behind();
        world.processes.in_container_fails = true;
        let (result, events) = world.status();
        // The container's process is known by its conmon.
        assert_eq!(
            result.unwrap().leftovers,
            [llama_server(5000), llama_server(6000)]
        );
        let warning = |pid: u32| {
            Event::Warning(format!(
                "cannot read the cgroup of PID {pid} (cannot read /proc/{pid}/cgroup: Too many \
                 open files (os error 24)); leaving it out of ai-ctl's containers"
            ))
        };
        let warnings: Vec<Event> = events
            .into_iter()
            .filter(|event| matches!(event, Event::Warning(_)))
            .collect();
        assert_eq!(warnings, [warning(777), warning(5000)]);
    }

    #[test]
    fn an_idle_gpu_asks_podman_nothing() {
        let mut world = World::new();
        world
            .processes
            .spawned(&[(777, "gnome-shell", USER_SYSTEMD)]);
        world.gpu = ScriptedGpu::new(vec![gpu_running(&[(777, "gnome-shell", 50)])]);
        let status = world.status().0.unwrap();
        assert!(status.leftovers.is_empty());
        assert_eq!(world.containers.listed.get(), 0);

        // Nor beside a gateway whose processes are all its own.
        let world = serving(0);
        world.down().0.unwrap();
        assert_eq!(world.containers.listed.get(), 0);
    }

    #[test]
    fn a_container_whose_conmon_is_gone_is_found_by_its_cgroup() {
        let mut world = left_behind();
        world.processes.spawned(&[(6000, "llama-server", 1)]);
        world.processes.contained = vec![6000];
        let status = world.status().0.unwrap();
        assert_eq!(status.leftovers, [llama_server(5000), llama_server(6000)]);
    }

    #[test]
    fn another_containers_process_is_not_ai_ctls() {
        let mut world = left_behind();
        world.containers.running = vec![7000];
        let status = world.status().0.unwrap();
        assert_eq!(status.leftovers, [llama_server(5000)]);
        assert_eq!(world.containers.listed.get(), 1);
    }

    #[test]
    fn a_loop_of_parents_ends_the_walk() {
        let mut world = serving(0);
        world
            .processes
            .spawned(&[(5000, "llama-server", 5001), (5001, "loop", 5000)]);
        let held = world.status().0.unwrap().gateway.unwrap().held;
        assert!(held.is_empty());
    }

    #[test]
    fn the_walk_ends_below_pid_1() {
        let mut world = World::new();
        world
            .processes
            .spawned(&[(5000, "llama-server", 1), (1, "systemd", 0)]);
        let chain = world.lifecycle().inspector().ancestry(5000, &mut |_| {});
        assert_eq!(chain, [(5000, Some("llama-server".into()))]);
    }

    /// `systemd --user`, under which a killed gateway's processes and the
    /// containers' conmon run.
    const USER_SYSTEMD: u32 = 4571;

    /// A native model of the build store.
    const BUILD: &str = "/data/builds/llama.cpp/b11538/llama-server";

    fn model(id: &str, state: &str) -> RunningModel {
        RunningModel {
            id: id.into(),
            state: state.into(),
            proxy: None,
        }
    }

    #[test]
    fn status_reports_the_gateway_its_models_and_its_gpu_memory() {
        // Generating the configuration below reads the GPU once.
        let mut world = serving(1);
        *world.gateway.config.borrow_mut() = Some(world.config());
        world.api.models = Some(vec![model("qwen3.8-27b", "ready")]);
        let (result, events) = world.status();
        assert_eq!(
            result.unwrap(),
            Status {
                gateway: Some(Serving {
                    pid: 1234,
                    listen: listen(),
                    models: Some(vec![model("qwen3.8-27b", "ready")]),
                    configuration: Configuration::Current,
                    held: vec![llama_server(5000)],
                }),
                gpu: Some(GpuMemory {
                    used_mib: 16,
                    usable_mib: 24075,
                }),
                leftovers: vec![],
            }
        );
        assert!(events.is_empty());
        // Once for the configuration above, once for `status`.
        assert_eq!(world.gpu.reads.get(), 2);
        assert!(world.gateway.signals.borrow().is_empty());
    }

    #[test]
    fn status_reports_a_gateway_killed_outside_ai_ctl_as_stopped_and_removes_its_record() {
        let mut world = World::new();
        *world.gateway.record.borrow_mut() = Some(record(1234));
        world.gateway.liveness = Some(Liveness::Exited);
        let (result, events) = world.status();
        let status = result.unwrap();
        assert_eq!(status.gateway, None);
        assert_eq!(
            events,
            [Event::Forgotten {
                pid: 1234,
                liveness: Liveness::Exited,
            }]
        );
        assert_eq!(world.gateway.record.borrow().as_ref(), None);
        assert_eq!(world.api.calls.get(), 0);
    }

    #[test]
    fn up_and_down_wait_for_no_one() {
        let mut world = World::new();
        world.gateway.locked = true;
        assert_eq!(
            message(world.up().0),
            "another ai-ctl is stopping the gateway"
        );
        assert_eq!(world.gateway.started.get(), 0);
        let mut world = serving(0);
        world.gateway.locked = true;
        assert_eq!(
            message(world.down().0),
            "another ai-ctl is stopping the gateway"
        );
        assert!(world.gateway.signals.borrow().is_empty());
        assert_eq!(world.api.unloads.get(), 0);
    }

    #[test]
    fn each_holder_of_the_lock_says_what_it_does() {
        let world = World::new();
        world.up().0.unwrap();
        assert_eq!(*world.gateway.locks.borrow(), ["starting the gateway"]);
        let world = serving(0);
        world.down().0.unwrap();
        assert_eq!(*world.gateway.locks.borrow(), ["stopping the gateway"]);
        let mut world = World::new();
        *world.gateway.record.borrow_mut() = Some(record(1234));
        world.gateway.liveness = Some(Liveness::Exited);
        world.status().0.unwrap();
        assert_eq!(
            *world.gateway.locks.borrow(),
            ["removing the record of a gateway that stopped"]
        );
    }

    #[test]
    fn status_observes_while_another_ai_ctl_holds_the_lock() {
        let mut world = serving(0);
        world.gateway.locked = true;
        let (result, _) = world.status();
        assert_eq!(result.unwrap().gateway.unwrap().pid, 1234);
    }

    #[test]
    fn status_leaves_a_stale_record_to_the_ai_ctl_that_holds_the_lock() {
        let mut world = World::new();
        world.gateway.locked = true;
        *world.gateway.record.borrow_mut() = Some(record(1234));
        world.gateway.liveness = Some(Liveness::Exited);
        let (result, events) = world.status();
        assert_eq!(result.unwrap().gateway, None);
        assert_eq!(
            events,
            [Event::Warning(
                "the gateway recorded with PID 1234 no longer runs; its record is left in place \
                 (another ai-ctl is stopping the gateway)"
                    .into()
            )]
        );
        assert_eq!(world.gateway.record.borrow().as_ref(), Some(&record(1234)));
    }

    #[test]
    fn inspecting_reports_a_stale_record_and_leaves_it() {
        let mut world = World::new();
        *world.gateway.record.borrow_mut() = Some(record(1234));
        world.gateway.liveness = Some(Liveness::Reused(Some("/usr/bin/bash".into())));
        let mut events = Vec::new();
        let inspection = world
            .lifecycle()
            .inspector()
            .inspect(&Probed::Found(gpu()), &mut |event| events.push(event))
            .unwrap();
        assert_eq!(inspection.status.gateway, None);
        assert_eq!(
            inspection.stale,
            Some((1234, Liveness::Reused(Some("/usr/bin/bash".into()))))
        );
        assert!(events.is_empty());
        assert_eq!(world.gateway.record.borrow().as_ref(), Some(&record(1234)));
    }

    #[test]
    fn status_tells_a_gateway_running_on_an_older_configuration() {
        let world = serving(0);
        *world.gateway.config.borrow_mut() = Some("models: {}\n".into());
        let gateway = world.status().0.unwrap().gateway.unwrap();
        assert_eq!(gateway.configuration, Configuration::Outdated);
    }

    #[test]
    fn status_without_the_gateways_api_or_the_gpu_still_reports_it_running() {
        let mut world = serving(0);
        world.api.models = None;
        world.gpu = ScriptedGpu::new(vec![Probed::Failed("NVML: driver not loaded".into())]);
        let (result, events) = world.status();
        let status = result.unwrap();
        let gateway = status.gateway.unwrap();
        assert_eq!((gateway.pid, gateway.models), (1234, None));
        assert!(gateway.held.is_empty());
        assert_eq!(
            gateway.configuration,
            Configuration::Unknown(
                "cannot size the models to the GPU's memory: cannot read the GPU: \
                 NVML: driver not loaded"
                    .into()
            )
        );
        assert_eq!(status.gpu, None);
        assert_eq!(
            events,
            [
                Event::Warning(
                    "the GPU's memory is left out: cannot read the GPU: NVML: driver not loaded"
                        .into()
                ),
                Event::Warning(
                    "cannot list the gateway's models \
                     (GET http://127.0.0.1:8080/running: timed out)"
                        .into()
                ),
            ]
        );
    }

    /// No gateway runs; on the GPU, Xorg, a native model and a container's
    /// process a killed gateway left.
    fn left_behind() -> World {
        let mut world = World::new();
        world.processes.programs = BTreeMap::from([
            (777, "/usr/bin/Xorg".into()),
            (5000, BUILD.into()),
            (6000, "/app/llama-server".into()),
        ]);
        world.containers.running = vec![6000];
        world.processes.spawned(&[
            (777, "Xorg", 1),
            (5000, "llama-server", USER_SYSTEMD),
            (6000, "llama-server", 5990),
            (5990, "conmon", USER_SYSTEMD),
        ]);
        world.gpu = ScriptedGpu::new(vec![gpu_running(&[
            (777, "Xorg", 50),
            (5000, "llama-server", 21094),
            (6000, "llama-server", 21094),
        ])]);
        world
    }

    #[test]
    fn status_lists_the_processes_a_killed_gateway_left_on_the_gpu() {
        let world = left_behind();
        let status = world.status().0.unwrap();
        assert_eq!(status.gateway, None);
        assert_eq!(status.leftovers, [llama_server(5000), llama_server(6000)]);
    }

    /// A server of each kind on the GPU, beside Xorg, each run by `parent`
    /// from its project's build; the servers as the GPU reports them.
    fn servers_of_every_kind(world: &mut World, parent: u32) -> Vec<GpuProcess> {
        let servers = [
            (5000, "llama-server", BUILD, 9120),
            (
                5001,
                "whisper-server",
                "/data/builds/whisper.cpp/v1.9.5/whisper-server",
                2252,
            ),
            (
                5002,
                "sd-server",
                "/data/builds/stable-diffusion.cpp/master-1024-77b72a2/sd-server",
                8400,
            ),
            (
                5003,
                "audiocpp_server",
                "/data/builds/audio.cpp/v0.9.3/audiocpp_server",
                4700,
            ),
        ];
        let mut on_gpu = vec![(777, "Xorg", 50)];
        world.processes.spawned(&[(777, "Xorg", 1)]);
        for (pid, name, program, mib) in servers {
            world.processes.spawned(&[(pid, name, parent)]);
            world.processes.programs.insert(pid, program.into());
            on_gpu.push((pid, name, mib));
        }
        let reading = gpu_running(&on_gpu);
        let processes = reading.found().unwrap().processes[1..].to_vec();
        world.gpu = ScriptedGpu::new(vec![reading]);
        processes
    }

    #[test]
    fn status_counts_a_server_of_every_kind_among_the_gateways_processes() {
        let mut world = serving(0);
        let servers = servers_of_every_kind(&mut world, 1234);
        let models = vec![
            model("qwen3.6-35b-a3b", "ready"),
            model("whisper-large-v3-turbo", "ready"),
            model("flux-3-image-klein", "starting"),
            model("qwen3-tts", "ready"),
        ];
        world.api.models = Some(models.clone());
        let status = world.status().0.unwrap();
        let gateway = status.gateway.unwrap();
        assert_eq!(gateway.models, Some(models));
        assert_eq!(gateway.held, servers);
        assert!(status.leftovers.is_empty());
    }

    #[test]
    fn status_lists_a_server_of_every_kind_a_killed_gateway_left() {
        let mut world = World::new();
        let servers = servers_of_every_kind(&mut world, USER_SYSTEMD);
        let status = world.status().0.unwrap();
        assert_eq!(status.gateway, None);
        assert_eq!(status.leftovers, servers);
    }

    #[test]
    fn a_program_that_cannot_be_read_is_left_out_with_a_warning() {
        let mut world = left_behind();
        world.processes.program_fails = true;
        let (result, events) = world.status();
        // The container's process is known without its program.
        assert_eq!(result.unwrap().leftovers, [llama_server(6000)]);
        let warning = |pid: u32| {
            Event::Warning(format!(
                "cannot read the program of PID {pid} (cannot read /proc/{pid}/exe: \
                 Too many open files (os error 24)); leaving it out"
            ))
        };
        let warnings: Vec<Event> = events
            .into_iter()
            .filter(|event| matches!(event, Event::Warning(_)))
            .collect();
        assert_eq!(warnings, [warning(777), warning(5000)]);
    }

    #[test]
    fn a_removed_build_still_running_is_left_behind_too() {
        let mut world = left_behind();
        world
            .processes
            .programs
            .insert(5000, format!("{BUILD} (deleted)").into());
        assert!(
            world
                .status()
                .0
                .unwrap()
                .leftovers
                .contains(&llama_server(5000))
        );
    }

    #[test]
    fn beside_a_running_gateway_only_another_gateways_build_is_left_behind() {
        let mut world = left_behind();
        *world.gateway.record.borrow_mut() = Some(record(1234));
        world
            .processes
            .spawned(&[(5000, "llama-server", 1234), (5100, "llama-server", 1)]);
        world.processes.programs.insert(5100, BUILD.into());
        world.gpu = ScriptedGpu::new(vec![gpu_running(&[
            (5000, "llama-server", 21094),
            (5100, "llama-server", 21094),
            (6000, "llama-server", 21094),
        ])]);
        let status = world.status().0.unwrap();
        assert_eq!(status.leftovers, [llama_server(5100)]);
        assert_eq!(
            status.gateway.unwrap().held,
            [llama_server(5000), llama_server(6000)]
        );
    }

    #[test]
    fn up_refuses_to_start_beside_processes_a_killed_gateway_left() {
        let world = left_behind();
        assert_eq!(
            message(world.up().0),
            "processes an earlier gateway left on the GPU hold 42188 MiB of its memory: \
             PID 5000 (llama-server, 21094 MiB), PID 6000 (llama-server, 21094 MiB); a model \
             started now would not \
             find the memory the inventory gives it, and ai-ctl never stops a backend itself: \
             end them with `kill 5000 6000`, then run `ai-ctl up` again"
        );
        assert_eq!(world.gateway.started.get(), 0);
    }

    #[test]
    fn the_log_is_printed_whole() {
        let world = World::new();
        *world.gateway.log.borrow_mut() = Some(b"[INFO] llama-swap listening\n".to_vec());
        let (result, out) = world.log(false);
        assert_eq!(result.unwrap(), LogEnd::Printed);
        assert_eq!(out, "[INFO] llama-swap listening\n");
    }

    #[test]
    fn a_log_longer_than_a_chunk_is_printed_whole() {
        let world = World::new();
        let long: Vec<u8> = (0..LOG_CHUNK + 10).map(|i| b'a' + (i % 26) as u8).collect();
        *world.gateway.log.borrow_mut() = Some(long.clone());
        let (result, out) = world.log(false);
        assert_eq!(result.unwrap(), LogEnd::Printed);
        assert_eq!(out.into_bytes(), long);
    }

    #[test]
    fn no_log_is_an_error_naming_up() {
        assert_eq!(
            message(World::new().log(false).0),
            "no gateway log at /state/logs/gateway.log: the gateway has not run yet; \
             start it with `ai-ctl up`"
        );
    }

    #[test]
    fn following_without_a_gateway_prints_the_log_and_ends() {
        let world = World::new();
        *world.gateway.log.borrow_mut() = Some(b"[INFO] shutdown complete\n".to_vec());
        let (result, out) = world.log(true);
        assert_eq!(result.unwrap(), LogEnd::NoGateway);
        assert_eq!(out, "[INFO] shutdown complete\n");
    }

    fn writing(changes: Vec<Logged>) -> World {
        let mut world = World::new();
        *world.gateway.record.borrow_mut() = Some(record(1234));
        *world.gateway.log.borrow_mut() = Some(b"one\n".to_vec());
        world.gateway.writes = true;
        *world.gateway.writing.borrow_mut() = changes.into();
        world
    }

    #[test]
    fn following_prints_what_the_gateway_writes_until_it_stops() {
        let world = writing(vec![
            Logged::Appends("two\n"),
            Logged::Appends("three\n"),
            Logged::Appends("[INFO] shutdown complete\n"),
        ]);
        let (result, out) = world.log(true);
        assert_eq!(result.unwrap(), LogEnd::Stopped);
        assert_eq!(out, "one\ntwo\nthree\n[INFO] shutdown complete\n");
    }

    #[test]
    fn following_ends_when_a_new_log_replaces_the_old_one() {
        let world = writing(vec![
            Logged::Appends("two\n"),
            Logged::Appends("three\n"),
            Logged::Replaces("x\n"),
            Logged::Appends("never read\n"),
        ]);
        let (result, out) = world.log(true);
        assert_eq!(result.unwrap(), LogEnd::Replaced);
        assert_eq!(out, "one\ntwo\nthree\n");
    }

    #[test]
    fn a_log_whose_reader_has_gone_stops_at_once() {
        let world = writing(vec![Logged::Appends("two\n"), Logged::Appends("three\n")]);
        let error = world.lifecycle().log(true, &mut ClosedPipe).unwrap_err();
        assert_eq!(error.to_string(), "cannot write the log");
        assert!(broken_pipe(&error));
        // Nothing was followed.
        assert_eq!(world.gateway.writing.borrow().len(), 2);
    }

    #[test]
    fn a_models_log_streams_from_the_running_gateway() {
        let world = World::new();
        *world.gateway.record.borrow_mut() = Some(record(1234));
        let mut out = Vec::new();
        world
            .lifecycle()
            .model_log("qwen3.8-27b", &mut out)
            .unwrap();
        assert_eq!(out, b"srv  load_model: loading model\n");
        assert_eq!(
            world.api.streamed.borrow().as_slice(),
            [(listen(), "qwen3.8-27b".to_owned())]
        );
    }

    #[test]
    fn a_models_log_needs_a_running_gateway_and_a_model_id() {
        let world = World::new();
        *world.gateway.record.borrow_mut() = Some(record(1234));
        let mut out = Vec::new();
        assert_eq!(
            message(world.lifecycle().model_log("../health", &mut out)),
            "model id `../health` must start with a letter or a digit and hold only ASCII \
             letters, digits, `.`, `_` and `-`"
        );
        let mut world = World::new();
        *world.gateway.record.borrow_mut() = Some(record(1234));
        world.gateway.liveness = Some(Liveness::Exited);
        assert!(
            message(world.lifecycle().model_log("qwen3.8-27b", &mut out))
                .starts_with("no gateway runs, and a model's output is kept by the gateway")
        );
        assert!(world.api.streamed.borrow().is_empty());
        assert!(out.is_empty());
    }
}
