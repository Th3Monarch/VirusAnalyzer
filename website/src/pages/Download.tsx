import { useState } from "react";
import { Check, Copy, Download as DownloadIcon, FolderArchive, Loader2, Monitor, Apple, Terminal } from "lucide-react";
import { Seo } from "../components/Seo";
import { Reveal } from "../components/Reveal";
import { GithubIcon } from "../components/GithubIcon";
import { Markdown } from "../components/Markdown";
import { useLatestRelease } from "../hooks/useLatestRelease";
import { useLanguage } from "../contexts/LanguageContext";
import { github, site } from "../config";
import type { DownloadAsset } from "../lib/github";

function formatBytes(bytes: number): string {
  if (!bytes) return "";
  const units = ["B", "KB", "MB", "GB"];
  let value = bytes;
  let unit = 0;
  while (value >= 1024 && unit < units.length - 1) {
    value /= 1024;
    unit += 1;
  }
  return `${value.toFixed(value >= 10 || unit === 0 ? 0 : 1)} ${units[unit]}`;
}

function CopyButton({ value, label }: { value: string; label: string }) {
  const { t } = useLanguage();
  const [copied, setCopied] = useState(false);

  const copy = async () => {
    try {
      await navigator.clipboard.writeText(value);
      setCopied(true);
      window.setTimeout(() => setCopied(false), 2000);
    } catch {
      setCopied(false);
    }
  };

  return (
    <button
      type="button"
      onClick={copy}
      className="inline-flex items-center gap-1.5 rounded-md border border-line-2 bg-ink-2 px-2.5 py-1.5 text-xs font-medium text-zinc-300 transition-colors hover:border-sky-500/60 hover:text-white focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-sky-400"
      aria-label={label}
    >
      {copied ? (
        <Check className="h-3.5 w-3.5 text-emerald-400" aria-hidden="true" />
      ) : (
        <Copy className="h-3.5 w-3.5" aria-hidden="true" />
      )}
      {copied ? t("common.copied") : t("common.copy")}
    </button>
  );
}

function DownloadCard({
  asset,
  cta,
  recommended = false,
}: {
  asset: DownloadAsset;
  cta: string;
  recommended?: boolean;
}) {
  const { t } = useLanguage();
  return (
    <div
      className={`relative h-full rounded-xl border bg-ink p-6 ${
        recommended ? "border-sky-500/50" : "border-line"
      }`}
    >
      {recommended && (
        <span className="absolute -top-3 left-5 rounded-full bg-sky-400 px-2.5 py-0.5 text-[11px] font-bold uppercase tracking-wide text-ink">
          {t("download.recommended")}
        </span>
      )}
      <h3 className="text-base font-semibold text-white">{asset.displayName}</h3>
      <p className="mt-1 font-mono text-xs text-zinc-500">
        {asset.name} · {formatBytes(asset.size)}
      </p>
      <a
        href={asset.url}
        target="_blank"
        rel="noopener noreferrer"
        className={`mt-4 inline-flex w-full items-center justify-center gap-2 rounded-lg px-4 py-2.5 text-sm font-semibold transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-sky-400 focus-visible:ring-offset-2 focus-visible:ring-offset-night ${
          recommended
            ? "bg-sky-400 text-ink hover:bg-sky-300"
            : "border border-line-2 bg-ink-2 text-zinc-100 hover:border-sky-500/60 hover:bg-ink-3"
        }`}
      >
        <DownloadIcon className="h-4 w-4" aria-hidden="true" />
        {cta}
      </a>
    </div>
  );
}

export function Download() {
  const { t } = useLanguage();
  const release = useLatestRelease();
  const bundle = release.status === "ready" ? release.bundle : null;
  const version = bundle?.version ?? site.fallbackVersion;

  const setup = bundle?.downloads.find((asset) => asset.kind === "setup");
  const portable = bundle?.downloads.find((asset) => asset.kind === "portable");
  const deb = bundle?.downloads.find((asset) => asset.kind === "deb");
  const appimage = bundle?.downloads.find((asset) => asset.kind === "appimage");
  const dmg = bundle?.downloads.filter((asset) => asset.kind === "dmg");
  const dmgX64 = dmg?.find((a) => a.name.includes("x64"));
  const dmgAarch64 = dmg?.find((a) => a.name.includes("aarch64"));

  const hasWindows = !!(setup || portable);
  const hasMacOS = dmg && dmg.length > 0;
  const hasLinux = !!(deb || appimage);

  const releaseUnavailable =
    release.status === "ready" && (!bundle || bundle.downloads.length === 0);

  return (
    <>
      <Seo title={t("seo.download.title")} description={t("seo.download.description")} path="/download" />

      <section className="mx-auto max-w-6xl px-4 py-16 sm:px-6">
        <p className="text-sm font-semibold text-sky-400">{t("download.eyebrow")}</p>
        <h1 className="mt-2 text-4xl font-bold tracking-tight text-white">
          {t("download.title").replace("{app}", site.appName)}
        </h1>
        <div className="mt-4 flex flex-wrap items-center gap-2 text-sm">
          <span className="rounded-md border border-line-2 bg-ink-2 px-2.5 py-1 font-mono text-zinc-200">
            v{version}
          </span>
          <span className="rounded-md border border-line-2 bg-ink-2 px-2.5 py-1 text-zinc-300">
            Windows · macOS · Linux
          </span>
          <span className="text-zinc-500">
            {t("download.requirements")}
          </span>
        </div>

        <div className="mt-10">
          {release.status === "loading" && (
            <div className="flex items-center gap-3 rounded-xl border border-line bg-ink p-6 text-sm text-zinc-400">
              <Loader2 className="h-5 w-5 animate-spin text-sky-400" aria-hidden="true" />
              {t("download.loading")}
            </div>
          )}

          {releaseUnavailable && (
            <div
              className="rounded-xl border border-line bg-ink p-6"
              role="status"
            >
              <h2 className="text-base font-semibold text-white">
                {t("download.unavailableTitle")}
              </h2>
              <p className="mt-2 max-w-2xl text-sm leading-relaxed text-zinc-400">
                {t("download.unavailableDesc")}
              </p>
              <div className="mt-4 flex flex-wrap gap-3">
                {github.configured && github.releasesUrl && (
                  <a
                    href={github.releasesUrl}
                    target="_blank"
                    rel="noopener noreferrer"
                    className="inline-flex items-center gap-2 rounded-lg border border-line-2 bg-ink-2 px-4 py-2.5 text-sm font-semibold text-zinc-100 transition-colors hover:border-sky-500/60 hover:bg-ink-3 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-sky-400"
                  >
                    <GithubIcon className="h-4 w-4" />
                    {t("changelog.viewReleases")}
                  </a>
                )}
                {!github.configured && (
                  <p className="text-sm text-zinc-500">
                    {t("download.githubConfigHint")}
                  </p>
                )}
              </div>
            </div>
          )}

          {bundle && (hasWindows || hasMacOS || hasLinux) && (
            <Reveal>
              <div className="space-y-8">
                {hasWindows && (
                  <div className="rounded-xl border border-line bg-ink p-6">
                    <div className="flex items-center gap-3">
                      <Monitor className="h-5 w-5 text-sky-400" aria-hidden="true" />
                      <h3 className="text-lg font-semibold text-white">Windows</h3>
                    </div>
                    <div className="mt-4 grid gap-4 sm:grid-cols-2">
                      {setup && (
                        <DownloadCard asset={setup} cta={t("download.downloadInstaller")} recommended />
                      )}
                      {portable && (
                        <DownloadCard asset={portable} cta={t("download.downloadPortable")} />
                      )}
                    </div>
                  </div>
                )}

                {hasMacOS && (
                  <div className="rounded-xl border border-line bg-ink p-6">
                    <div className="flex items-center gap-3">
                      <Apple className="h-5 w-5 text-sky-400" aria-hidden="true" />
                      <h3 className="text-lg font-semibold text-white">macOS</h3>
                    </div>
                    <div className="mt-4 grid gap-4 sm:grid-cols-2">
                      {dmgX64 && (
                        <DownloadCard asset={dmgX64} cta={`${t("download.downloadDmg")} (Intel)`} recommended />
                      )}
                      {dmgAarch64 && (
                        <DownloadCard asset={dmgAarch64} cta={`${t("download.downloadDmg")} (Apple Silicon)`} />
                      )}
                      {!dmgX64 && !dmgAarch64 && dmg[0] && (
                        <DownloadCard asset={dmg[0]} cta={t("download.downloadDmg")} recommended />
                      )}
                    </div>
                  </div>
                )}

                {hasLinux && (
                  <div className="rounded-xl border border-line bg-ink p-6">
                    <div className="flex items-center gap-3">
                      <Terminal className="h-5 w-5 text-sky-400" aria-hidden="true" />
                      <h3 className="text-lg font-semibold text-white">Linux</h3>
                    </div>
                    <div className="mt-4 grid gap-4 sm:grid-cols-2">
                      {deb && (
                        <DownloadCard asset={deb} cta={t("download.downloadDeb")} recommended />
                      )}
                      {appimage && (
                        <DownloadCard asset={appimage} cta={t("download.downloadAppimage")} />
                      )}
                    </div>
                  </div>
                )}

                {github.configured && github.repoUrl && (
                  <div className="flex h-full flex-col rounded-xl border border-line bg-ink p-6">
                    <FolderArchive className="h-6 w-6 text-sky-400" aria-hidden="true" />
                    <h3 className="mt-3 text-base font-semibold text-white">
                      {t("download.sourceTitle")}
                    </h3>
                    <p className="mt-1 text-sm leading-relaxed text-zinc-400">
                      {t("download.sourceDesc")}
                    </p>
                    <a
                      href={github.repoUrl}
                      target="_blank"
                      rel="noopener noreferrer"
                      className="mt-4 inline-flex items-center justify-center gap-2 rounded-lg border border-line-2 bg-ink-2 px-4 py-2.5 text-sm font-semibold text-zinc-100 transition-colors hover:border-sky-500/60 hover:bg-ink-3 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-sky-400"
                    >
                      <GithubIcon className="h-4 w-4" />
                      {t("nav.githubAria")}
                    </a>
                  </div>
                )}
              </div>
            </Reveal>
          )}
        </div>

        {bundle && (
          <section className="mt-16" aria-labelledby="verify-title">
            <h2 id="verify-title" className="text-2xl font-bold tracking-tight text-white">
              {t("download.verifyTitle")}
            </h2>
            <p className="mt-3 max-w-2xl text-sm leading-relaxed text-zinc-400">
              {t("download.verifyDesc")}
            </p>
            <pre className="mt-4 overflow-x-auto rounded-lg border border-line bg-night p-4 font-mono text-xs leading-relaxed text-zinc-300">
              certutil -hashfile Prometeo-{version}-Setup.exe SHA256
            </pre>

            {bundle.checksums.length > 0 ? (
              <ul className="mt-6 space-y-3">
                {bundle.checksums.map((checksum) => (
                  <li
                    key={checksum.targetName}
                    className="flex flex-col gap-2 rounded-xl border border-line bg-ink p-4 sm:flex-row sm:items-center sm:justify-between"
                  >
                    <div className="min-w-0">
                      <p className="truncate font-mono text-xs text-zinc-400">
                        {checksum.targetName}
                      </p>
                      <p className="mt-1 break-all font-mono text-xs text-zinc-200">
                        {checksum.hash}
                      </p>
                    </div>
                    <CopyButton
                      value={checksum.hash}
                      label={t("download.copySha256").replace("{name}", checksum.targetName)}
                    />
                  </li>
                ))}
              </ul>
            ) : (
              <p className="mt-4 text-sm text-zinc-500">
                {t("download.checksumsMissing")}
              </p>
            )}
          </section>
        )}

        {bundle && bundle.body && (
          <section className="mt-16" aria-labelledby="release-notes-title">
            <h2
              id="release-notes-title"
              className="text-2xl font-bold tracking-tight text-white"
            >
              {t("download.releaseNotes")}
            </h2>
            <div className="mt-4 rounded-xl border border-line bg-ink p-6">
              <Markdown>{bundle.body}</Markdown>
            </div>
          </section>
        )}
      </section>
    </>
  );
}
