//! Tipos de datos compartidos entre el backend Rust y el frontend.
//!
//! Mantener este archivo sincronizado con `src/types/index.ts` del frontend.
//! La serialización usa `camelCase` para que los nombres coincidan con TS.

// Los tipos reservados para fases posteriores aún no se construyen;
// se mantienen como contrato de la API entre frontend y backend.
#![allow(dead_code)]

use serde::{Deserialize, Serialize};

/// Nivel de amenaza final de un análisis.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ThreatLevel {
    #[default]
    Clean,
    Low,
    Medium,
    High,
    Critical,
}

/// Riesgo educativo de un comando (terminal / shell).
///
/// Se usa tanto en la referencia de PowerShell (Windows) como en la
/// clasificación de comandos de shell (Linux/macOS).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RiskLevel {
    Safe,
    Low,
    Medium,
    High,
}

/// Idioma en el que se genera el contenido de la aplicación y del motor de
/// evaluación. El motor compone el informe directamente en este idioma.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Language {
    #[default]
    Es,
    En,
}

impl Language {
    /// Interpreta un valor de configuración (`"es"` | `"en"`).
    ///
    /// Un `es` explícito siempre produce español; cualquier valor desconocido
    /// o vacío cae en `En` como fallback técnico.
    pub fn from_config(value: &str) -> Language {
        match value.trim().to_ascii_lowercase().as_str() {
            "es" => Language::Es,
            _ => Language::En,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Language::Es => "es",
            Language::En => "en",
        }
    }
}

/// Severidad de un hallazgo individual.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Info,
    Low,
    Medium,
    High,
    Critical,
}

/// Hashes calculados para un archivo.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileHashes {
    pub md5: Option<String>,
    pub sha1: Option<String>,
    pub sha256: Option<String>,
}

/// Hallazgo generado por el motor de reglas heurísticas.
///
/// Una regla devuelve un `Finding` con evidencia concreta. La presencia de un
/// hallazgo NO implica malware por sí sola: aporta puntos al `threat_score`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Finding {
    pub rule_name: String,
    pub category: String,
    pub severity: Severity,
    pub description: String,
    pub evidence: Vec<String>,
    pub points: u32,
}

/// Ficha descriptiva de una regla del catálogo heurístico (página Reglas).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuleInfo {
    pub id: String,
    pub category: String,
    pub name: String,
    pub description: String,
    pub severity: Severity,
    pub points: u32,
}

/// Análisis estático del archivo (FASE 3).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StaticAnalysis {
    /// Tipo de archivo detectado por magic bytes (o por extensión).
    pub file_type: String,
    pub file_type_extension: String,
    pub file_type_mime: String,
    /// Entropía de Shannon (bits/byte) de todo el archivo, 0..8.
    pub entropy: f64,
    /// Si el archivo es un ejecutable/objeto PE de Windows.
    pub is_pe: bool,
    /// Palabras clave sospechosas encontradas en una muestra del archivo.
    pub keywords: Vec<String>,
    /// El tipo detectado por magic bytes contradice la extensión del archivo.
    pub type_mismatch: bool,
    /// Detalle PE (solo cuando `is_pe` es verdadero).
    pub pe: Option<PeInfo>,
}

/// Estructura de un archivo PE (portable executable).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PeInfo {
    pub machine: String,
    /// Arquitectura: "x86", "x64", "arm", "arm64", "unknown".
    pub architecture: String,
    pub is_dll: bool,
    pub is_executable: bool,
    pub is_console: bool,
    pub is_gui: bool,
    pub image_base: u64,
    /// RVA del punto de entrada.
    pub entry_point: u64,
    /// Marca de tiempo del header COFF (segundos desde epoch).
    pub timestamp: u64,
    pub timestamp_iso: String,
    pub subsystem: String,
    pub dll_characteristics: u32,
    pub has_certificate: bool,
    pub certificate_size: u32,
    pub sections: Vec<PeSection>,
    pub imports: Vec<PeImportDll>,
    pub import_count: u32,
    pub exports: Vec<String>,
    pub export_count: u32,
}

/// Sección de un PE.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PeSection {
    pub name: String,
    pub virtual_size: u64,
    pub virtual_address: u64,
    pub raw_size: u64,
    /// Entropía de Shannon de los datos en disco de la sección.
    pub entropy: f64,
    /// Flags human-readable (CODE, INIT_DATA, EXEC, READ, WRITE…).
    pub flags: Vec<String>,
}

/// DLL importada por un PE (y sus funciones).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PeImportDll {
    pub name: String,
    pub functions: Vec<String>,
}

/// Resultado de un motor (vendor) concreto dentro de un análisis de VirusTotal.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VtVendorResult {
    pub engine: String,
    pub category: String,
    pub result: Option<String>,
}

/// Resultado de la consulta a VirusTotal por hash (FASE 5).
///
/// `available == true`: el hash existe en VirusTotal y `stats`/`vendors`
/// contienen datos. `available == false` con `error == None`: el hash no
/// está reportado (404). Los errores de red/clave/límite llegan en `error`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VirusTotalResult {
    pub available: bool,
    pub hash: String,
    pub malicious: u32,
    pub suspicious: u32,
    pub harmless: u32,
    pub undetected: u32,
    pub timeout: u32,
    pub type_unsupported: u32,
    pub total: u32,
    pub reputation: i32,
    pub times_submitted: u32,
    pub first_submission_iso: Option<String>,
    pub last_analysis_iso: Option<String>,
    pub meaningful_name: Option<String>,
    pub magic: Option<String>,
    pub size: Option<u64>,
    pub threat_names: Vec<String>,
    pub vendors: Vec<VtVendorResult>,
    pub permalink: String,
    pub error: Option<String>,
}

/// Evaluación explicativa generada a partir de la evidencia (FASE 6).
///
/// Motor determinista y sin red: sintetiza hallazgos, análisis estático y
/// reputación en un informe en lenguaje natural. NUNCA inventa resultados:
/// todo lo que afirma procede de datos ya extraídos.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AiAssessment {
    /// Veredicto derivado de la evidencia: clean | likely_clean | suspicious | malicious.
    pub verdict: String,
    /// Confianza de la evaluación (0.0 a 1.0).
    pub confidence: f32,
    /// Resumen de una o dos frases.
    pub summary: String,
    /// Narrativa por párrafos (línea base + uno por categoría con hallazgos).
    pub explanation: String,
    /// Indicadores concretos que sustentan la evaluación.
    pub indicators: Vec<String>,
    /// Impacto potencial si el archivo fuese malicioso.
    pub potential_impact: Vec<String>,
    /// Consecuencias a nivel de sistema.
    pub system_consequences: Vec<String>,
    /// Acciones recomendadas.
    pub recommended_actions: Vec<String>,
    /// Vectores de ataque probables.
    pub attack_vectors: Vec<String>,
    /// Categorías con mayor peso en la evaluación (claves `rules.category.*`).
    pub key_categories: Vec<String>,
}

/// Entrada de la línea temporal de un análisis.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TimelineEntry {
    pub time: String,
    pub label: String,
}

/// Archivo aislado en cuarentena (FASE 7).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QuarantineEntry {
    /// Identificador estable (p. ej. `Q-2026-000042`).
    pub id: String,
    /// Ruta original antes de aislar (para poder restaurar).
    pub original_path: String,
    /// Nombre original del archivo.
    pub original_name: String,
    /// Ruta del blob ya dentro del directorio de cuarentena.
    pub quarantined_path: String,
    pub size: u64,
    pub hashes: FileHashes,
    /// Motivo aportado por el usuario al aislar.
    pub reason: String,
    pub threat_level: ThreatLevel,
    pub quarantined_at: String,
}

/// Vista de la cuarentena: directorio efectivo + entradas.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QuarantineSummary {
    pub dir: String,
    pub entries: Vec<QuarantineEntry>,
}

/// Resultado completo de un análisis.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanResult {
    pub id: String,
    pub file_name: String,
    pub path: String,
    pub size: u64,
    pub hashes: FileHashes,
    pub threat_score: u32,
    pub threat_level: ThreatLevel,
    pub findings: Vec<Finding>,
    pub static_analysis: Option<StaticAnalysis>,
    pub reputation: Option<VirusTotalResult>,
    pub ai_assessment: Option<AiAssessment>,
    /// Idioma con el que se generó el contenido de este resultado.
    #[serde(default)]
    pub language: Language,
    pub scanned_at: String,
    pub timeline: Vec<TimelineEntry>,
}

/// Tipo de objetivo escaneado.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ScanKind {
    File,
    Folder,
}

/// Formato de informe exportable (FASE 8).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ReportFormat {
    Html,
    Csv,
}

/// Entrada del historial de análisis (resumen ligero para listados).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanHistoryEntry {
    /// Identificador estable del análisis (UUID). Las entradas antiguas sin id
    /// reciben uno derivado al cargar el historial (ver `scanner::history`).
    #[serde(default)]
    pub id: String,
    pub kind: ScanKind,
    pub path: String,
    pub name: String,
    /// Tamaño total en bytes (archivo) o suma de archivos (carpeta).
    pub size: u64,
    /// Número de archivos encontrados (carpeta).
    pub file_count: u32,
    /// Archivos con error de lectura (carpeta).
    pub error_count: u32,
    pub threat_level: ThreatLevel,
    pub scanned_at: String,
    pub duration_ms: u64,
}

/// Archivo individual dentro de un escaneo de carpeta.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FolderFileEntry {
    /// Ruta relativa a la carpeta escaneada.
    pub relative_path: String,
    pub size: u64,
    pub hashes: FileHashes,
    pub error: Option<String>,
}

/// Resultado de un escaneo de carpeta (recursivo).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FolderScanResult {
    pub id: String,
    pub folder_path: String,
    pub file_count: u32,
    pub scanned_count: u32,
    pub skipped_count: u32,
    pub error_count: u32,
    pub total_bytes: u64,
    pub scanned_at: String,
    pub duration_ms: u64,
    pub files: Vec<FolderFileEntry>,
}

/// Información básica de una ruta (para preparar el escaneo en la UI).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PathInfo {
    pub path: String,
    pub name: String,
    pub is_dir: bool,
    pub size: u64,
}

/// Información básica del sistema host.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SystemInfo {
    pub os_name: String,
    pub os_version: String,
    pub os_edition: Option<String>,
    pub os_family: String,
    pub architecture: String,
    pub hostname: String,
    pub username: String,
    pub cpu_physical_cores: usize,
    pub cpu_virtual_cores: usize,
    pub total_memory_bytes: u64,
}

/// Metadatos de la propia aplicación.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppInfo {
    pub name: String,
    pub version: String,
    pub tagline: String,
}
