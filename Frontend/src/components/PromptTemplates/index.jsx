import { useState, useEffect, useRef, useMemo, memo } from "react";
import PropTypes from "prop-types";
import { useTranslation } from "react-i18next";
import { PROMPT_TEMPLATES } from "../../lib/templates";
import { Command, Sparkles, Code, PenTool, MessageSquare } from "lucide-react";
import { useKeyboardNavigation } from "../../hooks/useKeyboardNavigation";

const CATEGORY_ICONS = {
  coding: <Code className="w-4 h-4 text-blue-400" aria-hidden="true" />,
  writing: <PenTool className="w-4 h-4 text-purple-400" aria-hidden="true" />,
  discussion: <MessageSquare className="w-4 h-4 text-green-400" aria-hidden="true" />,
  default: <Sparkles className="w-4 h-4 text-yellow-400" aria-hidden="true" />,
};

/**
 * Component displaying a list of prompt templates for quick selection.
 *
 * @component
 * @param {Object} props
 * @param {boolean} props.isVisible - Whether the menu is visible.
 * @param {Function} props.onSelect - Callback when a template is selected.
 * @param {Function} props.onClose - Callback to close the menu.
 * @param {string} [props.filterText=''] - Text to filter templates.
 * @param {Object} [props.position={ bottom: '100%', left: 0 }] - CSS position styles.
 */
const PromptTemplates = memo(function PromptTemplates({
  isVisible,
  onSelect,
  onClose,
  filterText = "",
  position = { bottom: "100%", left: 0 },
}) {
  const { t } = useTranslation();
  const [selectedIndex, setSelectedIndex] = useState(0);
  const listRef = useRef(null);

  // Filter templates based on user input after '/'
  const filteredTemplates = useMemo(() => {
    const lowerFilter = filterText.toLowerCase();
    return PROMPT_TEMPLATES.filter(
      (template) =>
        template.label.toLowerCase().includes(lowerFilter) ||
        template.description.toLowerCase().includes(lowerFilter) ||
        template.id.toLowerCase().includes(lowerFilter)
    );
  }, [filterText]);

  // Reset selection when filter changes
  useEffect(() => {
    // eslint-disable-next-line react-hooks/set-state-in-effect
    setSelectedIndex(0);
  }, [filterText]);

  // Handle keyboard navigation
  useKeyboardNavigation({
    isVisible,
    itemCount: filteredTemplates.length,
    selectedIndex,
    setSelectedIndex,
    onSelect: () => {
      if (filteredTemplates[selectedIndex]) {
        onSelect(filteredTemplates[selectedIndex]);
      }
    },
    onClose,
  });

  // Scroll selected item into view
  useEffect(() => {
    if (listRef.current) {
      const selectedElement = listRef.current.children[selectedIndex];
      if (selectedElement) {
        selectedElement.scrollIntoView({ block: "nearest" });
      }
    }
  }, [selectedIndex]);

  if (!isVisible || filteredTemplates.length === 0) return null;

  const getIcon = (category) => CATEGORY_ICONS[category] || CATEGORY_ICONS.default;

  return (
    <div
      className="absolute z-50 w-80 mb-2 bg-black/80 backdrop-blur-xl border border-white/10 rounded-xl shadow-2xl overflow-hidden animate-in fade-in zoom-in-95 duration-200 slide-in-from-bottom-2"
      style={position}
      role="listbox"
      aria-label={t("templates.title")}
    >
      <div className="px-3 py-2 border-b border-white/5 bg-white/5 flex items-center justify-between">
        <span className="text-xs font-medium text-muted-foreground uppercase tracking-wider flex items-center gap-1.5">
          <Command className="w-3 h-3" aria-hidden="true" /> {t("templates.title")}
        </span>
        <span className="text-[10px] text-muted-foreground/60">
          {t("templates.navigation_hint")}
        </span>
      </div>

      <div
        ref={listRef}
        className="max-h-64 overflow-y-auto p-1 scrollbar-thin scrollbar-thumb-white/10 scrollbar-track-transparent"
      >
        {filteredTemplates.map((template, index) => (
          <button
            key={template.id}
            onClick={() => onSelect(template)}
            onMouseEnter={() => setSelectedIndex(index)}
            className={`w-full text-left px-3 py-2.5 rounded-lg flex items-start gap-3 transition-all duration-150 ${
              index === selectedIndex
                ? "bg-primary/20 text-white shadow-sm border border-primary/10"
                : "text-muted-foreground hover:bg-white/5 hover:text-foreground border border-transparent"
            }`}
            role="option"
            aria-selected={index === selectedIndex}
            id={`template-option-${index}`}
          >
            <div
              className={`mt-0.5 p-1.5 rounded-md ${index === selectedIndex ? "bg-primary/20" : "bg-white/5"}`}
            >
              {getIcon(template.category)}
            </div>
            <div className="flex-1 min-w-0">
              <div className="flex items-center justify-between">
                <span className="font-medium text-sm truncate">{template.label}</span>
                {index === selectedIndex && (
                  <span className="text-[10px] opacity-70 bg-white/10 px-1.5 py-0.5 rounded">
                    {t("templates.enter_hint")}
                  </span>
                )}
              </div>
              <p className="text-xs opacity-70 truncate mt-0.5">{template.description}</p>
            </div>
          </button>
        ))}
      </div>
    </div>
  );
});

PromptTemplates.propTypes = {
  isVisible: PropTypes.bool.isRequired,
  onSelect: PropTypes.func.isRequired,
  onClose: PropTypes.func.isRequired,
  filterText: PropTypes.string,
  position: PropTypes.shape({
    bottom: PropTypes.oneOfType([PropTypes.string, PropTypes.number]),
    left: PropTypes.oneOfType([PropTypes.string, PropTypes.number]),
    top: PropTypes.oneOfType([PropTypes.string, PropTypes.number]),
    right: PropTypes.oneOfType([PropTypes.string, PropTypes.number]),
  }),
};

export default PromptTemplates;
