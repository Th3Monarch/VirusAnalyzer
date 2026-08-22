import { useEffect, useRef } from "react";
import { useLanguage } from "../../contexts/LanguageContext";
import { AlertTriangle } from "lucide-react";

interface ConfirmationDialogProps {
  title: string;
  message: string;
  confirmLabel?: string;
  cancelLabel?: string;
  onConfirm: () => void;
  onCancel: () => void;
  variant?: "warning" | "danger";
}

export function ConfirmationDialog({
  title,
  message,
  confirmLabel,
  cancelLabel,
  onConfirm,
  onCancel,
  variant = "warning",
}: ConfirmationDialogProps) {
  const { t } = useLanguage();
  const confirmRef = useRef<HTMLButtonElement>(null);

  useEffect(() => {
    confirmRef.current?.focus();
  }, []);

  const colors =
    variant === "danger"
      ? "border-critical/30 bg-critical/10"
      : "border-warn/30 bg-warn/10";

  const iconColor = variant === "danger" ? "text-critical" : "text-warn";

  return (
    <div className="fixed inset-0 z-[100] flex items-center justify-center bg-black/50" role="alertdialog" aria-modal="true" aria-label={title}>
      <div className={`mx-4 w-full max-w-sm rounded-xl border ${colors} bg-surface p-5 shadow-2xl`}>
        <div className="mb-3 flex items-center gap-3">
          <AlertTriangle className={`size-5 shrink-0 ${iconColor}`} />
          <h2 className="text-sm font-semibold text-ink">{title}</h2>
        </div>
        <p className="mb-5 text-sm leading-relaxed text-muted">{message}</p>
        <div className="flex justify-end gap-2">
          <button
            onClick={onCancel}
            className="rounded-lg bg-surface-2 px-4 py-2 text-sm font-medium text-muted transition-colors hover:bg-line hover:text-ink"
          >
            {cancelLabel ?? t("assistant.cancel")}
          </button>
          <button
            ref={confirmRef}
            onClick={onConfirm}
            className={`rounded-lg px-4 py-2 text-sm font-medium text-white transition-colors ${
              variant === "danger"
                ? "bg-critical hover:bg-critical/90"
                : "bg-warn hover:bg-warn/90"
            }`}
          >
            {confirmLabel ?? t("assistant.confirm")}
          </button>
        </div>
      </div>
    </div>
  );
}
