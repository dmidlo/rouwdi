use rouwdi_cargo::{
    parse_lockfile, plan_build, plan_source_fetches, resolve_features, resolve_workspace,
    validate_lockfile_against_fetch_plan, CargoModelError, CargoSourceKind, CargoTargetKind,
    CompilePhase,
};
use rouwdi_compiletime::plan_compile_time;
use rouwdi_contract::{ContractError, RouwdiContract};
use rouwdi_proof::{
    hash_bytes, verify_manifest_hashes, verify_manifest_references, ArtifactInterfaceProof,
    ArtifactManifestEntry, HashEntry, ProofBundle, ProofError, ProofStatus, RouwdiRunManifest,
    RunStatus, RuntimeProof, UnsupportedCapability,
};
use rouwdi_rustc::{lex_rust_source_with_diagnostics, RustSourceLexProof};
use rouwdi_source::{
    materialize_source_cache_with_options, snapshot_source, source_relative_path, SourceCacheKind,
    SourceCacheOptions, SourceCacheRequest, SourceCacheStatus, SourceError,
};
use rouwdi_targets::{TargetError, TargetPackRegistry};
use rouwdi_vfs::{join_path, normalize_path, Storage, VfsError};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

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
        let contract_root = parent_path(&request.contract_path)?;
        let source_root = join_path(&contract_root, &contract.source.root)?;
        let source_snapshot = snapshot_source(storage, &source_root)?;
        let manifest_path = source_relative_path(&source_root, &contract.project.manifest_path)?;
        let cargo_workspace = resolve_workspace(storage, &manifest_path)?;
        let source_fetch_plan = plan_source_fetches(&cargo_workspace);
        let source_cache_root = source_relative_path(&source_root, ".rouwdi/cache/sources")?;
        let source_cache_requests = source_fetch_plan
            .entries
            .iter()
            .map(|entry| SourceCacheRequest {
                package: entry.package.clone(),
                dependency: entry.dependency.clone(),
                kind: match entry.kind {
                    CargoSourceKind::Path => SourceCacheKind::Path,
                    CargoSourceKind::Git => SourceCacheKind::Git,
                    CargoSourceKind::Registry => SourceCacheKind::Registry,
                },
                locator: entry.locator.clone(),
                requirement: entry.requirement.clone(),
                target_cfg: entry.target_cfg.clone(),
            })
            .collect::<Vec<_>>();
        let source_cache = materialize_source_cache_with_options(
            storage,
            &source_cache_root,
            &source_cache_requests,
            SourceCacheOptions {
                vendor_root: contract
                    .resolver
                    .vendor
                    .as_deref()
                    .map(|vendor| source_relative_path(&source_root, vendor))
                    .transpose()?,
            },
        )?;
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
        let rust_source_lex = lex_build_plan_sources(storage, &build_plan)?;
        let lockfile_path = source_relative_path(&source_root, &contract.resolver.lockfile)?;
        let cargo_lockfile = match parse_lockfile(storage, &lockfile_path) {
            Ok(lockfile) => Some(lockfile),
            Err(CargoModelError::Vfs(VfsError::NotFound(_))) if !contract.resolver.frozen => None,
            Err(CargoModelError::Vfs(VfsError::NotFound(_))) => {
                return Err(CargoModelError::MissingFrozenLockfile(lockfile_path.clone()).into());
            }
            Err(err) => return Err(err.into()),
        };
        if let Some(lockfile) = &cargo_lockfile {
            validate_lockfile_against_fetch_plan(lockfile, &source_fetch_plan)?;
        }
        let target_packs = self.target_registry.validate_contract(&contract)?;

        let mut internal_blockers = Vec::new();
        let mut unsupported = Vec::new();
        for cache_entry in &source_cache.entries {
            if cache_entry.status == SourceCacheStatus::PlannedFetch {
                internal_blockers.push(UnsupportedCapability {
                    capability: format!("{:?} source fetcher", cache_entry.kind),
                    required_by: format!(
                        "dependency source materialization for {}",
                        cache_entry.dependency
                    ),
                    reason: cache_entry.reason.clone().unwrap_or_else(|| {
                        "remote source fetch is not embedded in this assembly".to_owned()
                    }),
                });
            }
        }
        for pack in &target_packs {
            if !pack.target_pack_embedded {
                internal_blockers.push(UnsupportedCapability {
                    capability: format!("{} target pack", pack.triple),
                    required_by: format!("artifact emission for {}", pack.triple),
                    reason: "target ABI/object/link metadata is not embedded in this assembly"
                        .to_owned(),
                });
            }
            if !pack.std_pack_embedded {
                internal_blockers.push(UnsupportedCapability {
                    capability: format!("{} std/core/alloc pack", pack.triple),
                    required_by: format!("Rust standard library resolution for {}", pack.triple),
                    reason: "std/core/alloc artifacts are not embedded in this assembly".to_owned(),
                });
            }
            if !pack.linker_pack_embedded {
                internal_blockers.push(UnsupportedCapability {
                    capability: format!("{} linker pack", pack.triple),
                    required_by: format!("final link for {}", pack.triple),
                    reason: "linker scripts/configuration/runtime objects are not embedded in this assembly"
                        .to_owned(),
                });
            }
        }
        if !self.target_registry.compiler.compiler_semantics_embedded {
            internal_blockers.push(UnsupportedCapability {
                capability: "rustc frontend semantics".to_owned(),
                required_by: "compile Rust crate graph".to_owned(),
                reason: "no Rust parser, expansion, typeck, borrowck, MIR, or metadata engine is embedded in this assembly".to_owned(),
            });
        }
        if !self.target_registry.compiler.codegen_embedded {
            internal_blockers.push(UnsupportedCapability {
                capability: "codegen".to_owned(),
                required_by: "object/module emission".to_owned(),
                reason: "no LLVM-grade codegen backend is embedded in this assembly".to_owned(),
            });
        }
        if !self.target_registry.compiler.linker_embedded {
            internal_blockers.push(UnsupportedCapability {
                capability: "linker".to_owned(),
                required_by: "final native/WASI artifact emission".to_owned(),
                reason: "no native or WASM linker is embedded in this assembly".to_owned(),
            });
        }
        let lexical_diagnostic_count = rust_source_lex
            .iter()
            .map(|proof| proof.diagnostics.len())
            .sum::<usize>();
        if lexical_diagnostic_count > 0 {
            internal_blockers.push(UnsupportedCapability {
                capability: "valid Rust lexical source".to_owned(),
                required_by: "compile Rust crate graph".to_owned(),
                reason: format!(
                    "upstream rustc_lexer reported {lexical_diagnostic_count} lexical diagnostic(s)"
                ),
            });
        }
        if build_plan
            .units
            .iter()
            .any(|unit| unit.phase == CompilePhase::BuildScript)
        {
            internal_blockers.push(UnsupportedCapability {
                capability: "build.rs compilation to compile-time WASM".to_owned(),
                required_by: "Cargo build script directives and generated files".to_owned(),
                reason: "precompiled compile-time WASM execution is embedded, but compiling build.rs source into sandbox modules is not embedded yet".to_owned(),
            });
        }
        if build_plan
            .units
            .iter()
            .any(|unit| unit.phase == CompilePhase::ProcMacro)
        {
            internal_blockers.push(UnsupportedCapability {
                capability: "proc-macro crate compilation to compile-time WASM".to_owned(),
                required_by: "Rust macro expansion".to_owned(),
                reason: "precompiled proc-macro WASM token-stream execution is embedded, but compiling proc-macro crates into sandbox modules is not embedded yet".to_owned(),
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

        let has_internal_blockers = !internal_blockers.is_empty();
        let mut build_chain_findings = internal_blockers;
        build_chain_findings.extend(unsupported);

        let status = if has_internal_blockers {
            RunStatus::Failed
        } else if !build_chain_findings.is_empty() {
            RunStatus::Unsupported
        } else {
            RunStatus::Succeeded
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
                status: if has_internal_blockers {
                    ProofStatus::Failed
                } else {
                    ProofStatus::Unsupported
                },
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
                    if has_internal_blockers {
                        ProofStatus::Failed
                    } else {
                        ProofStatus::Unsupported
                    }
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
        let run_root = source_relative_path(&source_root, &format!(".rouwdi/runs/{run_id}"))?;
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
        for cache_entry in &source_cache.entries {
            for file in &cache_entry.files {
                hashes.push(HashEntry {
                    label: format!(
                        "source-cache:{}:{}",
                        cache_entry.dependency, file.source_path
                    ),
                    path: file.cache_path.clone(),
                    sha256: file.sha256.clone(),
                });
            }
        }

        let mut manifest = RouwdiRunManifest {
            run_id: run_id.clone(),
            status,
            contract_sha256: normalized.sha256.clone(),
            source_tree_sha256: source_snapshot.tree_sha256.clone(),
            compiler_engine: self.target_registry.compiler.clone(),
            target_packs,
            artifacts: Vec::<ArtifactManifestEntry>::new(),
            unsupported: build_chain_findings.clone(),
            proof_files: Vec::new(),
        };
        let mut bundle = ProofBundle {
            manifest: manifest.clone(),
            normalized_contract: normalized,
            source_snapshot,
            source_cache,
            cargo_workspace,
            cargo_features,
            source_fetch_plan,
            build_plan,
            compile_time_plan,
            rust_source_lex,
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
            unsupported: build_chain_findings,
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

fn lex_build_plan_sources(
    storage: &dyn Storage,
    build_plan: &rouwdi_cargo::CargoBuildPlan,
) -> Result<Vec<RustSourceLexProof>, EngineError> {
    let mut paths = BTreeSet::new();
    for unit in &build_plan.units {
        if let Some(path) = &unit.source_path {
            paths.insert(path.clone());
        }
    }

    let mut proofs = Vec::new();
    for path in paths {
        let bytes = storage.read(&path)?;
        let source = String::from_utf8_lossy(&bytes);
        proofs.push(lex_rust_source_with_diagnostics(&path, &source));
    }
    Ok(proofs)
}

impl Default for RouwdiEngine {
    fn default() -> Self {
        Self::new(TargetPackRegistry::strict_embedded())
    }
}

fn parent_path(path: &str) -> Result<String, VfsError> {
    let path = normalize_path(path)?;
    Ok(path
        .rsplit_once('/')
        .map(|(parent, _)| parent.to_owned())
        .unwrap_or_default())
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
    fn build_writes_proof_bundle_and_fails_without_embedded_compiler() {
        let mut storage = fixture_storage();

        let report = RouwdiEngine::default()
            .build(&mut storage, BuildRequest::default())
            .unwrap();

        assert_eq!(report.status, RunStatus::Failed);
        assert!(report
            .unsupported
            .iter()
            .any(|item| item.capability == "rustc frontend semantics"));
        assert!(storage
            .read(&report.manifest_path)
            .unwrap()
            .starts_with(b"{"));
        assert!(storage
            .read(&format!("{}/source/source-cache.json", report.run_root))
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
            .read(&format!("{}/graph/rust-source-lex.json", report.run_root))
            .unwrap()
            .starts_with(b"["));
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

        assert_eq!(verify.status, RunStatus::Failed);
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

    #[test]
    fn contract_paths_are_resolved_relative_to_contract_file() {
        let mut storage = MemoryStorage::new();
        storage
            .write(
                "project/rouwdi.toml",
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
"#,
            )
            .unwrap();
        storage
            .write(
                "project/Cargo.toml",
                br#"
[package]
name = "app"
version = "0.1.0"
edition = "2021"
"#,
            )
            .unwrap();
        storage
            .write(
                "project/Cargo.lock",
                br#"
version = 4

[[package]]
name = "app"
version = "0.1.0"
"#,
            )
            .unwrap();
        storage
            .write("project/src/main.rs", b"fn main() {}\n")
            .unwrap();
        storage
            .write(
                "Cargo.toml",
                br#"
[package]
name = "wrong-root"
version = "0.1.0"
"#,
            )
            .unwrap();

        let report = RouwdiEngine::default()
            .build(
                &mut storage,
                BuildRequest {
                    contract_path: "project/rouwdi.toml".to_owned(),
                },
            )
            .unwrap();
        let snapshot: rouwdi_source::SourceSnapshot = serde_json::from_slice(
            &storage
                .read(&format!("{}/source/source-snapshot.json", report.run_root))
                .unwrap(),
        )
        .unwrap();

        assert!(report.run_root.starts_with("project/.rouwdi/runs/"));
        assert_eq!(snapshot.root, "project");
        assert!(snapshot
            .files
            .iter()
            .any(|file| file.path == "project/src/main.rs"));
        assert!(!snapshot.files.iter().any(|file| file.path == "Cargo.toml"));
        assert!(storage
            .read(&format!("{}/graph/cargo-lock.json", report.run_root))
            .unwrap()
            .starts_with(b"{"));
    }

    #[test]
    fn build_records_rust_lexer_diagnostics_inside_the_proof_bundle() {
        let mut storage = fixture_storage();
        storage
            .write("src/main.rs", b"fn main() { \"open\n")
            .unwrap();

        let report = RouwdiEngine::default()
            .build(&mut storage, BuildRequest::default())
            .unwrap();
        let lex_proofs: Vec<RustSourceLexProof> = serde_json::from_slice(
            &storage
                .read(&format!("{}/graph/rust-source-lex.json", report.run_root))
                .unwrap(),
        )
        .unwrap();

        assert_eq!(report.status, RunStatus::Failed);
        assert!(report
            .unsupported
            .iter()
            .any(|item| item.capability == "valid Rust lexical source"));
        assert!(lex_proofs.iter().any(|proof| {
            proof.path == "src/main.rs"
                && proof
                    .diagnostics
                    .iter()
                    .any(|diagnostic| diagnostic.message == "unterminated string literal")
        }));
    }

    #[test]
    fn build_materializes_path_dependency_source_cache_inside_rouwdi_state() {
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
root = "."

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

[dependencies]
helper = { path = "helper" }
"#,
            )
            .unwrap();
        storage
            .write(
                "Cargo.lock",
                br#"
version = 4

[[package]]
name = "app"
version = "0.1.0"

[[package]]
name = "helper"
version = "0.1.0"
"#,
            )
            .unwrap();
        storage.write("src/main.rs", b"fn main() {}\n").unwrap();
        storage
            .write(
                "helper/Cargo.toml",
                br#"
[package]
name = "helper"
version = "0.1.0"
edition = "2021"
"#,
            )
            .unwrap();
        storage
            .write("helper/src/lib.rs", b"pub fn helper() {}\n")
            .unwrap();

        let report = RouwdiEngine::default()
            .build(&mut storage, BuildRequest::default())
            .unwrap();
        let source_cache: rouwdi_source::SourceCacheProof = serde_json::from_slice(
            &storage
                .read(&format!("{}/source/source-cache.json", report.run_root))
                .unwrap(),
        )
        .unwrap();

        let helper = source_cache
            .entries
            .iter()
            .find(|entry| entry.dependency == "helper")
            .unwrap();
        assert_eq!(helper.status, rouwdi_source::SourceCacheStatus::Cached);
        assert!(helper
            .cache_path
            .as_deref()
            .unwrap()
            .starts_with(".rouwdi/cache/sources/"));
        assert!(helper.files.iter().any(|file| {
            file.cache_path.ends_with("src/lib.rs")
                && storage.read(&file.cache_path).unwrap() == b"pub fn helper() {}\n"
        }));
        let hashes: Vec<HashEntry> = serde_json::from_slice(
            &storage
                .read(&format!("{}/proofs/hashes.json", report.run_root))
                .unwrap(),
        )
        .unwrap();
        assert!(hashes.iter().any(|hash| {
            hash.label == "source-cache:helper:helper/src/lib.rs"
                && hash.path.ends_with("src/lib.rs")
        }));
    }

    #[test]
    fn build_records_remote_dependency_fetcher_as_internal_blocker() {
        let mut storage = fixture_storage();
        storage
            .write(
                "Cargo.toml",
                br#"
[package]
name = "app"
version = "0.1.0"
edition = "2021"

[dependencies]
serde = "1"
"#,
            )
            .unwrap();
        storage
            .write(
                "Cargo.lock",
                br#"
version = 4

[[package]]
name = "app"
version = "0.1.0"
dependencies = [
 "serde",
]

[[package]]
name = "serde"
version = "1.0.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "abc123"
"#,
            )
            .unwrap();

        let report = RouwdiEngine::default()
            .build(&mut storage, BuildRequest::default())
            .unwrap();
        let source_cache: rouwdi_source::SourceCacheProof = serde_json::from_slice(
            &storage
                .read(&format!("{}/source/source-cache.json", report.run_root))
                .unwrap(),
        )
        .unwrap();

        assert!(source_cache.entries.iter().any(|entry| {
            entry.dependency == "serde"
                && entry.status == rouwdi_source::SourceCacheStatus::PlannedFetch
        }));
        assert!(report.unsupported.iter().any(|item| {
            item.capability == "Registry source fetcher"
                && item.required_by.contains("serde")
                && item.reason.contains("no host Cargo")
        }));
    }

    #[test]
    fn build_materializes_vendored_registry_dependency_source_cache() {
        let mut storage = fixture_storage();
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

[resolver]
vendor = ".rouwdi/vendor"

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

[dependencies]
serde = "1"
"#,
            )
            .unwrap();
        storage
            .write(
                "Cargo.lock",
                br#"
version = 4

[[package]]
name = "app"
version = "0.1.0"
dependencies = [
 "serde",
]

[[package]]
name = "serde"
version = "1.0.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "abc123"
"#,
            )
            .unwrap();
        storage
            .write(
                ".rouwdi/vendor/serde/Cargo.toml",
                b"[package]\nname='serde'\nversion='1.0.0'\n",
            )
            .unwrap();
        storage
            .write(
                ".rouwdi/vendor/serde/src/lib.rs",
                b"pub trait Serialize {}\n",
            )
            .unwrap();

        let report = RouwdiEngine::default()
            .build(&mut storage, BuildRequest::default())
            .unwrap();
        let source_cache: rouwdi_source::SourceCacheProof = serde_json::from_slice(
            &storage
                .read(&format!("{}/source/source-cache.json", report.run_root))
                .unwrap(),
        )
        .unwrap();

        let serde = source_cache
            .entries
            .iter()
            .find(|entry| entry.dependency == "serde")
            .unwrap();
        assert_eq!(serde.status, rouwdi_source::SourceCacheStatus::Cached);
        assert!(serde
            .files
            .iter()
            .any(|file| file.source_path == ".rouwdi/vendor/serde/src/lib.rs"));
        assert!(!report
            .unsupported
            .iter()
            .any(|item| item.capability == "Registry source fetcher"));
        let hashes: Vec<HashEntry> = serde_json::from_slice(
            &storage
                .read(&format!("{}/proofs/hashes.json", report.run_root))
                .unwrap(),
        )
        .unwrap();
        assert!(hashes.iter().any(|hash| {
            hash.label == "source-cache:serde:.rouwdi/vendor/serde/src/lib.rs"
                && hash.path.contains(".rouwdi/cache/sources/")
        }));
    }
}
