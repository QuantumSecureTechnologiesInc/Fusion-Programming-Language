use anyhow::{Context, Result};
use colored::Colorize;
use std::process::Command;

use crate::config::FusionConfig;

pub async fn run(open: bool, json: bool) -> Result<()> {
    let root = FusionConfig::project_root().unwrap_or_else(|_| std::env::current_dir().unwrap());

    let mut args = vec!["doc".to_string()];
    if open {
        args.push("--open".to_string());
    }

    let output = Command::new("fuc.exe")
        .args(&args)
        .current_dir(&root)
        .output()
        .context("Failed to execute fuc.exe doc")?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let success = output.status.success();

    if json {
        let result = serde_json::json!({
            "action": "doc",
            "open": open,
            "success": success,
            "output": stdout.trim().to_string(),
        });
        println!("{}", serde_json::to_string_pretty(&result)?);
    } else {
        if success {
            println!("{} Documentation generated.", "OK".green().bold());
            let doc_dir = root.join("docs");
            if doc_dir.exists() {
                println!("  Output: {}", doc_dir.display());
            }
            if open {
                println!("  Opening in browser...");
            }
        } else {
            eprintln!("{} {}", "error:".red().bold(), stderr);
            anyhow::bail!("Documentation generation failed");
        }
    }

    Ok(())
}
