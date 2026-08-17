//! Implementación Windows de [`TerminalManager`] y [`ContextMenuProvider`].
//!
//! Delega a los módulos existentes `powershell`, `powershell_reference` y
//! `contextmenu` que ya encapsulan toda la lógica específica de Windows.

use crate::contextmenu;
use crate::models::Language;
use crate::powershell;
use crate::powershell_reference;

use super::terminal::{RiskLevel, TerminalCommandInfo, TerminalManager, TerminalResult};
use super::ContextMenuProvider;

/// Proveedor de terminal Windows: ejecuta comandos vía `powershell.exe`.
///
/// Mantiene una instancia de [`powershell::PowerShellManager`] internamente
/// para que `execute` y `cancel` compartan el mismo estado de ejecución
/// activa.
pub struct WindowsTerminalManager {
    manager: powershell::PowerShellManager,
}

impl WindowsTerminalManager {
    pub fn new() -> Self {
        Self {
            manager: powershell::PowerShellManager::default(),
        }
    }

    /// Devuelve una referencia al manager subyacente para que los comandos
    /// Tauri que aún usan el tipo concreto puedan acceder a él (transición).
    #[allow(dead_code)]
    pub fn inner(&self) -> &powershell::PowerShellManager {
        &self.manager
    }
}

impl Default for WindowsTerminalManager {
    fn default() -> Self {
        Self::new()
    }
}

impl TerminalManager for WindowsTerminalManager {
    fn execute(&self, command: &str, timeout_ms: u64) -> Result<TerminalResult, String> {
        let ps_result = powershell::execute(&self.manager, command, timeout_ms)?;
        Ok(TerminalResult {
            stdout: ps_result.stdout,
            stderr: ps_result.stderr,
            exit_code: ps_result.exit_code,
            duration_ms: ps_result.duration_ms,
            timed_out: ps_result.timed_out,
            cancelled: ps_result.cancelled,
            command: ps_result.command,
        })
    }

    fn cancel(&self) -> bool {
        powershell::cancel(&self.manager)
    }

    fn is_available(&self) -> bool {
        powershell::resolve_powershell().is_some()
    }

    fn classify(&self, command: &str) -> RiskLevel {
        powershell_reference::risk_for(command)
    }

    fn get_reference(&self, language: Language) -> Vec<TerminalCommandInfo> {
        powershell_reference::catalog(language)
            .into_iter()
            .map(|c| TerminalCommandInfo {
                name: c.name,
                category: c.category,
                description: c.description,
                usage: c.usage,
                example: c.example,
                risk: c.risk,
                warning: c.warning,
            })
            .collect()
    }

    fn prompt_label(&self) -> &'static str {
        "PS >"
    }

    fn display_name(&self) -> &'static str {
        "PowerShell"
    }
}

/// Menú contextual Windows: integra con el Explorador vía registro.
pub struct WindowsContextMenu;

impl ContextMenuProvider for WindowsContextMenu {
    fn install(&self, label: &str) -> Result<(), String> {
        contextmenu::install(label)
    }

    fn uninstall(&self) -> Result<(), String> {
        contextmenu::uninstall()
    }

    fn is_installed(&self) -> Result<bool, String> {
        contextmenu::is_installed()
    }
}
