$fuc = "c:\Users\Matth\Downloads\Fusion v2.0 Vortex\bin\fuc.exe"
$srcDir = "c:\Users\Matth\Downloads\Fusion v2.0 Vortex\crates\fuc\src"

Write-Host "=== Parse-only check on compiler sources ==="
$fuFiles = Get-ChildItem -Path $srcDir -Filter "*.fu" | Sort-Object Name
$pass = 0; $fail = 0
foreach ($f in $fuFiles) {
    $result = & $fuc $f.FullName --parse-only 2>&1
    if ($result -match "Finished parse \[ok\]") {
        Write-Host "  OK  $($f.Name)"
        $pass++
    } else {
        Write-Host "  FAIL $($f.Name)" -ForegroundColor Red
        $result | ForEach-Object { Write-Host "    $_" }
        $fail++
    }
}
Write-Host ""
Write-Host "Results: $pass passed, $fail failed out of $($fuFiles.Count) files"

# Also check the new neuralmesh_vortex_pqc
Write-Host ""
Write-Host "=== Parse check: neuralmesh_vortex_pqc ==="
$pqcFu = "c:\Users\Matth\Downloads\Fusion v2.0 Vortex\crates\neuralmesh_vortex_pqc\src\lib.fu"
$result = & $fuc $pqcFu --parse-only 2>&1
$result | ForEach-Object { Write-Host "  $_" }
