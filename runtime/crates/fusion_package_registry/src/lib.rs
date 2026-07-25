//! Fusion Package Registry
//!
//! Provides a local and remote package registry for the Fusion ecosystem.
//! Handles package upload/download, version management, search/discovery,
//! and authentication/authorization.

use chrono::{DateTime, Utc};
use semver::{Version, VersionReq};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;
use thiserror::Error;

// ==================== Errors ====================

#[derive(Error, Debug)]
pub enum RegistryError {
    #[error("Package not found: {0}")]
    PackageNotFound(String),

    #[error("Version not found: {0}@{1}")]
    VersionNotFound(String, String),

    #[error("Version already exists: {0}@{1}")]
    VersionAlreadyExists(String, String),

    #[error("Invalid semver: {0}")]
    InvalidSemver(String),

    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    #[error("Forbidden: {0}")]
    Forbidden(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("Checksum mismatch: expected {expected}, got {actual}")]
    ChecksumMismatch { expected: String, actual: String },

    #[error("Upload too large: {size} bytes exceeds limit {limit}")]
    UploadTooLarge { size: u64, limit: u64 },

    #[error("Search index error: {0}")]
    SearchIndex(String),
}

pub type RegistryResult<T> = Result<T, RegistryError>;

// ==================== Package Metadata ====================

/// A Fusion package manifest stored in the registry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageManifest {
    pub name: String,
    pub version: Version,
    pub description: String,
    pub authors: Vec<String>,
    pub license: Option<String>,
    pub repository: Option<String>,
    pub homepage: Option<String>,
    pub keywords: Vec<String>,
    pub categories: Vec<String>,
    pub dependencies: Vec<PackageDependency>,
    pub checksum: String,
    pub size_bytes: u64,
    pub uploaded_at: DateTime<Utc>,
    pub uploader_id: String,
}

/// A dependency declared in a package manifest.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageDependency {
    pub name: String,
    pub version_req: String,
    pub optional: bool,
    pub features: Vec<String>,
}

/// A version entry in the registry index.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionEntry {
    pub version: Version,
    pub checksum: String,
    pub size_bytes: u64,
    pub uploaded_at: DateTime<Utc>,
    pub yanked: bool,
    pub yanked_reason: Option<String>,
}

/// Full package record with all versions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageRecord {
    pub name: String,
    pub description: String,
    pub versions: Vec<VersionEntry>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// ==================== Authentication ====================

/// API token for registry authentication.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiToken {
    pub token_id: String,
    pub user_id: String,
    pub scopes: Vec<TokenScope>,
    pub created_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
}

/// Permission scope for an API token.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TokenScope {
    /// Read packages and metadata
    Read,
    /// Upload new versions
    Publish,
    /// Yank/unyank versions
    Yank,
    /// Admin operations (delete packages, manage users)
    Admin,
}

/// User account in the registry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub user_id: String,
    pub username: String,
    pub email: String,
    pub created_at: DateTime<Utc>,
    /// Set of package names this user owns
    pub owned_packages: HashSet<String>,
}

// ==================== Search Index ====================

/// Search index entry for fast package discovery.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchEntry {
    pub name: String,
    pub description: String,
    pub latest_version: Version,
    pub keywords: Vec<String>,
    pub download_count: u64,
    pub updated_at: DateTime<Utc>,
}

/// Search result with relevance scoring.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub entry: SearchEntry,
    pub score: f64,
}

// ==================== Local Registry ====================

/// Local filesystem-backed package registry.
///
/// Packages are stored under a root directory with structure:
/// ```text
/// <root>/
///   index/
///     <package_name>.json     # Package metadata and versions
///   packages/
///     <package_name>/
///       <version>.tar.gz      # Package archive
///       <version>.json        # Manifest for this version
///   tokens/
///     <token_id>.json         # API tokens
///   users/
///     <user_id>.json          # User accounts
///   search_index.json         # Search index
/// ```
pub struct LocalRegistry {
    root: PathBuf,
    max_upload_size: u64,
}

impl LocalRegistry {
    /// Create or open a local registry at the given path.
    pub fn open(root: PathBuf) -> RegistryResult<Self> {
        // Ensure directory structure exists
        fs::create_dir_all(root.join("index"))?;
        fs::create_dir_all(root.join("packages"))?;
        fs::create_dir_all(root.join("tokens"))?;
        fs::create_dir_all(root.join("users"))?;

        Ok(Self {
            root,
            max_upload_size: 100 * 1024 * 1024, // 100 MB default
        })
    }

    /// Set the maximum upload size in bytes.
    pub fn with_max_upload_size(mut self, size: u64) -> Self {
        self.max_upload_size = size;
        self
    }

    // ---- Package Operations ----

    /// Publish a new package version.
    pub fn publish(
        &self,
        manifest: &PackageManifest,
        archive_data: &[u8],
        token: &ApiToken,
    ) -> RegistryResult<()> {
        // Auth check
        self.check_scope(token, &TokenScope::Publish)?;
        self.check_package_owner(&manifest.name, &token.user_id)?;

        // Size check
        if archive_data.len() as u64 > self.max_upload_size {
            return Err(RegistryError::UploadTooLarge {
                size: archive_data.len() as u64,
                limit: self.max_upload_size,
            });
        }

        // Verify checksum
        let computed = compute_checksum(archive_data);
        if computed != manifest.checksum {
            return Err(RegistryError::ChecksumMismatch {
                expected: manifest.checksum.clone(),
                actual: computed,
            });
        }

        // Check version doesn't already exist
        let mut record = self.load_or_create_record(&manifest.name)?;
        if record.versions.iter().any(|v| v.version == manifest.version) {
            return Err(RegistryError::VersionAlreadyExists(
                manifest.name.clone(),
                manifest.version.to_string(),
            ));
        }

        // Store the archive
        let pkg_dir = self.root.join("packages").join(&manifest.name);
        fs::create_dir_all(&pkg_dir)?;
        let archive_path = pkg_dir.join(format!("{}.tar.gz", manifest.version));
        fs::write(&archive_path, archive_data)?;

        // Store the manifest
        let manifest_path = pkg_dir.join(format!("{}.json", manifest.version));
        let manifest_json = serde_json::to_string_pretty(manifest)
            .map_err(|e| RegistryError::Serialization(e.to_string()))?;
        fs::write(&manifest_path, manifest_json)?;

        // Update the index
        let entry = VersionEntry {
            version: manifest.version.clone(),
            checksum: manifest.checksum.clone(),
            size_bytes: manifest.size_bytes,
            uploaded_at: Utc::now(),
            yanked: false,
            yanked_reason: None,
        };
        record.versions.push(entry);
        record.updated_at = Utc::now();
        self.save_record(&record)?;

        // Update search index
        self.update_search_index(
            &manifest.name,
            &manifest.description,
            &manifest.version,
            &manifest.keywords,
        )?;

        Ok(())
    }

    /// Download a specific package version.
    pub fn download(
        &self,
        name: &str,
        version: &Version,
    ) -> RegistryResult<Vec<u8>> {
        let record = self.load_record(name)?;

        let entry = record
            .versions
            .iter()
            .find(|v| &v.version == version && !v.yanked)
            .ok_or_else(|| RegistryError::VersionNotFound(name.to_string(), version.to_string()))?;

        let archive_path = self
            .root
            .join("packages")
            .join(name)
            .join(format!("{}.tar.gz", version));

        let data = fs::read(&archive_path)?;

        // Verify checksum
        let computed = compute_checksum(&data);
        if computed != entry.checksum {
            return Err(RegistryError::ChecksumMismatch {
                expected: entry.checksum.clone(),
                actual: computed,
            });
        }

        Ok(data)
    }

    /// List all versions of a package.
    pub fn list_versions(&self, name: &str) -> RegistryResult<Vec<VersionEntry>> {
        let record = self.load_record(name)?;
        Ok(record.versions)
    }

    /// Get the latest non-yanked version of a package.
    pub fn latest_version(&self, name: &str) -> RegistryResult<VersionEntry> {
        let record = self.load_record(name)?;
        record
            .versions
            .iter()
            .filter(|v| !v.yanked)
            .max_by_key(|v| &v.version)
            .cloned()
            .ok_or_else(|| RegistryError::PackageNotFound(name.to_string()))
    }

    /// Resolve a version requirement against available versions.
    pub fn resolve_version(
        &self,
        name: &str,
        version_req: &str,
    ) -> RegistryResult<Version> {
        let req = VersionReq::parse(version_req)
            .map_err(|e| RegistryError::InvalidSemver(e.to_string()))?;

        let record = self.load_record(name)?;
        record
            .versions
            .iter()
            .filter(|v| !v.yanked && req.matches(&v.version))
            .max_by_key(|v| &v.version)
            .map(|v| v.version.clone())
            .ok_or_else(|| {
                RegistryError::VersionNotFound(name.to_string(), version_req.to_string())
            })
    }

    /// Yank a package version (mark as not recommended for use).
    pub fn yank(
        &self,
        name: &str,
        version: &Version,
        reason: Option<String>,
        token: &ApiToken,
    ) -> RegistryResult<()> {
        self.check_scope(token, &TokenScope::Yank)?;
        self.check_package_owner(name, &token.user_id)?;

        let mut record = self.load_record(name)?;
        let entry = record
            .versions
            .iter_mut()
            .find(|v| v.version == *version && !v.yanked)
            .ok_or_else(|| RegistryError::VersionNotFound(name.to_string(), version.to_string()))?;

        entry.yanked = true;
        entry.yanked_reason = reason;
        record.updated_at = Utc::now();
        self.save_record(&record)?;

        Ok(())
    }

    /// Unyank a previously yanked version.
    pub fn unyank(
        &self,
        name: &str,
        version: &Version,
        token: &ApiToken,
    ) -> RegistryResult<()> {
        self.check_scope(token, &TokenScope::Yank)?;
        self.check_package_owner(name, &token.user_id)?;

        let mut record = self.load_record(name)?;
        let entry = record
            .versions
            .iter_mut()
            .find(|v| v.version == *version && v.yanked)
            .ok_or_else(|| RegistryError::VersionNotFound(name.to_string(), version.to_string()))?;

        entry.yanked = false;
        entry.yanked_reason = None;
        record.updated_at = Utc::now();
        self.save_record(&record)?;

        Ok(())
    }

    /// Fetch package metadata (manifest) for a specific version.
    pub fn get_manifest(
        &self,
        name: &str,
        version: &Version,
    ) -> RegistryResult<PackageManifest> {
        let manifest_path = self
            .root
            .join("packages")
            .join(name)
            .join(format!("{}.json", version));

        let content = fs::read_to_string(&manifest_path)?;
        let manifest: PackageManifest = serde_json::from_str(&content)
            .map_err(|e| RegistryError::Serialization(e.to_string()))?;
        Ok(manifest)
    }

    // ---- Search & Discovery ----

    /// Search packages by query string.
    pub fn search(&self, query: &str, limit: usize) -> RegistryResult<Vec<SearchResult>> {
        let index = self.load_search_index()?;
        let query_lower = query.to_lowercase();

        let mut results: Vec<SearchResult> = index
            .entries
            .iter()
            .filter(|entry| {
                entry.name.to_lowercase().contains(&query_lower)
                    || entry.description.to_lowercase().contains(&query_lower)
                    || entry
                        .keywords
                        .iter()
                        .any(|k| k.to_lowercase().contains(&query_lower))
            })
            .map(|entry| SearchResult {
                entry: entry.clone(),
                score: self.score_entry(entry, &query_lower),
            })
            .collect();

        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(Ordering::Equal));
        results.truncate(limit);
        Ok(results)
    }

    /// List all packages (optionally limited).
    pub fn list_packages(&self, limit: usize) -> RegistryResult<Vec<SearchEntry>> {
        let index = self.load_search_index()?;
        let mut entries = index.entries;
        entries.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
        entries.truncate(limit);
        Ok(entries)
    }

    // ---- Authentication ----

    /// Create a new user account.
    pub fn create_user(
        &self,
        username: &str,
        email: &str,
    ) -> RegistryResult<User> {
        let user_id = uuid::Uuid::new_v4().to_string();
        let user = User {
            user_id: user_id.clone(),
            username: username.to_string(),
            email: email.to_string(),
            created_at: Utc::now(),
            owned_packages: HashSet::new(),
        };

        let path = self.root.join("users").join(format!("{}.json", user_id));
        let json = serde_json::to_string_pretty(&user)
            .map_err(|e| RegistryError::Serialization(e.to_string()))?;
        fs::write(path, json)?;

        Ok(user)
    }

    /// Claim ownership of a package name for a user.
    pub fn claim_package(&self, user_id: &str, package_name: &str) -> RegistryResult<()> {
        let mut user = self.load_user(user_id)?;
        user.owned_packages.insert(package_name.to_string());
        let path = self
            .root
            .join("users")
            .join(format!("{}.json", user_id));
        let json = serde_json::to_string_pretty(&user)
            .map_err(|e| RegistryError::Serialization(e.to_string()))?;
        fs::write(path, json)?;
        Ok(())
    }

    /// Create an API token for a user.
    pub fn create_token(
        &self,
        user_id: &str,
        scopes: Vec<TokenScope>,
        expires_in_days: Option<u64>,
    ) -> RegistryResult<ApiToken> {
        // Verify user exists
        self.load_user(user_id)?;

        let token_id = uuid::Uuid::new_v4().to_string();
        let expires_at = expires_in_days.map(|d| Utc::now() + chrono::Duration::days(d as i64));

        let token = ApiToken {
            token_id: token_id.clone(),
            user_id: user_id.to_string(),
            scopes,
            created_at: Utc::now(),
            expires_at,
        };

        let path = self.root.join("tokens").join(format!("{}.json", token_id));
        let json = serde_json::to_string_pretty(&token)
            .map_err(|e| RegistryError::Serialization(e.to_string()))?;
        fs::write(path, json)?;

        Ok(token)
    }

    /// Validate an API token.
    pub fn validate_token(&self, token_id: &str) -> RegistryResult<ApiToken> {
        let path = self.root.join("tokens").join(format!("{}.json", token_id));
        let content = fs::read_to_string(&path)?;
        let token: ApiToken = serde_json::from_str(&content)
            .map_err(|e| RegistryError::Serialization(e.to_string()))?;

        if let Some(expires_at) = token.expires_at {
            if Utc::now() > expires_at {
                return Err(RegistryError::Unauthorized("Token expired".into()));
            }
        }

        Ok(token)
    }

    // ---- Internal Helpers ----

    fn load_or_create_record(&self, name: &str) -> RegistryResult<PackageRecord> {
        let path = self.root.join("index").join(format!("{}.json", name));
        if path.exists() {
            let content = fs::read_to_string(&path)?;
            let record: PackageRecord = serde_json::from_str(&content)
                .map_err(|e| RegistryError::Serialization(e.to_string()))?;
            Ok(record)
        } else {
            Ok(PackageRecord {
                name: name.to_string(),
                description: String::new(),
                versions: Vec::new(),
                created_at: Utc::now(),
                updated_at: Utc::now(),
            })
        }
    }

    fn load_record(&self, name: &str) -> RegistryResult<PackageRecord> {
        let path = self.root.join("index").join(format!("{}.json", name));
        if !path.exists() {
            return Err(RegistryError::PackageNotFound(name.to_string()));
        }
        let content = fs::read_to_string(&path)?;
        let record: PackageRecord = serde_json::from_str(&content)
            .map_err(|e| RegistryError::Serialization(e.to_string()))?;
        Ok(record)
    }

    fn save_record(&self, record: &PackageRecord) -> RegistryResult<()> {
        let path = self
            .root
            .join("index")
            .join(format!("{}.json", record.name));
        let json = serde_json::to_string_pretty(record)
            .map_err(|e| RegistryError::Serialization(e.to_string()))?;
        fs::write(path, json)?;
        Ok(())
    }

    fn load_user(&self, user_id: &str) -> RegistryResult<User> {
        let path = self.root.join("users").join(format!("{}.json", user_id));
        if !path.exists() {
            return Err(RegistryError::Unauthorized("User not found".into()));
        }
        let content = fs::read_to_string(&path)?;
        let user: User = serde_json::from_str(&content)
            .map_err(|e| RegistryError::Serialization(e.to_string()))?;
        Ok(user)
    }

    fn check_scope(&self, token: &ApiToken, required: &TokenScope) -> RegistryResult<()> {
        if !token.scopes.contains(required) {
            return Err(RegistryError::Forbidden(format!(
                "Token missing required scope: {:?}",
                required
            )));
        }
        Ok(())
    }

    fn check_package_owner(&self, package_name: &str, user_id: &str) -> RegistryResult<()> {
        let user = self.load_user(user_id)?;
        if !user.owned_packages.contains(package_name) {
            return Err(RegistryError::Forbidden(format!(
                "User {} does not own package {}",
                user.username, package_name
            )));
        }
        Ok(())
    }

    fn load_search_index(&self) -> RegistryResult<SearchIndex> {
        let path = self.root.join("search_index.json");
        if path.exists() {
            let content = fs::read_to_string(&path)?;
            let index: SearchIndex = serde_json::from_str(&content)
                .map_err(|e| RegistryError::Serialization(e.to_string()))?;
            Ok(index)
        } else {
            Ok(SearchIndex {
                entries: Vec::new(),
            })
        }
    }

    fn save_search_index(&self, index: &SearchIndex) -> RegistryResult<()> {
        let path = self.root.join("search_index.json");
        let json = serde_json::to_string_pretty(index)
            .map_err(|e| RegistryError::Serialization(e.to_string()))?;
        fs::write(path, json)?;
        Ok(())
    }

    fn update_search_index(
        &self,
        name: &str,
        description: &str,
        version: &Version,
        keywords: &[String],
    ) -> RegistryResult<()> {
        let mut index = self.load_search_index()?;

        if let Some(entry) = index.entries.iter_mut().find(|e| e.name == name) {
            entry.latest_version = version.clone();
            entry.description = description.to_string();
            entry.keywords = keywords.to_vec();
            entry.updated_at = Utc::now();
        } else {
            index.entries.push(SearchEntry {
                name: name.to_string(),
                description: description.to_string(),
                latest_version: version.clone(),
                keywords: keywords.to_vec(),
                download_count: 0,
                updated_at: Utc::now(),
            });
        }

        self.save_search_index(&index)?;
        Ok(())
    }

    fn score_entry(&self, entry: &SearchEntry, query: &str) -> f64 {
        let mut score = 0.0;

        // Exact name match is highest score
        if entry.name.to_lowercase() == query {
            score += 100.0;
        } else if entry.name.to_lowercase().starts_with(query) {
            score += 50.0;
        } else if entry.name.to_lowercase().contains(query) {
            score += 20.0;
        }

        // Description match
        if entry.description.to_lowercase().contains(query) {
            score += 10.0;
        }

        // Keyword match
        for kw in &entry.keywords {
            if kw.to_lowercase().contains(query) {
                score += 5.0;
            }
        }

        // Popularity boost
        score += (entry.download_count as f64).log10().min(10.0);

        score
    }
}

// ==================== Search Index ====================

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SearchIndex {
    entries: Vec<SearchEntry>,
}

// ==================== Remote Registry Client ====================

/// HTTP client for interacting with a remote Fusion package registry.
pub struct RemoteRegistryClient {
    base_url: String,
    auth_token: Option<String>,
    http_client: reqwest::Client,
}

impl RemoteRegistryClient {
    pub fn new(base_url: &str) -> Self {
        Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            auth_token: None,
            http_client: reqwest::Client::new(),
        }
    }

    pub fn with_auth_token(mut self, token: String) -> Self {
        self.auth_token = Some(token);
        self
    }

    /// Fetch package versions from the remote registry.
    pub async fn fetch_versions(&self, name: &str) -> RegistryResult<Vec<Version>> {
        let url = format!("{}/api/v1/packages/{}/versions", self.base_url, name);
        let resp = self
            .http_client
            .get(&url)
            .send()
            .await
            .map_err(|e| RegistryError::SearchIndex(e.to_string()))?;

        if !resp.status().is_success() {
            return Err(RegistryError::PackageNotFound(name.to_string()));
        }

        let versions: Vec<Version> = resp
            .json()
            .await
            .map_err(|e| RegistryError::Serialization(e.to_string()))?;
        Ok(versions)
    }

    /// Download a package archive from the remote registry.
    pub async fn download(
        &self,
        name: &str,
        version: &Version,
    ) -> RegistryResult<Vec<u8>> {
        let url = format!(
            "{}/api/v1/packages/{}/{}.tar.gz",
            self.base_url, name, version
        );
        let mut req = self.http_client.get(&url);

        if let Some(ref token) = self.auth_token {
            req = req.bearer_auth(token);
        }

        let resp = req
            .send()
            .await
            .map_err(|e| RegistryError::SearchIndex(e.to_string()))?;

        if !resp.status().is_success() {
            return Err(RegistryError::VersionNotFound(
                name.to_string(),
                version.to_string(),
            ));
        }

        let bytes = resp
            .bytes()
            .await
            .map_err(|e| RegistryError::SearchIndex(e.to_string()))?;
        Ok(bytes.to_vec())
    }

    /// Publish a package to the remote registry.
    pub async fn publish(
        &self,
        manifest: &PackageManifest,
        archive_data: &[u8],
    ) -> RegistryResult<()> {
        let url = format!("{}/api/v1/packages", self.base_url);
        let mut req = self
            .http_client
            .post(&url)
            .header("Content-Type", "application/octet-stream")
            .header("X-Package-Manifest", serde_json::to_string(manifest).unwrap_or_default())
            .body(archive_data.to_vec());

        if let Some(ref token) = self.auth_token {
            req = req.bearer_auth(token);
        }

        let resp = req
            .send()
            .await
            .map_err(|e| RegistryError::SearchIndex(e.to_string()))?;

        if !resp.status().is_success() {
            return Err(RegistryError::Forbidden(format!(
                "Publish failed with status: {}",
                resp.status()
            )));
        }

        Ok(())
    }

    /// Search packages on the remote registry.
    pub async fn search(
        &self,
        query: &str,
        limit: usize,
    ) -> RegistryResult<Vec<SearchResult>> {
        let encoded_query: String = query
            .bytes()
            .map(|b| match b {
                b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                    (b as char).to_string()
                }
                _ => format!("%{:02X}", b),
            })
            .collect();
        let url = format!(
            "{}/api/v1/search?q={}&limit={}",
            self.base_url, encoded_query, limit
        );

        let resp = self
            .http_client
            .get(&url)
            .send()
            .await
            .map_err(|e| RegistryError::SearchIndex(e.to_string()))?;

        if !resp.status().is_success() {
            return Err(RegistryError::SearchIndex(format!(
                "Search failed with status: {}",
                resp.status()
            )));
        }

        let results: Vec<SearchResult> = resp
            .json()
            .await
            .map_err(|e| RegistryError::Serialization(e.to_string()))?;
        Ok(results)
    }
}

// ==================== Utility ====================

/// Compute SHA-256 checksum of data.
pub fn compute_checksum(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hex::encode(hasher.finalize())
}

use std::cmp::Ordering;

// ==================== FFI Exports ====================

/// FFI-compatible registry handle.
#[no_mangle]
pub extern "C" fn fusion_registry_create(root: *const i8) -> *mut LocalRegistry {
    if root.is_null() {
        return std::ptr::null_mut();
    }

    unsafe {
        let root_str = std::ffi::CStr::from_ptr(root).to_string_lossy();
        match LocalRegistry::open(PathBuf::from(root_str.as_ref())) {
            Ok(reg) => Box::into_raw(Box::new(reg)),
            Err(_) => std::ptr::null_mut(),
        }
    }
}

#[no_mangle]
pub extern "C" fn fusion_registry_destroy(registry: *mut LocalRegistry) {
    if !registry.is_null() {
        unsafe {
            let _ = Box::from_raw(registry);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn setup_registry() -> (TempDir, LocalRegistry) {
        let dir = TempDir::new().unwrap();
        let registry = LocalRegistry::open(dir.path().join("registry")).unwrap();
        (dir, registry)
    }

    fn make_test_user(registry: &LocalRegistry) -> User {
        registry.create_user("testuser", "test@example.com").unwrap()
    }

    fn make_test_token(registry: &LocalRegistry, user: &User) -> ApiToken {
        registry
            .create_token(
                &user.user_id,
                vec![TokenScope::Publish, TokenScope::Yank],
                None,
            )
            .unwrap()
    }

    fn make_test_manifest(name: &str, version: &str) -> PackageManifest {
        let archive = b"archive";
        PackageManifest {
            name: name.to_string(),
            version: Version::parse(version).unwrap(),
            description: format!("Test package {}", name),
            authors: vec!["test".to_string()],
            license: Some("MIT".to_string()),
            repository: None,
            homepage: None,
            keywords: vec!["test".to_string()],
            categories: vec![],
            dependencies: vec![],
            checksum: compute_checksum(archive),
            size_bytes: archive.len() as u64,
            uploaded_at: Utc::now(),
            uploader_id: "test-user".to_string(),
        }
    }

    #[test]
    fn test_publish_and_download() {
        let (_dir, registry) = setup_registry();
        let user = make_test_user(&registry);
        let token = make_test_token(&registry, &user);

        registry.claim_package(&user.user_id, "test-pkg").unwrap();

        let archive = b"archive";
        let mut manifest = make_test_manifest("test-pkg", "1.0.0");
        manifest.uploader_id = user.user_id.clone();

        // Publish
        registry.publish(&manifest, archive, &token).unwrap();

        // Download
        let downloaded = registry
            .download("test-pkg", &Version::parse("1.0.0").unwrap())
            .unwrap();
        assert_eq!(downloaded, archive);
    }

    #[test]
    fn test_list_versions() {
        let (_dir, registry) = setup_registry();
        let user = make_test_user(&registry);
        let token = make_test_token(&registry, &user);

        registry.claim_package(&user.user_id, "multi-ver").unwrap();

        for ver in &["1.0.0", "1.1.0", "2.0.0"] {
            let mut manifest = make_test_manifest("multi-ver", ver);
            manifest.uploader_id = user.user_id.clone();
            registry.publish(&manifest, b"archive", &token).unwrap();
        }

        let versions = registry.list_versions("multi-ver").unwrap();
        assert_eq!(versions.len(), 3);
    }

    #[test]
    fn test_resolve_version() {
        let (_dir, registry) = setup_registry();
        let user = make_test_user(&registry);
        let token = make_test_token(&registry, &user);

        registry.claim_package(&user.user_id, "resolve-pkg").unwrap();

        for ver in &["1.0.0", "1.1.0", "1.2.0", "2.0.0"] {
            let mut manifest = make_test_manifest("resolve-pkg", ver);
            manifest.uploader_id = user.user_id.clone();
            registry.publish(&manifest, b"archive", &token).unwrap();
        }

        let resolved = registry.resolve_version("resolve-pkg", "^1.0.0").unwrap();
        assert_eq!(resolved, Version::parse("1.2.0").unwrap());
    }

    #[test]
    fn test_yank() {
        let (_dir, registry) = setup_registry();
        let user = make_test_user(&registry);
        let token = make_test_token(&registry, &user);

        registry.claim_package(&user.user_id, "yank-pkg").unwrap();

        let mut manifest = make_test_manifest("yank-pkg", "1.0.0");
        manifest.uploader_id = user.user_id.clone();
        registry.publish(&manifest, b"archive", &token).unwrap();

        // Yank
        registry
            .yank(
                "yank-pkg",
                &Version::parse("1.0.0").unwrap(),
                Some("Deprecated".into()),
                &token,
            )
            .unwrap();

        // Latest should fail since the only version is yanked
        assert!(registry.latest_version("yank-pkg").is_err());

        // Unyank
        registry
            .unyank("yank-pkg", &Version::parse("1.0.0").unwrap(), &token)
            .unwrap();

        // Now it should work
        assert!(registry.latest_version("yank-pkg").is_ok());
    }

    #[test]
    fn test_search() {
        let (_dir, registry) = setup_registry();
        let user = make_test_user(&registry);
        let token = make_test_token(&registry, &user);

        registry.claim_package(&user.user_id, "fusion-math").unwrap();

        let mut manifest = make_test_manifest("fusion-math", "1.0.0");
        manifest.description = "Math library for Fusion".to_string();
        manifest.keywords = vec!["math".into(), "numeric".into()];
        manifest.uploader_id = user.user_id.clone();
        registry.publish(&manifest, b"archive", &token).unwrap();

        let results = registry.search("math", 10).unwrap();
        assert!(!results.is_empty());
        assert!(results[0].entry.name.contains("math"));
    }

    #[test]
    fn test_auth_token_scopes() {
        let (_dir, registry) = setup_registry();
        let user = make_test_user(&registry);

        registry.claim_package(&user.user_id, "auth-test").unwrap();

        let read_only_token = registry
            .create_token(&user.user_id, vec![TokenScope::Read], None)
            .unwrap();

        let manifest = make_test_manifest("auth-test", "1.0.0");
        let result = registry.publish(&manifest, b"archive", &read_only_token);
        assert!(result.is_err());
    }

    #[test]
    fn test_checksum_mismatch() {
        let (_dir, registry) = setup_registry();
        let user = make_test_user(&registry);
        let token = make_test_token(&registry, &user);

        registry.claim_package(&user.user_id, "checksum-pkg").unwrap();

        let mut manifest = make_test_manifest("checksum-pkg", "1.0.0");
        manifest.checksum = "bad_checksum".to_string();
        manifest.uploader_id = user.user_id.clone();

        let result = registry.publish(&manifest, b"archive", &token);
        assert!(result.is_err());
    }

    #[test]
    fn test_compute_checksum() {
        let data = b"hello world";
        let checksum = compute_checksum(data);
        assert_eq!(checksum.len(), 64); // SHA-256 hex length
    }
}
