# 🏗️ Architecture du Système

Ce document décrit l'architecture de haut niveau de WhytChat, une application de chat locale sécurisée utilisant Tauri, Rust et des modèles d'IA locaux.

## 🧩 Vue d'Ensemble

WhytChat suit une architecture **Monolithique Modulaire** distribuée en deux processus principaux (Frontend & Backend) communiquant via IPC.

### Diagramme de Haut Niveau

```mermaid
graph TD
    subgraph "Frontend (Electron-like)"
        UI[React UI] -->|Invoke / Events| IPC[Tauri IPC Bridge]
        Store[Zustand Store] <--> UI
    end

    subgraph "Backend (Rust Core)"
        IPC --> Main[Main Entry Point]
        Main --> AppState[AppState (Global Lock)]

        subgraph "Actor System (Tokio)"
            AppState --> Supervisor[Supervisor Actor]
            Supervisor --> LLM[LLM Actor]
            Supervisor --> RAG[RAG Actor]
            Supervisor --> Brain[Brain Analyzer]
        end

        subgraph "Persistence"
            RAG --> VectorDB[LanceDB (Vectors)]
            Main --> SQLite[SQLite (Chat History)]
            Main --> FS[PortablePathManager (Files)]
        end
    end

    LLM -->|HTTP| LlamaServer[llama-server.exe (GGUF)]
```

---

## 🛠️ Stack Technique

### Frontend (`apps/desktop-ui`)

- **Framework** : React 18
- **Build Tool** : Vite
- **Styling** : Tailwind CSS
- **State Management** : Zustand (avec persistance)
- **Langue** : JavaScript (ES6+)

### Backend (`apps/core`)

- **Langage** : Rust (Edition 2021)
- **Framework App** : Tauri 2.0 (Beta/RC)
- **Async Runtime** : Tokio
- **Base de Données** :
  - Relationnelle : `sqlx` (SQLite)
  - Vectorielle : `lancedb` + `fastembed`
- **Architecture** : Actor Model (implémentation custom sur Tokio Channels)

### Intelligence Artificielle

- **Inférence LLM** : `llama-server` (binaire externe piloté via HTTP)
- **Modèle LLM** : GGUF (ex: Qwen 2.5 7B)
- **Embeddings** : ONNX Runtime via `fastembed` (`AllMiniLML6V2`)
- **Classification** : "The Brain" (Regex + Fallback Sémantique)

---

## 🧠 Le Module "Brain"

Le "Brain" est un module d'analyse pré-LLM conçu pour router les requêtes intelligemment sans latence.

```mermaid
graph LR
    Input[User Input] --> Intent{Intent Classification}

    Intent -->|Regex Match| FastPath[Fast Path (~1ms)]
    Intent -->|No Match| Semantic[Semantic Fallback (~50ms)]

    FastPath --> ContextBuilder
    Semantic --> ContextBuilder

    ContextBuilder -->|Context Packet| Supervisor
```

Voir [IA_INTERNALS.md](./IA_INTERNALS.md) pour les détails.

---

## 💾 Gestion des Données

### Système de Fichiers (PortablePathManager)

Pour assurer la portabilité (notamment sur clé USB), aucun chemin absolu n'est utilisé en dur. Le `fs_manager.rs` résout dynamiquement les chemins :

- `data/` : Base de données, vecteurs, modèles.
- `config/` : Fichiers de configuration.

### Base de Données (SQLite)

- **Sessions** : Conversations actives.
- **Messages** : Historique des chats.
- **Library_Files** : Registre global des fichiers importés.
- **Session_Files_Link** : Table de liaison (Many-to-Many) entre Sessions et Fichiers.

---

## 🔒 Sécurité

- **Chiffrement** : Les configurations sensibles (clés API si existantes, paramètres système) sont chiffrées au repos (`encryption.rs`) utilisant `Aes256Gcm`.
- **Isolation** : Le LLM tourne dans un processus séparé. Le Frontend n'a pas d'accès direct au disque (passe par le Backend).
- **Contrôle d'Accès** : Les fichiers ne sont accessibles au RAG que s'ils sont explicitement liés à la session active.

---

_Dernière mise à jour : Novembre 2025_
