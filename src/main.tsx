import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";
import { AssistantProvider } from "./contexts/AssistantContext";
import { ConfigProvider } from "./contexts/ConfigContext";
import { PlatformProvider } from "./contexts/PlatformContext";
import { ThemeProvider } from "./contexts/ThemeContext";
import { LanguageProvider } from "./contexts/LanguageContext";
import { ToastProvider } from "./contexts/ToastContext";
import "./index.css";

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <ConfigProvider>
      <ThemeProvider>
        <LanguageProvider>
          <PlatformProvider>
            <ToastProvider>
              <AssistantProvider>
                <App />
              </AssistantProvider>
            </ToastProvider>
          </PlatformProvider>
        </LanguageProvider>
      </ThemeProvider>
    </ConfigProvider>
  </React.StrictMode>,
);
