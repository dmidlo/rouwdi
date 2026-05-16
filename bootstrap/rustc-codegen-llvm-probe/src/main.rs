use std::process::ExitCode;

fn main() -> ExitCode {
    let backend = rustc_codegen_llvm::LlvmCodegenBackend::new();
    let payload = serde_json::json!({
        "probe_name": "rustc_codegen_llvm_backend_constructor",
        "upstream_component": "rustc_codegen_llvm",
        "upstream_path": "third_party/rust/compiler/rustc_codegen_llvm",
        "backend_family": "llvm-grade",
        "entrypoint": "rustc_codegen_llvm::LlvmCodegenBackend::new",
        "backend_constructed": true,
        "backend_name": backend.name(),
        "object_bytes_emitted": false,
        "llvm_ir_emitted": false
    });

    match serde_json::to_string_pretty(&payload) {
        Ok(serialized) => {
            println!("{serialized}");
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("failed to serialize rustc_codegen_llvm probe: {error}");
            ExitCode::from(1)
        }
    }
}
