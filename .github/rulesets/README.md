# Branch Protection Rulesets

This directory contains the GitHub Repository Rulesets that enforce branch protection on the Fusion Vortex repository.

## Ruleset: Fusion Vortex Branch Protection - Main & Develop

**File:** `branch-protection-main-develop.json`

### Purpose

Enforces mandatory CI status checks and PR approval requirements on the `main` and `develop` branches to prevent regressions and ensure code quality before merge.

### Requirements for Merge

1. **Required Status Check:** The `CI Success` job must pass before any PR can be merged. This job aggregates results from:
   - `Test Rust Compiler` (multi-OS, multi-toolchain)
   - `Security Audit`
   - `Test VS Code Extension`

2. **Code Owner Review:** At least one approval from a CODEOWNERS-designated reviewer is required for each PR.

3. **Dismiss Stale Reviews:** Approvals are dismissed when new commits are pushed to prevent stale approvals.

4. **Linear History:** Only squash merge and rebase are permitted to maintain a clean commit history.

5. **Signed Commits:** All commits must be signed with a valid GPG or SSH key.

### Application

- **Target Branches:** `refs/heads/main` and `refs/heads/develop`
- **Enforcement:** Active (mandatory, not advisory)

### Deployment

To apply this ruleset to the GitHub repository:

```bash
# Using the apply script
bash scripts/apply_branch_protection.sh

# Or manually via GitHub CLI
gh api repos/QuantumSecureTechnologiesInc/Fusion-Vortex/rulesets \
  --method POST \
  --input .github/rulesets/branch-protection-main-develop.json
```

### Verification

To verify the ruleset is properly applied:

```powershell
# Run the verification script
pwsh scripts/verify-branch-protection.ps1

# With recording to this README
pwsh scripts/verify-branch-protection.ps1 -Record
```

### Troubleshooting

| Issue | Cause | Fix |
|-------|-------|-----|
| `CI Success` check not appearing | Workflow not triggered or job name changed | Ensure the CI workflow runs on PRs and the aggregation job is named exactly "CI Success" |
| CODEOWNERS review not required | Ruleset not applied or CODEOWNERS invalid | Run `gh api repos/<owner>/<repo>/codeowners/errors` to validate |
| Bypass actors needed | Emergency overrides required | Add entries to `bypass_actors` in the ruleset JSON (use sparingly) |

---

<!-- verification-record -->
