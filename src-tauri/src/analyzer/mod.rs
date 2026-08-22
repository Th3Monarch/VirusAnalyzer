//! Análisis estático de archivos (FASE 3).
//!
//! Detecta el tipo por magic bytes, calcula la entropía y, cuando el archivo
//! es un PE de Windows, extrae cabeceras, secciones, imports y exports.
//!
//! Seguridad: todo el análisis es estático; nunca se ejecuta el contenido.

pub mod keywords;
pub mod pe;

use std::fs::File;
use std::io::{BufReader, Read, Seek, SeekFrom};
use std::path::Path;

use crate::models::StaticAnalysis;

const HEAD_BYTES: usize = 8192;
const CHUNK_SIZE: usize = 1024 * 1024;

/// Ejecuta el análisis estático completo de un archivo.
pub fn analyze(path: &Path) -> Result<StaticAnalysis, String> {
    let mut file = File::open(path).map_err(|e| {
        format!(
            "No se pudo abrir {} para análisis estático: {e}",
            path.display()
        )
    })?;

    // 1. Magic bytes (solo el inicio del archivo).
    let mut head = vec![0u8; HEAD_BYTES];
    let n = file
        .read(&mut head)
        .map_err(|e| format!("Error de lectura: {e}"))?;
    head.truncate(n);

    let detected = infer::get(&head);
    let (file_type, file_type_extension, file_type_mime) = match detected {
        Some(t) => (
            t.to_string(),
            t.extension().to_string(),
            t.mime_type().to_string(),
        ),
        None => detect_by_extension(path),
    };

    // 2. ¿Es un PE? (cabecera DOS + firma NT dentro de los primeros bytes).
    let is_pe = pe::looks_like_pe(&head);

    // 3. Entropía global (streaming, no carga el archivo completo).
    file.seek(SeekFrom::Start(0))
        .map_err(|e| format!("Error al reposicionar el archivo: {e}"))?;
    let entropy = stream_entropy(&mut file)?;
    drop(file);

    // 4. Estructura PE + palabras clave sospechosas.
    let (pe_info, keywords) = if is_pe {
        let bytes = std::fs::read(path)
            .map_err(|e| format!("No se pudo leer el archivo para el análisis PE: {e}"))?;
        let kw = keywords::scan(&bytes);
        (pe::parse(&bytes), kw)
    } else {
        let kw = keywords::scan_prefix(path).unwrap_or_default();
        (None, kw)
    };

    let type_mismatch = detect_type_mismatch(path, &file_type_extension, detected.is_some());

    Ok(StaticAnalysis {
        file_type,
        file_type_extension,
        file_type_mime,
        entropy,
        is_pe,
        keywords,
        type_mismatch,
        pe: pe_info,
    })
}

/// Indica si el tipo detectado por magic bytes contradice la extensión del
/// archivo. Se ignoran alias legítimos (jpg/jpeg, docx/zip, exe/dll…).
fn detect_type_mismatch(path: &Path, detected_ext: &str, detected_by_magic: bool) -> bool {
    if !detected_by_magic {
        return false;
    }
    let actual = path
        .extension()
        .map(|e| e.to_string_lossy().to_lowercase())
        .unwrap_or_default();
    if actual.is_empty() {
        return false;
    }
    let detected = detected_ext.to_lowercase();
    if actual == detected {
        return false;
    }
    !compatible_extension(&detected, &actual)
}

fn compatible_extension(detected: &str, actual: &str) -> bool {
    let alias: &[(&str, &str)] = &[
        ("jpg", "jpeg"),
        ("tif", "tiff"),
        ("htm", "html"),
        ("m4a", "mp4"),
    ];
    for (a, b) in alias {
        if (detected == *a && actual == *b) || (detected == *b && actual == *a) {
            return true;
        }
    }
    // Contenedores ZIP usados por documentos y paquetes.
    if detected == "zip" {
        return matches!(
            actual,
            "zip"
                | "docx"
                | "xlsx"
                | "pptx"
                | "pptm"
                | "docm"
                | "odt"
                | "ods"
                | "odp"
                | "jar"
                | "apk"
                | "xpi"
                | "vsix"
        );
    }
    // Cualquier PE se detecta como "exe"; cubre las extensiones habituales.
    if detected == "exe" {
        return matches!(
            actual,
            "exe" | "dll" | "sys" | "scr" | "ocx" | "cpl" | "com" | "drv" | "efi"
        );
    }
    if detected == "json" {
        return matches!(actual, "json" | "geojson" | "map");
    }
    if detected == "xml" {
        return matches!(actual, "xml" | "svg" | "rss" | "xsd" | "xsl");
    }
    if detected == "mp4" {
        return matches!(actual, "mp4" | "m4v" | "mov");
    }
    false
}

/// Entropía de Shannon (bits/byte) leída por tramos.
fn stream_entropy(file: &mut File) -> Result<f64, String> {
    let mut counts = [0u64; 256];
    let mut total: u64 = 0;
    let mut buf = vec![0u8; CHUNK_SIZE];
    let mut reader = BufReader::with_capacity(CHUNK_SIZE, file);

    loop {
        let r = reader
            .read(&mut buf)
            .map_err(|e| format!("Error de lectura para entropía: {e}"))?;
        if r == 0 {
            break;
        }
        for &b in &buf[..r] {
            counts[b as usize] += 1;
        }
        total += r as u64;
    }

    Ok(entropy_from_counts(&counts, total))
}

fn entropy_from_counts(counts: &[u64; 256], total: u64) -> f64 {
    if total == 0 {
        return 0.0;
    }
    let mut h = 0.0f64;
    for &c in counts {
        if c == 0 {
            continue;
        }
        let p = c as f64 / total as f64;
        h -= p * p.log2();
    }
    h
}

/// Entropía de Shannon de un buffer en memoria.
pub fn entropy(data: &[u8]) -> f64 {
    let mut counts = [0u64; 256];
    for &b in data {
        counts[b as usize] += 1;
    }
    entropy_from_counts(&counts, data.len() as u64)
}

/// Fallback por extensión cuando los magic bytes no identifican el tipo.
fn detect_by_extension(path: &Path) -> (String, String, String) {
    let ext = path
        .extension()
        .map(|e| e.to_string_lossy().to_lowercase())
        .unwrap_or_default();

    let known: &[(&str, &str, &str, &str)] = &[
        ("js", "JavaScript", "js", "text/javascript"),
        ("vbs", "VBScript", "vbs", "text/vbscript"),
        ("vbe", "VBScript Encoded", "vbe", "text/vbscript"),
        ("ps1", "PowerShell Script", "ps1", "text/plain"),
        ("psm1", "PowerShell Module", "psm1", "text/plain"),
        ("cmd", "Windows Command Script", "cmd", "text/plain"),
        ("bat", "Batch File", "bat", "text/plain"),
        ("hta", "HTML Application", "hta", "text/html"),
        ("reg", "Windows Registry", "reg", "text/plain"),
        ("msi", "Windows Installer", "msi", "application/x-msi"),
        ("jar", "Java Archive", "jar", "application/java-archive"),
        ("py", "Python Script", "py", "text/x-python"),
        ("pl", "Perl Script", "pl", "text/plain"),
        ("sh", "Shell Script", "sh", "application/x-sh"),
        ("csv", "CSV Data", "csv", "text/csv"),
        ("log", "Log File", "log", "text/plain"),
    ];

    for (e, name, ext_out, mime) in known {
        if ext == *e {
            return ((*name).into(), (*ext_out).into(), (*mime).into());
        }
    }

    (
        format!("{ext} file"),
        ext.clone(),
        "application/octet-stream".into(),
    )
}

#[cfg(test)]
mod tests {
    /// El propio binario de test es un PE real: sirve de validación
    /// end-to-end del parser (cabeceras, secciones, imports, entropía).
    #[test]
    #[cfg(target_os = "windows")]
    fn analyzes_own_binary_as_pe() {
        let exe = std::env::current_exe().expect("current exe");
        let analysis = super::analyze(&exe).expect("análisis estático");

        assert!(analysis.is_pe, "el binario debería detectarse como PE");
        assert!(
            (0.0..=8.0).contains(&analysis.entropy),
            "entropía fuera de rango"
        );

        let pe = analysis.pe.expect("detalle PE");
        assert!(!pe.sections.is_empty(), "debería haber secciones");
        assert!(!pe.imports.is_empty(), "debería haber imports");
        assert!(
            pe.sections.iter().any(|s| !s.flags.is_empty()),
            "secciones con flags"
        );

        let import_count = pe.imports.iter().map(|d| d.functions.len()).sum::<usize>();
        assert!(import_count > 0, "debería haber funciones importadas");
        let has_system_dll = pe.imports.iter().any(|d| {
            d.name.to_lowercase().contains("kernel32") || d.name.to_lowercase().contains("ucrtbase")
        });
        assert!(has_system_dll, "debería importar DLLs del sistema");
    }
}
