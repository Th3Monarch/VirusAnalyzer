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
        hostname: env::var("COMPUTERNAME").unwrap_or_else(|_| "unknown".into()),
        username: env::var("USERNAME").unwrap_or_else(|_| "unknown".into()),
        cpu_physical_cores: System::physical_core_count().unwrap_or(0),
        cpu_virtual_cores: sys.cpus().len(),
        total_memory_bytes: sys.total_memory(),
    })
}
