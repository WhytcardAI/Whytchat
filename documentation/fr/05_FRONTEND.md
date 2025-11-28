# 🖥️ Frontend React - WhytChat V1

> Interface utilisateur React avec Vite et Tailwind CSS

---

## 📁 Structure des Fichiers

```
apps/desktop-ui/
├── src/
│   ├── App.jsx           # Composant racine avec routing logique
│   ├── main.jsx          # Point d'entrée React
│   ├── index.css         # Styles Tailwind
│   ├── i18n.js           # Configuration i18next (FR/EN)
│   ├── components/
│   │   ├── ChatInterface.jsx     # Interface de chat principale
│   │   ├── MessageBubble.jsx     # Bulle de message
│   │   ├── ThinkingBubble.jsx    # Animation "en train de réfléchir"
│   │   ├── PreflightCheck.jsx    # Écran de vérification au démarrage
│   │   ├── OnboardingFlow.jsx    # Assistant première utilisation
│   │   ├── MainLayout.jsx        # Layout avec sidebar
│   │   ├── Dashboard.jsx         # Tableau de bord
│   │   ├── Settings.jsx          # Paramètres
│   │   ├── ThemeSelector.jsx     # Sélecteur de thème
│   │   └── ui/                   # Composants ShadCN/UI
│   ├── hooks/
│   │   ├── useChatStream.js      # Hook streaming chat
│   │   └── useTheme.js           # Hook gestion thème
│   ├── store/
│   │   └── appStore.js           # Store Zustand
│   ├── lib/
│   │   └── utils.js              # Utilitaires (cn, etc.)
│   └── locales/
│       ├── en.json               # Traductions anglais
│       └── fr.json               # Traductions français
├── public/                       # Assets statiques
├── index.html                    # HTML template
├── vite.config.js               # Configuration Vite
├── tailwind.config.js           # Configuration Tailwind
├── postcss.config.js            # Configuration PostCSS
└── package.json                 # Dépendances
```

---

## 🧩 Composants Principaux

### App.jsx - Composant Racine

```jsx
import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";

function App() {
  const [appState, setAppState] = useState("loading");
  // 'loading' | 'preflight' | 'onboarding' | 'ready'

  useEffect(() => {
    checkInitialization();
  }, []);

  async function checkInitialization() {
    try {
      // Vérifier si première utilisation
      const needsOnboarding = await invoke("needs_onboarding");
      if (needsOnboarding) {
        setAppState("onboarding");
        return;
      }

      // Lancer preflight
      setAppState("preflight");
      await invoke("run_preflight");
      setAppState("ready");
    } catch (error) {
      console.error("Initialization failed:", error);
    }
  }

  // Routing conditionnel
  if (appState === "loading") return <LoadingSpinner />;
  if (appState === "preflight") return <PreflightCheck />;
  if (appState === "onboarding")
    return <OnboardingFlow onComplete={() => setAppState("ready")} />;

  return <MainLayout />;
}
```

### MainLayout - Structure Principale

```jsx
function MainLayout() {
  const [currentView, setCurrentView] = useState("chat");
  const { sessions, currentSessionId, setCurrentSession } = useAppStore();

  return (
    <div className="flex h-screen bg-background">
      {/* Sidebar */}
      <aside className="w-64 border-r flex flex-col">
        <SessionList
          sessions={sessions}
          currentId={currentSessionId}
          onSelect={setCurrentSession}
        />
      </aside>

      {/* Main Content */}
      <main className="flex-1 flex flex-col">
        {currentView === "chat" && <ChatInterface />}
        {currentView === "dashboard" && <Dashboard />}
        {currentView === "settings" && <Settings />}
      </main>
    </div>
  );
}
```

---

## 💬 ChatInterface.jsx - Interface de Chat

### Structure

```jsx
function ChatInterface() {
  const { messages, currentSessionId, addMessage } = useAppStore();
  const [input, setInput] = useState("");
  const [isStreaming, setIsStreaming] = useState(false);
  const [thinkingSteps, setThinkingSteps] = useState([]);

  // Hook personnalisé pour le streaming
  const { startChat, stopChat } = useChatStream({
    onToken: (token) => {
      // Ajouter token au dernier message assistant
      updateLastMessage(token);
    },
    onThinkingStep: (step) => {
      setThinkingSteps((prev) => [...prev, step]);
    },
    onComplete: () => {
      setIsStreaming(false);
      setThinkingSteps([]);
    },
    onError: (error) => {
      console.error("Chat error:", error);
      setIsStreaming(false);
    },
  });

  async function handleSend() {
    if (!input.trim() || isStreaming) return;

    // Ajouter message utilisateur
    const userMessage = { role: "user", content: input };
    addMessage(userMessage);
    setInput("");

    // Commencer le streaming
    setIsStreaming(true);
    addMessage({ role: "assistant", content: "" }); // Placeholder

    try {
      await startChat(currentSessionId, input);
    } catch (error) {
      console.error("Failed to start chat:", error);
    }
  }

  return (
    <div className="flex flex-col h-full">
      {/* Messages */}
      <div className="flex-1 overflow-y-auto p-4 space-y-4">
        {messages.map((msg, idx) => (
          <MessageBubble key={idx} message={msg} />
        ))}

        {/* Thinking indicator */}
        {isStreaming && thinkingSteps.length > 0 && (
          <ThinkingBubble steps={thinkingSteps} />
        )}
      </div>

      {/* Input */}
      <div className="border-t p-4">
        <div className="flex gap-2">
          <textarea
            value={input}
            onChange={(e) => setInput(e.target.value)}
            onKeyDown={(e) => {
              if (e.key === "Enter" && !e.shiftKey) {
                e.preventDefault();
                handleSend();
              }
            }}
            placeholder="Type your message..."
            className="flex-1 resize-none rounded-lg border p-3"
            rows={3}
            disabled={isStreaming}
          />
          <button
            onClick={handleSend}
            disabled={isStreaming || !input.trim()}
            className="px-4 py-2 bg-primary text-white rounded-lg"
          >
            {isStreaming ? "Stop" : "Send"}
          </button>
        </div>
      </div>
    </div>
  );
}
```

---

## 💭 ThinkingBubble.jsx - Animation de Réflexion

```jsx
import { useState } from "react";
import { ChevronDown, ChevronUp, Brain } from "lucide-react";

function ThinkingBubble({ steps }) {
  const [isExpanded, setIsExpanded] = useState(false);

  // Dernier step comme résumé
  const latestStep = steps[steps.length - 1];

  return (
    <div className="flex items-start gap-3 mb-4">
      {/* Avatar Brain */}
      <div className="w-8 h-8 rounded-full bg-purple-100 flex items-center justify-center">
        <Brain className="w-4 h-4 text-purple-600 animate-pulse" />
      </div>

      {/* Bubble */}
      <div className="flex-1 bg-purple-50 rounded-lg p-3 max-w-[80%]">
        {/* Header with toggle */}
        <div
          className="flex items-center justify-between cursor-pointer"
          onClick={() => setIsExpanded(!isExpanded)}
        >
          <span className="text-sm text-purple-700 font-medium">
            Thinking...
          </span>
          {steps.length > 1 && (
            <button className="text-purple-500">
              {isExpanded ? <ChevronUp size={16} /> : <ChevronDown size={16} />}
            </button>
          )}
        </div>

        {/* Latest step or all steps */}
        {isExpanded ? (
          <div className="mt-2 space-y-1">
            {steps.map((step, idx) => (
              <div
                key={idx}
                className="text-sm text-purple-600 flex items-center gap-2"
              >
                <span className="w-4 h-4 rounded-full bg-purple-200 text-xs flex items-center justify-center">
                  {idx + 1}
                </span>
                {step}
              </div>
            ))}
          </div>
        ) : (
          <p className="text-sm text-purple-600 mt-1 animate-pulse">
            {latestStep}
          </p>
        )}
      </div>
    </div>
  );
}

export default ThinkingBubble;
```

---

## 💬 MessageBubble.jsx - Bulle de Message

```jsx
import { useMemo } from "react";
import { User, Bot } from "lucide-react";
import ReactMarkdown from "react-markdown";
import { Prism as SyntaxHighlighter } from "react-syntax-highlighter";
import { oneDark } from "react-syntax-highlighter/dist/esm/styles/prism";

function MessageBubble({ message }) {
  const isUser = message.role === "user";

  // Composants personnalisés pour le markdown
  const markdownComponents = useMemo(
    () => ({
      code({ node, inline, className, children, ...props }) {
        const match = /language-(\w+)/.exec(className || "");
        return !inline && match ? (
          <SyntaxHighlighter
            style={oneDark}
            language={match[1]}
            PreTag="div"
            {...props}
          >
            {String(children).replace(/\n$/, "")}
          </SyntaxHighlighter>
        ) : (
          <code className="bg-gray-100 px-1 rounded" {...props}>
            {children}
          </code>
        );
      },
    }),
    []
  );

  return (
    <div
      className={`flex items-start gap-3 ${isUser ? "flex-row-reverse" : ""}`}
    >
      {/* Avatar */}
      <div
        className={`w-8 h-8 rounded-full flex items-center justify-center ${
          isUser ? "bg-blue-100" : "bg-green-100"
        }`}
      >
        {isUser ? (
          <User className="w-4 h-4 text-blue-600" />
        ) : (
          <Bot className="w-4 h-4 text-green-600" />
        )}
      </div>

      {/* Message Content */}
      <div
        className={`max-w-[70%] rounded-lg p-3 ${
          isUser ? "bg-blue-500 text-white" : "bg-gray-100 text-gray-900"
        }`}
      >
        {isUser ? (
          <p className="whitespace-pre-wrap">{message.content}</p>
        ) : (
          <div className="prose prose-sm max-w-none">
            <ReactMarkdown components={markdownComponents}>
              {message.content}
            </ReactMarkdown>
          </div>
        )}
      </div>
    </div>
  );
}

export default MessageBubble;
```

---

## 🎣 useChatStream.js - Hook de Streaming

```javascript
import { useEffect, useRef, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

/**
 * Hook pour gérer le streaming de chat
 * @param {Object} options
 * @param {Function} options.onToken - Callback pour chaque token reçu
 * @param {Function} options.onThinkingStep - Callback pour les étapes de réflexion
 * @param {Function} options.onComplete - Callback quand le streaming est terminé
 * @param {Function} options.onError - Callback en cas d'erreur
 */
export function useChatStream({
  onToken,
  onThinkingStep,
  onComplete,
  onError,
}) {
  const unlistenersRef = useRef([]);

  // Setup listeners globaux
  useEffect(() => {
    async function setupListeners() {
      // Listener pour les tokens
      const unlistenToken = await listen("chat-token", (event) => {
        onToken?.(event.payload);
      });

      // Listener pour les étapes de réflexion
      const unlistenThinking = await listen("thinking-step", (event) => {
        onThinkingStep?.(event.payload);
      });

      // Listener pour l'analyse brain
      const unlistenBrain = await listen("brain-analysis", (event) => {
        console.log("Brain analysis:", event.payload);
      });

      unlistenersRef.current = [unlistenToken, unlistenThinking, unlistenBrain];
    }

    setupListeners();

    // Cleanup
    return () => {
      unlistenersRef.current.forEach((unlisten) => unlisten());
    };
  }, [onToken, onThinkingStep]);

  // Démarrer le chat
  const startChat = useCallback(
    async (sessionId, content) => {
      try {
        await invoke("debug_chat", {
          sessionId,
          content,
        });
        onComplete?.();
      } catch (error) {
        onError?.(error);
      }
    },
    [onComplete, onError]
  );

  // Arrêter le chat (TODO: implémenter abort)
  const stopChat = useCallback(() => {
    // Future: invoke('abort_chat')
    console.warn("Stop chat not yet implemented");
  }, []);

  return { startChat, stopChat };
}
```

---

## 🗃️ appStore.js - Store Zustand

```javascript
import { create } from "zustand";
import { persist, createJSONStorage } from "zustand/middleware";
import { invoke } from "@tauri-apps/api/core";

const useAppStore = create(
  persist(
    (set, get) => ({
      // === State ===
      sessions: [],
      currentSessionId: null,
      messages: [],
      theme: "system", // 'light' | 'dark' | 'system'
      language: "en", // 'en' | 'fr'

      // === Session Actions ===
      loadSessions: async () => {
        try {
          const sessions = await invoke("get_all_sessions");
          set({ sessions });
        } catch (error) {
          console.error("Failed to load sessions:", error);
        }
      },

      createSession: async (name) => {
        try {
          const session = await invoke("create_session", { name });
          set((state) => ({
            sessions: [session, ...state.sessions],
            currentSessionId: session.id,
            messages: [],
          }));
          return session;
        } catch (error) {
          console.error("Failed to create session:", error);
          throw error;
        }
      },

      setCurrentSession: async (sessionId) => {
        try {
          const messages = await invoke("get_messages", { sessionId });
          set({
            currentSessionId: sessionId,
            messages,
          });
        } catch (error) {
          console.error("Failed to load messages:", error);
        }
      },

      deleteSession: async (sessionId) => {
        try {
          await invoke("delete_session", { sessionId });
          set((state) => ({
            sessions: state.sessions.filter((s) => s.id !== sessionId),
            currentSessionId:
              state.currentSessionId === sessionId
                ? null
                : state.currentSessionId,
            messages:
              state.currentSessionId === sessionId ? [] : state.messages,
          }));
        } catch (error) {
          console.error("Failed to delete session:", error);
        }
      },

      // === Message Actions ===
      addMessage: (message) => {
        set((state) => ({
          messages: [...state.messages, message],
        }));
      },

      updateLastMessage: (content) => {
        set((state) => {
          const messages = [...state.messages];
          if (messages.length > 0) {
            const last = messages[messages.length - 1];
            messages[messages.length - 1] = {
              ...last,
              content: last.content + content,
            };
          }
          return { messages };
        });
      },

      // === Settings Actions ===
      setTheme: (theme) => set({ theme }),
      setLanguage: (language) => set({ language }),
    }),
    {
      name: "whytchat-storage",
      storage: createJSONStorage(() => localStorage),
      partialize: (state) => ({
        // Persister seulement ces champs
        theme: state.theme,
        language: state.language,
        currentSessionId: state.currentSessionId,
      }),
      onRehydrateStorage: () => (state) => {
        // Appliquer le thème au chargement
        if (state?.theme) {
          applyTheme(state.theme);
        }
      },
    }
  )
);

function applyTheme(theme) {
  const root = document.documentElement;

  if (theme === "system") {
    const prefersDark = window.matchMedia(
      "(prefers-color-scheme: dark)"
    ).matches;
    root.classList.toggle("dark", prefersDark);
  } else {
    root.classList.toggle("dark", theme === "dark");
  }
}

export default useAppStore;
```

---

## 🌍 i18n.js - Internationalisation

```javascript
import i18n from "i18next";
import { initReactI18next } from "react-i18next";
import LanguageDetector from "i18next-browser-languagedetector";

import en from "./locales/en.json";
import fr from "./locales/fr.json";

i18n
  .use(LanguageDetector)
  .use(initReactI18next)
  .init({
    resources: {
      en: { translation: en },
      fr: { translation: fr },
    },
    fallbackLng: "en",
    supportedLngs: ["en", "fr"],
    interpolation: {
      escapeValue: false,
    },
    detection: {
      order: ["localStorage", "navigator"],
      caches: ["localStorage"],
    },
  });

export default i18n;
```

---

## 🎨 Styles et Configuration

### tailwind.config.js

```javascript
/** @type {import('tailwindcss').Config} */
export default {
  darkMode: "class",
  content: ["./index.html", "./src/**/*.{js,jsx,ts,tsx}"],
  theme: {
    extend: {
      colors: {
        background: "hsl(var(--background))",
        foreground: "hsl(var(--foreground))",
        primary: {
          DEFAULT: "hsl(var(--primary))",
          foreground: "hsl(var(--primary-foreground))",
        },
        // ... autres couleurs ShadCN
      },
    },
  },
  plugins: [require("@tailwindcss/typography")],
};
```

### vite.config.js

```javascript
import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import path from "path";

export default defineConfig({
  plugins: [react()],
  resolve: {
    alias: {
      "@": path.resolve(__dirname, "./src"),
    },
  },
  server: {
    port: 1420,
    strictPort: true,
  },
  build: {
    outDir: "dist",
    sourcemap: true,
  },
  // Empêcher Vite d'obscurcir les erreurs Rust
  clearScreen: false,
});
```

---

## 📦 Dépendances Clés

| Package                    | Version | Usage                       |
| -------------------------- | ------- | --------------------------- |
| `react`                    | ^18.3.1 | Framework UI                |
| `react-dom`                | ^18.3.1 | DOM rendering               |
| `@tauri-apps/api`          | ^2.0.0  | Communication avec Rust     |
| `zustand`                  | ^4.x    | State management            |
| `react-markdown`           | ^9.x    | Rendu markdown              |
| `react-syntax-highlighter` | ^15.x   | Syntax highlighting code    |
| `i18next`                  | ^23.x   | Internationalisation        |
| `react-i18next`            | ^14.x   | Bindings React pour i18next |
| `lucide-react`             | ^0.x    | Icônes                      |
| `tailwindcss`              | ^3.x    | Framework CSS               |
| `@tailwindcss/typography`  | ^0.5.x  | Plugin prose                |

---

_Généré depuis lecture directe de: App.jsx, ChatInterface.jsx, ThinkingBubble.jsx, MessageBubble.jsx, appStore.js, useChatStream.js, i18n.js, vite.config.js, tailwind.config.js, package.json_
