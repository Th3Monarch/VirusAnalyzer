//! Detección de palabras clave sospechosas en una muestra del archivo.
//!
//! Solo escanea el contenido como texto (bytes imprimibles); nunca lo ejecuta.
//! Las coincidencias alimentan la evidencia del motor de reglas (FASE 4).

use std::fs::File;
use std::io::Read;
use std::path::Path;

const PREFIX_BYTES: usize = 2 * 1024 * 1024;

const SUSPICIOUS_KEYWORDS: &[&str] = &[
    "powershell",
    "rundll32",
    "certutil",
    "mshta",
    "bitsadmin",
    "schtasks",
    "wscript",
    "cscript",
    "jscript",
    "vbscript",
    "hta",
    "base64",
    "downloadstring",
    "downloadfile",
    "invoke-webrequest",
    "invoke-expression",
    "encodedcommand",
    "eicar",
    "writeprocessmemory",
    "createremotethread",
    "getasynckeystate",
    "sethooks",
    "regsvr32",
    "reg add",
    "whoami",
    "cmd.exe",
    "wmic",
    "http://",
    "https://",
];

/// Busca las palabras clave presentes (deduplicadas, en minúsculas).
pub fn scan(data: &[u8]) -> Vec<String> {
    let hay = String::from_utf8_lossy(data).to_lowercase();
    let mut found: Vec<String> = SUSPICIOUS_KEYWORDS
        .iter()
        .filter(|kw| hay.contains(*kw))
        .map(|kw| kw.to_string())
        .collect();
    found.sort();
    found
}

/// Lee el prefijo del archivo y escanea las palabras clave.
pub fn scan_prefix(path: &Path) -> Result<Vec<String>, String> {
    let mut file =
        File::open(path).map_err(|e| format!("No se pudo abrir {}: {e}", path.display()))?;
    let mut buf = vec![0u8; PREFIX_BYTES];
    let n = file
        .read(&mut buf)
        .map_err(|e| format!("Error de lectura para palabras clave: {e}"))?;
    buf.truncate(n);
    Ok(scan(&buf))
}
