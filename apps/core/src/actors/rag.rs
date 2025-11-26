use crate::actors::messages::{AppError, ActorError, RagMessage};
use crate::actors::traits::RagActor;
use crate::fs_manager::PortablePathManager;
use arrow::array::{
    Array, FixedSizeListBuilder, Float32Builder, RecordBatch, RecordBatchIterator, StringArray,
    StringBuilder,
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
use tracing::{error, info, warn};
use lru::LruCache;
use sqlx::sqlite::SqlitePool;
use std::num::NonZeroUsize;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::{mpsc, oneshot};

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
    async fn ingest(
        &self,
        content: String,
        metadata: Option<String>,
    ) -> Result<String, AppError> {
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
        Ok(recv.await
            .map_err(|_| AppError::Actor(ActorError::Internal("RAG Actor failed to respond".to_string())))??)
    }

    async fn search_with_filters(
        &self,
        query: String,
        file_ids: Vec<String>,
    ) -> Result<Vec<String>, AppError> {
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
        Ok(recv.await
            .map_err(|_| AppError::Actor(ActorError::Internal("RAG Actor failed to respond".to_string())))??)
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
    pool: Option<SqlitePool>,
}

impl RagActorRunner {
    fn new(
        receiver: mpsc::Receiver<RagMessage>,
        db_path_override: Option<PathBuf>,
        pool: Option<SqlitePool>,
    ) -> Self {
        Self {
            receiver,
            embedding_model: None,
            embedding_cache: LruCache::new(NonZeroUsize::new(1000).unwrap()),
            db_connection: None,
            table_name: "knowledge_base".to_string(),
            db_path_override,
            pool,
        }
    }

    async fn run(mut self) {
        info!("RagActor started");

        // Initialize FastEmbed with local cache directory
        let embeddings_dir = PortablePathManager::models_dir().join("embeddings");
        let mut options = InitOptions::new(EmbeddingModel::AllMiniLML6V2);
        options.show_download_progress = false;
        options.cache_dir = embeddings_dir;

        match TextEmbedding::try_new(options) {
            Ok(model) => {
                info!("Embedding model loaded successfully");
                self.embedding_model = Some(model);
            }
            Err(e) => error!("Failed to load embedding model: {}", e),
        }

        // Initialize LanceDB
        let db_path = self
            .db_path_override
            .clone()
            .unwrap_or_else(PortablePathManager::vectors_dir);

        // Ensure directory exists if it's a custom path (PortablePathManager handles the default one)
        if self.db_path_override.is_some() {
            if let Some(parent) = db_path.parent() {
                if let Err(e) = std::fs::create_dir_all(parent) {
                    error!("Failed to create database directory at {:?}: {}", parent, e);
                }
            }
        }

        match connect(db_path.to_str().unwrap()).execute().await {
            Ok(conn) => {
                info!("Connected to LanceDB at {:?}", db_path);
                self.db_connection = Some(conn);
            }
            Err(e) => error!("Failed to connect to LanceDB: {}", e),
        }

        while let Some(msg) = self.receiver.recv().await {
            self.handle_message(msg).await;
        }
        info!("RagActor stopped");
    }

    async fn handle_message(&mut self, msg: RagMessage) {
        match msg {
            RagMessage::Ingest {
                content,
                metadata,
                responder,
            } => {
                let result = self.ingest_document(content, metadata).await;
                if let Err(_) = responder.send(result.map_err(AppError::from)) {
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
                if let Err(_) = responder.send(result.map_err(AppError::from)) {
                    warn!("Failed to send search response (channel closed)");
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

        // 1. Simple Chunking with overlap for better context
        let chunks: Vec<String> = content
            .split('\n')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty() && s.len() > 20)
            .collect();

        if chunks.is_empty() {
            return Ok("No valid chunks to ingest".to_string());
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
    ) -> Result<Vec<String>, ActorError> {
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

            let content_array =
                content_col
                    .as_any()
                    .downcast_ref::<StringArray>()
                    .ok_or(ActorError::RagError(
                        "Failed to downcast content column".to_string(),
                    ))?;

            for i in 0..content_array.len() {
                if !content_array.is_null(i) {
                    let text = content_array.value(i);
                    documents.push(text.to_string());
                }
            }
        }

        Ok(documents)
    }
}

