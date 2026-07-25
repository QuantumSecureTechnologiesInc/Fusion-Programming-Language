# Fusion post-commit hook (PowerShell)
#
# Runs after every commit to detect whether pre-commit was bypassed
# (git commit --no-verify) and, if so, records an audit entry. High-risk
# bypasses (crates/fuc/, runtime/, stdlib/) are flagged for explicit approval;
# the pre-commit escalation gate then blocks further commits until approved.
#
# This hook never fails a commit -- the commit has already happened. It only
# records the audit trail and prints guidance.

$ErrorActionPreference = "Continue"

$engine = Join-Path $PSScriptRoot "..\scripts\bypass_audit.ps1"
if (Test-Path $engine) {
    & powershell -ExecutionPolicy Bypass -NoProfile -File $engine -Mode Detect
}

exit 0
