@echo off
rem Forward to PowerShell implementation
powershell -ExecutionPolicy Bypass -NoProfile -File "%~dp0jfuc.ps1" %*
