# AGENTS.md

This file provides guidance to agents when working with code in this repository.

## Project Structure & Stack
- **Monorepo**:
  - `apps/core`: Rust Backend (Tauri) + SQLite + Actors (Tokio).
  - `apps/desktop-ui`: React Frontend + Vite + Tailwind CSS.
- **Critical Path Rule**: NEVER rely on relative paths like `../data`. ALWAYS use `apps/core/src/fs_manager.rs` (`PortablePathManager`).

## Commands (Run from Root)
- **Dev**: `npm run tauri dev` (Starts both Rust backend and React frontend).
- **Build**: `npm run tauri build`.
- **Lint**: `npm run lint` (JS & Rust).
- **Format**: `npm run format`.
- **Rust-specific**: Run cargo commands inside `apps/core` context if needed, but prefer npm scripts from root.

## Architecture Guidelines
- **Actor System**: The backend is NOT MVC. It uses an Actor model (`Supervisor`, `LLM`, `RAG`) managed in `apps/core/src/actors/`.
- **State Management**: `AppState` in `main.rs` holds the `SupervisorHandle`. This is the ONLY entry point for business logic.
- **Database**: Schema is defined in `apps/core/src/database.rs`. NO external `.sql` migration files.
- **IPC**: Frontend communicates via `invoke` (commands defined in `main.rs`) and listens to events (`listen`).

## Gotchas
- **File System**: `PortablePathManager` behaves differently in Debug (workspace root) vs Release (executable dir).
- **Frontend Config**: `vite.config.js` is tuned for Tauri (fixed port 1420). DO NOT change port without updating `tauri.conf.json`.