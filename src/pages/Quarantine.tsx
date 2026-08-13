import { useCallback, useEffect, useState } from "react";
import { Archive, FolderOpen, RefreshCw, RotateCcw, Trash2 } from "lucide-react";
import { useLanguage } from "../contexts/LanguageContext";
import { useToast } from "../contexts/ToastContext";
import { PageHeader } from "../components/ui/PageHeader";
import { Card, CardHeader } from "../components/ui/Card";
import { EmptyState } from "../components/ui/EmptyState";
import { Button } from "../components/ui/Button";
import { LevelBadge } from "../components/ui/Badge";
import { tauri } from "../lib/tauri";
import { formatBytes, formatDate } from "../lib/format";
import type { QuarantineEntry, QuarantineSummary } from "../types";

export function Quarantine() {
  const { t } = useLanguage();
  const { toast } = useToast();
  const [summary, setSummary] = useState<QuarantineSummary | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [busyId, setBusyId] = useState<string | null>(null);

  const load = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      setSummary(await tauri.getQuarantine());
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    void load();
  }, [load]);

  const onRestore = async (entry: QuarantineEntry) => {
    if (!window.confirm(t("quarantine.confirmRestore").replace("{file}", entry.originalName))) return;
    setBusyId(entry.id);
    try {
      await tauri.restoreQuarantined(entry.id);
      await load();
      toast(t("quarantine.restored").replace("{file}", entry.originalName), "success");
    } catch (e) {
      toast(e instanceof Error ? e.message : String(e), "error");
    } finally {
      setBusyId(null);
    }
  };

  const onDelete = async (entry: QuarantineEntry) => {
    if (!window.confirm(t("quarantine.confirmDelete").replace("{file}", entry.originalName))) return;
    setBusyId(entry.id);
    try {
      await tauri.deleteQuarantined(entry.id);
      await load();
      toast(t("quarantine.deleted").replace("{file}", entry.originalName), "success");
    } catch (e) {
      toast(e instanceof Error ? e.message : String(e), "error");
    } finally {
      setBusyId(null);
    }
  };

  const entries = summary?.entries ?? [];

  return (
    <div>
      <PageHeader
        title={t("quarantine.title")}
        subtitle={t("quarantine.subtitle")}
        actions={
          <Button variant="secondary" onClick={() => void load()} disabled={loading}>
            <RefreshCw className={`size-4 ${loading ? "animate-spin" : ""}`} />
            {t("common.refresh")}
          </Button>
        }
      />

      {error ? (
        <Card>
          <div role="alert" className="px-5 py-4">
            <p className="text-sm text-critical">{error}</p>
          </div>
        </Card>
      ) : null}

      <Card>
        <CardHeader title={t("quarantine.title")} />

        {loading && entries.length === 0 ? (
          <div className="px-5 py-10 text-center text-sm text-muted">{t("common.loading")}</div>
        ) : entries.length === 0 ? (
          <EmptyState
            icon={Archive}
            title={t("quarantine.empty")}
            description={t("quarantine.emptyDesc")}
          />
        ) : (
          <div className="overflow-x-auto">
            <table className="w-full text-left text-sm">
              <thead>
                <tr className="border-b border-line text-xs uppercase tracking-wider text-muted">
                  <th className="px-5 py-2 font-semibold">{t("quarantine.id")}</th>
                  <th className="px-5 py-2 font-semibold">{t("quarantine.originalName")}</th>
                  <th className="px-5 py-2 font-semibold">{t("quarantine.originalPath")}</th>
                  <th className="px-5 py-2 font-semibold">{t("quarantine.level")}</th>
                  <th className="px-5 py-2 font-semibold">{t("common.size")}</th>
                  <th className="px-5 py-2 font-semibold">{t("common.date")}</th>
                  <th className="px-5 py-2 font-semibold">{t("quarantine.reason")}</th>
                  <th className="px-5 py-2 text-right font-semibold">{t("common.actions")}</th>
                </tr>
              </thead>
              <tbody>
                {entries.map((entry) => (
                  <tr key={entry.id} className="border-b border-line last:border-0">
                    <td className="px-5 py-3 font-mono text-xs font-semibold text-ink">{entry.id}</td>
                    <td className="max-w-52 truncate px-5 py-3 font-medium text-ink">
                      <div className="flex items-center gap-1.5">
                        <Archive className="size-3.5 shrink-0 text-warn" />
                        <span className="truncate">{entry.originalName}</span>
                      </div>
                    </td>
                    <td className="max-w-64 truncate px-5 py-3 font-mono text-xs text-muted" title={entry.originalPath}>
                      {entry.originalPath}
                    </td>
                    <td className="px-5 py-3">
                      <LevelBadge level={entry.threatLevel} />
                    </td>
                    <td className="px-5 py-3 text-muted">{formatBytes(entry.size)}</td>
                    <td className="px-5 py-3 text-muted">{formatDate(entry.quarantinedAt)}</td>
                    <td className="max-w-48 truncate px-5 py-3 text-xs text-muted" title={entry.reason}>
                      {entry.reason || "—"}
                    </td>
                    <td className="px-5 py-3">
                      <div className="flex justify-end gap-2">
                        <Button variant="secondary" onClick={() => void onRestore(entry)} disabled={busyId === entry.id}>
                          <RotateCcw className="size-4" />
                          {t("quarantine.restore")}
                        </Button>
                        <Button variant="danger" onClick={() => void onDelete(entry)} disabled={busyId === entry.id}>
                          <Trash2 className="size-4" />
                          {t("quarantine.delete")}
                        </Button>
                      </div>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )}

        {summary ? (
          <div className="flex items-center gap-2 border-t border-line px-5 py-3 text-xs text-muted">
            <FolderOpen className="size-3.5 shrink-0" />
            <span>{t("quarantine.dir")}</span>
            <code className="ml-1 truncate font-mono text-[11px] text-ink/70">{summary.dir}</code>
          </div>
        ) : null}
      </Card>
    </div>
  );
}
