import {
  createContext,
  useCallback,
  useContext,
  useEffect,
  useMemo,
  useState,
  type ReactNode,
} from "react";
import { dictionaries, translate, type Dictionary, type TranslationKey } from "../lib/i18n";
import type { Language } from "../types";
import { useConfig } from "./ConfigContext";

const STORAGE_KEY = "va-language";

interface LanguageContextValue {
  language: Language;
  setLanguage: (lang: Language) => void;
  t: (key: TranslationKey) => string;
  dict: Dictionary;
}

const LanguageContext = createContext<LanguageContextValue | undefined>(undefined);

function readStored(): Language | null {
  const stored = localStorage.getItem(STORAGE_KEY);
  return stored === "es" || stored === "en" ? stored : null;
}

export function LanguageProvider({ children }: { children: ReactNode }) {
  const { config, updateConfig } = useConfig();

  const [language, setLanguageState] = useState<Language>(() => readStored() ?? "es");

  const setLanguage = useCallback(
    (lang: Language) => {
      setLanguageState(lang);
      localStorage.setItem(STORAGE_KEY, lang);
      void updateConfig({ language: lang });
    },
    [updateConfig],
  );

  useEffect(() => {
    document.documentElement.lang = language === "es" ? "es" : "en";
  }, [language]);

  // Sincroniza el idioma guardado en el backend si no hay preferencia local.
  useEffect(() => {
    if (config.language && readStored() === null) {
      setLanguageState(config.language);
    }
  }, [config.language]);

  const dict = dictionaries[language];
  const t = useCallback((key: TranslationKey) => translate(dict, key), [dict]);

  const value = useMemo(
    () => ({ language, setLanguage, t, dict }),
    [language, setLanguage, t, dict],
  );

  return <LanguageContext.Provider value={value}>{children}</LanguageContext.Provider>;
}

export function useLanguage(): LanguageContextValue {
  const ctx = useContext(LanguageContext);
  if (!ctx) throw new Error("useLanguage debe usarse dentro de <LanguageProvider>");
  return ctx;
}
