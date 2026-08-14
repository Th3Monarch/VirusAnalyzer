import { useEffect, useState } from "react";
import { ExternalLink, Loader2 } from "lucide-react";
import { Seo } from "../components/Seo";
import { GithubIcon } from "../components/GithubIcon";
import { Markdown } from "../components/Markdown";
import { useLanguage } from "../contexts/LanguageContext";
import type { Lang } from "../lib/i18n";
import { github } from "../config";
import {
  fetchRecentReleases,
  type Release,
} from "../lib/github";

type Status =
  | { state: "loading" }
  | { state: "ready"; releases: Release[] | null };

function formatDate(iso: string, lang: Lang): string {
  const date = new Date(iso);
  if (Number.isNaN(date.getTime())) return iso;
  return new Intl.DateTimeFormat(lang === "es" ? "es-ES" : "en-US", {
    year: "numeric",
    month: "long",
    day: "numeric",
  }).format(date);
}

const copy: Record<Lang, Record<string, string>> = {
  en: { eyebrow: "Changelog", title: "Release history" },
  es: { eyebrow: "Historial de cambios", title: "Historial de versiones" },
};

export function Changelog() {
  const { lang, t } = useLanguage();
  const c = copy[lang];
  const [status, setStatus] = useState<Status>({ state: "loading" });

  useEffect(() => {
    if (!github.configured) {
      setStatus({ state: "ready", releases: null });
      return;
    }
    let cancelled = false;
    fetchRecentReleases(github.owner, github.repository, 10).then((releases) => {
      if (cancelled) return;
      setStatus({ state: "ready", releases });
    });
    return () => {
      cancelled = true;
    };
  }, []);

  const fullHistory = t("changelog.fullHistory").split("{link}");

  return (
    <>
      <Seo title={t("seo.changelog.title")} description={t("seo.changelog.description")} path="/changelog" />

      <section className="mx-auto max-w-3xl px-4 py-16 sm:px-6">
        <p className="text-sm font-semibold text-sky-400">{c.eyebrow}</p>
        <h1 className="mt-2 text-4xl font-bold tracking-tight text-white">
          {c.title}
        </h1>

        <div className="mt-10">
          {status.state === "loading" && (
            <div className="flex items-center gap-3 rounded-xl border border-line bg-ink p-6 text-sm text-zinc-400">
              <Loader2 className="h-5 w-5 animate-spin text-sky-400" aria-hidden="true" />
              {t("changelog.loading")}
            </div>
          )}

          {status.state === "ready" && status.releases === null && (
            <div className="rounded-xl border border-line bg-ink p-6" role="status">
              <h2 className="text-base font-semibold text-white">
                {!github.configured
                  ? t("changelog.notConfigured")
                  : t("changelog.unavailable")}
              </h2>
              <div className="mt-4">
                {github.configured && github.releasesUrl ? (
                  <a
                    href={github.releasesUrl}
                    target="_blank"
                    rel="noopener noreferrer"
                    className="inline-flex items-center gap-2 rounded-lg border border-line-2 bg-ink-2 px-4 py-2.5 text-sm font-semibold text-zinc-100 transition-colors hover:border-sky-500/60 hover:bg-ink-3 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-sky-400"
                  >
                    <GithubIcon className="h-4 w-4" />
                    {t("changelog.viewReleases")}
                  </a>
                ) : (
                  <p className="text-sm text-zinc-500">
                    {t("changelog.configHint")}
                  </p>
                )}
              </div>
            </div>
          )}

          {status.state === "ready" && status.releases?.length === 0 && (
            <p className="rounded-xl border border-line bg-ink p-6 text-sm text-zinc-400">
              {t("changelog.noReleases")}
            </p>
          )}

          {status.state === "ready" && status.releases && status.releases.length > 0 && (
            <div className="space-y-6">
              {status.releases.map((release) => (
                <article
                  key={release.tag_name}
                  className="rounded-xl border border-line bg-ink p-6"
                >
                  <div className="flex flex-wrap items-center gap-2">
                    <h2 className="font-mono text-base font-semibold text-white">
                      {release.tag_name}
                    </h2>
                    {release.prerelease && (
                      <span className="rounded-full border border-amber-400/40 bg-amber-400/10 px-2 py-0.5 text-[11px] font-semibold uppercase tracking-wide text-amber-300">
                        {t("changelog.prerelease")}
                      </span>
                    )}
                    <span className="text-xs text-zinc-500">
                      {formatDate(release.published_at, lang)}
                    </span>
                  </div>
                  {release.name && release.name !== release.tag_name && (
                    <p className="mt-1 text-sm text-zinc-300">{release.name}</p>
                  )}
                  {release.body ? (
                    <div className="mt-4 rounded-lg border border-line bg-night p-4">
                      <Markdown>{release.body}</Markdown>
                    </div>
                  ) : (
                    <p className="mt-3 text-sm text-zinc-500">
                      {t("changelog.noNotes")}
                    </p>
                  )}
                  <a
                    href={release.html_url}
                    target="_blank"
                    rel="noopener noreferrer"
                    className="mt-4 inline-flex items-center gap-1.5 text-sm font-medium text-sky-400 transition-colors hover:text-sky-300 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-sky-400"
                  >
                    {t("changelog.viewOnGitHub")}
                    <ExternalLink className="h-3.5 w-3.5" aria-hidden="true" />
                  </a>
                </article>
              ))}
            </div>
          )}
        </div>

        <p className="mt-10 text-sm text-zinc-500">
          {github.configured && github.releasesUrl ? (
            <>
              {fullHistory[0]}
              <a
                href={github.releasesUrl}
                target="_blank"
                rel="noopener noreferrer"
                className="font-medium text-sky-400 transition-colors hover:text-sky-300 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-sky-400"
              >
                GitHub Releases
              </a>
              {fullHistory[1]}
            </>
          ) : (
            t("changelog.fullHistory").replace("{link}", "GitHub Releases")
          )}
        </p>
      </section>
    </>
  );
}
