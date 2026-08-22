import { useAssistant } from "../../contexts/AssistantContext";
import { useLanguage } from "../../contexts/LanguageContext";
import { Shield, ShieldAlert, Zap, ZapOff } from "lucide-react";

export function AssistantHUD() {
  const { providerInfo, ysmelActive, fenixActive } = useAssistant();
  const { t } = useLanguage();

  const isOllama = providerInfo?.provider === "ollama";
  const isAvailable = providerInfo?.available ?? false;

  return (
    <div className="flex items-center gap-3 overflow-x-auto border-b border-line/50 px-4 py-1.5">
      {/* Provider indicator */}
      <div className="flex min-w-0 items-center gap-1.5" title={isOllama ? `${providerInfo?.model}` : t("assistant.hud.disconnected")}>
        {isAvailable ? (
          <Zap className="size-3 shrink-0 text-good" />
        ) : (
          <ZapOff className="size-3 shrink-0 text-muted/50" />
        )}
        <span className="min-w-0 max-w-[120px] truncate text-[10px] text-muted/70">
          {isOllama ? `Ollama ${providerInfo?.model}` : t("assistant.hud.disconnected")}
        </span>
      </div>

      {/* Separator */}
      <div className="h-3 w-px shrink-0 bg-line" />

      {/* Security indicator */}
      <div className="flex items-center gap-1.5">
        <Shield className="size-3 shrink-0 text-accent" />
        <span className="text-[10px] text-muted/70">{t("assistant.hud.protected")}</span>
      </div>

      {/* Ysmel indicator */}
      {ysmelActive && (
        <>
          <div className="h-3 w-px shrink-0 bg-line" />
          <div className="flex items-center gap-1.5">
            <ShieldAlert className="size-3 shrink-0 text-warn" />
            <span className="text-[10px] font-medium text-warn">{t("assistant.hud.ysmelActive")}</span>
          </div>
        </>
      )}

      {/* Fenix indicator */}
      {fenixActive && (
        <>
          <div className="h-3 w-px shrink-0 bg-line" />
          <div className="flex items-center gap-1.5">
            <ShieldAlert className="size-3 shrink-0 text-critical" />
            <span className="text-[10px] font-medium text-critical">{t("assistant.hud.fenixActive")}</span>
          </div>
        </>
      )}
    </div>
  );
}
