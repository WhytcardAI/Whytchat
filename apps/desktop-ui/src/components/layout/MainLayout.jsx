import React from 'react';
import { useAppStore } from '../../store/appStore';
import { Plus, MessageSquare, Calendar, Users } from 'lucide-react';
import { cn } from '../../lib/utils';
import { useTranslation } from 'react-i18next';
import { RightPanel } from './RightPanel';

export function MainLayout({ children }) {
  const { isSidebarOpen, toggleSidebar, isThinking, thinkingSteps } = useAppStore();
  const { t } = useTranslation('common');

  return (
    <div className="flex h-screen bg-background text-text overflow-hidden font-sans">
      {/* Left Sidebar */}
      <aside className="w-72 bg-surface border-r border-border flex flex-col shrink-0">
        {/* Header / New Chat */}
        <div className="p-4">
          <button className="w-full bg-primary hover:bg-primary/90 text-white rounded-xl p-3 flex items-center justify-center gap-2 shadow-lg shadow-primary/20 transition-all font-medium">
            <Plus size={20} />
            <span>{t('nav.new_chat', 'Nouvelle')}</span>
          </button>
        </div>

        {/* History List */}
        <div className="flex-1 overflow-y-auto px-3 py-2 space-y-6">
          <HistoryGroup label="DATE.TODAY">
            <HistoryItem active label="New Chat" />
            <HistoryItem label="Projet Rust" />
          </HistoryGroup>

          <HistoryGroup label="DATE.YESTERDAY">
            <HistoryItem label="Recette de cuisine" />
            <HistoryItem label="Debug Tauri" />
          </HistoryGroup>

          <HistoryGroup label="DATE.LAST7DAYS">
            <HistoryItem icon={<Users size={16} />} label="Meeting: Team Sync" />
          </HistoryGroup>
        </div>
      </aside>

      {/* Center Content (Chat) */}
      <main className="flex-1 flex flex-col h-full relative bg-background min-w-0">
        {/* Minimal Header */}
        <header className="h-14 flex items-center justify-between px-6 border-b border-border/50 bg-surface/50 backdrop-blur">
          <div className="flex items-center gap-2">
            <div className="w-2 h-2 rounded-full bg-green-500"></div>
            <span className="font-medium text-sm text-text">New Chat</span>
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

function HistoryItem({ label, active, icon }) {
  return (
    <button className={cn(
      "w-full flex items-center gap-3 px-3 py-2.5 rounded-xl text-sm transition-all text-left group",
      active ? "bg-white shadow-sm ring-1 ring-border text-text" : "text-muted hover:bg-white/50 hover:text-text"
    )}>
      <span className={cn("shrink-0", active ? "text-primary" : "text-muted group-hover:text-text")}>
        {icon || <MessageSquare size={16} />}
      </span>
      <span className="truncate">{label}</span>
    </button>
  );
}
