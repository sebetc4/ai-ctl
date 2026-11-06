//! `ai-ctl completions <shell>`: prints a shell completion script.

use std::io::Write;

use clap::CommandFactory;
use clap_complete::{Shell, generate};

use super::cli::Cli;

/// Writes the completion script for `shell` to `out`.
pub fn write(shell: Shell, out: &mut impl Write) {
    let mut command = Cli::command();
    let name = command.get_name().to_owned();
    generate(shell, &mut command, name, out);
}
