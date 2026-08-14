import { useEffect, useState } from "react";
import { Link, NavLink, useLocation } from "react-router-dom";
import { Download, Menu, X } from "lucide-react";
import { Logo } from "./Logo";
import { GithubIcon } from "./GithubIcon";
import { DiscordIcon } from "./DiscordIcon";
import { useLanguage } from "../contexts/LanguageContext";
import { discord, github, site } from "../config";
import type { Lang } from "../lib/i18n";

const navLinks = [
  { to: "/", key: "nav.home" },
  { to: "/features", key: "nav.features" },
  { to: "/security", key: "nav.security" },
  { to: "/documentation", key: "nav.documentation" },
];

function navLinkClass({ isActive }: { isActive: boolean }): string {
  return `rounded-md px-3 py-2 text-sm font-medium transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-sky-400 ${
    isActive ? "text-white" : "text-zinc-400 hover:text-white"
  }`;
}

function LanguageSwitcher({ className }: { className?: string }) {
  const { lang, setLang, t } = useLanguage();
  const options: { value: Lang; label: string }[] = [
    { value: "en", label: "English" },
    { value: "es", label: "Español" },
  ];
  return (
    <div
      role="group"
      aria-label={t("languageSwitcher.label")}
      className={`flex items-center rounded-lg border border-line-2 bg-ink-2 p-0.5 ${className ?? ""}`}
    >
      {options.map((option) => (
        <button
          key={option.value}
          type="button"
          onClick={() => setLang(option.value)}
          aria-pressed={lang === option.value}
          title={option.label}
          aria-label={option.label}
          className={`rounded-md px-2 py-1 text-xs font-semibold transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-sky-400 ${
            lang === option.value
              ? "bg-sky-400 text-ink"
              : "text-zinc-400 hover:text-white"
          }`}
        >
          {option.value.toUpperCase()}
        </button>
      ))}
    </div>
  );
}

export function Navbar() {
  const { t } = useLanguage();
  const [open, setOpen] = useState(false);
  const { pathname } = useLocation();

  useEffect(() => {
    setOpen(false);
  }, [pathname]);

  return (
    <header className="sticky top-0 z-40 border-b border-line bg-night/85 backdrop-blur">
      <div className="mx-auto flex h-16 max-w-6xl items-center justify-between px-4 sm:px-6">
        <Link
          to="/"
          className="flex items-center gap-2.5 rounded-md focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-sky-400"
          aria-label={`${site.appName} — home`}
        >
          <Logo className="h-8 w-8" />
          <span className="text-lg font-semibold tracking-tight text-white">
            {site.appName}
          </span>
        </Link>

        <nav className="hidden items-center gap-1 md:flex" aria-label="Primary">
          {navLinks.map((link) => (
            <NavLink
              key={link.to}
              to={link.to}
              end={link.to === "/"}
              className={navLinkClass}
            >
              {t(link.key)}
            </NavLink>
          ))}
          {github.configured && github.repoUrl && (
            <a
              href={github.repoUrl}
              target="_blank"
              rel="noopener noreferrer"
              className="ml-1 rounded-md p-2 text-zinc-400 transition-colors hover:text-white focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-sky-400"
              aria-label={t("nav.githubAria")}
            >
              <GithubIcon className="h-5 w-5" />
            </a>
          )}
          {discord.configured && discord.inviteUrl && (
            <a
              href={discord.inviteUrl}
              target="_blank"
              rel="noopener noreferrer"
              className="rounded-md p-2 text-zinc-400 transition-colors hover:text-[#5865F2] focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-sky-400"
              aria-label={t("nav.discordAria")}
            >
              <DiscordIcon className="h-5 w-5" />
            </a>
          )}
          <LanguageSwitcher className="ml-1" />
          <Link
            to="/download"
            className="ml-2 inline-flex items-center gap-2 rounded-lg bg-sky-400 px-4 py-2 text-sm font-semibold text-ink transition-colors hover:bg-sky-300 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-sky-400 focus-visible:ring-offset-2 focus-visible:ring-offset-night"
          >
            <Download className="h-4 w-4" aria-hidden="true" />
            {t("nav.download")}
          </Link>
        </nav>

        <div className="flex items-center gap-2 md:hidden">
          <LanguageSwitcher />
          <button
            type="button"
            onClick={() => setOpen((value) => !value)}
            className="rounded-md p-2 text-zinc-300 hover:text-white focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-sky-400"
            aria-expanded={open}
            aria-controls="mobile-menu"
            aria-label={open ? t("nav.closeMenu") : t("nav.openMenu")}
          >
            {open ? (
              <X className="h-6 w-6" aria-hidden="true" />
            ) : (
              <Menu className="h-6 w-6" aria-hidden="true" />
            )}
          </button>
        </div>
      </div>

      {open && (
        <nav
          id="mobile-menu"
          className="border-t border-line bg-ink md:hidden"
          aria-label="Mobile"
        >
          <div className="mx-auto flex max-w-6xl flex-col gap-1 px-4 py-3 sm:px-6">
            {navLinks.map((link) => (
              <NavLink
                key={link.to}
                to={link.to}
                end={link.to === "/"}
                className={navLinkClass}
              >
                {t(link.key)}
              </NavLink>
            ))}
            {github.configured && github.repoUrl && (
              <a
                href={github.repoUrl}
                target="_blank"
                rel="noopener noreferrer"
                className="rounded-md px-3 py-2 text-sm font-medium text-zinc-400 transition-colors hover:text-white focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-sky-400"
              >
                {t("footer.github")}
              </a>
            )}
            {discord.configured && discord.inviteUrl && (
              <a
                href={discord.inviteUrl}
                target="_blank"
                rel="noopener noreferrer"
                className="rounded-md px-3 py-2 text-sm font-medium text-zinc-400 transition-colors hover:text-white focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-sky-400"
              >
                {t("footer.discord")}
              </a>
            )}
            <Link
              to="/download"
              className="mt-2 inline-flex items-center justify-center gap-2 rounded-lg bg-sky-400 px-4 py-2.5 text-sm font-semibold text-ink focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-sky-400 focus-visible:ring-offset-2 focus-visible:ring-offset-night"
            >
              <Download className="h-4 w-4" aria-hidden="true" />
              {t("nav.download")}
            </Link>
          </div>
        </nav>
      )}
    </header>
  );
}
