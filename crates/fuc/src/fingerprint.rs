//! Build fingerprinting and caching for incremental compilation.
//!
//! Provides per-file hash caching, dependency-aware invalidation,
//! and cache directory management for fast incremental builds.

use anyhow::{Context, Result};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

/// Cache directory for fingerprint data.
const FINGERPRINT_CACHE_DIR: &str = ".fusion-cache";

/// Cache file for storing per-file fingerprints.
const FINGERPRINT_CACHE_FILE: &str = "fingerprints.json";

/// Cache file for storing dependency invalidation data.
const DEPENDENCY_CACHE_FILE: &str = "dep_invalidation.json";

/// Per-file fingerprint entry stored in cache.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FileFingerprint {
    pub path: String,
    pub hash: String,
    pub modified: SystemTime,
    pub size: u64,
}

/// Dependency-based invalidation record.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DepInvalidation {
    /// Source file path
    pub source: String,
    /// Files that depend on this source
    pub dependents: Vec<String>,
    /// Hash of the source when dependents were last built
    pub source_hash: String,
}

/// Cache directory manager for fingerprint data.
pub struct FingerprintCache {
    cache_dir: PathBuf,
    fingerprints: HashMap<String, FileFingerprint>,
    dep_invalidations: HashMap<String, DepInvalidation>,
}

impl FingerprintCache {
    /// Open or create a fingerprint cache in the given source directory.
    pub fn open(source_dir: &Path) -> Result<Self> {
        let cache_dir = source_dir.join(FINGERPRINT_CACHE_DIR);
        fs::create_dir_all(&cache_dir)
            .context("Failed to create fingerprint cache directory")?;

        let fingerprints = Self::load_fingerprints(&cache_dir)?;
        let dep_invalidations = Self::load_dep_invalidations(&cache_dir)?;

        Ok(Self {
            cache_dir,
            fingerprints,
            dep_invalidations,
        })
    }

    /// Get the cache directory path.
    pub fn cache_dir(&self) -> &Path {
        &self.cache_dir
    }

    /// Check if a file is dirty (needs recompilation).
    pub fn is_dirty(&self, file_path: &Path) -> bool {
        let key = file_path.to_string_lossy().to_string();

        let current_meta = match fs::metadata(file_path) {
            Ok(m) => m,
            Err(_) => return true, // File doesn't exist or can't stat
        };

        match self.fingerprints.get(&key) {
            Some(fp) => {
                // Check modification time and size
                let current_modified = current_meta
                    .modified()
                    .unwrap_or(SystemTime::UNIX_EPOCH);
                let current_size = current_meta.len();

                fp.modified != current_modified || fp.size != current_size
            }
            None => true, // No fingerprint recorded
        }
    }

    /// Check if any file in a set is dirty.
    pub fn any_dirty(&self, files: &[&Path]) -> bool {
        files.iter().any(|f| self.is_dirty(f))
    }

    /// Compute the hash of a single file.
    pub fn hash_file(file_path: &Path) -> Result<String> {
        let content = fs::read(file_path)
            .with_context(|| format!("Failed to read file: {}", file_path.display()))?;
        let mut hasher = Sha256::new();
        hasher.update(&content);
        Ok(hex::encode(hasher.finalize()))
    }

    /// Compute the hash of a source directory (all .fu files).
    pub fn hash_source_dir(source_dir: &Path) -> Result<String> {
        let mut hasher = Sha256::new();

        if source_dir.is_dir() {
            let mut entries: Vec<_> = fs::read_dir(source_dir)
                .context("Failed to read source directory")?
                .filter_map(|e| e.ok())
                .filter(|e| {
                    e.path()
                        .extension()
                        .map_or(false, |ext| ext == "fu")
                })
                .collect();
            entries.sort_by_key(|e| e.path());

            for entry in &entries {
                let content = fs::read(entry.path())
                    .with_context(|| format!("Failed to read {}", entry.path().display()))?;
                hasher.update(entry.file_name().to_string_lossy().as_bytes());
                hasher.update(&content);
            }
        } else {
            let content = fs::read(source_dir)
                .with_context(|| format!("Failed to read {}", source_dir.display()))?;
            hasher.update(&content);
        }

        Ok(hex::encode(hasher.finalize()))
    }

    /// Compute the hash of a file, incorporating dependency hashes.
    /// This enables dependency-aware invalidation: if a dependency changes,
    /// all files that depend on it are marked dirty.
    pub fn hash_file_with_deps(
        &self,
        file_path: &Path,
        dependency_dirs: &[&Path],
    ) -> Result<String> {
        let mut hasher = Sha256::new();

        // Hash the file itself
        let content = fs::read(file_path)
            .with_context(|| format!("Failed to read {}", file_path.display()))?;
        hasher.update(&content);

        // Hash all dependency directories
        for dep_dir in dependency_dirs {
            if dep_dir.is_dir() {
                let dep_hash = Self::hash_source_dir(dep_dir)?;
                hasher.update(dep_hash.as_bytes());
            } else if dep_dir.is_file() {
                let dep_content = fs::read(dep_dir)
                    .with_context(|| format!("Failed to read {}", dep_dir.display()))?;
                hasher.update(&dep_content);
            }
        }

        Ok(hex::encode(hasher.finalize()))
    }

    /// Compute hash for a single file and update the cache.
    pub fn update_file(&mut self, file_path: &Path) -> Result<bool> {
        let key = file_path.to_string_lossy().to_string();
        let hash = Self::hash_file(file_path)?;
        let meta = fs::metadata(file_path)
            .with_context(|| format!("Failed to stat {}", file_path.display()))?;

        let modified = meta.modified().unwrap_or(SystemTime::UNIX_EPOCH);
        let size = meta.len();

        let dirty = match self.fingerprints.get(&key) {
            Some(fp) => fp.hash != hash,
            None => true,
        };

        self.fingerprints.insert(
            key,
            FileFingerprint {
                path: file_path.to_string_lossy().to_string(),
                hash,
                modified,
                size,
            },
        );

        Ok(dirty)
    }

    /// Update all .fu files in a source directory.
    pub fn update_source_dir(&mut self, source_dir: &Path) -> Result<Vec<PathBuf>> {
        let mut dirty_files = Vec::new();

        if source_dir.is_dir() {
            let entries: Vec<_> = fs::read_dir(source_dir)
                .context("Failed to read source directory")?
                .filter_map(|e| e.ok())
                .filter(|e| {
                    e.path()
                        .extension()
                        .map_or(false, |ext| ext == "fu")
                })
                .collect();

            for entry in &entries {
                if self.update_file(&entry.path())? {
                    dirty_files.push(entry.path());
                }
            }
        }

        Ok(dirty_files)
    }

    /// Register a dependency relationship: source depends on dep_dir.
    /// When dep_dir changes, all its dependents are marked for rebuild.
    pub fn register_dependency(
        &mut self,
        source: &Path,
        dep_dir: &Path,
    ) -> Result<()> {
        let source_key = source.to_string_lossy().to_string();
        let dep_key = dep_dir.to_string_lossy().to_string();
        let dep_hash = if dep_dir.is_file() {
            Self::hash_file(dep_dir)?
        } else {
            Self::hash_source_dir(dep_dir)?
        };

        let inv = self
            .dep_invalidations
            .entry(dep_key.clone())
            .or_insert_with(|| DepInvalidation {
                source: dep_key.clone(),
                dependents: Vec::new(),
                source_hash: dep_hash.clone(),
            });

        if !inv.dependents.contains(&source_key) {
            inv.dependents.push(source_key);
        }
        inv.source_hash = dep_hash;

        Ok(())
    }

    /// Check if a dependency has changed and return its dependent files.
    pub fn check_dep_invalidation(&self, dep_path: &Path) -> Option<Vec<String>> {
        let dep_key = dep_path.to_string_lossy().to_string();
        self.dep_invalidations
            .get(&dep_key)
            .map(|inv| inv.dependents.clone())
    }

    /// Invalidate dependents of a changed dependency.
    /// Returns the list of files that need rebuilding.
    pub fn invalidate_dependents(&mut self, dep_path: &Path) -> Result<Vec<PathBuf>> {
        let dep_key = dep_path.to_string_lossy().to_string();
        let current_hash = if dep_path.is_file() {
            Self::hash_file(dep_path)?
        } else {
            Self::hash_source_dir(dep_path)?
        };

        if let Some(inv) = self.dep_invalidations.get(&dep_key) {
            if inv.source_hash != current_hash {
                let dependents: Vec<PathBuf> = inv
                    .dependents
                    .iter()
                    .map(PathBuf::from)
                    .filter(|p| p.exists())
                    .collect();

                // Update the stored hash
                if let Some(inv_mut) = self.dep_invalidations.get_mut(&dep_key) {
                    inv_mut.source_hash = current_hash;
                }

                return Ok(dependents);
            }
        }

        Ok(Vec::new())
    }

    /// Save the fingerprint cache to disk.
    pub fn save(&self) -> Result<()> {
        let fp_path = self.cache_dir.join(FINGERPRINT_CACHE_FILE);
        let json = serde_json::to_string_pretty(&self.fingerprints)
            .context("Failed to serialize fingerprints")?;
        fs::write(&fp_path, json)
            .with_context(|| format!("Failed to write {}", fp_path.display()))?;

        let dep_path = self.cache_dir.join(DEPENDENCY_CACHE_FILE);
        let json = serde_json::to_string_pretty(&self.dep_invalidations)
            .context("Failed to serialize dependency invalidations")?;
        fs::write(&dep_path, json)
            .with_context(|| format!("Failed to write {}", dep_path.display()))?;

        Ok(())
    }

    /// Load fingerprints from disk.
    fn load_fingerprints(cache_dir: &Path) -> Result<HashMap<String, FileFingerprint>> {
        let path = cache_dir.join(FINGERPRINT_CACHE_FILE);
        if !path.exists() {
            return Ok(HashMap::new());
        }
        let content = fs::read_to_string(&path)
            .with_context(|| format!("Failed to read {}", path.display()))?;
        let data: HashMap<String, FileFingerprint> = serde_json::from_str(&content)
            .with_context(|| format!("Failed to parse {}", path.display()))?;
        Ok(data)
    }

    /// Load dependency invalidation data from disk.
    fn load_dep_invalidations(cache_dir: &Path) -> Result<HashMap<String, DepInvalidation>> {
        let path = cache_dir.join(DEPENDENCY_CACHE_FILE);
        if !path.exists() {
            return Ok(HashMap::new());
        }
        let content = fs::read_to_string(&path)
            .with_context(|| format!("Failed to read {}", path.display()))?;
        let data: HashMap<String, DepInvalidation> = serde_json::from_str(&content)
            .with_context(|| format!("Failed to parse {}", path.display()))?;
        Ok(data)
    }

    /// Clear the entire cache directory.
    pub fn clear(&mut self) -> Result<()> {
        if self.cache_dir.exists() {
            fs::remove_dir_all(&self.cache_dir)
                .context("Failed to remove cache directory")?;
            fs::create_dir_all(&self.cache_dir)
                .context("Failed to recreate cache directory")?;
        }
        self.fingerprints.clear();
        self.dep_invalidations.clear();
        Ok(())
    }

    /// Get cache statistics.
    pub fn stats(&self) -> FingerprintStats {
        let cache_dir_size = Self::dir_size(&self.cache_dir);
        FingerprintStats {
            cached_files: self.fingerprints.len(),
            dependency_entries: self.dep_invalidations.len(),
            cache_size_bytes: cache_dir_size,
        }
    }

    fn dir_size(path: &Path) -> u64 {
        if !path.exists() {
            return 0;
        }
        let mut total = 0;
        if let Ok(entries) = fs::read_dir(path) {
            for entry in entries.flatten() {
                if let Ok(meta) = entry.metadata() {
                    total += meta.len();
                }
            }
        }
        total
    }
}

/// Cache statistics.
#[derive(Debug, Clone)]
pub struct FingerprintStats {
    pub cached_files: usize,
    pub dependency_entries: usize,
    pub cache_size_bytes: u64,
}

// ==================== Legacy API ====================

/// Legacy API: check if source directory is dirty.
pub fn is_dirty(source_dir: &Path, _artifact: &Path) -> bool {
    let _cache = match FingerprintCache::open(source_dir) {
        Ok(c) => c,
        Err(_) => return true,
    };

    let current = match FingerprintCache::hash_source_dir(source_dir) {
        Ok(h) => h,
        Err(_) => return true,
    };

    let cache_file = source_dir.join(FINGERPRINT_CACHE_DIR).join("hash");
    match fs::read_to_string(&cache_file) {
        Ok(stored) => stored.trim() != current,
        Err(_) => true,
    }
}

/// Legacy API: save hash for incremental compilation.
pub fn save_hash(source_dir: &Path, _artifact: &Path) -> Result<()> {
    let hash = FingerprintCache::hash_source_dir(source_dir)?;
    let cache_dir = source_dir.join(FINGERPRINT_CACHE_DIR);
    fs::create_dir_all(&cache_dir)?;
    fs::write(cache_dir.join("hash"), hash)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_source(dir: &Path, name: &str, content: &str) -> PathBuf {
        let path = dir.join(name);
        fs::write(&path, content).unwrap();
        path
    }

    #[test]
    fn test_hash_file_consistency() {
        let dir = TempDir::new().unwrap();
        let file = create_test_source(dir.path(), "test.fu", "fn main() { }");

        let hash1 = FingerprintCache::hash_file(&file).unwrap();
        let hash2 = FingerprintCache::hash_file(&file).unwrap();
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_hash_file_change_detection() {
        let dir = TempDir::new().unwrap();
        let file = create_test_source(dir.path(), "test.fu", "fn main() { }");

        let hash1 = FingerprintCache::hash_file(&file).unwrap();

        // Modify the file
        fs::write(&file, "fn main() { updated }").unwrap();

        let hash2 = FingerprintCache::hash_file(&file).unwrap();
        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_hash_source_dir() {
        let dir = TempDir::new().unwrap();
        create_test_source(dir.path(), "a.fu", "fn a() { }");
        create_test_source(dir.path(), "b.fu", "fn b() { }");
        create_test_source(dir.path(), "ignore.txt", "not a fusion file");

        let hash = FingerprintCache::hash_source_dir(dir.path()).unwrap();
        assert!(!hash.is_empty());
        assert_eq!(hash.len(), 64); // SHA-256 hex length
    }

    #[test]
    fn test_fingerprint_cache_open_and_update() {
        let dir = TempDir::new().unwrap();
        let file = create_test_source(dir.path(), "test.fu", "fn main() { }");

        let mut cache = FingerprintCache::open(dir.path()).unwrap();
        assert!(cache.update_file(&file).unwrap());

        // Save and reload
        cache.save().unwrap();

        let cache2 = FingerprintCache::open(dir.path()).unwrap();
        assert!(!cache2.is_dirty(&file));
    }

    #[test]
    fn test_dependency_invalidation() {
        let dir = TempDir::new().unwrap();
        let source = create_test_source(dir.path(), "main.fu", "use lib");
        let dep = create_test_source(dir.path(), "lib.fu", "fn lib() { }");

        let mut cache = FingerprintCache::open(dir.path()).unwrap();
        cache.register_dependency(&source, &dep).unwrap();
        cache.save().unwrap();

        // Check dependents
        let dependents = cache.check_dep_invalidation(&dep).unwrap();
        assert_eq!(dependents.len(), 1);
        assert_eq!(dependents[0], source.to_string_lossy());
    }

    #[test]
    fn test_clear_cache() {
        let dir = TempDir::new().unwrap();
        let file = create_test_source(dir.path(), "test.fu", "fn main() { }");

        let mut cache = FingerprintCache::open(dir.path()).unwrap();
        cache.update_file(&file).unwrap();
        cache.save().unwrap();

        assert!(cache.stats().cached_files > 0);

        cache.clear().unwrap();
        assert_eq!(cache.stats().cached_files, 0);
    }

    #[test]
    fn test_stats() {
        let dir = TempDir::new().unwrap();
        create_test_source(dir.path(), "a.fu", "fn a() { }");
        create_test_source(dir.path(), "b.fu", "fn b() { }");

        let mut cache = FingerprintCache::open(dir.path()).unwrap();
        let file_a = dir.path().join("a.fu");
        let file_b = dir.path().join("b.fu");
        cache.update_file(&file_a).unwrap();
        cache.update_file(&file_b).unwrap();
        cache.save().unwrap();

        let stats = cache.stats();
        assert_eq!(stats.cached_files, 2);
    }

    #[test]
    fn test_hash_with_dependencies() {
        let dir = TempDir::new().unwrap();
        let source = create_test_source(dir.path(), "main.fu", "use lib");
        let dep_dir = dir.path().join("lib");
        fs::create_dir_all(&dep_dir).unwrap();
        create_test_source(&dep_dir, "lib.fu", "fn lib() { }");

        let mut cache = FingerprintCache::open(dir.path()).unwrap();

        let hash1 = cache
            .hash_file_with_deps(&source, &[&dep_dir])
            .unwrap();

        // Change the dependency
        create_test_source(&dep_dir, "lib.fu", "fn lib() { updated }");

        let hash2 = cache
            .hash_file_with_deps(&source, &[&dep_dir])
            .unwrap();

        assert_ne!(hash1, hash2);
    }
}
