import { useRef, useEffect, useCallback, useMemo } from 'react';
import { MessageBubble } from './MessageBubble';
import { ThinkingBubble } from './ThinkingBubble';
import { ChatInput } from './ChatInput';
import { MessageSquare } from 'lucide-react';
import { useAppStore } from '../../store/appStore';
import { useTranslation } from 'react-i18next';
import i18n from '../../i18n';
import { useChatStream } from '../../hooks/useChatStream';
import toast from 'react-hot-toast';
import { logger } from '../../lib/logger';

export function ChatInterface() {
  const { t } = useTranslation();
  const { isThinking, thinkingSteps, currentSessionId, setCurrentSessionId, loadSessions, createSession, quickAction, clearQuickAction, setIsCreatingSession } = useAppStore();

  const { messages, sendMessage, addSystemMessage } = useChatStream(currentSessionId);

  const messagesEndRef = useRef(null);
  const processingQuickActionRef = useRef(false);
  const lastProcessedActionRef = useRef(null);

  const scrollToBottom = useCallback(() => {
    messagesEndRef.current?.scrollIntoView({ behavior: "smooth" });
  }, []);

  useEffect(() => {
    scrollToBottom();
  }, [messages, scrollToBottom]);

  // Load sessions on mount
  useEffect(() => {
    loadSessions();
  }, [loadSessions]);

  // Handle Quick Actions (RAG) - ChatInterface is the ONLY handler for sending messages
  useEffect(() => {
    // Guard conditions to prevent multiple executions
    if (!quickAction) return;
    if (!currentSessionId) return; // Wait for session (Dashboard creates it if needed)
    if (processingQuickActionRef.current) return;

    // Check if we already processed this exact action (by comparing prompt)
    if (lastProcessedActionRef.current === quickAction.prompt) {
      logger.debug('CHAT', 'Skipping duplicate quickAction');
      clearQuickAction();
      return;
    }

    const handleQuickAction = async () => {
      processingQuickActionRef.current = true;
      lastProcessedActionRef.current = quickAction.prompt;

      logger.info('CHAT', 'Processing quickAction', {
        type: quickAction.type,
        sessionId: currentSessionId
      });

      // Capture the action data before clearing
      const actionPrompt = quickAction.prompt;
      const actionType = quickAction.type;

      // Clear immediately to prevent re-runs
      clearQuickAction();

      // Show system message if it's an analysis
      if (actionType === 'analyze') {
        addSystemMessage(t('thinking.analyzing', 'Analyzing request...'));
      }

      // Send the message
      try {
        await sendMessage(actionPrompt, currentSessionId, true); // Hide the prompt for quick actions
      } catch (error) {
        logger.system.error('quickAction:sendMessage', error);
      } finally {
        processingQuickActionRef.current = false;
      }
    };

    handleQuickAction();
  }, [quickAction, currentSessionId, sendMessage, clearQuickAction, t, addSystemMessage]);

  const handleSend = useCallback(async (text, _isWebEnabled) => {
    let activeSessionId = currentSessionId;

    if (!activeSessionId) {
      logger.session.create(t('session.title.default'));
      try {
        activeSessionId = await createSession(t('session.title.default'), i18n.language);
        logger.session.createSuccess(activeSessionId);
        setCurrentSessionId(activeSessionId);
      } catch (error) {
        logger.system.error('createSession', error);
        toast.error(t('session.error.create', 'Failed to create session'));
        return;
      }
    }

    logger.chat.sendMessage(activeSessionId, text);
    // The useChatStream hook will handle the rest
    await sendMessage(text, activeSessionId);

  }, [currentSessionId, createSession, setCurrentSessionId, sendMessage, t]);

  const renderedMessages = useMemo(() => {
    return messages.map((msg) => (
      <MessageBubble key={msg.id} role={msg.role} content={msg.content} sessionId={currentSessionId} />
    ));
  }, [messages, currentSessionId]);

  return (
    <div className="flex flex-col h-full w-full relative">
      {/* Messages Area */}
      <div className="flex-1 overflow-y-auto p-4 space-y-4 custom-scrollbar pb-40">
        {messages.length === 0 && (
          <div className="h-full flex flex-col items-center justify-center text-muted-foreground">
            {!currentSessionId ? (
              <div className="text-center animate-fade-in max-w-md px-6">
                <div className="w-16 h-16 rounded-2xl bg-surface border border-border flex items-center justify-center mx-auto mb-6 shadow-sm">
                  <MessageSquare className="w-8 h-8 text-muted-foreground/50" />
                </div>
                <h2 className="text-xl font-semibold text-foreground mb-3">{t('chat.empty.title', 'Welcome to WhytChat')}</h2>
                <p className="text-sm text-muted-foreground mb-8 leading-relaxed">{t('chat.empty.message')}</p>
                <button
                    onClick={() => {
                      logger.ui.click('ChatInterface:NewSession');
                      setIsCreatingSession(true);
                    }}
                    className="px-5 py-2.5 bg-primary text-primary-foreground dark:text-zinc-900 rounded-lg text-sm font-medium hover:opacity-90 transition-all shadow-sm"
                >
                    {t('chat.header.new_session')}
                </button>
              </div>
            ) : (
              <div className="flex flex-col items-center animate-fade-in">
                <div className="w-12 h-12 rounded-xl bg-primary/5 flex items-center justify-center mb-4">
                  <MessageSquare className="w-6 h-6 text-primary" />
                </div>
                <h2 className="text-base font-medium text-foreground mb-2">{t('chat.empty.session_ready')}</h2>
                <p className="text-sm text-muted-foreground max-w-xs text-center">{t('chat.empty.start_hint', 'Type your message below to start chatting')}</p>
              </div>
            )}
          </div>
        )}

        {renderedMessages}

        {/* Thinking Bubble - Shows during generation or if steps exist */}
        {(isThinking || thinkingSteps.length > 0) && (
          <ThinkingBubble steps={thinkingSteps} isThinking={isThinking} />
        )}

        <div ref={messagesEndRef} />
      </div>

      {/* Input Area (Floating) */}
      <div className="absolute bottom-0 left-0 right-0 bg-gradient-to-t from-background via-background/95 to-transparent pt-8 z-10">
        <ChatInput onSend={handleSend} disabled={isThinking} />
      </div>
    </div>
  );
}
