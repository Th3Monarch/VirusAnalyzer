import { BrowserRouter, Route, Routes } from "react-router-dom";
import { basename } from "./config";
import { Layout } from "./components/Layout";
import { ScrollToTop } from "./components/ScrollToTop";
import { Home } from "./pages/Home";
import { Download } from "./pages/Download";
import { Features } from "./pages/Features";
import { Security } from "./pages/Security";
import { Documentation } from "./pages/Documentation";
import { Faq } from "./pages/Faq";
import { About } from "./pages/About";
import { Changelog } from "./pages/Changelog";
import { NotFound } from "./pages/NotFound";

export default function App() {
  return (
    <BrowserRouter basename={basename}>
      <ScrollToTop />
      <Routes>
        <Route element={<Layout />}>
          <Route path="/" element={<Home />} />
          <Route path="/download" element={<Download />} />
          <Route path="/features" element={<Features />} />
          <Route path="/security" element={<Security />} />
          <Route path="/documentation" element={<Documentation />} />
          <Route path="/faq" element={<Faq />} />
          <Route path="/about" element={<About />} />
          <Route path="/changelog" element={<Changelog />} />
          <Route path="*" element={<NotFound />} />
        </Route>
      </Routes>
    </BrowserRouter>
  );
}
