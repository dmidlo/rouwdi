use rouwdi_cargo::{CargoBuildPlan, CompilePhase};
use rouwdi_engine::{BuildRequest, RouwdiEngine};
use rouwdi_proof::{
    ArtifactPipelineRecord, ArtifactPipelineStageStatus, RouwdiRunManifest, RunStatus,
};
use rouwdi_rustc::{
    RustBorrowCheckDiagnosticCode, RustBorrowCheckStageRecord, RustBorrowCheckStageStatus,
    RustCompilerPipelineStatus, RustCompilerStage, RustExpansionStageRecord,
    RustExpansionStageStatus, RustMirHandoffBlockerCategory, RustMirHandoffRecord,
    RustMirHandoffStatus, RustNameResolutionDiagnosticCode, RustNameResolutionStageRecord,
    RustNameResolutionStageStatus, RustParseStageRecord, RustParseStageStatus,
    RustTypeCheckDiagnosticCode, RustTypeCheckStageRecord, RustTypeCheckStageStatus,
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
    assert_eq!(record.status, RustCompilerPipelineStatus::MirHandoffBlocked);
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
    assert_eq!(
        record.type_check_stage.as_ref().unwrap().status,
        RustTypeCheckStageStatus::Succeeded
    );
    assert_eq!(
        record.borrow_check_stage.as_ref().unwrap().status,
        RustBorrowCheckStageStatus::Succeeded
    );
    assert!(record.missing_stage.is_none());
    let mir_handoff = record.mir_handoff.as_ref().unwrap();
    assert_eq!(mir_handoff.stage, RustCompilerStage::Mir);
    assert_eq!(mir_handoff.source_path, "src/main.rs");
    assert_eq!(mir_handoff.status, RustMirHandoffStatus::AdapterUnavailable);
    assert_eq!(
        mir_handoff.blocker_category,
        Some(RustMirHandoffBlockerCategory::UpstreamCompilerPayloadNotEmbedded)
    );
    assert_eq!(
        mir_handoff.blocker_component.as_deref(),
        Some("mir_handoff_payload_adapter")
    );
    assert_eq!(
        mir_handoff.payload_adapter_status,
        "payload_exported_load_blocked"
    );
    assert!(mir_handoff.payload_adapter_bootstrap_typechecked);
    assert!(mir_handoff.payload_adapter_bootstrap_artifact_located);
    assert!(mir_handoff.payload_carrier_created);
    assert!(!mir_handoff.payload_loaded_into_rouwdi_facade);
    assert_eq!(
        mir_handoff.payload_carrier_state.as_deref(),
        Some("payload_exported_load_blocked")
    );
    let payload_carrier = mir_handoff.payload_carrier.as_ref().unwrap();
    assert_eq!(
        payload_carrier.artifact.as_ref().unwrap().artifact_format,
        "rlib"
    );
    assert_eq!(
        payload_carrier
            .metadata_artifact
            .as_ref()
            .unwrap()
            .artifact_format,
        "rmeta"
    );
    assert_eq!(
        payload_carrier.load_blocker_kind.as_deref(),
        Some("compiler_payload_bundle_inspected_rlib_archive_not_loadable")
    );
    assert!(mir_handoff.payload_bundle_inspected);
    assert_eq!(
        mir_handoff.payload_loader_exported_artifact_class,
        Some(rouwdi_rustc_upstream::CompilerPayloadArtifactClass::RlibArchive)
    );
    assert_eq!(
        mir_handoff.payload_loader_metadata_artifact_class,
        Some(rouwdi_rustc_upstream::CompilerPayloadArtifactClass::MetadataOnly)
    );
    assert_eq!(
        mir_handoff.payload_loader_load_strategy,
        Some(rouwdi_rustc_upstream::CompilerPayloadLoadStrategy::InspectRlibArchive)
    );
    assert_eq!(
        mir_handoff.payload_loader_loadability_status,
        Some(
            rouwdi_rustc_upstream::CompilerPayloadLoadabilityStatus::UnsupportedCompilerPrivateArtifact
        )
    );
    assert_eq!(
        mir_handoff.payload_next_required_artifact_format.as_deref(),
        Some("wasm_component_or_module_with_explicit_rouwdi_compiler_payload_abi")
    );
    assert_eq!(mir_handoff.payload_adapter_probe_exit_code, 0);
    assert_eq!(
        mir_handoff.payload_adapter_probe_classification,
        "bootstrap_adapter_typechecked"
    );
    assert_eq!(
        mir_handoff.payload_adapter_symbol,
        "rouwdi_rustc_upstream::mir_handoff_payload_adapter"
    );
    assert!(mir_handoff
        .required_upstream_crates
        .contains(&"rustc_mir_build".to_owned()));
    assert!(report.bootstrap_diagnostics.iter().any(|item| {
        item.component == "upstream MIR adapter mir_handoff_payload_adapter"
            && item.required_by == "compile unit app:rust:app:wasm32-wasip1"
            && item.reason.contains("bootstrap authoritative probe")
    }));
    assert!(!report
        .bootstrap_diagnostics
        .iter()
        .any(|item| item.component == "rustc frontend semantics"));
    let mir_handoff_records: Vec<RustMirHandoffRecord> = serde_json::from_slice(
        &storage
            .read(&format!(
                "{}/graph/rust-source-mir-handoff.json",
                report.run_root
            ))
            .unwrap(),
    )
    .unwrap();
    assert_eq!(mir_handoff_records.len(), 1);
    assert_eq!(
        mir_handoff_records[0].blocker_component.as_deref(),
        Some("mir_handoff_payload_adapter")
    );
    let artifact_pipeline: Vec<ArtifactPipelineRecord> = serde_json::from_slice(
        &storage
            .read(&format!("{}/graph/artifact-pipeline.json", report.run_root))
            .unwrap(),
    )
    .unwrap();
    assert_eq!(artifact_pipeline.len(), 1);
    assert_eq!(artifact_pipeline[0].target_name, "wasi");
    assert_eq!(
        artifact_pipeline[0].blocked_at_stage,
        Some(RustCompilerStage::Mir)
    );
    assert_eq!(
        artifact_pipeline[0].blocker_component.as_deref(),
        Some("mir_handoff_payload_adapter")
    );
    assert_eq!(
        artifact_pipeline[0].expected_output_path,
        format!("{}/artifacts/app-wasm32-wasip1.wasm", report.run_root)
    );
    assert!(!artifact_pipeline[0].artifact_emitted);
    assert!(artifact_pipeline[0].compile_units.iter().any(|unit| {
        unit.unit_id == "app:rust:app:wasm32-wasip1"
            && unit.source_path == "src/main.rs"
            && unit.mir_handoff_status == Some(RustMirHandoffStatus::AdapterUnavailable)
    }));
    assert!(artifact_pipeline[0].remaining_stages.iter().any(|stage| {
        stage.stage == RustCompilerStage::Mir
            && stage.required_component == "rustc_middle"
            && stage.status == ArtifactPipelineStageStatus::Blocked
    }));
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
        record.status == RustCompilerPipelineStatus::MirHandoffBlocked
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
            && record
                .borrow_check_stage
                .as_ref()
                .is_some_and(|borrow_check| {
                    borrow_check.status == RustBorrowCheckStageStatus::Succeeded
                        && borrow_check.stage == RustCompilerStage::BorrowChecking
                })
            && record.mir_handoff.as_ref().is_some_and(|handoff| {
                handoff.stage == RustCompilerStage::Mir
                    && handoff.status == RustMirHandoffStatus::AdapterUnavailable
                    && handoff.blocker_component.as_deref() == Some("mir_handoff_payload_adapter")
            })
            && record.missing_stage.is_none()
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
    let borrow_check_records: Vec<RustBorrowCheckStageRecord> = serde_json::from_slice(
        &storage
            .read(&format!(
                "{}/graph/rust-source-borrow-check.json",
                report.run_root
            ))
            .unwrap(),
    )
    .unwrap();
    let borrow_check_unit_ids = borrow_check_records
        .iter()
        .map(|record| record.unit_id.clone())
        .collect::<BTreeSet<_>>();
    assert_eq!(borrow_check_unit_ids, rust_unit_ids);
    assert!(borrow_check_records
        .iter()
        .all(|record| record.status == RustBorrowCheckStageStatus::Succeeded));
    let mir_handoff_records: Vec<RustMirHandoffRecord> = serde_json::from_slice(
        &storage
            .read(&format!(
                "{}/graph/rust-source-mir-handoff.json",
                report.run_root
            ))
            .unwrap(),
    )
    .unwrap();
    let mir_handoff_unit_ids = mir_handoff_records
        .iter()
        .map(|record| record.compile_unit.unit_id.clone())
        .collect::<BTreeSet<_>>();
    assert_eq!(mir_handoff_unit_ids, rust_unit_ids);
    assert!(mir_handoff_records.iter().all(|record| {
        record.status == RustMirHandoffStatus::AdapterUnavailable
            && record.blocker_component.as_deref() == Some("mir_handoff_payload_adapter")
            && record.payload_carrier_state.as_deref() == Some("payload_exported_load_blocked")
            && record.payload_adapter_bootstrap_artifact_located
            && record.payload_carrier_created
            && !record.payload_loaded_into_rouwdi_facade
            && record.payload_carrier.as_ref().is_some_and(|carrier| {
                carrier.artifact.as_ref().is_some_and(|artifact| {
                    artifact.artifact_format == "rlib" && !artifact.loadable_by_rouwdi_wasm
                }) && carrier.metadata_artifact.as_ref().is_some_and(|artifact| {
                    artifact.artifact_format == "rmeta" && !artifact.loadable_by_rouwdi_wasm
                })
            })
    }));
    let artifact_pipeline: Vec<ArtifactPipelineRecord> = serde_json::from_slice(
        &storage
            .read(&format!("{}/graph/artifact-pipeline.json", report.run_root))
            .unwrap(),
    )
    .unwrap();
    assert_eq!(artifact_pipeline.len(), 1);
    assert_eq!(artifact_pipeline[0].target_name, "wasi");
    let artifact_unit_ids = artifact_pipeline[0]
        .compile_units
        .iter()
        .map(|unit| unit.unit_id.clone())
        .collect::<BTreeSet<_>>();
    assert_eq!(artifact_unit_ids, rust_unit_ids);
    assert_eq!(
        artifact_pipeline[0].expected_output_path,
        format!("{}/artifacts/app-wasm32-wasip1.wasm", report.run_root)
    );
    assert!(!artifact_pipeline[0].artifact_emitted);
}

#[test]
fn artifact_pipeline_records_every_requested_target_without_emitting_fake_artifacts() {
    let mut storage = no_deps_wasi_binary_storage();
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

[[targets]]
name = "native"
triple = "native_host"
artifact = "executable"
"#,
        )
        .unwrap();

    let report = RouwdiEngine::default()
        .build(&mut storage, BuildRequest::default())
        .unwrap();
    let manifest: RouwdiRunManifest =
        serde_json::from_slice(&storage.read(&report.manifest_path).unwrap()).unwrap();
    let artifact_pipeline: Vec<ArtifactPipelineRecord> = serde_json::from_slice(
        &storage
            .read(&format!("{}/graph/artifact-pipeline.json", report.run_root))
            .unwrap(),
    )
    .unwrap();

    assert_eq!(report.status, RunStatus::Failed);
    assert_eq!(artifact_pipeline.len(), 2);
    assert_eq!(manifest.artifact_pipeline, artifact_pipeline);
    let targets = artifact_pipeline
        .iter()
        .map(|record| record.target_name.as_str())
        .collect::<BTreeSet<_>>();
    assert_eq!(targets, BTreeSet::from(["native", "wasi"]));
    for record in &artifact_pipeline {
        assert_eq!(record.blocked_at_stage, Some(RustCompilerStage::Mir));
        assert_eq!(
            record.blocker_component.as_deref(),
            Some("mir_handoff_payload_adapter")
        );
        assert!(!record.artifact_emitted);
        assert!(storage.read(&record.expected_output_path).is_err());
        assert!(record.remaining_stages.iter().any(|stage| {
            stage.stage == RustCompilerStage::Mir
                && stage.status == ArtifactPipelineStageStatus::Blocked
        }));
    }
    assert!(artifact_pipeline.iter().any(|record| {
        record.target_name == "wasi"
            && record.expected_output_path
                == format!("{}/artifacts/app-wasm32-wasip1.wasm", report.run_root)
    }));
    assert!(artifact_pipeline.iter().any(|record| {
        record.target_name == "native"
            && record.expected_output_path
                == format!("{}/artifacts/app-native_host", report.run_root)
    }));
    assert!(manifest.artifacts.is_empty());
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
        item.component == "compiler stage rustc_middle"
            || item.component == "upstream MIR adapter mir_handoff_payload_adapter"
            || item.reason.contains("mir_not_embedded")
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
        item.component == "compiler stage rustc_middle"
            || item.component == "upstream MIR adapter mir_handoff_payload_adapter"
            || item.reason.contains("mir_not_embedded")
    }));
}

#[test]
fn borrow_invalid_source_stops_at_borrow_check_stage() {
    let mut storage = no_deps_wasi_binary_storage();
    storage
        .write(
            "src/main.rs",
            b"fn main() { let r; { let x = 1; r = &x; } let _y = r; }\n",
        )
        .unwrap();

    let report = RouwdiEngine::default()
        .build(&mut storage, BuildRequest::default())
        .unwrap();
    let manifest: RouwdiRunManifest =
        serde_json::from_slice(&storage.read(&report.manifest_path).unwrap()).unwrap();
    let borrow_check_records: Vec<RustBorrowCheckStageRecord> = serde_json::from_slice(
        &storage
            .read(&format!(
                "{}/graph/rust-source-borrow-check.json",
                report.run_root
            ))
            .unwrap(),
    )
    .unwrap();

    assert_eq!(report.status, RunStatus::Failed);
    assert_eq!(manifest.compiler_pipeline.len(), 1);
    let record = &manifest.compiler_pipeline[0];
    assert_eq!(record.status, RustCompilerPipelineStatus::BorrowCheckError);
    assert!(record.missing_stage.is_none());
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
    assert_eq!(
        record.type_check_stage.as_ref().unwrap().status,
        RustTypeCheckStageStatus::Succeeded
    );
    let borrow_check = record.borrow_check_stage.as_ref().unwrap();
    assert_eq!(borrow_check.status, RustBorrowCheckStageStatus::Failed);
    assert!(borrow_check.diagnostics.iter().any(|diagnostic| {
        diagnostic.code == RustBorrowCheckDiagnosticCode::BorrowedLocalEscapesScope
            && diagnostic.reference_local.as_deref() == Some("r")
            && diagnostic.borrowed_local.as_deref() == Some("x")
    }));
    assert_eq!(borrow_check_records.len(), 1);
    assert_eq!(
        borrow_check_records[0].status,
        RustBorrowCheckStageStatus::Failed
    );
    assert!(report.bootstrap_diagnostics.iter().any(|item| {
        item.component == "Rust borrow-check stage"
            && item.required_by == "compile unit app:rust:app:wasm32-wasip1"
    }));
    assert!(!report.bootstrap_diagnostics.iter().any(|item| {
        item.component == "compiler stage rustc_middle"
            || item.component == "upstream MIR adapter mir_handoff_payload_adapter"
            || item.reason.contains("mir_not_embedded")
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
