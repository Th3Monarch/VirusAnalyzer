use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;

/// Trait para Speech-to-Text.
#[async_trait::async_trait]
pub trait SpeechToText: Send + Sync {
    async fn transcribe(&self, audio: &[u8], language: &str) -> Result<String, VoiceError>;
    async fn health_check(&self) -> bool;
}

/// Trait para Text-to-Speech.
#[async_trait::async_trait]
pub trait TextToSpeech: Send + Sync {
    async fn synthesize(&self, text: &str, language: &str) -> Result<Vec<u8>, VoiceError>;
    async fn health_check(&self) -> bool;
}

// ---------------------------------------------------------------------------
// Kokoro TTS Provider
// ---------------------------------------------------------------------------

/// Voz Kokoro femenina adulta predeterminada por idioma.
pub fn default_voice_for_language(lang: &str) -> &'static str {
    match lang {
        "es" => "ef_dora",
        "pt" => "pf_dora",
        _ => "af_heart", // English / fallback
    }
}

/// Acento nativo deseado por idioma.
///
/// Kokoro v1.0 **no** ofrece variantes de acento (paisa, neoyorquino, etc.).
/// Estos metadatos describen el acento *objetivo* de Prometeo para que el
/// system prompt pueda instruir al LLM sobre cómo adaptar la personalidad.
pub fn desired_accent(lang: &str) -> AccentInfo {
    match lang {
        "es" => AccentInfo {
            label: "Paisa \u{2014} Colombia".into(),
            code: "co-paisa".into(),
            native_available: false,
            limitation: Some(
                "Kokoro v1.0 only provides a generic Latin American Spanish voice (ef_dora). \
                 A dedicated Colombian Paisa accent is not available."
                    .into(),
            ),
        },
        "en" => AccentInfo {
            label: "New York \u{2014} USA".into(),
            code: "us-nyc".into(),
            native_available: false,
            limitation: Some(
                "Kokoro v1.0 only provides standard American English voices (af_*). \
                 A dedicated New York City accent is not available."
                    .into(),
            ),
        },
        "pt" => AccentInfo {
            label: "Brasileiro".into(),
            code: "br".into(),
            native_available: true,
            limitation: None,
        },
        _ => AccentInfo {
            label: "Standard".into(),
            code: "std".into(),
            native_available: true,
            limitation: None,
        },
    }
}

/// Vozes disponibles de Kokoro agrupadas por idioma (solo voces femeninas adultas).
///
/// Fuentes: Kokoro-82M v1.0 voice pack + HuggingFace docs.
/// NOTA: Kokoro NO ofrece acentos regionales (paisa, neoyorquino, etc.).
pub fn available_voices(lang: &str) -> Vec<(&'static str, &'static str)> {
    match lang {
        "es" => vec![("ef_dora", "Dora")],
        "pt" => vec![("pf_dora", "Dora")],
        // English: all af_* voices are standard American. No NYC variant exists.
        _ => vec![
            ("af_heart", "Heart"),
            ("af_bella", "Bella"),
            ("af_sarah", "Sarah"),
            ("af_nicole", "Nicole"),
            ("af_sky", "Sky"),
            ("af_jessica", "Jessica"),
            ("af_kore", "Kore"),
            ("af_nova", "Nova"),
            ("af_alloy", "Alloy"),
            ("af_aoede", "Aoede"),
            ("af_river", "River"),
        ],
    }
}

/// Provider TTS que se conecta a un servidor Kokoro local.
///
/// Kokoro expone una API compatible con OpenAI:
/// `POST /v1/audio/speech` → audio bytes (WAV o MP3).
pub struct KokoroProvider {
    base_url: String,
    voice_id: String,
    client: reqwest::Client,
    available: Arc<std::sync::atomic::AtomicBool>,
}

impl KokoroProvider {
    pub fn new(base_url: String, voice_id: String) -> Self {
        Self {
            base_url,
            voice_id,
            client: Self::make_client(),
            available: Arc::new(std::sync::atomic::AtomicBool::new(false)),
        }
    }

    fn make_client() -> reqwest::Client {
        reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .unwrap_or_default()
    }

    pub async fn refresh_availability(&self) -> bool {
        let ok = self.health_check().await;
        self.available
            .store(ok, std::sync::atomic::Ordering::SeqCst);
        ok
    }

    pub fn is_available(&self) -> bool {
        self.available.load(std::sync::atomic::Ordering::SeqCst)
    }

    async fn health_check(&self) -> bool {
        match self
            .client
            .get(format!("{}/v1/models", self.base_url))
            .timeout(Duration::from_secs(3))
            .send()
            .await
        {
            Ok(resp) => resp.status().is_success(),
            Err(_) => false,
        }
    }
}

#[async_trait::async_trait]
impl TextToSpeech for KokoroProvider {
    async fn synthesize(&self, text: &str, _language: &str) -> Result<Vec<u8>, VoiceError> {
        if !self.is_available() {
            return Err(VoiceError::SynthesisFailed(
                "Kokoro server not available".into(),
            ));
        }

        let payload = serde_json::json!({
            "model": "kokoro",
            "input": text,
            "voice": self.voice_id,
            "response_format": "wav",
        });

        let response = self
            .client
            .post(format!("{}/v1/audio/speech", self.base_url))
            .json(&payload)
            .send()
            .await
            .map_err(|e| VoiceError::SynthesisFailed(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            return Err(VoiceError::SynthesisFailed(format!(
                "Kokoro returned HTTP {status}"
            )));
        }

        let bytes = response
            .bytes()
            .await
            .map_err(|e| VoiceError::SynthesisFailed(e.to_string()))?;

        Ok(bytes.to_vec())
    }

    async fn health_check(&self) -> bool {
        self.refresh_availability().await
    }
}

// ---------------------------------------------------------------------------
// Whisper STT Provider
// ---------------------------------------------------------------------------

/// Provider STT que se conecta a un servidor Whisper local.
///
/// Whisper expone una API compatible con OpenAI:
/// `POST /v1/audio/transcriptions` (multipart) → texto transcrito.
pub struct WhisperProvider {
    base_url: String,
    client: reqwest::Client,
    available: Arc<std::sync::atomic::AtomicBool>,
}

impl WhisperProvider {
    pub fn new(base_url: String) -> Self {
        Self {
            base_url,
            client: Self::make_client(),
            available: Arc::new(std::sync::atomic::AtomicBool::new(false)),
        }
    }

    fn make_client() -> reqwest::Client {
        reqwest::Client::builder()
            .timeout(Duration::from_secs(60))
            .build()
            .unwrap_or_default()
    }

    pub async fn refresh_availability(&self) -> bool {
        let ok = self.health_check().await;
        self.available
            .store(ok, std::sync::atomic::Ordering::SeqCst);
        ok
    }

    pub fn is_available(&self) -> bool {
        self.available.load(std::sync::atomic::Ordering::SeqCst)
    }

    async fn health_check(&self) -> bool {
        match self
            .client
            .get(format!("{}/v1/models", self.base_url))
            .timeout(Duration::from_secs(3))
            .send()
            .await
        {
            Ok(resp) => resp.status().is_success(),
            Err(_) => false,
        }
    }
}

#[async_trait::async_trait]
impl SpeechToText for WhisperProvider {
    async fn transcribe(&self, audio: &[u8], language: &str) -> Result<String, VoiceError> {
        if !self.is_available() {
            return Err(VoiceError::TranscriptionFailed(
                "Whisper server not available".into(),
            ));
        }

        let form = reqwest::multipart::Form::new()
            .part(
                "file",
                reqwest::multipart::Part::bytes(audio.to_vec())
                    .file_name("audio.wav")
                    .mime_str("audio/wav")
                    .map_err(|e| VoiceError::TranscriptionFailed(e.to_string()))?,
            )
            .text("model", "whisper-1")
            .text("language", language.to_string());

        let response = self
            .client
            .post(format!("{}/v1/audio/transcriptions", self.base_url))
            .multipart(form)
            .send()
            .await
            .map_err(|e| VoiceError::TranscriptionFailed(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            return Err(VoiceError::TranscriptionFailed(format!(
                "Whisper returned HTTP {status}"
            )));
        }

        let body: serde_json::Value = response
            .json()
            .await
            .map_err(|e| VoiceError::TranscriptionFailed(e.to_string()))?;

        let text = body["text"].as_str().unwrap_or("").to_string();

        Ok(text)
    }

    async fn health_check(&self) -> bool {
        self.refresh_availability().await
    }
}

// ---------------------------------------------------------------------------
// VoicePipeline
// ---------------------------------------------------------------------------

/// Pipeline de voz que orquesta STT y TTS.
pub struct VoicePipeline {
    stt: Option<Box<dyn SpeechToText>>,
    tts: Option<Box<dyn TextToSpeech>>,
    enabled: bool,
    auto_speak: bool,
    language: String,
    tts_url: String,
    stt_url: String,
    voice_id: String,
}

impl VoicePipeline {
    pub fn new() -> Self {
        Self {
            stt: None,
            tts: None,
            enabled: false,
            auto_speak: false,
            language: "es".into(),
            tts_url: "http://localhost:8880".into(),
            stt_url: "http://localhost:8080".into(),
            voice_id: "af_heart".into(),
        }
    }

    /// Configura los providers según la configuración de voz.
    pub async fn init_from_config(&mut self, config: &VoiceConfig) {
        self.enabled = config.enabled;
        self.auto_speak = config.auto_speak;
        self.language = config.language.clone();
        self.tts_url = config.tts_url.clone();
        self.stt_url = config.stt_url.clone();
        self.voice_id = config.voice_id.clone();

        // Inicializar Kokoro TTS si está configurado
        if config.enabled && config.tts_provider == "kokoro" {
            let voice = config.voice_id.clone();
            let kokoro = KokoroProvider::new(config.tts_url.clone(), voice);
            if kokoro.refresh_availability().await {
                self.tts = Some(Box::new(kokoro));
            }
        }

        // Inicializar Whisper STT si está configurado
        if config.enabled && config.stt_provider == "whisper" {
            let whisper = WhisperProvider::new(config.stt_url.clone());
            if whisper.refresh_availability().await {
                self.stt = Some(Box::new(whisper));
            }
        }
    }

    #[allow(dead_code)]
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    #[allow(dead_code)]
    pub fn set_auto_speak(&mut self, auto_speak: bool) {
        self.auto_speak = auto_speak;
    }

    #[allow(dead_code)]
    pub fn set_language(&mut self, language: String) {
        self.language = language;
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    #[allow(dead_code)]
    pub fn is_auto_speak(&self) -> bool {
        self.auto_speak
    }

    #[allow(dead_code)]
    pub fn language(&self) -> &str {
        &self.language
    }

    #[allow(dead_code)]
    pub fn set_stt(&mut self, stt: Box<dyn SpeechToText>) {
        self.stt = Some(stt);
    }

    #[allow(dead_code)]
    pub fn set_tts(&mut self, tts: Box<dyn TextToSpeech>) {
        self.tts = Some(tts);
    }

    pub fn tts_available(&self) -> bool {
        self.tts.is_some()
    }

    #[allow(dead_code)]
    pub fn stt_available(&self) -> bool {
        self.stt.is_some()
    }

    /// Procesa audio de entrada y devuelve texto transcrito.
    pub async fn transcribe(&self, audio: &[u8]) -> Result<String, VoiceError> {
        let stt = self.stt.as_ref().ok_or(VoiceError::NotAvailable)?;
        stt.transcribe(audio, &self.language).await
    }

    /// Convierte texto a audio.
    pub async fn synthesize(&self, text: &str) -> Result<Vec<u8>, VoiceError> {
        let tts = self.tts.as_ref().ok_or(VoiceError::NotAvailable)?;
        tts.synthesize(text, &self.language).await
    }

    /// Devuelve estado de salud de los providers.
    pub async fn health_check(&self) -> VoiceHealth {
        let tts_ok = match &self.tts {
            Some(t) => t.health_check().await,
            None => false,
        };
        let stt_ok = match &self.stt {
            Some(s) => s.health_check().await,
            None => false,
        };
        VoiceHealth {
            tts_available: tts_ok,
            stt_available: stt_ok,
            tts_url: self.tts_url.clone(),
            stt_url: self.stt_url.clone(),
        }
    }
}

impl Default for VoicePipeline {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Config & Types
// ---------------------------------------------------------------------------

/// Configuración de voz del assistant.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VoiceConfig {
    pub enabled: bool,
    pub auto_speak: bool,
    pub language: String,
    /// Proveedor STT seleccionado ("whisper" | "web" | "none").
    pub stt_provider: String,
    /// Proveedor TTS seleccionado ("kokoro" | "web" | "none").
    pub tts_provider: String,
    /// URL del servidor Kokoro TTS.
    pub tts_url: String,
    /// URL del servidor Whisper STT.
    pub stt_url: String,
    /// Velocidad de habla del TTS (0.5–2.0, default 1.0).
    pub speech_rate: f32,
    /// Volumen del TTS (0.0–1.0, default 1.0).
    pub volume: f32,
    /// ID de voz Kokoro (ej. "af_heart", "ef_dora").
    #[serde(default = "default_voice_id")]
    pub voice_id: String,
}

fn default_voice_id() -> String {
    "af_heart".into()
}

impl Default for VoiceConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            auto_speak: false,
            language: "es".into(),
            stt_provider: "web".into(),
            tts_provider: "web".into(),
            tts_url: "http://localhost:8880".into(),
            stt_url: "http://localhost:8080".into(),
            speech_rate: 1.0,
            volume: 1.0,
            voice_id: "af_heart".into(),
        }
    }
}

/// Estado de la grabación de voz.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VoiceRecordingState {
    pub recording: bool,
    pub available: bool,
    pub provider: String,
}

/// Estado de salud de los providers de voz.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VoiceHealth {
    pub tts_available: bool,
    pub stt_available: bool,
    pub tts_url: String,
    pub stt_url: String,
}

/// Errores del pipeline de voz.
#[derive(Debug)]
#[allow(dead_code)]
pub enum VoiceError {
    NotAvailable,
    TranscriptionFailed(String),
    SynthesisFailed(String),
    PermissionDenied,
    Unsupported,
}

/// Información de una voz disponible para el frontend.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VoiceInfo {
    pub id: String,
    pub name: String,
}

/// Información sobre el acento nativo de un idioma.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AccentInfo {
    /// Nombre legible del acento (ej. "Paisa — Colombia").
    pub label: String,
    /// Código interno del acento.
    pub code: String,
    /// Si Kokoro dispone nativamente de una voz con este acento.
    pub native_available: bool,
    /// Mensaje de limitación (solo cuando `native_available` es false).
    pub limitation: Option<String>,
}

/// Devuelve las voces disponibles para un idioma dado.
pub fn list_voices(language: &str) -> Vec<VoiceInfo> {
    available_voices(language)
        .into_iter()
        .map(|(id, name)| VoiceInfo {
            id: id.into(),
            name: name.into(),
        })
        .collect()
}

/// Devuelve información del acento para un idioma dado.
pub fn get_accent_info(language: &str) -> AccentInfo {
    desired_accent(language)
}

impl std::fmt::Display for VoiceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VoiceError::NotAvailable => write!(f, "Voice not available"),
            VoiceError::TranscriptionFailed(e) => write!(f, "Transcription failed: {e}"),
            VoiceError::SynthesisFailed(e) => write!(f, "Synthesis failed: {e}"),
            VoiceError::PermissionDenied => write!(f, "Microphone permission denied"),
            VoiceError::Unsupported => write!(f, "Voice not supported in this environment"),
        }
    }
}

impl std::error::Error for VoiceError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_voice_pipeline_default() {
        let pipeline = VoicePipeline::new();
        assert!(!pipeline.is_enabled());
        assert!(!pipeline.is_auto_speak());
    }

    #[test]
    fn test_voice_config_default() {
        let config = VoiceConfig::default();
        assert!(!config.enabled);
        assert!(!config.auto_speak);
        assert_eq!(config.stt_provider, "web");
        assert_eq!(config.tts_provider, "web");
        assert_eq!(config.tts_url, "http://localhost:8880");
        assert_eq!(config.stt_url, "http://localhost:8080");
        assert_eq!(config.voice_id, "af_heart");
    }

    #[test]
    fn test_voice_pipeline_setters() {
        let mut pipeline = VoicePipeline::new();
        pipeline.set_enabled(true);
        assert!(pipeline.is_enabled());
        pipeline.set_auto_speak(true);
        assert!(pipeline.is_auto_speak());
        pipeline.set_language("en".into());
        assert_eq!(pipeline.language(), "en");
    }

    #[test]
    fn test_voice_pipeline_not_available_initially() {
        let pipeline = VoicePipeline::new();
        assert!(!pipeline.tts_available());
        assert!(!pipeline.stt_available());
    }

    #[test]
    fn test_voice_error_display() {
        let err = VoiceError::NotAvailable;
        assert!(!format!("{err}").is_empty());
        let err = VoiceError::SynthesisFailed("test".into());
        assert!(format!("{err}").contains("test"));
        let err = VoiceError::TranscriptionFailed("stt err".into());
        assert!(format!("{err}").contains("stt err"));
    }

    #[test]
    fn test_voice_config_custom() {
        let config = VoiceConfig {
            enabled: true,
            auto_speak: true,
            language: "en".into(),
            tts_provider: "kokoro".into(),
            stt_provider: "whisper".into(),
            tts_url: "http://custom:8880".into(),
            stt_url: "http://custom:8080".into(),
            speech_rate: 1.2,
            volume: 0.8,
            voice_id: "af_bella".into(),
        };
        assert!(config.enabled);
        assert!(config.auto_speak);
        assert_eq!(config.language, "en");
        assert_eq!(config.tts_provider, "kokoro");
        assert_eq!(config.stt_provider, "whisper");
        assert!((config.speech_rate - 1.2).abs() < f32::EPSILON);
        assert!((config.volume - 0.8).abs() < f32::EPSILON);
        assert_eq!(config.voice_id, "af_bella");
    }

    #[test]
    fn test_voice_pipeline_language_default() {
        let pipeline = VoicePipeline::new();
        assert_eq!(pipeline.language(), "es");
    }

    #[test]
    fn test_default_voice_for_language() {
        assert_eq!(default_voice_for_language("es"), "ef_dora");
        assert_eq!(default_voice_for_language("en"), "af_heart");
        assert_eq!(default_voice_for_language("pt"), "pf_dora");
        assert_eq!(default_voice_for_language("fr"), "af_heart"); // fallback
    }

    #[test]
    fn test_available_voices_es() {
        let voices = available_voices("es");
        assert_eq!(voices.len(), 1); // Only ef_dora
        assert_eq!(voices[0].0, "ef_dora");
    }

    #[test]
    fn test_available_voices_en() {
        let voices = available_voices("en");
        assert!(voices.len() >= 10); // 11 female American voices
        assert!(voices.iter().any(|(id, _)| *id == "af_heart"));
        assert!(voices.iter().any(|(id, _)| *id == "af_bella"));
        assert!(voices.iter().any(|(id, _)| *id == "af_sarah"));
    }

    #[test]
    fn test_list_voices() {
        let voices = list_voices("es");
        assert!(!voices.is_empty());
        assert!(voices.iter().any(|v| v.id == "ef_dora"));
    }

    #[test]
    fn test_desired_accent_es() {
        let accent = desired_accent("es");
        assert_eq!(accent.code, "co-paisa");
        assert!(!accent.native_available);
        assert!(accent.limitation.is_some());
    }

    #[test]
    fn test_desired_accent_en() {
        let accent = desired_accent("en");
        assert_eq!(accent.code, "us-nyc");
        assert!(!accent.native_available);
        assert!(accent.limitation.is_some());
    }

    #[test]
    fn test_desired_accent_pt() {
        let accent = desired_accent("pt");
        assert_eq!(accent.code, "br");
        assert!(accent.native_available);
        assert!(accent.limitation.is_none());
    }

    #[test]
    fn test_get_accent_info() {
        let info = get_accent_info("es");
        assert_eq!(info.label, "Paisa \u{2014} Colombia");
    }
}
