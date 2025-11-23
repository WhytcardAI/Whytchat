# WhytChat V1.0.0 - "Backend Brain" Architecture

> **Statut :** Work In Progress (V1.0.0)
> **Vision :** Local-First, Secure, High-Performance AI Orchestration.

WhytChat est une application de bureau (Tauri v2) qui utilise le modèle d'acteurs pour orchestrer des agents IA en local.

## 🏗️ Architecture

Nous avons migré d'une architecture "Frontend-Heavy" vers un modèle **"Backend-Centric"**.

### 1. Backend (Rust / Tauri Core) - "The Brain"

- **Rôle :** Orchestration, Mémoire (RAG), Sécurité, Logique Métier.
- **Pattern :** Actor Model (Tokio + Channels). Chaque agent (Perception, Reasoning) est un acteur isolé.
- **Stockage :** LanceDB (Vector DB embarquée) pour le RAG local.

### 2. Frontend (React / Vite) - "The Dumb UI"

- **Rôle :** Affichage de l'état, Capture de l'intention utilisateur.
- **Pattern :** "Dumb Components". Aucune logique de prompt ou de machine à états dans le JS.
- **State :** Zustand (uniquement pour l'état UI : thème, onglet actif).

## 🚀 Démarrage Rapide

### Pré-requis

- **Node.js** (v20+)
- **Rust** (v1.75+)
- **Tauri CLI** (`cargo install tauri-cli --version "^2.0.0"`)
- **Windows Build Tools** (Pour Windows) :
  - Visual Studio Build Tools (C++ Workload)
  - _Note : Protoc et CMake sont gérés localement dans `apps/core/tools` pour ce projet._

### Installation

```bash
# À la racine du projet
npm install
```

### Développement

Lancer le Frontend et le Backend en mode dev (Hot Reload) :

> **Note Windows :** Assurez-vous que les variables d'environnement pour `protoc` et `cmake` sont configurées si vous ne passez pas par les scripts automatisés.

```bash
npm run dev
```

### Qualité & Linting

Nous imposons des standards stricts via Husky (Git Hooks).

```bash
# Linter tout le projet (JS + Rust)
npm run lint

# Formatter tout le code
npm run format
```

## 📂 Structure du Monorepo

```
WhytChat_V1/
├── apps/
│   ├── core/          # 🦀 Backend Rust (Tauri)
│   │   ├── src/main.rs
│   │   └── rust-toolchain.toml
│   └── desktop-ui/    # ⚛️ Frontend React
│       ├── src/
│       ├── eslint.config.js
│       └── vite.config.js
├── package.json       # 📦 Scripts globaux
└── README.md          # 📘 Vous êtes ici
```

## 🛡️ Règles d'Or

1.  **No Unwrap :** En Rust, ne jamais utiliser `.unwrap()`. Gérer les erreurs avec `Result` et `anyhow`.
2.  **Local-First :** Aucune donnée ne sort de la machine sans consentement explicite (ex: Recherche Web Tavily).
3.  **Type Safety :** Pas de "Stringly Typed code". Utiliser des Enums pour les messages inter-acteurs.
4.  **Dumb UI :** Si vous écrivez un `if (step === 'thinking')` complexe dans React, c'est probablement du code Backend mal placé.

---

_WhytCard Engineering - 2025_
