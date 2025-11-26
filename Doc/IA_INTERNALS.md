# 🧠 IA Internals : Le Cerveau de WhytChat

Ce document explique le fonctionnement interne des composants d'Intelligence Artificielle de WhytChat.

## 🤖 Modèle de Langage (LLM)

WhytChat n'utilise pas d'API externe (OpenAI, Anthropic). Tout tourne localement.

- **Moteur** : `llama-server` (basé sur `llama.cpp`).
- **Format** : GGUF (GPT-Generated Unified Format).
- **Modèle par défaut** : `Qwen 2.5 7B Instruct` (Quantized Q4_K_M).
  - _Pourquoi ce choix ?_ Excellent support du français, très performant pour le code et capable de suivre des instructions complexes avec une empreinte mémoire raisonnable (4.6GB).

## 🧩 Le Module Brain V2

Le "Brain" est la couche d'intelligence intermédiaire entre l'utilisateur et le LLM. Il a pour but de réduire la latence et d'améliorer la pertinence des réponses en analysant la requête AVANT de solliciter le modèle lourd.

### Architecture Two-Tier

Pour garantir une réactivité maximale, l'analyse se fait en deux temps :

1.  **Fast Path (Regex)** - _~1ms_
    - Utilise des expressions régulières compilées (Rust `regex`) pour détecter des intentions évidentes ("Code moi...", "Traduis...", "Bonjour").
    - Si une correspondance est trouvée avec une confiance > 0.8, l'analyse s'arrête là.

2.  **Semantic Fallback (Embeddings)** - _~50ms_
    - Si aucune regex ne matche, la requête est vectorisée (voir ci-dessous).
    - Le vecteur est comparé (Cosine Similarity) avec des vecteurs de référence pour chaque intention (Question, Commande, Créatif, etc.).
    - C'est plus lent mais beaucoup plus robuste aux variations de langage.

### Structure du ContextPacket

Le Brain produit un `ContextPacket` qui est transmis au LLM via le Supervisor :

```rust
pub struct ContextPacket {
    pub intent: IntentResult,           // L'intention détectée (ex: CodeRequest)
    pub keywords: Vec<KeywordResult>,   // Mots-clés extraits (TF-IDF)
    pub complexity: ComplexityMetrics,  // Score de complexité (0.0 - 1.0)
    pub language: Language,             // Langue détectée (FR/EN)
    pub rag_results: Vec<RagResult>,    // Documents pertinents (si RAG activé)
}
```

## 📚 RAG (Retrieval-Augmented Generation)

Le système RAG permet au LLM de "lire" vos documents.

### Stack Vectorielle

- **Embeddings** : `fastembed` (Rust wrapper pour ONNX Runtime).
  - Modèle : `AllMiniLML6V2` (Transforme le texte en vecteurs de 384 dimensions).
  - _Note_ : Modèle très léger (~23MB) et rapide, optimisé pour le CPU.
- **Stockage** : `LanceDB` (Base de données vectorielle embarquée, sans serveur).
  - Stocke les vecteurs et les métadonnées sur le disque local.

### Workflow d'Ingestion

1.  Fichier uploadé -> Texte brut.
2.  Découpage (Chunking) : Par ligne (split `\n`) avec filtre de longueur minimale.
3.  Vectorisation : `Text -> [0.12, -0.45, ...]`
4.  Indexation : LanceDB écrit les données sur le disque avec le tag `file:{id}`.

### Workflow de Recherche

1.  Question utilisateur -> Vecteur Query.
2.  Recherche ANN (Approximate Nearest Neighbor) dans LanceDB.
3.  **Filtrage** : Restriction aux fichiers liés à la session (`metadata IN ('file:ID1', ...)`).
4.  Récupération des 3 chunks les plus proches (Top-K).
5.  Injection dans le prompt système :
    > "Context:\n[CHUNK 1]\n[CHUNK 2]..."

## 🔮 Évolutions Futures (Phase 2)

- Remplacement des Regex par un modèle ONNX dédié (`DistilBERT`) pour la classification d'intention.
- Ajout de NER (Named Entity Recognition) pour extraire des noms propres et lieux.
- Amélioration du Chunking (Recursive Character Splitter) pour ne pas couper les phrases.

---

_Basé sur ARCHITECTURE_BRAIN_V2.md - Novembre 2025_
