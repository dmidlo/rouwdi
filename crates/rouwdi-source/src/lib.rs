use rouwdi_vfs::{join_path, normalize_path, Storage, VfsError};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Debug, thiserror::Error)]
pub enum SourceError {
    #[error(transparent)]
    Vfs(#[from] VfsError),
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
}
