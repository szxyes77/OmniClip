import { useState, useCallback, useEffect } from "react";

interface UseKeyboardNavigationOptions {
  itemCount: number;
  onSelect: (index: number) => void;
  onClose?: () => void;
  loop?: boolean;
}

export function useKeyboardNavigation({
  itemCount,
  onSelect,
  onClose,
  loop = false,
}: UseKeyboardNavigationOptions) {
  const [selectedIndex, setSelectedIndex] = useState(0);

  useEffect(() => {
    setSelectedIndex(0);
  }, [itemCount]);

  const handleKeyDown = useCallback(
    (e: React.KeyboardEvent | KeyboardEvent) => {
      switch (e.key) {
        case "ArrowDown":
          e.preventDefault();
          setSelectedIndex((prev) => {
            const next = prev + 1;
            return loop ? (next >= itemCount ? 0 : next) : Math.min(next, itemCount - 1);
          });
          break;
        case "ArrowUp":
          e.preventDefault();
          setSelectedIndex((prev) => {
            const next = prev - 1;
            return loop ? (next < 0 ? itemCount - 1 : next) : Math.max(next, 0);
          });
          break;
        case "Enter":
          e.preventDefault();
          onSelect(selectedIndex);
          break;
        case "Escape":
          e.preventDefault();
          onClose?.();
          break;
      }
    },
    [itemCount, selectedIndex, onSelect, onClose, loop]
  );

  return { selectedIndex, setSelectedIndex, handleKeyDown };
}
