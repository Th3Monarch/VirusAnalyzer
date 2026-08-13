import { useCallback, useEffect, useRef, useState } from "react";
import type { KeyboardEvent } from "react";
import { useSearchParams } from "react-router-dom";
import { History, ShieldAlert, SquareTerminal } from "lucide-react";
import { useLanguage } from "../../contexts/LanguageContext";
import { tauri } from "../../lib/tauri";
import type { RiskLevel } from "../../types";
import { Card, CardHeader } from "../ui/Card";
import { Button } from "../ui/Button";
import { CommandHistory } from "./CommandHistory";
import { CommandInput } from "./CommandInput";
import { Output, type OutputBlock } from "./Output";
import { RiskBadge } from "./RiskBadge";

const HISTORY_KEY = "va:ps:history";
const FAVORITES_KEY = "va:ps:favorites";
const HISTORY_LIMIT = 100;
const BLOCK_LIMIT = 50;
const SENSITIVE_PATTERN =
  /(password|passwd|\bpwd\b|token|apikey|api[-_]?key|secret|credential|authorization|bearer|private[-_]?key|connectionstring|oauth|session[-_]?key|client[-_]?secret)/i;

function loadList(key: string): string[] {
  try {
    const raw = localStorage.getItem(key);
    if (!raw) return [];
    const parsed = JSON.parse(raw);
    return Array.isArray(parsed) ? parsed.filter((x): x is string => typeof x === "string") : [];
  } catch {
    return [];
  }
}

function saveList(key: string, list: string[]): void {
  try {
    localStorage.setItem(key, JSON.stringify(list));
  } catch {
    // almacenamiento no disponible: no bloquea la ejecución
  }
}

function isSensitive(command: string): boolean {
  return SENSITIVE_PATTERN.test(command);
}

export function Terminal() {
  const { t } = useLanguage();
  const [searchParams, setSearchParams] = useSearchParams();

  const [input, setInput] = useState("");
  const [blocks, setBlocks] = useState<OutputBlock[]>([]);
  const [running, setRunning] = useState(false);
  const [risk, setRisk] = useState<RiskLevel | null>(null);
  const [navIndex, setNavIndex] = useState(-1);
  const [history, setHistory] = useState<string[]>(() => loadList(HISTORY_KEY));
  const [favorites, setFavorites] = useState<string[]>(() => loadList(FAVORITES_KEY));
  const [pendingConfirm, setPendingConfirm] = useState<{ command: string } | null>(null);
  const [sensitiveNotice, setSensitiveNotice] = useState(false);

  const inputRef = useRef<HTMLTextAreaElement>(null);
  const nextId = useRef(1);
  const historyRef = useRef<string[]>(history);

  useEffect(() => {
    historyRef.current = history;
  }, [history]);

  // Prefill desde la referencia (?cmd=Nombre). Nunca se ejecuta solo.
  useEffect(() => {
    const cmd = searchParams.get("cmd");
    if (cmd && cmd.trim()) {
      setInput(cmd.trim());
      setSearchParams({}, { replace: true });
      inputRef.current?.focus();
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  // Riesgo en vivo del comando escrito (heurística local del backend).
  useEffect(() => {
    const cmd = input.trim();
    if (!cmd) {
      setRisk(null);
      return;
    }
    const id = window.setTimeout(() => {
      void tauri
        .classifyPowerShellCommand(cmd)
        .then(setRisk)
        .catch(() => setRisk(null));
    }, 250);
    return () => window.clearTimeout(id);
  }, [input]);

  const pushHistory = useCallback((command: string) => {
    setHistory((prev) => {
      const next = [command, ...prev.filter((c) => c !== command)].slice(0, HISTORY_LIMIT);
      saveList(HISTORY_KEY, next);
      return next;
    });
  }, []);

  const toggleFavorite = useCallback((command: string) => {
    setFavorites((prev) => {
      const next = prev.includes(command)
        ? prev.filter((c) => c !== command)
        : [command, ...prev].slice(0, HISTORY_LIMIT);
      saveList(FAVORITES_KEY, next);
      return next;
    });
  }, []);

  const deleteFromHistory = useCallback((command: string) => {
    setHistory((prev) => {
      const next = prev.filter((c) => c !== command);
      saveList(HISTORY_KEY, next);
      return next;
    });
  }, []);

  const clearHistory = useCallback(() => {
    setHistory([]);
    saveList(HISTORY_KEY, []);
  }, []);

  const doExecute = useCallback(
    async (command: string, confirmed: boolean, riskLevel: RiskLevel) => {
      setRunning(true);
      setSensitiveNotice(false);
      try {
        const result = await tauri.executePowerShell(command, confirmed);
        setBlocks((prev) =>
          [
            {
              id: `${nextId.current++}`,
              command,
              stdout: result.stdout,
              stderr: result.stderr,
              exitCode: result.exitCode,
              durationMs: result.durationMs,
              timedOut: result.timedOut,
              cancelled: result.cancelled,
              risk: riskLevel,
            },
            ...prev,
          ].slice(0, BLOCK_LIMIT),
        );
        if (isSensitive(command)) {
          setSensitiveNotice(true);
        } else {
          pushHistory(command);
        }
      } catch (err) {
        const message = err instanceof Error ? err.message : String(err);
        setBlocks((prev) =>
          [
            {
              id: `${nextId.current++}`,
              command,
              stdout: "",
              stderr: message,
              exitCode: null,
              durationMs: 0,
              timedOut: false,
              cancelled: false,
              risk: riskLevel,
            },
            ...prev,
          ].slice(0, BLOCK_LIMIT),
        );
      } finally {
        setRunning(false);
      }
    },
    [pushHistory],
  );

  const run = useCallback(async () => {
    const command = input.trim();
    if (!command || running) return;
    setInput("");
    setNavIndex(-1);
    let level: RiskLevel = "safe";
    try {
      level = await tauri.classifyPowerShellCommand(command);
    } catch {
      level = "safe";
    }
    if (level === "high") {
      setPendingConfirm({ command });
      return;
    }
    void doExecute(command, false, level);
  }, [input, running, doExecute]);

  const cancel = useCallback(() => {
    void tauri.cancelPowerShell().catch(() => undefined);
  }, []);

  const clearOutput = useCallback(() => setBlocks([]), []);

  const reuse = useCallback((command: string) => {
    setInput(command);
    setNavIndex(-1);
    inputRef.current?.focus();
  }, []);

  const handleChange = useCallback((value: string) => {
    setInput(value);
    setNavIndex(-1);
  }, []);

  const onKeyDown = useCallback(
    (event: KeyboardEvent<HTMLTextAreaElement>) => {
      if ((event.ctrlKey || event.metaKey) && event.key === "Enter") {
        event.preventDefault();
        void run();
        return;
      }
      if ((event.ctrlKey || event.metaKey) && event.key.toLowerCase() === "l") {
        event.preventDefault();
        clearOutput();
        return;
      }
      if (event.key === "ArrowUp") {
        const list = historyRef.current;
        if (list.length > 0) {
          event.preventDefault();
          const next = navIndex + 1 < list.length ? navIndex + 1 : navIndex;
          setNavIndex(next);
          setInput(list[next]);
        }
        return;
      }
      if (event.key === "ArrowDown") {
        const list = historyRef.current;
        if (list.length > 0) {
          event.preventDefault();
          if (navIndex > 0) {
            const next = navIndex - 1;
            setNavIndex(next);
            setInput(list[next]);
          } else if (navIndex === 0) {
            setNavIndex(-1);
            setInput("");
          }
        }
        return;
      }
    },
    [navIndex, run, clearOutput],
  );

  const last = blocks[0];
  const statusText = running
    ? t("powershell.statusRunning")
    : last
      ? last.timedOut
        ? t("powershell.statusTimedOut")
        : last.cancelled
          ? t("powershell.statusCancelled")
          : t("powershell.statusDone")
      : t("powershell.statusIdle");

  return (
    <div className="grid grid-cols-1 gap-6 xl:grid-cols-3">
      <Card className="xl:col-span-2">
        <CardHeader
          title={t("powershell.terminalTitle")}
          subtitle={t("powershell.terminalSubtitle")}
          action={<SquareTerminal className="size-4 text-muted" />}
        />
        <div className="p-5">
          <Output blocks={blocks} />
          <div className="mt-4">
            <CommandInput
              value={input}
              onChange={handleChange}
              onRun={run}
              onCancel={cancel}
              onClear={clearOutput}
              onKeyDown={onKeyDown}
              running={running}
              risk={risk}
              inputRef={inputRef}
            />
          </div>
          <div className="mt-3 flex flex-wrap items-center gap-x-4 gap-y-1">
            <p className="text-[11px] text-muted">{statusText}</p>
            {sensitiveNotice ? (
              <p role="status" className="text-[11px] font-medium text-warn">
                {t("powershell.sensitiveNotSaved")}
              </p>
            ) : null}
          </div>
          <p className="mt-2 text-[11px] text-muted/70">{t("powershell.shortcuts")}</p>
        </div>
      </Card>

      <Card>
        <CardHeader
          title={t("powershell.history")}
          action={<History className="size-4 text-muted" />}
        />
        <div className="p-4">
          <CommandHistory
            history={history}
            favorites={favorites}
            onReuse={reuse}
            onToggleFavorite={toggleFavorite}
            onDelete={deleteFromHistory}
            onClear={clearHistory}
          />
        </div>
      </Card>

      {pendingConfirm ? (
        <div
          className="fixed inset-0 z-50 flex items-center justify-center bg-black/60 p-6"
          onClick={() => setPendingConfirm(null)}
        >
          <div
            role="dialog"
            aria-modal="true"
            aria-label={t("powershell.confirmTitle")}
            className="w-full max-w-lg rounded-xl border border-line bg-surface p-5 shadow-xl"
            onClick={(event) => event.stopPropagation()}
          >
            <div className="flex items-start gap-3">
              <div className="flex size-10 shrink-0 items-center justify-center rounded-lg bg-critical/15 text-critical">
                <ShieldAlert className="size-5" />
              </div>
              <div className="min-w-0 flex-1">
                <h3 className="text-sm font-semibold text-ink">{t("powershell.confirmTitle")}</h3>
                <p className="mt-1 text-xs leading-relaxed text-muted">
                  {t("powershell.confirmText")}
                </p>
                <div className="mt-3">
                  <p className="text-[10px] font-semibold uppercase tracking-wider text-muted">
                    {t("powershell.confirmCommand")}
                  </p>
                  <div className="mt-1 flex items-center justify-between gap-2 rounded-lg border border-critical/30 bg-critical/10 px-3 py-2">
                    <code className="min-w-0 flex-1 truncate font-mono text-xs text-critical">
                      {pendingConfirm.command}
                    </code>
                    <RiskBadge risk="high" className="shrink-0" />
                  </div>
                </div>
              </div>
            </div>
            <div className="mt-5 flex justify-end gap-2">
              <Button variant="secondary" onClick={() => setPendingConfirm(null)}>
                {t("common.cancel")}
              </Button>
              <Button
                variant="danger"
                onClick={() => {
                  const command = pendingConfirm.command;
                  setPendingConfirm(null);
                  void doExecute(command, true, "high");
                }}
              >
                {t("powershell.confirmExecute")}
              </Button>
            </div>
          </div>
        </div>
      ) : null}
    </div>
  );
}
