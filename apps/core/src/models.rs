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

impl Default for ModelConfig {
    fn default() -> Self {
        Self {
            model_id: "default-model.gguf".to_string(),
            temperature: 0.7,
            system_prompt: "You are a helpful assistant.".to_string(),
        }
    }
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
    /// Whether this session is marked as favorite (pinned).
    #[serde(default)]
    pub is_favorite: bool,
    /// Optional folder ID for organization.
    #[serde(default)]
    pub folder_id: Option<String>,
    /// Sort order within folder (lower = higher priority).
    #[serde(default)]
    pub sort_order: i32,
    /// Unix timestamp of when the session was last updated.
    #[serde(default)]
    pub updated_at: i64,
}

/// Represents a folder for organizing sessions.
#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Folder {
    /// The unique identifier for the folder (UUID).
    pub id: String,
    /// The user-defined name of the folder.
    pub name: String,
    /// The color for the folder (hex format).
    #[serde(default = "default_folder_color")]
    pub color: String,
    /// Sort order for folders.
    #[serde(default)]
    pub sort_order: i32,
    /// Unix timestamp of when the folder was created.
    pub created_at: i64,
}

fn default_folder_color() -> String {
    "#6366f1".to_string()
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

/// Represents a file in the global library.
#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct LibraryFile {
    /// The unique identifier for the file.
    pub id: String,
    /// The original name of the file.
    pub name: String,
    /// The relative path to the file in storage.
    pub path: String,
    /// The MIME type or extension.
    pub file_type: String,
    /// File size in bytes.
    #[serde(default)]
    pub size: i64,
    /// Unix timestamp of creation.
    pub created_at: i64,
}

/// Represents a file associated with a chat session (Joined View).
#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct SessionFile {
    /// The unique identifier for the file (from library).
    pub id: String,
    /// The ID of the session.
    pub session_id: String,
    /// The name of the file.
    pub name: String,
    /// The path to the file.
    pub path: String,
    /// The type of the file.
    pub file_type: String,
    /// File size.
    pub size: i64,
    /// Unix timestamp of when the file was attached to this session.
    pub attached_at: i64,
}
