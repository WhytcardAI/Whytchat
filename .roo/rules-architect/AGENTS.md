# Project Architecture Rules (Non-Obvious Only)

## Core Principles
- **Actor Model (Backend)**:
  - System is designed as a set of loose-coupled actors (Supervisor, LLM, RAG).
  - `Supervisor` orchestrates EVERYTHING. It's the only actor the frontend talks to (indirectly via Tauri commands).
  - Actors communicate via `tokio::sync::mpsc` channels.

## Data Flow
- **UI -> Backend**: Tauri Command -> `AppState` -> `SupervisorHandle` -> `Supervisor` Actor.
- **Backend -> UI**: `Supervisor` -> `Window::emit("event_name", payload)`.
- **Response Streaming**: The LLM actor streams tokens back to Supervisor, which emits them immediately to UI. NO buffering of full responses for display.

## State Management
- **Single Source of Truth**: `AppState` in Rust holds the actor handles and DB pool.
- **Frontend State**: Ephemeral. It should reconstruct its view based on events and DB queries (on load).

## Database
- **Migration Strategy**: Forward-only, defined in `database.rs`.
- **JSON Storage**: `model_config` is stored as a JSON blob in SQLite to allow flexible schema evolution for model parameters without table alters.