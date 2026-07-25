use anyhow::{Context, Result};
use colored::Colorize;
use std::collections::HashMap;
use std::process::Command;

use crate::commands::QuantumCommands;
use crate::config::FusionConfig;

const QUANTUM_API: &str = "https://quantum.fusion-lang.org/api";

pub async fn run(command: QuantumCommands, json: bool) -> Result<()> {
    match command {
        QuantumCommands::Simulate { circuit, shots } => simulate(&circuit, shots, json).await,
        QuantumCommands::Run { circuit, backend } => run_circuit(&circuit, &backend, json).await,
        QuantumCommands::Devices { provider } => devices(provider.as_deref(), json).await,
    }
}

async fn simulate(circuit: &str, shots: u32, json: bool) -> Result<()> {
    let root = FusionConfig::project_root().unwrap_or_else(|_| std::env::current_dir().unwrap());
    let circuit_path = root.join(circuit);

    if !circuit_path.exists() {
        anyhow::bail!("Circuit file not found: {}", circuit_path.display());
    }

    let args = vec![
        "quantum".to_string(),
        "simulate".to_string(),
        circuit_path.display().to_string(),
        format!("--shots={}", shots),
    ];

    let output = Command::new("fuc.exe")
        .args(&args)
        .current_dir(&root)
        .output()
        .context("Failed to execute fuc.exe quantum simulate")?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let success = output.status.success();

    if json {
        let result = serde_json::json!({
            "action": "simulate",
            "circuit": circuit,
            "shots": shots,
            "success": success,
            "output": stdout.trim().to_string(),
        });
        println!("{}", serde_json::to_string_pretty(&result)?);
    } else {
        println!("{} Circuit '{}' ({} shots)", "Quantum:".cyan().bold(), circuit.yellow(), shots);
        if !stdout.is_empty() {
            println!("{}", stdout);
        }
        if !stderr.is_empty() {
            eprintln!("{}", stderr);
        }
    }

    if !success {
        anyhow::bail!("Quantum simulation failed");
    }

    Ok(())
}

async fn run_circuit(circuit: &str, backend: &str, json: bool) -> Result<()> {
    let root = FusionConfig::project_root().unwrap_or_else(|_| std::env::current_dir().unwrap());
    let circuit_path = root.join(circuit);

    if !circuit_path.exists() {
        anyhow::bail!("Circuit file not found: {}", circuit_path.display());
    }

    let circuit_content = std::fs::read_to_string(&circuit_path)
        .context("Failed to read circuit file")?;

    let mut payload = HashMap::new();
    payload.insert("circuit", circuit_content.as_str());
    payload.insert("backend", backend);
    payload.insert("shots", "1024");

    let client = reqwest::Client::new();
    let resp = client
        .post(format!("{}/quantum/run", QUANTUM_API))
        .json(&payload)
        .send()
        .await
        .context("Failed to connect to quantum backend")?;

    let body: serde_json::Value = resp.json().await.context("Invalid response from quantum API")?;

    if json {
        println!("{}", serde_json::to_string_pretty(&body)?);
    } else {
        println!("{} Running on {}", "Quantum:".cyan().bold(), backend.green());
        if let Some(results) = body.get("results") {
            println!("{}", serde_json::to_string_pretty(results).unwrap_or_default());
        } else if let Some(error) = body.get("error") {
            eprintln!("{} {}", "error:".red().bold(), error);
        } else {
            println!("{}", serde_json::to_string_pretty(&body).unwrap_or_default());
        }
    }

    Ok(())
}

async fn devices(provider: Option<&str>, json: bool) -> Result<()> {
    let client = reqwest::Client::new();
    let url = match provider {
        Some(p) => format!("{}/quantum/devices?provider={}", QUANTUM_API, p),
        None => format!("{}/quantum/devices", QUANTUM_API),
    };

    let resp = client
        .get(&url)
        .send()
        .await
        .context("Failed to connect to quantum API")?;

    let body: serde_json::Value = resp.json().await.context("Invalid response from quantum API")?;

    if json {
        println!("{}", serde_json::to_string_pretty(&body)?);
    } else {
        println!("{} Available quantum devices:", "Quantum:".cyan().bold());
        if let Some(devices) = body["devices"].as_array() {
            for dev in devices {
                let name = dev["name"].as_str().unwrap_or("?");
                let prov = dev["provider"].as_str().unwrap_or("?");
                let qubits = dev["qubits"].as_u64().unwrap_or(0);
                let status = dev["status"].as_str().unwrap_or("unknown");
                let status_color = match status {
                    "online" => status.green(),
                    "busy" => status.yellow(),
                    _ => status.red(),
                };
                println!("  {} ({}) - {} qubits [{}]", name.green(), prov.dimmed(), qubits, status_color);
            }
        } else {
            println!("  No devices available.");
        }
    }

    Ok(())
}
