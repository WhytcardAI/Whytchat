import { create } from 'zustand';
import { invoke } from '@tauri-apps/api/core';
import { toast } from 'react-hot-toast';

// Global app state store
export const useAppStore = create(function(set, get) {
  return {
    // UI State
    isSidebarOpen: true,
    toggleSidebar: function() {
      return set(function(state) {
        return { isSidebarOpen: !state.isSidebarOpen };
      });
    },

    // Session State
    sessions: [],
    currentSessionId: null,
    setCurrentSessionId: function(id) {
      return set({ currentSessionId: id });
    },
    setSessions: function(sessions) {
      return set({ sessions: sessions });
    },

    // Load sessions from backend
    loadSessions: function() {
      return new Promise(function(resolve, reject) {
        invoke('get_all_sessions').then(function(sessions) {
          set({ sessions: sessions });
          resolve();
        }).catch(function(error) {
          console.error('Failed to load sessions:', error);
          get().showError('Failed to load sessions.');
          reject(error);
        });
      });
    },

    // Create new session
    createSession: function(title) {
      return new Promise(function(resolve, reject) {
        invoke('create_session', { title: title }).then(function(sessionId) {
          // Reload sessions to get the new one
          get().loadSessions().then(function() {
            set({ currentSessionId: sessionId });
            resolve(sessionId);
          }).catch(reject);
        }).catch(function(error) {
          console.error('Failed to create session:', error);
          get().showError('Failed to create session.');
          reject(error);
        });
      });
    },

    updateSession: function(sessionId, title, modelConfig) {
      return new Promise(function(resolve, reject) {
        invoke('update_session', {
          sessionId: sessionId,
          title: title,
          modelConfig: modelConfig,
        }).then(function() {
          // Reload sessions to get updated data
          get().loadSessions().then(resolve).catch(reject);
        }).catch(function(error) {
          console.error('Failed to update session:', error);
          get().showError('Failed to update session.');
          reject(error);
        });
      });
    },

    // Thinking State (Orchestration)
    isThinking: false,
    thinkingSteps: [],
    setThinking: function(isThinking) {
      return set({ isThinking: isThinking });
    },
    addThinkingStep: function(step) {
      return set(function(state) {
        return { thinkingSteps: state.thinkingSteps.concat([step]) };
      });
    },
    clearThinkingSteps: function() {
      return set({ thinkingSteps: [] });
    },

    // Onboarding & Configuration
    isConfigured: false, // Set to true once model is downloaded
    completeOnboarding: function() {
      return set({ isConfigured: true });
    },

    // Backend initialization state
    isBackendInitialized: false,
    isBackendInitializing: true, // Assume initializing at start
    initializeApp: async function() {
      try {
        set({ isBackendInitializing: true });
        await invoke('initialize_app');
        set({ isBackendInitialized: true, isBackendInitializing: false });
      } catch (error) {
        console.error('Failed to initialize backend:', error);
        set({ isBackendInitializing: false }); // Stop loading on error
        get().showError('Failed to initialize backend. Please restart the application.');
      }
    },

    // Error Handling
    error: null,
    setError: function(error) {
      return set({ error: error });
    },
    showError: function(message) {
      toast.error(message);
      set({ error: message });
    },
  };
});
