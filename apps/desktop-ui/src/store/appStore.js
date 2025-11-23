import { create } from 'zustand';
import { invoke } from '@tauri-apps/api/core';

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
        try {
          invoke('get_all_sessions').then(function(sessions) {
            set({ sessions: sessions });
            // If no current session, set to the first one
            if (!get().currentSessionId && sessions.length > 0) {
              set({ currentSessionId: sessions[0].id });
            }
            resolve();
          }).catch(function(error) {
            console.error('Failed to load sessions:', error);
            reject(error);
          });
        } catch (error) {
          console.error('Failed to load sessions:', error);
          reject(error);
        }
      });
    },

    // Create new session
    createSession: function(title) {
      return new Promise(function(resolve, reject) {
        try {
          invoke('create_session', { title: title }).then(function(sessionId) {
            // Reload sessions to get the new one
            get().loadSessions().then(function() {
              set({ currentSessionId: sessionId });
              resolve(sessionId);
            }).catch(reject);
          }).catch(function(error) {
            console.error('Failed to create session:', error);
            reject(error);
          });
        } catch (error) {
          console.error('Failed to create session:', error);
          reject(error);
        }
      });
    },

    updateSession: function(sessionId, title, modelConfig) {
      return new Promise(function(resolve, reject) {
        try {
          invoke('update_session', {
            sessionId: sessionId,
            title: title,
            modelConfig: modelConfig,
          }).then(function() {
            // Reload sessions to get updated data
            get().loadSessions().then(resolve).catch(reject);
          }).catch(function(error) {
            console.error('Failed to update session:', error);
            reject(error);
          });
        } catch (error) {
          console.error('Failed to update session:', error);
          reject(error);
        }
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
  };
});
