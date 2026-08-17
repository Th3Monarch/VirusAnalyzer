//! Referencia educativa de comandos PowerShell y clasificación de riesgo.
//!
//! SEPARACIÓN: este módulo únicamente explica comandos (catálogo, categorías,
//! riesgo educativo). NUNCA ejecuta nada; la ejecución vive en `powershell.rs`.
//!
//! La clasificación de riesgo es **educativa** y no pretende ser un sistema de
//! seguridad perfecto: sirve para informar y pedir confirmación en acciones de
//! alto impacto, no para bloquearlas.

use serde::{Deserialize, Serialize};

use crate::models::Language;

/// Re-export del tipo compartido en `models`.
pub use crate::models::RiskLevel;

/// Ficha educativa de un comando de la referencia.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PsCommandInfo {
    pub name: String,
    /// Clave de categoría (traducida en el frontend).
    pub category: String,
    pub description: String,
    pub usage: String,
    pub example: String,
    pub risk: RiskLevel,
    pub warning: Option<String>,
}

/// Categorías disponibles (claves; las etiquetas las traduce la UI).
#[allow(dead_code)] // se usa en los tests y en la documentación del catálogo.
pub const CATEGORIES: [&str; 7] = [
    "system",
    "processes",
    "services",
    "networking",
    "files",
    "security",
    "diagnostics",
];

struct Entry {
    name: &'static str,
    category: &'static str,
    description_es: &'static str,
    description_en: &'static str,
    usage: &'static str,
    example: &'static str,
    risk: RiskLevel,
    warning_es: Option<&'static str>,
    warning_en: Option<&'static str>,
}

const ENTRIES: &[Entry] = &[
    Entry {
        name: "Get-ComputerInfo",
        category: "system",
        description_es: "Devuelve información completa del sistema (SO, BIOS, hardware).",
        description_en: "Returns comprehensive system information (OS, BIOS, hardware).",
        usage: "Get-ComputerInfo",
        example: "Get-ComputerInfo | Format-List",
        risk: RiskLevel::Safe,
        warning_es: None,
        warning_en: None,
    },
    Entry {
        name: "Get-CimInstance",
        category: "system",
        description_es: "Consulta instancias de WMI/CIM (sistema, hardware, SO).",
        description_en: "Queries WMI/CIM instances (system, hardware, OS).",
        usage: "Get-CimInstance <Class>",
        example: "Get-CimInstance Win32_OperatingSystem",
        risk: RiskLevel::Low,
        warning_es: None,
        warning_en: None,
    },
    Entry {
        name: "Get-Host",
        category: "system",
        description_es: "Muestra información de la sesión actual de PowerShell.",
        description_en: "Shows information about the current PowerShell session.",
        usage: "Get-Host",
        example: "Get-Host | Select-Object Version",
        risk: RiskLevel::Safe,
        warning_es: None,
        warning_en: None,
    },
    Entry {
        name: "Get-Process",
        category: "processes",
        description_es: "Muestra los procesos en ejecución y su consumo de recursos.",
        description_en: "Shows running processes and their resource usage.",
        usage: "Get-Process [-Name <name>]",
        example: "Get-Process | Sort-Object CPU -Descending",
        risk: RiskLevel::Safe,
        warning_es: None,
        warning_en: None,
    },
    Entry {
        name: "Stop-Process",
        category: "processes",
        description_es: "Termina uno o varios procesos.",
        description_en: "Terminates one or more processes.",
        usage: "Stop-Process -Name <name> [-Force]",
        example: "Stop-Process -Name notepad",
        risk: RiskLevel::Medium,
        warning_es: Some("Termina procesos: se perderá cualquier trabajo no guardado del proceso."),
        warning_en: Some("Terminates processes: unsaved work in those processes will be lost."),
    },
    Entry {
        name: "Start-Process",
        category: "processes",
        description_es: "Inicia un proceso o abre un documento con su aplicación.",
        description_en: "Starts a process or opens a document with its associated app.",
        usage: "Start-Process <path> [-ArgumentList <args>]",
        example: "Start-Process notepad",
        risk: RiskLevel::Medium,
        warning_es: Some("Ejecuta programas; comprueba que el archivo es de confianza."),
        warning_en: Some("Runs programs; make sure the file is trustworthy."),
    },
    Entry {
        name: "Get-Service",
        category: "services",
        description_es: "Muestra los servicios del sistema y su estado.",
        description_en: "Shows system services and their status.",
        usage: "Get-Service",
        example: "Get-Service | Where-Object {$_.Status -eq 'Running'}",
        risk: RiskLevel::Safe,
        warning_es: None,
        warning_en: None,
    },
    Entry {
        name: "Restart-Service",
        category: "services",
        description_es: "Reinicia un servicio del sistema.",
        description_en: "Restarts a system service.",
        usage: "Restart-Service -Name <service>",
        example: "Restart-Service -Name spooler",
        risk: RiskLevel::Medium,
        warning_es: Some("Reinicia servicios: puede interrumpir temporalmente funciones del sistema."),
        warning_en: Some("Restarts services: may temporarily interrupt system functions."),
    },
    Entry {
        name: "Stop-Service",
        category: "services",
        description_es: "Detiene un servicio del sistema.",
        description_en: "Stops a system service.",
        usage: "Stop-Service -Name <service> [-Force]",
        example: "Stop-Service -Name spooler",
        risk: RiskLevel::High,
        warning_es: Some("Detiene servicios: puede dejar funciones del sistema sin respuesta."),
        warning_en: Some("Stops services: may leave system functions unresponsive."),
    },
    Entry {
        name: "Get-NetIPConfiguration",
        category: "networking",
        description_es: "Muestra la configuración de red de las interfaces.",
        description_en: "Shows the network configuration of the interfaces.",
        usage: "Get-NetIPConfiguration",
        example: "Get-NetIPConfiguration",
        risk: RiskLevel::Safe,
        warning_es: None,
        warning_en: None,
    },
    Entry {
        name: "Get-NetTCPConnection",
        category: "networking",
        description_es: "Muestra las conexiones TCP activas del sistema.",
        description_en: "Shows the active TCP connections on the system.",
        usage: "Get-NetTCPConnection [-State <state>]",
        example: "Get-NetTCPConnection -State Established",
        risk: RiskLevel::Low,
        warning_es: None,
        warning_en: None,
    },
    Entry {
        name: "Test-NetConnection",
        category: "networking",
        description_es: "Comprueba la conectividad con un host (ICMP, puerto, DNS).",
        description_en: "Tests connectivity to a host (ICMP, port, DNS).",
        usage: "Test-NetConnection <host> [-Port <port>]",
        example: "Test-NetConnection example.com -Port 443",
        risk: RiskLevel::Low,
        warning_es: None,
        warning_en: None,
    },
    Entry {
        name: "Resolve-DnsName",
        category: "networking",
        description_es: "Resuelve nombres DNS y muestra registros.",
        description_en: "Resolves DNS names and shows records.",
        usage: "Resolve-DnsName <name>",
        example: "Resolve-DnsName example.com",
        risk: RiskLevel::Low,
        warning_es: None,
        warning_en: None,
    },
    Entry {
        name: "Get-ChildItem",
        category: "files",
        description_es: "Lista archivos y carpetas de una ruta.",
        description_en: "Lists files and folders in a path.",
        usage: "Get-ChildItem [-Path <path>]",
        example: "Get-ChildItem C:\\Users\\Public -Recurse",
        risk: RiskLevel::Safe,
        warning_es: None,
        warning_en: None,
    },
    Entry {
        name: "Get-Item",
        category: "files",
        description_es: "Obtiene el elemento de una ruta (archivo, carpeta, clave).",
        description_en: "Gets the item at a path (file, folder, key).",
        usage: "Get-Item -Path <path>",
        example: "Get-Item C:\\Windows\\System32\\notepad.exe",
        risk: RiskLevel::Safe,
        warning_es: None,
        warning_en: None,
    },
    Entry {
        name: "Get-FileHash",
        category: "files",
        description_es: "Calcula el hash (MD5, SHA-1, SHA-256, etc.) de un archivo.",
        description_en: "Computes a hash (MD5, SHA-1, SHA-256, etc.) of a file.",
        usage: "Get-FileHash -Path <path> -Algorithm <algorithm>",
        example: "Get-FileHash C:\\Temp\\file.exe -Algorithm SHA256",
        risk: RiskLevel::Safe,
        warning_es: None,
        warning_en: None,
    },
    Entry {
        name: "Get-Content",
        category: "files",
        description_es: "Lee el contenido de un archivo de texto.",
        description_en: "Reads the contents of a text file.",
        usage: "Get-Content -Path <path>",
        example: "Get-Content C:\\Temp\\readme.txt",
        risk: RiskLevel::Safe,
        warning_es: None,
        warning_en: None,
    },
    Entry {
        name: "Remove-Item",
        category: "files",
        description_es: "Elimina archivos, carpetas o claves de registro.",
        description_en: "Deletes files, folders or registry keys.",
        usage: "Remove-Item -Path <path> [-Recurse] [-Force]",
        example: "Remove-Item C:\\Temp\\old -Recurse",
        risk: RiskLevel::High,
        warning_es: Some("Borra datos de forma permanente; no se puede deshacer."),
        warning_en: Some("Permanently deletes data; it cannot be undone."),
    },
    Entry {
        name: "Copy-Item",
        category: "files",
        description_es: "Copia archivos o carpetas a otra ubicación.",
        description_en: "Copies files or folders to another location.",
        usage: "Copy-Item -Path <origen> -Destination <destino>",
        example: "Copy-Item C:\\Temp\\file.txt C:\\Backup\\",
        risk: RiskLevel::Low,
        warning_es: None,
        warning_en: None,
    },
    Entry {
        name: "Get-MpComputerStatus",
        category: "security",
        description_es: "Muestra el estado de Windows Defender en este equipo.",
        description_en: "Shows the Windows Defender status on this machine.",
        usage: "Get-MpComputerStatus",
        example: "Get-MpComputerStatus | Select-Object AntivirusEnabled, RealTimeProtectionEnabled",
        risk: RiskLevel::Low,
        warning_es: None,
        warning_en: None,
    },
    Entry {
        name: "Get-MpThreat",
        category: "security",
        description_es: "Muestra las amenazas detectadas por Windows Defender.",
        description_en: "Shows the threats detected by Windows Defender.",
        usage: "Get-MpThreat",
        example: "Get-MpThreat | Format-List ThreatName, SeverityID",
        risk: RiskLevel::Low,
        warning_es: None,
        warning_en: None,
    },
    Entry {
        name: "Get-MpThreatDetection",
        category: "security",
        description_es: "Muestra el historial de detecciones de Windows Defender.",
        description_en: "Shows the Windows Defender detection history.",
        usage: "Get-MpThreatDetection",
        example: "Get-MpThreatDetection | Select-Object InitialDetectionTime, ProcessName",
        risk: RiskLevel::Low,
        warning_es: None,
        warning_en: None,
    },
    Entry {
        name: "Get-ExecutionPolicy",
        category: "security",
        description_es: "Muestra la política de ejecución de scripts de PowerShell.",
        description_en: "Shows the PowerShell script execution policy.",
        usage: "Get-ExecutionPolicy",
        example: "Get-ExecutionPolicy -List",
        risk: RiskLevel::Safe,
        warning_es: None,
        warning_en: None,
    },
    Entry {
        name: "Set-ExecutionPolicy",
        category: "security",
        description_es: "Cambia la política de ejecución de scripts de PowerShell.",
        description_en: "Changes the PowerShell script execution policy.",
        usage: "Set-ExecutionPolicy <policy>",
        example: "Set-ExecutionPolicy RemoteSigned -Scope CurrentUser",
        risk: RiskLevel::High,
        warning_es: Some("Cambia la seguridad de ejecución de scripts en el sistema."),
        warning_en: Some("Changes the script execution security on the system."),
    },
    Entry {
        name: "Get-HotFix",
        category: "diagnostics",
        description_es: "Muestra las actualizaciones y revisiones instaladas.",
        description_en: "Shows installed updates and hotfixes.",
        usage: "Get-HotFix",
        example: "Get-HotFix | Sort-Object InstalledOn -Descending",
        risk: RiskLevel::Low,
        warning_es: None,
        warning_en: None,
    },
    Entry {
        name: "Get-WinEvent",
        category: "diagnostics",
        description_es: "Lee registros de eventos de Windows (Sistema, Aplicación, etc.).",
        description_en: "Reads Windows event logs (System, Application, etc.).",
        usage: "Get-WinEvent -LogName <log> [-MaxEvents <n>]",
        example: "Get-WinEvent -LogName System -MaxEvents 20",
        risk: RiskLevel::Low,
        warning_es: None,
        warning_en: None,
    },
];

/// Devuelve el catálogo de la referencia traducido al idioma indicado.
pub fn catalog(lang: Language) -> Vec<PsCommandInfo> {
    let es = lang == Language::Es;
    ENTRIES
        .iter()
        .map(|e| PsCommandInfo {
            name: e.name.to_string(),
            category: e.category.to_string(),
            description: if es {
                e.description_es
            } else {
                e.description_en
            }
            .to_string(),
            usage: e.usage.to_string(),
            example: e.example.to_string(),
            risk: e.risk,
            warning: (if es { e.warning_es } else { e.warning_en }).map(String::from),
        })
        .collect()
}

/// Clasificación educativa del riesgo de un comando arbitrario.
///
/// Heurística por tokens (no infalible): detecta verbos destructivos, cambios
/// de configuración y ejecución de código dinámico. Si no hay coincidencia se
/// asume `Safe`.
pub fn risk_for(raw: &str) -> RiskLevel {
    let lower = raw.trim().to_lowercase();
    let tokens: Vec<&str> = lower
        .split(|c: char| !c.is_alphanumeric())
        .filter(|t| !t.is_empty())
        .collect();

    let has_pair = |a: &str, b: &str| {
        tokens
            .windows(2)
            .any(|w| w[0] == a && w[1] == b)
    };
    let has = |t: &str| tokens.contains(&t);

    const HIGH_PAIRS: [(&str, &str); 24] = [
        ("remove", "item"),
        ("delete", "item"),
        ("remove", "itemproperty"),
        ("new", "itemproperty"),
        ("set", "itemproperty"),
        ("remove", "psdrive"),
        ("stop", "service"),
        ("set", "service"),
        ("new", "service"),
        ("remove", "service"),
        ("stop", "computer"),
        ("restart", "computer"),
        ("invoke", "expression"),
        ("set", "executionpolicy"),
        ("clear", "eventlog"),
        ("format", "volume"),
        ("new", "localuser"),
        ("remove", "localuser"),
        ("set", "localuser"),
        ("add", "type"),
        ("remove", "appxpackage"),
        ("reg", "delete"),
        ("reg", "add"),
        ("net", "user"),
    ];
    const HIGH_SINGLE: [&str; 17] = [
        "rm", "del", "erase", "rmdir", "rd", "shutdown", "iex", "taskkill", "format", "cipher",
        "diskpart", "takeown", "icacls", "cacls", "reg", "netsh", "net",
    ];
    const MEDIUM_PAIRS: [(&str, &str); 14] = [
        ("stop", "process"),
        ("start", "process"),
        ("start", "service"),
        ("restart", "service"),
        ("new", "item"),
        ("move", "item"),
        ("copy", "item"),
        ("set", "item"),
        ("new", "psdrive"),
        ("start", "job"),
        ("stop", "job"),
        ("invoke", "webrequest"),
        ("install", "module"),
        ("set", "acl"),
    ];
    const MEDIUM_SINGLE: [&str; 6] = [
        "wmic", "chkdsk", "sfc", "dism", "regsvr32", "certutil",
    ];

    for (a, b) in HIGH_PAIRS {
        if has_pair(a, b) {
            return RiskLevel::High;
        }
    }
    if HIGH_SINGLE.iter().any(|t| has(t)) {
        return RiskLevel::High;
    }
    for (a, b) in MEDIUM_PAIRS {
        if has_pair(a, b) {
            return RiskLevel::Medium;
        }
    }
    if MEDIUM_SINGLE.iter().any(|t| has(t)) {
        return RiskLevel::Medium;
    }
    RiskLevel::Safe
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalog_is_complete_and_unique() {
        let cat = catalog(Language::En);
        assert!(!cat.is_empty());
        let mut names: Vec<&str> = cat.iter().map(|c| c.name.as_str()).collect();
        names.sort();
        names.dedup();
        assert_eq!(names.len(), cat.len(), "no debe haber nombres duplicados");
        for c in &cat {
            assert!(!c.description.trim().is_empty());
            assert!(!c.usage.trim().is_empty());
            assert!(!c.example.trim().is_empty());
            assert!(CATEGORIES.contains(&c.category.as_str()), "categoría válida");
        }
    }

    #[test]
    fn es_catalog_has_spanish_descriptions() {
        let cat = catalog(Language::Es);
        assert!(cat.iter().any(|c| c.description.contains("Muestra")));
        let en = catalog(Language::En);
        assert!(en.iter().all(|c| !c.description.chars().any(|ch| ch == 'á' || ch == 'ñ')));
    }

    #[test]
    fn risk_of_read_only_commands_is_low() {
        // La heurística devuelve Safe para comandos de solo lectura (no pide
        // confirmación); la etiqueta educativa "Low" del catálogo es aparte.
        assert_eq!(risk_for("Get-Date"), RiskLevel::Safe);
        assert_eq!(risk_for("Get-Process | Sort-Object CPU -Descending"), RiskLevel::Safe);
        assert_eq!(risk_for("Get-ChildItem C:\\Temp"), RiskLevel::Safe);
        assert_eq!(risk_for("Get-NetTCPConnection -State Established"), RiskLevel::Safe);
        assert_eq!(risk_for("Get-WinEvent -LogName System -MaxEvents 20"), RiskLevel::Safe);
    }

    #[test]
    fn risk_of_destructive_commands_is_high() {
        assert_eq!(risk_for("Remove-Item C:\\Temp\\old -Recurse -Force"), RiskLevel::High);
        assert_eq!(risk_for("Stop-Service -Name spooler"), RiskLevel::High);
        assert_eq!(risk_for("Invoke-Expression (Get-Content script.ps1)"), RiskLevel::High);
        assert_eq!(risk_for("Set-ExecutionPolicy Bypass"), RiskLevel::High);
        assert_eq!(risk_for("restart-computer -Force"), RiskLevel::High);
        assert_eq!(risk_for("shutdown /s /t 0"), RiskLevel::High);
        assert_eq!(risk_for("net user"), RiskLevel::High);
    }

    #[test]
    fn risk_of_medium_commands() {
        assert_eq!(risk_for("Stop-Process -Name notepad"), RiskLevel::Medium);
        assert_eq!(risk_for("Restart-Service -Name spooler"), RiskLevel::Medium);
        assert_eq!(risk_for("Start-Process notepad"), RiskLevel::Medium);
    }

    #[test]
    fn risk_ignores_case_and_whitespace() {
        assert_eq!(risk_for("  remove-ITEM  -Path C:\\x  "), RiskLevel::High);
    }
}
