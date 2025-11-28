use crate::actors::messages::{ActorError, AppError, RagMessage, SearchResult};
use crate::actors::traits::RagActor;
use crate::fs_manager::PortablePathManager;
use arrow::array::{
    Array, FixedSizeListBuilder, Float32Array, Float32Builder, RecordBatch, RecordBatchIterator,
    StringArray, StringBuilder,
};
use arrow::datatypes::{DataType, Field, Schema};
use async_trait::async_trait;
use fastembed::{EmbeddingModel, InitOptions, TextEmbedding};
use futures::TryStreamExt;
use lancedb::{
    connect,
    query::{ExecutableQuery, QueryBase},
    Connection,
};
use lru::LruCache;
use sqlx::sqlite::SqlitePool;
use std::num::NonZeroUsize;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::{mpsc, oneshot};
use tracing::{error, info, warn};

/// A handle to the `RagActor`.
///
/// This provides a public, cloneable interface for sending messages to the running RAG actor,
/// which manages the knowledge base and document retrieval.
#[derive(Clone)]
pub struct RagActorHandle {
    sender: mpsc::Sender<RagMessage>,
}

impl RagActorHandle {
    /// Creates a new `RagActor` with default options and returns a handle to it.
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self::new_with_options(None, None)
    }

    /// Creates a new `RagActor` with specific configuration options.
    ///
    /// This is useful for testing, allowing injection of a temporary database path or a
    /// pre-configured database pool.
    ///
    /// # Arguments
    ///
    /// * `db_path` - An optional override for the LanceDB vector database path.
    /// * `pool` - An optional `SqlitePool` for accessing session file metadata.
    pub fn new_with_options(db_path: Option<PathBuf>, pool: Option<SqlitePool>) -> Self {
        let (sender, receiver) = mpsc::channel(32);
        let actor = RagActorRunner::new(receiver, db_path, pool);
        tokio::spawn(async move { actor.run().await });
        Self { sender }
    }
}

#[async_trait]
impl RagActor for RagActorHandle {
    async fn ingest(&self, content: String, metadata: Option<String>) -> Result<String, AppError> {
        let (send, recv) = oneshot::channel();
        let msg = RagMessage::Ingest {
            content,
            metadata,
            responder: send,
        };
        self.sender
            .send(msg)
            .await
            .map_err(|_| AppError::Actor(ActorError::Internal("RAG Actor closed".to_string())))?;
        Ok(recv.await.map_err(|_| {
            AppError::Actor(ActorError::Internal(
                "RAG Actor failed to respond".to_string(),
            ))
        })??)
    }

    async fn search_with_filters(
        &self,
        query: String,
        file_ids: Vec<String>,
    ) -> Result<Vec<SearchResult>, AppError> {
        let (send, recv) = oneshot::channel();
        let msg = RagMessage::Search {
            query,
            file_ids,
            limit: 3, // Default limit
            responder: send,
        };
        self.sender
            .send(msg)
            .await
            .map_err(|_| AppError::Actor(ActorError::Internal("RAG Actor closed".to_string())))?;
        Ok(recv.await.map_err(|_| {
            AppError::Actor(ActorError::Internal(
                "RAG Actor failed to respond".to_string(),
            ))
        })??)
    }

    async fn delete_for_file(&self, file_id: String) -> Result<(), AppError> {
        let (send, recv) = oneshot::channel();
        let msg = RagMessage::DeleteForFile {
            file_id,
            responder: send,
        };
        self.sender
            .send(msg)
            .await
            .map_err(|_| AppError::Actor(ActorError::Internal("RAG Actor closed".to_string())))?;
        Ok(recv.await.map_err(|_| {
            AppError::Actor(ActorError::Internal(
                "RAG Actor failed to respond".to_string(),
            ))
        })??)
    }
}

// --- Actor Runner (Internal Logic) ---
struct RagActorRunner {
    receiver: mpsc::Receiver<RagMessage>,
    embedding_model: Option<TextEmbedding>,
    embedding_cache: LruCache<String, Vec<f32>>,
    db_connection: Option<Connection>,
    table_name: String,
    db_path_override: Option<PathBuf>,
    #[allow(dead_code)]
    pool: Option<SqlitePool>,
}

impl RagActorRunner {
    /// The cache size for the embedding cache.
    /// NonZeroUsize::new(1000) always succeeds since 1000 > 0.
    // SAFETY: 1000 is strictly positive, so new_unchecked is safe.
    // We use new_unchecked because Option::expect is not const-stable in Rust 1.80.0 (requires 1.83.0)
    const CACHE_SIZE: NonZeroUsize = unsafe { NonZeroUsize::new_unchecked(1000) };

    fn new(
        receiver: mpsc::Receiver<RagMessage>,
        db_path_override: Option<PathBuf>,
        pool: Option<SqlitePool>,
    ) -> Self {
        Self {
            receiver,
            embedding_model: None,
            embedding_cache: LruCache::new(Self::CACHE_SIZE),
            db_connection: None,
            table_name: "knowledge_base".to_string(),
            db_path_override,
            pool,
        }
    }

    async fn run(mut self) {
        info!("RagActor started");

        // Initialize components - errors are logged but don't stop the actor
        // as it can still function with reduced capabilities
        if let Err(e) = self.initialize_embedding_model() {
            error!("Failed to initialize embedding model: {}", e);
        }

        if let Err(e) = self.initialize_lancedb().await {
            error!("Failed to initialize LanceDB: {}", e);
        }

        while let Some(msg) = self.receiver.recv().await {
            self.handle_message(msg).await;
        }
        info!("RagActor stopped");
    }

    /// Initializes the FastEmbed embedding model.
    /// Returns an error if the model cannot be loaded.
    fn initialize_embedding_model(&mut self) -> Result<(), ActorError> {
        let embeddings_dir = PortablePathManager::models_dir().join("embeddings");
        let mut options = InitOptions::new(EmbeddingModel::AllMiniLML6V2);
        options.show_download_progress = false;
        options.cache_dir = embeddings_dir;

        match TextEmbedding::try_new(options) {
            Ok(model) => {
                info!("Embedding model loaded successfully");
                self.embedding_model = Some(model);
                Ok(())
            }
            Err(e) => Err(ActorError::RagError(format!(
                "Failed to load embedding model: {}",
                e
            ))),
        }
    }

    /// Initializes the LanceDB vector database connection.
    /// Returns an error if the database path is invalid or connection fails.
    async fn initialize_lancedb(&mut self) -> Result<(), ActorError> {
        let db_path = self
            .db_path_override
            .clone()
            .unwrap_or_else(PortablePathManager::vectors_dir);

        // Ensure directory exists if it's a custom path (PortablePathManager handles the default one)
        if self.db_path_override.is_some() {
            if let Some(parent) = db_path.parent() {
                if let Err(e) = std::fs::create_dir_all(parent) {
                    return Err(ActorError::RagError(format!(
                        "Failed to create database directory at {:?}: {}",
                        parent, e
                    )));
                }
            }
        }

        let db_path_str = db_path.to_str().ok_or_else(|| {
            ActorError::RagError(format!(
                "Failed to convert database path to string: {:?}",
                db_path
            ))
        })?;

        match connect(db_path_str).execute().await {
            Ok(conn) => {
                info!("Connected to LanceDB at {:?}", db_path);
                self.db_connection = Some(conn);
                Ok(())
            }
            Err(e) => Err(ActorError::RagError(format!(
                "Failed to connect to LanceDB: {}",
                e
            ))),
        }
    }

    async fn handle_message(&mut self, msg: RagMessage) {
        match msg {
            RagMessage::Ingest {
                content,
                metadata,
                responder,
            } => {
                let result = self.ingest_document(content, metadata).await;
                if responder.send(result.map_err(AppError::from)).is_err() {
                    warn!("Failed to send ingest response (channel closed)");
                }
            }
            RagMessage::Search {
                query,
                file_ids,
                limit,
                responder,
            } => {
                let result = self.search_documents(query, file_ids, limit).await;
                if responder.send(result.map_err(AppError::from)).is_err() {
                    warn!("Failed to send search response (channel closed)");
                }
            }
            RagMessage::DeleteForFile { file_id, responder } => {
                let result = self.delete_document_vectors(file_id).await;
                if responder.send(result.map_err(AppError::from)).is_err() {
                    warn!("Failed to send delete response (channel closed)");
                }
            }
        }
    }

    async fn ingest_document(
        &self,
        content: String,
        metadata: Option<String>,
    ) -> Result<String, ActorError> {
        let model = self.embedding_model.as_ref().ok_or(ActorError::RagError(
            "Embedding model not loaded".to_string(),
        ))?;
        let conn = self
            .db_connection
            .as_ref()
            .ok_or(ActorError::RagError("DB not connected".to_string()))?;

        // 1. Improved Chunking with overlap
        // We accumulate lines until we reach a target size (e.g., 512 chars)
        // and then emit a chunk. We keep an overlap (e.g., 50 chars) from the previous chunk.
        let target_chunk_size = 512;
        let overlap_size = 50;
        let mut chunks: Vec<String> = Vec::new();
        let mut current_chunk = String::new();

        for line in content.split('\n') {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }

            if current_chunk.len() + trimmed.len() > target_chunk_size {
                // Chunk is full, push it
                chunks.push(current_chunk.clone());

                // Start new chunk with overlap
                let start_index = if current_chunk.len() > overlap_size {
                    current_chunk.len() - overlap_size
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

        // Push the last chunk if not empty
        if !current_chunk.is_empty() {
            chunks.push(current_chunk);
        }

        // Filter out very small chunks that might be noise
        let chunks: Vec<String> = chunks.into_iter().filter(|s| s.len() > 20).collect();

        if chunks.is_empty() {
            warn!(
                "Document ingestion skipped: No valid chunks found (content length: {})",
                content.len()
            );
            return Ok(
                "No valid chunks to ingest (content might be too short or empty)".to_string(),
            );
        }

        // 2. Generate Embeddings
        let embeddings = model
            .embed(chunks.clone(), None)
            .map_err(|e| ActorError::RagError(format!("Embedding failed: {}", e)))?;

        // 3. Construct Arrow RecordBatch
        let total_chunks = chunks.len();
        let embedding_dim = 384; // AllMiniLML6V2 dimension

        // Define Schema with metadata column
        let schema = Arc::new(Schema::new(vec![
            Field::new("id", DataType::Utf8, false),
            Field::new("content", DataType::Utf8, false),
            Field::new("metadata", DataType::Utf8, true),
            Field::new(
                "vector",
                DataType::FixedSizeList(
                    Arc::new(Field::new("item", DataType::Float32, true)),
                    embedding_dim as i32,
                ),
                true,
            ),
        ]));

        // Build Arrays
        let mut id_builder = StringBuilder::with_capacity(total_chunks, total_chunks * 36);
        let mut content_builder = StringBuilder::with_capacity(total_chunks, total_chunks * 256);
        let mut metadata_builder = StringBuilder::with_capacity(total_chunks, total_chunks * 64);

        // Vector Builder: List of Floats
        let values_builder = Float32Builder::with_capacity(total_chunks * embedding_dim);
        let mut vector_builder = FixedSizeListBuilder::new(values_builder, embedding_dim as i32);

        // Use metadata (e.g., "session:uuid" format)
        let metadata_value = metadata.as_deref().unwrap_or("");

        for (i, chunk) in chunks.iter().enumerate() {
            id_builder.append_value(uuid::Uuid::new_v4().to_string());
            content_builder.append_value(chunk);
            metadata_builder.append_value(metadata_value);

            // Append vector
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
        )
        .map_err(|e| ActorError::RagError(format!("Failed to create RecordBatch: {}", e)))?;

        // 4. Ingest into LanceDB
        // Open or Create table
        let table_exists = conn
            .table_names()
            .execute()
            .await
            .map_err(|e| ActorError::RagError(format!("Failed to list tables: {}", e)))?
            .contains(&self.table_name);

        let reader = RecordBatchIterator::new(vec![Ok(batch)], schema.clone());

        if table_exists {
            let table = conn
                .open_table(&self.table_name)
                .execute()
                .await
                .map_err(|e| ActorError::RagError(format!("Failed to open table: {}", e)))?;

            table
                .add(Box::new(reader))
                .execute()
                .await
                .map_err(|e| ActorError::RagError(format!("Failed to add data: {}", e)))?;
        } else {
            conn.create_table(&self.table_name, Box::new(reader))
                .execute()
                .await
                .map_err(|e| ActorError::RagError(format!("Failed to create table: {}", e)))?;
        }

        info!("Ingested {} chunks into LanceDB", total_chunks);
        Ok(format!("Ingested {} chunks", total_chunks))
    }

    async fn search_documents(
        &mut self,
        query: String,
        file_ids: Vec<String>,
        limit: usize,
    ) -> Result<Vec<SearchResult>, ActorError> {
        let model = self.embedding_model.as_ref().ok_or(ActorError::RagError(
            "Embedding model not loaded".to_string(),
        ))?;
        let conn = self
            .db_connection
            .as_ref()
            .ok_or(ActorError::RagError("DB not connected".to_string()))?;

        // 1. Embed Query (with cache)
        let query_vec = match self.embedding_cache.get(&query) {
            Some(embedding) => {
                info!("Cache hit for query: '{}'", query);
                embedding.clone()
            }
            None => {
                info!("Cache miss for query: '{}'", query);
                let query_embedding = model
                    .embed(vec![query.clone()], None)
                    .map_err(|e| ActorError::RagError(format!("Embedding failed: {}", e)))?;
                let embedding = query_embedding
                    .first()
                    .ok_or(ActorError::RagError("No embedding generated".to_string()))?
                    .clone();
                self.embedding_cache.put(query.clone(), embedding.clone());
                embedding
            }
        };

        // 2. Search in LanceDB
        // Check if table exists first
        let table_names = conn
            .table_names()
            .execute()
            .await
            .map_err(|e| ActorError::RagError(format!("Failed to list tables: {}", e)))?;

        if !table_names.contains(&self.table_name) {
            return Ok(Vec::new());
        }

        let table = conn
            .open_table(&self.table_name)
            .execute()
            .await
            .map_err(|e| ActorError::RagError(format!("Failed to open table: {}", e)))?;

        let mut query = table.query();

        // Apply file filter if provided
        if !file_ids.is_empty() {
            // Construct OR filter: metadata = 'file:ID1' OR metadata = 'file:ID2'
            // Note: metadata field stores "file:{id}"
            let filter = file_ids
                .iter()
                .map(|id| format!("metadata = 'file:{}'", id))
                .collect::<Vec<_>>()
                .join(" OR ");
            query = query.only_if(filter);
        }

        let mut results = query
            .limit(limit)
            .nearest_to(query_vec)
            .map_err(|e| ActorError::RagError(format!("Query setup failed: {}", e)))?
            .execute()
            .await
            .map_err(|e| ActorError::RagError(format!("Search failed: {}", e)))?;

        // 3. Extract Content
        let mut documents = Vec::new();

        while let Some(batch) = results
            .try_next()
            .await
            .map_err(|e| ActorError::RagError(format!("Stream error: {}", e)))?
        {
            let content_col = batch.column_by_name("content").ok_or(ActorError::RagError(
                "Column 'content' not found".to_string(),
            ))?;
            let metadata_col = batch
                .column_by_name("metadata")
                .ok_or(ActorError::RagError(
                    "Column 'metadata' not found".to_string(),
                ))?;
            let distance_col = batch
                .column_by_name("_distance")
                .ok_or(ActorError::RagError(
                    "Column '_distance' not found".to_string(),
                ))?;

            let content_array =
                content_col
                    .as_any()
                    .downcast_ref::<StringArray>()
                    .ok_or(ActorError::RagError(
                        "Failed to downcast content column".to_string(),
                    ))?;
            let metadata_array =
                metadata_col
                    .as_any()
                    .downcast_ref::<StringArray>()
                    .ok_or(ActorError::RagError(
                        "Failed to downcast metadata column".to_string(),
                    ))?;
            let distance_array = distance_col.as_any().downcast_ref::<Float32Array>().ok_or(
                ActorError::RagError("Failed to downcast distance column".to_string()),
            )?;

            for i in 0..content_array.len() {
                if !content_array.is_null(i) {
                    let text = content_array.value(i).to_string();
                    let meta = if metadata_array.is_null(i) {
                        None
                    } else {
                        Some(metadata_array.value(i).to_string())
                    };
                    let score = if distance_array.is_null(i) {
                        0.0
                    } else {
                        distance_array.value(i)
                    };

                    documents.push(SearchResult {
                        content: text,
                        metadata: meta,
                        score,
                    });
                }
            }
        }

        Ok(documents)
    }

    async fn delete_document_vectors(&self, file_id: String) -> Result<(), ActorError> {
        let conn = self
            .db_connection
            .as_ref()
            .ok_or(ActorError::RagError("DB not connected".to_string()))?;

        let table_names = conn
            .table_names()
            .execute()
            .await
            .map_err(|e| ActorError::RagError(format!("Failed to list tables: {}", e)))?;

        if !table_names.contains(&self.table_name) {
            return Ok(());
        }

        let table = conn
            .open_table(&self.table_name)
            .execute()
            .await
            .map_err(|e| ActorError::RagError(format!("Failed to open table: {}", e)))?;

        // Delete where metadata = 'file:{file_id}'
        let predicate = format!("metadata = 'file:{}'", file_id);

        table
            .delete(&predicate)
            .await
            .map_err(|e| ActorError::RagError(format!("Failed to delete vectors: {}", e)))?;

        info!("Deleted vectors for file: {}", file_id);
        Ok(())
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use tokio::time::{timeout, Duration};

    /// Helper to create a RAG actor with a temporary database
    async fn create_test_rag_actor() -> (RagActorHandle, TempDir) {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let db_path = temp_dir.path().join("test_vectors");

        let handle = RagActorHandle::new_with_options(Some(db_path), None);

        // Give the actor time to initialize
        tokio::time::sleep(Duration::from_millis(500)).await;

        (handle, temp_dir)
    }

    #[tokio::test]
    async fn test_rag_actor_creation() {
        let (handle, _temp_dir) = create_test_rag_actor().await;

        // Actor should be running - test by attempting a search (should return empty)
        let result = timeout(
            Duration::from_secs(10),
            handle.search_with_filters("test query".to_string(), vec![]),
        )
        .await;

        assert!(result.is_ok(), "Search should complete within timeout");
        let search_result = result.unwrap();
        assert!(
            search_result.is_ok(),
            "Search should succeed: {:?}",
            search_result
        );
    }

    #[tokio::test]
    async fn test_ingest_single_document() {
        let (handle, _temp_dir) = create_test_rag_actor().await;

        let content =
            "This is a test document about machine learning and artificial intelligence. \
                       It contains information about neural networks and deep learning algorithms.";

        let result = timeout(
            Duration::from_secs(30),
            handle.ingest(content.to_string(), Some("file:test-doc-1".to_string())),
        )
        .await;

        assert!(result.is_ok(), "Ingest should complete within timeout");
        let ingest_result = result.unwrap();
        assert!(
            ingest_result.is_ok(),
            "Ingest should succeed: {:?}",
            ingest_result
        );

        let message = ingest_result.unwrap();
        assert!(
            message.contains("Ingested") || message.contains("chunks"),
            "Should return confirmation message: {}",
            message
        );
    }

    #[tokio::test]
    async fn test_ingest_and_search() {
        let (handle, _temp_dir) = create_test_rag_actor().await;

        // Ingest a document about Rust programming
        let content = "Rust is a systems programming language focused on safety and performance. \
                       It provides memory safety without garbage collection. \
                       The borrow checker ensures safe memory access at compile time. \
                       Rust is great for building reliable and efficient software.";

        let ingest_result = timeout(
            Duration::from_secs(30),
            handle.ingest(content.to_string(), Some("file:rust-doc".to_string())),
        )
        .await
        .expect("Ingest timeout")
        .expect("Ingest failed");

        assert!(ingest_result.contains("Ingested"));

        // Search for relevant content
        let search_result = timeout(
            Duration::from_secs(10),
            handle.search_with_filters("memory safety in Rust".to_string(), vec![]),
        )
        .await
        .expect("Search timeout")
        .expect("Search failed");

        assert!(!search_result.is_empty(), "Should find relevant documents");

        // Verify content relevance
        let found_content = search_result
            .iter()
            .any(|r| r.content.to_lowercase().contains("memory"));
        assert!(
            found_content,
            "Search results should contain memory-related content"
        );
    }

    #[tokio::test]
    async fn test_search_with_file_filter() {
        let (handle, _temp_dir) = create_test_rag_actor().await;

        // Ingest two different documents
        let doc1 = "Python is a high-level programming language known for its simplicity.";
        let doc2 = "JavaScript is the language of the web browser.";

        timeout(
            Duration::from_secs(30),
            handle.ingest(doc1.to_string(), Some("file:python-doc".to_string())),
        )
        .await
        .expect("Ingest timeout")
        .expect("Ingest doc1 failed");

        timeout(
            Duration::from_secs(30),
            handle.ingest(doc2.to_string(), Some("file:js-doc".to_string())),
        )
        .await
        .expect("Ingest timeout")
        .expect("Ingest doc2 failed");

        // Search with filter for python doc only
        let result = timeout(
            Duration::from_secs(10),
            handle.search_with_filters(
                "programming language".to_string(),
                vec!["python-doc".to_string()],
            ),
        )
        .await
        .expect("Search timeout")
        .expect("Search failed");

        // Should only return Python-related content
        for doc in &result {
            if let Some(meta) = &doc.metadata {
                assert!(
                    meta.contains("python"),
                    "Filtered results should only contain python docs"
                );
            }
        }
    }

    #[tokio::test]
    async fn test_delete_document_vectors() {
        let (handle, _temp_dir) = create_test_rag_actor().await;

        let file_id = "delete-test-doc";
        let content = "This document will be deleted from the vector database.";

        // Ingest document
        timeout(
            Duration::from_secs(30),
            handle.ingest(content.to_string(), Some(format!("file:{}", file_id))),
        )
        .await
        .expect("Ingest timeout")
        .expect("Ingest failed");

        // Verify it was ingested
        let search_before = timeout(
            Duration::from_secs(10),
            handle.search_with_filters("deleted vector database".to_string(), vec![]),
        )
        .await
        .expect("Search timeout")
        .expect("Search failed");

        assert!(
            !search_before.is_empty(),
            "Document should be found before deletion"
        );

        // Delete the document
        let delete_result = timeout(
            Duration::from_secs(10),
            handle.delete_for_file(file_id.to_string()),
        )
        .await
        .expect("Delete timeout");

        assert!(delete_result.is_ok(), "Delete should succeed");

        // Verify it was deleted (search with filter should return empty)
        let search_after = timeout(
            Duration::from_secs(10),
            handle.search_with_filters(
                "deleted vector database".to_string(),
                vec![file_id.to_string()],
            ),
        )
        .await
        .expect("Search timeout")
        .expect("Search failed");

        assert!(
            search_after.is_empty(),
            "Document should not be found after deletion"
        );
    }

    #[tokio::test]
    async fn test_ingest_empty_content() {
        let (handle, _temp_dir) = create_test_rag_actor().await;

        let result = timeout(Duration::from_secs(10), handle.ingest("".to_string(), None))
            .await
            .expect("Ingest timeout")
            .expect("Ingest failed");

        // Should handle empty content gracefully
        assert!(
            result.contains("No valid chunks") || result.contains("Ingested 0"),
            "Should handle empty content: {}",
            result
        );
    }

    #[tokio::test]
    async fn test_ingest_large_document_chunking() {
        let (handle, _temp_dir) = create_test_rag_actor().await;

        // Create a large document with newlines to trigger chunking
        // The chunking algorithm splits on '\n' and chunks at ~512 chars
        let large_content = (0..50)
            .map(|i| {
                format!(
                    "This is paragraph number {} of the comprehensive test document.\n\
                     It contains extensive information about topic {} which includes various details.\n\
                     The purpose of this paragraph is to ensure chunking works properly.\n\
                     Additional context about subject {} is provided here for testing.\n",
                    i,
                    i % 10,
                    i % 5
                )
            })
            .collect::<String>();

        let result = timeout(
            Duration::from_secs(60),
            handle.ingest(large_content, Some("file:large-doc".to_string())),
        )
        .await
        .expect("Ingest timeout")
        .expect("Ingest failed");

        // Should have created chunks
        assert!(
            result.contains("Ingested"),
            "Should successfully ingest large document: {}",
            result
        );

        // Extract chunk count - for large docs we expect multiple chunks
        // But the exact number depends on chunking algorithm, so just verify success
        info!("Large document ingest result: {}", result);
    }

    #[tokio::test]
    async fn test_search_empty_database() {
        let (handle, _temp_dir) = create_test_rag_actor().await;

        // Search on empty database should return empty results, not error
        let result = timeout(
            Duration::from_secs(10),
            handle.search_with_filters("any query".to_string(), vec![]),
        )
        .await
        .expect("Search timeout")
        .expect("Search should succeed on empty DB");

        assert!(result.is_empty(), "Empty database should return no results");
    }

    #[tokio::test]
    async fn test_semantic_search_quality() {
        let (handle, _temp_dir) = create_test_rag_actor().await;

        // Ingest documents about different topics
        let docs = vec![
            ("file:cooking", "Recipes for delicious pasta dishes. How to cook Italian food. Ingredients include tomatoes and basil."),
            ("file:programming", "Software development best practices. Code review guidelines. Writing clean and maintainable code."),
            ("file:gardening", "Growing vegetables in your backyard. Planting tomatoes and herbs. Organic gardening tips."),
        ];

        for (meta, content) in docs {
            timeout(
                Duration::from_secs(30),
                handle.ingest(content.to_string(), Some(meta.to_string())),
            )
            .await
            .expect("Ingest timeout")
            .expect("Ingest failed");
        }

        // Search for programming-related content
        let result = timeout(
            Duration::from_secs(10),
            handle.search_with_filters("software code development".to_string(), vec![]),
        )
        .await
        .expect("Search timeout")
        .expect("Search failed");

        assert!(!result.is_empty(), "Should find results");

        // The top result should be programming-related
        let top_result = &result[0];
        let is_programming_related = top_result.content.to_lowercase().contains("code")
            || top_result.content.to_lowercase().contains("software")
            || top_result.content.to_lowercase().contains("development");

        assert!(
            is_programming_related,
            "Top result should be programming-related, got: {}",
            top_result.content
        );
    }

    #[tokio::test]
    async fn test_multiple_ingests_same_file() {
        let (handle, _temp_dir) = create_test_rag_actor().await;

        let file_id = "multi-ingest-doc";

        // Ingest same file multiple times (simulating updates)
        for i in 0..3 {
            let content = format!("Version {} of the document content.", i);
            timeout(
                Duration::from_secs(30),
                handle.ingest(content, Some(format!("file:{}", file_id))),
            )
            .await
            .expect("Ingest timeout")
            .expect("Ingest failed");
        }

        // Search should return results (all versions are stored)
        let result = timeout(
            Duration::from_secs(10),
            handle.search_with_filters("Version document".to_string(), vec![file_id.to_string()]),
        )
        .await
        .expect("Search timeout")
        .expect("Search failed");

        assert!(!result.is_empty(), "Should find ingested documents");
    }
}
