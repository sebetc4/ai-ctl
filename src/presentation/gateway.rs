//! `ai-ctl gateway`: installs llama-swap, with its steps on stderr, and
//! prints its configuration.

use std::io::{self, Write};
use std::process::ExitCode;

use anyhow::Result;

use crate::application::gateway::{Event, Installation, Installer, Outcome};

use super::cli::GatewayAction;
use super::{Services, current_line};

/// Runs `ai-ctl gateway` and its subcommands.
pub fn run(action: GatewayAction, services: &Services) -> Result<ExitCode> {
    let output = match action {
        GatewayAction::Install => {
            let installer = Installer {
                downloads: services.downloads.as_ref(),
                store: services.releases.as_ref(),
                programs: services.programs.as_ref(),
            };
            let mut stderr = io::stderr();
            let installation = installer.install(&mut |event| {
                // A write error is ignored: the installation goes on.
                let _ = writeln!(stderr, "{}", render_event(&event));
            })?;
            render_installation(&installation)
        }
        GatewayAction::Config => services.configurator().generate()?.to_yaml(),
    };
    super::print(&output)?;
    Ok(ExitCode::SUCCESS)
}

pub fn render_event(event: &Event) -> String {
    match event {
        Event::Downloading { url } => format!("Downloading {url}"),
        Event::Verified { sha256 } => {
            format!("SHA-256 verified against the digest ai-ctl pins: {sha256}")
        }
        Event::Testing => "Testing the new binary".to_owned(),
        Event::Warning(message) => format!("warning: {message}"),
    }
}

pub fn render_installation(installation: &Installation) -> String {
    let release = installation.release;
    let mut out = match installation.outcome {
        Outcome::Downloaded => format!("Installed llama-swap {release}\n"),
        Outcome::Reused => format!("llama-swap {release} was installed already\n"),
        Outcome::Unchanged => format!("llama-swap {release} is installed and current already\n"),
    };
    current_line(
        &mut out,
        "gateway",
        &installation.binary,
        installation.previous,
    );
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::gateway::{Release, Sha256};

    fn installation(outcome: Outcome, previous: Option<&str>) -> Installation {
        Installation {
            release: Release::parse("v263").unwrap(),
            binary: "/home/me/.local/share/ai-ctl/llama-swap/v263/llama-swap".into(),
            outcome,
            previous: previous.map(|text| Release::parse(text).unwrap()),
        }
    }

    #[test]
    fn an_installation_names_the_binary_and_the_release_it_replaces() {
        assert_eq!(
            render_installation(&installation(Outcome::Downloaded, Some("v262"))),
            "Installed llama-swap v263\n\
             Current gateway: /home/me/.local/share/ai-ctl/llama-swap/v263/llama-swap (was v262)\n"
        );
        assert_eq!(
            render_installation(&installation(Outcome::Reused, None)),
            "llama-swap v263 was installed already\n\
             Current gateway: /home/me/.local/share/ai-ctl/llama-swap/v263/llama-swap\n"
        );
        assert!(
            render_installation(&installation(Outcome::Unchanged, None))
                .starts_with("llama-swap v263 is installed and current already\n")
        );
    }

    #[test]
    fn each_step_is_one_line() {
        let digest = "9f561442a43ed71043105891a10a41019efdc8c023f5a3c807bcf12fbd2a048d";
        let events = [
            Event::Downloading {
                url: "https://x/a.tar.gz".into(),
            },
            Event::Verified {
                sha256: Sha256::parse(digest).unwrap(),
            },
            Event::Testing,
            Event::Warning("cannot remove /tmp/x".into()),
        ];
        let lines: Vec<String> = events.iter().map(render_event).collect();
        assert_eq!(
            lines,
            [
                "Downloading https://x/a.tar.gz".to_owned(),
                format!("SHA-256 verified against the digest ai-ctl pins: {digest}"),
                "Testing the new binary".to_owned(),
                "warning: cannot remove /tmp/x".to_owned(),
            ]
        );
    }
}
