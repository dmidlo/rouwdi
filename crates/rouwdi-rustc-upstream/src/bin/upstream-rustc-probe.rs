use rouwdi_rustc_upstream::{
    classify_probe_output, probe_command_for, UpstreamProbeClassification, UpstreamProbeCommand,
    UpstreamProbeMode,
};
use serde::Serialize;
use std::env;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

#[derive(Debug, Clone, Serialize)]
struct ProbeReport {
    component: String,
    mode: String,
    command: String,
    workdir: String,
    exit_code: i32,
    outcome: String,
    classification: String,
    evidence: String,
    note: String,
}

#[derive(Debug)]
struct ProbeArgs {
    modes: Vec<UpstreamProbeMode>,
    json: bool,
    require_compiled: bool,
    components: Vec<String>,
}

fn main() -> ExitCode {
    let args = match ProbeArgs::parse(env::args().skip(1)) {
        Ok(args) => args,
        Err(message) => {
            eprintln!("{message}");
            eprintln!("{}", usage());
            return ExitCode::from(2);
        }
    };

    let repo_root = workspace_root();
    let mut reports = Vec::new();
    let mut spawn_failed = false;

    for component in &args.components {
        for mode in &args.modes {
            let command = probe_command_for(component, *mode);
            match run_probe(&repo_root, &command) {
                Ok(report) => reports.push(report),
                Err(message) => {
                    spawn_failed = true;
                    reports.push(ProbeReport {
                        component: command.component.clone(),
                        mode: command.mode.clone(),
                        command: render_command(&command),
                        workdir: command.workdir.clone(),
                        exit_code: -1,
                        outcome: "failed".to_owned(),
                        classification: "probe_spawn_failed".to_owned(),
                        evidence: message,
                        note: command.note.clone(),
                    });
                }
            }
        }
    }

    if args.json {
        println!(
            "{}",
            serde_json::to_string_pretty(&reports).expect("probe reports serialize to JSON")
        );
    } else {
        for report in &reports {
            println!(
                "{} [{}] exit={} {}: {}",
                report.component,
                report.mode,
                report.exit_code,
                report.classification,
                report.evidence
            );
            println!("  command: {}", report.command);
            println!("  workdir: {}", report.workdir);
            println!("  note: {}", report.note);
        }
    }

    if spawn_failed {
        return ExitCode::from(1);
    }

    if args.require_compiled
        && reports
            .iter()
            .any(|report| report.classification != "compiled" || report.exit_code != 0)
    {
        return ExitCode::from(1);
    }

    ExitCode::SUCCESS
}

impl ProbeArgs {
    fn parse(raw_args: impl IntoIterator<Item = String>) -> Result<Self, String> {
        let mut modes = Vec::new();
        let mut json = false;
        let mut require_compiled = false;
        let mut components = Vec::new();
        let mut iter = raw_args.into_iter();

        while let Some(arg) = iter.next() {
            match arg.as_str() {
                "--help" | "-h" => return Err("upstream rustc probe harness".to_owned()),
                "--json" => json = true,
                "--require-compiled" => require_compiled = true,
                "--mode" => {
                    let mode = iter
                        .next()
                        .ok_or_else(|| "--mode requires a value".to_owned())?;
                    if mode == "all" {
                        modes = UpstreamProbeMode::all();
                    } else {
                        modes.push(parse_mode(&mode)?);
                    }
                }
                value if value.starts_with("--mode=") => {
                    let mode = value.trim_start_matches("--mode=");
                    if mode == "all" {
                        modes = UpstreamProbeMode::all();
                    } else {
                        modes.push(parse_mode(mode)?);
                    }
                }
                value if value.starts_with('-') => {
                    return Err(format!("unsupported argument: {value}"));
                }
                component => components.push(component.to_owned()),
            }
        }

        if modes.is_empty() {
            modes.push(UpstreamProbeMode::XpyStage1);
        }
        if components.is_empty() {
            components.push("rustc_index".to_owned());
        }

        Ok(Self {
            modes,
            json,
            require_compiled,
            components,
        })
    }
}

fn parse_mode(mode: &str) -> Result<UpstreamProbeMode, String> {
    match mode {
        "raw" | "raw-cargo" => Ok(UpstreamProbeMode::RawCargo),
        "cfg" | "cargo-bootstrap-cfg" => Ok(UpstreamProbeMode::CargoWithBootstrapCfg),
        "no-default-features" | "cargo-no-default-features" => {
            Ok(UpstreamProbeMode::CargoNoDefaultFeatures)
        }
        "inject-new-range-api" | "cargo-injected-new-range-api" => {
            Ok(UpstreamProbeMode::CargoInjectedNewRangeApi)
        }
        "xpy-stage0" | "stage0" => Ok(UpstreamProbeMode::XpyStage0),
        "xpy-stage1" | "stage1" => Ok(UpstreamProbeMode::XpyStage1),
        other => Err(format!("unsupported probe mode: {other}")),
    }
}

fn run_probe(repo_root: &Path, probe: &UpstreamProbeCommand) -> Result<ProbeReport, String> {
    let output = Command::new(&probe.program)
        .args(&probe.args)
        .envs(probe.env.iter().map(|(key, value)| (key, value)))
        .current_dir(repo_root.join(&probe.workdir))
        .output()
        .map_err(|error| format!("failed to run probe command: {error}"))?;

    let exit_code = output.status.code().unwrap_or(-1);
    let combined_output = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let UpstreamProbeClassification {
        outcome,
        classification,
        evidence,
    } = classify_probe_output(exit_code, &combined_output);

    Ok(ProbeReport {
        component: probe.component.clone(),
        mode: probe.mode.clone(),
        command: render_command(probe),
        workdir: repo_root.join(&probe.workdir).display().to_string(),
        exit_code,
        outcome,
        classification,
        evidence,
        note: probe.note.clone(),
    })
}

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("adapter crate lives under workspace/crates/rouwdi-rustc-upstream")
        .to_path_buf()
}

fn render_command(probe: &UpstreamProbeCommand) -> String {
    let env_prefix = probe
        .env
        .iter()
        .map(|(key, value)| format!("{key}={value}"))
        .collect::<Vec<_>>()
        .join(" ");
    let command = std::iter::once(probe.program.as_str())
        .chain(probe.args.iter().map(String::as_str))
        .collect::<Vec<_>>()
        .join(" ");

    if env_prefix.is_empty() {
        command
    } else {
        format!("{env_prefix} {command}")
    }
}

fn usage() -> &'static str {
    "usage: upstream-rustc-probe [--mode <raw|cfg|no-default-features|inject-new-range-api|xpy-stage0|xpy-stage1|all>] [--json] [--require-compiled] [component ...]"
}
