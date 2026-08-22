import { useAssistant } from "../../contexts/AssistantContext";
import { useLanguage } from "../../contexts/LanguageContext";

interface Suggestion {
  key: string;
  labelKey: string;
  messageEs: string;
  messageEn: string;
}

const SUGGESTIONS: Suggestion[] = [
  { key: "scan", labelKey: "assistant.suggestions.scanFile", messageEs: "Quiero analizar un archivo", messageEn: "I want to analyze a file" },
  { key: "history", labelKey: "assistant.suggestions.viewHistory", messageEs: "Mostrar historial de análisis", messageEn: "Show scan history" },
  { key: "quarantine", labelKey: "assistant.suggestions.openQuarantine", messageEs: "Abrir cuarentena", messageEn: "Open quarantine" },
  { key: "rules", labelKey: "assistant.suggestions.explainRules", messageEs: "¿Qué reglas usa el motor heurístico?", messageEn: "What rules does the heuristic engine use?" },
  { key: "system", labelKey: "assistant.suggestions.systemInfo", messageEs: "Información del sistema", messageEn: "System information" },
  { key: "help", labelKey: "assistant.suggestions.help", messageEs: "¿Qué puedes hacer?", messageEn: "What can you do?" },
];

function getSuggestionMessage(s: Suggestion, lang: string): string {
  return lang === "es" ? s.messageEs : s.messageEn;
}

export function AssistantSuggestions() {
  const { sendMessage, isLoading } = useAssistant();
  const { t, language } = useLanguage();

  return (
    <div className="mb-4">
      <p className="mb-2 text-xs text-muted/60">{t("assistant.suggestionsTitle")}</p>
      <div className="flex flex-wrap gap-1.5">
        {SUGGESTIONS.map((s) => (
          <button
            key={s.key}
            onClick={() => void sendMessage(getSuggestionMessage(s, language))}
            disabled={isLoading}
            className="rounded-full border border-line bg-surface-2 px-3 py-1.5 text-[11px] text-muted transition-colors hover:border-accent/40 hover:bg-accent/10 hover:text-accent disabled:opacity-50"
          >
            {t(s.labelKey as any)}
          </button>
        ))}
      </div>
    </div>
  );
}
