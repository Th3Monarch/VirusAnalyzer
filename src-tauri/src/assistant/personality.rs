use serde::{Deserialize, Serialize};

/// Personalidad del assistant adaptable al contexto.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Personality {
    /// Nombre del asistente.
    pub name: String,
    /// Tono de comunicación.
    pub tone: Tone,
    /// Nivel de detalle en las respuestas.
    pub detail_level: DetailLevel,
    /// Si usa emojis en las respuestas.
    pub use_emojis: bool,
}

/// Tono de comunicación.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Tone {
    /// Profesional y directo.
    Professional,
    /// Amigable y cercano.
    Friendly,
    /// Técnico y detallado.
    Technical,
}

/// Nivel de detalle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DetailLevel {
    /// Respuestas cortas y al punto.
    Brief,
    /// Respuestas equilibradas.
    Normal,
    /// Respuestas detalladas con explicaciones.
    Detailed,
}

impl Default for Personality {
    fn default() -> Self {
        Self {
            name: "Prometeo".into(),
            tone: Tone::Professional,
            detail_level: DetailLevel::Normal,
            use_emojis: false,
        }
    }
}

impl Personality {
    /// Instrucciones de sistema en el idioma especificado.
    #[allow(dead_code)]
    pub fn system_instructions_for_lang(&self, lang: &str) -> String {
        let is_es = lang.starts_with("es");

        let tone_desc = match (self.tone, is_es) {
            (Tone::Professional, true) => "Sé profesional, preciso y objetivo.",
            (Tone::Professional, false) => "Be professional, precise, and objective.",
            (Tone::Friendly, true) => "Sé cercano, amigable y alentador.",
            (Tone::Friendly, false) => "Be warm, approachable, and encouraging.",
            (Tone::Technical, true) => "Sé detallado, técnico y minucioso.",
            (Tone::Technical, false) => "Be detailed, technical, and thorough.",
        };

        let detail_desc = match (self.detail_level, is_es) {
            (DetailLevel::Brief, true) => "Mantén las respuestas cortas y directas.",
            (DetailLevel::Brief, false) => "Keep responses short and to the point.",
            (DetailLevel::Normal, true) => "Proporciona respuestas equilibradas con contexto apropiado.",
            (DetailLevel::Normal, false) => "Provide balanced responses with appropriate context.",
            (DetailLevel::Detailed, true) => "Proporciona explicaciones completas con ejemplos.",
            (DetailLevel::Detailed, false) => "Provide comprehensive explanations with examples.",
        };

        let emoji_desc = match (self.use_emojis, is_es) {
            (true, true) => "Usa emojis relevantes ocasionalmente.",
            (true, false) => "Use relevant emojis occasionally.",
            (false, true) => "No uses emojis.",
            (false, false) => "Do not use emojis.",
        };

        let always_lang = if is_es {
            "Responde SIEMPRE en el mismo idioma que el usuario."
        } else {
            "Always respond in the same language as the user."
        };

        format!(
            "You are {name}, an AI security companion.\n\
             {tone_desc}\n\
             {detail_desc}\n\
             {emoji_desc}\n\
             {always_lang}",
            name = self.name,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_personality() {
        let p = Personality::default();
        assert_eq!(p.name, "Prometeo");
        assert_eq!(p.tone, Tone::Professional);
        assert_eq!(p.detail_level, DetailLevel::Normal);
        assert!(!p.use_emojis);
    }

    #[test]
    fn test_system_instructions_spanish() {
        let p = Personality::default();
        let instructions = p.system_instructions_for_lang("es");
        assert!(instructions.contains("profesional"));
        assert!(instructions.contains("equilibradas"));
        assert!(instructions.contains("mismo idioma"));
    }

    #[test]
    fn test_system_instructions_english() {
        let p = Personality::default();
        let instructions = p.system_instructions_for_lang("en");
        assert!(instructions.contains("professional"));
        assert!(instructions.contains("balanced"));
        assert!(instructions.contains("same language"));
    }

    #[test]
    fn test_system_instructions_friendly_tone() {
        let p = Personality {
            tone: Tone::Friendly,
            use_emojis: true,
            ..Default::default()
        };
        let instructions = p.system_instructions_for_lang("en");
        assert!(instructions.contains("warm"));
        assert!(instructions.contains("emojis"));
    }

    #[test]
    fn test_system_instructions_technical_tone() {
        let p = Personality {
            tone: Tone::Technical,
            detail_level: DetailLevel::Detailed,
            ..Default::default()
        };
        let instructions = p.system_instructions_for_lang("es");
        assert!(instructions.contains("detallado"));
        assert!(instructions.contains("explicaciones completas"));
    }

    #[test]
    fn test_system_instructions_brief() {
        let p = Personality {
            detail_level: DetailLevel::Brief,
            ..Default::default()
        };
        let instructions = p.system_instructions_for_lang("es");
        assert!(instructions.contains("cortas y directas"));
    }
}
