use rouwdi_cargo::{CargoBuildPlan, CompilePhase};
use rouwdi_vfs::{normalize_path, VfsError};
use serde::{Deserialize, Serialize};
use wasmi::{Caller, Config, Engine, Extern, Linker, Module, Store};

#[derive(Debug, thiserror::Error)]
pub enum CompileTimeError {
    #[error(transparent)]
    Vfs(#[from] VfsError),
    #[error("invalid cargo directive: {0}")]
    InvalidDirective(String),
    #[error("compile-time WASM module failed: {0}")]
    Wasm(String),
    #[error("compile-time WASM module is missing required export {0}")]
    MissingExport(&'static str),
    #[error("compile-time WASM stdout is not UTF-8: {0}")]
    StdoutUtf8(#[from] std::string::FromUtf8Error),
    #[error("compile-time WASM guest I/O failed: {0}")]
    GuestIo(String),
}

pub const COMPILE_TIME_ABI_VERSION: u32 = 1;
pub const COMPILE_TIME_HOST_TRIPLE: &str = "compile-time-wasm";
pub const COMPILE_TIME_EXECUTOR: &str = "wasmi-2.0.0-beta.2";
pub const COMPILE_TIME_IMPORT_MODULE: &str = "rouwdi:compile-time/abi@1";
pub const COMPILE_TIME_STDOUT_IMPORT: &str = "stdout";
pub const COMPILE_TIME_TOKEN_INPUT_LEN_IMPORT: &str = "token_input_len";
pub const COMPILE_TIME_TOKEN_INPUT_READ_IMPORT: &str = "token_input_read";
pub const COMPILE_TIME_TOKEN_OUTPUT_WRITE_IMPORT: &str = "token_output_write";
pub const COMPILE_TIME_START_EXPORT: &str = "_start";
pub const COMPILE_TIME_PROC_MACRO_EXPAND_EXPORT: &str = "_rouwdi_proc_macro_expand";
pub const COMPILE_TIME_MEMORY_EXPORT: &str = "memory";
pub const DEFAULT_COMPILE_TIME_FUEL: u64 = 10_000_000;
pub const DEFAULT_COMPILE_TIME_STDOUT_LIMIT: usize = 1024 * 1024;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompileTimePlan {
    pub abi_version: u32,
    pub executor: CompileTimeExecutorIdentity,
    pub build_scripts: Vec<BuildScriptSandboxPlan>,
    pub proc_macros: Vec<ProcMacroSandboxPlan>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompileTimeExecutorIdentity {
    pub engine: String,
    pub abi_version: u32,
    pub import_module: String,
    pub stdout_import: String,
    pub token_input_len_import: String,
    pub token_input_read_import: String,
    pub token_output_write_import: String,
    pub start_export: String,
    pub proc_macro_expand_export: String,
    pub memory_export: String,
    pub fuel_metering: bool,
    pub ambient_host_process: bool,
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
        executor: CompileTimeExecutorIdentity::default(),
        build_scripts,
        proc_macros,
    }
}

impl Default for CompileTimeExecutorIdentity {
    fn default() -> Self {
        Self {
            engine: COMPILE_TIME_EXECUTOR.to_owned(),
            abi_version: COMPILE_TIME_ABI_VERSION,
            import_module: COMPILE_TIME_IMPORT_MODULE.to_owned(),
            stdout_import: COMPILE_TIME_STDOUT_IMPORT.to_owned(),
            token_input_len_import: COMPILE_TIME_TOKEN_INPUT_LEN_IMPORT.to_owned(),
            token_input_read_import: COMPILE_TIME_TOKEN_INPUT_READ_IMPORT.to_owned(),
            token_output_write_import: COMPILE_TIME_TOKEN_OUTPUT_WRITE_IMPORT.to_owned(),
            start_export: COMPILE_TIME_START_EXPORT.to_owned(),
            proc_macro_expand_export: COMPILE_TIME_PROC_MACRO_EXPAND_EXPORT.to_owned(),
            memory_export: COMPILE_TIME_MEMORY_EXPORT.to_owned(),
            fuel_metering: true,
            ambient_host_process: false,
        }
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
pub struct BuildScriptWasmExecution {
    pub abi_version: u32,
    pub executor: CompileTimeExecutorIdentity,
    pub stdout: String,
    pub output: BuildScriptOutput,
    pub fuel_initial: u64,
    pub fuel_remaining: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProcMacroWasmExecution {
    pub abi_version: u32,
    pub executor: CompileTimeExecutorIdentity,
    pub input_tokens: String,
    pub output_tokens: String,
    pub fuel_initial: u64,
    pub fuel_remaining: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompileTimeWasmLimits {
    pub fuel: u64,
    pub max_stdout_bytes: usize,
}

impl Default for CompileTimeWasmLimits {
    fn default() -> Self {
        Self {
            fuel: DEFAULT_COMPILE_TIME_FUEL,
            max_stdout_bytes: DEFAULT_COMPILE_TIME_STDOUT_LIMIT,
        }
    }
}

#[derive(Debug)]
struct BuildScriptHostState {
    stdout: Vec<u8>,
    max_stdout_bytes: usize,
    stdout_error: Option<String>,
}

#[derive(Debug)]
struct ProcMacroHostState {
    input_tokens: Vec<u8>,
    output_tokens: Vec<u8>,
    max_output_bytes: usize,
    call_error: Option<String>,
}

pub fn execute_build_script_wasm(
    module_bytes: &[u8],
    limits: CompileTimeWasmLimits,
) -> Result<BuildScriptWasmExecution, CompileTimeError> {
    let mut config = Config::default();
    config.consume_fuel(true);
    let engine = Engine::new(&config);
    let module = Module::new(&engine, module_bytes)
        .map_err(|error| CompileTimeError::Wasm(error.to_string()))?;
    let mut store = Store::new(
        &engine,
        BuildScriptHostState {
            stdout: Vec::new(),
            max_stdout_bytes: limits.max_stdout_bytes,
            stdout_error: None,
        },
    );
    store
        .set_fuel(limits.fuel)
        .map_err(|error| CompileTimeError::Wasm(error.to_string()))?;

    let mut linker = <Linker<BuildScriptHostState>>::new(&engine);
    linker
        .func_wrap(
            COMPILE_TIME_IMPORT_MODULE,
            COMPILE_TIME_STDOUT_IMPORT,
            compile_time_stdout,
        )
        .map_err(|error| CompileTimeError::Wasm(error.to_string()))?;
    let instance = linker
        .instantiate_and_start(&mut store, &module)
        .map_err(|error| CompileTimeError::Wasm(error.to_string()))?;
    let start = instance
        .get_typed_func::<(), ()>(&store, COMPILE_TIME_START_EXPORT)
        .map_err(|_| CompileTimeError::MissingExport(COMPILE_TIME_START_EXPORT))?;
    start
        .call(&mut store, ())
        .map_err(|error| CompileTimeError::Wasm(error.to_string()))?;

    if let Some(error) = store.data().stdout_error.clone() {
        return Err(CompileTimeError::GuestIo(error));
    }

    let fuel_remaining = store
        .get_fuel()
        .map_err(|error| CompileTimeError::Wasm(error.to_string()))?;
    let stdout = String::from_utf8(store.data().stdout.clone())?;
    let output = parse_build_script_stdout(&stdout)?;

    Ok(BuildScriptWasmExecution {
        abi_version: COMPILE_TIME_ABI_VERSION,
        executor: CompileTimeExecutorIdentity::default(),
        stdout,
        output,
        fuel_initial: limits.fuel,
        fuel_remaining,
    })
}

pub fn execute_proc_macro_wasm(
    module_bytes: &[u8],
    input_tokens: &str,
    limits: CompileTimeWasmLimits,
) -> Result<ProcMacroWasmExecution, CompileTimeError> {
    if input_tokens.len() > i32::MAX as usize {
        return Err(CompileTimeError::Wasm(
            "proc-macro token input exceeds i32 ABI size".to_owned(),
        ));
    }

    let mut config = Config::default();
    config.consume_fuel(true);
    let engine = Engine::new(&config);
    let module = Module::new(&engine, module_bytes)
        .map_err(|error| CompileTimeError::Wasm(error.to_string()))?;
    let mut store = Store::new(
        &engine,
        ProcMacroHostState {
            input_tokens: input_tokens.as_bytes().to_vec(),
            output_tokens: Vec::new(),
            max_output_bytes: limits.max_stdout_bytes,
            call_error: None,
        },
    );
    store
        .set_fuel(limits.fuel)
        .map_err(|error| CompileTimeError::Wasm(error.to_string()))?;

    let mut linker = <Linker<ProcMacroHostState>>::new(&engine);
    linker
        .func_wrap(
            COMPILE_TIME_IMPORT_MODULE,
            COMPILE_TIME_TOKEN_INPUT_LEN_IMPORT,
            proc_macro_token_input_len,
        )
        .map_err(|error| CompileTimeError::Wasm(error.to_string()))?;
    linker
        .func_wrap(
            COMPILE_TIME_IMPORT_MODULE,
            COMPILE_TIME_TOKEN_INPUT_READ_IMPORT,
            proc_macro_token_input_read,
        )
        .map_err(|error| CompileTimeError::Wasm(error.to_string()))?;
    linker
        .func_wrap(
            COMPILE_TIME_IMPORT_MODULE,
            COMPILE_TIME_TOKEN_OUTPUT_WRITE_IMPORT,
            proc_macro_token_output_write,
        )
        .map_err(|error| CompileTimeError::Wasm(error.to_string()))?;

    let instance = linker
        .instantiate_and_start(&mut store, &module)
        .map_err(|error| CompileTimeError::Wasm(error.to_string()))?;
    let expand = instance
        .get_typed_func::<(), i32>(&store, COMPILE_TIME_PROC_MACRO_EXPAND_EXPORT)
        .map_err(|_| CompileTimeError::MissingExport(COMPILE_TIME_PROC_MACRO_EXPAND_EXPORT))?;
    let status = expand
        .call(&mut store, ())
        .map_err(|error| CompileTimeError::Wasm(error.to_string()))?;
    if status != 0 {
        return Err(CompileTimeError::Wasm(format!(
            "proc-macro expansion returned status {status}"
        )));
    }

    if let Some(error) = store.data().call_error.clone() {
        return Err(CompileTimeError::GuestIo(error));
    }

    let fuel_remaining = store
        .get_fuel()
        .map_err(|error| CompileTimeError::Wasm(error.to_string()))?;
    let output_tokens = String::from_utf8(store.data().output_tokens.clone())?;

    Ok(ProcMacroWasmExecution {
        abi_version: COMPILE_TIME_ABI_VERSION,
        executor: CompileTimeExecutorIdentity::default(),
        input_tokens: input_tokens.to_owned(),
        output_tokens,
        fuel_initial: limits.fuel,
        fuel_remaining,
    })
}

fn compile_time_stdout(mut caller: Caller<'_, BuildScriptHostState>, ptr: i32, len: i32) -> i32 {
    match read_guest_stdout(&caller, ptr, len) {
        Ok(bytes) => {
            let state = caller.data_mut();
            if state.stdout.len().saturating_add(bytes.len()) > state.max_stdout_bytes {
                state.stdout_error = Some(format!(
                    "stdout exceeded {} byte limit",
                    state.max_stdout_bytes
                ));
                return -1;
            }
            state.stdout.extend_from_slice(&bytes);
            0
        }
        Err(error) => {
            caller.data_mut().stdout_error = Some(error);
            -1
        }
    }
}

fn read_guest_stdout(
    caller: &Caller<'_, BuildScriptHostState>,
    ptr: i32,
    len: i32,
) -> Result<Vec<u8>, String> {
    if ptr < 0 || len < 0 {
        return Err("stdout pointer and length must be non-negative i32 values".to_owned());
    }
    let memory = caller
        .get_export(COMPILE_TIME_MEMORY_EXPORT)
        .and_then(Extern::into_memory)
        .ok_or_else(|| {
            format!("module must export {COMPILE_TIME_MEMORY_EXPORT:?} for stdout capture")
        })?;
    let mut bytes = vec![0_u8; len as usize];
    memory
        .read(caller, ptr as usize, &mut bytes)
        .map_err(|error| error.to_string())?;
    Ok(bytes)
}

fn proc_macro_token_input_len(caller: Caller<'_, ProcMacroHostState>) -> i32 {
    caller.data().input_tokens.len() as i32
}

fn proc_macro_token_input_read(
    mut caller: Caller<'_, ProcMacroHostState>,
    ptr: i32,
    len: i32,
) -> i32 {
    match proc_macro_copy_input_to_guest(&mut caller, ptr, len) {
        Ok(()) => 0,
        Err(error) => {
            caller.data_mut().call_error = Some(error);
            -1
        }
    }
}

fn proc_macro_token_output_write(
    mut caller: Caller<'_, ProcMacroHostState>,
    ptr: i32,
    len: i32,
) -> i32 {
    match read_proc_macro_guest_memory(&caller, ptr, len) {
        Ok(bytes) => {
            let state = caller.data_mut();
            if state.output_tokens.len().saturating_add(bytes.len()) > state.max_output_bytes {
                state.call_error = Some(format!(
                    "proc-macro output exceeded {} byte limit",
                    state.max_output_bytes
                ));
                return -1;
            }
            state.output_tokens.extend_from_slice(&bytes);
            0
        }
        Err(error) => {
            caller.data_mut().call_error = Some(error);
            -1
        }
    }
}

fn proc_macro_copy_input_to_guest(
    caller: &mut Caller<'_, ProcMacroHostState>,
    ptr: i32,
    len: i32,
) -> Result<(), String> {
    if ptr < 0 || len < 0 {
        return Err("token input pointer and length must be non-negative i32 values".to_owned());
    }
    let requested = len as usize;
    let input = caller.data().input_tokens.clone();
    if requested > input.len() {
        return Err(format!(
            "token input read requested {requested} byte(s), but only {} are available",
            input.len()
        ));
    }
    let memory = caller
        .get_export(COMPILE_TIME_MEMORY_EXPORT)
        .and_then(Extern::into_memory)
        .ok_or_else(|| {
            format!("module must export {COMPILE_TIME_MEMORY_EXPORT:?} for token input")
        })?;
    memory
        .write(caller, ptr as usize, &input[..requested])
        .map_err(|error| error.to_string())
}

fn read_proc_macro_guest_memory(
    caller: &Caller<'_, ProcMacroHostState>,
    ptr: i32,
    len: i32,
) -> Result<Vec<u8>, String> {
    if ptr < 0 || len < 0 {
        return Err("token output pointer and length must be non-negative i32 values".to_owned());
    }
    let memory = caller
        .get_export(COMPILE_TIME_MEMORY_EXPORT)
        .and_then(Extern::into_memory)
        .ok_or_else(|| {
            format!("module must export {COMPILE_TIME_MEMORY_EXPORT:?} for token output")
        })?;
    let mut bytes = vec![0_u8; len as usize];
    memory
        .read(caller, ptr as usize, &mut bytes)
        .map_err(|error| error.to_string())?;
    Ok(bytes)
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
        assert_eq!(plan.executor, CompileTimeExecutorIdentity::default());
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

    #[test]
    fn executes_build_script_wasm_and_parses_stdout_inside_sandbox() {
        let stdout = b"cargo:rustc-env=GENERATED=value\ncargo:warning=careful\n";
        let wasm = wat::parse_str(format!(
            r#"
(module
  (import "{COMPILE_TIME_IMPORT_MODULE}" "{COMPILE_TIME_STDOUT_IMPORT}"
    (func $stdout (param i32 i32) (result i32)))
  (memory (export "{COMPILE_TIME_MEMORY_EXPORT}") 1)
  (data (i32.const 16) "cargo:rustc-env=GENERATED=value\0acargo:warning=careful\0a")
  (func (export "{COMPILE_TIME_START_EXPORT}")
    (drop (call $stdout (i32.const 16) (i32.const {})))))
"#,
            stdout.len()
        ))
        .unwrap();

        let execution = execute_build_script_wasm(
            &wasm,
            CompileTimeWasmLimits {
                fuel: 100_000,
                max_stdout_bytes: 1024,
            },
        )
        .unwrap();

        assert_eq!(execution.executor, CompileTimeExecutorIdentity::default());
        assert_eq!(execution.stdout.as_bytes(), stdout);
        assert!(execution.fuel_remaining < execution.fuel_initial);
        assert_eq!(
            execution.output.directives[0],
            CargoDirective::RustcEnv {
                key: "GENERATED".to_owned(),
                value: "value".to_owned()
            }
        );
        assert_eq!(
            execution.output.diagnostics[0].level,
            DiagnosticLevel::Warning
        );
    }

    #[test]
    fn rejects_compile_time_wasm_without_memory_export() {
        let wasm = wat::parse_str(format!(
            r#"
(module
  (import "{COMPILE_TIME_IMPORT_MODULE}" "{COMPILE_TIME_STDOUT_IMPORT}"
    (func $stdout (param i32 i32) (result i32)))
  (func (export "{COMPILE_TIME_START_EXPORT}")
    (drop (call $stdout (i32.const 0) (i32.const 1)))))
"#,
        ))
        .unwrap();

        let err = execute_build_script_wasm(
            &wasm,
            CompileTimeWasmLimits {
                fuel: 100_000,
                max_stdout_bytes: 1024,
            },
        )
        .unwrap_err();

        assert!(err.to_string().contains("must export \"memory\""));
    }

    #[test]
    fn fuel_limits_compile_time_wasm_execution() {
        let wasm = wat::parse_str(format!(
            r#"
(module
  (func (export "{COMPILE_TIME_START_EXPORT}")
    (loop $again
      (br $again))))
"#,
        ))
        .unwrap();

        let err = execute_build_script_wasm(
            &wasm,
            CompileTimeWasmLimits {
                fuel: 1_000,
                max_stdout_bytes: 1024,
            },
        )
        .unwrap_err();

        assert!(err.to_string().contains("fuel"));
    }

    #[test]
    fn executes_proc_macro_wasm_token_abi_inside_sandbox() {
        let input = "TokenStream";
        let wasm = wat::parse_str(format!(
            r#"
(module
  (import "{COMPILE_TIME_IMPORT_MODULE}" "{COMPILE_TIME_TOKEN_INPUT_LEN_IMPORT}"
    (func $input_len (result i32)))
  (import "{COMPILE_TIME_IMPORT_MODULE}" "{COMPILE_TIME_TOKEN_INPUT_READ_IMPORT}"
    (func $input_read (param i32 i32) (result i32)))
  (import "{COMPILE_TIME_IMPORT_MODULE}" "{COMPILE_TIME_TOKEN_OUTPUT_WRITE_IMPORT}"
    (func $output_write (param i32 i32) (result i32)))
  (memory (export "{COMPILE_TIME_MEMORY_EXPORT}") 1)
  (data (i32.const 64) "expanded:")
  (func (export "{COMPILE_TIME_PROC_MACRO_EXPAND_EXPORT}") (result i32)
    (local $len i32)
    (local.set $len (call $input_len))
    (drop (call $input_read (i32.const 16) (local.get $len)))
    (drop (call $output_write (i32.const 64) (i32.const 9)))
    (drop (call $output_write (i32.const 16) (local.get $len)))
    (i32.const 0)))
"#,
        ))
        .unwrap();

        let execution = execute_proc_macro_wasm(
            &wasm,
            input,
            CompileTimeWasmLimits {
                fuel: 100_000,
                max_stdout_bytes: 1024,
            },
        )
        .unwrap();

        assert_eq!(execution.executor, CompileTimeExecutorIdentity::default());
        assert_eq!(execution.input_tokens, input);
        assert_eq!(execution.output_tokens, "expanded:TokenStream");
        assert!(execution.fuel_remaining < execution.fuel_initial);
    }

    #[test]
    fn proc_macro_execution_rejects_output_over_limit() {
        let wasm = wat::parse_str(format!(
            r#"
(module
  (import "{COMPILE_TIME_IMPORT_MODULE}" "{COMPILE_TIME_TOKEN_INPUT_LEN_IMPORT}"
    (func $input_len (result i32)))
  (import "{COMPILE_TIME_IMPORT_MODULE}" "{COMPILE_TIME_TOKEN_INPUT_READ_IMPORT}"
    (func $input_read (param i32 i32) (result i32)))
  (import "{COMPILE_TIME_IMPORT_MODULE}" "{COMPILE_TIME_TOKEN_OUTPUT_WRITE_IMPORT}"
    (func $output_write (param i32 i32) (result i32)))
  (memory (export "{COMPILE_TIME_MEMORY_EXPORT}") 1)
  (data (i32.const 64) "expanded:")
  (func (export "{COMPILE_TIME_PROC_MACRO_EXPAND_EXPORT}") (result i32)
    (drop (call $input_len))
    (drop (call $input_read (i32.const 16) (i32.const 0)))
    (drop (call $output_write (i32.const 64) (i32.const 9)))
    (i32.const 0)))
"#,
        ))
        .unwrap();

        let err = execute_proc_macro_wasm(
            &wasm,
            "",
            CompileTimeWasmLimits {
                fuel: 100_000,
                max_stdout_bytes: 4,
            },
        )
        .unwrap_err();

        assert!(err.to_string().contains("output exceeded 4 byte limit"));
    }
}
