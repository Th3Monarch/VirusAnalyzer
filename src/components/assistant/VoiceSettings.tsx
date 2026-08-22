import { useCallback, useEffect, useState } from "react";
import { useConfig } from "../../contexts/ConfigContext";
import { useLanguage } from "../../contexts/LanguageContext";
import { tauri } from "../../lib/tauri";
import type { VoiceHealth } from "../../types/voice";
import { Loader2, Mic, Plug, PlugZap, Volume2 } from "lucide-react";

interface VoiceSettingsProps {
  onClose: () => void;
}

export function VoiceSettings({ onClose }: VoiceSettingsProps) {
  const { t, language } = useLanguage();
  const { config, updateConfig } = useConfig();
  const [health, setHealth] = useState<VoiceHealth | null>(null);
  const [checking, setChecking] = useState(false);
  const [saving, setSaving] = useState(false);

  const voicePrefs = config.voice ?? {
    speechRate: 1.0,
    volume: 1.0,
    ttsProvider: "web",
    sttProvider: "web",
    ttsUrl: "http://localhost:8880",
    sttUrl: "http://localhost:8080",
    language,
  };

  // Local form state
  const [enabled, setEnabled] = useState(true);
  const [autoSpeak, setAutoSpeak] = useState(false);
  const [ttsProvider, setTtsProvider] = useState(voicePrefs.ttsProvider);
  const [sttProvider, setSttProvider] = useState(voicePrefs.sttProvider);
  const [ttsUrl, setTtsUrl] = useState(voicePrefs.ttsUrl);
  const [sttUrl, setSttUrl] = useState(voicePrefs.sttUrl);
  const [speechRate, setSpeechRate] = useState(voicePrefs.speechRate);
  const [volume, setVolume] = useState(voicePrefs.volume);
  const [voiceLanguage, setVoiceLanguage] = useState(voicePrefs.language || language);

  // Load voice state from backend
  useEffect(() => {
    void tauri.assistantGetVoiceState().catch(() => undefined);
    void tauri.assistantVoiceHealth().then(setHealth).catch(() => undefined);
  }, []);

  const checkHealth = useCallback(async () => {
    setChecking(true);
    try {
      // Save config first to connect providers
      const backendConfig = {
        enabled,
        autoSpeak,
        language: voiceLanguage,
        sttProvider,
        ttsProvider,
        ttsUrl,
        sttUrl,
        speechRate,
        volume,
      };
      await tauri.assistantUpdateVoiceConfig(backendConfig);
      const h = await tauri.assistantVoiceHealth();
      setHealth(h);
      // Update availability based on health
      setTtsProvider(h.ttsAvailable ? "kokoro" : "web");
      setSttProvider(h.sttAvailable ? "whisper" : "web");
    } catch (e) {
      console.error("Voice health check failed:", e);
    } finally {
      setChecking(false);
    }
  }, [enabled, autoSpeak, voiceLanguage, sttProvider, ttsProvider, ttsUrl, sttUrl, speechRate, volume]);

  const save = useCallback(async () => {
    setSaving(true);
    try {
      const backendConfig = {
        enabled,
        autoSpeak,
        language: voiceLanguage,
        sttProvider,
        ttsProvider,
        ttsUrl,
        sttUrl,
        speechRate,
        volume,
      };
      await tauri.assistantUpdateVoiceConfig(backendConfig);

      // Persist voice preferences to AppConfig
      await updateConfig({
        voice: {
          speechRate,
          volume,
          ttsProvider,
          sttProvider,
          ttsUrl,
          sttUrl,
          language: voiceLanguage,
        },
      });

      onClose();
    } catch (e) {
      console.error("Failed to save voice config:", e);
    } finally {
      setSaving(false);
    }
  }, [enabled, autoSpeak, voiceLanguage, sttProvider, ttsProvider, ttsUrl, sttUrl, speechRate, volume, updateConfig, onClose]);

  return (
    <div className="flex flex-col gap-4">
      <h3 className="text-sm font-semibold text-ink">{t("assistant.voiceSettings")}</h3>

      {/* Enable voice */}
      <label className="flex items-center gap-3">
        <input
          type="checkbox"
          checked={enabled}
          onChange={(e) => setEnabled(e.target.checked)}
          className="size-4 rounded border-line accent-accent"
        />
        <Mic className="size-4 text-muted" />
        <span className="text-sm text-ink">{t("assistant.voiceEnable")}</span>
      </label>

      {enabled && (
        <>
          {/* Auto-speak responses */}
          <label className="flex items-center gap-3">
            <input
              type="checkbox"
              checked={autoSpeak}
              onChange={(e) => setAutoSpeak(e.target.checked)}
              className="size-4 rounded border-line accent-accent"
            />
            <Volume2 className="size-4 text-muted" />
            <span className="text-sm text-ink">{t("assistant.voiceAutoSpeak")}</span>
          </label>

          {/* Language */}
          <div className="flex flex-col gap-1.5">
            <label className="text-xs font-medium text-muted">{t("assistant.voiceLanguage")}</label>
            <select
              value={voiceLanguage}
              onChange={(e) => setVoiceLanguage(e.target.value)}
              className="rounded-lg border border-line bg-surface-2 px-3 py-2 text-sm text-ink focus:border-accent focus:outline-none"
            >
              <option value="es">Español</option>
              <option value="en">English</option>
            </select>
          </div>

          {/* TTS Provider */}
          <div className="flex flex-col gap-1.5">
            <label className="text-xs font-medium text-muted">TTS Provider</label>
            <select
              value={ttsProvider}
              onChange={(e) => setTtsProvider(e.target.value)}
              className="rounded-lg border border-line bg-surface-2 px-3 py-2 text-sm text-ink focus:border-accent focus:outline-none"
            >
              <option value="web">{t("assistant.ttsWeb")}</option>
              <option value="kokoro">Kokoro (local server)</option>
            </select>
          </div>

          {/* Kokoro URL */}
          {ttsProvider === "kokoro" && (
            <div className="flex flex-col gap-1.5">
              <label className="text-xs font-medium text-muted">Kokoro URL</label>
              <input
                type="text"
                value={ttsUrl}
                onChange={(e) => setTtsUrl(e.target.value)}
                placeholder="http://localhost:8880"
                className="rounded-lg border border-line bg-surface-2 px-3 py-2 text-sm text-ink placeholder:text-muted/50 focus:border-accent focus:outline-none"
              />
            </div>
          )}

          {/* STT Provider */}
          <div className="flex flex-col gap-1.5">
            <label className="text-xs font-medium text-muted">STT Provider</label>
            <select
              value={sttProvider}
              onChange={(e) => setSttProvider(e.target.value)}
              className="rounded-lg border border-line bg-surface-2 px-3 py-2 text-sm text-ink focus:border-accent focus:outline-none"
            >
              <option value="web">{t("assistant.sttWeb")}</option>
              <option value="whisper">Whisper (local server)</option>
            </select>
          </div>

          {/* Whisper URL */}
          {sttProvider === "whisper" && (
            <div className="flex flex-col gap-1.5">
              <label className="text-xs font-medium text-muted">Whisper URL</label>
              <input
                type="text"
                value={sttUrl}
                onChange={(e) => setSttUrl(e.target.value)}
                placeholder="http://localhost:8080"
                className="rounded-lg border border-line bg-surface-2 px-3 py-2 text-sm text-ink placeholder:text-muted/50 focus:border-accent focus:outline-none"
              />
            </div>
          )}

          {/* Speech Rate */}
          <div className="flex flex-col gap-1.5">
            <label className="text-xs font-medium text-muted">
              {t("assistant.speechRate")} — {speechRate.toFixed(1)}x
            </label>
            <input
              type="range"
              min={0.5}
              max={2.0}
              step={0.1}
              value={speechRate}
              onChange={(e) => setSpeechRate(parseFloat(e.target.value))}
              className="w-full accent-accent"
            />
            <div className="flex justify-between text-[10px] text-muted/60">
              <span>0.5x</span>
              <span>1.0x</span>
              <span>2.0x</span>
            </div>
          </div>

          {/* Volume */}
          <div className="flex flex-col gap-1.5">
            <label className="text-xs font-medium text-muted">
              {t("assistant.volume")} — {Math.round(volume * 100)}%
            </label>
            <input
              type="range"
              min={0}
              max={1}
              step={0.05}
              value={volume}
              onChange={(e) => setVolume(parseFloat(e.target.value))}
              className="w-full accent-accent"
            />
            <div className="flex justify-between text-[10px] text-muted/60">
              <span>0%</span>
              <span>50%</span>
              <span>100%</span>
            </div>
          </div>

          {/* Health check button */}
          <button
            type="button"
            onClick={() => void checkHealth()}
            disabled={checking}
            className="inline-flex items-center gap-2 rounded-lg border border-line bg-surface-2 px-3 py-2 text-sm font-medium text-ink transition-colors hover:border-muted/50 disabled:opacity-50"
          >
            {checking ? (
              <Loader2 className="size-4 animate-spin" />
            ) : health?.ttsAvailable || health?.sttAvailable ? (
              <PlugZap className="size-4 text-good" />
            ) : (
              <Plug className="size-4" />
            )}
            {t("assistant.testConnection")}
          </button>

          {/* Health status */}
          {health && (
            <div className="flex flex-col gap-1">
              <div
                className={`rounded-lg border px-3 py-1.5 text-xs ${
                  health.ttsAvailable
                    ? "border-good/30 bg-good/10 text-good"
                    : "border-line bg-surface-2 text-muted"
                }`}
              >
                TTS (Kokoro): {health.ttsAvailable ? "Connected" : "Not available"}
              </div>
              <div
                className={`rounded-lg border px-3 py-1.5 text-xs ${
                  health.sttAvailable
                    ? "border-good/30 bg-good/10 text-good"
                    : "border-line bg-surface-2 text-muted"
                }`}
              >
                STT (Whisper): {health.sttAvailable ? "Connected" : "Not available"}
              </div>
            </div>
          )}

          {/* Info */}
          <p className="text-[11px] leading-relaxed text-muted/70">
            {t("assistant.voiceInfo")}
          </p>
        </>
      )}

      {/* Save button */}
      <button
        type="button"
        onClick={save}
        disabled={saving}
        className="mt-2 inline-flex items-center justify-center gap-2 rounded-lg bg-accent px-3 py-2 text-sm font-medium text-white transition-colors hover:bg-accent/90 disabled:opacity-50"
      >
        {saving && <Loader2 className="size-4 animate-spin" />}
        {t("common.save")}
      </button>
    </div>
  );
}
