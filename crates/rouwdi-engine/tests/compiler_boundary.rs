use rouwdi_cargo::{CargoBuildPlan, CompilePhase};
use rouwdi_engine::{BuildRequest, RouwdiEngine};
use rouwdi_proof::{RouwdiRunManifest, RunStatus};
use rouwdi_rustc::{
    RustCompilerPipelineStatus, RustCompilerStage, RustCompilerStageErrorCode,
    RustExpansionStageRecord, RustExpansionStageStatus, RustNameResolutionDiagnosticCode,
    RustNameResolutionStageRecord, RustNameResolutionStageStatus, RustParseStageRecord,
    RustParseStageStatus, RustTypeCheckDiagnosticCode, RustTypeCheckStageRecord,
    RustTypeCheckStageStatus,
};
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
    storage.write("src/main.rs", b"fn main() {}\n").unwrap();
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
    assert_eq!(
        record.parse_stage.as_ref().unwrap().status,
        RustParseStageStatus::Succeeded
    );
    assert_eq!(
        record.expansion_stage.as_ref().unwrap().status,
        RustExpansionStageStatus::NoExpansionRequired
    );
    assert_eq!(
        record.name_resolution_stage.as_ref().unwrap().status,
        RustNameResolutionStageStatus::Succeeded
    );
    let missing_stage = record.missing_stage.as_ref().unwrap();
    assert_eq!(missing_stage.stage, RustCompilerStage::BorrowChecking);
    assert_eq!(
        missing_stage.error_code,
        RustCompilerStageErrorCode::BorrowckNotEmbedded
    );
    assert_eq!(missing_stage.required_component, "rustc_borrowck");
    assert_eq!(
        record.type_check_stage.as_ref().unwrap().status,
        RustTypeCheckStageStatus::Succeeded
    );
    assert!(report.bootstrap_diagnostics.iter().any(|item| {
        item.component == "compiler stage rustc_borrowck"
            && item.required_by == "compile unit app:rust:app:wasm32-wasip1"
    }));
    assert!(!report
        .bootstrap_diagnostics
        .iter()
        .any(|item| item.component == "rustc frontend semantics"));
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
            && record.parse_stage.as_ref().is_some_and(|parse| {
                parse.status == RustParseStageStatus::Succeeded
                    && parse.stage == RustCompilerStage::Parse
            })
            && record.expansion_stage.as_ref().is_some_and(|expansion| {
                expansion.status == RustExpansionStageStatus::NoExpansionRequired
                    && expansion.stage == RustCompilerStage::MacroExpansion
            })
            && record
                .name_resolution_stage
                .as_ref()
                .is_some_and(|name_resolution| {
                    name_resolution.status == RustNameResolutionStageStatus::Succeeded
                        && name_resolution.stage == RustCompilerStage::NameResolution
                })
            && record.type_check_stage.as_ref().is_some_and(|type_check| {
                type_check.status == RustTypeCheckStageStatus::Succeeded
                    && type_check.stage == RustCompilerStage::TypeChecking
            })
            && record.missing_stage.as_ref().is_some_and(|missing| {
                missing.stage == RustCompilerStage::BorrowChecking
                    && missing.error_code == RustCompilerStageErrorCode::BorrowckNotEmbedded
            })
    }));
    let parse_records: Vec<RustParseStageRecord> = serde_json::from_slice(
        &storage
            .read(&format!("{}/graph/rust-source-parse.json", report.run_root))
            .unwrap(),
    )
    .unwrap();
    let parse_unit_ids = parse_records
        .iter()
        .map(|record| record.unit_id.clone())
        .collect::<BTreeSet<_>>();
    assert_eq!(parse_unit_ids, rust_unit_ids);
    assert!(parse_records
        .iter()
        .all(|record| record.status == RustParseStageStatus::Succeeded));
    let expansion_records: Vec<RustExpansionStageRecord> = serde_json::from_slice(
        &storage
            .read(&format!(
                "{}/graph/rust-source-expansion.json",
                report.run_root
            ))
            .unwrap(),
    )
    .unwrap();
    let expansion_unit_ids = expansion_records
        .iter()
        .map(|record| record.unit_id.clone())
        .collect::<BTreeSet<_>>();
    assert_eq!(expansion_unit_ids, rust_unit_ids);
    assert!(expansion_records
        .iter()
        .all(|record| record.status == RustExpansionStageStatus::NoExpansionRequired));
    let name_resolution_records: Vec<RustNameResolutionStageRecord> = serde_json::from_slice(
        &storage
            .read(&format!(
                "{}/graph/rust-source-name-resolution.json",
                report.run_root
            ))
            .unwrap(),
    )
    .unwrap();
    let name_resolution_unit_ids = name_resolution_records
        .iter()
        .map(|record| record.unit_id.clone())
        .collect::<BTreeSet<_>>();
    assert_eq!(name_resolution_unit_ids, rust_unit_ids);
    assert!(name_resolution_records
        .iter()
        .all(|record| record.status == RustNameResolutionStageStatus::Succeeded));
    assert!(name_resolution_records.iter().any(|record| {
        record.unit_id == "app:rust:app:wasm32-wasip1"
            && record
                .extern_prelude
                .iter()
                .any(|krate| krate.name == "helper")
            && record
                .resolved_paths
                .iter()
                .any(|path| path.path == "helper::message")
    }));
    let type_check_records: Vec<RustTypeCheckStageRecord> = serde_json::from_slice(
        &storage
            .read(&format!(
                "{}/graph/rust-source-type-check.json",
                report.run_root
            ))
            .unwrap(),
    )
    .unwrap();
    let type_check_unit_ids = type_check_records
        .iter()
        .map(|record| record.unit_id.clone())
        .collect::<BTreeSet<_>>();
    assert_eq!(type_check_unit_ids, rust_unit_ids);
    assert!(type_check_records
        .iter()
        .all(|record| record.status == RustTypeCheckStageStatus::Succeeded));
}

#[test]
fn macro_invocation_stops_at_expansion_stage() {
    let mut storage = no_deps_wasi_binary_storage();
    storage
        .write("src/main.rs", b"fn main() { println!(\"hello\"); }\n")
        .unwrap();

    let report = RouwdiEngine::default()
        .build(&mut storage, BuildRequest::default())
        .unwrap();
    let manifest: RouwdiRunManifest =
        serde_json::from_slice(&storage.read(&report.manifest_path).unwrap()).unwrap();
    let expansion_records: Vec<RustExpansionStageRecord> = serde_json::from_slice(
        &storage
            .read(&format!(
                "{}/graph/rust-source-expansion.json",
                report.run_root
            ))
            .unwrap(),
    )
    .unwrap();

    assert_eq!(report.status, RunStatus::Failed);
    assert_eq!(manifest.compiler_pipeline.len(), 1);
    let record = &manifest.compiler_pipeline[0];
    assert_eq!(record.status, RustCompilerPipelineStatus::ExpansionError);
    assert!(record.missing_stage.is_none());
    assert_eq!(
        record.expansion_stage.as_ref().unwrap().status,
        RustExpansionStageStatus::ExpansionRequired
    );
    assert!(record
        .expansion_stage
        .as_ref()
        .unwrap()
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.feature == "macro_invocation"
            && diagnostic.message.contains("rustc_expand is not embedded")));
    assert_eq!(expansion_records.len(), 1);
    assert_eq!(
        expansion_records[0].status,
        RustExpansionStageStatus::ExpansionRequired
    );
    assert!(report.bootstrap_diagnostics.iter().any(|item| {
        item.component == "Rust macro expansion stage"
            && item.required_by == "compile unit app:rust:app:wasm32-wasip1"
            && item.reason.contains("macro_invocation")
    }));
    assert!(!report.bootstrap_diagnostics.iter().any(|item| {
        item.component == "compiler stage rustc_expand"
            || item.reason.contains("macro_expansion_not_embedded")
    }));
}

#[test]
fn unresolved_name_stops_at_name_resolution_stage() {
    let mut storage = no_deps_wasi_binary_storage();
    storage
        .write("src/main.rs", b"fn main() { missing::call(); }\n")
        .unwrap();

    let report = RouwdiEngine::default()
        .build(&mut storage, BuildRequest::default())
        .unwrap();
    let manifest: RouwdiRunManifest =
        serde_json::from_slice(&storage.read(&report.manifest_path).unwrap()).unwrap();
    let name_resolution_records: Vec<RustNameResolutionStageRecord> = serde_json::from_slice(
        &storage
            .read(&format!(
                "{}/graph/rust-source-name-resolution.json",
                report.run_root
            ))
            .unwrap(),
    )
    .unwrap();

    assert_eq!(report.status, RunStatus::Failed);
    assert_eq!(manifest.compiler_pipeline.len(), 1);
    let record = &manifest.compiler_pipeline[0];
    assert_eq!(
        record.status,
        RustCompilerPipelineStatus::NameResolutionError
    );
    assert!(record.missing_stage.is_none());
    let name_resolution = record.name_resolution_stage.as_ref().unwrap();
    assert_eq!(
        name_resolution.status,
        RustNameResolutionStageStatus::Failed
    );
    assert!(name_resolution.diagnostics.iter().any(|diagnostic| {
        diagnostic.code == RustNameResolutionDiagnosticCode::UnresolvedPath
            && diagnostic.path == "missing::call"
    }));
    assert_eq!(name_resolution_records.len(), 1);
    assert_eq!(
        name_resolution_records[0].status,
        RustNameResolutionStageStatus::Failed
    );
    assert!(report.bootstrap_diagnostics.iter().any(|item| {
        item.component == "Rust name resolution stage"
            && item.required_by == "compile unit app:rust:app:wasm32-wasip1"
    }));
    assert!(!report.bootstrap_diagnostics.iter().any(|item| {
        item.component == "compiler stage rustc_borrowck"
            || item.reason.contains("borrowck_not_embedded")
    }));
}

#[test]
fn typed_invalid_source_stops_at_type_check_stage() {
    let mut storage = no_deps_wasi_binary_storage();
    storage
        .write("src/main.rs", b"fn main() { let answer: bool = 1; }\n")
        .unwrap();

    let report = RouwdiEngine::default()
        .build(&mut storage, BuildRequest::default())
        .unwrap();
    let manifest: RouwdiRunManifest =
        serde_json::from_slice(&storage.read(&report.manifest_path).unwrap()).unwrap();
    let type_check_records: Vec<RustTypeCheckStageRecord> = serde_json::from_slice(
        &storage
            .read(&format!(
                "{}/graph/rust-source-type-check.json",
                report.run_root
            ))
            .unwrap(),
    )
    .unwrap();

    assert_eq!(report.status, RunStatus::Failed);
    assert_eq!(manifest.compiler_pipeline.len(), 1);
    let record = &manifest.compiler_pipeline[0];
    assert_eq!(record.status, RustCompilerPipelineStatus::TypeCheckError);
    assert!(record.missing_stage.is_none());
    let type_check = record.type_check_stage.as_ref().unwrap();
    assert_eq!(type_check.status, RustTypeCheckStageStatus::Failed);
    assert!(type_check.diagnostics.iter().any(|diagnostic| {
        diagnostic.code == RustTypeCheckDiagnosticCode::MismatchedTypes
            && diagnostic.expected.as_deref() == Some("bool")
            && diagnostic.actual.as_deref() == Some("integer")
    }));
    assert_eq!(type_check_records.len(), 1);
    assert_eq!(
        type_check_records[0].status,
        RustTypeCheckStageStatus::Failed
    );
    assert!(report.bootstrap_diagnostics.iter().any(|item| {
        item.component == "Rust type-check stage"
            && item.required_by == "compile unit app:rust:app:wasm32-wasip1"
    }));
    assert!(!report.bootstrap_diagnostics.iter().any(|item| {
        item.component == "compiler stage rustc_borrowck"
            || item.reason.contains("borrowck_not_embedded")
    }));
}

#[test]
fn invalid_rust_syntax_stops_at_parse_stage() {
    let mut storage = no_deps_wasi_binary_storage();
    storage
        .write("src/main.rs", b"fn main( { let = ; }\n")
        .unwrap();

    let report = RouwdiEngine::default()
        .build(&mut storage, BuildRequest::default())
        .unwrap();
    let manifest: RouwdiRunManifest =
        serde_json::from_slice(&storage.read(&report.manifest_path).unwrap()).unwrap();
    let parse_records: Vec<RustParseStageRecord> = serde_json::from_slice(
        &storage
            .read(&format!("{}/graph/rust-source-parse.json", report.run_root))
            .unwrap(),
    )
    .unwrap();

    assert_eq!(report.status, RunStatus::Failed);
    assert_eq!(manifest.compiler_pipeline.len(), 1);
    let record = &manifest.compiler_pipeline[0];
    assert_eq!(record.status, RustCompilerPipelineStatus::ParseError);
    assert!(record.missing_stage.is_none());
    assert_eq!(
        record.parse_stage.as_ref().unwrap().status,
        RustParseStageStatus::Failed
    );
    assert!(record.parse_stage.as_ref().unwrap().diagnostic_count > 0);
    assert_eq!(parse_records.len(), 1);
    assert_eq!(parse_records[0].status, RustParseStageStatus::Failed);
    assert!(report
        .bootstrap_diagnostics
        .iter()
        .any(|item| item.component == "Rust parse stage"));
    assert!(!report.bootstrap_diagnostics.iter().any(|item| {
        item.component == "compiler stage rustc_parse"
            || item.reason.contains("rustc_parse_not_embedded")
    }));
}
