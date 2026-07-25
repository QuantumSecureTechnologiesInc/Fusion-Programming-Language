# Fusion v2.0 Vortex - Windows PowerShell Installer
#Requires -RunAsAdministrator

$ErrorActionPreference = "Stop"
$FusionVersion = "2.0.0"
$InstallDir = "$env:LOCALAPPDATA\Fusion"
$BinDir = "$InstallDir\bin"

Write-Host "=== Fusion v2.0 Vortex Installer ===" -ForegroundColor Cyan
Write-Host ""

# Step 1: Check Rust
Write-Host "[1/6] Checking Rust installation..."
try {
    $rustVersion = & rustc --version 2>&1
    Write-Host "  Found: $rustVersion" -ForegroundColor Green
} catch {
    Write-Host "  Error: Rust not found. Installing..." -ForegroundColor Yellow
    & Invoke-RestMethod -Uri https://win.rustup.rs/x86_64 -OutFile rustup-init.exe
    & .\rustup-init.exe -y
    Remove-Item rustup-init.exe
}

# Step 2: Build compiler
Write-Host "[2/6] Building fuc compiler..."
& cargo build --release --path crates/fuc --features llvm

# Step 3: Build CLI
Write-Host "[3/6] Building fusion CLI..."
& cargo build --release --path tools/fusion-cli

# Step 4: Install to PATH
Write-Host "[4/6] Installing to $BinDir..."
New-Item -ItemType Directory -Force -Path $BinDir | Out-Null
Copy-Item "target\release\fuc.exe" -Destination $BinDir
Copy-Item "target\release\fusion.exe" -Destination $BinDir

# Step 5: Add to PATH
Write-Host "[5/6] Adding to PATH..."
$CurrentPath = [Environment]::GetEnvironmentVariable("Path", "User")
if ($CurrentPath -notlike "*$BinDir*") {
    [Environment]::SetEnvironmentVariable("Path", "$CurrentPath;$BinDir", "User")
    Write-Host "  Added $BinDir to user PATH" -ForegroundColor Green
}

# Step 6: Install stdlib
Write-Host "[6/6] Installing standard library..."
New-Item -ItemType Directory -Force -Path "$InstallDir\stdlib" | Out-Null
Copy-Item -Recurse "stdlib\*" -Destination "$InstallDir\stdlib"

Write-Host ""
Write-Host "=== Installation Complete ===" -ForegroundColor Green
Write-Host "Restart your terminal to use 'fusion' command."