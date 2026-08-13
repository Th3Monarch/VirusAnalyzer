//! Motor de escaneo de archivos y carpetas.
//!
//! FASE 2: selección de rutas, lectura segura (solo bytes), hashes y
//! resultados básicos. El análisis heurístico llega en FASE 4.
//!
//! El escaneo es **estático**: nunca se ejecuta el contenido analizado.

use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use chrono::Local;
use uuid::Uuid;

use crate::config::ScanPreferences;
use crate::hashing;
use crate::models::{
    FileHashes, FolderFileEntry, FolderScanResult, Language, PathInfo, ScanKind, ScanResult,
    ThreatLevel, TimelineEntry, VirusTotalResult,
};

pub mod history;

/// Marca de tiempo ISO (para `scanned_at`).
pub fn now_iso() -> String {
    Local::now().to_rfc3339()
}

/// Marca de tiempo HH:MM:SS para la línea temporal.
pub fn time_label() -> String {
    Local::now().format("%H:%M:%S").to_string()
}

/// Contexto compartido durante un escaneo.
pub struct ScanContext {
    pub cancel: Arc<AtomicBool>,
    pub preferences: ScanPreferences,
    /// API key de VirusTotal (solo presente si el usuario la habilitó).
    pub virustotal_api_key: Option<String>,
    /// Idioma en el que se genera el contenido de la evaluación.
    pub language: Language,
}

impl ScanContext {
    pub fn is_cancelled(&self) -> bool {
        self.cancel.load(Ordering::Relaxed)
    }
}

/// Inspecciona una ruta para preparar el escaneo desde la UI.
pub fn path_info(path: &Path) -> Result<PathInfo, String> {
    let metadata = std::fs::metadata(path)
        .map_err(|e| format!("No se pudo acceder a {}: {e}", path.display()))?;
    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| path.display().to_string());
    Ok(PathInfo {
        path: path.to_string_lossy().into_owned(),
        name,
        is_dir: metadata.is_dir(),
        size: if metadata.is_dir() { 0 } else { metadata.len() },
    })
}

fn name_of(path: &Path) -> String {
    path.file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| path.display().to_string())
}

/// Escanea un archivo individual: hashes + línea temporal.
pub fn file_scan(ctx: &ScanContext, path: &Path) -> Result<ScanResult, String> {
    if ctx.is_cancelled() {
        return Err("Análisis cancelado".into());
    }

    let metadata = std::fs::metadata(path)
        .map_err(|e| format!("No se pudo acceder a {}: {e}", path.display()))?;
    if !metadata.is_file() {
        return Err(format!("La ruta no es un archivo: {}", path.display()));
    }

    let mut timeline = vec![TimelineEntry {
        time: time_label(),
        label: "File loaded".into(),
    }];

    let prefs = &ctx.preferences;
    let hashes = hashing::compute(path, prefs.compute_md5, prefs.compute_sha1, prefs.compute_sha256)
        .map_err(|e| format!("Error de lectura de {}: {e}", path.display()))?;

    if prefs.compute_md5 {
        timeline.push(TimelineEntry { time: time_label(), label: "MD5 calculated".into() });
    }
    if prefs.compute_sha1 {
        timeline.push(TimelineEntry { time: time_label(), label: "SHA-1 calculated".into() });
    }
    if prefs.compute_sha256 {
        timeline.push(TimelineEntry { time: time_label(), label: "SHA-256 calculated".into() });
    }

    let static_analysis = match crate::analyzer::analyze(path) {
        Ok(analysis) => {
            timeline.push(TimelineEntry {
                time: time_label(),
                label: format!("File type identified: {}", analysis.file_type),
            });
            if analysis.is_pe {
                timeline.push(TimelineEntry { time: time_label(), label: "PE structure analyzed".into() });
            }
            if analysis.pe.as_ref().is_some_and(|pe| pe.has_certificate) {
                timeline.push(TimelineEntry {
                    time: time_label(),
                    label: "Digital signature present".into(),
                });
            }
            Some(analysis)
        }
        Err(_) => {
            timeline.push(TimelineEntry { time: time_label(), label: "Static analysis failed".into() });
            None
        }
    };

    // Análisis heurístico (FASE 4): reglas + puntuación + nivel.
    let findings = crate::rules::evaluate(static_analysis.as_ref(), Some(&hashes));
    let threat_score = crate::rules::score(&findings);
    let threat_level = crate::rules::level_from_score(threat_score);
    if !findings.is_empty() {
        timeline.push(TimelineEntry {
            time: time_label(),
            label: format!("Heuristic analysis complete: {} finding(s)", findings.len()),
        });
    }

    // Reputación en VirusTotal por hash (FASE 5): solo si el usuario lo habilitó.
    let reputation = if let Some(key) = &ctx.virustotal_api_key {
        match hashes.sha256.as_deref() {
            Some(sha256) => {
                timeline.push(TimelineEntry {
                    time: time_label(),
                    label: "Querying VirusTotal by hash".into(),
                });
                match crate::virustotal::lookup(key, sha256) {
                    Ok(vt) => {
                        if vt.available {
                            timeline.push(TimelineEntry {
                                time: time_label(),
                                label: format!(
                                    "VirusTotal: {}/{} engines flagged",
                                    vt.malicious + vt.suspicious,
                                    vt.total
                                ),
                            });
                        } else {
                            timeline.push(TimelineEntry {
                                time: time_label(),
                                label: "VirusTotal: hash not reported".into(),
                            });
                        }
                        Some(vt)
                    }
                    Err(e) => {
                        timeline.push(TimelineEntry {
                            time: time_label(),
                            label: format!("VirusTotal: {e}"),
                        });
                        Some(VirusTotalResult {
                            available: false,
                            hash: sha256.to_string(),
                            error: Some(e),
                            ..Default::default()
                        })
                    }
                }
            }
            None => {
                timeline.push(TimelineEntry {
                    time: time_label(),
                    label: "VirusTotal skipped: SHA-256 disabled".into(),
                });
                None
            }
        }
    } else {
        None
    };

    // Evaluación explicativa basada en evidencia (FASE 6): motor local,
    // determinista y sin red. Sintetiza hallazgos, análisis y reputación
    // directamente en el idioma del contexto de escaneo.
    let file_name = name_of(path);
    let assessment = crate::assessment::build(
        &file_name,
        threat_level,
        threat_score,
        &findings,
        static_analysis.as_ref(),
        reputation.as_ref(),
        ctx.language,
    );
    timeline.push(TimelineEntry { time: time_label(), label: "Assessment generated".into() });

    timeline.push(TimelineEntry { time: time_label(), label: "Scan completed".into() });

    Ok(ScanResult {
        id: Uuid::new_v4().to_string(),
        file_name,
        path: path.to_string_lossy().into_owned(),
        size: metadata.len(),
        hashes,
        threat_score,
        threat_level,
        findings,
        static_analysis,
        reputation,
        ai_assessment: Some(assessment),
        language: ctx.language,
        scanned_at: now_iso(),
        timeline,
    })
}

/// Cuenta los archivos de una carpeta (recursivo) para el progreso.
pub fn count_files(ctx: &ScanContext, dir: &Path) -> Result<usize, String> {
    let mut count = 0;
    count_walk(ctx, dir, &mut count)?;
    Ok(count)
}

fn count_walk(ctx: &ScanContext, dir: &Path, count: &mut usize) -> Result<(), String> {
    let read_dir = std::fs::read_dir(dir)
        .map_err(|e| format!("No se pudo leer {}: {e}", dir.display()))?;
    for entry in read_dir {
        if ctx.is_cancelled() {
            return Ok(());
        }
        let entry = entry.map_err(|e| format!("Error al leer directorio: {e}"))?;
        let file_type = entry
            .file_type()
            .map_err(|e| format!("Error al inspeccionar {}: {e}", entry.path().display()))?;
        if file_type.is_symlink() && !ctx.preferences.follow_symlinks {
            continue;
        }
        if file_type.is_dir() {
            count_walk(ctx, &entry.path(), count)?;
        } else if file_type.is_file() {
            *count += 1;
        }
    }
    Ok(())
}

/// Escanea una carpeta de forma recursiva llamando `on_file` por cada archivo.
pub fn folder_scan(
    ctx: &ScanContext,
    folder: &Path,
    total: usize,
    on_file: &dyn Fn(usize, usize, &Path),
) -> Result<FolderScanResult, String> {
    if ctx.is_cancelled() {
        return Err("Análisis cancelado".into());
    }

    let max_bytes = ctx.preferences.max_file_size_mb.saturating_mul(1024 * 1024);

    let mut files = Vec::new();
    let mut scanned = 0usize;
    let mut skipped = 0usize;
    let mut errors = 0usize;
    let mut total_bytes = 0u64;

    walk(
        ctx,
        folder,
        folder,
        total,
        on_file,
        &mut files,
        &mut scanned,
        &mut skipped,
        &mut errors,
        &mut total_bytes,
        max_bytes,
    )?;

    if ctx.is_cancelled() {
        return Err("Análisis cancelado".into());
    }

    Ok(FolderScanResult {
        id: Uuid::new_v4().to_string(),
        folder_path: folder.to_string_lossy().into_owned(),
        file_count: (scanned + skipped + errors) as u32,
        scanned_count: scanned as u32,
        skipped_count: skipped as u32,
        error_count: errors as u32,
        total_bytes,
        scanned_at: now_iso(),
        duration_ms: 0,
        files,
    })
}

#[allow(clippy::too_many_arguments)]
fn walk(
    ctx: &ScanContext,
    base: &Path,
    dir: &Path,
    total: usize,
    on_file: &dyn Fn(usize, usize, &Path),
    files: &mut Vec<FolderFileEntry>,
    scanned: &mut usize,
    skipped: &mut usize,
    errors: &mut usize,
    total_bytes: &mut u64,
    max_bytes: u64,
) -> Result<(), String> {
    let read_dir = std::fs::read_dir(dir)
        .map_err(|e| format!("No se pudo leer {}: {e}", dir.display()))?;
    for entry in read_dir {
        if ctx.is_cancelled() {
            return Ok(());
        }
        let entry = entry.map_err(|e| format!("Error al leer directorio: {e}"))?;
        let path = entry.path();
        let file_type = entry
            .file_type()
            .map_err(|e| format!("Error al inspeccionar {}: {e}", path.display()))?;

        if file_type.is_symlink() && !ctx.preferences.follow_symlinks {
            continue;
        }

        if file_type.is_dir() {
            walk(
                ctx, base, &path, total, on_file, files, scanned, skipped, errors, total_bytes,
                max_bytes,
            )?;
            continue;
        }
        if !file_type.is_file() {
            continue;
        }

        let relative = path
            .strip_prefix(base)
            .unwrap_or(&path)
            .to_string_lossy()
            .into_owned();
        let size = entry.metadata().map(|m| m.len()).unwrap_or(0);
        on_file(*scanned, total, &path);
        *total_bytes += size;

        if size > max_bytes && max_bytes > 0 {
            *skipped += 1;
            files.push(FolderFileEntry {
                relative_path: relative,
                size,
                hashes: FileHashes::default(),
                error: Some(format!("Supera el límite de {} MB", ctx.preferences.max_file_size_mb)),
            });
            continue;
        }

        match hashing::compute(
            &path,
            ctx.preferences.compute_md5,
            ctx.preferences.compute_sha1,
            ctx.preferences.compute_sha256,
        ) {
            Ok(hashes) => {
                *scanned += 1;
                files.push(FolderFileEntry {
                    relative_path: relative,
                    size,
                    hashes,
                    error: None,
                });
            }
            Err(e) => {
                *errors += 1;
                files.push(FolderFileEntry {
                    relative_path: relative,
                    size,
                    hashes: FileHashes::default(),
                    error: Some(e.to_string()),
                });
            }
        }
    }
    Ok(())
}

/// Convierte un `ScanResult` de archivo en una entrada de historial.
pub fn entry_from_file(r: &ScanResult) -> crate::models::ScanHistoryEntry {
    crate::models::ScanHistoryEntry {
        id: r.id.clone(),
        kind: ScanKind::File,
        path: r.path.clone(),
        name: r.file_name.clone(),
        size: r.size,
        file_count: 0,
        error_count: 0,
        threat_level: r.threat_level,
        scanned_at: r.scanned_at.clone(),
        duration_ms: 0,
    }
}

/// Convierte un `FolderScanResult` en una entrada de historial.
pub fn entry_from_folder(r: &FolderScanResult) -> crate::models::ScanHistoryEntry {
    crate::models::ScanHistoryEntry {
        id: r.id.clone(),
        kind: ScanKind::Folder,
        path: r.folder_path.clone(),
        name: Path::new(&r.folder_path)
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_else(|| r.folder_path.clone()),
        size: r.total_bytes,
        file_count: r.file_count,
        error_count: r.error_count,
        threat_level: ThreatLevel::Clean,
        scanned_at: r.scanned_at.clone(),
        duration_ms: r.duration_ms,
    }
}
