//! Motor de evaluación explicativa (FASE 6).
//!
//! Sintetiza la evidencia ya extraída (hallazgos heurísticos, análisis
//! estático y reputación de VirusTotal) en un informe en lenguaje natural.
//!
//! Es un motor **determinista y sin red**: no consulta servicios de IA y no
//! inventa resultados. Toda afirmación procede de datos reales del análisis.
//!
//! Idioma: el informe se compone directamente en el idioma seleccionado
//! (`Language`), eligiendo el catálogo de plantillas `Lang` (ES/EN) en el
//! momento de generar. El motor no traduce texto producido en inglés: cada
//! idioma tiene su propio juego de plantillas. Los términos técnicos (APIs,
//! hashes, nombres y categorías de reglas) se conservan en su forma original.
//! La UI localiza sus rótulos estructurales por separado en el i18n del
//! frontend. Al finalizar se valida la lengua de la salida con una heurística
//! ligera y se registra cualquier desviación.

use crate::models::{
    AiAssessment, Finding, Language, StaticAnalysis, ThreatLevel, VirusTotalResult,
};

pub const VERDICT_CLEAN: &str = "clean";
pub const VERDICT_LIKELY_CLEAN: &str = "likely_clean";
pub const VERDICT_SUSPICIOUS: &str = "suspicious";
pub const VERDICT_MALICIOUS: &str = "malicious";

/// Reemplaza los marcadores `{nombre}` de una plantilla por sus valores.
fn fill(template: &str, pairs: &[(&str, &str)]) -> String {
    let mut out = template.to_string();
    for (key, value) in pairs {
        out = out.replace(key, value);
    }
    out
}

/// Orientación por categoría (impacto, consecuencias, acciones, vectores).
struct CatGuidance {
    impact: &'static str,
    consequences: &'static str,
    actions: &'static str,
    vectors: &'static [&'static str],
}

/// Catálogo de plantillas del motor, completo por idioma.
///
/// Cada idioma es una instancia de `Lang`; el motor elige una según el
/// `Language` recibido y compone el informe directamente con ella.
struct Lang {
    no_evidence: &'static str,
    vt_indicator_flagged: &'static str,
    vt_indicator_clean: &'static str,
    sum_clean: &'static str,
    sum_likely_clean: &'static str,
    sum_suspicious: &'static str,
    sum_malicious: &'static str,
    vt_note_flagged: &'static str,
    vt_note_clean: &'static str,
    vt_note_no_record: &'static str,
    exp_baseline: &'static str,
    exp_file_type: &'static str,
    exp_pe_kind: &'static str,
    exp_pe_dll: &'static str,
    exp_pe_exec: &'static str,
    exp_pe_signed: &'static str,
    exp_pe_unsigned: &'static str,
    exp_vt_flagged: &'static str,
    exp_vt_no_record: &'static str,
    exp_no_findings: &'static str,
    exp_cat_header: &'static str,
    exp_evidence: &'static str,
    cat_display: &'static [(&'static str, &'static str)],
    guidance: &'static [(&'static str, CatGuidance)],
    guid_no_actions: &'static [&'static str; 2],
    guid_no_impact: &'static str,
    /// Marcadores distintivos del idioma de salida (para validación).
    markers: &'static [&'static str],
    /// Marcadores del idioma contrario (no deben dominar la salida).
    counter_markers: &'static [&'static str],
}

const EN_CAT_DISPLAY: &[(&str, &str)] = &[
    ("process", "Process behavior"),
    ("persistence", "Persistence"),
    ("powershell", "PowerShell usage"),
    ("packing", "Packing / obfuscation"),
    ("network", "Network behavior"),
    ("signatures", "Known signatures"),
    ("general", "General"),
];

const EN_GUIDANCE: &[(&str, CatGuidance)] = &[
    (
        "process",
        CatGuidance {
            impact: "Code could be executed in the context of another process, bypassing security controls.",
            consequences: "Sensitive data (credentials, documents) could be read or stolen by injected code.",
            actions: "Treat the file as hostile, quarantine it and investigate which process created it.",
            vectors: &["Process injection and remote code execution"],
        },
    ),
    (
        "persistence",
        CatGuidance {
            impact: "The file could relaunch automatically on boot or user logon, surviving reboots.",
            consequences: "Repeated reinfection if persistence entries (registry, services, scheduled tasks) are not removed.",
            actions: "Delete the file and remove its startup entries before restarting the machine.",
            vectors: &["Persistence via autorun, service or scheduled task"],
        },
    ),
    (
        "powershell",
        CatGuidance {
            impact: "Payloads could be executed in memory through PowerShell without writing to disk.",
            consequences: "Fileless execution could download and run additional stages evasively.",
            actions: "Enable and review PowerShell ScriptBlock logging, and execute the script only in a sandbox.",
            vectors: &["Script-based execution (PowerShell)"],
        },
    ),
    (
        "packing",
        CatGuidance {
            impact: "The real code is hidden behind compression or encryption, limiting static analysis.",
            consequences: "Dynamic analysis (sandbox) is required to observe the unpacked behavior.",
            actions: "Run the file in an isolated VM with network and process monitoring.",
            vectors: &["Packed or obfuscated payload"],
        },
    ),
    (
        "network",
        CatGuidance {
            impact: "The file could reach a remote host to exfiltrate data or receive commands (C2).",
            consequences: "Data leaving the network increases the risk of follow-on attacks.",
            actions: "Block outbound connections from the affected host and review DNS, firewall and proxy logs.",
            vectors: &["Command and control over HTTP", "Data exfiltration over sockets"],
        },
    ),
    (
        "signatures",
        CatGuidance {
            impact: "The file matches a known malicious or test signature (for example EICAR).",
            consequences: "This is a confirmed indicator of a known-bad file.",
            actions: "Quarantine or delete the file immediately and report its hash to your security team.",
            vectors: &["Known-bad file"],
        },
    ),
    (
        "general",
        CatGuidance {
            impact: "A file that does not match its extension can trick users into opening it.",
            consequences: "Users may execute a file they believe is inert or safe.",
            actions: "Verify the real file type before opening it; do not execute unexpected content.",
            vectors: &["Type confusion / social engineering"],
        },
    ),
];

const EN: Lang = Lang {
    no_evidence: "no evidence",
    vt_indicator_flagged: "[reputation] VirusTotal flagged {n}/{total} engines ({names}).",
    vt_indicator_clean: "[reputation] VirusTotal reports {n}/{total} engines clean.",
    sum_clean: "\"{file}\" showed no suspicious indicators in its structure or content.",
    sum_likely_clean: "\"{file}\" shows only weak or benign signals ({n} low-severity finding(s)).",
    sum_suspicious: "\"{file}\" combines {n} suspicious indicator(s) across {cats_count}: {cats}.",
    sum_malicious: "\"{file}\" exhibits strong malware indicators ({tops}) with a threat score of {score}/100.",
    vt_note_flagged: " VirusTotal: {n}/{total} engines flagged it as malicious or suspicious.",
    vt_note_clean: " VirusTotal reported the hash as clean ({n}/{total} engines undetected).",
    vt_note_no_record: " VirusTotal had no record of the hash.",
    exp_baseline: "\"{file}\": static analysis inspected the file type, entropy, structure, content samples and (when configured) community reputation.",
    exp_file_type: " The file was identified as {type} (entropy {entropy} bits/byte).",
    exp_pe_kind: " It is a {arch}{kind} PE with {sections} sections and {imports} imported function(s).",
    exp_pe_dll: " DLL",
    exp_pe_exec: " executable",
    exp_pe_signed: " It carries a digital signature.",
    exp_pe_unsigned: " It is unsigned.",
    exp_vt_flagged: " Community reputation: {n}/{total} engines flagged it as malicious or suspicious.",
    exp_vt_no_record: " The hash was not reported to VirusTotal.",
    exp_no_findings: "No heuristic rules fired. There is no concrete evidence of suspicious behavior; the file is treated as clean unless new indicators appear.",
    exp_cat_header: "{heading}: {n} indicator(s).",
    exp_evidence: " ({ev})",
    cat_display: EN_CAT_DISPLAY,
    guidance: EN_GUIDANCE,
    guid_no_actions: &[
        "No suspicious indicators were found; no action is required.",
        "Re-scan the file if it changes or if it later behaves unexpectedly.",
    ],
    guid_no_impact: "No high-impact indicators were found; review the listed items before executing the file.",
    markers: &[" the ", " with ", " of ", " to ", " in ", " for ", " that ", " and ", " can ", " is ", " are "],
    counter_markers: &[" el ", " la ", " en ", " de ", " se ", " que ", " para ", " con "],
};

const ES_CAT_DISPLAY: &[(&str, &str)] = &[
    ("process", "Comportamiento de procesos"),
    ("persistence", "Persistencia"),
    ("powershell", "Uso de PowerShell"),
    ("packing", "Empaquetado / ofuscación"),
    ("network", "Comportamiento de red"),
    ("signatures", "Firmas conocidas"),
    ("general", "General"),
];

const ES_GUIDANCE: &[(&str, CatGuidance)] = &[
    (
        "process",
        CatGuidance {
            impact: "Se podría ejecutar código en el contexto de otro proceso, evadiendo los controles de seguridad.",
            consequences: "Datos sensibles (credenciales, documentos) podrían ser leídos o robados por el código inyectado.",
            actions: "Trate el archivo como hostil, aíslelo en cuarentena e investigue qué proceso lo creó.",
            vectors: &["Inyección de procesos y ejecución remota de código"],
        },
    ),
    (
        "persistence",
        CatGuidance {
            impact: "El archivo podría relanzarse automáticamente al iniciar el sistema o la sesión, sobreviviendo a los reinicios.",
            consequences: "Reinfección reiterada si no se eliminan las entradas de persistencia (registro, servicios, tareas programadas).",
            actions: "Elimine el archivo y quite sus entradas de inicio antes de reiniciar el equipo.",
            vectors: &["Persistencia mediante inicio automático, servicio o tarea programada"],
        },
    ),
    (
        "powershell",
        CatGuidance {
            impact: "Se podrían ejecutar cargas útiles en memoria mediante PowerShell sin escribir en disco.",
            consequences: "La ejecución sin archivos podría descargar y ejecutar etapas adicionales de forma evasiva.",
            actions: "Habilite y revise el registro de bloques de script de PowerShell y ejecute el script solo en un entorno aislado.",
            vectors: &["Ejecución basada en scripts (PowerShell)"],
        },
    ),
    (
        "packing",
        CatGuidance {
            impact: "El código real está oculto tras compresión o cifrado, lo que limita el análisis estático.",
            consequences: "Se requiere un análisis dinámico (sandbox) para observar el comportamiento tras desempaquetar.",
            actions: "Ejecute el archivo en una máquina virtual aislada con monitoreo de red y de procesos.",
            vectors: &["Carga útil empaquetada u ofuscada"],
        },
    ),
    (
        "network",
        CatGuidance {
            impact: "El archivo podría contactar con un host remoto para extraer datos o recibir comandos (C2).",
            consequences: "Los datos que salen de la red aumentan el riesgo de ataques posteriores.",
            actions: "Bloquee las conexiones salientes del host afectado y revise los registros de DNS, firewall y proxy.",
            vectors: &["Comando y control por HTTP", "Exfiltración de datos por sockets"],
        },
    ),
    (
        "signatures",
        CatGuidance {
            impact: "El archivo coincide con una firma conocida maliciosa o de prueba (por ejemplo, EICAR).",
            consequences: "Es un indicador confirmado de un archivo conocido como malicioso.",
            actions: "Aísle o elimine el archivo de inmediato e informe su hash a su equipo de seguridad.",
            vectors: &["Archivo conocido como malicioso"],
        },
    ),
    (
        "general",
        CatGuidance {
            impact: "Un archivo que no coincide con su extensión puede engañar a los usuarios para que lo abran.",
            consequences: "Los usuarios podrían ejecutar un archivo que creen inerte o seguro.",
            actions: "Verifique el tipo real del archivo antes de abrirlo; no ejecute contenido inesperado.",
            vectors: &["Confusión de tipo / ingeniería social"],
        },
    ),
];

const ES: Lang = Lang {
    no_evidence: "sin evidencia",
    vt_indicator_flagged: "[reputation] VirusTotal marcó {n}/{total} motores ({names}).",
    vt_indicator_clean: "[reputation] VirusTotal reporta {n}/{total} motores limpios.",
    sum_clean: "\"{file}\" no mostró indicadores sospechosos en su estructura ni en su contenido.",
    sum_likely_clean: "\"{file}\" solo muestra señales débiles o benignas ({n} hallazgo(s) de baja severidad).",
    sum_suspicious: "\"{file}\" combina {n} indicador(es) sospechoso(s) en {cats_count} categoría(s): {cats}.",
    sum_malicious: "\"{file}\" presenta indicadores claros de malware ({tops}) con una puntuación de amenaza de {score}/100.",
    vt_note_flagged: " VirusTotal: {n}/{total} motores lo marcaron como malicioso o sospechoso.",
    vt_note_clean: " VirusTotal reportó el hash como limpio ({n}/{total} motores sin detección).",
    vt_note_no_record: " VirusTotal no tenía registro de ese hash.",
    exp_baseline: "\"{file}\": el análisis estático inspeccionó el tipo de archivo, la entropía, la estructura, muestras del contenido y (si está configurado) la reputación comunitaria.",
    exp_file_type: " El archivo se identificó como {type} (entropía {entropy} bits/byte).",
    exp_pe_kind: " Es un PE {arch}{kind} con {sections} secciones y {imports} función(es) importada(s).",
    exp_pe_dll: " DLL",
    exp_pe_exec: " ejecutable",
    exp_pe_signed: " Está firmado digitalmente.",
    exp_pe_unsigned: " No está firmado.",
    exp_vt_flagged: " Reputación comunitaria: {n}/{total} motores lo marcaron como malicioso o sospechoso.",
    exp_vt_no_record: " El hash no fue reportado a VirusTotal.",
    exp_no_findings: "No se activó ninguna regla heurística. No hay evidencia concreta de comportamiento sospechoso; el archivo se trata como limpio salvo que aparezcan nuevos indicadores.",
    exp_cat_header: "{heading}: {n} indicador(es).",
    exp_evidence: " ({ev})",
    cat_display: ES_CAT_DISPLAY,
    guidance: ES_GUIDANCE,
    guid_no_actions: &[
        "No se encontraron indicadores sospechosos; no se requiere ninguna acción.",
        "Vuelva a analizar el archivo si cambia o si luego se comporta de forma inesperada.",
    ],
    guid_no_impact: "No se encontraron indicadores de alto impacto; revise los elementos listados antes de ejecutar el archivo.",
    markers: &[" el ", " la ", " en ", " de ", " se ", " que ", " para ", " con ", " los ", " las "],
    counter_markers: &[" the ", " with ", " of ", " to ", " in ", " for ", " that ", " and ", " can "],
};

fn cat_for(language: Language) -> &'static Lang {
    if language == Language::Es {
        &ES
    } else {
        &EN
    }
}

/// Construye la evaluación a partir de la evidencia del escaneo, compuesta
/// directamente en el idioma recibido.
pub fn build(
    file_name: &str,
    threat_level: ThreatLevel,
    threat_score: u32,
    findings: &[Finding],
    static_analysis: Option<&StaticAnalysis>,
    reputation: Option<&VirusTotalResult>,
    language: Language,
) -> AiAssessment {
    let cat = cat_for(language);
    let verdict = verdict(threat_level, reputation);
    let confidence = confidence(threat_level, reputation);

    let mut indicators: Vec<String> = findings
        .iter()
        .map(|f| {
            let ev = if f.evidence.is_empty() {
                cat.no_evidence.to_string()
            } else {
                f.evidence.join(", ")
            };
            format!("[{}] {}: {}", f.category, f.rule_name, ev)
        })
        .collect();

    // Si no hay hallazgos pero hay reputación, reflejar la evidencia externa.
    if indicators.is_empty() {
        if let Some(vt) = reputation {
            if vt.available && vt.malicious > 0 {
                indicators.push(fill(
                    cat.vt_indicator_flagged,
                    &[
                        ("{n}", &vt.malicious.to_string()),
                        ("{total}", &vt.total.to_string()),
                        ("{names}", &vt.threat_names.join(", ")),
                    ],
                ));
            } else if vt.available && vt.malicious == 0 {
                indicators.push(fill(
                    cat.vt_indicator_clean,
                    &[
                        ("{n}", &vt.harmless.to_string()),
                        ("{total}", &vt.total.to_string()),
                    ],
                ));
            }
        }
    }

    let (potential_impact, system_consequences, recommended_actions, attack_vectors) =
        guidance(cat, findings);
    let key_categories = key_categories(findings);
    let summary = summary(cat, file_name, verdict, threat_score, findings, reputation);
    let explanation = explanation(
        cat,
        language,
        file_name,
        static_analysis,
        reputation,
        findings,
    );

    let assessment = AiAssessment {
        verdict: verdict.to_string(),
        confidence,
        summary,
        explanation,
        indicators,
        potential_impact,
        system_consequences,
        recommended_actions,
        attack_vectors,
        key_categories,
    };

    validate_output(language, &assessment);
    assessment
}

/// Validación ligera del idioma de salida: compara la frecuencia de
/// marcadores comunes del idioma objetivo frente a los del idioma contrario
/// en los campos de prosa (resumen, explicación, acciones e impacto). Los
/// `indicators` se excluyen a propósito: combinan identificadores técnicos
/// (categorías, nombres de reglas, evidencia) que se conservan originales.
/// Ante cualquier desviación se registra en el log (no rompe el análisis; el
/// motor es determinista y la desviación sería un fallo interno).
fn validate_output(language: Language, a: &AiAssessment) {
    let (markers, wrong): (&[&str], &[&str]) = {
        let cat = cat_for(language);
        (cat.markers, cat.counter_markers)
    };
    let fields: [&str; 4] = [
        &a.summary,
        &a.explanation,
        &a.recommended_actions.join("\n"),
        &a.potential_impact.join("\n"),
    ];
    for f in fields {
        if f.trim().is_empty() {
            continue;
        }
        let lower = f.to_ascii_lowercase();
        let hits = markers.iter().filter(|m| lower.contains(**m)).count();
        let wrong_hits = wrong.iter().filter(|m| lower.contains(**m)).count();
        if wrong_hits >= 2 && wrong_hits >= hits {
            eprintln!(
                "AI LANGUAGE VALIDATION FAILED: lang={} markers={hits} wrong={wrong_hits}",
                language.as_str()
            );
        }
    }
}

/// Veredicto: nivel de amenaza + refuerzo por reputación externa.
fn verdict(threat_level: ThreatLevel, reputation: Option<&VirusTotalResult>) -> &'static str {
    let base = match threat_level {
        ThreatLevel::Clean => VERDICT_CLEAN,
        ThreatLevel::Low => VERDICT_LIKELY_CLEAN,
        ThreatLevel::Medium => VERDICT_SUSPICIOUS,
        ThreatLevel::High | ThreatLevel::Critical => VERDICT_MALICIOUS,
    };
    // Un archivo con score bajo pero flaggeado por varios motores externos
    // asciende a malicioso (evidencia externa fuerte).
    let vt_malicious = reputation.map(|r| r.malicious).unwrap_or(0);
    if base != VERDICT_MALICIOUS && vt_malicious >= 2 {
        VERDICT_MALICIOUS
    } else {
        base
    }
}

/// Confianza derivada de la fuerza de la evidencia (0.0 a 1.0).
fn confidence(threat_level: ThreatLevel, reputation: Option<&VirusTotalResult>) -> f32 {
    let mut c: f32 = match threat_level {
        ThreatLevel::Clean => 0.72,
        ThreatLevel::Low => 0.66,
        ThreatLevel::Medium => 0.62,
        ThreatLevel::High => 0.82,
        ThreatLevel::Critical => 0.92,
    };
    if let Some(vt) = reputation {
        if vt.available {
            // Acuerdo entre motor local y reputación externa refuerza la confianza.
            if (vt.malicious > 0 && threat_level != ThreatLevel::Clean)
                || (vt.malicious == 0 && threat_level == ThreatLevel::Clean)
            {
                c += 0.08;
            } else if vt.malicious > 0 {
                c += 0.04;
            } else {
                c -= 0.06;
            }
        } else {
            // Hash no reportado: hay menos evidencia.
            c -= 0.05;
        }
    }
    c.clamp(0.0, 1.0)
}

fn summary(
    cat: &'static Lang,
    file_name: &str,
    verdict: &str,
    threat_score: u32,
    findings: &[Finding],
    reputation: Option<&VirusTotalResult>,
) -> String {
    let vt_note = match reputation {
        Some(vt) if vt.available => {
            if vt.malicious + vt.suspicious > 0 {
                fill(
                    cat.vt_note_flagged,
                    &[
                        ("{n}", &(vt.malicious + vt.suspicious).to_string()),
                        ("{total}", &vt.total.to_string()),
                    ],
                )
            } else {
                fill(
                    cat.vt_note_clean,
                    &[
                        ("{n}", &vt.undetected.to_string()),
                        ("{total}", &vt.total.to_string()),
                    ],
                )
            }
        }
        Some(_) => cat.vt_note_no_record.to_string(),
        None => String::new(),
    };
    match verdict {
        VERDICT_CLEAN => fill(cat.sum_clean, &[("{file}", file_name)]) + &vt_note,
        VERDICT_LIKELY_CLEAN => {
            fill(
                cat.sum_likely_clean,
                &[("{file}", file_name), ("{n}", &findings.len().to_string())],
            ) + &vt_note
        }
        VERDICT_SUSPICIOUS => {
            let cats = key_categories(findings);
            fill(
                cat.sum_suspicious,
                &[
                    ("{file}", file_name),
                    ("{n}", &findings.len().to_string()),
                    ("{cats_count}", &cats.len().to_string()),
                    ("{cats}", &cats.join(", ")),
                ],
            ) + &vt_note
        }
        _ => {
            let tops: Vec<String> = findings
                .iter()
                .take(3)
                .map(|f| f.rule_name.clone())
                .collect();
            fill(
                cat.sum_malicious,
                &[
                    ("{file}", file_name),
                    ("{tops}", &tops.join(", ")),
                    ("{score}", &threat_score.to_string()),
                ],
            ) + &vt_note
        }
    }
}

/// Línea base + un párrafo por categoría con hallazgos.
fn explanation(
    cat: &'static Lang,
    language: Language,
    file_name: &str,
    static_analysis: Option<&StaticAnalysis>,
    reputation: Option<&VirusTotalResult>,
    findings: &[Finding],
) -> String {
    let mut parts: Vec<String> = Vec::new();

    let mut baseline = fill(cat.exp_baseline, &[("{file}", file_name)]);
    if let Some(a) = static_analysis {
        baseline.push_str(&fill(
            cat.exp_file_type,
            &[
                ("{type}", &a.file_type),
                ("{entropy}", &format!("{:.2}", a.entropy)),
            ],
        ));
        if let Some(pe) = &a.pe {
            let kind = if pe.is_dll {
                cat.exp_pe_dll
            } else {
                cat.exp_pe_exec
            };
            baseline.push_str(&fill(
                cat.exp_pe_kind,
                &[
                    ("{arch}", &pe.architecture),
                    ("{kind}", kind),
                    ("{sections}", &pe.sections.len().to_string()),
                    ("{imports}", &pe.import_count.to_string()),
                ],
            ));
            baseline.push_str(if pe.has_certificate {
                cat.exp_pe_signed
            } else {
                cat.exp_pe_unsigned
            });
        }
    }
    match reputation {
        Some(vt) if vt.available => baseline.push_str(&fill(
            cat.exp_vt_flagged,
            &[
                ("{n}", &(vt.malicious + vt.suspicious).to_string()),
                ("{total}", &vt.total.to_string()),
            ],
        )),
        Some(_) => baseline.push_str(cat.exp_vt_no_record),
        None => {}
    }
    parts.push(baseline);

    if findings.is_empty() {
        parts.push(cat.exp_no_findings.to_string());
        return parts.join("\n\n");
    }

    // Agrupar por categoría manteniendo el orden del catálogo.
    let order = [
        "process",
        "persistence",
        "powershell",
        "packing",
        "network",
        "signatures",
        "general",
    ];
    for cat_key in order {
        let cat_findings: Vec<&Finding> =
            findings.iter().filter(|f| f.category == cat_key).collect();
        if cat_findings.is_empty() {
            continue;
        }
        let heading = category_display(cat, cat_key);
        let mut para = fill(
            cat.exp_cat_header,
            &[
                ("{heading}", heading),
                ("{n}", &cat_findings.len().to_string()),
            ],
        );
        for f in cat_findings {
            let ev = if f.evidence.is_empty() {
                String::new()
            } else {
                fill(cat.exp_evidence, &[("{ev}", &f.evidence.join(", "))])
            };
            para.push_str(&format!(
                "\n- {}: {}{}",
                f.rule_name,
                crate::rules::description_in(&f.rule_name, language),
                ev
            ));
        }
        parts.push(para);
    }

    parts.join("\n\n")
}

/// Orientación (impacto, consecuencias, acciones, vectores) basada en las
/// categorías que realmente dispararon hallazgos. Texto genérico de seguridad,
/// no afirmaciones inventadas sobre el archivo.
fn guidance(
    cat: &'static Lang,
    findings: &[Finding],
) -> (Vec<String>, Vec<String>, Vec<String>, Vec<String>) {
    let mut cats: Vec<&str> = findings.iter().map(|f| f.category.as_str()).collect();
    cats.sort();
    cats.dedup();

    if cats.is_empty() {
        return (
            Vec::new(),
            Vec::new(),
            cat.guid_no_actions.iter().map(|s| s.to_string()).collect(),
            Vec::new(),
        );
    }

    let mut impact: Vec<String> = Vec::new();
    let mut consequences: Vec<String> = Vec::new();
    let mut actions: Vec<String> = Vec::new();
    let mut vectors: Vec<String> = Vec::new();

    for cat_key in &cats {
        if let Some(g) = cat
            .guidance
            .iter()
            .find(|(k, _)| *k == *cat_key)
            .map(|(_, g)| g)
        {
            impact.push(g.impact.to_string());
            consequences.push(g.consequences.to_string());
            actions.push(g.actions.to_string());
            vectors.extend(g.vectors.iter().map(|s| s.to_string()));
        }
    }

    // Dedup conservando el orden.
    let dedup = |v: Vec<String>| -> Vec<String> {
        let mut out: Vec<String> = Vec::new();
        for s in v {
            if !out.contains(&s) {
                out.push(s);
            }
        }
        out
    };

    if impact.is_empty() {
        impact.push(cat.guid_no_impact.to_string());
    }
    (
        dedup(impact),
        dedup(consequences),
        dedup(actions),
        dedup(vectors),
    )
}

/// Categorías con más peso (por puntos), top 3, como claves `rules.category.*`.
fn key_categories(findings: &[Finding]) -> Vec<String> {
    use std::collections::HashMap;
    let mut by_cat: HashMap<String, u32> = HashMap::new();
    for f in findings {
        *by_cat.entry(f.category.clone()).or_default() += f.points;
    }
    let mut list: Vec<(String, u32)> = by_cat.into_iter().collect();
    list.sort_by(|a, b| b.1.cmp(&a.1));
    list.into_iter().take(3).map(|(c, _)| c).collect()
}

fn category_display(cat: &'static Lang, key: &str) -> &'static str {
    cat.cat_display
        .iter()
        .find(|(k, _)| *k == key)
        .map(|(_, d)| *d)
        .unwrap_or("General")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::Severity;

    fn finding(category: &str, points: u32, severity: Severity, name: &str) -> Finding {
        Finding {
            rule_name: name.to_string(),
            category: category.to_string(),
            severity,
            description: format!("{name} description"),
            evidence: vec!["WriteProcessMemory".into()],
            points,
        }
    }

    #[test]
    fn clean_file_yields_clean_verdict() {
        let a = build(
            "test.txt",
            ThreatLevel::Clean,
            0,
            &[],
            None,
            None,
            Language::En,
        );
        assert_eq!(a.verdict, VERDICT_CLEAN);
        assert!(a.indicators.is_empty());
        assert!(a.summary.contains("no suspicious indicators"));
        assert!(a.recommended_actions.len() >= 1);
    }

    #[test]
    fn strong_findings_yield_malicious_verdict() {
        let findings = vec![
            finding(
                "process",
                25,
                Severity::Critical,
                "Process injection imports",
            ),
            finding("persistence", 6, Severity::Medium, "Registry persistence"),
        ];
        let a = build(
            "evil.exe",
            ThreatLevel::High,
            40,
            &findings,
            None,
            None,
            Language::En,
        );
        assert_eq!(a.verdict, VERDICT_MALICIOUS);
        assert!(a.confidence >= 0.8);
        assert!(!a.indicators.is_empty());
        assert!(a
            .potential_impact
            .iter()
            .any(|s| s.contains("another process")));
        assert!(a
            .recommended_actions
            .iter()
            .any(|s| s.contains("quarantine")));
        assert!(a.key_categories.first() == Some(&"process".to_string()));
        assert!(a.explanation.contains("Process behavior"));
    }

    #[test]
    fn vt_agreement_boosts_confidence() {
        let vt = VirusTotalResult {
            available: true,
            malicious: 3,
            total: 60,
            ..Default::default()
        };
        let findings = vec![finding(
            "process",
            25,
            Severity::Critical,
            "Process injection imports",
        )];
        let with = build(
            "a.exe",
            ThreatLevel::High,
            40,
            &findings,
            None,
            Some(&vt),
            Language::En,
        );
        let without = build(
            "a.exe",
            ThreatLevel::High,
            40,
            &findings,
            None,
            None,
            Language::En,
        );
        assert!(with.confidence > without.confidence);
        assert!(with.indicators.len() == 1, "solo los hallazgos locales");
    }

    #[test]
    fn low_score_escalates_on_vt_flags() {
        let vt = VirusTotalResult {
            available: true,
            malicious: 5,
            total: 60,
            ..Default::default()
        };
        let a = build(
            "x.bin",
            ThreatLevel::Low,
            10,
            &[],
            None,
            Some(&vt),
            Language::En,
        );
        assert_eq!(a.verdict, VERDICT_MALICIOUS);
        assert!(a.indicators.iter().any(|s| s.contains("flagged")));
    }

    #[test]
    fn es_output_is_spanish() {
        let findings = vec![
            finding(
                "process",
                25,
                Severity::Critical,
                "Process injection imports",
            ),
            finding("persistence", 6, Severity::Medium, "Registry persistence"),
        ];
        let a = build(
            "evil.exe",
            ThreatLevel::High,
            40,
            &findings,
            None,
            None,
            Language::Es,
        );
        assert_eq!(a.verdict, VERDICT_MALICIOUS);
        assert!(
            a.summary.contains("presenta indicadores"),
            "resumen en español"
        );
        assert!(
            a.explanation.contains("análisis estático"),
            "explicación en español"
        );
        assert!(
            a.explanation.contains("Comportamiento de procesos"),
            "título de categoría en español"
        );
        assert!(
            a.explanation.contains("inyección de procesos"),
            "descripción de regla en español"
        );
        assert!(a
            .potential_impact
            .iter()
            .any(|s| s.contains("otro proceso")));
        assert!(a
            .recommended_actions
            .iter()
            .any(|s| s.contains("cuarentena")));
        assert!(
            !a.summary.to_lowercase().contains("the file"),
            "sin texto en inglés"
        );
        assert!(
            !a.explanation.to_lowercase().contains("static analysis"),
            "sin explicación en inglés"
        );
    }

    #[test]
    fn en_output_is_english() {
        let findings = vec![finding(
            "process",
            25,
            Severity::Critical,
            "Process injection imports",
        )];
        let a = build(
            "evil.exe",
            ThreatLevel::High,
            40,
            &findings,
            None,
            None,
            Language::En,
        );
        assert!(a.summary.contains("exhibits strong malware indicators"));
        assert!(a.explanation.contains("static analysis"));
        assert!(!a.explanation.contains("análisis estático"));
    }

    #[test]
    fn es_clean_output_is_spanish() {
        let a = build(
            "test.txt",
            ThreatLevel::Clean,
            0,
            &[],
            None,
            None,
            Language::Es,
        );
        assert_eq!(a.verdict, VERDICT_CLEAN);
        assert!(
            a.summary.contains("no mostró indicadores"),
            "resumen limpio en español"
        );
        assert!(a.explanation.contains("análisis estático"));
        assert!(a
            .recommended_actions
            .iter()
            .any(|s| s.contains("no se requiere ninguna acción")));
    }

    #[test]
    fn language_from_config_falls_back_to_en() {
        assert_eq!(Language::from_config("es"), Language::Es);
        assert_eq!(Language::from_config("en"), Language::En);
        assert_eq!(Language::from_config("fr"), Language::En);
        assert_eq!(Language::from_config(""), Language::En);
        assert_eq!(Language::from_config("  ES "), Language::Es);
    }
}
