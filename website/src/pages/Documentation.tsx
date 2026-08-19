import type { ReactNode } from "react";
import { Link } from "react-router-dom";
import { Seo } from "../components/Seo";
import { useLanguage } from "../contexts/LanguageContext";
import type { Lang } from "../lib/i18n";
import { site } from "../config";

function Code({ children }: { children: string }) {
  return (
    <pre className="mt-4 overflow-x-auto rounded-lg border border-line bg-night p-4 font-mono text-xs leading-relaxed text-zinc-300">
      {children}
    </pre>
  );
}

function Section({
  id,
  title,
  children,
}: {
  id: string;
  title: string;
  children: ReactNode;
}) {
  return (
    <section id={id} className="scroll-mt-24">
      <h2 className="text-2xl font-bold tracking-tight text-white">{title}</h2>
      <div className="mt-4 space-y-4 text-sm leading-relaxed text-zinc-400">
        {children}
      </div>
    </section>
  );
}

function Bullet({ children }: { children: ReactNode }) {
  return (
    <li className="flex items-start gap-2">
      <span className="mt-1.5 h-1.5 w-1.5 shrink-0 rounded-full bg-sky-400" aria-hidden="true" />
      <span>{children}</span>
    </li>
  );
}

interface BulletItem {
  mono?: boolean;
  lead?: string;
  text: string;
}

interface DocSection {
  id: string;
  title: string;
  blocks: ReactNode;
}

function bullet(lead?: string, text = "", mono = false): BulletItem {
  return { lead, text, mono };
}

function buildDoc(lang: Lang, app: string): DocSection[] {
  const L = (en: string, es: string) => (lang === "es" ? es : en);

  const bullets = (items: BulletItem[]) => (
    <ul className="list-none space-y-2">
      {items.map((item) => (
        <Bullet key={`${item.lead ?? ""}${item.text}`}>
          {item.lead ? (
            <span className={item.mono ? "font-mono text-zinc-300" : "font-semibold text-zinc-200"}>
              {item.lead}
            </span>
          ) : null}
          {item.lead ? `${item.mono ? " — " : " "}${item.text}` : item.text}
        </Bullet>
      ))}
    </ul>
  );

  return [
    {
      id: "getting-started",
      title: L("Getting Started", "Primeros pasos"),
      blocks: (
        <>
          <p>
            {app} is a Windows desktop application for static malware analysis
            and threat assessment. It is distributed as a native x64 binary
            that requires the WebView2 Runtime.
          </p>
          {bullets([
            bullet(undefined, L("Windows 10 or Windows 11, x64", "Windows 10 u 11, x64")),
            bullet(
              undefined,
              L(
                "WebView2 Runtime (included in Windows 11 and most recent Windows 10 installations)",
                "WebView2 Runtime (incluido en Windows 11 y en la mayoría de instalaciones recientes de Windows 10)",
              ),
            ),
            bullet(
              undefined,
              L(
                "No administrator rights required for analysis",
                "No se requieren permisos de administrador para analizar",
              ),
            ),
          ])}
          <p>
            {L(
              "Download the installer or the portable version from the ",
              "Descarga el instalador o la versión portable desde la ",
            )}
            <Link
              to="/download"
              className="font-medium text-sky-400 transition-colors hover:text-sky-300 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-sky-400"
            >
              {L("download page", "página de descargas")}
            </Link>
            .
          </p>
        </>
      ),
    },
    {
      id: "installation",
      title: L("Installation", "Instalación"),
      blocks: (
        <>
          <p>
            <span className="font-semibold text-zinc-200">
              {L("Recommended — Setup installer:", "Recomendado — Instalador Setup:")}
            </span>{" "}
            {L(
              "installs the application for the current user, creates Start menu and desktop shortcuts, registers it in “Add or remove programs” and includes an uninstaller.",
              "instala la aplicación para el usuario actual, crea accesos directos en el menú Inicio y el escritorio, la registra en “Agregar o quitar programas” e incluye un desinstalador.",
            )}
          </p>
          <p>
            <span className="font-semibold text-zinc-200">
              {L("Portable:", "Portable:")}
            </span>{" "}
            {L("extract the ZIP anywhere (a USB drive, a folder) and run ", "extrae el ZIP donde quieras (un USB, una carpeta) y ejecuta ")}
            <span className="font-mono text-zinc-300">Prometeo.exe</span>{" "}
            {L(
              "directly. No installation or registry changes are made.",
              "directamente. No se realizan instalaciones ni cambios en el registro.",
            )}
          </p>
          <p>
            <span className="font-semibold text-zinc-200">
              {L("Portable requirements:", "Requisitos portable:")}
            </span>{" "}
            {L(
              "the WebView2 Runtime must be present. Configuration and history are stored in the application data folder.",
              "debe estar presente el WebView2 Runtime. La configuración y el historial se guardan en la carpeta de datos de la aplicación.",
            )}
          </p>
        </>
      ),
    },
    {
      id: "first-analysis",
      title: L("First Analysis", "Primer análisis"),
      blocks: (
        <>
          <p>{L("Launch ", "Ejecuta ")}{app} {L("and:", "y:")}</p>
          {bullets([
            bullet(undefined, L("Use the file or folder picker from the Scan page.", "Usa el selector de archivo o carpeta de la página Análisis.")),
            bullet(undefined, L("Or drag and drop a file onto the window.", "O arrastra y suelta un archivo sobre la ventana.")),
            bullet(undefined, L("Watch the progress as the file is inspected.", "Observa el progreso mientras se inspecciona el archivo.")),
            bullet(
              undefined,
              L(
                "Open the result to review hashes, findings, the threat score and the evidence-based assessment.",
                "Abre el resultado para revisar hashes, hallazgos, la puntuación de amenaza y la evaluación basada en evidencias.",
              ),
            ),
          ])}
          <p>
            {L(
              "Analysis is static: the file is never executed, and results are computed locally.",
              "El análisis es estático: el archivo nunca se ejecuta y los resultados se calculan localmente.",
            )}
          </p>
        </>
      ),
    },
    {
      id: "understanding-threat-scores",
      title: L("Understanding Threat Scores", "Cómo entender las puntuaciones de amenaza"),
      blocks: (
        <>
          <p>
            {L(
              "The threat score is an explainable number from 0 to 100, computed by a heuristic engine with 28 rules across seven categories (process, persistence, PowerShell, packing, network, signatures and general).",
              "La puntuación de amenaza es un número explicable de 0 a 100, calculado por un motor heurístico con 28 reglas en siete categorías (proceso, persistencia, PowerShell, empaquetado, red, firmas y general).",
            )}
          </p>
          {bullets([
            bullet(L("Clean", "Limpio"), "0", true),
            bullet(L("Low", "Bajo"), "1 to 14", true),
            bullet(L("Medium", "Medio"), "15 to 34", true),
            bullet(L("High", "Alto"), "35 to 64", true),
            bullet(L("Critical", "Crítico"), "65 or more", true),
          ])}
          <p>
            {L(
              "Every finding includes its severity, category, evidence and the points it contributed. The local assessment synthesizes this evidence into a plain-language explanation with a verdict and a confidence value.",
              "Cada hallazgo incluye su severidad, categoría, evidencia y los puntos que aportó. La evaluación local sintetiza esta evidencia en una explicación en lenguaje claro con un veredicto y un valor de confianza.",
            )}
          </p>
        </>
      ),
    },
    {
      id: "hash-analysis",
      title: L("Hash Analysis", "Análisis de hashes"),
      blocks: (
        <>
          <p>
            {L(
              "Every scanned file produces MD5, SHA-1 and SHA-256 hashes using streaming computation. You can choose which algorithms to compute in the settings and copy any hash with one click.",
              "Cada archivo analizado produce hashes MD5, SHA-1 y SHA-256 mediante cálculo por streaming. Puedes elegir qué algoritmos calcular en la configuración y copiar cualquier hash con un clic.",
            )}
          </p>
          <p>{L("Example verification command:", "Ejemplo de comando de verificación:")}</p>
          <Code>{`certutil -hashfile Prometeo-2.0.0-Setup.exe SHA256`}</Code>
        </>
      ),
    },
    {
      id: "virustotal-integration",
      title: L("VirusTotal Integration", "Integración con VirusTotal"),
      blocks: (
        <>
          <p>
            {L(
              "VirusTotal reputation checks are optional and disabled by default. To enable them, open Settings, switch on the integration and provide a VirusTotal API key.",
              "Las comprobaciones de reputación de VirusTotal son opcionales y están desactivadas por defecto. Para activarlas, abre Configuración, activa la integración y proporciona una clave API de VirusTotal.",
            )}
          </p>
          {bullets([
            bullet(undefined, L("Only MD5, SHA-1 or SHA-256 hashes are sent to VirusTotal.", "Solo se envían a VirusTotal los hashes MD5, SHA-1 o SHA-256.")),
            bullet(undefined, L("The file content is never uploaded.", "El contenido del archivo nunca se sube.")),
            bullet(undefined, L("The API key is stored in the local configuration and never logged.", "La clave API se guarda en la configuración local y nunca se registra.")),
            bullet(
              undefined,
              L(
                "A hash with no record is reported as “not available”, without an error.",
                "Un hash sin registros se informa como “no disponible”, sin error.",
              ),
            ),
          ])}
          <p>
            {L(
              "You can also look up a hash manually from the analysis view when the integration is enabled.",
              "También puedes consultar un hash manualmente desde la vista de análisis cuando la integración está activada.",
            )}
          </p>
        </>
      ),
    },
    {
      id: "quarantine",
      title: L("Quarantine", "Cuarentena"),
      blocks: (
        <>
          <p>
            {L(
              "Quarantine isolates a suspicious file by moving it to a dedicated folder. It is always an explicit user action — files are never quarantined automatically based on their score.",
              "La cuarentena aísla un archivo sospechoso moviéndolo a una carpeta dedicada. Siempre es una acción explícita del usuario — los archivos nunca se ponen en cuarentena automáticamente según su puntuación.",
            )}
          </p>
          {bullets([
            bullet(undefined, L("Isolate: moves the file and records its original path, hashes and reason.", "Aislar: mueve el archivo y registra su ruta original, hashes y motivo.")),
            bullet(undefined, L("Restore: returns the file to its original location, without overwriting existing files.", "Restaurar: devuelve el archivo a su ubicación original, sin sobrescribir archivos existentes.")),
            bullet(undefined, L("Delete: permanently removes the file and its record.", "Eliminar: borra permanentemente el archivo y su registro.")),
            bullet(undefined, L("The quarantine folder can be changed in Settings.", "La carpeta de cuarentena puede cambiarse en Configuración.")),
          ])}
        </>
      ),
    },
    {
      id: "analysis-history",
      title: L("Analysis History", "Historial de análisis"),
      blocks: (
        <p>
          {L(
            "Every analysis is stored in a persistent history that survives application restarts. Open the Results page to browse past analyses, search them and reopen any result to review its full details.",
            "Cada análisis se guarda en un historial persistente que sobrevive a los reinicios de la aplicación. Abre la página Resultados para consultar análisis anteriores, buscarlos y reabrir cualquier resultado para revisar todos sus detalles.",
          )}
        </p>
      ),
    },
    {
      id: "powershell",
      title: L("PowerShell", "PowerShell"),
      blocks: (
        <>
          <p>
            {L(
              "PowerShell is an advanced, optional tool, separate from the analyzer. It is invoked only by explicit user action and never by the scanner.",
              "PowerShell es una herramienta avanzada y opcional, independiente del analizador. Solo se invoca mediante acción explícita del usuario y nunca por el analizador.",
            )}
          </p>
          {bullets([
            bullet(undefined, L("Commands run with the current user’s privileges; no elevation.", "Los comandos se ejecutan con los privilegios del usuario actual; sin elevación.")),
            bullet(undefined, L("One execution at a time, with a 30-second timeout and explicit cancellation.", "Una ejecución a la vez, con un tiempo límite de 30 segundos y cancelación explícita.")),
            bullet(undefined, L("High-risk commands require confirmation in the interface.", "Los comandos de alto riesgo requieren confirmación en la interfaz.")),
            bullet(undefined, L("The built-in reference catalogs common commands without executing them.", "La referencia integrada cataloga comandos comunes sin ejecutarlos.")),
          ])}
        </>
      ),
    },
    {
      id: "reports",
      title: L("Reports", "Informes"),
      blocks: (
        <>
          <p>
            {L(
              "Generate an analysis report to share or archive your findings.",
              "Genera un informe de análisis para compartir o archivar tus hallazgos.",
            )}
          </p>
          {bullets([
            bullet(undefined, L("HTML: a self-contained document with summary, findings and reputation.", "HTML: un documento autocontenido con resumen, hallazgos y reputación.")),
            bullet(undefined, L("CSV: RFC 4180 export of the analysis data.", "CSV: exportación RFC 4180 de los datos del análisis.")),
            bullet(undefined, L("Preview the report before saving it.", "Vista previa del informe antes de guardarlo.")),
            bullet(undefined, L("Reports are built from stored results; they never rescan files.", "Los informes se generan a partir de resultados guardados; nunca vuelven a analizar archivos.")),
          ])}
        </>
      ),
    },
    {
      id: "troubleshooting",
      title: L("Troubleshooting", "Solución de problemas"),
      blocks: (
        <ul className="list-none space-y-3">
          <Bullet>
            <span className="font-semibold text-zinc-200">
              {L("Portable version does not start:", "La versión portable no arranca:")}
            </span>{" "}
            {L(
              "make sure the WebView2 Runtime is installed.",
              "asegúrate de que el WebView2 Runtime esté instalado.",
            )}
          </Bullet>
          <Bullet>
            <span className="font-semibold text-zinc-200">
              {L(
                "Windows shows a warning when running the application:",
                "Windows muestra un aviso al ejecutar la aplicación:",
              )}
            </span>{" "}
            {L(
              "verify the SHA-256 checksum against the published release before running it. Unsigned, newly compiled binaries are more likely to trigger heuristic warnings.",
              "verifica el checksum SHA-256 contra la versión publicada antes de ejecutarla. Los binarios sin firmar y recién compilados tienen más probabilidades de activar avisos heurísticos.",
            )}
          </Bullet>
          <Bullet>
            <span className="font-semibold text-zinc-200">
              {L(
                "“Analyze with Prometeo” is missing from the context menu:",
                "Falta “Analizar con Prometeo” en el menú contextual:",
              )}
            </span>{" "}
            {L(
              "enable it in Settings; the entry is registered for the current user only.",
              "actívalo en Configuración; la entrada se registra solo para el usuario actual.",
            )}
          </Bullet>
          <Bullet>
            <span className="font-semibold text-zinc-200">
              {L('VirusTotal says "not available":', 'VirusTotal indica “no disponible”:')}
            </span>{" "}
            {L(
              "the hash has no record on VirusTotal. No error occurred; the hash simply has not been reported.",
              "el hash no tiene registros en VirusTotal. No se ha producido ningún error; el hash simplemente no ha sido informado.",
            )}
          </Bullet>
        </ul>
      ),
    },
  ];
}

export function Documentation() {
  const { lang, t } = useLanguage();
  const sections = buildDoc(lang, site.appName);

  return (
    <>
      <Seo title={t("seo.documentation.title")} description={t("seo.documentation.description")} path="/documentation" />

      <div className="mx-auto max-w-6xl px-4 py-16 sm:px-6">
        <p className="text-sm font-semibold text-sky-400">{t("documentation.eyebrow")}</p>
        <h1 className="mt-2 text-4xl font-bold tracking-tight text-white">
          {t("documentation.title")}
        </h1>

        <div className="mt-8 lg:grid lg:grid-cols-[220px_1fr] lg:gap-12">
          <aside className="lg:sticky lg:top-24 lg:self-start">
            <nav
              aria-label={t("documentation.sectionsAria")}
              className="lg:border-l lg:border-line lg:pl-4"
            >
              <ul className="flex gap-2 overflow-x-auto pb-2 lg:flex-col lg:gap-1 lg:overflow-visible lg:pb-0">
                {sections.map((section) => (
                  <li key={section.id} className="shrink-0">
                    <a
                      href={`#${section.id}`}
                      className="block whitespace-nowrap rounded-md px-3 py-1.5 text-sm text-zinc-400 transition-colors hover:bg-ink-2 hover:text-white focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-sky-400 lg:whitespace-normal"
                    >
                      {section.title}
                    </a>
                  </li>
                ))}
              </ul>
            </nav>
          </aside>

          <div className="mt-10 space-y-14 lg:mt-0">
            {sections.map((section) => (
              <Section key={section.id} id={section.id} title={section.title}>
                {section.blocks}
              </Section>
            ))}
          </div>
        </div>
      </div>
    </>
  );
}
