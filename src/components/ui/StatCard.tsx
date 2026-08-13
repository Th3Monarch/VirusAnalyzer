import { memo } from "react";
import type { ComponentType } from "react";

export const StatCard = memo(function StatCard({
  icon: Icon,
  label,
  value,
  hint,
  tone = "default",
}: {
  icon: ComponentType<{ className?: string }>;
  label: string;
  value: string | number;
  hint?: string;
  tone?: "default" | "good" | "warn" | "bad" | "critical" | "accent";
}) {
  const toneStyles: Record<string, string> = {
    default: "text-muted",
    good: "text-good",
    warn: "text-warn",
    bad: "text-bad",
    critical: "text-critical",
    accent: "text-accent",
  };

  return (
    <div className="flex items-start gap-4 rounded-xl border border-line bg-surface p-4">
      <div
        className={`flex size-10 shrink-0 items-center justify-center rounded-lg border border-line bg-surface-2 ${toneStyles[tone]}`}
      >
        <Icon className="size-5" />
      </div>
      <div className="min-w-0">
        <p className="text-xs font-medium text-muted">{label}</p>
        <p className="mt-0.5 text-2xl font-bold tabular-nums tracking-tight text-ink">{value}</p>
        {hint ? <p className="mt-0.5 text-[11px] text-muted">{hint}</p> : null}
      </div>
    </div>
  );
});
