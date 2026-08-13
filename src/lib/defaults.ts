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
};
