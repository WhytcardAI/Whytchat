# 🛑 Audit Critique : Rapport de Santé & Dette Technique

> **Statut :** � EN COURS D'AMÉLIORATION
> **Date de l'audit initial :** 25 Novembre 2025
> **Dernière mise à jour :** 26 Novembre 2025

Ce document recense les défauts techniques majeurs, les risques de crash et la dette technique identifiée. Il sert de feuille de route pour la refactorisation et la stabilisation ("Hardening") de l'application.

## ✅ Problèmes Résolus (Novembre 2025)

### Architecture Fichiers

- ✅ **Entrée unique pour les fichiers externes** : Seule `KnowledgeView` permet l'upload de fichiers.
- ✅ **SessionWizard** : Sélectionne désormais des fichiers existants de la bibliothèque (pas d'upload).
- ✅ **ChatInput** : Text-only, plus d'upload de fichiers.
- ✅ **Support PDF/DOCX** : Module `text_extract.rs` avec crates `pdf-extract` et `docx-rs`.
- ✅ **Multi-fichiers** : Upload multiple supporté dans KnowledgeView.

### Système de Logging

- ✅ **Logger centralisé** : `apps/desktop-ui/src/lib/logger.js` avec niveaux configurables.
- ✅ **Logs structurés** : Toutes les actions UI/Store sont tracées via le logger.

## 🚨 Risques Critiques (Backend - Rust)

### 💣 Roulette Russe (Panics & Unwraps)

L'application contient **37 bombes à retardement** (`unwrap()` / `expect()`).

- **Mutex Poisoning (Danger Immédiat)** :
  - `apps/core/src/main.rs` : L'utilisation systémique de `state.app_handle.lock().unwrap()` est une faute grave. Si un seul thread panic pendant qu'il détient ce verrou, **toute l'application crashera** définitivement (Mutex Poisoning).
  - `apps/core/src/actors/supervisor.rs` : Idem pour les accès aux états partagés.

- **Fragilité Runtime** :
  - `apps/core/src/brain/intent.rs` : Des dizaines de `Regex::new(...).unwrap()` sont exécutés à la volée. Si une regex est mal formée, le thread panic.
  - `apps/core/src/encryption.rs` : `encrypt(...).expect("Encryption failed")`. En cas d'erreur crypto (ex: clé invalide), crash immédiat au lieu de gérer l'erreur proprement.
  - `apps/core/src/actors/llm.rs` : `temperature.unwrap_or(...)` et parsing JSON avec `unwrap()`. Risque de crash sur données mal formées venant du LLM.

### 🔇 Erreurs Silencieuses (The Silent Killers)

Le code avale des erreurs critiques, rendant le débogage impossible en production.

- **Système de Fichiers** :
  - `apps/core/src/actors/rag.rs` (Ligne 155) : `let _ = std::fs::create_dir_all(parent);`. Si la création du dossier de la base de données échoue (permissions, disque plein), le RAG plantera plus loin sans qu'on sache pourquoi.
  - `apps/core/src/fs_manager.rs` (Ligne 57, 66) : `let _ = std::fs::create_dir_all(&path);`. Idem pour les dossiers de configuration.

- **Communication Acteurs** :
  - `apps/core/src/actors/supervisor.rs` : `let _ = responder.send(result);`. Si le destinataire d'un message est mort, le superviseur continue comme si de rien n'était.
  - `apps/core/src/main.rs` : `window.emit(...).ok()`. Les événements envoyés au frontend sont envoyés dans le vide si ça échoue, sans log.

## ⚠️ Dette Technique Restante (Frontend - React)

### 🧱 Valeurs en Dur (Magic Strings)

Le code contient encore des chaînes de caractères qui contrôlent la logique.

- **États de l'Application** (`App.jsx`) : `'checking'`, `'passed'`, `'failed'`, `'onboarding'`. Devraient être des constantes ou une énumération.
- **États de Téléchargement** (`OnboardingWizard.jsx`) : `'waiting'`, `'downloading'`, `'complete'`, `'error'`.

## 🧹 Hygiène & Maintenance

### 👻 Code Mort & Dette Invisible

- **Zéro TODOs** : Il n'y a **AUCUN** commentaire `TODO`, `FIXME` ou `XXX` dans le code source du projet. C'est statistiquement impossible pour un projet en cours.

## 📉 Optimisations Manquées

- **Performance Regex** : Dans `apps/core/src/brain/intent.rs`, les objets `Regex` sont recréés et recompilés à chaque appel de fonction. C'est un gaspillage de CPU inutile.

## 🔍 Points de Fragilité des Flux Métier

1.  **Workflow RAG** :
    - ✅ **Chunking amélioré** : Overlap de 50 chars entre chunks (512 chars/chunk).
    - ⚠️ **Atomisation** : Pas de transactionnalité entre disque, SQL et LanceDB.
    - ⚠️ **Mémoire** : Chargement complet des fichiers en RAM (limite 10MB).

2.  **Workflow Chat** :
    - **Persistance Tardive** : Sauvegarde du message assistant uniquement à la fin du stream. Perte de données si crash.
    - **Parsing SSE** : Parsing manuel fragile dans `llm.rs`.

3.  **Preflight** :
    - **Port Hardcodé** : Test sur port 18080. Si occupé, faux positif.
    - **Timeout** : Peut geler l'UI pendant 30s.

4.  **Onboarding** :
    - **Intégrité** : Pas de vérification SHA256 du modèle téléchargé.
    - **Dépendance** : Pas de mode hors-ligne pour l'installation.

5.  **Settings** :
    - **Clé de Chiffrement** : Dépend d'une variable d'environnement (`ENCRYPTION_KEY`). Si perdue, toutes les données sont perdues.
