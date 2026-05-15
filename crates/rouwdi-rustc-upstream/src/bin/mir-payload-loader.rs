use rouwdi_rustc_upstream::{execute_compiler_payload_wasm, mir_compiler_payload_bundle};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

fn main() -> ExitCode {
    let args = Args::parse(env::args().skip(1));
    let workspace_root = workspace_root();
    let bundle = mir_compiler_payload_bundle();
    let artifact_path = args
        .artifact_path
        .unwrap_or_else(|| PathBuf::from(&bundle.exported_rlib_identity.path));
    let artifact_path = absolutize(&workspace_root, &artifact_path);
    let expected_sha256 = args
        .expected_sha256
        .unwrap_or_else(|| bundle.exported_rlib_identity.sha256.clone());
    let bytes = match fs::read(&artifact_path) {
        Ok(bytes) => bytes,
        Err(error) => {
            eprintln!("failed to read {}: {error}", artifact_path.display());
            return ExitCode::from(1);
        }
    };

    let relative = relative_path(&workspace_root, &artifact_path);
    match execute_compiler_payload_wasm(&relative, &bytes, &expected_sha256) {
        Ok(report) => {
            if args.json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&report).expect("loader report serializes")
                );
            } else {
                println!(
                    "mir payload loader: hash={} instantiated={} abi={} execute={} classification={}",
                    report.hash_verified,
                    report.module_instantiated,
                    report.abi_v1_exports_verified,
                    report.execute_called,
                    report.classification
                );
            }

            if report.hash_verified
                && report.module_instantiated
                && report.abi_v1_exports_verified
                && report.execute_called
                && report.generic_upstream_context_unavailable_replaced
            {
                ExitCode::SUCCESS
            } else {
                ExitCode::from(1)
            }
        }
        Err(error) => {
            eprintln!("{error}");
            ExitCode::from(1)
        }
    }
}

#[derive(Debug, Default)]
struct Args {
    json: bool,
    artifact_path: Option<PathBuf>,
    expected_sha256: Option<String>,
}

impl Args {
    fn parse(raw_args: impl IntoIterator<Item = String>) -> Self {
        let mut args = Self::default();
        let mut iter = raw_args.into_iter();
        while let Some(arg) = iter.next() {
            match arg.as_str() {
                "--json" => args.json = true,
                "--artifact" => {
                    args.artifact_path = iter.next().map(PathBuf::from);
                }
                "--expected-sha256" => {
                    args.expected_sha256 = iter.next();
                }
                value if value.starts_with("--artifact=") => {
                    args.artifact_path =
                        Some(PathBuf::from(value.trim_start_matches("--artifact=")));
                }
                value if value.starts_with("--expected-sha256=") => {
                    args.expected_sha256 =
                        Some(value.trim_start_matches("--expected-sha256=").to_owned());
                }
                _ => {}
            }
        }
        args
    }
}

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("adapter crate lives under workspace/crates/rouwdi-rustc-upstream")
        .to_path_buf()
}

fn absolutize(workspace_root: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        workspace_root.join(path)
    }
}

fn relative_path(workspace_root: &Path, path: &Path) -> String {
    path.strip_prefix(workspace_root)
        .unwrap_or(path)
        .display()
        .to_string()
}
