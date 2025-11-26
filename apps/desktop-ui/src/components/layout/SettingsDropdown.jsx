import { useState, useMemo } from 'react';
import { Sliders, Thermometer, Save, RotateCcw, MessageSquare, Activity } from 'lucide-react';
import { useTranslation } from 'react-i18next';
import { useAppStore } from '../../store/appStore';
import { cn } from '../../lib/utils';
import i18n from '../../i18n';
import { logger } from '../../lib/logger';

const getDefaultPrompt = (lang) => {
  return lang === 'fr'
    ? 'Tu es un assistant IA utile et amical. Réponds toujours en français de manière claire et concise.'
    : 'You are a helpful and friendly AI assistant. Always respond clearly and concisely.';
};

const DEFAULT_CONFIG = {
  temperature: 0.7,
  system_prompt: getDefaultPrompt(i18n.language)
};

export function SettingsDropdown({ onClose }) {
  const { t } = useTranslation('common');
  const { currentSessionId, updateSession, sessions, setDiagnosticsOpen } = useAppStore();

  // Get current session config
  const currentSession = sessions.find(s => s.id === currentSessionId);

  const [config, setConfig] = useState({
    temperature: currentSession?.model_config?.temperature ?? DEFAULT_CONFIG.temperature,
    system_prompt: currentSession?.model_config?.system_prompt ?? DEFAULT_CONFIG.system_prompt
  });

  const hasChanges = useMemo(() => {
    const originalTemp = currentSession?.model_config?.temperature ?? DEFAULT_CONFIG.temperature;
    const originalPrompt = currentSession?.model_config?.system_prompt ?? DEFAULT_CONFIG.system_prompt;
    return config.temperature !== originalTemp ||
           config.system_prompt !== originalPrompt;
  }, [config, currentSession?.model_config?.temperature, currentSession?.model_config?.system_prompt]);

  const handleSave = async () => {
    if (!currentSessionId) return;
    logger.ui.click('SettingsDropdown:Save', { sessionId: currentSessionId });
    try {
      // Build full model_config for backend
      const fullConfig = {
        model_id: currentSession?.model_config?.model_id || 'default-model.gguf',
        temperature: config.temperature,
        system_prompt: config.system_prompt
      };
      await updateSession(currentSessionId, currentSession?.title, fullConfig);
      logger.store.action('updateSession', { sessionId: currentSessionId, temperature: config.temperature });
      onClose();
    } catch (error) {
      logger.system.error('SettingsDropdown:Save', error);
    }
  };

  const handleReset = () => {
    logger.ui.click('SettingsDropdown:Reset');
    setConfig({
      ...DEFAULT_CONFIG,
      system_prompt: getDefaultPrompt(i18n.language)
    });
  };

  return (
    <div className="absolute right-0 top-full mt-2 w-80 bg-surface border border-border rounded-xl shadow-xl z-50 overflow-hidden animate-fade-in">
      {/* Header */}
      <div className="px-4 py-3 border-b border-border bg-background/50">
        <div className="flex items-center gap-2">
          <Sliders size={16} className="text-primary" />
          <span className="font-medium text-sm">{t('header.settings', 'Settings')}</span>
        </div>
      </div>

      {/* Settings */}
      <div className="p-4 space-y-4 max-h-80 overflow-y-auto custom-scrollbar">
        {/* System Prompt */}
        <div className="space-y-2">
          <div className="flex items-center gap-2">
            <label className="flex items-center gap-2 text-sm font-medium">
              <MessageSquare size={14} className="text-accent" />
              {t('settings.systemPrompt.label', 'System Prompt')}
            </label>
          </div>
          <textarea
            value={config.system_prompt}
            onChange={(e) => setConfig({ ...config, system_prompt: e.target.value })}
            placeholder={t('settings.systemPrompt.placeholder', 'Enter instructions for the AI...')}
            className="w-full h-24 px-3 py-2 text-xs bg-background border border-border rounded-lg resize-none focus:outline-none focus:ring-2 focus:ring-primary/30 focus:border-primary/50"
          />
          <p className="text-[10px] text-muted">
            {t('settings.systemPrompt.description', 'Define how the AI should behave and respond')}
          </p>
        </div>

        {/* Temperature */}
        <div className="space-y-2">
          <div className="flex items-center justify-between">
            <label className="flex items-center gap-2 text-sm font-medium">
              <Thermometer size={14} className="text-accent" />
              {t('settings.temperature.label', 'Temperature')}
            </label>
            <span className="text-xs font-mono bg-background px-2 py-0.5 rounded">
              {config.temperature.toFixed(2)}
            </span>
          </div>
          <input
            type="range"
            min="0"
            max="2"
            step="0.1"
            value={config.temperature}
            onChange={(e) => setConfig({ ...config, temperature: parseFloat(e.target.value) })}
            className="w-full h-2 bg-background rounded-lg appearance-none cursor-pointer accent-primary"
          />
          <p className="text-[10px] text-muted">
            {t('settings.temperature.description', 'Controls randomness (0=focused, 2=creative)')}
          </p>
        </div>
      </div>

      {/* Actions */}
      <div className="px-4 py-3 border-t border-border flex flex-col gap-3">
        <button
          onClick={() => {
            setDiagnosticsOpen(true);
            onClose();
          }}
          className="w-full flex items-center justify-center gap-2 px-3 py-2 rounded-lg bg-surface hover:bg-background border border-border hover:border-primary/30 text-xs font-medium transition-colors group"
        >
          <Activity size={14} className="text-blue-400 group-hover:text-blue-300" />
          {t('settings.diagnostics', 'Run Diagnostics')}
        </button>

        <div className="flex items-center justify-between gap-2">
        <button
          onClick={handleReset}
          className="flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-muted hover:text-text hover:bg-background transition-colors text-xs"
        >
          <RotateCcw size={12} />
          {t('settings.reset', 'Reset')}
        </button>
        <button
          onClick={handleSave}
          disabled={!hasChanges || !currentSessionId}
          className={cn(
            'flex items-center gap-1.5 px-4 py-1.5 rounded-lg text-xs font-medium transition-colors',
            hasChanges && currentSessionId
              ? 'bg-primary text-primary-foreground hover:opacity-90'
              : 'bg-background text-muted cursor-not-allowed'
          )}
        >
          <Save size={12} />
          {t('settings.save', 'Save')}
        </button>
        </div>
      </div>
    </div>
  );
}
