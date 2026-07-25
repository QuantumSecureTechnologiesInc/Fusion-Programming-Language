use anyhow::{Context, Result};
use colored::Colorize;
use std::path::PathBuf;
use std::process::Command;

use crate::config::FusionConfig;
use super::NeuralSealSubcommand;

pub async fn run(subcmd: NeuralSealSubcommand, json: bool) -> Result<()> {
    match subcmd {
        NeuralSealSubcommand::Keygen { level } => keygen(&level, json).await,
        NeuralSealSubcommand::Encrypt { key, input, output } => {
            encrypt(&key, &input, output.as_deref(), json).await
        }
        NeuralSealSubcommand::Decrypt { key, input, output } => {
            decrypt(&key, &input, output.as_deref(), json).await
        }
        NeuralSealSubcommand::Sign { key, input } => sign(&key, &input, json).await,
        NeuralSealSubcommand::Verify {
            key,
            input,
            signature,
        } => verify(&key, &input, &signature, json).await,
    }
}

fn resolve_path(path: &str) -> PathBuf {
    let p = PathBuf::from(path);
    if p.is_absolute() {
        p
    } else {
        std::env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .join(p)
    }
}

fn find_fuc() -> Result<String> {
    // Check for fuc.exe in PATH or common locations
    if let Ok(path) = which::which("fuc.exe") {
        return Ok(path.display().to_string());
    }
    if let Ok(path) = which::which("fuc") {
        return Ok(path.display().to_string());
    }
    // Fall back to fuc.exe name (will fail at exec time if not found)
    Ok("fuc.exe".to_string())
}

async fn keygen(level: &str, json: bool) -> Result<()> {
    let root = FusionConfig::project_root().unwrap_or_else(|_| std::env::current_dir().unwrap());
    let fuc = find_fuc()?;

    let args = vec![
        "neuralseal".to_string(),
        "keygen".to_string(),
        format!("--level={}", level),
    ];

    let output = Command::new(&fuc)
        .args(&args)
        .current_dir(&root)
        .output()
        .context("Failed to execute fuc.exe neuralseal keygen")?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let success = output.status.success();

    if json {
        let result = serde_json::json!({
            "action": "neuralseal_keygen",
            "level": level,
            "success": success,
            "output": stdout.trim().to_string(),
        });
        println!("{}", serde_json::to_string_pretty(&result)?);
    } else {
        if success {
            println!(
                "{} NeuralSeal keypair generated (level: {})",
                "OK".green().bold(),
                level.yellow()
            );
            let key_dir = root.join("keys");
            if key_dir.exists() {
                println!("  Keys: {}", key_dir.display());
            }
            if !stdout.is_empty() {
                println!("{}", stdout);
            }
        } else {
            eprintln!("{} {}", "error:".red().bold(), stderr);
            anyhow::bail!("Neuralseal keygen failed");
        }
    }

    Ok(())
}

async fn encrypt(key: &str, input: &str, output: Option<&str>, json: bool) -> Result<()> {
    let key_path = resolve_path(key);
    let input_path = resolve_path(input);

    if !key_path.exists() {
        anyhow::bail!("Key file not found: {}", key_path.display());
    }
    if !input_path.exists() {
        anyhow::bail!("Input file not found: {}", input_path.display());
    }

    let output_path = match output {
        Some(o) => resolve_path(o),
        None => {
            let ext = input_path
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("");
            if ext == "nsenc" {
                input_path.with_extension("nsdec")
            } else {
                let mut p = input_path.clone();
                p.set_extension(format!("{}.nsenc", ext));
                p
            }
        }
    };

    let fuc = find_fuc()?;
    let root = FusionConfig::project_root().unwrap_or_else(|_| std::env::current_dir().unwrap());

    let args = vec![
        "neuralseal".to_string(),
        "encrypt".to_string(),
        format!("--key={}", key_path.display()),
        format!("--input={}", input_path.display()),
        format!("--output={}", output_path.display()),
    ];

    let cmd_output = Command::new(&fuc)
        .args(&args)
        .current_dir(&root)
        .output()
        .context("Failed to execute fuc.exe neuralseal encrypt")?;

    let stdout = String::from_utf8_lossy(&cmd_output.stdout);
    let stderr = String::from_utf8_lossy(&cmd_output.stderr);
    let success = cmd_output.status.success();

    if json {
        let result = serde_json::json!({
            "action": "neuralseal_encrypt",
            "key": key,
            "input": input_path.display().to_string(),
            "output": output_path.display().to_string(),
            "success": success,
            "output_text": stdout.trim().to_string(),
        });
        println!("{}", serde_json::to_string_pretty(&result)?);
    } else {
        if success {
            println!(
                "{} {} -> {}",
                "Encrypted".green().bold(),
                input_path.display().to_string().yellow(),
                output_path.display().to_string().dimmed()
            );
        } else {
            eprintln!("{} {}", "error:".red().bold(), stderr);
            anyhow::bail!("Neuralseal encryption failed");
        }
    }

    Ok(())
}

async fn decrypt(key: &str, input: &str, output: Option<&str>, json: bool) -> Result<()> {
    let key_path = resolve_path(key);
    let input_path = resolve_path(input);

    if !key_path.exists() {
        anyhow::bail!("Key file not found: {}", key_path.display());
    }
    if !input_path.exists() {
        anyhow::bail!("Input file not found: {}", input_path.display());
    }

    let output_path = match output {
        Some(o) => resolve_path(o),
        None => {
            let stem = input_path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("output");
            input_path.with_extension(stem.trim_end_matches(".nsenc"))
        }
    };

    let fuc = find_fuc()?;
    let root = FusionConfig::project_root().unwrap_or_else(|_| std::env::current_dir().unwrap());

    let args = vec![
        "neuralseal".to_string(),
        "decrypt".to_string(),
        format!("--key={}", key_path.display()),
        format!("--input={}", input_path.display()),
        format!("--output={}", output_path.display()),
    ];

    let cmd_output = Command::new(&fuc)
        .args(&args)
        .current_dir(&root)
        .output()
        .context("Failed to execute fuc.exe neuralseal decrypt")?;

    let stdout = String::from_utf8_lossy(&cmd_output.stdout);
    let stderr = String::from_utf8_lossy(&cmd_output.stderr);
    let success = cmd_output.status.success();

    if json {
        let result = serde_json::json!({
            "action": "neuralseal_decrypt",
            "key": key,
            "input": input_path.display().to_string(),
            "output": output_path.display().to_string(),
            "success": success,
            "output_text": stdout.trim().to_string(),
        });
        println!("{}", serde_json::to_string_pretty(&result)?);
    } else {
        if success {
            println!(
                "{} {} -> {}",
                "Decrypted".green().bold(),
                input_path.display().to_string().yellow(),
                output_path.display().to_string().dimmed()
            );
        } else {
            eprintln!("{} {}", "error:".red().bold(), stderr);
            anyhow::bail!("Neuralseal decryption failed");
        }
    }

    Ok(())
}

async fn sign(key: &str, input: &str, json: bool) -> Result<()> {
    let key_path = resolve_path(key);
    let input_path = resolve_path(input);

    if !key_path.exists() {
        anyhow::bail!("Key file not found: {}", key_path.display());
    }
    if !input_path.exists() {
        anyhow::bail!("Input file not found: {}", input_path.display());
    }

    let sig_path = input_path.with_extension(format!(
        "{}.nssig",
        input_path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
    ));

    let fuc = find_fuc()?;
    let root = FusionConfig::project_root().unwrap_or_else(|_| std::env::current_dir().unwrap());

    let args = vec![
        "neuralseal".to_string(),
        "sign".to_string(),
        format!("--key={}", key_path.display()),
        format!("--input={}", input_path.display()),
        format!("--output={}", sig_path.display()),
    ];

    let cmd_output = Command::new(&fuc)
        .args(&args)
        .current_dir(&root)
        .output()
        .context("Failed to execute fuc.exe neuralseal sign")?;

    let stdout = String::from_utf8_lossy(&cmd_output.stdout);
    let stderr = String::from_utf8_lossy(&cmd_output.stderr);
    let success = cmd_output.status.success();

    if json {
        let result = serde_json::json!({
            "action": "neuralseal_sign",
            "key": key,
            "input": input_path.display().to_string(),
            "signature": sig_path.display().to_string(),
            "success": success,
            "output_text": stdout.trim().to_string(),
        });
        println!("{}", serde_json::to_string_pretty(&result)?);
    } else {
        if success {
            println!(
                "{} {} -> {}",
                "Signed".green().bold(),
                input_path.display().to_string().yellow(),
                sig_path.display().to_string().dimmed()
            );
        } else {
            eprintln!("{} {}", "error:".red().bold(), stderr);
            anyhow::bail!("Neuralseal signing failed");
        }
    }

    Ok(())
}

async fn verify(key: &str, input: &str, signature: &str, json: bool) -> Result<()> {
    let key_path = resolve_path(key);
    let input_path = resolve_path(input);
    let sig_path = resolve_path(signature);

    if !key_path.exists() {
        anyhow::bail!("Key file not found: {}", key_path.display());
    }
    if !input_path.exists() {
        anyhow::bail!("Input file not found: {}", input_path.display());
    }
    if !sig_path.exists() {
        anyhow::bail!("Signature file not found: {}", sig_path.display());
    }

    let fuc = find_fuc()?;
    let root = FusionConfig::project_root().unwrap_or_else(|_| std::env::current_dir().unwrap());

    let args = vec![
        "neuralseal".to_string(),
        "verify".to_string(),
        format!("--key={}", key_path.display()),
        format!("--input={}", input_path.display()),
        format!("--signature={}", sig_path.display()),
    ];

    let cmd_output = Command::new(&fuc)
        .args(&args)
        .current_dir(&root)
        .output()
        .context("Failed to execute fuc.exe neuralseal verify")?;

    let _stdout = String::from_utf8_lossy(&cmd_output.stdout);
    let _stderr = String::from_utf8_lossy(&cmd_output.stderr);
    let success = cmd_output.status.success();

    if json {
        let result = serde_json::json!({
            "action": "neuralseal_verify",
            "key": key,
            "input": input_path.display().to_string(),
            "signature": sig_path.display().to_string(),
            "valid": success,
        });
        println!("{}", serde_json::to_string_pretty(&result)?);
    } else {
        if success {
            println!(
                "{} Signature is {} for '{}'",
                "OK".green().bold(),
                "valid".green(),
                input_path.display().to_string()
            );
        } else {
            println!(
                "{} Signature is {} for '{}'",
                "FAIL".red().bold(),
                "invalid".red(),
                input_path.display().to_string()
            );
        }
    }

    if !success {
        anyhow::bail!("Neuralseal signature verification failed");
    }

    Ok(())
}
