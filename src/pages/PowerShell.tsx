import { AlertTriangle, BookOpen } from "lucide-react";
import { Link } from "react-router-dom";
import { useLanguage } from "../contexts/LanguageContext";
import { PageHeader } from "../components/ui/PageHeader";
import { Button } from "../components/ui/Button";
import { Terminal } from "../components/PowerShell/Terminal";

export function PowerShell() {
  const { t } = useLanguage();

  return (
    <div>
      <PageHeader
        title={t("powershell.title")}
        subtitle={t("powershell.subtitle")}
        actions={
          <Link to="/ps-reference">
            <Button variant="secondary">
              <BookOpen className="size-4" />
              {t("powershell.openReference")}
            </Button>
          </Link>
        }
      />

      {/* Advertencia */}
      <div className="mb-6 flex items-start gap-3 rounded-xl border border-warn/40 bg-warn/10 p-4">
        <AlertTriangle className="mt-0.5 size-5 shrink-0 text-warn" />
        <div>
          <p className="text-sm font-semibold text-ink">{t("powershell.warningTitle")}</p>
          <p className="mt-1 max-w-3xl text-sm leading-relaxed text-muted">
            {t("powershell.warningText")}
          </p>
          <p className="mt-1 max-w-3xl text-xs leading-relaxed text-muted/80">
            {t("powershell.permissionNote")}
          </p>
        </div>
      </div>

      <Terminal />
    </div>
  );
}
