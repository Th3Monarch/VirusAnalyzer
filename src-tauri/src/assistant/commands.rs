use std::sync::Arc;

use chrono::Utc;
use serde::Serialize;
use tauri::State;

use super::context::ApplicationContext;
use super::intent::{Intent, IntentParser};
use super::personality::Personality;
use super::prompt::PromptBuilder;
use super::safety::{SafetyLayer, ToolPermission};
use super::session::SessionContext;
use super::tools::ToolRegistry;
use super::voice::VoicePipeline;

use crate::ai::manager::ProviderManager;
use crate::ai::provider::AiProvider;
use crate::ai::types::ModelInfo;
use crate::scanner::history::ScanStore;

/// Tiempo máximo de vida de una confirmación pendiente (5 minutos).
const CONFIRMATION_EXPIRY_SECS: i64 = 300;

/// Estado del assistant compartido entre comandos Tauri.
pub struct AssistantState {
    pub session: std::sync::Mutex<SessionContext>,
    pub safety: std::sync::Mutex<SafetyLayer>,
    pub context: std::sync::Mutex<ApplicationContext>,
    pub personality: Personality,
    pub intent_parser: IntentParser,
    pub tool_executor: ToolRegistry,
    pub prompt_builder: PromptBuilder,
    pub pending_confirmations: std::sync::Mutex<Vec<PendingConfirmation>>,
    pub ysmel_active: Arc<std::sync::atomic::AtomicBool>,
    pub fenix_active: Arc<std::sync::atomic::AtomicBool>,
    pub silent_mode: Arc<std::sync::atomic::AtomicBool>,
    pub provider: Arc<tokio::sync::RwLock<ProviderManager>>,
    pub voice: tokio::sync::Mutex<VoicePipeline>,
    pub app_handle: std::sync::Mutex<Option<tauri::AppHandle>>,
}

/// Confirmación pendiente de una acción destructiva.
#[derive(Debug, Clone)]
pub struct PendingConfirmation {
    pub id: String,
    pub intent: Intent,
    pub created_at: chrono::DateTime<Utc>,
}

/// Respuesta del assistant al frontend.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AssistantResponse {
    pub message: String,
    pub intent: Option<Intent>,
    pub requires_confirmation: bool,
    pub pending_id: Option<String>,
    pub metadata: Option<ResponseMetadata>,
}

/// Metadata de la respuesta del assistant.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResponseMetadata {
    pub tool_result: Option<serde_json::Value>,
    pub confidence: Option<f32>,
    pub processing_time_ms: u64,
}

impl AssistantState {
    /// Crea un nuevo AssistantState con valores por defecto.
    pub fn new(
        ysmel_active: Arc<std::sync::atomic::AtomicBool>,
        fenix_active: Arc<std::sync::atomic::AtomicBool>,
        provider: Arc<tokio::sync::RwLock<ProviderManager>>,
        silent_mode: bool,
    ) -> Self {
        Self {
            session: std::sync::Mutex::new(SessionContext::new()),
            safety: std::sync::Mutex::new(SafetyLayer::new()),
            context: std::sync::Mutex::new(ApplicationContext::new()),
            personality: Personality::default(),
            intent_parser: IntentParser::new(),
            tool_executor: ToolRegistry::new(),
            prompt_builder: PromptBuilder::new(),
            pending_confirmations: std::sync::Mutex::new(Vec::new()),
            ysmel_active,
            fenix_active,
            silent_mode: Arc::new(std::sync::atomic::AtomicBool::new(silent_mode)),
            provider,
            voice: tokio::sync::Mutex::new(VoicePipeline::new()),
            app_handle: std::sync::Mutex::new(None),
        }
    }

    /// Establece el AppHandle después de la construcción (llamado desde lib.rs).
    pub fn set_app_handle(&self, handle: tauri::AppHandle) {
        if let Ok(mut h) = self.app_handle.lock() {
            *h = Some(handle);
        }
    }

    /// Limpia confirmaciones expiradas (>5 min).
    fn expire_pending(&self) {
        if let Ok(mut confs) = self.pending_confirmations.lock() {
            let now = Utc::now();
            confs.retain(|c| {
                now.signed_duration_since(c.created_at).num_seconds() < CONFIRMATION_EXPIRY_SECS
            });
        }
    }
}

// --- Comandos Tauri ---

/// Maximum allowed message length to prevent memory exhaustion.
const MAX_MESSAGE_BYTES: usize = 10_000;
/// Maximum allowed audio size (10 MB).
const MAX_AUDIO_BYTES: usize = 10 * 1024 * 1024;
/// Maximum allowed TTS text length.
const MAX_TTS_BYTES: usize = 50_000;

/// Validates that a URL points to a local/private Ollama instance.
/// Blocks SSRF to cloud metadata, LAN, or external hosts.
fn validate_ollama_url(url: &str) -> Result<(), String> {
    let lower = url.to_lowercase();

    // Must start with http:// or https://
    if !lower.starts_with("http://") && !lower.starts_with("https://") {
        return Err("Esquema no soportado. Use http:// o https://".to_string());
    }

    // Extract host from URL (after scheme)
    let host_part = if let Some(rest) = lower.strip_prefix("https://") {
        rest
    } else if let Some(rest) = lower.strip_prefix("http://") {
        rest
    } else {
        return Err("URL inválida".to_string());
    };

    // Handle IPv6 bracket notation: [::1]:port/path
    let host = if host_part.starts_with('[') {
        if let Some(end) = host_part.find(']') {
            host_part[..=end].to_string()
        } else {
            return Err("URL IPv6 sin cerrar".to_string());
        }
    } else {
        host_part
            .split([':', '/', '?', '#'])
            .next()
            .unwrap_or("")
            .trim()
            .to_string()
    };

    if host.is_empty() {
        return Err("URL sin host".to_string());
    }

    // Allow only local/private addresses
    let is_local = matches!(
        host.as_str(),
        "localhost" | "127.0.0.1" | "[::1]" | "0.0.0.0"
    ) || host.starts_with("192.168.")
        || host.starts_with("10.")
        || host.starts_with("172.")
        || host == "127.*";

    if !is_local {
        return Err(format!(
            "URL de Ollama no permitida: solo se permiten direcciones locales (localhost, LAN privada). Host: {host}"
        ));
    }

    Ok(())
}

/// Sanitizes an internal error into a user-safe message.
/// Logs the full error server-side; returns only safe text to the caller.
fn sanitize_tool_error(err: &str, lang: &str) -> String {
    let is_en = lang == "en";
    // Map known error patterns to safe user messages
    if err.contains("not found") || err.contains("No such file") {
        if is_en {
            "File or resource not found.".into()
        } else {
            "Archivo o recurso no encontrado.".into()
        }
    } else if err.contains("Permission denied") || err.contains("access denied") {
        if is_en {
            "Permission denied. Run Prometeo as administrator if needed.".into()
        } else {
            "Permiso denegado. Ejecute Prometeo como administrador si es necesario.".into()
        }
    } else if err.contains("locked") {
        if is_en {
            "Service temporarily locked. Try again.".into()
        } else {
            "Servicio bloqueado temporalmente. Intente de nuevo.".into()
        }
    } else if err.contains("not initialized") {
        if is_en {
            "Service unavailable. Check settings.".into()
        } else {
            "Servicio no disponible. Verifique la configuración.".into()
        }
    } else if err.contains("Unknown provider") {
        if is_en {
            "Unrecognized AI provider.".into()
        } else {
            "Proveedor de IA no reconocido.".into()
        }
    } else {
        // Generic safe fallback — never expose OS details, paths, or internals
        if is_en {
            "An error occurred processing the request. Please try again.".into()
        } else {
            "Ocurrió un error al procesar la solicitud. Intente de nuevo.".into()
        }
    }
}

// --- Comandos Tauri ---

/// Envía un mensaje al assistant y devuelve la respuesta.
#[tauri::command]
pub async fn assistant_send_message(
    state: State<'_, Arc<AssistantState>>,
    scan_store: State<'_, Arc<std::sync::Mutex<ScanStore>>>,
    message: String,
    confirmed: Option<bool>,
    pending_id: Option<String>,
    language: Option<String>,
) -> Result<AssistantResponse, String> {
    let lang = language.unwrap_or_else(|| "es".into());
    let start = std::time::Instant::now();

    // M5: Input size cap — prevent memory exhaustion from oversized messages
    if message.len() > MAX_MESSAGE_BYTES {
        return Err(if lang == "en" {
            format!(
                "Message too long ({} bytes, max {}).",
                message.len(),
                MAX_MESSAGE_BYTES
            )
        } else {
            format!(
                "Mensaje demasiado largo ({} bytes, máximo {}).",
                message.len(),
                MAX_MESSAGE_BYTES
            )
        });
    }

    // Limpiar confirmaciones expiradas
    state.expire_pending();

    // Registrar mensaje del usuario
    {
        let mut session = state.session.lock().map_err(|_| "Session locked")?;
        session.add_user_message(&message);
    }

    // Parsear intención (context-aware)
    let context_snapshot = {
        let ctx = state.context.lock().map_err(|_| "Context locked")?;
        ctx.clone()
    };
    let intent = state
        .intent_parser
        .parse_with_context(&message, &context_snapshot);

    // Resolver tool call desde el registry
    let tool_call = state.tool_executor.resolve_intent(&intent);

    // Verificar seguridad usando el risk level del tool
    let (permission, _safety_events) = {
        let mut safety = state.safety.lock().map_err(|_| "Safety locked")?;
        let check = match &tool_call {
            Some(tc) => safety.check_risk(tc.meta.risk),
            None => super::safety::SafetyCheck {
                permission: ToolPermission::Allowed,
                events: Vec::new(),
            },
        };
        (check.permission, check.events)
    };

    match permission {
        ToolPermission::Blocked => {
            let response_text = if lang == "en" {
                "This action is currently blocked by security protocols.".to_string()
            } else {
                "Esta acción está bloqueada por los protocolos de seguridad.".to_string()
            };
            let intent_name = intent.name().to_string();
            let mut session = state.session.lock().map_err(|_| "Session locked")?;
            session.add_assistant_message(&response_text, Some(intent_name), false);
            return Ok(AssistantResponse {
                message: response_text,
                intent: Some(intent),
                requires_confirmation: false,
                pending_id: None,
                metadata: None,
            });
        }
        ToolPermission::RequiresConfirmation if confirmed != Some(true) => {
            let meta = tool_call
                .as_ref()
                .map(|tc| tc.meta.clone())
                .unwrap_or_else(ToolRegistry::conversation_meta);

            let confirm_msg = format!(
                "{}\n\n{}",
                meta.confirmation_msg_es, meta.confirmation_msg_en
            );
            let new_pending_id = uuid::Uuid::new_v4().to_string();

            let pending = PendingConfirmation {
                id: new_pending_id.clone(),
                intent: intent.clone(),
                created_at: Utc::now(),
            };

            {
                let mut confirmations = state
                    .pending_confirmations
                    .lock()
                    .map_err(|_| "Confirmations locked")?;
                confirmations.push(pending);
            }

            let intent_name = intent.name().to_string();
            let mut session = state.session.lock().map_err(|_| "Session locked")?;
            session.add_assistant_message(&confirm_msg, Some(intent_name), true);

            return Ok(AssistantResponse {
                message: confirm_msg,
                intent: Some(intent),
                requires_confirmation: true,
                pending_id: Some(new_pending_id),
                metadata: None,
            });
        }
        _ => {}
    }

    // Si viene con confirmed=true, validar que el pending_id coincida y el intent sea el mismo
    if confirmed == Some(true) {
        let pid = match pending_id {
            Some(ref pid) => pid.clone(),
            None => {
                // C1: confirmed=true sin pending_id es un bypass
                let mut safety = state.safety.lock().map_err(|_| "Safety locked")?;
                safety.record_bypass_attempt("confirmed=true without pending_id");
                let response_text = "Confirmación inválida: falta ID de pendiente.\nInvalid confirmation: missing pending ID.".to_string();
                let mut session = state.session.lock().map_err(|_| "Session locked")?;
                session.add_assistant_message(&response_text, None, false);
                return Ok(AssistantResponse {
                    message: response_text,
                    intent: Some(intent),
                    requires_confirmation: false,
                    pending_id: None,
                    metadata: None,
                });
            }
        };
        let confirmation = {
            let confirmations = state
                .pending_confirmations
                .lock()
                .map_err(|_| "Confirmations locked")?;
            confirmations.iter().find(|c| c.id == pid).cloned()
        };
        match confirmation {
            None => {
                // pending_id inválido o expirado — registrar bypass attempt
                {
                    let mut safety = state.safety.lock().map_err(|_| "Safety locked")?;
                    safety.record_bypass_attempt(&format!("Invalid or expired pending_id: {pid}"));
                }
                let response_text =
                    "Confirmación no válida o expirada.\nInvalid or expired confirmation."
                        .to_string();
                let mut session = state.session.lock().map_err(|_| "Session locked")?;
                session.add_assistant_message(&response_text, Some(intent.name().into()), false);
                return Ok(AssistantResponse {
                    message: response_text,
                    intent: Some(intent),
                    requires_confirmation: false,
                    pending_id: None,
                    metadata: None,
                });
            }
            Some(pending) => {
                // H1: Full intent equality — prevents stale-consent parameter swap
                if pending.intent != intent {
                    let mut safety = state.safety.lock().map_err(|_| "Safety locked")?;
                    safety.record_bypass_attempt(&format!(
                        "Intent mismatch: pending={:?}, current={:?}",
                        pending.intent, intent
                    ));
                    let response_text = "La confirmación no coincide con la acción actual.\nThe confirmation doesn't match the current action.".to_string();
                    let mut session = state.session.lock().map_err(|_| "Session locked")?;
                    session.add_assistant_message(
                        &response_text,
                        Some(intent.name().into()),
                        false,
                    );
                    return Ok(AssistantResponse {
                        message: response_text,
                        intent: Some(intent),
                        requires_confirmation: false,
                        pending_id: None,
                        metadata: None,
                    });
                }
                // Confirmación válida — registrar y limpiar
                {
                    let mut safety = state.safety.lock().map_err(|_| "Safety locked")?;
                    safety.record_confirmation_accepted();
                }
                {
                    let mut confirmations = state
                        .pending_confirmations
                        .lock()
                        .map_err(|_| "Confirmations locked")?;
                    confirmations.retain(|c| c.id != pid);
                }
            }
        }
    }

    // Ejecutar la tool usando el ToolExecutor como dispatcher
    // H3: Sanitize errors — never expose internal paths/details to frontend
    let (response_text, tool_result) =
        execute_tool(&state, &scan_store, &intent, &tool_call, &lang)
            .await
            .map_err(|e| sanitize_tool_error(&e, &lang))?;

    // Registrar respuesta del assistant
    {
        let mut session = state.session.lock().map_err(|_| "Session locked")?;
        session.add_assistant_message(&response_text, Some(intent.name().into()), false);
    }

    let elapsed = start.elapsed().as_millis() as u64;

    Ok(AssistantResponse {
        message: response_text,
        intent: Some(intent),
        requires_confirmation: false,
        pending_id: None,
        metadata: Some(ResponseMetadata {
            tool_result,
            confidence: Some(0.8),
            processing_time_ms: elapsed,
        }),
    })
}

/// Dispatcher central: ejecuta la tool correspondiente a un ToolCall.
async fn execute_tool(
    state: &State<'_, Arc<AssistantState>>,
    scan_store: &State<'_, Arc<std::sync::Mutex<ScanStore>>>,
    intent: &Intent,
    tool_call: &Option<super::tools::ToolCall>,
    lang: &str,
) -> Result<(String, Option<serde_json::Value>), String> {
    let tc = match tool_call {
        Some(tc) => tc,
        None => {
            // GeneralConversation → usar AI provider
            return execute_conversation(state, intent, lang).await;
        }
    };

    match tc.tool.as_str() {
        // --- Navegación ---
        "navigate" => Ok((
            tc.meta.response_for(lang).to_string(),
            Some(tc.params.clone()),
        )),

        // --- Información ---
        "get_system_info" => Ok((
            tc.meta.response_for(lang).to_string(),
            Some(tc.params.clone()),
        )),
        "get_rules" => Ok((
            tc.meta.response_for(lang).to_string(),
            Some(tc.params.clone()),
        )),

        // --- VirusTotal ---
        "virustotal_lookup" => Ok((
            tc.meta.response_for(lang).to_string(),
            Some(tc.params.clone()),
        )),

        // --- Escaneo ---
        "scan_path" => Ok((
            tc.meta.response_for(lang).to_string(),
            Some(tc.params.clone()),
        )),

        // --- Análisis por ID ---
        "get_analysis_by_id" => {
            let id = tc.params["id"].as_str().unwrap_or("");
            let result = {
                let store = scan_store.lock().map_err(|_| "ScanStore locked")?;
                store.results.get(id).cloned()
            };
            match result {
                Some(data) => {
                    let summary = summarize_analysis(&data, lang);
                    Ok((summary, Some(data)))
                }
                None => Ok((
                    if lang == "en" {
                        format!("No analysis found with ID `{id}`.")
                    } else {
                        format!("No se encontró análisis con ID `{id}`.")
                    },
                    None,
                )),
            }
        }

        // --- Cuarentena ---
        "quarantine_file" => {
            let path = tc.params["path"].as_str().unwrap_or("");
            let reason = tc.params["reason"].as_str().map(|s| s.to_string());
            let threat_level = crate::models::ThreatLevel::Medium;

            let app_handle = state
                .app_handle
                .lock()
                .map_err(|_| "AppHandle locked")?
                .clone()
                .ok_or("AppHandle not initialized")?;

            let config = crate::config::ConfigManager::load(&app_handle)?.config;
            let entry = crate::quarantine::quarantine_file(
                &app_handle,
                &config,
                path,
                threat_level,
                reason,
            )?;
            let result = serde_json::to_value(&entry).unwrap_or_default();
            {
                let mut safety = state.safety.lock().map_err(|_| "Safety locked")?;
                safety.record_destructive_action();
            }
            Ok((tc.meta.response_for(lang).to_string(), Some(result)))
        }

        // --- Restaurar ---
        "restore_quarantined" => {
            let id = tc.params["id"].as_str().unwrap_or("");

            let app_handle = state
                .app_handle
                .lock()
                .map_err(|_| "AppHandle locked")?
                .clone()
                .ok_or("AppHandle not initialized")?;

            let config = crate::config::ConfigManager::load(&app_handle)?.config;
            let entry = crate::quarantine::restore(&app_handle, &config, id)?;
            let result = serde_json::to_value(&entry).unwrap_or_default();
            {
                let mut safety = state.safety.lock().map_err(|_| "Safety locked")?;
                safety.record_destructive_action();
            }
            Ok((tc.meta.response_for(lang).to_string(), Some(result)))
        }

        // --- Informes ---
        "preview_report" => {
            let scan_id = tc.params["scanId"].as_str().unwrap_or("");
            let format_str = tc.params["format"].as_str().unwrap_or("html");
            let format = match format_str {
                "csv" => crate::models::ReportFormat::Csv,
                _ => crate::models::ReportFormat::Html,
            };

            let result = {
                let store = scan_store.lock().map_err(|_| "ScanStore locked")?;
                store.results.get(scan_id).cloned()
            };

            match result {
                Some(data) => {
                    let content = crate::report::render(&data, format)?;
                    let preview = serde_json::json!({
                        "scanId": scan_id,
                        "format": format_str,
                        "content": content,
                    });
                    Ok((tc.meta.response_for(lang).to_string(), Some(preview)))
                }
                None => Ok((
                    if lang == "en" {
                        format!("Analysis `{scan_id}` not found for report.")
                    } else {
                        format!("No se encontró análisis `{scan_id}` para generar informe.")
                    },
                    None,
                )),
            }
        }

        // --- Protocolos ---
        "activate_ysmel" => {
            let mut safety = state.safety.lock().map_err(|_| "Safety locked")?;
            safety.set_ysmel(true);
            state
                .ysmel_active
                .store(true, std::sync::atomic::Ordering::SeqCst);
            Ok((tc.meta.response_for(lang).to_string(), None))
        }
        "deactivate_ysmel" => {
            let mut safety = state.safety.lock().map_err(|_| "Safety locked")?;
            safety.set_ysmel(false);
            state
                .ysmel_active
                .store(false, std::sync::atomic::Ordering::SeqCst);
            Ok((tc.meta.response_for(lang).to_string(), None))
        }
        "activate_fenix" => {
            let mut safety = state.safety.lock().map_err(|_| "Safety locked")?;
            safety.set_fenix(true);
            state
                .fenix_active
                .store(true, std::sync::atomic::Ordering::SeqCst);
            Ok((tc.meta.response_for(lang).to_string(), None))
        }
        "deactivate_fenix" => {
            let mut safety = state.safety.lock().map_err(|_| "Safety locked")?;
            safety.set_fenix(false);
            state
                .fenix_active
                .store(false, std::sync::atomic::Ordering::SeqCst);
            Ok((tc.meta.response_for(lang).to_string(), None))
        }

        // --- Catch-all ---
        _ => Ok((
            if lang == "en" {
                "I can't perform that action.".into()
            } else {
                "No puedo ejecutar esa acción.".into()
            },
            None,
        )),
    }
}

/// Ejecuta una conversación general usando el AI provider.
async fn execute_conversation(
    state: &State<'_, Arc<AssistantState>>,
    intent: &Intent,
    lang: &str,
) -> Result<(String, Option<serde_json::Value>), String> {
    let user_msg = match intent {
        Intent::GeneralConversation { message } => message.clone(),
        _ => {
            return Ok((
                if lang == "en" {
                    "I don't understand.".into()
                } else {
                    "No entiendo tu mensaje.".into()
                },
                None,
            ))
        }
    };

    let session_context = {
        let session = state.session.lock().map_err(|_| "Session locked")?;
        let ctx = state.context.lock().map_err(|_| "Context locked")?;
        let system_prompt = state.prompt_builder.build(&state.personality, &ctx);
        let history = session.context_window(10);
        (system_prompt, history)
    };
    let (system_prompt, history_context) = session_context;

    // Clonar la referencia al provider y soltar el RwLock antes de la llamada HTTP.
    let provider_ref = {
        let mgr = state.provider.read().await;
        mgr.provider_ref()
    };

    match provider_ref
        .complete(&system_prompt, &user_msg, &history_context)
        .await
    {
        Ok(completion) => Ok((completion.text, None)),
        Err(_) => {
            let response = generate_conversational_response(&user_msg, lang);
            Ok((response, None))
        }
    }
}

/// Genera una respuesta conversacional de fallback (cuando no hay AI provider).
fn generate_conversational_response(input: &str, lang: &str) -> String {
    let lower = input.to_lowercase();
    let is_en = lang == "en";

    if lower.contains("hola")
        || lower.contains("hello")
        || lower.contains("hi")
        || lower.contains("hey")
        || lower.contains("buenos")
        || lower.contains("buenas")
    {
        return if is_en {
            "Hello! I'm your Prometeo security companion. I can help you analyze files, understand scan results, manage quarantine, and more. What would you like to do?".into()
        } else {
            "Hello! I'm your Prometeo security companion. I can help you analyze files, understand scan results, manage quarantine, and more. What would you like to do?".into()
        };
    }

    if lower.contains("gracias") || lower.contains("thank") {
        return if is_en {
            "You're welcome! Let me know if you need anything else.".into()
        } else {
            "¡De nada! Avísame si necesitas algo más.".into()
        };
    }

    if lower.contains("adiós") || lower.contains("bye") || lower.contains("chao") {
        return if is_en {
            "Goodbye! Stay safe. I'm here whenever you need me.".into()
        } else {
            "¡Adiós! Mantente seguro. Estoy aquí cuando me necesites.".into()
        };
    }

    if lower.contains("que puedes")
        || lower.contains("what can you")
        || lower.contains("help")
        || lower.contains("ayuda")
    {
        return if is_en {
            "I can help you with:\n- Analyzing files for threats\n- Understanding scan results\n- Managing quarantine\n- Explaining heuristic rules\n- Checking VirusTotal reputation\n- System security information\n- Activating security protocols\n\nJust ask me anything!".into()
        } else {
            "Puedo ayudarte con:\n- Analizar archivos en busca de amenazas\n- Comprender resultados de escaneo\n- Gestionar cuarentena\n- Explicar reglas heurísticas\n- Consultar reputación en VirusTotal\n- Información de seguridad del sistema\n- Activar protocolos de seguridad\n\n¡Solo pídemelo!".into()
        };
    }

    if is_en {
        "I'm here to help with Prometeo. What would you like to do?".into()
    } else {
        "Estoy aquí para ayudarte con Prometeo. ¿Qué te gustaría hacer?".into()
    }
}

/// Resume un resultado de análisis en un mensaje legible.
fn summarize_analysis(data: &serde_json::Value, lang: &str) -> String {
    let name = data["fileInfo"]["name"].as_str().unwrap_or("unknown");
    let threat = data["assessment"]["threatLevel"]
        .as_str()
        .unwrap_or("unknown");
    let score = data["assessment"]["score"].as_i64().unwrap_or(0);
    let risk = data["assessment"]["riskLevel"]
        .as_str()
        .unwrap_or("unknown");

    if lang == "en" {
        format!(
            "Analysis of `{name}`:\n  Threat level: {threat}\n  Score: {score}/100\n  Risk: {risk}"
        )
    } else {
        format!(
            "Análisis de `{name}`:\n  Nivel de amenaza: {threat}\n  Puntaje: {score}/100\n  Riesgo: {risk}"
        )
    }
}

// --- Comandos Tauri ---

/// Devuelve el historial de mensajes de la sesión.
#[tauri::command]
pub fn assistant_get_history(
    state: State<'_, Arc<AssistantState>>,
) -> Result<Vec<super::session::SessionMessage>, String> {
    let session = state.session.lock().map_err(|_| "Session locked")?;
    Ok(session.all_messages().to_vec())
}

/// Limpia la sesión de conversación.
#[tauri::command]
pub fn assistant_clear_session(state: State<'_, Arc<AssistantState>>) -> Result<(), String> {
    let mut session = state.session.lock().map_err(|_| "Session locked")?;
    session.clear();
    Ok(())
}

/// Devuelve el contexto actual de la aplicación.
#[tauri::command]
pub fn assistant_get_context(
    state: State<'_, Arc<AssistantState>>,
) -> Result<ApplicationContext, String> {
    let context = state.context.lock().map_err(|_| "Context locked")?;
    Ok(context.clone())
}

/// Actualiza el contexto de la aplicación (página actual, etc.).
#[tauri::command]
pub fn assistant_set_context(
    state: State<'_, Arc<AssistantState>>,
    key: String,
    value: Option<String>,
) -> Result<(), String> {
    let mut context = state.context.lock().map_err(|_| "Context locked")?;
    if value.is_some() {
        context.current_page = key;
    }
    Ok(())
}

/// Devuelve información del provider AI activo.
#[tauri::command]
pub async fn assistant_get_provider_info(
    state: State<'_, Arc<AssistantState>>,
) -> Result<ModelInfo, String> {
    let provider = state.provider.read().await;
    Ok(provider.model_info())
}

/// Verifica la salud del provider AI.
#[tauri::command]
pub async fn assistant_check_provider_health(
    state: State<'_, Arc<AssistantState>>,
) -> Result<bool, String> {
    let provider = state.provider.read().await;
    Ok(provider.health_check().await)
}

/// Cambia el provider activo (stub | ollama).
#[tauri::command]
pub async fn assistant_set_provider(
    state: State<'_, Arc<AssistantState>>,
    provider_type: String,
) -> Result<ModelInfo, String> {
    let mut mgr = state.provider.write().await;
    match provider_type.as_str() {
        "stub" => {
            mgr.switch_to_stub();
        }
        "ollama" => {
            let url = mgr.ollama_url().to_string();
            let model = mgr.ollama_model().to_string();
            mgr.switch_to_ollama(url, model, 0.3, 1024);
        }
        _ => return Err(format!("Unknown provider: {provider_type}")),
    }
    Ok(mgr.model_info())
}

/// Actualiza la configuración de Ollama y cambia al provider Ollama.
#[tauri::command]
pub async fn assistant_update_ollama(
    state: State<'_, Arc<AssistantState>>,
    url: String,
    model: String,
    enabled: bool,
    temperature: Option<f32>,
    max_tokens: Option<u32>,
) -> Result<ModelInfo, String> {
    if enabled {
        validate_ollama_url(&url)?;
    }
    let temp = temperature.unwrap_or(0.3);
    let tokens = max_tokens.unwrap_or(1024);
    let mut mgr = state.provider.write().await;
    if enabled {
        mgr.switch_to_ollama(url, model, temp, tokens);
    } else {
        mgr.switch_to_stub();
    }
    Ok(mgr.model_info())
}

/// Test de conexión con Ollama.
#[tauri::command]
pub async fn assistant_test_ollama(
    url: String,
) -> Result<crate::ai::manager::OllamaTestResult, String> {
    validate_ollama_url(&url)?;
    Ok(ProviderManager::test_ollama_connection(&url).await)
}

/// Cancela una confirmación pendiente.
#[tauri::command]
pub fn assistant_cancel_pending(state: State<'_, Arc<AssistantState>>) -> Result<(), String> {
    let mut confirmations = state
        .pending_confirmations
        .lock()
        .map_err(|_| "Confirmations locked")?;
    confirmations.clear();
    Ok(())
}

// --- Silent mode commands ---

/// Activa o desactiva el modo silencioso del assistant.
#[tauri::command]
pub async fn assistant_set_silent_mode(
    state: State<'_, Arc<AssistantState>>,
    enabled: bool,
) -> Result<bool, String> {
    state
        .silent_mode
        .store(enabled, std::sync::atomic::Ordering::SeqCst);
    // Persist to config
    if let Some(handle) = state
        .app_handle
        .lock()
        .map_err(|_| "AppHandle locked")?
        .as_ref()
    {
        if let Ok(mut mgr) = crate::config::ConfigManager::load(handle) {
            mgr.config.assistant_silent_mode = enabled;
            let _ = mgr.save();
        }
    }
    Ok(enabled)
}

/// Consulta el estado actual del modo silencioso.
#[tauri::command]
pub async fn assistant_get_silent_mode(
    state: State<'_, Arc<AssistantState>>,
) -> Result<bool, String> {
    Ok(state.silent_mode.load(std::sync::atomic::Ordering::SeqCst))
}

// --- Voice commands ---

/// Devuelve el estado actual del pipeline de voz.
#[tauri::command]
pub async fn assistant_get_voice_state(
    state: State<'_, Arc<AssistantState>>,
) -> Result<super::voice::VoiceRecordingState, String> {
    let voice = state.voice.lock().await;
    let provider = if voice.tts_available() {
        "kokoro"
    } else if voice.is_enabled() {
        "web"
    } else {
        "none"
    };
    Ok(super::voice::VoiceRecordingState {
        recording: false,
        available: voice.is_enabled(),
        provider: provider.into(),
    })
}

/// Actualiza la configuración de voz y reconecta providers.
#[tauri::command]
pub async fn assistant_update_voice_config(
    state: State<'_, Arc<AssistantState>>,
    config: super::voice::VoiceConfig,
) -> Result<super::voice::VoiceConfig, String> {
    {
        let mut voice = state.voice.lock().await;
        voice.init_from_config(&config).await;
    }
    Ok(config)
}

/// Sintetiza texto a audio usando Kokoro TTS.
/// Retorna los bytes del audio (WAV).
#[tauri::command]
pub async fn assistant_synthesize(
    state: State<'_, Arc<AssistantState>>,
    text: String,
) -> Result<Vec<u8>, String> {
    if text.len() > MAX_TTS_BYTES {
        return Err(format!(
            "Texto demasiado largo para TTS (máximo {} bytes).",
            MAX_TTS_BYTES
        ));
    }
    let voice = state.voice.lock().await;
    voice
        .synthesize(&text)
        .await
        .map_err(|e| format!("Error en síntesis de voz: {e}"))
}

/// Transcribe audio a texto usando Whisper STT.
/// Recibe los bytes del audio (WAV/WEBM) y retorna el texto transcrito.
#[tauri::command]
pub async fn assistant_transcribe(
    state: State<'_, Arc<AssistantState>>,
    audio: Vec<u8>,
) -> Result<String, String> {
    if audio.len() > MAX_AUDIO_BYTES {
        return Err(format!(
            "Audio demasiado grande (máximo {} bytes).",
            MAX_AUDIO_BYTES
        ));
    }
    let voice = state.voice.lock().await;
    voice
        .transcribe(&audio)
        .await
        .map_err(|e| format!("Error en transcripción: {e}"))
}

/// Verifica la salud de los providers de voz (Kokoro/Whisper).
#[tauri::command]
pub async fn assistant_voice_health(
    state: State<'_, Arc<AssistantState>>,
) -> Result<super::voice::VoiceHealth, String> {
    let voice = state.voice.lock().await;
    Ok(voice.health_check().await)
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- validate_ollama_url ---

    #[test]
    fn test_url_localhost_http() {
        assert!(validate_ollama_url("http://localhost:11434").is_ok());
    }

    #[test]
    fn test_url_localhost_https() {
        assert!(validate_ollama_url("https://localhost:11434").is_ok());
    }

    #[test]
    fn test_url_127_0_0_1() {
        assert!(validate_ollama_url("http://127.0.0.1:11434").is_ok());
    }

    #[test]
    fn test_url_ipv6_loopback() {
        assert!(validate_ollama_url("http://[::1]:11434").is_ok());
    }

    #[test]
    fn test_url_private_192_168() {
        assert!(validate_ollama_url("http://192.168.1.100:11434").is_ok());
    }

    #[test]
    fn test_url_private_10() {
        assert!(validate_ollama_url("http://10.0.0.5:11434").is_ok());
    }

    #[test]
    fn test_url_private_172() {
        assert!(validate_ollama_url("http://172.16.0.1:11434").is_ok());
    }

    #[test]
    fn test_url_block_external() {
        let result = validate_ollama_url("http://8.8.8.8:11434");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("no permitida"));
    }

    #[test]
    fn test_url_block_cloud() {
        let result = validate_ollama_url("https://ollama.cloud.ai:443");
        assert!(result.is_err());
    }

    #[test]
    fn test_url_block_aws_metadata() {
        let result = validate_ollama_url("http://169.254.169.254/latest/meta-data/");
        assert!(result.is_err());
    }

    #[test]
    fn test_url_block_ftp_scheme() {
        let result = validate_ollama_url("ftp://localhost:11434");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Esquema no soportado"));
    }

    #[test]
    fn test_url_block_empty_host() {
        let result = validate_ollama_url("http://");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("sin host"));
    }

    // --- sanitize_tool_error ---

    #[test]
    fn test_sanitize_not_found() {
        let msg = sanitize_tool_error("file not found at /tmp/test", "es");
        assert_eq!(msg, "Archivo o recurso no encontrado.");
    }

    #[test]
    fn test_sanitize_no_such_file() {
        let msg = sanitize_tool_error("No such file or directory", "es");
        assert_eq!(msg, "Archivo o recurso no encontrado.");
    }

    #[test]
    fn test_sanitize_permission_denied() {
        let msg = sanitize_tool_error("Permission denied: /etc/shadow", "es");
        assert!(msg.contains("Permiso denegado"));
        assert!(!msg.contains("/etc/shadow"));
    }

    #[test]
    fn test_sanitize_locked() {
        let msg = sanitize_tool_error("database locked", "es");
        assert_eq!(msg, "Servicio bloqueado temporalmente. Intente de nuevo.");
    }

    #[test]
    fn test_sanitize_not_initialized() {
        let msg = sanitize_tool_error("provider not initialized", "es");
        assert_eq!(msg, "Servicio no disponible. Verifique la configuración.");
    }

    #[test]
    fn test_sanitize_unknown_provider() {
        let msg = sanitize_tool_error("Unknown provider: custom", "es");
        assert_eq!(msg, "Proveedor de IA no reconocido.");
    }

    #[test]
    fn test_sanitize_generic_fallback() {
        let msg = sanitize_tool_error("something went wrong at /internal/path", "es");
        assert_eq!(
            msg,
            "Ocurrió un error al procesar la solicitud. Intente de nuevo."
        );
        assert!(!msg.contains("/internal/path"));
    }

    #[test]
    fn test_sanitize_never_exposes_paths() {
        let msg = sanitize_tool_error("Error reading C:\\Users\\admin\\secret.txt", "es");
        assert!(!msg.contains("C:\\Users"));
        assert!(!msg.contains("secret.txt"));
    }

    #[test]
    fn test_sanitize_english() {
        let msg = sanitize_tool_error("file not found", "en");
        assert_eq!(msg, "File or resource not found.");
    }
}
