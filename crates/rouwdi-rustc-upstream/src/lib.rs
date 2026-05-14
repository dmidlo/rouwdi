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
    pub root_blockers: Vec<UpstreamCompilerRootBlocker>,
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
    #[serde(default)]
    pub shared_blocker: Option<String>,
}

impl UpstreamCompilerComponentImport {
    pub fn is_imported(&self) -> bool {
        self.import_status == "imported" || self.import_status == "upstream_backed"
    }

    pub fn bootstrap_probe_passed(&self) -> bool {
        self.import_status == "bootstrap_probe_passed"
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UpstreamCompilerRootBlocker {
    pub id: String,
    pub component: String,
    pub source_path: String,
    pub status: String,
    pub blocker_kind: String,
    pub summary: String,
    pub primary_probe_command: String,
    pub generated_by_probe: String,
    pub required_probe_mode: String,
    pub stage0_rustc: String,
    #[serde(default)]
    pub required_env: Vec<String>,
    #[serde(default)]
    pub required_cfg: Vec<String>,
    #[serde(default)]
    pub observed_unstable_features: Vec<String>,
    #[serde(default)]
    pub blocked_components: Vec<String>,
    #[serde(default)]
    pub probe_attempts: Vec<UpstreamProbeAttemptRecord>,
}

impl UpstreamCompilerRootBlocker {
    pub fn is_cleared(&self) -> bool {
        self.status == "cleared_by_bootstrap_stage1" || self.status == "cleared"
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UpstreamProbeAttemptRecord {
    pub name: String,
    pub command: String,
    pub exit_code: i32,
    pub outcome: String,
    pub classification: String,
    pub evidence: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UpstreamProbeMode {
    RawCargo,
    CargoWithBootstrapCfg,
    CargoNoDefaultFeatures,
    CargoInjectedNewRangeApi,
    XpyStage0,
    XpyStage1,
}

impl UpstreamProbeMode {
    pub fn label(&self) -> &'static str {
        match self {
            Self::RawCargo => "raw-cargo",
            Self::CargoWithBootstrapCfg => "cargo-bootstrap-cfg",
            Self::CargoNoDefaultFeatures => "cargo-no-default-features",
            Self::CargoInjectedNewRangeApi => "cargo-injected-new-range-api",
            Self::XpyStage0 => "xpy-stage0",
            Self::XpyStage1 => "xpy-stage1",
        }
    }

    pub fn all() -> Vec<Self> {
        vec![
            Self::RawCargo,
            Self::CargoWithBootstrapCfg,
            Self::CargoNoDefaultFeatures,
            Self::CargoInjectedNewRangeApi,
            Self::XpyStage0,
            Self::XpyStage1,
        ]
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UpstreamProbeCommand {
    pub component: String,
    pub mode: String,
    pub workdir: String,
    pub program: String,
    pub args: Vec<String>,
    pub env: Vec<(String, String)>,
    pub note: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UpstreamProbeClassification {
    pub outcome: String,
    pub classification: String,
    pub evidence: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedUpstreamBlocker {
    pub blocked_component: UpstreamCompilerComponentImport,
    pub shared_root: Option<UpstreamCompilerRootBlocker>,
}

impl ResolvedUpstreamBlocker {
    pub fn shared_blocker_id(&self) -> Option<&str> {
        self.shared_root.as_ref().map(|root| root.id.as_str())
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

pub fn root_blocker(id: &str) -> Option<UpstreamCompilerRootBlocker> {
    import_ledger()
        .root_blockers
        .into_iter()
        .find(|blocker| blocker.id == id)
}

pub fn resolve_component_blocker(name: &str) -> Option<ResolvedUpstreamBlocker> {
    let blocked_component = import_component(name)?;
    let shared_root = blocked_component
        .shared_blocker
        .as_deref()
        .and_then(root_blocker);

    Some(ResolvedUpstreamBlocker {
        blocked_component,
        shared_root,
    })
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

pub fn mir_handoff_resolved_blocker() -> Option<ResolvedUpstreamBlocker> {
    mir_handoff_blocker().and_then(|component| resolve_component_blocker(&component.name))
}

pub fn rustc_error_codes_import_probe() -> usize {
    RUSTC_ERROR_CODE_COUNT
}

pub fn probe_command_for(component_name: &str, mode: UpstreamProbeMode) -> UpstreamProbeCommand {
    let source_path = import_component(component_name)
        .map(|component| component.source_path)
        .unwrap_or_else(|| format!("third_party/rust/compiler/{component_name}"));
    let rust_relative_source_path = source_path
        .strip_prefix("third_party/rust/")
        .unwrap_or(&source_path)
        .replace('\\', "/");

    match mode {
        UpstreamProbeMode::RawCargo => UpstreamProbeCommand {
            component: component_name.to_owned(),
            mode: mode.label().to_owned(),
            workdir: ".".to_owned(),
            program: "cargo".to_owned(),
            args: vec![
                "check".to_owned(),
                "--manifest-path".to_owned(),
                format!("{source_path}/Cargo.toml"),
                "-p".to_owned(),
                component_name.to_owned(),
            ],
            env: vec![("RUSTC_BOOTSTRAP".to_owned(), "1".to_owned())],
            note: "control probe only: raw Cargo is not the rustc bootstrap environment".to_owned(),
        },
        UpstreamProbeMode::CargoWithBootstrapCfg => UpstreamProbeCommand {
            component: component_name.to_owned(),
            mode: mode.label().to_owned(),
            workdir: ".".to_owned(),
            program: "cargo".to_owned(),
            args: vec![
                "check".to_owned(),
                "--manifest-path".to_owned(),
                format!("{source_path}/Cargo.toml"),
                "-p".to_owned(),
                component_name.to_owned(),
            ],
            env: vec![
                ("RUSTC_BOOTSTRAP".to_owned(), "1".to_owned()),
                ("RUSTFLAGS".to_owned(), "--cfg=bootstrap".to_owned()),
            ],
            note: "control probe only: adds bootstrap cfg but still bypasses x.py stage orchestration"
                .to_owned(),
        },
        UpstreamProbeMode::CargoNoDefaultFeatures => UpstreamProbeCommand {
            component: component_name.to_owned(),
            mode: mode.label().to_owned(),
            workdir: ".".to_owned(),
            program: "cargo".to_owned(),
            args: vec![
                "check".to_owned(),
                "--manifest-path".to_owned(),
                format!("{source_path}/Cargo.toml"),
                "-p".to_owned(),
                component_name.to_owned(),
                "--no-default-features".to_owned(),
            ],
            env: vec![("RUSTC_BOOTSTRAP".to_owned(), "1".to_owned())],
            note: "isolation probe: proves whether the default/nightly feature surface is involved"
                .to_owned(),
        },
        UpstreamProbeMode::CargoInjectedNewRangeApi => UpstreamProbeCommand {
            component: component_name.to_owned(),
            mode: mode.label().to_owned(),
            workdir: ".".to_owned(),
            program: "cargo".to_owned(),
            args: vec![
                "check".to_owned(),
                "--manifest-path".to_owned(),
                format!("{source_path}/Cargo.toml"),
                "-p".to_owned(),
                component_name.to_owned(),
            ],
            env: vec![
                ("RUSTC_BOOTSTRAP".to_owned(), "1".to_owned()),
                (
                    "RUSTFLAGS".to_owned(),
                    "-Zcrate-attr=feature(new_range_api)".to_owned(),
                ),
            ],
            note: "isolation probe: injects the exact unstable feature into the probed crate"
                .to_owned(),
        },
        UpstreamProbeMode::XpyStage0 => UpstreamProbeCommand {
            component: component_name.to_owned(),
            mode: mode.label().to_owned(),
            workdir: "third_party/rust".to_owned(),
            program: "python".to_owned(),
            args: vec![
                "x.py".to_owned(),
                "check".to_owned(),
                rust_relative_source_path,
                "--stage".to_owned(),
                "0".to_owned(),
                "-v".to_owned(),
            ],
            env: Vec::new(),
            note: "bootstrap-owned probe; rustc bootstrap rejects stage0 checks unless local-rebuild is configured"
                .to_owned(),
        },
        UpstreamProbeMode::XpyStage1 => UpstreamProbeCommand {
            component: component_name.to_owned(),
            mode: mode.label().to_owned(),
            workdir: "third_party/rust".to_owned(),
            program: "python".to_owned(),
            args: vec![
                "x.py".to_owned(),
                "check".to_owned(),
                rust_relative_source_path,
                "--stage".to_owned(),
                "1".to_owned(),
                "-v".to_owned(),
            ],
            env: Vec::new(),
            note: "bootstrap-owned probe: stage0 compiler checks the requested stage1 compiler crate"
                .to_owned(),
        },
    }
}

pub fn classify_probe_output(exit_code: i32, combined_output: &str) -> UpstreamProbeClassification {
    if exit_code == 0 {
        let normalized = combined_output.replace("\r\n", "\n");
        let evidence = normalized
            .lines()
            .rev()
            .find(|line| {
                line.contains("Build completed successfully")
                    || line.contains("Finished")
                    || line.contains("Checking stage1 compiler artifacts")
            })
            .unwrap_or("probe command exited successfully")
            .to_owned();

        return UpstreamProbeClassification {
            outcome: "passed".to_owned(),
            classification: "compiled".to_owned(),
            evidence,
        };
    }

    let output = combined_output.replace("\r\n", "\n");
    if output.contains("new_range_api") {
        UpstreamProbeClassification {
            outcome: "failed".to_owned(),
            classification: "raw_cargo_stage0_feature_gate_mismatch".to_owned(),
            evidence: "E0658 unstable library feature new_range_api reached through raw Cargo"
                .to_owned(),
        }
    } else if output.contains("cannot check anything on stage 0") {
        UpstreamProbeClassification {
            outcome: "failed".to_owned(),
            classification: "xpy_stage0_check_not_supported".to_owned(),
            evidence: "x.py requires stage1 or build.local-rebuild=true for compiler crate checks"
                .to_owned(),
        }
    } else if output.contains("member of the wrong workspace") {
        UpstreamProbeClassification {
            outcome: "failed".to_owned(),
            classification: "workspace_isolation_failure".to_owned(),
            evidence: "Cargo resolved the pinned Rust checkout through the parent workspace"
                .to_owned(),
        }
    } else {
        UpstreamProbeClassification {
            outcome: "failed".to_owned(),
            classification: "command_failed".to_owned(),
            evidence: output
                .lines()
                .find(|line| line.contains("error"))
                .unwrap_or("probe command exited unsuccessfully")
                .to_owned(),
        }
    }
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
        assert!(ledger
            .components
            .iter()
            .any(|component| component.name == "rustc_index"
                && component.bootstrap_probe_passed()
                && !component.is_imported()));
        assert!(ledger
            .root_blockers
            .iter()
            .any(|blocker| blocker.id == "rustc_index"
                && blocker.component == "rustc_index"
                && blocker.is_cleared()));
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
        assert!(blocker.bootstrap_probe_passed());
        assert!(!blocker.is_imported());
        assert_eq!(
            blocker.source_path,
            "third_party/rust/compiler/rustc_middle"
        );
        assert_eq!(
            blocker.immediate_dependency_blocker.as_deref(),
            Some("rustc_index")
        );
        assert_eq!(blocker.shared_blocker.as_deref(), Some("rustc_index"));
    }

    #[test]
    fn shared_blocker_graph_resolves_downstream_components() {
        for component_name in [
            "rustc_parse",
            "rustc_expand",
            "rustc_resolve",
            "rustc_hir_analysis",
            "rustc_borrowck",
            "rustc_middle",
            "rustc_mir_build",
        ] {
            let resolved = resolve_component_blocker(component_name).unwrap();
            let shared_root = resolved.shared_root.unwrap();

            assert_eq!(shared_root.id, "rustc_index");
            assert_eq!(shared_root.status, "cleared_by_bootstrap_stage1");
            assert!(shared_root
                .blocked_components
                .contains(&component_name.to_owned()));
            assert!(shared_root.probe_attempts.iter().any(|attempt| {
                attempt.classification == "compiled"
                    && attempt.command.contains("rustc_index")
                    && attempt.exit_code == 0
            }));
        }
    }

    #[test]
    fn mir_handoff_resolves_through_the_shared_blocker_graph() {
        let resolved = mir_handoff_resolved_blocker().unwrap();

        assert_eq!(resolved.blocked_component.name, "rustc_middle");
        assert_eq!(resolved.shared_blocker_id(), Some("rustc_index"));
        assert!(resolved.shared_root.as_ref().unwrap().is_cleared());
    }

    #[test]
    fn stage1_probe_command_uses_rustc_bootstrap_environment() {
        let command = probe_command_for("rustc_index", UpstreamProbeMode::XpyStage1);

        assert_eq!(command.workdir, "third_party/rust");
        assert_eq!(command.program, "python");
        assert_eq!(
            command.args,
            vec![
                "x.py",
                "check",
                "compiler/rustc_index",
                "--stage",
                "1",
                "-v"
            ]
        );
        assert!(command.note.contains("bootstrap-owned"));
    }

    #[test]
    fn raw_cargo_new_range_api_failure_is_classified_precisely() {
        let classification = classify_probe_output(
            101,
            "error[E0658]: use of unstable library feature `new_range_api`",
        );

        assert_eq!(classification.outcome, "failed");
        assert_eq!(
            classification.classification,
            "raw_cargo_stage0_feature_gate_mismatch"
        );
        assert!(classification.evidence.contains("new_range_api"));
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
