//! Recolección de información básica del sistema.
//!
//! Solo se lee información del host: OS, arquitectura, hostname, usuario,
//! CPU y memoria. No se realiza ninguna acción sobre el sistema.

use std::env;

use sysinfo::System;

use crate::models::SystemInfo;

/// Recopila la información básica del sistema.
pub fn collect() -> Result<SystemInfo, String> {
    let os = os_info::get();

    let mut sys = System::new_all();
    sys.refresh_memory();
    sys.refresh_cpu_all();

    Ok(SystemInfo {
        os_name: os.os_type().to_string(),
        os_version: os.version().to_string(),
        os_edition: os.edition().map(|e| e.to_string()),
        os_family: env::consts::FAMILY.to_string(),
        architecture: env::consts::ARCH.to_string(),
        hostname: hostname(),
        username: username(),
        cpu_physical_cores: System::physical_core_count().unwrap_or(0),
        cpu_virtual_cores: sys.cpus().len(),
        total_memory_bytes: sys.total_memory(),
    })
}

/// Nombre del equipo, multiplataforma.
fn hostname() -> String {
    #[cfg(target_os = "windows")]
    {
        env::var("COMPUTERNAME").unwrap_or_else(|_| hostname_fallback())
    }
    #[cfg(not(target_os = "windows"))]
    {
        hostname_fallback()
    }
}

/// Nombre de usuario, multiplataforma.
fn username() -> String {
    #[cfg(target_os = "windows")]
    {
        env::var("USERNAME").unwrap_or_else(|_| username_fallback())
    }
    #[cfg(target_os = "macos")]
    {
        env::var("USER").unwrap_or_else(|_| username_fallback())
    }
    #[cfg(target_os = "linux")]
    {
        env::var("USER").unwrap_or_else(|_| username_fallback())
    }
}

/// Fallback: ejecuta `hostname` (Unix) o `echo %COMPUTERNAME%` (Windows).
fn hostname_fallback() -> String {
    std::process::Command::new("hostname")
        .output()
        .ok()
        .and_then(|o| {
            if o.status.success() {
                Some(String::from_utf8_lossy(&o.stdout).trim().to_string())
            } else {
                None
            }
        })
        .unwrap_or_else(|| "unknown".into())
}

/// Fallback para usuario: ejecuta `whoami` en Unix.
fn username_fallback() -> String {
    std::process::Command::new("whoami")
        .output()
        .ok()
        .and_then(|o| {
            if o.status.success() {
                Some(String::from_utf8_lossy(&o.stdout).trim().to_string())
            } else {
                None
            }
        })
        .unwrap_or_else(|| "unknown".into())
}
