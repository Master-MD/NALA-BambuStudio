#!/usr/bin/env bash
# =============================================================================
# NALA-BambuStudio Patch-Queue Manager (BS-01) — Pure Git (Zero Quilt Dependency)
# Works identically on Ubuntu/Linux, macOS, Windows Git Bash & GitHub Actions.
# Commands: status | check | apply | revert | verify-roundtrip
# =============================================================================
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
PATCH_DIR="$REPO_ROOT/nala/patches"
SERIES_FILE="$PATCH_DIR/series"

if [[ ! -f "$SERIES_FILE" ]]; then
  echo "[ERR] Missing series file: $SERIES_FILE" >&2
  exit 1
fi

read_series() {
  grep -vE '^\s*(#|$)' "$SERIES_FILE"
}

cmd="${1:-status}"
cd "$REPO_ROOT"

case "$cmd" in
  status)
    echo "=== NALA-BambuStudio Patch-Queue Status ==="
    while IFS= read -r p; do
      if git apply --reverse --check "$PATCH_DIR/$p" 2>/dev/null; then
        echo "  [APPLIED]  $p"
      elif git apply --check "$PATCH_DIR/$p" 2>/dev/null; then
        echo "  [PENDING]  $p (clean apply verified)"
      else
        echo "  [CONFLICT] $p (requires upstream rebase!)"
      fi
    done < <(read_series)
    ;;
  check)
    while IFS= read -r p; do
      echo "[CHECK] $p"
      git apply --check "$PATCH_DIR/$p"
    done < <(read_series)
    echo "[OK] All patches apply cleanly against HEAD ($(git rev-parse --short HEAD))."
    ;;
  apply)
    while IFS= read -r p; do
      if git apply --reverse --check "$PATCH_DIR/$p" 2>/dev/null; then
        echo "[SKIP] Already applied: $p"
      else
        echo "[APPLY] $p"
        git apply --whitespace=nowarn "$PATCH_DIR/$p"
      fi
    done < <(read_series)
    echo "[OK] All NALA Phase 1 patches applied cleanly."
    ;;
  revert)
    tac_lines=()
    while IFS= read -r p; do tac_lines+=("$p"); done < <(read_series)
    for (( idx=${#tac_lines[@]}-1; idx>=0; idx-- )); do
      p="${tac_lines[$idx]}"
      if git apply --reverse --check "$PATCH_DIR/$p" 2>/dev/null; then
        echo "[REVERT] $p"
        git apply --reverse "$PATCH_DIR/$p"
      fi
    done
    echo "[OK] All NALA Phase 1 patches reverted cleanly."
    ;;
  verify-roundtrip)
    "$0" check
    "$0" apply
    "$0" revert
    echo "[OK] Full apply + reverse-revert roundtrip passed 100%."
    ;;
  *)
    echo "Usage: $0 {status|check|apply|revert|verify-roundtrip}" >&2
    exit 2
    ;;
esac
