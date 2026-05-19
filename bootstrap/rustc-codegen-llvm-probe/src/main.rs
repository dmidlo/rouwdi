#![recursion_limit = "256"]

use rouwdi_object::{inspect_wasm_object, WasmObjectInspection};
use sha2::{Digest, Sha256};
use std::alloc::{alloc, dealloc, Layout};
use std::collections::{BTreeMap, BTreeSet};
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int, c_uint, c_void};
use std::path::PathBuf;
use std::process::ExitCode;
use std::ptr;
use std::slice;
use std::sync::atomic::AtomicBool;

#[derive(Debug, Clone)]
struct ProbeInput {
    json: bool,
    compile_unit_id: String,
    package: String,
    target: String,
    target_kind: String,
    profile: String,
    source_path: String,
    source_bytes: Vec<u8>,
    source_sha256: String,
    crate_identity: String,
    crate_artifact_kind: String,
    target_triple: String,
    target_spec: String,
    mir_body_hash: String,
    mono_item_count: u64,
    mono_item_graph_hash: String,
    mono_items: Vec<String>,
    extern_artifacts: Vec<ProbeExternArtifact>,
    link_dependency_artifacts: Vec<ProbeLinkDependencyArtifact>,
}

impl Default for ProbeInput {
    fn default() -> Self {
        Self {
            json: false,
            compile_unit_id: "app:rust:app:wasm32-wasip1".to_owned(),
            package: "app".to_owned(),
            target: "wasi".to_owned(),
            target_kind: "Bin".to_owned(),
            profile: "release".to_owned(),
            source_path: String::new(),
            source_bytes: Vec::new(),
            source_sha256: String::new(),
            crate_identity: "rouwdi_payload".to_owned(),
            crate_artifact_kind: "binary".to_owned(),
            target_triple: "wasm32-wasip1".to_owned(),
            target_spec: "rustc_target::spec::wasm32_wasip1".to_owned(),
            mir_body_hash: "a5e137ef6793c0b8".to_owned(),
            mono_item_count: 1,
            mono_item_graph_hash: "bec5817d61819666".to_owned(),
            mono_items: Vec::new(),
            extern_artifacts: Vec::new(),
            link_dependency_artifacts: Vec::new(),
        }
    }
}

#[derive(Debug, Clone)]
struct ProbeExternArtifact {
    extern_crate_name: String,
    path: String,
    sha256: String,
}

#[derive(Debug, Clone)]
struct ProbeLinkDependencyArtifact {
    path: String,
    sha256: String,
}

#[derive(Debug, Clone)]
struct ModuleSetup {
    attempted: bool,
    llvm_context_created: bool,
    llvm_module_created: bool,
    module_identity: Option<String>,
    module_identity_hash: Option<String>,
    module_target_triple: Option<String>,
    llvm_ir: Option<String>,
    llvm_ir_sha256: Option<String>,
    llvm_ir_size_bytes: Option<usize>,
    blocker_kind: Option<String>,
    blocker_reason: Option<String>,
}

#[derive(Debug, Clone)]
struct TargetMachineSetup {
    attempted: bool,
    target_triple: String,
    cpu: String,
    features: String,
    relocation_model: String,
    code_model: String,
    optimization_level: String,
    target_machine_created: bool,
    blocker_kind: Option<String>,
    blocker_reason: Option<String>,
}

#[derive(Debug, Clone)]
struct ObjectEmissionSetup {
    attempted: bool,
    api: String,
    codegen_lowering_invoked: bool,
    codegen_lowering_entrypoint: String,
    object_bytes_emitted: bool,
    wasm_object_bytes_emitted: bool,
    inspection: Option<WasmObjectInspection>,
    object_contains_codegened_function: bool,
    codegened_mono_item_count: u64,
    codegened_symbols: Vec<String>,
    object_derived_from: String,
    object_codegen_source: String,
    artifact_kind: Option<String>,
    artifact_sha256: Option<String>,
    artifact_size_bytes: Option<usize>,
    artifact_location: Option<String>,
    target_triple: String,
    retrieval_method: Option<String>,
    crate_artifact: Option<CrateArtifactOutput>,
    blocker_kind: Option<String>,
    blocker_reason: Option<String>,
}

#[derive(Debug, Clone)]
struct LinkerPayloadIdentity {
    sha256: String,
    size_bytes: usize,
}

#[derive(Debug, Clone)]
struct LinkExecution {
    linker_invoked: bool,
    command_args: Vec<String>,
    input_artifact_hashes: Vec<String>,
    output_artifact_path: Option<String>,
    stdout: String,
    stderr: String,
    exit_code: i32,
    final_module_bytes: Option<Vec<u8>>,
    final_module_sha256: Option<String>,
    final_module_size_bytes: Option<usize>,
    interface_proof: Option<InterfaceProof>,
    current_status: String,
    blocker_kind: String,
    blocker_reason: String,
}

#[derive(Debug, Clone)]
struct CrateArtifactOutput {
    artifact_path: String,
    artifact_sha256: String,
    artifact_size_bytes: usize,
    metadata_sha256: Option<String>,
    metadata_member: Option<String>,
    bytes: Vec<u8>,
}

#[derive(Debug, Clone)]
struct InterfaceProof {
    wasm_magic_valid: bool,
    wasm_version_valid: bool,
    exports: Vec<String>,
    imports: Vec<String>,
    start_present: bool,
    wasi_imports: Vec<String>,
    required_exports_checked: Vec<String>,
    missing_required_exports: Vec<String>,
    module_sha256: String,
    module_size_bytes: usize,
    passed: bool,
}

type LLVMTargetRef = *mut c_void;
type LLVMTargetMachineRef = *mut c_void;
type LLVMMemoryBufferRef = *mut c_void;

const LLVM_OBJECT_FILE: c_uint = 1;
const OBJECT_ARTIFACT_PATH: &str = "rouwdi-codegen-wasm32-wasip1.o";
const ALLOCATOR_OBJECT_ARTIFACT_PATH: &str = "rouwdi-codegen-wasm32-wasip1-allocator.o";
const FINAL_WASI_MODULE_PATH: &str = "rouwdi-codegen-wasm32-wasip1-linked.wasm";
const LLD_WRAPPER_ARCHIVE: &[u8] = include_bytes!(
    "../../../.rouwdi/codegen-llvm-probe/target-llvm-wrapper/lib/librouwdi-lld-wasm-wrapper.a"
);
const LLD_WASM_ARCHIVE: &[u8] =
    include_bytes!("../../../third_party/rust/build/wasm32-wasip1/llvm/build/lib/liblldWasm.a");
const LLD_COMMON_ARCHIVE: &[u8] =
    include_bytes!("../../../third_party/rust/build/wasm32-wasip1/llvm/build/lib/liblldCommon.a");
const UPSTREAM_CODEGEN_ENTRYPOINT: &str =
    "rustc_codegen_llvm::LlvmCodegenBackend::codegen_crate -> rustc_codegen_ssa::base::codegen_crate";
const CODEGEN_LOWERING_STATUS: &str =
    "codegen_lowering_blocked_at_rustc_codegen_ssa_base_codegen_crate_requires_live_tyctxt_and_codegen_unit";
const CODEGEN_LOWERING_BLOCKER_KIND: &str =
    "rustc_codegen_ssa_base_codegen_crate_requires_live_tyctxt_and_codegen_unit";
const CODEGEN_LOWERING_BLOCKER_COMPONENT: &str = "rustc_codegen_ssa::base::codegen_crate";
const CODEGEN_LOWERING_PATH: &[&str] = &[
    "rustc_codegen_llvm::LlvmCodegenBackend::codegen_crate",
    "rustc_codegen_ssa::base::codegen_crate",
    "rustc_middle::ty::TyCtxt::collect_and_partition_mono_items",
    "rustc_codegen_ssa::traits::backend::ExtraBackendMethods::compile_codegen_unit",
    "rustc_codegen_llvm::base::compile_codegen_unit",
    "rustc_codegen_llvm::base::module_codegen",
    "rustc_codegen_llvm::context::CodegenCx",
    "rustc_codegen_ssa::mir::codegen_mir",
];
const CODEGEN_LOWERING_MISSING_INPUTS: &[&str] = &[
    "live rustc_middle::ty::TyCtxt<'tcx>",
    "live rustc_middle::mir::mono::CodegenUnit<'tcx>",
    "live rustc_codegen_llvm::context::CodegenCx<'ll, 'tcx>",
    "rustc_codegen_ssa::ModuleCodegen<rustc_codegen_llvm::ModuleLlvm>",
];
static USING_INTERNAL_FEATURES: AtomicBool = AtomicBool::new(false);

#[link(name = "llvm-wrapper", kind = "static")]
unsafe extern "C" {}

#[link(name = "rouwdi-lld-wasm-wrapper", kind = "static")]
#[link(name = "lldWasm", kind = "static")]
#[link(name = "lldCommon", kind = "static")]
#[link(name = "LLVMOption", kind = "static")]
unsafe extern "C" {}

unsafe extern "C" {
    fn rouwdi_lld_wasm_link(
        argc: c_int,
        argv: *const *const c_char,
        stdout_ptr: *mut *mut c_char,
        stdout_len: *mut usize,
        stderr_ptr: *mut *mut c_char,
        stderr_len: *mut usize,
    ) -> c_int;
    fn rouwdi_lld_free(ptr: *mut c_char);
    fn LLVMContextCreate() -> *mut c_void;
    fn LLVMContextDispose(context: *mut c_void);
    fn LLVMModuleCreateWithNameInContext(
        module_id: *const c_char,
        context: *mut c_void,
    ) -> *mut c_void;
    fn LLVMDisposeModule(module: *mut c_void);
    fn LLVMSetTarget(module: *mut c_void, triple: *const c_char);
    fn LLVMPrintModuleToString(module: *mut c_void) -> *mut c_char;
    fn LLVMInitializeWebAssemblyTargetInfo();
    fn LLVMInitializeWebAssemblyTarget();
    fn LLVMInitializeWebAssemblyTargetMC();
    fn LLVMInitializeWebAssemblyAsmPrinter();
    fn LLVMGetTargetFromTriple(
        triple: *const c_char,
        target: *mut LLVMTargetRef,
        error_message: *mut *mut c_char,
    ) -> c_int;
    fn LLVMCreateTargetMachine(
        target: LLVMTargetRef,
        triple: *const c_char,
        cpu: *const c_char,
        features: *const c_char,
        level: c_uint,
        reloc: c_uint,
        code_model: c_uint,
    ) -> LLVMTargetMachineRef;
    fn LLVMDisposeTargetMachine(target_machine: LLVMTargetMachineRef);
    fn LLVMTargetMachineEmitToMemoryBuffer(
        target_machine: LLVMTargetMachineRef,
        module: *mut c_void,
        codegen: c_uint,
        error_message: *mut *mut c_char,
        out_mem_buf: *mut LLVMMemoryBufferRef,
    ) -> c_int;
    fn LLVMGetBufferStart(mem_buf: LLVMMemoryBufferRef) -> *const c_char;
    fn LLVMGetBufferSize(mem_buf: LLVMMemoryBufferRef) -> usize;
    fn LLVMDisposeMemoryBuffer(mem_buf: LLVMMemoryBufferRef);
    fn LLVMDisposeMessage(message: *mut c_char);
}

#[no_mangle]
pub extern "C" fn rouwdi_lld_read_file(
    path: *const c_char,
    data_out: *mut *mut c_char,
    len_out: *mut usize,
) -> bool {
    if path.is_null() || data_out.is_null() || len_out.is_null() {
        return false;
    }
    let path = unsafe { CStr::from_ptr(path) }
        .to_string_lossy()
        .into_owned();
    let bytes = match lld_path_candidates(&path)
        .into_iter()
        .find_map(|candidate| std::fs::read(&candidate).ok())
    {
        Some(bytes) => bytes,
        None => return false,
    };
    unsafe {
        *len_out = bytes.len();
        if bytes.is_empty() {
            *data_out = ptr::null_mut();
            return true;
        }
    }
    let Ok(layout) = Layout::array::<u8>(bytes.len()) else {
        return false;
    };
    let ptr = unsafe { alloc(layout) };
    if ptr.is_null() {
        return false;
    }
    unsafe {
        ptr::copy_nonoverlapping(bytes.as_ptr(), ptr, bytes.len());
        *data_out = ptr.cast::<c_char>();
    }
    true
}

#[no_mangle]
pub extern "C" fn rouwdi_lld_free_file(data: *mut c_char, len: usize) {
    if data.is_null() || len == 0 {
        return;
    }
    if let Ok(layout) = Layout::array::<u8>(len) {
        unsafe {
            dealloc(data.cast::<u8>(), layout);
        }
    }
}

#[no_mangle]
pub extern "C" fn rouwdi_lld_write_output(
    path: *const c_char,
    data: *const c_char,
    len: usize,
) -> bool {
    if path.is_null() || (data.is_null() && len != 0) {
        return false;
    }
    let path = unsafe { CStr::from_ptr(path) }
        .to_string_lossy()
        .into_owned();
    let bytes = if len == 0 {
        &[][..]
    } else {
        unsafe { slice::from_raw_parts(data.cast::<u8>(), len) }
    };
    lld_path_candidates(&path)
        .into_iter()
        .any(|candidate| std::fs::write(candidate, bytes).is_ok())
}

fn lld_path_candidates(path: &str) -> Vec<String> {
    let normalized = path.replace('\\', "/");
    let mut candidates = vec![normalized.clone()];
    for prefix in ["/workspace/", "workspace/", "/"] {
        if let Some(stripped) = normalized.strip_prefix(prefix) {
            candidates.push(stripped.to_owned());
        }
    }
    candidates.sort();
    candidates.dedup();
    candidates
}

fn main() -> ExitCode {
    eprintln!("rouwdi-codegen-progress: main-entered");
    let input = match parse_input() {
        Ok(input) => input,
        Err(error) => {
            eprintln!("{error}");
            return ExitCode::from(2);
        }
    };

    let backend = rustc_codegen_llvm::LlvmCodegenBackend::new();
    eprintln!("rouwdi-codegen-progress: backend-constructed");
    let backend_name = backend.name();
    let module_setup = attempt_llvm_module_setup(&input);
    eprintln!("rouwdi-codegen-progress: llvm-module-setup-complete");
    let target_machine_setup = if module_setup.llvm_module_created {
        attempt_target_machine_setup(&input)
    } else {
        TargetMachineSetup {
            attempted: false,
            target_triple: input.target_triple.clone(),
            cpu: "generic".to_owned(),
            features: String::new(),
            relocation_model: "pic".to_owned(),
            code_model: "default".to_owned(),
            optimization_level: "none".to_owned(),
            target_machine_created: false,
            blocker_kind: module_setup.blocker_kind.clone(),
            blocker_reason: module_setup.blocker_reason.clone(),
        }
    };
    eprintln!("rouwdi-codegen-progress: target-machine-setup-complete");
    let object_emission = attempt_object_emission(&input);
    eprintln!("rouwdi-codegen-progress: object-emission-complete");

    let llvm_ir_emitted = module_setup.llvm_ir.is_some();
    let rust_mono_item_wasm_object_emitted = object_emission.wasm_object_bytes_emitted
        && object_emission.object_contains_codegened_function
        && object_emission.codegened_mono_item_count > 0;
    let linker_payload_identity = linker_payload_identity();
    let link_execution =
        if rust_mono_item_wasm_object_emitted && input.crate_artifact_kind != "library" {
            Some(attempt_wasm_link(
                &input,
                &object_emission,
                &linker_payload_identity,
            ))
        } else {
            None
        };
    let codegen_lowering_status = if rust_mono_item_wasm_object_emitted {
        "rust_mono_item_wasm_object_emitted"
    } else if object_emission.wasm_object_bytes_emitted {
        CODEGEN_LOWERING_STATUS
    } else {
        "codegen_lowering_not_reached"
    };
    let codegen_lowering_blocker_kind =
        if object_emission.wasm_object_bytes_emitted && !rust_mono_item_wasm_object_emitted {
            CODEGEN_LOWERING_BLOCKER_KIND
        } else {
            "none"
        };
    let codegen_lowering_blocker_component =
        if object_emission.wasm_object_bytes_emitted && !rust_mono_item_wasm_object_emitted {
            CODEGEN_LOWERING_BLOCKER_COMPONENT
        } else {
            "none"
        };
    let codegen_lowering_blocker_reason = if object_emission.wasm_object_bytes_emitted
        && !rust_mono_item_wasm_object_emitted
    {
        "The embedded payload has the mono item graph proof and rustc_codegen_llvm backend, but it does not yet carry the live TyCtxt and CodegenUnit values required to enter rustc_codegen_ssa::base::codegen_crate and rustc_codegen_llvm::base::compile_codegen_unit; the emitted object is therefore kept classified as empty/probe-only and cannot be linked."
    } else {
        "none"
    };
    let codegen_contact_state = if rust_mono_item_wasm_object_emitted {
        "rust_mono_item_wasm_object_emitted"
    } else if object_emission.wasm_object_bytes_emitted {
        codegen_lowering_status
    } else if object_emission.attempted
        && object_emission
            .blocker_kind
            .as_deref()
            .is_some_and(|kind| kind != "none")
    {
        object_emission
            .blocker_kind
            .as_deref()
            .unwrap_or("object_emission_blocked")
    } else if llvm_ir_emitted {
        "llvm_ir_emitted"
    } else if target_machine_setup.target_machine_created {
        "target_machine_created"
    } else if target_machine_setup.attempted {
        "target_machine_setup_invoked"
    } else if module_setup.llvm_module_created {
        "llvm_module_setup_invoked"
    } else if cfg!(target_arch = "wasm32") {
        "rustc_codegen_llvm_backend_constructed_in_payload"
    } else {
        "rustc_codegen_llvm_backend_constructed"
    };
    let blocker_kind =
        if object_emission.wasm_object_bytes_emitted && !rust_mono_item_wasm_object_emitted {
            "codegen_lowering_to_object_not_implemented"
        } else {
            target_machine_setup
                .blocker_kind
                .as_deref()
                .or(module_setup.blocker_kind.as_deref())
                .or(object_emission.blocker_kind.as_deref())
                .unwrap_or("none")
        };
    let blocker_component =
        if object_emission.wasm_object_bytes_emitted && !rust_mono_item_wasm_object_emitted {
            "rustc_codegen_llvm mono item lowering"
        } else {
            "none"
        };
    let blocker_reason = if object_emission.wasm_object_bytes_emitted
        && !rust_mono_item_wasm_object_emitted
    {
        "LLVM emitted a valid Wasm object from the payload-created module, but rouwdi-owned inspection found no code-bearing function tied to the mono item graph; rustc_codegen_llvm mono-item lowering has not been invoked yet"
    } else {
        target_machine_setup
            .blocker_reason
            .as_deref()
            .or(module_setup.blocker_reason.as_deref())
            .or(object_emission.blocker_reason.as_deref())
            .unwrap_or("none")
    };
    let llvm_ir_artifact = if let Some(llvm_ir) = module_setup.llvm_ir.as_ref() {
        serde_json::json!({
            "artifact_kind": "llvm_ir",
            "byte_length": module_setup.llvm_ir_size_bytes,
            "sha256": module_setup.llvm_ir_sha256,
            "producer_backend": "rustc_codegen_llvm",
            "target_triple": input.target_triple,
            "mir_body_hash": input.mir_body_hash,
            "mono_item_graph_hash": input.mono_item_graph_hash,
            "module_identity": module_setup.module_identity,
            "module_identity_hash": module_setup.module_identity_hash,
            "target_machine_identity": target_machine_identity(&target_machine_setup),
            "payload_hash": null,
            "linker_required": true,
            "embedded_artifact_location": "backend_stdout.codegen_artifact.llvm_ir",
            "llvm_ir": llvm_ir,
        })
    } else {
        serde_json::Value::Null
    };
    let object_artifact = if object_emission.object_bytes_emitted {
        serde_json::json!({
            "artifact_kind": object_emission.artifact_kind,
            "byte_length": object_emission.artifact_size_bytes,
            "sha256": object_emission.artifact_sha256,
            "producer_backend": "rustc_codegen_llvm",
            "target_triple": object_emission.target_triple,
            "source_path": input.source_path,
            "source_sha256": input.source_sha256,
            "codegen_input_source_sha256": input.source_sha256,
            "codegen_input_source_bytes_sha256": input.source_sha256,
            "codegen_input_source_origin": "vfs_compile_unit_source",
            "mir_body_hash": input.mir_body_hash,
            "mono_item_graph_hash": input.mono_item_graph_hash,
            "llvm_module_identity_hash": module_setup.module_identity_hash,
            "target_machine_identity": target_machine_identity(&target_machine_setup),
            "embedded_artifact_location": object_emission.artifact_location,
            "retrieval_method": object_emission.retrieval_method,
            "object_inspection": object_emission.inspection.clone(),
            "object_format": object_emission.inspection.as_ref().map(|inspection| inspection.object_format.as_str()),
            "object_section_count": object_emission.inspection.as_ref().map(|inspection| inspection.object_section_count),
            "object_has_code_section": object_emission.inspection.as_ref().map(|inspection| inspection.object_has_code_section),
            "object_has_linking_metadata": object_emission.inspection.as_ref().map(|inspection| inspection.object_has_linking_metadata),
            "object_symbol_count": object_emission.inspection.as_ref().map(|inspection| inspection.object_symbol_count),
            "object_function_count": object_emission.inspection.as_ref().map(|inspection| inspection.object_function_count),
            "object_is_empty": object_emission.inspection.as_ref().map(|inspection| inspection.object_is_empty),
            "object_contains_codegened_function": object_emission.object_contains_codegened_function,
            "object_derived_from": object_emission.object_derived_from.clone(),
            "object_codegen_source": object_emission.object_codegen_source.clone(),
            "codegen_lowering_invoked": object_emission.codegen_lowering_invoked,
            "codegen_lowering_entrypoint": object_emission.codegen_lowering_entrypoint.clone(),
            "codegened_mono_item_count": object_emission.codegened_mono_item_count,
            "codegened_symbols": object_emission.codegened_symbols.clone(),
        })
    } else {
        serde_json::Value::Null
    };
    let codegen_artifact = if object_emission.object_bytes_emitted {
        object_artifact.clone()
    } else {
        llvm_ir_artifact.clone()
    };
    let final_module_artifact = if let Some(link) = link_execution.as_ref() {
        if let (Some(hash), Some(size)) = (
            link.final_module_sha256.as_ref(),
            link.final_module_size_bytes,
        ) {
            serde_json::json!({
                "artifact_kind": "wasm32_wasip1_module",
                "target_triple": input.target_triple,
                "source_path": input.source_path,
                "source_sha256": input.source_sha256,
                "codegen_input_source_sha256": input.source_sha256,
                "codegen_input_source_bytes_sha256": input.source_sha256,
                "codegen_input_source_origin": "vfs_compile_unit_source",
                "size_bytes": size,
                "sha256": hash,
                "producer_linker": "rouwdi-wasm-ld",
                "input_object_hash": object_emission.artifact_sha256,
                "artifact_path": format!("vfs:/workspace/{FINAL_WASI_MODULE_PATH}"),
            })
        } else {
            serde_json::Value::Null
        }
    } else {
        serde_json::Value::Null
    };
    let crate_artifact = object_emission
        .crate_artifact
        .as_ref()
        .map(|artifact| {
            serde_json::json!({
                "artifact_kind": "rlib",
                "target_triple": input.target_triple,
                "source_path": input.source_path,
                "source_sha256": input.source_sha256,
                "size_bytes": artifact.artifact_size_bytes,
                "sha256": artifact.artifact_sha256,
                "producer_backend": "rustc_codegen_llvm",
                "artifact_path": format!("vfs:/workspace/{}", artifact.artifact_path),
                "metadata_sha256": artifact.metadata_sha256,
                "metadata_proof": {
                    "member": artifact.metadata_member,
                    "sha256": artifact.metadata_sha256,
                    "embedded_in": format!("vfs:/workspace/{}", artifact.artifact_path)
                },
                "object_hash": object_emission.artifact_sha256,
            })
        })
        .unwrap_or(serde_json::Value::Null);
    let interface_proof = link_execution
        .as_ref()
        .and_then(|link| link.interface_proof.as_ref())
        .map(interface_proof_json)
        .unwrap_or(serde_json::Value::Null);
    let linker_handoff = if rust_mono_item_wasm_object_emitted
        && input.crate_artifact_kind != "library"
    {
        let link = link_execution
            .as_ref()
            .expect("link execution is attempted once mono-item object exists");
        serde_json::json!({
            "compile_unit_id": input.compile_unit_id,
            "target_triple": input.target_triple,
            "codegen_artifact_kind": "wasm_object",
            "codegen_artifact_hash": object_emission.artifact_sha256,
            "codegen_artifact_size": object_emission.artifact_size_bytes,
            "codegen_backend_identity": "rustc_codegen_llvm::LlvmCodegenBackend",
            "linker_payload": {
                "payload_name": "rouwdi-wasm-ld",
                "kind": "linker_payload",
                "component": "wasm-ld",
                "target": input.target_triple,
                "artifact_path": "embedded_registry:linker-payloads/rouwdi-wasm-ld",
                "sha256": linker_payload_identity.sha256,
                "size_bytes": linker_payload_identity.size_bytes,
                "embedding_method": "embedded_registry_static_linked_lld_archives",
                "execution_method": "embedded_wasi_component_in_process_lld_wasm_link",
                "linker_version": "LLD 22.1.0 source-payload",
                "supported_input_kind": "wasm_object",
                "supported_output_kind": "wasm32-wasip1 module"
            },
            "required_linker_component": "wasm-ld",
            "expected_final_artifact_kind": "wasm32-wasip1 module",
            "linker_input_count": link.input_artifact_hashes.len(),
            "required_runtime_objects": ["crt1-command.o"],
            "required_std_core_alloc_objects_or_archives": ["libcore.rlib", "liballoc.rlib", "libstd.rlib", "libwasip1.rlib", "libc.a", "libcompiler_builtins.rlib"],
            "required_std_objects_or_archives": ["libcore.rlib", "liballoc.rlib", "libstd.rlib", "libwasip1.rlib", "libc.a"],
            "current_status": link.current_status,
            "blocker_kind": link.blocker_kind,
            "linker_invoked": link.linker_invoked,
            "linker_command_args": link.command_args,
            "input_artifact_hashes": link.input_artifact_hashes,
            "output_artifact_path": link.output_artifact_path,
            "stdout": link.stdout,
            "stderr": link.stderr,
            "exit_code": link.exit_code,
            "final_module_artifact": final_module_artifact,
            "interface_proof": interface_proof,
            "next_command": if link.current_status == "interface_proof_passed" {
                "attempt runtime proof on the final linked wasm32-wasip1 module"
            } else {
                "fix the embedded wasm-ld invocation or target-pack inputs and re-run interface proof"
            },
            "proof_path": "dist/manifest.json#/codegen_payloads/0/linker_handoff"
        })
    } else {
        serde_json::Value::Null
    };

    let payload = serde_json::json!({
        "probe_name": "rustc_codegen_llvm_backend_execution",
        "upstream_component": "rustc_codegen_llvm",
        "upstream_path": "third_party/rust/compiler/rustc_codegen_llvm",
        "backend_family": "llvm-grade",
        "entrypoint": "rustc_codegen_llvm::LlvmCodegenBackend::new",
        "backend_constructed": true,
        "backend_name": backend_name,
        "codegen_contact_state": codegen_contact_state,
        "compile_unit_id": input.compile_unit_id,
            "package": input.package,
            "target": input.target,
            "target_kind": input.target_kind,
            "profile": input.profile,
            "source_path": input.source_path,
            "source_sha256": input.source_sha256,
        "codegen_input_source_sha256": input.source_sha256,
        "codegen_input_source_bytes_sha256": input.source_sha256,
            "codegen_input_source_origin": "vfs_compile_unit_source",
            "crate_identity": input.crate_identity,
            "crate_artifact_kind": input.crate_artifact_kind,
            "target_triple": input.target_triple,
        "target_spec": input.target_spec,
        "mir_body_hash": input.mir_body_hash,
        "mono_item_count": input.mono_item_count,
        "mono_item_graph_hash": input.mono_item_graph_hash,
            "mono_items": input.mono_items,
            "extern_artifacts": input.extern_artifacts.iter().map(|artifact| serde_json::json!({
                "extern_crate_name": artifact.extern_crate_name,
                "path": artifact.path,
                "sha256": artifact.sha256,
                "rustc_arg": format!("--extern {}={}", artifact.extern_crate_name, artifact.path),
            })).collect::<Vec<_>>(),
            "link_dependency_artifacts": input.link_dependency_artifacts.iter().map(|artifact| serde_json::json!({
                "path": artifact.path,
                "sha256": artifact.sha256,
            })).collect::<Vec<_>>(),
        "mono_proof_consumed": input.mono_item_count > 0
            && !input.mono_item_graph_hash.trim().is_empty()
            && !input.mir_body_hash.trim().is_empty(),
        "llvm_module_setup": {
            "attempted": module_setup.attempted,
            "llvm_context_created": module_setup.llvm_context_created,
            "llvm_module_created": module_setup.llvm_module_created,
            "module_identity": module_setup.module_identity,
            "module_identity_hash": module_setup.module_identity_hash,
            "module_target_triple": module_setup.module_target_triple,
            "blocker_kind": module_setup.blocker_kind,
            "blocker_reason": module_setup.blocker_reason,
        },
        "target_machine_setup": {
            "attempted": target_machine_setup.attempted,
            "target_triple": target_machine_setup.target_triple,
            "cpu": target_machine_setup.cpu,
            "features": target_machine_setup.features,
            "relocation_model": target_machine_setup.relocation_model,
            "code_model": target_machine_setup.code_model,
            "optimization_level": target_machine_setup.optimization_level,
            "target_machine_created": target_machine_setup.target_machine_created,
            "blocker_kind": target_machine_setup.blocker_kind,
            "blocker_reason": target_machine_setup.blocker_reason,
        },
        "llvm_ir_artifact": llvm_ir_artifact,
        "object_artifact": object_artifact,
        "codegen_artifact": codegen_artifact,
        "crate_artifact": crate_artifact,
        "final_module_artifact": final_module_artifact,
        "interface_proof": interface_proof,
        "runtime_proof_attempted": false,
        "runtime_proof": null,
        "llvm_ir_emitted": llvm_ir_emitted,
        "llvm_ir_sha256": module_setup.llvm_ir_sha256,
        "llvm_ir_size_bytes": module_setup.llvm_ir_size_bytes,
        "object_emission_attempted": object_emission.attempted,
        "object_emission_api": object_emission.api,
        "object_bytes_emitted": object_emission.object_bytes_emitted,
        "wasm_object_bytes_emitted": object_emission.wasm_object_bytes_emitted,
        "object_artifact_kind": object_emission.artifact_kind,
        "object_artifact_sha256": object_emission.artifact_sha256,
        "object_artifact_size_bytes": object_emission.artifact_size_bytes,
        "object_artifact_location": object_emission.artifact_location,
        "object_target_triple": object_emission.target_triple,
        "object_retrieval_method": object_emission.retrieval_method,
        "bitcode_emitted": false,
        "linker_required": input.crate_artifact_kind != "library",
        "linker_handoff_created": rust_mono_item_wasm_object_emitted && input.crate_artifact_kind != "library",
        "linker_handoff": linker_handoff,
        "rust_mono_item_wasm_object_emitted": rust_mono_item_wasm_object_emitted,
        "codegened_mono_item_count": object_emission.codegened_mono_item_count,
        "codegened_symbols": object_emission.codegened_symbols.clone(),
        "object_contains_codegened_function": object_emission.object_contains_codegened_function,
        "object_derived_from": object_emission.object_derived_from.clone(),
        "object_codegen_source": object_emission.object_codegen_source.clone(),
        "codegen_lowering_invoked": object_emission.codegen_lowering_invoked,
        "codegen_lowering_entrypoint": object_emission.codegen_lowering_entrypoint.clone(),
        "codegen_lowering_status": codegen_lowering_status,
        "codegen_lowering_blocker_kind": codegen_lowering_blocker_kind,
        "codegen_lowering_blocker_component": codegen_lowering_blocker_component,
        "codegen_lowering_blocker_reason": codegen_lowering_blocker_reason,
        "codegen_lowering_required_path": CODEGEN_LOWERING_PATH,
        "codegen_lowering_missing_inputs": CODEGEN_LOWERING_MISSING_INPUTS,
        "object_inspection": object_emission.inspection.clone(),
        "object_format": object_emission.inspection.as_ref().map(|inspection| inspection.object_format.as_str()),
        "object_section_count": object_emission.inspection.as_ref().map(|inspection| inspection.object_section_count),
        "object_has_code_section": object_emission.inspection.as_ref().map(|inspection| inspection.object_has_code_section),
        "object_has_linking_metadata": object_emission.inspection.as_ref().map(|inspection| inspection.object_has_linking_metadata),
        "object_symbol_count": object_emission.inspection.as_ref().map(|inspection| inspection.object_symbol_count),
        "object_function_count": object_emission.inspection.as_ref().map(|inspection| inspection.object_function_count),
        "object_is_empty": object_emission.inspection.as_ref().map(|inspection| inspection.object_is_empty),
        "object_has_code_bearing_content": object_emission.inspection.as_ref().map(|inspection| inspection.object_has_code_bearing_content),
        "blocker_kind": blocker_kind,
        "blocker_component": blocker_component,
        "blocker_reason": blocker_reason
    });

    match serde_json::to_string_pretty(&payload) {
        Ok(serialized) => {
            if input.json {
                println!("{serialized}");
            } else {
                println!(
                    "rustc_codegen_llvm probe: backend={} state={}",
                    backend_name, codegen_contact_state
                );
            }
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("failed to serialize rustc_codegen_llvm probe: {error}");
            ExitCode::from(1)
        }
    }
}

fn parse_input() -> Result<ProbeInput, String> {
    let mut input = ProbeInput::default();
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--json" => input.json = true,
            "--compile-unit-id" => input.compile_unit_id = next_value(&mut args, &arg)?,
            "--package" => input.package = next_value(&mut args, &arg)?,
            "--target" => input.target = next_value(&mut args, &arg)?,
            "--target-kind" => input.target_kind = next_value(&mut args, &arg)?,
            "--profile" => input.profile = next_value(&mut args, &arg)?,
            "--source-path" => input.source_path = next_value(&mut args, &arg)?,
            "--source-sha256" => input.source_sha256 = next_value(&mut args, &arg)?,
            "--source-hex" => {
                input.source_bytes = decode_hex(&next_value(&mut args, &arg)?)?;
            }
            "--crate-identity" => input.crate_identity = next_value(&mut args, &arg)?,
            "--crate-artifact-kind" => input.crate_artifact_kind = next_value(&mut args, &arg)?,
            "--target-triple" => input.target_triple = next_value(&mut args, &arg)?,
            "--target-spec" => input.target_spec = next_value(&mut args, &arg)?,
            "--mir-body-hash" => input.mir_body_hash = next_value(&mut args, &arg)?,
            "--mono-item-count" => {
                input.mono_item_count = next_value(&mut args, &arg)?
                    .parse()
                    .map_err(|error| format!("invalid --mono-item-count: {error}"))?;
            }
            "--mono-item-graph-hash" => input.mono_item_graph_hash = next_value(&mut args, &arg)?,
            "--mono-item" => input.mono_items.push(next_value(&mut args, &arg)?),
            "--extern-artifact" => {
                input
                    .extern_artifacts
                    .push(parse_extern_artifact(&next_value(&mut args, &arg)?)?);
            }
            "--link-dependency-artifact" => {
                input
                    .link_dependency_artifacts
                    .push(parse_link_dependency_artifact(&next_value(
                        &mut args, &arg,
                    )?)?);
            }
            unknown => return Err(format!("unknown argument: {unknown}")),
        }
    }
    if input.mono_items.is_empty() {
        input.mono_items.push("fn:rouwdi_payload::main".to_owned());
    }
    if input.source_path.trim().is_empty() {
        return Err("--source-path is required".to_owned());
    }
    if input.source_bytes.is_empty() {
        return Err("--source-hex is required and must not be empty".to_owned());
    }
    let actual_source_sha256 = sha256_hex(&input.source_bytes);
    if input.source_sha256.trim().is_empty() {
        input.source_sha256 = actual_source_sha256;
    } else if input.source_sha256 != actual_source_sha256 {
        return Err(format!(
            "--source-sha256 does not match --source-hex bytes: expected {}, got {}",
            input.source_sha256, actual_source_sha256
        ));
    }
    Ok(input)
}

fn parse_extern_artifact(value: &str) -> Result<ProbeExternArtifact, String> {
    let mut parts = value.splitn(3, '=');
    let extern_crate_name = parts
        .next()
        .filter(|part| !part.trim().is_empty())
        .ok_or_else(|| "--extern-artifact requires name=path=sha256".to_owned())?;
    let path = parts
        .next()
        .filter(|part| !part.trim().is_empty())
        .ok_or_else(|| "--extern-artifact requires name=path=sha256".to_owned())?;
    let sha256 = parts
        .next()
        .filter(|part| part.len() == 64)
        .ok_or_else(|| "--extern-artifact requires a 64-character sha256".to_owned())?;
    Ok(ProbeExternArtifact {
        extern_crate_name: extern_crate_name.to_owned(),
        path: path.to_owned(),
        sha256: sha256.to_owned(),
    })
}

fn parse_link_dependency_artifact(value: &str) -> Result<ProbeLinkDependencyArtifact, String> {
    let mut parts = value.rsplitn(2, '=');
    let sha256 = parts
        .next()
        .filter(|part| part.len() == 64)
        .ok_or_else(|| "--link-dependency-artifact requires path=sha256".to_owned())?;
    let path = parts
        .next()
        .filter(|part| !part.trim().is_empty())
        .ok_or_else(|| "--link-dependency-artifact requires path=sha256".to_owned())?;
    Ok(ProbeLinkDependencyArtifact {
        path: path.to_owned(),
        sha256: sha256.to_owned(),
    })
}

fn next_value(args: &mut impl Iterator<Item = String>, option: &str) -> Result<String, String> {
    args.next()
        .ok_or_else(|| format!("{option} requires a value"))
}

fn decode_hex(value: &str) -> Result<Vec<u8>, String> {
    if value.len() % 2 != 0 {
        return Err("--source-hex must have an even number of digits".to_owned());
    }
    let mut bytes = Vec::with_capacity(value.len() / 2);
    for chunk in value.as_bytes().chunks_exact(2) {
        let hi = hex_digit(chunk[0])?;
        let lo = hex_digit(chunk[1])?;
        bytes.push((hi << 4) | lo);
    }
    Ok(bytes)
}

fn hex_digit(byte: u8) -> Result<u8, String> {
    match byte {
        b'0'..=b'9' => Ok(byte - b'0'),
        b'a'..=b'f' => Ok(byte - b'a' + 10),
        b'A'..=b'F' => Ok(byte - b'A' + 10),
        _ => Err(format!(
            "invalid hex digit in --source-hex: {}",
            byte as char
        )),
    }
}

fn attempt_llvm_module_setup(input: &ProbeInput) -> ModuleSetup {
    let module_name = match CString::new(input.compile_unit_id.as_str()) {
        Ok(value) => value,
        Err(error) => {
            return ModuleSetup {
                attempted: true,
                llvm_context_created: false,
                llvm_module_created: false,
                module_identity: None,
                module_identity_hash: None,
                module_target_triple: None,
                llvm_ir: None,
                llvm_ir_sha256: None,
                llvm_ir_size_bytes: None,
                blocker_kind: Some("llvm_module_blocked_at_invalid_module_name".to_owned()),
                blocker_reason: Some(error.to_string()),
            };
        }
    };
    let target_triple = match CString::new(input.target_triple.as_str()) {
        Ok(value) => value,
        Err(error) => {
            return ModuleSetup {
                attempted: true,
                llvm_context_created: false,
                llvm_module_created: false,
                module_identity: None,
                module_identity_hash: None,
                module_target_triple: None,
                llvm_ir: None,
                llvm_ir_sha256: None,
                llvm_ir_size_bytes: None,
                blocker_kind: Some("llvm_module_blocked_at_invalid_target_triple".to_owned()),
                blocker_reason: Some(error.to_string()),
            };
        }
    };

    unsafe {
        let context = LLVMContextCreate();
        if context.is_null() {
            return ModuleSetup {
                attempted: true,
                llvm_context_created: false,
                llvm_module_created: false,
                module_identity: None,
                module_identity_hash: None,
                module_target_triple: None,
                llvm_ir: None,
                llvm_ir_sha256: None,
                llvm_ir_size_bytes: None,
                blocker_kind: Some("llvm_module_blocked_at_context_create_null".to_owned()),
                blocker_reason: Some("LLVMContextCreate returned null".to_owned()),
            };
        }

        let module = LLVMModuleCreateWithNameInContext(module_name.as_ptr(), context);
        if module.is_null() {
            LLVMContextDispose(context);
            return ModuleSetup {
                attempted: true,
                llvm_context_created: true,
                llvm_module_created: false,
                module_identity: None,
                module_identity_hash: None,
                module_target_triple: None,
                llvm_ir: None,
                llvm_ir_sha256: None,
                llvm_ir_size_bytes: None,
                blocker_kind: Some("llvm_module_blocked_at_module_create_null".to_owned()),
                blocker_reason: Some("LLVMModuleCreateWithNameInContext returned null".to_owned()),
            };
        }

        LLVMSetTarget(module, target_triple.as_ptr());
        let identity = format!(
            "module={};target={};mir={};mono={}",
            input.compile_unit_id,
            input.target_triple,
            input.mir_body_hash,
            input.mono_item_graph_hash
        );
        let identity_hash = sha256_hex(identity.as_bytes());
        let ir_message = LLVMPrintModuleToString(module);
        let llvm_ir = if ir_message.is_null() {
            None
        } else {
            let ir = CStr::from_ptr(ir_message).to_string_lossy().into_owned();
            LLVMDisposeMessage(ir_message);
            Some(ir)
        };
        let llvm_ir_sha256 = llvm_ir.as_deref().map(|ir| sha256_hex(ir.as_bytes()));
        let llvm_ir_size_bytes = llvm_ir.as_ref().map(|ir| ir.len());
        LLVMDisposeModule(module);
        LLVMContextDispose(context);

        ModuleSetup {
            attempted: true,
            llvm_context_created: true,
            llvm_module_created: true,
            module_identity: Some(identity),
            module_identity_hash: Some(identity_hash),
            module_target_triple: Some(input.target_triple.clone()),
            llvm_ir,
            llvm_ir_sha256,
            llvm_ir_size_bytes,
            blocker_kind: None,
            blocker_reason: None,
        }
    }
}

fn attempt_target_machine_setup(input: &ProbeInput) -> TargetMachineSetup {
    let cpu = "generic".to_owned();
    let features = String::new();
    let relocation_model = "pic".to_owned();
    let code_model = "default".to_owned();
    let optimization_level = "none".to_owned();
    let triple = match CString::new(input.target_triple.as_str()) {
        Ok(value) => value,
        Err(error) => {
            return TargetMachineSetup {
                attempted: true,
                target_triple: input.target_triple.clone(),
                cpu,
                features,
                relocation_model,
                code_model,
                optimization_level,
                target_machine_created: false,
                blocker_kind: Some("target_machine_blocked_at_invalid_target_triple".to_owned()),
                blocker_reason: Some(error.to_string()),
            };
        }
    };
    let cpu_c = CString::new(cpu.as_str()).expect("static CPU has no nul");
    let features_c = CString::new(features.as_str()).expect("static features have no nul");

    unsafe {
        LLVMInitializeWebAssemblyTargetInfo();
        LLVMInitializeWebAssemblyTarget();
        LLVMInitializeWebAssemblyTargetMC();
        LLVMInitializeWebAssemblyAsmPrinter();

        let mut target = std::ptr::null_mut();
        let mut error_message = std::ptr::null_mut();
        let lookup_failed =
            LLVMGetTargetFromTriple(triple.as_ptr(), &mut target, &mut error_message);
        if lookup_failed != 0 || target.is_null() {
            let reason = if error_message.is_null() {
                "LLVMGetTargetFromTriple returned no target".to_owned()
            } else {
                let message = CStr::from_ptr(error_message).to_string_lossy().into_owned();
                LLVMDisposeMessage(error_message);
                message
            };
            return TargetMachineSetup {
                attempted: true,
                target_triple: input.target_triple.clone(),
                cpu,
                features,
                relocation_model,
                code_model,
                optimization_level,
                target_machine_created: false,
                blocker_kind: Some("target_machine_blocked_at_missing_llvm_target".to_owned()),
                blocker_reason: Some(reason),
            };
        }

        let target_machine = LLVMCreateTargetMachine(
            target,
            triple.as_ptr(),
            cpu_c.as_ptr(),
            features_c.as_ptr(),
            0,
            2,
            0,
        );
        if target_machine.is_null() {
            return TargetMachineSetup {
                attempted: true,
                target_triple: input.target_triple.clone(),
                cpu,
                features,
                relocation_model,
                code_model,
                optimization_level,
                target_machine_created: false,
                blocker_kind: Some(
                    "target_machine_blocked_at_create_target_machine_null".to_owned(),
                ),
                blocker_reason: Some("LLVMCreateTargetMachine returned null".to_owned()),
            };
        }

        LLVMDisposeTargetMachine(target_machine);
        TargetMachineSetup {
            attempted: true,
            target_triple: input.target_triple.clone(),
            cpu,
            features,
            relocation_model,
            code_model,
            optimization_level,
            target_machine_created: true,
            blocker_kind: None,
            blocker_reason: None,
        }
    }
}

fn attempt_object_emission(input: &ProbeInput) -> ObjectEmissionSetup {
    let api = "LLVMTargetMachineEmitToMemoryBuffer(LLVMObjectFile)".to_owned();
    let target_triple = input.target_triple.clone();
    match emit_object_with_upstream_rustc_codegen(input) {
        Ok(lowered) => {
            let inspection = inspect_wasm_object(&lowered.object_bytes);
            let codegened_symbols =
                matching_codegened_symbols(&inspection, &lowered.codegened_symbols);
            let object_contains_codegened_function =
                inspection.object_has_code_bearing_content && !codegened_symbols.is_empty();
            let codegened_mono_item_count = codegened_symbols.len() as u64;

            ObjectEmissionSetup {
                attempted: true,
                api,
                codegen_lowering_invoked: true,
                codegen_lowering_entrypoint: UPSTREAM_CODEGEN_ENTRYPOINT.to_owned(),
                object_bytes_emitted: true,
                wasm_object_bytes_emitted: input.target_triple.starts_with("wasm32"),
                inspection: Some(inspection),
                object_contains_codegened_function,
                codegened_mono_item_count,
                codegened_symbols,
                object_derived_from: "rustc_codegen_llvm::LlvmCodegenBackend::codegen_crate"
                    .to_owned(),
                object_codegen_source: if object_contains_codegened_function {
                    "vfs_compile_unit_source".to_owned()
                } else {
                    "rustc_codegen_llvm_lowering_without_matching_mono_symbol".to_owned()
                },
                artifact_kind: Some(if input.target_triple.starts_with("wasm32") {
                    "wasm_object".to_owned()
                } else {
                    "native_object".to_owned()
                }),
                artifact_sha256: Some(sha256_hex(&lowered.object_bytes)),
                artifact_size_bytes: Some(lowered.object_bytes.len()),
                artifact_location: Some(format!("vfs:/workspace/{OBJECT_ARTIFACT_PATH}")),
                target_triple,
                retrieval_method: Some("rouwdi_owned_virtual_fs".to_owned()),
                crate_artifact: lowered.crate_artifact,
                blocker_kind: None,
                blocker_reason: None,
            }
        }
        Err(error) => object_blocked(
            api,
            target_triple,
            "object_emission_attempted_blocked_at_rustc_codegen_llvm_lowering",
            error,
        ),
    }
}

struct LoweredObject {
    object_bytes: Vec<u8>,
    additional_link_object_paths: Vec<String>,
    codegened_symbols: Vec<String>,
    crate_artifact: Option<CrateArtifactOutput>,
}

#[allow(rustc::bad_opt_access)]
fn emit_object_with_upstream_rustc_codegen(input: &ProbeInput) -> Result<LoweredObject, String> {
    let _ = std::fs::remove_file(OBJECT_ARTIFACT_PATH);

    eprintln!("rouwdi-codegen-progress: rustc-interface-config-start");
    let source = String::from_utf8(input.source_bytes.clone())
        .map_err(|error| format!("codegen input source is not valid UTF-8: {error}"))?;
    let actual_source_sha256 = sha256_hex(source.as_bytes());
    if actual_source_sha256 != input.source_sha256 {
        return Err(format!(
            "codegen input source hash mismatch: expected {}, got {}",
            input.source_sha256, actual_source_sha256
        ));
    }
    let mut opts = rustc_session::config::Options::default();
    let sysroot_path = PathBuf::from(sysroot_path());
    let target_libdir = sysroot_path
        .join("lib")
        .join("rustlib")
        .join(&input.target_triple)
        .join("lib");
    opts.crate_types = vec![if input.crate_artifact_kind == "library" {
        rustc_session::config::CrateType::Rlib
    } else {
        rustc_session::config::CrateType::Executable
    }];
    opts.crate_name = Some(sanitize_crate_name(&input.crate_identity));
    opts.sysroot = rustc_session::config::Sysroot::new(Some(sysroot_path.clone()));
    opts.target_triple = rustc_target::spec::TargetTuple::from_tuple(&input.target_triple);
    opts.search_paths.push(
        rustc_session::search_paths::SearchPath::from_sysroot_and_triple(
            &sysroot_path,
            &input.target_triple,
        ),
    );
    opts.search_paths
        .push(rustc_session::search_paths::SearchPath::new(
            rustc_session::search_paths::PathKind::Dependency,
            target_libdir.clone(),
        ));
    opts.search_paths
        .push(rustc_session::search_paths::SearchPath::new(
            rustc_session::search_paths::PathKind::Crate,
            target_libdir,
        ));
    for extern_artifact in &input.extern_artifacts {
        if let Some(parent) = PathBuf::from(&extern_artifact.path).parent() {
            opts.search_paths
                .push(rustc_session::search_paths::SearchPath::new(
                    rustc_session::search_paths::PathKind::Dependency,
                    parent.to_path_buf(),
                ));
        }
    }
    for dependency_artifact in &input.link_dependency_artifacts {
        if let Some(parent) = PathBuf::from(&dependency_artifact.path).parent() {
            opts.search_paths
                .push(rustc_session::search_paths::SearchPath::new(
                    rustc_session::search_paths::PathKind::Dependency,
                    parent.to_path_buf(),
                ));
        }
    }
    let externs = input
        .extern_artifacts
        .iter()
        .map(|artifact| {
            let mut locations = BTreeSet::new();
            locations.insert(rustc_session::utils::CanonicalizedPath::new(PathBuf::from(
                &artifact.path,
            )));
            (
                artifact.extern_crate_name.clone(),
                rustc_session::config::ExternEntry {
                    location: rustc_session::config::ExternLocation::ExactPaths(locations),
                    is_private_dep: false,
                    add_prelude: true,
                    nounused_dep: false,
                    force: false,
                },
            )
        })
        .collect::<BTreeMap<_, _>>();
    opts.externs = rustc_session::config::Externs::new(externs);
    opts.output_types = rustc_session::config::OutputTypes::new(&[(
        rustc_session::config::OutputType::Object,
        Some(rustc_session::config::OutFileName::Real(PathBuf::from(
            OBJECT_ARTIFACT_PATH,
        ))),
    )]);
    opts.unstable_features = rustc_feature::UnstableFeatures::Cheat;
    opts.cg.panic = Some(rustc_target::spec::PanicStrategy::Abort);
    opts.cg.link_dead_code = Some(true);
    opts.cg.codegen_units = Some(1);
    opts.cg.embed_bitcode = false;
    opts.cg.relocation_model = Some(rustc_target::spec::RelocModel::Pic);
    opts.unstable_opts.no_parallel_backend = true;
    opts.unstable_opts.panic_in_drop = rustc_target::spec::PanicStrategy::Abort;
    opts.edition = rustc_span::edition::Edition::Edition2021;

    let config = rustc_interface::Config {
        opts,
        crate_cfg: Vec::new(),
        crate_check_cfg: Vec::new(),
        input: rustc_session::config::Input::Str {
            name: rustc_span::FileName::Custom(input.source_path.clone()),
            input: source,
        },
        output_dir: None,
        output_file: None,
        ice_file: None,
        file_loader: None,
        lint_caps: Default::default(),
        psess_created: None,
        track_state: None,
        register_lints: None,
        override_queries: None,
        extra_symbols: Vec::new(),
        make_codegen_backend: Some(Box::new(|_sess| {
            rustc_codegen_llvm::LlvmCodegenBackend::new()
        })),
        using_internal_features: &USING_INTERNAL_FEATURES,
    };

    eprintln!("rouwdi-codegen-progress: rustc-interface-run-start");
    let compile_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(move || {
        rustc_interface::run_compiler(
            config,
            |compiler| -> Result<Vec<rustc_codegen_llvm::RouwdiWasmObject>, String> {
                eprintln!("rouwdi-codegen-progress: rustc-parse-start");
                let krate = rustc_interface::parse(&compiler.sess);
                eprintln!("rouwdi-codegen-progress: global-ctxt-start");
                rustc_interface::create_and_enter_global_ctxt(compiler, krate, |tcx| {
                    eprintln!("rouwdi-codegen-progress: resolver-start");
                    let _ = tcx.resolver_for_lowering();
                    rustc_interface::passes::write_dep_info(tcx);
                    rustc_interface::passes::write_interface(tcx);
                    eprintln!("rouwdi-codegen-progress: analysis-start");
                    tcx.ensure_ok().analysis(());
                    let metadata_bytes = if input.crate_artifact_kind == "library" {
                        eprintln!("rouwdi-codegen-progress: metadata-emit-start");
                        let metadata = rustc_metadata::fs::encode_and_write_metadata(tcx);
                        let bytes = metadata.stub_or_full().to_vec();
                        eprintln!(
                            "rouwdi-codegen-progress: metadata-emit-complete bytes={}",
                            bytes.len()
                        );
                        Some(bytes)
                    } else {
                        None
                    };
                    eprintln!("rouwdi-codegen-progress: sync-codegen-start");
                    let mut object_bytes = rustc_codegen_llvm::rouwdi_codegen_wasm_objects(tcx)?;
                    if let Some(bytes) = metadata_bytes {
                        object_bytes.push(rustc_codegen_llvm::RouwdiWasmObject {
                            role: "rust_metadata".to_owned(),
                            path: "lib.rmeta".to_owned(),
                            bytes,
                        });
                    }
                    eprintln!("rouwdi-codegen-progress: sync-codegen-returned");
                    tcx.report_unused_features();
                    Ok(object_bytes)
                })
            },
        )
    }));
    eprintln!("rouwdi-codegen-progress: rustc-interface-run-returned");
    let objects = match compile_result {
        Ok(Ok(objects)) => objects,
        Ok(Err(error)) => return Err(error),
        Err(payload) => {
            return Err(format!(
                "rustc_interface/rustc_codegen_llvm panicked while lowering the mono item: {}",
                panic_payload_to_string(payload),
            ));
        }
    };

    let Some(primary_object) = objects
        .iter()
        .find(|object| object.role == "mono_item_codegen_unit")
    else {
        return Err("rustc_codegen_llvm emitted no mono-item codegen object".to_owned());
    };
    let object_bytes = primary_object.bytes.clone();
    if object_bytes.is_empty() {
        return Err("rustc_codegen_llvm emitted an empty object buffer".to_owned());
    }
    for object in &objects {
        std::fs::write(&object.path, &object.bytes).map_err(|error| {
            format!(
                "failed to mirror rustc_codegen_llvm {} object to canonical rouwdi VFS path {}: {error}",
                object.role, object.path,
            )
        })?;
    }

    let crate_artifact = if input.crate_artifact_kind == "library" {
        Some(build_dependency_rlib_from_objects(input, &objects)?)
    } else {
        None
    };

    Ok(LoweredObject {
        object_bytes,
        additional_link_object_paths: objects
            .iter()
            .filter(|object| {
                object.role != "mono_item_codegen_unit" && object.role != "rust_metadata"
            })
            .map(|object| object.path.clone())
            .collect(),
        codegened_symbols: expected_codegened_symbols(input),
        crate_artifact,
    })
}

fn build_dependency_rlib_from_objects(
    input: &ProbeInput,
    objects: &[rustc_codegen_llvm::RouwdiWasmObject],
) -> Result<CrateArtifactOutput, String> {
    let crate_name = sanitize_crate_name(&input.crate_identity);
    let artifact_path = format!("lib{crate_name}.rlib");
    let _ = std::fs::remove_file(&artifact_path);

    let metadata = objects
        .iter()
        .find(|object| object.role == "rust_metadata")
        .ok_or_else(|| "library compile emitted no Rust metadata".to_owned())?;
    if metadata.bytes.is_empty() {
        return Err("library compile emitted empty Rust metadata".to_owned());
    }

    let object_members = objects
        .iter()
        .filter(|object| object.role != "rust_metadata")
        .filter(|object| !object.bytes.is_empty())
        .collect::<Vec<_>>();
    if object_members.is_empty() {
        return Err("library compile emitted no linkable object members".to_owned());
    }

    let mut members = Vec::with_capacity(object_members.len() + 1);
    members.push(("lib.rmeta".to_owned(), metadata.bytes.clone()));
    for (index, object) in object_members.iter().enumerate() {
        let name = if object.role == "allocator_shim_codegen_unit" {
            "alloc.o".to_owned()
        } else if index == 0 {
            "lib.o".to_owned()
        } else {
            format!("lib{index}.o")
        };
        members.push((name, object.bytes.clone()));
    }

    let bytes = write_gnu_archive(&members)?;
    std::fs::write(&artifact_path, &bytes)
        .map_err(|error| format!("dependency rlib {artifact_path} was not written: {error}"))?;
    let (metadata_member, metadata_sha256) = rlib_metadata_member_hash(&bytes);
    Ok(CrateArtifactOutput {
        artifact_path: artifact_path.clone(),
        artifact_sha256: sha256_hex(&bytes),
        artifact_size_bytes: bytes.len(),
        metadata_sha256,
        metadata_member,
        bytes,
    })
}

fn write_gnu_archive(members: &[(String, Vec<u8>)]) -> Result<Vec<u8>, String> {
    let mut archive = b"!<arch>\n".to_vec();
    for (name, data) in members {
        if name.is_empty() {
            return Err("archive member name cannot be empty".to_owned());
        }
        if name.len() > 15 {
            return Err(format!(
                "archive member name {name} is too long for the rouwdi dependency archive writer"
            ));
        }
        let header = format!(
            "{:<16}{:<12}{:<6}{:<6}{:<8}{:<10}`\n",
            format!("{name}/"),
            0,
            0,
            0,
            0o100644,
            data.len()
        );
        if header.len() != 60 {
            return Err(format!(
                "archive member {name} produced invalid header length {}",
                header.len()
            ));
        }
        archive.extend_from_slice(header.as_bytes());
        archive.extend_from_slice(data);
        if data.len() % 2 != 0 {
            archive.push(b'\n');
        }
    }
    Ok(archive)
}

fn add_extern_artifacts_to_options(input: &ProbeInput, opts: &mut rustc_session::config::Options) {
    for extern_artifact in &input.extern_artifacts {
        if let Some(parent) = PathBuf::from(&extern_artifact.path).parent() {
            opts.search_paths
                .push(rustc_session::search_paths::SearchPath::new(
                    rustc_session::search_paths::PathKind::Dependency,
                    parent.to_path_buf(),
                ));
        }
    }
    let externs = input
        .extern_artifacts
        .iter()
        .map(|artifact| {
            let mut locations = BTreeSet::new();
            locations.insert(rustc_session::utils::CanonicalizedPath::new(PathBuf::from(
                &artifact.path,
            )));
            (
                artifact.extern_crate_name.clone(),
                rustc_session::config::ExternEntry {
                    location: rustc_session::config::ExternLocation::ExactPaths(locations),
                    is_private_dep: false,
                    add_prelude: true,
                    nounused_dep: false,
                    force: false,
                },
            )
        })
        .collect::<BTreeMap<_, _>>();
    opts.externs = rustc_session::config::Externs::new(externs);
}

fn rlib_metadata_member_hash(bytes: &[u8]) -> (Option<String>, Option<String>) {
    if !bytes.starts_with(b"!<arch>\n") {
        return (None, None);
    }
    let mut offset = 8usize;
    while offset.saturating_add(60) <= bytes.len() {
        let header = &bytes[offset..offset + 60];
        let raw_name = String::from_utf8_lossy(&header[0..16])
            .trim()
            .trim_end_matches('/')
            .to_owned();
        let size_text = String::from_utf8_lossy(&header[48..58]).trim().to_owned();
        let Ok(size) = size_text.parse::<usize>() else {
            break;
        };
        let data_start = offset + 60;
        let data_end = data_start.saturating_add(size);
        if data_end > bytes.len() {
            break;
        }
        let data = &bytes[data_start..data_end];
        if raw_name.contains("rmeta")
            || data
                .windows(b"rustc".len())
                .any(|window| window == b"rustc")
        {
            return (Some(raw_name), Some(sha256_hex(data)));
        }
        offset = data_end + (size % 2);
    }
    (None, Some(sha256_hex(bytes)))
}

fn linker_payload_identity() -> LinkerPayloadIdentity {
    let mut digest = Sha256::new();
    digest.update(b"rouwdi-wasm-ld\0librouwdi-lld-wasm-wrapper.a\0");
    digest.update(LLD_WRAPPER_ARCHIVE);
    digest.update(b"\0liblldWasm.a\0");
    digest.update(LLD_WASM_ARCHIVE);
    digest.update(b"\0liblldCommon.a\0");
    digest.update(LLD_COMMON_ARCHIVE);
    LinkerPayloadIdentity {
        sha256: hex::encode(digest.finalize()),
        size_bytes: LLD_WRAPPER_ARCHIVE.len() + LLD_WASM_ARCHIVE.len() + LLD_COMMON_ARCHIVE.len(),
    }
}

fn attempt_wasm_link(
    input: &ProbeInput,
    object_emission: &ObjectEmissionSetup,
    _linker_payload: &LinkerPayloadIdentity,
) -> LinkExecution {
    let object_hash = object_emission
        .artifact_sha256
        .clone()
        .unwrap_or_else(|| "missing-object-hash".to_owned());
    let object_path = OBJECT_ARTIFACT_PATH.to_owned();
    let allocator_object_hash = std::fs::read(ALLOCATOR_OBJECT_ARTIFACT_PATH)
        .ok()
        .filter(|bytes| !bytes.is_empty())
        .map(|bytes| sha256_hex(&bytes));
    let mut input_artifact_hashes = vec![object_hash.clone()];
    if let Some(hash) = &allocator_object_hash {
        input_artifact_hashes.push(hash.clone());
    }
    for dependency in &input.link_dependency_artifacts {
        let actual_hash = std::fs::read(&dependency.path)
            .ok()
            .map(|bytes| sha256_hex(&bytes))
            .unwrap_or_else(|| "missing-dependency-artifact".to_owned());
        input_artifact_hashes.push(actual_hash);
    }
    let output_path = FINAL_WASI_MODULE_PATH.to_owned();
    let target_libdir = format!("{}/lib/rustlib/{}/lib", sysroot_path(), input.target_triple);
    let self_contained_dir = format!("{target_libdir}/self-contained");
    let mut command_args = vec![
        "wasm-ld".to_owned(),
        "--export".to_owned(),
        "__main_void".to_owned(),
        "-z".to_owned(),
        "stack-size=1048576".to_owned(),
        "--stack-first".to_owned(),
        "--no-demangle".to_owned(),
        format!("{self_contained_dir}/crt1-command.o"),
        object_path,
    ];
    if allocator_object_hash.is_some() {
        command_args.push(ALLOCATOR_OBJECT_ARTIFACT_PATH.to_owned());
    }
    for dependency in &input.link_dependency_artifacts {
        command_args.push(dependency.path.clone());
    }
    command_args.extend([
        format!("{target_libdir}/libpanic_abort-771e1103f866bdb4.rlib"),
        format!("{target_libdir}/libstd-b594a2ae141e7c9c.rlib"),
        format!("{target_libdir}/libwasip1-bdf89526125af68e.rlib"),
        format!("{target_libdir}/libcfg_if-f330595bed847612.rlib"),
        format!("{target_libdir}/librustc_demangle-3e5dfd60db0f61c6.rlib"),
        format!("{target_libdir}/libstd_detect-992133543cee23b6.rlib"),
        format!("{target_libdir}/libhashbrown-82db15a0bc02cc07.rlib"),
        format!("{target_libdir}/librustc_std_workspace_alloc-4243392e063083ab.rlib"),
        format!("{target_libdir}/libminiz_oxide-b1bb72b0c937980d.rlib"),
        format!("{target_libdir}/libadler2-6c6ce22a3d784b53.rlib"),
        format!("{target_libdir}/libunwind-801234150e5ffd1c.rlib"),
        format!("{target_libdir}/liblibc-d076481cb93820f2.rlib"),
        "-l".to_owned(),
        "c".to_owned(),
        format!("{target_libdir}/librustc_std_workspace_core-40703e9aafc1d450.rlib"),
        format!("{target_libdir}/liballoc-aa5de9cb44693937.rlib"),
        format!("{target_libdir}/libcore-fc7b12ec85c54ac0.rlib"),
        format!("{target_libdir}/libcompiler_builtins-242fe6d76c147fd1.rlib"),
        "-L".to_owned(),
        self_contained_dir,
        "-o".to_owned(),
        output_path.clone(),
        "--gc-sections".to_owned(),
        "-O3".to_owned(),
    ]);

    let mut stdout = String::new();
    let mut stderr = String::new();
    let exit_code = invoke_embedded_lld(&command_args, &mut stdout, &mut stderr);
    if exit_code != 0 {
        return LinkExecution {
            linker_invoked: true,
            command_args,
            input_artifact_hashes,
            output_artifact_path: Some(format!("vfs:/workspace/{FINAL_WASI_MODULE_PATH}")),
            stdout,
            stderr: if stderr.trim().is_empty() {
                "embedded wasm-ld returned a non-zero exit code without stderr".to_owned()
            } else {
                stderr
            },
            exit_code,
            final_module_bytes: None,
            final_module_sha256: None,
            final_module_size_bytes: None,
            interface_proof: None,
            current_status: "wasm_ld_invoked_blocked_at_link_error".to_owned(),
            blocker_kind: "wasm_ld_link_failed".to_owned(),
            blocker_reason: "embedded wasm-ld failed before emitting a final module".to_owned(),
        };
    }

    let final_module_bytes = match std::fs::read(&output_path) {
        Ok(bytes) => bytes,
        Err(error) => {
            return LinkExecution {
                linker_invoked: true,
                command_args,
                input_artifact_hashes,
                output_artifact_path: Some(format!("vfs:/workspace/{FINAL_WASI_MODULE_PATH}")),
                stdout,
                stderr,
                exit_code,
                final_module_bytes: None,
                final_module_sha256: None,
                final_module_size_bytes: None,
                interface_proof: None,
                current_status: "wasm_ld_invoked_blocked_at_missing_output_module".to_owned(),
                blocker_kind: "final_wasi_module_missing_after_link".to_owned(),
                blocker_reason: format!(
                    "embedded wasm-ld exited 0 but {FINAL_WASI_MODULE_PATH} could not be read: {error}"
                ),
            };
        }
    };
    let module_sha256 = sha256_hex(&final_module_bytes);
    let module_size = final_module_bytes.len();
    let interface_proof = build_interface_proof(&final_module_bytes, &module_sha256);
    let (current_status, blocker_kind, blocker_reason) = if interface_proof.passed {
        (
            "interface_proof_passed".to_owned(),
            "none".to_owned(),
            "none".to_owned(),
        )
    } else {
        (
            "wasm_ld_invoked_blocked_at_interface_proof".to_owned(),
            "interface_proof_failed".to_owned(),
            format!(
                "final linked module is missing required exports: {}",
                interface_proof.missing_required_exports.join(", ")
            ),
        )
    };

    LinkExecution {
        linker_invoked: true,
        command_args,
        input_artifact_hashes,
        output_artifact_path: Some(format!("vfs:/workspace/{FINAL_WASI_MODULE_PATH}")),
        stdout,
        stderr,
        exit_code,
        final_module_bytes: Some(final_module_bytes),
        final_module_sha256: Some(module_sha256),
        final_module_size_bytes: Some(module_size),
        interface_proof: Some(interface_proof),
        current_status,
        blocker_kind,
        blocker_reason,
    }
}

fn invoke_embedded_lld(args: &[String], stdout: &mut String, stderr: &mut String) -> i32 {
    let c_args = match args
        .iter()
        .map(|arg| CString::new(arg.as_str()))
        .collect::<Result<Vec<_>, _>>()
    {
        Ok(args) => args,
        Err(error) => {
            *stderr = format!("linker argument contained interior nul: {error}");
            return 1;
        }
    };
    let raw_args = c_args.iter().map(|arg| arg.as_ptr()).collect::<Vec<_>>();
    let mut stdout_ptr: *mut c_char = ptr::null_mut();
    let mut stdout_len = 0usize;
    let mut stderr_ptr: *mut c_char = ptr::null_mut();
    let mut stderr_len = 0usize;
    let exit_code = unsafe {
        rouwdi_lld_wasm_link(
            raw_args.len() as c_int,
            raw_args.as_ptr(),
            &mut stdout_ptr,
            &mut stdout_len,
            &mut stderr_ptr,
            &mut stderr_len,
        )
    };
    *stdout = unsafe { take_lld_string(stdout_ptr, stdout_len) };
    *stderr = unsafe { take_lld_string(stderr_ptr, stderr_len) };
    exit_code
}

unsafe fn take_lld_string(ptr: *mut c_char, len: usize) -> String {
    if ptr.is_null() {
        return String::new();
    }
    let bytes = slice::from_raw_parts(ptr.cast::<u8>(), len);
    let text = String::from_utf8_lossy(bytes).into_owned();
    rouwdi_lld_free(ptr);
    text
}

fn build_interface_proof(bytes: &[u8], module_sha256: &str) -> InterfaceProof {
    let inspection = inspect_wasm_object(bytes);
    let required_exports = vec!["_start".to_owned()];
    let missing_required_exports = required_exports
        .iter()
        .filter(|export| {
            !inspection
                .object_exports
                .iter()
                .any(|found| found == *export)
        })
        .cloned()
        .collect::<Vec<_>>();
    let wasi_imports = inspection
        .object_imports
        .iter()
        .filter(|import| import.starts_with("wasi_snapshot_preview1::"))
        .cloned()
        .collect::<Vec<_>>();
    InterfaceProof {
        wasm_magic_valid: inspection.wasm_magic_valid,
        wasm_version_valid: inspection.wasm_version_valid,
        exports: inspection.object_exports,
        imports: inspection.object_imports,
        start_present: missing_required_exports.is_empty(),
        wasi_imports,
        required_exports_checked: required_exports,
        missing_required_exports: missing_required_exports.clone(),
        module_sha256: module_sha256.to_owned(),
        module_size_bytes: bytes.len(),
        passed: inspection.wasm_magic_valid
            && inspection.wasm_version_valid
            && missing_required_exports.is_empty()
            && inspection.parse_errors.is_empty(),
    }
}

fn interface_proof_json(proof: &InterfaceProof) -> serde_json::Value {
    serde_json::json!({
        "wasm_magic_valid": proof.wasm_magic_valid,
        "wasm_version_valid": proof.wasm_version_valid,
        "exports": proof.exports.clone(),
        "imports": proof.imports.clone(),
        "_start_present": proof.start_present,
        "wasi_imports": proof.wasi_imports.clone(),
        "required_exports_checked": proof.required_exports_checked.clone(),
        "missing_required_exports": proof.missing_required_exports.clone(),
        "module_sha256": proof.module_sha256.clone(),
        "module_size_bytes": proof.module_size_bytes,
        "passed": proof.passed,
    })
}

fn sysroot_path() -> String {
    if cfg!(target_arch = "wasm32") {
        "/workspace/third_party/rust/build/x86_64-pc-windows-msvc/stage1".to_owned()
    } else {
        let mut path = std::env::current_dir().unwrap_or_else(|_| ".".into());
        path.push("third_party");
        path.push("rust");
        path.push("build");
        path.push("x86_64-pc-windows-msvc");
        path.push("stage1");
        path.display().to_string()
    }
}

fn sanitize_crate_name(crate_identity: &str) -> String {
    let mut sanitized = crate_identity
        .chars()
        .map(|ch| if ch.is_ascii_alphanumeric() { ch } else { '_' })
        .collect::<String>();
    if sanitized.is_empty() || sanitized.as_bytes()[0].is_ascii_digit() {
        sanitized.insert(0, '_');
    }
    sanitized
}

fn expected_codegened_symbols(input: &ProbeInput) -> Vec<String> {
    let mut symbols = input
        .mono_items
        .iter()
        .filter_map(|item| {
            if item.contains("::main") || item.ends_with(":main") {
                Some("main".to_owned())
            } else {
                item.rsplit("::")
                    .next()
                    .or_else(|| item.rsplit(':').next())
                    .map(str::to_owned)
            }
        })
        .filter(|symbol| !symbol.trim().is_empty())
        .collect::<Vec<_>>();
    symbols.sort();
    symbols.dedup();
    symbols
}

fn matching_codegened_symbols(
    inspection: &WasmObjectInspection,
    expected_symbols: &[String],
) -> Vec<String> {
    let mut matches = Vec::new();
    for expected in expected_symbols {
        for symbol in &inspection.object_symbols {
            let Some(name) = symbol.name.as_deref() else {
                continue;
            };
            if symbol.kind == "function"
                && !symbol.undefined
                && (name == expected || name.ends_with(expected))
            {
                matches.push(name.to_owned());
            }
        }
        for export in &inspection.object_exports {
            if export == expected || export.ends_with(expected) {
                matches.push(export.clone());
            }
        }
    }
    matches.sort();
    matches.dedup();
    matches
}

fn panic_payload_to_string(payload: Box<dyn std::any::Any + Send>) -> String {
    if let Some(message) = payload.downcast_ref::<&str>() {
        (*message).to_owned()
    } else if let Some(message) = payload.downcast_ref::<String>() {
        message.clone()
    } else {
        "non-string panic payload".to_owned()
    }
}

fn object_blocked(
    api: String,
    target_triple: String,
    blocker_kind: &str,
    blocker_reason: String,
) -> ObjectEmissionSetup {
    ObjectEmissionSetup {
        attempted: true,
        api,
        codegen_lowering_invoked: false,
        codegen_lowering_entrypoint: UPSTREAM_CODEGEN_ENTRYPOINT.to_owned(),
        object_bytes_emitted: false,
        wasm_object_bytes_emitted: false,
        inspection: None,
        object_contains_codegened_function: false,
        codegened_mono_item_count: 0,
        codegened_symbols: Vec::new(),
        object_derived_from:
            "rustc_codegen_llvm::LlvmCodegenBackend::new + LLVMTargetMachineEmitToMemoryBuffer"
                .to_owned(),
        object_codegen_source: "object_emission_blocked_before_module_inspection".to_owned(),
        artifact_kind: None,
        artifact_sha256: None,
        artifact_size_bytes: None,
        artifact_location: None,
        target_triple,
        retrieval_method: None,
        crate_artifact: None,
        blocker_kind: Some(blocker_kind.to_owned()),
        blocker_reason: Some(blocker_reason),
    }
}

unsafe fn llvm_error_message(message: *mut c_char, fallback: String) -> String {
    if message.is_null() {
        fallback
    } else {
        let reason = CStr::from_ptr(message).to_string_lossy().into_owned();
        LLVMDisposeMessage(message);
        reason
    }
}

fn target_machine_identity(setup: &TargetMachineSetup) -> Option<String> {
    if setup.target_machine_created {
        Some(format!(
            "target_machine=LLVM;target={};cpu={};features={};reloc={};code_model={};opt={}",
            setup.target_triple,
            setup.cpu,
            setup.features,
            setup.relocation_model,
            setup.code_model,
            setup.optimization_level
        ))
    } else {
        None
    }
}

fn sha256_hex(bytes: &[u8]) -> String {
    let mut digest = Sha256::new();
    digest.update(bytes);
    hex::encode(digest.finalize())
}
