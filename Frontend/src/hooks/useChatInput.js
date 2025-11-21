import { useState, useRef, useCallback } from "react";
import { open } from "@tauri-apps/plugin-dialog";
import { readTextFile } from "@tauri-apps/plugin-fs";
import { useTranslation } from "react-i18next";

/**
 * Hook to manage chat input state, file attachments, and template selection.
 *
 * @param {Function} onSend - Callback when a message is sent.
 * @param {boolean} isGenerating - Whether the AI is currently generating a response.
 */
export function useChatInput(onSend, isGenerating) {
  const { t } = useTranslation();
  const [inputValue, setInputValue] = useState("");
  const [showTemplates, setShowTemplates] = useState(false);
  const [templateFilter, setTemplateFilter] = useState("");
  const inputRef = useRef(null);

  const handleInputChange = useCallback((e) => {
    const value = e.target.value;
    const selectionStart = e.target.selectionStart;

    setInputValue(value);

    // Detect "/" trigger
    const lastSlashIndex = value.lastIndexOf("/", selectionStart - 1);

    if (lastSlashIndex !== -1) {
      const isStartOfLine = lastSlashIndex === 0 || value[lastSlashIndex - 1] === "\n";
      const isPrecededBySpace = value[lastSlashIndex - 1] === " ";

      if (isStartOfLine || isPrecededBySpace) {
        const textAfterSlash = value.substring(lastSlashIndex + 1, selectionStart);
        if (!textAfterSlash.includes(" ")) {
          setShowTemplates(true);
          setTemplateFilter(textAfterSlash);
          return;
        }
      }
    }

    setShowTemplates(false);
  }, []);

  const handleTemplateSelect = useCallback((template) => {
    const newValue = template.content;
    setInputValue(newValue);
    setShowTemplates(false);

    // Focus back and set cursor at the end or select placeholder
    // Using requestAnimationFrame for better timing than setTimeout
    requestAnimationFrame(() => {
      if (inputRef.current) {
        inputRef.current.focus();
        const placeholderIndex = newValue.indexOf("[");
        if (placeholderIndex !== -1) {
          const placeholderEnd = newValue.indexOf("]", placeholderIndex);
          if (placeholderEnd !== -1) {
            inputRef.current.setSelectionRange(placeholderIndex, placeholderEnd + 1);
            return;
          }
        }
      }
    });
  }, []);

  const handleSend = useCallback(
    (e) => {
      e?.preventDefault();
      const text = inputValue.trim();
      if (!text || isGenerating) return;

      onSend(text);
      setInputValue("");
      setShowTemplates(false);
    },
    [inputValue, isGenerating, onSend]
  );

  const handleAttachFile = useCallback(async () => {
    try {
      const selectedPath = await open({ multiple: false });
      if (typeof selectedPath === "string") {
        const content = await readTextFile(selectedPath);
        setInputValue(
          (prev) =>
            prev + `\n\n[${t("chat.input.file_attached")} ${selectedPath}]\n${content}\n[/Fichier]`
        );
      }
    } catch (error) {
      console.error("File error:", error);
    }
  }, [t]);

  return {
    inputValue,
    setInputValue,
    showTemplates,
    setShowTemplates,
    templateFilter,
    inputRef,
    handleInputChange,
    handleTemplateSelect,
    handleSend,
    handleAttachFile,
  };
}
