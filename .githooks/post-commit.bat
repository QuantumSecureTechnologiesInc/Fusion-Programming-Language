@echo off
REM Fusion post-commit hook (Windows batch wrapper)
REM Delegates to the PowerShell post-commit detector, which records an audit
REM entry when a commit bypassed pre-commit (git commit --no-verify).
powershell -ExecutionPolicy Bypass -NoProfile -File "%~dp0post-commit.ps1"
exit /b 0
