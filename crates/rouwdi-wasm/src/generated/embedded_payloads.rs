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
