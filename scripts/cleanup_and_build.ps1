$curBase = "c:\Users\Matth\Downloads\Fusion v2.0 Vortex"
$files = @("remove_rs.ps1","find_converter.ps1","check_converted.ps1","copy_converted.ps1")
foreach ($f in $files) {
    $p = Join-Path $curBase $f
    if (Test-Path $p) { Remove-Item $p -Force; Write-Host "Deleted $f" }
}

# Also archive remaining empty build dirs if they exist
$archiveBase = Join-Path $curBase ".archive\root-junk"
if (!(Test-Path $archiveBase)) { New-Item -ItemType Directory $archiveBase -Force | Out-Null }

# Rebuild runtime with MSVC
Write-Host ""
Write-Host "=== Rebuilding C runtime ==="
$vcvars = "C:\Program Files\Microsoft Visual Studio\18\Professional\VC\Auxiliary\Build\vcvarsall.bat"
$rtSrc = Join-Path $curBase "runtime\runtime.c"
$rtObj = Join-Path $curBase "bin\runtime.obj"
$rtLib = Join-Path $curBase "bin\fusionrt.lib"

if (Test-Path $rtSrc) {
    $cmd = """$vcvars"" x64 >nul 2>&1 && cl /c ""$rtSrc"" /Fo:""$rtObj"" /W3 /O2 /nologo 2>&1 && lib /out:""$rtLib"" ""$rtObj"" /nologo 2>&1 && echo BUILD_OK"
    $result = cmd /c $cmd 2>&1
    Write-Host $result
    if ($result -match "BUILD_OK") {
        Write-Host "Runtime built successfully: bin/fusionrt.lib"
    } else {
        Write-Host "Runtime build may have issues (see above)"
    }
}

# Also build fusionrt.c from runtime/native
$fnSrc = Join-Path $curBase "runtime\native\fusionrt.c"
$fnObj = Join-Path $curBase "bin\fusionrt_native.obj"
if (Test-Path $fnSrc) {
    $cmd2 = """$vcvars"" x64 >nul 2>&1 && cl /c ""$fnSrc"" /Fo:""$fnObj"" /I ""$(Join-Path $curBase 'runtime\native')"" /W3 /O2 /nologo 2>&1 && echo FUSIONRT_OK"
    $result2 = cmd /c $cmd2 2>&1
    Write-Host $result2
}
