import { useState } from "react";
import { Star, Tag, Hash, Plus, Settings } from "lucide-react";
import { useClipboardStore } from "../../store/clipboardStore";
import { AddTagDialog } from "./AddTagDialog";
import { TagManagerDialog } from "./TagManagerDialog";
import { useI18n } from "../../i18n/I18nContext";

export function TagSidebar() {
  const { t } = useI18n();
  const { tags, clips, activeTagFilter, setActiveTagFilter, starredOnly, toggleStarredOnly } =
    useClipboardStore();

  const [showAddDialog, setShowAddDialog] = useState(false);
  const [showManagerDialog, setShowManagerDialog] = useState(false);

  const totalRecords = clips.length;
  const starredCount = clips.filter((c) => c.is_starred).length;

  return (
    <div className="w-56 border-l border-border bg-surface/30 flex flex-col">
      <div className="p-3 border-b border-border">
        <h3 className="text-xs font-semibold text-muted-foreground uppercase tracking-wider">
          {t("tagSidebar.filters")}
        </h3>
      </div>

      <div className="p-2 space-y-0.5">
        <button
          onClick={() => setActiveTagFilter(null)}
          className={`w-full flex items-center gap-2 px-2 py-1.5 rounded-md text-sm transition-colors ${
            !activeTagFilter && !starredOnly
              ? "bg-accent/10 text-accent"
              : "text-foreground hover:bg-surface/50"
          }`}
        >
          <Hash className="w-3.5 h-3.5" />
          <span className="flex-1 text-left">{t("tagSidebar.allRecords")}</span>
          <span className="text-xs text-muted-foreground">{totalRecords}</span>
        </button>

        <button
          onClick={toggleStarredOnly}
          className={`w-full flex items-center gap-2 px-2 py-1.5 rounded-md text-sm transition-colors ${
            starredOnly
              ? "bg-amber-500/10 text-amber-500"
              : "text-foreground hover:bg-surface/50"
          }`}
        >
          <Star className="w-3.5 h-3.5" />
          <span className="flex-1 text-left">{t("tagSidebar.starred")}</span>
          <span className="text-xs text-muted-foreground">{starredCount}</span>
        </button>
      </div>

      <div className="p-3 border-b border-border mt-2 flex items-center justify-between">
        <h3 className="text-xs font-semibold text-muted-foreground uppercase tracking-wider">
          {t("tagSidebar.tags")}
        </h3>
        <div className="flex items-center gap-1">
          <button
            onClick={() => setShowAddDialog(true)}
            className="p-1 hover:bg-surface/50 rounded-md transition-colors"
            title={t("tagSidebar.addTag")}
          >
            <Plus className="w-3.5 h-3.5 text-muted-foreground hover:text-foreground" />
          </button>
          <button
            onClick={() => setShowManagerDialog(true)}
            className="p-1 hover:bg-surface/50 rounded-md transition-colors"
            title={t("tagSidebar.manageTags")}
          >
            <Settings className="w-3.5 h-3.5 text-muted-foreground hover:text-foreground" />
          </button>
        </div>
      </div>

      <div className="flex-1 overflow-y-auto p-2 space-y-0.5">
        {tags.length === 0 ? (
          <p className="text-xs text-muted-foreground px-2 py-1">{t("tagSidebar.noTags")}</p>
        ) : (
          tags.map((tag) => (
            <button
              key={tag.id}
              onClick={() =>
                setActiveTagFilter(activeTagFilter === tag.id ? null : tag.id)
              }
              className={`w-full flex items-center gap-2 px-2 py-1.5 rounded-md text-sm transition-colors ${
                activeTagFilter === tag.id
                  ? "bg-accent/10 text-accent ring-1 ring-accent/20"
                  : "text-foreground hover:bg-surface/50"
              }`}
            >
              <Tag className="w-3.5 h-3.5" style={{ color: tag.color }} />
              <span className="flex-1 text-left truncate">{tag.name}</span>
              <span className="text-xs text-muted-foreground">
                {tag.record_count ?? 0}
              </span>
            </button>
          ))
        )}
      </div>

      <AddTagDialog
        isOpen={showAddDialog}
        onClose={() => setShowAddDialog(false)}
        existingTags={tags}
      />

      <TagManagerDialog
        isOpen={showManagerDialog}
        onClose={() => setShowManagerDialog(false)}
        tags={tags}
      />
    </div>
  );
}
