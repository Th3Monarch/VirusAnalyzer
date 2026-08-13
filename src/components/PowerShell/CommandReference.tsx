import { useMemo } from "react";
import { AlertTriangle, BookOpen, CornerDownLeft } from "lucide-react";
import { useLanguage } from "../../contexts/LanguageContext";
import type { TranslationKey } from "../../lib/i18n";
import type { PsCommandInfo } from "../../types";
import { EmptyState } from "../ui/EmptyState";
import { Button } from "../ui/Button";
import { RiskBadge } from "./RiskBadge";

const CATEGORY_ORDER = [
  "system",
  "processes",
  "services",
  "networking",
  "files",
  "security",
  "diagnostics",
];

const CATEGORY_LABEL: Record<string, TranslationKey> = {
  system: "psRef.category.system",
  processes: "psRef.category.processes",
  services: "psRef.category.services",
  networking: "psRef.category.networking",
  files: "psRef.category.files",
  security: "psRef.category.security",
  diagnostics: "psRef.category.diagnostics",
};

interface CommandReferenceProps {
  commands: PsCommandInfo[];
  search: string;
  onLoad: (command: PsCommandInfo) => void;
}

export function CommandReference({ commands, search, onLoad }: CommandReferenceProps) {
  const { t } = useLanguage();

  const grouped = useMemo(() => {
    const query = search.trim().toLowerCase();
    const filtered = query
      ? commands.filter(
          (c) =>
            c.name.toLowerCase().includes(query) ||
            c.description.toLowerCase().includes(query) ||
            c.category.toLowerCase().includes(query),
        )
      : commands;
    const map = new Map<string, PsCommandInfo[]>();
    for (const command of filtered) {
      const list = map.get(command.category) ?? [];
      list.push(command);
      map.set(command.category, list);
    }
    return CATEGORY_ORDER.filter((cat) => map.has(cat)).map((cat) => [cat, map.get(cat)!] as const);
  }, [commands, search]);

  if (commands.length === 0) {
    return <EmptyState icon={BookOpen} title={t("psRef.title")} description={t("psRef.empty")} />;
  }

  if (grouped.length === 0) {
    return <EmptyState icon={BookOpen} title={t("psRef.empty")} />;
  }

  return (
    <div className="space-y-6">
      {grouped.map(([category, items]) => (
        <section key={category}>
          <h2 className="mb-3 text-xs font-semibold uppercase tracking-wider text-muted">
            {t(CATEGORY_LABEL[category] ?? "psRef.category.system")}
          </h2>
          <div className="grid grid-cols-1 gap-3 lg:grid-cols-2">
            {items.map((command) => (
              <div key={command.name} className="rounded-xl border border-line bg-surface p-4">
                <div className="flex items-start justify-between gap-3">
                  <div className="min-w-0">
                    <p className="font-mono text-sm font-bold text-ink">{command.name}</p>
                    <p className="mt-1 text-xs leading-relaxed text-muted">{command.description}</p>
                  </div>
                  <RiskBadge risk={command.risk} className="shrink-0" />
                </div>
                <div className="mt-3 space-y-1.5 text-xs">
                  <p>
                    <span className="font-semibold text-muted">{t("psRef.usage")}: </span>
                    <code className="rounded bg-surface-2 px-1.5 py-0.5 font-mono text-[11px] text-accent">
                      {command.usage}
                    </code>
                  </p>
                  <p>
                    <span className="font-semibold text-muted">{t("psRef.example")}: </span>
                    <code className="rounded bg-surface-2 px-1.5 py-0.5 font-mono text-[11px] text-ink">
                      {command.example}
                    </code>
                  </p>
                  {command.warning ? (
                    <p className="flex items-start gap-1.5 pt-1 text-warn">
                      <AlertTriangle className="mt-0.5 size-3.5 shrink-0" />
                      {command.warning}
                    </p>
                  ) : null}
                </div>
                <div className="mt-3 flex justify-end">
                  <Button
                    variant="secondary"
                    className="px-2.5 py-1 text-[11px]"
                    onClick={() => onLoad(command)}
                  >
                    <CornerDownLeft className="size-3.5" />
                    {t("psRef.load")}
                  </Button>
                </div>
              </div>
            ))}
          </div>
        </section>
      ))}
    </div>
  );
}
