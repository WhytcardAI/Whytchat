use serde::{Deserialize, Serialize};
use sqlx::types::Json;
use sqlx::FromRow;
use validator::Validate;

/// Represents the configuration for an AI model within a session.
#[derive(Debug, Serialize, Deserialize, Clone, Validate)]
pub struct ModelConfig {
    /// The identifier for the model to be used (e.g., file name or API identifier).
    #[validate(length(min = 1))]
    pub model_id: String,
    /// Controls the creativity of the model's responses. Value between 0.0 and 2.0.
    #[validate(range(min = 0.0, max = 2.0))]
    pub temperature: f32,
    /// The system-level instructions provided to the model for context.
    #[validate(length(min = 1))]
    pub system_prompt: String,
}

/// Represents a chat session.
#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Session {
    /// The unique identifier for the session (UUID).
    pub id: String,
    /// The user-defined title of the session.
    pub title: String,
    /// Unix timestamp of when the session was created.
    pub created_at: i64,
    /// The model configuration associated with this session.
    pub model_config: Json<ModelConfig>,
}

/// Represents a single message within a chat session.
#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Message {
    /// The unique identifier for the message.
    pub id: i64,
    /// The ID of the session this message belongs to.
    pub session_id: String,
    /// The role of the message sender (e.g., "user", "assistant").
    pub role: String,
    /// The text content of the message.
    pub content: String,
    /// Unix timestamp of when the message was created.
    pub created_at: i64,
}

/// Represents a file associated with a chat session.
#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct SessionFile {
    /// The unique identifier for the file association record.
    pub id: String,
    /// The ID of the session this file is associated with.
    pub session_id: String,
    /// The path to the file on the filesystem.
    pub file_path: String,
    /// The type of the file (e.g., MIME type).
    pub file_type: String,
    /// Unix timestamp of when the file was associated with the session.
    pub added_at: i64,
}
