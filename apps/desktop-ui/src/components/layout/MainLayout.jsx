import React, { useEffect, useCallback, useMemo } from 'react';
import { useAppStore } from '../../store/appStore';
import { Plus, MessageSquare } from 'lucide-react';
import { cn } from '../../lib/utils';
import { useTranslation } from 'react-i18next';
import { Toaster } from 'react-hot-toast';
import { RightPanel } from './RightPanel';

const HistoryGroup = React.memo(function HistoryGroup({ label, children }) {
  return React.createElement(
    'div',
    { className: 'space-y-2' },
    React.createElement(
      'h3',
      { className: 'text-[10px] font-bold text-muted uppercase tracking-wider px-3' },
      label
    ),
    React.createElement(
      'div',
      { className: 'space-y-1' },
      children
    )
  );
});

const HistoryItem = React.memo(function HistoryItem({ label, active, icon, onClick }) {
  return React.createElement(
    'button',
    {
      onClick: onClick,
      className: cn(
        'w-full flex items-center gap-3 px-3 py-2.5 rounded-xl text-sm transition-all text-left group',
        active ? 'bg-white shadow-sm ring-1 ring-border text-text' : 'text-muted hover:bg-white/50 hover:text-text'
      )
    },
    React.createElement(
      'span',
      {
        className: cn('shrink-0', active ? 'text-primary' : 'text-muted group-hover:text-text')
      },
      icon || React.createElement(MessageSquare, { size: 16 })
    ),
    React.createElement(
      'span',
      { className: 'truncate' },
      label
    )
  );
});

export function MainLayout({ children }) {
  const {
    isBackendInitialized,
    isBackendInitializing,
    initializeApp,
    isThinking,
    thinkingSteps,
    sessions,
    currentSessionId,
    createSession,
    setCurrentSessionId,
  } = useAppStore();
  const translationResult = useTranslation('common');
  const t = translationResult.t;

  useEffect(function() {
    initializeApp();
  }, [initializeApp]);

  const handleNewChat = useCallback(() => {
    createSession('New Chat').catch(function(error) {
      console.error('Failed to create new session:', error);
    });
  }, [createSession]);

  const handleSessionSelect = useCallback((sessionId) => {
    setCurrentSessionId(sessionId);
  }, [setCurrentSessionId]);

  const getSessionDisplayName = useCallback((session) => {
    return session.title || 'Session ' + session.id.slice(-8);
  }, []);

  const currentSessionDisplay = useMemo(() => {
    return currentSessionId ? 'Session ' + currentSessionId.slice(-8) : t('chat.header.new_session');
  }, [currentSessionId, t]);

  const sessionItems = useMemo(() => sessions.map(function(session) {
    return React.createElement(HistoryItem, {
      key: session.id,
      active: session.id === currentSessionId,
      label: getSessionDisplayName(session),
      onClick: function() { return handleSessionSelect(session.id); }
    });
  }), [sessions, currentSessionId, getSessionDisplayName, handleSessionSelect]);

  if (isBackendInitializing) {
    return React.createElement(
      'div',
      { className: 'flex h-screen w-full items-center justify-center bg-background' },
      React.createElement(
        'div',
        { className: 'flex flex-col items-center gap-4' },
        React.createElement('div', { className: 'w-10 h-10 animate-spin rounded-full border-4 border-primary border-t-transparent' }),
        React.createElement('p', { className: 'text-muted' }, 'Initializing Backend...')
      )
    );
  }

  if (!isBackendInitialized) {
    return React.createElement(
      'div',
      { className: 'flex h-screen w-full items-center justify-center bg-background' },
      React.createElement(
        'div',
        { className: 'text-center' },
        React.createElement('h1', { className: 'text-2xl font-bold text-destructive mb-2' }, 'Backend Initialization Failed'),
        React.createElement('p', { className: 'text-muted' }, 'Could not connect to the backend. Please restart the application.')
      )
    );
  }


  return React.createElement(
    'div',
    { className: 'flex h-screen bg-background text-text overflow-hidden font-sans' },
    // Left Sidebar
    React.createElement(
      'aside',
      { className: 'w-72 bg-surface border-r border-border flex flex-col shrink-0' },
      // Header / New Chat
      React.createElement(
        'div',
        { className: 'p-4' },
        React.createElement(
          'button',
          {
            onClick: handleNewChat,
            className: 'w-full bg-primary hover:bg-primary/90 text-white rounded-xl p-3 flex items-center justify-center gap-2 shadow-lg shadow-primary/20 transition-all font-medium'
          },
          React.createElement(Plus, { size: 20 }),
          React.createElement('span', null, t('nav.new_chat'))
        )
      ),
      // History List
      React.createElement(
        'div',
        { className: 'flex-1 overflow-y-auto px-3 py-2 space-y-6' },
        sessions.length > 0 ? React.createElement(
          HistoryGroup,
          { label: t('sessions.group') },
          sessionItems
        ) : React.createElement(
          'div',
          { className: 'text-center text-muted text-sm mt-10' },
          React.createElement(MessageSquare, { size: 32, className: 'mx-auto mb-2 opacity-20' }),
          React.createElement('p', null, t('sessions.empty'))
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
        { className: 'h-14 flex items-center justify-between px-6 border-b border-border/50 bg-surface/50 backdrop-blur' },
        React.createElement(
          'div',
          { className: 'flex items-center gap-2' },
          React.createElement('div', { className: 'w-2 h-2 rounded-full bg-green-500' }),
          React.createElement(
            'span',
            { className: 'font-medium text-sm text-text' },
            currentSessionDisplay
          )
        )
      ),
      children
    ),
    // Right Panel (Orchestration)
    React.createElement(RightPanel, { isThinking: isThinking, thinkingSteps: thinkingSteps }),
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
          boxShadow: '0 4px 6px -1px rgb(0 0 0 / 0.1), 0 2px 4px -2px rgb(0 0 0 / 0.1)',
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
