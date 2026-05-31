import { useState, useEffect } from "react";
import {
  Save,
  Lock,
  Clock,
  Ban,
  Plus,
  X,
  Download,
  Upload,
  Eye,
  EyeOff,
  Globe,
} from "lucide-react";
import { invoke } from "@tauri-apps/api/core";
import { useClipboardStore } from "../../store/clipboardStore";
import { exportData, importData } from "../../api/data";
import { useI18n } from "../../i18n/I18nContext";
import type { Language } from "../../i18n/I18nContext";

export function SettingsPage() {
  const { t, language, setLanguage } = useI18n();
  const { settings, loadSettings } = useClipboardStore();

  const [maxHistory, setMaxHistory] = useState(1000);
  const [cleanupDays, setCleanupDays] = useState(30);
  const [masterPassword, setMasterPassword] = useState("");
  const [confirmPassword, setConfirmPassword] = useState("");
  const [showPassword, setShowPassword] = useState(false);
  const [ignoreApps, setIgnoreApps] = useState<string[]>([]);
  const [newApp, setNewApp] = useState("");
  const [statusMessage, setStatusMessage] = useState("");
  const [isStatusError, setIsStatusError] = useState(false);
  const [isSaving, setIsSaving] = useState(false);

  useEffect(() => {
    if (settings) {
      setMaxHistory(settings.max_history_count);
      setCleanupDays(settings.auto_cleanup_days);
      setIgnoreApps(settings.ignore_apps || []);
    }
  }, [settings]);

  const handleSave = async () => {
    if (masterPassword && masterPassword !== confirmPassword) {
      setStatusMessage(t("settings.passwordMismatch"));
      setIsStatusError(true);
      return;
    }

    setIsSaving(true);
    setStatusMessage(t("settings.saving"));
    setIsStatusError(false);

    try {
      const update: Record<string, unknown> = {
        max_history_count: maxHistory,
        auto_cleanup_days: cleanupDays,
        ignore_apps: ignoreApps,
      };

      if (masterPassword.trim()) {
        const hash = await hashPassword(masterPassword);
        update.master_password_hash = hash;
      }

      await useClipboardStore.getState().updateSettings(update);
      await loadSettings();

      setStatusMessage(t("settings.saved"));
      setIsStatusError(false);
      setMasterPassword("");
      setConfirmPassword("");
    } catch (error) {
      console.error("Failed to save settings:", error);
      setStatusMessage(t("settings.saveFailed"));
      setIsStatusError(true);
    } finally {
      setIsSaving(false);
    }
  };

  const hashPassword = async (password: string): Promise<string> => {
    const encoder = new TextEncoder();
    const data = encoder.encode(password);
    const hashBuffer = await crypto.subtle.digest("SHA-256", data);
    const hashArray = Array.from(new Uint8Array(hashBuffer));
    return hashArray.map((b) => b.toString(16).padStart(2, "0")).join("");
  };

  const addIgnoreApp = () => {
    if (newApp.trim() && !ignoreApps.includes(newApp.trim())) {
      setIgnoreApps([...ignoreApps, newApp.trim()]);
      setNewApp("");
    }
  };

  const removeIgnoreApp = (app: string) => {
    setIgnoreApps(ignoreApps.filter((a) => a !== app));
  };

  const handleExport = async () => {
    try {
      const selectedPath = await invoke<string>("open_save_dialog", {
        defaultPath: `omniclip_backup_${new Date().toISOString().split("T")[0]}.json`,
      });

      if (!selectedPath) return;

      await exportData(selectedPath);
      setStatusMessage(t("settings.exportSuccess"));
      setIsStatusError(false);
    } catch (error) {
      console.error("Export failed:", error);
      setStatusMessage(t("settings.exportFailed"));
      setIsStatusError(true);
    }
  };

  const handleImport = async () => {
    try {
      const selected = await invoke<string>("open_file_dialog", {});

      if (!selected) return;

      const result = await importData(selected);
      setStatusMessage(
        t("settings.importSuccess", {
          records: result.imported_records,
          tags: result.imported_tags,
          duplicates: result.skipped_duplicates,
        })
      );
      setIsStatusError(false);
      await useClipboardStore.getState().loadInitial();
      await useClipboardStore.getState().refreshTags();
      await useClipboardStore.getState().loadSettings();
    } catch (error) {
      console.error("Import failed:", error);
      setStatusMessage(t("settings.importFailed"));
      setIsStatusError(true);
    }
  };

  return (
    <div className="flex-1 overflow-y-auto p-6 space-y-6">
      <div className="max-w-2xl mx-auto space-y-6">
        <div className="space-y-1">
          <h2 className="text-2xl font-bold text-foreground">
            {t("settings.title")}
          </h2>
          <p className="text-sm text-muted-foreground">
            {t("settings.description")}
          </p>
        </div>

        <div className="bg-surface/50 border border-border rounded-xl p-4 space-y-3">
          <div className="flex items-center gap-2">
            <Globe className="w-4 h-4 text-purple-500" />
            <h3 className="text-sm font-semibold text-foreground">
              {t("settings.language")}
            </h3>
          </div>
          <div className="flex gap-2">
            {(["zh", "en"] as Language[]).map((lang) => (
              <button
                key={lang}
                onClick={() => setLanguage(lang)}
                className={`px-4 py-1.5 rounded-md text-sm font-medium transition-colors ${
                  language === lang
                    ? "bg-accent text-accent-foreground"
                    : "bg-surface/50 text-muted-foreground hover:text-foreground"
                }`}
              >
                {lang === "zh" ? t("settings.chinese") : t("settings.english")}
              </button>
            ))}
          </div>
        </div>

        <div className="space-y-4">
          <div className="bg-surface/50 border border-border rounded-xl p-4 space-y-4">
            <div className="flex items-center gap-2">
              <Lock className="w-4 h-4 text-amber-500" />
              <h3 className="text-sm font-semibold text-foreground">
                {t("settings.masterPassword")}
              </h3>
            </div>
            <p className="text-xs text-muted-foreground">
              {t("settings.masterPasswordDesc")}
            </p>
            <div className="space-y-2">
              <div className="relative">
                <input
                  type={showPassword ? "text" : "password"}
                  value={masterPassword}
                  onChange={(e) => setMasterPassword(e.target.value)}
                  placeholder={t("settings.masterPasswordPlaceholder")}
                  className="w-full px-3 py-2 pr-10 bg-surface/50 border border-border rounded-md text-sm text-foreground placeholder:text-muted-foreground/60 focus:outline-none focus:ring-2 focus:ring-accent/50 focus:border-transparent"
                />
                <button
                  onClick={() => setShowPassword(!showPassword)}
                  className="absolute right-3 top-1/2 -translate-y-1/2 p-1 text-muted-foreground hover:text-foreground"
                >
                  {showPassword ? (
                    <EyeOff className="w-4 h-4" />
                  ) : (
                    <Eye className="w-4 h-4" />
                  )}
                </button>
              </div>
              {masterPassword && (
                <input
                  type="password"
                  value={confirmPassword}
                  onChange={(e) => setConfirmPassword(e.target.value)}
                  placeholder={t("settings.confirmPassword")}
                  className="w-full px-3 py-2 bg-surface/50 border border-border rounded-md text-sm text-foreground placeholder:text-muted-foreground/60 focus:outline-none focus:ring-2 focus:ring-accent/50 focus:border-transparent"
                />
              )}
            </div>
          </div>

          <div className="bg-surface/50 border border-border rounded-xl p-4 space-y-4">
            <div className="flex items-center gap-2">
              <Clock className="w-4 h-4 text-blue-500" />
              <h3 className="text-sm font-semibold text-foreground">
                {t("settings.dataRetention")}
              </h3>
            </div>
            <div className="grid grid-cols-2 gap-4">
              <div className="space-y-2">
                <label className="text-xs text-muted-foreground">
                  {t("settings.maxHistoryEntries")}
                </label>
                <input
                  type="number"
                  value={maxHistory}
                  onChange={(e) => setMaxHistory(Number(e.target.value))}
                  min={100}
                  max={10000}
                  className="w-full px-3 py-2 bg-surface/50 border border-border rounded-md text-sm text-foreground focus:outline-none focus:ring-2 focus:ring-accent/50 focus:border-transparent"
                />
              </div>
              <div className="space-y-2">
                <label className="text-xs text-muted-foreground">
                  {t("settings.autoCleanupDays")}
                </label>
                <input
                  type="number"
                  value={cleanupDays}
                  onChange={(e) => setCleanupDays(Number(e.target.value))}
                  min={0}
                  max={365}
                  className="w-full px-3 py-2 bg-surface/50 border border-border rounded-md text-sm text-foreground focus:outline-none focus:ring-2 focus:ring-accent/50 focus:border-transparent"
                />
              </div>
            </div>
          </div>

          <div className="bg-surface/50 border border-border rounded-xl p-4 space-y-4">
            <div className="flex items-center gap-2">
              <Ban className="w-4 h-4 text-red-500" />
              <h3 className="text-sm font-semibold text-foreground">
                {t("settings.ignoredApps")}
              </h3>
            </div>
            <p className="text-xs text-muted-foreground">
              {t("settings.ignoredAppsDesc")}
            </p>
            <div className="flex gap-2">
              <input
                type="text"
                value={newApp}
                onChange={(e) => setNewApp(e.target.value)}
                onKeyDown={(e) => e.key === "Enter" && addIgnoreApp()}
                placeholder={t("settings.ignoreAppsPlaceholder")}
                className="flex-1 px-3 py-2 bg-surface/50 border border-border rounded-md text-sm text-foreground placeholder:text-muted-foreground/60 focus:outline-none focus:ring-2 focus:ring-accent/50 focus:border-transparent"
              />
              <button
                onClick={addIgnoreApp}
                className="px-3 py-2 bg-accent/10 hover:bg-accent/20 rounded-md transition-colors"
              >
                <Plus className="w-4 h-4 text-accent" />
              </button>
            </div>
            {ignoreApps.length > 0 && (
              <div className="flex flex-wrap gap-2">
                {ignoreApps.map((app) => (
                  <span
                    key={app}
                    className="inline-flex items-center gap-1 px-2 py-1 bg-red-500/10 border border-red-500/20 rounded-md text-xs text-red-400"
                  >
                    {app}
                    <button
                      onClick={() => removeIgnoreApp(app)}
                      className="hover:text-red-300"
                    >
                      <X className="w-3 h-3" />
                    </button>
                  </span>
                ))}
              </div>
            )}
          </div>

          <div className="bg-surface/50 border border-border rounded-xl p-4 space-y-4">
            <div className="flex items-center gap-2">
              <Download className="w-4 h-4 text-green-500" />
              <h3 className="text-sm font-semibold text-foreground">
                {t("settings.dataExportImport")}
              </h3>
            </div>
            <p className="text-xs text-muted-foreground">
              {t("settings.dataExportImportDesc")}
            </p>
            <div className="flex gap-2">
              <button
                onClick={handleExport}
                className="flex-1 flex items-center justify-center gap-2 px-4 py-2 bg-green-500/10 hover:bg-green-500/20 rounded-md text-sm font-medium text-green-500 transition-colors"
              >
                <Download className="w-4 h-4" />
                {t("settings.exportData")}
              </button>
              <button
                onClick={handleImport}
                className="flex-1 flex items-center justify-center gap-2 px-4 py-2 bg-blue-500/10 hover:bg-blue-500/20 rounded-md text-sm font-medium text-blue-500 transition-colors"
              >
                <Upload className="w-4 h-4" />
                {t("settings.importData")}
              </button>
            </div>
          </div>
        </div>

        <div className="flex items-center gap-4">
          <button
            onClick={handleSave}
            disabled={isSaving}
            className="flex items-center gap-2 px-6 py-2.5 bg-accent hover:bg-accent/80 disabled:opacity-50 rounded-lg text-sm font-semibold text-accent-foreground transition-colors"
          >
            <Save className="w-4 h-4" />
            {isSaving ? t("settings.saving") : t("settings.saveSettings")}
          </button>

          {statusMessage && (
            <span
              className={`text-sm ${
                isStatusError ? "text-red-500" : "text-green-500"
              }`}
            >
              {statusMessage}
            </span>
          )}
        </div>
      </div>
    </div>
  );
}
