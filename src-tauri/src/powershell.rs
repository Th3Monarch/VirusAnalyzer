//! Ejecutor de Windows PowerShell (módulo PowerShell).
//!
//! Ejecuta el comando introducido por el usuario de forma controlada:
//!
//! - Usa `powershell.exe` directamente (nunca `cmd.exe → powershell.exe`).
//! - El comando se pasa como **un único argumento** de `-Command`, sin
//!   interpolación de shell: se trata como una entrada independiente.
//! - NO eleva privilegios: se ejecuta con los permisos del usuario actual.
//! - Timeout configurable para que ningún proceso quede colgado.
//! - Cancelación explícita desde la UI (mata el proceso hijo).
//! - El analizador de malware NUNCA invoca este módulo: solo el comando Tauri
//!   `execute_powershell` (autorización explícita del usuario) lo hace.
//!
//! Este módulo NO pretende ser una sandbox: PowerShell se ejecuta con los
//! permisos del usuario. El riesgo se mitiga con confirmación de comandos de
//! alto impacto (ver `powershell_reference`), no con un aislamiento falso.

use std::io::Read;
use std::path::PathBuf;
use std::process::{Child, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};

/// Timeout por defecto de una ejecución (30 s).
///
/// La arquitectura admite modificarlo por ejecución (p. ej. desde tests o
/// una futura preferencia en Ajustes) pasando otro valor a [`execute`].
pub const DEFAULT_TIMEOUT_MS: u64 = 30_000;

/// Límite de longitud del comando para evitar abusos evidentes.
pub const MAX_COMMAND_LEN: usize = 64 * 1024;

/// Mensaje cuando no se encuentra PowerShell.
pub const UNAVAILABLE_MSG: &str =
    "PowerShell no está disponible en este sistema: no se encontró powershell.exe.";

/// Resultado de una ejecución de PowerShell.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PowerShellResult {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: Option<i32>,
    pub duration_ms: u64,
    pub timed_out: bool,
    pub cancelled: bool,
    pub command: String,
}

/// Ejecución de PowerShell activa (si la hay), compartida entre el ejecutor y
/// el comando de cancelación.
#[derive(Default)]
pub struct ActivePowerShell {
    pub cancel: Arc<AtomicBool>,
    pub child: Arc<Mutex<Option<Child>>>,
}

/// Estado global del módulo PowerShell.
#[derive(Clone, Default)]
pub struct PowerShellManager {
    /// Ejecución activa (una a la vez, como el escáner).
    pub active: Arc<Mutex<Option<ActivePowerShell>>>,
}

/// Resuelve la ruta de `powershell.exe` de forma robusta: primero las
/// ubicaciones estándar de Windows y después una búsqueda en `PATH`. No asume
/// una ruta personalizada ni la cadena `cmd.exe`.
pub fn resolve_powershell() -> Option<PathBuf> {
    let system_root = std::env::var_os("SystemRoot")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(r"C:\Windows"));

    let standard = [
        system_root.join(r"System32\WindowsPowerShell\v1.0\powershell.exe"),
        system_root.join(r"Sysnative\WindowsPowerShell\v1.0\powershell.exe"),
        system_root.join(r"SysWOW64\WindowsPowerShell\v1.0\powershell.exe"),
    ];
    for candidate in standard {
        if candidate.is_file() {
            return Some(candidate);
        }
    }

    if let Some(path_var) = std::env::var_os("PATH") {
        for dir in std::env::split_paths(&path_var) {
            let candidate = dir.join("powershell.exe");
            if candidate.is_file() {
                return Some(candidate);
            }
        }
    }

    None
}

/// Ejecuta `command` en Windows PowerShell con un timeout.
///
/// `Err` se reserva para fallos técnicos (PowerShell no disponible o proceso
/// que no pudo iniciarse); el resto de condiciones (salida, error, timeout,
/// cancelación) se devuelven en [`PowerShellResult`] con sus banderas.
pub fn execute(
    manager: &PowerShellManager,
    command: &str,
    timeout_ms: u64,
) -> Result<PowerShellResult, String> {
    let started = Instant::now();

    let exe = resolve_powershell().ok_or_else(|| UNAVAILABLE_MSG.to_string())?;

    // Solo una ejecución a la vez.
    {
        let mut active = lock_opt(&manager.active);
        if active.is_some() {
            return Err("Ya hay un comando de PowerShell en ejecución".into());
        }
        let cancel = Arc::new(AtomicBool::new(false));
        *active = Some(ActivePowerShell {
            cancel: cancel.clone(),
            child: Arc::new(Mutex::new(None)),
        });
    }

    let spawn_result = std::process::Command::new(&exe)
        .args([
            "-NoLogo",
            "-NoProfile",
            "-NonInteractive",
            "-Command",
            command,
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn();

    let mut child = match spawn_result {
        Ok(child) => child,
        Err(e) => {
            clear_active(manager);
            return Err(format!("No se pudo iniciar PowerShell: {e}"));
        }
    };

    // Leemos stdout/stderr en hilos separados para evitar interbloqueos por
    // buffers llenos.
    let stdout_pipe = child.stdout.take().expect("stdout configurado como pipe");
    let stderr_pipe = child.stderr.take().expect("stderr configurado como pipe");
    let stdout_thread = std::thread::spawn(move || read_to_end_string(stdout_pipe));
    let stderr_thread = std::thread::spawn(move || read_to_end_string(stderr_pipe));

    let cancel = Arc::new(AtomicBool::new(false));
    let child_arc = Arc::new(Mutex::new(Some(child)));
    {
        let mut active = lock_opt(&manager.active);
        if let Some(a) = active.as_mut() {
            a.cancel = cancel.clone();
            a.child = child_arc.clone();
        }
    }

    let deadline = Instant::now() + Duration::from_millis(timeout_ms.max(100));
    let mut exit_code: Option<i32> = None;
    let mut timed_out = false;
    let mut cancelled = false;

    loop {
        if cancel.load(Ordering::Relaxed) {
            cancelled = true;
            terminate(&child_arc);
            break;
        }
        if Instant::now() >= deadline {
            timed_out = true;
            terminate(&child_arc);
            break;
        }
        let finished = {
            let mut guard = lock_opt(&child_arc);
            match guard.as_mut() {
                Some(c) => match c.try_wait() {
                    Ok(Some(status)) => {
                        exit_code = status.code();
                        *guard = None;
                        true
                    }
                    Ok(None) => false,
                    Err(_) => {
                        *guard = None;
                        true
                    }
                },
                None => true,
            }
        };
        if finished {
            break;
        }
        std::thread::sleep(Duration::from_millis(100));
    }

    // El proceso terminado cierra los pipes; los lectores acaban solos. Si
    // algún hijo heredó los descriptores y los mantiene abiertos, no esperamos
    // indefinidamente.
    let stdout = join_with_timeout(stdout_thread, Duration::from_millis(2_000));
    let stderr = join_with_timeout(stderr_thread, Duration::from_millis(2_000));

    // Asegurar el reap si se canceló/timereó.
    terminate(&child_arc);
    clear_active(manager);

    Ok(PowerShellResult {
        stdout,
        stderr,
        exit_code,
        duration_ms: started.elapsed().as_millis() as u64,
        timed_out,
        cancelled,
        command: command.to_string(),
    })
}

/// Cancela la ejecución activa (si la hay). Devuelve `true` si había algo que
/// cancelar.
pub fn cancel(manager: &PowerShellManager) -> bool {
    let active = lock_opt(&manager.active);
    let Some(a) = active.as_ref() else {
        return false;
    };
    a.cancel.store(true, Ordering::Relaxed);
    terminate(&a.child);
    true
}

fn clear_active(manager: &PowerShellManager) {
    *lock_opt(&manager.active) = None;
}

fn terminate(child_arc: &Arc<Mutex<Option<Child>>>) {
    let mut guard = lock_opt(child_arc);
    if let Some(child) = guard.as_mut() {
        let _ = child.kill();
        let _ = child.wait();
    }
}

fn read_to_end_string(mut reader: impl Read) -> String {
    let mut buf = Vec::new();
    let _ = reader.read_to_end(&mut buf);
    decode_output(buf)
}

fn join_with_timeout(handle: std::thread::JoinHandle<String>, timeout: Duration) -> String {
    let deadline = Instant::now() + timeout;
    while !handle.is_finished() {
        if Instant::now() >= deadline {
            return String::new();
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    handle.join().unwrap_or_default()
}

/// Decodifica la salida de PowerShell.
///
/// PowerShell 5.1 escribe en la página de códigos del sistema cuando se
/// redirige; si los bytes no son UTF-8 válidos se interpretan como la página
/// 1252 (aproximación correcta para textos latinos).
fn decode_output(bytes: Vec<u8>) -> String {
    match String::from_utf8(bytes.clone()) {
        Ok(s) => s,
        Err(_) => bytes.iter().map(|&b| decode_byte(b)).collect(),
    }
}

fn decode_byte(b: u8) -> char {
    match b {
        0x80..=0x9F => CP1252[b as usize - 0x80] as char,
        _ => b as char,
    }
}

const CP1252: [char; 32] = [
    '\u{20AC}', '\u{FFFD}', '\u{201A}', '\u{0192}', '\u{201E}', '\u{2026}', '\u{2020}', '\u{2021}',
    '\u{02C6}', '\u{2030}', '\u{0160}', '\u{2039}', '\u{0152}', '\u{FFFD}', '\u{017D}', '\u{FFFD}',
    '\u{FFFD}', '\u{2018}', '\u{2019}', '\u{201C}', '\u{201D}', '\u{2022}', '\u{2013}', '\u{2014}',
    '\u{02DC}', '\u{2122}', '\u{0161}', '\u{203A}', '\u{0153}', '\u{FFFD}', '\u{017E}', '\u{0178}',
];

fn lock_opt<T>(m: &Mutex<T>) -> std::sync::MutexGuard<'_, T> {
    m.lock().unwrap_or_else(|p| p.into_inner())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn exec(command: &str, timeout_ms: u64) -> PowerShellResult {
        let manager = PowerShellManager::default();
        execute(&manager, command, timeout_ms).expect("la ejecución debe lanzarse")
    }

    #[test]
    fn resolves_powershell_on_windows() {
        assert!(
            resolve_powershell().is_some(),
            "powershell.exe debe resolverse en una instalación de Windows normal"
        );
    }

    #[test]
    fn unavailable_when_command_missing() {
        let manager = PowerShellManager::default();
        // Timeout mínimo: no importa, falla antes de lanzar el proceso.
        let exe = resolve_powershell();
        if exe.is_none() {
            let r = execute(&manager, "Get-Date", 100);
            assert!(r.is_err());
        }
    }

    #[test]
    fn get_date_outputs() {
        let r = exec("Get-Date", 20_000);
        assert!(!r.timed_out, "no debe agotar el timeout");
        assert!(!r.cancelled);
        assert!(!r.stdout.trim().is_empty(), "debe devolver la fecha");
        assert_eq!(r.exit_code, Some(0));
    }

    #[test]
    fn get_process_outputs() {
        let r = exec("Get-Process", 20_000);
        assert!(!r.timed_out);
        assert!(!r.stdout.trim().is_empty(), "debe devolver procesos");
    }

    #[test]
    fn unknown_command_reports_error() {
        let r = exec("ThisCommandDoesNotExist12345", 20_000);
        assert!(
            !r.stderr.trim().is_empty() || r.exit_code != Some(0),
            "un comando inexistente debe producir error o código distinto de 0"
        );
    }

    #[test]
    fn timeout_is_respected() {
        let started = Instant::now();
        let r = exec("Start-Sleep -Seconds 30", 800);
        assert!(r.timed_out, "debe marcarse el timeout");
        assert!(started.elapsed() < Duration::from_secs(15), "no debe bloquear 30 s");
    }

    #[test]
    fn cancel_terminates_execution() {
        let manager = PowerShellManager::default();
        let runner = manager.clone();
        let handle = std::thread::spawn(move || {
            execute(&runner, "Start-Sleep -Seconds 30", 60_000).expect("ejecución en marcha")
        });
        std::thread::sleep(Duration::from_millis(700));
        assert!(cancel(&manager), "debe haber una ejecución activa que cancelar");
        let r = handle.join().expect("el hilo debe terminar");
        assert!(r.cancelled, "la ejecución debe marcarse cancelada");
    }

    #[test]
    fn only_one_execution_at_a_time() {
        let manager = PowerShellManager::default();
        let runner = manager.clone();
        let handle = std::thread::spawn(move || execute(&runner, "Start-Sleep -Seconds 5", 30_000));
        std::thread::sleep(Duration::from_millis(400));
        let second = execute(&manager, "Get-Date", 5_000);
        assert!(second.is_err(), "no debe permitirse una segunda ejecución simultánea");
        cancel(&manager);
        let _ = handle.join();
    }

    #[test]
    fn decodes_plain_ascii() {
        assert_eq!(decode_output(b"hello".to_vec()), "hello");
    }
}
