import { useEffect } from "react";
import { useLocation } from "react-router-dom";
import { useAssistant } from "../contexts/AssistantContext";
import { useLanguage } from "../contexts/LanguageContext";
import { tauri } from "../lib/tauri";

const PAGE_MAP: Record<string, string> = {
  "/": "dashboard",
  "/scan": "scan",
  "/results": "results",
  "/analysis": "analysis",
  "/quarantine": "quarantine",
  "/rules": "rules",
  "/system": "system",
  "/powershell": "powershell",
  "/ps-reference": "psReference",
  "/settings": "settings",
};

export function useAssistantSync() {
  const location = useLocation();
  const { messages, sendMessage } = useAssistant();
  const { language } = useLanguage();

  // Sync current page to backend context
  useEffect(() => {
    const page = PAGE_MAP[location.pathname] ?? "dashboard";
    void tauri.assistantSetContext("currentPage", page).catch(() => undefined);
  }, [location.pathname]);

  // Send welcome message on first open (no messages yet)
  useEffect(() => {
    if (messages.length === 0) {
      const welcome = language === "es"
        ? "¡Hola! Soy tu compañero de seguridad de Prometeo. Puedo ayudarte a analizar archivos, comprender resultados, gestionar la cuarentena y más. ¿Qué te gustaría hacer?"
        : "Hello! I'm your Prometeo security companion. I can help you analyze files, understand results, manage quarantine, and more. What would you like to do?";
      void sendMessage(welcome);
    }
  }, []); // Only on mount
}
