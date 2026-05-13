use rouwdi_contract::{ArtifactKind, RouwdiContract};
use serde::{Deserialize, Serialize};
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
    pub std_pack_hash: String,
    pub linker_pack_hash: String,
    pub artifact_kinds: Vec<ArtifactKind>,
    pub runtime_execution: RuntimeExecutionCapability,
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
    pub compiler_semantics_embedded: bool,
    pub codegen_embedded: bool,
    pub linker_embedded: bool,
}

impl CompilerEngineIdentity {
    pub fn incomplete_bootstrap_guard() -> Self {
        Self {
            engine_id: "rouwdi-bootstrap-guard".to_owned(),
            rust_release: "not-embedded".to_owned(),
            compiler_semantics_embedded: false,
            codegen_embedded: false,
            linker_embedded: false,
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
            TargetPack {
                triple: "wasm32-wasip1".to_owned(),
                target_pack_hash: "embedded-target-pack-pending".to_owned(),
                std_pack_hash: "embedded-std-pack-pending".to_owned(),
                linker_pack_hash: "embedded-linker-pack-pending".to_owned(),
                artifact_kinds: vec![ArtifactKind::Module, ArtifactKind::Component],
                runtime_execution: RuntimeExecutionCapability::Wasi,
            },
        );
        packs.insert(
            "wasm32-wasip2".to_owned(),
            TargetPack {
                triple: "wasm32-wasip2".to_owned(),
                target_pack_hash: "embedded-target-pack-pending".to_owned(),
                std_pack_hash: "embedded-std-pack-pending".to_owned(),
                linker_pack_hash: "embedded-linker-pack-pending".to_owned(),
                artifact_kinds: vec![ArtifactKind::Module, ArtifactKind::Component],
                runtime_execution: RuntimeExecutionCapability::Wasi,
            },
        );
        packs.insert(
            "native_host".to_owned(),
            TargetPack {
                triple: "native_host".to_owned(),
                target_pack_hash: "embedded-target-pack-pending".to_owned(),
                std_pack_hash: "embedded-std-pack-pending".to_owned(),
                linker_pack_hash: "embedded-linker-pack-pending".to_owned(),
                artifact_kinds: vec![
                    ArtifactKind::Executable,
                    ArtifactKind::Staticlib,
                    ArtifactKind::Archive,
                    ArtifactKind::Object,
                ],
                runtime_execution: RuntimeExecutionCapability::DelegatedOnly,
            },
        );
        Self {
            compiler: CompilerEngineIdentity::incomplete_bootstrap_guard(),
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
