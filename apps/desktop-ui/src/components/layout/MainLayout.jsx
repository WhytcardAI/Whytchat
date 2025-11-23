import { useAppStore } from '../../store/appStore';
import { Plus, MessageSquare } from 'lucide-react';
import { cn } from '../../lib/utils';
import { useTranslation } from 'react-i18next';
import { RightPanel } from './RightPanel';

export function MainLayout({ children }) {
  const { isThinking, thinkingSteps, sessions, currentSessionId, createSession, setCurrentSessionId } = useAppStore();
  const { t } = useTranslation('common');

  const handleNewChat = async () => {
    try {
      await createSession();
      // The createSession already sets currentSessionId
    } catch (error) {
      console.error('Failed to create new session:', error);
    }
  };

  const handleSessionSelect = (sessionId) => {
    setCurrentSessionId(sessionId);
  };

  return (
    <div className="flex h-screen bg-background text-text overflow-hidden font-sans">
      {/* Left Sidebar */}
      <aside className="w-72 bg-surface border-r border-border flex flex-col shrink-0">
        {/* Header / New Chat */}
        <div className="p-4">
          <button
            onClick={handleNewChat}
            className="w-full bg-primary hover:bg-primary/90 text-white rounded-xl p-3 flex items-center justify-center gap-2 shadow-lg shadow-primary/20 transition-all font-medium"
          >
            <Plus size={20} />
            <span>{t('nav.new_chat')}</span>
          </button>
        </div>

        {/* History List */}
        <div className="flex-1 overflow-y-auto px-3 py-2 space-y-6">
          {sessions.length > 0 ? (
            <HistoryGroup label={t('sessions.group')}>
              {sessions.map((session) => (
                <HistoryItem
                  key={session.id}
                  active={session.id === currentSessionId}
                  label={session.title || `Session ${session.id.slice(-8)}`}
                  onClick={() => handleSessionSelect(session.id)}
                />
              ))}
            </HistoryGroup>
          ) : (
            <div className="text-center text-muted text-sm mt-10">
              <MessageSquare size={32} className="mx-auto mb-2 opacity-20" />
              <p>{t('sessions.empty')}</p>
            </div>
          )}
        </div>
      </aside>

      {/* Center Content (Chat) */}
      <main className="flex-1 flex flex-col h-full relative bg-background min-w-0">
        {/* Minimal Header */}
        <header className="h-14 flex items-center justify-between px-6 border-b border-border/50 bg-surface/50 backdrop-blur">
          <div className="flex items-center gap-2">
            <div className="w-2 h-2 rounded-full bg-green-500"></div>
            <span className="font-medium text-sm text-text">
              {currentSessionId ? `Session ${currentSessionId.slice(-8)}` : t('chat.header.new_session')}
            </span>
          </div>
        </header>

        {children}
      </main>

      {/* Right Panel (Orchestration) */}
      <RightPanel isThinking={isThinking} thinkingSteps={thinkingSteps} />
    </div>
  );
}

function HistoryGroup({ label, children }) {
  return (
    <div className="space-y-2">
      <h3 className="text-[10px] font-bold text-muted uppercase tracking-wider px-3">{label}</h3>
      <div className="space-y-1">
        {children}
      </div>
    </div>
  );
}

function HistoryItem({ label, active, icon, onClick }) {
  return (
    <button
      onClick={onClick}
      className={cn(
        "w-full flex items-center gap-3 px-3 py-2.5 rounded-xl text-sm transition-all text-left group",
        active ? "bg-white shadow-sm ring-1 ring-border text-text" : "text-muted hover:bg-white/50 hover:text-text"
      )}
    >
      <span className={cn("shrink-0", active ? "text-primary" : "text-muted group-hover:text-text")}>
        {icon || <MessageSquare size={16} />}
      </span>
      <span className="truncate">{label}</span>
    </button>
  );
}
