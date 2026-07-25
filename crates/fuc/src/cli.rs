//! Fusion CLI - Command-line interface for the Fusion compiler.
//! Provides argument parsing and the main entry point for the compiler.

use crate::types::*;

/// CLI configuration for the Fusion compiler.
#[derive(Debug, Clone)]
pub struct FusionCli {
    /// Input source files to compile.
    pub input_files: FVec<FString>,
    /// Output file path (default: a.out).
    pub output_file: FString,
    /// Optimization level (0-3).
    pub opt_level: u32,
    /// Parse only, skip code generation.
    pub parse_only: bool,
    /// Semantic analysis only, skip code generation.
    pub sema_only: bool,
    /// Emit LLVM IR instead of binary.
    pub emit_llvm: bool,
    /// Build as a library instead of executable.
    pub is_library: bool,
    /// Verbose output.
    pub verbose: bool,
    /// Target WASM instead of native code.
    pub target_wasm: bool,
    /// Run only the Vortex borrow check, then exit.
    pub vortex_only: bool,
}

impl Default for FusionCli {
    fn default() -> Self {
        Self {
            input_files: Vec::new(),
            output_file: "a.out".to_string(),
            opt_level: 0,
            parse_only: false,
            sema_only: false,
            emit_llvm: false,
            is_library: false,
            verbose: false,
            target_wasm: false,
            vortex_only: false,
        }
    }
}

impl FusionCli {
    /// Creates a new CLI instance from command-line arguments.
    pub fn from_args(args: &[FString]) -> Self {
        let mut cli = Self::default();
        let mut i = 0;
        while i < args.len() {
            match args[i].as_str() {
                "-o" => {
                    if i + 1 < args.len() {
                        cli.output_file = args[i + 1].clone();
                        i += 1;
                    }
                }
                "--parse-only" => cli.parse_only = true,
                "--sema-only" => cli.sema_only = true,
                "--vortex-only" => cli.vortex_only = true,
                "--emit-llvm" => cli.emit_llvm = true,
                "--lib" => cli.is_library = true,
                "-v" | "--verbose" => cli.verbose = true,
                "--target" => {
                    if i + 1 < args.len() && args[i + 1] == "wasm" {
                        cli.target_wasm = true;
                        i += 1;
                    }
                }
                "-O0" => cli.opt_level = 0,
                "-O1" => cli.opt_level = 1,
                "-O2" => cli.opt_level = 2,
                "-O3" => cli.opt_level = 3,
                arg if !arg.starts_with('-') => {
                    cli.input_files.push(arg.to_string());
                }
                _ => {}
            }
            i += 1;
        }
        cli
    }
}