import { useState, useCallback } from 'react';
import { ChevronDown, Brain, Loader2, CheckCircle2, Search, Lightbulb, MessageSquare } from 'lucide-react';
import { useTranslation } from 'react-i18next';
import { cn } from '../../lib/utils';

const stepIcons = {
  'thinking.analyzing': Search,
  'thinking.searching_context': Search,
  'thinking.documents_found': CheckCircle2,
  'thinking.no_documents': Search,
  'thinking.intent': Lightbulb,
  'thinking.generating_response': MessageSquare,
};

// Minimal inline thinking indicator
export const ThinkingIndicator = ({ isActive }) => {
  const { t } = useTranslation('common');

  if (!isActive) return null;

  return (
    <div className="flex items-center gap-2 px-3 py-1.5 text-xs text-primary animate-pulse">
      <div className="relative">
        <Brain size={14} />
        <span className="absolute -top-0.5 -right-0.5 w-1.5 h-1.5 bg-primary rounded-full animate-ping" />
      </div>
      <span>{t('chat.thinking', 'Processing...')}</span>
    </div>
  );
};

// Detailed thinking bubble (expandable, shown below messages when needed)
export const ThinkingBubble = ({ steps = [], isExpanded = false, isThinking = false }) => {
  const [isOpen, setIsOpen] = useState(isExpanded);
  const { t } = useTranslation('common');

  const translateStep = useCallback((step) => {
    if (step.includes('|')) {
      const parts = step.split('|');
      const key = parts[0];
      const value = parts.slice(1).join('|');
      if (key === 'thinking.intent') {
        return { text: t('thinking.intent', { intent: value }), key };
      } else if (key === 'thinking.documents_found') {
        return { text: t('thinking.documents_found', { count: value }), key };
      }
    }
    if (step.startsWith('thinking.')) {
      return { text: t(step), key: step };
    }
    return { text: step, key: 'default' };
  }, [t]);

  if (!steps || steps.length === 0) return null;

  // Compact mode: just show a small expandable badge
  return (
    <div className="mb-3 max-w-3xl mx-auto w-full">
      <button
        onClick={() => setIsOpen(!isOpen)}
        className={cn(
          "flex items-center gap-2 px-3 py-1.5 rounded-lg text-xs transition-all",
          "bg-surface/60 border border-border/50 hover:bg-surface/80",
          isOpen && "rounded-b-none border-b-0"
        )}
      >
        <Brain size={12} className="text-accent" />
        <span className="text-muted">
          {steps.length} {t('thinking.steps', 'steps')}
        </span>
        <ChevronDown
          size={12}
          className={cn("text-muted transition-transform", isOpen && "rotate-180")}
        />
      </button>

      {isOpen && (
        <div className="bg-surface/60 border border-border/50 border-t-0 rounded-b-lg p-2 space-y-1 animate-fade-in">
          {steps.map((step, index) => {
            const { text, key } = translateStep(step);
            const IconComponent = stepIcons[key] || Lightbulb;
            const isLast = index === steps.length - 1;

            return (
              <div
                key={index}
                className={cn(
                  "flex items-center gap-2 px-2 py-1 rounded text-xs",
                  isLast ? "text-primary" : "text-muted"
                )}
              >
                <IconComponent size={10} />
                <span className="flex-1 truncate">{text}</span>
              </div>
            );
          })}

          {/* Loading dot - Only show if actively thinking */}
          {isThinking && (
            <div className="flex items-center gap-2 px-2 py-1 text-primary">
              <Loader2 size={10} className="animate-spin" />
              <span className="text-xs">{t('chat.thinking', 'Processing...')}</span>
            </div>
          )}
        </div>
      )}
    </div>
  );
};
