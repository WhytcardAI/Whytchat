# WhytChat AI Coding Instructions

You are an expert AI developer working on WhytChat, a local-first AI chat application built with Tauri 2.0, Rust, and React.

## 🏗️ Project Architecture

- **Monorepo Structure**:
  - `apps/core/`: Rust backend (Tauri process). "The Brain" of the application.
  - `apps/desktop-ui/`: React/Vite frontend.
  - `data/`: Local storage for DBs, models, and vectors.

- **Backend ("The Brain")**:
  - **Actor System**: Uses `tokio` actors for async processing.
    - `Supervisor`: Orchestrates all cognitive tasks.
    - `RagActor`: Manages LanceDB vector storage and retrieval.
    - `LlmActor`: Interfaces with local LLMs (llama.cpp).
  - **State Management**: `AppState` in `main.rs` holds `InitializedState` (supervisor handle, DB pool, rate limiter).
  - **Data Layer**:
    - **SQLite** (`sqlx`): Session metadata, chat history, settings.
    - **LanceDB**: Vector embeddings for RAG.
    - **FileSystem**: Managed via `PortablePathManager` to ensure portability.

## 💻 Key Development Workflows

- **Build & Run**:
  - Dev: `npm run dev` (starts Tauri + Vite).
  - Build: `npm run tauri build`.
- **Testing**:
  - Rust Unit/Integration: `cargo test --manifest-path apps/core/Cargo.toml`.
  - E2E (Playwright): `npm run test:e2e`.
- **Linting & Formatting**:
  - Run all: `npm run lint` (includes Clippy & ESLint).
  - Rust: `cargo clippy`, `cargo fmt`.
  - JS: `npm run lint:js`.

## 🧩 Coding Conventions & Patterns

### Rust Backend (`apps/core`)

- **Error Handling**:
  - Use `crate::error::AppError` for all internal errors.
  - Consolidate external errors (sqlx, io, etc.) into `AppError` using `?`.
  - Tauri commands should return `Result<T, String>` (map `AppError` to string).

- **Actor Communication**:
  - Do NOT call actor methods directly. Use `*Handle` structs (e.g., `SupervisorHandle`).
  - Use `tokio::sync::mpsc` for messaging and `oneshot` for responses.

- **Tauri Commands**:
  - Define in `main.rs` or specific modules.
  - Always check initialization: `if !state.is_initialized.load(Ordering::SeqCst) { ... }`.
  - Use `check_rate_limit_and_get_resources` for chat-related commands.

- **Logging**:
  - Use `tracing::{info, warn, error}`.
  - Do NOT use `println!`.

### Frontend (`apps/desktop-ui`)

- **Tauri Integration**:
  - Use `@tauri-apps/api` to invoke backend commands.
  - Handle errors gracefully assuming backend returns string errors.

## 🛡️ Security & Data

- **Secrets**: `LLAMA_AUTH_TOKEN` is generated at runtime.
- **Paths**: ALWAYS use `PortablePathManager` for file paths. Never hardcode absolute paths.
- **Models**: Stored in `data/models`. Validated by `validate_model_file`.

## 🚀 Specific Implementation Details

- **RAG System**:
  - Located in `apps/core/src/actors/rag.rs`.
  - Uses `fastembed` for embeddings and `lancedb` for storage.
  - Ingestion chunks text with overlap.

- **Database**:
  - Schema migrations in `apps/core/migrations/`.
  - Use `sqlx::query_as!` macros for type-safe queries when possible.
