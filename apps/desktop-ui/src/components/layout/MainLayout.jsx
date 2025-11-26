import React, { useEffect, useCallback, useMemo, useState } from 'react';
import { useAppStore } from '../../store/appStore';
import { Plus, MessageSquare, PanelLeft, Sparkles, Star, FolderPlus, ChevronDown, ChevronRight, Folder, Trash2 } from 'lucide-react';
import { cn } from '../../lib/utils';
import { useTranslation } from 'react-i18next';
import { Toaster } from 'react-hot-toast';
import { SessionWizard } from '../onboarding/SessionWizard';
import { HeaderActions } from './HeaderActions';
import { SessionItem } from './SessionItem';
import { DataSidebar } from './DataSidebar';

const HistoryGroup = React.memo(function HistoryGroup({ label, children, icon }) {
  return React.createElement(
    'div',
    { className: 'space-y-2' },
    React.createElement(
      'h3',
      { className: 'text-[10px] font-semibold text-muted uppercase tracking-wider px-3 flex items-center gap-1.5' },
      icon,
      label
    ),
    React.createElement(
      'div',
      { className: 'space-y-1' },
      children
    )
  );
});

export function MainLayout({ children, isCreatingSession, setIsCreatingSession }) {
  const {
    isBackendInitialized,
    isBackendInitializing,
    initializeApp,
    sessions,
    currentSessionId,
    setCurrentSessionId,
    isSidebarOpen,
    toggleSidebar,
    isRightSidebarOpen,
    loadFolders,
    folders,
    createFolder,
    deleteFolder
  } = useAppStore();
  const translationResult = useTranslation('common');
  const { t } = translationResult;

  const [isCreatingFolder, setIsCreatingFolder] = useState(false);
  const [newFolderName, setNewFolderName] = useState('');
  const [expandedFolders, setExpandedFolders] = useState({});

  useEffect(function() {
    initializeApp();
    loadFolders();
  }, [initializeApp, loadFolders]);

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

  const favoriteItems = useMemo(() => favoriteSessions.map(function(session) {
    return React.createElement(SessionItem, {
      key: session.id,
      session: session,
      active: session.id === currentSessionId,
      onClick: function() { return handleSessionSelect(session.id); }
    });
  }), [favoriteSessions, currentSessionId, handleSessionSelect]);

  if (isBackendInitializing) {
    return React.createElement(
      'div',
      { className: 'flex h-screen w-full items-center justify-center bg-background' },
      React.createElement(
        'div',
        { className: 'flex flex-col items-center gap-4' },
        React.createElement(
          'div',
          { className: 'relative' },
          React.createElement('div', { className: 'w-16 h-16 rounded-2xl bg-primary/20 flex items-center justify-center' },
            React.createElement(Sparkles, { size: 32, className: 'text-primary animate-pulse' })
          ),
          React.createElement('div', { className: 'absolute inset-0 w-16 h-16 rounded-2xl border-2 border-primary/30 animate-ping' })
        ),
        React.createElement('p', { className: 'text-muted text-sm' }, t('app.initializing', 'Initializing Backend...'))
      )
    );
  }

  if (!isBackendInitialized) {
    return React.createElement(
      'div',
      { className: 'flex h-screen w-full items-center justify-center bg-background' },
      React.createElement(
        'div',
        { className: 'text-center p-8 bg-surface/80 rounded-2xl border border-destructive/20 max-w-md' },
        React.createElement('div', { className: 'w-16 h-16 rounded-2xl bg-destructive/10 flex items-center justify-center mx-auto mb-4' },
          React.createElement('span', { className: 'text-destructive text-3xl' }, '!')
        ),
        React.createElement('h1', { className: 'text-xl font-bold text-destructive mb-2' }, t('app.init_failed', 'Backend Initialization Failed')),
        React.createElement('p', { className: 'text-muted text-sm' }, t('app.init_failed_desc', 'Could not connect to the backend. Please restart the application.'))
      )
    );
  }


  return React.createElement(
    'div',
    { className: 'flex h-screen bg-background text-text overflow-hidden font-sans' },
    // Left Sidebar
    isSidebarOpen && React.createElement(
      'aside',
      { className: 'w-72 bg-surface/80 backdrop-blur-xl border-r border-border flex flex-col shrink-0' },
      // Header / New Chat
      React.createElement(
        'div',
        { className: 'p-4 space-y-3' },
        // App Logo/Title
        React.createElement(
          'div',
          { className: 'flex items-center gap-3 px-1 mb-2' },
          React.createElement(
            'div',
            { className: 'w-10 h-10 rounded-xl bg-gradient-to-br from-primary to-accent flex items-center justify-center shadow-lg shadow-primary/20' },
            React.createElement(Sparkles, { size: 20, className: 'text-white' })
          ),
          React.createElement(
            'div',
            null,
            React.createElement('h1', { className: 'font-bold text-text' }, 'WhytChat'),
            React.createElement('p', { className: 'text-[10px] text-muted' }, 'AI Assistant')
          )
        ),
        React.createElement(
          'button',
          {
            onClick: handleNewChat,
            className: 'w-full bg-primary hover:bg-primary/90 text-white rounded-xl p-3 flex items-center justify-center gap-2 shadow-lg shadow-primary/20 transition-all font-medium hover:scale-[1.02] active:scale-[0.98]'
          },
          React.createElement(Plus, { size: 20 }),
          React.createElement('span', null, t('nav.new_chat'))
        )
      ),
      // History List
      React.createElement(
        'div',
        { className: 'flex-1 overflow-y-auto px-3 py-2 space-y-4 custom-scrollbar' },
        sessions.length > 0 || folders.length > 0 ? React.createElement(
          React.Fragment,
          null,
          // Favorites section
          favoriteSessions.length > 0 && React.createElement(
            HistoryGroup,
            {
              label: t('sessions.favorites', 'Favorites'),
              icon: React.createElement(Star, { size: 10, className: 'text-yellow-500', fill: 'currentColor' })
            },
            favoriteItems
          ),
          // Folders section
          folders.length > 0 && React.createElement(
            'div',
            { className: 'space-y-2' },
            React.createElement(
              'h3',
              { className: 'text-[10px] font-semibold text-muted uppercase tracking-wider px-3 flex items-center gap-1.5' },
              React.createElement(Folder, { size: 10 }),
              t('folders.title', 'Folders')
            ),
            folders.map(function(folder) {
              var folderSessions = sessionsByFolder[folder.id] || [];
              var isExpanded = expandedFolders[folder.id];
              return React.createElement(
                'div',
                { key: folder.id, className: 'space-y-1' },
                // Folder header
                React.createElement(
                  'button',
                  {
                    onClick: function() { toggleFolderExpanded(folder.id); },
                    className: 'w-full flex items-center gap-2 px-3 py-2 rounded-lg text-sm text-text hover:bg-surface/80 transition-colors group'
                  },
                  React.createElement(
                    isExpanded ? ChevronDown : ChevronRight,
                    { size: 14, className: 'text-muted shrink-0' }
                  ),
                  React.createElement(
                    'span',
                    { className: 'w-3 h-3 rounded-full shrink-0', style: { backgroundColor: folder.color || '#6366f1' } }
                  ),
                  React.createElement('span', { className: 'truncate flex-1 text-left' }, folder.name),
                  React.createElement(
                    'div',
                    { className: 'flex items-center gap-1' },
                    React.createElement(
                      'span',
                      { className: 'text-[10px] text-muted bg-background px-1.5 py-0.5 rounded' },
                      folderSessions.length
                    ),
                    React.createElement(
                      'button',
                      {
                        onClick: function(e) {
                          e.stopPropagation();
                          if (confirm(t('folders.delete_confirm', 'Delete this folder?'))) {
                            handleDeleteFolder(folder.id);
                          }
                        },
                        className: 'p-1 text-muted hover:text-red-400 transition-colors opacity-0 group-hover:opacity-100'
                      },
                      React.createElement(Trash2, { size: 12 })
                    )
                  )
                ),
                // Folder contents
                isExpanded && folderSessions.length > 0 && React.createElement(
                  'div',
                  { className: 'pl-4 space-y-1' },
                  folderSessions.map(function(session) {
                    return React.createElement(SessionItem, {
                      key: session.id,
                      session: session,
                      active: session.id === currentSessionId,
                      onClick: function() { handleSessionSelect(session.id); }
                    });
                  })
                ),
                isExpanded && folderSessions.length === 0 && React.createElement(
                  'p',
                  { className: 'pl-8 text-xs text-muted/50 py-1' },
                  t('folders.empty', 'No sessions')
                )
              );
            })
          ),
          // Unfiled sessions section (not in any folder and not favorites)
          sessionsByFolder.unfiled.filter(function(s) { return !s.is_favorite; }).length > 0 && React.createElement(
            HistoryGroup,
            { label: t('sessions.group') },
            sessionsByFolder.unfiled.filter(function(s) { return !s.is_favorite; }).map(function(session) {
              return React.createElement(SessionItem, {
                key: session.id,
                session: session,
                active: session.id === currentSessionId,
                onClick: function() { handleSessionSelect(session.id); }
              });
            })
          )
        ) : React.createElement(
          'div',
          { className: 'text-center text-muted text-sm mt-10 px-4' },
          React.createElement(
            'div',
            { className: 'w-16 h-16 rounded-2xl bg-background/50 flex items-center justify-center mx-auto mb-3' },
            React.createElement(MessageSquare, { size: 24, className: 'text-muted/30' })
          ),
          React.createElement('p', { className: 'text-muted/60' }, t('sessions.empty'))
        ),
        // Create folder button / input
        React.createElement(
          'div',
          { className: 'mt-4 px-1' },
          isCreatingFolder ? React.createElement(
            'div',
            { className: 'flex gap-2' },
            React.createElement('input', {
              type: 'text',
              value: newFolderName,
              onChange: function(e) { setNewFolderName(e.target.value); },
              onKeyDown: function(e) {
                if (e.key === 'Enter') handleCreateFolder();
                if (e.key === 'Escape') { setIsCreatingFolder(false); setNewFolderName(''); }
              },
              placeholder: t('folders.name_placeholder', 'Folder name...'),
              className: 'flex-1 px-2 py-1.5 text-sm bg-background border border-border rounded-lg focus:outline-none focus:ring-1 focus:ring-primary',
              autoFocus: true
            }),
            React.createElement(
              'button',
              {
                onClick: handleCreateFolder,
                className: 'px-3 py-1.5 text-sm bg-primary text-white rounded-lg hover:bg-primary/90 transition-colors'
              },
              t('common.save', 'Save')
            )
          ) : React.createElement(
            'button',
            {
              onClick: function() { setIsCreatingFolder(true); },
              className: 'w-full flex items-center gap-2 px-3 py-2 text-sm text-muted hover:text-text hover:bg-surface/80 rounded-lg transition-colors'
            },
            React.createElement(FolderPlus, { size: 14 }),
            t('folders.create', 'New folder')
          )
        )
      )
    ),
    // Center Content (Chat)
    React.createElement(
      'main',
      { className: 'flex-1 flex flex-col h-full relative bg-background min-w-0' },
      // Minimal Header
      React.createElement(
        'header',
        { className: 'h-14 flex items-center justify-between px-4 border-b border-border/50 bg-surface/50 backdrop-blur-xl z-40 relative' },
        React.createElement(
          'div',
          { className: 'flex items-center gap-3' },
          React.createElement(
            'button',
            {
              onClick: toggleSidebar,
              className: cn(
                'p-2 rounded-xl transition-all',
                isSidebarOpen
                  ? 'text-primary bg-primary/10'
                  : 'text-muted hover:text-text hover:bg-surface/80'
              ),
              title: isSidebarOpen ? t('sidebar.hide') : t('sidebar.show')
            },
            React.createElement(PanelLeft, { size: 18 })
          ),
          React.createElement('div', { className: 'w-px h-6 bg-border' }),
          React.createElement(
            'div',
            { className: 'flex items-center gap-2 px-3 py-1.5 bg-background/50 rounded-lg border border-border/50' },
            React.createElement('div', { className: 'w-2 h-2 rounded-full bg-success animate-pulse' }),
            React.createElement(
              'span',
              { className: 'font-medium text-sm text-text' },
              currentSessionDisplay
            )
          )
        ),
        // Header Actions (Files, Settings, Theme)
        React.createElement(HeaderActions)
      ),
      children
    ),
    // Right Sidebar (Data)
    isRightSidebarOpen && React.createElement(DataSidebar),
    // Session Creation Wizard
    isCreatingSession && React.createElement(SessionWizard, {
      onClose: function() { return setIsCreatingSession(false); }
    }),
    // Toast container for notifications
    React.createElement(Toaster, {
      position: 'bottom-right',
      toastOptions: {
        // Define default options
        className: 'font-sans',
        duration: 5000,
        style: {
          background: 'hsl(var(--surface))',
          color: 'hsl(var(--text))',
          border: '1px solid hsl(var(--border))',
          boxShadow: '0 10px 25px -5px rgb(0 0 0 / 0.2)',
          borderRadius: '12px',
        },

        // Default options for specific types
        error: {
          duration: 8000, // Errors are more important
          iconTheme: {
            primary: 'hsl(var(--destructive))',
            secondary: 'hsl(var(--surface))',
          },
        },
      },
    })
  );
}
