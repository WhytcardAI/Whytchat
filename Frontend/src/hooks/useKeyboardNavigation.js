import { useEffect } from "react";

/**
 * Custom hook for handling keyboard navigation in lists/menus.
 *
 * @param {Object} options
 * @param {boolean} options.isVisible - Whether the component is visible/active.
 * @param {number} options.itemCount - Total number of items in the list.
 * @param {number} options.selectedIndex - Current selected index.
 * @param {Function} options.setSelectedIndex - Function to update selected index.
 * @param {Function} options.onSelect - Callback when an item is selected (Enter/Tab).
 * @param {Function} options.onClose - Callback when closing (Escape).
 */
export function useKeyboardNavigation({
  isVisible,
  itemCount,
  selectedIndex,
  setSelectedIndex,
  onSelect,
  onClose,
}) {
  useEffect(() => {
    if (!isVisible) return;

    const handleKeyDown = (e) => {
      if (e.key === "ArrowDown") {
        e.preventDefault();
        setSelectedIndex((prev) => Math.min(prev + 1, itemCount - 1));
      } else if (e.key === "ArrowUp") {
        e.preventDefault();
        setSelectedIndex((prev) => Math.max(prev - 1, 0));
      } else if (e.key === "Enter" || e.key === "Tab") {
        e.preventDefault();
        onSelect();
      } else if (e.key === "Escape") {
        e.preventDefault();
        onClose();
      }
    };

    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [isVisible, itemCount, selectedIndex, setSelectedIndex, onSelect, onClose]);
}
