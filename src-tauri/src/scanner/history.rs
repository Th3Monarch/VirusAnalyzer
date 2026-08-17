//! Almacén del historial y resultados de escaneos, con persistencia en disco.
//!
//! El historial completo (`ScanHistoryEntry`) y los resultados completos
//! (JSON indexados por id) viven en un único archivo `history.json` dentro del
//! directorio de configuración de la app. Al arrancar se cargan a memoria; al
//! completar cada escaneo se vuelven a persistir.
//!
//! El identificador estable de un análisis es su `id` (UUID): se genera al
//! crear el resultado, se conserva igual en el historial, en los resultados y
//! en la navegación de la UI. Los historiales antiguos sin `id` reciben un id
//! derivado de datos inmutables (ruta, nombre, fecha y tipo) y se persisten
//! una única vez; no se regenera en cada carga.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::models::{ScanHistoryEntry, ScanKind};

/// Nombre del archivo de persistencia dentro del directorio de configuración.
pub const HISTORY_FILE: &str = "history.json";
const HISTORY_VERSION: u32 = 1;

/// Escaneo en curso (para poder cancelarlo).
pub struct ActiveScan {
    pub cancel: Arc<AtomicBool>,
}

/// Formato del archivo de persistencia del historial.
#[derive(Debug, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
struct HistoryFile {
    version: u32,
    history: Vec<ScanHistoryEntry>,
    results: Vec<StoredResult>,
}

impl Default for HistoryFile {
    fn default() -> Self {
        Self {
            version: HISTORY_VERSION,
            history: Vec::new(),
            results: Vec::new(),
        }
    }
}

/// Resultado completo almacenado junto a su id estable.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct StoredResult {
    id: String,
    value: serde_json::Value,
}

/// Estado global compartido con los comandos Tauri.
#[derive(Default)]
pub struct ScanStore {
    /// Historial en memoria (más reciente primero).
    pub history: Vec<ScanHistoryEntry>,
    /// Resultados completos indexados por id (la misma clave que usa el
    /// historial y la navegación de la UI).
    pub results: HashMap<String, serde_json::Value>,
    /// Escaneo activo, si lo hay.
    pub active: Option<ActiveScan>,
    /// Ruta del archivo de persistencia (si la tienda se cargó desde disco).
    file: Option<PathBuf>,
}

impl ScanStore {
    /// Carga el historial persistido (o una tienda vacía si no existe) y migra
    /// las entradas antiguas sin `id` a identificadores estables.
    pub fn load(file: PathBuf) -> Self {
        let mut store = Self {
            file: Some(file.clone()),
            ..Default::default()
        };
        let Ok(raw) = std::fs::read_to_string(&file) else {
            return store;
        };
        let Ok(data) = serde_json::from_str::<HistoryFile>(&raw) else {
            return store;
        };
        store.history = data.history;
        store.results = data.results.into_iter().map(|r| (r.id, r.value)).collect();
        if store.migrate_legacy_ids() {
            store.save();
        }
        store
    }

    /// Asigna ids estables a las entradas que no lo tienen. Devuelve `true` si
    /// algo cambió (para persistir el resultado de la migración).
    fn migrate_legacy_ids(&mut self) -> bool {
        let mut changed = false;
        for entry in self.history.iter_mut() {
            if entry.id.trim().is_empty() {
                entry.id = legacy_id(entry);
                changed = true;
            }
        }
        changed
    }

    /// Persiste historial y resultados en el archivo de la tienda.
    pub fn save(&self) {
        let Some(file) = &self.file else { return };
        if let Some(parent) = file.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let data = HistoryFile {
            version: HISTORY_VERSION,
            history: self.history.clone(),
            results: self
                .results
                .iter()
                .map(|(id, value)| StoredResult {
                    id: id.clone(),
                    value: value.clone(),
                })
                .collect(),
        };
        if let Ok(raw) = serde_json::to_string_pretty(&data) {
            let _ = std::fs::write(file, raw);
        }
    }
}

/// Deriva un id estable (no regenerado en cada carga) para entradas antiguas
/// sin `id`, a partir de datos inmutables de la propia entrada.
fn legacy_id(entry: &ScanHistoryEntry) -> String {
    use sha2::{Digest, Sha256};
    let kind = match entry.kind {
        ScanKind::File => "file",
        ScanKind::Folder => "folder",
    };
    let seed = format!("{kind}|{}|{}|{}", entry.path, entry.scanned_at, entry.name);
    let mut hasher = Sha256::new();
    hasher.update(seed.as_bytes());
    let digest = hasher.finalize();
    let hex: String = digest.iter().map(|b| format!("{b:02x}")).collect();
    format!("legacy-{}", &hex[..16])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::ThreatLevel;

    fn sample_entry(id: &str) -> ScanHistoryEntry {
        ScanHistoryEntry {
            id: id.to_string(),
            kind: ScanKind::File,
            path: r"C:\Temp\sample.exe".into(),
            name: "sample.exe".into(),
            size: 1234,
            file_count: 0,
            error_count: 0,
            threat_level: ThreatLevel::High,
            scanned_at: "2026-08-13T10:00:00Z".into(),
            duration_ms: 42,
        }
    }

    fn sample_result(id: &str) -> serde_json::Value {
        serde_json::json!({
            "id": id,
            "fileName": "sample.exe",
            "path": r"C:\Temp\sample.exe",
            "size": 1234,
            "hashes": { "md5": null, "sha1": null, "sha256": null },
            "threatScore": 55,
            "threatLevel": "high",
            "findings": [],
            "staticAnalysis": null,
            "reputation": null,
            "aiAssessment": null,
            "language": "es",
            "scannedAt": "2026-08-13T10:00:00Z",
            "timeline": []
        })
    }

    fn temp_file(tag: &str) -> PathBuf {
        let dir =
            std::env::temp_dir().join(format!("va-history-test-{tag}-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        dir.join(HISTORY_FILE)
    }

    #[test]
    fn round_trip_preserves_ids_and_results() {
        let path = temp_file("roundtrip");
        let mut store = ScanStore::load(path.clone());
        store.history.push(sample_entry("abc"));
        store.results.insert("abc".into(), sample_result("abc"));
        store.save();

        let reloaded = ScanStore::load(path.clone());
        assert_eq!(reloaded.history.len(), 1);
        assert_eq!(
            reloaded.history[0].id, "abc",
            "el id se conserva tras recargar"
        );
        let value = reloaded.results.get("abc").expect("resultado persistido");
        assert_eq!(value["id"], "abc");
        std::fs::remove_dir_all(path.parent().unwrap()).ok();
    }

    #[test]
    fn legacy_entry_without_id_gets_stable_id() {
        let path = temp_file("legacy");
        let raw = serde_json::json!({
            "version": 1,
            "history": [{
                "kind": "file",
                "path": r"C:\Temp\old.exe",
                "name": "old.exe",
                "size": 1,
                "fileCount": 0,
                "errorCount": 0,
                "threatLevel": "medium",
                "scannedAt": "2026-01-01T00:00:00Z",
                "durationMs": 1
            }],
            "results": []
        });
        std::fs::write(&path, raw.to_string()).unwrap();

        let first = ScanStore::load(path.clone());
        assert_eq!(first.history.len(), 1);
        let id1 = first.history[0].id.clone();
        assert!(id1.starts_with("legacy-"), "id estable derivado");

        // Segunda carga: el id ya persistido no se regenera.
        let second = ScanStore::load(path.clone());
        assert_eq!(
            second.history[0].id, id1,
            "el id no se regenera en cada carga"
        );
        std::fs::remove_dir_all(path.parent().unwrap()).ok();
    }

    #[test]
    fn missing_file_yields_empty_store() {
        let path = temp_file("missing");
        let store = ScanStore::load(path.clone());
        assert!(store.history.is_empty());
        assert!(store.results.is_empty());
        std::fs::remove_dir_all(path.parent().unwrap()).ok();
    }
}
