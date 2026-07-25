# run_all_tests_v2.ps1 - Fusion Compiler v2 Comprehensive Test Suite
# Tests fuc.exe (bootstrap), fuc2 (extended driver), and the full pipeline

$ROOT = Split-Path -Parent $MyInvocation.MyCommand.Path
$FUC = Join-Path $ROOT "bin\fuc.exe"
$FUC2 = Join-Path $ROOT "crates\fuc2\target\release\fuc2.exe"
$FIXTURES = Join-Path $ROOT "crates\fuc\tests\fixtures"
$INVALID = Join-Path $ROOT "crates\fuc\tests\fixtures\invalid"

Write-Host ""
Write-Host "=============================================" -ForegroundColor Cyan
Write-Host "  FUSION COMPILER v2 — FULL TEST SUITE" -ForegroundColor Cyan
Write-Host "=============================================" -ForegroundColor Cyan
Write-Host ""

$total = 0
$pass = 0

function Test-Case {
    param($name, $block)
    $script:total++
    try {
        $result = & $block
        if ($result) {
            $script:pass++
            Write-Host "  [PASS] $name" -ForegroundColor Green
        } else {
            Write-Host "  [FAIL] $name" -ForegroundColor Red
        }
    } catch {
        Write-Host "  [FAIL] $name — $_" -ForegroundColor Red
    }
}

#===========================================================
# SECTION 1: Bootstrap fuc.exe — Parse Only (valid fixtures)
#===========================================================
Write-Host "--- 1. Bootstrap: parse-only valid fixtures ---" -ForegroundColor Yellow
$valid_fixtures = @("hello_world", "arrays", "complex_arrays", "final_struct_test",
                     "safety_and_repeat", "sema_struct", "string_eq")

foreach ($f in $valid_fixtures) {
    $src = Join-Path $FIXTURES "$f.fu"
    Test-Case "parse $f" {
        $out = & $FUC $src --parse-only 2>&1
        $LASTEXITCODE -eq 0
    }
}

#===========================================================
# SECTION 2: Bootstrap fuc.exe — Reject invalid fixtures
#===========================================================
Write-Host ""
Write-Host "--- 2. Bootstrap: reject invalid fixtures ---" -ForegroundColor Yellow

Test-Case "reject parse_error.fu" {
    $out = & $FUC (Join-Path $INVALID "parse_error.fu") --parse-only 2>&1
    $LASTEXITCODE -ne 0
}

Test-Case "reject sema_error.fu (parse OK)" {
    $out = & $FUC (Join-Path $INVALID "sema_error.fu") --parse-only 2>&1
    $LASTEXITCODE -eq 0  # syntactically valid
}

Test-Case "reject parser_robustness.fu (parse tolerant)" {
    $out = & $FUC (Join-Path $INVALID "parser_robustness.fu") --parse-only 2>&1
    $LASTEXITCODE -eq 0  # parser is tolerant by design
}

#===========================================================
# SECTION 3: Bootstrap fuc.exe — Sema Only (valid fixtures)
#===========================================================
Write-Host ""
Write-Host "--- 3. Bootstrap: sema-only valid fixtures ---" -ForegroundColor Yellow

foreach ($f in $valid_fixtures) {
    $src = Join-Path $FIXTURES "$f.fu"
    Test-Case "sema $f" {
        $out = & $FUC $src --sema-only 2>&1
        $LASTEXITCODE -eq 0
    }
}

#===========================================================
# SECTION 4: Bootstrap fuc.exe — Reject at sema
#===========================================================
Write-Host ""
Write-Host "--- 4. Bootstrap: sema rejects invalid ---" -ForegroundColor Yellow

Test-Case "sema rejects sema_error.fu" {
    $out = & $FUC (Join-Path $INVALID "sema_error.fu") --sema-only 2>&1
    $LASTEXITCODE -ne 0
}

Test-Case "sema rejects parser_robustness.fu" {
    $out = & $FUC (Join-Path $INVALID "parser_robustness.fu") --sema-only 2>&1
    $LASTEXITCODE -ne 0
}

#===========================================================
# SECTION 5: Bootstrap fuc.exe — Codegen + Link + Run
#===========================================================
Write-Host ""
Write-Host "--- 5. Bootstrap: codegen + link + run ---" -ForegroundColor Yellow

foreach ($f in $valid_fixtures) {
    $src = Join-Path $FIXTURES "$f.fu"
    $exe = Join-Path $ROOT "test_${f}_v2.exe"
    $outFile = Join-Path $FIXTURES "$f.out"
    $exitFile = Join-Path $FIXTURES "$f.exit"

    Test-Case "compile+run $f" {
        $buildOut = & $FUC $src --emit-bin -o $exe 2>&1
        if ($LASTEXITCODE -ne 0) { return $false }
        if (-not (Test-Path $exe)) { return $false }

        $actualOut = & $exe 2>&1
        $actualExit = $LASTEXITCODE

        # Check exit code if .exit file exists
        if (Test-Path $exitFile) {
            $expectedExit = [int](Get-Content $exitFile -Raw).Trim()
            if ($actualExit -ne $expectedExit) { return $false }
        }

        # Check stdout if .out file exists
        if (Test-Path $outFile) {
            $expectedOut = (Get-Content $outFile -Raw).TrimEnd()
            $actualStr = ($actualOut -join "`n").TrimEnd()
            if ($actualStr -ne $expectedOut) { return $false }
        }

        return $true
    }
}

#===========================================================
# SECTION 6: CLI Flags
#===========================================================
Write-Host ""
Write-Host "--- 6. CLI flags ---" -ForegroundColor Yellow

Test-Case "--lib flag" {
    $out = & $FUC (Join-Path $ROOT "test_lib.fu") --lib 2>&1
    $LASTEXITCODE -eq 0
}

Test-Case "no --lib rejects (no main)" {
    $out = & $FUC (Join-Path $ROOT "test_lib.fu") --emit-bin -o test_nolib.exe 2>&1
    $LASTEXITCODE -ne 0
}

Test-Case "--opt-level 3" {
    $out = & $FUC (Join-Path $FIXTURES "hello_world.fu") --emit-bin -o test_opt_v2.exe --opt-level 3 2>&1
    $built = $LASTEXITCODE -eq 0
    $ran = $false
    if ($built -and (Test-Path "test_opt_v2.exe")) {
        $runOut = & ".\test_opt_v2.exe" 2>&1
        $ran = $LASTEXITCODE -eq 0
    }
    $built -and $ran
}

#===========================================================
# SECTION 7: fuc2 Extended Driver
#===========================================================
Write-Host ""
Write-Host "--- 7. fuc2 extended driver ---" -ForegroundColor Yellow

Test-Case "fuc2 builds (binary exists)" {
    Test-Path $FUC2
}

Test-Case "fuc2 compiles hello_world" {
    $out = & $FUC2 (Join-Path $FIXTURES "hello_world.fu") -o test_fuc2_hello.exe 2>&1
    $built = $LASTEXITCODE -eq 0
    $ran = $false
    if ($built -and (Test-Path "test_fuc2_hello.exe")) {
        $runOut = & ".\test_fuc2_hello.exe" 2>&1
        $ran = ($runOut -join "`n").Contains("Hello from Fusion")
    }
    $built -and $ran
}

Test-Case "fuc2 compiles fusionc.fu (ecosystem)" {
    $out = & $FUC2 (Join-Path $ROOT "fusion_ecosystem\src\fu\fusionc.fu") -o test_fuc2_ecosystem.exe 2>&1
    $built = $LASTEXITCODE -eq 0
    $ran = $false
    if ($built -and (Test-Path "test_fuc2_ecosystem.exe")) {
        $runOut = & ".\test_fuc2_ecosystem.exe" 2>&1
        $ran = ($runOut -join "`n").Contains("Pipeline Restored")
    }
    $built -and $ran
}

Test-Case "fuc2 --vortex borrow check" {
    $out = & $FUC2 (Join-Path $FIXTURES "hello_world.fu") -o test_vortex.exe --vortex 2>&1
    $LASTEXITCODE -eq 0 -and ($out -join "`n").Contains("Vortex check passed")
}

Test-Case "fuc2 --emit-flat" {
    $out = & $FUC2 (Join-Path $FIXTURES "arrays.fu") -o test_flat.exe --emit-flat --flat-output test_flat.fu 2>&1
    $LASTEXITCODE -eq 0 -and (Test-Path "test_flat.fu")
}

#===========================================================
# SECTION 8: Self-Hosting Milestone
#===========================================================
Write-Host ""
Write-Host "--- 8. Self-hosting milestone ---" -ForegroundColor Yellow

Test-Case "self-hosting preprocessor compiles" {
    $out = & $FUC2 (Join-Path $ROOT "self_hosting_preprocessor.fu") -o test_selfhost.exe 2>&1
    $LASTEXITCODE -eq 0
}

Test-Case "self-hosting preprocessor runs itself" {
    if (Test-Path "test_selfhost.exe") {
        $out = & ".\test_selfhost.exe" 2>&1
        ($out -join "`n").Contains("SELF-HOSTING MILESTONE ACHIEVED")
    } else {
        $false
    }
}

#===========================================================
# SECTION 9: Stage1 Self-Compilation
#===========================================================
Write-Host ""
Write-Host "--- 9. Stage1 self-compilation ---" -ForegroundColor Yellow

$stage1 = Join-Path $ROOT "crates\fuc\src\pure_fusion_compiler_minimal.fu"

Test-Case "stage1 compiles itself" {
    if (Test-Path $stage1) {
        $out = & $FUC $stage1 --emit-bin -o test_stage1_v2.exe 2>&1
        $LASTEXITCODE -eq 0
    } else {
        $false
    }
}

Test-Case "stage1 executes" {
    if (Test-Path "test_stage1_v2.exe") {
        $out = & ".\test_stage1_v2.exe" 2>&1
        $LASTEXITCODE -eq 0
    } else {
        $false
    }
}

#===========================================================
# SECTION 10: Vortex Borrow Checker
#===========================================================
Write-Host ""
Write-Host "--- 10. Vortex borrow checker (standalone) ---" -ForegroundColor Yellow

$vortexExe = Join-Path $ROOT "fusion_ecosystem\target\release\vortex_borrow_checker.exe"

Test-Case "vortex borrow checker runs" {
    if (Test-Path $vortexExe) {
        $out = & $vortexExe 2>&1
        $LASTEXITCODE -eq 0
    } else {
        $false
    }
}

#===========================================================
# SUMMARY
#===========================================================
Write-Host ""
Write-Host "=============================================" -ForegroundColor Cyan
Write-Host "  RESULTS: $pass / $total passed" -ForegroundColor $(if ($pass -eq $total) { "Green" } else { "Yellow" })
Write-Host "=============================================" -ForegroundColor Cyan
Write-Host ""
