use anyhow::{Context, Result};
use colored::Colorize;
use std::fs;
use std::process::Command;

use crate::commands::PkgCommands;
use crate::config::FusionConfig;

const REGISTRY_URL: &str = "https://packages.fusion-lang.org";

pub async fn run(command: PkgCommands, json: bool) -> Result<()> {
    match command {
        PkgCommands::Add { package } => add(&package, json).await,
        PkgCommands::Remove { package } => remove(&package, json).await,
        PkgCommands::Search { query } => search(&query, json).await,
        PkgCommands::Publish { dry_run } => publish(dry_run, json).await,
        PkgCommands::Update { package } => update(package.as_deref(), json).await,
    }
}

fn read_fusion_toml() -> Result<toml::Value> {
    let root = FusionConfig::project_root()?;
    let path = root.join("Fusion.toml");
    let content = fs::read_to_string(&path)
        .with_context(|| format!("Failed to read {}", path.display()))?;
    let value: toml::Value = toml::from_str(&content)
        .context("Failed to parse Fusion.toml")?;
    Ok(value)
}

fn write_fusion_toml(value: &toml::Value) -> Result<()> {
    let root = FusionConfig::project_root()?;
    let path = root.join("Fusion.toml");
    let content = toml::to_string_pretty(value)
        .context("Failed to serialize Fusion.toml")?;
    fs::write(&path, content)
        .with_context(|| format!("Failed to write {}", path.display()))?;
    Ok(())
}

async fn add(package: &str, json: bool) -> Result<()> {
    let mut config = read_fusion_toml()?;

    let (name, version) = if let Some(idx) = package.find('@') {
        (&package[..idx], Some(&package[idx + 1..]))
    } else {
        (package, None)
    };

    let deps = config
        .as_table_mut()
        .and_then(|t| t.get_mut("dependencies"))
        .and_then(|v| v.as_table_mut());

    match deps {
        Some(deps) => {
            deps.insert(name.to_string(), toml::Value::String(
                version.unwrap_or("*").to_string(),
            ));
        }
        None => {
            let mut table = toml::Table::new();
            table.insert(name.to_string(), toml::Value::String(
                version.unwrap_or("*").to_string(),
            ));
            config
                .as_table_mut()
                .unwrap()
                .insert("dependencies".to_string(), toml::Value::Table(table));
        }
    }

    write_fusion_toml(&config)?;

    println!("{} {} {}",
        "Added".green().bold(),
        name.yellow(),
        version.map(|v| format!("v{}", v)).unwrap_or_default()
    );

    // Run fuc.exe pkg install to fetch
    let root = FusionConfig::project_root()?;
    let status = Command::new("fuc.exe")
        .args(["pkg", "install"])
        .current_dir(&root)
        .status()
        .context("Failed to run fuc.exe pkg install")?;

    if json {
        let result = serde_json::json!({
            "action": "add",
            "package": name,
            "version": version.unwrap_or("*"),
            "install_success": status.success(),
        });
        println!("{}", serde_json::to_string_pretty(&result)?);
    }

    Ok(())
}

async fn remove(package: &str, json: bool) -> Result<()> {
    let mut config = read_fusion_toml()?;

    let removed = config
        .as_table_mut()
        .and_then(|t| t.get_mut("dependencies"))
        .and_then(|v| v.as_table_mut())
        .and_then(|deps| deps.remove(package));

    if removed.is_none() {
        anyhow::bail!("Package '{}' not found in dependencies", package);
    }

    write_fusion_toml(&config)?;

    if json {
        let result = serde_json::json!({ "action": "remove", "package": package });
        println!("{}", serde_json::to_string_pretty(&result)?);
    } else {
        println!("{} {}", "Removed".green().bold(), package.yellow());
    }

    Ok(())
}

async fn search(query: &str, json: bool) -> Result<()> {
    let client = reqwest::Client::new();
    let url = format!("{}/api/search?q={}", REGISTRY_URL, query);

    let resp = client
        .get(&url)
        .send()
        .await
        .context("Failed to connect to package registry")?;

    let body: serde_json::Value = resp.json().await
        .context("Invalid response from registry")?;

    if json {
        println!("{}", serde_json::to_string_pretty(&body)?);
    } else {
        println!("{} Results for '{}' :", "Pkg".cyan().bold(), query);
        if let Some(packages) = body.as_array() {
            for pkg in packages {
                let name = pkg["name"].as_str().unwrap_or("?");
                let ver = pkg["version"].as_str().unwrap_or("?");
                let desc = pkg["description"].as_str().unwrap_or("");
                println!("  {} v{} - {}", name.green(), ver.dimmed(), desc);
            }
        } else if let Some(pkgs) = body["packages"].as_array() {
            for pkg in pkgs {
                let name = pkg["name"].as_str().unwrap_or("?");
                let ver = pkg["version"].as_str().unwrap_or("?");
                let desc = pkg["description"].as_str().unwrap_or("");
                println!("  {} v{} - {}", name.green(), ver.dimmed(), desc);
            }
        }
    }

    Ok(())
}

async fn publish(dry_run: bool, json: bool) -> Result<()> {
    let root = FusionConfig::project_root()?;
    let config = read_fusion_toml()?;

    let name = config.get("package")
        .and_then(|p| p.get("name"))
        .and_then(|n| n.as_str())
        .context("Package name not set in Fusion.toml [package] section")?;

    let version = config.get("package")
        .and_then(|p| p.get("version"))
        .and_then(|v| v.as_str())
        .unwrap_or("0.1.0");

    let mut args = vec!["pkg".to_string(), "publish".to_string()];
    if dry_run {
        args.push("--dry-run".to_string());
    }

    let output = Command::new("fuc.exe")
        .args(&args)
        .current_dir(&root)
        .output()
        .context("Failed to execute fuc.exe pkg publish")?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let success = output.status.success();

    if json {
        let result = serde_json::json!({
            "action": "publish",
            "package": name,
            "version": version,
            "dry_run": dry_run,
            "success": success,
            "output": stdout.trim().to_string(),
        });
        println!("{}", serde_json::to_string_pretty(&result)?);
    } else {
        if success {
            if dry_run {
                println!("{} {} v{} (dry run)", "Published".green().bold(), name, version);
            } else {
                println!("{} {} v{}", "Published".green().bold(), name.yellow(), version);
            }
        } else {
            eprintln!("{} {}", "error:".red().bold(), stderr);
            anyhow::bail!("Publish failed");
        }
    }

    Ok(())
}

async fn update(package: Option<&str>, json: bool) -> Result<()> {
    let root = FusionConfig::project_root()?;

    let mut args = vec!["pkg".to_string(), "update".to_string()];
    if let Some(pkg) = package {
        args.push(pkg.to_string());
    }

    let status = Command::new("fuc.exe")
        .args(&args)
        .current_dir(&root)
        .status()
        .context("Failed to execute fuc.exe pkg update")?;

    if json {
        let result = serde_json::json!({
            "action": "update",
            "package": package.unwrap_or("all"),
            "success": status.success(),
        });
        println!("{}", serde_json::to_string_pretty(&result)?);
    } else {
        if status.success() {
            println!("{} Dependencies updated.", "OK".green().bold());
        } else {
            anyhow::bail!("Update failed");
        }
    }

    Ok(())
}
