# Manuel Technique WhytChat V1

Ce document consolide l'ensemble des spécifications techniques, standards de développement et détails d'implémentation de WhytChat V1. Il sert de référence unique pour les développeurs.

---

## 1. Architecture & Vision

### Les 6 Piliers de WhytChat V1

1.  **Installation Portable (Strict Confinement)**
    - **Zéro AppData :** Aucune donnée ne doit être écrite dans `%APPDATA%` ou les répertoires système.
    - **Self-Contained :** Tout (DB, logs, modèles) réside dans le dossier `data/` relatif à l'exécutable.
    - **Transportabilité :** Copier le dossier suffit pour déplacer l'application.

2.  **Onboarding Éducatif**
    - Sélection de la langue et du "système de pensée".
    - Téléchargement assisté du modèle GGUF optimal.
    - Séquence éducative "Privacy First".

3.  **Le Cerveau Cognitif (Backend Brain)**
    - **Backend Rust Orchestrator :** Le frontend est une "Coquille Vide". Toute l'intelligence est dans les acteurs Rust.
    - **Agents Neuronaux Invisibles :** Chaîne d'agents (Critique, Planificateur, Rédacteur) utilisant le modèle local.
    - **Séquence Cognitive :** Chain-of-Thought invisible pour l'utilisateur.

4.  **Session Atomique**
    - Configuration par chat (Température, System Prompt).
    - Focus Mode.

5.  **Bibliothèque de Connaissance (RAG)**
    - **Global Library :** Réservoir de documents.
    - **Clusters :** Regroupement thématique.
    - **Chat Injection :** Injection ciblée de clusters dans le contexte.

6.  **Connectivité (Tavily Toggle)**
    - **Air-Gapped par défaut.**
    - Accès web uniquement via un toggle explicite (API Tavily).

### Diagramme Logique (C4 Container)

```mermaid
graph TD
    User((Utilisateur))

    subgraph "Host Machine (Portable Directory)"
        subgraph "Frontend (Dumb UI)"
            React[React / Vite]
        end

        subgraph "Backend Brain (Rust Core)"
            API[Tauri Command Handler]

            subgraph "Cognitive Engine"
                Orchestrator[Supervisor Actor]
                Thinker[Thinking Agent]
                RAG[Knowledge Agent]
            end

            subgraph "Local Infrastructure"
                LLM_Engine[Llama.cpp Server (GGUF)]
                VectorDB[(LanceDB - Vectors)]
                RelationalDB[(SQLite - Chats/Config)]
            end
        end
    end

    External[Tavily API (Internet)]

    User <--> React
    React -- "Events / Commands" --> API
    API --> Orchestrator
    Orchestrator -- "Orchestre" --> Thinker
    Thinker -- "Inférence" --> LLM_Engine
    Orchestrator -- "Cherche" --> RAG
    RAG -- "Query" --> VectorDB
    Orchestrator -.-> |"Si Toggle ON"| External
```

---

## 2. Implémentation Technique

### Système d'Acteurs (Tokio)

Nous utilisons un système d'acteurs léger basé sur les channels `tokio` (`mpsc`).

#### Hiérarchie des Acteurs

- **Supervisor :** Point d'entrée unique. Reçoit les commandes Tauri et orchestre les tâches.
- **LlmActor :** Gère le processus enfant `llama-server.exe` et les requêtes HTTP vers l'API locale.
- **RagActor :** Gère l'indexation et la recherche dans LanceDB.

#### Pattern "Actor Handle"

Chaque acteur est composé de :

1.  **Handle (Public API) :** Struct légère contenant un `mpsc::Sender`. Expose des méthodes async typées.
2.  **Runner (Internal Logic) :** Struct contenant le `mpsc::Receiver` et l'état privé. Tourne dans une boucle `tokio::spawn`.

### Gestion des Données (PortablePathManager)

Le module `apps/core/src/fs_manager.rs` est critique. Il garantit que tous les chemins sont relatifs à l'exécutable.

- **Racine :** Détectée via `std::env::current_exe()`.
- **Structure :**
  - `data/db/` : SQLite.
  - `data/models/` : Modèles GGUF.
  - `data/vectors/` : LanceDB.

### Base de Données Vectorielle (RAG)

- **Moteur :** LanceDB (Embedded).
- **Embeddings :** `fastembed` (Rust native, ONNX).
- **Schéma :** `id` (UUID), `content` (Text), `metadata` (JSON), `vector` (Float32).

---

## 3. Standards de Développement

### Backend (Rust)

- **Sécurité :** Interdiction stricte de `unwrap()`. Utilisez `expect()` ou la gestion d'erreur (`?`).
- **Typage :** Préférez les types forts (`UserId`) aux primitifs.
- **Qualité :** Le code doit passer `cargo clippy -- -D warnings`.

### Frontend (React)

- **UI "Dumb" :** Aucune logique métier complexe. Tout est délégué au Backend.
- **État :** Zustand pour l'état UI local.
- **Code Style :** ESLint + Prettier obligatoires.

### Internationalisation (i18n)

- **Librairie :** `react-i18next`.
- **Règle :** Aucun texte en dur. Tout dans `src/locales/{lang}/common.json`.
- **Erreurs :** Le Backend renvoie des codes d'erreur (`LLM_MODEL_NOT_FOUND`), le Frontend traduit.

---

## 5. Environnement de Build (Windows)

Pour garantir une compilation reproductible et autonome sur Windows, le projet utilise des outils locaux situés dans `apps/core/tools/`.

### Pré-requis Locaux

- **Protoc** : Le compilateur Protocol Buffers est requis pour `lancedb`. Il est installé manuellement dans `apps/core/tools/protoc/bin/protoc.exe`.
- **CMake** : Requis pour la compilation de `aws-lc-sys` (dépendance transitive de `lancedb`). Installé dans `apps/core/tools/cmake/bin/cmake.exe`.

### Variables d'Environnement

Le script de lancement ou le terminal de développement doit configurer les variables suivantes avant la compilation :

```powershell
$env:PROTOC = "...\apps\core\tools\protoc\bin\protoc.exe"
$env:PATH = "...\apps\core\tools\cmake\bin;" + $env:PATH
```

### Dépendances Critiques (Cargo.toml)

Les versions des crates sont épinglées pour éviter les conflits de "Dependency Hell" fréquents avec l'écosystème Arrow/LanceDB :

- `lancedb` = "0.10" (default-features = false)
- `fastembed` = "4"
- `arrow` = "52.2.0"
- `chrono` = "0.4" (features = ["serde"])

---

## 6. Cartographie du Code (Code Map)

### Backend (`apps/core/`)

| Fichier                    | Rôle                                         |
| :------------------------- | :------------------------------------------- |
| `src/main.rs`              | Point d'entrée. Initialise FS et Acteurs.    |
| `src/fs_manager.rs`        | **CRITIQUE.** Gestion des chemins portables. |
| `src/actors/supervisor.rs` | Orchestrateur de la pensée.                  |
| `src/actors/llm.rs`        | Gestionnaire du processus `llama-server`.    |
| `src/actors/rag.rs`        | Gestionnaire de la base vectorielle.         |

### Frontend (`apps/desktop-ui/`)

| Fichier                                 | Rôle                         |
| :-------------------------------------- | :--------------------------- |
| `src/components/chat/ChatInterface.jsx` | Vue principale du chat.      |
| `src/store/appStore.js`                 | État global (Zustand).       |
| `src/locales/`                          | Fichiers de traduction JSON. |

### Données (`./data`)

_Généré au runtime, non versionné._
| Dossier | Contenu |
| :--- | :--- |
| `models/` | Fichiers `.gguf` (ex: `qwen2.5-7b...`). |
| `db/` | SQLite (`whytchat.db`). |
| `vectors/` | LanceDB tables. |
