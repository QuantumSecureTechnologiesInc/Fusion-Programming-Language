use anyhow::{Context, Result};
use colored::Colorize;
use std::collections::HashMap;
use std::process::Command;

use crate::commands::DeployCommands;
use crate::config::FusionConfig;

const DEPLOY_API: &str = "https://deploy.fusion-lang.org/api";

pub async fn run(command: DeployCommands, json: bool) -> Result<()> {
    match command {
        DeployCommands::Target { target, env } => deploy_target(&target, &env, json).await,
        DeployCommands::Status { id } => status(id.as_deref(), json).await,
        DeployCommands::Rollback { to } => rollback(to.as_deref(), json).await,
    }
}

async fn deploy_target(target: &str, env: &str, json: bool) -> Result<()> {
    let root = FusionConfig::project_root().unwrap_or_else(|_| std::env::current_dir().unwrap());

    println!("{} Deploying to {} ({})", "Deploy:".cyan().bold(), target.green(), env.yellow());

    // Build first
    println!("  Step 1/3: Building project...");
    let build_output = Command::new("fuc.exe")
        .args(["build", "--release"])
        .current_dir(&root)
        .output()
        .context("Failed to build for deployment")?;

    if !build_output.status.success() {
        let stderr = String::from_utf8_lossy(&build_output.stderr);
        anyhow::bail!("Build failed during deployment: {}", stderr);
    }

    // Package
    println!("  Step 2/3: Packaging...");
    let pkg_output = Command::new("fuc.exe")
        .args(["pkg", "pack"])
        .current_dir(&root)
        .output()
        .context("Failed to package for deployment")?;

    if !pkg_output.status.success() {
        let stderr = String::from_utf8_lossy(&pkg_output.stderr);
        anyhow::bail!("Packaging failed: {}", stderr);
    }

    // Deploy
    println!("  Step 3/3: Uploading...");
    let mut payload = HashMap::new();
    let root_str = root.display().to_string();
    payload.insert("target", target);
    payload.insert("environment", env);
    payload.insert("project_root", root_str.as_str());

    let client = reqwest::Client::new();
    let resp = client
        .post(format!("{}/deploy", DEPLOY_API))
        .json(&payload)
        .send()
        .await
        .context("Failed to connect to deployment service")?;

    let body: serde_json::Value = resp.json().await.context("Invalid deployment response")?;

    let deploy_id = body["deployment_id"].as_str().unwrap_or("unknown");

    if json {
        let mut result = body.clone();
        result["target"] = serde_json::json!(target);
        result["environment"] = serde_json::json!(env);
        println!("{}", serde_json::to_string_pretty(&result)?);
    } else {
        println!("\n{} Deployment successful!", "OK".green().bold());
        println!("  Target:   {}", target);
        println!("  Env:      {}", env);
        println!("  ID:       {}", deploy_id.dimmed());
        if let Some(url) = body["url"].as_str() {
            println!("  URL:      {}", url.cyan());
        }
    }

    Ok(())
}

async fn status(id: Option<&str>, json: bool) -> Result<()> {
    let url = match id {
        Some(deploy_id) => format!("{}/status/{}", DEPLOY_API, deploy_id),
        None => format!("{}/status/latest", DEPLOY_API),
    };

    let client = reqwest::Client::new();
    let resp = client
        .get(&url)
        .send()
        .await
        .context("Failed to connect to deployment service")?;

    let body: serde_json::Value = resp.json().await?;

    if json {
        println!("{}", serde_json::to_string_pretty(&body)?);
    } else {
        println!("{} Deployment Status:", "Deploy:".cyan().bold());
        if let Some(deployments) = body["deployments"].as_array() {
            for dep in deployments {
                let d_id = dep["id"].as_str().unwrap_or("?");
                let d_env = dep["environment"].as_str().unwrap_or("?");
                let d_status = dep["status"].as_str().unwrap_or("?");
                let d_time = dep["created_at"].as_str().unwrap_or("?");
                let status_color = match d_status {
                    "active" | "running" => d_status.green(),
                    "failed" => d_status.red(),
                    _ => d_status.yellow(),
                };
                println!("  {} [{}] {} - {}", d_id.dimmed(), d_env, status_color, d_time);
            }
        } else {
            let d_status = body["status"].as_str().unwrap_or("unknown");
            println!("  Status: {}", d_status);
        }
    }

    Ok(())
}

async fn rollback(to: Option<&str>, json: bool) -> Result<()> {
    let client = reqwest::Client::new();

    let mut payload = HashMap::new();
    if let Some(target) = to {
        payload.insert("target_deployment_id", target);
    }

    let resp = client
        .post(format!("{}/rollback", DEPLOY_API))
        .json(&payload)
        .send()
        .await
        .context("Failed to connect to deployment service")?;

    let body: serde_json::Value = resp.json().await?;

    if json {
        println!("{}", serde_json::to_string_pretty(&body)?);
    } else {
        println!("{} Rollback initiated!", "OK".green().bold());
        if let Some(msg) = body["message"].as_str() {
            println!("  {}", msg);
        }
        if let Some(id) = body["rollback_id"].as_str() {
            println!("  Rollback ID: {}", id.dimmed());
        }
    }

    Ok(())
}
