# WhytChat V1.0.0 - "Backend Brain" Architecture

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/Rust-1.75+-000000.svg?style=flat&logo=rust)](https://www.rust-lang.org/)
[![React](https://img.shields.io/badge/React-18+-61DAFB.svg?style=flat&logo=react)](https://reactjs.org/)
[![Tauri](https://img.shields.io/badge/Tauri-2.0+-24C8DB.svg?style=flat&logo=tauri)](https://tauri.app/)
[![Node.js](https://img.shields.io/badge/Node.js-20+-339933.svg?style=flat&logo=node.js)](https://nodejs.org/)
[![Linux Build](https://github.com/WhytcardAI/WhytChat/actions/workflows/linux-build.yml/badge.svg)](https://github.com/WhytcardAI/WhytChat/actions/workflows/linux-build.yml)
[![Windows Build](https://github.com/WhytcardAI/WhytChat/actions/workflows/windows-build.yml/badge.svg)](https://github.com/WhytcardAI/WhytChat/actions/workflows/windows-build.yml)
[![macOS Build](https://github.com/WhytcardAI/WhytChat/actions/workflows/macos-build.yml/badge.svg)](https://github.com/WhytcardAI/WhytChat/actions/workflows/macos-build.yml)

> **Statut :** Work In Progress (V1.0.0)
> **Vision :** Local-First, Secure, High-Performance AI Orchestration.
> **Licence :** MIT

WhytChat est une application de bureau moderne qui révolutionne l'interaction avec l'IA en plaçant la puissance de calcul et la confidentialité au cœur de l'expérience utilisateur.

![WhytChat Architecture](./docs/architecture-overview.png)

## ✨ Fonctionnalités Clés

- 🤖 **IA Locale First** : Modèles d'IA exécutés localement pour une confidentialité totale
- 🧠 **Architecture d'Acteurs** : Système de supervision intelligent avec acteurs spécialisés
- 📚 **RAG Avancé** : Recherche sémantique dans vos documents personnels
- 💬 **Interface Moderne** : Chat intuitif avec indicateurs de pensée en temps réel
- 🌍 **Multilingue** : Support français et anglais
- 🔒 **Sécurité Renforcée** : Aucune donnée ne quitte votre machine sans consentement
- 📁 **Gestion de Documents** : Upload et indexation de fichiers pour un contexte enrichi

## 📊 État du Projet (Novembre 2025)

### ✅ Phase 1 - Infrastructure Core (100% Complète)

| Composant                           | Statut | Description                                  |
| ----------------------------------- | ------ | -------------------------------------------- |
| 🏗️ **Architecture Backend-Centric** | ✅     | Séparation claire logique métier/UI          |
| 🎭 **Modèle d'Acteurs**             | ✅     | Supervisor, RagActor, LlmActor opérationnels |
| 💾 **Base de Données SQLite**       | ✅     | Tables sessions, messages, fichiers créées   |
| 🔍 **RAG avec LanceDB**             | ✅     | Recherche vectorielle locale fonctionnelle   |
| 🎨 **Interface Utilisateur**        | ✅     | Chat moderne avec composants React           |
| 🌐 **Internationalisation**         | ✅     | Support français/anglais complet             |

### 🚧 Phase 2 - Fonctionnalités Avancées (75% Complète)

| Fonctionnalité              | Statut | Priorité |
| --------------------------- | ------ | -------- |
| 📁 **Upload de Fichiers**   | 🚧     | Élevée   |
| 🔄 **API Tauri Complète**   | 🚧     | Élevée   |
| 💭 **États de Pensée**      | ✅     | Complète |
| 🎯 **Gestion des Sessions** | 🚧     | Moyenne  |

### 🎯 Phase 3 - Multi-Modèles & Recherche (Planifiée)

- 🤖 **Support Multi-Providers IA** (Ollama, OpenAI, etc.)
- 🌐 **Recherche Web Intégrée** (Tavily API)
- 📤 **Export/Import de Sessions**
- 🎨 **Thèmes Personnalisables**

### 📈 Métriques Clés

- **Lignes de Code** : ~8,500 (Rust: 60%, React: 40%)
- **Couverture Tests** : 0% (à implémenter)
- **Taille Build** : ~45MB (Linux AppImage)
- **Temps de Démarrage** : <3 secondes

### 🏃‍♂️ Comment contribuer

1. **🍴 Fork** le repository
2. **📋 Créez** une issue pour discuter de votre idée
3. **🌿 Créez** une branche : `git checkout -b feature/amazing-feature`
4. **💻 Commitez** vos changements : `git commit -m 'Add amazing feature'`
5. **🚀 Pushez** vers votre fork : `git push origin feature/amazing-feature`
6. **🔄 Ouvrez** une Pull Request

### Types de Contributions

- 🐛 **Bug Fixes** : Corrections de bugs
- ✨ **Features** : Nouvelles fonctionnalités
- 📚 **Documentation** : Amélioration de la docs
- 🧪 **Tests** : Ajout de tests
- 🎨 **UI/UX** : Amélioration de l'interface

### Standards de Code

```bash
# Linting et formatage automatique
npm run lint    # Vérification du code
npm run format  # Formatage automatique

# Pour Rust
cargo clippy    # Linting avancé
cargo fmt       # Formatage
```

### Backend-Centric Design

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   React UI      │    │   Tauri IPC     │    │   Rust Actors   │
│   (Dumb UI)     │◄──►│   Bridge        │◄──►│   (Smart Logic) │
│                 │    │                 │    │                 │
│ • Affichage     │    │ • Commandes     │    │ • Supervisor    │
│ • État UI       │    │ • Événements    │    │ • RAG Actor     │
│ • Interactions  │    │ • Streaming     │    │ • LLM Actor     │
└─────────────────┘    └─────────────────┘    └─────────────────┘
                              │                        │
                              ▼                        ▼
                   ┌─────────────────┐    ┌─────────────────┐
                   │   SQLite DB     │    │   LanceDB       │
                   │  (Sessions)     │    │  (Vectors)      │
                   └─────────────────┘    └─────────────────┘
```

### Technologies Principales

| Catégorie    | Technologies                                                                                                                                                                                                                                                                                                   | Description                           |
| ------------ | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------- |
| **Backend**  | ![Rust](https://img.shields.io/badge/Rust-000000?style=flat&logo=rust&logoColor=white)                                                                                                                                                                                                                         | Logique métier, sécurité, performance |
| **Frontend** | ![React](https://img.shields.io/badge/React-20232A?style=flat&logo=react&logoColor=61DAFB)                                                                                                                                                                                                                     | Interface utilisateur moderne         |
| **Desktop**  | ![Tauri](https://img.shields.io/badge/Tauri-24C8DB?style=flat&logo=tauri&logoColor=white)                                                                                                                                                                                                                      | Framework d'application native        |
| **IA/ML**    | ![LanceDB](https://img.shields.io/badge/LanceDB-000000?style=flat&logo=data:image/svg+xml;base64,PHN2ZyB3aWR0aD0iMjQiIGhlaWdodD0iMjQiIHZpZXdCb3g9IjAgMCAyNCAyNCIgZmlsbD0ibm9uZSIgeG1sbnM9Imh0dHA6Ly93d3cudzMub3JnLzIwMDAvc3ZnIj4KPHJlY3Qgd2lkdGg9IjI0IiBoZWlnaHQ9IjI0IiByeD0iNCIgZmlsbD0iIzAwMCIvPgo8L3N2Zz4K) | Base de données vectorielle           |
| **UI**       | ![Tailwind](https://img.shields.io/badge/Tailwind_CSS-38B2AC?style=flat&logo=tailwind-css&logoColor=white)                                                                                                                                                                                                     | Framework CSS utilitaire              |
| **Build**    | ![Vite](https://img.shields.io/badge/Vite-646CFF?style=flat&logo=vite&logoColor=white)                                                                                                                                                                                                                         | Outil de build ultra-rapide           |

### Outils & Qualité

- **ESLint + Prettier** : Qualité et formatage du code
- **Husky** : Git hooks pour la qualité
- **CMake + Protocol Buffers** : Outils de build (intégrés localement)

## 🚀 Démarrage Rapide

### Installation en 3 minutes

```bash
# 1. Clonez le repository
git clone https://github.com/WhytcardAI/WhytChat.git
cd WhytChat

# 2. Installez les dépendances
npm install

# 3. Lancez l'application
npm run dev
```

**C'est tout !** L'application se lance automatiquement avec le frontend et le backend.

### 📋 Prérequis Système

| OS          | Prérequis                                  |
| ----------- | ------------------------------------------ |
| **Linux**   | `libwebkit2gtk-4.0-dev`, `build-essential` |
| **macOS**   | Xcode Command Line Tools                   |
| **Windows** | Visual Studio Build Tools, WebView2        |

> 💡 **Astuce** : Tous les outils sont automatiquement téléchargés si manquants.

## 📂 Structure du Projet

```
WhytChat/
├── apps/
│   ├── core/                    # 🦀 Backend Rust (Tauri)
│   │   ├── src/                 # Code source principal
│   │   ├── Cargo.toml          # Dépendances Rust
│   │   └── tauri.conf.json     # Configuration Tauri
│   └── desktop-ui/             # ⚛️ Frontend React
│       ├── src/                # Composants et logique UI
│       ├── package.json        # Dépendances Node.js
│       └── tailwind.config.js  # Configuration Tailwind
├── docs/                        # 📚 Documentation
├── .github/                     # 🤖 CI/CD et templates
└── package.json                # 📦 Scripts du monorepo
```

### 🔧 Configuration & Environnement

#### Variables d'environnement

Copiez `.env.example` vers `.env` et configurez :

```bash
# Clés API (optionnel pour fonctionnalités avancées)
TAVILY_API_KEY=your_tavily_key_here

# Sécurité (Obligatoire)
# Token pour sécuriser la communication avec le processus llama-server.
# L'application ne démarrera pas sans cette variable.
LLAMA_AUTH_TOKEN=your_secure_token_here

# Clé de chiffrement (Obligatoire)
# Utilisée pour chiffrer les configurations sensibles (modèles, sessions) en base de données.
# Doit faire 32 caractères.
ENCRYPTION_KEY=01234567890123456789012345678901

# Configuration des modèles
DEFAULT_EMBEDDING_MODEL=all-MiniLM-L6-v2
DEFAULT_LLM_MODEL=llama2:7b
```

#### Base de données

La base SQLite est créée automatiquement dans `data/whytchat.db` au premier lancement.

### 🐛 Dépannage

#### Problèmes courants :

- **Erreur CMake/Protoc** : Les outils sont inclus dans `apps/core/tools/`
- **WebView2 manquant** : Téléchargez depuis [Microsoft](https://developer.microsoft.com/microsoft-edge/webview2/)
- **Port occupé** : Le dev server utilise le port 5173 par défaut
- **Modèles IA** : Téléchargés automatiquement au premier usage (~86MB)

#### Logs et debug :

- **Frontend** : Console du navigateur (F12)
- **Backend** : Terminal où `npm run dev` est lancé
- **Base de données** : Fichiers dans `data/` pour inspection

## 🛡️ Règles d'Or

1. **🚫 No Unwrap** : En Rust, ne jamais utiliser `.unwrap()`. Gérer les erreurs avec `Result` et `anyhow`.
2. **🏠 Local-First** : Aucune donnée ne sort de la machine sans consentement explicite.
3. **🔒 Type Safety** : Pas de "Stringly Typed code". Utiliser des Enums pour les messages inter-acteurs.
4. **🧠 Dumb UI** : Si vous écrivez un `if (step === 'thinking')` complexe dans React, c'est probablement du code Backend mal placé.

## 📈 Roadmap

### Phase 1 (Novembre 2025) - ✅ Infrastructure Core

- [x] Architecture Backend-Centric
- [x] Modèle d'acteurs fonctionnel
- [x] Base de données SQLite
- [x] Interface de chat basique
- [x] RAG avec LanceDB

### Phase 2 (Décembre 2025) - 🚧 Sessions & Fichiers

- [ ] Gestion complète des sessions
- [ ] Upload et indexation de fichiers
- [ ] API Tauri complète
- [ ] Interface d'administration

### Phase 3 (Janvier 2026) - 🎯 Multi-modèles & Recherche

- [ ] Support multi-providers IA
- [ ] Recherche web intégrée (Tavily)
- [ ] Export/Import de sessions
- [ ] Thèmes personnalisables

### Phase 4 (Février 2026) - 🚀 Production

- [ ] Tests automatisés complets
- [ ] Documentation développeur
- [ ] Packaging multi-plateforme
- [ ] Performance et optimisation

## 📞 Support & Communauté

## 📖 Documentation

- **[Guide Utilisateur](./docs/USER_GUIDE.md)** : Premiers pas avec WhytChat
- **[Manuel Technique](./docs/TECHNICAL_MANUAL.md)** : Architecture détaillée
- **[Guide de Contribution](./CONTRIBUTING.md)** : Comment contribuer
- **[Code de Conduite](./CODE_OF_CONDUCT.md)** : Règles communautaires

### 🐛 Signaler un Bug

1. Vérifiez les [Issues existantes](https://github.com/WhytcardAI/WhytChat/issues)
2. Utilisez le template de bug report
3. Fournissez : OS, version, logs, étapes de reproduction

### 💡 Demander une Feature

1. Vérifiez les [Discussions](https://github.com/WhytcardAI/WhytChat/discussions)
2. Créez une Feature Request avec votre cas d'usage

### 🤝 Contact

- **📧 Email** : jerome@whytcard.ai
- **🐙 GitHub** : [WhytcardAI](https://github.com/WhytcardAI)
- **💼 LinkedIn** : [WhytCard Engineering](https://linkedin.com/company/whytcard)

## 📄 Licence

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

Ce projet est sous licence MIT - voir le fichier [LICENSE](LICENSE) pour plus de détails.

---

<div align="center">

**WhytChat** - _Local-First AI Orchestration_

[![Made with ❤️ by WhytCard Engineering](https://img.shields.io/badge/Made%20with%20❤️%20by-WhytCard%20Engineering-FF6B6B.svg)](https://whytcard.ai)

_Novembre 2025_

</div>
