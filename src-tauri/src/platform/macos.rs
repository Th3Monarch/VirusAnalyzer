//! Implementación macOS de [`TerminalManager`] y [`ContextMenuProvider`].
//!
//! Ejecuta comandos a través del shell del usuario (`$SHELL` o `/bin/zsh`
//! como fallback en macOS). No hay menú contextual integrado.

use std::io::Read;
use std::process::{Child, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use crate::models::Language;

use super::terminal::{RiskLevel, TerminalCommandInfo, TerminalManager, TerminalResult};
use super::ContextMenuProvider;

const DEFAULT_TIMEOUT_MS: u64 = 30_000;
const MAX_COMMAND_LEN: usize = 64 * 1024;
const FALLBACK_SHELL: &str = "/bin/zsh";

#[derive(Default)]
struct ActiveExecution {
    cancel: Arc<AtomicBool>,
    child: Arc<Mutex<Option<Child>>>,
}

/// Proveedor de terminal macOS: ejecuta comandos vía el shell del usuario.
pub struct MacOSTerminalManager {
    active: Arc<Mutex<Option<ActiveExecution>>>,
}

impl MacOSTerminalManager {
    pub fn new() -> Self {
        Self {
            active: Arc::new(Mutex::new(None)),
        }
    }

    fn resolve_shell() -> String {
        std::env::var("SHELL").unwrap_or_else(|_| FALLBACK_SHELL.to_string())
    }
}

impl Default for MacOSTerminalManager {
    fn default() -> Self {
        Self::new()
    }
}

impl TerminalManager for MacOSTerminalManager {
    fn execute(&self, command: &str, timeout_ms: u64) -> Result<TerminalResult, String> {
        let started = Instant::now();
        let shell = Self::resolve_shell();

        if command.len() > MAX_COMMAND_LEN {
            return Err("El comando es demasiado largo".into());
        }

        {
            let mut active = self.active.lock().unwrap_or_else(|p| p.into_inner());
            if active.is_some() {
                return Err("Ya hay un comando en ejecución".into());
            }
            *active = Some(ActiveExecution {
                cancel: Arc::new(AtomicBool::new(false)),
                child: Arc::new(Mutex::new(None)),
            });
        }

        let spawn_result = std::process::Command::new(&shell)
            .args(["-c", command])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn();

        let mut child = match spawn_result {
            Ok(c) => c,
            Err(e) => {
                *self.active.lock().unwrap_or_else(|p| p.into_inner()) = None;
                return Err(format!("No se pudo iniciar el shell: {e}"));
            }
        };

        let stdout_pipe = child.stdout.take().expect("stdout configurado como pipe");
        let stderr_pipe = child.stderr.take().expect("stderr configurado como pipe");
        let stdout_thread = std::thread::spawn(move || read_to_end(stdout_pipe));
        let stderr_thread = std::thread::spawn(move || read_to_end(stderr_pipe));

        let cancel = Arc::new(AtomicBool::new(false));
        let child_arc = Arc::new(Mutex::new(Some(child)));
        {
            let mut active = self.active.lock().unwrap_or_else(|p| p.into_inner());
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
                let mut guard = child_arc.lock().unwrap_or_else(|p| p.into_inner());
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

        let stdout = join_with_timeout(stdout_thread, Duration::from_millis(2_000));
        let stderr = join_with_timeout(stderr_thread, Duration::from_millis(2_000));

        terminate(&child_arc);
        *self.active.lock().unwrap_or_else(|p| p.into_inner()) = None;

        Ok(TerminalResult {
            stdout,
            stderr,
            exit_code,
            duration_ms: started.elapsed().as_millis() as u64,
            timed_out,
            cancelled,
            command: command.to_string(),
        })
    }

    fn cancel(&self) -> bool {
        let active = self.active.lock().unwrap_or_else(|p| p.into_inner());
        let Some(a) = active.as_ref() else {
            return false;
        };
        a.cancel.store(true, Ordering::Relaxed);
        terminate(&a.child);
        true
    }

    fn is_available(&self) -> bool {
        let shell = Self::resolve_shell();
        std::path::Path::new(&shell).exists()
    }

    fn classify(&self, command: &str) -> RiskLevel {
        classify_shell_command(command)
    }

    fn get_reference(&self, _language: Language) -> Vec<TerminalCommandInfo> {
        macos_shell_reference()
    }

    fn prompt_label(&self) -> &'static str {
        "$ "
    }

    fn display_name(&self) -> &'static str {
        "Terminal"
    }
}

/// Menú contextual no disponible en macOS (stub).
pub struct StubContextMenu;

impl ContextMenuProvider for StubContextMenu {
    fn install(&self, _label: &str) -> Result<(), String> {
        Err("El menú contextual no está disponible en macOS".into())
    }

    fn uninstall(&self) -> Result<(), String> {
        Err("El menú contextual no está disponible en macOS".into())
    }

    fn is_installed(&self) -> Result<bool, String> {
        Ok(false)
    }
}

// ---------------------------------------------------------------------------
// Helpers internos (idénticos a Linux; se podrían unificar en un módulo
// compartido, pero por ahora mantenemos la duplicación para claridad).
// ---------------------------------------------------------------------------

fn read_to_end(mut reader: impl Read) -> String {
    let mut buf = Vec::new();
    let _ = reader.read_to_end(&mut buf);
    String::from_utf8(buf).unwrap_or_else(|e| String::from_utf8_lossy(&e.into_bytes()).into_owned())
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

fn terminate(child_arc: &Arc<Mutex<Option<Child>>>) {
    let mut guard = child_arc.lock().unwrap_or_else(|p| p.into_inner());
    if let Some(child) = guard.as_mut() {
        let _ = child.kill();
        let _ = child.wait();
    }
}

/// Clasificación heurística de riesgo para comandos shell.
fn classify_shell_command(raw: &str) -> RiskLevel {
    let lower = raw.trim().to_lowercase();
    let tokens: Vec<&str> = lower
        .split(|c: char| !c.is_alphanumeric() && c != '-')
        .filter(|t| !t.is_empty())
        .collect();

    let has = |t: &str| tokens.iter().any(|tok| *tok == t);

    let high_tokens = [
        "rm", "mkfs", "dd", "format", "shred", "fdisk",
        "chmod", "chown", "chgrp", "sudo", "su",
        "reboot", "shutdown", "halt", "launchctl",
        "mount", "umount", "pfctl", "iptables",
        "crontab", "at", "dscl", "diskutil",
        "defaults", "brew", "pip",
    ];
    for t in &high_tokens {
        if has(t) {
            return RiskLevel::High;
        }
    }

    let has_pair = |a: &str, b: &str| {
        tokens.windows(2).any(|w| w[0] == a && w[1] == b)
    };
    if has_pair("rm", "-rf")
        || has_pair("rm", "-fr")
        || has_pair("sudo", "rm")
        || has_pair("sudo", "chmod")
        || has_pair("sudo", "kill")
    {
        return RiskLevel::High;
    }

    let medium_tokens = [
        "kill", "pkill", "killall", "launchctl",
        "brew", "npm", "node", "python",
        "tar", "zip", "unzip", "cp", "mv", "ln",
        "ifconfig", "networksetup", "scutil",
    ];
    for t in &medium_tokens {
        if has(t) {
            return RiskLevel::Medium;
        }
    }

    RiskLevel::Safe
}

/// Catálogo de comandos de shell para macOS.
fn macos_shell_reference() -> Vec<TerminalCommandInfo> {
    vec![
        TerminalCommandInfo {
            name: "ls".into(),
            category: "files".into(),
            description: "Lista archivos y directorios.".into(),
            usage: "ls [-la] [path]".into(),
            example: "ls -la ~/Desktop".into(),
            risk: RiskLevel::Safe,
            warning: None,
        },
        TerminalCommandInfo {
            name: "cat".into(),
            category: "files".into(),
            description: "Muestra el contenido de un archivo.".into(),
            usage: "cat <file>".into(),
            example: "cat /etc/hosts".into(),
            risk: RiskLevel::Safe,
            warning: None,
        },
        TerminalCommandInfo {
            name: "ps".into(),
            category: "processes".into(),
            description: "Muestra los procesos en ejecución.".into(),
            usage: "ps aux".into(),
            example: "ps aux | grep Finder".into(),
            risk: RiskLevel::Safe,
            warning: None,
        },
        TerminalCommandInfo {
            name: "top".into(),
            category: "processes".into(),
            description: "Monitor de procesos en tiempo real.".into(),
            usage: "top -l 1".into(),
            example: "top -l 1 | head -20".into(),
            risk: RiskLevel::Safe,
            warning: None,
        },
        TerminalCommandInfo {
            name: "df".into(),
            category: "system".into(),
            description: "Muestra el uso de disco.".into(),
            usage: "df [-h]".into(),
            example: "df -h".into(),
            risk: RiskLevel::Safe,
            warning: None,
        },
        TerminalCommandInfo {
            name: "sysctl".into(),
            category: "system".into(),
            description: "Muestra o modifica parámetros del kernel.".into(),
            usage: "sysctl <variable>".into(),
            example: "sysctl hw.ncpu".into(),
            risk: RiskLevel::Low,
            warning: None,
        },
        TerminalCommandInfo {
            name: "sw_vers".into(),
            category: "system".into(),
            description: "Muestra la versión de macOS.".into(),
            usage: "sw_vers".into(),
            example: "sw_vers".into(),
            risk: RiskLevel::Safe,
            warning: None,
        },
        TerminalCommandInfo {
            name: "diskutil".into(),
            category: "system".into(),
            description: "Gestiona discos y volúmenes.".into(),
            usage: "diskutil list".into(),
            example: "diskutil list".into(),
            risk: RiskLevel::Medium,
            warning: Some("Manipulación de discos: usa con precaución.".into()),
        },
        TerminalCommandInfo {
            name: "rm".into(),
            category: "files".into(),
            description: "Elimina archivos o directorios.".into(),
            usage: "rm [-rf] <path>".into(),
            example: "rm -rf /tmp/old".into(),
            risk: RiskLevel::High,
            warning: Some("Borra datos de forma permanente; no se puede deshacer.".into()),
        },
        TerminalCommandInfo {
            name: "sudo".into(),
            category: "security".into(),
            description: "Ejecuta un comando con privilegios de administrador.".into(),
            usage: "sudo <command>".into(),
            example: "sudo launchctl restart com.apple.Finder".into(),
            risk: RiskLevel::High,
            warning: Some("Ejecuta con privilegios elevados; comprueba el comando.".into()),
        },
        TerminalCommandInfo {
            name: "launchctl".into(),
            category: "services".into(),
            description: "Gestiona servicios del sistema (launchd).".into(),
            usage: "launchctl <verb> <service>".into(),
            example: "launchctl list | head -20".into(),
            risk: RiskLevel::Medium,
            warning: Some("Controla servicios del sistema; modifica con cuidado.".into()),
        },
        TerminalCommandInfo {
            name: "log".into(),
            category: "diagnostics".into(),
            description: "Consulta el registro unificado de macOS.".into(),
            usage: "log show --predicate <filter>".into(),
            example: "log show --predicate 'process == \"kernel\"' --last 5m".into(),
            risk: RiskLevel::Safe,
            warning: None,
        },
    ]
}
