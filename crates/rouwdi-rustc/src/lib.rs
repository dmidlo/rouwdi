use serde::{Deserialize, Serialize};
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
pub struct RustCompileRequest {
    pub unit_id: String,
    pub package: String,
    pub target: String,
    pub target_kind: String,
    pub source_path: String,
    pub triple: String,
    pub profile: String,
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
    pub missing_stage: Option<MissingRustCompilerStage>,
}

pub fn rustc_component_inventory() -> Vec<RustcComponentStatus> {
    vec![
        embedded_component(
            "rustc_lexer",
            "third_party/rust/compiler/rustc_lexer",
            "real upstream Rust tokenization",
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

fn compiler_stage_components() -> [(RustCompilerStage, &'static str); 7] {
    [
        (RustCompilerStage::NameResolution, "rustc_resolve"),
        (RustCompilerStage::TypeChecking, "rustc_hir_analysis"),
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
    fn compiler_pipeline_returns_typed_missing_stage_after_parser_success() {
        let request = compile_request();

        let error = run_rust_compiler_pipeline(&request, "fn main() {}\n").unwrap_err();

        let RustCompilerPipelineError::MissingStage { missing } = error else {
            panic!("valid Rust source must advance to the next missing compiler stage");
        };
        assert_eq!(missing.stage, RustCompilerStage::NameResolution);
        assert_eq!(
            missing.error_code,
            RustCompilerStageErrorCode::NameResolutionNotEmbedded
        );
        assert_eq!(missing.required_component, "rustc_resolve");
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
    fn compiler_pipeline_record_preserves_missing_stage_identity() {
        let request = compile_request();

        let record = run_rust_compiler_pipeline_record(&request, "fn main() {}\n");

        assert_eq!(record.status, RustCompilerPipelineStatus::MissingStage);
        assert_eq!(
            record.parse_stage.as_ref().unwrap().status,
            RustParseStageStatus::Succeeded
        );
        assert_eq!(
            record.missing_stage.as_ref().unwrap().required_component,
            "rustc_resolve"
        );
        assert_eq!(
            record.missing_stage.as_ref().unwrap().error_code,
            RustCompilerStageErrorCode::NameResolutionNotEmbedded
        );
        assert_eq!(
            record.expansion_stage.as_ref().unwrap().status,
            RustExpansionStageStatus::NoExpansionRequired
        );
        assert_eq!(record.artifact, None);
    }
}
