import { create } from 'zustand';

export const useAppStore = create((set) => ({
  // UI State
  isSidebarOpen: true,
  toggleSidebar: () => set((state) => ({ isSidebarOpen: !state.isSidebarOpen })),

  // Session State (Placeholder)
  currentSessionId: null,
  setCurrentSessionId: (id) => set({ currentSessionId: id }),

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
