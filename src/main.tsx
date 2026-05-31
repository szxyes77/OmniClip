import React from "react";
import ReactDOM from "react-dom/client";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import { I18nProvider } from "./i18n/I18nContext";
import { OverlayWindow } from "./components/overlay/OverlayWindow";
import App from "./App";
import "./index.css";

async function checkWindowLabel(): Promise<string> {
  try {
    const window = getCurrentWebviewWindow();
    return window.label;
  } catch {
    return "main";
  }
}

async function bootstrap() {
  const label = await checkWindowLabel();

  if (label === "overlay") {
    ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
      <React.StrictMode>
        <I18nProvider>
          <div id="overlay-root" className="w-screen h-screen">
            <OverlayWindow />
          </div>
        </I18nProvider>
      </React.StrictMode>
    );
  } else {
    ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
      <React.StrictMode>
        <I18nProvider>
          <App />
        </I18nProvider>
      </React.StrictMode>
    );
  }
}

bootstrap();
