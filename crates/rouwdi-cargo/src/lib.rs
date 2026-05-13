use rouwdi_vfs::{join_path, normalize_path, Storage, VfsError};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

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
    #[error("package selected by rouwdi contract was not found: {0}")]
    MissingSelectedPackage(String),
    #[error("target selected by rouwdi contract was not found: {package}:{target}")]
    MissingSelectedTarget { package: String, target: String },
    #[error("feature selected by rouwdi contract was not found: {package}/{feature}")]
    MissingFeature { package: String, feature: String },
    #[error("resolver is frozen but lockfile is missing: {0}")]
    MissingFrozenLockfile(String),
    #[error("unsupported target cfg expression for {triple}: {cfg}")]
    UnsupportedTargetCfg { cfg: String, triple: String },
    #[error("frozen resolver lockfile is missing dependency {dependency} from {locator}")]
    MissingLockedDependency { dependency: String, locator: String },
    #[error("locked registry dependency is missing a checksum: {0}")]
    MissingRegistryChecksum(String),
    #[error("locked dependency {dependency} source does not match {expected}")]
    LockedSourceMismatch {
        dependency: String,
        expected: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CargoWorkspace {
    pub root_manifest_path: String,
    pub packages: Vec<CargoPackage>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CargoLockfile {
    pub path: String,
    pub version: Option<i64>,
    pub packages: Vec<CargoLockedPackage>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CargoLockedPackage {
    pub name: String,
    pub version: String,
    pub source: Option<String>,
    pub checksum: Option<String>,
    pub dependencies: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CargoPackage {
    pub name: String,
    pub version: Option<String>,
    pub edition: Option<String>,
    pub manifest_path: String,
    pub build_script: Option<String>,
    pub targets: Vec<CargoTarget>,
    pub dependencies: Vec<CargoDependency>,
    pub features: Vec<CargoFeature>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CargoTarget {
    pub name: String,
    pub kind: CargoTargetKind,
    pub path: Option<String>,
    pub proc_macro: bool,
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
    pub default_features: bool,
    pub features: Vec<String>,
    pub scope: DependencyScope,
    pub target_cfg: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DependencyScope {
    Normal,
    Build,
    Dev,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CargoFeature {
    pub name: String,
    pub members: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CargoFeatureResolution {
    pub selected_package: String,
    pub default_features: bool,
    pub requested_features: Vec<String>,
    pub packages: Vec<PackageFeatureResolution>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PackageFeatureResolution {
    pub package: String,
    pub enabled_features: Vec<String>,
    pub enabled_optional_dependencies: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CargoSourceFetchPlan {
    pub entries: Vec<CargoSourceFetch>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct CargoSourceFetch {
    pub package: String,
    pub dependency: String,
    pub scope: DependencyScope,
    pub kind: CargoSourceKind,
    pub locator: String,
    pub requirement: Option<String>,
    pub target_cfg: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CargoSourceKind {
    Path,
    Git,
    Registry,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CargoBuildPlan {
    pub units: Vec<CompileUnit>,
    pub edges: Vec<CompileUnitEdge>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompileUnit {
    pub id: String,
    pub package: String,
    pub manifest_path: String,
    pub target: String,
    pub source_path: Option<String>,
    pub target_kind: CargoTargetKind,
    pub phase: CompilePhase,
    pub triple: String,
    pub profile: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CompilePhase {
    BuildScript,
    ProcMacro,
    Rust,
    Link,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompileUnitEdge {
    pub from: String,
    pub to: String,
    pub reason: String,
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

pub fn resolve_features(
    workspace: &CargoWorkspace,
    selected_package: &str,
    default_features: bool,
    requested_features: &[String],
) -> Result<CargoFeatureResolution, CargoModelError> {
    let packages_by_name = workspace
        .packages
        .iter()
        .map(|package| (package.name.clone(), package))
        .collect::<BTreeMap<_, _>>();
    if !packages_by_name.contains_key(selected_package) {
        return Err(CargoModelError::MissingSelectedPackage(
            selected_package.to_owned(),
        ));
    }

    let mut active_packages = BTreeSet::from([selected_package.to_owned()]);
    let mut enabled_features = BTreeMap::<String, BTreeSet<String>>::new();
    let mut enabled_optional_dependencies = BTreeMap::<String, BTreeSet<String>>::new();
    let mut queue = Vec::<(String, String)>::new();

    if default_features {
        queue.push((selected_package.to_owned(), "default".to_owned()));
    }
    for feature in requested_features {
        queue.push((selected_package.to_owned(), feature.clone()));
    }

    let mut changed = true;
    while changed {
        changed = false;

        while let Some((package_name, feature_name)) = queue.pop() {
            let package = packages_by_name
                .get(&package_name)
                .ok_or_else(|| CargoModelError::MissingSelectedPackage(package_name.clone()))?;
            let feature_map = package
                .features
                .iter()
                .map(|feature| (feature.name.as_str(), feature))
                .collect::<BTreeMap<_, _>>();
            if let Some(feature) = feature_map.get(feature_name.as_str()) {
                if enabled_features
                    .entry(package_name.clone())
                    .or_default()
                    .insert(feature_name.clone())
                {
                    changed = true;
                    for member in &feature.members {
                        apply_feature_member(
                            workspace,
                            package,
                            member,
                            &mut active_packages,
                            &mut enabled_optional_dependencies,
                            &mut queue,
                        )?;
                    }
                }
            } else if feature_name == "default" {
                continue;
            } else if enable_optional_dependency(
                workspace,
                package,
                &feature_name,
                &mut active_packages,
                &mut enabled_optional_dependencies,
                &mut queue,
            )? {
                changed = true;
            } else {
                return Err(CargoModelError::MissingFeature {
                    package: package_name,
                    feature: feature_name,
                });
            }
        }

        for package_name in active_packages.clone() {
            let Some(package) = packages_by_name.get(&package_name) else {
                continue;
            };
            for dependency in package
                .dependencies
                .iter()
                .filter(|dependency| dependency.scope != DependencyScope::Dev)
            {
                if dependency.optional
                    && !enabled_optional_dependencies
                        .get(&package.name)
                        .is_some_and(|enabled| enabled.contains(&dependency.name))
                {
                    continue;
                }
                if activate_dependency_package(
                    workspace,
                    dependency,
                    &mut active_packages,
                    &mut queue,
                )? {
                    changed = true;
                }
            }
        }
    }

    let packages = active_packages
        .into_iter()
        .map(|package| {
            let mut enabled = enabled_features
                .remove(&package)
                .unwrap_or_default()
                .into_iter()
                .collect::<Vec<_>>();
            enabled.sort();
            let mut optional = enabled_optional_dependencies
                .remove(&package)
                .unwrap_or_default()
                .into_iter()
                .collect::<Vec<_>>();
            optional.sort();
            PackageFeatureResolution {
                package,
                enabled_features: enabled,
                enabled_optional_dependencies: optional,
            }
        })
        .collect::<Vec<_>>();

    Ok(CargoFeatureResolution {
        selected_package: selected_package.to_owned(),
        default_features,
        requested_features: requested_features.to_vec(),
        packages,
    })
}

pub fn plan_build(
    workspace: &CargoWorkspace,
    feature_resolution: &CargoFeatureResolution,
    selected_package: &str,
    selected_target: &str,
    selected_target_kind: CargoTargetKind,
    profile: &str,
    triples: &[String],
) -> Result<CargoBuildPlan, CargoModelError> {
    let root = workspace
        .packages
        .iter()
        .find(|package| package.name == selected_package)
        .ok_or_else(|| CargoModelError::MissingSelectedPackage(selected_package.to_owned()))?;
    if !root
        .targets
        .iter()
        .any(|target| target.name == selected_target && target.kind == selected_target_kind)
    {
        return Err(CargoModelError::MissingSelectedTarget {
            package: selected_package.to_owned(),
            target: selected_target.to_owned(),
        });
    }

    let build_triples_by_package =
        build_triples_by_package(workspace, feature_resolution, selected_package, triples)?;
    let packages_by_name = workspace
        .packages
        .iter()
        .map(|package| (package.name.as_str(), package))
        .collect::<BTreeMap<_, _>>();

    let mut units = Vec::new();
    let mut edges = Vec::new();
    for package in workspace.packages.iter().filter_map(|package| {
        build_triples_by_package
            .get(&package.name)
            .map(|package_triples| (package, package_triples))
    }) {
        let (package, package_triples) = package;
        let package_anchor = package.name.replace('-', "_");
        let mut package_predecessors = Vec::new();
        if package.build_script.is_some() {
            for triple in package_triples {
                let id = format!("{package_anchor}:build-script:{triple}");
                units.push(CompileUnit {
                    id: id.clone(),
                    package: package.name.clone(),
                    manifest_path: package.manifest_path.clone(),
                    target: package
                        .build_script
                        .clone()
                        .unwrap_or_else(|| "build.rs".to_owned()),
                    source_path: package.build_script.clone(),
                    target_kind: CargoTargetKind::Bin,
                    phase: CompilePhase::BuildScript,
                    triple: triple.clone(),
                    profile: profile.to_owned(),
                });
                package_predecessors.push((
                    id,
                    Some(triple.clone()),
                    "build script directives".to_owned(),
                ));
            }
        }
        for target in package
            .targets
            .iter()
            .filter(|target| target.kind == CargoTargetKind::Lib && target.proc_macro)
        {
            let id = format!("{package_anchor}:proc-macro:{}", target.name);
            units.push(CompileUnit {
                id: id.clone(),
                package: package.name.clone(),
                manifest_path: package.manifest_path.clone(),
                target: target.name.clone(),
                source_path: target.path.clone(),
                target_kind: target.kind.clone(),
                phase: CompilePhase::ProcMacro,
                triple: "compile-time-wasm".to_owned(),
                profile: profile.to_owned(),
            });
            package_predecessors.push((id, None, "proc macro expansion".to_owned()));
        }

        for triple in package_triples {
            for target in package.targets.iter().filter(|target| {
                target.kind == CargoTargetKind::Lib
                    || (package.name == selected_package
                        && target.kind == selected_target_kind
                        && target.name == selected_target)
            }) {
                let id = format!("{package_anchor}:rust:{}:{triple}", target.name);
                units.push(CompileUnit {
                    id: id.clone(),
                    package: package.name.clone(),
                    manifest_path: package.manifest_path.clone(),
                    target: target.name.clone(),
                    source_path: target.path.clone(),
                    target_kind: target.kind.clone(),
                    phase: CompilePhase::Rust,
                    triple: triple.clone(),
                    profile: profile.to_owned(),
                });
                for (from, predecessor_triple, reason) in &package_predecessors {
                    if predecessor_triple
                        .as_ref()
                        .is_some_and(|predecessor_triple| predecessor_triple != triple)
                    {
                        continue;
                    }
                    edges.push(CompileUnitEdge {
                        from: from.clone(),
                        to: id.clone(),
                        reason: reason.clone(),
                    });
                }
            }
        }
    }

    for (package, package_triples) in workspace.packages.iter().filter_map(|package| {
        build_triples_by_package
            .get(&package.name)
            .map(|package_triples| (package, package_triples))
    }) {
        for dependency in package.dependencies.iter().filter(|dependency| {
            dependency.scope == DependencyScope::Normal
                || dependency.scope == DependencyScope::Build
        }) {
            if !dependency_enabled_for_features(package, dependency, feature_resolution) {
                continue;
            }
            let dependency_name = dependency_package_name(dependency);
            let Some(dependency_package) = packages_by_name.get(dependency_name.as_str()) else {
                continue;
            };
            let Some(dependency_triples) = build_triples_by_package.get(&dependency_package.name)
            else {
                continue;
            };
            for triple in package_triples {
                if !dependency_triples.contains(triple)
                    || !dependency_applies_to_triple(dependency, triple)?
                {
                    continue;
                }
                for from_target in dependency_package
                    .targets
                    .iter()
                    .filter(|target| target.kind == CargoTargetKind::Lib)
                {
                    let from = rust_unit_id(&dependency_package.name, &from_target.name, triple);
                    for to_target in package.targets.iter().filter(|target| {
                        target.kind == CargoTargetKind::Lib
                            || (package.name == selected_package
                                && target.kind == selected_target_kind
                                && target.name == selected_target)
                    }) {
                        edges.push(CompileUnitEdge {
                            from: from.clone(),
                            to: rust_unit_id(&package.name, &to_target.name, triple),
                            reason: format!(
                                "{:?} dependency {}",
                                dependency.scope, dependency.name
                            ),
                        });
                    }
                }
                for from_target in dependency_package
                    .targets
                    .iter()
                    .filter(|target| target.kind == CargoTargetKind::Lib && target.proc_macro)
                {
                    let from = format!(
                        "{}:proc-macro:{}",
                        dependency_package.name.replace('-', "_"),
                        from_target.name
                    );
                    for to_target in package.targets.iter().filter(|target| {
                        target.kind == CargoTargetKind::Lib
                            || (package.name == selected_package
                                && target.kind == selected_target_kind
                                && target.name == selected_target)
                    }) {
                        edges.push(CompileUnitEdge {
                            from: from.clone(),
                            to: rust_unit_id(&package.name, &to_target.name, triple),
                            reason: format!("proc macro dependency {}", dependency.name),
                        });
                    }
                }
            }
        }
    }

    for triple in triples {
        let root_anchor = selected_package.replace('-', "_");
        let rust_unit = format!("{root_anchor}:rust:{selected_target}:{triple}");
        let link_unit = format!("{root_anchor}:link:{selected_target}:{triple}");
        units.push(CompileUnit {
            id: link_unit.clone(),
            package: selected_package.to_owned(),
            manifest_path: root.manifest_path.clone(),
            target: selected_target.to_owned(),
            source_path: None,
            target_kind: selected_target_kind.clone(),
            phase: CompilePhase::Link,
            triple: triple.clone(),
            profile: profile.to_owned(),
        });
        edges.push(CompileUnitEdge {
            from: rust_unit,
            to: link_unit,
            reason: "final artifact link".to_owned(),
        });
    }

    units.sort_by(|left, right| left.id.cmp(&right.id));
    edges.sort_by(|left, right| {
        left.from
            .cmp(&right.from)
            .then_with(|| left.to.cmp(&right.to))
            .then_with(|| left.reason.cmp(&right.reason))
    });
    Ok(CargoBuildPlan { units, edges })
}

fn build_triples_by_package(
    workspace: &CargoWorkspace,
    feature_resolution: &CargoFeatureResolution,
    selected_package: &str,
    triples: &[String],
) -> Result<BTreeMap<String, BTreeSet<String>>, CargoModelError> {
    let enabled_packages = feature_resolution
        .packages
        .iter()
        .map(|package| package.package.as_str())
        .collect::<BTreeSet<_>>();
    let packages_by_name = workspace
        .packages
        .iter()
        .map(|package| (package.name.as_str(), package))
        .collect::<BTreeMap<_, _>>();
    if !packages_by_name.contains_key(selected_package) {
        return Err(CargoModelError::MissingSelectedPackage(
            selected_package.to_owned(),
        ));
    }

    let mut triples_by_package = BTreeMap::<String, BTreeSet<String>>::new();
    triples_by_package.insert(
        selected_package.to_owned(),
        triples.iter().cloned().collect::<BTreeSet<_>>(),
    );

    let mut changed = true;
    while changed {
        changed = false;
        let frontier = triples_by_package
            .iter()
            .map(|(package, triples)| (package.clone(), triples.clone()))
            .collect::<Vec<_>>();
        for (package_name, package_triples) in frontier {
            let Some(package) = packages_by_name.get(package_name.as_str()) else {
                continue;
            };
            for dependency in package.dependencies.iter().filter(|dependency| {
                dependency.scope == DependencyScope::Normal
                    || dependency.scope == DependencyScope::Build
            }) {
                if !dependency_enabled_for_features(package, dependency, feature_resolution) {
                    continue;
                }
                let Some(dependency_package) =
                    workspace_package_for_dependency(workspace, dependency)
                else {
                    continue;
                };
                if !enabled_packages.contains(dependency_package.name.as_str()) {
                    continue;
                }
                for triple in &package_triples {
                    if dependency_applies_to_triple(dependency, triple)?
                        && triples_by_package
                            .entry(dependency_package.name.clone())
                            .or_default()
                            .insert(triple.clone())
                    {
                        changed = true;
                    }
                }
            }
        }
    }

    Ok(triples_by_package)
}

fn dependency_enabled_for_features(
    package: &CargoPackage,
    dependency: &CargoDependency,
    feature_resolution: &CargoFeatureResolution,
) -> bool {
    !dependency.optional
        || feature_resolution
            .packages
            .iter()
            .find(|resolution| resolution.package == package.name)
            .is_some_and(|resolution| {
                resolution
                    .enabled_optional_dependencies
                    .contains(&dependency.name)
            })
}

fn dependency_applies_to_triple(
    dependency: &CargoDependency,
    triple: &str,
) -> Result<bool, CargoModelError> {
    match dependency.target_cfg.as_deref() {
        Some(cfg) => target_cfg_matches(cfg, triple),
        None => Ok(true),
    }
}

fn target_cfg_matches(cfg: &str, triple: &str) -> Result<bool, CargoModelError> {
    let cfg = cfg.trim();
    if let Some(body) = cfg
        .strip_prefix("cfg(")
        .and_then(|body| body.strip_suffix(')'))
    {
        eval_cfg_expr(body, &TargetFacts::from_triple(triple), triple)
    } else {
        Ok(cfg == triple)
    }
}

#[derive(Debug, Clone)]
struct TargetFacts {
    arch: String,
    os: String,
    vendor: String,
    env: String,
    abi: String,
    pointer_width: String,
    endian: String,
    families: Vec<String>,
    features: Vec<String>,
    atomics: Vec<String>,
}

impl TargetFacts {
    fn from_triple(triple: &str) -> Self {
        let mut parts = triple.split('-').collect::<Vec<_>>();
        while parts.len() < 4 {
            parts.push("");
        }
        let arch = parts[0].to_owned();
        let vendor = parts[1].to_owned();
        let env = parts.last().copied().unwrap_or_default().to_owned();
        let os = if triple == "native_host" {
            "native_host"
        } else if triple.contains("wasip1") || triple.contains("wasip2") {
            "wasi"
        } else if triple.contains("windows") {
            "windows"
        } else if triple.contains("linux") {
            "linux"
        } else if triple.contains("darwin") {
            "macos"
        } else {
            parts.get(2).copied().unwrap_or("unknown")
        }
        .to_owned();
        let mut families = Vec::new();
        if os == "windows" {
            families.push("windows".to_owned());
        } else if os == "linux" || os == "macos" {
            families.push("unix".to_owned());
        } else if os == "wasi" || arch == "wasm32" || arch == "wasm64" {
            families.push("wasm".to_owned());
        } else if os == "native_host" {
            families.push("native".to_owned());
        }
        let pointer_width = if arch.contains("64") {
            "64"
        } else if arch.contains("32") || arch == "arm" {
            "32"
        } else {
            "unknown"
        }
        .to_owned();
        let env = if triple.contains("wasip1") {
            "p1".to_owned()
        } else if triple.contains("wasip2") {
            "p2".to_owned()
        } else {
            env
        };
        Self {
            arch,
            os,
            vendor,
            env,
            abi: String::new(),
            pointer_width,
            endian: "little".to_owned(),
            families,
            features: Vec::new(),
            atomics: target_atomics(triple),
        }
    }

    fn matches_key_value(
        &self,
        key: &str,
        value: &str,
        raw_cfg: &str,
        triple: &str,
    ) -> Result<bool, CargoModelError> {
        Ok(match key {
            "target_arch" => self.arch == value,
            "target_os" => self.os == value,
            "target_vendor" => self.vendor == value,
            "target_env" => self.env == value,
            "target_abi" => self.abi == value,
            "target_feature" => self.features.iter().any(|feature| feature == value),
            "target_has_atomic" => self.atomics.iter().any(|atomic| atomic == value),
            "target_pointer_width" => self.pointer_width == value,
            "target_endian" => self.endian == value,
            "target_family" => self.families.iter().any(|family| family == value),
            _ => {
                return Err(CargoModelError::UnsupportedTargetCfg {
                    cfg: raw_cfg.to_owned(),
                    triple: triple.to_owned(),
                })
            }
        })
    }

    fn matches_bare_atom(
        &self,
        atom: &str,
        raw_cfg: &str,
        triple: &str,
    ) -> Result<bool, CargoModelError> {
        Ok(match atom {
            "unix" => self.families.iter().any(|family| family == "unix"),
            "windows" => self.families.iter().any(|family| family == "windows"),
            _ => {
                return Err(CargoModelError::UnsupportedTargetCfg {
                    cfg: raw_cfg.to_owned(),
                    triple: triple.to_owned(),
                })
            }
        })
    }
}

fn target_atomics(triple: &str) -> Vec<String> {
    let values: &[&str] = if triple.starts_with("x86_64-") || triple.starts_with("aarch64-") {
        &["8", "16", "32", "64", "ptr"]
    } else if triple.starts_with("wasm32-") {
        &["8", "16", "32", "ptr"]
    } else if triple == "native_host" {
        &["ptr"]
    } else {
        &[]
    };
    values.iter().map(|value| (*value).to_owned()).collect()
}

fn eval_cfg_expr(expr: &str, facts: &TargetFacts, triple: &str) -> Result<bool, CargoModelError> {
    let expr = expr.trim();
    if expr.is_empty() {
        return Err(CargoModelError::UnsupportedTargetCfg {
            cfg: expr.to_owned(),
            triple: triple.to_owned(),
        });
    }
    if let Some(inner) = cfg_call_arg(expr, "all") {
        return split_cfg_args(inner)?
            .into_iter()
            .try_fold(true, |matched, arg| {
                Ok(matched && eval_cfg_expr(arg, facts, triple)?)
            });
    }
    if let Some(inner) = cfg_call_arg(expr, "any") {
        let mut matched = false;
        for arg in split_cfg_args(inner)? {
            matched |= eval_cfg_expr(arg, facts, triple)?;
        }
        return Ok(matched);
    }
    if let Some(inner) = cfg_call_arg(expr, "not") {
        let args = split_cfg_args(inner)?;
        if args.len() != 1 {
            return Err(CargoModelError::UnsupportedTargetCfg {
                cfg: expr.to_owned(),
                triple: triple.to_owned(),
            });
        }
        return Ok(!eval_cfg_expr(args[0], facts, triple)?);
    }
    if let Some((key, value)) = split_cfg_key_value(expr)? {
        let value =
            unquote_cfg_value(value).ok_or_else(|| CargoModelError::UnsupportedTargetCfg {
                cfg: expr.to_owned(),
                triple: triple.to_owned(),
            })?;
        return facts.matches_key_value(key.trim(), &value, expr, triple);
    }
    facts.matches_bare_atom(expr, expr, triple)
}

fn cfg_call_arg<'a>(expr: &'a str, name: &str) -> Option<&'a str> {
    expr.strip_prefix(name)
        .and_then(|tail| tail.strip_prefix('('))
        .and_then(|tail| tail.strip_suffix(')'))
}

fn split_cfg_args(input: &str) -> Result<Vec<&str>, CargoModelError> {
    let mut args = Vec::new();
    let mut start = 0usize;
    let mut depth = 0usize;
    let mut in_string = false;
    let mut escaped = false;
    for (idx, ch) in input.char_indices() {
        if in_string {
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == '"' {
                in_string = false;
            }
            continue;
        }
        match ch {
            '"' => in_string = true,
            '(' => depth += 1,
            ')' => {
                depth =
                    depth
                        .checked_sub(1)
                        .ok_or_else(|| CargoModelError::UnsupportedTargetCfg {
                            cfg: input.to_owned(),
                            triple: "*".to_owned(),
                        })?;
            }
            ',' if depth == 0 => {
                let arg = input[start..idx].trim();
                if arg.is_empty() {
                    return Err(CargoModelError::UnsupportedTargetCfg {
                        cfg: input.to_owned(),
                        triple: "*".to_owned(),
                    });
                }
                args.push(arg);
                start = idx + 1;
            }
            _ => {}
        }
    }
    if depth != 0 || in_string {
        return Err(CargoModelError::UnsupportedTargetCfg {
            cfg: input.to_owned(),
            triple: "*".to_owned(),
        });
    }
    let arg = input[start..].trim();
    if !arg.is_empty() {
        args.push(arg);
    }
    Ok(args)
}

fn split_cfg_key_value(input: &str) -> Result<Option<(&str, &str)>, CargoModelError> {
    let mut in_string = false;
    let mut escaped = false;
    for (idx, ch) in input.char_indices() {
        if in_string {
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == '"' {
                in_string = false;
            }
            continue;
        }
        if ch == '"' {
            in_string = true;
        } else if ch == '=' {
            return Ok(Some((&input[..idx], &input[idx + 1..])));
        }
    }
    if in_string {
        return Err(CargoModelError::UnsupportedTargetCfg {
            cfg: input.to_owned(),
            triple: "*".to_owned(),
        });
    }
    Ok(None)
}

fn unquote_cfg_value(value: &str) -> Option<String> {
    let value = value.trim();
    let value = value.strip_prefix('"')?.strip_suffix('"')?;
    Some(value.replace("\\\"", "\"").replace("\\\\", "\\"))
}

fn apply_feature_member(
    workspace: &CargoWorkspace,
    package: &CargoPackage,
    member: &str,
    active_packages: &mut BTreeSet<String>,
    enabled_optional_dependencies: &mut BTreeMap<String, BTreeSet<String>>,
    queue: &mut Vec<(String, String)>,
) -> Result<(), CargoModelError> {
    if let Some(dependency_name) = member.strip_prefix("dep:") {
        enable_optional_dependency(
            workspace,
            package,
            dependency_name,
            active_packages,
            enabled_optional_dependencies,
            queue,
        )?;
        return Ok(());
    }

    if let Some((dependency_name, dependency_feature)) = member.split_once('/') {
        let weak = dependency_name.ends_with('?');
        let dependency_name = dependency_name.trim_end_matches('?');
        if weak
            && !enabled_optional_dependencies
                .get(&package.name)
                .is_some_and(|enabled| enabled.contains(dependency_name))
        {
            return Ok(());
        }
        if let Some(dependency) = package
            .dependencies
            .iter()
            .find(|dependency| dependency.name == dependency_name)
        {
            if dependency.optional {
                enabled_optional_dependencies
                    .entry(package.name.clone())
                    .or_default()
                    .insert(dependency.name.clone());
            }
            if let Some(dependency_package) =
                workspace_package_for_dependency(workspace, dependency)
            {
                active_packages.insert(dependency_package.name.clone());
                queue.push((
                    dependency_package.name.clone(),
                    dependency_feature.to_owned(),
                ));
            }
        }
        return Ok(());
    }

    if package
        .features
        .iter()
        .any(|feature| feature.name == member)
    {
        queue.push((package.name.clone(), member.to_owned()));
        return Ok(());
    }

    if !enable_optional_dependency(
        workspace,
        package,
        member,
        active_packages,
        enabled_optional_dependencies,
        queue,
    )? {
        return Err(CargoModelError::MissingFeature {
            package: package.name.clone(),
            feature: member.to_owned(),
        });
    }
    Ok(())
}

fn enable_optional_dependency(
    workspace: &CargoWorkspace,
    package: &CargoPackage,
    dependency_name: &str,
    active_packages: &mut BTreeSet<String>,
    enabled_optional_dependencies: &mut BTreeMap<String, BTreeSet<String>>,
    queue: &mut Vec<(String, String)>,
) -> Result<bool, CargoModelError> {
    let Some(dependency) = package
        .dependencies
        .iter()
        .find(|dependency| dependency.name == dependency_name)
    else {
        return Ok(false);
    };
    if !dependency.optional {
        return Ok(false);
    }

    let inserted = enabled_optional_dependencies
        .entry(package.name.clone())
        .or_default()
        .insert(dependency.name.clone());
    activate_dependency_package(workspace, dependency, active_packages, queue)?;
    Ok(inserted)
}

fn activate_dependency_package(
    workspace: &CargoWorkspace,
    dependency: &CargoDependency,
    active_packages: &mut BTreeSet<String>,
    queue: &mut Vec<(String, String)>,
) -> Result<bool, CargoModelError> {
    let Some(dependency_package) = workspace_package_for_dependency(workspace, dependency) else {
        return Ok(false);
    };
    let changed = active_packages.insert(dependency_package.name.clone());
    if dependency.default_features {
        queue.push((dependency_package.name.clone(), "default".to_owned()));
    }
    for feature in &dependency.features {
        queue.push((dependency_package.name.clone(), feature.clone()));
    }
    Ok(changed)
}

fn workspace_package_for_dependency<'a>(
    workspace: &'a CargoWorkspace,
    dependency: &CargoDependency,
) -> Option<&'a CargoPackage> {
    let package_name = dependency_package_name(dependency);
    workspace
        .packages
        .iter()
        .find(|package| package.name == package_name)
}

fn dependency_package_name(dependency: &CargoDependency) -> String {
    dependency
        .package
        .clone()
        .unwrap_or_else(|| dependency.name.clone())
}

fn rust_unit_id(package: &str, target: &str, triple: &str) -> String {
    format!("{}:rust:{target}:{triple}", package.replace('-', "_"))
}

pub fn parse_lockfile(
    storage: &dyn Storage,
    lockfile_path: &str,
) -> Result<CargoLockfile, CargoModelError> {
    let path = normalize_path(lockfile_path)?;
    let bytes = storage.read(&path)?;
    let text = String::from_utf8_lossy(&bytes);
    let value: toml::Value = toml::from_str(&text).map_err(|source| CargoModelError::Toml {
        path: path.clone(),
        source,
    })?;
    let version = value.get("version").and_then(toml::Value::as_integer);
    let mut packages = Vec::new();
    if let Some(array) = value.get("package").and_then(toml::Value::as_array) {
        for item in array.iter().filter_map(toml::Value::as_table) {
            let name = string_field(item, "name")
                .ok_or_else(|| CargoModelError::MissingPackageName(path.clone()))?;
            let package_version = string_field(item, "version").unwrap_or_default();
            packages.push(CargoLockedPackage {
                name,
                version: package_version,
                source: string_field(item, "source"),
                checksum: string_field(item, "checksum"),
                dependencies: string_array_field(item, "dependencies"),
            });
        }
    }
    packages.sort_by(|left, right| {
        left.name
            .cmp(&right.name)
            .then_with(|| left.version.cmp(&right.version))
            .then_with(|| left.source.cmp(&right.source))
    });
    Ok(CargoLockfile {
        path,
        version,
        packages,
    })
}

pub fn plan_source_fetches(workspace: &CargoWorkspace) -> CargoSourceFetchPlan {
    let mut entries = Vec::new();
    for package in &workspace.packages {
        for dep in &package.dependencies {
            let (kind, locator) = if let Some(path) = &dep.path {
                (CargoSourceKind::Path, path.clone())
            } else if let Some(git) = &dep.git {
                (CargoSourceKind::Git, git.clone())
            } else {
                (
                    CargoSourceKind::Registry,
                    dep.registry
                        .clone()
                        .unwrap_or_else(|| "crates-io".to_owned()),
                )
            };
            entries.push(CargoSourceFetch {
                package: package.name.clone(),
                dependency: dep.package.clone().unwrap_or_else(|| dep.name.clone()),
                scope: dep.scope.clone(),
                kind,
                locator,
                requirement: dep.requirement.clone(),
                target_cfg: dep.target_cfg.clone(),
            });
        }
    }
    entries.sort();
    entries.dedup();
    CargoSourceFetchPlan { entries }
}

pub fn validate_lockfile_against_fetch_plan(
    lockfile: &CargoLockfile,
    source_fetch_plan: &CargoSourceFetchPlan,
) -> Result<(), CargoModelError> {
    for entry in &source_fetch_plan.entries {
        if entry.kind == CargoSourceKind::Path {
            continue;
        }
        let Some(locked) = lockfile
            .packages
            .iter()
            .find(|package| package.name == entry.dependency)
        else {
            return Err(CargoModelError::MissingLockedDependency {
                dependency: entry.dependency.clone(),
                locator: entry.locator.clone(),
            });
        };
        match entry.kind {
            CargoSourceKind::Registry => {
                let Some(source) = locked.source.as_deref() else {
                    return Err(CargoModelError::LockedSourceMismatch {
                        dependency: entry.dependency.clone(),
                        expected: entry.locator.clone(),
                    });
                };
                if !source.starts_with("registry+")
                    || (entry.locator != "crates-io" && !source.contains(&entry.locator))
                {
                    return Err(CargoModelError::LockedSourceMismatch {
                        dependency: entry.dependency.clone(),
                        expected: entry.locator.clone(),
                    });
                }
                if locked
                    .checksum
                    .as_deref()
                    .is_none_or(|checksum| checksum.is_empty())
                {
                    return Err(CargoModelError::MissingRegistryChecksum(
                        entry.dependency.clone(),
                    ));
                }
            }
            CargoSourceKind::Git => {
                let Some(source) = locked.source.as_deref() else {
                    return Err(CargoModelError::LockedSourceMismatch {
                        dependency: entry.dependency.clone(),
                        expected: entry.locator.clone(),
                    });
                };
                if !source.starts_with("git+") || !source.contains(&entry.locator) {
                    return Err(CargoModelError::LockedSourceMismatch {
                        dependency: entry.dependency.clone(),
                        expected: entry.locator.clone(),
                    });
                }
            }
            CargoSourceKind::Path => {}
        }
    }
    Ok(())
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
    let value = read_manifest_value(storage, manifest_path)?;
    for member in workspace_members(storage, manifest_path)? {
        load_package_closure(storage, &member, seen, packages)?;
    }
    if value
        .get("package")
        .and_then(toml::Value::as_table)
        .is_some()
    {
        let package = package_from_value(storage, manifest_path, value)?;
        for dep in package
            .dependencies
            .iter()
            .filter_map(|dep| dep.path.as_deref())
        {
            let dep_manifest = join_path(dep, "Cargo.toml")?;
            load_package_closure(storage, &dep_manifest, seen, packages)?;
        }
        packages.push(package);
    }
    Ok(())
}

pub fn parse_manifest(
    storage: &dyn Storage,
    manifest_path: &str,
) -> Result<CargoPackage, CargoModelError> {
    let manifest_path = normalize_path(manifest_path)?;
    let value = read_manifest_value(storage, &manifest_path)?;
    package_from_value(storage, &manifest_path, value)
}

fn read_manifest_value(
    storage: &dyn Storage,
    manifest_path: &str,
) -> Result<toml::Value, CargoModelError> {
    let manifest_path = normalize_path(manifest_path)?;
    let bytes = storage.read(&manifest_path)?;
    let text = String::from_utf8_lossy(&bytes);
    toml::from_str(&text).map_err(|source| CargoModelError::Toml {
        path: manifest_path,
        source,
    })
}

fn package_from_value(
    storage: &dyn Storage,
    manifest_path: &str,
    value: toml::Value,
) -> Result<CargoPackage, CargoModelError> {
    let package_table = value
        .get("package")
        .and_then(toml::Value::as_table)
        .ok_or_else(|| CargoModelError::MissingPackage(manifest_path.to_owned()))?;
    let name = string_field(package_table, "name")
        .ok_or_else(|| CargoModelError::MissingPackageName(manifest_path.to_owned()))?;
    let version = string_field(package_table, "version");
    let edition = string_field(package_table, "edition");
    let base = manifest_dir(manifest_path);
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
        manifest_path: manifest_path.to_owned(),
        build_script: build_script_path(storage, package_table, &base)?,
        targets: collect_targets(storage, &name, &value, &base)?,
        dependencies,
        features: collect_features(&value),
    })
}

fn workspace_members(
    storage: &dyn Storage,
    manifest_path: &str,
) -> Result<Vec<String>, CargoModelError> {
    let value = read_manifest_value(storage, manifest_path)?;
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

fn collect_targets(
    storage: &dyn Storage,
    package_name: &str,
    value: &toml::Value,
    base: &str,
) -> Result<Vec<CargoTarget>, CargoModelError> {
    let mut targets = Vec::new();
    if value.get("lib").is_some() {
        let lib_table = value.get("lib").and_then(toml::Value::as_table);
        let lib_name = value
            .get("lib")
            .and_then(|lib| lib.get("name"))
            .and_then(toml::Value::as_str)
            .unwrap_or(package_name)
            .to_owned();
        let path = match value
            .get("lib")
            .and_then(|lib| lib.get("path"))
            .and_then(toml::Value::as_str)
        {
            Some(path) => Some(join_path(base, path)?),
            None => Some(join_path(base, "src/lib.rs")?),
        };
        let proc_macro = lib_table
            .and_then(|lib| lib.get("proc-macro"))
            .and_then(toml::Value::as_bool)
            .unwrap_or(false);
        targets.push(CargoTarget {
            name: lib_name,
            kind: CargoTargetKind::Lib,
            path,
            proc_macro,
        });
    } else if package_auto_target_enabled(value, "autolib")
        && storage_has_file(storage, &join_path(base, "src/lib.rs")?)?
    {
        targets.push(CargoTarget {
            name: package_name.to_owned(),
            kind: CargoTargetKind::Lib,
            path: Some(join_path(base, "src/lib.rs")?),
            proc_macro: false,
        });
    }
    let explicit_bins = targets.len();
    collect_array_targets(
        storage,
        value,
        "bin",
        CargoTargetKind::Bin,
        package_name,
        base,
        &mut targets,
    )?;
    if targets.len() == explicit_bins
        && package_auto_target_enabled(value, "autobins")
        && storage_has_file(storage, &join_path(base, "src/main.rs")?)?
    {
        targets.push(CargoTarget {
            name: package_name.to_owned(),
            kind: CargoTargetKind::Bin,
            path: Some(join_path(base, "src/main.rs")?),
            proc_macro: false,
        });
    }
    collect_array_targets(
        storage,
        value,
        "example",
        CargoTargetKind::Example,
        package_name,
        base,
        &mut targets,
    )?;
    if targets.is_empty() {
        targets.push(CargoTarget {
            name: package_name.to_owned(),
            kind: CargoTargetKind::Bin,
            path: Some(join_path(base, "src/main.rs")?),
            proc_macro: false,
        });
    }
    Ok(targets)
}

fn package_auto_target_enabled(value: &toml::Value, key: &str) -> bool {
    value
        .get("package")
        .and_then(|package| package.get(key))
        .and_then(toml::Value::as_bool)
        .unwrap_or(true)
}

fn storage_has_file(storage: &dyn Storage, path: &str) -> Result<bool, CargoModelError> {
    match storage.read(path) {
        Ok(_) => Ok(true),
        Err(VfsError::NotFound(_)) => Ok(false),
        Err(err) => Err(CargoModelError::Vfs(err)),
    }
}

fn collect_array_targets(
    storage: &dyn Storage,
    value: &toml::Value,
    key: &str,
    kind: CargoTargetKind,
    fallback_name: &str,
    base: &str,
    targets: &mut Vec<CargoTarget>,
) -> Result<(), CargoModelError> {
    if let Some(array) = value.get(key).and_then(toml::Value::as_array) {
        for item in array.iter().filter_map(toml::Value::as_table) {
            let name = string_field(item, "name").unwrap_or_else(|| fallback_name.to_owned());
            let path = match string_field(item, "path") {
                Some(path) => Some(join_path(base, &path)?),
                None => inferred_explicit_target_path(storage, base, &kind, &name, fallback_name)?,
            };
            targets.push(CargoTarget {
                name,
                kind: kind.clone(),
                path,
                proc_macro: false,
            });
        }
    }
    Ok(())
}

fn inferred_explicit_target_path(
    storage: &dyn Storage,
    base: &str,
    kind: &CargoTargetKind,
    name: &str,
    fallback_name: &str,
) -> Result<Option<String>, CargoModelError> {
    let candidates = match kind {
        CargoTargetKind::Bin => {
            let mut candidates = Vec::new();
            if name == fallback_name {
                candidates.push(join_path(base, "src/main.rs")?);
            }
            candidates.push(join_path(base, &format!("src/bin/{name}.rs"))?);
            candidates.push(join_path(base, &format!("src/bin/{name}/main.rs"))?);
            candidates
        }
        CargoTargetKind::Example => vec![
            join_path(base, &format!("examples/{name}.rs"))?,
            join_path(base, &format!("examples/{name}/main.rs"))?,
        ],
        CargoTargetKind::Lib | CargoTargetKind::Test | CargoTargetKind::Bench => Vec::new(),
    };
    for candidate in &candidates {
        if storage_has_file(storage, candidate)? {
            return Ok(Some(candidate.clone()));
        }
    }
    Ok(candidates.into_iter().next())
}

fn collect_features(value: &toml::Value) -> Vec<CargoFeature> {
    let mut features = Vec::new();
    let Some(table) = value.get("features").and_then(toml::Value::as_table) else {
        return features;
    };
    for (name, value) in table {
        features.push(CargoFeature {
            name: name.clone(),
            members: value
                .as_array()
                .into_iter()
                .flatten()
                .filter_map(toml::Value::as_str)
                .map(str::to_owned)
                .collect(),
        });
    }
    features.sort_by(|left, right| left.name.cmp(&right.name));
    features
}

fn build_script_path(
    storage: &dyn Storage,
    package_table: &toml::map::Map<String, toml::Value>,
    base: &str,
) -> Result<Option<String>, CargoModelError> {
    match package_table.get("build") {
        Some(value) if value.as_bool() == Some(false) => Ok(None),
        Some(value) if value.as_str().is_some() => {
            Ok(Some(join_path(base, value.as_str().unwrap())?))
        }
        Some(_) => Err(CargoModelError::InvalidDependency {
            package: string_field(package_table, "name").unwrap_or_default(),
            dependency: "package.build".to_owned(),
        }),
        None => {
            let default_path = join_path(base, "build.rs")?;
            match storage.read(&default_path) {
                Ok(_) => Ok(Some(default_path)),
                Err(VfsError::NotFound(_)) => Ok(None),
                Err(err) => Err(CargoModelError::Vfs(err)),
            }
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
            default_features: true,
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
        default_features: table
            .get("default-features")
            .and_then(toml::Value::as_bool)
            .unwrap_or(true),
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

    #[test]
    fn resolves_virtual_workspace_members_without_requiring_root_package() {
        let mut storage = MemoryStorage::new();
        storage
            .write(
                "Cargo.toml",
                br#"
[workspace]
members = ["crates/app", "crates/macro-crate"]
resolver = "2"
"#,
            )
            .unwrap();
        storage
            .write(
                "crates/app/Cargo.toml",
                br#"
[package]
name = "app"
version = "0.1.0"
edition = "2021"
build = "build/build.rs"

[features]
default = ["derive"]
derive = ["macro-crate"]

[dependencies]
macro-crate = { path = "../macro-crate", optional = true }
"#,
            )
            .unwrap();
        storage
            .write("crates/app/build/build.rs", b"fn main() {}\n")
            .unwrap();
        storage
            .write(
                "crates/macro-crate/Cargo.toml",
                br#"
[package]
name = "macro-crate"
version = "0.1.0"
edition = "2021"

[lib]
proc-macro = true
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
            vec!["app", "macro-crate"]
        );
        let app = &workspace.packages[0];
        assert_eq!(
            app.build_script.as_deref(),
            Some("crates/app/build/build.rs")
        );
        assert!(app.targets.iter().any(|target| {
            target.name == "app" && target.path.as_deref() == Some("crates/app/src/main.rs")
        }));
        assert_eq!(app.features[0].name, "default");
        assert!(workspace.packages[1]
            .targets
            .iter()
            .any(|target| target.proc_macro
                && target.path.as_deref() == Some("crates/macro-crate/src/lib.rs")));
    }

    #[test]
    fn discovers_default_lib_and_bin_targets_from_source_layout_without_host_cargo() {
        let mut storage = MemoryStorage::new();
        storage
            .write(
                "Cargo.toml",
                br#"
[package]
name = "app"
version = "0.1.0"
edition = "2021"
"#,
            )
            .unwrap();
        storage.write("src/lib.rs", b"pub fn lib() {}\n").unwrap();
        storage.write("src/main.rs", b"fn main() {}\n").unwrap();

        let package = parse_manifest(&storage, "Cargo.toml").unwrap();

        assert!(package.targets.iter().any(|target| {
            target.kind == CargoTargetKind::Lib
                && target.name == "app"
                && target.path.as_deref() == Some("src/lib.rs")
        }));
        assert!(package.targets.iter().any(|target| {
            target.kind == CargoTargetKind::Bin
                && target.name == "app"
                && target.path.as_deref() == Some("src/main.rs")
        }));
    }

    #[test]
    fn resolves_default_requested_and_dependency_features_without_host_cargo() {
        let mut storage = MemoryStorage::new();
        storage
            .write(
                "Cargo.toml",
                br#"
[workspace]
members = ["app", "helper", "macro-crate"]
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

[features]
default = ["dep:helper", "macro-crate/derive"]
extra = ["helper/std"]

[dependencies]
helper = { path = "../helper", optional = true, default-features = false, features = ["alloc"] }
macro-crate = { path = "../macro-crate" }
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

[features]
alloc = []
std = []
"#,
            )
            .unwrap();
        storage
            .write(
                "macro-crate/Cargo.toml",
                br#"
[package]
name = "macro-crate"
version = "0.1.0"
edition = "2021"

[lib]
proc-macro = true

[features]
derive = []
"#,
            )
            .unwrap();
        let workspace = resolve_workspace(&storage, "Cargo.toml").unwrap();

        let resolution = resolve_features(&workspace, "app", true, &["extra".to_owned()]).unwrap();

        let app = resolution
            .packages
            .iter()
            .find(|package| package.package == "app")
            .unwrap();
        assert_eq!(app.enabled_optional_dependencies, vec!["helper".to_owned()]);
        let helper = resolution
            .packages
            .iter()
            .find(|package| package.package == "helper")
            .unwrap();
        assert_eq!(
            helper.enabled_features,
            vec!["alloc".to_owned(), "std".to_owned()]
        );
        let macro_crate = resolution
            .packages
            .iter()
            .find(|package| package.package == "macro-crate")
            .unwrap();
        assert_eq!(macro_crate.enabled_features, vec!["derive".to_owned()]);
    }

    #[test]
    fn parses_lockfile_packages_without_host_cargo() {
        let mut storage = MemoryStorage::new();
        storage
            .write(
                "Cargo.lock",
                br#"
version = 4

[[package]]
name = "app"
version = "0.1.0"
dependencies = [
 "serde",
]

[[package]]
name = "serde"
version = "1.0.228"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "abc123"
"#,
            )
            .unwrap();

        let lockfile = parse_lockfile(&storage, "Cargo.lock").unwrap();

        assert_eq!(lockfile.version, Some(4));
        assert_eq!(lockfile.packages[0].name, "app");
        assert_eq!(lockfile.packages[1].checksum.as_deref(), Some("abc123"));
    }

    #[test]
    fn plans_build_script_proc_macro_rust_and_link_units() {
        let workspace = CargoWorkspace {
            root_manifest_path: "Cargo.toml".to_owned(),
            packages: vec![
                CargoPackage {
                    name: "macro-crate".to_owned(),
                    version: Some("0.1.0".to_owned()),
                    edition: Some("2021".to_owned()),
                    manifest_path: "macro-crate/Cargo.toml".to_owned(),
                    build_script: None,
                    targets: vec![CargoTarget {
                        name: "macro_crate".to_owned(),
                        kind: CargoTargetKind::Lib,
                        path: None,
                        proc_macro: true,
                    }],
                    dependencies: Vec::new(),
                    features: Vec::new(),
                },
                CargoPackage {
                    name: "app".to_owned(),
                    version: Some("0.1.0".to_owned()),
                    edition: Some("2021".to_owned()),
                    manifest_path: "app/Cargo.toml".to_owned(),
                    build_script: Some("app/build.rs".to_owned()),
                    targets: vec![CargoTarget {
                        name: "app".to_owned(),
                        kind: CargoTargetKind::Bin,
                        path: Some("src/main.rs".to_owned()),
                        proc_macro: false,
                    }],
                    dependencies: vec![CargoDependency {
                        name: "macro-crate".to_owned(),
                        package: None,
                        requirement: None,
                        path: Some("macro-crate".to_owned()),
                        git: None,
                        registry: None,
                        optional: false,
                        default_features: true,
                        features: Vec::new(),
                        scope: DependencyScope::Normal,
                        target_cfg: None,
                    }],
                    features: Vec::new(),
                },
            ],
        };
        let feature_resolution = resolve_features(&workspace, "app", true, &[]).unwrap();

        let plan = plan_build(
            &workspace,
            &feature_resolution,
            "app",
            "app",
            CargoTargetKind::Bin,
            "release",
            &["wasm32-wasip1".to_owned(), "native_host".to_owned()],
        )
        .unwrap();

        assert!(plan
            .units
            .iter()
            .any(|unit| unit.phase == CompilePhase::BuildScript));
        assert!(plan.units.iter().any(|unit| {
            unit.phase == CompilePhase::Rust
                && unit.package == "app"
                && unit.source_path.as_deref() == Some("src/main.rs")
        }));
        assert!(plan
            .units
            .iter()
            .any(|unit| unit.phase == CompilePhase::ProcMacro));
        assert!(plan
            .units
            .iter()
            .any(|unit| unit.phase == CompilePhase::Link && unit.triple == "wasm32-wasip1"));
        assert!(plan
            .edges
            .iter()
            .any(|edge| edge.reason == "final artifact link"));
        assert!(plan
            .edges
            .iter()
            .any(|edge| edge.reason == "proc macro dependency macro-crate"));
    }

    #[test]
    fn normalizes_explicit_target_paths_relative_to_package_manifest() {
        let mut storage = MemoryStorage::new();
        storage
            .write(
                "crates/app/Cargo.toml",
                br#"
[package]
name = "app"
version = "0.1.0"
edition = "2021"

[[bin]]
name = "tool"
path = "tools/tool.rs"

[[example]]
name = "demo"
"#,
            )
            .unwrap();
        storage
            .write("crates/app/tools/tool.rs", b"fn main() {}\n")
            .unwrap();
        storage
            .write("crates/app/examples/demo.rs", b"fn main() {}\n")
            .unwrap();

        let package = parse_manifest(&storage, "crates/app/Cargo.toml").unwrap();

        assert!(package.targets.iter().any(|target| {
            target.kind == CargoTargetKind::Bin
                && target.name == "tool"
                && target.path.as_deref() == Some("crates/app/tools/tool.rs")
        }));
        assert!(package.targets.iter().any(|target| {
            target.kind == CargoTargetKind::Example
                && target.name == "demo"
                && target.path.as_deref() == Some("crates/app/examples/demo.rs")
        }));
    }

    #[test]
    fn plans_target_specific_dependencies_only_for_matching_triples() {
        let workspace = CargoWorkspace {
            root_manifest_path: "Cargo.toml".to_owned(),
            packages: vec![
                CargoPackage {
                    name: "app".to_owned(),
                    version: Some("0.1.0".to_owned()),
                    edition: Some("2021".to_owned()),
                    manifest_path: "app/Cargo.toml".to_owned(),
                    build_script: None,
                    targets: vec![CargoTarget {
                        name: "app".to_owned(),
                        kind: CargoTargetKind::Bin,
                        path: Some("src/main.rs".to_owned()),
                        proc_macro: false,
                    }],
                    dependencies: vec![
                        dep_with_cfg("linux-helper", "cfg(all(unix, not(target_os = \"macos\")))"),
                        dep_with_cfg("wasi-helper", "cfg(target_os = \"wasi\")"),
                        dep_with_cfg("windows-helper", "cfg(windows)"),
                        dep_with_cfg("exact-helper", "wasm32-wasip1"),
                        dep_with_cfg("atomic-helper", "cfg(target_has_atomic = \"ptr\")"),
                    ],
                    features: Vec::new(),
                },
                lib_package("linux-helper"),
                lib_package("wasi-helper"),
                lib_package("windows-helper"),
                lib_package("exact-helper"),
                lib_package("atomic-helper"),
            ],
        };
        let feature_resolution = resolve_features(&workspace, "app", true, &[]).unwrap();

        let plan = plan_build(
            &workspace,
            &feature_resolution,
            "app",
            "app",
            CargoTargetKind::Bin,
            "release",
            &[
                "wasm32-wasip1".to_owned(),
                "x86_64-unknown-linux-gnu".to_owned(),
            ],
        )
        .unwrap();

        assert!(plan
            .units
            .iter()
            .any(|unit| unit.id == "linux_helper:rust:linux_helper:x86_64-unknown-linux-gnu"));
        assert!(!plan
            .units
            .iter()
            .any(|unit| unit.id == "linux_helper:rust:linux_helper:wasm32-wasip1"));
        assert!(plan
            .units
            .iter()
            .any(|unit| unit.id == "wasi_helper:rust:wasi_helper:wasm32-wasip1"));
        assert!(!plan
            .units
            .iter()
            .any(|unit| unit.package == "windows-helper"));
        assert!(plan
            .edges
            .iter()
            .any(|edge| edge.reason == "Normal dependency exact-helper"
                && edge.from == "exact_helper:rust:exact_helper:wasm32-wasip1"));
        assert!(plan
            .units
            .iter()
            .any(|unit| unit.id == "atomic_helper:rust:atomic_helper:wasm32-wasip1"));
    }

    #[test]
    fn target_cfg_evaluator_rejects_unknown_cfg_atoms() {
        let dep = CargoDependency {
            name: "helper".to_owned(),
            package: None,
            requirement: None,
            path: Some("helper".to_owned()),
            git: None,
            registry: None,
            optional: false,
            default_features: true,
            features: Vec::new(),
            scope: DependencyScope::Normal,
            target_cfg: Some("cfg(target_os_version = \"15\")".to_owned()),
        };

        let err = dependency_applies_to_triple(&dep, "wasm32-wasip1").unwrap_err();

        assert!(err.to_string().contains("unsupported target cfg"));
    }

    fn lib_package(name: &str) -> CargoPackage {
        CargoPackage {
            name: name.to_owned(),
            version: Some("0.1.0".to_owned()),
            edition: Some("2021".to_owned()),
            manifest_path: format!("{name}/Cargo.toml"),
            build_script: None,
            targets: vec![CargoTarget {
                name: name.replace('-', "_"),
                kind: CargoTargetKind::Lib,
                path: Some("src/lib.rs".to_owned()),
                proc_macro: false,
            }],
            dependencies: Vec::new(),
            features: Vec::new(),
        }
    }

    fn dep_with_cfg(name: &str, cfg: &str) -> CargoDependency {
        CargoDependency {
            name: name.to_owned(),
            package: None,
            requirement: None,
            path: Some(name.to_owned()),
            git: None,
            registry: None,
            optional: false,
            default_features: true,
            features: Vec::new(),
            scope: DependencyScope::Normal,
            target_cfg: Some(cfg.to_owned()),
        }
    }

    #[test]
    fn plans_dependency_sources_without_host_cargo() {
        let workspace = CargoWorkspace {
            root_manifest_path: "Cargo.toml".to_owned(),
            packages: vec![CargoPackage {
                name: "app".to_owned(),
                version: Some("0.1.0".to_owned()),
                edition: Some("2021".to_owned()),
                manifest_path: "Cargo.toml".to_owned(),
                build_script: None,
                targets: Vec::new(),
                dependencies: vec![
                    CargoDependency {
                        name: "helper".to_owned(),
                        package: None,
                        requirement: None,
                        path: Some("helper".to_owned()),
                        git: None,
                        registry: None,
                        optional: false,
                        default_features: true,
                        features: Vec::new(),
                        scope: DependencyScope::Normal,
                        target_cfg: None,
                    },
                    CargoDependency {
                        name: "serde".to_owned(),
                        package: None,
                        requirement: Some("1".to_owned()),
                        path: None,
                        git: None,
                        registry: None,
                        optional: false,
                        default_features: true,
                        features: Vec::new(),
                        scope: DependencyScope::Normal,
                        target_cfg: None,
                    },
                    CargoDependency {
                        name: "tool".to_owned(),
                        package: Some("actual-tool".to_owned()),
                        requirement: None,
                        path: None,
                        git: Some("https://example.invalid/tool.git".to_owned()),
                        registry: None,
                        optional: false,
                        default_features: true,
                        features: Vec::new(),
                        scope: DependencyScope::Build,
                        target_cfg: Some("cfg(unix)".to_owned()),
                    },
                ],
                features: Vec::new(),
            }],
        };

        let plan = plan_source_fetches(&workspace);

        assert_eq!(plan.entries.len(), 3);
        assert!(plan
            .entries
            .iter()
            .any(|entry| entry.dependency == "helper" && entry.kind == CargoSourceKind::Path));
        assert!(plan
            .entries
            .iter()
            .any(|entry| entry.dependency == "serde" && entry.kind == CargoSourceKind::Registry));
        assert!(plan
            .entries
            .iter()
            .any(|entry| entry.dependency == "actual-tool"
                && entry.kind == CargoSourceKind::Git
                && entry.target_cfg.as_deref() == Some("cfg(unix)")));
    }

    #[test]
    fn validates_registry_and_git_dependencies_are_locked_for_frozen_resolution() {
        let source_fetch_plan = CargoSourceFetchPlan {
            entries: vec![
                CargoSourceFetch {
                    package: "app".to_owned(),
                    dependency: "serde".to_owned(),
                    scope: DependencyScope::Normal,
                    kind: CargoSourceKind::Registry,
                    locator: "crates-io".to_owned(),
                    requirement: Some("1".to_owned()),
                    target_cfg: None,
                },
                CargoSourceFetch {
                    package: "app".to_owned(),
                    dependency: "tool".to_owned(),
                    scope: DependencyScope::Build,
                    kind: CargoSourceKind::Git,
                    locator: "https://example.invalid/tool.git".to_owned(),
                    requirement: None,
                    target_cfg: None,
                },
                CargoSourceFetch {
                    package: "app".to_owned(),
                    dependency: "local-helper".to_owned(),
                    scope: DependencyScope::Normal,
                    kind: CargoSourceKind::Path,
                    locator: "local-helper".to_owned(),
                    requirement: None,
                    target_cfg: None,
                },
            ],
        };
        let lockfile = CargoLockfile {
            path: "Cargo.lock".to_owned(),
            version: Some(4),
            packages: vec![
                CargoLockedPackage {
                    name: "serde".to_owned(),
                    version: "1.0.0".to_owned(),
                    source: Some(
                        "registry+https://github.com/rust-lang/crates.io-index".to_owned(),
                    ),
                    checksum: Some("abc123".to_owned()),
                    dependencies: Vec::new(),
                },
                CargoLockedPackage {
                    name: "tool".to_owned(),
                    version: "0.1.0".to_owned(),
                    source: Some("git+https://example.invalid/tool.git#abc123".to_owned()),
                    checksum: None,
                    dependencies: Vec::new(),
                },
            ],
        };

        validate_lockfile_against_fetch_plan(&lockfile, &source_fetch_plan).unwrap();
    }

    #[test]
    fn rejects_unlocked_registry_dependencies_for_frozen_resolution() {
        let source_fetch_plan = CargoSourceFetchPlan {
            entries: vec![CargoSourceFetch {
                package: "app".to_owned(),
                dependency: "serde".to_owned(),
                scope: DependencyScope::Normal,
                kind: CargoSourceKind::Registry,
                locator: "crates-io".to_owned(),
                requirement: Some("1".to_owned()),
                target_cfg: None,
            }],
        };
        let missing = CargoLockfile {
            path: "Cargo.lock".to_owned(),
            version: Some(4),
            packages: Vec::new(),
        };
        let without_checksum = CargoLockfile {
            path: "Cargo.lock".to_owned(),
            version: Some(4),
            packages: vec![CargoLockedPackage {
                name: "serde".to_owned(),
                version: "1.0.0".to_owned(),
                source: Some("registry+https://github.com/rust-lang/crates.io-index".to_owned()),
                checksum: None,
                dependencies: Vec::new(),
            }],
        };

        assert!(matches!(
            validate_lockfile_against_fetch_plan(&missing, &source_fetch_plan),
            Err(CargoModelError::MissingLockedDependency { .. })
        ));
        assert!(matches!(
            validate_lockfile_against_fetch_plan(&without_checksum, &source_fetch_plan),
            Err(CargoModelError::MissingRegistryChecksum(_))
        ));
    }
}
