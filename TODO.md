# Roadmap de Développement - WhytChat V1

Cette roadmap suit l'implémentation stricte des 6 piliers de l'architecture V1.

## Phase 1 : Le Cerveau Cognitif & Fondations (Core)
*Objectif : Un backend Rust capable de "penser" localement avant de répondre.*

- [x] **Moteur d'Inférence Local (Rust -> llama-server)**
    - [x] Implémenter le Supervisor Actor pour gérer le processus `llama-server`.
    - [x] Créer le client HTTP Rust interne pour communiquer avec l'API locale du LLM.
    - [x] Définir les structs de requêtes/réponses (Sampling params, Prompt template).
- [x] **Architecture "Agents Invisibles"**
    - [x] Implémenter la chaîne cognitive : `Input -> Planificateur -> Exécuteur -> Critique -> Output`.
    - [x] Gérer l'état de "Pensée" (événements envoyés au frontend pour feedback visuel).

## Phase 2 : Installation Portable & Données (Store)
*Objectif : Zéro AppData, tout dans le dossier de l'exécutable.*

- [x] **Gestion de Fichiers Portable**
    - [x] Configurer Tauri pour utiliser un chemin relatif pour la base de données et les logs.
    - [x] Bloquer toute écriture dans les répertoires système par défaut.
- [ ] **Bibliothèque de Connaissance (LanceDB)**
    - [ ] Intégrer LanceDB en mode embarqué.
    - [ ] Créer le schéma de base pour la `Global Library` et les `Clusters`.
    - [ ] Implémenter l'ingestion de documents (Texte -> Chunking -> Embedding -> Store).

## Phase 3 : Interface & Session Atomique (UI)
*Objectif : Une UI "bête" mais réactive et configurable.*

- [ ] **Session Manager**
    - [ ] Créer le modèle de données `SessionConfig` (System prompt, Température, Context injections).
    - [ ] Implémenter la persistance des sessions en SQLite.
- [ ] **Focus Mode UI**
    - [ ] Développer la vue "Focus" (minimaliste, chat centré).
    - [ ] Intégrer le sélecteur de configuration par chat (Drawer de paramètres).

## Phase 4 : Connectivité & Onboarding (Features)
*Objectif : Ouverture contrôlée et premier lancement.*

- [ ] **Connectivité (Tavily)**
    - [ ] Implémenter le service `WebSearch` (appel API Tavily sécurisé).
    - [ ] Créer le Toggle UI "Accès Web" et le connecter au backend.
- [ ] **Onboarding Flow**
    - [ ] Créer l'écran de sélection de langue (i18n initial).
    - [ ] Développer le gestionnaire de téléchargement de modèles (Barre de progression, vérification de hash).
    - [ ] Intégrer la séquence éducative "Privacy First" (Slides/Animation).

## Phase 5 : Packaging & Release (Distribution)
*Objectif : Un exécutable unique, portable et signé.*

- [ ] **CI/CD**
    - [ ] Configurer le build pour inclure les binaires sidecar (`llama-server`) si nécessaire ou le script de DL.
    - [ ] Tests de portabilité (Vérifier l'absence de résidus après suppression du dossier).
