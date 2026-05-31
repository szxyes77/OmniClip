import { create } from "zustand";
import type { ClipboardRecord } from "../types";
import { searchRecords } from "../api/clipboard";

interface SearchState {
  query: string;
  results: ClipboardRecord[];
  isSearching: boolean;
  hasSearched: boolean;

  setQuery: (q: string) => void;
  executeSearch: () => Promise<void>;
  clearSearch: () => void;
}

export const useSearchStore = create<SearchState>((set, get) => ({
  query: "",
  results: [],
  isSearching: false,
  hasSearched: false,

  setQuery: (q) => set({ query: q }),

  executeSearch: async () => {
    const { query } = get();
    if (!query.trim()) {
      set({ results: [], hasSearched: false });
      return;
    }

    set({ isSearching: true });
    try {
      const results = await searchRecords(query);
      set({ results, isSearching: false, hasSearched: true });
    } catch (error) {
      console.error("Search failed:", error);
      set({ isSearching: false });
    }
  },

  clearSearch: () => set({ query: "", results: [], hasSearched: false }),
}));
