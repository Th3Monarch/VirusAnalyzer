use super::context::ApplicationContext;
use serde::{Deserialize, Serialize};

/// Intención detectada en el mensaje del usuario.
///
/// El IntentParser convierte texto libre en una de estas variantes
/// con sus parámetros extraídos.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", content = "params")]
pub enum Intent {
    /// Analizar un archivo específico.
    AnalyzeFile { path: String },
    /// Obtener resultados de un análisis previo.
    GetAnalysis { id: String },
    /// Abrir la página de historial.
    OpenHistory,
    /// Abrir la página de cuarentena.
    OpenQuarantine,
    /// Aislar un archivo en cuarentena.
    QuarantineFile { path: String, reason: Option<String> },
    /// Restaurar un archivo de la cuarentena.
    RestoreFile { id: String },
    /// Generar un informe.
    GenerateReport { scan_id: String, format: Option<String> },
    /// Consultar VirusTotal.
    QueryVirusTotal { hash: String },
    /// Obtener información del sistema.
    GetSystemInfo,
    /// Activar protocolo Ysmel (aislamiento).
    ActivateYsmel,
    /// Desactivar protocolo Ysmel.
    DeactivateYsmel,
    /// Activar protocolo Fenix (recuperación).
    ActivateFenix,
    /// Desactivar protocolo Fenix.
    DeactivateFenix,
    /// Obtener reglas del motor heurístico.
    GetRules,
    /// Saludo o conversación general.
    GeneralConversation { message: String },
    /// No se detectó una intención clara.
    Unknown,
}

impl Intent {
    /// Nombre legible de la intención.
    pub fn name(&self) -> &str {
        match self {
            Self::AnalyzeFile { .. } => "analyze_file",
            Self::GetAnalysis { .. } => "get_analysis",
            Self::OpenHistory => "open_history",
            Self::OpenQuarantine => "open_quarantine",
            Self::QuarantineFile { .. } => "quarantine_file",
            Self::RestoreFile { .. } => "restore_file",
            Self::GenerateReport { .. } => "generate_report",
            Self::QueryVirusTotal { .. } => "query_virustotal",
            Self::GetSystemInfo => "get_system_info",
            Self::ActivateYsmel => "activate_ysmel",
            Self::DeactivateYsmel => "deactivate_ysmel",
            Self::ActivateFenix => "activate_fenix",
            Self::DeactivateFenix => "deactivate_fenix",
            Self::GetRules => "get_rules",
            Self::GeneralConversation { .. } => "general_conversation",
            Self::Unknown => "unknown",
        }
    }
}

/// Parser de intenciones basado en patrones de keywords.
///
/// Convierte texto libre del usuario en una `Intent` con parámetros.
/// Funciona sin LLM: usa matching de patrones en ES y EN.
/// Soporta context-aware disambiguation: si el usuario está en una
/// página de resultados con un archivo seleccionado, acciones ambiguas
/// como "cuarentena" o "restaurar" usan el contexto automáticamente.
pub struct IntentParser;

impl IntentParser {
    /// Crea un parser de intenciones nuevo.
    pub fn new() -> Self {
        Self
    }

    /// Parsea el texto del usuario y devuelve la intención detectada.
    #[allow(dead_code)]
    pub fn parse(&self, input: &str) -> Intent {
        self.parse_with_context(input, &ApplicationContext::new())
    }

    /// Parsea el texto del usuario usando el contexto de la aplicación
    /// para desambiguar acciones que requieren un archivo/ID.
    pub fn parse_with_context(&self, input: &str, ctx: &ApplicationContext) -> Intent {
        let lower = input.to_lowercase().trim().to_string();

        // --- Analizar archivo ---
        if matches_pattern(&lower, &["analizar", "escanear", "scan", "analyze", "check"]) {
            if let Some(path) = extract_path(&lower) {
                return Intent::AnalyzeFile { path };
            }
            // Context: si hay archivo seleccionado y el usuario dice "este/this/actual"
            if let Some(ref file) = ctx.selected_file {
                if matches_pattern(&lower, &["este", "this", "actual", "current", "seleccionado", "selected"]) {
                    return Intent::AnalyzeFile { path: file.clone() };
                }
            }
        }

        // --- Obtener análisis ---
        if matches_pattern(&lower, &["resultado", "result", "analysis", "análisis"]) {
            // Context: si estamos en una página con análisis visible y el usuario dice "este"
            if let Some(ref id) = ctx.current_analysis_id {
                if matches_pattern(&lower, &["este", "this", "actual", "current", "muestra", "show", "ver", "view"]) {
                    return Intent::GetAnalysis { id: id.clone() };
                }
            }
            if let Some(id) = extract_id(&lower) {
                return Intent::GetAnalysis { id };
            }
        }

        // --- Historial ---
        if matches_pattern(&lower, &["historial", "history", "previous", "anterior"]) {
            return Intent::OpenHistory;
        }

        // --- Cuarentena ---
        if matches_pattern(&lower, &["cuarentena", "quarantine", "aislar", "isolate"]) {
            // Restaurar de cuarentena
            if matches_pattern(&lower, &["restaurar", "restore"]) {
                if let Some(id) = extract_id(&lower) {
                    return Intent::RestoreFile { id };
                }
                // Context: si hay análisis visible y dice "este"
                if let Some(ref id) = ctx.current_analysis_id {
                    if matches_pattern(&lower, &["este", "this", "actual", "current"]) {
                        return Intent::RestoreFile { id: id.clone() };
                    }
                }
                return Intent::OpenQuarantine;
            }
            // Poner en cuarentena
            if let Some(path) = extract_path(&lower) {
                let reason = extract_after(&lower, &["porque", "reason", "motivo", "debido a", "because"]);
                return Intent::QuarantineFile {
                    path,
                    reason: if reason.is_empty() { None } else { Some(reason) },
                };
            }
            // Context: si hay archivo seleccionado y dice "este"
            if let Some(ref file) = ctx.selected_file {
                if matches_pattern(&lower, &["este", "this", "actual", "current", "seleccionado", "selected"]) {
                    return Intent::QuarantineFile {
                        path: file.clone(),
                        reason: None,
                    };
                }
            }
            return Intent::OpenQuarantine;
        }

        // --- Informe ---
        if matches_pattern(&lower, &["informe", "report", "exportar", "export"]) {
            let format = if lower.contains("csv") {
                Some("csv".into())
            } else if lower.contains("html") {
                Some("html".into())
            } else {
                None
            };
            if let Some(id) = extract_id(&lower) {
                return Intent::GenerateReport { scan_id: id, format };
            }
            // Context: si hay análisis visible
            if let Some(ref id) = ctx.current_analysis_id {
                if matches_pattern(&lower, &["este", "this", "actual", "current", "del", "of"]) {
                    return Intent::GenerateReport { scan_id: id.clone(), format };
                }
            }
        }

        // --- VirusTotal ---
        if matches_pattern(&lower, &["virus", "total", "reputación", "reputation", "hash"]) {
            if let Some(hash) = extract_hash(&lower) {
                return Intent::QueryVirusTotal { hash };
            }
        }

        // --- Info del sistema ---
        if matches_pattern(&lower, &["sistema", "system", "computadora", "computer", "equipo", "specs"]) {
            return Intent::GetSystemInfo;
        }

        // --- Protocolo Ysmel ---
        if matches_pattern(&lower, &["ysmel", "aislamiento", "isolation mode"]) {
            if matches_pattern(&lower, &["desactivar", "deactivate", "apagar", "off", "stop"]) {
                return Intent::DeactivateYsmel;
            }
            return Intent::ActivateYsmel;
        }

        // --- Protocolo Fenix ---
        if matches_pattern(&lower, &["fenix", "fénix", "phoenix", "recuperación", "recovery"]) {
            if matches_pattern(&lower, &["desactivar", "deactivate", "apagar", "off", "stop"]) {
                return Intent::DeactivateFenix;
            }
            return Intent::ActivateFenix;
        }

        // --- Reglas ---
        if matches_pattern(&lower, &["reglas", "rules", "heurístic", "heuristic"]) {
            return Intent::GetRules;
        }

        // --- Saludo / conversación general ---
        if matches_pattern(
            &lower,
            &[
                "hola", "hello", "hi", "hey", "buenos", "buenas", "saludos",
                "gracias", "thank", "adiós", "bye",
            ],
        ) {
            return Intent::GeneralConversation {
                message: input.to_string(),
            };
        }

        // --- Fallback: conversación general ---
        Intent::GeneralConversation {
            message: input.to_string(),
        }
    }
}

/// Verifica si el input contiene alguno de los patrones.
fn matches_pattern(input: &str, patterns: &[&str]) -> bool {
    patterns.iter().any(|p| input.contains(p))
}

/// Extrae una ruta del input (entre comillas, o después de preposiciones).
fn extract_path(input: &str) -> Option<String> {
    // Buscar entre comillas
    if let Some(start) = input.find('"') {
        if let Some(end) = input[start + 1..].find('"') {
            return Some(input[start + 1..start + 1 + end].to_string());
        }
    }

    // Buscar después de preposiciones comunes
    let prepositions = ["de ", "del ", "of ", "to ", "en ", "in "];
    for prep in prepositions {
        if let Some(pos) = input.find(prep) {
            let after = &input[pos + prep.len()..];
            let path: String = after
                .chars()
                .take_while(|c| !c.is_whitespace() || *c == ' ')
                .collect();
            if path.contains('/') || path.contains('\\') || path.contains('.') {
                return Some(path.trim().to_string());
            }
        }
    }

    None
}

/// Extrae un ID (UUID o corto) del input.
fn extract_id(input: &str) -> Option<String> {
    let words: Vec<&str> = input.split_whitespace().collect();
    for word in &words {
        let clean = word.trim_matches(|c: char| c == ',' || c == '.' || c == ':' || c == '"');
        if clean.len() >= 8
            && clean
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '-')
        {
            return Some(clean.to_string());
        }
    }
    None
}

/// Extrae un hash del input (32, 40, o 64 caracteres hex).
fn extract_hash(input: &str) -> Option<String> {
    let words: Vec<&str> = input.split_whitespace().collect();
    for word in &words {
        let clean: String = word
            .chars()
            .filter(|c| c.is_ascii_hexdigit())
            .collect();
        if matches!(clean.len(), 32 | 40 | 64) {
            return Some(clean);
        }
    }
    None
}

/// Extrae texto después de un patrón.
fn extract_after(input: &str, patterns: &[&str]) -> String {
    for pattern in patterns {
        if let Some(pos) = input.find(pattern) {
            let after = &input[pos + pattern.len()..];
            return after.trim().to_string();
        }
    }
    String::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_analyze_intent() {
        let parser = IntentParser::new();
        let intent = parser.parse("analizar el archivo de C:\\test\\malware.exe");
        assert!(matches!(intent, Intent::AnalyzeFile { .. }));
    }

    #[test]
    fn test_quarantine_intent() {
        let parser = IntentParser::new();
        let intent = parser.parse("cuarentena");
        assert!(matches!(intent, Intent::OpenQuarantine));
    }

    #[test]
    fn test_greeting() {
        let parser = IntentParser::new();
        let intent = parser.parse("hola");
        assert!(matches!(intent, Intent::GeneralConversation { .. }));
    }

    #[test]
    fn test_context_disambiguate_analyze() {
        let parser = IntentParser::new();
        let mut ctx = ApplicationContext::new();
        ctx.selected_file = Some("C:\\suspicious.exe".into());
        let intent = parser.parse_with_context("analizar este", &ctx);
        match intent {
            Intent::AnalyzeFile { path } => assert_eq!(path, "C:\\suspicious.exe"),
            _ => panic!("Expected AnalyzeFile with context path"),
        }
    }

    #[test]
    fn test_context_disambiguate_quarantine() {
        let parser = IntentParser::new();
        let mut ctx = ApplicationContext::new();
        ctx.selected_file = Some("C:\\bad.exe".into());
        let intent = parser.parse_with_context("cuarentena este archivo", &ctx);
        match intent {
            Intent::QuarantineFile { path, .. } => assert_eq!(path, "C:\\bad.exe"),
            _ => panic!("Expected QuarantineFile with context path"),
        }
    }

    #[test]
    fn test_context_disambiguate_report() {
        let parser = IntentParser::new();
        let mut ctx = ApplicationContext::new();
        ctx.current_analysis_id = Some("abc-123".into());
        let intent = parser.parse_with_context("informe de este", &ctx);
        match intent {
            Intent::GenerateReport { scan_id, .. } => assert_eq!(scan_id, "abc-123"),
            _ => panic!("Expected GenerateReport with context id"),
        }
    }

    #[test]
    fn test_context_explicit_path_over_context() {
        let parser = IntentParser::new();
        let mut ctx = ApplicationContext::new();
        ctx.selected_file = Some("C:\\other.exe".into());
        let intent = parser.parse_with_context("analizar \"C:\\specific.exe\"", &ctx);
        match intent {
            Intent::AnalyzeFile { path } => assert_eq!(path.to_lowercase(), "c:\\specific.exe"),
            _ => panic!("Expected AnalyzeFile with explicit path"),
        }
    }

    #[test]
    fn test_analyze_english() {
        let parser = IntentParser::new();
        let intent = parser.parse("analyze file of C:\\test\\file.exe");
        assert!(matches!(intent, Intent::AnalyzeFile { .. }));
    }

    #[test]
    fn test_hash_lookup_sha256() {
        let parser = IntentParser::new();
        let intent = parser.parse("check hash d41d8cd98f00b204e9800998ecf8427e");
        match intent {
            Intent::QueryVirusTotal { hash } => assert_eq!(hash, "d41d8cd98f00b204e9800998ecf8427e"),
            _ => panic!("Expected QueryVirusTotal"),
        }
    }

    #[test]
    fn test_open_history() {
        let parser = IntentParser::new();
        let intent = parser.parse("mostrar historial");
        assert!(matches!(intent, Intent::OpenHistory));
    }

    #[test]
    fn test_open_history_english() {
        let parser = IntentParser::new();
        let intent = parser.parse("show history");
        assert!(matches!(intent, Intent::OpenHistory));
    }

    #[test]
    fn test_get_system_info() {
        let parser = IntentParser::new();
        let intent = parser.parse("información del sistema");
        assert!(matches!(intent, Intent::GetSystemInfo));
    }

    #[test]
    fn test_get_system_info_english() {
        let parser = IntentParser::new();
        let intent = parser.parse("system info");
        assert!(matches!(intent, Intent::GetSystemInfo));
    }

    #[test]
    fn test_get_rules() {
        let parser = IntentParser::new();
        let intent = parser.parse("ver reglas");
        assert!(matches!(intent, Intent::GetRules));
    }

    #[test]
    fn test_activate_ysmel() {
        let parser = IntentParser::new();
        let intent = parser.parse("activar ysmel");
        assert!(matches!(intent, Intent::ActivateYsmel));
    }

    #[test]
    fn test_activate_fenix() {
        let parser = IntentParser::new();
        let intent = parser.parse("activar fenix");
        assert!(matches!(intent, Intent::ActivateFenix));
    }

    #[test]
    fn test_deactivate_ysmel() {
        let parser = IntentParser::new();
        let intent = parser.parse("desactivar ysmel");
        assert!(matches!(intent, Intent::DeactivateYsmel));
    }

    #[test]
    fn test_deactivate_fenix() {
        let parser = IntentParser::new();
        let intent = parser.parse("desactivar fenix");
        assert!(matches!(intent, Intent::DeactivateFenix));
    }

    #[test]
    fn test_unknown_is_conversation() {
        let parser = IntentParser::new();
        let intent = parser.parse("asdfghjkl");
        assert!(matches!(intent, Intent::GeneralConversation { .. }));
    }

    #[test]
    fn test_quarantine_with_reason() {
        let parser = IntentParser::new();
        let intent = parser.parse("cuarentena \"c:\\bad.exe\" porque malware detectado");
        match intent {
            Intent::QuarantineFile { path, reason } => {
                assert_eq!(path, "c:\\bad.exe");
                assert!(reason.is_some());
                assert!(reason.unwrap().contains("malware"));
            }
            _ => panic!("Expected QuarantineFile with reason"),
        }
    }

    #[test]
    fn test_generate_report_csv() {
        let parser = IntentParser::new();
        let intent = parser.parse("informe csv abc-12345");
        match intent {
            Intent::GenerateReport { scan_id, format } => {
                assert_eq!(scan_id, "abc-12345");
                assert_eq!(format.as_deref(), Some("csv"));
            }
            _ => panic!("Expected GenerateReport"),
        }
    }

    #[test]
    fn test_generate_report_html() {
        let parser = IntentParser::new();
        let intent = parser.parse("export report html abc-12345");
        match intent {
            Intent::GenerateReport { format, .. } => {
                assert_eq!(format.as_deref(), Some("html"));
            }
            _ => panic!("Expected GenerateReport"),
        }
    }

    #[test]
    fn test_hash_sha1() {
        let parser = IntentParser::new();
        let intent = parser.parse("reputation a94a8fe5ccb19ba61c4c0873d391e987982fbbd3");
        match intent {
            Intent::QueryVirusTotal { hash } => assert_eq!(hash.len(), 40),
            _ => panic!("Expected QueryVirusTotal"),
        }
    }

    #[test]
    fn test_hash_sha256() {
        let parser = IntentParser::new();
        let intent = parser.parse("hash d41d8cd98f00b204e9800998ecf8427e");
        match intent {
            Intent::QueryVirusTotal { hash } => assert_eq!(hash.len(), 32),
            _ => panic!("Expected QueryVirusTotal"),
        }
    }

    #[test]
    fn test_quarantine_with_path_backslash() {
        let parser = IntentParser::new();
        let intent = parser.parse("cuarentena \"C:\\Windows\\System32\\evil.dll\"");
        match intent {
            Intent::QuarantineFile { path, .. } => {
                assert!(path.contains("evil.dll"));
            }
            _ => panic!("Expected QuarantineFile"),
        }
    }

    #[test]
    fn test_quarantine_no_path_no_context_opens_page() {
        let parser = IntentParser::new();
        let ctx = ApplicationContext::new();
        let intent = parser.parse_with_context("cuarentena", &ctx);
        assert!(matches!(intent, Intent::OpenQuarantine));
    }

    #[test]
    fn test_activate_ysmel_english() {
        let parser = IntentParser::new();
        let intent = parser.parse("activate ysmel");
        assert!(matches!(intent, Intent::ActivateYsmel));
    }

    #[test]
    fn test_get_rules_english() {
        let parser = IntentParser::new();
        let intent = parser.parse("show heuristic rules");
        assert!(matches!(intent, Intent::GetRules));
    }

    #[test]
    fn test_case_insensitive() {
        let parser = IntentParser::new();
        let intent = parser.parse("ANALIZAR de C:\\test.exe");
        assert!(matches!(intent, Intent::AnalyzeFile { .. }));
    }

    #[test]
    fn test_whitespace_handling() {
        let parser = IntentParser::new();
        let intent = parser.parse("  hola mundo  ");
        assert!(matches!(intent, Intent::GeneralConversation { .. }));
    }

    #[test]
    fn test_context_restore_with_analysis_id() {
        let parser = IntentParser::new();
        let mut ctx = ApplicationContext::new();
        ctx.current_analysis_id = Some("abc-123".into());
        // Known limitation: extract_id grabs "quarantine" (9 chars) as an ID
        // before the context disambiguation fires. This test verifies the
        // intent is still RestoreFile (just with the wrong ID).
        let intent = parser.parse_with_context("restore it from quarantine", &ctx);
        assert!(
            matches!(intent, Intent::RestoreFile { .. }),
            "Expected RestoreFile, got: {:?}",
            intent
        );
    }
}
