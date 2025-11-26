import { useState, useRef, useEffect } from 'react';
import { ArrowRight, X, Sparkles, ChevronDown, ChevronRight, Settings2 } from 'lucide-react';
import { useAppStore } from '../../store/appStore';
import { cn } from '../../lib/utils';
import { useTranslation } from 'react-i18next';
import i18n from '../../i18n';

export function SessionWizard({ onClose }) {
  const { t } = useTranslation('common');
  const { createSession } = useAppStore();
  const [sessionTitle, setSessionTitle] = useState('');
  const [systemPrompt, setSystemPrompt] = useState('');
  const [temperature, setTemperature] = useState(0.7);
  const [showAdvanced, setShowAdvanced] = useState(false);
  const [isCreating, setIsCreating] = useState(false);
  const inputRef = useRef(null);

  // Auto-focus input on mount
  useEffect(() => {
    inputRef.current?.focus();
  }, []);

  const handleCreate = async () => {
    if (!sessionTitle.trim()) return;

    setIsCreating(true);
    try {
      const promptToSend = systemPrompt.trim() || null;
      await createSession(sessionTitle.trim(), i18n.language, promptToSend, temperature);
      onClose();
    } catch (error) {
      console.error('Failed to create session:', error);
    } finally {
      setIsCreating(false);
    }
  };

  const handleKeyPress = (e) => {
    if (e.key === 'Enter' && sessionTitle.trim() && !e.shiftKey) {
      handleCreate();
    }
    if (e.key === 'Escape') {
      onClose();
    }
  };

  return (
    <div className="fixed inset-0 bg-background/80 backdrop-blur-sm flex items-center justify-center z-50 animate-fade-in">
      <div className="w-full max-w-md bg-surface/95 backdrop-blur-xl rounded-2xl shadow-2xl border border-border overflow-hidden animate-scale-in max-h-[90vh] flex flex-col">

        {/* Header */}
        <div className="p-6 border-b border-border/50 bg-gradient-to-br from-primary/5 to-accent/5 shrink-0">
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-4">
              <div className="w-12 h-12 bg-gradient-to-br from-primary to-accent rounded-xl flex items-center justify-center shadow-lg shadow-primary/20">
                <Sparkles className="w-6 h-6 text-white" />
              </div>
              <div>
                <h2 className="text-lg font-bold text-text">
                  {t('session_wizard.title', 'New Conversation')}
                </h2>
                <p className="text-sm text-muted">
                  {t('session_wizard.subtitle', 'Give your conversation a name')}
                </p>
              </div>
            </div>
            <button
              onClick={onClose}
              className="p-2 hover:bg-background/50 rounded-xl transition-colors group"
            >
              <X size={20} className="text-muted group-hover:text-text transition-colors" />
            </button>
          </div>
        </div>

        {/* Content */}
        <div className="p-6 space-y-6 overflow-y-auto custom-scrollbar">
          <div className="space-y-2">
            <label className="text-sm font-medium text-text">
              {t('session_wizard.label', 'Conversation Title')}
            </label>
            <input
              ref={inputRef}
              type="text"
              value={sessionTitle}
              onChange={(e) => setSessionTitle(e.target.value)}
              onKeyDown={handleKeyPress}
              placeholder={t('session_wizard.placeholder', 'e.g., AI Discussion')}
              className="w-full px-4 py-3.5 bg-background/50 border border-border/50 rounded-xl focus:outline-none focus:ring-2 focus:ring-primary/30 focus:border-primary/50 transition-all placeholder:text-muted/50"
              disabled={isCreating}
            />
          </div>

          {/* Advanced Options Toggle */}
          <div className="space-y-4">
            <button
              onClick={() => setShowAdvanced(!showAdvanced)}
              className="flex items-center gap-2 text-sm font-medium text-muted hover:text-primary transition-colors"
            >
              {showAdvanced ? <ChevronDown size={16} /> : <ChevronRight size={16} />}
              <Settings2 size={16} />
              {t('session_wizard.advanced', 'Advanced Options')}
            </button>

            {showAdvanced && (
              <div className="space-y-4 animate-fade-in pl-2 border-l-2 border-border/50 ml-1">
                {/* System Prompt */}
                <div className="space-y-2">
                  <label className="text-sm font-medium text-text">
                    {t('session_wizard.system_prompt', 'System Prompt')}
                  </label>
                  <textarea
                    value={systemPrompt}
                    onChange={(e) => setSystemPrompt(e.target.value)}
                    placeholder={t('session_wizard.system_prompt_placeholder', 'You are a helpful assistant...')}
                    className="w-full px-4 py-3 bg-background/50 border border-border/50 rounded-xl focus:outline-none focus:ring-2 focus:ring-primary/30 focus:border-primary/50 transition-all placeholder:text-muted/50 min-h-[100px] resize-y text-sm"
                    disabled={isCreating}
                  />
                </div>

                {/* Temperature */}
                <div className="space-y-2">
                  <div className="flex justify-between">
                    <label className="text-sm font-medium text-text">
                      {t('session_wizard.temperature', 'Creativity (Temperature)')}
                    </label>
                    <span className="text-sm text-muted font-mono">{temperature}</span>
                  </div>
                  <input
                    type="range"
                    min="0"
                    max="2"
                    step="0.1"
                    value={temperature}
                    onChange={(e) => setTemperature(parseFloat(e.target.value))}
                    className="w-full accent-primary h-2 bg-border/50 rounded-lg appearance-none cursor-pointer"
                    disabled={isCreating}
                  />
                  <div className="flex justify-between text-xs text-muted">
                    <span>{t('session_wizard.temp_precise', 'Precise')}</span>
                    <span>{t('session_wizard.temp_creative', 'Creative')}</span>
                  </div>
                </div>
              </div>
            )}
          </div>

          <div className="flex gap-3 pt-2">
            <button
              onClick={onClose}
              className="flex-1 bg-background hover:bg-surface border border-border text-text rounded-xl p-3 font-medium transition-all"
            >
              {t('session_wizard.cancel', 'Cancel')}
            </button>
            <button
              onClick={handleCreate}
              disabled={!sessionTitle.trim() || isCreating}
              className={cn(
                "flex-1 bg-primary hover:bg-primary/90 disabled:bg-muted disabled:cursor-not-allowed text-white rounded-xl p-3 flex items-center justify-center gap-2 transition-all font-medium shadow-lg shadow-primary/20 hover:shadow-xl hover:shadow-primary/30 hover:scale-[1.02] active:scale-[0.98]",
                isCreating && "animate-pulse"
              )}
            >
              {isCreating ? (
                <>
                  <div className="w-4 h-4 border-2 border-white border-t-transparent rounded-full animate-spin" />
                  {t('session_wizard.creating', 'Creating...')}
                </>
              ) : (
                <>
                  {t('session_wizard.create', 'Create')}
                  <ArrowRight size={18} />
                </>
              )}
            </button>
          </div>
        </div>
      </div>
    </div>
  );
}
