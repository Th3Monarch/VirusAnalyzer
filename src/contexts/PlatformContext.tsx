import { createContext, useContext, useEffect, useState, type ReactNode } from "react";
import type { Platform } from "../types";
import { tauri } from "../lib/tauri";

interface PlatformState {
  platform: Platform | null;
  isWindows: boolean;
  isLinux: boolean;
  isMacos: boolean;
}

const PlatformContext = createContext<PlatformState>({
  platform: null,
  isWindows: false,
  isLinux: false,
  isMacos: false,
});

export function PlatformProvider({ children }: { children: ReactNode }) {
  const [state, setState] = useState<PlatformState>({
    platform: null,
    isWindows: false,
    isLinux: false,
    isMacos: false,
  });

  useEffect(() => {
    let alive = true;
    void tauri
      .getPlatform()
      .then((p) => {
        if (alive) {
          setState({
            platform: p,
            isWindows: p === "windows",
            isLinux: p === "linux",
            isMacos: p === "macos",
          });
        }
      })
      .catch(() => {
        // Fallback: asumir Windows para compatibilidad
        if (alive) {
          setState({ platform: "windows", isWindows: true, isLinux: false, isMacos: false });
        }
      });
    return () => {
      alive = false;
    };
  }, []);

  return <PlatformContext.Provider value={state}>{children}</PlatformContext.Provider>;
}

export function usePlatform(): PlatformState {
  return useContext(PlatformContext);
}
