# 🚀 Déploiement - WhytChat V1

> Guide de build, distribution et installation

---

## 🏗️ Prérequis

### Développement

| Outil     | Version | Installation                     |
| --------- | ------- | -------------------------------- |
| Rust      | 1.80.0+ | `rustup install stable`          |
| Node.js   | 18+     | [nodejs.org](https://nodejs.org) |
| npm       | 9+      | Inclus avec Node.js              |
| Tauri CLI | 2.0.0+  | `npm install -g @tauri-apps/cli` |

### Build Windows

| Outil         | Version | Notes                |
| ------------- | ------- | -------------------- |
| Visual Studio | 2019+   | Build Tools C++      |
| Windows SDK   | 10.0+   | Inclus avec VS       |
| WebView2      | Runtime | Windows 10/11 inclus |

---

## 📦 Scripts de Build

### package.json (Racine)

```json
{
  "scripts": {
    "dev": "tauri dev",
    "build": "tauri build",
    "build:debug": "tauri build --debug",
    "lint": "npm run lint:rust && npm run lint:js",
    "lint:rust": "cargo clippy --manifest-path apps/core/Cargo.toml -- -D warnings",
    "lint:js": "npm run --prefix apps/desktop-ui lint",
    "test": "cargo test --manifest-path apps/core/Cargo.toml",
    "test:e2e": "npm run --prefix apps/desktop-ui test"
  }
}
```

---

## 🔧 Développement

### Démarrer en mode Dev

```bash
# Depuis la racine du projet
npm run dev

# Ou manuellement
cd apps/desktop-ui && npm run dev  # Terminal 1
cd apps/core && cargo tauri dev    # Terminal 2
```

Le mode développement :

- Démarre Vite sur `http://localhost:1420`
- Lance le backend Rust avec hot-reload
- Ouvre la fenêtre WhytChat

### Vérifications avant commit

```bash
# Lint complet
npm run lint

# Tests Rust
npm run test

# Tests E2E (optionnel)
npm run test:e2e
```

---

## 🏭 Build Production

### Build Standard

```bash
# Build release (optimisé)
npm run build
```

### Outputs

```
target/release/
├── whytchat-core.exe           # Exécutable Windows
└── bundle/
    ├── nsis/
    │   └── WhytChat_1.0.0_x64-setup.exe    # Installateur NSIS
    └── msi/
        └── WhytChat_1.0.0_x64_en-US.msi    # Installateur MSI
```

### Configuration de Build

```json
// apps/core/tauri.conf.json
{
  "bundle": {
    "active": true,
    "targets": ["nsis", "msi"],
    "icon": ["icons/32x32.png", "icons/128x128.png", "icons/icon.ico"],
    "windows": {
      "certificateThumbprint": null,
      "digestAlgorithm": "sha256"
    }
  }
}
```

---

## 📁 Structure de Distribution

### Arborescence Installée

```
WhytChat/
├── WhytChat.exe                 # Exécutable principal
├── data/
│   ├── db/
│   │   └── whytchat.sqlite      # Base de données (créée au runtime)
│   ├── models/
│   │   ├── default-model.gguf   # Modèle LLM (téléchargé)
│   │   └── embeddings/          # Cache FastEmbed
│   ├── vectors/
│   │   └── knowledge_base.lance/# Base vectorielle
│   └── files/                   # Fichiers uploadés
│   └── .encryption_key          # Clé de chiffrement (générée)
└── tools/
    └── llama/
        └── llama-server.exe     # Serveur LLM (téléchargé)
```

### Fichiers Créés au Runtime

| Fichier                   | Création              | Contenu              |
| ------------------------- | --------------------- | -------------------- |
| `data/db/whytchat.sqlite` | Premier démarrage     | Sessions, messages   |
| `data/.encryption_key`    | Premier démarrage     | Clé AES-256 (Base64) |
| `data/models/*.gguf`      | Onboarding (download) | Modèle LLM           |
| `data/models/embeddings/` | Premier embedding     | Cache FastEmbed      |
| `data/vectors/`           | Première ingestion    | Index LanceDB        |

---

## 📥 Installation

### Installateur NSIS (Recommandé)

```bash
# Double-cliquer sur l'installateur
WhytChat_1.0.0_x64-setup.exe

# Installation silencieuse
WhytChat_1.0.0_x64-setup.exe /S
```

Options NSIS :

- Chemin d'installation personnalisable
- Raccourcis Bureau/Menu Démarrer
- Désinstallation propre

### Installateur MSI

```bash
# Installation standard
msiexec /i WhytChat_1.0.0_x64_en-US.msi

# Installation silencieuse
msiexec /i WhytChat_1.0.0_x64_en-US.msi /qn
```

### Mode Portable

Pour un déploiement sans installation :

```bash
# Copier le dossier release complet
xcopy /E target\release\WhytChat portable_folder\

# S'assurer que le dossier data/ existe
mkdir portable_folder\data
```

---

## ⚙️ Configuration Runtime

### Premier Démarrage

1. **Preflight Check** - Vérifie les dépendances
2. **Onboarding** - Guide téléchargement modèle si absent
3. **Initialisation** - Crée la base de données et la clé

### Variables d'Environnement (Optionnelles)

| Variable         | Description                   | Défaut      |
| ---------------- | ----------------------------- | ----------- |
| `ENCRYPTION_KEY` | Clé AES (32 chars) pour tests | Auto-généré |
| `RUST_LOG`       | Niveau de log                 | `info`      |

### Fichier de Log

```
%APPDATA%\com.whytchat.app\logs\
└── whytchat.log
```

---

## 🔄 Mise à Jour

### Processus de Mise à Jour

1. **Télécharger** le nouvel installateur
2. **Fermer** WhytChat
3. **Exécuter** l'installateur (remplace l'ancien)
4. **Démarrer** WhytChat

### Données Préservées

| Donnée                  | Préservée | Notes                       |
| ----------------------- | --------- | --------------------------- |
| Sessions & Messages     | ✅        | Dans `data/db/`             |
| Fichiers uploadés       | ✅        | Dans `data/files/`          |
| Clé de chiffrement      | ✅        | Dans `data/.encryption_key` |
| Index vectoriel         | ✅        | Dans `data/vectors/`        |
| Modèle LLM              | ✅        | Dans `data/models/`         |
| Préférences utilisateur | ✅        | localStorage du WebView     |

---

## 🐛 Debugging Production

### Logs en Production

```bash
# Activer les logs détaillés
set RUST_LOG=debug
WhytChat.exe

# Ou modifier le registre Windows pour persistance
```

### Vérifier l'Installation

```bash
# Vérifier les fichiers essentiels
dir "C:\Program Files\WhytChat\data"

# Vérifier la base de données
sqlite3 "C:\Program Files\WhytChat\data\db\whytchat.sqlite" ".tables"
```

### Erreurs Communes

| Erreur                         | Cause                 | Solution                  |
| ------------------------------ | --------------------- | ------------------------- |
| "Model not found"              | Modèle non téléchargé | Relancer l'onboarding     |
| "Database error"               | DB corrompue          | Supprimer whytchat.sqlite |
| "Encryption key invalid"       | Clé corrompue         | Supprimer .encryption_key |
| "llama-server failed to start" | Port 8765 occupé      | Fermer l'autre processus  |

---

## 📊 Taille des Fichiers

### Build Release

| Composant           | Taille |
| ------------------- | ------ |
| `whytchat-core.exe` | ~15 MB |
| Installateur NSIS   | ~12 MB |
| Installateur MSI    | ~14 MB |

### Données Runtime

| Composant                 | Taille  | Notes      |
| ------------------------- | ------- | ---------- |
| Modèle LLM (Qwen 7B Q4)   | ~4.7 GB | Téléchargé |
| llama-server + DLLs       | ~200 MB | Téléchargé |
| Cache FastEmbed           | ~100 MB | Généré     |
| Base SQLite (vide)        | ~50 KB  | Générée    |
| Index LanceDB (1000 docs) | ~50 MB  | Variable   |

---

## 🔒 Sécurité de Distribution

### Signature de Code (Optionnel)

```json
// tauri.conf.json
{
  "bundle": {
    "windows": {
      "certificateThumbprint": "VOTRE_THUMBPRINT",
      "digestAlgorithm": "sha256",
      "timestampUrl": "http://timestamp.digicert.com"
    }
  }
}
```

### Vérifications de Sécurité

- [ ] Code signé avec certificat valide
- [ ] SHA256 des installateurs publié
- [ ] Pas de secrets dans le binaire
- [ ] CSP Tauri configuré correctement

---

## 📋 Checklist de Release

### Avant le Build

- [ ] Tous les tests passent (`npm run test`)
- [ ] Lint propre (`npm run lint`)
- [ ] Version mise à jour dans `Cargo.toml` et `package.json`
- [ ] Changelog à jour

### Build

- [ ] Build release (`npm run build`)
- [ ] Installateur NSIS généré
- [ ] Installateur MSI généré
- [ ] Test d'installation sur machine vierge

### Après Release

- [ ] Tag Git créé
- [ ] Release GitHub avec binaires
- [ ] SHA256 des fichiers publié
- [ ] Documentation mise à jour

---

## 🌐 Distribution Multi-Plateforme

### Cibles Supportées (Tauri 2.0)

| Plateforme    | Status | Notes                      |
| ------------- | ------ | -------------------------- |
| Windows x64   | ✅     | Testé                      |
| Windows ARM64 | ⚠️     | Non testé                  |
| macOS x64     | ⚠️     | Nécessite adaptation llama |
| macOS ARM64   | ⚠️     | Nécessite adaptation llama |
| Linux x64     | ⚠️     | Nécessite adaptation llama |

### Build Cross-Platform

```bash
# Windows (depuis Windows)
npm run build

# macOS (depuis macOS)
npm run build

# Linux (depuis Linux)
npm run build
```

---

_Généré depuis lecture directe de: tauri.conf.json, Cargo.toml, package.json, scripts de build_
