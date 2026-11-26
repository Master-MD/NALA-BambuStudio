//! sHeriFF 4-Stage Zero-Trust Skin-Firewall & Pure-Token Manifest Verifier (BS-03).
//! Blocks JS/XSS, CSS breakout, external @import/url(http...), prompt-injections & zero-width steganography.

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use crate::sha256::sha256_hex;

const FORBIDDEN_PATTERNS: &[&str] = &[
    "<script",
    "</style",
    "javascript:",
    "eval(",
    "function(",
    "onload=",
    "onerror=",
    "<iframe",
    "<foreignobject",
    "@import",
    "url(http",
    "url(//",
    "ignore previous instructions",
    "system prompt",
    "curl ",
    "wget ",
    "\u{200b}",
    "\u{200c}",
    "\u{200d}",
    "\u{feff}",
];

const REQUIRED_COLOR_KEYS: &[&str] = &[
    "accent",
    "accent_pressed",
    "accent_hovered",
    "bg_primary",
    "bg_surface",
    "bg_elevated",
    "text_primary",
    "text_muted",
    "border",
    "warning",
    "danger",
];

#[derive(Debug, Clone)]
pub struct SkinAuditResult {
    pub id: String,
    pub file_name: String,
    pub canonical_sha256: String,
    pub tokens: BTreeMap<String, String>,
}

fn is_valid_hex_color(val: &str) -> bool {
    let bytes = val.as_bytes();
    if bytes.len() != 7 || bytes[0] != b'#' {
        return false;
    }
    bytes[1..].iter().all(|b| b.is_ascii_hexdigit())
}

fn parse_flat_json_strings(raw: &str) -> Result<BTreeMap<String, String>, String> {
    let mut map = BTreeMap::new();
    for line in raw.lines() {
        let trimmed = line.trim().trim_end_matches(',');
        if trimmed.is_empty() || trimmed == "{" || trimmed == "}" || trimmed.starts_with("//") {
            continue;
        }
        let Some(colon) = trimmed.find(':') else {
            continue;
        };
        let key = trimmed[..colon].trim();
        let val = trimmed[colon + 1..].trim();
        if !key.starts_with('"') || !key.ends_with('"') {
            continue;
        }
        if !val.starts_with('"') || !val.ends_with('"') {
            continue;
        }
        let k = key[1..key.len() - 1].to_string();
        let v = val[1..val.len() - 1].to_string();
        map.insert(k, v);
    }
    if map.is_empty() {
        return Err("Empty or unparseable Pure-Token skin JSON".to_string());
    }
    Ok(map)
}

pub fn audit_skin_file(path: &Path) -> Result<SkinAuditResult, String> {
    let raw = fs::read_to_string(path)
        .map_err(|e| format!("Cannot read skin {}: {}", path.display(), e))?;
    let lower = raw.to_lowercase();
    for pat in FORBIDDEN_PATTERNS {
        if lower.contains(pat) || raw.contains(pat) {
            return Err(format!(
                "sHeriFF VETO on {}: forbidden pattern detected ({:?})",
                path.display(),
                pat
            ));
        }
    }

    let tokens = parse_flat_json_strings(&raw)?;
    let id = tokens
        .get("id")
        .cloned()
        .ok_or_else(|| format!("Missing 'id' in {}", path.display()))?;
    if !id
        .bytes()
        .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-' || b == b'_')
    {
        return Err(format!("Invalid skin id {:?} (only a-z, 0-9, -, _)", id));
    }

    for &req_key in REQUIRED_COLOR_KEYS {
        let val = tokens
            .get(req_key)
            .ok_or_else(|| format!("Skin '{}' missing required token '{}'", id, req_key))?;
        if !is_valid_hex_color(val) {
            return Err(format!(
                "Skin '{}' token '{}' has invalid #RRGGBB hex value: {:?}",
                id, req_key, val
            ));
        }
    }

    // Canonical hash over sorted key=value pairs
    let mut canonical = String::new();
    for (k, v) in &tokens {
        canonical.push_str(k);
        canonical.push('=');
        canonical.push_str(v);
        canonical.push('\n');
    }
    let canonical_sha256 = sha256_hex(canonical.as_bytes());
    let file_name = path
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "unknown.nala-skin.json".to_string());

    Ok(SkinAuditResult {
        id,
        file_name,
        canonical_sha256,
        tokens,
    })
}

pub fn generate_css_override(skin: &SkinAuditResult) -> String {
    let g = |k: &str| skin.tokens.get(k).map(String::as_str).unwrap_or("#10B981");
    format!(
        ":root {{\n  --nala-accent: {acc};\n  --nala-accent-pressed: {acc_p};\n  --nala-accent-hovered: {acc_h};\n  --nala-bg: {bg};\n  --nala-surface: {surf};\n  --nala-elevated: {elev};\n  --nala-text: {txt};\n  --nala-muted: {mut_t};\n  --nala-border: {brd};\n  --brand-green: {acc};\n  --primary-color: {acc};\n}}\nhtml, body {{ background-color: {bg} !important; color: {txt} !important; }}\nbutton.primary, .bbl-btn-primary, .btn-green {{ background-color: {acc} !important; border-color: {acc} !important; }}\n",
        acc = g("accent"),
        acc_p = g("accent_pressed"),
        acc_h = g("accent_hovered"),
        bg = g("bg_primary"),
        surf = g("bg_surface"),
        elev = g("bg_elevated"),
        txt = g("text_primary"),
        mut_t = g("text_muted"),
        brd = g("border"),
    )
}

pub fn seal_skins_directory(skins_dir: &Path) -> Result<String, String> {
    let mut results = Vec::new();
    let entries = fs::read_dir(skins_dir)
        .map_err(|e| format!("Cannot read skins dir {}: {}", skins_dir.display(), e))?;
    for ent in entries.flatten() {
        let p = ent.path();
        if p.file_name()
            .and_then(|n| n.to_str())
            .map(|n| n.ends_with(".nala-skin.json"))
            .unwrap_or(false)
        {
            results.push(audit_skin_file(&p)?);
        }
    }
    results.sort_by(|a, b| a.id.cmp(&b.id));
    if results.is_empty() {
        return Err("No *.nala-skin.json files found".to_string());
    }

    let mut manifest = String::from("{\n  \"schema\": \"nala-sheriff-skin-manifest-v1\",\n  \"skins\": [\n");
    for (idx, r) in results.iter().enumerate() {
        let comma = if idx + 1 < results.len() { "," } else { "" };
        let name = r.tokens.get("name").map(String::as_str).unwrap_or(&r.id);
        manifest.push_str(&format!(
            "    {{\"id\": \"{}\", \"name\": \"{}\", \"file\": \"{}\", \"canonical_sha256\": \"{}\", \"accent\": \"{}\"}}{}\n",
            r.id,
            name,
            r.file_name,
            r.canonical_sha256,
            r.tokens.get("accent").map(String::as_str).unwrap_or("#10B981"),
            comma
        ));
    }
    manifest.push_str("  ]\n}\n");
    fs::write(skins_dir.join("skins_manifest.json"), &manifest)
        .map_err(|e| format!("Cannot write skins_manifest.json: {}", e))?;
    Ok(format!("Sealed {} skins with FIPS-180-4 SHA-256", results.len()))
}

pub fn verify_skins_directory(skins_dir: &Path) -> Result<usize, String> {
    let manifest_raw = fs::read_to_string(skins_dir.join("skins_manifest.json"))
        .map_err(|e| format!("Missing skins_manifest.json: {}", e))?;
    let mut verified = 0usize;
    let entries = fs::read_dir(skins_dir)
        .map_err(|e| format!("Cannot read skins dir {}: {}", skins_dir.display(), e))?;
    for ent in entries.flatten() {
        let p = ent.path();
        if p.file_name()
            .and_then(|n| n.to_str())
            .map(|n| n.ends_with(".nala-skin.json"))
            .unwrap_or(false)
        {
            let res = audit_skin_file(&p)?;
            let needle = format!("\"canonical_sha256\": \"{}\"", res.canonical_sha256);
            if !manifest_raw.contains(&needle) {
                return Err(format!(
                    "HASH_MISMATCH for skin '{}' ({}): expected digest not found in skins_manifest.json",
                    res.id, res.canonical_sha256
                ));
            }
            verified += 1;
        }
    }
    if verified == 0 {
        return Err("No skins verified".to_string());
    }
    Ok(verified)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sheriff_blocks_xss_and_prompt_injection() {
        let tmp = std::env::temp_dir().join("nala_test_bad_skin.nala-skin.json");
        let _ = fs::write(&tmp, r#"{"id":"evil","accent":"<script>alert(1)</script>"}"#);
        let err = audit_skin_file(&tmp).unwrap_err();
        let _ = fs::remove_file(&tmp);
        assert!(err.contains("sHeriFF VETO"));
    }
}
