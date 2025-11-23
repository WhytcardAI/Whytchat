# Project Coding Rules (Non-Obvious Only)

## Rust Backend (`apps/core`)
- **Strict Logic Flow**: All business logic MUST originate from `SupervisorHandle` methods. DO NOT bypass the supervisor to call `LLM` or `RAG` actors directly from Tauri commands.
- **Database Access**: 
  - Use `sqlx` query macros for compile-time verification whenever possible.
  - Schema changes MUST be added to the `init_db` function in `apps/core/src/database.rs`.
- **File I/O**:
  - NEVER use `std::fs` with hardcoded paths like `./data`.
  - ALWAYS use `PortablePathManager::*` methods to resolve paths (e.g., `models_dir()`, `session_files_dir()`).
  - This is critical for cross-platform and development/production compatibility.

## Frontend (`apps/desktop-ui`)
- **IPC Communication**:
  - Use `window.electron.ipcRenderer` style logic but via Tauri's `invoke`.
  - Listen for backend events (e.g., `chat-token`, `thinking-step`) using `listen()` from `@tauri-apps/api/event`.
  - DO NOT implement complex business logic in React components; delegate to the backend.
- **State**:
  - Use local state for UI interactions.
  - Rely on backend events to update conversation state (streaming tokens).

## General
- **No Dead Code**: Remove unused imports and functions immediately.
- **Structure**: Keep `apps/core` actors isolated. `apps/desktop-ui` components should be small and focused.