import { useDeferredValue, useEffect, useState } from "react";
import { useNavigate } from "react-router-dom";
import { FileSearch, FolderOpen, FileUp, RefreshCw, Search } from "lucide-react";
import { useLanguage } from "../contexts/LanguageContext";
import { PageHeader } from "../components/ui/PageHeader";
import { Card, CardHeader } from "../components/ui/Card";
import { Button } from "../components/ui/Button";
import { LevelBadge } from "../components/ui/Badge";
import { EmptyState } from "../components/ui/EmptyState";
import { tauri } from "../lib/tauri";
import { formatBytes, formatDate, formatDurationMs } from "../lib/format";
import type { ScanHistoryEntry } from "../types";

export function Results() {
  const { t } = useLanguage();
  const navigate = useNavigate();
  const [history, setHistory] = useState<ScanHistoryEntry[]>([]);
  const [loading, setLoading] = useState(true);
  const [query, setQuery] = useState("");

  const load = async () => {
    setLoading(true);
    try {
      const entries = await tauri.getScanHistory();
      setHistory(entries);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    void load();
  }, []);

  const deferredQuery = useDeferredValue(query);
  const q = deferredQuery.trim().toLowerCase();
  const filtered = q
    ? history.filter(
        (e) =>
          e.name.toLowerCase().includes(q) ||
          e.path.toLowerCase().includes(q) ||
          e.id.toLowerCase().includes(q),
      )
    : history;

  return (
    <div>
      <PageHeader
        title={t("results.title")}
        subtitle={t("results.subtitle")}
        actions={
          <Button variant="secondary" onClick={() => void load()}>
            <RefreshCw className="size-4" />
            {t("results.refresh")}
          </Button>
        }
      />

      <Card>
        <CardHeader title={t("results.title")} />

        <div className="px-5 py-4">
          <div className="relative max-w-md">
            <Search className="pointer-events-none absolute left-3 top-1/2 size-4 -translate-y-1/2 text-muted" />
            <input
              type="search"
              value={query}
              onChange={(e) => setQuery(e.currentTarget.value)}
              placeholder={t("results.searchPlaceholder")}
              className="w-full rounded-lg border border-line bg-surface-2 px-3 py-2 pl-9 text-sm text-ink placeholder:text-muted focus:border-accent focus:outline-none"
            />
          </div>

          <div className="mt-4 overflow-x-auto">
            {loading ? (
              <p className="py-8 text-center text-sm text-muted">{t("common.loading")}</p>
            ) : filtered.length === 0 ? (
              history.length === 0 ? (
                <EmptyState
                  icon={FileSearch}
                  title={t("results.empty")}
                  description={t("scan.dropHere")}
                />
              ) : (
                <EmptyState icon={FileSearch} title={t("results.noMatch")} />
              )
            ) : (
              <table className="w-full text-left text-sm">
                <thead>
                  <tr className="border-b border-line text-xs uppercase tracking-wider text-muted">
                    <th className="pb-2 pr-4 font-semibold">{t("results.fileName")}</th>
                    <th className="pb-2 pr-4 font-semibold">{t("results.kind")}</th>
                    <th className="pb-2 pr-4 font-semibold">{t("common.size")}</th>
                    <th className="pb-2 pr-4 font-semibold">{t("results.fileCount")}</th>
                    <th className="pb-2 pr-4 font-semibold">{t("results.duration")}</th>
                    <th className="pb-2 pr-4 font-semibold">{t("results.threat")}</th>
                    <th className="pb-2 pr-4 font-semibold">{t("common.date")}</th>
                    <th className="pb-2 font-semibold">{t("common.actions")}</th>
                  </tr>
                </thead>
                <tbody>
                  {filtered.map((entry) => (
                    <tr
                      key={entry.id}
                      className="cursor-pointer border-b border-line last:border-0 hover:bg-surface-2/50"
                      onClick={() => navigate(`/analysis/${entry.id}`)}
                    >
                      <td className="py-2.5 pr-4">
                        <div className="flex items-center gap-2">
                          {entry.kind === "folder" ? (
                            <FolderOpen className="size-4 shrink-0 text-accent" />
                          ) : (
                            <FileUp className="size-4 shrink-0 text-accent" />
                          )}
                          <div className="min-w-0">
                            <p className="max-w-[220px] truncate font-medium text-ink">{entry.name}</p>
                            <p className="max-w-[220px] truncate text-[11px] text-muted">{entry.path}</p>
                          </div>
                        </div>
                      </td>
                      <td className="py-2.5 pr-4 text-xs text-muted">
                        {entry.kind === "folder" ? t("scan.kind.folder") : t("scan.kind.file")}
                      </td>
                      <td className="py-2.5 pr-4 text-xs text-muted">{formatBytes(entry.size)}</td>
                      <td className="py-2.5 pr-4 text-xs text-muted">
                        {entry.kind === "folder" ? entry.fileCount : "—"}
                      </td>
                      <td className="py-2.5 pr-4 text-xs text-muted">{formatDurationMs(entry.durationMs)}</td>
                      <td className="py-2.5 pr-4">
                        <LevelBadge level={entry.threatLevel} />
                      </td>
                      <td className="py-2.5 pr-4 text-xs text-muted">{formatDate(entry.scannedAt)}</td>
                      <td className="py-2.5">
                        <Button variant="ghost" onClick={() => navigate(`/analysis/${entry.id}`)}>
                          {t("results.open")}
                        </Button>
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            )}
          </div>
        </div>
      </Card>
    </div>
  );
}
