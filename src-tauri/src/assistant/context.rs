use serde::{Deserialize, Serialize};

/// Contexto de la aplicación en el momento actual.
///
/// Se recopila antes de cada interacción con el AI para proporcionar
/// información relevante al modelo. Incluye estado de la UI, resultados
/// recientes, y estado de protocolos.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApplicationContext {
    /// Página actual del usuario ("dashboard", "scan", "results", etc.)
    pub current_page: String,
    /// Archivo actualmente seleccionado (si lo hay).
    pub selected_file: Option<String>,
    /// ID del análisis actualmente visible.
    pub current_analysis_id: Option<String>,
    /// Número de archivos en el historial.
    pub history_count: u32,
    /// Número de archivos en cuarentena.
    pub quarantine_count: u32,
    /// Si hay un escaneo activo.
    pub scan_active: bool,
    /// Información del sistema (OS, hostname, etc.)
    pub system_summary: Option<String>,
    /// Protocolos activos.
    pub ysmel_active: bool,
    pub fenix_active: bool,
    /// Idioma preferido del usuario ("es", "en").
    pub language: String,
    /// Nivel de amenaza del último análisis visible.
    pub current_threat_level: Option<String>,
    /// Resumen del último escaneo (nombre del archivo + veredicto).
    pub last_scan_summary: Option<String>,
    /// Número total de amenazas detectadas en la sesión actual.
    pub total_threats_detected: u32,
}

impl ApplicationContext {
    /// Crea un contexto vacío con valores por defecto.
    pub fn new() -> Self {
        Self::default()
    }

    /// Serializa el contexto a un string para incluir en el prompt del AI.
    /// Formato legible por LLM, optimizado para consumo de tokens.
    #[allow(dead_code)]
    pub fn to_prompt_context(&self) -> String {
        let mut parts: Vec<String> = Vec::new();

        parts.push(format!("Page: {}", self.current_page));

        if let Some(ref file) = self.selected_file {
            parts.push(format!("Selected file: {file}"));
        }
        if let Some(ref id) = self.current_analysis_id {
            parts.push(format!("Viewing analysis: {id}"));
        }
        if self.history_count > 0 {
            parts.push(format!("History: {} analyses", self.history_count));
        }
        if self.quarantine_count > 0 {
            parts.push(format!("Quarantine: {} files", self.quarantine_count));
        }
        if self.scan_active {
            parts.push("Scan in progress".into());
        }
        if let Some(ref level) = self.current_threat_level {
            parts.push(format!("Current threat level: {level}"));
        }
        if let Some(ref summary) = self.last_scan_summary {
            parts.push(format!("Last scan: {summary}"));
        }
        if self.total_threats_detected > 0 {
            parts.push(format!("Threats detected this session: {}", self.total_threats_detected));
        }
        if self.ysmel_active {
            parts.push("Ysmel: ACTIVE".into());
        }
        if self.fenix_active {
            parts.push("Fenix: ACTIVE".into());
        }
        parts.push(format!("Language: {}", self.language));

        parts.join(" | ")
    }

    /// Serializa solo los campos que cambian frecuentemente para detectar
    /// si el contexto relevante para el LLM ha cambiado desde la última llamada.
    /// Útil para evitar re-enviar contexto idéntico al provider.
    #[allow(dead_code)]
    pub fn fingerprint(&self) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut h = DefaultHasher::new();
        self.current_page.hash(&mut h);
        self.selected_file.hash(&mut h);
        self.current_analysis_id.hash(&mut h);
        self.scan_active.hash(&mut h);
        self.ysmel_active.hash(&mut h);
        self.fenix_active.hash(&mut h);
        self.current_threat_level.hash(&mut h);
        self.last_scan_summary.hash(&mut h);
        self.total_threats_detected.hash(&mut h);
        self.language.hash(&mut h);
        h.finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prompt_context_minimal() {
        let ctx = ApplicationContext::new();
        let text = ctx.to_prompt_context();
        assert!(text.contains("Page:"));
        assert!(text.contains("Language:"));
    }

    #[test]
    fn test_prompt_context_full() {
        let ctx = ApplicationContext {
            current_page: "results".into(),
            selected_file: Some("malware.exe".into()),
            current_analysis_id: Some("abc-123".into()),
            history_count: 5,
            quarantine_count: 2,
            scan_active: false,
            system_summary: None,
            ysmel_active: true,
            fenix_active: false,
            language: "es".into(),
            current_threat_level: Some("high".into()),
            last_scan_summary: Some("malware.exe → Malicious".into()),
            total_threats_detected: 3,
        };
        let text = ctx.to_prompt_context();
        assert!(text.contains("results"));
        assert!(text.contains("malware.exe"));
        assert!(text.contains("abc-123"));
        assert!(text.contains("5 analyses"));
        assert!(text.contains("2 files"));
        assert!(text.contains("high"));
        assert!(text.contains("Ysmel: ACTIVE"));
        assert!(text.contains("Threats detected this session: 3"));
    }

    #[test]
    fn test_prompt_context_fenix_active() {
        let mut ctx = ApplicationContext::new();
        ctx.fenix_active = true;
        let text = ctx.to_prompt_context();
        assert!(text.contains("Fenix: ACTIVE"));
    }

    #[test]
    fn test_prompt_context_scan_active() {
        let mut ctx = ApplicationContext::new();
        ctx.scan_active = true;
        let text = ctx.to_prompt_context();
        assert!(text.contains("Scan in progress"));
    }

    #[test]
    fn test_fingerprint_deterministic() {
        let mut ctx = ApplicationContext::new();
        ctx.current_page = "dashboard".into();
        let f1 = ctx.fingerprint();
        let f2 = ctx.fingerprint();
        assert_eq!(f1, f2);
    }

    #[test]
    fn test_fingerprint_changes_on_field_modification() {
        let mut ctx = ApplicationContext::new();
        ctx.current_page = "dashboard".into();
        let f1 = ctx.fingerprint();
        ctx.current_page = "results".into();
        let f2 = ctx.fingerprint();
        assert_ne!(f1, f2);
    }

    #[test]
    fn test_prompt_context_total_threats_zero_omitted() {
        let ctx = ApplicationContext::new();
        let text = ctx.to_prompt_context();
        assert!(!text.contains("Threats detected"));
    }
}
