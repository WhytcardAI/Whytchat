use serde::Serialize;
use tauri::Window;
use tokio::sync::oneshot;

/// Defines errors that can occur within the actor system.
#[derive(Debug, thiserror::Error, Serialize, Clone)]
pub enum ActorError {
    /// An error originating from the LLM actor.
    #[error("LLM request failed: {0}")]
    #[allow(dead_code)]
    LlmError(String),
    /// An error originating from the RAG actor.
    #[error("RAG request failed: {0}")]
    RagError(String),
    /// A generic internal error within an actor.
    #[error("Internal system error: {0}")]
    Internal(String),
    /// An error indicating that an actor operation timed out.
    #[error("Operation timed out: {0}")]
    Timeout(String),
}

impl From<tokio::time::error::Elapsed> for ActorError {
    fn from(err: tokio::time::error::Elapsed) -> Self {
        ActorError::Timeout(format!("Actor operation timed out: {}", err))
    }
}

// Re-export AppError for convenience
pub use crate::error::AppError;

/// Messages that can be sent to the `LlmActor`.
#[derive(Debug)]
pub enum LlmMessage {
    /// A request to generate a complete text response.
    #[allow(dead_code)]
    Generate {
        prompt: String,
        system_prompt: Option<String>,
        temperature: Option<f32>,
        /// A channel to send the final `String` result back.
        responder: oneshot::Sender<Result<String, AppError>>,
    },
    /// A request to generate a complete text response with specific parameters.
    GenerateWithParams {
        prompt: String,
        system_prompt: Option<String>,
        temperature: Option<f32>,
        /// A channel to send the final `String` result back.
        responder: oneshot::Sender<Result<String, AppError>>,
    },
    /// A request to generate a streaming text response.
    #[allow(dead_code)]
    StreamGenerate {
        prompt: String,
        system_prompt: Option<String>,
        temperature: Option<f32>,
        /// A channel to send each generated token (chunk) back.
        chunk_sender: tokio::sync::mpsc::Sender<Result<String, AppError>>,
        /// A channel to signal completion or an error for the whole stream.
        responder: oneshot::Sender<Result<(), AppError>>,
    },
    /// A request to generate a streaming text response with specific parameters.
    StreamGenerateWithParams {
        prompt: String,
        system_prompt: Option<String>,
        temperature: Option<f32>,
        /// A channel to send each generated token (chunk) back.
        chunk_sender: tokio::sync::mpsc::Sender<Result<String, AppError>>,
        /// A channel to signal completion or an error for the whole stream.
        responder: oneshot::Sender<Result<(), AppError>>,
    },
}

/// Messages that can be sent to the `RagActor`.
#[derive(Debug)]
pub enum RagMessage {
    /// A request to ingest content into the knowledge base.
    Ingest {
        content: String,
        /// Optional metadata (as a JSON string) to associate with the content.
        metadata: Option<String>,
        /// A channel to send the result (e.g., a confirmation message) back.
        responder: oneshot::Sender<Result<String, AppError>>,
    },
    /// A request to search the knowledge base.
    Search {
        query: String,
        /// A list of file IDs to filter the search.
        file_ids: Vec<String>,
        /// The maximum number of results to return.
        limit: usize,
        /// A channel to send the search results back.
        responder: oneshot::Sender<Result<Vec<SearchResult>, AppError>>,
    },
    /// A request to delete all vectors associated with a specific file.
    DeleteForFile {
        file_id: String,
        responder: oneshot::Sender<Result<(), AppError>>,
    },
}

#[derive(Debug, Clone, Serialize)]
pub struct SearchResult {
    pub content: String,
    pub metadata: Option<String>,
    pub score: f32,
}

/// Messages that can be sent to the `SupervisorActor`.
#[derive(Debug)]
#[allow(clippy::large_enum_variant)]
pub enum SupervisorMessage {
    /// A request to process a user's message from a specific session.
    ProcessUserMessage {
        session_id: String,
        content: String,
        /// The Tauri window, used for emitting events back to the frontend.
        window: Option<Window>,
        /// A channel to send the final assistant response back.
        responder: oneshot::Sender<Result<String, AppError>>,
    },
    /// A request to ingest content, which the supervisor will delegate to the RAG actor.
    IngestContent {
        content: String,
        metadata: Option<String>,
        responder: oneshot::Sender<Result<String, AppError>>,
    },
    /// A request to reindex a specific file (delete vectors + ingest).
    ReindexFile {
        file_id: String,
        content: String,
        responder: oneshot::Sender<Result<String, AppError>>,
    },
    /// A command to shut down the supervisor and its child actors.
    #[allow(dead_code)]
    Shutdown,
}
