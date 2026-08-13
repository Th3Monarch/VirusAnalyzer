import { useEffect, useState } from "react";
import { useNavigate } from "react-router-dom";
import {
  Activity,
  Archive,
  CheckCircle2,
  Cpu,
  FileSearch,
  FileUp,
  FolderOpen,
  Globe,
  ShieldAlert,
  ShieldX,
  XCircle,
} from "lucide-react";
import { useLanguage } from "../contexts/LanguageContext";
import { useConfig } from "../contexts/ConfigContext";
import { tauri } from "../lib/tauri";
import { formatBytes, formatDate } from "../lib/format";
import type { AppInfo, ScanHistoryEntry, SystemInfo } from "../types";
import { PageHeader } from "../components/ui/PageHeader";
import { Card, CardHeader } from "../components/ui/Card";
import { StatCard } from "../components/ui/StatCard";
import { LevelBadge } from "../components/ui/Badge";
import { Button } from "../components/ui/Button";
import { EmptyState } from "../components/ui/EmptyState";

export function Dashboard() {
  const { t } = useLanguage();
  const navigate = useNavigate();
  const { config } = useConfig();

  const [appInfo, setAppInfo] = useState<AppInfo | null>(null);
  const [sysInfo, setSysInfo] = useState<SystemInfo | null>(null);
  const [sysError, setSysError] = useState<string | null>(null);
  const [history, setHistory] = useState<ScanHistoryEntry[]>([]);

  useEffect(() => {
    void tauri.getAppInfo().then(setAppInfo).catch(() => undefined);
    void tauri
      .getSystemInfo()
      .then(setSysInfo)
      .catch((e) => setSysError(e instanceof Error ? e.message : String(e)));
    void tauri.getScanHistory().then(setHistory).catch(() => undefined);
  }, []);

  const hasVtKey = Boolean(config.virustotalApiKey);
  const vtActive = hasVtKey && config.virustotalEnabled;
  const threats = history.filter((e) => e.threatLevel !== "Clean").length;
  const criticals = history.filter((e) => e.threatLevel === "Critical").length;
  const recent = history.slice(0, 5);

  return (
    <div>
      <PageHeader
        title={`${t("dashboard.title")}${appInfo ? ` · ${appInfo.name}` : ""}`}
        subtitle={t("dashboard.subtitle")}
      />

      <div className="grid grid-cols-1 gap-4 sm:grid-cols-2 xl:grid-cols-4">
        <StatCard icon={FileSearch} label={t("dashboard.totalScans")} value={history.length} />
        <StatCard icon={ShieldAlert} label={t("dashboard.threats")} value={threats} tone="warn" />
        <StatCard icon={ShieldX} label={t("dashboard.criticalThreats")} value={criticals} tone="critical" />
        <StatCard icon={Archive} label={t("dashboard.quarantined")} value={0} tone="accent" />
      </div>

      <div className="mt-6 grid grid-cols-1 gap-6 xl:grid-cols-3">
        {/* Análisis recientes */}
        <Card className="xl:col-span-2">
          <CardHeader
            title={t("dashboard.recentScans")}
            action={
              recent.length > 0 ? (
                <Button variant="ghost" onClick={() => navigate("/results")}>
                  {t("results.title")}
                </Button>
              ) : undefined
            }
          />
          {recent.length === 0 ? (
            <EmptyState
              icon={FileSearch}
              title={t("dashboard.noScansYet")}
              description={t("scan.dropHere")}
            >
              <Button variant="secondary" onClick={() => navigate("/scan")}>
                {t("scan.title")}
              </Button>
            </EmptyState>
          ) : (
            <ul className="divide-y divide-line">
              {recent.map((entry) => (
                <li
                  key={entry.id}
                  className="flex cursor-pointer items-center gap-3 px-5 py-3 hover:bg-surface-2/50"
                  onClick={() => navigate(`/analysis/${entry.id}`)}
                >
                  {entry.kind === "folder" ? (
                    <FolderOpen className="size-4 shrink-0 text-accent" />
                  ) : (
                    <FileUp className="size-4 shrink-0 text-accent" />
                  )}
                  <div className="min-w-0 flex-1">
                    <p className="truncate text-sm font-medium text-ink">{entry.name}</p>
                    <p className="truncate text-[11px] text-muted">
                      {formatBytes(entry.size)}
                      {entry.kind === "folder" ? ` · ${entry.fileCount} ${t("scan.filesFound")}` : ""}
                    </p>
                  </div>
                  <span className="shrink-0 text-[11px] text-muted">{formatDate(entry.scannedAt)}</span>
                  <LevelBadge level={entry.threatLevel} />
                </li>
              ))}
            </ul>
          )}
        </Card>

        <div className="space-y-6">
          {/* Estado del sistema */}
          <Card>
            <CardHeader title={t("dashboard.systemStatus")} />
            <div className="px-5 py-4">
              {sysError ? (
                <EmptyState icon={XCircle} title={t("common.error")} description={sysError} />
              ) : !sysInfo ? (
                <p className="text-sm text-muted">{t("common.loading")}</p>
              ) : (
                <ul className="space-y-3">
                  <li className="flex items-center gap-3">
                    <Cpu className="size-4 text-accent" />
                    <span className="text-sm text-muted">{t("system.os")}</span>
                    <span className="ml-auto text-sm font-medium text-ink">
                      {sysInfo.osName} {sysInfo.osVersion}
                    </span>
                  </li>
                  <li className="flex items-center gap-3">
                    <Activity className="size-4 text-accent" />
                    <span className="text-sm text-muted">{t("system.cpu")}</span>
                    <span className="ml-auto text-sm font-medium text-ink">
                      {sysInfo.cpuPhysicalCores} / {sysInfo.cpuVirtualCores}
                    </span>
                  </li>
                  <li className="flex items-center gap-3">
                    <Globe className="size-4 text-accent" />
                    <span className="text-sm text-muted">{t("common.memory")}</span>
                    <span className="ml-auto text-sm font-medium text-ink">
                      {formatBytes(sysInfo.totalMemoryBytes)}
                    </span>
                  </li>
                </ul>
              )}
            </div>
          </Card>

          {/* Estado de VirusTotal */}
          <Card>
            <CardHeader title={t("dashboard.virustotalStatus")} />
            <div className="px-5 py-4">
              {vtActive ? (
                <div className="flex items-center gap-3">
                  <CheckCircle2 className="size-5 text-good" />
                  <div>
                    <p className="text-sm font-medium text-ink">{t("dashboard.vtAvailable")}</p>
                    <p className="text-xs text-muted">{t("dashboard.vtAvailableDesc")}</p>
                  </div>
                </div>
              ) : hasVtKey ? (
                <div className="flex items-center gap-3">
                  <XCircle className="size-5 text-warn" />
                  <div>
                    <p className="text-sm font-medium text-ink">{t("dashboard.vtDisabled")}</p>
                    <p className="text-xs text-muted">{t("dashboard.vtDisabledDesc")}</p>
                  </div>
                  <Button variant="secondary" className="ml-auto shrink-0" onClick={() => navigate("/settings")}>
                    {t("dashboard.vtConfigured")}
                  </Button>
                </div>
              ) : (
                <div className="flex items-center gap-3">
                  <XCircle className="size-5 text-muted" />
                  <div>
                    <p className="text-sm font-medium text-ink">{t("dashboard.vtMissing")}</p>
                    <p className="text-xs text-muted">{t("settings.vtKeyHelp")}</p>
                  </div>
                </div>
              )}
            </div>
          </Card>

          {/* Actividad reciente */}
          <Card>
            <CardHeader title={t("dashboard.recentActivity")} />
            {recent.length === 0 ? (
              <EmptyState icon={Activity} title={t("dashboard.noScansYet")} />
            ) : (
              <ul className="divide-y divide-line">
                {recent.slice(0, 3).map((entry) => (
                  <li key={entry.id} className="px-5 py-2.5 text-xs">
                    <span className="text-muted">{formatDate(entry.scannedAt)}</span>
                    <p className="truncate font-medium text-ink">{entry.name}</p>
                  </li>
                ))}
              </ul>
            )}
          </Card>
        </div>
      </div>
    </div>
  );
}
