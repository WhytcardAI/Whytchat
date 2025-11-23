# Project Documentation Rules (Non-Obvious Only)

## Terminology
- **"The Brain"**: Refers to the Rust Backend (`apps/core`).
- **"Desktop UI"**: Refers to the React Frontend (`apps/desktop-ui`).
- **"Supervisor"**: The central orchestrator actor in Rust.

## Source of Truth
- **IPC API**: `apps/core/src/main.rs` is the canonical definition of available Tauri commands (`#[tauri::command]`).
- **Database Schema**: `apps/core/src/database.rs` contains the `CREATE TABLE` SQL strings. Use this, not assumptions, to understand the data model.
- **Frontend Routes**: There is no router. The app uses conditional rendering in `App.jsx` based on configuration state (`OnboardingWizard` vs `MainLayout`).

## Localization
- **Files**: `apps/desktop-ui/src/locales/{en,fr}/common.json`.
- **System**: Uses `i18next`. New text strings must be added to both EN and FR files.