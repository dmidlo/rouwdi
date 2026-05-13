use rouwdi_cargo::{CargoBuildPlan, CompilePhase};
use rouwdi_vfs::{normalize_path, VfsError};
use serde::{Deserialize, Serialize};

#[derive(Debug, thiserror::Error)]
pub enum CompileTimeError {
    #[error(transparent)]
    Vfs(#[from] VfsError),
    #[error("invalid cargo directive: {0}")]
    InvalidDirective(String),
}

pub const COMPILE_TIME_ABI_VERSION: u32 = 1;
pub const COMPILE_TIME_HOST_TRIPLE: &str = "compile-time-wasm";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompileTimePlan {
    pub abi_version: u32,
    pub build_scripts: Vec<BuildScriptSandboxPlan>,
    pub proc_macros: Vec<ProcMacroSandboxPlan>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BuildScriptSandboxPlan {
    pub unit_id: String,
    pub package: String,
    pub source_path: String,
    pub host: String,
    pub target: String,
    pub profile: String,
    pub out_dir: String,
    pub env: Vec<EnvBinding>,
    pub grants: SandboxGrants,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProcMacroSandboxPlan {
    pub unit_id: String,
    pub package: String,
    pub target: String,
    pub abi: String,
    pub host: String,
    pub grants: SandboxGrants,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EnvBinding {
    pub key: String,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SandboxGrants {
    pub filesystem_root: String,
    pub out_dir_write: bool,
    pub network: bool,
    pub ambient_host_process: bool,
}

pub fn plan_compile_time(build_plan: &CargoBuildPlan) -> CompileTimePlan {
    let mut build_scripts = Vec::new();
    let mut proc_macros = Vec::new();

    for unit in &build_plan.units {
        match unit.phase {
            CompilePhase::BuildScript => {
                let out_dir = format!(
                    ".rouwdi/compile-time/{}/out",
                    unit.package.replace('-', "_")
                );
                build_scripts.push(BuildScriptSandboxPlan {
                    unit_id: unit.id.clone(),
                    package: unit.package.clone(),
                    source_path: unit
                        .source_path
                        .clone()
                        .unwrap_or_else(|| unit.target.clone()),
                    host: COMPILE_TIME_HOST_TRIPLE.to_owned(),
                    target: unit.triple.clone(),
                    profile: unit.profile.clone(),
                    out_dir: out_dir.clone(),
                    env: cargo_build_script_env(unit, &out_dir),
                    grants: SandboxGrants::compile_time(&out_dir),
                });
            }
            CompilePhase::ProcMacro => {
                proc_macros.push(ProcMacroSandboxPlan {
                    unit_id: unit.id.clone(),
                    package: unit.package.clone(),
                    target: unit.target.clone(),
                    abi: format!("rouwdi-proc-macro-abi-v{COMPILE_TIME_ABI_VERSION}"),
                    host: COMPILE_TIME_HOST_TRIPLE.to_owned(),
                    grants: SandboxGrants::compile_time(".rouwdi/proc-macro"),
                });
            }
            CompilePhase::Rust | CompilePhase::Link => {}
        }
    }

    build_scripts.sort_by(|left, right| left.unit_id.cmp(&right.unit_id));
    proc_macros.sort_by(|left, right| left.unit_id.cmp(&right.unit_id));
    CompileTimePlan {
        abi_version: COMPILE_TIME_ABI_VERSION,
        build_scripts,
        proc_macros,
    }
}

impl SandboxGrants {
    fn compile_time(out_dir: &str) -> Self {
        Self {
            filesystem_root: out_dir.to_owned(),
            out_dir_write: true,
            network: false,
            ambient_host_process: false,
        }
    }
}

fn cargo_build_script_env(unit: &rouwdi_cargo::CompileUnit, out_dir: &str) -> Vec<EnvBinding> {
    let mut env = vec![
        EnvBinding {
            key: "OUT_DIR".to_owned(),
            value: out_dir.to_owned(),
        },
        EnvBinding {
            key: "CARGO_MANIFEST_DIR".to_owned(),
            value: manifest_dir(&unit.manifest_path),
        },
        EnvBinding {
            key: "HOST".to_owned(),
            value: COMPILE_TIME_HOST_TRIPLE.to_owned(),
        },
        EnvBinding {
            key: "TARGET".to_owned(),
            value: unit.triple.clone(),
        },
        EnvBinding {
            key: "PROFILE".to_owned(),
            value: unit.profile.clone(),
        },
    ];
    env.sort_by(|left, right| left.key.cmp(&right.key));
    env
}

fn manifest_dir(manifest_path: &str) -> String {
    manifest_path
        .rsplit_once('/')
        .map(|(dir, _)| dir.to_owned())
        .unwrap_or_else(|| ".".to_owned())
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BuildScriptOutput {
    pub directives: Vec<CargoDirective>,
    pub diagnostics: Vec<CompileTimeDiagnostic>,
    pub ignored_lines: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum CargoDirective {
    RerunIfChanged { path: String },
    RerunIfEnvChanged { variable: String },
    RustcLinkLib { value: String },
    RustcLinkSearch { value: String },
    RustcCfg { cfg: String },
    RustcEnv { key: String, value: String },
    RustcFlags { value: String },
    RustcCheckCfg { value: String },
    Metadata { key: String, value: String },
    Warning { message: String },
    Error { message: String },
    Unknown { key: String, value: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompileTimeDiagnostic {
    pub level: DiagnosticLevel,
    pub message: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DiagnosticLevel {
    Warning,
    Error,
}

pub fn parse_build_script_stdout(stdout: &str) -> Result<BuildScriptOutput, CompileTimeError> {
    let mut directives = Vec::new();
    let mut diagnostics = Vec::new();
    let mut ignored_lines = Vec::new();

    for raw_line in stdout.lines() {
        let line = raw_line.trim_end_matches('\r');
        let Some(payload) = cargo_payload(line) else {
            if !line.trim().is_empty() {
                ignored_lines.push(line.to_owned());
            }
            continue;
        };
        let (key, value) = payload.split_once('=').ok_or_else(|| {
            CompileTimeError::InvalidDirective(format!("missing '=' in {line:?}"))
        })?;
        let directive = parse_directive(key.trim(), value.trim())?;
        match &directive {
            CargoDirective::Warning { message } => diagnostics.push(CompileTimeDiagnostic {
                level: DiagnosticLevel::Warning,
                message: message.clone(),
            }),
            CargoDirective::Error { message } => diagnostics.push(CompileTimeDiagnostic {
                level: DiagnosticLevel::Error,
                message: message.clone(),
            }),
            _ => {}
        }
        directives.push(directive);
    }

    Ok(BuildScriptOutput {
        directives,
        diagnostics,
        ignored_lines,
    })
}

fn cargo_payload(line: &str) -> Option<&str> {
    line.strip_prefix("cargo::")
        .or_else(|| line.strip_prefix("cargo:"))
}

fn parse_directive(key: &str, value: &str) -> Result<CargoDirective, CompileTimeError> {
    Ok(match key {
        "rerun-if-changed" => CargoDirective::RerunIfChanged {
            path: normalize_path(value)?,
        },
        "rerun-if-env-changed" => CargoDirective::RerunIfEnvChanged {
            variable: required_value(key, value)?.to_owned(),
        },
        "rustc-link-lib" => CargoDirective::RustcLinkLib {
            value: required_value(key, value)?.to_owned(),
        },
        "rustc-link-search" => CargoDirective::RustcLinkSearch {
            value: required_value(key, value)?.to_owned(),
        },
        "rustc-cfg" => CargoDirective::RustcCfg {
            cfg: required_value(key, value)?.to_owned(),
        },
        "rustc-env" => {
            let (env_key, env_value) = value.split_once('=').ok_or_else(|| {
                CompileTimeError::InvalidDirective(
                    "rustc-env must be formatted as KEY=VALUE".to_owned(),
                )
            })?;
            CargoDirective::RustcEnv {
                key: required_value("rustc-env key", env_key)?.to_owned(),
                value: env_value.to_owned(),
            }
        }
        "rustc-flags" => CargoDirective::RustcFlags {
            value: required_value(key, value)?.to_owned(),
        },
        "rustc-check-cfg" => CargoDirective::RustcCheckCfg {
            value: required_value(key, value)?.to_owned(),
        },
        "warning" => CargoDirective::Warning {
            message: required_value(key, value)?.to_owned(),
        },
        "error" => CargoDirective::Error {
            message: required_value(key, value)?.to_owned(),
        },
        other if other.starts_with("metadata.") => CargoDirective::Metadata {
            key: other.strip_prefix("metadata.").unwrap_or(other).to_owned(),
            value: value.to_owned(),
        },
        other => CargoDirective::Unknown {
            key: other.to_owned(),
            value: value.to_owned(),
        },
    })
}

fn required_value<'a>(key: &str, value: &'a str) -> Result<&'a str, CompileTimeError> {
    if value.is_empty() {
        Err(CompileTimeError::InvalidDirective(format!(
            "{key} value must not be empty"
        )))
    } else {
        Ok(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rouwdi_cargo::{CargoBuildPlan, CargoTargetKind, CompileUnit, CompileUnitEdge};

    #[test]
    fn plans_compile_time_sandboxes_without_host_process_grants() {
        let plan = plan_compile_time(&CargoBuildPlan {
            units: vec![
                CompileUnit {
                    id: "app:build-script:wasm32-wasip1".to_owned(),
                    package: "app".to_owned(),
                    manifest_path: "crates/app/Cargo.toml".to_owned(),
                    target: "crates/app/build.rs".to_owned(),
                    source_path: Some("crates/app/build.rs".to_owned()),
                    target_kind: CargoTargetKind::Bin,
                    phase: CompilePhase::BuildScript,
                    triple: "wasm32-wasip1".to_owned(),
                    profile: "release".to_owned(),
                },
                CompileUnit {
                    id: "derive:proc-macro:derive".to_owned(),
                    package: "derive".to_owned(),
                    manifest_path: "derive/Cargo.toml".to_owned(),
                    target: "derive".to_owned(),
                    source_path: Some("derive/src/lib.rs".to_owned()),
                    target_kind: CargoTargetKind::Lib,
                    phase: CompilePhase::ProcMacro,
                    triple: COMPILE_TIME_HOST_TRIPLE.to_owned(),
                    profile: "release".to_owned(),
                },
            ],
            edges: Vec::<CompileUnitEdge>::new(),
        });

        assert_eq!(plan.abi_version, COMPILE_TIME_ABI_VERSION);
        assert_eq!(plan.build_scripts[0].target, "wasm32-wasip1");
        assert_eq!(
            plan.build_scripts[0]
                .env
                .iter()
                .find(|env| env.key == "CARGO_MANIFEST_DIR")
                .unwrap()
                .value,
            "crates/app"
        );
        assert!(!plan.build_scripts[0].grants.ambient_host_process);
        assert!(!plan.proc_macros[0].grants.network);
    }

    #[test]
    fn parses_cargo_directives_from_build_script_stdout() {
        let output = parse_build_script_stdout(
            "\
cargo:rerun-if-changed=build.rs
cargo::rerun-if-env-changed=CC
cargo:rustc-link-lib=static=z
cargo:rustc-cfg=has_thing
cargo:rustc-env=GENERATED=value
cargo:warning=careful
ordinary stdout
",
        )
        .unwrap();

        assert_eq!(output.directives.len(), 6);
        assert_eq!(
            output.directives[0],
            CargoDirective::RerunIfChanged {
                path: "build.rs".to_owned()
            }
        );
        assert_eq!(
            output.directives[4],
            CargoDirective::RustcEnv {
                key: "GENERATED".to_owned(),
                value: "value".to_owned()
            }
        );
        assert_eq!(output.diagnostics[0].level, DiagnosticLevel::Warning);
        assert_eq!(output.ignored_lines, vec!["ordinary stdout".to_owned()]);
    }

    #[test]
    fn rejects_build_script_paths_that_escape_the_sandbox() {
        let err = parse_build_script_stdout("cargo:rerun-if-changed=../host-file\n").unwrap_err();

        assert!(err.to_string().contains("path escapes virtual root"));
    }

    #[test]
    fn rejects_malformed_env_directives() {
        let err = parse_build_script_stdout("cargo:rustc-env=NO_VALUE\n").unwrap_err();

        assert!(err.to_string().contains("KEY=VALUE"));
    }
}
