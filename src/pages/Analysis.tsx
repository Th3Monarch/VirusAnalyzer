import { useEffect, useRef, useState } from "react";
import { Link, useNavigate, useParams } from "react-router-dom";
import {
  AlertTriangle,
  Archive,
  ArrowLeft,
  Check,
  ChevronDown,
  Copy,
  ExternalLink,
  FileCode2,
  FileSearch,
  FileSpreadsheet,
  FolderOpen,
  ListTree,
  Eye,
  ShieldCheck,
} from "lucide-react";
import { openUrl } from "@tauri-apps/plugin-opener";
import { save } from "@tauri-apps/plugin-dialog";
import { useLanguage } from "../contexts/LanguageContext";
import { useToast } from "../contexts/ToastContext";
import { useConfig } from "../contexts/ConfigContext";
import { PageHeader } from "../components/ui/PageHeader";
import { Card, CardHeader } from "../components/ui/Card";
import { Button } from "../components/ui/Button";
import { LevelBadge } from "../components/ui/Badge";
import { EmptyState } from "../components/ui/EmptyState";
import { tauri } from "../lib/tauri";
import { formatBytes, formatDate, formatDurationMs, shortenHash } from "../lib/format";
import type {
  AiAssessment,
  Finding,
  FolderScanResult,
  PeImportDll,
  PeSection,
  ReportFormat,
  ScanResult,
  Severity,
  StaticAnalysis,
  ThreatLevel,
  VirusTotalResult,
} from "../types";
import type { TranslationKey } from "../lib/i18n";

type Loaded = { kind: "file"; data: ScanResult } | { kind: "folder"; data: FolderScanResult };

async function copyText(text: string): Promise<boolean> {
  try {
    await navigator.clipboard.writeText(text);
    return true;
  } catch {
    try {
      const area = document.createElement("textarea");
      area.value = text;
      area.style.position = "fixed";
      area.style.opacity = "0";
      document.body.appendChild(area);
      area.select();
      const ok = document.execCommand("copy");
      document.body.removeChild(area);
      return ok;
    } catch {
      return false;
    }
  }
}

function HashRow({ label, value }: { label: string; value: string | null | undefined }) {
  const { t } = useLanguage();
  const [copied, setCopied] = useState(false);
  if (!value) return null;

  const copy = async () => {
    if (await copyText(value)) {
      setCopied(true);
      setTimeout(() => setCopied(false), 1500);
    }
  };

  return (
    <div className="flex items-center gap-2 py-1.5">
      <span className="w-16 shrink-0 text-xs font-medium text-muted">{label}</span>
      <code className="min-w-0 flex-1 truncate font-mono text-xs text-ink">{value}</code>
      <Button variant="ghost" onClick={() => void copy()} title={t("analysis.copy")} aria-label={`${t("analysis.copy")}: ${label}`}>
        {copied ? <Check className="size-3.5 text-good" /> : <Copy className="size-3.5" />}
      </Button>
    </div>
  );
}

const HEX = (n: number) => `0x${(n >>> 0).toString(16).toUpperCase()}`;

const SEVERITY_STYLES: Record<Severity, string> = {
  info: "border-info/30 bg-info/15 text-info",
  low: "border-info/30 bg-info/15 text-info",
  medium: "border-warn/30 bg-warn/15 text-warn",
  high: "border-bad/30 bg-bad/15 text-bad",
  critical: "border-critical/30 bg-critical/15 text-critical",
};

const LEVEL_BAR_STYLES: Record<ThreatLevel, string> = {
  Clean: "bg-good",
  Low: "bg-info",
  Medium: "bg-warn",
  High: "bg-bad",
  Critical: "bg-critical",
};

const CATEGORY_KEYS: Record<string, TranslationKey> = {
  process: "rules.category.process",
  persistence: "rules.category.persistence",
  powershell: "rules.category.powershell",
  packing: "rules.category.packing",
  network: "rules.category.network",
  signatures: "rules.category.signatures",
  general: "rules.category.general",
};

function ScoreBar({ score, level }: { score: number; level: ThreatLevel }) {
  return (
    <div className="flex min-w-0 items-center gap-3">
      <div className="h-2 min-w-0 flex-1 overflow-hidden rounded-full bg-surface-2">
        <div
          className={`h-full rounded-full ${LEVEL_BAR_STYLES[level]}`}
          style={{ width: `${Math.min(100, Math.max(score, score > 0 ? 3 : 0))}%` }}
        />
      </div>
      <span className="shrink-0 font-mono text-sm font-semibold text-ink">{score}/100</span>
    </div>
  );
}

function ReputationCard({ data, onRetry }: { data: VirusTotalResult; onRetry?: () => void }) {
  const { t } = useLanguage();
  if (data.error) {
    return (
      <Card>
        <CardHeader title="VirusTotal" action={<TypeChip label={t("analysis.vtError")} tone="bad" />} />
        <div className="space-y-3 px-5 py-4">
          <p className="text-xs leading-relaxed text-critical">{data.error}</p>
          {onRetry ? (
            <Button variant="secondary" onClick={onRetry}>
              <ExternalLink className="size-4" />
              {t("analysis.vtRetry")}
            </Button>
          ) : null}
        </div>
      </Card>
    );
  }

  if (!data.available) {
    return (
      <Card>
        <CardHeader title="VirusTotal" action={<TypeChip label={t("analysis.vtNotFound")} tone="warn" />} />
        <div className="px-5 py-4">
          <p className="text-xs leading-relaxed text-muted">{t("analysis.vtNotFoundDesc")}</p>
          <code className="mt-2 block break-all font-mono text-[11px] text-ink/70">{data.hash}</code>
        </div>
      </Card>
    );
  }

  const stats = [
    { label: t("analysis.vtMalicious"), value: data.malicious, cls: "text-critical" },
    { label: t("analysis.vtSuspicious"), value: data.suspicious, cls: "text-warn" },
    { label: t("analysis.vtHarmless"), value: data.harmless, cls: "text-good" },
    { label: t("analysis.vtUndetected"), value: data.undetected, cls: "text-muted" },
  ];

  return (
    <Card>
      <CardHeader
        title="VirusTotal"
        subtitle={data.meaningfulName ?? data.hash}
        action={
          <Button variant="secondary" onClick={() => void openUrl(data.permalink)}>
            <ExternalLink className="size-4" />
            {t("analysis.vtOpen")}
          </Button>
        }
      />
      <div className="px-5 py-4">
        <div className="grid grid-cols-2 gap-3 sm:grid-cols-4">
          {stats.map((s) => (
            <div key={s.label} className="rounded-lg border border-line bg-surface-2/50 px-3 py-2.5">
              <p className={`text-lg font-semibold ${s.cls}`}>{s.value}</p>
              <p className="text-[11px] text-muted">{s.label}</p>
            </div>
          ))}
        </div>

        {data.threatNames.length > 0 ? (
          <div className="mt-4">
            <p className="text-[10px] font-semibold uppercase tracking-wider text-muted">{t("analysis.vtThreats")}</p>
            <div className="mt-1.5 flex flex-wrap gap-1.5">
              {data.threatNames.map((n) => (
                <span
                  key={n}
                  className="rounded border border-bad/30 bg-bad/15 px-2 py-0.5 font-mono text-[11px] text-bad"
                >
                  {n}
                </span>
              ))}
            </div>
          </div>
        ) : null}

        {data.vendors.length > 0 ? (
          <div className="mt-4">
            <p className="text-[10px] font-semibold uppercase tracking-wider text-muted">
              {t("analysis.vtEngines")} ({data.total})
            </p>
            <div className="mt-1.5 max-h-64 overflow-y-auto rounded-lg border border-line">
              <table className="w-full text-left text-xs">
                <tbody>
                  {data.vendors.map((v) => (
                    <tr key={v.engine} className="border-b border-line last:border-0">
                      <td className="py-1.5 pl-3 pr-4 text-ink">{v.engine}</td>
                      <td className="py-1.5 pr-3">
                        <span
                          className={
                            v.category === "malicious"
                              ? "font-medium text-critical"
                              : v.category === "suspicious"
                                ? "font-medium text-warn"
                                : "text-muted"
                          }
                        >
                          {v.category}
                        </span>
                      </td>
                      <td className="py-1.5 pr-3 font-mono text-[11px] text-muted">{v.result ?? "—"}</td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          </div>
        ) : null}
      </div>
    </Card>
  );
}

function FindingsList({ findings }: { findings: Finding[] }) {
  const { t } = useLanguage();
  return (
    <div className="space-y-3 px-5 py-4">
      {findings.map((f, i) => (
        <div key={i} className="rounded-lg border border-line bg-surface-2/40 px-4 py-3">
          <div className="flex flex-wrap items-center gap-2">
            <span
              className={`inline-flex items-center rounded-full border px-2 py-0.5 text-[11px] font-semibold ${SEVERITY_STYLES[f.severity]}`}
            >
              {f.severity}
            </span>
            <span className="text-xs font-medium text-ink">{f.ruleName}</span>
            <span className="text-[11px] text-muted">{t(CATEGORY_KEYS[f.category] ?? "rules.category.general")}</span>
            <span className="ml-auto shrink-0 font-mono text-[11px] font-semibold text-ink/70">
              +{f.points} {t("analysis.points")}
            </span>
          </div>
          <p className="mt-1.5 text-xs leading-relaxed text-muted">{f.description}</p>
          {f.evidence.length > 0 && (
            <div className="mt-2.5">
              <p className="text-[10px] font-semibold uppercase tracking-wider text-muted">
                {t("analysis.evidence")}
              </p>
              <div className="mt-1 flex flex-wrap gap-1.5">
                {f.evidence.map((e) => (
                  <code
                    key={e}
                    className="rounded border border-line bg-surface px-1.5 py-px font-mono text-[10px] text-ink/80"
                  >
                    {e}
                  </code>
                ))}
              </div>
            </div>
          )}
        </div>
      ))}
    </div>
  );
}

function EntropyBar({ value }: { value: number }) {
  const { t } = useLanguage();
  const high = value >= 7;
  const pct = Math.min(100, Math.round((value / 8) * 100));
  return (
    <div className="min-w-0 flex-1">
      <div className="h-1.5 w-full overflow-hidden rounded-full bg-surface-2">
        <div
          className={`h-full rounded-full ${high ? "bg-bad" : "bg-accent"}`}
          style={{ width: `${pct}%` }}
        />
      </div>
      <p className={`mt-1 text-[11px] ${high ? "text-bad" : "text-muted"}`}>
        {value.toFixed(2)} bits/byte · {high ? t("analysis.entropyHigh") : t("analysis.entropyOk")}
      </p>
    </div>
  );
}

function TypeChip({ label, tone }: { label: string; tone?: "good" | "warn" | "bad" | "accent" }) {
  const toneCls =
    tone === "good"
      ? "border-good/30 bg-good/15 text-good"
      : tone === "warn"
        ? "border-warn/30 bg-warn/15 text-warn"
        : tone === "bad"
          ? "border-bad/30 bg-bad/15 text-bad"
          : "border-accent/30 bg-accent/15 text-accent";
  return (
    <span className={`inline-flex items-center rounded-full border px-2 py-0.5 text-[11px] font-medium ${toneCls}`}>
      {label}
    </span>
  );
}

function SectionsTable({ sections }: { sections: PeSection[] }) {
  const { t } = useLanguage();
  return (
    <div className="overflow-x-auto">
      <table className="w-full text-left text-xs">
        <thead>
          <tr className="border-b border-line text-[11px] uppercase tracking-wider text-muted">
            <th className="pb-2 pr-4 font-semibold">{t("analysis.sections")}</th>
            <th className="pb-2 pr-4 font-semibold">{t("analysis.virtualAddress")}</th>
            <th className="pb-2 pr-4 font-semibold">{t("analysis.virtualSize")}</th>
            <th className="pb-2 pr-4 font-semibold">{t("analysis.rawSize")}</th>
            <th className="pb-2 pr-4 font-semibold">{t("analysis.fileEntropy")}</th>
            <th className="pb-2 font-semibold">{t("analysis.flags")}</th>
          </tr>
        </thead>
        <tbody>
          {sections.map((s) => (
            <tr key={s.name + s.virtualAddress} className="border-b border-line last:border-0">
              <td className="py-2 pr-4 font-mono font-semibold text-ink">{s.name}</td>
              <td className="py-2 pr-4 font-mono text-muted">{HEX(s.virtualAddress)}</td>
              <td className="py-2 pr-4 text-muted">{formatBytes(s.virtualSize)}</td>
              <td className="py-2 pr-4 text-muted">{formatBytes(s.rawSize)}</td>
              <td className="py-2 pr-4">
                <span className={`font-mono font-semibold ${s.entropy >= 7 ? "text-bad" : "text-ink"}`}>
                  {s.entropy.toFixed(2)}
                </span>
              </td>
              <td className="py-2">
                <div className="flex flex-wrap gap-1">
                  {s.flags.map((f) => (
                    <span
                      key={f}
                      className={`rounded border px-1 py-px text-[10px] font-medium ${f === "EXEC" || f === "WRITE" ? "border-warn/40 bg-warn/10 text-warn" : "border-line bg-surface-2 text-muted"}`}
                    >
                      {f}
                    </span>
                  ))}
                </div>
              </td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}

function ImportsList({ imports }: { imports: PeImportDll[] }) {
  const { t } = useLanguage();
  const [open, setOpen] = useState<Record<string, boolean>>({});
  const visible = (dll: PeImportDll) => (open[dll.name] ? dll.functions : dll.functions.slice(0, 12));

  return (
    <div className="grid grid-cols-1 gap-3 md:grid-cols-2">
      {imports.map((dll) => (
        <div key={dll.name} className="rounded-lg border border-line bg-surface-2/50 px-3 py-2.5">
          <div className="flex items-center justify-between gap-2">
            <p className="truncate font-mono text-xs font-semibold text-ink">{dll.name}</p>
            <span className="shrink-0 text-[11px] text-muted">
              {dll.functions.length} {t("analysis.functions").toLowerCase()}
            </span>
          </div>
          {dll.functions.length > 0 && (
            <div className="mt-2 flex flex-wrap gap-1">
              {visible(dll).map((f) => (
                <span key={f} className="rounded border border-line bg-surface px-1.5 py-px font-mono text-[10px] text-muted">
                  {f}
                </span>
              ))}
            </div>
          )}
          {dll.functions.length > 12 && (
            <button
              className="mt-2 text-[11px] font-medium text-accent hover:underline"
              onClick={() => setOpen((s) => ({ ...s, [dll.name]: !s[dll.name] }))}
            >
              {t("analysis.showMore")}
            </button>
          )}
        </div>
      ))}
    </div>
  );
}

const AI_VERDICT_STYLES: Record<AiAssessment["verdict"], string> = {
  clean: "border-good/30 bg-good/15 text-good",
  likely_clean: "border-info/30 bg-info/15 text-info",
  suspicious: "border-warn/30 bg-warn/15 text-warn",
  malicious: "border-bad/30 bg-bad/15 text-bad",
};

const AI_VERDICT_KEYS: Record<AiAssessment["verdict"], TranslationKey> = {
  clean: "analysis.aiVerdict.clean",
  likely_clean: "analysis.aiVerdict.likelyClean",
  suspicious: "analysis.aiVerdict.suspicious",
  malicious: "analysis.aiVerdict.malicious",
};

function AssessmentCard({ data }: { data: AiAssessment }) {
  const { t } = useLanguage();
  const [showDetail, setShowDetail] = useState(true);

  return (
    <Card>
      <CardHeader
        title={t("analysis.aiTitle")}
        action={
          <div className="flex flex-wrap items-center justify-end gap-2">
            <span
              className={`inline-flex items-center rounded-full border px-2.5 py-0.5 text-[11px] font-semibold ${AI_VERDICT_STYLES[data.verdict]}`}
            >
              {t(AI_VERDICT_KEYS[data.verdict])}
            </span>
            <span className="text-[11px] text-muted">
              {t("analysis.aiConfidence")} {Math.round(data.confidence * 100)}%
            </span>
          </div>
        }
      />
      <div className="space-y-4 px-5 py-4">
        <p className="text-sm font-medium leading-relaxed text-ink">{data.summary}</p>

        {data.keyCategories.length > 0 && (
          <div className="flex flex-wrap gap-1.5">
            {data.keyCategories.map((c) => (
              <span
                key={c}
                className="rounded border border-line bg-surface-2 px-2 py-0.5 text-[11px] text-muted"
              >
                {t(CATEGORY_KEYS[c] ?? "rules.category.general")}
              </span>
            ))}
          </div>
        )}

        {data.explanation && (
          <div className="rounded-lg border border-line bg-surface-2/40 px-4 py-3">
            <p className="text-[10px] font-semibold uppercase tracking-wider text-muted">
              {t("analysis.aiExplanation")}
            </p>
            <div className="mt-1.5 space-y-2 text-xs leading-relaxed text-muted">
              {data.explanation.split("\n\n").map((para, i) => (
                <div key={i} className="space-y-0.5">
                  {para.split("\n").map((line, j) => (
                    <p key={j} className={j === 0 ? "font-medium text-ink" : "pl-3 text-muted"}>
                      {line}
                    </p>
                  ))}
                </div>
              ))}
            </div>
          </div>
        )}

        <button
          type="button"
          onClick={() => setShowDetail((s) => !s)}
          className="flex items-center gap-1.5 text-xs font-medium text-accent"
        >
          <ChevronDown className={`size-3.5 transition-transform ${showDetail ? "rotate-180" : ""}`} />
          {t("analysis.aiDetail")}
        </button>

        {showDetail && (
          <div className="grid grid-cols-1 gap-4 md:grid-cols-2">
            {data.indicators.length > 0 && (
              <div>
                <p className="text-[10px] font-semibold uppercase tracking-wider text-muted">
                  {t("analysis.aiIndicators")}
                </p>
                <ul className="mt-1.5 space-y-1">
                  {data.indicators.map((item, idx) => (
                    <li
                      key={idx}
                      className="rounded border border-line bg-surface-2/50 px-2.5 py-1.5 font-mono text-[10px] text-ink/80"
                    >
                      {item}
                    </li>
                  ))}
                </ul>
              </div>
            )}
            {data.attackVectors.length > 0 && (
              <div>
                <p className="text-[10px] font-semibold uppercase tracking-wider text-muted">
                  {t("analysis.aiVectors")}
                </p>
                <ul className="mt-1.5 space-y-1">
                  {data.attackVectors.map((v, idx) => (
                    <li key={idx} className="flex items-start gap-2 text-xs text-ink">
                      <AlertTriangle className="mt-0.5 size-3.5 shrink-0 text-warn" />
                      {v}
                    </li>
                  ))}
                </ul>
              </div>
            )}
            {data.potentialImpact.length > 0 && (
              <div>
                <p className="text-[10px] font-semibold uppercase tracking-wider text-muted">
                  {t("analysis.aiImpact")}
                </p>
                <ul className="mt-1.5 space-y-1">
                  {data.potentialImpact.map((v, idx) => (
                    <li key={idx} className="text-xs leading-relaxed text-ink/80">
                      • {v}
                    </li>
                  ))}
                </ul>
              </div>
            )}
            {data.systemConsequences.length > 0 && (
              <div>
                <p className="text-[10px] font-semibold uppercase tracking-wider text-muted">
                  {t("analysis.aiConsequences")}
                </p>
                <ul className="mt-1.5 space-y-1">
                  {data.systemConsequences.map((v, idx) => (
                    <li key={idx} className="text-xs leading-relaxed text-ink/80">
                      • {v}
                    </li>
                  ))}
                </ul>
              </div>
            )}
            {data.recommendedActions.length > 0 && (
              <div className="md:col-span-2">
                <p className="text-[10px] font-semibold uppercase tracking-wider text-muted">
                  {t("analysis.aiActions")}
                </p>
                <ol className="mt-1.5 space-y-1.5">
                  {data.recommendedActions.map((v, idx) => (
                    <li key={idx} className="flex items-start gap-2 text-xs leading-relaxed text-ink">
                      <span className="mt-px inline-flex size-4 shrink-0 items-center justify-center rounded-full bg-accent/15 text-[10px] font-semibold text-accent">
                        {idx + 1}
                      </span>
                      {v}
                    </li>
                  ))}
                </ol>
              </div>
            )}
          </div>
        )}
      </div>
    </Card>
  );
}

function StaticAnalysisCard({ analysis }: { analysis: StaticAnalysis }) {
  const { t } = useLanguage();
  const [showExports, setShowExports] = useState(false);
  const pe = analysis.pe;
  const exportsList = showExports ? pe?.exports ?? [] : (pe?.exports ?? []).slice(0, 60);

  return (
    <Card>
      <CardHeader title={t("analysis.staticAnalysis")} />
      <div className="px-5 py-4">
        {/* Tipo + entropía */}
        <div className="flex flex-wrap items-start gap-4">
          <div className="min-w-0 flex-1">
            <p className="text-xs text-muted">{t("analysis.fileType")}</p>
            <div className="mt-1 flex flex-wrap items-center gap-2">
              <span className="font-medium text-ink">{analysis.fileType}</span>
              <TypeChip label={analysis.fileTypeExtension} />
              {analysis.isPe ? (
                <TypeChip label={pe?.architecture ?? "PE"} tone={pe?.architecture === "x86" ? "warn" : "good"} />
              ) : null}
            </div>
            <p className="mt-0.5 font-mono text-[11px] text-muted">{analysis.fileTypeMime}</p>
          </div>
          <div className="w-full sm:w-64">
            <p className="mb-1 text-xs text-muted">{t("analysis.fileEntropy")}</p>
            <EntropyBar value={analysis.entropy} />
          </div>
        </div>

        {analysis.isPe && pe ? (
          <>
            <div className="mt-5 flex flex-wrap gap-2">
              {pe.isDll ? (
                <TypeChip label={t("analysis.isDll")} tone="accent" />
              ) : (
                <TypeChip label={t("analysis.isExe")} tone="good" />
              )}
              <TypeChip
                label={pe.isGui ? "GUI" : pe.isConsole ? "Console" : pe.subsystem}
                tone={pe.isConsole ? "accent" : "warn"}
              />
              {pe.hasCertificate ? (
                <TypeChip label={t("analysis.signedYes")} tone="good" />
              ) : (
                <TypeChip label={t("analysis.signedNo")} tone="warn" />
              )}
            </div>

            <dl className="mt-4 grid grid-cols-1 gap-x-6 gap-y-3 text-sm sm:grid-cols-2 lg:grid-cols-3">
              <div>
                <dt className="text-xs text-muted">{t("analysis.architecture")}</dt>
                <dd className="mt-0.5 font-medium text-ink">{pe.architecture}</dd>
              </div>
              <div>
                <dt className="text-xs text-muted">{t("analysis.machine")}</dt>
                <dd className="mt-0.5 font-mono text-xs text-ink">{pe.machine}</dd>
              </div>
              <div>
                <dt className="text-xs text-muted">{t("analysis.subsystem")}</dt>
                <dd className="mt-0.5 text-ink">{pe.subsystem}</dd>
              </div>
              <div>
                <dt className="text-xs text-muted">{t("analysis.entryPoint")}</dt>
                <dd className="mt-0.5 font-mono text-xs text-ink">{HEX(pe.entryPoint)}</dd>
              </div>
              <div>
                <dt className="text-xs text-muted">{t("analysis.imageBase")}</dt>
                <dd className="mt-0.5 font-mono text-xs text-ink">{HEX(pe.imageBase)}</dd>
              </div>
              <div>
                <dt className="text-xs text-muted">{t("analysis.timestamp")}</dt>
                <dd className="mt-0.5 text-ink">
                  {pe.timestampIso ? formatDate(pe.timestampIso) : pe.timestamp}
                </dd>
              </div>
            </dl>

            {/* Secciones */}
            <h4 className="mt-6 text-xs font-semibold uppercase tracking-wider text-muted">
              {t("analysis.sections")} ({pe.sections.length})
            </h4>
            <div className="mt-2">
              <SectionsTable sections={pe.sections} />
            </div>

            {/* Imports */}
            <h4 className="mt-6 text-xs font-semibold uppercase tracking-wider text-muted">
              {t("analysis.imports")} ({pe.imports.length})
            </h4>
            <div className="mt-2">
              {pe.imports.length > 0 ? (
                <ImportsList imports={pe.imports} />
              ) : (
                <p className="text-xs text-muted">{t("common.none")}</p>
              )}
            </div>

            {/* Exports */}
            {pe.exports.length > 0 && (
              <>
                <h4 className="mt-6 text-xs font-semibold uppercase tracking-wider text-muted">
                  {t("analysis.exports")} ({pe.exportCount})
                </h4>
                <div className="mt-2 flex flex-wrap gap-1.5">
                  {exportsList.map((name) => (
                    <span key={name} className="rounded border border-line bg-surface-2 px-2 py-0.5 font-mono text-[11px] text-muted">
                      {name}
                    </span>
                  ))}
                  {!showExports && pe.exports.length > 60 && (
                    <button className="text-[11px] font-medium text-accent hover:underline" onClick={() => setShowExports(true)}>
                      {t("analysis.showMore")}
                    </button>
                  )}
                </div>
              </>
            )}
          </>
        ) : null}
      </div>
    </Card>
  );
}
function FileDetail({ data }: { data: ScanResult }) {
  const { t } = useLanguage();
  const { toast } = useToast();
  const { config } = useConfig();
  const vtHash = data.hashes.sha256 ?? data.hashes.sha1 ?? data.hashes.md5;
  const vtEnabled = config.virustotalEnabled && Boolean(config.virustotalApiKey?.trim());
  const [vtResult, setVtResult] = useState<VirusTotalResult | null>(null);
  const [vtLoading, setVtLoading] = useState(false);
  const [vtError, setVtError] = useState<string | null>(null);
  const [quarantining, setQuarantining] = useState(false);
  const [quarantineNotice, setQuarantineNotice] = useState<{ ok: boolean; text: string } | null>(null);

  const quarantine = async () => {
    if (!window.confirm(t("quarantine.confirmQuarantine").replace("{file}", data.fileName))) return;
    setQuarantining(true);
    setQuarantineNotice(null);
    try {
      await tauri.quarantineFile(data.path, data.threatLevel, t("quarantine.reasonUser"));
      setQuarantineNotice({ ok: true, text: t("quarantine.quarantined") });
      toast(t("quarantine.quarantined"), "success");
    } catch (e) {
      const text = e instanceof Error ? e.message : String(e);
      setQuarantineNotice({ ok: false, text });
      toast(text, "error");
    } finally {
      setQuarantining(false);
    }
  };

  const checkVt = async () => {
    if (!vtHash || vtLoading) return;
    setVtResult(null);
    setVtError(null);
    // Aviso inmediato si VirusTotal no está habilitado: no espera al backend.
    if (!vtEnabled) {
      setVtError(t("analysis.vtNotConfigured"));
      return;
    }
    setVtLoading(true);
    try {
      setVtResult(await tauri.checkVirusTotal(vtHash));
    } catch (e) {
      setVtError(e instanceof Error ? e.message : String(e));
    } finally {
      setVtLoading(false);
    }
  };

  return (
    <div className="grid grid-cols-1 gap-6 xl:grid-cols-3">
      <div className="space-y-6 xl:col-span-2">
        <Card>
          <CardHeader
            title={t("analysis.overview")}
            action={
              <div className="flex flex-wrap items-center justify-end gap-2">
                <Button variant="danger" onClick={() => void quarantine()} disabled={quarantining}>
                  <Archive className="size-4" />
                  {quarantining ? t("common.loading") : t("quarantine.quarantineNow")}
                </Button>
                <LevelBadge level={data.threatLevel} />
              </div>
            }
          />
          <dl className="grid grid-cols-1 gap-x-6 gap-y-3 px-5 py-4 text-sm sm:grid-cols-2">
            <div>
              <dt className="text-xs text-muted">{t("results.fileName")}</dt>
              <dd className="mt-0.5 font-medium text-ink">{data.fileName}</dd>
            </div>
            <div>
              <dt className="text-xs text-muted">{t("common.size")}</dt>
              <dd className="mt-0.5 text-ink">{formatBytes(data.size)}</dd>
            </div>
            <div>
              <dt className="text-xs text-muted">{t("analysis.hasPath")}</dt>
              <dd className="mt-0.5 break-all font-mono text-xs text-ink">{data.path}</dd>
            </div>
            <div>
              <dt className="text-xs text-muted">{t("common.date")}</dt>
              <dd className="mt-0.5 text-ink">{formatDate(data.scannedAt)}</dd>
            </div>
          </dl>
          <div className="border-t border-line px-5 py-4">
            <div className="flex items-center justify-between gap-3">
              <p className="text-xs text-muted">{t("analysis.threatScore")}</p>
              <div className="w-full max-w-sm">
                <ScoreBar score={data.threatScore} level={data.threatLevel} />
              </div>
            </div>
          </div>
          {data.aiAssessment?.verdict === "malicious" &&
          (data.threatLevel === "Clean" || data.threatLevel === "Low" || data.threatLevel === "Medium") &&
          (vtResult ?? data.reputation) ? (
            <div className="border-t border-line px-5 py-3">
              <div className="flex items-start gap-2.5 rounded-lg border border-warn/30 bg-warn/10 px-3 py-2.5">
                <AlertTriangle className="mt-0.5 size-4 shrink-0 text-warn" />
                <div className="min-w-0">
                  <p className="text-xs font-semibold text-ink">{t("analysis.vtMismatch")}</p>
                  <p className="mt-0.5 text-xs leading-relaxed text-ink/80">
                    {t("analysis.vtMismatchDesc")
                      .replace("{level}", data.threatLevel)
                      .replace("{score}", String(data.threatScore))
                      .replace(
                        "{malicious}",
                        String((vtResult ?? data.reputation)!.malicious)
                      )
                      .replace("{total}", String((vtResult ?? data.reputation)!.total))}
                  </p>
                </div>
              </div>
            </div>
          ) : null}
          {quarantineNotice ? (
            <p
              role="status"
              aria-live="polite"
              className={`border-t border-line px-5 py-3 text-xs ${quarantineNotice.ok ? "text-good" : "text-critical"}`}
            >
              {quarantineNotice.text}
            </p>
          ) : null}
        </Card>

        {data.aiAssessment ? <AssessmentCard data={data.aiAssessment} /> : null}

        <Card>
          <CardHeader title={t("analysis.hashes")} />
          <div className="px-5 py-3">
            <HashRow label={t("analysis.md5")} value={data.hashes.md5} />
            <HashRow label={t("analysis.sha1")} value={data.hashes.sha1} />
            <HashRow label={t("analysis.sha256")} value={data.hashes.sha256} />
            {!data.hashes.md5 && !data.hashes.sha1 && !data.hashes.sha256 && (
              <p className="py-3 text-xs text-muted">{t("common.none")}</p>
            )}
          </div>
          {vtHash && (
            <div className="border-t border-line px-5 py-3">
              <div className="flex flex-wrap items-center gap-2">
                <Button variant="secondary" onClick={() => void checkVt()} disabled={vtLoading}>
                  <ExternalLink className="size-4" />
                  {vtLoading ? t("common.loading") : t("analysis.vtCheck")}
                </Button>
              </div>
              {vtError ? (
                <div
                  role="status"
                  className="mt-3 flex flex-wrap items-start justify-between gap-3 rounded-lg border border-warn/30 bg-warn/10 px-3 py-2.5"
                >
                  <p className="min-w-0 flex-1 text-xs leading-relaxed text-ink">{vtError}</p>
                  {!vtEnabled ? (
                    <Link to="/settings">
                      <Button variant="secondary">{t("analysis.vtGoSettings")}</Button>
                    </Link>
                  ) : null}
                </div>
              ) : null}
            </div>
          )}
        </Card>

        {data.reputation || vtResult ? (
          <ReputationCard
            data={vtResult ?? data.reputation!}
            onRetry={vtHash ? () => void checkVt() : undefined}
          />
        ) : null}

        {data.staticAnalysis ? <StaticAnalysisCard analysis={data.staticAnalysis} /> : null}

        <Card>
          <CardHeader
            title={t("analysis.findings")}
            action={
              data.findings.length > 0 ? (
                <span className="text-xs font-semibold text-ink">{data.findings.length}</span>
              ) : undefined
            }
          />
          {data.findings.length === 0 ? (
            <EmptyState
              icon={ShieldCheck}
              title={t("analysis.noFindings")}
              description={t("analysis.noFindingsDesc")}
            />
          ) : (
            <FindingsList findings={data.findings} />
          )}
        </Card>
      </div>

      <Card>
        <CardHeader title={t("analysis.timeline")} />
        <ol className="px-5 py-4">
          {data.timeline.map((entry, i) => (
            <li key={i} className="relative flex gap-3 pb-4 last:pb-0">
              {i < data.timeline.length - 1 && (
                <span className="absolute left-[5px] top-4 h-full w-px bg-line" />
              )}
              <span className="relative mt-1 size-2.5 shrink-0 rounded-full bg-accent ring-2 ring-accent/20" />
              <div className="min-w-0">
                <p className="text-xs font-medium text-ink">{entry.label}</p>
                <p className="text-[11px] text-muted">{entry.time}</p>
              </div>
            </li>
          ))}
          {data.timeline.length === 0 && (
            <p className="py-3 text-xs text-muted">{t("common.none")}</p>
          )}
        </ol>
      </Card>
    </div>
  );
}

function FolderDetail({ data }: { data: FolderScanResult }) {
  const { t } = useLanguage();

  const stats = [
    { label: t("analysis.files"), value: data.fileCount },
    { label: t("analysis.scanned"), value: data.scannedCount },
    { label: t("analysis.skipped"), value: data.skippedCount },
    { label: t("analysis.withErrors"), value: data.errorCount },
  ];

  return (
    <div className="space-y-6">
      <Card>
        <CardHeader title={t("analysis.overview")} action={<LevelBadge level="Clean" />} />
        <div className="px-5 py-4">
          <p className="break-all font-mono text-xs text-muted">{data.folderPath}</p>
          <div className="mt-4 grid grid-cols-2 gap-3 sm:grid-cols-4">
            {stats.map((s) => (
              <div key={s.label} className="rounded-lg border border-line bg-surface-2/50 px-3 py-2.5">
                <p className="text-lg font-semibold text-ink">{s.value}</p>
                <p className="text-[11px] text-muted">{s.label}</p>
              </div>
            ))}
          </div>
          <div className="mt-3 grid grid-cols-2 gap-3">
            <div className="rounded-lg border border-line bg-surface-2/50 px-3 py-2.5">
              <p className="text-lg font-semibold text-ink">{formatBytes(data.totalBytes)}</p>
              <p className="text-[11px] text-muted">{t("analysis.totalBytes")}</p>
            </div>
            <div className="rounded-lg border border-line bg-surface-2/50 px-3 py-2.5">
              <p className="text-lg font-semibold text-ink">{formatDurationMs(data.durationMs)}</p>
              <p className="text-[11px] text-muted">{t("analysis.duration")}</p>
            </div>
          </div>
        </div>
      </Card>

      <Card>
        <CardHeader title={t("analysis.fileList")} />
        <div className="overflow-x-auto px-5 py-4">
          <table className="w-full text-left text-sm">
            <thead>
              <tr className="border-b border-line text-xs uppercase tracking-wider text-muted">
                <th className="pb-2 pr-4 font-semibold">{t("analysis.relativePath")}</th>
                <th className="pb-2 pr-4 font-semibold">{t("common.size")}</th>
                <th className="pb-2 pr-4 font-semibold">MD5</th>
                <th className="pb-2 pr-4 font-semibold">SHA-1</th>
                <th className="pb-2 pr-4 font-semibold">SHA-256</th>
                <th className="pb-2 font-semibold">{t("analysis.error")}</th>
              </tr>
            </thead>
            <tbody>
              {data.files.map((file) => (
                <tr key={file.relativePath} className="border-b border-line last:border-0">
                  <td className="max-w-[260px] py-2 pr-4">
                    <p className="truncate text-xs text-ink" title={file.relativePath}>
                      {file.relativePath}
                    </p>
                  </td>
                  <td className="py-2 pr-4 text-xs text-muted">{formatBytes(file.size)}</td>
                  <td className="py-2 pr-4 font-mono text-[11px] text-muted">{shortenHash(file.hashes.md5)}</td>
                  <td className="py-2 pr-4 font-mono text-[11px] text-muted">{shortenHash(file.hashes.sha1)}</td>
                  <td className="py-2 pr-4 font-mono text-[11px] text-muted">{shortenHash(file.hashes.sha256)}</td>
                  <td className="py-2 text-xs">
                    {file.error ? (
                      <span className="inline-flex items-center gap-1 text-critical">
                        <AlertTriangle className="size-3.5" />
                        {file.error}
                      </span>
                    ) : (
                      <span className="text-muted">—</span>
                    )}
                  </td>
                </tr>
              ))}
              {data.files.length === 0 && (
                <tr>
                  <td colSpan={6} className="p-0">
                    <EmptyState icon={FolderOpen} title={t("results.empty")} />
                  </td>
                </tr>
              )}
            </tbody>
          </table>
        </div>
      </Card>
    </div>
  );
}

export function Analysis() {
  const { t } = useLanguage();
  const { toast } = useToast();
  const navigate = useNavigate();
  const { id } = useParams<{ id: string }>();
  const [loaded, setLoaded] = useState<Loaded | null>(null);
  const [missing, setMissing] = useState(false);
  const [loadError, setLoadError] = useState<string | null>(null);
  const [exporting, setExporting] = useState(false);
  const [preview, setPreview] = useState<{ format: ReportFormat; content: string } | null>(null);
  const previewCloseRef = useRef<HTMLButtonElement>(null);

  useEffect(() => {
    if (!id) return;
    setLoaded(null);
    setMissing(false);
    setLoadError(null);
    void tauri
      .getAnalysisById(id)
      .then((result) => {
        if (!result) {
          setMissing(true);
          return;
        }
        setLoaded(
          "folderPath" in result ? { kind: "folder", data: result } : { kind: "file", data: result },
        );
      })
      .catch((e) => setLoadError(e instanceof Error ? e.message : String(e)));
  }, [id]);

  useEffect(() => {
    if (!preview) return;
    const previous = document.activeElement as HTMLElement | null;
    previewCloseRef.current?.focus();
    const onKey = (e: KeyboardEvent) => {
      if (e.key === "Escape") setPreview(null);
    };
    window.addEventListener("keydown", onKey);
    return () => {
      window.removeEventListener("keydown", onKey);
      previous?.focus?.();
    };
  }, [preview]);

  const exportReport = async (format: ReportFormat) => {
    if (!id) return;
    try {
      const path = await save({
        defaultPath: `prometeo-report-${id.slice(0, 8)}.${format}`,
        filters: [{ name: format.toUpperCase(), extensions: [format] }],
      });
      if (!path) return;
      const target = Array.isArray(path) ? path[0] : path;
      if (!target) return;
      setExporting(true);
      const saved = await tauri.exportReport(id, format, target);
      toast(t("report.saved").replace("{path}", saved), "success");
    } catch (e) {
      toast(e instanceof Error ? e.message : String(e), "error");
    } finally {
      setExporting(false);
    }
  };

  const openPreview = async (format: ReportFormat) => {
    if (!id) return;
    try {
      setPreview({ format, content: await tauri.previewReport(id, format) });
    } catch (e) {
      toast(e instanceof Error ? e.message : String(e), "error");
    }
  };

  return (
    <div>
      <PageHeader
        title={t("analysis.title")}
        subtitle={t("analysis.subtitle")}
        actions={
          <div className="flex items-center gap-2">
            {loaded && id ? (
              <>
                <Button variant="ghost" onClick={() => void openPreview("html")} title={t("report.preview")}>
                  <Eye className="size-4" />
                  {t("report.preview")}
                </Button>
                <Button
                  variant="secondary"
                  disabled={exporting}
                  onClick={() => void exportReport("html")}
                  title={t("report.exportHtml")}
                >
                  <FileCode2 className="size-4" />
                  {t("report.exportHtml")}
                </Button>
                <Button
                  variant="secondary"
                  disabled={exporting}
                  onClick={() => void exportReport("csv")}
                  title={t("report.exportCsv")}
                >
                  <FileSpreadsheet className="size-4" />
                  {t("report.exportCsv")}
                </Button>
              </>
            ) : null}
            {id ? (
              <Button variant="ghost" onClick={() => navigate(-1)}>
                <ArrowLeft className="size-4" />
                {t("results.title")}
              </Button>
            ) : undefined}
          </div>
        }
      />

      {!id ? (
        <Card>
          <EmptyState icon={ListTree} title={t("analysis.noSelection")} description={t("results.empty")} />
        </Card>
      ) : missing ? (
        <Card>
          <EmptyState icon={FileSearch} title={t("analysis.notFound")}>
            <Link to="/results">
              <Button variant="secondary">{t("results.title")}</Button>
            </Link>
          </EmptyState>
        </Card>
      ) : loadError ? (
        <Card>
          <EmptyState icon={AlertTriangle} title={t("common.error")} description={loadError}>
            <Link to="/results">
              <Button variant="secondary">{t("results.title")}</Button>
            </Link>
          </EmptyState>
        </Card>
      ) : loaded ? (
        loaded.kind === "file" ? (
          <FileDetail data={loaded.data} />
        ) : (
          <FolderDetail data={loaded.data} />
        )
      ) : (
        <Card>
          <p className="py-8 text-center text-sm text-muted">{t("common.loading")}</p>
        </Card>
      )}
      {preview ? (
        <div
          role="dialog"
          aria-modal="true"
          aria-label={t("report.previewTitle")}
          className="fixed inset-0 z-50 flex items-center justify-center bg-black/60 p-6"
          onClick={() => setPreview(null)}
        >
          <div
            className="flex h-[85vh] w-full max-w-4xl animate-va-pop flex-col overflow-hidden rounded-xl border border-line bg-surface shadow-2xl"
            onClick={(e) => e.stopPropagation()}
          >
            <div className="flex items-center justify-between border-b border-line px-4 py-3">
              <p className="text-sm font-medium text-ink">
                {t("report.previewTitle")} · {preview.format.toUpperCase()}
              </p>
              <Button variant="ghost" ref={previewCloseRef} onClick={() => setPreview(null)}>
                {t("common.close")}
              </Button>
            </div>
            <div className="flex-1 overflow-auto bg-white">
              {preview.format === "html" ? (
                <iframe
                  title="report-preview"
                  sandbox=""
                  srcDoc={preview.content}
                  className="h-full w-full border-0"
                />
              ) : (
                <pre className="whitespace-pre-wrap p-4 font-mono text-xs text-gray-900">{preview.content}</pre>
              )}
            </div>
          </div>
        </div>
      ) : null}
    </div>
  );
}
