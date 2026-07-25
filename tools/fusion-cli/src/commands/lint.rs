use anyhow::{Context, Result};
use colored::Colorize;
use std::process::Command;

use crate::config::FusionConfig;

pub async fn run(fix: bool, json: bool) -> Result<()> {
    let root = FusionConfig::project_root().unwrap_or_else(|_| std::env::current_dir().unwrap());

    let mut args = vec!["lint".to_string()];
    if fix {
        args.push("--fix".to_string());
    }

    let output = Command::new("fuc.exe")
        .args(&args)
        .current_dir(&root)
        .output()
        .context("Failed to execute fuc.exe lint")?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let success = output.status.success();

    if json {
        let result = serde_json::json!({
            "success": success,
            "fix_mode": fix,
            "output": stdout.trim().to_string(),
        });
        println!("{}", serde_json::to_string_pretty(&result)?);
    } else {
        if success {
            println!("{} No lint issues found.", "OK".green().bold());
        } else if fix {
            println!("{} Lint issues fixed.", "fixed!".yellow().bold());
        } else {
            println!("{} Lint issues found:", "WARN".yellow().bold());
        }
        if !stdout.is_empty() {
            println!("{}", stdout);
        }
        if !stderr.is_empty() {
            eprintln!("{}", stderr);
        }
    }

    if !success {
        anyhow::bail!("Lint found issues");
    }

    Ok(())
}
