use std::path::PathBuf;
use std::env;
use anyhow::{Result, bail};

pub struct Sysroot {
    pub std_path: PathBuf,
    pub runtime_obj: PathBuf,
}

impl Sysroot {
    /// Discovers the Fusion sysroot by checking multiple locations in priority order:
    /// 1. `FUSION_HOME` environment variable
    /// 2. Common installation paths (`/usr/local/lib/fusion`, `/usr/lib/fusion`, etc.)
    /// 3. Relative to the compiler binary (parent of parent of exe)
    /// Validates that stdlib files exist at the discovered path.
    pub fn discover() -> Result<Self> {
        // Priority 1: FUSION_HOME environment variable
        if let Ok(fusion_home) = env::var("FUSION_HOME") {
            let home = PathBuf::from(&fusion_home);
            if let Some(sysroot) = Self::validate_path(&home) {
                return Ok(sysroot);
            }
        }

        // Priority 2: Common installation paths
        let common_paths: Vec<PathBuf> = {
            let mut paths = vec![
                PathBuf::from("/usr/local/lib/fusion"),
                PathBuf::from("/usr/lib/fusion"),
                PathBuf::from("/opt/fusion"),
            ];

            // Windows-specific paths
            if cfg!(target_os = "windows") {
                if let Ok(program_files) = env::var("ProgramFiles") {
                    paths.push(PathBuf::from(program_files).join("Fusion").join("lib"));
                }
                if let Ok(app_data) = env::var("LOCALAPPDATA") {
                    paths.push(PathBuf::from(app_data).join("Fusion").join("lib"));
                }
            }

            // macOS path
            if cfg!(target_os = "macos") {
                paths.push(PathBuf::from("/opt/homebrew/lib/fusion"));
            }

            // XDG default
            if let Ok(home) = env::var("HOME") {
                paths.push(PathBuf::from(home).join(".fusion").join("lib"));
            }

            paths
        };

        for path in &common_paths {
            if let Some(sysroot) = Self::validate_path(path) {
                return Ok(sysroot);
            }
        }

        // Priority 3: Relative to the compiler binary
        if let Ok(exe) = env::current_exe() {
            if let Some(exe_dir) = exe.parent() {
                // Try parent of the binary directory (e.g., bin/../lib/fusion)
                if let Some(root) = exe_dir.parent() {
                    if let Some(sysroot) = Self::validate_path(&root.join("lib").join("fusion")) {
                        return Ok(sysroot);
                    }
                }
                // Try sibling directory
                if let Some(sysroot) = Self::validate_path(&exe_dir.join("fusion")) {
                    return Ok(sysroot);
                }
            }
        }

        bail!(
            "Fusion sysroot not found. Set the FUSION_HOME environment variable to the Fusion \
             installation directory (e.g., /usr/local/lib/fusion), or install Fusion to a \
             standard location."
        )
    }

    /// Validates that a candidate root contains the expected stdlib structure.
    /// Returns `Some(Sysroot)` if valid, `None` otherwise.
    fn validate_path(root: &PathBuf) -> Option<Self> {
        let std_path = root.join("std");
        let runtime_obj = root.join("runtime.o");

        // Check that at least the std directory exists and contains .fu files
        if std_path.exists() && std_path.is_dir() {
            // Look for at least one .fu file in std/ to confirm this is a real sysroot
            if let Ok(entries) = std::fs::read_dir(&std_path) {
                let has_stdlib = entries
                    .filter_map(|e| e.ok())
                    .any(|e| {
                        e.path()
                            .extension()
                            .and_then(|ext| ext.to_str())
                            == Some("fu")
                    });

                if has_stdlib {
                    return Some(Self {
                        std_path,
                        runtime_obj,
                    });
                }
            }
        }

        None
    }
}
