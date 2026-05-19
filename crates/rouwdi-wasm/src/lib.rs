use rouwdi_engine::{BuildRequest, RouwdiEngine};
use rouwdi_proof::RunStatus;
use rouwdi_vfs::{HostStorage, Storage};
use std::path::PathBuf;

pub mod payloads;

#[no_mangle]
pub extern "C" fn rouwdi_abi_version() -> u32 {
    1
}

#[no_mangle]
pub extern "C" fn rouwdi_host_toolchain_dependency_count() -> u32 {
    0
}

#[no_mangle]
pub extern "C" fn rouwdi_embedded_compiler_payload_count() -> u32 {
    payloads::embedded_compiler_payloads().len() as u32
}

#[no_mangle]
pub extern "C" fn rouwdi_mir_payload_embedded_size_bytes() -> u64 {
    payloads::embedded_compiler_payloads()
        .iter()
        .find(|payload| payload.name == "rouwdi-mir-handoff-payload")
        .map(|payload| payload.bytes.len() as u64)
        .unwrap_or(0)
}

#[no_mangle]
pub extern "C" fn rouwdi_mir_payload_embedded_hash_verified() -> u32 {
    payloads::mir_payload_report()
        .filter(|report| report.hash_verified && report.size_verified)
        .map(|_| 1)
        .unwrap_or(0)
}

pub fn build_with_storage(storage: &mut dyn Storage, contract_path: &str) -> i32 {
    let engine = RouwdiEngine::default()
        .with_embedded_mir_payload_execution_provider(
            payloads::mir_payload_execution_for_engine_request,
        )
        .with_embedded_linked_wasi_module_provider(
            payloads::linked_wasi_module_artifact_for_engine,
        );
    match engine.build(
        storage,
        BuildRequest {
            contract_path: contract_path.to_owned(),
        },
    ) {
        Ok(report) => match report.status {
            RunStatus::Succeeded => 0,
            RunStatus::Unsupported => 2,
            RunStatus::Failed => 1,
        },
        Err(_) => 1,
    }
}

pub fn cli_main() -> i32 {
    let mut args = std::env::args().skip(1);
    let command = args.next().unwrap_or_else(|| "build".to_owned());
    match command.as_str() {
        "build" => {
            let contract_path = args.next().unwrap_or_else(|| "rouwdi.toml".to_owned());
            let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
            let mut storage = HostStorage::new(cwd);
            let engine = RouwdiEngine::default()
                .with_embedded_mir_payload_execution_provider(
                    payloads::mir_payload_execution_for_engine_request,
                )
                .with_embedded_linked_wasi_module_provider(
                    payloads::linked_wasi_module_artifact_for_engine,
                );
            match engine.build(&mut storage, BuildRequest { contract_path }) {
                Ok(report) => {
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&report).unwrap_or_default()
                    );
                    match report.status {
                        RunStatus::Succeeded => 0,
                        RunStatus::Unsupported => 2,
                        RunStatus::Failed => 1,
                    }
                }
                Err(err) => {
                    eprintln!("{err}");
                    1
                }
            }
        }
        "verify" => {
            let run_root = args
                .next()
                .unwrap_or_else(|| ".rouwdi/runs/latest".to_owned());
            let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
            let storage = HostStorage::new(cwd);
            match RouwdiEngine::default().verify(&storage, &run_root) {
                Ok(report) => {
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&report).unwrap_or_default()
                    );
                    0
                }
                Err(err) => {
                    eprintln!("{err}");
                    1
                }
            }
        }
        "abi-version" => {
            println!("{}", rouwdi_abi_version());
            0
        }
        "payloads" => match payloads::load_mir_handoff_payload() {
            Ok(report) => {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&report).unwrap_or_default()
                );
                if report.hash_verified
                    && report.size_verified
                    && report.module_instantiated
                    && report.abi_v1_exports_verified
                    && report.execute_called
                    && report.execution_source == "embedded_registry"
                    && !report.external
                {
                    0
                } else {
                    1
                }
            }
            Err(err) => {
                println!("{}", serde_json::to_string_pretty(&err).unwrap_or_default());
                1
            }
        },
        "codegen-payloads" => {
            let source_path = args.next().unwrap_or_else(|| "src/main.rs".to_owned());
            let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
            let source_bytes = match std::fs::read(cwd.join(&source_path)) {
                Ok(bytes) => bytes,
                Err(err) => {
                    eprintln!("failed to read codegen payload source {source_path}: {err}");
                    return 1;
                }
            };
            let source_sha256 = {
                use sha2::{Digest, Sha256};
                let digest = Sha256::digest(&source_bytes);
                let mut out = String::with_capacity(digest.len() * 2);
                for byte in digest {
                    use std::fmt::Write as _;
                    let _ = write!(&mut out, "{byte:02x}");
                }
                out
            };
            let request = rouwdi_engine::EmbeddedLinkedWasiModuleRequest {
                compile_unit_id: "app:rust:app:wasm32-wasip1".to_owned(),
                package: "app".to_owned(),
                target: "wasi".to_owned(),
                cargo_target_kind: "Bin".to_owned(),
                source_path,
                source_bytes,
                source_sha256,
                profile: "release".to_owned(),
                target_triple: "wasm32-wasip1".to_owned(),
                crate_name: "rouwdi_payload".to_owned(),
                mir_body_hash: "diagnostic-codegen-payload".to_owned(),
                mono_item_graph_hash: "diagnostic-codegen-payload".to_owned(),
                mono_items: vec!["fn:rouwdi_payload::main".to_owned()],
            };
            match payloads::load_codegen_backend_payload(&request) {
                Ok(report) => {
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&report).unwrap_or_default()
                    );
                    if report.hash_verified
                        && report.size_verified
                        && report.module_instantiated
                        && report.start_called
                        && report.execution_source == "embedded_registry"
                        && !report.external
                        && report.backend_constructed
                        && report.llvm_module_created
                        && report.target_machine_created
                        && report.llvm_ir_emitted
                        && report.object_emission_attempted
                        && (!report.linker_handoff_created
                            || (report.wasm_object_bytes_emitted
                                && report.final_module_sha256.is_some()
                                && report.final_module_size_bytes.is_some()
                                && report.final_module_artifact_path.is_some()
                                && report.runtime_proof_attempted
                                && report.runtime_proof_passed))
                    {
                        0
                    } else {
                        1
                    }
                }
                Err(err) => {
                    println!("{}", serde_json::to_string_pretty(&err).unwrap_or_default());
                    1
                }
            }
        }
        _ => {
            eprintln!("unknown rouwdi command: {command}");
            64
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rouwdi_vfs::MemoryStorage;

    const HOST_WIT: &str = include_str!("../wit/rouwdi.wit");

    #[test]
    fn exports_no_host_toolchain_dependency_claim() {
        assert_eq!(rouwdi_abi_version(), 1);
        assert_eq!(rouwdi_host_toolchain_dependency_count(), 0);
        assert!(rouwdi_embedded_compiler_payload_count() >= 1);
        assert!(rouwdi_mir_payload_embedded_size_bytes() > 80_000_000);
        assert_eq!(rouwdi_mir_payload_embedded_hash_verified(), 1);
    }

    #[test]
    fn host_wit_exposes_substrate_not_toolchain_calls() {
        for forbidden in ["run-cargo", "run-rustc", "run-linker", "cargo:", "rustc:"] {
            assert!(
                !HOST_WIT.contains(forbidden),
                "host WIT must not expose {forbidden}"
            );
        }
        for required in [
            "interface storage",
            "interface network",
            "interface host-runtime",
        ] {
            assert!(
                HOST_WIT.contains(required),
                "host WIT is missing {required}"
            );
        }
    }

    #[test]
    fn storage_build_emits_first_wasi_module_artifact() {
        let mut storage = MemoryStorage::new();
        storage
            .write(
                "rouwdi.toml",
                br#"
contract_version = 1
[project]
manifest_path = "Cargo.toml"
package = "app"
bin = "app"
[source]
mode = "snapshot"
[[targets]]
name = "wasi"
triple = "wasm32-wasip1"
artifact = "module"
"#,
            )
            .unwrap();
        storage
            .write(
                "Cargo.toml",
                br#"
[package]
name = "app"
version = "0.1.0"
edition = "2021"
"#,
            )
            .unwrap();
        storage
            .write(
                "Cargo.lock",
                br#"
version = 4

[[package]]
name = "app"
version = "0.1.0"
"#,
            )
            .unwrap();
        storage.write("src/main.rs", b"fn main() {}\n").unwrap();

        let report = RouwdiEngine::default()
            .with_embedded_mir_payload_execution_provider(
                payloads::mir_payload_execution_for_engine_request,
            )
            .with_embedded_linked_wasi_module_provider(
                payloads::linked_wasi_module_artifact_for_engine,
            )
            .build(
                &mut storage,
                BuildRequest {
                    contract_path: "rouwdi.toml".to_owned(),
                },
            )
            .expect("engine build should not return an error");

        assert_eq!(report.status, RunStatus::Succeeded);
    }
}
