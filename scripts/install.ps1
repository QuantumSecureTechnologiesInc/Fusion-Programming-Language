$ErrorActionPreference = "Stop"

Write-Host ">>> Fusion Native Installer (Windows)" -ForegroundColor Cyan
Write-Host ">>> Strictly No Rust Dependencies" -ForegroundColor Cyan

# 1. Compile Native Runtime (C)
# Requires: cl.exe (MSVC) or gcc (MinGW)
$RuntimeSrc = "runtime\native\fusionrt.c"
$BinDir = "bin"

if (!(Test-Path $BinDir)) { New-Item -ItemType Directory -Path $BinDir | Out-Null }

if (Get-Command "cl" -ErrorAction SilentlyContinue) {
    Write-Host "Using MSVC (cl.exe)..." -ForegroundColor Yellow
    # Compile /c /Fo:runtime\native\fusionrt.obj runtime\native\fusionrt.c
    cl.exe /c /nologo /Fo"$BinDir\fusionrt.obj" $RuntimeSrc
    
    # Create static lib
    lib.exe /nologo /OUT:"$BinDir\fusionrt.lib" "$BinDir\fusionrt.obj"
}
elseif (Get-Command "gcc" -ErrorAction SilentlyContinue) {
    Write-Host "Using GCC..." -ForegroundColor Yellow
    gcc -c $RuntimeSrc -o "$BinDir\fusionrt.o"
    ar rcs "$BinDir\libfusionrt.a" "$BinDir\fusionrt.o"
}
else {
    Write-Error "No C compiler found (cl.exe or gcc needed for runtime)."
    exit 1
}

Write-Host ">>> Runtime built." -ForegroundColor Green

# 2. Check for existing compiler
$Compiler = "$BinDir\fuc.exe"
if (!(Test-Path $Compiler)) {
    Write-Warning "No existing compiler found at $Compiler."
    Write-Warning "Cannot self-host without a Stage0 binary."
    # In full release, we would pull a binary here.
    exit 1
}

Write-Host ">>> Compiling Fusion Compiler (Self-Hosting)..." -ForegroundColor Yellow

# 3. Compile crates/fuc
& $Compiler "crates\fuc\src\main.fu" `
    -o "$BinDir\fuc_new.obj" `
    --lib-path $BinDir

if ($LASTEXITCODE -ne 0) {
    Write-Error "Compilation failed."
    exit 1
}

Write-Host ">>> Linking new compiler..." -ForegroundColor Yellow

if (Get-Command "cl" -ErrorAction SilentlyContinue) {
    link.exe /nologo /OUT:"$BinDir\fuc_new.exe" "$BinDir\fuc_new.obj" "$BinDir\fusionrt.lib"
}
else {
    gcc "$BinDir\fuc_new.obj" -o "$BinDir\fuc_new.exe" -L $BinDir -lfusionrt
}

Write-Host ">>> SUCCESS: New compiler built at $BinDir\fuc_new.exe" -ForegroundColor Green
