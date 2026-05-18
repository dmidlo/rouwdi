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
    "b9ae49950e1f1f12768211d4b5f8fa9f6a8ebb52cacafe2bb701688db59f7c54";
pub(super) const MIR_PAYLOAD_SIZE_BYTES: u64 = 88495302;
pub(super) const MIR_PAYLOAD_BYTES: &[u8] = include_bytes!(
    "../../../../.rouwdi/direct-rustc-private-pack/target/wasm32-wasip1/release/rouwdi_mir_adapter_probe.wasm"
);

pub(super) const CODEGEN_PAYLOAD_NAME: &str = "rouwdi-llvm-codegen-backend-payload";
pub(super) const CODEGEN_PAYLOAD_KIND: &str = "codegen_backend_payload";
pub(super) const CODEGEN_PAYLOAD_BACKEND: &str = "rustc_codegen_llvm";
pub(super) const CODEGEN_PAYLOAD_BACKEND_FAMILY: &str = "llvm-grade";
pub(super) const CODEGEN_PAYLOAD_TARGET_TRIPLE: &str = "wasm32-wasip1";
pub(super) const CODEGEN_PAYLOAD_ARTIFACT_PATH: &str =
    ".rouwdi/codegen-llvm-probe/wasm-target/wasm32-wasip1/release/deps/rouwdi_rustc_codegen_llvm_probe-2877186751952474.wasm";
pub(super) const CODEGEN_PAYLOAD_GENERATION_COMMAND: &str =
    "powershell -ExecutionPolicy Bypass -File bootstrap/rustc-codegen-llvm-probe/run-wasm-target-check.ps1";
pub(super) const CODEGEN_PAYLOAD_LOAD_STRATEGY: &str = "instantiate_wasi_cli_module";
pub(super) const CODEGEN_PAYLOAD_EMBEDDING_METHOD: &str = "raw_include_bytes";
pub(super) const CODEGEN_PAYLOAD_STATE: &str = "embedded_payload";
pub(super) const CODEGEN_PAYLOAD_SHA256: &str =
    "74c01c58f0d108ca7b214c54bf5f9f5671ca876e47695d643c34630e0bd3c04a";
pub(super) const CODEGEN_PAYLOAD_SIZE_BYTES: u64 = 67765756;
pub(super) const CODEGEN_PAYLOAD_BYTES: &[u8] = include_bytes!(
    "../../../../.rouwdi/codegen-llvm-probe/wasm-target/wasm32-wasip1/release/deps/rouwdi_rustc_codegen_llvm_probe-2877186751952474.wasm"
);
