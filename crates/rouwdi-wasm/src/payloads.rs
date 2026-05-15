use serde::Serialize;
use sha2::{Digest, Sha256};

#[cfg(feature = "embedded-mir-payload")]
#[path = "generated/embedded_payloads.rs"]
mod embedded_payloads;

#[derive(Debug, Clone, Copy, Serialize)]
pub struct EmbeddedCompilerPayload {
    pub name: &'static str,
    pub kind: &'static str,
    pub stage: &'static str,
    pub abi_name: &'static str,
    pub abi_version: u32,
    pub target_triple: &'static str,
    pub build_source_path: &'static str,
    pub generation_command: &'static str,
    pub load_strategy: &'static str,
    pub embedding_method: &'static str,
    pub state: &'static str,
    pub expected_sha256: &'static str,
    pub expected_size_bytes: u64,
    pub uncompressed_size_bytes: u64,
    pub compressed_size_bytes: Option<u64>,
    #[serde(skip_serializing)]
    pub bytes: &'static [u8],
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct EmbeddedCompilerPayloadReport {
    pub name: String,
    pub kind: String,
    pub stage: String,
    pub abi_name: String,
    pub abi_version: u32,
    pub target_triple: String,
    pub build_source_path: String,
    pub generation_command: String,
    pub load_strategy: String,
    pub embedding_method: String,
    pub state: String,
    pub expected_sha256: String,
    pub actual_sha256: String,
    pub expected_size_bytes: u64,
    pub actual_size_bytes: u64,
    pub uncompressed_size_bytes: u64,
    pub compressed_size_bytes: Option<u64>,
    pub hash_verified: bool,
    pub size_verified: bool,
    pub loader_available: bool,
}

#[cfg(feature = "embedded-mir-payload")]
static EMBEDDED_COMPILER_PAYLOADS: &[EmbeddedCompilerPayload] = &[EmbeddedCompilerPayload {
    name: embedded_payloads::MIR_PAYLOAD_NAME,
    kind: embedded_payloads::MIR_PAYLOAD_KIND,
    stage: embedded_payloads::MIR_PAYLOAD_STAGE,
    abi_name: embedded_payloads::MIR_PAYLOAD_ABI_NAME,
    abi_version: embedded_payloads::MIR_PAYLOAD_ABI_VERSION,
    target_triple: embedded_payloads::MIR_PAYLOAD_TARGET_TRIPLE,
    build_source_path: embedded_payloads::MIR_PAYLOAD_BUILD_SOURCE_PATH,
    generation_command: embedded_payloads::MIR_PAYLOAD_GENERATION_COMMAND,
    load_strategy: embedded_payloads::MIR_PAYLOAD_LOAD_STRATEGY,
    embedding_method: embedded_payloads::MIR_PAYLOAD_EMBEDDING_METHOD,
    state: embedded_payloads::MIR_PAYLOAD_STATE,
    expected_sha256: embedded_payloads::MIR_PAYLOAD_SHA256,
    expected_size_bytes: embedded_payloads::MIR_PAYLOAD_SIZE_BYTES,
    uncompressed_size_bytes: embedded_payloads::MIR_PAYLOAD_SIZE_BYTES,
    compressed_size_bytes: None,
    bytes: embedded_payloads::MIR_PAYLOAD_BYTES,
}];

#[cfg(not(feature = "embedded-mir-payload"))]
static EMBEDDED_COMPILER_PAYLOADS: &[EmbeddedCompilerPayload] = &[];

pub fn embedded_compiler_payloads() -> &'static [EmbeddedCompilerPayload] {
    EMBEDDED_COMPILER_PAYLOADS
}

pub fn embedded_compiler_payload_reports() -> Vec<EmbeddedCompilerPayloadReport> {
    embedded_compiler_payloads()
        .iter()
        .map(|payload| {
            let actual_sha256 = sha256_hex(payload.bytes);
            let actual_size_bytes = payload.bytes.len() as u64;
            let hash_verified = actual_sha256 == payload.expected_sha256;
            let size_verified = actual_size_bytes == payload.expected_size_bytes;

            EmbeddedCompilerPayloadReport {
                name: payload.name.to_owned(),
                kind: payload.kind.to_owned(),
                stage: payload.stage.to_owned(),
                abi_name: payload.abi_name.to_owned(),
                abi_version: payload.abi_version,
                target_triple: payload.target_triple.to_owned(),
                build_source_path: payload.build_source_path.to_owned(),
                generation_command: payload.generation_command.to_owned(),
                load_strategy: payload.load_strategy.to_owned(),
                embedding_method: payload.embedding_method.to_owned(),
                state: if hash_verified && size_verified {
                    "embedded_payload_hash_verified".to_owned()
                } else {
                    "embedded_payload_hash_mismatch".to_owned()
                },
                expected_sha256: payload.expected_sha256.to_owned(),
                actual_sha256,
                expected_size_bytes: payload.expected_size_bytes,
                actual_size_bytes,
                uncompressed_size_bytes: payload.uncompressed_size_bytes,
                compressed_size_bytes: payload.compressed_size_bytes,
                hash_verified,
                size_verified,
                loader_available: hash_verified && size_verified,
            }
        })
        .collect()
}

pub fn mir_payload_report() -> Option<EmbeddedCompilerPayloadReport> {
    embedded_compiler_payload_reports()
        .into_iter()
        .find(|payload| payload.name == "rouwdi-mir-handoff-payload")
}

fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut out = String::with_capacity(digest.len() * 2);
    for byte in digest {
        use std::fmt::Write as _;
        let _ = write!(&mut out, "{byte:02x}");
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(feature = "embedded-mir-payload")]
    fn embedded_registry_carries_direct_mir_payload_bytes() {
        let payloads = embedded_compiler_payloads();
        let mir = payloads
            .iter()
            .find(|payload| payload.name == "rouwdi-mir-handoff-payload")
            .expect("canonical registry must include direct MIR payload");

        assert_eq!(mir.kind, "compiler_payload");
        assert_eq!(mir.stage, "mir_handoff");
        assert_eq!(mir.abi_name, "rouwdi.compiler-payload.mir-handoff");
        assert_eq!(mir.abi_version, 1);
        assert_eq!(mir.target_triple, "wasm32-wasip1");
        assert_eq!(mir.embedding_method, "raw_include_bytes");
        assert_eq!(mir.state, "embedded_payload");
        assert_eq!(mir.bytes.len() as u64, mir.expected_size_bytes);
        assert!(mir.bytes.len() > 80_000_000);
        assert_eq!(&mir.bytes[..4], b"\0asm");

        let report = mir_payload_report().expect("MIR payload report must exist");
        assert!(report.hash_verified);
        assert!(report.size_verified);
        assert!(report.loader_available);
        assert_eq!(report.actual_sha256, mir.expected_sha256);
    }
}
