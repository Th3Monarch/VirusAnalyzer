import {
  AlertTriangle,
  Archive,
  FileSearch,
  FileX2,
  Globe,
  Lock,
  ScrollText,
  ShieldCheck,
  Terminal,
  type LucideIcon,
} from "lucide-react";
import { Seo } from "../components/Seo";
import { Reveal } from "../components/Reveal";
import { useLanguage } from "../contexts/LanguageContext";
import type { Lang } from "../lib/i18n";
import { github, site } from "../config";

interface LocalizedText {
  en: string;
  es: string;
}

interface Card {
  icon: LucideIcon;
  title: LocalizedText;
  text: LocalizedText;
}

const cards: Card[] = [
  {
    icon: FileSearch,
    title: {
      en: "What VirusAnalyzer analyzes",
      es: "Qué analiza VirusAnalyzer",
    },
    text: {
      en: "Files and folders selected by the user: file type, entropy, PE structure, imports, exports, strings and hashes. All checks are static — nothing inside the file is executed.",
      es: "Archivos y carpetas seleccionados por el usuario: tipo de archivo, entropía, estructura PE, imports, exports, cadenas y hashes. Todas las comprobaciones son estáticas — nada dentro del archivo se ejecuta.",
    },
  },
  {
    icon: FileX2,
    title: {
      en: "What it does NOT do",
      es: "Lo que NO hace",
    },
    text: {
      en: "VirusAnalyzer does not provide real-time protection, automatic blocking, sandboxing, dynamic analysis or network behavior analysis. It analyzes on demand and never runs the file it is inspecting.",
      es: "VirusAnalyzer no ofrece protección en tiempo real, bloqueo automático, sandboxing, análisis dinámico ni análisis de comportamiento de red. Analiza bajo demanda y nunca ejecuta el archivo que está inspeccionando.",
    },
  },
  {
    icon: Globe,
    title: {
      en: "When it makes network requests",
      es: "Cuándo realiza peticiones de red",
    },
    text: {
      en: "Only when the optional VirusTotal integration is enabled in settings and a hash lookup is performed. By default the application makes no network requests.",
      es: "Solo cuando la integración opcional con VirusTotal está activada en la configuración y se realiza una consulta de hash. Por defecto la aplicación no realiza ninguna petición de red.",
    },
  },
  {
    icon: Lock,
    title: {
      en: "What is sent to VirusTotal",
      es: "Qué se envía a VirusTotal",
    },
    text: {
      en: "Only file hashes (MD5, SHA-1 or SHA-256). The file content is never uploaded. The integration requires explicit opt-in and a VirusTotal API key stored in the local configuration.",
      es: "Solo los hashes del archivo (MD5, SHA-1 o SHA-256). El contenido del archivo nunca se sube. La integración requiere consentimiento explícito y una clave API de VirusTotal guardada en la configuración local.",
    },
  },
  {
    icon: Terminal,
    title: {
      en: "How the terminal works",
      es: "Cómo funciona el terminal",
    },
    text: {
      en: "The terminal is a separate administrative tool. Commands run only on explicit user request, with a timeout and one execution at a time. High-risk commands require confirmation from the interface. The scanner never invokes the terminal.",
      es: "El terminal es una herramienta administrativa independiente. Los comandos se ejecutan solo bajo petición explícita del usuario, con un tiempo límite y una ejecución a la vez. Los comandos de alto riesgo requieren confirmación en la interfaz. El analizador nunca invoca el terminal.",
    },
  },
  {
    icon: Archive,
    title: {
      en: "How quarantine works",
      es: "Cómo funciona la cuarentena",
    },
    text: {
      en: "Quarantine moves a file to an isolated folder on explicit user action. Files are never quarantined automatically based on score. Restore and delete are also explicit.",
      es: "La cuarentena mueve un archivo a una carpeta aislada mediante acción explícita del usuario. Los archivos nunca se ponen en cuarentena automáticamente según su puntuación. Restaurar y eliminar también son acciones explícitas.",
    },
  },
  {
    icon: ScrollText,
    title: {
      en: "How to verify a release",
      es: "Cómo verificar una versión",
    },
    text: {
      en: "Every release ships SHA-256 checksum files. Compute the hash of a downloaded file locally and compare it with the published value before running it.",
      es: "Cada versión incluye archivos de checksums SHA-256. Calcula el hash de un archivo descargado localmente y compáralo con el valor publicado antes de ejecutarlo.",
    },
  },
  {
    icon: ShieldCheck,
    title: {
      en: "Code signing status",
      es: "Estado de la firma de código",
    },
    text: {
      en: "The project supports code signing when configured. Current releases may be unsigned; this does not affect the checksum verification described above.",
      es: "El proyecto admite la firma de código cuando está configurada. Las versiones actuales pueden no estar firmadas; esto no afecta a la verificación de checksums descrita arriba.",
    },
  },
];

interface SecurityCopy {
  eyebrow: string;
  title: string;
  importantLabel: string;
  importantText: string;
  avTitle: string;
  avP1: string;
  avP2: string;
  avBullets: string[];
  avLink: string;
  avFallback: string;
  reportTitle: string;
  reportText: string;
  openIssue: string;
  configHint: string;
}

const copy: Record<Lang, SecurityCopy> = {
  en: {
    eyebrow: "Security",
    title: "Built with security in mind.",
    importantLabel: "Important:",
    importantText:
      "{app} is an analysis and threat assessment tool. It is not a replacement for a professional endpoint security solution.",
    avTitle: "Antivirus detections",
    avP1:
      "Newly compiled or unsigned applications can sometimes receive heuristic antivirus detections. Antivirus engines vary in their heuristics, and a detection on a recently compiled binary is not, by itself, proof that the binary is malicious.",
    avP2:
      "{app} does not intentionally use obfuscation, packing, anti-analysis techniques or antivirus evasion. Its behavior is documented and its source code is public.",
    avBullets: [
      "Verify the SHA-256 hash of the file against the checksums published with the release.",
      "Review the corresponding GitHub release and source code.",
      "Consider that heuristic machine-learning detections (such as Microsoft’s “!ml” family) are statistical and can affect unsigned binaries more frequently.",
    ],
    avLink: "Browse releases on GitHub",
    avFallback: "Releases are published on GitHub when the repository is configured.",
    reportTitle: "Reporting problems",
    reportText:
      "If you find a bug, a security issue or a false detection on the project itself, please report it through the official issue tracker so it can be investigated.",
    openIssue: "Open an issue on GitHub",
    configHint:
      "Configure “githubOwner” and “githubRepository” in src/site.config.json to link the issue tracker.",
  },
  es: {
    eyebrow: "Seguridad",
    title: "Construido con la seguridad en mente.",
    importantLabel: "Importante:",
    importantText:
      "{app} es una herramienta de análisis y evaluación de amenazas. No sustituye a una solución profesional de seguridad de endpoints.",
    avTitle: "Detecciones de antivirus",
    avP1:
      "Las aplicaciones recién compiladas o sin firmar pueden recibir a veces detecciones heurísticas de antivirus. Los motores antivirus varían en sus heurísticas, y una detección sobre un binario recién compilado no es, por sí sola, prueba de que el binario sea malicioso.",
    avP2:
      "{app} no usa intencionadamente ofuscación, empaquetado, técnicas anti-análisis ni evasión de antivirus. Su comportamiento está documentado y su código fuente es público.",
    avBullets: [
      "Verifica el hash SHA-256 del archivo contra los checksums publicados con la versión.",
      "Revisa la versión de GitHub correspondiente y el código fuente.",
      "Ten en cuenta que las detecciones heurísticas de aprendizaje automático (como la familia “!ml” de Microsoft) son estadísticas y pueden afectar más a los binarios sin firmar.",
    ],
    avLink: "Explorar versiones en GitHub",
    avFallback: "Las versiones se publican en GitHub cuando el repositorio está configurado.",
    reportTitle: "Notificar problemas",
    reportText:
      "Si encuentras un error, un problema de seguridad o una falsa detección sobre el propio proyecto, infórmalo a través del rastreador de incidencias oficial para que pueda ser investigado.",
    openIssue: "Abrir una incidencia en GitHub",
    configHint:
      "Configura “githubOwner” y “githubRepository” en src/site.config.json para enlazar el rastreador de incidencias.",
  },
};

export function Security() {
  const { lang, t } = useLanguage();
  const c = copy[lang];

  return (
    <>
      <Seo title={t("seo.security.title")} description={t("seo.security.description")} path="/security" />

      <section className="mx-auto max-w-6xl px-4 py-16 sm:px-6">
        <p className="text-sm font-semibold text-sky-400">{c.eyebrow}</p>
        <h1 className="mt-2 text-4xl font-bold tracking-tight text-white">
          {c.title}
        </h1>

        <div
          className="mt-8 flex items-start gap-3 rounded-xl border border-amber-400/30 bg-amber-400/5 p-5"
          role="note"
        >
          <AlertTriangle className="mt-0.5 h-5 w-5 shrink-0 text-amber-400" aria-hidden="true" />
          <p className="text-sm leading-relaxed text-amber-200">
            <span className="font-semibold">{c.importantLabel}</span>{" "}
            {c.importantText.replace("{app}", site.appName)}
          </p>
        </div>

        <div className="mt-12 grid gap-6 md:grid-cols-2">
          {cards.map((card, index) => (
            <Reveal key={card.title[lang]} delay={(index % 2) * 60}>
              <article className="h-full rounded-xl border border-line bg-ink p-6">
                <div className="flex h-10 w-10 items-center justify-center rounded-lg border border-line-2 bg-ink-2">
                  <card.icon className="h-5 w-5 text-emerald-400" aria-hidden="true" />
                </div>
                <h2 className="mt-4 text-base font-semibold text-white">
                  {card.title[lang]}
                </h2>
                <p className="mt-2 text-sm leading-relaxed text-zinc-400">
                  {card.text[lang]}
                </p>
              </article>
            </Reveal>
          ))}
        </div>

        <section className="mt-16" aria-labelledby="antivirus-title">
          <h2 id="antivirus-title" className="text-2xl font-bold tracking-tight text-white">
            {c.avTitle}
          </h2>
          <div className="mt-5 max-w-3xl space-y-4 text-sm leading-relaxed text-zinc-400">
            <p>{c.avP1}</p>
            <p>{c.avP2.replace("{app}", site.appName)}</p>
            <ul className="list-disc space-y-1 pl-5">
              {c.avBullets.map((item) => (
                <li key={item}>{item}</li>
              ))}
            </ul>
            <p>
              {github.configured && github.releasesUrl ? (
                <a
                  href={github.releasesUrl}
                  target="_blank"
                  rel="noopener noreferrer"
                  className="font-medium text-sky-400 transition-colors hover:text-sky-300 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-sky-400"
                >
                  {c.avLink}
                </a>
              ) : (
                c.avFallback
              )}
            </p>
          </div>
        </section>

        <section className="mt-16" aria-labelledby="report-title">
          <h2 id="report-title" className="text-2xl font-bold tracking-tight text-white">
            {c.reportTitle}
          </h2>
          <p className="mt-4 max-w-3xl text-sm leading-relaxed text-zinc-400">
            {c.reportText}
          </p>
          <div className="mt-5">
            {github.configured && github.issuesUrl ? (
              <a
                href={github.issuesUrl}
                target="_blank"
                rel="noopener noreferrer"
                className="inline-flex items-center gap-2 rounded-lg border border-line-2 bg-ink-2 px-4 py-2.5 text-sm font-semibold text-zinc-100 transition-colors hover:border-sky-500/60 hover:bg-ink-3 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-sky-400"
              >
                {c.openIssue}
              </a>
            ) : (
              <p className="text-sm text-zinc-500">{c.configHint}</p>
            )}
          </div>
        </section>
      </section>
    </>
  );
}
