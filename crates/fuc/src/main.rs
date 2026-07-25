//! Fusion Compiler (fuc) — CLI binary entry point.
//! Reads .fu source files, lexes/parses them, runs semantic analysis,
//! Vortex borrow checking, IR lowering, optimization, and emits code.

use std::process;

use fuc::cli::FusionCli;
use fuc::lexer;
use fuc::parser;
use fuc::sema;
use fuc::vortex;
use fuc::ir_lower;
use fuc::optimizer;
use fuc::borrowck;
use fuc::diagnostics::chaos_vacuum::ChaosVacuumReporter;
#[cfg(feature = "wasm")]
use fuc::wasm::WasmCodeGenerator;

#[cfg(feature = "llvm")]
use fuc::codegen::Backend;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let cli = FusionCli::from_args(&args[1..]);

    if cli.input_files.is_empty() {
        eprintln!("fuc: no input files");
        eprintln!("Usage: fuc [--target wasm|native] [--parse-only] [--sema-only] [--emit-llvm] [--emit-bin] [-o output] [-v] <input.fu>");
        process::exit(1);
    }

    if cli.verbose {
        eprintln!("fuc: compiling {} file(s)", cli.input_files.len());
    }

    // Compile the first input file
    let input_path = &cli.input_files[0];
    let source = match std::fs::read_to_string(input_path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("fuc: error reading '{}': {}", input_path, e);
            process::exit(1);
        }
    };

    // Lex
    let token_stream = lexer::lex(&source);
    if cli.verbose {
        eprintln!("fuc: lexed {} tokens", token_stream.tokens.len());
    }

    // Parse
    let output = parser::parse_output(&source);
    if !output.errors.is_empty() {
        for err in &output.errors {
            eprintln!("fuc: parse error: {}", err);
        }
        process::exit(1);
    }
    let program = output.program.unwrap();
    if cli.verbose {
        eprintln!("fuc: parsed {} declarations, {} structs",
            program.declarations.len(), program.structs.len());
    }

    if cli.parse_only {
        eprintln!("fuc: parse succeeded [ok]");
        process::exit(0);
    }

    // Semantic analysis
    let mut analyzer = sema::Analyzer::new();
    let sema_out = analyzer.analyze_output(program);
    if !sema_out.errors.is_empty() {
        for err in &sema_out.errors {
            eprintln!("fuc: sema error: {}", err);
        }
        process::exit(1);
    }
    let typed_prog = match sema_out.program {
        Some(p) => p,
        None => {
            eprintln!("fuc: semantic analysis produced no program");
            process::exit(1);
        }
    };
    if cli.verbose {
        eprintln!("fuc: semantic analysis passed ({} functions, {} structs)",
            typed_prog.functions.len(), typed_prog.structs.len());
    }

    if cli.sema_only {
        eprintln!("fuc: semantic analysis succeeded [ok]");
        process::exit(0);
    }

    // Borrow checking (affine type tracking)
    let mut checker = borrowck::BorrowChecker::new();
    let borrow_errors = checker.check_program(&typed_prog);
    if !borrow_errors.is_empty() {
        for err in &borrow_errors {
            eprintln!("fuc: borrow error: {} at {:?}", err.message, err.span);
        }
        process::exit(1);
    }
    if cli.verbose {
        eprintln!("fuc: borrow checking passed");
    }

    // Vortex safety checking
    let mut vortex_ctx = vortex::VortexContext::new();
    let vortex_ok = vortex_ctx.verify_program(&typed_prog);
    if !vortex_ok {
        let reporter = ChaosVacuumReporter::new(input_path, &source);
        for collision in &vortex_ctx.collisions {
            reporter.publish_collision_report(collision);
        }
        eprintln!("fuc: vortex safety check failed");
        process::exit(1);
    }
    if cli.verbose {
        eprintln!("fuc: vortex safety check passed");
    }

    if cli.vortex_only {
        eprintln!("fuc: vortex safety check passed [ok]");
        process::exit(0);
    }

    // IR lowering
    let ir_module = ir_lower::lower_program(&typed_prog);
    if cli.verbose {
        eprintln!("fuc: IR lowered ({} functions, {} structs, {} externs)",
            ir_module.functions.len(), ir_module.structs.len(), ir_module.externs.len());
    }

    // Optimize
    let optimized = optimizer::optimize(ir_module);
    if cli.verbose {
        eprintln!("fuc: optimized ({} functions)", optimized.functions.len());
    }

    // Codegen
    if cli.target_wasm {
        // WASM codegen
        #[cfg(feature = "wasm")]
        {
            // Note: WASM codegen uses the original AST declarations, not the IR module
            // For now, re-parse (in a full pipeline this would be restructured)
            let output = parser::parse_output(&source);
            let program = output.program.unwrap();
            let mut codegen = WasmCodeGenerator::new();
            match codegen.generate(&program.declarations) {
                Ok(wasm_bytes) => {
                    let out_path = if cli.output_file == "a.out" {
                        "output.wasm".to_string()
                    } else {
                        cli.output_file.clone()
                    };
                    if let Err(e) = std::fs::write(&out_path, &wasm_bytes) {
                        eprintln!("fuc: error writing '{}': {}", out_path, e);
                        process::exit(1);
                    }
                    if cli.verbose {
                        eprintln!("fuc: wrote {} bytes to {}", wasm_bytes.len(), out_path);
                    }
                    eprintln!("fuc: compiled '{}' -> '{}' [ok]", input_path, out_path);
                }
                Err(e) => {
                    eprintln!("fuc: codegen error: {}", e);
                    process::exit(1);
                }
            }
        }
        #[cfg(not(feature = "wasm"))]
        {
            eprintln!("fuc: WASM backend not available (rebuild with --features wasm)");
            process::exit(1);
        }
    } else {
        // Native codegen via LLVM (if available)
        #[cfg(feature = "llvm")]
        {
            use fuc::codegen::llvm_backend::LlvmBackend;
            use inkwell::context::Context;

            let context = Context::create();
            let config = fuc::codegen::CodegenConfig::default();
            let mut backend = LlvmBackend::new(&context, input_path, &config);

            match backend.compile_module(&optimized) {
                Ok(_) => {
                    let out_path = if cli.output_file == "a.out" {
                        if cli.emit_llvm {
                            "output.ll".to_string()
                        } else {
                            "output.o".to_string()
                        }
                    } else {
                        cli.output_file.clone()
                    };

                    if cli.emit_llvm {
                        let llvm_ir = backend.write_to_string();
                        if let Err(e) = std::fs::write(&out_path, &llvm_ir) {
                            eprintln!("fuc: error writing '{}': {}", out_path, e);
                            process::exit(1);
                        }
                        eprintln!("fuc: emitted LLVM IR to '{}' [ok]", out_path);
                    } else {
                        match backend.write_object_file(&out_path) {
                            Ok(()) => eprintln!("fuc: compiled '{}' -> '{}' [ok]", input_path, out_path),
                            Err(e) => {
                                eprintln!("fuc: codegen error: {}", e);
                                process::exit(1);
                            }
                        }
                    }
                }
                Err(e) => {
                    eprintln!("fuc: LLVM codegen error: {}", e);
                    process::exit(1);
                }
            }
        }

        #[cfg(not(feature = "llvm"))]
        {
            eprintln!("fuc: no native backend available (build with --features llvm)");
            eprintln!("fuc: use --target wasm for WASM output");
            process::exit(1);
        }
    }
}