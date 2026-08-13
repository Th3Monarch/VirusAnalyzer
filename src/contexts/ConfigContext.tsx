import {
  createContext,
  useCallback,
  useContext,
  useEffect,
  useMemo,
  useState,
  type ReactNode,
} from "react";
import type { AppConfig } from "../types";
import { DEFAULT_CONFIG } from "../lib/defaults";
import { tauri } from "../lib/tauri";

interface ConfigContextValue {
  /** Configuración cargada del backend (o valores por defecto). */
  config: AppConfig;
  /** `true` cuando el backend ya respondió o falló. */
  loaded: boolean;
  error: string | null;
  /** Combina y persiste un cambio parcial de configuración. */
  updateConfig: (partial: Partial<AppConfig>) => Promise<void>;
  reload: () => Promise<void>;
}

const ConfigContext = createContext<ConfigContextValue | undefined>(undefined);

export function ConfigProvider({ children }: { children: ReactNode }) {
  const [config, setConfig] = useState<AppConfig>(DEFAULT_CONFIG);
  const [loaded, setLoaded] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const reload = useCallback(async () => {
    try {
      const remote = await tauri.getConfig();
      setConfig(remote);
      setError(null);
    } catch (e) {
      // En navegador (npm run dev sin Tauri) el comando falla; se usa el default.
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setLoaded(true);
    }
  }, []);

  const updateConfig = useCallback(
    async (partial: Partial<AppConfig>) => {
      const next = { ...config, ...partial };
      setConfig(next);
      try {
        const saved = await tauri.saveConfig(next);
        setConfig(saved);
        setError(null);
      } catch (e) {
        setError(e instanceof Error ? e.message : String(e));
      }
    },
    [config],
  );

  useEffect(() => {
    void reload();
  }, [reload]);

  const value = useMemo(
    () => ({ config, loaded, error, updateConfig, reload }),
    [config, loaded, error, updateConfig, reload],
  );

  return <ConfigContext.Provider value={value}>{children}</ConfigContext.Provider>;
}

export function useConfig(): ConfigContextValue {
  const ctx = useContext(ConfigContext);
  if (!ctx) throw new Error("useConfig debe usarse dentro de <ConfigProvider>");
  return ctx;
}
