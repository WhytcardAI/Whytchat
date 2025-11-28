# 🏛️ Architecture - WhytChat V1

> Système d'acteurs Tokio avec orchestration Supervisor

---

## 🎭 Système d'Acteurs

Le backend utilise un pattern **Actor Model** basé sur les channels `tokio::sync::mpsc`.

```
┌─────────────────────────────────────────────────────────────────┐
│                         TAURI COMMANDS                          │
│                  (22 commandes dans main.rs)                    │
└─────────────────────────────┬───────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                       AppState (Mutex)                          │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │                  InitializedState                        │   │
│  │  • supervisor: SupervisorHandle                          │   │
│  │  • pool: SqlitePool                                      │   │
│  │  • rate_limiter: Mutex<RateLimiter>                      │   │
│  └─────────────────────────────────────────────────────────┘   │
└─────────────────────────────┬───────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                    SupervisorHandle                             │
│              (mpsc::Sender<SupervisorMessage>)                  │
│                                                                 │
│  Messages:                                                      │
│  • ProcessUserMessage { session_id, content, window, responder }│
│  • IngestContent { content, metadata, responder }               │
│  • ReindexFile { file_id, content, responder }                  │
│  • Shutdown                                                     │
└────────────────────────┬────────────────────────────────────────┘
                         │
          ┌──────────────┼──────────────┐
          │              │              │
          ▼              ▼              ▼
┌─────────────┐  ┌─────────────┐  ┌─────────────┐
│ BrainAnalyzer│  │ LlmActorHandle│  │ RagActorHandle│
│             │  │             │  │             │
│ • Intent    │  │ • generate  │  │ • ingest    │
│ • Keywords  │  │ • stream    │  │ • search    │
│ • Complexity│  │             │  │ • delete    │
└─────────────┘  └─────────────┘  └─────────────┘
```

---

## 📦 Modules Backend (apps/core/src/)

### Fichiers Principaux

| Fichier           | Lignes | Rôle                                         |
| ----------------- | ------ | -------------------------------------------- |
| `main.rs`         | ~1530  | Point d'entrée, 22 commandes Tauri, download |
| `database.rs`     | ~350   | CRUD sessions, messages, folders, files      |
| `encryption.rs`   | ~180   | AES-256-GCM encrypt/decrypt                  |
| `error.rs`        | ~100   | AppError enum centralisé                     |
| `fs_manager.rs`   | ~180   | PortablePathManager (chemins portables)      |
| `models.rs`       | ~120   | Structs: Session, Message, Folder, etc.      |
| `rate_limiter.rs` | ~90    | Limite 20 req/min par session                |
| `text_extract.rs` | ~170   | Extraction PDF, DOCX, TXT, CSV, JSON         |
| `preflight.rs`    | -      | Vérifications au démarrage                   |
| `diagnostics.rs`  | -      | Tests runtime catégorisés                    |

### Dossier actors/

| Fichier         | Rôle                                             |
| --------------- | ------------------------------------------------ |
| `mod.rs`        | Exporte: llm, messages, rag, supervisor, traits  |
| `supervisor.rs` | Orchestrateur principal (SupervisorRunner)       |
| `llm.rs`        | Communication llama-server HTTP                  |
| `rag.rs`        | LanceDB + FastEmbed embeddings                   |
| `messages.rs`   | Enums: LlmMessage, RagMessage, SupervisorMessage |
| `traits.rs`     | Traits: LlmActor, RagActor                       |

### Dossier brain/

| Fichier              | Rôle                                    |
| -------------------- | --------------------------------------- |
| `mod.rs`             | Exporte tous les sous-modules           |
| `analyzer.rs`        | BrainAnalyzer - orchestrateur principal |
| `intent.rs`          | Classification regex rapide             |
| `semantic_intent.rs` | Classification embeddings (fallback)    |
| `keywords.rs`        | Extraction TF-IDF                       |
| `complexity.rs`      | Score de complexité texte               |
| `context_packet.rs`  | Struct ContextPacket de sortie          |

---

## 🔄 Pattern Handle/Runner

Chaque acteur utilise le pattern **Handle + Runner** :

```rust
// Handle (public, cloneable)
pub struct SupervisorHandle {
    sender: mpsc::Sender<SupervisorMessage>,
}

// Runner (internal, owns the receiver)
struct SupervisorRunner<L, R> {
    receiver: mpsc::Receiver<SupervisorMessage>,
    llm_actor: Arc<L>,
    rag_actor: Arc<R>,
    brain_analyzer: Arc<BrainAnalyzer>,
    db_pool: Option<SqlitePool>,
}
```

**Avantages :**

- Handle clonable pour partage entre threads
- Runner isolé avec son propre état
- Communication via messages typés
- Timeout sur les réponses (oneshot channels)

---

## 🧠 Brain Module (Analyse Pré-LLM)

Le module Brain analyse le message AVANT d'appeler le LLM :

```rust
pub fn analyze(&self, query: &str) -> ContextPacket {
    // 1. Classification intent (regex puis semantic)
    packet.intent = self.classify_intent_smart(query);

    // 2. Extraction keywords TF-IDF
    packet.keywords = self.keyword_extractor.extract(query, Some(10));

    // 3. Score complexité
    packet.complexity = self.complexity_scorer.analyze(query);

    // 4. Détection langue (fr/en)
    packet.language = self.detect_language(query);

    // 5. Stratégies suggérées
    packet.suggested_strategies = self.suggest_strategies(&packet);

    // 6. Décision RAG
    packet.should_use_rag = self.should_use_rag(&packet);

    packet
}
```

### Intents Supportés

```rust
pub enum Intent {
    Greeting,      // "Bonjour", "Hello"
    Farewell,      // "Au revoir", "Bye"
    Question,      // "Comment...", "What is..."
    Command,       // "Fais...", "Create..."
    CodeRequest,   // "Écris du code", "Write a function"
    Explanation,   // "Explique...", "Explain..."
    Translation,   // "Traduis...", "Translate..."
    Analysis,      // "Analyse...", "Analyze..."
    Creative,      // "Imagine...", "Write a story"
    Help,          // "Aide...", "Help..."
    Unknown,       // Fallback
}
```

---

## 📊 Flux de Données Principal

```
[User Message]
      │
      ▼
┌─────────────────┐
│  debug_chat()   │  ← Commande Tauri
│    main.rs      │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│  RateLimiter    │  ← 20 req/min/session
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ SupervisorHandle│
│ process_message │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│  BrainAnalyzer  │  ← Analyse intent, keywords, complexity
│    analyze()    │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│   RagActor      │  ← Si should_use_rag = true
│   search()      │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│   LlmActor      │  ← Streaming via llama-server
│ stream_generate │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│  Window.emit()  │  ← Events: "chat-token", "thinking-step"
│   → Frontend    │
└─────────────────┘
```

---

## 🔐 Gestion d'État (AppState)

```rust
struct AppState {
    is_initialized: Arc<AtomicBool>,
    app_handle: Arc<Mutex<Option<InitializedState>>>,
}

struct InitializedState {
    supervisor: SupervisorHandle,
    pool: SqlitePool,
    rate_limiter: Mutex<RateLimiter>,
}
```

**Fonctions utilitaires :**

- `get_initialized_state()` - Récupère le lock
- `get_pool()` - Extrait le pool SQLite
- `get_pool_and_supervisor()` - Pool + Supervisor
- `check_rate_limit_and_get_resources()` - Rate limit + resources

---

## 🌐 Communication Frontend ↔ Backend

### Events Émis (Backend → Frontend)

| Event               | Payload                  | Source           |
| ------------------- | ------------------------ | ---------------- |
| `chat-token`        | `String` (token LLM)     | LlmActorRunner   |
| `thinking-step`     | `String` (étape analyse) | SupervisorRunner |
| `brain-analysis`    | `ContextPacket` (JSON)   | SupervisorRunner |
| `download-progress` | `u64` (0-100)            | download_model   |
| `download-status`   | `{step, detail}` (JSON)  | download_model   |

### Commands Invoquées (Frontend → Backend)

Voir [06_COMMANDES_TAURI.md](06_COMMANDES_TAURI.md) pour la liste complète des 22 commandes.

---

_Généré depuis lecture directe de: supervisor.rs, llm.rs, rag.rs, messages.rs, traits.rs, analyzer.rs, main.rs_
