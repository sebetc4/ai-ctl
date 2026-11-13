//! `ai-ctl bench list` and `bench compare`: the saved runs, and runs side by
//! side, one table per measure, each run a column.

use std::collections::BTreeSet;
use std::fmt::Write as _;

use anyhow::Result;

use crate::domain::bench::{
    GpuSummary, Run, Sample, Scenario, arrival_gaps, common_scenarios, comparable_counters, drafts,
};
use crate::domain::stats::Summary;

use super::bench::{mean, mean_duration, mean_speed, percent, row, source};
use super::table;

/// The saved runs, one line each.
pub fn render_list(saved: &[(String, Result<Run>)]) -> String {
    if saved.is_empty() {
        return "No saved run; `ai-ctl bench run <model>` makes one.\n".to_owned();
    }
    let mut rows = vec![row(["run", "source", "repetitions", "scenarios"])];
    for (id, run) in saved {
        rows.push(match run {
            Ok(run) => vec![
                id.clone(),
                source(&run.setup.source),
                run.plan.repeat.to_string(),
                names(&run.scenarios()),
            ],
            Err(err) => vec![id.clone(), format!("cannot be read: {err:#}")],
        });
    }
    table(&rows, "  ")
}

/// The names of `scenarios`, in a list.
fn names(scenarios: &[Scenario]) -> String {
    let names: Vec<&str> = scenarios.iter().map(|scenario| scenario.name()).collect();
    names.join(", ")
}

/// How the difference between two runs is told.
#[derive(Clone, Copy)]
enum Change {
    /// In percent of the first run's value.
    Relative,
    /// In percentage points, for shares.
    Points,
    /// Not told: a difference of differences says little.
    None,
}

/// One measure across runs.
struct Measure {
    label: String,
    values: Vec<Option<Summary>>,
    format: fn(&Summary) -> String,
    change: Change,
}

impl Measure {
    fn new(
        label: impl Into<String>,
        values: Vec<Option<Summary>>,
        format: fn(&Summary) -> String,
        change: Change,
    ) -> Self {
        Self {
            label: label.into(),
            values,
            format,
            change,
        }
    }
}

/// A single value as a summary, for a measure made once.
fn one(value: Option<f64>) -> Option<Summary> {
    Summary::of(value)
}

fn speed(summary: &Summary) -> String {
    mean_speed(Some(*summary))
}

fn duration(summary: &Summary) -> String {
    mean_duration(Some(*summary))
}

fn millis(summary: &Summary) -> String {
    format!("{:.1} ms", summary.mean)
}

fn added(summary: &Summary) -> String {
    format!("{:+.1} ms", summary.mean)
}

fn share(summary: &Summary) -> String {
    percent(Some(summary.mean))
}

fn mib(summary: &Summary) -> String {
    format!("{:.0} MiB", summary.mean)
}

fn celsius(summary: &Summary) -> String {
    format!("{:.0} °C", summary.mean)
}

fn mhz(summary: &Summary) -> String {
    format!("{:.0} MHz", summary.mean)
}

fn watts(summary: &Summary) -> String {
    mean(Some(*summary), 1.0, 0, " W")
}

/// The difference of the second run from the first.
fn change(first: Option<&Summary>, second: Option<&Summary>, change: Change) -> String {
    let (Some(first), Some(second)) = (first, second) else {
        return "–".to_owned();
    };
    match change {
        Change::Relative if first.mean != 0.0 => format!(
            "{:+.1}%",
            (second.mean - first.mean) / first.mean.abs() * 100.0
        ),
        Change::Points => format!("{:+.1} pt", (second.mean - first.mean) * 100.0),
        Change::Relative | Change::None => "–".to_owned(),
    }
}

/// A table of `measures`, one column per run; with two runs, a last column
/// tells how the second differs from the first. Nothing when no run has
/// any of them.
fn compare(title: &str, header: &str, measures: Vec<Measure>) -> String {
    let measures: Vec<Measure> = measures
        .into_iter()
        .filter(|measure| measure.values.iter().any(Option::is_some))
        .collect();
    let Some(first) = measures.first() else {
        return String::new();
    };
    let runs = first.values.len();
    let mut head = vec![header.to_owned()];
    head.extend((1..=runs).map(|run| format!("#{run}")));
    let two = runs == 2 && measures.iter().any(|m| !matches!(m.change, Change::None));
    if two {
        head.push("change".to_owned());
    }
    let mut rows = vec![head];
    for measure in &measures {
        let mut cells = vec![measure.label.clone()];
        cells.extend(measure.values.iter().map(|value| {
            value
                .as_ref()
                .map_or("–".to_owned(), |summary| (measure.format)(summary))
        }));
        if two {
            cells.push(change(
                measure.values[0].as_ref(),
                measure.values[1].as_ref(),
                measure.change,
            ));
        }
        rows.push(cells);
    }
    format!("\n{title}\n{}", table(&rows, "  "))
}

/// Runs side by side: what each ran, then one table per measure.
pub fn render_compare(runs: &[Run]) -> String {
    let mut out = String::from("Runs\n");
    let rows: Vec<Vec<String>> = runs
        .iter()
        .enumerate()
        .map(|(index, run)| {
            vec![
                format!("#{}", index + 1),
                run.id(),
                source(&run.setup.source),
                format!("context {}", run.n_ctx),
            ]
        })
        .collect();
    out.push_str(&table(&rows, "  "));
    out.push_str(&differences(runs));
    out.push_str(&depths(runs));
    out.push_str(&compare(
        "Prefill of fresh prompts",
        "",
        vec![
            Measure::new(
                "speed",
                runs.iter()
                    .map(|run| run.prefill.as_ref().and_then(|p| p.prefill()))
                    .collect(),
                speed,
                Change::Relative,
            ),
            Measure::new(
                "TTFT",
                runs.iter()
                    .map(|run| run.prefill.as_ref().and_then(|p| p.ttft()))
                    .collect(),
                duration,
                Change::Relative,
            ),
        ],
    ));
    out.push_str(&sessions(runs));
    out.push_str(&overhead(runs));
    out.push_str(&arrivals(runs));
    out.push_str(&drafts_accepted(runs));
    out.push_str(&gpu(runs));
    out
}

/// The command lines and machines that differ between runs.
fn differences(runs: &[Run]) -> String {
    let mut out = String::new();
    let lines: Vec<BTreeSet<&String>> = runs
        .iter()
        .map(|run| run.setup.command.iter().collect())
        .collect();
    let mut rows = Vec::new();
    for (index, run) in runs.iter().enumerate() {
        for line in &run.setup.command {
            if !lines.iter().all(|other| other.contains(line)) {
                rows.push(vec![format!("#{}", index + 1), line.clone()]);
            }
        }
    }
    if !rows.is_empty() {
        out.push_str("\nCommand lines that differ\n");
        out.push_str(&table(&rows, "  "));
    }
    let machines: Vec<String> = runs
        .iter()
        .map(|run| {
            let machine = &run.setup.machine;
            format!(
                "{}, driver {}, CUDA {}, profile {}, {}",
                machine.gpu,
                machine.driver,
                machine.cuda,
                machine.power_profile.as_deref().unwrap_or("unknown"),
                match machine.ac_power {
                    Some(true) => "on AC power",
                    Some(false) => "on battery",
                    None => "power supply unknown",
                }
            )
        })
        .collect();
    if machines.iter().any(|machine| machine != &machines[0]) {
        out.push_str("\nMachines that differ\n");
        let rows: Vec<Vec<String>> = machines
            .into_iter()
            .enumerate()
            .map(|(index, machine)| vec![format!("#{}", index + 1), machine])
            .collect();
        out.push_str(&table(&rows, "  "));
    }
    out
}

/// Generation, the time to the first token and the fill's prefill, by
/// depth: the standard depths, then the context's limit.
fn depths(runs: &[Run]) -> String {
    let standard: BTreeSet<u32> = runs
        .iter()
        .flat_map(|run| &run.depths)
        .filter(|result| !result.depth.at_limit)
        .map(|result| result.depth.tokens)
        .collect();
    let limits: BTreeSet<u32> = runs
        .iter()
        .flat_map(|run| &run.depths)
        .filter(|result| result.depth.at_limit)
        .map(|result| result.depth.tokens)
        .collect();
    let mut keys: Vec<(String, Option<u32>)> = standard
        .iter()
        .map(|&tokens| (tokens.to_string(), Some(tokens)))
        .collect();
    if !limits.is_empty() {
        let label = match limits.iter().collect::<Vec<_>>()[..] {
            [tokens] => format!("{tokens} (limit)"),
            _ => "limit".to_owned(),
        };
        keys.push((label, None));
    }
    let measures = |measure: fn(&crate::domain::bench::DepthResult) -> Option<Summary>,
                    format: fn(&Summary) -> String| {
        keys.iter()
            .map(|(label, tokens)| {
                let values = runs
                    .iter()
                    .map(|run| {
                        run.depths
                            .iter()
                            .find(|result| match tokens {
                                Some(tokens) => {
                                    !result.depth.at_limit && result.depth.tokens == *tokens
                                }
                                None => result.depth.at_limit,
                            })
                            .and_then(measure)
                    })
                    .collect();
                Measure::new(label.clone(), values, format, Change::Relative)
            })
            .collect()
    };
    let mut out = compare(
        "Generation by context depth, tokens/s",
        "depth",
        measures(|result| result.generation(), speed),
    );
    out.push_str(&compare(
        "Time to the first token on the filled context",
        "depth",
        measures(|result| result.ttft(), duration),
    ));
    out.push_str(&compare(
        "Prefill of each depth's fill, tokens/s",
        "depth",
        measures(|result| one(result.fill.prefill_speed()), speed),
    ));
    out
}

/// The time to the first token of each turn of the sessions.
fn sessions(runs: &[Run]) -> String {
    let turns: Vec<_> = runs
        .iter()
        .map(|run| run.session.as_ref().map(|session| session.turns()))
        .collect();
    let count = turns.iter().flatten().map(Vec::len).max().unwrap_or(0);
    let measures = (0..count)
        .map(|index| {
            let values = turns
                .iter()
                .map(|turns| turns.as_ref()?.get(index)?.ttft)
                .collect();
            Measure::new((index + 1).to_string(), values, duration, Change::Relative)
        })
        .collect();
    let mut out = compare("Session: time to the first token", "turn", measures);
    let stalled: Vec<String> = runs
        .iter()
        .enumerate()
        .filter(|(_, run)| run.session.as_ref().is_some_and(|s| !s.cache_grows()))
        .map(|(index, _)| format!("#{}", index + 1))
        .collect();
    if !stalled.is_empty() {
        let _ = writeln!(
            out,
            "  The cache did not grow at every turn in {}.",
            stalled.join(", ")
        );
    }
    out
}

/// What the gateway adds, run by run.
fn overhead(runs: &[Run]) -> String {
    let added_by = |pick: fn(&crate::domain::bench::Overhead) -> f64| {
        runs.iter()
            .map(|run| one(run.overhead.as_ref()?.added().map(|added| pick(&added))))
            .collect()
    };
    compare(
        "Gateway overhead, added to the median",
        "",
        vec![
            Measure::new("TTFT", added_by(|added| added.ttft_ms), added, Change::None),
            Measure::new(
                "total",
                added_by(|added| added.total_ms),
                added,
                Change::None,
            ),
        ],
    )
}

/// The repetitions at every depth of `run`: generation on a filled context.
fn depth_samples(run: &Run) -> Vec<Sample> {
    run.depths
        .iter()
        .flat_map(|result| result.samples.iter().cloned())
        .collect()
}

/// Every turn of every session of `run`.
fn session_samples(run: &Run) -> Vec<Sample> {
    run.session
        .iter()
        .flat_map(|session| session.sessions.concat())
        .collect()
}

/// The gaps between arrivals of tokens, generating at depth and in
/// sessions: a decoding step, and its stalls.
fn arrivals(runs: &[Run]) -> String {
    let mut measures = Vec::new();
    for (scenario, samples) in [
        ("depth", depth_samples as fn(&Run) -> Vec<Sample>),
        ("session", session_samples),
    ] {
        let gaps: Vec<Option<Summary>> =
            runs.iter().map(|run| arrival_gaps(&samples(run))).collect();
        measures.push(Measure::new(
            format!("{scenario}, median"),
            gaps.iter()
                .map(|gaps| one(gaps.map(|g| g.median)))
                .collect(),
            millis,
            Change::Relative,
        ));
        measures.push(Measure::new(
            format!("{scenario}, p99"),
            gaps.iter()
                .map(|gaps| one(gaps.and_then(|g| g.p99)))
                .collect(),
            millis,
            Change::Relative,
        ));
    }
    compare("Gaps between token arrivals", "", measures)
}

/// The share of drafts accepted, by scenario, then as the server counted
/// them over the run, position by position.
fn drafts_accepted(runs: &[Run]) -> String {
    let accepted = |samples: fn(&Run) -> Vec<Sample>| {
        runs.iter()
            .map(|run| one(drafts(&samples(run)).and_then(|d| d.acceptance())))
            .collect()
    };
    let mut measures = vec![
        Measure::new("depth", accepted(depth_samples), share, Change::Points),
        Measure::new("session", accepted(session_samples), share, Change::Points),
    ];
    let Some(counters) = comparable_counters(runs) else {
        return compare("Drafts accepted", "", measures);
    };
    measures.extend([
        Measure::new(
            "run",
            counters
                .iter()
                .map(|server| one(server.as_ref()?.drafts.acceptance()))
                .collect(),
            share,
            Change::Points,
        ),
        Measure::new(
            "tokens per step",
            counters
                .iter()
                .map(|server| one(server.as_ref()?.tokens_per_step()))
                .collect(),
            tokens_per_step,
            Change::Relative,
        ),
    ]);
    let positions = counters
        .iter()
        .flatten()
        .map(|server| server.acceptance_at().len())
        .max()
        .unwrap_or(0);
    for position in 0..positions {
        measures.push(Measure::new(
            format!("position {}", position + 1),
            counters
                .iter()
                .map(|server| one(server.as_ref()?.acceptance_at().get(position).copied()))
                .collect(),
            share,
            Change::Points,
        ));
    }
    compare("Drafts accepted", "", measures)
}

fn tokens_per_step(summary: &Summary) -> String {
    format!("{:.2}", summary.mean)
}

/// The GPU over the scenarios every run ran.
fn gpu(runs: &[Run]) -> String {
    let common = common_scenarios(runs);
    let summaries: Vec<_> = runs.iter().map(|run| run.gpu_over(&common)).collect();
    let measure = |label: &str, pick: fn(&GpuSummary) -> f64, format, change| {
        Measure::new(
            label,
            summaries
                .iter()
                .map(|gpu| one(gpu.as_ref().map(pick)))
                .collect(),
            format,
            change,
        )
    };
    compare(
        &format!("GPU over {}", names(&common)),
        "",
        vec![
            measure(
                "peak memory",
                |g| g.peak_memory_mib as f64,
                mib,
                Change::Relative,
            ),
            measure(
                "max temperature",
                |g| f64::from(g.max_temperature_c),
                celsius,
                Change::Relative,
            ),
            measure(
                "SM clock, median",
                |g| g.sm_clock_mhz.median,
                mhz,
                Change::Relative,
            ),
            measure("power, mean", |g| g.power_w.mean, watts, Change::Relative),
            measure("slowed by heat", |g| g.thermal, share, Change::Points),
        ],
    )
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;
    use crate::domain::bench::Source;
    use crate::domain::bench::testing::{run, setup};
    use crate::domain::timestamp::Timestamp;

    /// The test run again an hour later, on another image with drafts of
    /// four tokens: generation 10% faster, its session's TTFT halved.
    fn faster() -> Run {
        let mut run = run();
        run.started = Timestamp::parse("2026-10-11T13:00:00Z").unwrap();
        run.setup.source = Source::Image {
            reference: "ghcr.io/ggml-org/llama.cpp:server-cuda13".into(),
            digest: Some(format!("sha256:{}", "3f".repeat(32))),
        };
        run.setup.command.push("--spec-draft-n-max 4".into());
        run.setup.machine.power_profile = Some("balanced".into());
        for result in &mut run.depths {
            for sample in &mut result.samples {
                if let Some(server) = sample.server.as_mut() {
                    server.predicted_ms /= 1.1;
                }
            }
        }
        for session in &mut run.session.as_mut().unwrap().sessions {
            for sample in session {
                sample.client.ttft /= 2;
            }
        }
        run
    }

    #[test]
    fn two_runs_are_set_side_by_side_with_the_change() {
        let mut first = run();
        first.depths[0].samples[0].client.gaps = vec![Duration::from_millis(40); 100];
        assert_eq!(
            render_compare(&[first, faster()]),
            "\
Runs
  #1  20261011T120000Z-qwen3.8-27b  ghcr.io/ggml-org/llama.cpp:server-cuda-b11538                   context 98304
  #2  20261011T130000Z-qwen3.8-27b  ghcr.io/ggml-org/llama.cpp:server-cuda13 (sha256:3f3f3f3f3f3f)  context 98304

Command lines that differ
  #2  --spec-draft-n-max 4

Machines that differ
  #1  NVIDIA GeForce RTX 5090 Laptop GPU, driver 615.71.09, CUDA 13.4.0, profile performance, on AC power
  #2  NVIDIA GeForce RTX 5090 Laptop GPU, driver 615.71.09, CUDA 13.4.0, profile balanced, on AC power

Generation by context depth, tokens/s
  depth          #1              #2              change
  8192           62.0 ± 2.8 t/s  68.2 ± 3.1 t/s  +10.0%
  97791 (limit)  62.0 ± 2.8 t/s  68.2 ± 3.1 t/s  +10.0%

Time to the first token on the filled context
  depth          #1             #2             change
  8192           190 ± 14 ms    190 ± 14 ms    +0.0%
  97791 (limit)  1.49 ± 0.01 s  1.49 ± 0.01 s  +0.0%

Prefill of each depth's fill, tokens/s
  depth          #1          #2          change
  8192           2000.0 t/s  2000.0 t/s  +0.0%
  97791 (limit)  2000.0 t/s  2000.0 t/s  +0.0%

Prefill of fresh prompts
         #1                 #2                 change
  speed  1307.3 ± 17.7 t/s  1307.3 ± 17.7 t/s  +0.0%
  TTFT   31.30 ± 0.42 s     31.30 ± 0.42 s     +0.0%

Session: time to the first token
  turn  #1             #2             change
  1     6.05 ± 0.07 s  3.02 ± 0.04 s  -50.0%
  2     920 ± 28 ms    460 ± 14 ms    -50.0%

Gateway overhead, added to the median
         #1       #2
  TTFT   +1.5 ms  +1.5 ms
  total  +2.5 ms  +2.5 ms

Gaps between token arrivals
                 #1       #2  change
  depth, median  40.0 ms  –   –
  depth, p99     40.0 ms  –   –

Drafts accepted
                   #1     #2     change
  depth            80.0%  80.0%  +0.0 pt
  run              80.0%  80.0%  +0.0 pt
  tokens per step  3.40   3.40   +0.0%
  position 1       90.0%  90.0%  +0.0 pt
  position 2       80.0%  80.0%  +0.0 pt
  position 3       70.0%  70.0%  +0.0 pt

GPU over depth, prefill, session, overhead
                    #1         #2         change
  peak memory       23300 MiB  23300 MiB  +0.0%
  max temperature   86 °C      86 °C      +0.0%
  SM clock, median  1950 MHz   1950 MHz   +0.0%
  power, mean       107 W      107 W      +0.0%
  slowed by heat    33.3%      33.3%      +0.0 pt
"
        );
    }

    #[test]
    fn three_runs_have_no_change_column_and_a_missing_measure_shows() {
        let mut without_prefill = faster();
        without_prefill.prefill = None;
        without_prefill.session.as_mut().unwrap().sessions[0][1] = sample_without_cache();
        let text = render_compare(&[run(), without_prefill, faster()]);
        assert!(
            text.contains(
                "Prefill of fresh prompts\n         #1                 #2  #3\n  speed  1307.3 ± 17.7 t/s  –   1307.3 ± 17.7 t/s\n"
            ),
            "{text}"
        );
        assert!(!text.contains("change"), "{text}");
        assert!(
            text.contains("  The cache did not grow at every turn in #2.\n"),
            "{text}"
        );
        // The same command and machine: no difference to list.
        let same = render_compare(&[run(), run()]);
        assert!(!same.contains("differ"), "{same}");
        // Runs of other scenarios: the server's counters and the GPU over
        // a whole run do not compare, only what both ran.
        let mut depth_only = run();
        (depth_only.prefill, depth_only.session, depth_only.overhead) = (None, None, None);
        let text = render_compare(&[run(), depth_only]);
        assert!(text.contains("\nGPU over depth\n"), "{text}");
        assert!(!text.contains("tokens per step"), "{text}");
    }

    #[test]
    fn limits_of_different_sizes_share_one_row() {
        // A model of a smaller context.
        let mut smaller = run();
        smaller.n_ctx = 65_536;
        smaller.depths[1].depth.tokens = 65_023;
        let text = render_compare(&[run(), smaller]);
        assert!(
            text.contains("\n  limit  62.0 ± 2.8 t/s  62.0 ± 2.8 t/s  +0.0%\n"),
            "{text}"
        );
    }

    fn sample_without_cache() -> Sample {
        let mut sample = run().session.unwrap().sessions[0][0].clone();
        sample.client.ttft = Duration::from_secs(6);
        sample
    }

    #[test]
    fn the_list_names_each_runs_source_and_scenarios() {
        let mut bare = run();
        bare.started = Timestamp::parse("2026-10-11T13:00:00Z").unwrap();
        (bare.prefill, bare.overhead) = (None, None);
        bare.setup = setup();
        let saved = vec![
            ("20261011T120000Z-qwen3.8-27b".to_owned(), Ok(run())),
            ("20261011T130000Z-qwen3.8-27b".to_owned(), Ok(bare)),
            (
                "20261011T140000Z-qwen3.8-27b".to_owned(),
                Err(anyhow::anyhow!("malformed run")),
            ),
        ];
        assert_eq!(
            render_list(&saved),
            "  run                           source                                         repetitions  scenarios
  20261011T120000Z-qwen3.8-27b  ghcr.io/ggml-org/llama.cpp:server-cuda-b11538  2            depth, prefill, session, overhead
  20261011T130000Z-qwen3.8-27b  ghcr.io/ggml-org/llama.cpp:server-cuda-b11538  2            depth, session
  20261011T140000Z-qwen3.8-27b  cannot be read: malformed run
"
        );
        assert_eq!(
            render_list(&[]),
            "No saved run; `ai-ctl bench run <model>` makes one.\n"
        );
    }

    #[test]
    fn a_change_from_zero_is_not_told() {
        let zero = Summary::of([0.0]).unwrap();
        let some = Summary::of([5.0]).unwrap();
        assert_eq!(change(Some(&zero), Some(&some), Change::Relative), "–");
        assert_eq!(
            change(Some(&some), Some(&zero), Change::Relative),
            "-100.0%"
        );
    }
}
