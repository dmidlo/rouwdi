use rustc_index::{Idx, IndexVec};
use serde::{Deserialize, Serialize};

pub const IMPORT_LEDGER_PATH: &str = "bootstrap/upstream-rustc-import.toml";
pub const ADAPTER_CRATE: &str = "crates/rouwdi-rustc-upstream";
pub const RUSTC_INDEX_ADAPTER_SYMBOL: &str = "rouwdi_rustc_upstream::rustc_index_adapter_surface";
pub const MIR_HANDOFF_PAYLOAD_ADAPTER_SYMBOL: &str =
    "rouwdi_rustc_upstream::mir_handoff_payload_adapter";
pub const MIR_HANDOFF_PAYLOAD_CARRIER_COMMAND: &str =
    "cargo run -p rouwdi-rustc-upstream --bin mir-payload-carrier -- --json";
pub const MIR_PAYLOAD_EXPORT_MANIFEST_PATH: &str = "bootstrap/mir-payload-export-manifest.toml";

const IMPORT_LEDGER_TOML: &str = include_str!("../../../bootstrap/upstream-rustc-import.toml");
const MIR_PAYLOAD_EXPORT_MANIFEST_TOML: &str =
    include_str!("../../../bootstrap/mir-payload-export-manifest.toml");

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
    pub bootstrap_adapter_probes: Vec<BootstrapAdapterProbeRecord>,
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
    #[serde(default)]
    pub adapter_symbol: Option<String>,
    #[serde(default)]
    pub adapter_evidence: Option<String>,
}

impl UpstreamCompilerComponentImport {
    pub fn is_imported(&self) -> bool {
        matches!(
            self.import_status.as_str(),
            "imported" | "upstream_backed" | "adapter_embedded" | "adapter_partially_embedded"
        )
    }

    pub fn adapter_embedded(&self) -> bool {
        matches!(
            self.import_status.as_str(),
            "adapter_embedded" | "adapter_partially_embedded"
        )
    }

    pub fn bootstrap_probe_passed(&self) -> bool {
        self.import_status == "bootstrap_probe_passed"
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RustcIndexAdapterSurface {
    pub component: String,
    pub source_path: String,
    pub import_status: String,
    pub adapter_symbol: String,
    pub upstream_types: Vec<String>,
    pub sample_len: usize,
    pub sample_next_index: usize,
    pub sample_indices: Vec<usize>,
    pub sample_values: Vec<String>,
    pub nightly_feature_surface_enabled: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MirHandoffAdapterBoundary {
    pub adapter_symbol: String,
    pub payload_adapter_status: MirHandoffPayloadAdapterStatus,
    pub payload_adapter_available: bool,
    pub payload_adapter_feature: String,
    pub payload_carrier_state: Option<MirHandoffPayloadCarrierState>,
    pub payload_carrier_created: bool,
    pub bootstrap_artifact_located: bool,
    pub payload_loaded_into_rouwdi_facade: bool,
    pub payload_adapter_blocker_kind: Option<String>,
    pub payload_adapter_blocker_reason: Option<String>,
    pub intended_components: Vec<String>,
    pub embedded_prerequisite_adapters: Vec<String>,
    pub missing_adapter_symbols: Vec<String>,
    pub required_context_objects: Vec<String>,
    pub required_upstream_modules: Vec<String>,
    pub blocker_component: Option<String>,
    pub blocker_import_status: Option<String>,
    pub blocker_reason: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MirHandoffPayloadAdapterStatus {
    Typechecked,
    PayloadCarrierCreated,
    PayloadLoadBlocked,
    PayloadExportedLoadBlocked,
    TypecheckedByBootstrapProbe,
    BlockedByBootstrapProbe,
    BlockedByNormalWorkspaceCargo,
}

impl MirHandoffPayloadAdapterStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Typechecked => "typechecked",
            Self::PayloadCarrierCreated => "payload_carrier_created",
            Self::PayloadLoadBlocked => "payload_load_blocked",
            Self::PayloadExportedLoadBlocked => "payload_exported_load_blocked",
            Self::TypecheckedByBootstrapProbe => "typechecked_by_bootstrap_probe",
            Self::BlockedByBootstrapProbe => "blocked_by_bootstrap_probe",
            Self::BlockedByNormalWorkspaceCargo => "blocked_by_normal_workspace_cargo",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MirHandoffPayloadCarrierState {
    BootstrapPayloadLocated,
    PayloadCarrierCreated,
    PayloadLoadBlocked,
    PayloadExportedLoadBlocked,
    PayloadLoaded,
}

impl MirHandoffPayloadCarrierState {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::BootstrapPayloadLocated => "bootstrap_payload_located",
            Self::PayloadCarrierCreated => "payload_carrier_created",
            Self::PayloadLoadBlocked => "payload_load_blocked",
            Self::PayloadExportedLoadBlocked => "payload_exported_load_blocked",
            Self::PayloadLoaded => "payload_loaded",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BootstrapMirAdapterArtifactRecord {
    pub crate_name: String,
    pub artifact_kind: String,
    pub artifact_format: String,
    pub path: String,
    pub sha256: String,
    pub size_bytes: u64,
    pub host_triple: String,
    pub profile: String,
    pub emitted_by: String,
    pub loadable_by_rouwdi_wasm: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MirHandoffPayloadCarrier {
    pub carrier_id: String,
    pub state: MirHandoffPayloadCarrierState,
    pub adapter_symbol: String,
    pub bootstrap_adapter_crate: String,
    pub bootstrap_adapter_source_path: String,
    pub xpy_command: String,
    pub xpy_stage: u32,
    pub bootstrap_probe_kind: String,
    pub bootstrap_probe_exit_code: i32,
    pub bootstrap_probe_classification: String,
    pub bootstrap_adapter_typechecked: bool,
    pub upstream_type_surface: Vec<String>,
    pub provider_surface: Vec<String>,
    pub typechecked_entrypoints: Vec<String>,
    pub artifact_locate_command: String,
    pub artifact_build_command: Option<String>,
    pub export_manifest_path: Option<String>,
    pub artifact: Option<BootstrapMirAdapterArtifactRecord>,
    pub metadata_artifact: Option<BootstrapMirAdapterArtifactRecord>,
    pub export_manifest: Option<MirPayloadExportManifest>,
    pub bootstrap_artifact_located: bool,
    pub carrier_created: bool,
    pub loaded_into_rouwdi_facade: bool,
    pub load_blocker_kind: Option<String>,
    pub load_blocker_reason: Option<String>,
    pub next_artifact_command: Option<String>,
    pub next_artifact_command_exit_code: Option<i32>,
    pub next_artifact_command_evidence: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MirPayloadExportManifest {
    pub schema_version: u32,
    pub adapter_crate: String,
    pub bootstrap_stage: u32,
    pub command: String,
    pub host_triple: String,
    pub target_triple: String,
    pub profile: String,
    pub loadable_by_rouwdi_wasm: bool,
    pub loader_blocker_kind: Option<String>,
    pub loader_blocker_reason: Option<String>,
    pub exported_payload: BootstrapMirAdapterArtifactRecord,
    pub metadata_artifact: BootstrapMirAdapterArtifactRecord,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MirHandoffPayloadAdapter {
    pub adapter_symbol: String,
    pub status: MirHandoffPayloadAdapterStatus,
    pub adapter_available: bool,
    pub typechecked_under_current_build: bool,
    pub bootstrap_artifact_located: bool,
    pub payload_carrier_created: bool,
    pub payload_loaded_into_rouwdi_facade: bool,
    pub payload_carrier: Option<MirHandoffPayloadCarrier>,
    pub cargo_feature: String,
    pub authoritative_probe_kind: String,
    pub authoritative_probe_command: String,
    pub authoritative_probe_workdir: String,
    pub authoritative_probe_exit_code: i32,
    pub authoritative_probe_classification: String,
    pub authoritative_probe_evidence: String,
    pub bootstrap_adapter_crate: Option<String>,
    pub bootstrap_adapter_source_path: Option<String>,
    pub bootstrap_adapter_typechecked: bool,
    pub normal_workspace_probe_command: String,
    pub normal_workspace_probe_exit_code: i32,
    pub upstream_type_surface: Vec<String>,
    pub typechecked_entrypoints: Vec<String>,
    pub required_upstream_crates: Vec<String>,
    pub required_upstream_modules: Vec<String>,
    pub required_context_objects: Vec<String>,
    pub embedded_prerequisite_adapters: Vec<String>,
    pub blocker_component: Option<String>,
    pub blocker_import_status: Option<String>,
    pub blocker_probe_command: Option<String>,
    pub blocker_kind: Option<String>,
    pub blocker_reason: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BootstrapAdapterProbeRecord {
    pub name: String,
    pub adapter_crate: String,
    pub source_path: String,
    pub command: String,
    pub workdir: String,
    #[serde(default)]
    pub stage: Option<u32>,
    pub exit_code: i32,
    pub outcome: String,
    pub classification: String,
    pub evidence: String,
    pub authoritative: bool,
    #[serde(default)]
    pub normal_workspace_probe_command: Option<String>,
    #[serde(default)]
    pub typechecked_entrypoints: Vec<String>,
    #[serde(default)]
    pub upstream_type_surface: Vec<String>,
    #[serde(default)]
    pub provider_surface: Vec<String>,
    #[serde(default)]
    pub required_upstream_crates: Vec<String>,
    #[serde(default)]
    pub required_upstream_modules: Vec<String>,
    #[serde(default)]
    pub payload_state: Option<String>,
    #[serde(default)]
    pub payload_carrier_created: Option<bool>,
    #[serde(default)]
    pub artifact_locate_command: Option<String>,
    #[serde(default)]
    pub artifact_build_command: Option<String>,
    #[serde(default)]
    pub export_manifest_path: Option<String>,
    #[serde(default)]
    pub artifact: Option<BootstrapMirAdapterArtifactRecord>,
    #[serde(default)]
    pub metadata_artifact: Option<BootstrapMirAdapterArtifactRecord>,
    #[serde(default)]
    pub payload_loaded_into_rouwdi_facade: Option<bool>,
    #[serde(default)]
    pub payload_load_blocker_kind: Option<String>,
    #[serde(default)]
    pub payload_load_blocker_reason: Option<String>,
    #[serde(default)]
    pub next_artifact_command: Option<String>,
    #[serde(default)]
    pub next_artifact_command_exit_code: Option<i32>,
    #[serde(default)]
    pub next_artifact_command_evidence: Option<String>,
}

impl BootstrapAdapterProbeRecord {
    pub fn typechecked(&self) -> bool {
        self.authoritative
            && self.exit_code == 0
            && self.outcome == "passed"
            && self.classification == "bootstrap_adapter_typechecked"
    }

    pub fn payload_carrier_created(&self) -> bool {
        self.payload_carrier_created.unwrap_or(false)
            || matches!(
                self.payload_state.as_deref(),
                Some(
                    "payload_carrier_created"
                        | "payload_load_blocked"
                        | "payload_exported_load_blocked"
                        | "payload_loaded"
                )
            )
    }
}

#[cfg(feature = "real-rustc-mir-payload")]
pub mod real_mir_payload_adapter {
    use rustc_hir::def_id::LocalDefId;
    use rustc_middle::mir::Body;
    use rustc_middle::query::Providers as QueryProviders;
    use rustc_middle::ty::TyCtxt;
    use rustc_middle::util::Providers as UtilityProviders;
    use rustc_session::Session;

    pub struct RealMirHandoffPayload<'a, 'tcx> {
        pub session: &'a Session,
        pub tcx: TyCtxt<'tcx>,
        pub query_providers: &'a QueryProviders,
        pub body_owner: LocalDefId,
        pub body: &'tcx Body<'tcx>,
    }

    pub fn register_mir_build_providers(providers: &mut UtilityProviders) {
        rustc_mir_build::provide(providers);
        let _build_mir_inner = providers.hooks.build_mir_inner_impl;
    }

    pub fn mir_handoff_payload_adapter<'a, 'tcx>(
        session: &'a Session,
        tcx: TyCtxt<'tcx>,
        query_providers: &'a QueryProviders,
        body_owner: LocalDefId,
    ) -> RealMirHandoffPayload<'a, 'tcx> {
        let body = tcx.optimized_mir(body_owner.to_def_id());
        RealMirHandoffPayload {
            session,
            tcx,
            query_providers,
            body_owner,
            body,
        }
    }

    pub fn typechecked_entrypoints() -> Vec<String> {
        vec![
            "rouwdi_rustc_upstream::real_mir_payload_adapter::mir_handoff_payload_adapter<'a, 'tcx>(&rustc_session::Session, rustc_middle::ty::TyCtxt<'tcx>, &rustc_middle::query::Providers, rustc_hir::def_id::LocalDefId) -> RealMirHandoffPayload<'a, 'tcx>".to_owned(),
            "rouwdi_rustc_upstream::real_mir_payload_adapter::register_mir_build_providers(&mut rustc_middle::util::Providers)".to_owned(),
        ]
    }

    pub fn type_surface() -> Vec<String> {
        vec![
            "rustc_middle::mir::Body<'tcx>".to_owned(),
            "rustc_middle::ty::TyCtxt<'tcx>".to_owned(),
            "rustc_middle::query::Providers".to_owned(),
            "rustc_middle::util::Providers".to_owned(),
            "rustc_session::Session".to_owned(),
            "rustc_hir::def_id::LocalDefId".to_owned(),
            "rustc_span::def_id::LocalDefId".to_owned(),
            "rustc_mir_build::provide".to_owned(),
            "rustc_middle::hooks::Providers::build_mir_inner_impl".to_owned(),
        ]
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

pub fn mir_payload_export_manifest() -> MirPayloadExportManifest {
    toml::from_str(MIR_PAYLOAD_EXPORT_MANIFEST_TOML)
        .expect("bootstrap/mir-payload-export-manifest.toml must remain valid")
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

pub fn bootstrap_adapter_probe(name: &str) -> Option<BootstrapAdapterProbeRecord> {
    import_ledger()
        .bootstrap_adapter_probes
        .into_iter()
        .find(|probe| probe.name == name)
}

pub fn mir_handoff_bootstrap_adapter_probe() -> Option<BootstrapAdapterProbeRecord> {
    bootstrap_adapter_probe("mir_handoff_payload_adapter")
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

fn mir_payload_required_upstream_crates() -> Vec<String> {
    vec![
        "rustc_middle".to_owned(),
        "rustc_session".to_owned(),
        "rustc_hir".to_owned(),
        "rustc_span".to_owned(),
        "rustc_mir_build".to_owned(),
    ]
}

fn mir_payload_required_upstream_modules() -> Vec<String> {
    vec![
        "rustc_middle::mir".to_owned(),
        "rustc_middle::ty".to_owned(),
        "rustc_middle::query".to_owned(),
        "rustc_middle::util".to_owned(),
        "rustc_middle::hooks".to_owned(),
        "rustc_session".to_owned(),
        "rustc_hir::def_id".to_owned(),
        "rustc_span::def_id".to_owned(),
        "rustc_mir_build".to_owned(),
    ]
}

fn mir_payload_required_context_objects() -> Vec<String> {
    vec![
        "rustc_session::Session".to_owned(),
        "rustc_middle::ty::TyCtxt<'tcx>".to_owned(),
        "rustc_middle::query::Providers".to_owned(),
        "rustc_middle::util::Providers".to_owned(),
        "rustc_hir::def_id::LocalDefId for the compile unit body owner".to_owned(),
        "rustc_middle::mir::Body<'tcx> returned from TyCtxt query".to_owned(),
    ]
}

#[cfg(feature = "real-rustc-mir-payload")]
fn mir_payload_type_surface() -> Vec<String> {
    real_mir_payload_adapter::type_surface()
}

#[cfg(not(feature = "real-rustc-mir-payload"))]
fn mir_payload_type_surface() -> Vec<String> {
    vec![
        "rustc_middle::mir::Body<'tcx>".to_owned(),
        "rustc_middle::ty::TyCtxt<'tcx>".to_owned(),
        "rustc_middle::query::Providers".to_owned(),
        "rustc_middle::util::Providers".to_owned(),
        "rustc_session::Session".to_owned(),
        "rustc_hir::def_id::LocalDefId".to_owned(),
        "rustc_span::def_id::LocalDefId".to_owned(),
        "rustc_mir_build::provide".to_owned(),
        "rustc_middle::hooks::Providers::build_mir_inner_impl".to_owned(),
    ]
}

#[cfg(feature = "real-rustc-mir-payload")]
fn mir_payload_typechecked_entrypoints() -> Vec<String> {
    real_mir_payload_adapter::typechecked_entrypoints()
}

#[cfg(not(feature = "real-rustc-mir-payload"))]
fn mir_payload_typechecked_entrypoints() -> Vec<String> {
    Vec::new()
}

fn parse_payload_carrier_state(value: Option<&str>) -> Option<MirHandoffPayloadCarrierState> {
    match value {
        Some("bootstrap_payload_located") => {
            Some(MirHandoffPayloadCarrierState::BootstrapPayloadLocated)
        }
        Some("payload_carrier_created") => {
            Some(MirHandoffPayloadCarrierState::PayloadCarrierCreated)
        }
        Some("payload_load_blocked") => Some(MirHandoffPayloadCarrierState::PayloadLoadBlocked),
        Some("payload_exported_load_blocked") => {
            Some(MirHandoffPayloadCarrierState::PayloadExportedLoadBlocked)
        }
        Some("payload_loaded") => Some(MirHandoffPayloadCarrierState::PayloadLoaded),
        _ => None,
    }
}

pub fn mir_handoff_payload_carrier() -> Option<MirHandoffPayloadCarrier> {
    let probe = mir_handoff_bootstrap_adapter_probe()?;
    let export_manifest = probe
        .export_manifest_path
        .as_deref()
        .filter(|path| *path == MIR_PAYLOAD_EXPORT_MANIFEST_PATH)
        .map(|_| mir_payload_export_manifest());
    let bootstrap_adapter_typechecked = probe.typechecked();
    let exported_payload = probe.artifact.clone().or_else(|| {
        export_manifest
            .as_ref()
            .map(|manifest| manifest.exported_payload.clone())
    });
    let metadata_artifact = probe.metadata_artifact.clone().or_else(|| {
        export_manifest
            .as_ref()
            .map(|manifest| manifest.metadata_artifact.clone())
    });
    let bootstrap_artifact_located = exported_payload.is_some() || metadata_artifact.is_some();
    let loaded_into_rouwdi_facade = probe.payload_loaded_into_rouwdi_facade.unwrap_or(false);
    let carrier_created = probe.payload_carrier_created()
        || bootstrap_artifact_located
        || loaded_into_rouwdi_facade
        || probe.payload_load_blocker_kind.is_some();
    let state = parse_payload_carrier_state(probe.payload_state.as_deref()).unwrap_or_else(|| {
        if loaded_into_rouwdi_facade {
            MirHandoffPayloadCarrierState::PayloadLoaded
        } else if exported_payload.is_some() && probe.payload_load_blocker_kind.is_some() {
            MirHandoffPayloadCarrierState::PayloadExportedLoadBlocked
        } else if probe.payload_load_blocker_kind.is_some() {
            MirHandoffPayloadCarrierState::PayloadLoadBlocked
        } else if carrier_created {
            MirHandoffPayloadCarrierState::PayloadCarrierCreated
        } else {
            MirHandoffPayloadCarrierState::BootstrapPayloadLocated
        }
    });
    let artifact_kind = exported_payload
        .as_ref()
        .map(|artifact| artifact.artifact_kind.as_str())
        .unwrap_or("unlocated");

    Some(MirHandoffPayloadCarrier {
        carrier_id: format!(
            "{}.stage{}.{}",
            probe.name,
            probe.stage.unwrap_or(1),
            artifact_kind
        ),
        state,
        adapter_symbol: MIR_HANDOFF_PAYLOAD_ADAPTER_SYMBOL.to_owned(),
        bootstrap_adapter_crate: probe.adapter_crate,
        bootstrap_adapter_source_path: probe.source_path,
        xpy_command: probe.command,
        xpy_stage: probe.stage.unwrap_or(1),
        bootstrap_probe_kind: "bootstrap_xpy_stage1".to_owned(),
        bootstrap_probe_exit_code: probe.exit_code,
        bootstrap_probe_classification: probe.classification,
        bootstrap_adapter_typechecked,
        upstream_type_surface: probe.upstream_type_surface,
        provider_surface: probe.provider_surface,
        typechecked_entrypoints: probe.typechecked_entrypoints,
        artifact_locate_command: probe
            .artifact_locate_command
            .unwrap_or_else(|| MIR_HANDOFF_PAYLOAD_CARRIER_COMMAND.to_owned()),
        artifact_build_command: probe.artifact_build_command,
        export_manifest_path: probe.export_manifest_path,
        artifact: exported_payload,
        metadata_artifact,
        export_manifest,
        bootstrap_artifact_located,
        carrier_created,
        loaded_into_rouwdi_facade,
        load_blocker_kind: probe.payload_load_blocker_kind,
        load_blocker_reason: probe.payload_load_blocker_reason,
        next_artifact_command: probe.next_artifact_command,
        next_artifact_command_exit_code: probe.next_artifact_command_exit_code,
        next_artifact_command_evidence: probe.next_artifact_command_evidence,
    })
}

pub fn mir_handoff_payload_adapter() -> MirHandoffPayloadAdapter {
    let index_surface = rustc_index_adapter_surface();
    let blocker = mir_handoff_blocker();
    let bootstrap_probe = mir_handoff_bootstrap_adapter_probe();
    let payload_carrier = mir_handoff_payload_carrier();
    let bootstrap_adapter_typechecked = bootstrap_probe
        .as_ref()
        .is_some_and(BootstrapAdapterProbeRecord::typechecked);
    let typechecked_under_current_build = cfg!(feature = "real-rustc-mir-payload");
    let payload_carrier_created = payload_carrier
        .as_ref()
        .is_some_and(|carrier| carrier.carrier_created);
    let bootstrap_artifact_located = payload_carrier
        .as_ref()
        .is_some_and(|carrier| carrier.bootstrap_artifact_located);
    let payload_loaded_into_rouwdi_facade = payload_carrier
        .as_ref()
        .is_some_and(|carrier| carrier.loaded_into_rouwdi_facade);
    let payload_load_blocked = payload_carrier
        .as_ref()
        .is_some_and(|carrier| carrier.state == MirHandoffPayloadCarrierState::PayloadLoadBlocked);
    let payload_exported_load_blocked = payload_carrier.as_ref().is_some_and(|carrier| {
        carrier.state == MirHandoffPayloadCarrierState::PayloadExportedLoadBlocked
    });
    let status = if typechecked_under_current_build {
        MirHandoffPayloadAdapterStatus::Typechecked
    } else if payload_exported_load_blocked {
        MirHandoffPayloadAdapterStatus::PayloadExportedLoadBlocked
    } else if payload_load_blocked {
        MirHandoffPayloadAdapterStatus::PayloadLoadBlocked
    } else if payload_carrier_created {
        MirHandoffPayloadAdapterStatus::PayloadCarrierCreated
    } else if bootstrap_adapter_typechecked {
        MirHandoffPayloadAdapterStatus::TypecheckedByBootstrapProbe
    } else if bootstrap_probe.is_some() {
        MirHandoffPayloadAdapterStatus::BlockedByBootstrapProbe
    } else {
        MirHandoffPayloadAdapterStatus::BlockedByNormalWorkspaceCargo
    };
    let authoritative_probe_kind = if bootstrap_probe.is_some() {
        "bootstrap_xpy_stage1"
    } else {
        "normal_workspace_cargo_control"
    };
    let authoritative_probe_command = bootstrap_probe
        .as_ref()
        .map(|probe| probe.command.clone())
        .unwrap_or_else(|| {
            "cargo check -p rouwdi-rustc-upstream --features real-rustc-mir-payload".to_owned()
        });
    let authoritative_probe_workdir = bootstrap_probe
        .as_ref()
        .map(|probe| probe.workdir.clone())
        .unwrap_or_else(|| ".".to_owned());
    let authoritative_probe_exit_code = bootstrap_probe
        .as_ref()
        .map(|probe| probe.exit_code)
        .unwrap_or(if typechecked_under_current_build {
            0
        } else {
            1
        });
    let authoritative_probe_classification = bootstrap_probe
        .as_ref()
        .map(|probe| probe.classification.clone())
        .unwrap_or_else(|| {
            if typechecked_under_current_build {
                "compiled".to_owned()
            } else {
                "normal_workspace_cargo_feature_gate".to_owned()
            }
        });
    let authoritative_probe_evidence = bootstrap_probe
        .as_ref()
        .map(|probe| probe.evidence.clone())
        .unwrap_or_else(|| {
            if typechecked_under_current_build {
                "real-rustc-mir-payload feature is typechecked in the current build".to_owned()
            } else {
                "normal workspace Cargo cannot type-check compiler-private MIR crates".to_owned()
            }
        });
    let integration_blocker = !payload_loaded_into_rouwdi_facade && bootstrap_adapter_typechecked;
    let carrier_blocker_kind = payload_carrier
        .as_ref()
        .and_then(|carrier| carrier.load_blocker_kind.clone());
    let carrier_blocker_reason = payload_carrier
        .as_ref()
        .and_then(|carrier| carrier.load_blocker_reason.clone());
    let carrier_state_label = payload_carrier
        .as_ref()
        .map(|carrier| carrier.state.as_str().to_owned());

    MirHandoffPayloadAdapter {
        adapter_symbol: MIR_HANDOFF_PAYLOAD_ADAPTER_SYMBOL.to_owned(),
        status,
        adapter_available: payload_loaded_into_rouwdi_facade,
        typechecked_under_current_build,
        bootstrap_artifact_located,
        payload_carrier_created,
        payload_loaded_into_rouwdi_facade,
        payload_carrier,
        cargo_feature: "real-rustc-mir-payload".to_owned(),
        authoritative_probe_kind: authoritative_probe_kind.to_owned(),
        authoritative_probe_command,
        authoritative_probe_workdir,
        authoritative_probe_exit_code,
        authoritative_probe_classification,
        authoritative_probe_evidence,
        bootstrap_adapter_crate: bootstrap_probe
            .as_ref()
            .map(|probe| probe.adapter_crate.clone()),
        bootstrap_adapter_source_path: bootstrap_probe
            .as_ref()
            .map(|probe| probe.source_path.clone()),
        bootstrap_adapter_typechecked,
        normal_workspace_probe_command:
            "cargo check -p rouwdi-rustc-upstream --features real-rustc-mir-payload".to_owned(),
        normal_workspace_probe_exit_code: if typechecked_under_current_build {
            0
        } else {
            1
        },
        upstream_type_surface: mir_payload_type_surface(),
        typechecked_entrypoints: mir_payload_typechecked_entrypoints(),
        required_upstream_crates: mir_payload_required_upstream_crates(),
        required_upstream_modules: mir_payload_required_upstream_modules(),
        required_context_objects: mir_payload_required_context_objects(),
        embedded_prerequisite_adapters: vec![index_surface.adapter_symbol],
        blocker_component: blocker
            .as_ref()
            .map(|component| component.name.clone())
            .or_else(|| integration_blocker.then(|| "mir_handoff_payload_adapter".to_owned())),
        blocker_import_status: blocker
            .as_ref()
            .map(|component| component.import_status.clone())
            .or_else(|| integration_blocker.then(|| {
                carrier_state_label
                    .clone()
                    .unwrap_or_else(|| "bootstrap_adapter_typechecked".to_owned())
            })),
        blocker_probe_command: blocker
            .as_ref()
            .map(|component| component.probe_command.clone())
            .or_else(|| {
                integration_blocker.then(|| {
                    bootstrap_probe
                        .as_ref()
                        .expect("integration blocker requires bootstrap probe")
                        .command
                        .clone()
                })
            }),
        blocker_kind: blocker
            .as_ref()
            .map(|component| component.blocker_kind.clone())
            .or(carrier_blocker_kind)
            .or_else(|| {
                integration_blocker
                    .then(|| "bootstrap_adapter_not_loaded_into_current_facade".to_owned())
            }),
        blocker_reason: blocker
            .map(|component| component.exact_blocker)
            .or(carrier_blocker_reason)
            .or_else(|| {
            integration_blocker.then(|| {
                let probe = bootstrap_probe
                    .as_ref()
                    .expect("integration blocker requires bootstrap probe");
                format!(
                    "bootstrap-owned MIR payload adapter typechecked under `{}` in {}; evidence: {}; normal workspace Cargo remains a non-authoritative control probe, and the next blocker is loading this bootstrap-checked payload into the rouwdi.wasm compiler facade without fabricating MIR bodies or artifacts",
                    probe.command, probe.workdir, probe.evidence
                )
            })
        }),
    }
}

pub fn rustc_error_codes_import_probe() -> usize {
    RUSTC_ERROR_CODE_COUNT
}

pub fn rustc_index_adapter_surface() -> RustcIndexAdapterSurface {
    let component =
        import_component("rustc_index").expect("upstream rustc import ledger includes rustc_index");
    let mut lanes: IndexVec<usize, &str> = IndexVec::new();
    let parser_lane = lanes.push("rustc_parse dependency lane");
    let mir_lane = lanes.push("rustc_middle MIR index lane");
    let explicit_mir_lane = <usize as Idx>::new(1);

    RustcIndexAdapterSurface {
        component: component.name,
        source_path: component.source_path,
        import_status: component.import_status,
        adapter_symbol: RUSTC_INDEX_ADAPTER_SYMBOL.to_owned(),
        upstream_types: vec![
            "rustc_index::Idx".to_owned(),
            "rustc_index::IndexVec<usize, T>".to_owned(),
            "rustc_index::IndexSlice<usize, T>".to_owned(),
        ],
        sample_len: lanes.len(),
        sample_next_index: lanes.next_index().index(),
        sample_indices: vec![
            parser_lane.index(),
            mir_lane.index(),
            explicit_mir_lane.index(),
        ],
        sample_values: vec![
            lanes[parser_lane].to_owned(),
            lanes[explicit_mir_lane].to_owned(),
        ],
        nightly_feature_surface_enabled: false,
    }
}

pub fn mir_handoff_adapter_boundary() -> MirHandoffAdapterBoundary {
    let index_surface = rustc_index_adapter_surface();
    let payload_adapter = mir_handoff_payload_adapter();
    let blocker = mir_handoff_blocker();

    MirHandoffAdapterBoundary {
        adapter_symbol: MIR_HANDOFF_PAYLOAD_ADAPTER_SYMBOL.to_owned(),
        payload_adapter_status: payload_adapter.status,
        payload_adapter_available: payload_adapter.adapter_available,
        payload_adapter_feature: payload_adapter.cargo_feature.clone(),
        payload_carrier_state: payload_adapter
            .payload_carrier
            .as_ref()
            .map(|carrier| carrier.state),
        payload_carrier_created: payload_adapter.payload_carrier_created,
        bootstrap_artifact_located: payload_adapter.bootstrap_artifact_located,
        payload_loaded_into_rouwdi_facade: payload_adapter.payload_loaded_into_rouwdi_facade,
        payload_adapter_blocker_kind: payload_adapter.blocker_kind.clone(),
        payload_adapter_blocker_reason: payload_adapter.blocker_reason.clone(),
        intended_components: vec!["rustc_middle".to_owned(), "rustc_mir_build".to_owned()],
        embedded_prerequisite_adapters: vec![index_surface.adapter_symbol],
        missing_adapter_symbols: Vec::new(),
        required_context_objects: mir_payload_required_context_objects(),
        required_upstream_modules: mir_payload_required_upstream_modules(),
        blocker_component: blocker
            .as_ref()
            .map(|component| component.name.clone())
            .or_else(|| payload_adapter.blocker_component.clone()),
        blocker_import_status: blocker
            .as_ref()
            .map(|component| component.import_status.clone())
            .or_else(|| payload_adapter.blocker_import_status.clone()),
        blocker_reason: blocker
            .as_ref()
            .map(|component| component.exact_blocker.clone())
            .or_else(|| payload_adapter.blocker_reason.clone()),
    }
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
                && component.adapter_embedded()
                && component.is_imported()
                && component.adapter_symbol.as_deref() == Some(RUSTC_INDEX_ADAPTER_SYMBOL)));
        assert!(ledger
            .root_blockers
            .iter()
            .any(|blocker| blocker.id == "rustc_index"
                && blocker.component == "rustc_index"
                && blocker.is_cleared()));
        assert!(ledger.components.iter().any(|component| {
            component.name == "rustc_middle"
                && component.import_status == "adapter_partially_embedded"
                && component.adapter_symbol.as_deref() == Some(MIR_HANDOFF_PAYLOAD_ADAPTER_SYMBOL)
        }));
        assert!(ledger
            .bootstrap_adapter_probes
            .iter()
            .any(|probe| probe.name == "mir_handoff_payload_adapter"
                && probe.typechecked()
                && probe.command.contains("rouwdi-mir-adapter-probe")));
    }

    #[test]
    fn rustc_index_adapter_surface_exercises_real_upstream_indexvec() {
        let surface = rustc_index_adapter_surface();

        assert_eq!(surface.component, "rustc_index");
        assert_eq!(surface.import_status, "adapter_embedded");
        assert_eq!(surface.adapter_symbol, RUSTC_INDEX_ADAPTER_SYMBOL);
        assert!(surface
            .upstream_types
            .contains(&"rustc_index::IndexVec<usize, T>".to_owned()));
        assert_eq!(surface.sample_len, 2);
        assert_eq!(surface.sample_next_index, 2);
        assert_eq!(surface.sample_indices, vec![0, 1, 1]);
        assert_eq!(
            surface.sample_values,
            vec![
                "rustc_parse dependency lane".to_owned(),
                "rustc_middle MIR index lane".to_owned()
            ]
        );
        assert!(!surface.nightly_feature_surface_enabled);
    }

    #[test]
    fn mir_handoff_payload_adapter_symbol_exists_and_records_real_type_surface() {
        let adapter = mir_handoff_payload_adapter();

        assert_eq!(adapter.adapter_symbol, MIR_HANDOFF_PAYLOAD_ADAPTER_SYMBOL);
        assert_eq!(
            adapter.status,
            MirHandoffPayloadAdapterStatus::PayloadExportedLoadBlocked
        );
        assert!(!adapter.adapter_available);
        assert!(adapter.bootstrap_adapter_typechecked);
        assert!(adapter.bootstrap_artifact_located);
        assert!(adapter.payload_carrier_created);
        assert!(!adapter.payload_loaded_into_rouwdi_facade);
        let carrier = adapter.payload_carrier.as_ref().unwrap();
        assert_eq!(
            carrier.state,
            MirHandoffPayloadCarrierState::PayloadExportedLoadBlocked
        );
        assert_eq!(carrier.artifact.as_ref().unwrap().artifact_format, "rlib");
        assert_eq!(
            carrier.metadata_artifact.as_ref().unwrap().artifact_format,
            "rmeta"
        );
        assert_eq!(
            carrier.load_blocker_kind.as_deref(),
            Some("rustc_private_rlib_not_rouwdi_loadable")
        );
        assert!(carrier
            .next_artifact_command
            .as_deref()
            .is_some_and(|command| command.contains("x.py build")));
        assert_eq!(adapter.authoritative_probe_kind, "bootstrap_xpy_stage1");
        assert!(adapter
            .authoritative_probe_command
            .contains("src/tools/rouwdi-mir-adapter-probe"));
        assert_eq!(adapter.authoritative_probe_exit_code, 0);
        assert_eq!(
            adapter.authoritative_probe_classification,
            "bootstrap_adapter_typechecked"
        );
        assert_eq!(adapter.cargo_feature, "real-rustc-mir-payload");
        assert_eq!(adapter.normal_workspace_probe_exit_code, 1);
        assert!(adapter
            .upstream_type_surface
            .contains(&"rustc_middle::mir::Body<'tcx>".to_owned()));
        assert!(adapter
            .upstream_type_surface
            .contains(&"rustc_middle::ty::TyCtxt<'tcx>".to_owned()));
        assert!(adapter
            .upstream_type_surface
            .contains(&"rustc_middle::query::Providers".to_owned()));
        assert!(adapter
            .upstream_type_surface
            .contains(&"rustc_session::Session".to_owned()));
        assert!(adapter
            .upstream_type_surface
            .contains(&"rustc_mir_build::provide".to_owned()));
        assert!(adapter
            .blocker_reason
            .as_deref()
            .is_some_and(|reason| reason.contains("real rlib payload")));
    }

    #[test]
    fn mir_handoff_payload_carrier_records_bootstrap_exported_payload_identity() {
        let carrier = mir_handoff_payload_carrier().unwrap();

        assert_eq!(
            carrier.state,
            MirHandoffPayloadCarrierState::PayloadExportedLoadBlocked
        );
        assert_eq!(
            carrier.bootstrap_adapter_crate,
            "third_party/rust/src/tools/rouwdi-mir-adapter-probe"
        );
        assert_eq!(carrier.xpy_stage, 1);
        assert!(carrier.bootstrap_adapter_typechecked);
        assert!(carrier.bootstrap_artifact_located);
        assert!(carrier.carrier_created);
        assert!(!carrier.loaded_into_rouwdi_facade);
        assert!(carrier
            .upstream_type_surface
            .contains(&"rustc_middle::mir::Body<'tcx>".to_owned()));
        assert!(carrier
            .provider_surface
            .contains(&"rustc_mir_build::provide(&mut rustc_middle::util::Providers)".to_owned()));
        let artifact = carrier.artifact.as_ref().unwrap();
        assert_eq!(artifact.crate_name, "rouwdi_mir_adapter_probe");
        assert_eq!(artifact.artifact_kind, "rust_rlib");
        assert_eq!(artifact.artifact_format, "rlib");
        assert!(artifact.path.ends_with("librouwdi_mir_adapter_probe.rlib"));
        assert_eq!(
            artifact.sha256,
            "3408dea65b1695dae215b62e886a9f56980c5c1d8fb17a2551ce1ed751cdc19c"
        );
        assert_eq!(artifact.size_bytes, 54210);
        assert!(!artifact.loadable_by_rouwdi_wasm);
        let metadata_artifact = carrier.metadata_artifact.as_ref().unwrap();
        assert_eq!(metadata_artifact.artifact_kind, "rustc_metadata");
        assert_eq!(metadata_artifact.artifact_format, "rmeta");
        assert!(metadata_artifact
            .path
            .ends_with("librouwdi_mir_adapter_probe-346edc2538de29f5.rmeta"));
        assert_eq!(
            metadata_artifact.sha256,
            "58843be7fbc65b466b03e365fba652be5399165bf0fb601b0eab6315efa1d4e1"
        );
        assert_eq!(metadata_artifact.size_bytes, 27097);
        assert!(!metadata_artifact.loadable_by_rouwdi_wasm);
        assert_eq!(
            carrier.load_blocker_kind.as_deref(),
            Some("rustc_private_rlib_not_rouwdi_loadable")
        );
        assert!(carrier
            .load_blocker_reason
            .as_deref()
            .is_some_and(|reason| reason.contains("loader for rustc-private rlib")));
        assert_eq!(carrier.next_artifact_command_exit_code, Some(0));
        assert_eq!(
            carrier.export_manifest_path.as_deref(),
            Some(MIR_PAYLOAD_EXPORT_MANIFEST_PATH)
        );
        assert_eq!(
            carrier.export_manifest.as_ref().unwrap().exported_payload,
            artifact.clone()
        );
    }

    #[test]
    fn mir_payload_export_manifest_distinguishes_metadata_from_payload() {
        let manifest = mir_payload_export_manifest();

        assert_eq!(manifest.schema_version, 1);
        assert_eq!(
            manifest.command,
            "python x.py build src/tools/rouwdi-mir-adapter-probe --stage 1 -v"
        );
        assert_eq!(manifest.exported_payload.artifact_format, "rlib");
        assert_eq!(manifest.metadata_artifact.artifact_format, "rmeta");
        assert_ne!(
            manifest.exported_payload.path,
            manifest.metadata_artifact.path
        );
        assert_eq!(manifest.exported_payload.size_bytes, 54210);
        assert_eq!(manifest.metadata_artifact.size_bytes, 27097);
        assert!(!manifest.exported_payload.loadable_by_rouwdi_wasm);
        assert!(!manifest.metadata_artifact.loadable_by_rouwdi_wasm);
        assert_eq!(
            manifest.loader_blocker_kind.as_deref(),
            Some("rustc_private_rlib_not_rouwdi_loadable")
        );
    }

    #[test]
    fn mir_handoff_boundary_names_the_current_payload_adapter_blocker() {
        let boundary = mir_handoff_adapter_boundary();

        assert_eq!(boundary.adapter_symbol, MIR_HANDOFF_PAYLOAD_ADAPTER_SYMBOL);
        assert_eq!(
            boundary.payload_adapter_status,
            MirHandoffPayloadAdapterStatus::PayloadExportedLoadBlocked
        );
        assert!(!boundary.payload_adapter_available);
        assert_eq!(
            boundary.payload_carrier_state,
            Some(MirHandoffPayloadCarrierState::PayloadExportedLoadBlocked)
        );
        assert!(boundary.payload_carrier_created);
        assert!(boundary.bootstrap_artifact_located);
        assert!(!boundary.payload_loaded_into_rouwdi_facade);
        assert_eq!(
            boundary.blocker_component.as_deref(),
            Some("mir_handoff_payload_adapter")
        );
        assert_eq!(
            boundary.blocker_import_status.as_deref(),
            Some("payload_exported_load_blocked")
        );
        assert!(boundary
            .embedded_prerequisite_adapters
            .contains(&RUSTC_INDEX_ADAPTER_SYMBOL.to_owned()));
        assert!(boundary.missing_adapter_symbols.is_empty());
        assert!(boundary
            .required_context_objects
            .contains(&"rustc_middle::ty::TyCtxt<'tcx>".to_owned()));
        assert!(boundary
            .blocker_reason
            .as_deref()
            .is_some_and(|reason| reason.contains("real rlib payload")));
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
    fn mir_handoff_components_are_bootstrap_partially_embedded() {
        assert!(mir_handoff_blocker().is_none());

        let components = mir_handoff_components();
        assert_eq!(components.len(), 2);
        assert!(components.iter().all(|component| {
            component.import_status == "adapter_partially_embedded"
                && component.is_imported()
                && component.probe_command.contains("rouwdi-mir-adapter-probe")
                && component.blocker_kind == "rustc_private_rlib_not_rouwdi_loadable"
                && component.adapter_symbol.as_deref() == Some(MIR_HANDOFF_PAYLOAD_ADAPTER_SYMBOL)
        }));
    }

    #[test]
    fn cleared_root_blocker_keeps_stage1_probe_evidence() {
        let shared_root = root_blocker("rustc_index").unwrap();

        assert_eq!(shared_root.id, "rustc_index");
        assert_eq!(shared_root.status, "cleared_by_bootstrap_stage1");
        assert!(shared_root
            .blocked_components
            .contains(&"rustc_middle".to_owned()));
        assert!(shared_root.probe_attempts.iter().any(|attempt| {
            attempt.classification == "compiled"
                && attempt.command.contains("rustc_index")
                && attempt.exit_code == 0
        }));
    }

    #[test]
    fn mir_handoff_resolves_through_the_shared_blocker_graph() {
        assert!(mir_handoff_resolved_blocker().is_none());
        let probe = mir_handoff_bootstrap_adapter_probe().unwrap();
        assert!(probe.typechecked());
        assert!(probe
            .required_upstream_crates
            .contains(&"rustc_mir_build".to_owned()));
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
