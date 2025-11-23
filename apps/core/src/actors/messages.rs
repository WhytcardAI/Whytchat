use serde::Serialize;
use tauri::Window;
use tokio::sync::oneshot;

// Global errors for the actor system
#[derive(Debug, thiserror::Error, Serialize)]
pub enum ActorError {
    #[error("LLM request failed: {0}")]
    #[allow(dead_code)]
    LlmError(String),
    #[error("RAG request failed: {0}")]
    RagError(String),
    #[error("Internal system error: {0}")]
    Internal(String),
}

// --- LLM Messages ---
#[derive(Debug)]
pub enum LlmMessage {
    #[allow(dead_code)]
    Generate {
        prompt: String,
        system_prompt: Option<String>,
        temperature: Option<f32>,
        responder: oneshot::Sender<Result<String, ActorError>>,
    },
    GenerateWithParams {
        prompt: String,
        system_prompt: Option<String>,
        temperature: Option<f32>,
        responder: oneshot::Sender<Result<String, ActorError>>,
    },
    #[allow(dead_code)]
    StreamGenerate {
        prompt: String,
        system_prompt: Option<String>,
        temperature: Option<f32>,
        // Sender for chunks (Ok(token)) or Error
        // We use mpsc for streaming multiple chunks
        chunk_sender: tokio::sync::mpsc::Sender<Result<String, ActorError>>,
        // Final signal when done (optional, but good for synchronization)
        responder: oneshot::Sender<Result<(), ActorError>>,
    },
    StreamGenerateWithParams {
        prompt: String,
        system_prompt: Option<String>,
        temperature: Option<f32>,
        chunk_sender: tokio::sync::mpsc::Sender<Result<String, ActorError>>,
        responder: oneshot::Sender<Result<(), ActorError>>,
    },
}

// --- RAG Messages ---
#[derive(Debug)]
pub enum RagMessage {
    Ingest {
        content: String,
        metadata: Option<String>, // JSON string
        responder: oneshot::Sender<Result<String, ActorError>>,
    },
    Search {
        query: String,
        session_id: Option<String>,
        limit: usize,
        responder: oneshot::Sender<Result<Vec<String>, ActorError>>, // Returns list of chunks
    },
}

// --- Supervisor Messages (Routing) ---
#[derive(Debug)]
#[allow(clippy::large_enum_variant)]
pub enum SupervisorMessage {
    ProcessUserMessage {
        session_id: String,
        content: String,
        window: Option<Window>, // For emitting thinking events
        responder: oneshot::Sender<Result<String, ActorError>>,
    },
    IngestContent {
        content: String,
        metadata: Option<String>,
        responder: oneshot::Sender<Result<String, ActorError>>,
    },
    #[allow(dead_code)]
    Shutdown,
}
