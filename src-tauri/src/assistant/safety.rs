use super::tools::RiskLevel;
use std::collections::VecDeque;
use std::time::{Duration, Instant};

/// Nivel de permiso de una herramienta.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolPermission {
    /// Permitido sin confirmación.
    Allowed,
    /// Requiere confirmación explícita del usuario.
    RequiresConfirmation,
    /// Bloqueado completamente.
    Blocked,
}

/// Evento de seguridad para audit trail.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct SecurityEvent {
    pub timestamp: Instant,
    pub kind: SecurityEventKind,
    pub detail: String,
}

/// Tipos de eventos en el audit trail de seguridad.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SecurityEventKind {
    /// Acción bloqueada por protocolo.
    Blocked,
    /// Confirmación recibida y ejecutada.
    ConfirmationAccepted,
    /// Intento de bypass (pending_id inválido o intent mismatch).
    BypassAttempt,
    /// Rate limit alcanzado.
    RateLimited,
}

/// Capa de seguridad que controla qué tools puede ejecutar el assistant.
///
/// SafetyLayer es la ÚLTIMA puerta antes de ejecutar cualquier acción.
/// Independientemente de lo que el parser o el LLM generen, esta capa
/// decide si la acción procede.
///
/// Funcionalidades:
/// - Evalúa RiskLevel contra estado de protocolos (Ysmel/Fenix)
/// - Rate limiting por tipo de acción destructiva
/// - Audit trail de eventos de seguridad
pub struct SafetyLayer {
    /// Si el protocolo Ysmel está activo.
    ysmel_active: bool,
    /// Si el protocolo Fenix está activo.
    fenix_active: bool,
    /// Timestamps de acciones destructivas recientes (sliding window).
    destructive_actions: VecDeque<Instant>,
    /// Log de eventos de seguridad (últimos N eventos).
    audit_log: VecDeque<SecurityEvent>,
    /// Capacidad máxima del audit log.
    audit_capacity: usize,
}

/// Resultado de una verificación de seguridad.
pub struct SafetyCheck {
    pub permission: ToolPermission,
    pub events: Vec<SecurityEvent>,
}

impl SafetyLayer {
    /// Ventana de rate limiting: 60 segundos.
    const RATE_WINDOW: Duration = Duration::from_secs(60);
    /// Máximo de acciones destructivas por ventana.
    const MAX_DESTRUCTIVE_PER_WINDOW: usize = 5;
    /// Capacidad del audit log.
    const DEFAULT_AUDIT_CAPACITY: usize = 100;

    /// Crea una capa de seguridad con estado limpio.
    pub fn new() -> Self {
        Self {
            ysmel_active: false,
            fenix_active: false,
            destructive_actions: VecDeque::new(),
            audit_log: VecDeque::new(),
            audit_capacity: Self::DEFAULT_AUDIT_CAPACITY,
        }
    }

    /// Activa o desactiva el protocolo Ysmel.
    pub fn set_ysmel(&mut self, active: bool) {
        self.ysmel_active = active;
    }

    /// Activa o desactiva el protocolo Fenix.
    pub fn set_fenix(&mut self, active: bool) {
        self.fenix_active = active;
    }

    /// Evalúa el permiso dado un RiskLevel y el estado de Fenix,
    /// aplicando rate limiting para acciones destructivas.
    pub fn check_risk(&mut self, risk: RiskLevel) -> SafetyCheck {
        let mut events = Vec::new();

        // 1. Evaluar permiso base según risk level + Fenix
        let mut base_permission = match risk {
            RiskLevel::None => ToolPermission::Allowed,
            RiskLevel::Medium => ToolPermission::RequiresConfirmation,
            RiskLevel::High => {
                if self.fenix_active {
                    events.push(SecurityEvent {
                        timestamp: Instant::now(),
                        kind: SecurityEventKind::Blocked,
                        detail: "High-risk action blocked by Fenix protocol".into(),
                    });
                    ToolPermission::Blocked
                } else {
                    ToolPermission::RequiresConfirmation
                }
            }
            RiskLevel::Critical => {
                events.push(SecurityEvent {
                    timestamp: Instant::now(),
                    kind: SecurityEventKind::Blocked,
                    detail: "Critical-risk action always blocked".into(),
                });
                ToolPermission::Blocked
            }
        };

        // 2. Rate limiting para acciones que requieren confirmación
        if base_permission == ToolPermission::RequiresConfirmation {
            self.clean_old_actions();
            if self.destructive_actions.len() >= Self::MAX_DESTRUCTIVE_PER_WINDOW {
                events.push(SecurityEvent {
                    timestamp: Instant::now(),
                    kind: SecurityEventKind::RateLimited,
                    detail: format!(
                        "Rate limit: {} destructive actions in {}s window (max {}). Action blocked.",
                        self.destructive_actions.len(),
                        Self::RATE_WINDOW.as_secs(),
                        Self::MAX_DESTRUCTIVE_PER_WINDOW,
                    ),
                });
                base_permission = ToolPermission::Blocked;
            }
        }

        // 3. Registrar eventos en audit log
        for event in &events {
            self.push_audit(event.clone());
        }

        SafetyCheck {
            permission: base_permission,
            events,
        }
    }

    /// Registra que una acción destructiva fue ejecutada (para rate limiting).
    pub fn record_destructive_action(&mut self) {
        self.destructive_actions.push_back(Instant::now());
        self.clean_old_actions();
    }

    /// Registra un evento de seguridad en el audit log.
    pub fn push_audit(&mut self, event: SecurityEvent) {
        if self.audit_log.len() >= self.audit_capacity {
            self.audit_log.pop_front();
        }
        self.audit_log.push_back(event);
    }

    /// Registra un bypass attempt.
    pub fn record_bypass_attempt(&mut self, detail: &str) {
        self.push_audit(SecurityEvent {
            timestamp: Instant::now(),
            kind: SecurityEventKind::BypassAttempt,
            detail: detail.to_string(),
        });
    }

    /// Registra confirmación recibida.
    pub fn record_confirmation_accepted(&mut self) {
        self.push_audit(SecurityEvent {
            timestamp: Instant::now(),
            kind: SecurityEventKind::ConfirmationAccepted,
            detail: "User confirmed destructive action".into(),
        });
    }

    /// Limpia acciones fuera de la ventana de rate limiting.
    fn clean_old_actions(&mut self) {
        let cutoff = Instant::now() - Self::RATE_WINDOW;
        while self
            .destructive_actions
            .front()
            .map_or(false, |t| *t < cutoff)
        {
            self.destructive_actions.pop_front();
        }
    }
}

impl Default for SafetyLayer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_risk_none_is_allowed() {
        let mut safety = SafetyLayer::new();
        let check = safety.check_risk(RiskLevel::None);
        assert_eq!(check.permission, ToolPermission::Allowed);
    }

    #[test]
    fn test_risk_medium_requires_confirmation() {
        let mut safety = SafetyLayer::new();
        let check = safety.check_risk(RiskLevel::Medium);
        assert_eq!(check.permission, ToolPermission::RequiresConfirmation);
    }

    #[test]
    fn test_risk_high_blocked_in_fenix() {
        let mut safety = SafetyLayer::new();
        safety.set_fenix(true);
        let check = safety.check_risk(RiskLevel::High);
        assert_eq!(check.permission, ToolPermission::Blocked);
        assert!(!check.events.is_empty());
    }

    #[test]
    fn test_risk_high_requires_confirmation_without_fenix() {
        let mut safety = SafetyLayer::new();
        let check = safety.check_risk(RiskLevel::High);
        assert_eq!(check.permission, ToolPermission::RequiresConfirmation);
    }

    #[test]
    fn test_risk_critical_always_blocked() {
        let mut safety = SafetyLayer::new();
        let check = safety.check_risk(RiskLevel::Critical);
        assert_eq!(check.permission, ToolPermission::Blocked);
    }

    #[test]
    fn test_audit_log_capped() {
        let mut safety = SafetyLayer::new();
        safety.audit_capacity = 3;
        for i in 0..5 {
            safety.push_audit(SecurityEvent {
                timestamp: Instant::now(),
                kind: SecurityEventKind::Blocked,
                detail: format!("event {i}"),
            });
        }
        assert_eq!(safety.audit_log.len(), 3);
    }

    #[test]
    fn test_bypass_attempt_recorded() {
        let mut safety = SafetyLayer::new();
        safety.record_bypass_attempt("invalid pending_id");
        assert_eq!(safety.audit_log.len(), 1);
        assert_eq!(safety.audit_log[0].kind, SecurityEventKind::BypassAttempt);
    }

    #[test]
    fn test_record_confirmation_accepted() {
        let mut safety = SafetyLayer::new();
        safety.record_confirmation_accepted();
        assert_eq!(safety.audit_log.len(), 1);
        assert_eq!(safety.audit_log[0].kind, SecurityEventKind::ConfirmationAccepted);
    }

    #[test]
    fn test_record_destructive_action() {
        let mut safety = SafetyLayer::new();
        safety.record_destructive_action();
        assert_eq!(safety.destructive_actions.len(), 1);
    }

    #[test]
    fn test_rate_limit_blocks() {
        let mut safety = SafetyLayer::new();
        for _ in 0..6 {
            safety.record_destructive_action();
        }
        let check = safety.check_risk(RiskLevel::High);
        assert_eq!(check.permission, ToolPermission::Blocked);
        let rate_limited = check.events.iter().any(|e| e.kind == SecurityEventKind::RateLimited);
        assert!(rate_limited);
    }

    #[test]
    fn test_rate_limit_not_triggered_within_window() {
        let mut safety = SafetyLayer::new();
        for _ in 0..4 {
            safety.record_destructive_action();
        }
        let check = safety.check_risk(RiskLevel::High);
        assert_eq!(check.permission, ToolPermission::RequiresConfirmation);
    }

    #[test]
    fn test_ysmel_active_blocks_high() {
        let mut safety = SafetyLayer::new();
        safety.set_ysmel(true);
        let check = safety.check_risk(RiskLevel::High);
        assert_eq!(check.permission, ToolPermission::RequiresConfirmation);
    }

    #[test]
    fn test_ysmel_does_not_block_medium() {
        let mut safety = SafetyLayer::new();
        safety.set_ysmel(true);
        let check = safety.check_risk(RiskLevel::Medium);
        assert_eq!(check.permission, ToolPermission::RequiresConfirmation);
    }

    #[test]
    fn test_ysmel_and_fenix_combined() {
        let mut safety = SafetyLayer::new();
        safety.set_ysmel(true);
        safety.set_fenix(true);
        let check_high = safety.check_risk(RiskLevel::High);
        assert_eq!(check_high.permission, ToolPermission::Blocked);
        let check_medium = safety.check_risk(RiskLevel::Medium);
        assert_eq!(check_medium.permission, ToolPermission::RequiresConfirmation);
        let check_none = safety.check_risk(RiskLevel::None);
        assert_eq!(check_none.permission, ToolPermission::Allowed);
    }

    #[test]
    fn test_audit_log_fifo() {
        let mut safety = SafetyLayer::new();
        safety.audit_capacity = 3;
        for i in 0..5 {
            safety.push_audit(SecurityEvent {
                timestamp: Instant::now(),
                kind: SecurityEventKind::Blocked,
                detail: format!("event {i}"),
            });
        }
        assert_eq!(safety.audit_log.len(), 3);
        assert_eq!(safety.audit_log[0].detail, "event 2");
        assert_eq!(safety.audit_log[2].detail, "event 4");
    }

    #[test]
    fn test_no_events_on_allowed() {
        let mut safety = SafetyLayer::new();
        let check = safety.check_risk(RiskLevel::None);
        assert!(check.events.is_empty());
    }

    #[test]
    fn test_set_fenix_toggles() {
        let mut safety = SafetyLayer::new();
        assert!(!safety.fenix_active);
        safety.set_fenix(true);
        assert!(safety.fenix_active);
        safety.set_fenix(false);
        assert!(!safety.fenix_active);
    }
}
