import {
  Archive,
  Brain,
  FileSearch,
  FileText,
  Gauge,
  Globe,
  Hash,
  History,
  MousePointerClick,
  ScanSearch,
  Terminal,
  type LucideIcon,
} from "lucide-react";
import { Seo } from "../components/Seo";
import { Reveal } from "../components/Reveal";
import { useLanguage } from "../contexts/LanguageContext";
import type { Lang } from "../lib/i18n";

interface LocalizedText {
  en: string;
  es: string;
}

interface Feature {
  icon: LucideIcon;
  title: LocalizedText;
  description: LocalizedText;
  points: Record<Lang, string[]>;
}

const features: Feature[] = [
  {
    icon: FileSearch,
    title: { en: "Static Analysis", es: "Análisis estático" },
    description: {
      en: "Inspects suspicious files without automatically executing them.",
      es: "Inspecciona archivos sospechosos sin ejecutarlos automáticamente.",
    },
    points: {
      en: [
        "File type detection by magic bytes",
        "Shannon entropy analysis (global and per PE section)",
        "PE parser: headers, architecture, imports, exports",
        "Authenticode certificate detection",
        "Suspicious string analysis on non-executable content",
      ],
      es: [
        "Detección del tipo de archivo por magic bytes",
        "Análisis de entropía de Shannon (global y por sección PE)",
        "Parser PE: cabeceras, arquitectura, imports, exports",
        "Detección de certificados Authenticode",
        "Análisis de cadenas sospechosas en contenido no ejecutable",
      ],
    },
  },
  {
    icon: ScanSearch,
    title: { en: "File & Folder Scanning", es: "Análisis de archivos y carpetas" },
    description: {
      en: "Analyze a single file or a whole folder with live progress and cancellation.",
      es: "Analiza un único archivo o una carpeta completa con progreso en vivo y cancelación.",
    },
    points: {
      en: [
        "Drag & drop support",
        "Streaming hashing with configurable hash set",
        "Recursive folder scans with progress events",
        "Cancellable at any time",
      ],
      es: [
        "Soporte de arrastrar y soltar",
        "Hashing por streaming con conjunto de hashes configurable",
        "Análisis recursivo de carpetas con eventos de progreso",
        "Cancelable en cualquier momento",
      ],
    },
  },
  {
    icon: Gauge,
    title: { en: "Threat Assessment", es: "Evaluación de amenazas" },
    description: {
      en: "A heuristic engine scores files using static indicators and explains every finding.",
      es: "Un motor heurístico puntúa archivos usando indicadores estáticos y explica cada hallazgo.",
    },
    points: {
      en: [
        "28 rules across 7 categories",
        "Score from 0 to 100 with clear levels",
        "Findings include severity, category and evidence",
        "Rule catalog browsable in the app",
      ],
      es: [
        "28 reglas en 7 categorías",
        "Puntuación de 0 a 100 con niveles claros",
        "Los hallazgos incluyen severidad, categoría y evidencia",
        "Catálogo de reglas navegable en la app",
      ],
    },
  },
  {
    icon: Hash,
    title: { en: "Hash Analysis", es: "Análisis de hashes" },
    description: {
      en: "Generate and analyze MD5, SHA-1 and SHA-256 hashes of any file.",
      es: "Genera y analiza los hashes MD5, SHA-1 y SHA-256 de cualquier archivo.",
    },
    points: {
      en: [
        "Streaming hash computation",
        "Select which algorithms to compute",
        "Copy hashes with one click",
      ],
      es: [
        "Cálculo de hashes por streaming",
        "Elige qué algoritmos calcular",
        "Copia los hashes con un clic",
      ],
    },
  },
  {
    icon: Globe,
    title: { en: "VirusTotal Integration", es: "Integración con VirusTotal" },
    description: {
      en: "Optional reputation checks using file hashes — never file content.",
      es: "Comprobaciones de reputación opcionales usando hashes del archivo — nunca su contenido.",
    },
    points: {
      en: [
        "Opt-in: disabled by default",
        "Only MD5 / SHA-1 / SHA-256 are sent",
        "Engine counts, threat names and detection permalink",
        "Manual hash lookup from the analysis view",
      ],
      es: [
        "Consentimiento previo: desactivada por defecto",
        "Solo se envían MD5 / SHA-1 / SHA-256",
        "Número de motores, nombres de amenaza y enlace del detalle",
        "Consulta manual de hashes desde la vista de análisis",
      ],
    },
  },
  {
    icon: Brain,
    title: { en: "AI-Assisted Assessment", es: "Evaluación asistida por IA" },
    description: {
      en: "A local, deterministic engine summarizes the evidence in natural language.",
      es: "Un motor local y determinista resume la evidencia en lenguaje natural.",
    },
    points: {
      en: [
        "No external AI service, no network",
        "Never invents results: every claim comes from real data",
        "Verdict, confidence and per-category explanation",
        "Available in English and Spanish",
      ],
      es: [
        "Sin servicio de IA externo, sin red",
        "Nunca inventa resultados: cada afirmación viene de datos reales",
        "Veredicto, confianza y explicación por categoría",
        "Disponible en español e inglés",
      ],
    },
  },
  {
    icon: Archive,
    title: { en: "Quarantine", es: "Cuarentena" },
    description: {
      en: "Isolate suspicious files when explicitly requested by the user.",
      es: "Aísla archivos sospechosos cuando el usuario lo pide explícitamente.",
    },
    points: {
      en: [
        "Moves the file to an isolated folder",
        "Restore without overwriting existing paths",
        "Permanent deletion with full tracking",
        "Never quarantines automatically",
      ],
      es: [
        "Mueve el archivo a una carpeta aislada",
        "Restauración sin sobrescribir rutas existentes",
        "Eliminación permanente con registro completo",
        "Nunca pone en cuarentena automáticamente",
      ],
    },
  },
  {
    icon: History,
    title: { en: "Analysis History", es: "Historial de análisis" },
    description: {
      en: "Keep track of previous analyses across sessions.",
      es: "Guarda el registro de análisis anteriores entre sesiones.",
    },
    points: {
      en: [
        "Persistent history that survives restarts",
        "Search and open past results",
        "Full results indexed by stable id",
      ],
      es: [
        "Historial persistente que sobrevive a los reinicios",
        "Busca y abre resultados anteriores",
        "Resultados completos indexados por id estable",
      ],
    },
  },
  {
    icon: FileText,
    title: { en: "Reports", es: "Informes" },
    description: {
      en: "Generate analysis reports to share or archive.",
      es: "Genera informes de análisis para compartir o archivar.",
    },
    points: {
      en: [
        "Self-contained HTML reports",
        "RFC 4180 CSV export",
        "Preview before saving",
      ],
      es: [
        "Informes HTML autocontenidos",
        "Exportación CSV RFC 4180",
        "Vista previa antes de guardar",
      ],
    },
  },
  {
    icon: Terminal,
    title: { en: "Terminal Tools", es: "Herramientas de terminal" },
    description: {
      en: "Integrated terminal adapted to your platform: PowerShell on Windows, POSIX shell on macOS and Linux.",
      es: "Terminal integrada adaptada a tu plataforma: PowerShell en Windows, shell POSIX en macOS y Linux.",
    },
    points: {
      en: [
        "Executed only by explicit user action",
        "High-risk commands require confirmation",
        "Educational command reference included",
      ],
      es: [
        "Se ejecuta solo por acción explícita del usuario",
        "Los comandos de alto riesgo requieren confirmación",
        "Incluye referencia educativa de comandos",
      ],
    },
  },
  {
    icon: MousePointerClick,
    title: { en: "Context Menu Integration", es: "Integración en el menú contextual" },
    description: {
      en: "Optional “Analyze with VirusAnalyzer” entry in Windows Explorer.",
      es: "Entrada opcional “Analizar con VirusAnalyzer” en el Explorador de Windows.",
    },
    points: {
      en: [
        "Applies to files and folders",
        "Current user only, no admin rights required",
        "Toggle from the settings",
      ],
      es: [
        "Se aplica a archivos y carpetas",
        "Solo para el usuario actual, sin permisos de administrador",
        "Activable desde la configuración",
      ],
    },
  },
];

const copy = {
  en: {
    eyebrow: "Features",
    title: "Powerful analysis. Simple interface.",
    intro:
      "VirusAnalyzer helps you understand suspicious files by combining static analysis, an explainable heuristic engine and optional reputation data — all in a cross-platform desktop application for Windows, macOS and Linux.",
  },
  es: {
    eyebrow: "Características",
    title: "Análisis potente. Interfaz sencilla.",
    intro:
      "VirusAnalyzer te ayuda a entender archivos sospechosos combinando análisis estático, un motor heurístico explicable y datos de reputación opcionales — todo en una aplicación de escritorio multiplataforma para Windows, macOS y Linux.",
  },
} satisfies Record<"en" | "es", Record<string, string>>;

export function Features() {
  const { lang, t } = useLanguage();
  const c = copy[lang];

  return (
    <>
      <Seo title={t("seo.features.title")} description={t("seo.features.description")} path="/features" />

      <section className="mx-auto max-w-6xl px-4 py-16 sm:px-6">
        <p className="text-sm font-semibold text-sky-400">{c.eyebrow}</p>
        <h1 className="mt-2 max-w-3xl text-4xl font-bold tracking-tight text-white">
          {c.title}
        </h1>
        <p className="mt-4 max-w-2xl text-base leading-relaxed text-zinc-400">
          {c.intro}
        </p>

        <div className="mt-12 grid gap-6 md:grid-cols-2">
          {features.map((feature, index) => (
            <Reveal key={feature.title[lang]} delay={(index % 2) * 60}>
              <article className="h-full rounded-xl border border-line bg-ink p-6 transition-colors hover:border-line-2">
                <div className="flex items-start gap-4">
                  <div className="flex h-10 w-10 shrink-0 items-center justify-center rounded-lg border border-line-2 bg-ink-2">
                    <feature.icon className="h-5 w-5 text-sky-400" aria-hidden="true" />
                  </div>
                  <div>
                    <h2 className="text-base font-semibold text-white">
                      {feature.title[lang]}
                    </h2>
                    <p className="mt-1.5 text-sm leading-relaxed text-zinc-400">
                      {feature.description[lang]}
                    </p>
                  </div>
                </div>
                <ul className="mt-4 space-y-2 border-t border-line pt-4">
                  {feature.points[lang].map((point) => (
                    <li
                      key={point}
                      className="flex items-start gap-2 text-sm text-zinc-400"
                    >
                      <span
                        className="mt-1.5 h-1.5 w-1.5 shrink-0 rounded-full bg-sky-400"
                        aria-hidden="true"
                      />
                      {point}
                    </li>
                  ))}
                </ul>
              </article>
            </Reveal>
          ))}
        </div>
      </section>
    </>
  );
}
