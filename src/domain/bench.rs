//! The bench: what a run sends through the gateway, scenario by scenario,
//! and what each request gives back.
//!
//! The scenarios follow the measurement protocol of `docs/stack.md` §9. Each
//! request is measured twice: by the client, which times the stream, and by
//! llama-server, whose `timings` object tells how many prompt tokens came from
//! the cache and how fast the rest went.

use std::fmt;
use std::iter::Sum;
use std::ops::Add;
use std::time::Duration;

use crate::domain::build::Build;
use crate::domain::inventory::Model;
use crate::domain::machine::Version;
use crate::domain::stats::Summary;
use crate::domain::timestamp::Timestamp;

/// The scenarios of a run, in the order it takes them.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Scenario {
    /// Generation speed at several context depths, each context prefilled
    /// once so the repetitions measure generation only.
    Depth,
    /// Prefill speed on a prompt the cache has never seen: an agent's first
    /// request.
    Prefill,
    /// An agent's session, its history sent back at every turn: the reuse of
    /// the prompt cache.
    Session,
    /// The gateway's own cost: the same request through llama-swap and
    /// straight to llama-server.
    Overhead,
}

impl Scenario {
    pub const ALL: [Self; 4] = [Self::Depth, Self::Prefill, Self::Session, Self::Overhead];

    pub fn name(self) -> &'static str {
        match self {
            Self::Depth => "depth",
            Self::Prefill => "prefill",
            Self::Session => "session",
            Self::Overhead => "overhead",
        }
    }

    /// The scenario named `name`, as [`Scenario::name`] gives it.
    pub fn from_name(name: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|scenario| scenario.name() == name)
    }
}

impl fmt::Display for Scenario {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

/// The depths at which generation is measured when they fit below the
/// context's limit: 8K, 32K and 64K tokens.
pub const DEPTHS: [u32; 3] = [8_192, 32_768, 65_536];
/// Tokens each request of the depth and session scenarios generates.
pub const GENERATE: u32 = 512;
/// The fresh prompt of the prefill scenario: 40K tokens, the size of an
/// agent's first request (`docs/stack.md` §9).
pub const PREFILL: u32 = 40_960;
/// Tokens a prefill request generates: one, since only the time to it counts.
pub const PREFILL_GENERATE: u32 = 1;
/// How many times each measure is made, by default.
pub const REPEAT: u32 = 5;
/// Requests the overhead scenario sends each way: the difference it measures
/// is in milliseconds, against seconds for the other scenarios.
pub const OVERHEAD_REQUESTS: u32 = 50;
/// Tokens each request of the overhead scenario generates.
pub const OVERHEAD_GENERATE: u32 = 16;
/// Tokens of the context a request leaves free beyond those it generates.
/// Without context shift, llama-server ends a generation once the context
/// holds all but one token (`prompt.n_tokens() + 1 >= n_ctx` in
/// `tools/server/server-context.cpp`): a prompt that leaves room for the
/// tokens to generate and no more may get one fewer.
pub const FREE: u32 = 1;

/// A context depth at which generation is measured.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Depth {
    /// Tokens of the prompt.
    pub tokens: u32,
    /// Whether it is the context's limit: its size less the tokens to
    /// generate.
    pub at_limit: bool,
}

/// The shape of an agent's session.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SessionPlan {
    /// Turns of a session.
    pub turns: u32,
    /// Tokens of the first prompt: an agent's system prompt and task.
    pub opening: u32,
    /// Tokens each later turn adds after the reply: a tool's output, a
    /// message.
    pub turn: u32,
}

impl SessionPlan {
    /// Ten turns, opening on 8K tokens and adding 1K at each turn.
    pub const DEFAULT: Self = Self {
        turns: 10,
        opening: 8_192,
        turn: 1_024,
    };

    /// The most context a session can fill: its last prompt and reply, every
    /// reply at full length and kept in the history. Each later turn adds
    /// `turn_extra` tokens besides its code and the reply before it: its own
    /// text, and what the model's template puts around the two messages.
    pub fn peak(&self, generate: u32, turn_extra: u32) -> u64 {
        u64::from(self.opening)
            + u64::from(self.turns.saturating_sub(1))
                * (u64::from(self.turn) + u64::from(turn_extra))
            + u64::from(self.turns) * u64::from(generate)
    }
}

/// What a run sends, scenario by scenario.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Plan {
    /// How many times each measure is made.
    pub repeat: u32,
    /// Tokens each request of the depth and session scenarios generates.
    pub generate: u32,
    /// Ascending, the last one at the context's limit.
    pub depths: Vec<Depth>,
    /// Tokens of the prefill scenario's prompt.
    pub prefill: u32,
    pub session: SessionPlan,
    /// Requests the overhead scenario sends each way.
    pub overhead: u32,
}

/// Why no plan fits a model.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum PlanError {
    #[error("each measure must be made at least once")]
    NoRepetition,
    #[error(
        "the {scenario} scenario needs a context of {needed} tokens; the model's holds {ctx_size}"
    )]
    ContextTooSmall {
        scenario: Scenario,
        needed: u64,
        ctx_size: u32,
    },
}

impl Plan {
    /// The default plan for a model whose context holds `ctx_size` tokens,
    /// each measure made `repeat` times. Every request leaves [`FREE`]
    /// tokens of the context free beyond those it generates. The prefill
    /// prompt shrinks to the context's limit when it does not fit; a
    /// session among `scenarios` that does not fit is an error, each of its
    /// later turns adding `turn_extra` tokens (see [`SessionPlan::peak`]).
    pub fn new(
        ctx_size: u32,
        repeat: u32,
        scenarios: &[Scenario],
        turn_extra: u32,
    ) -> Result<Self, PlanError> {
        if repeat == 0 {
            return Err(PlanError::NoRepetition);
        }
        let limit = ctx_size.saturating_sub(GENERATE + FREE);
        if limit == 0 {
            return Err(PlanError::ContextTooSmall {
                scenario: Scenario::Depth,
                needed: u64::from(GENERATE + FREE) + 1,
                ctx_size,
            });
        }
        let session = SessionPlan::DEFAULT;
        let needed = session.peak(GENERATE, turn_extra) + u64::from(FREE);
        if scenarios.contains(&Scenario::Session) && needed > u64::from(ctx_size) {
            return Err(PlanError::ContextTooSmall {
                scenario: Scenario::Session,
                needed,
                ctx_size,
            });
        }
        Ok(Self {
            repeat,
            generate: GENERATE,
            depths: depths(limit),
            prefill: PREFILL.min(ctx_size - PREFILL_GENERATE - FREE),
            session,
            overhead: OVERHEAD_REQUESTS,
        })
    }
}

/// [`DEPTHS`] below `limit`, then `limit` itself.
fn depths(limit: u32) -> Vec<Depth> {
    DEPTHS
        .iter()
        .filter(|&&tokens| tokens < limit)
        .map(|&tokens| Depth {
            tokens,
            at_limit: false,
        })
        .chain([Depth {
            tokens: limit,
            at_limit: true,
        }])
        .collect()
}

/// Who speaks a message.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Role {
    System,
    User,
    Assistant,
}

impl Role {
    /// The name chat completions give the role.
    pub fn name(self) -> &'static str {
        match self {
            Self::System => "system",
            Self::User => "user",
            Self::Assistant => "assistant",
        }
    }
}

/// A message of a conversation the bench sends.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Message {
    pub role: Role,
    pub content: String,
    /// An assistant's reasoning, sent back in the history as OpenCode does.
    pub reasoning: Option<String>,
}

impl Message {
    pub fn system(content: impl Into<String>) -> Self {
        Self {
            role: Role::System,
            content: content.into(),
            reasoning: None,
        }
    }

    pub fn user(content: impl Into<String>) -> Self {
        Self {
            role: Role::User,
            content: content.into(),
            reasoning: None,
        }
    }

    /// An assistant's reply, with its reasoning when there was any.
    pub fn assistant(content: impl Into<String>, reasoning: impl Into<String>) -> Self {
        let reasoning = reasoning.into();
        Self {
            role: Role::Assistant,
            content: content.into(),
            reasoning: (!reasoning.is_empty()).then_some(reasoning),
        }
    }
}

/// What the client timed of one streamed request.
#[derive(Debug, Clone, PartialEq)]
pub struct ClientTimings {
    /// From sending the request to the first content or reasoning token.
    pub ttft: Duration,
    /// From sending the request to the end of the stream.
    pub total: Duration,
    /// The gap before each token after the first, zero for tokens that
    /// arrived together; see [`arrival_gaps`].
    pub gaps: Vec<Duration>,
}

/// Draft tokens of speculative decoding (MTP): proposed, and accepted by the
/// model.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Drafts {
    pub proposed: u64,
    pub accepted: u64,
}

impl Drafts {
    /// The share of proposed tokens accepted; `None` when none was proposed.
    pub fn acceptance(self) -> Option<f64> {
        (self.proposed > 0).then(|| self.accepted as f64 / self.proposed as f64)
    }
}

impl Add for Drafts {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self {
            proposed: self.proposed + other.proposed,
            accepted: self.accepted + other.accepted,
        }
    }
}

impl Sum for Drafts {
    fn sum<I: Iterator<Item = Self>>(drafts: I) -> Self {
        drafts.fold(Self::default(), Add::add)
    }
}

/// llama-server's counters of speculative decoding, from its `/metrics`,
/// since it started.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SpecCounters {
    pub drafts: Drafts,
    /// Decoding steps that checked a draft.
    pub steps: u64,
    /// For each position of a draft, from the first, the steps that
    /// accepted it.
    pub accepted_at: Vec<u64>,
}

impl SpecCounters {
    /// What the server counted from `before` to these counters.
    pub fn since(&self, before: &Self) -> Self {
        Self {
            drafts: Drafts {
                proposed: self.drafts.proposed.saturating_sub(before.drafts.proposed),
                accepted: self.drafts.accepted.saturating_sub(before.drafts.accepted),
            },
            steps: self.steps.saturating_sub(before.steps),
            accepted_at: self
                .accepted_at
                .iter()
                .enumerate()
                .map(|(position, &count)| {
                    count.saturating_sub(before.accepted_at.get(position).copied().unwrap_or(0))
                })
                .collect(),
        }
    }

    /// For each position of a draft, the share of steps that accepted it.
    pub fn acceptance_at(&self) -> Vec<f64> {
        if self.steps == 0 {
            return Vec::new();
        }
        self.accepted_at
            .iter()
            .map(|&count| count as f64 / self.steps as f64)
            .collect()
    }

    /// Tokens each step produced: the one it decoded and the drafts it
    /// accepted.
    pub fn tokens_per_step(&self) -> Option<f64> {
        (self.steps > 0).then(|| 1.0 + self.drafts.accepted as f64 / self.steps as f64)
    }
}

/// How far a counter of `/metrics` may be from its true value:
/// llama-server writes a counter with six significant digits, so from a
/// million on it is rounded.
pub fn rounding(counter: u64) -> u64 {
    let mut unit = 1;
    let mut value = counter;
    while value >= 1_000_000 {
        value /= 10;
        unit *= 10;
    }
    unit / 2
}

/// MTP's drafts over a run: as each request reported them, and as the
/// server's counters, read before and after, add them up.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DraftCheck {
    /// Every request sent between the two readings, measured or not.
    pub requests: Drafts,
    pub before: SpecCounters,
    pub after: SpecCounters,
}

impl DraftCheck {
    /// What the server counted over the run.
    pub fn server(&self) -> SpecCounters {
        self.after.since(&self.before)
    }

    /// Whether the requests account for every draft the server counted, to
    /// the rounding of its counters: no other client used the model, and
    /// no request's `timings` went unread.
    pub fn agrees(&self) -> bool {
        let server = self.server().drafts;
        let close = |counted: u64, reported: u64, before: u64, after: u64| {
            counted.abs_diff(reported) <= rounding(before) + rounding(after)
        };
        close(
            server.proposed,
            self.requests.proposed,
            self.before.drafts.proposed,
            self.after.drafts.proposed,
        ) && close(
            server.accepted,
            self.requests.accepted,
            self.before.drafts.accepted,
            self.after.drafts.accepted,
        )
    }
}

/// llama-server's `timings` object for one request.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ServerTimings {
    /// Prompt tokens taken from the cache.
    pub cache_n: u64,
    /// Prompt tokens evaluated.
    pub prompt_n: u64,
    pub prompt_ms: f64,
    /// Tokens generated.
    pub predicted_n: u64,
    pub predicted_ms: f64,
    /// `None` when no draft was proposed: llama-server leaves `draft_n` and
    /// `draft_n_accepted` out then.
    pub drafts: Option<Drafts>,
}

impl ServerTimings {
    /// Prompt tokens evaluated per second.
    pub fn prefill_speed(&self) -> Option<f64> {
        rate(self.prompt_n, self.prompt_ms)
    }

    /// Tokens generated per second.
    pub fn generation_speed(&self) -> Option<f64> {
        rate(self.predicted_n, self.predicted_ms)
    }
}

/// `count` per second over `ms` milliseconds; `None` for no count or no time.
fn rate(count: u64, ms: f64) -> Option<f64> {
    (count > 0 && ms > 0.0).then(|| count as f64 * 1000.0 / ms)
}

/// The `usage` object of an OpenAI-compatible response.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Usage {
    pub prompt_tokens: u64,
    pub completion_tokens: u64,
}

/// One measured request.
#[derive(Debug, Clone, PartialEq)]
pub struct Sample {
    pub client: ClientTimings,
    /// `None` from a server that does not send llama-server's `timings`.
    pub server: Option<ServerTimings>,
    pub usage: Option<Usage>,
}

impl Sample {
    pub fn ttft_ms(&self) -> f64 {
        millis(self.client.ttft)
    }

    pub fn total_ms(&self) -> f64 {
        millis(self.client.total)
    }

    /// Tokens generated per second: the server's figure, or else the client's,
    /// from the tokens after the first over the time after it.
    pub fn generation_speed(&self) -> Option<f64> {
        match self.server {
            Some(server) => server.generation_speed(),
            None => {
                let usage = self.usage?;
                let after_first = self.client.total.checked_sub(self.client.ttft)?;
                rate(
                    usage.completion_tokens.saturating_sub(1),
                    millis(after_first),
                )
            }
        }
    }

    /// Prompt tokens evaluated per second, as the server reports it.
    pub fn prefill_speed(&self) -> Option<f64> {
        self.server?.prefill_speed()
    }

    /// Prompt tokens taken from the cache.
    pub fn cached(&self) -> Option<u64> {
        self.server.map(|server| server.cache_n)
    }

    /// Prompt tokens evaluated.
    pub fn evaluated(&self) -> Option<u64> {
        self.server.map(|server| server.prompt_n)
    }
}

fn millis(duration: Duration) -> f64 {
    duration.as_secs_f64() * 1000.0
}

/// Tokens closer than this arrived together: a decoding step takes tens of
/// milliseconds, and the drafts it accepts are sent at once.
pub const BURST: Duration = Duration::from_millis(1);

/// The gaps between the arrivals of tokens in `samples`, in milliseconds.
/// With speculative decoding, tokens come in bursts, one per decoding step
/// (on the 27B with MTP, two gaps in three were under a millisecond): a
/// gap under [`BURST`] continues a burst and is left out, so the median is
/// the time of a step and the p99 its stalls.
pub fn arrival_gaps(samples: &[Sample]) -> Option<Summary> {
    Summary::of(
        samples
            .iter()
            .flat_map(|sample| sample.client.gaps.iter())
            .filter(|&&gap| gap >= BURST)
            .map(|&gap| millis(gap)),
    )
}

/// The drafts of `samples` together; `None` when none reported any.
pub fn drafts(samples: &[Sample]) -> Option<Drafts> {
    let reported: Vec<Drafts> = samples
        .iter()
        .filter_map(|sample| sample.server?.drafts)
        .collect();
    (!reported.is_empty()).then(|| reported.into_iter().sum())
}

/// Generation at one depth.
#[derive(Debug, Clone, PartialEq)]
pub struct DepthResult {
    pub depth: Depth,
    /// The request that filled the context: a fresh prefill at this depth.
    pub fill: Sample,
    /// The repetitions on the filled context.
    pub samples: Vec<Sample>,
}

impl DepthResult {
    /// Tokens generated per second.
    pub fn generation(&self) -> Option<Summary> {
        Summary::of(self.samples.iter().filter_map(Sample::generation_speed))
    }

    /// Time to the first token, the context in the cache, in milliseconds.
    pub fn ttft(&self) -> Option<Summary> {
        Summary::of(self.samples.iter().map(Sample::ttft_ms))
    }
}

/// Prefills of a fresh prompt.
#[derive(Debug, Clone, PartialEq)]
pub struct PrefillResult {
    pub samples: Vec<Sample>,
}

impl PrefillResult {
    /// Prompt tokens evaluated per second.
    pub fn prefill(&self) -> Option<Summary> {
        Summary::of(self.samples.iter().filter_map(Sample::prefill_speed))
    }

    /// Time to the first token, in milliseconds.
    pub fn ttft(&self) -> Option<Summary> {
        Summary::of(self.samples.iter().map(Sample::ttft_ms))
    }
}

/// Sessions of the session scenario.
#[derive(Debug, Clone, PartialEq)]
pub struct SessionResult {
    /// Each session's turns, in order.
    pub sessions: Vec<Vec<Sample>>,
}

/// One turn across the sessions of a run.
#[derive(Debug, Clone, PartialEq)]
pub struct TurnSummary {
    /// From 1.
    pub turn: usize,
    /// Time to the first token, in milliseconds.
    pub ttft: Option<Summary>,
    /// Prompt tokens taken from the cache.
    pub cached: Option<Summary>,
    /// Prompt tokens evaluated.
    pub evaluated: Option<Summary>,
}

impl SessionResult {
    /// Each turn, summarised across the sessions that reached it.
    pub fn turns(&self) -> Vec<TurnSummary> {
        let turns = self.sessions.iter().map(Vec::len).max().unwrap_or(0);
        (0..turns)
            .map(|index| {
                let samples: Vec<&Sample> = self
                    .sessions
                    .iter()
                    .filter_map(|session| session.get(index))
                    .collect();
                let tokens = |count: fn(&Sample) -> Option<u64>| {
                    Summary::of(samples.iter().filter_map(|s| count(s)).map(|n| n as f64))
                };
                TurnSummary {
                    turn: index + 1,
                    ttft: Summary::of(samples.iter().map(|s| s.ttft_ms())),
                    cached: tokens(Sample::cached),
                    evaluated: tokens(Sample::evaluated),
                }
            })
            .collect()
    }

    /// Whether every session took more tokens from the cache at each turn
    /// than at the one before: the cache reused across turns. `false` when a
    /// turn has no server timings, or no session has two turns.
    pub fn cache_grows(&self) -> bool {
        let mut sessions = self.sessions.iter().filter(|s| s.len() > 1).peekable();
        sessions.peek().is_some()
            && sessions.all(|session| {
                let cached: Option<Vec<u64>> = session.iter().map(Sample::cached).collect();
                cached.is_some_and(|cached| cached.windows(2).all(|pair| pair[1] > pair[0]))
            })
    }
}

/// The same request, sent through the gateway and straight to llama-server.
#[derive(Debug, Clone, PartialEq)]
pub struct OverheadResult {
    pub gateway: Vec<Sample>,
    pub direct: Vec<Sample>,
}

/// What the gateway adds to a request, in milliseconds: the difference of the
/// medians.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Overhead {
    pub ttft_ms: f64,
    pub total_ms: f64,
}

impl OverheadResult {
    /// The median of `measure` through the gateway, then straight to
    /// llama-server; `None` when either way has no request.
    pub fn medians(&self, measure: fn(&Sample) -> f64) -> Option<(f64, f64)> {
        let median = |samples: &[Sample]| {
            Summary::of(samples.iter().map(measure)).map(|summary| summary.median)
        };
        Some((median(&self.gateway)?, median(&self.direct)?))
    }

    pub fn added(&self) -> Option<Overhead> {
        let added = |measure| {
            self.medians(measure)
                .map(|(gateway, direct)| gateway - direct)
        };
        Some(Overhead {
            ttft_ms: added(Sample::ttft_ms)?,
            total_ms: added(Sample::total_ms)?,
        })
    }
}

/// What slowed the GPU's clocks down at a reading, as NVML reports it.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Slowdown {
    /// The power limit: the usual state of a laptop's GPU under load.
    pub power: bool,
    /// The GPU's temperature, as the driver or the hardware sets it.
    pub thermal: bool,
    /// The hardware's own slowdown or power brake, from an external
    /// signal: a power supply that cannot follow, say.
    pub hardware: bool,
    /// A policy of the board or the driver: NVML's BoardLimit or
    /// Reliability. This machine's GPU reports Reliability at idle, so it
    /// is told apart from the slowdowns above.
    pub policy: bool,
}

/// The GPU at one instant of a run.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GpuReading {
    /// The scenario under way; `None` before the first one.
    pub scenario: Option<Scenario>,
    pub memory_used_mib: u64,
    pub temperature_c: u32,
    pub sm_clock_mhz: u32,
    pub power_w: f64,
    pub slowdown: Slowdown,
    /// NVML's clock-event reasons, every bit as it reported them; `None` in
    /// a run saved before ai-ctl kept them.
    // LEGACY: `None` only in the runs of format 1, which must stay readable
    // for `bench list` and `bench compare`; the `Option` goes with the run
    // store's read of that format.
    pub clock_events: Option<u64>,
}

/// The GPU over a part of a run.
#[derive(Debug, Clone, PartialEq)]
pub struct GpuSummary {
    pub peak_memory_mib: u64,
    pub max_temperature_c: u32,
    pub sm_clock_mhz: Summary,
    pub power_w: Summary,
    /// The share of readings slowed down by each cause.
    pub power_capped: f64,
    pub thermal: f64,
    pub hardware: f64,
    /// The share of readings a board or reliability policy limited.
    pub policy: f64,
}

impl GpuSummary {
    /// The summary of `readings`; `None` when there is none.
    pub fn of<'a>(readings: impl IntoIterator<Item = &'a GpuReading>) -> Option<Self> {
        let readings: Vec<&GpuReading> = readings.into_iter().collect();
        let share = |slowed: fn(&Slowdown) -> bool| {
            readings.iter().filter(|r| slowed(&r.slowdown)).count() as f64 / readings.len() as f64
        };
        Some(Self {
            peak_memory_mib: readings.iter().map(|r| r.memory_used_mib).max()?,
            max_temperature_c: readings.iter().map(|r| r.temperature_c).max()?,
            sm_clock_mhz: Summary::of(readings.iter().map(|r| f64::from(r.sm_clock_mhz)))?,
            power_w: Summary::of(readings.iter().map(|r| r.power_w))?,
            power_capped: share(|s| s.power),
            thermal: share(|s| s.thermal),
            hardware: share(|s| s.hardware),
            policy: share(|s| s.policy),
        })
    }
}

/// Where the measured model's server comes from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Source {
    /// An installed native build of its backend.
    Build(Build),
    /// A container image, with its digest when podman gives one.
    Image {
        reference: String,
        digest: Option<String>,
    },
}

/// The machine a run measured on.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Machine {
    pub gpu: String,
    pub gpu_memory_mib: u64,
    pub driver: String,
    pub cuda: Version,
    /// The power profile, such as `performance`; `None` when it cannot be
    /// read.
    pub power_profile: Option<String>,
    /// Whether the machine ran on an external power supply; `None` when it
    /// cannot be read.
    pub ac_power: Option<bool>,
}

/// What a run measured: the model as the gateway serves it, and where.
#[derive(Debug, Clone, PartialEq)]
pub struct Setup {
    /// The model's inventory entry.
    pub model: Model,
    pub source: Source,
    /// The command llama-swap starts the model's server with, one line
    /// each for the program and for each option with its value, such as
    /// `--ctx-size 98304`; `${PORT}` is left as it is.
    pub command: Vec<String>,
    pub machine: Machine,
    /// The version of ai-ctl that made the run.
    pub ai_ctl: String,
}

/// One model measured, scenario by scenario.
#[derive(Debug, Clone, PartialEq)]
pub struct Run {
    /// When the run started.
    pub started: Timestamp,
    pub model: String,
    pub setup: Setup,
    /// The server's context size, which set the plan.
    pub n_ctx: u32,
    pub plan: Plan,
    /// Generation at each depth, ascending; empty when the scenario did not
    /// run.
    pub depths: Vec<DepthResult>,
    pub prefill: Option<PrefillResult>,
    pub session: Option<SessionResult>,
    pub overhead: Option<OverheadResult>,
    /// `None` when the server's `/metrics` could not be read, or a failed
    /// request ended the run.
    pub drafts: Option<DraftCheck>,
    /// The GPU read at a fixed period from the first scenario on; empty
    /// when it could not be read.
    pub gpu: Vec<GpuReading>,
}

impl Run {
    /// The name a saved run goes by: when it started, then the model, such
    /// as `20261011T154305Z-qwen3.8-27b`, so that names sort in time.
    pub fn id(&self) -> String {
        let started: String = self
            .started
            .to_string()
            .chars()
            .filter(|c| !matches!(c, '-' | ':'))
            .collect();
        format!("{started}-{}", self.model)
    }

    /// Whether the run holds results of `scenario`.
    pub fn holds(&self, scenario: Scenario) -> bool {
        match scenario {
            Scenario::Depth => !self.depths.is_empty(),
            Scenario::Prefill => self.prefill.is_some(),
            Scenario::Session => self.session.is_some(),
            Scenario::Overhead => self.overhead.is_some(),
        }
    }

    /// The scenarios the run holds results of, in the order a run takes
    /// them.
    pub fn scenarios(&self) -> Vec<Scenario> {
        Scenario::ALL
            .into_iter()
            .filter(|&scenario| self.holds(scenario))
            .collect()
    }

    /// The GPU over `scenario`, or over the whole run for `None`.
    pub fn gpu(&self, scenario: Option<Scenario>) -> Option<GpuSummary> {
        GpuSummary::of(
            self.gpu
                .iter()
                .filter(|reading| scenario.is_none() || reading.scenario == scenario),
        )
    }

    /// The GPU over the readings taken during `scenarios`.
    pub fn gpu_over(&self, scenarios: &[Scenario]) -> Option<GpuSummary> {
        GpuSummary::of(self.gpu.iter().filter(|reading| {
            reading
                .scenario
                .is_some_and(|scenario| scenarios.contains(&scenario))
        }))
    }
}

/// The scenarios every one of `runs` holds results of, in the order a run
/// takes them: the part of the runs whose GPU readings compare.
pub fn common_scenarios(runs: &[Run]) -> Vec<Scenario> {
    Scenario::ALL
        .into_iter()
        .filter(|&scenario| runs.iter().all(|run| run.holds(scenario)))
        .collect()
}

/// What the server counted over each of `runs`, when the counts compare:
/// they cover a whole run, so only runs of the same scenarios compare.
/// `None` when the runs' scenarios differ; `None` in the list for a run
/// without counters.
pub fn comparable_counters(runs: &[Run]) -> Option<Vec<Option<SpecCounters>>> {
    let scenarios = runs.first()?.scenarios();
    if runs.iter().any(|run| run.scenarios() != scenarios) {
        return None;
    }
    Some(
        runs.iter()
            .map(|run| run.drafts.as_ref().map(DraftCheck::server))
            .collect(),
    )
}

/// Values for the tests of every layer.
#[cfg(test)]
pub mod testing {
    use std::collections::BTreeMap;

    use super::*;

    /// The 27B on a native build, as the shipped inventory has it.
    pub fn setup() -> Setup {
        Setup {
            model: Model {
                id: "qwen3.8-27b".into(),
                backend: "llama-native".into(),
                repo: Some("unsloth/Qwen3.8-27B-GGUF".into()),
                dir: "llm/qwen3.8-27b".into(),
                files: BTreeMap::from([("model".into(), "Qwen3.8-27B-UD-Q4_K_M.gguf".into())]),
                flags: BTreeMap::new(),
                ctx_size: Some(98_304),
                vram_mib: 22_733,
                ttl: Some(1800),
                preload: true,
                audiocpp: Default::default(),
            },
            source: Source::Image {
                reference: "ghcr.io/ggml-org/llama.cpp:server-cuda-b11538".into(),
                digest: None,
            },
            command: ["llama-server", "--port ${PORT}", "--ctx-size 98304"]
                .map(String::from)
                .to_vec(),
            machine: Machine {
                gpu: "NVIDIA GeForce RTX 5090 Laptop GPU".into(),
                gpu_memory_mib: 24_463,
                driver: "615.71.09".into(),
                cuda: Version::new(13, 4, 0),
                power_profile: Some("performance".into()),
                ac_power: Some(true),
            },
            ai_ctl: "0.2.0".into(),
        }
    }

    /// A request: `cached` and `evaluated` prompt tokens, `predicted` tokens
    /// generated, the first after `ttft_ms` and the rest in `gen_ms`.
    pub fn sample(
        cached: u64,
        evaluated: u64,
        ttft_ms: u64,
        predicted: u64,
        gen_ms: u64,
    ) -> Sample {
        Sample {
            client: ClientTimings {
                ttft: Duration::from_millis(ttft_ms),
                total: Duration::from_millis(ttft_ms + gen_ms),
                gaps: Vec::new(),
            },
            server: Some(ServerTimings {
                cache_n: cached,
                prompt_n: evaluated,
                prompt_ms: ttft_ms as f64,
                predicted_n: predicted,
                predicted_ms: gen_ms as f64,
                drafts: None,
            }),
            usage: Some(Usage {
                prompt_tokens: cached + evaluated,
                completion_tokens: predicted,
            }),
        }
    }

    /// `sample` with `proposed` drafts, `accepted` of them.
    pub fn drafted(mut sample: Sample, proposed: u64, accepted: u64) -> Sample {
        if let Some(server) = sample.server.as_mut() {
            server.drafts = Some(Drafts { proposed, accepted });
        }
        sample
    }

    /// The 27B measured twice in each scenario.
    pub fn run() -> Run {
        let plan = Plan::new(98_304, 2, &Scenario::ALL, 0).unwrap();
        let depth = |tokens, at_limit, ttft| DepthResult {
            depth: Depth { tokens, at_limit },
            fill: sample(0, u64::from(tokens), u64::from(tokens) / 2, 512, 8_000),
            samples: vec![
                drafted(sample(u64::from(tokens) - 1, 1, ttft, 512, 8_000), 400, 300),
                drafted(
                    sample(u64::from(tokens) - 1, 1, ttft + 20, 512, 8_533),
                    400,
                    340,
                ),
            ],
        };
        Run {
            started: Timestamp::parse("2026-10-11T12:00:00Z").unwrap(),
            model: "qwen3.8-27b".into(),
            setup: setup(),
            n_ctx: 98_304,
            depths: vec![depth(8_192, false, 180), depth(97_791, true, 1_480)],
            prefill: Some(PrefillResult {
                samples: vec![
                    sample(45, 40_915, 31_000, 1, 0),
                    sample(45, 40_915, 31_600, 1, 0),
                ],
            }),
            session: Some(SessionResult {
                sessions: vec![
                    vec![
                        sample(37, 8_155, 6_000, 512, 8_000),
                        sample(8_700, 1_040, 900, 512, 8_000),
                    ],
                    vec![
                        sample(43, 8_149, 6_100, 512, 8_000),
                        sample(8_700, 1_040, 940, 512, 8_000),
                    ],
                ],
            }),
            overhead: Some(OverheadResult {
                gateway: vec![sample(700, 1, 46, 16, 250), sample(700, 1, 47, 16, 251)],
                direct: vec![sample(700, 1, 45, 16, 249), sample(700, 1, 45, 16, 250)],
            }),
            plan,
            drafts: Some(DraftCheck {
                requests: Drafts {
                    proposed: 9_000,
                    accepted: 7_200,
                },
                before: SpecCounters::default(),
                after: SpecCounters {
                    drafts: Drafts {
                        proposed: 9_000,
                        accepted: 7_200,
                    },
                    steps: 3_000,
                    accepted_at: vec![2_700, 2_400, 2_100],
                },
            }),
            gpu: vec![
                reading(None, 18_500, 60, 2_100, 40.0, RELIABILITY),
                reading(Some(Scenario::Depth), 22_900, 78, 2_000, 110.0, POWER),
                reading(Some(Scenario::Depth), 23_300, 86, 1_700, 104.0, HOT),
                reading(Some(Scenario::Session), 20_100, 80, 1_950, 108.0, BRAKE),
            ],
        }
    }

    pub const POWER: Slowdown = Slowdown {
        power: true,
        thermal: false,
        hardware: false,
        policy: false,
    };
    pub const HOT: Slowdown = Slowdown {
        power: true,
        thermal: true,
        hardware: false,
        policy: false,
    };
    /// The hardware's power brake, under the power limit.
    pub const BRAKE: Slowdown = Slowdown {
        power: true,
        thermal: false,
        hardware: true,
        policy: false,
    };
    /// What this machine's GPU reports at idle.
    pub const RELIABILITY: Slowdown = Slowdown {
        power: false,
        thermal: false,
        hardware: false,
        policy: true,
    };

    pub fn reading(
        scenario: Option<Scenario>,
        memory_used_mib: u64,
        temperature_c: u32,
        sm_clock_mhz: u32,
        power_w: f64,
        slowdown: Slowdown,
    ) -> GpuReading {
        GpuReading {
            scenario,
            memory_used_mib,
            temperature_c,
            sm_clock_mhz,
            power_w,
            slowdown,
            clock_events: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ms(value: u64) -> Duration {
        Duration::from_millis(value)
    }

    fn timings(cache_n: u64, prompt_n: u64, prompt_ms: f64) -> ServerTimings {
        ServerTimings {
            cache_n,
            prompt_n,
            prompt_ms,
            predicted_n: 100,
            predicted_ms: 2000.0,
            drafts: None,
        }
    }

    fn sample(ttft: u64, total: u64, server: Option<ServerTimings>) -> Sample {
        Sample {
            client: ClientTimings {
                ttft: ms(ttft),
                total: ms(total),
                gaps: Vec::new(),
            },
            server,
            usage: None,
        }
    }

    fn cached(cache_n: u64) -> Sample {
        sample(100, 1000, Some(timings(cache_n, 50, 25.0)))
    }

    #[test]
    fn scenarios_have_the_names_runs_use() {
        let names: Vec<&str> = Scenario::ALL.iter().map(|s| s.name()).collect();
        assert_eq!(names, ["depth", "prefill", "session", "overhead"]);
        assert_eq!(Scenario::Overhead.to_string(), "overhead");
        for scenario in Scenario::ALL {
            assert_eq!(Scenario::from_name(scenario.name()), Some(scenario));
        }
        assert_eq!(Scenario::from_name("load"), None);
    }

    #[test]
    fn an_assistant_message_keeps_its_reasoning_when_there_is_some() {
        assert_eq!(Message::assistant("Hi.", "").reasoning, None);
        let reply = Message::assistant("Hi.", "Greet back.");
        assert_eq!(reply.reasoning.as_deref(), Some("Greet back."));
        assert_eq!(reply.role.name(), "assistant");
        assert_eq!(Message::system("Be brief.").role.name(), "system");
        assert_eq!(Message::user("Hello").role.name(), "user");
    }

    #[test]
    fn depths_reach_the_limit_of_the_context() {
        // The 27B: 98304 tokens of context, less the 512 to generate and one
        // left free.
        let plan = Plan::new(98_304, REPEAT, &Scenario::ALL, 0).unwrap();
        let depths: Vec<(u32, bool)> = plan.depths.iter().map(|d| (d.tokens, d.at_limit)).collect();
        assert_eq!(
            depths,
            [
                (8_192, false),
                (32_768, false),
                (65_536, false),
                (97_791, true)
            ]
        );
    }

    #[test]
    fn a_depth_beyond_the_limit_is_skipped() {
        let depths: Vec<u32> = Plan::new(32_768, 1, &Scenario::ALL, 0)
            .unwrap()
            .depths
            .iter()
            .map(|d| d.tokens)
            .collect();
        assert_eq!(depths, [8_192, 32_255]);
        // A standard depth equal to the limit is measured once, as the limit.
        let at_limit = Plan::new(65_536 + GENERATE + FREE, 1, &Scenario::ALL, 0)
            .unwrap()
            .depths;
        assert_eq!(at_limit.len(), 3);
        assert_eq!(
            at_limit[2],
            Depth {
                tokens: 65_536,
                at_limit: true
            }
        );
    }

    #[test]
    fn the_prefill_prompt_shrinks_to_a_small_context() {
        assert_eq!(
            Plan::new(98_304, 1, &Scenario::ALL, 0).unwrap().prefill,
            PREFILL
        );
        assert_eq!(
            Plan::new(32_768, 1, &Scenario::ALL, 0).unwrap().prefill,
            32_766
        );
    }

    #[test]
    fn the_default_plan_measures_each_scenario_five_times() {
        let plan = Plan::new(131_072, REPEAT, &Scenario::ALL, 0).unwrap();
        assert_eq!(plan.repeat, 5);
        assert_eq!(plan.generate, GENERATE);
        assert_eq!(plan.session, SessionPlan::DEFAULT);
        assert_eq!(plan.overhead, OVERHEAD_REQUESTS);
    }

    #[test]
    fn a_session_fills_its_last_prompt_and_reply() {
        // 8192 + 9 turns of 1024 + 10 replies of 512.
        assert_eq!(SessionPlan::DEFAULT.peak(512, 0), 22_528);
        // Each of the 9 later turns adds its own text and the template's.
        assert_eq!(SessionPlan::DEFAULT.peak(512, 63), 22_528 + 9 * 63);
        let one = SessionPlan {
            turns: 1,
            opening: 100,
            turn: 50,
        };
        assert_eq!(one.peak(10, 0), 110);
        assert_eq!(one.peak(10, 63), 110);
    }

    #[test]
    fn a_session_needs_its_peak_and_a_free_token() {
        // 8192 + 9 turns of 1024 and 63 + 10 replies of 512, and one free.
        let session = [Scenario::Session];
        assert!(Plan::new(23_096, 1, &session, 63).is_ok());
        assert_eq!(
            Plan::new(23_095, 1, &session, 63),
            Err(PlanError::ContextTooSmall {
                scenario: Scenario::Session,
                needed: 23_096,
                ctx_size: 23_095
            })
        );
    }

    #[test]
    fn a_session_left_out_needs_no_room_in_the_context() {
        let others = [Scenario::Depth, Scenario::Prefill, Scenario::Overhead];
        let plan = Plan::new(16_384, 1, &others, 63).unwrap();
        assert_eq!(plan.depths.last().unwrap().tokens, 16_384 - GENERATE - FREE);
        assert_eq!(plan.session, SessionPlan::DEFAULT);
    }

    #[test]
    fn a_plan_that_does_not_fit_is_refused() {
        assert_eq!(
            Plan::new(98_304, 0, &Scenario::ALL, 0),
            Err(PlanError::NoRepetition)
        );
        assert_eq!(
            Plan::new(GENERATE, 1, &Scenario::ALL, 0),
            Err(PlanError::ContextTooSmall {
                scenario: Scenario::Depth,
                needed: u64::from(GENERATE + FREE) + 1,
                ctx_size: GENERATE
            })
        );
        let error = Plan::new(16_384, 1, &Scenario::ALL, 0).unwrap_err();
        assert_eq!(
            error.to_string(),
            "the session scenario needs a context of 22529 tokens; the model's holds 16384"
        );
    }

    #[test]
    fn speeds_come_from_the_server_timings() {
        let timings = ServerTimings {
            cache_n: 0,
            prompt_n: 40_960,
            prompt_ms: 20_480.0,
            predicted_n: 512,
            predicted_ms: 10_240.0,
            drafts: None,
        };
        assert_eq!(timings.prefill_speed(), Some(2000.0));
        assert_eq!(timings.generation_speed(), Some(50.0));
        let sample = sample(20_500, 30_800, Some(timings));
        assert_eq!(sample.prefill_speed(), Some(2000.0));
        assert_eq!(sample.generation_speed(), Some(50.0));
        assert_eq!((sample.ttft_ms(), sample.total_ms()), (20_500.0, 30_800.0));
    }

    #[test]
    fn a_speed_over_no_time_or_no_token_is_unknown() {
        let mut timings = timings(0, 0, 0.0);
        assert_eq!(timings.prefill_speed(), None);
        timings.predicted_n = 0;
        assert_eq!(timings.generation_speed(), None);
    }

    #[test]
    fn without_timings_the_client_measures_generation() {
        let mut sample = sample(500, 2500, None);
        assert_eq!(sample.generation_speed(), None);
        sample.usage = Some(Usage {
            prompt_tokens: 1000,
            completion_tokens: 101,
        });
        // 100 tokens after the first, in the 2 s after it.
        assert_eq!(sample.generation_speed(), Some(50.0));
        assert_eq!(sample.prefill_speed(), None);
        assert_eq!(sample.cached(), None);
    }

    #[test]
    fn drafts_add_up_across_requests() {
        let with = |proposed, accepted| {
            let mut timings = timings(0, 1, 1.0);
            timings.drafts = Some(Drafts { proposed, accepted });
            sample(1, 2, Some(timings))
        };
        let samples = [with(200, 120), cached(0), with(100, 63), sample(1, 2, None)];
        let total = drafts(&samples).unwrap();
        assert_eq!(
            total,
            Drafts {
                proposed: 300,
                accepted: 183
            }
        );
        assert_eq!(total.acceptance(), Some(0.61));
        assert_eq!(drafts(&[cached(0)]), None);
        assert_eq!(Drafts::default().acceptance(), None);
    }

    fn counters(proposed: u64, accepted: u64, steps: u64, accepted_at: &[u64]) -> SpecCounters {
        SpecCounters {
            drafts: Drafts { proposed, accepted },
            steps,
            accepted_at: accepted_at.to_vec(),
        }
    }

    #[test]
    fn the_server_counts_a_run_as_the_difference_of_its_counters() {
        let before = counters(300, 200, 100, &[90, 70, 40]);
        let after = counters(1_500, 1_100, 500, &[450, 380, 270]);
        let run = after.since(&before);
        assert_eq!(run, counters(1_200, 900, 400, &[360, 310, 230]));
        assert_eq!(run.acceptance_at(), [0.9, 0.775, 0.575]);
        assert_eq!(run.tokens_per_step(), Some(3.25));
        // A server that drafted nothing before the run lists no position.
        assert_eq!(after.since(&SpecCounters::default()), after);
        assert!(SpecCounters::default().acceptance_at().is_empty());
        assert_eq!(SpecCounters::default().tokens_per_step(), None);
    }

    #[test]
    fn counters_from_a_million_on_are_rounded_to_six_digits() {
        assert_eq!(rounding(999_999), 0);
        assert_eq!(rounding(1_234_570), 5);
        assert_eq!(rounding(12_345_700), 50);
    }

    #[test]
    fn the_requests_account_for_every_draft_the_server_counted() {
        let check = |requests: Drafts, before: SpecCounters, after: SpecCounters| DraftCheck {
            requests,
            before,
            after,
        };
        let exact = check(
            Drafts {
                proposed: 1_200,
                accepted: 900,
            },
            counters(300, 200, 100, &[]),
            counters(1_500, 1_100, 500, &[]),
        );
        assert!(exact.agrees());
        // Another client's request.
        let mut shared = exact.clone();
        shared.after.drafts.proposed += 3;
        assert!(!shared.agrees());
        // A request whose accepted drafts went unread.
        let mut unread = exact.clone();
        unread.after.drafts.accepted += 3;
        assert!(!unread.agrees());
        // Read as 1.23457e+06 and 1.27457e+06: each within 5 of its value.
        let rounded = check(
            Drafts {
                proposed: 40_008,
                accepted: 30_000,
            },
            counters(1_234_570, 900_000, 0, &[]),
            counters(1_274_570, 930_000, 0, &[]),
        );
        assert!(rounded.agrees());
    }

    fn reading(scenario: Option<Scenario>, memory: u64, celsius: u32, slowed: bool) -> GpuReading {
        GpuReading {
            scenario,
            memory_used_mib: memory,
            temperature_c: celsius,
            sm_clock_mhz: if slowed { 1_600 } else { 2_000 },
            power_w: 100.0,
            slowdown: Slowdown {
                power: true,
                thermal: slowed,
                hardware: false,
                policy: false,
            },
            clock_events: None,
        }
    }

    #[test]
    fn the_gpu_is_summarised_by_scenario_and_over_the_run() {
        let gpu = vec![
            reading(None, 18_000, 55, false),
            reading(Some(Scenario::Depth), 23_000, 80, false),
            reading(Some(Scenario::Depth), 23_400, 87, true),
            reading(Some(Scenario::Session), 20_000, 82, false),
        ];
        let depth =
            GpuSummary::of(gpu.iter().filter(|r| r.scenario == Some(Scenario::Depth))).unwrap();
        assert_eq!(
            (depth.peak_memory_mib, depth.max_temperature_c),
            (23_400, 87)
        );
        assert_eq!(
            (depth.sm_clock_mhz.min, depth.sm_clock_mhz.median),
            (1_600.0, 1_800.0)
        );
        assert_eq!(
            (depth.power_capped, depth.thermal, depth.hardware),
            (1.0, 0.5, 0.0)
        );
        assert_eq!(GpuSummary::of([]), None);
        let mut run = Run {
            started: Timestamp::EPOCH,
            model: "m".into(),
            setup: testing::setup(),
            n_ctx: 98_304,
            plan: Plan::new(98_304, 1, &Scenario::ALL, 0).unwrap(),
            depths: Vec::new(),
            prefill: None,
            session: None,
            overhead: None,
            drafts: None,
            gpu,
        };
        assert_eq!(run.id(), "19700101T000000Z-m");
        assert_eq!(run.gpu(Some(Scenario::Depth)), Some(depth));
        // One reading slowed by heat among the run's four.
        assert_eq!(run.gpu(None).unwrap().thermal, 0.25);
        assert_eq!(run.gpu(Some(Scenario::Prefill)), None);
        run.gpu.clear();
        assert_eq!(run.gpu(None), None);
    }

    #[test]
    fn a_run_holds_the_scenarios_it_has_results_of() {
        let mut run = testing::run();
        assert_eq!(run.scenarios(), Scenario::ALL);
        run.depths.clear();
        run.session = None;
        assert!(!run.holds(Scenario::Depth) && !run.holds(Scenario::Session));
        assert_eq!(run.scenarios(), [Scenario::Prefill, Scenario::Overhead]);
    }

    /// The test run, measured at depth only.
    fn depth_only() -> Run {
        let mut run = testing::run();
        (run.prefill, run.session, run.overhead) = (None, None, None);
        run
    }

    #[test]
    fn runs_compare_their_gpu_over_the_scenarios_they_all_ran() {
        let runs = [testing::run(), depth_only()];
        let common = common_scenarios(&runs);
        assert_eq!(common, [Scenario::Depth]);
        assert_eq!(
            runs[0].gpu_over(&common),
            runs[0].gpu(Some(Scenario::Depth))
        );
        assert_eq!(
            common_scenarios(&[testing::run(), testing::run()]),
            Scenario::ALL
        );
        // The depth's two readings and the session's one, not the idle
        // reading before the first scenario, which a policy limited.
        let both = runs[0]
            .gpu_over(&[Scenario::Depth, Scenario::Session])
            .unwrap();
        assert_eq!(
            (both.peak_memory_mib, both.sm_clock_mhz.min),
            (23_300, 1_700.0)
        );
        assert_eq!((both.hardware, both.policy), (1.0 / 3.0, 0.0));
        assert_eq!(runs[0].gpu_over(&[Scenario::Prefill]), None);
        assert_eq!(runs[0].gpu_over(&[]), None);
    }

    #[test]
    fn the_servers_counters_compare_only_between_runs_of_the_same_scenarios() {
        let mut without_counters = testing::run();
        without_counters.drafts = None;
        let server = testing::run().drafts.unwrap().server();
        assert_eq!(
            comparable_counters(&[testing::run(), without_counters]),
            Some(vec![Some(server), None])
        );
        assert_eq!(comparable_counters(&[testing::run(), depth_only()]), None);
    }

    #[test]
    fn a_hardware_slowdown_is_summarised_with_its_share() {
        let run = testing::run();
        // The session's one reading, under the power brake.
        assert_eq!(run.gpu(Some(Scenario::Session)).unwrap().hardware, 1.0);
        assert_eq!(run.gpu(None).unwrap().hardware, 0.25);
    }

    #[test]
    fn a_policy_limit_is_summarised_apart_from_the_slowdowns() {
        let mut limited = reading(Some(Scenario::Depth), 23_000, 80, false);
        limited.slowdown.policy = true;
        let free = reading(Some(Scenario::Depth), 23_000, 80, false);
        let summary = GpuSummary::of([&limited, &free]).unwrap();
        assert_eq!(summary.policy, 0.5);
        assert_eq!(
            (summary.power_capped, summary.thermal, summary.hardware),
            (1.0, 0.0, 0.0)
        );
    }

    #[test]
    fn arrival_gaps_pool_every_request_and_skip_bursts() {
        let mut first = sample(1, 2, None);
        first.client.gaps = vec![ms(40), Duration::ZERO, Duration::from_micros(300), ms(44)];
        let mut second = sample(1, 2, None);
        second.client.gaps = vec![ms(36), ms(1)];
        let summary = arrival_gaps(&[first, second]).unwrap();
        // 40, 44, 36 and 1: the zero and the burst left out.
        assert_eq!(summary.mean, 30.25);
        assert_eq!((summary.min, summary.median), (1.0, 38.0));
        assert_eq!(arrival_gaps(&[cached(0)]), None);
    }

    #[test]
    fn a_depth_summarises_its_repetitions_not_its_fill() {
        let mut fast = timings(8_191, 1, 1.0);
        fast.predicted_ms = 1000.0;
        let result = DepthResult {
            depth: Depth {
                tokens: 8_192,
                at_limit: false,
            },
            fill: sample(4000, 6000, Some(timings(0, 8_192, 4000.0))),
            samples: vec![
                sample(40, 2040, Some(timings(8_191, 1, 1.0))),
                sample(60, 1060, Some(fast)),
            ],
        };
        let generation = result.generation().unwrap();
        assert_eq!((generation.min, generation.mean), (50.0, 75.0));
        assert_eq!(result.ttft().unwrap().mean, 50.0);
        assert_eq!(result.fill.prefill_speed(), Some(2048.0));
    }

    #[test]
    fn a_prefill_summarises_its_speed_and_first_token() {
        let result = PrefillResult {
            samples: vec![
                sample(20_000, 20_010, Some(timings(0, 40_960, 20_480.0))),
                sample(21_000, 21_010, Some(timings(0, 40_960, 20_480.0))),
            ],
        };
        assert_eq!(result.prefill().unwrap().median, 2000.0);
        assert_eq!(result.ttft().unwrap().median, 20_500.0);
    }

    #[test]
    fn a_session_is_summarised_turn_by_turn() {
        let result = SessionResult {
            sessions: vec![
                vec![cached(0), cached(9_000), cached(10_500)],
                vec![cached(0), cached(9_100)],
            ],
        };
        let turns = result.turns();
        assert_eq!(turns.len(), 3);
        assert_eq!(turns[0].turn, 1);
        assert_eq!(turns[1].cached.unwrap().median, 9_050.0);
        assert_eq!(turns[1].evaluated.unwrap().median, 50.0);
        // One session reached the third turn.
        assert_eq!(turns[2].cached.unwrap().stddev, None);
        assert_eq!(turns[2].ttft.unwrap().mean, 100.0);
        assert!(result.cache_grows());
    }

    #[test]
    fn a_session_whose_cache_stalls_does_not_grow() {
        let stalled = SessionResult {
            sessions: vec![
                vec![cached(0), cached(9_000), cached(10_500)],
                vec![cached(0), cached(9_000), cached(9_000)],
            ],
        };
        assert!(!stalled.cache_grows());
        let untimed = SessionResult {
            sessions: vec![vec![cached(0), sample(1, 2, None)]],
        };
        assert!(!untimed.cache_grows());
        let single = SessionResult {
            sessions: vec![vec![cached(0)]],
        };
        assert!(!single.cache_grows());
        assert!(SessionResult { sessions: vec![] }.turns().is_empty());
    }

    #[test]
    fn the_overhead_is_the_difference_of_the_medians() {
        let result = OverheadResult {
            gateway: vec![sample(42, 300, None), sample(44, 304, None)],
            direct: vec![sample(41, 299, None), sample(41, 301, None)],
        };
        assert_eq!(result.medians(Sample::ttft_ms), Some((43.0, 41.0)));
        assert_eq!(result.medians(Sample::total_ms), Some((302.0, 300.0)));
        let added = result.added().unwrap();
        assert_eq!((added.ttft_ms, added.total_ms), (2.0, 2.0));
        let one_way = OverheadResult {
            gateway: vec![sample(42, 300, None)],
            direct: Vec::new(),
        };
        assert_eq!(one_way.medians(Sample::ttft_ms), None);
        assert_eq!(one_way.added(), None);
    }
}
