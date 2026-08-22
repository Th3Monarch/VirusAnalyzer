import type { AppConfig } from "../types";

export const DEFAULT_CONFIG: AppConfig = {
  version: 1,
  language: "es",
  theme: "dark",
  virustotalApiKey: null,
  virustotalEnabled: false,
  contextMenuEnabled: false,
  scan: {
    maxFileSizeMb: 200,
    followSymlinks: false,
    computeMd5: true,
    computeSha1: true,
    computeSha256: true,
  },
  storage: {
    quarantineDir: null,
    keepHistoryDays: 90,
  },
  ollama: {
    url: "http://localhost:11434",
    model: "llama3.2",
    enabled: false,
    temperature: 0.3,
    maxTokens: 1024,
  },
  voice: {
    speechRate: 1.0,
    volume: 1.0,
    ttsProvider: "web",
    sttProvider: "web",
    ttsUrl: "http://localhost:8880",
    sttUrl: "http://localhost:8080",
    language: "es",
    voiceId: "af_heart",
  },
};
