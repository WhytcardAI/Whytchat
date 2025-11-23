# Roadmap WhytChat V1 - Feature by Feature

Ce document trace la route pour la finalisation de WhytChat V1. Nous procédons fonctionnalité par fonctionnalité pour garantir la stabilité et la qualité.

## ✅ Feature 1 : Le Squelette (Terminé)
*   **Objectif :** Structure du projet, Standards, et Portabilité.
*   **Livrables :**
    *   [x] Monorepo (Rust + React).
    *   [x] Système de fichiers portable (`PortablePathManager`).
    *   [x] Internationalisation (i18n) prête.
    *   [x] Architecture Frontend scalable (`components`, `store`, `hooks`).

## ✅ Feature 2 : Le Système Nerveux (Terminé)
*   **Objectif :** Connecter l'Interface Utilisateur au Backend Rust.
*   **Décisions UX :**
    *   **Thinking :** Affichage détaillé des étapes de réflexion (collapsible/accordéon).
    *   **Tavily :** Toggle Switch (ON/OFF) dans la barre de saisie.
*   **Tâches :**
    *   [x] Créer l'interface de Chat (Input, Liste de messages).
    *   [x] Implémenter le composant "Thinking Process" (Ouvrir/Fermer).
    *   [x] Câbler la commande Tauri `send_message`.
    *   [x] Afficher la réponse du Backend dans l'UI.

## 🚧 Feature 3 : Le Cerveau (LLM Local - Qwen) (En Cours)
*   **Objectif :** Remplacer le Mock par une vraie inférence locale.
*   **Décisions UX :**
    *   **Modèle Unique :** Qwen (Optimisé).
    *   **Onboarding :** Présentation -> Langue -> Téléchargement Auto (Serveur + Modèle).
*   **Tâches :**
    *   [x] Créer l'écran d'Onboarding (Wizard).
    *   [x] Intégrer le binaire `llama-server`.
    *   [ ] Script de téléchargement Rust (avec barre de progression).
    *   [ ] Implémenter le streaming de réponse (Token par token).

## 📅 Feature 4 : La Mémoire (RAG & LanceDB)
*   **Objectif :** Donner une mémoire à long terme à l'IA.
*   **Décisions UX :**
    *   **Bibliothèque Centrale :** Gestion des documents dans un onglet dédié.
*   **Tâches :**
    *   [ ] Créer la vue "Bibliothèque" (Upload, Liste).
    *   [ ] Intégrer `lancedb` et `fastembed` dans le Backend.
    *   [ ] Pipeline d'ingestion.

## 📅 Feature 5 : Les Sessions (SQLite)
*   **Objectif :** Sauvegarder l'historique et les préférences.
*   **Tâches :**
    *   [ ] Intégrer SQLite.
    *   [ ] Sauvegarder les messages dans la DB.
    *   [ ] Gérer la liste des conversations (Sidebar).

## 📅 Feature 6 : Les Sens (Tavily Web Search)
*   **Objectif :** Connecter l'IA au web (sur demande explicite).
*   **Tâches :**
    *   [ ] Ajouter le "Toggle" Tavily dans l'UI.
    *   [ ] Implémenter l'appel API sécurisé côté Backend.
    *   [ ] Injecter les résultats de recherche dans le contexte.
