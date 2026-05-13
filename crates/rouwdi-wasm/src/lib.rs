use rouwdi_engine::{BuildRequest, RouwdiEngine};
use rouwdi_proof::RunStatus;
use rouwdi_vfs::{HostStorage, Storage};
use std::path::PathBuf;

#[no_mangle]
pub extern "C" fn rouwdi_abi_version() -> u32 {
    1
}

#[no_mangle]
pub extern "C" fn rouwdi_host_toolchain_dependency_count() -> u32 {
    0
}

pub fn build_with_storage(storage: &mut dyn Storage, contract_path: &str) -> i32 {
    match RouwdiEngine::default().build(
        storage,
        BuildRequest {
            contract_path: contract_path.to_owned(),
        },
    ) {
        Ok(report) => match report.status {
            RunStatus::Succeeded => 0,
            RunStatus::Unsupported => 2,
            RunStatus::Failed => 1,
        },
        Err(_) => 1,
    }
}

pub fn cli_main() -> i32 {
    let mut args = std::env::args().skip(1);
    let command = args.next().unwrap_or_else(|| "build".to_owned());
    match command.as_str() {
        "build" => {
            let contract_path = args.next().unwrap_or_else(|| "rouwdi.toml".to_owned());
            let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
            let mut storage = HostStorage::new(cwd);
            match RouwdiEngine::default().build(&mut storage, BuildRequest { contract_path }) {
                Ok(report) => {
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&report).unwrap_or_default()
                    );
                    match report.status {
                        RunStatus::Succeeded => 0,
                        RunStatus::Unsupported => 2,
                        RunStatus::Failed => 1,
                    }
                }
                Err(err) => {
                    eprintln!("{err}");
                    1
                }
            }
        }
        "verify" => {
            let run_root = args
                .next()
                .unwrap_or_else(|| ".rouwdi/runs/latest".to_owned());
            let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
            let storage = HostStorage::new(cwd);
            match RouwdiEngine::default().verify(&storage, &run_root) {
                Ok(report) => {
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&report).unwrap_or_default()
                    );
                    0
                }
                Err(err) => {
                    eprintln!("{err}");
                    1
                }
            }
        }
        "abi-version" => {
            println!("{}", rouwdi_abi_version());
            0
        }
        _ => {
            eprintln!("unknown rouwdi command: {command}");
            64
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rouwdi_vfs::MemoryStorage;

    #[test]
    fn exports_no_host_toolchain_dependency_claim() {
        assert_eq!(rouwdi_abi_version(), 1);
        assert_eq!(rouwdi_host_toolchain_dependency_count(), 0);
    }

    #[test]
    fn storage_build_returns_unsupported_until_full_compiler_is_embedded() {
        let mut storage = MemoryStorage::new();
        storage
            .write(
                "rouwdi.toml",
                br#"
contract_version = 1
[project]
manifest_path = "Cargo.toml"
package = "app"
bin = "app"
[source]
mode = "snapshot"
[[targets]]
name = "wasi"
triple = "wasm32-wasip1"
artifact = "module"
"#,
            )
            .unwrap();
        storage
            .write(
                "Cargo.toml",
                br#"
[package]
name = "app"
version = "0.1.0"
edition = "2021"
"#,
            )
            .unwrap();
        storage.write("src/main.rs", b"fn main() {}\n").unwrap();

        assert_eq!(build_with_storage(&mut storage, "rouwdi.toml"), 2);
    }
}
