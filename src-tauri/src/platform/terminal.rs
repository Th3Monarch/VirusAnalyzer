//! Abstracción unificada de terminal / shell para todas las plataformas.
//!
//! Define el trait [`TerminalManager`] que cada plataforma implementa con
//! su shell nativo: PowerShell en Windows, `$SHELL` o `/bin/sh` en
//! Linux/macOS.
//!
//! El frontend usa los mismos comandos Tauri independientemente del SO; la
//! capa `platform` delega al proveedor correcto.

use serde::{Deserialize, Serialize};

pub use crate::models::RiskLevel;

/// Timeout por defecto de una ejecución (30 s).
pub const DEFAULT_TIMEOUT_MS: u64 = 30_000;

/// Límite de longitud del comando para evitar abusos evidentes.
pub const MAX_COMMAND_LEN: usize = 64 * 1024;

/// Resultado de una ejecución de terminal.
///
/// Estructuralmente idéntico al anterior `PowerShellResult` para mantener
/// compatibilidad con el contrato del frontend.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TerminalResult {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: Option<i32>,
    pub duration_ms: u64,
    pub timed_out: bool,
    pub cancelled: bool,
    pub command: String,
}

/// Ficha educativa de un comando de la referencia.
///
/// Equivalente al anterior `PsCommandInfo`; el nombre es genérico para
/// reflejar que en Linux/macOS no se refiere a PowerShell.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TerminalCommandInfo {
    pub name: String,
    pub category: String,
    pub description: String,
    pub usage: String,
    pub example: String,
    pub risk: RiskLevel,
    pub warning: Option<String>,
}

/// Trait que cada plataforma debe implementar para ofrecer terminal.
#[allow(dead_code)]
pub trait TerminalManager: Send + Sync {
    /// Ejecuta un comando con un timeout máximo en milisegundos.
    ///
    /// `Err` solo se devuelve por fallos técnicos (shell no disponible,
    /// proceso que no pudo iniciarse). El resto (salida, error, timeout,
    /// cancelación) se devuelve en [`TerminalResult`].
    fn execute(&self, command: &str, timeout_ms: u64) -> Result<TerminalResult, String>;

    /// Cancela la ejecución activa (si la hay). Devuelve `true` si había
    /// algo que cancelar.
    fn cancel(&self) -> bool;

    /// Indica si el shell está disponible en este sistema.
    fn is_available(&self) -> bool;

    /// Clasificación educativa del riesgo de un comando.
    fn classify(&self, command: &str) -> RiskLevel;

    /// Catálogo educativo de comandos disponibles para esta plataforma.
    fn get_reference(&self, language: crate::models::Language) -> Vec<TerminalCommandInfo>;

    /// Etiqueta que se muestra antes del prompt (p.ej. `PS >`, `$ `).
    fn prompt_label(&self) -> &'static str;

    /// Nombre legible del terminal (p.ej. "PowerShell", "Terminal").
    fn display_name(&self) -> &'static str;
}
