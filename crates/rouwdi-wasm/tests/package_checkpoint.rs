use serde_json::Value;
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};

const MIN_PRODUCT_SIZE_BYTES: u64 = 1_048_576;

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
            "metadata_reference_only" | "external_hash_verified_payload"
        ),
        "unexpected MIR payload state: {state}"
    );
    assert_eq!(payload["embedded"], Value::Bool(false));
    assert_ne!(state, "embedded_payload");
    assert_ne!(state, "embedded_and_instantiated_payload");
    assert_eq!(payload["single_file_product"], Value::Bool(false));
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
        payload["sha256"].as_str().map(str::len),
        Some(64),
        "manifest must retain MIR payload SHA-256"
    );
    assert!(
        payload["size_bytes"].as_u64().unwrap_or_default() > artifact.len(),
        "current external MIR payload must not be implied to be inside dist/rouwdi.wasm"
    );
}
