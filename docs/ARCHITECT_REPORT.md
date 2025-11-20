# Rapport d'Architecture & Audit Technique - WhytChat / Wyhtcard

**Date** : 19 Novembre 2025
**Auteur** : Lead Software Architect
**Version** : 1.0

---

## 1. Résumé Exécutif

L'audit révèle un écosystème divisé en deux entités techniquement distinctes mais alignées sur une vision commune :
1.  **WhytChat (Desktop App)** : Une application locale Tauri/React intégrant un LLM. L'état actuel correspond à une **Version 1 (V1)** fonctionnelle mais simplifiée par rapport à la vision "Mixture of Agents". Le système simule une interaction multi-agents de manière séquentielle sur un modèle unique.
2.  **Wyhtcard (Web)** : Un site vitrine React/Vite moderne, techniquement mature, respectant les standards de production (SEO, i18n, Performance).

**Verdict Global** : Le projet possède des fondations solides. Le site web est prêt au déploiement. L'application Desktop nécessite une refonte architecturale du Backend Rust pour supporter la véritable promesse "Multi-Vendeurs" (V2).

---

## 2. Analyse Détaillée : WhytChat (Desktop App)

### 2.1 Backend (Rust/Tauri)
Le cœur du système repose sur l'orchestration du binaire `llama-server`.

*   **État Actuel (Architecture V1)** :
    *   **Mono-Processus** : Le code ne gère qu'une seule instance de `llama-server` (port 8080).
    *   **Simulation MoA** : La fonction `chat_multi` envoie des requêtes séquentielles avec des *System Prompts* différents au même modèle. Cela contredit la vision d'utiliser des modèles "Experts" spécialisés (Phi-3 pour la logique, Gemma pour la créativité).
    *   **Modèle Fixe** : Le chemin du modèle est codé en dur (`qwen2.5-3b-instruct-q4_k_m.gguf` dans `llama.rs`). Aucune flexibilité pour l'utilisateur.

*   **Points Critiques & Risques** :
    *   **Robustesse** : Utilisation fréquente de `.unwrap()` dans des sections critiques (notamment sur les `Mutex`), risquant de faire crasher l'application en cas d'erreur inattendue.
    *   **Sécurité** : Le serveur Llama est lancé sans restriction d'interface explicite (risque d'exposition sur le réseau local si non maîtrisé) et la configuration CSP Tauri autorise `http://ipc.localhost`.

### 2.2 Frontend (React)
*   **Architecture** : Simple et efficace, basée sur `Zustand` pour l'état global.
*   **Dette Technique** : Faible. Le code est propre mais mélange parfois logique métier et affichage dans `App.jsx`.
*   **UX** : Interface fonctionnelle mais basique. Manque de feedback visuel avancé pour l'utilisateur lors des phases de "réflexion" des différents agents.

---

## 3. Analyse Détaillée : Wyhtcard (Site Web)

*   **Architecture** : Excellente séparation des préoccupations. Utilisation de composants atomiques et d'un système de design centralisé (`styles/components.js`).
*   **Qualité** :
    *   **i18n** : Implémentation complète et robuste.
    *   **Performance** : Utilisation de `framer-motion` optimisée et lazy loading.
    *   **SEO** : Balisage sémantique correct.
*   **Conclusion** : Ce module est en état "Release Candidate".

---

## 4. Écart par rapport à la Vision (Gap Analysis)

| Fonctionnalité Clé | Vision (Cible) | État Actuel (Réalité) |
| :--- | :--- | :--- |
| **Mixture of Agents** | Orchestration de modèles hétérogènes (Llama, Phi, Gemma) | Simulation via Prompts sur modèle unique (Qwen) |
| **Exécution** | Parallèle (Vitesse & Diversité) | Séquentielle (Lenteur cumulative) |
| **Infrastructure** | Gestion dynamique de N processus serveurs | Gestion statique d'un seul processus |
| **Réseau** | Découverte LAN & P2P | Localhost uniquement |

---

## 5. Recommandations Stratégiques

### Priorité 1 : Refonte Backend pour le Vrai Multi-Modèle (V2)
Pour atteindre la vision, le Backend Rust doit évoluer :
1.  **Dynamic Server Manager** : Remplacer `Option<Child>` par `HashMap<Port, Child>` pour gérer plusieurs serveurs en parallèle.
2.  **Configuration Dynamique** : Créer un fichier `agents.json` permettant de définir quel modèle utiliser pour quel expert (ex: "Expert Logique" -> charge `phi-3.gguf` sur le port 8081).
3.  **Téléchargement Multi-Modèles** : Étendre `llama_install.rs` pour gérer une file d'attente de téléchargements.

### Priorité 2 : Sécurisation & Robustesse
1.  **Error Handling** : Remplacer tous les `unwrap()` par une gestion d'erreur propre (`Result<T, AppError>`) propagée au Frontend.
2.  **Network Hardening** : Forcer le binding de `llama-server` sur `127.0.0.1` explicitement.

### Priorité 3 : Expérience Utilisateur (Frontend WhytChat)
1.  **Visualisation du Raisonnement** : Afficher en temps réel quel agent est en train de "réfléchir" et son statut (Chargement modèle -> Inférence -> Réponse).
2.  **Gestion des Modèles** : Ajouter une page "Paramètres" pour voir les modèles téléchargés et assigner les experts.

## 6. Plan d'Action Immédiat

1.  **Refactor Rust** : Sécuriser le code existant (suppression `unwrap`) avant d'ajouter de la complexité.
2.  **Architecture V2** : Implémenter la structure de données `AgentRegistry` dans le Backend.
3.  **POC Multi-Process** : Tester le lancement simultané de 2 modèles légers (ex: Qwen 0.5B et Phi-3) pour valider la charge mémoire.

---
*Fin du rapport.*