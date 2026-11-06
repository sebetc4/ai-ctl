use std::fs;
use std::net::TcpListener;
use std::path::Path;
use std::process::{self, Output};

use ai_ctl::domain::bench::Scenario;
use ai_ctl::domain::gateway::{Release, Sha256};
use ai_ctl::presentation::{
    cli::{BenchAction, BuildAction, Cli, Command, GatewayAction, ScenarioArg},
    completions,
};
use clap::{CommandFactory, Parser, ValueEnum};
use clap_complete::Shell;

#[test]
fn cli_definition_is_valid() {
    Cli::command().debug_assert();
}

#[test]
fn a_command_is_required() {
    assert!(Cli::try_parse_from(["ai-ctl"]).is_err());
}

#[test]
fn completions_takes_a_shell() {
    let cli = Cli::try_parse_from(["ai-ctl", "completions", "bash"]).unwrap();
    assert!(matches!(
        cli.command,
        Command::Completions { shell: Shell::Bash }
    ));
}

#[test]
fn completions_rejects_an_unknown_shell() {
    assert!(Cli::try_parse_from(["ai-ctl", "completions", "tcsh"]).is_err());
}

#[test]
fn bash_completions_name_the_binary() {
    let mut out = Vec::new();
    completions::write(Shell::Bash, &mut out);
    let script = String::from_utf8(out).unwrap();
    assert!(script.contains("ai-ctl"));
}

#[test]
fn doctor_takes_an_optional_json_flag() {
    let cli = Cli::try_parse_from(["ai-ctl", "doctor"]).unwrap();
    assert!(matches!(cli.command, Command::Doctor { json: false }));
    let cli = Cli::try_parse_from(["ai-ctl", "doctor", "--json"]).unwrap();
    assert!(matches!(cli.command, Command::Doctor { json: true }));
}

#[test]
fn init_requires_a_models_directory() {
    assert!(Cli::try_parse_from(["ai-ctl", "init"]).is_err());
    let cli = Cli::try_parse_from(["ai-ctl", "init", "--models-dir", "/data/models"]).unwrap();
    assert!(matches!(
        cli.command,
        Command::Init { models_dir } if models_dir == Path::new("/data/models")
    ));
}

fn build_command(args: &[&str]) -> ai_ctl::presentation::cli::BuildCommand {
    let argv = ["ai-ctl", "build"].iter().chain(args).copied();
    match Cli::try_parse_from(argv).unwrap().command {
        Command::Build(command) => command,
        other => panic!("{other:?}"),
    }
}

#[test]
fn build_takes_a_project_and_builds_the_latest_release_by_default() {
    let command = build_command(&["llama.cpp"]);
    assert_eq!(command.project.as_deref(), Some("llama.cpp"));
    assert_eq!(command.reference, "latest");
    assert!(!command.force);
    assert_eq!(command.jobs, None);
    assert!(command.action.is_none());

    let command = build_command(&["llama.cpp", "--ref", "79e2e74", "--force", "--jobs", "8"]);
    assert_eq!(command.reference, "79e2e74");
    assert!(command.force);
    assert_eq!(command.jobs.map(|jobs| jobs.get()), Some(8));
}

#[test]
fn build_needs_a_project_and_a_positive_job_count() {
    assert!(Cli::try_parse_from(["ai-ctl", "build"]).is_err());
    assert!(Cli::try_parse_from(["ai-ctl", "build", "--ref", "b1"]).is_err());
    assert!(Cli::try_parse_from(["ai-ctl", "build", "llama.cpp", "--jobs", "0"]).is_err());
}

#[test]
fn build_subcommands_manage_the_builds() {
    assert!(matches!(
        build_command(&["list"]).action,
        Some(BuildAction::List { project: None })
    ));
    assert!(matches!(
        build_command(&["list", "llama.cpp"]).action,
        Some(BuildAction::List { project: Some(p) }) if p == "llama.cpp"
    ));
    assert!(matches!(
        build_command(&["use", "llama.cpp", "b11538"]).action,
        Some(BuildAction::Use { project, id }) if project == "llama.cpp" && id == "b11538"
    ));
    assert!(matches!(
        build_command(&["prune", "--keep", "2"]).action,
        Some(BuildAction::Prune {
            keep: 2,
            project: None
        })
    ));
    assert!(Cli::try_parse_from(["ai-ctl", "build", "prune"]).is_err());
    assert!(Cli::try_parse_from(["ai-ctl", "build", "use", "llama.cpp"]).is_err());
    assert!(Cli::try_parse_from(["ai-ctl", "build", "list", "--force"]).is_err());
}

/// Runs the binary with its configuration under `config_home`, and its
/// other directories under the parent of `config_home`.
fn ai_ctl(config_home: &Path, args: &[&str]) -> Output {
    process::Command::new(env!("CARGO_BIN_EXE_ai-ctl"))
        .args(args)
        .env_clear()
        .env("HOME", config_home.parent().unwrap())
        .env("XDG_CONFIG_HOME", config_home)
        .output()
        .unwrap()
}

#[test]
fn init_installs_the_inventory_once() {
    let root = tempfile::tempdir().unwrap();
    let config = root.path().join("config");
    let models = root.path().join("models");
    fs::create_dir(&models).unwrap();
    let models_arg = models.to_str().unwrap();
    let inventory = config.join("ai-ctl/inventory.toml");

    let first = ai_ctl(&config, &["init", "--models-dir", models_arg]);
    assert!(first.status.success(), "{first:?}");
    let installed = fs::read_to_string(&inventory).unwrap();
    assert!(installed.contains(&format!("models_dir = \"{models_arg}\"")));
    assert!(!installed.contains("${MODELS_DIR}"));

    fs::write(&inventory, "# edited by hand\n").unwrap();
    let second = ai_ctl(&config, &["init", "--models-dir", models_arg]);
    assert!(!second.status.success());
    let stderr = String::from_utf8_lossy(&second.stderr);
    assert!(stderr.contains("already exists"), "{stderr}");
    assert_eq!(
        fs::read_to_string(&inventory).unwrap(),
        "# edited by hand\n"
    );
}

#[test]
fn init_refuses_a_missing_models_directory() {
    let root = tempfile::tempdir().unwrap();
    let config = root.path().join("config");
    let missing = root.path().join("missing");
    let output = ai_ctl(
        &config,
        &["init", "--models-dir", missing.to_str().unwrap()],
    );
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("does not exist"), "{stderr}");
    assert!(!config.exists(), "nothing is written on failure");
}

#[test]
fn build_list_on_a_fresh_machine_says_how_to_build() {
    let root = tempfile::tempdir().unwrap();
    let config = root.path().join("config");
    let output = ai_ctl(&config, &["build", "list"]);
    assert!(output.status.success(), "{output:?}");
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        "llama.cpp: no build yet; `ai-ctl build llama.cpp` makes one\n\
         whisper.cpp: no build yet; `ai-ctl build whisper.cpp` makes one\n\
         stable-diffusion.cpp: no build yet; `ai-ctl build stable-diffusion.cpp` makes one\n\
         audio.cpp: no build yet; `ai-ctl build audio.cpp` makes one\n"
    );
}

#[test]
fn build_use_refuses_a_build_that_does_not_exist() {
    let root = tempfile::tempdir().unwrap();
    let config = root.path().join("config");
    let output = ai_ctl(&config, &["build", "use", "llama.cpp", "b1"]);
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("llama.cpp has no build b1"), "{stderr}");
    let unknown = ai_ctl(&config, &["build", "use", "nope.cpp", "b1"]);
    let stderr = String::from_utf8_lossy(&unknown.stderr);
    assert!(stderr.contains("no recipe for `nope.cpp`"), "{stderr}");
}

fn gateway_action(args: &[&str]) -> Result<GatewayAction, clap::Error> {
    let argv = ["ai-ctl", "gateway"].iter().chain(args).copied();
    Cli::try_parse_from(argv).map(|cli| match cli.command {
        Command::Gateway { action } => action,
        other => panic!("{other:?}"),
    })
}

fn install_version(args: &[&str]) -> Option<Release> {
    match gateway_action(args).unwrap() {
        GatewayAction::Install { version, .. } => version,
        other => panic!("{other:?}"),
    }
}

#[test]
fn gateway_install_takes_an_optional_digest() {
    let digest = "9f561442a43ed71043105891a10a41019efdc8c023f5a3c807bcf12fbd2a048d";
    match gateway_action(&["install", "--version", "264", "--sha256", digest]).unwrap() {
        GatewayAction::Install { sha256, .. } => {
            assert_eq!(sha256, Some(Sha256::parse(digest).unwrap()));
        }
        other => panic!("{other:?}"),
    }
    let error = gateway_action(&["install", "--sha256", "ABC"])
        .unwrap_err()
        .to_string();
    assert!(error.contains("is not a SHA-256 digest"), "{error}");
}

#[test]
fn gateway_install_takes_an_optional_release() {
    assert_eq!(install_version(&["install"]), None);
    for text in ["262", "v262"] {
        assert_eq!(
            install_version(&["install", "--version", text]),
            Some(Release::parse("262").unwrap())
        );
    }
    let error = gateway_action(&["install", "--version", "latest"])
        .unwrap_err()
        .to_string();
    assert!(error.contains("is not a llama-swap release"), "{error}");
    assert!(gateway_action(&[]).is_err());
}

#[test]
fn gateway_config_needs_an_inventory_and_a_build() {
    assert!(matches!(
        gateway_action(&["config"]).unwrap(),
        GatewayAction::Config
    ));
    let root = tempfile::tempdir().unwrap();
    let config = root.path().join("config");
    let output = ai_ctl(&config, &["gateway", "config"]);
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("ai-ctl init --models-dir <dir>"),
        "{stderr}"
    );

    let models = root.path().to_str().unwrap();
    assert!(
        ai_ctl(&config, &["init", "--models-dir", models])
            .status
            .success()
    );
    let output = ai_ctl(&config, &["gateway", "config"]);
    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("llama.cpp has no build; run `ai-ctl build llama.cpp`"),
        "{stderr}"
    );
}

#[test]
fn logs_takes_a_follow_flag_and_a_model() {
    let parse = |args: &[&str]| {
        let argv = ["ai-ctl", "logs"].iter().chain(args).copied();
        match Cli::try_parse_from(argv).unwrap().command {
            Command::Logs { follow, model } => (follow, model),
            other => panic!("{other:?}"),
        }
    };
    assert_eq!(parse(&[]), (false, None));
    assert_eq!(parse(&["-f"]), (true, None));
    assert_eq!(
        parse(&["--follow", "qwen3.8-27b"]),
        (true, Some("qwen3.8-27b".to_owned()))
    );
}

#[test]
fn doctor_exits_1_when_a_check_fails() {
    // No inventory and no llama-swap: whatever the machine, checks fail.
    let root = tempfile::tempdir().unwrap();
    let output = ai_ctl(&root.path().join("config"), &["doctor", "--json"]);
    assert_eq!(output.status.code(), Some(1));
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report["healthy"], false);
}

#[test]
fn status_without_a_gateway_exits_3() {
    let root = tempfile::tempdir().unwrap();
    let output = ai_ctl(&root.path().join("config"), &["status"]);
    assert_eq!(output.status.code(), Some(3));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.starts_with("Gateway     stopped\n"), "{stdout}");
}

/// A process the test started, killed and waited for when the test ends,
/// by a panic too.
struct Running(process::Child);

impl Drop for Running {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

#[test]
fn status_with_a_running_gateway_exits_0() {
    let root = tempfile::tempdir().unwrap();
    let config = root.path().join("config");
    // The recorded llama-swap: a link to `sleep`, which this test runs; a
    // copy could not be run while still open for writing.
    let binary = root.path().join("llama-swap");
    std::os::unix::fs::symlink("/usr/bin/sleep", &binary).unwrap();
    let gateway = Running(process::Command::new(&binary).arg("60").spawn().unwrap());
    let pid = gateway.0.id();
    let state = root.path().join(".local/state/ai-ctl/gateway");
    fs::create_dir_all(&state).unwrap();
    // Nothing listens there: `status` only warns that it cannot list the
    // models.
    let listen = TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap();
    fs::write(
        state.join("gateway.json"),
        format!(
            "{{\"pid\": {pid}, \"binary\": \"{}\", \"listen\": \"{listen}\"}}",
            binary.display()
        ),
    )
    .unwrap();
    let output = ai_ctl(&config, &["status"]);
    drop(gateway);
    assert_eq!(output.status.code(), Some(0), "{output:?}");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.starts_with(&format!(
            "Gateway     running on http://{listen} (PID {pid})\n"
        )),
        "{stdout}"
    );
}

#[test]
fn logs_before_any_up_is_an_error_naming_up() {
    let root = tempfile::tempdir().unwrap();
    let output = ai_ctl(&root.path().join("config"), &["logs"]);
    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("the gateway has not run yet; start it with `ai-ctl up`"),
        "{stderr}"
    );
}

#[test]
fn bench_run_takes_a_model_repetitions_and_scenarios() {
    let parse = |args: &[&str]| {
        let argv = ["ai-ctl", "bench", "run"].iter().chain(args).copied();
        Cli::try_parse_from(argv).map(|cli| match cli.command {
            Command::Bench {
                action:
                    BenchAction::Run {
                        model,
                        repeat,
                        scenarios,
                    },
            } => (model, repeat, scenarios),
            other => panic!("{other:?}"),
        })
    };
    assert_eq!(
        parse(&["qwen3.8-27b"]).unwrap(),
        ("qwen3.8-27b".to_owned(), 5, vec![])
    );
    assert_eq!(
        parse(&[
            "m",
            "--repeat",
            "10",
            "--scenario",
            "session",
            "--scenario",
            "depth"
        ])
        .unwrap(),
        (
            "m".to_owned(),
            10,
            vec![ScenarioArg::Session, ScenarioArg::Depth]
        )
    );
    assert!(parse(&[]).is_err());
    assert!(parse(&["m", "--repeat", "0"]).is_err());
    assert!(parse(&["m", "--scenario", "load"]).is_err());
}

#[test]
fn each_scenario_option_runs_the_scenario_of_its_name() {
    for arg in ScenarioArg::value_variants() {
        let name = arg.to_possible_value().unwrap().get_name().to_owned();
        assert_eq!(Scenario::from(*arg).name(), name);
    }
}

#[test]
fn bench_run_without_a_gateway_names_up() {
    let root = tempfile::tempdir().unwrap();
    let output = ai_ctl(
        &root.path().join("config"),
        &["bench", "run", "qwen3.8-27b"],
    );
    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_eq!(
        stderr,
        "error: no gateway runs; start it with `ai-ctl up`\n"
    );
}

#[test]
fn bench_compare_takes_two_runs_or_more() {
    let parse = |args: &[&str]| {
        let argv = ["ai-ctl", "bench", "compare"].iter().chain(args).copied();
        Cli::try_parse_from(argv).map(|cli| match cli.command {
            Command::Bench {
                action: BenchAction::Compare { runs },
            } => runs,
            other => panic!("{other:?}"),
        })
    };
    assert_eq!(parse(&["a", "b", "c"]).unwrap(), ["a", "b", "c"]);
    assert!(parse(&["a"]).is_err());
    assert!(parse(&[]).is_err());
    assert!(matches!(
        Cli::try_parse_from(["ai-ctl", "bench", "list"])
            .unwrap()
            .command,
        Command::Bench {
            action: BenchAction::List
        }
    ));
}

#[test]
fn bench_list_and_compare_without_saved_runs() {
    let root = tempfile::tempdir().unwrap();
    let config = root.path().join("config");
    let list = ai_ctl(&config, &["bench", "list"]);
    assert!(list.status.success());
    assert_eq!(
        String::from_utf8_lossy(&list.stdout),
        "No saved run; `ai-ctl bench run <model>` makes one.\n"
    );
    let compare = ai_ctl(&config, &["bench", "compare", "a", "b"]);
    assert_eq!(compare.status.code(), Some(1));
    assert_eq!(
        String::from_utf8_lossy(&compare.stderr),
        "error: no run is saved as a; `ai-ctl bench list` lists them\n"
    );
}
