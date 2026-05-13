use std::path::{Path, PathBuf};
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
    let (wasm_path, guest_args) = runner_invocation()?;
    run_wasm_module(wasm_path, guest_args).await
}

async fn run_wasm_module(wasm_path: PathBuf, guest_args: Vec<String>) -> wasmtime::Result<i32> {
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

fn runner_invocation() -> wasmtime::Result<(PathBuf, Vec<String>)> {
    let mut args = std::env::args().skip(1).collect::<Vec<_>>();
    if args.first().is_some_and(|arg| arg == "run-wasm") {
        if args.len() < 2 {
            return Err(wasmtime::Error::msg("run-wasm requires a wasm path"));
        }
        let wasm_path = PathBuf::from(args.remove(1));
        args.remove(0);
        let guest_args = direct_wasm_guest_args(&wasm_path, args);
        Ok((wasm_path, guest_args))
    } else {
        Ok((resolve_wasm_path(), guest_args()))
    }
}

fn direct_wasm_guest_args(wasm_path: &Path, args: Vec<String>) -> Vec<String> {
    let mut guest_args = vec![wasm_path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("module.wasm")
        .to_owned()];
    guest_args.extend(args);
    guest_args
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
    fn direct_wasm_runner_passes_module_name_as_argv0() {
        let wasm_path = std::path::PathBuf::from("target/wasm32-wasip1/debug/test.wasm");
        let guest_args = super::direct_wasm_guest_args(&wasm_path, vec!["--nocapture".to_owned()]);

        assert_eq!(guest_args, vec!["test.wasm", "--nocapture"]);
    }

    #[test]
    fn extracts_wasi_exit_codes_from_error_chain() {
        let err: wasmtime::Error = wasmtime_wasi::I32Exit(2).into();

        assert_eq!(super::wasi_exit_code(&err), Some(2));
    }
}
