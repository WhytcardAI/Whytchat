# RAG System Documentation

## Overview

The Retrieval-Augmented Generation (RAG) system in WhytChat V1 allows the AI to "read" and "remember" user documents. It is fully local, running efficiently on the user's hardware without sending data to the cloud.

## 🧩 Core Components

### 1. Text Extraction
-   **Supported Formats**: .txt, .md, .pdf, .docx
-   **Library**: `pdf-extract` for PDFs, `docx-rs` for Word documents.
-   **Process**: Raw text is extracted from files upon upload to the Global Library.

### 2. Embedding Generation
-   **Model**: `all-MiniLM-L6-v2` (Quantized ONNX format)
-   **Library**: `fastembed-rs`
-   **Performance**: Highly optimized for CPU inference; runs efficiently even on non-GPU machines.
-   **Chunking**: Text is split into overlapping chunks (Target: 512 chars, Overlap: 50 chars) to preserve context.

### 3. Vector Storage (LanceDB)
-   **Database**: LanceDB (embedded vector database)
-   **Storage Path**: `[AppLocalData]/vectors`
-   **Schema**:
    -   `id`: Unique chunk ID
    -   `content`: The text chunk itself
    -   `metadata`: `file:{file_id}` (Used for filtering)
    -   `vector`: 384-dimensional float array

## 🔄 Workflow

### Ingestion Pipeline
1.  User uploads a file to the Global Library.
2.  **Supervisor Actor** sends an `IngestContent` message to the **RAG Actor**.
3.  **RAG Actor**:
    *   Extracts text from the file.
    *   Splits text into chunks.
    *   Generates embeddings for each chunk.
    *   Writes chunks + vectors to LanceDB with `metadata = file:{uuid}`.

### Retrieval Pipeline
1.  User asks a question in a session.
2.  **Supervisor Actor** identifies attached files for the current session.
3.  **Supervisor Actor** requests a search from the **RAG Actor** with:
    *   `query`: The user's message.
    *   `file_ids`: List of file UUIDs active in the session.
4.  **RAG Actor**:
    *   Embeds the user's query.
    *   Queries LanceDB for the nearest neighbors (cosine similarity).
    *   **Crucial Step**: Applies a filter `metadata IN [file:id1, file:id2]` to ensure only relevant documents are searched.
5.  **Supervisor Actor** constructs the final prompt:
    ```text
    System: You are a helpful assistant.
    Context: [Content of chunk 1]
    [Content of chunk 2]
    ...
    User: [User Question]
    ```

## ⚡ Performance & Optimization

-   **Caching**: Query embeddings are cached in an LRU cache (`size=1000`) to prevent re-embedding the same questions.
-   **Filtering**: Filtering by file ID at the database level (pre-filtering) ensures speed even as the global library grows large.
-   **Asynchronous**: All RAG operations happen in a dedicated Tokio actor, preventing UI freezes.