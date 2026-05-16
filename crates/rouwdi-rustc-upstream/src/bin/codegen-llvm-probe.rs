use std::process::ExitCode;

fn main() -> ExitCode {
    let json = std::env::args().skip(1).any(|arg| arg == "--json");
    let probe = rouwdi_rustc_upstream::rustc_codegen_llvm_backend_probe();

    if json {
        match serde_json::to_string_pretty(&probe) {
            Ok(serialized) => {
                println!("{serialized}");
                ExitCode::SUCCESS
            }
            Err(error) => {
                eprintln!("failed to serialize rustc_codegen_llvm probe: {error}");
                ExitCode::from(1)
            }
        }
    } else {
        println!(
            "rustc_codegen_llvm probe: entrypoint={} constructed={} target_status={}",
            probe.entrypoint, probe.backend_constructed, probe.target_loadable_status
        );
        ExitCode::SUCCESS
    }
}
