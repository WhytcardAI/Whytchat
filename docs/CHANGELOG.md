# Journal des Modifications

Toutes les modifications notables de ce projet seront documentées dans ce fichier.

Le format est basé sur [Keep a Changelog](https://keepachangelog.com/fr/1.0.0/), et ce projet adhère au [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Non publié]

### Ajouté

- **Moteur Cognitif (Thinking Mode)** : Implémentation de la boucle de raisonnement dans `supervisor.rs` (Analyse -> Recherche -> Réponse).
- **Intégration LLM Local** : Lancement automatique de `llama-server.exe` via `llm.rs` et communication HTTP.
- **Feedback Temps Réel** : Émission d'événements `thinking-step` du Backend vers le Frontend pour afficher le processus de pensée.
- **Interface de Chat** : Mise à jour de `ChatInterface.jsx` pour écouter les événements de pensée et afficher les étapes en temps réel.
- **PortablePathManager** (`apps/core/src/fs_manager.rs`) : Module centralisé pour la gestion des chemins de fichiers en mode portable.
- Implémentation du **Confinement Strict** : Toutes les données (DB, modèles, logs) sont désormais forcées dans un répertoire `./data` relatif à l'exécutable.
- Initialisation automatique de l'arborescence `./data`, `./data/db`, `./data/models`, `./data/vectors` au démarrage de l'application.
- **Outils de Build Locaux** : Intégration de `protoc` et `cmake` dans `apps/core/tools` pour garantir une compilation autonome sur Windows.
- **Icônes** : Ajout des icônes par défaut dans `apps/core/icons` pour permettre le packaging Tauri.

### Modifié

- **Gestion des Chemins (Dev)** : Patch de `fs_manager.rs` pour supporter correctement le dossier `data` en mode développement (`cargo tauri dev`).
- **Supervisor** : Transformation en véritable orchestrateur multi-agents.
- **Architecture** : Transition vers une architecture totalement "Air-Gapped" et portable par défaut.
- `apps/core/src/main.rs` : Intégration de l'initialisation du système de fichiers portable au démarrage du backend.
- **Dépendances Rust** : Mise à jour de `lancedb` (0.10), `fastembed` (4), `chrono` (0.4) et `arrow` (52.2.0) pour résoudre les conflits de compilation et les erreurs de linkage sur Windows.
- **Configuration de Build** : Ajout de `apps/core/build.rs` pour la gestion correcte du contexte Tauri.

### Corrigé

- Résolution des erreurs de compilation liées à `ort` et `aws-lc-sys` en forçant l'utilisation de `cmake` local et en ajustant les features des crates.
- Correction des conflits de version `arrow-arith` via l'alignement des versions de `chrono` et `arrow`.
