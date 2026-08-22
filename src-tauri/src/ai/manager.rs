use super::ollama::OllamaProvider;
use super::provider::AiProvider;
use super::stub::StubProvider;
use super::types::{AiCompletion, AiError, ModelInfo};

use std::sync::Arc;
use tokio::sync::RwLock;

/// Manager dinámico que selecciona entre Stub y Ollama.
///
/// Se comparte con `Arc<RwLock<ProviderManager>>` para que los comandos
/// Tauri puedan leer el provider activo y la configuración pueda cambiarlo
/// en caliente.
///
/// **Importante**: Para evitar bloquear el RwLock durante llamadas HTTP,
/// el manager expone `clone_provider()` que devuelve un `Box<dyn AiProvider>`
/// clonado. El caller hace `.read().await` solo para clonar, luego suelta
/// el lock y ejecuta la llamada async.
pub struct ProviderManager {
    provider: Arc<dyn AiProvider>,
    ollama_url: String,
    ollama_model: String,
}

impl ProviderManager {
    pub fn new() -> Arc<RwLock<Self>> {
        Arc::new(RwLock::new(Self {
            provider: Arc::new(StubProvider::new()),
            ollama_url: "http://localhost:11434".into(),
            ollama_model: "llama3.2".into(),
        }))
    }

    /// Configura el manager desde AppConfig. Si ollama está habilitado,
    /// intenta health check y conecta. Modifica self en-place.
    pub async fn init_from_config(&mut self, config: &crate::config::AppConfig) {
        self.ollama_url = config.ollama.url.clone();
        self.ollama_model = config.ollama.model.clone();

        if config.ollama.enabled {
            let ollama = OllamaProvider::with_config(
                config.ollama.url.clone(),
                config.ollama.model.clone(),
                config.ollama.temperature,
                config.ollama.max_tokens,
            );
            if ollama.refresh_availability().await {
                self.provider = Arc::new(ollama);
            }
            // Si no está disponible, queda en stub
        }
    }

    /// Cambia a Ollama con la URL, modelo y parámetros dados.
    pub fn switch_to_ollama(
        &mut self,
        url: String,
        model: String,
        temperature: f32,
        max_tokens: u32,
    ) {
        self.ollama_url = url.clone();
        self.ollama_model = model.clone();
        self.provider = Arc::new(OllamaProvider::with_config(
            url,
            model,
            temperature,
            max_tokens,
        ));
    }

    /// Cambia a StubProvider (modo determinista).
    pub fn switch_to_stub(&mut self) {
        self.provider = Arc::new(StubProvider::new());
    }

    /// Devuelve una referencia Arc al provider activo.
    /// El caller puede clonar el Arc y soltar el lock antes de hacer la llamada.
    pub fn provider_ref(&self) -> Arc<dyn AiProvider> {
        Arc::clone(&self.provider)
    }

    /// Devuelve la info del modelo activo.
    pub fn model_info(&self) -> ModelInfo {
        self.provider.model_info()
    }

    /// URL actual de Ollama.
    pub fn ollama_url(&self) -> &str {
        &self.ollama_url
    }

    /// Modelo actual de Ollama.
    pub fn ollama_model(&self) -> &str {
        &self.ollama_model
    }

    /// Test de conexión con Ollama.
    pub async fn test_ollama_connection(url: &str) -> OllamaTestResult {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(5))
            .build()
            .unwrap_or_default();

        match client.get(format!("{}/api/tags", url)).send().await {
            Ok(resp) => {
                if resp.status().is_success() {
                    match resp.json::<serde_json::Value>().await {
                        Ok(body) => {
                            let models = body["models"]
                                .as_array()
                                .map(|arr| {
                                    arr.iter()
                                        .filter_map(|m| m["name"].as_str().map(String::from))
                                        .collect()
                                })
                                .unwrap_or_default();
                            OllamaTestResult {
                                connected: true,
                                models,
                                error: None,
                            }
                        }
                        Err(e) => OllamaTestResult {
                            connected: false,
                            models: vec![],
                            error: Some(format!("Parse error: {e}")),
                        },
                    }
                } else {
                    OllamaTestResult {
                        connected: false,
                        models: vec![],
                        error: Some(format!("HTTP {}", resp.status())),
                    }
                }
            }
            Err(e) => OllamaTestResult {
                connected: false,
                models: vec![],
                error: Some(e.to_string()),
            },
        }
    }
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OllamaTestResult {
    pub connected: bool,
    pub models: Vec<String>,
    pub error: Option<String>,
}

#[async_trait::async_trait]
impl AiProvider for ProviderManager {
    fn name(&self) -> &str {
        self.provider.name()
    }

    fn model_info(&self) -> ModelInfo {
        self.provider.model_info()
    }

    async fn complete(
        &self,
        system_prompt: &str,
        user_message: &str,
        context: &[String],
    ) -> Result<AiCompletion, AiError> {
        self.provider
            .complete(system_prompt, user_message, context)
            .await
    }

    async fn health_check(&self) -> bool {
        self.provider.health_check().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_manager_starts_with_stub() {
        let manager = ProviderManager::new();
        let mgr = manager.read().await;
        assert_eq!(mgr.name(), "stub");
        assert!(mgr.health_check().await);
    }

    #[tokio::test]
    async fn test_manager_switch_to_stub() {
        let manager = ProviderManager::new();
        {
            let mut mgr = manager.write().await;
            mgr.switch_to_ollama("http://fake:11434".into(), "test".into(), 0.3, 1024);
            assert_eq!(mgr.name(), "ollama");
        }
        {
            let mut mgr = manager.write().await;
            mgr.switch_to_stub();
            assert_eq!(mgr.name(), "stub");
        }
    }

    #[tokio::test]
    async fn test_manager_switch_to_ollama() {
        let manager = ProviderManager::new();
        let mut mgr = manager.write().await;
        mgr.switch_to_ollama("http://myhost:11434".into(), "mistral".into(), 0.5, 2048);
        assert_eq!(mgr.ollama_url(), "http://myhost:11434");
        assert_eq!(mgr.ollama_model(), "mistral");
        assert_eq!(mgr.name(), "ollama");
    }

    #[tokio::test]
    async fn test_manager_model_info() {
        let manager = ProviderManager::new();
        let mgr = manager.read().await;
        let info = mgr.model_info();
        assert_eq!(info.provider, "stub");
        assert!(info.available);
    }

    #[tokio::test]
    async fn test_manager_provider_ref() {
        let manager = ProviderManager::new();
        let provider_ref;
        {
            let mgr = manager.read().await;
            provider_ref = mgr.provider_ref();
        }
        assert_eq!(provider_ref.name(), "stub");
    }

    #[tokio::test]
    async fn test_manager_complete_delegates_to_stub() {
        let manager = ProviderManager::new();
        let result = manager
            .read()
            .await
            .complete("sys", "hello", &[])
            .await
            .unwrap();
        assert!(!result.text.is_empty());
    }
}
