import { AlertTriangle, ShieldCheck } from "lucide-react";
import { useLanguage } from "../../contexts/LanguageContext";
import type { TranslationKey } from "../../lib/i18n";
import type { RiskLevel } from "../../types";

const STYLES: Record<RiskLevel, string> = {
  safe: "border-good/30 bg-good/15 text-good",
  low: "border-info/30 bg-info/15 text-info",
  medium: "border-warn/30 bg-warn/15 text-warn",
  high: "border-critical/30 bg-critical/15 text-critical",
};

const LABELS: Record<RiskLevel, TranslationKey> = {
  safe: "powershell.riskSafe",
  low: "powershell.riskLow",
  medium: "powershell.riskMedium",
  high: "powershell.riskHigh",
};

export function RiskBadge({ risk, className = "" }: { risk: RiskLevel; className?: string }) {
  const { t } = useLanguage();
  return (
    <span
      className={`inline-flex items-center gap-1 rounded-full border px-2 py-0.5 text-[11px] font-semibold ${STYLES[risk]} ${className}`}
    >
      {risk === "high" ? (
        <AlertTriangle className="size-3" />
      ) : (
        <ShieldCheck className="size-3" />
      )}
      {t(LABELS[risk])}
    </span>
  );
}
