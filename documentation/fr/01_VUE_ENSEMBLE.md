# 📚 Vue d'Ensemble - WhytChat V1

> **Date d'analyse** : 27 novembre 2025  
> **Version** : 1.0.0  
> **Fichiers analysés** : 30+

---

## 🎯 Qu'est-ce que WhytChat ?

WhytChat est une application de chat IA **local-first** conçue pour garantir confidentialité et performance.

### Philosophie

| Principe | Description |
|----------|-------------|
| **100% Local** | Aucune donnée envoyée à des serveurs externes |
| **Privé** | Chiffrement AES-256-GCM des données sensibles |
| **Intelligent** | Analyse pré-LLM (Brain) pour optimiser les réponses |
| **Portable** | Exécutable depuis n'importe quel dossier |

---

## 🛠️ Stack Technologique

| Couche | Technologies |
|--------|--------------|
| Desktop Framework | Tauri 2.0 RC |
| Backend | Rust 1.80.0+, Tokio (async) |
| Database | SQLite (sqlx), LanceDB (vectors) |
| LLM | llama-server, GGUF models (Qwen2.5-Coder-7B) |
| Embeddings | FastEmbed (AllMiniLML6V2, 384-dim) |
| Encryption | AES-256-GCM |
| Frontend | React 18, Vite, Zustand |
| Styling | Tailwind CSS |
| i18n | i18next (FR/EN) |

---

## 📊 Métriques du Codebase

| Catégorie | Fichiers | Lignes estimées |
|-----------|----------|-----------------|
| Backend Rust | 22 | ~6,000 |
| Frontend React | 8+ | ~1,500 |
| Tests | 4 | ~400 |
| Config | 5 | ~200 |
| **Total** | **~40** | **~8,100** |

---

## 📋 État du Projet

### Irrégularités Identifiées

| Sévérité | Nombre | Description |
|----------|--------|-------------|
| 🔴 HIGH | 4 | Tests ne compilent pas |
| ⚠️ MEDIUM | 7 | Features incomplètes |
| ℹ️ LOW | 7 | Code style, optimisations |
| **Total** | **18** | |

### Commandes Tauri IPC

Le backend expose **22 commandes** réparties ainsi :

- **Sessions** : 7 commandes (CRUD, favoris, déplacement)
- **Messages** : 2 commandes (chat, historique)
- **Fichiers** : 6 commandes (upload, liste, suppression, indexation)
- **Dossiers** : 4 commandes (CRUD, organisation)
- **Système** : 4 commandes (init, diagnostics, preflight)

---

## 🗺️ Navigation dans la Documentation

1. **[01_VUE_ENSEMBLE.md](01_VUE_ENSEMBLE.md)** - Ce fichier (introduction)
2. **[02_ARCHITECTURE.md](02_ARCHITECTURE.md)** - Structure et architecture technique
3. **[03_BACKEND_RUST.md](03_BACKEND_RUST.md)** - Modules Rust détaillés
4. **[04_FRONTEND_REACT.md](04_FRONTEND_REACT.md)** - Composants React détaillés
5. **[05_FLUX_DONNEES.md](05_FLUX_DONNEES.md)** - Flux de données complets
6. **[06_SECURITE.md](06_SECURITE.md)** - Analyse de sécurité
7. **[07_IRREGULARITES.md](07_IRREGULARITES.md)** - Problèmes identifiés
8. **[08_RECOMMANDATIONS.md](08_RECOMMANDATIONS.md)** - Actions suggérées
9. **[09_METRIQUES.md](09_METRIQUES.md)** - Statistiques détaillées

---

## ⚡ Démarrage Rapide

```bash
# Installation des dépendances
npm install

# Mode développement
npm run dev          # Démarre Tauri + Vite

# Build production
npm run build

# Tests
npm run lint         # ESLint + Clippy
npm run test:e2e     # Tests Playwright
```

---

## 📁 Structure des Dossiers

```
WhytChat_V1/
├── apps/
│   ├── core/           # Backend Rust (Tauri)
│   └── desktop-ui/     # Frontend React
├── data/               # Données locales (DB, models, vectors)
├── Doc/                # Documentation legacy
└── documentation/      # Nouvelle documentation structurée
    └── fr/             # Documentation française
```

---

_Document généré le 27 novembre 2025_
