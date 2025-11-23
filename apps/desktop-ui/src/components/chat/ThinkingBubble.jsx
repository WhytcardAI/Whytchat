import React, { useState, useCallback } from 'react';
import { ChevronDown, ChevronRight, BrainCircuit } from 'lucide-react';
import { useTranslation } from 'react-i18next';

export const ThinkingBubble = React.memo(function ThinkingBubble({ steps = [] }) {
  const [isOpen, setIsOpen] = useState(true);
  const { t } = useTranslation('common');

  // Helper to translate step text from backend
  const translateStep = useCallback((step) => {
    // Handle translation keys with parameters (e.g., "thinking.intent|Some intent")
    if (step.includes('|')) {
      const parts = step.split('|');
      const key = parts[0];
      const value = parts.slice(1).join('|'); // Rejoin in case value contains pipes
      if (key === 'thinking.intent') {
        return t('thinking.intent', { intent: value });
      } else if (key === 'thinking.documents_found') {
        return t('thinking.documents_found', { count: value });
      }
    }

    // Check if step is a translation key
    if (step.startsWith('thinking.')) {
      return t(step);
    }

    // Return as-is if not a translation key
    return step;
  }, [t]);

  if (!steps || steps.length === 0) return null;

  return (
    <div className="mb-4 max-w-3xl mx-auto w-full">
      <div className="bg-surface/50 border border-slate-700 rounded-lg overflow-hidden">
        <button
          onClick={function() { return setIsOpen(!isOpen); }}
          className="w-full flex items-center justify-between p-3 text-xs font-medium text-muted hover:bg-slate-700/50 transition-colors"
        >
          <div className="flex items-center gap-2">
            <BrainCircuit size={14} className="text-accent animate-pulse" />
            <span>{t('chat.thinking_process', 'Thought Process')}</span>
          </div>
          {isOpen ? <ChevronDown size={14} /> : <ChevronRight size={14} />}
        </button>

        {isOpen && (
          <div className="p-3 pt-0 border-t border-slate-700/50 bg-slate-900/30">
            <ul className="space-y-2">
              {steps.map(function(step, index) {
                return (
                  <li key={index} className="flex gap-3 text-xs text-slate-300 animate-in fade-in slide-in-from-left-2 duration-300">
                    <span className="text-slate-600 font-mono">{index + 1}.</span>
                    <span>{translateStep(step)}</span>
                  </li>
                );
              })}
              <li className="flex gap-3 text-xs text-accent animate-pulse">
                <span className="text-slate-600 font-mono">...</span>
                <span>{t('chat.thinking', 'Reasoning...')}</span>
              </li>
            </ul>
          </div>
        )}
      </div>
    </div>
  );
});
