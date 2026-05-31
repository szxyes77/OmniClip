import { useCallback, useEffect, useState, useRef } from "react";
import { getHistory, onClipboardUpdate, starItem as apiStarItem, deleteItem as apiDeleteItem, addTags as apiAddTags } from "../api/clipboard";
import type { ClipboardRecord } from "../types";

interface UseClipboardHistoryOptions {
  limit?: number;
  autoRefresh?: boolean;
}

export function useClipboardHistory({
  limit = 50,
  autoRefresh = true,
}: UseClipboardHistoryOptions = {}) {
  const [records, setRecords] = useState<ClipboardRecord[]>([]);
  const [isLoading, setIsLoading] = useState(false);
  const [hasMore, setHasMore] = useState(true);
  const offsetRef = useRef(0);

  const loadHistory = useCallback(
    async (search = "", reset = false) => {
      if (reset) {
        offsetRef.current = 0;
        setHasMore(true);
      }

      if (isLoading || (!hasMore && !reset)) return;

      setIsLoading(true);
      try {
        const currentOffset = reset ? 0 : offsetRef.current;
        const data = await getHistory(search, limit, currentOffset);

        if (reset) {
          setRecords(data);
        } else {
          setRecords((prev) => [...prev, ...data]);
        }

        offsetRef.current = currentOffset + data.length;
        setHasMore(data.length === limit);
      } catch (error) {
        console.error("Failed to load clipboard history:", error);
      } finally {
        setIsLoading(false);
      }
    },
    [limit, isLoading, hasMore]
  );

  const loadMore = useCallback(() => {
    loadHistory();
  }, [loadHistory]);

  const handleStarItem = useCallback(async (id: string) => {
    try {
      const newStarred = await apiStarItem(id);
      setRecords((prev) =>
        prev.map((r) => (r.id === id ? { ...r, is_starred: newStarred } : r))
      );
    } catch (error) {
      console.error("Failed to star item:", error);
    }
  }, []);

  const handleDeleteItem = useCallback(async (id: string) => {
    try {
      await apiDeleteItem(id);
      setRecords((prev) => prev.filter((r) => r.id !== id));
    } catch (error) {
      console.error("Failed to delete item:", error);
    }
  }, []);

  const handleAddTags = useCallback(async (id: string, tags: string[]) => {
    try {
      await apiAddTags(id, tags);
    } catch (error) {
      console.error("Failed to add tags:", error);
    }
  }, []);

  useEffect(() => {
    if (autoRefresh) {
      loadHistory("", true);
    }
  }, [autoRefresh, loadHistory]);

  useEffect(() => {
    const unlisten = onClipboardUpdate((record) => {
      setRecords((prev) => [record, ...prev]);
    });

    return () => {
      unlisten.then((fn) => fn());
    };
  }, []);

  return {
    records,
    isLoading,
    hasMore,
    loadHistory,
    loadMore,
    handleStarItem,
    handleDeleteItem,
    handleAddTags,
  };
}
