use super::provider::AiProvider;
use super::types::{AiCompletion, AiError, ModelInfo};

/// Provider determinista sin red: genera respuestas predefinidas.
///
/// Se usa cuando Ollama no está disponible. Las respuestas son útiles
/// pero no contextualizadas con LLM real.
pub struct StubProvider;

impl StubProvider {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait::async_trait]
impl AiProvider for StubProvider {
    fn name(&self) -> &str {
        "stub"
    }

    fn model_info(&self) -> ModelInfo {
        ModelInfo {
            provider: "stub".into(),
            model: "deterministic".into(),
            available: true,
        }
    }

    async fn complete(
        &self,
        _system_prompt: &str,
        user_message: &str,
        _context: &[String],
    ) -> Result<AiCompletion, AiError> {
        let response = generate_stub_response(user_message);
        Ok(AiCompletion {
            text: response,
            model: "deterministic".into(),
            provider: "stub".into(),
        })
    }

    async fn health_check(&self) -> bool {
        true
    }
}

fn generate_stub_response(input: &str) -> String {
    let lower = input.to_lowercase();

    if lower.contains("analizar") || lower.contains("scan") || lower.contains("analyz") {
        return "I can help you analyze files. Use the scan page to upload a file, or tell me the path and I'll guide you through the process.".into();
    }

    if lower.contains("hola")
        || lower.contains("hello")
        || lower.contains("hi")
        || lower.contains("hey")
    {
        return "Hello! I'm your Prometeo assistant. I can help you with malware analysis, file scanning, quarantine management, and understanding scan results. What would you like to do?".into();
    }

    if lower.contains("cuarentena") || lower.contains("quarantine") {
        return "The quarantine isolates suspicious files without deleting them. You can restore files from the quarantine page if needed. Would you like me to check your quarantine status?".into();
    }

    if lower.contains("amenaza") || lower.contains("threat") || lower.contains("malware") {
        return "Prometeo uses heuristic analysis, static file inspection, and optional VirusTotal reputation checks to assess threats. Each finding contributes to a threat score from 0-100. Want me to explain a specific finding?".into();
    }

    if lower.contains("regla") || lower.contains("rule") || lower.contains("heuristic") {
        return "The heuristic engine applies 28 rules across 7 categories: suspicious keywords, PE anomalies, entropy patterns, encoding indicators, metadata anomalies, persistence mechanisms, and network indicators. Want details on a specific category?".into();
    }

    if lower.contains("virus") || lower.contains("total") {
        return "VirusTotal integration requires an API key configured in Settings. It checks file hashes against 70+ antivirus engines. Would you like to check a specific hash?".into();
    }

    if lower.contains("ayuda") || lower.contains("help") || lower.contains("what can you do") {
        return "I can help you with:\n- Analyzing files for threats\n- Understanding scan results\n- Managing quarantine\n- Explaining heuristic rules\n- Checking VirusTotal reputation\n- System security information\n\nJust ask me anything!".into();
    }

    "I'm here to help with Prometeo. You can ask me about analyzing files, understanding results, managing quarantine, or any security-related questions. What would you like to know?".into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_stub_always_healthy() {
        let stub = StubProvider::new();
        assert!(stub.health_check().await);
    }

    #[tokio::test]
    async fn test_stub_complete_returns_text() {
        let stub = StubProvider::new();
        let result = stub.complete("system", "hello", &[]).await.unwrap();
        assert!(!result.text.is_empty());
    }

    #[test]
    fn test_stub_name() {
        let stub = StubProvider::new();
        assert_eq!(stub.name(), "stub");
    }

    #[test]
    fn test_stub_model_info() {
        let stub = StubProvider::new();
        let info = stub.model_info();
        assert_eq!(info.provider, "stub");
        assert!(info.available);
    }

    #[tokio::test]
    async fn test_stub_scan_response() {
        let stub = StubProvider::new();
        let result = stub.complete("", "analizar archivo", &[]).await.unwrap();
        assert!(result.text.contains("analyze") || result.text.contains("scan"));
    }

    #[tokio::test]
    async fn test_stub_quarantine_response() {
        let stub = StubProvider::new();
        let result = stub.complete("", "cuarentena", &[]).await.unwrap();
        assert!(result.text.to_lowercase().contains("quarantine"));
    }

    #[tokio::test]
    async fn test_stub_threat_response() {
        let stub = StubProvider::new();
        let result = stub.complete("", "que es un malware", &[]).await.unwrap();
        assert!(result.text.to_lowercase().contains("threat"));
    }

    #[tokio::test]
    async fn test_stub_help_response() {
        let stub = StubProvider::new();
        let result = stub.complete("", "ayuda", &[]).await.unwrap();
        assert!(!result.text.is_empty());
    }

    #[tokio::test]
    async fn test_stub_default_response() {
        let stub = StubProvider::new();
        let result = stub.complete("", "random question 123", &[]).await.unwrap();
        assert!(result.text.contains("Prometeo"));
    }
}
