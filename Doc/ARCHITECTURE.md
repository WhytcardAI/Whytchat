# WhytChat V1 Architecture

## Overview

WhytChat V1 is a **local-first AI application** built on a modern, high-performance stack. It leverages Rust for backend logic and system operations, combined with a React-based frontend for a responsive user interface.

## 🛠️ Tech Stack

### Frontend (UI)
- **Framework**: React 18 + Vite
- **Language**: JavaScript (ES Modules)
- **Styling**: Tailwind CSS + clsx/tailwind-merge
- **State Management**: Zustand
- **Internationalization**: i18next
- **Drag & Drop**: @dnd-kit

### Backend (Core)
- **Framework**: Tauri 2.0 (RC)
- **Language**: Rust (v1.80.0+)
- **Database**: SQLite (via SQLx)
- **Vector Database**: LanceDB (local embedding storage)
- **AI Engine**: `llama-server` (GGUF models) + custom RAG pipeline

## 📐 System Design

### Global Library Architecture

Unlike traditional session-based chat apps, WhytChat uses a **Global Library** for document management:

1.  **Upload Once**: Files are uploaded to a central library (`library_files` table).
2.  **Link Anywhere**: A many-to-many relationship (`session_files_link`) allows a single file to be referenced in multiple chat sessions without duplication.
3.  **Centralized RAG**: Vectors are generated once per file and stored in LanceDB. The RAG system filters by file ID during retrieval.

### Database Schema (SQLite)

*   **`sessions`**: Stores chat sessions (UUID, title, model config).
*   **`messages`**: Stores chat history linked to sessions.
*   **`folders`**: Hierarchical organization for sessions.
*   **`library_files`**: The master list of all uploaded documents.
*   **`session_files_link`**: Connects library files to sessions.

### RAG Pipeline (Rust Actors)

The backend uses an actor-based concurrency model:

1.  **Supervisor Actor**: Orchestrates the flow. Receives user input, decides on RAG usage, and dispatches tasks.
2.  **RAG Actor**:
    *   **Ingest**: Chunks text, generates embeddings (AllMiniLML6V2), and stores them in LanceDB.
    *   **Search**: Retrieves relevant chunks based on cosine similarity, filtered by file IDs linked to the current session.
3.  **LLM Actor**: Interfaces with the local `llama-server` API to generate responses using the retrieved context.

### Planned UI: Split View

The target interface for WhytChat V1 is a **Split View** design:
-   **Left Pane**: Standard chat interface.
-   **Right Pane**: Document viewer/Knowledge graph.
-   **Goal**: To allow users to see the source material side-by-side with the AI's analysis.

## 🔒 Security & Privacy

-   **Local Storage**: All data resides in `AppLocalData`.
-   **Offline Capable**: No internet connection required after model download.
-   **Encryption**: Sensitive fields (like future API keys) are encrypted using AES-256-GCM.