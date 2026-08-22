import { useCallback, useEffect, useState } from "react";
import { useAssistant } from "../../contexts/AssistantContext";
import { useLanguage } from "../../contexts/LanguageContext";
import { tauri } from "../../lib/tauri";
import type { OllamaTestResult } from "../../types/assistant";
import { Loader2, Plug, PlugZap, RefreshCw, X } from "lucide-react";

interface OllamaSettingsProps {
  onClose: () => void;
}

export function OllamaSettings({ onClose }: OllamaSettingsProps) {
  const { t } = useLanguage();
  const { refreshProviderInfo } = useAssistant();
  const [url, setUrl] = useState("http://localhost:11434");
  const [model, setModel] = useState("llama3.2");
  const [enabled, setEnabled] = useState(false);
  const [temperature, setTemperature] = useState(0.3);
  const [maxTokens, setMaxTokens] = useState(1024);
  const [testResult, setTestResult] = useState<OllamaTestResult | null>(null);
  const [testing, setTesting] = useState(false);
  const [saving, setSaving] = useState(false);

  // Load current config
  useEffect(() => {
    void tauri.getConfig().then((config) => {
      if (config.ollama) {
        setUrl(config.ollama.url);
        setModel(config.ollama.model);
        setEnabled(config.ollama.enabled);
        setTemperature(config.ollama.temperature ?? 0.3);
        setMaxTokens(config.ollama.maxTokens ?? 1024);
      }
    }).catch(() => undefined);
  }, []);

  const testConnection = useCallback(async () => {
    setTesting(true);
    setTestResult(null);
    try {
      const result = await tauri.assistantTestOllama(url);
      setTestResult(result);
      if (result.connected && result.models.length > 0 && !result.models.includes(model)) {
        setModel(result.models[0]);
      }
    } catch (e) {
      setTestResult({ connected: false, models: [], error: String(e) });
    } finally {
      setTesting(false);
    }
  }, [url, model]);

  const save = useCallback(async () => {
    setSaving(true);
    try {
      // Save to config
      const config = await tauri.getConfig();
      config.ollama = { url, model, enabled, temperature, maxTokens };
      await tauri.saveConfig(config);

      // Update provider in backend
      await tauri.assistantUpdateOllama(url, model, enabled, temperature, maxTokens);
      // Refresh provider info so HUD updates
      await refreshProviderInfo();
      onClose();
    } catch (e) {
      console.error("Failed to save Ollama config:", e);
    } finally {
      setSaving(false);
    }
  }, [url, model, enabled, temperature, maxTokens, onClose]);

  return (
    <div className="flex flex-col gap-4">
      {/* Header */}
      <div className="flex items-center justify-between">
        <h3 className="text-sm font-semibold text-ink">Ollama</h3>
        <button
          type="button"
          onClick={onClose}
          className="rounded p-1 text-muted hover:bg-surface-2 hover:text-ink"
        >
          <X className="size-4" />
        </button>
      </div>

      {/* Toggle */}
      <label className="flex items-center gap-3">
        <input
          type="checkbox"
          checked={enabled}
          onChange={(e) => setEnabled(e.target.checked)}
          className="size-4 rounded border-line accent-accent"
        />
        <span className="text-sm text-ink">{t("assistant.ollamaMode")}</span>
      </label>

      {enabled && (
        <>
          {/* URL */}
          <div className="flex flex-col gap-1.5">
            <label className="text-xs font-medium text-muted">URL</label>
            <input
              type="text"
              value={url}
              onChange={(e) => setUrl(e.target.value)}
              placeholder="http://localhost:11434"
              className="rounded-lg border border-line bg-surface-2 px-3 py-2 text-sm text-ink placeholder:text-muted/50 focus:border-accent focus:outline-none"
            />
          </div>

          {/* Model */}
          <div className="flex flex-col gap-1.5">
            <label className="text-xs font-medium text-muted">{t("assistant.model")}</label>
            {testResult?.connected && testResult.models.length > 0 ? (
              <select
                value={model}
                onChange={(e) => setModel(e.target.value)}
                className="rounded-lg border border-line bg-surface-2 px-3 py-2 text-sm text-ink focus:border-accent focus:outline-none"
              >
                {testResult.models.map((m) => (
                  <option key={m} value={m}>{m}</option>
                ))}
              </select>
            ) : (
              <input
                type="text"
                value={model}
                onChange={(e) => setModel(e.target.value)}
                placeholder="llama3.2"
                className="rounded-lg border border-line bg-surface-2 px-3 py-2 text-sm text-ink placeholder:text-muted/50 focus:border-accent focus:outline-none"
              />
            )}
          </div>

          {/* Temperature */}
          <div className="flex flex-col gap-1.5">
            <label className="text-xs font-medium text-muted">
              {t("assistant.temperature")} — {temperature.toFixed(2)}
            </label>
            <input
              type="range"
              min={0}
              max={1}
              step={0.05}
              value={temperature}
              onChange={(e) => setTemperature(parseFloat(e.target.value))}
              className="w-full accent-accent"
            />
            <div className="flex justify-between text-[10px] text-muted/60">
              <span>{t("assistant.temperaturePrecise")}</span>
              <span>{t("assistant.temperatureCreative")}</span>
            </div>
          </div>

          {/* Max tokens */}
          <div className="flex flex-col gap-1.5">
            <label className="text-xs font-medium text-muted">
              {t("assistant.maxTokens")} — {maxTokens}
            </label>
            <input
              type="range"
              min={256}
              max={4096}
              step={256}
              value={maxTokens}
              onChange={(e) => setMaxTokens(parseInt(e.target.value, 10))}
              className="w-full accent-accent"
            />
            <div className="flex justify-between text-[10px] text-muted/60">
              <span>256</span>
              <span>4096</span>
            </div>
          </div>

          {/* Test button */}
          <button
            type="button"
            onClick={testConnection}
            disabled={testing}
            className="inline-flex items-center gap-2 rounded-lg border border-line bg-surface-2 px-3 py-2 text-sm font-medium text-ink transition-colors hover:border-muted/50 disabled:opacity-50"
          >
            {testing ? (
              <Loader2 className="size-4 animate-spin" />
            ) : testResult?.connected ? (
              <PlugZap className="size-4 text-good" />
            ) : (
              <Plug className="size-4" />
            )}
            {t("assistant.testConnection")}
          </button>

          {/* Test result */}
          {testResult && (
            <div
              className={`rounded-lg border px-3 py-2 text-xs ${
                testResult.connected
                  ? "border-good/30 bg-good/10 text-good"
                  : "border-critical/30 bg-critical/10 text-critical"
              }`}
            >
              {testResult.connected ? (
                <span>{t("assistant.connected")} — {testResult.models.length} {t("assistant.modelsAvailable")}</span>
              ) : (
                <span>{testResult.error || t("assistant.connectionFailed")}</span>
              )}
            </div>
          )}

          {/* Refresh models */}
          {testResult?.connected && (
            <button
              type="button"
              onClick={testConnection}
              disabled={testing}
              className="inline-flex items-center gap-1.5 text-xs text-muted hover:text-ink"
            >
              <RefreshCw className="size-3" />
              {t("assistant.refreshModels")}
            </button>
          )}
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
