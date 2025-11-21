import { useEffect } from "react";

/**
 * Custom hook for handling keyboard shortcuts with cross-platform support.
 *
 * @param {string} key - The key to listen for (e.g., 'k', 'b', 'Escape'). Case-insensitive.
 * @param {Function} callback - The function to execute when the shortcut is triggered.
 * @param {Object} [options={}] - Configuration options for the hook.
 * @param {boolean} [options.ctrlKey=false] - Whether Ctrl (Windows/Linux) or Cmd (Mac) is required.
 * @param {boolean} [options.shiftKey=false] - Whether Shift is required.
 * @param {boolean} [options.altKey=false] - Whether Alt (Option on Mac) is required.
 * @param {boolean} [options.preventDefault=true] - Whether to prevent default browser behavior.
 *
 * @example
 * // Trigger on Ctrl+K (or Cmd+K)
 * useHotkeys('k', () => console.log('Search triggered'), { ctrlKey: true });
 */
export function useHotkeys(key, callback, options = {}) {
  const { ctrlKey = false, shiftKey = false, altKey = false, preventDefault = true } = options;

  useEffect(() => {
    // Safety check: ensure callback is a function
    if (typeof callback !== "function") {
      console.error(`useHotkeys: Expected callback to be a function, but got ${typeof callback}`);
      return;
    }

    const handleKeyDown = (event) => {
      // Check if the key matches (case-insensitive)
      if (event.key.toLowerCase() !== key.toLowerCase()) return;

      // Check modifiers
      // On Mac, metaKey is Cmd. On Windows/Linux, ctrlKey is Ctrl.
      // We treat ctrlKey option as "Ctrl OR Cmd" for cross-platform support.
      const isCtrlOrCmd = event.ctrlKey || event.metaKey;

      if (ctrlKey && !isCtrlOrCmd) return;
      // If we didn't ask for Ctrl, but it was pressed, we generally ignore it to avoid conflicts
      // unless we want loose matching. Here we enforce strict matching.
      if (!ctrlKey && isCtrlOrCmd) return;

      if (shiftKey !== event.shiftKey) return;
      if (altKey !== event.altKey) return;

      if (preventDefault) {
        event.preventDefault();
      }

      callback(event);
    };

    window.addEventListener("keydown", handleKeyDown);

    return () => {
      window.removeEventListener("keydown", handleKeyDown);
    };
  }, [key, callback, ctrlKey, shiftKey, altKey, preventDefault]);
}
