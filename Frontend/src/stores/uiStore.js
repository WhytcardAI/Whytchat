import { create } from 'zustand';
import { persist } from 'zustand/middleware';

export const useUIStore = create(
  persist(
    (set) => ({
      isSidebarOpen: true,
      isContextPanelOpen: true,
      toggleSidebar: () => set((state) => ({ isSidebarOpen: !state.isSidebarOpen })),
      toggleContextPanel: () => set((state) => ({ isContextPanelOpen: !state.isContextPanelOpen })),
      setSidebarOpen: (isOpen) => set({ isSidebarOpen: isOpen }),
      setContextPanelOpen: (isOpen) => set({ isContextPanelOpen: isOpen }),
    }),
    {
      name: 'whytchat-ui-storage',
    }
  )
);