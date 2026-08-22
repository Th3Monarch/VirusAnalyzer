// Tipos del assistant AI companion. Mantener sincronizado con
// src-tauri/src/assistant/commands.rs y src-tauri/src/assistant/intent.rs.

export type IntentType =
  | "analyze_file"
  | "get_analysis"
  | "open_history"
  | "open_quarantine"
  | "quarantine_file"
  | "restore_file"
  | "generate_report"
  | "query_virustotal"
  | "get_system_info"
  | "activate_ysmel"
  | "deactivate_ysmel"
  | "activate_fenix"
  | "deactivate_fenix"
  | "get_rules"
  | "general_conversation"
  | "unknown";

export interface Intent {
  type: IntentType;
  params?: Record<string, unknown>;
}

export interface AssistantMessage {
  id: string;
  role: "user" | "assistant" | "system";
  content: string;
  timestamp: string;
  intent?: string | null;
  requiresConfirmation: boolean;
}

export interface ResponseMetadata {
  toolResult?: unknown | null;
  confidence?: number | null;
  processingTimeMs: number;
}

export interface AssistantResponse {
  message: string;
  intent?: Intent | null;
  requiresConfirmation: boolean;
  pendingId?: string | null;
  metadata?: ResponseMetadata | null;
}

export interface ApplicationContext {
  currentPage: string;
  selectedFile?: string | null;
  currentAnalysisId?: string | null;
  historyCount: number;
  quarantineCount: number;
  scanActive: boolean;
  systemSummary?: string | null;
  ysmelActive: boolean;
  fenixActive: boolean;
  language: string;
  currentThreatLevel?: string | null;
}

export interface ModelInfo {
  provider: string;
  model: string;
  available: boolean;
}

export interface OllamaTestResult {
  connected: boolean;
  models: string[];
  error?: string | null;
}

export interface OllamaConfig {
  url: string;
  model: string;
  enabled: boolean;
}
