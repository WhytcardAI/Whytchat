# 📚 Documentation WhytChat V1

> Documentation générée le 28 novembre 2025 basée sur analyse directe du code source
> Mise à jour: Janvier 2025 - Rapport d'analyse ajouté

---

## 📁 Structure de la Documentation

| Fichier                                            | Description                                 |
| -------------------------------------------------- | ------------------------------------------- |
| [01_VUE_ENSEMBLE.md](01_VUE_ENSEMBLE.md)           | Vue d'ensemble du projet et stack technique |
| [02_ARCHITECTURE.md](02_ARCHITECTURE.md)           | Architecture système d'acteurs Tokio        |
| [03_BACKEND.md](03_BACKEND.md)                     | Détail de tous les modules Rust             |
| [04_BRAIN_MODULE.md](04_BRAIN_MODULE.md)           | Module d'analyse pré-LLM (intent, keywords) |
| [05_FRONTEND.md](05_FRONTEND.md)                   | Composants React, hooks et store Zustand    |
| [06_COMMANDES_TAURI.md](06_COMMANDES_TAURI.md)     | Liste exhaustive des 22 commandes IPC       |
| [07_FLUX_DONNEES.md](07_FLUX_DONNEES.md)           | Flux complet d'un message utilisateur       |
| [08_CONFIGURATION.md](08_CONFIGURATION.md)         | Fichiers de configuration et schéma DB      |
| [09_TESTS.md](09_TESTS.md)                         | Tests Rust et E2E Playwright                |
| [10_SECURITE.md](10_SECURITE.md)                   | Encryption AES-256-GCM et rate limiting     |
| [11_RAG_SYSTEM.md](11_RAG_SYSTEM.md)               | Système RAG (LanceDB, FastEmbed, chunking)  |
| [12_DEPLOIEMENT.md](12_DEPLOIEMENT.md)             | Build, distribution et installation         |
| [ANALYSIS_REPORT_2024.md](ANALYSIS_REPORT_2024.md) | Rapport d'analyse du codebase (Jan 2025)    |

---

## 🚀 Démarrage Rapide

### Prérequis

- **Rust** 1.80.0+ (`rust-toolchain.toml`)
- **Node.js** 18+
- **Tauri CLI** 2.0.0+

### Commandes

```bash
# Développement
npm run dev

# Build production
npm run build

# Tests Rust (44 tests)
cargo test --manifest-path apps/core/Cargo.toml

# Lint
npm run lint
```

---

## 📊 Métriques du Projet

| Métrique                     | Valeur |
| ---------------------------- | ------ |
| Fichiers Rust (src/)         | 22     |
| Fichiers JSX/JS (frontend)   | ~30    |
| Commandes Tauri              | 22     |
| Tests unitaires Rust         | 44     |
| Dépendances Rust             | 35     |
| Dépendances npm (desktop-ui) | 17     |

---

## 🔧 Stack Technique

### Backend (Rust)

- **Tauri** 2.0.0-rc - Framework desktop
- **SQLite** (sqlx 0.8) - Base de données sessions/messages
- **LanceDB** 0.10 - Base vectorielle RAG
- **FastEmbed** 4 - Embeddings (AllMiniLML6V2)
- **AES-GCM** 0.10.3 - Encryption des configurations

### Frontend (React)

- **React** 18.3.1
- **Vite** 5.4.1
- **Zustand** 5.0.0 - State management
- **Tailwind CSS** 3.4.10
- **i18next** - Internationalisation (fr/en)

---

_Documentation basée sur lecture directe du code source_
