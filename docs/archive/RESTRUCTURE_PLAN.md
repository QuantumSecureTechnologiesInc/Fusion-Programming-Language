# Fusion v2.0 Vortex -- Phase 0 Restructure Plan

**Generated**: 2026-07-01
**Based on**: inventory (14,401 files, 317 MB) + truth audit (12 PASS, 28 FAIL, 7 PARTIAL, 8 UNVERIFIED)

## Executive Summary

| Action | Count | Notes |
|--------|-------|-------|
| KEEP_AS_IS | ~12,000 | Core source, config, docs |
| RELOCATE | ~50 | Move to correct locations |
| ARCHIVE | ~500 | .archive/ for historical reference |
| TRASH | ~2,000 | Root junk, build artifacts, duplicates |
| REWRITE | ~30 | .fu stubs needing real implementation |
| FLAG_FOR_HUMAN | ~20 | Needs human decision |

Expected disk freed by TRASH + ARCHIVE: ~80 MB

## Target Layout

```
Fusion v2.0 Vortex/
├── bin/                    # Bootstrap binaries (KEEP)
│   └── fuc.exe
├── crates/                 # Rust crates (KEEP)
│   ├── fuc/                # Compiler core
│   └── fuc2/               # Self-hosting driver
├── src/                    # Fusion source (KEEP)
├── stdlib/                 # Standard library (KEEP)
├── runtime/                # Native runtime (KEEP)
├── registry/               # Package registry (KEEP)
├── docs/                   # Documentation (KEEP)
├── tests/                  # Test suites (KEEP)
├── examples/               # Example code (KEEP)
├── scripts/                # Build/CI scripts (KEEP)
├── .archive/               # Archived files
│   ├── toy-compiler/       # Early compiler prototypes
│   ├── build-artifacts/    # Old .o, .stage.o, .ll files
│   └── build-logs/         # Old build logs
├── inventory/              # Phase 0 outputs
├── docs-truth-audit/       # Phase 0 outputs
├── restructure/            # This plan
├── Cargo.toml              # (KEEP)
├── Fusion.toml             # (KEEP)
└── README.md               # (KEEP)
```

## Action Lists

### ARCHIVE: Toy Compiler Prototypes
Move to `.archive/toy-compiler/`:
- `compiler.fu`, `compiler_v1.fu`, `compiler_v2.fu`, `compiler_v3.fu`
- `compiler.exe`, `compiler_v3.exe`
- `compiler.o`, `compiler_v3.o`
- `compiler_helpers.c`, `compiler_helpers.o`
- `hello.fu`, `hello.ll`, `hello.o`, `hello_compiled.exe`
- `helpers.c`, `helpers.o`
- `test_runtime.c`

### ARCHIVE: Build Artifacts
Move to `.archive/build-artifacts/`:
- All root `*.stage.o` files (~90 files)
- All root `*.stage.ll` files
- `test_simple_compiler.fu`

### ARCHIVE: Build Logs
Move to `.archive/build-logs/`:
- `build_debug.log`, `build_error.log`, `build_error_v2.log`
- `build_failure.log`, `build_failure_20.log`, `build_failure_20_utf8.log`
- `build_failure_utf8.log`, `build_final.log`
- `ninja_build.log`, `ninja_config.log`
- `mock_build_error.log`, `firebase-debug.log`

### TRASH (Flag for Human Review)
These require `mavis-trash` or manual deletion:
- Root `*.exe` files (test artifacts, not in bin/)
- Root `*.o` files (test artifacts)
- Root `*.lib`, `*.a` files
- `target_fuc/`, `target_fuc2/`, `target_fuc_native/`, `target_fusion_cli/` directories
- `v` (empty file)
- `panic!(strcmp should return int)` (empty file)
- `%TEMP%install-qwen.bat`
- `cmake_build/` directory
- Duplicate `Fusion v2.0 Vortex/` nested directory

### REWRITE (Flag for Future)
These .fu files are stubs needing real implementation:
- `src/stdlib/vortex.fu` (2KB stub, not a real entropy engine)
- `src/ml/*.fu` (placeholder neural net primitives)
- `src/quantum/*.fu` (simulator only, no real QPU)
- `crates/fuc/src/vortex.rs` (2-line stub)
- `crates/fuc/src/net.rs` (39-line stub)

### FLAG_FOR_HUMAN
- `BUILD_POLICY.md` -- References `fusion build` command that doesn't exist
- `FUSION_VS_RUST.md` -- Marketing comparison, needs fact-checking
- `docs/launch/PRESS_RELEASE.md` -- Premature
- `docs/roadmap/THE_FINAL_VERDICT.md` -- Overstates readiness
- `docs/roadmap/FINAL_ACCURATE_STATUS.md` -- Overstates readiness

## Risks & Rollback

- All ARCHIVE moves use `git mv` where files are tracked (history preserved)
- All ARCHIVE moves use `Copy-Item` + verify for untracked files
- TRASH items require `mavis-trash` (not available in this session -- flagged for human)
- `cargo check -p fuc` passes before and after any KEEP-area changes

## Deferred Items

- Full cleanup of `registry/crates/` (262 dirs, many stubs) -- requires per-crate audit
- `AI Training/` directory cleanup -- 67 files, mostly aspirational docs
- `examples/converted/` cleanup -- 2,700 files classified as converted-rust-masquerade
- `source_archives/` cleanup -- compressed source archives
- `vcpkg/` cleanup -- vendored dependencies

## Execution Status

- [x] ARCHIVE: toy-compiler (not executed -- files already absent or cleaned)
- [x] ARCHIVE: build-artifacts (not executed -- files already absent or cleaned)
- [ ] ARCHIVE: build-logs (not executed)
- [ ] TRASH items: FLAGGED FOR HUMAN (requires mavis-trash or manual deletion)
- [ ] Nested `Fusion v2.0 Vortex/` directory: FLAGGED FOR HUMAN
- [x] Keep areas verified: `cargo check -p fuc` passes cleanly