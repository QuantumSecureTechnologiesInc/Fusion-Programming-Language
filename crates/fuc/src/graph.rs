//! Dependency graph resolution for the Fusion build system.
use std::collections::{HashMap, HashSet, VecDeque};
use std::fs;
use std::path::{Path, PathBuf};
use anyhow::Result;
use regex::Regex;

pub struct BuildGraph {
    pub root_name: String,
    pub packages: Vec<PackageNode>,
}

pub struct PackageNode {
    pub name: String,
    pub path: PathBuf,
}

impl BuildGraph {
    pub fn topological_sort(&self) -> Result<Vec<&PackageNode>> {
        let mut name_to_idx: HashMap<&str, usize> = HashMap::new();
        for (i, pkg) in self.packages.iter().enumerate() {
            name_to_idx.insert(&pkg.name, i);
        }

        // Build adjacency list from file-level use/mod deps (already resolved in resolve_dependencies).
        // Here we just return packages in dependency order as placed by topological sort.
        Ok(self.packages.iter().collect())
    }
}

/// Parse `use` and `mod` declarations from a `.fu` file to discover dependencies.
fn parse_file_deps(content: &str) -> Vec<String> {
    let mut deps = Vec::new();
    let use_re = Regex::new(r"^use\s+([\w:]+)").unwrap();
    let mod_re = Regex::new(r"^mod\s+(\w+)").unwrap();

    for line in content.lines() {
        let trimmed = line.trim();
        if let Some(caps) = use_re.captures(trimmed) {
            let path = caps.get(1).unwrap().as_str();
            // Extract the top-level crate name (first segment before `::`)
            if let Some(first) = path.split("::").next() {
                // Skip `super`, `self`, `crate` — these are intra-file references
                if !matches!(first, "super" | "self" | "crate") {
                    deps.push(first.to_string());
                }
            }
        } else if let Some(caps) = mod_re.captures(trimmed) {
            let mod_name = caps.get(1).unwrap().as_str();
            deps.push(mod_name.to_string());
        }
    }
    deps
}

/// Recursively discover all `.fu` files under a directory.
fn discover_fu_files(dir: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    if dir.is_dir() {
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    // Skip hidden dirs and target dirs
                    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                        if name.starts_with('.') || name == "target" {
                            continue;
                        }
                    }
                    files.extend(discover_fu_files(&path));
                } else if path.extension().and_then(|e| e.to_str()) == Some("fu") {
                    files.push(path);
                }
            }
        }
    }
    files
}

/// Determine the package name for a file based on its path relative to root.
fn file_package_name(root: &Path, file: &Path) -> String {
    file.strip_prefix(root)
        .ok()
        .and_then(|rel| {
            let components: Vec<&str> = rel.iter().map(|c| c.to_str().unwrap_or("")).collect();
            if components.len() <= 1 {
                Some("main".to_string())
            } else {
                Some(components[0].to_string())
            }
        })
        .unwrap_or_else(|| "main".to_string())
}

pub fn resolve_dependencies(root: &Path) -> Result<BuildGraph> {
    let fu_files = discover_fu_files(root);

    if fu_files.is_empty() {
        return Ok(BuildGraph {
            root_name: "main".to_string(),
            packages: vec![PackageNode {
                name: "main".to_string(),
                path: root.to_path_buf(),
            }],
        });
    }

    // Map package name -> set of files belonging to it
    let mut package_files: HashMap<String, Vec<PathBuf>> = HashMap::new();
    // Map file -> list of dependency names found via use/mod
    let mut file_deps: HashMap<PathBuf, Vec<String>> = HashMap::new();
    // Map package name -> set of packages it depends on
    let mut package_deps: HashMap<String, HashSet<String>> = HashMap::new();

    for file in &fu_files {
        let pkg_name = file_package_name(root, file);
        package_files.entry(pkg_name.clone()).or_default().push(file.clone());

        if let Ok(content) = fs::read_to_string(file) {
            let deps = parse_file_deps(&content);
            file_deps.insert(file.clone(), deps.clone());
            let dep_set = package_deps.entry(pkg_name).or_default();
            for dep in deps {
                dep_set.insert(dep);
            }
        }
    }

    // Topological sort using Kahn's algorithm
    let all_packages: Vec<String> = package_files.keys().cloned().collect();
    let mut in_degree: HashMap<String, usize> = HashMap::new();
    let mut adj: HashMap<String, Vec<String>> = HashMap::new();

    for pkg in &all_packages {
        in_degree.entry(pkg.clone()).or_insert(0);
        adj.entry(pkg.clone()).or_default();
    }

    // Only count edges between packages that actually exist in our file set
    let package_set: HashSet<&String> = all_packages.iter().collect();
    for (pkg, deps) in &package_deps {
        for dep in deps {
            if package_set.contains(dep) && dep != pkg {
                adj.entry(dep.clone()).or_default().push(pkg.clone());
                *in_degree.entry(pkg.clone()).or_insert(0) += 1;
            }
        }
    }

    // Enqueue packages with no dependencies
    let mut queue: VecDeque<String> = VecDeque::new();
    let mut sorted: Vec<String> = Vec::new();

    for pkg in &all_packages {
        if *in_degree.get(pkg).unwrap_or(&0) == 0 {
            queue.push_back(pkg.clone());
        }
    }

    while let Some(pkg) = queue.pop_front() {
        sorted.push(pkg.clone());
        if let Some(neighbors) = adj.get(&pkg) {
            for neighbor in neighbors {
                let deg = in_degree.get_mut(neighbor).unwrap();
                *deg -= 1;
                if *deg == 0 {
                    queue.push_back(neighbor.clone());
                }
            }
        }
    }

    // Add any remaining packages (cycles or unreachable) at the end
    for pkg in &all_packages {
        if !sorted.contains(pkg) {
            sorted.push(pkg.clone());
        }
    }

    let packages: Vec<PackageNode> = sorted
        .into_iter()
        .map(|name| {
            let path = package_files
                .get(&name)
                .and_then(|files| files.first())
                .cloned()
                .unwrap_or_else(|| root.join(&name));
            PackageNode { name, path }
        })
        .collect();

    Ok(BuildGraph {
        root_name: packages.first().map(|p| p.name.clone()).unwrap_or_else(|| "main".to_string()),
        packages,
    })
}
