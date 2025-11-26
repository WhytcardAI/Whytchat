# 📚 Système RAG (Retrieval-Augmented Generation)

Ce document détaille l'implémentation technique du système RAG dans WhytChat V1.

## 🎯 Objectif

Le système RAG permet au modèle de langage (LLM) d'accéder à des données privées et spécifiques à l'utilisateur qui ne font pas partie de son entraînement initial. Dans WhytChat, ce système est **local**, **privé** et **isolé par session**.

## 🏗️ Architecture Technique

### 1. Stack Technologique

- **Base de Données Vectorielle** : [LanceDB](https://lancedb.com/) (Embarquée, sans serveur).
- **Modèle d'Embedding** : `AllMiniLML6V2` via [fastembed-rs](https://github.com/Anush008/fastembed-rs).
  - _Dimension_ : 384
  - _Taille_ : ~23 MB (téléchargé automatiquement au premier lancement).
  - _Performance_ : Très rapide sur CPU, idéal pour une application desktop.

### 2. Isolation et Filtrage des Données

Contrairement à beaucoup de systèmes RAG qui mélangent toutes les données dans un index global, WhytChat implémente une **gestion granulaire par fichier**.

- Chaque chunk de texte ingéré est tagué avec une métadonnée `metadata`: `file:{file_id}`.
- Les fichiers sont stockés dans une **Librairie Globale** (`library_files`) et peuvent être liés à plusieurs sessions.
- Lors de la recherche, un filtre strict est appliqué sur les fichiers liés à la session active : `WHERE metadata IN ('file:ID1', 'file:ID2', ...)`.
- **Bénéfice** : Flexibilité totale. Un même document peut être utilisé dans plusieurs contextes sans duplication, et les sessions ne voient que ce qui leur est explicitement lié.

## 🔄 Flux de Données

### A. Ingestion (Upload)

1.  **Upload UI** : L'utilisateur upload un fichier via la `DataSidebar`.
2.  **Stockage Fichier** : Le fichier original est sauvegardé dans `data/library/{file_id}_{filename}`.
3.  **Enregistrement DB** : Une entrée est créée dans la table `library_files` (SQLite).
4.  **Liaison** : Le fichier est lié à la session courante via `session_files_link`.
5.  **Traitement (RagActor)** :
    - **Lecture** : Le contenu est lu en mémoire.
    - **Chunking** : Découpage par sauts de ligne (`\n`) avec filtrage des lignes trop courtes (< 20 chars).
    - **Embedding** : Conversion des chunks en vecteurs (Float32Array[384]).
    - **Indexation** : Écriture dans la table `knowledge_base` de LanceDB avec le tag `file:{id}`.

### B. Récupération (Retrieval)

1.  **Message Utilisateur** : L'utilisateur envoie un message.
2.  **Analyse (Brain)** : Le système détermine si le RAG est nécessaire (`should_use_rag`).
3.  **Récupération des IDs** : Le Supervisor récupère la liste des `file_ids` liés à la session.
4.  **Recherche (RagActor)** :
    - La requête utilisateur est vectorisée.
    - Recherche ANN (Approximate Nearest Neighbor) dans LanceDB.
    - **Filtre** : `metadata IN (file:id1, file:id2...)`.
    - **Limit** : Top 3 résultats les plus proches.
5.  **Construction du Prompt** :
    - Les chunks trouvés sont concaténés.
    - Ils sont injectés dans le prompt système sous la section `Context:`.

## 💾 Schéma de Données (LanceDB)

La table `knowledge_base` suit ce schéma Arrow :

| Champ      | Type               | Description                             |
| :--------- | :----------------- | :-------------------------------------- |
| `id`       | Utf8               | UUID unique du chunk.                   |
| `content`  | Utf8               | Le texte brut du chunk.                 |
| `metadata` | Utf8               | Tag de fichier (ex: `file:123-abc`).    |
| `vector`   | FixedSizeList<f32> | Le vecteur d'embedding (dim 384).       |

## 🚀 Performance & Optimisations

- **Cache LRU** : Les embeddings des requêtes fréquentes sont mis en cache (taille 1000) pour éviter de recalculer les vecteurs inutilement.
- **LanceDB** : Utilise un index sur disque optimisé, permettant de gérer des millions de vecteurs sans charger toute la DB en RAM.

---

_Document généré automatiquement - Novembre 2025_
