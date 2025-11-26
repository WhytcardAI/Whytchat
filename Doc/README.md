# 📘 WhytChat - Documentation Technique

Bienvenue dans la documentation centrale de WhytChat. Ce dossier constitue la **Source de Vérité Unique** pour comprendre, maintenir et faire évoluer le projet.

## 🧭 Index de la Documentation

| Fichier | Description | Public Cible |
| :--- | :--- | :--- |
| **[🏠 README.md](./README.md)** | Point d'entrée et guide de démarrage rapide. | Tous |
| **[🏗️ ARCHITECTURE.md](./ARCHITECTURE.md)** | Vue d'ensemble du système, diagrammes et stack technique. | Architectes, Nouveaux Devs |
| **[⚙️ PROCESSUS.md](./PROCESSUS.md)** | Détail des flux métier (RAG, Chat, Démarrage) avec diagrammes de séquence. | Développeurs Backend/Frontend |
| **[🧠 IA_INTERNALS.md](./IA_INTERNALS.md)** | Fonctionnement interne du "Brain", GGUF, Embeddings et Vector Store. | Data Scientists, Devs IA |
| **[🛑 AUDIT_CRITIQUE.md](./AUDIT_CRITIQUE.md)** | Rapport de santé, risques identifiés et dette technique connue. | Lead Dev, Maintainers |
| **[📏 STANDARDS.md](./STANDARDS.md)** | Conventions de code, Best Practices et Règles "Gold Standard". | Tous |
| **[🛠️ METHODOLOGIE_DEV.md](./METHODOLOGIE_DEV.md)** | Guide du cycle de développement (SDLC), Git, Tests et Release. | Tous les développeurs |

---

## 🚀 Quick Start (Guide de Démarrage)

### Pré-requis
*   **Node.js** (v18+)
*   **Rust** (Dernière version stable)
*   **VS Code** (Recommandé avec extensions Rust-Analyzer & Tailwind)
*   **OS** : Windows 10/11 (Cible principale actuelle), macOS/Linux supportés.

### Installation & Lancement

1.  **Cloner le dépôt**
    ```bash
    git clone <url-du-repo>
    cd WhytChat_V1
    ```

2.  **Installer les dépendances Frontend**
    ```bash
    npm install
    ```

3.  **Lancer en mode Développement**
    Cette commande lance simultanément le Backend (Rust) et le Frontend (Vite) avec Hot-Reload.
    ```bash
    npm run tauri dev
    ```
    *Le premier lancement peut être long (compilation Rust).*

4.  **Builder pour la Production**
    ```bash
    npm run tauri build
    ```
    *L'exécutable sera généré dans `apps/core/target/release/`.*

### Commandes Utiles

*   `npm run lint` : Vérifie la qualité du code (JS & Rust).
*   `npm run format` : Formate le code automatiquement.

---

## 📂 Structure du Projet (Monorepo)

*   `apps/core/` : **Backend Rust**. Contient la logique métier, les acteurs (Tokio), la gestion de base de données (SQLite) et l'IA.
*   `apps/desktop-ui/` : **Frontend React**. Interface utilisateur, gestion d'état (Zustand) et composants graphiques.
*   `Doc/` : **Documentation** (Vous êtes ici).
*   `tools/` : Binaires externes nécessaires (llama-server, etc.).

---

> *Cette documentation est maintenue à jour avec l'évolution du code. Dernière mise à jour : Novembre 2025.*