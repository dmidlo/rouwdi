use rouwdi_vfs::{join_path, normalize_path, Storage, VfsError};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Debug, thiserror::Error)]
pub enum SourceError {
    #[error(transparent)]
    Vfs(#[from] VfsError),
    #[error("source cache failure: {0}")]
    Cache(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceFileProof {
    pub path: String,
    pub len: u64,
    pub sha256: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceSnapshot {
    pub root: String,
    pub files: Vec<SourceFileProof>,
    pub tree_sha256: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceCacheRequest {
    pub package: String,
    pub dependency: String,
    pub kind: SourceCacheKind,
    pub locator: String,
    pub requirement: Option<String>,
    pub target_cfg: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceCacheKind {
    Path,
    Git,
    Registry,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceCacheStatus {
    Cached,
    PlannedFetch,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceCacheProof {
    pub cache_root: String,
    pub entries: Vec<SourceCacheEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceCacheEntry {
    pub package: String,
    pub dependency: String,
    pub kind: SourceCacheKind,
    pub locator: String,
    pub requirement: Option<String>,
    pub target_cfg: Option<String>,
    pub status: SourceCacheStatus,
    pub source_tree_sha256: Option<String>,
    pub cache_path: Option<String>,
    pub files: Vec<SourceCacheFile>,
    pub reason: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceCacheFile {
    pub source_path: String,
    pub cache_path: String,
    pub len: u64,
    pub sha256: String,
}

pub fn snapshot_source(storage: &dyn Storage, root: &str) -> Result<SourceSnapshot, SourceError> {
    let root = normalize_path(root)?;
    let mut files = Vec::new();
    visit(storage, &root, &mut files)?;
    files.sort_by(|left, right| left.path.cmp(&right.path));

    let mut tree_digest = Sha256::new();
    for file in &files {
        tree_digest.update(file.path.as_bytes());
        tree_digest.update(b"\0");
        tree_digest.update(file.sha256.as_bytes());
        tree_digest.update(b"\0");
        tree_digest.update(file.len.to_le_bytes());
    }

    Ok(SourceSnapshot {
        root,
        files,
        tree_sha256: hex::encode(tree_digest.finalize()),
    })
}

pub fn materialize_source_cache(
    storage: &mut dyn Storage,
    cache_root: &str,
    requests: &[SourceCacheRequest],
) -> Result<SourceCacheProof, SourceError> {
    let cache_root = normalize_path(cache_root)?;
    let mut entries = Vec::new();
    for request in requests {
        entries.push(match request.kind {
            SourceCacheKind::Path => cache_path_source(storage, &cache_root, request)?,
            SourceCacheKind::Git | SourceCacheKind::Registry => planned_fetch_source(request),
        });
    }
    entries.sort_by(|left, right| {
        left.package
            .cmp(&right.package)
            .then_with(|| left.dependency.cmp(&right.dependency))
            .then_with(|| left.locator.cmp(&right.locator))
            .then_with(|| format!("{:?}", left.kind).cmp(&format!("{:?}", right.kind)))
    });
    Ok(SourceCacheProof {
        cache_root,
        entries,
    })
}

fn cache_path_source(
    storage: &mut dyn Storage,
    cache_root: &str,
    request: &SourceCacheRequest,
) -> Result<SourceCacheEntry, SourceError> {
    let snapshot = snapshot_source(storage, &request.locator)?;
    let cache_path = join_path(cache_root, &snapshot.tree_sha256)?;
    let mut files = Vec::new();
    for file in &snapshot.files {
        let relative = strip_snapshot_root(&snapshot.root, &file.path)?;
        let cached_path = join_path(&cache_path, &relative)?;
        let bytes = storage.read(&file.path)?;
        storage.write(&cached_path, &bytes)?;
        files.push(SourceCacheFile {
            source_path: file.path.clone(),
            cache_path: cached_path,
            len: file.len,
            sha256: file.sha256.clone(),
        });
    }
    Ok(SourceCacheEntry {
        package: request.package.clone(),
        dependency: request.dependency.clone(),
        kind: request.kind,
        locator: request.locator.clone(),
        requirement: request.requirement.clone(),
        target_cfg: request.target_cfg.clone(),
        status: SourceCacheStatus::Cached,
        source_tree_sha256: Some(snapshot.tree_sha256),
        cache_path: Some(cache_path),
        files,
        reason: None,
    })
}

fn planned_fetch_source(request: &SourceCacheRequest) -> SourceCacheEntry {
    SourceCacheEntry {
        package: request.package.clone(),
        dependency: request.dependency.clone(),
        kind: request.kind,
        locator: request.locator.clone(),
        requirement: request.requirement.clone(),
        target_cfg: request.target_cfg.clone(),
        status: SourceCacheStatus::PlannedFetch,
        source_tree_sha256: None,
        cache_path: None,
        files: Vec::new(),
        reason: Some(
            "source fetch and unpack must run inside rouwdi through host network/storage substrate; no host Cargo fallback is available".to_owned(),
        ),
    }
}

fn strip_snapshot_root(root: &str, path: &str) -> Result<String, SourceError> {
    let root = normalize_path(root)?;
    let path = normalize_path(path)?;
    if root.is_empty() {
        return Ok(path);
    }
    if let Some(relative) = path.strip_prefix(&format!("{root}/")) {
        return Ok(relative.to_owned());
    }
    Err(SourceError::Cache(format!(
        "source file {path} is outside snapshot root {root}"
    )))
}

fn visit(
    storage: &dyn Storage,
    path: &str,
    files: &mut Vec<SourceFileProof>,
) -> Result<(), SourceError> {
    for entry in storage.list(path)? {
        if should_skip(&entry.path) {
            continue;
        }
        if entry.is_dir {
            visit(storage, &entry.path, files)?;
        } else {
            let bytes = storage.read(&entry.path)?;
            let mut digest = Sha256::new();
            digest.update(&bytes);
            files.push(SourceFileProof {
                path: entry.path,
                len: bytes.len() as u64,
                sha256: hex::encode(digest.finalize()),
            });
        }
    }
    Ok(())
}

fn should_skip(path: &str) -> bool {
    path.split('/')
        .any(|part| matches!(part, ".git" | ".rouwdi" | "target"))
}

pub fn source_relative_path(root: &str, child: &str) -> Result<String, SourceError> {
    Ok(join_path(root, child)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rouwdi_vfs::MemoryStorage;

    #[test]
    fn source_snapshot_hashes_files_and_excludes_generated_state() {
        let mut storage = MemoryStorage::new();
        storage
            .write("Cargo.toml", b"[package]\nname='app'\n")
            .unwrap();
        storage.write("src/main.rs", b"fn main() {}\n").unwrap();
        storage
            .write(".rouwdi/runs/old/manifest.json", b"ignore")
            .unwrap();
        storage.write("target/debug/app", b"ignore").unwrap();

        let snapshot = snapshot_source(&storage, ".").unwrap();

        assert_eq!(snapshot.files.len(), 2);
        assert_eq!(snapshot.files[0].path, "Cargo.toml");
        assert_eq!(snapshot.tree_sha256.len(), 64);
    }

    #[test]
    fn source_snapshot_excludes_generated_state_inside_nested_project_root() {
        let mut storage = MemoryStorage::new();
        storage
            .write("examples/app/Cargo.toml", b"[package]\nname='app'\n")
            .unwrap();
        storage
            .write("examples/app/src/main.rs", b"fn main() {}\n")
            .unwrap();
        storage
            .write("examples/app/.rouwdi/runs/old/manifest.json", b"ignore")
            .unwrap();
        storage
            .write("examples/app/target/wasm32-wasip1/app.wasm", b"ignore")
            .unwrap();

        let snapshot = snapshot_source(&storage, "examples/app").unwrap();

        assert_eq!(
            snapshot
                .files
                .iter()
                .map(|file| file.path.as_str())
                .collect::<Vec<_>>(),
            vec!["examples/app/Cargo.toml", "examples/app/src/main.rs"]
        );
    }

    #[test]
    fn materializes_path_dependency_sources_into_content_addressed_cache() {
        let mut storage = MemoryStorage::new();
        storage
            .write("helper/Cargo.toml", b"[package]\nname='helper'\n")
            .unwrap();
        storage
            .write("helper/src/lib.rs", b"pub fn helper() {}\n")
            .unwrap();

        let proof = materialize_source_cache(
            &mut storage,
            ".rouwdi/cache/sources",
            &[SourceCacheRequest {
                package: "app".to_owned(),
                dependency: "helper".to_owned(),
                kind: SourceCacheKind::Path,
                locator: "helper".to_owned(),
                requirement: None,
                target_cfg: None,
            }],
        )
        .unwrap();

        let entry = &proof.entries[0];
        assert_eq!(entry.status, SourceCacheStatus::Cached);
        assert_eq!(entry.source_tree_sha256.as_deref().unwrap().len(), 64);
        assert_eq!(entry.files.len(), 2);
        assert!(entry
            .files
            .iter()
            .any(|file| file.cache_path.ends_with("src/lib.rs")));
        assert_eq!(
            storage
                .read(
                    entry
                        .files
                        .iter()
                        .find(|file| file.cache_path.ends_with("src/lib.rs"))
                        .unwrap()
                        .cache_path
                        .as_str()
                )
                .unwrap(),
            b"pub fn helper() {}\n"
        );
    }

    #[test]
    fn records_remote_sources_as_planned_fetches_without_host_cargo_fallback() {
        let mut storage = MemoryStorage::new();

        let proof = materialize_source_cache(
            &mut storage,
            ".rouwdi/cache/sources",
            &[SourceCacheRequest {
                package: "app".to_owned(),
                dependency: "serde".to_owned(),
                kind: SourceCacheKind::Registry,
                locator: "crates-io".to_owned(),
                requirement: Some("1".to_owned()),
                target_cfg: None,
            }],
        )
        .unwrap();

        let entry = &proof.entries[0];
        assert_eq!(entry.status, SourceCacheStatus::PlannedFetch);
        assert!(entry.reason.as_deref().unwrap().contains("no host Cargo"));
        assert!(entry.cache_path.is_none());
    }
}
