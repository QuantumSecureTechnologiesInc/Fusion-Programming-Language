use anyhow::{Context, Result};
use colored::Colorize;
use std::process::Command;

use crate::config::FusionConfig;

pub async fn run(check: bool, json: bool) -> Result<()> {
    let root = FusionConfig::project_root().unwrap_or_else(|_| std::env::current_dir().unwrap());

    let mut args = vec!["fmt".to_string()];
    if check {
        args.push("--check".to_string());
    }

    let output = Command::new("fuc.exe")
        .args(&args)
        .current_dir(&root)
        .output()
        .context("Failed to execute fuc.exe fmt")?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let success = output.status.success();

    if json {
        let result = serde_json::json!({
            "success": success,
            "check_only": check,
            "output": stdout.trim().to_string(),
        });
        println!("{}", serde_json::to_string_pretty(&result)?);
    } else if check {
        if success {
            println!("{} All files are formatted correctly.", "OK".green().bold());
        } else {
            println!("{} Some files need formatting. Run `fusion fmt` to fix.", "check:".yellow().bold());
            if !stdout.is_empty() {
                println!("{}", stdout);
            }
        }
    } else {
        if success {
            println!("{} Code formatted successfully.", "OK".green().bold());
        } else {
            anyhow::bail!("Formatting failed: {}", stderr);
        }
    }

    if !success {
        anyhow::bail!("fmt command failed");
    }

    Ok(())
}
