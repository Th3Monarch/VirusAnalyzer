use super::context::ApplicationContext;
use super::personality::{DetailLevel, Personality, Tone};

/// Construye el system prompt para el AI basado en el contexto actual.
///
/// El prompt se adapta al:
/// - Idioma del usuario (ES/EN)
/// - Nivel de amenaza actual
/// - Página y estado de la aplicación
/// - Protocolos activos (Ysmel/Fenix)
/// - Personalidad del assistant
pub struct PromptBuilder;

impl PromptBuilder {
    pub fn new() -> Self {
        Self
    }

    /// Construye el system prompt completo adaptado al contexto.
    pub fn build(&self, personality: &Personality, context: &ApplicationContext) -> String {
        let lang = if context.language.starts_with("es") {
            Lang::Es
        } else {
            Lang::En
        };

        let mut sections: Vec<String> = Vec::new();

        // 1. Identidad y personalidad
        sections.push(self.section_identity(personality, lang));

        // 2. Rol y capacidades
        sections.push(self.section_role(lang));

        // 3. Restricciones de seguridad
        sections.push(self.section_restrictions(lang));

        // 4. Contexto actual de la aplicación
        sections.push(self.section_context(context, lang));

        // 5. Acciones disponibles según el estado
        sections.push(self.section_available_actions(context, lang));

        // 6. Protocolos activos
        if let Some(protocols) = self.section_protocols(context, lang) {
            sections.push(protocols);
        }

        // 7. Adaptación al nivel de amenaza
        if let Some(threat) = self.section_threat_adaptation(context, lang) {
            sections.push(threat);
        }

        // 8. Formato de respuesta
        sections.push(self.section_response_format(lang));

        sections.join("\n\n")
    }

    fn section_identity(&self, personality: &Personality, lang: Lang) -> String {
        let tone_desc = match (personality.tone, lang) {
            (Tone::Professional, Lang::Es) => "Sé profesional, preciso y objetivo.",
            (Tone::Professional, Lang::En) => "Be professional, precise, and objective.",
            (Tone::Friendly, Lang::Es) => "Sé cercano, amigable y alentador.",
            (Tone::Friendly, Lang::En) => "Be warm, approachable, and encouraging.",
            (Tone::Technical, Lang::Es) => "Sé detallado, técnico y minucioso.",
            (Tone::Technical, Lang::En) => "Be detailed, technical, and thorough.",
        };

        let detail_desc = match (personality.detail_level, lang) {
            (DetailLevel::Brief, Lang::Es) => "Mantén las respuestas cortas y directas.",
            (DetailLevel::Brief, Lang::En) => "Keep responses short and to the point.",
            (DetailLevel::Normal, Lang::Es) => "Proporciona respuestas equilibradas con contexto apropiado.",
            (DetailLevel::Normal, Lang::En) => "Provide balanced responses with appropriate context.",
            (DetailLevel::Detailed, Lang::Es) => "Proporciona explicaciones completas con ejemplos.",
            (DetailLevel::Detailed, Lang::En) => "Provide comprehensive explanations with examples.",
        };

        let emoji_desc = match (personality.use_emojis, lang) {
            (true, Lang::Es) => "Usa emojis relevantes ocasionalmente.",
            (true, Lang::En) => "Use relevant emojis occasionally.",
            (false, Lang::Es) => "No uses emojis.",
            (false, Lang::En) => "Do not use emojis.",
        };

        format!(
            "## Identidad\n\
             Eres {name}, un companion de seguridad AI para Prometeo.\n\
             {tone_desc}\n\
             {detail_desc}\n\
             {emoji_desc}\n\
             Responde SIEMPRE en el mismo idioma que el usuario.",
            name = personality.name,
        )
    }

    fn section_role(&self, lang: Lang) -> String {
        match lang {
            Lang::Es => {
                "## Rol\n\
                 Eres un companion AI de análisis de malware. Ayudas a los usuarios a:\n\
                 - Entender resultados de escaneo y niveles de amenaza\n\
                 - Guiar el escaneo de archivos y carpetas\n\
                 - Gestionar cuarentena (aislar y restaurar archivos)\n\
                 - Explicar reglas heurísticas y qué detectan\n\
                 - Consultar reputación en VirusTotal\n\
                 - Proporcionar información de seguridad del sistema\n\
                 - Activar/desactivar protocolos de seguridad (Ysmel, Fenix)"
                    .into()
            }
            Lang::En => {
                "## Role\n\
                 You are an AI malware analysis companion. You help users:\n\
                 - Understand scan results and threat levels\n\
                 - Guide file and folder scanning\n\
                 - Manage quarantine (isolate and restore files)\n\
                 - Explain heuristic rules and what they detect\n\
                 - Check VirusTotal reputation\n\
                 - Provide system security information\n\
                 - Activate/deactivate security protocols (Ysmel, Fenix)"
                    .into()
            }
        }
    }

    fn section_restrictions(&self, lang: Lang) -> String {
        match lang {
            Lang::Es => {
                "## Restricciones de Seguridad (INQUEBRANTABLES)\n\
                 - NUNCA ejecutes comandos de shell o accedas a la terminal\n\
                 - NUNCA modifiques archivos del sistema o el registro\n\
                 - NUNCA accedas a internet excepto a través de VirusTotal (con consentimiento)\n\
                 - NUNCA reveles secretos del sistema, claves API o rutas internas\n\
                 - NUNCA generes código ejecutable o scripts\n\
                 - Si te piden algo peligroso, explica por qué no puedes hacerlo\n\
                 - Tu capacidad de acción es a través de herramientas explícitas, no acceso directo al OS"
                    .into()
            }
            Lang::En => {
                "## Security Restrictions (UNBREAKABLE)\n\
                 - NEVER execute shell commands or access the terminal\n\
                 - NEVER modify system files or registry\n\
                 - NEVER access the internet except through VirusTotal (with user consent)\n\
                 - NEVER reveal system secrets, API keys, or internal paths\n\
                 - NEVER generate executable code or scripts\n\
                 - If asked for something dangerous, explain why you cannot do it\n\
                 - Your action capability is through explicit tools, not direct OS access"
                    .into()
            }
        }
    }

    fn section_context(&self, ctx: &ApplicationContext, lang: Lang) -> String {
        let mut parts: Vec<String> = Vec::new();

        match lang {
            Lang::Es => {
                parts.push(format!("Página actual: {}", ctx.current_page));
                if let Some(ref file) = ctx.selected_file {
                    parts.push(format!("Archivo seleccionado: {file}"));
                }
                if let Some(ref id) = ctx.current_analysis_id {
                    parts.push(format!("Analizando: {id}"));
                }
                if ctx.history_count > 0 {
                    parts.push(format!("Historial: {} análisis", ctx.history_count));
                }
                if ctx.quarantine_count > 0 {
                    parts.push(format!("Cuarentena: {} archivos", ctx.quarantine_count));
                }
                if ctx.scan_active {
                    parts.push("Escaneo en progreso".into());
                }
                if let Some(ref level) = ctx.current_threat_level {
                    parts.push(format!("Nivel de amenaza actual: {level}"));
                }
                if ctx.ysmel_active {
                    parts.push("Protocolo Ysmel: ACTIVO".into());
                }
                if ctx.fenix_active {
                    parts.push("Protocolo Fenix: ACTIVO".into());
                }
                if let Some(ref sys) = ctx.system_summary {
                    parts.push(format!("Sistema: {sys}"));
                }
                format!("## Contexto Actual\n{}", parts.join("\n"))
            }
            Lang::En => {
                parts.push(format!("Current page: {}", ctx.current_page));
                if let Some(ref file) = ctx.selected_file {
                    parts.push(format!("Selected file: {file}"));
                }
                if let Some(ref id) = ctx.current_analysis_id {
                    parts.push(format!("Viewing analysis: {id}"));
                }
                if ctx.history_count > 0 {
                    parts.push(format!("History: {} analyses", ctx.history_count));
                }
                if ctx.quarantine_count > 0 {
                    parts.push(format!("Quarantine: {} files", ctx.quarantine_count));
                }
                if ctx.scan_active {
                    parts.push("Scan in progress".into());
                }
                if let Some(ref level) = ctx.current_threat_level {
                    parts.push(format!("Current threat level: {level}"));
                }
                if ctx.ysmel_active {
                    parts.push("Ysmel protocol: ACTIVE".into());
                }
                if ctx.fenix_active {
                    parts.push("Fenix protocol: ACTIVE".into());
                }
                if let Some(ref sys) = ctx.system_summary {
                    parts.push(format!("System: {sys}"));
                }
                format!("## Current Context\n{}", parts.join("\n"))
            }
        }
    }

    fn section_available_actions(&self, ctx: &ApplicationContext, lang: Lang) -> String {
        let mut actions: Vec<String> = Vec::new();

        // Acciones siempre disponibles
        match lang {
            Lang::Es => {
                actions.push("- Consultar información del sistema".into());
                actions.push("- Ver historial de análisis".into());
                actions.push("- Explicar reglas heurísticas".into());
                if ctx.quarantine_count > 0 {
                    actions.push("- Ver archivos en cuarentena".into());
                }
            }
            Lang::En => {
                actions.push("- Get system information".into());
                actions.push("- View analysis history".into());
                actions.push("- Explain heuristic rules".into());
                if ctx.quarantine_count > 0 {
                    actions.push("- View quarantined files".into());
                }
            }
        }

        // Acciones condicionales según estado
        if !ctx.fenix_active {
            match lang {
                Lang::Es => {
                    actions.push("- Analizar un archivo (necesita ruta)".into());
                    actions.push("- Poner archivo en cuarentena (requiere confirmación)".into());
                    actions.push("- Restaurar archivo de cuarentena (requiere confirmación)".into());
                    actions.push("- Generar informe de análisis".into());
                    actions.push("- Consultar VirusTotal (necesita hash)".into());
                }
                Lang::En => {
                    actions.push("- Analyze a file (needs path)".into());
                    actions.push("- Quarantine a file (requires confirmation)".into());
                    actions.push("- Restore file from quarantine (requires confirmation)".into());
                    actions.push("- Generate analysis report".into());
                    actions.push("- Query VirusTotal (needs hash)".into());
                }
            }
        } else {
            match lang {
                Lang::Es => {
                    actions.push("- Analizar archivo (solo lectura, permitido en Fenix)".into());
                    actions.push("- Generar informe (permitido en Fenix)".into());
                    actions.push("- Consultar VirusTotal (permitido en Fenix)".into());
                    actions.push("⚠️ Cuarentena y restauración BLOQUEADOS en modo Fenix".into());
                }
                Lang::En => {
                    actions.push("- Analyze file (read-only, allowed in Fenix)".into());
                    actions.push("- Generate report (allowed in Fenix)".into());
                    actions.push("- Query VirusTotal (allowed in Fenix)".into());
                    actions.push("⚠️ Quarantine and restore BLOCKED in Fenix mode".into());
                }
            }
        }

        // Protocolos
        if !ctx.ysmel_active && !ctx.fenix_active {
            match lang {
                Lang::Es => {
                    actions.push("- Activar protocolo Ysmel (aislamiento de red)".into());
                    actions.push("- Activar protocolo Fenix (modo recuperación)".into());
                }
                Lang::En => {
                    actions.push("- Activate Ysmel protocol (network isolation)".into());
                    actions.push("- Activate Fenix protocol (recovery mode)".into());
                }
            }
        } else if ctx.ysmel_active {
            match lang {
                Lang::Es => {
                    actions.push("- Desactivar protocolo Ysmel".into());
                }
                Lang::En => {
                    actions.push("- Deactivate Ysmel protocol".into());
                }
            }
        }

        let header = match lang {
            Lang::Es => "## Acciones Disponibles",
            Lang::En => "## Available Actions",
        };

        format!("{header}\n{}", actions.join("\n"))
    }

    fn section_protocols(&self, ctx: &ApplicationContext, lang: Lang) -> Option<String> {
        if ctx.ysmel_active && ctx.fenix_active {
            return match lang {
                Lang::Es => Some(
                    "## Protocolos Activos\n\
                     🔒 Ysmel: ACTIVO — Aislamiento de red habilitado. No se permiten conexiones externas.\n\
                     🔄 Fenix: ACTIVO — Modo recuperación. Solo operaciones de lectura permitidas."
                        .into(),
                ),
                Lang::En => Some(
                    "## Active Protocols\n\
                     🔒 Ysmel: ACTIVE — Network isolation enabled. No external connections allowed.\n\
                     🔄 Fenix: ACTIVE — Recovery mode. Read-only operations allowed."
                        .into(),
                ),
            };
        }

        if ctx.ysmel_active {
            return match lang {
                Lang::Es => Some(
                    "## Protocolo Ysmel: ACTIVO\n\
                     El sistema está aislado de la red. Algunas funciones pueden estar limitadas.\n\
                     VirusTotal no está disponible. enfócate en análisis local."
                        .into(),
                ),
                Lang::En => Some(
                    "## Ysmel Protocol: ACTIVE\n\
                     The system is isolated from the network. Some features may be limited.\n\
                     VirusTotal is unavailable. Focus on local analysis."
                        .into(),
                ),
            };
        }

        if ctx.fenix_active {
            return match lang {
                Lang::Es => Some(
                    "## Protocolo Fenix: ACTIVO\n\
                     Modo de recuperación activo. Solo operaciones de lectura.\n\
                     Enfócate en evaluar el estado del sistema y sugerir remediación.\n\
                     NO realices cuarentena ni restauración (bloqueado por Fenix)."
                        .into(),
                ),
                Lang::En => Some(
                    "## Fenix Protocol: ACTIVE\n\
                     Recovery mode active. Read-only operations only.\n\
                     Focus on assessing system state and suggesting remediation.\n\
                     DO NOT quarantine or restore (blocked by Fenix)."
                        .into(),
                ),
            };
        }

        None
    }

    fn section_threat_adaptation(&self, ctx: &ApplicationContext, lang: Lang) -> Option<String> {
        let level = ctx.current_threat_level.as_deref()?;

        let is_high_threat = matches!(
            level.to_lowercase().as_str(),
            "high" | "critical" | "alto" | "crítico"
        );

        if is_high_threat {
            return match lang {
                Lang::Es => Some(format!(
                    "## ⚠️ Nivel de Amenaza: {level}\n\
                     El sistema detectó una amenaza significativa.\n\
                     - Sé más directo y urgente en tus respuestas\n\
                     - Prioriza explicar el nivel de riesgo\n\
                     - Sugiere acciones inmediatas (cuarentena, escaneo profundo)\n\
                     - No minimices la severidad"
                )),
                Lang::En => Some(format!(
                    "## ⚠️ Threat Level: {level}\n\
                     The system detected a significant threat.\n\
                     - Be more direct and urgent in your responses\n\
                     - Prioritize explaining the risk level\n\
                     - Suggest immediate actions (quarantine, deep scan)\n\
                     - Do not minimize the severity"
                )),
            };
        }

        None
    }

    fn section_response_format(&self, lang: Lang) -> String {
        match lang {
            Lang::Es => {
                "## Formato de Respuesta\n\
                 - Sé conciso pero informativo\n\
                 - Usa viñetas para explicar resultados\n\
                 - Sé claro sobre qué ocurrirá antes de sugerir acciones\n\
                 - Si necesitas confirmar una acción destructiva, indica claramente que requiere confirmación\n\
                 - Para análisis de archivos, estructura: Amenaza / Puntaje / Nivel de Riesgo / Recomendación"
                    .into()
            }
            Lang::En => {
                "## Response Format\n\
                 - Be concise but informative\n\
                 - Use bullet points to explain results\n\
                 - Be clear about what will happen before suggesting actions\n\
                 - If you need to confirm a destructive action, clearly indicate it requires confirmation\n\
                 - For file analysis, structure: Threat / Score / Risk Level / Recommendation"
                    .into()
            }
        }
    }
}

impl Default for PromptBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Copy)]
enum Lang {
    Es,
    En,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_context() -> ApplicationContext {
        let mut ctx = ApplicationContext::new();
        ctx.language = "es".into();
        ctx.current_page = "results".into();
        ctx
    }

    #[test]
    fn test_build_prompt_contains_identity() {
        let builder = PromptBuilder::new();
        let personality = Personality::default();
        let context = test_context();
        let prompt = builder.build(&personality, &context);
        assert!(prompt.contains("Prometeo"));
        assert!(prompt.contains("Identidad"));
    }

    #[test]
    fn test_build_prompt_contains_restrictions() {
        let builder = PromptBuilder::new();
        let personality = Personality::default();
        let context = test_context();
        let prompt = builder.build(&personality, &context);
        assert!(prompt.contains("NUNCA"));
        assert!(prompt.contains("shell"));
    }

    #[test]
    fn test_build_prompt_english() {
        let builder = PromptBuilder::new();
        let personality = Personality::default();
        let mut context = test_context();
        context.language = "en".into();
        let prompt = builder.build(&personality, &context);
        assert!(prompt.contains("Role"));
        assert!(prompt.contains("NEVER"));
    }

    #[test]
    fn test_build_prompt_with_threat() {
        let builder = PromptBuilder::new();
        let personality = Personality::default();
        let mut context = test_context();
        context.current_threat_level = Some("high".into());
        let prompt = builder.build(&personality, &context);
        assert!(prompt.contains("Amenaza"));
        assert!(prompt.contains("urgente"));
    }

    #[test]
    fn test_build_prompt_with_ysmel() {
        let builder = PromptBuilder::new();
        let personality = Personality::default();
        let mut context = test_context();
        context.ysmel_active = true;
        let prompt = builder.build(&personality, &context);
        assert!(prompt.contains("Ysmel"));
        assert!(prompt.contains("aislado"));
    }

    #[test]
    fn test_build_prompt_with_fenix_blocks_quarantine() {
        let builder = PromptBuilder::new();
        let personality = Personality::default();
        let mut context = test_context();
        context.fenix_active = true;
        let prompt = builder.build(&personality, &context);
        assert!(prompt.contains("Fenix"));
        assert!(prompt.contains("BLOQUEADOS"));
    }

    #[test]
    fn test_build_prompt_actions_differ_by_fenix() {
        let builder = PromptBuilder::new();
        let personality = Personality::default();

        let mut ctx_normal = test_context();
        ctx_normal.fenix_active = false;
        let prompt_normal = builder.build(&personality, &ctx_normal);

        let mut ctx_fenix = test_context();
        ctx_fenix.fenix_active = true;
        let prompt_fenix = builder.build(&personality, &ctx_fenix);

        assert!(prompt_normal.contains("Poner archivo en cuarentena"));
        assert!(prompt_fenix.contains("BLOQUEADOS"));
        assert!(!prompt_fenix.contains("Poner archivo en cuarentena"));
    }

    #[test]
    fn test_build_prompt_with_selected_file() {
        let builder = PromptBuilder::new();
        let personality = Personality::default();
        let mut context = test_context();
        context.selected_file = Some("C:\\suspicious.exe".into());
        let prompt = builder.build(&personality, &context);
        assert!(prompt.contains("C:\\suspicious.exe"));
    }

    #[test]
    fn test_build_prompt_with_scan_summary() {
        let builder = PromptBuilder::new();
        let personality = Personality::default();
        let mut context = test_context();
        context.last_scan_summary = Some("3 threats found in last scan".into());
        let prompt = builder.build(&personality, &context);
        assert!(prompt.contains("Contexto"));
    }

    #[test]
    fn test_build_prompt_total_threats() {
        let builder = PromptBuilder::new();
        let personality = Personality::default();
        let mut context = test_context();
        context.total_threats_detected = 42;
        let prompt = builder.build(&personality, &context);
        assert!(prompt.contains("Contexto"));
    }

    #[test]
    fn test_build_prompt_english_threat_level() {
        let builder = PromptBuilder::new();
        let personality = Personality::default();
        let mut context = test_context();
        context.language = "en".into();
        context.current_threat_level = Some("critical".into());
        let prompt = builder.build(&personality, &context);
        assert!(prompt.contains("Threat"));
        assert!(prompt.contains("immediate"));
    }

    #[test]
    fn test_build_prompt_no_optional_fields() {
        let builder = PromptBuilder::new();
        let personality = Personality::default();
        let context = test_context();
        let prompt = builder.build(&personality, &context);
        assert!(!prompt.contains("Selected file:"));
        assert!(!prompt.contains("Last scan:"));
        assert!(!prompt.contains("Analysis ID:"));
    }

    #[test]
    fn test_build_prompt_response_format() {
        let builder = PromptBuilder::new();
        let personality = Personality::default();
        let context = test_context();
        let prompt = builder.build(&personality, &context);
        assert!(prompt.contains("Formato de Respuesta"));
    }

    #[test]
    fn test_build_prompt_ysmel_blocks_network() {
        let builder = PromptBuilder::new();
        let personality = Personality::default();
        let mut context = test_context();
        context.ysmel_active = true;
        context.language = "en".into();
        let prompt = builder.build(&personality, &context);
        assert!(prompt.contains("Ysmel"));
        assert!(prompt.contains("isolated"));
    }
}
