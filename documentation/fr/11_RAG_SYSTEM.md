# 🔍 Système RAG - WhytChat V1

> Retrieval-Augmented Generation avec LanceDB et FastEmbed

---

## 🎯 Vue d'Ensemble

Le système RAG (Retrieval-Augmented Generation) permet d'enrichir les réponses du LLM avec du contexte provenant de documents indexés.

```
┌─────────────────────────────────────────────────────────────────┐
│                        RAG Pipeline                             │
│                                                                 │
│  [Document]  ──► [Chunking] ──► [Embedding] ──► [LanceDB]       │
│                                                                 │
│  [Query] ──► [Embedding] ──► [Vector Search] ──► [Top-K Results]│
│                                                                 │
│  [Results + Query] ──► [LLM] ──► [Augmented Response]           │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

---

## 📦 Composants

| Composant    | Technologie   | Version |
| ------------ | ------------- | ------- |
| Vector DB    | LanceDB       | 0.10    |
| Embeddings   | FastEmbed     | 4       |
| Modèle Embed | AllMiniLML6V2 | -       |
| Dimensions   | 384           | -       |
| Format Arrow | Arrow 52      | -       |

---

## 🗃️ LanceDB - Base Vectorielle

### Structure de la Table

```rust
// Schéma Arrow de la table "knowledge_base"
let schema = Arc::new(Schema::new(vec![
    Field::new("id", DataType::Utf8, false),           // UUID du chunk
    Field::new("content", DataType::Utf8, false),      // Texte du chunk
    Field::new("metadata", DataType::Utf8, true),      // Metadata JSON ("file:{id}")
    Field::new(
        "vector",
        DataType::FixedSizeList(
            Arc::new(Field::new("item", DataType::Float32, true)),
            384,  // Dimension AllMiniLML6V2
        ),
        true,
    ),
]));
```

### Emplacement

```
data/
└── vectors/
    └── knowledge_base.lance/    ← Table LanceDB
```

### Connexion

```rust
async fn initialize_lancedb(&mut self) -> Result<(), ActorError> {
    let db_path = self.db_path_override
        .clone()
        .unwrap_or_else(PortablePathManager::vectors_dir);

    let db_path_str = db_path.to_str().ok_or_else(|| {
        ActorError::RagError("Invalid database path".to_string())
    })?;

    let conn = connect(db_path_str).execute().await?;
    self.db_connection = Some(conn);

    Ok(())
}
```

---

## 🧠 FastEmbed - Embeddings

### Configuration

```rust
fn initialize_embedding_model(&mut self) -> Result<(), ActorError> {
    let embeddings_dir = PortablePathManager::models_dir().join("embeddings");

    let mut options = InitOptions::new(EmbeddingModel::AllMiniLML6V2);
    options.show_download_progress = false;
    options.cache_dir = embeddings_dir;

    let model = TextEmbedding::try_new(options)?;
    self.embedding_model = Some(model);

    Ok(())
}
```

### Caractéristiques du Modèle

| Propriété    | Valeur        |
| ------------ | ------------- |
| Modèle       | AllMiniLML6V2 |
| Dimensions   | 384           |
| Séquence max | 256 tokens    |
| Taille       | ~22 MB        |
| Performance  | ~5-10ms/texte |

### Cache Embeddings (LRU)

```rust
struct RagActorRunner {
    embedding_cache: LruCache<String, Vec<f32>>,
    // ...
}

const CACHE_SIZE: NonZeroUsize = unsafe { NonZeroUsize::new_unchecked(1000) };
```

Le cache LRU stocke les 1000 derniers embeddings de requêtes pour éviter les recalculs.

---

## ✂️ Chunking - Découpage des Documents

### Stratégie

```rust
const TARGET_CHUNK_SIZE: usize = 512;   // caractères
const OVERLAP_SIZE: usize = 50;         // caractères d'overlap

async fn ingest_document(
    &self,
    content: String,
    metadata: Option<String>,
) -> Result<String, ActorError> {

    let mut chunks: Vec<String> = Vec::new();
    let mut current_chunk = String::new();

    for line in content.split('\n') {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        if current_chunk.len() + trimmed.len() > TARGET_CHUNK_SIZE {
            // Chunk plein, le sauvegarder
            chunks.push(current_chunk.clone());

            // Nouveau chunk avec overlap
            let start_index = if current_chunk.len() > OVERLAP_SIZE {
                current_chunk.len() - OVERLAP_SIZE
            } else {
                0
            };
            let overlap = current_chunk[start_index..].to_string();
            current_chunk = overlap + " " + trimmed;
        } else {
            if !current_chunk.is_empty() {
                current_chunk.push(' ');
            }
            current_chunk.push_str(trimmed);
        }
    }

    // Dernier chunk
    if !current_chunk.is_empty() {
        chunks.push(current_chunk);
    }

    // Filtrer les chunks trop petits
    let chunks: Vec<String> = chunks.into_iter()
        .filter(|s| s.len() > 20)
        .collect();

    // ...
}
```

### Paramètres de Chunking

| Paramètre       | Valeur    | Raison                       |
| --------------- | --------- | ---------------------------- |
| Taille cible    | 512 chars | Équilibre contexte/précision |
| Overlap         | 50 chars  | Préserver la continuité      |
| Taille minimale | 20 chars  | Éviter les chunks vides      |
| Séparateur      | `\n`      | Préserver la structure       |

---

## 📥 Ingestion

### Processus Complet

```rust
async fn ingest_document(
    &self,
    content: String,
    metadata: Option<String>,
) -> Result<String, ActorError> {

    // 1. Chunking
    let chunks = self.chunk_content(&content);

    if chunks.is_empty() {
        return Ok("No valid chunks to ingest".to_string());
    }

    // 2. Génération des embeddings (batch)
    let embeddings = self.embedding_model
        .as_ref()
        .ok_or(ActorError::RagError("Model not loaded".to_string()))?
        .embed(chunks.clone(), None)?;

    // 3. Construction du RecordBatch Arrow
    let total_chunks = chunks.len();
    let embedding_dim = 384;

    let mut id_builder = StringBuilder::with_capacity(total_chunks, total_chunks * 36);
    let mut content_builder = StringBuilder::with_capacity(total_chunks, total_chunks * 256);
    let mut metadata_builder = StringBuilder::with_capacity(total_chunks, total_chunks * 64);

    let values_builder = Float32Builder::with_capacity(total_chunks * embedding_dim);
    let mut vector_builder = FixedSizeListBuilder::new(values_builder, embedding_dim as i32);

    let metadata_value = metadata.as_deref().unwrap_or("");

    for (i, chunk) in chunks.iter().enumerate() {
        id_builder.append_value(uuid::Uuid::new_v4().to_string());
        content_builder.append_value(chunk);
        metadata_builder.append_value(metadata_value);

        if let Some(embedding) = embeddings.get(i) {
            vector_builder.values().append_slice(embedding);
            vector_builder.append(true);
        }
    }

    let batch = RecordBatch::try_new(
        schema.clone(),
        vec![
            Arc::new(id_builder.finish()),
            Arc::new(content_builder.finish()),
            Arc::new(metadata_builder.finish()),
            Arc::new(vector_builder.finish()),
        ],
    )?;

    // 4. Insertion dans LanceDB
    let table_exists = conn.table_names().execute().await?
        .contains(&self.table_name);

    let reader = RecordBatchIterator::new(vec![Ok(batch)], schema.clone());

    if table_exists {
        let table = conn.open_table(&self.table_name).execute().await?;
        table.add(Box::new(reader)).execute().await?;
    } else {
        conn.create_table(&self.table_name, Box::new(reader)).execute().await?;
    }

    Ok(format!("Ingested {} chunks", total_chunks))
}
```

### Metadata Format

Les métadonnées suivent le format `file:{uuid}` :

```rust
// Pour un fichier de la bibliothèque
let metadata = Some(format!("file:{}", file_id));

// Exemple
// metadata = "file:550e8400-e29b-41d4-a716-446655440000"
```

---

## 🔍 Recherche

### Processus de Recherche

```rust
async fn search_documents(
    &mut self,
    query: String,
    file_ids: Vec<String>,
    limit: usize,
) -> Result<Vec<SearchResult>, ActorError> {

    // 1. Vérifier le cache
    let query_vec = match self.embedding_cache.get(&query) {
        Some(embedding) => {
            info!("Cache hit for query: '{}'", query);
            embedding.clone()
        }
        None => {
            info!("Cache miss for query: '{}'", query);
            let embedding = self.embedding_model
                .as_ref()
                .ok_or(ActorError::RagError("Model not loaded".to_string()))?
                .embed(vec![query.clone()], None)?
                .remove(0);

            self.embedding_cache.put(query.clone(), embedding.clone());
            embedding
        }
    };

    // 2. Vérifier si la table existe
    let table_names = conn.table_names().execute().await?;
    if !table_names.contains(&self.table_name) {
        return Ok(Vec::new());  // Pas encore de données
    }

    // 3. Construire la requête
    let table = conn.open_table(&self.table_name).execute().await?;
    let mut query = table.query();

    // 4. Appliquer le filtre par fichier
    if !file_ids.is_empty() {
        let filter = file_ids.iter()
            .map(|id| format!("metadata = 'file:{}'", id))
            .collect::<Vec<_>>()
            .join(" OR ");
        query = query.only_if(filter);
    }

    // 5. Recherche vectorielle
    let mut results = query
        .limit(limit)
        .nearest_to(query_vec)?
        .execute()
        .await?;

    // 6. Extraction des résultats
    let mut documents = Vec::new();

    while let Some(batch) = results.try_next().await? {
        let content_col = batch.column_by_name("content")?;
        let metadata_col = batch.column_by_name("metadata")?;
        let distance_col = batch.column_by_name("_distance")?;

        let content_array = content_col.as_any().downcast_ref::<StringArray>()?;
        let metadata_array = metadata_col.as_any().downcast_ref::<StringArray>()?;
        let distance_array = distance_col.as_any().downcast_ref::<Float32Array>()?;

        for i in 0..content_array.len() {
            if !content_array.is_null(i) {
                documents.push(SearchResult {
                    content: content_array.value(i).to_string(),
                    metadata: if metadata_array.is_null(i) {
                        None
                    } else {
                        Some(metadata_array.value(i).to_string())
                    },
                    score: if distance_array.is_null(i) {
                        0.0
                    } else {
                        distance_array.value(i)
                    },
                });
            }
        }
    }

    Ok(documents)
}
```

### Score de Similarité

LanceDB retourne une **distance** (plus petit = plus similaire).

```rust
pub struct SearchResult {
    pub content: String,
    pub metadata: Option<String>,
    pub score: f32,  // Distance (0 = identique)
}
```

---

## 🗑️ Suppression

### Supprimer les Vecteurs d'un Fichier

```rust
async fn delete_document_vectors(&self, file_id: String) -> Result<(), ActorError> {
    let conn = self.db_connection.as_ref()
        .ok_or(ActorError::RagError("DB not connected".to_string()))?;

    let table_names = conn.table_names().execute().await?;
    if !table_names.contains(&self.table_name) {
        return Ok(());  // Table n'existe pas, rien à supprimer
    }

    let table = conn.open_table(&self.table_name).execute().await?;

    // Suppression par prédicat SQL
    let predicate = format!("metadata = 'file:{}'", file_id);
    table.delete(&predicate).await?;

    info!("Deleted vectors for file: {}", file_id);
    Ok(())
}
```

---

## 🔄 Intégration avec le Supervisor

### Décision RAG

```rust
// Dans BrainAnalyzer
fn should_use_rag(&self, packet: &ContextPacket) -> bool {
    // RAG pour questions, analyses, explications
    if matches!(packet.intent, Intent::Question | Intent::Analysis | Intent::Explanation) {
        return true;
    }

    // RAG si complexité élevée
    if packet.complexity.overall > 0.6 {
        return true;
    }

    // RAG si keywords techniques
    let technical_keywords = ["code", "function", "api", "data", "file", "document"];
    if packet.keywords.iter().any(|k| technical_keywords.contains(&k.as_str())) {
        return true;
    }

    false
}
```

### Utilisation dans le Flux

```rust
// Dans SupervisorRunner
async fn handle_process_message(&self, ...) -> Result<String, String> {
    // 1. Analyse Brain
    let context = self.brain_analyzer.analyze(&content);

    // 2. RAG si nécessaire
    let rag_context = if context.should_use_rag {
        window.emit("thinking-step", "Searching knowledge base...")?;

        match self.rag_actor.search(&content, 5).await {
            Ok(results) if !results.is_empty() => {
                format_rag_context(&results)
            }
            _ => String::new()
        }
    } else {
        String::new()
    };

    // 3. Construire prompt avec contexte RAG
    let system_prompt = build_system_prompt(&context, &rag_context);

    // 4. Générer réponse
    self.llm_actor.stream_generate(&content, Some(&system_prompt), &window).await
}

fn format_rag_context(results: &[SearchResult]) -> String {
    let mut context = String::from("\n\n### Relevant Context:\n");
    for (i, result) in results.iter().enumerate() {
        context.push_str(&format!(
            "\n[{}] (score: {:.2})\n{}\n",
            i + 1,
            result.score,
            result.content
        ));
    }
    context
}
```

---

## 📊 Messages RAG Actor

### RagMessage Enum

```rust
pub enum RagMessage {
    /// Ingestion de contenu
    Ingest {
        content: String,
        metadata: Option<String>,
        responder: oneshot::Sender<Result<String, AppError>>,
    },

    /// Recherche avec filtres
    Search {
        query: String,
        file_ids: Vec<String>,
        limit: usize,
        responder: oneshot::Sender<Result<Vec<SearchResult>, AppError>>,
    },

    /// Suppression des vecteurs d'un fichier
    DeleteForFile {
        file_id: String,
        responder: oneshot::Sender<Result<(), AppError>>,
    },
}
```

### RagActorHandle

```rust
#[async_trait]
impl RagActor for RagActorHandle {
    async fn ingest(&self, content: String, metadata: Option<String>) -> Result<String, AppError>;
    async fn search_with_filters(&self, query: String, file_ids: Vec<String>) -> Result<Vec<SearchResult>, AppError>;
    async fn delete_for_file(&self, file_id: String) -> Result<(), AppError>;
}
```

---

## 📈 Performance

### Benchmarks Typiques

| Opération             | Temps    | Notes                 |
| --------------------- | -------- | --------------------- |
| Embedding (1 texte)   | 5-10ms   | GPU: 1-2ms            |
| Embedding (batch 100) | 50-100ms | Parallélisé           |
| Recherche top-5       | 10-50ms  | Dépend taille index   |
| Ingestion 1000 chunks | 2-5s     | Embedding + insertion |

### Optimisations

1. **Cache LRU** - Évite les recalculs d'embeddings fréquents
2. **Batch embedding** - Tous les chunks d'un document en une fois
3. **Lazy loading** - Modèle chargé seulement au premier besoin
4. **Index LanceDB** - Optimisé pour recherche vectorielle

---

## 🧪 Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_ingest_and_search() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().to_path_buf();

        let handle = RagActorHandle::new_with_options(Some(db_path), None);

        // Ingestion
        let result = handle.ingest(
            "Rust is a systems programming language".to_string(),
            Some("file:test-123".to_string())
        ).await;
        assert!(result.is_ok());

        // Recherche
        let results = handle.search_with_filters(
            "programming language".to_string(),
            vec![]
        ).await.unwrap();

        assert!(!results.is_empty());
        assert!(results[0].content.contains("Rust"));
    }

    #[tokio::test]
    async fn test_delete_vectors() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().to_path_buf();

        let handle = RagActorHandle::new_with_options(Some(db_path), None);

        // Ingestion
        handle.ingest(
            "Test content".to_string(),
            Some("file:to-delete".to_string())
        ).await.unwrap();

        // Suppression
        let result = handle.delete_for_file("to-delete".to_string()).await;
        assert!(result.is_ok());

        // Vérification
        let results = handle.search_with_filters(
            "Test".to_string(),
            vec!["to-delete".to_string()]
        ).await.unwrap();

        assert!(results.is_empty());
    }
}
```

---

## 📋 Checklist RAG

| Élément                 | Status | Notes                 |
| ----------------------- | ------ | --------------------- |
| LanceDB intégré         | ✅     | Version 0.10          |
| FastEmbed AllMiniLML6V2 | ✅     | 384 dimensions        |
| Chunking avec overlap   | ✅     | 512 chars, 50 overlap |
| Cache LRU embeddings    | ✅     | 1000 entrées          |
| Filtrage par fichier    | ✅     | Prédicat SQL          |
| Suppression par fichier | ✅     | delete_for_file       |
| Intégration Brain       | ✅     | should_use_rag        |
| Batch embedding         | ✅     | Performance optimisée |

---

_Généré depuis lecture directe de: actors/rag.rs, actors/messages.rs, actors/traits.rs, brain/analyzer.rs_
