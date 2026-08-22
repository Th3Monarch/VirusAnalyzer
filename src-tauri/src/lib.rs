//! Prometeo 2.0 — Backend Tauri.
//!
//! El frontend (React) está separado estrictamente del backend (Rust).
//! Rust posee todo el acceso a sistema (archivos, procesos, hashing, red)
//! y expone un conjunto pequeño de comandos a la interfaz.

// Silencia los mensajes informativos del linker MSVC (bibliotecas .dll.lib).
#![allow(linker_messages)]

mod ai;
mod analyzer;
mod assessment;
mod assistant;
mod config;
#[cfg(target_os = "windows")]
mod contextmenu;
mod hashing;
mod models;
mod platform;
#[cfg(target_os = "windows")]
mod powershell;
#[cfg(target_os = "windows")]
mod powershell_reference;
mod quarantine;
mod report;
mod rules;
mod scanner;
mod system;
mod virustotal;

use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Instant;

use serde_json::json;
use tauri::Emitter;
use tauri::Manager;
use uuid::Uuid;

use assistant::commands::AssistantState;
use config::{AppConfig, ConfigManager};
use models::{AppInfo, SystemInfo};
use scanner::history::{ActiveScan, ScanStore};
use scanner::ScanContext;

/// Devuelve la configuración actual de la aplicación.
#[tauri::command]
fn get_config(app: tauri::AppHandle) -> Result<AppConfig, String> {
    ConfigManager::load(&app).map(|m| m.config)
}

/// Persiste la configuración completa y devuelve el valor guardado.
#[tauri::command]
fn save_config(app: tauri::AppHandle, config: AppConfig) -> Result<AppConfig, String> {
    let mut manager = ConfigManager::load(&app)?;
    manager.config = config;
    manager.save()?;
    Ok(manager.config)
}

/// Metadatos de la propia aplicación.
#[tauri::command]
fn get_app_info() -> AppInfo {
    AppInfo {
        name: "Prometeo 2.0".into(),
        version: env!("CARGO_PKG_VERSION").into(),
        tagline: "Analyze. Understand. Protect.".into(),
    }
}

/// Información básica del sistema host.
#[tauri::command]
fn get_system_info() -> Result<SystemInfo, String> {
    system::collect()
}

/// Plataforma actual de ejecución (windows | linux | macos).
#[tauri::command]
fn get_platform() -> platform::Platform {
    platform::Platform::current()
}

/// Inspecciona una ruta (archivo o carpeta) para preparar un escaneo.
#[tauri::command]
fn get_path_info(path: String) -> Result<models::PathInfo, String> {
    scanner::path_info(&PathBuf::from(path))
}

/// Lanza un escaneo de archivo o carpeta en segundo plano.
///
/// Devuelve el `scanId` inmediatamente; el progreso llega por eventos
/// `scan-progress` y el final por `scan-completed` / `scan-error` / `scan-cancelled`.
#[tauri::command]
fn scan_path(
    app: tauri::AppHandle,
    state: tauri::State<'_, Arc<Mutex<ScanStore>>>,
    path: String,
) -> Result<String, String> {
    let target = PathBuf::from(&path);
    if !target.exists() {
        return Err(format!("La ruta no existe: {path}"));
    }

    {
        let guard = state.lock().map_err(|_| "Estado bloqueado".to_string())?;
        if guard.active.is_some() {
            return Err("Ya hay un análisis en curso".into());
        }
    }

    let is_dir = target.is_dir();
    let id = Uuid::new_v4().to_string();
    let cancel = Arc::new(AtomicBool::new(false));

    {
        let mut guard = state.lock().map_err(|_| "Estado bloqueado".to_string())?;
        guard.active = Some(ActiveScan {
            cancel: cancel.clone(),
        });
    }

    let config = ConfigManager::load(&app)
        .map(|m| m.config)
        .unwrap_or_default();
    let virustotal_api_key = if config.virustotal_enabled {
        config.virustotal_api_key.filter(|k| !k.trim().is_empty())
    } else {
        None
    };
    let store = state.inner().clone();
    let app_handle = app.clone();
    let scan_id = id.clone();

    tauri::async_runtime::spawn_blocking(move || {
        let ctx = ScanContext {
            cancel: cancel.clone(),
            preferences: config.scan,
            virustotal_api_key,
            // El idioma se lee de la configuración en el momento del escaneo
            // (fuente única de verdad): los cambios en tiempo real se aplican
            // al siguiente análisis. Valor desconocido → `en` como fallback.
            language: models::Language::from_config(&config.language),
        };
        let started = Instant::now();
        let emit = |event: &str, payload: serde_json::Value| {
            let _ = app_handle.emit(event, payload);
        };

        emit(
            "scan-progress",
            json!({ "scanId": scan_id, "current": 0, "total": 0, "filePath": target.to_string_lossy() }),
        );

        let outcome: Result<(serde_json::Value, models::ScanHistoryEntry), String> = if is_dir {
            let total = scanner::count_files(&ctx, &target).unwrap_or(0);
            let result = scanner::folder_scan(&ctx, &target, total, &|current, total, path| {
                emit(
                    "scan-progress",
                    json!({ "scanId": scan_id, "current": current, "total": total, "filePath": path.to_string_lossy() }),
                );
            });
            match result {
                Ok(mut r) => {
                    r.duration_ms = started.elapsed().as_millis() as u64;
                    let mut entry = scanner::entry_from_folder(&r);
                    entry.duration_ms = r.duration_ms;
                    Ok((json!(r), entry))
                }
                Err(e) => Err(e),
            }
        } else {
            match scanner::file_scan(&ctx, &target) {
                Ok(mut r) => {
                    r.timeline.push(models::TimelineEntry {
                        time: scanner::time_label(),
                        label: "Analysis queued".into(),
                    });
                    let mut entry = scanner::entry_from_file(&r);
                    entry.duration_ms = started.elapsed().as_millis() as u64;
                    Ok((json!(r), entry))
                }
                Err(e) => Err(e),
            }
        };

        if ctx.is_cancelled() {
            emit("scan-cancelled", json!({ "scanId": scan_id }));
            if let Ok(mut guard) = store.lock() {
                guard.active = None;
            }
            return;
        }

        match outcome {
            Ok((value, entry)) => {
                if let Ok(mut guard) = store.lock() {
                    // La clave de resultados usa el id del análisis (el mismo
                    // que ve la UI en el historial y en la ruta de detalle),
                    // no el id del escaneo. Así `get_analysis_by_id`, el
                    // informe y la vista previa localizan el resultado.
                    guard.results.insert(entry.id.clone(), value);
                    guard.history.insert(0, entry.clone());
                    guard.active = None;
                    guard.save();
                }
                emit(
                    "scan-completed",
                    json!({ "scanId": scan_id, "entry": entry }),
                );
            }
            Err(message) => {
                if let Ok(mut guard) = store.lock() {
                    guard.active = None;
                }
                emit(
                    "scan-error",
                    json!({ "scanId": scan_id, "message": message }),
                );
            }
        }
    });

    Ok(id)
}

/// Cancela el escaneo en curso.
#[tauri::command]
fn cancel_scan(state: tauri::State<'_, Arc<Mutex<ScanStore>>>) -> Result<bool, String> {
    let guard = state.lock().map_err(|_| "Estado bloqueado".to_string())?;
    if let Some(active) = &guard.active {
        active.cancel.store(true, Ordering::Relaxed);
        Ok(true)
    } else {
        Ok(false)
    }
}

/// Devuelve el historial de escaneos (resumen ligero).
#[tauri::command]
fn get_scan_history(
    state: tauri::State<'_, Arc<Mutex<ScanStore>>>,
) -> Result<Vec<models::ScanHistoryEntry>, String> {
    let guard = state.lock().map_err(|_| "Estado bloqueado".to_string())?;
    Ok(guard.history.clone())
}

/// Devuelve el resultado completo de un análisis por su id estable.
///
/// Consulta la única fuente de verdad (el historial persistente, cargado en
/// memoria y actualizado en cada escaneo), por lo que funciona tanto en la
/// misma sesión como después de reiniciar la aplicación.
#[tauri::command]
fn get_analysis_by_id(
    state: tauri::State<'_, Arc<Mutex<ScanStore>>>,
    id: String,
) -> Result<Option<serde_json::Value>, String> {
    result_by_id(&state, &id)
}

/// Devuelve el resultado completo de un escaneo por su id (equivalente a
/// `get_analysis_by_id`; se mantiene por compatibilidad).
#[tauri::command]
fn get_scan_result(
    state: tauri::State<'_, Arc<Mutex<ScanStore>>>,
    id: String,
) -> Result<Option<serde_json::Value>, String> {
    result_by_id(&state, &id)
}

fn result_by_id(
    state: &tauri::State<'_, Arc<Mutex<ScanStore>>>,
    id: &str,
) -> Result<Option<serde_json::Value>, String> {
    let guard = state.lock().map_err(|_| "Estado bloqueado".to_string())?;
    Ok(guard.results.get(id).cloned())
}

/// Catálogo descriptivo de las reglas del motor heurístico (página Reglas).
#[tauri::command]
fn get_rules() -> Vec<models::RuleInfo> {
    rules::catalog_info()
}

/// Consulta la reputación de un hash en VirusTotal (consentimiento explícito:
/// el usuario debe haber habilitado la integración y tener una API key).
#[tauri::command]
fn virustotal_lookup(
    app: tauri::AppHandle,
    hash: String,
) -> Result<models::VirusTotalResult, String> {
    let manager = ConfigManager::load(&app)?;
    if !manager.config.virustotal_enabled {
        return Err("La consulta a VirusTotal está deshabilitada en Ajustes".into());
    }
    let key = manager
        .config
        .virustotal_api_key
        .filter(|k| !k.trim().is_empty())
        .ok_or("No hay una API key de VirusTotal configurada")?;
    let hash = hash.trim().to_lowercase();
    if !matches!(hash.len(), 32 | 40 | 64) {
        return Err(
            "Hash inválido: se esperan 32 (MD5), 40 (SHA-1) o 64 (SHA-256) caracteres".into(),
        );
    }
    virustotal::lookup(&key, &hash)
}

// --- Cuarentena (FASE 7) ---------------------------------------------------

/// Aísla un archivo (acción explícita del usuario): lo mueve a la cuarentena.
#[tauri::command]
fn quarantine_file(
    app: tauri::AppHandle,
    path: String,
    threat_level: models::ThreatLevel,
    reason: Option<String>,
) -> Result<models::QuarantineEntry, String> {
    let config = ConfigManager::load(&app)?.config;
    quarantine::quarantine_file(&app, &config, &path, threat_level, reason)
}

/// Devuelve el directorio efectivo de cuarentena y sus entradas.
#[tauri::command]
fn get_quarantine(app: tauri::AppHandle) -> Result<models::QuarantineSummary, String> {
    let config = ConfigManager::load(&app)?.config;
    quarantine::summary(&app, &config)
}

/// Restaura una entrada de cuarentena a su ubicación original.
#[tauri::command]
fn restore_quarantined(
    app: tauri::AppHandle,
    id: String,
) -> Result<models::QuarantineEntry, String> {
    let config = ConfigManager::load(&app)?.config;
    quarantine::restore(&app, &config, &id)
}

/// Elimina definitivamente una entrada de cuarentena.
#[tauri::command]
fn delete_quarantined(app: tauri::AppHandle, id: String) -> Result<bool, String> {
    let config = ConfigManager::load(&app)?.config;
    quarantine::delete(&app, &config, &id)?;
    Ok(true)
}

// --- Informes (FASE 8) ------------------------------------------------------

/// Genera y escribe un informe (HTML/CSV) de un análisis ya almacenado.
#[tauri::command]
fn export_report(
    state: tauri::State<'_, Arc<Mutex<ScanStore>>>,
    scan_id: String,
    format: models::ReportFormat,
    path: String,
) -> Result<String, String> {
    let value = {
        let guard = state.lock().map_err(|_| "Estado bloqueado".to_string())?;
        guard.results.get(&scan_id).cloned()
    }
    .ok_or("No se encontró el análisis solicitado".to_string())?;
    let content = report::render(&value, format)?;
    std::fs::write(&path, content).map_err(|e| format!("No se pudo escribir el informe: {e}"))?;
    Ok(path)
}

/// Devuelve el contenido de un informe (HTML/CSV) para vista previa.
#[tauri::command]
fn preview_report(
    state: tauri::State<'_, Arc<Mutex<ScanStore>>>,
    scan_id: String,
    format: models::ReportFormat,
) -> Result<String, String> {
    let value = {
        let guard = state.lock().map_err(|_| "Estado bloqueado".to_string())?;
        guard.results.get(&scan_id).cloned()
    }
    .ok_or("No se encontró el análisis solicitado".to_string())?;
    report::render(&value, format)
}

// --- Terminal multiplataforma -----------------------------------------------
//
// Este módulo SOLO se activa por petición explícita del usuario en su página.
// El escáner de malware es estático y nunca invoca terminal.

/// Ejecuta un comando en el shell del sistema con los permisos del usuario.
///
/// `confirm` debe ser `true` para comandos de alto riesgo: el frontend muestra
/// una confirmación explícita antes de pasarlo. No eleva privilegios.
#[tauri::command]
async fn execute_powershell(
    state: tauri::State<'_, Arc<dyn platform::TerminalManager>>,
    command: String,
    confirm: Option<bool>,
) -> Result<platform::TerminalResult, String> {
    let trimmed = command.trim();
    if trimmed.is_empty() {
        return Err("El comando está vacío".into());
    }
    if trimmed.len() > platform::terminal::MAX_COMMAND_LEN {
        return Err("El comando es demasiado largo".into());
    }
    let risk = state.classify(trimmed);
    if risk == platform::RiskLevel::High && confirm != Some(true) {
        return Err(
            "Comando de alto riesgo: requiere confirmación explícita desde la interfaz".into(),
        );
    }
    let owned = trimmed.to_string();
    let terminal = state.inner().clone();
    let result = tauri::async_runtime::spawn_blocking(move || {
        terminal.execute(&owned, platform::terminal::DEFAULT_TIMEOUT_MS)
    })
    .await
    .map_err(|e| format!("Error interno al ejecutar terminal: {e}"))??;
    Ok(result)
}

/// Cancela la ejecución del terminal en curso (si la hay).
#[tauri::command]
fn cancel_powershell(
    state: tauri::State<'_, Arc<dyn platform::TerminalManager>>,
) -> Result<bool, String> {
    Ok(state.cancel())
}

/// Clasificación educativa del riesgo de un comando.
#[tauri::command]
fn classify_powershell_command(command: String) -> platform::RiskLevel {
    platform::current_terminal().classify(&command)
}

/// Catálogo educativo de comandos, traducido al idioma indicado.
#[tauri::command]
fn get_powershell_reference(language: String) -> Vec<platform::TerminalCommandInfo> {
    let lang = models::Language::from_config(&language);
    platform::current_terminal().get_reference(lang)
}

// --- Menú contextual -------------------------------------------------------

/// Registra el menú contextual (solo para el usuario actual).
#[tauri::command]
fn install_context_menu(label: String) -> Result<bool, String> {
    platform::current_context_menu().install(&label)?;
    Ok(true)
}

/// Elimina la entrada del menú contextual.
#[tauri::command]
fn uninstall_context_menu() -> Result<bool, String> {
    platform::current_context_menu().uninstall()?;
    Ok(true)
}

/// Comprueba si el menú contextual está registrado.
#[tauri::command]
fn is_context_menu_installed() -> Result<bool, String> {
    platform::current_context_menu().is_installed()
}

/// Devuelve (una sola vez) la ruta con la que se lanzó la aplicación desde el
/// menú contextual. El frontend la usa para iniciar el análisis directamente.
#[tauri::command]
fn take_launch_path(state: tauri::State<'_, Arc<Mutex<Option<String>>>>) -> Option<String> {
    state.lock().ok().and_then(|mut guard| guard.take())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            // La tienda se carga desde disco (historial + resultados) y queda
            // disponible para todos los comandos.
            let dir = app.path().app_config_dir()?;
            let store = scanner::history::ScanStore::load(dir.join(scanner::history::HISTORY_FILE));
            app.manage(Arc::new(Mutex::new(store)));
            // Terminal multiplataforma (reemplaza PowerShellManager).
            app.manage(platform::current_terminal());
            // Ruta recibida al lanzarse desde el menú contextual (se consume
            // una sola vez con `take_launch_path`).
            app.manage(Arc::new(Mutex::new(std::env::args().nth(1))));
            // Estado del assistant AI companion.
            let ysmel_active = Arc::new(AtomicBool::new(false));
            let fenix_active = Arc::new(AtomicBool::new(false));
            let provider = crate::ai::manager::ProviderManager::new();
            let assistant_silent_mode = {
                let app_handle = app.handle().clone();
                crate::config::ConfigManager::load(&app_handle)
                    .map(|m| m.config.assistant_silent_mode)
                    .unwrap_or(false)
            };
            let assistant = Arc::new(AssistantState::new(
                ysmel_active.clone(),
                fenix_active.clone(),
                provider,
                assistant_silent_mode,
            ));
            assistant.set_app_handle(app.handle().clone());

            // Inicializar provider Ollama desde config en background.
            // Si Ollama está habilitado y disponible, se conecta automáticamente.
            let assistant_clone = Arc::clone(&assistant);
            let app_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                let config = match crate::config::ConfigManager::load(&app_handle) {
                    Ok(m) => m.config,
                    Err(_) => return,
                };
                let mut mgr = assistant_clone.provider.write().await;
                mgr.init_from_config(&config).await;
            });

            // Inicializar pipeline de voz desde config en background.
            let assistant_voice = Arc::clone(&assistant);
            let app_handle_v = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                let config = match crate::config::ConfigManager::load(&app_handle_v) {
                    Ok(m) => m.config,
                    Err(_) => return,
                };
                let voice_config = crate::assistant::voice::VoiceConfig {
                    enabled: config.voice.tts_provider == "kokoro"
                        || config.voice.stt_provider == "whisper",
                    auto_speak: false,
                    language: config.voice.language.clone(),
                    stt_provider: config.voice.stt_provider.clone(),
                    tts_provider: config.voice.tts_provider.clone(),
                    tts_url: config.voice.tts_url.clone(),
                    stt_url: config.voice.stt_url.clone(),
                    speech_rate: config.voice.speech_rate,
                    volume: config.voice.volume,
                    voice_id: if config.voice.voice_id.is_empty() {
                        crate::assistant::voice::default_voice_for_language(&config.voice.language)
                            .into()
                    } else {
                        config.voice.voice_id.clone()
                    },
                };
                let mut voice = assistant_voice.voice.lock().await;
                voice.init_from_config(&voice_config).await;
            });

            app.manage(assistant);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_config,
            save_config,
            get_app_info,
            get_platform,
            get_system_info,
            get_path_info,
            scan_path,
            cancel_scan,
            get_scan_history,
            get_analysis_by_id,
            get_scan_result,
            get_rules,
            virustotal_lookup,
            quarantine_file,
            get_quarantine,
            restore_quarantined,
            delete_quarantined,
            export_report,
            preview_report,
            execute_powershell,
            cancel_powershell,
            classify_powershell_command,
            get_powershell_reference,
            install_context_menu,
            uninstall_context_menu,
            is_context_menu_installed,
            take_launch_path,
            assistant::commands::assistant_send_message,
            assistant::commands::assistant_get_history,
            assistant::commands::assistant_clear_session,
            assistant::commands::assistant_get_context,
            assistant::commands::assistant_set_context,
            assistant::commands::assistant_get_provider_info,
            assistant::commands::assistant_check_provider_health,
            assistant::commands::assistant_cancel_pending,
            assistant::commands::assistant_set_provider,
            assistant::commands::assistant_update_ollama,
            assistant::commands::assistant_test_ollama,
            assistant::commands::assistant_set_silent_mode,
            assistant::commands::assistant_get_silent_mode,
            assistant::commands::assistant_get_voice_state,
            assistant::commands::assistant_update_voice_config,
            assistant::commands::assistant_synthesize,
            assistant::commands::assistant_transcribe,
            assistant::commands::assistant_voice_health,
            assistant::commands::assistant_list_voices,
            assistant::commands::assistant_get_accent_info,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
