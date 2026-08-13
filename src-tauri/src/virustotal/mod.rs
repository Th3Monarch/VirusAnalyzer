//! Integración opcional con VirusTotal (FASE 5).
//!
//! Consulta la reputación de un archivo por su **hash** (MD5, SHA-1 o
//! SHA-256). Nunca sube archivos: los únicos datos que llegan a VirusTotal
//! son el hash y la clave de API en la cabecera `x-apikey`.
//!
//! Seguridad:
//! - Solo se consulta cuando el usuario habilita la integración en Ajustes
//!   (`virustotal_enabled`) y existe una API key.
//! - La clave nunca se registra ni se expone fuera de la app.
//! - El resultado de VirusTotal es evidencia adicional, no un veredicto único.

use std::time::Duration;

use chrono::DateTime;
use serde_json::Value;

use crate::models::{VirusTotalResult, VtVendorResult};

const API_URL: &str = "https://www.virustotal.com/api/v3/files";
const REQUEST_TIMEOUT: Duration = Duration::from_secs(20);

/// Consulta la reputación de un hash en VirusTotal.
pub fn lookup(api_key: &str, hash: &str) -> Result<VirusTotalResult, String> {
    let url = format!("{API_URL}/{hash}");
    let response = ureq::get(&url)
        .set("x-apikey", api_key)
        .set("User-Agent", "VirusAnalyzer/2.0")
        .timeout(REQUEST_TIMEOUT)
        .call();

    let body = match response {
        Ok(resp) => resp
            .into_string()
            .map_err(|e| format!("Respuesta ilegible de VirusTotal: {e}"))?,
        Err(ureq::Error::Status(code, resp)) => {
            let body = resp.into_string().unwrap_or_default();
            return handle_error(code, &body, hash);
        }
        Err(e) => return Err(format!("Error de red al consultar VirusTotal: {e}")),
    };

    let json: Value = serde_json::from_str(&body)
        .map_err(|e| format!("Respuesta JSON inválida de VirusTotal: {e}"))?;
    parse_ok(&json, hash)
}

fn handle_error(code: u16, _body: &str, hash: &str) -> Result<VirusTotalResult, String> {
    match code {
        404 => Ok(VirusTotalResult {
            available: false,
            hash: hash.to_string(),
            ..Default::default()
        }),
        401 => Err("Clave de API de VirusTotal inválida (401)".into()),
        403 => Err("Acceso denegado por VirusTotal (403)".into()),
        429 => Err("Límite de consultas de VirusTotal alcanzado (429)".into()),
        _ => Err(format!("VirusTotal respondió con estado {code}")),
    }
}

fn parse_ok(json: &Value, hash: &str) -> Result<VirusTotalResult, String> {
    let data = json
        .get("data")
        .and_then(|d| d.as_object())
        .ok_or("VirusTotal no devolvió 'data' en la respuesta")?;
    let attrs = data
        .get("attributes")
        .and_then(|a| a.as_object())
        .ok_or("VirusTotal no devolvió 'attributes'")?;

    let mut result = VirusTotalResult {
        available: true,
        hash: hash.to_string(),
        ..Default::default()
    };

    if let Some(stats) = attrs.get("last_analysis_stats").and_then(|s| s.as_object()) {
        result.malicious = num(stats.get("malicious"));
        result.suspicious = num(stats.get("suspicious"));
        result.harmless = num(stats.get("harmless"));
        result.undetected = num(stats.get("undetected"));
        result.timeout = num(stats.get("timeout"));
        result.type_unsupported = num(stats.get("type-unsupported"));
    }
    result.total = result.malicious
        + result.suspicious
        + result.harmless
        + result.undetected
        + result.timeout
        + result.type_unsupported;

    result.reputation = attrs.get("reputation").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
    result.times_submitted = attrs.get("times_submitted").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
    result.first_submission_iso = epoch_iso(attrs.get("first_submission_date"));
    result.last_analysis_iso = epoch_iso(attrs.get("last_analysis_date"));
    result.meaningful_name = attrs.get("meaningful_name").and_then(|v| v.as_str()).map(String::from);
    result.magic = attrs.get("magic").and_then(|v| v.as_str()).map(String::from);
    result.size = attrs.get("size").and_then(|v| v.as_u64());

    if let Some(engine_results) = attrs.get("last_analysis_results").and_then(|r| r.as_object()) {
        for (engine, entry) in engine_results {
            let Some(obj) = entry.as_object() else { continue; };
            let category = obj.get("category").and_then(|c| c.as_str()).unwrap_or("").to_string();
            let res = obj.get("result").and_then(|r| r.as_str()).map(String::from);
            if matches!(category.as_str(), "malicious" | "suspicious") {
                if let Some(r) = &res {
                    if !r.is_empty() && !result.threat_names.contains(r) {
                        result.threat_names.push(r.clone());
                    }
                }
            }
            result.vendors.push(VtVendorResult {
                engine: engine.clone(),
                category,
                result: res,
            });
        }
        // Motores que detectan algo primero (malicioso > sospechoso).
        result.vendors.sort_by_key(|v| category_rank(&v.category));
    }

    result.permalink = format!("https://www.virustotal.com/gui/file/{hash}/detection");
    Ok(result)
}

fn category_rank(category: &str) -> u8 {
    match category {
        "malicious" => 0,
        "suspicious" => 1,
        _ => 2,
    }
}

fn num(v: Option<&Value>) -> u32 {
    v.and_then(|x| x.as_u64()).unwrap_or(0) as u32
}

fn epoch_iso(v: Option<&Value>) -> Option<String> {
    let ts = v.and_then(|x| x.as_i64())?;
    DateTime::from_timestamp(ts, 0).map(|d| d.to_rfc3339())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_analysis_response() {
        let json: Value = serde_json::json!({
            "data": {
                "attributes": {
                    "last_analysis_stats": {
                        "harmless": 60, "malicious": 5, "suspicious": 1,
                        "undetected": 2, "timeout": 0, "type-unsupported": 0
                    },
                    "reputation": -10,
                    "times_submitted": 3,
                    "first_submission_date": 1700000000,
                    "last_analysis_date": 1705000000,
                    "meaningful_name": "evil.exe",
                    "magic": "PE32 executable",
                    "size": 123456,
                    "last_analysis_results": {
                        "AV-Example": {"category": "malicious", "result": "Trojan.Generic", "engine_name": "AV-Example"},
                        "SuspEngine": {"category": "suspicious", "result": "Heuristic.Susp", "engine_name": "SuspEngine"},
                        "BenignEngine": {"category": "undetected", "result": null, "engine_name": "BenignEngine"}
                    }
                }
            }
        });
        let r = parse_ok(&json, "abc").expect("parse");
        assert_eq!(r.malicious, 5);
        assert_eq!(r.suspicious, 1);
        assert_eq!(r.total, 68);
        assert_eq!(r.reputation, -10);
        assert!(r.threat_names.contains(&"Trojan.Generic".to_string()));
        assert_eq!(r.vendors.len(), 3);
        assert_eq!(r.vendors[0].engine, "AV-Example");
        assert_eq!(r.vendors[1].engine, "SuspEngine");
        assert_eq!(r.permalink, "https://www.virustotal.com/gui/file/abc/detection");
    }

    #[test]
    fn not_found_is_available_false() {
        let r = handle_error(404, "", "abc").expect("not found");
        assert!(!r.available);
        assert!(r.error.is_none());
        assert_eq!(r.hash, "abc");
    }

    #[test]
    fn quota_error_is_error() {
        let r = handle_error(429, "", "abc");
        assert!(r.is_err());
    }
}
