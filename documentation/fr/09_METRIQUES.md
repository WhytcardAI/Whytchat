# 📊 Métriques du Codebase - WhytChat V1

> Statistiques détaillées et dépendances du projet

---

## 📑 Table des Matières

1. [Répartition du Code](#1-répartition-du-code)
2. [Commandes Tauri IPC](#2-commandes-tauri-ipc)
3. [Dépendances](#3-dépendances)
4. [Complexité](#4-complexité)

---

## 1. Répartition du Code

### 1.1 Vue Globale

| Catégorie | Fichiers | Lignes | % Total |
|-----------|----------|--------|---------|
| Backend Rust | 22 | ~6,000 | 74% |
| Frontend React | 8+ | ~1,500 | 18% |
| Tests | 4 | ~400 | 5% |
| Config | 5 | ~200 | 3% |
| **Total** | **~40** | **~8,100** | 100% |

### 1.2 Détail Backend Rust

| Module | Fichiers | Lignes | Description |
|--------|----------|--------|-------------|
| Fondations | 2 | ~230 | models.rs, error.rs |
| Entry Point | 1 | ~1,500 | main.rs (22 commandes) |
| Actors | 4 | ~560 | supervisor, messages, rag, llm |
| Brain | 6 | ~750 | analyzer, intent, keywords, etc. |
| Database | 3 | ~775 | database, encryption, fs_manager |
| Utilitaires | 4 | ~1,620 | diagnostics, preflight, etc. |
| Tests | 4 | ~400 | supervisor, brain, flow, chaos |

### 1.3 Détail Frontend React

| Catégorie | Fichiers | Lignes | Description |
|-----------|----------|--------|-------------|
| Store | 1 | ~390 | appStore.js |
| Hooks | 1 | ~170 | useChatStream.js |
| Layout | 3 | ~435 | MainLayout, TitleBar, Rail |
| Chat | 3 | ~535 | ChatInterface, ChatInput, MessageBubble |
| Config | 4 | ~100 | vite, tailwind, eslint, i18n |

---

## 2. Commandes Tauri IPC

### 2.1 Total : 22 Commandes

```
┌─────────────────────────────────────────────┐
│           COMMANDES TAURI IPC               │
├─────────────────────────────────────────────┤
│  Sessions (7)     │  Messages (2)           │
│  ├─ create        │  ├─ debug_chat          │
│  ├─ update        │  └─ get_messages        │
│  ├─ delete        │                         │
│  ├─ list          │  Files (6)              │
│  ├─ get           │  ├─ upload              │
│  ├─ toggle_fav    │  ├─ get_session_files   │
│  └─ move_folder   │  ├─ list_library        │
│                   │  ├─ delete              │
│  Folders (4)      │  ├─ save_generated      │
│  ├─ list          │  └─ reindex             │
│  ├─ create        │                         │
│  ├─ delete        │  System (4)             │
│  └─ move_file     │  ├─ initialize_app      │
│                   │  ├─ download_model      │
│                   │  ├─ run_diagnostics     │
│                   │  └─ preflight_check     │
└─────────────────────────────────────────────┘
```

### 2.2 Détail par Catégorie

**Sessions (7)** :
| Commande | Params | Return |
|----------|--------|--------|
| `create_session` | title, language, system_prompt, temperature | session_id |
| `update_session` | session_id, title, model_config | () |
| `delete_session` | session_id | () |
| `list_sessions` | - | Vec<Session> |
| `get_session` | session_id | Session |
| `toggle_session_favorite` | session_id | is_favorite |
| `move_session_to_folder` | session_id, folder_id | () |

**Messages (2)** :
| Commande | Params | Return |
|----------|--------|--------|
| `debug_chat` | session_id, message | response |
| `get_session_messages` | session_id | Vec<Message> |

**Files (6)** :
| Commande | Params | Return |
|----------|--------|--------|
| `upload_file_for_session` | session_id, file_path | file_id |
| `get_session_files` | session_id | Vec<File> |
| `list_library_files` | - | Vec<LibraryFile> |
| `delete_file` | file_id | () |
| `save_generated_file` | content, filename | path |
| `reindex_library` | - | () |

**Folders (4)** :
| Commande | Params | Return |
|----------|--------|--------|
| `list_folders` | - | Vec<Folder> |
| `create_folder` | name, color, folder_type | Folder |
| `delete_folder` | folder_id | () |
| `move_file_to_folder` | file_id, folder_id | () |

**System (4)** :
| Commande | Params | Return |
|----------|--------|--------|
| `initialize_app` | - | () |
| `download_model` | url | () |
| `run_diagnostics` | - | Vec<DiagResult> |
| `preflight_check` | - | PreflightResult |

---

## 3. Dépendances

### 3.1 Rust (Cargo.toml)

**Core** :
| Crate | Version | Usage |
|-------|---------|-------|
| `tauri` | 2.0.0-rc | Framework desktop |
| `tokio` | 1.x | Async runtime |
| `sqlx` | 0.8 | SQLite ORM |
| `serde` | 1.x | Serialization |

**AI/ML** :
| Crate | Version | Usage |
|-------|---------|-------|
| `lancedb` | 0.10 | Vector DB |
| `fastembed` | 4 | Embeddings |
| `reqwest` | 0.12 | HTTP client (llama) |

**Crypto** :
| Crate | Version | Usage |
|-------|---------|-------|
| `aes-gcm` | 0.10.3 | Chiffrement |
| `base64` | 0.22 | Encoding |
| `rand` | 0.8 | RNG |

**Utilitaires** :
| Crate | Version | Usage |
|-------|---------|-------|
| `tracing` | 0.1 | Logging |
| `thiserror` | 1.x | Error handling |
| `uuid` | 1.x | UUID generation |
| `chrono` | =0.4.38 | Dates (pinné) |

### 3.2 JavaScript (package.json)

**Core** :
| Package | Version | Usage |
|---------|---------|-------|
| `react` | ^18.2 | UI framework |
| `react-dom` | ^18.2 | React DOM |
| `@tauri-apps/api` | ^2.0 | Tauri bridge |

**State & Routing** :
| Package | Version | Usage |
|---------|---------|-------|
| `zustand` | ^4.4 | State management |
| `react-router-dom` | ^6.x | Routing |

**UI** :
| Package | Version | Usage |
|---------|---------|-------|
| `tailwindcss` | ^3.4 | Styling |
| `lucide-react` | ^0.x | Icons |
| `react-hot-toast` | ^2.4 | Notifications |

**i18n** :
| Package | Version | Usage |
|---------|---------|-------|
| `i18next` | ^23.x | Internationalization |
| `react-i18next` | ^13.x | React bindings |

**Dev** :
| Package | Version | Usage |
|---------|---------|-------|
| `vite` | ^5.x | Build tool |
| `eslint` | ^8.x | Linting |
| `playwright` | ^1.x | E2E tests |

---

## 4. Complexité

### 4.1 Fichiers les Plus Complexes

| Rang | Fichier | Lignes | Complexité | Raison |
|------|---------|--------|------------|--------|
| 1 | `main.rs` | ~1,500 | Haute | 22 commandes, state mgmt |
| 2 | `diagnostics.rs` | ~1,000 | Haute | 28 tests, async |
| 3 | `llm.rs` | ~600 | Haute | Streaming, circuit breaker |
| 4 | `database.rs` | ~480 | Moyenne | CRUD, encryption |
| 5 | `appStore.js` | ~390 | Moyenne | State centralisé |

### 4.2 Métriques de Complexité

**Cyclomatic Complexity (estimée)** :

| Module | Score | Évaluation |
|--------|-------|------------|
| `supervisor.rs` | 15 | Modéré |
| `llm.rs` | 20 | Élevé |
| `brain/analyzer.rs` | 12 | Modéré |
| `database.rs` | 18 | Modéré-Élevé |
| `appStore.js` | 25 | Élevé |

### 4.3 Couverture de Tests

| Module | Tests | Coverage | Status |
|--------|-------|----------|--------|
| Supervisor | 0 | 0% | 🔴 Cassé |
| Brain | 0 | 0% | 🔴 Cassé |
| RAG | 0 | 0% | ⚠️ Aucun |
| LLM | 0 | 0% | ⚠️ Aucun |
| Database | 0 | 0% | ⚠️ Aucun |
| Diagnostics | 28 | ~60% | ✅ OK |
| **Global** | **28** | **~15%** | ⚠️ |

---

## 📊 Résumé Visuel

```
╔════════════════════════════════════════════════════════════╗
║                    WHYTCHAT V1 METRICS                      ║
╠════════════════════════════════════════════════════════════╣
║                                                             ║
║  Code Lines        ████████████████████████░░  8,100        ║
║  Rust              ████████████████████░░░░░░  6,000 (74%)  ║
║  React             ████░░░░░░░░░░░░░░░░░░░░░░  1,500 (18%)  ║
║                                                             ║
║  IPC Commands      ██████████████████████████  22           ║
║  Rust Crates       ████████████████████░░░░░░  ~20          ║
║  NPM Packages      ████████████████░░░░░░░░░░  ~15          ║
║                                                             ║
║  Test Coverage     ███░░░░░░░░░░░░░░░░░░░░░░░  ~15%         ║
║  Irregularities    ██████████████████░░░░░░░░  18           ║
║                                                             ║
╚════════════════════════════════════════════════════════════╝
```

---

## 📚 Voir Aussi

- [01_VUE_ENSEMBLE.md](01_VUE_ENSEMBLE.md) - Vue d'ensemble
- [07_IRREGULARITES.md](07_IRREGULARITES.md) - Problèmes identifiés
- [08_RECOMMANDATIONS.md](08_RECOMMANDATIONS.md) - Actions suggérées

---

_Document généré le 27 novembre 2025_
