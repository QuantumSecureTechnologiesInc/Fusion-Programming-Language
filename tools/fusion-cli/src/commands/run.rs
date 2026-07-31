use anyhow::{Context, Result};
use colored::Colorize;
use std::path::Path;
use std::process::Command;

/// Supported language runtimes for polyglot execution.
fn detect_lang(file: &str) -> Option<&'static str> {
    let ext = Path::new(file).extension()?.to_str()?;
    match ext {
        "fu" | "fusion" => Some("fusion"),
        "py" => Some("python"),
        "js" | "mjs" => Some("javascript"),
        "ts" | "mts" => Some("javascript"),
        "java" => Some("java"),
        "rs" => Some("rust"),
        "c" | "cpp" | "cxx" => Some("c"),
        _ => None,
    }
}

fn run_fusion(file: &str, verbose: bool) -> Result<()> {
    let mut args = vec!["run".to_string(), file.to_string()];
    if verbose {
        args.push("--verbose".to_string());
    }

    let status = Command::new("fuc.exe")
        .args(&args)
        .stdin(std::process::Stdio::inherit())
        .stdout(std::process::Stdio::inherit())
        .stderr(std::process::Stdio::inherit())
        .status()
        .context("Failed to execute fuc.exe run")?;

    if !status.success() {
        anyhow::bail!("Process exited with code {:?}", status.code());
    }
    Ok(())
}

fn run_python(file: &str, verbose: bool) -> Result<()> {
    if verbose {
        println!("{} Running Python: {}", ">>".cyan().bold(), file);
    }
    let status = Command::new("python")
        .arg(file)
        .stdin(std::process::Stdio::inherit())
        .stdout(std::process::Stdio::inherit())
        .stderr(std::process::Stdio::inherit())
        .status()
        .context("Failed to execute python")?;
    if !status.success() {
        anyhow::bail!("Python process exited with code {:?}", status.code());
    }
    Ok(())
}

fn run_javascript(file: &str, verbose: bool) -> Result<()> {
    if verbose {
        println!("{} Running JavaScript: {}", ">>".cyan().bold(), file);
    }
    // Prefer node, fallback to deno
    let program = if Command::new("node").arg("--version").output().is_ok() {
        "node"
    } else if Command::new("deno").arg("--version").output().is_ok() {
        "deno"
    } else {
        anyhow::bail!("No JavaScript runtime found. Install node or deno.");
    };
    let status = Command::new(program)
        .arg(file)
        .stdin(std::process::Stdio::inherit())
        .stdout(std::process::Stdio::inherit())
        .stderr(std::process::Stdio::inherit())
        .status()
        .context(format!("Failed to execute {}", program))?;
    if !status.success() {
        anyhow::bail!("{} process exited with code {:?}", program, status.code());
    }
    Ok(())
}

fn run_java(file: &str, verbose: bool) -> Result<()> {
    if verbose {
        println!("{} Running Java: {}", ">>".cyan().bold(), file);
    }
    // For .java files, compile first then run the class
    let path = Path::new(file);
    let class_name = path
        .file_stem()
        .context("Invalid Java filename")?
        .to_str()
        .context("Invalid filename")?;

    // Compile
    let compile_status = Command::new("javac")
        .arg(file)
        .stdin(std::process::Stdio::inherit())
        .stdout(std::process::Stdio::inherit())
        .stderr(std::process::Stdio::inherit())
        .status()
        .context("Failed to compile Java file")?;
    if !compile_status.success() {
        anyhow::bail!("Java compilation failed with code {:?}", compile_status.code());
    }

    // Run
    let run_status = Command::new("java")
        .arg(class_name)
        .stdin(std::process::Stdio::inherit())
        .stdout(std::process::Stdio::inherit())
        .stderr(std::process::Stdio::inherit())
        .status()
        .context("Failed to execute java")?;
    if !run_status.success() {
        anyhow::bail!("Java process exited with code {:?}", run_status.code());
    }
    Ok(())
}

fn run_rust(file: &str, verbose: bool) -> Result<()> {
    if verbose {
        println!("{} Running Rust: {}", ">>".cyan().bold(), file);
    }
    let path = Path::new(file);
    if path.file_name().map_or(false, |f| f.to_str() == Some("Cargo.toml")) {
        // It's a Cargo project
        let status = Command::new("cargo")
            .arg("run")
            .current_dir(path.parent().unwrap_or(Path::new(".")))
            .stdin(std::process::Stdio::inherit())
            .stdout(std::process::Stdio::inherit())
            .stderr(std::process::Stdio::inherit())
            .status()
            .context("Failed to execute cargo run")?;
        if !status.success() {
            anyhow::bail!("Cargo process exited with code {:?}", status.code());
        }
    } else {
        // Single file - use cargo-script or rustc + run
        let status = Command::new("rustc")
            .arg(file)
            .arg("-o")
            .arg("fusion_rust_temp")
            .stdin(std::process::Stdio::inherit())
            .stdout(std::process::Stdio::inherit())
            .stderr(std::process::Stdio::inherit())
            .status()
            .context("Failed to compile Rust file")?;
        if !status.success() {
            anyhow::bail!("Rust compilation failed with code {:?}", status.code());
        }
        let run_status = Command::new("./fusion_rust_temp")
            .stdin(std::process::Stdio::inherit())
            .stdout(std::process::Stdio::inherit())
            .stderr(std::process::Stdio::inherit())
            .status()?;
        let _ = std::fs::remove_file("fusion_rust_temp");
        if !run_status.success() {
            anyhow::bail!("Rust program exited with code {:?}", run_status.code());
        }
    }
    Ok(())
}

pub async fn run(file: Option<&str>, lang: Option<&str>, polyglot: bool, verbose: bool) -> Result<()> {
    let file = match file {
        Some(f) => f.to_string(),
        None => {
            let config_path = crate::config::FusionConfig::find_config_file()?;
            let root = config_path.parent().context("Invalid config path")?;
            let main_file = root.join("src").join("main.fusion");
            if main_file.exists() {
                main_file.display().to_string()
            } else {
                anyhow::bail!("No file specified and no src/main.fusion found. Run `fusion run <file>`.");
            }
        }
    };

    if !std::path::Path::new(&file).exists() {
        anyhow::bail!("File not found: {}", file);
    }

    if verbose {
        println!("{} Running {}", ">>".cyan().bold(), file);
    }

    // Explicit --lang flag takes precedence
    if let Some(lang) = lang {
        match lang {
            "python" | "py" => return run_python(&file, verbose),
            "javascript" | "js" | "node" => return run_javascript(&file, verbose),
            "java" => return run_java(&file, verbose),
            "rust" | "rs" => return run_rust(&file, verbose),
            "fusion" | "fu" => return run_fusion(&file, verbose),
            "c" | "cpp" | "cxx" => {
                // Compile and run C/C++
                let file_clone = file.clone();
                let ext = Path::new(&file_clone).extension().and_then(|e| e.to_str()).unwrap_or("c");
                let compiler = if ext == "c" { "gcc" } else { "g++" };
                let out_name = "fusion_c_temp";
                let compile_status = Command::new(compiler)
                    .arg(&file_clone)
                    .arg("-o")
                    .arg(out_name)
                    .stdin(std::process::Stdio::inherit())
                    .stdout(std::process::Stdio::inherit())
                    .stderr(std::process::Stdio::inherit())
                    .status()
                    .context(format!("Failed to compile {}", ext))?;
                if !compile_status.success() {
                    anyhow::bail!("{} compilation failed", ext);
                }
                let run_status = Command::new(format!("./{}", out_name))
                    .stdin(std::process::Stdio::inherit())
                    .stdout(std::process::Stdio::inherit())
                    .stderr(std::process::Stdio::inherit())
                    .status()?;
                let _ = std::fs::remove_file(out_name);
                if !run_status.success() {
                    anyhow::bail!("{} program exited with code {:?}", ext, run_status.code());
                }
                return Ok(());
            }
            _ => anyhow::bail!("Unsupported language: {}. Supported: python, javascript, java, rust, c, cpp, fusion", lang),
        }
    }

    // --polyglot: auto-detect language from extension
    if polyglot {
        let detected = detect_lang(&file).unwrap_or("fusion");
        if verbose {
            println!("{} Detected language: {}", ">>".cyan().bold(), detected);
        }
        return match detected {
            "python" => run_python(&file, verbose),
            "javascript" => run_javascript(&file, verbose),
            "java" => run_java(&file, verbose),
            "rust" => run_rust(&file, verbose),
            "fusion" | _ => run_fusion(&file, verbose),
        };
    }

    // Default: run as Fusion
    run_fusion(&file, verbose)
}
