use rouwdi_cargo::CargoWorkspace;
use rouwdi_contract::NormalizedContract;
use rouwdi_source::SourceSnapshot;
use rouwdi_targets::{CompilerEngineIdentity, TargetPack};
use rouwdi_vfs::{Storage, VfsError};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Debug, thiserror::Error)]
pub enum ProofError {
    #[error(transparent)]
    Vfs(#[from] VfsError),
    #[error("proof JSON failure: {0}")]
    Json(#[from] serde_json::Error),
    #[error("proof verification failure: {0}")]
    Verification(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RunStatus {
    Succeeded,
    Failed,
    Unsupported,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UnsupportedCapability {
    pub capability: String,
    pub required_by: String,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HashEntry {
    pub label: String,
    pub path: String,
    pub sha256: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArtifactManifestEntry {
    pub target: String,
    pub path: String,
    pub artifact_kind: String,
    pub sha256: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WasmModuleExport {
    pub name: String,
    pub kind: WasmExportKind,
    pub index: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WasmExportKind {
    Function,
    Table,
    Memory,
    Global,
    Tag,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum NativeExecutableFormat {
    Elf,
    MachO,
    Pe,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RouwdiRunManifest {
    pub run_id: String,
    pub status: RunStatus,
    pub contract_sha256: String,
    pub source_tree_sha256: String,
    pub compiler_engine: CompilerEngineIdentity,
    pub target_packs: Vec<TargetPack>,
    pub artifacts: Vec<ArtifactManifestEntry>,
    pub unsupported: Vec<UnsupportedCapability>,
    pub proof_files: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProofBundle {
    pub manifest: RouwdiRunManifest,
    pub normalized_contract: NormalizedContract,
    pub source_snapshot: SourceSnapshot,
    pub cargo_workspace: CargoWorkspace,
    pub hashes: Vec<HashEntry>,
}

impl ProofBundle {
    pub fn write_to_storage(
        &self,
        storage: &mut dyn Storage,
        run_root: &str,
    ) -> Result<Vec<String>, ProofError> {
        let files = [
            ("manifest.json", serde_json::to_vec_pretty(&self.manifest)?),
            (
                "rouwdi.toml.normalized.json",
                serde_json::to_vec_pretty(&self.normalized_contract)?,
            ),
            (
                "source/source-snapshot.json",
                serde_json::to_vec_pretty(&self.source_snapshot)?,
            ),
            (
                "source/source-tree.hashes.json",
                serde_json::to_vec_pretty(&self.source_snapshot.files)?,
            ),
            (
                "graph/cargo-resolve.json",
                serde_json::to_vec_pretty(&self.cargo_workspace)?,
            ),
            (
                "graph/build-plan.json",
                serde_json::to_vec_pretty(&self.manifest)?,
            ),
            (
                "toolchain/rouwdi-engine.json",
                serde_json::to_vec_pretty(&self.manifest.compiler_engine)?,
            ),
            (
                "toolchain/target-packs.json",
                serde_json::to_vec_pretty(&self.manifest.target_packs)?,
            ),
            (
                "proofs/hashes.json",
                serde_json::to_vec_pretty(&self.hashes)?,
            ),
        ];
        let mut written = Vec::new();
        for (relative, bytes) in files {
            let path = if run_root.is_empty() {
                relative.to_owned()
            } else {
                format!("{run_root}/{relative}")
            };
            storage.write(&path, &bytes)?;
            written.push(path);
        }
        storage.write(
            &format!("{run_root}/events.jsonl"),
            format!(
                "{{\"event\":\"run-finalized\",\"run_id\":\"{}\",\"status\":\"{:?}\"}}\n",
                self.manifest.run_id, self.manifest.status
            )
            .as_bytes(),
        )?;
        written.push(format!("{run_root}/events.jsonl"));
        Ok(written)
    }
}

pub fn hash_bytes(bytes: &[u8]) -> String {
    let mut digest = Sha256::new();
    digest.update(bytes);
    hex::encode(digest.finalize())
}

pub fn hash_storage_file(storage: &dyn Storage, path: &str) -> Result<String, ProofError> {
    Ok(hash_bytes(&storage.read(path)?))
}

pub fn verify_manifest_hashes(
    storage: &dyn Storage,
    hash_entries: &[HashEntry],
) -> Result<(), ProofError> {
    for entry in hash_entries {
        let actual = hash_storage_file(storage, &entry.path)?;
        if actual != entry.sha256 {
            return Err(ProofError::Verification(format!(
                "hash mismatch for {}: expected {}, got {}",
                entry.path, entry.sha256, actual
            )));
        }
    }
    Ok(())
}

pub fn parse_wasm_exports(bytes: &[u8]) -> Result<Vec<WasmModuleExport>, ProofError> {
    if bytes.len() < 8 || &bytes[..4] != b"\0asm" || &bytes[4..8] != b"\x01\0\0\0" {
        return Err(ProofError::Verification(
            "not a WebAssembly 1.0 module".to_owned(),
        ));
    }
    let mut offset = 8;
    while offset < bytes.len() {
        let section_id = bytes[offset];
        offset += 1;
        let (section_len, next) = read_varuint(bytes, offset)?;
        offset = next;
        let section_end = offset
            .checked_add(section_len as usize)
            .ok_or_else(|| ProofError::Verification("WASM section length overflow".to_owned()))?;
        if section_end > bytes.len() {
            return Err(ProofError::Verification(
                "WASM section extends past end of module".to_owned(),
            ));
        }
        if section_id == 7 {
            return parse_export_section(bytes, offset, section_end);
        }
        offset = section_end;
    }
    Ok(Vec::new())
}

pub fn missing_wasm_exports(exports: &[WasmModuleExport], required: &[String]) -> Vec<String> {
    required
        .iter()
        .filter(|required| !exports.iter().any(|export| export.name == **required))
        .cloned()
        .collect()
}

pub fn classify_native_executable(bytes: &[u8]) -> NativeExecutableFormat {
    if bytes.starts_with(b"\x7fELF") {
        NativeExecutableFormat::Elf
    } else if bytes.starts_with(b"MZ") {
        NativeExecutableFormat::Pe
    } else if matches!(
        bytes.get(..4),
        Some(b"\xfe\xed\xfa\xce")
            | Some(b"\xce\xfa\xed\xfe")
            | Some(b"\xfe\xed\xfa\xcf")
            | Some(b"\xcf\xfa\xed\xfe")
            | Some(b"\xca\xfe\xba\xbe")
            | Some(b"\xbe\xba\xfe\xca")
    ) {
        NativeExecutableFormat::MachO
    } else {
        NativeExecutableFormat::Unknown
    }
}

fn parse_export_section(
    bytes: &[u8],
    mut offset: usize,
    section_end: usize,
) -> Result<Vec<WasmModuleExport>, ProofError> {
    let (count, next) = read_varuint(bytes, offset)?;
    offset = next;
    let mut exports = Vec::new();
    for _ in 0..count {
        let (name_len, name_start) = read_varuint(bytes, offset)?;
        let name_end = name_start
            .checked_add(name_len as usize)
            .ok_or_else(|| ProofError::Verification("WASM export name overflow".to_owned()))?;
        if name_end > section_end {
            return Err(ProofError::Verification(
                "WASM export name extends past export section".to_owned(),
            ));
        }
        let name = std::str::from_utf8(&bytes[name_start..name_end])
            .map_err(|err| ProofError::Verification(format!("invalid export name UTF-8: {err}")))?
            .to_owned();
        offset = name_end;
        let kind_byte = *bytes
            .get(offset)
            .ok_or_else(|| ProofError::Verification("missing WASM export kind".to_owned()))?;
        offset += 1;
        let (index, next) = read_varuint(bytes, offset)?;
        offset = next;
        exports.push(WasmModuleExport {
            name,
            kind: export_kind(kind_byte),
            index,
        });
    }
    Ok(exports)
}

fn read_varuint(bytes: &[u8], mut offset: usize) -> Result<(u32, usize), ProofError> {
    let mut result = 0u32;
    let mut shift = 0;
    loop {
        let byte = *bytes
            .get(offset)
            .ok_or_else(|| ProofError::Verification("unexpected end of varuint".to_owned()))?;
        offset += 1;
        result |= ((byte & 0x7f) as u32)
            .checked_shl(shift)
            .ok_or_else(|| ProofError::Verification("varuint shift overflow".to_owned()))?;
        if byte & 0x80 == 0 {
            return Ok((result, offset));
        }
        shift += 7;
        if shift > 28 {
            return Err(ProofError::Verification("varuint is too large".to_owned()));
        }
    }
}

fn export_kind(kind: u8) -> WasmExportKind {
    match kind {
        0 => WasmExportKind::Function,
        1 => WasmExportKind::Table,
        2 => WasmExportKind::Memory,
        3 => WasmExportKind::Global,
        4 => WasmExportKind::Tag,
        _ => WasmExportKind::Unknown,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rouwdi_vfs::MemoryStorage;

    fn varuint(mut value: u32) -> Vec<u8> {
        let mut out = Vec::new();
        loop {
            let mut byte = (value & 0x7f) as u8;
            value >>= 7;
            if value != 0 {
                byte |= 0x80;
            }
            out.push(byte);
            if value == 0 {
                return out;
            }
        }
    }

    fn wasm_with_exports(exports: &[(&str, u8)]) -> Vec<u8> {
        let mut section = Vec::new();
        section.extend(varuint(exports.len() as u32));
        for (name, kind) in exports {
            section.extend(varuint(name.len() as u32));
            section.extend(name.as_bytes());
            section.push(*kind);
            section.extend(varuint(0));
        }
        let mut module = b"\0asm\x01\0\0\0".to_vec();
        module.push(7);
        module.extend(varuint(section.len() as u32));
        module.extend(section);
        module
    }

    #[test]
    fn verifies_hash_entries_against_storage() {
        let mut storage = MemoryStorage::new();
        storage.write("artifact", b"bytes").unwrap();
        let hashes = vec![HashEntry {
            label: "artifact".to_owned(),
            path: "artifact".to_owned(),
            sha256: hash_storage_file(&storage, "artifact").unwrap(),
        }];

        verify_manifest_hashes(&storage, &hashes).unwrap();
    }

    #[test]
    fn parses_wasm_exports_without_host_runtime() {
        let module = wasm_with_exports(&[("_start", 0), ("memory", 2)]);
        let exports = parse_wasm_exports(&module).unwrap();

        assert_eq!(exports[0].name, "_start");
        assert_eq!(exports[0].kind, WasmExportKind::Function);
        assert_eq!(
            missing_wasm_exports(&exports, &["_start".to_owned(), "missing".to_owned()]),
            vec!["missing".to_owned()]
        );
    }

    #[test]
    fn classifies_native_executable_headers() {
        assert_eq!(
            classify_native_executable(b"\x7fELF..."),
            NativeExecutableFormat::Elf
        );
        assert_eq!(
            classify_native_executable(b"MZ..."),
            NativeExecutableFormat::Pe
        );
        assert_eq!(
            classify_native_executable(b"\xfe\xed\xfa\xcf..."),
            NativeExecutableFormat::MachO
        );
        assert_eq!(
            classify_native_executable(b"script"),
            NativeExecutableFormat::Unknown
        );
    }
}
