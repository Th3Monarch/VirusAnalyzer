import { Link } from "react-router-dom";
import { Seo } from "../components/Seo";
import { useLanguage } from "../contexts/LanguageContext";

export function NotFound() {
  const { t } = useLanguage();

  return (
    <>
      <Seo title={t("seo.notFound.title")} description={t("seo.notFound.description")} path="/404" />

      <section className="mx-auto max-w-2xl px-4 py-24 text-center sm:px-6">
        <p className="font-mono text-sm font-semibold text-sky-400">{t("notFound.code")}</p>
        <h1 className="mt-2 text-4xl font-bold tracking-tight text-white">
          {t("notFound.title")}
        </h1>
        <p className="mt-4 text-base text-zinc-400">
          {t("notFound.description")}
        </p>
        <Link
          to="/"
          className="mt-8 inline-flex items-center justify-center gap-2 rounded-lg bg-sky-400 px-5 py-2.5 text-sm font-semibold text-ink transition-colors hover:bg-sky-300 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-sky-400 focus-visible:ring-offset-2 focus-visible:ring-offset-night"
        >
          {t("notFound.backHome")}
        </Link>
      </section>
    </>
  );
}
