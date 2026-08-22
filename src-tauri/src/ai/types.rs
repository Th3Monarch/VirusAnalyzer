use serde::{Deserialize, Serialize};

/// Información del modelo AI activo.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelInfo {
    pub provider: String,
    pub model: String,
    pub available: bool,
}

/// Respuesta cruda del provider AI.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct AiCompletion {
    pub text: String,
    pub model: String,
    pub provider: String,
}

/// Error del provider AI.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum AiError {
    NotAvailable,
    ConnectionFailed(String),
    ParseError(String),
    RateLimited,
    Timeout,
}
