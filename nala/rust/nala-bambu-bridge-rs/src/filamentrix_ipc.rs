//! AGPLv3-clean Process-Boundary IPC Bridge between NALA-BambuStudio and NALA Filamentrix (BS-06).
//! Localhost-only (127.0.0.1:18766), strict 10 MB body limit, SEC-01..SEC-05 headers, 0% PII.

use std::fs;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::time::Duration;

use crate::sha256::sha256_hex;

const MAX_BODY_BYTES: usize = 10 * 1024 * 1024;

pub fn json_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => {}
            c => out.push(c),
        }
    }
    out
}

/// Dynamically resolves BambuStudio.conf without any hardcoded usernames or paths (Zero-PII).
pub fn resolve_bambu_conf_path() -> Option<PathBuf> {
    if let Ok(home) = std::env::var("HOME") {
        let mac = PathBuf::from(&home).join("Library/Application Support/BambuStudio/BambuStudio.conf");
        if mac.exists() {
            return Some(mac);
        }
        let linux = PathBuf::from(&home).join(".config/BambuStudio/BambuStudio.conf");
        if linux.exists() {
            return Some(linux);
        }
    }
    if let Ok(appdata) = std::env::var("APPDATA") {
        let win = PathBuf::from(appdata).join("BambuStudio/BambuStudio.conf");
        if win.exists() {
            return Some(win);
        }
    }
    None
}

/// Probes whether NALA Filamentrix local API (127.0.0.1:18765) is reachable.
pub fn probe_filamentrix_local(port: u16) -> bool {
    let addr = format!("127.0.0.1:{}", port);
    TcpStream::connect_timeout(
        &addr.parse().unwrap_or_else(|_| "127.0.0.1:18765".parse().unwrap()),
        Duration::from_millis(350),
    )
    .is_ok()
}

/// Parses BambuStudio `[perf]` startup trace log lines and computes total & top bottlenecks.
pub fn analyze_perf_trace_log(path: &Path) -> Result<String, String> {
    let content = fs::read_to_string(path)
        .map_err(|e| format!("Cannot read PerfTrace log {}: {}", path.display(), e))?;
    let mut stages: Vec<(String, u64)> = Vec::new();
    for line in content.lines() {
        if let Some(idx) = line.find("[perf]") {
            let rest = &line[idx + 6..];
            if let Some(cost_idx) = rest.find(" cost ") {
                let name = rest[..cost_idx].trim();
                let after = rest[cost_idx + 6..].trim();
                let ms_str = after.split_whitespace().next().unwrap_or("0");
                if let Ok(ms) = ms_str.parse::<u64>() {
                    stages.push((name.to_string(), ms));
                }
            }
        }
    }
    stages.sort_by(|a, b| b.1.cmp(&a.1));
    let total_ms: u64 = stages.iter().map(|s| s.1).sum();
    let top_json: Vec<String> = stages
        .iter()
        .take(10)
        .map(|(n, ms)| format!(r#"{{"stage":"{}","cost_ms":{}}}"#, json_escape(n), ms))
        .collect();
    Ok(format!(
        r#"{{"ok":true,"stages_parsed":{},"total_measured_ms":{},"top_stages":[{}]}}"#,
        stages.len(),
        total_ms,
        top_json.join(",")
    ))
}

/// Parses a G-code header (exported by headless `bambu-studio --slice`) for time & filament grams.
pub fn parse_gcode_metrics(gcode_head: &str) -> (String, f64, u32) {
    let mut est_time = String::from("unknown");
    let mut grams = 0.0f64;
    let mut layers = 0u32;
    for line in gcode_head.lines().take(250) {
        let l = line.trim();
        if let Some(v) = l.strip_prefix("; estimated printing time (normal mode) =") {
            est_time = v.trim().to_string();
        } else if let Some(v) = l.strip_prefix("; total filament weight [g] :") {
            grams = v
                .split(',')
                .filter_map(|p| p.trim().parse::<f64>().ok())
                .sum();
        } else if let Some(v) = l.strip_prefix("; total layer number:") {
            layers = v.trim().parse::<u32>().unwrap_or(0);
        }
    }
    (est_time, (grams * 100.0).round() / 100.0, layers)
}

/// Computes SHA-256 digest of a Bambu network plugin binary (`libbambu_networking.so`/`.dylib`).
pub fn hash_plugin_file(path: &Path) -> Result<String, String> {
    let bytes = fs::read(path).map_err(|e| format!("Cannot read plugin {}: {}", path.display(), e))?;
    Ok(sha256_hex(&bytes))
}

fn write_http_response(stream: &mut TcpStream, status: &str, body: &str) {
    let resp = format!(
        "HTTP/1.1 {}\r\n\
         Content-Type: application/json; charset=utf-8\r\n\
         Content-Length: {}\r\n\
         X-Content-Type-Options: nosniff\r\n\
         X-Frame-Options: SAMEORIGIN\r\n\
         Referrer-Policy: no-referrer\r\n\
         X-NALA-Version: 1.0.0-bambustudio-phase1\r\n\
         Connection: close\r\n\r\n{}",
        status,
        body.len(),
        body
    );
    let _ = stream.write_all(resp.as_bytes());
}

pub fn serve_local_bridge(bind_port: u16, filamentrix_port: u16) -> Result<(), String> {
    let addr = format!("127.0.0.1:{}", bind_port);
    let listener =
        TcpListener::bind(&addr).map_err(|e| format!("Cannot bind {}: {}", addr, e))?;
    eprintln!(
        "[NALA-BRIDGE-RS] Listening on http://{} (Localhost-Only, AGPLv3-Clean IPC -> Filamentrix :{})",
        addr, filamentrix_port
    );

    for stream_res in listener.incoming() {
        let Ok(mut stream) = stream_res else { continue };
        let _ = stream.set_read_timeout(Some(Duration::from_secs(3)));
        let mut buf = [0u8; 8192];
        let Ok(n) = stream.read(&mut buf) else { continue };
        if n == 0 {
            continue;
        }
        let req = String::from_utf8_lossy(&buf[..n]);
        let first_line = req.lines().next().unwrap_or("");
        let mut parts = first_line.split_whitespace();
        let method = parts.next().unwrap_or("GET");
        let path = parts.next().unwrap_or("/");

        // SEC-04: Reject oversized payloads (>10 MB)
        for line in req.lines() {
            if let Some(v) = line.to_lowercase().strip_prefix("content-length:") {
                if let Ok(cl) = v.trim().parse::<usize>() {
                    if cl > MAX_BODY_BYTES {
                        write_http_response(
                            &mut stream,
                            "413 Payload Too Large",
                            r#"{"ok":false,"error":"SEC-04 body exceeds 10MB"}"#,
                        );
                        continue;
                    }
                }
            }
        }

        match (method, path) {
            ("GET", "/health") => {
                let fil_ok = probe_filamentrix_local(filamentrix_port);
                let conf_found = resolve_bambu_conf_path().is_some();
                let mode = std::env::var("NALA_NETWORK_MODE").unwrap_or_else(|_| "compat".to_string());
                let skin = std::env::var("NALA_SKIN").unwrap_or_else(|_| "nala-obsidian".to_string());
                let body = format!(
                    r#"{{"ok":true,"service":"nala-bambu-bridge-rs","version":"1.0.0","agplv3_ipc_isolated":true,"filamentrix_reachable":{},"bambu_conf_detected":{},"network_mode":"{}","active_skin":"{}"}}"#,
                    fil_ok,
                    conf_found,
                    json_escape(&mode),
                    json_escape(&skin)
                );
                write_http_response(&mut stream, "200 OK", &body);
            }
            ("POST", "/api/gcode-metrics") => {
                let body_part = req.split("\r\n\r\n").nth(1).unwrap_or("");
                let (t, g, l) = parse_gcode_metrics(body_part);
                let resp = format!(
                    r#"{{"ok":true,"estimated_time":"{}","filament_grams":{:.2},"layers":{}}}"#,
                    json_escape(&t),
                    g,
                    l
                );
                write_http_response(&mut stream, "200 OK", &resp);
            }
            _ => {
                write_http_response(
                    &mut stream,
                    "404 Not Found",
                    r#"{"ok":false,"error":"Unknown route","routes":["GET /health","POST /api/gcode-metrics"]}"#,
                );
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gcode_metrics_parser() {
        let sample = "; generated by BambuStudio 02.09.02.62\n; total layer number: 142\n; total filament weight [g] : 42.15, 11.35\n; estimated printing time (normal mode) = 2h 14m 08s\n";
        let (t, g, l) = parse_gcode_metrics(sample);
        assert_eq!(t, "2h 14m 08s");
        assert!((g - 53.50).abs() < 0.001);
        assert_eq!(l, 142);
    }
}
