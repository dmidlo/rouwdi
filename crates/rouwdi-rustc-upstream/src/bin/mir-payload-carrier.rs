use rouwdi_rustc_upstream::{mir_handoff_payload_carrier, MirHandoffPayloadCarrier};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::env;
use std::fs;
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

#[derive(Debug, Clone, Serialize)]
struct CarrierReport {
    command: String,
    workspace_root: String,
    carrier: Option<MirHandoffPayloadCarrier>,
    local_artifact: Option<LocalArtifactIdentity>,
    local_metadata_artifact: Option<LocalArtifactIdentity>,
    artifact_identity_matches_ledger: Option<bool>,
    metadata_identity_matches_ledger: Option<bool>,
    loadable_by_rouwdi_wasm: bool,
    blocker_kind: Option<String>,
    blocker_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
struct LocalArtifactIdentity {
    path: String,
    sha256: String,
    size_bytes: u64,
}

#[derive(Debug, Clone)]
struct CarrierArgs {
    json: bool,
    require_located: bool,
}

fn main() -> ExitCode {
    let args = match CarrierArgs::parse(env::args().skip(1)) {
        Ok(args) => args,
        Err(message) => {
            eprintln!("{message}");
            eprintln!("{}", usage());
            return ExitCode::from(2);
        }
    };

    let workspace_root = workspace_root();
    let carrier = mir_handoff_payload_carrier();
    let local_artifact = carrier
        .as_ref()
        .and_then(|carrier| carrier.artifact.as_ref())
        .and_then(|artifact| locate_artifact(&workspace_root, &artifact.path).ok());
    let local_metadata_artifact = carrier
        .as_ref()
        .and_then(|carrier| carrier.metadata_artifact.as_ref())
        .and_then(|artifact| locate_artifact(&workspace_root, &artifact.path).ok());
    let artifact_identity_matches_ledger = carrier
        .as_ref()
        .and_then(|carrier| carrier.artifact.as_ref())
        .map(|artifact| {
            local_artifact.as_ref().is_some_and(|local| {
                local.sha256.eq_ignore_ascii_case(&artifact.sha256)
                    && local.size_bytes == artifact.size_bytes
            })
        });
    let metadata_identity_matches_ledger = carrier
        .as_ref()
        .and_then(|carrier| carrier.metadata_artifact.as_ref())
        .map(|artifact| {
            local_metadata_artifact.as_ref().is_some_and(|local| {
                local.sha256.eq_ignore_ascii_case(&artifact.sha256)
                    && local.size_bytes == artifact.size_bytes
            })
        });
    let loadable_by_rouwdi_wasm = carrier
        .as_ref()
        .is_some_and(|carrier| carrier.loaded_into_rouwdi_facade);
    let blocker_kind = carrier
        .as_ref()
        .and_then(|carrier| carrier.load_blocker_kind.clone());
    let blocker_reason = carrier
        .as_ref()
        .and_then(|carrier| carrier.load_blocker_reason.clone());

    let report = CarrierReport {
        command: "cargo run -p rouwdi-rustc-upstream --bin mir-payload-carrier -- --json"
            .to_owned(),
        workspace_root: workspace_root.display().to_string(),
        carrier,
        local_artifact,
        local_metadata_artifact,
        artifact_identity_matches_ledger,
        metadata_identity_matches_ledger,
        loadable_by_rouwdi_wasm,
        blocker_kind,
        blocker_reason,
    };

    if args.json {
        println!(
            "{}",
            serde_json::to_string_pretty(&report).expect("carrier report serializes to JSON")
        );
    } else {
        print_text_report(&report);
    }

    if args.require_located && !matches!(report.artifact_identity_matches_ledger, Some(true)) {
        return ExitCode::from(1);
    }

    ExitCode::SUCCESS
}

impl CarrierArgs {
    fn parse(raw_args: impl IntoIterator<Item = String>) -> Result<Self, String> {
        let mut json = false;
        let mut require_located = false;

        for arg in raw_args {
            match arg.as_str() {
                "--help" | "-h" => return Err("MIR payload carrier locator".to_owned()),
                "--json" => json = true,
                "--require-located" => require_located = true,
                value if value.starts_with('-') => {
                    return Err(format!("unsupported argument: {value}"));
                }
                value => return Err(format!("unexpected positional argument: {value}")),
            }
        }

        Ok(Self {
            json,
            require_located,
        })
    }
}

fn locate_artifact(
    workspace_root: &Path,
    relative_path: &str,
) -> io::Result<LocalArtifactIdentity> {
    let path = workspace_root.join(relative_path);
    let mut file = fs::File::open(&path)?;
    let mut hasher = Sha256::new();
    let mut buffer = [0_u8; 8192];

    loop {
        let read = file.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }

    let size_bytes = file.metadata()?.len();
    Ok(LocalArtifactIdentity {
        path: relative_path.to_owned(),
        sha256: hex::encode(hasher.finalize()),
        size_bytes,
    })
}

fn print_text_report(report: &CarrierReport) {
    if let Some(carrier) = &report.carrier {
        println!(
            "{} state={} artifact_located={} carrier_created={} loaded={}",
            carrier.carrier_id,
            carrier.state.as_str(),
            carrier.bootstrap_artifact_located,
            carrier.carrier_created,
            carrier.loaded_into_rouwdi_facade
        );
        if let Some(artifact) = &carrier.artifact {
            println!(
                "  artifact: {} {} {} bytes sha256={}",
                artifact.artifact_format, artifact.path, artifact.size_bytes, artifact.sha256
            );
        }
        if let Some(artifact) = &carrier.metadata_artifact {
            println!(
                "  metadata: {} {} {} bytes sha256={}",
                artifact.artifact_format, artifact.path, artifact.size_bytes, artifact.sha256
            );
        }
        if let Some(kind) = &carrier.load_blocker_kind {
            println!("  blocker: {kind}");
        }
    } else {
        println!("no MIR payload carrier is recorded in the import ledger");
    }
}

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("adapter crate lives under workspace/crates/rouwdi-rustc-upstream")
        .to_path_buf()
}

fn usage() -> &'static str {
    "usage: mir-payload-carrier [--json] [--require-located]"
}
