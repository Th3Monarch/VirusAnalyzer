import { useCallback, useEffect, useState } from "react";
import { useNavigate } from "react-router-dom";
import { ArrowLeft, Search } from "lucide-react";
import { useLanguage } from "../contexts/LanguageContext";
import { useToast } from "../contexts/ToastContext";
import { tauri } from "../lib/tauri";
import type { PsCommandInfo } from "../types";
import { PageHeader } from "../components/ui/PageHeader";
import { Button } from "../components/ui/Button";
import { EmptyState } from "../components/ui/EmptyState";
import { CommandReference } from "../components/PowerShell/CommandReference";

export function PSReference() {
  const { t, language } = useLanguage();
  const { toast } = useToast();
  const navigate = useNavigate();

  const [commands, setCommands] = useState<PsCommandInfo[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [search, setSearch] = useState("");

  useEffect(() => {
    let alive = true;
    setLoading(true);
    setError(null);
    void tauri
      .getPowerShellReference(language)
      .then((data) => {
        if (alive) setCommands(data);
      })
      .catch((err: unknown) => {
        if (alive) setError(err instanceof Error ? err.message : String(err));
      })
      .finally(() => {
        if (alive) setLoading(false);
      });
    return () => {
      alive = false;
    };
  }, [language]);

  const load = useCallback(
    (command: PsCommandInfo) => {
      navigate(`/powershell?cmd=${encodeURIComponent(command.name)}`);
      toast(t("psRef.loaded"), "success");
    },
    [navigate, toast, t],
  );

  return (
    <div>
      <PageHeader
        title={t("psRef.title")}
        subtitle={t("psRef.subtitle")}
        actions={
          <Button variant="secondary" onClick={() => navigate("/powershell")}>
            <ArrowLeft className="size-4" />
            {t("psRef.backToPs")}
          </Button>
        }
      />

      <div className="mb-6 flex items-center gap-2 rounded-xl border border-line bg-surface-2/60 px-3 py-2.5">
        <Search className="size-4 shrink-0 text-muted" />
        <input
          value={search}
          onChange={(event) => setSearch(event.target.value)}
          placeholder={t("psRef.search")}
          aria-label={t("psRef.search")}
          className="w-full bg-transparent text-sm text-ink placeholder:text-muted/60 focus:outline-none"
        />
      </div>

      {loading ? (
        <p role="status" className="py-16 text-center text-sm text-muted">
          …
        </p>
      ) : error ? (
        <EmptyState icon={Search} title={t("common.error")} description={error} />
      ) : (
        <CommandReference commands={commands} search={search} onLoad={load} />
      )}
    </div>
  );
}
