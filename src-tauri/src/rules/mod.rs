//! Motor de reglas heurísticas (FASE 4).
//!
//! Cada regla del catálogo evalúa la evidencia de `StaticAnalysis` (y los
//! hashes) y devuelve `Finding`s concretos. Los hallazgos aportan puntos al
//! `threat_score`; la presencia de indicadores NO es un veredicto automático.
//!
//! Seguridad: el motor solo analiza datos ya extraídos; nunca ejecuta archivos.

use crate::models::{
    FileHashes, Finding, Language, PeInfo, RuleInfo, Severity, StaticAnalysis, ThreatLevel,
};

/// Tope del `threat_score` (0..100).
pub const MAX_POINTS: u32 = 100;

/// Categorías del catálogo (coinciden con las claves i18n `rules.category.*`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Category {
    Process,
    Persistence,
    PowerShell,
    Packing,
    Network,
    Signatures,
    General,
}

impl Category {
    pub fn as_str(self) -> &'static str {
        match self {
            Category::Process => "process",
            Category::Persistence => "persistence",
            Category::PowerShell => "powershell",
            Category::Packing => "packing",
            Category::Network => "network",
            Category::Signatures => "signatures",
            Category::General => "general",
        }
    }
}

/// Definición estática de una regla.
struct Rule {
    id: &'static str,
    category: Category,
    severity: Severity,
    points: u32,
    name: &'static str,
    description: &'static str,
}

/// Catálogo de reglas (orden de evaluación fijo).
const RULES: &[Rule] = &[
    // Packing
    Rule {
        id: "upx-packed",
        category: Category::Packing,
        severity: Severity::High,
        points: 12,
        name: "UPX packed sections",
        description: "UPX0/UPX1 sections indicate UPX compression, common in both packed malware and legitimate installers.",
    },
    Rule {
        id: "packed-known",
        category: Category::Packing,
        severity: Severity::High,
        points: 12,
        name: "Known packer sections",
        description: "Section names match a known packer or protector (ASPack, MPRESS, Themida, VMProtect, …).",
    },
    Rule {
        id: "entropy-text",
        category: Category::Packing,
        severity: Severity::Medium,
        points: 8,
        name: "High .text section entropy",
        description: "The code section has unusually high entropy, typical of packed, encrypted or compressed code.",
    },
    Rule {
        id: "writable-exec",
        category: Category::Packing,
        severity: Severity::High,
        points: 12,
        name: "Writable and executable section",
        description: "A section has both WRITE and EXECUTE rights, the classic staging pattern for injected code.",
    },
    Rule {
        id: "many-sections",
        category: Category::Packing,
        severity: Severity::Low,
        points: 3,
        name: "Unusually many sections",
        description: "More than 40 sections is unusual for a standard PE and frequent in packed binaries.",
    },
    Rule {
        id: "entropy-script",
        category: Category::Packing,
        severity: Severity::Medium,
        points: 6,
        name: "High entropy non-PE file",
        description: "A non-executable file with very high entropy: likely encrypted, compressed or obfuscated content.",
    },
    // Process
    Rule {
        id: "process-injection",
        category: Category::Process,
        severity: Severity::Critical,
        points: 25,
        name: "Process injection imports",
        description: "Imports a combination of APIs commonly used for process injection (WriteProcessMemory, CreateRemoteThread, …).",
    },
    Rule {
        id: "rwx-allocation",
        category: Category::Process,
        severity: Severity::Medium,
        points: 6,
        name: "RWX allocation and thread creation",
        description: "Combines VirtualAlloc, VirtualProtect and CreateThread: the typical shellcode execution setup.",
    },
    Rule {
        id: "process-enumeration",
        category: Category::Process,
        severity: Severity::Low,
        points: 3,
        name: "Process enumeration",
        description: "APIs that enumerate running processes, often used for evasion or target discovery.",
    },
    Rule {
        id: "keylogging",
        category: Category::Process,
        severity: Severity::Medium,
        points: 6,
        name: "Keylogging APIs",
        description: "Imports hooks or keyboard-state APIs associated with keystroke capture.",
    },
    Rule {
        id: "anti-debug",
        category: Category::Process,
        severity: Severity::Low,
        points: 4,
        name: "Anti-debugging APIs",
        description: "Imports APIs used to detect debuggers or analysis environments.",
    },
    // Persistence
    Rule {
        id: "persistence-registry",
        category: Category::Persistence,
        severity: Severity::Medium,
        points: 6,
        name: "Registry persistence",
        description: "Registry write APIs (or 'reg add') that can establish persistence through auto-start keys.",
    },
    Rule {
        id: "persistence-service",
        category: Category::Persistence,
        severity: Severity::Medium,
        points: 6,
        name: "Service installation",
        description: "Windows service management APIs used to register a persistent service.",
    },
    Rule {
        id: "persistence-scheduled",
        category: Category::Persistence,
        severity: Severity::Medium,
        points: 6,
        name: "Scheduled task references",
        description: "References schtasks or scheduled-task execution, a common persistence or delivery mechanism.",
    },
    // PowerShell
    Rule {
        id: "powershell-invocation",
        category: Category::PowerShell,
        severity: Severity::Medium,
        points: 6,
        name: "PowerShell references",
        description: "The file references PowerShell, a frequent execution vehicle for malicious payloads.",
    },
    Rule {
        id: "powershell-download",
        category: Category::PowerShell,
        severity: Severity::High,
        points: 12,
        name: "PowerShell download cradle",
        description: "PowerShell combined with download/execute primitives (DownloadString, IEX, Invoke-WebRequest, …).",
    },
    Rule {
        id: "powershell-encoded",
        category: Category::PowerShell,
        severity: Severity::High,
        points: 12,
        name: "Encoded PowerShell command",
        description: "EncodedCommand is present; obfuscated one-liners are a staple of malware delivery.",
    },
    // Network
    Rule {
        id: "network-downloader",
        category: Category::Network,
        severity: Severity::Medium,
        points: 8,
        name: "File downloader APIs",
        description: "APIs that download files over HTTP(S), often used to fetch subsequent stages.",
    },
    Rule {
        id: "network-winhttp",
        category: Category::Network,
        severity: Severity::Medium,
        points: 6,
        name: "WinHTTP client",
        description: "WinHTTP APIs used to communicate with remote hosts over HTTP.",
    },
    Rule {
        id: "network-wininet",
        category: Category::Network,
        severity: Severity::Low,
        points: 4,
        name: "WinINet client",
        description: "WinINet APIs used for internet communication.",
    },
    Rule {
        id: "network-socket",
        category: Category::Network,
        severity: Severity::Low,
        points: 3,
        name: "Socket (Winsock) usage",
        description: "Imports ws2_32: direct socket communication, typical of C2 beacons.",
    },
    Rule {
        id: "network-dns",
        category: Category::Network,
        severity: Severity::Low,
        points: 2,
        name: "DNS resolution APIs",
        description: "APIs for hostname resolution, used to reach command-and-control domains.",
    },
    // Signatures
    Rule {
        id: "eicar-detected",
        category: Category::Signatures,
        severity: Severity::High,
        points: 20,
        name: "EICAR test string",
        description: "The EICAR anti-malware test string was found. Used to validate detection.",
    },
    Rule {
        id: "known-hash",
        category: Category::Signatures,
        severity: Severity::Critical,
        points: 100,
        name: "Known malicious hash",
        description: "The file hash matches an entry in the built-in signature list.",
    },
    // General
    Rule {
        id: "unsigned-pe",
        category: Category::General,
        severity: Severity::Low,
        points: 2,
        name: "Unsigned PE",
        description: "The executable carries no digital signature; unsigned binaries are a weak signal on modern Windows.",
    },
    Rule {
        id: "type-mismatch-executable",
        category: Category::General,
        severity: Severity::High,
        points: 10,
        name: "Executable hidden as another type",
        description: "A PE file whose extension contradicts its content, typical of staged malware.",
    },
    Rule {
        id: "type-mismatch",
        category: Category::General,
        severity: Severity::Medium,
        points: 6,
        name: "Content and extension mismatch",
        description: "The detected file type does not match the file extension.",
    },
    Rule {
        id: "script-keywords",
        category: Category::General,
        severity: Severity::Medium,
        points: 6,
        name: "Multiple suspicious strings",
        description: "Several suspicious keywords found in a non-executable file (obfuscation, download, execution).",
    },
];

fn rule(id: &str) -> &'static Rule {
    RULES
        .iter()
        .find(|r| r.id == id)
        .unwrap_or_else(|| panic!("regla desconocida: {id}"))
}

/// Descripciones en español de las reglas (clave: nombre de la regla).
///
/// La regla conserva su descripción en inglés como referencia técnica; el
/// motor de evaluación usa esta ficha para componer su informe directamente
/// en español.
const DESCRIPTIONS_ES: &[(&str, &str)] = &[
    ("UPX packed sections", "Las secciones UPX0/UPX1 indican compresión UPX, habitual tanto en malware empaquetado como en instaladores legítimos."),
    ("Known packer sections", "Los nombres de sección coinciden con un empaquetador o protector conocido (ASPack, MPRESS, Themida, VMProtect, …)."),
    ("High .text section entropy", "La sección de código tiene una entropía inusualmente alta, típica de código empaquetado, cifrado o comprimido."),
    ("Writable and executable section", "Una sección tiene permisos de escritura y ejecución a la vez, el patrón clásico para alojar código inyectado."),
    ("Unusually many sections", "Más de 40 secciones es inusual en un PE estándar y frecuente en binarios empaquetados."),
    ("High entropy non-PE file", "Un archivo no ejecutable con entropía muy alta: probablemente contenido cifrado, comprimido u ofuscado."),
    ("Process injection imports", "Importa una combinación de APIs habitualmente usadas para inyección de procesos (WriteProcessMemory, CreateRemoteThread, …)."),
    ("RWX allocation and thread creation", "Combina VirtualAlloc, VirtualProtect y CreateThread: la configuración típica para ejecutar shellcode."),
    ("Process enumeration", "APIs que enumeran procesos en ejecución, a menudo usadas para evasión o reconocimiento de objetivos."),
    ("Keylogging APIs", "Importa hooks o APIs de estado de teclado asociadas a la captura de pulsaciones."),
    ("Anti-debugging APIs", "Importa APIs usadas para detectar depuradores o entornos de análisis."),
    ("Registry persistence", "APIs de escritura en el registro (o 'reg add') que pueden establecer persistencia mediante claves de inicio automático."),
    ("Service installation", "APIs de gestión de servicios de Windows usadas para registrar un servicio persistente."),
    ("Scheduled task references", "Referencia schtasks o la ejecución de tareas programadas, un mecanismo habitual de persistencia o entrega."),
    ("PowerShell references", "El archivo hace referencia a PowerShell, un vehículo de ejecución frecuente para cargas maliciosas."),
    ("PowerShell download cradle", "PowerShell combinado con primitivas de descarga/ejecución (DownloadString, IEX, Invoke-WebRequest, …)."),
    ("Encoded PowerShell command", "Está presente EncodedCommand; las líneas ofuscadas son un recurso clásico en la entrega de malware."),
    ("File downloader APIs", "APIs que descargan archivos por HTTP(S), a menudo usadas para obtener etapas posteriores."),
    ("WinHTTP client", "APIs WinHTTP usadas para comunicarse con hosts remotos por HTTP."),
    ("WinINet client", "APIs WinINet usadas para comunicación por Internet."),
    ("Socket (Winsock) usage", "Importa ws2_32: comunicación directa por sockets, típica de balizas C2."),
    ("DNS resolution APIs", "APIs de resolución de nombres, usadas para alcanzar dominios de comando y control."),
    ("EICAR test string", "Se encontró la cadena de prueba antimalware EICAR. Se usa para validar la detección."),
    ("Known malicious hash", "El hash del archivo coincide con una entrada de la lista de firmas integrada."),
    ("Unsigned PE", "El ejecutable no lleva firma digital; los binarios sin firmar son una señal débil en Windows moderno."),
    ("Executable hidden as another type", "Un archivo PE cuya extensión contradice su contenido, típico de malware preparado para distribuir."),
    ("Content and extension mismatch", "El tipo de archivo detectado no coincide con la extensión del archivo."),
    ("Multiple suspicious strings", "Se encontraron varias palabras clave sospechosas en un archivo no ejecutable (ofuscación, descarga, ejecución)."),
];

/// Descripción localizada de una regla para el informe del motor de
/// evaluación. En `Es` se usa la ficha en español; el resto conserva la
/// descripción original en inglés.
pub fn description_in<'a>(rule_name: &'a str, lang: Language) -> &'a str {
    if lang == Language::Es {
        for (name, es) in DESCRIPTIONS_ES {
            if *name == rule_name {
                return es;
            }
        }
    }
    RULES
        .iter()
        .find(|r| r.name == rule_name)
        .map(|r| r.description)
        .unwrap_or(rule_name)
}

fn finding(id: &str, evidence: Vec<String>) -> Finding {
    let r = rule(id);
    Finding {
        rule_name: r.name.to_string(),
        category: r.category.as_str().to_string(),
        severity: r.severity,
        description: r.description.to_string(),
        evidence,
        points: r.points,
    }
}

/// Normaliza un nombre de API quitando el sufijo ANSI/Unicode (A/W) y
/// convirtiendo a minúsculas: `CreateRemoteThreadW` → `createremotethread`.
fn norm_api(name: &str) -> String {
    name.trim_end_matches(|c| c == 'A' || c == 'W')
        .to_ascii_lowercase()
}

/// Funciones importadas que coinciden con alguna aguja (prefijo seguro).
fn import_matches(pe: &PeInfo, needles: &[&str]) -> Vec<String> {
    let mut found: Vec<String> = Vec::new();
    for dll in &pe.imports {
        for f in &dll.functions {
            let norm = norm_api(f);
            if needles.iter().any(|n| norm == *n || norm.starts_with(n)) {
                found.push(f.clone());
            }
        }
    }
    // Excluye tokens benignos que comparten prefijo con APIs sensibles.
    found.retain(|f| {
        let norm = norm_api(f);
        norm != "openprocesstoken"
            && norm != "openthreadtoken"
            && !norm.starts_with("createthreadpool")
    });
    found.sort();
    found.dedup();
    found
}

fn has_any_keyword(keywords: &[String], needles: &[&str]) -> bool {
    keywords.iter().any(|k| needles.contains(&k.as_str()))
}

fn keyword_hits(keywords: &[String], needles: &[&str]) -> Vec<String> {
    keywords
        .iter()
        .filter(|k| needles.contains(&k.as_str()))
        .cloned()
        .collect()
}

/// Evalúa el análisis estático y devuelve los hallazgos del catálogo.
pub fn evaluate(analysis: Option<&StaticAnalysis>, hashes: Option<&FileHashes>) -> Vec<Finding> {
    let mut out: Vec<Finding> = Vec::new();
    let Some(a) = analysis else {
        return out;
    };

    let pe = a.pe.as_ref();
    let kw = &a.keywords;

    // --- Packing ---
    if let Some(pe) = pe {
        let names: Vec<String> = pe.sections.iter().map(|s| s.name.to_lowercase()).collect();

        if names
            .iter()
            .any(|n| n == "upx0" || n == "upx1" || n.starts_with("upx"))
        {
            out.push(finding("upx-packed", vec!["UPX0/UPX1 sections".into()]));
        }

        let packer_markers = [
            "aspack",
            "petite",
            "mpress",
            "themida",
            "vmprotect",
            "vmp",
            "nsp",
            "svp",
            "rpcrypt",
            "wpack",
            "sforce",
            "peshield",
            "enigma",
            "obsidium",
            "vprotect",
            "packed",
        ];
        let packer_hits: Vec<String> = names
            .iter()
            .filter(|n| packer_markers.iter().any(|m| n.contains(m)))
            .cloned()
            .collect();
        if !packer_hits.is_empty() {
            out.push(finding("packed-known", packer_hits));
        }

        let text_entropy = pe
            .sections
            .iter()
            .filter(|s| s.name == ".text" || s.name == "CODE")
            .map(|s| s.entropy)
            .fold(0.0f64, f64::max);
        if text_entropy >= 7.0 {
            out.push(finding(
                "entropy-text",
                vec![format!(".text entropy {text_entropy:.2}")],
            ));
        }

        let wx: Vec<String> = pe
            .sections
            .iter()
            .filter(|s| s.flags.iter().any(|f| f == "EXEC") && s.flags.iter().any(|f| f == "WRITE"))
            .map(|s| s.name.clone())
            .collect();
        if !wx.is_empty() {
            out.push(finding("writable-exec", wx));
        }

        if pe.sections.len() > 40 {
            out.push(finding(
                "many-sections",
                vec![format!("{} sections", pe.sections.len())],
            ));
        }
    }

    if !a.is_pe && a.entropy >= 7.0 {
        out.push(finding(
            "entropy-script",
            vec![format!("entropy {:.2}", a.entropy)],
        ));
    }

    // --- Process ---
    if let Some(pe) = pe {
        let injection_needles = [
            "openprocess",
            "virtualallocex",
            "virtualprotectex",
            "writeprocessmemory",
            "createremotethread",
            "ntcreatethreadex",
            "ntwritevirtualmemory",
            "ntmapviewofsection",
            "setthreadcontext",
            "ntsetcontextthread",
        ];
        let inj = import_matches(pe, &injection_needles);
        let strong_markers = [
            "writeprocessmemory",
            "createremotethread",
            "ntcreatethreadex",
            "ntwritevirtualmemory",
            "ntmapviewofsection",
            "virtualallocex",
        ];
        let has_strong = inj
            .iter()
            .any(|f| strong_markers.iter().any(|m| norm_api(f).starts_with(m)));
        if inj.len() >= 2 && has_strong {
            out.push(finding("process-injection", inj));
        }

        let rwx = [
            ("virtualalloc", "VirtualAlloc"),
            ("virtualprotect", "VirtualProtect"),
            ("createthread", "CreateThread"),
        ]
        .iter()
        .filter_map(|(needle, _)| {
            let hits = import_matches(pe, &[*needle]);
            if hits.is_empty() {
                None
            } else {
                hits.first().cloned()
            }
        })
        .collect::<Vec<String>>();
        if rwx.len() == 3 {
            out.push(finding("rwx-allocation", rwx));
        }

        let enum_hits = import_matches(
            pe,
            &[
                "toolhelp32snapshot",
                "process32first",
                "process32next",
                "ntquerysysteminformation",
            ],
        );
        if enum_hits.len() >= 2 {
            out.push(finding("process-enumeration", enum_hits));
        }

        let keylog = import_matches(pe, &["setwindowshookex", "getasynckeystate", "getkeystate"]);
        if !keylog.is_empty() {
            out.push(finding("keylogging", keylog));
        }

        let anti = import_matches(pe, &["isdebuggerpresent", "checkremotedebuggerpresent"]);
        if !anti.is_empty() {
            out.push(finding("anti-debug", anti));
        }
    }

    // --- Persistence ---
    if let Some(pe) = pe {
        let reg = import_matches(pe, &["regsetvalueex", "regcreatekeyex", "regopenkeyex"]);
        if reg.len() >= 2 {
            out.push(finding("persistence-registry", reg));
        }

        let svc = import_matches(pe, &["createservice", "openscmanager", "startservice"]);
        if svc.len() >= 2 {
            out.push(finding("persistence-service", svc));
        }
    }
    // Las reglas basadas en strings solo aplican a contenido no ejecutable:
    // en un PE las cadenas pueden pertenecer a datos embebidos y generan ruido.
    if !a.is_pe {
        if has_any_keyword(kw, &["reg add"]) {
            out.push(finding("persistence-registry", vec!["reg add".into()]));
        }
        if has_any_keyword(kw, &["schtasks"]) {
            out.push(finding("persistence-scheduled", vec!["schtasks".into()]));
        }
    }

    // --- PowerShell ---
    let has_ps = has_any_keyword(kw, &["powershell"]);
    let dl_kw = keyword_hits(
        kw,
        &[
            "downloadstring",
            "downloadfile",
            "invoke-webrequest",
            "invoke-expression",
        ],
    );
    if !a.is_pe && has_ps {
        out.push(finding("powershell-invocation", vec!["powershell".into()]));
    }
    if !a.is_pe && has_ps && !dl_kw.is_empty() {
        out.push(finding("powershell-download", dl_kw));
    }
    if !a.is_pe && has_any_keyword(kw, &["encodedcommand"]) {
        out.push(finding("powershell-encoded", vec!["EncodedCommand".into()]));
    }

    // --- Network ---
    if let Some(pe) = pe {
        let dl = import_matches(pe, &["urldownloadtofile", "urlopenblockingstream"]);
        if !dl.is_empty() {
            out.push(finding("network-downloader", dl));
        }

        let wh = import_matches(
            pe,
            &[
                "winhttpopen",
                "winhttpconnect",
                "winhttpsendrequest",
                "winhttpopenrequest",
            ],
        );
        if wh.len() >= 2 {
            out.push(finding("network-winhttp", wh));
        }

        let wi = import_matches(pe, &["internetopen", "internetconnect", "internetopenurl"]);
        if wi.len() >= 2 {
            out.push(finding("network-wininet", wi));
        }

        if pe
            .imports
            .iter()
            .any(|d| d.name.to_lowercase().contains("ws2_32"))
        {
            out.push(finding(
                "network-socket",
                vec!["ws2_32.dll imported".into()],
            ));
        }

        let dns = import_matches(
            pe,
            &["gethostbyname", "getaddrinfo", "dnsquery", "inet_addr"],
        );
        if !dns.is_empty() {
            out.push(finding("network-dns", dns));
        }
    }

    // --- Signatures ---
    if !a.is_pe && has_any_keyword(kw, &["eicar"]) {
        out.push(finding("eicar-detected", vec!["EICAR test string".into()]));
    }
    if let Some(hit) = match_known_hash(hashes) {
        out.push(finding("known-hash", vec![hit]));
    }

    // --- General ---
    if a.is_pe && !a.pe.as_ref().is_some_and(|p| p.has_certificate) {
        out.push(finding("unsigned-pe", Vec::new()));
    }
    if a.type_mismatch {
        let evidence = vec![format!(
            "content {} ({}) vs extension",
            a.file_type, a.file_type_extension
        )];
        if a.is_pe {
            out.push(finding("type-mismatch-executable", evidence));
        } else {
            out.push(finding("type-mismatch", evidence));
        }
    }
    if !a.is_pe && kw.len() >= 3 {
        out.push(finding("script-keywords", kw.clone()));
    }

    // Orden estable: más crítico primero, luego más puntos.
    out.sort_by_key(|f| (severity_rank(f.severity), f.points));
    out.reverse();
    out
}

fn severity_rank(s: Severity) -> u8 {
    match s {
        Severity::Critical => 5,
        Severity::High => 4,
        Severity::Medium => 3,
        Severity::Low => 2,
        Severity::Info => 1,
    }
}

/// Suma ponderada de los hallazgos, con tope en `MAX_POINTS`.
pub fn score(findings: &[Finding]) -> u32 {
    findings
        .iter()
        .map(|f| f.points)
        .sum::<u32>()
        .min(MAX_POINTS)
}

/// Niveles: Clean(0) · Low(1–14) · Medium(15–34) · High(35–64) · Critical(65+).
pub fn level_from_score(score: u32) -> ThreatLevel {
    match score {
        0 => ThreatLevel::Clean,
        1..=14 => ThreatLevel::Low,
        15..=34 => ThreatLevel::Medium,
        35..=64 => ThreatLevel::High,
        _ => ThreatLevel::Critical,
    }
}

/// Fichas descriptivas del catálogo para la página de Reglas.
pub fn catalog_info() -> Vec<RuleInfo> {
    RULES
        .iter()
        .map(|r| RuleInfo {
            id: r.id.to_string(),
            category: r.category.as_str().to_string(),
            name: r.name.to_string(),
            description: r.description.to_string(),
            severity: r.severity,
            points: r.points,
        })
        .collect()
}

/// Lista de firmas hash conocidas (SHA-256 en hexadecimal).
///
/// Vacía por defecto; extender con hashes reales de muestras conocidas.
const KNOWN_HASHES: &[&str] = &[
    // "1f26639f3e9c9d8c7f2e3b2c7d3d1c9c3e4f5a6b7c8d9e0f1a2b3c4d5e6f7a8b9",
];

/// Busca los hashes del archivo en la lista de firmas conocidas.
fn match_known_hash(hashes: Option<&FileHashes>) -> Option<String> {
    let h = hashes?;
    for want in KNOWN_HASHES {
        if h.sha256.as_deref() == Some(*want)
            || h.sha1.as_deref() == Some(*want)
            || h.md5.as_deref() == Some(*want)
        {
            return Some(format!("{want}"));
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{PeImportDll, PeInfo, PeSection, StaticAnalysis};

    fn sample_analysis() -> StaticAnalysis {
        StaticAnalysis {
            file_type: "PE32+".into(),
            file_type_extension: "exe".into(),
            file_type_mime: "application/x-msdownload".into(),
            entropy: 6.1,
            is_pe: true,
            keywords: vec!["powershell".into(), "downloadstring".into()],
            type_mismatch: false,
            pe: Some(PeInfo {
                machine: "AMD64".into(),
                architecture: "x64".into(),
                is_dll: false,
                is_executable: true,
                is_console: true,
                is_gui: false,
                image_base: 0x140000000,
                entry_point: 0x1000,
                timestamp: 0,
                timestamp_iso: String::new(),
                subsystem: "Windows CUI".into(),
                dll_characteristics: 0,
                has_certificate: false,
                certificate_size: 0,
                sections: vec![
                    PeSection {
                        name: ".text".into(),
                        virtual_size: 100,
                        virtual_address: 0x1000,
                        raw_size: 100,
                        entropy: 5.0,
                        flags: vec!["CODE".into(), "EXEC".into(), "READ".into()],
                    },
                    PeSection {
                        name: ".data".into(),
                        virtual_size: 50,
                        virtual_address: 0x2000,
                        raw_size: 50,
                        entropy: 4.0,
                        flags: vec!["INIT_DATA".into(), "READ".into(), "WRITE".into()],
                    },
                ],
                imports: vec![
                    PeImportDll {
                        name: "kernel32.dll".into(),
                        functions: vec![
                            "OpenProcess".into(),
                            "VirtualAllocEx".into(),
                            "WriteProcessMemory".into(),
                            "CreateRemoteThread".into(),
                        ],
                    },
                    PeImportDll {
                        name: "advapi32.dll".into(),
                        functions: vec!["RegSetValueExW".into(), "RegCreateKeyExW".into()],
                    },
                ],
                import_count: 6,
                exports: vec![],
                export_count: 0,
            }),
        }
    }

    #[test]
    fn detects_injection_and_persistence() {
        let findings = evaluate(Some(&sample_analysis()), None);
        let has = |id: &str| findings.iter().any(|f| f.rule_name == rule(id).name);
        assert!(has("process-injection"), "debería detectar inyección");
        assert!(has("persistence-registry"), "debería detectar persistencia");
        assert!(has("unsigned-pe"), "debería señalar binario sin firma");
        // Las reglas de strings no deben dispararse en un PE (ruido).
        assert!(!has("powershell-download"), "keywords no aplican a PEs");
        assert!(!has("eicar-detected"), "keywords no aplican a PEs");
    }

    #[test]
    fn keyword_rules_fire_for_non_pe() {
        let analysis = StaticAnalysis {
            file_type: "VBScript".into(),
            file_type_extension: "vbs".into(),
            file_type_mime: "text/vbscript".into(),
            entropy: 5.5,
            is_pe: false,
            keywords: vec![
                "powershell".into(),
                "downloadstring".into(),
                "schtasks".into(),
                "eicar".into(),
            ],
            type_mismatch: false,
            pe: None,
        };
        let findings = evaluate(Some(&analysis), None);
        let has = |id: &str| findings.iter().any(|f| f.rule_name == rule(id).name);
        assert!(has("powershell-download"), "script con powershell+descarga");
        assert!(has("persistence-scheduled"), "script con schtasks");
        assert!(has("eicar-detected"), "script con cadena EICAR");
        assert!(has("script-keywords"), "varias keywords sospechosas");
    }

    #[test]
    fn evaluates_own_test_binary() {
        let exe = std::env::current_exe().expect("current exe");
        let analysis = crate::analyzer::analyze(&exe).expect("análisis estático");
        let findings = evaluate(Some(&analysis), None);
        let score = score(&findings);
        assert!((0..=100).contains(&score), "score fuera de rango");
        let level = level_from_score(score);
        assert!(
            matches!(
                level,
                ThreatLevel::Low | ThreatLevel::Clean | ThreatLevel::Medium
            ),
            "el propio binario no debería elevarse a High/Critical"
        );
    }

    #[test]
    fn localized_descriptions() {
        use crate::models::Language;
        let es = description_in("Process injection imports", Language::Es);
        assert!(es.contains("inyección"), "descripción en español");
        assert_eq!(
            description_in("Process injection imports", Language::En),
            rule("process-injection").description,
            "en conserva la descripción original"
        );
        // Regla sin traducción disponible: conserva el nombre.
        assert_eq!(description_in("Unknown rule", Language::Es), "Unknown rule");
    }

    #[test]
    fn scoring_maps_levels() {
        assert_eq!(level_from_score(0), ThreatLevel::Clean);
        assert_eq!(level_from_score(10), ThreatLevel::Low);
        assert_eq!(level_from_score(20), ThreatLevel::Medium);
        assert_eq!(level_from_score(40), ThreatLevel::High);
        assert_eq!(level_from_score(90), ThreatLevel::Critical);
    }
}
