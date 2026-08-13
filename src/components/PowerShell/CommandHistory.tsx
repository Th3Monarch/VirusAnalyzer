import { History, Play, Star, StarOff, Trash2 } from "lucide-react";
import { useLanguage } from "../../contexts/LanguageContext";
import { Button } from "../ui/Button";

interface CommandHistoryProps {
  history: string[];
  favorites: string[];
  onReuse: (command: string) => void;
  onToggleFavorite: (command: string) => void;
  onDelete: (command: string) => void;
  onClear: () => void;
}

export function CommandHistory({
  history,
  favorites,
  onReuse,
  onToggleFavorite,
  onDelete,
  onClear,
}: CommandHistoryProps) {
  const { t } = useLanguage();

  const renderItem = (command: string, favorite: boolean, deletable: boolean) => (
    <li key={command} className="flex items-center gap-1 rounded-lg px-2 py-1.5 transition-colors hover:bg-surface-2">
      <button
        type="button"
        onClick={() => onReuse(command)}
        title={command}
        className="min-w-0 flex-1 truncate text-left font-mono text-xs text-ink transition-colors hover:text-accent focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-accent"
      >
        {command}
      </button>
      <button
        type="button"
        onClick={() => onToggleFavorite(command)}
        aria-label={favorite ? t("powershell.removeFavorite") : t("powershell.addFavorite")}
        title={favorite ? t("powershell.removeFavorite") : t("powershell.addFavorite")}
        className={`shrink-0 rounded p-1 transition-colors ${
          favorite ? "text-warn hover:text-warn/80" : "text-muted hover:text-ink"
        } focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-accent`}
      >
        {favorite ? <Star className="size-3.5 fill-current" /> : <StarOff className="size-3.5" />}
      </button>
      {deletable ? (
        <button
          type="button"
          onClick={() => onDelete(command)}
          aria-label={t("powershell.historyDelete")}
          title={t("powershell.historyDelete")}
          className="shrink-0 rounded p-1 text-muted transition-colors hover:text-critical focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-accent"
        >
          <Trash2 className="size-3.5" />
        </button>
      ) : null}
    </li>
  );

  return (
    <div className="space-y-6">
      <div>
        <p className="mb-2 flex items-center gap-1.5 text-xs font-semibold uppercase tracking-wider text-muted">
          <Star className="size-3.5 text-warn" />
          {t("powershell.favorites")}
        </p>
        {favorites.length === 0 ? (
          <p className="rounded-lg border border-dashed border-line px-3 py-4 text-center text-xs text-muted">
            {t("powershell.favoritesEmpty")}
          </p>
        ) : (
          <ul className="space-y-0.5">{favorites.map((cmd) => renderItem(cmd, true, false))}</ul>
        )}
      </div>

      <div>
        <div className="mb-2 flex items-center justify-between gap-2">
          <p className="flex items-center gap-1.5 text-xs font-semibold uppercase tracking-wider text-muted">
            <History className="size-3.5" />
            {t("powershell.history")}
          </p>
          {history.length > 0 ? (
            <Button variant="ghost" onClick={onClear} className="px-2 py-0.5 text-[11px]">
              {t("powershell.historyClear")}
            </Button>
          ) : null}
        </div>
        {history.length === 0 ? (
          <p className="rounded-lg border border-dashed border-line px-3 py-4 text-center text-xs text-muted">
            {t("powershell.historyEmpty")}
          </p>
        ) : (
          <ul className="space-y-0.5">
            {history.map((cmd) => (
              <li key={`${cmd}-${history.indexOf(cmd)}`} className="group">
                <div className="flex items-center gap-1 rounded-lg px-2 py-1.5 transition-colors hover:bg-surface-2">
                  <button
                    type="button"
                    onClick={() => onReuse(cmd)}
                    title={t("powershell.run")}
                    className="shrink-0 rounded p-1 text-muted opacity-0 transition-opacity group-hover:opacity-100 focus-visible:opacity-100 focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-accent"
                  >
                    <Play className="size-3.5" />
                  </button>
                  <button
                    type="button"
                    onClick={() => onReuse(cmd)}
                    title={cmd}
                    className="min-w-0 flex-1 truncate text-left font-mono text-xs text-ink transition-colors hover:text-accent focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-accent"
                  >
                    {cmd}
                  </button>
                  <button
                    type="button"
                    onClick={() => onToggleFavorite(cmd)}
                    aria-label={t("powershell.addFavorite")}
                    title={t("powershell.addFavorite")}
                    className="shrink-0 rounded p-1 text-muted transition-colors hover:text-warn focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-accent"
                  >
                    <StarOff className="size-3.5" />
                  </button>
                  <button
                    type="button"
                    onClick={() => onDelete(cmd)}
                    aria-label={t("powershell.historyDelete")}
                    title={t("powershell.historyDelete")}
                    className="shrink-0 rounded p-1 text-muted transition-colors hover:text-critical focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-accent"
                  >
                    <Trash2 className="size-3.5" />
                  </button>
                </div>
              </li>
            ))}
          </ul>
        )}
      </div>
    </div>
  );
}
