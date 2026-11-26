#!/usr/bin/env bash
# =============================================================================
# NALA-BambuStudio Local Multi-OS Builder & Verifier (BS-01..BS-06)
# Builds Rust Bridge, verifies sHeriFF Skins + 5 PDFs + Rumantsch .mo,
# applies NALA patch queue, and optionally compiles BambuStudio with ThinLTO.
# Usage: ./nala_build_local.sh [--bridge-only | --full-slicer]
# =============================================================================
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
MODE="${1:---bridge-only}"
OS="$(uname -s)"

cd "$REPO_ROOT"
echo "=== [1/5] NALA Patch-Queue Check ==="
bash nala/scripts/nala_patch_queue.sh check

echo "=== [2/5] Build & Test Pure Safe Rust Bridge (BS-06) ==="
export PATH="$HOME/.cargo/bin:$PATH"
cargo test --manifest-path nala/rust/nala-bambu-bridge-rs/Cargo.toml
cargo build --release --manifest-path nala/rust/nala-bambu-bridge-rs/Cargo.toml
BRIDGE="nala/rust/nala-bambu-bridge-rs/target/release/nala-bambu-bridge-rs"

echo "=== [3/5] Verify sHeriFF Skins (BS-03) & 5-Language PDFs (BS-05) ==="
"$BRIDGE" --verify-skins nala/skins
"$BRIDGE" --build-pdfs nala/docs/pdf
if command -v msgfmt >/dev/null 2>&1; then
  mkdir -p resources/i18n/rm
  msgfmt --check -o resources/i18n/rm/BambuStudio.mo nala/i18n/rm/BambuStudio.po
fi
"$BRIDGE" --self-check nala

if [[ "$MODE" == "--full-slicer" ]]; then
  echo "=== [4/5] Applying NALA Patch-Queue for Full Slicer Build ==="
  bash nala/scripts/nala_patch_queue.sh apply
  if [[ "$OS" == "Linux" ]]; then
    echo "=== [5/5] Building NALA-BambuStudio on Ubuntu/Linux (ThinLTO + AppImage) ==="
    export CFLAGS="${CFLAGS:-} -O3 -flto=auto"
    export CXXFLAGS="${CXXFLAGS:-} -O3 -flto=auto"
    ./BuildLinux.sh -dsi
  elif [[ "$OS" == "Darwin" ]]; then
    echo "=== [5/5] Building NALA-BambuStudio on macOS ($(uname -m), Ninja) ==="
    ./BuildMac.sh -x -d
    ./BuildMac.sh -x -s
  else
    echo "[INFO] Use build_win.bat on Windows after applying nala/scripts/nala_patch_queue.sh apply"
  fi
else
  echo "=== [4/5] Bridge-only verification complete (--full-slicer for full C++ compile) ==="
fi
