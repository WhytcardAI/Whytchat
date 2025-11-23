import { create } from 'zustand';
import { invoke } from '@tauri-apps/api/core';

// Global app state store
export const useAppStore = create((set) => ({
  // UI State
  isSidebarOpen: true,
  toggleSidebar: () => set((state) => ({ isSidebarOpen: !state.isSidebarOpen })),

  // Session State (Placeholder)
  currentSessionId: null,
  sessions: [],
  setCurrentSessionId: (id) => set({ currentSessionId: id }),
  setSessions: (sessions) => set({ sessions }),

  // Load sessions from backend
  loadSessions: async () => {
    try {
      const sessions = await invoke('list_sessions');
      set({ sessions });
    } catch (error) {
      console.error('Failed to load sessions:', error);
    }
  },

  // Create new session
  createSession: async () => {
    try {
      const sessionId = await invoke('create_session');
      set((state) => ({
        sessions: [...state.sessions, { id: sessionId, created_at: new Date().toISOString() }],
        currentSessionId: sessionId
      }));
      return sessionId;
    } catch (error) {
      console.error('Failed to create session:', error);
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
