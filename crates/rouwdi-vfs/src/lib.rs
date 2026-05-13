use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

#[derive(Debug, thiserror::Error)]
pub enum VfsError {
    #[error("path escapes virtual root: {0}")]
    PathEscapesRoot(String),
    #[error("path is absolute and cannot be used in virtual storage: {0}")]
    AbsolutePath(String),
    #[error("path contains a Windows drive prefix and cannot be used in virtual storage: {0}")]
    DrivePath(String),
    #[error("entry not found: {0}")]
    NotFound(String),
    #[error("entry is not a directory: {0}")]
    NotDirectory(String),
    #[error("entry is not a file: {0}")]
    NotFile(String),
    #[error("host storage I/O failure at {path}: {source}")]
    Io {
        path: String,
        #[source]
        source: std::io::Error,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DirEntry {
    pub path: String,
    pub is_dir: bool,
    pub len: u64,
}

pub trait Storage {
    fn read(&self, path: &str) -> Result<Vec<u8>, VfsError>;
    fn write(&mut self, path: &str, bytes: &[u8]) -> Result<(), VfsError>;
    fn list(&self, path: &str) -> Result<Vec<DirEntry>, VfsError>;
    fn mkdir(&mut self, path: &str) -> Result<(), VfsError>;
    fn remove(&mut self, path: &str) -> Result<(), VfsError>;
}

pub fn normalize_path(path: &str) -> Result<String, VfsError> {
    let rendered = path.replace('\\', "/");
    if rendered.starts_with('/') {
        return Err(VfsError::AbsolutePath(path.to_owned()));
    }
    if rendered.len() >= 2 && rendered.as_bytes()[1] == b':' {
        return Err(VfsError::DrivePath(path.to_owned()));
    }

    let mut parts: Vec<&str> = Vec::new();
    for part in rendered.split('/') {
        match part {
            "" | "." => {}
            ".." => {
                if parts.pop().is_none() {
                    return Err(VfsError::PathEscapesRoot(path.to_owned()));
                }
            }
            value => parts.push(value),
        }
    }
    Ok(parts.join("/"))
}

pub fn join_path(base: &str, child: &str) -> Result<String, VfsError> {
    let base = normalize_path(base)?;
    let child = normalize_path(child)?;
    match (base.is_empty(), child.is_empty()) {
        (true, true) => Ok(String::new()),
        (true, false) => Ok(child),
        (false, true) => Ok(base),
        (false, false) => Ok(format!("{base}/{child}")),
    }
}

fn parent_dirs(path: &str) -> Vec<String> {
    let mut parents = Vec::new();
    let mut current = String::new();
    for part in path.split('/').filter(|part| !part.is_empty()) {
        if !current.is_empty() {
            parents.push(current.clone());
            current.push('/');
        }
        current.push_str(part);
    }
    parents
}

#[derive(Debug, Default, Clone)]
pub struct MemoryStorage {
    files: BTreeMap<String, Vec<u8>>,
    dirs: BTreeSet<String>,
}

impl MemoryStorage {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn put_text(&mut self, path: &str, text: &str) -> Result<(), VfsError> {
        self.write(path, text.as_bytes())
    }
}

impl Storage for MemoryStorage {
    fn read(&self, path: &str) -> Result<Vec<u8>, VfsError> {
        let path = normalize_path(path)?;
        self.files
            .get(&path)
            .cloned()
            .ok_or(VfsError::NotFound(path))
    }

    fn write(&mut self, path: &str, bytes: &[u8]) -> Result<(), VfsError> {
        let path = normalize_path(path)?;
        for parent in parent_dirs(&path) {
            self.dirs.insert(parent);
        }
        self.files.insert(path, bytes.to_vec());
        Ok(())
    }

    fn list(&self, path: &str) -> Result<Vec<DirEntry>, VfsError> {
        let path = normalize_path(path)?;
        if !path.is_empty() && self.files.contains_key(&path) {
            return Err(VfsError::NotDirectory(path));
        }
        if !path.is_empty() && !self.dirs.contains(&path) {
            let prefix = format!("{path}/");
            if !self
                .files
                .keys()
                .any(|candidate| candidate.starts_with(&prefix))
            {
                return Err(VfsError::NotFound(path));
            }
        }

        let prefix = if path.is_empty() {
            String::new()
        } else {
            format!("{path}/")
        };
        let mut children: BTreeMap<String, DirEntry> = BTreeMap::new();
        for dir in &self.dirs {
            if !dir.starts_with(&prefix) || dir == &path {
                continue;
            }
            let remainder = &dir[prefix.len()..];
            if let Some((name, _)) = remainder.split_once('/') {
                let child_path = join_path(&path, name)?;
                children.entry(child_path.clone()).or_insert(DirEntry {
                    path: child_path,
                    is_dir: true,
                    len: 0,
                });
            } else {
                children.insert(
                    dir.clone(),
                    DirEntry {
                        path: dir.clone(),
                        is_dir: true,
                        len: 0,
                    },
                );
            }
        }
        for (file, bytes) in &self.files {
            if !file.starts_with(&prefix) {
                continue;
            }
            let remainder = &file[prefix.len()..];
            if let Some((name, _)) = remainder.split_once('/') {
                let child_path = join_path(&path, name)?;
                children.entry(child_path.clone()).or_insert(DirEntry {
                    path: child_path,
                    is_dir: true,
                    len: 0,
                });
            } else {
                children.insert(
                    file.clone(),
                    DirEntry {
                        path: file.clone(),
                        is_dir: false,
                        len: bytes.len() as u64,
                    },
                );
            }
        }
        Ok(children.into_values().collect())
    }

    fn mkdir(&mut self, path: &str) -> Result<(), VfsError> {
        let path = normalize_path(path)?;
        for parent in parent_dirs(&path) {
            self.dirs.insert(parent);
        }
        if !path.is_empty() {
            self.dirs.insert(path);
        }
        Ok(())
    }

    fn remove(&mut self, path: &str) -> Result<(), VfsError> {
        let path = normalize_path(path)?;
        self.files.remove(&path);
        self.dirs.remove(&path);
        let prefix = format!("{path}/");
        self.files
            .retain(|candidate, _| !candidate.starts_with(&prefix));
        self.dirs
            .retain(|candidate| !candidate.starts_with(&prefix));
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct HostStorage {
    root: PathBuf,
}

impl HostStorage {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    fn resolve(&self, path: &str) -> Result<PathBuf, VfsError> {
        let normalized = normalize_path(path)?;
        Ok(self.root.join(Path::new(&normalized)))
    }

    fn rel(&self, path: &Path) -> Result<String, VfsError> {
        let rel = path
            .strip_prefix(&self.root)
            .map_err(|_| VfsError::PathEscapesRoot(path.to_string_lossy().replace('\\', "/")))?;
        Ok(rel.to_string_lossy().replace('\\', "/"))
    }
}

impl Storage for HostStorage {
    fn read(&self, path: &str) -> Result<Vec<u8>, VfsError> {
        let resolved = self.resolve(path)?;
        std::fs::read(&resolved).map_err(|source| VfsError::Io {
            path: resolved.display().to_string(),
            source,
        })
    }

    fn write(&mut self, path: &str, bytes: &[u8]) -> Result<(), VfsError> {
        let resolved = self.resolve(path)?;
        if let Some(parent) = resolved.parent() {
            std::fs::create_dir_all(parent).map_err(|source| VfsError::Io {
                path: parent.display().to_string(),
                source,
            })?;
        }
        std::fs::write(&resolved, bytes).map_err(|source| VfsError::Io {
            path: resolved.display().to_string(),
            source,
        })
    }

    fn list(&self, path: &str) -> Result<Vec<DirEntry>, VfsError> {
        let resolved = self.resolve(path)?;
        if !resolved.exists() {
            return Err(VfsError::NotFound(path.to_owned()));
        }
        if !resolved.is_dir() {
            return Err(VfsError::NotDirectory(path.to_owned()));
        }
        let mut entries = Vec::new();
        for item in std::fs::read_dir(&resolved).map_err(|source| VfsError::Io {
            path: resolved.display().to_string(),
            source,
        })? {
            let item = item.map_err(|source| VfsError::Io {
                path: resolved.display().to_string(),
                source,
            })?;
            let metadata = item.metadata().map_err(|source| VfsError::Io {
                path: item.path().display().to_string(),
                source,
            })?;
            entries.push(DirEntry {
                path: self.rel(&item.path())?,
                is_dir: metadata.is_dir(),
                len: metadata.len(),
            });
        }
        entries.sort_by(|left, right| left.path.cmp(&right.path));
        Ok(entries)
    }

    fn mkdir(&mut self, path: &str) -> Result<(), VfsError> {
        let resolved = self.resolve(path)?;
        std::fs::create_dir_all(&resolved).map_err(|source| VfsError::Io {
            path: resolved.display().to_string(),
            source,
        })
    }

    fn remove(&mut self, path: &str) -> Result<(), VfsError> {
        let resolved = self.resolve(path)?;
        if resolved.is_dir() {
            std::fs::remove_dir_all(&resolved).map_err(|source| VfsError::Io {
                path: resolved.display().to_string(),
                source,
            })
        } else if resolved.exists() {
            std::fs::remove_file(&resolved).map_err(|source| VfsError::Io {
                path: resolved.display().to_string(),
                source,
            })
        } else {
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_relative_paths_without_escaping_root() {
        assert_eq!(normalize_path("./a/b/../c").unwrap(), "a/c");
        assert!(normalize_path("../secret").is_err());
        assert!(normalize_path("C:/secret").is_err());
        assert!(normalize_path("/secret").is_err());
    }

    #[test]
    fn memory_storage_lists_direct_children() {
        let mut storage = MemoryStorage::new();
        storage.write("src/main.rs", b"fn main() {}").unwrap();
        storage.write("README.md", b"hello").unwrap();

        let entries = storage.list("").unwrap();
        assert_eq!(
            entries
                .into_iter()
                .map(|entry| (entry.path, entry.is_dir))
                .collect::<Vec<_>>(),
            vec![("README.md".to_owned(), false), ("src".to_owned(), true),]
        );
    }
}
