//! `bench`: measures a model served through the gateway, with the protocol
//! of `docs/stack.md` §9.
//!
//! The prompts are an agent's, from [`crate::domain::prompt`], fitted to an
//! exact number of tokens as the model's own template and tokenizer count
//! them. A run sends them scenario by scenario through the running gateway,
//! which loads the model when it must; it starts nothing itself.

use std::cell::Cell;
use std::fmt;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::time::Duration;

use anyhow::{Context, Result, anyhow, ensure};

use crate::application::gateway_config::Configurator;
use crate::application::lifecycle::live_record;
use crate::application::ports::{
    ChatClient, ChatRequest, Containers, GatewayApi, GatewayProcess, GpuMonitor, ModelServer,
    Monitoring, PowerProbe, Reply, RunStore,
};
use crate::domain::bench::{
    Depth, DepthResult, DraftCheck, Drafts, Machine, Message, OVERHEAD_GENERATE, OverheadResult,
    PREFILL_GENERATE, Plan, PrefillResult, Run, Sample, Scenario, SessionResult, Setup, Source,
    SpecCounters,
};
use crate::domain::inventory::{BackendKind, BackendSource, check_id};
use crate::domain::machine::Gpu;
use crate::domain::prompt::{self, CORPUS, Corpus};
use crate::domain::timestamp::Timestamp;

/// Tokens the warm-up request generates: a few decoding steps, drafts
/// included.
const WARMUP_GENERATE: u32 = 16;
/// The request before a fresh one sends the same prompt cut to this
/// fraction: a quarter is more than the 10 % a prompt must share with a
/// slot's for llama-server to pick that slot by similarity
/// (`--slot-prompt-similarity`), and the fresh prompt keeps all of it, more
/// than the half under which llama-server saves the slot's context.
const LEAD_FRACTION: usize = 4;
/// How often the GPU is read during a run: a decoding step takes tens of
/// milliseconds, a change of temperature or clock several seconds.
const GPU_PERIOD: Duration = Duration::from_millis(250);
/// The sampling seed of every request, so two runs sample alike.
pub const SEED: u32 = 42;

/// What `bench run` measures.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Options {
    /// The model's ID, as the gateway serves it.
    pub model: String,
    /// How many times each measure is made.
    pub repeat: u32,
    /// The scenarios to run, which run in the order of [`Scenario::ALL`].
    pub scenarios: Vec<Scenario>,
}

/// A request of a run.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Step {
    /// The first request, on a loaded model: not measured.
    WarmUp,
    /// The request that fills the context at a depth.
    Fill(Depth),
    /// A repetition on the filled context, from 1.
    Generate { depth: Depth, repetition: u32 },
    /// A fresh prompt, from 1.
    Prefill { repetition: u32 },
    /// A turn of a session, both from 1.
    Turn { session: u32, turn: u32 },
    /// The request before a fresh one, or before the overhead: not
    /// measured. Given a prompt that keeps less than half of its slot's
    /// context, llama-server first saves that context into its RAM prompt
    /// cache, unless it is there already, and may load a cached one in its
    /// place (`get_available_slot` in `tools/server/server-context.cpp`).
    /// This request pays that work, and leaves in the slot a prompt the
    /// next request keeps whole, so that a scenario's first repetition is
    /// measured as its others.
    SetAside,
    /// A request of the overhead scenario, from 1, through the gateway or
    /// straight to llama-server.
    Overhead { request: u32, direct: bool },
}

impl Step {
    /// Whether the request must miss the prompt cache: it evaluates its
    /// prompt whole, as the first request of an agent.
    pub fn fresh(&self) -> bool {
        matches!(
            self,
            Self::Fill(_) | Self::Prefill { .. } | Self::Turn { turn: 1, .. }
        )
    }
}

impl fmt::Display for Step {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::WarmUp => f.write_str("the warm-up request"),
            Self::Fill(depth) => write!(f, "the fill at {} tokens", depth.tokens),
            Self::Generate { depth, repetition } => {
                write!(f, "repetition {repetition} at {} tokens", depth.tokens)
            }
            Self::Prefill { repetition } => write!(f, "prefill {repetition}"),
            Self::Turn { session, turn } => write!(f, "turn {turn} of session {session}"),
            Self::SetAside => f.write_str("the request that sets the context aside"),
            Self::Overhead { request, direct } => {
                let way = if *direct {
                    "straight to llama-server"
                } else {
                    "through the gateway"
                };
                write!(f, "overhead request {request} {way}")
            }
        }
    }
}

/// A step of a run, for the user to follow.
#[derive(Debug, Clone, PartialEq)]
pub enum Event {
    /// Reading the model's context, which the gateway loads first when it
    /// must.
    Loading { model: String },
    /// What the model's context gives the run.
    Planned { n_ctx: u32, plan: Plan },
    /// A scenario starts.
    Scenario(Scenario),
    /// A request was measured.
    Measured { step: Step, sample: Sample },
    /// Something went wrong that the run goes on without.
    Warning(String),
}

/// A run measured, and where it is saved.
#[derive(Debug)]
pub struct Recorded {
    pub run: Run,
    /// The saved run's file, or why the run cannot be saved.
    pub saved: Result<PathBuf>,
    /// The request that ended the run before its last scenario; the run
    /// holds the scenarios completed before it.
    pub failure: Option<anyhow::Error>,
}

/// `bench run`, with the ports it drives.
pub struct Bench<'a> {
    /// What the gateway runs the model with, and the GPU.
    pub configurator: Configurator<'a>,
    pub power: &'a dyn PowerProbe,
    pub containers: &'a dyn Containers,
    pub gateway: &'a dyn GatewayProcess,
    pub api: &'a dyn GatewayApi,
    pub server: &'a dyn ModelServer,
    pub chat: &'a dyn ChatClient,
    pub monitor: &'a dyn GpuMonitor,
    pub runs: &'a dyn RunStore,
    pub clock: fn() -> Timestamp,
}

impl Bench<'_> {
    /// Measures `options.model` through the running gateway, and saves the
    /// run after each scenario and at its end. A request that fails ends the
    /// run, naming its step: the
    /// scenarios completed before it make the run, saved and returned with
    /// that failure; with none completed, the failure is the error.
    pub fn run(&self, options: &Options, events: &mut dyn FnMut(Event)) -> Result<Recorded> {
        let model = options.model.as_str();
        check_id("model", model)?;
        ensure!(!options.scenarios.is_empty(), "no scenario to run");
        let listen = self.listen()?;
        let setup = self.setup(model, events)?;
        let started = (self.clock)();
        events(Event::Loading {
            model: model.to_owned(),
        });
        let n_ctx = self
            .server
            .context_size(listen, model)
            .with_context(|| format!("cannot read the context size of {model}"))?;
        let runs = |scenario| options.scenarios.contains(&scenario);
        let prompts = Prompts::load(self.server, listen, model)?;
        let turn_extra = if runs(Scenario::Session) {
            prompts.turn_extra()?
        } else {
            0
        };
        let plan = Plan::new(n_ctx, options.repeat, &options.scenarios, turn_extra)?;
        // Read before the run, so a model without one fails at once.
        let direct = runs(Scenario::Overhead)
            .then(|| self.direct(listen, model))
            .transpose()?;
        let runner = Runner {
            chat: self.chat,
            prompts,
            listen,
            model,
            drafts: Cell::default(),
        };
        events(Event::Planned {
            n_ctx,
            plan: plan.clone(),
        });
        let prepared = runner.prepare(&plan, &options.scenarios)?;
        let warm_up = prompt::agent("");
        runner.send(listen, Step::WarmUp, &warm_up, WARMUP_GENERATE, events)?;
        let before = self.spec_counters(listen, model, events);
        runner.drafts.take();
        let mut run = Run {
            started,
            model: model.to_owned(),
            setup,
            n_ctx,
            plan,
            depths: Vec::new(),
            prefill: None,
            session: None,
            overhead: None,
            drafts: None,
            gpu: Vec::new(),
        };
        let monitoring = self.monitor(events);
        let mut completed = Vec::new();
        let mut failure = None;
        // The run is saved after each scenario, so that a stop loses only
        // the scenario it was in: the file a first save wrote, replaced.
        let mut file = None;
        let mut unsaved = false;
        for prepared in &prepared {
            let scenario = prepared.scenario();
            events(Event::Scenario(scenario));
            if let Some(monitoring) = &monitoring {
                monitoring.enter(scenario);
            }
            match runner.measure(prepared, &mut run, direct, events) {
                Ok(()) => {
                    completed.push(scenario.name());
                    match self.store(&run, file.as_ref()) {
                        Ok(path) => file = Some(path),
                        Err(err) if !unsaved => {
                            unsaved = true;
                            events(Event::Warning(format!(
                                "the run cannot be saved as it is measured ({err:#}), so a stop \
                                 before its end loses it"
                            )));
                        }
                        Err(_) => {}
                    }
                }
                Err(err) => {
                    failure = Some((scenario, err));
                    break;
                }
            }
        }
        if let Some(monitoring) = monitoring {
            match monitoring.finish() {
                Ok(readings) => {
                    if let Some((count, last)) = readings.failed {
                        events(Event::Warning(lost_readings(count, &last)));
                    }
                    run.gpu = readings.taken;
                }
                Err(err) => events(Event::Warning(format!("{err:#}"))),
            }
        }
        let Some((failed, err)) = failure else {
            if let Some(before) = before {
                run.drafts = self
                    .spec_counters(listen, model, events)
                    .map(|after| DraftCheck {
                        requests: runner.drafts.get(),
                        before,
                        after,
                    });
            }
            let saved = self.store(&run, file.as_ref());
            return Ok(Recorded {
                run,
                saved,
                failure: None,
            });
        };
        // The scenarios completed make a run of their own, as if they alone
        // were selected. The failed request's drafts are unknown, so the
        // drafts go unchecked.
        if completed.is_empty() {
            return Err(err);
        }
        run.gpu.retain(|reading| reading.scenario != Some(failed));
        let completed = listed(&completed);
        let saved = self.store(&run, file.as_ref());
        Ok(Recorded {
            run,
            saved,
            failure: Some(anyhow!(
                "{err:#}; the run holds only the scenarios it completed: {completed}"
            )),
        })
    }

    /// Saves `run`, in place of `file` when a save during the run wrote it.
    fn store(&self, run: &Run, file: Option<&PathBuf>) -> Result<PathBuf> {
        match file {
            Some(_) => self.runs.replace(run),
            None => self.runs.save(run),
        }
    }

    /// What the gateway runs `model` with, and on what machine. The
    /// gateway must run the configuration the inventory gives now, or the
    /// run would record another model, source or flags than it measures.
    fn setup(&self, model: &str, events: &mut dyn FnMut(Event)) -> Result<Setup> {
        let prepared = self.configurator.prepare()?;
        let models = prepared.inventory.models();
        // The configuration holds one entry per model, in the inventory's
        // order.
        let (entry, configured) = models
            .iter()
            .zip(&prepared.config.models)
            .find(|(entry, _)| entry.id == model)
            .ok_or_else(|| {
                let ids: Vec<&str> = models.iter().map(|entry| entry.id.as_str()).collect();
                anyhow!(
                    "the inventory has no model {model}; it has {}",
                    ids.join(", ")
                )
            })?;
        ensure!(
            self.gateway.config()?.as_deref() == Some(prepared.config.to_yaml().as_str()),
            "the gateway runs another configuration than the inventory and the builds give now; \
             restart it with `ai-ctl down` and `ai-ctl up` so that the run records what it measures"
        );
        let backend = prepared
            .inventory
            .backend(&entry.backend)
            .with_context(|| format!("backend `{}` is not declared", entry.backend))?;
        // The bench speaks llama-server's API alone: another server would be
        // loaded, evicting what it cannot sit beside, to fail on its first
        // request.
        ensure!(
            backend.kind == BackendKind::LlamaServer,
            "{model} runs on {}; the bench measures llama-server models only",
            backend.kind
        );
        let source = match &backend.source {
            BackendSource::Build(project) => Source::Build(
                prepared
                    .builds
                    .get(project)
                    .with_context(|| format!("no build of {project} was resolved"))?
                    .clone(),
            ),
            BackendSource::Image(reference) => Source::Image {
                reference: reference.clone(),
                digest: self
                    .containers
                    .digest(reference)
                    .map_err(|err| {
                        events(Event::Warning(format!(
                            "the run records the image without its digest ({err:#})"
                        )));
                    })
                    .ok(),
            },
        };
        Ok(Setup {
            model: entry.clone(),
            source,
            command: configured
                .cmd
                .lines()
                .iter()
                .map(|line| line.join(" "))
                .collect(),
            machine: self.machine(&prepared.gpu),
            ai_ctl: env!("CARGO_PKG_VERSION").to_owned(),
        })
    }

    /// The machine: `gpu`, the one the configuration was sized to, and its
    /// power, which a laptop's speed depends on.
    fn machine(&self, gpu: &Gpu) -> Machine {
        Machine {
            gpu_memory_mib: gpu.memory_total >> 20,
            gpu: gpu.name.clone(),
            driver: gpu.driver_version.clone(),
            cuda: gpu.cuda_version,
            power_profile: self.power.power_profile().found().cloned(),
            ac_power: self.power.ac_power().found().copied(),
        }
    }

    /// The GPU's reading, started; `None`, with a warning, when the GPU
    /// cannot be read.
    fn monitor(&self, events: &mut dyn FnMut(Event)) -> Option<Box<dyn Monitoring + '_>> {
        self.monitor
            .start(GPU_PERIOD)
            .map_err(|err| {
                events(Event::Warning(format!(
                    "the run goes on without the GPU's memory, temperature, clock and power \
                     ({err:#})"
                )));
            })
            .ok()
    }

    /// The server's counters of speculative decoding; `None`, with a
    /// warning, when they cannot be read, as from a server run without
    /// `--metrics`.
    fn spec_counters(
        &self,
        listen: SocketAddr,
        model: &str,
        events: &mut dyn FnMut(Event),
    ) -> Option<SpecCounters> {
        self.server
            .spec_counters(listen, model)
            .map_err(|err| {
                events(Event::Warning(format!(
                    "cannot read the server's counters, which check the drafts ({err:#})"
                )));
            })
            .ok()
    }

    /// The address of the running gateway.
    fn listen(&self) -> Result<SocketAddr> {
        let record =
            live_record(self.gateway)?.context("no gateway runs; start it with `ai-ctl up`")?;
        Ok(record.listen)
    }

    /// The address of `model`'s own server, which llama-swap gives.
    fn direct(&self, listen: SocketAddr, model: &str) -> Result<SocketAddr> {
        let running = self
            .api
            .running(listen)
            .context("cannot read the address of the model's own server")?;
        let entry = running
            .iter()
            .find(|entry| entry.id == model)
            .ok_or_else(|| anyhow!("the gateway does not run {model} after loading it"))?;
        entry.proxy.ok_or_else(|| {
            anyhow!(
                "the gateway gives no address of {model}'s own server; the overhead cannot be \
                 measured"
            )
        })
    }
}

/// The saved runs, oldest first, each read or the reason it cannot be.
pub fn saved(store: &dyn RunStore) -> Result<Vec<(String, Result<Run>)>> {
    Ok(store
        .ids()?
        .into_iter()
        .map(|id| {
            let run = store.load(&id);
            (id, run)
        })
        .collect())
}

/// The runs saved under `ids`, in that order.
pub fn load(store: &dyn RunStore, ids: &[String]) -> Result<Vec<Run>> {
    ids.iter().map(|id| store.load(id)).collect()
}

/// The warning that `count` GPU readings failed, the last with `last`.
fn lost_readings(count: usize, last: &str) -> String {
    if count == 1 {
        format!("1 GPU reading failed during the run ({last}); the run's GPU figures leave it out")
    } else {
        format!(
            "{count} GPU readings failed during the run (the last: {last}); the run's GPU \
             figures leave them out"
        )
    }
}

/// `names` as a sentence lists them: `depth`, `depth and prefill`,
/// `depth, prefill and session`.
fn listed(names: &[&str]) -> String {
    match names {
        [rest @ .., last] if !rest.is_empty() => format!("{} and {last}", rest.join(", ")),
        names => names.concat(),
    }
}

/// A run under way: its prompts, where it sends them, and the drafts its
/// requests reported.
struct Runner<'a> {
    chat: &'a dyn ChatClient,
    prompts: Prompts<'a>,
    listen: SocketAddr,
    model: &'a str,
    drafts: Cell<Drafts>,
}

impl Runner<'_> {
    /// The fresh prompts of `scenarios`, in the order they run. They are
    /// fitted before the GPU is watched, so that its readings hold the
    /// measures alone, and a prompt that cannot be fitted fails the run
    /// before any request.
    fn prepare(&self, plan: &Plan, scenarios: &[Scenario]) -> Result<Vec<Prepared>> {
        Scenario::ALL
            .into_iter()
            .filter(|scenario| scenarios.contains(scenario))
            .map(|scenario| {
                Ok(match scenario {
                    Scenario::Depth => Prepared::Depths(
                        plan.depths
                            .iter()
                            .map(|depth| self.fresh(depth.tokens as usize))
                            .collect::<Result<_>>()?,
                    ),
                    Scenario::Prefill => Prepared::Prefill(self.fresh(plan.prefill as usize)?),
                    Scenario::Session => {
                        Prepared::Session(self.fresh(plan.session.opening as usize)?)
                    }
                    Scenario::Overhead => Prepared::Overhead,
                })
            })
            .collect()
    }

    /// Measures the `prepared` scenario into `run`, which keeps nothing of
    /// it on a failure. The overhead is measured only with `direct`, the
    /// address of the model's own server.
    fn measure(
        &self,
        prepared: &Prepared,
        run: &mut Run,
        direct: Option<SocketAddr>,
        events: &mut dyn FnMut(Event),
    ) -> Result<()> {
        match prepared {
            Prepared::Depths(fresh) => run.depths = self.depths(&run.plan, fresh, events)?,
            Prepared::Prefill(fresh) => {
                run.prefill = Some(self.prefill(&run.plan, fresh, events)?);
            }
            Prepared::Session(opening) => {
                run.session = Some(self.sessions(&run.plan, opening, events)?);
            }
            Prepared::Overhead => {
                if let Some(direct) = direct {
                    run.overhead = Some(self.overhead(&run.plan, direct, events)?);
                }
            }
        }
        Ok(())
    }

    /// Sends `messages` to `addr` for `max_tokens` tokens exactly, out of
    /// the cache when `step` is fresh, and reports the measure.
    fn send(
        &self,
        addr: SocketAddr,
        step: Step,
        messages: &[Message],
        max_tokens: u32,
        events: &mut dyn FnMut(Event),
    ) -> Result<Reply> {
        let request = ChatRequest {
            model: self.model.to_owned(),
            messages: messages.to_vec(),
            max_tokens,
            ignore_eos: true,
            seed: SEED,
            cache_prompt: !step.fresh(),
        };
        let reply = self
            .chat
            .stream(addr, &request)
            .with_context(|| step.to_string())?;
        if let Some(drafts) = reply.sample.server.and_then(|server| server.drafts) {
            self.drafts.set(self.drafts.get() + drafts);
        }
        events(Event::Measured {
            step,
            sample: reply.sample.clone(),
        });
        Ok(reply)
    }

    /// The agent's prompt of `tokens` tokens, with the prompt the request
    /// before it sends: the same cut to a [`LEAD_FRACTION`], or the agent's
    /// own prompt without code when that fraction cannot hold it, which
    /// shares all but its instruction with the fresh prompt. The cut is not
    /// fitted: the token or so where its code ends keeps it far above the
    /// tenth llama-server's slot similarity asks for.
    fn fresh(&self, tokens: usize) -> Result<Fresh> {
        let own = self.prompts.own;
        let cut = tokens / LEAD_FRACTION;
        let lead = if cut > own {
            prompt::agent(&self.prompts.corpus.text(0, cut - own))
        } else {
            prompt::agent("")
        };
        Ok(Fresh {
            prompt: self.prompts.agent(tokens)?,
            lead,
        })
    }

    /// Sends `lead` before `next`, through the gateway: see
    /// [`Step::SetAside`].
    fn set_aside(&self, next: Step, lead: &[Message], events: &mut dyn FnMut(Event)) -> Result<()> {
        self.send(self.listen, Step::SetAside, lead, WARMUP_GENERATE, events)
            .with_context(|| format!("before {next}"))?;
        Ok(())
    }

    /// Sends the fresh `step` after the request that sets the context
    /// aside.
    fn send_fresh(
        &self,
        step: Step,
        fresh: &Fresh,
        max_tokens: u32,
        events: &mut dyn FnMut(Event),
    ) -> Result<Reply> {
        self.set_aside(step, &fresh.lead, events)?;
        self.send(
            self.listen,
            step,
            &fresh.prompt.messages,
            max_tokens,
            events,
        )
    }

    /// Generation at each depth: the context filled once with a fresh
    /// prompt, then the same prompt again, from the cache.
    fn depths(
        &self,
        plan: &Plan,
        fresh: &[Fresh],
        events: &mut dyn FnMut(Event),
    ) -> Result<Vec<DepthResult>> {
        let mut results = Vec::new();
        for (&depth, fresh) in plan.depths.iter().zip(fresh) {
            let fill = self.send_fresh(Step::Fill(depth), fresh, plan.generate, events)?;
            let mut samples = Vec::new();
            for repetition in 1..=plan.repeat {
                let step = Step::Generate { depth, repetition };
                let messages = &fresh.prompt.messages;
                let reply = self.send(self.listen, step, messages, plan.generate, events)?;
                samples.push(reply.sample);
            }
            results.push(DepthResult {
                depth,
                fill: fill.sample,
                samples,
            });
        }
        Ok(results)
    }

    /// The same prompt, evaluated whole each time.
    fn prefill(
        &self,
        plan: &Plan,
        fresh: &Fresh,
        events: &mut dyn FnMut(Event),
    ) -> Result<PrefillResult> {
        let mut samples = Vec::new();
        for repetition in 1..=plan.repeat {
            let step = Step::Prefill { repetition };
            let reply = self.send_fresh(step, fresh, PREFILL_GENERATE, events)?;
            samples.push(reply.sample);
        }
        Ok(PrefillResult { samples })
    }

    /// Agents' sessions, each the same: the first turn evaluated whole,
    /// then each turn sends the history back, the replies with their
    /// reasoning, and adds the next part of the code.
    fn sessions(
        &self,
        plan: &Plan,
        opening: &Fresh,
        events: &mut dyn FnMut(Event),
    ) -> Result<SessionResult> {
        let shape = plan.session;
        let mut sessions = Vec::new();
        for session in 1..=plan.repeat {
            let (mut messages, mut from) = (opening.prompt.messages.clone(), opening.prompt.code);
            let mut samples = Vec::new();
            for turn in 1..=shape.turns {
                let step = Step::Turn { session, turn };
                let reply = if turn == 1 {
                    self.send_fresh(step, opening, plan.generate, events)?
                } else {
                    messages.push(self.prompts.turn(from, shape.turn as usize));
                    from += shape.turn as usize;
                    self.send(self.listen, step, &messages, plan.generate, events)?
                };
                messages.push(Message::assistant(reply.content, reply.reasoning));
                samples.push(reply.sample);
            }
            sessions.push(samples);
        }
        Ok(SessionResult { sessions })
    }

    /// The same short request, in the cache, sent in turn through the
    /// gateway and straight to the model's own server at `own`. It is the
    /// warm-up's, which the request that sets the context aside puts in
    /// the slot.
    fn overhead(
        &self,
        plan: &Plan,
        own: SocketAddr,
        events: &mut dyn FnMut(Event),
    ) -> Result<OverheadResult> {
        let messages = prompt::agent("");
        let first = Step::Overhead {
            request: 1,
            direct: false,
        };
        self.set_aside(first, &messages, events)?;
        let mut result = OverheadResult {
            gateway: Vec::new(),
            direct: Vec::new(),
        };
        for request in 1..=plan.overhead {
            for (addr, direct, into) in [
                (self.listen, false, &mut result.gateway),
                (own, true, &mut result.direct),
            ] {
                let step = Step::Overhead { request, direct };
                into.push(
                    self.send(addr, step, &messages, OVERHEAD_GENERATE, events)?
                        .sample,
                );
            }
        }
        Ok(result)
    }
}

/// A scenario of a run, with the fresh prompts it sends.
enum Prepared {
    /// One for each depth of the plan, in its order.
    Depths(Vec<Fresh>),
    Prefill(Fresh),
    /// The opening of each session.
    Session(Fresh),
    Overhead,
}

impl Prepared {
    fn scenario(&self) -> Scenario {
        match self {
            Prepared::Depths(_) => Scenario::Depth,
            Prepared::Prefill(_) => Scenario::Prefill,
            Prepared::Session(_) => Scenario::Session,
            Prepared::Overhead => Scenario::Overhead,
        }
    }
}

/// A fresh request's prompt, and the messages of the request before it.
struct Fresh {
    prompt: Fitted,
    lead: Vec<Message>,
}

/// Attempts at fitting a prompt. The first misses by the tokens the
/// template adds around the code; the next by the token or so where the
/// code meets the text after it, which a second attempt corrects.
const FIT_ATTEMPTS: usize = 4;

/// A prompt fitted to a number of tokens.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Fitted {
    pub messages: Vec<Message>,
    /// Tokens of the templated prompt.
    pub tokens: usize,
    /// Tokens of the corpus it holds.
    pub code: usize,
}

/// The prompts of one model, built with its template and tokenizer.
pub struct Prompts<'a> {
    server: &'a dyn ModelServer,
    listen: SocketAddr,
    model: &'a str,
    corpus: Corpus,
    /// Tokens of the agent's own prompt, without code.
    own: usize,
}

impl<'a> Prompts<'a> {
    /// Tokenizes the corpus with `model`'s tokenizer, through the gateway
    /// listening on `listen`, and counts the agent's own prompt.
    pub fn load(server: &'a dyn ModelServer, listen: SocketAddr, model: &'a str) -> Result<Self> {
        let pieces = server
            .pieces(listen, model, CORPUS)
            .context("cannot tokenize the bench's corpus")?;
        let mut prompts = Self {
            server,
            listen,
            model,
            corpus: Corpus::new(pieces)?,
            own: 0,
        };
        prompts.own = prompts.count(&prompt::agent(""))?;
        Ok(prompts)
    }

    /// Tokens of `messages` once templated, ready for the reply.
    pub fn count(&self, messages: &[Message]) -> Result<usize> {
        let prompt = self.server.template(self.listen, self.model, messages)?;
        self.server.count(self.listen, self.model, &prompt)
    }

    /// The agent's prompt of exactly `tokens` tokens; when no attempt is
    /// exact, the longest one under `tokens`.
    pub fn agent(&self, tokens: usize) -> Result<Fitted> {
        let own = self.own;
        ensure!(
            own <= tokens,
            "a prompt of {tokens} tokens cannot hold the agent's own {own}"
        );
        let mut code = tokens - own;
        let mut best: Option<Fitted> = None;
        let mut closest = usize::MAX;
        for _ in 0..FIT_ATTEMPTS {
            let messages = prompt::agent(&self.corpus.text(0, code));
            let counted = self.count(&messages)?;
            closest = closest.min(counted);
            if counted <= tokens && best.as_ref().is_none_or(|best| counted > best.tokens) {
                best = Some(Fitted {
                    messages,
                    tokens: counted,
                    code,
                });
            }
            if counted == tokens {
                break;
            }
            code = (code + tokens).saturating_sub(counted);
        }
        best.ok_or_else(|| {
            anyhow!(
                "no prompt of {tokens} tokens could be fitted: the shortest attempt made {closest}"
            )
        })
    }

    /// Tokens a later turn of a session adds besides its code and the reply
    /// before it: the turn's own text, and what the template puts around
    /// the reply and the turn.
    pub fn turn_extra(&self) -> Result<u32> {
        let mut next = prompt::agent("");
        next.extend([Message::assistant("", ""), prompt::turn("")]);
        let extra = self.count(&next)?.saturating_sub(self.own);
        u32::try_from(extra).context("a session's turn adds more tokens than a context holds")
    }

    /// A later turn of a session: `tokens` tokens of the corpus from token
    /// `from`, where the turn before stopped.
    pub fn turn(&self, from: usize, tokens: usize) -> Message {
        prompt::turn(&self.corpus.text(from, tokens))
    }
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::collections::BTreeMap;

    use super::*;
    use crate::application::fakes::{
        FakeApi, FakeBuilds, FakeChat, FakeContainers, FakeGateway, FakeInventory, FakeModelServer,
        FakeMonitor, FakePower, FakeRuns, ScriptedGpu, b11538, gpu, inventory, inventory_parts,
        model, record,
    };
    use crate::application::ports::{Liveness, RunningModel};
    use crate::domain::inventory::{Backend, Inventory};
    use crate::domain::machine::Probed;

    const LISTEN: SocketAddr =
        SocketAddr::new(std::net::IpAddr::V4(std::net::Ipv4Addr::LOCALHOST), 8080);

    #[test]
    fn the_corpus_is_tokenized_by_the_models_server() {
        let server = FakeModelServer::default();
        let prompts = Prompts::load(&server, LISTEN, "qwen3.8-27b").unwrap();
        // One token a character: the corpus's whole text, and no more.
        assert_eq!(prompts.corpus.text(0, CORPUS.chars().count()), CORPUS);
        assert_eq!(
            server.calls.borrow()[0],
            "pieces qwen3.8-27b 127.0.0.1:8080"
        );
    }

    #[test]
    fn a_prompt_is_fitted_to_the_token() {
        let server = FakeModelServer::default();
        let prompts = Prompts::load(&server, LISTEN, "m").unwrap();
        for tokens in [4_000, 8_192, 40_960] {
            let fitted = prompts.agent(tokens).unwrap();
            assert_eq!(fitted.tokens, tokens);
            assert_eq!(prompts.count(&fitted.messages).unwrap(), tokens);
            assert!(
                fitted.messages[1]
                    .content
                    .contains("<file path=\"src/presentation/cli.rs\">")
            );
        }
    }

    #[test]
    fn a_boundary_that_merges_tokens_is_corrected() {
        // Two characters where the code ends and the instruction starts
        // make one token: the first attempt falls one short.
        let server = FakeModelServer {
            boundary_merge: true,
            ..FakeModelServer::default()
        };
        let prompts = Prompts::load(&server, LISTEN, "m").unwrap();
        let fitted = prompts.agent(5_000).unwrap();
        assert_eq!(fitted.tokens, 5_000);
        let attempts = server
            .calls
            .borrow()
            .iter()
            .filter(|c| c.starts_with("template"))
            .count();
        // One for the agent's own tokens, then two attempts.
        assert_eq!(attempts, 3);
    }

    #[test]
    fn without_an_exact_fit_the_longest_shorter_prompt_is_kept() {
        // Every count is rounded up to an even number: an odd target is
        // never reached.
        let server = FakeModelServer {
            even: true,
            ..FakeModelServer::default()
        };
        let prompts = Prompts::load(&server, LISTEN, "m").unwrap();
        let fitted = prompts.agent(5_001).unwrap();
        assert_eq!(fitted.tokens, 5_000);
    }

    #[test]
    fn a_tokenizer_that_overshoots_every_attempt_fits_no_prompt() {
        let server = FakeModelServer {
            overshoot: Some(5_000),
            ..FakeModelServer::default()
        };
        let prompts = Prompts::load(&server, LISTEN, "m").unwrap();
        assert_eq!(
            prompts.agent(5_000).unwrap_err().to_string(),
            "no prompt of 5000 tokens could be fitted: the shortest attempt made 5003"
        );
    }

    #[test]
    fn a_prompt_too_short_for_the_agent_is_refused() {
        let server = FakeModelServer::default();
        let prompts = Prompts::load(&server, LISTEN, "m").unwrap();
        let error = prompts.agent(100).unwrap_err().to_string();
        assert!(
            error.starts_with("a prompt of 100 tokens cannot hold the agent's own "),
            "{error}"
        );
    }

    #[test]
    fn a_turn_continues_the_code_where_the_last_stopped() {
        let server = FakeModelServer::default();
        let prompts = Prompts::load(&server, LISTEN, "m").unwrap();
        let first = prompts.turn(0, 10);
        assert!(first.content.ends_with("<file path"));
        let next = prompts.turn(10, 26);
        assert!(next.content.ends_with("=\"src/presentation/cli.rs\""));
    }

    #[test]
    fn a_later_turn_adds_its_text_and_the_templates_markers() {
        let server = FakeModelServer::default();
        let prompts = Prompts::load(&server, LISTEN, "m").unwrap();
        // The fake template marks each message with its role.
        let text = prompt::turn("").content.chars().count();
        let markers = "<assistant>".len() + "<user>".len();
        assert_eq!(prompts.turn_extra().unwrap() as usize, text + markers);
    }

    #[test]
    fn a_server_that_fails_is_named() {
        let server = FakeModelServer {
            fails: true,
            ..FakeModelServer::default()
        };
        let error = Prompts::load(&server, LISTEN, "m").err().unwrap();
        assert_eq!(
            format!("{error:#}"),
            "cannot tokenize the bench's corpus: connection refused"
        );
    }

    /// The address of the model's own server.
    fn own() -> SocketAddr {
        "127.0.0.1:5801".parse().unwrap()
    }

    fn started() -> Timestamp {
        Timestamp::parse("2026-10-11T12:00:00Z").unwrap()
    }

    /// The model of the fake inventory.
    const MODEL: &str = "qwen3.8-27b";

    /// A gateway running the inventory's configuration, its model's server
    /// with a context of 98304 tokens.
    struct World {
        inventory: FakeInventory,
        builds: FakeBuilds,
        gpu: ScriptedGpu,
        power: FakePower,
        containers: FakeContainers,
        gateway: FakeGateway,
        api: FakeApi,
        server: FakeModelServer,
        chat: FakeChat,
        monitor: FakeMonitor,
        runs: FakeRuns,
    }

    impl World {
        fn new() -> Self {
            let mut api = FakeApi::answering_after(0);
            api.models = Some(vec![RunningModel {
                id: MODEL.into(),
                state: "ready".into(),
                proxy: Some(own()),
            }]);
            let world = Self {
                inventory: FakeInventory(Ok(Some(inventory("/models")))),
                builds: FakeBuilds::current_b11538(),
                gpu: ScriptedGpu::new(vec![Probed::Found(gpu())]),
                power: FakePower::healthy(),
                containers: FakeContainers::default(),
                gateway: FakeGateway {
                    record: RefCell::new(Some(record(1234))),
                    ..FakeGateway::default()
                },
                api,
                server: FakeModelServer::default(),
                chat: FakeChat::default(),
                monitor: FakeMonitor::default(),
                runs: FakeRuns::default(),
            };
            let config = world.configurator().generate().unwrap().to_yaml();
            world.gateway.config.replace(Some(config));
            world
        }

        fn configurator(&self) -> Configurator<'_> {
            Configurator {
                inventory: &self.inventory,
                builds: &self.builds,
                gpu: &self.gpu,
                files_dir: std::path::Path::new(crate::application::fakes::GATEWAY_DIR),
            }
        }

        fn bench(&self) -> Bench<'_> {
            Bench {
                configurator: self.configurator(),
                power: &self.power,
                containers: &self.containers,
                gateway: &self.gateway,
                api: &self.api,
                server: &self.server,
                chat: &self.chat,
                monitor: &self.monitor,
                runs: &self.runs,
                clock: started,
            }
        }

        /// The run of `scenarios`, recorded, with its events.
        fn record(&self, scenarios: &[Scenario]) -> (Result<Recorded>, Vec<Event>) {
            let bench = self.bench();
            let options = Options {
                model: MODEL.into(),
                repeat: 2,
                scenarios: scenarios.to_vec(),
            };
            let mut events = Vec::new();
            let recorded = bench.run(&options, &mut |event| events.push(event));
            (recorded, events)
        }

        /// The run, or the error that ended it, partial or not.
        fn run(&self, scenarios: &[Scenario]) -> (Result<Run>, Vec<Event>) {
            let (recorded, events) = self.record(scenarios);
            let run = recorded.and_then(|recorded| match recorded.failure {
                Some(failure) => Err(failure),
                None => Ok(recorded.run),
            });
            (run, events)
        }

        /// The requests sent: where, and the prompt.
        fn sent(&self) -> Vec<(SocketAddr, String)> {
            self.chat
                .requests
                .borrow()
                .iter()
                .map(|(addr, request)| (*addr, FakeChat::prompt(&request.messages)))
                .collect()
        }
    }

    fn tokens(sample: &Sample) -> u64 {
        sample.usage.unwrap().prompt_tokens
    }

    #[test]
    fn a_context_too_small_for_a_session_runs_the_other_scenarios() {
        let mut world = World::new();
        world.server.ctx_size = Some(16_384);
        let run = world.run(&[Scenario::Depth]).0.unwrap();
        assert_eq!(run.depths.last().unwrap().depth.tokens, 16_384 - 512 - 1);

        let error = world
            .run(&[Scenario::Depth, Scenario::Session])
            .0
            .unwrap_err();
        assert_eq!(
            error.to_string(),
            "the session scenario needs a context of 23096 tokens; the model's holds 16384"
        );
    }

    #[test]
    fn a_session_is_planned_on_the_tokens_its_last_turn_sends() {
        // The fake's turn adds 63 tokens besides its code and the reply.
        let mut world = World::new();
        world.server.ctx_size = Some(23_095);
        let error = world.run(&[Scenario::Session]).0.unwrap_err();
        assert_eq!(
            error.to_string(),
            "the session scenario needs a context of 23096 tokens; the model's holds 23095"
        );

        world.server.ctx_size = Some(23_096);
        let run = world.run(&[Scenario::Session]).0.unwrap();
        let last = run.session.unwrap().sessions[0].last().unwrap().clone();
        let filled = tokens(&last) + last.usage.unwrap().completion_tokens;
        assert_eq!(filled + 1, 23_096);
    }

    #[test]
    fn a_run_measures_every_scenario_through_the_gateway() {
        let world = World::new();
        let (run, events) = world.run(&Scenario::ALL);
        let run = run.unwrap();
        assert_eq!((run.started, run.model.as_str()), (started(), MODEL));
        assert_eq!(run.n_ctx, 98_304);
        assert_eq!(run.plan, Plan::new(98_304, 2, &Scenario::ALL, 0).unwrap());
        assert_eq!(run.depths.len(), 4);
        assert_eq!(run.prefill.unwrap().samples.len(), 2);
        assert_eq!(run.session.unwrap().sessions.len(), 2);
        let overhead = run.overhead.unwrap();
        assert_eq!((overhead.gateway.len(), overhead.direct.len()), (50, 50));
        // The warm-up, 4 depths of a fill and 2 repetitions, 2 prefills, 2
        // sessions of 10 turns, and 50 overhead requests each way; the
        // context set aside before each fill, prefill, session and the
        // overhead.
        let requests = world.chat.requests.borrow();
        assert_eq!(
            requests.len(),
            1 + 4 * (1 + 3) + 2 * (1 + 1) + 2 * (1 + 10) + 1 + 50 * 2
        );
        assert!(requests.iter().all(|(_, request)| request.ignore_eos
            && request.seed == SEED
            && request.model == MODEL));
        assert_eq!(
            events[..3],
            [
                Event::Loading {
                    model: MODEL.into()
                },
                Event::Planned {
                    n_ctx: 98_304,
                    plan: Plan::new(98_304, 2, &Scenario::ALL, 0).unwrap()
                },
                Event::Measured {
                    step: Step::WarmUp,
                    sample: events[2].sample().clone()
                },
            ]
        );
        let scenarios: Vec<Scenario> = events
            .iter()
            .filter_map(|event| match event {
                Event::Scenario(scenario) => Some(*scenario),
                _ => None,
            })
            .collect();
        assert_eq!(scenarios, Scenario::ALL);
        let measured = events
            .iter()
            .filter(|event| matches!(event, Event::Measured { .. }))
            .count();
        assert_eq!(measured, requests.len());
    }

    impl Event {
        fn sample(&self) -> &Sample {
            match self {
                Event::Measured { sample, .. } => sample,
                other => panic!("not a measure: {other:?}"),
            }
        }
    }

    #[test]
    fn a_depth_fills_the_context_once_and_generates_on_it_again() {
        let world = World::new();
        let run = world.run(&[Scenario::Depth]).0.unwrap();
        let depths: Vec<u32> = run.depths.iter().map(|d| d.depth.tokens).collect();
        assert_eq!(depths, [8_192, 32_768, 65_536, 97_791]);
        for result in &run.depths {
            let depth = u64::from(result.depth.tokens);
            assert_eq!(tokens(&result.fill), depth);
            assert_eq!(result.fill.cached(), Some(0));
            assert_eq!(result.samples.len(), 2);
            for sample in &result.samples {
                assert_eq!(tokens(sample), depth);
                assert_eq!(sample.cached(), Some(depth - 1));
                assert_eq!(sample.evaluated(), Some(1));
            }
        }
        let sent = world.sent();
        assert_eq!(sent.len(), 1 + 4 * 4);
        assert!(sent[2..5].iter().all(|request| request == &sent[2]));
        assert!(sent.iter().all(|(addr, _)| *addr == record(1).listen));
        // Each fill comes after a quarter of its prompt, or the agent's
        // own prompt when a quarter cannot hold it, as the fake's 2645
        // tokens at 8192; only the fill asks to miss the cache.
        let prompts = Prompts::load(&world.server, LISTEN, MODEL).unwrap();
        let requests = world.chat.requests.borrow();
        assert_eq!(requests[1].1.messages, prompt::agent(""));
        assert_eq!(
            requests[5].1.messages,
            prompts.agent(8_192).unwrap().messages
        );
        drop(requests);
        let cached: Vec<bool> = world
            .chat
            .requests
            .borrow()
            .iter()
            .map(|(_, request)| request.cache_prompt)
            .collect();
        assert_eq!(cached[1..5], [true, false, true, true]);
        assert_eq!(cached[5..9], [true, false, true, true]);
    }

    #[test]
    fn a_prompt_that_cannot_be_fitted_fails_before_any_request() {
        let mut world = World::new();
        world.server.overshoot = Some(40_960);
        let (run, _) = world.run(&[Scenario::Prefill]);
        assert_eq!(
            run.unwrap_err().to_string(),
            "no prompt of 40960 tokens could be fitted: the shortest attempt made 40963"
        );
        assert!(world.chat.requests.borrow().is_empty());
        assert!(!world.monitor.finished.get(), "the GPU was watched");
    }

    #[test]
    fn a_fresh_prompt_is_fitted_and_its_lead_cut_without_counting_again() {
        let world = World::new();
        world.run(&[Scenario::Prefill]).0.unwrap();
        // The agent's bare prompt once, then the fresh prompt, exact at
        // the first attempt with the fake's tokenizer.
        let counts = world
            .server
            .calls
            .borrow()
            .iter()
            .filter(|call| call.starts_with("count"))
            .count();
        assert_eq!(counts, 2);
    }

    #[test]
    fn a_session_counts_the_agents_bare_prompt_once() {
        let world = World::new();
        world.run(&[Scenario::Session]).0.unwrap();
        // The bare prompt when the prompts load, the opening, exact at the
        // first attempt, and the opening followed by a turn.
        let calls = world.server.calls.borrow();
        let count = |kind: &str| calls.iter().filter(|call| call.starts_with(kind)).count();
        assert_eq!((count("template"), count("count")), (3, 3));
    }

    #[test]
    fn a_fresh_request_comes_after_the_context_is_set_aside() {
        let world = World::new();
        let (run, events) = world.run(&Scenario::ALL);
        run.unwrap();
        let steps: Vec<Step> = events
            .iter()
            .filter_map(|event| match event {
                Event::Measured { step, .. } => Some(*step),
                _ => None,
            })
            .collect();
        for (i, step) in steps.iter().enumerate() {
            if step.fresh() {
                assert_eq!(steps[i - 1], Step::SetAside, "before {step}");
            }
        }
        let overhead = steps
            .iter()
            .position(|step| matches!(step, Step::Overhead { .. }))
            .unwrap();
        assert_eq!(steps[overhead - 1], Step::SetAside);
        // Whatever came before, a fresh request finds in the slot a prompt
        // it shares more than a tenth of its own with, and keeps at least
        // half of it, its reply included: llama-server then neither saves
        // nor loads a context.
        let sent = world.sent();
        let requests = world.chat.requests.borrow();
        for (i, (_, request)) in requests.iter().enumerate().skip(1) {
            if request.cache_prompt {
                continue;
            }
            let (lead, prompt) = (&sent[i - 1].1, &sent[i].1);
            let shared = lead
                .chars()
                .zip(prompt.chars())
                .take_while(|(a, b)| a == b)
                .count();
            let held = lead.chars().count() + "<assistant>".len() + WARMUP_GENERATE as usize;
            assert!(shared * 10 > prompt.chars().count(), "request {i}");
            assert!(shared * 2 >= held, "request {i}");
        }
    }

    #[test]
    fn each_prefill_evaluates_the_same_prompt_whole() {
        let world = World::new();
        let run = world.run(&[Scenario::Prefill]).0.unwrap();
        let samples = run.prefill.unwrap().samples;
        for sample in &samples {
            assert_eq!(tokens(sample), 40_960);
            assert_eq!(sample.cached(), Some(0));
        }
        let requests = world.chat.requests.borrow();
        assert_eq!(requests.len(), 1 + 2 * 2);
        assert!(
            [&requests[2], &requests[4]]
                .iter()
                .all(|(_, r)| r.max_tokens == PREFILL_GENERATE && !r.cache_prompt)
        );
        let sent = world.sent();
        assert_eq!((&sent[1], &sent[2]), (&sent[3], &sent[4]));
    }

    #[test]
    fn two_runs_send_the_same_prompts() {
        let first = World::new();
        first.run(&Scenario::ALL).0.unwrap();
        let second = World::new();
        second.run(&Scenario::ALL).0.unwrap();
        assert_eq!(first.sent(), second.sent());
    }

    #[test]
    fn a_session_sends_its_history_back_and_reads_on() {
        let world = World::new();
        let run = world.run(&[Scenario::Session]).0.unwrap();
        let result = run.session.unwrap();
        assert!(result.cache_grows());
        let first = &result.sessions[0];
        assert_eq!(first.len(), 10);
        // Turn 2 finds turn 1's prompt and reply in the cache.
        assert_eq!(
            first[1].cached(),
            Some(tokens(&first[0]) + "<assistant>".len() as u64 + 512)
        );
        // The warm-up, then each session after the context is set aside.
        let requests = world.chat.requests.borrow();
        assert_eq!(requests.len(), 1 + 2 * (1 + 10));
        let last = &requests[11].1.messages;
        assert_eq!(last.len(), 2 + 9 * 2);
        assert_eq!(last[2].role.name(), "assistant");
        assert_eq!(last[2].reasoning.as_deref(), Some("r".repeat(512).as_str()));
        // Each turn reads the code on from where the turn before stopped.
        let prompts = Prompts::load(&world.server, LISTEN, MODEL).unwrap();
        let opening = prompts.agent(8_192).unwrap();
        assert_eq!(last[3], prompts.turn(opening.code, 1_024));
        assert_eq!(last[5], prompts.turn(opening.code + 1_024, 1_024));
        // Each session opens out of the cache, and is the same.
        assert!(!requests[2].1.cache_prompt && !requests[13].1.cache_prompt);
        assert!(requests[3..12].iter().all(|(_, r)| r.cache_prompt));
        assert_eq!(requests[1..12], requests[12..23]);
        assert_eq!(result.sessions[0], result.sessions[1]);
    }

    #[test]
    fn the_overhead_alternates_the_gateway_and_the_models_own_server() {
        let world = World::new();
        let run = world.run(&[Scenario::Overhead]).0.unwrap();
        let overhead = run.overhead.unwrap();
        assert!(
            overhead
                .direct
                .iter()
                .all(|sample| sample.cached().is_some())
        );
        let sent = world.sent();
        assert_eq!(sent.len(), 2 + 100);
        // The warm-up and the request that sets the context aside go
        // through the gateway.
        let gateway = record(1).listen;
        assert_eq!((sent[0].0, sent[1].0), (gateway, gateway));
        for pair in sent[2..].chunks(2) {
            assert_eq!((pair[0].0, pair[1].0), (gateway, own()));
            assert_eq!(pair[0].1, sent[1].1);
            assert_eq!(pair[1].1, sent[1].1);
        }
        let requests = world.chat.requests.borrow();
        assert!(
            requests[1..]
                .iter()
                .all(|(_, r)| r.max_tokens == OVERHEAD_GENERATE)
        );
    }

    #[test]
    fn only_the_chosen_scenarios_run() {
        let mut world = World::new();
        // Read only for the overhead.
        world.api.models = None;
        let run = world.run(&[Scenario::Session, Scenario::Depth]).0.unwrap();
        assert_eq!(run.depths.len(), 4);
        assert!(run.session.is_some());
        assert_eq!((run.prefill, run.overhead), (None, None));
        let (run, _) = world.run(&[]);
        assert_eq!(run.unwrap_err().to_string(), "no scenario to run");
    }

    #[test]
    fn without_a_gateway_nothing_is_sent() {
        let mut world = World::new();
        world.gateway.liveness = Some(Liveness::Exited);
        let error = world.run(&Scenario::ALL).0.unwrap_err();
        assert_eq!(
            error.to_string(),
            "no gateway runs; start it with `ai-ctl up`"
        );
        world.gateway.record.replace(None);
        assert!(world.run(&Scenario::ALL).0.is_err());
        assert!(world.chat.requests.borrow().is_empty());
        assert!(world.server.calls.borrow().is_empty());
    }

    #[test]
    fn a_model_id_is_checked_before_anything_is_sent() {
        let world = World::new();
        let bench = world.bench();
        let options = Options {
            model: "../m".into(),
            repeat: 1,
            scenarios: Scenario::ALL.to_vec(),
        };
        let error = bench.run(&options, &mut |_| {}).unwrap_err().to_string();
        assert!(error.starts_with("model id `../m` must start"), "{error}");
        assert!(world.server.calls.borrow().is_empty());
    }

    #[test]
    fn the_overhead_needs_the_models_own_address_before_the_run() {
        let mut world = World::new();
        world.api.models = Some(vec![RunningModel {
            id: MODEL.into(),
            state: "ready".into(),
            proxy: None,
        }]);
        let error = world.run(&Scenario::ALL).0.unwrap_err().to_string();
        assert_eq!(
            error,
            "the gateway gives no address of qwen3.8-27b's own server; the overhead cannot be measured"
        );
        world.api.models = Some(vec![]);
        let error = world.run(&Scenario::ALL).0.unwrap_err().to_string();
        assert_eq!(
            error,
            "the gateway does not run qwen3.8-27b after loading it"
        );
        assert!(world.chat.requests.borrow().is_empty());
    }

    #[test]
    fn a_failed_request_ends_the_run_and_names_its_step() {
        let mut world = World::new();
        // The warm-up, the context set aside, the fill, then the first
        // repetition.
        world.chat.fails_at = Some(3);
        let error = world.run(&[Scenario::Depth]).0.unwrap_err();
        assert_eq!(
            format!("{error:#}"),
            "repetition 1 at 8192 tokens: answered 500: the slot is busy"
        );
        assert_eq!(world.chat.requests.borrow().len(), 4);
        world.chat.requests.borrow_mut().clear();
        world.chat.fails_at = Some(1);
        let error = world.run(&[Scenario::Depth]).0.unwrap_err();
        assert_eq!(
            format!("{error:#}"),
            "before the fill at 8192 tokens: the request that sets the context aside: \
             answered 500: the slot is busy"
        );
        assert_eq!(world.chat.requests.borrow().len(), 2);
        // Nothing was completed to save.
        assert!(world.runs.runs.borrow().is_empty());
    }

    /// The ID of the World's runs.
    const ID: &str = "20261011T120000Z-qwen3.8-27b";

    #[test]
    fn a_run_is_saved_before_it_is_returned() {
        let world = World::new();
        let recorded = world.record(&[Scenario::Depth]).0.unwrap();
        assert_eq!(recorded.run.id(), ID);
        assert_eq!(
            recorded.saved.unwrap(),
            PathBuf::from(format!("/data/bench/{ID}.json"))
        );
        assert_eq!(world.runs.runs.borrow().get(ID), Some(&recorded.run));
    }

    #[test]
    fn a_run_that_cannot_be_saved_is_still_returned() {
        let mut world = World::new();
        world.runs.full = true;
        let (recorded, events) = world.record(&[Scenario::Depth, Scenario::Prefill]);
        let recorded = recorded.unwrap();
        assert_eq!(recorded.run.depths.len(), 4);
        assert_eq!(
            format!("{:#}", recorded.saved.unwrap_err()),
            format!("cannot write /data/bench/{ID}.json: No space left on device (os error 28)")
        );
        let warnings: Vec<&Event> = events
            .iter()
            .filter(|event| matches!(event, Event::Warning(text) if text.contains("saved")))
            .collect();
        assert_eq!(
            warnings,
            [&Event::Warning(format!(
                "the run cannot be saved as it is measured (cannot write /data/bench/{ID}.json: \
                 No space left on device (os error 28)), so a stop before its end loses it"
            ))]
        );
    }

    #[test]
    fn a_run_is_saved_after_each_scenario() {
        let world = World::new();
        world
            .record(&[Scenario::Depth, Scenario::Prefill])
            .0
            .unwrap();
        // Depths, prefill, and whether the GPU's readings are in.
        let writes: Vec<(usize, bool, bool)> = world
            .runs
            .writes
            .borrow()
            .iter()
            .map(|run| (run.depths.len(), run.prefill.is_some(), !run.gpu.is_empty()))
            .collect();
        assert_eq!(
            writes,
            [(4, false, false), (4, true, false), (4, true, true)]
        );
    }

    /// The request of the session's second turn, after the warm-up, the
    /// depths and the prefills, each with the request setting the context
    /// aside, and the session's own.
    const SECOND_TURN: usize = 1 + 4 * (1 + 1 + 2) + 2 * (1 + 1) + 1 + 1;

    #[test]
    fn a_failed_scenario_saves_the_ones_completed_before_it() {
        let mut world = World::new();
        world.chat.fails_at = Some(SECOND_TURN);
        world.server.counters = RefCell::new(vec![counters(0, 0), counters(100, 50)]);
        let scenarios = [Scenario::Depth, Scenario::Prefill, Scenario::Session];
        let recorded = world.record(&scenarios).0.unwrap();
        assert_eq!(format!("{:#}", recorded.failure.unwrap()), PARTIAL);
        assert_eq!(
            recorded.saved.unwrap(),
            PathBuf::from(format!("/data/bench/{ID}.json"))
        );
        // Once after each scenario completed, then once more with the readings.
        assert_eq!(world.runs.writes.borrow().len(), 3);
        let runs = world.runs.runs.borrow();
        let saved = &runs[ID];
        assert_eq!(saved, &recorded.run);
        assert_eq!(saved.depths.len(), 4);
        assert_eq!(saved.prefill.as_ref().unwrap().samples.len(), 2);
        assert_eq!(saved.session, None);
        // The failed request's drafts are unknown: no check.
        assert_eq!(saved.drafts, None);
        let read: Vec<Option<Scenario>> = saved.gpu.iter().map(|r| r.scenario).collect();
        assert_eq!(read, [None, Some(Scenario::Depth), Some(Scenario::Prefill)]);
    }

    /// The failure of the session's second turn, after depth and prefill.
    const PARTIAL: &str = "turn 2 of session 1: answered 500: the slot is busy; the run holds \
                           only the scenarios it completed: depth and prefill";

    #[test]
    fn a_failed_scenario_returns_the_ones_before_it_when_they_cannot_be_saved() {
        let mut world = World::new();
        world.chat.fails_at = Some(SECOND_TURN);
        world.runs.full = true;
        let scenarios = [Scenario::Depth, Scenario::Prefill, Scenario::Session];
        let recorded = world.record(&scenarios).0.unwrap();
        assert_eq!(format!("{:#}", recorded.failure.unwrap()), PARTIAL);
        assert_eq!(
            format!("{:#}", recorded.saved.unwrap_err()),
            format!("cannot write /data/bench/{ID}.json: No space left on device (os error 28)")
        );
        assert_eq!(recorded.run.depths.len(), 4);
        assert_eq!(recorded.run.prefill.unwrap().samples.len(), 2);
        assert_eq!(recorded.run.session, None);
    }

    #[test]
    fn gpu_readings_that_fail_are_counted_in_a_warning() {
        let mut world = World::new();
        let scenarios = [Scenario::Depth, Scenario::Session];
        world.monitor.lost_from = Some(Scenario::Session);
        let (run, events) = world.run(&scenarios);
        // The reading as the run starts, and the depth's.
        assert_eq!(run.unwrap().gpu.len(), 2);
        let warning = Event::Warning(
            "1 GPU reading failed during the run (NVML: the GPU is lost); the run's GPU \
             figures leave it out"
                .into(),
        );
        assert!(events.contains(&warning), "{events:?}");

        world.monitor.lost_from = Some(Scenario::Depth);
        let (run, events) = world.run(&scenarios);
        assert_eq!(run.unwrap().gpu.len(), 1);
        let warning = Event::Warning(
            "2 GPU readings failed during the run (the last: NVML: the GPU is lost); the run's \
             GPU figures leave them out"
                .into(),
        );
        assert!(events.contains(&warning), "{events:?}");
    }

    #[test]
    fn scenarios_are_listed_as_a_sentence_lists_them() {
        assert_eq!(listed(&["depth"]), "depth");
        assert_eq!(listed(&["depth", "prefill"]), "depth and prefill");
        assert_eq!(
            listed(&["depth", "prefill", "session"]),
            "depth, prefill and session"
        );
    }

    #[test]
    fn saved_runs_are_each_read_or_named_with_their_reason() {
        let unreadable = "20261012T080000Z-qwen3.8-27b";
        let runs = FakeRuns {
            unreadable: BTreeMap::from([(unreadable.to_owned(), "format 2 is unknown".to_owned())]),
            ..FakeRuns::default()
        };
        let run = crate::domain::bench::testing::run();
        runs.save(&run).unwrap();
        let listed = saved(&runs).unwrap();
        assert_eq!(listed.len(), 2);
        assert_eq!(listed[0].0, run.id());
        assert_eq!(listed[0].1.as_ref().unwrap(), &run);
        assert_eq!(listed[1].0, unreadable);
        assert_eq!(
            format!("{:#}", listed[1].1.as_ref().unwrap_err()),
            format!("cannot read /data/bench/{unreadable}.json: format 2 is unknown")
        );
    }

    #[test]
    fn runs_load_in_the_order_asked() {
        let runs = FakeRuns::default();
        let first = crate::domain::bench::testing::run();
        let mut second = first.clone();
        second.started = Timestamp::parse("2026-10-12T08:00:00Z").unwrap();
        runs.save(&first).unwrap();
        runs.save(&second).unwrap();
        let loaded = load(&runs, &[second.id(), first.id()]).unwrap();
        assert_eq!(loaded, [second, first.clone()]);
        let error = load(&runs, &[first.id(), "20261013T000000Z-m".into()]).unwrap_err();
        assert_eq!(
            format!("{error:#}"),
            "no run is saved as 20261013T000000Z-m; `ai-ctl bench list` lists them"
        );
    }

    #[test]
    fn a_server_without_its_context_size_fails_the_run() {
        let mut world = World::new();
        world.server.fails = true;
        let error = world.run(&Scenario::ALL).0.unwrap_err();
        assert_eq!(
            format!("{error:#}"),
            "cannot read the context size of qwen3.8-27b: connection refused"
        );
    }

    fn counters(proposed: u64, accepted: u64) -> SpecCounters {
        SpecCounters {
            drafts: Drafts { proposed, accepted },
            steps: proposed / 3,
            accepted_at: vec![accepted / 2, accepted / 3],
        }
    }

    #[test]
    fn the_servers_counters_check_the_drafts_of_the_requests_after_the_warm_up() {
        let world = World::new();
        // Every request after the warm-up: 20 turns of 512 tokens, 100
        // overhead requests of 16 tokens, and the 3 of 16 tokens that set
        // the context aside, before each session and the overhead.
        let sent = FakeChat::drafts(512).proposed * 20 + FakeChat::drafts(16).proposed * 103;
        let accepted = FakeChat::drafts(512).accepted * 20 + FakeChat::drafts(16).accepted * 103;
        let before = counters(5_000, 4_000);
        let after = counters(5_000 + sent, 4_000 + accepted);
        *world.server.counters.borrow_mut() = vec![before.clone(), after.clone()];
        let run = world
            .run(&[Scenario::Session, Scenario::Overhead])
            .0
            .unwrap();
        let check = run.drafts.unwrap();
        assert_eq!(
            check.requests,
            Drafts {
                proposed: sent,
                accepted
            }
        );
        assert!(check.agrees());
        assert_eq!((check.before, check.after), (before, after));
        let calls = world.server.calls.borrow();
        assert_eq!(calls.iter().filter(|c| c.starts_with("metrics")).count(), 2);
    }

    #[test]
    fn without_the_servers_counters_the_run_goes_on_with_a_warning() {
        let world = World::new();
        let (run, events) = world.run(&[Scenario::Prefill]);
        assert_eq!(run.unwrap().drafts, None);
        assert!(
            events.contains(&Event::Warning(
                "cannot read the server's counters, which check the drafts \
             (answered 501: metrics are disabled)"
                    .into()
            ))
        );
        // A first reading but no second: nothing to check.
        let world = World::new();
        *world.server.counters.borrow_mut() = vec![counters(0, 0)];
        let (run, events) = world.run(&[Scenario::Prefill]);
        assert_eq!(run.unwrap().drafts, None);
        assert_eq!(
            events
                .iter()
                .filter(|event| matches!(event, Event::Warning(_)))
                .count(),
            1
        );
    }

    #[test]
    fn the_gpu_is_read_through_the_scenarios_each_reading_labelled() {
        let world = World::new();
        let run = world
            .run(&[Scenario::Session, Scenario::Prefill])
            .0
            .unwrap();
        // One reading as the monitor starts, after the warm-up, then one
        // as each scenario starts, in the order they run.
        let labels: Vec<Option<Scenario>> = run.gpu.iter().map(|r| r.scenario).collect();
        assert_eq!(
            labels,
            [None, Some(Scenario::Prefill), Some(Scenario::Session)]
        );
        assert!(world.monitor.finished.get());
        assert_eq!(run.gpu(None).unwrap().peak_memory_mib, 3_000);
    }

    #[test]
    fn without_the_gpu_the_run_goes_on_with_a_warning() {
        let mut world = World::new();
        world.monitor.fails = true;
        let (run, events) = world.run(&[Scenario::Prefill]);
        assert!(run.unwrap().gpu.is_empty());
        assert!(
            events.contains(&Event::Warning(
                "the run goes on without the GPU's memory, temperature, clock and power \
             (NVML: driver not loaded)"
                    .into()
            ))
        );
    }

    #[test]
    fn a_run_records_what_the_gateway_runs_the_model_with() {
        let world = World::new();
        let setup = world.run(&[Scenario::Prefill]).0.unwrap().setup;
        assert_eq!(setup.model, model());
        assert_eq!(setup.source, Source::Build(b11538()));
        assert_eq!(
            setup.command[0],
            "/data/builds/llama.cpp/b11538/llama-server"
        );
        assert!(
            setup.command.contains(&"--ctx-size 98304".to_owned()),
            "{:?}",
            setup.command
        );
        assert_eq!(setup.machine.gpu, "NVIDIA GeForce RTX 5090 Laptop GPU");
        assert_eq!(
            (setup.machine.gpu_memory_mib, setup.machine.driver.as_str()),
            (24_463, "615.71.09")
        );
        assert_eq!(setup.machine.power_profile.as_deref(), Some("performance"));
        assert_eq!(setup.machine.ac_power, Some(true));
        assert_eq!(setup.ai_ctl, env!("CARGO_PKG_VERSION"));
    }

    #[test]
    fn a_run_reads_the_gpu_once() {
        let world = World::new();
        world.gpu.reads.set(0);
        // A GPU lost after the configuration was sized to it is not read
        // again for the run's record.
        world
            .gpu
            .readings
            .replace(vec![Probed::Found(gpu()), Probed::Missing]);
        let (run, _) = world.run(&[Scenario::Prefill]);
        assert_eq!(world.gpu.reads.get(), 1);
        assert_eq!(
            run.unwrap().setup.machine.gpu,
            "NVIDIA GeForce RTX 5090 Laptop GPU"
        );
    }

    /// The fake inventory, its model run by the image `image`.
    fn on_image(image: &str) -> World {
        let mut parts = inventory_parts("/models");
        parts.backends.push(Backend {
            id: "llama-image".into(),
            kind: BackendKind::LlamaServer,
            source: BackendSource::Image(image.into()),
        });
        parts.models[0].backend = "llama-image".into();
        let mut world = World::new();
        world.inventory = FakeInventory(Ok(Some(Inventory::new(parts).unwrap())));
        let config = world.configurator().generate().unwrap().to_yaml();
        world.gateway.config.replace(Some(config));
        world
    }

    #[test]
    fn an_image_is_recorded_with_its_digest_when_podman_gives_it() {
        const IMAGE: &str = "ghcr.io/ggml-org/llama.cpp:server-cuda13";
        let mut world = on_image(IMAGE);
        world.containers.local = vec![IMAGE.to_owned()];
        let setup = world.run(&[Scenario::Prefill]).0.unwrap().setup;
        assert_eq!(
            setup.source,
            Source::Image {
                reference: IMAGE.into(),
                digest: Some(format!("sha256:{}", "ab".repeat(32))),
            }
        );
        assert!(
            setup.command[0].starts_with("podman run"),
            "{:?}",
            setup.command
        );
        let world = on_image(IMAGE);
        let (run, events) = world.run(&[Scenario::Prefill]);
        assert_eq!(
            run.unwrap().setup.source,
            Source::Image {
                reference: IMAGE.into(),
                digest: None
            }
        );
        assert!(events.contains(&Event::Warning(format!(
            "the run records the image without its digest \
             (podman image inspect {IMAGE}: no such image)"
        ))));
    }

    #[test]
    fn a_gateway_on_another_configuration_is_refused_before_anything_is_sent() {
        let world = World::new();
        world.gateway.config.replace(Some("models: {}\n".into()));
        let error = world.run(&Scenario::ALL).0.unwrap_err().to_string();
        assert_eq!(
            error,
            "the gateway runs another configuration than the inventory and the builds give now; \
             restart it with `ai-ctl down` and `ai-ctl up` so that the run records what it \
             measures"
        );
        assert!(world.server.calls.borrow().is_empty());
        assert!(world.chat.requests.borrow().is_empty());
    }

    #[test]
    fn a_model_of_another_server_than_llama_server_is_refused_before_any_request() {
        let mut parts = inventory_parts("/models");
        parts.backends.push(Backend {
            id: "whisper-native".into(),
            kind: BackendKind::WhisperServer,
            source: BackendSource::Build("whisper.cpp".into()),
        });
        let mut whisper = model();
        whisper.id = "whisper-large-v3".into();
        whisper.backend = "whisper-native".into();
        whisper.dir = "stt/whisper-large-v3".into();
        whisper.files = BTreeMap::from([("model".into(), "ggml-large-v3.bin".into())]);
        whisper.ctx_size = None;
        whisper.vram_mib = 1000;
        whisper.preload = false;
        parts.models.push(whisper);
        let mut world = World::new();
        world.inventory = FakeInventory(Ok(Some(Inventory::new(parts).unwrap())));
        let v1_9_5 = crate::domain::build::BuildId::parse("v1.9.5").unwrap();
        world.builds.builds.push(crate::domain::build::Build {
            project: "whisper.cpp".into(),
            cmake_flags: crate::application::fakes::recipe_flags(
                &crate::domain::recipe::WHISPER_CPP,
            ),
            id: v1_9_5.clone(),
            patches: vec!["api-key.patch".into(), "reuse-address.patch".into()],
            ..b11538()
        });
        world
            .builds
            .other_currents
            .insert("whisper.cpp".into(), v1_9_5);
        let config = world.configurator().generate().unwrap().to_yaml();
        world.gateway.config.replace(Some(config));
        let options = Options {
            model: "whisper-large-v3".into(),
            repeat: 1,
            scenarios: Scenario::ALL.to_vec(),
        };
        let error = world.bench().run(&options, &mut |_| {}).unwrap_err();
        assert_eq!(
            error.to_string(),
            "whisper-large-v3 runs on whisper-server; the bench measures llama-server models only"
        );
        assert!(world.server.calls.borrow().is_empty());
        assert!(world.chat.requests.borrow().is_empty());
    }

    #[test]
    fn a_model_the_inventory_lacks_is_refused() {
        let world = World::new();
        let options = Options {
            model: "qwen3.6-35b-a3b".into(),
            repeat: 1,
            scenarios: Scenario::ALL.to_vec(),
        };
        let error = world.bench().run(&options, &mut |_| {}).unwrap_err();
        assert_eq!(
            error.to_string(),
            "the inventory has no model qwen3.6-35b-a3b; it has qwen3.8-27b"
        );
    }

    #[test]
    fn steps_are_named_as_errors_quote_them() {
        let depth = Depth {
            tokens: 8_192,
            at_limit: false,
        };
        let named: Vec<String> = [
            Step::WarmUp,
            Step::Fill(depth),
            Step::Prefill { repetition: 2 },
            Step::Turn {
                session: 1,
                turn: 4,
            },
            Step::SetAside,
            Step::Overhead {
                request: 3,
                direct: true,
            },
            Step::Overhead {
                request: 3,
                direct: false,
            },
        ]
        .iter()
        .map(Step::to_string)
        .collect();
        assert_eq!(
            named,
            [
                "the warm-up request",
                "the fill at 8192 tokens",
                "prefill 2",
                "turn 4 of session 1",
                "the request that sets the context aside",
                "overhead request 3 straight to llama-server",
                "overhead request 3 through the gateway",
            ]
        );
    }

    #[test]
    fn the_overhead_goes_to_the_measured_models_own_server() {
        let mut world = World::new();
        let other: SocketAddr = "127.0.0.1:5800".parse().unwrap();
        world.api.models = Some(vec![
            RunningModel {
                id: "qwen3.6-35b-a3b".into(),
                state: "ready".into(),
                proxy: Some(other),
            },
            RunningModel {
                id: MODEL.into(),
                state: "ready".into(),
                proxy: Some(own()),
            },
        ]);
        world.run(&[Scenario::Overhead]).0.unwrap();
        let sent = world.sent();
        assert!(sent.iter().all(|(addr, _)| *addr != other), "{sent:?}");
        assert_eq!(sent.iter().filter(|(addr, _)| *addr == own()).count(), 50);
    }

    #[test]
    fn a_gpu_reading_that_cannot_be_finished_is_a_warning() {
        let mut world = World::new();
        world.monitor.finish_fails = true;
        let (run, events) = world.run(&[Scenario::Prefill]);
        assert!(run.unwrap().gpu.is_empty());
        assert!(
            events.contains(&Event::Warning("the GPU's reading failed".into())),
            "{events:?}"
        );
    }
}
