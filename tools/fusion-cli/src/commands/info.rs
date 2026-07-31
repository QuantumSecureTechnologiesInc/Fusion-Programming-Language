use anyhow::Result;
use colored::Colorize;

use crate::config::FusionConfig;

pub async fn run(json: bool) -> Result<()> {
    let root = FusionConfig::project_root()?;
    let config = FusionConfig::load().unwrap_or_else(|_| toml::Value::Table(toml::map::Map::new()));

    let pkg_name = config.get("package")
        .and_then(|p| p.get("name"))
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");
    let pkg_version = config.get("package")
        .and_then(|p| p.get("version"))
        .and_then(|v| v.as_str())
        .unwrap_or("0.0.0");
    let pkg_type = config.get("package")
        .and_then(|p| p.get("type"))
        .and_then(|v| v.as_str())
        .unwrap_or("application");

    let build_opt = config.get("build")
        .and_then(|b| b.get("optimization_level"))
        .and_then(|v| v.as_integer())
        .unwrap_or(2);
    let build_lto = config.get("build")
        .and_then(|b| b.get("lto"))
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    let runtime_target = config.get("runtime")
        .and_then(|r| r.get("target"))
        .and_then(|v| v.as_str())
        .unwrap_or("native");

    let test_parallel = config.get("test")
        .and_then(|t| t.get("parallel"))
        .and_then(|v| v.as_bool())
        .unwrap_or(true);

    if json {
        let result = serde_json::json!({
            "name": pkg_name,
            "version": pkg_version,
            "type": pkg_type,
            "root": root.display().to_string(),
            "build": {
                "optimization_level": build_opt,
                "lto": build_lto,
            },
            "runtime": {
                "target": runtime_target,
            },
            "test": {
                "parallel": test_parallel,
            },
        });
        println!("{}", serde_json::to_string_pretty(&result)?);
    } else {
        println!("{}", "Project Info:".cyan().bold());
        println!("  Name:       {}", pkg_name.green());
        println!("  Version:    {}", pkg_version);
        println!("  Type:       {}", pkg_type);
        println!("  Root:       {}", root.display());
        println!();
        println!("{}", "Build:".cyan().bold());
        println!("  Opt Level:  {}", build_opt);
        println!("  LTO:        {}", build_lto);
        println!();
        println!("{}", "Runtime:".cyan().bold());
        println!("  Target:     {}", runtime_target);
        println!();
        println!("{}", "Test:".cyan().bold());
        println!("  Parallel:   {}", test_parallel);
    }

    Ok(())
}
