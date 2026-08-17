//! Capa de abstracción multiplataforma.
//!
//! Cada funcionalidad dependiente del SO se ejecuta a través de un trait
//! definido aquí y por proveedores concretos en los módulos `windows`,
//! `linux` y `macos`. El frontend usa los mismos comandos Tauri
//! independientemente del SO.
//!
//! # Convención de uso
//!
//! ```ignore
//! let terminal = platform::current_terminal();
//! let result = terminal.execute("ls -la", 30_000)?;
//! ```

pub mod terminal;

#[cfg(target_os = "windows")]
mod windows;
#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "macos")]
mod macos;

use std::sync::Arc;

use serde::{Deserialize, Serialize};

pub use terminal::{RiskLevel, TerminalCommandInfo, TerminalManager, TerminalResult};

/// Identificador de plataforma runtime, serializable para el frontend.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Platform {
    Windows,
    Linux,
    #[serde(rename = "macos")]
    Macos,
}

impl Platform {
    /// Detecta la plataforma actual en tiempo de compilación.
    pub fn current() -> Self {
        if cfg!(target_os = "windows") {
            Platform::Windows
        } else if cfg!(target_os = "macos") {
            Platform::Macos
        } else {
            Platform::Linux
        }
    }
}

impl std::fmt::Display for Platform {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Platform::Windows => write!(f, "Windows"),
            Platform::Linux => write!(f, "Linux"),
            Platform::Macos => write!(f, "macOS"),
        }
    }
}

/// Trait para proveedores de menú contextual.
///
/// En Windows se integra con el Explorador vía registro. En Linux/macOS
/// no hay implementación real y los métodos devuelven errores o `false`.
pub trait ContextMenuProvider: Send + Sync {
    fn install(&self, label: &str) -> Result<(), String>;
    fn uninstall(&self) -> Result<(), String>;
    fn is_installed(&self) -> Result<bool, String>;
}

/// Crea el proveedor de terminal de la plataforma actual.
pub fn current_terminal() -> Arc<dyn TerminalManager> {
    #[cfg(target_os = "windows")]
    {
        Arc::new(windows::WindowsTerminalManager::new())
    }
    #[cfg(target_os = "linux")]
    {
        Arc::new(linux::LinuxTerminalManager::new())
    }
    #[cfg(target_os = "macos")]
    {
        Arc::new(macos::MacOSTerminalManager::new())
    }
}

/// Crea el proveedor de menú contextual de la plataforma actual.
pub fn current_context_menu() -> Arc<dyn ContextMenuProvider> {
    #[cfg(target_os = "windows")]
    {
        Arc::new(windows::WindowsContextMenu)
    }
    #[cfg(target_os = "linux")]
    {
        Arc::new(linux::StubContextMenu)
    }
    #[cfg(target_os = "macos")]
    {
        Arc::new(macos::StubContextMenu)
    }
}
