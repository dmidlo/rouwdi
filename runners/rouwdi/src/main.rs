use std::path::PathBuf;

fn main() {
    let wasm_path = std::env::var_os("ROUWDI_WASM")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("dist").join("rouwdi.wasm"));
    println!(
        "rouwdi native runner substrate placeholder: load {} with a WASI runtime embedding",
        wasm_path.display()
    );
    println!("this runner does not provide Cargo, rustc, linker, target policy, validation, or proof logic");
}
