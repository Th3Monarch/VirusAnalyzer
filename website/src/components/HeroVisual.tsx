import { useLanguage } from "../contexts/LanguageContext";
import type { Lang } from "../lib/i18n";

type SeverityKey = "High" | "Medium" | "Low" | "External";

interface Detection {
  label: string;
  severity: SeverityKey;
}

interface Content {
  windowTitle: string;
  fileMeta: string;
  verdict: string;
  threatScore: string;
  scoreMeta: string;
  detections: Detection[];
  footnote: string;
  caption: string;
}

const content: Record<Lang, Content> = {
  en: {
    windowTitle: "static-analysis",
    fileMeta: "PE32+ · x64 · 1.2 MB",
    verdict: "Suspicious",
    threatScore: "Threat score",
    scoreMeta: "Level: High · 28 heuristic rules",
    detections: [
      { label: "Imports: VirtualAlloc with RWX protection", severity: "High" },
      { label: "Suspicious string: powershell -enc", severity: "Medium" },
      { label: "Type mismatch: PE disguised as .pdf", severity: "Medium" },
      { label: "High entropy .text section", severity: "Low" },
      { label: "VirusTotal: 3 / 72 engines flag it", severity: "External" },
    ],
    footnote: "Static analysis · no execution",
    caption: "Conceptual illustration",
  },
  es: {
    windowTitle: "analisis-estatico",
    fileMeta: "PE32+ · x64 · 1,2 MB",
    verdict: "Sospechoso",
    threatScore: "Puntuación de amenaza",
    scoreMeta: "Nivel: Alto · 28 reglas heurísticas",
    detections: [
      { label: "Imports: VirtualAlloc con protección RWX", severity: "High" },
      { label: "Cadena sospechosa: powershell -enc", severity: "Medium" },
      { label: "Tipo no coincidente: PE disfrazado de .pdf", severity: "Medium" },
      { label: "Sección .text con alta entropía", severity: "Low" },
      { label: "VirusTotal: 3 / 72 motores lo marcan", severity: "External" },
    ],
    footnote: "Análisis estático · sin ejecución",
    caption: "Ilustración conceptual",
  },
};

const severityLabels: Record<Lang, Record<SeverityKey, string>> = {
  en: { High: "High", Medium: "Medium", Low: "Low", External: "External" },
  es: { High: "Alto", Medium: "Medio", Low: "Bajo", External: "Externo" },
};

const severityStyles: Record<SeverityKey, string> = {
  High: "bg-red-500/10 text-red-400 border-red-500/30",
  Medium: "bg-amber-400/10 text-amber-400 border-amber-400/30",
  Low: "bg-emerald-400/10 text-emerald-400 border-emerald-400/30",
  External: "bg-sky-400/10 text-sky-400 border-sky-400/30",
};

export function HeroVisual() {
  const { lang } = useLanguage();
  const c = content[lang];

  return (
    <div className="relative">
      <div className="pointer-events-none absolute -inset-6 rounded-3xl bg-sky-400/5 blur-2xl" aria-hidden="true" />

      <div className="relative overflow-hidden rounded-xl border border-line bg-ink shadow-2xl shadow-black/40">
        <div className="flex items-center justify-between border-b border-line px-4 py-3">
          <div className="flex items-center gap-2">
            <span className="h-2.5 w-2.5 rounded-full bg-red-500/70" aria-hidden="true" />
            <span className="h-2.5 w-2.5 rounded-full bg-amber-400/70" aria-hidden="true" />
            <span className="h-2.5 w-2.5 rounded-full bg-emerald-400/70" aria-hidden="true" />
          </div>
          <p className="font-mono text-xs text-zinc-500">{c.windowTitle}</p>
        </div>

        <div className="p-5">
          <div className="flex items-center justify-between gap-3">
            <div>
              <p className="text-sm font-semibold text-white">sample_archive.exe</p>
              <p className="font-mono text-xs text-zinc-500">{c.fileMeta}</p>
            </div>
            <span className="rounded-md border border-amber-400/30 bg-amber-400/10 px-2 py-0.5 text-xs font-semibold text-amber-400">
              {c.verdict}
            </span>
          </div>

          <div className="mt-4">
            <div className="flex items-center justify-between text-xs">
              <span className="text-zinc-400">{c.threatScore}</span>
              <span className="font-mono text-zinc-300">
                72<span className="text-zinc-500">/100</span>
              </span>
            </div>
            <div
              className="mt-1.5 h-1.5 overflow-hidden rounded-full bg-ink-3"
              role="presentation"
            >
              <div className="h-full w-[72%] rounded-full bg-gradient-to-r from-amber-400 to-red-500" />
            </div>
            <p className="mt-1 text-xs text-zinc-500">{c.scoreMeta}</p>
          </div>

          <div className="mt-4 space-y-1.5 rounded-lg border border-line bg-night p-3">
            <p className="font-mono text-[11px] text-zinc-500">
              sha256: <span className="text-zinc-300">9f86d08…e4b3c2</span>
            </p>
            <p className="font-mono text-[11px] text-zinc-500">
              md5: <span className="text-zinc-300">e99a18c…4b7</span>
            </p>
            <p className="font-mono text-[11px] text-zinc-500">
              sha1: <span className="text-zinc-300">4e1243b…29f</span>
            </p>
          </div>

          <ul className="mt-4 space-y-2">
            {c.detections.map((item) => (
              <li key={item.label} className="flex items-center justify-between gap-3 text-xs">
                <span className="text-zinc-400">{item.label}</span>
                <span
                  className={`shrink-0 rounded border px-1.5 py-0.5 text-[10px] font-semibold ${
                    severityStyles[item.severity]
                  }`}
                >
                  {severityLabels[lang][item.severity]}
                </span>
              </li>
            ))}
          </ul>

          <div className="mt-4 flex items-center justify-between border-t border-line pt-3 text-[11px] text-zinc-500">
            <span className="inline-flex items-center gap-1.5">
              <span className="h-1.5 w-1.5 rounded-full bg-emerald-400" aria-hidden="true" />
              {c.footnote}
            </span>
            <span>{c.caption}</span>
          </div>
        </div>
      </div>
    </div>
  );
}
