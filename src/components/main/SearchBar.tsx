import { useEffect, useState, useCallback, useRef } from "react";
import { Search, X } from "lucide-react";
import { useI18n } from "../../i18n/I18nContext";
import { useClipboardStore } from "../../store/clipboardStore";
import { useDebounce } from "../../hooks";

export function SearchBar() {
  const { t } = useI18n();
  const [inputValue, setInputValue] = useState("");
  const { searchQuery, setSearchQuery, loadInitial } = useClipboardStore();
  const debouncedValue = useDebounce(inputValue, 300);
  const inputRef = useRef<HTMLInputElement>(null);

  const executeSearch = useCallback(() => {
    if (debouncedValue !== searchQuery) {
      setSearchQuery(debouncedValue);
      loadInitial();
    }
  }, [debouncedValue, searchQuery, setSearchQuery, loadInitial]);

  useEffect(() => {
    executeSearch();
  }, [executeSearch]);

  const handleClear = () => {
    setInputValue("");
    setSearchQuery("");
    loadInitial();
    inputRef.current?.focus();
  };

  return (
    <div className="sticky top-0 z-10 px-4 py-3 border-b border-border/50 bg-background/80 backdrop-blur-xl">
      <div className="relative flex items-center gap-2">
        <div className="relative flex-1">
          <Search className="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-muted-foreground" />
          <input
            ref={inputRef}
            type="text"
            value={inputValue}
            onChange={(e) => setInputValue(e.target.value)}
            placeholder={t("search.placeholder")}
            className="w-full pl-9 pr-10 py-2.5 text-sm bg-surface/50 border border-border rounded-lg outline-none focus:ring-2 focus:ring-accent/50 focus:border-accent transition-all placeholder:text-muted-foreground"
          />
          {inputValue && (
            <button
              onClick={handleClear}
              className="absolute right-3 top-1/2 -translate-y-1/2 p-0.5 rounded hover:bg-surface transition-colors"
            >
              <X className="w-3.5 h-3.5 text-muted-foreground" />
            </button>
          )}
        </div>
      </div>
    </div>
  );
}
