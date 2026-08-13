import type { KeyboardEvent, Ref } from "react";
import { SquareTerminal, Trash2, X } from "lucide-react";
import { useLanguage } from "../../contexts/LanguageContext";
import { Button } from "../ui/Button";
import type { RiskLevel } from "../../types";
import { RiskBadge } from "./RiskBadge";

interface CommandInputProps {
  value: string;
  onChange: (value: string) => void;
  onRun: () => void;
  onCancel: () => void;
  onClear: () => void;
  onKeyDown: (event: KeyboardEvent<HTMLTextAreaElement>) => void;
  running: boolean;
  risk: RiskLevel | null;
  inputRef?: Ref<HTMLTextAreaElement>;
}

export function CommandInput({
  value,
  onChange,
  onRun,
  onCancel,
  onClear,
  onKeyDown,
  running,
  risk,
  inputRef,
}: CommandInputProps) {
  const { t } = useLanguage();

  return (
    <div className="rounded-xl border border-line bg-surface-2/60 p-3">
      <div className="flex items-center gap-2">
        <span className="select-none font-mono text-xs font-bold text-accent">{t("powershell.prompt")}</span>
        <textarea
          ref={inputRef}
          value={value}
          onChange={(event) => onChange(event.target.value)}
          onKeyDown={onKeyDown}
          placeholder={t("powershell.placeholder")}
          spellCheck={false}
          rows={2}
          aria-label={t("powershell.placeholder")}
          className="min-h-[3.5rem] flex-1 resize-none bg-transparent font-mono text-[13px] leading-relaxed text-ink placeholder:text-muted/60 focus:outline-none"
        />
      </div>
      <div className="mt-2 flex flex-wrap items-center justify-between gap-2">
        <div className="flex items-center gap-2">
          {running ? (
            <Button variant="danger" onClick={onCancel}>
              <X className="size-4" />
              {t("powershell.cancel")}
            </Button>
          ) : (
            <Button onClick={onRun} disabled={value.trim().length === 0}>
              <SquareTerminal className="size-4" />
              {t("powershell.run")}
            </Button>
          )}
          <Button variant="secondary" onClick={onClear} disabled={running}>
            <Trash2 className="size-4" />
            {t("powershell.clear")}
          </Button>
        </div>
        {risk ? <RiskBadge risk={risk} /> : null}
      </div>
    </div>
  );
}
