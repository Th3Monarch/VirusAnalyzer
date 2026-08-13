//! Persistencia de configuración.
//!
//! Configuración en JSON dentro del directorio de configuración de la app.
//! La arquitectura está preparada para migrar posteriormente a SQLite
//! (la migración de datos se gestiona con el campo `version`).
//!
//! Seguridad: la API key de VirusTotal se guarda en este archivo y nunca se
//! registra en logs ni se expone fuera de la app.

use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};

/// Versión actual del esquema de configuración.
pub const CONFIG_VERSION: u32 = 1;
/// Nombre del archivo de configuración.
pub const CONFIG_FILE: &str = "config.json";

/// Preferencia de tema de la interfaz.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ThemePreference {
    Dark,
    Light,
    System,
}

/// Preferencias de escaneo (se usan a partir de la FASE 2).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct ScanPreferences {
    /// Tamaño máximo por archivo (MB) por debajo del cual se analiza.
    pub max_file_size_mb: u64,
    /// Si se siguen enlaces simbólicos al escanear carpetas.
    pub follow_symlinks: bool,
    pub compute_md5: bool,
    pub compute_sha1: bool,
    pub compute_sha256: bool,
}

impl Default for ScanPreferences {
    fn default() -> Self {
        Self {
            max_file_size_mb: 200,
            follow_symlinks: false,
            compute_md5: true,
            compute_sha1: true,
            compute_sha256: true,
        }
    }
}

/// Preferencias de almacenamiento (cuarentena e historial).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct StoragePreferences {
    /// Ubicación personalizada de cuarentena (si es `None` se usa la
    /// predeterminada de la app).
    pub quarantine_dir: Option<PathBuf>,
    /// Días que se conserva el historial de análisis.
    pub keep_history_days: u64,
}

impl Default for StoragePreferences {
    fn default() -> Self {
        Self {
            quarantine_dir: None,
            keep_history_days: 90,
        }
    }
}

/// Configuración completa de la aplicación.
///
/// Todos los campos tienen `#[serde(default)]` para que archivos antiguos
/// sin los campos nuevos sigan cargándose sin romperse.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct AppConfig {
    /// Versión del esquema, usada para migraciones.
    pub version: u32,
    /// Idioma de la interfaz (`es` | `en`).
    pub language: String,
    pub theme: ThemePreference,
    /// API key de VirusTotal. Opcional; nunca obligatoria.
    pub virustotal_api_key: Option<String>,
    /// Consentimiento explícito para consultar reputación por hash en
    /// VirusTotal durante el análisis de archivos individuales.
    pub virustotal_enabled: bool,
    /// Integración con el menú contextual de Windows.
    pub context_menu_enabled: bool,
    pub scan: ScanPreferences,
    pub storage: StoragePreferences,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            version: CONFIG_VERSION,
            language: "es".into(),
            theme: ThemePreference::Dark,
            virustotal_api_key: None,
            virustotal_enabled: false,
            context_menu_enabled: false,
            scan: ScanPreferences::default(),
            storage: StoragePreferences::default(),
        }
    }
}

/// Carga, migra y persiste [`AppConfig`].
pub struct ConfigManager {
    pub config: AppConfig,
    pub path: PathBuf,
}

impl ConfigManager {
    /// Ruta absoluta del archivo de configuración para la app indicada.
    pub fn config_path(app: &AppHandle) -> Result<PathBuf, String> {
        let dir = app
            .path()
            .app_config_dir()
            .map_err(|e| format!("No se pudo resolver el directorio de configuración: {e}"))?;
        Ok(dir.join(CONFIG_FILE))
    }

    /// Carga la configuración; si no existe crea los valores por defecto.
    pub fn load(app: &AppHandle) -> Result<Self, String> {
        let path = Self::config_path(app)?;
        let config = match fs::read_to_string(&path) {
            Ok(raw) => serde_json::from_str(&raw)
                .map(Self::migrate)
                .map_err(|e| format!("Configuración inválida en {}: {e}", path.display()))?,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => AppConfig::default(),
            Err(e) => return Err(format!("No se pudo leer la configuración: {e}")),
        };
        Ok(Self { config, path })
    }

    /// Guarda la configuración (crea el directorio si es necesario).
    pub fn save(&self) -> Result<(), String> {
        let parent = self.path.parent().ok_or("Ruta de configuración inválida")?;
        fs::create_dir_all(parent)
            .map_err(|e| format!("No se pudo crear el directorio de configuración: {e}"))?;
        let raw = serde_json::to_string_pretty(&self.config)
            .map_err(|e| format!("No se pudo serializar la configuración: {e}"))?;
        fs::write(&self.path, raw).map_err(|e| format!("No se pudo escribir la configuración: {e}"))
    }

    /// Pipeline de migraciones futuras.
    ///
    /// TODO(migraciones): cuando `version` sea mayor que 1, aplicar aquí las
    /// migraciones `v(n) -> v(n+1)` antes de devolver la configuración.
    /// Actualmente el esquema es v1 y no hay migraciones pendientes.
    fn migrate(config: AppConfig) -> AppConfig {
        if config.version < CONFIG_VERSION {
            // Migraciones pendientes para versiones futuras.
        }
        config
    }
}
