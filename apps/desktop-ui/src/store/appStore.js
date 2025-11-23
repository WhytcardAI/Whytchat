import { create } from 'zustand';

// Global app state store
export const useAppStore = create((set) => ({
  // UI State
  isSidebarOpen: true,
  toggleSidebar: function() { return set(function(state) { return ({ isSidebarOpen: !state.isSidebarOpen }) }) },

  // Session State (Placeholder)
  currentSessionId: null,
  setCurrentSessionId: function(id) { return set({ currentSessionId: id }) },

  // Thinking State (Orchestration)
  isThinking: false,
  thinkingSteps: [],
  setThinking: function(isThinking) { return set({ isThinking: isThinking }) },
  addThinkingStep: function(step) { return set((state) => ({ thinkingSteps: [...state.thinkingSteps, step] })) },
  clearThinkingSteps: function() { return set({ thinkingSteps: [] }) },

  // Onboarding & Configuration
  isConfigured: false, // Set to true once model is downloaded
  completeOnboarding: function() { return set({ isConfigured: true }) },
}));
