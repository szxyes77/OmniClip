import { useState, useRef, useEffect } from "react";
import { X, Plus } from "lucide-react";
import type { Tag } from "../../types";
import { createTag } from "../../api/tags";
import { useClipboardStore } from "../../store/clipboardStore";
import { useI18n } from "../../i18n/I18nContext";

const SOFT_COLORS = [
  "#F472B6", "#A78BFA", "#60A5FA", "#34D399",
  "#FBBF24", "#FB923C", "#F87171", "#818CF8",
  "#38BDF8", "#2DD4BF", "#C084FC", "#E879F9",
];

interface AddTagDialogProps {
  isOpen: boolean;
  onClose: () => void;
  existingTags: Tag[];
}

export function AddTagDialog({ isOpen, onClose, existingTags }: AddTagDialogProps) {
  const { t } = useI18n();
  const [inputValue, setInputValue] = useState("");
  const [showAutocomplete, setShowAutocomplete] = useState(false);
  const [isCreating, setIsCreating] = useState(false);
  const inputRef = useRef<HTMLInputElement>(null);
  const containerRef = useRef<HTMLDivElement>(null);
  const refreshTags = useClipboardStore((state) => state.refreshTags);

  useEffect(() => {
    if (isOpen && inputRef.current) {
      inputRef.current.focus();
    }
  }, [isOpen]);

  useEffect(() => {
    const handleClickOutside = (e: MouseEvent) => {
      if (containerRef.current && !containerRef.current.contains(e.target as Node)) {
        onClose();
      }
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
        onClose();
      }
      if (e.key === "Enter" && inputValue.trim()) {
        handleCreate();
      }
    };
    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [isOpen, inputValue, onClose]);

  const filteredTags = existingTags.filter(
    (tag) =>
      tag.name.toLowerCase().includes(inputValue.toLowerCase()) &&
      inputValue.length > 0
  );

  const getRandomColor = () => {
    return SOFT_COLORS[Math.floor(Math.random() * SOFT_COLORS.length)];
  };

  const handleCreate = async () => {
    if (!inputValue.trim() || isCreating) return;
    const exists = existingTags.find(
      (t) => t.name.toLowerCase() === inputValue.trim().toLowerCase()
    );
    if (exists) {
      setInputValue("");
      setShowAutocomplete(false);
      onClose();
      return;
    }

    setIsCreating(true);
    try {
      await createTag(inputValue.trim(), getRandomColor());
      await refreshTags();
      setInputValue("");
      setShowAutocomplete(false);
      onClose();
    } catch (error) {
      console.error("Failed to create tag:", error);
    } finally {
      setIsCreating(false);
    }
  };

  const handleSelectExisting = (tag: Tag) => {
    setInputValue(tag.name);
    setShowAutocomplete(false);
  };

  if (!isOpen) return null;

  return (
    <div className="fixed inset-0 bg-black/40 backdrop-blur-sm z-50 flex items-center justify-center">
      <div
        ref={containerRef}
        className="bg-surface border border-border rounded-xl shadow-2xl w-96 overflow-hidden"
      >
        <div className="flex items-center justify-between px-4 py-3 border-b border-border">
          <h3 className="text-sm font-semibold text-foreground">{t("addTag.title")}</h3>
          <button
            onClick={onClose}
            className="p-1 hover:bg-surface/50 rounded-md transition-colors"
          >
            <X className="w-4 h-4 text-muted-foreground" />
          </button>
        </div>

        <div className="p-4 space-y-3">
          <div className="relative">
            <input
              ref={inputRef}
              type="text"
              value={inputValue}
              onChange={(e) => {
                setInputValue(e.target.value);
                setShowAutocomplete(true);
              }}
              onFocus={() => setShowAutocomplete(true)}
              onBlur={() => setTimeout(() => setShowAutocomplete(false), 200)}
              placeholder={t("addTag.placeholder")}
              className="w-full px-3 py-2 bg-surface/50 border border-border rounded-md text-sm text-foreground placeholder:text-muted-foreground/60 focus:outline-none focus:ring-2 focus:ring-accent/50 focus:border-transparent"
            />

            {showAutocomplete && filteredTags.length > 0 && (
              <div className="absolute z-10 mt-1 w-full bg-surface border border-border rounded-md shadow-lg max-h-48 overflow-y-auto">
                {filteredTags.map((tag) => (
                  <button
                    key={tag.id}
                    onMouseDown={() => handleSelectExisting(tag)}
                    className="w-full flex items-center gap-2 px-3 py-2 hover:bg-surface/50 transition-colors text-left"
                  >
                    <span
                      className="w-3 h-3 rounded-full"
                      style={{ backgroundColor: tag.color }}
                    />
                    <span className="text-sm text-foreground">{tag.name}</span>
                  </button>
                ))}
              </div>
            )}
          </div>

          <button
            onClick={handleCreate}
            disabled={!inputValue.trim() || isCreating}
            className="w-full flex items-center justify-center gap-2 px-4 py-2 bg-accent/10 hover:bg-accent/20 disabled:opacity-50 disabled:cursor-not-allowed rounded-md text-sm font-medium text-accent transition-colors"
          >
            <Plus className="w-4 h-4" />
            {isCreating ? t("addTag.creating") : t("addTag.create")}
          </button>
        </div>
      </div>
    </div>
  );
}
