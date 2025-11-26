# AGENTS.md

This file provides guidance to agents when working with code in this repository.

## 📚 Documentation

See [Doc/README.md](Doc/README.md) for comprehensive documentation.

## Project Structure & Stack

- **Monorepo**: `apps/core` (Rust/Tauri Backend + Actors) & `apps/desktop-ui` (React/Vite Frontend).
- **Critical Path Rule**: NEVER rely on relative paths like `../data`. ALWAYS use `apps/core/src/fs_manager.rs` (`PortablePathManager`).
- **I18n**: Locales are in `apps/desktop-ui/src/locales` (UI) - check `common.json`.

## Commands (Run from Root)

- **Dev**: `npm run tauri dev` (Starts Rust backend + React frontend).
- **Test Backend**: `cargo test` inside `apps/core`.
- **Test UI (E2E)**: `npm run test:e2e` (Requires existing RELEASE build via `npm run tauri build`).
- **Lint/Format**: `npm run lint` / `npm run format`.

## Architecture Guidelines

- **Actor System**: Backend logic lives in `apps/core/src/actors/` (`Supervisor`, `LLM`, `RAG`). NOT MVC.
- **State**: `AppState` in `main.rs` is the ONLY entry point. Holds `SupervisorHandle`.
- **Brain**: Intent routing logic is hybrid (Regex + Semantic) in `apps/core/src/brain/`.
- **IA**: Uses external `llama-server` binary. Do NOT modify `apps/core/tools/` binaries.

## Gotchas

- **File System**: Debug uses workspace root, Release uses executable dir. `PortablePathManager` handles this.
- **Windows**: `PROTOC` env var must point to `apps/core/tools/protoc/bin/protoc.exe`.
- **Tauri Config**: Frontend port 1420 is hardcoded in `vite.config.js` and `tauri.conf.json`. Keep in sync.