use rouwdi_vfs::normalize_path;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;

#[derive(Debug, thiserror::Error)]
pub enum ContractError {
    #[error("contract TOML parse failure: {0}")]
    Toml(#[from] toml::de::Error),
    #[error("contract JSON normalization failure: {0}")]
    Json(#[from] serde_json::Error),
    #[error("unsupported contract_version {0}; expected 1")]
    UnsupportedVersion(u32),
    #[error("missing required field: {0}")]
    Missing(&'static str),
    #[error("invalid contract value: {0}")]
    Invalid(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RouwdiContract {
    pub contract_version: u32,
    pub project: ProjectContract,
    pub source: SourceContract,
    #[serde(default)]
    pub resolver: ResolverContract,
    #[serde(default)]
    pub toolchain: ToolchainContract,
    #[serde(default)]
    pub targets: Vec<TargetContract>,
    #[serde(default)]
    pub proof: ProofContract,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectContract {
    pub manifest_path: String,
    pub package: String,
    #[serde(default)]
    pub bin: Option<String>,
    #[serde(default)]
    pub example: Option<String>,
    #[serde(default = "default_profile")]
    pub profile: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourceContract {
    pub mode: SourceMode,
    #[serde(default = "default_source_root")]
    pub root: String,
    #[serde(default)]
    pub ref_name: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceMode {
    Git,
    Snapshot,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResolverContract {
    #[serde(default = "default_lockfile")]
    pub lockfile: String,
    #[serde(default)]
    pub offline: bool,
    #[serde(default = "default_true")]
    pub frozen: bool,
    #[serde(default)]
    pub vendor: Option<String>,
}

impl Default for ResolverContract {
    fn default() -> Self {
        Self {
            lockfile: default_lockfile(),
            offline: false,
            frozen: true,
            vendor: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ToolchainContract {
    #[serde(default = "default_channel")]
    pub channel: String,
    #[serde(default = "default_edition_floor")]
    pub edition_floor: String,
    #[serde(default = "default_true")]
    pub std: bool,
}

impl Default for ToolchainContract {
    fn default() -> Self {
        Self {
            channel: default_channel(),
            edition_floor: default_edition_floor(),
            std: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TargetContract {
    pub name: String,
    pub triple: String,
    pub artifact: ArtifactKind,
    #[serde(default)]
    pub interface: InterfaceContract,
    #[serde(default)]
    pub runtime: RuntimeContract,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ArtifactKind {
    Module,
    Component,
    Executable,
    Staticlib,
    Archive,
    Object,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InterfaceContract {
    #[serde(default)]
    pub required_exports: Vec<String>,
    #[serde(default)]
    pub require_executable: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RuntimeContract {
    #[serde(default)]
    pub required: bool,
    #[serde(default)]
    pub kind: Option<RuntimeKind>,
    #[serde(default)]
    pub mode: RuntimeMode,
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default = "default_timeout_seconds")]
    pub timeout_seconds: u64,
    #[serde(default)]
    pub expected_exit_code: Option<i32>,
    #[serde(default)]
    pub stdout_contains: Option<String>,
}

impl Default for RuntimeContract {
    fn default() -> Self {
        Self {
            required: false,
            kind: None,
            mode: RuntimeMode::Local,
            args: Vec::new(),
            timeout_seconds: default_timeout_seconds(),
            expected_exit_code: None,
            stdout_contains: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RuntimeKind {
    Wasi,
    Native,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RuntimeMode {
    #[default]
    Local,
    Delegated,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProofContract {
    #[serde(default = "default_true")]
    pub emit_hashes: bool,
    #[serde(default = "default_true")]
    pub emit_build_graph: bool,
    #[serde(default = "default_true")]
    pub emit_source_snapshot: bool,
    #[serde(default = "default_true")]
    pub emit_runtime_transcripts: bool,
}

impl Default for ProofContract {
    fn default() -> Self {
        Self {
            emit_hashes: true,
            emit_build_graph: true,
            emit_source_snapshot: true,
            emit_runtime_transcripts: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NormalizedContract {
    pub contract: RouwdiContract,
    pub canonical_json: String,
    pub sha256: String,
}

impl RouwdiContract {
    pub fn parse(input: &str) -> Result<Self, ContractError> {
        let contract: Self = toml::from_str(input)?;
        contract.validate()?;
        Ok(contract)
    }

    pub fn validate(&self) -> Result<(), ContractError> {
        if self.contract_version != 1 {
            return Err(ContractError::UnsupportedVersion(self.contract_version));
        }
        validate_path("project.manifest_path", &self.project.manifest_path)?;
        if self.project.package.trim().is_empty() {
            return Err(ContractError::Missing("project.package"));
        }
        if self.project.bin.is_some() == self.project.example.is_some() {
            return Err(ContractError::Invalid(
                "exactly one of project.bin or project.example must be set".to_owned(),
            ));
        }
        validate_path("source.root", &self.source.root)?;
        if let Some(vendor) = &self.resolver.vendor {
            validate_path("resolver.vendor", vendor)?;
        }
        validate_path("resolver.lockfile", &self.resolver.lockfile)?;
        if self.targets.is_empty() {
            return Err(ContractError::Missing("targets"));
        }

        let mut names = BTreeSet::new();
        for target in &self.targets {
            if !names.insert(target.name.clone()) {
                return Err(ContractError::Invalid(format!(
                    "duplicate target name: {}",
                    target.name
                )));
            }
            if target.name.trim().is_empty() {
                return Err(ContractError::Missing("targets.name"));
            }
            if target.triple.trim().is_empty() {
                return Err(ContractError::Missing("targets.triple"));
            }
            if matches!(target.artifact, ArtifactKind::Executable)
                && target.triple.starts_with("wasm32-")
            {
                return Err(ContractError::Invalid(format!(
                    "WASM target {} cannot declare executable artifact",
                    target.triple
                )));
            }
            if target.runtime.required && target.runtime.kind.is_none() {
                return Err(ContractError::Missing("targets.runtime.kind"));
            }
        }
        Ok(())
    }

    pub fn normalize(&self) -> Result<NormalizedContract, ContractError> {
        self.validate()?;
        let canonical_json = serde_json::to_string(self)?;
        let mut digest = Sha256::new();
        digest.update(canonical_json.as_bytes());
        Ok(NormalizedContract {
            contract: self.clone(),
            canonical_json,
            sha256: hex::encode(digest.finalize()),
        })
    }
}

fn validate_path(label: &'static str, value: &str) -> Result<(), ContractError> {
    normalize_path(value).map_err(|err| ContractError::Invalid(format!("{label}: {err}")))?;
    Ok(())
}

fn default_profile() -> String {
    "release".to_owned()
}

fn default_source_root() -> String {
    ".".to_owned()
}

fn default_lockfile() -> String {
    "Cargo.lock".to_owned()
}

fn default_channel() -> String {
    "stable".to_owned()
}

fn default_edition_floor() -> String {
    "2021".to_owned()
}

fn default_timeout_seconds() -> u64 {
    10
}

fn default_true() -> bool {
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_contract() -> &'static str {
        r#"
contract_version = 1

[project]
manifest_path = "Cargo.toml"
package = "app"
bin = "app"
profile = "release"

[source]
mode = "snapshot"
root = "."

[[targets]]
name = "wasi"
triple = "wasm32-wasip1"
artifact = "module"

[targets.interface]
required_exports = ["_start"]

[targets.runtime]
required = true
kind = "wasi"
expected_exit_code = 0
"#
    }

    #[test]
    fn parses_and_normalizes_final_contract_shape() {
        let contract = RouwdiContract::parse(valid_contract()).unwrap();
        let normalized = contract.normalize().unwrap();

        assert_eq!(contract.project.package, "app");
        assert_eq!(contract.targets[0].triple, "wasm32-wasip1");
        assert_eq!(normalized.sha256.len(), 64);
        assert!(normalized.canonical_json.contains("\"contract_version\":1"));
    }

    #[test]
    fn rejects_host_tool_path_style_inputs() {
        let invalid = valid_contract().replace("Cargo.toml", "C:/toolchains/Cargo.toml");
        let err = RouwdiContract::parse(&invalid).unwrap_err();

        assert!(err.to_string().contains("Windows drive prefix"));
    }

    #[test]
    fn rejects_ambiguous_primary_selection() {
        let invalid =
            valid_contract().replace("bin = \"app\"", "bin = \"app\"\nexample = \"demo\"");
        let err = RouwdiContract::parse(&invalid).unwrap_err();

        assert!(err.to_string().contains("exactly one"));
    }
}
