//! `ai-ctl bench`: measures a model through the gateway, each request on
//! stderr as it is measured, then the run's results on stdout.

use std::fmt::Write as _;
use std::io::{self, Write};
use std::process::ExitCode;

use anyhow::{Context, Result, anyhow};

use crate::application::bench::{self, Bench, Event, Options, Recorded, Step};
use crate::domain::bench::{DraftCheck, Drafts, Run, Sample, Scenario, Source, drafts};
use crate::domain::stats::Summary;

use super::cli::BenchAction;
use super::compare;
use super::{Services, table};

/// Runs `ai-ctl bench`.
pub fn run(action: BenchAction, services: &Services) -> Result<ExitCode> {
    match action {
        BenchAction::Run {
            model,
            repeat,
            scenarios,
        } => {
            let scenarios = if scenarios.is_empty() {
                Scenario::ALL.to_vec()
            } else {
                scenarios.into_iter().map(Scenario::from).collect()
            };
            let bench = Bench {
                configurator: services.configurator(),
                power: services.power.as_ref(),
                containers: services.containers.as_ref(),
                gateway: services.gateway.as_ref(),
                api: services.api.as_ref(),
                server: services.server.as_ref(),
                chat: services.chat.as_ref(),
                monitor: services.monitor.as_ref(),
                runs: services.runs.as_ref(),
                clock: services.clock,
            };
            let options = Options {
                model,
                repeat,
                scenarios,
            };
            let mut stderr = io::stderr();
            let recorded = bench.run(&options, &mut |event| {
                if let Some(line) = render_event(&event) {
                    // A write error is ignored: the run goes on.
                    let _ = writeln!(stderr, "{line}");
                }
            })?;
            report(recorded, &mut io::stdout().lock())?;
            Ok(ExitCode::SUCCESS)
        }
        BenchAction::List => {
            super::print(&compare::render_list(&bench::saved(
                services.runs.as_ref(),
            )?))?;
            Ok(ExitCode::SUCCESS)
        }
        BenchAction::Compare { runs } => {
            let runs = bench::load(services.runs.as_ref(), &runs)?;
            super::print(&compare::render_compare(&runs))?;
            Ok(ExitCode::SUCCESS)
        }
    }
}

/// Prints a measured run to `out`, then where it is saved. A run that
/// cannot be saved, or that a failed request ended, is an error once it is
/// printed.
fn report(recorded: Recorded, out: &mut dyn Write) -> Result<()> {
    let Recorded {
        run,
        saved,
        failure,
    } = recorded;
    super::print_to(out, &render_run(&run)).with_context(|| match &saved {
        Ok(path) => format!(
            "the run is saved in {}, but cannot be printed",
            path.display()
        ),
        Err(err) => format!("the run can be neither saved ({err:#}) nor printed"),
    })?;
    match (saved, failure) {
        (Ok(path), failure) => {
            super::print_to(
                out,
                &format!("\nSaved as {} in {}\n", run.id(), path.display()),
            )?;
            failure.map_or(Ok(()), Err)
        }
        (Err(save), None) => Err(save.context("the run is printed above, but cannot be saved")),
        (Err(save), Some(failure)) => Err(anyhow!(
            "{failure:#}; the run is printed above, but cannot be saved: {save:#}"
        )),
    }
}

/// The line an event prints, if any: the overhead's requests are too many
/// to print each.
pub fn render_event(event: &Event) -> Option<String> {
    Some(match event {
        Event::Loading { model } => {
            format!("Reading {model}'s context; the gateway loads the model first if it must")
        }
        Event::Planned { n_ctx, plan } => {
            let depths: Vec<String> = plan
                .depths
                .iter()
                .map(|depth| depth.tokens.to_string())
                .collect();
            format!(
                "Context of {n_ctx} tokens: depths {}, the last one at the limit; {}",
                depths.join(", "),
                repetitions(plan.repeat)
            )
        }
        Event::Scenario(scenario) => title(*scenario).to_owned(),
        Event::Warning(message) => format!("warning: {message}"),
        Event::Measured { step, sample } => match step {
            Step::WarmUp => format!("Warm-up: TTFT {}", duration(sample.ttft_ms())),
            Step::Fill(depth) => format!(
                "  {}: filled, {} tokens evaluated at {}, TTFT {}",
                depth.tokens,
                count(sample.evaluated()),
                speed(sample.prefill_speed()),
                duration(sample.ttft_ms())
            ),
            Step::Generate { depth, repetition } => format!(
                "  {} #{repetition}: {}, TTFT {}",
                depth.tokens,
                generation(sample),
                duration(sample.ttft_ms())
            ),
            Step::Prefill { repetition } => format!(
                "  #{repetition}: {} tokens evaluated at {}, TTFT {}",
                count(sample.evaluated()),
                speed(sample.prefill_speed()),
                duration(sample.ttft_ms())
            ),
            Step::Turn { session, turn } => format!(
                "  session {session}, turn {turn}: {} cached, {} evaluated, TTFT {}",
                count(sample.cached()),
                count(sample.evaluated()),
                duration(sample.ttft_ms())
            ),
            Step::SetAside => return None,
            Step::Overhead { request, direct } => {
                if !direct || request % 10 != 0 {
                    return None;
                }
                format!("  {request} requests each way")
            }
        },
    })
}

fn title(scenario: Scenario) -> &'static str {
    match scenario {
        Scenario::Depth => "Generation by context depth",
        Scenario::Prefill => "Prefill of fresh prompts",
        Scenario::Session => "Agent sessions",
        Scenario::Overhead => "Gateway overhead",
    }
}

/// The results of a run, scenario by scenario.
pub fn render_run(run: &Run) -> String {
    let plan = &run.plan;
    let mut out = format!(
        "\n{} at {}: context of {} tokens, {}\nSource: {}\n",
        run.model,
        run.started,
        run.n_ctx,
        repetitions(plan.repeat),
        source(&run.setup.source)
    );
    if !run.depths.is_empty() {
        let _ = writeln!(
            out,
            "\n{}, {} tokens per request",
            title(Scenario::Depth),
            plan.generate
        );
        let mut rows = vec![row(["depth", "prefill", "generation", "TTFT", "drafts"])];
        for result in &run.depths {
            let limit = if result.depth.at_limit {
                " (limit)"
            } else {
                ""
            };
            rows.push(vec![
                format!("{}{limit}", result.depth.tokens),
                speed(result.fill.prefill_speed()),
                mean_speed(result.generation()),
                mean_duration(result.ttft()),
                accepted(&result.samples),
            ]);
        }
        out.push_str(&table(&rows, "  "));
    }
    if let Some(prefill) = &run.prefill {
        let _ = writeln!(
            out,
            "\n{}, {} tokens each",
            title(Scenario::Prefill),
            plan.prefill
        );
        let mut rows = vec![row(["prefill", "TTFT"])];
        rows.push(vec![
            mean_speed(prefill.prefill()),
            mean_duration(prefill.ttft()),
        ]);
        out.push_str(&table(&rows, "  "));
    }
    if let Some(session) = &run.session {
        let _ = writeln!(
            out,
            "\n{}: {} of {} turns, {} tokens per reply",
            title(Scenario::Session),
            session.sessions.len(),
            plan.session.turns,
            plan.generate
        );
        let mut rows = vec![row(["turn", "TTFT", "cached", "evaluated"])];
        for turn in session.turns() {
            let tokens = |summary: Option<Summary>| {
                summary.map_or("–".to_owned(), |s| format!("{:.0}", s.median))
            };
            rows.push(vec![
                turn.turn.to_string(),
                mean_duration(turn.ttft),
                tokens(turn.cached),
                tokens(turn.evaluated),
            ]);
        }
        out.push_str(&table(&rows, "  "));
        out.push_str(if session.cache_grows() {
            "  The cache grew at every turn of every session.\n"
        } else {
            "  The cache did not grow at every turn of every session.\n"
        });
        let samples: Vec<Sample> = session.sessions.concat();
        if drafts(&samples).is_some() {
            let _ = writeln!(out, "  Drafts accepted: {}", accepted(&samples));
        }
    }
    if let Some(overhead) = &run.overhead {
        let _ = writeln!(
            out,
            "\n{}: {} requests each way",
            title(Scenario::Overhead),
            overhead.gateway.len()
        );
        let added = overhead.added();
        let mut rows = vec![row(["", "gateway", "direct", "added"])];
        for (name, measure, added) in [
            (
                "TTFT",
                Sample::ttft_ms as fn(&Sample) -> f64,
                added.map(|added| added.ttft_ms),
            ),
            ("total", Sample::total_ms, added.map(|added| added.total_ms)),
        ] {
            let (gateway, direct) = overhead.medians(measure).unzip();
            rows.push(vec![
                name.to_owned(),
                gateway.map_or("–".to_owned(), |ms| format!("{ms:.1} ms")),
                direct.map_or("–".to_owned(), |ms| format!("{ms:.1} ms")),
                added.map_or("–".to_owned(), |ms| format!("{ms:+.1} ms")),
            ]);
        }
        out.push_str(&table(&rows, "  "));
    }
    if let Some(check) = &run.drafts {
        out.push_str(&render_drafts(check));
    }
    if !run.gpu.is_empty() {
        out.push_str(&render_gpu(run));
    }
    out
}

/// The GPU over each scenario of a run, then over the whole run.
fn render_gpu(run: &Run) -> String {
    let mut out = format!("\nGPU: {} readings\n", run.gpu.len());
    let mut rows = vec![row([
        "",
        "peak memory",
        "max temp",
        "SM clock",
        "power",
        "slowed by",
        "policy limit",
    ])];
    let parts = Scenario::ALL
        .iter()
        .map(|&scenario| (scenario.name(), Some(scenario)))
        .chain([("run", None)]);
    for (name, scenario) in parts {
        let Some(gpu) = run.gpu(scenario) else {
            if scenario.is_some_and(|scenario| run.holds(scenario)) {
                rows.push(vec![name.to_owned(), "no reading".to_owned()]);
            }
            continue;
        };
        let causes: Vec<String> = [
            ("power", gpu.power_capped),
            ("thermal", gpu.thermal),
            ("hardware", gpu.hardware),
        ]
        .iter()
        .filter(|(_, share)| *share > 0.0)
        .map(|(cause, share)| format!("{cause} {:.0}%", share * 100.0))
        .collect();
        rows.push(vec![
            name.to_owned(),
            format!("{} MiB", gpu.peak_memory_mib),
            format!("{} °C", gpu.max_temperature_c),
            format!(
                "{:.0} MHz, min {:.0}",
                gpu.sm_clock_mhz.median, gpu.sm_clock_mhz.min
            ),
            mean(Some(gpu.power_w), 1.0, 0, " W"),
            if causes.is_empty() {
                "–".to_owned()
            } else {
                causes.join(", ")
            },
            if gpu.policy > 0.0 {
                format!("{:.0}%", gpu.policy * 100.0)
            } else {
                "–".to_owned()
            },
        ]);
    }
    out.push_str(&table(&rows, "  "));
    out
}

/// The drafts of speculative decoding over a run, as the server counted
/// them, and whether its requests account for them all.
fn render_drafts(check: &DraftCheck) -> String {
    let server = check.server();
    if server.drafts.proposed == 0 && check.requests.proposed == 0 {
        return "\nDrafts: none proposed\n".to_owned();
    }
    let mut out = format!(
        "\nDrafts: {} of {} accepted, {} tokens per decoding step\n",
        percent(server.drafts.acceptance()),
        server.drafts.proposed,
        server
            .tokens_per_step()
            .map_or("–".to_owned(), |tokens| format!("{tokens:.2}"))
    );
    let acceptance = server.acceptance_at();
    if !acceptance.is_empty() {
        let mut position = vec!["position".to_owned()];
        let mut accepted = vec!["accepted".to_owned()];
        for (index, share) in acceptance.iter().enumerate() {
            position.push((index + 1).to_string());
            accepted.push(percent(Some(*share)));
        }
        out.push_str(&table(&[position, accepted], "  "));
    }
    if check.agrees() {
        out.push_str("  The requests account for every draft the server counted.\n");
    } else {
        let _ = writeln!(
            out,
            "  The server counted {} drafts, {} accepted; the requests reported {}, {} accepted: \
             another client used the model during the run.",
            server.drafts.proposed,
            server.drafts.accepted,
            check.requests.proposed,
            check.requests.accepted
        );
    }
    out
}

/// Where a model's server came from: a build, or an image and the start
/// of its digest.
pub(super) fn source(source: &Source) -> String {
    match source {
        Source::Build(build) => format!("{} {}", build.project, build.id),
        Source::Image {
            reference,
            digest: Some(digest),
        } => format!("{reference} ({})", &digest[..digest.len().min(19)]),
        Source::Image {
            reference,
            digest: None,
        } => reference.clone(),
    }
}

/// `count` repetitions, in words.
fn repetitions(count: u32) -> String {
    if count == 1 {
        "1 repetition".to_owned()
    } else {
        format!("{count} repetitions")
    }
}

/// The share of the drafts of `samples` accepted.
fn accepted(samples: &[Sample]) -> String {
    percent(drafts(samples).and_then(Drafts::acceptance))
}

pub(super) fn percent(share: Option<f64>) -> String {
    share.map_or("–".to_owned(), |share| format!("{:.1}%", share * 100.0))
}

pub(super) fn row<const N: usize>(cells: [&str; N]) -> Vec<String> {
    cells.iter().map(|&cell| cell.to_owned()).collect()
}

/// A mean divided by `scale`, with its standard deviation when there is
/// one.
pub(super) fn mean(summary: Option<Summary>, scale: f64, digits: usize, unit: &str) -> String {
    let Some(summary) = summary else {
        return "–".to_owned();
    };
    let value = |v: f64| format!("{:.digits$}", v / scale);
    match summary.stddev {
        Some(stddev) => format!("{} ± {}{unit}", value(summary.mean), value(stddev)),
        None => format!("{}{unit}", value(summary.mean)),
    }
}

/// A mean speed in tokens per second.
pub(super) fn mean_speed(summary: Option<Summary>) -> String {
    mean(summary, 1.0, 1, " t/s")
}

/// A mean duration given in milliseconds, in seconds from one second on.
pub(super) fn mean_duration(summary: Option<Summary>) -> String {
    match summary {
        Some(s) if in_seconds(s.mean) => mean(summary, 1000.0, 2, " s"),
        _ => mean(summary, 1.0, 0, " ms"),
    }
}

/// Whether `ms` is written in seconds: from what rounds to a second on.
fn in_seconds(ms: f64) -> bool {
    ms.round() >= 1000.0
}

fn duration(ms: f64) -> String {
    if in_seconds(ms) {
        format!("{:.2} s", ms / 1000.0)
    } else {
        format!("{ms:.0} ms")
    }
}

fn speed(speed: Option<f64>) -> String {
    speed.map_or("–".to_owned(), |speed| format!("{speed:.1} t/s"))
}

fn count(count: Option<u64>) -> String {
    count.map_or("–".to_owned(), |count| count.to_string())
}

/// The tokens a request generated, and at what speed.
fn generation(sample: &Sample) -> String {
    let generated = sample
        .server
        .map(|server| server.predicted_n)
        .or(sample.usage.map(|usage| usage.completion_tokens));
    format!(
        "{} tokens at {}",
        count(generated),
        speed(sample.generation_speed())
    )
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;
    use crate::domain::bench::SpecCounters;
    use crate::domain::bench::testing::{run, sample};

    const SAVED: &str = "/data/bench/20261011T120000Z-qwen3.8-27b.json";
    const FULL: &str = "cannot write /data/bench/20261011T120000Z-qwen3.8-27b.json: No space left \
                        on device (os error 28)";
    const FAILED: &str = "turn 2 of session 1: answered 500: the slot is busy; the run holds only \
                          the scenarios it completed: depth and prefill";

    /// What [`report`] writes, and its error.
    fn reported(saved: Result<PathBuf>, failure: Option<&str>) -> (String, Option<String>) {
        let recorded = Recorded {
            run: run(),
            saved,
            failure: failure.map(|failure| anyhow!(failure.to_owned())),
        };
        let mut out = Vec::new();
        let error = report(recorded, &mut out).err();
        let out = String::from_utf8(out).unwrap();
        (out, error.map(|error| format!("{error:#}")))
    }

    #[test]
    fn a_run_is_printed_then_where_it_is_saved() {
        let (out, error) = reported(Ok(SAVED.into()), None);
        assert_eq!(error, None);
        let saved = format!("\nSaved as {} in {SAVED}\n", run().id());
        assert_eq!(out, render_run(&run()) + &saved);
    }

    #[test]
    fn a_run_that_cannot_be_saved_fails_once_printed() {
        let (out, error) = reported(Err(anyhow!(FULL)), None);
        assert_eq!(
            error.unwrap(),
            format!("the run is printed above, but cannot be saved: {FULL}")
        );
        assert_eq!(out, render_run(&run()));
    }

    #[test]
    fn a_run_a_request_ended_is_printed_and_saved_before_its_error() {
        let (out, error) = reported(Ok(SAVED.into()), Some(FAILED));
        assert_eq!(error.unwrap(), FAILED);
        let saved = format!("\nSaved as {} in {SAVED}\n", run().id());
        assert_eq!(out, render_run(&run()) + &saved);

        let (out, error) = reported(Err(anyhow!(FULL)), Some(FAILED));
        assert_eq!(
            error.unwrap(),
            format!("{FAILED}; the run is printed above, but cannot be saved: {FULL}")
        );
        assert_eq!(out, render_run(&run()));
    }

    #[test]
    fn a_run_shows_each_scenario_in_a_table() {
        assert_eq!(
            render_run(&run()),
            "
qwen3.8-27b at 2026-10-11T12:00:00Z: context of 98304 tokens, 2 repetitions
Source: ghcr.io/ggml-org/llama.cpp:server-cuda-b11538

Generation by context depth, 512 tokens per request
  depth          prefill     generation      TTFT           drafts
  8192           2000.0 t/s  62.0 ± 2.8 t/s  190 ± 14 ms    80.0%
  97791 (limit)  2000.0 t/s  62.0 ± 2.8 t/s  1.49 ± 0.01 s  80.0%

Prefill of fresh prompts, 40960 tokens each
  prefill            TTFT
  1307.3 ± 17.7 t/s  31.30 ± 0.42 s

Agent sessions: 2 of 10 turns, 512 tokens per reply
  turn  TTFT           cached  evaluated
  1     6.05 ± 0.07 s  40      8152
  2     920 ± 28 ms    8700    1040
  The cache grew at every turn of every session.

Gateway overhead: 2 requests each way
         gateway   direct    added
  TTFT   46.5 ms   45.0 ms   +1.5 ms
  total  297.0 ms  294.5 ms  +2.5 ms

Drafts: 80.0% of 9000 accepted, 3.40 tokens per decoding step
  position  1      2      3
  accepted  90.0%  80.0%  70.0%
  The requests account for every draft the server counted.

GPU: 4 readings
            peak memory  max temp  SM clock            power      slowed by                             policy limit
  depth     23300 MiB    86 °C     1850 MHz, min 1700  107 ± 4 W  power 100%, thermal 50%               –
  prefill   no reading
  session   20100 MiB    80 °C     1950 MHz, min 1950  108 W      power 100%, hardware 100%             –
  overhead  no reading
  run       23300 MiB    86 °C     1975 MHz, min 1700  90 ± 34 W  power 75%, thermal 25%, hardware 25%  25%
"
        );
    }

    #[test]
    fn drafts_another_client_made_are_pointed_out() {
        let mut check = run().drafts.unwrap();
        check.after.drafts.proposed += 120;
        check.after.drafts.accepted += 100;
        let text = render_drafts(&check);
        assert!(
            text.ends_with(
                "  The server counted 9120 drafts, 7300 accepted; the requests reported 9000, \
                 7200 accepted: another client used the model during the run.\n"
            ),
            "{text}"
        );
        let none = DraftCheck {
            requests: Drafts::default(),
            before: SpecCounters::default(),
            after: SpecCounters::default(),
        };
        assert_eq!(render_drafts(&none), "\nDrafts: none proposed\n");
    }

    #[test]
    fn a_run_shows_only_the_scenarios_it_ran() {
        let mut run = run();
        run.depths.clear();
        run.prefill = None;
        run.overhead = None;
        run.drafts = None;
        run.gpu.clear();
        let session = run.session.as_mut().unwrap();
        session.sessions[1][1] = sample(30, 9_710, 5_900, 512, 8_000);
        let text = render_run(&run);
        assert!(text.contains("\nAgent sessions: 2 of 10 turns"), "{text}");
        assert!(text.ends_with("  The cache did not grow at every turn of every session.\n"));
        for absent in ["depth", "Prefill of", "overhead", "Drafts:", "GPU"] {
            assert!(!text.contains(absent), "{absent}: {text}");
        }
    }

    #[test]
    fn a_measure_without_server_timings_shows_what_the_client_saw() {
        let mut sample = sample(0, 0, 250, 0, 0);
        sample.server = None;
        sample.usage = None;
        let line = render_event(&Event::Measured {
            step: Step::Turn {
                session: 1,
                turn: 1,
            },
            sample,
        });
        assert_eq!(
            line.as_deref(),
            Some("  session 1, turn 1: – cached, – evaluated, TTFT 250 ms")
        );
    }

    #[test]
    fn each_request_prints_a_line_but_the_overheads() {
        let run = run();
        let depth = run.depths[0].depth;
        let lines: Vec<Option<String>> = [
            Event::Loading {
                model: "qwen3.8-27b".into(),
            },
            Event::Planned {
                n_ctx: 98_304,
                plan: run.plan.clone(),
            },
            Event::Measured {
                step: Step::WarmUp,
                sample: sample(0, 700, 350, 16, 250),
            },
            Event::Scenario(Scenario::Depth),
            Event::Measured {
                step: Step::Fill(depth),
                sample: run.depths[0].fill.clone(),
            },
            Event::Measured {
                step: Step::Generate {
                    depth,
                    repetition: 2,
                },
                sample: run.depths[0].samples[1].clone(),
            },
            Event::Measured {
                step: Step::Prefill { repetition: 1 },
                sample: run.prefill.as_ref().unwrap().samples[0].clone(),
            },
            Event::Measured {
                step: Step::Turn {
                    session: 2,
                    turn: 2,
                },
                sample: run.session.as_ref().unwrap().sessions[1][1].clone(),
            },
            Event::Measured {
                step: Step::SetAside,
                sample: sample(0, 700, 350, 16, 250),
            },
            Event::Measured {
                step: Step::Overhead {
                    request: 10,
                    direct: false,
                },
                sample: sample(700, 1, 46, 16, 250),
            },
            Event::Measured {
                step: Step::Overhead {
                    request: 9,
                    direct: true,
                },
                sample: sample(700, 1, 46, 16, 250),
            },
            Event::Measured {
                step: Step::Overhead {
                    request: 10,
                    direct: true,
                },
                sample: sample(700, 1, 46, 16, 250),
            },
        ]
        .iter()
        .map(render_event)
        .collect();
        let printed: Vec<&str> = lines.iter().flatten().map(String::as_str).collect();
        assert_eq!(
            printed,
            [
                "Reading qwen3.8-27b's context; the gateway loads the model first if it must",
                "Context of 98304 tokens: depths 8192, 32768, 65536, 97791, the last one at the \
                 limit; 2 repetitions",
                "Warm-up: TTFT 350 ms",
                "Generation by context depth",
                "  8192: filled, 8192 tokens evaluated at 2000.0 t/s, TTFT 4.10 s",
                "  8192 #2: 512 tokens at 60.0 t/s, TTFT 200 ms",
                "  #1: 40915 tokens evaluated at 1319.8 t/s, TTFT 31.00 s",
                "  session 2, turn 2: 8700 cached, 1040 evaluated, TTFT 940 ms",
                "  10 requests each way",
            ]
        );
    }
}
