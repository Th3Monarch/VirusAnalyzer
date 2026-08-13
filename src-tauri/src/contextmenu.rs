//! Menú contextual de Windows (integración con el Explorador).
//!
//! Registra «Analizar con VirusAnalyzer» en
//! `HKCU\Software\Classes\*\shell\VirusAnalyzer` para que el botón derecho
//! sobre un archivo o carpeta permita abrir el análisis directamente.
//!
//! - Solo afecta al usuario actual (sin privilegios de administrador).
//! - Se usa `reg.exe` (binario del sistema) sin pasar por un shell, por lo que
//!   no hay expansión de `%1`: el literal se guarda tal cual y lo expande el
//!   Explorador al hacer clic.

use std::process::Command;

/// Clave del registro. El `*` cubre archivos y carpetas.
const SHELL_KEY: &str = r"HKCU\Software\Classes\*\shell\VirusAnalyzer";

fn reg(args: &[&str]) -> Result<(), String> {
    let output = Command::new("reg.exe")
        .args(args)
        .output()
        .map_err(|e| format!("No se pudo ejecutar reg.exe: {e}"))?;
    if output.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let msg = stderr.trim();
        if msg.is_empty() {
            Err(format!(
                "reg.exe devolvió el código {}",
                output.status.code().unwrap_or(-1)
            ))
        } else {
            Err(msg.to_string())
        }
    }
}

/// Indica si el menú contextual está registrado actualmente.
pub fn is_installed() -> Result<bool, String> {
    let output = Command::new("reg.exe")
        .args(["query", SHELL_KEY])
        .output()
        .map_err(|e| format!("No se pudo consultar el registro: {e}"))?;
    Ok(output.status.success())
}

/// Registra el menú contextual apuntando al ejecutable actual.
pub fn install(label: &str) -> Result<(), String> {
    let exe = std::env::current_exe()
        .map_err(|e| format!("No se pudo localizar el ejecutable de la aplicación: {e}"))?;
    let exe_str = exe.to_string_lossy().to_string();

    reg(&["add", SHELL_KEY, "/ve", "/d", label, "/f"])?;
    reg(&["add", SHELL_KEY, "/v", "Icon", "/d", &exe_str, "/f"])?;

    let command = format!("\"{exe_str}\" \"%1\"");
    reg(&[
        "add",
        &format!("{SHELL_KEY}\\command"),
        "/ve",
        "/d",
        &command,
        "/f",
    ])?;
    Ok(())
}

/// Elimina el menú contextual. Es idempotente.
pub fn uninstall() -> Result<(), String> {
    if !is_installed()? {
        return Ok(());
    }
    reg(&["delete", SHELL_KEY, "/f"])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn query_never_fails() {
        // Consultar el estado del registro no debe fallar en Windows.
        let result = is_installed();
        assert!(result.is_ok());
    }
}
