import { useState } from "react";
import { useLanguage } from "../../contexts/LanguageContext";
import { OllamaSettings } from "./OllamaSettings";
import { VoiceSettings } from "./VoiceSettings";
import { Cpu, Mic } from "lucide-react";

type SettingsTab = "ai" | "voice";

interface AssistantSettingsProps {
  onClose: () => void;
}

export function AssistantSettings({ onClose }: AssistantSettingsProps) {
  const { t } = useLanguage();
  const [tab, setTab] = useState<SettingsTab>("ai");

  return (
    <div className="flex flex-col" style={{ minHeight: 300 }}>
      {/* Tab bar */}
      <div className="flex border-b border-line">
        <button
          onClick={() => setTab("ai")}
          className={`flex-1 flex items-center justify-center gap-1.5 px-3 py-2 text-xs font-medium transition-colors ${
            tab === "ai"
              ? "border-b-2 border-accent text-accent"
              : "text-muted hover:text-ink"
          }`}
        >
          <Cpu className="size-3" />
          AI
        </button>
        <button
          onClick={() => setTab("voice")}
          className={`flex-1 flex items-center justify-center gap-1.5 px-3 py-2 text-xs font-medium transition-colors ${
            tab === "voice"
              ? "border-b-2 border-accent text-accent"
              : "text-muted hover:text-ink"
          }`}
        >
          <Mic className="size-3" />
          {t("assistant.voiceSettings")}
        </button>
      </div>

      {/* Tab content */}
      <div className="flex-1 overflow-y-auto px-4 py-3">
        {tab === "ai" ? (
          <OllamaSettings onClose={onClose} />
        ) : (
          <VoiceSettings onClose={onClose} />
        )}
      </div>
    </div>
  );
}
