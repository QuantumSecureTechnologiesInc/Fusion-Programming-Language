use anyhow::Result;
use colored::Colorize;

use crate::config::FusionConfig;

pub async fn run(json: bool) -> Result<()> {
    let root = FusionConfig::project_root()?;
    let config = FusionConfig::load().unwrap_or_default();

    let pkg_name = config.package.as_ref()
        .and_then(|p| p.name.as_deref())
        .unwrap_or("unknown");
    let pkg_version = config.package.as_ref()
        .and_then(|p| p.version.as_deref())
        .unwrap_or("0.0.0");
    let pkg_type = config.package.as_ref()
        .and_then(|p| p.pkg_type.as_deref())
        .unwrap_or("application");

    let build_opt = config.build.as_ref()
        .and_then(|b| b.optimization_level)
        .unwrap_or(2);
    let build_lto = config.build.as_ref()
        .and_then(|b| b.lto)
        .unwrap_or(false);

    let runtime_target = config.runtime.as_ref()
        .and_then(|r| r.target.as_deref())
        .unwrap_or("native");

    let test_parallel = config.test.as_ref()
        .and_then(|t| t.parallel)
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
