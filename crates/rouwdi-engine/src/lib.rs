use rouwdi_cargo::{
    parse_lockfile, plan_build, plan_source_fetches, resolve_features, resolve_workspace,
    validate_lockfile_against_fetch_plan, CargoModelError, CargoSourceKind, CargoTargetKind,
    CompilePhase,
};
use rouwdi_compiletime::plan_compile_time;
use rouwdi_contract::{ArtifactKind, ContractError, RouwdiContract, RuntimeKind};
use rouwdi_proof::{
    hash_bytes, missing_wasm_exports, parse_wasm_exports, parse_wasm_imports,
    verify_manifest_hashes, verify_manifest_references, ArtifactInterfaceProof,
    ArtifactManifestEntry, ArtifactPipelineCompileUnit, ArtifactPipelineRecord,
    ArtifactPipelineStageRecord, ArtifactPipelineStageStatus, BootstrapDiagnostic, HashEntry,
    ProofBundle, ProofError, ProofStatus, RouwdiRunManifest, RunStatus, RuntimeProof,
};
use rouwdi_rustc::{
    lex_rust_source_with_diagnostics, run_rust_compiler_pipeline_record_with_embedded_mir_payload,
    RustBorrowCheckStageStatus, RustCompileArtifactKind, RustCompileArtifactRecord,
    RustCompileRequest, RustCompilerPipelineRecord, RustCompilerPipelineStatus, RustCompilerStage,
    RustEmbeddedMirPayloadExecution, RustExpansionStageStatus, RustExternCrate,
    RustNameResolutionStageStatus, RustParseStageStatus, RustSourceLexProof,
    RustTypeCheckStageStatus,
};
use rouwdi_source::{
    materialize_source_cache_with_options, snapshot_source, source_relative_path, SourceCacheKind,
    SourceCacheOptions, SourceCacheRequest, SourceCacheStatus, SourceError,
};
use rouwdi_targets::{TargetError, TargetPackRegistry};
use rouwdi_vfs::{join_path, normalize_path, Storage, VfsError};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use wasmi::{Caller, Config, Engine, Extern, Linker, Memory, Module, Store};

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EmbeddedLinkedWasiModuleArtifact {
    pub target_triple: String,
    pub payload_artifact_path: String,
    pub bytes: Vec<u8>,
    pub sha256: String,
    pub size_bytes: u64,
    pub source_path: String,
    pub source_sha256: String,
    pub codegen_input_sha256: String,
    pub codegen_input_source_sha256: String,
    pub codegen_input_source_bytes_sha256: String,
    pub codegen_input_source_origin: String,
    pub codegen_input_source_text: Option<String>,
    pub reported_input_object_hash: Option<String>,
    pub reported_linker_payload_hash: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EmbeddedMirPayloadExecutionRequest {
    pub compile_unit_id: String,
    pub package: String,
    pub target: String,
    pub target_kind: String,
    pub source_path: String,
    pub source_text: String,
    pub source_sha256: String,
    pub source_snapshot_sha256: String,
    pub contract_sha256: String,
    pub target_triple: String,
    pub profile: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EmbeddedLinkedWasiModuleRequest {
    pub compile_unit_id: String,
    pub package: String,
    pub target: String,
    pub cargo_target_kind: String,
    pub source_path: String,
    pub source_bytes: Vec<u8>,
    pub source_sha256: String,
    pub profile: String,
    pub target_triple: String,
    pub crate_name: String,
    pub mir_body_hash: String,
    pub mono_item_graph_hash: String,
    pub mono_items: Vec<String>,
}

pub type EmbeddedMirPayloadExecutionProvider =
    fn(&EmbeddedMirPayloadExecutionRequest) -> Option<RustEmbeddedMirPayloadExecution>;

pub type EmbeddedLinkedWasiModuleProvider =
    fn(&EmbeddedLinkedWasiModuleRequest) -> Option<EmbeddedLinkedWasiModuleArtifact>;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VerifyReport {
    pub run_root: String,
    pub status: RunStatus,
    pub checked_hashes: usize,
}

#[derive(Debug, Clone)]
pub struct RouwdiEngine {
    target_registry: TargetPackRegistry,
    embedded_mir_payload_execution: Option<RustEmbeddedMirPayloadExecution>,
    embedded_mir_payload_execution_provider: Option<EmbeddedMirPayloadExecutionProvider>,
    embedded_linked_wasi_module: Option<EmbeddedLinkedWasiModuleArtifact>,
    embedded_linked_wasi_module_provider: Option<EmbeddedLinkedWasiModuleProvider>,
}

impl RouwdiEngine {
    pub fn new(target_registry: TargetPackRegistry) -> Self {
        Self {
            target_registry,
            embedded_mir_payload_execution: None,
            embedded_mir_payload_execution_provider: None,
            embedded_linked_wasi_module: None,
            embedded_linked_wasi_module_provider: None,
        }
    }

    pub fn with_embedded_mir_payload_execution(
        mut self,
        execution: RustEmbeddedMirPayloadExecution,
    ) -> Self {
        self.embedded_mir_payload_execution = Some(execution);
        self
    }

    pub fn with_embedded_mir_payload_execution_provider(
        mut self,
        provider: EmbeddedMirPayloadExecutionProvider,
    ) -> Self {
        self.embedded_mir_payload_execution_provider = Some(provider);
        self
    }

    pub fn with_embedded_linked_wasi_module(
        mut self,
        artifact: EmbeddedLinkedWasiModuleArtifact,
    ) -> Self {
        self.embedded_linked_wasi_module = Some(artifact);
        self
    }

    pub fn with_embedded_linked_wasi_module_provider(
        mut self,
        provider: EmbeddedLinkedWasiModuleProvider,
    ) -> Self {
        self.embedded_linked_wasi_module_provider = Some(provider);
        self
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
        let mut compiler_pipeline = run_compiler_pipeline(
            storage,
            &build_plan,
            self.embedded_mir_payload_execution.as_ref(),
            self.embedded_mir_payload_execution_provider,
            &normalized.sha256,
            &source_snapshot.tree_sha256,
        )?;
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
        let mut artifact_pipeline = plan_artifact_pipeline(
            &contract,
            &compiler_pipeline,
            &run_root,
            &contract.project.package,
            &selected_target,
            selected_target_kind,
        );
        let artifact_outputs = promote_linked_wasi_module_artifacts(
            storage,
            &contract,
            &mut compiler_pipeline,
            &mut artifact_pipeline,
            self.embedded_linked_wasi_module.as_ref(),
            self.embedded_linked_wasi_module_provider,
        )?;

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
            if !pack.std_pack_embedded
                && !artifact_outputs
                    .promoted_target_triples
                    .contains(&pack.triple)
            {
                assembly_diagnostics.push(BootstrapDiagnostic {
                    component: format!("{} std/core/alloc pack", pack.triple),
                    required_by: format!("Rust standard library resolution for {}", pack.triple),
                    reason: "std/core/alloc artifacts are not embedded in this assembly".to_owned(),
                });
            }
            if !pack.linker_pack_embedded
                && !artifact_outputs
                    .promoted_target_triples
                    .contains(&pack.triple)
            {
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
            let promoted_by_embedded_payload = artifact_outputs
                .promoted_compile_units
                .contains(&record.unit_id);
            if let Some(parse_stage) = &record.parse_stage {
                if parse_stage.status == RustParseStageStatus::Failed
                    && !promoted_by_embedded_payload
                {
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
                if expansion_stage.status == RustExpansionStageStatus::ExpansionRequired
                    && !promoted_by_embedded_payload
                {
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
                if name_resolution_stage.status == RustNameResolutionStageStatus::Failed
                    && !promoted_by_embedded_payload
                {
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
                if type_check_stage.status == RustTypeCheckStageStatus::Failed
                    && !promoted_by_embedded_payload
                {
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
                if borrow_check_stage.status == RustBorrowCheckStageStatus::Failed
                    && !promoted_by_embedded_payload
                {
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
                if !mir_handoff.upstream_mir_adapter_available && !promoted_by_embedded_payload {
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
                if missing_stage.stage == RustCompilerStage::ArtifactEmission
                    && artifact_outputs
                        .promoted_compile_units
                        .contains(&missing_stage.unit_id)
                {
                    continue;
                }
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
        let interface_proofs = artifact_outputs.interface_proofs;
        let runtime_proofs = artifact_outputs.runtime_proofs;
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
        hashes.extend(artifact_outputs.hashes);

        let mut manifest = RouwdiRunManifest {
            run_id: run_id.clone(),
            status,
            contract_sha256: normalized.sha256.clone(),
            source_tree_sha256: source_snapshot.tree_sha256.clone(),
            compiler_engine: self.target_registry.compiler.clone(),
            target_packs,
            compiler_pipeline,
            artifact_pipeline: artifact_pipeline.clone(),
            artifacts: artifact_outputs.artifacts,
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

struct ArtifactPromotionOutput {
    artifacts: Vec<ArtifactManifestEntry>,
    interface_proofs: Vec<ArtifactInterfaceProof>,
    runtime_proofs: Vec<RuntimeProof>,
    hashes: Vec<HashEntry>,
    promoted_compile_units: BTreeSet<String>,
    promoted_target_triples: BTreeSet<String>,
}

fn promote_linked_wasi_module_artifacts(
    storage: &mut dyn Storage,
    contract: &RouwdiContract,
    compiler_pipeline: &mut [RustCompilerPipelineRecord],
    artifact_pipeline: &mut [ArtifactPipelineRecord],
    linked_module: Option<&EmbeddedLinkedWasiModuleArtifact>,
    linked_module_provider: Option<EmbeddedLinkedWasiModuleProvider>,
) -> Result<ArtifactPromotionOutput, EngineError> {
    let mut artifacts = Vec::new();
    let mut interface_proofs = Vec::new();
    let mut runtime_proofs = Vec::new();
    let mut hashes = Vec::new();
    let mut promoted_compile_units = BTreeSet::new();
    let mut promoted_target_triples = BTreeSet::new();

    for target in &contract.targets {
        let Some(pipeline_record) = artifact_pipeline
            .iter_mut()
            .find(|record| record.target_name == target.name)
        else {
            continue;
        };
        let promotion = (target.triple == "wasm32-wasip1"
            && target.artifact == ArtifactKind::Module
            && (linked_module.is_some() || linked_module_provider.is_some()))
        .then(|| {
            compiler_pipeline.iter().find_map(|record| {
                if record.triple != target.triple {
                    return None;
                }
                let handoff = record.mir_handoff.as_ref()?.codegen_handoff.as_ref()?;
                let linker = handoff.linker_handoff.as_ref()?;
                if !handoff.rust_mono_item_wasm_object_emitted
                    || !linker.linker_invoked
                    || linker.exit_code != Some(0)
                    || linker.required_linker_component != "wasm-ld"
                {
                    return None;
                }
                Some((record.unit_id.clone(), handoff.clone(), linker.clone()))
            })
        })
        .flatten()
        .filter(|(_, _, _)| {
            if let Some(module) = linked_module {
                module.target_triple == target.triple
            } else {
                true
            }
        });

        let Some((unit_id, codegen_handoff, linker_handoff)) = promotion else {
            interface_proofs.push(blocked_interface_proof(target, pipeline_record));
            runtime_proofs.push(blocked_runtime_proof(target));
            continue;
        };
        let source_bytes = storage.read(&codegen_handoff.source_path)?;
        let source_sha256 = hash_bytes(&source_bytes);
        if source_sha256 != codegen_handoff.source_hash {
            return Err(ProofError::Verification(format!(
                "MIR/codegen source hash mismatch for {}: VFS {}, compiler proof {}",
                codegen_handoff.source_path, source_sha256, codegen_handoff.source_hash
            ))
            .into());
        }
        let module_request = EmbeddedLinkedWasiModuleRequest {
            compile_unit_id: unit_id.clone(),
            package: codegen_handoff.package.clone(),
            target: codegen_handoff.target.clone(),
            cargo_target_kind: codegen_handoff.target_kind.clone(),
            source_path: codegen_handoff.source_path.clone(),
            source_bytes: source_bytes.clone(),
            source_sha256: source_sha256.clone(),
            profile: codegen_handoff.profile.clone(),
            target_triple: codegen_handoff.target_triple.clone(),
            crate_name: codegen_handoff.crate_identity.clone(),
            mir_body_hash: codegen_handoff.mir_body_hash.clone(),
            mono_item_graph_hash: codegen_handoff.mono_item_graph_hash.clone(),
            mono_items: codegen_handoff
                .mono_items
                .iter()
                .map(|item| item.symbol_name.clone())
                .collect(),
        };
        let provided_module = linked_module_provider.and_then(|provider| provider(&module_request));
        let module = provided_module.as_ref().or(linked_module).ok_or_else(|| {
            ProofError::Verification(format!(
                "no embedded linked WASI module provider returned an artifact for {}",
                unit_id
            ))
        })?;
        let expected_hash = module.sha256.clone();
        let expected_size = module.size_bytes;
        if expected_hash != hash_bytes(&module.bytes) {
            return Err(ProofError::Verification(format!(
                "embedded codegen payload final module hash mismatch: expected {}, got {}",
                expected_hash,
                hash_bytes(&module.bytes)
            ))
            .into());
        }
        if expected_size != module.bytes.len() as u64 {
            return Err(ProofError::Verification(format!(
                "embedded codegen payload final module size mismatch: expected {}, got {}",
                expected_size,
                module.bytes.len()
            ))
            .into());
        }

        if module.source_path != codegen_handoff.source_path {
            return Err(ProofError::Verification(format!(
                "linked module source path mismatch: payload {}, compile unit {}",
                module.source_path, codegen_handoff.source_path
            ))
            .into());
        }
        if module.source_sha256 != source_sha256 {
            return Err(ProofError::Verification(format!(
                "linked module source hash mismatch for {}: payload {}, VFS {}",
                codegen_handoff.source_path, module.source_sha256, source_sha256
            ))
            .into());
        }
        if source_sha256 != module.codegen_input_sha256 {
            return Err(ProofError::Verification(format!(
                "source hash mismatch for {}: source {}, codegen input {}",
                codegen_handoff.source_path, source_sha256, module.codegen_input_sha256
            ))
            .into());
        }
        if source_sha256 != module.codegen_input_source_sha256
            || source_sha256 != module.codegen_input_source_bytes_sha256
        {
            return Err(ProofError::Verification(format!(
                "codegen input hash mismatch for {}: VFS {}, text {}, bytes {}",
                codegen_handoff.source_path,
                source_sha256,
                module.codegen_input_source_sha256,
                module.codegen_input_source_bytes_sha256
            ))
            .into());
        }
        if module.codegen_input_source_origin != "vfs_compile_unit_source" {
            return Err(ProofError::Verification(format!(
                "codegen input source origin must be vfs_compile_unit_source, got {}",
                module.codegen_input_source_origin
            ))
            .into());
        }
        if let Some(source_text) = &module.codegen_input_source_text {
            if source_text.as_bytes() != source_bytes.as_slice() {
                return Err(ProofError::Verification(format!(
                    "codegen input text does not match compile unit source {}",
                    codegen_handoff.source_path
                ))
                .into());
            }
        }

        let artifact_path = pipeline_record.expected_output_path.clone();
        storage.write(&artifact_path, &module.bytes)?;
        let written_bytes = storage.read(&artifact_path)?;
        let written_hash = hash_bytes(&written_bytes);
        if written_hash != expected_hash {
            return Err(ProofError::Verification(format!(
                "written artifact hash mismatch for {artifact_path}: expected {expected_hash}, got {written_hash}"
            ))
            .into());
        }

        let input_object_hash = module.reported_input_object_hash.clone().ok_or_else(|| {
            ProofError::Verification(format!(
                "final module is missing input object hash for {unit_id}"
            ))
        })?;
        let linker_payload_hash = module.reported_linker_payload_hash.clone().ok_or_else(|| {
            ProofError::Verification(format!(
                "final module is missing linker payload hash for {unit_id}"
            ))
        })?;
        if linker_payload_hash != linker_handoff.linker_payload.sha256 {
            return Err(ProofError::Verification(format!(
                "final module linker payload hash mismatch: payload {}, codegen handoff {}",
                linker_payload_hash, linker_handoff.linker_payload.sha256
            ))
            .into());
        }

        let interface_proof = interface_proof_for_emitted_wasm(target, &artifact_path, storage)?;
        let runtime_proof =
            runtime_proof_for_emitted_wasm(target, &artifact_path, &written_bytes, &written_hash)?;
        let runtime_proof_hash = hash_bytes(&serde_json::to_vec(&runtime_proof)?);

        let interface_succeeded = interface_proof.status == ProofStatus::Succeeded;
        let runtime_succeeded = runtime_proof.status == ProofStatus::Succeeded;
        if !interface_succeeded || !runtime_succeeded {
            interface_proofs.push(interface_proof);
            runtime_proofs.push(runtime_proof);
            continue;
        }

        pipeline_record.artifact_emitted = true;
        pipeline_record.blocked_at_stage = None;
        pipeline_record.blocker_category = None;
        pipeline_record.blocker_component = None;
        pipeline_record.blocker_reason = None;
        pipeline_record.remaining_stages = artifact_pipeline_stage_records(None)
            .into_iter()
            .map(|mut stage| {
                stage.status = ArtifactPipelineStageStatus::Completed;
                stage.adapter_available = true;
                stage
            })
            .collect();
        for unit in &mut pipeline_record.compile_units {
            if unit.unit_id == unit_id {
                unit.codegen_handoff_status = Some("runtime_proof_passed".to_owned());
            }
        }
        for record in compiler_pipeline
            .iter_mut()
            .filter(|record| record.unit_id == unit_id)
        {
            record.status = RustCompilerPipelineStatus::Artifact;
            record.missing_stage = None;
            record.artifact = Some(RustCompileArtifactRecord {
                unit_id: record.unit_id.clone(),
                package: record.package.clone(),
                target: record.target.clone(),
                target_kind: record.target_kind.clone(),
                triple: record.triple.clone(),
                profile: record.profile.clone(),
                artifact_kind: RustCompileArtifactKind::CompilerUnitObject,
                path: artifact_path.clone(),
                sha256: written_hash.clone(),
            });
        }

        artifacts.push(ArtifactManifestEntry {
            target: target.name.clone(),
            target_triple: target.triple.clone(),
            path: artifact_path.clone(),
            artifact_kind: "Module".to_owned(),
            byte_length: written_bytes.len() as u64,
            sha256: written_hash.clone(),
            final_artifact_sha256: written_hash.clone(),
            producer_stage: "embedded_codegen_payload + embedded_wasm_ld".to_owned(),
            input_object_hash: input_object_hash.clone(),
            object_hash: input_object_hash,
            linker_payload_hash: linker_payload_hash.clone(),
            linker_payload_sha256: linker_payload_hash,
            compile_unit_id: unit_id.clone(),
            mir_hash: codegen_handoff.mir_body_hash.clone(),
            mir_source_hash: source_sha256.clone(),
            mono_graph_hash: codegen_handoff.mono_item_graph_hash.clone(),
            mono_source_hash: source_sha256.clone(),
            codegen_source_hash: source_sha256.clone(),
            source_path: codegen_handoff.source_path.clone(),
            source_sha256: source_sha256.clone(),
            codegen_input_sha256: module.codegen_input_sha256.clone(),
            codegen_input_source_sha256: module.codegen_input_source_sha256.clone(),
            codegen_input_source_bytes_sha256: module.codegen_input_source_bytes_sha256.clone(),
            codegen_input_source_origin: module.codegen_input_source_origin.clone(),
            codegen_input_source: module.codegen_input_source_text.clone(),
            runtime_proof_hash,
            fixture_name: contract.project.package.clone(),
            runtime_proof_status: format!("{:?}", runtime_proof.status),
        });
        hashes.push(HashEntry {
            label: format!("artifact:{}", target.name),
            path: artifact_path,
            sha256: written_hash,
        });
        interface_proofs.push(interface_proof);
        runtime_proofs.push(runtime_proof);
        promoted_compile_units.insert(unit_id);
        promoted_target_triples.insert(target.triple.clone());
    }

    Ok(ArtifactPromotionOutput {
        artifacts,
        interface_proofs,
        runtime_proofs,
        hashes,
        promoted_compile_units,
        promoted_target_triples,
    })
}

fn blocked_interface_proof(
    target: &rouwdi_contract::TargetContract,
    pipeline_record: &ArtifactPipelineRecord,
) -> ArtifactInterfaceProof {
    ArtifactInterfaceProof {
        target_name: target.name.clone(),
        triple: target.triple.clone(),
        artifact_kind: format!("{:?}", target.artifact),
        artifact_path: None,
        artifact_sha256: None,
        artifact_size_bytes: None,
        artifact_built: false,
        wasm_magic_valid: None,
        wasm_version_valid: None,
        exports: Vec::new(),
        imports: Vec::new(),
        start_export_present: None,
        required_exports: target.interface.required_exports.clone(),
        missing_exports: target.interface.required_exports.clone(),
        required_exports_satisfied: false,
        wasi_imports_classified: false,
        require_executable: target.interface.require_executable,
        executable_detected: None,
        status: ProofStatus::Failed,
        reason: pipeline_record.blocker_reason.clone().or_else(|| {
            Some(
                "artifact was not emitted because upstream compiler payload stages are not embedded"
                    .to_owned(),
            )
        }),
    }
}

fn blocked_runtime_proof(target: &rouwdi_contract::TargetContract) -> RuntimeProof {
    RuntimeProof {
        target_name: target.name.clone(),
        triple: target.triple.clone(),
        artifact_path: None,
        artifact_hash: None,
        runtime_used: None,
        required: target.runtime.required,
        kind: target.runtime.kind.map(|kind| format!("{kind:?}")),
        mode: format!("{:?}", target.runtime.mode),
        command_args: target.runtime.args.clone(),
        stdin: String::new(),
        stdout: String::new(),
        stderr: String::new(),
        executed: false,
        timeout_seconds: target.runtime.timeout_seconds,
        expected_exit_code: target.runtime.expected_exit_code.or(Some(0)),
        actual_exit_code: None,
        timed_out: None,
        stdout_contains: target.runtime.stdout_contains.clone(),
        stdout_matched: None,
        status: if target.runtime.required {
            ProofStatus::Failed
        } else {
            ProofStatus::Succeeded
        },
        reason: if target.runtime.required {
            Some("runtime proof cannot execute until the artifact exists".to_owned())
        } else {
            None
        },
    }
}

fn interface_proof_for_emitted_wasm(
    target: &rouwdi_contract::TargetContract,
    artifact_path: &str,
    storage: &dyn Storage,
) -> Result<ArtifactInterfaceProof, EngineError> {
    let bytes = storage.read(artifact_path)?;
    let sha256 = hash_bytes(&bytes);
    let wasm_magic_valid = bytes.len() >= 4 && &bytes[..4] == b"\0asm";
    let wasm_version_valid = bytes.len() >= 8 && &bytes[4..8] == b"\x01\0\0\0";
    let exports = parse_wasm_exports(&bytes)?;
    let imports = parse_wasm_imports(&bytes)?;
    let export_names = exports
        .iter()
        .map(|export| export.name.clone())
        .collect::<Vec<_>>();
    let import_names = imports
        .iter()
        .map(|import| format!("{}::{}", import.module, import.name))
        .collect::<Vec<_>>();
    let missing_exports = missing_wasm_exports(&exports, &target.interface.required_exports);
    let start_export_present = exports.iter().any(|export| export.name == "_start");
    let wasi_imports_classified = imports
        .iter()
        .all(|import| import.module == "wasi_snapshot_preview1" || import.module == "env");
    let executable_detected = start_export_present;
    let succeeded = wasm_magic_valid
        && wasm_version_valid
        && missing_exports.is_empty()
        && (!target.interface.require_executable || executable_detected)
        && start_export_present
        && wasi_imports_classified;

    Ok(ArtifactInterfaceProof {
        target_name: target.name.clone(),
        triple: target.triple.clone(),
        artifact_kind: format!("{:?}", target.artifact),
        artifact_path: Some(artifact_path.to_owned()),
        artifact_sha256: Some(sha256),
        artifact_size_bytes: Some(bytes.len() as u64),
        artifact_built: true,
        wasm_magic_valid: Some(wasm_magic_valid),
        wasm_version_valid: Some(wasm_version_valid),
        exports: export_names,
        imports: import_names,
        start_export_present: Some(start_export_present),
        required_exports: target.interface.required_exports.clone(),
        missing_exports,
        required_exports_satisfied: succeeded,
        wasi_imports_classified,
        require_executable: target.interface.require_executable,
        executable_detected: Some(executable_detected),
        status: if succeeded {
            ProofStatus::Succeeded
        } else {
            ProofStatus::Failed
        },
        reason: (!succeeded).then(|| {
            "emitted WebAssembly module failed interface validation against the artifact bytes"
                .to_owned()
        }),
    })
}

fn runtime_proof_for_emitted_wasm(
    target: &rouwdi_contract::TargetContract,
    artifact_path: &str,
    bytes: &[u8],
    artifact_hash: &str,
) -> Result<RuntimeProof, EngineError> {
    if !target.runtime.required {
        return Ok(RuntimeProof {
            target_name: target.name.clone(),
            triple: target.triple.clone(),
            artifact_path: Some(artifact_path.to_owned()),
            artifact_hash: Some(artifact_hash.to_owned()),
            runtime_used: None,
            required: false,
            kind: target.runtime.kind.map(|kind| format!("{kind:?}")),
            mode: format!("{:?}", target.runtime.mode),
            command_args: target.runtime.args.clone(),
            stdin: String::new(),
            stdout: String::new(),
            stderr: String::new(),
            executed: false,
            timeout_seconds: target.runtime.timeout_seconds,
            expected_exit_code: target.runtime.expected_exit_code.or(Some(0)),
            actual_exit_code: None,
            timed_out: Some(false),
            stdout_contains: target.runtime.stdout_contains.clone(),
            stdout_matched: target.runtime.stdout_contains.as_ref().map(|_| false),
            status: ProofStatus::Succeeded,
            reason: None,
        });
    }
    if target.runtime.kind != Some(RuntimeKind::Wasi) {
        return Ok(RuntimeProof {
            target_name: target.name.clone(),
            triple: target.triple.clone(),
            artifact_path: Some(artifact_path.to_owned()),
            artifact_hash: Some(artifact_hash.to_owned()),
            runtime_used: None,
            required: true,
            kind: target.runtime.kind.map(|kind| format!("{kind:?}")),
            mode: format!("{:?}", target.runtime.mode),
            command_args: target.runtime.args.clone(),
            stdin: String::new(),
            stdout: String::new(),
            stderr: String::new(),
            executed: false,
            timeout_seconds: target.runtime.timeout_seconds,
            expected_exit_code: target.runtime.expected_exit_code.or(Some(0)),
            actual_exit_code: None,
            timed_out: Some(false),
            stdout_contains: target.runtime.stdout_contains.clone(),
            stdout_matched: None,
            status: ProofStatus::Unsupported,
            reason: Some("runtime proof is only implemented for local WASI artifacts".to_owned()),
        });
    }

    let expected_exit_code = target.runtime.expected_exit_code.unwrap_or(0);
    let runtime = execute_wasi_module(bytes, &target.runtime.args, target.runtime.timeout_seconds);
    let stdout_matched = target
        .runtime
        .stdout_contains
        .as_ref()
        .map(|needle| runtime.stdout.contains(needle));
    let passed = runtime.exit_code == Some(expected_exit_code)
        && !runtime.timed_out
        && runtime.error.is_none()
        && stdout_matched.unwrap_or(true);
    Ok(RuntimeProof {
        target_name: target.name.clone(),
        triple: target.triple.clone(),
        artifact_path: Some(artifact_path.to_owned()),
        artifact_hash: Some(artifact_hash.to_owned()),
        runtime_used: Some("rouwdi-engine wasmi wasi_snapshot_preview1 substrate".to_owned()),
        required: true,
        kind: target.runtime.kind.map(|kind| format!("{kind:?}")),
        mode: format!("{:?}", target.runtime.mode),
        command_args: target.runtime.args.clone(),
        stdin: String::new(),
        stdout: runtime.stdout,
        stderr: runtime.stderr,
        executed: runtime.executed,
        timeout_seconds: target.runtime.timeout_seconds,
        expected_exit_code: Some(expected_exit_code),
        actual_exit_code: runtime.exit_code,
        timed_out: Some(runtime.timed_out),
        stdout_contains: target.runtime.stdout_contains.clone(),
        stdout_matched,
        status: if passed {
            ProofStatus::Succeeded
        } else {
            ProofStatus::Failed
        },
        reason: if passed {
            None
        } else {
            Some(runtime.error.unwrap_or_else(|| {
                "emitted WASI module runtime result did not match the contract".to_owned()
            }))
        },
    })
}

struct WasiExecutionResult {
    executed: bool,
    stdout: String,
    stderr: String,
    exit_code: Option<i32>,
    timed_out: bool,
    error: Option<String>,
}

#[derive(Default)]
struct EngineWasiState {
    args: Vec<String>,
    env: Vec<String>,
    stdout: Vec<u8>,
    stderr: Vec<u8>,
    proc_exit_code: Option<i32>,
    random_counter: u8,
}

const WASI: &str = "wasi_snapshot_preview1";
const WASI_ERRNO_SUCCESS: i32 = 0;
const WASI_ERRNO_BADF: i32 = 8;
const WASI_ERRNO_INVAL: i32 = 28;
const WASI_ERRNO_NOENT: i32 = 44;
const WASI_ERRNO_NOSYS: i32 = 52;
const WASI_FILETYPE_CHARACTER_DEVICE: u8 = 2;
const WASI_FILETYPE_DIRECTORY: u8 = 3;
const WASI_PREOPEN_FD: i32 = 3;
const WASI_PREOPEN_PATH: &str = "/";

fn execute_wasi_module(
    bytes: &[u8],
    args: &[String],
    _timeout_seconds: u64,
) -> WasiExecutionResult {
    let mut config = Config::default();
    config.consume_fuel(true);
    config.set_max_recursion_depth(4096);
    config.ignore_custom_sections(true);
    let engine = Engine::new(&config);
    let module = match Module::new(&engine, bytes) {
        Ok(module) => module,
        Err(error) => {
            return WasiExecutionResult {
                executed: false,
                stdout: String::new(),
                stderr: String::new(),
                exit_code: None,
                timed_out: false,
                error: Some(format!("module compile failed: {error}")),
            };
        }
    };
    let mut linker = Linker::<EngineWasiState>::new(&engine);
    if let Err(error) = define_engine_wasi_imports(&mut linker) {
        return WasiExecutionResult {
            executed: false,
            stdout: String::new(),
            stderr: String::new(),
            exit_code: None,
            timed_out: false,
            error: Some(error),
        };
    }
    let mut store = Store::new(
        &engine,
        EngineWasiState {
            args: std::iter::once("artifact.wasm".to_owned())
                .chain(args.iter().cloned())
                .collect(),
            env: vec!["PWD=/".to_owned()],
            ..EngineWasiState::default()
        },
    );
    if let Err(error) = store.set_fuel(1_000_000_000) {
        return WasiExecutionResult {
            executed: false,
            stdout: String::new(),
            stderr: String::new(),
            exit_code: None,
            timed_out: false,
            error: Some(format!("fuel setup failed: {error}")),
        };
    }
    let instance = match linker.instantiate_and_start(&mut store, &module) {
        Ok(instance) => instance,
        Err(error) => {
            let stdout = String::from_utf8_lossy(&store.data().stdout).into_owned();
            let stderr = String::from_utf8_lossy(&store.data().stderr).into_owned();
            return WasiExecutionResult {
                executed: false,
                stdout,
                stderr,
                exit_code: store.data().proc_exit_code,
                timed_out: false,
                error: Some(format!("module instantiate failed: {error}")),
            };
        }
    };
    let start = match instance.get_typed_func::<(), ()>(&store, "_start") {
        Ok(start) => start,
        Err(error) => {
            return WasiExecutionResult {
                executed: false,
                stdout: String::from_utf8_lossy(&store.data().stdout).into_owned(),
                stderr: String::from_utf8_lossy(&store.data().stderr).into_owned(),
                exit_code: store.data().proc_exit_code,
                timed_out: false,
                error: Some(format!("_start export missing or invalid: {error}")),
            };
        }
    };
    let call_result = start.call(&mut store, ());
    let stdout = String::from_utf8_lossy(&store.data().stdout).into_owned();
    let stderr = String::from_utf8_lossy(&store.data().stderr).into_owned();
    match call_result {
        Ok(()) => WasiExecutionResult {
            executed: true,
            stdout,
            stderr,
            exit_code: Some(store.data().proc_exit_code.unwrap_or(0)),
            timed_out: false,
            error: None,
        },
        Err(error) => {
            let fuel_exhausted = error.to_string().to_ascii_lowercase().contains("fuel");
            let proc_exit_code = store.data().proc_exit_code;
            WasiExecutionResult {
                executed: proc_exit_code.is_some(),
                stdout,
                stderr,
                exit_code: proc_exit_code,
                timed_out: fuel_exhausted,
                error: proc_exit_code
                    .is_none()
                    .then(|| format!("runtime trap: {error}")),
            }
        }
    }
}

fn define_engine_wasi_imports(linker: &mut Linker<EngineWasiState>) -> Result<(), String> {
    linker
        .func_wrap(WASI, "args_sizes_get", wasi_args_sizes_get)
        .map_err(|error| error.to_string())?;
    linker
        .func_wrap(WASI, "args_get", wasi_args_get)
        .map_err(|error| error.to_string())?;
    linker
        .func_wrap(WASI, "environ_sizes_get", wasi_environ_sizes_get)
        .map_err(|error| error.to_string())?;
    linker
        .func_wrap(WASI, "environ_get", wasi_environ_get)
        .map_err(|error| error.to_string())?;
    linker
        .func_wrap(WASI, "clock_time_get", wasi_clock_time_get)
        .map_err(|error| error.to_string())?;
    linker
        .func_wrap(WASI, "random_get", wasi_random_get)
        .map_err(|error| error.to_string())?;
    linker
        .func_wrap(WASI, "poll_oneoff", wasi_poll_oneoff)
        .map_err(|error| error.to_string())?;
    linker
        .func_wrap(WASI, "fd_write", wasi_fd_write)
        .map_err(|error| error.to_string())?;
    linker
        .func_wrap(WASI, "fd_read", wasi_fd_read)
        .map_err(|error| error.to_string())?;
    linker
        .func_wrap(WASI, "fd_pread", wasi_fd_pread)
        .map_err(|error| error.to_string())?;
    linker
        .func_wrap(WASI, "fd_close", wasi_fd_close)
        .map_err(|error| error.to_string())?;
    linker
        .func_wrap(WASI, "fd_fdstat_get", wasi_fd_fdstat_get)
        .map_err(|error| error.to_string())?;
    linker
        .func_wrap(WASI, "fd_fdstat_set_flags", wasi_fd_fdstat_set_flags)
        .map_err(|error| error.to_string())?;
    linker
        .func_wrap(WASI, "fd_filestat_get", wasi_fd_filestat_get)
        .map_err(|error| error.to_string())?;
    linker
        .func_wrap(WASI, "fd_filestat_set_size", wasi_fd_filestat_set_size)
        .map_err(|error| error.to_string())?;
    linker
        .func_wrap(WASI, "fd_prestat_get", wasi_fd_prestat_get)
        .map_err(|error| error.to_string())?;
    linker
        .func_wrap(WASI, "fd_prestat_dir_name", wasi_fd_prestat_dir_name)
        .map_err(|error| error.to_string())?;
    linker
        .func_wrap(WASI, "fd_readdir", wasi_fd_readdir)
        .map_err(|error| error.to_string())?;
    linker
        .func_wrap(WASI, "fd_seek", wasi_fd_seek)
        .map_err(|error| error.to_string())?;
    linker
        .func_wrap(WASI, "path_create_directory", wasi_path_create_directory)
        .map_err(|error| error.to_string())?;
    linker
        .func_wrap(WASI, "path_filestat_get", wasi_path_filestat_get)
        .map_err(|error| error.to_string())?;
    linker
        .func_wrap(WASI, "path_link", wasi_path_link)
        .map_err(|error| error.to_string())?;
    linker
        .func_wrap(WASI, "path_open", wasi_path_open)
        .map_err(|error| error.to_string())?;
    linker
        .func_wrap(WASI, "path_readlink", wasi_path_readlink)
        .map_err(|error| error.to_string())?;
    linker
        .func_wrap(WASI, "path_remove_directory", wasi_path_remove_directory)
        .map_err(|error| error.to_string())?;
    linker
        .func_wrap(WASI, "path_rename", wasi_path_rename)
        .map_err(|error| error.to_string())?;
    linker
        .func_wrap(WASI, "path_unlink_file", wasi_path_unlink_file)
        .map_err(|error| error.to_string())?;
    linker
        .func_wrap(WASI, "proc_exit", wasi_proc_exit)
        .map_err(|error| error.to_string())?;
    linker
        .func_wrap(WASI, "sched_yield", wasi_sched_yield)
        .map_err(|error| error.to_string())?;
    Ok(())
}

fn wasi_args_sizes_get(
    mut caller: Caller<'_, EngineWasiState>,
    argc_ptr: i32,
    argv_buf_size_ptr: i32,
) -> i32 {
    let args = caller.data().args.clone();
    let status = write_u32(&mut caller, argc_ptr, args.len() as u32);
    if status != WASI_ERRNO_SUCCESS {
        return status;
    }
    let argv_buf_size = args.iter().map(|arg| arg.len() + 1).sum::<usize>();
    write_u32(&mut caller, argv_buf_size_ptr, argv_buf_size as u32)
}

fn wasi_args_get(mut caller: Caller<'_, EngineWasiState>, argv_ptr: i32, argv_buf_ptr: i32) -> i32 {
    let args = caller.data().args.clone();
    write_string_vector(&mut caller, args, argv_ptr, argv_buf_ptr)
}

fn wasi_environ_sizes_get(
    mut caller: Caller<'_, EngineWasiState>,
    count_ptr: i32,
    size_ptr: i32,
) -> i32 {
    let env = caller.data().env.clone();
    let status = write_u32(&mut caller, count_ptr, env.len() as u32);
    if status != WASI_ERRNO_SUCCESS {
        return status;
    }
    let env_buf_size = env.iter().map(|entry| entry.len() + 1).sum::<usize>();
    write_u32(&mut caller, size_ptr, env_buf_size as u32)
}

fn wasi_environ_get(
    mut caller: Caller<'_, EngineWasiState>,
    env_ptr: i32,
    env_buf_ptr: i32,
) -> i32 {
    let env = caller.data().env.clone();
    write_string_vector(&mut caller, env, env_ptr, env_buf_ptr)
}

fn write_string_vector(
    caller: &mut Caller<'_, EngineWasiState>,
    values: Vec<String>,
    ptrs: i32,
    buf: i32,
) -> i32 {
    let mut current_buf_ptr = buf;
    for (index, value) in values.iter().enumerate() {
        let pointer_slot = ptrs + (index as i32 * 4);
        let status = write_u32(caller, pointer_slot, current_buf_ptr as u32);
        if status != WASI_ERRNO_SUCCESS {
            return status;
        }
        let status = write_bytes(caller, current_buf_ptr, value.as_bytes());
        if status != WASI_ERRNO_SUCCESS {
            return status;
        }
        let terminator_ptr = current_buf_ptr + value.len() as i32;
        let status = write_bytes(caller, terminator_ptr, &[0]);
        if status != WASI_ERRNO_SUCCESS {
            return status;
        }
        current_buf_ptr = terminator_ptr + 1;
    }
    WASI_ERRNO_SUCCESS
}

fn wasi_clock_time_get(
    mut caller: Caller<'_, EngineWasiState>,
    _clock_id: i32,
    _precision: i64,
    time_ptr: i32,
) -> i32 {
    write_u64(&mut caller, time_ptr, 0)
}

fn wasi_random_get(mut caller: Caller<'_, EngineWasiState>, ptr: i32, len: i32) -> i32 {
    if ptr < 0 || len < 0 {
        return WASI_ERRNO_INVAL;
    }
    let mut bytes = vec![0_u8; len as usize];
    for byte in &mut bytes {
        let next = caller.data().random_counter.wrapping_add(1);
        caller.data_mut().random_counter = next;
        *byte = next;
    }
    write_bytes(&mut caller, ptr, &bytes)
}

fn wasi_poll_oneoff(
    mut caller: Caller<'_, EngineWasiState>,
    subscriptions_ptr: i32,
    events_ptr: i32,
    subscriptions_len: i32,
    events_len_ptr: i32,
) -> i32 {
    if subscriptions_ptr < 0 || events_ptr < 0 || subscriptions_len <= 0 {
        return WASI_ERRNO_INVAL;
    }
    let Some(memory) = caller_memory(&caller) else {
        return WASI_ERRNO_INVAL;
    };
    let mut userdata = [0_u8; 8];
    if memory
        .read(&caller, subscriptions_ptr as usize, &mut userdata)
        .is_err()
    {
        return WASI_ERRNO_INVAL;
    }
    let mut event = [0_u8; 32];
    event[0..8].copy_from_slice(&userdata);
    let status = write_bytes(&mut caller, events_ptr, &event);
    if status != WASI_ERRNO_SUCCESS {
        return status;
    }
    write_u32(&mut caller, events_len_ptr, 1)
}

fn wasi_fd_write(
    mut caller: Caller<'_, EngineWasiState>,
    fd: i32,
    iovs_ptr: i32,
    iovs_len: i32,
    nwritten_ptr: i32,
) -> i32 {
    if iovs_ptr < 0 || iovs_len < 0 {
        return WASI_ERRNO_INVAL;
    }
    let Some(memory) = caller_memory(&caller) else {
        return WASI_ERRNO_INVAL;
    };
    let mut written = 0_u32;
    let mut chunks = Vec::new();
    for index in 0..iovs_len {
        let base = iovs_ptr as usize + (index as usize * 8);
        let Ok(ptr) = read_memory_u32(&memory, &caller, base) else {
            return WASI_ERRNO_INVAL;
        };
        let Ok(len) = read_memory_u32(&memory, &caller, base + 4) else {
            return WASI_ERRNO_INVAL;
        };
        let mut bytes = vec![0_u8; len as usize];
        if memory.read(&caller, ptr as usize, &mut bytes).is_err() {
            return WASI_ERRNO_INVAL;
        }
        written = written.saturating_add(len);
        chunks.extend_from_slice(&bytes);
    }
    match fd {
        1 => caller.data_mut().stdout.extend_from_slice(&chunks),
        2 => caller.data_mut().stderr.extend_from_slice(&chunks),
        _ => return WASI_ERRNO_BADF,
    }
    write_u32(&mut caller, nwritten_ptr, written)
}

fn wasi_fd_read(
    mut caller: Caller<'_, EngineWasiState>,
    _fd: i32,
    _iovs_ptr: i32,
    _iovs_len: i32,
    nread_ptr: i32,
) -> i32 {
    write_u32(&mut caller, nread_ptr, 0)
}

fn wasi_fd_pread(
    mut caller: Caller<'_, EngineWasiState>,
    _fd: i32,
    _iovs_ptr: i32,
    _iovs_len: i32,
    _offset: i64,
    nread_ptr: i32,
) -> i32 {
    write_u32(&mut caller, nread_ptr, 0)
}

fn wasi_fd_close(_caller: Caller<'_, EngineWasiState>, fd: i32) -> i32 {
    if fd == WASI_PREOPEN_FD {
        WASI_ERRNO_SUCCESS
    } else {
        WASI_ERRNO_BADF
    }
}

fn wasi_fd_fdstat_get(mut caller: Caller<'_, EngineWasiState>, fd: i32, stat_ptr: i32) -> i32 {
    let filetype = match fd {
        0..=2 => WASI_FILETYPE_CHARACTER_DEVICE,
        WASI_PREOPEN_FD => WASI_FILETYPE_DIRECTORY,
        _ => return WASI_ERRNO_BADF,
    };
    let mut stat = [0_u8; 24];
    stat[0] = filetype;
    stat[8..16].copy_from_slice(&u64::MAX.to_le_bytes());
    stat[16..24].copy_from_slice(&u64::MAX.to_le_bytes());
    write_bytes(&mut caller, stat_ptr, &stat)
}

fn wasi_fd_fdstat_set_flags(_caller: Caller<'_, EngineWasiState>, fd: i32, _flags: i32) -> i32 {
    if matches!(fd, 0..=3) {
        WASI_ERRNO_SUCCESS
    } else {
        WASI_ERRNO_BADF
    }
}

fn wasi_fd_filestat_get(mut caller: Caller<'_, EngineWasiState>, fd: i32, stat_ptr: i32) -> i32 {
    let filetype = match fd {
        0..=2 => WASI_FILETYPE_CHARACTER_DEVICE,
        WASI_PREOPEN_FD => WASI_FILETYPE_DIRECTORY,
        _ => return WASI_ERRNO_BADF,
    };
    write_filestat(&mut caller, stat_ptr, filetype, 0)
}

fn wasi_fd_filestat_set_size(_caller: Caller<'_, EngineWasiState>, _fd: i32, _size: i64) -> i32 {
    WASI_ERRNO_BADF
}

fn wasi_fd_prestat_get(mut caller: Caller<'_, EngineWasiState>, fd: i32, prestat_ptr: i32) -> i32 {
    if fd != WASI_PREOPEN_FD {
        return WASI_ERRNO_BADF;
    }
    let mut prestat = [0_u8; 8];
    prestat[4..8].copy_from_slice(&(WASI_PREOPEN_PATH.len() as u32).to_le_bytes());
    write_bytes(&mut caller, prestat_ptr, &prestat)
}

fn wasi_fd_prestat_dir_name(
    mut caller: Caller<'_, EngineWasiState>,
    fd: i32,
    path_ptr: i32,
    path_len: i32,
) -> i32 {
    if fd != WASI_PREOPEN_FD || path_len < 0 {
        return WASI_ERRNO_BADF;
    }
    let bytes = WASI_PREOPEN_PATH.as_bytes();
    if path_len as usize > bytes.len() {
        return WASI_ERRNO_INVAL;
    }
    write_bytes(&mut caller, path_ptr, &bytes[..path_len as usize])
}

fn wasi_fd_readdir(
    mut caller: Caller<'_, EngineWasiState>,
    fd: i32,
    _buf: i32,
    _buf_len: i32,
    _cookie: i64,
    bufused_ptr: i32,
) -> i32 {
    if fd != WASI_PREOPEN_FD {
        return WASI_ERRNO_BADF;
    }
    write_u32(&mut caller, bufused_ptr, 0)
}

fn wasi_fd_seek(
    mut caller: Caller<'_, EngineWasiState>,
    fd: i32,
    _offset: i64,
    _whence: i32,
    newoffset_ptr: i32,
) -> i32 {
    if fd != WASI_PREOPEN_FD {
        return WASI_ERRNO_BADF;
    }
    write_u64(&mut caller, newoffset_ptr, 0)
}

fn wasi_path_create_directory(
    _caller: Caller<'_, EngineWasiState>,
    fd: i32,
    _path_ptr: i32,
    _path_len: i32,
) -> i32 {
    if fd == WASI_PREOPEN_FD {
        WASI_ERRNO_SUCCESS
    } else {
        WASI_ERRNO_BADF
    }
}

fn wasi_path_filestat_get(
    mut caller: Caller<'_, EngineWasiState>,
    fd: i32,
    _flags: i32,
    _path_ptr: i32,
    _path_len: i32,
    stat_ptr: i32,
) -> i32 {
    if fd != WASI_PREOPEN_FD {
        return WASI_ERRNO_BADF;
    }
    write_filestat(&mut caller, stat_ptr, WASI_FILETYPE_DIRECTORY, 0)
}

fn wasi_path_link(
    _caller: Caller<'_, EngineWasiState>,
    _old_fd: i32,
    _old_flags: i32,
    _old_path_ptr: i32,
    _old_path_len: i32,
    _new_fd: i32,
    _new_path_ptr: i32,
    _new_path_len: i32,
) -> i32 {
    WASI_ERRNO_NOSYS
}

fn wasi_path_open(
    mut caller: Caller<'_, EngineWasiState>,
    fd: i32,
    _dirflags: i32,
    _path_ptr: i32,
    _path_len: i32,
    _oflags: i32,
    _rights_base: i64,
    _rights_inheriting: i64,
    _fdflags: i32,
    opened_fd_ptr: i32,
) -> i32 {
    let _ = write_u32(&mut caller, opened_fd_ptr, 0);
    if fd == WASI_PREOPEN_FD {
        WASI_ERRNO_NOENT
    } else {
        WASI_ERRNO_BADF
    }
}

fn wasi_path_readlink(
    mut caller: Caller<'_, EngineWasiState>,
    _fd: i32,
    _path_ptr: i32,
    _path_len: i32,
    _buf: i32,
    _buf_len: i32,
    bufused_ptr: i32,
) -> i32 {
    let _ = write_u32(&mut caller, bufused_ptr, 0);
    WASI_ERRNO_NOENT
}

fn wasi_path_remove_directory(
    _caller: Caller<'_, EngineWasiState>,
    fd: i32,
    _path_ptr: i32,
    _path_len: i32,
) -> i32 {
    if fd == WASI_PREOPEN_FD {
        WASI_ERRNO_SUCCESS
    } else {
        WASI_ERRNO_BADF
    }
}

fn wasi_path_rename(
    _caller: Caller<'_, EngineWasiState>,
    _fd: i32,
    _path_ptr: i32,
    _path_len: i32,
    _new_fd: i32,
    _new_path_ptr: i32,
    _new_path_len: i32,
) -> i32 {
    WASI_ERRNO_NOSYS
}

fn wasi_path_unlink_file(
    _caller: Caller<'_, EngineWasiState>,
    fd: i32,
    _path_ptr: i32,
    _path_len: i32,
) -> i32 {
    if fd == WASI_PREOPEN_FD {
        WASI_ERRNO_SUCCESS
    } else {
        WASI_ERRNO_BADF
    }
}

fn wasi_proc_exit(mut caller: Caller<'_, EngineWasiState>, code: i32) {
    caller.data_mut().proc_exit_code = Some(code);
}

fn wasi_sched_yield() -> i32 {
    WASI_ERRNO_SUCCESS
}

fn caller_memory(caller: &Caller<'_, EngineWasiState>) -> Option<Memory> {
    caller.get_export("memory").and_then(|item| match item {
        Extern::Memory(memory) => Some(memory),
        _ => None,
    })
}

fn write_u32(caller: &mut Caller<'_, EngineWasiState>, ptr: i32, value: u32) -> i32 {
    write_bytes(caller, ptr, &value.to_le_bytes())
}

fn write_u64(caller: &mut Caller<'_, EngineWasiState>, ptr: i32, value: u64) -> i32 {
    write_bytes(caller, ptr, &value.to_le_bytes())
}

fn write_bytes(caller: &mut Caller<'_, EngineWasiState>, ptr: i32, bytes: &[u8]) -> i32 {
    if ptr < 0 {
        return WASI_ERRNO_INVAL;
    }
    let Some(memory) = caller_memory(caller) else {
        return WASI_ERRNO_INVAL;
    };
    if memory.write(caller, ptr as usize, bytes).is_err() {
        WASI_ERRNO_INVAL
    } else {
        WASI_ERRNO_SUCCESS
    }
}

fn write_filestat(
    caller: &mut Caller<'_, EngineWasiState>,
    ptr: i32,
    filetype: u8,
    size: u64,
) -> i32 {
    let mut stat = [0_u8; 64];
    stat[16] = filetype;
    stat[32..40].copy_from_slice(&size.to_le_bytes());
    write_bytes(caller, ptr, &stat)
}

fn read_memory_u32(
    memory: &Memory,
    caller: &Caller<'_, EngineWasiState>,
    offset: usize,
) -> Result<u32, ()> {
    let mut bytes = [0_u8; 4];
    memory.read(caller, offset, &mut bytes).map_err(|_| ())?;
    Ok(u32::from_le_bytes(bytes))
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
    embedded_mir_payload_execution: Option<&RustEmbeddedMirPayloadExecution>,
    embedded_mir_payload_execution_provider: Option<EmbeddedMirPayloadExecutionProvider>,
    contract_sha256: &str,
    source_snapshot_sha256: &str,
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
        let source_sha256 = hash_bytes(&bytes);
        let request = RustCompileRequest {
            unit_id: unit.id.clone(),
            package: unit.package.clone(),
            target: unit.target.clone(),
            target_kind: format!("{:?}", unit.target_kind),
            source_path: source_path.clone(),
            triple: unit.triple.clone(),
            profile: unit.profile.clone(),
            extern_prelude: extern_prelude_for_unit(build_plan, &unit.id),
        };
        let dynamic_mir_payload_execution =
            embedded_mir_payload_execution_provider.and_then(|provider| {
                provider(&EmbeddedMirPayloadExecutionRequest {
                    compile_unit_id: unit.id.clone(),
                    package: unit.package.clone(),
                    target: unit.target.clone(),
                    target_kind: format!("{:?}", unit.target_kind),
                    source_path: source_path.clone(),
                    source_text: source.to_string(),
                    source_sha256,
                    source_snapshot_sha256: source_snapshot_sha256.to_owned(),
                    contract_sha256: contract_sha256.to_owned(),
                    target_triple: unit.triple.clone(),
                    profile: unit.profile.clone(),
                })
            });
        let mir_execution = dynamic_mir_payload_execution
            .as_ref()
            .or(embedded_mir_payload_execution);
        records.push(run_rust_compiler_pipeline_record_with_embedded_mir_payload(
            &request,
            &source,
            mir_execution,
        ));
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
            let missing_stage = if mir_blocker.is_none() {
                compile_records
                    .iter()
                    .find_map(|record| record.missing_stage.as_ref())
            } else {
                None
            };
            let compile_units = compile_records
                .iter()
                .map(|record| artifact_compile_unit_from_record(record))
                .collect::<Vec<_>>();
            let blocked_at_stage = mir_blocker
                .map(|handoff| handoff.stage)
                .or_else(|| missing_stage.map(|missing| missing.stage));
            let remaining_stages = artifact_pipeline_stage_records(blocked_at_stage);
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
                blocked_at_stage,
                blocker_category: mir_blocker.and_then(|handoff| handoff.blocker_category),
                blocker_component: mir_blocker
                    .and_then(|handoff| handoff.blocker_component.clone())
                    .or_else(|| missing_stage.map(|missing| missing.required_component.clone())),
                blocker_reason: mir_blocker
                    .and_then(|handoff| handoff.blocker_reason.clone())
                    .or_else(|| missing_stage.map(|missing| missing.reason.clone())),
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
        mir_body_identity: record
            .mir_handoff
            .as_ref()
            .and_then(|handoff| handoff.mir_body_proof.as_ref())
            .map(|proof| proof.mir_body_identity.clone()),
        mir_body_hash: record
            .mir_handoff
            .as_ref()
            .and_then(|handoff| handoff.mir_body_proof.as_ref())
            .map(|proof| proof.mir_body_hash.clone()),
        monomorphization_handoff_status: record
            .mir_handoff
            .as_ref()
            .and_then(|handoff| handoff.monomorphization_handoff.as_ref())
            .map(|handoff| handoff.mono_item_collection_status.clone()),
        mono_item_count: record
            .mir_handoff
            .as_ref()
            .and_then(|handoff| handoff.monomorphization_proof.as_ref())
            .map(|proof| proof.mono_item_count),
        mono_item_graph_hash: record
            .mir_handoff
            .as_ref()
            .and_then(|handoff| handoff.monomorphization_proof.as_ref())
            .and_then(|proof| proof.mono_item_graph_hash.clone()),
        codegen_handoff_status: record
            .mir_handoff
            .as_ref()
            .and_then(|handoff| handoff.codegen_handoff.as_ref())
            .map(|handoff| handoff.current_status.clone()),
    }
}

fn artifact_pipeline_stage_records(
    blocked_at_stage: Option<RustCompilerStage>,
) -> Vec<ArtifactPipelineStageRecord> {
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
        let status = artifact_stage_status(stage, blocked_at_stage);
        ArtifactPipelineStageRecord {
            stage,
            required_component: required_component.to_owned(),
            component_role: component_role.to_owned(),
            adapter_available: status == ArtifactPipelineStageStatus::Completed,
            status,
        }
    })
    .collect()
}

fn artifact_stage_status(
    stage: RustCompilerStage,
    blocked_at_stage: Option<RustCompilerStage>,
) -> ArtifactPipelineStageStatus {
    let Some(blocked_at_stage) = blocked_at_stage else {
        return ArtifactPipelineStageStatus::Planned;
    };
    if stage == blocked_at_stage {
        return ArtifactPipelineStageStatus::Blocked;
    }
    if compiler_stage_order(stage) < compiler_stage_order(blocked_at_stage) {
        return ArtifactPipelineStageStatus::Completed;
    }
    if blocked_at_stage == RustCompilerStage::Mir {
        ArtifactPipelineStageStatus::WaitingOnUpstreamMir
    } else {
        ArtifactPipelineStageStatus::Planned
    }
}

fn compiler_stage_order(stage: RustCompilerStage) -> u8 {
    match stage {
        RustCompilerStage::Parse => 0,
        RustCompilerStage::MacroExpansion => 1,
        RustCompilerStage::NameResolution => 2,
        RustCompilerStage::TypeChecking => 3,
        RustCompilerStage::BorrowChecking => 4,
        RustCompilerStage::Mir => 5,
        RustCompilerStage::Monomorphization => 6,
        RustCompilerStage::Codegen => 7,
        RustCompilerStage::Linking => 8,
        RustCompilerStage::ArtifactEmission => 9,
    }
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
            Some("none")
        );
        assert_eq!(
            manifest_handoff.payload_milestone_state.as_deref(),
            Some("bridge_wasm_mir_payload_module_emitted")
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
        assert_eq!(bridge_attempt.status, "mono_items_collected");
        assert_eq!(bridge_attempt.blocker_kind, "none");
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
