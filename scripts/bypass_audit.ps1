# Fusion --no-verify Bypass Audit Engine
#
# Central engine for auditing and escalating `git commit --no-verify` bypasses
# that touch high-risk, production-impacting areas (crates/fuc/, runtime/,
# stdlib/).
#
# Because `--no-verify` skips the pre-commit hook entirely, the bypass cannot be
# detected from inside pre-commit. Instead:
#   * pre-commit writes a freshness marker on success ("<head> <staged-tree>").
#   * post-commit compares the marker against the new commit's parent AND its
#     committed tree to decide whether pre-commit actually ran for THIS commit
#     (Mode=Detect). Binding the tree hash defeats a stale marker left behind by
#     an aborted commit that is later re-attempted with --no-verify.
#   * pre-commit refuses to start a new commit while an unapproved high-risk
#     bypass is pending (Mode=Gate), which enforces explicit approval.
#   * scripts/approve_bypass.ps1 records the reviewer's approval (Mode=Approve).
#
# The audit trail is a JSON-lines file at .fusion/audit/no-verify-bypass.log
# (matched by the repo's *.log ignore rule, so it stays a local, reviewable
# record and is never committed by the same hook that writes it).

[CmdletBinding()]
param(
    [ValidateSet("Detect", "Gate", "Approve")]
    [string]$Mode = "Detect",

    # Approve mode inputs
    [string]$Commit,
    [string]$Approver,
    [string]$Justification
)

$ErrorActionPreference = "Stop"

# High-risk, production-impacting path prefixes.
$HighRiskPrefixes = @("crates/fuc/", "runtime/", "stdlib/")

function Get-RepoRoot {
    $root = git rev-parse --show-toplevel 2>$null
    if ($LASTEXITCODE -ne 0 -or [string]::IsNullOrWhiteSpace($root)) {
        throw "bypass_audit: not inside a git work tree"
    }
    return $root.Trim()
}

function Get-GitDir {
    $dir = git rev-parse --git-dir 2>$null
    if ($LASTEXITCODE -ne 0 -or [string]::IsNullOrWhiteSpace($dir)) {
        throw "bypass_audit: cannot resolve git dir"
    }
    return $dir.Trim()
}

function Get-MarkerPath {
    return (Join-Path (Get-GitDir) "fusion_precommit_marker")
}

function Get-AuditLogPath {
    $auditDir = Join-Path (Get-RepoRoot) ".fusion/audit"
    if (-not (Test-Path $auditDir)) {
        New-Item -ItemType Directory -Force -Path $auditDir | Out-Null
    }
    return (Join-Path $auditDir "no-verify-bypass.log")
}

function Get-HeadSha {
    $sha = git rev-parse --verify --quiet HEAD 2>$null
    if ([string]::IsNullOrWhiteSpace($sha)) { return "ROOT" }
    return $sha.Trim()
}

# The marker value pre-commit would have written for THIS commit:
# "<parent-of-HEAD> <tree-of-HEAD>". If pre-commit ran, the marker it wrote
# ("<HEAD-at-precommit> <staged-tree>") equals this, because the commit's parent
# is the HEAD pre-commit saw and the committed tree is the tree it staged.
function Get-ExpectedMarker {
    $parent = git rev-parse --verify --quiet "HEAD~1" 2>$null
    if ([string]::IsNullOrWhiteSpace($parent)) { $parent = "ROOT" }
    $tree = git rev-parse --verify --quiet "HEAD^{tree}" 2>$null
    if ([string]::IsNullOrWhiteSpace($tree)) { $tree = "NOTREE" }
    return ("{0} {1}" -f $parent.Trim(), $tree.Trim())
}

function Read-AuditRecords {
    $log = Get-AuditLogPath
    if (-not (Test-Path $log)) { return @() }
    $records = @()
    foreach ($line in Get-Content $log -ErrorAction SilentlyContinue) {
        if ([string]::IsNullOrWhiteSpace($line)) { continue }
        try { $records += ($line | ConvertFrom-Json) } catch { }
    }
    return $records
}

function Append-AuditRecord {
    param($Record)
    $log = Get-AuditLogPath
    $json = ($Record | ConvertTo-Json -Compress -Depth 5)
    Add-Content -Path $log -Value $json -Encoding UTF8
}

# Bypass records that touch high-risk paths and have no matching approval.
function Get-PendingHighRiskBypasses {
    $records = Read-AuditRecords
    $approvedCommits = @{}
    foreach ($r in $records) {
        if ($r.event -eq "approval" -and $r.commit) { $approvedCommits[$r.commit] = $true }
    }
    $pending = @()
    foreach ($r in $records) {
        if ($r.event -eq "bypass" -and $r.high_risk -eq $true) {
            if (-not $approvedCommits.ContainsKey($r.commit)) { $pending += $r }
        }
    }
    return $pending
}

switch ($Mode) {

    #===========================================================================
    # DETECT: called from post-commit. Decides whether --no-verify was used.
    #===========================================================================
    "Detect" {
        $marker = Get-MarkerPath
        $expected = Get-ExpectedMarker

        $preCommitRan = $false
        if (Test-Path $marker) {
            $recorded = (Get-Content $marker -Raw -ErrorAction SilentlyContinue).Trim()
            if ($recorded -eq $expected) { $preCommitRan = $true }
        }
        # Consume the marker so it can never satisfy a later commit.
        Remove-Item $marker -Force -ErrorAction SilentlyContinue

        if ($preCommitRan) {
            # Normal, verified commit -- nothing to audit.
            exit 0
        }

        # pre-commit did NOT run for this commit => --no-verify (or hook failure).
        $commit = Get-HeadSha
        $branch = (git rev-parse --abbrev-ref HEAD 2>$null)
        if ($branch) { $branch = $branch.Trim() } else { $branch = "(unknown)" }
        $userName = (git config user.name 2>$null); if ($userName) { $userName = $userName.Trim() } else { $userName = $env:USERNAME }
        $userEmail = (git config user.email 2>$null); if ($userEmail) { $userEmail = $userEmail.Trim() } else { $userEmail = "(unknown)" }

        $changed = @()
        $diffOut = git diff-tree --no-commit-id --name-only -r HEAD 2>$null
        if ($diffOut) {
            foreach ($f in ($diffOut -split "`n")) {
                $f = $f.Trim()
                if (-not [string]::IsNullOrWhiteSpace($f)) { $changed += $f }
            }
        }

        $highRiskFiles = @()
        foreach ($f in $changed) {
            foreach ($prefix in $HighRiskPrefixes) {
                if ($f.StartsWith($prefix)) { $highRiskFiles += $f; break }
            }
        }
        $isHighRisk = $highRiskFiles.Count -gt 0

        $record = [ordered]@{
            timestamp    = (Get-Date).ToUniversalTime().ToString("yyyy-MM-ddTHH:mm:ssZ")
            event        = "bypass"
            commit       = $commit
            branch       = $branch
            user         = $userName
            email        = $userEmail
            high_risk    = $isHighRisk
            files        = $changed
            risk_files   = $highRiskFiles
            status       = $(if ($isHighRisk) { "PENDING_APPROVAL" } else { "LOGGED" })
        }
        Append-AuditRecord -Record $record

        $log = Get-AuditLogPath
        Write-Host ""
        Write-Host "================================================================" -ForegroundColor Yellow
        Write-Host "[AUDIT] --no-verify bypass recorded" -ForegroundColor Yellow
        Write-Host "   Commit : $commit" -ForegroundColor Yellow
        Write-Host "   Branch : $branch" -ForegroundColor Yellow
        Write-Host "   Author : $userName <$userEmail>" -ForegroundColor Yellow
        Write-Host "   Trail  : $log" -ForegroundColor DarkGray

        if ($isHighRisk) {
            Write-Host ""
            Write-Host "[ESCALATION REQUIRED] Production-impacting files bypassed pre-commit:" -ForegroundColor Red
            foreach ($f in $highRiskFiles) { Write-Host "     - $f" -ForegroundColor Red }
            Write-Host ""
            Write-Host "   These changes touch high-risk areas (crates/fuc/, runtime/, stdlib/)" -ForegroundColor Red
            Write-Host "   and require EXPLICIT APPROVAL. The next commit is blocked until an" -ForegroundColor Red
            Write-Host "   owner approves this bypass:" -ForegroundColor Red
            Write-Host ""
            Write-Host "     powershell -ExecutionPolicy Bypass -File scripts\approve_bypass.ps1 -Commit $commit -Justification `"<reason>`"" -ForegroundColor Cyan
            Write-Host ""
            Write-Host "================================================================" -ForegroundColor Red
        } else {
            Write-Host "================================================================" -ForegroundColor Yellow
        }
        exit 0
    }

    #===========================================================================
    # GATE: called at the start of pre-commit. Blocks new commits while an
    # unapproved high-risk bypass is pending -> enforces explicit approval.
    #===========================================================================
    "Gate" {
        $pending = Get-PendingHighRiskBypasses
        if ($pending.Count -eq 0) { exit 0 }

        $log = Get-AuditLogPath
        Write-Host ""
        Write-Host "[BLOCKED] Unapproved high-risk --no-verify bypass pending approval" -ForegroundColor Red
        foreach ($p in $pending) {
            Write-Host "   Commit : $($p.commit)  (author: $($p.user))" -ForegroundColor Red
            foreach ($f in $p.risk_files) { Write-Host "     - $f" -ForegroundColor Red }
        }
        Write-Host ""
        Write-Host "   A prior bypass touched production-impacting code and must be" -ForegroundColor Yellow
        Write-Host "   reviewed and approved before further commits are allowed." -ForegroundColor Yellow
        Write-Host "   Audit trail: $log" -ForegroundColor DarkGray
        Write-Host ""
        Write-Host "   To approve (owner only):" -ForegroundColor Yellow
        foreach ($p in $pending) {
            Write-Host "     powershell -ExecutionPolicy Bypass -File scripts\approve_bypass.ps1 -Commit $($p.commit) -Justification `"<reason>`"" -ForegroundColor Cyan
        }
        Write-Host ""
        exit 1
    }

    #===========================================================================
    # APPROVE: records an explicit approval for a pending bypass.
    #===========================================================================
    "Approve" {
        if ([string]::IsNullOrWhiteSpace($Commit)) {
            Write-Host "[ERROR] -Commit is required for approval" -ForegroundColor Red
            exit 2
        }
        $records = Read-AuditRecords
        $match = $records | Where-Object { $_.event -eq "bypass" -and $_.commit -eq $Commit }
        if (-not $match) {
            Write-Host "[ERROR] No recorded bypass found for commit '$Commit'" -ForegroundColor Red
            Write-Host "        Use the full commit SHA shown in the audit trail." -ForegroundColor Yellow
            exit 2
        }

        $approver = if (-not [string]::IsNullOrWhiteSpace($Approver)) { $Approver } else {
            $n = git config user.name 2>$null
            if ($n) { $n.Trim() } else { $env:USERNAME }
        }
        $reason = if (-not [string]::IsNullOrWhiteSpace($Justification)) { $Justification } else { "(no justification provided)" }

        $record = [ordered]@{
            timestamp     = (Get-Date).ToUniversalTime().ToString("yyyy-MM-ddTHH:mm:ssZ")
            event         = "approval"
            commit        = $Commit
            approver      = $approver
            justification = $reason
            status        = "APPROVED"
        }
        Append-AuditRecord -Record $record

        Write-Host "[OK] Bypass for commit $Commit approved by $approver" -ForegroundColor Green
        Write-Host "     Justification: $reason"
        Write-Host "     Audit trail  : $(Get-AuditLogPath)" -ForegroundColor DarkGray
        exit 0
    }
}
