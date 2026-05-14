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
}

impl fmt::Display for RustCompilerPipelineError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
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
        }
    }
}

impl std::error::Error for RustCompilerPipelineError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RustCompilerPipelineStatus {
    Artifact,
    MissingStage,
    ParseError,
    ExpansionError,
    NameResolutionError,
    TypeCheckError,
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
    pub missing_stage: Option<MissingRustCompilerStage>,
}

pub fn rustc_component_inventory() -> Vec<RustcComponentStatus> {
    vec![
        embedded_component(
            "rustc_lexer",
            "third_party/rust/compiler/rustc_lexer",
            "real upstream Rust tokenization",
        ),
        embedded_component(
            "rouwdi_name_resolution",
            "crates/rouwdi-rustc/src/lib.rs",
            "stage-local Rust name resolution for macro-free compile units",
        ),
        embedded_component(
            "rouwdi_type_check",
            "crates/rouwdi-rustc/src/lib.rs",
            "stage-local Rust type checking for macro-free compile units",
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

fn compiler_stage_components() -> [(RustCompilerStage, &'static str); 5] {
    [
        (RustCompilerStage::BorrowChecking, "rustc_borrowck"),
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
    }
}

fn pending_component(name: &str, upstream_path: &str, role: &str) -> RustcComponentStatus {
    RustcComponentStatus {
        name: name.to_owned(),
        upstream_path: upstream_path.to_owned(),
        role: role.to_owned(),
        embedded_in_assembly: false,
        required_for_complete_semantics: true,
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
    fn compiler_pipeline_advances_to_borrowck_after_type_check_success() {
        let request = compile_request();

        let error = run_rust_compiler_pipeline(&request, "fn main() {}\n").unwrap_err();

        let RustCompilerPipelineError::MissingStage { missing } = error else {
            panic!("valid Rust source must advance to the next missing compiler stage");
        };
        assert_eq!(missing.stage, RustCompilerStage::BorrowChecking);
        assert_eq!(
            missing.error_code,
            RustCompilerStageErrorCode::BorrowckNotEmbedded
        );
        assert_eq!(missing.required_component, "rustc_borrowck");
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
    fn compiler_pipeline_record_preserves_type_check_and_borrowck_boundary() {
        let request = compile_request();

        let record = run_rust_compiler_pipeline_record(&request, "fn main() {}\n");

        assert_eq!(record.status, RustCompilerPipelineStatus::MissingStage);
        assert_eq!(
            record.parse_stage.as_ref().unwrap().status,
            RustParseStageStatus::Succeeded
        );
        assert_eq!(
            record.missing_stage.as_ref().unwrap().required_component,
            "rustc_borrowck"
        );
        assert_eq!(
            record.missing_stage.as_ref().unwrap().error_code,
            RustCompilerStageErrorCode::BorrowckNotEmbedded
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
        assert_eq!(record.artifact, None);
    }
}
