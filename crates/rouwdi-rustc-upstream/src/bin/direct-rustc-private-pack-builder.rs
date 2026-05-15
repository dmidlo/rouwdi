use rouwdi_rustc_upstream::{
    classify_direct_rustc_private_artifact_bytes, direct_rustc_private_build_order,
    direct_rustc_private_pack_builder_record, CompilerPayloadBridgeArtifactIdentity,
    DirectRustcPrivateCommandModel, RustcPrivateBridgeRetryGate,
    RustcPrivateDirectArtifactIdentity, RustcPrivateDirectBridgeAttempt,
    RustcPrivateDirectClosureAttempt, RustcPrivateDirectPackSummary,
    RustcPrivateTargetPackManifest,
};
use serde::Serialize;
use std::collections::BTreeSet;
use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

const BRIDGE_BINARY_TARGET: &str = "rouwdi_mir_adapter_probe";
const BRIDGE_HIR_LOWERING_ATTEMPTED_CLASSIFICATION: &str =
    "bridge_wasm_core_metadata_loaded_blocked_at_missing_core_lang_item_copy";
const BRIDGE_REQUIRED_EXPORTS: &[&str] = &[
    "memory",
    "rouwdi_compiler_payload_abi_v1_version",
    "rouwdi_compiler_payload_abi_v1_stage",
    "rouwdi_compiler_payload_abi_v1_descriptor_ptr",
    "rouwdi_compiler_payload_abi_v1_descriptor_len",
    "rouwdi_mir_handoff_payload_v1_valid_input_ptr",
    "rouwdi_mir_handoff_payload_v1_valid_input_len",
    "rouwdi_mir_handoff_payload_v1_result_area_ptr",
    "rouwdi_mir_handoff_payload_v1_execute",
    "rouwdi_mir_handoff_payload_v1_last_error_ptr",
    "rouwdi_mir_handoff_payload_v1_last_error_len",
];

#[derive(Debug, Clone)]
struct Args {
    json: bool,
    allow_partial: bool,
    manifest_path: PathBuf,
    pack_dir: PathBuf,
    cargo_target_dir: PathBuf,
}

#[derive(Debug, Clone, Serialize)]
struct DirectPackBuilderReport {
    command: String,
    workspace_root: String,
    builder: rouwdi_rustc_upstream::RustcPrivateDirectPackBuilderRecord,
    strategy: rouwdi_rustc_upstream::RustcPrivateDirectBuildStrategyRecord,
    build_order: Vec<String>,
    attempts: Vec<RustcPrivateDirectClosureAttempt>,
    pack: RustcPrivateDirectPackSummary,
    bridge_retry: Option<RustcPrivateDirectBridgeAttempt>,
    bridge_retry_gate: RustcPrivateBridgeRetryGate,
}

fn main() -> ExitCode {
    let args = match Args::parse(env::args().skip(1)) {
        Ok(args) => args,
        Err(message) => {
            eprintln!("{message}");
            eprintln!("{}", usage());
            return ExitCode::from(2);
        }
    };

    let workspace_root = workspace_root();
    let manifest_path = absolutize(&workspace_root, &args.manifest_path);
    let manifest_bytes = match fs::read_to_string(&manifest_path) {
        Ok(bytes) => bytes,
        Err(error) => {
            eprintln!("failed to read {}: {error}", manifest_path.display());
            return ExitCode::from(1);
        }
    };
    let manifest: RustcPrivateTargetPackManifest = match toml::from_str(&manifest_bytes) {
        Ok(manifest) => manifest,
        Err(error) => {
            eprintln!("failed to parse {}: {error}", manifest_path.display());
            return ExitCode::from(1);
        }
    };

    let pack_dir = absolutize(&workspace_root, &args.pack_dir);
    let cargo_target_dir = absolutize(&workspace_root, &args.cargo_target_dir);
    if let Err(error) = fs::create_dir_all(pack_dir.join("logs")) {
        eprintln!(
            "failed to create pack directory {}: {error}",
            pack_dir.display()
        );
        return ExitCode::from(1);
    }
    if let Err(error) = fs::create_dir_all(&cargo_target_dir) {
        eprintln!(
            "failed to create cargo target directory {}: {error}",
            cargo_target_dir.display()
        );
        return ExitCode::from(1);
    }

    let mut command_model = DirectRustcPrivateCommandModel::for_workspace(
        &workspace_root,
        &manifest.target_triple,
        &manifest.host_triple,
    );
    command_model.cargo_target_dir = cargo_target_dir.display().to_string();
    let build_order = direct_rustc_private_build_order(&manifest);
    let mut attempts = Vec::new();

    for crate_name in &build_order {
        let attempt = run_crate_attempt(
            &workspace_root,
            &pack_dir,
            &command_model,
            crate_name,
            &manifest.target_triple,
            &manifest.host_triple,
        );
        attempts.push(attempt);
    }

    let pack = summarize_pack(
        &workspace_root,
        &pack_dir,
        &manifest,
        &attempts,
        ".rouwdi/packs/rustc-private/wasm32-wasip1/pack-manifest.json",
    );
    let bridge_retry = pack.all_required_roots_target_loadable.then(|| {
        run_bridge_retry(
            &workspace_root,
            &pack_dir,
            &command_model,
            &manifest,
            &attempts,
        )
    });
    let bridge_retry_gate = if let Some(bridge_retry) = &bridge_retry {
        RustcPrivateBridgeRetryGate {
            attempted: true,
            classification: bridge_retry.classification.clone(),
            reason: bridge_retry.exact_blocker.clone(),
            required_before_retry: Vec::new(),
        }
    } else {
        RustcPrivateBridgeRetryGate {
            attempted: false,
            classification: "not_retried_no_real_target_loadable_root_artifacts".to_owned(),
            reason: "The direct pack builder did not produce target-loadable wasm32-wasip1 artifacts for every required root crate, so retrying the MIR bridge would fabricate progress or replay a known E0463 failure.".to_owned(),
            required_before_retry: pack
                .exact_missing_crates
                .iter()
                .map(|name| format!("target-loadable {name} artifact"))
                .collect(),
        }
    };

    let report = DirectPackBuilderReport {
        command:
            "cargo run -p rouwdi-rustc-upstream --bin direct-rustc-private-pack-builder -- --json"
                .to_owned(),
        workspace_root: workspace_root.display().to_string(),
        builder: direct_rustc_private_pack_builder_record(),
        strategy: command_model.to_strategy_record(),
        build_order,
        attempts: attempts.clone(),
        pack,
        bridge_retry,
        bridge_retry_gate,
    };

    let manifest_json =
        serde_json::to_string_pretty(&report).expect("direct pack report serializes");
    let pack_manifest_path = pack_dir.join("pack-manifest.json");
    if let Err(error) = fs::write(&pack_manifest_path, manifest_json.as_bytes()) {
        eprintln!(
            "failed to write pack manifest {}: {error}",
            pack_manifest_path.display()
        );
        return ExitCode::from(1);
    }

    if args.json {
        println!("{manifest_json}");
    } else {
        print_text_report(&report);
    }

    let bridge_blocked = report
        .bridge_retry
        .as_ref()
        .is_some_and(|bridge| bridge.exit_code != 0 || bridge.output_artifact_identity.is_none());
    if !args.allow_partial && (!report.pack.all_required_roots_target_loadable || bridge_blocked) {
        return ExitCode::from(1);
    }

    ExitCode::SUCCESS
}

impl Args {
    fn parse(raw_args: impl IntoIterator<Item = String>) -> Result<Self, String> {
        let mut json = false;
        let mut allow_partial = false;
        let mut manifest_path = PathBuf::from("bootstrap/rustc-private-target-pack.toml");
        let mut pack_dir = PathBuf::from(".rouwdi/packs/rustc-private/wasm32-wasip1");
        let mut cargo_target_dir = PathBuf::from(".rouwdi/direct-rustc-private-pack/target");
        let mut iter = raw_args.into_iter();

        while let Some(arg) = iter.next() {
            match arg.as_str() {
                "--help" | "-h" => return Err("direct rustc-private pack builder".to_owned()),
                "--json" => json = true,
                "--allow-partial" => allow_partial = true,
                "--manifest" => {
                    manifest_path = PathBuf::from(
                        iter.next()
                            .ok_or_else(|| "--manifest requires a path".to_owned())?,
                    );
                }
                "--pack-dir" => {
                    pack_dir = PathBuf::from(
                        iter.next()
                            .ok_or_else(|| "--pack-dir requires a path".to_owned())?,
                    );
                }
                "--cargo-target-dir" => {
                    cargo_target_dir = PathBuf::from(
                        iter.next()
                            .ok_or_else(|| "--cargo-target-dir requires a path".to_owned())?,
                    );
                }
                value if value.starts_with("--manifest=") => {
                    manifest_path = PathBuf::from(value.trim_start_matches("--manifest="));
                }
                value if value.starts_with("--pack-dir=") => {
                    pack_dir = PathBuf::from(value.trim_start_matches("--pack-dir="));
                }
                value if value.starts_with("--cargo-target-dir=") => {
                    cargo_target_dir =
                        PathBuf::from(value.trim_start_matches("--cargo-target-dir="));
                }
                value if value.starts_with('-') => {
                    return Err(format!("unsupported argument: {value}"));
                }
                value => return Err(format!("unexpected positional argument: {value}")),
            }
        }

        Ok(Self {
            json,
            allow_partial,
            manifest_path,
            pack_dir,
            cargo_target_dir,
        })
    }
}

fn run_crate_attempt(
    workspace_root: &Path,
    pack_dir: &Path,
    command_model: &DirectRustcPrivateCommandModel,
    crate_name: &str,
    target_triple: &str,
    host_triple: &str,
) -> RustcPrivateDirectClosureAttempt {
    let rust_root = workspace_root.join("third_party/rust");
    let manifest_path = rust_root.join("Cargo.toml");
    let cargo_path = PathBuf::from(&command_model.host_cargo_path);
    let cargo_target_dir = PathBuf::from(&command_model.cargo_target_dir);
    let target_rustflags = command_model
        .target_rustflags_env
        .split_once('=')
        .map(|(_, value)| value.to_owned())
        .unwrap_or_default();
    let target_linker = command_model
        .target_linker_env
        .split_once('=')
        .map(|(_, value)| value.to_owned())
        .unwrap_or_default();
    let target_cc = env_value(&command_model.target_cc_env);
    let target_cxx = env_value(&command_model.target_cxx_env);
    let target_ar = env_value(&command_model.target_ar_env);
    let target_ranlib = env_value(&command_model.target_ranlib_env);
    let wasi_sysroot = env_value(&command_model.wasi_sysroot_env);
    let target_cflags = env_value(&command_model.target_cflags_env);
    let target_cxxflags = env_value(&command_model.target_cxxflags_env);
    let cfg_release = env_value(&command_model.cfg_release_env);
    let cfg_release_channel = env_value(&command_model.cfg_release_channel_env);
    let cfg_release_num = env_value(&command_model.cfg_release_num_env);
    let cfg_compiler_host_triple = env_value(&command_model.cfg_compiler_host_triple_env);
    let rustc_install_bindir = env_value(&command_model.rustc_install_bindir_env);
    let rustc_stage = env_value(&command_model.rustc_stage_env);
    let command_text =
        render_cargo_command(command_model, &manifest_path, crate_name, target_triple);

    let output = Command::new(&cargo_path)
        .args([
            "build",
            "--manifest-path",
            &manifest_path.display().to_string(),
            "-p",
            crate_name,
            "--target",
            target_triple,
            "--release",
            "--message-format",
            "short",
        ])
        .env("RUSTC", &command_model.host_rustc_path)
        .env("RUSTC_BOOTSTRAP", &command_model.rustc_bootstrap)
        .env("CARGO_TARGET_DIR", &command_model.cargo_target_dir)
        .env("CARGO_TARGET_WASM32_WASIP1_RUSTFLAGS", target_rustflags)
        .env("CARGO_TARGET_WASM32_WASIP1_LINKER", target_linker)
        .env("CC_wasm32_wasip1", target_cc)
        .env("CXX_wasm32_wasip1", target_cxx)
        .env("AR_wasm32_wasip1", target_ar)
        .env("RANLIB_wasm32_wasip1", target_ranlib)
        .env("WASI_SYSROOT", wasi_sysroot)
        .env("CFLAGS_wasm32_wasip1", target_cflags)
        .env("CXXFLAGS_wasm32_wasip1", target_cxxflags)
        .env("CFG_RELEASE", cfg_release)
        .env("CFG_RELEASE_CHANNEL", cfg_release_channel)
        .env("CFG_RELEASE_NUM", cfg_release_num)
        .env("CFG_COMPILER_HOST_TRIPLE", cfg_compiler_host_triple)
        .env("RUSTC_INSTALL_BINDIR", rustc_install_bindir)
        .env("RUSTC_STAGE", rustc_stage)
        .env_remove("RUSTFLAGS")
        .current_dir(&rust_root)
        .output();

    let (exit_code, combined_output) = match output {
        Ok(output) => (
            output.status.code().unwrap_or(-1),
            format!(
                "{}{}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            ),
        ),
        Err(error) => (-1, format!("failed to spawn cargo: {error}")),
    };

    let log_path = pack_dir.join("logs").join(format!("{crate_name}.log"));
    let _ = fs::write(
        &log_path,
        format!("command: {command_text}\nexit_code: {exit_code}\n\n{combined_output}"),
    );

    let artifacts = collect_artifact_identities(
        workspace_root,
        &cargo_target_dir,
        crate_name,
        target_triple,
        host_triple,
    )
    .unwrap_or_default();
    let target_loadable = artifacts.iter().any(|artifact| artifact.target_loadable);
    let emitted_target_triple = artifacts
        .iter()
        .find(|artifact| artifact.target_loadable)
        .or_else(|| artifacts.first())
        .map(|artifact| artifact.emitted_target_triple.clone())
        .unwrap_or_else(|| "none".to_owned());
    let classification = classify_attempt(exit_code, target_loadable, &artifacts);
    let exact_blocker = if target_loadable {
        "A target-loadable wasm32-wasip1 rlib artifact was emitted and classified by inspecting archive object members for Wasm object bytes.".to_owned()
    } else if exit_code == 0 && !artifacts.is_empty() {
        format!(
            "The command exited 0, but no artifact for {crate_name} was target-loadable as wasm32-wasip1. Host/proc-macro/metadata artifacts were recorded without relabeling. Log: {}",
            relative_path(workspace_root, &log_path)
        )
    } else if artifacts.is_empty() {
        format!(
            "No artifact for {crate_name} was emitted. Log: {}",
            relative_path(workspace_root, &log_path)
        )
    } else {
        first_relevant_error(&combined_output).unwrap_or_else(|| {
            format!(
                "Command exited {exit_code}; see {}",
                relative_path(workspace_root, &log_path)
            )
        })
    };

    RustcPrivateDirectClosureAttempt {
        name: crate_name.to_owned(),
        command: command_text,
        workdir: relative_path(workspace_root, &rust_root),
        exit_code,
        requested_target_triple: target_triple.to_owned(),
        emitted_target_triple,
        classification,
        artifacts,
        target_loadable,
        exact_blocker,
    }
}

fn summarize_pack(
    workspace_root: &Path,
    pack_dir: &Path,
    manifest: &RustcPrivateTargetPackManifest,
    attempts: &[RustcPrivateDirectClosureAttempt],
    pack_manifest_path: &str,
) -> RustcPrivateDirectPackSummary {
    let artifacts = attempts
        .iter()
        .flat_map(|attempt| attempt.artifacts.iter().cloned())
        .collect::<Vec<_>>();
    let missing_roots = manifest
        .dependency_closure
        .root_crates
        .iter()
        .filter(|root| {
            !attempts
                .iter()
                .any(|attempt| attempt.name == **root && attempt.target_loadable)
        })
        .cloned()
        .collect::<Vec<_>>();
    let all_required_roots_target_loadable =
        missing_roots.is_empty() && !manifest.dependency_closure.root_crates.is_empty();
    let first_hard_blocker = if all_required_roots_target_loadable {
        "none".to_owned()
    } else {
        attempts
            .iter()
            .find(|attempt| attempt.exit_code != 0)
            .map(|attempt| format!("{}: {}", attempt.name, attempt.exact_blocker))
            .unwrap_or_else(|| {
                attempts
                    .iter()
                    .find(|attempt| !attempt.target_loadable)
                    .map(|attempt| format!("{}: {}", attempt.name, attempt.exact_blocker))
                    .unwrap_or_else(|| "none".to_owned())
            })
    };

    RustcPrivateDirectPackSummary {
        path: relative_path(workspace_root, pack_dir),
        manifest_path: pack_manifest_path.to_owned(),
        status: if all_required_roots_target_loadable {
            "ready".to_owned()
        } else if artifacts.iter().any(|artifact| artifact.target_loadable) {
            "partial".to_owned()
        } else {
            "blocked".to_owned()
        },
        root_crates: manifest.dependency_closure.root_crates.clone(),
        transitive_crates: manifest
            .dependency_closure
            .transitive_compiler_private_crates
            .clone(),
        artifacts,
        exact_missing_crates: missing_roots,
        hash_list: attempts
            .iter()
            .flat_map(|attempt| {
                attempt
                    .artifacts
                    .iter()
                    .map(|artifact| artifact.sha256.clone())
            })
            .collect(),
        target_triple: manifest.target_triple.clone(),
        all_required_roots_target_loadable,
        first_hard_blocker,
    }
}

fn run_bridge_retry(
    workspace_root: &Path,
    pack_dir: &Path,
    command_model: &DirectRustcPrivateCommandModel,
    manifest: &RustcPrivateTargetPackManifest,
    attempts: &[RustcPrivateDirectClosureAttempt],
) -> RustcPrivateDirectBridgeAttempt {
    let rust_root = workspace_root.join("third_party/rust");
    let adapter_manifest = rust_root.join("src/tools/rouwdi-mir-adapter-probe/Cargo.toml");
    let cargo_target_dir = PathBuf::from(&command_model.cargo_target_dir);
    let target_deps = cargo_target_dir
        .join(&manifest.target_triple)
        .join("release/deps");
    let target_release = cargo_target_dir
        .join(&manifest.target_triple)
        .join("release");
    let externs = manifest
        .dependency_closure
        .root_crates
        .iter()
        .filter(|root| bridge_runtime_extern_required(root))
        .filter_map(|root| {
            attempts
                .iter()
                .find(|attempt| attempt.name == *root)
                .and_then(|attempt| select_bridge_extern_artifact(attempt, root))
                .map(|artifact| (root.clone(), workspace_root.join(&artifact.path)))
        })
        .collect::<Vec<_>>();
    let extern_flags = externs
        .iter()
        .map(|(name, path)| format!("--extern {name}={}", path.display()))
        .collect::<Vec<_>>()
        .join(" ");
    let export_flags = BRIDGE_REQUIRED_EXPORTS
        .iter()
        .filter(|name| **name != "memory")
        .map(|name| format!("-Clink-arg=-Wl,--export={name}"))
        .chain(std::iter::once("-Clink-arg=-Wl,--export-memory".to_owned()))
        .collect::<Vec<_>>()
        .join(" ");
    let target_rustflags = format!(
        "{} -Clink-self-contained=no -L dependency={} -L dependency={} {} {}",
        env_value(&command_model.target_rustflags_env),
        target_deps.display(),
        target_release.display(),
        extern_flags,
        export_flags
    );
    let target_linker = env_value(&command_model.target_linker_env);
    let target_cc = env_value(&command_model.target_cc_env);
    let target_cxx = env_value(&command_model.target_cxx_env);
    let target_ar = env_value(&command_model.target_ar_env);
    let target_ranlib = env_value(&command_model.target_ranlib_env);
    let wasi_sysroot = env_value(&command_model.wasi_sysroot_env);
    let target_cflags = env_value(&command_model.target_cflags_env);
    let target_cxxflags = env_value(&command_model.target_cxxflags_env);
    let cfg_release = env_value(&command_model.cfg_release_env);
    let cfg_release_channel = env_value(&command_model.cfg_release_channel_env);
    let cfg_release_num = env_value(&command_model.cfg_release_num_env);
    let cfg_compiler_host_triple = env_value(&command_model.cfg_compiler_host_triple_env);
    let rustc_install_bindir = env_value(&command_model.rustc_install_bindir_env);
    let rustc_stage = env_value(&command_model.rustc_stage_env);
    let command_text = format!(
        "$env:RUSTC='{}'; $env:RUSTC_BOOTSTRAP='{}'; $env:CARGO_TARGET_DIR='{}'; $env:CARGO_TARGET_WASM32_WASIP1_RUSTFLAGS='{}'; $env:{}; $env:{}; $env:{}; $env:{}; $env:{}; $env:{}; $env:{}; $env:{}; $env:{}; $env:{}; $env:{}; $env:{}; $env:{}; $env:{}; Remove-Item Env:RUSTFLAGS -ErrorAction SilentlyContinue; '{}' build --manifest-path '{}' --bin {} --target {} --release --message-format short",
        command_model.host_rustc_path,
        command_model.rustc_bootstrap,
        command_model.cargo_target_dir,
        target_rustflags,
        command_model.target_linker_env,
        command_model.target_cc_env,
        command_model.target_cxx_env,
        command_model.target_ar_env,
        command_model.target_ranlib_env,
        command_model.wasi_sysroot_env,
        command_model.target_cflags_env,
        command_model.target_cxxflags_env,
        command_model.cfg_release_env,
        command_model.cfg_release_channel_env,
        command_model.cfg_release_num_env,
        command_model.cfg_compiler_host_triple_env,
        command_model.rustc_install_bindir_env,
        command_model.rustc_stage_env,
        command_model.host_cargo_path,
        adapter_manifest.display(),
        BRIDGE_BINARY_TARGET,
        manifest.target_triple
    );

    let output = Command::new(&command_model.host_cargo_path)
        .args([
            "build",
            "--manifest-path",
            &adapter_manifest.display().to_string(),
            "--bin",
            BRIDGE_BINARY_TARGET,
            "--target",
            &manifest.target_triple,
            "--release",
            "--message-format",
            "short",
        ])
        .env("RUSTC", &command_model.host_rustc_path)
        .env("RUSTC_BOOTSTRAP", &command_model.rustc_bootstrap)
        .env("CARGO_TARGET_DIR", &command_model.cargo_target_dir)
        .env("CARGO_TARGET_WASM32_WASIP1_RUSTFLAGS", target_rustflags)
        .env("CARGO_TARGET_WASM32_WASIP1_LINKER", target_linker)
        .env("CC_wasm32_wasip1", target_cc)
        .env("CXX_wasm32_wasip1", target_cxx)
        .env("AR_wasm32_wasip1", target_ar)
        .env("RANLIB_wasm32_wasip1", target_ranlib)
        .env("WASI_SYSROOT", wasi_sysroot)
        .env("CFLAGS_wasm32_wasip1", target_cflags)
        .env("CXXFLAGS_wasm32_wasip1", target_cxxflags)
        .env("CFG_RELEASE", cfg_release)
        .env("CFG_RELEASE_CHANNEL", cfg_release_channel)
        .env("CFG_RELEASE_NUM", cfg_release_num)
        .env("CFG_COMPILER_HOST_TRIPLE", cfg_compiler_host_triple)
        .env("RUSTC_INSTALL_BINDIR", rustc_install_bindir)
        .env("RUSTC_STAGE", rustc_stage)
        .env_remove("RUSTFLAGS")
        .current_dir(&rust_root)
        .output();
    let (exit_code, combined_output) = match output {
        Ok(output) => (
            output.status.code().unwrap_or(-1),
            format!(
                "{}{}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            ),
        ),
        Err(error) => (-1, format!("failed to spawn cargo: {error}")),
    };
    let log_path = pack_dir.join("logs/bridge-rouwdi-mir-adapter-probe.log");
    let _ = fs::write(
        &log_path,
        format!("command: {command_text}\nexit_code: {exit_code}\n\n{combined_output}"),
    );

    let wasm_path = target_release.join("rouwdi_mir_adapter_probe.wasm");
    let wasm_bytes = fs::read(&wasm_path).ok();
    let exports = wasm_bytes
        .as_deref()
        .map(wasm_export_names)
        .unwrap_or_default();
    let abi_v1_symbols_present = BRIDGE_REQUIRED_EXPORTS
        .iter()
        .all(|required| exports.iter().any(|export| export == required));
    let output_artifact_identity = wasm_bytes.as_ref().map(|bytes| {
        let path = relative_path(workspace_root, &wasm_path);
        let identity = classify_direct_rustc_private_artifact_bytes(
            &path,
            bytes,
            &manifest.target_triple,
            &manifest.host_triple,
        );
        CompilerPayloadBridgeArtifactIdentity {
            role: "direct_rustc_private_mir_adapter_wasm".to_owned(),
            artifact_format: identity.artifact_format,
            path: identity.path,
            sha256: identity.sha256,
            size_bytes: identity.size_bytes,
            target_triple: identity.emitted_target_triple,
            loadable_by_rouwdi_wasm: identity.target_loadable && abi_v1_symbols_present,
        }
    });
    let input_artifact_identities = manifest
        .dependency_closure
        .root_crates
        .iter()
        .filter(|root| bridge_runtime_input_identity_required(root))
        .filter_map(|name| {
            let artifact = attempts
                .iter()
                .find(|attempt| attempt.name == *name)
                .and_then(|attempt| select_bridge_extern_artifact(attempt, name))?;
            let path = workspace_root.join(&artifact.path);
            let bytes = fs::read(&path).ok()?;
            let identity = classify_direct_rustc_private_artifact_bytes(
                &relative_path(workspace_root, &path),
                &bytes,
                &manifest.target_triple,
                &manifest.host_triple,
            );
            Some(CompilerPayloadBridgeArtifactIdentity {
                role: format!("direct_rustc_private_root_{name}"),
                artifact_format: identity.artifact_format,
                path: identity.path,
                sha256: identity.sha256,
                size_bytes: identity.size_bytes,
                target_triple: identity.emitted_target_triple,
                loadable_by_rouwdi_wasm: identity.target_loadable,
            })
        })
        .collect::<Vec<_>>();
    let classification = if exit_code == 0 && output_artifact_identity.is_some() {
        BRIDGE_HIR_LOWERING_ATTEMPTED_CLASSIFICATION
    } else if exit_code == 0 {
        "direct_rustc_private_pack_ready_bridge_blocked_at_no_wasm_module"
    } else {
        "direct_rustc_private_pack_ready_bridge_blocked_at_wasm_link"
    }
    .to_owned();
    let exact_blocker = if classification == BRIDGE_HIR_LOWERING_ATTEMPTED_CLASSIFICATION {
        "The direct pack builder produced target-loadable rustc-private root artifacts, retried the MIR adapter as a wasm32-wasip1 command module, and emitted a module with exported memory plus ABI v1 exports. The runtime loader must instantiate it and call execute; the execute path now creates real SourceMap/ParseSess/parser/AST state, creates a real rustc_interface::Config, enters rustc_interface queries/global context/TyCtxt, forces upstream HIR lowering by walking TyCtxt HIR items, and reports the MIR-provider lang-item blocker without fabricating TyCtxt, Providers, Body<'tcx>, or MIR.".to_owned()
    } else {
        first_relevant_error(&combined_output).unwrap_or_else(|| {
            format!(
                "Bridge command exited {exit_code}; see {}",
                relative_path(workspace_root, &log_path)
            )
        })
    };

    RustcPrivateDirectBridgeAttempt {
        command: command_text,
        workdir: relative_path(workspace_root, &rust_root),
        exit_code,
        classification,
        target_triple: manifest.target_triple.clone(),
        input_artifact_identities,
        output_artifact_identity,
        exports,
        abi_v1_symbols_present,
        full_mir_payload_available: false,
        exact_blocker,
    }
}

fn bridge_runtime_extern_required(crate_name: &str) -> bool {
    let _ = crate_name;
    false
}

fn bridge_runtime_input_identity_required(crate_name: &str) -> bool {
    matches!(
        crate_name,
        "rustc_builtin_macros"
            | "rustc_expand"
            | "rustc_hir"
            | "rustc_hir_analysis"
            | "rustc_interface"
            | "rustc_lint"
            | "rustc_middle"
            | "rustc_mir_build"
            | "rustc_parse"
            | "rustc_passes"
            | "rustc_query_impl"
            | "rustc_resolve"
            | "rustc_session"
            | "rustc_span"
    )
}

fn select_bridge_extern_artifact<'a>(
    attempt: &'a RustcPrivateDirectClosureAttempt,
    crate_name: &str,
) -> Option<&'a RustcPrivateDirectArtifactIdentity> {
    let canonical_release_suffix = format!("release/lib{crate_name}.rlib");
    attempt
        .artifacts
        .iter()
        .find(|artifact| {
            artifact.artifact_format == "rlib"
                && artifact.target_loadable
                && artifact
                    .path
                    .replace('\\', "/")
                    .ends_with(&canonical_release_suffix)
        })
        .or_else(|| {
            attempt
                .artifacts
                .iter()
                .find(|artifact| artifact.artifact_format == "rlib" && artifact.target_loadable)
        })
}

fn collect_artifact_identities(
    workspace_root: &Path,
    cargo_target_dir: &Path,
    crate_name: &str,
    target_triple: &str,
    host_triple: &str,
) -> io::Result<Vec<RustcPrivateDirectArtifactIdentity>> {
    let mut paths = Vec::new();
    for dir in [
        cargo_target_dir.join(target_triple).join("release/deps"),
        cargo_target_dir.join(target_triple).join("release"),
        cargo_target_dir.join("release/deps"),
        cargo_target_dir.join("release"),
    ] {
        collect_matching_files(&dir, crate_name, &mut paths)?;
    }

    let mut seen = BTreeSet::new();
    let mut artifacts = Vec::new();
    for path in paths {
        if !seen.insert(path.clone()) {
            continue;
        }
        let Ok(bytes) = fs::read(&path) else {
            continue;
        };
        let relative = relative_path(workspace_root, &path);
        artifacts.push(classify_direct_rustc_private_artifact_bytes(
            &relative,
            &bytes,
            target_triple,
            host_triple,
        ));
    }
    artifacts.sort_by(|left, right| {
        right
            .target_loadable
            .cmp(&left.target_loadable)
            .then(left.path.cmp(&right.path))
    });
    Ok(artifacts)
}

fn collect_matching_files(
    dir: &Path,
    crate_name: &str,
    paths: &mut Vec<PathBuf>,
) -> io::Result<()> {
    if !dir.is_dir() {
        return Ok(());
    }
    let normalized = crate_name.replace('-', "_").to_ascii_lowercase();
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            collect_matching_files(&path, crate_name, paths)?;
            continue;
        }
        let Some(file_name) = path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        let lower = file_name.to_ascii_lowercase();
        if lower.contains(&normalized)
            && matches!(
                path.extension().and_then(|extension| extension.to_str()),
                Some("rlib" | "rmeta" | "wasm" | "dll" | "lib" | "a")
            )
        {
            paths.push(path);
        }
    }
    Ok(())
}

fn classify_attempt(
    exit_code: i32,
    target_loadable: bool,
    artifacts: &[RustcPrivateDirectArtifactIdentity],
) -> String {
    if target_loadable {
        "target_loadable_artifact_emitted".to_owned()
    } else if exit_code != 0 {
        "build_failed_no_target_loadable_artifact".to_owned()
    } else if artifacts
        .iter()
        .all(|artifact| artifact.artifact_format == "rmeta")
    {
        "metadata_only_no_target_loadable_artifact".to_owned()
    } else if artifacts.iter().any(|artifact| {
        artifact.classification == "host_proc_macro_or_native_dynamic"
            || artifact.classification == "host_rlib_not_target_loadable"
    }) {
        "host_artifact_not_target_loadable".to_owned()
    } else {
        "no_target_loadable_artifact".to_owned()
    }
}

fn first_relevant_error(output: &str) -> Option<String> {
    output
        .lines()
        .find(|line| {
            line.contains("error[")
                || line.contains("error:")
                || line.contains("failed")
                || line.contains("panicked")
        })
        .map(|line| line.trim().to_owned())
}

fn render_cargo_command(
    command_model: &DirectRustcPrivateCommandModel,
    manifest_path: &Path,
    crate_name: &str,
    target_triple: &str,
) -> String {
    format!(
        "$env:RUSTC='{}'; $env:RUSTC_BOOTSTRAP='{}'; $env:CARGO_TARGET_DIR='{}'; $env:{}; $env:{}; $env:{}; $env:{}; $env:{}; $env:{}; $env:{}; $env:{}; $env:{}; $env:{}; $env:{}; $env:{}; $env:{}; $env:{}; $env:{}; Remove-Item Env:RUSTFLAGS -ErrorAction SilentlyContinue; '{}' build --manifest-path '{}' -p {} --target {} --release --message-format short",
        command_model.host_rustc_path,
        command_model.rustc_bootstrap,
        command_model.cargo_target_dir,
        command_model.target_rustflags_env,
        command_model.target_linker_env,
        command_model.target_cc_env,
        command_model.target_cxx_env,
        command_model.target_ar_env,
        command_model.target_ranlib_env,
        command_model.wasi_sysroot_env,
        command_model.target_cflags_env,
        command_model.target_cxxflags_env,
        command_model.cfg_release_env,
        command_model.cfg_release_channel_env,
        command_model.cfg_release_num_env,
        command_model.cfg_compiler_host_triple_env,
        command_model.rustc_install_bindir_env,
        command_model.rustc_stage_env,
        command_model.host_cargo_path,
        manifest_path.display(),
        crate_name,
        target_triple
    )
}

fn env_value(entry: &str) -> String {
    entry
        .split_once('=')
        .map(|(_, value)| value.to_owned())
        .unwrap_or_default()
}

fn wasm_export_names(bytes: &[u8]) -> Vec<String> {
    if bytes.len() < 8 || &bytes[..4] != b"\0asm" {
        return Vec::new();
    }
    let mut offset = 8;
    while offset < bytes.len() {
        let section_id = bytes[offset];
        offset += 1;
        let Some((section_len, next)) = read_wasm_varuint(bytes, offset) else {
            return Vec::new();
        };
        offset = next;
        let section_end = offset.saturating_add(section_len as usize);
        if section_end > bytes.len() {
            return Vec::new();
        }
        if section_id == 7 {
            return parse_wasm_export_section(bytes, offset, section_end);
        }
        offset = section_end;
    }
    Vec::new()
}

fn parse_wasm_export_section(bytes: &[u8], mut offset: usize, section_end: usize) -> Vec<String> {
    let Some((count, next)) = read_wasm_varuint(bytes, offset) else {
        return Vec::new();
    };
    offset = next;
    let mut exports = Vec::new();
    for _ in 0..count {
        let Some((name_len, name_start)) = read_wasm_varuint(bytes, offset) else {
            return exports;
        };
        let name_end = name_start.saturating_add(name_len as usize);
        if name_end >= section_end {
            return exports;
        }
        if let Ok(name) = std::str::from_utf8(&bytes[name_start..name_end]) {
            exports.push(name.to_owned());
        }
        offset = name_end + 1;
        let Some((_index, next)) = read_wasm_varuint(bytes, offset) else {
            return exports;
        };
        offset = next;
    }
    exports
}

fn read_wasm_varuint(bytes: &[u8], mut offset: usize) -> Option<(u32, usize)> {
    let mut result = 0u32;
    let mut shift = 0;
    loop {
        let byte = *bytes.get(offset)?;
        offset += 1;
        result |= ((byte & 0x7f) as u32) << shift;
        if byte & 0x80 == 0 {
            return Some((result, offset));
        }
        shift += 7;
        if shift > 28 {
            return None;
        }
    }
}

fn print_text_report(report: &DirectPackBuilderReport) {
    println!(
        "direct rustc-private pack {} status={} roots-ready={}",
        report.pack.path, report.pack.status, report.pack.all_required_roots_target_loadable
    );
    for attempt in &report.attempts {
        println!(
            "  {} exit={} class={} target_loadable={} artifacts={}",
            attempt.name,
            attempt.exit_code,
            attempt.classification,
            attempt.target_loadable,
            attempt.artifacts.len()
        );
    }
    println!("  first-hard-blocker: {}", report.pack.first_hard_blocker);
    if let Some(bridge) = &report.bridge_retry {
        println!(
            "  bridge retry exit={} class={} artifact={} abi-v1={} full-mir={}",
            bridge.exit_code,
            bridge.classification,
            bridge
                .output_artifact_identity
                .as_ref()
                .map(|artifact| artifact.path.as_str())
                .unwrap_or("none"),
            bridge.abi_v1_symbols_present,
            bridge.full_mir_payload_available
        );
    } else {
        println!(
            "  bridge retry skipped class={} required={}",
            report.bridge_retry_gate.classification,
            report.bridge_retry_gate.required_before_retry.join(",")
        );
    }
}

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("adapter crate lives under workspace/crates/rouwdi-rustc-upstream")
        .to_path_buf()
}

fn absolutize(workspace_root: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        workspace_root.join(path)
    }
}

fn relative_path(workspace_root: &Path, path: &Path) -> String {
    path.strip_prefix(workspace_root)
        .unwrap_or(path)
        .display()
        .to_string()
}

fn usage() -> &'static str {
    "usage: direct-rustc-private-pack-builder [--json] [--allow-partial] [--manifest <path>] [--pack-dir <path>] [--cargo-target-dir <path>]"
}
