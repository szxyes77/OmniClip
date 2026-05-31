import { List } from "react-window";
import { RecordCard } from "./RecordCard";
import { useClipboardStore } from "../../store/clipboardStore";
import { useI18n } from "../../i18n/I18nContext";
import { useCallback, useRef, useState, useEffect } from "react";

export function RecordList() {
  const { t } = useI18n();
  const { clips, isLoading, loadMore } = useClipboardStore();
  const containerRef = useRef<HTMLDivElement>(null);
  const [listHeight, setListHeight] = useState(600);

  useEffect(() => {
    const updateHeight = () => {
      if (containerRef.current) {
        setListHeight(containerRef.current.clientHeight);
      }
    };
    updateHeight();
    window.addEventListener("resize", updateHeight);
    return () => window.removeEventListener("resize", updateHeight);
  }, []);

  const handleItemsRendered = useCallback(
    ({ visibleStopIndex }: { visibleStopIndex: number }) => {
      if (visibleStopIndex >= clips.length - 5) {
        loadMore();
      }
    },
    [clips.length, loadMore]
  );

  const Row = useCallback(
    ({ index, style }: { index: number; style: React.CSSProperties }) => {
      const record = clips[index];
      if (!record) return null;
      return (
        <div style={style}>
          <RecordCard record={record} />
        </div>
      );
    },
    [clips]
  );

  if (clips.length === 0) {
    if (isLoading) {
      return (
        <div className="flex flex-col items-center justify-center py-16 text-muted-foreground">
          <div className="w-6 h-6 border-2 border-accent/30 border-t-accent rounded-full animate-spin" />
          <p className="text-sm mt-3">{t("recordList.loading")}</p>
        </div>
      );
    }
    return (
      <div className="flex flex-col items-center justify-center py-16 text-muted-foreground">
        <p className="text-sm">{t("recordList.noRecords")}</p>
        <p className="text-xs mt-1">{t("recordList.noRecordsHint")}</p>
      </div>
    );
  }

  const VirtualList = List as unknown as React.ComponentType<{
    height: number;
    width: string | number;
    itemCount: number;
    itemSize: number;
    onItemsRendered: (props: { visibleStopIndex: number }) => void;
    children: React.ComponentType<{ index: number; style: React.CSSProperties }>;
  }>;

  return (
    <div className="flex-1 min-h-0" ref={containerRef}>
      <VirtualList
        height={listHeight}
        width="100%"
        itemCount={clips.length}
        itemSize={80}
        onItemsRendered={handleItemsRendered}
        children={Row}
      />
      {isLoading && clips.length > 0 && (
        <div className="flex items-center justify-center py-4">
          <div className="w-4 h-4 border-2 border-accent/30 border-t-accent rounded-full animate-spin" />
        </div>
      )}
    </div>
  );
}
