import { Settings, Palette, Database, History } from "lucide-react";
import { useSettingsStore } from "../../store/settingsStore";
import { useState } from "react";

type SettingsTab = "general" | "appearance" | "data";

export function SettingsPanel() {
  const { settings, updateSetting } = useSettingsStore();
  const [activeTab, setActiveTab] = useState<SettingsTab>("general");

  if (!settings) {
    return (
      <div className="flex-1 flex items-center justify-center text-gray-400">
        <div className="w-6 h-6 border-2 border-gray-300 border-t-gray-500 rounded-full animate-spin" />
      </div>
    );
  }

  const tabs = [
    { key: "general" as SettingsTab, label: "General", icon: <Settings className="w-4 h-4" /> },
    { key: "appearance" as SettingsTab, label: "Appearance", icon: <Palette className="w-4 h-4" /> },
    { key: "data" as SettingsTab, label: "Data", icon: <Database className="w-4 h-4" /> },
  ];

  return (
    <div className="flex-1 flex overflow-hidden">
      <div className="w-48 border-r border-gray-200/50 dark:border-gray-700/50 p-3 space-y-1">
        {tabs.map((tab) => (
          <button
            key={tab.key}
            onClick={() => setActiveTab(tab.key)}
            className={`w-full flex items-center gap-2 px-3 py-2 rounded-lg text-sm transition-all ${
              activeTab === tab.key
                ? "bg-indigo-500/10 text-indigo-700 dark:text-indigo-300 font-medium"
                : "text-gray-600 dark:text-gray-400 hover:bg-gray-100/60 dark:hover:bg-slate-800/60"
            }`}
          >
            {tab.icon}
            {tab.label}
          </button>
        ))}
      </div>

      <div className="flex-1 p-8 overflow-y-auto">
        <div className="max-w-lg">
          {activeTab === "general" && (
            <div className="space-y-6">
              <h2 className="text-lg font-semibold text-gray-900 dark:text-gray-100">
                General Settings
              </h2>

              <div>
                <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                  Global Shortcut
                </label>
                <input
                  type="text"
                  value={settings.global_shortcut}
                  onChange={(e) => updateSetting("global_shortcut", e.target.value)}
                  className="w-full px-3 py-2 bg-white dark:bg-slate-800 border border-gray-200 dark:border-gray-700 rounded-lg text-sm focus:ring-2 focus:ring-indigo-500/20 focus:border-indigo-500 outline-none"
                />
                <p className="text-xs text-gray-500 mt-1">
                  Press to toggle the floating clipboard window
                </p>
              </div>

              <div>
                <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                  <History className="w-4 h-4 inline mr-1" />
                  Max History Count
                </label>
                <input
                  type="number"
                  value={settings.max_history_count}
                  onChange={(e) =>
                    updateSetting("max_history_count", parseInt(e.target.value) || 1000)
                  }
                  className="w-full px-3 py-2 bg-white dark:bg-slate-800 border border-gray-200 dark:border-gray-700 rounded-lg text-sm focus:ring-2 focus:ring-indigo-500/20 focus:border-indigo-500 outline-none"
                  min={100}
                  max={10000}
                />
              </div>

              <div>
                <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                  Auto Cleanup (days)
                </label>
                <input
                  type="number"
                  value={settings.auto_cleanup_days}
                  onChange={(e) =>
                    updateSetting("auto_cleanup_days", parseInt(e.target.value) || 30)
                  }
                  className="w-full px-3 py-2 bg-white dark:bg-slate-800 border border-gray-200 dark:border-gray-700 rounded-lg text-sm focus:ring-2 focus:ring-indigo-500/20 focus:border-indigo-500 outline-none"
                  min={1}
                  max={365}
                />
                <p className="text-xs text-gray-500 mt-1">
                  Records older than this will be auto-deleted (except starred)
                </p>
              </div>

              <div className="flex items-center justify-between">
                <span className="text-sm text-gray-700 dark:text-gray-300">
                  Keep starred records
                </span>
                <button
                  onClick={() => updateSetting("keep_starred", !settings.keep_starred)}
                  className={`relative w-11 h-6 rounded-full transition-colors ${
                    settings.keep_starred ? "bg-indigo-500" : "bg-gray-300 dark:bg-slate-600"
                  }`}
                >
                  <span
                    className={`absolute top-0.5 left-0.5 w-5 h-5 bg-white rounded-full transition-transform ${
                      settings.keep_starred ? "translate-x-5" : "translate-x-0"
                    }`}
                  />
                </button>
              </div>
            </div>
          )}

          {activeTab === "appearance" && (
            <div className="space-y-6">
              <h2 className="text-lg font-semibold text-gray-900 dark:text-gray-100">
                Appearance
              </h2>

              <div>
                <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-3">
                  Theme
                </label>
                <div className="grid grid-cols-3 gap-3">
                  {(["light", "dark", "system"] as const).map((theme) => (
                    <button
                      key={theme}
                      onClick={() => updateSetting("theme", theme)}
                      className={`px-4 py-3 rounded-xl border-2 text-sm font-medium transition-all ${
                        settings.theme === theme
                          ? "border-indigo-500 bg-indigo-50 dark:bg-indigo-500/10 text-indigo-700 dark:text-indigo-300"
                          : "border-gray-200 dark:border-gray-700 text-gray-600 dark:text-gray-400 hover:border-gray-300"
                      }`}
                    >
                      {theme.charAt(0).toUpperCase() + theme.slice(1)}
                    </button>
                  ))}
                </div>
              </div>
            </div>
          )}

          {activeTab === "data" && (
            <div className="space-y-6">
              <h2 className="text-lg font-semibold text-gray-900 dark:text-gray-100">
                Data Management
              </h2>

              <div className="p-4 bg-amber-50 dark:bg-amber-500/10 border border-amber-200 dark:border-amber-500/20 rounded-xl">
                <p className="text-sm text-amber-800 dark:text-amber-400 font-medium">
                  Export / Import
                </p>
                <p className="text-xs text-amber-600 dark:text-amber-500 mt-1">
                  Export your clipboard history as JSON or import from a backup file.
                </p>
                <div className="flex gap-2 mt-3">
                  <button className="px-3 py-1.5 bg-amber-600 hover:bg-amber-700 text-white text-sm rounded-lg transition-colors">
                    Export Data
                  </button>
                  <button className="px-3 py-1.5 bg-white dark:bg-slate-800 text-amber-700 dark:text-amber-400 border border-amber-200 dark:border-amber-500/20 text-sm rounded-lg hover:bg-amber-100 dark:hover:bg-amber-500/10 transition-colors">
                    Import Data
                  </button>
                </div>
              </div>

              <div className="p-4 bg-red-50 dark:bg-red-500/10 border border-red-200 dark:border-red-500/20 rounded-xl">
                <p className="text-sm text-red-800 dark:text-red-400 font-medium">
                  Danger Zone
                </p>
                <p className="text-xs text-red-600 dark:text-red-500 mt-1">
                  This action cannot be undone. All clipboard history will be permanently deleted.
                </p>
                <button className="mt-3 px-3 py-1.5 bg-red-600 hover:bg-red-700 text-white text-sm rounded-lg transition-colors">
                  Clear All History
                </button>
              </div>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
