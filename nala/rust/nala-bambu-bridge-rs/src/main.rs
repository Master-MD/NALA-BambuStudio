#![forbid(unsafe_code)]
//! NALA-BambuStudio Phase 1 — 100% Pure Safe Rust Bridge, sHeriFF Skin-Guard & 5-Language PDF Engine.
//! Edition 2024 | Zero External Crates | Zero Python | Zero PII | AGPLv3-Clean IPC Boundary.

mod filamentrix_ipc;
mod pdf_gen;
mod sha256;
mod skin_guard;

use std::env;
use std::path::PathBuf;
use std::process::ExitCode;

const VERSION: &str = "1.0.0-nala-bambustudio-phase1";

fn print_usage() {
    println!("nala-bambu-bridge-rs v{}", VERSION);
    println!("Usage:");
    println!("  --version                        Print version");
    println!("  --self-check <nala_dir>          Run NCSP v2.5 & sHeriFF integrity self-check");
    println!("  --seal-skins <skins_dir>         Audit & seal *.nala-skin.json with SHA-256");
    println!("  --verify-skins <skins_dir>       Verify skins against skins_manifest.json");
    println!("  --emit-skin-css <skin_file>      Emit sanitized CSS custom properties for WebViews");
    println!("  --build-pdfs <out_dir>           Generate 5-language offline PDFs (DE/FR/IT/RM/EN)");
    println!("  --hash-plugin <plugin_path>      Compute SHA-256 pin for libbambu_networking");
    println!("  --probe-startup-log <log_path>   Parse BambuStudio [perf] startup trace log");
    println!("  --serve [port] [filamentrix_port] Run localhost-only IPC bridge (default 18766 -> 18765)");
}

fn main() -> ExitCode {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        print_usage();
        return ExitCode::SUCCESS;
    }

    match args[1].as_str() {
        "--version" => {
            println!("nala-bambu-bridge-rs {}", VERSION);
            ExitCode::SUCCESS
        }
        "--seal-skins" => {
            let dir = PathBuf::from(args.get(2).map(String::as_str).unwrap_or("nala/skins"));
            match skin_guard::seal_skins_directory(&dir) {
                Ok(msg) => {
                    println!("[OK] {}", msg);
                    ExitCode::SUCCESS
                }
                Err(e) => {
                    eprintln!("[ERR] {}", e);
                    ExitCode::FAILURE
                }
            }
        }
        "--verify-skins" => {
            let dir = PathBuf::from(args.get(2).map(String::as_str).unwrap_or("nala/skins"));
            match skin_guard::verify_skins_directory(&dir) {
                Ok(n) => {
                    println!("[OK] Verified {} SHA-256 sHeriFF skins in {}", n, dir.display());
                    ExitCode::SUCCESS
                }
                Err(e) => {
                    eprintln!("[ERR] {}", e);
                    ExitCode::FAILURE
                }
            }
        }
        "--emit-skin-css" => {
            let Some(file) = args.get(2) else {
                eprintln!("[ERR] Missing <skin_file>");
                return ExitCode::FAILURE;
            };
            match skin_guard::audit_skin_file(&PathBuf::from(file)) {
                Ok(res) => {
                    println!("{}", skin_guard::generate_css_override(&res));
                    ExitCode::SUCCESS
                }
                Err(e) => {
                    eprintln!("[ERR] {}", e);
                    ExitCode::FAILURE
                }
            }
        }
        "--build-pdfs" => {
            let dir = PathBuf::from(args.get(2).map(String::as_str).unwrap_or("nala/docs/pdf"));
            match pdf_gen::generate_all_5_language_pdfs(&dir) {
                Ok(list) => {
                    for item in list {
                        println!("[PDF OK] {}", item);
                    }
                    ExitCode::SUCCESS
                }
                Err(e) => {
                    eprintln!("[ERR] {}", e);
                    ExitCode::FAILURE
                }
            }
        }
        "--hash-plugin" => {
            let Some(file) = args.get(2) else {
                eprintln!("[ERR] Missing <plugin_path>");
                return ExitCode::FAILURE;
            };
            match filamentrix_ipc::hash_plugin_file(&PathBuf::from(file)) {
                Ok(h) => {
                    println!("NALA_NETWORK_PLUGIN_SHA256={}", h);
                    ExitCode::SUCCESS
                }
                Err(e) => {
                    eprintln!("[ERR] {}", e);
                    ExitCode::FAILURE
                }
            }
        }
        "--probe-startup-log" => {
            let Some(file) = args.get(2) else {
                eprintln!("[ERR] Missing <log_path>");
                return ExitCode::FAILURE;
            };
            match filamentrix_ipc::analyze_perf_trace_log(&PathBuf::from(file)) {
                Ok(json) => {
                    println!("{}", json);
                    ExitCode::SUCCESS
                }
                Err(e) => {
                    eprintln!("[ERR] {}", e);
                    ExitCode::FAILURE
                }
            }
        }
        "--serve" => {
            let bind_port = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(18766);
            let fil_port = args.get(3).and_then(|s| s.parse().ok()).unwrap_or(18765);
            if let Err(e) = filamentrix_ipc::serve_local_bridge(bind_port, fil_port) {
                eprintln!("[ERR] {}", e);
                return ExitCode::FAILURE;
            }
            ExitCode::SUCCESS
        }
        "--self-check" => {
            let nala_dir = PathBuf::from(args.get(2).map(String::as_str).unwrap_or("nala"));
            let skins_ok = skin_guard::verify_skins_directory(&nala_dir.join("skins"));
            let pdf_dir = nala_dir.join("docs/pdf");
            let expected_pdfs = [
                "NALA_BambuStudio_Handbuch_DE.pdf",
                "NALA_BambuStudio_Handbuch_FR.pdf",
                "NALA_BambuStudio_Handbuch_IT.pdf",
                "NALA_BambuStudio_Handbuch_RM.pdf",
                "NALA_BambuStudio_Handbuch_EN.pdf",
            ];
            let mut pdf_count = 0usize;
            for f in expected_pdfs {
                let p = pdf_dir.join(f);
                if let Ok(meta) = std::fs::metadata(&p) {
                    if meta.len() > 500 {
                        pdf_count += 1;
                    }
                }
            }
            match skins_ok {
                Ok(n_skins) if pdf_count == 5 => {
                    println!(
                        r#"{{"ok":true,"version":"{}","skins_verified":{},"pdfs_verified":{},"filamentrix_local_online":{},"zero_unsafe":true,"zero_python":true}}"#,
                        VERSION,
                        n_skins,
                        pdf_count,
                        filamentrix_ipc::probe_filamentrix_local(18765)
                    );
                    ExitCode::SUCCESS
                }
                Ok(n_skins) => {
                    eprintln!(
                        "[SELF-CHECK FAIL] skins_verified={} pdfs_verified={}/5",
                        n_skins, pdf_count
                    );
                    ExitCode::FAILURE
                }
                Err(e) => {
                    eprintln!("[SELF-CHECK FAIL] {}", e);
                    ExitCode::FAILURE
                }
            }
        }
        _ => {
            print_usage();
            ExitCode::FAILURE
        }
    }
}
