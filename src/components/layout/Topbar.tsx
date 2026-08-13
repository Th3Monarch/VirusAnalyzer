import { useLocation } from "react-router-dom";
import { Languages, Moon, Sun } from "lucide-react";
import { useLanguage } from "../../contexts/LanguageContext";
import { useTheme } from "../../contexts/ThemeContext";
import { useConfig } from "../../contexts/ConfigContext";
import type { Language } from "../../types";
import type { TranslationKey } from "../../lib/i18n";

const TITLES: Record<string, TranslationKey> = {
  "/": "nav.dashboard",
  "/scan": "nav.scan",
  "/results": "nav.results",
  "/analysis": "nav.analysisDetail",
  "/quarantine": "nav.quarantine",
  "/rules": "nav.rules",
  "/system": "nav.system",
  "/powershell": "nav.powershell",
  "/settings": "nav.settings",
};

const THEME_ORDER: readonly ("dark" | "light" | "system")[] = ["dark", "light", "system"];

export function Topbar() {
  const location = useLocation();
  const { t, language, setLanguage } = useLanguage();
  const { theme, resolvedTheme, setTheme } = useTheme();
  const { config, loaded } = useConfig();

  const titleKey = TITLES[location.pathname] ?? "nav.dashboard";
  const hasVtKey = Boolean(config.virustotalApiKey);

  const cycleTheme = () => {
    const next = THEME_ORDER[(THEME_ORDER.indexOf(theme) + 1) % THEME_ORDER.length];
    setTheme(next);
  };

  const toggleLanguage = () => {
    const next: Language = language === "es" ? "en" : "es";
    setLanguage(next);
  };

  return (
    <header className="flex h-14 shrink-0 items-center justify-between border-b border-line bg-surface/70 px-6 backdrop-blur">
      <h2 className="text-sm font-semibold text-ink">{t(titleKey)}</h2>

      <div className="flex items-center gap-2">
        <span
          className={`inline-flex items-center gap-1.5 rounded-full border px-2.5 py-1 text-[11px] font-medium ${
            hasVtKey
              ? "border-good/30 bg-good/10 text-good"
              : "border-line bg-surface-2 text-muted"
          }`}
          title={hasVtKey ? t("dashboard.vtAvailable") : t("dashboard.vtMissing")}
        >
          <span className={`size-1.5 rounded-full ${hasVtKey ? "bg-good" : "bg-muted"}`} />
          VirusTotal
        </span>

        <button
          type="button"
          onClick={toggleLanguage}
          aria-label={language === "es" ? "Switch to English" : "Cambiar a español"}
          className="inline-flex items-center gap-1.5 rounded-lg border border-line bg-surface-2 px-2.5 py-1.5 text-xs font-medium text-ink transition-colors hover:border-muted/50 focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2"
          title={language === "es" ? "Switch to English" : "Cambiar a español"}
        >
          <Languages className="size-3.5" />
          {language === "es" ? "ES" : "EN"}
        </button>

        <button
          type="button"
          onClick={cycleTheme}
          aria-label={theme === "dark" ? "Cambiar a claro" : theme === "light" ? "Cambiar a oscuro" : "Tema del sistema"}
          className="inline-flex items-center gap-1.5 rounded-lg border border-line bg-surface-2 px-2.5 py-1.5 text-xs font-medium text-ink transition-colors hover:border-muted/50 focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2"
          title={theme === "dark" ? "Cambiar a claro" : theme === "light" ? "Cambiar a oscuro" : "Tema del sistema"}
        >
          {resolvedTheme === "dark" ? <Moon className="size-3.5" /> : <Sun className="size-3.5" />}
        </button>

        {loaded ? <span className="ml-1 text-[11px] text-muted">v{config.version}</span> : null}
      </div>
    </header>
  );
}
