# 🏗️ Architecture Technique - WhytChat V1

> Structure détaillée du projet et patterns architecturaux

---

## 📂 Structure Monorepo

```
WhytChat_V1/
├── apps/
│   ├── core/                 # Backend Rust (Tauri)
│   │   ├── src/
│   │   │   ├── actors/       # Système d'acteurs (Supervisor, LLM, RAG)
│   │   │   ├── brain/        # Analyse intelligente pré-LLM
│   │   │   └── tests/        # Tests unitaires et chaos
│   │   ├── migrations/       # Schéma SQLite
│   │   └── tools/            # Binaires (llama-server)
│   │
│   └── desktop-ui/           # Frontend React
│       ├── src/
│       │   ├── components/   # UI Components
│       │   ├── hooks/        # React Hooks
│       │   ├── store/        # Zustand State
│       │   └── locales/      # Traductions
│       └── public/
│
├── data/                     # Données locales
│   ├── db/                   # SQLite database
│   ├── models/               # GGUF models + embeddings
│   ├── vectors/              # LanceDB vectors
│   └── files/                # Fichiers uploadés
│
└── documentation/            # Documentation
    └── fr/                   # Documentation française
```

---

## 🔄 Architecture Actor System

Le backend utilise un système d'acteurs asynchrones basé sur Tokio.

```
┌─────────────────────────────────────────────────────────────┐
│                        SUPERVISOR                            │
│  (Orchestrateur principal - routes les messages)             │
├─────────────────────────────────────────────────────────────┤
│                           │                                  │
│    ┌──────────────┐      │      ┌──────────────┐            │
│    │   BRAIN      │      │      │    DATABASE  │            │
│    │  Analyzer    │◄─────┼─────►│   (SQLite)   │            │
│    └──────────────┘      │      └──────────────┘            │
│           │              │              │                    │
│           ▼              │              │                    │
│    ┌──────────────┐      │      ┌──────────────┐            │
│    │  RAG ACTOR   │◄─────┼─────►│   LLM ACTOR  │            │
│    │  (LanceDB)   │      │      │(llama-server)│            │
│    └──────────────┘      │      └──────────────┘            │
│                          │                                   │
└─────────────────────────────────────────────────────────────┘
```

### Responsabilités des Acteurs

| Acteur | Responsabilité |
|--------|----------------|
| **Supervisor** | Orchestration, routing, émission d'événements |
| **Brain** | Analyse pré-LLM (intent, keywords, complexity) |
| **RAG** | Embeddings, stockage vectoriel, recherche sémantique |
| **LLM** | Communication avec llama-server, streaming |
| **Database** | CRUD SQLite avec chiffrement |

---

## 🔗 Communication Frontend ↔ Backend

### Pattern IPC Tauri

```
Frontend (React)              Tauri IPC              Backend (Rust)
      │                           │                        │
      │  invoke('debug_chat')     │                        │
      ├──────────────────────────►├───────────────────────►│
      │                           │                        │
      │                           │   emit('chat-token')   │
      │◄──────────────────────────┼◄───────────────────────┤
      │                           │                        │
      │                           │ emit('thinking-step')  │
      │◄──────────────────────────┼◄───────────────────────┤
```

### Événements Émis

| Événement | Payload | Description |
|-----------|---------|-------------|
| `chat-token` | `{ content: string }` | Token de réponse LLM |
| `thinking-step` | `{ step: string, details: string }` | Étape de réflexion |
| `brain-analysis` | `{ intent, keywords, ... }` | Résultat analyse Brain |

---

## 💾 Architecture des Données

### SQLite (Données Structurées)

```sql
sessions ────────────┬──────────── messages
    │                │
    │                └──── session_files
    │
folders ─────────────┴──────────── library_files
```

### LanceDB (Vecteurs)

```
knowledge_base.lance/
├── data/           # Chunks de texte vectorisés
└── index/          # Index pour recherche rapide
```

### Schéma de Chiffrement

```
ModelConfig (JSON) 
    │
    ▼ AES-256-GCM
Ciphertext (Base64)
    │
    ▼ SQLite TEXT
sessions.model_config
```

---

## 🎯 Patterns Utilisés

### 1. Handle Pattern (Actors)

```rust
// Séparation entre l'acteur et son interface
pub struct SupervisorHandle {
    sender: mpsc::Sender<SupervisorMessage>,
}

impl SupervisorHandle {
    pub async fn process_message(...) -> Result<...> {
        let (tx, rx) = oneshot::channel();
        self.sender.send(Message { response_tx: tx }).await?;
        rx.await?
    }
}
```

### 2. State Singleton (Tauri)

```rust
pub struct AppState {
    pub is_initialized: AtomicBool,
    pub initialized: OnceLock<InitializedState>,
}

// Usage dans commandes
#[tauri::command]
async fn my_command(state: State<'_, AppState>) -> Result<...> {
    if !state.is_initialized.load(Ordering::SeqCst) {
        return Err("Not initialized");
    }
    // ...
}
```

### 3. Zustand Persist (Frontend)

```javascript
const useAppStore = create(
  persist(
    (set, get) => ({ /* state & actions */ }),
    {
      name: 'whytchat-storage',
      partialize: (state) => ({
        // Seuls ces champs sont persistés
        theme: state.theme,
        currentSessionId: state.currentSessionId,
      }),
    }
  )
);
```

---

## 🔌 Ports Réseau

| Port | Service | Usage |
|------|---------|-------|
| 1420 | Vite dev server | Frontend dev |
| 8080 | llama-server | LLM inference |
| 18080 | llama-server (test) | Preflight checks |

---

## 🔐 Variables d'Environnement

| Variable | Description | Default |
|----------|-------------|---------|
| `ENCRYPTION_KEY` | Clé AES-256 (32 bytes hex) | Auto-généré |
| `LLAMA_AUTH_TOKEN` | Token auth llama-server | Auto-généré |
| `RUST_LOG` | Niveau de log | `info` |

---

## 📚 Voir Aussi

- [03_BACKEND_RUST.md](03_BACKEND_RUST.md) - Détails des modules Rust
- [04_FRONTEND_REACT.md](04_FRONTEND_REACT.md) - Détails des composants React
- [05_FLUX_DONNEES.md](05_FLUX_DONNEES.md) - Flux complets

---

_Document généré le 27 novembre 2025_
