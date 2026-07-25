# Recovery Route (Failed Changes)

How to restore the known-good state when a change breaks the build, tests, or
regression suite. Linked from the root [AGENTS.md](../../AGENTS.md).

**Owner:** `build-infrastructure` (see `.qoder/project-owners.json`)

## Manual Rollback (step-by-step)

Run these commands in order to restore the project:

1. **Restore working tree:**
   ```powershell
   git reset --hard HEAD
   git clean -fd
   ```

2. **Rebuild compiler:**
   ```powershell
   cargo build -p fuc --release --features llvm,wasm
   ```
   > **Note:** The `wasm` feature is required for the `fuc` binary, and `llvm` is required for native codegen used by the regression suite. This requires LLVM and `libxml2s.lib` to be available in the environment.

3. **Validate recovery:**
   ```powershell
   powershell -ExecutionPolicy Bypass -File scripts\run_native_regression.ps1
   ```

## Recovery Route Checklist

| Step | Command | Confirms |
|------|---------|----------|
| Rollback | `git reset --hard HEAD` | Working tree matches last commit |
| Rebuild | `cargo build -p fuc --release --features llvm,wasm` | Compiler compiles cleanly |
| Validate | `scripts\run_native_regression.ps1` | All regression fixtures pass |

> Recovery is complete when the regression gate reports **Regression passed**.
> If regression fails after rollback, the issue is in the committed state —
> escalate to the subsystem owner for the affected component.

## Verification Results

**Date:** 2026-07-24
**Scenario:** Manual rollback from dirty working tree.

| Step | Result | Notes |
|------|--------|-------|
| Git Reset | PASS | Working tree restored to HEAD `d909c0af` |
| Git Clean | PASS | Untracked files removed (some `artifacts/` dirs had permission issues) |
| Build | PASS (wasm only) / FAIL (llvm) | `--features llvm` fails due to missing `libxml2s.lib` in current environment |
| Regression | FAIL | 6/27 passed. Failures due to missing native backend and parser limitations (e.g., array literals) |

**Conclusion:** The recovery route successfully restores the source tree to the committed state. However, full regression validation requires an environment with LLVM libraries and may expose committed bugs in the parser/compiler.
