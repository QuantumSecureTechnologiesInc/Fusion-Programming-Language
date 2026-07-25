# Fusion Pre-commit Hook (PowerShell)
# Enforces Fusion Flux Engine policy and scope boundaries before commits

$ErrorActionPreference = "Stop"

# Get staged files
$STAGED_FILES = git diff --cached --name-only
$VIOLATIONS = 0
$WARNINGS = 0
$REVIEW_REQUIRED = 0

# Path to the --no-verify bypass audit engine
$BypassAudit = Join-Path $PSScriptRoot "..\scripts\bypass_audit.ps1"

# Writes a freshness marker "<head> <staged-tree>" recording the current HEAD
# (the parent-to-be) and the staged tree that will be committed, so the
# post-commit hook can tell whether pre-commit actually ran for the new commit.
# A missing/stale marker at post-commit time means --no-verify was used. Binding
# the staged tree defeats a stale marker left by an aborted commit that is later
# re-attempted with --no-verify and different content.
function Write-PrecommitMarker {
    try {
        $gitDir = (git rev-parse --git-dir 2>$null)
        if ([string]::IsNullOrWhiteSpace($gitDir)) { return }
        $head = (git rev-parse --verify --quiet HEAD 2>$null)
        if ([string]::IsNullOrWhiteSpace($head)) { $head = "ROOT" }
        $tree = (git write-tree 2>$null)
        if ([string]::IsNullOrWhiteSpace($tree)) { $tree = "NOTREE" }
        $markerPath = Join-Path $gitDir.Trim() "fusion_precommit_marker"
        Set-Content -Path $markerPath -Value ("{0} {1}" -f $head.Trim(), $tree.Trim()) -NoNewline -Encoding UTF8
    } catch { }
}

#================================================================================
# SECTION 0: ESCALATION GATE (block while a high-risk bypass awaits approval)
#================================================================================
if (Test-Path $BypassAudit) {
    $prevEAP = $ErrorActionPreference
    $ErrorActionPreference = "Continue"
    & powershell -ExecutionPolicy Bypass -NoProfile -File $BypassAudit -Mode Gate
    $gateExit = $LASTEXITCODE
    $ErrorActionPreference = $prevEAP
    if ($gateExit -ne 0) {
        exit 1
    }
}

#================================================================================
# SECTION 1: SCOPE BOUNDARY ENFORCEMENT (Hard Blocks)
#================================================================================
Write-Host "=== Scope Boundary Validation ===" -ForegroundColor Cyan

# Block commits to bin/ directory (prebuilt binaries)
foreach ($file in $STAGED_FILES) {
    if ($file -match '^bin/') {
        Write-Host "[BLOCKED] Changes to bin/ directory are prohibited" -ForegroundColor Red
        Write-Host "   File: $file"
        Write-Host "   Reason: bin/ contains prebuilt binaries. Regenerate via scripts/bootstrap_native.ps1"
        Write-Host "   Action: Remove from staging with 'git reset HEAD <file>'"
        $VIOLATIONS++
    }
}

# Block commits to target/ directories (build artifacts)
foreach ($file in $STAGED_FILES) {
    if ($file -match '^target/' -or $file -match '^target_') {
        Write-Host "[BLOCKED] Changes to target directories are prohibited" -ForegroundColor Red
        Write-Host "   File: $file"
        Write-Host "   Reason: target/ contains build artifacts that should not be committed"
        Write-Host "   Action: Add to .gitignore and remove from staging"
        $VIOLATIONS++
    }
}

# Block commits of compiled build artifacts at the repository root
# (only additions/modifications -- staged deletions must be allowed so the
#  cleanup script's removals can be committed)
$STAGED_ADDED = git diff --cached --name-only --diff-filter=d
foreach ($file in $STAGED_ADDED) {
    # Only consider root-level files (no directory separator in the path)
    if ($file -notmatch '/' -and $file -match '\.(exe|obj|o|ll|bc|map|pdb)$') {
        Write-Host "[BLOCKED] Compiled build artifact staged at repository root" -ForegroundColor Red
        Write-Host "   File: $file"
        Write-Host "   Reason: Root-level build output (.exe/.obj/.o/.stage.o/.ll/.bc/.map/.pdb) must not be committed"
        Write-Host "   Action: Run 'scripts/clean_root_artifacts.ps1' then 'git reset HEAD $file'"
        $VIOLATIONS++
    }
}

# Block commits to Cargo.lock (unless intentional)
foreach ($file in $STAGED_FILES) {
    if ($file -eq "Cargo.lock") {
        Write-Host "[WARNING] Changes to Cargo.lock detected" -ForegroundColor Yellow
        Write-Host "   File: $file"
        Write-Host "   Reason: Cargo.lock should only be updated via cargo commands, never hand-edited"
        Write-Host "   Action: If intentional, use 'git commit --no-verify' with justification"
        $WARNINGS++
    }
}

#================================================================================
# SECTION 2: HIGH-RISK AREA REVIEW TRIGGERS
#================================================================================
Write-Host ""
Write-Host "=== High-Risk Area Review ===" -ForegroundColor Cyan

# Define high-risk paths
$COMPILER_CRATE = "crates/fuc/src/"
$RUNTIME_DIR = "runtime/"
$STDLIB_SECURITY_FILES = @("security.fu", "hybrid_crypto.fu", "ed25519.fu", "x25519.fu", "weave_kem.fu", "weave_sig.fu", "sha3.fu")
$CRITICAL_COMPILER_FILES = @("parser.rs", "sema.rs", "codegen/", "ir_lower.rs", "main.rs")

# Check for compiler crate changes
foreach ($file in $STAGED_FILES) {
    $escapedCrate = [regex]::Escape($COMPILER_CRATE)
    if ($file -match "^$escapedCrate") {
        $REVIEW_REQUIRED++
        Write-Host "[REVIEW REQUIRED] Compiler crate modification" -ForegroundColor Yellow
        Write-Host "   File: $file"
        Write-Host "   Risk: Compiler changes can affect all Fusion programs"
        Write-Host "   Required: Run 'cargo build -p fuc' and 'scripts/run_native_regression.ps1'"
        
        # Flag critical compiler files
        foreach ($critical in $CRITICAL_COMPILER_FILES) {
            if ($file -match [regex]::Escape($critical)) {
                Write-Host "   [CRITICAL] This is a core compiler file - requires thorough review" -ForegroundColor Red
            }
        }
    }
}

# Check for runtime changes
foreach ($file in $STAGED_FILES) {
    $escapedRuntime = [regex]::Escape($RUNTIME_DIR)
    if ($file -match "^$escapedRuntime") {
        $REVIEW_REQUIRED++
        Write-Host "[REVIEW REQUIRED] Runtime modification" -ForegroundColor Yellow
        Write-Host "   File: $file"
        Write-Host "   Risk: Runtime changes affect program execution"
        Write-Host "   Required: Run 'cmake -B cmake_build -DCMAKE_BUILD_TYPE=Release && cmake --build cmake_build'"
    }
}

# Check for stdlib security-sensitive file changes
foreach ($file in $STAGED_FILES) {
    foreach ($sec_file in $STDLIB_SECURITY_FILES) {
        if ($file -match [regex]::Escape($sec_file)) {
            $REVIEW_REQUIRED++
            Write-Host "[REVIEW REQUIRED] Security-sensitive stdlib modification" -ForegroundColor Yellow
            Write-Host "   File: $file"
            Write-Host "   Risk: Security files require careful review for vulnerabilities"
            Write-Host "   Required: Run security gate and verify no unsafe code introduced"
        }
    }
}

#================================================================================
# SECTION 3: BUILD POLICY COMPLIANCE
#================================================================================
Write-Host ""
Write-Host "=== Build Policy Compliance ===" -ForegroundColor Cyan

# Check for prohibited cargo commands in staged files
foreach ($file in $STAGED_FILES) {
    if (Test-Path $file) {
        $ext = [System.IO.Path]::GetExtension($file)
        if ($ext -match '\.(sh|ps1|bash|yml|yaml)$' -or $file -match 'Makefile$') {
            $content = Get-Content $file -Raw -ErrorAction SilentlyContinue
            if ($content -and $content -match '(?m)(^|[^#])(cargo (build|test|run|check))') {
                # Exempt runtime directory, CI workflow files, and pre-commit hooks themselves
                if (-not $file.StartsWith('runtime/') -and -not $file.StartsWith('.github/workflows/') -and -not $file.StartsWith('.githooks/')) {
                    Write-Host "[VIOLATION] Policy violation in: $file" -ForegroundColor Red
                    Write-Host "   Contains prohibited 'cargo' command"
                    Write-Host "   Use 'fusion build/test/run' instead"
                    $VIOLATIONS++
                }
            }
        }
    }
}

# Check BUILD_POLICY.md is not being removed
$deletedFiles = git diff --cached --name-status
if ($deletedFiles -match '^D.*BUILD_POLICY\.md') {
    Write-Host "[BLOCKED] Cannot delete BUILD_POLICY.md" -ForegroundColor Red
    Write-Host "   This file is required for policy enforcement"
    $VIOLATIONS++
}

# Check .fusion/build-policy.json is not being removed
if ($deletedFiles -match '^D.*\.fusion/build-policy\.json') {
    Write-Host "[BLOCKED] Cannot delete .fusion/build-policy.json" -ForegroundColor Red
    Write-Host "   This file is required for policy enforcement"
    $VIOLATIONS++
}

#================================================================================
# SECTION 4: VALIDATION SEQUENCE (AGENTS.md steps 1-4)
#================================================================================
Write-Host ""
Write-Host "=== Validation Sequence (AGENTS.md steps 1-4) ===" -ForegroundColor Cyan

# Detect staged Rust/TOML files
$RUST_FILES = @()
foreach ($file in $STAGED_FILES) {
    if ($file -match '\.(rs|toml)$') {
        $RUST_FILES += $file
    }
}

if ($RUST_FILES.Count -eq 0) {
    Write-Host "No Rust files staged -- skipping validation sequence" -ForegroundColor Green
} else {
    Write-Host "Staged Rust/TOML files:$($RUST_FILES -join ', ')"
    Write-Host ""

    $VALIDATION_FAILED = $false

    # Step 1: Format Check
    Write-Host "Step 1/4: cargo fmt --all -- --check" -ForegroundColor Yellow
    $prevEAP = $ErrorActionPreference
    $ErrorActionPreference = "Continue"
    $fmtOutput = cargo fmt --all -- --check 2>&1
    $fmtExit = $LASTEXITCODE
    $ErrorActionPreference = $prevEAP
    if ($fmtExit -eq 0) {
        Write-Host "Format check passed" -ForegroundColor Green
    } else {
        Write-Host "Format check FAILED" -ForegroundColor Red
        Write-Host "Run 'cargo fmt --all' to fix formatting, then re-stage."
        $VALIDATION_FAILED = $true
    }

    # Step 2: Clippy Lint
    Write-Host ""
    Write-Host "Step 2/4: cargo clippy --all-targets --all-features -- -D warnings" -ForegroundColor Yellow
    $prevEAP = $ErrorActionPreference
    $ErrorActionPreference = "Continue"
    $clippyOutput = cargo clippy --all-targets --all-features -- -D warnings 2>&1
    $clippyExit = $LASTEXITCODE
    $ErrorActionPreference = $prevEAP
    if ($clippyExit -eq 0) {
        Write-Host "Clippy check passed" -ForegroundColor Green
    } else {
        Write-Host "Clippy check FAILED" -ForegroundColor Red
        Write-Host "Fix the warnings above before committing."
        $VALIDATION_FAILED = $true
    }

    # Step 3: Build
    Write-Host ""
    Write-Host "Step 3/4: cargo build --verbose --all-features" -ForegroundColor Yellow
    $prevEAP = $ErrorActionPreference
    $ErrorActionPreference = "Continue"
    $buildOutput = cargo build --verbose --all-features 2>&1
    $buildExit = $LASTEXITCODE
    $ErrorActionPreference = $prevEAP
    if ($buildExit -eq 0) {
        Write-Host "Build passed" -ForegroundColor Green
    } else {
        Write-Host "Build FAILED" -ForegroundColor Red
        Write-Host "Fix build errors above before committing."
        $VALIDATION_FAILED = $true
    }

    # Step 4: Test
    Write-Host ""
    Write-Host "Step 4/4: cargo test --verbose --all-features" -ForegroundColor Yellow
    $prevEAP = $ErrorActionPreference
    $ErrorActionPreference = "Continue"
    $testOutput = cargo test --verbose --all-features 2>&1
    $testExit = $LASTEXITCODE
    $ErrorActionPreference = $prevEAP
    if ($testExit -eq 0) {
        Write-Host "Tests passed" -ForegroundColor Green
    } else {
        Write-Host "Tests FAILED" -ForegroundColor Red
        Write-Host "Fix failing tests above before committing."
        $VALIDATION_FAILED = $true
    }

    # Report validation results
    if ($VALIDATION_FAILED) {
        Write-Host ""
        Write-Host "=== Validation sequence FAILED ===" -ForegroundColor Red
        Write-Host "Fix the errors above before committing." -ForegroundColor Red
        $VIOLATIONS++
    } else {
        Write-Host ""
        Write-Host "All validation checks passed" -ForegroundColor Green
    }
}

#================================================================================
# SECTION 5: SECURITY GATE (secrets, dependency, SBOM)
#================================================================================
Write-Host ""
Write-Host "=== Security Gate ===" -ForegroundColor Cyan

$securityScript = Join-Path $PSScriptRoot "..\scripts\security_gate.ps1"
if (Test-Path $securityScript) {
    $prevEAP = $ErrorActionPreference
    $ErrorActionPreference = "Continue"
    $securityOutput = powershell -ExecutionPolicy Bypass -NoProfile -File $securityScript 2>&1
    $securityExit = $LASTEXITCODE
    $ErrorActionPreference = $prevEAP

    if ($securityExit -ne 0) {
        Write-Host "[BLOCKED] Security gate FAILED" -ForegroundColor Red
        Write-Host "   Secrets, dependency issues, or policy violations detected."
        Write-Host "   Fix the issues above before committing."
        Write-Host "   Full output:"
        foreach ($line in $securityOutput) {
            Write-Host "     $line"
        }
        $VIOLATIONS++
    } else {
        Write-Host "Security gate passed" -ForegroundColor Green
    }
} else {
    Write-Host "[WARNING] Security script not found: $securityScript" -ForegroundColor Yellow
    Write-Host "   Skipping security gate (script missing)"
    $WARNINGS++
}

#================================================================================
# SECTION 6: NATIVEREGRESSION GATE (crates/fuc/src/)
#================================================================================
Write-Host ""
Write-Host "=== Compiler Regression Gate ===" -ForegroundColor Cyan

# Detect if any staged files are inside the compiler crate source
$COMPILER_SRC_PREFIX = "crates/fuc/src/"
$hasCompilerChanges = $false
foreach ($file in $STAGED_FILES) {
    $escapedPrefix = [regex]::Escape($COMPILER_SRC_PREFIX)
    if ($file -match "^$escapedPrefix") {
        $hasCompilerChanges = $true
        break
    }
}

if ($hasCompilerChanges) {
    Write-Host "Compiler crate source changes detected -- running native regression" -ForegroundColor Yellow

    $regressionScript = Join-Path $PSScriptRoot "..\scripts\run_native_regression.ps1"
    if (Test-Path $regressionScript) {
        $prevEAP = $ErrorActionPreference
        $ErrorActionPreference = "Continue"
        $regressionOutput = powershell -ExecutionPolicy Bypass -NoProfile -File $regressionScript 2>&1
        $regressionExit = $LASTEXITCODE
        $ErrorActionPreference = $prevEAP

        if ($regressionExit -ne 0) {
            Write-Host "[BLOCKED] Native regression FAILED" -ForegroundColor Red
            Write-Host "   The compiler regression gate detected failures."
            Write-Host "   Fix regressions before committing."
            Write-Host "   Full output:"
            foreach ($line in $regressionOutput) {
                Write-Host "     $line"
            }
            Write-Host ""
            Write-Host "   Regression log: artifacts/native-regression/run_native_regression.log" -ForegroundColor DarkGray
            $VIOLATIONS++
        } else {
            Write-Host "Native regression passed" -ForegroundColor Green
        }
    } else {
        Write-Host "[WARNING] Regression script not found: $regressionScript" -ForegroundColor Yellow
        Write-Host "   Skipping regression gate (script missing)"
        $WARNINGS++
    }
} else {
    Write-Host "No compiler crate source changes -- skipping regression gate" -ForegroundColor Green
}

#================================================================================
# SECTION 7: REPORT RESULTS
#================================================================================
Write-Host ""
Write-Host "========================================" -ForegroundColor Cyan

if ($VIOLATIONS -gt 0) {
    Write-Host "[COMMIT BLOCKED] $VIOLATIONS violation(s) detected" -ForegroundColor Red
    Write-Host ""
    Write-Host "Please fix the violations above before committing."
    Write-Host "See BUILD_POLICY.md for details."
    Write-Host ""
    Write-Host "To bypass (emergency only):"
    Write-Host "  git commit --no-verify"
    Write-Host "  NOTE: --no-verify on crates/fuc/, runtime/, or stdlib/ is audited"
    Write-Host "        to .fusion/audit/no-verify-bypass.log and requires approval."
    Write-Host ""
    exit 1
}

if ($WARNINGS -gt 0 -or $REVIEW_REQUIRED -gt 0) {
    Write-PrecommitMarker
    Write-Host "[COMMIT ALLOWED WITH WARNINGS] $WARNINGS warning(s), $REVIEW_REQUIRED review(s) required" -ForegroundColor Yellow
    Write-Host ""
    Write-Host "The commit will proceed, but please address the following:"
    if ($REVIEW_REQUIRED -gt 0) {
        Write-Host "  - Complete required reviews and validation steps"
        Write-Host "  - Run appropriate test suites"
        Write-Host "  - Document any risk mitigation"
    }
    if ($WARNINGS -gt 0) {
        Write-Host "  - Review warnings above"
    }
    Write-Host ""
    exit 0
}

Write-PrecommitMarker
Write-Host "[ALL CHECKS PASSED] Commit allowed" -ForegroundColor Green
exit 0
