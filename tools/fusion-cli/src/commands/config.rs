use anyhow::{Context, Result};
use colored::Colorize;

use super::ConfigCommands;
use crate::config::FusionConfig;

pub async fn run(command: ConfigCommands, json: bool) -> Result<()> {
    match command {
        ConfigCommands::Get { key } => config_get(&key, json),
        ConfigCommands::Set { key, value } => config_set(&key, &value, json),
        ConfigCommands::List => config_list(json),
        ConfigCommands::Validate => config_validate(json),
        ConfigCommands::Show => config_show(json),
    }
}

fn config_get(key: &str, json: bool) -> Result<()> {
    let config = FusionConfig::load()?;

    let parts: Vec<&str> = key.split('.').collect();
    let section = parts.first().context("Key must start with a section name")?;
    let field = parts.get(1);

    let value = config
        .get(*section)
        .with_context(|| format!("[{}] section not found in Fusion.toml", section))?;

    let result = if let Some(f) = field {
        value.get(*f).with_context(|| format!("{}.{} not found", section, f))?
    } else {
        value
    };

    if json {
        println!(
            "{}",
            serde_json::json!({"key": key, "value": result}).to_string()
        );
    } else {
        println!("{} = {}", key, result);
    }

    Ok(())
}

fn config_set(key: &str, value: &str, json: bool) -> Result<()> {
    let config_path = FusionConfig::find_config_file()?;
    let mut content =
        std::fs::read_to_string(&config_path).context("Failed to read Fusion.toml")?;

    let parts: Vec<&str> = key.split('.').collect();
    let section = parts.first().context("Key must start with a section name")?;
    let field = parts.get(1).context("Key must be in format 'section.field'")?;

    let new_line = format!("{} = {}", field, value);

    if !content.contains(&format!("[{}]", section)) {
        // Append new section
        content.push_str(&format!("\n[{}]\n{}\n", section, new_line));
    } else {
        // Find and replace the existing key
        let mut lines: Vec<String> = content.lines().map(String::from).collect();
        let mut in_section = false;
        let mut inserted = false;

        for i in 0..lines.len() {
            if lines[i].trim() == format!("[{}]", section) {
                in_section = true;
            } else if lines[i].trim().starts_with('[') && in_section {
                if !inserted {
                    lines.insert(i, new_line.clone());
                    inserted = true;
                }
                in_section = false;
            } else if in_section && lines[i].trim().starts_with(field) && !inserted {
                lines[i] = new_line.clone();
                inserted = true;
            }
        }

        if in_section && !inserted {
            lines.push(new_line.clone());
        }

        content = lines.join("\n");
    }

    std::fs::write(&config_path, &content).context("Failed to write Fusion.toml")?;

    if json {
        println!(
            "{}",
            serde_json::json!({"action": "set", "key": key, "value": value, "status": "ok"})
                .to_string()
        );
    } else {
        println!("{} {}", "OK:".green().bold(), format!("{} = {}", key, value));
    }

    Ok(())
}

fn config_list(json: bool) -> Result<()> {
    let config = FusionConfig::load()?;

    let sections = [
        ("package", "Package metadata (name, version, description, type)"),
        ("build", "Build settings (optimization_level, debug_info, lto, incremental)"),
        ("runtime", "Runtime configuration (profile, target)"),
        ("features", "Feature flags for conditional compilation"),
        ("test", "Test settings (parallel, timeout, coverage)"),
        ("deploy", "Deployment settings (target, registry, kubernetes, docker)"),
        ("ai", "AI provider settings (provider, ollama, mistral, deepseek)"),
        ("quantum", "Quantum computing settings (backend, aws_braket, ibm_quantum)"),
    ];

    if json {
        let list: Vec<_> = sections
            .iter()
            .filter(|(name, _)| config.get(*name).is_some())
            .map(|(name, desc)| serde_json::json!({"name": name, "description": desc}))
            .collect();
        println!("{}", serde_json::to_string_pretty(&list)?);
    } else {
        let config_path = FusionConfig::find_config_file()?;
        println!("Config: {}", config_path.display());
        println!();
        println!("Sections:");
        for (name, desc) in &sections {
            if config.get(*name).is_some() {
                println!("  [{}] - {}", name.cyan(), desc);
            }
        }
        println!();
        println!("Use 'fusion config get <section.field>' to view a value");
        println!("Use 'fusion config set <section.field> <value>' to set a value");
    }

    Ok(())
}

fn config_validate(json: bool) -> Result<()> {
    let config_path = FusionConfig::find_config_file()?;
    let mut errors: Vec<String> = Vec::new();
    let mut warnings: Vec<String> = Vec::new();

    match FusionConfig::load_from(&config_path) {
        Ok(config) => {
            // Validate [package] section
            if let Some(pkg) = config.get("package") {
                if pkg.get("name").and_then(|v| v.as_str()).unwrap_or("").is_empty() {
                    errors.push("[package] name is required".to_string());
                }
                if pkg.get("version").is_none() {
                    warnings.push("[package] version not set, defaulting to 0.1.0".to_string());
                }
            } else {
                errors.push("[package] section is required".to_string());
            }

            // Validate [build] section
            if let Some(build) = config.get("build") {
                if let Some(opt) = build.get("optimization_level").and_then(|v| v.as_integer()) {
                    if opt > 3 {
                        errors.push(format!("[build] optimization_level must be 0-3, got {}", opt));
                    }
                }
            }

            // Validate [quantum] section
            if let Some(quantum) = config.get("quantum") {
                if let Some(shots) = quantum.get("shots").and_then(|v| v.as_integer()) {
                    if shots == 0 {
                        errors.push("[quantum] shots must be greater than 0".to_string());
                    }
                    if shots > 100_000 {
                        warnings.push(format!(
                            "[quantum] shots value {} is very high, may cause slow execution",
                            shots
                        ));
                    }
                }
            }
        }
        Err(e) => {
            errors.push(format!("Failed to parse Fusion.toml: {}", e));
        }
    }

    let valid = errors.is_empty();

    if json {
        println!(
            "{}",
            serde_json::json!({
                "valid": valid,
                "errors": errors,
                "warnings": warnings
            })
            .to_string()
        );
    } else {
        if valid {
            println!("{} {}", "OK:".green().bold(), "Fusion.toml is valid");
        } else {
            println!("{} {}", "FAIL:".red().bold(), "Fusion.toml has errors");
        }

        if !errors.is_empty() {
            println!();
            println!("Errors:");
            for err in &errors {
                println!("  {} {}", "x".red(), err);
            }
        }

        if !warnings.is_empty() {
            println!();
            println!("Warnings:");
            for warn in &warnings {
                println!("  {} {}", "!".yellow(), warn);
            }
        }
    }

    if !valid {
        anyhow::bail!("Configuration validation failed");
    }

    Ok(())
}

fn config_show(json: bool) -> Result<()> {
    let config_path = FusionConfig::find_config_file()?;
    let content =
        std::fs::read_to_string(&config_path).context("Failed to read Fusion.toml")?;

    if json {
        let config = FusionConfig::load()?;
        println!(
            "{}",
            serde_json::to_string_pretty(&config)?
        );
    } else {
        println!("Config: {}", config_path.display());
        println!();
        println!("{}", content);
    }

    Ok(())
}
