use crate::encryption;
use crate::fs_manager::PortablePathManager;
use crate::models::{Folder, LibraryFile, Message, ModelConfig, Session, SessionFile};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use sqlx::sqlite::{SqliteConnectOptions, SqlitePool, SqlitePoolOptions};
use sqlx::types::Json;
use std::str::FromStr;
use tracing::info;
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
    is_favorite: bool,
    folder_id: Option<String>,
    sort_order: i32,
    updated_at: i64,
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
            is_favorite: self.is_favorite,
            folder_id: self.folder_id,
            sort_order: self.sort_order,
            updated_at: self.updated_at,
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
    create_session_with_id(pool, &id, title, model_config).await
}

pub async fn create_session_with_id(
    pool: &SqlitePool,
    id: &str,
    title: String,
    model_config: ModelConfig,
) -> Result<Session, sqlx::Error> {
    let created_at = Utc::now().timestamp();
    let updated_at = created_at;

    // Encrypt model_config
    let config_bytes =
        serde_json::to_vec(&model_config).map_err(|e| sqlx::Error::Protocol(e.to_string()))?;
    let ciphertext = encryption::encrypt(&config_bytes).map_err(sqlx::Error::Protocol)?;
    let wrapper = EncryptedConfigWrapper { ciphertext };
    let config_json = Json(wrapper);

    // We still return the original session with cleartext config to the caller,
    // but we save the encrypted version.
    let _ = sqlx::query(
        r#"
        INSERT INTO sessions (id, title, created_at, model_config, is_favorite, folder_id, sort_order, updated_at)
        VALUES (?, ?, ?, ?, 0, NULL, 0, ?)
        "#,
    )
    .bind(id)
    .bind(&title)
    .bind(created_at)
    .bind(config_json)
    .bind(updated_at)
    .execute(pool)
    .await?;

    Ok(Session {
        id: id.to_string(),
        title,
        created_at,
        model_config: Json(model_config),
        is_favorite: false,
        folder_id: None,
        sort_order: 0,
        updated_at,
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
        SELECT id, title, created_at, model_config, is_favorite, folder_id, sort_order, updated_at
        FROM sessions
        WHERE id = ?
        "#,
    )
    .bind(id)
    .fetch_optional(pool)
    .await?;

    match row {
        Some(r) => r.into_session().map_err(sqlx::Error::Protocol),
        None => Err(sqlx::Error::RowNotFound),
    }
}

/// Lists all sessions, ordered by favorites first, then by updated_at (descending).
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
        SELECT id, title, created_at, model_config, is_favorite, folder_id, sort_order, updated_at
        FROM sessions
        ORDER BY is_favorite DESC, updated_at DESC
        "#,
    )
    .fetch_all(pool)
    .await?;

    rows.into_iter()
        .map(|row| row.into_session().map_err(sqlx::Error::Protocol))
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
    let updated_at = Utc::now().timestamp();

    let new_title = title.unwrap_or(current_session.title);
    let new_config = model_config.unwrap_or(current_session.model_config.0);

    // Encrypt new config
    let config_bytes =
        serde_json::to_vec(&new_config).map_err(|e| sqlx::Error::Protocol(e.to_string()))?;
    let ciphertext = encryption::encrypt(&config_bytes).map_err(sqlx::Error::Protocol)?;
    let wrapper = EncryptedConfigWrapper { ciphertext };
    let config_json = Json(wrapper);

    sqlx::query(
        r#"
        UPDATE sessions
        SET title = ?, model_config = ?, updated_at = ?
        WHERE id = ?
        "#,
    )
    .bind(&new_title)
    .bind(config_json)
    .bind(updated_at)
    .bind(id)
    .execute(pool)
    .await?;

    Ok(Session {
        id: id.to_string(),
        title: new_title,
        created_at: current_session.created_at,
        model_config: Json(new_config),
        is_favorite: current_session.is_favorite,
        folder_id: current_session.folder_id,
        sort_order: current_session.sort_order,
        updated_at,
    })
}

/// Toggle favorite status for a session.
pub async fn toggle_session_favorite(pool: &SqlitePool, id: &str) -> Result<bool, sqlx::Error> {
    let current = get_session(pool, id).await?;
    let new_favorite = !current.is_favorite;

    sqlx::query(
        r#"
        UPDATE sessions
        SET is_favorite = ?, updated_at = ?
        WHERE id = ?
        "#,
    )
    .bind(new_favorite)
    .bind(Utc::now().timestamp())
    .bind(id)
    .execute(pool)
    .await?;

    Ok(new_favorite)
}

/// Delete a session and all its messages and files.
pub async fn delete_session(pool: &SqlitePool, id: &str) -> Result<(), sqlx::Error> {
    info!("Request to delete session: {}", id);
    let mut tx = pool.begin().await?;

    // Delete messages first (foreign key constraint)
    sqlx::query("DELETE FROM messages WHERE session_id = ?")
        .bind(id)
        .execute(&mut *tx)
        .await?;

    // Delete session file links (files remain in library)
    sqlx::query("DELETE FROM session_files_link WHERE session_id = ?")
        .bind(id)
        .execute(&mut *tx)
        .await?;

    // Delete the session itself
    sqlx::query("DELETE FROM sessions WHERE id = ?")
        .bind(id)
        .execute(&mut *tx)
        .await?;

    tx.commit().await?;

    info!("Deleted session {} and all associated data", id);
    Ok(())
}

/// Clear all messages from a session without deleting the session.
#[allow(dead_code)]
pub async fn clear_session_messages(
    pool: &SqlitePool,
    session_id: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM messages WHERE session_id = ?")
        .bind(session_id)
        .execute(pool)
        .await?;

    // Update session timestamp
    sqlx::query("UPDATE sessions SET updated_at = ? WHERE id = ?")
        .bind(Utc::now().timestamp())
        .bind(session_id)
        .execute(pool)
        .await?;

    info!("Cleared all messages from session {}", session_id);
    Ok(())
}

/// Move a session to a folder.
pub async fn move_session_to_folder(
    pool: &SqlitePool,
    session_id: &str,
    folder_id: Option<&str>,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        UPDATE sessions
        SET folder_id = ?, updated_at = ?
        WHERE id = ?
        "#,
    )
    .bind(folder_id)
    .bind(Utc::now().timestamp())
    .bind(session_id)
    .execute(pool)
    .await?;

    Ok(())
}

// --- Folders CRUD ---

/// Create a new folder.
pub async fn create_folder(
    pool: &SqlitePool,
    name: String,
    color: Option<String>,
    folder_type: Option<String>,
) -> Result<Folder, sqlx::Error> {
    let id = Uuid::new_v4().to_string();
    let created_at = Utc::now().timestamp();
    let folder_color = color.unwrap_or_else(|| "#6366f1".to_string());
    let f_type = folder_type.unwrap_or_else(|| "session".to_string());

    sqlx::query(
        r#"
        INSERT INTO folders (id, name, color, sort_order, created_at, type)
        VALUES (?, ?, ?, 0, ?, ?)
        "#,
    )
    .bind(&id)
    .bind(&name)
    .bind(&folder_color)
    .bind(created_at)
    .bind(&f_type)
    .execute(pool)
    .await?;

    Ok(Folder {
        id,
        name,
        color: folder_color,
        sort_order: 0,
        created_at,
        folder_type: f_type,
    })
}

/// List all folders.
pub async fn list_folders(pool: &SqlitePool) -> Result<Vec<Folder>, sqlx::Error> {
    sqlx::query_as::<_, Folder>(
        r#"
        SELECT id, name, color, sort_order, created_at, type as folder_type
        FROM folders
        ORDER BY sort_order ASC, created_at ASC
        "#,
    )
    .fetch_all(pool)
    .await
}

/// Delete a folder (sessions and files in it become unfiled).
pub async fn delete_folder(pool: &SqlitePool, id: &str) -> Result<(), sqlx::Error> {
    // First, unfile all sessions in this folder
    sqlx::query(
        r#"
        UPDATE sessions SET folder_id = NULL WHERE folder_id = ?
        "#,
    )
    .bind(id)
    .execute(pool)
    .await?;

    // Unfile all library files in this folder
    sqlx::query(
        r#"
        UPDATE library_files SET folder_id = NULL WHERE folder_id = ?
        "#,
    )
    .bind(id)
    .execute(pool)
    .await?;

    // Then delete the folder
    sqlx::query(
        r#"
        DELETE FROM folders WHERE id = ?
        "#,
    )
    .bind(id)
    .execute(pool)
    .await?;

    Ok(())
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

// --- Library Files CRUD ---

pub async fn get_library_file(
    pool: &SqlitePool,
    file_id: &str,
) -> Result<LibraryFile, sqlx::Error> {
    sqlx::query_as::<_, LibraryFile>(
        r#"
        SELECT id, name, path, file_type, size, created_at, folder_id
        FROM library_files
        WHERE id = ?
        "#,
    )
    .bind(file_id)
    .fetch_one(pool)
    .await
}

pub async fn add_library_file(
    pool: &SqlitePool,
    id: &str,
    name: &str,
    path: &str,
    file_type: &str,
    size: i64,
) -> Result<LibraryFile, sqlx::Error> {
    let created_at = Utc::now().timestamp();

    sqlx::query_as::<_, LibraryFile>(
        r#"
        INSERT INTO library_files (id, name, path, file_type, size, created_at, folder_id)
        VALUES (?, ?, ?, ?, ?, ?, NULL)
        RETURNING id, name, path, file_type, size, created_at, folder_id
        "#,
    )
    .bind(id)
    .bind(name)
    .bind(path)
    .bind(file_type)
    .bind(size)
    .bind(created_at)
    .fetch_one(pool)
    .await
}

pub async fn link_file_to_session(
    pool: &SqlitePool,
    session_id: &str,
    file_id: &str,
) -> Result<(), sqlx::Error> {
    let attached_at = Utc::now().timestamp();
    sqlx::query(
        r#"
        INSERT OR IGNORE INTO session_files_link (session_id, file_id, attached_at)
        VALUES (?, ?, ?)
        "#,
    )
    .bind(session_id)
    .bind(file_id)
    .bind(attached_at)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn get_session_files(
    pool: &SqlitePool,
    session_id: &str,
) -> Result<Vec<SessionFile>, sqlx::Error> {
    sqlx::query_as::<_, SessionFile>(
        r#"
        SELECT
            l.id,
            s.session_id,
            l.name,
            l.path,
            l.file_type,
            l.size,
            s.attached_at
        FROM library_files l
        JOIN session_files_link s ON l.id = s.file_id
        WHERE s.session_id = ?
        ORDER BY s.attached_at ASC
        "#,
    )
    .bind(session_id)
    .fetch_all(pool)
    .await
}

#[allow(dead_code)]
pub async fn list_library_files(pool: &SqlitePool) -> Result<Vec<LibraryFile>, sqlx::Error> {
    sqlx::query_as::<_, LibraryFile>(
        r#"
        SELECT id, name, path, file_type, size, created_at, folder_id
        FROM library_files
        ORDER BY created_at DESC
        "#,
    )
    .fetch_all(pool)
    .await
}

/// Move a file to a folder.
pub async fn move_file_to_folder(
    pool: &SqlitePool,
    file_id: &str,
    folder_id: Option<&str>,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        UPDATE library_files
        SET folder_id = ?
        WHERE id = ?
        "#,
    )
    .bind(folder_id)
    .bind(file_id)
    .execute(pool)
    .await?;

    Ok(())
}

/// Delete a library file and its session links.
/// Returns the path of the file so it can be deleted from disk.
pub async fn delete_library_file(pool: &SqlitePool, file_id: &str) -> Result<String, sqlx::Error> {
    // Get file path first
    let file = sqlx::query_as::<_, LibraryFile>(
        r#"
        SELECT id, name, path, file_type, size, created_at, folder_id
        FROM library_files
        WHERE id = ?
        "#,
    )
    .bind(file_id)
    .fetch_one(pool)
    .await?;

    // Delete links
    sqlx::query("DELETE FROM session_files_link WHERE file_id = ?")
        .bind(file_id)
        .execute(pool)
        .await?;

    // Delete file record
    sqlx::query("DELETE FROM library_files WHERE id = ?")
        .bind(file_id)
        .execute(pool)
        .await?;

    Ok(file.path)
}
