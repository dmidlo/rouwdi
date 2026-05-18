use serde_json::Value;
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};

const MIN_PRODUCT_SIZE_BYTES: u64 = 1_048_576;
const EXTERNAL_ONLY_STATES: &[&str] =
    &["metadata_reference_only", "external_hash_verified_payload"];
const PROBE_ONLY_CODEGEN_BLOCKERS: &[&str] = &[
    "codegen_lowering_blocked_at_codegen_lowering_to_object_not_implemented",
    "codegen_lowering_blocked_at_rustc_codegen_ssa_base_codegen_crate_requires_live_tyctxt_and_codegen_unit",
];

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("rouwdi-wasm is under workspace/crates/rouwdi-wasm")
        .to_path_buf()
}

fn sha256_hex(path: &Path) -> String {
    let bytes = fs::read(path).expect("artifact must be readable");
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}

fn canonical_manifest_policy_error(manifest: &Value) -> Option<String> {
    if manifest["package_mode"] != Value::String("canonical_single_file".to_owned()) {
        return Some("package_mode must be canonical_single_file".to_owned());
    }
    if manifest["single_file_product"] != Value::Bool(true) {
        return Some("single_file_product must be true".to_owned());
    }
    if manifest["canonical_artifact_path"] != Value::String("dist/rouwdi.wasm".to_owned()) {
        return Some("canonical_artifact_path must point at dist/rouwdi.wasm".to_owned());
    }

    let payload = &manifest["mir_payload"];
    if payload["embedded"] != Value::Bool(true) {
        return Some("MIR payload must be embedded".to_owned());
    }
    if payload["external"] == Value::Bool(true) {
        return Some("canonical package must not require external MIR payload".to_owned());
    }
    let state = payload["state"].as_str().unwrap_or_default();
    if EXTERNAL_ONLY_STATES.contains(&state) {
        return Some(format!("external-only MIR payload state rejected: {state}"));
    }
    if !matches!(
        state,
        "embedded_payload_executed_blocked_at_mir_provider_requires_lang_items"
            | "embedded_payload_mir_body_hash_emitted"
            | "embedded_payload_mono_items_collected"
    ) {
        return Some(format!("unexpected canonical MIR payload state: {state}"));
    }
    if payload["mir_body_identity_emitted"] == Value::Bool(true)
        && payload["mir_body_hash"]
            .as_str()
            .is_none_or(|hash| hash.is_empty())
    {
        return Some("MIR success must include mir_body_hash".to_owned());
    }
    if payload["mir_body_identity_emitted"] == Value::Bool(true)
        && payload["rustc_monomorphize_invoked"] != Value::Bool(true)
    {
        return Some("MIR success must attempt rustc_monomorphize".to_owned());
    }
    if payload["fabricated_mono_items"] == Value::Bool(true) {
        return Some("fabricated mono items are rejected".to_owned());
    }
    if payload["linker_handoff_created"] == Value::Bool(true)
        && payload["rust_mono_item_wasm_object_emitted"] != Value::Bool(true)
    {
        return Some("linker handoff requires mono-item-derived Wasm object".to_owned());
    }
    if payload["instantiated"] != Value::Bool(true) {
        return Some("MIR payload must be instantiated".to_owned());
    }
    if payload["abi_verified"] != Value::Bool(true) {
        return Some("MIR payload ABI must be verified".to_owned());
    }
    if payload["executed"] != Value::Bool(true) {
        return Some("MIR payload execute must be called".to_owned());
    }
    if payload["execution_source"] != Value::String("embedded_registry".to_owned()) {
        return Some("MIR payload execution_source must be embedded_registry".to_owned());
    }
    if payload["opened_external_file"] != Value::Bool(false) {
        return Some("MIR payload execution must not open an external payload file".to_owned());
    }
    if payload["single_file_product"] != Value::Bool(true) {
        return Some("MIR payload single_file_product must be true".to_owned());
    }
    None
}

#[test]
fn dist_rouwdi_wasm_is_the_canonical_assembly_checkpoint() {
    let workspace = workspace_root();
    let manifest_path = workspace.join("dist/manifest.json");
    let artifact_path = workspace.join("dist/rouwdi.wasm");
    let source_path = workspace.join("target/wasm32-wasip1/release/rouwdi-assembly.wasm");
    let stub_path = workspace.join("target/wasm32-wasip1/release/rouwdi_wasm.wasm");

    let manifest: Value = serde_json::from_slice(
        &fs::read(&manifest_path).expect("run scripts/package.ps1 before workspace tests"),
    )
    .expect("dist/manifest.json must be valid JSON");

    assert_eq!(canonical_manifest_policy_error(&manifest), None);
    assert_eq!(
        manifest["package_mode"],
        Value::String("canonical_single_file".to_owned())
    );
    assert_eq!(
        manifest["canonical_artifact_path"],
        Value::String("dist/rouwdi.wasm".to_owned())
    );
    assert_eq!(
        manifest["source_build_artifact_path"],
        Value::String("target/wasm32-wasip1/release/rouwdi-assembly.wasm".to_owned())
    );
    assert_ne!(
        manifest["source_build_artifact_path"],
        Value::String("target/wasm32-wasip1/release/rouwdi_wasm.wasm".to_owned())
    );

    let artifact = fs::metadata(&artifact_path).expect("dist/rouwdi.wasm must exist");
    let source = fs::metadata(&source_path).expect("source assembly artifact must exist");
    let stub = fs::metadata(&stub_path).expect("cdylib stub must exist for rejection evidence");

    assert!(
        artifact.len() >= MIN_PRODUCT_SIZE_BYTES,
        "dist/rouwdi.wasm is suspiciously tiny"
    );
    assert_eq!(artifact.len(), source.len());
    assert_ne!(artifact.len(), stub.len());

    let artifact_sha256 = sha256_hex(&artifact_path);
    let source_sha256 = sha256_hex(&source_path);
    let stub_sha256 = sha256_hex(&stub_path);

    assert_eq!(artifact_sha256, source_sha256);
    assert_ne!(artifact_sha256, stub_sha256);
    assert_eq!(
        manifest["artifact"]["sha256"],
        Value::String(artifact_sha256)
    );
    assert_eq!(
        manifest["artifact"]["size_bytes"],
        Value::Number(artifact.len().into())
    );
    assert_eq!(
        manifest["rejected_cdylib_stub"]["rejected_as_product"],
        Value::Bool(true)
    );

    let payload = &manifest["mir_payload"];
    let state = payload["state"]
        .as_str()
        .expect("MIR payload state must be explicit");
    assert!(
        matches!(
            state,
            "embedded_payload_executed_blocked_at_mir_provider_requires_lang_items"
                | "embedded_payload_mir_body_hash_emitted"
                | "embedded_payload_mono_items_collected"
        ),
        "unexpected MIR payload state: {state}"
    );
    assert!(!EXTERNAL_ONLY_STATES.contains(&state));
    assert_eq!(payload["embedded"], Value::Bool(true));
    assert_eq!(payload["instantiated"], Value::Bool(true));
    assert_eq!(payload["abi_verified"], Value::Bool(true));
    assert_eq!(payload["executed"], Value::Bool(true));
    assert_eq!(
        payload["execution_source"],
        Value::String("embedded_registry".to_owned())
    );
    assert_eq!(payload["external"], Value::Bool(false));
    assert_eq!(payload["opened_external_file"], Value::Bool(false));
    assert_eq!(payload["single_file_product"], Value::Bool(true));
    assert_eq!(
        payload["metadata_source_path"],
        Value::String("bootstrap/mir-payload-export-manifest.toml".to_owned())
    );
    assert!(
        payload["path"]
            .as_str()
            .is_some_and(|path| path.ends_with("rouwdi_mir_adapter_probe.wasm")),
        "manifest must retain MIR payload path/hash/size metadata"
    );
    assert_eq!(
        payload["original_sha256"].as_str().map(str::len),
        Some(64),
        "manifest must retain MIR payload SHA-256"
    );
    assert!(
        payload["size_bytes"].as_u64().unwrap_or_default() > 80_000_000,
        "embedded MIR payload must have the direct payload size recorded"
    );
    assert_eq!(
        payload["embedding_method"],
        Value::String("raw_include_bytes".to_owned())
    );
    assert_eq!(payload["hash_verified"], Value::Bool(true));
    assert_eq!(payload["size_verified"], Value::Bool(true));
    assert_eq!(payload["wasm_magic_verified"], Value::Bool(true));
    assert_eq!(payload["module_instantiated"], Value::Bool(true));
    assert_eq!(payload["abi_v1_exports_verified"], Value::Bool(true));
    assert_eq!(payload["version_called"], Value::Bool(true));
    assert_eq!(payload["stage_called"], Value::Bool(true));
    assert_eq!(payload["descriptor_bytes_read"], Value::Bool(true));
    assert_eq!(payload["valid_input_bytes_read"], Value::Bool(true));
    assert_eq!(payload["execute_called"], Value::Bool(true));
    assert!(
        payload["execute_trapped"].is_boolean(),
        "manifest must record whether embedded execute trapped"
    );
    if payload["execute_trapped"] == Value::Bool(true) {
        assert!(
            payload["execute_trap"]
                .as_str()
                .is_some_and(|trap| !trap.is_empty()),
            "manifest must retain the embedded execute trap string"
        );
    }
    assert!(
        payload["output_bytes_read"] == Value::Bool(true)
            || payload["error_bytes_read"] == Value::Bool(true)
    );
    if payload["mir_body_identity_emitted"] == Value::Bool(true) {
        assert_eq!(payload["result_kind"], Value::String("output".to_owned()));
        assert_eq!(payload["mir_provider_invoked"], Value::Bool(true));
        assert_eq!(
            payload["next_frontier"],
            if payload["monomorphization_status"]
                == Value::String("mono_items_collected".to_owned())
            {
                if payload["rust_mono_item_wasm_object_emitted"] == Value::Bool(true)
                    && payload["linker_handoff_created"] == Value::Bool(true)
                {
                    Value::String("linking".to_owned())
                } else if payload["codegen_wasm_object_bytes_emitted"] == Value::Bool(true)
                    && payload["rust_mono_item_wasm_object_emitted"] != Value::Bool(true)
                {
                    Value::String(payload["codegen_handoff_status"].as_str().unwrap_or(
                        "codegen_lowering_blocked_at_codegen_lowering_to_object_not_implemented"
                    ).to_owned())
                } else {
                    Value::String("codegen".to_owned())
                }
            } else {
                Value::String("monomorphization".to_owned())
            }
        );
        assert_eq!(payload["rustc_monomorphize_invoked"], Value::Bool(true));
        assert!(
            payload["monomorphization_status"]
                .as_str()
                .is_some_and(|status| {
                    status == "mono_items_collected"
                        || status == "rustc_monomorphize_adapter_embedded"
                        || status.starts_with("rustc_monomorphize_invoked_blocked_at_")
                }),
            "manifest must carry exact monomorphization contact status"
        );
        assert_eq!(payload["fabricated_mono_items"], Value::Bool(false));
        if payload["monomorphization_status"] == Value::String("mono_items_collected".to_owned()) {
            assert!(
                payload["mono_item_count"]
                    .as_u64()
                    .is_some_and(|count| count > 0),
                "mono success must carry mono_item_count"
            );
            assert!(
                payload["mono_item_graph_hash"]
                    .as_str()
                    .is_some_and(|hash| !hash.is_empty()),
                "mono success must carry mono_item_graph_hash"
            );
            assert!(
                payload["mono_items"]
                    .as_array()
                    .is_some_and(|items| !items.is_empty()),
                "mono success must carry upstream mono_items"
            );
            assert_eq!(
                payload["mono_items_derived_from"],
                Value::String(
                    "rustc_middle::ty::TyCtxt::collect_and_partition_mono_items".to_owned()
                )
            );
            let codegen_status = payload["codegen_handoff_status"]
                .as_str()
                .unwrap_or_default();
            assert!(
                codegen_status == "rust_mono_item_wasm_object_emitted"
                    || PROBE_ONLY_CODEGEN_BLOCKERS.contains(&codegen_status),
                "canonical package must either emit a mono-item Wasm object or block at exact codegen lowering, got {codegen_status}"
            );
            assert_eq!(payload["rustc_codegen_llvm_attempted"], Value::Bool(true));
            assert_eq!(
                payload["codegen_backend_family"],
                Value::String("llvm-grade".to_owned())
            );
            assert_eq!(
                payload["codegen_expected_output_kind"],
                Value::String("wasm_object".to_owned())
            );
            assert!(
                payload["codegen_blocker_kind"].is_null()
                    || payload["codegen_blocker_kind"] == Value::String("none".to_owned())
                    || payload["codegen_blocker_kind"]
                        == Value::String("codegen_lowering_to_object_not_implemented".to_owned())
            );
            assert_eq!(
                payload["codegen_contact_state"],
                payload["codegen_handoff_status"]
            );
            if payload["rust_mono_item_wasm_object_emitted"] != Value::Bool(true) {
                assert_eq!(
                    payload["codegen_lowering_status"],
                    payload["codegen_handoff_status"]
                );
                assert_eq!(
                    payload["codegen_lowering_blocker_component"],
                    Value::String("rustc_codegen_ssa::base::codegen_crate".to_owned())
                );
                assert!(payload["codegen_lowering_required_path"]
                    .as_array()
                    .is_some_and(|path| path
                        .iter()
                        .any(|item| item == "rustc_codegen_llvm::base::compile_codegen_unit")));
                assert!(payload["codegen_lowering_missing_inputs"]
                    .as_array()
                    .is_some_and(|inputs| inputs
                        .iter()
                        .any(|item| item.as_str().is_some_and(|input| input.contains("TyCtxt")))));
            }
            assert_eq!(
                payload["host_probe_codegen_contact_state"],
                Value::String("target_machine_created".to_owned())
            );
            assert_eq!(payload["host_probe_llvm_module_created"], Value::Bool(true));
            assert_eq!(
                payload["host_probe_target_machine_created"],
                Value::Bool(true)
            );
            assert_eq!(payload["codegen_mono_proof_consumed"], Value::Bool(true));
            assert_eq!(payload["llvm_module_setup_invoked"], Value::Bool(true));
            assert_eq!(payload["llvm_context_created"], Value::Bool(true));
            assert_eq!(payload["llvm_module_created"], Value::Bool(true));
            assert_eq!(
                payload["llvm_module_identity_hash"],
                Value::String(
                    "23e20683dffb9b3b673ff866ace8826b8c6a933ef27e138f4e09f3e0a9d19e70".to_owned()
                )
            );
            assert_eq!(payload["target_machine_setup_invoked"], Value::Bool(true));
            assert_eq!(payload["target_machine_created"], Value::Bool(true));
            assert_eq!(
                payload["backend_payload_kind"],
                Value::String("codegen_backend_payload".to_owned())
            );
            assert!(
                payload["backend_payload_blocker_kind"].is_null()
                    || payload["backend_payload_blocker_kind"] == Value::String("none".to_owned())
                    || payload["backend_payload_blocker_kind"]
                        == Value::String("codegen_lowering_to_object_not_implemented".to_owned())
            );
            assert_eq!(payload["check_only_target_loadable"], Value::Bool(true));
            assert_eq!(
                payload["executable_backend_payload_linked"],
                Value::Bool(true)
            );
            assert_eq!(
                payload["backend_payload_build_attempted"],
                Value::Bool(true)
            );
            assert_eq!(payload["backend_payload_build_exit_code"], Value::from(0));
            assert_eq!(
                payload["backend_payload_final_link_invoked"],
                Value::Bool(true)
            );
            assert!(payload["backend_payload_linker"]
                .as_str()
                .is_some_and(|linker| linker.ends_with("wasm32-wasip1-clang.exe")));
            assert_eq!(
                payload["backend_payload_first_undefined_symbol"],
                Value::String(String::new())
            );
            assert!(payload["backend_payload_llvm_undefined_symbols"]
                .as_array()
                .is_some_and(|symbols| symbols.is_empty()));
            assert_eq!(
                payload["llvm_wrapper_target"],
                Value::String("wasm32-wasip1".to_owned())
            );
            assert_eq!(
                payload["llvm_wrapper_artifact_kind"],
                Value::String("staticlib".to_owned())
            );
            assert!(payload["llvm_wrapper_path"]
                .as_str()
                .is_some_and(|path| path.ends_with("libllvm-wrapper.a")));
            assert_eq!(
                payload["llvm_wrapper_sha256"].as_str().map(str::len),
                Some(64)
            );
            assert!(payload["llvm_wrapper_size_bytes"]
                .as_u64()
                .is_some_and(|size| size > 1_000_000));
            assert_eq!(payload["llvm_wrapper_target_loadable"], Value::Bool(true));
            assert_eq!(
                payload["target_llvm_library_closure_available"],
                Value::Bool(true)
            );
            assert_eq!(
                payload["target_llvm_library_closure_status"],
                Value::String("available".to_owned())
            );
            assert_eq!(
                payload["target_llvm_library_closure_build_attempted"],
                Value::Bool(true)
            );
            assert_eq!(
                payload["target_llvm_library_closure_build_exit_code"],
                Value::from(0)
            );
            assert_eq!(
                payload["target_llvm_library_closure_report_path"],
                Value::String(
                    ".rouwdi/codegen-llvm-probe/target-llvm-closure-report.json".to_owned()
                )
            );
            assert_eq!(
                payload["target_llvm_library_closure_first_error"],
                Value::String(String::new())
            );
            assert_eq!(
                payload["codegen_object_emission_attempted"],
                Value::Bool(true)
            );
            assert_eq!(payload["codegen_llvm_ir_emitted"], Value::Bool(true));
            assert_eq!(
                payload["codegen_llvm_ir_sha256"],
                Value::String(
                    "6b151410d83fa3fafc9c88ac4ef889635be7173652e0c6af95e015a515d72267".to_owned()
                )
            );
            assert_eq!(payload["codegen_llvm_ir_size_bytes"], Value::from(121));
            assert!(payload["codegen_object_emission_api"]
                .as_str()
                .is_some_and(|api| api.contains("LLVMTargetMachineEmitToMemoryBuffer")));
            assert_eq!(payload["codegen_object_bytes_emitted"], Value::Bool(true));
            assert_eq!(
                payload["codegen_wasm_object_bytes_emitted"],
                Value::Bool(true)
            );
            assert_eq!(
                payload["codegen_object_artifact_kind"],
                Value::String("wasm_object".to_owned())
            );
            assert_eq!(
                payload["codegen_object_artifact_sha256"],
                Value::String(
                    "0e4d3959d217324e5ca237cb9dc19cd1f40907a25da90c40ec68d71b67101985".to_owned()
                )
            );
            assert_eq!(
                payload["codegen_object_artifact_size_bytes"],
                Value::from(207)
            );
            assert_eq!(
                payload["codegen_object_artifact_location"],
                Value::String("vfs:/workspace/rouwdi-codegen-wasm32-wasip1.o".to_owned())
            );
            assert_eq!(
                payload["codegen_object_retrieval_method"],
                Value::String("rouwdi_owned_virtual_fs".to_owned())
            );
            assert_eq!(
                payload["codegen_object_bytes_retrieved_by_rouwdi"],
                Value::Bool(true)
            );
            assert_eq!(payload["codegen_object_sha256_verified"], Value::Bool(true));
            assert_eq!(
                payload["object_format"],
                Value::String("wasm_object".to_owned())
            );
            assert!(payload["object_section_count"]
                .as_u64()
                .is_some_and(|count| count > 0));
            assert!(payload["object_symbol_count"].is_number());
            assert!(payload["object_function_count"].is_number());
            assert!(payload["object_is_empty"].is_boolean());
            assert!(payload["object_has_code_bearing_content"].is_boolean());
            assert_eq!(payload["object_wasm_magic_valid"], Value::Bool(true));
            assert_eq!(payload["object_wasm_version_valid"], Value::Bool(true));
            assert!(payload["object_sections"]
                .as_array()
                .is_some_and(|sections| !sections.is_empty()));
            assert_eq!(
                payload["object_inspection"]["object_section_count"],
                payload["object_section_count"]
            );
            assert_eq!(
                payload["object_inspection"]["object_function_count"],
                payload["object_function_count"]
            );
            assert_eq!(
                payload["object_inspection"]["object_is_empty"],
                payload["object_is_empty"]
            );
            assert!(payload["object_parse_errors"]
                .as_array()
                .is_some_and(|errors| errors.is_empty()));
            if payload["rust_mono_item_wasm_object_emitted"] == Value::Bool(true) {
                assert_eq!(
                    payload["object_contains_codegened_function"],
                    Value::Bool(true)
                );
                assert!(payload["codegened_mono_item_count"]
                    .as_u64()
                    .is_some_and(|count| count > 0));
                assert_eq!(
                    payload["object_codegen_source"],
                    Value::String("mono_item_graph".to_owned())
                );
                assert_eq!(payload["linker_handoff_created"], Value::Bool(true));
            } else {
                assert_eq!(
                    payload["object_contains_codegened_function"],
                    Value::Bool(false)
                );
                assert_eq!(payload["codegened_mono_item_count"], Value::from(0));
                assert_eq!(payload["object_function_count"], Value::from(0));
                assert_eq!(payload["object_is_empty"], Value::Bool(true));
                assert_eq!(
                    payload["codegen_blocker_kind"],
                    Value::String("codegen_lowering_to_object_not_implemented".to_owned())
                );
                assert_eq!(payload["linker_handoff_created"], Value::Bool(false));
            }
        }
        assert!(
            payload["mir_body_identity"]
                .as_str()
                .is_some_and(|identity| identity.contains("def_id=")),
            "manifest must carry the real MIR body identity when emitted"
        );
        assert!(
            payload["mir_body_hash"]
                .as_str()
                .is_some_and(|hash| !hash.is_empty()),
            "manifest must carry MIR body hash when emitted"
        );
    }
    assert_eq!(
        payload["input_contract_sha256"].as_str().map(str::len),
        Some(64)
    );
    assert_eq!(payload["payload_registry_entry"], Value::Bool(true));

    let payload_size = payload["size_bytes"].as_u64().unwrap();
    assert!(
        artifact.len() >= payload_size + MIN_PRODUCT_SIZE_BYTES,
        "dist/rouwdi.wasm is too small to carry the raw MIR payload"
    );

    let codegen_payloads = manifest["codegen_payloads"]
        .as_array()
        .expect("canonical manifest must list codegen payload routes");
    assert!(
        codegen_payloads.iter().any(|entry| {
            entry["payload_name"] == Value::String("rouwdi-llvm-codegen-backend-payload".to_owned())
                && entry["payload_kind"] == Value::String("codegen_backend_payload".to_owned())
                && entry["backend_family"] == Value::String("llvm-grade".to_owned())
                && entry["upstream_component"] == Value::String("rustc_codegen_llvm".to_owned())
                && entry["target_triple"] == Value::String("wasm32-wasip1".to_owned())
                && entry["check_only_status"]
                    == Value::String("rustc_codegen_llvm_target_loadable_check_only".to_owned())
                && entry["check_only_target_loadable"] == Value::Bool(true)
                && entry["executable_backend_payload_linked"] == Value::Bool(true)
                && entry["backend_payload_build_attempted"] == Value::Bool(true)
                && entry["backend_payload_build_exit_code"] == Value::from(0)
                && entry["backend_payload_final_link_invoked"] == Value::Bool(true)
                && entry["backend_payload_linker"]
                    .as_str()
                    .is_some_and(|linker| linker.ends_with("wasm32-wasip1-clang.exe"))
                && entry["backend_payload_first_undefined_symbol"] == Value::String(String::new())
                && entry["backend_payload_llvm_undefined_symbols"]
                    .as_array()
                    .is_some_and(|symbols| symbols.is_empty())
                && entry["host_probe_state"]
                    == Value::String("host_codegen_probe_backend_constructed".to_owned())
                && entry["host_probe_codegen_contact_state"]
                    == Value::String("target_machine_created".to_owned())
                && entry["host_probe_llvm_module_created"] == Value::Bool(true)
                && entry["host_probe_target_machine_created"] == Value::Bool(true)
                && entry["codegen_contact_state"]
                    .as_str()
                    .is_some_and(|status| {
                        status == "rust_mono_item_wasm_object_emitted"
                            || PROBE_ONLY_CODEGEN_BLOCKERS.contains(&status)
                    })
                && entry["mono_proof_consumed"] == Value::Bool(true)
                && entry["llvm_wrapper_target"] == Value::String("wasm32-wasip1".to_owned())
                && entry["llvm_wrapper_artifact_kind"] == Value::String("staticlib".to_owned())
                && entry["llvm_wrapper_path"]
                    .as_str()
                    .is_some_and(|path| path.ends_with("libllvm-wrapper.a"))
                && entry["llvm_wrapper_sha256"]
                    .as_str()
                    .is_some_and(|hash| hash.len() == 64)
                && entry["llvm_wrapper_target_loadable"] == Value::Bool(true)
                && entry["target_llvm_library_closure_available"] == Value::Bool(true)
                && entry["target_llvm_library_closure_status"]
                    == Value::String("available".to_owned())
                && entry["target_llvm_library_closure_build_attempted"] == Value::Bool(true)
                && entry["target_llvm_library_closure_build_exit_code"] == Value::from(0)
                && entry["target_llvm_library_closure_first_error"] == Value::String(String::new())
                && entry["embedded_in_dist_rouwdi_wasm"] == Value::Bool(true)
                && entry["instantiated"] == Value::Bool(true)
                && entry["executed"] == Value::Bool(true)
                && entry["llvm_module_created"] == Value::Bool(true)
                && entry["target_machine_created"] == Value::Bool(true)
                && entry["llvm_ir_emitted"] == Value::Bool(true)
                && entry["llvm_ir_sha256"]
                    == Value::String(
                        "6b151410d83fa3fafc9c88ac4ef889635be7173652e0c6af95e015a515d72267"
                            .to_owned(),
                    )
                && entry["llvm_ir_size_bytes"] == Value::from(121)
                && entry["object_emission_attempted"] == Value::Bool(true)
                && entry["object_emission_api"]
                    .as_str()
                    .is_some_and(|api| api.contains("LLVMTargetMachineEmitToMemoryBuffer"))
                && entry["object_bytes_emitted"] == Value::Bool(true)
                && entry["wasm_object_bytes_emitted"] == Value::Bool(true)
                && entry["object_artifact_kind"] == Value::String("wasm_object".to_owned())
                && entry["object_artifact_sha256"]
                    == Value::String(
                        "0e4d3959d217324e5ca237cb9dc19cd1f40907a25da90c40ec68d71b67101985"
                            .to_owned(),
                    )
                && entry["object_artifact_size_bytes"] == Value::from(207)
                && entry["object_artifact_location"]
                    == Value::String("vfs:/workspace/rouwdi-codegen-wasm32-wasip1.o".to_owned())
                && entry["object_retrieval_method"]
                    == Value::String("rouwdi_owned_virtual_fs".to_owned())
                && entry["object_bytes_retrieved_by_rouwdi"] == Value::Bool(true)
                && entry["object_sha256_verified"] == Value::Bool(true)
                && entry["object_format"] == Value::String("wasm_object".to_owned())
                && entry["object_section_count"]
                    .as_u64()
                    .is_some_and(|count| count > 0)
                && entry["object_symbol_count"].is_number()
                && entry["object_function_count"].is_number()
                && entry["object_is_empty"].is_boolean()
                && entry["object_has_code_bearing_content"].is_boolean()
                && entry["object_wasm_magic_valid"] == Value::Bool(true)
                && entry["object_wasm_version_valid"] == Value::Bool(true)
                && entry["object_sections"]
                    .as_array()
                    .is_some_and(|sections| !sections.is_empty())
                && entry["object_inspection"]["object_section_count"]
                    == entry["object_section_count"]
                && entry["object_inspection"]["object_function_count"]
                    == entry["object_function_count"]
                && entry["object_inspection"]["object_is_empty"] == entry["object_is_empty"]
                && entry["object_parse_errors"]
                    .as_array()
                    .is_some_and(|errors| errors.is_empty())
                && if entry["rust_mono_item_wasm_object_emitted"] == Value::Bool(true) {
                    entry["object_contains_codegened_function"] == Value::Bool(true)
                        && entry["codegened_mono_item_count"]
                            .as_u64()
                            .is_some_and(|count| count > 0)
                        && entry["object_codegen_source"]
                            == Value::String("mono_item_graph".to_owned())
                        && entry["linker_handoff_created"] == Value::Bool(true)
                } else {
                    entry["object_contains_codegened_function"] == Value::Bool(false)
                        && entry["codegened_mono_item_count"] == Value::from(0)
                        && entry["object_function_count"] == Value::from(0)
                        && entry["object_is_empty"] == Value::Bool(true)
                        && entry["codegen_lowering_status"] == entry["codegen_contact_state"]
                        && entry["codegen_lowering_blocker_component"]
                            == Value::String("rustc_codegen_ssa::base::codegen_crate".to_owned())
                        && entry["codegen_lowering_required_path"]
                            .as_array()
                            .is_some_and(|path| {
                                path.iter().any(|item| {
                                    item == "rustc_codegen_llvm::base::compile_codegen_unit"
                                })
                            })
                        && entry["linker_handoff_created"] == Value::Bool(false)
                }
        }),
        "canonical manifest must retain the assembly-owned LLVM backend payload route"
    );

    let embedded_payloads = manifest["embedded_payloads"]
        .as_array()
        .expect("canonical manifest must list embedded payloads");
    assert!(
        embedded_payloads.iter().any(|entry| {
            entry["name"] == Value::String("rouwdi-mir-handoff-payload".to_owned())
                && entry["embedded"] == Value::Bool(true)
                && entry["external"] == Value::Bool(false)
                && entry["instantiated"] == Value::Bool(true)
                && entry["abi_verified"] == Value::Bool(true)
                && entry["executed"] == Value::Bool(true)
                && entry["execution_source"] == Value::String("embedded_registry".to_owned())
                && entry["embedded_sha256"] == payload["embedded_sha256"]
        }),
        "embedded payload registry entry must include the MIR payload"
    );
}

#[test]
fn canonical_policy_rejects_external_only_payload_states() {
    for state in EXTERNAL_ONLY_STATES {
        let manifest = serde_json::json!({
            "package_mode": "canonical_single_file",
            "canonical_artifact_path": "dist/rouwdi.wasm",
            "single_file_product": true,
            "mir_payload": {
                "state": state,
                "embedded": false,
                "external": true,
                "single_file_product": false
            }
        });

        let error = canonical_manifest_policy_error(&manifest)
            .expect("external-only state must be rejected");
        assert!(
            error.contains("embedded")
                || error.contains("external-only")
                || error.contains("external"),
            "unexpected rejection reason: {error}"
        );
    }
}

#[test]
fn dev_external_payload_manifest_is_not_a_product_checkpoint() {
    let manifest = serde_json::json!({
        "package_mode": "dev_external_payload",
        "single_file_product": false,
        "not_product_complete": true,
        "mir_payload": {
            "state": "external_hash_verified_payload",
            "embedded": false,
            "external": true,
            "single_file_product": false
        }
    });

    let error = canonical_manifest_policy_error(&manifest)
        .expect("dev external payload manifest must not satisfy canonical policy");
    assert!(error.contains("package_mode"));
}
