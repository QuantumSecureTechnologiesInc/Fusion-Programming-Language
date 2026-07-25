use anyhow::{Context, Result};
use colored::Colorize;
use indicatif::ProgressBar;
use std::process::Command;

use crate::config::FusionConfig;

pub async fn run(filter: Option<&str>, parallel: bool, json: bool) -> Result<()> {
    let root = FusionConfig::project_root().unwrap_or_else(|_| std::env::current_dir().unwrap());

    let pb = ProgressBar::new_spinner();
    pb.set_message("Running tests...");
    pb.enable_steady_tick(std::time::Duration::from_millis(100));

    let mut args = vec!["test".to_string()];
    if let Some(f) = filter {
        args.push(format!("--filter={}", f));
    }
    if parallel {
        args.push("--parallel".to_string());
    }

    let output = Command::new("fuc.exe")
        .args(&args)
        .current_dir(&root)
        .output()
        .context("Failed to execute fuc.exe test")?;

    pb.finish_and_clear();

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let success = output.status.success();

    if json {
        let result = serde_json::json!({
            "success": success,
            "output": stdout.trim().to_string(),
            "filter": filter.unwrap_or(""),
            "parallel": parallel,
        });
        println!("{}", serde_json::to_string_pretty(&result)?);
    } else {
        if success {
            println!("{} Tests passed!", "OK".green().bold());
        } else {
            println!("{} Tests failed!", "FAIL".red().bold());
        }
        if !stdout.is_empty() {
            println!("{}", stdout);
        }
        if !stderr.is_empty() {
            eprintln!("{}", stderr);
        }
    }

    if !success {
        anyhow::bail!("Tests failed");
    }

    Ok(())
}
