use rouwdi_contract::{ArtifactKind, RouwdiContract};
use rouwdi_rustc::{
    complete_rustc_semantics_embedded, rustc_component_inventory, RustcComponentStatus,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;

const WASM32_WASIP1_LINKER_PAYLOAD_SHA256: &str =
    "b04d1efd1d7a2f39f774f641e4a2e9e98350816aa19a36de57c46d59f0026dcd";

#[derive(Debug, thiserror::Error)]
pub enum TargetError {
    #[error("target pack is not embedded for {0}")]
    MissingTargetPack(String),
    #[error("target {triple} does not support requested artifact kind {artifact:?}")]
    UnsupportedArtifact {
        triple: String,
        artifact: ArtifactKind,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TargetPack {
    pub triple: String,
    pub target_pack_hash: String,
    pub target_spec_bundle_hash: String,
    pub target_spec_sources: Vec<EmbeddedSourceFile>,
    pub target_abi: TargetAbiModel,
    pub std_pack_hash: Option<String>,
    pub linker_pack_hash: Option<String>,
    pub target_pack_embedded: bool,
    pub std_pack_embedded: bool,
    pub linker_pack_embedded: bool,
    pub artifact_kinds: Vec<ArtifactKind>,
    pub runtime_execution: RuntimeExecutionCapability,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EmbeddedSourceFile {
    pub path: String,
    pub sha256: String,
    pub len: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TargetAbiModel {
    pub family: String,
    pub pointer_widths: Vec<u16>,
    pub object_formats: Vec<String>,
    pub operating_systems: Vec<String>,
    pub environments: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RuntimeExecutionCapability {
    Wasi,
    NativeHost,
    DelegatedOnly,
    Unsupported,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompilerEngineIdentity {
    pub engine_id: String,
    pub rust_release: String,
    pub source_custody: Vec<SourceCustodyEntry>,
    pub rustc_components: Vec<RustcComponentStatus>,
    pub compiler_semantics_embedded: bool,
    pub codegen_embedded: bool,
    pub linker_embedded: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceCustodyEntry {
    pub role: String,
    pub repository_url: String,
    pub path: String,
    pub commit: String,
    pub embedded_in_assembly: bool,
}

impl CompilerEngineIdentity {
    pub fn from_embedded_component_inventory() -> Self {
        let rustc_components = rustc_component_inventory();
        let codegen_embedded = rustc_components.iter().any(|component| {
            component.name == "rustc_codegen_llvm" && component.embedded_in_assembly
        });
        let linker_embedded = rustc_components
            .iter()
            .any(|component| component.name == "lld" && component.embedded_in_assembly);
        Self {
            engine_id: "rouwdi-upstream-rustc-custody".to_owned(),
            rust_release: "1.95.0-src-800892799d7666fe1dc17abd862100a6cf273718".to_owned(),
            source_custody: vec![
                SourceCustodyEntry {
                    role: "rust-cargo-llvm-source-custody".to_owned(),
                    repository_url: "https://github.com/rust-lang/rust".to_owned(),
                    path: "third_party/rust".to_owned(),
                    commit: "800892799d7666fe1dc17abd862100a6cf273718".to_owned(),
                    embedded_in_assembly: false,
                },
                SourceCustodyEntry {
                    role: "cargo-source-custody".to_owned(),
                    repository_url: "https://github.com/rust-lang/cargo.git".to_owned(),
                    path: "third_party/rust/src/tools/cargo".to_owned(),
                    commit: "a343accce8526b128adc517d33348573d22920a3".to_owned(),
                    embedded_in_assembly: false,
                },
                SourceCustodyEntry {
                    role: "llvm-source-custody".to_owned(),
                    repository_url: "https://github.com/rust-lang/llvm-project.git".to_owned(),
                    path: "third_party/rust/src/llvm-project".to_owned(),
                    commit: "eaab4d9841b9a8a12783d927b2df2291c1c79269".to_owned(),
                    embedded_in_assembly: false,
                },
            ],
            rustc_components,
            compiler_semantics_embedded: complete_rustc_semantics_embedded(),
            codegen_embedded,
            linker_embedded,
        }
    }

    pub fn is_complete(&self) -> bool {
        self.compiler_semantics_embedded && self.codegen_embedded && self.linker_embedded
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TargetPackRegistry {
    pub compiler: CompilerEngineIdentity,
    pub packs: BTreeMap<String, TargetPack>,
}

impl TargetPackRegistry {
    pub fn strict_embedded() -> Self {
        let mut packs = BTreeMap::new();
        let mut wasm32_wasip1 = target_pack(
            "wasm32-wasip1",
            wasm32_wasip1_target_specs(),
            TargetAbiModel::wasm32_wasi("p1"),
            vec![ArtifactKind::Module, ArtifactKind::Component],
            RuntimeExecutionCapability::Wasi,
        );
        wasm32_wasip1.linker_pack_hash = Some(WASM32_WASIP1_LINKER_PAYLOAD_SHA256.to_owned());
        wasm32_wasip1.linker_pack_embedded = true;
        packs.insert("wasm32-wasip1".to_owned(), wasm32_wasip1);
        packs.insert(
            "wasm32-wasip2".to_owned(),
            target_pack(
                "wasm32-wasip2",
                wasm32_wasip2_target_specs(),
                TargetAbiModel::wasm32_wasi("p2"),
                vec![ArtifactKind::Module, ArtifactKind::Component],
                RuntimeExecutionCapability::Wasi,
            ),
        );
        packs.insert(
            "native_host".to_owned(),
            target_pack(
                "native_host",
                native_host_family_target_specs(),
                TargetAbiModel::native_host_family(),
                vec![
                    ArtifactKind::Executable,
                    ArtifactKind::Staticlib,
                    ArtifactKind::Archive,
                    ArtifactKind::Object,
                ],
                RuntimeExecutionCapability::DelegatedOnly,
            ),
        );
        Self {
            compiler: CompilerEngineIdentity::from_embedded_component_inventory(),
            packs,
        }
    }

    pub fn validate_contract(
        &self,
        contract: &RouwdiContract,
    ) -> Result<Vec<TargetPack>, TargetError> {
        let mut packs = Vec::new();
        for target in &contract.targets {
            let pack = self
                .packs
                .get(&target.triple)
                .ok_or_else(|| TargetError::MissingTargetPack(target.triple.clone()))?;
            if !pack.artifact_kinds.contains(&target.artifact) {
                return Err(TargetError::UnsupportedArtifact {
                    triple: target.triple.clone(),
                    artifact: target.artifact,
                });
            }
            packs.push(pack.clone());
        }
        Ok(packs)
    }
}

fn target_pack(
    triple: &str,
    target_spec_sources: Vec<EmbeddedSourceFile>,
    target_abi: TargetAbiModel,
    artifact_kinds: Vec<ArtifactKind>,
    runtime_execution: RuntimeExecutionCapability,
) -> TargetPack {
    let target_spec_bundle_hash = embedded_source_bundle_hash(&target_spec_sources);
    let target_pack_hash = target_pack_hash(
        triple,
        &target_spec_bundle_hash,
        &target_abi,
        &artifact_kinds,
        runtime_execution,
    );
    TargetPack {
        triple: triple.to_owned(),
        target_pack_hash,
        target_spec_bundle_hash,
        target_spec_sources,
        target_abi,
        std_pack_hash: None,
        linker_pack_hash: None,
        target_pack_embedded: true,
        std_pack_embedded: false,
        linker_pack_embedded: false,
        artifact_kinds,
        runtime_execution,
    }
}

impl TargetAbiModel {
    fn wasm32_wasi(variant: &str) -> Self {
        Self {
            family: "wasm32-wasi".to_owned(),
            pointer_widths: vec![32],
            object_formats: vec!["wasm".to_owned()],
            operating_systems: vec!["wasi".to_owned()],
            environments: vec![variant.to_owned()],
        }
    }

    fn native_host_family() -> Self {
        Self {
            family: "native-host-family".to_owned(),
            pointer_widths: vec![64],
            object_formats: vec!["elf".to_owned(), "mach-o".to_owned(), "pe-coff".to_owned()],
            operating_systems: vec!["linux".to_owned(), "macos".to_owned(), "windows".to_owned()],
            environments: vec!["gnu".to_owned(), "msvc".to_owned(), "darwin".to_owned()],
        }
    }
}

fn wasm32_wasip1_target_specs() -> Vec<EmbeddedSourceFile> {
    vec![
        embedded_source(
            "third_party/rust/compiler/rustc_target/src/spec/targets/wasm32_wasip1.rs",
            include_bytes!(
                "../../../third_party/rust/compiler/rustc_target/src/spec/targets/wasm32_wasip1.rs"
            ),
        ),
        embedded_source(
            "third_party/rust/compiler/rustc_target/src/spec/base/wasm.rs",
            include_bytes!("../../../third_party/rust/compiler/rustc_target/src/spec/base/wasm.rs"),
        ),
    ]
}

fn wasm32_wasip2_target_specs() -> Vec<EmbeddedSourceFile> {
    vec![
        embedded_source(
            "third_party/rust/compiler/rustc_target/src/spec/targets/wasm32_wasip2.rs",
            include_bytes!(
                "../../../third_party/rust/compiler/rustc_target/src/spec/targets/wasm32_wasip2.rs"
            ),
        ),
        embedded_source(
            "third_party/rust/compiler/rustc_target/src/spec/base/wasm.rs",
            include_bytes!("../../../third_party/rust/compiler/rustc_target/src/spec/base/wasm.rs"),
        ),
    ]
}

fn native_host_family_target_specs() -> Vec<EmbeddedSourceFile> {
    vec![
        embedded_source(
            "third_party/rust/compiler/rustc_target/src/spec/targets/x86_64_unknown_linux_gnu.rs",
            include_bytes!(
                "../../../third_party/rust/compiler/rustc_target/src/spec/targets/x86_64_unknown_linux_gnu.rs"
            ),
        ),
        embedded_source(
            "third_party/rust/compiler/rustc_target/src/spec/targets/aarch64_unknown_linux_gnu.rs",
            include_bytes!(
                "../../../third_party/rust/compiler/rustc_target/src/spec/targets/aarch64_unknown_linux_gnu.rs"
            ),
        ),
        embedded_source(
            "third_party/rust/compiler/rustc_target/src/spec/targets/x86_64_apple_darwin.rs",
            include_bytes!(
                "../../../third_party/rust/compiler/rustc_target/src/spec/targets/x86_64_apple_darwin.rs"
            ),
        ),
        embedded_source(
            "third_party/rust/compiler/rustc_target/src/spec/targets/aarch64_apple_darwin.rs",
            include_bytes!(
                "../../../third_party/rust/compiler/rustc_target/src/spec/targets/aarch64_apple_darwin.rs"
            ),
        ),
        embedded_source(
            "third_party/rust/compiler/rustc_target/src/spec/targets/x86_64_pc_windows_msvc.rs",
            include_bytes!(
                "../../../third_party/rust/compiler/rustc_target/src/spec/targets/x86_64_pc_windows_msvc.rs"
            ),
        ),
    ]
}

fn embedded_source(path: &str, bytes: &[u8]) -> EmbeddedSourceFile {
    EmbeddedSourceFile {
        path: path.to_owned(),
        sha256: hash_bytes(bytes),
        len: bytes.len() as u64,
    }
}

fn embedded_source_bundle_hash(files: &[EmbeddedSourceFile]) -> String {
    let mut digest = Sha256::new();
    for file in files {
        digest.update(file.path.as_bytes());
        digest.update(b"\0");
        digest.update(file.sha256.as_bytes());
        digest.update(b"\0");
        digest.update(file.len.to_le_bytes());
    }
    hex::encode(digest.finalize())
}

fn target_pack_hash(
    triple: &str,
    target_spec_bundle_hash: &str,
    target_abi: &TargetAbiModel,
    artifact_kinds: &[ArtifactKind],
    runtime_execution: RuntimeExecutionCapability,
) -> String {
    let mut digest = Sha256::new();
    digest.update(triple.as_bytes());
    digest.update(b"\0");
    digest.update(target_spec_bundle_hash.as_bytes());
    digest.update(b"\0");
    digest.update(target_abi.family.as_bytes());
    digest.update(b"\0");
    for pointer_width in &target_abi.pointer_widths {
        digest.update(pointer_width.to_le_bytes());
        digest.update(b"\0");
    }
    for object_format in &target_abi.object_formats {
        digest.update(object_format.as_bytes());
        digest.update(b"\0");
    }
    for os in &target_abi.operating_systems {
        digest.update(os.as_bytes());
        digest.update(b"\0");
    }
    for env in &target_abi.environments {
        digest.update(env.as_bytes());
        digest.update(b"\0");
    }
    for artifact_kind in artifact_kinds {
        digest.update(format!("{artifact_kind:?}").as_bytes());
        digest.update(b"\0");
    }
    digest.update(format!("{runtime_execution:?}").as_bytes());
    hex::encode(digest.finalize())
}

fn hash_bytes(bytes: &[u8]) -> String {
    let mut digest = Sha256::new();
    digest.update(bytes);
    hex::encode(digest.finalize())
}

impl Default for TargetPackRegistry {
    fn default() -> Self {
        Self::strict_embedded()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rouwdi_contract::RouwdiContract;

    #[test]
    fn registry_knows_required_initial_target_names_but_marks_compiler_incomplete() {
        let registry = TargetPackRegistry::strict_embedded();

        assert!(registry.packs.contains_key("wasm32-wasip1"));
        assert!(registry.packs.contains_key("wasm32-wasip2"));
        assert!(registry.packs.contains_key("native_host"));
        assert_eq!(registry.compiler.engine_id, "rouwdi-upstream-rustc-custody");
        assert_eq!(
            registry.compiler.source_custody[0].commit,
            "800892799d7666fe1dc17abd862100a6cf273718"
        );
        assert_eq!(
            registry.compiler.source_custody[1].commit,
            "a343accce8526b128adc517d33348573d22920a3"
        );
        assert_eq!(
            registry.compiler.source_custody[2].commit,
            "eaab4d9841b9a8a12783d927b2df2291c1c79269"
        );
        assert!(registry.packs["wasm32-wasip1"].target_pack_embedded);
        assert!(!registry.packs["wasm32-wasip1"].std_pack_embedded);
        assert!(registry.packs["wasm32-wasip1"].linker_pack_embedded);
        assert_eq!(registry.packs["wasm32-wasip1"].target_pack_hash.len(), 64);
        assert_eq!(
            registry.packs["wasm32-wasip1"].target_abi.object_formats,
            vec!["wasm".to_owned()]
        );
        assert!(registry.packs["wasm32-wasip1"].std_pack_hash.is_none());
        assert_eq!(
            registry.packs["wasm32-wasip1"].linker_pack_hash.as_deref(),
            Some(WASM32_WASIP1_LINKER_PAYLOAD_SHA256)
        );
        assert_eq!(
            registry.packs["wasm32-wasip1"]
                .target_spec_bundle_hash
                .len(),
            64
        );
        assert!(registry.packs["wasm32-wasip1"]
            .target_spec_sources
            .iter()
            .any(|source| source.path.ends_with("wasm32_wasip1.rs")));
        assert!(registry
            .compiler
            .rustc_components
            .iter()
            .any(|component| component.name == "rustc_lexer" && component.embedded_in_assembly));
        assert!(registry
            .compiler
            .rustc_components
            .iter()
            .any(|component| component.name == "rustc_codegen_llvm"
                && component.embedded_in_assembly));
        assert!(registry.compiler.linker_embedded);
        assert!(registry
            .compiler
            .rustc_components
            .iter()
            .any(|component| component.name == "lld" && component.embedded_in_assembly));
        assert!(registry
            .compiler
            .rustc_components
            .iter()
            .any(|component| component.name == "rustc_expand" && !component.embedded_in_assembly));
        assert!(!registry.compiler.is_complete());
    }

    #[test]
    fn validates_requested_artifact_against_target_pack() {
        let contract = RouwdiContract::parse(
            r#"
contract_version = 1
[project]
manifest_path = "Cargo.toml"
package = "app"
bin = "app"
[source]
mode = "snapshot"
[[targets]]
name = "wasi"
triple = "wasm32-wasip1"
artifact = "module"
"#,
        )
        .unwrap();
        let packs = TargetPackRegistry::strict_embedded()
            .validate_contract(&contract)
            .unwrap();

        assert_eq!(packs[0].triple, "wasm32-wasip1");
    }
}
