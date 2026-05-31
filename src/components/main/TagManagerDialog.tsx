import { useState, useRef, useEffect } from "react";
import { X, Edit2, Trash2, Check } from "lucide-react";
import type { Tag } from "../../types";
import { useClipboardStore } from "../../store/clipboardStore";
import { useI18n } from "../../i18n/I18nContext";

interface TagManagerDialogProps {
  isOpen: boolean;
  onClose: () => void;
  tags: Tag[];
}

export function TagManagerDialog({ isOpen, onClose, tags }: TagManagerDialogProps) {
  const { t } = useI18n();
  const [editingTagId, setEditingTagId] = useState<string | null>(null);
  const [editingName, setEditingName] = useState("");
  const [deletingTagId, setDeletingTagId] = useState<string | null>(null);
  const inputRef = useRef<HTMLInputElement>(null);
  const { renameTag, deleteTag, refreshTags } = useClipboardStore();

  useEffect(() => {
    if (editingTagId && inputRef.current) {
      inputRef.current.focus();
      inputRef.current.select();
    }
  }, [editingTagId]);

  useEffect(() => {
    const handleClickOutside = (e: MouseEvent) => {
      const target = e.target as HTMLElement;
      if (target.closest("[data-dialog]")) return;
      onClose();
    };
    if (isOpen) {
      document.addEventListener("mousedown", handleClickOutside);
    }
    return () => document.removeEventListener("mousedown", handleClickOutside);
  }, [isOpen, onClose]);

  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (!isOpen) return;
      if (e.key === "Escape") {
        if (editingTagId) {
          setEditingTagId(null);
        } else {
          onClose();
        }
      }
    };
    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [isOpen, editingTagId, onClose]);

  const handleStartEdit = (tag: Tag) => {
    setEditingTagId(tag.id);
    setEditingName(tag.name);
  };

  const handleSaveRename = async (tagId: string) => {
    if (!editingName.trim()) return;
    try {
      await renameTag(tagId, editingName.trim());
      await refreshTags();
      setEditingTagId(null);
    } catch (error) {
      console.error("Failed to rename tag:", error);
    }
  };

  const handleDelete = async (tagId: string) => {
    try {
      await deleteTag(tagId);
      await refreshTags();
      setDeletingTagId(null);
    } catch (error) {
      console.error("Failed to delete tag:", error);
    }
  };

  if (!isOpen) return null;

  return (
    <div className="fixed inset-0 bg-black/40 backdrop-blur-sm z-50 flex items-center justify-center">
      <div
        data-dialog
        className="bg-surface border border-border rounded-xl shadow-2xl w-[420px] max-h-[500px] flex flex-col overflow-hidden"
      >
        <div className="flex items-center justify-between px-4 py-3 border-b border-border">
          <h3 className="text-sm font-semibold text-foreground">{t("manageTags.title")}</h3>
          <button
            onClick={onClose}
            className="p-1 hover:bg-surface/50 rounded-md transition-colors"
          >
            <X className="w-4 h-4 text-muted-foreground" />
          </button>
        </div>

        <div className="flex-1 overflow-y-auto p-3 space-y-1">
          {tags.length === 0 ? (
            <p className="text-center text-xs text-muted-foreground py-8">
              {t("manageTags.noTags")} {t("manageTags.noTagsHint")}
            </p>
          ) : (
            tags.map((tag) => (
              <div
                key={tag.id}
                className="flex items-center gap-2 px-3 py-2 rounded-md hover:bg-surface/50 transition-colors group"
              >
                <span
                  className="w-4 h-4 rounded-full flex-shrink-0"
                  style={{ backgroundColor: tag.color }}
                />

                {editingTagId === tag.id ? (
                  <div className="flex-1 flex items-center gap-1">
                    <input
                      ref={inputRef}
                      type="text"
                      value={editingName}
                      onChange={(e) => setEditingName(e.target.value)}
                      onKeyDown={(e) => {
                        if (e.key === "Enter") handleSaveRename(tag.id);
                        if (e.key === "Escape") setEditingTagId(null);
                      }}
                      className="flex-1 px-2 py-0.5 bg-surface/80 border border-border rounded text-sm text-foreground focus:outline-none focus:ring-1 focus:ring-accent/50"
                    />
                    <button
                      onClick={() => handleSaveRename(tag.id)}
                      className="p-1 text-green-500 hover:text-green-400"
                    >
                      <Check className="w-3.5 h-3.5" />
                    </button>
                  </div>
                ) : (
                  <>
                    <span className="flex-1 text-sm text-foreground truncate">
                      {tag.name}
                    </span>
                    <div className="flex items-center gap-0.5 opacity-0 group-hover:opacity-100 transition-opacity">
                      <button
                        onClick={() => handleStartEdit(tag)}
                        className="p-1 hover:bg-surface/80 rounded transition-colors"
                      >
                        <Edit2 className="w-3.5 h-3.5 text-muted-foreground hover:text-foreground" />
                      </button>
                      <button
                        onClick={() =>
                          setDeletingTagId(
                            deletingTagId === tag.id ? null : tag.id
                          )
                        }
                        className="p-1 hover:bg-surface/80 rounded transition-colors"
                      >
                        <Trash2 className="w-3.5 h-3.5 text-muted-foreground hover:text-red-400" />
                      </button>
                    </div>
                  </>
                )}

                {deletingTagId === tag.id && editingTagId !== tag.id && (
                  <div className="absolute right-4 top-1/2 -translate-y-1/2 bg-surface border border-border rounded-md shadow-lg px-3 py-2 flex items-center gap-2 z-10">
                    <span className="text-xs text-foreground">{t("manageTags.deleteConfirm")}</span>
                    <button
                      onClick={() => handleDelete(tag.id)}
                      className="px-2 py-0.5 bg-red-500/20 hover:bg-red-500/30 text-red-400 rounded text-xs transition-colors"
                    >
                      {t("manageTags.yes")}
                    </button>
                    <button
                      onClick={() => setDeletingTagId(null)}
                      className="px-2 py-0.5 bg-surface/50 hover:bg-surface/80 text-muted-foreground rounded text-xs transition-colors"
                    >
                      {t("manageTags.no")}
                    </button>
                  </div>
                )}
              </div>
            ))
          )}
        </div>
      </div>
    </div>
  );
}
