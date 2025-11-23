# WhytChat V1.0.0 - "Backend Brain" Architecture

> **Statut :** Work In Progress (V1.0.0)
> **Vision :** Local-First, Secure, High-Performance AI Orchestration.
> **Licence :** MIT

WhytChat est une application de bureau (Tauri v2) qui utilise le modèle d'acteurs pour orchestrer des agents IA en local.

## 📊 État du Projet (Novembre 2025)

### ✅ Ce qui a été accompli :

#### 🏗️ Architecture & Infrastructure

- **Migration vers architecture "Backend-Centric"** : Séparation claire entre logique métier (Rust) et UI (React)
- **Modèle d'acteurs implémenté** : Supervisor, RagActor, LlmActor avec communication via channels Tokio
- **Base de données SQLite intégrée** : Tables `sessions`, `messages`, `session_files` créées
- **Stockage vectoriel** : LanceDB pour le RAG local avec modèles d'embedding

#### 🔧 Fonctionnalités Core

- **Système de sessions** : Gestion des conversations avec persistance
- **RAG (Retrieval-Augmented Generation)** : Recherche sémantique dans les documents locaux
- **Interface utilisateur** : Chat interface avec composants React/Vite
- **Internationalisation** : Support français/anglais avec i18next

#### 📦 Intégration & Déploiement

- **Monorepo configuré** : Structure apps/ avec core (Rust) et desktop-ui (React)
- **Outils automatisés** : CMake et Protoc intégrés localement
- **Qualité du code** : ESLint, Prettier, Husky (git hooks)
- **Repository GitHub** : Code source poussé avec CI/CD basique

### 🚧 Ce qui reste à faire :

#### 🔄 Fonctionnalités en cours

- **API Tauri complète** : Commandes pour gérer les sessions depuis le frontend
- **Upload de fichiers** : Interface pour ajouter des documents aux sessions
- **Intégration RAG complète** : Utilisation des fichiers de session dans les recherches
- **Gestion des paramètres** : Configuration des modèles IA par session

#### 🎯 Améliorations futures

- **Multi-modèles IA** : Support pour différents providers (Ollama, OpenAI, etc.)
- **Recherche web intégrée** : Tavily API pour enrichir les réponses
- **Thèmes et personnalisation** : Interface adaptative
- **Export/Import** : Sauvegarde et restauration des sessions
- **Tests automatisés** : Suite de tests complète
- **Documentation technique** : Guides détaillés pour les développeurs

### 🏃‍♂️ Comment contribuer

1. **Fork** le repository
2. **Clone** votre fork : `git clone https://github.com/YOUR_USERNAME/WhytChat.git`
3. **Installez** les dépendances : `npm install`
4. **Lancez** le développement : `npm run dev`
5. **Créez** une branche pour votre feature : `git checkout -b feature/amazing-feature`
6. **Committez** vos changements : `git commit -m 'Add amazing feature'`
7. **Pushez** vers votre fork : `git push origin feature/amazing-feature`
8. **Ouvrez** une Pull Request

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

## 🛠️ Technologies Utilisées

### Backend (Rust)

- **Tauri v2** : Framework pour applications desktop
- **Tokio** : Runtime async pour le modèle d'acteurs
- **LanceDB** : Base de données vectorielle pour le RAG
- **FastEmbed** : Modèles d'embedding locaux
- **SQLite** : Base de données relationnelle pour les sessions
- **Serde** : Sérialisation/désérialisation JSON

### Frontend (React)

- **React 18** : Framework UI avec hooks
- **Vite** : Build tool et dev server
- **Tailwind CSS** : Framework CSS utilitaire
- **Zustand** : State management léger
- **i18next** : Internationalisation

### Outils & Qualité

- **ESLint + Prettier** : Qualité et formatage du code
- **Husky** : Git hooks pour la qualité
- **CMake + Protocol Buffers** : Outils de build (intégrés localement)

## 🚀 Démarrage Rapide

### Pré-requis Système

#### Pour tous les OS :

- **Node.js** v20+ ([Télécharger](https://nodejs.org/))
- **Rust** v1.75+ ([Installer](https://rustup.rs/))
- **Git** ([Télécharger](https://git-scm.com/))

#### Pour Windows :

- **Visual Studio Build Tools** avec workload C++ ([Télécharger](https://visualstudio.microsoft.com/visual-cpp-build-tools/))
- **WebView2 Runtime** (installé automatiquement par Tauri)

#### Pour Linux :

- **webkit2gtk** (Ubuntu/Debian: `sudo apt install libwebkit2gtk-4.0-dev`)

#### Pour macOS :

- **Xcode Command Line Tools** : `xcode-select --install`

### Installation Automatisée

```bash
# Clone du repository
git clone https://github.com/WhytcardAI/WhytChat.git
cd WhytChat

# Installation des dépendances
npm install

# Installation de Tauri CLI (si pas déjà fait)
cargo install tauri-cli --version "^2.0.0"
```

### Installation

```bash
# À la racine du projet
npm install
```

### Scripts Disponibles

```bash
# Développement complet (Frontend + Backend)
npm run dev

# Build de production
npm run build

# Linting et formatage
npm run lint          # Vérification ESLint
npm run format        # Formatage Prettier
npm run type-check    # Vérification TypeScript (si applicable)

# Tests (à implémenter)
npm test

# Nettoyage
npm run clean
```

### Structure Détaillée du Projet

```
WhytChat/
├── apps/
│   ├── core/                    # 🦀 Backend Rust (Tauri)
│   │   ├── src/
│   │   │   ├── actors/          # Modèle d'acteurs (Supervisor, RAG, LLM)
│   │   │   ├── database.rs      # Gestion SQLite
│   │   │   ├── fs_manager.rs    # Gestionnaire de fichiers
│   │   │   ├── main.rs          # Point d'entrée Tauri
│   │   │   └── models.rs        # Structures de données
│   │   ├── tools/               # Outils intégrés (CMake, Protoc)
│   │   ├── Cargo.toml           # Dépendances Rust
│   │   └── tauri.conf.json      # Configuration Tauri
│   └── desktop-ui/              # ⚛️ Frontend React
│       ├── src/
│       │   ├── components/      # Composants UI
│       │   ├── locales/         # Traductions i18n
│       │   ├── store/           # État Zustand
│       │   └── main.jsx         # Point d'entrée React
│       ├── package.json         # Dépendances Node.js
│       └── tailwind.config.js   # Configuration Tailwind
├── docs/                        # 📚 Documentation
│   ├── specs/                   # Spécifications techniques
│   ├── CHANGELOG.md             # Historique des versions
│   └── *.md                     # Guides et documentation
├── .github/                     # 🤖 Intégration GitHub
├── package.json                 # 📦 Scripts globaux du monorepo
├── Cargo.toml                   # 📦 Workspace Rust
├── LICENSE                      # ⚖️ Licence MIT
└── README.md                    # 📖 Ce fichier
```

### 🔧 Configuration & Environnement

#### Variables d'environnement

Copiez `.env.example` vers `.env` et configurez :

```bash
# Clés API (optionnel pour fonctionnalités avancées)
TAVILY_API_KEY=your_tavily_key_here

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

## 📈 Roadmap

### Phase 1 (Novembre 2025) - ✅ Core Infrastructure

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

## 🤝 Contribution

Nous accueillons les contributions ! Voici comment participer :

### Types de contributions

- 🐛 **Bug reports** : Signalez les problèmes via GitHub Issues
- 💡 **Features** : Proposez de nouvelles fonctionnalités
- 📝 **Documentation** : Améliorez la documentation
- 🧪 **Tests** : Ajoutez des tests unitaires/intégration
- 🎨 **UI/UX** : Améliorez l'interface utilisateur

### Processus

1. Vérifiez les [Issues](https://github.com/WhytcardAI/WhytChat/issues) existantes
2. Créez une Issue pour discuter de votre idée
3. Forkez le repo et créez une branche feature
4. Implémentez vos changements avec tests
5. Soumettez une Pull Request

## 📞 Contact & Support

- **Repository** : [WhytCardAI/WhytChat](https://github.com/WhytcardAI/WhytChat)
- **Issues** : [GitHub Issues](https://github.com/WhytcardAI/WhytChat/issues)
- **Discussions** : [GitHub Discussions](https://github.com/WhytcardAI/WhytChat/discussions)
- **Email** : jerome@whytcard.ai

## 📄 Licence

Ce projet est sous licence MIT - voir le fichier [LICENSE](LICENSE) pour plus de détails.

---

_WhytCard Engineering - 2025_
