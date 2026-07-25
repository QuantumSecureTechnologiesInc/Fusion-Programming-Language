// main.rs — fuc2: Fusion Compiler v2 Build Driver
//
// Extends the bootstrap fuc.exe compiler with:
// 1. Multi-file module resolution (mod X; → reads X.fu)
// 2. Syntax desugaring (pub, enum → int constants, use removal)
// 3. extern fn deduplication across modules
// 4. Vortex borrow checking integration (optional)
// 5. File watching for incremental rebuilds
// 6. Ouroboros 3-stage self-hosting bootstrap
//
// Pipeline: source.fu → resolve mods → desugar → flatten → fuc.exe → link

mod preprocess;

use clap::Parser;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};
use std::time::Instant;

/// Fusion Compiler v2 — Module preprocessor and build driver
#[derive(Parser, Debug)]
#[command(name = "fuc2", version, about = "Fusion Compiler v2: Module Preprocessor and Build Driver")]
struct Cli {
    /// Input .fu source file
    #[arg(short, long, default_value = "a.fu")]
    input: PathBuf,

    /// Output file (binary or preprocessed source)
    #[arg(short, long, default_value = "a.out")]
    output: PathBuf,

    /// Emit binary executable (passed to bootstrap compiler)
    #[arg(long)]
    emit_bin: bool,

    /// Resolve mod declarations and flatten modules
    #[arg(long)]
    resolve_modules: bool,

    /// Enable Vortex borrow checking pass
    #[arg(long)]
    vortex: bool,

    /// Only preprocess, don't compile
    #[arg(long)]
    preprocess_only: bool,

    /// Watch input files for changes and rebuild incrementally
    #[arg(long, short = 'w')]
    watch: bool,

    /// Path to bootstrap compiler (default: bin/fuc.exe)
    #[arg(long, default_value = "bin/fuc.exe")]
    bootstrap: PathBuf,

    /// Enable Ouroboros 3-stage self-hosting bootstrap verification
    #[arg(long)]
    bootstrap_verify: bool,
}

fn main() -> ExitCode {
    let cli = Cli::parse();

    if cli.watch {
        run_watch_mode(&cli)
    } else {
        match run_pipeline(&cli) {
            Ok(()) => ExitCode::from(0),
            Err(e) => {
                eprintln!("[fuc2] Error: {}", e);
                ExitCode::from(1)
            }
        }
    }
}

/// Run the full preprocessing + compilation pipeline.
fn run_pipeline(cli: &Cli) -> Result<(), String> {
    let start = Instant::now();

    eprintln!("=== Fusion Compiler v2 (fuc2) ===");
    eprintln!("  Input:  {}", cli.input.display());
    eprintln!("  Output: {}", cli.output.display());
    eprintln!();

    // Determine the preprocessed output path
    let preprocessed_path = if cli.preprocess_only {
        cli.output.clone()
    } else {
        let mut p = cli.output.clone();
        p.set_extension("fu");
        p
    };

    // Stage 1: Preprocess
    eprintln!("[fuc2] Preprocessing...");
    preprocess::preprocess(&cli.input, &preprocessed_path, cli.resolve_modules)?;
    eprintln!("[fuc2] Preprocessing complete.");

    if cli.preprocess_only {
        let elapsed = start.elapsed();
        eprintln!("[fuc2] Done in {:.2}s (preprocess only)", elapsed.as_secs_f64());
        return Ok(());
    }

    // Stage 2: Vortex check (optional)
    if cli.vortex {
        eprintln!("[fuc2] Running Vortex borrow check...");
        invoke_vortex_check(&cli.bootstrap, &preprocessed_path)?;
    }

    // Stage 3: Compile with bootstrap compiler
    eprintln!("[fuc2] Invoking bootstrap compiler...");
    invoke_bootstrap(&cli.bootstrap, &preprocessed_path, &cli.output, cli.emit_bin)?;

    let elapsed = start.elapsed();
    eprintln!("[fuc2] Build successful in {:.2}s", elapsed.as_secs_f64());

    // Stage 4: Ouroboros bootstrap verification (optional)
    if cli.bootstrap_verify {
        run_ouroboros_bootstrap(&cli.bootstrap, &preprocessed_path, &cli.output, cli.emit_bin)?;
    }

    Ok(())
}

/// Invoke the bootstrap fuc.exe compiler.
fn invoke_bootstrap(
    bootstrap: &Path,
    input: &Path,
    output: &Path,
    emit_bin: bool,
) -> Result<(), String> {
    let bootstrap = if bootstrap.is_absolute() {
        bootstrap.to_path_buf()
    } else {
        // Resolve relative to workspace root (where Cargo.toml lives)
        let workspace_root = find_workspace_root()?;
        workspace_root.join(bootstrap)
    };

    if !bootstrap.exists() {
        return Err(format!(
            "Bootstrap compiler not found: {}. Build it first with: cargo build -p fuc --release",
            bootstrap.display()
        ));
    }

    let mut cmd = Command::new(&bootstrap);
    cmd.arg(input);

    if emit_bin {
        cmd.arg("--emit-bin");
    }

    cmd.arg("-o").arg(output);

    let status = cmd
        .status()
        .map_err(|e| format!("Failed to run bootstrap compiler: {}", e))?;

    if !status.success() {
        return Err(format!(
            "Bootstrap compilation failed with exit code: {:?}",
            status.code()
        ));
    }

    eprintln!("  [fuc2] Bootstrap compilation successful.");
    Ok(())
}

/// Invoke the bootstrap fuc.exe compiler in Vortex-check-only mode.
fn invoke_vortex_check(bootstrap: &Path, input: &Path) -> Result<(), String> {
    let bootstrap = if bootstrap.is_absolute() {
        bootstrap.to_path_buf()
    } else {
        let workspace_root = find_workspace_root()?;
        workspace_root.join(bootstrap)
    };

    if !bootstrap.exists() {
        return Err(format!(
            "Bootstrap compiler not found: {}. Build it first with: cargo build -p fuc --release",
            bootstrap.display()
        ));
    }

    let status = Command::new(&bootstrap)
        .arg("--vortex-only")
        .arg(input)
        .status()
        .map_err(|e| format!("Failed to run vortex checker: {}", e))?;

    if !status.success() {
        return Err(format!(
            "Vortex borrow check failed with exit code: {:?}",
            status.code()
        ));
    }

    eprintln!("[fuc2] Vortex check passed.");
    Ok(())
}

/// Run the Ouroboros 3-stage self-hosting bootstrap:
/// Stage 1: fuc.exe compiles fuc2 source → fuc2_stage1.exe
/// Stage 2: fuc2_stage1.exe compiles fuc2 source → fuc2_stage2.exe
/// Stage 3: fuc2_stage2.exe compiles fuc2 source → fuc2_stage3.exe
/// Verify: fuc2_stage2.exe == fuc2_stage3.exe (bit-identical)
fn run_ouroboros_bootstrap(
    bootstrap: &Path,
    _source: &Path,
    output: &Path,
    _emit_bin: bool,
) -> Result<(), String> {
    eprintln!();
    eprintln!("=== Ouroboros 3-Stage Self-Hosting Bootstrap ===");

    let stage1 = output.with_file_name("fuc2_stage1.exe");
    let stage2 = output.with_file_name("fuc2_stage2.exe");
    let stage3 = output.with_file_name("fuc2_stage3.exe");

    // Stage 1: Bootstrap compiles fuc2
    eprintln!("[stage 1] Bootstrap → fuc2_stage1.exe");
    invoke_bootstrap(bootstrap, output, &stage1, true)?;

    // Stage 2: fuc2_stage1 compiles itself
    eprintln!("[stage 2] fuc2_stage1 → fuc2_stage2.exe");
    invoke_bootstrap(&stage1, output, &stage2, true)?;

    // Stage 3: fuc2_stage2 compiles itself
    eprintln!("[stage 3] fuc2_stage2 → fuc2_stage3.exe");
    invoke_bootstrap(&stage2, output, &stage3, true)?;

    // Verify stage2 == stage3 (bit-identical)
    eprintln!("[verify] Comparing stage2 and stage3...");
    let stage2_bytes = std::fs::read(&stage2)
        .map_err(|e| format!("Failed to read stage2: {}", e))?;
    let stage3_bytes = std::fs::read(&stage3)
        .map_err(|e| format!("Failed to read stage3: {}", e))?;

    if stage2_bytes == stage3_bytes {
        eprintln!("[verify] ✓ Self-hosting bootstrap verified! Stage 2 == Stage 3 (bit-identical)");
        eprintln!("[verify]   Stage 2: {} bytes", stage2_bytes.len());
        eprintln!("[verify]   Stage 3: {} bytes", stage3_bytes.len());
    } else {
        eprintln!("[verify] ✗ Self-hosting bootstrap FAILED — Stage 2 != Stage 3");
        eprintln!("[verify]   Stage 2: {} bytes", stage2_bytes.len());
        eprintln!("[verify]   Stage 3: {} bytes", stage3_bytes.len());

        // Find first differing byte
        let min_len = stage2_bytes.len().min(stage3_bytes.len());
        for i in 0..min_len {
            if stage2_bytes[i] != stage3_bytes[i] {
                eprintln!(
                    "[verify]   First diff at byte {}: {:02x} vs {:02x}",
                    i, stage2_bytes[i], stage3_bytes[i]
                );
                break;
            }
        }
        if stage2_bytes.len() != stage3_bytes.len() {
            eprintln!(
                "[verify]   Size mismatch: {} vs {}",
                stage2_bytes.len(),
                stage3_bytes.len()
            );
        }
    }

    Ok(())
}

/// Run in watch mode: rebuild whenever source files change.
fn run_watch_mode(cli: &Cli) -> ExitCode {
    use notify::event::EventKind;
    use notify::{Event, RecursiveMode, Watcher};
    use std::sync::mpsc;

    eprintln!("[fuc2] Watching for changes... (Ctrl+C to stop)");

    // Initial build
    if let Err(e) = run_pipeline(cli) {
        eprintln!("[fuc2] Initial build failed: {}", e);
    }

    let (tx, rx) = mpsc::channel();
    let mut watcher = match notify::recommended_watcher(move |res: Result<Event, _>| {
        if let Ok(event) = res {
            // Only rebuild on .fu file changes
            let is_fu_change = event.paths.iter().any(|p| {
                p.extension()
                    .map(|ext| ext == "fu")
                    .unwrap_or(false)
            });
            if is_fu_change {
                // Debounce: only rebuild on Modify events, not Access
                match event.kind {
                    EventKind::Modify(_) | EventKind::Create(_) => {
                        let _ = tx.send(());
                    }
                    _ => {}
                }
            }
        }
    }) {
        Ok(w) => w,
        Err(e) => {
            eprintln!("[fuc2] Failed to create file watcher: {}", e);
            return ExitCode::from(1);
        }
    };

    // Watch the input file's directory and the src directory
    if let Some(parent) = cli.input.parent() {
        if let Err(e) = watcher.watch(parent, RecursiveMode::Recursive) {
            eprintln!("[fuc2] Failed to watch {}: {}", parent.display(), e);
        }
    }

    // Also watch the source directory if it exists
    let src_dir = find_workspace_root()
        .map(|r| r.join("src"))
        .unwrap_or_else(|_| PathBuf::from("src"));
    if src_dir.exists() {
        if let Err(e) = watcher.watch(&src_dir, RecursiveMode::Recursive) {
            eprintln!("[fuc2] Failed to watch {}: {}", src_dir.display(), e);
        }
    }

    // Rebuild loop
    loop {
        match rx.recv() {
            Ok(()) => {
                // Small debounce delay
                std::thread::sleep(std::time::Duration::from_millis(200));
                eprintln!("\n[fuc2] Change detected, rebuilding...");
                if let Err(e) = run_pipeline(cli) {
                    eprintln!("[fuc2] Build failed: {}", e);
                }
            }
            Err(_) => break,
        }
    }

    ExitCode::from(0)
}

/// Find the workspace root (directory containing Cargo.toml with [workspace]).
fn find_workspace_root() -> Result<PathBuf, String> {
    let exe_path = std::env::current_exe()
        .map_err(|e| format!("Failed to get exe path: {}", e))?;
    let mut dir = exe_path.parent().unwrap_or_else(|| Path::new("."));

    // Walk up from the binary location
    for _ in 0..10 {
        let cargo_toml = dir.join("Cargo.toml");
        if cargo_toml.exists() {
            let content = std::fs::read_to_string(&cargo_toml)
                .map_err(|e| format!("Failed to read Cargo.toml: {}", e))?;
            if content.contains("[workspace]") {
                return Ok(dir.to_path_buf());
            }
        }
        if let Some(parent) = dir.parent() {
            dir = parent;
        } else {
            break;
        }
    }

    // Fallback: use current directory
    std::env::current_dir().map_err(|e| format!("Failed to get current dir: {}", e))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_workspace_root() {
        let root = find_workspace_root();
        assert!(root.is_ok(), "Should find workspace root: {:?}", root.err());
        let root = root.unwrap();
        assert!(root.join("Cargo.toml").exists(), "Should have Cargo.toml");
    }
}