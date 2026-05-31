import { create } from "zustand";
import { invoke } from "@tauri-apps/api/core";
import type { ClipboardRecord, Tag, AppSettings } from "../types";
import { getHistory, searchClips as apiSearchClips, starItem, deleteItem, addTags, pasteRecord } from "../api/clipboard";
import { getTags, renameTag as apiRenameTag, deleteTag as apiDeleteTag } from "../api/tags";
import { getSettings } from "../api/settings";

interface ClipboardState {
  clips: ClipboardRecord[];
  tags: Tag[];
  settings: AppSettings | null;
  searchQuery: string;
  activeTagFilter: string | null;
  starredOnly: boolean;
  isLoading: boolean;
  hasMore: boolean;
  offset: number;

  loadInitial: () => Promise<void>;
  loadMore: () => Promise<void>;
  setSearchQuery: (query: string) => void;
  setActiveTagFilter: (tagId: string | null) => void;
  toggleStarredOnly: () => void;
  handleStar: (id: string) => Promise<void>;
  handleDelete: (id: string) => Promise<void>;
  handlePaste: (id: string) => Promise<void>;
  handleAddTags: (id: string, tags: string[]) => Promise<void>;
  refreshTags: () => Promise<void>;
  renameTag: (id: string, newName: string) => Promise<void>;
  deleteTag: (id: string) => Promise<void>;
  loadSettings: () => Promise<void>;
  updateSettings: (settings: Record<string, unknown>) => Promise<void>;
}

export const useClipboardStore = create<ClipboardState>((set, get) => ({
  clips: [],
  tags: [],
  settings: null,
  searchQuery: "",
  activeTagFilter: null,
  starredOnly: false,
  isLoading: false,
  hasMore: true,
  offset: 0,

  loadInitial: async () => {
    set({ isLoading: true, clips: [], offset: 0, hasMore: true });
    try {
      const { searchQuery } = get();
      let records: ClipboardRecord[] = [];

      if (searchQuery.trim()) {
        records = await apiSearchClips(searchQuery, 50);
      } else {
        records = await getHistory("", 50, 0);
      }

      set({ clips: records, offset: records.length, hasMore: records.length === 50, isLoading: false });
    } catch (error) {
      console.error("Failed to load records:", error);
      set({ isLoading: false });
    }
  },

  loadMore: async () => {
    const { offset, isLoading, hasMore, searchQuery } = get();
    if (isLoading || !hasMore) return;

    set({ isLoading: true });
    try {
      let newRecords: ClipboardRecord[] = [];
      if (searchQuery.trim()) {
        newRecords = await apiSearchClips(searchQuery, 50);
      } else {
        newRecords = await getHistory("", 50, offset);
      }

      set((state) => ({
        clips: [...state.clips, ...newRecords],
        offset: state.offset + newRecords.length,
        hasMore: newRecords.length === 50,
        isLoading: false,
      }));
    } catch (error) {
      console.error("Failed to load more:", error);
      set({ isLoading: false });
    }
  },

  setSearchQuery: (query: string) => {
    set({ searchQuery: query });
  },

  setActiveTagFilter: (tagId: string | null) => {
    set({ activeTagFilter: tagId });
  },

  toggleStarredOnly: () => {
    set((state) => ({ starredOnly: !state.starredOnly }));
  },

  handleStar: async (id: string) => {
    try {
      const newStarred = await starItem(id);
      set((state) => ({
        clips: state.clips.map((r) => (r.id === id ? { ...r, is_starred: newStarred } : r)),
      }));
    } catch (error) {
      console.error("Failed to star:", error);
    }
  },

  handleDelete: async (id: string) => {
    try {
      await deleteItem(id);
      set((state) => ({
        clips: state.clips.filter((r) => r.id !== id),
      }));
    } catch (error) {
      console.error("Failed to delete:", error);
    }
  },

  handlePaste: async (id: string) => {
    try {
      await pasteRecord(id);
      const now = new Date().toISOString();
      set((state) => ({
        clips: state.clips.map((r) =>
          r.id === id ? { ...r, copy_count: r.copy_count + 1, last_used_at: now } : r
        ),
      }));
    } catch (error) {
      console.error("Failed to paste:", error);
    }
  },

  handleAddTags: async (id: string, tags: string[]) => {
    try {
      await addTags(id, tags);
    } catch (error) {
      console.error("Failed to add tags:", error);
    }
  },

  refreshTags: async () => {
    try {
      const tags = await getTags();
      set({ tags });
    } catch (error) {
      console.error("Failed to refresh tags:", error);
    }
  },

  renameTag: async (id: string, newName: string) => {
    try {
      await apiRenameTag(id, newName);
      await get().refreshTags();
    } catch (error) {
      console.error("Failed to rename tag:", error);
    }
  },

  deleteTag: async (id: string) => {
    try {
      await apiDeleteTag(id);
      const { activeTagFilter } = get();
      if (activeTagFilter === id) {
        set({ activeTagFilter: null });
      }
      await get().refreshTags();
    } catch (error) {
      console.error("Failed to delete tag:", error);
    }
  },

  loadSettings: async () => {
    try {
      const settings = await getSettings();
      set({ settings });
    } catch (error) {
      console.error("Failed to load settings:", error);
    }
  },

  updateSettings: async (settings: Record<string, unknown>) => {
    try {
      const updatePayload: Record<string, unknown> = {};
      if (settings.max_history_count !== undefined) {
        updatePayload.max_history_count = settings.max_history_count;
      }
      if (settings.auto_cleanup_days !== undefined) {
        updatePayload.auto_cleanup_days = settings.auto_cleanup_days;
      }
      if (settings.keep_starred !== undefined) {
        updatePayload.keep_starred = settings.keep_starred;
      }
      if (settings.global_shortcut !== undefined) {
        updatePayload.global_shortcut = settings.global_shortcut;
      }
      if (settings.theme !== undefined) {
        updatePayload.theme = settings.theme;
      }
      if (settings.master_password_hash !== undefined) {
        updatePayload.master_password_hash = settings.master_password_hash;
      }
      if (settings.ignore_apps !== undefined) {
        updatePayload.ignore_apps = settings.ignore_apps;
      }

      await invoke<void>("update_settings", { settings: updatePayload });
    } catch (error) {
      console.error("Failed to update settings:", error);
      throw error;
    }
  },
}));
