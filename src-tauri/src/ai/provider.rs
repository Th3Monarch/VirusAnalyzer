use super::types::{AiCompletion, AiError, ModelInfo};

/// Trait que define la interfaz para providers de AI.
///
/// Cada provider (Stub, Ollama, etc.) implementa este trait.
/// El assistant usa este trait de forma genérica.
#[async_trait::async_trait]
pub trait AiProvider: Send + Sync {
    /// Nombre del provider.
    #[allow(dead_code)]
    fn name(&self) -> &str;

    /// Información del modelo activo.
    fn model_info(&self) -> ModelInfo;

    /// Envía un prompt y devuelve la respuesta.
    async fn complete(
        &self,
        system_prompt: &str,
        user_message: &str,
        context: &[String],
    ) -> Result<AiCompletion, AiError>;

    /// Verifica si el provider está disponible.
    async fn health_check(&self) -> bool;
}
