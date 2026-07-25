# Fusion v2.0 Vortex - Windows Uninstaller
$InstallDir = "$env:LOCALAPPDATA\Fusion"
$BinDir = "$InstallDir\bin"

Write-Host "=== Fusion v2.0 Vortex Uninstaller ==="

# Remove from PATH
$CurrentPath = [Environment]::GetEnvironmentVariable("Path", "User")
$NewPath = ($CurrentPath -split ";" | Where-Object { $_ -ne $BinDir }) -join ";"
[Environment]::SetEnvironmentVariable("Path", $NewPath, "User")

# Remove files
Remove-Item -Recurse -Force $InstallDir -ErrorAction SilentlyContinue

Write-Host "Fusion v2.0 Vortex has been uninstalled."
Write-Host "Restart your terminal to apply PATH changes."