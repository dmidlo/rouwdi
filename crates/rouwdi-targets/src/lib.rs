use rouwdi_contract::{ArtifactKind, RouwdiContract};
use rouwdi_rustc::{
    complete_rustc_semantics_embedded, rustc_component_inventory, RustcComponentStatus,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;

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
    pub std_pack_hash: String,
    pub linker_pack_hash: String,
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
        packs.insert(
            "wasm32-wasip1".to_owned(),
            target_pack(
                "wasm32-wasip1",
                wasm32_wasip1_target_specs(),
                vec![ArtifactKind::Module, ArtifactKind::Component],
                RuntimeExecutionCapability::Wasi,
            ),
        );
        packs.insert(
            "wasm32-wasip2".to_owned(),
            target_pack(
                "wasm32-wasip2",
                wasm32_wasip2_target_specs(),
                vec![ArtifactKind::Module, ArtifactKind::Component],
                RuntimeExecutionCapability::Wasi,
            ),
        );
        packs.insert(
            "native_host".to_owned(),
            target_pack(
                "native_host",
                native_host_family_target_specs(),
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
    artifact_kinds: Vec<ArtifactKind>,
    runtime_execution: RuntimeExecutionCapability,
) -> TargetPack {
    TargetPack {
        triple: triple.to_owned(),
        target_pack_hash: "embedded-target-pack-pending".to_owned(),
        target_spec_bundle_hash: embedded_source_bundle_hash(&target_spec_sources),
        target_spec_sources,
        std_pack_hash: "embedded-std-pack-pending".to_owned(),
        linker_pack_hash: "embedded-linker-pack-pending".to_owned(),
        target_pack_embedded: false,
        std_pack_embedded: false,
        linker_pack_embedded: false,
        artifact_kinds,
        runtime_execution,
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
        assert!(!registry.packs["wasm32-wasip1"].target_pack_embedded);
        assert!(!registry.packs["wasm32-wasip1"].std_pack_embedded);
        assert!(!registry.packs["wasm32-wasip1"].linker_pack_embedded);
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
                && !component.embedded_in_assembly));
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
