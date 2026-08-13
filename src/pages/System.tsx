import { useEffect, useState } from "react";
import { Loader2, XCircle } from "lucide-react";
import { useLanguage } from "../contexts/LanguageContext";
import { tauri } from "../lib/tauri";
import { formatBytes } from "../lib/format";
import type { SystemInfo } from "../types";
import { PageHeader } from "../components/ui/PageHeader";
import { Card, CardHeader } from "../components/ui/Card";
import { InfoList, InfoRow } from "../components/ui/InfoList";
import { EmptyState } from "../components/ui/EmptyState";

export function System() {
  const { t } = useLanguage();
  const [info, setInfo] = useState<SystemInfo | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    void tauri
      .getSystemInfo()
      .then(setInfo)
      .catch((e) => setError(e instanceof Error ? e.message : String(e)));
  }, []);

  return (
    <div>
      <PageHeader title={t("system.title")} subtitle={t("system.subtitle")} />

      <div className="grid grid-cols-1 gap-6 xl:grid-cols-2">
        <Card>
          <CardHeader title={t("system.os")} />
          <div className="px-5 py-2">
            {error ? (
              <EmptyState icon={XCircle} title={t("common.error")} description={error} />
            ) : !info ? (
              <div className="flex items-center gap-2 py-4 text-sm text-muted">
                <Loader2 className="size-4 animate-spin" />
                {t("common.loading")}
              </div>
            ) : (
              <InfoList>
                <InfoRow label={t("system.os")} value={`${info.osName} ${info.osVersion}`} />
                <InfoRow label={t("system.edition")} value={info.osEdition ?? t("common.unknown")} />
                <InfoRow label={t("system.architecture")} value={info.architecture} mono />
                <InfoRow label={t("system.version")} value={info.osVersion} mono />
              </InfoList>
            )}
          </div>
        </Card>

        <Card>
          <CardHeader title={t("system.cpu")} />
          <div className="px-5 py-2">
            {error ? (
              <EmptyState icon={XCircle} title={t("common.error")} description={error} />
            ) : !info ? (
              <div className="flex items-center gap-2 py-4 text-sm text-muted">
                <Loader2 className="size-4 animate-spin" />
                {t("common.loading")}
              </div>
            ) : (
              <InfoList>
                <InfoRow label={t("system.cpuCores")} value={String(info.cpuPhysicalCores)} />
                <InfoRow label={t("system.virtualCores")} value={String(info.cpuVirtualCores)} />
                <InfoRow label={t("system.memory")} value={formatBytes(info.totalMemoryBytes)} />
              </InfoList>
            )}
          </div>
        </Card>

        <Card className="xl:col-span-2">
          <CardHeader title={t("nav.system")} />
          <div className="px-5 py-2">
            {error ? (
              <EmptyState icon={XCircle} title={t("common.error")} description={error} />
            ) : !info ? (
              <div className="flex items-center gap-2 py-4 text-sm text-muted">
                <Loader2 className="size-4 animate-spin" />
                {t("common.loading")}
              </div>
            ) : (
              <div className="grid grid-cols-1 gap-x-8 sm:grid-cols-2">
                <InfoList>
                  <InfoRow label={t("system.hostname")} value={info.hostname} mono />
                  <InfoRow label={t("system.username")} value={info.username} mono />
                </InfoList>
                <InfoList>
                  <InfoRow label={t("system.os")} value={`${info.osName} ${info.osVersion}`} />
                  <InfoRow label={t("system.edition")} value={info.osEdition ?? t("common.unknown")} />
                </InfoList>
              </div>
            )}
          </div>
        </Card>
      </div>
    </div>
  );
}
