# ⚙️ Processus et Flux Métier

Ce document détaille les workflows critiques de l'application, du démarrage à l'interaction utilisateur.

## 1. Démarrage & Preflight (Bootstrap)

Avant que l'utilisateur puisse voir l'interface principale, une série de vérifications est effectuée.

```mermaid
sequenceDiagram
    participant UI as React UI
    participant Rust as Backend (Rust)
    participant FS as File System
    participant Llama as Llama Server

    UI->>Rust: invoke('run_preflight_check')
    Rust->>FS: Vérifier structure dossiers
    Rust->>FS: Vérifier présence Modèle (>3GB)
    Rust->>Llama: Tentative démarrage (Port 18080)
    Llama-->>Rust: HTTP 200 OK (Health Check)
    Rust-->>UI: PreflightStatus (OK/Error)
```

## 2. Onboarding (Installation Modèle)

Si le Preflight échoue (modèle manquant), l'utilisateur est redirigé vers l'assistant d'onboarding.

```mermaid
sequenceDiagram
    participant UI as React UI
    participant Rust as Backend
    participant HuggingFace as HF (Internet)

    UI->>Rust: invoke('download_model')
    loop Progression
        Rust->>HuggingFace: GET (Range Headers)
        HuggingFace-->>Rust: Chunk Data
        Rust->>UI: Event 'download-progress'
    end
    Rust-->>UI: Download Complete
```

## 3. Workflow de Conversation (Chat)

Le cœur de l'application.

```mermaid
sequenceDiagram
    participant User
    participant UI as React UI
    participant Supervisor as Actor: Supervisor
    participant Brain as Brain Module
    participant RAG as Actor: RAG
    participant LLM as Actor: LLM

    User->>UI: Envoie message
    UI->>Supervisor: invoke('debug_chat')

    rect rgb(200, 240, 200)
        Note over Supervisor,Brain: Analyse Intention
        Supervisor->>Brain: analyze(text)
        Brain-->>Supervisor: ContextPacket (Intent, Keywords)
    end

    alt Intent requires RAG
        Supervisor->>RAG: search(query)
        RAG-->>Supervisor: Documents trouvés
    end

    Supervisor->>Supervisor: Construire Prompt System + Context

    Supervisor->>LLM: stream_generate()
    loop Streaming
        LLM-->>Supervisor: Token
        Supervisor-->>UI: Event 'chat-token'
    end

    Supervisor->>Supervisor: Sauvegarder Réponse Complète (DB)
```

## 4. Ingestion de Documents (RAG)

Import de fichiers pour enrichir la base de connaissances.

```mermaid
sequenceDiagram
    participant UI as KnowledgeView
    participant Main as Main.rs
    participant Extract as text_extract.rs
    participant RAG as Actor: RAG
    participant VectorDB as LanceDB

    UI->>Main: invoke('upload_file_for_session')
    Main->>Extract: extract_text_from_file()
    Extract-->>Main: Contenu texte (UTF-8/PDF/DOCX)
    Main->>Main: Écrire fichier original sur disque
    Main->>Main: Insert SQLite (library_files + session_files_link)
    Main->>RAG: ingest_content(text, metadata)

    RAG->>RAG: Split Text (Chunks avec overlap)
    RAG->>RAG: Generate Embeddings (FastEmbed 384-dim)
    RAG->>VectorDB: Insert Vectors avec tag file:{uuid}
    RAG-->>UI: Ingestion Complete
```

### Formats Supportés

| Extension                      | Extraction                | Crate         |
| ------------------------------ | ------------------------- | ------------- |
| `.txt`, `.md`, `.csv`, `.json` | UTF-8 direct              | N/A           |
| `.pdf`                         | `extract_text_from_mem()` | `pdf-extract` |
| `.docx`, `.doc`                | Itération paragraphes     | `docx-rs`     |

## 5. Liaison de Fichiers Existants

Lors de la création d'une session, l'utilisateur peut sélectionner des fichiers de la bibliothèque.

```mermaid
sequenceDiagram
    participant UI as SessionWizard
    participant Main as Main.rs
    participant DB as SQLite

    UI->>Main: invoke('link_library_file_to_session')
    Main->>DB: get_library_file(file_id)
    DB-->>Main: LibraryFile (vérifie existence)
    Main->>DB: link_file_to_session(session_id, file_id)
    Main-->>UI: OK (pas de ré-ingestion)
```

> **Note** : Les vecteurs existent déjà dans LanceDB, seule la table de liaison est mise à jour.

## 6. Inférence LLM (Détail Technique)

Comment le backend Rust communique avec le processus `llama-server`.

1.  **Rust** envoie une requête HTTP POST à `http://127.0.0.1:8080/completion`.
2.  **Payload JSON** contient le prompt, la température, `stream: true`.
3.  **Llama-server** répond en SSE (Server-Sent Events).
4.  **Rust** lit le flux ligne par ligne, parse `data: {...}` et extrait le champ `content`.

## 7. Diagnostics

Le module de diagnostics permet de tester la chaîne entière sans interaction utilisateur manuelle.

- **Test RAG** : Crée un fichier temporaire, l'ingère, fait une recherche, vérifie le résultat, nettoie.
- **Test LLM** : Envoie un prompt simple ("Say hello") et chronomètre la réponse.

---

_Dernière mise à jour : Novembre 2025_
