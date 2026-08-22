import { useCallback, useEffect, useRef } from "react";
import { useLanguage } from "../../contexts/LanguageContext";
import { tauri } from "../../lib/tauri";
import type { AssistantMessage as AssistantMessageType } from "../../types/assistant";
import { Bot, User, Volume2 } from "lucide-react";

interface Props {
  message: AssistantMessageType;
}

export function AssistantMessage({ message }: Props) {
  const isUser = message.role === "user";
  const { language } = useLanguage();
  const audioContextRef = useRef<AudioContext | null>(null);

  // Cleanup AudioContext on unmount to prevent browser context leak
  useEffect(() => {
    return () => {
      if (audioContextRef.current) {
        void audioContextRef.current.close();
        audioContextRef.current = null;
      }
    };
  }, []);

  const handleSpeak = useCallback(async () => {
    if (isUser) return;

    // Cancel any ongoing speech
    if ("speechSynthesis" in window) {
      window.speechSynthesis.cancel();
    }
    if (audioContextRef.current) {
      await audioContextRef.current.close().catch(() => undefined);
      audioContextRef.current = null;
    }

    // Try backend Kokoro TTS first
    try {
      const health = await tauri.assistantVoiceHealth();
      if (health.ttsAvailable) {
        const audioBytes = await tauri.assistantSynthesize(message.content);
        const buffer = new Uint8Array(audioBytes).buffer;
        const ctx = new AudioContext();
        audioContextRef.current = ctx;
        const audioBuffer = await ctx.decodeAudioData(buffer);
        const source = ctx.createBufferSource();
        source.buffer = audioBuffer;
        source.connect(ctx.destination);
        source.start(0);
        source.onended = () => {
          void ctx.close();
          audioContextRef.current = null;
        };
        return;
      }
    } catch {
      // Fallback to Web Speech API
    }

    // Fallback: Web Speech API
    if (!("speechSynthesis" in window)) return;
    const utterance = new SpeechSynthesisUtterance(message.content);
    utterance.lang = language === "es" ? "es-ES" : "en-US";
    utterance.rate = 1.0;
    window.speechSynthesis.speak(utterance);
  }, [message.content, language, isUser]);

  return (
    <div className={`mb-3 flex gap-2 ${isUser ? "flex-row-reverse" : ""}`}>
      {/* Avatar */}
      <div
        className={`flex size-7 shrink-0 items-center justify-center rounded-full ${
          isUser ? "bg-accent/15 text-accent" : "bg-surface-2 text-muted"
        }`}
      >
        {isUser ? <User className="size-3.5" /> : <Bot className="size-3.5" />}
      </div>

      {/* Bubble */}
      <div
        className={`max-w-[85%] rounded-2xl px-3.5 py-2.5 text-[13px] leading-relaxed ${
          isUser
            ? "bg-accent/15 text-ink rounded-br-md"
            : "bg-surface-2 text-ink rounded-bl-md"
        }`}
      >
        <div className="whitespace-pre-wrap">{message.content}</div>

        {/* Message actions */}
        <div className="mt-1.5 flex items-center gap-2 border-t border-line pt-1.5">
          {message.intent && message.intent !== "general_conversation" && message.intent !== "unknown" && (
            <span className="text-[10px] uppercase tracking-wider text-muted/60">
              {message.intent.replace(/_/g, " ")}
            </span>
          )}

          {/* Speaker button for assistant messages */}
          {!isUser && (
            <button
              onClick={() => void handleSpeak()}
              className="ml-auto text-muted/40 transition-colors hover:text-accent"
              title="Read aloud"
            >
              <Volume2 className="size-3" />
            </button>
          )}
        </div>
      </div>
    </div>
  );
}
