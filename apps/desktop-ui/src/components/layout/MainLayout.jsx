import React, { useEffect, useCallback, useMemo, useState } from 'react';
import { useAppStore } from '../../store/appStore';
import { Plus, MessageSquare, PanelLeft, Sparkles, Star, FolderPlus, ChevronDown, ChevronRight, Folder, Trash2 } from 'lucide-react';
import { cn } from '../../lib/utils';
import { useTranslation } from 'react-i18next';
import toast, { Toaster } from 'react-hot-toast';
import { SessionWizard } from '../onboarding/SessionWizard';
import { HeaderActions } from './HeaderActions';
import { SessionItem } from './SessionItem';
import { Rail } from './Rail';
import { KnowledgeView } from '../views/KnowledgeView';
import { TitleBar } from './TitleBar';

const HistoryGroup = React.memo(function HistoryGroup({ label, children, icon }) {
  return (
    <div className="space-y-2">
      <h3 className="text-[10px] font-semibold text-muted uppercase tracking-wider px-3 flex items-center gap-1.5">
        {icon}
        {label}
      </h3>
      <div className="space-y-1">
        {children}
      </div>
    </div>
  );
});

export function MainLayout({ children }) {
  const {
    isBackendInitialized,
    isBackendInitializing,
    initializationError,
    sessions,
    currentSessionId,
    setCurrentSessionId,
    isSidebarOpen,
    toggleSidebar,
    loadFolders,
    folders,
    createFolder,
    deleteFolder,
    currentView,
    isCreatingSession,
    setIsCreatingSession
  } = useAppStore();
  const { t } = useTranslation('common');

  const [isCreatingFolder, setIsCreatingFolder] = useState(false);
  const [newFolderName, setNewFolderName] = useState('');
  const [expandedFolders, setExpandedFolders] = useState({});

  useEffect(() => {
    // initializeApp is called in App.jsx during preflight
    // Only load folders here
    loadFolders();
  }, [loadFolders]);

  const handleNewChat = useCallback(() => {
    setIsCreatingSession(true);
  }, [setIsCreatingSession]);

  const handleSessionSelect = useCallback((sessionId) => {
    setCurrentSessionId(sessionId);
  }, [setCurrentSessionId]);

  const handleCreateFolder = useCallback(async () => {
    if (!newFolderName.trim()) return;
    await createFolder(newFolderName.trim());
    setNewFolderName('');
    setIsCreatingFolder(false);
  }, [newFolderName, createFolder]);

  const toggleFolderExpanded = useCallback((folderId) => {
    setExpandedFolders(prev => ({
      ...prev,
      [folderId]: !prev[folderId]
    }));
  }, []);

  const handleDeleteFolder = useCallback(async (folderId) => {
    await deleteFolder(folderId);
  }, [deleteFolder]);

  // Group sessions by folder
  const sessionsByFolder = useMemo(() => {
    const grouped = { unfiled: [] };
    folders.forEach(f => { grouped[f.id] = []; });
    sessions.forEach(s => {
      if (s.folder_id && grouped[s.folder_id]) {
        grouped[s.folder_id].push(s);
      } else {
        grouped.unfiled.push(s);
      }
    });
    return grouped;
  }, [sessions, folders]);

  const currentSessionDisplay = useMemo(() => {
    const currentSession = sessions.find(s => s.id === currentSessionId);
    if (currentSession) {
      return currentSession.title || 'Session ' + currentSessionId.slice(-8);
    }
    return t('chat.header.new_session');
  }, [currentSessionId, sessions, t]);

  // Get favorite sessions
  const favoriteSessions = useMemo(() => {
    return sessions.filter(s => s.is_favorite);
  }, [sessions]);

  const favoriteItems = useMemo(() => favoriteSessions.map(session => (
    <SessionItem
      key={session.id}
      session={session}
      active={session.id === currentSessionId}
      onClick={() => handleSessionSelect(session.id)}
    />
  )), [favoriteSessions, currentSessionId, handleSessionSelect]);

  if (isBackendInitializing) {
    return (
      <div className="flex h-screen w-full items-center justify-center bg-background">
        <div className="flex flex-col items-center gap-4">
          <div className="relative">
            <div className="w-16 h-16 rounded-2xl bg-primary/20 flex items-center justify-center">
              <Sparkles size={32} className="text-primary animate-pulse" />
            </div>
            <div className="absolute inset-0 w-16 h-16 rounded-2xl border-2 border-primary/30 animate-ping" />
          </div>
          <p className="text-muted text-sm">{t('app.initializing', 'Initializing Backend...')}</p>
        </div>
      </div>
    );
  }

  if (!isBackendInitialized) {
    return (
      <div className="flex h-screen w-full items-center justify-center bg-background">
        <div className="text-center p-8 bg-surface/80 rounded-2xl border border-destructive/20 max-w-md">
          <div className="w-16 h-16 rounded-2xl bg-destructive/10 flex items-center justify-center mx-auto mb-4">
            <span className="text-destructive text-3xl">!</span>
          </div>
          <h1 className="text-xl font-bold text-destructive mb-2">{t('app.init_failed', 'Backend Initialization Failed')}</h1>
          <p className="text-muted text-sm mb-4">{t('app.init_failed_desc', 'Could not connect to the backend. Please restart the application.')}</p>
          {initializationError && (
            <div className="p-3 bg-destructive/5 rounded-lg border border-destructive/10 text-xs text-destructive/80 font-mono break-all">
              {initializationError}
            </div>
          )}
        </div>
      </div>
    );
  }

  return (
    <div className="flex flex-col h-screen bg-background text-text overflow-hidden font-sans">
      <TitleBar />
      <div className="flex flex-1 overflow-hidden relative min-w-0">
        {/* New Navigation Rail */}
        <Rail />

        {/* Left Sidebar (History) - Hidden in Knowledge view */}
        {isSidebarOpen && currentView !== 'knowledge' && (
            <aside className="w-72 bg-surface border-r border-border flex flex-col shrink-0">
              {/* Header / New Chat */}
              <div className="p-4 space-y-4">
                {/* App Logo/Title */}
                <div className="flex items-center gap-3 px-2">
                  <div className="w-8 h-8 rounded-lg bg-primary flex items-center justify-center">
                    <Sparkles size={16} className="text-primary-foreground" />
                  </div>
                  <div>
                    <h1 className="font-semibold text-sm text-foreground tracking-tight">WhytChat</h1>
                    <p className="text-[10px] text-muted-foreground">Workspace</p>
                  </div>
                </div>
                <button
                  onClick={handleNewChat}
                  className="w-full bg-primary hover:opacity-90 text-primary-foreground dark:text-zinc-900 rounded-lg p-2.5 flex items-center justify-center gap-2 transition-all text-sm font-medium shadow-sm hover:shadow-md"
                >
                  <Plus size={16} />
                  <span>{t('nav.new_chat')}</span>
                </button>
              </div>

              {/* History List */}
              <div className="flex-1 overflow-y-auto px-2 py-2 space-y-6 custom-scrollbar">
                {sessions.length > 0 || folders.length > 0 ? (
                  <>
                    {/* Favorites section */}
                    {favoriteSessions.length > 0 && (
                      <HistoryGroup
                        label={t('sessions.favorites', 'Favorites')}
                        icon={<Star size={10} className="text-yellow-500" fill="currentColor" />}
                      >
                        {favoriteItems}
                      </HistoryGroup>
                    )}

                    {/* Folders section */}
                    {folders.length > 0 && (
                      <div className="space-y-2">
                        <h3 className="text-[10px] font-semibold text-muted uppercase tracking-wider px-3 flex items-center gap-1.5">
                          <Folder size={10} />
                          {t('folders.title', 'Folders')}
                        </h3>
                        {folders.map(folder => {
                          const folderSessions = sessionsByFolder[folder.id] || [];
                          const isExpanded = expandedFolders[folder.id];
                          return (
                            <div key={folder.id} className="space-y-1">
                              {/* Folder header */}
                              <button
                                onClick={() => toggleFolderExpanded(folder.id)}
                                className="w-full flex items-center gap-2 px-3 py-2 rounded-lg text-sm text-text hover:bg-surface/80 transition-colors group"
                              >
                                {isExpanded ? <ChevronDown size={14} className="text-muted shrink-0" /> : <ChevronRight size={14} className="text-muted shrink-0" />}
                                <span className="w-3 h-3 rounded-full shrink-0" style={{ backgroundColor: folder.color || '#6366f1' }} />
                                <span className="truncate flex-1 text-left">{folder.name}</span>
                                <div className="flex items-center gap-1">
                                  <span className="text-[10px] text-muted bg-background px-1.5 py-0.5 rounded">
                                    {folderSessions.length}
                                  </span>
                                  <button
                                    onClick={(e) => {
                                      e.stopPropagation();
                                      toast((t) => (
                                        <div className="flex flex-col gap-2 min-w-[200px]">
                                          <span className="font-medium text-sm">Delete folder &quot;{folder.name}&quot;?</span>
                                          <div className="flex gap-2 justify-end">
                                            <button
                                              className="px-3 py-1.5 bg-surface border border-border rounded-lg text-xs hover:bg-background transition-colors"
                                              onClick={() => toast.dismiss(t.id)}
                                            >
                                              Cancel
                                            </button>
                                            <button
                                              className="px-3 py-1.5 bg-destructive text-destructive-foreground rounded-lg text-xs hover:opacity-90 transition-colors"
                                              onClick={() => {
                                                handleDeleteFolder(folder.id);
                                                toast.dismiss(t.id);
                                              }}
                                            >
                                              Delete
                                            </button>
                                          </div>
                                        </div>
                                      ), { duration: 5000, icon: '🗑️' });
                                    }}
                                    className="p-1 text-muted hover:text-red-400 transition-colors opacity-0 group-hover:opacity-100"
                                  >
                                    <Trash2 size={12} />
                                  </button>
                                </div>
                              </button>
                              {/* Folder contents */}
                              {isExpanded && folderSessions.length > 0 && (
                                <div className="pl-4 space-y-1">
                                  {folderSessions.map(session => (
                                    <SessionItem
                                      key={session.id}
                                      session={session}
                                      active={session.id === currentSessionId}
                                      onClick={() => handleSessionSelect(session.id)}
                                    />
                                  ))}
                                </div>
                              )}
                              {isExpanded && folderSessions.length === 0 && (
                                <p className="pl-8 text-xs text-muted/50 py-1">{t('folders.empty', 'No sessions')}</p>
                              )}
                            </div>
                          );
                        })}
                      </div>
                    )}

                    {/* Unfiled sessions section */}
                    {sessionsByFolder.unfiled.filter(s => !s.is_favorite).length > 0 && (
                      <HistoryGroup label={t('sessions.group')}>
                        {sessionsByFolder.unfiled.filter(s => !s.is_favorite).map(session => (
                          <SessionItem
                            key={session.id}
                            session={session}
                            active={session.id === currentSessionId}
                            onClick={() => handleSessionSelect(session.id)}
                          />
                        ))}
                      </HistoryGroup>
                    )}
                  </>
                ) : (
                  <div className="text-center text-muted text-sm mt-10 px-4">
                    <div className="w-16 h-16 rounded-2xl bg-background/50 flex items-center justify-center mx-auto mb-3">
                      <MessageSquare size={24} className="text-muted/30" />
                    </div>
                    <p className="text-muted/60">{t('sessions.empty')}</p>
                  </div>
                )}

                {/* Create folder button / input */}
                <div className="mt-4 px-1">
                  {isCreatingFolder ? (
                    <div className="flex gap-2">
                      <input
                        type="text"
                        value={newFolderName}
                        onChange={(e) => setNewFolderName(e.target.value)}
                        onKeyDown={(e) => {
                          if (e.key === 'Enter') handleCreateFolder();
                          if (e.key === 'Escape') { setIsCreatingFolder(false); setNewFolderName(''); }
                        }}
                        placeholder={t('folders.name_placeholder', 'Folder name...')}
                        className="flex-1 px-2 py-1.5 text-sm bg-background border border-border rounded-lg focus:outline-none focus:ring-1 focus:ring-primary"
                        autoFocus
                      />
                      <button
                        onClick={handleCreateFolder}
                        className="px-3 py-1.5 text-sm bg-primary text-primary-foreground rounded-lg hover:opacity-90 transition-colors"
                      >
                        {t('common.save', 'Save')}
                      </button>
                    </div>
                  ) : (
                    <button
                      onClick={() => setIsCreatingFolder(true)}
                      className="w-full flex items-center gap-2 px-3 py-2 text-sm text-muted hover:text-text hover:bg-surface/80 rounded-lg transition-colors"
                    >
                      <FolderPlus size={14} />
                      {t('folders.create', 'New folder')}
                    </button>
                  )}
                </div>
              </div>
            </aside>
        )}

        {/* Center Content (Chat) */}
        <main className="flex-1 flex flex-col h-full relative bg-background min-w-0">
          {/* Minimal Header */}
          <header className="h-14 flex items-center justify-between px-4 border-b border-border bg-surface z-40 relative">
            <div className="flex items-center gap-3">
              <button
                onClick={toggleSidebar}
                className={cn(
                  'p-2 rounded-xl transition-all',
                  isSidebarOpen
                    ? 'text-primary bg-primary/10'
                    : 'text-muted hover:text-text hover:bg-surface/80'
                )}
                title={isSidebarOpen ? t('sidebar.hide') : t('sidebar.show')}
              >
                <PanelLeft size={18} />
              </button>
              <div className="w-px h-6 bg-border" />
              <div className="flex items-center gap-2 px-3 py-1.5 bg-background rounded-lg border border-border">
                <div className="w-2 h-2 rounded-full bg-success animate-pulse" />
                <span className="font-medium text-sm text-text">
                  {currentSessionDisplay}
                </span>
              </div>
            </div>
            {/* Header Actions (Files, Settings, Theme) */}
            <HeaderActions />
          </header>

          {/* Chat Content */}
          {children}

          {/* Knowledge View Overlay */}
          {currentView === 'knowledge' && (
            <div className="absolute inset-0 z-50 bg-background animate-in slide-in-from-bottom-4 fade-in duration-200">
              <KnowledgeView />
            </div>
          )}
        </main>
      </div>

      {/* Session Creation Wizard */}
      {isCreatingSession && (
        <SessionWizard onClose={() => setIsCreatingSession(false)} />
      )}

      {/* Toast container for notifications */}
      <Toaster
        position="bottom-right"
        toastOptions={{
          className: 'font-sans',
          duration: 5000,
          style: {
            background: 'hsl(var(--surface))',
            color: 'hsl(var(--text))',
            border: '1px solid hsl(var(--border))',
            boxShadow: '0 10px 25px -5px rgb(0 0 0 / 0.2)',
            borderRadius: '12px',
          },
          error: {
            duration: 8000,
            iconTheme: {
              primary: 'hsl(var(--destructive))',
              secondary: 'hsl(var(--surface))',
            },
          },
        }}
      />
    </div>
  );
}
