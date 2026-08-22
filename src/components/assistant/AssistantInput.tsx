import { forwardRef, useCallback, useEffect, useState, type KeyboardEvent } from "react";
import { useAssistant } from "../../contexts/AssistantContext";
import { useConfig } from "../../contexts/ConfigContext";
import { useLanguage } from "../../contexts/LanguageContext";
import { useVoice } from "../../hooks/useVoice";
import { Send, Mic, MicOff, Volume2, VolumeX } from "lucide-react";

export const AssistantInput = forwardRef<HTMLTextAreaElement>(function AssistantInput(_, ref) {
  const { sendMessage, isLoading } = useAssistant();
  const { t, language } = useLanguage();
  const { config } = useConfig();
  const [value, setValue] = useState("");
  const speechRate = config.voice?.speechRate ?? 1.0;
  const volume = config.voice?.volume ?? 1.0;
  const {
    state: voiceState,
    startListening,
    stopListening,
    available: voiceAvailable,
    transcript,
    speak,
    stopSpeaking,
    speaking,
  } = useVoice(language, speechRate, volume);

  // Auto-fill from voice transcript
  useEffect(() => {
    if (transcript) {
      setValue((prev) => (prev ? `${prev} ${transcript}` : transcript));
    }
  }, [transcript]);

  const handleSend = useCallback(() => {
    const trimmed = value.trim();
    if (!trimmed || isLoading) return;
    void sendMessage(trimmed);
    setValue("");
  }, [value, isLoading, sendMessage]);

  const handleKeyDown = useCallback(
    (e: KeyboardEvent<HTMLTextAreaElement>) => {
      if (e.key === "Enter" && !e.shiftKey) {
        e.preventDefault();
        handleSend();
      }
    },
    [handleSend],
  );

  const toggleVoice = useCallback(() => {
    if (voiceState === "listening") {
      stopListening();
    } else {
      startListening();
    }
  }, [voiceState, startListening, stopListening]);

  const isListening = voiceState === "listening";

  return (
    <div className="border-t border-line px-4 py-3">
      <div className="flex items-end gap-2 rounded-xl border border-line bg-surface-2 px-3 py-2">
        <textarea
          ref={ref}
          value={value}
          onChange={(e) => setValue(e.target.value)}
          onKeyDown={handleKeyDown}
          placeholder={
            isListening
              ? (t("assistant.voiceListening") ?? "Listening...")
              : t("assistant.placeholder")
          }
          rows={1}
          className="max-h-24 min-h-[36px] flex-1 resize-none bg-transparent text-[13px] text-ink placeholder-muted/50 outline-none"
          style={{ fieldSizing: "content" } as React.CSSProperties}
        />

        {/* Voice buttons */}
        {voiceAvailable && (
          <>
            <button
              onClick={toggleVoice}
              disabled={isLoading}
              className={`flex size-7 shrink-0 items-center justify-center rounded-lg transition-colors ${
                isListening
                  ? "bg-critical/20 text-critical animate-pulse"
                  : "text-muted hover:bg-surface hover:text-ink"
              }`}
              title={isListening ? (t("assistant.voiceStop") ?? "Stop listening") : (t("assistant.voiceStart") ?? "Start listening")}
            >
              {isListening ? <MicOff className="size-3.5" /> : <Mic className="size-3.5" />}
            </button>

            <button
              onClick={() => (speaking ? stopSpeaking() : value && speak(value))}
              disabled={!value.trim() && !speaking}
              className={`flex size-7 shrink-0 items-center justify-center rounded-lg transition-colors ${
                speaking
                  ? "bg-accent/20 text-accent"
                  : "text-muted hover:bg-surface hover:text-ink disabled:opacity-40"
              }`}
              title={speaking ? (t("assistant.voiceMute") ?? "Stop speaking") : (t("assistant.voiceSpeak") ?? "Speak aloud")}
            >
              {speaking ? <VolumeX className="size-3.5" /> : <Volume2 className="size-3.5" />}
            </button>
          </>
        )}

        <button
          onClick={handleSend}
          disabled={!value.trim() || isLoading}
          className="flex size-7 shrink-0 items-center justify-center rounded-lg bg-accent text-white transition-colors hover:bg-accent/80 disabled:opacity-40"
          title={t("assistant.send")}
        >
          <Send className="size-3.5" />
        </button>
      </div>

      {/* Voice status indicator */}
      {isListening && (
        <div className="mt-1.5 flex items-center justify-center gap-1.5">
          <span className="size-1.5 rounded-full bg-critical animate-pulse" />
          <span className="text-[10px] text-critical">{t("assistant.voiceListening")}</span>
        </div>
      )}

      {voiceState === "error" && (
        <p className="mt-1.5 text-center text-[10px] text-critical/70">
          {t("assistant.voiceError")}
        </p>
      )}

      {!isListening && voiceState !== "error" && (
        <p className="mt-1.5 text-center text-[10px] text-muted/50">
          {t("assistant.toggleHint")} · Enter
          {voiceAvailable ? " · 🎤" : ""}
        </p>
      )}
    </div>
  );
});
