import { invoke } from "@tauri-apps/api/core";
import type {
  AppConfig,
  AppInfo,
  FolderScanResult,
  PathInfo,
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

export interface TauriClient {
  getConfig(): Promise<AppConfig>;
  saveConfig(config: AppConfig): Promise<AppConfig>;
  getAppInfo(): Promise<AppInfo>;
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
}

export const tauri: TauriClient = {
  getConfig: () => invoke<AppConfig>("get_config"),
  saveConfig: (config) => invoke<AppConfig>("save_config", { config }),
  getAppInfo: () => invoke<AppInfo>("get_app_info"),
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
};
