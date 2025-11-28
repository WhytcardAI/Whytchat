# Carte des Dépendances - WhytChat V1

> Généré automatiquement le 2025-11-28

## 🏗️ Architecture Globale

```mermaid
graph TB
    subgraph Frontend["⚛️ Frontend React"]
        App[App.jsx]
        subgraph Components["📦 Components"]
            Chat[ChatInterface]
            Dashboard[Dashboard]
            Onboarding[OnboardingWizard]
            Preflight[PreflightCheck]
            Diagnostics[TestConsole]
        end
        subgraph Stores["🏪 Stores"]
            AppStore[appStore]
        end
        subgraph Layout["🎨 Layout"]
            MainLayout[MainLayout]
            Rail[Rail/Sidebar]
            TitleBar[TitleBar]
        end
    end

    subgraph Backend["🦀 Backend Rust"]
        Main[main.rs]
        subgraph Actors["🎭 Actors"]
            Supervisor[SupervisorHandle]
            LlmActor[LlmActorHandle]
            RagActor[RagActorHandle]
        end
        subgraph Core["⚙️ Core Modules"]
            Database[database.rs]
            Encryption[encryption.rs]
            FsManager[fs_manager.rs]
            Preflight_rs[preflight.rs]
        end
        subgraph Brain["🧠 Brain"]
            Analyzer[analyzer.rs]
            Intent[intent.rs]
            ContextPacket[context_packet.rs]
        end
    end

    subgraph Data["💾 Data Layer"]
        SQLite[(SQLite DB)]
        LanceDB[(LanceDB Vectors)]
        Models[("GGUF Models")]
    end

    App --> |"invoke()"| Main
    Chat --> |"send_message"| Supervisor
    Dashboard --> |"get_sessions"| Database
    Onboarding --> |"download_model"| Main

    Main --> Supervisor
    Supervisor --> LlmActor
    Supervisor --> RagActor
    LlmActor --> Models
    RagActor --> LanceDB
    Database --> SQLite
    Main --> Encryption
    Main --> FsManager
```

## 🦀 Modules Backend (Détaillé)

```mermaid
graph LR
    subgraph MainModule["main.rs - Entry Point"]
        Commands[24 Tauri Commands]
        State[AppState]
        Init[initialize_app]
    end

    subgraph ActorSystem["actors/"]
        supervisor["supervisor.rs<br/>746 lignes"]
        llm["llm.rs<br/>820 lignes"]
        rag["rag.rs<br/>554 lignes"]
        messages["messages.rs<br/>132 lignes"]
        traits["traits.rs<br/>49 lignes"]
    end

    subgraph CoreModules["Core Modules"]
        database["database.rs<br/>1260 lignes<br/>52 fonctions"]
        error["error.rs<br/>119 lignes"]
        encryption["encryption.rs<br/>196 lignes"]
        fs_manager["fs_manager.rs<br/>159 lignes"]
        models["models.rs<br/>81 lignes"]
        preflight["preflight.rs<br/>319 lignes"]
        diagnostics["diagnostics.rs<br/>1803 lignes"]
        rate_limiter["rate_limiter.rs<br/>81 lignes"]
        text_extract["text_extract.rs<br/>147 lignes"]
    end

    subgraph BrainModule["brain/"]
        analyzer["analyzer.rs<br/>327 lignes"]
        intent["intent.rs<br/>302 lignes"]
        context["context_packet.rs<br/>99 lignes"]
        keywords["keywords.rs<br/>61 lignes"]
        complexity["complexity.rs<br/>86 lignes"]
        semantic["semantic_intent.rs<br/>143 lignes"]
    end

    subgraph Tests["tests/"]
        chaos["chaos_test.rs<br/>196 lignes"]
        download["download_tests.rs<br/>217 lignes"]
        supervisor_test["supervisor_tests.rs<br/>330 lignes"]
    end

    Commands --> supervisor
    supervisor --> llm
    supervisor --> rag
    llm --> messages
    rag --> messages
    Commands --> database
    Commands --> preflight
    Init --> encryption
    Init --> fs_manager
    supervisor --> analyzer
    analyzer --> intent
    analyzer --> context
```

## ⚛️ Composants Frontend (Détaillé)

```mermaid
graph TD
    subgraph AppRoot["App.jsx"]
        App["App<br/>Routing & State"]
    end

    subgraph Pages["Pages/Views"]
        Chat["ChatInterface.jsx<br/>635 lignes"]
        Dash["Dashboard.jsx<br/>263 lignes"]
        Onboard["OnboardingWizard.jsx<br/>239 lignes"]
        Pre["PreflightCheck.jsx<br/>204 lignes"]
        Test["TestConsole.jsx<br/>594 lignes"]
        Knowledge["KnowledgeView.jsx<br/>247 lignes"]
    end

    subgraph ChatComponents["chat/"]
        ChatInput["ChatInput.jsx<br/>272 lignes"]
        MessageBubble["MessageBubble.jsx<br/>325 lignes"]
        ThinkingBubble["ThinkingBubble.jsx<br/>48 lignes"]
    end

    subgraph LayoutComponents["layout/"]
        MainLayout["MainLayout.jsx<br/>220 lignes"]
        Rail["Rail.jsx<br/>416 lignes"]
        TitleBar["TitleBar.jsx<br/>81 lignes"]
        SessionItem["SessionItem.jsx<br/>154 lignes"]
        FilesDropdown["FilesDropdown.jsx<br/>244 lignes"]
        HeaderActions["HeaderActions.jsx<br/>81 lignes"]
        SettingsDropdown["SettingsDropdown.jsx<br/>157 lignes"]
    end

    subgraph Shared["Shared"]
        ErrorBoundary["ErrorBoundary.jsx<br/>80 lignes"]
    end

    subgraph StateManagement["State"]
        AppStore["appStore.js<br/>Zustand"]
        useChatStream["useChatStream.js<br/>Hook"]
    end

    App --> MainLayout
    MainLayout --> Rail
    MainLayout --> TitleBar
    MainLayout --> Pages
    Chat --> ChatComponents
    Chat --> useChatStream
    Rail --> SessionItem
    Rail --> FilesDropdown
    App --> ErrorBoundary
    Pages --> AppStore
```

## 📊 Statistiques Complètes

### Backend Rust

| Métrique | Valeur |
|----------|--------|
| **Fichiers** | 27 |
| **Lignes de code** | 11,848 |
| **Fonctions** | 382 |
| **Structs** | 47 |
| **Enums** | 8 |
| **Implémentations** | ~50 |
| **Tests unitaires** | 125 |

### Frontend React

| Métrique | Valeur |
|----------|--------|
| **Fichiers** | 26 |
| **Lignes de code** | 5,001 |
| **Composants** | ~20 |
| **Hooks personnalisés** | 2 |
| **Stores Zustand** | 1 |

### Total Projet

| Métrique | Valeur |
|----------|--------|
| **Fichiers source** | 53 |
| **Lignes de code totales** | ~16,850 |
| **Ratio Backend/Frontend** | 70% / 30% |

## 🔗 Commandes Tauri (24 commandes)

### Gestion des Sessions
| Commande | Description |
|----------|-------------|
| `initialize_app` | Initialise l'application et les acteurs |
| `create_session` | Crée une nouvelle session de chat |
| `list_sessions` | Liste toutes les sessions |
| `delete_session` | Supprime une session |
| `update_session` | Met à jour le titre/config d'une session |
| `toggle_session_favorite` | Bascule le favori d'une session |
| `get_session_messages` | Récupère les messages d'une session |

### Chat & LLM
| Commande | Description |
|----------|-------------|
| `debug_chat` | Envoie un message et stream la réponse |
| `download_model` | Télécharge le modèle LLM |
| `check_model_exists` | Vérifie si le modèle existe |

### Fichiers & Documents
| Commande | Description |
|----------|-------------|
| `upload_file_for_session` | Upload un fichier pour une session |
| `get_session_files` | Liste les fichiers d'une session |
| `delete_file` | Supprime un fichier |
| `save_generated_file` | Sauvegarde un fichier généré |
| `list_library_files` | Liste les fichiers de la bibliothèque |
| `link_library_file_to_session` | Lie un fichier bibliothèque à une session |
| `reindex_library` | Réindexe la bibliothèque RAG |

### Dossiers
| Commande | Description |
|----------|-------------|
| `create_folder` | Crée un dossier |
| `list_folders` | Liste les dossiers |
| `delete_folder` | Supprime un dossier |
| `move_session_to_folder` | Déplace une session dans un dossier |
| `move_file_to_folder` | Déplace un fichier dans un dossier |

### Diagnostics
| Commande | Description |
|----------|-------------|
| `run_quick_preflight_check` | Vérification rapide du système |
| `run_diagnostic_category` | Exécute une catégorie de tests |

## 📦 Dépendances Externes

### Backend (Cargo.toml)

#### Framework & Runtime
- `tauri` - Framework desktop
- `tokio` - Runtime async
- `async-trait` - Traits async
- `futures` - Primitives async

#### Base de données
- `sqlx` - ORM async SQLite
- `lancedb` - Base vectorielle
- `arrow` - Format columnar

#### IA & ML
- `fastembed` - Embeddings locaux

#### Sécurité
- `aes-gcm` - Chiffrement AES-256
- `rand` - Générateur aléatoire

#### Utilitaires
- `serde` / `serde_json` - Sérialisation
- `chrono` - Date/heure
- `uuid` - Identifiants uniques
- `tracing` - Logging structuré
- `thiserror` - Gestion d'erreurs
- `reqwest` - Client HTTP
- `regex` - Expressions régulières

#### Extraction de texte
- `pdf-extract` - Extraction PDF
- `docx-rs` - Extraction DOCX
- `zip` - Archives ZIP

### Frontend (package.json)

#### Core
- `react` / `react-dom` - Framework UI
- `zustand` - State management

#### UI
- `lucide-react` - Icônes
- `tailwind-merge` - Utilitaires CSS
- `clsx` - Classes conditionnelles
- `react-hot-toast` - Notifications

#### i18n
- `i18next` - Internationalisation
- `react-i18next` - Bindings React
- `i18next-browser-languagedetector` - Détection langue

#### Tauri
- `@tauri-apps/api` - API Tauri
- `@tauri-apps/plugin-shell` - Plugin shell
- `@tauri-apps/plugin-dialog` - Plugin dialogues

## ⚠️ Points d'Attention

### Avertissements Détectés

| Fichier | Issue | Sévérité |
|---------|-------|----------|
| `database.rs` | 54 `unwrap()` (dans tests principalement) | 🟡 Moyen |
| `text_extract.rs` | 6 `unwrap()` | 🟡 Moyen |
| `supervisor.rs` | 11 `unwrap()` | 🟡 Moyen |
| `rag.rs` | 1 bloc `unsafe` | 🟠 À documenter |

### Cohérence Tauri
✅ **Validée** - Toutes les 24 commandes frontend existent dans le backend.

## 🔄 Flux de Données

```mermaid
sequenceDiagram
    participant U as User
    participant F as Frontend
    participant T as Tauri IPC
    participant S as Supervisor
    participant L as LlmActor
    participant R as RagActor
    participant DB as SQLite
    participant V as LanceDB

    U->>F: Envoie message
    F->>T: invoke("debug_chat")
    T->>S: process_message()
    S->>R: search_documents()
    R->>V: Vector search
    V-->>R: Résultats RAG
    R-->>S: SearchResults
    S->>L: generate_stream()
    L-->>S: Token stream
    S-->>T: Emit events
    T-->>F: Stream tokens
    F-->>U: Affiche réponse
    S->>DB: save_message()
```

## 📁 Structure des Fichiers Analysés

### Backend (`apps/core/src/`)

```
src/
├── main.rs                 # Entry point (1532 lignes)
├── database.rs             # SQLite operations (1260 lignes)
├── diagnostics.rs          # Test system (1803 lignes)
├── encryption.rs           # AES-256-GCM (196 lignes)
├── error.rs                # AppError enum (119 lignes)
├── fs_manager.rs           # Path management (159 lignes)
├── models.rs               # Data models (81 lignes)
├── preflight.rs            # System checks (319 lignes)
├── rate_limiter.rs         # Rate limiting (81 lignes)
├── text_extract.rs         # PDF/DOCX extraction (147 lignes)
├── actors/
│   ├── mod.rs              # Module exports
│   ├── supervisor.rs       # Actor orchestrator (746 lignes)
│   ├── llm.rs              # LLM actor (820 lignes)
│   ├── rag.rs              # RAG actor (554 lignes)
│   ├── messages.rs         # Actor messages (132 lignes)
│   └── traits.rs           # Actor traits (49 lignes)
├── brain/
│   ├── mod.rs              # Module exports
│   ├── analyzer.rs         # Message analyzer (327 lignes)
│   ├── intent.rs           # Intent detection (302 lignes)
│   ├── context_packet.rs   # Context structure (99 lignes)
│   ├── keywords.rs         # Keyword extraction (61 lignes)
│   ├── complexity.rs       # Complexity scoring (86 lignes)
│   └── semantic_intent.rs  # Semantic analysis (143 lignes)
└── tests/
    ├── mod.rs              # Test module
    ├── chaos_test.rs       # Resilience tests (196 lignes)
    ├── download_tests.rs   # Download tests (217 lignes)
    └── supervisor_tests.rs # Supervisor tests (330 lignes)
```

### Frontend (`apps/desktop-ui/src/`)

```
src/
├── App.jsx                 # Root component
├── main.jsx                # Entry point
├── i18n.js                 # i18n config
├── components/
│   ├── chat/
│   │   ├── ChatInterface.jsx   # Main chat (635 lignes)
│   │   ├── ChatInput.jsx       # Input component (272 lignes)
│   │   ├── MessageBubble.jsx   # Message display (325 lignes)
│   │   └── ThinkingBubble.jsx  # Loading state (48 lignes)
│   ├── dashboard/
│   │   └── Dashboard.jsx       # Dashboard view (263 lignes)
│   ├── diagnostics/
│   │   ├── TestConsole.jsx     # Test console (594 lignes)
│   │   └── index.js            # Exports
│   ├── layout/
│   │   ├── MainLayout.jsx      # Main layout (220 lignes)
│   │   ├── Rail.jsx            # Sidebar (416 lignes)
│   │   ├── TitleBar.jsx        # Title bar (81 lignes)
│   │   ├── SessionItem.jsx     # Session item (154 lignes)
│   │   ├── FilesDropdown.jsx   # Files menu (244 lignes)
│   │   ├── HeaderActions.jsx   # Header actions (81 lignes)
│   │   └── SettingsDropdown.jsx # Settings (157 lignes)
│   ├── onboarding/
│   │   ├── OnboardingWizard.jsx # Setup wizard (239 lignes)
│   │   └── SessionWizard.jsx   # Session setup (211 lignes)
│   ├── preflight/
│   │   └── PreflightCheck.jsx  # System check (204 lignes)
│   ├── views/
│   │   └── KnowledgeView.jsx   # Knowledge base (247 lignes)
│   └── ErrorBoundary.jsx       # Error boundary (80 lignes)
├── hooks/
│   └── useChatStream.js        # Chat streaming hook
├── lib/
│   ├── logger.js               # Logging utility
│   └── utils.js                # Utility functions
└── store/
    └── appStore.js             # Zustand store
```

---

*Document généré par `scripts/analyze-codebase.ps1`*

