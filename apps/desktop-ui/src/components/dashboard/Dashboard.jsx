import { useState, useRef, useEffect } from 'react';
import { MessageSquare, Plus, FileUp, X, FileText, ArrowRight, Loader2 } from 'lucide-react';
import { useTranslation } from 'react-i18next';
import { useAppStore } from '../../store/appStore';
import i18n from '../../i18n';
import toast from 'react-hot-toast';
import { logger } from '../../lib/logger';

export function Dashboard({ onNewChat }) {
  const { t } = useTranslation('common');
  const { createSession, uploadFile, setCurrentSessionId, setQuickAction, quickAction, clearQuickAction, currentSessionId } = useAppStore();
  const [selectedFiles, setSelectedFiles] = useState([]);
  const [isCreating, setIsCreating] = useState(false);
  const fileInputRef = useRef(null);
  const handledQuickActionRef = useRef(null); // Track which quickAction we've handled

  // Handle quickAction from KnowledgeView when no session exists
  // Dashboard is responsible for creating the session, ChatInterface handles sending the message
  useEffect(() => {
    // Guard: Skip if no action, already creating, or session already exists
    if (!quickAction || isCreating || currentSessionId) {
      return;
    }

    // Use prompt as unique identifier for the action
    const actionId = quickAction.prompt;
    if (handledQuickActionRef.current === actionId) {
      logger.debug('DASHBOARD', 'Skipping already handled quickAction');
      return;
    }

    // Mark this action as being handled
    handledQuickActionRef.current = actionId;

    logger.info('DASHBOARD', 'Creating session for quickAction', { type: quickAction.type });

    const createSessionForQuickAction = async () => {
      setIsCreating(true);

      // Capture the quickAction data - we'll keep it for ChatInterface
      const actionData = { ...quickAction };

      try {
        const title = actionData.payload
          ? t('dashboard.analysis_title', 'Analysis: {{name}}', { name: actionData.payload })
          : t('chat.new_chat', 'New Chat');

        logger.session.create(title);
        const sessionId = await createSession(title, i18n.language);

        if (sessionId) {
          logger.session.createSuccess(sessionId);
          // DON'T clear quickAction - ChatInterface will pick it up and send the message
          setCurrentSessionId(sessionId);
        } else {
          // Session creation failed silently
          clearQuickAction();
          handledQuickActionRef.current = null;
        }
      } catch (error) {
        logger.system.error('Dashboard:createSessionForQuickAction', error);
        toast.error(t('common.error', 'An error occurred'));
        clearQuickAction();
        handledQuickActionRef.current = null;
      } finally {
        setIsCreating(false);
      }
    };

    createSessionForQuickAction();
  }, [quickAction, isCreating, currentSessionId, createSession, setCurrentSessionId, clearQuickAction, t]);

  const handleFileSelect = (e) => {
    const files = Array.from(e.target.files || []);
    logger.file.select(files.map(f => f.name).join(', '));
    setSelectedFiles(prev => [...prev, ...files]);
    if (fileInputRef.current) fileInputRef.current.value = '';
  };

  const removeFile = (index) => {
    logger.ui.click('Dashboard:RemoveFile', { index });
    setSelectedFiles(prev => prev.filter((_, i) => i !== index));
  };

  const handleStartAnalysis = async () => {
    if (selectedFiles.length === 0) {
      logger.ui.click('Dashboard:NewChat');
      if (onNewChat) onNewChat();
      return;
    }

    logger.ui.click('Dashboard:StartAnalysis', { fileCount: selectedFiles.length });
    setIsCreating(true);
    try {
      // Create session with a descriptive title based on files
      const title = selectedFiles.length === 1
        ? `Analysis: ${selectedFiles[0].name}`
        : `Analysis: ${selectedFiles.length} files`;

      logger.session.create(title);
      const sessionId = await createSession(title, i18n.language);

      if (sessionId) {
        logger.session.createSuccess(sessionId);
        // Upload all files
        const uploadPromises = selectedFiles.map(file => {
          logger.file.upload(file.name, sessionId);
          return uploadFile(sessionId, file);
        });
        await Promise.all(uploadPromises);

        // Create the prompt for analysis
        const analysisPrompt = t('rag.analyze_prompt', 'Can you analyze the file {{fileName}} and tell me what it is about?', { fileName: selectedFiles.map(f => f.name).join(', ') });

        // Mark this action as already handled by us (prevent useEffect from processing it)
        handledQuickActionRef.current = analysisPrompt;

        // Set quick action to trigger immediate analysis
        logger.file.analyze(selectedFiles.map(f => f.name).join(', '));
        setQuickAction({
          type: 'analyze',
          payload: 'Initial Files',
          prompt: analysisPrompt
        });

        // Switch to the new session
        setCurrentSessionId(sessionId);
        toast.success(t('dashboard.analysis_started', 'Analysis started'));
      }
    } catch (error) {
      logger.system.error('startAnalysis', error);
      toast.error(t('common.error', 'An error occurred'));
    } finally {
      setIsCreating(false);
    }
  };

  return (
    <div className="flex flex-col items-center justify-center h-full bg-background text-text p-8 animate-in fade-in zoom-in duration-500">
      <div className="max-w-2xl w-full text-center space-y-8">
        {/* Icon / Brand */}
        <div className="flex justify-center">
          <div className="w-20 h-20 bg-primary/10 rounded-3xl flex items-center justify-center shadow-xl shadow-primary/5 ring-1 ring-primary/20">
            <MessageSquare className="w-10 h-10 text-primary" strokeWidth={1.5} />
          </div>
        </div>

        {/* Welcome Text */}
        <div className="space-y-4">
          <h1 className="text-4xl font-bold tracking-tight text-text">
            {t('dashboard.welcome')}
          </h1>
          <p className="text-lg text-muted max-w-md mx-auto leading-relaxed">
            {t('dashboard.subtitle')}
          </p>
        </div>

        {/* File Selection Area */}
        <div className="max-w-md mx-auto w-full space-y-4">
          {selectedFiles.length > 0 && (
            <div className="bg-surface/50 border border-border rounded-xl p-2 space-y-2 max-h-40 overflow-y-auto custom-scrollbar">
              {selectedFiles.map((file, idx) => (
                <div key={idx} className="flex items-center justify-between p-2 bg-background rounded-lg text-sm group">
                  <div className="flex items-center gap-2 truncate">
                    <FileText size={14} className="text-primary shrink-0" />
                    <span className="truncate text-text/80">{file.name}</span>
                  </div>
                  <button
                    onClick={() => removeFile(idx)}
                    className="p-1 hover:bg-destructive/10 hover:text-destructive rounded transition-colors opacity-0 group-hover:opacity-100"
                  >
                    <X size={14} />
                  </button>
                </div>
              ))}
            </div>
          )}

          <div className="flex gap-3">
            <button
              onClick={() => fileInputRef.current?.click()}
              className="flex-1 py-3 px-4 border border-dashed border-border rounded-xl text-sm text-muted hover:text-primary hover:border-primary/50 hover:bg-primary/5 transition-all flex items-center justify-center gap-2"
            >
              <FileUp size={18} />
              {selectedFiles.length === 0 ? t('dashboard.add_files', 'Add Context Files') : t('dashboard.add_more', 'Add More')}
            </button>
            <input
              type="file"
              ref={fileInputRef}
              onChange={handleFileSelect}
              className="hidden"
              multiple
            />
          </div>
        </div>

        {/* Actions */}
        <div className="flex flex-col sm:flex-row items-center justify-center gap-4 pt-2">
          <button
            onClick={handleStartAnalysis}
            disabled={isCreating}
            className="group relative inline-flex items-center justify-center gap-2 px-8 py-3 text-sm font-medium text-primary-foreground transition-all duration-300 bg-primary rounded-xl hover:opacity-90 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-primary disabled:opacity-70 disabled:cursor-not-allowed min-w-[200px]"
          >
            {isCreating ? (
              <Loader2 className="w-5 h-5 animate-spin" />
            ) : selectedFiles.length > 0 ? (
              <>
                <span>{t('dashboard.start_analysis', 'Start Analysis')}</span>
                <ArrowRight className="w-5 h-5 transition-transform group-hover:translate-x-1" />
              </>
            ) : (
              <>
                <Plus className="w-5 h-5 transition-transform group-hover:rotate-90" />
                <span>{t('dashboard.new_chat')}</span>
              </>
            )}
          </button>
        </div>
      </div>
    </div>
  );
}
