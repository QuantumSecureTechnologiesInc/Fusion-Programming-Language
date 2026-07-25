use anyhow::{Context, Result};
use colored::Colorize;
use dialoguer::Input;
use std::collections::HashMap;
use std::process::Command;

use crate::commands::AiCommands;

const OLLAMA_API: &str = "http://localhost:11434";

pub async fn run(command: AiCommands, json: bool) -> Result<()> {
    match command {
        AiCommands::Chat { model } => chat(&model, json).await,
        AiCommands::Complete { model, prompt } => complete(&model, &prompt, json).await,
        AiCommands::Explain { file } => explain(&file, json).await,
        AiCommands::Review { file } => review(&file, json).await,
        AiCommands::Generate { description } => generate(&description.join(" "), json).await,
        AiCommands::Models => list_models(json).await,
        AiCommands::Pull { model } => pull(&model, json).await,
    }
}

async fn chat(model: &str, _json: bool) -> Result<()> {
    println!("{} Starting chat with {} (type 'exit' to quit)", "ai:".cyan().bold(), model.yellow());

    loop {
        let input: String = Input::new()
            .with_prompt("you".cyan().to_string())
            .allow_empty(true)
            .interact_text()
            .context("Failed to read input")?;

        if input.trim().is_empty() || input.trim() == "exit" {
            println!("{}", "Chat ended.".dimmed());
            break;
        }

        let mut payload = HashMap::new();
        payload.insert("model", model);
        payload.insert("prompt", &input);
        payload.insert("stream", "false");

        let client = reqwest::Client::new();
        let resp = client
            .post(format!("{}/api/generate", OLLAMA_API))
            .json(&payload)
            .send()
            .await
            .context("Failed to connect to Ollama")?;

        if !resp.status().is_success() {
            eprintln!("{} Ollama returned status {}", "error:".red().bold(), resp.status());
            continue;
        }

        let body: serde_json::Value = resp.json().await.context("Invalid response from Ollama")?;
        let response = body["response"].as_str().unwrap_or("no response");

        println!("{} {}\n", model.green().bold(), response);
    }

    Ok(())
}

async fn complete(model: &str, prompt: &str, json: bool) -> Result<()> {
    let mut payload = HashMap::new();
    payload.insert("model", model);
    payload.insert("prompt", prompt);
    payload.insert("stream", "false");

    let client = reqwest::Client::new();
    let resp = client
        .post(format!("{}/api/generate", OLLAMA_API))
        .json(&payload)
        .send()
        .await
        .context("Failed to connect to Ollama")?;

    let body: serde_json::Value = resp.json().await?;
    let response = body["response"].as_str().unwrap_or("");

    if json {
        let result = serde_json::json!({
            "model": model,
            "prompt": prompt,
            "response": response,
        });
        println!("{}", serde_json::to_string_pretty(&result)?);
    } else {
        println!("{}", response);
    }

    Ok(())
}

async fn explain(file: &str, json: bool) -> Result<()> {
    let content = std::fs::read_to_string(file)
        .with_context(|| format!("Failed to read {}", file))?;

    let prompt = format!("Explain this Fusion code in detail:\n\n{}", content);
    let mut payload = HashMap::new();
    payload.insert("model", "llama3");
    payload.insert("prompt", prompt.as_str());
    payload.insert("stream", "false");

    let client = reqwest::Client::new();
    let resp = client
        .post(format!("{}/api/generate", OLLAMA_API))
        .json(&payload)
        .send()
        .await
        .context("Failed to connect to Ollama")?;

    let body: serde_json::Value = resp.json().await?;
    let response = body["response"].as_str().unwrap_or("");

    if json {
        let result = serde_json::json!({
            "file": file,
            "explanation": response,
        });
        println!("{}", serde_json::to_string_pretty(&result)?);
    } else {
        println!("{} {}", "Explanation:".cyan().bold(), file);
        println!("{}", response);
    }

    Ok(())
}

async fn review(file: &str, json: bool) -> Result<()> {
    let content = std::fs::read_to_string(file)
        .with_context(|| format!("Failed to read {}", file))?;

    let prompt = format!("Review this Fusion code for bugs, security issues, and improvements:\n\n{}", content);
    let mut payload = HashMap::new();
    payload.insert("model", "llama3");
    payload.insert("prompt", prompt.as_str());
    payload.insert("stream", "false");

    let client = reqwest::Client::new();
    let resp = client
        .post(format!("{}/api/generate", OLLAMA_API))
        .json(&payload)
        .send()
        .await
        .context("Failed to connect to Ollama")?;

    let body: serde_json::Value = resp.json().await?;
    let response = body["response"].as_str().unwrap_or("");

    if json {
        let result = serde_json::json!({
            "file": file,
            "review": response,
        });
        println!("{}", serde_json::to_string_pretty(&result)?);
    } else {
        println!("{} {}", "Code Review:".cyan().bold(), file);
        println!("{}", response);
    }

    Ok(())
}

async fn generate(description: &str, json: bool) -> Result<()> {
    let prompt = format!("Generate Fusion code based on this description:\n\n{}", description);
    let mut payload = HashMap::new();
    payload.insert("model", "llama3");
    payload.insert("prompt", prompt.as_str());
    payload.insert("stream", "false");

    let client = reqwest::Client::new();
    let resp = client
        .post(format!("{}/api/generate", OLLAMA_API))
        .json(&payload)
        .send()
        .await
        .context("Failed to connect to Ollama")?;

    let body: serde_json::Value = resp.json().await?;
    let response = body["response"].as_str().unwrap_or("");

    if json {
        let result = serde_json::json!({
            "description": description,
            "code": response,
        });
        println!("{}", serde_json::to_string_pretty(&result)?);
    } else {
        println!("{} Generated code:", "ai:".cyan().bold());
        println!("{}", response);
    }

    Ok(())
}

async fn list_models(json: bool) -> Result<()> {
    let output = Command::new("ollama")
        .args(["list"])
        .output()
        .context("Failed to execute ollama list. Is Ollama installed?")?;

    let stdout = String::from_utf8_lossy(&output.stdout);

    if json {
        let models: Vec<&str> = stdout.lines().skip(1).filter(|l| !l.trim().is_empty()).collect();
        let result = serde_json::json!({ "models": models });
        println!("{}", serde_json::to_string_pretty(&result)?);
    } else {
        println!("{} Available models:", "ai:".cyan().bold());
        if stdout.trim().is_empty() {
            println!("  No models installed. Run `fusion ai pull <model>` to download one.");
        } else {
            println!("{}", stdout);
        }
    }

    Ok(())
}

async fn pull(model: &str, json: bool) -> Result<()> {
    println!("{} Pulling model '{}'...", "ai:".cyan().bold(), model.yellow());

    let output = Command::new("ollama")
        .args(["pull", model])
        .stdin(std::process::Stdio::inherit())
        .stdout(std::process::Stdio::inherit())
        .stderr(std::process::Stdio::inherit())
        .status()
        .context("Failed to execute ollama pull. Is Ollama installed?")?;

    if output.success() {
        if json {
            let result = serde_json::json!({ "model": model, "status": "pulled" });
            println!("{}", serde_json::to_string_pretty(&result)?);
        } else {
            println!("{} Model '{}' pulled successfully.", "OK".green().bold(), model);
        }
    } else {
        anyhow::bail!("Failed to pull model '{}'", model);
    }

    Ok(())
}
