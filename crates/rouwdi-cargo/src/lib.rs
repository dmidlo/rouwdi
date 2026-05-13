use rouwdi_vfs::{join_path, normalize_path, Storage, VfsError};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

#[derive(Debug, thiserror::Error)]
pub enum CargoModelError {
    #[error(transparent)]
    Vfs(#[from] VfsError),
    #[error("Cargo.toml parse failure at {path}: {source}")]
    Toml {
        path: String,
        #[source]
        source: toml::de::Error,
    },
    #[error("missing [package] table in {0}")]
    MissingPackage(String),
    #[error("missing package.name in {0}")]
    MissingPackageName(String),
    #[error("invalid dependency declaration for {package}:{dependency}")]
    InvalidDependency { package: String, dependency: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CargoWorkspace {
    pub root_manifest_path: String,
    pub packages: Vec<CargoPackage>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CargoPackage {
    pub name: String,
    pub version: Option<String>,
    pub edition: Option<String>,
    pub manifest_path: String,
    pub targets: Vec<CargoTarget>,
    pub dependencies: Vec<CargoDependency>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CargoTarget {
    pub name: String,
    pub kind: CargoTargetKind,
    pub path: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CargoTargetKind {
    Lib,
    Bin,
    Example,
    Test,
    Bench,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CargoDependency {
    pub name: String,
    pub package: Option<String>,
    pub requirement: Option<String>,
    pub path: Option<String>,
    pub git: Option<String>,
    pub registry: Option<String>,
    pub optional: bool,
    pub features: Vec<String>,
    pub scope: DependencyScope,
    pub target_cfg: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DependencyScope {
    Normal,
    Build,
    Dev,
}

pub fn resolve_workspace(
    storage: &dyn Storage,
    manifest_path: &str,
) -> Result<CargoWorkspace, CargoModelError> {
    let root_manifest_path = normalize_path(manifest_path)?;
    let mut seen = BTreeSet::new();
    let mut packages = Vec::new();
    load_package_closure(storage, &root_manifest_path, &mut seen, &mut packages)?;
    packages.sort_by(|left, right| left.manifest_path.cmp(&right.manifest_path));
    Ok(CargoWorkspace {
        root_manifest_path,
        packages,
    })
}

fn load_package_closure(
    storage: &dyn Storage,
    manifest_path: &str,
    seen: &mut BTreeSet<String>,
    packages: &mut Vec<CargoPackage>,
) -> Result<(), CargoModelError> {
    if !seen.insert(manifest_path.to_owned()) {
        return Ok(());
    }
    let package = parse_manifest(storage, manifest_path)?;

    for member in workspace_members(storage, manifest_path)? {
        load_package_closure(storage, &member, seen, packages)?;
    }
    for dep in package
        .dependencies
        .iter()
        .filter_map(|dep| dep.path.as_deref())
    {
        let dep_manifest = join_path(dep, "Cargo.toml")?;
        load_package_closure(storage, &dep_manifest, seen, packages)?;
    }

    packages.push(package);
    Ok(())
}

pub fn parse_manifest(
    storage: &dyn Storage,
    manifest_path: &str,
) -> Result<CargoPackage, CargoModelError> {
    let manifest_path = normalize_path(manifest_path)?;
    let bytes = storage.read(&manifest_path)?;
    let text = String::from_utf8_lossy(&bytes);
    let value: toml::Value = toml::from_str(&text).map_err(|source| CargoModelError::Toml {
        path: manifest_path.clone(),
        source,
    })?;
    let package_table = value
        .get("package")
        .and_then(toml::Value::as_table)
        .ok_or_else(|| CargoModelError::MissingPackage(manifest_path.clone()))?;
    let name = string_field(package_table, "name")
        .ok_or_else(|| CargoModelError::MissingPackageName(manifest_path.clone()))?;
    let version = string_field(package_table, "version");
    let edition = string_field(package_table, "edition");
    let base = manifest_dir(&manifest_path);
    let mut dependencies = Vec::new();
    collect_dependency_table(
        &name,
        None,
        DependencyScope::Normal,
        value.get("dependencies"),
        &base,
        &mut dependencies,
    )?;
    collect_dependency_table(
        &name,
        None,
        DependencyScope::Build,
        value.get("build-dependencies"),
        &base,
        &mut dependencies,
    )?;
    collect_dependency_table(
        &name,
        None,
        DependencyScope::Dev,
        value.get("dev-dependencies"),
        &base,
        &mut dependencies,
    )?;
    collect_target_dependencies(&name, &value, &base, &mut dependencies)?;

    Ok(CargoPackage {
        name: name.clone(),
        version,
        edition,
        manifest_path: manifest_path.clone(),
        targets: collect_targets(&name, &value),
        dependencies,
    })
}

fn workspace_members(
    storage: &dyn Storage,
    manifest_path: &str,
) -> Result<Vec<String>, CargoModelError> {
    let bytes = storage.read(manifest_path)?;
    let text = String::from_utf8_lossy(&bytes);
    let value: toml::Value = toml::from_str(&text).map_err(|source| CargoModelError::Toml {
        path: manifest_path.to_owned(),
        source,
    })?;
    let base = manifest_dir(manifest_path);
    let mut members = Vec::new();
    if let Some(array) = value
        .get("workspace")
        .and_then(|workspace| workspace.get("members"))
        .and_then(toml::Value::as_array)
    {
        for member in array.iter().filter_map(toml::Value::as_str) {
            members.push(join_path(&join_path(&base, member)?, "Cargo.toml")?);
        }
    }
    Ok(members)
}

fn collect_targets(package_name: &str, value: &toml::Value) -> Vec<CargoTarget> {
    let mut targets = Vec::new();
    if value.get("lib").is_some() {
        let lib_name = value
            .get("lib")
            .and_then(|lib| lib.get("name"))
            .and_then(toml::Value::as_str)
            .unwrap_or(package_name)
            .to_owned();
        let path = value
            .get("lib")
            .and_then(|lib| lib.get("path"))
            .and_then(toml::Value::as_str)
            .map(str::to_owned);
        targets.push(CargoTarget {
            name: lib_name,
            kind: CargoTargetKind::Lib,
            path,
        });
    }
    collect_array_targets(
        value,
        "bin",
        CargoTargetKind::Bin,
        package_name,
        &mut targets,
    );
    collect_array_targets(
        value,
        "example",
        CargoTargetKind::Example,
        package_name,
        &mut targets,
    );
    if targets.is_empty() {
        targets.push(CargoTarget {
            name: package_name.to_owned(),
            kind: CargoTargetKind::Bin,
            path: Some("src/main.rs".to_owned()),
        });
    }
    targets
}

fn collect_array_targets(
    value: &toml::Value,
    key: &str,
    kind: CargoTargetKind,
    fallback_name: &str,
    targets: &mut Vec<CargoTarget>,
) {
    if let Some(array) = value.get(key).and_then(toml::Value::as_array) {
        for item in array.iter().filter_map(toml::Value::as_table) {
            targets.push(CargoTarget {
                name: string_field(item, "name").unwrap_or_else(|| fallback_name.to_owned()),
                kind: kind.clone(),
                path: string_field(item, "path"),
            });
        }
    }
}

fn collect_target_dependencies(
    package_name: &str,
    value: &toml::Value,
    base: &str,
    dependencies: &mut Vec<CargoDependency>,
) -> Result<(), CargoModelError> {
    let Some(targets) = value.get("target").and_then(toml::Value::as_table) else {
        return Ok(());
    };
    for (cfg, target_table) in targets {
        collect_dependency_table(
            package_name,
            Some(cfg.clone()),
            DependencyScope::Normal,
            target_table.get("dependencies"),
            base,
            dependencies,
        )?;
        collect_dependency_table(
            package_name,
            Some(cfg.clone()),
            DependencyScope::Build,
            target_table.get("build-dependencies"),
            base,
            dependencies,
        )?;
    }
    Ok(())
}

fn collect_dependency_table(
    package_name: &str,
    target_cfg: Option<String>,
    scope: DependencyScope,
    value: Option<&toml::Value>,
    base: &str,
    dependencies: &mut Vec<CargoDependency>,
) -> Result<(), CargoModelError> {
    let Some(table) = value.and_then(toml::Value::as_table) else {
        return Ok(());
    };
    for (name, value) in table {
        dependencies.push(parse_dependency(
            package_name,
            name,
            value,
            base,
            scope.clone(),
            target_cfg.clone(),
        )?);
    }
    Ok(())
}

fn parse_dependency(
    package_name: &str,
    name: &str,
    value: &toml::Value,
    base: &str,
    scope: DependencyScope,
    target_cfg: Option<String>,
) -> Result<CargoDependency, CargoModelError> {
    if let Some(requirement) = value.as_str() {
        return Ok(CargoDependency {
            name: name.to_owned(),
            package: None,
            requirement: Some(requirement.to_owned()),
            path: None,
            git: None,
            registry: None,
            optional: false,
            features: Vec::new(),
            scope,
            target_cfg,
        });
    }
    let Some(table) = value.as_table() else {
        return Err(CargoModelError::InvalidDependency {
            package: package_name.to_owned(),
            dependency: name.to_owned(),
        });
    };
    let raw_path = string_field(table, "path");
    let path = raw_path
        .as_deref()
        .map(|path| join_path(base, path))
        .transpose()?;
    Ok(CargoDependency {
        name: name.to_owned(),
        package: string_field(table, "package"),
        requirement: string_field(table, "version"),
        path,
        git: string_field(table, "git"),
        registry: string_field(table, "registry"),
        optional: table
            .get("optional")
            .and_then(toml::Value::as_bool)
            .unwrap_or(false),
        features: string_array_field(table, "features"),
        scope,
        target_cfg,
    })
}

fn string_field(table: &toml::map::Map<String, toml::Value>, key: &str) -> Option<String> {
    table
        .get(key)
        .and_then(toml::Value::as_str)
        .map(str::to_owned)
}

fn string_array_field(table: &toml::map::Map<String, toml::Value>, key: &str) -> Vec<String> {
    table
        .get(key)
        .and_then(toml::Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(toml::Value::as_str)
        .map(str::to_owned)
        .collect()
}

fn manifest_dir(path: &str) -> String {
    match path.rsplit_once('/') {
        Some((dir, _)) => dir.to_owned(),
        None => String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rouwdi_vfs::MemoryStorage;

    #[test]
    fn resolves_workspace_members_and_path_dependencies_without_host_cargo() {
        let mut storage = MemoryStorage::new();
        storage
            .write(
                "Cargo.toml",
                br#"
[workspace]
members = ["app"]

[package]
name = "root"
version = "0.1.0"
edition = "2021"

[dependencies]
helper = { path = "helper", features = ["derive"] }
"#,
            )
            .unwrap();
        storage
            .write(
                "app/Cargo.toml",
                br#"
[package]
name = "app"
version = "0.1.0"
edition = "2021"
"#,
            )
            .unwrap();
        storage
            .write(
                "helper/Cargo.toml",
                br#"
[package]
name = "helper"
version = "0.1.0"
edition = "2021"
"#,
            )
            .unwrap();

        let workspace = resolve_workspace(&storage, "Cargo.toml").unwrap();

        assert_eq!(
            workspace
                .packages
                .iter()
                .map(|package| package.name.as_str())
                .collect::<Vec<_>>(),
            vec!["root", "app", "helper"]
        );
        assert_eq!(
            workspace.packages[0].dependencies[0].path.as_deref(),
            Some("helper")
        );
    }
}
