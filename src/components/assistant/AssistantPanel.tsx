import { useCallback, useEffect, useRef, useState } from "react";
import { useAssistant } from "../../contexts/AssistantContext";
import { useLanguage } from "../../contexts/LanguageContext";
import { AssistantInput } from "./AssistantInput";
import { AssistantMessage } from "./AssistantMessage";
import { AssistantSuggestions } from "./AssistantSuggestions";
import { AssistantTyping } from "./AssistantTyping";
import { AssistantHUD } from "./AssistantHUD";
import { AssistantSettings } from "./AssistantSettings";
import { ConfirmationDialog } from "./ConfirmationDialog";
import { X, Trash2, BotMessageSquare, Settings, VolumeX, Volume2, AlertCircle } from "lucide-react";

export function AssistantPanel() {
  const { messages, isOpen, setOpen, isLoading, clearSession, pendingConfirmation, confirmAction, cancelAction, error, silentMode, toggleSilentMode } =
    useAssistant();
  const { t } = useLanguage();
  const messagesEndRef = useRef<HTMLDivElement>(null);
  const messagesContainerRef = useRef<HTMLDivElement>(null);
  const inputRef = useRef<HTMLTextAreaElement>(null);
  const [showSettings, setShowSettings] = useState(false);
  const [confirmClear, setConfirmClear] = useState(false);
  const isUserScrolledUp = useRef(false);

  const handleScroll = useCallback(() => {
    const container = messagesContainerRef.current;
    if (!container) return;
    const threshold = 50;
    isUserScrolledUp.current =
      container.scrollTop + container.clientHeight < container.scrollHeight - threshold;
  }, []);

  useEffect(() => {
    if (!isUserScrolledUp.current) {
      messagesEndRef.current?.scrollIntoView({ behavior: "smooth" });
    }
  }, [messages, isLoading]);

  useEffect(() => {
    if (isOpen) {
      const timer = setTimeout(() => inputRef.current?.focus(), 100);
      return () => clearTimeout(timer);
    }
  }, [isOpen]);

  useEffect(() => {
    setConfirmClear(false);
  }, [messages.length]);

  useEffect(() => {
    if (!isOpen) return;
    const handler = (e: KeyboardEvent) => {
      if (e.key === "Escape") {
        if (showSettings) {
          setShowSettings(false);
        } else if (pendingConfirmation) {
          void cancelAction();
        } else {
          setOpen(false);
        }
      }
      if (e.key === "/" && !showSettings && document.activeElement?.tagName !== "TEXTAREA") {
        e.preventDefault();
        inputRef.current?.focus();
      }
    };
    window.addEventListener("keydown", handler);
    return () => window.removeEventListener("keydown", handler);
  }, [isOpen, setOpen, showSettings, pendingConfirmation, cancelAction]);

  const handleClearSession = useCallback(() => {
    setConfirmClear(true);
  }, []);

  const handleConfirmClear = useCallback(async () => {
    await clearSession();
    setConfirmClear(false);
  }, [clearSession]);

  if (!isOpen) return null;

  const showSuggestions = messages.length === 0 && !showSettings;

  return (
    <>
      {confirmClear && (
        <ConfirmationDialog
          title={t("assistant.confirmClearTitle")}
          message={t("assistant.confirmClearMessage")}
          variant="danger"
          onConfirm={() => void handleConfirmClear()}
          onCancel={() => setConfirmClear(false)}
        />
      )}

      <div
        className="assistant-panel fixed bottom-4 right-4 z-50 flex w-96 max-w-[calc(100vw-2rem)] flex-col rounded-2xl border border-line bg-surface shadow-2xl animate-va-pop"
        role="dialog"
        aria-label={t("assistant.panelTitle")}
      >
        <div className="flex items-center justify-between border-b border-line px-4 py-3">
          <div className="flex items-center gap-2.5">
            <div className="flex size-8 items-center justify-center rounded-lg bg-accent/15 text-accent">
              <BotMessageSquare className="size-4" />
            </div>
            <div className="min-w-0">
              <p className="text-sm font-semibold text-ink truncate">{t("assistant.panelTitle")}</p>
              <p className="text-[10px] text-muted">{t("assistant.subtitle")}</p>
            </div>
          </div>
          <div className="flex items-center gap-1">
            <button
              onClick={() => void toggleSilentMode()}
              className={`rounded-lg p-1.5 transition-colors ${
                silentMode ? "bg-warn/15 text-warn" : "text-muted hover:bg-surface-2 hover:text-ink"
              }`}
              title={silentMode ? t("assistant.silentModeEnabled") : t("assistant.silentModeDisabled")}
              aria-label={silentMode ? t("assistant.silentModeEnabled") : t("assistant.silentModeDisabled")}
              aria-pressed={silentMode}
            >
              {silentMode ? <VolumeX className="size-3.5" /> : <Volume2 className="size-3.5" />}
            </button>
            <button
              onClick={() => setShowSettings((prev) => !prev)}
              className={`rounded-lg p-1.5 transition-colors ${
                showSettings ? "bg-accent/15 text-accent" : "text-muted hover:bg-surface-2 hover:text-ink"
              }`}
              title={t("assistant.settings") ?? "Settings"}
              aria-label={t("assistant.settings") ?? "Settings"}
            >
              <Settings className="size-3.5" />
            </button>
            <button
              onClick={handleClearSession}
              className="rounded-lg p-1.5 text-muted transition-colors hover:bg-surface-2 hover:text-ink"
              title={t("assistant.clear") ?? "Clear"}
              aria-label={t("assistant.clear") ?? "Clear"}
            >
              <Trash2 className="size-3.5" />
            </button>
            <button
              onClick={() => setOpen(false)}
              className="rounded-lg p-1.5 text-muted transition-colors hover:bg-surface-2 hover:text-ink"
              title={t("assistant.close") ?? "Close"}
              aria-label={t("assistant.close") ?? "Close"}
            >
              <X className="size-4" />
            </button>
          </div>
        </div>

        <AssistantHUD />

        {showSettings ? (
          <div className="flex-1 overflow-y-auto" style={{ maxHeight: "min(400px, 60vh)" }}>
            <AssistantSettings onClose={() => setShowSettings(false)} />
          </div>
        ) : (
          <>
            {error && (
              <div className="flex items-center gap-2 border-b border-critical/30 bg-critical/10 px-4 py-2" role="alert" aria-live="assertive">
                <AlertCircle className="size-3.5 shrink-0 text-critical" />
                <p className="text-xs text-critical truncate">{error}</p>
              </div>
            )}

            <div
              ref={messagesContainerRef}
              onScroll={handleScroll}
              className="flex-1 overflow-y-auto px-4 py-3"
              style={{ maxHeight: "min(400px, 60vh)" }}
              role="log"
              aria-label={t("assistant.panelTitle")}
              aria-live="polite"
            >
              {showSuggestions && (
                <div className="mb-4 flex gap-2">
                  <div className="flex size-7 shrink-0 items-center justify-center rounded-full bg-surface-2 text-muted">
                    <BotMessageSquare className="size-3.5" />
                  </div>
                  <div className="max-w-[85%] rounded-2xl rounded-bl-md bg-surface-2 px-3.5 py-2.5 text-[13px] leading-relaxed text-ink">
                    {t("assistant.welcome")}
                  </div>
                </div>
              )}
              {showSuggestions && <AssistantSuggestions />}
              {messages.map((msg) => (
                <AssistantMessage key={msg.id} message={msg} />
              ))}
              {isLoading && <AssistantTyping />}
              <div ref={messagesEndRef} />
            </div>

            {pendingConfirmation && (
              <div className="border-t border-warn/30 bg-warn/10 px-4 py-3" role="alertdialog" aria-label={t("assistant.confirm")}>
                <p className="mb-2 text-xs leading-relaxed text-warn">{pendingConfirmation.message}</p>
                <div className="flex gap-2">
                  <button
                    onClick={() => void confirmAction()}
                    className="rounded-lg bg-warn/20 px-3 py-1.5 text-xs font-medium text-warn transition-colors hover:bg-warn/30"
                    autoFocus
                  >
                    {t("assistant.confirm")}
                  </button>
                  <button
                    onClick={() => void cancelAction()}
                    className="rounded-lg bg-surface-2 px-3 py-1.5 text-xs font-medium text-muted transition-colors hover:bg-line"
                  >
                    {t("assistant.cancel")}
                  </button>
                </div>
              </div>
            )}

            <AssistantInput ref={inputRef} />
          </>
        )}
      </div>
    </>
  );
}
