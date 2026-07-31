use anyhow::{Context, Result};
use colored::Colorize;
use indicatif::ProgressBar;
use std::process::Command;

use crate::config::FusionConfig;

pub async fn run(target: &str, release: bool, verbose: bool, json: bool) -> Result<()> {
    let config = FusionConfig::load().unwrap_or_else(|_| toml::Value::Table(toml::map::Map::new()));
    let root = FusionConfig::project_root().unwrap_or_else(|_| std::env::current_dir().unwrap());

    let pb = ProgressBar::new_spinner();
    pb.set_message("Building project...");
    pb.enable_steady_tick(std::time::Duration::from_millis(100));

    let build_dir = root.join("build");
    std::fs::create_dir_all(&build_dir).context("Failed to create build directory")?;

    let mut args = vec![
        "build".to_string(),
        format!("--target={}", target),
    ];
    if release {
        args.push("--release".to_string());
    }
    if verbose {
        args.push("--verbose".to_string());
    }

    // Read build options from config if available
    if let Some(build_cfg) = config.get("build") {
        if let Some(opt) = build_cfg.get("optimization_level").and_then(|v| v.as_integer()) {
            args.push(format!("--opt-level={}", opt));
        }
        if let Some(true) = build_cfg.get("lto").and_then(|v| v.as_bool()) {
            args.push("--lto".to_string());
        }
        if let Some(true) = build_cfg.get("debug_info").and_then(|v| v.as_bool()) {
            args.push("--debug".to_string());
        }
    }

    let output = Command::new("fuc.exe")
        .args(&args)
        .current_dir(&root)
        .output()
        .context("Failed to execute fuc.exe build")?;

    pb.finish_and_clear();

    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);

    if !output.status.success() {
        if json {
            let result = serde_json::json!({
                "success": false,
                "error": stderr.trim().to_string(),
                "target": target,
                "release": release,
            });
            println!("{}", serde_json::to_string_pretty(&result)?);
        } else {
            eprintln!("{} Build failed:", "error:".red().bold());
            eprintln!("{}", stderr);
        }
        anyhow::bail!("Build failed with exit code {:?}", output.status.code());
    }

    if json {
        let result = serde_json::json!({
            "success": true,
            "target": target,
            "release": release,
            "build_dir": build_dir.display().to_string(),
        });
        println!("{}", serde_json::to_string_pretty(&result)?);
    } else {
        println!("{} {}", "Build successful!".green().bold(), format!("({})", target).dimmed());
        if !stdout.is_empty() {
            println!("{}", stdout);
        }
        println!("  Output: {}", build_dir.display());
        if release {
            println!("  Mode:   {}", "release".yellow());
        }
    }

    Ok(())
}
