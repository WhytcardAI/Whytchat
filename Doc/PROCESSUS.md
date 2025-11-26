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
    participant UI as React UI
    participant Main as Main.rs
    participant RAG as Actor: RAG
    participant VectorDB as LanceDB

    UI->>Main: invoke('upload_file')
    Main->>Main: Écrire fichier sur disque
    Main->>Main: Insert SQLite (Metadata)
    Main->>RAG: ingest_file(path)
    
    RAG->>RAG: Split Text (Chunks)
    RAG->>RAG: Generate Embeddings (FastEmbed)
    RAG->>VectorDB: Insert Vectors
    RAG-->>UI: Ingestion Complete
```

## 5. Inférence LLM (Détail Technique)

Comment le backend Rust communique avec le processus `llama-server`.

1.  **Rust** envoie une requête HTTP POST à `http://127.0.0.1:8080/completion`.
2.  **Payload JSON** contient le prompt, la température, `stream: true`.
3.  **Llama-server** répond en SSE (Server-Sent Events).
4.  **Rust** lit le flux ligne par ligne, parse `data: {...}` et extrait le champ `content`.

## 6. Diagnostics

Le module de diagnostics permet de tester la chaîne entière sans interaction utilisateur manuelle.

*   **Test RAG** : Crée un fichier temporaire, l'ingère, fait une recherche, vérifie le résultat, nettoie.
*   **Test LLM** : Envoie un prompt simple ("Say hello") et chronomètre la réponse.

---
*Dernière mise à jour : Novembre 2025*