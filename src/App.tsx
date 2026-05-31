import { useState, useEffect } from "react";
import { Settings } from "lucide-react";
import { useI18n } from "./i18n/I18nContext";
import { SearchBar } from "./components/main/SearchBar";
import { RecordList } from "./components/main/RecordList";
import { TagSidebar } from "./components/main/TagSidebar";
import { SettingsPage } from "./components/main/SettingsPage";
import { useClipboardStore } from "./store/clipboardStore";

export default function App() {
  const { t } = useI18n();
  const { loadInitial, refreshTags, loadSettings } = useClipboardStore();
  const [showSettings, setShowSettings] = useState(false);

  useEffect(() => {
    loadInitial();
    refreshTags();
    loadSettings();
  }, []);

  if (showSettings) {
    return (
      <div className="flex h-screen bg-background text-foreground">
        <div className="flex-1 flex flex-col">
          <div className="flex items-center justify-between px-6 py-3 border-b border-border">
            <button
              onClick={() => setShowSettings(false)}
              className="text-sm text-foreground hover:text-accent transition-colors"
            >
              {t("app.backToHome")}
            </button>
          </div>
          <SettingsPage />
        </div>
      </div>
    );
  }

  return (
    <div className="flex h-screen bg-background text-foreground">
      <div className="flex-1 flex flex-col min-w-0">
        <div className="flex items-center justify-end px-4 py-2 border-b border-border">
          <button
            onClick={() => setShowSettings(true)}
            className="p-2 hover:bg-surface/50 rounded-md transition-colors"
            title={t("app.settings")}
          >
            <Settings className="w-4 h-4 text-muted-foreground hover:text-foreground" />
          </button>
        </div>
        <SearchBar />
        <RecordList />
      </div>
      <TagSidebar />
    </div>
  );
}
