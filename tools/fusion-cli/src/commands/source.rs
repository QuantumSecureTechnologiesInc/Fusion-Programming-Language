use anyhow::{Context, Result};
use colored::Colorize;
use std::path::{Path, PathBuf};
use std::process::Command;
use walkdir::WalkDir;

use super::SourceSubcommand;

pub async fn run(subcmd: SourceSubcommand, json: bool) -> Result<()> {
    match subcmd {
        SourceSubcommand::List => list(json).await,
        SourceSubcommand::Compile { file } => compile(&file, json).await,
        SourceSubcommand::Test { filter } => test(filter.as_deref(), json).await,
        SourceSubcommand::Audit => audit(json).await,
    }
}

fn find_source_files_dir() -> Result<PathBuf> {
    // Walk up from cwd looking for "Source Files" directory
    let mut dir = std::env::current_dir().context("Failed to get current directory")?;
    loop {
        let candidate = dir.join("Source Files");
        if candidate.exists() && candidate.is_dir() {
            return Ok(candidate);
        }
        let parent = dir.join("..");
        if parent == dir {
            break;
        }
        dir = parent;
    }
    anyhow::bail!(
        "No 'Source Files' directory found. Run this command from the Fusion project root."
    )
}

fn collect_fu_files(root: &Path) -> Vec<PathBuf> {
    WalkDir::new(root)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.file_type().is_file()
                && e.path()
                    .extension()
                    .map(|ext| ext == "fu")
                    .unwrap_or(false)
        })
        .map(|e| e.into_path())
        .collect()
}

async fn list(json: bool) -> Result<()> {
    let source_dir = find_source_files_dir()?;
    let files = collect_fu_files(&source_dir);

    if json {
        let entries: Vec<serde_json::Value> = files
            .iter()
            .map(|f| {
                let rel = f.strip_prefix(&source_dir).unwrap_or(f);
                serde_json::json!({
                    "path": rel.display().to_string(),
                    "size": std::fs::metadata(f).map(|m| m.len()).unwrap_or(0),
                })
            })
            .collect();
        let result = serde_json::json!({
            "action": "source_list",
            "count": entries.len(),
            "files": entries,
        });
        println!("{}", serde_json::to_string_pretty(&result)?);
    } else {
        println!(
            "{} {}",
            "Source Files:".cyan().bold(),
            format!("{} files found", files.len()).dimmed()
        );
        for f in &files {
            let rel = f.strip_prefix(&source_dir).unwrap_or(f);
            let size = std::fs::metadata(f).map(|m| m.len()).unwrap_or(0);
            let size_str = if size > 1024 {
                format!("{} KB", size / 1024)
            } else {
                format!("{} B", size)
            };
            println!("  {} ({})", rel.display().to_string().green(), size_str.dimmed());
        }
    }

    Ok(())
}

async fn compile(file: &str, json: bool) -> Result<()> {
    let source_dir = find_source_files_dir()?;
    let file_path = if Path::new(file).is_absolute() {
        PathBuf::from(file)
    } else {
        source_dir.join(file)
    };

    if !file_path.exists() {
        anyhow::bail!("File not found: {}", file_path.display());
    }

    println!(
        "{} Compiling {}...",
        "Source:".cyan().bold(),
        file_path.display().to_string().yellow()
    );

    let args = vec![
        "build".to_string(),
        file_path.display().to_string(),
    ];

    let output = Command::new("fuc.exe")
        .args(&args)
        .output()
        .context("Failed to execute fuc.exe build")?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let success = output.status.success();

    if json {
        let result = serde_json::json!({
            "action": "source_compile",
            "file": file_path.display().to_string(),
            "success": success,
            "output": stdout.trim().to_string(),
        });
        println!("{}", serde_json::to_string_pretty(&result)?);
    } else {
        if success {
            println!(
                "{} Compiled {}",
                "OK".green().bold(),
                file_path.display().to_string()
            );
            if !stdout.is_empty() {
                println!("{}", stdout);
            }
        } else {
            println!(
                "{} Failed to compile {}",
                "FAIL".red().bold(),
                file_path.display().to_string()
            );
            if !stderr.is_empty() {
                eprintln!("{}", stderr);
            }
            anyhow::bail!("Compilation failed");
        }
    }

    Ok(())
}

async fn test(filter: Option<&str>, json: bool) -> Result<()> {
    let source_dir = find_source_files_dir()?;
    let files = collect_fu_files(&source_dir);

    let test_files: Vec<&PathBuf> = files
        .iter()
        .filter(|f| {
            let name = f
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("");
            let is_test = name.contains("test") || name.contains("Test");
            match filter {
                Some(filt) => is_test && name.contains(filt),
                None => is_test,
            }
        })
        .collect();

    if test_files.is_empty() {
        if json {
            let result = serde_json::json!({
                "action": "source_test",
                "total": 0,
                "passed": 0,
                "failed": 0,
            });
            println!("{}", serde_json::to_string_pretty(&result)?);
        } else {
            println!("{} No test files found.", "OK".green().bold());
        }
        return Ok(());
    }

    let mut passed = 0u32;
    let mut failed = 0u32;
    let total = test_files.len() as u32;

    if !json {
        println!(
            "{} Running {} test file(s)...",
            "Source:".cyan().bold(),
            total
        );
    }

    for f in &test_files {
        let rel = f.strip_prefix(&source_dir).unwrap_or(f);
        let args = vec!["test".to_string(), f.display().to_string()];

        let output = Command::new("fuc.exe")
            .args(&args)
            .output()
            .context("Failed to execute fuc.exe test")?;

        let success = output.status.success();

        if success {
            passed += 1;
            if !json {
                println!("  {} {}", "PASS".green().bold(), rel.display().to_string());
            }
        } else {
            failed += 1;
            let stderr = String::from_utf8_lossy(&output.stderr);
            if !json {
                println!("  {} {}", "FAIL".red().bold(), rel.display().to_string());
                if !stderr.is_empty() {
                    eprintln!("    {}", stderr.trim());
                }
            }
        }
    }

    if json {
        let result = serde_json::json!({
            "action": "source_test",
            "total": total,
            "passed": passed,
            "failed": failed,
        });
        println!("{}", serde_json::to_string_pretty(&result)?);
    } else {
        println!();
        if failed == 0 {
            println!(
                "{} {}/{} test files passed.",
                "OK".green().bold(),
                passed,
                total
            );
        } else {
            println!(
                "{} {}/{} test files failed.",
                "FAIL".red().bold(),
                failed,
                total
            );
            anyhow::bail!("{} test file(s) failed", failed);
        }
    }

    Ok(())
}

async fn audit(json: bool) -> Result<()> {
    let source_dir = find_source_files_dir()?;
    let files = collect_fu_files(&source_dir);

    let mut stubs: Vec<String> = Vec::new();
    let mut real: Vec<String> = Vec::new();
    let mut empty: Vec<String> = Vec::new();

    for f in &files {
        let rel = f
            .strip_prefix(&source_dir)
            .unwrap_or(f)
            .display()
            .to_string();

        let content = std::fs::read_to_string(f).unwrap_or_default();
        let trimmed = content.trim();

        if trimmed.is_empty() {
            empty.push(rel);
        } else if is_stub(trimmed) {
            stubs.push(rel);
        } else {
            real.push(rel);
        }
    }

    if json {
        let result = serde_json::json!({
            "action": "source_audit",
            "total": files.len(),
            "real": real.len(),
            "stubs": stubs.len(),
            "empty": empty.len(),
            "real_files": real,
            "stub_files": stubs,
            "empty_files": empty,
        });
        println!("{}", serde_json::to_string_pretty(&result)?);
    } else {
        println!(
            "{} Source Files Audit:",
            "Source:".cyan().bold()
        );
        println!(
            "  Total files: {}",
            files.len()
        );
        println!(
            "  {} {}",
            "Real implementations:".green().bold(),
            real.len()
        );
        println!(
            "  {} {}",
            "Stubs:".yellow().bold(),
            stubs.len()
        );
        println!(
            "  {} {}",
            "Empty:".red().bold(),
            empty.len()
        );

        if !stubs.is_empty() {
            println!("\n{}:", "Stub files".yellow());
            for s in &stubs {
                println!("  {}", s);
            }
        }
        if !empty.is_empty() {
            println!("\n{}:", "Empty files".red());
            for e in &empty {
                println!("  {}", e);
            }
        }
    }

    Ok(())
}

fn is_stub(content: &str) -> bool {
    let lower = content.to_lowercase();

    // Check for common stub indicators
    if lower.contains("todo!") || lower.contains("unimplemented!") {
        return true;
    }
    if lower.contains("// stub") || lower.contains("// placeholder") || lower.contains("// todo") {
        return true;
    }
    if lower.contains("fn main() {}") || lower.contains("fn main() {\n}") {
        return true;
    }

    // If the file only has comments or whitespace, it's effectively a stub
    let meaningful_lines: Vec<&str> = content
        .lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty() && !l.starts_with("//") && !l.starts_with("#["))
        .collect();

    meaningful_lines.len() <= 1
}
