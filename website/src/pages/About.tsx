import { FileSearch, GraduationCap, Lock, ShieldCheck, Sparkles, type LucideIcon } from "lucide-react";
import { Seo } from "../components/Seo";
import { Reveal } from "../components/Reveal";
import { useLanguage } from "../contexts/LanguageContext";
import type { Lang } from "../lib/i18n";
import { github, site } from "../config";

interface LocalizedText {
  en: string;
  es: string;
}

interface Pillar {
  icon: LucideIcon;
  title: LocalizedText;
  text: LocalizedText;
}

const pillars: Pillar[] = [
  {
    icon: FileSearch,
    title: { en: "Static-first analysis", es: "Análisis estático primero" },
    text: {
      en: "The analyzer inspects files without executing them: type, entropy, structure, strings and hashes.",
      es: "El analizador inspecciona archivos sin ejecutarlos: tipo, entropía, estructura, cadenas y hashes.",
    },
  },
  {
    icon: Sparkles,
    title: { en: "Explainable heuristics", es: "Heurísticas explicables" },
    text: {
      en: "Every score is the sum of explicit, evidence-backed rules that the user can review.",
      es: "Cada puntuación es la suma de reglas explícitas respaldadas por evidencia que el usuario puede revisar.",
    },
  },
  {
    icon: Lock,
    title: { en: "Privacy by default", es: "Privacidad por defecto" },
    text: {
      en: "Analysis is local and the app makes no network requests unless the optional VirusTotal integration is enabled.",
      es: "El análisis es local y la app no realiza peticiones de red salvo que se active la integración opcional con VirusTotal.",
    },
  },
  {
    icon: GraduationCap,
    title: { en: "Learning and research", es: "Aprendizaje e investigación" },
    text: {
      en: "Built to help analysts, students and enthusiasts understand how suspicious files are assessed.",
      es: "Creada para ayudar a analistas, estudiantes y entusiastas a entender cómo se evalúan los archivos sospechosos.",
    },
  },
];

const copy: Record<Lang, Record<string, string>> = {
  en: {
    eyebrow: "About",
    title: "About {app}",
    intro:
      "{app} is a Windows desktop application for static malware analysis and threat assessment. It combines traditional static checks, an explainable heuristic engine and optional reputation data to help you understand what a suspicious file looks like — without executing it.",
    whyTitle: "Why it exists",
    whyP1:
      "{app} was built for learning, research and defensive security. Static analysis is a foundational skill for anyone who works with suspicious files, and this project makes the process tangible: every heuristic, every score and every finding is visible and explained.",
    whyP2:
      "Because the analysis is deterministic and local, the project does not rely on opaque cloud services to produce a result. When external reputation data is useful, it is available as an optional, hash-only integration.",
    whyP3:
      "{app} is not an antivirus and is not a substitute for endpoint protection. It is a defensive, educational tool that respects the user’s data and runs entirely at their control.",
    transparencyTitle: "Transparency",
    transparencyText:
      "The source code is open and every release publishes SHA-256 checksums. You are encouraged to verify downloads and build the project yourself.",
    repoLabel: "Repository and issue tracker: ",
    repoFallback:
      "The repository link is added here once the GitHub configuration is set in src/site.config.json.",
  },
  es: {
    eyebrow: "Acerca de",
    title: "Acerca de {app}",
    intro:
      "{app} es una aplicación de escritorio para Windows de análisis estático de malware y evaluación de amenazas. Combina comprobaciones estáticas tradicionales, un motor heurístico explicable y datos de reputación opcionales para ayudarte a entender cómo es un archivo sospechoso — sin ejecutarlo.",
    whyTitle: "Por qué existe",
    whyP1:
      "{app} se creó para el aprendizaje, la investigación y la seguridad defensiva. El análisis estático es una habilidad fundamental para cualquiera que trabaje con archivos sospechosos, y este proyecto hace tangible el proceso: cada heurística, cada puntuación y cada hallazgo son visibles y están explicados.",
    whyP2:
      "Como el análisis es determinista y local, el proyecto no depende de servicios en la nube opacos para producir un resultado. Cuando los datos de reputación externos resultan útiles, están disponibles como una integración opcional solo por hash.",
    whyP3:
      "{app} no es un antivirus ni sustituye a la protección de endpoints. Es una herramienta defensiva y educativa que respeta los datos del usuario y se ejecuta por completo bajo su control.",
    transparencyTitle: "Transparencia",
    transparencyText:
      "El código fuente es abierto y cada versión publica checksums SHA-256. Te animamos a verificar las descargas y a compilar el proyecto tú mismo.",
    repoLabel: "Repositorio y rastreador de incidencias: ",
    repoFallback:
      "El enlace al repositorio se añadirá aquí cuando la configuración de GitHub esté establecida en src/site.config.json.",
  },
};

export function About() {
  const { lang, t } = useLanguage();
  const c = copy[lang];

  return (
    <>
      <Seo title={t("seo.about.title")} description={t("seo.about.description")} path="/about" />

      <section className="mx-auto max-w-6xl px-4 py-16 sm:px-6">
        <p className="text-sm font-semibold text-sky-400">{c.eyebrow}</p>
        <h1 className="mt-2 text-4xl font-bold tracking-tight text-white">
          {c.title.replace("{app}", site.appName)}
        </h1>
        <p className="mt-5 max-w-3xl text-base leading-relaxed text-zinc-400">
          {c.intro.replace("{app}", site.appName)}
        </p>

        <div className="mt-12 grid gap-6 md:grid-cols-2">
          {pillars.map((pillar, index) => (
            <Reveal key={pillar.title[lang]} delay={(index % 2) * 60}>
              <article className="h-full rounded-xl border border-line bg-ink p-6">
                <div className="flex h-10 w-10 items-center justify-center rounded-lg border border-line-2 bg-ink-2">
                  <pillar.icon className="h-5 w-5 text-sky-400" aria-hidden="true" />
                </div>
                <h2 className="mt-4 text-base font-semibold text-white">
                  {pillar.title[lang]}
                </h2>
                <p className="mt-2 text-sm leading-relaxed text-zinc-400">
                  {pillar.text[lang]}
                </p>
              </article>
            </Reveal>
          ))}
        </div>

        <section className="mt-16" aria-labelledby="why-title">
          <h2 id="why-title" className="text-2xl font-bold tracking-tight text-white">
            {c.whyTitle}
          </h2>
          <div className="mt-5 max-w-3xl space-y-4 text-sm leading-relaxed text-zinc-400">
            <p>{c.whyP1.replace("{app}", site.appName)}</p>
            <p>{c.whyP2}</p>
            <p>{c.whyP3.replace("{app}", site.appName)}</p>
          </div>
        </section>

        <section className="mt-16" aria-labelledby="transparency-title">
          <h2 id="transparency-title" className="text-2xl font-bold tracking-tight text-white">
            {c.transparencyTitle}
          </h2>
          <div className="mt-5 max-w-3xl space-y-4 text-sm leading-relaxed text-zinc-400">
            <p className="flex items-start gap-2">
              <ShieldCheck className="mt-0.5 h-5 w-5 shrink-0 text-emerald-400" aria-hidden="true" />
              <span>{c.transparencyText}</span>
            </p>
            {github.configured && github.repoUrl ? (
              <p>
                {c.repoLabel}
                <a
                  href={github.repoUrl}
                  target="_blank"
                  rel="noopener noreferrer"
                  className="font-medium text-sky-400 transition-colors hover:text-sky-300 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-sky-400"
                >
                  {github.repoUrl}
                </a>
                .
              </p>
            ) : (
              <p>{c.repoFallback}</p>
            )}
          </div>
        </section>
      </section>
    </>
  );
}
