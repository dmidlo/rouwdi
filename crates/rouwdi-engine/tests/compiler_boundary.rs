use rouwdi_engine::{BuildRequest, RouwdiEngine};
use rouwdi_proof::{RouwdiRunManifest, RunStatus};
use rouwdi_rustc::{RustCompilerPipelineStatus, RustCompilerStage};
use rouwdi_vfs::{MemoryStorage, Storage};

fn no_deps_wasi_binary_storage() -> MemoryStorage {
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
profile = "release"

[source]
mode = "snapshot"
root = "."

[[targets]]
name = "wasi"
triple = "wasm32-wasip1"
artifact = "module"

[targets.interface]
required_exports = ["_start"]

[targets.runtime]
required = true
kind = "wasi"
expected_exit_code = 0
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
    storage
        .write("src/main.rs", b"fn main() { println!(\"hello\"); }\n")
        .unwrap();
    storage
}

#[test]
fn no_deps_wasi_binary_reaches_internal_compiler_boundary() {
    let mut storage = no_deps_wasi_binary_storage();

    let report = RouwdiEngine::default()
        .build(&mut storage, BuildRequest::default())
        .unwrap();
    let manifest: RouwdiRunManifest =
        serde_json::from_slice(&storage.read(&report.manifest_path).unwrap()).unwrap();

    assert_eq!(report.status, RunStatus::Failed);
    assert_eq!(manifest.compiler_pipeline.len(), 1);
    let record = &manifest.compiler_pipeline[0];
    assert_eq!(record.unit_id, "app:rust:app:wasm32-wasip1");
    assert_eq!(record.source_path, "src/main.rs");
    assert_eq!(record.status, RustCompilerPipelineStatus::MissingStage);
    let missing_stage = record.missing_stage.as_ref().unwrap();
    assert_eq!(missing_stage.stage, RustCompilerStage::Parse);
    assert_eq!(missing_stage.required_component, "rustc_parse");
    assert!(report.unsupported.iter().any(|item| {
        item.capability == "compiler stage rustc_parse"
            && item.required_by == "compile unit app:rust:app:wasm32-wasip1"
    }));
    assert!(!report
        .unsupported
        .iter()
        .any(|item| item.capability == "rustc frontend semantics"));
}
