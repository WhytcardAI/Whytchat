# Repository Map: WhytChat

## Overview
WhytChat is a local-first, privacy-focused AI chat application built with Tauri v2, Rust, and React.

## Directory Structure

### Root
- `package.json`: Monorepo configuration, scripts for dev, build, lint, and format.
- `Cargo.toml`: Workspace configuration for Rust crates.
- `.gitignore`: Git ignore rules.
- `.env.example`: Template for environment variables.

### Apps

#### `apps/core` (Rust Backend)
The "Brain" of the application. Handles LLM inference, RAG, database, and orchestration.

- `Cargo.toml`: Dependencies (Tauri, Tokio, LanceDB, Sqlx, etc.).
- `tauri.conf.json`: Tauri configuration (permissions, bundle settings).
- `build.rs`: Build script.
- `src/`
    - `main.rs`: Entry point. Initializes logging, DB, FS, and Tauri. Defines commands: `debug_chat`, `download_model`, `upload_file_for_session`.
    - `fs_manager.rs`: Manages portable file paths (critical for "portable app" goal).
    - `database.rs`: SQLite interactions (sessions, messages).
    - `models.rs`: Data structures (structs for DB/API).
    - `actors/`: Actor model implementation.
        - `mod.rs`: Module definition.
        - `supervisor.rs`: Main orchestrator. Manages state, processes user messages, coordinates LLM and RAG.
        - `llm.rs`: Manages `llama-server` process and HTTP requests.
        - `rag.rs`: Manages Vector DB (LanceDB) and embedding generation.
        - `messages.rs`: Message types for actor communication.
- `tools/`: Local build tools (CMake, Protoc) for Windows reproducibility.

#### `apps/desktop-ui` (React Frontend)
The "Dumb UI" that renders state provided by the backend.

- `package.json`: Frontend dependencies (React, Vite, Tailwind, Zustand).
- `vite.config.js`: Vite configuration.
- `tailwind.config.js`: Tailwind CSS configuration.
- `src/`
    - `main.jsx`: Entry point.
    - `App.jsx`: Root component, handles routing/onboarding check.
    - `store/appStore.js`: Zustand store for global state.
    - `i18n.js`: Internationalization config.
    - `components/`: UI Components.
        - `chat/`: Chat-specific components (`ChatInterface`, `MessageBubble`, etc.).
        - `layout/`: Layout components (`MainLayout`, `RightPanel`).
        - `onboarding/`: Setup wizard (`OnboardingWizard`).
    - `locales/`: Translation files (`en/common.json`, `fr/common.json`).

### Docs
- `docs/TECHNICAL_MANUAL.md`: Comprehensive technical specification.
- `docs/ROADMAP.md`: Project goals and timeline.
- `docs/USER_GUIDE.md`: End-user documentation.
- `docs/TECHNICAL_PLAN.md`: Implementation plan.

## Dysfunctions & Issues Identified

1.  **Hardcoded Session ID**: `apps/core/src/main.rs` uses `"default-session"` in `debug_chat`.
2.  **Hardcoded Model URL**: `apps/core/src/main.rs` hardcodes the Qwen model URL. Should be configurable.
3.  **Missing CI/CD**: `.github/workflows/release.yml` is missing.
4.  **Error Handling**: RAG ingestion errors in `upload_file_for_session` are logged but don't notify the user effectively.
5.  **Security**: `upload_file_for_session` has a basic check for binary files (`\0`) which might be insufficient.

## Workflow Requirements
- **Release**: Need GitHub Action for building Tauri app on Windows, Linux, macOS.
- **Code Quality**: Need workflows for `cargo clippy`, `cargo fmt`, `eslint`, `prettier`.