import { lazy, Suspense } from "react";
import { HashRouter, Route, Routes } from "react-router-dom";
import { Layout } from "./components/layout/Layout";
import { Card } from "./components/ui/Card";

const Dashboard = lazy(() => import("./pages/Dashboard").then((m) => ({ default: m.Dashboard })));
const Scan = lazy(() => import("./pages/Scan").then((m) => ({ default: m.Scan })));
const Results = lazy(() => import("./pages/Results").then((m) => ({ default: m.Results })));
const Analysis = lazy(() => import("./pages/Analysis").then((m) => ({ default: m.Analysis })));
const Quarantine = lazy(() => import("./pages/Quarantine").then((m) => ({ default: m.Quarantine })));
const Rules = lazy(() => import("./pages/Rules").then((m) => ({ default: m.Rules })));
const System = lazy(() => import("./pages/System").then((m) => ({ default: m.System })));
const PowerShell = lazy(() => import("./pages/PowerShell").then((m) => ({ default: m.PowerShell })));
const PSReference = lazy(() => import("./pages/PSReference").then((m) => ({ default: m.PSReference })));
const Settings = lazy(() => import("./pages/Settings").then((m) => ({ default: m.Settings })));

function PageLoader() {
  return (
    <Card>
      <p role="status" className="py-8 text-center text-sm text-muted">…</p>
    </Card>
  );
}

function App() {
  return (
    <HashRouter>
      <Suspense fallback={<PageLoader />}>
        <Routes>
          <Route element={<Layout />}>
            <Route path="/" element={<Dashboard />} />
            <Route path="/scan" element={<Scan />} />
            <Route path="/results" element={<Results />} />
            <Route path="/analysis/:id?" element={<Analysis />} />
            <Route path="/quarantine" element={<Quarantine />} />
            <Route path="/rules" element={<Rules />} />
            <Route path="/system" element={<System />} />
            <Route path="/powershell" element={<PowerShell />} />
            <Route path="/ps-reference" element={<PSReference />} />
            <Route path="/settings" element={<Settings />} />
          </Route>
        </Routes>
      </Suspense>
    </HashRouter>
  );
}

export default App;
