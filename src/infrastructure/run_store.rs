//! The bench's saved runs, one JSON file each.
//!
//! ```text
//! bench/
//!   20261011T154305Z-qwen3.8-27b.json
//!   .20261011T154305Z-qwen3.8-27b.json.4242.tmp   a run being saved, out of sight
//! ```
//!
//! A run is written beside its name, then linked under it
//! ([`files::write_new`]), so a saved run is always complete and never
//! replaces another. Every sample is kept, gaps
//! between tokens included, so that a later ai-ctl can summarise them in
//! another way.

use std::collections::BTreeMap;
use std::fs;
use std::io;
use std::path::PathBuf;
use std::time::Duration;

use anyhow::{Context, Result, anyhow, bail, ensure};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::application::ports::RunStore;
use crate::domain::bench::{
    ClientTimings, Depth, DepthResult, DraftCheck, Drafts, GpuReading, Machine, OverheadResult,
    Plan, PrefillResult, Run, Sample, Scenario, ServerTimings, SessionPlan, SessionResult, Setup,
    Slowdown, Source, SpecCounters, Usage,
};
use crate::domain::build::{Build, BuildId, CommitId};
use crate::domain::inventory::{FlagValue, Model};
use crate::domain::machine::{HostCompiler, Version};
use crate::domain::timestamp::Timestamp;

use super::files;

/// The format this ai-ctl writes. Format 1 lacks each GPU reading's
/// `policy` and `clock_events`; formats 1 and 2 hold a native build as its
/// `build.json` held it, `format` included.
const FORMAT: u32 = 3;
/// The formats this ai-ctl reads.
// LEGACY: the runs saved before format 3 must stay readable for `bench list`
// and `bench compare`. Formats 1 and 2 leave this list, and the reads marked
// LEGACY below go, once no such run is kept or they are migrated to format 3.
const READS: [u32; 3] = [1, 2, 3];
/// The format of `build.json` that runs of formats 1 and 2 hold.
// LEGACY: read in the runs saved before format 3 only (see `READS`).
const BUILD_JSON_FORMAT: u32 = 1;
const EXTENSION: &str = ".json";

/// The saved runs, in one directory.
pub struct FsRunStore {
    dir: PathBuf,
}

impl FsRunStore {
    pub fn new(dir: PathBuf) -> Self {
        Self { dir }
    }

    fn path(&self, id: &str) -> PathBuf {
        self.dir.join(format!("{id}{EXTENSION}"))
    }
}

impl RunStore for FsRunStore {
    fn save(&self, run: &Run) -> Result<PathBuf> {
        let text = to_json(run)?;
        let path = self.path(&run.id());
        // A run saved before is never replaced: only the run being measured
        // replaces its own file, through `replace`.
        ensure!(
            files::write_new(&path, &text, 0o666)?,
            "a run is already saved as {}",
            path.display()
        );
        Ok(path)
    }

    fn replace(&self, run: &Run) -> Result<PathBuf> {
        let path = self.path(&run.id());
        files::replace_file(&path, &to_json(run)?)?;
        Ok(path)
    }

    fn ids(&self) -> Result<Vec<String>> {
        let entries = match fs::read_dir(&self.dir) {
            Err(err) if err.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
            entries => entries.with_context(|| format!("cannot read {}", self.dir.display()))?,
        };
        let mut ids = Vec::new();
        for entry in entries {
            let entry = entry.with_context(|| format!("cannot read {}", self.dir.display()))?;
            let name = entry.file_name();
            let Some(id) = name.to_str().and_then(|name| name.strip_suffix(EXTENSION)) else {
                continue;
            };
            if !id.starts_with('.') {
                ids.push(id.to_owned());
            }
        }
        ids.sort();
        Ok(ids)
    }

    fn load(&self, id: &str) -> Result<Run> {
        ensure!(
            !id.is_empty() && !id.starts_with('.') && !id.contains('/'),
            "{id:?} is not the name of a run"
        );
        let path = self.path(id);
        let text = match fs::read_to_string(&path) {
            Err(err) if err.kind() == io::ErrorKind::NotFound => {
                bail!("no run is saved as {id}; `ai-ctl bench list` lists them")
            }
            text => text.with_context(|| format!("cannot read {}", path.display()))?,
        };
        from_json(&text).with_context(|| format!("cannot read {}", path.display()))
    }
}

/// A saved run, as written to disk.
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct RunJson {
    format: u32,
    started: String,
    model: String,
    setup: SetupJson,
    n_ctx: u32,
    plan: PlanJson,
    depths: Vec<DepthResultJson>,
    prefill: Option<Vec<SampleJson>>,
    session: Option<Vec<Vec<SampleJson>>>,
    overhead: Option<OverheadJson>,
    drafts: Option<DraftCheckJson>,
    gpu: Vec<GpuReadingJson>,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct SetupJson {
    model: ModelJson,
    source: SourceJson,
    command: Vec<String>,
    machine: MachineJson,
    ai_ctl: String,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ModelJson {
    id: String,
    backend: String,
    repo: Option<String>,
    dir: PathBuf,
    files: BTreeMap<String, PathBuf>,
    flags: BTreeMap<String, Value>,
    ctx_size: Option<u32>,
    vram_mib: u32,
    ttl: Option<u32>,
    preload: bool,
}

/// A native build, or an image.
#[derive(Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "lowercase", deny_unknown_fields)]
enum SourceJson {
    Build {
        build: BuildJson,
    },
    Image {
        reference: String,
        digest: Option<String>,
    },
}

/// The run's own copy of a native build's fields, so that a new format of
/// `build.json` leaves the saved runs readable.
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct BuildJson {
    /// `build.json`'s format, in runs of formats 1 and 2 only.
    // LEGACY: the runs saved before format 3 must stay readable (see `READS`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    format: Option<u32>,
    project: String,
    id: String,
    commit: String,
    toolkit: String,
    host_compiler: HostCompilerJson,
    cmake_flags: BTreeMap<String, String>,
    /// Left out when the build has none, as before builds had patches.
    // LEGACY: absent from the runs saved before builds had patches, which
    // must stay readable (see `READS`); left out when empty, so that format 3
    // keeps the shape it had before patches. A format that always writes the
    // key would let the omission go once no such run is kept.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    patches: Vec<String>,
    built_at: String,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct HostCompilerJson {
    command: String,
    path: PathBuf,
    version: String,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct MachineJson {
    gpu: String,
    gpu_memory_mib: u64,
    driver: String,
    cuda: String,
    power_profile: Option<String>,
    ac_power: Option<bool>,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct PlanJson {
    repeat: u32,
    generate: u32,
    depths: Vec<DepthJson>,
    prefill: u32,
    session: SessionPlanJson,
    overhead: u32,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct DepthJson {
    tokens: u32,
    at_limit: bool,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct SessionPlanJson {
    turns: u32,
    opening: u32,
    turn: u32,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct DepthResultJson {
    depth: DepthJson,
    fill: SampleJson,
    samples: Vec<SampleJson>,
}

/// A request, its durations in microseconds.
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct SampleJson {
    ttft_us: u64,
    total_us: u64,
    gaps_us: Vec<u64>,
    server: Option<ServerJson>,
    usage: Option<UsageJson>,
}

/// llama-server's `timings`, under its own names.
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ServerJson {
    cache_n: u64,
    prompt_n: u64,
    prompt_ms: f64,
    predicted_n: u64,
    predicted_ms: f64,
    draft_n: Option<u64>,
    draft_n_accepted: Option<u64>,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct UsageJson {
    prompt_tokens: u64,
    completion_tokens: u64,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct OverheadJson {
    gateway: Vec<SampleJson>,
    direct: Vec<SampleJson>,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct DraftCheckJson {
    requests: DraftsJson,
    before: CountersJson,
    after: CountersJson,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct DraftsJson {
    proposed: u64,
    accepted: u64,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct CountersJson {
    proposed: u64,
    accepted: u64,
    steps: u64,
    accepted_at: Vec<u64>,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct GpuReadingJson {
    scenario: Option<String>,
    memory_used_mib: u64,
    temperature_c: u32,
    sm_clock_mhz: u32,
    power_w: f64,
    power_capped: bool,
    thermal: bool,
    hardware: bool,
    // LEGACY: absent from the runs of format 1, which must stay readable (see
    // `READS`).
    #[serde(default)]
    policy: bool,
    // LEGACY: absent from the runs of format 1, as `policy` is.
    #[serde(default)]
    clock_events: Option<u64>,
}

/// Serializes `run` as the content of its file.
pub fn to_json(run: &Run) -> Result<String> {
    let json = RunJson {
        format: FORMAT,
        started: run.started.to_string(),
        model: run.model.clone(),
        setup: setup_json(&run.setup)?,
        n_ctx: run.n_ctx,
        plan: plan_json(&run.plan),
        depths: run
            .depths
            .iter()
            .map(|result| DepthResultJson {
                depth: depth_json(result.depth),
                fill: sample_json(&result.fill),
                samples: samples_json(&result.samples),
            })
            .collect(),
        prefill: run
            .prefill
            .as_ref()
            .map(|result| samples_json(&result.samples)),
        session: run.session.as_ref().map(|result| {
            result
                .sessions
                .iter()
                .map(|session| samples_json(session))
                .collect()
        }),
        overhead: run.overhead.as_ref().map(|result| OverheadJson {
            gateway: samples_json(&result.gateway),
            direct: samples_json(&result.direct),
        }),
        drafts: run.drafts.as_ref().map(|check| DraftCheckJson {
            requests: DraftsJson {
                proposed: check.requests.proposed,
                accepted: check.requests.accepted,
            },
            before: counters_json(&check.before),
            after: counters_json(&check.after),
        }),
        gpu: run.gpu.iter().map(gpu_json).collect(),
    };
    let mut text = serde_json::to_string_pretty(&json).context("cannot serialize the run")?;
    text.push('\n');
    Ok(text)
}

/// Reads the content of a run's file.
pub fn from_json(text: &str) -> Result<Run> {
    let format = files::json_format(text).context("malformed run")?;
    ensure!(
        READS.contains(&format),
        "run format {format} is not supported (this ai-ctl reads formats 1, 2 and 3)"
    );
    let json: RunJson = serde_json::from_str(text).context("malformed run")?;
    Ok(Run {
        started: Timestamp::parse(&json.started)
            .with_context(|| format!("started `{}` is not a UTC timestamp", json.started))?,
        model: json.model,
        setup: setup(json.setup)?,
        n_ctx: json.n_ctx,
        plan: plan(json.plan),
        depths: json
            .depths
            .into_iter()
            .map(|result| DepthResult {
                depth: depth(result.depth),
                fill: sample(result.fill),
                samples: samples(result.samples),
            })
            .collect(),
        prefill: json.prefill.map(|samples| PrefillResult {
            samples: self::samples(samples),
        }),
        session: json.session.map(|sessions| SessionResult {
            sessions: sessions.into_iter().map(samples).collect(),
        }),
        overhead: json.overhead.map(|result| OverheadResult {
            gateway: samples(result.gateway),
            direct: samples(result.direct),
        }),
        drafts: json.drafts.map(|check| DraftCheck {
            requests: Drafts {
                proposed: check.requests.proposed,
                accepted: check.requests.accepted,
            },
            before: counters(check.before),
            after: counters(check.after),
        }),
        gpu: json.gpu.into_iter().map(gpu).collect::<Result<_>>()?,
    })
}

fn setup_json(setup: &Setup) -> Result<SetupJson> {
    let model = &setup.model;
    Ok(SetupJson {
        model: ModelJson {
            id: model.id.clone(),
            backend: model.backend.clone(),
            repo: model.repo.clone(),
            dir: model.dir.clone(),
            files: model.files.clone(),
            flags: model
                .flags
                .iter()
                .map(|(name, value)| Ok((name.clone(), flag_json(name, value)?)))
                .collect::<Result<_>>()?,
            ctx_size: model.ctx_size,
            vram_mib: model.vram_mib,
            ttl: model.ttl,
            preload: model.preload,
        },
        source: match &setup.source {
            Source::Build(build) => SourceJson::Build {
                build: build_json(build),
            },
            Source::Image { reference, digest } => SourceJson::Image {
                reference: reference.clone(),
                digest: digest.clone(),
            },
        },
        command: setup.command.clone(),
        machine: MachineJson {
            gpu: setup.machine.gpu.clone(),
            gpu_memory_mib: setup.machine.gpu_memory_mib,
            driver: setup.machine.driver.clone(),
            cuda: setup.machine.cuda.to_string(),
            power_profile: setup.machine.power_profile.clone(),
            ac_power: setup.machine.ac_power,
        },
        ai_ctl: setup.ai_ctl.clone(),
    })
}

fn build_json(build: &Build) -> BuildJson {
    BuildJson {
        format: None,
        project: build.project.clone(),
        id: build.id.to_string(),
        commit: build.commit.to_string(),
        toolkit: build.toolkit.to_string(),
        host_compiler: HostCompilerJson {
            command: build.host_compiler.command.clone(),
            path: build.host_compiler.path.clone(),
            version: build.host_compiler.version.to_string(),
        },
        cmake_flags: build.cmake_flags.clone(),
        patches: build.patches.clone(),
        built_at: build.built_at.to_string(),
    }
}

fn build(json: BuildJson) -> Result<Build> {
    // LEGACY: a run saved before format 3, which must stay readable (see
    // `READS`).
    if let Some(format) = json.format {
        ensure!(
            format == BUILD_JSON_FORMAT,
            "the build's format {format} is not supported (runs held build.json's format \
             {BUILD_JSON_FORMAT})"
        );
    }
    let version = |what: &str, text: &str| {
        Version::parse(text).with_context(|| format!("{what} `{text}` is not a version"))
    };
    Ok(Build {
        project: json.project,
        id: BuildId::parse(&json.id)?,
        commit: CommitId::parse(&json.commit)?,
        toolkit: version("toolkit", &json.toolkit)?,
        host_compiler: HostCompiler {
            command: json.host_compiler.command,
            path: json.host_compiler.path,
            version: version("host compiler version", &json.host_compiler.version)?,
        },
        cmake_flags: json.cmake_flags,
        patches: json.patches,
        built_at: Timestamp::parse(&json.built_at)
            .with_context(|| format!("built_at `{}` is not a UTC timestamp", json.built_at))?,
    })
}

fn setup(json: SetupJson) -> Result<Setup> {
    let model = json.model;
    Ok(Setup {
        model: Model {
            flags: model
                .flags
                .into_iter()
                .map(|(name, value)| Ok((name.clone(), flag(&name, value)?)))
                .collect::<Result<_>>()?,
            id: model.id,
            backend: model.backend,
            repo: model.repo,
            dir: model.dir,
            files: model.files,
            ctx_size: model.ctx_size,
            vram_mib: model.vram_mib,
            ttl: model.ttl,
            preload: model.preload,
            // The bench runs llama-server models, which have none.
            audiocpp: Default::default(),
        },
        source: match json.source {
            SourceJson::Build { build } => Source::Build(self::build(build)?),
            SourceJson::Image { reference, digest } => Source::Image { reference, digest },
        },
        command: json.command,
        machine: Machine {
            cuda: Version::parse(&json.machine.cuda)
                .with_context(|| format!("cuda `{}` is not a version", json.machine.cuda))?,
            gpu: json.machine.gpu,
            gpu_memory_mib: json.machine.gpu_memory_mib,
            driver: json.machine.driver,
            power_profile: json.machine.power_profile,
            ac_power: json.machine.ac_power,
        },
        ai_ctl: json.ai_ctl,
    })
}

fn flag_json(name: &str, value: &FlagValue) -> Result<Value> {
    Ok(match value {
        FlagValue::Bool(value) => Value::Bool(*value),
        FlagValue::Int(value) => Value::from(*value),
        FlagValue::Float(value) => serde_json::Number::from_f64(*value)
            .map(Value::Number)
            .ok_or_else(|| anyhow!("flag `{name}` is not a finite number"))?,
        FlagValue::Text(value) => Value::String(value.clone()),
    })
}

fn flag(name: &str, value: Value) -> Result<FlagValue> {
    Ok(match value {
        Value::Bool(value) => FlagValue::Bool(value),
        Value::Number(number) => match number.as_i64() {
            Some(value) => FlagValue::Int(value),
            None => FlagValue::Float(
                number
                    .as_f64()
                    .with_context(|| format!("flag `{name}` is out of range"))?,
            ),
        },
        Value::String(value) => FlagValue::Text(value),
        other => bail!("flag `{name}` is not a scalar: {other}"),
    })
}

fn plan_json(plan: &Plan) -> PlanJson {
    PlanJson {
        repeat: plan.repeat,
        generate: plan.generate,
        depths: plan.depths.iter().copied().map(depth_json).collect(),
        prefill: plan.prefill,
        session: SessionPlanJson {
            turns: plan.session.turns,
            opening: plan.session.opening,
            turn: plan.session.turn,
        },
        overhead: plan.overhead,
    }
}

fn plan(json: PlanJson) -> Plan {
    Plan {
        repeat: json.repeat,
        generate: json.generate,
        depths: json.depths.into_iter().map(depth).collect(),
        prefill: json.prefill,
        session: SessionPlan {
            turns: json.session.turns,
            opening: json.session.opening,
            turn: json.session.turn,
        },
        overhead: json.overhead,
    }
}

fn depth_json(depth: Depth) -> DepthJson {
    DepthJson {
        tokens: depth.tokens,
        at_limit: depth.at_limit,
    }
}

fn depth(json: DepthJson) -> Depth {
    Depth {
        tokens: json.tokens,
        at_limit: json.at_limit,
    }
}

fn micros(duration: Duration) -> u64 {
    u64::try_from(duration.as_micros()).unwrap_or(u64::MAX)
}

fn samples_json(samples: &[Sample]) -> Vec<SampleJson> {
    samples.iter().map(sample_json).collect()
}

fn sample_json(sample: &Sample) -> SampleJson {
    SampleJson {
        ttft_us: micros(sample.client.ttft),
        total_us: micros(sample.client.total),
        gaps_us: sample.client.gaps.iter().copied().map(micros).collect(),
        server: sample.server.map(|server| ServerJson {
            cache_n: server.cache_n,
            prompt_n: server.prompt_n,
            prompt_ms: server.prompt_ms,
            predicted_n: server.predicted_n,
            predicted_ms: server.predicted_ms,
            draft_n: server.drafts.map(|drafts| drafts.proposed),
            draft_n_accepted: server.drafts.map(|drafts| drafts.accepted),
        }),
        usage: sample.usage.map(|usage| UsageJson {
            prompt_tokens: usage.prompt_tokens,
            completion_tokens: usage.completion_tokens,
        }),
    }
}

fn samples(json: Vec<SampleJson>) -> Vec<Sample> {
    json.into_iter().map(sample).collect()
}

fn sample(json: SampleJson) -> Sample {
    Sample {
        client: ClientTimings {
            ttft: Duration::from_micros(json.ttft_us),
            total: Duration::from_micros(json.total_us),
            gaps: json
                .gaps_us
                .into_iter()
                .map(Duration::from_micros)
                .collect(),
        },
        server: json.server.map(|server| ServerTimings {
            cache_n: server.cache_n,
            prompt_n: server.prompt_n,
            prompt_ms: server.prompt_ms,
            predicted_n: server.predicted_n,
            predicted_ms: server.predicted_ms,
            drafts: server.draft_n.map(|proposed| Drafts {
                proposed,
                accepted: server.draft_n_accepted.unwrap_or(0),
            }),
        }),
        usage: json.usage.map(|usage| Usage {
            prompt_tokens: usage.prompt_tokens,
            completion_tokens: usage.completion_tokens,
        }),
    }
}

fn counters_json(counters: &SpecCounters) -> CountersJson {
    CountersJson {
        proposed: counters.drafts.proposed,
        accepted: counters.drafts.accepted,
        steps: counters.steps,
        accepted_at: counters.accepted_at.clone(),
    }
}

fn counters(json: CountersJson) -> SpecCounters {
    SpecCounters {
        drafts: Drafts {
            proposed: json.proposed,
            accepted: json.accepted,
        },
        steps: json.steps,
        accepted_at: json.accepted_at,
    }
}

fn gpu_json(reading: &GpuReading) -> GpuReadingJson {
    GpuReadingJson {
        scenario: reading.scenario.map(|scenario| scenario.name().to_owned()),
        memory_used_mib: reading.memory_used_mib,
        temperature_c: reading.temperature_c,
        sm_clock_mhz: reading.sm_clock_mhz,
        power_w: reading.power_w,
        power_capped: reading.slowdown.power,
        thermal: reading.slowdown.thermal,
        hardware: reading.slowdown.hardware,
        policy: reading.slowdown.policy,
        clock_events: reading.clock_events,
    }
}

fn gpu(json: GpuReadingJson) -> Result<GpuReading> {
    let scenario = match json.scenario {
        Some(name) => {
            Some(Scenario::from_name(&name).with_context(|| format!("no scenario `{name}`"))?)
        }
        None => None,
    };
    Ok(GpuReading {
        scenario,
        memory_used_mib: json.memory_used_mib,
        temperature_c: json.temperature_c,
        sm_clock_mhz: json.sm_clock_mhz,
        power_w: json.power_w,
        slowdown: Slowdown {
            power: json.power_capped,
            thermal: json.thermal,
            hardware: json.hardware,
            policy: json.policy,
        },
        clock_events: json.clock_events,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::bench::testing;
    use crate::domain::build::{BuildId, CommitId};
    use crate::domain::machine::HostCompiler;

    fn build() -> crate::domain::build::Build {
        crate::domain::build::Build {
            project: "llama.cpp".into(),
            id: BuildId::parse("b11538").unwrap(),
            commit: CommitId::parse("79e2e74eb11022c1ba2e438df7f0ca2d4c10f8b6").unwrap(),
            toolkit: Version::new(13, 4, 92),
            host_compiler: HostCompiler {
                command: "g++".into(),
                path: "/usr/bin/g++".into(),
                version: Version::new(16, 2, 1),
            },
            cmake_flags: BTreeMap::from([("GGML_CUDA".into(), "ON".into())]),
            patches: vec![],
            built_at: Timestamp::parse("2026-10-09T20:48:47Z").unwrap(),
        }
    }

    /// The test run, on a native build, with gaps between tokens and flags
    /// of every kind.
    fn run() -> Run {
        let mut run = testing::run();
        run.setup.source = Source::Build(build());
        run.setup.model.flags = BTreeMap::from([
            ("jinja".into(), FlagValue::Bool(true)),
            ("spec-draft-n-max".into(), FlagValue::Int(2)),
            ("temp".into(), FlagValue::Float(0.6)),
            ("top-p".into(), FlagValue::Float(1.0)),
            ("cache-type-k".into(), FlagValue::Text("q8_0".into())),
        ]);
        run.depths[0].samples[0].client.gaps = vec![
            Duration::from_micros(41_250),
            Duration::ZERO,
            Duration::from_micros(310),
        ];
        run
    }

    #[test]
    fn a_run_reads_back_as_it_was_written() {
        let run = run();
        assert_eq!(from_json(&to_json(&run).unwrap()).unwrap(), run);
        let on_image = testing::run();
        assert_eq!(from_json(&to_json(&on_image).unwrap()).unwrap(), on_image);
        let mut bare = testing::run();
        (bare.prefill, bare.session, bare.overhead, bare.drafts) = (None, None, None, None);
        bare.depths.clear();
        bare.gpu.clear();
        assert_eq!(from_json(&to_json(&bare).unwrap()).unwrap(), bare);
        let mut patched = run.clone();
        patched.setup.source = Source::Build(crate::domain::build::Build {
            patches: vec!["api-key.patch".into()],
            ..build()
        });
        assert_eq!(from_json(&to_json(&patched).unwrap()).unwrap(), patched);
    }

    #[test]
    fn a_run_is_readable_json() {
        let json: Value = serde_json::from_str(&to_json(&run()).unwrap()).unwrap();
        assert_eq!(json["format"], 3);
        assert_eq!(json["started"], "2026-10-11T12:00:00Z");
        assert_eq!(json["setup"]["source"]["kind"], "build");
        // The run's own copy of the build, without `build.json`'s format.
        let build = &json["setup"]["source"]["build"];
        assert_eq!(build["id"], "b11538");
        assert_eq!(build["host_compiler"]["version"], "16.2.1");
        assert_eq!(build.get("format"), None);
        assert_eq!(build.get("patches"), None);
        assert_eq!(json["setup"]["model"]["flags"]["temp"], 0.6);
        assert_eq!(json["depths"][0]["samples"][0]["gaps_us"][0], 41_250);
        assert_eq!(json["gpu"][1]["scenario"], "depth");
        assert_eq!(json["gpu"][0]["policy"], true);
    }

    #[test]
    fn a_reading_keeps_every_clock_event_bit() {
        let mut run = run();
        run.gpu[1].clock_events = Some(0x404);
        let read = from_json(&to_json(&run).unwrap()).unwrap();
        assert_eq!(read.gpu[1].clock_events, Some(0x404));
        assert!(read.gpu[0].slowdown.policy);
        assert_eq!(read, run);
    }

    #[test]
    fn a_run_of_format_1_reads_without_policy_or_clock_events() {
        let text = edited(|json| {
            json["format"] = 1.into();
            for reading in json["gpu"].as_array_mut().unwrap() {
                let reading = reading.as_object_mut().unwrap();
                reading.remove("policy");
                reading.remove("clock_events");
            }
        });
        let read = from_json(&text).unwrap();
        assert_eq!(read.gpu.len(), run().gpu.len());
        assert!(
            read.gpu
                .iter()
                .all(|r| r.clock_events.is_none() && !r.slowdown.policy)
        );
    }

    #[test]
    fn a_run_of_format_2_reads_the_build_json_it_embedded() {
        for format in [1, 2] {
            let text = edited(|json| {
                json["format"] = format.into();
                json["setup"]["source"]["build"]["format"] = 1.into();
            });
            assert_eq!(from_json(&text).unwrap().setup, run().setup, "{format}");
        }
        assert_eq!(
            error(&edited(
                |json| json["setup"]["source"]["build"]["format"] = 2.into()
            )),
            "the build's format 2 is not supported (runs held build.json's format 1)"
        );
    }

    fn edited(edit: impl FnOnce(&mut Value)) -> String {
        let mut json: Value = serde_json::from_str(&to_json(&run()).unwrap()).unwrap();
        edit(&mut json);
        json.to_string()
    }

    fn error(text: &str) -> String {
        format!("{:#}", from_json(text).unwrap_err())
    }

    #[test]
    fn another_format_or_an_unknown_field_is_refused() {
        let later = "run format 4 is not supported (this ai-ctl reads formats 1, 2 and 3)";
        assert_eq!(error(&edited(|json| json["format"] = 4.into())), later);
        // A later format's new fields do not hide its format.
        let notes = edited(|json| {
            json["format"] = 4.into();
            json["notes"] = "x".into();
            json.as_object_mut().unwrap().remove("started");
        });
        assert_eq!(error(&notes), later);
        let missing = error(&edited(|json| {
            json.as_object_mut().unwrap().remove("format");
        }));
        assert!(
            missing.starts_with("malformed run: missing field `format`"),
            "{missing}"
        );
        let unknown = error(&edited(|json| json["speed"] = 1.into()));
        assert!(
            unknown.starts_with("malformed run: unknown field `speed`"),
            "{unknown}"
        );
        assert_eq!(
            error(&edited(|json| json["gpu"][0]["scenario"] = "load".into())),
            "no scenario `load`"
        );
        assert_eq!(
            error(&edited(
                |json| json["setup"]["model"]["flags"]["temp"] = Value::Null
            )),
            "flag `temp` is not a scalar: null"
        );
    }

    #[test]
    fn a_saved_run_is_listed_and_read_back() {
        let dir = tempfile::tempdir().unwrap();
        let store = FsRunStore::new(dir.path().join("bench"));
        assert!(store.ids().unwrap().is_empty());
        let run = run();
        let path = store.save(&run).unwrap();
        assert_eq!(
            path,
            dir.path().join("bench/20261011T120000Z-qwen3.8-27b.json")
        );
        let mut later = testing::run();
        later.started = Timestamp::parse("2026-10-11T13:00:00Z").unwrap();
        store.save(&later).unwrap();
        // A run being saved, and a file of another kind, are not runs.
        fs::write(dir.path().join("bench/.20261011T140000Z-m.json.42"), "{").unwrap();
        fs::write(dir.path().join("bench/notes.txt"), "").unwrap();
        assert_eq!(
            store.ids().unwrap(),
            [
                "20261011T120000Z-qwen3.8-27b",
                "20261011T130000Z-qwen3.8-27b"
            ]
        );
        assert_eq!(store.load("20261011T120000Z-qwen3.8-27b").unwrap(), run);

        let mut more = run.clone();
        more.n_ctx = 4096;
        assert_eq!(store.replace(&more).unwrap(), path);
        assert_eq!(store.load("20261011T120000Z-qwen3.8-27b").unwrap(), more);
        assert_eq!(store.ids().unwrap().len(), 2, "no file left beside it");
    }

    #[test]
    fn a_saved_run_is_never_replaced() {
        let dir = tempfile::tempdir().unwrap();
        let store = FsRunStore::new(dir.path().to_owned());
        store.save(&run()).unwrap();
        let error = store.save(&testing::run()).unwrap_err().to_string();
        assert!(error.starts_with("a run is already saved as "), "{error}");
        assert_eq!(store.load(&run().id()).unwrap(), run());
        let names: Vec<_> = fs::read_dir(dir.path()).unwrap().collect();
        assert_eq!(names.len(), 1);
    }

    #[test]
    fn a_save_that_cannot_write_leaves_nothing_behind() {
        let dir = tempfile::tempdir().unwrap();
        let store = FsRunStore::new(dir.path().to_owned());
        // Writing to /dev/full fails as a full disk does, once the file
        // is open.
        let temporary = files::temporary_for(&store.path(&run().id())).unwrap();
        std::os::unix::fs::symlink("/dev/full", &temporary).unwrap();
        let error = format!("{:#}", store.save(&run()).unwrap_err());
        assert!(
            error.ends_with("No space left on device (os error 28)"),
            "{error}"
        );
        let names: Vec<_> = fs::read_dir(dir.path()).unwrap().collect();
        assert!(names.is_empty(), "{names:?}");
    }

    #[test]
    fn an_unknown_or_invalid_name_is_refused() {
        let dir = tempfile::tempdir().unwrap();
        let store = FsRunStore::new(dir.path().to_owned());
        assert_eq!(
            store.load("20261011T120000Z-m").unwrap_err().to_string(),
            "no run is saved as 20261011T120000Z-m; `ai-ctl bench list` lists them"
        );
        for id in ["", "../x", ".hidden"] {
            assert_eq!(
                store.load(id).unwrap_err().to_string(),
                format!("{id:?} is not the name of a run")
            );
        }
        fs::write(dir.path().join("broken.json"), "{").unwrap();
        let error = format!("{:#}", store.load("broken").unwrap_err());
        assert!(error.contains("broken.json: malformed run"), "{error}");
    }
}
