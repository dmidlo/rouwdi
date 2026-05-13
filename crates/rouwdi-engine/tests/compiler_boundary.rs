use rouwdi_cargo::{CargoBuildPlan, CompilePhase};
use rouwdi_engine::{BuildRequest, RouwdiEngine};
use rouwdi_proof::{RouwdiRunManifest, RunStatus};
use rouwdi_rustc::{RustCompilerPipelineStatus, RustCompilerStage, RustCompilerStageErrorCode};
use rouwdi_vfs::{MemoryStorage, Storage};
use std::collections::BTreeSet;

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
    assert_eq!(
        missing_stage.error_code,
        RustCompilerStageErrorCode::RustcParseNotEmbedded
    );
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

#[test]
fn engine_records_compiler_pipeline_for_every_rust_compile_unit() {
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

[dependencies]
helper = { path = "helper" }
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
dependencies = [
 "helper",
]

[[package]]
name = "helper"
version = "0.1.0"
"#,
        )
        .unwrap();
    storage
        .write("src/main.rs", b"fn main() { helper::message(); }\n")
        .unwrap();
    storage
        .write(
            "helper/Cargo.toml",
            br#"
[package]
name = "helper"
version = "0.1.0"
edition = "2021"
"#,
        )
        .unwrap();
    storage
        .write("helper/src/lib.rs", b"pub fn message() {}\n")
        .unwrap();

    let report = RouwdiEngine::default()
        .build(&mut storage, BuildRequest::default())
        .unwrap();
    let manifest: RouwdiRunManifest =
        serde_json::from_slice(&storage.read(&report.manifest_path).unwrap()).unwrap();
    let build_plan: CargoBuildPlan = serde_json::from_slice(
        &storage
            .read(&format!("{}/graph/build-plan.json", report.run_root))
            .unwrap(),
    )
    .unwrap();

    let rust_unit_ids = build_plan
        .units
        .iter()
        .filter(|unit| unit.phase == CompilePhase::Rust)
        .map(|unit| unit.id.clone())
        .collect::<BTreeSet<_>>();
    let pipeline_unit_ids = manifest
        .compiler_pipeline
        .iter()
        .map(|record| record.unit_id.clone())
        .collect::<BTreeSet<_>>();

    assert_eq!(report.status, RunStatus::Failed);
    assert_eq!(pipeline_unit_ids, rust_unit_ids);
    assert_eq!(pipeline_unit_ids.len(), 2);
    assert!(pipeline_unit_ids.contains("app:rust:app:wasm32-wasip1"));
    assert!(pipeline_unit_ids.contains("helper:rust:helper:wasm32-wasip1"));
    assert!(manifest.compiler_pipeline.iter().all(|record| {
        record.status == RustCompilerPipelineStatus::MissingStage
            && record.missing_stage.as_ref().is_some_and(|missing| {
                missing.stage == RustCompilerStage::Parse
                    && missing.error_code == RustCompilerStageErrorCode::RustcParseNotEmbedded
            })
    }));
}
