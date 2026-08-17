import { useEffect, useState, type FormEvent } from "react";
import {
  FolderOpen,
  KeyRound,
  Languages,
  LayoutPanelLeft,
  Loader2,
  Moon,
  MousePointer2,
  Palette,
  Sun,
  Trash2,
} from "lucide-react";
import { open } from "@tauri-apps/plugin-dialog";
import { useLanguage } from "../contexts/LanguageContext";
import { usePlatform } from "../contexts/PlatformContext";
import { useTheme } from "../contexts/ThemeContext";
import { useConfig } from "../contexts/ConfigContext";
import { useToast } from "../contexts/ToastContext";
import { PageHeader } from "../components/ui/PageHeader";
import { Card, CardHeader } from "../components/ui/Card";
import { Button } from "../components/ui/Button";
import { tauri } from "../lib/tauri";

export function Settings() {
  const { t, language, setLanguage } = useLanguage();
  const { theme, setTheme } = useTheme();
  const { config, updateConfig } = useConfig();
  const { toast } = useToast();
  const { isWindows } = usePlatform();

  const [vtKey, setVtKey] = useState(config.virustotalApiKey ?? "");
  const [notice, setNotice] = useState<{ kind: "ok" | "err"; text: string } | null>(null);
  const [dirNotice, setDirNotice] = useState<{ kind: "ok" | "err"; text: string } | null>(null);

  // Menú contextual
  const [cmInstalled, setCmInstalled] = useState<boolean | null>(null);
  const [cmBusy, setCmBusy] = useState(false);

  useEffect(() => {
    let alive = true;
    void tauri
      .isContextMenuInstalled()
      .then((installed) => {
        if (alive) setCmInstalled(installed);
      })
      .catch(() => {
        if (alive) setCmInstalled(false);
      });
    return () => {
      alive = false;
    };
  }, []);

  const toggleContextMenu = async () => {
    setCmBusy(true);
    try {
      if (cmInstalled) {
        await tauri.uninstallContextMenu();
        await updateConfig({ contextMenuEnabled: false });
        setCmInstalled(false);
        toast(t("settings.contextMenuRemoved"), "success");
      } else {
        await tauri.installContextMenu(t("contextMenu.label"));
        await updateConfig({ contextMenuEnabled: true });
        setCmInstalled(true);
        toast(t("settings.contextMenuInstalledToast"), "success");
      }
    } catch (e) {
      toast(e instanceof Error ? e.message : String(e), "error");
    } finally {
      setCmBusy(false);
    }
  };

  const chooseQuarantineDir = async () => {
    try {
      const selected = await open({ directory: true, title: t("settings.storageQuarantine") });
      if (typeof selected === "string") {
        await updateConfig({ storage: { ...config.storage, quarantineDir: selected } });
        setDirNotice({ kind: "ok", text: t("settings.storageDirSaved") });
      }
    } catch (e) {
      setDirNotice({ kind: "err", text: e instanceof Error ? e.message : String(e) });
    }
  };

  const resetQuarantineDir = async () => {
    try {
      await updateConfig({ storage: { ...config.storage, quarantineDir: null } });
      setDirNotice({ kind: "ok", text: t("settings.storageDirReset") });
    } catch (e) {
      setDirNotice({ kind: "err", text: e instanceof Error ? e.message : String(e) });
    }
  };

  const onSaveVtKey = async (e: FormEvent) => {
    e.preventDefault();
    const value = vtKey.trim() || null;
    try {
      await updateConfig({ virustotalApiKey: value });
      setNotice({ kind: "ok", text: t("settings.vtKeySaved") });
    } catch {
      setNotice({ kind: "err", text: t("settings.vtKeyError") });
    }
  };

  const onClearVtKey = async () => {
    setVtKey("");
    try {
      await updateConfig({ virustotalApiKey: null });
      setNotice({ kind: "ok", text: t("settings.vtKeySaved") });
    } catch {
      setNotice({ kind: "err", text: t("settings.vtKeyError") });
    }
  };

  const themeOptions = [
    { value: "dark", label: t("settings.themeDark"), icon: Moon },
    { value: "light", label: t("settings.themeLight"), icon: Sun },
    { value: "system", label: t("settings.themeSystem"), icon: Palette },
  ] as const;

  return (
    <div>
      <PageHeader title={t("settings.title")} subtitle={t("settings.subtitle")} />

      <div className="max-w-3xl space-y-6">
        {/* Idioma */}
        <Card>
          <CardHeader title={t("settings.language")} action={<Languages className="size-4 text-muted" />} />
          <div className="flex flex-wrap gap-2 px-5 py-4">
            <Button
              variant={language === "es" ? "primary" : "secondary"}
              onClick={() => setLanguage("es")}
            >
              {t("settings.languageEs")}
            </Button>
            <Button
              variant={language === "en" ? "primary" : "secondary"}
              onClick={() => setLanguage("en")}
            >
              {t("settings.languageEn")}
            </Button>
          </div>
        </Card>

        {/* Tema */}
        <Card>
          <CardHeader title={t("settings.theme")} action={<LayoutPanelLeft className="size-4 text-muted" />} />
          <div className="flex flex-wrap gap-2 px-5 py-4">
            {themeOptions.map((option) => (
              <Button
                key={option.value}
                variant={theme === option.value ? "primary" : "secondary"}
                onClick={() => setTheme(option.value)}
              >
                <option.icon className="size-4" />
                {option.label}
              </Button>
            ))}
          </div>
        </Card>

        {/* VirusTotal */}
        <Card>
          <CardHeader title={t("settings.vtKey")} action={<KeyRound className="size-4 text-muted" />} />
          <div className="space-y-4 px-5 py-4">
            <div>
              <div className="flex items-center justify-between gap-4">
                <div>
                  <p className="text-sm font-medium text-ink">{t("settings.vtEnabled")}</p>
                  <p className="mt-0.5 text-xs text-muted">{t("settings.vtEnabledHelp")}</p>
                </div>
                <button
                  type="button"
                  onClick={() => void updateConfig({ virustotalEnabled: !config.virustotalEnabled })}
                  aria-pressed={config.virustotalEnabled}
                  className={`relative h-6 w-11 shrink-0 rounded-full transition-colors ${config.virustotalEnabled ? "bg-accent" : "bg-muted/30"}`}
                >
                  <span
                    className={`absolute top-0.5 size-5 rounded-full bg-white shadow transition-all ${config.virustotalEnabled ? "left-[22px]" : "left-0.5"}`}
                  />
                </button>
              </div>
              <p className="mt-2 rounded-lg border border-line bg-surface-2/50 px-3 py-2 text-xs leading-relaxed text-muted">
                {t("settings.vtConsent")}
              </p>
            </div>

            <form onSubmit={onSaveVtKey} className="space-y-3 border-t border-line pt-4">
              <input
                type="password"
                value={vtKey}
                onChange={(e) => setVtKey(e.currentTarget.value)}
                placeholder={config.virustotalApiKey ? "••••••••••••••••" : t("settings.vtKey")}
                autoComplete="off"
                className="w-full rounded-lg border border-line bg-surface-2 px-3 py-2 font-mono text-sm text-ink placeholder:text-muted focus:border-accent focus:outline-none"
              />
              <p className="text-xs leading-relaxed text-muted">{t("settings.vtKeyHelp")}</p>
              {notice ? (
                <p className={`text-xs font-medium ${notice.kind === "ok" ? "text-good" : "text-critical"}`}>
                  {notice.text}
                </p>
              ) : null}
              <div className="flex gap-2">
                <Button type="submit">{t("settings.vtKeySave")}</Button>
                <Button variant="secondary" type="button" onClick={onClearVtKey}>
                  <Trash2 className="size-4" />
                  {t("settings.vtKeyClear")}
                </Button>
              </div>
            </form>
          </div>
        </Card>

        {/* Menú contextual (solo Windows) */}
        {isWindows ? (
          <Card>
            <CardHeader
              title={t("settings.contextMenu")}
              subtitle={t("settings.contextMenuHelp")}
              action={<MousePointer2 className="size-4 text-muted" />}
            />
            <div className="px-5 py-4">
              <div className="flex flex-wrap items-center justify-between gap-4">
                <div className="min-w-0">
                  <p className="text-sm font-medium text-ink">
                    {cmInstalled === null
                      ? t("settings.contextMenuChecking")
                      : cmInstalled
                        ? t("settings.contextMenuInstalled")
                        : t("settings.contextMenuNotInstalled")}
                  </p>
                  <p className="mt-0.5 max-w-md text-xs leading-relaxed text-muted">
                    {t("settings.contextMenuDesc")}
                  </p>
                </div>
                <Button
                  variant={cmInstalled ? "secondary" : "primary"}
                  onClick={() => void toggleContextMenu()}
                  disabled={cmInstalled === null || cmBusy}
                >
                  {cmBusy ? <Loader2 className="size-4 animate-spin" /> : null}
                  {cmInstalled ? t("settings.contextMenuDisable") : t("settings.contextMenuEnable")}
                </Button>
              </div>
            </div>
          </Card>
        ) : null}

        {/* Almacenamiento */}
        <Card>
          <CardHeader title={t("settings.storage")} />
          <div className="space-y-4 px-5 py-4">
            <div>
              <label className="mb-1 block text-sm text-muted">{t("settings.storageQuarantine")}</label>
              <div className="flex items-center gap-2">
                <input
                  readOnly
                  value={config.storage.quarantineDir ?? t("settings.storageDefaultDir")}
                  className="w-full rounded-lg border border-line bg-surface-2 px-3 py-2 font-mono text-sm text-ink"
                />
                <Button variant="secondary" onClick={() => void chooseQuarantineDir()}>
                  <FolderOpen className="size-4" />
                  {t("common.change")}
                </Button>
                {config.storage.quarantineDir ? (
                  <Button variant="ghost" onClick={() => void resetQuarantineDir()}>
                    {t("common.reset")}
                  </Button>
                ) : null}
              </div>
              <p className="mt-1.5 text-xs leading-relaxed text-muted">{t("settings.storageQuarantineHelp")}</p>
              {dirNotice ? (
                <p className={`mt-1 text-xs font-medium ${dirNotice.kind === "ok" ? "text-good" : "text-critical"}`}>
                  {dirNotice.text}
                </p>
              ) : null}
            </div>
            <div>
              <label className="mb-1 block text-sm text-muted">{t("settings.storageHistory")}</label>
              <input
                type="number"
                readOnly
                disabled
                value={config.storage.keepHistoryDays}
                className="w-32 cursor-not-allowed rounded-lg border border-line bg-surface-2 px-3 py-2 text-sm text-muted"
              />
              <p className="mt-1 text-xs leading-relaxed text-muted">{t("settings.storageHistoryPending")}</p>
            </div>
          </div>
        </Card>
      </div>
    </div>
  );
}
