//! Command-line definition (clap derive).

use std::num::NonZeroUsize;
use std::path::PathBuf;

use clap::{Args, Parser, Subcommand, ValueEnum};
use clap_complete::Shell;

use crate::domain::bench::{REPEAT, Scenario};

/// Diagnose, build, serve and benchmark local AI backends on a single GPU.
#[derive(Debug, Parser)]
#[command(name = "ai-ctl", version, about)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

/// Top-level commands.
#[derive(Debug, Subcommand)]
pub enum Command {
    /// Install the inventory of backends and models into the config directory
    Init {
        /// Directory holding the models' files, GGUF, safetensors or ggml, one subdirectory per model
        #[arg(long, value_name = "DIR")]
        models_dir: PathBuf,
    },
    /// Check everything the stack needs on this machine, read-only
    Doctor {
        /// Print the checks as JSON
        #[arg(long)]
        json: bool,
    },
    /// Build a backend from its sources, and manage the installed builds
    Build(BuildCommand),
    /// Install the gateway, llama-swap, and show its configuration
    Gateway {
        #[command(subcommand)]
        action: GatewayAction,
    },
    /// Start the gateway on the inventory's address, and return once it answers
    Up,
    /// Stop the gateway and its models, and return once their GPU memory is released
    Down,
    /// Show whether the gateway runs, its models and the GPU memory in use; exit 3 when it does not run
    Status {
        /// Print the same reading as JSON
        #[arg(long)]
        json: bool,
    },
    /// Print the gateway's log, which holds every model's output, or stream one model's output
    Logs {
        /// Keep printing the gateway's log as it grows, until the gateway stops
        #[arg(short, long)]
        follow: bool,
        /// Stream this model's output instead: what the gateway kept of it, then its live
        /// output, until Ctrl-C or the gateway stops
        model: Option<String>,
    },
    /// Measure a model through the gateway
    Bench {
        #[command(subcommand)]
        action: BenchAction,
    },
    /// Print a shell completion script
    Completions {
        /// Target shell
        #[arg(value_enum)]
        shell: Shell,
    },
}

/// `ai-ctl build <project>` builds; its subcommands manage the builds.
#[derive(Debug, Args)]
#[command(args_conflicts_with_subcommands = true, subcommand_negates_reqs = true)]
pub struct BuildCommand {
    #[command(subcommand)]
    pub action: Option<BuildAction>,
    /// Project to build, such as llama.cpp
    #[arg(required = true)]
    pub project: Option<String>,
    /// Git tag, branch or commit to build; `latest` is the newest release
    #[arg(long = "ref", value_name = "REF", default_value = "latest")]
    pub reference: String,
    /// Build again a build that is already installed
    #[arg(long)]
    pub force: bool,
    /// Parallel compile jobs [default: one per processor]
    #[arg(long, value_name = "N")]
    pub jobs: Option<NonZeroUsize>,
}

/// Managing the installed builds.
#[derive(Debug, Subcommand)]
pub enum BuildAction {
    /// Tell where each current build the inventory runs stands against its project's newest
    /// release; exit 4 when `build update` has something to build, 1 when a project cannot be
    /// read
    Outdated {
        /// Only this project, used by the inventory or not
        project: Option<String>,
    },
    /// Build what `build outdated` finds, each project at its newest release, one after the
    /// other; a failure stops that project only; a build off its recipe is replaced by its new
    /// build of the same release, and no other is removed
    Update {
        /// Parallel compile jobs [default: one per processor]
        #[arg(long, value_name = "N")]
        jobs: Option<NonZeroUsize>,
        /// Only this project, used by the inventory or not
        project: Option<String>,
    },
    /// List the installed builds; `*` marks the current one
    List {
        /// Only this project's builds
        project: Option<String>,
    },
    /// Make an installed build the current one, without compiling
    Use {
        /// Project, such as llama.cpp
        project: String,
        /// Build id, as `build list` shows it
        id: String,
    },
    /// Remove all but the newest builds; the current one is always kept, and
    /// nothing is removed while the running gateway serves a build to remove
    Prune {
        /// How many of the newest builds to keep, besides the current one
        #[arg(long, value_name = "N")]
        keep: usize,
        /// Only this project's builds
        project: Option<String>,
    },
}

/// Managing the gateway.
#[derive(Debug, Subcommand)]
pub enum GatewayAction {
    /// Download the llama-swap release this ai-ctl pins, verify it against the digest ai-ctl
    /// carries and make it current
    Install,
    /// Print the configuration `up` gives llama-swap, generated from the inventory
    Config,
}

/// Measuring the models.
#[derive(Debug, Subcommand)]
pub enum BenchAction {
    /// Measure a llama-server model the running gateway serves: generation by context depth, the
    /// prefill of fresh prompts, an agent's sessions and the gateway's overhead
    Run {
        /// Model ID, as the inventory names it
        model: String,
        /// How many times each measure is made
        #[arg(
            long,
            value_name = "N",
            default_value_t = REPEAT,
            value_parser = clap::value_parser!(u32).range(1..)
        )]
        repeat: u32,
        /// Run only this scenario; repeat the option for several [default: every scenario]
        #[arg(long = "scenario", value_enum, value_name = "SCENARIO")]
        scenarios: Vec<ScenarioArg>,
    },
    /// List the saved runs, oldest first
    List,
    /// Set saved runs side by side, one table per measure
    Compare {
        /// Saved runs, as `bench list` names them; the first is the reference
        #[arg(value_name = "RUN", num_args = 2.., required = true)]
        runs: Vec<String>,
    },
}

/// A scenario of `bench run`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum ScenarioArg {
    /// Generation at 8K, 32K and 64K tokens of context and at its limit
    Depth,
    /// Prefill of a fresh 40K-token prompt
    Prefill,
    /// Sessions of 10 turns, the history sent back at each
    Session,
    /// The same short request through the gateway and straight to the model's server
    Overhead,
}

impl From<ScenarioArg> for Scenario {
    fn from(scenario: ScenarioArg) -> Self {
        match scenario {
            ScenarioArg::Depth => Self::Depth,
            ScenarioArg::Prefill => Self::Prefill,
            ScenarioArg::Session => Self::Session,
            ScenarioArg::Overhead => Self::Overhead,
        }
    }
}
