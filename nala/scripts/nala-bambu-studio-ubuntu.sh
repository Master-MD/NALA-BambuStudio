#!/usr/bin/env bash
# =============================================================================
# NALA-BambuStudio Ubuntu Intel-GPU & Startup Turbo Launcher (BS-02 / BS-03 / BS-04)
# Usage: ./nala-bambu-studio-ubuntu.sh [BambuStudio args...]
# Env overrides:
#   NALA_SKIN=nala-obsidian|porsche-v4|doom-uac|matrix-construct|bambu-classic
#   NALA_NETWORK_MODE=compat|lan_only
#   NALA_NETWORK_PLUGIN_SHA256=<64-hex-sha256>
#   NALA_GL_MAX_MSAA=4   NALA_GL_MAX_TEX=4096
# =============================================================================
set -euo pipefail

export NALA_SKIN="${NALA_SKIN:-nala-obsidian}"
export NALA_NETWORK_MODE="${NALA_NETWORK_MODE:-compat}"
export NALA_GL_MAX_MSAA="${NALA_GL_MAX_MSAA:-4}"
export NALA_GL_MAX_TEX="${NALA_GL_MAX_TEX:-4096}"
export MESA_SHADER_CACHE_MAX_SIZE="${MESA_SHADER_CACHE_MAX_SIZE:-512M}"

# 1. Runtime Intel GPU / Wayland WebKitGTK DMA-BUF Guard
INTEL_GPU=0
for f in /sys/class/drm/card*/device/vendor; do
  if [[ -r "$f" ]] && grep -qi "0x8086" "$f" 2>/dev/null; then
    INTEL_GPU=1
    break
  fi
done

if [[ "$INTEL_GPU" -eq 1 || -n "${WAYLAND_DISPLAY:-}" ]]; then
  export WEBKIT_DISABLE_DMABUF_RENDERER="${WEBKIT_DISABLE_DMABUF_RENDERER:-1}"
  if [[ "$INTEL_GPU" -eq 1 ]]; then
    export WEBKIT_DISABLE_COMPOSITING_MODE="${WEBKIT_DISABLE_COMPOSITING_MODE:-1}"
  fi
  if [[ -n "${WAYLAND_DISPLAY:-}" && "${NALA_FORCE_WAYLAND:-0}" != "1" ]]; then
    export GDK_BACKEND="${GDK_BACKEND:-x11}"
  fi
fi

# 2. Fast allocator auto-detection (mimalloc -> jemalloc) for faster preset JSON & slicing allocs
if [[ -z "${LD_PRELOAD:-}" ]]; then
  for alloc in \
    /usr/lib/x86_64-linux-gnu/libmimalloc.so.2 \
    /usr/lib/aarch64-linux-gnu/libmimalloc.so.2 \
    /usr/lib/x86_64-linux-gnu/libjemalloc.so.2 \
    /usr/lib/aarch64-linux-gnu/libjemalloc.so.2; do
    if [[ -r "$alloc" ]]; then
      export LD_PRELOAD="$alloc"
      break
    fi
  done
fi

# 3. Resolve BambuStudio executable
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
CANDIDATES=(
  "${NALA_BAMBU_BIN:-}"
  "$REPO_ROOT/build/src/bambu-studio"
  "/usr/local/bin/bambu-studio"
  "/usr/bin/bambu-studio"
)
BIN=""
for c in "${CANDIDATES[@]}"; do
  if [[ -n "$c" && -x "$c" ]]; then
    BIN="$c"
    break
  fi
done

if [[ -z "$BIN" ]]; then
  echo "[NALA-TURBO] BambuStudio binary not found. Build first via nala/scripts/nala_build_local.sh or set NALA_BAMBU_BIN." >&2
  exit 1
fi

echo "[NALA-TURBO] Launching $BIN | skin=$NALA_SKIN | net=$NALA_NETWORK_MODE | intel_gpu=$INTEL_GPU | preload=${LD_PRELOAD:-none}"
exec "$BIN" "$@"
