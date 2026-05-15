use rouwdi_cargo::{
    parse_lockfile, plan_build, plan_source_fetches, resolve_features, resolve_workspace,
    validate_lockfile_against_fetch_plan, CargoModelError, CargoSourceKind, CargoTargetKind,
    CompilePhase,
};
use rouwdi_compiletime::plan_compile_time;
use rouwdi_contract::{ArtifactKind, ContractError, RouwdiContract};
use rouwdi_proof::{
    hash_bytes, verify_manifest_hashes, verify_manifest_references, ArtifactInterfaceProof,
    ArtifactManifestEntry, ArtifactPipelineCompileUnit, ArtifactPipelineRecord,
    ArtifactPipelineStageRecord, ArtifactPipelineStageStatus, BootstrapDiagnostic, HashEntry,
    ProofBundle, ProofError, ProofStatus, RouwdiRunManifest, RunStatus, RuntimeProof,
};
use rouwdi_rustc::{
    lex_rust_source_with_diagnostics, run_rust_compiler_pipeline_record,
    RustBorrowCheckStageStatus, RustCompileRequest, RustCompilerPipelineRecord, RustCompilerStage,
    RustExpansionStageStatus, RustExternCrate, RustNameResolutionStageStatus, RustParseStageStatus,
    RustSourceLexProof, RustTypeCheckStageStatus,
};
use rouwdi_source::{
    materialize_source_cache_with_options, snapshot_source, source_relative_path, SourceCacheKind,
    SourceCacheOptions, SourceCacheRequest, SourceCacheStatus, SourceError,
};
use rouwdi_targets::{TargetError, TargetPackRegistry};
use rouwdi_vfs::{join_path, normalize_path, Storage, VfsError};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

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
    pub bootstrap_diagnostics: Vec<BootstrapDiagnostic>,
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
            selected_target_kind.clone(),
            &contract.project.profile,
            &target_triples,
        )?;
        let compile_time_plan = plan_compile_time(&build_plan);
        let rust_source_lex = lex_build_plan_sources(storage, &build_plan)?;
        let compiler_pipeline = run_compiler_pipeline(storage, &build_plan)?;
        let rust_source_parse = compiler_pipeline
            .iter()
            .filter_map(|record| record.parse_stage.clone())
            .collect::<Vec<_>>();
        let rust_source_expansion = compiler_pipeline
            .iter()
            .filter_map(|record| record.expansion_stage.clone())
            .collect::<Vec<_>>();
        let rust_source_name_resolution = compiler_pipeline
            .iter()
            .filter_map(|record| record.name_resolution_stage.clone())
            .collect::<Vec<_>>();
        let rust_source_type_check = compiler_pipeline
            .iter()
            .filter_map(|record| record.type_check_stage.clone())
            .collect::<Vec<_>>();
        let rust_source_borrow_check = compiler_pipeline
            .iter()
            .filter_map(|record| record.borrow_check_stage.clone())
            .collect::<Vec<_>>();
        let rust_source_mir_handoff = compiler_pipeline
            .iter()
            .filter_map(|record| record.mir_handoff.clone())
            .collect::<Vec<_>>();
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
        let run_id = deterministic_run_id(&normalized.sha256, &source_snapshot.tree_sha256);
        let run_root = source_relative_path(&source_root, &format!(".rouwdi/runs/{run_id}"))?;
        let artifact_pipeline = plan_artifact_pipeline(
            &contract,
            &compiler_pipeline,
            &run_root,
            &contract.project.package,
            &selected_target,
            selected_target_kind,
        );

        let mut assembly_diagnostics = Vec::new();
        let mut host_diagnostics = Vec::new();
        for cache_entry in &source_cache.entries {
            if cache_entry.status == SourceCacheStatus::PlannedFetch {
                assembly_diagnostics.push(BootstrapDiagnostic {
                    component: format!("{:?} source fetcher", cache_entry.kind),
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
                assembly_diagnostics.push(BootstrapDiagnostic {
                    component: format!("{} target pack", pack.triple),
                    required_by: format!("artifact emission for {}", pack.triple),
                    reason: "target ABI/object/link metadata is not embedded in this assembly"
                        .to_owned(),
                });
            }
            if !pack.std_pack_embedded {
                assembly_diagnostics.push(BootstrapDiagnostic {
                    component: format!("{} std/core/alloc pack", pack.triple),
                    required_by: format!("Rust standard library resolution for {}", pack.triple),
                    reason: "std/core/alloc artifacts are not embedded in this assembly".to_owned(),
                });
            }
            if !pack.linker_pack_embedded {
                assembly_diagnostics.push(BootstrapDiagnostic {
                    component: format!("{} linker pack", pack.triple),
                    required_by: format!("final link for {}", pack.triple),
                    reason: "linker scripts/configuration/runtime objects are not embedded in this assembly"
                        .to_owned(),
                });
            }
        }
        let lexical_diagnostic_count = rust_source_lex
            .iter()
            .map(|proof| proof.diagnostics.len())
            .sum::<usize>();
        if lexical_diagnostic_count > 0 {
            assembly_diagnostics.push(BootstrapDiagnostic {
                component: "valid Rust lexical source".to_owned(),
                required_by: "compile Rust crate graph".to_owned(),
                reason: format!(
                    "upstream rustc_lexer reported {lexical_diagnostic_count} lexical diagnostic(s)"
                ),
            });
        }
        for record in &compiler_pipeline {
            if let Some(parse_stage) = &record.parse_stage {
                if parse_stage.status == RustParseStageStatus::Failed {
                    assembly_diagnostics.push(BootstrapDiagnostic {
                        component: "Rust parse stage".to_owned(),
                        required_by: format!("compile unit {}", parse_stage.unit_id),
                        reason: format!(
                            "internal Rust parser reported {} diagnostic(s) for {}",
                            parse_stage.diagnostic_count, parse_stage.source_path
                        ),
                    });
                }
            }
            if let Some(expansion_stage) = &record.expansion_stage {
                if expansion_stage.status == RustExpansionStageStatus::ExpansionRequired {
                    let required_features = expansion_stage
                        .diagnostics
                        .iter()
                        .map(|diagnostic| diagnostic.feature.as_str())
                        .collect::<Vec<_>>()
                        .join(", ");
                    assembly_diagnostics.push(BootstrapDiagnostic {
                        component: "Rust macro expansion stage".to_owned(),
                        required_by: format!("compile unit {}", expansion_stage.unit_id),
                        reason: format!(
                            "internal Rust expansion requires embedded support for {required_features}"
                        ),
                    });
                }
            }
            if let Some(name_resolution_stage) = &record.name_resolution_stage {
                if name_resolution_stage.status == RustNameResolutionStageStatus::Failed {
                    assembly_diagnostics.push(BootstrapDiagnostic {
                        component: "Rust name resolution stage".to_owned(),
                        required_by: format!("compile unit {}", name_resolution_stage.unit_id),
                        reason: format!(
                            "internal Rust name resolution reported {} diagnostic(s) for {}",
                            name_resolution_stage.diagnostic_count,
                            name_resolution_stage.source_path
                        ),
                    });
                }
            }
            if let Some(type_check_stage) = &record.type_check_stage {
                if type_check_stage.status == RustTypeCheckStageStatus::Failed {
                    assembly_diagnostics.push(BootstrapDiagnostic {
                        component: "Rust type-check stage".to_owned(),
                        required_by: format!("compile unit {}", type_check_stage.unit_id),
                        reason: format!(
                            "internal Rust type checker reported {} diagnostic(s) for {}",
                            type_check_stage.diagnostic_count, type_check_stage.source_path
                        ),
                    });
                }
            }
            if let Some(borrow_check_stage) = &record.borrow_check_stage {
                if borrow_check_stage.status == RustBorrowCheckStageStatus::Failed {
                    assembly_diagnostics.push(BootstrapDiagnostic {
                        component: "Rust borrow-check stage".to_owned(),
                        required_by: format!("compile unit {}", borrow_check_stage.unit_id),
                        reason: format!(
                            "internal Rust borrow checker reported {} diagnostic(s) for {}",
                            borrow_check_stage.diagnostic_count, borrow_check_stage.source_path
                        ),
                    });
                }
            }
            if let Some(mir_handoff) = &record.mir_handoff {
                if !mir_handoff.upstream_mir_adapter_available {
                    assembly_diagnostics.push(BootstrapDiagnostic {
                        component: format!(
                            "upstream MIR adapter {}",
                            mir_handoff
                                .blocker_component
                                .as_deref()
                                .unwrap_or(mir_handoff.intended_upstream_component.as_str())
                        ),
                        required_by: format!("compile unit {}", mir_handoff.compile_unit.unit_id),
                        reason: mir_handoff.blocker_reason.clone().unwrap_or_else(|| {
                            "upstream MIR adapter is not embedded in this assembly".to_owned()
                        }),
                    });
                }
            }
        }
        for record in &compiler_pipeline {
            if let Some(missing_stage) = &record.missing_stage {
                assembly_diagnostics.push(BootstrapDiagnostic {
                    component: missing_stage.component(),
                    required_by: missing_stage.required_by(),
                    reason: missing_stage.reason.clone(),
                });
            }
        }
        if build_plan
            .units
            .iter()
            .any(|unit| unit.phase == CompilePhase::BuildScript)
        {
            assembly_diagnostics.push(BootstrapDiagnostic {
                component: "build.rs compilation to compile-time WASM".to_owned(),
                required_by: "Cargo build script directives and generated files".to_owned(),
                reason: "precompiled compile-time WASM execution is embedded, but compiling build.rs source into sandbox modules is not embedded yet".to_owned(),
            });
        }
        if build_plan
            .units
            .iter()
            .any(|unit| unit.phase == CompilePhase::ProcMacro)
        {
            assembly_diagnostics.push(BootstrapDiagnostic {
                component: "proc-macro crate compilation to compile-time WASM".to_owned(),
                required_by: "Rust macro expansion".to_owned(),
                reason: "precompiled proc-macro WASM token-stream execution is embedded, but compiling proc-macro crates into sandbox modules is not embedded yet".to_owned(),
            });
        }
        for target in &contract.targets {
            if target.runtime.required && target.triple == "native_host" {
                host_diagnostics.push(BootstrapDiagnostic {
                    component: "native runtime execution".to_owned(),
                    required_by: format!("runtime proof for target {}", target.name),
                    reason: "native execution is a host runtime capability and must be recorded as delegated or unavailable in the current host substrate".to_owned(),
                });
            }
        }

        let has_assembly_diagnostics = !assembly_diagnostics.is_empty();
        let mut bootstrap_diagnostics = assembly_diagnostics;
        bootstrap_diagnostics.extend(host_diagnostics);

        let status = if has_assembly_diagnostics {
            RunStatus::Failed
        } else if !bootstrap_diagnostics.is_empty() {
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
                status: if has_assembly_diagnostics {
                    ProofStatus::Failed
                } else {
                    ProofStatus::Unsupported
                },
                reason: artifact_pipeline
                    .iter()
                    .find(|record| record.target_name == target.name)
                    .and_then(|record| record.blocker_reason.clone())
                    .or_else(|| {
                        Some(
                            "artifact was not emitted because upstream compiler payload stages are not embedded"
                                .to_owned(),
                        )
                    }),
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
                    if has_assembly_diagnostics {
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
            compiler_pipeline,
            artifact_pipeline: artifact_pipeline.clone(),
            artifacts: Vec::<ArtifactManifestEntry>::new(),
            bootstrap_diagnostics: bootstrap_diagnostics.clone(),
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
            rust_source_parse,
            rust_source_expansion,
            rust_source_name_resolution,
            rust_source_type_check,
            rust_source_borrow_check,
            rust_source_mir_handoff,
            artifact_pipeline,
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
            bootstrap_diagnostics,
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

fn run_compiler_pipeline(
    storage: &dyn Storage,
    build_plan: &rouwdi_cargo::CargoBuildPlan,
) -> Result<Vec<RustCompilerPipelineRecord>, EngineError> {
    let mut records = Vec::new();
    for unit in build_plan
        .units
        .iter()
        .filter(|unit| unit.phase == CompilePhase::Rust)
    {
        let source_path = unit.source_path.clone().ok_or_else(|| {
            VfsError::NotFound(format!("source path for compile unit {}", unit.id))
        })?;
        let bytes = storage.read(&source_path)?;
        let source = String::from_utf8_lossy(&bytes);
        let request = RustCompileRequest {
            unit_id: unit.id.clone(),
            package: unit.package.clone(),
            target: unit.target.clone(),
            target_kind: format!("{:?}", unit.target_kind),
            source_path,
            triple: unit.triple.clone(),
            profile: unit.profile.clone(),
            extern_prelude: extern_prelude_for_unit(build_plan, &unit.id),
        };
        records.push(run_rust_compiler_pipeline_record(&request, &source));
    }
    Ok(records)
}

fn plan_artifact_pipeline(
    contract: &RouwdiContract,
    compiler_pipeline: &[RustCompilerPipelineRecord],
    run_root: &str,
    package: &str,
    selected_target: &str,
    selected_target_kind: CargoTargetKind,
) -> Vec<ArtifactPipelineRecord> {
    contract
        .targets
        .iter()
        .map(|target| {
            let compile_records = compiler_pipeline
                .iter()
                .filter(|record| record.triple == target.triple)
                .collect::<Vec<_>>();
            let mir_blocker = compile_records
                .iter()
                .filter_map(|record| record.mir_handoff.as_ref())
                .find(|handoff| !handoff.upstream_mir_adapter_available);
            let compile_units = compile_records
                .iter()
                .map(|record| artifact_compile_unit_from_record(record))
                .collect::<Vec<_>>();
            let remaining_stages = artifact_pipeline_stage_records(mir_blocker.is_some());
            let expected_output_path =
                expected_artifact_path(run_root, selected_target, &target.triple, target.artifact);

            ArtifactPipelineRecord {
                target_name: target.name.clone(),
                triple: target.triple.clone(),
                package: package.to_owned(),
                cargo_target: selected_target.to_owned(),
                cargo_target_kind: format!("{:?}", selected_target_kind),
                expected_artifact_kind: target.artifact,
                expected_output_path,
                artifact_emitted: false,
                compile_units,
                remaining_stages,
                blocked_at_stage: mir_blocker.map(|handoff| handoff.stage),
                blocker_category: mir_blocker.and_then(|handoff| handoff.blocker_category),
                blocker_component: mir_blocker
                    .and_then(|handoff| handoff.blocker_component.clone()),
                blocker_reason: mir_blocker.and_then(|handoff| handoff.blocker_reason.clone()),
            }
        })
        .collect()
}

fn artifact_compile_unit_from_record(
    record: &RustCompilerPipelineRecord,
) -> ArtifactPipelineCompileUnit {
    ArtifactPipelineCompileUnit {
        unit_id: record.unit_id.clone(),
        package: record.package.clone(),
        target: record.target.clone(),
        target_kind: record.target_kind.clone(),
        source_path: record.source_path.clone(),
        triple: record.triple.clone(),
        frontend_parse_status: record
            .parse_stage
            .as_ref()
            .map(|stage| format!("{:?}", stage.status)),
        frontend_expansion_status: record
            .expansion_stage
            .as_ref()
            .map(|stage| format!("{:?}", stage.status)),
        frontend_name_resolution_status: record
            .name_resolution_stage
            .as_ref()
            .map(|stage| format!("{:?}", stage.status)),
        frontend_type_check_status: record
            .type_check_stage
            .as_ref()
            .map(|stage| format!("{:?}", stage.status)),
        frontend_borrow_check_status: record
            .borrow_check_stage
            .as_ref()
            .map(|stage| format!("{:?}", stage.status)),
        mir_handoff_status: record.mir_handoff.as_ref().map(|handoff| handoff.status),
        mir_handoff_blocker_component: record
            .mir_handoff
            .as_ref()
            .and_then(|handoff| handoff.blocker_component.clone()),
    }
}

fn artifact_pipeline_stage_records(mir_blocked: bool) -> Vec<ArtifactPipelineStageRecord> {
    [
        (
            RustCompilerStage::Mir,
            "rustc_middle",
            "MIR, query model, and compiler metadata",
        ),
        (
            RustCompilerStage::Monomorphization,
            "rustc_monomorphize",
            "monomorphization collector",
        ),
        (
            RustCompilerStage::Codegen,
            "rustc_codegen_llvm",
            "LLVM-grade codegen backend",
        ),
        (
            RustCompilerStage::Linking,
            "lld",
            "native and WebAssembly linker implementation",
        ),
        (
            RustCompilerStage::ArtifactEmission,
            "rouwdi_artifact_writer",
            "final artifact writer fed by upstream compiler payload",
        ),
    ]
    .into_iter()
    .map(|(stage, required_component, component_role)| {
        let status = if mir_blocked && stage == RustCompilerStage::Mir {
            ArtifactPipelineStageStatus::Blocked
        } else if mir_blocked {
            ArtifactPipelineStageStatus::WaitingOnUpstreamMir
        } else {
            ArtifactPipelineStageStatus::Planned
        };
        ArtifactPipelineStageRecord {
            stage,
            required_component: required_component.to_owned(),
            component_role: component_role.to_owned(),
            adapter_available: false,
            status,
        }
    })
    .collect()
}

fn expected_artifact_path(
    run_root: &str,
    selected_target: &str,
    triple: &str,
    artifact: ArtifactKind,
) -> String {
    let filename = match artifact {
        ArtifactKind::Module => format!("{selected_target}-{triple}.wasm"),
        ArtifactKind::Component => format!("{selected_target}-{triple}.component.wasm"),
        ArtifactKind::Executable => format!("{selected_target}-{triple}"),
        ArtifactKind::Staticlib => format!("lib{selected_target}-{triple}.a"),
        ArtifactKind::Archive => format!("{selected_target}-{triple}.a"),
        ArtifactKind::Object => format!("{selected_target}-{triple}.o"),
    };
    format!("{run_root}/artifacts/{filename}")
}

fn extern_prelude_for_unit(
    build_plan: &rouwdi_cargo::CargoBuildPlan,
    unit_id: &str,
) -> Vec<RustExternCrate> {
    let units_by_id = build_plan
        .units
        .iter()
        .map(|unit| (unit.id.as_str(), unit))
        .collect::<BTreeMap<_, _>>();
    let mut externs = BTreeMap::<String, RustExternCrate>::new();
    for edge in build_plan.edges.iter().filter(|edge| edge.to == unit_id) {
        let Some(from_unit) = units_by_id.get(edge.from.as_str()) else {
            continue;
        };
        if from_unit.phase != CompilePhase::Rust {
            continue;
        }
        let crate_name = dependency_crate_name_from_edge_reason(&edge.reason)
            .unwrap_or_else(|| from_unit.package.clone())
            .replace('-', "_");
        externs
            .entry(crate_name.clone())
            .or_insert(RustExternCrate {
                name: crate_name,
                source_unit_id: Some(from_unit.id.clone()),
                package: Some(from_unit.package.clone()),
            });
    }
    externs.into_values().collect()
}

fn dependency_crate_name_from_edge_reason(reason: &str) -> Option<String> {
    reason
        .strip_prefix("Normal dependency ")
        .or_else(|| reason.strip_prefix("Build dependency "))
        .map(str::to_owned)
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
            .bootstrap_diagnostics
            .iter()
            .any(
                |item| item.component == "upstream MIR adapter mir_handoff_payload_adapter"
                    && item.required_by == "compile unit app:rust:app:wasm32-wasip1"
                    && item.reason.contains("bootstrap authoritative probe")
            ));
        assert!(!report
            .bootstrap_diagnostics
            .iter()
            .any(|item| item.component == "rustc frontend semantics"));
        assert!(storage
            .read(&report.manifest_path)
            .unwrap()
            .starts_with(b"{"));
        let manifest: RouwdiRunManifest =
            serde_json::from_slice(&storage.read(&report.manifest_path).unwrap()).unwrap();
        assert_eq!(manifest.compiler_pipeline.len(), 1);
        assert!(manifest.compiler_pipeline[0].parse_stage.is_some());
        assert!(manifest.compiler_pipeline[0].expansion_stage.is_some());
        assert!(manifest.compiler_pipeline[0].missing_stage.is_none());
        assert_eq!(
            manifest.compiler_pipeline[0]
                .mir_handoff
                .as_ref()
                .unwrap()
                .blocker_component
                .as_deref(),
            Some("mir_handoff_payload_adapter")
        );
        assert_eq!(
            manifest.compiler_pipeline[0]
                .mir_handoff
                .as_ref()
                .unwrap()
                .payload_adapter_status,
            "payload_context_attempted"
        );
        let manifest_handoff = manifest.compiler_pipeline[0].mir_handoff.as_ref().unwrap();
        assert_eq!(
            manifest_handoff.payload_carrier_state.as_deref(),
            Some("payload_context_attempted")
        );
        assert!(manifest_handoff.payload_adapter_bootstrap_artifact_located);
        assert!(manifest_handoff.payload_carrier_created);
        assert!(!manifest_handoff.payload_loaded_into_rouwdi_facade);
        assert_eq!(
            manifest_handoff.payload_abi_selected_route.as_deref(),
            Some("wasm32_wasip1_module")
        );
        assert_eq!(
            manifest_handoff.payload_abi_bridge_blocker_kind.as_deref(),
            Some("missing_core_lang_item_copy")
        );
        assert_eq!(
            manifest_handoff.payload_milestone_state.as_deref(),
            Some("bridge_wasm_core_metadata_loaded_blocked_at_missing_core_lang_item_copy")
        );
        let target_pack = manifest_handoff.payload_target_pack.as_ref().unwrap();
        assert_eq!(target_pack.target_triple, "wasm32-wasip1");
        assert!(target_pack.attempted);
        assert_eq!(target_pack.status, "ready");
        assert_eq!(target_pack.exit_code, 0);
        assert_eq!(target_pack.blocker_kind, "none");
        assert!(target_pack.std_available);
        assert!(target_pack.core_available);
        assert!(target_pack.alloc_available);
        let bridge_attempt = manifest_handoff.payload_bridge_attempt.as_ref().unwrap();
        assert_eq!(bridge_attempt.status, "context_attempted");
        assert_eq!(bridge_attempt.blocker_kind, "missing_core_lang_item_copy");
        assert_eq!(
            manifest.compiler_pipeline[0]
                .mir_handoff
                .as_ref()
                .unwrap()
                .intended_upstream_component,
            "rustc_middle"
        );
        assert_eq!(manifest.artifact_pipeline.len(), 1);
        assert_eq!(manifest.artifact_pipeline[0].target_name, "wasi");
        assert_eq!(
            manifest.artifact_pipeline[0].expected_output_path,
            format!("{}/artifacts/app-wasm32-wasip1.wasm", report.run_root)
        );
        assert!(!manifest.artifact_pipeline[0].artifact_emitted);
        assert!(manifest.compiler_pipeline[0].type_check_stage.is_some());
        assert!(manifest.compiler_pipeline[0].borrow_check_stage.is_some());
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
            .read(&format!("{}/graph/rust-source-parse.json", report.run_root))
            .unwrap()
            .starts_with(b"["));
        assert!(storage
            .read(&format!(
                "{}/graph/rust-source-expansion.json",
                report.run_root
            ))
            .unwrap()
            .starts_with(b"["));
        assert!(storage
            .read(&format!(
                "{}/graph/rust-source-name-resolution.json",
                report.run_root
            ))
            .unwrap()
            .starts_with(b"["));
        assert!(storage
            .read(&format!(
                "{}/graph/rust-source-type-check.json",
                report.run_root
            ))
            .unwrap()
            .starts_with(b"["));
        assert!(storage
            .read(&format!(
                "{}/graph/rust-source-borrow-check.json",
                report.run_root
            ))
            .unwrap()
            .starts_with(b"["));
        assert!(storage
            .read(&format!(
                "{}/graph/rust-source-mir-handoff.json",
                report.run_root
            ))
            .unwrap()
            .starts_with(b"["));
        assert!(storage
            .read(&format!("{}/graph/artifact-pipeline.json", report.run_root))
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
            .bootstrap_diagnostics
            .iter()
            .any(|item| item.component == "valid Rust lexical source"));
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
    fn build_records_remote_dependency_fetcher_as_bootstrap_diagnostic() {
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
        assert!(report.bootstrap_diagnostics.iter().any(|item| {
            item.component == "Registry source fetcher"
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
            .bootstrap_diagnostics
            .iter()
            .any(|item| item.component == "Registry source fetcher"));
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
