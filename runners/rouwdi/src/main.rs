use std::path::PathBuf;
use wasmtime::{Engine, Linker, Module, Store};
use wasmtime_wasi::p1::{add_to_linker_async, WasiP1Ctx};
use wasmtime_wasi::{DirPerms, FilePerms, I32Exit, WasiCtxBuilder};

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    match run().await {
        Ok(code) => std::process::exit(code),
        Err(err) => {
            eprintln!("{err:#}");
            std::process::exit(1);
        }
    }
}

async fn run() -> wasmtime::Result<i32> {
    let wasm_path = resolve_wasm_path();
    let guest_args = guest_args();
    let cwd = std::env::current_dir()?;

    let engine = Engine::default();
    let mut linker = Linker::<WasiP1Ctx>::new(&engine);
    add_to_linker_async(&mut linker, |ctx| ctx)?;

    let module = Module::from_file(&engine, &wasm_path)?;
    let mut builder = WasiCtxBuilder::new();
    builder
        .inherit_stdio()
        .args(&guest_args)
        .inherit_network()
        .preopened_dir(&cwd, ".", DirPerms::all(), FilePerms::all())?;
    let mut store = Store::new(&engine, builder.build_p1());

    let instance = linker.instantiate_async(&mut store, &module).await?;
    let start = instance.get_typed_func::<(), ()>(&mut store, "_start")?;
    match start.call_async(&mut store, ()).await {
        Ok(()) => Ok(0),
        Err(err) => {
            if let Some(code) = wasi_exit_code(&err) {
                Ok(code)
            } else {
                Err(err)
            }
        }
    }
}

fn wasi_exit_code(err: &wasmtime::Error) -> Option<i32> {
    err.chain()
        .find_map(|cause| cause.downcast_ref::<I32Exit>().map(|exit| exit.0))
}

fn resolve_wasm_path() -> PathBuf {
    std::env::var_os("ROUWDI_WASM")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("dist").join("rouwdi.wasm"))
}

fn guest_args() -> Vec<String> {
    let mut args = Vec::new();
    args.push("rouwdi.wasm".to_owned());
    args.extend(std::env::args().skip(1));
    if args.len() == 1 {
        args.push("build".to_owned());
    }
    args
}

#[cfg(test)]
mod tests {
    #[test]
    fn default_guest_args_call_build_inside_the_wasm_assembly() {
        let args = super::guest_args();

        assert_eq!(args[0], "rouwdi.wasm");
    }

    #[test]
    fn extracts_wasi_exit_codes_from_error_chain() {
        let err: wasmtime::Error = wasmtime_wasi::I32Exit(2).into();

        assert_eq!(super::wasi_exit_code(&err), Some(2));
    }
}
