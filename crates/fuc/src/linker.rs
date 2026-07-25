//! Fusion linker driver.
//!
//! Invokes the system linker to produce a final executable from compiled
//! object files and the Fusion runtime library.

use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{bail, Context, Result};

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Link a set of object files (and the Fusion runtime) into `output`.
///
/// The function locates the system linker, discovers pre-compiled runtime
/// objects shipped with the Fusion source tree, and constructs the correct
/// platform-specific link command.
pub fn link_bin(objects: &[String], output: &str) -> Result<()> {
    let linker = find_linker()?;
    let runtime_dir = find_runtime_dir()?;
    let runtime_objects = discover_runtime_objects(&runtime_dir);

    if runtime_objects.is_empty() {
        bail!(
            "no runtime objects found under {}; expected at least one .o or .a file",
            runtime_dir.display()
        );
    }

    let mut cmd = Command::new(&linker);

    // -----------------------------------------------------------------------
    // Platform-specific flags
    // -----------------------------------------------------------------------
    if cfg!(target_os = "windows") {
        // MSVC-style flags (link.exe / lld-link)
        // Output flag
        cmd.arg(format!("/OUT:{}", output));

        // Suppress default libs that conflict with the Fusion runtime
        cmd.arg("/NOLOGO");

        // Add runtime search paths and object files
        for obj in &runtime_objects {
            cmd.arg(obj);
        }
    } else if cfg!(target_os = "macos") {
        // macOS ld64 / ld-style flags
        cmd.arg("-o").arg(output);

        // Add a runtime library search path
        cmd.arg(format!("-L{}", runtime_dir.display()));

        // Link runtime objects directly
        for obj in &runtime_objects {
            cmd.arg(obj);
        }

        // Standard system libraries the runtime may depend on
        cmd.arg("-lm").arg("-lpthread");
    } else {
        // Linux / other ELF targets — ld-compatible flags
        cmd.arg("-o").arg(output);

        cmd.arg(format!("-L{}", runtime_dir.display()));

        for obj in &runtime_objects {
            cmd.arg(obj);
        }

        cmd.arg("-lm").arg("-lpthread").arg("-ldl");
    }

    // -----------------------------------------------------------------------
    // User-supplied object files (compiled .o files from the build graph)
    // -----------------------------------------------------------------------
    for obj in objects {
        cmd.arg(obj);
    }

    // -----------------------------------------------------------------------
    // Execute
    // -----------------------------------------------------------------------
    let status = cmd
        .status()
        .with_context(|| format!("failed to execute linker: {}", linker.display()))?;

    if !status.success() {
        bail!(
            "linker exited with status {}",
            status
                .code()
                .map_or_else(|| "signal".to_string(), |c| c.to_string())
        );
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// Linker discovery
// ---------------------------------------------------------------------------

/// Probe well-known linker binaries and return the first one that exists on
/// `PATH`.
fn find_linker() -> Result<PathBuf> {
    if cfg!(target_os = "windows") {
        // Prefer LLVM's lld-link (ships with Rust/LLVM toolchains), fall back
        // to MSVC link.exe.
        for candidate in &["lld-link", "link"] {
            if let Some(path) = which(candidate) {
                return Ok(path);
            }
        }
        bail!("no Windows linker found on PATH (tried lld-link, link)")
    } else {
        // Unix-like: prefer cc → gcc → clang.
        for candidate in &["cc", "gcc", "clang"] {
            if let Some(path) = which(candidate) {
                return Ok(path);
            }
        }
        bail!("no Unix linker found on PATH (tried cc, gcc, clang)")
    }
}

/// Portable `which` implementation — looks for `name` on `PATH` and returns
/// the full path if found.
fn which(name: &str) -> Option<PathBuf> {
    let path_var = env::var_os("PATH")?;
    let paths: Vec<PathBuf> = env::split_paths(&path_var).collect();

    for dir in paths {
        let candidate = dir.join(name);
        if candidate.exists() {
            return Some(candidate);
        }

        // Also try with common extensions on Windows
        if cfg!(target_os = "windows") {
            for ext in &[".exe", ".cmd", ".bat"] {
                let candidate = dir.join(format!("{}{}", name, ext));
                if candidate.exists() {
                    return Some(candidate);
                }
            }
        }
    }

    None
}

// ---------------------------------------------------------------------------
// Runtime object discovery
// ---------------------------------------------------------------------------

/// Return the path to the `runtime/` directory that contains pre-compiled
/// native objects. The lookup walks up from the current working directory to
/// find the project root (where the `runtime/` folder lives).
fn find_runtime_dir() -> Result<PathBuf> {
    let cwd = env::current_dir().context("failed to determine current directory")?;

    // Walk up from cwd looking for the `runtime/` directory
    let mut dir: &Path = &cwd;
    loop {
        let candidate = dir.join("runtime");
        if candidate.is_dir() {
            return Ok(candidate);
        }
        match dir.parent() {
            Some(parent) => dir = parent,
            None => break,
        }
    }

    // Fallback: check relative to the Cargo manifest directory (when building
    // from within the crates/fuc workspace).
    if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
        let candidate = PathBuf::from(&manifest_dir)
            .join("..")
            .join("..")
            .join("runtime");
        if candidate.is_dir() {
            return Ok(candidate);
        }
    }

    bail!(
        "could not locate the Fusion runtime directory; \
         expected a 'runtime/' folder in the project root"
    )
}

/// Collect all `.o` and `.a` files in `runtime/native/` (the pre-compiled
/// platform-specific runtime library). Also collects `.o` files from the
/// `runtime/` root directory (collection runtimes like vector, hashmap, hashset).
fn discover_runtime_objects(runtime_dir: &Path) -> Vec<String> {
    let mut objects = Vec::new();

    // 1. Collect from runtime/native/ (primary location)
    let native_dir = runtime_dir.join("native");
    if native_dir.is_dir() {
        collect_objects(&native_dir, &mut objects);
    }

    // 2. Collect from runtime/ root (vector_runtime.o, hashmap_runtime.o, etc.)
    //    but skip if native/ already provides them via a static library.
    let has_native_lib = objects.iter().any(|o| o.ends_with(".a"));
    if !has_native_lib {
        collect_objects(runtime_dir, &mut objects);
    }

    objects
}

/// Helper: collect .o and .a files from a single directory.
fn collect_objects(dir: &Path, objects: &mut Vec<String>) {
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            // Only pick up files, not subdirectories
            if !path.is_file() {
                continue;
            }
            if let Some(ext) = path.extension() {
                let ext_str = ext.to_string_lossy();
                if ext_str == "o" || ext_str == "a" {
                    let name = path
                        .file_stem()
                        .map(|s| s.to_string_lossy().to_string())
                        .unwrap_or_default();

                    // Skip objects clearly for a different platform
                    if cfg!(target_os = "windows") && name.contains("linux") {
                        continue;
                    }
                    if cfg!(target_os = "linux") && name.contains("win") {
                        continue;
                    }

                    objects.push(path.to_string_lossy().to_string());
                }
            }
        }
    }
}
