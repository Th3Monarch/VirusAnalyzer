import { memo } from "react";
import type { Severity, ThreatLevel } from "../../types";

const levelStyles: Record<ThreatLevel, string> = {
  Clean: "bg-good/15 text-good border-good/30",
  Low: "bg-info/15 text-info border-info/30",
  Medium: "bg-warn/15 text-warn border-warn/30",
  High: "bg-bad/15 text-bad border-bad/30",
  Critical: "bg-critical/15 text-critical border-critical/30",
};

const severityStyles: Record<Severity, string> = {
  info: "bg-info/15 text-info border-info/30",
  low: "bg-info/15 text-info border-info/30",
  medium: "bg-warn/15 text-warn border-warn/30",
  high: "bg-bad/15 text-bad border-bad/30",
  critical: "bg-critical/15 text-critical border-critical/30",
};

export const LevelBadge = memo(function LevelBadge({ level }: { level: ThreatLevel }) {
  return (
    <span
      className={`inline-flex items-center gap-1.5 rounded-full border px-2.5 py-0.5 text-xs font-semibold ${levelStyles[level]}`}
    >
      <span className="size-1.5 rounded-full bg-current" />
      {level}
    </span>
  );
});

export const SeverityBadge = memo(function SeverityBadge({ severity }: { severity: Severity }) {
  return (
    <span
      className={`inline-flex items-center rounded-full border px-2.5 py-0.5 text-xs font-medium ${severityStyles[severity]}`}
    >
      {severity}
    </span>
  );
});
