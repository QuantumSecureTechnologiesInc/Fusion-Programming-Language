//! Forge Build System & Package Manager
//! Addresses: No package manager, No build caching, No dependency graph.
//! Integrates the core Fusion Runtime Triad: Supernova, Nebula, Pulsar.
use crate::types::*;

use crate::codegen::CodegenConfig;
use sha2::{Digest, Sha256};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

/// Represents the target runtime execution environment for the Fusion manifest.
#[derive(Clone, Debug, PartialEq)]
pub enum RuntimeTarget {
    /// Legacy, synchronous, deterministic execution kernel (v2.0)
    Nebula,
    /// High-performance, asynchronous, PQC-hardened, hardware-aware kernel (v3.0)
    Supernova,
    /// Ultra-lightweight, zero-dependency embedded/WASM target
    Pulsar,
}

/// Represents a package dependency in the dependency graph.
#[derive(Clone, Debug)]
pub struct Dependency {
    pub name: FString,
    pub version: FString,
    pub path: Option<FString>,
    pub registry: Option<FString>,
    pub features: Vec<FString>,
    pub optional: bool,
}

/// Resolved dependency with exact version and source path.
#[derive(Clone, Debug)]
pub struct ResolvedDependency {
    pub name: FString,
    pub version: FString,
    pub source: DependencySource,
    pub checksum: String,
}

/// Where a dependency is sourced from.
#[derive(Clone, Debug)]
pub enum DependencySource {
    /// Local path dependency
    Local(PathBuf),
    /// Registry package (URL + version)
    Registry { url: String, version: String },
    /// Git dependency
    Git { url: String, rev: String },
}

/// Entry in the lock file.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct LockEntry {
    pub name: String,
    pub version: String,
    pub source: String,
    pub checksum: String,
}

/// The lock file format.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct LockFile {
    pub version: String,
    pub packages: Vec<LockEntry>,
}

impl Default for LockFile {
    fn default() -> Self {
        Self {
            version: "1".to_string(),
            packages: Vec::new(),
        }
    }
}

/// Represents a parsed `fusion.toml` manifest file.
#[derive(Clone, Debug)]
pub struct ProjectManifest {
    pub name: FString,
    pub version: FString,
    pub authors: FVec<FString>,
    pub description: Option<FString>,
    pub license: Option<FString>,
    pub repository: Option<FString>,
    pub runtime: RuntimeTarget,
    pub dependencies: FVec<Dependency>,
    pub dev_dependencies: FVec<Dependency>,
    pub build_cache_enabled: bool,
    pub registry_url: FString,
}

pub struct Forge {
    pub manifest: ProjectManifest,
    pub cache_dir: FString,
    pub project_root: PathBuf,
}

impl Forge {
    /// Initializes the Forge build system for a given project directory.
    /// Parses `Fusion.toml` from the project root if it exists.
    pub fn new(project_root: &str) -> Result<Self, FString> {
        let root = PathBuf::from(project_root);
        let manifest_path = root.join("Fusion.toml");

        let manifest = if manifest_path.exists() {
            Self::parse_manifest(&manifest_path)?
        } else {
            ProjectManifest {
                name: "fusion_project".to_string(),
                version: "0.1.0".to_string(),
                authors: vec![],
                description: None,
                license: None,
                repository: None,
                runtime: RuntimeTarget::Supernova,
                dependencies: vec![],
                dev_dependencies: vec![],
                build_cache_enabled: true,
                registry_url: "https://registry.fusionlang.dev".to_string(),
            }
        };

        Ok(Self {
            manifest,
            cache_dir: format!("{}/.fusion_cache", project_root),
            project_root: root,
        })
    }

    /// Parses a `Fusion.toml` file into a `ProjectManifest`.
    fn parse_manifest(path: &Path) -> Result<ProjectManifest, FString> {
        let content = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read {}: {}", path.display(), e))?;

        let toml_value: toml::Value = content
            .parse::<toml::Value>()
            .map_err(|e| format!("Failed to parse Fusion.toml: {}", e))?;

        let name = toml_value
            .get("package")
            .and_then(|pkg| pkg.get("name"))
            .and_then(|v| v.as_str())
            .unwrap_or("fusion_project")
            .to_string();

        let version = toml_value
            .get("package")
            .and_then(|pkg| pkg.get("version"))
            .and_then(|v| v.as_str())
            .unwrap_or("0.1.0")
            .to_string();

        let authors = toml_value
            .get("package")
            .and_then(|pkg| pkg.get("authors"))
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default();

        let description = toml_value
            .get("package")
            .and_then(|pkg| pkg.get("description"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let license = toml_value
            .get("package")
            .and_then(|pkg| pkg.get("license"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let repository = toml_value
            .get("package")
            .and_then(|pkg| pkg.get("repository"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let runtime = toml_value
            .get("runtime")
            .and_then(|rt| rt.get("profile"))
            .and_then(|v| v.as_str())
            .map(|profile| match profile {
                "supernova" | "Supernova" => RuntimeTarget::Supernova,
                "nebula" | "Nebula" => RuntimeTarget::Nebula,
                "pulsar" | "Pulsar" => RuntimeTarget::Pulsar,
                _ => RuntimeTarget::Supernova,
            })
            .unwrap_or(RuntimeTarget::Supernova);

        let registry_url = toml_value
            .get("registry")
            .and_then(|r| r.get("url"))
            .and_then(|v| v.as_str())
            .unwrap_or("https://registry.fusionlang.dev")
            .to_string();

        // Parse [dependencies]
        let dependencies = Self::parse_dep_table(&toml_value, "dependencies");

        // Parse [dev-dependencies]
        let dev_dependencies = Self::parse_dep_table(&toml_value, "dev-dependencies");

        let build_cache_enabled = toml_value
            .get("build")
            .and_then(|b| b.get("incremental"))
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        Ok(ProjectManifest {
            name,
            version,
            authors,
            description,
            license,
            repository,
            runtime,
            dependencies,
            dev_dependencies,
            build_cache_enabled,
            registry_url,
        })
    }

    /// Parse a dependency table from the manifest TOML.
    fn parse_dep_table(toml_value: &toml::Value, table_name: &str) -> Vec<Dependency> {
        toml_value
            .get(table_name)
            .and_then(|v| v.as_table())
            .map(|table| {
                table
                    .iter()
                    .map(|(dep_name, dep_val)| match dep_val {
                        toml::Value::String(version_str) => Dependency {
                            name: dep_name.clone(),
                            version: version_str.clone(),
                            path: None,
                            registry: None,
                            features: vec![],
                            optional: false,
                        },
                        toml::Value::Table(dep_table) => {
                            let version = dep_table
                                .get("version")
                                .and_then(|v| v.as_str())
                                .unwrap_or("*")
                                .to_string();
                            let path = dep_table
                                .get("path")
                                .and_then(|v| v.as_str())
                                .map(|s| s.to_string());
                            let registry = dep_table
                                .get("registry")
                                .and_then(|v| v.as_str())
                                .map(|s| s.to_string());
                            let features = dep_table
                                .get("features")
                                .and_then(|v| v.as_array())
                                .map(|arr| {
                                    arr.iter()
                                        .filter_map(|v| v.as_str().map(|s| s.to_string()))
                                        .collect()
                                })
                                .unwrap_or_default();
                            let optional = dep_table
                                .get("optional")
                                .and_then(|v| v.as_bool())
                                .unwrap_or(false);
                            Dependency {
                                name: dep_name.clone(),
                                version,
                                path,
                                registry,
                                features,
                                optional,
                            }
                        }
                        _ => Dependency {
                            name: dep_name.clone(),
                            version: "*".to_string(),
                            path: None,
                            registry: None,
                            features: vec![],
                            optional: false,
                        },
                    })
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Resolve all dependencies with version constraints using topological sort.
    /// Returns the resolved dependency graph in build order.
    pub fn resolve_dependencies(&self) -> Result<Vec<ResolvedDependency>, FString> {
        let mut resolved = Vec::new();
        let mut visited = HashSet::new();
        let mut in_progress = HashSet::new();

        for dep in &self.manifest.dependencies {
            self.resolve_dep_recursive(dep, &mut resolved, &mut visited, &mut in_progress)?;
        }

        Ok(resolved)
    }

    /// Recursively resolve a dependency and its transitive dependencies.
    fn resolve_dep_recursive(
        &self,
        dep: &Dependency,
        resolved: &mut Vec<ResolvedDependency>,
        visited: &mut HashSet<String>,
        in_progress: &mut HashSet<String>,
    ) -> Result<(), FString> {
        if visited.contains(&dep.name) {
            return Ok(());
        }

        // Cycle detection
        if in_progress.contains(&dep.name) {
            return Err(format!(
                "Dependency cycle detected: {} depends on itself",
                dep.name
            ));
        }
        in_progress.insert(dep.name.clone());

        // Determine the source
        let source = if let Some(ref path) = dep.path {
            let abs_path = if Path::new(path).is_absolute() {
                PathBuf::from(path)
            } else {
                self.project_root.join(path)
            };

            // Compute checksum of the local dependency
            let checksum = self.compute_path_checksum(&abs_path)?;

            ResolvedDependency {
                name: dep.name.clone(),
                version: "local".to_string(),
                source: DependencySource::Local(abs_path),
                checksum,
            }
        } else {
            // Registry dependency: use the version constraint as the version string
            // In a real implementation, this would query the registry
            ResolvedDependency {
                name: dep.name.clone(),
                version: dep.version.clone(),
                source: DependencySource::Registry {
                    url: self.manifest.registry_url.clone(),
                    version: dep.version.clone(),
                },
                checksum: String::new(),
            }
        };

        // Resolve transitive dependencies by reading the dependency's own manifest
        // (for local deps only; registry deps would need fetching)
        if let DependencySource::Local(ref path) = source.source {
            let sub_manifest_path = path.join("Fusion.toml");
            if sub_manifest_path.exists() {
                if let Ok(sub_manifest) = Self::parse_manifest(&sub_manifest_path) {
                    for sub_dep in &sub_manifest.dependencies {
                        self.resolve_dep_recursive(
                            sub_dep,
                            resolved,
                            visited,
                            in_progress,
                        )?;
                    }
                }
            }
        }

        in_progress.remove(&dep.name);
        visited.insert(dep.name.clone());
        resolved.push(source);

        Ok(())
    }

    /// Compute a checksum for a local path dependency.
    fn compute_path_checksum(&self, path: &Path) -> Result<String, FString> {
        let mut hasher = Sha256::new();

        if path.is_file() {
            let content = fs::read(path)
                .map_err(|e| format!("Failed to read {}: {}", path.display(), e))?;
            hasher.update(&content);
        } else if path.is_dir() {
            let mut entries: Vec<_> = fs::read_dir(path)
                .map_err(|e| format!("Failed to read dir {}: {}", path.display(), e))?
                .filter_map(|e| e.ok())
                .filter(|e| {
                    e.path()
                        .extension()
                        .map_or(false, |ext| ext == "fu" || ext == "toml")
                })
                .collect();
            entries.sort_by_key(|e| e.path());
            for entry in &entries {
                if let Ok(content) = fs::read(entry.path()) {
                    hasher.update(entry.file_name().to_string_lossy().as_bytes());
                    hasher.update(&content);
                }
            }
        }

        Ok(hex::encode(hasher.finalize()))
    }

    /// Generate a lock file from resolved dependencies.
    pub fn generate_lock_file(&self, resolved: &[ResolvedDependency]) -> LockFile {
        let packages: Vec<LockEntry> = resolved
            .iter()
            .map(|dep| {
                let source_str = match &dep.source {
                    DependencySource::Local(path) => format!("path+{}", path.display()),
                    DependencySource::Registry { url, version } => {
                        format!("registry+{}#{}", url, version)
                    }
                    DependencySource::Git { url, rev } => format!("git+{}#{}", url, rev),
                };
                LockEntry {
                    name: dep.name.clone(),
                    version: dep.version.clone(),
                    source: source_str,
                    checksum: dep.checksum.clone(),
                }
            })
            .collect();

        LockFile {
            version: "1".to_string(),
            packages,
        }
    }

    /// Write the lock file to disk.
    pub fn write_lock_file(
        &self,
        resolved: &[ResolvedDependency],
    ) -> Result<PathBuf, FString> {
        let lock = self.generate_lock_file(resolved);
        let lock_path = self.project_root.join("Fusion.lock");
        let content = serde_json::to_string_pretty(&lock)
            .map_err(|e| format!("Failed to serialize lock file: {}", e))?;
        fs::write(&lock_path, content)
            .map_err(|e| format!("Failed to write lock file: {}", e))?;
        Ok(lock_path)
    }

    /// Read an existing lock file, if present.
    pub fn read_lock_file(&self) -> Option<LockFile> {
        let lock_path = self.project_root.join("Fusion.lock");
        let content = fs::read_to_string(&lock_path).ok()?;
        serde_json::from_str(&content).ok()
    }

    /// Check if the lock file is up-to-date with the manifest.
    pub fn is_lock_file_valid(&self) -> bool {
        match self.read_lock_file() {
            Some(lock) => {
                // Verify each locked package is still compatible with manifest deps
                let lock_map: HashMap<&str, &LockEntry> = lock
                    .packages
                    .iter()
                    .map(|p| (p.name.as_str(), p))
                    .collect();

                self.manifest.dependencies.iter().all(|dep| {
                    lock_map
                        .get(dep.name.as_str())
                        .map_or(false, |entry| entry.version == dep.version)
                })
            }
            None => false,
        }
    }

    /// Resolve dependencies and write the lock file if needed.
    pub fn ensure_lock_file(&self) -> Result<Vec<ResolvedDependency>, FString> {
        if self.is_lock_file_valid() {
            // Lock file is valid; use it for resolution
            let lock = self.read_lock_file().unwrap();
            return Ok(lock
                .packages
                .iter()
                .map(|entry| {
                    let source = if entry.source.starts_with("path+") {
                        let path = entry.source.strip_prefix("path+").unwrap();
                        DependencySource::Local(PathBuf::from(path))
                    } else if entry.source.starts_with("registry+") {
                        let rest = entry.source.strip_prefix("registry+").unwrap();
                        if let Some((url, version)) = rest.split_once('#') {
                            DependencySource::Registry {
                                url: url.to_string(),
                                version: version.to_string(),
                            }
                        } else {
                            DependencySource::Registry {
                                url: self.manifest.registry_url.clone(),
                                version: entry.version.clone(),
                            }
                        }
                    } else {
                        DependencySource::Registry {
                            url: self.manifest.registry_url.clone(),
                            version: entry.version.clone(),
                        }
                    };

                    ResolvedDependency {
                        name: entry.name.clone(),
                        version: entry.version.clone(),
                        source,
                        checksum: entry.checksum.clone(),
                    }
                })
                .collect());
        }

        // Need to resolve and generate lock file
        let resolved = self.resolve_dependencies()?;
        self.write_lock_file(&resolved)?;
        Ok(resolved)
    }

    /// Build the dependency graph as adjacency list for visualization/debugging.
    pub fn dependency_graph(&self) -> HashMap<String, Vec<String>> {
        let mut graph: HashMap<String, Vec<String>> = HashMap::new();

        for dep in &self.manifest.dependencies {
            let deps_of_dep = self.transitive_dep_names(dep);
            graph.insert(dep.name.clone(), deps_of_dep);
        }

        graph
    }

    /// Get transitive dependency names for a dependency.
    fn transitive_dep_names(&self, dep: &Dependency) -> Vec<String> {
        let mut names = Vec::new();
        if let Some(ref path) = dep.path {
            let abs_path = if Path::new(path).is_absolute() {
                PathBuf::from(path)
            } else {
                self.project_root.join(path)
            };
            let sub_manifest_path = abs_path.join("Fusion.toml");
            if let Ok(content) = fs::read_to_string(&sub_manifest_path) {
                if let Ok(toml_value) = content.parse::<toml::Value>() {
                    if let Some(sub_deps) = Self::parse_dep_table(&toml_value, "dependencies")
                        .into_iter()
                        .map(|d| d.name)
                        .collect::<Vec<_>>()
                        .into()
                    {
                        names = sub_deps;
                    }
                }
            }
        }
        names
    }

    /// Configures the codegen backend to link against the correct runtime.
    pub fn configure_target(&self, mut base_config: CodegenConfig) -> CodegenConfig {
        match self.manifest.runtime {
            RuntimeTarget::Supernova => {
                base_config.link_libs.push("supernova_rt".to_string());
                base_config
                    .link_libs
                    .push("qst_neuralseal_pqc".to_string());
            }
            RuntimeTarget::Nebula => {
                base_config.link_libs.push("nebula_rt_v2".to_string());
            }
            RuntimeTarget::Pulsar => {
                base_config.target_triple = "wasm32-unknown-unknown".to_string();
                base_config.link_libs.push("pulsar_micro_rt".to_string());
            }
        }
        base_config
    }
}

// ==================== Tests ====================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_manifest() {
        let content = r#"
[package]
name = "my-project"
version = "1.0.0"
authors = ["Alice"]
description = "A test project"
license = "MIT"

[runtime]
profile = "supernova"

[dependencies]
fusion-std = "^0.2.0"
fusion-math = { version = ">=1.0.0", features = ["complex"] }
my-local-dep = { path = "../local-dep" }
"#;

        let toml_value: toml::Value = content.parse().unwrap();
        let name = toml_value
            .get("package")
            .and_then(|p| p.get("name"))
            .and_then(|v| v.as_str())
            .unwrap();
        assert_eq!(name, "my-project");

        let deps = Forge::parse_dep_table(&toml_value, "dependencies");
        assert_eq!(deps.len(), 3);
        assert_eq!(deps[0].name, "fusion-std");
        assert_eq!(deps[0].version, "^0.2.0");
        assert!(deps[1].features.contains(&"complex".to_string()));
        assert!(deps[2].path.is_some());
    }

    #[test]
    fn test_lock_file_roundtrip() {
        let lock = LockFile {
            version: "1".to_string(),
            packages: vec![LockEntry {
                name: "test-pkg".to_string(),
                version: "1.0.0".to_string(),
                source: "registry+https://registry.fusionlang.dev#1.0.0".to_string(),
                checksum: "abc123".to_string(),
            }],
        };

        let json = serde_json::to_string_pretty(&lock).unwrap();
        let restored: LockFile = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.packages.len(), 1);
        assert_eq!(restored.packages[0].name, "test-pkg");
    }

    #[test]
    fn test_dependency_source_display() {
        let local = DependencySource::Local(PathBuf::from("/path/to/dep"));
        let registry = DependencySource::Registry {
            url: "https://registry.fusionlang.dev".to_string(),
            version: "1.0.0".to_string(),
        };
        let git = DependencySource::Git {
            url: "https://github.com/user/repo".to_string(),
            rev: "abc123".to_string(),
        };

        match &local {
            DependencySource::Local(p) => assert_eq!(p.display().to_string(), "/path/to/dep"),
            _ => panic!("Expected Local"),
        }
        match &registry {
            DependencySource::Registry { url, version } => {
                assert!(url.contains("fusionlang"));
                assert_eq!(version, "1.0.0");
            }
            _ => panic!("Expected Registry"),
        }
        match &git {
            DependencySource::Git { url, rev } => {
                assert!(url.contains("github"));
                assert_eq!(rev, "abc123");
            }
            _ => panic!("Expected Git"),
        }
    }
}
