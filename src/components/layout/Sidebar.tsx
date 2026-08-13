import { NavLink } from "react-router-dom";
import {
  Archive,
  BookOpen,
  FileSearch,
  History,
  LayoutDashboard,
  MonitorCog,
  Radar,
  ScrollText,
  Settings,
  ShieldCheck,
  SquareTerminal,
  type LucideIcon,
} from "lucide-react";
import { useLanguage } from "../../contexts/LanguageContext";
import type { TranslationKey } from "../../lib/i18n";

interface NavItem {
  to: string;
  label: TranslationKey;
  icon: LucideIcon;
  end?: boolean;
}

interface NavSection {
  group: TranslationKey;
  items: NavItem[];
}

const NAV_SECTIONS: NavSection[] = [
  {
    group: "nav.analysis",
    items: [
      { to: "/", label: "nav.dashboard", icon: LayoutDashboard, end: true },
      { to: "/scan", label: "nav.scan", icon: Radar },
      { to: "/results", label: "nav.results", icon: History },
      { to: "/analysis", label: "nav.analysisDetail", icon: FileSearch },
    ],
  },
  {
    group: "nav.management",
    items: [{ to: "/quarantine", label: "nav.quarantine", icon: Archive }],
  },
  {
    group: "nav.intelligence",
    items: [{ to: "/rules", label: "nav.rules", icon: ScrollText }],
  },
  {
    group: "nav.tools",
    items: [
      { to: "/system", label: "nav.system", icon: MonitorCog },
      { to: "/powershell", label: "nav.powershell", icon: SquareTerminal },
      { to: "/ps-reference", label: "nav.psReference", icon: BookOpen },
    ],
  },
  {
    group: "nav.configuration",
    items: [{ to: "/settings", label: "nav.settings", icon: Settings }],
  },
];

export function Sidebar() {
  const { t } = useLanguage();

  return (
    <aside className="flex w-60 shrink-0 flex-col border-r border-line bg-surface">
      <div className="flex items-center gap-3 px-5 py-5">
        <div className="flex size-9 items-center justify-center rounded-lg bg-accent/15 text-accent">
          <ShieldCheck className="size-5" />
        </div>
        <div className="min-w-0">
          <p className="truncate text-sm font-bold tracking-tight text-ink">
            VirusAnalyzer <span className="text-accent">2.0</span>
          </p>
          <p className="truncate text-[10px] uppercase tracking-widest text-muted">
            {t("app.tagline")}
          </p>
        </div>
      </div>

      <nav className="flex-1 overflow-y-auto px-3 pb-4">
        {NAV_SECTIONS.map((section) => (
          <div key={section.group} className="mt-4 first:mt-1">
            <p className="px-3 pb-1.5 text-[10px] font-semibold uppercase tracking-widest text-muted/70">
              {t(section.group)}
            </p>
            <ul className="space-y-0.5">
              {section.items.map((item) => (
                <li key={item.to}>
                  <NavLink
                    to={item.to}
                    end={item.end}
                    className={({ isActive }) =>
                      `group flex items-center gap-2.5 rounded-lg px-3 py-2 text-[13px] font-medium transition-colors ${
                        isActive
                          ? "bg-accent/10 text-accent"
                          : "text-muted hover:bg-surface-2 hover:text-ink"
                      }`
                    }
                  >
                    <item.icon className="size-4" />
                    {t(item.label)}
                  </NavLink>
                </li>
              ))}
            </ul>
          </div>
        ))}
      </nav>

      <div className="border-t border-line px-5 py-3">
        <p className="text-[10px] leading-relaxed text-muted">{t("app.analyzeUnderstandProtect")}</p>
      </div>
    </aside>
  );
}
