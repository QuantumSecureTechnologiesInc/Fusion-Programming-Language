use anyhow::{Context, Result};
use colored::Colorize;
use std::collections::HashMap;
use std::process::Command;

use super::ProviderSubcommand;

const OLLAMA_API: &str = "http://localhost:11434";

const PROVIDERS: &[(&str, &str, &str)] = &[
    ("ollama", "Ollama", "Local model serving"),
    ("openai", "OpenAI", "GPT-4o, GPT-4, GPT-3.5"),
    ("anthropic", "Anthropic", "Claude 3.5, Claude 3"),
    ("google", "Google AI", "Gemini, PaLM"),
    ("mistral", "Mistral AI", "Mistral, Mixtral"),
    ("cohere", "Cohere", "Command, Embed"),
    ("huggingface", "Hugging Face", "Open-source models"),
    ("stability", "Stability AI", "Stable Diffusion"),
    ("replicate", "Replicate", "Cloud model hosting"),
    ("together", "Together AI", "Open-source inference"),
    ("groq", "Groq", "Ultra-fast inference"),
    ("perplexity", "Perplexity", "Sonar models"),
    ("bedrock", "AWS Bedrock", "Enterprise AI"),
    ("azure", "Azure OpenAI", "Enterprise GPT"),
    ("watsonx", "IBM watsonx", "Enterprise AI"),
    ("ai21", "AI21 Labs", "Jurassic, Jamba"),
    ("inflection", "Inflection AI", "Pi assistant"),
    ("xai", "xAI", "Grok"),
    ("deepseek", "DeepSeek", "DeepSeek-V3"),
    ("qwen", "Alibaba Qwen", "Qwen series"),
    ("meta", "Meta AI", "Llama series"),
    ("nvidia", "NVIDIA NIM", "NIM inference"),
    ("fireworks", "Fireworks AI", "Fast inference"),
    ("anyscale", "Anyscale", "Ray-based serving"),
    ("modal", "Modal", "Serverless GPU"),
    ("vllm", "vLLM", "High-throughput serving"),
];

pub async fn run(subcmd: ProviderSubcommand, json: bool) -> Result<()> {
    match subcmd {
        ProviderSubcommand::List => list(json).await,
        ProviderSubcommand::Status { provider } => status(&provider, json).await,
        ProviderSubcommand::Pull { model } => pull(&model, json).await,
        ProviderSubcommand::Test { model } => test_model(&model, json).await,
    }
}

async fn list(json: bool) -> Result<()> {
    if json {
        let entries: Vec<serde_json::Value> = PROVIDERS
            .iter()
            .map(|(id, name, desc)| {
                serde_json::json!({
                    "id": id,
                    "name": name,
                    "description": desc,
                })
            })
            .collect();
        let result = serde_json::json!({
            "action": "providers_list",
            "count": entries.len(),
            "providers": entries,
        });
        println!("{}", serde_json::to_string_pretty(&result)?);
    } else {
        println!(
            "{} {}",
            "AI Model Providers:".cyan().bold(),
            format!("{} available", PROVIDERS.len()).dimmed()
        );
        println!();
        for (id, name, desc) in PROVIDERS {
            println!(
                "  {} {} - {}",
                name.green(),
                format!("({})", id).dimmed(),
                desc
            );
        }
    }

    Ok(())
}

async fn status(provider: &str, json: bool) -> Result<()> {
    let found = PROVIDERS.iter().find(|(id, _, _)| *id == provider);
    if found.is_none() {
        anyhow::bail!(
            "Unknown provider '{}'. Run `fusion providers list` to see available providers.",
            provider
        );
    }

    let (_, name, _) = found.unwrap();

    // For Ollama, check local API directly
    if provider == "ollama" {
        let client = reqwest::Client::new();
        let resp = client
            .get(format!("{}/api/tags", OLLAMA_API))
            .timeout(std::time::Duration::from_secs(5))
            .send()
            .await;

        let online = resp.is_ok() && resp.unwrap().status().is_success();

        if json {
            let result = serde_json::json!({
                "action": "provider_status",
                "provider": provider,
                "name": name,
                "online": online,
            });
            println!("{}", serde_json::to_string_pretty(&result)?);
        } else {
            if online {
                println!(
                    "{} {} is {}",
                    "OK".green().bold(),
                    name.green(),
                    "online".green()
                );
                // Show available models
                let models_resp = client
                    .get(format!("{}/api/tags", OLLAMA_API))
                    .timeout(std::time::Duration::from_secs(5))
                    .send()
                    .await;
                if let Ok(resp) = models_resp {
                    if let Ok(body) = resp.json::<serde_json::Value>().await {
                        if let Some(models) = body["models"].as_array() {
                            println!(
                                "  {} {}",
                                "Models:".cyan(),
                                models.len()
                            );
                            for m in models.iter().take(10) {
                                let mname = m["name"].as_str().unwrap_or("?");
                                let msize = m["size"].as_u64().unwrap_or(0);
                                let size_str = if msize > 1_073_741_824 {
                                    format!("{} GB", msize / 1_073_741_824)
                                } else if msize > 1_048_576 {
                                    format!("{} MB", msize / 1_048_576)
                                } else {
                                    format!("{} B", msize)
                                };
                                println!("    {} ({})", mname.green(), size_str.dimmed());
                            }
                            if models.len() > 10 {
                                println!(
                                    "    ... and {} more",
                                    models.len() - 10
                                );
                            }
                        }
                    }
                }
            } else {
                println!(
                    "{} {} is {}",
                    "WARN".yellow().bold(),
                    name.yellow(),
                    "offline".red()
                );
                println!("  Start Ollama to use local models: `ollama serve`");
            }
        }

        return Ok(());
    }

    // For cloud providers, check if they have an endpoint configured
    let has_key = match provider {
        "openai" => std::env::var("OPENAI_API_KEY").is_ok(),
        "anthropic" => std::env::var("ANTHROPIC_API_KEY").is_ok(),
        "google" => std::env::var("GOOGLE_API_KEY").is_ok() || std::env::var("GEMINI_API_KEY").is_ok(),
        "mistral" => std::env::var("MISTRAL_API_KEY").is_ok(),
        "cohere" => std::env::var("COHERE_API_KEY").is_ok(),
        "huggingface" => std::env::var("HUGGING_FACE_HUB_TOKEN").is_ok() || std::env::var("HF_TOKEN").is_ok(),
        "groq" => std::env::var("GROQ_API_KEY").is_ok(),
        "deepseek" => std::env::var("DEEPSEEK_API_KEY").is_ok(),
        "xai" => std::env::var("XAI_API_KEY").is_ok(),
        "bedrock" => std::env::var("AWS_ACCESS_KEY_ID").is_ok(),
        "azure" => std::env::var("AZURE_OPENAI_ENDPOINT").is_ok(),
        "watsonx" => std::env::var("WATSONX_APIKEY").is_ok(),
        "together" => std::env::var("TOGETHER_API_KEY").is_ok(),
        "fireworks" => std::env::var("FIREWORKS_API_KEY").is_ok(),
        "replicate" => std::env::var("REPLICATE_API_TOKEN").is_ok(),
        _ => false,
    };

    if json {
        let result = serde_json::json!({
            "action": "provider_status",
            "provider": provider,
            "name": name,
            "configured": has_key,
        });
        println!("{}", serde_json::to_string_pretty(&result)?);
    } else {
        if has_key {
            println!(
                "{} {} is {}",
                "OK".green().bold(),
                name.green(),
                "configured".green()
            );
        } else {
            println!(
                "{} {} is {}",
                "WARN".yellow().bold(),
                name.yellow(),
                "not configured".yellow()
            );
            println!(
                "  Set the appropriate API key environment variable to enable this provider."
            );
        }
    }

    Ok(())
}

async fn pull(model: &str, json: bool) -> Result<()> {
    println!(
        "{} Pulling model '{}'...",
        "Providers:".cyan().bold(),
        model.yellow()
    );

    let status = Command::new("ollama")
        .args(["pull", model])
        .stdin(std::process::Stdio::inherit())
        .stdout(std::process::Stdio::inherit())
        .stderr(std::process::Stdio::inherit())
        .status()
        .context("Failed to execute ollama pull. Is Ollama installed?")?;

    if status.success() {
        if json {
            let result = serde_json::json!({
                "action": "providers_pull",
                "model": model,
                "status": "pulled",
            });
            println!("{}", serde_json::to_string_pretty(&result)?);
        } else {
            println!(
                "{} Model '{}' pulled successfully.",
                "OK".green().bold(),
                model
            );
        }
    } else {
        anyhow::bail!("Failed to pull model '{}'", model);
    }

    Ok(())
}

async fn test_model(model: &str, json: bool) -> Result<()> {
    println!(
        "{} Testing model '{}'...",
        "Providers:".cyan().bold(),
        model.yellow()
    );

    let test_prompt = "Say hello in exactly one sentence.";

    let mut payload = HashMap::new();
    payload.insert("model", model);
    payload.insert("prompt", test_prompt);
    payload.insert("stream", "false");

    let client = reqwest::Client::new();
    let resp = match client
        .post(format!("{}/api/generate", OLLAMA_API))
        .json(&payload)
        .timeout(std::time::Duration::from_secs(60))
        .send()
        .await
    {
        Ok(r) => r,
        Err(e) => {
            if json {
                let result = serde_json::json!({
                    "action": "providers_test",
                    "model": model,
                    "success": false,
                    "error": e.to_string(),
                });
                println!("{}", serde_json::to_string_pretty(&result)?);
                return Ok(());
            }
            anyhow::bail!(
                "Failed to connect to Ollama at {}. Is it running? ({})",
                OLLAMA_API,
                e
            );
        }
    };

    if !resp.status().is_success() {
        let status = resp.status();
        if json {
            let result = serde_json::json!({
                "action": "providers_test",
                "model": model,
                "success": false,
                "error": format!("HTTP {}", status),
            });
            println!("{}", serde_json::to_string_pretty(&result)?);
            return Ok(());
        }
        anyhow::bail!("Ollama returned status {}", status);
    }

    let body: serde_json::Value = resp
        .json()
        .await
        .context("Invalid response from Ollama")?;

    let response = body["response"].as_str().unwrap_or("no response");
    let duration_ms = body["total_duration"].as_u64().unwrap_or(0) / 1_000_000;
    let eval_count = body["eval_count"].as_u64().unwrap_or(0);
    let eval_duration_ns = body["eval_duration"].as_u64().unwrap_or(0);
    let tokens_per_sec = if eval_duration_ns > 0 {
        (eval_count as f64) / (eval_duration_ns as f64 / 1_000_000_000.0)
    } else {
        0.0
    };

    if json {
        let result = serde_json::json!({
            "action": "providers_test",
            "model": model,
            "success": true,
            "prompt": test_prompt,
            "response": response,
            "duration_ms": duration_ms,
            "eval_count": eval_count,
            "tokens_per_sec": format!("{:.1}", tokens_per_sec),
        });
        println!("{}", serde_json::to_string_pretty(&result)?);
    } else {
        println!(
            "{} Model '{}' responded:",
            "OK".green().bold(),
            model.green()
        );
        println!("  {}", response);
        println!();
        println!(
            "  {} {}ms | {} tokens | {:.1} tok/s",
            "Performance:".cyan(),
            duration_ms,
            eval_count,
            tokens_per_sec
        );
    }

    Ok(())
}
