import { create } from "zustand";
import { persist } from "zustand/middleware";

export const useKnowledgeStore = create(
  persist(
    (set, get) => ({
      documents: [], // Liste des documents indexés { id, name, path, indexedAt, status }
      isIndexing: false,

      addDocument: (doc) => {
        set((state) => ({
          documents: [
            ...state.documents,
            { ...doc, id: Date.now().toString(), indexedAt: Date.now(), status: "ready" },
          ],
        }));
      },

      setIndexing: (status) => set({ isIndexing: status }),

      removeDocument: (id) => {
        set((state) => ({
          documents: state.documents.filter((d) => d.id !== id),
        }));
      },
    }),
    {
      name: "whytchat_knowledge_base",
    }
  )
);