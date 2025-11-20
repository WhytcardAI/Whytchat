import { create } from "zustand";
import { persist } from "zustand/middleware";

export const useSettingsStore = create(
  persist(
    (set) => ({
      // Général
      language: "fr",
      autoStart: false,
      notifications: true,

      // Modèle IA
      temperature: 0.7,
      maxTokens: 4096,
      contextWindow: 16384,

      // Serveur
      serverPort: 8080,
      gpuLayers: 33,
      autoStartServer: true,

      // Apparence (force dark au lieu de light si ancienne sauvegarde)
      theme:
        typeof window !== "undefined" && localStorage.getItem("whytchat_settings")
          ? JSON.parse(localStorage.getItem("whytchat_settings")).state?.theme || "dark"
          : "dark",
      fontSize: 14, // px
      animations: true,

      // Confidentialité
      saveHistory: true,
      autoDelete: "never", // "never", "7days", "30days", "90days"

      // Actions
      setLanguage: (language) => set({ language }),
      setAutoStart: (autoStart) => set({ autoStart }),
      setNotifications: (notifications) => set({ notifications }),
      setTemperature: (temperature) => set({ temperature }),
      setMaxTokens: (maxTokens) => set({ maxTokens }),
      setContextWindow: (contextWindow) => set({ contextWindow }),
      setServerPort: (serverPort) => set({ serverPort }),
      setGpuLayers: (gpuLayers) => set({ gpuLayers }),
      setAutoStartServer: (autoStartServer) => set({ autoStartServer }),
      setTheme: (theme) => set({ theme }),
      setFontSize: (fontSize) => set({ fontSize }),
      setAnimations: (animations) => set({ animations }),
      setSaveHistory: (saveHistory) => set({ saveHistory }),
      setAutoDelete: (autoDelete) => set({ autoDelete }),
    }),
    {
      name: "whytchat_settings",
    }
  )
);
