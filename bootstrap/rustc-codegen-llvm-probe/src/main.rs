#![recursion_limit = "256"]

use rouwdi_object::{inspect_wasm_object, WasmObjectInspection};
use sha2::{Digest, Sha256};
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int, c_uint, c_void};
use std::process::ExitCode;

#[derive(Debug, Clone)]
struct ProbeInput {
    json: bool,
    compile_unit_id: String,
    crate_identity: String,
    target_triple: String,
    target_spec: String,
    mir_body_hash: String,
    mono_item_count: u64,
    mono_item_graph_hash: String,
    mono_items: Vec<String>,
}

impl Default for ProbeInput {
    fn default() -> Self {
        Self {
            json: false,
            compile_unit_id: "app:rust:app:wasm32-wasip1".to_owned(),
            crate_identity: "rouwdi_payload".to_owned(),
            target_triple: "wasm32-wasip1".to_owned(),
            target_spec: "rustc_target::spec::wasm32_wasip1".to_owned(),
            mir_body_hash: "a5e137ef6793c0b8".to_owned(),
            mono_item_count: 1,
            mono_item_graph_hash: "bec5817d61819666".to_owned(),
            mono_items: Vec::new(),
        }
    }
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
    blocker_kind: Option<String>,
    blocker_reason: Option<String>,
}

type LLVMTargetRef = *mut c_void;
type LLVMTargetMachineRef = *mut c_void;
type LLVMMemoryBufferRef = *mut c_void;

const LLVM_OBJECT_FILE: c_uint = 1;
const OBJECT_ARTIFACT_PATH: &str = "rouwdi-codegen-wasm32-wasip1.o";
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

#[link(name = "llvm-wrapper", kind = "static")]
unsafe extern "C" {}

unsafe extern "C" {
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

fn main() -> ExitCode {
    let input = match parse_input() {
        Ok(input) => input,
        Err(error) => {
            eprintln!("{error}");
            return ExitCode::from(2);
        }
    };

    let backend = rustc_codegen_llvm::LlvmCodegenBackend::new();
    let backend_name = backend.name();
    let module_setup = attempt_llvm_module_setup(&input);
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
    let object_emission = attempt_object_emission(&input);

    let llvm_ir_emitted = module_setup.llvm_ir.is_some();
    let rust_mono_item_wasm_object_emitted = object_emission.wasm_object_bytes_emitted
        && object_emission.object_contains_codegened_function
        && object_emission.codegened_mono_item_count > 0;
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
    let linker_handoff = if rust_mono_item_wasm_object_emitted {
        serde_json::json!({
            "compile_unit_id": input.compile_unit_id,
            "target_triple": input.target_triple,
            "codegen_artifact_kind": "wasm_object",
            "codegen_artifact_hash": object_emission.artifact_sha256,
            "codegen_artifact_size": object_emission.artifact_size_bytes,
            "codegen_backend_identity": "rustc_codegen_llvm::LlvmCodegenBackend",
            "required_linker_component": "wasm-ld",
            "expected_final_artifact_kind": "wasm32-wasip1 module",
            "linker_input_count": 1,
            "required_runtime_objects": ["crt1-command.o"],
            "required_std_objects_or_archives": ["libcore.rlib", "liballoc.rlib", "libstd.rlib", "libwasip1.rlib", "libc.a"],
            "current_status": "wasm_ld_payload_required",
            "blocker_kind": "lld_not_embedded",
            "next_command": "embed a rouwdi-owned wasm-ld/lld payload and invoke it with the emitted wasm object plus the wasm32-wasip1 target-pack runtime objects",
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
        "crate_identity": input.crate_identity,
        "target_triple": input.target_triple,
        "target_spec": input.target_spec,
        "mir_body_hash": input.mir_body_hash,
        "mono_item_count": input.mono_item_count,
        "mono_item_graph_hash": input.mono_item_graph_hash,
        "mono_items": input.mono_items,
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
        "linker_required": true,
        "linker_handoff_created": rust_mono_item_wasm_object_emitted,
        "linker_handoff": linker_handoff,
        "rust_mono_item_wasm_object_emitted": rust_mono_item_wasm_object_emitted,
        "codegened_mono_item_count": object_emission.codegened_mono_item_count,
        "codegened_symbols": object_emission.codegened_symbols.clone(),
        "object_contains_codegened_function": object_emission.object_contains_codegened_function,
        "object_derived_from": object_emission.object_derived_from.clone(),
        "object_codegen_source": object_emission.object_codegen_source.clone(),
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
            "--crate-identity" => input.crate_identity = next_value(&mut args, &arg)?,
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
            unknown => return Err(format!("unknown argument: {unknown}")),
        }
    }
    if input.mono_items.is_empty() {
        input.mono_items.push("fn:rouwdi_payload::main".to_owned());
    }
    Ok(input)
}

fn next_value(args: &mut impl Iterator<Item = String>, option: &str) -> Result<String, String> {
    args.next()
        .ok_or_else(|| format!("{option} requires a value"))
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
    let module_name = match CString::new(input.compile_unit_id.as_str()) {
        Ok(value) => value,
        Err(error) => {
            return object_blocked(
                api,
                target_triple,
                "object_emission_attempted_blocked_at_invalid_module_name",
                error.to_string(),
            );
        }
    };
    let triple = match CString::new(input.target_triple.as_str()) {
        Ok(value) => value,
        Err(error) => {
            return object_blocked(
                api,
                target_triple,
                "object_emission_attempted_blocked_at_invalid_target_triple",
                error.to_string(),
            );
        }
    };
    let cpu = CString::new("generic").expect("static CPU has no nul");
    let features = CString::new("").expect("static features have no nul");

    unsafe {
        LLVMInitializeWebAssemblyTargetInfo();
        LLVMInitializeWebAssemblyTarget();
        LLVMInitializeWebAssemblyTargetMC();
        LLVMInitializeWebAssemblyAsmPrinter();

        let context = LLVMContextCreate();
        if context.is_null() {
            return object_blocked(
                api,
                target_triple,
                "object_emission_attempted_blocked_at_LLVMContextCreate",
                "LLVMContextCreate returned null".to_owned(),
            );
        }

        let module = LLVMModuleCreateWithNameInContext(module_name.as_ptr(), context);
        if module.is_null() {
            LLVMContextDispose(context);
            return object_blocked(
                api,
                target_triple,
                "object_emission_attempted_blocked_at_LLVMModuleCreateWithNameInContext",
                "LLVMModuleCreateWithNameInContext returned null".to_owned(),
            );
        }
        LLVMSetTarget(module, triple.as_ptr());

        let mut target = std::ptr::null_mut();
        let mut error_message = std::ptr::null_mut();
        let lookup_failed =
            LLVMGetTargetFromTriple(triple.as_ptr(), &mut target, &mut error_message);
        if lookup_failed != 0 || target.is_null() {
            let reason = llvm_error_message(
                error_message,
                "LLVMGetTargetFromTriple returned no target".to_owned(),
            );
            LLVMDisposeModule(module);
            LLVMContextDispose(context);
            return object_blocked(
                api,
                target_triple,
                "object_emission_attempted_blocked_at_LLVMGetTargetFromTriple",
                reason,
            );
        }

        let target_machine = LLVMCreateTargetMachine(
            target,
            triple.as_ptr(),
            cpu.as_ptr(),
            features.as_ptr(),
            0,
            2,
            0,
        );
        if target_machine.is_null() {
            LLVMDisposeModule(module);
            LLVMContextDispose(context);
            return object_blocked(
                api,
                target_triple,
                "object_emission_attempted_blocked_at_LLVMCreateTargetMachine",
                "LLVMCreateTargetMachine returned null".to_owned(),
            );
        }

        let mut out_buffer = std::ptr::null_mut();
        let mut emit_error = std::ptr::null_mut();
        let failed = LLVMTargetMachineEmitToMemoryBuffer(
            target_machine,
            module,
            LLVM_OBJECT_FILE,
            &mut emit_error,
            &mut out_buffer,
        );
        if failed != 0 || out_buffer.is_null() {
            let reason = llvm_error_message(
                emit_error,
                "LLVMTargetMachineEmitToMemoryBuffer returned no object buffer".to_owned(),
            );
            LLVMDisposeTargetMachine(target_machine);
            LLVMDisposeModule(module);
            LLVMContextDispose(context);
            return object_blocked(
                api,
                target_triple,
                "object_emission_attempted_blocked_at_LLVMTargetMachineEmitToMemoryBuffer",
                reason,
            );
        }

        let ptr = LLVMGetBufferStart(out_buffer);
        let len = LLVMGetBufferSize(out_buffer);
        if ptr.is_null() || len == 0 {
            LLVMDisposeMemoryBuffer(out_buffer);
            LLVMDisposeTargetMachine(target_machine);
            LLVMDisposeModule(module);
            LLVMContextDispose(context);
            return object_blocked(
                api,
                target_triple,
                "object_emission_attempted_blocked_at_empty_LLVM_memory_buffer",
                "LLVMTargetMachineEmitToMemoryBuffer returned an empty buffer".to_owned(),
            );
        }

        let object_bytes = std::slice::from_raw_parts(ptr.cast::<u8>(), len).to_vec();
        LLVMDisposeMemoryBuffer(out_buffer);
        LLVMDisposeTargetMachine(target_machine);
        LLVMDisposeModule(module);
        LLVMContextDispose(context);

        if cfg!(target_arch = "wasm32") {
            if let Err(error) = std::fs::write(OBJECT_ARTIFACT_PATH, &object_bytes) {
                return object_blocked(
                    api,
                    target_triple,
                    "object_emission_attempted_blocked_at_rouwdi_virtual_fs_write",
                    format!(
                        "failed to write emitted object bytes to rouwdi-owned WASI VFS: {error}"
                    ),
                );
            }
        }

        let inspection = inspect_wasm_object(&object_bytes);
        let object_contains_codegened_function = false;
        let codegened_mono_item_count = 0;
        let codegened_symbols = Vec::new();

        ObjectEmissionSetup {
            attempted: true,
            api,
            object_bytes_emitted: true,
            wasm_object_bytes_emitted: input.target_triple.starts_with("wasm32"),
            inspection: Some(inspection),
            object_contains_codegened_function,
            codegened_mono_item_count,
            codegened_symbols,
            object_derived_from:
                "rustc_codegen_llvm::LlvmCodegenBackend::new + LLVMTargetMachineEmitToMemoryBuffer"
                    .to_owned(),
            object_codegen_source: if object_contains_codegened_function {
                "mono_item_graph".to_owned()
            } else {
                "empty_llvm_module_before_mono_item_lowering".to_owned()
            },
            artifact_kind: Some(if input.target_triple.starts_with("wasm32") {
                "wasm_object".to_owned()
            } else {
                "native_object".to_owned()
            }),
            artifact_sha256: Some(sha256_hex(&object_bytes)),
            artifact_size_bytes: Some(object_bytes.len()),
            artifact_location: Some(if cfg!(target_arch = "wasm32") {
                format!("vfs:/workspace/{OBJECT_ARTIFACT_PATH}")
            } else {
                "memory://LLVMTargetMachineEmitToMemoryBuffer".to_owned()
            }),
            target_triple,
            retrieval_method: Some(if cfg!(target_arch = "wasm32") {
                "rouwdi_owned_virtual_fs".to_owned()
            } else {
                "guest_memory".to_owned()
            }),
            blocker_kind: None,
            blocker_reason: None,
        }
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
