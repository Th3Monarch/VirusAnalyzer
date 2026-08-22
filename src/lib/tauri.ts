import { invoke } from "@tauri-apps/api/core";
import type {
  AppConfig,
  AppInfo,
  FolderScanResult,
  PathInfo,
  Platform,
  PowerShellResult,
  PsCommandInfo,
  QuarantineEntry,
  QuarantineSummary,
  ReportFormat,
  RiskLevel,
  RuleInfo,
  ScanHistoryEntry,
  ScanResult,
  SystemInfo,
  ThreatLevel,
  VirusTotalResult,
} from "../types";
import type {
  ApplicationContext,
  AssistantMessage,
  AssistantResponse,
  ModelInfo,
  OllamaTestResult,
} from "../types/assistant";
import type { AccentInfo, VoiceConfig, VoiceHealth, VoiceInfo, VoiceRecordingState } from "../types/voice";

export interface TauriClient {
  getConfig(): Promise<AppConfig>;
  saveConfig(config: AppConfig): Promise<AppConfig>;
  getAppInfo(): Promise<AppInfo>;
  getPlatform(): Promise<Platform>;
  getSystemInfo(): Promise<SystemInfo>;
  getPathInfo(path: string): Promise<PathInfo>;
  scanPath(path: string): Promise<string>;
  cancelScan(): Promise<boolean>;
  getScanHistory(): Promise<ScanHistoryEntry[]>;
  getAnalysisById(id: string): Promise<ScanResult | FolderScanResult | null>;
  getRules(): Promise<RuleInfo[]>;
  checkVirusTotal(hash: string): Promise<VirusTotalResult>;
  quarantineFile(path: string, threatLevel: ThreatLevel, reason?: string): Promise<QuarantineEntry>;
  getQuarantine(): Promise<QuarantineSummary>;
  restoreQuarantined(id: string): Promise<QuarantineEntry>;
  deleteQuarantined(id: string): Promise<boolean>;
  exportReport(scanId: string, format: ReportFormat, path: string): Promise<string>;
  previewReport(scanId: string, format: ReportFormat): Promise<string>;
  executePowerShell(command: string, confirm?: boolean): Promise<PowerShellResult>;
  cancelPowerShell(): Promise<boolean>;
  classifyPowerShellCommand(command: string): Promise<RiskLevel>;
  getPowerShellReference(language: string): Promise<PsCommandInfo[]>;
  installContextMenu(label: string): Promise<boolean>;
  uninstallContextMenu(): Promise<boolean>;
  isContextMenuInstalled(): Promise<boolean>;
  takeLaunchPath(): Promise<string | null>;

  // Assistant
  assistantSendMessage(message: string, confirmed?: boolean, pendingId?: string, language?: string): Promise<AssistantResponse>;
  assistantGetHistory(): Promise<AssistantMessage[]>;
  assistantClearSession(): Promise<void>;
  assistantGetContext(): Promise<ApplicationContext>;
  assistantSetContext(key: string, value?: string): Promise<void>;
  assistantGetProviderInfo(): Promise<ModelInfo>;
  assistantCheckProviderHealth(): Promise<boolean>;
  assistantCancelPending(): Promise<void>;
  assistantSetSilentMode(enabled: boolean): Promise<boolean>;
  assistantGetSilentMode(): Promise<boolean>;
  assistantSetProvider(providerType: string): Promise<ModelInfo>;
  assistantUpdateOllama(url: string, model: string, enabled: boolean, temperature?: number, maxTokens?: number): Promise<ModelInfo>;
  assistantTestOllama(url: string): Promise<OllamaTestResult>;
  assistantGetVoiceState(): Promise<VoiceRecordingState>;
  assistantUpdateVoiceConfig(config: VoiceConfig): Promise<VoiceConfig>;
  assistantSynthesize(text: string): Promise<number[]>;
  assistantTranscribe(audio: number[]): Promise<string>;
  assistantVoiceHealth(): Promise<VoiceHealth>;
  assistantListVoices(language: string): Promise<VoiceInfo[]>;
  assistantGetAccentInfo(language: string): Promise<AccentInfo>;
}

export const tauri: TauriClient = {
  getConfig: () => invoke<AppConfig>("get_config"),
  saveConfig: (config) => invoke<AppConfig>("save_config", { config }),
  getAppInfo: () => invoke<AppInfo>("get_app_info"),
  getPlatform: () => invoke<Platform>("get_platform"),
  getSystemInfo: () => invoke<SystemInfo>("get_system_info"),
  getPathInfo: (path) => invoke<PathInfo>("get_path_info", { path }),
  scanPath: (path) => invoke<string>("scan_path", { path }),
  cancelScan: () => invoke<boolean>("cancel_scan"),
  getScanHistory: () => invoke<ScanHistoryEntry[]>("get_scan_history"),
  getAnalysisById: (id) => invoke<ScanResult | FolderScanResult | null>("get_analysis_by_id", { id }),
  getRules: () => invoke<RuleInfo[]>("get_rules"),
  checkVirusTotal: (hash) => invoke<VirusTotalResult>("virustotal_lookup", { hash }),
  quarantineFile: (path, threatLevel, reason) =>
    invoke<QuarantineEntry>("quarantine_file", { path, threatLevel, reason }),
  getQuarantine: () => invoke<QuarantineSummary>("get_quarantine"),
  restoreQuarantined: (id) => invoke<QuarantineEntry>("restore_quarantined", { id }),
  deleteQuarantined: (id) => invoke<boolean>("delete_quarantined", { id }),
  exportReport: (scanId, format, path) =>
    invoke<string>("export_report", { scanId, format, path }),
  previewReport: (scanId, format) =>
    invoke<string>("preview_report", { scanId, format }),
  executePowerShell: (command, confirm) =>
    invoke<PowerShellResult>("execute_powershell", { command, confirm }),
  cancelPowerShell: () => invoke<boolean>("cancel_powershell"),
  classifyPowerShellCommand: (command) =>
    invoke<RiskLevel>("classify_powershell_command", { command }),
  getPowerShellReference: (language) =>
    invoke<PsCommandInfo[]>("get_powershell_reference", { language }),
  installContextMenu: (label) => invoke<boolean>("install_context_menu", { label }),
  uninstallContextMenu: () => invoke<boolean>("uninstall_context_menu"),
  isContextMenuInstalled: () => invoke<boolean>("is_context_menu_installed"),
  takeLaunchPath: () => invoke<string | null>("take_launch_path"),

  // Assistant
  assistantSendMessage: (message: string, confirmed?: boolean, pendingId?: string, language?: string) =>
    invoke<AssistantResponse>("assistant_send_message", { message, confirmed, pendingId, language: language ?? null }),
  assistantGetHistory: () => invoke<AssistantMessage[]>("assistant_get_history"),
  assistantClearSession: () => invoke<void>("assistant_clear_session"),
  assistantGetContext: () => invoke<ApplicationContext>("assistant_get_context"),
  assistantSetContext: (key, value) =>
    invoke<void>("assistant_set_context", { key, value }),
  assistantGetProviderInfo: () => invoke<ModelInfo>("assistant_get_provider_info"),
  assistantCheckProviderHealth: () => invoke<boolean>("assistant_check_provider_health"),
  assistantCancelPending: () => invoke<void>("assistant_cancel_pending"),
  assistantSetSilentMode: (enabled) => invoke<boolean>("assistant_set_silent_mode", { enabled }),
  assistantGetSilentMode: () => invoke<boolean>("assistant_get_silent_mode"),
  assistantSetProvider: (providerType) =>
    invoke<ModelInfo>("assistant_set_provider", { providerType }),
  assistantUpdateOllama: (url, model, enabled, temperature?, maxTokens?) =>
    invoke<ModelInfo>("assistant_update_ollama", { url, model, enabled, temperature: temperature ?? null, maxTokens: maxTokens ?? null }),
  assistantTestOllama: (url) =>
    invoke<OllamaTestResult>("assistant_test_ollama", { url }),
  assistantGetVoiceState: () =>
    invoke<VoiceRecordingState>("assistant_get_voice_state"),
  assistantUpdateVoiceConfig: (config) =>
    invoke<VoiceConfig>("assistant_update_voice_config", { config }),
  assistantSynthesize: (text) =>
    invoke<number[]>("assistant_synthesize", { text }),
  assistantTranscribe: (audio) =>
    invoke<string>("assistant_transcribe", { audio }),
  assistantVoiceHealth: () =>
    invoke<VoiceHealth>("assistant_voice_health"),
  assistantListVoices: (language) =>
    invoke<VoiceInfo[]>("assistant_list_voices", { language }),
  assistantGetAccentInfo: (language) =>
    invoke<AccentInfo>("assistant_get_accent_info", { language }),
};
