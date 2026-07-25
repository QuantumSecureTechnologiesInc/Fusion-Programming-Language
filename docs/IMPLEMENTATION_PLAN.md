# Fusion v2.0 Vortex — Full Implementation Plan

> Generated from deep codebase analysis. Phased, ordered by dependency and impact.

---

## Executive Summary

The Fusion compiler pipeline exists structurally (lexer → parser → sema → borrowck → vortex → IR lower → optimize → codegen) but has ~40 critical stubs that prevent producing working executables. The goal is a language where you can write programs, compile them, and run them.

**Architecture at a glance:**
- `crates/fuc/` — Rust compiler crate (64 source files)
- `crates/fuc2/` — Build driver / preprocessor (2 files)
- `stdlib/` — Fusion source standard library (46 .fu files)
- `runtime/` — C runtime linked into Fusion executables
- `registry/crates/` — ~250+ aspirational Rust crates (mostly stubs)

---

## PHASE 0: Critical Bug Fixes in Existing Code (Effort: Small)

**Goal:** Fix the 4 HIGH-severity SIMPLIFIED_PARTS that silently produce wrong output.

| # | File | Line(s) | Bug | Fix |
|---|------|---------|-----|-----|
| 1 | `crates/fuc/src/sema.rs` → `ir_lower.rs` | 502 | `MemberAccess` field_index already reads from `TypedExpressionKind` — **already fixed** in current source. Verify sema passes correct `field_index`. | Confirm sema.rs populates `field_index` correctly for all struct member accesses (not just field 0). |
| 2 | `crates/fuc/src/ir_lower.rs` | 561-694 | Match arms — **already implemented** with test blocks for int/bool/wildcard. String match still emits `Comment("TODO")`. | Add string pattern matching via runtime `strcmp` call. |
| 3 | `crates/fuc/src/optimizer_cfg.rs` | 73 | `func.blocks.retain(\|_b\| reachable.contains(&0))` — always keeps all blocks. | Change to `func.blocks.retain(\|i, _b\| reachable.contains(&i))`. Need to use `retain` with index tracking (Vec::retain doesn't give index — iterate and build new vec). |
| 4 | `crates/fuc/src/ssa.rs` | 96-103 | `insert_phi_nodes` and `rename_variables` are empty stubs. | Implement dominance frontier computation and Cytron's algorithm (Phase 2 work — not blocking compilation). |

**Parallelization:** All 4 fixes are independent; can be done in parallel by 2-4 subagents.

**Verification:** Run `cargo test -p fuc` — all 39 existing tests must still pass.

---

## PHASE 1: Make the Compiler Produce Executables (Effort: Large — Critical Path)

**Goal:** End-to-end: `fuc input.fu -o output.exe` produces a runnable native executable.

### 1A. Filesystem Module (`crates/fuc/src/fs.rs`) — Effort: Small

**Current:** All functions return placeholders (`"file_content_placeholder"`, `true`).

**Implement:**
```rust
pub fn read_to_string(path: &str) -> Result<FString, FString> {
    std::fs::read_to_string(path).map_err(|e| e.to_string())
}
pub fn write_string(path: &str, content: &str) -> Result<(), FString> {
    std::fs::write(path, content).map_err(|e| e.to_string())
}
pub fn exists(path: &str) -> FBool {
    std::path::Path::new(path).exists()
}
pub fn metadata(path: &str) -> Result<FileMetadata, FString> {
    let meta = std::fs::metadata(path).map_err(|e| e.to_string())?;
    Ok(FileMetadata {
        size: meta.len() as FSize,
        is_dir: meta.is_dir(),
        is_readonly: meta.permissions().readonly(),
    })
}
```

**Also add:** `read_dir`, `create_dir_all`, `remove_file`, `copy`, `rename`.

**Verification:** Write a test .fu file that reads itself and prints contents. Compile and run.

### 1B. Linker (`crates/fuc/src/linker.rs`) — Effort: Medium

**Current:** 7 lines, prints message and returns `Ok(())`.

**Implement:**
```rust
pub fn link_bin(objects: &[String], output: &str) -> Result<()> {
    // Detect platform
    let link_cmd = if cfg!(target_os = "windows") {
        // Use MSVC link.exe or lld-link
        find_linker(&["lld-link", "link.exe"])?
    } else if cfg!(target_os = "macos") {
        find_linker(&["cc", "clang"])?
    } else {
        find_linker(&["cc", "gcc", "clang"])?
    };

    let mut cmd = Command::new(link_cmd);
    
    if cfg!(target_os = "windows") {
        // MSVC-style linking
        cmd.arg(format!("/OUT:{}", output));
        // Link runtime
        let runtime_obj = find_runtime_obj()?;
        cmd.arg(&runtime_obj);
        for obj in objects {
            cmd.arg(obj);
        }
        // Link standard libraries
        cmd.args(["kernel32.lib", "msvcrt.lib", "ws2_32.lib"]);
    } else {
        // Unix-style linking
        cmd.arg("-o").arg(output);
        // Link runtime
        let runtime_obj = find_runtime_obj()?;
        cmd.arg(&runtime_obj);
        for obj in objects {
            cmd.arg(obj);
        }
        // Link standard libraries
        cmd.args(["-lc", "-lpthread"]);
    }
    
    let status = cmd.status()?;
    if !status.success() {
        return Err(anyhow::anyhow!("Linker failed with exit code: {:?}", status.code()));
    }
    Ok(())
}
```

**Key tasks:**
1. Find the linker binary (search PATH, common locations)
2. Locate pre-compiled runtime objects (`runtime/vector_runtime.o`, `runtime/hashmap_runtime.o`, `runtime/hashset_runtime.o`, `runtime/native/libfusionrt.a`)
3. Pass correct platform flags
4. Handle `--link-lib` and `--lib-path` CLI flags from `CompilerArgs`
5. On Windows, compile `runtime/runtime.c` or use pre-built `runtime/native/fusionrt_win.o`

**Verification:** Compile a hello-world .fu file end-to-end with `--emit-bin` and run the resulting executable.

### 1C. Native Backend Without LLVM Feature Gate — Effort: Large

**Current:** `llvm_backend.rs` is behind `#![cfg(feature = "llvm")]`. Without it, `fuc` prints "no native backend available".

**Two options (choose one):**

**Option A (Recommended): Enable LLVM by default.**
1. Remove `#![cfg(feature = "llvm")]` from `llvm_backend.rs`
2. Make `inkwell` a required dependency in `Cargo.toml` (not optional)
3. Fix all LLVM backend gaps (see 1C-LLVM below)

**Option B: Cranelift fallback backend.**
1. Add `cranelift-codegen` + `cranelift-object` + `target-lexicon` as dependencies
2. Implement a `CraneliftBackend` that implements the `Backend` trait
3. Cranelift is simpler than LLVM and faster to compile

**Option C: Direct object emission via object crate.**
1. Use the `object` crate to emit COFF (Windows) / ELF (Linux) / Mach-O (macOS) directly
2. Simplest option but requires manual code generation

**Recommendation:** Option A (LLVM) for full language support. Option C as minimum viable.

### 1C-LLVM. LLVM Backend Gaps — Effort: Medium

Files to modify: `crates/fuc/src/codegen/llvm_backend.rs`

| Gap | Line(s) | Fix |
|-----|---------|-----|
| `resolve_address` only handles Variable/Pointer | 397-424 | Implement `Element` (GEP with index) and `Field` (GEP with struct field offset) |
| Phi nodes have no incoming values | scattered | After compiling all blocks in a function, call `phi.add_incoming(&[&val1, &val2], &[&block1, &block2])` |
| Float modulo not supported | instruction compilation | Emit `builder.build_float_rem()` |
| Logical ops on floats | instruction compilation | Emit appropriate float comparisons |
| Binary op on non-int types | instruction compilation | Add float/bool dispatch in binary op compilation |
| Bool coercion | instruction compilation | Convert bool to i1 or i64 as needed |
| Closure capture packing | MakeClosure | Allocate heap env struct, store captured vars, return (fn_ptr, env_ptr) pair |
| Void as value type | type mapping | Return empty struct `{}` or handle as no-value |

**Verification:** Compile a program with structs, match, closures, floats — all should produce correct native code.

### 1D. WASM Backend Fixes — Effort: Medium

File: `crates/fuc/src/wasm/codegen.rs` (1385 lines)

| Gap | Line(s) | Fix |
|-----|---------|-----|
| String literals emit `I32Const(0)` | ~350-380 | Emit proper string data section offsets with `DataSection` entries |
| For-loop emits body once | 291-304 | Emit proper WASM `loop`/`block`/`br_if` structure |
| MemberAccess returns base | 394-398 | Compute field byte offset, emit `I32Add` with `field_byte_offset(idx)` |
| Match drops scrutinee | 411-420 | Emit `br_table` or nested `if`/`else` for pattern dispatch |
| StructLiteral doesn't allocate | 399-403 | `emit_heap_alloc`, store each field at offset |
| ArrayLiteral doesn't allocate | 404-410 | `emit_heap_alloc`, store each element |
| Closures inlined | 421-424 | Emit separate WASM function + env struct allocation |
| Section sizes emit 0 | finalization | Fill in actual section sizes |

**Verification:** Compile to .wasm, run with `wasmtime output.wasm` or in browser.

### 1E. fuc2 Vortex Check Integration — Effort: Small

File: `crates/fuc2/src/main.rs` line 107-111

**Current:** Always prints "OK (no violations detected)".

**Fix:** Either:
1. Invoke the actual `fuc.exe --vortex` on the preprocessed source, or
2. Import and call `vortex::VortexContext::verify_program()` directly (if fuc2 can link against fuc library)

**Verification:** Create a .fu file with intentional borrow violations — fuc2 should detect them.

### 1F. String Pattern Matching in IR Lowering — Effort: Small

File: `crates/fuc/src/ir_lower.rs` line 643-646

**Current:** `Comment("TODO: string pattern match")` then jumps to next block.

**Fix:**
```rust
"string" => {
    // Call runtime strcmp function
    let cmp_reg = self.next_reg();
    let cmp_val = self.temp_val(cmp_reg, Type::Bool);
    self.emit(Instruction::Call {
        dest: Some(cmp_reg_val),
        func_name: "fusion_strcmp".to_string(),
        args: vec![scrutinee_val.clone(), self.string_val(arm.pattern.str_val.clone())],
    });
    // Check if strcmp returns 0 (equal)
    let eq_reg = self.next_reg();
    let eq_val = self.temp_val(eq_reg, Type::Bool);
    self.emit(Instruction::BinaryOperation {
        dest: eq_val.clone(),
        op: BinaryOp::Eq,
        op1: cmp_val,
        op2: self.int_val(0),
    });
    self.set_terminator(Terminator::ConditionalJump {
        cond: eq_val,
        then_block: arm_body_blocks[i],
        else_block: next_block,
    });
}
```

**Verification:** Compile a match on strings — should produce correct branching.

---

## PHASE 2: Core Optimizer & SSA (Effort: Medium)

**Goal:** Fix the optimizer and SSA so generated code is actually efficient.

### 2A. Optimizer Dead Block Elimination Fix

File: `crates/fuc/src/optimizer_cfg.rs` line 72-74

```rust
// BEFORE (broken):
func.blocks.retain(|_b| reachable.contains(&0));

// AFTER:
let mut new_blocks = Vec::new();
let mut id_map: HashMap<usize, usize> = HashMap::new();
for (old_id, block) in func.blocks.iter().enumerate() {
    if reachable.contains(&old_id) {
        let new_id = new_blocks.len();
        id_map.insert(old_id, new_id);
        new_blocks.push(block.clone());
    }
}
func.blocks = new_blocks;
// Remap block references in terminators and instructions
remap_block_ids(func, &id_map);
```

### 2B. Optimizer: Additional Constant Folding

File: `crates/fuc/src/optimizer.rs`

Add folding for:
- `UnaryNot`: `!true` → `false`, `!false` → `true`
- `Copy`: propagate through single-definition chains
- `Alloca` + immediate `Store` + `Load` → forwarded value

### 2C. SSA Phi Placement (Cytron's Algorithm)

File: `crates/fuc/src/ssa.rs`

Implement:
1. **Dominator tree computation** (Lengauer-Tarjan algorithm)
2. **Dominance frontier calculation** for each block
3. **Phi node insertion** at iterated dominance frontiers
4. **Variable renaming** via pre-order dominator tree traversal

This is algorithmically complex but well-defined. Reference: Cytron et al. 1991, "Efficiently Computing Static Single Assignment Form."

**Verification:** Run optimizer on test programs, verify dead blocks are eliminated. Verify SSA conversion produces valid phi nodes.

---

## PHASE 3: Standard Library Implementation (Effort: Medium-Large)

**Goal:** Make stdlib functions actually work at runtime.

### 3A. C Runtime Implementation (Critical)

The Fusion stdlib uses `extern fn` declarations that map to C functions in `runtime/runtime.c` and `runtime/native/fusionrt.c`. These C files are **already substantially implemented** (789 and 733 lines respectively). Key functions that exist:
- I/O: `fusion_println`, `fusion_print`, `fusion_print_int`, `fusion_read_line`, `fusion_string_to_int`, `fusion_int_to_string`
- Strings: `fusion_strlen`, `fusion_strcmp`, `fusion_strcpy`, `fusion_str_repeat`, `fusion_str_trim`, `fusion_str_substring`, `fusion_str_starts_with`, `fusion_str_ends_with`, `fusion_str_replace`
- Filesystem: `fusion_fs_read_to_string`, `fusion_fs_write_string`, `fusion_fs_exists`, `fusion_fs_read_dir`, `fusion_fs_create_dir`, `fusion_fs_remove_file`, `fusion_fs_copy`, `fusion_fs_rename`, `fusion_fs_metadata`
- Memory: `fusion_malloc`, `fusion_free`, `fusion_realloc`, `fusion_memset`, `fusion_memcpy`
- Math: `fusion_abs`, `fusion_min`, `fusion_max`, `fusion_sqrt`, `fusion_pow`, `fusion_log`, `fusion_sin`, `fusion_cos`, `fusion_tan`, `fusion_atan2`, `fusion_floor`, `fusion_ceil`, `fusion_round`
- Network: `fusion_net_connect`, `fusion_net_send`, `fusion_net_recv`, `fusion_net_close`, `fusion_net_listen`, `fusion_net_accept`
- Random: `fusion_rand_int`, `fusion_rand_float`
- Time: `fusion_time_now`, `fusion_sleep_ms`
- Process: `fusion_exec`, `fusion_exit`, `fusion_getenv`, `fusion_setenv`
- Vector: `fusion_vi_*`, `fusion_vb_*`, `fusion_vs_*`
- HashMap: `fusion_hmii_*`, `fusion_hmis_*`, `fusion_hmss_*`
- Hashset: `fusion_hs_*`

**What needs fixing:** Ensure linker能找到这些符号 (Windows .lib / Unix .o). Pre-build step or integrate into build.

### 3B. Fusion Stdlib Files (Already Mostly Implemented)

The `stdlib/*.fu` files are **thin wrappers** around `extern fn` C runtime functions. They are already complete for:
- `io.fu` — println, print_str, print_int, read_line, read_int ✓
- `string.fu` — ManagedString, str_len, str_equals, str_contains_char, str_char_at ✓
- `vector.fu` — VectorInt, VectorBool, VectorString (handle-based, all ops) ✓
- `hashmap.fu` — HashMapIntInt, HashMapIntString, HashMapStringString ✓
- `hashset.fu`, `fs.fu`, `process.fu`, `log.fu`, `json.fu` — all wrapped ✓

**What needs implementing:**
1. Build step that compiles `runtime/runtime.c` → `runtime.o` for the target platform
2. Build step that compiles `runtime/vector_runtime.c`, `runtime/hashmap_runtime.c`, `runtime/hashset_runtime.c` → `.o` files
3. Link these into the final executable

### 3C. Registry Crate Stubs (Deferred)

The 250+ registry crates (`registry/crates/`) are mostly aspirational. Focus only on the ones actually referenced by the compiler:
- `fusion-core` — type system core (already referenced in Cargo.toml)
- `fusion_quantum` — quantum SDK
- `fusion_runtime_core` — runtime core
- `fusion_finance` — finance module
- `fusion_runtime_scheduler` — task scheduler

These are **NOT needed** for basic Fusion compilation. Defer to Phase 6.

---

## PHASE 4: Semantic Analysis Hardening (Effort: Medium)

**Goal:** Make sema catch real errors and produce correct TypedAST.

### 4A. Sema.rs Enhancements

File: `crates/fuc/src/sema.rs` (446 lines)

Current sema does:
- Two-pass analysis (collect signatures, then check bodies)
- Type inference for basic expressions
- Function call type checking
- Binary operation type checking

**Needs:**
1. **Enum type support** — currently parser skips `enum` via `skip_aspirational_item`
2. **Impl block support** — currently skipped
3. **Trait support** — currently skipped
4. **Const/static support** — currently skipped
5. **Module system** — currently no multi-file support in sema
6. **Generic type parameters** — declared in AST but not checked
7. **Better error messages** with source spans
8. **Float type checking** — ensure float ops only on floats, int ops only on ints

### 4B. Parser: Unskip Aspirational Constructs

File: `crates/fuc/src/parser.rs`

Currently, `enum`, `impl`, `trait`, `const`, `static`, `type` are all consumed by `skip_aspirational_item()`. Implement real parsing for:

1. **Enums** — parse variants (unit, tuple, struct variants) into `EnumDefinition`
2. **Impl blocks** — parse method definitions into `ImplDefinition`
3. **Trait declarations** — parse method signatures into `TraitDefinition`
4. **Const/static** — parse initializers into `ConstDefinition`/`StaticDefinition`

### 4C. For-Loop Bounds Check

File: `crates/fuc/src/ir_lower.rs` line 309

Add bounds check: compare index against array length, emit `ConditionalJump` to exit block if index >= length.

---

## PHASE 5: PQC (Post-Quantum Cryptography) Integration (Effort: Large)

**Goal:** Implement 50/50 classical + PQC enforcement.

### 5A. PQC Module in Compiler

File: `crates/fuc/src/pqc.rs` (89 lines)

**Current:** Has `SecureTcpStream` and `SecureTcpListener` using `ring::agreement::X25519` (classical only) with a "Kyber768 Mock Frame" (random bytes, not real Kyber).

**Implement:**
1. Replace mock Kyber with real `pqcrypto-mlkem` (already in Cargo.toml)
2. Add ML-DSA (Dilithium) signatures via `pqcrypto-mldsa`
3. Implement hybrid key exchange: X25519 + ML-KEM-768
4. Implement hybrid signatures: Ed25519 + ML-DSA-65
5. Add 50/50 enforcement: every cryptographic operation must use both classical AND PQC

### 5B. PQC in Fusion Language

File: `src/compiler/pqc.fu` (if exists) or create `stdlib/pqc.fu`

```fusion
// Post-Quantum Cryptography enforcement
// Every key exchange, signature, and encryption must be hybrid (50/50)

extern fn pqc_hybrid_kem_keygen() -> (PublicKey, PrivateKey);
extern fn pqc_hybrid_kem_encaps(pk: PublicKey) -> (Ciphertext, SharedSecret);
extern fn pqc_hybrid_kem_decaps(sk: PrivateKey, ct: Ciphertext) -> SharedSecret;
extern fn pqc_hybrid_sign_keygen() -> (VerifyKey, SignKey);
extern fn pqc_hybrid_sign(sk: SignKey, msg: string) -> Signature;
extern fn pqc_hybrid_verify(vk: VerifyKey, msg: string, sig: Signature) -> bool;
```

### 5C. Security Policy Engine

File: `src/security/` directory

Implement compile-time enforcement:
1. Lint pass that flags uses of classical-only crypto
2. Require all `net::connect` to use `SecureTcpStream` (hybrid)
3. Require all file I/O with `.enc` extension to use PQC encryption
4. Compile-time warning/error for non-hybrid crypto usage

---

## PHASE 6: Advanced Features (Effort: Very Large — Deferred)

### 6A. LSP Implementation
File: `crates/fuc/src/lsp.rs` (127 lines)

Implement real LSP: JSON-RPC over stdin/stdout, document sync, diagnostics, completion, go-to-definition.

### 6B. Package Manager
File: `crates/fuc/src/forge_pkg.rs`

Parse `Fusion.toml`, resolve dependencies, fetch from registry.

### 6C. Runtime Components
- FiberScheduler, MemoryManager, JIT compiler — all empty structs
- Flux Resolve GPU SAT solver — hardcoded results
- Cortex ML scheduler model loading — stub

### 6D. Formal Verification
All proofs in `src/security/` are "Admitted" — implement actual proofs.

### 6E. Self-Hosting Verification
Run Ouroboros 3-stage bootstrap: fuc.exe → compiles fuc source → fuc_stage1 → compiles fuc source → fuc_stage2 → verify stage1 == stage2 (bit-identical).

---

## Implementation Order & Parallelization

```
Phase 0 (bug fixes)          ← 2-4 parallel subagents, 1 day
    ↓
Phase 1A (fs.rs)             ← 1 subagent, 1 day
Phase 1B (linker.rs)         ← 1 subagent, 2 days (parallel with 1A)
Phase 1C (LLVM backend)      ← 2 parallel subagents, 3 days
Phase 1D (WASM backend)      ← 1 subagent, 2 days (parallel with 1C)
Phase 1E (fuc2 vortex)       ← 1 subagent, 0.5 day
Phase 1F (string match)      ← 1 subagent, 0.5 day
    ↓
Phase 2 (optimizer + SSA)    ← 1-2 subagents, 2 days
    ↓
Phase 3 (stdlib runtime)     ← 2 parallel subagents, 2 days
    ↓
Phase 4 (sema hardening)     ← 2 parallel subagents, 3 days
    ↓
Phase 5 (PQC)                ← 2 parallel subagents, 5 days
    ↓
Phase 6 (advanced)           ← 3+ parallel subagents, 10+ days
```

**Total estimated effort:** ~30-40 agent-days for Phases 0-5 (making the compiler functional).
Phase 6 is production-hardening and can be done incrementally.

---

## Success Criteria

After Phases 0-3:
- [ ] `fuc hello.fu -o hello.exe --emit-bin` produces a working executable on Windows
- [ ] The executable prints "Hello, World!" when run
- [ ] Programs using structs, match, while, for, if/else compile and run correctly
- [ ] String comparison and pattern matching work
- [ ] `fuc2 --vortex input.fu` catches borrow violations
- [ ] All 39 existing tests still pass
- [ ] New tests cover: linker, fs, string match, struct field access, closures

After Phase 5:
- [ ] Hybrid PQC key exchange works end-to-end
- [ ] Hybrid signatures verify correctly
- [ ] Compile-time enforcement rejects classical-only crypto
