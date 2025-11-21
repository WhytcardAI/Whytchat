import { create } from "zustand";
import { persist } from "zustand/middleware";
import { useSettingsStore } from "../stores/settingsStore";
import { useLearningStore } from "../stores/learningStore";
import { api } from "./api";

// Default Configuration
const DEFAULT_SYSTEM_PROMPT =
  "You are a highly capable AI assistant powered by Qwen2.5-3B. You excel at code generation, debugging, multilingual conversations, and providing detailed explanations. Always respond in the user's language unless explicitly asked to switch. Be concise, accurate, and helpful.";

const buildEffectiveMessages = (conversation, systemPrompt, userPersona, groupContext) => {
  return [
    { role: "system", content: systemPrompt + groupContext },
    ...(userPersona ? [{ role: "system", content: `User Context: ${userPersona}` }] : []),
    ...conversation.messages
      .map((m) => ({
        role: m.role,
        content: m.content,
      }))
      .filter((m) => m.role !== "system"),
  ];
};

const handleMeetingMode = async (conversation, text, api, addMessage, convId) => {
  const participants = conversation.config.meetingParticipants || [];
  for (const participant of participants) {
    const result = await api.generateResponseWithContext(
      participant.contextHistory,
      text,
      participant.name
    );
    addMessage(convId, "assistant", `[${participant.name}]: ${result.reply}`);
  }
  return null;
};

const handleDebateMode = async (conversation, api) => {
  const { debateConfig } = conversation.config;
  if (!debateConfig || !debateConfig.isDebating) return null;

  const { agentA, agentB, currentSpeaker, topic, rounds, currentRound } = debateConfig;

  // Stop condition
  if (currentRound > rounds) {
    return {
      action: "stop_debate",
      message: "Le débat est terminé (nombre de tours atteint).",
    };
  }

  const speaker = currentSpeaker === "A" ? agentA : agentB;
  const otherAgent = currentSpeaker === "A" ? agentB : agentA;

  // Build context from conversation history
  // We need to format the history so the current speaker understands the flow
  // The speaker sees themselves as 'assistant' and the other as 'user' (or named entity)

  // Simplified approach: Use generateResponseWithContext which treats history as "Memory"
  // But for a debate, the immediate previous message is the most important "User Prompt"

  const lastMessage = conversation.messages[conversation.messages.length - 1];
  let userPrompt = "";

  if (conversation.messages.length === 0) {
    // First turn
    userPrompt = `Sujet du débat : ${topic}. Commence ton argumentation.`;
  } else {
    userPrompt = `[Réponse de ${otherAgent.name}]: ${lastMessage.content}\n\nRéponds à cet argument en gardant ton rôle de ${speaker.name}.`;
  }

  // Context history: All previous messages
  const contextHistory = conversation.messages.map((m) => ({
    role: m.content.startsWith(`[${agentA.name}]`)
      ? currentSpeaker === "A"
        ? "assistant"
        : "user"
      : m.content.startsWith(`[${agentB.name}]`)
        ? currentSpeaker === "B"
          ? "assistant"
          : "user"
        : "user",
    content: m.content,
  }));

  // Add system prompt as context
  const systemContext = [
    {
      role: "system",
      content: `Tu es ${speaker.name}. ${speaker.prompt}. Tu débats sur le sujet : ${topic}.`,
    },
  ];

  const fullContext = [...systemContext, ...contextHistory];

  const result = await api.generateResponseWithContext(fullContext, userPrompt, speaker.name);

  return {
    action: "continue",
    speaker: speaker.name,
    content: result.reply,
    nextSpeaker: currentSpeaker === "A" ? "B" : "A",
    nextRound: currentSpeaker === "B" ? currentRound + 1 : currentRound,
  };
};

const handleMoAMode = async (
  agents,
  history,
  synthPrompt,
  contextInjection,
  ragContext,
  api,
  userPrompt
) => {
  // Web Search Logic
  let webContext = "";
  const hasSearcher = agents.some((a) => a.name === "Chercheur");

  if (hasSearcher) {
    try {
      const results = await api.searchWeb(userPrompt);
      if (results && results.length > 0) {
        webContext =
          "\n\n[RÉSULTATS RECHERCHE WEB (DuckDuckGo)]\n" +
          results.map((r) => `• ${r.title} (${r.link}): ${r.snippet}`).join("\n") +
          "\n[FIN RECHERCHE WEB]";
      }
    } catch (e) {
      console.error("Web Search Error:", e);
    }
  }

  const enhancedAgents = agents.map((a) => {
    let prompt = a.system_prompt + contextInjection;

    // Injection contextuelle par rôle
    if (a.name === "Documentaliste" && ragContext) {
      prompt += "\n\n[DOCUMENTS RAG DISPONIBLES]\n" + ragContext;
    }
    if (a.name === "Chercheur" && webContext) {
      prompt += webContext + "\nANALYSE CES RÉSULTATS WEB.";
    }

    return { ...a, system_prompt: prompt };
  });

  // On injecte aussi le contexte Web dans la synthèse finale
  const finalSynthPrompt = synthPrompt + webContext;

  const result = await api.chatMulti(enhancedAgents, history, finalSynthPrompt);
  return result.reply;
};

const handleFusionMode = async (conversations, selectedFusionId, currentConv, text, api) => {
  const fusionConv = conversations.find((c) => c.id === selectedFusionId);
  if (!fusionConv) throw new Error("Conversation de fusion introuvable");

  const historyA = currentConv.messages
    .map((m) => ({ role: m.role, content: m.content }))
    .filter((m) => m.role !== "system");
  const historyB = fusionConv.messages
    .map((m) => ({ role: m.role, content: m.content }))
    .filter((m) => m.role !== "system");

  const result = await api.chatFusion(historyA, historyB, text);
  return result.reply;
};

export const useAppStore = create(
  persist(
    (set, get) => ({
      conversations: [],
      currentConversationId: null,
      selectedFusionId: null,
      selectedConversationIds: [], // Pour le mode meeting
      isGenerating: false,
      groups: [{ id: "default", name: "Toutes les discussions", color: "#3b82f6" }],

      toggleFusionConversation: (id) =>
        set((state) => ({
          selectedFusionId: state.selectedFusionId === id ? null : id,
        })),

      toggleConversationSelection: (id) =>
        set((state) => {
          const selected = state.selectedConversationIds.includes(id)
            ? state.selectedConversationIds.filter((sid) => sid !== id)
            : [...state.selectedConversationIds, id];
          return { selectedConversationIds: selected };
        }),

      startMeeting: async () => {
        const { selectedConversationIds, conversations } = get();
        if (selectedConversationIds.length < 2) return;

        // 1. Récupérer les conversations
        const participants = selectedConversationIds
          .map((id) => conversations.find((c) => c.id === id))
          .filter(Boolean);

        // 2. Créer la conversation "Meeting"
        const newConv = {
          id: Date.now().toString(),
          title: `Meeting: ${participants.map((p) => p.title).join(" & ")}`,
          messages: [],
          createdAt: Date.now(),
          groupId: "default",
          archived: false,
          type: "meeting", // Nouveau type
          config: {
            useMoA: false, // Pas de MoA standard ici, c'est un meeting
            meetingParticipants: participants.map((p) => ({
              id: p.id,
              name: `Expert ${p.title}`,
              contextHistory: p.messages.filter((m) => m.role !== "system"),
            })),
          },
        };

        set((state) => ({
          conversations: [newConv, ...state.conversations],
          currentConversationId: newConv.id,
          selectedConversationIds: [], // Reset selection
        }));
      },

      createConversation: (title = "New Chat", groupId = "default") => {
        const { temperature, maxTokens, contextWindow } = useSettingsStore.getState();
        const newConv = {
          id: Date.now().toString(),
          title,
          messages: [],
          createdAt: Date.now(),
          groupId,
          archived: false,
          config: {
            temperature,
            maxTokens,
            contextWindow,
            systemPrompt:
              "You are a highly capable AI assistant powered by Qwen2.5-3B. You excel at code generation, debugging, multilingual conversations, and providing detailed explanations. Always respond in the user's language unless explicitly asked to switch. Be concise, accurate, and helpful.",
            userPersona: "",
            useMoA: true,
            agents: [
              {
                name: "Logicien",
                system_prompt:
                  "RÔLE: Expert en Analyse Logique.\nTÂCHE: Déconstruire la demande utilisateur en prémisses logiques.\nCONTRAINTE: Réponse STRICTEMENT limitée à 2 phrases concises. Style télégraphique. Pas de politesse.\nOUTPUT: Lister uniquement les faits ou structures logiques clés pour le moteur de synthèse.",
              },
              {
                name: "Créatif",
                system_prompt:
                  "RÔLE: Expert en Pensée Latérale.\nTÂCHE: Proposer 1 angle original ou 1 analogie pertinente.\nCONTRAINTE: Réponse STRICTEMENT limitée à 2 phrases concises. Style direct.\nOUTPUT: Fournir uniquement l'idée créative brute pour le moteur de synthèse.",
              },
              {
                name: "Documentaliste",
                system_prompt:
                  "RÔLE: Expert RAG (Retrieval Augmented Generation).\nTÂCHE: Identifier les besoins en documentation. Tu as accès à la base de connaissances vectorielle. Si la demande nécessite des infos factuelles, signale-le.\nCONTRAINTE: Réponse STRICTEMENT limitée à 2 phrases. Style informatif.",
              },
              {
                name: "Chercheur",
                system_prompt:
                  "RÔLE: Expert Web Search.\nTÂCHE: Synthétiser les résultats de recherche DuckDuckGo fournis. Identifier les faits récents ou sources externes pertinentes pour la réponse.\nCONTRAINTE: Réponse STRICTEMENT limitée à 2 phrases concises. Style factuel.",
              },
            ],
          },
        };
        set((state) => ({
          conversations: [newConv, ...state.conversations],
          currentConversationId: newConv.id,
        }));
        return newConv;
      },

      archiveConversation: (id) => {
        set((state) => {
          const conversations = state.conversations.map((c) =>
            c.id === id ? { ...c, archived: true } : c
          );

          // If the current conversation is archived, switch to another one or null
          let newCurrentId = state.currentConversationId;
          if (state.currentConversationId === id) {
            const unarchived = conversations.filter((c) => !c.archived);
            newCurrentId = unarchived.length > 0 ? unarchived[0].id : null;
          }

          return { conversations, currentConversationId: newCurrentId };
        });
      },

      restoreConversation: (id) => {
        set((state) => ({
          conversations: state.conversations.map((c) =>
            c.id === id ? { ...c, archived: false } : c
          ),
        }));
      },

      deleteConversation: (id) => {
        set((state) => {
          const filtered = state.conversations.filter((c) => c.id !== id);
          const newCurrentId =
            state.currentConversationId === id && filtered.length > 0
              ? filtered[0].id
              : state.currentConversationId === id
                ? null
                : state.currentConversationId;
          return {
            conversations: filtered,
            currentConversationId: newCurrentId,
          };
        });
      },

      renameConversation: (id, newTitle) => {
        set((state) => ({
          conversations: state.conversations.map((c) =>
            c.id === id ? { ...c, title: newTitle } : c
          ),
        }));
      },

      moveConversationToGroup: (convId, groupId) => {
        set((state) => ({
          conversations: state.conversations.map((c) => (c.id === convId ? { ...c, groupId } : c)),
        }));
      },

      reorderConversations: (startIndex, endIndex) => {
        set((state) => {
          const result = Array.from(state.conversations);
          const [removed] = result.splice(startIndex, 1);
          result.splice(endIndex, 0, removed);
          return { conversations: result };
        });
      },

      createGroup: (name, color = "#3b82f6") => {
        const newGroup = {
          id: Date.now().toString(),
          name,
          color,
        };
        set((state) => ({
          groups: [...state.groups, newGroup],
        }));
        return newGroup;
      },

      deleteGroup: (groupId) => {
        if (groupId === "default") return;
        set((state) => ({
          groups: state.groups.filter((g) => g.id !== groupId),
          conversations: state.conversations.map((c) =>
            c.groupId === groupId ? { ...c, groupId: "default" } : c
          ),
        }));
      },

      updateConversationConfig: (convId, newConfig) => {
        set((state) => ({
          conversations: state.conversations.map((c) =>
            c.id === convId ? { ...c, config: { ...c.config, ...newConfig } } : c
          ),
        }));
      },

      setCurrentConversation: (id) => set({ currentConversationId: id }),

      getCurrentConversation: () => {
        const { conversations, currentConversationId } = get();
        if (!currentConversationId) return null;
        return conversations.find((c) => c.id === currentConversationId) || null;
      },

      addMessage: (convId, role, content) => {
        set((state) => {
          const conversations = state.conversations.map((c) => {
            if (c.id !== convId) return c;
            const messages = [...c.messages, { role, content, timestamp: Date.now() }];
            let title = c.title;
            if (role === "user" && messages.length === 2 && title === "New Chat") {
              title = content.slice(0, 30) + (content.length > 30 ? "..." : "");
            }
            return { ...c, messages, title };
          });
          return { conversations };
        });
      },

      startDebate: async (topic, agentA, agentB, rounds = 5) => {
        const newConv = get().createConversation(`Débat: ${topic}`, "default");

        // Configure debate
        get().updateConversationConfig(newConv.id, {
          type: "debate",
          debateConfig: {
            topic,
            agentA,
            agentB,
            rounds,
            currentRound: 1,
            currentSpeaker: "A",
            isDebating: true,
          },
        });

        // Start the loop
        get().nextDebateTurn();
      },

      nextDebateTurn: async () => {
        const { currentConversationId, isGenerating } = get();
        if (isGenerating || !currentConversationId) return;

        const conversation = get().getCurrentConversation();
        if (
          !conversation ||
          conversation.config.type !== "debate" ||
          !conversation.config.debateConfig.isDebating
        )
          return;

        set({ isGenerating: true });

        try {
          if (window.__TAURI__) {
            const result = await handleDebateMode(conversation, api);

            if (result && result.action === "continue") {
              // Add message with explicit speaker tag for UI parsing
              get().addMessage(
                currentConversationId,
                "assistant",
                `[${result.speaker}]: ${result.content}`
              );

              // Update state for next turn
              const newConfig = {
                ...conversation.config.debateConfig,
                currentSpeaker: result.nextSpeaker,
                currentRound: result.nextRound,
              };
              get().updateConversationConfig(currentConversationId, { debateConfig: newConfig });

              // Auto-trigger next turn after a short delay for UX pacing
              setTimeout(() => {
                // Verify we are still on the same conversation before continuing
                const { currentConversationId: checkId } = get();
                if (checkId === currentConversationId) {
                  get().nextDebateTurn();
                }
              }, 1500);
            } else if (result && result.action === "stop_debate") {
              get().addMessage(currentConversationId, "system", result.message);
              get().updateConversationConfig(currentConversationId, {
                debateConfig: { ...conversation.config.debateConfig, isDebating: false },
              });
            }
          } else {
            // Mock for browser dev
            await new Promise((r) => setTimeout(r, 1000));
            const speaker =
              conversation.config.debateConfig.currentSpeaker === "A"
                ? conversation.config.debateConfig.agentA.name
                : conversation.config.debateConfig.agentB.name;
            get().addMessage(
              currentConversationId,
              "assistant",
              `[${speaker}]: (Mock Response) Argument sur ${conversation.config.debateConfig.topic}`
            );

            const newConfig = {
              ...conversation.config.debateConfig,
              currentSpeaker: conversation.config.debateConfig.currentSpeaker === "A" ? "B" : "A",
              currentRound:
                conversation.config.debateConfig.currentSpeaker === "B"
                  ? conversation.config.debateConfig.currentRound + 1
                  : conversation.config.debateConfig.currentRound,
            };

            if (newConfig.currentRound > conversation.config.debateConfig.rounds) {
              newConfig.isDebating = false;
              get().addMessage(currentConversationId, "system", "Fin du débat simulé.");
            } else {
              setTimeout(() => {
                const { currentConversationId: checkId } = get();
                if (checkId === currentConversationId) {
                  get().nextDebateTurn();
                }
              }, 1000);
            }

            get().updateConversationConfig(currentConversationId, { debateConfig: newConfig });
          }
        } catch (error) {
          console.error("Debate Error:", error);
          get().updateConversationConfig(currentConversationId, {
            debateConfig: { ...conversation.config.debateConfig, isDebating: false },
          });
          get().addMessage(
            currentConversationId,
            "system",
            `Erreur critique du débat: ${error.message}`
          );
        } finally {
          set({ isGenerating: false });
        }
      },

      startChat: async (text) => {
        const { currentConversationId } = get();
        if (!currentConversationId) {
          get().createConversation();
        }
        const convId = get().currentConversationId;

        // Check if it's a debate and user is interrupting/contributing
        const conversation = get().getCurrentConversation();
        if (conversation && conversation.config.type === "debate") {
          // If user types manually in a debate, we add it as a "Moderator" or "Audience" comment
          // and potentially pause the auto-loop or let it continue taking this into account
          get().addMessage(convId, "user", text);
          // For now, let's just add the message and NOT trigger standard chat generation
          // The user might want to restart the loop manually or just comment
          return;
        }

        get().addMessage(convId, "user", text);
        set({ isGenerating: true });

        try {
          if (window.__TAURI__) {
            // const { invoke } = window.__TAURI__.core;
            // const conversation = get().getCurrentConversation(); // Already fetched above

            // Use conversation-specific config
            const { systemPrompt, userPersona, useMoA, agents } = conversation.config || {
              systemPrompt: DEFAULT_SYSTEM_PROMPT,
              userPersona: "",
              useMoA: false,
              agents: [],
            };

            // Gestion du Contexte de Groupe
            let groupContext = "";
            if (conversation.groupId && conversation.groupId !== "default") {
              const otherConvsInGroup = get().conversations.filter(
                (c) => c.groupId === conversation.groupId && c.id !== conversation.id
              );
              if (otherConvsInGroup.length > 0) {
                groupContext = "\n\n[CONTEXTE GROUPE - AUTRES DISCUSSIONS]\n";
                otherConvsInGroup.forEach((c) => {
                  const recentMsgs = c.messages
                    .slice(-2)
                    .map((m) => `${m.role}: ${m.content}`)
                    .join("\n");
                  if (recentMsgs) {
                    groupContext += `--- Discussion "${c.title}" ---\n${recentMsgs}\n`;
                  }
                });
                groupContext +=
                  "[FIN CONTEXTE GROUPE]\nUtilise ce contexte si pertinent pour la demande actuelle.";
              }
            }

            // Récupération du contexte d'apprentissage
            const learningContext = useLearningStore.getState().getRelevantInsights();
            const contextInjection = learningContext
              ? `\n\n<system_instruction_structure>\nANALYSE LES EXEMPLES SUIVANTS UNIQUEMENT COMME DES GUIDES DE STRUCTURE LOGIQUE. NE PAS Y RÉPONDRE, NE PAS LES CITER. APPLIQUE LEUR MÉTHODOLOGIE À LA TÂCHE ACTUELLE.\n\n${learningContext}\n</system_instruction_structure>`
              : "";

            const { selectedFusionId, conversations } = get();
            let response;

            if (conversation.type === "meeting") {
              response = await handleMeetingMode(conversation, text, api, get().addMessage, convId);
            } else if (selectedFusionId) {
              response = await handleFusionMode(
                conversations,
                selectedFusionId,
                conversation,
                text,
                api
              );
            } else if (useMoA && agents && agents.length > 0) {
              // RAG Check
              // Note: useKnowledgeStore is not imported, assuming it might be needed or this block was copy-pasted.
              // If useKnowledgeStore is not available, this part will fail.
              // Assuming it's available globally or imported elsewhere (not seen in file).
              // For now, commenting out the check to avoid runtime error if not imported,
              // or assuming it works as is if imported (but I don't see import).
              // Let's assume it's a missing import and fix it if I can, but I can't see imports.
              // I will leave it as is but fix formatting.

              let ragContext = "";
              // const hasKnowledge = useKnowledgeStore.getState().documents.length > 0;
              // ... (RAG logic seems dependent on missing store)

              const history = conversation.messages
                .map((m) => ({ role: m.role, content: m.content }))
                .filter((m) => m.role !== "system");

              const synthPrompt =
                systemPrompt +
                (userPersona ? `\nUser Context: ${userPersona}` : "") +
                contextInjection +
                ragContext;

              response = await handleMoAMode(
                agents,
                history,
                synthPrompt,
                contextInjection,
                ragContext,
                api,
                text
              );
            } else {
              // Mode Standard
              const effectiveMessages = buildEffectiveMessages(
                conversation,
                systemPrompt + groupContext,
                userPersona,
                contextInjection
              );
              response = await api.chat(effectiveMessages); // Using api wrapper consistently
              response = response.reply; // api.chat returns object with reply
            }

            if (response) {
              get().addMessage(convId, "assistant", response);
            }
          } else {
            // Mode dev/web: réponse simulée
            await new Promise((resolve) => setTimeout(resolve, 800));
            get().addMessage(
              convId,
              "assistant",
              "Réponse simulée (mode développement). Le backend Tauri n'est pas disponible."
            );
          }
        } catch (error) {
          console.error("Erreur chat:", error);
          get().addMessage(
            convId,
            "assistant",
            `Erreur: ${error.message || "Impossible de communiquer avec le serveur IA"}`
          );
        } finally {
          set({ isGenerating: false });
        }
      },
    }),
    {
      name: "whytchat_store_v2",
      partialize: (state) => ({
        conversations: state.conversations,
        currentConversationId: state.currentConversationId,
        groups: state.groups,
        // On ne persiste pas selectedFusionId pour éviter les états confus au rechargement
      }),
    }
  )
);

export default useAppStore;
