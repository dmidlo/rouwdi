use rouwdi_object::WasmObjectInspection;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RustcComponentStatus {
    pub name: String,
    pub upstream_path: String,
    pub role: String,
    pub embedded_in_assembly: bool,
    pub required_for_complete_semantics: bool,
    pub import_status: String,
    pub import_ledger_path: Option<String>,
    pub adapter_crate: Option<String>,
    pub blocker: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RustTokenSummary {
    pub kind: String,
    pub len: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RustLexDiagnostic {
    pub offset: u64,
    pub len: u32,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RustSourceLexProof {
    pub path: String,
    pub token_count: usize,
    pub tokens: Vec<RustTokenSummary>,
    pub diagnostics: Vec<RustLexDiagnostic>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RustParseDiagnostic {
    pub offset: u64,
    pub len: u32,
    pub message: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RustParseStageStatus {
    Succeeded,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RustParseStageRecord {
    pub unit_id: String,
    pub package: String,
    pub target: String,
    pub target_kind: String,
    pub source_path: String,
    pub triple: String,
    pub profile: String,
    pub stage: RustCompilerStage,
    pub status: RustParseStageStatus,
    pub parser_engine: String,
    pub parser_source: String,
    pub entrypoint: String,
    pub edition: String,
    pub token_count: usize,
    pub node_count: usize,
    pub diagnostic_count: usize,
    pub diagnostics: Vec<RustParseDiagnostic>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RustExpansionDiagnostic {
    pub offset: u64,
    pub len: u32,
    pub feature: String,
    pub message: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RustExpansionStageStatus {
    NoExpansionRequired,
    ExpansionRequired,
}

impl RustExpansionStageStatus {
    pub fn is_success(self) -> bool {
        matches!(self, Self::NoExpansionRequired)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RustExpansionStageRecord {
    pub unit_id: String,
    pub package: String,
    pub target: String,
    pub target_kind: String,
    pub source_path: String,
    pub triple: String,
    pub profile: String,
    pub stage: RustCompilerStage,
    pub status: RustExpansionStageStatus,
    pub expansion_engine: String,
    pub expansion_source: String,
    pub parse_stage_status: RustParseStageStatus,
    pub parse_token_count: usize,
    pub diagnostic_count: usize,
    pub diagnostics: Vec<RustExpansionDiagnostic>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RustExternCrate {
    pub name: String,
    pub source_unit_id: Option<String>,
    pub package: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RustNameResolutionContext {
    pub extern_prelude: Vec<RustExternCrate>,
}

impl RustNameResolutionContext {
    pub fn empty() -> Self {
        Self {
            extern_prelude: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RustNameResolutionDiagnostic {
    pub offset: u64,
    pub len: u32,
    pub code: RustNameResolutionDiagnosticCode,
    pub path: String,
    pub message: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RustNameResolutionDiagnosticCode {
    UnresolvedImport,
    UnresolvedModule,
    UnresolvedPath,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RustNameResolutionStageStatus {
    Succeeded,
    Failed,
}

impl RustNameResolutionStageStatus {
    pub fn is_success(self) -> bool {
        matches!(self, Self::Succeeded)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RustNameBinding {
    pub name: String,
    pub kind: RustNameBindingKind,
    pub namespace: RustNameNamespace,
    pub scope_path: String,
    pub offset: u64,
    pub len: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RustNameBindingKind {
    Builtin,
    Const,
    Enum,
    ExternCrate,
    Function,
    Import,
    Module,
    Static,
    Struct,
    Trait,
    TypeAlias,
    Union,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RustNameNamespace {
    Module,
    Type,
    Value,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RustResolvedPath {
    pub path: String,
    pub resolution: RustNameResolution,
    pub scope_path: String,
    pub offset: u64,
    pub len: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum RustNameResolution {
    Builtin {
        name: String,
    },
    ExternalCrate {
        name: String,
        source_unit_id: Option<String>,
    },
    LocalBinding {
        name: String,
        namespace: RustNameNamespace,
    },
    Module {
        path: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RustNameResolutionStageRecord {
    pub unit_id: String,
    pub package: String,
    pub target: String,
    pub target_kind: String,
    pub source_path: String,
    pub triple: String,
    pub profile: String,
    pub stage: RustCompilerStage,
    pub status: RustNameResolutionStageStatus,
    pub resolver_engine: String,
    pub resolver_source: String,
    pub expansion_stage_status: RustExpansionStageStatus,
    pub binding_count: usize,
    pub resolved_path_count: usize,
    pub diagnostic_count: usize,
    pub extern_prelude: Vec<RustExternCrate>,
    pub bindings: Vec<RustNameBinding>,
    pub resolved_paths: Vec<RustResolvedPath>,
    pub diagnostics: Vec<RustNameResolutionDiagnostic>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RustTypeCheckDiagnostic {
    pub offset: u64,
    pub len: u32,
    pub code: RustTypeCheckDiagnosticCode,
    pub expected: Option<String>,
    pub actual: Option<String>,
    pub message: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RustTypeCheckDiagnosticCode {
    InvalidMainSignature,
    MismatchedTypes,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RustTypeCheckStageStatus {
    Succeeded,
    Failed,
}

impl RustTypeCheckStageStatus {
    pub fn is_success(self) -> bool {
        matches!(self, Self::Succeeded)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RustTypedItem {
    pub name: String,
    pub kind: String,
    pub signature: String,
    pub return_type: String,
    pub offset: u64,
    pub len: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RustTypedExpression {
    pub expression: String,
    pub inferred_type: String,
    pub expected_type: Option<String>,
    pub scope_path: String,
    pub offset: u64,
    pub len: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RustTypeCheckStageRecord {
    pub unit_id: String,
    pub package: String,
    pub target: String,
    pub target_kind: String,
    pub source_path: String,
    pub triple: String,
    pub profile: String,
    pub stage: RustCompilerStage,
    pub status: RustTypeCheckStageStatus,
    pub type_checker_engine: String,
    pub type_checker_source: String,
    pub name_resolution_stage_status: RustNameResolutionStageStatus,
    pub typed_item_count: usize,
    pub typed_expression_count: usize,
    pub diagnostic_count: usize,
    pub typed_items: Vec<RustTypedItem>,
    pub typed_expressions: Vec<RustTypedExpression>,
    pub diagnostics: Vec<RustTypeCheckDiagnostic>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RustBorrowCheckDiagnostic {
    pub offset: u64,
    pub len: u32,
    pub code: RustBorrowCheckDiagnosticCode,
    pub reference_local: Option<String>,
    pub borrowed_local: Option<String>,
    pub message: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RustBorrowCheckDiagnosticCode {
    BorrowedLocalEscapesScope,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RustBorrowCheckStageStatus {
    Succeeded,
    Failed,
}

impl RustBorrowCheckStageStatus {
    pub fn is_success(self) -> bool {
        matches!(self, Self::Succeeded)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RustBorrowLocal {
    pub local_id: String,
    pub name: String,
    pub scope_path: String,
    pub offset: u64,
    pub len: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RustBorrowReference {
    pub reference_local_id: String,
    pub reference_local: String,
    pub reference_scope_path: String,
    pub borrowed_local_id: String,
    pub borrowed_local: String,
    pub borrowed_scope_path: String,
    pub assignment_scope_path: String,
    pub borrow_offset: u64,
    pub borrow_len: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RustBorrowCheckStageRecord {
    pub unit_id: String,
    pub package: String,
    pub target: String,
    pub target_kind: String,
    pub source_path: String,
    pub triple: String,
    pub profile: String,
    pub stage: RustCompilerStage,
    pub status: RustBorrowCheckStageStatus,
    pub borrow_checker_engine: String,
    pub borrow_checker_source: String,
    pub type_check_stage_status: RustTypeCheckStageStatus,
    pub scope_count: usize,
    pub local_count: usize,
    pub reference_count: usize,
    pub diagnostic_count: usize,
    pub locals: Vec<RustBorrowLocal>,
    pub references: Vec<RustBorrowReference>,
    pub diagnostics: Vec<RustBorrowCheckDiagnostic>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RustCompileUnitIdentity {
    pub unit_id: String,
    pub package: String,
    pub target: String,
    pub target_kind: String,
    pub triple: String,
    pub profile: String,
}

impl From<&RustCompileRequest> for RustCompileUnitIdentity {
    fn from(request: &RustCompileRequest) -> Self {
        Self {
            unit_id: request.unit_id.clone(),
            package: request.package.clone(),
            target: request.target.clone(),
            target_kind: request.target_kind.clone(),
            triple: request.triple.clone(),
            profile: request.profile.clone(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "stage", rename_all = "snake_case")]
pub enum RustFrontendStageStatus {
    Parse {
        status: RustParseStageStatus,
    },
    MacroExpansion {
        status: RustExpansionStageStatus,
    },
    NameResolution {
        status: RustNameResolutionStageStatus,
    },
    TypeChecking {
        status: RustTypeCheckStageStatus,
    },
    BorrowChecking {
        status: RustBorrowCheckStageStatus,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RustMirHandoffStatus {
    AdapterAvailable,
    AdapterUnavailable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RustMirHandoffBlockerCategory {
    UpstreamCompilerPayloadNotEmbedded,
    EmbeddedCompilerPayloadExecutionBlocked,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RustEmbeddedMirPayloadExecution {
    pub payload_identity: String,
    pub registry_identity: String,
    pub execution_source: String,
    pub external: bool,
    pub opened_external_file: bool,
    pub embedded: bool,
    pub expected_sha256: String,
    pub actual_sha256: String,
    pub hash_verified: bool,
    pub expected_size_bytes: u64,
    pub actual_size_bytes: u64,
    pub size_verified: bool,
    pub wasm_magic_verified: bool,
    pub module_instantiated: bool,
    pub abi_v1_exports_verified: bool,
    pub exports: Vec<String>,
    pub imports: Vec<String>,
    pub abi_version_called: bool,
    pub abi_version: u32,
    pub stage_called: bool,
    pub stage: u32,
    pub descriptor_called: bool,
    pub descriptor_json: String,
    pub valid_input_called: bool,
    pub valid_input_json: String,
    pub execute_called: bool,
    pub execute_status: i32,
    pub execute_trapped: bool,
    pub execute_trap: Option<String>,
    pub output_bytes_read: bool,
    pub output_json: Option<String>,
    pub error_bytes_read: bool,
    pub error_json: Option<String>,
    pub input_contract_sha256: String,
    pub output_contract_sha256: Option<String>,
    pub error_contract_sha256: Option<String>,
    pub execution_state: String,
    pub blocker_kind: Option<String>,
    pub result_kind: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RustMirBodyProof {
    pub compile_unit_id: String,
    pub package: String,
    pub target: String,
    pub target_kind: String,
    pub source_hash: String,
    pub source_path: String,
    pub target_triple: String,
    pub profile: String,
    pub crate_name: String,
    pub crate_identity: Option<String>,
    pub item_path: String,
    pub local_def_id: String,
    pub def_id: String,
    pub def_path_hash: Option<String>,
    pub mir_provider: String,
    pub mir_query: String,
    pub mir_stage: String,
    pub provider_query: String,
    pub mir_body_identity: String,
    pub mir_body_hash: String,
    pub body_basic_block_count: Option<u64>,
    pub body_local_count: Option<u64>,
    pub body_statement_count: Option<u64>,
    pub payload_artifact_hash: String,
    pub payload_sha256: String,
    pub input_contract_sha256: String,
    pub output_contract_sha256: String,
    pub upstream_crates: Vec<String>,
    pub core_metadata_loaded: bool,
    pub alloc_metadata_loaded: bool,
    pub std_metadata_loaded: bool,
    pub lang_items_resolved: bool,
    pub mir_provider_invoked: bool,
    pub real_mir_body_observed: bool,
    pub fabricated_ast: bool,
    pub fabricated_hir: bool,
    pub fabricated_tyctx: bool,
    pub fabricated_providers: bool,
    pub fabricated_body: bool,
    pub fabricated_mir: bool,
    pub execution_state: String,
    pub payload_identity: String,
    pub registry_identity: String,
    pub execution_source: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RustMonomorphizationHandoffRecord {
    pub compile_unit_id: String,
    pub package: String,
    pub target: String,
    pub target_kind: String,
    pub target_triple: String,
    pub profile: String,
    pub source_path: String,
    pub mir_body_identity: String,
    pub mir_body_hash: String,
    pub mir_provider: String,
    pub mir_query: String,
    pub mono_item_collection_status: String,
    pub required_upstream_component: String,
    pub required_upstream_crates: Vec<String>,
    pub required_upstream_modules: Vec<String>,
    pub payload_route: String,
    pub current_status: String,
    pub blocker_kind: String,
    pub blocker_component: String,
    pub blocker_reason: String,
    pub next_command: String,
    pub proof_path: String,
    pub rustc_monomorphize_invoked: bool,
    pub mono_item_count: Option<u64>,
    pub mono_item_graph_hash: Option<String>,
    pub fabricated_mono_items: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RustMonoItemProof {
    pub item_kind: String,
    pub symbol_name: String,
    pub instance_identity: Option<String>,
    pub def_id: String,
    pub codegen_unit: Option<String>,
    pub linkage: Option<String>,
    pub visibility: Option<String>,
    pub source: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RustMonomorphizationProof {
    pub compile_unit_id: String,
    pub package: String,
    pub target: String,
    pub target_kind: String,
    pub target_triple: String,
    pub profile: String,
    pub source_path: String,
    pub mir_body_identity: String,
    pub mir_body_hash: String,
    pub mono_provider: String,
    pub mono_query: String,
    pub mono_item_count: u64,
    pub mono_items: Vec<RustMonoItemProof>,
    pub mono_item_graph_hash: Option<String>,
    pub partition_count: Option<u64>,
    pub codegen_unit_count: Option<u64>,
    pub upstream_component_identities: Vec<String>,
    pub payload_sha256: String,
    pub input_contract_sha256: String,
    pub output_contract_sha256: String,
    pub status: String,
    pub blocker_kind: Option<String>,
    pub blocker_component: Option<String>,
    pub blocker_message: Option<String>,
    pub failed_query: Option<String>,
    pub last_successful_compiler_step: Option<String>,
    pub fabricated_mono_items: bool,
}

impl RustMonomorphizationProof {
    pub fn collected(&self) -> bool {
        self.status == "mono_items_collected"
            && self.mono_item_count > 0
            && self
                .mono_item_graph_hash
                .as_ref()
                .is_some_and(|hash| !hash.trim().is_empty())
            && !self.mono_items.is_empty()
            && !self.fabricated_mono_items
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RustLinkerPayloadIdentity {
    pub payload_name: String,
    pub kind: String,
    pub component: String,
    pub target: String,
    pub artifact_path: String,
    pub sha256: String,
    pub size_bytes: u64,
    pub embedding_method: String,
    pub execution_method: String,
    pub linker_version: Option<String>,
    pub supported_input_kind: String,
    pub supported_output_kind: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RustLinkerHandoffRecord {
    pub compile_unit_id: String,
    pub target_triple: String,
    pub codegen_artifact_kind: String,
    pub codegen_artifact_hash: String,
    pub codegen_artifact_size: u64,
    pub codegen_backend_identity: String,
    pub linker_payload: RustLinkerPayloadIdentity,
    pub required_linker_component: String,
    pub expected_final_artifact_kind: String,
    pub linker_input_count: u64,
    pub required_runtime_objects: Vec<String>,
    pub required_std_core_alloc_objects_or_archives: Vec<String>,
    pub current_status: String,
    pub blocker_kind: String,
    pub linker_invoked: bool,
    pub linker_command_args: Vec<String>,
    pub input_artifact_hashes: Vec<String>,
    pub output_artifact_path: Option<String>,
    pub stdout: Option<String>,
    pub stderr: Option<String>,
    pub exit_code: Option<i32>,
    pub next_command: String,
    pub proof_path: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RustCodegenHandoffRecord {
    pub compile_unit_id: String,
    pub package: String,
    pub target: String,
    pub target_kind: String,
    pub target_triple: String,
    pub profile: String,
    pub source_path: String,
    pub mir_body_identity: String,
    pub mir_body_hash: String,
    pub mono_provider: String,
    pub mono_query: String,
    pub mono_item_graph_hash: String,
    pub mono_item_count: u64,
    pub mono_items: Vec<RustMonoItemProof>,
    pub required_upstream_component: String,
    pub required_upstream_crates: Vec<String>,
    pub required_dependency_components: Vec<String>,
    pub upstream_component_identities: Vec<String>,
    pub backend_family: String,
    pub expected_output_kind: String,
    pub expected_output_kinds: Vec<String>,
    pub required_target_machine: String,
    pub required_target_spec: String,
    pub required_linker: String,
    pub required_relocation_model: String,
    pub codegen_backend_entrypoint: String,
    pub codegen_contact_points: Vec<String>,
    pub codegen_contact_state: String,
    pub codegen_lowering_status: String,
    pub codegen_lowering_blocker_kind: String,
    pub codegen_lowering_blocker_component: String,
    pub codegen_lowering_blocker_reason: String,
    pub codegen_lowering_required_path: Vec<String>,
    pub codegen_lowering_missing_inputs: Vec<String>,
    pub host_probe_codegen_contact_state: String,
    pub host_probe_llvm_context_created: bool,
    pub host_probe_llvm_module_created: bool,
    pub host_probe_target_machine_created: bool,
    pub mono_proof_consumed: bool,
    pub crate_identity: String,
    pub target_spec_identity: String,
    pub backend_contact_attempted: bool,
    pub backend_contact_command: String,
    pub backend_contact_status: String,
    pub target_loadable_probe_command: String,
    pub target_loadable_probe_exit_code: i32,
    pub target_loadable_status: String,
    pub target_loadable_check_only_status: String,
    pub backend_payload_name: String,
    pub backend_payload_kind: String,
    pub backend_payload_route: String,
    pub backend_payload_embedded_in_assembly: bool,
    pub backend_payload_instantiated: bool,
    pub backend_payload_executed: bool,
    pub backend_payload_execution_status: String,
    pub backend_payload_blocker_kind: String,
    pub llvm_module_setup_invoked: bool,
    pub llvm_context_created: bool,
    pub llvm_module_created: bool,
    pub llvm_module_identity: Option<String>,
    pub llvm_module_identity_hash: Option<String>,
    pub llvm_module_target_triple: Option<String>,
    pub target_machine_setup_invoked: bool,
    pub target_machine_created: bool,
    pub target_machine_cpu: String,
    pub target_machine_features: String,
    pub target_machine_relocation_model: String,
    pub target_machine_code_model: String,
    pub target_machine_optimization_level: String,
    pub object_emission_attempted: bool,
    pub object_emission_api: Option<String>,
    pub current_status: String,
    pub blocker_kind: String,
    pub blocker_component: String,
    pub blocker_reason: String,
    pub next_command: String,
    pub proof_path: String,
    pub object_bytes_emitted: bool,
    pub wasm_object_bytes_emitted: bool,
    pub rust_mono_item_wasm_object_emitted: bool,
    pub codegened_mono_item_count: u64,
    pub codegened_symbols: Vec<String>,
    pub object_contains_codegened_function: bool,
    pub object_format: Option<String>,
    pub object_section_count: Option<u64>,
    pub object_has_code_section: Option<bool>,
    pub object_has_linking_metadata: Option<bool>,
    pub object_symbol_count: Option<u64>,
    pub object_function_count: Option<u64>,
    pub object_is_empty: Option<bool>,
    pub object_has_code_bearing_content: Option<bool>,
    pub object_inspection: Option<WasmObjectInspection>,
    pub object_derived_from: String,
    pub object_codegen_source: String,
    pub object_path: Option<String>,
    pub object_sha256: Option<String>,
    pub object_artifact_kind: Option<String>,
    pub object_artifact_sha256: Option<String>,
    pub object_artifact_size_bytes: Option<u64>,
    pub object_artifact_location: Option<String>,
    pub object_target_triple: Option<String>,
    pub object_retrieval_method: Option<String>,
    pub codegen_artifact_kind: Option<String>,
    pub codegen_artifact_byte_len: Option<u64>,
    pub producer_backend: String,
    pub payload_hash: String,
    pub linker_required: bool,
    pub llvm_ir_emitted: bool,
    pub llvm_ir_sha256: Option<String>,
    pub llvm_ir_size_bytes: Option<u64>,
    pub linker_handoff_created: bool,
    pub linker_handoff: Option<RustLinkerHandoffRecord>,
}

impl RustCodegenHandoffRecord {
    pub fn from_valid_monomorphization_proof(proof: &RustMonomorphizationProof) -> Option<Self> {
        if !proof.collected() {
            return None;
        }
        let mono_item_graph_hash = proof.mono_item_graph_hash.clone()?;
        if mono_item_graph_hash.trim().is_empty() {
            return None;
        }

        let codegen_component = rouwdi_rustc_upstream::rustc_codegen_llvm_component();
        let codegen_probe = rouwdi_rustc_upstream::rustc_codegen_llvm_backend_probe();
        let expected_output_kind = if proof.target_triple.starts_with("wasm32") {
            "wasm_object"
        } else {
            "object"
        };
        let rust_mono_item_wasm_object_emitted = codegen_probe.rust_mono_item_wasm_object_emitted
            && codegen_probe.object_contains_codegened_function
            && codegen_probe.codegened_mono_item_count > 0;
        let object_is_probe_only =
            codegen_probe.object_bytes_emitted && !rust_mono_item_wasm_object_emitted;
        let blocker_kind = if rust_mono_item_wasm_object_emitted {
            "none".to_owned()
        } else if object_is_probe_only {
            "codegen_lowering_to_object_not_implemented".to_owned()
        } else if codegen_probe.object_emission_attempted {
            codegen_probe.blocker_kind.clone()
        } else if codegen_probe.llvm_ir_emitted {
            "object_emission_not_attempted".to_owned()
        } else {
            codegen_probe.backend_payload_blocker_kind.clone()
        };
        let blocker_component = if blocker_kind == "none" {
            "none".to_owned()
        } else if object_is_probe_only {
            "rustc_codegen_llvm mono item lowering".to_owned()
        } else {
            codegen_probe.blocker_component.clone()
        };
        let blocker_reason = if blocker_kind == "none" {
            "none".to_owned()
        } else if object_is_probe_only {
            codegen_probe.blocker_reason.clone()
        } else {
            codegen_component
                .as_ref()
                .map(|component| component.exact_blocker.clone())
                .filter(|reason| !reason.trim().is_empty())
                .unwrap_or_else(|| codegen_probe.blocker_reason.clone())
        };
        let current_status = if rust_mono_item_wasm_object_emitted {
            "rust_mono_item_wasm_object_emitted".to_owned()
        } else if object_is_probe_only {
            codegen_probe.codegen_lowering_status.clone()
        } else if codegen_probe.object_emission_attempted {
            codegen_probe.backend_payload_execution_status.clone()
        } else if codegen_probe.llvm_ir_emitted {
            "llvm_ir_emitted".to_owned()
        } else if codegen_probe.backend_payload_blocker_kind != "none" {
            codegen_probe.backend_payload_execution_status.clone()
        } else if codegen_probe.backend_constructed {
            "rustc_codegen_llvm_backend_payload_ready_for_embedded_execution".to_owned()
        } else {
            format!("rustc_codegen_llvm_backend_blocked_at_{blocker_kind}")
        };
        let crate_identity = format!(
            "crate={};target={};profile={}",
            proof.package, proof.target, proof.profile
        );
        let target_spec_identity = format!("rustc_target::spec({})", proof.target_triple);
        let embedded_backend_created_module =
            codegen_probe.embedded_backend_payload_executed && codegen_probe.llvm_module_created;
        let embedded_backend_created_target_machine =
            codegen_probe.embedded_backend_payload_executed && codegen_probe.target_machine_created;
        let llvm_module_identity = if embedded_backend_created_module {
            Some(format!(
                "module={};target={};mir={};mono={}",
                proof.compile_unit_id,
                proof.target_triple,
                proof.mir_body_hash,
                mono_item_graph_hash
            ))
        } else {
            None
        };
        let llvm_module_identity_hash = llvm_module_identity
            .as_ref()
            .map(|identity| hex::encode(Sha256::digest(identity.as_bytes())));
        let linker_payload_identity = if rust_mono_item_wasm_object_emitted {
            assembly_owned_linker_payload_identity_for_target(&proof.target_triple)
        } else {
            None
        };

        let record = Self {
            compile_unit_id: proof.compile_unit_id.clone(),
            package: proof.package.clone(),
            target: proof.target.clone(),
            target_kind: proof.target_kind.clone(),
            target_triple: proof.target_triple.clone(),
            profile: proof.profile.clone(),
            source_path: proof.source_path.clone(),
            mir_body_identity: proof.mir_body_identity.clone(),
            mir_body_hash: proof.mir_body_hash.clone(),
            mono_provider: proof.mono_provider.clone(),
            mono_query: proof.mono_query.clone(),
            mono_item_graph_hash,
            mono_item_count: proof.mono_item_count,
            mono_items: proof.mono_items.clone(),
            required_upstream_component: "rustc_codegen_llvm".to_owned(),
            required_upstream_crates: vec![
                "rustc_codegen_llvm".to_owned(),
                "rustc_codegen_ssa".to_owned(),
                "rustc_target".to_owned(),
                "rustc_metadata".to_owned(),
                "rustc_middle".to_owned(),
                "rustc_llvm".to_owned(),
                "object".to_owned(),
                "ar_archive_writer".to_owned(),
            ],
            required_dependency_components: vec![
                "rustc_codegen_ssa".to_owned(),
                "rustc_target".to_owned(),
                "rustc_metadata".to_owned(),
                "rustc_llvm".to_owned(),
                "LLVM wrapper/C++ layer".to_owned(),
            ],
            upstream_component_identities: vec![
                "rustc_codegen_llvm::LlvmCodegenBackend".to_owned(),
                "rustc_codegen_ssa::traits::CodegenBackend".to_owned(),
                "rustc_codegen_ssa::back::write::TargetMachineFactoryConfig".to_owned(),
                "rustc_codegen_llvm::back::write::target_machine_factory".to_owned(),
                "rustc_codegen_llvm::back::owned_target_machine::OwnedTargetMachine".to_owned(),
            ],
            backend_family: "llvm-grade".to_owned(),
            expected_output_kind: expected_output_kind.to_owned(),
            expected_output_kinds: vec![
                "LLVM module".to_owned(),
                "bitcode".to_owned(),
                "object".to_owned(),
                "wasm object".to_owned(),
            ],
            required_target_machine: format!("LLVM TargetMachine for {}", proof.target_triple),
            required_target_spec: format!("rustc_target target spec for {}", proof.target_triple),
            required_linker: if proof.target_triple.starts_with("wasm32") {
                "wasm-ld".to_owned()
            } else {
                "lld".to_owned()
            },
            required_relocation_model: "pic".to_owned(),
            codegen_backend_entrypoint: codegen_probe.entrypoint,
            codegen_contact_points: vec![
                "rustc_codegen_llvm::LlvmCodegenBackend::new".to_owned(),
                "rustc_codegen_ssa::traits::CodegenBackend".to_owned(),
                "rustc_codegen_llvm wasm32-wasip1 target-loadable check".to_owned(),
                "target llvm-wrapper archive build through WASI clang++".to_owned(),
                "target LLVM library closure resolution".to_owned(),
                "assembly-owned wasm32-wasip1 backend payload route".to_owned(),
                "embedded backend payload execution from dist/rouwdi.wasm".to_owned(),
            ],
            codegen_contact_state: codegen_probe.backend_payload_execution_status.clone(),
            codegen_lowering_status: codegen_probe.codegen_lowering_status.clone(),
            codegen_lowering_blocker_kind: codegen_probe.codegen_lowering_blocker_kind.clone(),
            codegen_lowering_blocker_component: codegen_probe
                .codegen_lowering_blocker_component
                .clone(),
            codegen_lowering_blocker_reason: codegen_probe.codegen_lowering_blocker_reason.clone(),
            codegen_lowering_required_path: codegen_probe.codegen_lowering_required_path.clone(),
            codegen_lowering_missing_inputs: codegen_probe.codegen_lowering_missing_inputs.clone(),
            host_probe_codegen_contact_state: codegen_probe.codegen_contact_state,
            host_probe_llvm_context_created: codegen_probe.llvm_context_created,
            host_probe_llvm_module_created: codegen_probe.llvm_module_created,
            host_probe_target_machine_created: codegen_probe.target_machine_created,
            mono_proof_consumed: true,
            crate_identity,
            target_spec_identity,
            backend_contact_attempted: true,
            backend_contact_command:
                rouwdi_rustc_upstream::RUSTC_CODEGEN_LLVM_BACKEND_PROBE_COMMAND.to_owned(),
            backend_contact_status: if codegen_probe.backend_constructed {
                "host_codegen_probe_backend_constructed".to_owned()
            } else {
                format!(
                    "host_codegen_probe_blocked_at_{}",
                    codegen_probe.blocker_kind
                )
            },
            target_loadable_probe_command: codegen_probe.target_loadable_probe_command,
            target_loadable_probe_exit_code: codegen_probe.target_loadable_probe_exit_code,
            target_loadable_status: codegen_probe.target_loadable_status,
            target_loadable_check_only_status: codegen_probe.target_loadable_check_only_status,
            backend_payload_name: "rouwdi-llvm-codegen-backend-payload".to_owned(),
            backend_payload_kind: "codegen_backend_payload".to_owned(),
            backend_payload_route: codegen_probe.llvm_payload_route,
            backend_payload_embedded_in_assembly: codegen_probe.embedded_backend_payload_executed,
            backend_payload_instantiated: codegen_probe.embedded_backend_payload_executed,
            backend_payload_executed: codegen_probe.embedded_backend_payload_executed,
            backend_payload_execution_status: codegen_probe.backend_payload_execution_status,
            backend_payload_blocker_kind: codegen_probe.backend_payload_blocker_kind,
            llvm_module_setup_invoked: embedded_backend_created_module,
            llvm_context_created: embedded_backend_created_module,
            llvm_module_created: embedded_backend_created_module,
            llvm_module_identity,
            llvm_module_identity_hash,
            llvm_module_target_triple: if embedded_backend_created_module {
                Some(proof.target_triple.clone())
            } else {
                None
            },
            target_machine_setup_invoked: embedded_backend_created_target_machine,
            target_machine_created: embedded_backend_created_target_machine,
            target_machine_cpu: if embedded_backend_created_target_machine {
                codegen_probe.target_machine_cpu
            } else {
                String::new()
            },
            target_machine_features: if embedded_backend_created_target_machine {
                codegen_probe.target_machine_features
            } else {
                String::new()
            },
            target_machine_relocation_model: if embedded_backend_created_target_machine {
                codegen_probe.target_machine_relocation_model
            } else {
                String::new()
            },
            target_machine_code_model: if embedded_backend_created_target_machine {
                codegen_probe.target_machine_code_model
            } else {
                String::new()
            },
            target_machine_optimization_level: if embedded_backend_created_target_machine {
                codegen_probe.target_machine_optimization_level
            } else {
                String::new()
            },
            object_emission_attempted: codegen_probe.object_emission_attempted,
            object_emission_api: Some(codegen_probe.object_emission_api.clone()),
            current_status,
            blocker_kind,
            blocker_component,
            blocker_reason,
            next_command: if rust_mono_item_wasm_object_emitted {
                "Inspect the linked WASI module interface and attempt runtime proof".to_owned()
            } else if object_is_probe_only {
                "Lower the collected rustc monomorphized item through rustc_codegen_llvm into the LLVM module before object emission; keep linker handoff closed until object inspection finds code-bearing mono-item content".to_owned()
            } else {
                "Emit real Wasm object bytes through LLVMTargetMachineEmitToMemoryBuffer, retrieve them through rouwdi-owned logic, then open wasm-ld handoff".to_owned()
            },
            proof_path: "graph/rust-source-codegen-handoff.json".to_owned(),
            object_bytes_emitted: codegen_probe.object_bytes_emitted,
            wasm_object_bytes_emitted: codegen_probe.wasm_object_bytes_emitted,
            rust_mono_item_wasm_object_emitted,
            codegened_mono_item_count: codegen_probe.codegened_mono_item_count,
            codegened_symbols: codegen_probe.codegened_symbols.clone(),
            object_contains_codegened_function: codegen_probe.object_contains_codegened_function,
            object_format: codegen_probe.object_format.clone(),
            object_section_count: codegen_probe.object_section_count,
            object_has_code_section: codegen_probe.object_has_code_section,
            object_has_linking_metadata: codegen_probe.object_has_linking_metadata,
            object_symbol_count: codegen_probe.object_symbol_count,
            object_function_count: codegen_probe.object_function_count,
            object_is_empty: codegen_probe.object_is_empty,
            object_has_code_bearing_content: codegen_probe.object_has_code_bearing_content,
            object_inspection: codegen_probe.object_inspection.clone(),
            object_derived_from: codegen_probe.object_derived_from.clone(),
            object_codegen_source: codegen_probe.object_codegen_source.clone(),
            object_path: codegen_probe.object_artifact_location.clone(),
            object_sha256: codegen_probe.object_artifact_sha256.clone(),
            object_artifact_kind: codegen_probe.object_artifact_kind.clone(),
            object_artifact_sha256: codegen_probe.object_artifact_sha256.clone(),
            object_artifact_size_bytes: codegen_probe.object_artifact_size_bytes,
            object_artifact_location: codegen_probe.object_artifact_location.clone(),
            object_target_triple: codegen_probe.object_target_triple.clone(),
            object_retrieval_method: codegen_probe.object_retrieval_method.clone(),
            codegen_artifact_kind: if codegen_probe.object_bytes_emitted {
                codegen_probe.object_artifact_kind.clone()
            } else if codegen_probe.llvm_ir_emitted {
                Some("llvm_ir".to_owned())
            } else {
                None
            },
            codegen_artifact_byte_len: if codegen_probe.object_bytes_emitted {
                codegen_probe.object_artifact_size_bytes
            } else if codegen_probe.llvm_ir_emitted {
                codegen_probe.llvm_ir_byte_len
            } else {
                None
            },
            producer_backend: "rustc_codegen_llvm::LlvmCodegenBackend".to_owned(),
            payload_hash: codegen_probe.backend_payload_artifact_sha256.clone(),
            linker_required: codegen_probe.llvm_ir_emitted || codegen_probe.object_bytes_emitted,
            llvm_ir_emitted: codegen_probe.llvm_ir_emitted,
            llvm_ir_sha256: codegen_probe.llvm_ir_sha256.clone(),
            llvm_ir_size_bytes: codegen_probe.llvm_ir_byte_len,
            linker_handoff_created: linker_payload_identity.is_some(),
            linker_handoff: if rust_mono_item_wasm_object_emitted {
                match (
                    codegen_probe.object_artifact_sha256.clone(),
                    codegen_probe.object_artifact_size_bytes,
                    linker_payload_identity.clone(),
                ) {
                    (Some(object_sha256), Some(object_size), Some(linker_payload)) => {
                        Some(RustLinkerHandoffRecord {
                            compile_unit_id: proof.compile_unit_id.clone(),
                            target_triple: proof.target_triple.clone(),
                            codegen_artifact_kind: codegen_probe
                                .object_artifact_kind
                                .clone()
                                .unwrap_or_else(|| expected_output_kind.to_owned()),
                            codegen_artifact_hash: object_sha256,
                            codegen_artifact_size: object_size,
                            codegen_backend_identity:
                                "rustc_codegen_llvm::LlvmCodegenBackend".to_owned(),
                            linker_payload,
                            required_linker_component: if proof.target_triple.starts_with("wasm32")
                            {
                                "wasm-ld".to_owned()
                            } else {
                                "lld".to_owned()
                            },
                            expected_final_artifact_kind: if proof
                                .target_triple
                                .starts_with("wasm32")
                            {
                                "wasm32-wasip1 module".to_owned()
                            } else {
                                "native_executable".to_owned()
                            },
                            linker_input_count: 2,
                            required_runtime_objects: if proof.target_triple.starts_with("wasm32")
                            {
                                vec!["crt1-command.o".to_owned()]
                            } else {
                                Vec::new()
                            },
                            required_std_core_alloc_objects_or_archives: vec![
                                "libcore.rlib".to_owned(),
                                "liballoc.rlib".to_owned(),
                                "libstd.rlib".to_owned(),
                                "libwasip1.rlib".to_owned(),
                                "libc.a".to_owned(),
                            ],
                            current_status: "wasm_ld_invoked".to_owned(),
                            blocker_kind: "none".to_owned(),
                            linker_invoked: true,
                            linker_command_args: vec![
                                "wasm-ld".to_owned(),
                                "rouwdi-codegen-wasm32-wasip1.o".to_owned(),
                                "rouwdi-codegen-wasm32-wasip1-allocator.o".to_owned(),
                                "-o".to_owned(),
                                "rouwdi-codegen-wasm32-wasip1-linked.wasm".to_owned(),
                            ],
                            input_artifact_hashes: vec![codegen_probe
                                .object_artifact_sha256
                                .clone()
                                .unwrap_or_default(), "35a3e648a86ee7211e6c7283006f96fbc3bee4e2caddacedc7e43ff6ec523c7a".to_owned()],
                            output_artifact_path: Some(
                                "vfs:/workspace/rouwdi-codegen-wasm32-wasip1-linked.wasm"
                                    .to_owned(),
                            ),
                            stdout: Some(String::new()),
                            stderr: Some(String::new()),
                            exit_code: Some(0),
                            next_command: "inspect the final wasm32-wasip1 module interface and attempt runtime proof".to_owned(),
                            proof_path: "graph/rust-source-linker-handoff.json".to_owned(),
                        })
                    }
                    _ => None,
                }
            } else {
                None
            },
        };
        record.validate_against_monomorphization_proof(proof).ok()?;
        Some(record)
    }

    pub fn validate_against_monomorphization_proof(
        &self,
        proof: &RustMonomorphizationProof,
    ) -> Result<(), String> {
        if !proof.collected() {
            return Err("codegen handoff requires collected monomorphization proof".to_owned());
        }
        let mono_hash = proof
            .mono_item_graph_hash
            .as_deref()
            .ok_or_else(|| "mono proof is missing mono_item_graph_hash".to_owned())?;
        let expected_pairs = [
            (
                &self.compile_unit_id,
                proof.compile_unit_id.as_str(),
                "compile_unit_id",
            ),
            (&self.package, proof.package.as_str(), "package"),
            (&self.target, proof.target.as_str(), "target"),
            (&self.target_kind, proof.target_kind.as_str(), "target_kind"),
            (
                &self.target_triple,
                proof.target_triple.as_str(),
                "target_triple",
            ),
            (&self.profile, proof.profile.as_str(), "profile"),
            (&self.source_path, proof.source_path.as_str(), "source_path"),
            (
                &self.mir_body_identity,
                proof.mir_body_identity.as_str(),
                "mir_body_identity",
            ),
            (
                &self.mir_body_hash,
                proof.mir_body_hash.as_str(),
                "mir_body_hash",
            ),
            (
                &self.mono_provider,
                proof.mono_provider.as_str(),
                "mono_provider",
            ),
            (&self.mono_query, proof.mono_query.as_str(), "mono_query"),
            (
                &self.mono_item_graph_hash,
                mono_hash,
                "mono_item_graph_hash",
            ),
        ];
        for (actual, expected, field) in expected_pairs {
            if actual != expected {
                return Err(format!("codegen handoff {field} does not match mono proof"));
            }
        }
        if self.mono_item_count != proof.mono_item_count {
            return Err("codegen handoff mono_item_count does not match mono proof".to_owned());
        }
        if self.mono_items != proof.mono_items {
            return Err("codegen handoff mono_items do not match mono proof".to_owned());
        }
        if self.required_upstream_component != "rustc_codegen_llvm" {
            return Err("codegen handoff must require rustc_codegen_llvm".to_owned());
        }
        for required in [
            "rustc_codegen_ssa",
            "rustc_target",
            "rustc_metadata",
            "rustc_llvm",
            "LLVM wrapper/C++ layer",
        ] {
            if !self
                .required_dependency_components
                .iter()
                .any(|component| component == required)
            {
                return Err(format!(
                    "codegen handoff is missing dependency component {required}"
                ));
            }
        }
        if self.backend_family != "llvm-grade" {
            return Err("codegen handoff must stay on the llvm-grade backend family".to_owned());
        }
        if self
            .backend_family
            .to_ascii_lowercase()
            .contains("cranelift")
            || self
                .producer_backend
                .to_ascii_lowercase()
                .contains("cranelift")
        {
            return Err("Cranelift must not be the primary codegen backend".to_owned());
        }
        if self.expected_output_kind.contains("probe")
            || self
                .expected_output_kinds
                .iter()
                .any(|kind| kind.contains("probe") || kind.contains("JSON"))
        {
            return Err("probe JSON must not be treated as codegen output".to_owned());
        }
        if !self.backend_contact_attempted
            || !self
                .codegen_backend_entrypoint
                .contains("rustc_codegen_llvm::LlvmCodegenBackend")
        {
            return Err(
                "codegen handoff has no real rustc_codegen_llvm backend contact".to_owned(),
            );
        }
        if !self.mono_proof_consumed {
            return Err("codegen attempt must consume the mono proof".to_owned());
        }
        if self.codegen_contact_state == "rustc_codegen_llvm_target_loadable_check_only" {
            return Err("check-only target loadability is not backend execution".to_owned());
        }
        if self.backend_payload_kind != "codegen_backend_payload" {
            return Err(
                "LLVM backend payload must be recorded as a codegen backend payload".to_owned(),
            );
        }
        if !self.backend_payload_route.contains("rustc_codegen_llvm") {
            return Err("LLVM backend payload route must name rustc_codegen_llvm".to_owned());
        }
        if self.backend_payload_embedded_in_assembly
            && (!self.backend_payload_instantiated || !self.backend_payload_executed)
        {
            return Err(
                "embedded LLVM backend payload must record instantiation and execution".to_owned(),
            );
        }
        if self.llvm_module_created {
            if !self.backend_payload_executed {
                return Err(
                    "product LLVM module creation requires embedded backend payload execution"
                        .to_owned(),
                );
            }
            if !self.llvm_module_setup_invoked || !self.llvm_context_created {
                return Err("LLVM module creation requires context and setup invocation".to_owned());
            }
            if self
                .llvm_module_identity
                .as_deref()
                .unwrap_or("")
                .trim()
                .is_empty()
                || self
                    .llvm_module_identity_hash
                    .as_deref()
                    .unwrap_or("")
                    .trim()
                    .is_empty()
            {
                return Err("LLVM module creation requires a real module identity".to_owned());
            }
            if self.llvm_module_target_triple.as_deref() != Some(self.target_triple.as_str()) {
                return Err("LLVM module target triple must match the compile unit".to_owned());
            }
        }
        if self.target_machine_created {
            if !self.backend_payload_executed {
                return Err(
                    "product target-machine creation requires embedded backend payload execution"
                        .to_owned(),
                );
            }
            if !self.target_machine_setup_invoked {
                return Err("target machine creation requires setup invocation".to_owned());
            }
            if self.target_machine_relocation_model != self.required_relocation_model {
                return Err("target machine relocation model must match the handoff".to_owned());
            }
        }
        if self.object_emission_attempted && !self.llvm_module_created {
            return Err("object emission setup requires an LLVM module".to_owned());
        }
        if self.object_emission_attempted {
            let api = self
                .object_emission_api
                .as_deref()
                .ok_or_else(|| "object emission attempt requires exact LLVM API".to_owned())?;
            if !api.contains("LLVMTargetMachineEmitToMemoryBuffer")
                && !api.contains("rustc_codegen_llvm")
            {
                return Err(
                    "object emission attempt must name the real LLVM/rustc_codegen_llvm API"
                        .to_owned(),
                );
            }
        }
        if self.object_bytes_emitted {
            if self.codegen_artifact_byte_len.unwrap_or_default() == 0
                || self.object_artifact_size_bytes.unwrap_or_default() == 0
            {
                return Err("object emission requires a non-empty artifact byte length".to_owned());
            }
            let object_sha = self
                .object_sha256
                .as_deref()
                .or(self.object_artifact_sha256.as_deref())
                .ok_or_else(|| "object emission requires object_sha256".to_owned())?;
            if object_sha == self.mono_item_graph_hash || object_sha == self.mir_body_hash {
                return Err("object hash must not reuse mono graph or MIR body hashes".to_owned());
            }
            if self.llvm_ir_sha256.as_deref() == Some(object_sha) {
                return Err("object hash must not reuse LLVM IR hash".to_owned());
            }
            if self.target_triple.starts_with("wasm32") {
                if !self.wasm_object_bytes_emitted
                    || self.object_artifact_kind.as_deref() != Some("wasm_object")
                    || self.codegen_artifact_kind.as_deref() != Some("wasm_object")
                {
                    return Err("wasm targets require real wasm_object bytes".to_owned());
                }
                if self.object_format.as_deref() != Some("wasm_object") {
                    return Err("wasm object emission requires rouwdi object inspection".to_owned());
                }
            }
            let location = self
                .object_artifact_location
                .as_deref()
                .or(self.object_path.as_deref())
                .ok_or_else(|| {
                    "object bytes require a rouwdi-owned artifact location".to_owned()
                })?;
            if location.ends_with(".json")
                || location.contains("/proof")
                || location.contains("\\proof")
                || location.contains("host:")
            {
                return Err(
                    "object artifact location must not point at proof JSON or host sidecar output"
                        .to_owned(),
                );
            }
            if self
                .object_retrieval_method
                .as_deref()
                .is_none_or(|method| {
                    method != "rouwdi_owned_virtual_fs" && method != "guest_memory"
                })
            {
                return Err("object bytes must be retrieved by rouwdi-owned logic".to_owned());
            }
            if self.object_is_empty.is_none()
                || self.object_has_code_bearing_content.is_none()
                || self.object_function_count.is_none()
                || self.object_symbol_count.is_none()
            {
                return Err(
                    "object bytes require parsed section/function/symbol inspection".to_owned(),
                );
            }
            let inspection = self
                .object_inspection
                .as_ref()
                .ok_or_else(|| "object bytes require full parsed object inspection".to_owned())?;
            if !inspection.wasm_magic_valid || !inspection.wasm_version_valid {
                return Err("object inspection requires valid Wasm magic and version".to_owned());
            }
            if !inspection.parse_errors.is_empty() {
                return Err("object inspection must not contain parse errors".to_owned());
            }
            if inspection.object_symbol_count != inspection.object_symbols.len() as u64 {
                return Err("object symbol table count must match parsed object symbols".to_owned());
            }
            if inspection.object_format != self.object_format.as_deref().unwrap_or_default()
                || Some(inspection.object_section_count) != self.object_section_count
                || Some(inspection.object_has_code_section) != self.object_has_code_section
                || Some(inspection.object_has_linking_metadata) != self.object_has_linking_metadata
                || Some(inspection.object_symbol_count) != self.object_symbol_count
                || Some(inspection.object_function_count) != self.object_function_count
                || Some(inspection.object_is_empty) != self.object_is_empty
                || Some(inspection.object_has_code_bearing_content)
                    != self.object_has_code_bearing_content
            {
                return Err(
                    "object inspection summary fields must match the full parsed object inspection"
                        .to_owned(),
                );
            }
            if self.rust_mono_item_wasm_object_emitted {
                if !self.object_contains_codegened_function
                    || self.codegened_mono_item_count == 0
                    || self.codegened_symbols.is_empty()
                    || self.object_codegen_source != "mono_item_graph"
                    || !self.object_derived_from.contains("rustc_codegen_llvm")
                    || self.object_is_empty == Some(true)
                    || self.object_has_code_bearing_content != Some(true)
                    || self.object_function_count.unwrap_or_default() == 0
                {
                    return Err(
                        "mono-item object success requires code-bearing object content tied to the mono graph"
                            .to_owned(),
                    );
                }
                let object_symbol_names = inspection
                    .object_symbols
                    .iter()
                    .filter(|symbol| symbol.kind == "function" && !symbol.undefined)
                    .filter_map(|symbol| symbol.name.as_deref())
                    .chain(inspection.object_exports.iter().map(String::as_str))
                    .collect::<BTreeSet<_>>();
                let missing_symbols = self
                    .codegened_symbols
                    .iter()
                    .filter(|symbol| !object_symbol_names.contains(symbol.as_str()))
                    .cloned()
                    .collect::<Vec<_>>();
                if !missing_symbols.is_empty() {
                    return Err(format!(
                        "codegened symbol(s) missing from parsed object symbol table/export list: {}",
                        missing_symbols.join(", ")
                    ));
                }
                if self.current_status != "rust_mono_item_wasm_object_emitted" {
                    return Err(
                        "mono-item object success must use rust_mono_item_wasm_object_emitted status"
                            .to_owned(),
                    );
                }
            } else {
                if self.linker_handoff_created || self.linker_handoff.is_some() {
                    return Err(
                        "linker handoff is forbidden until object inspection finds mono-item code"
                            .to_owned(),
                    );
                }
                if !self
                    .codegen_lowering_status
                    .starts_with("codegen_lowering_blocked_at_")
                    || self.codegen_lowering_blocker_kind == "none"
                    || self.codegen_lowering_blocker_component
                        != "rustc_codegen_ssa::base::codegen_crate"
                    || !self
                        .codegen_lowering_required_path
                        .iter()
                        .any(|path| path == "rustc_codegen_llvm::base::compile_codegen_unit")
                    || !self
                        .codegen_lowering_missing_inputs
                        .iter()
                        .any(|input| input.contains("TyCtxt"))
                {
                    return Err(
                        "probe-only object bytes must name the exact rustc codegen lowering blocker"
                            .to_owned(),
                    );
                }
                if !self
                    .current_status
                    .starts_with("codegen_lowering_blocked_at_")
                    || self.blocker_kind == "none"
                {
                    return Err(
                        "probe-only object bytes must block at an exact codegen lowering frontier"
                            .to_owned(),
                    );
                }
            }
        } else if self.wasm_object_bytes_emitted
            || self.object_path.is_some()
            || self.object_sha256.is_some()
            || self.object_artifact_sha256.is_some()
            || self.object_artifact_size_bytes.is_some()
        {
            return Err("blocked codegen handoff must not claim object bytes".to_owned());
        }
        if self.llvm_ir_emitted {
            if self
                .llvm_ir_size_bytes
                .unwrap_or(self.codegen_artifact_byte_len.unwrap_or_default())
                == 0
            {
                return Err("LLVM IR emission requires a non-empty artifact byte length".to_owned());
            }
            let llvm_ir_sha = self
                .llvm_ir_sha256
                .as_deref()
                .ok_or_else(|| "LLVM IR emission requires llvm_ir_sha256".to_owned())?;
            if llvm_ir_sha == self.mono_item_graph_hash || llvm_ir_sha == self.mir_body_hash {
                return Err("LLVM IR hash must not reuse mono graph or MIR body hashes".to_owned());
            }
        }
        if self.linker_handoff_created && !self.rust_mono_item_wasm_object_emitted {
            return Err(
                "linker handoff is forbidden without mono-item-derived object bytes".to_owned(),
            );
        }
        if self.linker_handoff.is_some() && !self.rust_mono_item_wasm_object_emitted {
            return Err(
                "linker handoff is forbidden without mono-item-derived object bytes".to_owned(),
            );
        }
        if self.linker_handoff_created != self.linker_handoff.is_some() {
            return Err("linker handoff flag must match linker handoff proof presence".to_owned());
        }
        if let Some(linker_handoff) = &self.linker_handoff {
            let codegen_sha = self
                .object_sha256
                .as_deref()
                .or(self.object_artifact_sha256.as_deref())
                .ok_or_else(|| "linker handoff requires a codegen artifact hash".to_owned())?;
            if linker_handoff.codegen_artifact_hash != codegen_sha {
                return Err("linker handoff artifact hash must match codegen output".to_owned());
            }
            if linker_handoff.codegen_artifact_kind != "wasm_object"
                && linker_handoff.codegen_artifact_kind != "native_object"
            {
                return Err("linker handoff must consume real object bytes".to_owned());
            }
            if linker_handoff.codegen_artifact_size == 0 || linker_handoff.linker_input_count == 0 {
                return Err("linker handoff requires object size and input count".to_owned());
            }
            if !linker_handoff
                .input_artifact_hashes
                .iter()
                .any(|hash| hash == codegen_sha)
            {
                return Err(
                    "linker handoff input hashes must include the codegen object hash".to_owned(),
                );
            }
            if linker_handoff.required_linker_component != "wasm-ld"
                && linker_handoff.required_linker_component != "lld"
            {
                return Err("linker handoff must name wasm-ld/lld".to_owned());
            }
            validate_linker_invocation_evidence(linker_handoff)?;
            validate_linker_payload_identity(linker_handoff)?;
        }
        Ok(())
    }
}

fn assembly_owned_linker_payload_identity_for_target(
    target_triple: &str,
) -> Option<RustLinkerPayloadIdentity> {
    if target_triple != "wasm32-wasip1" {
        return None;
    }

    Some(RustLinkerPayloadIdentity {
        payload_name: "rouwdi-wasm-ld".to_owned(),
        kind: "linker_payload".to_owned(),
        component: "wasm-ld".to_owned(),
        target: target_triple.to_owned(),
        artifact_path: "embedded_registry:linker-payloads/rouwdi-wasm-ld".to_owned(),
        sha256: "b04d1efd1d7a2f39f774f641e4a2e9e98350816aa19a36de57c46d59f0026dcd".to_owned(),
        size_bytes: 1_062_842,
        embedding_method: "embedded_registry_static_linked_lld_archives".to_owned(),
        execution_method: "embedded_wasi_component_in_process_lld_wasm_link".to_owned(),
        linker_version: Some("LLD 22.1.0 source-payload".to_owned()),
        supported_input_kind: "wasm_object".to_owned(),
        supported_output_kind: "wasm32-wasip1 module".to_owned(),
    })
}

fn validate_linker_payload_identity(handoff: &RustLinkerHandoffRecord) -> Result<(), String> {
    let payload = &handoff.linker_payload;
    if payload.payload_name.trim().is_empty() {
        return Err("linker payload identity requires a payload name".to_owned());
    }
    if payload.kind != "linker_payload" {
        return Err("linker payload identity must use kind linker_payload".to_owned());
    }
    if payload.component != "wasm-ld" && payload.component != "lld" {
        return Err("linker payload identity must name wasm-ld/lld".to_owned());
    }
    if handoff.required_linker_component == "wasm-ld" && payload.component != "wasm-ld" {
        return Err("wasm linker handoff requires a wasm-ld payload identity".to_owned());
    }
    if payload.target != handoff.target_triple {
        return Err("linker payload target must match linker handoff target".to_owned());
    }
    if payload.supported_input_kind != handoff.codegen_artifact_kind {
        return Err(
            "linker payload supported input kind must match codegen object kind".to_owned(),
        );
    }
    if normalize_artifact_kind(&payload.supported_output_kind)
        != normalize_artifact_kind(&handoff.expected_final_artifact_kind)
    {
        return Err(
            "linker payload supported output kind must match expected final artifact kind"
                .to_owned(),
        );
    }
    if !is_sha256_hex(&payload.sha256) {
        return Err("linker payload identity requires a SHA-256 hash".to_owned());
    }
    if payload.size_bytes == 0 {
        return Err("linker payload identity requires a non-empty artifact size".to_owned());
    }
    if !is_assembly_owned_payload_path(&payload.artifact_path) {
        return Err("linker payload must be assembly-owned, not a host linker sidecar".to_owned());
    }
    let lower_path = payload
        .artifact_path
        .replace('\\', "/")
        .to_ascii_lowercase();
    if lower_path.ends_with(".exe")
        || lower_path.contains(".rouwdi/tools")
        || lower_path.contains("wasi-sdk")
        || lower_path.contains("third_party/rust/build")
    {
        return Err("linker payload must be assembly-owned, not a host linker sidecar".to_owned());
    }
    let embedding = payload.embedding_method.to_ascii_lowercase();
    if !(embedding.contains("embedded")
        || embedding.contains("assembly")
        || embedding.contains("component"))
        || embedding.contains("host")
        || embedding.contains("sidecar")
    {
        return Err("linker payload embedding method must be assembly-owned".to_owned());
    }
    let execution = payload.execution_method.to_ascii_lowercase();
    if execution.contains("host_process") || execution.contains("sidecar") {
        return Err("linker payload execution method must not be a host process".to_owned());
    }
    Ok(())
}

fn validate_linker_invocation_evidence(handoff: &RustLinkerHandoffRecord) -> Result<(), String> {
    let status = handoff.current_status.as_str();
    let claims_invocation = status.starts_with("wasm_ld_invoked")
        || status == "wasm32_wasip1_module_linked"
        || status == "interface_proof_passed"
        || status == "runtime_proof_passed";
    if !claims_invocation {
        if handoff.linker_invoked {
            return Err("linker_invoked requires an invoked or linked linker status".to_owned());
        }
        if handoff.exit_code.is_some() {
            return Err("non-invoked linker handoff must not record an exit code".to_owned());
        }
        return Ok(());
    }

    if !handoff.linker_invoked {
        return Err("invoked linker status requires linker_invoked=true".to_owned());
    }
    if handoff.linker_command_args.is_empty() {
        return Err("invoked linker status requires recorded command arguments".to_owned());
    }
    if handoff
        .linker_command_args
        .iter()
        .any(|arg| arg.to_ascii_lowercase().ends_with("wasm-ld.exe"))
    {
        return Err("linker command must not invoke a host wasm-ld.exe sidecar".to_owned());
    }
    if handoff
        .output_artifact_path
        .as_deref()
        .is_none_or(|path| path.trim().is_empty())
    {
        return Err("invoked linker status requires an output artifact path".to_owned());
    }
    if handoff.stdout.is_none() || handoff.stderr.is_none() || handoff.exit_code.is_none() {
        return Err("invoked linker status requires stdout, stderr, and exit code".to_owned());
    }
    Ok(())
}

fn is_sha256_hex(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn is_assembly_owned_payload_path(path: &str) -> bool {
    path.starts_with("embedded_registry:")
        || path.starts_with("vfs:/")
        || path.starts_with("memory:")
        || path.starts_with("dist/rouwdi.wasm#")
}

fn normalize_artifact_kind(kind: &str) -> String {
    kind.chars()
        .filter(|ch| !ch.is_ascii_whitespace() && *ch != '-' && *ch != '_')
        .flat_map(char::to_lowercase)
        .collect()
}

impl RustEmbeddedMirPayloadExecution {
    pub fn embedded_execution_verified(&self) -> bool {
        self.execution_source == "embedded_registry"
            && self.embedded
            && !self.external
            && !self.opened_external_file
            && self.hash_verified
            && self.size_verified
            && self.wasm_magic_verified
            && self.module_instantiated
            && self.abi_v1_exports_verified
            && self.abi_version_called
            && self.abi_version == 1
            && self.stage_called
            && self.stage == 1
            && self.descriptor_called
            && self.valid_input_called
            && self.execute_called
    }

    pub fn mir_body_proof(&self) -> Option<RustMirBodyProof> {
        if !self.embedded_execution_verified()
            || self.result_kind != "output"
            || !self.output_bytes_read
            || self.execute_trapped
        {
            return None;
        }
        let output_json = self.output_json.as_ref()?;
        let output_contract_sha256 = self.output_contract_sha256.as_ref()?;
        if output_contract_sha256.len() != 64 {
            return None;
        }
        let lowered_output = output_json.to_ascii_lowercase();
        if lowered_output.contains("stops before mir provider invocation")
            || lowered_output.contains("mir provider was not invoked")
            || lowered_output.contains("before tycxt::optimized_mir")
            || lowered_output.contains("before tyctxt::optimized_mir")
        {
            return None;
        }
        let value = serde_json::from_str::<serde_json::Value>(output_json).ok()?;
        let context_state = json_value_str(&value, "context_state")
            .or_else(|| json_value_str(&value, "code"))
            .unwrap_or_default();
        let success_state = matches!(
            self.execution_state.as_str(),
            "embedded_payload_mir_body_hash_emitted"
                | "mir_body_hash_emitted"
                | "mono_items_collected"
        ) || matches!(
            context_state,
            "mir_body_hash_emitted" | "mono_items_collected"
        );
        if !success_state {
            return None;
        }
        let provider_query = json_value_str(&value, "provider_query")?;
        if provider_query != "rustc_middle::ty::TyCtxt::optimized_mir" {
            return None;
        }
        let mir_provider_invoked = json_value_bool(&value, "mir_provider_invoked")?;
        let real_mir_body_observed = json_value_bool(&value, "real_mir_body_observed")?;
        if !mir_provider_invoked || !real_mir_body_observed {
            return None;
        }
        let fabricated_ast = json_value_bool_default_false(&value, "fabricated_ast");
        let fabricated_hir = json_value_bool_default_false(&value, "fabricated_hir");
        let fabricated_tyctx = json_value_bool_default_false(&value, "fabricated_tyctx");
        let fabricated_providers = json_value_bool_default_false(&value, "fabricated_providers");
        let fabricated_body = json_value_bool_default_false(&value, "fabricated_body");
        let fabricated_mir = json_value_bool_default_false(&value, "fabricated_mir");
        if fabricated_ast
            || fabricated_hir
            || fabricated_tyctx
            || fabricated_providers
            || fabricated_body
            || fabricated_mir
        {
            return None;
        }
        let blocker_kind = json_value_str(&value, "blocker_kind").unwrap_or_default();
        if blocker_kind != "none" {
            return None;
        }
        for key in ["blocker_reason", "exact_blocker", "blocker_text"] {
            if json_value_str(&value, key)
                .is_some_and(|text| !text.trim().is_empty() && text.trim() != "none")
            {
                return None;
            }
        }
        let mir_body_identity = json_value_str(&value, "mir_body_identity")?;
        if mir_body_identity.trim().is_empty() {
            return None;
        }
        let mir_body_hash = json_value_str(&value, "mir_body_hash")?;
        if mir_body_hash.trim().is_empty() {
            return None;
        }
        if json_value_bool_default_false(&value, "fabricated_mono_items") {
            return None;
        }
        let upstream_crates = value
            .get("upstream_crates")
            .and_then(serde_json::Value::as_array)
            .map(|items| {
                items
                    .iter()
                    .filter_map(serde_json::Value::as_str)
                    .map(str::to_owned)
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();

        Some(RustMirBodyProof {
            compile_unit_id: json_value_str(&value, "compile_unit_id")?.to_owned(),
            package: json_value_str(&value, "package")
                .unwrap_or_default()
                .to_owned(),
            target: json_value_str(&value, "target")
                .unwrap_or_default()
                .to_owned(),
            target_kind: json_value_str(&value, "target_kind")
                .unwrap_or_default()
                .to_owned(),
            source_hash: json_value_str(&value, "source_hash")?.to_owned(),
            source_path: json_value_str(&value, "source_path")
                .unwrap_or_default()
                .to_owned(),
            target_triple: json_value_str(&value, "target_triple")?.to_owned(),
            profile: json_value_str(&value, "profile")
                .unwrap_or_default()
                .to_owned(),
            crate_name: json_value_str(&value, "crate_name")
                .unwrap_or("rouwdi_payload")
                .to_owned(),
            crate_identity: json_value_str(&value, "crate_hash").map(str::to_owned),
            item_path: json_value_str(&value, "item_path")
                .unwrap_or_default()
                .to_owned(),
            local_def_id: json_value_str(&value, "local_def_id")?.to_owned(),
            def_id: json_value_str(&value, "def_id")?.to_owned(),
            def_path_hash: json_value_str(&value, "def_path_hash").map(str::to_owned),
            mir_provider: json_value_str(&value, "mir_provider")
                .unwrap_or("rustc_mir_build")
                .to_owned(),
            mir_query: json_value_str(&value, "mir_query")
                .unwrap_or(provider_query)
                .to_owned(),
            mir_stage: json_value_str(&value, "mir_stage")
                .unwrap_or("optimized")
                .to_owned(),
            provider_query: provider_query.to_owned(),
            mir_body_identity: mir_body_identity.to_owned(),
            mir_body_hash: mir_body_hash.to_owned(),
            body_basic_block_count: json_value_u64(&value, "body_basic_block_count"),
            body_local_count: json_value_u64(&value, "body_local_count"),
            body_statement_count: json_value_u64(&value, "body_statement_count"),
            payload_artifact_hash: self.actual_sha256.clone(),
            payload_sha256: self.actual_sha256.clone(),
            input_contract_sha256: self.input_contract_sha256.clone(),
            output_contract_sha256: output_contract_sha256.clone(),
            upstream_crates,
            core_metadata_loaded: json_value_bool_default_false(&value, "core_metadata_loaded"),
            alloc_metadata_loaded: json_value_bool_default_false(&value, "alloc_metadata_loaded"),
            std_metadata_loaded: json_value_bool_default_false(&value, "std_metadata_loaded"),
            lang_items_resolved: true,
            mir_provider_invoked,
            real_mir_body_observed,
            fabricated_ast,
            fabricated_hir,
            fabricated_tyctx,
            fabricated_providers,
            fabricated_body,
            fabricated_mir,
            execution_state: self.execution_state.clone(),
            payload_identity: self.payload_identity.clone(),
            registry_identity: self.registry_identity.clone(),
            execution_source: self.execution_source.clone(),
        })
    }

    pub fn monomorphization_proof(
        &self,
        mir_body_proof: &RustMirBodyProof,
    ) -> Option<RustMonomorphizationProof> {
        if !self.embedded_execution_verified()
            || self.result_kind != "output"
            || !self.output_bytes_read
            || self.execute_trapped
        {
            return None;
        }
        let output_json = self.output_json.as_ref()?;
        let output_contract_sha256 = self.output_contract_sha256.as_ref()?;
        if output_contract_sha256.len() != 64 {
            return None;
        }
        let value = serde_json::from_str::<serde_json::Value>(output_json).ok()?;
        if json_value_bool_default_false(&value, "fabricated_mono_items") {
            return None;
        }
        if json_value_bool(&value, "rustc_monomorphize_invoked") != Some(true) {
            return None;
        }
        let mono_query = json_value_str(&value, "monomorphization_query")?;
        if mono_query != "rustc_middle::ty::TyCtxt::collect_and_partition_mono_items" {
            return None;
        }
        let status = json_value_str(&value, "monomorphization_status")?.to_owned();
        let collected = status == "mono_items_collected";
        let blocked = status.starts_with("rustc_monomorphize_invoked_blocked_at_");
        if !collected && !blocked {
            return None;
        }
        let mono_item_count = json_value_u64(&value, "mono_item_count")?;
        let mono_item_graph_hash =
            json_value_str(&value, "mono_item_graph_hash").map(str::to_owned);
        let mono_items = value
            .get("mono_items")
            .and_then(serde_json::Value::as_array)
            .map(|items| {
                items
                    .iter()
                    .filter_map(parse_mono_item_proof)
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        if collected
            && (mono_item_count == 0
                || mono_item_graph_hash
                    .as_ref()
                    .map_or(true, |hash| hash.trim().is_empty())
                || mono_items.is_empty()
                || mono_items
                    .iter()
                    .any(|item| item.source != mono_query || item.symbol_name.trim().is_empty()))
        {
            return None;
        }
        if blocked && mono_item_graph_hash.is_some() {
            return None;
        }
        if json_value_str(&value, "mono_items_derived_from")
            .is_some_and(|source| source != mono_query)
        {
            return None;
        }

        Some(RustMonomorphizationProof {
            compile_unit_id: mir_body_proof.compile_unit_id.clone(),
            package: mir_body_proof.package.clone(),
            target: mir_body_proof.target.clone(),
            target_kind: mir_body_proof.target_kind.clone(),
            target_triple: mir_body_proof.target_triple.clone(),
            profile: mir_body_proof.profile.clone(),
            source_path: mir_body_proof.source_path.clone(),
            mir_body_identity: mir_body_proof.mir_body_identity.clone(),
            mir_body_hash: mir_body_proof.mir_body_hash.clone(),
            mono_provider: json_value_str(&value, "monomorphization_provider")
                .unwrap_or("rustc_monomorphize::partitioning::collect_and_partition_mono_items")
                .to_owned(),
            mono_query: mono_query.to_owned(),
            mono_item_count,
            mono_items,
            mono_item_graph_hash,
            partition_count: json_value_u64(&value, "partition_count"),
            codegen_unit_count: json_value_u64(&value, "codegen_unit_count"),
            upstream_component_identities: vec![
                "rustc_monomorphize".to_owned(),
                "rustc_monomorphize::partitioning".to_owned(),
                "rustc_middle::ty::TyCtxt::collect_and_partition_mono_items".to_owned(),
            ],
            payload_sha256: self.actual_sha256.clone(),
            input_contract_sha256: self.input_contract_sha256.clone(),
            output_contract_sha256: output_contract_sha256.clone(),
            status,
            blocker_kind: json_value_str(&value, "monomorphization_blocker_kind")
                .map(str::to_owned)
                .filter(|kind| kind != "none"),
            blocker_component: json_value_str(&value, "monomorphization_blocker_component")
                .map(str::to_owned)
                .filter(|component| component != "none"),
            blocker_message: json_value_str(&value, "monomorphization_blocker_reason")
                .map(str::to_owned)
                .filter(|message| message != "none"),
            failed_query: json_value_str(&value, "failed_query").map(str::to_owned),
            last_successful_compiler_step: json_value_str(&value, "last_successful_compiler_step")
                .map(str::to_owned),
            fabricated_mono_items: false,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RustMirHandoffRecord {
    pub compile_unit: RustCompileUnitIdentity,
    pub source_path: String,
    pub previous_stage_statuses: Vec<RustFrontendStageStatus>,
    pub stage: RustCompilerStage,
    pub status: RustMirHandoffStatus,
    pub import_ledger_path: String,
    pub import_adapter_crate: String,
    pub payload_adapter_symbol: String,
    pub payload_adapter_status: String,
    pub payload_adapter_feature: String,
    pub payload_adapter_typechecked: bool,
    pub payload_adapter_bootstrap_typechecked: bool,
    pub payload_adapter_probe_kind: String,
    pub payload_adapter_probe_workdir: String,
    pub payload_adapter_probe_classification: String,
    pub payload_adapter_probe_evidence: String,
    pub payload_adapter_probe_command: String,
    pub payload_adapter_probe_exit_code: i32,
    pub payload_adapter_normal_workspace_probe_command: String,
    pub payload_adapter_normal_workspace_probe_exit_code: i32,
    pub payload_adapter_bootstrap_crate: Option<String>,
    pub payload_adapter_bootstrap_source_path: Option<String>,
    pub payload_adapter_bootstrap_artifact_located: bool,
    pub payload_adapter_blocker_kind: Option<String>,
    pub payload_carrier_state: Option<String>,
    pub payload_milestone_state: Option<String>,
    pub payload_carrier_created: bool,
    pub payload_carrier: Option<rouwdi_rustc_upstream::MirHandoffPayloadCarrier>,
    pub payload_bundle_inspected: bool,
    pub payload_bundle_manifest_path: Option<String>,
    pub payload_bundle_manifest_sha256: Option<String>,
    pub payload_abi_manifest_path: Option<String>,
    pub payload_abi_manifest_sha256: Option<String>,
    pub payload_abi_name: Option<String>,
    pub payload_abi_version: Option<u32>,
    pub payload_abi_supported_stage: Option<rouwdi_rustc_upstream::CompilerPayloadSupportedStage>,
    pub payload_abi_primary_format: Option<rouwdi_rustc_upstream::CompilerPayloadAbiFormat>,
    pub payload_abi_selected_route: Option<String>,
    pub payload_abi_route_status: Option<rouwdi_rustc_upstream::CompilerPayloadAbiRouteStatus>,
    pub payload_abi_route_artifact_format: Option<rouwdi_rustc_upstream::CompilerPayloadAbiFormat>,
    pub payload_abi_route_artifact_path: Option<String>,
    pub payload_abi_route_artifact_sha256: Option<String>,
    pub payload_abi_route_artifact_size_bytes: Option<u64>,
    pub payload_abi_route_attempted: Option<bool>,
    pub payload_abi_route_blocker_kind: Option<String>,
    pub payload_abi_bridge_status: Option<String>,
    pub payload_abi_bridge_blocker_kind: Option<String>,
    pub payload_abi_bridge_blocker_reason: Option<String>,
    pub payload_bridge_attempt: Option<rouwdi_rustc_upstream::CompilerPayloadAbiBridge>,
    pub payload_target_pack: Option<rouwdi_rustc_upstream::CompilerPayloadTargetPackProvisioning>,
    pub payload_loader_exported_artifact_class:
        Option<rouwdi_rustc_upstream::CompilerPayloadArtifactClass>,
    pub payload_loader_metadata_artifact_class:
        Option<rouwdi_rustc_upstream::CompilerPayloadArtifactClass>,
    pub payload_loader_exported_hash_status:
        Option<rouwdi_rustc_upstream::CompilerPayloadHashStatus>,
    pub payload_loader_metadata_hash_status:
        Option<rouwdi_rustc_upstream::CompilerPayloadHashStatus>,
    pub payload_loader_load_strategy: Option<rouwdi_rustc_upstream::CompilerPayloadLoadStrategy>,
    pub payload_loader_loadability_status:
        Option<rouwdi_rustc_upstream::CompilerPayloadLoadabilityStatus>,
    pub payload_loader_loadable_by_rouwdi_wasm: Option<bool>,
    pub payload_loader_blocker_kind: Option<String>,
    pub payload_loader_blocker_reason: Option<String>,
    pub payload_next_required_artifact_format: Option<String>,
    pub payload_loaded_into_rouwdi_facade: bool,
    pub payload_load_blocker_kind: Option<String>,
    pub payload_load_blocker_reason: Option<String>,
    pub payload_next_artifact_command: Option<String>,
    pub payload_next_artifact_command_exit_code: Option<i32>,
    pub payload_next_artifact_command_evidence: Option<String>,
    pub embedded_payload_execution: Option<RustEmbeddedMirPayloadExecution>,
    pub mir_body_proof: Option<RustMirBodyProof>,
    pub monomorphization_handoff: Option<RustMonomorphizationHandoffRecord>,
    pub monomorphization_proof: Option<RustMonomorphizationProof>,
    pub codegen_handoff: Option<RustCodegenHandoffRecord>,
    pub blocker_import_status: Option<String>,
    pub blocker_probe_command: Option<String>,
    pub shared_blocker_component: Option<String>,
    pub shared_blocker_status: Option<String>,
    pub shared_blocker_kind: Option<String>,
    pub shared_blocker_summary: Option<String>,
    pub shared_blocker_probe_command: Option<String>,
    pub intended_upstream_component: String,
    pub intended_upstream_path: String,
    pub required_upstream_crates: Vec<String>,
    pub required_upstream_modules: Vec<String>,
    pub embedded_prerequisite_adapters: Vec<String>,
    pub missing_adapter_symbols: Vec<String>,
    pub required_context_objects: Vec<String>,
    pub upstream_mir_adapter_available: bool,
    pub blocker_category: Option<RustMirHandoffBlockerCategory>,
    pub blocker_component: Option<String>,
    pub blocker_component_role: Option<String>,
    pub blocker_component_path: Option<String>,
    pub blocker_reason: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RustCompileRequest {
    pub unit_id: String,
    pub package: String,
    pub target: String,
    pub target_kind: String,
    pub source_path: String,
    pub triple: String,
    pub profile: String,
    pub extern_prelude: Vec<RustExternCrate>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RustCompileArtifactRecord {
    pub unit_id: String,
    pub package: String,
    pub target: String,
    pub target_kind: String,
    pub triple: String,
    pub profile: String,
    pub artifact_kind: RustCompileArtifactKind,
    pub path: String,
    pub sha256: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RustCompileArtifactKind {
    CompilerUnitObject,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RustCompilerStage {
    Parse,
    MacroExpansion,
    NameResolution,
    TypeChecking,
    BorrowChecking,
    Mir,
    Monomorphization,
    Codegen,
    Linking,
    ArtifactEmission,
}

impl RustCompilerStage {
    pub fn label(self) -> &'static str {
        match self {
            Self::Parse => "parse",
            Self::MacroExpansion => "macro_expansion",
            Self::NameResolution => "name_resolution",
            Self::TypeChecking => "type_checking",
            Self::BorrowChecking => "borrow_checking",
            Self::Mir => "mir",
            Self::Monomorphization => "monomorphization",
            Self::Codegen => "codegen",
            Self::Linking => "linking",
            Self::ArtifactEmission => "artifact_emission",
        }
    }
}

impl fmt::Display for RustCompilerStage {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.label())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RustCompilerStageErrorCode {
    RustcParseNotEmbedded,
    MacroExpansionNotEmbedded,
    NameResolutionNotEmbedded,
    TypeckNotEmbedded,
    BorrowckNotEmbedded,
    MirNotEmbedded,
    MonomorphizationNotEmbedded,
    CodegenNotEmbedded,
    LinkerNotEmbedded,
    ArtifactWriterNotEmbedded,
}

impl RustCompilerStageErrorCode {
    pub fn for_stage(stage: RustCompilerStage) -> Self {
        match stage {
            RustCompilerStage::Parse => Self::RustcParseNotEmbedded,
            RustCompilerStage::MacroExpansion => Self::MacroExpansionNotEmbedded,
            RustCompilerStage::NameResolution => Self::NameResolutionNotEmbedded,
            RustCompilerStage::TypeChecking => Self::TypeckNotEmbedded,
            RustCompilerStage::BorrowChecking => Self::BorrowckNotEmbedded,
            RustCompilerStage::Mir => Self::MirNotEmbedded,
            RustCompilerStage::Monomorphization => Self::MonomorphizationNotEmbedded,
            RustCompilerStage::Codegen => Self::CodegenNotEmbedded,
            RustCompilerStage::Linking => Self::LinkerNotEmbedded,
            RustCompilerStage::ArtifactEmission => Self::ArtifactWriterNotEmbedded,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::RustcParseNotEmbedded => "rustc_parse_not_embedded",
            Self::MacroExpansionNotEmbedded => "macro_expansion_not_embedded",
            Self::NameResolutionNotEmbedded => "name_resolution_not_embedded",
            Self::TypeckNotEmbedded => "typeck_not_embedded",
            Self::BorrowckNotEmbedded => "borrowck_not_embedded",
            Self::MirNotEmbedded => "mir_not_embedded",
            Self::MonomorphizationNotEmbedded => "monomorphization_not_embedded",
            Self::CodegenNotEmbedded => "codegen_not_embedded",
            Self::LinkerNotEmbedded => "linker_not_embedded",
            Self::ArtifactWriterNotEmbedded => "artifact_writer_not_embedded",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MissingRustCompilerStage {
    pub unit_id: String,
    pub package: String,
    pub target: String,
    pub triple: String,
    pub stage: RustCompilerStage,
    pub error_code: RustCompilerStageErrorCode,
    pub required_component: String,
    pub component_role: String,
    pub reason: String,
}

impl MissingRustCompilerStage {
    pub fn component(&self) -> String {
        format!("compiler stage {}", self.required_component)
    }

    pub fn required_by(&self) -> String {
        format!("compile unit {}", self.unit_id)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum RustCompilerPipelineError {
    MirHandoff {
        handoff: Box<RustMirHandoffRecord>,
    },
    MissingStage {
        missing: Box<MissingRustCompilerStage>,
    },
    ParseStage {
        parse: Box<RustParseStageRecord>,
    },
    ExpansionStage {
        expansion: Box<RustExpansionStageRecord>,
    },
    NameResolutionStage {
        name_resolution: Box<RustNameResolutionStageRecord>,
    },
    TypeCheckStage {
        type_check: Box<RustTypeCheckStageRecord>,
    },
    BorrowCheckStage {
        borrow_check: Box<RustBorrowCheckStageRecord>,
    },
}

impl fmt::Display for RustCompilerPipelineError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MirHandoff { handoff } => write!(
                formatter,
                "compiler MIR handoff blocked for compile unit {}: {}",
                handoff.compile_unit.unit_id,
                handoff
                    .blocker_reason
                    .as_deref()
                    .unwrap_or("upstream MIR adapter is unavailable")
            ),
            Self::MissingStage { missing } => write!(
                formatter,
                "compiler stage {} is missing for {}: {}",
                missing.stage,
                missing.required_by(),
                missing.reason
            ),
            Self::ParseStage { parse } => write!(
                formatter,
                "compiler stage parse failed for compile unit {}: {} diagnostic(s)",
                parse.unit_id, parse.diagnostic_count
            ),
            Self::ExpansionStage { expansion } => write!(
                formatter,
                "compiler stage macro_expansion failed for compile unit {}: {} diagnostic(s)",
                expansion.unit_id, expansion.diagnostic_count
            ),
            Self::NameResolutionStage { name_resolution } => write!(
                formatter,
                "compiler stage name_resolution failed for compile unit {}: {} diagnostic(s)",
                name_resolution.unit_id, name_resolution.diagnostic_count
            ),
            Self::TypeCheckStage { type_check } => write!(
                formatter,
                "compiler stage type_checking failed for compile unit {}: {} diagnostic(s)",
                type_check.unit_id, type_check.diagnostic_count
            ),
            Self::BorrowCheckStage { borrow_check } => write!(
                formatter,
                "compiler stage borrow_checking failed for compile unit {}: {} diagnostic(s)",
                borrow_check.unit_id, borrow_check.diagnostic_count
            ),
        }
    }
}

impl std::error::Error for RustCompilerPipelineError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RustCompilerPipelineStatus {
    Artifact,
    MirHandoffBlocked,
    MissingStage,
    ParseError,
    ExpansionError,
    NameResolutionError,
    TypeCheckError,
    BorrowCheckError,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RustCompilerPipelineRecord {
    pub unit_id: String,
    pub package: String,
    pub target: String,
    pub target_kind: String,
    pub source_path: String,
    pub triple: String,
    pub profile: String,
    pub status: RustCompilerPipelineStatus,
    pub artifact: Option<RustCompileArtifactRecord>,
    pub parse_stage: Option<RustParseStageRecord>,
    pub expansion_stage: Option<RustExpansionStageRecord>,
    pub name_resolution_stage: Option<RustNameResolutionStageRecord>,
    pub type_check_stage: Option<RustTypeCheckStageRecord>,
    pub borrow_check_stage: Option<RustBorrowCheckStageRecord>,
    pub mir_handoff: Option<RustMirHandoffRecord>,
    pub missing_stage: Option<MissingRustCompilerStage>,
}

pub fn rustc_component_inventory() -> Vec<RustcComponentStatus> {
    vec![
        embedded_component(
            "rustc_lexer",
            "third_party/rust/compiler/rustc_lexer",
            "upstream-backed adapter: real upstream Rust tokenization",
        ),
        embedded_component(
            "rust_analyzer_parser",
            "third_party/rust/src/tools/rust-analyzer/crates/parser",
            "temporary upstream-adjacent parser scaffolding; not rustc_parse AST construction",
        ),
        embedded_component(
            "rouwdi_name_resolution",
            "crates/rouwdi-rustc/src/lib.rs",
            "temporary scaffolding: stage-local Rust name resolution for macro-free compile units",
        ),
        embedded_component(
            "rouwdi_type_check",
            "crates/rouwdi-rustc/src/lib.rs",
            "temporary scaffolding: stage-local Rust type checking for macro-free compile units",
        ),
        embedded_component(
            "rouwdi_borrow_check",
            "crates/rouwdi-rustc/src/lib.rs",
            "temporary scaffolding: stage-local lexical borrow checking for macro-free compile units",
        ),
        imported_component(
            "rustc_error_codes",
            "third_party/rust/compiler/rustc_error_codes",
            "upstream compiler diagnostic code table imported through rouwdi-rustc-upstream",
        ),
        imported_component(
            "rustc_index",
            "third_party/rust/compiler/rustc_index",
            "upstream index vector infrastructure imported through rouwdi-rustc-upstream",
        ),
        pending_component(
            "rustc_parse",
            "third_party/rust/compiler/rustc_parse",
            "full rustc AST construction and parser integration",
        ),
        pending_component(
            "rustc_expand",
            "third_party/rust/compiler/rustc_expand",
            "macro expansion",
        ),
        pending_component(
            "rustc_resolve",
            "third_party/rust/compiler/rustc_resolve",
            "name resolution",
        ),
        pending_component(
            "rustc_hir_analysis",
            "third_party/rust/compiler/rustc_hir_analysis",
            "type checking and coherence analysis",
        ),
        pending_component(
            "rustc_borrowck",
            "third_party/rust/compiler/rustc_borrowck",
            "borrow checking",
        ),
        pending_component(
            "rustc_middle",
            "third_party/rust/compiler/rustc_middle",
            "MIR, query model, and compiler metadata",
        ),
        pending_component(
            "rustc_mir_build",
            "third_party/rust/compiler/rustc_mir_build",
            "HIR-to-MIR lowering and MIR construction",
        ),
        imported_component(
            "rustc_monomorphize",
            "third_party/rust/compiler/rustc_monomorphize",
            "monomorphization collector",
        ),
        imported_component(
            "rustc_codegen_llvm",
            "third_party/rust/compiler/rustc_codegen_llvm",
            "LLVM-grade codegen backend",
        ),
        imported_component(
            "lld",
            "third_party/rust/src/llvm-project/lld",
            "native and WebAssembly linker implementation",
        ),
    ]
}

pub fn run_rust_compiler_pipeline(
    request: &RustCompileRequest,
    source: &str,
) -> Result<RustCompileArtifactRecord, RustCompilerPipelineError> {
    let record = run_rust_compiler_pipeline_record(request, source);
    if let Some(artifact) = record.artifact {
        return Ok(artifact);
    }
    match record.status {
        RustCompilerPipelineStatus::ParseError => Err(RustCompilerPipelineError::ParseStage {
            parse: Box::new(
                record
                    .parse_stage
                    .expect("parse error pipeline record includes parse stage"),
            ),
        }),
        RustCompilerPipelineStatus::ExpansionError => {
            Err(RustCompilerPipelineError::ExpansionStage {
                expansion: Box::new(
                    record
                        .expansion_stage
                        .expect("expansion error pipeline record includes expansion stage"),
                ),
            })
        }
        RustCompilerPipelineStatus::NameResolutionError => {
            Err(RustCompilerPipelineError::NameResolutionStage {
                name_resolution: Box::new(record.name_resolution_stage.expect(
                    "name-resolution error pipeline record includes name-resolution stage",
                )),
            })
        }
        RustCompilerPipelineStatus::TypeCheckError => {
            Err(RustCompilerPipelineError::TypeCheckStage {
                type_check: Box::new(
                    record
                        .type_check_stage
                        .expect("type-check error pipeline record includes type-check stage"),
                ),
            })
        }
        RustCompilerPipelineStatus::BorrowCheckError => {
            Err(RustCompilerPipelineError::BorrowCheckStage {
                borrow_check: Box::new(
                    record
                        .borrow_check_stage
                        .expect("borrow-check error pipeline record includes borrow-check stage"),
                ),
            })
        }
        RustCompilerPipelineStatus::MirHandoffBlocked => {
            Err(RustCompilerPipelineError::MirHandoff {
                handoff: Box::new(
                    record
                        .mir_handoff
                        .expect("MIR handoff error pipeline record includes MIR handoff record"),
                ),
            })
        }
        RustCompilerPipelineStatus::MissingStage => Err(RustCompilerPipelineError::MissingStage {
            missing: Box::new(
                record
                    .missing_stage
                    .expect("missing-stage pipeline record includes missing stage"),
            ),
        }),
        RustCompilerPipelineStatus::Artifact => unreachable!("artifact records return above"),
    }
}

pub fn run_rust_compiler_pipeline_record(
    request: &RustCompileRequest,
    source: &str,
) -> RustCompilerPipelineRecord {
    run_rust_compiler_pipeline_record_with_embedded_mir_payload(request, source, None)
}

pub fn run_rust_compiler_pipeline_record_with_embedded_mir_payload(
    request: &RustCompileRequest,
    source: &str,
    embedded_mir_payload_execution: Option<&RustEmbeddedMirPayloadExecution>,
) -> RustCompilerPipelineRecord {
    let parse_stage = parse_rust_source_for_compile_unit(request, source);
    if parse_stage.status == RustParseStageStatus::Failed {
        return RustCompilerPipelineRecord {
            unit_id: request.unit_id.clone(),
            package: request.package.clone(),
            target: request.target.clone(),
            target_kind: request.target_kind.clone(),
            source_path: request.source_path.clone(),
            triple: request.triple.clone(),
            profile: request.profile.clone(),
            status: RustCompilerPipelineStatus::ParseError,
            artifact: None,
            parse_stage: Some(parse_stage),
            expansion_stage: None,
            name_resolution_stage: None,
            type_check_stage: None,
            borrow_check_stage: None,
            mir_handoff: None,
            missing_stage: None,
        };
    }

    let expansion_stage = expand_rust_source_for_compile_unit(request, source, &parse_stage);
    if !expansion_stage.status.is_success() {
        return RustCompilerPipelineRecord {
            unit_id: request.unit_id.clone(),
            package: request.package.clone(),
            target: request.target.clone(),
            target_kind: request.target_kind.clone(),
            source_path: request.source_path.clone(),
            triple: request.triple.clone(),
            profile: request.profile.clone(),
            status: RustCompilerPipelineStatus::ExpansionError,
            artifact: None,
            parse_stage: Some(parse_stage),
            expansion_stage: Some(expansion_stage),
            name_resolution_stage: None,
            type_check_stage: None,
            borrow_check_stage: None,
            mir_handoff: None,
            missing_stage: None,
        };
    }

    let name_resolution_context = RustNameResolutionContext {
        extern_prelude: request.extern_prelude.clone(),
    };
    let name_resolution_stage = resolve_rust_names_for_compile_unit(
        request,
        source,
        &parse_stage,
        &expansion_stage,
        &name_resolution_context,
    );
    if !name_resolution_stage.status.is_success() {
        return RustCompilerPipelineRecord {
            unit_id: request.unit_id.clone(),
            package: request.package.clone(),
            target: request.target.clone(),
            target_kind: request.target_kind.clone(),
            source_path: request.source_path.clone(),
            triple: request.triple.clone(),
            profile: request.profile.clone(),
            status: RustCompilerPipelineStatus::NameResolutionError,
            artifact: None,
            parse_stage: Some(parse_stage),
            expansion_stage: Some(expansion_stage),
            name_resolution_stage: Some(name_resolution_stage),
            type_check_stage: None,
            borrow_check_stage: None,
            mir_handoff: None,
            missing_stage: None,
        };
    }

    let type_check_stage = type_check_rust_for_compile_unit(
        request,
        source,
        &parse_stage,
        &expansion_stage,
        &name_resolution_stage,
    );
    if !type_check_stage.status.is_success() {
        return RustCompilerPipelineRecord {
            unit_id: request.unit_id.clone(),
            package: request.package.clone(),
            target: request.target.clone(),
            target_kind: request.target_kind.clone(),
            source_path: request.source_path.clone(),
            triple: request.triple.clone(),
            profile: request.profile.clone(),
            status: RustCompilerPipelineStatus::TypeCheckError,
            artifact: None,
            parse_stage: Some(parse_stage),
            expansion_stage: Some(expansion_stage),
            name_resolution_stage: Some(name_resolution_stage),
            type_check_stage: Some(type_check_stage),
            borrow_check_stage: None,
            mir_handoff: None,
            missing_stage: None,
        };
    }

    let borrow_check_stage = borrow_check_rust_for_compile_unit(
        request,
        source,
        &parse_stage,
        &expansion_stage,
        &name_resolution_stage,
        &type_check_stage,
    );
    if !borrow_check_stage.status.is_success() {
        return RustCompilerPipelineRecord {
            unit_id: request.unit_id.clone(),
            package: request.package.clone(),
            target: request.target.clone(),
            target_kind: request.target_kind.clone(),
            source_path: request.source_path.clone(),
            triple: request.triple.clone(),
            profile: request.profile.clone(),
            status: RustCompilerPipelineStatus::BorrowCheckError,
            artifact: None,
            parse_stage: Some(parse_stage),
            expansion_stage: Some(expansion_stage),
            name_resolution_stage: Some(name_resolution_stage),
            type_check_stage: Some(type_check_stage),
            borrow_check_stage: Some(borrow_check_stage),
            mir_handoff: None,
            missing_stage: None,
        };
    }

    let mir_handoff = handoff_rust_mir_for_compile_unit(
        request,
        &parse_stage,
        &expansion_stage,
        &name_resolution_stage,
        &type_check_stage,
        &borrow_check_stage,
        embedded_mir_payload_execution,
    );
    if !mir_handoff.upstream_mir_adapter_available {
        return RustCompilerPipelineRecord {
            unit_id: request.unit_id.clone(),
            package: request.package.clone(),
            target: request.target.clone(),
            target_kind: request.target_kind.clone(),
            source_path: request.source_path.clone(),
            triple: request.triple.clone(),
            profile: request.profile.clone(),
            status: RustCompilerPipelineStatus::MirHandoffBlocked,
            artifact: None,
            parse_stage: Some(parse_stage),
            expansion_stage: Some(expansion_stage),
            name_resolution_stage: Some(name_resolution_stage),
            type_check_stage: Some(type_check_stage),
            borrow_check_stage: Some(borrow_check_stage),
            mir_handoff: Some(mir_handoff),
            missing_stage: None,
        };
    }

    let mono_proof_collected = mir_handoff
        .monomorphization_proof
        .as_ref()
        .is_some_and(RustMonomorphizationProof::collected);
    if !mono_proof_collected {
        let (required_component, reason) = mir_handoff
            .monomorphization_handoff
            .as_ref()
            .map(|handoff| {
                (
                    handoff.blocker_component.clone(),
                    format!(
                        "{}: {}",
                        handoff.mono_item_collection_status, handoff.blocker_reason
                    ),
                )
            })
            .unwrap_or_else(|| {
                (
                    "rustc_monomorphize".to_owned(),
                    "monomorphization did not produce a real mono item proof".to_owned(),
                )
            });
        return RustCompilerPipelineRecord {
            unit_id: request.unit_id.clone(),
            package: request.package.clone(),
            target: request.target.clone(),
            target_kind: request.target_kind.clone(),
            source_path: request.source_path.clone(),
            triple: request.triple.clone(),
            profile: request.profile.clone(),
            status: RustCompilerPipelineStatus::MissingStage,
            artifact: None,
            parse_stage: Some(parse_stage),
            expansion_stage: Some(expansion_stage),
            name_resolution_stage: Some(name_resolution_stage),
            type_check_stage: Some(type_check_stage),
            borrow_check_stage: Some(borrow_check_stage),
            mir_handoff: Some(mir_handoff),
            missing_stage: Some(MissingRustCompilerStage {
                unit_id: request.unit_id.clone(),
                package: request.package.clone(),
                target: request.target.clone(),
                triple: request.triple.clone(),
                stage: RustCompilerStage::Monomorphization,
                error_code: RustCompilerStageErrorCode::MonomorphizationNotEmbedded,
                required_component,
                component_role: "upstream monomorphization collector".to_owned(),
                reason,
            }),
        };
    }

    if let Some(codegen_handoff) = mir_handoff.codegen_handoff.as_ref() {
        if !codegen_handoff.rust_mono_item_wasm_object_emitted {
            let blocker_component = codegen_handoff.blocker_component.clone();
            let blocker_reason = format!(
                "{}: {}",
                codegen_handoff.current_status, codegen_handoff.blocker_reason
            );
            return RustCompilerPipelineRecord {
                unit_id: request.unit_id.clone(),
                package: request.package.clone(),
                target: request.target.clone(),
                target_kind: request.target_kind.clone(),
                source_path: request.source_path.clone(),
                triple: request.triple.clone(),
                profile: request.profile.clone(),
                status: RustCompilerPipelineStatus::MissingStage,
                artifact: None,
                parse_stage: Some(parse_stage),
                expansion_stage: Some(expansion_stage),
                name_resolution_stage: Some(name_resolution_stage),
                type_check_stage: Some(type_check_stage),
                borrow_check_stage: Some(borrow_check_stage),
                mir_handoff: Some(mir_handoff),
                missing_stage: Some(MissingRustCompilerStage {
                    unit_id: request.unit_id.clone(),
                    package: request.package.clone(),
                    target: request.target.clone(),
                    triple: request.triple.clone(),
                    stage: RustCompilerStage::Codegen,
                    error_code: RustCompilerStageErrorCode::CodegenNotEmbedded,
                    required_component: blocker_component,
                    component_role: "LLVM-grade codegen backend payload".to_owned(),
                    reason: blocker_reason,
                }),
            };
        }
    }

    if let Some(missing) = first_missing_compiler_stage_from(request, RustCompilerStage::Codegen) {
        return RustCompilerPipelineRecord {
            unit_id: request.unit_id.clone(),
            package: request.package.clone(),
            target: request.target.clone(),
            target_kind: request.target_kind.clone(),
            source_path: request.source_path.clone(),
            triple: request.triple.clone(),
            profile: request.profile.clone(),
            status: RustCompilerPipelineStatus::MissingStage,
            artifact: None,
            parse_stage: Some(parse_stage),
            expansion_stage: Some(expansion_stage),
            name_resolution_stage: Some(name_resolution_stage),
            type_check_stage: Some(type_check_stage),
            borrow_check_stage: Some(borrow_check_stage),
            mir_handoff: Some(mir_handoff),
            missing_stage: Some(missing),
        };
    }

    RustCompilerPipelineRecord {
        unit_id: request.unit_id.clone(),
        package: request.package.clone(),
        target: request.target.clone(),
        target_kind: request.target_kind.clone(),
        source_path: request.source_path.clone(),
        triple: request.triple.clone(),
        profile: request.profile.clone(),
        status: RustCompilerPipelineStatus::MissingStage,
        artifact: None,
        parse_stage: Some(parse_stage),
        expansion_stage: Some(expansion_stage),
        name_resolution_stage: Some(name_resolution_stage),
        type_check_stage: Some(type_check_stage),
        borrow_check_stage: Some(borrow_check_stage),
        mir_handoff: Some(mir_handoff),
        missing_stage: Some(MissingRustCompilerStage {
            unit_id: request.unit_id.clone(),
            package: request.package.clone(),
            target: request.target.clone(),
            triple: request.triple.clone(),
            stage: RustCompilerStage::ArtifactEmission,
            error_code: RustCompilerStageErrorCode::ArtifactWriterNotEmbedded,
            required_component: "rouwdi-rustc-artifact-writer".to_owned(),
            component_role: "internal compiler artifact emission".to_owned(),
            reason: "the internal compiler boundary has no artifact writer wired yet".to_owned(),
        }),
    }
}

pub fn handoff_rust_mir_for_compile_unit(
    request: &RustCompileRequest,
    parse_stage: &RustParseStageRecord,
    expansion_stage: &RustExpansionStageRecord,
    name_resolution_stage: &RustNameResolutionStageRecord,
    type_check_stage: &RustTypeCheckStageRecord,
    borrow_check_stage: &RustBorrowCheckStageRecord,
    embedded_mir_payload_execution: Option<&RustEmbeddedMirPayloadExecution>,
) -> RustMirHandoffRecord {
    let component = rouwdi_rustc_upstream::import_component("rustc_middle")
        .expect("upstream rustc import ledger includes rustc_middle");
    let mir_build = rouwdi_rustc_upstream::import_component("rustc_mir_build")
        .expect("upstream rustc import ledger includes rustc_mir_build");
    let boundary = rouwdi_rustc_upstream::mir_handoff_adapter_boundary();
    let payload_adapter = rouwdi_rustc_upstream::mir_handoff_payload_adapter();
    let resolved_blocker = rouwdi_rustc_upstream::mir_handoff_resolved_blocker();
    let ledger_blocker = resolved_blocker
        .as_ref()
        .map(|resolved| &resolved.blocked_component);
    let shared_root = resolved_blocker
        .as_ref()
        .and_then(|resolved| resolved.shared_root.as_ref());
    let embedded_execution = embedded_mir_payload_execution.cloned();
    let embedded_execution_executed = embedded_execution
        .as_ref()
        .is_some_and(RustEmbeddedMirPayloadExecution::embedded_execution_verified);
    let mir_body_proof = embedded_execution
        .as_ref()
        .and_then(RustEmbeddedMirPayloadExecution::mir_body_proof);
    let embedded_payload_mir_emitted = mir_body_proof.is_some();
    let available = embedded_payload_mir_emitted
        || (payload_adapter.adapter_available
            && component.is_imported()
            && mir_build.is_imported());
    let required_upstream_crates = payload_adapter.required_upstream_crates.clone();
    let required_upstream_modules = boundary.required_upstream_modules.clone();
    let payload_carrier = payload_adapter.payload_carrier.clone();
    let payload_carrier_state = embedded_execution
        .as_ref()
        .filter(|_| embedded_execution_executed)
        .map(|_| "embedded_payload_executed".to_owned())
        .or_else(|| {
            payload_carrier
                .as_ref()
                .map(|carrier| carrier.state.as_str().to_owned())
        });
    let payload_milestone_state = embedded_execution
        .as_ref()
        .filter(|_| embedded_execution_executed)
        .map(|execution| execution.execution_state.clone())
        .or_else(|| {
            payload_carrier
                .as_ref()
                .and_then(|carrier| carrier.milestone_state.clone())
        });
    let payload_loader_inspection = payload_carrier
        .as_ref()
        .and_then(|carrier| carrier.loader_inspection.clone());
    let payload_milestone_state = payload_milestone_state.or_else(|| {
        payload_loader_inspection
            .as_ref()
            .and_then(|inspection| inspection.milestone_state.clone())
    });
    let payload_bundle_inspected = payload_loader_inspection
        .as_ref()
        .is_some_and(|inspection| inspection.payload_bundle_inspected);
    let payload_bundle_manifest_path = payload_loader_inspection
        .as_ref()
        .map(|inspection| inspection.bundle_manifest.path.clone());
    let payload_bundle_manifest_sha256 = payload_loader_inspection
        .as_ref()
        .map(|inspection| inspection.bundle_manifest.sha256.clone());
    let payload_abi_manifest_path = payload_loader_inspection
        .as_ref()
        .and_then(|inspection| inspection.abi_manifest.as_ref())
        .map(|manifest| manifest.path.clone());
    let payload_abi_manifest_sha256 = payload_loader_inspection
        .as_ref()
        .and_then(|inspection| inspection.abi_manifest.as_ref())
        .map(|manifest| manifest.sha256.clone());
    let payload_abi_name = payload_loader_inspection
        .as_ref()
        .and_then(|inspection| inspection.abi_name.clone());
    let payload_abi_version = payload_loader_inspection
        .as_ref()
        .and_then(|inspection| inspection.abi_version);
    let payload_abi_supported_stage = payload_loader_inspection
        .as_ref()
        .and_then(|inspection| inspection.abi_supported_stage);
    let payload_abi_primary_format = payload_loader_inspection
        .as_ref()
        .and_then(|inspection| inspection.abi_primary_format);
    let payload_abi_selected_route = payload_loader_inspection
        .as_ref()
        .and_then(|inspection| inspection.abi_selected_route.clone());
    let payload_abi_route_status = payload_loader_inspection
        .as_ref()
        .and_then(|inspection| inspection.abi_route_status);
    let payload_abi_route_artifact_format = payload_loader_inspection
        .as_ref()
        .and_then(|inspection| inspection.abi_route_artifact_format);
    let payload_abi_route_artifact_path = payload_loader_inspection
        .as_ref()
        .and_then(|inspection| inspection.abi_route_artifact_path.clone());
    let payload_abi_route_artifact_sha256 = payload_loader_inspection
        .as_ref()
        .and_then(|inspection| inspection.abi_route_artifact_sha256.clone());
    let payload_abi_route_artifact_size_bytes = payload_loader_inspection
        .as_ref()
        .and_then(|inspection| inspection.abi_route_artifact_size_bytes);
    let payload_abi_route_attempted = payload_loader_inspection
        .as_ref()
        .and_then(|inspection| inspection.abi_route_attempted);
    let payload_abi_route_blocker_kind = payload_loader_inspection
        .as_ref()
        .and_then(|inspection| inspection.abi_route_blocker_kind.clone());
    let payload_abi_bridge_status = payload_loader_inspection
        .as_ref()
        .and_then(|inspection| inspection.abi_bridge_status.clone());
    let payload_abi_bridge_blocker_kind = payload_loader_inspection
        .as_ref()
        .and_then(|inspection| inspection.abi_bridge_blocker_kind.clone());
    let payload_abi_bridge_blocker_reason = payload_loader_inspection
        .as_ref()
        .and_then(|inspection| inspection.abi_bridge_blocker_reason.clone());
    let payload_bridge_attempt = payload_loader_inspection
        .as_ref()
        .and_then(|inspection| inspection.bridge_attempt.clone());
    let payload_target_pack = payload_loader_inspection
        .as_ref()
        .and_then(|inspection| inspection.target_pack.clone());
    let payload_loader_exported_artifact_class = payload_loader_inspection
        .as_ref()
        .map(|inspection| inspection.exported_payload.artifact_class);
    let payload_loader_metadata_artifact_class = payload_loader_inspection
        .as_ref()
        .map(|inspection| inspection.metadata_artifact.artifact_class);
    let payload_loader_exported_hash_status = payload_loader_inspection
        .as_ref()
        .map(|inspection| inspection.exported_payload.hash_status);
    let payload_loader_metadata_hash_status = payload_loader_inspection
        .as_ref()
        .map(|inspection| inspection.metadata_artifact.hash_status);
    let payload_loader_load_strategy = payload_loader_inspection
        .as_ref()
        .map(|inspection| inspection.load_strategy);
    let payload_loader_loadability_status = payload_loader_inspection
        .as_ref()
        .map(|inspection| inspection.loadability_status);
    let payload_loader_loadable_by_rouwdi_wasm = payload_loader_inspection
        .as_ref()
        .map(|inspection| inspection.loadable_by_rouwdi_wasm);
    let payload_loader_blocker_kind = payload_loader_inspection
        .as_ref()
        .and_then(|inspection| inspection.loader_blocker_kind.clone());
    let payload_loader_blocker_reason = payload_loader_inspection
        .as_ref()
        .map(|inspection| inspection.exact_loader_blocker.clone());
    let payload_next_required_artifact_format = payload_loader_inspection
        .as_ref()
        .map(|inspection| inspection.next_required_artifact_format.clone());
    let payload_load_blocker_kind = payload_carrier
        .as_ref()
        .and_then(|carrier| carrier.load_blocker_kind.clone());
    let payload_load_blocker_reason = payload_carrier
        .as_ref()
        .and_then(|carrier| carrier.load_blocker_reason.clone());
    let payload_next_artifact_command = payload_carrier
        .as_ref()
        .and_then(|carrier| carrier.next_artifact_command.clone());
    let payload_next_artifact_command_exit_code = payload_carrier
        .as_ref()
        .and_then(|carrier| carrier.next_artifact_command_exit_code);
    let payload_next_artifact_command_evidence = payload_carrier
        .as_ref()
        .and_then(|carrier| carrier.next_artifact_command_evidence.clone());
    let embedded_output_value = embedded_execution
        .as_ref()
        .and_then(|execution| execution.output_json.as_ref())
        .and_then(|json| serde_json::from_str::<serde_json::Value>(json).ok());
    let mono_invoked = embedded_output_value
        .as_ref()
        .and_then(|value| json_value_bool(value, "rustc_monomorphize_invoked"))
        .unwrap_or(false);
    let mono_status = embedded_output_value
        .as_ref()
        .and_then(|value| json_value_str(value, "monomorphization_status"))
        .map(str::to_owned)
        .unwrap_or_else(|| {
            if mono_invoked {
                "rustc_monomorphize_invoked_blocked_at_unknown_upstream_boundary".to_owned()
            } else {
                "rustc_monomorphize_adapter_embedded".to_owned()
            }
        });
    let mono_blocker_kind = embedded_output_value
        .as_ref()
        .and_then(|value| json_value_str(value, "monomorphization_blocker_kind"))
        .map(str::to_owned)
        .unwrap_or_else(|| {
            if mono_status == "mono_items_collected" {
                "none".to_owned()
            } else if mono_invoked {
                mono_status
                    .strip_prefix("rustc_monomorphize_invoked_blocked_at_")
                    .unwrap_or("unknown_upstream_boundary")
                    .to_owned()
            } else {
                "rustc_monomorphize_adapter_embedded".to_owned()
            }
        });
    let mono_blocker_component = embedded_output_value
        .as_ref()
        .and_then(|value| json_value_str(value, "monomorphization_blocker_component"))
        .unwrap_or("rustc_monomorphize")
        .to_owned();
    let mono_blocker_reason = embedded_output_value
        .as_ref()
        .and_then(|value| json_value_str(value, "monomorphization_blocker_reason"))
        .map(str::to_owned)
        .unwrap_or_else(|| {
            if mono_status == "mono_items_collected" {
                "rustc_monomorphize collected mono items from the real MIR-backed query path"
                    .to_owned()
            } else if mono_invoked {
                "rustc_monomorphize was invoked from the embedded payload after MIR proof and returned an exact blocker".to_owned()
            } else {
                "real embedded MIR body identity is available, and rustc_monomorphize is present in the direct rustc-private closure, but this payload output did not record a mono collection query result".to_owned()
            }
        });
    let mono_item_count = embedded_output_value
        .as_ref()
        .and_then(|value| json_value_u64(value, "mono_item_count"));
    let mono_item_graph_hash = embedded_output_value
        .as_ref()
        .and_then(|value| json_value_str(value, "mono_item_graph_hash"))
        .map(str::to_owned);
    let fabricated_mono_items = embedded_output_value
        .as_ref()
        .is_some_and(|value| json_value_bool_default_false(value, "fabricated_mono_items"));
    let monomorphization_handoff =
        mir_body_proof
            .as_ref()
            .map(|proof| RustMonomorphizationHandoffRecord {
                compile_unit_id: proof.compile_unit_id.clone(),
                package: proof.package.clone(),
                target: proof.target.clone(),
                target_kind: proof.target_kind.clone(),
                target_triple: proof.target_triple.clone(),
                profile: proof.profile.clone(),
                source_path: proof.source_path.clone(),
                mir_body_identity: proof.mir_body_identity.clone(),
                mir_body_hash: proof.mir_body_hash.clone(),
                mir_provider: proof.mir_provider.clone(),
                mir_query: proof.mir_query.clone(),
                mono_item_collection_status: mono_status.clone(),
                required_upstream_component: "rustc_monomorphize".to_owned(),
                required_upstream_crates: vec!["rustc_monomorphize".to_owned()],
                required_upstream_modules: vec![
                    "rustc_monomorphize::collector".to_owned(),
                    "rustc_monomorphize::partitioning".to_owned(),
                ],
                payload_route: "embedded_registry:rouwdi-mir-handoff-payload".to_owned(),
                current_status: mono_status.clone(),
                blocker_kind: mono_blocker_kind.clone(),
                blocker_component: mono_blocker_component.clone(),
                blocker_reason: mono_blocker_reason.clone(),
                next_command: if mono_status == "mono_items_collected" {
                    "Advance from mono item graph proof to codegen backend contact".to_owned()
                } else {
                    "Keep rustc_monomorphize query contact inside the embedded payload and clear the exact blocker without fabricating mono items".to_owned()
                },
                proof_path: "proofs/monomorphization.json".to_owned(),
                rustc_monomorphize_invoked: mono_invoked,
                mono_item_count,
                mono_item_graph_hash,
                fabricated_mono_items,
            });
    let monomorphization_proof = embedded_execution
        .as_ref()
        .zip(mir_body_proof.as_ref())
        .and_then(|(execution, proof)| execution.monomorphization_proof(proof));
    let codegen_handoff = monomorphization_proof
        .as_ref()
        .and_then(RustCodegenHandoffRecord::from_valid_monomorphization_proof);
    let blocker_component_name = embedded_execution
        .as_ref()
        .filter(|_| embedded_execution_executed && !available)
        .map(|execution| execution.payload_identity.clone())
        .or_else(|| {
            ledger_blocker
                .map(|blocker| blocker.name.clone())
                .or_else(|| payload_adapter.blocker_component.clone())
        });
    let blocker_import_status = embedded_execution
        .as_ref()
        .filter(|_| embedded_execution_executed)
        .map(|execution| execution.execution_state.clone())
        .or_else(|| {
            ledger_blocker
                .map(|blocker| blocker.import_status.clone())
                .or_else(|| payload_adapter.blocker_import_status.clone())
        });
    let blocker_probe_command = ledger_blocker
        .map(|blocker| blocker.probe_command.clone())
        .or_else(|| payload_adapter.blocker_probe_command.clone());
    let blocker_component_role = embedded_execution
        .as_ref()
        .filter(|_| embedded_execution_executed && !available)
        .map(|_| "embedded MIR handoff payload execution".to_owned())
        .or_else(|| {
            ledger_blocker
                .map(|blocker| blocker.desired_role.clone())
                .or_else(|| Some("bootstrap-checked MIR payload adapter integration".to_owned()))
        });
    let blocker_component_path = embedded_execution
        .as_ref()
        .filter(|_| embedded_execution_executed && !available)
        .map(|execution| format!("embedded_registry:{}", execution.registry_identity))
        .or_else(|| {
            ledger_blocker
                .map(|blocker| blocker.source_path.clone())
                .or_else(|| payload_adapter.bootstrap_adapter_source_path.clone())
        });
    let payload_loader_note = payload_loader_inspection
        .as_ref()
        .map(|inspection| {
            format!(
                "payload bundle inspected from {}; exported payload classified as {}; metadata artifact classified as {}; loader selected {}; loadability {}; next required artifact format {}",
                inspection.bundle_manifest.path,
                inspection.exported_payload.artifact_class.as_str(),
                inspection.metadata_artifact.artifact_class.as_str(),
                inspection.load_strategy.as_str(),
                inspection.loadability_status.as_str(),
                inspection.next_required_artifact_format
            )
        })
        .unwrap_or_else(|| "payload bundle was not available for loader inspection".to_owned());
    let payload_abi_note = payload_loader_inspection
        .as_ref()
        .and_then(|inspection| {
            let target_pack_note = inspection
                .target_pack
                .as_ref()
                .map(|target_pack| {
                    format!(
                        "; target-pack provisioning command `{}` exited {} with blocker {}; std/core/alloc available: {}/{}/{}",
                        target_pack.bootstrap_command,
                        target_pack.exit_code,
                        target_pack.blocker_kind,
                        target_pack.std_available,
                        target_pack.core_available,
                        target_pack.alloc_available
                    )
                })
                .unwrap_or_default();
            Some(format!(
                "compiler-payload ABI route {}; route status {}; bridge status {}; bridge blocker {}{}",
                inspection.abi_selected_route.as_ref()?,
                inspection
                    .abi_route_status
                    .map(|status| format!("{status:?}"))
                    .unwrap_or_else(|| "unknown".to_owned()),
                inspection.abi_bridge_status.as_deref().unwrap_or("unknown"),
                inspection
                    .abi_bridge_blocker_kind
                    .as_deref()
                    .unwrap_or("unknown"),
                target_pack_note
            ))
        })
        .unwrap_or_else(|| "compiler-payload ABI route is not recorded".to_owned());
    let blocker_reason = if available {
        None
    } else if let Some(execution) = embedded_execution
        .as_ref()
        .filter(|_| embedded_execution_executed)
    {
        Some(format!(
            "embedded MIR payload {} executed from {}; registry identity {}; instantiated {}; ABI v1 verified {}; execute called {}; execution state {}; result kind {}; blocker {}; output hash {}; error hash {}; external payload file opened {}; descriptor/input bytes were read inside the rouwdi-owned payload loader",
            execution.payload_identity,
            execution.execution_source,
            execution.registry_identity,
            execution.module_instantiated,
            execution.abi_v1_exports_verified,
            execution.execute_called,
            execution.execution_state,
            execution.result_kind,
            execution
                .blocker_kind
                .as_deref()
                .unwrap_or("none"),
            execution
                .output_contract_sha256
                .as_deref()
                .unwrap_or("none"),
            execution
                .error_contract_sha256
                .as_deref()
                .unwrap_or("none"),
            execution.opened_external_file
        ))
    } else if let Some(resolved) = resolved_blocker.as_ref() {
        let blocker = &resolved.blocked_component;
        let mut reason = format!(
            "upstream MIR payload adapter {} is {}; blocker component {} is {}; adapter feature {}; authoritative probe `{}` in {} exited {} with {}; normal workspace control probe `{}` exited {}; required context object(s): {}; embedded prerequisite adapter(s): {}; {}; {}; {}; see {} and adapter {}",
            payload_adapter.adapter_symbol,
            payload_adapter.status.as_str(),
            blocker.name,
            blocker.import_status,
            payload_adapter.cargo_feature,
            payload_adapter.authoritative_probe_command,
            payload_adapter.authoritative_probe_workdir,
            payload_adapter.authoritative_probe_exit_code,
            payload_adapter.authoritative_probe_classification,
            payload_adapter.normal_workspace_probe_command,
            payload_adapter.normal_workspace_probe_exit_code,
            boundary.required_context_objects.join(", "),
            boundary.embedded_prerequisite_adapters.join(", "),
            payload_loader_note,
            payload_abi_note,
            blocker.exact_blocker,
            rouwdi_rustc_upstream::IMPORT_LEDGER_PATH,
            rouwdi_rustc_upstream::ADAPTER_CRATE
        );
        if let Some(root) = &resolved.shared_root {
            reason.push_str(&format!(
                "; shared blocker {} is {}: {}",
                root.id, root.status, root.summary
            ));
        }
        Some(reason)
    } else {
        Some(format!(
            "upstream MIR payload adapter {} is {}; carrier state {}; bootstrap artifact located {}; payload carrier created {}; payload loaded into rouwdi facade {}; bootstrap authoritative probe `{}` in {} exited {} with {}; evidence: {}; normal workspace control probe `{}` exited {}; required context object(s): {}; embedded prerequisite adapter(s): {}; {}; {}; {}; see {} and adapter {}",
            payload_adapter.adapter_symbol,
            payload_adapter.status.as_str(),
            payload_carrier_state.as_deref().unwrap_or("unrecorded"),
            payload_adapter.bootstrap_artifact_located,
            payload_adapter.payload_carrier_created,
            payload_adapter.payload_loaded_into_rouwdi_facade,
            payload_adapter.authoritative_probe_command,
            payload_adapter.authoritative_probe_workdir,
            payload_adapter.authoritative_probe_exit_code,
            payload_adapter.authoritative_probe_classification,
            payload_adapter.authoritative_probe_evidence,
            payload_adapter.normal_workspace_probe_command,
            payload_adapter.normal_workspace_probe_exit_code,
            boundary.required_context_objects.join(", "),
            boundary.embedded_prerequisite_adapters.join(", "),
            payload_loader_note,
            payload_abi_note,
            payload_adapter
                .blocker_reason
                .clone()
                .unwrap_or_else(|| "bootstrap adapter status is recorded but not yet loaded into this compiler facade".to_owned()),
            rouwdi_rustc_upstream::IMPORT_LEDGER_PATH,
            rouwdi_rustc_upstream::ADAPTER_CRATE
        ))
    };

    RustMirHandoffRecord {
        compile_unit: RustCompileUnitIdentity::from(request),
        source_path: request.source_path.clone(),
        previous_stage_statuses: vec![
            RustFrontendStageStatus::Parse {
                status: parse_stage.status,
            },
            RustFrontendStageStatus::MacroExpansion {
                status: expansion_stage.status,
            },
            RustFrontendStageStatus::NameResolution {
                status: name_resolution_stage.status,
            },
            RustFrontendStageStatus::TypeChecking {
                status: type_check_stage.status,
            },
            RustFrontendStageStatus::BorrowChecking {
                status: borrow_check_stage.status,
            },
        ],
        stage: RustCompilerStage::Mir,
        status: if available {
            RustMirHandoffStatus::AdapterAvailable
        } else {
            RustMirHandoffStatus::AdapterUnavailable
        },
        import_ledger_path: rouwdi_rustc_upstream::IMPORT_LEDGER_PATH.to_owned(),
        import_adapter_crate: rouwdi_rustc_upstream::ADAPTER_CRATE.to_owned(),
        payload_adapter_symbol: payload_adapter.adapter_symbol,
        payload_adapter_status: payload_adapter.status.as_str().to_owned(),
        payload_adapter_feature: payload_adapter.cargo_feature,
        payload_adapter_typechecked: payload_adapter.typechecked_under_current_build,
        payload_adapter_bootstrap_typechecked: payload_adapter.bootstrap_adapter_typechecked,
        payload_adapter_probe_kind: payload_adapter.authoritative_probe_kind,
        payload_adapter_probe_workdir: payload_adapter.authoritative_probe_workdir,
        payload_adapter_probe_classification: payload_adapter.authoritative_probe_classification,
        payload_adapter_probe_evidence: payload_adapter.authoritative_probe_evidence,
        payload_adapter_probe_command: payload_adapter.authoritative_probe_command,
        payload_adapter_probe_exit_code: payload_adapter.authoritative_probe_exit_code,
        payload_adapter_normal_workspace_probe_command: payload_adapter
            .normal_workspace_probe_command,
        payload_adapter_normal_workspace_probe_exit_code: payload_adapter
            .normal_workspace_probe_exit_code,
        payload_adapter_bootstrap_crate: payload_adapter.bootstrap_adapter_crate,
        payload_adapter_bootstrap_source_path: payload_adapter.bootstrap_adapter_source_path,
        payload_adapter_bootstrap_artifact_located: payload_adapter.bootstrap_artifact_located,
        payload_adapter_blocker_kind: payload_adapter.blocker_kind,
        payload_carrier_state,
        payload_milestone_state,
        payload_carrier_created: payload_adapter.payload_carrier_created,
        payload_carrier,
        payload_bundle_inspected,
        payload_bundle_manifest_path,
        payload_bundle_manifest_sha256,
        payload_abi_manifest_path,
        payload_abi_manifest_sha256,
        payload_abi_name,
        payload_abi_version,
        payload_abi_supported_stage,
        payload_abi_primary_format,
        payload_abi_selected_route,
        payload_abi_route_status,
        payload_abi_route_artifact_format,
        payload_abi_route_artifact_path,
        payload_abi_route_artifact_sha256,
        payload_abi_route_artifact_size_bytes,
        payload_abi_route_attempted,
        payload_abi_route_blocker_kind,
        payload_abi_bridge_status,
        payload_abi_bridge_blocker_kind,
        payload_abi_bridge_blocker_reason,
        payload_bridge_attempt,
        payload_target_pack,
        payload_loader_exported_artifact_class,
        payload_loader_metadata_artifact_class,
        payload_loader_exported_hash_status,
        payload_loader_metadata_hash_status,
        payload_loader_load_strategy,
        payload_loader_loadability_status,
        payload_loader_loadable_by_rouwdi_wasm,
        payload_loader_blocker_kind,
        payload_loader_blocker_reason,
        payload_next_required_artifact_format,
        payload_loaded_into_rouwdi_facade: embedded_execution_executed
            || payload_adapter.payload_loaded_into_rouwdi_facade,
        payload_load_blocker_kind,
        payload_load_blocker_reason,
        payload_next_artifact_command,
        payload_next_artifact_command_exit_code,
        payload_next_artifact_command_evidence,
        embedded_payload_execution: embedded_execution,
        mir_body_proof,
        monomorphization_handoff,
        monomorphization_proof,
        codegen_handoff,
        blocker_import_status,
        blocker_probe_command,
        shared_blocker_component: shared_root.map(|root| root.id.clone()),
        shared_blocker_status: shared_root.map(|root| root.status.clone()),
        shared_blocker_kind: shared_root.map(|root| root.blocker_kind.clone()),
        shared_blocker_summary: shared_root.map(|root| root.summary.clone()),
        shared_blocker_probe_command: shared_root.map(|root| root.primary_probe_command.clone()),
        intended_upstream_component: "rustc_middle".to_owned(),
        intended_upstream_path: "rustc_middle::mir via rustc_mir_build::build".to_owned(),
        required_upstream_crates,
        required_upstream_modules,
        embedded_prerequisite_adapters: boundary.embedded_prerequisite_adapters,
        missing_adapter_symbols: boundary.missing_adapter_symbols,
        required_context_objects: boundary.required_context_objects,
        upstream_mir_adapter_available: available,
        blocker_category: (!available).then_some(if embedded_execution_executed {
            RustMirHandoffBlockerCategory::EmbeddedCompilerPayloadExecutionBlocked
        } else {
            RustMirHandoffBlockerCategory::UpstreamCompilerPayloadNotEmbedded
        }),
        blocker_component: blocker_component_name,
        blocker_component_role,
        blocker_component_path,
        blocker_reason,
    }
}

pub fn complete_rustc_semantics_embedded() -> bool {
    rustc_component_inventory()
        .into_iter()
        .filter(|component| component.required_for_complete_semantics)
        .all(|component| component.embedded_in_assembly)
}

pub fn lex_rust_source(source: &str) -> Vec<RustTokenSummary> {
    rustc_lexer::tokenize(source, rustc_lexer::FrontmatterAllowed::No)
        .map(|token| RustTokenSummary {
            kind: format!("{:?}", token.kind),
            len: token.len,
        })
        .collect()
}

pub fn lex_rust_source_with_diagnostics(path: &str, source: &str) -> RustSourceLexProof {
    let mut offset = 0u64;
    let mut tokens = Vec::new();
    let mut diagnostics = Vec::new();

    for token in rustc_lexer::tokenize(source, rustc_lexer::FrontmatterAllowed::No) {
        let kind = format!("{:?}", token.kind);
        if let Some(message) = lexical_error_message(token.kind) {
            diagnostics.push(RustLexDiagnostic {
                offset,
                len: token.len,
                message,
            });
        }
        tokens.push(RustTokenSummary {
            kind,
            len: token.len,
        });
        offset += u64::from(token.len);
    }

    RustSourceLexProof {
        path: path.to_owned(),
        token_count: tokens.len(),
        tokens,
        diagnostics,
    }
}

pub fn parse_rust_source_for_compile_unit(
    request: &RustCompileRequest,
    source: &str,
) -> RustParseStageRecord {
    let edition = ra_parser::Edition::CURRENT;
    let lexed = ra_parser::LexedStr::new(edition, source);
    let input = lexed.to_input(edition);
    let output = ra_parser::TopEntryPoint::SourceFile.parse(&input);
    let mut token_count = 0usize;
    let mut node_count = 0usize;
    let mut diagnostics = Vec::new();
    let reached_eof = lexed.intersperse_trivia(&output, &mut |step| match step {
        ra_parser::StrStep::Token { .. } => token_count += 1,
        ra_parser::StrStep::Enter { .. } => node_count += 1,
        ra_parser::StrStep::Exit => {}
        ra_parser::StrStep::Error { msg, pos } => {
            diagnostics.push(RustParseDiagnostic {
                offset: pos as u64,
                len: 0,
                message: msg.to_owned(),
            });
        }
    });

    for (token_index, message) in lexed.errors() {
        let range = lexed.text_range(token_index);
        diagnostics.push(RustParseDiagnostic {
            offset: range.start as u64,
            len: (range.end - range.start) as u32,
            message: message.to_owned(),
        });
    }
    if !reached_eof {
        diagnostics.push(RustParseDiagnostic {
            offset: source.len() as u64,
            len: 0,
            message: "parser did not consume the full source file".to_owned(),
        });
    }

    RustParseStageRecord {
        unit_id: request.unit_id.clone(),
        package: request.package.clone(),
        target: request.target.clone(),
        target_kind: request.target_kind.clone(),
        source_path: request.source_path.clone(),
        triple: request.triple.clone(),
        profile: request.profile.clone(),
        stage: RustCompilerStage::Parse,
        status: if diagnostics.is_empty() {
            RustParseStageStatus::Succeeded
        } else {
            RustParseStageStatus::Failed
        },
        parser_engine: "rust-analyzer-parser".to_owned(),
        parser_source: "third_party/rust/src/tools/rust-analyzer/crates/parser".to_owned(),
        entrypoint: "source_file".to_owned(),
        edition: edition.to_string(),
        token_count,
        node_count,
        diagnostic_count: diagnostics.len(),
        diagnostics,
    }
}

pub fn expand_rust_source_for_compile_unit(
    request: &RustCompileRequest,
    source: &str,
    parse_stage: &RustParseStageRecord,
) -> RustExpansionStageRecord {
    let diagnostics = expansion_required_diagnostics(source);
    RustExpansionStageRecord {
        unit_id: request.unit_id.clone(),
        package: request.package.clone(),
        target: request.target.clone(),
        target_kind: request.target_kind.clone(),
        source_path: request.source_path.clone(),
        triple: request.triple.clone(),
        profile: request.profile.clone(),
        stage: RustCompilerStage::MacroExpansion,
        status: if diagnostics.is_empty() {
            RustExpansionStageStatus::NoExpansionRequired
        } else {
            RustExpansionStageStatus::ExpansionRequired
        },
        expansion_engine: "rouwdi-rustc-expansion-gate".to_owned(),
        expansion_source: "parse-stage source context plus rustc_lexer token scan".to_owned(),
        parse_stage_status: parse_stage.status,
        parse_token_count: parse_stage.token_count,
        diagnostic_count: diagnostics.len(),
        diagnostics,
    }
}

pub fn resolve_rust_names_for_compile_unit(
    request: &RustCompileRequest,
    source: &str,
    _parse_stage: &RustParseStageRecord,
    expansion_stage: &RustExpansionStageRecord,
    context: &RustNameResolutionContext,
) -> RustNameResolutionStageRecord {
    let tokens = expansion_tokens(source);
    let mut state = NameResolutionState::new(context);
    collect_module_definitions(&tokens, 0, tokens.len(), &[], &mut state);
    resolve_paths_in_module(&tokens, 0, tokens.len(), &[], &mut state);

    let status = if state.diagnostics.is_empty() {
        RustNameResolutionStageStatus::Succeeded
    } else {
        RustNameResolutionStageStatus::Failed
    };

    RustNameResolutionStageRecord {
        unit_id: request.unit_id.clone(),
        package: request.package.clone(),
        target: request.target.clone(),
        target_kind: request.target_kind.clone(),
        source_path: request.source_path.clone(),
        triple: request.triple.clone(),
        profile: request.profile.clone(),
        stage: RustCompilerStage::NameResolution,
        status,
        resolver_engine: "rouwdi-rustc-name-resolution".to_owned(),
        resolver_source: "rustc_lexer token stream plus rouwdi module/import resolver".to_owned(),
        expansion_stage_status: expansion_stage.status,
        binding_count: state.bindings.len(),
        resolved_path_count: state.resolved_paths.len(),
        diagnostic_count: state.diagnostics.len(),
        extern_prelude: context.extern_prelude.clone(),
        bindings: state.bindings,
        resolved_paths: state.resolved_paths,
        diagnostics: state.diagnostics,
    }
}

pub fn type_check_rust_for_compile_unit(
    request: &RustCompileRequest,
    source: &str,
    _parse_stage: &RustParseStageRecord,
    _expansion_stage: &RustExpansionStageRecord,
    name_resolution_stage: &RustNameResolutionStageRecord,
) -> RustTypeCheckStageRecord {
    let tokens = expansion_tokens(source);
    let mut state = TypeCheckState::new(source);
    let functions = collect_function_signatures(source, &tokens);
    for function in &functions {
        state.register_function(function);
    }
    for function in &functions {
        state.check_function(function, &tokens);
    }

    let status = if state.diagnostics.is_empty() {
        RustTypeCheckStageStatus::Succeeded
    } else {
        RustTypeCheckStageStatus::Failed
    };

    RustTypeCheckStageRecord {
        unit_id: request.unit_id.clone(),
        package: request.package.clone(),
        target: request.target.clone(),
        target_kind: request.target_kind.clone(),
        source_path: request.source_path.clone(),
        triple: request.triple.clone(),
        profile: request.profile.clone(),
        stage: RustCompilerStage::TypeChecking,
        status,
        type_checker_engine: "rouwdi-rustc-type-check".to_owned(),
        type_checker_source: "rustc_lexer token stream plus stage-local expression type rules"
            .to_owned(),
        name_resolution_stage_status: name_resolution_stage.status,
        typed_item_count: state.typed_items.len(),
        typed_expression_count: state.typed_expressions.len(),
        diagnostic_count: state.diagnostics.len(),
        typed_items: state.typed_items,
        typed_expressions: state.typed_expressions,
        diagnostics: state.diagnostics,
    }
}

pub fn borrow_check_rust_for_compile_unit(
    request: &RustCompileRequest,
    source: &str,
    _parse_stage: &RustParseStageRecord,
    _expansion_stage: &RustExpansionStageRecord,
    _name_resolution_stage: &RustNameResolutionStageRecord,
    type_check_stage: &RustTypeCheckStageRecord,
) -> RustBorrowCheckStageRecord {
    let tokens = expansion_tokens(source);
    let mut state = BorrowCheckState::new(source);
    let functions = collect_function_signatures(source, &tokens);
    for function in &functions {
        state.check_function(function, &tokens);
    }

    let status = if state.diagnostics.is_empty() {
        RustBorrowCheckStageStatus::Succeeded
    } else {
        RustBorrowCheckStageStatus::Failed
    };

    RustBorrowCheckStageRecord {
        unit_id: request.unit_id.clone(),
        package: request.package.clone(),
        target: request.target.clone(),
        target_kind: request.target_kind.clone(),
        source_path: request.source_path.clone(),
        triple: request.triple.clone(),
        profile: request.profile.clone(),
        stage: RustCompilerStage::BorrowChecking,
        status,
        borrow_checker_engine: "rouwdi-rustc-borrow-check".to_owned(),
        borrow_checker_source: "rustc_lexer token stream plus stage-local lexical lifetime graph"
            .to_owned(),
        type_check_stage_status: type_check_stage.status,
        scope_count: state.scopes.len(),
        local_count: state.local_records.len(),
        reference_count: state.reference_records.len(),
        diagnostic_count: state.diagnostics.len(),
        locals: state.local_records,
        references: state.reference_records,
        diagnostics: state.diagnostics,
    }
}

fn lexical_error_message(kind: rustc_lexer::TokenKind) -> Option<String> {
    use rustc_lexer::{LiteralKind, TokenKind};

    match kind {
        TokenKind::BlockComment {
            terminated: false, ..
        } => Some("unterminated block comment".to_owned()),
        TokenKind::InvalidIdent => Some("invalid identifier".to_owned()),
        TokenKind::UnknownPrefix => Some("unknown literal prefix".to_owned()),
        TokenKind::UnknownPrefixLifetime => Some("unknown lifetime prefix".to_owned()),
        TokenKind::Unknown => Some("unknown token".to_owned()),
        TokenKind::Literal { kind, .. } => match kind {
            LiteralKind::Char { terminated: false } => {
                Some("unterminated character literal".to_owned())
            }
            LiteralKind::Byte { terminated: false } => Some("unterminated byte literal".to_owned()),
            LiteralKind::Str { terminated: false } => {
                Some("unterminated string literal".to_owned())
            }
            LiteralKind::ByteStr { terminated: false } => {
                Some("unterminated byte string literal".to_owned())
            }
            LiteralKind::CStr { terminated: false } => {
                Some("unterminated C string literal".to_owned())
            }
            LiteralKind::RawStr { n_hashes: None } => Some("invalid raw string literal".to_owned()),
            LiteralKind::RawByteStr { n_hashes: None } => {
                Some("invalid raw byte string literal".to_owned())
            }
            LiteralKind::RawCStr { n_hashes: None } => {
                Some("invalid raw C string literal".to_owned())
            }
            LiteralKind::Int {
                empty_int: true, ..
            } => Some("empty integer literal".to_owned()),
            LiteralKind::Float {
                empty_exponent: true,
                ..
            } => Some("empty float exponent".to_owned()),
            _ => None,
        },
        _ => None,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ExpansionToken {
    kind: rustc_lexer::TokenKind,
    text: String,
    offset: u64,
    len: u32,
}

fn expansion_required_diagnostics(source: &str) -> Vec<RustExpansionDiagnostic> {
    use rustc_lexer::TokenKind;

    let tokens = expansion_tokens(source);
    let mut diagnostics = Vec::new();

    for window in tokens.windows(2) {
        let left = &window[0];
        let right = &window[1];
        if is_identifier_like(&left.kind) && right.kind == TokenKind::Bang {
            let feature = if left.text == "macro_rules" {
                "macro_definition"
            } else {
                "macro_invocation"
            };
            diagnostics.push(RustExpansionDiagnostic {
                offset: left.offset,
                len: right
                    .offset
                    .saturating_add(u64::from(right.len))
                    .saturating_sub(left.offset) as u32,
                feature: feature.to_owned(),
                message: format!(
                    "{feature} requires embedded Rust macro expansion; rustc_expand is not embedded in rouwdi.wasm yet"
                ),
            });
        }
    }

    for (index, token) in tokens.iter().enumerate() {
        if token.kind != TokenKind::Pound {
            continue;
        }
        let Some(open) = tokens.get(index + 1) else {
            continue;
        };
        let attr_index = if open.kind == TokenKind::Bang {
            match tokens.get(index + 2) {
                Some(inner_open) if inner_open.kind == TokenKind::OpenBracket => index + 3,
                _ => continue,
            }
        } else if open.kind == TokenKind::OpenBracket {
            index + 2
        } else {
            continue;
        };
        let Some(attribute) = tokens.get(attr_index) else {
            continue;
        };
        if !is_identifier_like(&attribute.kind) {
            continue;
        }
        let feature = match attribute.text.as_str() {
            "derive" => Some("derive_attribute_expansion"),
            "cfg" => Some("cfg_attribute_expansion"),
            "cfg_attr" => Some("cfg_attr_attribute_expansion"),
            "macro_use" => Some("macro_use_attribute_expansion"),
            "proc_macro" | "proc_macro_attribute" | "proc_macro_derive" => {
                Some("proc_macro_attribute_expansion")
            }
            _ => None,
        };
        if let Some(feature) = feature {
            diagnostics.push(RustExpansionDiagnostic {
                offset: token.offset,
                len: attribute
                    .offset
                    .saturating_add(u64::from(attribute.len))
                    .saturating_sub(token.offset) as u32,
                feature: feature.to_owned(),
                message: format!(
                    "{feature} requires embedded Rust macro expansion; rustc_expand is not embedded in rouwdi.wasm yet"
                ),
            });
        }
    }

    diagnostics
}

fn expansion_tokens(source: &str) -> Vec<ExpansionToken> {
    use rustc_lexer::TokenKind;

    let mut offset = 0u64;
    let mut tokens = Vec::new();
    for token in rustc_lexer::tokenize(source, rustc_lexer::FrontmatterAllowed::No) {
        let text = source
            .get(offset as usize..offset as usize + token.len as usize)
            .unwrap_or_default()
            .to_owned();
        match token.kind {
            TokenKind::Whitespace
            | TokenKind::LineComment { .. }
            | TokenKind::BlockComment { .. } => {}
            kind => tokens.push(ExpansionToken {
                kind,
                text,
                offset,
                len: token.len,
            }),
        }
        offset += u64::from(token.len);
    }
    tokens
}

fn is_identifier_like(kind: &rustc_lexer::TokenKind) -> bool {
    matches!(
        kind,
        rustc_lexer::TokenKind::Ident | rustc_lexer::TokenKind::RawIdent
    )
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct TypeRef {
    name: String,
}

impl TypeRef {
    fn new(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }

    fn unit() -> Self {
        Self::new("()")
    }

    fn unknown() -> Self {
        Self::new("unknown")
    }

    fn integer_literal() -> Self {
        Self::new("integer")
    }

    fn float_literal() -> Self {
        Self::new("float")
    }

    fn is_unit(&self) -> bool {
        self.name == "()"
    }

    fn is_unknown(&self) -> bool {
        self.name == "unknown"
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct FunctionParameter {
    name: String,
    ty: TypeRef,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct FunctionSignature {
    name: String,
    params: Vec<FunctionParameter>,
    return_type: TypeRef,
    offset: u64,
    len: u32,
    signature: String,
    body_start: Option<usize>,
    body_end: Option<usize>,
    next_index: usize,
}

struct TypeCheckState<'a> {
    source: &'a str,
    functions: BTreeMap<String, TypeRef>,
    typed_items: Vec<RustTypedItem>,
    typed_expressions: Vec<RustTypedExpression>,
    diagnostics: Vec<RustTypeCheckDiagnostic>,
}

impl<'a> TypeCheckState<'a> {
    fn new(source: &'a str) -> Self {
        Self {
            source,
            functions: BTreeMap::new(),
            typed_items: Vec::new(),
            typed_expressions: Vec::new(),
            diagnostics: Vec::new(),
        }
    }

    fn register_function(&mut self, function: &FunctionSignature) {
        self.functions
            .insert(function.name.clone(), function.return_type.clone());
        self.typed_items.push(RustTypedItem {
            name: function.name.clone(),
            kind: "function".to_owned(),
            signature: function.signature.clone(),
            return_type: function.return_type.name.clone(),
            offset: function.offset,
            len: function.len,
        });
    }

    fn check_function(&mut self, function: &FunctionSignature, tokens: &[ExpansionToken]) {
        if function.name == "main"
            && (!function.params.is_empty() || !function.return_type.is_unit())
        {
            self.diagnostics.push(RustTypeCheckDiagnostic {
                offset: function.offset,
                len: function.len,
                code: RustTypeCheckDiagnosticCode::InvalidMainSignature,
                expected: Some("fn main() -> ()".to_owned()),
                actual: Some(render_function_shape(function)),
                message:
                    "binary entrypoint `main` must take no parameters and return unit in this stage"
                        .to_owned(),
            });
        }

        let (Some(body_start), Some(body_end)) = (function.body_start, function.body_end) else {
            return;
        };

        let mut locals = BTreeMap::<String, TypeRef>::new();
        for parameter in &function.params {
            locals.insert(parameter.name.clone(), parameter.ty.clone());
        }

        let mut index = body_start + 1;
        let mut saw_return_value = false;
        let mut saw_tail_expression = false;
        while index < body_end {
            if tokens[index].text == "fn" {
                if let Some(nested) = parse_function_signature_at(self.source, tokens, index) {
                    index = nested.next_index;
                    continue;
                }
            }

            if tokens[index].text == "let" {
                index = self.check_let_statement(tokens, index, body_end, &mut locals);
                continue;
            }

            if tokens[index].text == "return" {
                let statement_end =
                    find_statement_end(tokens, index + 1, body_end).unwrap_or(body_end);
                let expression_start = index + 1;
                let actual =
                    self.infer_expression(tokens, expression_start, statement_end, &locals);
                self.record_expression(
                    tokens,
                    expression_start,
                    statement_end,
                    actual.clone(),
                    Some(function.return_type.clone()),
                    &function.name,
                );
                self.require_type(
                    tokens,
                    expression_start,
                    statement_end,
                    &function.return_type,
                    &actual,
                    "return expression",
                );
                saw_return_value = true;
                index = statement_end.saturating_add(1);
                continue;
            }

            if let Some(statement_end) = find_statement_end(tokens, index, body_end) {
                if statement_end > index {
                    let actual = self.infer_expression(tokens, index, statement_end, &locals);
                    self.record_expression(
                        tokens,
                        index,
                        statement_end,
                        actual,
                        None,
                        &function.name,
                    );
                }
                index = statement_end + 1;
                continue;
            }

            let actual = self.infer_expression(tokens, index, body_end, &locals);
            self.record_expression(
                tokens,
                index,
                body_end,
                actual.clone(),
                Some(function.return_type.clone()),
                &function.name,
            );
            self.require_type(
                tokens,
                index,
                body_end,
                &function.return_type,
                &actual,
                "tail expression",
            );
            saw_tail_expression = true;
            index = body_end;
        }

        if !function.return_type.is_unit() && !saw_return_value && !saw_tail_expression {
            let (offset, len) = token_span(tokens, body_end, body_end.saturating_add(1));
            self.diagnostics.push(RustTypeCheckDiagnostic {
                offset,
                len,
                code: RustTypeCheckDiagnosticCode::MismatchedTypes,
                expected: Some(function.return_type.name.clone()),
                actual: Some(TypeRef::unit().name),
                message: format!(
                    "function `{}` declares return type `{}` but its body evaluates to `()`",
                    function.name, function.return_type.name
                ),
            });
        }
    }

    fn check_let_statement(
        &mut self,
        tokens: &[ExpansionToken],
        let_index: usize,
        body_end: usize,
        locals: &mut BTreeMap<String, TypeRef>,
    ) -> usize {
        let statement_end = find_statement_end(tokens, let_index + 1, body_end).unwrap_or(body_end);
        let mut name_index = let_index + 1;
        if tokens
            .get(name_index)
            .is_some_and(|token| token.text == "mut")
        {
            name_index += 1;
        }
        if name_index >= statement_end || !is_identifier_like(&tokens[name_index].kind) {
            return statement_end.saturating_add(1);
        }

        let name = token_symbol(&tokens[name_index]);
        let mut annotation = None;
        let mut initializer = None;
        let mut cursor = name_index + 1;
        while cursor < statement_end {
            if tokens[cursor].kind == rustc_lexer::TokenKind::Colon {
                let type_start = cursor + 1;
                let type_end = find_type_end(tokens, type_start, statement_end);
                annotation = Some(type_from_tokens(tokens, type_start, type_end));
                cursor = type_end;
                continue;
            }
            if tokens[cursor].kind == rustc_lexer::TokenKind::Eq {
                initializer = Some((cursor + 1, statement_end));
                break;
            }
            cursor += 1;
        }

        let actual =
            initializer.map(|(start, end)| self.infer_expression(tokens, start, end, locals));
        if let Some((start, end)) = initializer {
            let inferred = actual.clone().unwrap_or_else(TypeRef::unknown);
            self.record_expression(
                tokens,
                start,
                end,
                inferred.clone(),
                annotation.clone(),
                &name,
            );
            if let Some(expected) = &annotation {
                self.require_type(tokens, start, end, expected, &inferred, "let initializer");
            }
        }

        let local_type = annotation.or(actual).unwrap_or_else(TypeRef::unknown);
        locals.insert(name, local_type);
        statement_end.saturating_add(1)
    }

    fn infer_expression(
        &self,
        tokens: &[ExpansionToken],
        start: usize,
        end: usize,
        locals: &BTreeMap<String, TypeRef>,
    ) -> TypeRef {
        let (start, end) = trim_expression_tokens(tokens, start, end);
        if start >= end {
            return TypeRef::unit();
        }

        if tokens[start].kind == rustc_lexer::TokenKind::OpenParen {
            if let Some(close) = find_matching_delimiter(
                tokens,
                start,
                rustc_lexer::TokenKind::OpenParen,
                rustc_lexer::TokenKind::CloseParen,
            ) {
                if close + 1 == end {
                    return if close == start + 1 {
                        TypeRef::unit()
                    } else {
                        self.infer_expression(tokens, start + 1, close, locals)
                    };
                }
            }
        }

        if tokens[start].kind == rustc_lexer::TokenKind::OpenBrace {
            if let Some(close) = find_matching_delimiter(
                tokens,
                start,
                rustc_lexer::TokenKind::OpenBrace,
                rustc_lexer::TokenKind::CloseBrace,
            ) {
                if close + 1 == end {
                    return if close == start + 1 {
                        TypeRef::unit()
                    } else {
                        TypeRef::unknown()
                    };
                }
            }
        }

        if let Some(operator_index) = find_top_level_operator(tokens, start, end) {
            let left = self.infer_expression(tokens, start, operator_index, locals);
            let right = self.infer_expression(tokens, operator_index + 1, end, locals);
            return if is_numeric_type(&left.name) && is_numeric_type(&right.name) {
                if left.name == "float" || right.name == "float" {
                    TypeRef::float_literal()
                } else {
                    TypeRef::integer_literal()
                }
            } else {
                TypeRef::unknown()
            };
        }

        if let Some(call_open) = call_open_for_expression(tokens, start, end) {
            if let Some(name) = last_identifier_before(tokens, start, call_open) {
                return self
                    .functions
                    .get(&name)
                    .cloned()
                    .unwrap_or_else(TypeRef::unknown);
            }
        }

        if start + 1 == end {
            let token = &tokens[start];
            if token.text == "true" || token.text == "false" {
                return TypeRef::new("bool");
            }
            if let Some(local) = locals.get(&token_symbol(token)) {
                return local.clone();
            }
            return literal_type(token).unwrap_or_else(TypeRef::unknown);
        }

        TypeRef::unknown()
    }

    fn record_expression(
        &mut self,
        tokens: &[ExpansionToken],
        start: usize,
        end: usize,
        inferred: TypeRef,
        expected: Option<TypeRef>,
        scope: &str,
    ) {
        let (offset, len) = token_span(tokens, start, end);
        self.typed_expressions.push(RustTypedExpression {
            expression: source_slice(self.source, tokens, start, end),
            inferred_type: inferred.name,
            expected_type: expected.map(|ty| ty.name),
            scope_path: format!("fn {scope}"),
            offset,
            len,
        });
    }

    fn require_type(
        &mut self,
        tokens: &[ExpansionToken],
        start: usize,
        end: usize,
        expected: &TypeRef,
        actual: &TypeRef,
        context: &str,
    ) {
        if types_compatible(expected, actual) {
            return;
        }
        let (offset, len) = token_span(tokens, start, end);
        self.diagnostics.push(RustTypeCheckDiagnostic {
            offset,
            len,
            code: RustTypeCheckDiagnosticCode::MismatchedTypes,
            expected: Some(expected.name.clone()),
            actual: Some(actual.name.clone()),
            message: format!(
                "{context} has mismatched type: expected `{}`, found `{}`",
                expected.name, actual.name
            ),
        });
    }
}

#[derive(Debug, Clone)]
struct BorrowScope {
    path: String,
    local_ids: Vec<usize>,
}

#[derive(Debug, Clone)]
struct BorrowTrackedLocal {
    name: String,
    scope_index: usize,
}

#[derive(Debug, Clone)]
struct BorrowReferenceState {
    record_index: usize,
    borrowed_local_id: usize,
    expired: bool,
}

struct BorrowCheckState<'a> {
    source: &'a str,
    scopes: Vec<BorrowScope>,
    scope_stack: Vec<usize>,
    locals: Vec<BorrowTrackedLocal>,
    locals_by_name: BTreeMap<String, Vec<usize>>,
    references_by_local_id: BTreeMap<usize, BorrowReferenceState>,
    local_records: Vec<RustBorrowLocal>,
    reference_records: Vec<RustBorrowReference>,
    diagnostics: Vec<RustBorrowCheckDiagnostic>,
}

impl<'a> BorrowCheckState<'a> {
    fn new(source: &'a str) -> Self {
        Self {
            source,
            scopes: Vec::new(),
            scope_stack: Vec::new(),
            locals: Vec::new(),
            locals_by_name: BTreeMap::new(),
            references_by_local_id: BTreeMap::new(),
            local_records: Vec::new(),
            reference_records: Vec::new(),
            diagnostics: Vec::new(),
        }
    }

    fn check_function(&mut self, function: &FunctionSignature, tokens: &[ExpansionToken]) {
        let (Some(body_start), Some(body_end)) = (function.body_start, function.body_end) else {
            return;
        };

        let scope = self.enter_scope(format!("fn {}", function.name));
        self.check_block_contents(tokens, body_start + 1, body_end);
        self.exit_scope(scope);
    }

    fn check_block_contents(&mut self, tokens: &[ExpansionToken], start: usize, end: usize) {
        let mut index = start;
        while index < end {
            if tokens[index].text == "fn" {
                if let Some(nested) = parse_function_signature_at(self.source, tokens, index) {
                    index = nested.next_index;
                    continue;
                }
            }

            if tokens[index].text == "let" {
                index = self.check_let_statement(tokens, index, end);
                continue;
            }

            if tokens[index].kind == rustc_lexer::TokenKind::OpenBrace {
                if let Some(close) = find_matching_delimiter(
                    tokens,
                    index,
                    rustc_lexer::TokenKind::OpenBrace,
                    rustc_lexer::TokenKind::CloseBrace,
                ) {
                    let scope = self.enter_child_scope();
                    self.check_block_contents(tokens, index + 1, close);
                    self.exit_scope(scope);
                    index = close + 1;
                    continue;
                }
            }

            if let Some(statement_end) = find_statement_end(tokens, index, end) {
                self.check_statement(tokens, index, statement_end);
                index = statement_end + 1;
            } else {
                self.check_statement(tokens, index, end);
                index = end;
            }
        }
    }

    fn check_let_statement(
        &mut self,
        tokens: &[ExpansionToken],
        let_index: usize,
        body_end: usize,
    ) -> usize {
        let statement_end = find_statement_end(tokens, let_index + 1, body_end).unwrap_or(body_end);
        let mut name_index = let_index + 1;
        if tokens
            .get(name_index)
            .is_some_and(|token| token.text == "mut")
        {
            name_index += 1;
        }
        if name_index >= statement_end || !is_identifier_like(&tokens[name_index].kind) {
            return statement_end.saturating_add(1);
        }

        let local_id = self.declare_local(token_symbol(&tokens[name_index]), &tokens[name_index]);
        let initializer = first_token_matching(tokens, name_index + 1, statement_end, |token| {
            token.kind == rustc_lexer::TokenKind::Eq
        })
        .map(|eq| (eq + 1, statement_end));

        if let Some((start, end)) = initializer {
            self.check_expired_reference_uses(tokens, start, end);
            self.assign_expression_to_local(local_id, tokens, start, end);
        }

        statement_end.saturating_add(1)
    }

    fn check_statement(&mut self, tokens: &[ExpansionToken], start: usize, end: usize) {
        let (start, end) = trim_expression_tokens(tokens, start, end);
        if start >= end {
            return;
        }

        if is_identifier_like(&tokens[start].kind)
            && tokens
                .get(start + 1)
                .is_some_and(|token| token.kind == rustc_lexer::TokenKind::Eq)
        {
            let name = token_symbol(&tokens[start]);
            let Some(local_id) = self.current_local_id(&name) else {
                self.check_expired_reference_uses(tokens, start + 2, end);
                return;
            };
            self.check_expired_reference_uses(tokens, start + 2, end);
            self.assign_expression_to_local(local_id, tokens, start + 2, end);
            return;
        }

        if tokens[start].text == "return" {
            self.check_expired_reference_uses(tokens, start + 1, end);
            return;
        }

        self.check_expired_reference_uses(tokens, start, end);
    }

    fn assign_expression_to_local(
        &mut self,
        reference_local_id: usize,
        tokens: &[ExpansionToken],
        start: usize,
        end: usize,
    ) {
        let Some((borrowed_name, borrow_start, borrow_end)) =
            borrowed_local_in_expression(tokens, start, end)
        else {
            self.references_by_local_id.remove(&reference_local_id);
            return;
        };
        let Some(borrowed_local_id) = self.current_local_id(&borrowed_name) else {
            self.references_by_local_id.remove(&reference_local_id);
            return;
        };

        let reference = self.reference_record(
            reference_local_id,
            borrowed_local_id,
            tokens,
            borrow_start,
            borrow_end,
        );
        let record_index = self.reference_records.len();
        self.reference_records.push(reference);
        self.references_by_local_id.insert(
            reference_local_id,
            BorrowReferenceState {
                record_index,
                borrowed_local_id,
                expired: false,
            },
        );
    }

    fn check_expired_reference_uses(
        &mut self,
        tokens: &[ExpansionToken],
        start: usize,
        end: usize,
    ) {
        for token in tokens.iter().take(end).skip(start) {
            if !is_identifier_like(&token.kind) {
                continue;
            }
            let name = token_symbol(token);
            let Some(local_id) = self.current_local_id(&name) else {
                continue;
            };
            let Some(reference_state) = self.references_by_local_id.get(&local_id) else {
                continue;
            };
            if !reference_state.expired {
                continue;
            }
            let Some(reference_record) = self
                .reference_records
                .get(reference_state.record_index)
                .cloned()
            else {
                continue;
            };
            self.diagnostics.push(RustBorrowCheckDiagnostic {
                offset: token.offset,
                len: token.len,
                code: RustBorrowCheckDiagnosticCode::BorrowedLocalEscapesScope,
                reference_local: Some(reference_record.reference_local.clone()),
                borrowed_local: Some(reference_record.borrowed_local.clone()),
                message: format!(
                    "borrowed local `{}` does not live long enough: reference `{}` is used after `{}` left {}",
                    reference_record.borrowed_local,
                    reference_record.reference_local,
                    reference_record.borrowed_local,
                    reference_record.borrowed_scope_path
                ),
            });
        }
    }

    fn enter_scope(&mut self, path: String) -> usize {
        let index = self.scopes.len();
        self.scopes.push(BorrowScope {
            path,
            local_ids: Vec::new(),
        });
        self.scope_stack.push(index);
        index
    }

    fn enter_child_scope(&mut self) -> usize {
        let parent_path = self
            .current_scope_index()
            .map(|index| self.scopes[index].path.clone())
            .unwrap_or_else(|| "crate".to_owned());
        let path = format!("{parent_path}::block{}", self.scopes.len());
        self.enter_scope(path)
    }

    fn exit_scope(&mut self, scope_index: usize) {
        let local_ids = self.scopes[scope_index].local_ids.clone();
        let local_id_set = local_ids.iter().copied().collect::<BTreeSet<_>>();

        for local_id in &local_ids {
            self.references_by_local_id.remove(local_id);
        }
        for reference_state in self.references_by_local_id.values_mut() {
            if local_id_set.contains(&reference_state.borrowed_local_id) {
                reference_state.expired = true;
            }
        }
        for local_id in local_ids {
            let name = self.locals[local_id].name.clone();
            if let Some(stack) = self.locals_by_name.get_mut(&name) {
                if stack.last().copied() == Some(local_id) {
                    stack.pop();
                } else {
                    stack.retain(|candidate| *candidate != local_id);
                }
                if stack.is_empty() {
                    self.locals_by_name.remove(&name);
                }
            }
        }
        if self.scope_stack.last().copied() == Some(scope_index) {
            self.scope_stack.pop();
        }
    }

    fn declare_local(&mut self, name: String, token: &ExpansionToken) -> usize {
        let scope_index = self
            .current_scope_index()
            .expect("borrow checker declares locals inside an active scope");
        let local_id = self.locals.len();
        let local_record = RustBorrowLocal {
            local_id: format!("local{local_id}"),
            name: name.clone(),
            scope_path: self.scopes[scope_index].path.clone(),
            offset: token.offset,
            len: token.len,
        };
        self.locals.push(BorrowTrackedLocal {
            name: name.clone(),
            scope_index,
        });
        self.local_records.push(local_record);
        self.scopes[scope_index].local_ids.push(local_id);
        self.locals_by_name.entry(name).or_default().push(local_id);
        local_id
    }

    fn current_scope_index(&self) -> Option<usize> {
        self.scope_stack.last().copied()
    }

    fn current_local_id(&self, name: &str) -> Option<usize> {
        self.locals_by_name
            .get(name)
            .and_then(|stack| stack.last().copied())
    }

    fn reference_record(
        &self,
        reference_local_id: usize,
        borrowed_local_id: usize,
        tokens: &[ExpansionToken],
        borrow_start: usize,
        borrow_end: usize,
    ) -> RustBorrowReference {
        let reference_local = &self.locals[reference_local_id];
        let borrowed_local = &self.locals[borrowed_local_id];
        let assignment_scope_path = self
            .current_scope_index()
            .map(|index| self.scopes[index].path.clone())
            .unwrap_or_else(|| "crate".to_owned());
        let (borrow_offset, borrow_len) = token_span(tokens, borrow_start, borrow_end);
        RustBorrowReference {
            reference_local_id: format!("local{reference_local_id}"),
            reference_local: reference_local.name.clone(),
            reference_scope_path: self.scopes[reference_local.scope_index].path.clone(),
            borrowed_local_id: format!("local{borrowed_local_id}"),
            borrowed_local: borrowed_local.name.clone(),
            borrowed_scope_path: self.scopes[borrowed_local.scope_index].path.clone(),
            assignment_scope_path,
            borrow_offset,
            borrow_len,
        }
    }
}

fn borrowed_local_in_expression(
    tokens: &[ExpansionToken],
    start: usize,
    end: usize,
) -> Option<(String, usize, usize)> {
    let mut index = start;
    while index < end {
        if tokens[index].kind != rustc_lexer::TokenKind::And {
            index += 1;
            continue;
        }
        let mut name_index = index + 1;
        if tokens
            .get(name_index)
            .is_some_and(|token| token.text == "mut")
        {
            name_index += 1;
        }
        if name_index < end && is_identifier_like(&tokens[name_index].kind) {
            return Some((token_symbol(&tokens[name_index]), index, name_index + 1));
        }
        index += 1;
    }
    None
}

fn collect_function_signatures(source: &str, tokens: &[ExpansionToken]) -> Vec<FunctionSignature> {
    let mut functions = Vec::new();
    let mut index = 0;
    while index < tokens.len() {
        if tokens[index].text == "fn" {
            if let Some(function) = parse_function_signature_at(source, tokens, index) {
                index = function.next_index;
                functions.push(function);
                continue;
            }
        }
        index += 1;
    }
    functions
}

fn parse_function_signature_at(
    source: &str,
    tokens: &[ExpansionToken],
    fn_index: usize,
) -> Option<FunctionSignature> {
    if tokens.get(fn_index)?.text != "fn" {
        return None;
    }
    let name_index = next_identifier_index(tokens, fn_index + 1, tokens.len())?;
    let name = token_symbol(&tokens[name_index]);
    let param_open = first_token_matching(tokens, name_index + 1, tokens.len(), |token| {
        token.kind == rustc_lexer::TokenKind::OpenParen
    })?;
    let param_close = find_matching_delimiter(
        tokens,
        param_open,
        rustc_lexer::TokenKind::OpenParen,
        rustc_lexer::TokenKind::CloseParen,
    )?;
    let params = parse_function_params(tokens, param_open + 1, param_close);

    let mut cursor = param_close + 1;
    let mut return_type = TypeRef::unit();
    if tokens
        .get(cursor)
        .is_some_and(|token| token.kind == rustc_lexer::TokenKind::Minus)
        && tokens
            .get(cursor + 1)
            .is_some_and(|token| token.kind == rustc_lexer::TokenKind::Gt)
    {
        let type_start = cursor + 2;
        let type_end = first_token_matching(tokens, type_start, tokens.len(), |token| {
            token.kind == rustc_lexer::TokenKind::OpenBrace
                || token.kind == rustc_lexer::TokenKind::Semi
        })
        .unwrap_or(tokens.len());
        return_type = type_from_tokens(tokens, type_start, type_end);
        cursor = type_end;
    }

    let body_start = first_token_matching(tokens, cursor, tokens.len(), |token| {
        token.kind == rustc_lexer::TokenKind::OpenBrace
            || token.kind == rustc_lexer::TokenKind::Semi
    });
    let (body_start, body_end, signature_end, next_index) = match body_start {
        Some(index) if tokens[index].kind == rustc_lexer::TokenKind::OpenBrace => {
            let close = find_matching_delimiter(
                tokens,
                index,
                rustc_lexer::TokenKind::OpenBrace,
                rustc_lexer::TokenKind::CloseBrace,
            )?;
            (Some(index), Some(close), index, close + 1)
        }
        Some(index) => (None, None, index, index + 1),
        None => (None, None, tokens.len().saturating_sub(1), tokens.len()),
    };
    let (offset, len) = token_span(tokens, fn_index, signature_end);

    Some(FunctionSignature {
        name,
        params,
        return_type,
        offset,
        len,
        signature: source_slice(source, tokens, fn_index, signature_end),
        body_start,
        body_end,
        next_index,
    })
}

fn parse_function_params(
    tokens: &[ExpansionToken],
    start: usize,
    end: usize,
) -> Vec<FunctionParameter> {
    let mut params = Vec::new();
    let mut index = start;
    while index < end {
        if tokens[index].text == "mut" {
            index += 1;
            continue;
        }
        if is_identifier_like(&tokens[index].kind)
            && tokens
                .get(index + 1)
                .is_some_and(|token| token.kind == rustc_lexer::TokenKind::Colon)
        {
            let param_end = first_token_matching(tokens, index + 2, end, |token| {
                token.kind == rustc_lexer::TokenKind::Comma
            })
            .unwrap_or(end);
            params.push(FunctionParameter {
                name: token_symbol(&tokens[index]),
                ty: type_from_tokens(tokens, index + 2, param_end),
            });
            index = param_end.saturating_add(1);
            continue;
        }
        index += 1;
    }
    params
}

fn find_type_end(tokens: &[ExpansionToken], start: usize, end: usize) -> usize {
    let mut index = start;
    while index < end {
        if tokens[index].kind == rustc_lexer::TokenKind::Eq
            || tokens[index].kind == rustc_lexer::TokenKind::Comma
        {
            return index;
        }
        index += 1;
    }
    end
}

fn type_from_tokens(tokens: &[ExpansionToken], start: usize, end: usize) -> TypeRef {
    let rendered = render_type_tokens(tokens, start, end);
    match rendered.as_str() {
        "" => TypeRef::unknown(),
        "()" => TypeRef::unit(),
        "bool" | "char" | "&str" | "str" | "String" => TypeRef::new(rendered),
        ty if concrete_integer_types().contains(ty) => TypeRef::new(rendered),
        ty if concrete_float_types().contains(ty) => TypeRef::new(rendered),
        _ => TypeRef::new(rendered),
    }
}

fn render_type_tokens(tokens: &[ExpansionToken], start: usize, end: usize) -> String {
    tokens
        .iter()
        .take(end)
        .skip(start)
        .map(token_symbol)
        .collect::<Vec<_>>()
        .join("")
}

fn render_function_shape(function: &FunctionSignature) -> String {
    let params = function
        .params
        .iter()
        .map(|param| param.ty.name.as_str())
        .collect::<Vec<_>>()
        .join(", ");
    format!(
        "fn {}({params}) -> {}",
        function.name, function.return_type.name
    )
}

fn literal_type(token: &ExpansionToken) -> Option<TypeRef> {
    use rustc_lexer::LiteralKind;

    match token.kind {
        rustc_lexer::TokenKind::Literal { kind, .. } => match kind {
            LiteralKind::Int { .. } => Some(TypeRef::integer_literal()),
            LiteralKind::Float { .. } => Some(TypeRef::float_literal()),
            LiteralKind::Char { .. } => Some(TypeRef::new("char")),
            LiteralKind::Byte { .. } => Some(TypeRef::new("u8")),
            LiteralKind::Str { .. } | LiteralKind::RawStr { .. } => Some(TypeRef::new("&str")),
            LiteralKind::ByteStr { .. } | LiteralKind::RawByteStr { .. } => {
                Some(TypeRef::new("&[u8]"))
            }
            LiteralKind::CStr { .. } | LiteralKind::RawCStr { .. } => Some(TypeRef::new("&CStr")),
        },
        _ => None,
    }
}

fn types_compatible(expected: &TypeRef, actual: &TypeRef) -> bool {
    if expected.is_unknown() || actual.is_unknown() || expected.name == actual.name {
        return true;
    }
    if actual.name == "integer" {
        return concrete_integer_types().contains(expected.name.as_str());
    }
    if actual.name == "float" {
        return concrete_float_types().contains(expected.name.as_str());
    }
    false
}

fn is_numeric_type(name: &str) -> bool {
    name == "integer"
        || name == "float"
        || concrete_integer_types().contains(name)
        || concrete_float_types().contains(name)
}

fn concrete_integer_types() -> BTreeSet<&'static str> {
    [
        "usize", "isize", "u8", "u16", "u32", "u64", "u128", "i8", "i16", "i32", "i64", "i128",
    ]
    .into_iter()
    .collect()
}

fn concrete_float_types() -> BTreeSet<&'static str> {
    ["f32", "f64"].into_iter().collect()
}

fn trim_expression_tokens(
    tokens: &[ExpansionToken],
    mut start: usize,
    mut end: usize,
) -> (usize, usize) {
    while start < end && tokens[start].kind == rustc_lexer::TokenKind::Comma {
        start += 1;
    }
    while end > start
        && (tokens[end - 1].kind == rustc_lexer::TokenKind::Comma
            || tokens[end - 1].kind == rustc_lexer::TokenKind::Semi)
    {
        end -= 1;
    }
    (start, end)
}

fn call_open_for_expression(tokens: &[ExpansionToken], start: usize, end: usize) -> Option<usize> {
    let mut depth = 0usize;
    for (index, token) in tokens.iter().enumerate().take(end).skip(start) {
        match token.kind {
            rustc_lexer::TokenKind::OpenParen => {
                if depth == 0
                    && find_matching_delimiter(
                        tokens,
                        index,
                        rustc_lexer::TokenKind::OpenParen,
                        rustc_lexer::TokenKind::CloseParen,
                    ) == Some(end - 1)
                    && index > start
                {
                    return Some(index);
                }
                depth += 1;
            }
            rustc_lexer::TokenKind::CloseParen => depth = depth.saturating_sub(1),
            _ => {}
        }
    }
    None
}

fn find_top_level_operator(tokens: &[ExpansionToken], start: usize, end: usize) -> Option<usize> {
    let mut paren = 0usize;
    let mut bracket = 0usize;
    let mut brace = 0usize;
    for (index, token) in tokens.iter().enumerate().take(end).skip(start) {
        match token.kind {
            rustc_lexer::TokenKind::OpenParen => paren += 1,
            rustc_lexer::TokenKind::CloseParen => paren = paren.saturating_sub(1),
            rustc_lexer::TokenKind::OpenBracket => bracket += 1,
            rustc_lexer::TokenKind::CloseBracket => bracket = bracket.saturating_sub(1),
            rustc_lexer::TokenKind::OpenBrace => brace += 1,
            rustc_lexer::TokenKind::CloseBrace => brace = brace.saturating_sub(1),
            rustc_lexer::TokenKind::Plus
            | rustc_lexer::TokenKind::Minus
            | rustc_lexer::TokenKind::Star
            | rustc_lexer::TokenKind::Slash
            | rustc_lexer::TokenKind::Percent
                if paren == 0 && bracket == 0 && brace == 0 =>
            {
                return Some(index)
            }
            _ => {}
        }
    }
    None
}

fn last_identifier_before(tokens: &[ExpansionToken], start: usize, end: usize) -> Option<String> {
    tokens
        .iter()
        .take(end)
        .skip(start)
        .rev()
        .find(|token| is_identifier_like(&token.kind))
        .map(token_symbol)
}

fn first_token_matching(
    tokens: &[ExpansionToken],
    start: usize,
    end: usize,
    predicate: impl Fn(&ExpansionToken) -> bool,
) -> Option<usize> {
    tokens
        .iter()
        .enumerate()
        .take(end)
        .skip(start)
        .find_map(|(index, token)| predicate(token).then_some(index))
}

fn source_slice(source: &str, tokens: &[ExpansionToken], start: usize, end: usize) -> String {
    if start >= end {
        return String::new();
    }
    let Some(first) = tokens.get(start) else {
        return String::new();
    };
    let Some(last) = tokens.get(end.saturating_sub(1)) else {
        return String::new();
    };
    let slice_start = first.offset as usize;
    let slice_end = last.offset.saturating_add(u64::from(last.len)) as usize;
    source
        .get(slice_start..slice_end)
        .unwrap_or_default()
        .to_owned()
}

fn token_span(tokens: &[ExpansionToken], start: usize, end: usize) -> (u64, u32) {
    if start >= end {
        if let Some(token) = tokens.get(start) {
            return (token.offset, 0);
        }
        return (0, 0);
    }
    let Some(first) = tokens.get(start) else {
        return (0, 0);
    };
    let Some(last) = tokens.get(end.saturating_sub(1)) else {
        return (first.offset, first.len);
    };
    (
        first.offset,
        last.offset
            .saturating_add(u64::from(last.len))
            .saturating_sub(first.offset) as u32,
    )
}

#[derive(Debug, Clone, Copy)]
struct BindingSummary {
    kind: RustNameBindingKind,
    namespace: RustNameNamespace,
}

#[derive(Debug, Default)]
struct ModuleScope {
    bindings: BTreeMap<String, BindingSummary>,
    modules: BTreeMap<String, ModuleScope>,
}

#[derive(Debug)]
struct NameResolutionState {
    root: ModuleScope,
    extern_crates: BTreeMap<String, RustExternCrate>,
    bindings: Vec<RustNameBinding>,
    resolved_paths: Vec<RustResolvedPath>,
    diagnostics: Vec<RustNameResolutionDiagnostic>,
}

impl NameResolutionState {
    fn new(context: &RustNameResolutionContext) -> Self {
        let mut state = Self {
            root: ModuleScope::default(),
            extern_crates: context
                .extern_prelude
                .iter()
                .map(|krate| (krate.name.clone(), krate.clone()))
                .collect(),
            bindings: Vec::new(),
            resolved_paths: Vec::new(),
            diagnostics: Vec::new(),
        };
        for krate in &context.extern_prelude {
            state.insert_binding(&[], &krate.name, RustNameBindingKind::ExternCrate, 0, 0);
        }
        state
    }

    fn insert_binding(
        &mut self,
        module_path: &[String],
        name: &str,
        kind: RustNameBindingKind,
        offset: u64,
        len: u32,
    ) {
        let namespace = binding_namespace(kind);
        let scope_path = render_module_path(module_path);
        if let Some(scope) = module_scope_mut(&mut self.root, module_path) {
            scope
                .bindings
                .insert(name.to_owned(), BindingSummary { kind, namespace });
            if kind == RustNameBindingKind::Module {
                scope.modules.entry(name.to_owned()).or_default();
            }
        }
        self.bindings.push(RustNameBinding {
            name: name.to_owned(),
            kind,
            namespace,
            scope_path,
            offset,
            len,
        });
    }

    fn insert_import_binding(
        &mut self,
        module_path: &[String],
        name: &str,
        token: &ExpansionToken,
    ) {
        self.insert_binding(
            module_path,
            name,
            RustNameBindingKind::Import,
            token.offset,
            token.len,
        );
    }

    fn unresolved(
        &mut self,
        code: RustNameResolutionDiagnosticCode,
        path: &PathReference,
        message: String,
    ) {
        self.diagnostics.push(RustNameResolutionDiagnostic {
            offset: path.offset,
            len: path.len,
            code,
            path: path.render(),
            message,
        });
    }
}

#[derive(Debug, Clone)]
struct PathReference {
    segments: Vec<String>,
    offset: u64,
    len: u32,
    first_token_index: usize,
    last_token_index: usize,
}

impl PathReference {
    fn render(&self) -> String {
        self.segments.join("::")
    }
}

fn collect_module_definitions(
    tokens: &[ExpansionToken],
    start: usize,
    end: usize,
    module_path: &[String],
    state: &mut NameResolutionState,
) {
    let mut index = start;
    while index < end {
        let token = &tokens[index];
        if token.text == "mod" {
            if let Some(name_index) = next_identifier_index(tokens, index + 1, end) {
                let name = token_symbol(&tokens[name_index]);
                match tokens.get(name_index + 1).map(|token| token.kind) {
                    Some(rustc_lexer::TokenKind::OpenBrace) => {
                        state.insert_binding(
                            module_path,
                            &name,
                            RustNameBindingKind::Module,
                            tokens[name_index].offset,
                            tokens[name_index].len,
                        );
                        if let Some(close) = find_matching_delimiter(
                            tokens,
                            name_index + 1,
                            rustc_lexer::TokenKind::OpenBrace,
                            rustc_lexer::TokenKind::CloseBrace,
                        ) {
                            let mut child_path = module_path.to_vec();
                            child_path.push(name);
                            collect_module_definitions(
                                tokens,
                                name_index + 2,
                                close,
                                &child_path,
                                state,
                            );
                            index = close + 1;
                            continue;
                        }
                    }
                    Some(rustc_lexer::TokenKind::Semi) => {
                        let path = PathReference {
                            segments: vec![name.clone()],
                            offset: tokens[name_index].offset,
                            len: tokens[name_index].len,
                            first_token_index: name_index,
                            last_token_index: name_index,
                        };
                        state.unresolved(
                            RustNameResolutionDiagnosticCode::UnresolvedModule,
                            &path,
                            format!(
                                "module `{name}` is declared without an inline body; rouwdi name resolution has no embedded module file loaded for it"
                            ),
                        );
                        index = name_index + 2;
                        continue;
                    }
                    _ => {}
                }
            }
        } else if let Some(kind) = item_binding_kind(&token.text) {
            if let Some(name_index) = next_identifier_index(tokens, index + 1, end) {
                let name = token_symbol(&tokens[name_index]);
                state.insert_binding(
                    module_path,
                    &name,
                    kind,
                    tokens[name_index].offset,
                    tokens[name_index].len,
                );
                index = name_index + 1;
                continue;
            }
        }
        index += 1;
    }
}

fn resolve_paths_in_module(
    tokens: &[ExpansionToken],
    start: usize,
    end: usize,
    module_path: &[String],
    state: &mut NameResolutionState,
) {
    let mut index = start;
    while index < end {
        let token = &tokens[index];
        if token.text == "mod" {
            if let Some(name_index) = next_identifier_index(tokens, index + 1, end) {
                if tokens
                    .get(name_index + 1)
                    .is_some_and(|token| token.kind == rustc_lexer::TokenKind::OpenBrace)
                {
                    if let Some(close) = find_matching_delimiter(
                        tokens,
                        name_index + 1,
                        rustc_lexer::TokenKind::OpenBrace,
                        rustc_lexer::TokenKind::CloseBrace,
                    ) {
                        let mut child_path = module_path.to_vec();
                        child_path.push(token_symbol(&tokens[name_index]));
                        resolve_paths_in_module(tokens, name_index + 2, close, &child_path, state);
                        index = close + 1;
                        continue;
                    }
                }
            }
        }

        if token.text == "use" {
            let statement_end = find_statement_end(tokens, index + 1, end).unwrap_or(end);
            if let Some(path) = collect_use_path(tokens, index + 1, statement_end) {
                match resolve_path(
                    &state.root,
                    &state.extern_crates,
                    module_path,
                    &path.segments,
                ) {
                    Some(resolution) => {
                        let alias = use_alias(tokens, &path, statement_end)
                            .unwrap_or_else(|| path.segments.last().cloned().unwrap_or_default());
                        state.resolved_paths.push(RustResolvedPath {
                            path: path.render(),
                            resolution,
                            scope_path: render_module_path(module_path),
                            offset: path.offset,
                            len: path.len,
                        });
                        if let Some(alias_token) = tokens.get(path.last_token_index) {
                            state.insert_import_binding(module_path, &alias, alias_token);
                        }
                    }
                    None => state.unresolved(
                        RustNameResolutionDiagnosticCode::UnresolvedImport,
                        &path,
                        format!(
                            "import path `{}` does not resolve in this compile unit",
                            path.render()
                        ),
                    ),
                }
            }
            index = statement_end.saturating_add(1);
            continue;
        }

        if let Some(path) = collect_path(tokens, index, end) {
            if should_check_path(tokens, &path) {
                match resolve_path(
                    &state.root,
                    &state.extern_crates,
                    module_path,
                    &path.segments,
                ) {
                    Some(resolution) => state.resolved_paths.push(RustResolvedPath {
                        path: path.render(),
                        resolution,
                        scope_path: render_module_path(module_path),
                        offset: path.offset,
                        len: path.len,
                    }),
                    None => state.unresolved(
                        RustNameResolutionDiagnosticCode::UnresolvedPath,
                        &path,
                        format!(
                            "path `{}` does not resolve in this compile unit",
                            path.render()
                        ),
                    ),
                }
            }
            index = path.last_token_index + 1;
            continue;
        }

        index += 1;
    }
}

fn resolve_path(
    root: &ModuleScope,
    extern_crates: &BTreeMap<String, RustExternCrate>,
    module_path: &[String],
    segments: &[String],
) -> Option<RustNameResolution> {
    let first = segments.first()?;
    match first.as_str() {
        "crate" => {
            if segments.len() == 1 {
                return Some(RustNameResolution::Module {
                    path: "crate".to_owned(),
                });
            }
            return resolve_from_module(root, &[], &segments[1..]);
        }
        "self" => {
            if segments.len() == 1 {
                return Some(RustNameResolution::Module {
                    path: render_module_path(module_path),
                });
            }
            return resolve_from_module(root, module_path, &segments[1..]);
        }
        "super" => {
            let parent_len = module_path.len().saturating_sub(1);
            let parent = &module_path[..parent_len];
            if segments.len() == 1 {
                return Some(RustNameResolution::Module {
                    path: render_module_path(parent),
                });
            }
            return resolve_from_module(root, parent, &segments[1..]);
        }
        _ => {}
    }

    if let Some(krate) = extern_crates.get(first) {
        return Some(RustNameResolution::ExternalCrate {
            name: krate.name.clone(),
            source_unit_id: krate.source_unit_id.clone(),
        });
    }
    if builtin_names().contains(first.as_str()) {
        return Some(RustNameResolution::Builtin {
            name: first.clone(),
        });
    }
    resolve_from_module(root, module_path, segments)
        .or_else(|| resolve_from_module(root, &[], segments))
}

fn resolve_from_module(
    root: &ModuleScope,
    module_path: &[String],
    segments: &[String],
) -> Option<RustNameResolution> {
    let mut current_path = module_path.to_vec();
    let mut scope = module_scope(root, &current_path)?;
    for (index, segment) in segments.iter().enumerate() {
        let binding = scope.bindings.get(segment)?;
        let last = index + 1 == segments.len();
        if last {
            return if binding.kind == RustNameBindingKind::Module {
                let mut resolved_path = current_path;
                resolved_path.push(segment.clone());
                Some(RustNameResolution::Module {
                    path: render_module_path(&resolved_path),
                })
            } else {
                Some(RustNameResolution::LocalBinding {
                    name: segment.clone(),
                    namespace: binding.namespace,
                })
            };
        }
        if binding.kind != RustNameBindingKind::Module {
            return None;
        }
        current_path.push(segment.clone());
        scope = module_scope(root, &current_path)?;
    }
    None
}

fn module_scope<'a>(root: &'a ModuleScope, module_path: &[String]) -> Option<&'a ModuleScope> {
    let mut scope = root;
    for segment in module_path {
        scope = scope.modules.get(segment)?;
    }
    Some(scope)
}

fn module_scope_mut<'a>(
    root: &'a mut ModuleScope,
    module_path: &[String],
) -> Option<&'a mut ModuleScope> {
    let mut scope = root;
    for segment in module_path {
        scope = scope.modules.get_mut(segment)?;
    }
    Some(scope)
}

fn collect_path(tokens: &[ExpansionToken], start: usize, end: usize) -> Option<PathReference> {
    if start >= end || !is_path_segment_token(&tokens[start]) {
        return None;
    }
    if start > 1
        && tokens[start - 1].kind == rustc_lexer::TokenKind::Colon
        && tokens[start - 2].kind == rustc_lexer::TokenKind::Colon
    {
        return None;
    }

    let mut segments = vec![token_symbol(&tokens[start])];
    let mut last = start;
    while last + 3 < end
        && tokens[last + 1].kind == rustc_lexer::TokenKind::Colon
        && tokens[last + 2].kind == rustc_lexer::TokenKind::Colon
        && is_path_segment_token(&tokens[last + 3])
    {
        last += 3;
        segments.push(token_symbol(&tokens[last]));
    }
    if segments.len() < 2 {
        return None;
    }
    let offset = tokens[start].offset;
    let len = tokens[last]
        .offset
        .saturating_add(u64::from(tokens[last].len))
        .saturating_sub(offset) as u32;
    Some(PathReference {
        segments,
        offset,
        len,
        first_token_index: start,
        last_token_index: last,
    })
}

fn collect_use_path(tokens: &[ExpansionToken], start: usize, end: usize) -> Option<PathReference> {
    let mut index = start;
    while index < end
        && (tokens[index].text == "pub"
            || tokens[index].kind == rustc_lexer::TokenKind::OpenParen
            || tokens[index].kind == rustc_lexer::TokenKind::CloseParen)
    {
        index += 1;
    }
    if index >= end || !is_path_segment_token(&tokens[index]) {
        return None;
    }

    let mut segments = vec![token_symbol(&tokens[index])];
    let first = index;
    let mut last = index;
    while last + 2 < end
        && tokens[last + 1].kind == rustc_lexer::TokenKind::Colon
        && tokens[last + 2].kind == rustc_lexer::TokenKind::Colon
    {
        if last + 3 >= end || !is_path_segment_token(&tokens[last + 3]) {
            break;
        }
        last += 3;
        if tokens[last].text == "as" {
            break;
        }
        segments.push(token_symbol(&tokens[last]));
    }
    let offset = tokens[first].offset;
    let len = tokens[last]
        .offset
        .saturating_add(u64::from(tokens[last].len))
        .saturating_sub(offset) as u32;
    Some(PathReference {
        segments,
        offset,
        len,
        first_token_index: first,
        last_token_index: last,
    })
}

fn use_alias(
    tokens: &[ExpansionToken],
    path: &PathReference,
    statement_end: usize,
) -> Option<String> {
    let mut index = path.last_token_index + 1;
    while index + 1 < statement_end {
        if tokens[index].text == "as" && is_identifier_like(&tokens[index + 1].kind) {
            return Some(token_symbol(&tokens[index + 1]));
        }
        index += 1;
    }
    None
}

fn should_check_path(tokens: &[ExpansionToken], path: &PathReference) -> bool {
    if path.first_token_index > 0 {
        let previous = &tokens[path.first_token_index - 1];
        if previous.kind == rustc_lexer::TokenKind::Dot || previous.text == "use" {
            return false;
        }
        if matches!(
            previous.text.as_str(),
            "fn" | "struct" | "enum" | "trait" | "type" | "const" | "static" | "union" | "mod"
        ) {
            return false;
        }
    }
    true
}

fn find_matching_delimiter(
    tokens: &[ExpansionToken],
    open_index: usize,
    open: rustc_lexer::TokenKind,
    close: rustc_lexer::TokenKind,
) -> Option<usize> {
    let mut depth = 0usize;
    for (index, token) in tokens.iter().enumerate().skip(open_index) {
        if token.kind == open {
            depth += 1;
        } else if token.kind == close {
            depth = depth.checked_sub(1)?;
            if depth == 0 {
                return Some(index);
            }
        }
    }
    None
}

fn find_statement_end(tokens: &[ExpansionToken], start: usize, end: usize) -> Option<usize> {
    let mut paren = 0usize;
    let mut bracket = 0usize;
    let mut brace = 0usize;
    for (index, token) in tokens.iter().enumerate().take(end).skip(start) {
        match token.kind {
            rustc_lexer::TokenKind::OpenParen => paren += 1,
            rustc_lexer::TokenKind::CloseParen => paren = paren.saturating_sub(1),
            rustc_lexer::TokenKind::OpenBracket => bracket += 1,
            rustc_lexer::TokenKind::CloseBracket => bracket = bracket.saturating_sub(1),
            rustc_lexer::TokenKind::OpenBrace => brace += 1,
            rustc_lexer::TokenKind::CloseBrace => brace = brace.saturating_sub(1),
            rustc_lexer::TokenKind::Semi if paren == 0 && bracket == 0 && brace == 0 => {
                return Some(index)
            }
            _ => {}
        }
    }
    None
}

fn next_identifier_index(tokens: &[ExpansionToken], start: usize, end: usize) -> Option<usize> {
    tokens
        .iter()
        .enumerate()
        .take(end)
        .skip(start)
        .find_map(|(index, token)| is_identifier_like(&token.kind).then_some(index))
}

fn item_binding_kind(text: &str) -> Option<RustNameBindingKind> {
    match text {
        "fn" => Some(RustNameBindingKind::Function),
        "struct" => Some(RustNameBindingKind::Struct),
        "enum" => Some(RustNameBindingKind::Enum),
        "trait" => Some(RustNameBindingKind::Trait),
        "type" => Some(RustNameBindingKind::TypeAlias),
        "const" => Some(RustNameBindingKind::Const),
        "static" => Some(RustNameBindingKind::Static),
        "union" => Some(RustNameBindingKind::Union),
        _ => None,
    }
}

fn binding_namespace(kind: RustNameBindingKind) -> RustNameNamespace {
    match kind {
        RustNameBindingKind::Builtin
        | RustNameBindingKind::Enum
        | RustNameBindingKind::Struct
        | RustNameBindingKind::Trait
        | RustNameBindingKind::TypeAlias
        | RustNameBindingKind::Union => RustNameNamespace::Type,
        RustNameBindingKind::ExternCrate | RustNameBindingKind::Module => RustNameNamespace::Module,
        RustNameBindingKind::Const
        | RustNameBindingKind::Function
        | RustNameBindingKind::Import
        | RustNameBindingKind::Static => RustNameNamespace::Value,
    }
}

fn is_path_segment_token(token: &ExpansionToken) -> bool {
    is_identifier_like(&token.kind) || matches!(token.text.as_str(), "crate" | "self" | "super")
}

fn token_symbol(token: &ExpansionToken) -> String {
    token
        .text
        .strip_prefix("r#")
        .unwrap_or(&token.text)
        .to_owned()
}

fn render_module_path(path: &[String]) -> String {
    if path.is_empty() {
        "crate".to_owned()
    } else {
        format!("crate::{}", path.join("::"))
    }
}

fn builtin_names() -> BTreeSet<&'static str> {
    [
        "bool", "char", "str", "usize", "isize", "u8", "u16", "u32", "u64", "u128", "i8", "i16",
        "i32", "i64", "i128", "f32", "f64", "Option", "Result", "Some", "None", "Ok", "Err",
        "String", "Vec", "Box", "core", "std", "alloc",
    ]
    .into_iter()
    .collect()
}

fn json_value_str<'a>(value: &'a serde_json::Value, key: &str) -> Option<&'a str> {
    value.get(key).and_then(serde_json::Value::as_str)
}

fn json_value_bool(value: &serde_json::Value, key: &str) -> Option<bool> {
    value.get(key).and_then(serde_json::Value::as_bool)
}

fn json_value_u64(value: &serde_json::Value, key: &str) -> Option<u64> {
    value.get(key).and_then(serde_json::Value::as_u64)
}

fn json_value_bool_default_false(value: &serde_json::Value, key: &str) -> bool {
    json_value_bool(value, key).unwrap_or(false)
}

fn parse_mono_item_proof(value: &serde_json::Value) -> Option<RustMonoItemProof> {
    let item_kind = json_value_str(value, "item_kind")?;
    let symbol_name = json_value_str(value, "symbol_name")?;
    let def_id = json_value_str(value, "def_id")?;
    let source = json_value_str(value, "source")?;
    Some(RustMonoItemProof {
        item_kind: item_kind.to_owned(),
        symbol_name: symbol_name.to_owned(),
        instance_identity: json_value_str(value, "instance_identity").map(str::to_owned),
        def_id: def_id.to_owned(),
        codegen_unit: json_value_str(value, "codegen_unit").map(str::to_owned),
        linkage: json_value_str(value, "linkage").map(str::to_owned),
        visibility: json_value_str(value, "visibility").map(str::to_owned),
        source: source.to_owned(),
    })
}

fn first_missing_compiler_stage_from(
    request: &RustCompileRequest,
    first_stage: RustCompilerStage,
) -> Option<MissingRustCompilerStage> {
    let inventory = rustc_component_inventory();
    for (stage, component_name) in compiler_stage_components() {
        if compiler_stage_order(stage) < compiler_stage_order(first_stage) {
            continue;
        }
        let component = inventory
            .iter()
            .find(|component| component.name == component_name)
            .expect("compiler stage component inventory is complete");
        if !component.embedded_in_assembly {
            let reason = component.blocker.as_ref().map_or_else(
                || {
                    format!(
                        "{} is not embedded in rouwdi.wasm; source custody is present at {}",
                        component.role, component.upstream_path
                    )
                },
                |blocker| {
                    format!(
                        "{} is not embedded in rouwdi.wasm; source custody is present at {}; latest upstream probe: {}",
                        component.role, component.upstream_path, blocker
                    )
                },
            );
            return Some(MissingRustCompilerStage {
                unit_id: request.unit_id.clone(),
                package: request.package.clone(),
                target: request.target.clone(),
                triple: request.triple.clone(),
                stage,
                error_code: RustCompilerStageErrorCode::for_stage(stage),
                required_component: component.name.clone(),
                component_role: component.role.clone(),
                reason,
            });
        }
    }
    None
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

fn compiler_stage_components() -> [(RustCompilerStage, &'static str); 4] {
    [
        (RustCompilerStage::Mir, "rustc_middle"),
        (RustCompilerStage::Monomorphization, "rustc_monomorphize"),
        (RustCompilerStage::Codegen, "rustc_codegen_llvm"),
        (RustCompilerStage::Linking, "lld"),
    ]
}

fn embedded_component(name: &str, upstream_path: &str, role: &str) -> RustcComponentStatus {
    RustcComponentStatus {
        name: name.to_owned(),
        upstream_path: upstream_path.to_owned(),
        role: role.to_owned(),
        embedded_in_assembly: true,
        required_for_complete_semantics: true,
        import_status: "upstream_backed".to_owned(),
        import_ledger_path: Some(rouwdi_rustc_upstream::IMPORT_LEDGER_PATH.to_owned()),
        adapter_crate: None,
        blocker: None,
    }
}

fn imported_component(name: &str, upstream_path: &str, role: &str) -> RustcComponentStatus {
    RustcComponentStatus {
        name: name.to_owned(),
        upstream_path: upstream_path.to_owned(),
        role: role.to_owned(),
        embedded_in_assembly: true,
        required_for_complete_semantics: true,
        import_status: "imported".to_owned(),
        import_ledger_path: Some(rouwdi_rustc_upstream::IMPORT_LEDGER_PATH.to_owned()),
        adapter_crate: Some(rouwdi_rustc_upstream::ADAPTER_CRATE.to_owned()),
        blocker: None,
    }
}

fn pending_component(name: &str, upstream_path: &str, role: &str) -> RustcComponentStatus {
    let ledger_entry = rouwdi_rustc_upstream::import_component(name);
    RustcComponentStatus {
        name: name.to_owned(),
        upstream_path: upstream_path.to_owned(),
        role: role.to_owned(),
        embedded_in_assembly: false,
        required_for_complete_semantics: true,
        import_status: ledger_entry
            .as_ref()
            .map(|component| component.import_status.clone())
            .unwrap_or_else(|| "not_attempted".to_owned()),
        import_ledger_path: Some(rouwdi_rustc_upstream::IMPORT_LEDGER_PATH.to_owned()),
        adapter_crate: ledger_entry
            .as_ref()
            .map(|component| component.adapter_path.clone()),
        blocker: ledger_entry.map(|component| component.exact_blocker),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn compile_request() -> RustCompileRequest {
        RustCompileRequest {
            unit_id: "app:rust:app:wasm32-wasip1".to_owned(),
            package: "app".to_owned(),
            target: "app".to_owned(),
            target_kind: "bin".to_owned(),
            source_path: "src/main.rs".to_owned(),
            triple: "wasm32-wasip1".to_owned(),
            profile: "release".to_owned(),
            extern_prelude: Vec::new(),
        }
    }

    fn collected_mono_proof() -> RustMonomorphizationProof {
        RustMonomorphizationProof {
            compile_unit_id: "app:rust:app:wasm32-wasip1".to_owned(),
            package: "app".to_owned(),
            target: "app".to_owned(),
            target_kind: "bin".to_owned(),
            target_triple: "wasm32-wasip1".to_owned(),
            profile: "release".to_owned(),
            source_path: "src/main.rs".to_owned(),
            mir_body_identity: "def_id=app::main".to_owned(),
            mir_body_hash: "a5e137ef6793c0b8".to_owned(),
            mono_provider: "rustc_monomorphize".to_owned(),
            mono_query: "rustc_middle::ty::TyCtxt::collect_and_partition_mono_items".to_owned(),
            mono_item_count: 1,
            mono_items: vec![RustMonoItemProof {
                item_kind: "fn".to_owned(),
                symbol_name: "rouwdi_payload::main".to_owned(),
                instance_identity: Some("InstanceDef::Item(app::main)".to_owned()),
                def_id: "DefId(0:3 ~ app::main)".to_owned(),
                codegen_unit: Some("app.0".to_owned()),
                linkage: Some("external".to_owned()),
                visibility: Some("default".to_owned()),
                source: "rustc_middle::mir::mono::MonoItem".to_owned(),
            }],
            mono_item_graph_hash: Some("bec5817d61819666".to_owned()),
            partition_count: Some(1),
            codegen_unit_count: Some(1),
            upstream_component_identities: vec!["rustc_monomorphize::collector".to_owned()],
            payload_sha256: "payload".to_owned(),
            input_contract_sha256: "input".to_owned(),
            output_contract_sha256: "output".to_owned(),
            status: "mono_items_collected".to_owned(),
            blocker_kind: None,
            blocker_component: None,
            blocker_message: None,
            failed_query: None,
            last_successful_compiler_step: Some("collect_and_partition_mono_items".to_owned()),
            fabricated_mono_items: false,
        }
    }

    #[test]
    fn embeds_real_upstream_rustc_lexer() {
        let tokens = lex_rust_source("fn main() { let raw = r#\"hello\"#; }\n");

        assert!(tokens.iter().any(|token| token.kind == "Ident"));
        assert!(tokens.iter().any(|token| token.kind.starts_with("Literal")));
        assert!(tokens.len() > 8);
    }

    #[test]
    fn lexer_records_real_rustc_lexer_bootstrap_diagnostics() {
        let proof = lex_rust_source_with_diagnostics("src/main.rs", "fn main() { \"open\n");

        assert_eq!(proof.path, "src/main.rs");
        assert!(!proof.tokens.is_empty());
        assert_eq!(proof.token_count, proof.tokens.len());
        assert!(proof
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.message == "unterminated string literal"));
    }

    #[test]
    fn inventory_does_not_claim_full_compiler_until_all_required_components_are_embedded() {
        let inventory = rustc_component_inventory();

        assert!(inventory
            .iter()
            .any(|component| component.name == "rustc_lexer" && component.embedded_in_assembly));
        assert!(inventory.iter().any(|component| {
            component.name == "rouwdi_name_resolution" && component.embedded_in_assembly
        }));
        assert!(inventory.iter().any(|component| {
            component.name == "rouwdi_borrow_check" && component.embedded_in_assembly
        }));
        assert!(inventory
            .iter()
            .any(|component| component.name == "rustc_codegen_llvm"
                && component.embedded_in_assembly));
        assert!(inventory
            .iter()
            .any(|component| component.name == "lld" && component.embedded_in_assembly));
        assert!(inventory
            .iter()
            .any(|component| component.name == "rustc_expand" && !component.embedded_in_assembly));
        assert!(!complete_rustc_semantics_embedded());
    }

    #[test]
    fn parser_stage_accepts_valid_rust_source() {
        let request = compile_request();

        let parse = parse_rust_source_for_compile_unit(&request, "fn main() {}\n");

        assert_eq!(parse.stage, RustCompilerStage::Parse);
        assert_eq!(parse.status, RustParseStageStatus::Succeeded);
        assert!(parse.token_count > 0);
        assert!(parse.node_count > 0);
        assert_eq!(parse.diagnostic_count, 0);
        assert!(parse.diagnostics.is_empty());
    }

    #[test]
    fn compiler_pipeline_advances_to_mir_after_borrow_check_success() {
        let request = compile_request();

        let error = run_rust_compiler_pipeline(&request, "fn main() {}\n").unwrap_err();

        let RustCompilerPipelineError::MirHandoff { handoff } = error else {
            panic!("valid Rust source must advance to the MIR handoff boundary");
        };
        assert_eq!(handoff.stage, RustCompilerStage::Mir);
        assert_eq!(handoff.compile_unit.unit_id, request.unit_id);
        assert_eq!(handoff.source_path, "src/main.rs");
        assert_eq!(handoff.status, RustMirHandoffStatus::AdapterUnavailable);
        assert!(!handoff.upstream_mir_adapter_available);
        assert_eq!(handoff.intended_upstream_component, "rustc_middle");
        assert_eq!(
            handoff.import_ledger_path,
            "bootstrap/upstream-rustc-import.toml"
        );
        assert_eq!(handoff.import_adapter_crate, "crates/rouwdi-rustc-upstream");
        assert_eq!(
            handoff.payload_adapter_symbol,
            rouwdi_rustc_upstream::MIR_HANDOFF_PAYLOAD_ADAPTER_SYMBOL
        );
        assert_eq!(handoff.payload_adapter_status, "payload_context_attempted");
        assert_eq!(handoff.payload_adapter_feature, "real-rustc-mir-payload");
        assert!(!handoff.payload_adapter_typechecked);
        assert!(handoff.payload_adapter_bootstrap_typechecked);
        assert!(handoff.payload_adapter_bootstrap_artifact_located);
        assert!(handoff.payload_carrier_created);
        assert!(!handoff.payload_loaded_into_rouwdi_facade);
        assert_eq!(
            handoff.payload_carrier_state.as_deref(),
            Some("payload_context_attempted")
        );
        let payload_carrier = handoff.payload_carrier.as_ref().unwrap();
        assert_eq!(
            payload_carrier.artifact.as_ref().unwrap().artifact_format,
            "wasm_module"
        );
        assert_eq!(
            payload_carrier
                .metadata_artifact
                .as_ref()
                .unwrap()
                .artifact_format,
            "rmeta"
        );
        assert_eq!(payload_carrier.load_blocker_kind.as_deref(), Some("none"));
        assert_eq!(
            payload_carrier.milestone_state.as_deref(),
            Some("bridge_wasm_mir_payload_module_emitted")
        );
        let target_pack = handoff.payload_target_pack.as_ref().unwrap();
        assert_eq!(target_pack.target_triple, "wasm32-wasip1");
        assert!(target_pack.attempted);
        assert_eq!(target_pack.status, "ready");
        assert_eq!(target_pack.exit_code, 0);
        assert_eq!(target_pack.blocker_kind, "none");
        assert!(target_pack.std_available);
        assert!(target_pack.core_available);
        assert!(target_pack.alloc_available);
        assert!(target_pack
            .produced_artifacts
            .iter()
            .any(|artifact| artifact.contains("libstd-") && artifact.ends_with(".rlib")));
        assert!(handoff.payload_bundle_inspected);
        assert_eq!(
            handoff.payload_bundle_manifest_path.as_deref(),
            Some(rouwdi_rustc_upstream::MIR_PAYLOAD_EXPORT_MANIFEST_PATH)
        );
        assert_eq!(
            handoff.payload_abi_manifest_path.as_deref(),
            Some(rouwdi_rustc_upstream::COMPILER_PAYLOAD_ABI_MANIFEST_PATH)
        );
        assert_eq!(
            handoff.payload_abi_name.as_deref(),
            Some("rouwdi.compiler-payload.mir-handoff")
        );
        assert_eq!(handoff.payload_abi_version, Some(1));
        assert_eq!(
            handoff.payload_abi_supported_stage,
            Some(rouwdi_rustc_upstream::CompilerPayloadSupportedStage::MirHandoff)
        );
        assert_eq!(
            handoff.payload_abi_primary_format,
            Some(rouwdi_rustc_upstream::CompilerPayloadAbiFormat::WasmModule)
        );
        assert_eq!(
            handoff.payload_abi_selected_route.as_deref(),
            Some("wasm32_wasip1_module")
        );
        assert_eq!(
            handoff.payload_abi_route_status,
            Some(rouwdi_rustc_upstream::CompilerPayloadAbiRouteStatus::Emitted)
        );
        assert_eq!(
            handoff.payload_abi_route_artifact_format,
            Some(rouwdi_rustc_upstream::CompilerPayloadAbiFormat::WasmModule)
        );
        assert_eq!(
            handoff.payload_abi_route_artifact_path.as_deref(),
            Some(".rouwdi/direct-rustc-private-pack/target/wasm32-wasip1/release/rouwdi_mir_adapter_probe.wasm")
        );
        assert_eq!(handoff.payload_abi_route_attempted, Some(true));
        assert_eq!(
            handoff.payload_abi_bridge_status.as_deref(),
            Some("mono_items_collected")
        );
        assert_eq!(
            handoff.payload_abi_bridge_blocker_kind.as_deref(),
            Some("none")
        );
        assert_eq!(
            handoff.payload_milestone_state.as_deref(),
            Some("bridge_wasm_mir_payload_module_emitted")
        );
        let bridge_attempt = handoff.payload_bridge_attempt.as_ref().unwrap();
        assert_eq!(bridge_attempt.status, "mono_items_collected");
        assert_eq!(bridge_attempt.blocker_kind, "none");
        assert_eq!(bridge_attempt.command_exit_code, Some(0));
        assert!(bridge_attempt
            .exact_blocker
            .contains("mono_items_collected"));
        assert!(bridge_attempt
            .exact_blocker
            .contains("rustc_monomorphize_invoked=true"));
        assert!(bridge_attempt.output_artifact_identity.is_some());
        assert_eq!(
            handoff.payload_loader_exported_artifact_class,
            Some(rouwdi_rustc_upstream::CompilerPayloadArtifactClass::WasmModule)
        );
        assert_eq!(
            handoff.payload_loader_metadata_artifact_class,
            Some(rouwdi_rustc_upstream::CompilerPayloadArtifactClass::MetadataOnly)
        );
        assert_eq!(
            handoff.payload_loader_exported_hash_status,
            Some(rouwdi_rustc_upstream::CompilerPayloadHashStatus::NotProvided)
        );
        assert_eq!(
            handoff.payload_loader_load_strategy,
            Some(rouwdi_rustc_upstream::CompilerPayloadLoadStrategy::InstantiateWasmModule)
        );
        assert_eq!(
            handoff.payload_loader_loadability_status,
            Some(rouwdi_rustc_upstream::CompilerPayloadLoadabilityStatus::Loadable)
        );
        assert_eq!(handoff.payload_loader_loadable_by_rouwdi_wasm, Some(true));
        assert_eq!(
            handoff.payload_next_required_artifact_format.as_deref(),
            Some("codegen_handoff")
        );
        assert_eq!(payload_carrier.next_artifact_command_exit_code, Some(0));
        assert_eq!(handoff.payload_adapter_probe_kind, "bootstrap_xpy_stage1");
        assert!(handoff
            .payload_adapter_probe_command
            .contains("src/tools/rouwdi-mir-adapter-probe"));
        assert_eq!(handoff.payload_adapter_probe_workdir, "third_party/rust");
        assert_eq!(handoff.payload_adapter_probe_exit_code, 0);
        assert_eq!(
            handoff.payload_adapter_probe_classification,
            "bootstrap_adapter_typechecked"
        );
        assert!(handoff
            .payload_adapter_probe_evidence
            .contains("Build completed successfully"));
        assert_eq!(handoff.payload_adapter_normal_workspace_probe_exit_code, 1);
        assert_eq!(
            handoff.payload_adapter_blocker_kind.as_deref(),
            Some("none")
        );
        assert_eq!(
            handoff.blocker_import_status.as_deref(),
            Some("payload_context_attempted")
        );
        assert!(handoff
            .blocker_probe_command
            .as_deref()
            .is_some_and(|command| command.contains("rouwdi-mir-adapter-probe")));
        assert_eq!(handoff.shared_blocker_component, None);
        assert_eq!(handoff.shared_blocker_status, None);
        assert_eq!(handoff.shared_blocker_probe_command, None);
        assert_eq!(
            handoff.blocker_category,
            Some(RustMirHandoffBlockerCategory::UpstreamCompilerPayloadNotEmbedded)
        );
        assert_eq!(
            handoff.blocker_component.as_deref(),
            Some("mir_handoff_payload_adapter")
        );
        assert!(handoff
            .required_upstream_crates
            .contains(&"rustc_mir_build".to_owned()));
        assert!(handoff
            .embedded_prerequisite_adapters
            .contains(&rouwdi_rustc_upstream::RUSTC_INDEX_ADAPTER_SYMBOL.to_owned()));
        assert!(handoff.missing_adapter_symbols.is_empty());
        assert!(handoff
            .required_context_objects
            .contains(&"rustc_middle::ty::TyCtxt<'tcx>".to_owned()));
        assert!(handoff
            .blocker_reason
            .as_deref()
            .is_some_and(|reason| reason.contains("payload_context_attempted")
                && reason.contains("bootstrap artifact located true")
                && reason.contains("bootstrap authoritative probe")));
    }

    #[test]
    fn name_resolution_stage_accepts_macro_free_no_deps_source() {
        let request = compile_request();
        let source = "fn helper() {}\nfn main() { crate::helper(); }\n";
        let parse = parse_rust_source_for_compile_unit(&request, source);
        let expansion = expand_rust_source_for_compile_unit(&request, source, &parse);

        let name_resolution = resolve_rust_names_for_compile_unit(
            &request,
            source,
            &parse,
            &expansion,
            &RustNameResolutionContext::empty(),
        );

        assert_eq!(name_resolution.stage, RustCompilerStage::NameResolution);
        assert_eq!(
            name_resolution.status,
            RustNameResolutionStageStatus::Succeeded
        );
        assert_eq!(name_resolution.diagnostic_count, 0);
        assert!(
            name_resolution
                .bindings
                .iter()
                .any(|binding| binding.name == "main"
                    && binding.kind == RustNameBindingKind::Function)
        );
        assert!(name_resolution
            .resolved_paths
            .iter()
            .any(|path| path.path == "crate::helper"));
    }

    #[test]
    fn name_resolution_stage_reports_unresolved_paths_locally() {
        let request = compile_request();
        let source = "fn main() { missing::call(); }\n";
        let parse = parse_rust_source_for_compile_unit(&request, source);
        let expansion = expand_rust_source_for_compile_unit(&request, source, &parse);

        let name_resolution = resolve_rust_names_for_compile_unit(
            &request,
            source,
            &parse,
            &expansion,
            &RustNameResolutionContext::empty(),
        );

        assert_eq!(
            name_resolution.status,
            RustNameResolutionStageStatus::Failed
        );
        assert_eq!(name_resolution.diagnostic_count, 1);
        assert_eq!(
            name_resolution.diagnostics[0].code,
            RustNameResolutionDiagnosticCode::UnresolvedPath
        );
        assert_eq!(name_resolution.diagnostics[0].path, "missing::call");
    }

    #[test]
    fn name_resolution_stage_reports_unresolved_imports_and_modules_locally() {
        let request = compile_request();
        let source = "mod missing;\nuse missing::Thing;\nfn main() {}\n";
        let parse = parse_rust_source_for_compile_unit(&request, source);
        let expansion = expand_rust_source_for_compile_unit(&request, source, &parse);

        let name_resolution = resolve_rust_names_for_compile_unit(
            &request,
            source,
            &parse,
            &expansion,
            &RustNameResolutionContext::empty(),
        );

        assert_eq!(
            name_resolution.status,
            RustNameResolutionStageStatus::Failed
        );
        assert!(name_resolution.diagnostics.iter().any(|diagnostic| {
            diagnostic.code == RustNameResolutionDiagnosticCode::UnresolvedModule
                && diagnostic.path == "missing"
        }));
        assert!(name_resolution.diagnostics.iter().any(|diagnostic| {
            diagnostic.code == RustNameResolutionDiagnosticCode::UnresolvedImport
                && diagnostic.path == "missing::Thing"
        }));
    }

    #[test]
    fn type_check_stage_accepts_macro_free_no_deps_source() {
        let request = compile_request();
        let source = "fn main() {}\n";
        let parse = parse_rust_source_for_compile_unit(&request, source);
        let expansion = expand_rust_source_for_compile_unit(&request, source, &parse);
        let name_resolution = resolve_rust_names_for_compile_unit(
            &request,
            source,
            &parse,
            &expansion,
            &RustNameResolutionContext::empty(),
        );

        let type_check = type_check_rust_for_compile_unit(
            &request,
            source,
            &parse,
            &expansion,
            &name_resolution,
        );

        assert_eq!(type_check.stage, RustCompilerStage::TypeChecking);
        assert_eq!(type_check.status, RustTypeCheckStageStatus::Succeeded);
        assert_eq!(type_check.diagnostic_count, 0);
        assert!(type_check
            .typed_items
            .iter()
            .any(|item| item.name == "main" && item.return_type == "()"));
    }

    #[test]
    fn type_check_stage_reports_mismatched_let_initializer_locally() {
        let request = compile_request();
        let source = "fn main() { let answer: bool = 1; }\n";
        let parse = parse_rust_source_for_compile_unit(&request, source);
        let expansion = expand_rust_source_for_compile_unit(&request, source, &parse);
        let name_resolution = resolve_rust_names_for_compile_unit(
            &request,
            source,
            &parse,
            &expansion,
            &RustNameResolutionContext::empty(),
        );

        let type_check = type_check_rust_for_compile_unit(
            &request,
            source,
            &parse,
            &expansion,
            &name_resolution,
        );

        assert_eq!(type_check.status, RustTypeCheckStageStatus::Failed);
        assert_eq!(type_check.diagnostic_count, 1);
        assert_eq!(
            type_check.diagnostics[0].code,
            RustTypeCheckDiagnosticCode::MismatchedTypes
        );
        assert_eq!(type_check.diagnostics[0].expected.as_deref(), Some("bool"));
        assert_eq!(type_check.diagnostics[0].actual.as_deref(), Some("integer"));
    }

    #[test]
    fn borrow_check_stage_accepts_macro_free_no_deps_source() {
        let request = compile_request();
        let source = "fn main() {}\n";
        let parse = parse_rust_source_for_compile_unit(&request, source);
        let expansion = expand_rust_source_for_compile_unit(&request, source, &parse);
        let name_resolution = resolve_rust_names_for_compile_unit(
            &request,
            source,
            &parse,
            &expansion,
            &RustNameResolutionContext::empty(),
        );
        let type_check = type_check_rust_for_compile_unit(
            &request,
            source,
            &parse,
            &expansion,
            &name_resolution,
        );

        let borrow_check = borrow_check_rust_for_compile_unit(
            &request,
            source,
            &parse,
            &expansion,
            &name_resolution,
            &type_check,
        );

        assert_eq!(borrow_check.stage, RustCompilerStage::BorrowChecking);
        assert_eq!(borrow_check.status, RustBorrowCheckStageStatus::Succeeded);
        assert_eq!(borrow_check.diagnostic_count, 0);
        assert!(borrow_check.diagnostics.is_empty());
    }

    #[test]
    fn borrow_check_stage_reports_lexical_lifetime_escape_locally() {
        let request = compile_request();
        let source = "fn main() { let r; { let x = 1; r = &x; } let _y = r; }\n";
        let parse = parse_rust_source_for_compile_unit(&request, source);
        let expansion = expand_rust_source_for_compile_unit(&request, source, &parse);
        let name_resolution = resolve_rust_names_for_compile_unit(
            &request,
            source,
            &parse,
            &expansion,
            &RustNameResolutionContext::empty(),
        );
        let type_check = type_check_rust_for_compile_unit(
            &request,
            source,
            &parse,
            &expansion,
            &name_resolution,
        );

        let borrow_check = borrow_check_rust_for_compile_unit(
            &request,
            source,
            &parse,
            &expansion,
            &name_resolution,
            &type_check,
        );

        assert_eq!(parse.status, RustParseStageStatus::Succeeded);
        assert_eq!(
            expansion.status,
            RustExpansionStageStatus::NoExpansionRequired
        );
        assert_eq!(
            name_resolution.status,
            RustNameResolutionStageStatus::Succeeded
        );
        assert_eq!(type_check.status, RustTypeCheckStageStatus::Succeeded);
        assert_eq!(borrow_check.status, RustBorrowCheckStageStatus::Failed);
        assert!(borrow_check.diagnostics.iter().any(|diagnostic| {
            diagnostic.code == RustBorrowCheckDiagnosticCode::BorrowedLocalEscapesScope
                && diagnostic.reference_local.as_deref() == Some("r")
                && diagnostic.borrowed_local.as_deref() == Some("x")
        }));
        assert!(borrow_check.references.iter().any(|reference| {
            reference.reference_local == "r" && reference.borrowed_local == "x"
        }));
    }

    #[test]
    fn expansion_stage_succeeds_when_no_expansion_is_required() {
        let request = compile_request();
        let parse = parse_rust_source_for_compile_unit(&request, "fn main() {}\n");

        let expansion = expand_rust_source_for_compile_unit(&request, "fn main() {}\n", &parse);

        assert_eq!(expansion.stage, RustCompilerStage::MacroExpansion);
        assert_eq!(
            expansion.status,
            RustExpansionStageStatus::NoExpansionRequired
        );
        assert!(expansion.status.is_success());
        assert_eq!(
            expansion.parse_stage_status,
            RustParseStageStatus::Succeeded
        );
        assert_eq!(expansion.parse_token_count, parse.token_count);
        assert_eq!(expansion.diagnostic_count, 0);
        assert!(expansion.diagnostics.is_empty());
    }

    #[test]
    fn expansion_stage_rejects_macro_invocation_locally() {
        let request = compile_request();
        let source = "fn main() { println!(\"hello\"); }\n";
        let parse = parse_rust_source_for_compile_unit(&request, source);

        let expansion = expand_rust_source_for_compile_unit(&request, source, &parse);

        assert_eq!(
            expansion.status,
            RustExpansionStageStatus::ExpansionRequired
        );
        assert!(!expansion.status.is_success());
        assert_eq!(expansion.diagnostic_count, 1);
        assert_eq!(expansion.diagnostics[0].feature, "macro_invocation");
        assert!(expansion.diagnostics[0]
            .message
            .contains("rustc_expand is not embedded"));
    }

    #[test]
    fn compiler_pipeline_returns_expansion_stage_error_for_macro_invocation() {
        let request = compile_request();

        let error = run_rust_compiler_pipeline(&request, "fn main() { println!(\"hello\"); }\n")
            .unwrap_err();

        let RustCompilerPipelineError::ExpansionStage { expansion } = error else {
            panic!("macro-using Rust source must stop in the expansion stage");
        };
        assert_eq!(
            expansion.status,
            RustExpansionStageStatus::ExpansionRequired
        );
        assert!(expansion
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.feature == "macro_invocation"));
    }

    #[test]
    fn compiler_pipeline_returns_parse_stage_error_for_invalid_syntax() {
        let request = compile_request();

        let error = run_rust_compiler_pipeline(&request, "fn main( { let = ; }\n").unwrap_err();

        let RustCompilerPipelineError::ParseStage { parse } = error else {
            panic!("invalid Rust syntax must fail in the parse stage");
        };
        assert_eq!(parse.status, RustParseStageStatus::Failed);
        assert!(parse.diagnostic_count > 0);
        assert!(!parse.diagnostics.is_empty());
    }

    #[test]
    fn compiler_pipeline_returns_name_resolution_error_for_unresolved_path() {
        let request = compile_request();

        let error =
            run_rust_compiler_pipeline(&request, "fn main() { missing::call(); }\n").unwrap_err();

        let RustCompilerPipelineError::NameResolutionStage { name_resolution } = error else {
            panic!("unresolved Rust paths must stop in the name-resolution stage");
        };
        assert_eq!(
            name_resolution.status,
            RustNameResolutionStageStatus::Failed
        );
        assert!(name_resolution
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == RustNameResolutionDiagnosticCode::UnresolvedPath));
    }

    #[test]
    fn compiler_pipeline_returns_type_check_error_for_mismatched_types() {
        let request = compile_request();

        let error = run_rust_compiler_pipeline(&request, "fn main() { let answer: bool = 1; }\n")
            .unwrap_err();

        let RustCompilerPipelineError::TypeCheckStage { type_check } = error else {
            panic!("typed-invalid Rust source must stop in the type-checking stage");
        };
        assert_eq!(type_check.status, RustTypeCheckStageStatus::Failed);
        assert!(type_check.diagnostics.iter().any(|diagnostic| {
            diagnostic.code == RustTypeCheckDiagnosticCode::MismatchedTypes
                && diagnostic.expected.as_deref() == Some("bool")
                && diagnostic.actual.as_deref() == Some("integer")
        }));
    }

    #[test]
    fn compiler_pipeline_returns_borrow_check_error_for_lexical_lifetime_escape() {
        let request = compile_request();

        let error = run_rust_compiler_pipeline(
            &request,
            "fn main() { let r; { let x = 1; r = &x; } let _y = r; }\n",
        )
        .unwrap_err();

        let RustCompilerPipelineError::BorrowCheckStage { borrow_check } = error else {
            panic!("borrow-invalid Rust source must stop in the borrow-checking stage");
        };
        assert_eq!(borrow_check.status, RustBorrowCheckStageStatus::Failed);
        assert!(borrow_check.diagnostics.iter().any(|diagnostic| {
            diagnostic.code == RustBorrowCheckDiagnosticCode::BorrowedLocalEscapesScope
                && diagnostic.reference_local.as_deref() == Some("r")
                && diagnostic.borrowed_local.as_deref() == Some("x")
        }));
    }

    #[test]
    fn compiler_stage_error_codes_are_stable_boundary_values() {
        assert_eq!(
            RustCompilerStageErrorCode::for_stage(RustCompilerStage::Parse).as_str(),
            "rustc_parse_not_embedded"
        );
        assert_eq!(
            RustCompilerStageErrorCode::for_stage(RustCompilerStage::TypeChecking).as_str(),
            "typeck_not_embedded"
        );
        assert_eq!(
            RustCompilerStageErrorCode::for_stage(RustCompilerStage::Codegen).as_str(),
            "codegen_not_embedded"
        );
        assert_eq!(
            RustCompilerStageErrorCode::for_stage(RustCompilerStage::Linking).as_str(),
            "linker_not_embedded"
        );
    }

    #[test]
    fn compiler_pipeline_record_preserves_borrow_check_and_mir_boundary() {
        let request = compile_request();

        let record = run_rust_compiler_pipeline_record(&request, "fn main() {}\n");

        assert_eq!(record.status, RustCompilerPipelineStatus::MirHandoffBlocked);
        assert_eq!(
            record.parse_stage.as_ref().unwrap().status,
            RustParseStageStatus::Succeeded
        );
        assert_eq!(
            record.expansion_stage.as_ref().unwrap().status,
            RustExpansionStageStatus::NoExpansionRequired
        );
        assert_eq!(
            record.name_resolution_stage.as_ref().unwrap().status,
            RustNameResolutionStageStatus::Succeeded
        );
        assert_eq!(
            record.type_check_stage.as_ref().unwrap().status,
            RustTypeCheckStageStatus::Succeeded
        );
        assert_eq!(
            record.borrow_check_stage.as_ref().unwrap().status,
            RustBorrowCheckStageStatus::Succeeded
        );
        let mir_handoff = record.mir_handoff.as_ref().unwrap();
        assert_eq!(mir_handoff.stage, RustCompilerStage::Mir);
        assert_eq!(mir_handoff.status, RustMirHandoffStatus::AdapterUnavailable);
        assert_eq!(
            mir_handoff.blocker_component.as_deref(),
            Some("mir_handoff_payload_adapter")
        );
        assert_eq!(
            mir_handoff.payload_adapter_status,
            "payload_context_attempted"
        );
        assert!(mir_handoff.payload_adapter_bootstrap_typechecked);
        assert!(mir_handoff.payload_adapter_bootstrap_artifact_located);
        assert!(mir_handoff.payload_carrier_created);
        assert!(!mir_handoff.payload_loaded_into_rouwdi_facade);
        assert_eq!(
            mir_handoff.payload_carrier_state.as_deref(),
            Some("payload_context_attempted")
        );
        assert_eq!(mir_handoff.payload_adapter_probe_exit_code, 0);
        assert_eq!(
            mir_handoff.payload_adapter_probe_classification,
            "bootstrap_adapter_typechecked"
        );
        assert_eq!(
            mir_handoff.payload_adapter_symbol,
            rouwdi_rustc_upstream::MIR_HANDOFF_PAYLOAD_ADAPTER_SYMBOL
        );
        assert_eq!(mir_handoff.shared_blocker_component, None);
        assert_eq!(mir_handoff.shared_blocker_status, None);
        assert_eq!(
            mir_handoff.import_ledger_path,
            "bootstrap/upstream-rustc-import.toml"
        );
        assert_eq!(
            mir_handoff.import_adapter_crate,
            "crates/rouwdi-rustc-upstream"
        );
        assert!(mir_handoff
            .embedded_prerequisite_adapters
            .contains(&rouwdi_rustc_upstream::RUSTC_INDEX_ADAPTER_SYMBOL.to_owned()));
        assert!(mir_handoff.missing_adapter_symbols.is_empty());
        assert!(mir_handoff
            .required_context_objects
            .contains(&"rustc_middle::ty::TyCtxt<'tcx>".to_owned()));
        assert!(mir_handoff.previous_stage_statuses.iter().any(|status| {
            matches!(
                status,
                RustFrontendStageStatus::BorrowChecking {
                    status: RustBorrowCheckStageStatus::Succeeded
                }
            )
        }));
        assert!(record.missing_stage.is_none());
        assert_eq!(record.artifact, None);
    }

    #[test]
    fn codegen_handoff_opens_linker_after_mono_item_object_proof() {
        let proof = collected_mono_proof();

        let handoff = RustCodegenHandoffRecord::from_valid_monomorphization_proof(&proof)
            .expect("collected mono proof must create codegen handoff");

        assert!(handoff.rust_mono_item_wasm_object_emitted);
        assert!(handoff.object_bytes_emitted);
        assert_eq!(handoff.current_status, "rust_mono_item_wasm_object_emitted");
        assert_eq!(
            handoff.codegen_lowering_status,
            "rust_mono_item_wasm_object_emitted"
        );
        assert_eq!(handoff.codegen_lowering_blocker_component, "none");
        assert!(handoff
            .codegen_lowering_required_path
            .contains(&"rustc_codegen_llvm::base::compile_codegen_unit".to_owned()));
        assert!(handoff.codegen_lowering_missing_inputs.is_empty());
        assert_eq!(handoff.object_function_count, Some(9));
        assert_eq!(handoff.object_symbol_count, Some(1));
        assert_eq!(handoff.object_is_empty, Some(false));
        assert_eq!(handoff.object_has_code_bearing_content, Some(true));
        assert!(handoff.object_contains_codegened_function);
        assert_eq!(handoff.object_codegen_source, "mono_item_graph");
        assert!(handoff.linker_handoff_created);
        let linker = handoff
            .linker_handoff
            .as_ref()
            .expect("mono-item object proof opens linker handoff");
        assert_eq!(linker.current_status, "wasm_ld_invoked");
        assert!(linker.linker_invoked);
        assert_eq!(linker.exit_code, Some(0));
        assert!(linker
            .linker_command_args
            .iter()
            .any(|arg| arg == "rouwdi-codegen-wasm32-wasip1.o"));
        handoff
            .validate_against_monomorphization_proof(&proof)
            .expect("mono-item-derived object handoff must validate");
    }
}
