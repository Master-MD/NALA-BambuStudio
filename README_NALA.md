# NALA-BambuStudio — Phase 1 Complete (`BS-01` – `BS-06`)

> **Fork von:** [`bambulab/BambuStudio`](https://github.com/bambulab/BambuStudio) (`v02.09.02.62`, **AGPLv3**)  
> **Standard:** `TheStandart` · `NCSP v2.5` · `sHeriFF` Zero-PII & Skin-Guard · `100 %` Bambu-Drucker-Parität (`A1`, `P1`, `X1`, `H2D`, `H3C`, `AMS`)

---

## Was Phase 1 liefert (`0 %` Feature-Verlust, `0` Upstream-Merge-Konflikte)

| Paket | Modul | Beschreibung |
| :--- | :--- | :--- |
| **`BS-01`** | **Upstream Auto-Sync & Multi-OS CI** | [`.github/workflows/nala-upstream-sync-and-build.yml`](.github/workflows/nala-upstream-sync-and-build.yml) + [`nala/scripts/nala_patch_queue.sh`](nala/scripts/nala_patch_queue.sh) überwachen täglich `bambulab/BambuStudio`, halten `master` konfliktfrei synchron, wenden die 4 chirurgischen C++-Patches auf `nala-optimized` an und bauen Ubuntu, macOS (ARM64/Intel) & Windows automatisch inkl. SHA-256 & Non-Empty-Binary-Gate. |
| **`BS-02`** | **Ubuntu Intel-GPU & Startup Turbo** | [`nala/patches/0001-bs02-ubuntu-intel-gpu-webkit-and-startup-turbo.patch`](nala/patches/0001-bs02-ubuntu-intel-gpu-webkit-and-startup-turbo.patch) + [`nala/scripts/nala-bambu-studio-ubuntu.sh`](nala/scripts/nala-bambu-studio-ubuntu.sh): Automatische Intel-Mesa-Erkennung (`0x8086`), setzt `WEBKIT_DISABLE_DMABUF_RENDERER=1` vor `wxEntry()`, begrenzt Intel-MSAA auf `X4` & Texturen auf `4096px` und lädt `mimalloc`/`jemalloc` automatisch vor. |
| **`BS-03`** | **Merge-Safe `sHeriFF` Skin-Engine (5 Skins)** | [`nala/patches/0002-bs03-merge-safe-sheriff-skin-engine.patch`](nala/patches/0002-bs03-merge-safe-sheriff-skin-engine.patch) + [`nala/skins/`](nala/skins/): Zentraler Runtime-Hook in `StateColor.cpp` + CSS-Token-Injection in `WebView.cpp` (0 Edits an 299 `#00AE42`-Stellen!). 5 FIPS-180-4 SHA-256 versiegelte Skins: `nala-obsidian`, `porsche-v4`, `doom-uac`, `matrix-construct`, `bambu-classic`. |
| **`BS-04`** | **`sHeriFF` Dual-Mode Privacy & Security Guard** | [`nala/patches/0003-bs04-sheriff-dual-mode-privacy-and-plugin-guard.patch`](nala/patches/0003-bs04-sheriff-dual-mode-privacy-and-plugin-guard.patch): `NALA_NETWORK_MODE=compat` (Standard — 100 % Cloud/Bambu Connect/H2D/H3C Parität + optionales in-process SHA-256 Pinning via `NALA_NETWORK_PLUGIN_SHA256`) ↔ `NALA_NETWORK_MODE=lan_only` (0 % Cloud-OSS, 0 % Telemetrie). |
| **`BS-05`** | **5-Sprachen-Suite (`DE/FR/IT/RM/EN`) + Offline-PDFs** | [`nala/patches/0004-bs05-romansh-5lang-i18n.patch`](nala/patches/0004-bs05-romansh-5lang-i18n.patch) + [`resources/i18n/rm/BambuStudio.mo`](resources/i18n/rm/BambuStudio.mo) (Rumantsch Grischun!) + 5 echte Offline-PDF-Handbücher unter [`nala/docs/pdf/`](nala/docs/pdf/) + Offline In-App Quickstart [`nala/docs/inapp/quickstart.html`](nala/docs/inapp/quickstart.html). |
| **`BS-06`** | **100 % Pure Safe Rust Filamentrix Bridge** | [`nala/rust/nala-bambu-bridge-rs/`](nala/rust/nala-bambu-bridge-rs/) (Edition 2024, `#![forbid(unsafe_code)]`, 0 Crates, 0 % Python): Localhost-Only IPC (`127.0.0.1:18766` → `NALA Filamentrix :18765`, AGPLv3-saubere Prozess-Isolation), `sHeriFF` Skin-Firewall, Plugin-SHA-256-Pinning, `PerfTrace`-Analyzer & 5-Sprachen-PDF-Generator. |

---

## Schnellstart

```bash
# 1. NALA Bridge bauen, 5 Skins & 5 PDFs + Patch-Queue verifizieren:
bash nala/scripts/nala_build_local.sh --bridge-only

# 2. Vollen optimierten Slicer bauen (Ubuntu oder macOS):
bash nala/scripts/nala_build_local.sh --full-slicer

# 3. Unter Ubuntu mit Intel-GPU-Turbo & Porsche v4 / NALA Obsidian Skin starten:
NALA_SKIN=nala-obsidian NALA_NETWORK_MODE=compat bash nala/scripts/nala-bambu-studio-ubuntu.sh

# 4. Pure-Rust Bridge zu NALA Filamentrix (:18765) starten:
./nala/rust/nala-bambu-bridge-rs/target/release/nala-bambu-bridge-rs --serve 18766 18765
```

## Lizenz & AGPLv3-Isolation
`NALA-BambuStudio` und alle Patches unter `nala/patches/` sowie `nala-bambu-bridge-rs` stehen sauber unter **GNU AGPLv3** (entsprechend Upstream `bambulab/BambuStudio`). Die Kommunikation mit `NALA Filamentrix` erfolgt ausschliesslich über lokale Prozess-/IPC-Grenzen (`127.0.0.1:18766` / Headless `--slice` / `.3mf`).
