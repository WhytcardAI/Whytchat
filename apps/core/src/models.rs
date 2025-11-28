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
    /// The type of folder ("session" or "document").
    #[serde(default = "default_folder_type", rename = "type")]
    pub folder_type: String,
}

fn default_folder_color() -> String {
    "#6366f1".to_string()
}

fn default_folder_type() -> String {
    "session".to_string()
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
    /// Optional folder ID for organization.
    #[serde(default)]
    pub folder_id: Option<String>,
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

#[cfg(test)]
mod tests {
    use super::*;
    use validator::Validate;

    // ==================== ModelConfig Tests ====================

    #[test]
    fn test_model_config_default() {
        let config = ModelConfig::default();

        assert_eq!(config.model_id, "default-model.gguf");
        assert!((config.temperature - 0.7).abs() < f32::EPSILON);
        assert_eq!(config.system_prompt, "You are a helpful assistant.");
    }

    #[test]
    fn test_model_config_valid() {
        let config = ModelConfig {
            model_id: "my-model.gguf".to_string(),
            temperature: 0.5,
            system_prompt: "You are a coding assistant.".to_string(),
        };

        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_model_config_invalid_empty_model_id() {
        let config = ModelConfig {
            model_id: "".to_string(),
            temperature: 0.7,
            system_prompt: "Valid prompt".to_string(),
        };

        let result = config.validate();
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.field_errors().contains_key("model_id"));
    }

    #[test]
    fn test_model_config_invalid_empty_system_prompt() {
        let config = ModelConfig {
            model_id: "model.gguf".to_string(),
            temperature: 0.7,
            system_prompt: "".to_string(),
        };

        let result = config.validate();
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.field_errors().contains_key("system_prompt"));
    }

    #[test]
    fn test_model_config_temperature_min_boundary() {
        let config = ModelConfig {
            model_id: "model.gguf".to_string(),
            temperature: 0.0, // Minimum valid
            system_prompt: "Valid prompt".to_string(),
        };

        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_model_config_temperature_max_boundary() {
        let config = ModelConfig {
            model_id: "model.gguf".to_string(),
            temperature: 2.0, // Maximum valid
            system_prompt: "Valid prompt".to_string(),
        };

        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_model_config_temperature_below_min() {
        let config = ModelConfig {
            model_id: "model.gguf".to_string(),
            temperature: -0.1, // Invalid: below 0
            system_prompt: "Valid prompt".to_string(),
        };

        let result = config.validate();
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.field_errors().contains_key("temperature"));
    }

    #[test]
    fn test_model_config_temperature_above_max() {
        let config = ModelConfig {
            model_id: "model.gguf".to_string(),
            temperature: 2.1, // Invalid: above 2.0
            system_prompt: "Valid prompt".to_string(),
        };

        let result = config.validate();
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.field_errors().contains_key("temperature"));
    }

    #[test]
    fn test_model_config_serialization() {
        let config = ModelConfig {
            model_id: "test-model.gguf".to_string(),
            temperature: 0.8,
            system_prompt: "You are helpful.".to_string(),
        };

        let json = serde_json::to_string(&config).expect("Serialization failed");
        assert!(json.contains("test-model.gguf"));
        assert!(json.contains("0.8"));

        let deserialized: ModelConfig =
            serde_json::from_str(&json).expect("Deserialization failed");
        assert_eq!(deserialized.model_id, "test-model.gguf");
    }

    // ==================== Session Tests ====================

    #[test]
    fn test_session_serialization() {
        let session = Session {
            id: "test-session-123".to_string(),
            title: "Test Session".to_string(),
            created_at: 1700000000,
            model_config: Json(ModelConfig::default()),
            is_favorite: true,
            folder_id: Some("folder-456".to_string()),
            sort_order: 1,
            updated_at: 1700000100,
        };

        let json = serde_json::to_string(&session).expect("Serialization failed");
        assert!(json.contains("test-session-123"));
        assert!(json.contains("Test Session"));
        assert!(json.contains("\"is_favorite\":true"));
        assert!(json.contains("folder-456"));
    }

    #[test]
    fn test_session_deserialization_with_defaults() {
        let json = r#"{
            "id": "sess-1",
            "title": "Minimal Session",
            "created_at": 1700000000,
            "model_config": {
                "model_id": "model.gguf",
                "temperature": 0.7,
                "system_prompt": "Hello"
            }
        }"#;

        let session: Session = serde_json::from_str(json).expect("Deserialization failed");
        assert_eq!(session.id, "sess-1");
        assert!(!session.is_favorite); // Default
        assert!(session.folder_id.is_none()); // Default
        assert_eq!(session.sort_order, 0); // Default
        assert_eq!(session.updated_at, 0); // Default
    }

    // ==================== Folder Tests ====================

    #[test]
    fn test_folder_default_color() {
        assert_eq!(default_folder_color(), "#6366f1");
    }

    #[test]
    fn test_folder_default_type() {
        assert_eq!(default_folder_type(), "session");
    }

    #[test]
    fn test_folder_serialization() {
        let folder = Folder {
            id: "folder-123".to_string(),
            name: "My Folder".to_string(),
            color: "#ff0000".to_string(),
            sort_order: 5,
            created_at: 1700000000,
            folder_type: "document".to_string(),
        };

        let json = serde_json::to_string(&folder).expect("Serialization failed");
        assert!(json.contains("folder-123"));
        assert!(json.contains("My Folder"));
        assert!(json.contains("#ff0000"));
        // Note: folder_type is renamed to "type" in JSON
        assert!(json.contains("\"type\":\"document\""));
    }

    // ==================== Message Tests ====================

    #[test]
    fn test_message_serialization() {
        let message = Message {
            id: 42,
            session_id: "sess-123".to_string(),
            role: "user".to_string(),
            content: "Hello, AI!".to_string(),
            created_at: 1700000000,
        };

        let json = serde_json::to_string(&message).expect("Serialization failed");
        assert!(json.contains("\"id\":42"));
        assert!(json.contains("sess-123"));
        assert!(json.contains("user"));
        assert!(json.contains("Hello, AI!"));
    }

    #[test]
    fn test_message_roles() {
        let user_msg = Message {
            id: 1,
            session_id: "s1".to_string(),
            role: "user".to_string(),
            content: "Question".to_string(),
            created_at: 0,
        };

        let assistant_msg = Message {
            id: 2,
            session_id: "s1".to_string(),
            role: "assistant".to_string(),
            content: "Answer".to_string(),
            created_at: 1,
        };

        assert_eq!(user_msg.role, "user");
        assert_eq!(assistant_msg.role, "assistant");
    }

    // ==================== LibraryFile Tests ====================

    #[test]
    fn test_library_file_serialization() {
        let file = LibraryFile {
            id: "file-123".to_string(),
            name: "document.pdf".to_string(),
            path: "/files/document.pdf".to_string(),
            file_type: "application/pdf".to_string(),
            size: 1024,
            created_at: 1700000000,
            folder_id: None,
        };

        let json = serde_json::to_string(&file).expect("Serialization failed");
        assert!(json.contains("file-123"));
        assert!(json.contains("document.pdf"));
        assert!(json.contains("application/pdf"));
        assert!(json.contains("1024"));
    }

    #[test]
    fn test_library_file_with_folder() {
        let file = LibraryFile {
            id: "file-456".to_string(),
            name: "notes.txt".to_string(),
            path: "/files/notes.txt".to_string(),
            file_type: "text/plain".to_string(),
            size: 256,
            created_at: 1700000000,
            folder_id: Some("folder-789".to_string()),
        };

        assert_eq!(file.folder_id, Some("folder-789".to_string()));
    }

    // ==================== SessionFile Tests ====================

    #[test]
    fn test_session_file_serialization() {
        let session_file = SessionFile {
            id: "file-abc".to_string(),
            session_id: "sess-xyz".to_string(),
            name: "attachment.pdf".to_string(),
            path: "/files/attachment.pdf".to_string(),
            file_type: "application/pdf".to_string(),
            size: 2048,
            attached_at: 1700000000,
        };

        let json = serde_json::to_string(&session_file).expect("Serialization failed");
        assert!(json.contains("file-abc"));
        assert!(json.contains("sess-xyz"));
        assert!(json.contains("attachment.pdf"));
        assert!(json.contains("attached_at"));
    }
}
