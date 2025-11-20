import { create } from "zustand";
import { persist } from "zustand/middleware";

export const useLearningStore = create(
  persist(
    (set, get) => ({
      // Stocke les retours d'expérience sous forme de { id, agentName, content, isGood, timestamp }
      insights: [],

      addInsight: (agentName, content, isGood) => {
        set((state) => ({
          insights: [
            {
              id: Date.now().toString(),
              agentName,
              content,
              isGood,
              timestamp: Date.now(),
            },
            ...state.insights,
          ],
        }));
      },

      // Récupère les X derniers bons exemples pour un contexte donné (simplifié ici pour tous les agents)
      getRelevantInsights: (limit = 3) => {
        return get()
          .insights.filter((i) => i.isGood)
          .slice(0, limit)
          .map((i) => `[Exemple de bon raisonnement par ${i.agentName}]: ${i.content}`)
          .join("\n\n");
      },

      clearInsights: () => set({ insights: [] }),
    }),
    {
      name: "whytchat_learning_context",
    }
  )
);