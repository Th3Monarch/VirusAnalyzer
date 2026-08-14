import { useState } from "react";
import { ChevronDown } from "lucide-react";
import { Seo } from "../components/Seo";
import { Reveal } from "../components/Reveal";
import { useLanguage } from "../contexts/LanguageContext";
import type { Lang } from "../lib/i18n";
import { github } from "../config";

interface LocalizedText {
  en: string;
  es: string;
}

interface FaqItem {
  question: LocalizedText;
  answer: LocalizedText;
}

const faqs: FaqItem[] = [
  {
    question: { en: "What is VirusAnalyzer?", es: "¿Qué es VirusAnalyzer?" },
    answer: {
      en: "VirusAnalyzer is a Windows desktop application for malware analysis and threat assessment. It performs static analysis of suspicious files and folders, computes hashes, applies an explainable heuristic engine and optionally checks reputation on VirusTotal by hash.",
      es: "VirusAnalyzer es una aplicación de escritorio para Windows de análisis de malware y evaluación de amenazas. Realiza un análisis estático de archivos y carpetas sospechosos, calcula hashes, aplica un motor heurístico explicable y opcionalmente consulta la reputación en VirusTotal por hash.",
    },
  },
  {
    question: { en: "Is VirusAnalyzer an antivirus?", es: "¿Es VirusAnalyzer un antivirus?" },
    answer: {
      en: "No. VirusAnalyzer is an analysis and threat assessment tool, not a real-time antivirus. It does not offer real-time protection, automatic blocking or scheduled scanning, and it is not a replacement for a professional endpoint security solution.",
      es: "No. VirusAnalyzer es una herramienta de análisis y evaluación de amenazas, no un antivirus en tiempo real. No ofrece protección en tiempo real, bloqueo automático ni análisis programados, y no sustituye a una solución profesional de seguridad de endpoints.",
    },
  },
  {
    question: { en: "Does VirusAnalyzer execute suspicious files?", es: "¿Ejecuta VirusAnalyzer archivos sospechosos?" },
    answer: {
      en: "No. Analysis is static: VirusAnalyzer inspects the file (type, entropy, PE structure, imports, strings) without executing its contents. It never runs the file it is analyzing, and it provides no sandbox or dynamic analysis.",
      es: "No. El análisis es estático: VirusAnalyzer inspecciona el archivo (tipo, entropía, estructura PE, imports, cadenas) sin ejecutar su contenido. Nunca ejecuta el archivo que está analizando y no ofrece sandbox ni análisis dinámico.",
    },
  },
  {
    question: { en: "Does VirusAnalyzer upload my files?", es: "¿Sube VirusAnalyzer mis archivos?" },
    answer: {
      en: "No. The application only makes network requests when the optional VirusTotal integration is enabled and a hash lookup is performed. Even then, only file hashes (MD5, SHA-1 or SHA-256) are sent — never the file content.",
      es: "No. La aplicación solo realiza peticiones de red cuando la integración opcional con VirusTotal está activada y se consulta un hash. Incluso entonces, solo se envían los hashes del archivo (MD5, SHA-1 o SHA-256) — nunca el contenido.",
    },
  },
  {
    question: { en: "What is VirusTotal used for?", es: "¿Para qué se usa VirusTotal?" },
    answer: {
      en: "The optional VirusTotal integration compares the hash of a file against VirusTotal's database to show how many engines have flagged it. It is disabled by default and requires an explicit opt-in and an API key.",
      es: "La integración opcional con VirusTotal compara el hash de un archivo con la base de datos de VirusTotal para mostrar cuántos motores lo han marcado. Está desactivada por defecto y requiere un consentimiento explícito y una clave API.",
    },
  },
  {
    question: { en: "Can I use VirusAnalyzer without VirusTotal?", es: "¿Puedo usar VirusAnalyzer sin VirusTotal?" },
    answer: {
      en: "Yes. The VirusTotal integration is optional and off by default. All static analysis, hashing and heuristic assessment work fully offline.",
      es: "Sí. La integración con VirusTotal es opcional y está desactivada por defecto. Todo el análisis estático, el hashing y la evaluación heurística funcionan completamente sin conexión.",
    },
  },
  {
    question: { en: "Is VirusAnalyzer free?", es: "¿Es gratis VirusAnalyzer?" },
    answer: {
      en: "The application and its source code are open source. You can download it for free and build it yourself from source.",
      es: "La aplicación y su código fuente son de código abierto. Puedes descargarla gratis y compilarla tú mismo desde el código.",
    },
  },
  {
    question: { en: "Is the source code available?", es: "¿Está disponible el código fuente?" },
    answer: {
      en: "Yes. The project is open source. The source code, releases and issue tracker are available on GitHub when the repository is configured.",
      es: "Sí. El proyecto es de código abierto. El código fuente, las versiones y el rastreador de incidencias están disponibles en GitHub cuando el repositorio está configurado.",
    },
  },
  {
    question: { en: "How can I verify a download?", es: "¿Cómo puedo verificar una descarga?" },
    answer: {
      en: "Every release publishes SHA-256 checksums. Compute the hash of the downloaded file locally (for example with certutil -hashfile file SHA256) and compare it with the published checksum before running it.",
      es: "Cada versión publica checksums SHA-256. Calcula el hash del archivo descargado localmente (por ejemplo con certutil -hashfile archivo SHA256) y compáralo con el checksum publicado antes de ejecutarlo.",
    },
  },
  {
    question: { en: "Why does Windows Defender warn about an executable?", es: "¿Por qué Windows Defender avisa sobre un ejecutable?" },
    answer: {
      en: "Newly compiled or unsigned Windows executables can be flagged by heuristic machine-learning detections, even when they are benign. VirusAnalyzer does not use obfuscation, packing or evasion techniques. Verify the SHA-256 checksum against the published release before running it.",
      es: "Los ejecutables de Windows recién compilados o sin firmar pueden marcarse con detecciones heurísticas de aprendizaje automático, incluso cuando son benignos. VirusAnalyzer no usa ofuscación, empaquetado ni técnicas de evasión. Verifica el checksum SHA-256 contra la versión publicada antes de ejecutarlo.",
    },
  },
  {
    question: { en: "What platforms are supported?", es: "¿Qué plataformas se soportan?" },
    answer: {
      en: "Windows 10 and Windows 11 on x64, with the WebView2 Runtime (included in Windows 11 and in most recent Windows 10 installations).",
      es: "Windows 10 y Windows 11 en x64, con el WebView2 Runtime (incluido en Windows 11 y en la mayoría de instalaciones recientes de Windows 10).",
    },
  },
];

const copy: Record<Lang, Record<string, string>> = {
  en: {
    eyebrow: "FAQ",
    title: "Frequently asked questions",
    stillQuestions: "Still have questions?",
    checkDocs: "Check the documentation for more details.",
    openIssueBefore: "Check the documentation or open an issue on ",
  },
  es: {
    eyebrow: "FAQ",
    title: "Preguntas frecuentes",
    stillQuestions: "¿Aún tienes preguntas?",
    checkDocs: "Consulta la documentación para más detalles.",
    openIssueBefore: "Consulta la documentación o abre una incidencia en ",
  },
};

function FaqItem({ item, index }: { item: FaqItem; index: number }) {
  const { lang } = useLanguage();
  const [open, setOpen] = useState(index === 0);

  const panelId = `faq-panel-${index}`;
  const buttonId = `faq-button-${index}`;

  return (
    <div className="rounded-xl border border-line bg-ink">
      <h3>
        <button
          id={buttonId}
          type="button"
          aria-expanded={open}
          aria-controls={panelId}
          onClick={() => setOpen((value) => !value)}
          className="flex w-full items-center justify-between gap-4 px-5 py-4 text-left text-sm font-semibold text-white transition-colors hover:text-sky-300 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-sky-400 rounded-xl"
        >
          {item.question[lang]}
          <ChevronDown
            className={`h-4 w-4 shrink-0 text-zinc-500 transition-transform ${
              open ? "rotate-180" : ""
            }`}
            aria-hidden="true"
          />
        </button>
      </h3>
      <div
        id={panelId}
        role="region"
        aria-labelledby={buttonId}
        className={`grid transition-all duration-200 ease-out ${
          open ? "grid-rows-[1fr] opacity-100" : "grid-rows-[0fr] opacity-0"
        }`}
      >
        <div className="overflow-hidden">
          <p className="px-5 pb-5 text-sm leading-relaxed text-zinc-400">
            {item.answer[lang]}
          </p>
        </div>
      </div>
    </div>
  );
}

export function Faq() {
  const { lang, t } = useLanguage();
  const c = copy[lang];

  return (
    <>
      <Seo title={t("seo.faq.title")} description={t("seo.faq.description")} path="/faq" />

      <section className="mx-auto max-w-3xl px-4 py-16 sm:px-6">
        <p className="text-sm font-semibold text-sky-400">{c.eyebrow}</p>
        <h1 className="mt-2 text-4xl font-bold tracking-tight text-white">
          {c.title}
        </h1>

        <div className="mt-10 space-y-3">
          {faqs.map((item, index) => (
            <Reveal key={item.question[lang]} delay={(index % 4) * 40}>
              <FaqItem item={item} index={index} />
            </Reveal>
          ))}
        </div>

        <div className="mt-12 rounded-xl border border-line bg-ink p-6 text-sm text-zinc-400">
          <p className="font-semibold text-white">{c.stillQuestions}</p>
          <p className="mt-2 leading-relaxed">
            {github.configured && github.issuesUrl ? (
              <>
                {c.openIssueBefore}
                <a
                  href={github.issuesUrl}
                  target="_blank"
                  rel="noopener noreferrer"
                  className="font-medium text-sky-400 transition-colors hover:text-sky-300 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-sky-400"
                >
                  GitHub
                </a>
                .
              </>
            ) : (
              c.checkDocs
            )}
          </p>
        </div>
      </section>
    </>
  );
}
