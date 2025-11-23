use crate::fs_manager::PortablePathManager;
use crate::models::{Message, ModelConfig, Session, SessionFile};
use chrono::Utc;
use log::info;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePool, SqlitePoolOptions};
use sqlx::types::Json;
use std::str::FromStr;
use uuid::Uuid;

pub async fn init_db() -> Result<SqlitePool, sqlx::Error> {
    let db_path = PortablePathManager::db_dir().join("whytchat.sqlite");
    let db_url = format!("sqlite://{}", db_path.to_string_lossy());

    info!("Initializing database at: {}", db_url);

    let options = SqliteConnectOptions::from_str(&db_url)?.create_if_missing(true);

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(options)
        .await?;

    // Run migrations manually for now
    // Note: We are using model_config JSON column instead of separate columns
    // If the table exists with old schema, this might fail or need manual migration.
    // For dev, we'll assume fresh start or compatible schema.
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS sessions (
            id TEXT PRIMARY KEY,
            title TEXT NOT NULL,
            created_at DATETIME NOT NULL,
            model_config JSON NOT NULL
        );
        CREATE TABLE IF NOT EXISTS messages (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            session_id TEXT NOT NULL,
            role TEXT NOT NULL,
            content TEXT NOT NULL,
            created_at DATETIME NOT NULL,
            FOREIGN KEY(session_id) REFERENCES sessions(id)
        );
        CREATE TABLE IF NOT EXISTS session_files (
            id TEXT PRIMARY KEY,
            session_id TEXT NOT NULL,
            file_path TEXT NOT NULL,
            file_type TEXT NOT NULL,
            added_at DATETIME NOT NULL,
            FOREIGN KEY(session_id) REFERENCES sessions(id)
        );
        "#,
    )
    .execute(&pool)
    .await?;

    info!("Database initialized and migrations applied.");

    Ok(pool)
}

// --- Sessions CRUD ---

pub async fn create_session(
    pool: &SqlitePool,
    title: String,
    model_config: ModelConfig,
) -> Result<Session, sqlx::Error> {
    let id = Uuid::new_v4().to_string();
    let created_at = Utc::now().timestamp();
    let config_json = Json(model_config);

    sqlx::query_as::<_, Session>(
        r#"
        INSERT INTO sessions (id, title, created_at, model_config)
        VALUES (?, ?, ?, ?)
        RETURNING id, title, created_at, model_config as "model_config: Json<ModelConfig>"
        "#,
    )
    .bind(&id)
    .bind(&title)
    .bind(created_at)
    .bind(config_json)
    .fetch_one(pool)
    .await
}

pub async fn get_session(pool: &SqlitePool, id: &str) -> Result<Session, sqlx::Error> {
    sqlx::query_as::<_, Session>(
        r#"
        SELECT id, title, created_at, model_config as "model_config: Json<ModelConfig>"
        FROM sessions
        WHERE id = ?
        "#,
    )
    .bind(id)
    .fetch_one(pool)
    .await
}

pub async fn get_all_sessions(pool: &SqlitePool) -> Result<Vec<Session>, sqlx::Error> {
    sqlx::query_as::<_, Session>(
        r#"
        SELECT id, title, created_at, model_config as "model_config: Json<ModelConfig>"
        FROM sessions
        ORDER BY created_at DESC
        "#,
    )
    .fetch_all(pool)
    .await
}

pub async fn update_session(
    pool: &SqlitePool,
    id: &str,
    title: Option<String>,
    model_config: Option<ModelConfig>,
) -> Result<Session, sqlx::Error> {
    // Get the current session to preserve unchanged fields
    let current_session = get_session(pool, id).await?;
    
    let new_title = title.unwrap_or(current_session.title);
    let new_config = model_config.unwrap_or(current_session.model_config.0);
    let config_json = Json(new_config);

    sqlx::query_as::<_, Session>(
        r#"
        UPDATE sessions
        SET title = ?, model_config = ?
        WHERE id = ?
        RETURNING id, title, created_at, model_config as "model_config: Json<ModelConfig>"
        "#,
    )
    .bind(&new_title)
    .bind(config_json)
    .bind(id)
    .fetch_one(pool)
    .await
}

// --- Messages CRUD ---

pub async fn add_message(
    pool: &SqlitePool,
    session_id: &str,
    role: &str,
    content: &str,
) -> Result<Message, sqlx::Error> {
    let created_at = Utc::now().timestamp();

    sqlx::query_as::<_, Message>(
        r#"
        INSERT INTO messages (session_id, role, content, created_at)
        VALUES (?, ?, ?, ?)
        RETURNING id, session_id, role, content, created_at
        "#,
    )
    .bind(session_id)
    .bind(role)
    .bind(content)
    .bind(created_at)
    .fetch_one(pool)
    .await
}

pub async fn get_session_messages(
    pool: &SqlitePool,
    session_id: &str,
) -> Result<Vec<Message>, sqlx::Error> {
    sqlx::query_as::<_, Message>(
        r#"
        SELECT id, session_id, role, content, created_at
        FROM messages
        WHERE session_id = ?
        ORDER BY created_at ASC
        "#,
    )
    .bind(session_id)
    .fetch_all(pool)
    .await
}

// --- Session Files CRUD ---

pub async fn add_session_file(
    pool: &SqlitePool,
    session_id: &str,
    file_path: &str,
    file_type: &str,
) -> Result<SessionFile, sqlx::Error> {
    let id = Uuid::new_v4().to_string();
    let added_at = Utc::now().timestamp();

    sqlx::query_as::<_, SessionFile>(
        r#"
        INSERT INTO session_files (id, session_id, file_path, file_type, added_at)
        VALUES (?, ?, ?, ?, ?)
        RETURNING id, session_id, file_path, file_type, added_at
        "#,
    )
    .bind(&id)
    .bind(session_id)
    .bind(file_path)
    .bind(file_type)
    .bind(added_at)
    .fetch_one(pool)
    .await
}

pub async fn get_session_files(
    pool: &SqlitePool,
    session_id: &str,
) -> Result<Vec<SessionFile>, sqlx::Error> {
    sqlx::query_as::<_, SessionFile>(
        r#"
        SELECT id, session_id, file_path, file_type, added_at
        FROM session_files
        WHERE session_id = ?
        ORDER BY added_at ASC
        "#,
    )
    .bind(session_id)
    .fetch_all(pool)
    .await
}
