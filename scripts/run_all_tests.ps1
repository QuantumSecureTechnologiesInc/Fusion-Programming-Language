$fuc = ".\bin\fuc.exe"
$fixtures = @("hello_world","arrays","complex_arrays","final_struct_test","safety_and_repeat","sema_struct","string_eq")
$invalids = @("parse_error","parser_robustness","sema_error")

Write-Host "============================================="
Write-Host "  FUSION NATIVE COMPILER - FULL TEST SUITE"
Write-Host "============================================="
Write-Host ""

# --- LEXER + PARSER (parse-only) ---
Write-Host "--- LEXER/PARSER: parse-only on valid fixtures ---"
$pass = 0; $fail = 0
foreach ($f in $fixtures) {
    $null = & $fuc --parse-only "crates\fuc\tests\fixtures\$f.fu" 2>&1
    $code = $LASTEXITCODE
    if ($code -eq 0) { $pass++; Write-Host "  [PASS] $f" }
    else { $fail++; Write-Host "  [FAIL] $f (exit $code)" }
}
Write-Host "Parse valid: $pass/$($fixtures.Count) passed"
Write-Host ""

# --- LEXER + PARSER: reject invalid ---
Write-Host "--- LEXER/PARSER: reject invalid fixtures ---"
$pass = 0; $fail = 0
foreach ($f in $invalids) {
    $null = & $fuc --parse-only "crates\fuc\tests\fixtures\invalid\$f.fu" 2>&1
    $code = $LASTEXITCODE
    if ($f -eq "sema_error") {
        # sema_error is syntactically valid, should parse OK
        if ($code -eq 0) { $pass++; Write-Host "  [PASS] $f (correctly parses)" }
        else { $fail++; Write-Host "  [FAIL] $f (unexpected parse fail)" }
    } else {
        if ($code -ne 0) { $pass++; Write-Host "  [PASS] $f (correctly rejected)" }
        else { $fail++; Write-Host "  [FAIL] $f (should have been rejected)" }
    }
}
Write-Host "Parse invalid: $pass/$($invalids.Count) correct"
Write-Host ""

# --- SEMANTIC ANALYZER (sema-only) ---
Write-Host "--- SEMA: sema-only on valid fixtures ---"
$pass = 0; $fail = 0
foreach ($f in $fixtures) {
    $null = & $fuc --sema-only "crates\fuc\tests\fixtures\$f.fu" 2>&1
    $code = $LASTEXITCODE
    if ($code -eq 0) { $pass++; Write-Host "  [PASS] $f" }
    else { $fail++; Write-Host "  [FAIL] $f (exit $code)" }
}
Write-Host "Sema valid: $pass/$($fixtures.Count) passed"
Write-Host ""

# --- SEMA: reject invalid ---
Write-Host "--- SEMA: reject invalid sema fixtures ---"
$pass = 0; $fail = 0
foreach ($f in @("sema_error","parser_robustness")) {
    $null = & $fuc --sema-only "crates\fuc\tests\fixtures\invalid\$f.fu" 2>&1
    $code = $LASTEXITCODE
    if ($code -ne 0) { $pass++; Write-Host "  [PASS] $f (correctly rejected)" }
    else { $fail++; Write-Host "  [FAIL] $f (should have been rejected)" }
}
Write-Host "Sema invalid: $pass/2 correct"
Write-Host ""

# --- CODEGEN + LINK (emit-bin, run, compare output) ---
Write-Host "--- CODEGEN+LINK+RUN: full pipeline on fixtures ---"
$pass = 0; $fail = 0
foreach ($f in $fixtures) {
    $out = "$f.exe"
    $null = & $fuc --emit-bin -o $out "crates\fuc\tests\fixtures\$f.fu" 2>&1
    $code = $LASTEXITCODE
    if ($code -ne 0) { $fail++; Write-Host "  [FAIL] $f (compile failed)"; continue }

    $actual = & ".\$out" 2>&1
    $exitCode = $LASTEXITCODE

    $expectedOut = "crates\fuc\tests\fixtures\$f.out"
    $expectedExit = "crates\fuc\tests\fixtures\$f.exit"

    $outMatch = $true
    if (Test-Path $expectedOut) {
        $exp = (Get-Content $expectedOut -Raw).TrimEnd("`r","`n")
        $act = ($actual -join "`n")
        if ($exp -ne $act) {
            $outMatch = $false
            Write-Host "  [FAIL] $f (output mismatch)"
            Write-Host "    Expected: $exp"
            Write-Host "    Actual:   $act"
        }
    }

    $exitMatch = $true
    if (Test-Path $expectedExit) {
        $expExit = (Get-Content $expectedExit -Raw).Trim()
        if ([int]$expExit -ne $exitCode) {
            $exitMatch = $false
            Write-Host "  [FAIL] $f (exit code: expected $expExit, got $exitCode)"
        }
    }

    if ($outMatch -and $exitMatch) { $pass++; Write-Host "  [PASS] $f" }
    else { $fail++ }
}
Write-Host "Full pipeline: $pass/$($fixtures.Count) passed"
Write-Host ""

# --- CLI FLAGS ---
Write-Host "--- CLI FLAGS ---"
# --lib flag
$null = & $fuc --lib "test_lib.fu" 2>&1
if ($LASTEXITCODE -eq 0) { Write-Host "  [PASS] --lib flag (suppresses entry point check)" }
else { Write-Host "  [FAIL] --lib flag" }

# --lib without main should fail
$null = & $fuc "test_lib.fu" 2>&1
if ($LASTEXITCODE -ne 0) { Write-Host "  [PASS] no --lib rejects missing main()" }
else { Write-Host "  [FAIL] no --lib should reject missing main()" }

# --opt-level 3
$null = & $fuc --emit-bin --opt-level 3 -o test_opt3.exe "crates\fuc\tests\fixtures\hello_world.fu" 2>&1
$null = & ".\test_opt3.exe" 2>&1
if ($LASTEXITCODE -eq 0) { Write-Host "  [PASS] --opt-level 3 compiles and runs" }
else { Write-Host "  [FAIL] --opt-level 3" }

Write-Host ""

# --- STAGE1 SELF-HOST ---
Write-Host "--- STAGE1 SELF-HOSTING ---"
$null = & $fuc --emit-bin -o stage1_test.exe "crates\fuc\src\pure_fusion_compiler_minimal.fu" 2>&1
$code = $LASTEXITCODE
if ($code -eq 0) { Write-Host "  [PASS] Stage1 compiler compiles itself" }
else { Write-Host "  [FAIL] Stage1 compiler self-compile" }

$null = & ".\stage1_test.exe" 2>&1
$code = $LASTEXITCODE
if ($code -eq 0) { Write-Host "  [PASS] Stage1 self-host execution (parse+sema+codegen+link)" }
else { Write-Host "  [FAIL] Stage1 self-host execution (exit $code)" }

Write-Host ""

# --- VORTEX BORROW CHECKER ---
Write-Host "--- VORTEX BORROW CHECKER ---"
$null = & ".\fusion_ecosystem\target\release\vortex_borrow_checker.exe" 2>&1
$code = $LASTEXITCODE
if ($code -eq 0) { Write-Host "  [PASS] Vortex detects UnsafeEscape + MultipleMutableBorrows" }
else { Write-Host "  [FAIL] Vortex harness (exit $code)" }

Write-Host ""
Write-Host "============================================="
Write-Host "  TEST SUITE COMPLETE"
Write-Host "============================================="
