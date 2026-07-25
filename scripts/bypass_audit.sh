#!/usr/bin/env bash
# Fusion --no-verify Bypass Audit Engine (bash mirror of scripts/bypass_audit.ps1)
#
# Modes:
#   detect   Called from post-commit. Decides whether --no-verify was used by
#            comparing the pre-commit freshness marker ("<head> <staged-tree>")
#            against the new commit's parent AND committed tree, and appends an
#            audit record for any bypass. Binding the tree hash defeats a stale
#            marker left by an aborted commit later re-attempted with --no-verify.
#   gate     Called at the start of pre-commit. Blocks new commits while an
#            unapproved high-risk bypass is pending (enforces explicit approval).
#   approve  Records an explicit approval for a pending bypass.
#
# Audit trail (JSON lines): .fusion/audit/no-verify-bypass.log
# The log line format is kept identical to the PowerShell engine so the two are
# fully interoperable.

set -u

MODE="${1:-detect}"

RED='\033[0;31m'
YELLOW='\033[1;33m'
GREEN='\033[0;32m'
CYAN='\033[0;36m'
GRAY='\033[0;90m'
NC='\033[0m'

HIGH_RISK_PREFIXES=("crates/fuc/" "runtime/" "stdlib/")

repo_root() { git rev-parse --show-toplevel 2>/dev/null; }
git_dir() { git rev-parse --git-dir 2>/dev/null; }
marker_path() { echo "$(git_dir)/fusion_precommit_marker"; }

audit_log() {
    local dir
    dir="$(repo_root)/.fusion/audit"
    mkdir -p "$dir"
    echo "$dir/no-verify-bypass.log"
}

head_sha() { git rev-parse --verify --quiet HEAD 2>/dev/null || echo "ROOT"; }

# The marker value pre-commit would have written for THIS commit:
# "<parent-of-HEAD> <tree-of-HEAD>".
expected_marker() {
    local parent tree
    parent="$(git rev-parse --verify --quiet HEAD~1 2>/dev/null || echo ROOT)"
    tree="$(git rev-parse --verify --quiet 'HEAD^{tree}' 2>/dev/null || echo NOTREE)"
    printf '%s %s' "$parent" "$tree"
}

# Escape a string for safe embedding inside a JSON double-quoted value.
json_escape() {
    local s="$1"
    s="${s//\\/\\\\}"
    s="${s//\"/\\\"}"
    printf '%s' "$s"
}

# Join array elements ("a" "b") into a JSON array string: ["a","b"]
json_array() {
    local out="["
    local first=1
    local item
    for item in "$@"; do
        if [ $first -eq 0 ]; then out="$out,"; fi
        out="$out\"$(json_escape "$item")\""
        first=0
    done
    echo "$out]"
}

case "$MODE" in

    detect)
        MARKER="$(marker_path)"
        EXPECTED="$(expected_marker)"
        PRECOMMIT_RAN=0
        if [ -f "$MARKER" ]; then
            # Strip only CR/LF -- the space between head and tree is significant.
            RECORDED="$(tr -d '\r\n' < "$MARKER")"
            if [ "$RECORDED" = "$EXPECTED" ]; then PRECOMMIT_RAN=1; fi
        fi
        rm -f "$MARKER" 2>/dev/null || true

        if [ $PRECOMMIT_RAN -eq 1 ]; then
            exit 0
        fi

        COMMIT="$(head_sha)"
        BRANCH="$(git rev-parse --abbrev-ref HEAD 2>/dev/null || echo '(unknown)')"
        USER_NAME="$(git config user.name 2>/dev/null || echo "${USER:-unknown}")"
        USER_EMAIL="$(git config user.email 2>/dev/null || echo '(unknown)')"

        CHANGED=()
        while IFS= read -r f; do
            [ -n "$f" ] && CHANGED+=("$f")
        done < <(git diff-tree --no-commit-id --name-only -r HEAD 2>/dev/null)

        RISK_FILES=()
        for f in "${CHANGED[@]:-}"; do
            [ -z "$f" ] && continue
            for prefix in "${HIGH_RISK_PREFIXES[@]}"; do
                if [[ "$f" == "$prefix"* ]]; then RISK_FILES+=("$f"); break; fi
            done
        done

        HIGH_RISK="false"
        STATUS="LOGGED"
        if [ "${#RISK_FILES[@]}" -gt 0 ]; then
            HIGH_RISK="true"
            STATUS="PENDING_APPROVAL"
        fi

        TS="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
        FILES_JSON="$(json_array "${CHANGED[@]:-}")"
        RISK_JSON="$(json_array "${RISK_FILES[@]:-}")"
        LINE="{\"timestamp\":\"$TS\",\"event\":\"bypass\",\"commit\":\"$COMMIT\",\"branch\":\"$(json_escape "$BRANCH")\",\"user\":\"$(json_escape "$USER_NAME")\",\"email\":\"$(json_escape "$USER_EMAIL")\",\"high_risk\":$HIGH_RISK,\"files\":$FILES_JSON,\"risk_files\":$RISK_JSON,\"status\":\"$STATUS\"}"

        LOG="$(audit_log)"
        echo "$LINE" >> "$LOG"

        echo ""
        echo -e "${YELLOW}================================================================${NC}"
        echo -e "${YELLOW}[AUDIT] --no-verify bypass recorded${NC}"
        echo -e "${YELLOW}   Commit : $COMMIT${NC}"
        echo -e "${YELLOW}   Branch : $BRANCH${NC}"
        echo -e "${YELLOW}   Author : $USER_NAME <$USER_EMAIL>${NC}"
        echo -e "${GRAY}   Trail  : $LOG${NC}"

        if [ "$HIGH_RISK" = "true" ]; then
            echo ""
            echo -e "${RED}[ESCALATION REQUIRED] Production-impacting files bypassed pre-commit:${NC}"
            for f in "${RISK_FILES[@]}"; do echo -e "${RED}     - $f${NC}"; done
            echo ""
            echo -e "${RED}   These changes touch high-risk areas (crates/fuc/, runtime/, stdlib/)${NC}"
            echo -e "${RED}   and require EXPLICIT APPROVAL. The next commit is blocked until an${NC}"
            echo -e "${RED}   owner approves this bypass:${NC}"
            echo ""
            echo -e "${CYAN}     bash scripts/bypass_audit.sh approve $COMMIT \"<reason>\"${NC}"
            echo ""
            echo -e "${RED}================================================================${NC}"
        else
            echo -e "${YELLOW}================================================================${NC}"
        fi
        exit 0
        ;;

    gate)
        LOG="$(audit_log)"
        [ -f "$LOG" ] || exit 0

        # Approved commits (parsed order-independently for cross-engine logs).
        APPROVED="$(grep '"event":"approval"' "$LOG" 2>/dev/null | sed 's/.*"commit":"\([^"]*\)".*/\1/' | sort -u)"

        PENDING=""
        while IFS= read -r line; do
            if printf '%s' "$line" | grep -q '"event":"bypass"' \
               && printf '%s' "$line" | grep -q '"high_risk":true'; then
                c="$(printf '%s' "$line" | sed 's/.*"commit":"\([^"]*\)".*/\1/')"
                if ! printf '%s\n' "$APPROVED" | grep -qx "$c"; then
                    PENDING="$PENDING $c"
                fi
            fi
        done < "$LOG"

        PENDING="$(echo "$PENDING" | xargs -n1 2>/dev/null | sort -u | xargs 2>/dev/null)"
        [ -z "$PENDING" ] && exit 0

        echo ""
        echo -e "${RED}[BLOCKED] Unapproved high-risk --no-verify bypass pending approval${NC}"
        for c in $PENDING; do echo -e "${RED}   Commit : $c${NC}"; done
        echo ""
        echo -e "${YELLOW}   A prior bypass touched production-impacting code and must be${NC}"
        echo -e "${YELLOW}   reviewed and approved before further commits are allowed.${NC}"
        echo -e "${GRAY}   Audit trail: $LOG${NC}"
        echo ""
        echo -e "${YELLOW}   To approve (owner only):${NC}"
        for c in $PENDING; do
            echo -e "${CYAN}     bash scripts/bypass_audit.sh approve $c \"<reason>\"${NC}"
        done
        echo ""
        exit 1
        ;;

    approve)
        COMMIT="${2:-}"
        JUSTIFICATION="${3:-(no justification provided)}"
        if [ -z "$COMMIT" ]; then
            echo -e "${RED}[ERROR] commit SHA is required: approve <commit> [justification]${NC}"
            exit 2
        fi
        LOG="$(audit_log)"
        if [ ! -f "$LOG" ] || ! grep '"event":"bypass"' "$LOG" | grep -q "\"commit\":\"$COMMIT\""; then
            echo -e "${RED}[ERROR] No recorded bypass found for commit '$COMMIT'${NC}"
            exit 2
        fi
        APPROVER="$(git config user.name 2>/dev/null || echo "${USER:-unknown}")"
        TS="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
        LINE="{\"timestamp\":\"$TS\",\"event\":\"approval\",\"commit\":\"$COMMIT\",\"approver\":\"$(json_escape "$APPROVER")\",\"justification\":\"$(json_escape "$JUSTIFICATION")\",\"status\":\"APPROVED\"}"
        echo "$LINE" >> "$LOG"
        echo -e "${GREEN}[OK] Bypass for commit $COMMIT approved by $APPROVER${NC}"
        echo "     Justification: $JUSTIFICATION"
        echo -e "${GRAY}     Audit trail  : $LOG${NC}"
        exit 0
        ;;

    *)
        echo "Usage: bypass_audit.sh {detect|gate|approve <commit> [justification]}" >&2
        exit 2
        ;;
esac
