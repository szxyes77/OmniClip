import { useEffect, useState, useRef, useCallback } from "react";
import { Search, X } from "lucide-react";
import type { ClipboardRecord } from "../../types";
import { invoke } from "@tauri-apps/api/core";
import { formatRelativeTime } from "../../utils/format";
import { useKeyboardNavigation, useClickOutside } from "../../hooks";

export function FloatingWindow() {
  const [records, setRecords] = useState<ClipboardRecord[]>([]);
  const [query, setQuery] = useState("");
  const [isLoading, setIsLoading] = useState(false);
  const inputRef = useRef<HTMLInputElement>(null);

  const containerRef = useClickOutside<HTMLDivElement>(async () => {
    try {
      await invoke<void>("hide_floating_window");
    } catch (error) {
      console.error("Failed to hide window:", error);
    }
  });

  const handlePaste = useCallback(
    async (record: ClipboardRecord) => {
      try {
        await invoke<void>("paste_record", { id: record.id });
      } catch (error) {
        console.error("Failed to paste:", error);
      }
    },
    []
  );

  const hideWindow = useCallback(async () => {
    try {
      await invoke<void>("hide_floating_window");
    } catch (error) {
      console.error("Failed to hide window:", error);
    }
  }, []);

  const filteredRecords = query
    ? records.filter(
        (r) =>
          r.title.toLowerCase().includes(query.toLowerCase()) ||
          r.content.toLowerCase().includes(query.toLowerCase())
      )
    : records;

  const { selectedIndex, setSelectedIndex, handleKeyDown } = useKeyboardNavigation({
    itemCount: filteredRecords.length,
    onSelect: (index) => {
      if (filteredRecords[index]) {
        handlePaste(filteredRecords[index]);
      }
    },
    onClose: hideWindow,
  });

  useEffect(() => {
    loadRecent();
    inputRef.current?.focus();
  }, []);

  const loadRecent = async () => {
    setIsLoading(true);
    try {
      const data = await invoke<ClipboardRecord[]>("get_clipboard_records", {
        limit: 20,
        offset: 0,
      });
      setRecords(data);
    } catch (error) {
      console.error("Failed to load records:", error);
    } finally {
      setIsLoading(false);
    }
  };

  return (
    <div
      ref={containerRef}
      className="w-full h-full glass-dark rounded-xl overflow-hidden animate-scale-in"
      onKeyDown={handleKeyDown}
    >
      <div className="px-3 py-2.5 border-b border-white/10">
        <div className="flex items-center gap-2 px-3 py-1.5 bg-white/10 rounded-lg">
          <Search className="w-4 h-4 text-gray-400" />
          <input
            ref={inputRef}
            type="text"
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            placeholder="Search..."
            className="flex-1 bg-transparent outline-none text-white text-sm placeholder-gray-500"
          />
          {query && (
            <button onClick={() => setQuery("")} className="text-gray-400 hover:text-white">
              <X className="w-3.5 h-3.5" />
            </button>
          )}
        </div>
      </div>

      <div className="overflow-y-auto p-2 space-y-1" style={{ maxHeight: "calc(100% - 56px)" }}>
        {isLoading ? (
          <div className="flex justify-center py-8">
            <div className="w-5 h-5 border-2 border-white/30 border-t-white rounded-full animate-spin" />
          </div>
        ) : filteredRecords.length === 0 ? (
          <div className="flex flex-col items-center justify-center py-8 text-gray-500 text-sm">
            <p>No records found</p>
          </div>
        ) : (
          filteredRecords.map((record, index) => (
            <button
              key={record.id}
              onClick={() => handlePaste(record)}
              onMouseEnter={() => setSelectedIndex(index)}
              className={`w-full text-left px-3 py-2.5 rounded-lg transition-colors ${
                index === selectedIndex
                  ? "bg-indigo-500/30 text-white"
                  : "text-gray-300 hover:bg-white/5"
              }`}
            >
              <div className="flex items-center gap-2">
                <span
                  className={`px-1.5 py-0.5 text-[10px] rounded ${
                    record.record_type === "text"
                      ? "bg-blue-500/20 text-blue-300"
                      : record.record_type === "image"
                      ? "bg-green-500/20 text-green-300"
                      : "bg-amber-500/20 text-amber-300"
                  }`}
                >
                  {record.record_type}
                </span>
                <span className="flex-1 text-sm truncate">{record.title}</span>
                <span className="text-[10px] text-gray-500">
                  {formatRelativeTime(record.created_at)}
                </span>
              </div>
            </button>
          ))
        )}
      </div>

      <div className="px-3 py-1.5 border-t border-white/10 flex items-center justify-between text-[10px] text-gray-500">
        <span>{filteredRecords.length} items</span>
        <div className="flex gap-2">
          <span>Enter to paste</span>
          <span>Esc to close</span>
        </div>
      </div>
    </div>
  );
}
