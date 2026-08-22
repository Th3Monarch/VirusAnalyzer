use super::intent::Intent;

/// Nivel de riesgo de una tool.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RiskLevel {
    /// Lectura pura, siempre permitido.
    None,
    /// Requiere confirmación del usuario.
    Medium,
    /// Requiere confirmación y se bloquea en Fenix.
    High,
    /// Siempre bloqueado (requiere migración a High + confirmación).
    #[allow(dead_code)]
    Critical,
}

/// Metadata de una tool registrada.
#[derive(Debug, Clone)]
pub struct ToolMeta {
    pub risk: RiskLevel,
    pub confirmation_msg_es: String,
    pub confirmation_msg_en: String,
    pub response_es: String,
    pub response_en: String,
}

/// Llamada a herramienta preparada para ejecución.
#[derive(Debug, Clone)]
pub struct ToolCall {
    pub tool: String,
    pub params: serde_json::Value,
    pub meta: ToolMeta,
}

impl ToolMeta {
    /// Devuelve la respuesta en el idioma dado.
    pub fn response_for(&self, lang: &str) -> &str {
        if lang == "en" {
            &self.response_en
        } else {
            &self.response_es
        }
    }
}

/// Registry central de tools. Source of truth para risk levels,
/// mensajes de confirmación y respuestas.
///
/// `ToolRegistry::resolve_intent` es la ÚNICA función que mapea
/// Intent → ToolCall. Todo el resto del sistema consume esto.
pub struct ToolRegistry;

impl ToolRegistry {
    /// Crea un registry con todas las tools registradas.
    pub fn new() -> Self {
        Self
    }

    /// Resuelve un Intent en un ToolCall con metadata completa.
    /// Devuelve None solo para GeneralConversation / Unknown.
    pub fn resolve_intent(&self, intent: &Intent) -> Option<ToolCall> {
        match intent {
            // --- Lectura: risk None ---
            Intent::AnalyzeFile { path } => Some(ToolCall {
                tool: "scan_path".into(),
                params: serde_json::json!({ "path": path }),
                meta: ToolMeta {
                    risk: RiskLevel::None,
                    confirmation_msg_es: String::new(),
                    confirmation_msg_en: String::new(),
                    response_es: format!("Voy a analizar `{path}`. Esto puede tomar un momento."),
                    response_en: format!("I'll analyze `{path}`. This may take a moment."),
                },
            }),
            Intent::GetAnalysis { id } => Some(ToolCall {
                tool: "get_analysis_by_id".into(),
                params: serde_json::json!({ "id": id }),
                meta: ToolMeta {
                    risk: RiskLevel::None,
                    confirmation_msg_es: String::new(),
                    confirmation_msg_en: String::new(),
                    response_es: format!("Buscando resultado del análisis `{id}`..."),
                    response_en: format!("Looking up analysis result `{id}`..."),
                },
            }),
            Intent::OpenHistory => Some(ToolCall {
                tool: "navigate".into(),
                params: serde_json::json!({ "page": "results" }),
                meta: ToolMeta {
                    risk: RiskLevel::None,
                    confirmation_msg_es: String::new(),
                    confirmation_msg_en: String::new(),
                    response_es: "Abriendo historial de escaneos...".into(),
                    response_en: "Opening scan history...".into(),
                },
            }),
            Intent::OpenQuarantine => Some(ToolCall {
                tool: "navigate".into(),
                params: serde_json::json!({ "page": "quarantine" }),
                meta: ToolMeta {
                    risk: RiskLevel::None,
                    confirmation_msg_es: String::new(),
                    confirmation_msg_en: String::new(),
                    response_es: "Abriendo cuarentena...".into(),
                    response_en: "Opening quarantine...".into(),
                },
            }),
            Intent::GenerateReport { scan_id, format } => Some(ToolCall {
                tool: "preview_report".into(),
                params: serde_json::json!({ "scanId": scan_id, "format": format }),
                meta: ToolMeta {
                    risk: RiskLevel::None,
                    confirmation_msg_es: String::new(),
                    confirmation_msg_en: String::new(),
                    response_es: format!("Generando informe para análisis `{scan_id}`..."),
                    response_en: format!("Generating report for analysis `{scan_id}`..."),
                },
            }),
            Intent::QueryVirusTotal { hash } => Some(ToolCall {
                tool: "virustotal_lookup".into(),
                params: serde_json::json!({ "hash": hash }),
                meta: ToolMeta {
                    risk: RiskLevel::None,
                    confirmation_msg_es: String::new(),
                    confirmation_msg_en: String::new(),
                    response_es: format!("Consultando reputación en VirusTotal: `{hash}`."),
                    response_en: format!("Querying VirusTotal reputation: `{hash}`."),
                },
            }),
            Intent::GetSystemInfo => Some(ToolCall {
                tool: "get_system_info".into(),
                params: serde_json::json!({}),
                meta: ToolMeta {
                    risk: RiskLevel::None,
                    confirmation_msg_es: String::new(),
                    confirmation_msg_en: String::new(),
                    response_es: "Obteniendo información del sistema...".into(),
                    response_en: "Getting system information...".into(),
                },
            }),
            Intent::GetRules => Some(ToolCall {
                tool: "get_rules".into(),
                params: serde_json::json!({}),
                meta: ToolMeta {
                    risk: RiskLevel::None,
                    confirmation_msg_es: String::new(),
                    confirmation_msg_en: String::new(),
                    response_es: "Mostrando catálogo de reglas heurísticas...".into(),
                    response_en: "Showing heuristic rules catalog...".into(),
                },
            }),

            // --- Destructivas: risk High (requieren confirmación + bloqueadas en Fenix) ---
            Intent::QuarantineFile { path, reason } => Some(ToolCall {
                tool: "quarantine_file".into(),
                params: serde_json::json!({ "path": path, "reason": reason }),
                meta: ToolMeta {
                    risk: RiskLevel::High,
                    confirmation_msg_es: format!(
                        "¿Aislar en cuarentena?\nArchivo: {path}\nMotivo: {}",
                        reason.as_deref().unwrap_or("sin motivo")
                    ),
                    confirmation_msg_en: format!(
                        "Quarantine file?\nFile: {path}\nReason: {}",
                        reason.as_deref().unwrap_or("none")
                    ),
                    response_es: format!("Archivo `{path}` aislado en cuarentena."),
                    response_en: format!("File `{path}` quarantined."),
                },
            }),
            Intent::RestoreFile { id } => Some(ToolCall {
                tool: "restore_quarantined".into(),
                params: serde_json::json!({ "id": id }),
                meta: ToolMeta {
                    risk: RiskLevel::High,
                    confirmation_msg_es: format!("¿Restaurar archivo de cuarentena?\nID: {id}"),
                    confirmation_msg_en: format!("Restore quarantined file?\nID: {id}"),
                    response_es: format!("Archivo `{id}` restaurado de cuarentena."),
                    response_en: format!("File `{id}` restored from quarantine."),
                },
            }),

            // --- Protocolos: risk Medium ---
            Intent::ActivateYsmel => Some(ToolCall {
                tool: "activate_ysmel".into(),
                params: serde_json::json!({}),
                meta: ToolMeta {
                    risk: RiskLevel::Medium,
                    confirmation_msg_es:
                        "¿Activar protocolo Ysmel?\nEsto aislará la red del sistema.".into(),
                    confirmation_msg_en:
                        "Activate Ysmel protocol?\nThis will isolate the system network.".into(),
                    response_es: "Protocolo Ysmel activado. Aislamiento de red habilitado.".into(),
                    response_en: "Ysmel protocol activated. Network isolation enabled.".into(),
                },
            }),
            Intent::DeactivateYsmel => Some(ToolCall {
                tool: "deactivate_ysmel".into(),
                params: serde_json::json!({}),
                meta: ToolMeta {
                    risk: RiskLevel::High,
                    confirmation_msg_es:
                        "¿Desactivar protocolo Ysmel?\nEsto restaurará la conectividad de red."
                            .into(),
                    confirmation_msg_en:
                        "Deactivate Ysmel protocol?\nThis will restore network connectivity.".into(),
                    response_es: "Protocolo Ysmel desactivado. Conectividad restaurada.".into(),
                    response_en: "Ysmel protocol deactivated. Network connectivity restored."
                        .into(),
                },
            }),
            Intent::ActivateFenix => Some(ToolCall {
                tool: "activate_fenix".into(),
                params: serde_json::json!({}),
                meta: ToolMeta {
                    risk: RiskLevel::Medium,
                    confirmation_msg_es:
                        "¿Activar protocolo Fenix?\nModo de recuperación de emergencia.".into(),
                    confirmation_msg_en: "Activate Fenix protocol?\nEmergency recovery mode."
                        .into(),
                    response_es: "Protocolo Fenix activado. Modo de recuperación habilitado."
                        .into(),
                    response_en: "Fenix protocol activated. Emergency recovery mode enabled."
                        .into(),
                },
            }),
            Intent::DeactivateFenix => Some(ToolCall {
                tool: "deactivate_fenix".into(),
                params: serde_json::json!({}),
                meta: ToolMeta {
                    risk: RiskLevel::Medium,
                    confirmation_msg_es: "¿Desactivar protocolo Fenix?".into(),
                    confirmation_msg_en: "Deactivate Fenix protocol?".into(),
                    response_es: "Protocolo Fenix desactivado.".into(),
                    response_en: "Fenix protocol deactivated.".into(),
                },
            }),

            // --- No requieren tool ---
            Intent::GeneralConversation { .. } | Intent::Unknown => None,
        }
    }

    /// Devuelve un ToolMeta por defecto para conversación general.
    pub fn conversation_meta() -> ToolMeta {
        ToolMeta {
            risk: RiskLevel::None,
            confirmation_msg_es: String::new(),
            confirmation_msg_en: String::new(),
            response_es: String::new(),
            response_en: String::new(),
        }
    }
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::super::context::ApplicationContext;
    use super::super::intent::IntentParser;
    use super::super::safety::{SafetyLayer, ToolPermission};
    use super::*;

    #[test]
    fn test_analyze_is_none_risk() {
        let reg = ToolRegistry::new();
        let tc = reg
            .resolve_intent(&Intent::AnalyzeFile {
                path: "test.exe".into(),
            })
            .unwrap();
        assert_eq!(tc.meta.risk, RiskLevel::None);
    }

    #[test]
    fn test_quarantine_is_high_risk() {
        let reg = ToolRegistry::new();
        let tc = reg
            .resolve_intent(&Intent::QuarantineFile {
                path: "bad.exe".into(),
                reason: None,
            })
            .unwrap();
        assert_eq!(tc.meta.risk, RiskLevel::High);
    }

    #[test]
    fn test_general_conversation_returns_none() {
        let reg = ToolRegistry::new();
        assert!(reg
            .resolve_intent(&Intent::GeneralConversation {
                message: "hi".into()
            })
            .is_none());
    }

    #[test]
    fn test_unknown_returns_none() {
        let reg = ToolRegistry::new();
        assert!(reg.resolve_intent(&Intent::Unknown).is_none());
    }

    #[test]
    fn test_get_analysis_is_none_risk() {
        let reg = ToolRegistry::new();
        let tc = reg
            .resolve_intent(&Intent::GetAnalysis {
                id: "test-id".into(),
            })
            .unwrap();
        assert_eq!(tc.meta.risk, RiskLevel::None);
        assert_eq!(tc.tool, "get_analysis_by_id");
    }

    #[test]
    fn test_open_history_navigates() {
        let reg = ToolRegistry::new();
        let tc = reg.resolve_intent(&Intent::OpenHistory).unwrap();
        assert_eq!(tc.tool, "navigate");
        assert_eq!(tc.params["page"], "results");
    }

    #[test]
    fn test_open_quarantine_navigates() {
        let reg = ToolRegistry::new();
        let tc = reg.resolve_intent(&Intent::OpenQuarantine).unwrap();
        assert_eq!(tc.tool, "navigate");
        assert_eq!(tc.params["page"], "quarantine");
    }

    #[test]
    fn test_generate_report() {
        let reg = ToolRegistry::new();
        let tc = reg
            .resolve_intent(&Intent::GenerateReport {
                scan_id: "scan-1".into(),
                format: Some("pdf".into()),
            })
            .unwrap();
        assert_eq!(tc.tool, "preview_report");
        assert_eq!(tc.meta.risk, RiskLevel::None);
    }

    #[test]
    fn test_query_virustotal() {
        let reg = ToolRegistry::new();
        let tc = reg
            .resolve_intent(&Intent::QueryVirusTotal {
                hash: "abc123".into(),
            })
            .unwrap();
        assert_eq!(tc.tool, "virustotal_lookup");
        assert_eq!(tc.meta.risk, RiskLevel::None);
    }

    #[test]
    fn test_get_system_info() {
        let reg = ToolRegistry::new();
        let tc = reg.resolve_intent(&Intent::GetSystemInfo).unwrap();
        assert_eq!(tc.tool, "get_system_info");
    }

    #[test]
    fn test_get_rules() {
        let reg = ToolRegistry::new();
        let tc = reg.resolve_intent(&Intent::GetRules).unwrap();
        assert_eq!(tc.tool, "get_rules");
    }

    #[test]
    fn test_restore_file_is_high_risk() {
        let reg = ToolRegistry::new();
        let tc = reg
            .resolve_intent(&Intent::RestoreFile { id: "q-1".into() })
            .unwrap();
        assert_eq!(tc.meta.risk, RiskLevel::High);
        assert_eq!(tc.tool, "restore_quarantined");
    }

    #[test]
    fn test_activate_ysmel_medium_risk() {
        let reg = ToolRegistry::new();
        let tc = reg.resolve_intent(&Intent::ActivateYsmel).unwrap();
        assert_eq!(tc.meta.risk, RiskLevel::Medium);
        assert_eq!(tc.tool, "activate_ysmel");
    }

    #[test]
    fn test_deactivate_ysmel_high_risk() {
        let reg = ToolRegistry::new();
        let tc = reg.resolve_intent(&Intent::DeactivateYsmel).unwrap();
        assert_eq!(tc.meta.risk, RiskLevel::High);
    }

    #[test]
    fn test_activate_fenix_medium_risk() {
        let reg = ToolRegistry::new();
        let tc = reg.resolve_intent(&Intent::ActivateFenix).unwrap();
        assert_eq!(tc.meta.risk, RiskLevel::Medium);
    }

    #[test]
    fn test_deactivate_fenix_medium_risk() {
        let reg = ToolRegistry::new();
        let tc = reg.resolve_intent(&Intent::DeactivateFenix).unwrap();
        assert_eq!(tc.meta.risk, RiskLevel::Medium);
    }

    #[test]
    fn test_conversation_meta() {
        let meta = ToolRegistry::conversation_meta();
        assert_eq!(meta.risk, RiskLevel::None);
        assert!(meta.response_es.is_empty());
    }

    #[test]
    fn test_all_intents_have_confirmation_messages() {
        let reg = ToolRegistry::new();
        let intents: Vec<Intent> = vec![
            Intent::AnalyzeFile { path: "x".into() },
            Intent::GetAnalysis { id: "x".into() },
            Intent::OpenHistory,
            Intent::OpenQuarantine,
            Intent::GenerateReport {
                scan_id: "x".into(),
                format: Some("pdf".into()),
            },
            Intent::QueryVirusTotal { hash: "x".into() },
            Intent::GetSystemInfo,
            Intent::GetRules,
            Intent::QuarantineFile {
                path: "x".into(),
                reason: None,
            },
            Intent::RestoreFile { id: "x".into() },
            Intent::ActivateYsmel,
            Intent::DeactivateYsmel,
            Intent::ActivateFenix,
            Intent::DeactivateFenix,
        ];
        for intent in intents {
            let tc = reg
                .resolve_intent(&intent)
                .expect(&format!("missing tool for {:?}", intent));
            assert!(
                !tc.meta.response_es.is_empty(),
                "empty response_es for {:?}",
                intent
            );
            assert!(
                !tc.meta.response_en.is_empty(),
                "empty response_en for {:?}",
                intent
            );
            assert!(!tc.tool.is_empty(), "empty tool name for {:?}", intent);
        }
    }

    // --- Integration tests: full pipeline (parse → resolve → safety) ---

    #[test]
    fn test_pipeline_analyze_quoted_path() {
        let parser = IntentParser::new();
        let ctx = ApplicationContext::new();
        let intent = parser.parse_with_context("analizar \"C:\\malware.exe\"", &ctx);
        let reg = ToolRegistry::new();
        let tc = reg.resolve_intent(&intent).unwrap();
        assert_eq!(tc.tool, "scan_path");
        assert_eq!(tc.meta.risk, RiskLevel::None);
    }

    #[test]
    fn test_pipeline_quarantine_file_is_high_risk() {
        let parser = IntentParser::new();
        let ctx = ApplicationContext::new();
        let intent = parser.parse_with_context("cuarentena \"C:\\bad.exe\" porque malware", &ctx);
        let mut safety = SafetyLayer::new();
        let reg = ToolRegistry::new();
        let tc = reg.resolve_intent(&intent).unwrap();
        let check = safety.check_risk(tc.meta.risk);
        assert_eq!(check.permission, ToolPermission::RequiresConfirmation);
    }

    #[test]
    fn test_pipeline_quarantine_blocked_in_fenix() {
        let parser = IntentParser::new();
        let ctx = ApplicationContext::new();
        let intent = parser.parse_with_context("cuarentena \"C:\\bad.exe\"", &ctx);
        let mut safety = SafetyLayer::new();
        safety.set_fenix(true);
        let reg = ToolRegistry::new();
        let tc = reg.resolve_intent(&intent).unwrap();
        let check = safety.check_risk(tc.meta.risk);
        assert_eq!(check.permission, ToolPermission::Blocked);
    }

    #[test]
    fn test_pipeline_greeting_no_tool() {
        let parser = IntentParser::new();
        let ctx = ApplicationContext::new();
        let intent = parser.parse_with_context("hola", &ctx);
        let reg = ToolRegistry::new();
        let tc = reg.resolve_intent(&intent);
        assert!(tc.is_none());
    }

    #[test]
    fn test_pipeline_history_navigates() {
        let parser = IntentParser::new();
        let ctx = ApplicationContext::new();
        let intent = parser.parse_with_context("mostrar historial", &ctx);
        let reg = ToolRegistry::new();
        let tc = reg.resolve_intent(&intent).unwrap();
        assert_eq!(tc.tool, "navigate");
        assert_eq!(tc.meta.risk, RiskLevel::None);
    }

    #[test]
    fn test_pipeline_rate_limit_blocks_after_burst() {
        let parser = IntentParser::new();
        let ctx = ApplicationContext::new();
        let intent = parser.parse_with_context("cuarentena \"C:\\x.exe\"", &ctx);
        let mut safety = SafetyLayer::new();
        let reg = ToolRegistry::new();
        let tc = reg.resolve_intent(&intent).unwrap();

        for _ in 0..5 {
            safety.record_destructive_action();
        }
        let check = safety.check_risk(tc.meta.risk);
        assert_eq!(check.permission, ToolPermission::Blocked);
    }

    #[test]
    fn test_pipeline_english_query_virustotal() {
        let parser = IntentParser::new();
        let ctx = ApplicationContext::new();
        let intent = parser.parse_with_context("check hash d41d8cd98f00b204e9800998ecf8427e", &ctx);
        let reg = ToolRegistry::new();
        let tc = reg.resolve_intent(&intent).unwrap();
        assert_eq!(tc.tool, "virustotal_lookup");
        assert_eq!(tc.meta.risk, RiskLevel::None);
    }

    #[test]
    fn test_pipeline_activate_ysmel_medium_risk() {
        let parser = IntentParser::new();
        let ctx = ApplicationContext::new();
        let intent = parser.parse_with_context("activar ysmel", &ctx);
        let mut safety = SafetyLayer::new();
        let reg = ToolRegistry::new();
        let tc = reg.resolve_intent(&intent).unwrap();
        let check = safety.check_risk(tc.meta.risk);
        assert_eq!(check.permission, ToolPermission::RequiresConfirmation);
    }

    #[test]
    fn test_pipeline_deactivate_ysmel_blocked_in_fenix() {
        let parser = IntentParser::new();
        let ctx = ApplicationContext::new();
        let intent = parser.parse_with_context("desactivar ysmel", &ctx);
        let mut safety = SafetyLayer::new();
        safety.set_fenix(true);
        let reg = ToolRegistry::new();
        let tc = reg.resolve_intent(&intent).unwrap();
        let check = safety.check_risk(tc.meta.risk);
        assert_eq!(check.permission, ToolPermission::Blocked);
    }
}
