use serde::{Deserialize, Serialize};

pub const IMPORT_LEDGER_PATH: &str = "bootstrap/upstream-rustc-import.toml";
pub const ADAPTER_CRATE: &str = "crates/rouwdi-rustc-upstream";

const IMPORT_LEDGER_TOML: &str = include_str!("../../../bootstrap/upstream-rustc-import.toml");

macro_rules! count_error_codes {
    ($($code:tt,)*) => {
        pub const RUSTC_ERROR_CODE_COUNT: usize = <[()]>::len(&[$({
            let _ = stringify!($code);
        },)*]);
    };
}

rustc_error_codes::error_codes!(count_error_codes);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UpstreamRustcImportLedger {
    pub ledger: ImportLedgerMetadata,
    #[serde(default)]
    pub components: Vec<UpstreamCompilerComponentImport>,
    #[serde(default)]
    pub frontend_stages: Vec<FrontendStageClassification>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImportLedgerMetadata {
    pub schema_version: u32,
    pub source_tree: String,
    pub pinned_revision: String,
    pub adapter_crate: String,
    pub stage0_rustc: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UpstreamCompilerComponentImport {
    pub name: String,
    pub source_path: String,
    pub desired_role: String,
    pub import_status: String,
    pub attempted: bool,
    pub adapter_path: String,
    pub probe_command: String,
    pub blocker_kind: String,
    pub exact_blocker: String,
    #[serde(default)]
    pub immediate_dependency_blocker: Option<String>,
}

impl UpstreamCompilerComponentImport {
    pub fn is_imported(&self) -> bool {
        self.import_status == "imported" || self.import_status == "upstream_backed"
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FrontendStageClassification {
    pub stage: String,
    pub classification: String,
    pub current_engine: String,
    pub upstream_replacement: String,
    pub ledger_component: String,
    pub notes: String,
}

pub fn import_ledger() -> UpstreamRustcImportLedger {
    toml::from_str(IMPORT_LEDGER_TOML)
        .expect("bootstrap/upstream-rustc-import.toml must remain valid")
}

pub fn import_component(name: &str) -> Option<UpstreamCompilerComponentImport> {
    import_ledger()
        .components
        .into_iter()
        .find(|component| component.name == name)
}

pub fn frontend_stage_classification(stage: &str) -> Option<FrontendStageClassification> {
    import_ledger()
        .frontend_stages
        .into_iter()
        .find(|classification| classification.stage == stage)
}

pub fn mir_handoff_components() -> Vec<UpstreamCompilerComponentImport> {
    ["rustc_middle", "rustc_mir_build"]
        .into_iter()
        .filter_map(import_component)
        .collect()
}

pub fn mir_handoff_blocker() -> Option<UpstreamCompilerComponentImport> {
    mir_handoff_components()
        .into_iter()
        .find(|component| !component.is_imported())
}

pub fn rustc_error_codes_import_probe() -> usize {
    RUSTC_ERROR_CODE_COUNT
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ledger_is_machine_readable_and_names_the_pinned_tree() {
        let ledger = import_ledger();

        assert_eq!(ledger.ledger.schema_version, 1);
        assert_eq!(ledger.ledger.source_tree, "third_party/rust");
        assert_eq!(ledger.ledger.adapter_crate, ADAPTER_CRATE);
        assert!(ledger.components.iter().any(|component| {
            component.name == "rustc_middle"
                && component.import_status == "blocked"
                && component.exact_blocker.contains("new_range_api")
        }));
    }

    #[test]
    fn adapter_records_an_upstream_component_beyond_lexer_parser() {
        let rustc_error_codes = import_component("rustc_error_codes").unwrap();

        assert!(rustc_error_codes_import_probe() > 100);
        assert_eq!(rustc_error_codes.import_status, "imported");
        assert!(rustc_error_codes.attempted);
        assert_eq!(
            rustc_error_codes.source_path,
            "third_party/rust/compiler/rustc_error_codes"
        );
    }

    #[test]
    fn mir_handoff_reports_the_first_blocked_upstream_component() {
        let blocker = mir_handoff_blocker().unwrap();

        assert_eq!(blocker.name, "rustc_middle");
        assert_eq!(
            blocker.source_path,
            "third_party/rust/compiler/rustc_middle"
        );
        assert_eq!(
            blocker.immediate_dependency_blocker.as_deref(),
            Some("rustc_index")
        );
    }

    #[test]
    fn frontend_stages_are_classified_as_scaffolding_or_upstream_backed() {
        let parse = frontend_stage_classification("parse").unwrap();
        let borrow = frontend_stage_classification("borrow_check").unwrap();

        assert_eq!(parse.ledger_component, "rustc_parse");
        assert!(parse.classification.contains("temporary"));
        assert_eq!(borrow.ledger_component, "rustc_borrowck");
        assert!(borrow.classification.contains("temporary"));
    }
}
