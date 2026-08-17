import { Link } from "react-router-dom";
import {
  ArrowRight,
  Archive,
  Brain,
  FileSearch,
  Gauge,
  Globe,
  Hash,
  Lock,
  MousePointerClick,
  ScanSearch,
  ShieldCheck,
  ListChecks,
  Download,
  type LucideIcon,
} from "lucide-react";
import { Seo } from "../components/Seo";
import { Button } from "../components/Button";
import { HeroVisual } from "../components/HeroVisual";
import { Reveal } from "../components/Reveal";
import { useLatestRelease } from "../hooks/useLatestRelease";
import { useLanguage } from "../contexts/LanguageContext";
import { github, site } from "../config";

interface LocalizedText {
  en: string;
  es: string;
}

interface Feature {
  icon: LucideIcon;
  title: LocalizedText;
  text: LocalizedText;
}

const featurePreview: Feature[] = [
  {
    icon: FileSearch,
    title: { en: "Static Analysis", es: "Análisis estático" },
    text: {
      en: "Inspects suspicious files without automatically executing them.",
      es: "Inspecciona archivos sospechosos sin ejecutarlos automáticamente.",
    },
  },
  {
    icon: Gauge,
    title: { en: "Threat Assessment", es: "Evaluación de amenazas" },
    text: {
      en: "Scores files with 28 heuristic rules across seven categories.",
      es: "Puntúa archivos con 28 reglas heurísticas en siete categorías.",
    },
  },
  {
    icon: Hash,
    title: { en: "Hash Analysis", es: "Análisis de hashes" },
    text: {
      en: "Generates MD5, SHA-1 and SHA-256 hashes of any file.",
      es: "Genera hashes MD5, SHA-1 y SHA-256 de cualquier archivo.",
    },
  },
  {
    icon: Globe,
    title: { en: "VirusTotal Integration", es: "Integración con VirusTotal" },
    text: {
      en: "Optional reputation checks by hash — never by file content.",
      es: "Comprobaciones de reputación opcionales por hash — nunca por contenido.",
    },
  },
  {
    icon: Brain,
    title: { en: "AI-Assisted Assessment", es: "Evaluación asistida por IA" },
    text: {
      en: "Local, deterministic evidence-based explanation of every result.",
      es: "Explicación local y determinista de cada resultado basada en evidencias.",
    },
  },
  {
    icon: Archive,
    title: { en: "Quarantine", es: "Cuarentena" },
    text: {
      en: "Isolate, restore or delete suspicious files on explicit request.",
      es: "Aísla, restaura o elimina archivos sospechosos bajo petición explícita.",
    },
  },
];

interface Step {
  icon: LucideIcon;
  title: LocalizedText;
  text: LocalizedText;
}

const steps: Step[] = [
  {
    icon: MousePointerClick,
    title: { en: "Select a file", es: "Selecciona un archivo" },
    text: {
      en: "Pick a file or folder from your system — or drag and drop it in.",
      es: "Elige un archivo o carpeta de tu sistema — o arrástralo y suéltalo.",
    },
  },
  {
    icon: ScanSearch,
    title: { en: "Analyze", es: "Analiza" },
    text: {
      en: "VirusAnalyzer performs a static analysis: type, entropy, PE structure, imports and strings.",
      es: "VirusAnalyzer realiza un análisis estático: tipo, entropía, estructura PE, imports y cadenas.",
    },
  },
  {
    icon: ListChecks,
    title: { en: "Review indicators", es: "Revisa los indicadores" },
    text: {
      en: "A heuristic engine turns raw evidence into findings with severities and an explainable score.",
      es: "Un motor heurístico convierte la evidencia en hallazgos con severidades y una puntuación explicable.",
    },
  },
  {
    icon: Globe,
    title: { en: "Check reputation", es: "Comprueba la reputación" },
    text: {
      en: "Optionally compare the file hashes against VirusTotal. Only hashes leave your machine.",
      es: "Opcionalmente compara los hashes del archivo con VirusTotal. Solo los hashes salen de tu equipo.",
    },
  },
  {
    icon: ShieldCheck,
    title: { en: "Decide what to do", es: "Decide qué hacer" },
    text: {
      en: "Read the assessment, export a report, or quarantine the file when you choose to.",
      es: "Lee la evaluación, exporta un informe o pon el archivo en cuarentena cuando lo decidas.",
    },
  },
];

interface SecurityPoint {
  icon: LucideIcon;
  title: LocalizedText;
  text: LocalizedText;
}

const securityPoints: SecurityPoint[] = [
  {
    icon: ShieldCheck,
    title: { en: "Static-first analysis", es: "Análisis estático primero" },
    text: {
      en: "Files are never executed automatically.",
      es: "Los archivos nunca se ejecutan automáticamente.",
    },
  },
  {
    icon: Globe,
    title: { en: "Hash-only reputation", es: "Reputación solo por hash" },
    text: {
      en: "VirusTotal is optional, opt-in and receives only hashes.",
      es: "VirusTotal es opcional, requiere consentimiento y solo recibe hashes.",
    },
  },
  {
    icon: Lock,
    title: { en: "User-controlled terminal", es: "Terminal controlado por el usuario" },
    text: {
      en: "Terminal commands run only on explicit user request.",
      es: "Los comandos del terminal se ejecutan solo bajo petición explícita del usuario.",
    },
  },
  {
    icon: Hash,
    title: { en: "Checksums & transparency", es: "Checksums y transparencia" },
    text: {
      en: "Releases ship SHA-256 checksums for verification.",
      es: "Las versiones incluyen checksums SHA-256 para su verificación.",
    },
  },
];

const copy: Record<"en" | "es", Record<string, LocalizedText["en"]>> = {
  en: {
    heroText:
      "A cross-platform malware analysis and threat assessment tool designed to help you understand suspicious files before making a decision.",
    downloadForWindows: "Download for your platform",
    viewOnGitHub: "View on GitHub",
    readDocs: "Read the documentation",
    metaLine: "Windows 10+ / macOS 10.15+ / Linux · Open source",
    featuresTitle: "Powerful analysis. Simple interface.",
    featuresSub:
      "Everything you need to understand a file before you trust it — in a desktop application for Windows, macOS and Linux.",
    exploreAll: "Explore all features",
    howTitle: "How it works",
    howSub:
      "A guided workflow that turns a suspicious file into an understandable verdict — using static analysis, not execution.",
    securityTitle: "Built with security in mind",
    securityCta: "Learn about our security model",
    ctaTitle: "Ready to analyze?",
    ctaText:
      "Download and take a closer look at the files you are unsure about.",
    seeWhat: "See what it can do",
  },
  es: {
    heroText:
      "Una herramienta de análisis de malware y evaluación de amenazas multiplataforma, diseñada para ayudarte a entender archivos sospechosos antes de tomar una decisión.",
    downloadForWindows: "Descargar para tu plataforma",
    viewOnGitHub: "Ver en GitHub",
    readDocs: "Leer la documentación",
    metaLine: "Windows 10+ / macOS 10.15+ / Linux · Código abierto",
    featuresTitle: "Análisis potente. Interfaz sencilla.",
    featuresSub:
      "Todo lo que necesitas para entender un archivo antes de confiar en él — en una aplicación de escritorio para Windows, macOS y Linux.",
    exploreAll: "Explora todas las características",
    howTitle: "Cómo funciona",
    howSub:
      "Un flujo guiado que convierte un archivo sospechoso en un veredicto comprensible — usando análisis estático, no ejecución.",
    securityTitle: "Construido con la seguridad en mente",
    securityCta: "Conoce nuestro modelo de seguridad",
    ctaTitle: "¿Listo para analizar?",
    ctaText:
      "Descárgalo y echa un vistazo de cerca a los archivos de los que no estás seguro.",
    seeWhat: "Mira lo que puede hacer",
  },
};

export function Home() {
  const { lang, t } = useLanguage();
  const release = useLatestRelease();
  const version =
    release.status === "ready" && release.bundle
      ? release.bundle.version
      : site.fallbackVersion;
  const c = copy[lang];

  return (
    <>
      <Seo title={t("seo.home.title")} description={t("seo.home.description")} path="/" />

      <section className="relative overflow-hidden">
        <div
          className="pointer-events-none absolute -top-40 left-1/2 h-96 w-[42rem] -translate-x-1/2 rounded-full bg-sky-400/10 blur-3xl"
          aria-hidden="true"
        />
        <div className="relative mx-auto grid max-w-6xl items-center gap-14 px-4 py-20 sm:px-6 lg:grid-cols-2 lg:py-28">
          <div>
            <p className="inline-flex flex-wrap items-center gap-x-3 gap-y-1 rounded-full border border-line-2 bg-ink-2 px-3 py-1 text-xs text-zinc-400">
              <span className="font-semibold text-sky-400">Windows · macOS · Linux</span>
              <span aria-hidden="true">·</span>
              <span>
                {t("common.version")}{" "}
                <span className="font-mono text-zinc-200">
                  v{version}
                </span>
              </span>
            </p>
            <h1 className="mt-6 text-5xl font-bold tracking-tight text-white sm:text-6xl">
              {site.appName}
            </h1>
            <p className="mt-3 text-xl font-medium text-sky-400 sm:text-2xl">
              {t("footer.tagline")}.
            </p>
            <p className="mt-5 max-w-xl text-base leading-relaxed text-zinc-400">
              {c.heroText}
            </p>
            <div className="mt-8 flex flex-wrap gap-3">
              <Button to="/download">
                <Download className="h-4 w-4" aria-hidden="true" />
                {c.downloadForWindows}
              </Button>
              {github.configured && github.repoUrl ? (
                <Button href={github.repoUrl} variant="secondary" external>
                  {c.viewOnGitHub}
                </Button>
              ) : (
                <Button to="/documentation" variant="secondary">
                  {c.readDocs}
                </Button>
              )}
            </div>
            <p className="mt-6 text-xs text-zinc-500">{c.metaLine}</p>
          </div>

          <Reveal delay={100}>
            <HeroVisual />
          </Reveal>
        </div>
      </section>

      <section className="mx-auto max-w-6xl px-4 py-16 sm:px-6">
        <div className="max-w-2xl">
          <h2 className="text-3xl font-bold tracking-tight text-white">
            {c.featuresTitle}
          </h2>
          <p className="mt-3 text-zinc-400">{c.featuresSub}</p>
        </div>
        <div className="mt-10 grid gap-5 sm:grid-cols-2 lg:grid-cols-3">
          {featurePreview.map((feature, index) => (
            <Reveal key={feature.title[lang]} delay={index * 60}>
              <div className="group h-full rounded-xl border border-line bg-ink p-6 transition-colors hover:border-line-2">
                <feature.icon
                  className="h-6 w-6 text-sky-400"
                  aria-hidden="true"
                />
                <h3 className="mt-4 text-base font-semibold text-white">
                  {feature.title[lang]}
                </h3>
                <p className="mt-2 text-sm leading-relaxed text-zinc-400">
                  {feature.text[lang]}
                </p>
              </div>
            </Reveal>
          ))}
        </div>
        <div className="mt-8">
          <Link
            to="/features"
            className="inline-flex items-center gap-1.5 text-sm font-medium text-sky-400 transition-colors hover:text-sky-300 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-sky-400"
          >
            {c.exploreAll}
            <ArrowRight className="h-4 w-4" aria-hidden="true" />
          </Link>
        </div>
      </section>

      <section className="border-y border-line bg-ink">
        <div className="mx-auto max-w-6xl px-4 py-16 sm:px-6">
          <h2 className="text-3xl font-bold tracking-tight text-white">
            {c.howTitle}
          </h2>
          <p className="mt-3 max-w-2xl text-zinc-400">{c.howSub}</p>
          <ol className="mt-10 grid gap-6 md:grid-cols-5">
            {steps.map((step, index) => (
              <Reveal key={step.title[lang]} delay={index * 60}>
                <li className="relative h-full">
                  <div className="flex h-10 w-10 items-center justify-center rounded-lg border border-line-2 bg-ink-2">
                    <step.icon className="h-5 w-5 text-sky-400" aria-hidden="true" />
                  </div>
                  <p className="mt-3 font-mono text-xs text-zinc-500">
                    {t("common.step").replace("{n}", String(index + 1))}
                  </p>
                  <h3 className="mt-1 text-sm font-semibold text-white">
                    {step.title[lang]}
                  </h3>
                  <p className="mt-1.5 text-xs leading-relaxed text-zinc-400">
                    {step.text[lang]}
                  </p>
                </li>
              </Reveal>
            ))}
          </ol>
        </div>
      </section>

      <section className="mx-auto max-w-6xl px-4 py-16 sm:px-6">
        <h2 className="text-3xl font-bold tracking-tight text-white">
          {c.securityTitle}
        </h2>
        <div className="mt-10 grid gap-5 sm:grid-cols-2 lg:grid-cols-4">
          {securityPoints.map((point, index) => (
            <Reveal key={point.title[lang]} delay={index * 60}>
              <div className="h-full rounded-xl border border-line bg-ink p-6">
                <point.icon className="h-6 w-6 text-emerald-400" aria-hidden="true" />
                <h3 className="mt-4 text-sm font-semibold text-white">
                  {point.title[lang]}
                </h3>
                <p className="mt-2 text-sm leading-relaxed text-zinc-400">
                  {point.text[lang]}
                </p>
              </div>
            </Reveal>
          ))}
        </div>
        <div className="mt-8">
          <Link
            to="/security"
            className="inline-flex items-center gap-1.5 text-sm font-medium text-sky-400 transition-colors hover:text-sky-300 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-sky-400"
          >
            {c.securityCta}
            <ArrowRight className="h-4 w-4" aria-hidden="true" />
          </Link>
        </div>
      </section>

      <section className="border-t border-line bg-ink">
        <div className="mx-auto max-w-6xl px-4 py-16 text-center sm:px-6">
          <h2 className="text-3xl font-bold tracking-tight text-white">
            {c.ctaTitle}
          </h2>
          <p className="mx-auto mt-3 max-w-xl text-zinc-400">{c.ctaText}</p>
          <div className="mt-8 flex flex-wrap justify-center gap-3">
            <Button to="/download">
              <Download className="h-4 w-4" aria-hidden="true" />
              {c.downloadForWindows}
            </Button>
            <Button to="/features" variant="secondary">
              {c.seeWhat}
            </Button>
          </div>
        </div>
      </section>
    </>
  );
}
