import { useEffect, useState, useCallback } from "react";
import { useDebounce } from "./useDebounce";

interface UseDebouncedSearchOptions {
  delay?: number;
  onSearch: (query: string) => Promise<void>;
}

export function useDebouncedSearch({
  delay = 300,
  onSearch,
}: UseDebouncedSearchOptions) {
  const [query, setQuery] = useState("");
  const [isSearching, setIsSearching] = useState(false);
  const debouncedQuery = useDebounce(query, delay);

  const executeSearch = useCallback(async () => {
    if (!debouncedQuery.trim()) {
      return;
    }
    setIsSearching(true);
    try {
      await onSearch(debouncedQuery);
    } catch (error) {
      console.error("Search failed:", error);
    } finally {
      setIsSearching(false);
    }
  }, [debouncedQuery, onSearch]);

  useEffect(() => {
    executeSearch();
  }, [executeSearch]);

  const clearSearch = useCallback(() => {
    setQuery("");
    setIsSearching(false);
  }, []);

  return {
    query,
    setQuery,
    isSearching,
    clearSearch,
  };
}
