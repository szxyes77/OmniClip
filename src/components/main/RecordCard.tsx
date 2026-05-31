import { useState, useCallback } from "react";
import { FileText, Image, Folder, Star, Trash2, Copy, Tag, X } from "lucide-react";
import type { ClipboardRecord } from "../../types";
import { formatRelativeTime } from "../../utils/format";
import { useClipboardStore } from "../../store/clipboardStore";
import { TagBadge } from "./TagBadge";
import { useI18n } from "../../i18n/I18nContext";

const MAX_SUMMARY = 60;

function getTypeIcon(type: string) {
  switch (type) {
    case "text":
      return <FileText className="w-4 h-4 text-blue-500" />;
    case "image":
      return <Image className="w-4 h-4 text-green-500" />;
    case "files":
      return <Folder className="w-4 h-4 text-amber-500" />;
    default:
      return <FileText className="w-4 h-4 text-gray-500" />;
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
  return record.content.length > MAX_SUMMARY
    ? record.content.slice(0, MAX_SUMMARY) + "..."
    : record.content;
}

interface ContextMenuState {
  visible: boolean;
  x: number;
  y: number;
  record: ClipboardRecord | null;
}

export function RecordCard({ record }: { record: ClipboardRecord }) {
  const { t } = useI18n();
  const { handleStar, handleDelete, handlePaste, handleAddTags } = useClipboardStore();
  const [contextMenu, setContextMenu] = useState<ContextMenuState>({
    visible: false,
    x: 0,
    y: 0,
    record: null,
  });
  const [showTagInput, setShowTagInput] = useState(false);
  const [tagInput, setTagInput] = useState("");

  const handleContextMenu = useCallback(
    (e: React.MouseEvent, record: ClipboardRecord) => {
      e.preventDefault();
      e.stopPropagation();
      setContextMenu({ visible: true, x: e.clientX, y: e.clientY, record });
    },
    []
  );

  const closeContextMenu = useCallback(() => {
    setContextMenu({ visible: false, x: 0, y: 0, record: null });
  }, []);

  const handleCopyPlain = useCallback(async () => {
    if (!contextMenu.record) return;
    try {
      await navigator.clipboard.writeText(contextMenu.record.content);
      closeContextMenu();
    } catch (error) {
      console.error("Failed to copy:", error);
    }
  }, [contextMenu.record, closeContextMenu]);

  const handleDeleteFromMenu = useCallback(async () => {
    if (!contextMenu.record) return;
    await handleDelete(contextMenu.record.id);
    closeContextMenu();
  }, [contextMenu.record, handleDelete, closeContextMenu]);

  const handleAddTag = useCallback(async () => {
    if (!contextMenu.record || !tagInput.trim()) return;
    const tags = tagInput.split(",").map((t) => t.trim()).filter(Boolean);
    await handleAddTags(contextMenu.record.id, tags);
    setTagInput("");
    setShowTagInput(false);
    closeContextMenu();
  }, [contextMenu.record, tagInput, handleAddTags, closeContextMenu]);

  return (
    <div
      onContextMenu={(e) => handleContextMenu(e, record)}
      onClick={() => handlePaste(record.id)}
      className="group flex items-start gap-3 p-3 rounded-lg border border-transparent hover:border-border hover:bg-surface/50 cursor-pointer transition-all"
    >
      <div className="flex-shrink-0 mt-0.5">{getTypeIcon(record.record_type)}</div>

      <div className="flex-1 min-w-0">
        <div className="flex items-start justify-between gap-2">
          <p className="text-sm text-foreground leading-relaxed break-words">
            {getSummary(record)}
          </p>
          <button
            onClick={(e) => {
              e.stopPropagation();
              handleStar(record.id);
            }}
            className="flex-shrink-0 p-1 rounded hover:bg-surface transition-colors"
          >
            <Star
              className={`w-4 h-4 ${
                record.is_starred
                  ? "fill-amber-500 text-amber-500"
                  : "text-muted-foreground opacity-0 group-hover:opacity-100"
              }`}
            />
          </button>
        </div>

        {record.tags && record.tags.length > 0 && (
          <div className="flex items-center gap-1 mt-1.5 flex-wrap">
            {record.tags.map((tag) => (
              <TagBadge key={tag.id} name={tag.name} color={tag.color} />
            ))}
          </div>
        )}

        <div className="flex items-center gap-2 mt-1.5">
          <span className="text-xs text-muted-foreground">
            {formatRelativeTime(record.created_at)}
          </span>
          {record.copy_count > 1 && (
            <span className="text-xs text-muted-foreground">
              {t("recordList.copied", { count: record.copy_count })}
            </span>
          )}
        </div>
      </div>

      {contextMenu.visible && contextMenu.record?.id === record.id && (
        <div
          className="fixed z-50 min-w-[180px] py-1 bg-popover border border-border rounded-lg shadow-lg"
          style={{ left: contextMenu.x, top: contextMenu.y }}
        >
          <button
            onClick={handleCopyPlain}
            className="w-full flex items-center gap-2 px-3 py-2 text-sm text-popover-foreground hover:bg-accent transition-colors"
          >
            <Copy className="w-3.5 h-3.5" />
            <span>{t("recordList.copyPlain")}</span>
          </button>
          <button
            onClick={() => {
              setShowTagInput(!showTagInput);
            }}
            className="w-full flex items-center gap-2 px-3 py-2 text-sm text-popover-foreground hover:bg-accent transition-colors"
          >
            <Tag className="w-3.5 h-3.5" />
            <span>{t("recordList.editTags")}</span>
          </button>
          <div className="my-1 border-t border-border" />
          <button
            onClick={handleDeleteFromMenu}
            className="w-full flex items-center gap-2 px-3 py-2 text-sm text-red-500 hover:bg-red-500/10 transition-colors"
          >
            <Trash2 className="w-3.5 h-3.5" />
            <span>{t("recordList.delete")}</span>
          </button>

          {showTagInput && (
            <div className="px-3 py-2 border-t border-border">
              <div className="flex items-center gap-1">
                <input
                  type="text"
                  value={tagInput}
                  onChange={(e) => setTagInput(e.target.value)}
                  onKeyDown={(e) => e.key === "Enter" && handleAddTag()}
                  placeholder={t("recordList.tagInputPlaceholder")}
                  className="flex-1 text-xs px-2 py-1 bg-background border border-border rounded outline-none focus:ring-1 focus:ring-accent"
                  autoFocus
                />
                <button onClick={() => setShowTagInput(false)} className="p-1 hover:bg-surface rounded">
                  <X className="w-3 h-3 text-muted-foreground" />
                </button>
              </div>
            </div>
          )}
        </div>
      )}
    </div>
  );
}
