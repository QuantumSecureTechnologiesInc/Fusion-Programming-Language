use anyhow::{Context, Result};
use serde::Deserialize;
use std::path::{Path, PathBuf};

/// Top-level Fusion.toml configuration
#[derive(Debug, Deserialize, Default)]
pub struct FusionConfig {
    pub package: Option<PackageConfig>,
    pub build: Option<BuildConfig>,
    pub runtime: Option<RuntimeConfig>,
    pub ai: Option<AiConfig>,
    pub quantum: Option<QuantumConfig>,
    pub deploy: Option<DeployConfig>,
    pub sentinel: Option<SentinelConfig>,
    pub test: Option<TestConfig>,
}

#[derive(Debug, Deserialize, Default)]
pub struct PackageConfig {
    pub name: Option<String>,
    pub version: Option<String>,
    #[serde(rename = "type")]
    pub pkg_type: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
pub struct BuildConfig {
    pub optimization_level: Option<u8>,
    pub debug_info: Option<bool>,
    pub incremental: Option<bool>,
    pub lto: Option<bool>,
}

#[derive(Debug, Deserialize, Default)]
pub struct RuntimeConfig {
    pub profile: Option<String>,
    pub target: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
pub struct AiConfig {
    pub provider: Option<String>,
    pub default_device: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
pub struct QuantumConfig {
    pub default_backend: Option<String>,
    pub max_qubits: Option<u32>,
    pub shots: Option<u32>,
}

#[derive(Debug, Deserialize, Default)]
pub struct DeployConfig {
    pub target: Option<String>,
    pub registry: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
pub struct SentinelConfig {
    pub enabled: Option<bool>,
    pub mode: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
pub struct TestConfig {
    pub parallel: Option<bool>,
    pub timeout: Option<String>,
}

impl FusionConfig {
    /// Load config from the current directory or ancestors
    pub fn load() -> Result<Self> {
        let path = Self::find_config_file()?;
        Self::load_from(&path)
    }

    /// Load config from a specific path
    pub fn load_from(path: &Path) -> Result<Self> {
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read {}", path.display()))?;
        let config: FusionConfig = toml::from_str(&content)
            .with_context(|| format!("Failed to parse {}", path.display()))?;
        Ok(config)
    }

    /// Find Fusion.toml by walking up from cwd
    pub fn find_config_file() -> Result<PathBuf> {
        let mut dir = std::env::current_dir().context("Failed to get current directory")?;
        loop {
            let candidate = dir.join("Fusion.toml");
            if candidate.exists() {
                return Ok(candidate);
            }
            if !dir.pop() {
                break;
            }
        }
        anyhow::bail!(
            "No Fusion.toml found. Run `fusion init` to create a new project, \
             or run this command inside a Fusion project directory."
        )
    }

    /// Get the project root directory (parent of Fusion.toml)
    pub fn project_root() -> Result<PathBuf> {
        let config_path = Self::find_config_file()?;
        Ok(config_path
            .parent()
            .context("Invalid config path")?
            .to_path_buf())
    }
}
