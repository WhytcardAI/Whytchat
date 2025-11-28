import { useState, useRef, useEffect } from 'react';
import { ArrowRight, X, Sparkles, ChevronDown, ChevronRight, Settings2, FileText } from 'lucide-react';
import { useAppStore } from '../../store/appStore';
import { cn } from '../../lib/utils';
import { useTranslation } from 'react-i18next';
import i18n from '../../i18n';
import { logger } from '../../lib/logger';
import { invoke } from '@tauri-apps/api/core';

export function SessionWizard({ onClose }) {
  const { t } = useTranslation('common');
  const { createSession, libraryFiles, loadLibraryFiles } = useAppStore();
  const [sessionTitle, setSessionTitle] = useState('');
  const [systemPrompt, setSystemPrompt] = useState('');
  const [temperature, setTemperature] = useState(0.7);
  const [showAdvanced, setShowAdvanced] = useState(false);
  const [isCreating, setIsCreating] = useState(false);
  const [selectedFileIds, setSelectedFileIds] = useState([]);
  const inputRef = useRef(null);

  // Auto-focus input on mount and load library files
  useEffect(() => {
    inputRef.current?.focus();
    loadLibraryFiles();
  }, [loadLibraryFiles]);

  const toggleFileSelection = (fileId) => {
    logger.ui.click('SessionWizard:ToggleFile', { fileId });
    setSelectedFileIds(prev =>
      prev.includes(fileId)
        ? prev.filter(id => id !== fileId)
        : [...prev, fileId]
    );
  };

  const handleCreate = async () => {
    if (!sessionTitle.trim()) return;

    logger.ui.click('SessionWizard:Create', { title: sessionTitle, filesCount: selectedFileIds.length });
    setIsCreating(true);
    try {
      const promptToSend = systemPrompt.trim() || null;
      logger.session.create(sessionTitle.trim());
      const sessionId = await createSession(sessionTitle.trim(), i18n.language, promptToSend, temperature);

      // Link selected library files to the new session
      if (selectedFileIds.length > 0 && sessionId) {
        for (const fileId of selectedFileIds) {
          logger.file.link(fileId, sessionId);
          // Tauri auto-converts camelCase to snake_case
          await invoke('link_library_file_to_session', {
            sessionId: sessionId,
            fileId: fileId
          });
        }
      }

      logger.session.createSuccess(sessionId);
      onClose();
    } catch (error) {
      logger.system.error('SessionWizard:Create', error);
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
                <Sparkles className="w-6 h-6 text-primary-foreground" />
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

          {/* Library Files Selection */}
          {libraryFiles.length > 0 && (
            <div className="space-y-2">
              <label className="text-sm font-medium text-text flex justify-between items-center">
                <span>{t('session_wizard.files', 'Context Files')}</span>
                <span className="text-xs text-muted">{selectedFileIds.length} {t('session_wizard.selected', 'selected')}</span>
              </label>

              <div className="space-y-1 max-h-32 overflow-y-auto custom-scrollbar border border-border/50 rounded-lg p-2 bg-background/30">
                {libraryFiles.map((file) => (
                  <button
                    key={file.id}
                    type="button"
                    onClick={() => toggleFileSelection(file.id)}
                    className={cn(
                      "w-full flex items-center justify-between p-2 rounded-lg text-sm transition-all",
                      selectedFileIds.includes(file.id)
                        ? "bg-primary/10 border border-primary/30"
                        : "hover:bg-background/50 border border-transparent"
                    )}
                  >
                    <div className="flex items-center gap-2 truncate">
                      <FileText size={14} className={cn(
                        "shrink-0",
                        selectedFileIds.includes(file.id) ? "text-primary" : "text-muted"
                      )} />
                      <span className="truncate text-text/80">{file.original_name || file.file_name}</span>
                    </div>
                    {selectedFileIds.includes(file.id) && (
                      <div className="w-4 h-4 bg-primary rounded-full flex items-center justify-center">
                        <svg className="w-2.5 h-2.5 text-white" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={3} d="M5 13l4 4L19 7" />
                        </svg>
                      </div>
                    )}
                  </button>
                ))}
              </div>
              <p className="text-xs text-muted">
                {t('session_wizard.files_hint', 'Select files from your library to include in this conversation')}
              </p>
            </div>
          )}

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
                "flex-1 bg-primary hover:opacity-90 disabled:bg-muted disabled:cursor-not-allowed text-primary-foreground dark:text-zinc-900 rounded-xl p-3 flex items-center justify-center gap-2 transition-all font-medium hover:scale-[1.02] active:scale-[0.98]",
                isCreating && "animate-pulse"
              )}
            >
              {isCreating ? (
                <>
                  <div className="w-4 h-4 border-2 border-primary-foreground border-t-transparent rounded-full animate-spin" />
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
