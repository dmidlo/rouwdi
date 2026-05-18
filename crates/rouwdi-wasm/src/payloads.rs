use rouwdi_object::{inspect_wasm_object, WasmObjectInspection};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use wasmi::{Caller, Config, Engine, Extern, Linker, Memory, Module, Store, TypedResumableCall};

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

#[derive(Debug, Clone, Copy, Serialize)]
pub struct EmbeddedCodegenPayload {
    pub name: &'static str,
    pub kind: &'static str,
    pub backend: &'static str,
    pub backend_family: &'static str,
    pub target_triple: &'static str,
    pub artifact_path: &'static str,
    pub generation_command: &'static str,
    pub load_strategy: &'static str,
    pub embedding_method: &'static str,
    pub state: &'static str,
    pub expected_sha256: &'static str,
    pub expected_size_bytes: u64,
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct EmbeddedCompilerPayloadLoadReport {
    pub name: String,
    pub kind: String,
    pub stage: String,
    pub abi_name: String,
    pub abi_version: u32,
    pub registry_identity: String,
    pub execution_source: String,
    pub external: bool,
    pub opened_external_file: bool,
    pub build_source_path: String,
    pub load_strategy: String,
    pub embedding_method: String,
    pub expected_sha256: String,
    pub actual_sha256: String,
    pub hash_verified: bool,
    pub expected_size_bytes: u64,
    pub actual_size_bytes: u64,
    pub size_verified: bool,
    pub wasm_magic_verified: bool,
    pub module_instantiated: bool,
    pub imports: Vec<String>,
    pub exports: Vec<String>,
    pub abi_v1_exports_verified: bool,
    pub version_called: bool,
    pub version: u32,
    pub stage_called: bool,
    pub stage_code: u32,
    pub descriptor_ptr: u32,
    pub descriptor_len: u32,
    pub descriptor_bytes_read: bool,
    pub descriptor_json: String,
    pub valid_input_ptr: u32,
    pub valid_input_len: u32,
    pub valid_input_bytes_read: bool,
    pub valid_input_json: String,
    pub execute_called: bool,
    pub execute_status: i32,
    pub execute_trapped: bool,
    pub execute_trap: Option<String>,
    pub output_ptr: u32,
    pub output_len: u32,
    pub error_ptr: u32,
    pub error_len: u32,
    pub output_bytes_read: bool,
    pub output_json: Option<String>,
    pub error_bytes_read: bool,
    pub error_json: Option<String>,
    pub input_contract_sha256: String,
    pub output_contract_sha256: Option<String>,
    pub error_contract_sha256: Option<String>,
    pub execution_state: String,
    pub blocker_kind: Option<String>,
    pub result_kind: String,
    pub stdout_bytes: usize,
    pub stderr_bytes: usize,
    pub stdout_text: String,
    pub stderr_text: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct EmbeddedCompilerPayloadLoadError {
    pub name: String,
    pub execution_source: String,
    pub external: bool,
    pub opened_external_file: bool,
    pub error: String,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct EmbeddedCodegenPayloadExecutionReport {
    pub name: String,
    pub kind: String,
    pub backend: String,
    pub backend_family: String,
    pub target_triple: String,
    pub execution_source: String,
    pub external: bool,
    pub opened_external_file: bool,
    pub artifact_path: String,
    pub load_strategy: String,
    pub embedding_method: String,
    pub expected_sha256: String,
    pub actual_sha256: String,
    pub hash_verified: bool,
    pub expected_size_bytes: u64,
    pub actual_size_bytes: u64,
    pub size_verified: bool,
    pub wasm_magic_verified: bool,
    pub module_instantiated: bool,
    pub start_called: bool,
    pub start_trapped: bool,
    pub start_trap: Option<String>,
    pub execute_status: i32,
    pub imports: Vec<String>,
    pub exports: Vec<String>,
    pub argv: Vec<String>,
    pub stdout_bytes: usize,
    pub stderr_bytes: usize,
    pub stdout_text: String,
    pub stderr_text: String,
    pub wasi_trace: Vec<String>,
    pub output_json: Option<serde_json::Value>,
    pub backend_constructed: bool,
    pub backend_name: Option<String>,
    pub codegen_contact_state: Option<String>,
    pub codegen_lowering_status: Option<String>,
    pub codegen_lowering_blocker_kind: Option<String>,
    pub codegen_lowering_blocker_component: Option<String>,
    pub codegen_lowering_blocker_reason: Option<String>,
    pub codegen_lowering_required_path: Vec<String>,
    pub codegen_lowering_missing_inputs: Vec<String>,
    pub mono_proof_consumed: bool,
    pub mir_body_hash: Option<String>,
    pub mono_item_count: Option<u64>,
    pub mono_item_graph_hash: Option<String>,
    pub llvm_context_created: bool,
    pub llvm_module_created: bool,
    pub llvm_module_identity: Option<String>,
    pub llvm_module_identity_hash: Option<String>,
    pub llvm_module_target_triple: Option<String>,
    pub target_machine_setup_invoked: bool,
    pub target_machine_created: bool,
    pub target_machine_cpu: Option<String>,
    pub target_machine_features: Option<String>,
    pub target_machine_relocation_model: Option<String>,
    pub target_machine_code_model: Option<String>,
    pub target_machine_optimization_level: Option<String>,
    pub llvm_ir_emitted: bool,
    pub llvm_ir_sha256: Option<String>,
    pub llvm_ir_size_bytes: Option<u64>,
    pub bitcode_emitted: bool,
    pub object_emission_attempted: bool,
    pub object_emission_api: Option<String>,
    pub object_bytes_emitted: bool,
    pub wasm_object_bytes_emitted: bool,
    pub rust_mono_item_wasm_object_emitted: bool,
    pub codegened_mono_item_count: u64,
    pub codegened_symbols: Vec<String>,
    pub object_contains_codegened_function: bool,
    pub object_symbol_table_contains_codegened_symbol: bool,
    pub object_derived_from: Option<String>,
    pub object_codegen_source: Option<String>,
    pub object_inspection: Option<WasmObjectInspection>,
    pub object_format: Option<String>,
    pub object_section_count: Option<u64>,
    pub object_has_code_section: Option<bool>,
    pub object_has_linking_metadata: Option<bool>,
    pub object_symbol_count: Option<u64>,
    pub object_function_count: Option<u64>,
    pub object_is_empty: Option<bool>,
    pub object_has_code_bearing_content: Option<bool>,
    pub object_artifact_kind: Option<String>,
    pub object_artifact_sha256: Option<String>,
    pub object_artifact_size_bytes: Option<u64>,
    pub object_artifact_location: Option<String>,
    pub object_target_triple: Option<String>,
    pub object_retrieval_method: Option<String>,
    pub object_bytes_retrieved_by_rouwdi: bool,
    pub object_sha256_verified: bool,
    pub codegen_artifact_kind: Option<String>,
    pub codegen_artifact_sha256: Option<String>,
    pub codegen_artifact_size_bytes: Option<u64>,
    pub codegen_artifact_location: Option<String>,
    pub linker_required: bool,
    pub linker_handoff_created: bool,
    pub blocker_kind: Option<String>,
    pub blocker_component: Option<String>,
    pub blocker_reason: Option<String>,
}

#[derive(Debug)]
struct PayloadWasiState {
    args: Vec<String>,
    env: Vec<String>,
    stdout: Vec<u8>,
    stderr: Vec<u8>,
    proc_exit_code: Option<i32>,
    random_counter: u8,
    next_fd: i32,
    fds: BTreeMap<i32, VirtualFd>,
    written_files: BTreeMap<String, Vec<u8>>,
    created_dirs: BTreeSet<String>,
    wasi_trace: Vec<String>,
}

#[derive(Debug)]
enum VirtualFd {
    File { bytes: &'static [u8], position: u64 },
    WrittenFile { path: String, position: u64 },
    Directory { entries: Vec<VirtualDirEntry> },
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct VirtualDirEntry {
    name: String,
    filetype: u8,
}

struct VirtualFile {
    path: &'static str,
    bytes: &'static [u8],
}

const WASI: &str = "wasi_snapshot_preview1";
const ABI_VERSION_SYMBOL: &str = "rouwdi_compiler_payload_abi_v1_version";
const ABI_STAGE_SYMBOL: &str = "rouwdi_compiler_payload_abi_v1_stage";
const ABI_DESCRIPTOR_PTR_SYMBOL: &str = "rouwdi_compiler_payload_abi_v1_descriptor_ptr";
const ABI_DESCRIPTOR_LEN_SYMBOL: &str = "rouwdi_compiler_payload_abi_v1_descriptor_len";
const MIR_VALID_INPUT_PTR_SYMBOL: &str = "rouwdi_mir_handoff_payload_v1_valid_input_ptr";
const MIR_VALID_INPUT_LEN_SYMBOL: &str = "rouwdi_mir_handoff_payload_v1_valid_input_len";
const MIR_RESULT_AREA_PTR_SYMBOL: &str = "rouwdi_mir_handoff_payload_v1_result_area_ptr";
const MIR_EXECUTE_SYMBOL: &str = "rouwdi_mir_handoff_payload_v1_execute";
const MIR_LAST_ERROR_PTR_SYMBOL: &str = "rouwdi_mir_handoff_payload_v1_last_error_ptr";
const MIR_LAST_ERROR_LEN_SYMBOL: &str = "rouwdi_mir_handoff_payload_v1_last_error_len";
const PAYLOAD_FUEL_CHUNK: u64 = 1_000_000_000;
const PAYLOAD_MAX_FUEL_RESUMES: usize = 50_000;
const WASI_ERRNO_SUCCESS: i32 = 0;
const WASI_ERRNO_BADF: i32 = 8;
const WASI_ERRNO_INVAL: i32 = 28;
const WASI_ERRNO_NOENT: i32 = 44;
const WASI_ERRNO_NOSYS: i32 = 52;
const WASI_FILETYPE_CHARACTER_DEVICE: u8 = 2;
const WASI_FILETYPE_DIRECTORY: u8 = 3;
const WASI_FILETYPE_REGULAR_FILE: u8 = 4;
const WASI_PREOPEN_FD: i32 = 3;
const WASI_PREOPEN_PATH: &str = "/";
const WASI_OFLAGS_CREAT: i32 = 1;
const WASI_OFLAGS_DIRECTORY: i32 = 2;
const WASI_OFLAGS_TRUNC: i32 = 8;
const VIRTUAL_SYSROOT: &str = "third_party/rust/build/x86_64-pc-windows-msvc/stage1";
const VIRTUAL_RUSTLIB: &str =
    "third_party/rust/build/x86_64-pc-windows-msvc/stage1/lib/rustlib/wasm32-wasip1/lib";
const CODEGEN_OBJECT_ARTIFACT_PATH: &str = "rouwdi-codegen-wasm32-wasip1.o";
const PAYLOAD_ARG0: &str = "rouwdi_mir_adapter_probe.wasm";

macro_rules! virtual_file {
    ($path:literal) => {
        VirtualFile {
            path: $path,
            bytes: include_bytes!(concat!("../../../", $path)),
        }
    };
}

static VIRTUAL_FILES: &[VirtualFile] = &[
    virtual_file!("third_party/rust/build/x86_64-pc-windows-msvc/stage1/lib/rustlib/wasm32-wasip1/lib/libaddr2line-e53ee625818e66de.rlib"),
    virtual_file!("third_party/rust/build/x86_64-pc-windows-msvc/stage1/lib/rustlib/wasm32-wasip1/lib/libaddr2line-e53ee625818e66de.rmeta"),
    virtual_file!("third_party/rust/build/x86_64-pc-windows-msvc/stage1/lib/rustlib/wasm32-wasip1/lib/libadler2-6c6ce22a3d784b53.rlib"),
    virtual_file!("third_party/rust/build/x86_64-pc-windows-msvc/stage1/lib/rustlib/wasm32-wasip1/lib/libadler2-6c6ce22a3d784b53.rmeta"),
    virtual_file!("third_party/rust/build/x86_64-pc-windows-msvc/stage1/lib/rustlib/wasm32-wasip1/lib/liballoc-aa5de9cb44693937.rlib"),
    virtual_file!("third_party/rust/build/x86_64-pc-windows-msvc/stage1/lib/rustlib/wasm32-wasip1/lib/liballoc-aa5de9cb44693937.rmeta"),
    virtual_file!("third_party/rust/build/x86_64-pc-windows-msvc/stage1/lib/rustlib/wasm32-wasip1/lib/libcfg_if-f330595bed847612.rlib"),
    virtual_file!("third_party/rust/build/x86_64-pc-windows-msvc/stage1/lib/rustlib/wasm32-wasip1/lib/libcfg_if-f330595bed847612.rmeta"),
    virtual_file!("third_party/rust/build/x86_64-pc-windows-msvc/stage1/lib/rustlib/wasm32-wasip1/lib/libcompiler_builtins-242fe6d76c147fd1.rlib"),
    virtual_file!("third_party/rust/build/x86_64-pc-windows-msvc/stage1/lib/rustlib/wasm32-wasip1/lib/libcompiler_builtins-242fe6d76c147fd1.rmeta"),
    virtual_file!("third_party/rust/build/x86_64-pc-windows-msvc/stage1/lib/rustlib/wasm32-wasip1/lib/libcore-fc7b12ec85c54ac0.rlib"),
    virtual_file!("third_party/rust/build/x86_64-pc-windows-msvc/stage1/lib/rustlib/wasm32-wasip1/lib/libcore-fc7b12ec85c54ac0.rmeta"),
    virtual_file!("third_party/rust/build/x86_64-pc-windows-msvc/stage1/lib/rustlib/wasm32-wasip1/lib/libgetopts-4973914a20a2bcab.rlib"),
    virtual_file!("third_party/rust/build/x86_64-pc-windows-msvc/stage1/lib/rustlib/wasm32-wasip1/lib/libgetopts-4973914a20a2bcab.rmeta"),
    virtual_file!("third_party/rust/build/x86_64-pc-windows-msvc/stage1/lib/rustlib/wasm32-wasip1/lib/libgimli-99ea63bed8e623b8.rlib"),
    virtual_file!("third_party/rust/build/x86_64-pc-windows-msvc/stage1/lib/rustlib/wasm32-wasip1/lib/libgimli-99ea63bed8e623b8.rmeta"),
    virtual_file!("third_party/rust/build/x86_64-pc-windows-msvc/stage1/lib/rustlib/wasm32-wasip1/lib/libhashbrown-82db15a0bc02cc07.rlib"),
    virtual_file!("third_party/rust/build/x86_64-pc-windows-msvc/stage1/lib/rustlib/wasm32-wasip1/lib/libhashbrown-82db15a0bc02cc07.rmeta"),
    virtual_file!("third_party/rust/build/x86_64-pc-windows-msvc/stage1/lib/rustlib/wasm32-wasip1/lib/liblibc-d076481cb93820f2.rlib"),
    virtual_file!("third_party/rust/build/x86_64-pc-windows-msvc/stage1/lib/rustlib/wasm32-wasip1/lib/liblibc-d076481cb93820f2.rmeta"),
    virtual_file!("third_party/rust/build/x86_64-pc-windows-msvc/stage1/lib/rustlib/wasm32-wasip1/lib/libmemchr-1c439cd9baea8fa9.rlib"),
    virtual_file!("third_party/rust/build/x86_64-pc-windows-msvc/stage1/lib/rustlib/wasm32-wasip1/lib/libmemchr-1c439cd9baea8fa9.rmeta"),
    virtual_file!("third_party/rust/build/x86_64-pc-windows-msvc/stage1/lib/rustlib/wasm32-wasip1/lib/libminiz_oxide-b1bb72b0c937980d.rlib"),
    virtual_file!("third_party/rust/build/x86_64-pc-windows-msvc/stage1/lib/rustlib/wasm32-wasip1/lib/libminiz_oxide-b1bb72b0c937980d.rmeta"),
    virtual_file!("third_party/rust/build/x86_64-pc-windows-msvc/stage1/lib/rustlib/wasm32-wasip1/lib/libobject-689541b9682e059d.rlib"),
    virtual_file!("third_party/rust/build/x86_64-pc-windows-msvc/stage1/lib/rustlib/wasm32-wasip1/lib/libobject-689541b9682e059d.rmeta"),
    virtual_file!("third_party/rust/build/x86_64-pc-windows-msvc/stage1/lib/rustlib/wasm32-wasip1/lib/libpanic_abort-771e1103f866bdb4.rlib"),
    virtual_file!("third_party/rust/build/x86_64-pc-windows-msvc/stage1/lib/rustlib/wasm32-wasip1/lib/libpanic_abort-771e1103f866bdb4.rmeta"),
    virtual_file!("third_party/rust/build/x86_64-pc-windows-msvc/stage1/lib/rustlib/wasm32-wasip1/lib/libpanic_unwind-b86d45a3567913a4.rlib"),
    virtual_file!("third_party/rust/build/x86_64-pc-windows-msvc/stage1/lib/rustlib/wasm32-wasip1/lib/libpanic_unwind-b86d45a3567913a4.rmeta"),
    virtual_file!("third_party/rust/build/x86_64-pc-windows-msvc/stage1/lib/rustlib/wasm32-wasip1/lib/libproc_macro-0f637836edc73bd6.rlib"),
    virtual_file!("third_party/rust/build/x86_64-pc-windows-msvc/stage1/lib/rustlib/wasm32-wasip1/lib/libproc_macro-0f637836edc73bd6.rmeta"),
    virtual_file!("third_party/rust/build/x86_64-pc-windows-msvc/stage1/lib/rustlib/wasm32-wasip1/lib/librustc_demangle-3e5dfd60db0f61c6.rlib"),
    virtual_file!("third_party/rust/build/x86_64-pc-windows-msvc/stage1/lib/rustlib/wasm32-wasip1/lib/librustc_demangle-3e5dfd60db0f61c6.rmeta"),
    virtual_file!("third_party/rust/build/x86_64-pc-windows-msvc/stage1/lib/rustlib/wasm32-wasip1/lib/librustc_literal_escaper-c68215798b1e662f.rlib"),
    virtual_file!("third_party/rust/build/x86_64-pc-windows-msvc/stage1/lib/rustlib/wasm32-wasip1/lib/librustc_literal_escaper-c68215798b1e662f.rmeta"),
    virtual_file!("third_party/rust/build/x86_64-pc-windows-msvc/stage1/lib/rustlib/wasm32-wasip1/lib/librustc_std_workspace_alloc-4243392e063083ab.rlib"),
    virtual_file!("third_party/rust/build/x86_64-pc-windows-msvc/stage1/lib/rustlib/wasm32-wasip1/lib/librustc_std_workspace_alloc-4243392e063083ab.rmeta"),
    virtual_file!("third_party/rust/build/x86_64-pc-windows-msvc/stage1/lib/rustlib/wasm32-wasip1/lib/librustc_std_workspace_core-40703e9aafc1d450.rlib"),
    virtual_file!("third_party/rust/build/x86_64-pc-windows-msvc/stage1/lib/rustlib/wasm32-wasip1/lib/librustc_std_workspace_core-40703e9aafc1d450.rmeta"),
    virtual_file!("third_party/rust/build/x86_64-pc-windows-msvc/stage1/lib/rustlib/wasm32-wasip1/lib/librustc_std_workspace_std-b88f9aa124f30ca4.rlib"),
    virtual_file!("third_party/rust/build/x86_64-pc-windows-msvc/stage1/lib/rustlib/wasm32-wasip1/lib/librustc_std_workspace_std-b88f9aa124f30ca4.rmeta"),
    virtual_file!("third_party/rust/build/x86_64-pc-windows-msvc/stage1/lib/rustlib/wasm32-wasip1/lib/libstd_detect-992133543cee23b6.rlib"),
    virtual_file!("third_party/rust/build/x86_64-pc-windows-msvc/stage1/lib/rustlib/wasm32-wasip1/lib/libstd_detect-992133543cee23b6.rmeta"),
    virtual_file!("third_party/rust/build/x86_64-pc-windows-msvc/stage1/lib/rustlib/wasm32-wasip1/lib/libstd-b594a2ae141e7c9c.rlib"),
    virtual_file!("third_party/rust/build/x86_64-pc-windows-msvc/stage1/lib/rustlib/wasm32-wasip1/lib/libstd-b594a2ae141e7c9c.rmeta"),
    virtual_file!("third_party/rust/build/x86_64-pc-windows-msvc/stage1/lib/rustlib/wasm32-wasip1/lib/libsysroot-aa7459ddf5b8c47e.rlib"),
    virtual_file!("third_party/rust/build/x86_64-pc-windows-msvc/stage1/lib/rustlib/wasm32-wasip1/lib/libsysroot-aa7459ddf5b8c47e.rmeta"),
    virtual_file!("third_party/rust/build/x86_64-pc-windows-msvc/stage1/lib/rustlib/wasm32-wasip1/lib/libtest-3dd6218e18a7bd6e.rlib"),
    virtual_file!("third_party/rust/build/x86_64-pc-windows-msvc/stage1/lib/rustlib/wasm32-wasip1/lib/libtest-3dd6218e18a7bd6e.rmeta"),
    virtual_file!("third_party/rust/build/x86_64-pc-windows-msvc/stage1/lib/rustlib/wasm32-wasip1/lib/libunwind-801234150e5ffd1c.rlib"),
    virtual_file!("third_party/rust/build/x86_64-pc-windows-msvc/stage1/lib/rustlib/wasm32-wasip1/lib/libunwind-801234150e5ffd1c.rmeta"),
    virtual_file!("third_party/rust/build/x86_64-pc-windows-msvc/stage1/lib/rustlib/wasm32-wasip1/lib/libwasip1-bdf89526125af68e.rlib"),
    virtual_file!("third_party/rust/build/x86_64-pc-windows-msvc/stage1/lib/rustlib/wasm32-wasip1/lib/libwasip1-bdf89526125af68e.rmeta"),
    virtual_file!("third_party/rust/build/x86_64-pc-windows-msvc/stage1/lib/rustlib/wasm32-wasip1/lib/self-contained/crt1-command.o"),
    virtual_file!("third_party/rust/build/x86_64-pc-windows-msvc/stage1/lib/rustlib/wasm32-wasip1/lib/self-contained/crt1-reactor.o"),
    virtual_file!("third_party/rust/build/x86_64-pc-windows-msvc/stage1/lib/rustlib/wasm32-wasip1/lib/self-contained/libc.a"),
    virtual_file!("third_party/rust/build/x86_64-pc-windows-msvc/stage1/lib/rustlib/wasm32-wasip1/lib/self-contained/libunwind.a"),
];

const REQUIRED_ABI_EXPORTS: &[&str] = &[
    "memory",
    ABI_VERSION_SYMBOL,
    ABI_STAGE_SYMBOL,
    ABI_DESCRIPTOR_PTR_SYMBOL,
    ABI_DESCRIPTOR_LEN_SYMBOL,
    MIR_VALID_INPUT_PTR_SYMBOL,
    MIR_VALID_INPUT_LEN_SYMBOL,
    MIR_RESULT_AREA_PTR_SYMBOL,
    MIR_EXECUTE_SYMBOL,
    MIR_LAST_ERROR_PTR_SYMBOL,
    MIR_LAST_ERROR_LEN_SYMBOL,
];

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

#[cfg(feature = "embedded-mir-payload")]
static EMBEDDED_CODEGEN_PAYLOADS: &[EmbeddedCodegenPayload] = &[EmbeddedCodegenPayload {
    name: embedded_payloads::CODEGEN_PAYLOAD_NAME,
    kind: embedded_payloads::CODEGEN_PAYLOAD_KIND,
    backend: embedded_payloads::CODEGEN_PAYLOAD_BACKEND,
    backend_family: embedded_payloads::CODEGEN_PAYLOAD_BACKEND_FAMILY,
    target_triple: embedded_payloads::CODEGEN_PAYLOAD_TARGET_TRIPLE,
    artifact_path: embedded_payloads::CODEGEN_PAYLOAD_ARTIFACT_PATH,
    generation_command: embedded_payloads::CODEGEN_PAYLOAD_GENERATION_COMMAND,
    load_strategy: embedded_payloads::CODEGEN_PAYLOAD_LOAD_STRATEGY,
    embedding_method: embedded_payloads::CODEGEN_PAYLOAD_EMBEDDING_METHOD,
    state: embedded_payloads::CODEGEN_PAYLOAD_STATE,
    expected_sha256: embedded_payloads::CODEGEN_PAYLOAD_SHA256,
    expected_size_bytes: embedded_payloads::CODEGEN_PAYLOAD_SIZE_BYTES,
    bytes: embedded_payloads::CODEGEN_PAYLOAD_BYTES,
}];

#[cfg(not(feature = "embedded-mir-payload"))]
static EMBEDDED_CODEGEN_PAYLOADS: &[EmbeddedCodegenPayload] = &[];

pub fn embedded_compiler_payloads() -> &'static [EmbeddedCompilerPayload] {
    EMBEDDED_COMPILER_PAYLOADS
}

pub fn embedded_codegen_payloads() -> &'static [EmbeddedCodegenPayload] {
    EMBEDDED_CODEGEN_PAYLOADS
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

pub fn load_embedded_compiler_payload(
    name: &str,
) -> Result<EmbeddedCompilerPayloadLoadReport, EmbeddedCompilerPayloadLoadError> {
    #[cfg(not(target_arch = "wasm32"))]
    {
        let name = name.to_owned();
        let thread_name = name.clone();
        return std::thread::Builder::new()
            .name("rouwdi-embedded-payload-loader".to_owned())
            .stack_size(512 * 1024 * 1024)
            .spawn(move || load_embedded_compiler_payload_inline(&thread_name))
            .map_err(|error| EmbeddedCompilerPayloadLoadError {
                name: name.clone(),
                execution_source: "embedded_registry".to_owned(),
                external: false,
                opened_external_file: false,
                error: format!("failed to spawn embedded payload loader thread: {error}"),
            })?
            .join()
            .map_err(|_| EmbeddedCompilerPayloadLoadError {
                name,
                execution_source: "embedded_registry".to_owned(),
                external: false,
                opened_external_file: false,
                error: "embedded payload loader thread panicked".to_owned(),
            })?;
    }

    #[cfg(target_arch = "wasm32")]
    {
        load_embedded_compiler_payload_inline(name)
    }
}

fn load_embedded_compiler_payload_inline(
    name: &str,
) -> Result<EmbeddedCompilerPayloadLoadReport, EmbeddedCompilerPayloadLoadError> {
    let Some(payload) = embedded_compiler_payloads()
        .iter()
        .find(|payload| payload.name == name)
    else {
        return Err(EmbeddedCompilerPayloadLoadError {
            name: name.to_owned(),
            execution_source: "embedded_registry".to_owned(),
            external: false,
            opened_external_file: false,
            error: "embedded compiler payload was not found in registry".to_owned(),
        });
    };

    execute_embedded_payload(payload).map_err(|error| EmbeddedCompilerPayloadLoadError {
        name: payload.name.to_owned(),
        execution_source: "embedded_registry".to_owned(),
        external: false,
        opened_external_file: false,
        error,
    })
}

pub fn load_mir_handoff_payload(
) -> Result<EmbeddedCompilerPayloadLoadReport, EmbeddedCompilerPayloadLoadError> {
    load_embedded_compiler_payload("rouwdi-mir-handoff-payload")
}

pub fn load_codegen_backend_payload(
) -> Result<EmbeddedCodegenPayloadExecutionReport, EmbeddedCompilerPayloadLoadError> {
    #[cfg(not(target_arch = "wasm32"))]
    {
        return std::thread::Builder::new()
            .name("rouwdi-embedded-codegen-payload-loader".to_owned())
            .stack_size(1024 * 1024 * 1024)
            .spawn(load_codegen_backend_payload_inline)
            .map_err(|error| EmbeddedCompilerPayloadLoadError {
                name: "rouwdi-llvm-codegen-backend-payload".to_owned(),
                execution_source: "embedded_registry".to_owned(),
                external: false,
                opened_external_file: false,
                error: format!("failed to spawn embedded codegen payload loader thread: {error}"),
            })?
            .join()
            .map_err(|_| EmbeddedCompilerPayloadLoadError {
                name: "rouwdi-llvm-codegen-backend-payload".to_owned(),
                execution_source: "embedded_registry".to_owned(),
                external: false,
                opened_external_file: false,
                error: "embedded codegen payload loader thread panicked".to_owned(),
            })?;
    }

    #[cfg(target_arch = "wasm32")]
    {
        load_codegen_backend_payload_inline()
    }
}

fn load_codegen_backend_payload_inline(
) -> Result<EmbeddedCodegenPayloadExecutionReport, EmbeddedCompilerPayloadLoadError> {
    let Some(payload) = embedded_codegen_payloads()
        .iter()
        .find(|payload| payload.name == "rouwdi-llvm-codegen-backend-payload")
    else {
        return Err(EmbeddedCompilerPayloadLoadError {
            name: "rouwdi-llvm-codegen-backend-payload".to_owned(),
            execution_source: "embedded_registry".to_owned(),
            external: false,
            opened_external_file: false,
            error: "embedded codegen payload was not found in registry".to_owned(),
        });
    };

    execute_embedded_codegen_payload(payload).map_err(|error| EmbeddedCompilerPayloadLoadError {
        name: payload.name.to_owned(),
        execution_source: "embedded_registry".to_owned(),
        external: false,
        opened_external_file: false,
        error,
    })
}

pub fn mir_payload_execution_for_engine() -> Option<rouwdi_rustc::RustEmbeddedMirPayloadExecution> {
    let report = load_mir_handoff_payload().ok()?;
    Some(rouwdi_rustc::RustEmbeddedMirPayloadExecution {
        payload_identity: report.name,
        registry_identity: report.registry_identity,
        execution_source: report.execution_source,
        external: report.external,
        opened_external_file: report.opened_external_file,
        embedded: true,
        expected_sha256: report.expected_sha256,
        actual_sha256: report.actual_sha256,
        hash_verified: report.hash_verified,
        expected_size_bytes: report.expected_size_bytes,
        actual_size_bytes: report.actual_size_bytes,
        size_verified: report.size_verified,
        wasm_magic_verified: report.wasm_magic_verified,
        module_instantiated: report.module_instantiated,
        abi_v1_exports_verified: report.abi_v1_exports_verified,
        exports: report.exports,
        imports: report.imports,
        abi_version_called: report.version_called,
        abi_version: report.version,
        stage_called: report.stage_called,
        stage: report.stage_code,
        descriptor_called: report.descriptor_bytes_read,
        descriptor_json: report.descriptor_json,
        valid_input_called: report.valid_input_bytes_read,
        valid_input_json: report.valid_input_json,
        execute_called: report.execute_called,
        execute_status: report.execute_status,
        execute_trapped: report.execute_trapped,
        execute_trap: report.execute_trap,
        output_bytes_read: report.output_bytes_read,
        output_json: report.output_json,
        error_bytes_read: report.error_bytes_read,
        error_json: report.error_json,
        input_contract_sha256: report.input_contract_sha256,
        output_contract_sha256: report.output_contract_sha256,
        error_contract_sha256: report.error_contract_sha256,
        execution_state: report.execution_state,
        blocker_kind: report.blocker_kind,
        result_kind: report.result_kind,
    })
}

fn execute_embedded_codegen_payload(
    payload: &EmbeddedCodegenPayload,
) -> Result<EmbeddedCodegenPayloadExecutionReport, String> {
    let actual_sha256 = sha256_hex(payload.bytes);
    let actual_size_bytes = payload.bytes.len() as u64;
    let hash_verified = actual_sha256 == payload.expected_sha256;
    let size_verified = actual_size_bytes == payload.expected_size_bytes;
    let wasm_magic_verified = payload.bytes.starts_with(b"\0asm");
    if !hash_verified || !size_verified || !wasm_magic_verified {
        return Err(format!(
            "embedded codegen payload identity verification failed: hash_verified={hash_verified} size_verified={size_verified} wasm_magic_verified={wasm_magic_verified}"
        ));
    }

    let mut config = Config::default();
    config.consume_fuel(true);
    config.set_max_recursion_depth(8192);
    config.ignore_custom_sections(true);
    let engine = Engine::new(&config);
    let module = Module::new(&engine, payload.bytes)
        .map_err(|error| format!("failed to compile embedded codegen payload: {error}"))?;
    let imports = module
        .imports()
        .map(|import| format!("{}::{}", import.module(), import.name()))
        .collect::<Vec<_>>();
    let exports = module
        .exports()
        .map(|export| export.name().to_owned())
        .collect::<Vec<_>>();
    let argv = codegen_payload_argv();
    let mut store = Store::new(
        &engine,
        PayloadWasiState {
            args: argv.clone(),
            env: vec!["PWD=/".to_owned(), "RUST_BACKTRACE=1".to_owned()],
            stdout: Vec::new(),
            stderr: Vec::new(),
            proc_exit_code: None,
            random_counter: 0,
            next_fd: WASI_PREOPEN_FD + 1,
            fds: BTreeMap::new(),
            written_files: BTreeMap::new(),
            created_dirs: BTreeSet::new(),
            wasi_trace: Vec::new(),
        },
    );
    store
        .set_fuel(PAYLOAD_FUEL_CHUNK)
        .map_err(|error| error.to_string())?;
    let mut linker = Linker::<PayloadWasiState>::new(&engine);
    define_wasi_imports(&mut linker)?;
    let instance = linker
        .instantiate_and_start(&mut store, &module)
        .map_err(|error| format!("failed to instantiate embedded codegen payload: {error}"))?;
    let start = instance
        .get_typed_func::<(), ()>(&store, "_start")
        .map_err(|error| format!("missing/corrupt _start export: {error}"))?;
    let start_outcome = call_start_with_resumable_fuel(&start, &mut store)?;
    let start_trap = start_outcome.trap.clone();
    let start_trapped = start_trap.is_some() && store.data().proc_exit_code.is_none();
    if start_trapped {
        let stdout_text = String::from_utf8_lossy(&store.data().stdout).into_owned();
        let stderr_text = String::from_utf8_lossy(&store.data().stderr).into_owned();
        return Err(format!(
            "embedded codegen payload start trapped: {}; stdout={}; stderr={}",
            start_trap.clone().unwrap_or_default(),
            stdout_text,
            stderr_text
        ));
    }

    let stdout_bytes = store.data().stdout.len();
    let stderr_bytes = store.data().stderr.len();
    let stdout_text = String::from_utf8_lossy(&store.data().stdout).into_owned();
    let stderr_text = String::from_utf8_lossy(&store.data().stderr).into_owned();
    let wasi_trace = store.data().wasi_trace.clone();
    let mut output_json = serde_json::from_str::<serde_json::Value>(&stdout_text).ok();
    attach_runtime_proof_to_codegen_output(&mut output_json, &store.data().written_files);
    let llvm_module_setup = output_json
        .as_ref()
        .and_then(|value| value.get("llvm_module_setup"));
    let target_machine_setup = output_json
        .as_ref()
        .and_then(|value| value.get("target_machine_setup"));
    let codegen_artifact = output_json
        .as_ref()
        .and_then(|value| value.get("codegen_artifact"));
    let object_artifact = output_json
        .as_ref()
        .and_then(|value| value.get("object_artifact"));
    let object_output_path = output_json
        .as_ref()
        .and_then(|value| value.get("object_artifact_location"))
        .and_then(serde_json::Value::as_str)
        .map(normalize_reported_vfs_path)
        .unwrap_or_else(|| CODEGEN_OBJECT_ARTIFACT_PATH.to_owned());
    let retrieved_object_bytes = store.data().written_files.get(&object_output_path);
    let object_inspection = retrieved_object_bytes.map(|bytes| inspect_wasm_object(bytes));
    let retrieved_object_sha256 = retrieved_object_bytes.map(|bytes| sha256_hex(bytes));
    let retrieved_object_size = retrieved_object_bytes.map(|bytes| bytes.len() as u64);
    let reported_object_sha256 = json_string_field(output_json.as_ref(), "object_artifact_sha256")
        .or_else(|| json_string_field(object_artifact, "sha256"));
    let object_sha256_verified = retrieved_object_sha256.is_some()
        && reported_object_sha256.as_deref() == retrieved_object_sha256.as_deref();
    let object_bytes_retrieved_by_rouwdi = retrieved_object_bytes.is_some();
    let object_artifact_kind = json_string_field(output_json.as_ref(), "object_artifact_kind")
        .or_else(|| json_string_field(object_artifact, "artifact_kind"));
    let object_artifact_location =
        json_string_field(output_json.as_ref(), "object_artifact_location")
            .or_else(|| json_string_field(object_artifact, "embedded_artifact_location"));
    let object_target_triple = json_string_field(output_json.as_ref(), "object_target_triple")
        .or_else(|| json_string_field(object_artifact, "target_triple"));
    let object_retrieval_method =
        json_string_field(output_json.as_ref(), "object_retrieval_method")
            .or_else(|| json_string_field(object_artifact, "retrieval_method"));
    let object_bytes_emitted =
        json_bool(output_json.as_ref(), "object_bytes_emitted") && object_bytes_retrieved_by_rouwdi;
    let wasm_object_bytes_emitted = json_bool(output_json.as_ref(), "wasm_object_bytes_emitted")
        && object_bytes_emitted
        && object_artifact_kind.as_deref() == Some("wasm_object");
    let reported_codegened_symbols = json_string_array(output_json.as_ref(), "codegened_symbols");
    let object_symbol_table_contains_codegened_symbol =
        object_contains_any_reported_codegened_symbol(
            object_inspection.as_ref(),
            &reported_codegened_symbols,
        );
    let reported_object_contains_codegened_function =
        json_bool(output_json.as_ref(), "object_contains_codegened_function")
            || json_bool(object_artifact, "object_contains_codegened_function");
    let object_contains_codegened_function = wasm_object_bytes_emitted
        && reported_object_contains_codegened_function
        && object_symbol_table_contains_codegened_symbol
        && object_inspection
            .as_ref()
            .is_some_and(|inspection| inspection.object_has_code_bearing_content);
    let rust_mono_item_wasm_object_emitted = wasm_object_bytes_emitted
        && object_contains_codegened_function
        && json_bool(output_json.as_ref(), "rust_mono_item_wasm_object_emitted");
    let codegened_mono_item_count = if rust_mono_item_wasm_object_emitted {
        output_json
            .as_ref()
            .and_then(|value| value.get("codegened_mono_item_count"))
            .and_then(serde_json::Value::as_u64)
            .unwrap_or(0)
    } else {
        0
    };
    let codegened_symbols = if rust_mono_item_wasm_object_emitted {
        reported_codegened_symbols
    } else {
        Vec::new()
    };
    let codegen_contact_state = if wasm_object_bytes_emitted && !rust_mono_item_wasm_object_emitted
    {
        json_string_field(output_json.as_ref(), "codegen_lowering_status").or_else(|| {
            Some(
                "codegen_lowering_blocked_at_codegen_lowering_to_object_not_implemented".to_owned(),
            )
        })
    } else {
        json_string_field(output_json.as_ref(), "codegen_contact_state")
    };
    let blocker_kind = if wasm_object_bytes_emitted && !rust_mono_item_wasm_object_emitted {
        Some("codegen_lowering_to_object_not_implemented".to_owned())
    } else {
        json_string_field(output_json.as_ref(), "blocker_kind").filter(|kind| kind != "none")
    };
    let blocker_component = if wasm_object_bytes_emitted && !rust_mono_item_wasm_object_emitted {
        Some("rustc_codegen_llvm mono item lowering".to_owned())
    } else {
        json_string_field(output_json.as_ref(), "blocker_component")
            .filter(|component| component != "none")
    };
    let blocker_reason = if wasm_object_bytes_emitted && !rust_mono_item_wasm_object_emitted {
        Some("rouwdi-owned Wasm object inspection found no code-bearing function tied to the mono item graph; linker handoff is blocked until rustc_codegen_llvm lowers the mono item into the LLVM module".to_owned())
    } else {
        json_string_field(output_json.as_ref(), "blocker_reason").filter(|reason| reason != "none")
    };
    let codegen_artifact_kind = if object_bytes_emitted {
        object_artifact_kind.clone()
    } else {
        json_string_field(codegen_artifact, "artifact_kind")
    };
    let codegen_artifact_sha256 = if object_bytes_emitted {
        retrieved_object_sha256.clone()
    } else {
        json_string_field(codegen_artifact, "sha256")
    };
    let codegen_artifact_size_bytes = if object_bytes_emitted {
        retrieved_object_size
    } else {
        codegen_artifact
            .and_then(|value| value.get("byte_length"))
            .and_then(serde_json::Value::as_u64)
    };
    let codegen_artifact_location = if object_bytes_emitted {
        object_artifact_location.clone()
    } else {
        json_string_field(codegen_artifact, "embedded_artifact_location")
    };

    Ok(EmbeddedCodegenPayloadExecutionReport {
        name: payload.name.to_owned(),
        kind: payload.kind.to_owned(),
        backend: payload.backend.to_owned(),
        backend_family: payload.backend_family.to_owned(),
        target_triple: payload.target_triple.to_owned(),
        execution_source: "embedded_registry".to_owned(),
        external: false,
        opened_external_file: false,
        artifact_path: payload.artifact_path.to_owned(),
        load_strategy: payload.load_strategy.to_owned(),
        embedding_method: payload.embedding_method.to_owned(),
        expected_sha256: payload.expected_sha256.to_owned(),
        actual_sha256,
        hash_verified,
        expected_size_bytes: payload.expected_size_bytes,
        actual_size_bytes,
        size_verified,
        wasm_magic_verified,
        module_instantiated: true,
        start_called: true,
        start_trapped,
        start_trap,
        execute_status: store.data().proc_exit_code.unwrap_or(start_outcome.status),
        imports,
        exports,
        argv,
        stdout_bytes,
        stderr_bytes,
        stdout_text,
        stderr_text,
        wasi_trace,
        backend_constructed: json_bool(output_json.as_ref(), "backend_constructed"),
        backend_name: json_string_field(output_json.as_ref(), "backend_name"),
        codegen_contact_state,
        codegen_lowering_status: json_string_field(output_json.as_ref(), "codegen_lowering_status"),
        codegen_lowering_blocker_kind: json_string_field(
            output_json.as_ref(),
            "codegen_lowering_blocker_kind",
        )
        .filter(|kind| kind != "none"),
        codegen_lowering_blocker_component: json_string_field(
            output_json.as_ref(),
            "codegen_lowering_blocker_component",
        )
        .filter(|component| component != "none"),
        codegen_lowering_blocker_reason: json_string_field(
            output_json.as_ref(),
            "codegen_lowering_blocker_reason",
        )
        .filter(|reason| reason != "none"),
        codegen_lowering_required_path: json_string_array(
            output_json.as_ref(),
            "codegen_lowering_required_path",
        ),
        codegen_lowering_missing_inputs: json_string_array(
            output_json.as_ref(),
            "codegen_lowering_missing_inputs",
        ),
        mono_proof_consumed: json_bool(output_json.as_ref(), "mono_proof_consumed"),
        mir_body_hash: json_string_field(output_json.as_ref(), "mir_body_hash"),
        mono_item_count: output_json
            .as_ref()
            .and_then(|value| value.get("mono_item_count"))
            .and_then(serde_json::Value::as_u64),
        mono_item_graph_hash: json_string_field(output_json.as_ref(), "mono_item_graph_hash"),
        llvm_context_created: json_bool(llvm_module_setup, "llvm_context_created"),
        llvm_module_created: json_bool(llvm_module_setup, "llvm_module_created"),
        llvm_module_identity: json_string_field(llvm_module_setup, "module_identity"),
        llvm_module_identity_hash: json_string_field(llvm_module_setup, "module_identity_hash"),
        llvm_module_target_triple: json_string_field(llvm_module_setup, "module_target_triple"),
        target_machine_setup_invoked: json_bool(target_machine_setup, "attempted"),
        target_machine_created: json_bool(target_machine_setup, "target_machine_created"),
        target_machine_cpu: json_string_field(target_machine_setup, "cpu"),
        target_machine_features: json_string_field(target_machine_setup, "features"),
        target_machine_relocation_model: json_string_field(
            target_machine_setup,
            "relocation_model",
        ),
        target_machine_code_model: json_string_field(target_machine_setup, "code_model"),
        target_machine_optimization_level: json_string_field(
            target_machine_setup,
            "optimization_level",
        ),
        llvm_ir_emitted: json_bool(output_json.as_ref(), "llvm_ir_emitted"),
        llvm_ir_sha256: json_string_field(output_json.as_ref(), "llvm_ir_sha256"),
        llvm_ir_size_bytes: output_json
            .as_ref()
            .and_then(|value| value.get("llvm_ir_size_bytes"))
            .and_then(serde_json::Value::as_u64),
        bitcode_emitted: json_bool(output_json.as_ref(), "bitcode_emitted"),
        object_emission_attempted: json_bool(output_json.as_ref(), "object_emission_attempted"),
        object_emission_api: json_string_field(output_json.as_ref(), "object_emission_api"),
        object_bytes_emitted,
        wasm_object_bytes_emitted,
        rust_mono_item_wasm_object_emitted,
        codegened_mono_item_count,
        codegened_symbols,
        object_contains_codegened_function,
        object_symbol_table_contains_codegened_symbol,
        object_derived_from: json_string_field(output_json.as_ref(), "object_derived_from")
            .or_else(|| json_string_field(object_artifact, "object_derived_from")),
        object_codegen_source: json_string_field(output_json.as_ref(), "object_codegen_source")
            .or_else(|| json_string_field(object_artifact, "object_codegen_source")),
        object_format: object_inspection
            .as_ref()
            .map(|inspection| inspection.object_format.clone()),
        object_section_count: object_inspection
            .as_ref()
            .map(|inspection| inspection.object_section_count),
        object_has_code_section: object_inspection
            .as_ref()
            .map(|inspection| inspection.object_has_code_section),
        object_has_linking_metadata: object_inspection
            .as_ref()
            .map(|inspection| inspection.object_has_linking_metadata),
        object_symbol_count: object_inspection
            .as_ref()
            .map(|inspection| inspection.object_symbol_count),
        object_function_count: object_inspection
            .as_ref()
            .map(|inspection| inspection.object_function_count),
        object_is_empty: object_inspection
            .as_ref()
            .map(|inspection| inspection.object_is_empty),
        object_has_code_bearing_content: object_inspection
            .as_ref()
            .map(|inspection| inspection.object_has_code_bearing_content),
        object_inspection,
        object_artifact_kind,
        object_artifact_sha256: retrieved_object_sha256.or(reported_object_sha256),
        object_artifact_size_bytes: retrieved_object_size.or_else(|| {
            output_json
                .as_ref()
                .and_then(|value| value.get("object_artifact_size_bytes"))
                .and_then(serde_json::Value::as_u64)
                .or_else(|| {
                    object_artifact
                        .and_then(|value| value.get("byte_length"))
                        .and_then(serde_json::Value::as_u64)
                })
        }),
        object_artifact_location,
        object_target_triple,
        object_retrieval_method,
        object_bytes_retrieved_by_rouwdi,
        object_sha256_verified,
        codegen_artifact_kind,
        codegen_artifact_sha256,
        codegen_artifact_size_bytes,
        codegen_artifact_location,
        linker_required: json_bool(output_json.as_ref(), "linker_required"),
        linker_handoff_created: json_bool(output_json.as_ref(), "linker_handoff_created")
            && rust_mono_item_wasm_object_emitted,
        blocker_kind,
        blocker_component,
        blocker_reason,
        output_json,
    })
}

fn normalize_reported_vfs_path(path: &str) -> String {
    normalize_virtual_path(
        path.strip_prefix("vfs:")
            .or_else(|| path.strip_prefix("memory:"))
            .unwrap_or(path),
    )
}

fn attach_runtime_proof_to_codegen_output(
    output_json: &mut Option<serde_json::Value>,
    written_files: &BTreeMap<String, Vec<u8>>,
) {
    let Some(value) = output_json.as_mut() else {
        return;
    };
    let interface_passed = value
        .get("interface_proof")
        .and_then(|proof| proof.get("passed"))
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(false);
    if !interface_passed {
        return;
    }
    let final_module_path = value
        .get("final_module_artifact")
        .and_then(|artifact| artifact.get("artifact_path"))
        .and_then(serde_json::Value::as_str)
        .or_else(|| {
            value
                .get("linker_handoff")
                .and_then(|handoff| handoff.get("final_module_artifact"))
                .and_then(|artifact| artifact.get("artifact_path"))
                .and_then(serde_json::Value::as_str)
        })
        .map(normalize_reported_vfs_path)
        .unwrap_or_else(|| "rouwdi-codegen-wasm32-wasip1-linked.wasm".to_owned());
    let Some(module_bytes) = written_files.get(&final_module_path).cloned() else {
        return;
    };
    let module_hash = sha256_hex(&module_bytes);
    let runtime_proof = run_linked_wasi_runtime_proof(&module_bytes, &module_hash);
    let runtime_passed = runtime_proof
        .get("passed")
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(false);
    if let Some(object) = value.as_object_mut() {
        object.insert(
            "runtime_proof_attempted".to_owned(),
            serde_json::json!(true),
        );
        object.insert("runtime_proof".to_owned(), runtime_proof.clone());
        object.insert(
            "codegen_contact_state".to_owned(),
            serde_json::json!(if runtime_passed {
                "runtime_proof_passed"
            } else {
                "runtime_proof_attempted_failed"
            }),
        );
        object.insert(
            "blocker_kind".to_owned(),
            serde_json::json!(if runtime_passed {
                "none"
            } else {
                "runtime_proof_failed"
            }),
        );
        object.insert(
            "blocker_component".to_owned(),
            serde_json::json!(if runtime_passed {
                "none"
            } else {
                "rouwdi-wasm runtime proof"
            }),
        );
        object.insert(
            "blocker_reason".to_owned(),
            serde_json::json!(if runtime_passed {
                "none"
            } else {
                "final linked wasm32-wasip1 module executed with a non-success runtime result"
            }),
        );
        if let Some(handoff) = object
            .get_mut("linker_handoff")
            .and_then(serde_json::Value::as_object_mut)
        {
            handoff.insert(
                "runtime_proof_attempted".to_owned(),
                serde_json::json!(true),
            );
            handoff.insert("runtime_proof".to_owned(), runtime_proof);
            handoff.insert(
                "current_status".to_owned(),
                serde_json::json!(if runtime_passed {
                    "runtime_proof_passed"
                } else {
                    "runtime_proof_attempted_failed"
                }),
            );
            handoff.insert(
                "next_command".to_owned(),
                serde_json::json!(if runtime_passed {
                    "none"
                } else {
                    "fix the runtime proof failure for the final linked wasm32-wasip1 module"
                }),
            );
        }
    }
}

fn run_linked_wasi_runtime_proof(bytes: &[u8], module_hash: &str) -> serde_json::Value {
    let command_args: Vec<String> = Vec::new();
    let mut proof = serde_json::json!({
        "runtime_substrate": "rouwdi-wasm wasmi",
        "module_hash": module_hash,
        "command_args": command_args,
        "stdin": "",
        "stdout": "",
        "stderr": "",
        "exit_code": null,
        "timeout": false,
        "expected_result": {
            "exit_code": 0
        },
        "actual_result": null,
        "passed": false
    });

    let mut config = Config::default();
    config.consume_fuel(true);
    config.set_max_recursion_depth(4096);
    config.ignore_custom_sections(true);
    let engine = Engine::new(&config);
    let module = match Module::new(&engine, bytes) {
        Ok(module) => module,
        Err(error) => {
            proof["actual_result"] = serde_json::json!({
                "status": "module_compile_failed",
                "error": error.to_string(),
            });
            return proof;
        }
    };
    let mut store = Store::new(
        &engine,
        PayloadWasiState {
            args: vec!["rouwdi-codegen-wasm32-wasip1-linked.wasm".to_owned()],
            env: vec!["PWD=/".to_owned()],
            stdout: Vec::new(),
            stderr: Vec::new(),
            proc_exit_code: None,
            random_counter: 0,
            next_fd: WASI_PREOPEN_FD + 1,
            fds: BTreeMap::new(),
            written_files: BTreeMap::new(),
            created_dirs: BTreeSet::new(),
            wasi_trace: Vec::new(),
        },
    );
    let mut linker = Linker::<PayloadWasiState>::new(&engine);
    if let Err(error) = define_wasi_imports(&mut linker) {
        proof["actual_result"] = serde_json::json!({
            "status": "runtime_import_definition_failed",
            "error": error,
        });
        return proof;
    }
    let instance = match linker.instantiate_and_start(&mut store, &module) {
        Ok(instance) => instance,
        Err(error) => {
            proof["actual_result"] = serde_json::json!({
                "status": "module_instantiate_failed",
                "error": error.to_string(),
            });
            return proof;
        }
    };
    let start = match instance.get_typed_func::<(), ()>(&store, "_start") {
        Ok(start) => start,
        Err(error) => {
            proof["actual_result"] = serde_json::json!({
                "status": "start_export_missing",
                "error": error.to_string(),
            });
            return proof;
        }
    };
    let outcome = match call_start_with_resumable_fuel(&start, &mut store) {
        Ok(outcome) => outcome,
        Err(error) => {
            proof["actual_result"] = serde_json::json!({
                "status": "runtime_fuel_error",
                "error": error,
            });
            return proof;
        }
    };
    let stdout = String::from_utf8_lossy(&store.data().stdout).into_owned();
    let stderr = String::from_utf8_lossy(&store.data().stderr).into_owned();
    let exit_code = store.data().proc_exit_code.unwrap_or(outcome.status);
    let trapped = outcome.trap.is_some() && store.data().proc_exit_code.is_none();
    let passed = !trapped && exit_code == 0;
    proof["stdout"] = serde_json::json!(stdout);
    proof["stderr"] = serde_json::json!(stderr);
    proof["exit_code"] = serde_json::json!(exit_code);
    proof["actual_result"] = serde_json::json!({
        "status": if passed { "runtime_proof_passed" } else { "runtime_proof_failed" },
        "exit_code": exit_code,
        "trap": outcome.trap,
    });
    proof["passed"] = serde_json::json!(passed);
    proof
}

fn execute_embedded_payload(
    payload: &EmbeddedCompilerPayload,
) -> Result<EmbeddedCompilerPayloadLoadReport, String> {
    let actual_sha256 = sha256_hex(payload.bytes);
    let actual_size_bytes = payload.bytes.len() as u64;
    let hash_verified = actual_sha256 == payload.expected_sha256;
    let size_verified = actual_size_bytes == payload.expected_size_bytes;
    let wasm_magic_verified = payload.bytes.starts_with(b"\0asm");
    if !hash_verified || !size_verified || !wasm_magic_verified {
        return Err(format!(
            "embedded payload identity verification failed: hash_verified={hash_verified} size_verified={size_verified} wasm_magic_verified={wasm_magic_verified}"
        ));
    }

    let mut config = Config::default();
    config.consume_fuel(true);
    config.set_max_recursion_depth(4096);
    config.ignore_custom_sections(true);
    let engine = Engine::new(&config);
    let module = Module::new(&engine, payload.bytes)
        .map_err(|error| format!("failed to compile embedded Wasm payload: {error}"))?;
    trace_payload_loader("module compiled");
    let imports = module
        .imports()
        .map(|import| format!("{}::{}", import.module(), import.name()))
        .collect::<Vec<_>>();
    let exports = module
        .exports()
        .map(|export| export.name().to_owned())
        .collect::<Vec<_>>();
    let abi_v1_exports_verified = REQUIRED_ABI_EXPORTS
        .iter()
        .all(|required| exports.iter().any(|export| export == required));
    if !abi_v1_exports_verified {
        return Err(format!(
            "embedded payload missing ABI v1 exports; found [{}]",
            exports.join(", ")
        ));
    }

    let mut store = Store::new(
        &engine,
        PayloadWasiState {
            args: vec![PAYLOAD_ARG0.to_owned()],
            env: vec!["PWD=/".to_owned(), "RUST_BACKTRACE=1".to_owned()],
            stdout: Vec::new(),
            stderr: Vec::new(),
            proc_exit_code: None,
            random_counter: 0,
            next_fd: WASI_PREOPEN_FD + 1,
            fds: BTreeMap::new(),
            written_files: BTreeMap::new(),
            created_dirs: BTreeSet::new(),
            wasi_trace: Vec::new(),
        },
    );
    store
        .set_fuel(PAYLOAD_FUEL_CHUNK)
        .map_err(|error| error.to_string())?;
    let mut linker = Linker::<PayloadWasiState>::new(&engine);
    define_wasi_imports(&mut linker)?;
    let instance = linker
        .instantiate_and_start(&mut store, &module)
        .map_err(|error| format!("failed to instantiate embedded Wasm payload: {error}"))?;
    trace_payload_loader("module instantiated");
    let memory = instance
        .get_export(&store, "memory")
        .and_then(Extern::into_memory)
        .ok_or_else(|| "embedded payload did not export memory".to_owned())?;

    let version = call_u32_export(&instance, &mut store, ABI_VERSION_SYMBOL)?;
    let stage_code = call_u32_export(&instance, &mut store, ABI_STAGE_SYMBOL)?;
    let descriptor_ptr = call_u32_export(&instance, &mut store, ABI_DESCRIPTOR_PTR_SYMBOL)?;
    let descriptor_len = call_u32_export(&instance, &mut store, ABI_DESCRIPTOR_LEN_SYMBOL)?;
    let descriptor_json = read_guest_string(
        &memory,
        &store,
        descriptor_ptr,
        descriptor_len,
        "descriptor",
    )?;
    trace_payload_loader("descriptor read");
    let valid_input_ptr = call_u32_export(&instance, &mut store, MIR_VALID_INPUT_PTR_SYMBOL)?;
    let valid_input_len = call_u32_export(&instance, &mut store, MIR_VALID_INPUT_LEN_SYMBOL)?;
    let valid_input_json = read_guest_string(
        &memory,
        &store,
        valid_input_ptr,
        valid_input_len,
        "valid input",
    )?;
    trace_payload_loader("valid input read");
    let result_area_ptr = call_u32_export(&instance, &mut store, MIR_RESULT_AREA_PTR_SYMBOL)?;
    let output_ptr_slot = result_area_ptr;
    let output_len_slot = result_area_ptr
        .checked_add(4)
        .ok_or_else(|| "result area output len slot overflowed".to_owned())?;
    let error_ptr_slot = result_area_ptr
        .checked_add(8)
        .ok_or_else(|| "result area error ptr slot overflowed".to_owned())?;
    let error_len_slot = result_area_ptr
        .checked_add(12)
        .ok_or_else(|| "result area error len slot overflowed".to_owned())?;
    let execute = instance
        .get_typed_func::<(i32, i32, i32, i32, i32, i32), i32>(&store, MIR_EXECUTE_SYMBOL)
        .map_err(|error| format!("missing/corrupt execute export: {error}"))?;
    let execute_outcome = call_execute_with_resumable_fuel(
        &execute,
        &mut store,
        (
            valid_input_ptr as i32,
            valid_input_len as i32,
            output_ptr_slot as i32,
            output_len_slot as i32,
            error_ptr_slot as i32,
            error_len_slot as i32,
        ),
    )?;
    let execute_status = execute_outcome.status;
    trace_payload_loader("execute returned");

    let output_ptr = read_guest_u32(&memory, &store, output_ptr_slot, "output ptr slot")?;
    let output_len = read_guest_u32(&memory, &store, output_len_slot, "output len slot")?;
    let error_ptr = read_guest_u32(&memory, &store, error_ptr_slot, "error ptr slot")?;
    let error_len = read_guest_u32(&memory, &store, error_len_slot, "error len slot")?;
    let output_json = if output_len > 0 {
        Some(read_guest_string(
            &memory,
            &store,
            output_ptr,
            output_len,
            "execute output",
        )?)
    } else {
        None
    };
    let error_json = if error_len > 0 {
        Some(read_guest_string(
            &memory,
            &store,
            error_ptr,
            error_len,
            "execute error",
        )?)
    } else if output_json.is_some() {
        None
    } else {
        let last_error_ptr = call_u32_export(&instance, &mut store, MIR_LAST_ERROR_PTR_SYMBOL)?;
        let last_error_len = call_u32_export(&instance, &mut store, MIR_LAST_ERROR_LEN_SYMBOL)?;
        (last_error_len > 0)
            .then(|| {
                read_guest_string(
                    &memory,
                    &store,
                    last_error_ptr,
                    last_error_len,
                    "last error",
                )
            })
            .transpose()?
    }
    .or_else(|| {
        execute_outcome.trap.as_ref().map(|trap| {
            format!(
                "{{\"code\":\"embedded_payload_execute_trapped\",\"kind\":\"wasm_trap\",\"message\":\"embedded payload execute trapped inside rouwdi-owned wasmi runtime\",\"blocker_kind\":\"embedded_payload_execute_trapped\",\"blocker_component\":\"{MIR_EXECUTE_SYMBOL}\",\"context_state\":\"embedded_payload_execute_trapped\",\"execute_trap\":{}}}",
                json_string(trap)
            )
        })
    });
    let evidence_json = output_json
        .as_deref()
        .or(error_json.as_deref())
        .unwrap_or("");
    let evidence_value = serde_json::from_str::<serde_json::Value>(evidence_json).ok();
    let descriptor_value = serde_json::from_str::<serde_json::Value>(&descriptor_json).ok();
    let raw_state = json_str(&evidence_value, "context_state")
        .or_else(|| json_str(&evidence_value, "code"))
        .or_else(|| json_str(&descriptor_value, "bridge_state"))
        .unwrap_or("unknown_payload_execution_result");
    let blocker_kind = json_str(&evidence_value, "blocker_kind").map(str::to_owned);
    let execution_state = canonical_execution_state(raw_state, blocker_kind.as_deref());
    let result_kind = if output_json.is_some() {
        "output"
    } else if error_json.is_some() {
        "error"
    } else {
        "empty"
    }
    .to_owned();
    let stdout_bytes = store.data().stdout.len();
    let stderr_bytes = store.data().stderr.len();
    let stdout_text = String::from_utf8_lossy(&store.data().stdout).into_owned();
    let stderr_text = String::from_utf8_lossy(&store.data().stderr).into_owned();

    Ok(EmbeddedCompilerPayloadLoadReport {
        name: payload.name.to_owned(),
        kind: payload.kind.to_owned(),
        stage: payload.stage.to_owned(),
        abi_name: payload.abi_name.to_owned(),
        abi_version: payload.abi_version,
        registry_identity: payload.name.to_owned(),
        execution_source: "embedded_registry".to_owned(),
        external: false,
        opened_external_file: false,
        build_source_path: payload.build_source_path.to_owned(),
        load_strategy: payload.load_strategy.to_owned(),
        embedding_method: payload.embedding_method.to_owned(),
        expected_sha256: payload.expected_sha256.to_owned(),
        actual_sha256,
        hash_verified,
        expected_size_bytes: payload.expected_size_bytes,
        actual_size_bytes,
        size_verified,
        wasm_magic_verified,
        module_instantiated: true,
        imports,
        exports,
        abi_v1_exports_verified,
        version_called: true,
        version,
        stage_called: true,
        stage_code,
        descriptor_ptr,
        descriptor_len,
        descriptor_bytes_read: true,
        descriptor_json,
        valid_input_ptr,
        valid_input_len,
        valid_input_bytes_read: true,
        valid_input_json: valid_input_json.clone(),
        execute_called: true,
        execute_status,
        execute_trapped: execute_outcome.trap.is_some(),
        execute_trap: execute_outcome.trap,
        output_ptr,
        output_len,
        error_ptr,
        error_len,
        output_bytes_read: output_json.is_some(),
        output_contract_sha256: output_json
            .as_deref()
            .map(|json| sha256_hex(json.as_bytes())),
        output_json,
        error_bytes_read: error_json.is_some(),
        error_contract_sha256: error_json
            .as_deref()
            .map(|json| sha256_hex(json.as_bytes())),
        error_json,
        input_contract_sha256: sha256_hex(valid_input_json.as_bytes()),
        execution_state,
        blocker_kind,
        result_kind,
        stdout_bytes,
        stderr_bytes,
        stdout_text,
        stderr_text,
    })
}

fn define_wasi_imports(linker: &mut Linker<PayloadWasiState>) -> Result<(), String> {
    linker
        .func_wrap(WASI, "args_sizes_get", wasi_args_sizes_get)
        .map_err(to_string)?;
    linker
        .func_wrap(WASI, "args_get", wasi_args_get)
        .map_err(to_string)?;
    linker
        .func_wrap(WASI, "environ_sizes_get", wasi_environ_sizes_get)
        .map_err(to_string)?;
    linker
        .func_wrap(WASI, "environ_get", wasi_environ_get)
        .map_err(to_string)?;
    linker
        .func_wrap(WASI, "clock_time_get", wasi_clock_time_get)
        .map_err(to_string)?;
    linker
        .func_wrap(WASI, "random_get", wasi_random_get)
        .map_err(to_string)?;
    linker
        .func_wrap(WASI, "poll_oneoff", wasi_poll_oneoff)
        .map_err(to_string)?;
    linker
        .func_wrap(WASI, "fd_write", wasi_fd_write)
        .map_err(to_string)?;
    linker
        .func_wrap(WASI, "fd_read", wasi_fd_read)
        .map_err(to_string)?;
    linker
        .func_wrap(WASI, "fd_pread", wasi_fd_pread)
        .map_err(to_string)?;
    linker
        .func_wrap(WASI, "fd_close", wasi_fd_close)
        .map_err(to_string)?;
    linker
        .func_wrap(WASI, "fd_fdstat_get", wasi_fd_fdstat_get)
        .map_err(to_string)?;
    linker
        .func_wrap(WASI, "fd_fdstat_set_flags", wasi_fd_fdstat_set_flags)
        .map_err(to_string)?;
    linker
        .func_wrap(WASI, "fd_filestat_get", wasi_fd_filestat_get)
        .map_err(to_string)?;
    linker
        .func_wrap(WASI, "fd_filestat_set_size", wasi_fd_filestat_set_size)
        .map_err(to_string)?;
    linker
        .func_wrap(WASI, "fd_prestat_get", wasi_fd_prestat_get)
        .map_err(to_string)?;
    linker
        .func_wrap(WASI, "fd_prestat_dir_name", wasi_fd_prestat_dir_name)
        .map_err(to_string)?;
    linker
        .func_wrap(WASI, "fd_readdir", wasi_fd_readdir)
        .map_err(to_string)?;
    linker
        .func_wrap(WASI, "fd_seek", wasi_fd_seek)
        .map_err(to_string)?;
    linker
        .func_wrap(WASI, "path_create_directory", wasi_path_create_directory)
        .map_err(to_string)?;
    linker
        .func_wrap(WASI, "path_filestat_get", wasi_path_filestat_get)
        .map_err(to_string)?;
    linker
        .func_wrap(WASI, "path_link", wasi_path_link)
        .map_err(to_string)?;
    linker
        .func_wrap(WASI, "path_open", wasi_path_open)
        .map_err(to_string)?;
    linker
        .func_wrap(WASI, "path_readlink", wasi_path_readlink)
        .map_err(to_string)?;
    linker
        .func_wrap(WASI, "path_remove_directory", wasi_path_remove_directory)
        .map_err(to_string)?;
    linker
        .func_wrap(WASI, "path_rename", wasi_path_rename)
        .map_err(to_string)?;
    linker
        .func_wrap(WASI, "path_unlink_file", wasi_path_unlink_file)
        .map_err(to_string)?;
    linker
        .func_wrap(WASI, "proc_exit", wasi_proc_exit)
        .map_err(to_string)?;
    linker
        .func_wrap(WASI, "sched_yield", wasi_sched_yield)
        .map_err(to_string)?;
    Ok(())
}

fn wasi_args_sizes_get(
    mut caller: Caller<'_, PayloadWasiState>,
    argc_ptr: i32,
    argv_buf_size_ptr: i32,
) -> i32 {
    let args = caller.data().args.clone();
    let status = write_u32(&mut caller, argc_ptr, args.len() as u32);
    if status != WASI_ERRNO_SUCCESS {
        return status;
    }
    let argv_buf_size = args
        .iter()
        .map(|arg| arg.len().saturating_add(1))
        .sum::<usize>();
    write_u32(&mut caller, argv_buf_size_ptr, argv_buf_size as u32)
}

fn wasi_args_get(
    mut caller: Caller<'_, PayloadWasiState>,
    argv_ptr: i32,
    argv_buf_ptr: i32,
) -> i32 {
    let args = caller.data().args.clone();
    let mut current_buf_ptr = argv_buf_ptr;
    for (index, arg) in args.iter().enumerate() {
        let bytes = arg.as_bytes();
        let pointer_slot = argv_ptr + (index as i32 * 4);
        let status = write_u32(&mut caller, pointer_slot, current_buf_ptr as u32);
        if status != WASI_ERRNO_SUCCESS {
            return status;
        }
        let status = write_bytes(&mut caller, current_buf_ptr, bytes);
        if status != WASI_ERRNO_SUCCESS {
            return status;
        }
        let terminator_ptr = current_buf_ptr + bytes.len() as i32;
        let status = write_bytes(&mut caller, terminator_ptr, &[0]);
        if status != WASI_ERRNO_SUCCESS {
            return status;
        }
        current_buf_ptr = terminator_ptr + 1;
    }
    WASI_ERRNO_SUCCESS
}

fn wasi_environ_sizes_get(
    mut caller: Caller<'_, PayloadWasiState>,
    count_ptr: i32,
    size_ptr: i32,
) -> i32 {
    let env = caller.data().env.clone();
    let status = write_u32(&mut caller, count_ptr, env.len() as u32);
    if status != WASI_ERRNO_SUCCESS {
        return status;
    }
    let env_buf_size = env
        .iter()
        .map(|entry| entry.len().saturating_add(1))
        .sum::<usize>();
    write_u32(&mut caller, size_ptr, env_buf_size as u32)
}

fn wasi_environ_get(
    mut caller: Caller<'_, PayloadWasiState>,
    env_ptr: i32,
    env_buf_ptr: i32,
) -> i32 {
    let env = caller.data().env.clone();
    let mut current_buf_ptr = env_buf_ptr;
    for (index, entry) in env.iter().enumerate() {
        let bytes = entry.as_bytes();
        let pointer_slot = env_ptr + (index as i32 * 4);
        let status = write_u32(&mut caller, pointer_slot, current_buf_ptr as u32);
        if status != WASI_ERRNO_SUCCESS {
            return status;
        }
        let status = write_bytes(&mut caller, current_buf_ptr, bytes);
        if status != WASI_ERRNO_SUCCESS {
            return status;
        }
        let terminator_ptr = current_buf_ptr + bytes.len() as i32;
        let status = write_bytes(&mut caller, terminator_ptr, &[0]);
        if status != WASI_ERRNO_SUCCESS {
            return status;
        }
        current_buf_ptr = terminator_ptr + 1;
    }
    WASI_ERRNO_SUCCESS
}

fn wasi_clock_time_get(
    mut caller: Caller<'_, PayloadWasiState>,
    _clock_id: i32,
    _precision: i64,
    time_ptr: i32,
) -> i32 {
    write_u64(&mut caller, time_ptr, 0)
}

fn wasi_random_get(mut caller: Caller<'_, PayloadWasiState>, ptr: i32, len: i32) -> i32 {
    if ptr < 0 || len < 0 {
        return WASI_ERRNO_INVAL;
    }
    let mut bytes = vec![0_u8; len as usize];
    for byte in &mut bytes {
        let next = caller.data().random_counter.wrapping_add(1);
        caller.data_mut().random_counter = next;
        *byte = next;
    }
    write_bytes(&mut caller, ptr, &bytes)
}

fn wasi_poll_oneoff(
    mut caller: Caller<'_, PayloadWasiState>,
    subscriptions_ptr: i32,
    events_ptr: i32,
    subscriptions_len: i32,
    events_len_ptr: i32,
) -> i32 {
    if subscriptions_ptr < 0 || events_ptr < 0 || subscriptions_len <= 0 {
        return WASI_ERRNO_INVAL;
    }
    let Some(memory) = caller_memory(&caller) else {
        return WASI_ERRNO_INVAL;
    };
    let mut userdata = [0_u8; 8];
    if memory
        .read(&caller, subscriptions_ptr as usize, &mut userdata)
        .is_err()
    {
        return WASI_ERRNO_INVAL;
    }

    // event { userdata: u64, error: errno, type: eventtype, fd_readwrite: zeroed }
    let mut event = [0_u8; 32];
    event[0..8].copy_from_slice(&userdata);
    let status = write_bytes(&mut caller, events_ptr, &event);
    if status != WASI_ERRNO_SUCCESS {
        return status;
    }
    write_u32(&mut caller, events_len_ptr, 1)
}

fn wasi_fd_write(
    mut caller: Caller<'_, PayloadWasiState>,
    fd: i32,
    iovs_ptr: i32,
    iovs_len: i32,
    nwritten_ptr: i32,
) -> i32 {
    if iovs_ptr < 0 || iovs_len < 0 {
        return WASI_ERRNO_INVAL;
    }
    let Some(memory) = caller_memory(&caller) else {
        return WASI_ERRNO_INVAL;
    };
    let mut written = 0_u32;
    let mut chunks = Vec::new();
    for index in 0..iovs_len {
        let base = iovs_ptr as usize + (index as usize * 8);
        let Ok(ptr) = read_memory_u32(&memory, &caller, base) else {
            return WASI_ERRNO_INVAL;
        };
        let Ok(len) = read_memory_u32(&memory, &caller, base + 4) else {
            return WASI_ERRNO_INVAL;
        };
        let mut bytes = vec![0_u8; len as usize];
        if memory.read(&caller, ptr as usize, &mut bytes).is_err() {
            return WASI_ERRNO_INVAL;
        }
        written = written.saturating_add(len);
        chunks.extend_from_slice(&bytes);
    }
    match fd {
        1 => caller.data_mut().stdout.extend_from_slice(&chunks),
        2 => caller.data_mut().stderr.extend_from_slice(&chunks),
        _ => {
            let Some((path, position)) = caller.data().fds.get(&fd).and_then(|fd| match fd {
                VirtualFd::WrittenFile { path, position } => Some((path.clone(), *position)),
                _ => None,
            }) else {
                return WASI_ERRNO_BADF;
            };
            let file = caller
                .data_mut()
                .written_files
                .entry(path.clone())
                .or_default();
            let start = position as usize;
            let end = start.saturating_add(chunks.len());
            if file.len() < end {
                file.resize(end, 0);
            }
            file[start..end].copy_from_slice(&chunks);
            if let Some(VirtualFd::WrittenFile { position, .. }) =
                caller.data_mut().fds.get_mut(&fd)
            {
                *position = end as u64;
            }
            record_wasi_trace(
                &mut caller,
                format!("fd_write fd={fd} path={path:?} bytes={}", chunks.len()),
            );
        }
    }
    write_u32(&mut caller, nwritten_ptr, written)
}

fn wasi_fd_read(
    mut caller: Caller<'_, PayloadWasiState>,
    fd: i32,
    iovs_ptr: i32,
    iovs_len: i32,
    nread_ptr: i32,
) -> i32 {
    if iovs_ptr < 0 || iovs_len < 0 {
        return WASI_ERRNO_INVAL;
    }
    if fd == 0 || fd == WASI_PREOPEN_FD {
        return write_u32(&mut caller, nread_ptr, 0);
    }
    let Some(memory) = caller_memory(&caller) else {
        return WASI_ERRNO_INVAL;
    };
    let mut iovs = Vec::new();
    for index in 0..iovs_len {
        let base = iovs_ptr as usize + (index as usize * 8);
        let Ok(ptr) = read_memory_u32(&memory, &caller, base) else {
            return WASI_ERRNO_INVAL;
        };
        let Ok(len) = read_memory_u32(&memory, &caller, base + 4) else {
            return WASI_ERRNO_INVAL;
        };
        iovs.push((ptr, len));
    }

    let mut writes = Vec::<(u32, Vec<u8>)>::new();
    let mut total_read = 0_u32;
    {
        let (bytes, mut position) = match caller.data().fds.get(&fd) {
            Some(VirtualFd::File { bytes, position }) => (bytes.to_vec(), *position),
            Some(VirtualFd::WrittenFile { path, position }) => {
                let bytes = caller
                    .data()
                    .written_files
                    .get(path)
                    .cloned()
                    .unwrap_or_default();
                (bytes, *position)
            }
            _ => return WASI_ERRNO_BADF,
        };
        for (ptr, len) in iovs {
            let start = (position as usize).min(bytes.len());
            let requested_end = start.saturating_add(len as usize).min(bytes.len());
            let chunk = bytes[start..requested_end].to_vec();
            position = requested_end as u64;
            total_read = total_read.saturating_add(chunk.len() as u32);
            writes.push((ptr, chunk));
            if total_read == 0 || requested_end == bytes.len() {
                break;
            }
        }
        match caller.data_mut().fds.get_mut(&fd) {
            Some(VirtualFd::File {
                position: file_position,
                ..
            })
            | Some(VirtualFd::WrittenFile {
                position: file_position,
                ..
            }) => *file_position = position,
            _ => {}
        }
    }

    for (ptr, bytes) in writes {
        if memory.write(&mut caller, ptr as usize, &bytes).is_err() {
            return WASI_ERRNO_INVAL;
        }
    }
    write_u32(&mut caller, nread_ptr, total_read)
}

fn wasi_fd_pread(
    mut caller: Caller<'_, PayloadWasiState>,
    fd: i32,
    iovs_ptr: i32,
    iovs_len: i32,
    offset: i64,
    nread_ptr: i32,
) -> i32 {
    if iovs_ptr < 0 || iovs_len < 0 || offset < 0 {
        return WASI_ERRNO_INVAL;
    }
    if fd == 0 || fd == WASI_PREOPEN_FD {
        return write_u32(&mut caller, nread_ptr, 0);
    }
    let Some(memory) = caller_memory(&caller) else {
        return WASI_ERRNO_INVAL;
    };
    let mut iovs = Vec::new();
    for index in 0..iovs_len {
        let base = iovs_ptr as usize + (index as usize * 8);
        let Ok(ptr) = read_memory_u32(&memory, &caller, base) else {
            return WASI_ERRNO_INVAL;
        };
        let Ok(len) = read_memory_u32(&memory, &caller, base + 4) else {
            return WASI_ERRNO_INVAL;
        };
        iovs.push((ptr, len));
    }

    let bytes = match caller.data().fds.get(&fd) {
        Some(VirtualFd::File { bytes, .. }) => bytes.to_vec(),
        Some(VirtualFd::WrittenFile { path, .. }) => caller
            .data()
            .written_files
            .get(path)
            .cloned()
            .unwrap_or_default(),
        _ => return WASI_ERRNO_BADF,
    };
    let mut position = offset as usize;
    let mut writes = Vec::<(u32, Vec<u8>)>::new();
    let mut total_read = 0_u32;
    for (ptr, len) in iovs {
        let start = position.min(bytes.len());
        let requested_end = start.saturating_add(len as usize).min(bytes.len());
        let chunk = bytes[start..requested_end].to_vec();
        position = requested_end;
        total_read = total_read.saturating_add(chunk.len() as u32);
        writes.push((ptr, chunk));
        if total_read == 0 || requested_end == bytes.len() {
            break;
        }
    }

    for (ptr, bytes) in writes {
        if memory.write(&mut caller, ptr as usize, &bytes).is_err() {
            return WASI_ERRNO_INVAL;
        }
    }
    write_u32(&mut caller, nread_ptr, total_read)
}

fn wasi_fd_close(mut caller: Caller<'_, PayloadWasiState>, fd: i32) -> i32 {
    record_wasi_trace(&mut caller, format!("fd_close fd={fd}"));
    if fd == WASI_PREOPEN_FD {
        return WASI_ERRNO_SUCCESS;
    }
    if fd >= WASI_PREOPEN_FD && caller.data_mut().fds.remove(&fd).is_some() {
        WASI_ERRNO_SUCCESS
    } else {
        return WASI_ERRNO_BADF;
    }
}

fn wasi_fd_fdstat_get(mut caller: Caller<'_, PayloadWasiState>, fd: i32, stat_ptr: i32) -> i32 {
    record_wasi_trace(&mut caller, format!("fd_fdstat_get fd={fd}"));
    let filetype = match fd {
        0..=2 => WASI_FILETYPE_CHARACTER_DEVICE,
        WASI_PREOPEN_FD => WASI_FILETYPE_DIRECTORY,
        _ => match caller.data().fds.get(&fd) {
            Some(VirtualFd::File { .. }) => WASI_FILETYPE_REGULAR_FILE,
            Some(VirtualFd::WrittenFile { .. }) => WASI_FILETYPE_REGULAR_FILE,
            Some(VirtualFd::Directory { .. }) => WASI_FILETYPE_DIRECTORY,
            None => return WASI_ERRNO_BADF,
        },
    };
    let mut stat = [0_u8; 24];
    stat[0] = filetype;
    stat[8..16].copy_from_slice(&u64::MAX.to_le_bytes());
    stat[16..24].copy_from_slice(&u64::MAX.to_le_bytes());
    let status = write_bytes(&mut caller, stat_ptr, &stat);
    record_wasi_trace(
        &mut caller,
        format!("fd_fdstat_get result fd={fd} filetype={filetype} status={status}"),
    );
    status
}

fn wasi_fd_fdstat_set_flags(mut caller: Caller<'_, PayloadWasiState>, fd: i32, flags: i32) -> i32 {
    record_wasi_trace(
        &mut caller,
        format!("fd_fdstat_set_flags fd={fd} flags={flags}"),
    );
    if matches!(fd, 0..=2 | WASI_PREOPEN_FD) || caller.data().fds.contains_key(&fd) {
        WASI_ERRNO_SUCCESS
    } else {
        WASI_ERRNO_BADF
    }
}

fn wasi_fd_filestat_get(mut caller: Caller<'_, PayloadWasiState>, fd: i32, stat_ptr: i32) -> i32 {
    record_wasi_trace(&mut caller, format!("fd_filestat_get fd={fd}"));
    let (filetype, size) = match fd {
        0..=2 => (WASI_FILETYPE_CHARACTER_DEVICE, 0),
        WASI_PREOPEN_FD => (WASI_FILETYPE_DIRECTORY, 0),
        _ => match caller.data().fds.get(&fd) {
            Some(VirtualFd::File { bytes, .. }) => (WASI_FILETYPE_REGULAR_FILE, bytes.len() as u64),
            Some(VirtualFd::WrittenFile { path, .. }) => (
                WASI_FILETYPE_REGULAR_FILE,
                caller
                    .data()
                    .written_files
                    .get(path)
                    .map(|bytes| bytes.len() as u64)
                    .unwrap_or(0),
            ),
            Some(VirtualFd::Directory { .. }) => (WASI_FILETYPE_DIRECTORY, 0),
            None => return WASI_ERRNO_BADF,
        },
    };
    let status = write_filestat(&mut caller, stat_ptr, filetype, size);
    record_wasi_trace(
        &mut caller,
        format!("fd_filestat_get result fd={fd} filetype={filetype} size={size} status={status}"),
    );
    status
}

fn wasi_fd_filestat_set_size(mut caller: Caller<'_, PayloadWasiState>, fd: i32, size: i64) -> i32 {
    if size < 0 {
        return WASI_ERRNO_INVAL;
    }
    let Some(path) = caller
        .data()
        .fds
        .get(&fd)
        .and_then(|virtual_fd| match virtual_fd {
            VirtualFd::WrittenFile { path, .. } => Some(path.clone()),
            _ => None,
        })
    else {
        return WASI_ERRNO_BADF;
    };
    let size = size as usize;
    caller
        .data_mut()
        .written_files
        .entry(path.clone())
        .or_default()
        .resize(size, 0);
    if let Some(VirtualFd::WrittenFile { position, .. }) = caller.data_mut().fds.get_mut(&fd) {
        *position = (*position).min(size as u64);
    }
    record_wasi_trace(
        &mut caller,
        format!("fd_filestat_set_size fd={fd} path={path:?} size={size}"),
    );
    WASI_ERRNO_SUCCESS
}

fn wasi_fd_prestat_get(mut caller: Caller<'_, PayloadWasiState>, fd: i32, prestat_ptr: i32) -> i32 {
    if fd != WASI_PREOPEN_FD {
        return WASI_ERRNO_BADF;
    }
    let mut prestat = [0_u8; 8];
    prestat[4..8].copy_from_slice(&(WASI_PREOPEN_PATH.len() as u32).to_le_bytes());
    write_bytes(&mut caller, prestat_ptr, &prestat)
}

fn wasi_fd_prestat_dir_name(
    mut caller: Caller<'_, PayloadWasiState>,
    fd: i32,
    path_ptr: i32,
    path_len: i32,
) -> i32 {
    if fd != WASI_PREOPEN_FD || path_len < 0 {
        return WASI_ERRNO_BADF;
    }
    let bytes = WASI_PREOPEN_PATH.as_bytes();
    if path_len as usize > bytes.len() {
        return WASI_ERRNO_INVAL;
    }
    write_bytes(&mut caller, path_ptr, &bytes[..path_len as usize])
}

fn wasi_fd_readdir(
    mut caller: Caller<'_, PayloadWasiState>,
    fd: i32,
    buf: i32,
    buf_len: i32,
    cookie: i64,
    bufused_ptr: i32,
) -> i32 {
    if buf < 0 || buf_len < 0 || cookie < 0 {
        return WASI_ERRNO_INVAL;
    }
    let entries = if fd == WASI_PREOPEN_FD {
        virtual_dir_entries("")
    } else {
        let Some(VirtualFd::Directory { entries, .. }) = caller.data().fds.get(&fd) else {
            return WASI_ERRNO_BADF;
        };
        entries.clone()
    };
    let Some(memory) = caller_memory(&caller) else {
        return WASI_ERRNO_INVAL;
    };
    let mut cursor = buf as usize;
    let limit = (buf as usize).saturating_add(buf_len as usize);
    let mut used = 0_u32;
    for (index, entry) in entries.into_iter().enumerate().skip(cookie as usize) {
        let name_bytes = entry.name.as_bytes();
        let record_len = 24_usize.saturating_add(name_bytes.len());
        if cursor.saturating_add(record_len) > limit {
            break;
        }
        let mut dirent = [0_u8; 24];
        dirent[0..8].copy_from_slice(&((index + 1) as u64).to_le_bytes());
        dirent[8..16].copy_from_slice(&((index + 1) as u64).to_le_bytes());
        dirent[16..20].copy_from_slice(&(name_bytes.len() as u32).to_le_bytes());
        dirent[20] = entry.filetype;
        if memory.write(&mut caller, cursor, &dirent).is_err()
            || memory
                .write(&mut caller, cursor + dirent.len(), name_bytes)
                .is_err()
        {
            return WASI_ERRNO_INVAL;
        }
        cursor += record_len;
        used = used.saturating_add(record_len as u32);
    }
    write_u32(&mut caller, bufused_ptr, used)
}

fn wasi_fd_seek(
    mut caller: Caller<'_, PayloadWasiState>,
    fd: i32,
    offset: i64,
    whence: i32,
    newoffset_ptr: i32,
) -> i32 {
    if fd == WASI_PREOPEN_FD {
        return write_u64(&mut caller, newoffset_ptr, 0);
    }
    let file_size = match caller.data().fds.get(&fd) {
        Some(VirtualFd::File { bytes, .. }) => bytes.len() as i128,
        Some(VirtualFd::WrittenFile { path, .. }) => caller
            .data()
            .written_files
            .get(path)
            .map(|bytes| bytes.len() as i128)
            .unwrap_or(0),
        _ => return WASI_ERRNO_BADF,
    };
    let new_position = {
        let position = match caller.data_mut().fds.get_mut(&fd) {
            Some(VirtualFd::File { position, .. })
            | Some(VirtualFd::WrittenFile { position, .. }) => position,
            _ => return WASI_ERRNO_BADF,
        };
        let base = match whence {
            0 => 0_i128,
            1 => *position as i128,
            2 => file_size,
            _ => return WASI_ERRNO_INVAL,
        };
        let next = base + offset as i128;
        if next < 0 {
            return WASI_ERRNO_INVAL;
        }
        *position = next as u64;
        *position
    };
    write_u64(&mut caller, newoffset_ptr, new_position)
}

fn wasi_path_create_directory(
    mut caller: Caller<'_, PayloadWasiState>,
    fd: i32,
    path_ptr: i32,
    path_len: i32,
) -> i32 {
    if fd != WASI_PREOPEN_FD {
        record_wasi_trace(
            &mut caller,
            format!("path_create_directory rejected_bad_fd fd={fd} path_len={path_len}"),
        );
        return WASI_ERRNO_BADF;
    }
    let Some(path) = read_path(&caller, path_ptr, path_len) else {
        record_wasi_trace(
            &mut caller,
            format!(
                "path_create_directory rejected_invalid_path path_ptr={path_ptr} path_len={path_len}"
            ),
        );
        return WASI_ERRNO_INVAL;
    };
    let normalized = normalize_virtual_path(&path);
    if normalized.is_empty()
        || virtual_dir_exists(&normalized)
        || caller.data().created_dirs.contains(&normalized)
    {
        return WASI_ERRNO_SUCCESS;
    }
    caller.data_mut().created_dirs.insert(normalized);
    WASI_ERRNO_SUCCESS
}

fn wasi_path_filestat_get(
    mut caller: Caller<'_, PayloadWasiState>,
    fd: i32,
    _flags: i32,
    path_ptr: i32,
    path_len: i32,
    stat_ptr: i32,
) -> i32 {
    if fd != WASI_PREOPEN_FD {
        return WASI_ERRNO_BADF;
    }
    let Some(path) = read_path(&caller, path_ptr, path_len) else {
        return WASI_ERRNO_INVAL;
    };
    let normalized = normalize_virtual_path(&path);
    if virtual_dir_exists(&normalized) || caller.data().created_dirs.contains(&normalized) {
        write_filestat(&mut caller, stat_ptr, WASI_FILETYPE_DIRECTORY, 0)
    } else if let Some(file) = virtual_file(&normalized) {
        write_filestat(
            &mut caller,
            stat_ptr,
            WASI_FILETYPE_REGULAR_FILE,
            file.bytes.len() as u64,
        )
    } else if let Some(size) = caller
        .data()
        .written_files
        .get(&normalized)
        .map(|bytes| bytes.len() as u64)
    {
        write_filestat(&mut caller, stat_ptr, WASI_FILETYPE_REGULAR_FILE, size)
    } else {
        WASI_ERRNO_NOENT
    }
}

fn wasi_path_link(
    _caller: Caller<'_, PayloadWasiState>,
    _old_fd: i32,
    _old_flags: i32,
    _old_path_ptr: i32,
    _old_path_len: i32,
    _new_fd: i32,
    _new_path_ptr: i32,
    _new_path_len: i32,
) -> i32 {
    WASI_ERRNO_NOSYS
}

fn wasi_path_open(
    mut caller: Caller<'_, PayloadWasiState>,
    fd: i32,
    _dirflags: i32,
    path_ptr: i32,
    path_len: i32,
    oflags: i32,
    rights_base: i64,
    rights_inheriting: i64,
    fdflags: i32,
    opened_fd_ptr: i32,
) -> i32 {
    if fd != WASI_PREOPEN_FD {
        return WASI_ERRNO_BADF;
    }
    let Some(path) = read_path(&caller, path_ptr, path_len) else {
        return WASI_ERRNO_INVAL;
    };
    let normalized = normalize_virtual_path(&path);
    record_wasi_trace(
        &mut caller,
        format!(
            "path_open fd={fd} path={path:?} normalized={normalized:?} oflags={oflags} rights_base={rights_base} rights_inheriting={rights_inheriting} fdflags={fdflags}"
        ),
    );
    let next_fd = caller.data().next_fd;
    if oflags & WASI_OFLAGS_DIRECTORY != 0 {
        if !virtual_dir_exists(&normalized) && !caller.data().created_dirs.contains(&normalized) {
            let _ = write_u32(&mut caller, opened_fd_ptr, 0);
            record_wasi_trace(
                &mut caller,
                format!("path_open directory_missing normalized={normalized:?}"),
            );
            return WASI_ERRNO_NOENT;
        }
    }
    if let Some(file) = virtual_file(&normalized) {
        caller.data_mut().next_fd = next_fd.saturating_add(1);
        caller.data_mut().fds.insert(
            next_fd,
            VirtualFd::File {
                bytes: file.bytes,
                position: 0,
            },
        );
        record_wasi_trace(
            &mut caller,
            format!("path_open opened_virtual_file fd={next_fd} normalized={normalized:?}"),
        );
        let status = write_u32(&mut caller, opened_fd_ptr, next_fd as u32);
        record_wasi_trace(
            &mut caller,
            format!("path_open result fd={next_fd} status={status}"),
        );
        return status;
    }
    if caller.data().written_files.contains_key(&normalized) {
        let opened_path = normalized.clone();
        if oflags & WASI_OFLAGS_TRUNC != 0 {
            caller
                .data_mut()
                .written_files
                .insert(normalized.clone(), Vec::new());
        }
        caller.data_mut().next_fd = next_fd.saturating_add(1);
        caller.data_mut().fds.insert(
            next_fd,
            VirtualFd::WrittenFile {
                path: opened_path.clone(),
                position: 0,
            },
        );
        record_wasi_trace(
            &mut caller,
            format!("path_open opened_written_file fd={next_fd} normalized={opened_path:?}"),
        );
        let status = write_u32(&mut caller, opened_fd_ptr, next_fd as u32);
        record_wasi_trace(
            &mut caller,
            format!("path_open result fd={next_fd} status={status}"),
        );
        return status;
    }
    if virtual_dir_exists(&normalized) || caller.data().created_dirs.contains(&normalized) {
        caller.data_mut().next_fd = next_fd.saturating_add(1);
        caller.data_mut().fds.insert(
            next_fd,
            VirtualFd::Directory {
                entries: virtual_dir_entries(&normalized),
            },
        );
        record_wasi_trace(
            &mut caller,
            format!("path_open opened_directory fd={next_fd} normalized={normalized:?}"),
        );
        let status = write_u32(&mut caller, opened_fd_ptr, next_fd as u32);
        record_wasi_trace(
            &mut caller,
            format!("path_open result fd={next_fd} status={status}"),
        );
        return status;
    }
    if oflags & WASI_OFLAGS_CREAT != 0 {
        let created_path = normalized.clone();
        caller
            .data_mut()
            .written_files
            .insert(created_path.clone(), Vec::new());
        caller.data_mut().next_fd = next_fd.saturating_add(1);
        caller.data_mut().fds.insert(
            next_fd,
            VirtualFd::WrittenFile {
                path: created_path.clone(),
                position: 0,
            },
        );
        record_wasi_trace(
            &mut caller,
            format!("path_open created_written_file fd={next_fd} normalized={created_path:?}"),
        );
        let status = write_u32(&mut caller, opened_fd_ptr, next_fd as u32);
        record_wasi_trace(
            &mut caller,
            format!("path_open result fd={next_fd} status={status}"),
        );
        return status;
    }
    if is_rouwdi_writable_output_path(&normalized) {
        let output_path = normalized.clone();
        caller
            .data_mut()
            .written_files
            .insert(output_path.clone(), Vec::new());
        caller.data_mut().next_fd = next_fd.saturating_add(1);
        caller.data_mut().fds.insert(
            next_fd,
            VirtualFd::WrittenFile {
                path: output_path.clone(),
                position: 0,
            },
        );
        record_wasi_trace(
            &mut caller,
            format!(
                "path_open created_implicit_codegen_output fd={next_fd} normalized={output_path:?}"
            ),
        );
        let status = write_u32(&mut caller, opened_fd_ptr, next_fd as u32);
        record_wasi_trace(
            &mut caller,
            format!("path_open result fd={next_fd} status={status}"),
        );
        return status;
    }
    let _ = write_u32(&mut caller, opened_fd_ptr, 0);
    record_wasi_trace(
        &mut caller,
        format!("path_open noent normalized={normalized:?} oflags={oflags}"),
    );
    WASI_ERRNO_NOENT
}

fn is_rouwdi_writable_output_path(path: &str) -> bool {
    path == CODEGEN_OBJECT_ARTIFACT_PATH
        || path.ends_with(".rcgu.o")
        || path.ends_with(".rcgu.bc")
        || path.ends_with(".rcgu.ll")
        || path.ends_with(".rcgu.s")
}

fn wasi_path_readlink(
    mut caller: Caller<'_, PayloadWasiState>,
    _fd: i32,
    _path_ptr: i32,
    _path_len: i32,
    _buf: i32,
    _buf_len: i32,
    bufused_ptr: i32,
) -> i32 {
    let _ = write_u32(&mut caller, bufused_ptr, 0);
    WASI_ERRNO_NOENT
}

fn wasi_path_remove_directory(
    mut caller: Caller<'_, PayloadWasiState>,
    fd: i32,
    path_ptr: i32,
    path_len: i32,
) -> i32 {
    if fd != WASI_PREOPEN_FD {
        return WASI_ERRNO_BADF;
    }
    let Some(path) = read_path(&caller, path_ptr, path_len) else {
        return WASI_ERRNO_INVAL;
    };
    let normalized = normalize_virtual_path(&path);
    if caller.data_mut().created_dirs.remove(&normalized) {
        WASI_ERRNO_SUCCESS
    } else if virtual_dir_exists(&normalized) {
        WASI_ERRNO_SUCCESS
    } else {
        WASI_ERRNO_NOENT
    }
}

fn wasi_path_rename(
    _caller: Caller<'_, PayloadWasiState>,
    _fd: i32,
    _path_ptr: i32,
    _path_len: i32,
    _new_fd: i32,
    _new_path_ptr: i32,
    _new_path_len: i32,
) -> i32 {
    WASI_ERRNO_NOSYS
}

fn wasi_path_unlink_file(
    mut caller: Caller<'_, PayloadWasiState>,
    fd: i32,
    path_ptr: i32,
    path_len: i32,
) -> i32 {
    if fd != WASI_PREOPEN_FD {
        return WASI_ERRNO_BADF;
    }
    let Some(path) = read_path(&caller, path_ptr, path_len) else {
        return WASI_ERRNO_INVAL;
    };
    let normalized = normalize_virtual_path(&path);
    if caller
        .data_mut()
        .written_files
        .remove(&normalized)
        .is_some()
    {
        WASI_ERRNO_SUCCESS
    } else {
        WASI_ERRNO_NOENT
    }
}

fn wasi_proc_exit(mut caller: Caller<'_, PayloadWasiState>, code: i32) {
    caller.data_mut().proc_exit_code = Some(code);
}

fn wasi_sched_yield() -> i32 {
    WASI_ERRNO_SUCCESS
}

fn call_u32_export(
    instance: &wasmi::Instance,
    store: &mut Store<PayloadWasiState>,
    name: &str,
) -> Result<u32, String> {
    let func = instance
        .get_typed_func::<(), i32>(&*store, name)
        .map_err(|error| format!("missing/corrupt {name} export: {error}"))?;
    let raw = func
        .call(store, ())
        .map_err(|error| format!("{name} export trapped: {error}"))?;
    if raw < 0 {
        Err(format!("{name} returned negative pointer/value {raw}"))
    } else {
        Ok(raw as u32)
    }
}

struct PayloadExecuteOutcome {
    status: i32,
    trap: Option<String>,
}

fn call_start_with_resumable_fuel(
    start: &wasmi::TypedFunc<(), ()>,
    store: &mut Store<PayloadWasiState>,
) -> Result<PayloadExecuteOutcome, String> {
    store
        .set_fuel(PAYLOAD_FUEL_CHUNK)
        .map_err(|error| error.to_string())?;
    let mut call = match start.call_resumable(&mut *store, ()) {
        Ok(call) => call,
        Err(error) => {
            return Ok(PayloadExecuteOutcome {
                status: -1903,
                trap: Some(format!("{error:?}")),
            });
        }
    };
    let mut resumes = 0_usize;
    loop {
        match call {
            TypedResumableCall::Finished(()) => {
                return Ok(PayloadExecuteOutcome {
                    status: 0,
                    trap: None,
                });
            }
            TypedResumableCall::HostTrap(trap) => {
                return Ok(PayloadExecuteOutcome {
                    status: -1902,
                    trap: Some(format!("host import trap: {trap:?}")),
                });
            }
            TypedResumableCall::OutOfFuel(out_of_fuel) => {
                resumes = resumes.saturating_add(1);
                if resumes > PAYLOAD_MAX_FUEL_RESUMES {
                    return Err(format!(
                        "_start export exceeded rouwdi-owned fuel budget after {resumes} resumptions"
                    ));
                }
                let next_fuel = PAYLOAD_FUEL_CHUNK.max(out_of_fuel.required_fuel());
                store
                    .set_fuel(next_fuel)
                    .map_err(|error| error.to_string())?;
                if resumes == 1 || resumes % 1000 == 0 {
                    trace_payload_loader(&format!(
                        "_start resumed after out-of-fuel ({resumes} chunks)"
                    ));
                }
                call = match out_of_fuel.resume(&mut *store) {
                    Ok(call) => call,
                    Err(error) => {
                        return Ok(PayloadExecuteOutcome {
                            status: -1901,
                            trap: Some(format!("{error:?}")),
                        });
                    }
                };
            }
        }
    }
}

fn call_execute_with_resumable_fuel(
    execute: &wasmi::TypedFunc<(i32, i32, i32, i32, i32, i32), i32>,
    store: &mut Store<PayloadWasiState>,
    params: (i32, i32, i32, i32, i32, i32),
) -> Result<PayloadExecuteOutcome, String> {
    store
        .set_fuel(PAYLOAD_FUEL_CHUNK)
        .map_err(|error| error.to_string())?;
    let mut call = execute
        .call_resumable(&mut *store, params)
        .map_err(|error| format!("execute export trapped before resumable call: {error}"))?;
    let mut resumes = 0_usize;
    loop {
        match call {
            TypedResumableCall::Finished(status) => {
                return Ok(PayloadExecuteOutcome { status, trap: None });
            }
            TypedResumableCall::HostTrap(trap) => {
                return Ok(PayloadExecuteOutcome {
                    status: -1902,
                    trap: Some(format!("host import trap: {trap:?}")),
                });
            }
            TypedResumableCall::OutOfFuel(out_of_fuel) => {
                resumes = resumes.saturating_add(1);
                if resumes > PAYLOAD_MAX_FUEL_RESUMES {
                    return Err(format!(
                        "execute export exceeded rouwdi-owned fuel budget after {resumes} resumptions"
                    ));
                }
                let next_fuel = PAYLOAD_FUEL_CHUNK.max(out_of_fuel.required_fuel());
                store
                    .set_fuel(next_fuel)
                    .map_err(|error| error.to_string())?;
                if resumes == 1 || resumes % 1000 == 0 {
                    trace_payload_loader(&format!(
                        "execute resumed after out-of-fuel ({resumes} chunks)"
                    ));
                }
                call = match out_of_fuel.resume(&mut *store) {
                    Ok(call) => call,
                    Err(error) => {
                        return Ok(PayloadExecuteOutcome {
                            status: -1901,
                            trap: Some(format!("{error:?}")),
                        });
                    }
                };
            }
        }
    }
}

fn read_guest_string(
    memory: &Memory,
    store: &Store<PayloadWasiState>,
    ptr: u32,
    len: u32,
    label: &str,
) -> Result<String, String> {
    let bytes = read_guest_bytes(memory, store, ptr, len, label)?;
    String::from_utf8(bytes).map_err(|error| format!("{label} bytes were not UTF-8: {error}"))
}

fn read_guest_u32(
    memory: &Memory,
    store: &Store<PayloadWasiState>,
    ptr: u32,
    label: &str,
) -> Result<u32, String> {
    let bytes = read_guest_bytes(memory, store, ptr, 4, label)?;
    let raw: [u8; 4] = bytes
        .try_into()
        .map_err(|_| format!("{label} did not contain 4 bytes"))?;
    Ok(u32::from_le_bytes(raw))
}

fn read_guest_bytes(
    memory: &Memory,
    store: &Store<PayloadWasiState>,
    ptr: u32,
    len: u32,
    label: &str,
) -> Result<Vec<u8>, String> {
    let mut bytes = vec![0_u8; len as usize];
    memory
        .read(store, ptr as usize, &mut bytes)
        .map_err(|error| format!("failed to read {label} at {ptr:#x}+{len}: {error}"))?;
    Ok(bytes)
}

fn caller_memory(caller: &Caller<'_, PayloadWasiState>) -> Option<Memory> {
    caller.get_export("memory").and_then(Extern::into_memory)
}

fn write_u32(caller: &mut Caller<'_, PayloadWasiState>, ptr: i32, value: u32) -> i32 {
    write_bytes(caller, ptr, &value.to_le_bytes())
}

fn write_u64(caller: &mut Caller<'_, PayloadWasiState>, ptr: i32, value: u64) -> i32 {
    write_bytes(caller, ptr, &value.to_le_bytes())
}

fn write_bytes(caller: &mut Caller<'_, PayloadWasiState>, ptr: i32, bytes: &[u8]) -> i32 {
    if ptr < 0 {
        return WASI_ERRNO_INVAL;
    }
    let Some(memory) = caller_memory(caller) else {
        return WASI_ERRNO_INVAL;
    };
    memory
        .write(caller, ptr as usize, bytes)
        .map(|_| WASI_ERRNO_SUCCESS)
        .unwrap_or(WASI_ERRNO_INVAL)
}

fn write_filestat(
    caller: &mut Caller<'_, PayloadWasiState>,
    stat_ptr: i32,
    filetype: u8,
    size: u64,
) -> i32 {
    let mut stat = [0_u8; 64];
    stat[16] = filetype;
    stat[24..32].copy_from_slice(&1_u64.to_le_bytes());
    stat[32..40].copy_from_slice(&size.to_le_bytes());
    write_bytes(caller, stat_ptr, &stat)
}

fn read_memory_u32(
    memory: &Memory,
    caller: &Caller<'_, PayloadWasiState>,
    ptr: usize,
) -> Result<u32, ()> {
    let mut raw = [0_u8; 4];
    memory.read(caller, ptr, &mut raw).map_err(|_| ())?;
    Ok(u32::from_le_bytes(raw))
}

fn read_path(caller: &Caller<'_, PayloadWasiState>, ptr: i32, len: i32) -> Option<String> {
    if ptr < 0 || len < 0 {
        return None;
    }
    let memory = caller_memory(caller)?;
    let mut bytes = vec![0_u8; len as usize];
    memory.read(caller, ptr as usize, &mut bytes).ok()?;
    String::from_utf8(bytes).ok()
}

fn virtual_dir_exists(path: &str) -> bool {
    let normalized = normalize_virtual_path(path);
    normalized.is_empty()
        || normalized == "third_party"
        || normalized == "third_party/rust"
        || normalized == "third_party/rust/build"
        || normalized == "third_party/rust/build/x86_64-pc-windows-msvc"
        || normalized == VIRTUAL_SYSROOT
        || normalized == "third_party/rust/build/x86_64-pc-windows-msvc/stage1/lib"
        || normalized == "third_party/rust/build/x86_64-pc-windows-msvc/stage1/lib/rustlib"
        || normalized
            == "third_party/rust/build/x86_64-pc-windows-msvc/stage1/lib/rustlib/wasm32-wasip1"
        || normalized == VIRTUAL_RUSTLIB
        || normalized
            == "third_party/rust/build/x86_64-pc-windows-msvc/stage1/lib/rustlib/wasm32-wasip1/lib/self-contained"
        || VIRTUAL_FILES
            .iter()
            .any(|file| parent_virtual_path(file.path) == normalized)
}

fn virtual_file(path: &str) -> Option<&'static VirtualFile> {
    let normalized = normalize_virtual_path(path);
    VIRTUAL_FILES
        .iter()
        .find(|file| file.path == normalized.as_str())
}

fn virtual_dir_entries(path: &str) -> Vec<VirtualDirEntry> {
    let normalized = normalize_virtual_path(path);
    let mut entries = Vec::new();
    for dir in virtual_child_dirs(&normalized) {
        entries.push(VirtualDirEntry {
            name: (*dir).to_owned(),
            filetype: WASI_FILETYPE_DIRECTORY,
        });
    }
    for file in VIRTUAL_FILES
        .iter()
        .filter(|file| parent_virtual_path(file.path) == normalized)
    {
        if let Some(name) = file.path.rsplit('/').next() {
            entries.push(VirtualDirEntry {
                name: name.to_owned(),
                filetype: WASI_FILETYPE_REGULAR_FILE,
            });
        }
    }
    entries.sort_by(|left, right| left.name.cmp(&right.name));
    entries.dedup_by(|left, right| left.name == right.name);
    entries
}

fn virtual_child_dirs(path: &str) -> &'static [&'static str] {
    match path {
        "" => &["third_party"],
        "third_party" => &["rust"],
        "third_party/rust" => &["build"],
        "third_party/rust/build" => &["x86_64-pc-windows-msvc"],
        "third_party/rust/build/x86_64-pc-windows-msvc" => &["stage1"],
        VIRTUAL_SYSROOT => &["lib"],
        "third_party/rust/build/x86_64-pc-windows-msvc/stage1/lib" => &["rustlib"],
        "third_party/rust/build/x86_64-pc-windows-msvc/stage1/lib/rustlib" => &["wasm32-wasip1"],
        "third_party/rust/build/x86_64-pc-windows-msvc/stage1/lib/rustlib/wasm32-wasip1" => {
            &["lib"]
        }
        VIRTUAL_RUSTLIB => &["self-contained"],
        _ => &[],
    }
}

fn normalize_virtual_path(path: &str) -> String {
    let normalized = path.replace('\\', "/");
    let trimmed = normalized
        .trim_start_matches('/')
        .trim_end_matches('/')
        .trim_start_matches("./");
    trimmed
        .strip_prefix("workspace/")
        .unwrap_or(trimmed)
        .to_owned()
}

fn parent_virtual_path(path: &str) -> String {
    path.rsplit_once('/')
        .map(|(parent, _)| parent.to_owned())
        .unwrap_or_default()
}

fn canonical_execution_state(raw_state: &str, blocker_kind: Option<&str>) -> String {
    if raw_state == "mir_body_identity_emitted" {
        return "embedded_payload_mir_body_identity_emitted".to_owned();
    }
    if raw_state == "mir_body_hash_emitted" {
        return "embedded_payload_mir_body_hash_emitted".to_owned();
    }
    if raw_state == "mono_items_collected" {
        return "embedded_payload_mono_items_collected".to_owned();
    }
    if blocker_kind.is_some_and(|kind| kind.starts_with("missing_core_lang_item"))
        || raw_state.contains("lang_item")
    {
        return "embedded_payload_executed_blocked_at_mir_provider_requires_lang_items".to_owned();
    }
    format!("embedded_payload_executed_{raw_state}")
}

fn json_str<'a>(value: &'a Option<serde_json::Value>, key: &str) -> Option<&'a str> {
    value
        .as_ref()
        .and_then(|value| value.get(key))
        .and_then(serde_json::Value::as_str)
}

fn json_bool(value: Option<&serde_json::Value>, key: &str) -> bool {
    value
        .and_then(|value| value.get(key))
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(false)
}

fn json_string_field(value: Option<&serde_json::Value>, key: &str) -> Option<String> {
    value
        .and_then(|value| value.get(key))
        .and_then(serde_json::Value::as_str)
        .map(str::to_owned)
}

fn json_string_array(value: Option<&serde_json::Value>, key: &str) -> Vec<String> {
    value
        .and_then(|value| value.get(key))
        .and_then(serde_json::Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(serde_json::Value::as_str)
                .map(str::to_owned)
                .collect()
        })
        .unwrap_or_default()
}

fn object_contains_any_reported_codegened_symbol(
    inspection: Option<&WasmObjectInspection>,
    reported_symbols: &[String],
) -> bool {
    let Some(inspection) = inspection else {
        return false;
    };
    reported_symbols.iter().any(|reported| {
        let reported = reported.as_str();
        !reported.is_empty()
            && (inspection.object_symbols.iter().any(|symbol| {
                symbol.kind == "function"
                    && !symbol.undefined
                    && symbol.name.as_deref() == Some(reported)
            }) || inspection
                .object_exports
                .iter()
                .any(|export| export == reported))
    })
}

fn codegen_payload_argv() -> Vec<String> {
    vec![
        "rouwdi-rustc-codegen-llvm-probe.wasm".to_owned(),
        "--json".to_owned(),
        "--compile-unit-id".to_owned(),
        "app:rust:app:wasm32-wasip1".to_owned(),
        "--crate-identity".to_owned(),
        "rouwdi_payload".to_owned(),
        "--target-triple".to_owned(),
        "wasm32-wasip1".to_owned(),
        "--target-spec".to_owned(),
        "rustc_target::spec::wasm32_wasip1".to_owned(),
        "--mir-body-hash".to_owned(),
        "a5e137ef6793c0b8".to_owned(),
        "--mono-item-count".to_owned(),
        "1".to_owned(),
        "--mono-item-graph-hash".to_owned(),
        "bec5817d61819666".to_owned(),
        "--mono-item".to_owned(),
        "fn:rouwdi_payload::main".to_owned(),
    ]
}

fn to_string(error: impl ToString) -> String {
    error.to_string()
}

fn record_wasi_trace(caller: &mut Caller<'_, PayloadWasiState>, message: String) {
    let trace = &mut caller.data_mut().wasi_trace;
    if trace.len() < 1024 {
        trace.push(message);
    }
}

fn json_string(value: &str) -> String {
    serde_json::to_string(value).unwrap_or_else(|_| "\"<json-string-encoding-failed>\"".to_owned())
}

fn trace_payload_loader(_message: &str) {
    #[cfg(not(target_arch = "wasm32"))]
    if std::env::var_os("ROUWDI_PAYLOAD_TRACE").is_some() {
        eprintln!("rouwdi embedded payload loader: {_message}");
    }
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

    #[test]
    #[cfg(feature = "embedded-mir-payload")]
    fn embedded_loader_instantiates_and_executes_mir_payload_from_registry_bytes() {
        let report = load_embedded_compiler_payload("rouwdi-mir-handoff-payload")
            .expect("embedded MIR payload must instantiate and execute from registry bytes");

        assert_eq!(report.execution_source, "embedded_registry");
        assert!(!report.external);
        assert!(!report.opened_external_file);
        assert!(report.hash_verified);
        assert!(report.size_verified);
        assert!(report.wasm_magic_verified);
        assert!(report.module_instantiated);
        assert!(report.abi_v1_exports_verified);
        assert!(report.version_called);
        assert_eq!(report.version, 1);
        assert!(report.stage_called);
        assert_eq!(report.stage_code, 1);
        assert!(report.descriptor_bytes_read);
        assert!(report.descriptor_json.contains("mir-handoff"));
        assert!(report.valid_input_bytes_read);
        assert!(report.valid_input_json.contains("compile_unit_id"));
        assert!(report.execute_called);
        assert!(report.output_bytes_read || report.error_bytes_read);
        assert!(report
            .exports
            .iter()
            .any(|export| export == "rouwdi_mir_handoff_payload_v1_execute"));
        assert!(report
            .imports
            .iter()
            .all(|import| !import.contains("wasmtime")));
        assert!(
            report
                .execution_state
                .starts_with("embedded_payload_executed")
                || report.execution_state == "embedded_payload_mir_body_hash_emitted"
                || report.execution_state == "embedded_payload_mono_items_collected"
        );
        assert_eq!(report.input_contract_sha256.len(), 64);
        assert!(
            report
                .output_contract_sha256
                .as_deref()
                .is_some_and(|hash| hash.len() == 64)
                || report
                    .error_contract_sha256
                    .as_deref()
                    .is_some_and(|hash| hash.len() == 64)
        );
    }
}
