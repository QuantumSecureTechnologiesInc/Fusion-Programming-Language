use anyhow::{Context, Result};
use colored::Colorize;

use crate::config::FusionConfig;

pub async fn run(cache: bool, json: bool) -> Result<()> {
    let root = FusionConfig::project_root().unwrap_or_else(|_| std::env::current_dir().unwrap());

    let mut removed = vec![];

    // Remove build directory
    let build_dir = root.join("build");
    if build_dir.exists() {
        std::fs::remove_dir_all(&build_dir).context("Failed to remove build directory")?;
        removed.push("build/");
    }

    // Remove target directory (for native builds)
    let target_dir = root.join("target");
    if target_dir.exists() {
        std::fs::remove_dir_all(&target_dir).context("Failed to remove target directory")?;
        removed.push("target/");
    }

    if cache {
        let cache_dir = root.join(".fusion-cache");
        if cache_dir.exists() {
            std::fs::remove_dir_all(&cache_dir).context("Failed to remove cache")?;
            removed.push(".fusion-cache/");
        }

        // Global cache
        if let Some(home) = dirs_next::home_dir() {
            let global_cache = home.join(".fusion").join("cache");
            if global_cache.exists() {
                std::fs::remove_dir_all(&global_cache).ok();
                removed.push("~/.fusion/cache/");
            }
        }
    }

    if json {
        let result = serde_json::json!({
            "action": "clean",
            "removed": removed,
            "cache": cache,
        });
        println!("{}", serde_json::to_string_pretty(&result)?);
    } else {
        if removed.is_empty() {
            println!("{} Nothing to clean.", "OK".green().bold());
        } else {
            println!("{} Cleaned:", "OK".green().bold());
            for dir in &removed {
                println!("  - {}", dir);
            }
        }
    }

    Ok(())
}
