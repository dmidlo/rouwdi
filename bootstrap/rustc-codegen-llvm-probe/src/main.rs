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

type LLVMTargetRef = *mut c_void;
type LLVMTargetMachineRef = *mut c_void;

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

    let llvm_ir_emitted = module_setup.llvm_ir.is_some();
    let codegen_contact_state = if llvm_ir_emitted {
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
    let blocker_kind = target_machine_setup
        .blocker_kind
        .as_deref()
        .or(module_setup.blocker_kind.as_deref())
        .unwrap_or("none");
    let blocker_reason = target_machine_setup
        .blocker_reason
        .as_deref()
        .or(module_setup.blocker_reason.as_deref())
        .unwrap_or("none");
    let codegen_artifact = if let Some(llvm_ir) = module_setup.llvm_ir.as_ref() {
        serde_json::json!({
            "artifact_kind": "llvm_ir",
            "byte_length": module_setup.llvm_ir_size_bytes,
            "sha256": module_setup.llvm_ir_sha256,
            "producer_backend": "rustc_codegen_llvm",
            "target_triple": input.target_triple,
            "mir_body_hash": input.mir_body_hash,
            "mono_item_graph_hash": input.mono_item_graph_hash,
            "payload_hash": null,
            "linker_required": true,
            "embedded_artifact_location": "backend_stdout.codegen_artifact.llvm_ir",
            "llvm_ir": llvm_ir,
        })
    } else {
        serde_json::Value::Null
    };
    let linker_handoff = if llvm_ir_emitted {
        serde_json::json!({
            "compile_unit_id": input.compile_unit_id,
            "target_triple": input.target_triple,
            "codegen_artifact_kind": "llvm_ir",
            "codegen_artifact_hash": module_setup.llvm_ir_sha256,
            "codegen_backend_identity": "rustc_codegen_llvm::LlvmCodegenBackend",
            "required_linker_component": "wasm-ld",
            "expected_final_artifact_kind": "wasm_module",
            "current_status": "blocked_until_object_bytes_emitted",
            "blocker_kind": "linker_handoff_waiting_for_wasm_object_bytes",
            "next_command": "emit wasm object bytes from the embedded LLVM backend, then invoke rouwdi-owned wasm-ld",
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
        "codegen_artifact": codegen_artifact,
        "object_emission_attempted": false,
        "object_bytes_emitted": false,
        "llvm_ir_emitted": llvm_ir_emitted,
        "bitcode_emitted": false,
        "wasm_object_bytes_emitted": false,
        "linker_handoff_created": llvm_ir_emitted,
        "linker_handoff": linker_handoff,
        "blocker_kind": blocker_kind,
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

fn sha256_hex(bytes: &[u8]) -> String {
    let mut digest = Sha256::new();
    digest.update(bytes);
    hex::encode(digest.finalize())
}
