import { useState, useRef, useEffect, useCallback, useMemo } from 'react';
import { MessageBubble } from './MessageBubble';
import { ThinkingBubble } from './ThinkingBubble';
import { ChatInput } from './ChatInput';
import { invoke } from '@tauri-apps/api/core';
import { MessageSquare, Loader2, CheckCircle, AlertCircle } from 'lucide-react';
import { useAppStore } from '../../store/appStore';
import { useTranslation } from 'react-i18next';
import i18n from '../../i18n';
import { useChatStream } from '../../hooks/useChatStream';
import { cn } from '../../lib/utils';

export function ChatInterface() {
  const { t } = useTranslation();
  const [uploadStatus, setUploadStatus] = useState(null); // null, 'uploading', 'success', 'error'
  const { isThinking, thinkingSteps, currentSessionId, setCurrentSessionId, loadSessions, createSession } = useAppStore();

  const { messages, sendMessage } = useChatStream(currentSessionId);

  const messagesEndRef = useRef(null);

  const scrollToBottom = useCallback(() => {
    messagesEndRef.current?.scrollIntoView({ behavior: "smooth" });
  }, []);

  useEffect(() => {
    scrollToBottom();
  }, [messages, scrollToBottom]);

  // Load sessions on mount
  useEffect(() => {
    loadSessions();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []); // Only run once on mount

  const handleSend = useCallback(async (text, _isWebEnabled) => {
    let activeSessionId = currentSessionId;

    if (!activeSessionId) {
      console.log("No active session, creating one...");
      try {
        activeSessionId = await createSession(t('session.title.default'), i18n.language);
        setCurrentSessionId(activeSessionId);
      } catch (error) {
        console.error("Failed to auto-create session:", error);
        // We can't use the stream's setMessages, so maybe we show an alert here in the future
        return;
      }
    }

    // The useChatStream hook will handle the rest
    await sendMessage(text, activeSessionId);

  }, [currentSessionId, createSession, setCurrentSessionId, sendMessage, t]);

  const handleFileUpload = useCallback(async (file) => {
    let activeSessionId = currentSessionId;
    if (!activeSessionId) {
         try {
            activeSessionId = await createSession(t('session.title.default'), i18n.language);
            setCurrentSessionId(activeSessionId);
        } catch (error) {
            console.error("Failed to auto-create session for upload:", error);
            setUploadStatus('error');
            return;
        }
    }

    setUploadStatus('uploading');
    try {
      const reader = new FileReader();
      reader.onload = async (e) => {
        const arrayBuffer = e.target.result;
        const fileData = Array.from(new Uint8Array(arrayBuffer));
        await invoke('upload_file_for_session', {
          session_id: activeSessionId,
          file_name: file.name,
          file_data: fileData
        });
        setUploadStatus('success');
        setTimeout(() => setUploadStatus(null), 3000); // Reset after 3 seconds
      };
      reader.onerror = () => {
        setUploadStatus('error');
        setTimeout(() => setUploadStatus(null), 3000);
      };
      reader.readAsArrayBuffer(file);
    } catch (error) {
      console.error('Upload error:', error);
      setUploadStatus('error');
      setTimeout(() => setUploadStatus(null), 3000);
    }
  }, [currentSessionId, createSession, setCurrentSessionId, t]);

  const renderedMessages = useMemo(() => {
    return messages.map((msg, idx) => (
      <MessageBubble key={idx} role={msg.role} content={msg.content} />
    ));
  }, [messages]);

  return (
    <div className="flex flex-col h-full w-full relative">
      {/* Messages Area */}
      <div className="flex-1 overflow-y-auto p-4 space-y-4 custom-scrollbar pb-40">
        {messages.length === 0 && (
          <div className="h-full flex flex-col items-center justify-center text-muted">
            {!currentSessionId ? (
              <div className="text-center animate-fade-in">
                <div className="w-20 h-20 rounded-2xl bg-surface/80 border border-border flex items-center justify-center mx-auto mb-4">
                  <MessageSquare className="w-10 h-10 text-muted/40" />
                </div>
                <h2 className="text-lg font-medium text-text mb-2">{t('chat.empty.title', 'Welcome to WhytChat')}</h2>
                <p className="text-sm text-muted mb-6 max-w-xs">{t('chat.empty.message')}</p>
                <button
                    onClick={() => createSession(t('session.title.default'), i18n.language).then(id => setCurrentSessionId(id))}
                    className="px-6 py-3 bg-primary text-white rounded-xl font-medium shadow-lg shadow-primary/20 hover:bg-primary/90 hover:scale-[1.02] active:scale-[0.98] transition-all"
                >
                    {t('chat.header.new_session')}
                </button>
              </div>
            ) : (
              <div className="flex flex-col items-center animate-fade-in">
                <div className="w-20 h-20 rounded-2xl bg-primary/10 border border-primary/20 flex items-center justify-center mb-4">
                  <MessageSquare className="w-10 h-10 text-primary" />
                </div>
                <h2 className="text-lg font-medium text-text mb-2">{t('chat.empty.session_ready')}</h2>
                <p className="text-sm text-muted max-w-xs text-center">{t('chat.empty.start_hint', 'Type your message below to start chatting')}</p>
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
        {uploadStatus && (
          <div className="max-w-3xl mx-auto px-4 mb-3">
            <div className={cn(
              "flex items-center gap-2 p-3 rounded-xl text-sm border animate-fade-in",
              uploadStatus === 'success' && "bg-success/10 border-success/20 text-success",
              uploadStatus === 'error' && "bg-destructive/10 border-destructive/20 text-destructive",
              uploadStatus === 'uploading' && "bg-primary/10 border-primary/20 text-primary"
            )}>
              {uploadStatus === 'uploading' && <Loader2 size={16} className="animate-spin" />}
              {uploadStatus === 'success' && <CheckCircle size={16} />}
              {uploadStatus === 'error' && <AlertCircle size={16} />}
              <span>
                {uploadStatus === 'uploading' ? t('chat.upload.uploading') : uploadStatus === 'success' ? t('chat.upload.success') : t('chat.upload.error')}
              </span>
            </div>
          </div>
        )}
        <ChatInput onSend={handleSend} onFileUpload={handleFileUpload} disabled={isThinking} />
      </div>
    </div>
  );
}
