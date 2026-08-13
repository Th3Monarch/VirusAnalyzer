import { useEffect } from "react";
import { Outlet, useNavigate } from "react-router-dom";
import { Sidebar } from "./Sidebar";
import { Topbar } from "./Topbar";
import { tauri } from "../../lib/tauri";

export function Layout() {
  const navigate = useNavigate();

  useEffect(() => {
    let alive = true;
    void tauri
      .takeLaunchPath()
      .then((path) => {
        if (alive && path) {
          navigate(`/scan?path=${encodeURIComponent(path)}`, { replace: true });
        }
      })
      .catch(() => undefined);
    return () => {
      alive = false;
    };
  }, [navigate]);

  return (
    <div className="flex h-full overflow-hidden bg-background text-ink">
      <Sidebar />
      <div className="flex min-w-0 flex-1 flex-col">
        <Topbar />
        <main className="flex-1 overflow-y-auto p-6">
          <div key="page" className="animate-va-fade-up">
            <Outlet />
          </div>
        </main>
      </div>
    </div>
  );
}
