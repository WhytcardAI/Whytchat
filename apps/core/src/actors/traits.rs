use crate::actors::messages::{AppError};
use async_trait::async_trait;
use tokio::sync::mpsc;

/// Defines the public interface for an LLM (Large Language Model) actor.
///
/// This trait abstracts the specific implementation of the LLM, allowing for different
/// backends (e.g., local llama.cpp, remote API) to be used interchangeably.
#[async_trait]
pub trait LlmActor: Send + Sync + 'static {
    /// Generates a complete text response based on a prompt and optional parameters.
    async fn generate_with_params(&self, prompt: String, system_prompt: Option<String>, temperature: Option<f32>) -> Result<String, AppError>;

    /// Generates a streaming response, sending chunks of text as they are produced.
    async fn stream_generate_with_params(
        &self,
        prompt: String,
        system_prompt: Option<String>,
        temperature: Option<f32>,
        chunk_sender: mpsc::Sender<Result<String, AppError>>,
    ) -> Result<(), AppError>;
}

/// Defines the public interface for a RAG (Retrieval-Augmented Generation) actor.
///
/// This trait abstracts the logic for managing and querying a knowledge base.
#[async_trait]
pub trait RagActor: Send + Sync + 'static {
    /// Ingests new content into the knowledge base.
    async fn ingest(&self, content: String, metadata: Option<String>) -> Result<String, AppError>;

    /// Searches the knowledge base for content relevant to a query.
    async fn search_with_filters(&self, query: String, file_ids: Vec<String>) -> Result<Vec<String>, AppError>;
}
