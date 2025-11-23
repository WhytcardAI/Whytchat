use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::types::Json;
use sqlx::FromRow;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ModelConfig {
    pub model_id: String,
    pub temperature: f32,
    pub system_prompt: String,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Session {
    pub id: String,
    pub title: String,
    pub created_at: i64,
    pub model_config: Json<ModelConfig>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Message {
    pub id: i64,
    pub session_id: String,
    pub role: String,
    pub content: String,
    pub created_at: i64,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct SessionFile {
    pub id: String,
    pub session_id: String,
    pub file_path: String,
    pub file_type: String,
    pub added_at: i64,
}
