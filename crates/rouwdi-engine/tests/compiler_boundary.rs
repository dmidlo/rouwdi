use rouwdi_cargo::{CargoBuildPlan, CompilePhase};
use rouwdi_engine::{BuildRequest, RouwdiEngine};
use rouwdi_proof::{
    ArtifactPipelineRecord, ArtifactPipelineStageStatus, RouwdiRunManifest, RunStatus,
};
use rouwdi_rustc::{
    RustBorrowCheckDiagnosticCode, RustBorrowCheckStageRecord, RustBorrowCheckStageStatus,
    RustCodegenHandoffRecord, RustCompilerPipelineStatus, RustCompilerStage,
    RustEmbeddedMirPayloadExecution, RustExpansionStageRecord, RustExpansionStageStatus,
    RustMirBodyProof, RustMirHandoffBlockerCategory, RustMirHandoffRecord, RustMirHandoffStatus,
    RustMonomorphizationProof, RustNameResolutionDiagnosticCode, RustNameResolutionStageRecord,
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

fn synthetic_embedded_mir_payload_execution() -> RustEmbeddedMirPayloadExecution {
    RustEmbeddedMirPayloadExecution {
        payload_identity: "rouwdi-mir-handoff-payload".to_owned(),
        registry_identity: "rouwdi-mir-handoff-payload".to_owned(),
        execution_source: "embedded_registry".to_owned(),
        external: false,
        opened_external_file: false,
        embedded: true,
        expected_sha256: "a".repeat(64),
        actual_sha256: "a".repeat(64),
        hash_verified: true,
        expected_size_bytes: 8,
        actual_size_bytes: 8,
        size_verified: true,
        wasm_magic_verified: true,
        module_instantiated: true,
        abi_v1_exports_verified: true,
        exports: vec![
            "memory".to_owned(),
            "rouwdi_compiler_payload_abi_v1_version".to_owned(),
            "rouwdi_compiler_payload_abi_v1_stage".to_owned(),
            "rouwdi_compiler_payload_abi_v1_descriptor_ptr".to_owned(),
            "rouwdi_compiler_payload_abi_v1_descriptor_len".to_owned(),
            "rouwdi_mir_handoff_payload_v1_valid_input_ptr".to_owned(),
            "rouwdi_mir_handoff_payload_v1_valid_input_len".to_owned(),
            "rouwdi_mir_handoff_payload_v1_result_area_ptr".to_owned(),
            "rouwdi_mir_handoff_payload_v1_execute".to_owned(),
            "rouwdi_mir_handoff_payload_v1_last_error_ptr".to_owned(),
            "rouwdi_mir_handoff_payload_v1_last_error_len".to_owned(),
        ],
        imports: Vec::new(),
        abi_version_called: true,
        abi_version: 1,
        stage_called: true,
        stage: 1,
        descriptor_called: true,
        descriptor_json: "{}".to_owned(),
        valid_input_called: true,
        valid_input_json: "{}".to_owned(),
        execute_called: true,
        execute_status: -1901,
        execute_trapped: true,
        execute_trap: Some("wasm `unreachable` instruction executed".to_owned()),
        output_bytes_read: false,
        output_json: None,
        error_bytes_read: true,
        error_json: Some(
            r#"{"blocker_kind":"lang_items_query_failed_before_mir_provider"}"#.to_owned(),
        ),
        input_contract_sha256: "b".repeat(64),
        output_contract_sha256: None,
        error_contract_sha256: Some("c".repeat(64)),
        execution_state: "embedded_payload_executed_blocked_at_mir_provider_requires_lang_items"
            .to_owned(),
        blocker_kind: Some("lang_items_query_failed_before_mir_provider".to_owned()),
        result_kind: "error".to_owned(),
    }
}

fn synthetic_embedded_mir_payload_success_execution() -> RustEmbeddedMirPayloadExecution {
    let output_json = r#"{"code":"mir_body_hash_emitted","kind":"context_attempt_succeeded","message":"real upstream MIR body observed and rustc_monomorphize contacted","blocker_kind":"none","blocker_component":"none","context_state":"mir_body_hash_emitted","compile_unit_id":"app:rust:app:wasm32-wasip1","package":"app","target":"wasi","target_kind":"bin","target_triple":"wasm32-wasip1","profile":"release","source_path":"src/main.rs","source_hash":"8fd8d3f8a05c9b7b","crate_name":"rouwdi_payload","crate_hash":"0123456789abcdef","item_path":"rouwdi_payload::main","local_def_id":"LocalDefId(0)","def_id":"DefId(0:3 ~ rouwdi_payload[0000]::main)","def_path_hash":"DefPathHash(0123456789abcdef)","mir_provider":"rustc_mir_build","mir_query":"rustc_middle::ty::TyCtxt::optimized_mir","mir_stage":"optimized","mir_body_identity":"def_id=DefId(0:3 ~ rouwdi_payload[0000]::main);phase=optimized;basic_blocks=1;locals=1;source_path=src/main.rs","mir_body_hash":"feedfacecafebeef","body_basic_block_count":1,"body_local_count":1,"body_statement_count":0,"provider_query":"rustc_middle::ty::TyCtxt::optimized_mir","upstream_crates":["core","alloc","std"],"payload_artifact_hash":"recorded-by-host-loader-sha256","payload_sha256":"recorded-by-host-loader-sha256","input_contract_sha256":"bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb","core_metadata_loaded":true,"alloc_metadata_loaded":true,"std_metadata_loaded":true,"mir_provider_invoked":true,"real_mir_body_observed":true,"rustc_monomorphize_imported":true,"rustc_monomorphize_invoked":true,"monomorphization_query":"rustc_middle::ty::TyCtxt::collect_and_partition_mono_items","monomorphization_status":"rustc_monomorphize_invoked_blocked_at_codegen_backend_not_ready","monomorphization_blocker_kind":"codegen_backend_not_ready","monomorphization_blocker_component":"rustc_monomorphize/rustc_middle::ty::TyCtxt::collect_and_partition_mono_items","monomorphization_blocker_reason":"synthetic test blocker after real MIR proof","mono_item_count":0,"codegen_unit_count":0,"mono_item_graph_hash":null,"fabricated_ast":false,"fabricated_hir":false,"fabricated_tyctx":false,"fabricated_providers":false,"fabricated_body":false,"fabricated_mir":false,"fabricated_mono_items":false}"#;
    RustEmbeddedMirPayloadExecution {
        execute_status: 0,
        execute_trapped: false,
        execute_trap: None,
        output_bytes_read: true,
        output_json: Some(output_json.to_owned()),
        error_bytes_read: false,
        error_json: None,
        output_contract_sha256: Some("d".repeat(64)),
        error_contract_sha256: None,
        execution_state: "embedded_payload_mir_body_hash_emitted".to_owned(),
        blocker_kind: Some("none".to_owned()),
        result_kind: "output".to_owned(),
        ..synthetic_embedded_mir_payload_execution()
    }
}

fn synthetic_embedded_mono_items_collected_execution() -> RustEmbeddedMirPayloadExecution {
    let mut execution = synthetic_embedded_mir_payload_success_execution();
    let output = execution
        .output_json
        .as_ref()
        .unwrap()
        .replace(
            r#""code":"mir_body_hash_emitted""#,
            r#""code":"mono_items_collected""#,
        )
        .replace(
            r#""context_state":"mir_body_hash_emitted""#,
            r#""context_state":"mono_items_collected""#,
        )
        .replace(
            r#""monomorphization_status":"rustc_monomorphize_invoked_blocked_at_codegen_backend_not_ready""#,
            r#""monomorphization_status":"mono_items_collected""#,
        )
        .replace(
            r#""monomorphization_blocker_kind":"codegen_backend_not_ready""#,
            r#""monomorphization_blocker_kind":"none""#,
        )
        .replace(
            r#""monomorphization_blocker_component":"rustc_monomorphize/rustc_middle::ty::TyCtxt::collect_and_partition_mono_items""#,
            r#""monomorphization_blocker_component":"none""#,
        )
        .replace(
            r#""monomorphization_blocker_reason":"synthetic test blocker after real MIR proof""#,
            r#""monomorphization_blocker_reason":"none""#,
        )
        .replace(
            r#""mono_item_count":0,"codegen_unit_count":0,"mono_item_graph_hash":null"#,
            r#""monomorphization_provider":"rustc_monomorphize::partitioning::collect_and_partition_mono_items","failed_query":"rustc_middle::ty::TyCtxt::collect_and_partition_mono_items","last_successful_compiler_step":"rustc_middle::ty::TyCtxt::optimized_mir","mono_item_count":1,"mono_items":[{"item_kind":"fn","symbol_name":"_RNvC8rouwdi_4main","instance_identity":"Instance { def: Item(DefId(0:3 ~ rouwdi_payload[0000]::main)), args: [] }","def_id":"DefId(0:3 ~ rouwdi_payload[0000]::main)","codegen_unit":"rouwdi_payload.abc123","linkage":"External","visibility":"Default","source":"rustc_middle::ty::TyCtxt::collect_and_partition_mono_items"}],"mono_items_derived_from":"rustc_middle::ty::TyCtxt::collect_and_partition_mono_items","partition_count":1,"codegen_unit_count":1,"mono_item_graph_hash":"0123456789abcdef""#,
        );
    execution.output_json = Some(output);
    execution.execution_state = "mono_items_collected".to_owned();
    execution
}

fn collected_mono_proof_and_codegen_handoff(
) -> (RustMonomorphizationProof, RustCodegenHandoffRecord) {
    let mut storage = no_deps_wasi_binary_storage();
    let execution = synthetic_embedded_mono_items_collected_execution();

    let report = RouwdiEngine::default()
        .with_embedded_mir_payload_execution(execution)
        .build(&mut storage, BuildRequest::default())
        .unwrap();
    let manifest: RouwdiRunManifest =
        serde_json::from_slice(&storage.read(&report.manifest_path).unwrap()).unwrap();
    let mir_handoff = manifest.compiler_pipeline[0].mir_handoff.as_ref().unwrap();
    (
        mir_handoff.monomorphization_proof.as_ref().unwrap().clone(),
        mir_handoff.codegen_handoff.as_ref().unwrap().clone(),
    )
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
        "payload_context_attempted"
    );
    assert!(mir_handoff.payload_adapter_bootstrap_typechecked);
    assert!(mir_handoff.payload_adapter_bootstrap_artifact_located);
    assert!(mir_handoff.payload_carrier_created);
    assert!(!mir_handoff.payload_loaded_into_rouwdi_facade);
    assert_eq!(
        mir_handoff.payload_carrier_state.as_deref(),
        Some("payload_context_attempted")
    );
    let payload_carrier = mir_handoff.payload_carrier.as_ref().unwrap();
    assert_eq!(
        payload_carrier.artifact.as_ref().unwrap().artifact_format,
        "wasm_module"
    );
    assert_eq!(
        payload_carrier
            .metadata_artifact
            .as_ref()
            .unwrap()
            .artifact_format,
        "rmeta"
    );
    assert_eq!(payload_carrier.load_blocker_kind.as_deref(), Some("none"));
    assert_eq!(
        mir_handoff.payload_milestone_state.as_deref(),
        Some("bridge_wasm_mir_payload_module_emitted")
    );
    let target_pack = mir_handoff.payload_target_pack.as_ref().unwrap();
    assert_eq!(target_pack.target_triple, "wasm32-wasip1");
    assert!(target_pack.attempted);
    assert_eq!(target_pack.status, "ready");
    assert_eq!(target_pack.exit_code, 0);
    assert_eq!(target_pack.blocker_kind, "none");
    assert!(target_pack.std_available);
    assert!(target_pack.core_available);
    assert!(target_pack.alloc_available);
    assert!(target_pack
        .produced_artifacts
        .iter()
        .any(|artifact| artifact.contains("libcore-") && artifact.ends_with(".rlib")));
    assert!(mir_handoff.payload_bundle_inspected);
    assert_eq!(
        mir_handoff.payload_abi_manifest_path.as_deref(),
        Some(rouwdi_rustc_upstream::COMPILER_PAYLOAD_ABI_MANIFEST_PATH)
    );
    assert_eq!(
        mir_handoff.payload_abi_selected_route.as_deref(),
        Some("wasm32_wasip1_module")
    );
    assert_eq!(
        mir_handoff.payload_abi_route_status,
        Some(rouwdi_rustc_upstream::CompilerPayloadAbiRouteStatus::Emitted)
    );
    assert_eq!(
        mir_handoff.payload_abi_route_artifact_path.as_deref(),
        Some(".rouwdi/direct-rustc-private-pack/target/wasm32-wasip1/release/rouwdi_mir_adapter_probe.wasm")
    );
    assert_eq!(mir_handoff.payload_abi_route_attempted, Some(true));
    assert_eq!(
        mir_handoff.payload_abi_bridge_blocker_kind.as_deref(),
        Some("none")
    );
    let bridge_attempt = mir_handoff.payload_bridge_attempt.as_ref().unwrap();
    assert_eq!(bridge_attempt.status, "mono_items_collected");
    assert_eq!(bridge_attempt.blocker_kind, "none");
    assert_eq!(bridge_attempt.command_exit_code, Some(0));
    assert!(bridge_attempt
        .input_artifact_identities
        .iter()
        .any(
            |artifact| artifact.role == "direct_rustc_private_root_rustc_span"
                && artifact.artifact_format == "rlib"
                && artifact.loadable_by_rouwdi_wasm
        ));
    assert!(bridge_attempt.output_artifact_identity.is_some());
    assert_eq!(
        mir_handoff.payload_loader_exported_artifact_class,
        Some(rouwdi_rustc_upstream::CompilerPayloadArtifactClass::WasmModule)
    );
    assert_eq!(
        mir_handoff.payload_loader_metadata_artifact_class,
        Some(rouwdi_rustc_upstream::CompilerPayloadArtifactClass::MetadataOnly)
    );
    assert_eq!(
        mir_handoff.payload_loader_load_strategy,
        Some(rouwdi_rustc_upstream::CompilerPayloadLoadStrategy::InstantiateWasmModule)
    );
    assert_eq!(
        mir_handoff.payload_loader_loadability_status,
        Some(rouwdi_rustc_upstream::CompilerPayloadLoadabilityStatus::Loadable)
    );
    assert_eq!(
        mir_handoff.payload_next_required_artifact_format.as_deref(),
        Some("codegen_handoff")
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
fn proof_bundle_records_embedded_mir_payload_execution() {
    let mut storage = no_deps_wasi_binary_storage();
    let execution = synthetic_embedded_mir_payload_execution();

    let report = RouwdiEngine::default()
        .with_embedded_mir_payload_execution(execution.clone())
        .build(&mut storage, BuildRequest::default())
        .unwrap();
    let manifest: RouwdiRunManifest =
        serde_json::from_slice(&storage.read(&report.manifest_path).unwrap()).unwrap();
    let proof_path = format!("{}/proofs/mir-handoff-payload.json", report.run_root);

    assert_eq!(report.status, RunStatus::Failed);
    assert!(manifest.proof_files.contains(&proof_path));
    let proof_records: Vec<RustEmbeddedMirPayloadExecution> =
        serde_json::from_slice(&storage.read(&proof_path).unwrap()).unwrap();
    assert_eq!(proof_records, vec![execution.clone()]);

    let mir_handoff_records: Vec<RustMirHandoffRecord> = serde_json::from_slice(
        &storage
            .read(&format!(
                "{}/graph/rust-source-mir-handoff.json",
                report.run_root
            ))
            .unwrap(),
    )
    .unwrap();
    let embedded_execution = mir_handoff_records[0]
        .embedded_payload_execution
        .as_ref()
        .unwrap();
    assert_eq!(embedded_execution.execution_source, "embedded_registry");
    assert!(embedded_execution.embedded);
    assert!(embedded_execution.module_instantiated);
    assert!(embedded_execution.abi_v1_exports_verified);
    assert!(embedded_execution.execute_called);
    assert!(embedded_execution.error_bytes_read);
    assert!(!embedded_execution.opened_external_file);
    assert_eq!(embedded_execution.result_kind, "error");
    assert_eq!(
        embedded_execution.execution_state,
        "embedded_payload_executed_blocked_at_mir_provider_requires_lang_items"
    );
}

#[test]
fn embedded_mir_body_output_becomes_mir_stage_success_and_monomorphization_frontier() {
    let mut storage = no_deps_wasi_binary_storage();
    let execution = synthetic_embedded_mir_payload_success_execution();

    let report = RouwdiEngine::default()
        .with_embedded_mir_payload_execution(execution.clone())
        .build(&mut storage, BuildRequest::default())
        .unwrap();
    let manifest: RouwdiRunManifest =
        serde_json::from_slice(&storage.read(&report.manifest_path).unwrap()).unwrap();
    let record = &manifest.compiler_pipeline[0];
    let mir_handoff = record.mir_handoff.as_ref().unwrap();

    assert_eq!(report.status, RunStatus::Failed);
    assert_eq!(record.status, RustCompilerPipelineStatus::MissingStage);
    assert_eq!(
        record.missing_stage.as_ref().unwrap().stage,
        RustCompilerStage::Monomorphization
    );
    assert_eq!(mir_handoff.status, RustMirHandoffStatus::AdapterAvailable);
    assert!(mir_handoff.upstream_mir_adapter_available);
    assert!(mir_handoff.blocker_category.is_none());
    assert!(
        !mir_handoff
            .embedded_payload_execution
            .as_ref()
            .unwrap()
            .opened_external_file
    );
    let mir_body_proof = mir_handoff.mir_body_proof.as_ref().unwrap();
    assert_eq!(
        mir_body_proof.provider_query,
        "rustc_middle::ty::TyCtxt::optimized_mir"
    );
    assert_eq!(mir_body_proof.compile_unit_id, "app:rust:app:wasm32-wasip1");
    assert_eq!(mir_body_proof.mir_body_hash, "feedfacecafebeef");
    assert!(mir_body_proof.core_metadata_loaded);
    assert!(mir_body_proof.lang_items_resolved);
    assert!(mir_body_proof.mir_provider_invoked);
    assert!(!mir_body_proof.fabricated_mir);
    assert_eq!(
        mir_handoff
            .monomorphization_handoff
            .as_ref()
            .unwrap()
            .required_upstream_component,
        "rustc_monomorphize"
    );

    let artifact_pipeline: Vec<ArtifactPipelineRecord> = serde_json::from_slice(
        &storage
            .read(&format!("{}/graph/artifact-pipeline.json", report.run_root))
            .unwrap(),
    )
    .unwrap();
    assert_eq!(
        artifact_pipeline[0].blocked_at_stage,
        Some(RustCompilerStage::Monomorphization)
    );
    assert_eq!(
        artifact_pipeline[0]
            .blocker_component
            .as_deref()
            .is_some_and(|component| component.contains("rustc_monomorphize")),
        true
    );
    assert!(artifact_pipeline[0].compile_units.iter().any(|unit| {
        unit.mir_handoff_status == Some(RustMirHandoffStatus::AdapterAvailable)
            && unit.mir_body_hash.as_deref() == Some("feedfacecafebeef")
            && unit.monomorphization_handoff_status.as_deref()
                == Some("rustc_monomorphize_invoked_blocked_at_codegen_backend_not_ready")
    }));
    assert!(artifact_pipeline[0].remaining_stages.iter().any(|stage| {
        stage.stage == RustCompilerStage::Mir
            && stage.status == ArtifactPipelineStageStatus::Completed
    }));
    assert!(artifact_pipeline[0].remaining_stages.iter().any(|stage| {
        stage.stage == RustCompilerStage::Monomorphization
            && stage.status == ArtifactPipelineStageStatus::Blocked
    }));

    let mir_body_path = format!("{}/proofs/mir-body.json", report.run_root);
    assert!(manifest.proof_files.contains(&mir_body_path));
    let mir_body_records: Vec<RustMirBodyProof> =
        serde_json::from_slice(&storage.read(&mir_body_path).unwrap()).unwrap();
    assert_eq!(mir_body_records, vec![mir_body_proof.clone()]);
}

#[test]
fn mono_item_graph_success_writes_mono_proof_and_opens_codegen_handoff() {
    let mut storage = no_deps_wasi_binary_storage();
    let execution = synthetic_embedded_mono_items_collected_execution();

    let report = RouwdiEngine::default()
        .with_embedded_mir_payload_execution(execution)
        .build(&mut storage, BuildRequest::default())
        .unwrap();
    let manifest: RouwdiRunManifest =
        serde_json::from_slice(&storage.read(&report.manifest_path).unwrap()).unwrap();
    let mir_handoff = manifest.compiler_pipeline[0].mir_handoff.as_ref().unwrap();
    let mono_proof = mir_handoff.monomorphization_proof.as_ref().unwrap();

    assert_eq!(mono_proof.status, "mono_items_collected");
    assert_eq!(mono_proof.mono_item_count, 1);
    assert_eq!(
        mono_proof.mono_query,
        "rustc_middle::ty::TyCtxt::collect_and_partition_mono_items"
    );
    assert_eq!(
        mono_proof.mono_item_graph_hash.as_deref(),
        Some("0123456789abcdef")
    );
    assert!(mono_proof
        .mono_items
        .iter()
        .all(|item| item.source == mono_proof.mono_query));
    let codegen_handoff = mir_handoff.codegen_handoff.as_ref().unwrap();
    assert_eq!(
        codegen_handoff.required_upstream_component,
        "rustc_codegen_llvm"
    );
    assert_eq!(codegen_handoff.package, "app");
    assert_eq!(codegen_handoff.target, "wasi");
    assert_eq!(codegen_handoff.target_kind, "bin");
    assert_eq!(codegen_handoff.profile, "release");
    assert_eq!(codegen_handoff.source_path, "src/main.rs");
    assert_eq!(
        codegen_handoff.mir_body_identity,
        mono_proof.mir_body_identity
    );
    assert_eq!(codegen_handoff.mono_provider, mono_proof.mono_provider);
    assert_eq!(codegen_handoff.mono_query, mono_proof.mono_query);
    assert_eq!(codegen_handoff.backend_family, "llvm-grade");
    assert_eq!(codegen_handoff.expected_output_kind, "wasm_object");
    assert_eq!(
        codegen_handoff.required_target_machine,
        "LLVM TargetMachine for wasm32-wasip1"
    );
    assert_eq!(
        codegen_handoff.required_target_spec,
        "rustc_target target spec for wasm32-wasip1"
    );
    assert_eq!(codegen_handoff.required_relocation_model, "pic");
    assert!(codegen_handoff
        .required_upstream_crates
        .contains(&"rustc_codegen_ssa".to_owned()));
    assert!(codegen_handoff
        .upstream_component_identities
        .iter()
        .any(|identity| identity.contains("LlvmCodegenBackend")));
    assert!(codegen_handoff.backend_contact_attempted);
    assert!(codegen_handoff
        .codegen_backend_entrypoint
        .contains("rustc_codegen_llvm::LlvmCodegenBackend"));
    assert_eq!(codegen_handoff.target_loadable_probe_exit_code, 0);
    assert_eq!(
        codegen_handoff.target_loadable_status,
        "rustc_codegen_llvm_target_loadable_check_only"
    );
    assert_eq!(
        codegen_handoff.target_loadable_check_only_status,
        "rustc_codegen_llvm_target_loadable_check_only"
    );
    assert!(codegen_handoff
        .required_dependency_components
        .contains(&"rustc_codegen_ssa".to_owned()));
    assert!(codegen_handoff
        .required_dependency_components
        .contains(&"LLVM wrapper/C++ layer".to_owned()));
    assert!(codegen_handoff
        .expected_output_kinds
        .contains(&"LLVM module".to_owned()));
    assert!(codegen_handoff
        .expected_output_kinds
        .contains(&"wasm object".to_owned()));
    assert!(codegen_handoff
        .codegen_contact_points
        .iter()
        .any(|point| point.contains("LlvmCodegenBackend::new")));
    assert_eq!(
        codegen_handoff.codegen_contact_state,
        "rustc_codegen_llvm_backend_payload_blocked_at_target_llvm_library_closure"
    );
    assert_eq!(
        codegen_handoff.host_probe_codegen_contact_state,
        "target_machine_created"
    );
    assert!(codegen_handoff.host_probe_llvm_context_created);
    assert!(codegen_handoff.host_probe_llvm_module_created);
    assert!(codegen_handoff.host_probe_target_machine_created);
    assert!(codegen_handoff.mono_proof_consumed);
    assert!(!codegen_handoff.llvm_module_setup_invoked);
    assert!(!codegen_handoff.llvm_context_created);
    assert!(!codegen_handoff.llvm_module_created);
    assert!(codegen_handoff.llvm_module_identity.is_none());
    assert!(codegen_handoff.llvm_module_identity_hash.is_none());
    assert!(codegen_handoff.llvm_module_target_triple.is_none());
    assert!(!codegen_handoff.target_machine_setup_invoked);
    assert!(!codegen_handoff.target_machine_created);
    assert!(codegen_handoff.target_machine_cpu.is_empty());
    assert!(codegen_handoff.target_machine_relocation_model.is_empty());
    assert_eq!(
        codegen_handoff.backend_payload_kind,
        "codegen_backend_payload"
    );
    assert_eq!(
        codegen_handoff.backend_payload_blocker_kind,
        "wasm_codegen_payload_blocked_at_target_llvm_library_closure"
    );
    assert!(!codegen_handoff.backend_payload_embedded_in_assembly);
    assert_eq!(
        codegen_handoff.current_status,
        "rustc_codegen_llvm_backend_payload_blocked_at_target_llvm_library_closure"
    );
    assert_eq!(
        codegen_handoff.blocker_kind,
        "wasm_codegen_payload_blocked_at_target_llvm_library_closure"
    );
    assert_eq!(
        codegen_handoff.blocker_component,
        "rustc_codegen_llvm target LLVM library closure"
    );
    assert!(codegen_handoff.blocker_reason.contains("host evidence"));
    assert!(codegen_handoff
        .blocker_reason
        .contains("target-compatible LLVM library closure"));
    assert!(codegen_handoff
        .blocker_reason
        .contains("host evidence only"));
    assert!(!codegen_handoff.object_emission_attempted);
    assert!(!codegen_handoff.object_bytes_emitted);
    assert!(codegen_handoff.object_sha256.is_none());
    assert!(!codegen_handoff.llvm_ir_emitted);
    assert!(!codegen_handoff.linker_handoff_created);
    assert!(codegen_handoff.linker_handoff.is_none());
    codegen_handoff
        .validate_against_monomorphization_proof(mono_proof)
        .unwrap();
    assert_eq!(
        manifest.compiler_pipeline[0]
            .missing_stage
            .as_ref()
            .unwrap()
            .stage,
        RustCompilerStage::Codegen
    );

    let mono_path = format!("{}/proofs/monomorphization.json", report.run_root);
    assert!(manifest.proof_files.contains(&mono_path));
    let mono_records: Vec<RustMonomorphizationProof> =
        serde_json::from_slice(&storage.read(&mono_path).unwrap()).unwrap();
    assert_eq!(mono_records, vec![mono_proof.clone()]);
    let codegen_path = format!("{}/graph/rust-source-codegen-handoff.json", report.run_root);
    assert!(manifest.proof_files.contains(&codegen_path));
    assert_eq!(
        manifest.artifact_pipeline[0].blocked_at_stage,
        Some(RustCompilerStage::Codegen)
    );
    assert_eq!(
        manifest.artifact_pipeline[0].blocker_component.as_deref(),
        Some("rustc_codegen_llvm target LLVM library closure")
    );
    assert!(manifest.artifact_pipeline[0]
        .remaining_stages
        .iter()
        .any(|stage| stage.stage == RustCompilerStage::Codegen
            && stage.status == ArtifactPipelineStageStatus::Blocked));
    assert!(manifest.artifact_pipeline[0]
        .remaining_stages
        .iter()
        .any(|stage| stage.stage == RustCompilerStage::Monomorphization
            && stage.status == ArtifactPipelineStageStatus::Completed));
    assert!(manifest.artifact_pipeline[0]
        .compile_units
        .iter()
        .any(|unit| {
            unit.mono_item_count == Some(1)
                && unit.mono_item_graph_hash.as_deref() == Some("0123456789abcdef")
                && unit.codegen_handoff_status.as_deref()
                    == Some(
                        "rustc_codegen_llvm_backend_payload_blocked_at_target_llvm_library_closure",
                    )
        }));
}

#[test]
fn mono_success_without_count_or_graph_does_not_open_codegen() {
    let mut storage = no_deps_wasi_binary_storage();
    let mut execution = synthetic_embedded_mono_items_collected_execution();
    let output = execution
        .output_json
        .as_ref()
        .unwrap()
        .replace(r#""mono_item_count":1"#, r#""mono_item_count":0"#)
        .replace(
            r#""mono_item_graph_hash":"0123456789abcdef""#,
            r#""mono_item_graph_hash":null"#,
        );
    execution.output_json = Some(output);

    let report = RouwdiEngine::default()
        .with_embedded_mir_payload_execution(execution)
        .build(&mut storage, BuildRequest::default())
        .unwrap();
    let manifest: RouwdiRunManifest =
        serde_json::from_slice(&storage.read(&report.manifest_path).unwrap()).unwrap();
    let mir_handoff = manifest.compiler_pipeline[0].mir_handoff.as_ref().unwrap();

    assert!(mir_handoff.monomorphization_proof.is_none());
    assert!(mir_handoff.codegen_handoff.is_none());
    assert_eq!(
        manifest.compiler_pipeline[0]
            .missing_stage
            .as_ref()
            .unwrap()
            .stage,
        RustCompilerStage::Monomorphization
    );
}

#[test]
fn mono_success_from_non_upstream_derivation_is_rejected() {
    let mut storage = no_deps_wasi_binary_storage();
    let mut execution = synthetic_embedded_mono_items_collected_execution();
    let output = execution.output_json.as_ref().unwrap().replace(
        r#""mono_items_derived_from":"rustc_middle::ty::TyCtxt::collect_and_partition_mono_items""#,
        r#""mono_items_derived_from":"hir_traversal_local_function_list""#,
    );
    execution.output_json = Some(output);

    let report = RouwdiEngine::default()
        .with_embedded_mir_payload_execution(execution)
        .build(&mut storage, BuildRequest::default())
        .unwrap();
    let manifest: RouwdiRunManifest =
        serde_json::from_slice(&storage.read(&report.manifest_path).unwrap()).unwrap();
    let mir_handoff = manifest.compiler_pipeline[0].mir_handoff.as_ref().unwrap();

    assert!(mir_handoff.monomorphization_proof.is_none());
    assert!(mir_handoff.codegen_handoff.is_none());
    assert_eq!(
        manifest.artifact_pipeline[0].blocked_at_stage,
        Some(RustCompilerStage::Monomorphization)
    );
}

#[test]
fn codegen_handoff_requires_monomorphization_proof() {
    let mut storage = no_deps_wasi_binary_storage();
    let mut execution = synthetic_embedded_mono_items_collected_execution();
    let output = execution
        .output_json
        .as_ref()
        .unwrap()
        .replace(r#""mono_items":[{"#, r#""mono_items_bypassed":[{"#);
    execution.output_json = Some(output);

    let report = RouwdiEngine::default()
        .with_embedded_mir_payload_execution(execution)
        .build(&mut storage, BuildRequest::default())
        .unwrap();
    let manifest: RouwdiRunManifest =
        serde_json::from_slice(&storage.read(&report.manifest_path).unwrap()).unwrap();
    let mir_handoff = manifest.compiler_pipeline[0].mir_handoff.as_ref().unwrap();

    assert!(mir_handoff.monomorphization_proof.is_none());
    assert!(mir_handoff.codegen_handoff.is_none());
    assert!(!manifest
        .proof_files
        .iter()
        .any(|path| path.ends_with("rust-source-codegen-handoff.json")));
}

#[test]
fn codegen_success_without_real_backend_contact_is_rejected() {
    let (mono_proof, mut codegen_handoff) = collected_mono_proof_and_codegen_handoff();

    codegen_handoff.backend_contact_attempted = false;
    codegen_handoff.current_status = "object_bytes_emitted".to_owned();
    codegen_handoff.object_bytes_emitted = true;
    codegen_handoff.object_path = Some("run/artifacts/app.o".to_owned());
    codegen_handoff.object_sha256 = Some("f".repeat(64));

    assert!(codegen_handoff
        .validate_against_monomorphization_proof(&mono_proof)
        .unwrap_err()
        .contains("no real rustc_codegen_llvm backend contact"));
}

#[test]
fn check_only_target_loadability_is_not_codegen_execution() {
    let (mono_proof, mut codegen_handoff) = collected_mono_proof_and_codegen_handoff();

    codegen_handoff.codegen_contact_state =
        "rustc_codegen_llvm_target_loadable_check_only".to_owned();

    assert!(codegen_handoff
        .validate_against_monomorphization_proof(&mono_proof)
        .unwrap_err()
        .contains("check-only target loadability is not backend execution"));
}

#[test]
fn codegen_attempt_without_mono_proof_consumption_is_rejected() {
    let (mono_proof, mut codegen_handoff) = collected_mono_proof_and_codegen_handoff();

    codegen_handoff.mono_proof_consumed = false;

    assert!(codegen_handoff
        .validate_against_monomorphization_proof(&mono_proof)
        .unwrap_err()
        .contains("consume the mono proof"));
}

#[test]
fn cranelift_primary_backend_is_rejected() {
    let (mono_proof, mut codegen_handoff) = collected_mono_proof_and_codegen_handoff();

    codegen_handoff.backend_family = "cranelift".to_owned();

    assert!(codegen_handoff
        .validate_against_monomorphization_proof(&mono_proof)
        .unwrap_err()
        .contains("llvm-grade"));
}

#[test]
fn fake_object_bytes_and_reused_mono_hash_are_rejected() {
    let (mono_proof, mut codegen_handoff) = collected_mono_proof_and_codegen_handoff();

    codegen_handoff.current_status = "wasm_object_bytes_emitted".to_owned();
    codegen_handoff.object_bytes_emitted = true;
    codegen_handoff.object_path = Some("run/artifacts/app.wasm.o".to_owned());
    codegen_handoff.object_sha256 = Some(codegen_handoff.mono_item_graph_hash.clone());
    codegen_handoff.codegen_artifact_byte_len = Some(128);

    assert!(codegen_handoff
        .validate_against_monomorphization_proof(&mono_proof)
        .unwrap_err()
        .contains("object hash must not reuse mono graph"));
}

#[test]
fn fake_llvm_ir_reusing_mir_hash_is_rejected() {
    let (mono_proof, mut codegen_handoff) = collected_mono_proof_and_codegen_handoff();

    codegen_handoff.llvm_ir_emitted = true;
    codegen_handoff.llvm_ir_sha256 = Some(codegen_handoff.mir_body_hash.clone());
    codegen_handoff.codegen_artifact_byte_len = Some(128);

    assert!(codegen_handoff
        .validate_against_monomorphization_proof(&mono_proof)
        .unwrap_err()
        .contains("LLVM IR hash must not reuse"));
}

#[test]
fn linker_handoff_without_codegen_bytes_is_rejected() {
    let (mono_proof, mut codegen_handoff) = collected_mono_proof_and_codegen_handoff();

    codegen_handoff.linker_handoff_created = true;

    assert!(codegen_handoff
        .validate_against_monomorphization_proof(&mono_proof)
        .unwrap_err()
        .contains("linker handoff is forbidden"));
}

#[test]
fn rustc_codegen_llvm_is_named_and_attempted_in_import_ledger() {
    let component = rouwdi_rustc_upstream::import_component("rustc_codegen_llvm")
        .expect("rustc_codegen_llvm must be in upstream import ledger");

    assert!(component.attempted);
    assert_eq!(
        component.source_path,
        "third_party/rust/compiler/rustc_codegen_llvm"
    );
    assert_eq!(
        component.blocker_kind,
        "wasm_codegen_payload_blocked_at_target_llvm_library_closure"
    );
    assert!(component
        .probe_command
        .contains("compiler/rustc_codegen_llvm"));
    assert!(component
        .exact_blocker
        .contains("target-compatible LLVM library closure"));
    assert!(component.exact_blocker.contains("llvm-config.exe"));
    assert!(component.exact_blocker.contains("LLVM context/module"));
    assert!(component.exact_blocker.contains("target machine"));
    assert!(component.exact_blocker.contains("No object"));
    assert!(component
        .adapter_evidence
        .as_deref()
        .is_some_and(|evidence| evidence.contains("LlvmCodegenBackend::new")));
}

#[test]
fn fake_mir_identity_without_payload_output_is_rejected() {
    let mut storage = no_deps_wasi_binary_storage();
    let mut execution = synthetic_embedded_mir_payload_execution();
    execution.execution_state = "embedded_payload_mir_body_identity_emitted".to_owned();
    execution.blocker_kind = Some("none".to_owned());

    let report = RouwdiEngine::default()
        .with_embedded_mir_payload_execution(execution)
        .build(&mut storage, BuildRequest::default())
        .unwrap();
    let manifest: RouwdiRunManifest =
        serde_json::from_slice(&storage.read(&report.manifest_path).unwrap()).unwrap();
    let mir_handoff = manifest.compiler_pipeline[0].mir_handoff.as_ref().unwrap();

    assert_eq!(mir_handoff.status, RustMirHandoffStatus::AdapterUnavailable);
    assert!(mir_handoff.mir_body_proof.is_none());
    assert_eq!(
        manifest.artifact_pipeline[0].blocked_at_stage,
        Some(RustCompilerStage::Mir)
    );
}

#[test]
fn external_payload_path_cannot_satisfy_canonical_mir_success() {
    let mut storage = no_deps_wasi_binary_storage();
    let mut execution = synthetic_embedded_mir_payload_success_execution();
    execution.execution_source = "external_path".to_owned();
    execution.external = true;
    execution.opened_external_file = true;

    let report = RouwdiEngine::default()
        .with_embedded_mir_payload_execution(execution)
        .build(&mut storage, BuildRequest::default())
        .unwrap();
    let manifest: RouwdiRunManifest =
        serde_json::from_slice(&storage.read(&report.manifest_path).unwrap()).unwrap();
    let mir_handoff = manifest.compiler_pipeline[0].mir_handoff.as_ref().unwrap();

    assert_eq!(mir_handoff.status, RustMirHandoffStatus::AdapterUnavailable);
    assert!(mir_handoff.mir_body_proof.is_none());
    assert_eq!(
        manifest.artifact_pipeline[0].blocked_at_stage,
        Some(RustCompilerStage::Mir)
    );
}

#[test]
fn metadata_only_payload_output_cannot_satisfy_mir_success() {
    let mut storage = no_deps_wasi_binary_storage();
    let mut execution = synthetic_embedded_mir_payload_success_execution();
    execution.output_json = Some(
        r#"{"code":"mir_body_identity_emitted","kind":"metadata","context_state":"mir_body_identity_emitted","mir_provider_invoked":false,"fabricated_mir":false}"#
            .to_owned(),
    );

    let report = RouwdiEngine::default()
        .with_embedded_mir_payload_execution(execution)
        .build(&mut storage, BuildRequest::default())
        .unwrap();
    let manifest: RouwdiRunManifest =
        serde_json::from_slice(&storage.read(&report.manifest_path).unwrap()).unwrap();
    let mir_handoff = manifest.compiler_pipeline[0].mir_handoff.as_ref().unwrap();

    assert_eq!(mir_handoff.status, RustMirHandoffStatus::AdapterUnavailable);
    assert!(mir_handoff.mir_body_proof.is_none());
    assert_eq!(
        manifest.artifact_pipeline[0].blocked_at_stage,
        Some(RustCompilerStage::Mir)
    );
}

#[test]
fn mir_success_without_body_hash_is_rejected() {
    let mut storage = no_deps_wasi_binary_storage();
    let mut execution = synthetic_embedded_mir_payload_success_execution();
    let output = execution
        .output_json
        .as_ref()
        .unwrap()
        .replace(r#","mir_body_hash":"feedfacecafebeef""#, "");
    execution.output_json = Some(output);

    let report = RouwdiEngine::default()
        .with_embedded_mir_payload_execution(execution)
        .build(&mut storage, BuildRequest::default())
        .unwrap();
    let manifest: RouwdiRunManifest =
        serde_json::from_slice(&storage.read(&report.manifest_path).unwrap()).unwrap();
    let mir_handoff = manifest.compiler_pipeline[0].mir_handoff.as_ref().unwrap();

    assert_eq!(mir_handoff.status, RustMirHandoffStatus::AdapterUnavailable);
    assert!(mir_handoff.mir_body_proof.is_none());
    assert!(mir_handoff.monomorphization_handoff.is_none());
    assert_eq!(
        manifest.artifact_pipeline[0].blocked_at_stage,
        Some(RustCompilerStage::Mir)
    );
}

#[test]
fn stale_pre_mir_provider_text_rejects_mir_success() {
    let mut storage = no_deps_wasi_binary_storage();
    let mut execution = synthetic_embedded_mir_payload_success_execution();
    let output = execution.output_json.as_ref().unwrap().replace(
        "real upstream MIR body observed and rustc_monomorphize contacted",
        "stops before MIR provider invocation",
    );
    execution.output_json = Some(output);

    let report = RouwdiEngine::default()
        .with_embedded_mir_payload_execution(execution)
        .build(&mut storage, BuildRequest::default())
        .unwrap();
    let manifest: RouwdiRunManifest =
        serde_json::from_slice(&storage.read(&report.manifest_path).unwrap()).unwrap();
    let mir_handoff = manifest.compiler_pipeline[0].mir_handoff.as_ref().unwrap();

    assert_eq!(mir_handoff.status, RustMirHandoffStatus::AdapterUnavailable);
    assert!(mir_handoff.mir_body_proof.is_none());
    assert!(mir_handoff.monomorphization_handoff.is_none());
    assert_eq!(
        manifest.artifact_pipeline[0].blocked_at_stage,
        Some(RustCompilerStage::Mir)
    );
}

#[test]
fn blocker_none_with_blocker_reason_rejects_mir_success() {
    let mut storage = no_deps_wasi_binary_storage();
    let mut execution = synthetic_embedded_mir_payload_success_execution();
    let output = execution.output_json.as_ref().unwrap().replace(
        r#""blocker_component":"none""#,
        r#""blocker_component":"none","blocker_reason":"still blocked""#,
    );
    execution.output_json = Some(output);

    let report = RouwdiEngine::default()
        .with_embedded_mir_payload_execution(execution)
        .build(&mut storage, BuildRequest::default())
        .unwrap();
    let manifest: RouwdiRunManifest =
        serde_json::from_slice(&storage.read(&report.manifest_path).unwrap()).unwrap();
    let mir_handoff = manifest.compiler_pipeline[0].mir_handoff.as_ref().unwrap();

    assert_eq!(mir_handoff.status, RustMirHandoffStatus::AdapterUnavailable);
    assert!(mir_handoff.mir_body_proof.is_none());
    assert!(mir_handoff.monomorphization_handoff.is_none());
    assert_eq!(
        manifest.artifact_pipeline[0].blocked_at_stage,
        Some(RustCompilerStage::Mir)
    );
}

#[test]
fn fabricated_mono_items_reject_monomorphization_handoff() {
    let mut storage = no_deps_wasi_binary_storage();
    let mut execution = synthetic_embedded_mir_payload_success_execution();
    let output = execution.output_json.as_ref().unwrap().replace(
        r#""fabricated_mono_items":false"#,
        r#""fabricated_mono_items":true"#,
    );
    execution.output_json = Some(output);

    let report = RouwdiEngine::default()
        .with_embedded_mir_payload_execution(execution)
        .build(&mut storage, BuildRequest::default())
        .unwrap();
    let manifest: RouwdiRunManifest =
        serde_json::from_slice(&storage.read(&report.manifest_path).unwrap()).unwrap();
    let mir_handoff = manifest.compiler_pipeline[0].mir_handoff.as_ref().unwrap();

    assert_eq!(mir_handoff.status, RustMirHandoffStatus::AdapterUnavailable);
    assert!(mir_handoff.mir_body_proof.is_none());
    assert!(mir_handoff.monomorphization_handoff.is_none());
    assert_eq!(
        manifest.artifact_pipeline[0].blocked_at_stage,
        Some(RustCompilerStage::Mir)
    );
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
            && record.payload_carrier_state.as_deref() == Some("payload_context_attempted")
            && record.payload_adapter_bootstrap_artifact_located
            && record.payload_carrier_created
            && !record.payload_loaded_into_rouwdi_facade
            && record.payload_carrier.as_ref().is_some_and(|carrier| {
                carrier.artifact.as_ref().is_some_and(|artifact| {
                    artifact.artifact_format == "wasm_module" && artifact.loadable_by_rouwdi_wasm
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
