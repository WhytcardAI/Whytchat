# WhytChat V1 Features

## ✅ Existing Features

### Core Chat
-   **Local AI**: Runs GGUF models (e.g., Qwen, Llama) locally via `llama-server`.
-   **Session Management**: Create, rename, delete, and organize chats into folders.
-   **History**: Persistent chat history using SQLite.
-   **Markdown Support**: Code highlighting, tables, and rich text rendering.

### Document Management (Global Library)
-   **Global Library**: Upload files once, use them in multiple chats.
-   **File Support**: PDF, DOCX, TXT, MD.
-   **RAG Integration**: Automatic vectorization and semantic search for attached files.
-   **File Linking**: Easy "attach/detach" workflow for sessions.

### Privacy & Security
-   **Offline-First**: Zero data leaves the machine (except specifically requested model downloads).
-   **Encryption**: AES-256-GCM encryption for sensitive local data.

## 📅 Roadmap (Planned)

### Split View UI
-   **Side-by-Side**: View documents on the right while chatting on the left.
-   **Context Highlighting**: AI highlights relevant sections in the document viewer.
-   **Knowledge Graph**: Visual representation of linked concepts.

### Advanced AI
-   **Multi-Model Support**: Switch between models per session.
-   **Internet Access**: Optional browsing capability (Search Actor).
-   **Vision**: Drag-and-drop image analysis (Multimodal models).