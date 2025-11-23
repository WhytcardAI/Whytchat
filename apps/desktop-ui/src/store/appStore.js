import { create } from 'zustand';
import { invoke } from '@tauri-apps/api/core';

// Global app state store
export const useAppStore = create((set, get) => ({
  // UI State
  isSidebarOpen: true,
  toggleSidebar: () => set((state) => ({ isSidebarOpen: !state.isSidebarOpen })),

  // Session State
  sessions: [],
  currentSessionId: null,
  setCurrentSessionId: (id) => set({ currentSessionId: id }),

  // Session Management Actions
  createSession: (title) => new Promise(async (resolve, reject) => {
    try {
      const sessionId = await invoke('create_session', { title: title });
      // Reload sessions to get the new one
      await get().loadSessions();
      set({ currentSessionId: sessionId });
      return sessionId;
    } catch (error) {
      console.error('Failed to create session:', error);
      throw error;
    }
  },

  loadSessions: async () => {
    try {
      const sessions = await invoke('get_all_sessions');
      set({ sessions });
      // If no current session, set to the first one
      if (!get().currentSessionId && sessions.length > 0) {
        set({ currentSessionId: sessions[0].id });
      }
    } catch (error) {
      console.error('Failed to load sessions:', error);
    }
  },

  updateSession: async function(sessionId, title, modelConfig) {
    try {
      await invoke('update_session', {
        sessionId,
        title,
        modelConfig,
      });
      // Reload sessions to get updated data
      await get().loadSessions();
    } catch (error) {
      console.error('Failed to update session:', error);
      throw error;
    }
  },

  // Thinking State (Orchestration)
  isThinking: false,
  thinkingSteps: [],
  setThinking: (isThinking) => set({ isThinking }),
  addThinkingStep: (step) => set((state) => ({ thinkingSteps: [...state.thinkingSteps, step] })),
  clearThinkingSteps: () => set({ thinkingSteps: [] }),

  // Onboarding & Configuration
  isConfigured: false, // Set to true once model is downloaded
  completeOnboarding: () => set({ isConfigured: true }),
}));
