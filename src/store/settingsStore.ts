import { create } from "zustand";
import type { AppSettings, Tag } from "../types";
import { getSettings, updateSettings } from "../api/settings";
import { getTags } from "../api/tags";

interface SettingsState {
  settings: AppSettings | null;
  tags: Tag[];
  isLoading: boolean;

  loadSettings: () => Promise<void>;
  loadTags: () => Promise<void>;
  updateSetting: <K extends keyof AppSettings>(key: K, value: AppSettings[K]) => Promise<void>;
}

export const useSettingsStore = create<SettingsState>((set) => ({
  settings: null,
  tags: [],
  isLoading: false,

  loadSettings: async () => {
    set({ isLoading: true });
    try {
      const settings = await getSettings();
      set({ settings, isLoading: false });
    } catch (error) {
      console.error("Failed to load settings:", error);
      set({ isLoading: false });
    }
  },

  loadTags: async () => {
    try {
      const tags = await getTags();
      set({ tags });
    } catch (error) {
      console.error("Failed to load tags:", error);
    }
  },

  updateSetting: async (key, value) => {
    try {
      await updateSettings({ [key]: value });
      set((state) => ({
        settings: state.settings ? { ...state.settings, [key]: value } : null,
      }));
    } catch (error) {
      console.error("Failed to update settings:", error);
    }
  },
}));
