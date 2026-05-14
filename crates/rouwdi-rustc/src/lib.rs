use serde::{Deserialize, Serialize};
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
        pending_component(
            "rustc_monomorphize",
            "third_party/rust/compiler/rustc_monomorphize",
            "monomorphization collector",
        ),
        pending_component(
            "rustc_codegen_llvm",
            "third_party/rust/compiler/rustc_codegen_llvm",
            "LLVM-grade codegen backend",
        ),
        pending_component(
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

    if let Some(missing) = first_missing_compiler_stage(request) {
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
    let available =
        payload_adapter.adapter_available && component.is_imported() && mir_build.is_imported();
    let required_upstream_crates = payload_adapter.required_upstream_crates.clone();
    let required_upstream_modules = boundary.required_upstream_modules.clone();
    let payload_carrier = payload_adapter.payload_carrier.clone();
    let payload_carrier_state = payload_carrier
        .as_ref()
        .map(|carrier| carrier.state.as_str().to_owned());
    let payload_loader_inspection = payload_carrier
        .as_ref()
        .and_then(|carrier| carrier.loader_inspection.clone());
    let payload_milestone_state = payload_loader_inspection
        .as_ref()
        .and_then(|inspection| inspection.milestone_state.clone())
        .or_else(|| {
            payload_carrier
                .as_ref()
                .and_then(|carrier| carrier.milestone_state.clone())
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
    let blocker_component_name = ledger_blocker
        .map(|blocker| blocker.name.clone())
        .or_else(|| payload_adapter.blocker_component.clone());
    let blocker_import_status = ledger_blocker
        .map(|blocker| blocker.import_status.clone())
        .or_else(|| payload_adapter.blocker_import_status.clone());
    let blocker_probe_command = ledger_blocker
        .map(|blocker| blocker.probe_command.clone())
        .or_else(|| payload_adapter.blocker_probe_command.clone());
    let blocker_component_role = ledger_blocker
        .map(|blocker| blocker.desired_role.clone())
        .or_else(|| Some("bootstrap-checked MIR payload adapter integration".to_owned()));
    let blocker_component_path = ledger_blocker
        .map(|blocker| blocker.source_path.clone())
        .or_else(|| payload_adapter.bootstrap_adapter_source_path.clone());
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
        payload_loaded_into_rouwdi_facade: payload_adapter.payload_loaded_into_rouwdi_facade,
        payload_load_blocker_kind,
        payload_load_blocker_reason,
        payload_next_artifact_command,
        payload_next_artifact_command_exit_code,
        payload_next_artifact_command_evidence,
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
        blocker_category: (!available)
            .then_some(RustMirHandoffBlockerCategory::UpstreamCompilerPayloadNotEmbedded),
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
        if function.name == "main" {
            if !function.params.is_empty() || !function.return_type.is_unit() {
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
        for index in start..end {
            if !is_identifier_like(&tokens[index].kind) {
                continue;
            }
            let name = token_symbol(&tokens[index]);
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
                offset: tokens[index].offset,
                len: tokens[index].len,
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
        .map(|token| token_symbol(token))
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
    for index in start..end {
        match tokens[index].kind {
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
    for index in start..end {
        match tokens[index].kind {
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

fn first_missing_compiler_stage(request: &RustCompileRequest) -> Option<MissingRustCompilerStage> {
    let inventory = rustc_component_inventory();
    for (stage, component_name) in compiler_stage_components() {
        let component = inventory
            .iter()
            .find(|component| component.name == component_name)
            .expect("compiler stage component inventory is complete");
        if !component.embedded_in_assembly {
            return Some(MissingRustCompilerStage {
                unit_id: request.unit_id.clone(),
                package: request.package.clone(),
                target: request.target.clone(),
                triple: request.triple.clone(),
                stage,
                error_code: RustCompilerStageErrorCode::for_stage(stage),
                required_component: component.name.clone(),
                component_role: component.role.clone(),
                reason: format!(
                    "{} is not embedded in rouwdi.wasm; source custody is present at {}",
                    component.role, component.upstream_path
                ),
            });
        }
    }
    None
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
                && !component.embedded_in_assembly));
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
        assert_eq!(
            handoff.payload_adapter_status,
            "payload_exported_load_blocked"
        );
        assert_eq!(handoff.payload_adapter_feature, "real-rustc-mir-payload");
        assert!(!handoff.payload_adapter_typechecked);
        assert!(handoff.payload_adapter_bootstrap_typechecked);
        assert!(handoff.payload_adapter_bootstrap_artifact_located);
        assert!(handoff.payload_carrier_created);
        assert!(!handoff.payload_loaded_into_rouwdi_facade);
        assert_eq!(
            handoff.payload_carrier_state.as_deref(),
            Some("payload_exported_load_blocked")
        );
        let payload_carrier = handoff.payload_carrier.as_ref().unwrap();
        assert_eq!(
            payload_carrier.artifact.as_ref().unwrap().artifact_format,
            "rlib"
        );
        assert_eq!(
            payload_carrier
                .metadata_artifact
                .as_ref()
                .unwrap()
                .artifact_format,
            "rmeta"
        );
        assert_eq!(
            payload_carrier.load_blocker_kind.as_deref(),
            Some("llvm_wasm32_wasip1_sysroot_missing_machine_endian")
        );
        assert_eq!(
            payload_carrier.milestone_state.as_deref(),
            Some(
                "stage2_wasm_host_route_blocked_at_llvm_wasm32_wasip1_machine_endian_header_missing"
            )
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
            Some(rouwdi_rustc_upstream::CompilerPayloadAbiRouteStatus::ShimEmittedBridgeAttemptedBlocked)
        );
        assert_eq!(
            handoff.payload_abi_route_artifact_format,
            Some(rouwdi_rustc_upstream::CompilerPayloadAbiFormat::WasmModule)
        );
        assert_eq!(
            handoff.payload_abi_route_artifact_path.as_deref(),
            Some("target/wasm32-wasip1/release/rouwdi_compiler_payload_abi.wasm")
        );
        assert_eq!(handoff.payload_abi_route_attempted, Some(true));
        assert_eq!(
            handoff.payload_abi_bridge_status.as_deref(),
            Some("attempted_blocked")
        );
        assert_eq!(
            handoff.payload_abi_bridge_blocker_kind.as_deref(),
            Some("llvm_wasm32_wasip1_sysroot_missing_machine_endian")
        );
        assert_eq!(
            handoff.payload_milestone_state.as_deref(),
            Some(
                "stage2_wasm_host_route_blocked_at_llvm_wasm32_wasip1_machine_endian_header_missing"
            )
        );
        let bridge_attempt = handoff.payload_bridge_attempt.as_ref().unwrap();
        assert_eq!(bridge_attempt.status, "attempted_blocked");
        assert_eq!(
            bridge_attempt.blocker_kind,
            "llvm_wasm32_wasip1_sysroot_missing_machine_endian"
        );
        assert_eq!(bridge_attempt.command_exit_code, Some(101));
        assert!(bridge_attempt.exact_blocker.contains("rustc_middle"));
        assert!(bridge_attempt.output_artifact_identity.is_none());
        assert_eq!(
            handoff.payload_loader_exported_artifact_class,
            Some(rouwdi_rustc_upstream::CompilerPayloadArtifactClass::RlibArchive)
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
            Some(rouwdi_rustc_upstream::CompilerPayloadLoadStrategy::InspectRlibArchive)
        );
        assert_eq!(
            handoff.payload_loader_loadability_status,
            Some(
                rouwdi_rustc_upstream::CompilerPayloadLoadabilityStatus::UnsupportedCompilerPrivateArtifact
            )
        );
        assert_eq!(handoff.payload_loader_loadable_by_rouwdi_wasm, Some(false));
        assert_eq!(
            handoff.payload_next_required_artifact_format.as_deref(),
            Some("rustc_private_to_wasm_mir_handoff_bridge")
        );
        assert_eq!(payload_carrier.next_artifact_command_exit_code, Some(1));
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
            Some("llvm_wasm32_wasip1_sysroot_missing_machine_endian")
        );
        assert_eq!(
            handoff.blocker_import_status.as_deref(),
            Some("payload_exported_load_blocked")
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
            .is_some_and(|reason| reason.contains("payload_exported_load_blocked")
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
            "payload_exported_load_blocked"
        );
        assert!(mir_handoff.payload_adapter_bootstrap_typechecked);
        assert!(mir_handoff.payload_adapter_bootstrap_artifact_located);
        assert!(mir_handoff.payload_carrier_created);
        assert!(!mir_handoff.payload_loaded_into_rouwdi_facade);
        assert_eq!(
            mir_handoff.payload_carrier_state.as_deref(),
            Some("payload_exported_load_blocked")
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
}
