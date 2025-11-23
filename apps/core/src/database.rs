use crate::encryption;
use crate::fs_manager::PortablePathManager;
use crate::models::{Message, ModelConfig, Session, SessionFile};
use chrono::Utc;
use tracing::info;
use serde::{Deserialize, Serialize};
use sqlx::sqlite::{SqliteConnectOptions, SqlitePool, SqlitePoolOptions};
use sqlx::types::Json;
use std::str::FromStr;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug)]
struct EncryptedConfigWrapper {
    ciphertext: String,
}

#[derive(sqlx::FromRow)]
struct SessionRow {
    id: String,
    title: String,
    created_at: i64,
    model_config: Json<EncryptedConfigWrapper>,
}

impl SessionRow {
    fn into_session(self) -> Result<Session, String> {
        let decrypted_bytes = encryption::decrypt(&self.model_config.0.ciphertext)?;
        let model_config: ModelConfig = serde_json::from_slice(&decrypted_bytes)
            .map_err(|e| format!("Failed to deserialize model config: {}", e))?;

        Ok(Session {
            id: self.id,
            title: self.title,
            created_at: self.created_at,
            model_config: Json(model_config),
        })
    }
}

/// Initializes the SQLite database connection pool.
///
/// This function sets up the database by:
/// 1. Determining the path to the SQLite file using `PortablePathManager`.
/// 2. Creating the database file if it doesn't exist.
/// 3. Establishing a connection pool with a maximum of 5 connections.
/// 4. Running database migrations to ensure the schema is up to date.
///
/// # Returns
///
/// A `Result` containing the `SqlitePool` on success, or an `sqlx::Error` on failure.
pub async fn init_db() -> Result<SqlitePool, sqlx::Error> {
    let db_path = PortablePathManager::db_dir().join("whytchat.sqlite");
    let db_url = format!("sqlite://{}", db_path.to_string_lossy());

    info!("Initializing database at: {}", db_url);

    let options = SqliteConnectOptions::from_str(&db_url)?.create_if_missing(true);

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(options)
        .await?;

    // Run migrations automatically
    sqlx::migrate!("./migrations").run(&pool).await?;

    info!("Database initialized and migrations applied.");

    Ok(pool)
}

// --- Sessions CRUD ---

/// Creates a new session in the database.
///
/// # Arguments
///
/// * `pool` - A reference to the `SqlitePool`.
/// * `title` - The title of the new session.
/// * `model_config` - The `ModelConfig` associated with the session. The configuration
///   will be encrypted before being stored in the database.
///
/// # Returns
///
/// A `Result` containing the newly created `Session` on success, or an `sqlx::Error` on failure.
pub async fn create_session(
    pool: &SqlitePool,
    title: String,
    model_config: ModelConfig,
) -> Result<Session, sqlx::Error> {
    let id = Uuid::new_v4().to_string();
    let created_at = Utc::now().timestamp();

    // Encrypt model_config
    let config_bytes = serde_json::to_vec(&model_config).map_err(|e| sqlx::Error::Protocol(e.to_string().into()))?;
    let ciphertext = encryption::encrypt(&config_bytes).map_err(|e| sqlx::Error::Protocol(e.into()))?;
    let wrapper = EncryptedConfigWrapper { ciphertext };
    let config_json = Json(wrapper);

    // We still return the original session with cleartext config to the caller,
    // but we save the encrypted version.
    let _ = sqlx::query(
        r#"
        INSERT INTO sessions (id, title, created_at, model_config)
        VALUES (?, ?, ?, ?)
        "#,
    )
    .bind(&id)
    .bind(&title)
    .bind(created_at)
    .bind(config_json)
    .execute(pool)
    .await?;

    Ok(Session {
        id,
        title,
        created_at,
        model_config: Json(model_config),
    })
}

/// Retrieves a single session by its ID.
///
/// The model configuration is decrypted before the session is returned.
///
/// # Arguments
///
/// * `pool` - A reference to the `SqlitePool`.
/// * `id` - The ID of the session to retrieve.
///
/// # Returns
///
/// A `Result` containing the `Session` on success, or an `sqlx::Error` if not found or on failure.
#[allow(dead_code)]
pub async fn get_session(pool: &SqlitePool, id: &str) -> Result<Session, sqlx::Error> {
    let row = sqlx::query_as::<_, SessionRow>(
        r#"
        SELECT id, title, created_at, model_config as "model_config: Json<EncryptedConfigWrapper>"
        FROM sessions
        WHERE id = ?
        "#,
    )
    .bind(id)
    .fetch_one(pool)
    .await?;

    row.into_session().map_err(|e| sqlx::Error::Protocol(e.into()))
}

/// Lists all sessions, ordered by creation date (descending).
///
/// The model configuration for each session is decrypted.
///
/// # Arguments
///
/// * `pool` - A reference to the `SqlitePool`.
///
/// # Returns
///
/// A `Result` containing a `Vec<Session>` on success, or an `sqlx::Error` on failure.
pub async fn list_sessions(pool: &SqlitePool) -> Result<Vec<Session>, sqlx::Error> {
    let rows = sqlx::query_as::<_, SessionRow>(
        r#"
        SELECT id, title, created_at, model_config as "model_config: Json<EncryptedConfigWrapper>"
        FROM sessions
        ORDER BY created_at DESC
        "#,
    )
    .fetch_all(pool)
    .await?;

    rows.into_iter()
        .map(|row| row.into_session().map_err(|e| sqlx::Error::Protocol(e.into())))
        .collect()
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
    
    // Encrypt new config
    let config_bytes = serde_json::to_vec(&new_config).map_err(|e| sqlx::Error::Protocol(e.to_string().into()))?;
    let ciphertext = encryption::encrypt(&config_bytes).map_err(|e| sqlx::Error::Protocol(e.into()))?;
    let wrapper = EncryptedConfigWrapper { ciphertext };
    let config_json = Json(wrapper);

    sqlx::query(
        r#"
        UPDATE sessions
        SET title = ?, model_config = ?
        WHERE id = ?
        "#,
    )
    .bind(&new_title)
    .bind(config_json)
    .bind(id)
    .execute(pool)
    .await?;

    Ok(Session {
        id: id.to_string(),
        title: new_title,
        created_at: current_session.created_at,
        model_config: Json(new_config),
    })
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

#[allow(dead_code)]
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