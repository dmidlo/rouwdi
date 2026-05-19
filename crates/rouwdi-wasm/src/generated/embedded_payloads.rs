pub(super) const MIR_PAYLOAD_NAME: &str = "rouwdi-mir-handoff-payload";
pub(super) const MIR_PAYLOAD_KIND: &str = "compiler_payload";
pub(super) const MIR_PAYLOAD_STAGE: &str = "mir_handoff";
pub(super) const MIR_PAYLOAD_ABI_NAME: &str = "rouwdi.compiler-payload.mir-handoff";
pub(super) const MIR_PAYLOAD_ABI_VERSION: u32 = 1;
pub(super) const MIR_PAYLOAD_TARGET_TRIPLE: &str = "wasm32-wasip1";
pub(super) const MIR_PAYLOAD_BUILD_SOURCE_PATH: &str =
    ".rouwdi/direct-rustc-private-pack/target/wasm32-wasip1/release/rouwdi_mir_adapter_probe.wasm";
pub(super) const MIR_PAYLOAD_GENERATION_COMMAND: &str =
    "cargo run -p rouwdi-rustc-upstream --bin direct-rustc-private-pack-builder";
pub(super) const MIR_PAYLOAD_LOAD_STRATEGY: &str = "instantiate_wasm_module";
pub(super) const MIR_PAYLOAD_EMBEDDING_METHOD: &str = "raw_include_bytes";
pub(super) const MIR_PAYLOAD_STATE: &str = "embedded_payload";
pub(super) const MIR_PAYLOAD_SHA256: &str =
    "274c096ee655396249883e0fd4080eae0f9fa1300259e556473fc1f6cd68abf9";
pub(super) const MIR_PAYLOAD_SIZE_BYTES: u64 = 88623078;
pub(super) const MIR_PAYLOAD_BYTES: &[u8] = include_bytes!(
    "../../../../.rouwdi/direct-rustc-private-pack/target/wasm32-wasip1/release/rouwdi_mir_adapter_probe.wasm"
);

pub(super) const CODEGEN_PAYLOAD_NAME: &str = "rouwdi-llvm-codegen-backend-payload";
pub(super) const CODEGEN_PAYLOAD_KIND: &str = "codegen_backend_payload";
pub(super) const CODEGEN_PAYLOAD_BACKEND: &str = "rustc_codegen_llvm";
pub(super) const CODEGEN_PAYLOAD_BACKEND_FAMILY: &str = "llvm-grade";
pub(super) const CODEGEN_PAYLOAD_TARGET_TRIPLE: &str = "wasm32-wasip1";
pub(super) const CODEGEN_PAYLOAD_ARTIFACT_PATH: &str =
    ".rouwdi/codegen-llvm-probe/wasm-target/wasm32-wasip1/release/deps/rouwdi_rustc_codegen_llvm_probe-c3347eb5e6bcbfc2.wasm";
pub(super) const CODEGEN_PAYLOAD_GENERATION_COMMAND: &str =
    "powershell -ExecutionPolicy Bypass -File bootstrap/rustc-codegen-llvm-probe/run-wasm-target-check.ps1";
pub(super) const CODEGEN_PAYLOAD_LOAD_STRATEGY: &str = "instantiate_wasi_cli_module";
pub(super) const CODEGEN_PAYLOAD_EMBEDDING_METHOD: &str = "raw_include_bytes";
pub(super) const CODEGEN_PAYLOAD_STATE: &str = "embedded_payload";
pub(super) const CODEGEN_PAYLOAD_SHA256: &str =
    "b95c2ebb89fce004315207534a5aad4553d20e32fc979b9d85b561333cac7a99";
pub(super) const CODEGEN_PAYLOAD_SIZE_BYTES: u64 = 144125994;
pub(super) const CODEGEN_PAYLOAD_BYTES: &[u8] = include_bytes!(
    "../../../../.rouwdi/codegen-llvm-probe/wasm-target/wasm32-wasip1/release/deps/rouwdi_rustc_codegen_llvm_probe-c3347eb5e6bcbfc2.wasm"
);
