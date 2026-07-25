#!/usr/bin/env bash
# apply_branch_protection.sh — apply the committed repository ruleset that
# enforces CI status checks and code-owner review (PR approval) to the GitHub
# repository. Idempotent: creates the ruleset if it is missing, or updates it
# if the local name already exists on the remote.
#
# Usage:    bash scripts/apply_branch_protection.sh [owner/name]
# Requires: gh (GitHub CLI) authenticated with repo admin permissions.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
RULESET_FILE="${SCRIPT_DIR}/../.github/rulesets/branch-protection-main-develop.json"

if [[ ! -f "$RULESET_FILE" ]]; then
  echo "Ruleset file not found: $RULESET_FILE" >&2
  exit 1
fi

# Resolve repository
REPO="${1:-}"
if [[ -z "$REPO" ]]; then
  REMOTE_URL=$(git -C "$SCRIPT_DIR/.." remote get-url origin 2>/dev/null || true)
  if [[ "$REMOTE_URL" =~ github\.com[:/](.+)/(.+?)(\.git)?$ ]]; then
    REPO="${BASH_REMATCH[1]}/${BASH_REMATCH[2]}"
  else
    REPO="QuantumSecureTechnologiesInc/Fusion-Vortex"
  fi
fi

echo "Repository: $REPO"
echo "Ruleset file: $RULESET_FILE"

NAME="$(jq -r '.name' "$RULESET_FILE")"
echo "Ruleset name: $NAME"

# Check if ruleset already exists
EXISTING_ID=$(gh api "repos/$REPO/rulesets" --jq ".[] | select(.name == \"$NAME\") | .id" 2>/dev/null || true)

if [[ -n "$EXISTING_ID" ]]; then
  echo "Ruleset already exists (id=$EXISTING_ID). Updating..."
  gh api "repos/$REPO/rulesets/$EXISTING_ID" \
    --method PUT \
    --input "$RULESET_FILE" \
    --silent
  echo "Ruleset updated successfully."
else
  echo "Ruleset not found. Creating..."
  gh api "repos/$REPO/rulesets" \
    --method POST \
    --input "$RULESET_FILE" \
    --silent
  echo "Ruleset created successfully."
fi

echo ""
echo "Branch protection is now active on main and develop branches."
echo "Required status check: CI Success"
echo "Code owner review: Required"
echo ""
echo "Verify with: pwsh scripts/verify-branch-protection.ps1"
