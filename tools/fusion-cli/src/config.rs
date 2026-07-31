use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

/// Represents the Fusion.toml configuration file.
pub struct FusionConfig;

impl FusionConfig {
    /// Find the Fusion.toml config file by walking up from the current directory.
    pub fn find_config_file() -> Result<PathBuf> {
        let current = std::env::current_dir().context("Failed to get current directory")?;
        Self::find_config_in(&current)
    }

    /// Find Fusion.toml starting from the given directory and walking up.
    fn find_config_in(dir: &Path) -> Result<PathBuf> {
        let candidates = ["Fusion.toml", "fusion.toml"];
        for name in &candidates {
            let path = dir.join(name);
            if path.exists() {
                return Ok(path);
            }
        }

        if let Some(parent) = dir.parent() {
            Self::find_config_in(parent)
        } else {
            anyhow::bail!(
                "No Fusion.toml found. Run `fusion init` to create a new project."
            )
        }
    }

    /// Convenience: load from the project's Fusion.toml.
    pub fn load() -> Result<toml::Value> {
        let path = Self::find_config_file()?;
        Self::load_from(path)
    }

    /// Load and parse the Fusion.toml at the given path.
    pub fn load_from(path: impl AsRef<Path>) -> Result<toml::Value> {
        let path = path.as_ref();
        let content =
            std::fs::read_to_string(path).with_context(|| format!("Failed to read {}", path.display()))?;
        let value: toml::Value =
            toml::from_str(&content).with_context(|| format!("Failed to parse {}", path.display()))?;
        Ok(value)
    }

    /// Return the project root directory (parent of Fusion.toml).
    pub fn project_root() -> Result<PathBuf> {
        let config = Self::find_config_file()?;
        config
            .parent()
            .map(|p| p.to_path_buf())
            .context("Invalid config path")
    }
}
