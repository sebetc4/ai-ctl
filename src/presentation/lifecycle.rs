//! `ai-ctl up`, `down`, `status` and `logs`: start, stop and read the
//! gateway, with their steps on stderr.

use std::fmt::Write as _;
use std::io::{self, Write};
use std::process::ExitCode;

use anyhow::Result;

use crate::application::lifecycle::{
    Configuration, Down, Event, Lifecycle, LogEnd, Status, Up, kill_command, mib, models_list,
    named,
};
use crate::domain::machine::GpuProcess;

use super::Services;

/// The exit status of `status` when no gateway runs, as `systemctl status`
/// gives for a unit that does not run.
const NOT_RUNNING: u8 = 3;

fn lifecycle(services: &Services) -> Lifecycle<'_> {
    Lifecycle {
        configurator: services.configurator(),
        releases: services.releases.as_ref(),
        gateway: services.gateway.as_ref(),
        processes: services.processes.as_ref(),
        ports: services.ports.as_ref(),
        api: services.api.as_ref(),
        containers: services.containers.as_ref(),
        pause: services.pause,
        now: services.now,
    }
}

/// Runs `ai-ctl up`.
pub fn up(services: &Services) -> Result<ExitCode> {
    let mut stderr = io::stderr();
    let up = lifecycle(services).up(&mut |event| {
        // A write error is ignored: starting goes on.
        let _ = writeln!(stderr, "{}", render_event(&event));
    })?;
    super::print(&render_up(&up))?;
    Ok(ExitCode::SUCCESS)
}

/// Runs `ai-ctl down`.
pub fn down(services: &Services) -> Result<ExitCode> {
    let mut stderr = io::stderr();
    let down = lifecycle(services).down(&mut |event| {
        // A write error is ignored: stopping goes on.
        let _ = writeln!(stderr, "{}", render_event(&event));
    })?;
    super::print(&render_down(&down))?;
    Ok(ExitCode::SUCCESS)
}

/// Runs `ai-ctl status`: exits 0 when the gateway runs, 3 when it does not.
pub fn status(services: &Services) -> Result<ExitCode> {
    let mut stderr = io::stderr();
    let status = lifecycle(services).status(&mut |event| {
        // A write error is ignored: reading goes on.
        let _ = writeln!(stderr, "{}", render_event(&event));
    })?;
    super::print(&render_status(&status))?;
    Ok(if status.gateway.is_some() {
        ExitCode::SUCCESS
    } else {
        ExitCode::from(NOT_RUNNING)
    })
}

/// Runs `ai-ctl logs`: the gateway's log, or one model's stream. A reader
/// that goes (`ai-ctl logs | head`) ends it without an error.
pub fn logs(services: &Services, follow: bool, model: Option<&str>) -> Result<ExitCode> {
    let lifecycle = lifecycle(services);
    let mut stdout = io::stdout().lock();
    let ended = match model {
        Some(model) => lifecycle
            .model_log(model, &mut stdout)
            .map(|()| Some("The gateway ended the stream")),
        None => lifecycle.log(follow, &mut stdout).map(render_log_end),
    };
    match ended {
        Ok(Some(line)) => {
            let _ = writeln!(io::stderr(), "{line}");
        }
        Ok(None) => {}
        Err(err) if super::closed_pipe(&err) => {}
        Err(err) => return Err(err),
    }
    Ok(ExitCode::SUCCESS)
}

pub fn render_event(event: &Event) -> String {
    match event {
        Event::Forgotten { pid, liveness } => format!(
            "The gateway recorded with PID {pid} {}; record removed",
            liveness.describe(*pid)
        ),
        Event::Pulling { image } => format!("Pulling {image}; an image takes several minutes"),
        Event::Starting { release, listen } => format!("Starting llama-swap {release} on {listen}"),
        Event::Unloading => "Unloading the models".to_owned(),
        Event::Stopping { pid } => format!("Stopping llama-swap (PID {pid})"),
        Event::Killing { pid } => {
            format!("llama-swap (PID {pid}) outlived SIGTERM; sending SIGKILL")
        }
        Event::Releasing { processes } => {
            let pids: Vec<String> = processes.iter().map(|p| p.pid.to_string()).collect();
            format!(
                "Waiting for PID {} to release {} MiB of GPU memory",
                pids.join(", "),
                mib(processes)
            )
        }
        Event::Warning(message) => format!("warning: {message}"),
    }
}

pub fn render_up(up: &Up) -> String {
    match up {
        Up::Started {
            pid,
            release,
            listen,
            log,
            preload,
        } => {
            let mut out =
                format!("Gateway up on http://{listen} (llama-swap {release}, PID {pid})\n");
            if !preload.is_empty() {
                let _ = writeln!(out, "Loading in the background: {}", preload.join(", "));
            }
            let _ = writeln!(out, "Log: {}", log.display());
            out
        }
        Up::Running { pid, listen } => {
            format!("Gateway up already on http://{listen} (PID {pid})\n")
        }
    }
}

/// The line `logs` ends with on stderr, if any.
pub fn render_log_end(end: LogEnd) -> Option<&'static str> {
    match end {
        LogEnd::Printed => None,
        LogEnd::NoGateway => Some("No gateway running; nothing more to follow"),
        LogEnd::Stopped => Some("The gateway stopped"),
        LogEnd::Replaced => {
            Some("A new gateway log replaced this one; run `ai-ctl logs -f` again to follow it")
        }
    }
}

/// One line of `status`: its label in a column, then its value.
fn line(out: &mut String, label: &str, value: &str) {
    let _ = writeln!(out, "{label:<12}{value}");
}

pub fn render_status(status: &Status) -> String {
    let mut out = String::new();
    let Some(gateway) = &status.gateway else {
        line(&mut out, "Gateway", "stopped");
        if let Some(gpu) = status.gpu {
            line(
                &mut out,
                "GPU memory",
                &format!("{} of {} MiB in use", gpu.used_mib, gpu.usable_mib),
            );
        }
        render_leftovers(&mut out, &status.leftovers);
        return out;
    };
    line(&mut out, "Gateway", &gateway.running_on());
    match gateway.models.as_deref() {
        Some([]) => line(&mut out, "Models", "none loaded"),
        Some(models) => line(&mut out, "Models", &models_list(models)),
        // The warning on stderr says why.
        None => {}
    }
    if let Some(gpu) = status.gpu {
        let mut memory = format!("{} of {} MiB in use", gpu.used_mib, gpu.usable_mib);
        if !gateway.held.is_empty() {
            let _ = write!(
                memory,
                ", {} MiB by the gateway's models",
                mib(&gateway.held)
            );
        }
        line(&mut out, "GPU memory", &memory);
    }
    match &gateway.configuration {
        Configuration::Current => {}
        Configuration::Outdated => out.push_str(
            "The inventory, a build or llama-swap changed since the gateway started; \
             run `ai-ctl down`, then `ai-ctl up`, to apply it\n",
        ),
        Configuration::Unknown(reason) => {
            let _ = writeln!(
                out,
                "Cannot tell whether the gateway runs on what the inventory, the builds and \
                 llama-swap give now: {reason}"
            );
        }
    }
    render_leftovers(&mut out, &status.leftovers);
    out
}

/// The processes an earlier gateway left on the GPU, with how to end them.
fn render_leftovers(out: &mut String, leftovers: &[GpuProcess]) {
    if leftovers.is_empty() {
        return;
    }
    let _ = writeln!(
        out,
        "Left on the GPU by an earlier gateway: {}; `ai-ctl up` refuses to start until they \
         are gone, and ai-ctl never stops a backend itself: end them with `{}`",
        named(leftovers),
        kill_command(leftovers)
    );
}

pub fn render_down(down: &Down) -> String {
    match down {
        Down::Stopped { pid, released } if released.is_empty() => {
            format!("Gateway stopped (PID {pid})\n")
        }
        Down::Stopped { pid, released } => format!(
            "Gateway stopped (PID {pid}); {} MiB of GPU memory released\n",
            mib(released)
        ),
        Down::NotRunning => "No gateway running\n".to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::lifecycle::{GpuMemory, Serving};
    use crate::application::ports::{Liveness, RunningModel};
    use crate::domain::gateway::Release;

    fn listen() -> std::net::SocketAddr {
        "127.0.0.1:8080".parse().unwrap()
    }

    fn process(pid: u32, mib: Option<u64>) -> GpuProcess {
        GpuProcess {
            pid,
            name: "llama-server".into(),
            memory: mib.map(|mib| mib << 20),
        }
    }

    #[test]
    fn a_stopped_gateway_says_what_it_released() {
        assert_eq!(
            render_down(&Down::Stopped {
                pid: 4242,
                released: vec![process(5000, Some(21094)), process(6000, Some(906))],
            }),
            "Gateway stopped (PID 4242); 22000 MiB of GPU memory released\n"
        );
        assert_eq!(
            render_down(&Down::Stopped {
                pid: 4242,
                released: vec![],
            }),
            "Gateway stopped (PID 4242)\n"
        );
        assert_eq!(render_down(&Down::NotRunning), "No gateway running\n");
    }

    #[test]
    fn a_started_gateway_names_its_address_its_models_and_its_log() {
        let up = Up::Started {
            pid: 4242,
            release: Release::parse("263").unwrap(),
            listen: listen(),
            log: "/home/me/.local/state/ai-ctl/logs/gateway.log".into(),
            preload: vec!["qwen3.8-27b".into()],
        };
        assert_eq!(
            render_up(&up),
            "Gateway up on http://127.0.0.1:8080 (llama-swap v263, PID 4242)\n\
             Loading in the background: qwen3.8-27b\n\
             Log: /home/me/.local/state/ai-ctl/logs/gateway.log\n"
        );
        assert_eq!(
            render_up(&Up::Running {
                pid: 4242,
                listen: listen()
            }),
            "Gateway up already on http://127.0.0.1:8080 (PID 4242)\n"
        );
    }

    #[test]
    fn each_step_is_one_line() {
        let events = [
            Event::Forgotten {
                pid: 7,
                liveness: Liveness::Exited,
            },
            Event::Forgotten {
                pid: 7,
                liveness: Liveness::Reused(Some("/usr/bin/bash".into())),
            },
            Event::Forgotten {
                pid: 7,
                liveness: Liveness::Reused(None),
            },
            Event::Pulling {
                image: "ghcr.io/ggml-org/llama.cpp:server-cuda13".into(),
            },
            Event::Starting {
                release: Release::parse("263").unwrap(),
                listen: listen(),
            },
            Event::Unloading,
            Event::Stopping { pid: 4242 },
            Event::Killing { pid: 4242 },
            Event::Releasing {
                processes: vec![process(5000, Some(21094)), process(6000, None)],
            },
            Event::Warning("cannot remove /x".into()),
        ];
        let lines: Vec<String> = events.iter().map(render_event).collect();
        assert_eq!(
            lines,
            [
                "The gateway recorded with PID 7 has exited; record removed",
                "The gateway recorded with PID 7 is gone; PID 7 now runs /usr/bin/bash; record removed",
                "The gateway recorded with PID 7 is gone; PID 7 now runs another user's program; record removed",
                "Pulling ghcr.io/ggml-org/llama.cpp:server-cuda13; an image takes several minutes",
                "Starting llama-swap v263 on 127.0.0.1:8080",
                "Unloading the models",
                "Stopping llama-swap (PID 4242)",
                "llama-swap (PID 4242) outlived SIGTERM; sending SIGKILL",
                "Waiting for PID 5000, 6000 to release 21094 MiB of GPU memory",
                "warning: cannot remove /x",
            ]
        );
    }

    fn serving(models: Option<Vec<RunningModel>>, held: Vec<GpuProcess>) -> Serving {
        Serving {
            pid: 4242,
            listen: listen(),
            models,
            configuration: Configuration::Current,
            held,
        }
    }

    const GPU: Option<GpuMemory> = Some(GpuMemory {
        used_mib: 19891,
        usable_mib: 24075,
    });

    #[test]
    fn a_running_gateway_shows_its_models_and_their_memory() {
        let status = Status {
            gateway: Some(serving(
                Some(vec![RunningModel {
                    id: "qwen3.6-35b-a3b".into(),
                    state: "ready".into(),
                    proxy: None,
                }]),
                vec![process(5000, Some(19876))],
            )),
            gpu: GPU,
            leftovers: vec![],
        };
        assert_eq!(
            render_status(&status),
            "Gateway     running on http://127.0.0.1:8080 (PID 4242)\n\
             Models      qwen3.6-35b-a3b (ready)\n\
             GPU memory  19891 of 24075 MiB in use, 19876 MiB by the gateway's models\n"
        );
    }

    #[test]
    fn a_gateway_without_models_or_an_api_says_what_it_knows() {
        let mut gateway = serving(Some(vec![]), vec![]);
        let status = Status {
            gateway: Some(gateway.clone()),
            gpu: None,
            leftovers: vec![],
        };
        assert_eq!(
            render_status(&status),
            "Gateway     running on http://127.0.0.1:8080 (PID 4242)\n\
             Models      none loaded\n"
        );
        gateway.models = None;
        gateway.configuration = Configuration::Outdated;
        let status = Status {
            gateway: Some(gateway),
            gpu: None,
            leftovers: vec![],
        };
        assert_eq!(
            render_status(&status),
            "Gateway     running on http://127.0.0.1:8080 (PID 4242)\n\
             The inventory, a build or llama-swap changed since the gateway started; \
             run `ai-ctl down`, then `ai-ctl up`, to apply it\n"
        );
        let mut gateway = serving(Some(vec![]), vec![]);
        gateway.configuration = Configuration::Unknown(
            "cannot size the models to the GPU's memory: cannot read the GPU: NVML: Unknown Error"
                .into(),
        );
        let status = Status {
            gateway: Some(gateway),
            gpu: None,
            leftovers: vec![],
        };
        assert_eq!(
            render_status(&status),
            "Gateway     running on http://127.0.0.1:8080 (PID 4242)\n\
             Models      none loaded\n\
             Cannot tell whether the gateway runs on what the inventory, the builds and \
             llama-swap give now: cannot size the models to the GPU's memory: cannot read the \
             GPU: NVML: Unknown Error\n"
        );
    }

    #[test]
    fn a_stopped_gateway_shows_the_gpu_and_what_was_left_on_it() {
        let status = Status {
            gateway: None,
            gpu: GPU,
            leftovers: vec![process(5000, Some(19876)), process(6000, None)],
        };
        assert_eq!(
            render_status(&status),
            "Gateway     stopped\n\
             GPU memory  19891 of 24075 MiB in use\n\
             Left on the GPU by an earlier gateway: PID 5000 (llama-server, 19876 MiB), \
             PID 6000 (llama-server); `ai-ctl up` refuses to start until they are gone, \
             and ai-ctl never stops a backend itself: end them with `kill 5000 6000`\n"
        );
    }

    #[test]
    fn following_says_why_it_ended() {
        assert_eq!(render_log_end(LogEnd::Printed), None);
        assert_eq!(
            render_log_end(LogEnd::NoGateway),
            Some("No gateway running; nothing more to follow")
        );
        assert_eq!(render_log_end(LogEnd::Stopped), Some("The gateway stopped"));
        assert!(render_log_end(LogEnd::Replaced).is_some());
    }
}
