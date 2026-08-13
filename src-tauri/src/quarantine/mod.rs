//! Administración de cuarentena (FASE 7): aislar, listar, restaurar, eliminar.
//!
//! Los archivos sospechosos se **mueven** (no se copian) a un directorio de
//! cuarentena aislado y se registran en un manifiesto (`manifest.json`) con su
//! ruta original, hashes, motivo y nivel. El usuario puede restaurarlos a su
//! ubicación original o eliminarlos definitivamente.
//!
//! Reglas de seguridad:
//! - **Nunca** se elimina ni se aísla un archivo automáticamente por tener una
//!   puntuación alta: siempre es una acción explícita del usuario.
//! - No se sobrescribe ninguna ruta al restaurar (si la original existe, se
//!   rechaza la operación para no perder datos).
//! - Los blobs usan identificadores estables (`Q-<año>-<secuencia>`) sin
//!   extensión original.

use std::fs;
use std::path::{Path, PathBuf};

use tauri::{AppHandle, Manager};

use crate::config::AppConfig;
use crate::hashing;
use crate::models::{FileHashes, QuarantineEntry, QuarantineSummary, ThreatLevel};

const MANIFEST_FILE: &str = "manifest.json";

/// Directorio efectivo de cuarentena: el configurado por el usuario o el
/// predeterminado de la app.
pub fn quarantine_dir(app: &AppHandle, config: &AppConfig) -> Result<PathBuf, String> {
    if let Some(dir) = &config.storage.quarantine_dir {
        let dir = dir.as_os_str().to_string_lossy().trim().to_string();
        if !dir.is_empty() {
            return Ok(PathBuf::from(dir));
        }
    }
    let base = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("No se pudo resolver el directorio de datos: {e}"))?;
    Ok(base.join("quarantine"))
}

fn manifest_path(dir: &Path) -> PathBuf {
    dir.join(MANIFEST_FILE)
}

fn load_manifest(dir: &Path) -> Vec<QuarantineEntry> {
    match fs::read_to_string(manifest_path(dir)) {
        Ok(raw) => serde_json::from_str(&raw).unwrap_or_default(),
        Err(_) => Vec::new(),
    }
}

fn save_manifest(dir: &Path, entries: &[QuarantineEntry]) -> Result<(), String> {
    fs::create_dir_all(dir)
        .map_err(|e| format!("No se pudo crear el directorio de cuarentena: {e}"))?;
    let raw = serde_json::to_string_pretty(entries)
        .map_err(|e| format!("No se pudo serializar el manifiesto: {e}"))?;
    let target = manifest_path(dir);
    let tmp = dir.join(format!("{MANIFEST_FILE}.tmp"));
    fs::write(&tmp, raw).map_err(|e| format!("No se pudo escribir el manifiesto: {e}"))?;
    fs::rename(&tmp, &target)
        .map_err(|e| format!("No se pudo actualizar el manifiesto: {e}"))
}

fn next_id(dir: &Path) -> String {
    let year = chrono::Local::now().format("%Y").to_string();
    let prefix = format!("Q-{year}-");
    let max: u32 = load_manifest(dir)
        .iter()
        .filter_map(|e| e.id.strip_prefix(&prefix).and_then(|s| s.parse::<u32>().ok()))
        .max()
        .unwrap_or(0);
    format!("{prefix}{:06}", max + 1)
}

fn hashes_or_err(path: &Path) -> Result<FileHashes, String> {
    hashing::compute(path, true, true, true)
        .map_err(|e| format!("No se pudieron calcular los hashes: {e}"))
}

/// Aísla un archivo: lo mueve a `dir` y registra su metadata.
fn quarantine_to_dir(
    dir: &Path,
    path: &str,
    threat_level: ThreatLevel,
    reason: Option<String>,
) -> Result<QuarantineEntry, String> {
    let src = PathBuf::from(path);
    let meta = fs::metadata(&src).map_err(|e| format!("No se pudo acceder a {path}: {e}"))?;
    if !meta.is_file() {
        return Err(format!("La ruta no es un archivo: {path}"));
    }
    if src.starts_with(dir) {
        return Err("El archivo ya se encuentra en cuarentena".into());
    }

    let hashes = hashes_or_err(&src)?;
    let original_name = src
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| "file".into());

    let id = next_id(dir);
    let blob = dir.join(id.clone());
    fs::create_dir_all(dir)
        .map_err(|e| format!("No se pudo crear el directorio de cuarentena: {e}"))?;
    fs::rename(&src, &blob)
        .map_err(|e| format!("No se pudo mover {} a cuarentena: {e}", src.display()))?;

    let entry = QuarantineEntry {
        id,
        original_path: src.to_string_lossy().into_owned(),
        original_name,
        quarantined_path: blob.to_string_lossy().into_owned(),
        size: meta.len(),
        hashes,
        reason: reason.unwrap_or_default(),
        threat_level,
        quarantined_at: crate::scanner::now_iso(),
    };

    let mut entries = load_manifest(dir);
    entries.push(entry.clone());
    save_manifest(dir, &entries)?;
    Ok(entry)
}

/// Restaura `id` a su ubicación original (sin sobrescribir rutas existentes).
fn restore_from_dir(dir: &Path, id: &str) -> Result<QuarantineEntry, String> {
    let mut entries = load_manifest(dir);
    let idx = entries
        .iter()
        .position(|e| e.id == id)
        .ok_or_else(|| format!("No hay ninguna entrada con id {id}"))?;

    let entry = entries[idx].clone();
    let blob = PathBuf::from(&entry.quarantined_path);
    if !blob.exists() {
        return Err(format!("El archivo aislado ya no existe: {}", blob.display()));
    }
    let target = PathBuf::from(&entry.original_path);
    if target.exists() {
        return Err(format!(
            "La ubicación original ya existe ({}). No se restaura para evitar sobrescribir el archivo actual.",
            target.display()
        ));
    }
    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("No se pudo crear el directorio de destino: {e}"))?;
    }
    fs::rename(&blob, &target)
        .map_err(|e| format!("No se pudo restaurar {}: {e}", blob.display()))?;

    entries.remove(idx);
    save_manifest(dir, &entries)?;
    Ok(entry)
}

/// Elimina definitivamente `id` de cuarentena (archivo + registro).
fn delete_from_dir(dir: &Path, id: &str) -> Result<(), String> {
    let mut entries = load_manifest(dir);
    let idx = entries
        .iter()
        .position(|e| e.id == id)
        .ok_or_else(|| format!("No hay ninguna entrada con id {id}"))?;
    let entry = entries.remove(idx);
    let blob = PathBuf::from(&entry.quarantined_path);
    if blob.exists() {
        fs::remove_file(&blob)
            .map_err(|e| format!("No se pudo eliminar {}: {e}", blob.display()))?;
    }
    save_manifest(dir, &entries)
}

fn list_from_dir(dir: &Path) -> Vec<QuarantineEntry> {
    let mut entries = load_manifest(dir);
    entries.sort_by(|a, b| b.quarantined_at.cmp(&a.quarantined_at));
    entries
}

// ---------------------------------------------------------------------------
// API pública (comandos Tauri)
// ---------------------------------------------------------------------------

pub fn quarantine_file(
    app: &AppHandle,
    config: &AppConfig,
    path: &str,
    threat_level: ThreatLevel,
    reason: Option<String>,
) -> Result<QuarantineEntry, String> {
    quarantine_to_dir(&quarantine_dir(app, config)?, path, threat_level, reason)
}

pub fn summary(app: &AppHandle, config: &AppConfig) -> Result<QuarantineSummary, String> {
    let dir = quarantine_dir(app, config)?;
    Ok(QuarantineSummary {
        dir: dir.to_string_lossy().into_owned(),
        entries: list_from_dir(&dir),
    })
}

pub fn restore(app: &AppHandle, config: &AppConfig, id: &str) -> Result<QuarantineEntry, String> {
    restore_from_dir(&quarantine_dir(app, config)?, id)
}

pub fn delete(app: &AppHandle, config: &AppConfig, id: &str) -> Result<(), String> {
    delete_from_dir(&quarantine_dir(app, config)?, id)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tmp_dir(tag: &str) -> PathBuf {
        let base = std::env::temp_dir().join(format!("va_q_test_{tag}_{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&base).expect("crear dir temporal");
        base
    }

    fn entry(id: &str) -> QuarantineEntry {
        QuarantineEntry {
            id: id.into(),
            ..Default::default()
        }
    }

    #[test]
    fn quarantine_moves_and_restores() {
        let tmp = tmp_dir("roundtrip");
        let quarantine = tmp.join("quarantine");
        let source = tmp.join("sample.txt");
        fs::write(&source, b"not actually malware").expect("escribir");
        let original_path = source.to_string_lossy().into_owned();

        let e = quarantine_to_dir(
            &quarantine,
            &original_path,
            ThreatLevel::High,
            Some("test".into()),
        )
        .expect("aislar");
        assert_eq!(e.original_name, "sample.txt");
        assert!(!source.exists(), "el original debe haber sido movido");
        assert!(PathBuf::from(&e.quarantined_path).exists(), "el blob existe");

        let restored = restore_from_dir(&quarantine, &e.id).expect("restaurar");
        assert_eq!(restored.original_name, "sample.txt");
        assert!(source.exists(), "el archivo vuelve a su ruta original");
        assert!(!PathBuf::from(&e.quarantined_path).exists(), "el blob desaparece");
        assert!(list_from_dir(&quarantine).is_empty(), "el manifiesto queda vacío");
        fs::remove_dir_all(&tmp).ok();
    }

    #[test]
    fn restore_refuses_overwrite() {
        let tmp = tmp_dir("overwrite");
        let quarantine = tmp.join("quarantine");
        let source = tmp.join("keep.txt");
        fs::write(&source, b"original file still here").expect("escribir");
        let original_path = source.to_string_lossy().into_owned();

        let e = quarantine_to_dir(&quarantine, &original_path, ThreatLevel::High, None)
            .expect("aislar");
        // Recrear la ruta original para simular que otro archivo la ocupa.
        fs::write(&source, b"another file").expect("recrear");
        let err = restore_from_dir(&quarantine, &e.id).expect_err("debe rechazar");
        assert!(err.contains("ya existe"), "no sobrescribir: {err}");
        assert!(PathBuf::from(&e.quarantined_path).exists(), "el blob se conserva");
        fs::remove_dir_all(&tmp).ok();
    }

    #[test]
    fn delete_removes_blob_and_record() {
        let tmp = tmp_dir("delete");
        let quarantine = tmp.join("quarantine");
        let source = tmp.join("bad.exe");
        fs::write(&source, b"MZ fake").expect("escribir");
        let e = quarantine_to_dir(&quarantine, &source.to_string_lossy(), ThreatLevel::Critical, None)
            .expect("aislar");
        delete_from_dir(&quarantine, &e.id).expect("eliminar");
        assert!(!PathBuf::from(&e.quarantined_path).exists());
        assert!(list_from_dir(&quarantine).is_empty());
        fs::remove_dir_all(&tmp).ok();
    }

    #[test]
    fn next_id_increments() {
        let dir = tmp_dir("nextid");
        save_manifest(&dir, &[entry("Q-2026-000001"), entry("Q-2026-000007"), entry("Q-2026-000003")])
            .expect("manifest");
        assert_eq!(next_id(&dir), "Q-2026-000008");
        fs::remove_dir_all(&dir).ok();
    }
}
