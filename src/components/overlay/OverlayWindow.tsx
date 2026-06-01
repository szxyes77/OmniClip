import { useEffect, useState, useCallback, useRef } from "react";
import { FileText, Image, Folder } from "lucide-react";
import type { ClipboardRecord } from "../../types";
import { invoke } from "@tauri-apps/api/core";
import { listen, UnlistenFn } from "@tauri-apps/api/event";
import { formatRelativeTime } from "../../utils/format";

const MAX_ITEMS = 10;

function getTypeIcon(type: string) {
  switch (type) {
    case "text":
      return <FileText className="w-3.5 h-3.5 text-blue-400" />;
    case "image":
      return <Image className="w-3.5 h-3.5 text-green-400" />;
    case "files":
      return <Folder className="w-3.5 h-3.5 text-amber-400" />;
    default:
      return <FileText className="w-3.5 h-3.5 text-gray-400" />;
  }
}

function getSummary(record: ClipboardRecord): string {
  if (record.record_type === "files") {
    try {
      const paths = JSON.parse(record.content) as string[];
      return paths.join(", ");
    } catch {
      return record.content;
    }
  }
  if (record.record_type === "image") {
    return record.title;
  }
  return record.content.length > 50
    ? record.content.slice(0, 50) + "..."
    : record.content;
}

export function OverlayWindow() {
  const [records, setRecords] = useState<ClipboardRecord[]>([]);
  const [selectedIndex, setSelectedIndex] = useState(0);
  const [isInteractive, setIsInteractive] = useState(false);
  const isUnmounted = useRef(false);

  const loadRecords = useCallback(async () => {
    try {
      const data = await invoke<ClipboardRecord[]>("get_clipboard_records", {
        limit: MAX_ITEMS,
        offset: 0,
      });
      if (!isUnmounted.current) {
        setRecords(data);
        setSelectedIndex(0);
      }
    } catch (error) {
      console.error("Failed to load records:", error);
    }
  }, []);

  const handlePaste = useCallback(async (record: ClipboardRecord) => {
    try {
      await invoke("set_overlay_passthrough", { enable: false });
      await invoke("paste_from_overlay", { id: record.id });
    } catch (error) {
      console.error("Failed to paste:", error);
    }
  }, []);

  const hideWindow = useCallback(async () => {
    try {
      await invoke("hide_overlay_window");
    } catch (error) {
      console.error("Failed to hide:", error);
    }
  }, []);

  const handleKeyDown = useCallback(
    (e: React.KeyboardEvent | KeyboardEvent) => {
      if (!isInteractive) return;

      switch (e.key) {
        case "ArrowDown":
          e.preventDefault();
          setSelectedIndex((prev) => Math.min(prev + 1, records.length - 1));
          break;
        case "ArrowUp":
          e.preventDefault();
          setSelectedIndex((prev) => Math.max(prev - 1, 0));
          break;
        case "Enter":
          e.preventDefault();
          if (records[selectedIndex]) {
            handlePaste(records[selectedIndex]);
          }
          break;
        case "Escape":
          e.preventDefault();
          hideWindow();
          break;
      }
    },
    [isInteractive, records.length, selectedIndex, handlePaste, hideWindow]
  );

  const handleWheel = useCallback(
    (e: React.WheelEvent) => {
      if (!isInteractive) return;
      if (e.deltaY > 0) {
        setSelectedIndex((prev) => Math.min(prev + 1, records.length - 1));
      } else {
        setSelectedIndex((prev) => Math.max(prev - 1, 0));
      }
    },
    [isInteractive, records.length]
  );

  useEffect(() => {
    loadRecords();

    let unlistenUpdate: UnlistenFn | null = null;
    listen<ClipboardRecord>("clipboard-update", () => {
      loadRecords();
    }).then((fn) => {
      unlistenUpdate = fn;
    });

    const handleGlobalKeyDown = (e: KeyboardEvent) => {
      if (!isInteractive) {
        if (e.key === "Escape") {
          hideWindow();
        }
      }
    };

    window.addEventListener("keydown", handleGlobalKeyDown);

    return () => {
      isUnmounted.current = true;
      unlistenUpdate?.();
      window.removeEventListener("keydown", handleGlobalKeyDown);
    };
  }, [loadRecords, hideWindow, isInteractive]);

  useEffect(() => {
    const toggleInteractive = async (interactive: boolean) => {
      setIsInteractive(interactive);
      await invoke("set_overlay_passthrough", { enable: !interactive });
    };

    const handleMouseEnter = () => toggleInteractive(true);
    const handleMouseLeave = () => {
      toggleInteractive(false);
      if (!isInteractive) {
        hideWindow();
      }
    };

    const overlayEl = document.getElementById("overlay-root");
    if (overlayEl) {
      overlayEl.addEventListener("mouseenter", handleMouseEnter);
      overlayEl.addEventListener("mouseleave", handleMouseLeave);
    }

    return () => {
      if (overlayEl) {
        overlayEl.removeEventListener("mouseenter", handleMouseEnter);
        overlayEl.removeEventListener("mouseleave", handleMouseLeave);
      }
    };
  }, [hideWindow, isInteractive]);

  return (
    <div
      id="overlay-container"
      className="w-full h-full rounded-xl overflow-hidden animate-scale-in"
      style={{
        background: "rgba(15, 23, 42, 0.85)",
        backdropFilter: "blur(20px) saturate(180%)",
        WebkitBackdropFilter: "blur(20px) saturate(180%)",
        border: "1px solid rgba(255, 255, 255, 0.1)",
        boxShadow: "0 8px 32px rgba(0, 0, 0, 0.4)",
      }}
      onKeyDown={handleKeyDown}
      onWheel={handleWheel}
      tabIndex={0}
    >
      <div className="px-3 py-2 border-b border-white/10 cursor-move" data-tauri-drag-region>
        <div className="flex items-center justify-between">
          <span className="text-xs font-medium text-gray-300">Quick Paste</span>
          <span className="text-[10px] text-gray-500">
            {records.length} items &middot; Enter to paste &middot; Esc to close
          </span>
        </div>
      </div>

      <div className="overflow-y-auto p-1.5 space-y-0.5" style={{ maxHeight: "calc(100% - 40px)" }}>
        {records.length === 0 ? (
          <div className="flex flex-col items-center justify-center py-8 text-gray-500 text-xs">
            <p>No clipboard history</p>
          </div>
        ) : (
          records.map((record, index) => (
            <button
              key={record.id}
              onClick={() => handlePaste(record)}
              className={`w-full text-left px-2.5 py-2 rounded-lg transition-all ${
                index === selectedIndex
                  ? "bg-indigo-500/40 text-white shadow-inner"
                  : "text-gray-300 hover:bg-white/5"
              }`}
            >
              <div className="flex items-center gap-2">
                <div className="flex-shrink-0">{getTypeIcon(record.record_type)}</div>
                <span className="flex-1 text-xs truncate">{getSummary(record)}</span>
                <span className="text-[10px] text-gray-500 flex-shrink-0">
                  {formatRelativeTime(record.created_at)}
                </span>
              </div>
            </button>
          ))
        )}
      </div>
    </div>
  );
}
