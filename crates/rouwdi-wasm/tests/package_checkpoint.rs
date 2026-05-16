use serde_json::Value;
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};

const MIN_PRODUCT_SIZE_BYTES: u64 = 1_048_576;
const EXTERNAL_ONLY_STATES: &[&str] =
    &["metadata_reference_only", "external_hash_verified_payload"];

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
            | "embedded_payload_mir_body_identity_emitted"
            | "embedded_payload_mir_body_hash_emitted"
    ) {
        return Some(format!("unexpected canonical MIR payload state: {state}"));
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
                | "embedded_payload_mir_body_identity_emitted"
                | "embedded_payload_mir_body_hash_emitted"
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
            Value::String("monomorphization".to_owned())
        );
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
