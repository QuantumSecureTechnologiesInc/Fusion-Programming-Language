// src/codegen/mod.rs - Code Generation Backends

use crate::ir;

#[derive(Debug, Clone)]
pub struct CodegenConfig {
    pub optimization_level: u32,
    pub target_triple: String,
    pub link_libs: Vec<String>,
}

impl Default for CodegenConfig {
    fn default() -> Self {
        Self {
            optimization_level: 0,
            target_triple: "x86_64-unknown-linux-gnu".to_string(),
            link_libs: Vec::new(),
        }
    }
}

#[derive(Debug)]
pub enum CodegenError {
    LlvmError(String),
    Unsupported(String),
}

impl std::fmt::Display for CodegenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CodegenError::LlvmError(s) => write!(f, "LLVM error: {}", s),
            CodegenError::Unsupported(s) => write!(f, "Unsupported: {}", s),
        }
    }
}

impl std::error::Error for CodegenError {}

pub trait Backend {
    fn compile_module(&mut self, module: &ir::IrModule) -> Result<Vec<u8>, CodegenError>;
}

// Conditionally include LLVM backend when inkwell is available
#[cfg(feature = "llvm")]
pub mod llvm_instsel;
#[cfg(feature = "llvm")]
pub mod llvm_backend;