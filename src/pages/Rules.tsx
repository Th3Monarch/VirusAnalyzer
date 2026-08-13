import { useEffect, useState } from "react";
import { Box, ListChecks, Network, ScrollText, ShieldCheck, Terminal, Workflow } from "lucide-react";
import { useLanguage } from "../contexts/LanguageContext";
import { PageHeader } from "../components/ui/PageHeader";
import { Card } from "../components/ui/Card";
import { SeverityBadge } from "../components/ui/Badge";
import { EmptyState } from "../components/ui/EmptyState";
import { tauri } from "../lib/tauri";
import type { RuleCategory, RuleInfo } from "../types";
import type { TranslationKey } from "../lib/i18n";

const CATEGORY_META: Array<{
  key: RuleCategory;
  icon: typeof Box;
  title: TranslationKey;
  desc: TranslationKey;
}> = [
  { key: "process", icon: Workflow, title: "rules.category.process", desc: "rules.category.processDesc" },
  { key: "persistence", icon: ShieldCheck, title: "rules.category.persistence", desc: "rules.category.persistenceDesc" },
  { key: "powershell", icon: Terminal, title: "rules.category.powershell", desc: "rules.category.powershellDesc" },
  { key: "packing", icon: Box, title: "rules.category.packing", desc: "rules.category.packingDesc" },
  { key: "network", icon: Network, title: "rules.category.network", desc: "rules.category.networkDesc" },
  { key: "signatures", icon: ScrollText, title: "rules.category.signatures", desc: "rules.category.signaturesDesc" },
  { key: "general", icon: ListChecks, title: "rules.category.general", desc: "rules.category.generalDesc" },
];

export function Rules() {
  const { t } = useLanguage();
  const [rules, setRules] = useState<RuleInfo[] | null>(null);
  const [error, setError] = useState(false);

  useEffect(() => {
    let mounted = true;
    void tauri
      .getRules()
      .then((list) => {
        if (mounted) setRules(list);
      })
      .catch(() => {
        if (mounted) setError(true);
      });
    return () => {
      mounted = false;
    };
  }, []);

  return (
    <div>
      <PageHeader
        title={t("rules.title")}
        subtitle={t("rules.subtitle")}
        actions={
          rules ? (
            <span className="text-xs font-medium text-muted">
              {rules.length} {t("rules.ruleCount")}
            </span>
          ) : undefined
        }
      />

      <Card className="mb-6 p-5">
        <p className="text-sm leading-relaxed text-muted">{t("rules.description")}</p>
      </Card>

      {error ? (
        <Card>
          <EmptyState icon={ListChecks} title={t("rules.error")} />
        </Card>
      ) : rules === null ? (
        <Card>
          <p className="py-8 text-center text-sm text-muted">{t("common.loading")}</p>
        </Card>
      ) : (
        <div className="space-y-8">
          {CATEGORY_META.map((cat) => {
            const items = rules.filter((r) => r.category === cat.key);
            if (items.length === 0) return null;
            return (
              <section key={cat.key}>
                <div className="mb-3 flex items-start gap-3">
                  <div className="flex size-9 shrink-0 items-center justify-center rounded-lg border border-line bg-surface-2 text-accent">
                    <cat.icon className="size-4.5" />
                  </div>
                  <div className="min-w-0">
                    <div className="flex items-center gap-2">
                      <p className="text-sm font-semibold text-ink">{t(cat.title)}</p>
                      <span className="text-[11px] text-muted">{items.length}</span>
                    </div>
                    <p className="mt-0.5 text-xs leading-relaxed text-muted">{t(cat.desc)}</p>
                  </div>
                </div>

                <div className="grid grid-cols-1 gap-3 md:grid-cols-2 xl:grid-cols-3">
                  {items.map((rule) => (
                    <Card key={rule.id} className="p-4">
                      <div className="flex items-center justify-between gap-2">
                        <p className="min-w-0 truncate text-xs font-semibold text-ink" title={rule.name}>
                          {rule.name}
                        </p>
                        <SeverityBadge severity={rule.severity} />
                      </div>
                      <p className="mt-2 min-h-8 text-xs leading-relaxed text-muted">{rule.description}</p>
                      <div className="mt-3 flex items-center justify-between gap-2 border-t border-line pt-2.5">
                        <code className="truncate font-mono text-[10px] text-muted">{rule.id}</code>
                        <span className="shrink-0 font-mono text-[11px] font-semibold text-ink/70">
                          +{rule.points} {t("rules.points")}
                        </span>
                      </div>
                    </Card>
                  ))}
                </div>
              </section>
            );
          })}
        </div>
      )}
    </div>
  );
}
