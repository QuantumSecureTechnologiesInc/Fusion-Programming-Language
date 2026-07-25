use anyhow::{Context, Result};
use colored::Colorize;
use std::process::Command;

use crate::commands::CryptoCommands;

pub async fn run(command: CryptoCommands, json: bool) -> Result<()> {
    match command {
        CryptoCommands::Keygen { output, algorithm } => keygen(&output, &algorithm, json).await,
        CryptoCommands::Sign { file, key } => sign(&file, &key, json).await,
        CryptoCommands::Verify { file, sig, key } => verify(&file, &sig, &key, json).await,
        CryptoCommands::Encrypt { file, key } => encrypt(&file, &key, json).await,
        CryptoCommands::Decrypt { file, key } => decrypt(&file, &key, json).await,
    }
}

async fn keygen(output: &str, algorithm: &str, json: bool) -> Result<()> {
    let args = vec![
        "crypto".to_string(),
        "keygen".to_string(),
        format!("--output={}", output),
        format!("--algorithm={}", algorithm),
    ];

    let output_result = Command::new("fuc.exe")
        .args(&args)
        .output()
        .context("Failed to execute fuc.exe crypto keygen")?;

    let _stdout = String::from_utf8_lossy(&output_result.stdout);
    let stderr = String::from_utf8_lossy(&output_result.stderr);
    let success = output_result.status.success();

    if success {
        let pub_path = format!("{}/fusion.key.public", output);
        let sec_path = format!("{}/fusion.key.secret", output);

        if json {
            let result = serde_json::json!({
                "action": "keygen",
                "algorithm": algorithm,
                "public_key": pub_path,
                "secret_key": sec_path,
            });
            println!("{}", serde_json::to_string_pretty(&result)?);
        } else {
            println!("{} Generated {} keypair", "Crypto:".cyan().bold(), algorithm.yellow());
            println!("  Public:  {}", pub_path);
            println!("  Secret:  {}", sec_path);
            println!("  Algorithm: {}", "ML-KEM + ML-DSA (hybrid)".dimmed());
        }
    } else {
        eprintln!("{} {}", "error:".red().bold(), stderr);
        anyhow::bail!("Key generation failed");
    }

    Ok(())
}

async fn sign(file: &str, key: &str, json: bool) -> Result<()> {
    if !std::path::Path::new(file).exists() {
        anyhow::bail!("File not found: {}", file);
    }
    if !std::path::Path::new(key).exists() {
        anyhow::bail!("Key file not found: {}", key);
    }

    let sig_file = format!("{}.sig", file);

    let args = vec![
        "crypto".to_string(),
        "sign".to_string(),
        file.to_string(),
        format!("--key={}", key),
        format!("--output={}", sig_file),
    ];

    let output = Command::new("fuc.exe")
        .args(&args)
        .output()
        .context("Failed to execute fuc.exe crypto sign")?;

    let _stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let success = output.status.success();

    if success {
        if json {
            let result = serde_json::json!({
                "action": "sign",
                "file": file,
                "signature": sig_file,
                "key": key,
            });
            println!("{}", serde_json::to_string_pretty(&result)?);
        } else {
            println!("{} {} -> {}", "Signed".green().bold(), file.yellow(), sig_file.dimmed());
        }
    } else {
        eprintln!("{} {}", "error:".red().bold(), stderr);
        anyhow::bail!("Signing failed");
    }

    Ok(())
}

async fn verify(file: &str, sig: &str, key: &str, json: bool) -> Result<()> {
    if !std::path::Path::new(file).exists() {
        anyhow::bail!("File not found: {}", file);
    }
    if !std::path::Path::new(sig).exists() {
        anyhow::bail!("Signature file not found: {}", sig);
    }
    if !std::path::Path::new(key).exists() {
        anyhow::bail!("Key file not found: {}", key);
    }

    let args = vec![
        "crypto".to_string(),
        "verify".to_string(),
        file.to_string(),
        sig.to_string(),
        format!("--key={}", key),
    ];

    let output = Command::new("fuc.exe")
        .args(&args)
        .output()
        .context("Failed to execute fuc.exe crypto verify")?;

    let _stdout = String::from_utf8_lossy(&output.stdout);
    let _stderr = String::from_utf8_lossy(&output.stderr);
    let success = output.status.success();

    if json {
        let result = serde_json::json!({
            "action": "verify",
            "file": file,
            "signature": sig,
            "valid": success,
        });
        println!("{}", serde_json::to_string_pretty(&result)?);
    } else {
        if success {
            println!("{} Signature is {} for '{}'", "OK".green().bold(), "valid".green(), file);
        } else {
            println!("{} Signature is {} for '{}'", "FAIL".red().bold(), "invalid".red(), file);
        }
    }

    if !success {
        anyhow::bail!("Signature verification failed");
    }

    Ok(())
}

async fn encrypt(file: &str, key: &str, json: bool) -> Result<()> {
    if !std::path::Path::new(file).exists() {
        anyhow::bail!("File not found: {}", file);
    }
    if !std::path::Path::new(key).exists() {
        anyhow::bail!("Key file not found: {}", key);
    }

    let enc_file = format!("{}.enc", file);

    let args = vec![
        "crypto".to_string(),
        "encrypt".to_string(),
        file.to_string(),
        format!("--key={}", key),
        format!("--output={}", enc_file),
    ];

    let output = Command::new("fuc.exe")
        .args(&args)
        .output()
        .context("Failed to execute fuc.exe crypto encrypt")?;

    let _stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let success = output.status.success();

    if success {
        if json {
            let result = serde_json::json!({
                "action": "encrypt",
                "file": file,
                "encrypted": enc_file,
                "key": key,
            });
            println!("{}", serde_json::to_string_pretty(&result)?);
        } else {
            println!("{} {} -> {}", "Encrypted".green().bold(), file.yellow(), enc_file.dimmed());
        }
    } else {
        eprintln!("{} {}", "error:".red().bold(), stderr);
        anyhow::bail!("Encryption failed");
    }

    Ok(())
}

async fn decrypt(file: &str, key: &str, json: bool) -> Result<()> {
    if !std::path::Path::new(file).exists() {
        anyhow::bail!("File not found: {}", file);
    }
    if !std::path::Path::new(key).exists() {
        anyhow::bail!("Key file not found: {}", key);
    }

    let dec_file = file.strip_suffix(".enc").unwrap_or(file);

    let args = vec![
        "crypto".to_string(),
        "decrypt".to_string(),
        file.to_string(),
        format!("--key={}", key),
        format!("--output={}", dec_file),
    ];

    let output = Command::new("fuc.exe")
        .args(&args)
        .output()
        .context("Failed to execute fuc.exe crypto decrypt")?;

    let _stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let success = output.status.success();

    if success {
        if json {
            let result = serde_json::json!({
                "action": "decrypt",
                "file": file,
                "decrypted": dec_file,
                "key": key,
            });
            println!("{}", serde_json::to_string_pretty(&result)?);
        } else {
            println!("{} {} -> {}", "Decrypted".green().bold(), file.yellow(), dec_file.dimmed());
        }
    } else {
        eprintln!("{} {}", "error:".red().bold(), stderr);
        anyhow::bail!("Decryption failed");
    }

    Ok(())
}
