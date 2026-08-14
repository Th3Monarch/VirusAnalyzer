import { Link } from "react-router-dom";
import { Logo } from "./Logo";
import { GithubIcon } from "./GithubIcon";
import { DiscordIcon } from "./DiscordIcon";
import { useLanguage } from "../contexts/LanguageContext";
import { discord, github, site } from "../config";

const year = new Date().getFullYear();

const productLinks = [
  { to: "/download", key: "nav.download" },
  { to: "/features", key: "nav.features" },
  { to: "/security", key: "nav.security" },
  { to: "/documentation", key: "nav.documentation" },
  { to: "/changelog", key: "footer.releases" },
  { to: "/faq", key: "nav.faq" },
  { to: "/about", key: "nav.about" },
];

export function Footer() {
  const { t } = useLanguage();

  return (
    <footer className="border-t border-line bg-ink">
      <div className="mx-auto max-w-6xl px-4 py-12 sm:px-6">
        <div className="grid gap-10 sm:grid-cols-2 md:grid-cols-3">
          <div>
            <div className="flex items-center gap-2.5">
              <Logo className="h-7 w-7" />
              <span className="text-base font-semibold text-white">
                {site.appName}
              </span>
            </div>
            <p className="mt-3 text-sm text-zinc-400">{t("footer.tagline")}.</p>
            <p className="mt-1 text-xs leading-relaxed text-zinc-500">
              {t("footer.description")}
            </p>
          </div>

          <nav aria-label={t("footer.product")}>
            <h2 className="text-xs font-semibold uppercase tracking-wider text-zinc-500">
              {t("footer.product")}
            </h2>
            <ul className="mt-4 space-y-2.5">
              {productLinks.map((link) => (
                <li key={link.to}>
                  <Link
                    to={link.to}
                    className="text-sm text-zinc-400 transition-colors hover:text-white focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-sky-400"
                  >
                    {t(link.key)}
                  </Link>
                </li>
              ))}
            </ul>
          </nav>

          <nav aria-label={t("footer.community")}>
            <h2 className="text-xs font-semibold uppercase tracking-wider text-zinc-500">
              {t("footer.community")}
            </h2>
            <ul className="mt-4 space-y-2.5">
              {github.configured && github.repoUrl && (
                <li>
                  <a
                    href={github.repoUrl}
                    target="_blank"
                    rel="noopener noreferrer"
                    className="inline-flex items-center gap-2 text-sm text-zinc-400 transition-colors hover:text-white focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-sky-400"
                  >
                    <GithubIcon className="h-4 w-4" />
                    {t("footer.github")}
                  </a>
                </li>
              )}
              {discord.configured && discord.inviteUrl && (
                <li>
                  <a
                    href={discord.inviteUrl}
                    target="_blank"
                    rel="noopener noreferrer"
                    className="inline-flex items-center gap-2 text-sm text-zinc-400 transition-colors hover:text-white focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-sky-400"
                  >
                    <DiscordIcon className="h-4 w-4" />
                    {t("footer.discord")}
                  </a>
                </li>
              )}
              {github.configured && github.releasesUrl && (
                <li>
                  <a
                    href={github.releasesUrl}
                    target="_blank"
                    rel="noopener noreferrer"
                    className="text-sm text-zinc-400 transition-colors hover:text-white focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-sky-400"
                  >
                    {t("footer.releases")}
                  </a>
                </li>
              )}
              {github.configured && github.issuesUrl && (
                <li>
                  <a
                    href={github.issuesUrl}
                    target="_blank"
                    rel="noopener noreferrer"
                    className="text-sm text-zinc-400 transition-colors hover:text-white focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-sky-400"
                  >
                    {t("footer.issues")}
                  </a>
                </li>
              )}
              {!discord.configured && (
                <li className="text-sm text-zinc-600">
                  {t("footer.configHint")}
                </li>
              )}
            </ul>
          </nav>
        </div>

        <div className="mt-10 border-t border-line pt-6 text-xs leading-relaxed text-zinc-500">
          <p>{t("footer.copyright").replace("{year}", String(year)).replace("{app}", site.appName)}</p>
          <p className="mt-1">{t("footer.disclaimer")}</p>
        </div>
      </div>
    </footer>
  );
}
