import { useEffect, useRef } from "react";
import { Clock, TerminalSquare, Timer, XCircle } from "lucide-react";
import { useLanguage } from "../../contexts/LanguageContext";
import { formatDurationMs } from "../../lib/format";
import type { RiskLevel } from "../../types";
import { RiskBadge } from "./RiskBadge";

export interface OutputBlock {
  id: string;
  command: string;
  stdout: string;
  stderr: string;
  exitCode: number | null;
  durationMs: number;
  timedOut: boolean;
  cancelled: boolean;
  risk: RiskLevel;
}

function OutputSection({ label, value, tone }: { label: string; value: string; tone: "default" | "error" }) {
  if (!value.trim()) return null;
  return (
    <div className="mt-2">
      <p
        className={`text-[10px] font-semibold uppercase tracking-wider ${
          tone === "error" ? "text-critical" : "text-muted"
        }`}
      >
        {label}
      </p>
      <pre
        className={`mt-1 max-h-72 overflow-auto whitespace-pre-wrap break-words rounded-lg border border-line bg-surface-2/50 p-2.5 font-mono text-xs leading-relaxed ${
          tone === "error" ? "text-critical" : "text-ink"
        }`}
      >
        {value}
      </pre>
    </div>
  );
}

export function Output({ blocks }: { blocks: OutputBlock[] }) {
  const { t } = useLanguage();
  const scrollRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const el = scrollRef.current;
    if (el) el.scrollTop = el.scrollHeight;
  }, [blocks]);

  if (blocks.length === 0) {
    return (
      <div className="flex items-center gap-2 rounded-xl border border-dashed border-line px-4 py-8 text-xs text-muted">
        <TerminalSquare className="size-4 shrink-0" />
        <span>
          {t("powershell.statusIdle")} {t("powershell.historyEmpty")}
        </span>
      </div>
    );
  }

  return (
    <div ref={scrollRef} className="max-h-[28rem] space-y-3 overflow-auto pr-1">
      {blocks.map((block) => (
        <div key={block.id} className="rounded-xl border border-line bg-surface-2/40 p-3">
          <div className="flex flex-wrap items-center justify-between gap-2">
            <p className="min-w-0 truncate font-mono text-xs font-bold text-accent">
              PS&gt; {block.command}
            </p>
            <RiskBadge risk={block.risk} />
          </div>

          <OutputSection label={t("powershell.output")} value={block.stdout} tone="default" />
          <OutputSection label={t("powershell.error")} value={block.stderr} tone="error" />

          {!block.stdout.trim() && !block.stderr.trim() ? (
            <p className="mt-2 text-xs italic text-muted">{t("powershell.noOutput")}</p>
          ) : null}

          <div className="mt-2.5 flex flex-wrap items-center gap-x-4 gap-y-1 border-t border-line pt-2 text-[11px] text-muted">
            <span className="inline-flex items-center gap-1">
              <Clock className="size-3" />
              {t("powershell.exitCode")}: {block.exitCode ?? "—"}
            </span>
            <span className="inline-flex items-center gap-1">
              <Timer className="size-3" />
              {t("powershell.duration")}: {formatDurationMs(block.durationMs)}
            </span>
            {block.timedOut ? (
              <span className="inline-flex items-center gap-1 font-semibold text-warn">
                <XCircle className="size-3" />
                {t("powershell.timedOut")}
              </span>
            ) : null}
            {block.cancelled ? (
              <span className="inline-flex items-center gap-1 font-semibold text-critical">
                <XCircle className="size-3" />
                {t("powershell.cancelled")}
              </span>
            ) : null}
          </div>
        </div>
      ))}
    </div>
  );
}
