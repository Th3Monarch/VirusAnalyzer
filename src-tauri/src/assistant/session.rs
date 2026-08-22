use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Mensaje en la sesión de conversación.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionMessage {
    pub id: String,
    /// "user" | "assistant" | "system"
    pub role: String,
    pub content: String,
    pub timestamp: String,
    /// Intención detectada (solo en mensajes del assistant).
    pub intent: Option<String>,
    /// Si requiere confirmación del usuario.
    pub requires_confirmation: bool,
}

/// Estado de la sesión de conversación.
#[derive(Debug)]
pub struct SessionContext {
    messages: Vec<SessionMessage>,
    max_messages: usize,
}

impl SessionContext {
    /// Crea una sesión vacía con un límite de 50 mensajes.
    pub fn new() -> Self {
        Self {
            messages: Vec::new(),
            max_messages: 50,
        }
    }

    /// Agrega un mensaje del usuario.
    /// Deduplicación: si el contenido es idéntico al último mensaje del mismo rol, lo omite.
    pub fn add_user_message(&mut self, content: &str) -> &SessionMessage {
        let is_dup = self
            .messages
            .last()
            .is_some_and(|m| m.role == "user" && m.content == content);
        if is_dup {
            return self.messages.last().expect("checked last above");
        }
        let msg = SessionMessage {
            id: Uuid::new_v4().to_string(),
            role: "user".into(),
            content: content.to_string(),
            timestamp: Utc::now().to_rfc3339(),
            intent: None,
            requires_confirmation: false,
        };
        self.messages.push(msg);
        self.trim_to_limit();
        self.messages.last().expect("message was just pushed")
    }

    /// Agrega un mensaje del assistant.
    /// Deduplicación: si el contenido es idéntico al último mensaje del mismo rol, lo omite.
    pub fn add_assistant_message(
        &mut self,
        content: &str,
        intent: Option<String>,
        requires_confirmation: bool,
    ) -> &SessionMessage {
        let is_dup = self
            .messages
            .last()
            .is_some_and(|m| m.role == "assistant" && m.content == content);
        if is_dup {
            return self.messages.last().expect("checked last above");
        }
        let msg = SessionMessage {
            id: Uuid::new_v4().to_string(),
            role: "assistant".into(),
            content: content.to_string(),
            timestamp: Utc::now().to_rfc3339(),
            intent,
            requires_confirmation,
        };
        self.messages.push(msg);
        self.trim_to_limit();
        self.messages.last().expect("message was just pushed")
    }

    /// Devuelve los últimos N mensajes para contexto del LLM.
    pub fn context_window(&self, n: usize) -> Vec<String> {
        self.messages
            .iter()
            .rev()
            .take(n)
            .rev()
            .map(|m| format!("[{}]: {}", m.role, m.content))
            .collect()
    }

    /// Devuelve todos los mensajes (para el frontend).
    pub fn all_messages(&self) -> &[SessionMessage] {
        &self.messages
    }

    /// Limpia la sesión.
    pub fn clear(&mut self) {
        self.messages.clear();
    }

    /// Número de mensajes en la sesión.
    #[allow(dead_code)]
    pub fn len(&self) -> usize {
        self.messages.len()
    }

    /// Si la sesión está vacía.
    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.messages.is_empty()
    }

    /// Mantiene solo los últimos `max_messages`.
    fn trim_to_limit(&mut self) {
        if self.messages.len() > self.max_messages {
            let drain = self.messages.len() - self.max_messages;
            self.messages.drain(..drain);
        }
    }
}

impl Default for SessionContext {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_context() {
        let mut session = SessionContext::new();
        session.add_user_message("hola");
        session.add_assistant_message("¡Hola!", None, false);
        assert_eq!(session.len(), 2);
        assert!(!session.is_empty());
    }

    #[test]
    fn test_context_window() {
        let mut session = SessionContext::new();
        for i in 0..10 {
            session.add_user_message(&format!("msg {i}"));
        }
        let window = session.context_window(3);
        assert_eq!(window.len(), 3);
        assert!(window[0].contains("msg 7"));
    }

    #[test]
    fn test_trim() {
        let mut session = SessionContext::new();
        session.max_messages = 5;
        for i in 0..10 {
            session.add_user_message(&format!("msg {i}"));
        }
        assert_eq!(session.len(), 5);
    }

    #[test]
    fn test_clear() {
        let mut session = SessionContext::new();
        session.add_user_message("hola");
        session.add_assistant_message("¡Hola!", None, false);
        assert_eq!(session.len(), 2);
        session.clear();
        assert_eq!(session.len(), 0);
        assert!(session.is_empty());
    }

    #[test]
    fn test_last_message() {
        let mut session = SessionContext::new();
        session.add_user_message("first");
        session.add_assistant_message("second", None, false);
        let last = session.messages.last().unwrap();
        assert_eq!(last.role, "assistant");
        assert_eq!(last.content, "second");
    }

    #[test]
    fn test_last_message_empty() {
        let session = SessionContext::new();
        assert!(session.messages.last().is_none());
    }

    #[test]
    fn test_context_window_full() {
        let mut session = SessionContext::new();
        session.add_user_message("first");
        let window = session.context_window(10);
        assert_eq!(window.len(), 1);
        assert!(window[0].contains("first"));
    }

    #[test]
    fn test_context_window_empty() {
        let session = SessionContext::new();
        let window = session.context_window(5);
        assert!(window.is_empty());
    }

    #[test]
    fn test_user_assistant_alternation() {
        let mut session = SessionContext::new();
        session.add_user_message("q1");
        session.add_assistant_message("a1", None, false);
        session.add_user_message("q2");
        session.add_assistant_message("a2", None, false);
        assert_eq!(session.messages[0].role, "user");
        assert_eq!(session.messages[1].role, "assistant");
        assert_eq!(session.messages[2].role, "user");
        assert_eq!(session.messages[3].role, "assistant");
    }

    #[test]
    fn test_dedup_consecutive_user() {
        let mut session = SessionContext::new();
        session.add_user_message("hello");
        session.add_user_message("hello");
        assert_eq!(session.len(), 1);
    }

    #[test]
    fn test_dedup_consecutive_assistant() {
        let mut session = SessionContext::new();
        session.add_assistant_message("response", None, false);
        session.add_assistant_message("response", None, false);
        assert_eq!(session.len(), 1);
    }

    #[test]
    fn test_no_dedup_different_content() {
        let mut session = SessionContext::new();
        session.add_user_message("hello");
        session.add_user_message("world");
        assert_eq!(session.len(), 2);
    }

    #[test]
    fn test_no_dedup_different_roles() {
        let mut session = SessionContext::new();
        session.add_user_message("hello");
        session.add_assistant_message("hello", None, false);
        assert_eq!(session.len(), 2);
    }

    #[test]
    fn test_no_dedup_alternating_same_content() {
        let mut session = SessionContext::new();
        session.add_user_message("test");
        session.add_assistant_message("test", None, false);
        session.add_user_message("test");
        assert_eq!(session.len(), 3);
    }
}
