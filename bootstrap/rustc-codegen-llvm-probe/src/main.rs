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

#[cfg(not(target_arch = "wasm32"))]
type LLVMTargetRef = *mut c_void;
#[cfg(not(target_arch = "wasm32"))]
type LLVMTargetMachineRef = *mut c_void;

#[cfg(not(target_arch = "wasm32"))]
#[link(name = "llvm-wrapper", kind = "static")]
unsafe extern "C" {}

#[cfg(not(target_arch = "wasm32"))]
unsafe extern "C" {
    fn LLVMContextCreate() -> *mut c_void;
    fn LLVMContextDispose(context: *mut c_void);
    fn LLVMModuleCreateWithNameInContext(
        module_id: *const c_char,
        context: *mut c_void,
    ) -> *mut c_void;
    fn LLVMDisposeModule(module: *mut c_void);
    fn LLVMSetTarget(module: *mut c_void, triple: *const c_char);
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

    let codegen_contact_state = if target_machine_setup.target_machine_created {
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
        "object_emission_attempted": false,
        "object_bytes_emitted": false,
        "llvm_ir_emitted": false,
        "bitcode_emitted": false,
        "wasm_object_bytes_emitted": false,
        "linker_handoff_created": false,
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

#[cfg(not(target_arch = "wasm32"))]
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
        LLVMDisposeModule(module);
        LLVMContextDispose(context);

        ModuleSetup {
            attempted: true,
            llvm_context_created: true,
            llvm_module_created: true,
            module_identity: Some(identity),
            module_identity_hash: Some(identity_hash),
            module_target_triple: Some(input.target_triple.clone()),
            blocker_kind: None,
            blocker_reason: None,
        }
    }
}

#[cfg(target_arch = "wasm32")]
fn attempt_llvm_module_setup(_input: &ProbeInput) -> ModuleSetup {
    ModuleSetup {
        attempted: false,
        llvm_context_created: false,
        llvm_module_created: false,
        module_identity: None,
        module_identity_hash: None,
        module_target_triple: None,
        blocker_kind: None,
        blocker_reason: None,
    }
}

#[cfg(not(target_arch = "wasm32"))]
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

#[cfg(target_arch = "wasm32")]
fn attempt_target_machine_setup(input: &ProbeInput) -> TargetMachineSetup {
    TargetMachineSetup {
        attempted: false,
        target_triple: input.target_triple.clone(),
        cpu: "generic".to_owned(),
        features: String::new(),
        relocation_model: "pic".to_owned(),
        code_model: "default".to_owned(),
        optimization_level: "none".to_owned(),
        target_machine_created: false,
        blocker_kind: None,
        blocker_reason: None,
    }
}

fn sha256_hex(bytes: &[u8]) -> String {
    let mut digest = Sha256::new();
    digest.update(bytes);
    hex::encode(digest.finalize())
}
