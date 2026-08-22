use super::provider::AiProvider;
use super::types::{AiCompletion, AiError, ModelInfo};
use std::sync::Arc;
use std::time::Duration;

/// Provider que se conecta a Ollama local para respuestas contextuales.
///
/// Usa el endpoint `/api/chat` con mensajes estructurados en vez del
/// prompt plano de `/api/generate`. Incluye cliente HTTP compartido
/// y reintentos para errores transitorios.
pub struct OllamaProvider {
    base_url: String,
    model: String,
    temperature: f32,
    max_tokens: u32,
    client: reqwest::Client,
    /// Se actualiza con health_check para evitar llamadas a un server caído.
    available: Arc<std::sync::atomic::AtomicBool>,
}

impl OllamaProvider {
    pub fn with_config(base_url: String, model: String, temperature: f32, max_tokens: u32) -> Self {
        Self {
            base_url,
            model,
            temperature,
            max_tokens,
            client: Self::make_client(),
            available: Arc::new(std::sync::atomic::AtomicBool::new(false)),
        }
    }

    fn make_client() -> reqwest::Client {
        reqwest::Client::builder()
            .timeout(Duration::from_secs(15))
            .pool_max_idle_per_host(2)
            .build()
            .unwrap_or_default()
    }

    /// Verifica si Ollama está disponible y actualiza el flag.
    pub async fn refresh_availability(&self) -> bool {
        let ok = self.health_check().await;
        self.available
            .store(ok, std::sync::atomic::Ordering::SeqCst);
        ok
    }

    /// Devuelve si el último health check fue exitoso.
    pub fn is_available(&self) -> bool {
        self.available.load(std::sync::atomic::Ordering::SeqCst)
    }

    /// Construye el array de mensajes para `/api/chat`.
    ///
    /// Formato optimizado para chat models:
    /// 1. System prompt (identidad + contexto + restricciones)
    /// 2. Historial previo como pares user/assistant
    /// 3. Mensaje actual del usuario
    fn build_messages(
        &self,
        system_prompt: &str,
        user_message: &str,
        context: &[String],
    ) -> Vec<serde_json::Value> {
        let mut messages = Vec::new();

        // System prompt
        messages.push(serde_json::json!({
            "role": "system",
            "content": system_prompt,
        }));

        // Historial de conversación previo (como pares user/assistant)
        for ctx_msg in context {
            let (role, content) = if let Some(stripped) = ctx_msg.strip_prefix("[user]: ") {
                ("user", stripped)
            } else if let Some(stripped) = ctx_msg.strip_prefix("[assistant]: ") {
                ("assistant", stripped)
            } else {
                // Default: tratar como mensaje del usuario
                ("user", ctx_msg.as_str())
            };
            messages.push(serde_json::json!({
                "role": role,
                "content": content,
            }));
        }

        // Mensaje actual del usuario
        messages.push(serde_json::json!({
            "role": "user",
            "content": user_message,
        }));

        messages
    }

    /// Realiza una petición con reintentos (máx 1 reintento en errores 5xx).
    async fn request_with_retry(
        &self,
        payload: &serde_json::Value,
    ) -> Result<serde_json::Value, AiError> {
        let url = format!("{}/api/chat", self.base_url);
        let max_retries = 1;

        for attempt in 0..=max_retries {
            let response = self
                .client
                .post(&url)
                .json(payload)
                .send()
                .await
                .map_err(|e| AiError::ConnectionFailed(e.to_string()))?;

            if response.status().is_success() {
                return response
                    .json::<serde_json::Value>()
                    .await
                    .map_err(|e| AiError::ParseError(e.to_string()));
            }

            // Solo reintentar en 5xx (errores del server, no del cliente)
            let status = response.status().as_u16();
            if status >= 500 && attempt < max_retries {
                let delay = Duration::from_millis(500 * (attempt as u64 + 1));
                tokio::time::sleep(delay).await;
                continue;
            }

            return Err(AiError::ConnectionFailed(format!(
                "Ollama returned HTTP {status}"
            )));
        }

        Err(AiError::ConnectionFailed("Max retries exhausted".into()))
    }
}

#[async_trait::async_trait]
impl AiProvider for OllamaProvider {
    fn name(&self) -> &str {
        "ollama"
    }

    fn model_info(&self) -> ModelInfo {
        ModelInfo {
            provider: "ollama".into(),
            model: self.model.clone(),
            available: self.is_available(),
        }
    }

    async fn complete(
        &self,
        system_prompt: &str,
        user_message: &str,
        context: &[String],
    ) -> Result<AiCompletion, AiError> {
        // Si sabemos que Ollama no está disponible, fallar rápido
        if !self.is_available() {
            // Intentar un health check antes de fallar completamente
            if !self.refresh_availability().await {
                return Err(AiError::ConnectionFailed(
                    "Ollama no está disponible. Verifique que el servidor esté ejecutándose."
                        .into(),
                ));
            }
        }

        let messages = self.build_messages(system_prompt, user_message, context);

        let payload = serde_json::json!({
            "model": self.model,
            "messages": messages,
            "stream": false,
            "options": {
                "temperature": self.temperature,
                "num_predict": self.max_tokens,
                "top_p": 0.9,
            }
        });

        let body = self.request_with_retry(&payload).await?;

        let text = body["message"]["content"]
            .as_str()
            .unwrap_or("No response from Ollama")
            .to_string();

        Ok(AiCompletion {
            text,
            model: self.model.clone(),
            provider: "ollama".into(),
        })
    }

    async fn health_check(&self) -> bool {
        match self
            .client
            .get(format!("{}/api/tags", self.base_url))
            .timeout(Duration::from_secs(3))
            .send()
            .await
        {
            Ok(resp) => resp.status().is_success(),
            Err(_) => false,
        }
    }
}
