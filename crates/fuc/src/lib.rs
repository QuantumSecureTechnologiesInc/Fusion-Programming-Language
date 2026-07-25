//! Fusion Compiler (fuc) - Rust compiler core library.
//! This crate provides the Rust-based compiler backend for the Fusion language.
//! The self-hosted Fusion sources (.fu files) live alongside these Rust modules.

// Core types (must come first)
pub mod types;

// AST and type definitions
pub mod ast_types;
pub mod ast;
pub mod ir;
pub mod ir_inliner;
pub mod ir_lower;

// Frontend stages
pub mod parser;
pub mod lexer;
pub mod sema;
pub mod cli;

// Type system and analysis
pub mod borrowck;
pub mod monomorphize;
pub mod vortex;

// Intermediate representation
pub mod ssa;
pub mod optimizer;
pub mod optimizer_cfg;

// Feature toggle engine
pub mod feature_toggle;

// Interaction witness system (cross-feature compatibility verification)
pub mod interaction_witness;

// Code generation
pub mod codegen;
pub mod wasm;

// Tooling and developer experience
pub mod fmt;
pub mod lsp;
pub mod linter;
pub mod macros;
pub mod profiler;
pub mod test_framework;
pub mod rtti;

// Diagnostics
pub mod diagnostics;

// Build system
pub mod forge;
pub mod forge_pkg;
pub mod forge_orch;
pub mod cache;
pub mod sysroot;
pub mod graph;
pub mod fingerprint;
pub mod linker;

// Runtime subsystems
pub mod runtime;
pub mod gc;
pub mod reactor;
pub mod tensorweave;

// Networking and distributed systems
pub mod net;
pub mod nexus;
pub mod solver;
pub mod cluster;

// Process supervision
pub mod warden_daemon;
pub mod warden_supervisor;

// Standard library and utilities
pub mod fs;
pub mod unicode;
pub mod stdlib;
pub mod mem;

// Security
pub mod neuralseal;
pub mod pqc;

// Platform-specific
pub mod metal_drivers;