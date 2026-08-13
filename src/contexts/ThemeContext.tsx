import {
  createContext,
  useCallback,
  useContext,
  useEffect,
  useMemo,
  useState,
  type ReactNode,
} from "react";
import type { ThemePreference } from "../types";
import { useConfig } from "./ConfigContext";

const STORAGE_KEY = "va-theme";

interface ThemeContextValue {
  theme: ThemePreference;
  /** Tema efectivo tras resolver "system". */
  resolvedTheme: "dark" | "light";
  setTheme: (theme: ThemePreference) => void;
}

const ThemeContext = createContext<ThemeContextValue | undefined>(undefined);

function resolveSystemTheme(): "dark" | "light" {
  return window.matchMedia("(prefers-color-scheme: dark)").matches ? "dark" : "light";
}

function readStored(): ThemePreference | null {
  const stored = localStorage.getItem(STORAGE_KEY);
  return stored === "dark" || stored === "light" || stored === "system" ? stored : null;
}

export function ThemeProvider({ children }: { children: ReactNode }) {
  const { config, updateConfig } = useConfig();

  const [theme, setThemeState] = useState<ThemePreference>(() => readStored() ?? "dark");
  const [systemTheme, setSystemTheme] = useState<"dark" | "light">(resolveSystemTheme);

  const resolvedTheme: "dark" | "light" =
    theme === "system" ? systemTheme : theme;

  const apply = useCallback(
    (t: ThemePreference) => {
      const resolved = t === "system" ? resolveSystemTheme() : t;
      document.documentElement.classList.toggle("dark", resolved === "dark");
    },
    [],
  );

  const setTheme = useCallback(
    (t: ThemePreference) => {
      setThemeState(t);
      localStorage.setItem(STORAGE_KEY, t);
      apply(t);
      void updateConfig({ theme: t });
    },
    [apply, updateConfig],
  );

  // Sincroniza el tema guardado en el backend al cargar la configuración.
  useEffect(() => {
    if (config.theme && readStored() === null) {
      setThemeState(config.theme);
    }
    apply(theme);
  }, [config.theme, theme, apply]);

  // Escucha cambios del sistema cuando el tema es "system".
  useEffect(() => {
    const mql = window.matchMedia("(prefers-color-scheme: dark)");
    const onChange = () => setSystemTheme(mql.matches ? "dark" : "light");
    mql.addEventListener("change", onChange);
    return () => mql.removeEventListener("change", onChange);
  }, []);

  const value = useMemo(
    () => ({ theme, resolvedTheme, setTheme }),
    [theme, resolvedTheme, setTheme],
  );

  return <ThemeContext.Provider value={value}>{children}</ThemeContext.Provider>;
}

export function useTheme(): ThemeContextValue {
  const ctx = useContext(ThemeContext);
  if (!ctx) throw new Error("useTheme debe usarse dentro de <ThemeProvider>");
  return ctx;
}
