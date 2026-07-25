# Approve a pending high-risk --no-verify bypass.
#
# Records an explicit, owner-authorized approval in the audit trail
# (.fusion/audit/no-verify-bypass.log). Until a pending high-risk bypass is
# approved here, the pre-commit gate blocks further commits.
#
# Usage:
#   powershell -ExecutionPolicy Bypass -File scripts\approve_bypass.ps1 `
#       -Commit <full-sha> -Justification "Emergency hotfix, reviewed by @owner"
#
# See BUILD_POLICY.md ("--no-verify Bypass Auditing") for the full policy.

[CmdletBinding()]
param(
    [Parameter(Mandatory = $true)]
    [string]$Commit,

    [string]$Approver,

    [Parameter(Mandatory = $true)]
    [string]$Justification
)

$ErrorActionPreference = "Stop"
$engine = Join-Path $PSScriptRoot "bypass_audit.ps1"

if (-not (Test-Path $engine)) {
    Write-Host "[ERROR] Audit engine not found: $engine" -ForegroundColor Red
    exit 2
}

$engineArgs = @(
    "-Mode", "Approve",
    "-Commit", $Commit,
    "-Justification", $Justification
)
if (-not [string]::IsNullOrWhiteSpace($Approver)) {
    $engineArgs += @("-Approver", $Approver)
}

& powershell -ExecutionPolicy Bypass -NoProfile -File $engine @engineArgs
exit $LASTEXITCODE
