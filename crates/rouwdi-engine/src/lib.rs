use rouwdi_cargo::{
    parse_lockfile, plan_build, plan_source_fetches, resolve_features, resolve_workspace,
    validate_lockfile_against_fetch_plan, CargoModelError, CargoTargetKind, CompilePhase,
};
use rouwdi_compiletime::plan_compile_time;
use rouwdi_contract::{ContractError, RouwdiContract};
use rouwdi_proof::{
    hash_bytes, verify_manifest_hashes, verify_manifest_references, ArtifactInterfaceProof,
    ArtifactManifestEntry, HashEntry, ProofBundle, ProofError, ProofStatus, RouwdiRunManifest,
    RunStatus, RuntimeProof, UnsupportedCapability,
};
use rouwdi_source::{snapshot_source, source_relative_path, SourceError};
use rouwdi_targets::{TargetError, TargetPackRegistry};
use rouwdi_vfs::{Storage, VfsError};
use serde::{Deserialize, Serialize};

#[derive(Debug, thiserror::Error)]
pub enum EngineError {
    #[error(transparent)]
    Vfs(#[from] VfsError),
    #[error(transparent)]
    Contract(#[from] ContractError),
    #[error(transparent)]
    Source(#[from] SourceError),
    #[error(transparent)]
    Cargo(#[from] CargoModelError),
    #[error(transparent)]
    Targets(#[from] TargetError),
    #[error(transparent)]
    Proof(#[from] ProofError),
    #[error("engine JSON failure: {0}")]
    Json(#[from] serde_json::Error),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BuildRequest {
    pub contract_path: String,
}

impl Default for BuildRequest {
    fn default() -> Self {
        Self {
            contract_path: "rouwdi.toml".to_owned(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BuildReport {
    pub run_id: String,
    pub status: RunStatus,
    pub run_root: String,
    pub manifest_path: String,
    pub proof_files: Vec<String>,
    pub unsupported: Vec<UnsupportedCapability>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VerifyReport {
    pub run_root: String,
    pub status: RunStatus,
    pub checked_hashes: usize,
}

#[derive(Debug, Clone)]
pub struct RouwdiEngine {
    target_registry: TargetPackRegistry,
}

impl RouwdiEngine {
    pub fn new(target_registry: TargetPackRegistry) -> Self {
        Self { target_registry }
    }

    pub fn build(
        &self,
        storage: &mut dyn Storage,
        request: BuildRequest,
    ) -> Result<BuildReport, EngineError> {
        let contract_bytes = storage.read(&request.contract_path)?;
        let contract_text = String::from_utf8_lossy(&contract_bytes);
        let contract = RouwdiContract::parse(&contract_text)?;
        let normalized = contract.normalize()?;
        let source_snapshot = snapshot_source(storage, &contract.source.root)?;
        let manifest_path =
            source_relative_path(&contract.source.root, &contract.project.manifest_path)?;
        let cargo_workspace = resolve_workspace(storage, &manifest_path)?;
        let source_fetch_plan = plan_source_fetches(&cargo_workspace);
        let (selected_target, selected_target_kind) =
            match (&contract.project.bin, &contract.project.example) {
                (Some(bin), None) => (bin.clone(), CargoTargetKind::Bin),
                (None, Some(example)) => (example.clone(), CargoTargetKind::Example),
                _ => unreachable!("contract validation requires exactly one primary target"),
            };
        let target_triples = contract
            .targets
            .iter()
            .map(|target| target.triple.clone())
            .collect::<Vec<_>>();
        let cargo_features = resolve_features(
            &cargo_workspace,
            &contract.project.package,
            contract.project.default_features,
            &contract.project.features,
        )?;
        let build_plan = plan_build(
            &cargo_workspace,
            &cargo_features,
            &contract.project.package,
            &selected_target,
            selected_target_kind,
            &contract.project.profile,
            &target_triples,
        )?;
        let compile_time_plan = plan_compile_time(&build_plan);
        let cargo_lockfile = match parse_lockfile(storage, &contract.resolver.lockfile) {
            Ok(lockfile) => Some(lockfile),
            Err(CargoModelError::Vfs(VfsError::NotFound(_))) if !contract.resolver.frozen => None,
            Err(CargoModelError::Vfs(VfsError::NotFound(_))) => {
                return Err(CargoModelError::MissingFrozenLockfile(
                    contract.resolver.lockfile.clone(),
                )
                .into());
            }
            Err(err) => return Err(err.into()),
        };
        if let Some(lockfile) = &cargo_lockfile {
            validate_lockfile_against_fetch_plan(lockfile, &source_fetch_plan)?;
        }
        let target_packs = self.target_registry.validate_contract(&contract)?;

        let mut unsupported = Vec::new();
        for pack in &target_packs {
            if !pack.target_pack_embedded {
                unsupported.push(UnsupportedCapability {
                    capability: format!("{} target pack", pack.triple),
                    required_by: format!("artifact emission for {}", pack.triple),
                    reason: "target ABI/object/link metadata is not embedded in this assembly"
                        .to_owned(),
                });
            }
            if !pack.std_pack_embedded {
                unsupported.push(UnsupportedCapability {
                    capability: format!("{} std/core/alloc pack", pack.triple),
                    required_by: format!("Rust standard library resolution for {}", pack.triple),
                    reason: "std/core/alloc artifacts are not embedded in this assembly".to_owned(),
                });
            }
            if !pack.linker_pack_embedded {
                unsupported.push(UnsupportedCapability {
                    capability: format!("{} linker pack", pack.triple),
                    required_by: format!("final link for {}", pack.triple),
                    reason: "linker scripts/configuration/runtime objects are not embedded in this assembly"
                        .to_owned(),
                });
            }
        }
        if !self.target_registry.compiler.compiler_semantics_embedded {
            unsupported.push(UnsupportedCapability {
                capability: "rustc frontend semantics".to_owned(),
                required_by: "compile Rust crate graph".to_owned(),
                reason: "no Rust parser, expansion, typeck, borrowck, MIR, or metadata engine is embedded in this assembly".to_owned(),
            });
        }
        if !self.target_registry.compiler.codegen_embedded {
            unsupported.push(UnsupportedCapability {
                capability: "codegen".to_owned(),
                required_by: "object/module emission".to_owned(),
                reason: "no LLVM-grade codegen backend is embedded in this assembly".to_owned(),
            });
        }
        if !self.target_registry.compiler.linker_embedded {
            unsupported.push(UnsupportedCapability {
                capability: "linker".to_owned(),
                required_by: "final native/WASI artifact emission".to_owned(),
                reason: "no native or WASM linker is embedded in this assembly".to_owned(),
            });
        }
        if build_plan
            .units
            .iter()
            .any(|unit| unit.phase == CompilePhase::BuildScript)
        {
            unsupported.push(UnsupportedCapability {
                capability: "build.rs compile-time sandbox execution".to_owned(),
                required_by: "Cargo build script directives and generated files".to_owned(),
                reason: "build scripts are planned for compile-time WASM but execution is not embedded yet".to_owned(),
            });
        }
        if build_plan
            .units
            .iter()
            .any(|unit| unit.phase == CompilePhase::ProcMacro)
        {
            unsupported.push(UnsupportedCapability {
                capability: "proc-macro compile-time sandbox execution".to_owned(),
                required_by: "Rust macro expansion".to_owned(),
                reason: "proc-macro crates are planned for compile-time WASM but token-stream execution is not embedded yet".to_owned(),
            });
        }
        for target in &contract.targets {
            if target.runtime.required && target.triple == "native_host" {
                unsupported.push(UnsupportedCapability {
                    capability: "native runtime execution".to_owned(),
                    required_by: format!("runtime proof for target {}", target.name),
                    reason: "native execution is a host runtime capability and must be recorded as delegated or unsupported by the current host".to_owned(),
                });
            }
        }

        let status = if unsupported.is_empty() {
            RunStatus::Succeeded
        } else {
            RunStatus::Unsupported
        };
        let interface_proofs = contract
            .targets
            .iter()
            .map(|target| ArtifactInterfaceProof {
                target_name: target.name.clone(),
                triple: target.triple.clone(),
                artifact_kind: format!("{:?}", target.artifact),
                artifact_path: None,
                artifact_built: false,
                required_exports: target.interface.required_exports.clone(),
                missing_exports: target.interface.required_exports.clone(),
                require_executable: target.interface.require_executable,
                executable_detected: None,
                status: ProofStatus::Unsupported,
                reason: Some(
                    "artifact was not built because compiler/codegen/linker are not embedded"
                        .to_owned(),
                ),
            })
            .collect::<Vec<_>>();
        let runtime_proofs = contract
            .targets
            .iter()
            .map(|target| RuntimeProof {
                target_name: target.name.clone(),
                triple: target.triple.clone(),
                required: target.runtime.required,
                kind: target.runtime.kind.map(|kind| format!("{kind:?}")),
                mode: format!("{:?}", target.runtime.mode),
                executed: false,
                expected_exit_code: target.runtime.expected_exit_code,
                actual_exit_code: None,
                timed_out: None,
                stdout_contains: target.runtime.stdout_contains.clone(),
                stdout_matched: None,
                status: if target.runtime.required {
                    ProofStatus::Unsupported
                } else {
                    ProofStatus::Succeeded
                },
                reason: if target.runtime.required {
                    Some("runtime proof cannot execute until the artifact exists".to_owned())
                } else {
                    None
                },
            })
            .collect::<Vec<_>>();
        let run_id = deterministic_run_id(&normalized.sha256, &source_snapshot.tree_sha256);
        let run_root = format!(".rouwdi/runs/{run_id}");
        let mut hashes = Vec::new();
        hashes.push(HashEntry {
            label: "contract".to_owned(),
            path: request.contract_path.clone(),
            sha256: hash_bytes(&contract_bytes),
        });
        for file in &source_snapshot.files {
            hashes.push(HashEntry {
                label: format!("source:{}", file.path),
                path: file.path.clone(),
                sha256: file.sha256.clone(),
            });
        }

        let mut manifest = RouwdiRunManifest {
            run_id: run_id.clone(),
            status,
            contract_sha256: normalized.sha256.clone(),
            source_tree_sha256: source_snapshot.tree_sha256.clone(),
            compiler_engine: self.target_registry.compiler.clone(),
            target_packs,
            artifacts: Vec::<ArtifactManifestEntry>::new(),
            unsupported: unsupported.clone(),
            proof_files: Vec::new(),
        };
        let mut bundle = ProofBundle {
            manifest: manifest.clone(),
            normalized_contract: normalized,
            source_snapshot,
            cargo_workspace,
            cargo_features,
            source_fetch_plan,
            build_plan,
            compile_time_plan,
            cargo_lockfile,
            interface_proofs,
            runtime_proofs,
            hashes,
        };
        let proof_files = bundle.write_to_storage(storage, &run_root)?;
        manifest.proof_files = proof_files.clone();
        bundle.manifest = manifest.clone();
        storage.write(
            &format!("{run_root}/manifest.json"),
            &serde_json::to_vec_pretty(&manifest)?,
        )?;

        Ok(BuildReport {
            run_id,
            status,
            manifest_path: format!("{run_root}/manifest.json"),
            run_root,
            proof_files,
            unsupported,
        })
    }

    pub fn verify(
        &self,
        storage: &dyn Storage,
        run_root: &str,
    ) -> Result<VerifyReport, EngineError> {
        let manifest_path = format!("{run_root}/manifest.json");
        let manifest: RouwdiRunManifest = serde_json::from_slice(&storage.read(&manifest_path)?)?;
        verify_manifest_references(storage, &manifest)?;
        let hashes_path = format!("{run_root}/proofs/hashes.json");
        let hashes: Vec<HashEntry> = serde_json::from_slice(&storage.read(&hashes_path)?)?;
        verify_manifest_hashes(storage, &hashes)?;
        Ok(VerifyReport {
            run_root: run_root.to_owned(),
            status: manifest.status,
            checked_hashes: hashes.len(),
        })
    }
}

impl Default for RouwdiEngine {
    fn default() -> Self {
        Self::new(TargetPackRegistry::strict_embedded())
    }
}

pub fn deterministic_run_id(contract_sha256: &str, source_tree_sha256: &str) -> String {
    let seed = format!("{contract_sha256}:{source_tree_sha256}");
    let digest = hash_bytes(seed.as_bytes());
    format!("run-{}", &digest[..16])
}

#[cfg(test)]
mod tests {
    use super::*;
    use rouwdi_vfs::MemoryStorage;

    fn fixture_storage() -> MemoryStorage {
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
profile = "release"

[source]
mode = "snapshot"
root = "."

[[targets]]
name = "wasi"
triple = "wasm32-wasip1"
artifact = "module"

[targets.interface]
required_exports = ["_start"]

[targets.runtime]
required = true
kind = "wasi"
expected_exit_code = 0
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
        storage
            .write(
                "Cargo.lock",
                br#"
version = 4

[[package]]
name = "app"
version = "0.1.0"
"#,
            )
            .unwrap();
        storage
    }

    #[test]
    fn build_writes_proof_bundle_and_refuses_fake_success_without_embedded_compiler() {
        let mut storage = fixture_storage();

        let report = RouwdiEngine::default()
            .build(&mut storage, BuildRequest::default())
            .unwrap();

        assert_eq!(report.status, RunStatus::Unsupported);
        assert!(report
            .unsupported
            .iter()
            .any(|item| item.capability == "rustc frontend semantics"));
        assert!(storage
            .read(&report.manifest_path)
            .unwrap()
            .starts_with(b"{"));
        assert!(storage
            .read(&format!("{}/graph/cargo-resolve.json", report.run_root))
            .unwrap()
            .starts_with(b"{"));
        assert!(storage
            .read(&format!("{}/graph/features.json", report.run_root))
            .unwrap()
            .starts_with(b"{"));
        assert!(storage
            .read(&format!("{}/graph/cargo-lock.json", report.run_root))
            .unwrap()
            .starts_with(b"{"));
        assert!(storage
            .read(&format!("{}/graph/source-fetch-plan.json", report.run_root))
            .unwrap()
            .starts_with(b"{"));
        assert!(storage
            .read(&format!("{}/graph/compiletime-plan.json", report.run_root))
            .unwrap()
            .starts_with(b"{"));
        assert!(storage
            .read(&format!("{}/proofs/interface-wasi.json", report.run_root))
            .unwrap()
            .starts_with(b"{"));
        assert!(storage
            .read(&format!("{}/proofs/runtime-wasi.json", report.run_root))
            .unwrap()
            .starts_with(b"{"));
    }

    #[test]
    fn verify_checks_hashes_in_written_proof_bundle() {
        let mut storage = fixture_storage();
        let engine = RouwdiEngine::default();
        let report = engine.build(&mut storage, BuildRequest::default()).unwrap();

        let verify = engine.verify(&storage, &report.run_root).unwrap();

        assert_eq!(verify.status, RunStatus::Unsupported);
        assert!(verify.checked_hashes >= 3);
    }

    #[test]
    fn frozen_resolver_rejects_missing_lockfile() {
        let mut storage = fixture_storage();
        storage.remove("Cargo.lock").unwrap();

        let err = RouwdiEngine::default()
            .build(&mut storage, BuildRequest::default())
            .unwrap_err();

        assert!(err.to_string().contains("resolver is frozen"));
    }
}
