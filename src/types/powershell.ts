// Tipos del módulo PowerShell (mantener sincronizados con
// `src-tauri/src/powershell.rs` y `src-tauri/src/powershell_reference.rs`).

/** Riesgo educativo de un comando PowerShell. */
export type RiskLevel = "safe" | "low" | "medium" | "high";

/** Resultado de una ejecución de PowerShell. */
export interface PowerShellResult {
  stdout: string;
  stderr: string;
  exitCode: number | null;
  durationMs: number;
  timedOut: boolean;
  cancelled: boolean;
  command: string;
}

/** Ficha educativa de un comando de la referencia. */
export interface PsCommandInfo {
  name: string;
  /** Clave de categoría (traducida en la UI). */
  category: string;
  description: string;
  usage: string;
  example: string;
  risk: RiskLevel;
  warning?: string | null;
}
