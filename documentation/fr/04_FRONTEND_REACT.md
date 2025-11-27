# ⚛️ Frontend React - WhytChat V1

> Documentation détaillée des modules `apps/desktop-ui/src/`

---

## 📑 Table des Matières

1. [State Management](#1-state-management)
2. [Hooks](#2-hooks)
3. [Layout Components](#3-layout-components)
4. [Chat Components](#4-chat-components)
5. [Configuration](#5-configuration)

---

## 1. State Management

### 1.1 appStore.js (~390 lignes)

**Framework** : Zustand avec middleware `persist`

```javascript
import { create } from 'zustand';
import { persist } from 'zustand/middleware';

export const useAppStore = create(
  persist(
    (set, get) => ({
      // State & actions
    }),
    {
      name: 'whytchat-storage',
      partialize: (state) => ({
        isConfigured: state.isConfigured,
        isSidebarOpen: state.isSidebarOpen,
        currentView: state.currentView,
        currentSessionId: state.currentSessionId,
        theme: state.theme,
      }),
    }
  )
);
```

### États Principaux

| État | Type | Persisté | Description |
|------|------|----------|-------------|
| `currentView` | `'knowledge' \| 'chat'` | ✅ | Vue active |
| `isSidebarOpen` | `boolean` | ✅ | État sidebar |
| `theme` | `'light' \| 'dark'` | ✅ | Thème visuel |
| `currentSessionId` | `string \| null` | ✅ | Session active |
| `sessions` | `Session[]` | ❌ | Liste sessions |
| `folders` | `Folder[]` | ❌ | Liste dossiers |
| `isThinking` | `boolean` | ❌ | LLM en cours |
| `thinkingSteps` | `Step[]` | ❌ | Étapes réflexion |

### Actions CRUD Sessions

```javascript
// Création
createSession: async (title, language, systemPrompt, temperature) => {
  const sessionId = await invoke('create_session', {
    title,
    language,
    system_prompt: systemPrompt,
    systemPrompt: systemPrompt,  // ⚠️ Double params
    temperature,
  });
  await get().loadSessions();
  set({ currentSessionId: sessionId });
  return sessionId;
},

// Suppression
deleteSession: async (sessionId) => {
  await invoke('delete_session', {
    session_id: sessionId,
    sessionId: sessionId,  // ⚠️ Double params
  });
  // Update local state
},

// Favoris
toggleFavorite: async (sessionId) => {
  const isFavorite = await invoke('toggle_session_favorite', {
    session_id: sessionId,
    sessionId: sessionId,
  });
  // Update local state
},
```

### Actions Thinking

```javascript
// Pour l'indicateur de "réflexion"
setThinking: (isThinking) => set({ isThinking }),

clearThinkingSteps: () => set({ thinkingSteps: [] }),

addThinkingStep: (step) => set((state) => ({
  thinkingSteps: [...state.thinkingSteps, step],
})),
```

---

## 2. Hooks

### 2.1 useChatStream.js (~170 lignes)

**But** : Gestion du streaming chat avec Tauri events.

**Pattern Singleton** :

```javascript
// Empêche duplicates en React Strict Mode
let globalListenersSetup = false;
let messageHandler = null;
let thinkingHandler = null;

export function useChatStream() {
  const [messages, setMessages] = useState([]);
  const { setThinking, addThinkingStep, clearThinkingSteps } = useAppStore();

  useEffect(() => {
    if (globalListenersSetup) return;
    globalListenersSetup = true;

    // Setup event listeners
    listen('chat-token', (event) => {
      if (messageHandler) messageHandler(event.payload);
    });

    listen('thinking-step', (event) => {
      if (thinkingHandler) thinkingHandler(event.payload);
    });
  }, []);

  // ...
}
```

**Fonction sendMessage** :

```javascript
const sendMessage = async (text, sessionId) => {
  // 1. Ajouter message user localement
  setMessages((prev) => [...prev, {
    id: crypto.randomUUID(),
    role: 'user',
    content: text,
  }]);

  // 2. Préparer réponse assistant
  setThinking(true);
  clearThinkingSteps();

  // 3. Setup handlers
  let assistantContent = '';
  messageHandler = (payload) => {
    assistantContent += payload.content;
    // Update assistant message
  };

  // 4. Invoquer backend
  try {
    await invoke('debug_chat', {
      session_id: sessionId,
      sessionId: sessionId,
      message: text,
    });
  } finally {
    setThinking(false);
  }
};
```

---

## 3. Layout Components

### 3.1 MainLayout.jsx (~310 lignes)

**Structure** :

```jsx
<MainLayout>
  <TitleBar />           {/* h-8 */}
  <Container>
    <Rail />             {/* w-16 */}
    <Sidebar />          {/* w-72, conditionnel */}
    <Main>
      <Header />         {/* h-14 */}
      {children}
      <KnowledgeView />  {/* overlay conditionnel */}
    </Main>
  </Container>
  <SessionWizard />      {/* modal */}
  <Toaster />
</MainLayout>
```

**État Sidebar** :

```jsx
const { isSidebarOpen, toggleSidebar } = useAppStore();

// Toggle via Rail ou bouton
<button onClick={toggleSidebar}>
  {isSidebarOpen ? <ChevronLeft /> : <ChevronRight />}
</button>
```

### 3.2 TitleBar.jsx (~65 lignes)

**Barre titre custom Tauri** (sans décoration native).

```jsx
export function TitleBar() {
  return (
    <div 
      data-tauri-drag-region 
      className="h-8 flex items-center justify-between bg-white dark:bg-gray-900"
    >
      <div className="flex items-center">
        <img src="/logo.png" className="h-5 w-5" />
        <span>WhytChat</span>
      </div>
      
      <div className="flex">
        <button onClick={() => appWindow.minimize()}>
          <Minus />
        </button>
        <button onClick={() => appWindow.toggleMaximize()}>
          <Square />
        </button>
        <button onClick={() => appWindow.close()}>
          <X />
        </button>
      </div>
    </div>
  );
}
```

### 3.3 Rail.jsx (~60 lignes)

**Navigation verticale** (icônes uniquement).

```jsx
const items = [
  { id: 'chat', icon: MessageSquare, label: t('rail.chat') },
  { id: 'knowledge', icon: BookOpen, label: t('rail.knowledge') },
];

export function Rail() {
  const { currentView, setView } = useAppStore();

  return (
    <nav className="w-16 flex flex-col items-center py-4">
      {items.map((item) => (
        <button
          key={item.id}
          onClick={() => setView(item.id)}
          className={cn(
            'p-3 rounded-lg',
            currentView === item.id && 'bg-primary/10'
          )}
        >
          <item.icon />
        </button>
      ))}
      
      {/* Settings - ⚠️ Non implémenté */}
      <button className="mt-auto">
        <Settings />
      </button>
    </nav>
  );
}
```

---

## 4. Chat Components

### 4.1 ChatInterface.jsx (~130 lignes)

**Interface principale du chat**.

```jsx
export function ChatInterface() {
  const { currentSessionId } = useAppStore();
  const { messages, sendMessage, loadMessages } = useChatStream();

  useEffect(() => {
    if (currentSessionId) {
      loadMessages(currentSessionId);
    }
  }, [currentSessionId]);

  const handleSend = async (text) => {
    if (!currentSessionId) {
      // Créer session automatiquement
      const newId = await createSession(text.slice(0, 50));
      await sendMessage(text, newId);
    } else {
      await sendMessage(text, currentSessionId);
    }
  };

  return (
    <div className="flex flex-col h-full">
      <MessageList messages={messages} />
      
      {/* ThinkingBubble - ⚠️ Commenté */}
      {/* {isThinking && <ThinkingBubble steps={thinkingSteps} />} */}
      
      <ChatInput onSend={handleSend} />
    </div>
  );
}
```

### 4.2 ChatInput.jsx (~105 lignes)

**Champ de saisie avec actions**.

```jsx
export function ChatInput({ onSend }) {
  const [text, setText] = useState('');
  const textareaRef = useRef(null);

  const handleSubmit = () => {
    if (!text.trim()) return;
    onSend(text.trim());
    setText('');
  };

  // Auto-resize textarea
  useEffect(() => {
    const textarea = textareaRef.current;
    textarea.style.height = 'auto';
    textarea.style.height = `${textarea.scrollHeight}px`;
  }, [text]);

  return (
    <div className="border-t p-4">
      <div className="flex items-end gap-2">
        <textarea
          ref={textareaRef}
          value={text}
          onChange={(e) => setText(e.target.value)}
          onKeyDown={(e) => {
            if (e.key === 'Enter' && !e.shiftKey) {
              e.preventDefault();
              handleSubmit();
            }
          }}
          placeholder={t('chat.placeholder')}
          className="flex-1 resize-none"
        />
        
        <button onClick={handleSubmit}>
          <Send />
        </button>
      </div>
    </div>
  );
}
```

### 4.3 MessageBubble.jsx (~300 lignes)

**Affichage des messages avec Markdown**.

**Syntaxes Supportées** :

| Syntax | Rendu |
|--------|-------|
| `# H1`, `## H2`, `### H3` | Titres stylisés |
| `- item`, `* item` | Liste à puces |
| `1. item` | Liste numérotée |
| `> quote` | Blockquote |
| `` `code` `` | Code inline |
| `**bold**` | Texte gras |
| ` ```lang...``` ` | Code block avec bouton Save |

**Limitations** :
- ❌ Pas de tables
- ❌ Pas d'images
- ❌ Pas de liens cliquables

**Code Blocks** :

```jsx
const renderCodeBlock = (content, language) => (
  <div className="relative group">
    <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto">
      <code>{content}</code>
    </pre>
    
    <div className="absolute top-2 right-2 opacity-0 group-hover:opacity-100">
      <button onClick={() => navigator.clipboard.writeText(content)}>
        <Copy />
      </button>
      <button onClick={() => saveToFile(content, language)}>
        <Download />
      </button>
    </div>
  </div>
);
```

---

## 5. Configuration

### 5.1 vite.config.js

```javascript
export default defineConfig({
  plugins: [react()],
  server: {
    port: 1420,
    strictPort: true,
  },
  build: {
    outDir: 'dist',
    sourcemap: true,
  },
});
```

### 5.2 tailwind.config.js

```javascript
module.exports = {
  darkMode: 'class',
  content: ['./src/**/*.{js,jsx}'],
  theme: {
    extend: {
      colors: {
        primary: {...},
        secondary: {...},
      },
    },
  },
  plugins: [],
};
```

### 5.3 i18n.js

**Configuration i18next** :

```javascript
import i18n from 'i18next';
import { initReactI18next } from 'react-i18next';

import fr from './locales/fr.json';
import en from './locales/en.json';

i18n.use(initReactI18next).init({
  resources: { fr, en },
  lng: 'fr',
  fallbackLng: 'en',
  interpolation: {
    escapeValue: false,
  },
});
```

---

## 📊 Index des Fichiers

| # | Fichier | Lignes | Description |
|---|---------|--------|-------------|
| 1 | `store/appStore.js` | ~390 | État Zustand centralisé |
| 2 | `hooks/useChatStream.js` | ~170 | Streaming chat |
| 3 | `components/layout/MainLayout.jsx` | ~310 | Layout principal |
| 4 | `components/layout/TitleBar.jsx` | ~65 | Barre titre Tauri |
| 5 | `components/layout/Rail.jsx` | ~60 | Navigation |
| 6 | `components/chat/ChatInterface.jsx` | ~130 | Interface chat |
| 7 | `components/chat/ChatInput.jsx` | ~105 | Champ saisie |
| 8 | `components/chat/MessageBubble.jsx` | ~300 | Affichage messages |

---

## ⚠️ Irrégularités Identifiées

| # | Fichier | Problème |
|---|---------|----------|
| 1 | `appStore.js` | Double params (snake_case + camelCase) |
| 2 | `useChatStream.js` | Variables globales peuvent leaker |
| 3 | `ChatInterface.jsx` | ThinkingBubble commenté |
| 4 | `MessageBubble.jsx` | Markdown parser limité |
| 5 | `Rail.jsx` | Settings button non fonctionnel |

---

## 📚 Voir Aussi

- [02_ARCHITECTURE.md](02_ARCHITECTURE.md) - Architecture globale
- [03_BACKEND_RUST.md](03_BACKEND_RUST.md) - Backend Rust
- [05_FLUX_DONNEES.md](05_FLUX_DONNEES.md) - Flux complets

---

_Document généré le 27 novembre 2025_
