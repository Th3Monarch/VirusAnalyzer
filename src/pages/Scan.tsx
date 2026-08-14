import { useCallback, useEffect, useState } from "react";
import { useNavigate, useSearchParams } from "react-router-dom";
import { open } from "@tauri-apps/plugin-dialog";
import {
  AlertTriangle,
  CheckCircle2,
  FileUp,
  FolderOpen,
  Loader2,
  Play,
  RefreshCw,
  UploadCloud,
  X,
} from "lucide-react";
import { useLanguage } from "../contexts/LanguageContext";
import { PageHeader } from "../components/ui/PageHeader";
import { Card } from "../components/ui/Card";
import { Button } from "../components/ui/Button";
import { LevelBadge } from "../components/ui/Badge";
import { tauri } from "../lib/tauri";
import { onDragDrop, subscribeScanEvents } from "../lib/events";
import { formatBytes } from "../lib/format";
import type { PathInfo, ScanProgress } from "../types";

type Status = "idle" | "preparing" | "ready" | "scanning" | "done" | "error" | "cancelled";

export function Scan() {
  const { t } = useLanguage();
  const navigate = useNavigate();
  const [searchParams, setSearchParams] = useSearchParams();

  const [selected, setSelected] = useState<PathInfo | null>(null);
  const [status, setStatus] = useState<Status>("idle");
  const [scanId, setScanId] = useState<string | null>(null);
  const [dragOver, setDragOver] = useState(false);
  const [progress, setProgress] = useState<ScanProgress>({
    scanId: "",
    current: 0,
    total: 0,
    filePath: null,
  });
  const [message, setMessage] = useState<string | null>(null);
  const [cancelling, setCancelling] = useState(false);

  const selectPath = useCallback(async (path: string) => {
    setStatus("preparing");
    setMessage(null);
    setProgress({ scanId: "", current: 0, total: 0, filePath: null });
    try {
      const info = await tauri.getPathInfo(path);
      setSelected(info);
      setStatus("ready");
    } catch (e) {
      setMessage(String(e));
      setStatus("error");
    }
  }, []);

  const runScan = useCallback(async (info: PathInfo) => {
    setSelected(info);
    setStatus("scanning");
    setMessage(null);
    setCancelling(false);
    setProgress({ scanId: "", current: 0, total: 0, filePath: info.path });
    try {
      const id = await tauri.scanPath(info.path);
      setScanId(id);
    } catch (e) {
      setStatus("error");
      setMessage(String(e));
    }
  }, []);

  // Lanzado desde el menú contextual de Windows: `?path=<ruta>` inicia el
  // análisis directamente (acción explícita del usuario).
  useEffect(() => {
    const raw = searchParams.get("path");
    if (!raw || !raw.trim()) return;
    setSearchParams({}, { replace: true });
    void (async () => {
      try {
        const info = await tauri.getPathInfo(raw.trim());
        await runScan(info);
      } catch (e) {
        setStatus("error");
        setMessage(String(e));
      }
    })();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  useEffect(() => {
    const unlisten = subscribeScanEvents((event) => {
      switch (event.type) {
        case "progress":
          if (scanId && event.progress.scanId !== scanId) return;
          setProgress(event.progress);
          break;
        case "completed":
          if (event.scanId === scanId) {
            setStatus("done");
            setProgress((p) => ({ ...p, current: p.total }));
            navigate(`/analysis/${event.entry.id}`);
          }
          break;
        case "error":
          if (event.scanId === scanId) {
            setStatus("error");
            setMessage(event.message);
          }
          break;
        case "cancelled":
          if (event.scanId === scanId) {
            setStatus("cancelled");
            setCancelling(false);
          }
          break;
      }
    });
    return () => {
      void unlisten.then((fn) => fn());
    };
  }, [scanId, navigate]);

  useEffect(() => {
    const unlisten = onDragDrop(
      (paths) => {
        if (paths.length > 0 && status !== "scanning") {
          void selectPath(paths[0]);
        }
      },
      ({ over }) => setDragOver(over),
    );
    return unlisten;
  }, [status, selectPath]);

  const chooseFile = async () => {
    const result = await open({ multiple: false, directory: false });
    if (typeof result === "string") await selectPath(result);
  };

  const chooseFolder = async () => {
    const result = await open({ multiple: false, directory: true });
    if (typeof result === "string") await selectPath(result);
  };

  const start = async () => {
    if (!selected || status === "scanning") return;
    await runScan(selected);
  };

  const cancel = async () => {
    setCancelling(true);
    try {
      await tauri.cancelScan();
    } catch {
      setCancelling(false);
      setMessage(t("scan.cancelFailed"));
    }
  };

  const reset = () => {
    setSelected(null);
    setScanId(null);
    setMessage(null);
    setStatus("idle");
    setProgress({ scanId: "", current: 0, total: 0, filePath: null });
    setCancelling(false);
  };

  const percent =
    progress.total > 0 ? Math.min(100, Math.round((progress.current / progress.total) * 100)) : 0;
  const scanning = status === "scanning";

  return (
    <div>
      <PageHeader
        title={t("scan.title")}
        subtitle={t("scan.subtitle")}
        actions={selected ? <LevelBadge level="Clean" /> : undefined}
      />

      <Card className="max-w-3xl">
        {/* Zona de selección / drop */}
        {!selected || scanning ? (
          <div
            className={`m-5 flex flex-col items-center justify-center gap-4 rounded-xl border-2 border-dashed px-6 py-14 text-center transition-colors ${
              dragOver ? "border-accent bg-accent/5" : "border-line bg-surface-2/50"
            }`}
          >
            <div className="flex size-14 items-center justify-center rounded-2xl bg-surface text-accent">
              {scanning ? <Loader2 className="size-7 animate-spin" /> : <UploadCloud className="size-7" />}
            </div>
            <div>
              <p className="text-sm font-medium text-ink">
                {scanning ? t("scan.scanningLabel") : t("scan.dropHere")}
              </p>
              <p className="mt-1 text-xs text-muted">
                {scanning
                  ? progress.filePath ?? t("scan.waitingSelection")
                  : t("scan.waitingSelection")}
              </p>
            </div>

            {!scanning && (
              <div className="flex flex-wrap items-center justify-center gap-2">
                <Button onClick={() => void chooseFile()}>
                  <FileUp className="size-4" />
                  {t("scan.selectFile")}
                </Button>
                <Button variant="secondary" onClick={() => void chooseFolder()}>
                  <FolderOpen className="size-4" />
                  {t("scan.selectFolder")}
                </Button>
              </div>
            )}
          </div>
        ) : (
          <div className="px-5 py-5">
            <div className="flex items-center gap-3 rounded-xl border border-line bg-surface-2/50 px-4 py-3">
              {selected.isDir ? (
                <FolderOpen className="size-5 shrink-0 text-accent" />
              ) : (
                <FileUp className="size-5 shrink-0 text-accent" />
              )}
              <div className="min-w-0 flex-1">
                <p className="truncate text-sm font-medium text-ink">{selected.name}</p>
                <p className="truncate text-xs text-muted">{selected.path}</p>
              </div>
              <div className="shrink-0 text-right">
                <p className="text-xs font-medium text-ink">
                  {selected.isDir ? t("scan.kind.folder") : t("scan.kind.file")}
                </p>
                <p className="text-xs text-muted">{selected.isDir ? t("scan.selected") : formatBytes(selected.size)}</p>
              </div>
            </div>
          </div>
        )}

        {/* Progreso */}
        {scanning && (
          <div className="px-5 pb-4">
            <div className="mb-1.5 flex items-center justify-between text-xs">
              <span className="text-muted">
                {t("scan.progress")}: {progress.total > 0 ? `${progress.current} / ${progress.total} ${t("scan.filesFound")}` : t("scan.unknownTotal")}
              </span>
              <span className="font-medium text-ink">{progress.total > 0 ? `${percent}%` : "…"}</span>
            </div>
            <div className="h-2 w-full overflow-hidden rounded-full bg-surface-2">
              <div
                className={`h-full rounded-full bg-accent transition-all ${progress.total === 0 ? "w-1/3 animate-pulse" : ""}`}
                style={{ width: progress.total > 0 ? `${percent}%` : undefined }}
              />
            </div>
            {progress.filePath ? (
              <p className="mt-2 truncate text-[11px] text-muted" title={progress.filePath}>
                <span className="font-medium text-ink">{t("scan.currentFile")}:</span> {progress.filePath}
              </p>
            ) : null}
          </div>
        )}

        {/* Mensajes */}
        {(status === "error" || status === "cancelled") && message && (
          <div
            role="status"
            aria-live="polite"
            className="mx-5 mb-4 flex items-start gap-2 rounded-lg border border-critical/30 bg-critical/10 px-3 py-2 text-xs text-critical"
          >
            <AlertTriangle className="mt-0.5 size-4 shrink-0" />
            <span>{message}</span>
          </div>
        )}
        {status === "cancelled" && !message && (
          <div
            role="status"
            aria-live="polite"
            className="mx-5 mb-4 flex items-start gap-2 rounded-lg border border-warn/30 bg-warn/10 px-3 py-2 text-xs text-warn"
          >
            <AlertTriangle className="mt-0.5 size-4 shrink-0" />
            <span>{t("scan.cancelled")}</span>
          </div>
        )}
        {status === "done" && (
          <div
            role="status"
            aria-live="polite"
            className="mx-5 mb-4 flex items-start gap-2 rounded-lg border border-good/30 bg-good/10 px-3 py-2 text-xs text-good"
          >
            <CheckCircle2 className="mt-0.5 size-4 shrink-0" />
            <span>{t("scan.selectedInfo")}</span>
          </div>
        )}

        {/* Acciones */}
        <div className="flex items-center justify-between gap-3 border-t border-line px-5 py-4">
          <div className="text-xs text-muted">
            {status === "idle" || status === "ready" ? t("scan.selectedInfo") : status === "preparing" ? t("scan.waitingSelection") : ""}
          </div>
          <div className="flex items-center gap-2">
            {scanning ? (
              <Button variant="secondary" onClick={() => void cancel()} disabled={cancelling}>
                <X className="size-4" />
                {cancelling ? t("scan.cancelling") : t("scan.cancel")}
              </Button>
            ) : selected ? (
              <>
                <Button variant="secondary" onClick={reset}>
                  <RefreshCw className="size-4" />
                  {t("scan.chooseAnother")}
                </Button>
                <Button onClick={() => void start()}>
                  <Play className="size-4" />
                  {t("scan.start")}
                </Button>
              </>
            ) : (
              <Button onClick={() => void start()} disabled>
                <Play className="size-4" />
                {t("scan.start")}
              </Button>
            )}
          </div>
        </div>
      </Card>
    </div>
  );
}
