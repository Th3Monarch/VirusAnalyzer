// Tipos base compartidos. Mantener sincronizado con src-tauri/src/models.rs
// y src-tauri/src/config/mod.rs (serialización camelCase).

export type ThreatLevel = "Clean" | "Low" | "Medium" | "High" | "Critical";

export type Severity = "info" | "low" | "medium" | "high" | "critical";

export type ThemePreference = "dark" | "light" | "system";
export type ReportFormat = "html" | "csv";

export type Language = "es" | "en";

export interface FileHashes {
  md5?: string | null;
  sha1?: string | null;
  sha256?: string | null;
}

export interface Finding {
  ruleName: string;
  category: string;
  severity: Severity;
  description: string;
  evidence: string[];
  points: number;
}

/** Análisis estático del archivo (FASE 3). */
export interface StaticAnalysis {
  fileType: string;
  fileTypeExtension: string;
  fileTypeMime: string;
  entropy: number;
  isPe: boolean;
  keywords: string[];
  typeMismatch: boolean;
  pe?: PeInfo | null;
}

export interface PeInfo {
  machine: string;
  architecture: string;
  isDll: boolean;
  isExecutable: boolean;
  isConsole: boolean;
  isGui: boolean;
  imageBase: number;
  entryPoint: number;
  timestamp: number;
  timestampIso: string;
  subsystem: string;
  dllCharacteristics: number;
  hasCertificate: boolean;
  certificateSize: number;
  sections: PeSection[];
  imports: PeImportDll[];
  importCount: number;
  exports: string[];
  exportCount: number;
}

export interface PeSection {
  name: string;
  virtualSize: number;
  virtualAddress: number;
  rawSize: number;
  entropy: number;
  flags: string[];
}

export interface PeImportDll {
  name: string;
  functions: string[];
}

export type RuleCategory =
  | "process"
  | "persistence"
  | "powershell"
  | "packing"
  | "network"
  | "signatures"
  | "general";

/** Ficha descriptiva de una regla del catálogo heurístico (FASE 4). */
export interface RuleInfo {
  id: string;
  category: RuleCategory;
  name: string;
  description: string;
  severity: Severity;
  points: number;
}

/** Resultado de un motor (vendor) de VirusTotal. */
export interface VtVendorResult {
  engine: string;
  category: string;
  result?: string | null;
}

/** Reputación por hash consultada a VirusTotal (FASE 5). */
export interface VirusTotalResult {
  available: boolean;
  hash: string;
  malicious: number;
  suspicious: number;
  harmless: number;
  undetected: number;
  timeout: number;
  typeUnsupported: number;
  total: number;
  reputation: number;
  timesSubmitted: number;
  firstSubmissionIso?: string | null;
  lastAnalysisIso?: string | null;
  meaningfulName?: string | null;
  magic?: string | null;
  size?: number | null;
  threatNames: string[];
  vendors: VtVendorResult[];
  permalink: string;
  error?: string | null;
}

/** Evaluación explicativa basada en evidencia (FASE 6, motor local). */
export interface AiAssessment {
  verdict: "clean" | "likely_clean" | "suspicious" | "malicious";
  confidence: number;
  summary: string;
  explanation: string;
  indicators: string[];
  potentialImpact: string[];
  systemConsequences: string[];
  recommendedActions: string[];
  attackVectors: string[];
  keyCategories: string[];
}

/** Archivo aislado en cuarentena (FASE 7). */
export interface QuarantineEntry {
  id: string;
  originalPath: string;
  originalName: string;
  quarantinedPath: string;
  size: number;
  hashes: FileHashes;
  reason: string;
  threatLevel: ThreatLevel;
  quarantinedAt: string;
}

/** Vista de la cuarentena: directorio efectivo + entradas. */
export interface QuarantineSummary {
  dir: string;
  entries: QuarantineEntry[];
}

export interface TimelineEntry {
  time: string;
  label: string;
}

export interface ScanResult {
  id: string;
  fileName: string;
  path: string;
  size: number;
  hashes: FileHashes;
  threatScore: number;
  threatLevel: ThreatLevel;
  findings: Finding[];
  staticAnalysis?: StaticAnalysis | null;
  reputation?: VirusTotalResult | null;
  aiAssessment?: AiAssessment | null;
  language: Language;
  scannedAt: string;
  timeline: TimelineEntry[];
}

export type ScanKind = "file" | "folder";

export interface ScanHistoryEntry {
  id: string;
  kind: ScanKind;
  path: string;
  name: string;
  size: number;
  fileCount: number;
  errorCount: number;
  threatLevel: ThreatLevel;
  scannedAt: string;
  durationMs: number;
}

export interface FolderFileEntry {
  relativePath: string;
  size: number;
  hashes: FileHashes;
  error?: string | null;
}

export type { PowerShellResult, PsCommandInfo, RiskLevel } from "./powershell";

/** Plataforma de ejecución de la aplicación. */
export type Platform = "windows" | "linux" | "macos";

export interface FolderScanResult {
  id: string;
  folderPath: string;
  fileCount: number;
  scannedCount: number;
  skippedCount: number;
  errorCount: number;
  totalBytes: number;
  scannedAt: string;
  durationMs: number;
  files: FolderFileEntry[];
}

export interface PathInfo {
  path: string;
  name: string;
  isDir: boolean;
  size: number;
}

export interface ScanProgress {
  scanId: string;
  current: number;
  total: number;
  filePath?: string | null;
}

export type ScanEvent =
  | { type: "progress"; progress: ScanProgress }
  | { type: "completed"; scanId: string; entry: ScanHistoryEntry }
  | { type: "error"; scanId: string; message: string }
  | { type: "cancelled"; scanId: string };

export interface ScanPreferences {
  maxFileSizeMb: number;
  followSymlinks: boolean;
  computeMd5: boolean;
  computeSha1: boolean;
  computeSha256: boolean;
}

export interface StoragePreferences {
  quarantineDir?: string | null;
  keepHistoryDays: number;
}

export interface AppConfig {
  version: number;
  language: Language;
  theme: ThemePreference;
  virustotalApiKey?: string | null;
  virustotalEnabled: boolean;
  contextMenuEnabled: boolean;
  scan: ScanPreferences;
  storage: StoragePreferences;
}

export interface SystemInfo {
  osName: string;
  osVersion: string;
  osEdition?: string | null;
  osFamily: string;
  architecture: string;
  hostname: string;
  username: string;
  cpuPhysicalCores: number;
  cpuVirtualCores: number;
  totalMemoryBytes: number;
}

export interface AppInfo {
  name: string;
  version: string;
  tagline: string;
}
