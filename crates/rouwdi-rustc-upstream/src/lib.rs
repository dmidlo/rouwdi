use rustc_index::{Idx, IndexVec};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

pub const IMPORT_LEDGER_PATH: &str = "bootstrap/upstream-rustc-import.toml";
pub const ADAPTER_CRATE: &str = "crates/rouwdi-rustc-upstream";
pub const RUSTC_INDEX_ADAPTER_SYMBOL: &str = "rouwdi_rustc_upstream::rustc_index_adapter_surface";
pub const MIR_HANDOFF_PAYLOAD_ADAPTER_SYMBOL: &str =
    "rouwdi_rustc_upstream::mir_handoff_payload_adapter";
pub const MIR_HANDOFF_PAYLOAD_CARRIER_COMMAND: &str =
    "cargo run -p rouwdi-rustc-upstream --bin mir-payload-carrier -- --json";
pub const MIR_PAYLOAD_EXPORT_MANIFEST_PATH: &str = "bootstrap/mir-payload-export-manifest.toml";
pub const COMPILER_PAYLOAD_ABI_MANIFEST_PATH: &str = "bootstrap/compiler-payload-abi.toml";
pub const RUSTC_PRIVATE_TARGET_PACK_MANIFEST_PATH: &str =
    "bootstrap/rustc-private-target-pack.toml";
pub const STAGE2_WASM_HOST_TOOLING_MANIFEST_PATH: &str = "bootstrap/tooling.toml";
pub const DIRECT_RUSTC_PRIVATE_PACK_BUILDER_COMMAND: &str =
    "cargo run -p rouwdi-rustc-upstream --bin direct-rustc-private-pack-builder -- --json";
pub const RUSTC_CODEGEN_LLVM_BACKEND_PROBE_COMMAND: &str =
    "powershell -ExecutionPolicy Bypass -File bootstrap/rustc-codegen-llvm-probe/run-host-probe.ps1";
pub const RUSTC_CODEGEN_LLVM_HOST_STAGE1_CHECK_COMMAND: &str =
    "python x.py check compiler/rustc_codegen_llvm --stage 1 -v";
pub const RUSTC_CODEGEN_LLVM_WASM_TARGET_CHECK_COMMAND: &str =
    "powershell -ExecutionPolicy Bypass -File bootstrap/rustc-codegen-llvm-probe/run-wasm-target-check.ps1";
pub const COMPILER_PAYLOAD_ABI_V1_VERSION_SYMBOL: &str = "rouwdi_compiler_payload_abi_v1_version";
pub const COMPILER_PAYLOAD_ABI_V1_STAGE_SYMBOL: &str = "rouwdi_compiler_payload_abi_v1_stage";
pub const COMPILER_PAYLOAD_ABI_V1_DESCRIPTOR_PTR_SYMBOL: &str =
    "rouwdi_compiler_payload_abi_v1_descriptor_ptr";
pub const COMPILER_PAYLOAD_ABI_V1_DESCRIPTOR_LEN_SYMBOL: &str =
    "rouwdi_compiler_payload_abi_v1_descriptor_len";
pub const MIR_HANDOFF_PAYLOAD_ABI_V1_EXECUTE_SYMBOL: &str = "rouwdi_mir_handoff_payload_v1_execute";
pub const MIR_HANDOFF_PAYLOAD_ABI_V1_VALID_INPUT_PTR_SYMBOL: &str =
    "rouwdi_mir_handoff_payload_v1_valid_input_ptr";
pub const MIR_HANDOFF_PAYLOAD_ABI_V1_VALID_INPUT_LEN_SYMBOL: &str =
    "rouwdi_mir_handoff_payload_v1_valid_input_len";
pub const MIR_HANDOFF_PAYLOAD_ABI_V1_RESULT_AREA_PTR_SYMBOL: &str =
    "rouwdi_mir_handoff_payload_v1_result_area_ptr";
pub const MIR_HANDOFF_PAYLOAD_ABI_V1_LAST_ERROR_PTR_SYMBOL: &str =
    "rouwdi_mir_handoff_payload_v1_last_error_ptr";
pub const MIR_HANDOFF_PAYLOAD_ABI_V1_LAST_ERROR_LEN_SYMBOL: &str =
    "rouwdi_mir_handoff_payload_v1_last_error_len";
pub const MIR_HANDOFF_CONTEXT_STATE_HIR_LOWERING_ATTEMPTED: &str = "mir_payload_module_emitted";
pub const MIR_HANDOFF_BRIDGE_MILESTONE_HIR_LOWERING_ATTEMPTED: &str =
    "bridge_wasm_mir_payload_module_emitted";
pub const MIR_HANDOFF_BLOCKER_KIND_MIR_LANG_ITEMS: &str = "none";
pub const MIR_HANDOFF_NEXT_ARTIFACT_FORMAT_MIR_LANG_ITEMS: &str = "embedded_mir_body_output";
pub const MIR_HANDOFF_CONTEXT_STATE_CRATE_AST_CREATED: &str =
    MIR_HANDOFF_CONTEXT_STATE_HIR_LOWERING_ATTEMPTED;
pub const MIR_HANDOFF_BRIDGE_MILESTONE_CRATE_AST_CREATED: &str =
    MIR_HANDOFF_BRIDGE_MILESTONE_HIR_LOWERING_ATTEMPTED;
pub const MIR_HANDOFF_BLOCKER_KIND_HIR_TYCX: &str = MIR_HANDOFF_BLOCKER_KIND_MIR_LANG_ITEMS;
pub const MIR_HANDOFF_NEXT_ARTIFACT_FORMAT_HIR_TYCX: &str =
    MIR_HANDOFF_NEXT_ARTIFACT_FORMAT_MIR_LANG_ITEMS;
pub const MIR_HANDOFF_BRIDGE_WASM_SHA256: &str =
    "b9ae49950e1f1f12768211d4b5f8fa9f6a8ebb52cacafe2bb701688db59f7c54";
pub const MIR_HANDOFF_BRIDGE_WASM_SIZE_BYTES: u64 = 88_495_302;
pub const MIR_HANDOFF_CONTEXT_STATE_SOURCE_MAP_CREATED: &str =
    MIR_HANDOFF_CONTEXT_STATE_CRATE_AST_CREATED;
pub const MIR_HANDOFF_BRIDGE_MILESTONE_SOURCE_MAP_CREATED: &str =
    MIR_HANDOFF_BRIDGE_MILESTONE_CRATE_AST_CREATED;
pub const MIR_HANDOFF_CONTEXT_STATE_MIR_BODY_HASH_EMITTED: &str = "mir_body_hash_emitted";

const IMPORT_LEDGER_TOML: &str = include_str!("../../../bootstrap/upstream-rustc-import.toml");
const MIR_PAYLOAD_EXPORT_MANIFEST_TOML: &str =
    include_str!("../../../bootstrap/mir-payload-export-manifest.toml");
const COMPILER_PAYLOAD_ABI_MANIFEST_TOML: &str =
    include_str!("../../../bootstrap/compiler-payload-abi.toml");
const RUSTC_PRIVATE_TARGET_PACK_MANIFEST_TOML: &str =
    include_str!("../../../bootstrap/rustc-private-target-pack.toml");
const STAGE2_WASM_HOST_TOOLING_MANIFEST_TOML: &str =
    include_str!("../../../bootstrap/tooling.toml");

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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RustcCodegenLlvmBackendProbe {
    pub probe_name: String,
    pub feature_enabled: bool,
    pub upstream_component: String,
    pub upstream_path: String,
    pub backend_family: String,
    pub entrypoint: String,
    pub backend_constructor_referenced: bool,
    pub backend_constructed: bool,
    pub backend_name: Option<String>,
    pub host_probe_command: String,
    pub host_probe_exit_code: i32,
    pub llvm_config_path: String,
    pub llvm_libdir: String,
    pub llvm_libs: Vec<String>,
    pub llvm_system_libs: Vec<String>,
    pub llvm_wrapper_path: String,
    pub host_probe_link_search_paths: Vec<String>,
    pub host_probe_resolved_libraries: Vec<String>,
    pub host_probe_unresolved_symbols: Vec<String>,
    pub codegen_contact_state: String,
    pub mono_proof_consumed: bool,
    pub compile_unit_id: String,
    pub crate_identity: String,
    pub mir_body_hash: String,
    pub mono_item_count: u64,
    pub mono_item_graph_hash: String,
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
    pub target_loadable_probe_command: String,
    pub target_loadable_probe_exit_code: i32,
    pub target_loadable_status: String,
    pub target_loadable_check_only_status: String,
    pub backend_payload_build_attempted: bool,
    pub backend_payload_build_exit_code: i32,
    pub executable_backend_payload_linked: bool,
    pub backend_payload_artifact_path: String,
    pub backend_payload_artifact_sha256: String,
    pub backend_payload_artifact_size_bytes: u64,
    pub embedded_backend_payload_executed: bool,
    pub backend_payload_final_link_invoked: bool,
    pub backend_payload_linker: String,
    pub backend_payload_first_undefined_symbol: String,
    pub backend_payload_llvm_undefined_symbols: Vec<String>,
    pub backend_payload_execution_status: String,
    pub backend_payload_blocker_kind: String,
    pub llvm_wrapper_target: String,
    pub llvm_wrapper_target_artifact_kind: String,
    pub llvm_wrapper_target_path: String,
    pub llvm_wrapper_target_sha256: String,
    pub llvm_wrapper_target_size_bytes: u64,
    pub llvm_wrapper_target_built_by: String,
    pub llvm_wrapper_target_linked_into: String,
    pub llvm_wrapper_target_loadable: bool,
    pub llvm_wrapper_target_blocker_kind: String,
    pub llvm_wrapper_target_blocker_reason: String,
    pub target_llvm_library_closure_available: bool,
    pub target_llvm_library_closure_status: String,
    pub enzyme_libloading_blocker_present: bool,
    pub target_loadable_components: Vec<String>,
    pub llvm_payload_route: String,
    pub blocker_kind: String,
    pub blocker_component: String,
    pub blocker_reason: String,
    pub object_emission_attempted: bool,
    pub object_bytes_emitted: bool,
    pub llvm_ir_emitted: bool,
    pub llvm_ir_sha256: Option<String>,
    pub llvm_ir_byte_len: Option<u64>,
}

impl UpstreamCompilerComponentImport {
    pub fn is_imported(&self) -> bool {
        matches!(
            self.import_status.as_str(),
            "imported"
                | "upstream_backed"
                | "adapter_embedded"
                | "adapter_partially_embedded"
                | "embedded_codegen_payload_executed"
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
    #[serde(default)]
    pub milestone_state: Option<String>,
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
    PayloadLoadableShimOnly,
    PayloadContextAttempted,
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
            Self::PayloadLoadableShimOnly => "payload_loadable_shim_only",
            Self::PayloadContextAttempted => "payload_context_attempted",
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
    PayloadLoadableShimOnly,
    PayloadContextAttempted,
    PayloadLoaded,
}

impl MirHandoffPayloadCarrierState {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::BootstrapPayloadLocated => "bootstrap_payload_located",
            Self::PayloadCarrierCreated => "payload_carrier_created",
            Self::PayloadLoadBlocked => "payload_load_blocked",
            Self::PayloadExportedLoadBlocked => "payload_exported_load_blocked",
            Self::PayloadLoadableShimOnly => "payload_loadable_shim_only",
            Self::PayloadContextAttempted => "payload_context_attempted",
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
    #[serde(default)]
    pub milestone_state: Option<String>,
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
    pub payload_bundle: Option<CompilerPayloadBundle>,
    pub loader_inspection: Option<CompilerPayloadLoaderInspection>,
    pub target_pack: Option<CompilerPayloadTargetPackProvisioning>,
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
    #[serde(default)]
    pub bundle_format_version: Option<u32>,
    pub adapter_crate: String,
    pub bootstrap_stage: u32,
    pub command: String,
    pub host_triple: String,
    pub target_triple: String,
    pub profile: String,
    pub loadable_by_rouwdi_wasm: bool,
    pub loader_blocker_kind: Option<String>,
    pub loader_blocker_reason: Option<String>,
    #[serde(default)]
    pub loadability_status: Option<CompilerPayloadLoadabilityStatus>,
    #[serde(default)]
    pub milestone_state: Option<String>,
    #[serde(default)]
    pub upstream_context_handle_v1: Option<UpstreamContextHandleV1Record>,
    #[serde(default)]
    pub wasm_runtime_execution: Option<CompilerPayloadWasmRuntimeExecutionRecord>,
    #[serde(default)]
    pub exact_loader_blocker: Option<String>,
    #[serde(default)]
    pub next_required_artifact_format: Option<String>,
    #[serde(default)]
    pub upstream_type_surface: Vec<String>,
    #[serde(default)]
    pub provider_surface: Vec<String>,
    #[serde(default)]
    pub adapter_entrypoints: Vec<String>,
    #[serde(default)]
    pub compiler_payload_abi: Option<CompilerPayloadAbiReference>,
    #[serde(default)]
    pub target_pack: Option<CompilerPayloadTargetPackProvisioning>,
    #[serde(default)]
    pub bridge: Option<CompilerPayloadAbiBridge>,
    #[serde(default)]
    pub loadable_export_routes: Vec<CompilerPayloadExportRoute>,
    pub exported_payload: BootstrapMirAdapterArtifactRecord,
    pub metadata_artifact: BootstrapMirAdapterArtifactRecord,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompilerPayloadAbiReference {
    pub manifest_path: String,
    pub abi_name: String,
    pub abi_version: u32,
    pub supported_stage: CompilerPayloadSupportedStage,
    pub selected_route: String,
    pub selected_route_status: CompilerPayloadAbiRouteStatus,
    pub bridge_status: String,
    pub bridge_blocker_kind: String,
    pub bridge_blocker_reason: String,
    #[serde(default)]
    pub milestone_state: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UpstreamContextHandleV1Record {
    pub selected_strategy: String,
    pub owner: String,
    pub scope: String,
    #[serde(default)]
    pub refers_to: Vec<String>,
    pub lifetime_rules: String,
    pub invalid_handle_behavior: String,
    pub proof_identity: String,
    pub opaque: bool,
    pub serializable: bool,
    pub may_cross_wasm_instance_boundaries: bool,
    pub raw_pointers_allowed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompilerPayloadWasmRuntimeExecutionRecord {
    pub tool_command: String,
    pub artifact_sha256: String,
    pub module_instantiated: bool,
    pub abi_exports_called: bool,
    pub descriptor_bytes_read: bool,
    pub valid_input_bytes_read: bool,
    pub execute_called: bool,
    pub output_or_error_bytes_read: bool,
    pub classification: String,
    pub context_state: String,
    pub blocker_kind: String,
    pub exact_blocker: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompilerPayloadAbiManifest {
    pub schema_version: u32,
    pub abi_name: String,
    pub abi_version: u32,
    pub abi_status: String,
    pub supported_stage: CompilerPayloadSupportedStage,
    pub primary_format: CompilerPayloadAbiFormat,
    pub supported_formats: Vec<CompilerPayloadAbiFormat>,
    pub selected_route: String,
    #[serde(default)]
    pub milestone_state: Option<String>,
    #[serde(default)]
    pub upstream_context_handle_v1: Option<UpstreamContextHandleV1Record>,
    #[serde(default)]
    pub wasm_runtime_execution: Option<CompilerPayloadWasmRuntimeExecutionRecord>,
    pub symbol_prefix: String,
    pub payload_identity: CompilerPayloadAbiIdentity,
    pub required_upstream: CompilerPayloadAbiRequiredUpstream,
    pub input_contract: CompilerPayloadAbiContract,
    pub output_contract: CompilerPayloadAbiContract,
    pub error_contract: CompilerPayloadAbiErrorContract,
    pub proof_metadata: CompilerPayloadAbiProofMetadata,
    pub versioning: CompilerPayloadAbiVersioning,
    pub bridge: CompilerPayloadAbiBridge,
    #[serde(default)]
    pub target_pack: Option<CompilerPayloadTargetPackProvisioning>,
    #[serde(default)]
    pub symbols: Vec<CompilerPayloadAbiSymbol>,
    #[serde(default)]
    pub artifact_routes: Vec<CompilerPayloadAbiRoute>,
}

impl CompilerPayloadAbiManifest {
    pub fn selected_artifact_route(&self) -> Option<&CompilerPayloadAbiRoute> {
        self.artifact_routes
            .iter()
            .find(|route| route.route == self.selected_route)
    }

    pub fn required_symbol_names(&self) -> Vec<&str> {
        self.symbols
            .iter()
            .filter(|symbol| symbol.required)
            .map(|symbol| symbol.name.as_str())
            .collect()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CompilerPayloadSupportedStage {
    MirHandoff,
}

impl CompilerPayloadSupportedStage {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::MirHandoff => "mir_handoff",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CompilerPayloadAbiFormat {
    WasmModule,
    WasmComponent,
}

impl CompilerPayloadAbiFormat {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::WasmModule => "wasm_module",
            Self::WasmComponent => "wasm_component",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompilerPayloadAbiIdentity {
    pub kind: String,
    pub name: String,
    pub abi_id: String,
    pub purpose: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompilerPayloadAbiRequiredUpstream {
    pub components: Vec<String>,
    pub type_surface: Vec<String>,
    pub provider_surface: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompilerPayloadAbiContract {
    pub shape: String,
    pub encoding: String,
    pub ownership: String,
    #[serde(default)]
    pub required_fields: Vec<String>,
    pub notes: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompilerPayloadAbiErrorContract {
    pub shape: String,
    pub encoding: String,
    pub ownership: String,
    #[serde(default)]
    pub required_fields: Vec<String>,
    #[serde(default)]
    pub known_codes: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompilerPayloadAbiProofMetadata {
    #[serde(default)]
    pub emitted_fields: Vec<String>,
    #[serde(default)]
    pub hash_scope: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompilerPayloadAbiVersioning {
    pub compatibility: String,
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
    pub rule: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompilerPayloadAbiBridge {
    pub rustc_private_payload: String,
    pub rustc_private_artifact_format: String,
    pub strategy: String,
    pub command_attempted: String,
    #[serde(default)]
    pub command_exit_code: Option<i32>,
    #[serde(default)]
    pub command_evidence: Option<String>,
    #[serde(default)]
    pub commands_attempted: Vec<CompilerPayloadBridgeCommand>,
    #[serde(default)]
    pub input_artifact_identities: Vec<CompilerPayloadBridgeArtifactIdentity>,
    #[serde(default)]
    pub output_artifact_identity: Option<CompilerPayloadBridgeArtifactIdentity>,
    pub target_triple: String,
    pub status: String,
    #[serde(default)]
    pub milestone_state: Option<String>,
    pub blocker_kind: String,
    pub blocker_reason: String,
    pub exact_blocker: String,
    pub next_command: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RustcPrivateTargetPackManifest {
    pub schema_version: u32,
    pub target_triple: String,
    pub host_triple: String,
    pub status: String,
    pub milestone_state: String,
    #[serde(default)]
    pub route_decision: Option<String>,
    #[serde(default)]
    pub upstream_context_handle_v1: Option<UpstreamContextHandleV1Record>,
    #[serde(default)]
    pub wasm_runtime_execution: Option<CompilerPayloadWasmRuntimeExecutionRecord>,
    pub exact_blocker: String,
    pub next_command: String,
    pub dependency_closure: RustcPrivateDependencyClosure,
    pub target_loadable_resolution: RustcPrivateTargetLoadableResolution,
    #[serde(default)]
    pub fallback_architecture: Option<RustcPrivateFallbackArchitecture>,
    #[serde(default)]
    pub root_crates: Vec<RustcPrivateRootCrateAttempt>,
    #[serde(default)]
    pub stage2_wasm_host_root_crates: Vec<RustcPrivateStage2WasmHostRootCrateAttempt>,
    #[serde(default)]
    pub route_discovery_attempts: Vec<RustcPrivateRouteDiscoveryAttempt>,
    pub bridge_retry: RustcPrivateBridgeRetry,
    #[serde(default)]
    pub bridge_retry_after_stage2_wasm_host: Option<RustcPrivateBridgeRetryGate>,
    #[serde(default)]
    pub direct_pack_builder: Option<RustcPrivateDirectPackBuilderRecord>,
    #[serde(default)]
    pub direct_build_strategy: Option<RustcPrivateDirectBuildStrategyRecord>,
    #[serde(default)]
    pub direct_closure_attempts: Vec<RustcPrivateDirectClosureAttempt>,
    #[serde(default)]
    pub direct_pack: Option<RustcPrivateDirectPackSummary>,
    #[serde(default)]
    pub direct_bridge_retry: Option<RustcPrivateDirectBridgeAttempt>,
    #[serde(default)]
    pub bridge_retry_after_direct_pack: Option<RustcPrivateBridgeRetryGate>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RustcPrivateDependencyClosure {
    pub metadata_command: String,
    pub metadata_exit_code: i32,
    #[serde(default)]
    pub root_crates: Vec<String>,
    #[serde(default)]
    pub transitive_compiler_private_crates: Vec<String>,
    #[serde(default)]
    pub per_root: Vec<RustcPrivatePerRootClosure>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RustcPrivatePerRootClosure {
    pub root: String,
    #[serde(default)]
    pub crates: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RustcPrivateRootCrateAttempt {
    pub name: String,
    pub command: String,
    pub workdir: String,
    pub exit_code: i32,
    pub requested_target_triple: String,
    pub emitted_target_triple: String,
    pub landed_in: String,
    #[serde(default)]
    pub produced_rlib_paths: Vec<String>,
    #[serde(default)]
    pub produced_rmeta_paths: Vec<String>,
    pub target_loadable: bool,
    pub exact_blocker: String,
    pub next_command: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RustcPrivateStage2WasmHostRootCrateAttempt {
    pub name: String,
    pub command: String,
    pub workdir: String,
    pub exit_code: i32,
    pub classification: String,
    pub cargo_targeted_wasm32_wasip1_before_blocker: bool,
    pub root_crate_cargo_targeted_wasm32_wasip1: bool,
    pub target_loadable: bool,
    pub artifact_format: String,
    pub artifact_path: String,
    pub sha256: String,
    pub size_bytes: u64,
    pub log_path: String,
    pub exact_blocker: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RustcPrivateFallbackArchitecture {
    pub selected: String,
    pub status: String,
    pub reason: String,
    #[serde(default)]
    pub must_not_do: Vec<String>,
    pub next_manifest: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RustcPrivateBridgeRetryGate {
    pub attempted: bool,
    pub classification: String,
    pub reason: String,
    #[serde(default)]
    pub required_before_retry: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RustcPrivateRouteDiscoveryAttempt {
    pub name: String,
    pub command: String,
    pub workdir: String,
    pub exit_code: i32,
    pub classification: String,
    pub evidence: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RustcPrivateTargetLoadableResolution {
    pub selected_strategy: String,
    #[serde(default)]
    pub rejected_strategies: Vec<String>,
    pub target_sysroot_path: String,
    pub target_libdir_path: String,
    pub target_rustc_private_artifact_count: usize,
    pub host_artifact_dir: String,
    pub exact_blocker: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RustcPrivateBridgeRetry {
    pub command: String,
    pub workdir: String,
    pub exit_code: i32,
    pub classification: String,
    pub evidence: String,
    pub after_root_crate_attempts: bool,
    #[serde(default)]
    pub missing_crates: Vec<String>,
    pub output_artifact: Option<CompilerPayloadBridgeArtifactIdentity>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RustcPrivateDirectPackBuilderRecord {
    pub tool: String,
    pub command: String,
    pub manifest_path: String,
    pub pack_manifest_path: String,
    pub consumes_manifest: bool,
    pub machine_readable: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RustcPrivateDirectBuildStrategyRecord {
    pub strategy: String,
    pub host_cargo_path: String,
    pub host_rustc_path: String,
    pub host_triple: String,
    pub target_triple: String,
    pub stage1_host_sysroot: String,
    pub stage1_target_sysroot: String,
    pub target_libdir_path: String,
    pub wasi_sdk_root: String,
    pub wasi_sysroot: String,
    pub wasi_linker_path: String,
    pub target_cc_env: String,
    pub target_cxx_env: String,
    pub target_ar_env: String,
    pub target_ranlib_env: String,
    pub wasi_sysroot_env: String,
    pub target_cflags_env: String,
    pub target_cxxflags_env: String,
    pub cargo_target_dir: String,
    pub rustc_bootstrap: String,
    pub cfg_release_env: String,
    pub cfg_release_channel_env: String,
    pub cfg_release_num_env: String,
    #[serde(default)]
    pub cfg_version_env: Option<String>,
    pub cfg_compiler_host_triple_env: String,
    #[serde(default)]
    pub rustc_install_bindir_env: Option<String>,
    pub rustc_stage_env: String,
    #[serde(default)]
    pub global_rustflags: Option<String>,
    pub target_rustflags_env: String,
    pub target_linker_env: String,
    pub host_flags_separated: bool,
    pub command_model: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RustcPrivateDirectClosureAttempt {
    pub name: String,
    pub command: String,
    pub workdir: String,
    pub exit_code: i32,
    pub requested_target_triple: String,
    pub emitted_target_triple: String,
    pub classification: String,
    #[serde(default)]
    pub artifacts: Vec<RustcPrivateDirectArtifactIdentity>,
    pub target_loadable: bool,
    pub exact_blocker: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RustcPrivateDirectArtifactIdentity {
    pub path: String,
    pub artifact_format: String,
    pub emitted_target_triple: String,
    pub sha256: String,
    pub size_bytes: u64,
    pub target_loadable: bool,
    pub classification: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RustcPrivateDirectPackSummary {
    pub path: String,
    pub manifest_path: String,
    pub status: String,
    pub root_crates: Vec<String>,
    pub transitive_crates: Vec<String>,
    #[serde(default)]
    pub artifacts: Vec<RustcPrivateDirectArtifactIdentity>,
    #[serde(default)]
    pub exact_missing_crates: Vec<String>,
    #[serde(default)]
    pub hash_list: Vec<String>,
    pub target_triple: String,
    pub all_required_roots_target_loadable: bool,
    pub first_hard_blocker: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RustcPrivateDirectBridgeAttempt {
    pub command: String,
    pub workdir: String,
    pub exit_code: i32,
    pub classification: String,
    pub target_triple: String,
    #[serde(default)]
    pub input_artifact_identities: Vec<CompilerPayloadBridgeArtifactIdentity>,
    pub output_artifact_identity: Option<CompilerPayloadBridgeArtifactIdentity>,
    #[serde(default)]
    pub exports: Vec<String>,
    pub abi_v1_symbols_present: bool,
    pub full_mir_payload_available: bool,
    pub exact_blocker: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DirectRustcPrivateCommandModel {
    pub host_cargo_path: String,
    pub host_rustc_path: String,
    pub host_triple: String,
    pub target_triple: String,
    pub stage1_host_sysroot: String,
    pub stage1_target_sysroot: String,
    pub target_libdir_path: String,
    pub wasi_sdk_root: String,
    pub wasi_sysroot: String,
    pub wasi_linker_path: String,
    pub target_cc_env: String,
    pub target_cxx_env: String,
    pub target_ar_env: String,
    pub target_ranlib_env: String,
    pub wasi_sysroot_env: String,
    pub target_cflags_env: String,
    pub target_cxxflags_env: String,
    pub cargo_target_dir: String,
    pub rustc_bootstrap: String,
    pub cfg_release_env: String,
    pub cfg_release_channel_env: String,
    pub cfg_release_num_env: String,
    pub cfg_version_env: String,
    pub cfg_compiler_host_triple_env: String,
    pub rustc_install_bindir_env: String,
    pub rustc_stage_env: String,
    pub global_rustflags: Option<String>,
    pub target_rustflags_env: String,
    pub target_linker_env: String,
    pub host_flags_separated: bool,
}

impl DirectRustcPrivateCommandModel {
    pub fn for_workspace(
        workspace_root: &std::path::Path,
        target_triple: &str,
        host_triple: &str,
    ) -> Self {
        let rust_root = workspace_root.join("third_party/rust");
        let stage1 = rust_root.join("build").join(host_triple).join("stage1");
        let target_libdir = stage1.join("lib/rustlib").join(target_triple).join("lib");
        let wasi_sdk_root =
            workspace_root.join(".rouwdi/tools/wasi-sdk/wasi-sdk-33.0-x86_64-windows");
        let wasi_sysroot = wasi_sdk_root.join("share/wasi-sysroot");
        let wasi_linker = wasi_sdk_root.join("bin/wasm32-wasip1-clang.exe");
        let wasi_cxx = wasi_sdk_root.join("bin/wasm32-wasip1-clang++.exe");
        let wasi_ar = wasi_sdk_root.join("bin/llvm-ar.exe");
        let wasi_ranlib = wasi_sdk_root.join("bin/llvm-ranlib.exe");
        let cargo_target_dir = workspace_root.join(".rouwdi/direct-rustc-private-pack/target");
        let host_cargo_path = rust_root
            .join("build")
            .join(host_triple)
            .join("stage0/bin/cargo.exe");
        let host_rustc_path = stage1.join("bin/rustc.exe");
        let target_rustflags = format!(
            "-Zunstable-options --cfg=bootstrap -C relocation-model=pic --sysroot {}",
            stage1.display()
        );

        Self {
            host_cargo_path: host_cargo_path.display().to_string(),
            host_rustc_path: host_rustc_path.display().to_string(),
            host_triple: host_triple.to_owned(),
            target_triple: target_triple.to_owned(),
            stage1_host_sysroot: stage1.display().to_string(),
            stage1_target_sysroot: stage1.display().to_string(),
            target_libdir_path: target_libdir.display().to_string(),
            wasi_sdk_root: wasi_sdk_root.display().to_string(),
            wasi_sysroot: wasi_sysroot.display().to_string(),
            wasi_linker_path: wasi_linker.display().to_string(),
            target_cc_env: format!("CC_wasm32_wasip1={}", wasi_linker.display()),
            target_cxx_env: format!("CXX_wasm32_wasip1={}", wasi_cxx.display()),
            target_ar_env: format!("AR_wasm32_wasip1={}", wasi_ar.display()),
            target_ranlib_env: format!("RANLIB_wasm32_wasip1={}", wasi_ranlib.display()),
            wasi_sysroot_env: format!("WASI_SYSROOT={}", wasi_sysroot.display()),
            target_cflags_env: format!("CFLAGS_wasm32_wasip1=--sysroot={}", wasi_sysroot.display()),
            target_cxxflags_env: format!(
                "CXXFLAGS_wasm32_wasip1=--sysroot={}",
                wasi_sysroot.display()
            ),
            cargo_target_dir: cargo_target_dir.display().to_string(),
            rustc_bootstrap: "1".to_owned(),
            cfg_release_env: "CFG_RELEASE=1.97.0-dev".to_owned(),
            cfg_release_channel_env: "CFG_RELEASE_CHANNEL=dev".to_owned(),
            cfg_release_num_env: "CFG_RELEASE_NUM=1.97.0".to_owned(),
            cfg_version_env: "CFG_VERSION=1.97.0-dev".to_owned(),
            cfg_compiler_host_triple_env: format!("CFG_COMPILER_HOST_TRIPLE={host_triple}"),
            rustc_install_bindir_env: "RUSTC_INSTALL_BINDIR=bin".to_owned(),
            rustc_stage_env: "RUSTC_STAGE=1".to_owned(),
            global_rustflags: None,
            target_rustflags_env: format!(
                "CARGO_TARGET_WASM32_WASIP1_RUSTFLAGS={target_rustflags}"
            ),
            target_linker_env: format!(
                "CARGO_TARGET_WASM32_WASIP1_LINKER={}",
                wasi_linker.display()
            ),
            host_flags_separated: true,
        }
    }

    pub fn to_strategy_record(&self) -> RustcPrivateDirectBuildStrategyRecord {
        RustcPrivateDirectBuildStrategyRecord {
            strategy: "direct_wasm32_wasip1_rustc_private_pack_without_stage2_wasm_host_llvm"
                .to_owned(),
            host_cargo_path: self.host_cargo_path.clone(),
            host_rustc_path: self.host_rustc_path.clone(),
            host_triple: self.host_triple.clone(),
            target_triple: self.target_triple.clone(),
            stage1_host_sysroot: self.stage1_host_sysroot.clone(),
            stage1_target_sysroot: self.stage1_target_sysroot.clone(),
            target_libdir_path: self.target_libdir_path.clone(),
            wasi_sdk_root: self.wasi_sdk_root.clone(),
            wasi_sysroot: self.wasi_sysroot.clone(),
            wasi_linker_path: self.wasi_linker_path.clone(),
            target_cc_env: self.target_cc_env.clone(),
            target_cxx_env: self.target_cxx_env.clone(),
            target_ar_env: self.target_ar_env.clone(),
            target_ranlib_env: self.target_ranlib_env.clone(),
            wasi_sysroot_env: self.wasi_sysroot_env.clone(),
            target_cflags_env: self.target_cflags_env.clone(),
            target_cxxflags_env: self.target_cxxflags_env.clone(),
            cargo_target_dir: self.cargo_target_dir.clone(),
            rustc_bootstrap: self.rustc_bootstrap.clone(),
            cfg_release_env: self.cfg_release_env.clone(),
            cfg_release_channel_env: self.cfg_release_channel_env.clone(),
            cfg_release_num_env: self.cfg_release_num_env.clone(),
            cfg_version_env: Some(self.cfg_version_env.clone()),
            cfg_compiler_host_triple_env: self.cfg_compiler_host_triple_env.clone(),
            rustc_install_bindir_env: Some(self.rustc_install_bindir_env.clone()),
            rustc_stage_env: self.rustc_stage_env.clone(),
            global_rustflags: self.global_rustflags.clone(),
            target_rustflags_env: self.target_rustflags_env.clone(),
            target_linker_env: self.target_linker_env.clone(),
            host_flags_separated: self.host_flags_separated,
            command_model: "RUSTC points at stage1 host rustc; RUSTC_BOOTSTRAP=1 plus CFG_VERSION/CFG_RELEASE/CFG_RELEASE_CHANNEL/CFG_RELEASE_NUM/CFG_COMPILER_HOST_TRIPLE/RUSTC_INSTALL_BINDIR/RUSTC_STAGE model the stage1 bootstrap environment; RUSTFLAGS is deliberately unset; target-only sysroot/linker/C/AR/WASI flags are passed through CARGO_TARGET_WASM32_WASIP1_*, CC_wasm32_wasip1, CXX_wasm32_wasip1, AR_wasm32_wasip1, RANLIB_wasm32_wasip1, CFLAGS_wasm32_wasip1, CXXFLAGS_wasm32_wasip1, and WASI_SYSROOT so host build scripts and proc macros keep using the stage1 host sysroot while target C build scripts use the WASI SDK.".to_owned(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompilerPayloadTargetPackProvisioning {
    pub target_triple: String,
    pub bootstrap_command: String,
    pub workdir: String,
    #[serde(default)]
    pub setup_path: Option<String>,
    #[serde(default)]
    pub wasi_sdk_root: Option<String>,
    #[serde(default)]
    pub wasi_root: Option<String>,
    pub attempted: bool,
    pub status: String,
    pub exit_code: i32,
    pub sysroot_path: String,
    pub target_libdir_path: String,
    #[serde(default)]
    pub produced_artifacts: Vec<String>,
    pub std_available: bool,
    pub core_available: bool,
    pub alloc_available: bool,
    #[serde(default)]
    pub rustc_private_manifest_path: Option<String>,
    #[serde(default)]
    pub rustc_private_status: Option<String>,
    #[serde(default)]
    pub rustc_private_milestone_state: Option<String>,
    pub blocker_kind: String,
    pub exact_blocker: String,
    pub next_command: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompilerPayloadBridgeCommand {
    pub command: String,
    pub workdir: String,
    pub exit_code: i32,
    pub classification: String,
    pub evidence: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompilerPayloadBridgeArtifactIdentity {
    pub role: String,
    pub artifact_format: String,
    pub path: String,
    pub sha256: String,
    pub size_bytes: u64,
    pub target_triple: String,
    pub loadable_by_rouwdi_wasm: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompilerPayloadAbiSymbol {
    pub name: String,
    pub kind: String,
    pub signature: String,
    pub required: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompilerPayloadAbiRoute {
    pub route: String,
    pub artifact_format: CompilerPayloadAbiFormat,
    pub target_triple: String,
    pub crate_path: String,
    #[serde(default)]
    pub command: Option<String>,
    pub attempted: bool,
    pub status: CompilerPayloadAbiRouteStatus,
    #[serde(default)]
    pub artifact_path: Option<String>,
    #[serde(default)]
    pub artifact_sha256: Option<String>,
    #[serde(default)]
    pub artifact_size_bytes: Option<u64>,
    pub bridge_required: bool,
    pub bridge_status: String,
    #[serde(default)]
    pub blocker_kind: Option<String>,
    pub loadable_as_full_payload: bool,
    pub exact_blocker: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CompilerPayloadAbiRouteStatus {
    Planned,
    Blocked,
    AttemptedBlocked,
    ShimEmittedBridgeAttemptedBlocked,
    Emitted,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompilerPayloadManifestIdentity {
    pub path: String,
    pub schema_version: u32,
    pub sha256: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompilerPayloadBundle {
    pub bundle_format_version: u32,
    #[serde(default)]
    pub milestone_state: Option<String>,
    pub payload_manifest: CompilerPayloadManifestIdentity,
    pub compiler_payload_abi_manifest: Option<CompilerPayloadManifestIdentity>,
    pub compiler_payload_abi: Option<CompilerPayloadAbiManifest>,
    pub bridge_attempt: Option<CompilerPayloadAbiBridge>,
    pub target_pack: Option<CompilerPayloadTargetPackProvisioning>,
    pub selected_abi_route: Option<CompilerPayloadAbiRoute>,
    pub exported_rlib_identity: BootstrapMirAdapterArtifactRecord,
    pub metadata_artifact_identity: BootstrapMirAdapterArtifactRecord,
    pub bootstrap_command: String,
    pub stage: u32,
    pub host_triple: String,
    pub target_triple: String,
    pub upstream_type_surface: Vec<String>,
    pub provider_surface: Vec<String>,
    pub adapter_entrypoints: Vec<String>,
    pub loadability_status: CompilerPayloadLoadabilityStatus,
    pub exact_loader_blocker: String,
    pub next_required_artifact_format: String,
    pub loadable_export_routes: Vec<CompilerPayloadExportRoute>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompilerPayloadExportRoute {
    pub route: String,
    pub artifact_format: String,
    #[serde(default)]
    pub command: Option<String>,
    pub attempted: bool,
    pub status: CompilerPayloadExportRouteStatus,
    #[serde(default)]
    pub blocker_kind: Option<String>,
    pub exact_blocker: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CompilerPayloadExportRouteStatus {
    Planned,
    Blocked,
    AttemptedBlocked,
    InspectedUnsupported,
    Emitted,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CompilerPayloadArtifactClass {
    MetadataOnly,
    RlibArchive,
    NativeDynamicPayload,
    StaticPayload,
    WasmModule,
    WasmComponent,
    UnsupportedCompilerPrivateArtifact,
}

impl CompilerPayloadArtifactClass {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::MetadataOnly => "metadata_only",
            Self::RlibArchive => "rlib_archive",
            Self::NativeDynamicPayload => "native_dynamic_payload",
            Self::StaticPayload => "static_payload",
            Self::WasmModule => "wasm_module",
            Self::WasmComponent => "wasm_component",
            Self::UnsupportedCompilerPrivateArtifact => "unsupported_compiler_private_artifact",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CompilerPayloadHashStatus {
    NotProvided,
    Verified,
    Mismatch,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CompilerPayloadLoadStrategy {
    InspectMetadataOnly,
    InspectRlibArchive,
    LoadNativeDynamicPayload,
    LinkStaticPayload,
    InstantiateWasmModule,
    InstantiateWasmComponent,
    UnsupportedCompilerPrivateArtifact,
}

impl CompilerPayloadLoadStrategy {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::InspectMetadataOnly => "inspect_metadata_only",
            Self::InspectRlibArchive => "inspect_rlib_archive",
            Self::LoadNativeDynamicPayload => "load_native_dynamic_payload",
            Self::LinkStaticPayload => "link_static_payload",
            Self::InstantiateWasmModule => "instantiate_wasm_module",
            Self::InstantiateWasmComponent => "instantiate_wasm_component",
            Self::UnsupportedCompilerPrivateArtifact => "unsupported_compiler_private_artifact",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CompilerPayloadLoadabilityStatus {
    MetadataOnly,
    Loadable,
    Blocked,
    UnsupportedCompilerPrivateArtifact,
}

impl CompilerPayloadLoadabilityStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::MetadataOnly => "metadata_only",
            Self::Loadable => "loadable",
            Self::Blocked => "blocked",
            Self::UnsupportedCompilerPrivateArtifact => "unsupported_compiler_private_artifact",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompilerPayloadArtifactInspection {
    pub identity: BootstrapMirAdapterArtifactRecord,
    pub artifact_class: CompilerPayloadArtifactClass,
    pub hash_status: CompilerPayloadHashStatus,
    pub computed_sha256: Option<String>,
    pub size_bytes: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompilerPayloadLoaderInspection {
    pub payload_bundle_inspected: bool,
    #[serde(default)]
    pub milestone_state: Option<String>,
    pub bundle_manifest: CompilerPayloadManifestIdentity,
    pub abi_manifest: Option<CompilerPayloadManifestIdentity>,
    pub bridge_attempt: Option<CompilerPayloadAbiBridge>,
    pub abi_name: Option<String>,
    pub abi_version: Option<u32>,
    pub abi_supported_stage: Option<CompilerPayloadSupportedStage>,
    pub abi_primary_format: Option<CompilerPayloadAbiFormat>,
    pub abi_selected_route: Option<String>,
    pub abi_route_status: Option<CompilerPayloadAbiRouteStatus>,
    pub abi_route_artifact_format: Option<CompilerPayloadAbiFormat>,
    pub abi_route_artifact_path: Option<String>,
    pub abi_route_artifact_sha256: Option<String>,
    pub abi_route_artifact_size_bytes: Option<u64>,
    pub abi_route_attempted: Option<bool>,
    pub abi_route_blocker_kind: Option<String>,
    pub abi_bridge_status: Option<String>,
    pub abi_bridge_blocker_kind: Option<String>,
    pub abi_bridge_blocker_reason: Option<String>,
    pub target_pack: Option<CompilerPayloadTargetPackProvisioning>,
    pub exported_payload: CompilerPayloadArtifactInspection,
    pub metadata_artifact: CompilerPayloadArtifactInspection,
    pub load_strategy: CompilerPayloadLoadStrategy,
    pub loadability_status: CompilerPayloadLoadabilityStatus,
    pub loadable_by_rouwdi_wasm: bool,
    pub loader_blocker_kind: Option<String>,
    pub exact_loader_blocker: String,
    pub next_required_artifact_format: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompilerPayloadWasmRuntimeExecution {
    pub artifact_path: String,
    pub expected_sha256: String,
    pub computed_sha256: String,
    pub hash_verified: bool,
    pub module_instantiated: bool,
    pub exports: Vec<String>,
    pub abi_v1_exports_verified: bool,
    pub version_called: bool,
    pub version: u32,
    pub stage_called: bool,
    pub stage: u32,
    pub descriptor_ptr: u32,
    pub descriptor_len: u32,
    pub descriptor_bytes_read: bool,
    pub descriptor_json: String,
    pub valid_input_ptr: u32,
    pub valid_input_len: u32,
    pub valid_input_bytes_read: bool,
    pub valid_input_json: String,
    pub execute_called: bool,
    pub execute_status: i32,
    pub output_ptr: u32,
    pub output_len: u32,
    pub error_ptr: u32,
    pub error_len: u32,
    pub output_bytes_read: bool,
    pub output_json: Option<String>,
    pub error_bytes_read: bool,
    pub error_json: Option<String>,
    pub classification: String,
    pub context_handle_strategy: String,
    pub context_state: String,
    pub generic_upstream_context_unavailable_replaced: bool,
    pub fabricated_ast: bool,
    pub fabricated_hir: bool,
    pub fabricated_tyctx: bool,
    pub fabricated_providers: bool,
    pub fabricated_body: bool,
    pub fabricated_mir: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Stage2WasmHostToolingManifest {
    pub schema_version: u32,
    pub milestone: String,
    pub recorded_by: String,
    pub stage2_wasm_host_route: Stage2WasmHostRouteTooling,
    pub tools: Stage2WasmHostTools,
    #[serde(default)]
    pub commands: Vec<Stage2WasmHostToolingCommand>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Stage2WasmHostRouteTooling {
    pub command: String,
    pub workdir: String,
    pub bootstrap_config: String,
    pub bootstrap_config_download_ci_llvm: bool,
    pub bootstrap_config_llvm_ninja: bool,
    pub bootstrap_config_wasm32_wasip1_wasi_root: String,
    pub bootstrap_config_wasm32_wasip1_cc: String,
    pub bootstrap_config_wasm32_wasip1_cxx: String,
    pub bootstrap_config_wasm32_wasip1_ar: String,
    pub bootstrap_config_wasm32_wasip1_ranlib: String,
    pub bootstrap_config_wasm32_wasip1_linker: String,
    pub path_prefix: String,
    pub decision: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Stage2WasmHostTools {
    pub ninja: Stage2ToolRecord,
    pub cmake: Stage2CMakeToolRecord,
    pub llvm: Stage2LlvmToolRecord,
    pub clang: Stage2ClangToolRecord,
    pub python: Stage2PythonToolRecord,
    pub fetch_extract: Stage2FetchExtractToolRecord,
    pub wasi_sdk: Stage2WasiSdkToolRecord,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Stage2ToolRecord {
    pub required: bool,
    pub status: String,
    pub source: String,
    pub version: String,
    pub path: String,
    pub archive: String,
    pub archive_url: String,
    pub archive_sha256: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Stage2CMakeToolRecord {
    pub required: bool,
    pub required_because: String,
    pub status: String,
    pub source: String,
    pub version: String,
    pub path: String,
    pub archive: String,
    pub archive_url: String,
    pub archive_sha256: String,
    pub detected_on_path: bool,
    pub detected_path: String,
    pub detected_version: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Stage2LlvmToolRecord {
    pub download_ci_llvm_configured: bool,
    pub llvm_tools_found: bool,
    pub wasm_ld_found: bool,
    pub wasm_ld_path: String,
    pub wasm_ld_version: String,
    pub lld_link_found: bool,
    pub lld_link_path: String,
    pub lld_link_version: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Stage2ClangToolRecord {
    pub clang_found: bool,
    pub clang_path: String,
    pub clang_version: String,
    pub clang_cl_found: bool,
    pub clang_cl_path: String,
    pub clang_cl_version: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Stage2PythonToolRecord {
    pub required: bool,
    pub found: bool,
    pub path: String,
    pub version: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Stage2FetchExtractToolRecord {
    pub curl_found: bool,
    pub curl_path: String,
    pub curl_version: String,
    pub tar_found: bool,
    pub tar_path: String,
    pub tar_version: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Stage2WasiSdkToolRecord {
    pub required: bool,
    pub status: String,
    pub root: String,
    pub wasi_root: String,
    pub clang: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Stage2WasmHostToolingCommand {
    pub name: String,
    pub command: String,
    #[serde(default)]
    pub workdir: Option<String>,
    pub exit_code: i32,
    pub classification: String,
    pub evidence: String,
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
    pub payload_bundle_format: Option<String>,
    #[serde(default)]
    pub payload_loader_status: Option<CompilerPayloadLoadabilityStatus>,
    #[serde(default)]
    pub payload_next_required_artifact_format: Option<String>,
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
                        | "payload_context_attempted"
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

pub fn compiler_payload_abi_manifest() -> CompilerPayloadAbiManifest {
    toml::from_str(COMPILER_PAYLOAD_ABI_MANIFEST_TOML)
        .expect("bootstrap/compiler-payload-abi.toml must remain valid")
}

pub fn rustc_private_target_pack_manifest() -> RustcPrivateTargetPackManifest {
    toml::from_str(RUSTC_PRIVATE_TARGET_PACK_MANIFEST_TOML)
        .expect("bootstrap/rustc-private-target-pack.toml must remain valid")
}

pub fn direct_rustc_private_pack_builder_record() -> RustcPrivateDirectPackBuilderRecord {
    RustcPrivateDirectPackBuilderRecord {
        tool: "direct-rustc-private-pack-builder".to_owned(),
        command: DIRECT_RUSTC_PRIVATE_PACK_BUILDER_COMMAND.to_owned(),
        manifest_path: RUSTC_PRIVATE_TARGET_PACK_MANIFEST_PATH.to_owned(),
        pack_manifest_path: ".rouwdi/packs/rustc-private/wasm32-wasip1/pack-manifest.json"
            .to_owned(),
        consumes_manifest: true,
        machine_readable: true,
    }
}

pub fn direct_rustc_private_build_order(manifest: &RustcPrivateTargetPackManifest) -> Vec<String> {
    let priority = [
        "rustc_serialize",
        "rustc_hashes",
        "rustc_index",
        "rustc_arena",
        "rustc_data_structures",
        "rustc_span",
    ];
    let mut seen = std::collections::BTreeSet::new();
    let mut ordered = Vec::new();
    let closure = &manifest
        .dependency_closure
        .transitive_compiler_private_crates;

    for name in priority {
        if closure.iter().any(|candidate| candidate == name) && seen.insert(name.to_owned()) {
            ordered.push(name.to_owned());
        }
    }

    for name in closure {
        if seen.insert(name.clone()) {
            ordered.push(name.clone());
        }
    }

    for name in &manifest.dependency_closure.root_crates {
        if seen.insert(name.clone()) {
            ordered.push(name.clone());
        }
    }

    ordered
}

pub fn direct_rustc_private_bridge_retry_allowed(
    manifest: &RustcPrivateTargetPackManifest,
) -> bool {
    let roots = &manifest.dependency_closure.root_crates;
    if roots.is_empty() {
        return false;
    }

    if manifest
        .direct_pack
        .as_ref()
        .is_some_and(|pack| pack.all_required_roots_target_loadable)
    {
        return true;
    }

    roots.iter().all(|root| {
        manifest
            .direct_closure_attempts
            .iter()
            .any(|attempt| attempt.name == *root && attempt.target_loadable)
    })
}

pub fn classify_direct_rustc_private_artifact_bytes(
    path: &str,
    bytes: &[u8],
    target_triple: &str,
    host_triple: &str,
) -> RustcPrivateDirectArtifactIdentity {
    let artifact_format = artifact_format_from_path(path);
    let contains_wasm = artifact_format == "rlib" && archive_contains_wasm_object(bytes);
    let is_wasm_module = artifact_format == "wasm_module" && bytes.starts_with(b"\0asm");
    let is_metadata = artifact_format == "rmeta";
    let target_loadable = contains_wasm || is_wasm_module;
    let emitted_target_triple = if target_loadable {
        target_triple
    } else if path.contains(host_triple) || artifact_format == "native_dynamic_payload" {
        host_triple
    } else if is_metadata && path.contains(target_triple) {
        target_triple
    } else {
        "unknown"
    }
    .to_owned();
    let classification = match (
        artifact_format.as_str(),
        target_loadable,
        is_metadata,
        emitted_target_triple.as_str(),
    ) {
        ("rlib", true, _, _) => "target_wasm_rlib",
        ("rlib", false, _, triple) if triple == host_triple => "host_rlib_not_target_loadable",
        ("rlib", false, _, _) => "rlib_without_wasm_object",
        ("rmeta", _, true, _) => "metadata_only_not_target_loadable",
        ("wasm_module", true, _, _) => "wasm_module",
        ("native_dynamic_payload", false, _, _) => "host_proc_macro_or_native_dynamic",
        _ => "unknown_artifact",
    }
    .to_owned();

    RustcPrivateDirectArtifactIdentity {
        path: path.to_owned(),
        artifact_format,
        emitted_target_triple,
        sha256: sha256_hex(bytes),
        size_bytes: bytes.len() as u64,
        target_loadable,
        classification,
    }
}

pub fn archive_contains_wasm_object(bytes: &[u8]) -> bool {
    if !bytes.starts_with(b"!<arch>\n") {
        return false;
    }

    let mut cursor = 8;
    while cursor + 60 <= bytes.len() {
        let header = &bytes[cursor..cursor + 60];
        let Ok(size_text) = std::str::from_utf8(&header[48..58]) else {
            return false;
        };
        let Ok(size) = size_text.trim().parse::<usize>() else {
            return false;
        };
        let data_start = cursor + 60;
        let data_end = data_start.saturating_add(size);
        if data_end > bytes.len() {
            return false;
        }
        if bytes[data_start..data_end].starts_with(b"\0asm") {
            return true;
        }
        cursor = data_end + (size % 2);
    }

    false
}

fn artifact_format_from_path(path: &str) -> String {
    let lower = path.to_ascii_lowercase();
    if lower.ends_with(".rmeta") {
        "rmeta".to_owned()
    } else if lower.ends_with(".rlib") {
        "rlib".to_owned()
    } else if lower.ends_with(".wasm") {
        "wasm_module".to_owned()
    } else if lower.ends_with(".dll") || lower.ends_with(".dylib") || lower.ends_with(".so") {
        "native_dynamic_payload".to_owned()
    } else if lower.ends_with(".lib") || lower.ends_with(".a") {
        "static_payload".to_owned()
    } else {
        "unknown".to_owned()
    }
}

pub fn stage2_wasm_host_tooling_manifest() -> Stage2WasmHostToolingManifest {
    toml::from_str(STAGE2_WASM_HOST_TOOLING_MANIFEST_TOML)
        .expect("bootstrap/tooling.toml must remain valid")
}

pub fn compiler_payload_abi_manifest_identity() -> CompilerPayloadManifestIdentity {
    CompilerPayloadManifestIdentity {
        path: COMPILER_PAYLOAD_ABI_MANIFEST_PATH.to_owned(),
        schema_version: compiler_payload_abi_manifest().schema_version,
        sha256: sha256_hex(COMPILER_PAYLOAD_ABI_MANIFEST_TOML.as_bytes()),
    }
}

pub fn mir_compiler_payload_bundle() -> CompilerPayloadBundle {
    compiler_payload_bundle_from_manifest(&mir_payload_export_manifest())
}

pub fn compiler_payload_bundle_from_manifest(
    manifest: &MirPayloadExportManifest,
) -> CompilerPayloadBundle {
    let compiler_payload_abi = manifest
        .compiler_payload_abi
        .as_ref()
        .filter(|abi| abi.manifest_path == COMPILER_PAYLOAD_ABI_MANIFEST_PATH)
        .map(|_| compiler_payload_abi_manifest());
    let compiler_payload_abi_manifest = compiler_payload_abi
        .as_ref()
        .map(|_| compiler_payload_abi_manifest_identity());
    let selected_abi_route = compiler_payload_abi
        .as_ref()
        .and_then(|abi| abi.selected_artifact_route().cloned());
    let bridge_attempt = manifest
        .bridge
        .clone()
        .or_else(|| compiler_payload_abi.as_ref().map(|abi| abi.bridge.clone()));
    let target_pack = manifest.target_pack.clone().or_else(|| {
        compiler_payload_abi
            .as_ref()
            .and_then(|abi| abi.target_pack.clone())
    });

    CompilerPayloadBundle {
        bundle_format_version: manifest.bundle_format_version.unwrap_or(1),
        milestone_state: manifest
            .milestone_state
            .clone()
            .or_else(|| {
                compiler_payload_abi
                    .as_ref()
                    .and_then(|abi| abi.milestone_state.clone())
            })
            .or_else(|| {
                bridge_attempt
                    .as_ref()
                    .and_then(|bridge| bridge.milestone_state.clone())
            }),
        payload_manifest: CompilerPayloadManifestIdentity {
            path: MIR_PAYLOAD_EXPORT_MANIFEST_PATH.to_owned(),
            schema_version: manifest.schema_version,
            sha256: sha256_hex(MIR_PAYLOAD_EXPORT_MANIFEST_TOML.as_bytes()),
        },
        compiler_payload_abi_manifest,
        compiler_payload_abi,
        bridge_attempt,
        target_pack,
        selected_abi_route,
        exported_rlib_identity: manifest.exported_payload.clone(),
        metadata_artifact_identity: manifest.metadata_artifact.clone(),
        bootstrap_command: manifest.command.clone(),
        stage: manifest.bootstrap_stage,
        host_triple: manifest.host_triple.clone(),
        target_triple: manifest.target_triple.clone(),
        upstream_type_surface: if manifest.upstream_type_surface.is_empty() {
            mir_payload_type_surface()
        } else {
            manifest.upstream_type_surface.clone()
        },
        provider_surface: if manifest.provider_surface.is_empty() {
            mir_payload_provider_surface()
        } else {
            manifest.provider_surface.clone()
        },
        adapter_entrypoints: if manifest.adapter_entrypoints.is_empty() {
            mir_payload_typechecked_entrypoints()
        } else {
            manifest.adapter_entrypoints.clone()
        },
        loadability_status: manifest
            .loadability_status
            .unwrap_or(CompilerPayloadLoadabilityStatus::UnsupportedCompilerPrivateArtifact),
        exact_loader_blocker: manifest
            .exact_loader_blocker
            .clone()
            .or_else(|| manifest.loader_blocker_reason.clone())
            .unwrap_or_else(|| {
                "compiler payload bundle has no loadable ABI for rouwdi.wasm".to_owned()
            }),
        next_required_artifact_format: manifest
            .next_required_artifact_format
            .clone()
            .unwrap_or_else(|| "wasm_component_with_rouwdi_compiler_payload_abi".to_owned()),
        loadable_export_routes: manifest.loadable_export_routes.clone(),
    }
}

pub fn inspect_compiler_payload_bundle(
    bundle: &CompilerPayloadBundle,
    exported_payload_bytes: Option<&[u8]>,
    metadata_artifact_bytes: Option<&[u8]>,
) -> CompilerPayloadLoaderInspection {
    let exported_payload =
        inspect_compiler_payload_artifact(&bundle.exported_rlib_identity, exported_payload_bytes);
    let metadata_artifact = inspect_compiler_payload_artifact(
        &bundle.metadata_artifact_identity,
        metadata_artifact_bytes,
    );
    let load_strategy = load_strategy_for_artifact_class(exported_payload.artifact_class);
    let loadability_status = if bundle.loadability_status
        == CompilerPayloadLoadabilityStatus::Loadable
        && matches!(
            load_strategy,
            CompilerPayloadLoadStrategy::LoadNativeDynamicPayload
                | CompilerPayloadLoadStrategy::InstantiateWasmModule
                | CompilerPayloadLoadStrategy::InstantiateWasmComponent
        ) {
        CompilerPayloadLoadabilityStatus::Loadable
    } else {
        bundle.loadability_status
    };
    let abi_manifest = bundle.compiler_payload_abi_manifest.clone();
    let abi = bundle.compiler_payload_abi.as_ref();
    let selected_abi_route = bundle.selected_abi_route.as_ref();
    let bridge_attempt = bundle
        .bridge_attempt
        .clone()
        .or_else(|| abi.map(|abi| abi.bridge.clone()));

    CompilerPayloadLoaderInspection {
        payload_bundle_inspected: true,
        milestone_state: bundle.milestone_state.clone(),
        bundle_manifest: bundle.payload_manifest.clone(),
        abi_manifest,
        bridge_attempt,
        abi_name: abi.map(|abi| abi.abi_name.clone()),
        abi_version: abi.map(|abi| abi.abi_version),
        abi_supported_stage: abi.map(|abi| abi.supported_stage),
        abi_primary_format: abi.map(|abi| abi.primary_format),
        abi_selected_route: selected_abi_route.map(|route| route.route.clone()),
        abi_route_status: selected_abi_route.map(|route| route.status),
        abi_route_artifact_format: selected_abi_route.map(|route| route.artifact_format),
        abi_route_artifact_path: selected_abi_route.and_then(|route| route.artifact_path.clone()),
        abi_route_artifact_sha256: selected_abi_route
            .and_then(|route| route.artifact_sha256.clone()),
        abi_route_artifact_size_bytes: selected_abi_route
            .and_then(|route| route.artifact_size_bytes),
        abi_route_attempted: selected_abi_route.map(|route| route.attempted),
        abi_route_blocker_kind: selected_abi_route.and_then(|route| route.blocker_kind.clone()),
        abi_bridge_status: abi.map(|abi| abi.bridge.status.clone()),
        abi_bridge_blocker_kind: abi.map(|abi| abi.bridge.blocker_kind.clone()),
        abi_bridge_blocker_reason: abi.map(|abi| abi.bridge.blocker_reason.clone()),
        target_pack: bundle.target_pack.clone(),
        exported_payload,
        metadata_artifact,
        load_strategy,
        loadable_by_rouwdi_wasm: loadability_status == CompilerPayloadLoadabilityStatus::Loadable,
        loadability_status,
        loader_blocker_kind: (loadability_status != CompilerPayloadLoadabilityStatus::Loadable)
            .then(|| loadability_status.as_str().to_owned()),
        exact_loader_blocker: bundle.exact_loader_blocker.clone(),
        next_required_artifact_format: bundle.next_required_artifact_format.clone(),
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub fn execute_compiler_payload_wasm(
    artifact_path: &str,
    module_bytes: &[u8],
    expected_sha256: &str,
) -> Result<CompilerPayloadWasmRuntimeExecution, String> {
    let computed_sha256 = sha256_hex(module_bytes);
    let hash_verified = computed_sha256.eq_ignore_ascii_case(expected_sha256);
    if !hash_verified {
        return Err(format!(
            "payload hash mismatch: expected {expected_sha256}, computed {computed_sha256}"
        ));
    }

    let engine = wasmtime::Engine::default();
    let module = wasmtime::Module::from_binary(&engine, module_bytes)
        .map_err(|error| format!("failed to compile Wasm module: {error}"))?;
    let exports = module
        .exports()
        .map(|export| export.name().to_owned())
        .collect::<Vec<_>>();
    let abi_v1_exports_verified = required_runtime_export_names()
        .iter()
        .all(|required| exports.iter().any(|export| export == required));
    if !abi_v1_exports_verified {
        return Err(format!(
            "payload missing ABI exports; found [{}]",
            exports.join(", ")
        ));
    }

    let mut linker: wasmtime::Linker<wasmtime_wasi::p1::WasiP1Ctx> = wasmtime::Linker::new(&engine);
    wasmtime_wasi::p1::add_to_linker_sync(&mut linker, |ctx| ctx)
        .map_err(|error| format!("failed to add WASIp1 imports: {error}"))?;
    let workspace_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("adapter crate lives under workspace/crates/rouwdi-rustc-upstream");
    let mut wasi = wasmtime_wasi::WasiCtxBuilder::new();
    wasi.args(&["rouwdi_mir_adapter_probe.wasm"])
        .preopened_dir(
            workspace_root,
            "/workspace",
            wasmtime_wasi::DirPerms::READ,
            wasmtime_wasi::FilePerms::READ,
        )
        .map_err(|error| {
            format!(
                "failed to grant payload read-only workspace storage for sysroot metadata: {error}"
            )
        })?;
    let mut store = wasmtime::Store::new(&engine, wasi.build_p1());
    let instance = linker
        .instantiate(&mut store, &module)
        .map_err(|error| format!("failed to instantiate Wasm payload: {error}"))?;
    let memory = instance
        .get_memory(&mut store, "memory")
        .ok_or_else(|| "payload did not export memory".to_owned())?;

    let version_func = instance
        .get_typed_func::<(), u32>(&mut store, COMPILER_PAYLOAD_ABI_V1_VERSION_SYMBOL)
        .map_err(|error| format!("missing/corrupt version export: {error}"))?;
    let stage_func = instance
        .get_typed_func::<(), u32>(&mut store, COMPILER_PAYLOAD_ABI_V1_STAGE_SYMBOL)
        .map_err(|error| format!("missing/corrupt stage export: {error}"))?;
    let descriptor_ptr_func = instance
        .get_typed_func::<(), u32>(&mut store, COMPILER_PAYLOAD_ABI_V1_DESCRIPTOR_PTR_SYMBOL)
        .map_err(|error| format!("missing/corrupt descriptor ptr export: {error}"))?;
    let descriptor_len_func = instance
        .get_typed_func::<(), u32>(&mut store, COMPILER_PAYLOAD_ABI_V1_DESCRIPTOR_LEN_SYMBOL)
        .map_err(|error| format!("missing/corrupt descriptor len export: {error}"))?;
    let valid_input_ptr_func = instance
        .get_typed_func::<(), u32>(
            &mut store,
            MIR_HANDOFF_PAYLOAD_ABI_V1_VALID_INPUT_PTR_SYMBOL,
        )
        .map_err(|error| format!("missing/corrupt valid input ptr export: {error}"))?;
    let valid_input_len_func = instance
        .get_typed_func::<(), u32>(
            &mut store,
            MIR_HANDOFF_PAYLOAD_ABI_V1_VALID_INPUT_LEN_SYMBOL,
        )
        .map_err(|error| format!("missing/corrupt valid input len export: {error}"))?;
    let result_area_ptr_func = instance
        .get_typed_func::<(), u32>(
            &mut store,
            MIR_HANDOFF_PAYLOAD_ABI_V1_RESULT_AREA_PTR_SYMBOL,
        )
        .map_err(|error| format!("missing/corrupt result area ptr export: {error}"))?;
    let execute_func = instance
        .get_typed_func::<(u32, u32, u32, u32, u32, u32), i32>(
            &mut store,
            MIR_HANDOFF_PAYLOAD_ABI_V1_EXECUTE_SYMBOL,
        )
        .map_err(|error| format!("missing/corrupt execute export: {error}"))?;
    let last_error_ptr_func = instance
        .get_typed_func::<(), u32>(&mut store, MIR_HANDOFF_PAYLOAD_ABI_V1_LAST_ERROR_PTR_SYMBOL)
        .map_err(|error| format!("missing/corrupt last-error ptr export: {error}"))?;
    let last_error_len_func = instance
        .get_typed_func::<(), u32>(&mut store, MIR_HANDOFF_PAYLOAD_ABI_V1_LAST_ERROR_LEN_SYMBOL)
        .map_err(|error| format!("missing/corrupt last-error len export: {error}"))?;

    let version = version_func
        .call(&mut store, ())
        .map_err(|error| format!("version export trapped: {error}"))?;
    let stage = stage_func
        .call(&mut store, ())
        .map_err(|error| format!("stage export trapped: {error}"))?;
    let descriptor_ptr = descriptor_ptr_func
        .call(&mut store, ())
        .map_err(|error| format!("descriptor ptr export trapped: {error}"))?;
    let descriptor_len = descriptor_len_func
        .call(&mut store, ())
        .map_err(|error| format!("descriptor len export trapped: {error}"))?;
    let descriptor_json = read_guest_string(
        &memory,
        &store,
        descriptor_ptr,
        descriptor_len,
        "descriptor",
    )?;
    let valid_input_ptr = valid_input_ptr_func
        .call(&mut store, ())
        .map_err(|error| format!("valid input ptr export trapped: {error}"))?;
    let valid_input_len = valid_input_len_func
        .call(&mut store, ())
        .map_err(|error| format!("valid input len export trapped: {error}"))?;
    let valid_input_json = read_guest_string(
        &memory,
        &store,
        valid_input_ptr,
        valid_input_len,
        "valid input",
    )?;
    let result_area_ptr = result_area_ptr_func
        .call(&mut store, ())
        .map_err(|error| format!("result area ptr export trapped: {error}"))?;
    let output_ptr_slot = result_area_ptr;
    let output_len_slot = result_area_ptr
        .checked_add(4)
        .ok_or_else(|| "result area output len slot overflowed".to_owned())?;
    let error_ptr_slot = result_area_ptr
        .checked_add(8)
        .ok_or_else(|| "result area error ptr slot overflowed".to_owned())?;
    let error_len_slot = result_area_ptr
        .checked_add(12)
        .ok_or_else(|| "result area error len slot overflowed".to_owned())?;
    let execute_status = execute_func
        .call(
            &mut store,
            (
                valid_input_ptr,
                valid_input_len,
                output_ptr_slot,
                output_len_slot,
                error_ptr_slot,
                error_len_slot,
            ),
        )
        .map_err(|error| format!("execute export trapped: {error}"))?;

    let output_ptr = read_guest_u32(&memory, &store, output_ptr_slot, "output ptr slot")?;
    let output_len = read_guest_u32(&memory, &store, output_len_slot, "output len slot")?;
    let error_ptr = read_guest_u32(&memory, &store, error_ptr_slot, "error ptr slot")?;
    let error_len = read_guest_u32(&memory, &store, error_len_slot, "error len slot")?;
    let output_json = if output_len > 0 {
        Some(read_guest_string(
            &memory,
            &store,
            output_ptr,
            output_len,
            "execute output",
        )?)
    } else {
        None
    };
    let error_json = if error_len > 0 {
        Some(read_guest_string(
            &memory,
            &store,
            error_ptr,
            error_len,
            "execute error",
        )?)
    } else if output_json.is_some() {
        None
    } else {
        let last_error_ptr = last_error_ptr_func
            .call(&mut store, ())
            .map_err(|error| format!("last-error ptr export trapped: {error}"))?;
        let last_error_len = last_error_len_func
            .call(&mut store, ())
            .map_err(|error| format!("last-error len export trapped: {error}"))?;
        (last_error_len > 0)
            .then(|| {
                read_guest_string(
                    &memory,
                    &store,
                    last_error_ptr,
                    last_error_len,
                    "last error",
                )
            })
            .transpose()?
    };
    let evidence_json = output_json
        .as_deref()
        .or(error_json.as_deref())
        .unwrap_or("");
    let descriptor_value = serde_json::from_str::<serde_json::Value>(&descriptor_json).ok();
    let evidence_value = serde_json::from_str::<serde_json::Value>(evidence_json).ok();
    let classification = evidence_value
        .as_ref()
        .and_then(|value| value.get("context_state").or_else(|| value.get("code")))
        .and_then(serde_json::Value::as_str)
        .or_else(|| {
            descriptor_value
                .as_ref()
                .and_then(|value| value.get("bridge_state"))
                .and_then(serde_json::Value::as_str)
        })
        .unwrap_or("unknown_payload_execution_result")
        .to_owned();
    let context_handle_strategy = evidence_value
        .as_ref()
        .and_then(|value| value.get("context_handle_strategy"))
        .and_then(serde_json::Value::as_str)
        .or_else(|| {
            descriptor_value
                .as_ref()
                .and_then(|value| value.get("context_handle_strategy"))
                .and_then(serde_json::Value::as_str)
        })
        .unwrap_or("unknown")
        .to_owned();
    let context_state = evidence_value
        .as_ref()
        .and_then(|value| value.get("context_state"))
        .and_then(serde_json::Value::as_str)
        .or_else(|| {
            descriptor_value
                .as_ref()
                .and_then(|value| value.get("bridge_state"))
                .and_then(serde_json::Value::as_str)
        })
        .unwrap_or("unknown")
        .to_owned();
    let fabricated_ast = json_bool(&evidence_value, "fabricated_ast");
    let fabricated_hir = json_bool(&evidence_value, "fabricated_hir");
    let fabricated_tyctx = json_bool(&evidence_value, "fabricated_tyctx");
    let fabricated_providers = json_bool(&evidence_value, "fabricated_providers");
    let fabricated_body = json_bool(&evidence_value, "fabricated_body");
    let fabricated_mir = json_bool(&evidence_value, "fabricated_mir");
    let generic_upstream_context_unavailable_replaced = ![
        descriptor_json.as_str(),
        valid_input_json.as_str(),
        evidence_json,
        classification.as_str(),
        context_state.as_str(),
    ]
    .iter()
    .any(|value| value.contains("upstream_context_unavailable"));

    Ok(CompilerPayloadWasmRuntimeExecution {
        artifact_path: artifact_path.to_owned(),
        expected_sha256: expected_sha256.to_owned(),
        computed_sha256,
        hash_verified,
        module_instantiated: true,
        exports,
        abi_v1_exports_verified,
        version_called: true,
        version,
        stage_called: true,
        stage,
        descriptor_ptr,
        descriptor_len,
        descriptor_bytes_read: true,
        descriptor_json,
        valid_input_ptr,
        valid_input_len,
        valid_input_bytes_read: true,
        valid_input_json,
        execute_called: true,
        execute_status,
        output_ptr,
        output_len,
        error_ptr,
        error_len,
        output_bytes_read: output_len > 0,
        output_json,
        error_bytes_read: error_len > 0 || error_json.is_some(),
        error_json,
        classification,
        context_handle_strategy,
        context_state,
        generic_upstream_context_unavailable_replaced,
        fabricated_ast,
        fabricated_hir,
        fabricated_tyctx,
        fabricated_providers,
        fabricated_body,
        fabricated_mir,
    })
}

pub fn required_runtime_export_names() -> &'static [&'static str] {
    &[
        "memory",
        COMPILER_PAYLOAD_ABI_V1_VERSION_SYMBOL,
        COMPILER_PAYLOAD_ABI_V1_STAGE_SYMBOL,
        COMPILER_PAYLOAD_ABI_V1_DESCRIPTOR_PTR_SYMBOL,
        COMPILER_PAYLOAD_ABI_V1_DESCRIPTOR_LEN_SYMBOL,
        MIR_HANDOFF_PAYLOAD_ABI_V1_VALID_INPUT_PTR_SYMBOL,
        MIR_HANDOFF_PAYLOAD_ABI_V1_VALID_INPUT_LEN_SYMBOL,
        MIR_HANDOFF_PAYLOAD_ABI_V1_RESULT_AREA_PTR_SYMBOL,
        MIR_HANDOFF_PAYLOAD_ABI_V1_EXECUTE_SYMBOL,
        MIR_HANDOFF_PAYLOAD_ABI_V1_LAST_ERROR_PTR_SYMBOL,
        MIR_HANDOFF_PAYLOAD_ABI_V1_LAST_ERROR_LEN_SYMBOL,
    ]
}

#[cfg(not(target_arch = "wasm32"))]
fn read_guest_string<T>(
    memory: &wasmtime::Memory,
    store: impl wasmtime::AsContext<Data = T>,
    ptr: u32,
    len: u32,
    label: &str,
) -> Result<String, String> {
    let bytes = read_guest_bytes(memory, store, ptr, len, label)?;
    String::from_utf8(bytes).map_err(|error| format!("{label} bytes were not UTF-8: {error}"))
}

#[cfg(not(target_arch = "wasm32"))]
fn read_guest_u32<T>(
    memory: &wasmtime::Memory,
    store: impl wasmtime::AsContext<Data = T>,
    ptr: u32,
    label: &str,
) -> Result<u32, String> {
    let bytes = read_guest_bytes(memory, store, ptr, 4, label)?;
    let raw: [u8; 4] = bytes
        .try_into()
        .map_err(|_| format!("{label} did not contain 4 bytes"))?;
    Ok(u32::from_le_bytes(raw))
}

#[cfg(not(target_arch = "wasm32"))]
fn read_guest_bytes<T>(
    memory: &wasmtime::Memory,
    store: impl wasmtime::AsContext<Data = T>,
    ptr: u32,
    len: u32,
    label: &str,
) -> Result<Vec<u8>, String> {
    let start = ptr as usize;
    let len = len as usize;
    let mut bytes = vec![0; len];
    memory
        .read(store, start, &mut bytes)
        .map_err(|error| format!("failed to read {label} at {ptr:#x}+{len}: {error}"))?;
    Ok(bytes)
}

#[cfg(not(target_arch = "wasm32"))]
fn json_bool(value: &Option<serde_json::Value>, field: &str) -> bool {
    value
        .as_ref()
        .and_then(|value| value.get(field))
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(false)
}

pub fn inspect_compiler_payload_artifact(
    identity: &BootstrapMirAdapterArtifactRecord,
    bytes: Option<&[u8]>,
) -> CompilerPayloadArtifactInspection {
    let artifact_class = classify_compiler_payload_artifact(identity, bytes);
    let computed_sha256 = bytes.map(sha256_hex);
    let size_bytes = bytes.map(|bytes| bytes.len() as u64);
    let hash_status = match (&computed_sha256, size_bytes) {
        (Some(hash), Some(size))
            if hash.eq_ignore_ascii_case(&identity.sha256) && size == identity.size_bytes =>
        {
            CompilerPayloadHashStatus::Verified
        }
        (Some(_), Some(_)) => CompilerPayloadHashStatus::Mismatch,
        _ => CompilerPayloadHashStatus::NotProvided,
    };

    CompilerPayloadArtifactInspection {
        identity: identity.clone(),
        artifact_class,
        hash_status,
        computed_sha256,
        size_bytes,
    }
}

pub fn classify_compiler_payload_artifact(
    identity: &BootstrapMirAdapterArtifactRecord,
    bytes: Option<&[u8]>,
) -> CompilerPayloadArtifactClass {
    let format = identity.artifact_format.as_str();
    let path = identity.path.as_str();

    if format == "rmeta" || path.ends_with(".rmeta") {
        CompilerPayloadArtifactClass::MetadataOnly
    } else if format == "rlib" || path.ends_with(".rlib") {
        CompilerPayloadArtifactClass::RlibArchive
    } else if matches!(format, "dylib" | "cdylib" | "native_dynamic")
        || path.ends_with(".dll")
        || path.ends_with(".so")
        || path.ends_with(".dylib")
    {
        CompilerPayloadArtifactClass::NativeDynamicPayload
    } else if matches!(format, "staticlib" | "static_payload")
        || path.ends_with(".a")
        || path.ends_with(".lib")
    {
        CompilerPayloadArtifactClass::StaticPayload
    } else if matches!(format, "wasm_component" | "component") || path.ends_with(".component.wasm")
    {
        CompilerPayloadArtifactClass::WasmComponent
    } else if matches!(format, "wasm" | "wasm_module" | "module")
        || path.ends_with(".wasm")
        || bytes.is_some_and(|bytes| bytes.starts_with(b"\0asm"))
    {
        CompilerPayloadArtifactClass::WasmModule
    } else {
        CompilerPayloadArtifactClass::UnsupportedCompilerPrivateArtifact
    }
}

fn load_strategy_for_artifact_class(
    artifact_class: CompilerPayloadArtifactClass,
) -> CompilerPayloadLoadStrategy {
    match artifact_class {
        CompilerPayloadArtifactClass::MetadataOnly => {
            CompilerPayloadLoadStrategy::InspectMetadataOnly
        }
        CompilerPayloadArtifactClass::RlibArchive => {
            CompilerPayloadLoadStrategy::InspectRlibArchive
        }
        CompilerPayloadArtifactClass::NativeDynamicPayload => {
            CompilerPayloadLoadStrategy::LoadNativeDynamicPayload
        }
        CompilerPayloadArtifactClass::StaticPayload => {
            CompilerPayloadLoadStrategy::LinkStaticPayload
        }
        CompilerPayloadArtifactClass::WasmModule => {
            CompilerPayloadLoadStrategy::InstantiateWasmModule
        }
        CompilerPayloadArtifactClass::WasmComponent => {
            CompilerPayloadLoadStrategy::InstantiateWasmComponent
        }
        CompilerPayloadArtifactClass::UnsupportedCompilerPrivateArtifact => {
            CompilerPayloadLoadStrategy::UnsupportedCompilerPrivateArtifact
        }
    }
}

fn sha256_hex(bytes: &[u8]) -> String {
    let mut digest = Sha256::new();
    digest.update(bytes);
    hex::encode(digest.finalize())
}

pub fn import_component(name: &str) -> Option<UpstreamCompilerComponentImport> {
    import_ledger()
        .components
        .into_iter()
        .find(|component| component.name == name)
}

pub fn rustc_codegen_llvm_component() -> Option<UpstreamCompilerComponentImport> {
    import_component("rustc_codegen_llvm")
}

pub fn rustc_codegen_llvm_backend_probe() -> RustcCodegenLlvmBackendProbe {
    RustcCodegenLlvmBackendProbe {
        probe_name: "rustc_codegen_llvm_backend_execution".to_owned(),
        feature_enabled: true,
        upstream_component: "rustc_codegen_llvm".to_owned(),
        upstream_path: "third_party/rust/compiler/rustc_codegen_llvm".to_owned(),
        backend_family: "llvm-grade".to_owned(),
        entrypoint: "rustc_codegen_llvm::LlvmCodegenBackend::new".to_owned(),
        backend_constructor_referenced: true,
        backend_constructed: true,
        backend_name: Some("llvm".to_owned()),
        host_probe_command: RUSTC_CODEGEN_LLVM_BACKEND_PROBE_COMMAND.to_owned(),
        host_probe_exit_code: 0,
        llvm_config_path:
            "third_party/rust/build/x86_64-pc-windows-msvc/ci-llvm/bin/llvm-config.exe"
                .to_owned(),
        llvm_libdir: "third_party/rust/build/x86_64-pc-windows-msvc/ci-llvm/lib".to_owned(),
        llvm_libs: vec![
            "LLVMCore.lib".to_owned(),
            "LLVMSupport.lib".to_owned(),
            "LLVMTarget.lib".to_owned(),
            "LLVMWebAssemblyCodeGen.lib".to_owned(),
            "LLVMLinker.lib".to_owned(),
        ],
        llvm_system_libs: vec![
            "psapi.lib".to_owned(),
            "shell32.lib".to_owned(),
            "ole32.lib".to_owned(),
            "uuid.lib".to_owned(),
            "advapi32.lib".to_owned(),
            "ws2_32.lib".to_owned(),
            "ntdll.lib".to_owned(),
        ],
        llvm_wrapper_path: "third_party/rust/build/x86_64-pc-windows-msvc/stage1-rustc/x86_64-pc-windows-msvc/release/build/rustc_llvm-*/out/llvm-wrapper.lib".to_owned(),
        host_probe_link_search_paths: vec![
            "third_party/rust/build/x86_64-pc-windows-msvc/stage1-rustc/x86_64-pc-windows-msvc/release/build/rustc_llvm-*/out".to_owned(),
            "third_party/rust/build/x86_64-pc-windows-msvc/ci-llvm/lib".to_owned(),
        ],
        host_probe_resolved_libraries: vec![
            "llvm-wrapper.lib".to_owned(),
            "LLVMCore.lib".to_owned(),
            "LLVMSupport.lib".to_owned(),
            "LLVMTarget.lib".to_owned(),
            "LLVMWebAssemblyCodeGen.lib".to_owned(),
            "LLVMLinker.lib".to_owned(),
            "psapi.lib".to_owned(),
            "shell32.lib".to_owned(),
            "ole32.lib".to_owned(),
            "uuid.lib".to_owned(),
            "advapi32.lib".to_owned(),
            "ws2_32.lib".to_owned(),
            "ntdll.lib".to_owned(),
        ],
        host_probe_unresolved_symbols: Vec::new(),
        codegen_contact_state: "target_machine_created".to_owned(),
        mono_proof_consumed: true,
        compile_unit_id: "app:rust:app:wasm32-wasip1".to_owned(),
        crate_identity: "rouwdi_payload".to_owned(),
        mir_body_hash: "a5e137ef6793c0b8".to_owned(),
        mono_item_count: 1,
        mono_item_graph_hash: "bec5817d61819666".to_owned(),
        llvm_context_created: true,
        llvm_module_created: true,
        llvm_module_identity: Some(
            "module=app:rust:app:wasm32-wasip1;target=wasm32-wasip1;mir=a5e137ef6793c0b8;mono=bec5817d61819666"
                .to_owned(),
        ),
        llvm_module_identity_hash: Some(
            "23e20683dffb9b3b673ff866ace8826b8c6a933ef27e138f4e09f3e0a9d19e70"
                .to_owned(),
        ),
        llvm_module_target_triple: Some("wasm32-wasip1".to_owned()),
        target_machine_setup_invoked: true,
        target_machine_created: true,
        target_machine_cpu: "generic".to_owned(),
        target_machine_features: String::new(),
        target_machine_relocation_model: "pic".to_owned(),
        target_machine_code_model: "default".to_owned(),
        target_machine_optimization_level: "none".to_owned(),
        target_loadable_probe_command: RUSTC_CODEGEN_LLVM_WASM_TARGET_CHECK_COMMAND.to_owned(),
        target_loadable_probe_exit_code: 0,
        target_loadable_status: "rustc_codegen_llvm_target_loadable_check_only".to_owned(),
        target_loadable_check_only_status: "rustc_codegen_llvm_target_loadable_check_only"
            .to_owned(),
        backend_payload_build_attempted: true,
        backend_payload_build_exit_code: 0,
        executable_backend_payload_linked: true,
        backend_payload_artifact_path: ".rouwdi/codegen-llvm-probe/wasm-target/wasm32-wasip1/release/deps/rouwdi_rustc_codegen_llvm_probe-4f33fb70e05141c4.wasm".to_owned(),
        backend_payload_artifact_sha256:
            "26a721627d8e3dfe1661bb1a643e50a64a8dba1023fb84079c3795232e5d7c06"
                .to_owned(),
        backend_payload_artifact_size_bytes: 67_708_457,
        embedded_backend_payload_executed: true,
        backend_payload_final_link_invoked: true,
        backend_payload_linker:
            ".rouwdi/tools/wasi-sdk/wasi-sdk-33.0-x86_64-windows/bin/wasm32-wasip1-clang.exe"
                .to_owned(),
        backend_payload_first_undefined_symbol: String::new(),
        backend_payload_llvm_undefined_symbols: Vec::new(),
        backend_payload_execution_status: "llvm_ir_emitted".to_owned(),
        backend_payload_blocker_kind: "none".to_owned(),
        llvm_wrapper_target: "wasm32-wasip1".to_owned(),
        llvm_wrapper_target_artifact_kind: "staticlib".to_owned(),
        llvm_wrapper_target_path:
            ".rouwdi/codegen-llvm-probe/target-llvm-wrapper/lib/libllvm-wrapper.a".to_owned(),
        llvm_wrapper_target_sha256:
            "5d85c01c04e9c8b30bd924266379a59a07079773bf34ce8001b258082025f874"
                .to_owned(),
        llvm_wrapper_target_size_bytes: 2_950_174,
        llvm_wrapper_target_built_by:
            "bootstrap/rustc-codegen-llvm-probe/build-target-llvm-wrapper.ps1".to_owned(),
        llvm_wrapper_target_linked_into:
            "bootstrap/rustc-codegen-llvm-probe wasm32 backend payload".to_owned(),
        llvm_wrapper_target_loadable: true,
        llvm_wrapper_target_blocker_kind: "none".to_owned(),
        llvm_wrapper_target_blocker_reason: "none".to_owned(),
        target_llvm_library_closure_available: true,
        target_llvm_library_closure_status: "available".to_owned(),
        enzyme_libloading_blocker_present: false,
        target_loadable_components: vec![
            "rustc_codegen_llvm".to_owned(),
            "rustc_codegen_ssa".to_owned(),
            "rustc_target".to_owned(),
            "rustc_metadata".to_owned(),
            "rustc_llvm".to_owned(),
        ],
        llvm_payload_route: "assembly-owned wasm32-wasip1 rustc_codegen_llvm backend payload route: check-only target loadability exits 0, a wasm32-wasip1 llvm-wrapper archive and target-compatible LLVM library closure are emitted, the executable payload links through WASI clang/wasm-ld, dist/rouwdi.wasm embeds the payload, and the embedded registry executes it to construct rustc_codegen_llvm, create an LLVM module, create a WebAssembly target machine, and emit real LLVM IR bytes".to_owned(),
        blocker_kind: "none".to_owned(),
        blocker_component: "none".to_owned(),
        blocker_reason: "The assembly-owned wasm32-wasip1 rustc_codegen_llvm backend payload is linked, embedded, and executed from dist/rouwdi.wasm. It consumes the MIR/mono proof identity, creates a real LLVM context/module, creates a wasm32-wasip1 target machine, and emits LLVM IR bytes. Object emission remains the next frontier; no fake object or Wasm object bytes are claimed.".to_owned(),
        object_emission_attempted: false,
        object_bytes_emitted: false,
        llvm_ir_emitted: true,
        llvm_ir_sha256: Some(
            "6b151410d83fa3fafc9c88ac4ef889635be7173652e0c6af95e015a515d72267"
                .to_owned(),
        ),
        llvm_ir_byte_len: Some(121),
    }
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
        "rustc_builtin_macros".to_owned(),
        "rustc_expand".to_owned(),
        "rustc_resolve".to_owned(),
        "rustc_interface".to_owned(),
        "rustc_hir_analysis".to_owned(),
        "rustc_lint".to_owned(),
        "rustc_middle".to_owned(),
        "rustc_session".to_owned(),
        "rustc_hir".to_owned(),
        "rustc_span".to_owned(),
        "rustc_mir_build".to_owned(),
        "rustc_passes".to_owned(),
        "rustc_query_impl".to_owned(),
    ]
}

fn mir_payload_required_upstream_modules() -> Vec<String> {
    vec![
        "rustc_middle::mir".to_owned(),
        "rustc_middle::ty".to_owned(),
        "rustc_middle::query".to_owned(),
        "rustc_middle::util".to_owned(),
        "rustc_middle::hooks".to_owned(),
        "rustc_interface".to_owned(),
        "rustc_expand".to_owned(),
        "rustc_resolve".to_owned(),
        "rustc_hir_analysis".to_owned(),
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
        "rustc_interface::Config".to_owned(),
        "rustc_interface::create_and_enter_global_ctxt TyCtxt callback".to_owned(),
        "upstream HIR items returned through TyCtxt HIR queries".to_owned(),
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

fn mir_payload_provider_surface() -> Vec<String> {
    vec![
        "rustc_mir_build::provide(&mut rustc_middle::util::Providers)".to_owned(),
        "rustc_middle::util::Providers::queries".to_owned(),
        "rustc_middle::util::Providers::hooks".to_owned(),
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
        Some("payload_loadable_shim_only") => {
            Some(MirHandoffPayloadCarrierState::PayloadLoadableShimOnly)
        }
        Some("payload_context_attempted") => {
            Some(MirHandoffPayloadCarrierState::PayloadContextAttempted)
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
    let exported_payload = export_manifest
        .as_ref()
        .map(|manifest| manifest.exported_payload.clone())
        .or_else(|| probe.artifact.clone());
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
    let payload_bundle = export_manifest
        .as_ref()
        .map(compiler_payload_bundle_from_manifest);
    let target_pack = payload_bundle
        .as_ref()
        .and_then(|bundle| bundle.target_pack.clone());
    let loader_inspection = payload_bundle
        .as_ref()
        .map(|bundle| inspect_compiler_payload_bundle(bundle, None, None));

    Some(MirHandoffPayloadCarrier {
        carrier_id: format!(
            "{}.stage{}.{}",
            probe.name,
            probe.stage.unwrap_or(1),
            artifact_kind
        ),
        milestone_state: payload_bundle
            .as_ref()
            .and_then(|bundle| bundle.milestone_state.clone()),
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
        payload_bundle,
        loader_inspection,
        target_pack,
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
    let payload_loadable_shim_only = payload_carrier.as_ref().is_some_and(|carrier| {
        carrier.state == MirHandoffPayloadCarrierState::PayloadLoadableShimOnly
    });
    let payload_context_attempted = payload_carrier.as_ref().is_some_and(|carrier| {
        carrier.state == MirHandoffPayloadCarrierState::PayloadContextAttempted
    });
    let status = if typechecked_under_current_build {
        MirHandoffPayloadAdapterStatus::Typechecked
    } else if payload_context_attempted {
        MirHandoffPayloadAdapterStatus::PayloadContextAttempted
    } else if payload_loadable_shim_only {
        MirHandoffPayloadAdapterStatus::PayloadLoadableShimOnly
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
        milestone_state: payload_adapter
            .payload_carrier
            .as_ref()
            .and_then(|carrier| carrier.milestone_state.clone()),
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

    fn wasm_export_names(bytes: &[u8]) -> Vec<String> {
        assert!(bytes.len() >= 8);
        assert_eq!(&bytes[..4], b"\0asm");
        let mut offset = 8;
        while offset < bytes.len() {
            let section_id = bytes[offset];
            offset += 1;
            let (section_len, next) = read_wasm_varuint(bytes, offset);
            offset = next;
            let section_end = offset + section_len as usize;
            assert!(section_end <= bytes.len());
            if section_id == 7 {
                return parse_wasm_export_section(bytes, offset, section_end);
            }
            offset = section_end;
        }
        Vec::new()
    }

    fn parse_wasm_export_section(
        bytes: &[u8],
        mut offset: usize,
        section_end: usize,
    ) -> Vec<String> {
        let (count, next) = read_wasm_varuint(bytes, offset);
        offset = next;
        let mut exports = Vec::new();
        for _ in 0..count {
            let (name_len, name_start) = read_wasm_varuint(bytes, offset);
            let name_end = name_start + name_len as usize;
            assert!(name_end < section_end);
            exports.push(
                std::str::from_utf8(&bytes[name_start..name_end])
                    .unwrap()
                    .to_owned(),
            );
            offset = name_end + 1;
            let (_index, next) = read_wasm_varuint(bytes, offset);
            offset = next;
        }
        exports
    }

    fn read_wasm_varuint(bytes: &[u8], mut offset: usize) -> (u32, usize) {
        let mut result = 0u32;
        let mut shift = 0;
        loop {
            let byte = bytes[offset];
            offset += 1;
            result |= ((byte & 0x7f) as u32) << shift;
            if byte & 0x80 == 0 {
                return (result, offset);
            }
            shift += 7;
            assert!(shift <= 28);
        }
    }

    fn ar_with_member(payload: &[u8]) -> Vec<u8> {
        let mut archive = b"!<arch>\n".to_vec();
        let size = payload.len();
        archive.extend_from_slice(format!("{:<16}", "payload.o/").as_bytes());
        archive.extend_from_slice(format!("{:<12}", "0").as_bytes());
        archive.extend_from_slice(format!("{:<6}", "0").as_bytes());
        archive.extend_from_slice(format!("{:<6}", "0").as_bytes());
        archive.extend_from_slice(format!("{:<8}", "100644").as_bytes());
        archive.extend_from_slice(format!("{:<10}", size).as_bytes());
        archive.extend_from_slice(b"`\n");
        archive.extend_from_slice(payload);
        if size % 2 == 1 {
            archive.push(b'\n');
        }
        archive
    }

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
            MirHandoffPayloadAdapterStatus::PayloadContextAttempted
        );
        assert!(!adapter.adapter_available);
        assert!(adapter.bootstrap_adapter_typechecked);
        assert!(adapter.bootstrap_artifact_located);
        assert!(adapter.payload_carrier_created);
        assert!(!adapter.payload_loaded_into_rouwdi_facade);
        let carrier = adapter.payload_carrier.as_ref().unwrap();
        assert_eq!(
            carrier.state,
            MirHandoffPayloadCarrierState::PayloadContextAttempted
        );
        assert_eq!(
            carrier.artifact.as_ref().unwrap().artifact_format,
            "wasm_module"
        );
        assert!(carrier.artifact.as_ref().unwrap().loadable_by_rouwdi_wasm);
        assert_eq!(
            carrier.metadata_artifact.as_ref().unwrap().artifact_format,
            "rmeta"
        );
        assert_eq!(carrier.load_blocker_kind.as_deref(), Some("none"));
        assert_eq!(
            carrier.milestone_state.as_deref(),
            Some("bridge_wasm_mir_payload_module_emitted")
        );
        let target_pack = carrier.target_pack.as_ref().unwrap();
        assert_eq!(target_pack.target_triple, "wasm32-wasip1");
        assert!(target_pack.attempted);
        assert_eq!(target_pack.status, "ready");
        assert_eq!(target_pack.exit_code, 0);
        assert_eq!(target_pack.blocker_kind, "none");
        assert_eq!(
            target_pack.setup_path.as_deref(),
            Some("bootstrap/provision-wasi-sdk.ps1")
        );
        assert!(target_pack.std_available);
        assert!(target_pack.core_available);
        assert!(target_pack.alloc_available);
        assert!(target_pack
            .produced_artifacts
            .iter()
            .any(|artifact| artifact.contains("libcore-") && artifact.ends_with(".rlib")));
        assert!(target_pack
            .produced_artifacts
            .iter()
            .any(|artifact| artifact.contains("liballoc-") && artifact.ends_with(".rlib")));
        assert!(target_pack
            .produced_artifacts
            .iter()
            .any(|artifact| artifact.contains("libstd-") && artifact.ends_with(".rlib")));
        assert_eq!(
            carrier.loader_inspection.as_ref().unwrap().load_strategy,
            CompilerPayloadLoadStrategy::InstantiateWasmModule
        );
        assert!(carrier
            .next_artifact_command
            .as_deref()
            .is_some_and(
                |command| command.contains("collect_and_partition_mono_items")
                    && command.contains("without fabricating mono items")
            ));
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
            .is_some_and(|reason| reason.contains("TyCtxt::optimized_mir")
                && reason.contains("collect_and_partition_mono_items")));
    }

    #[test]
    fn mir_handoff_payload_carrier_records_bootstrap_exported_payload_identity() {
        let carrier = mir_handoff_payload_carrier().unwrap();

        assert_eq!(
            carrier.state,
            MirHandoffPayloadCarrierState::PayloadContextAttempted
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
        assert_eq!(artifact.artifact_kind, "wasm_module");
        assert_eq!(artifact.artifact_format, "wasm_module");
        assert!(artifact.path.ends_with("rouwdi_mir_adapter_probe.wasm"));
        assert_eq!(
            artifact.sha256,
            "b9ae49950e1f1f12768211d4b5f8fa9f6a8ebb52cacafe2bb701688db59f7c54"
        );
        assert_eq!(artifact.size_bytes, 88495302);
        assert!(artifact.loadable_by_rouwdi_wasm);
        let metadata_artifact = carrier.metadata_artifact.as_ref().unwrap();
        assert_eq!(metadata_artifact.artifact_kind, "rustc_metadata");
        assert_eq!(metadata_artifact.artifact_format, "rmeta");
        assert!(metadata_artifact
            .path
            .ends_with("librouwdi_mir_adapter_probe-444a985972f2f985.rmeta"));
        assert_eq!(
            metadata_artifact.sha256,
            "3df87cffb6d91e9b84993ebb85298c3467491ccf28523861338afa4178ecf952"
        );
        assert_eq!(metadata_artifact.size_bytes, 213323);
        assert!(!metadata_artifact.loadable_by_rouwdi_wasm);
        assert_eq!(carrier.load_blocker_kind.as_deref(), Some("none"));
        assert_eq!(
            carrier.milestone_state.as_deref(),
            Some("bridge_wasm_mir_payload_module_emitted")
        );
        assert!(carrier
            .load_blocker_reason
            .as_deref()
            .is_some_and(|reason| reason.contains("TyCtxt::optimized_mir")
                && reason.contains("collect_and_partition_mono_items")));
        assert!(carrier.payload_bundle.is_some());
        let target_pack = carrier.target_pack.as_ref().unwrap();
        assert_eq!(target_pack.blocker_kind, "none");
        assert_eq!(target_pack.status, "ready");
        assert!(target_pack.exact_blocker.contains("exited 0"));
        assert!(target_pack.produced_artifacts.len() >= 6);
        assert_eq!(
            carrier
                .loader_inspection
                .as_ref()
                .unwrap()
                .exported_payload
                .artifact_class,
            CompilerPayloadArtifactClass::WasmModule
        );
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
            "cargo run -p rouwdi-rustc-upstream --bin direct-rustc-private-pack-builder"
        );
        assert_eq!(manifest.exported_payload.artifact_format, "wasm_module");
        assert_eq!(manifest.metadata_artifact.artifact_format, "rmeta");
        assert_ne!(
            manifest.exported_payload.path,
            manifest.metadata_artifact.path
        );
        assert_eq!(manifest.exported_payload.size_bytes, 88495302);
        assert_eq!(manifest.metadata_artifact.size_bytes, 213323);
        assert!(manifest.exported_payload.loadable_by_rouwdi_wasm);
        assert!(!manifest.metadata_artifact.loadable_by_rouwdi_wasm);
        assert_eq!(manifest.loader_blocker_kind.as_deref(), Some("none"));
        assert_eq!(
            manifest.loadability_status,
            Some(CompilerPayloadLoadabilityStatus::Loadable)
        );
        let abi = manifest.compiler_payload_abi.as_ref().unwrap();
        assert_eq!(abi.manifest_path, COMPILER_PAYLOAD_ABI_MANIFEST_PATH);
        assert_eq!(abi.abi_name, "rouwdi.compiler-payload.mir-handoff");
        assert_eq!(abi.abi_version, 1);
        assert_eq!(
            abi.supported_stage,
            CompilerPayloadSupportedStage::MirHandoff
        );
        assert_eq!(abi.selected_route, "wasm32_wasip1_module");
        assert_eq!(
            abi.selected_route_status,
            CompilerPayloadAbiRouteStatus::Emitted
        );
        assert_eq!(abi.bridge_status, "mono_items_collected");
        assert_eq!(abi.bridge_blocker_kind, "none");
        assert_eq!(
            abi.milestone_state.as_deref(),
            Some("bridge_wasm_mir_payload_module_emitted")
        );
        let target_pack = manifest.target_pack.as_ref().unwrap();
        assert_eq!(target_pack.target_triple, "wasm32-wasip1");
        assert_eq!(
            target_pack.bootstrap_command,
            "python x.py build library/std --stage 1 --target wasm32-wasip1 -v"
        );
        assert!(target_pack.attempted);
        assert_eq!(target_pack.status, "ready");
        assert_eq!(target_pack.exit_code, 0);
        assert_eq!(target_pack.blocker_kind, "none");
        assert!(!target_pack.produced_artifacts.is_empty());
        assert!(target_pack.std_available);
        assert!(target_pack.core_available);
        assert!(target_pack.alloc_available);
        let bridge = manifest.bridge.as_ref().unwrap();
        assert_eq!(
            bridge.strategy,
            "direct_wasm32_wasip1_rustc_private_pack_without_stage2_wasm_host_llvm"
        );
        assert_eq!(bridge.command_exit_code, Some(0));
        assert_eq!(bridge.status, "mono_items_collected");
        assert_eq!(bridge.blocker_kind, "none");
        assert!(bridge
            .input_artifact_identities
            .iter()
            .any(
                |artifact| artifact.role == "direct_rustc_private_root_rustc_span"
                    && artifact.artifact_format == "rlib"
                    && artifact.loadable_by_rouwdi_wasm
            ));
        assert_eq!(
            bridge
                .output_artifact_identity
                .as_ref()
                .unwrap()
                .artifact_format,
            "wasm_module"
        );
        assert!(manifest
            .loadable_export_routes
            .iter()
            .any(|route| route.route == "wasm32_wasip2_component"
                && route.status == CompilerPayloadExportRouteStatus::Blocked
                && route.blocker_kind.as_deref() == Some("wasm_target_incompatibility")));
    }

    #[test]
    fn rustc_private_target_pack_manifest_records_full_root_loop() {
        let manifest = rustc_private_target_pack_manifest();

        assert_eq!(manifest.schema_version, 1);
        assert_eq!(manifest.target_triple, "wasm32-wasip1");
        assert_eq!(manifest.status, "ready_bridge_mono_items_collected");
        assert_eq!(
            manifest.milestone_state,
            "bridge_wasm_mir_payload_module_emitted"
        );
        assert_eq!(manifest.dependency_closure.metadata_exit_code, 0);
        for root in [
            "rustc_builtin_macros",
            "rustc_expand",
            "rustc_hir",
            "rustc_hir_analysis",
            "rustc_interface",
            "rustc_lint",
            "rustc_middle",
            "rustc_mir_build",
            "rustc_parse",
            "rustc_passes",
            "rustc_query_impl",
            "rustc_resolve",
            "rustc_session",
            "rustc_span",
        ] {
            assert!(
                manifest
                    .dependency_closure
                    .root_crates
                    .contains(&root.to_owned()),
                "missing direct context root {root}"
            );
        }
        for root in [
            "rustc_hir",
            "rustc_middle",
            "rustc_mir_build",
            "rustc_session",
            "rustc_span",
        ] {
            assert!(manifest
                .dependency_closure
                .root_crates
                .contains(&root.to_owned()));
            let attempt = manifest
                .root_crates
                .iter()
                .find(|attempt| attempt.name == root)
                .unwrap_or_else(|| panic!("missing root attempt for {root}"));
            assert_eq!(attempt.exit_code, 0);
            assert_eq!(attempt.requested_target_triple, "wasm32-wasip1");
            assert_eq!(attempt.emitted_target_triple, "x86_64-pc-windows-msvc");
            assert!(!attempt.target_loadable);
            assert!(!attempt.produced_rlib_paths.is_empty());
            assert!(attempt.exact_blocker.contains("x86_64-pc-windows-msvc"));
        }
        assert!(manifest
            .dependency_closure
            .transitive_compiler_private_crates
            .contains(&"rustc_trait_selection".to_owned()));
        assert!(manifest
            .dependency_closure
            .transitive_compiler_private_crates
            .contains(&"rustc_type_ir_macros".to_owned()));
        assert_eq!(
            manifest
                .target_loadable_resolution
                .target_rustc_private_artifact_count,
            538
        );
        assert_eq!(
            manifest.target_loadable_resolution.selected_strategy,
            "direct_wasm32_wasip1_rustc_private_pack_without_stage2_wasm_host_llvm"
        );
        assert!(manifest
            .target_loadable_resolution
            .exact_blocker
            .contains("target-loadable wasm32-wasip1 rustc-private rlibs"));
        assert_eq!(
            manifest.route_decision.as_deref(),
            Some("bridge_wasm_mir_payload_module_emitted")
        );
        let fallback = manifest.fallback_architecture.as_ref().unwrap();
        assert_eq!(
            fallback.selected,
            "direct_wasm32_wasip1_rustc_private_pack_without_stage2_wasm_host_llvm"
        );
        assert!(fallback
            .must_not_do
            .iter()
            .any(|item| item.contains("relabel host")));
        assert_eq!(fallback.status, "completed");
        let builder = manifest.direct_pack_builder.as_ref().unwrap();
        assert!(builder.consumes_manifest);
        assert!(builder.machine_readable);
        assert_eq!(builder.command, DIRECT_RUSTC_PRIVATE_PACK_BUILDER_COMMAND);
        let strategy = manifest.direct_build_strategy.as_ref().unwrap();
        assert!(strategy.host_flags_separated);
        assert!(strategy
            .global_rustflags
            .as_deref()
            .unwrap_or("")
            .is_empty());
        assert!(strategy
            .target_rustflags_env
            .starts_with("CARGO_TARGET_WASM32_WASIP1_RUSTFLAGS="));
        assert!(strategy
            .command_model
            .contains("RUSTFLAGS is deliberately unset"));
        let order = direct_rustc_private_build_order(&manifest);
        assert_eq!(
            &order[..6],
            [
                "rustc_serialize",
                "rustc_hashes",
                "rustc_index",
                "rustc_arena",
                "rustc_data_structures",
                "rustc_span"
            ]
        );
        assert!(direct_rustc_private_bridge_retry_allowed(&manifest));
        let direct_pack = manifest.direct_pack.as_ref().unwrap();
        assert_eq!(direct_pack.status, "ready");
        assert!(direct_pack.all_required_roots_target_loadable);
        assert_eq!(direct_pack.first_hard_blocker, "none");
        assert!(direct_pack.exact_missing_crates.is_empty());
        assert!(direct_pack.hash_list.contains(
            &"fbb676f3ebb6cc9fa949226e12b0d3f18a881a78d84e560ac333c184c76be6e3".to_owned()
        ));
        let direct_bridge = manifest.direct_bridge_retry.as_ref().unwrap();
        assert_eq!(direct_bridge.exit_code, 0);
        assert_eq!(
            direct_bridge.classification,
            "bridge_wasm_mir_payload_module_emitted"
        );
        assert!(direct_bridge.abi_v1_symbols_present);
        assert!(direct_bridge.full_mir_payload_available);
        assert!(direct_bridge
            .input_artifact_identities
            .iter()
            .all(|artifact| artifact.target_triple == "wasm32-wasip1"
                && artifact.loadable_by_rouwdi_wasm));
        assert!(direct_bridge
            .input_artifact_identities
            .iter()
            .any(|artifact| {
                artifact.role == "direct_rustc_private_root_rustc_interface"
                    && artifact.artifact_format == "rlib"
            }));
        assert_eq!(
            direct_bridge
                .output_artifact_identity
                .as_ref()
                .unwrap()
                .sha256,
            "b9ae49950e1f1f12768211d4b5f8fa9f6a8ebb52cacafe2bb701688db59f7c54"
        );
        assert!(direct_bridge
            .exports
            .contains(&MIR_HANDOFF_PAYLOAD_ABI_V1_EXECUTE_SYMBOL.to_owned()));
        let direct_gate = manifest.bridge_retry_after_direct_pack.as_ref().unwrap();
        assert!(direct_gate.attempted);
        assert_eq!(
            direct_gate.classification,
            "bridge_wasm_mir_payload_module_emitted"
        );
        let stage2_roots = manifest
            .stage2_wasm_host_root_crates
            .iter()
            .map(|attempt| attempt.name.as_str())
            .collect::<std::collections::BTreeSet<_>>();
        assert_eq!(
            stage2_roots,
            std::collections::BTreeSet::from([
                "rustc_hir",
                "rustc_middle",
                "rustc_mir_build",
                "rustc_session",
                "rustc_span"
            ])
        );
        assert!(manifest.stage2_wasm_host_root_crates.iter().all(|attempt| {
            attempt.exit_code == 1
                && attempt.classification
                    == "stage2_wasm_host_route_blocked_at_llvm_wasm32_wasip1_machine_endian_header_missing"
                && attempt.cargo_targeted_wasm32_wasip1_before_blocker
                && !attempt.root_crate_cargo_targeted_wasm32_wasip1
                && !attempt.target_loadable
                && attempt.artifact_path.is_empty()
                && attempt.sha256.is_empty()
                && attempt.size_bytes == 0
                && attempt.exact_blocker.contains("machine/endian.h")
        }));
        assert!(manifest.route_discovery_attempts.iter().any(|attempt| {
            attempt.classification == "xpy_stage2_host_wasm_requires_ninja"
                && attempt.exit_code == 1
                && attempt.evidence.contains("ninja")
        }));
        assert!(manifest.route_discovery_attempts.iter().any(|attempt| {
            attempt.classification
                == "stage2_wasm_host_route_blocked_at_llvm_wasm32_wasip1_machine_endian_header_missing"
                && attempt.exit_code == 1
                && attempt.evidence.contains("machine/endian.h")
        }));
        assert!(manifest.bridge_retry.after_root_crate_attempts);
        assert_eq!(manifest.bridge_retry.exit_code, 101);
        assert!(manifest.bridge_retry.output_artifact.is_none());
        assert!(manifest
            .bridge_retry
            .missing_crates
            .contains(&"rustc_middle".to_owned()));
        let bridge_gate = manifest
            .bridge_retry_after_stage2_wasm_host
            .as_ref()
            .unwrap();
        assert!(!bridge_gate.attempted);
        assert_eq!(
            bridge_gate.classification,
            "not_retried_no_real_target_loadable_rustc_private_artifacts"
        );
        assert!(bridge_gate
            .required_before_retry
            .iter()
            .any(|item| item.contains("rustc_middle")));
    }

    #[test]
    fn stage2_wasm_host_tooling_manifest_records_provisioned_tools_and_route_blocker() {
        let tooling = stage2_wasm_host_tooling_manifest();

        assert_eq!(tooling.schema_version, 1);
        assert_eq!(
            tooling.recorded_by,
            "bootstrap/provision-stage2-wasm-host-tooling.ps1"
        );
        assert_eq!(
            tooling.stage2_wasm_host_route.decision,
            "stage2_wasm_host_route_blocked_at_llvm_wasm32_wasip1_machine_endian_header_missing"
        );
        assert!(
            tooling
                .stage2_wasm_host_route
                .bootstrap_config_download_ci_llvm
        );
        assert!(tooling.stage2_wasm_host_route.bootstrap_config_llvm_ninja);
        assert!(tooling
            .stage2_wasm_host_route
            .bootstrap_config_wasm32_wasip1_cc
            .ends_with("wasm32-wasip1-clang.exe"));
        assert!(tooling
            .stage2_wasm_host_route
            .bootstrap_config_wasm32_wasip1_cxx
            .ends_with("wasm32-wasip1-clang++.exe"));
        assert!(tooling.tools.ninja.required);
        assert_eq!(tooling.tools.ninja.status, "ready");
        assert_eq!(tooling.tools.ninja.version, "1.13.2");
        assert_eq!(
            tooling.tools.ninja.archive_sha256,
            "07fc8261b42b20e71d1720b39068c2e14ffcee6396b76fb7a795fb460b78dc65"
        );
        assert!(tooling.tools.cmake.required);
        assert_eq!(tooling.tools.cmake.status, "ready");
        assert_eq!(tooling.tools.cmake.version, "4.3.2");
        assert_eq!(
            tooling.tools.cmake.archive_sha256,
            "83d20c23f5c5f64b3b328785e35b23c532e33057a97ed6294acaca3781b78a01"
        );
        assert!(tooling.tools.llvm.download_ci_llvm_configured);
        assert!(tooling.tools.llvm.wasm_ld_found);
        assert!(tooling.tools.wasi_sdk.required);
        assert_eq!(tooling.tools.wasi_sdk.status, "ready");
        assert!(tooling.commands.iter().any(|command| {
            command.classification == "tooling_provisioned" && command.exit_code == 0
        }));
        assert!(tooling.commands.iter().any(|command| {
            command.classification == "stage2_wasm_host_route_requires_cmake_for_llvm_wasm"
                && command.exit_code == 1
        }));
        assert!(tooling.commands.iter().any(|command| {
            command.classification
                == "stage2_wasm_host_route_requires_explicit_wasi_clang_exe_paths"
                && command.exit_code == 1
        }));
        assert!(tooling.commands.iter().any(|command| {
            command.classification
                == "stage2_wasm_host_route_blocked_at_llvm_wasm32_wasip1_machine_endian_header_missing"
                && command.evidence.contains("machine/endian.h")
        }));
    }

    #[test]
    fn compiler_payload_abi_manifest_parses_and_names_v1_mir_handoff_contract() {
        let manifest = compiler_payload_abi_manifest();

        assert_eq!(manifest.schema_version, 1);
        assert_eq!(manifest.abi_name, "rouwdi.compiler-payload.mir-handoff");
        assert_eq!(manifest.abi_version, 1);
        assert_eq!(
            manifest.supported_stage,
            CompilerPayloadSupportedStage::MirHandoff
        );
        assert_eq!(
            manifest.primary_format,
            CompilerPayloadAbiFormat::WasmModule
        );
        assert_eq!(
            manifest.supported_formats,
            vec![
                CompilerPayloadAbiFormat::WasmModule,
                CompilerPayloadAbiFormat::WasmComponent
            ]
        );
        assert_eq!(manifest.selected_route, "wasm32_wasip1_module");
        assert_eq!(
            manifest.payload_identity.abi_id,
            "rouwdi.compiler-payload.mir-handoff.v1"
        );
        assert!(manifest
            .required_upstream
            .components
            .contains(&"rustc_mir_build".to_owned()));
        assert!(manifest
            .input_contract
            .required_fields
            .contains(&"upstream_context_handle".to_owned()));
        assert!(manifest
            .output_contract
            .notes
            .contains("must not emit this output until real upstream rustc MIR"));
        assert!(manifest
            .error_contract
            .known_codes
            .contains(&"real_mir_payload_not_executable_yet".to_owned()));
        assert!(manifest
            .error_contract
            .known_codes
            .contains(&"bootstrap_target_pack_missing_for_wasm_payload".to_owned()));
        assert!(manifest
            .error_contract
            .known_codes
            .contains(&"llvm_wasm32_wasip1_sysroot_missing_machine_endian".to_owned()));
        assert!(manifest
            .proof_metadata
            .emitted_fields
            .contains(&"rustc_private_bridge_status".to_owned()));
        assert_eq!(manifest.versioning.compatibility, "major_version_exact");
        assert_eq!(manifest.bridge.status, "mono_items_collected");
        assert_eq!(
            manifest.milestone_state.as_deref(),
            Some("bridge_wasm_mir_payload_module_emitted")
        );
        assert_eq!(
            manifest.bridge.milestone_state.as_deref(),
            Some("bridge_wasm_mir_payload_module_emitted")
        );
        assert_eq!(manifest.bridge.blocker_kind, "none");
        let target_pack = manifest.target_pack.as_ref().unwrap();
        assert_eq!(target_pack.target_triple, "wasm32-wasip1");
        assert!(target_pack.attempted);
        assert_eq!(target_pack.status, "ready");
        assert_eq!(target_pack.blocker_kind, "none");
        assert!(target_pack.std_available);
        assert!(target_pack.core_available);
        assert!(target_pack.alloc_available);
        assert!(target_pack
            .produced_artifacts
            .iter()
            .any(|artifact| artifact.ends_with(".rlib")));
        assert_eq!(manifest.bridge.command_exit_code, Some(0));
        assert_eq!(manifest.bridge.target_triple, "wasm32-wasip1");
        assert!(manifest
            .bridge
            .commands_attempted
            .iter()
            .any(|command| command.classification
                == "direct_rustc_private_pack_ready_bridge_mono_items_collected"
                && command.exit_code == 0));
        assert!(manifest
            .bridge
            .commands_attempted
            .iter()
            .any(
                |command| command.classification == "bootstrap_target_pack_missing"
                    && command.exit_code == 1
            ));
        assert!(manifest
            .bridge
            .commands_attempted
            .iter()
            .any(|command| command.classification == "missing_wasi_sdk"
                && command.exit_code == 1
                && command.evidence.contains("WASI_SDK_PATH")));
        assert!(manifest
            .bridge
            .commands_attempted
            .iter()
            .any(
                |command| command.classification == "wasm32_wasip1_target_pack_ready"
                    && command.exit_code == 0
                    && command.evidence.contains("core, alloc, and std")
            ));
        assert!(manifest
            .bridge
            .commands_attempted
            .iter()
            .any(|command| command.classification
                == "rustc_private_target_crates_not_emitted_after_closure_attempt"
                && command.exit_code == 101
                && command.evidence.contains("rustc_middle")));
        assert!(manifest
            .bridge
            .commands_attempted
            .iter()
            .any(
                |command| command.classification == "rustc_private_target_route_host_bound"
                    && command.exit_code == 0
                    && command.command.contains("rustc_mir_build")
            ));
        assert!(manifest
            .bridge
            .commands_attempted
            .iter()
            .any(
                |command| command.classification == "xpy_stage2_host_wasm_requires_ninja"
                    && command.exit_code == 1
                    && command.evidence.contains("ninja")
            ));
        assert!(manifest
            .bridge
            .commands_attempted
            .iter()
            .any(
                |command| command.classification
                    == "stage2_wasm_host_route_blocked_at_llvm_wasm32_wasip1_machine_endian_header_missing"
                    && command.exit_code == 1
                    && command.evidence.contains("machine/endian.h")
            ));
        let output = manifest.bridge.output_artifact_identity.as_ref().unwrap();
        assert_eq!(output.artifact_format, "wasm_module");
        assert_eq!(
            output.sha256,
            "b9ae49950e1f1f12768211d4b5f8fa9f6a8ebb52cacafe2bb701688db59f7c54"
        );
        assert!(output.loadable_by_rouwdi_wasm);

        let required_symbols = manifest.required_symbol_names();
        assert!(required_symbols.contains(&COMPILER_PAYLOAD_ABI_V1_VERSION_SYMBOL));
        assert!(required_symbols.contains(&COMPILER_PAYLOAD_ABI_V1_STAGE_SYMBOL));
        assert!(required_symbols.contains(&COMPILER_PAYLOAD_ABI_V1_DESCRIPTOR_PTR_SYMBOL));
        assert!(required_symbols.contains(&COMPILER_PAYLOAD_ABI_V1_DESCRIPTOR_LEN_SYMBOL));
        assert!(required_symbols.contains(&MIR_HANDOFF_PAYLOAD_ABI_V1_VALID_INPUT_PTR_SYMBOL));
        assert!(required_symbols.contains(&MIR_HANDOFF_PAYLOAD_ABI_V1_VALID_INPUT_LEN_SYMBOL));
        assert!(required_symbols.contains(&MIR_HANDOFF_PAYLOAD_ABI_V1_RESULT_AREA_PTR_SYMBOL));
        assert!(required_symbols.contains(&MIR_HANDOFF_PAYLOAD_ABI_V1_EXECUTE_SYMBOL));
        assert!(required_symbols.contains(&MIR_HANDOFF_PAYLOAD_ABI_V1_LAST_ERROR_PTR_SYMBOL));
        assert!(required_symbols.contains(&MIR_HANDOFF_PAYLOAD_ABI_V1_LAST_ERROR_LEN_SYMBOL));

        let handle = manifest.upstream_context_handle_v1.as_ref().unwrap();
        assert_eq!(handle.selected_strategy, "payload_owned_context");
        assert_eq!(handle.owner, "payload");
        assert_eq!(handle.scope, "payload-local");
        assert!(handle.opaque);
        assert!(!handle.serializable);
        assert!(!handle.may_cross_wasm_instance_boundaries);
        assert!(!handle.raw_pointers_allowed);
        assert!(handle
            .refers_to
            .iter()
            .any(|item| item.contains("rustc_span::SourceMap")));

        let runtime = manifest.wasm_runtime_execution.as_ref().unwrap();
        assert_eq!(runtime.classification, "mono_items_collected");
        assert!(runtime.module_instantiated);
        assert!(runtime.abi_exports_called);
        assert!(runtime.descriptor_bytes_read);
        assert!(runtime.valid_input_bytes_read);
        assert!(runtime.execute_called);
        assert!(runtime.output_or_error_bytes_read);
        assert_eq!(runtime.blocker_kind, "none");
    }

    #[test]
    fn rustc_codegen_llvm_probe_records_real_backend_contact_and_target_blocker() {
        let component = import_component("rustc_codegen_llvm").unwrap();
        let probe = rustc_codegen_llvm_backend_probe();

        assert!(component.attempted);
        assert_eq!(
            component.source_path,
            "third_party/rust/compiler/rustc_codegen_llvm"
        );
        assert_eq!(component.blocker_kind, "none");
        assert!(component
            .probe_command
            .contains("compiler/rustc_codegen_llvm"));
        assert!(component.exact_blocker.contains("embedded"));
        assert!(component.exact_blocker.contains("LLVM IR"));
        assert!(component.exact_blocker.contains("exits 0"));
        assert!(component.exact_blocker.contains("LLVM context/module"));
        assert!(component.exact_blocker.contains("target machine"));
        assert!(component
            .exact_blocker
            .contains("Object/Wasm-object emission remains"));
        assert_eq!(
            component.adapter_symbol.as_deref(),
            Some("rouwdi_rustc_upstream::rustc_codegen_llvm_backend_probe")
        );
        assert_eq!(
            probe.entrypoint,
            "rustc_codegen_llvm::LlvmCodegenBackend::new"
        );
        assert!(probe.backend_constructor_referenced);
        assert!(probe.backend_constructed);
        assert_eq!(probe.backend_name.as_deref(), Some("llvm"));
        assert_eq!(probe.host_probe_exit_code, 0);
        assert!(probe.host_probe_unresolved_symbols.is_empty());
        assert!(probe.llvm_config_path.contains("llvm-config.exe"));
        assert!(probe.llvm_libdir.contains("ci-llvm"));
        assert!(probe.llvm_libs.contains(&"LLVMCore.lib".to_owned()));
        assert!(probe
            .llvm_libs
            .contains(&"LLVMWebAssemblyCodeGen.lib".to_owned()));
        assert!(probe.llvm_system_libs.contains(&"advapi32.lib".to_owned()));
        assert!(probe
            .host_probe_resolved_libraries
            .contains(&"llvm-wrapper.lib".to_owned()));
        assert_eq!(probe.codegen_contact_state, "target_machine_created");
        assert!(probe.mono_proof_consumed);
        assert_eq!(probe.compile_unit_id, "app:rust:app:wasm32-wasip1");
        assert_eq!(probe.mir_body_hash, "a5e137ef6793c0b8");
        assert_eq!(probe.mono_item_count, 1);
        assert_eq!(probe.mono_item_graph_hash, "bec5817d61819666");
        assert!(probe.llvm_context_created);
        assert!(probe.llvm_module_created);
        assert!(probe
            .llvm_module_identity
            .as_deref()
            .is_some_and(|identity| identity.contains("mono=bec5817d61819666")));
        assert!(probe
            .llvm_module_identity_hash
            .as_deref()
            .is_some_and(|hash| hash.len() == 64));
        assert_eq!(
            probe.llvm_module_target_triple.as_deref(),
            Some("wasm32-wasip1")
        );
        assert!(probe.target_machine_setup_invoked);
        assert!(probe.target_machine_created);
        assert_eq!(probe.target_machine_cpu, "generic");
        assert_eq!(probe.target_machine_relocation_model, "pic");
        assert_eq!(probe.target_loadable_probe_exit_code, 0);
        assert_eq!(
            probe.target_loadable_status,
            "rustc_codegen_llvm_target_loadable_check_only"
        );
        assert_eq!(
            probe.target_loadable_check_only_status,
            "rustc_codegen_llvm_target_loadable_check_only"
        );
        assert!(probe.backend_payload_build_attempted);
        assert_eq!(probe.backend_payload_build_exit_code, 0);
        assert!(probe.executable_backend_payload_linked);
        assert_eq!(probe.backend_payload_artifact_sha256.len(), 64);
        assert!(probe.backend_payload_artifact_size_bytes > 60_000_000);
        assert!(probe.embedded_backend_payload_executed);
        assert!(probe.backend_payload_final_link_invoked);
        assert!(probe
            .backend_payload_linker
            .ends_with("wasm32-wasip1-clang.exe"));
        assert!(probe.backend_payload_first_undefined_symbol.is_empty());
        assert!(probe.backend_payload_llvm_undefined_symbols.is_empty());
        assert_eq!(probe.backend_payload_execution_status, "llvm_ir_emitted");
        assert_eq!(probe.backend_payload_blocker_kind, "none");
        assert_eq!(probe.llvm_wrapper_target, "wasm32-wasip1");
        assert_eq!(probe.llvm_wrapper_target_artifact_kind, "staticlib");
        assert!(probe
            .llvm_wrapper_target_path
            .ends_with("libllvm-wrapper.a"));
        assert_eq!(probe.llvm_wrapper_target_sha256.len(), 64);
        assert!(probe.llvm_wrapper_target_size_bytes > 1_000_000);
        assert_eq!(
            probe.llvm_wrapper_target_built_by,
            "bootstrap/rustc-codegen-llvm-probe/build-target-llvm-wrapper.ps1"
        );
        assert!(probe.llvm_wrapper_target_loadable);
        assert_eq!(probe.llvm_wrapper_target_blocker_kind, "none");
        assert!(probe.target_llvm_library_closure_available);
        assert_eq!(probe.target_llvm_library_closure_status, "available");
        assert!(!probe.enzyme_libloading_blocker_present);
        assert_eq!(probe.blocker_kind, "none");
        assert_eq!(probe.blocker_component, "none");
        assert!(probe.blocker_reason.contains("embedded"));
        assert!(probe.blocker_reason.contains("LLVM IR"));
        assert!(probe.blocker_reason.contains("Object emission remains"));
        assert!(probe
            .target_loadable_components
            .contains(&"rustc_codegen_ssa".to_owned()));
        assert!(!probe.object_emission_attempted);
        assert!(!probe.object_bytes_emitted);
        assert!(probe.llvm_ir_emitted);
        assert_eq!(
            probe.llvm_ir_sha256.as_deref(),
            Some("6b151410d83fa3fafc9c88ac4ef889635be7173652e0c6af95e015a515d72267")
        );
        assert_eq!(probe.llvm_ir_byte_len, Some(121));
    }

    #[test]
    fn selected_abi_route_is_a_real_wasm_context_attempt_with_bridge_execution_recorded() {
        let manifest = compiler_payload_abi_manifest();
        let route = manifest.selected_artifact_route().unwrap();

        assert_eq!(route.route, "wasm32_wasip1_module");
        assert_eq!(route.artifact_format, CompilerPayloadAbiFormat::WasmModule);
        assert_eq!(route.target_triple, "wasm32-wasip1");
        assert!(route.attempted);
        assert_eq!(route.status, CompilerPayloadAbiRouteStatus::Emitted);
        assert_eq!(route.bridge_status, "mono_items_collected");
        assert_eq!(route.blocker_kind.as_deref(), Some("none"));
        assert!(route.loadable_as_full_payload);

        let workspace = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .ancestors()
            .nth(2)
            .expect("adapter crate lives under workspace/crates/rouwdi-rustc-upstream");
        let artifact_path = route.artifact_path.as_ref().unwrap();
        let bytes = std::fs::read(workspace.join(artifact_path)).expect(
            "run `cargo run -p rouwdi-rustc-upstream --bin direct-rustc-private-pack-builder` first",
        );

        assert!(bytes.starts_with(b"\0asm"));
        let exports = wasm_export_names(&bytes);
        for symbol in manifest.required_symbol_names() {
            assert!(
                exports.iter().any(|export| export == symbol),
                "missing required ABI export {symbol}"
            );
        }
        assert_eq!(route.artifact_size_bytes, Some(bytes.len() as u64));
        let artifact_hash = sha256_hex(&bytes);
        assert_eq!(
            route.artifact_sha256.as_deref(),
            Some(artifact_hash.as_str())
        );
    }

    #[test]
    fn compiler_payload_wasm_loader_instantiates_and_executes_context_attempt() {
        let manifest = compiler_payload_abi_manifest();
        let route = manifest.selected_artifact_route().unwrap();
        let workspace = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .ancestors()
            .nth(2)
            .expect("adapter crate lives under workspace/crates/rouwdi-rustc-upstream");
        let artifact_path = workspace.join(route.artifact_path.as_ref().unwrap());
        let expected_sha256 = route.artifact_sha256.as_deref().unwrap();
        let bytes = std::fs::read(&artifact_path).expect(
            "run `cargo run -p rouwdi-rustc-upstream --bin direct-rustc-private-pack-builder` first",
        );

        let report = execute_compiler_payload_wasm(
            route.artifact_path.as_ref().unwrap(),
            &bytes,
            expected_sha256,
        )
        .expect("loadable bridge Wasm must instantiate and execute");

        assert!(report.hash_verified);
        assert!(report.module_instantiated);
        assert!(report.abi_v1_exports_verified);
        assert!(report.exports.iter().any(|export| export == "memory"));
        assert!(report.exports.iter().any(|export| export == "_start"));
        assert!(report.version_called);
        assert_eq!(report.version, 1);
        assert!(report.stage_called);
        assert_eq!(report.stage, 1);
        assert!(report.descriptor_bytes_read);
        assert!(report
            .descriptor_json
            .contains("rustc-interface-sysroot-mono-items-collected"));
        assert!(report.valid_input_bytes_read);
        assert!(report.valid_input_json.contains("UpstreamContextHandleV1"));
        assert!(report.valid_input_json.contains("\"raw_pointer\":false"));
        assert!(report.execute_called);
        assert_eq!(report.execute_status, 0);
        assert!(report.output_bytes_read || report.error_bytes_read);
        assert!(report.output_bytes_read);
        assert!(!report.error_bytes_read);
        assert_eq!(report.classification, "mono_items_collected");
        assert_eq!(report.context_handle_strategy, "payload_owned_context");
        assert_eq!(report.context_state, "mono_items_collected");
        assert!(report.generic_upstream_context_unavailable_replaced);
        assert!(!report.fabricated_ast);
        assert!(!report.fabricated_hir);
        assert!(!report.fabricated_tyctx);
        assert!(!report.fabricated_providers);
        assert!(!report.fabricated_body);
        assert!(!report.fabricated_mir);
        assert!(report.output_json.as_deref().is_some_and(|json| json
            .contains("\"source_map_created\":true")
            && json.contains("\"parse_session_created\":true")
            && json.contains("\"parser_invoked\":true")
            && json.contains("\"crate_ast_created\":true")
            && json.contains("\"rustc_interface_config_created\":true")
            && json.contains("\"tyctx_entered\":true")
            && json.contains("\"hir_lowering_attempted\":true")
            && json.contains("\"core_metadata_loaded\":true")
            && json.contains("\"mir_provider_invoked\":true")
            && json.contains("\"real_mir_body_observed\":true")
            && json.contains("\"mir_body_identity\"")
            && json.contains("\"mir_body_hash\"")));
    }

    #[test]
    fn compiler_payload_bundle_records_loader_boundary_and_export_routes() {
        let bundle = mir_compiler_payload_bundle();

        assert_eq!(bundle.bundle_format_version, 1);
        assert_eq!(
            bundle.payload_manifest.path,
            MIR_PAYLOAD_EXPORT_MANIFEST_PATH
        );
        assert_eq!(bundle.payload_manifest.sha256.len(), 64);
        assert_eq!(bundle.exported_rlib_identity.artifact_format, "wasm_module");
        assert_eq!(bundle.metadata_artifact_identity.artifact_format, "rmeta");
        assert_eq!(bundle.stage, 1);
        assert_eq!(bundle.host_triple, "x86_64-pc-windows-msvc");
        assert_eq!(bundle.target_triple, "wasm32-wasip1");
        assert!(bundle
            .upstream_type_surface
            .contains(&"rustc_middle::mir::Body<'tcx>".to_owned()));
        assert!(bundle
            .provider_surface
            .contains(&"rustc_mir_build::provide(&mut rustc_middle::util::Providers)".to_owned()));
        assert!(bundle
            .adapter_entrypoints
            .iter()
            .any(|entrypoint| entrypoint.contains("mir_handoff_payload_adapter")));
        assert_eq!(
            bundle.loadability_status,
            CompilerPayloadLoadabilityStatus::Loadable
        );
        assert_eq!(bundle.next_required_artifact_format, "codegen_handoff");
        assert_eq!(
            bundle.compiler_payload_abi_manifest.as_ref().unwrap().path,
            COMPILER_PAYLOAD_ABI_MANIFEST_PATH
        );
        assert_eq!(
            bundle
                .compiler_payload_abi
                .as_ref()
                .unwrap()
                .supported_stage,
            CompilerPayloadSupportedStage::MirHandoff
        );
        assert_eq!(
            bundle.selected_abi_route.as_ref().unwrap().status,
            CompilerPayloadAbiRouteStatus::Emitted
        );
        let bridge = bundle.bridge_attempt.as_ref().unwrap();
        assert_eq!(bridge.status, "mono_items_collected");
        assert_eq!(bridge.blocker_kind, "none");
        let target_pack = bundle.target_pack.as_ref().unwrap();
        assert!(target_pack.attempted);
        assert_eq!(target_pack.blocker_kind, "none");
        assert!(target_pack.std_available);
        assert!(target_pack.core_available);
        assert!(target_pack.alloc_available);
        assert_eq!(
            bundle.milestone_state.as_deref(),
            Some("bridge_wasm_mir_payload_module_emitted")
        );
        assert!(bridge.output_artifact_identity.is_some());
        assert!(bundle.loadable_export_routes.iter().any(|route| {
            route.route == "explicit_rouwdi_compiler_payload_bundle"
                && route.attempted
                && route.status == CompilerPayloadExportRouteStatus::Emitted
                && route.blocker_kind.as_deref() == Some("none")
        }));
    }

    #[test]
    fn compiler_payload_loader_classifies_supported_artifact_families_without_executing_them() {
        fn artifact(format: &str, path: &str) -> BootstrapMirAdapterArtifactRecord {
            BootstrapMirAdapterArtifactRecord {
                crate_name: "payload".to_owned(),
                artifact_kind: format.to_owned(),
                artifact_format: format.to_owned(),
                path: path.to_owned(),
                sha256: "0".repeat(64),
                size_bytes: 0,
                host_triple: "x86_64-pc-windows-msvc".to_owned(),
                profile: "release".to_owned(),
                emitted_by: "classification-only".to_owned(),
                loadable_by_rouwdi_wasm: false,
            }
        }

        let cases = [
            (
                artifact("rmeta", "payload.rmeta"),
                CompilerPayloadArtifactClass::MetadataOnly,
            ),
            (
                artifact("rlib", "libpayload.rlib"),
                CompilerPayloadArtifactClass::RlibArchive,
            ),
            (
                artifact("cdylib", "payload.dll"),
                CompilerPayloadArtifactClass::NativeDynamicPayload,
            ),
            (
                artifact("staticlib", "payload.lib"),
                CompilerPayloadArtifactClass::StaticPayload,
            ),
            (
                artifact("wasm_module", "payload.wasm"),
                CompilerPayloadArtifactClass::WasmModule,
            ),
            (
                artifact("wasm_component", "payload.component.wasm"),
                CompilerPayloadArtifactClass::WasmComponent,
            ),
            (
                artifact("rustc_private_blob", "payload.bin"),
                CompilerPayloadArtifactClass::UnsupportedCompilerPrivateArtifact,
            ),
        ];

        for (identity, expected) in cases {
            assert_eq!(
                classify_compiler_payload_artifact(&identity, None),
                expected
            );
        }
    }

    #[test]
    fn direct_pack_classifier_accepts_only_real_wasm_target_artifacts() {
        let wasm_archive = ar_with_member(b"\0asm\x01\0\0\0");
        let host_archive = ar_with_member(b"MZhost-object-not-wasm");

        let wasm_identity = classify_direct_rustc_private_artifact_bytes(
            ".rouwdi/direct-rustc-private-pack/target/wasm32-wasip1/release/deps/libroot.rlib",
            &wasm_archive,
            "wasm32-wasip1",
            "x86_64-pc-windows-msvc",
        );
        assert_eq!(wasm_identity.classification, "target_wasm_rlib");
        assert_eq!(wasm_identity.emitted_target_triple, "wasm32-wasip1");
        assert!(wasm_identity.target_loadable);

        let host_identity = classify_direct_rustc_private_artifact_bytes(
            "third_party/rust/build/x86_64-pc-windows-msvc/stage1-rustc/release/libroot.rlib",
            &host_archive,
            "wasm32-wasip1",
            "x86_64-pc-windows-msvc",
        );
        assert_eq!(
            host_identity.classification,
            "host_rlib_not_target_loadable"
        );
        assert_eq!(
            host_identity.emitted_target_triple,
            "x86_64-pc-windows-msvc"
        );
        assert!(!host_identity.target_loadable);

        let metadata_identity = classify_direct_rustc_private_artifact_bytes(
            ".rouwdi/direct-rustc-private-pack/target/wasm32-wasip1/release/deps/libroot.rmeta",
            b"rustc metadata is not loadable code",
            "wasm32-wasip1",
            "x86_64-pc-windows-msvc",
        );
        assert_eq!(
            metadata_identity.classification,
            "metadata_only_not_target_loadable"
        );
        assert_eq!(metadata_identity.emitted_target_triple, "wasm32-wasip1");
        assert!(!metadata_identity.target_loadable);
    }

    #[test]
    fn compiler_payload_loader_hash_verifies_current_wasm_and_records_context_attempt_boundary() {
        let bundle = mir_compiler_payload_bundle();
        let workspace = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .ancestors()
            .nth(2)
            .expect("adapter crate lives under workspace/crates/rouwdi-rustc-upstream");
        let wasm_bytes = std::fs::read(workspace.join(&bundle.exported_rlib_identity.path)).expect(
            "run `cargo run -p rouwdi-rustc-upstream --bin direct-rustc-private-pack-builder` first",
        );
        let rmeta_bytes = std::fs::read(workspace.join(&bundle.metadata_artifact_identity.path))
            .expect(
                "run `python x.py check src/tools/rouwdi-mir-adapter-probe --stage 1 -v` first",
            );

        let inspection =
            inspect_compiler_payload_bundle(&bundle, Some(&wasm_bytes), Some(&rmeta_bytes));

        assert!(inspection.payload_bundle_inspected);
        assert_eq!(
            inspection.exported_payload.artifact_class,
            CompilerPayloadArtifactClass::WasmModule
        );
        assert_eq!(
            inspection.metadata_artifact.artifact_class,
            CompilerPayloadArtifactClass::MetadataOnly
        );
        assert_eq!(
            inspection.exported_payload.hash_status,
            CompilerPayloadHashStatus::Verified
        );
        assert_eq!(
            inspection.metadata_artifact.hash_status,
            CompilerPayloadHashStatus::Verified
        );
        assert_eq!(
            inspection.load_strategy,
            CompilerPayloadLoadStrategy::InstantiateWasmModule
        );
        assert_eq!(
            inspection.loadability_status,
            CompilerPayloadLoadabilityStatus::Loadable
        );
        assert!(inspection.loadable_by_rouwdi_wasm);
        assert_eq!(
            inspection.abi_manifest.as_ref().unwrap().path,
            COMPILER_PAYLOAD_ABI_MANIFEST_PATH
        );
        assert_eq!(
            inspection.abi_selected_route.as_deref(),
            Some("wasm32_wasip1_module")
        );
        assert_eq!(
            inspection.abi_route_status,
            Some(CompilerPayloadAbiRouteStatus::Emitted)
        );
        assert_eq!(inspection.abi_bridge_blocker_kind.as_deref(), Some("none"));
        assert_eq!(
            inspection.milestone_state.as_deref(),
            Some("bridge_wasm_mir_payload_module_emitted")
        );
        let target_pack = inspection.target_pack.as_ref().unwrap();
        assert!(target_pack.attempted);
        assert_eq!(target_pack.status, "ready");
        assert_eq!(target_pack.blocker_kind, "none");
        assert!(target_pack.exact_blocker.contains("exited 0"));
        let bridge = inspection.bridge_attempt.as_ref().unwrap();
        assert_eq!(bridge.status, "mono_items_collected");
        assert_eq!(bridge.command_exit_code, Some(0));
        assert!(bridge.exact_blocker.contains("mono_items_collected"));
        assert!(inspection
            .exact_loader_blocker
            .contains("does not fabricate"));
    }

    #[test]
    fn mir_payload_boundary_rejects_fake_mir_and_fake_wasm_payload_claims() {
        let manifest = compiler_payload_abi_manifest();
        let bundle = mir_compiler_payload_bundle();
        let inspection = inspect_compiler_payload_bundle(&bundle, None, None);

        assert!(manifest
            .input_contract
            .notes
            .contains("does not serialize or fabricate TyCtxt"));
        assert!(manifest
            .output_contract
            .notes
            .contains("must not emit this output until real upstream rustc MIR"));
        assert_eq!(
            manifest
                .bridge
                .output_artifact_identity
                .as_ref()
                .unwrap()
                .artifact_format,
            "wasm_module"
        );
        assert!(manifest.artifact_routes.iter().any(|route| {
            route.route == "wasm32_wasip1_module" && route.loadable_as_full_payload
        }));
        assert_eq!(
            inspection.loadability_status,
            CompilerPayloadLoadabilityStatus::Loadable
        );
        assert!(inspection.loadable_by_rouwdi_wasm);
        assert!(inspection
            .exact_loader_blocker
            .contains("does not fabricate"));
        assert!(manifest
            .bridge
            .exact_blocker
            .contains("mono_items_collected"));
        assert!(manifest
            .bridge
            .exact_blocker
            .contains("No fabricated HIR, TyCtxt, Providers"));
    }

    #[test]
    fn mir_handoff_boundary_names_the_current_payload_adapter_blocker() {
        let boundary = mir_handoff_adapter_boundary();

        assert_eq!(boundary.adapter_symbol, MIR_HANDOFF_PAYLOAD_ADAPTER_SYMBOL);
        assert_eq!(
            boundary.milestone_state.as_deref(),
            Some("bridge_wasm_mir_payload_module_emitted")
        );
        assert_eq!(
            boundary.payload_adapter_status,
            MirHandoffPayloadAdapterStatus::PayloadContextAttempted
        );
        assert!(!boundary.payload_adapter_available);
        assert_eq!(
            boundary.payload_carrier_state,
            Some(MirHandoffPayloadCarrierState::PayloadContextAttempted)
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
            Some("payload_context_attempted")
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
            .is_some_and(|reason| reason.contains("TyCtxt::optimized_mir")
                && reason.contains("collect_and_partition_mono_items")));
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
                && component.blocker_kind == "none"
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
