#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

/// WhytChat V1 Backend Entry Point
/// "The Brain" - Orchestrator of Cognitive Actors
mod actors;
mod brain;
mod database;
mod diagnostics;
mod error;
mod fs_manager;
mod models;
mod preflight;
mod rate_limiter;
mod text_extract;

use actors::supervisor::SupervisorHandle;
use fs_manager::PortablePathManager;
use rate_limiter::RateLimiter;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};
mod encryption;
use futures::StreamExt;
use std::fs::File;
use std::time::Duration;
use tauri::{Emitter, RunEvent, State, WindowEvent};
use tokio::io::AsyncWriteExt;
use tracing::{error, info, subscriber::set_global_default, warn};
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_subscriber::{layer::SubscriberExt, EnvFilter, Registry};
use zip::ZipArchive;

// --- Constants ---
const DEFAULT_MODEL_FILENAME: &str = "default-model.gguf";
const LLAMA_SERVER_URL: &str = "https://github.com/ggml-org/llama.cpp/releases/download/b4154/llama-b4154-bin-win-avx2-x64.zip";
const MODEL_URL: &str = "https://huggingface.co/Qwen/Qwen2.5-Coder-7B-Instruct-GGUF/resolve/main/qwen2.5-coder-7b-instruct-q4_k_m.gguf";

// --- State Management ---

/// Holds the fully initialized state of the application, including actor handles and database pools.
/// This struct is stored within `AppState` once initialization is complete.
struct InitializedState {
    /// Handle to the supervisor actor, the main entry point for business logic.
    supervisor: SupervisorHandle,
    /// The main database connection pool.
    pool: sqlx::sqlite::SqlitePool,
    /// The rate limiter for incoming requests.
    rate_limiter: Mutex<RateLimiter>,
}

/// Manages the application's global state, including its initialization status.
///
/// This state is managed by Tauri and is accessible from all commands. It uses an `AtomicBool`
/// to quickly check if the backend is ready, and a `Mutex` to safely access the
/// `InitializedState` once it's available.
struct AppState {
    /// A flag indicating if the backend services have been initialized.
    is_initialized: Arc<AtomicBool>,
    /// The container for the `InitializedState`, available after `initialize_app` succeeds.
    app_handle: Arc<Mutex<Option<InitializedState>>>,
}

impl AppState {
    /// Creates a new, uninitialized `AppState`.
    fn new() -> Self {
        Self {
            is_initialized: Arc::new(AtomicBool::new(false)),
            app_handle: Arc::new(Mutex::new(None)),
        }
    }
}

// --- State Access Utilities ---

/// Acquires the app_handle lock and returns the initialized state reference.
/// Returns an error if the lock is poisoned or the state is not initialized.
fn get_initialized_state(
    state: &AppState,
) -> Result<std::sync::MutexGuard<'_, Option<InitializedState>>, String> {
    state
        .app_handle
        .lock()
        .map_err(|e| format!("Failed to acquire app_handle lock: {}", e))
}

/// Extracts the database pool from the initialized state.
/// This is a convenience function that handles the common pattern of getting just the pool.
fn get_pool(state: &AppState) -> Result<sqlx::sqlite::SqlitePool, String> {
    let handle = get_initialized_state(state)?;
    let initialized_state = handle.as_ref().ok_or("Application state not found")?;
    Ok(initialized_state.pool.clone())
}

/// Extracts both the database pool and supervisor handle from the initialized state.
/// Used by commands that need to interact with both the database and the actor system.
fn get_pool_and_supervisor(
    state: &AppState,
) -> Result<(sqlx::sqlite::SqlitePool, SupervisorHandle), String> {
    let handle = get_initialized_state(state)?;
    let initialized_state = handle.as_ref().ok_or("Application state not found")?;
    Ok((
        initialized_state.pool.clone(),
        initialized_state.supervisor.clone(),
    ))
}

/// Checks rate limit for a session and returns the pool and supervisor if allowed.
/// This combines the common pattern of rate limiting before processing a request.
fn check_rate_limit_and_get_resources(
    state: &AppState,
    session_id: &str,
) -> Result<(sqlx::sqlite::SqlitePool, SupervisorHandle), String> {
    let handle = get_initialized_state(state)?;
    let initialized_state = handle.as_ref().ok_or("Application state not found")?;

    let mut limiter = initialized_state
        .rate_limiter
        .lock()
        .map_err(|e| format!("Failed to acquire rate_limiter lock: {}", e))?;
    if !limiter.check(session_id) {
        return Err(error::AppError::RateLimited.to_string());
    }

    Ok((
        initialized_state.pool.clone(),
        initialized_state.supervisor.clone(),
    ))
}

// --- Tauri Commands ---

/// Asynchronously initializes the application's backend services.
/// This function is called by the frontend after the UI is ready.
#[tauri::command]
async fn initialize_app(state: State<'_, AppState>) -> Result<(), String> {
    if state.is_initialized.load(Ordering::SeqCst) {
        info!("Application already initialized.");
        return Ok(());
    }

    info!("Initializing application...");

    // Initialize File System
    if let Err(e) = PortablePathManager::init() {
        error!("Failed to initialize portable file system: {}", e);
        return Err(format!("FS Init failed: {}", e));
    }

    // Initialize Database
    let db_pool = database::init_db()
        .await
        .map_err(|e| format!("Failed to initialize database: {}", e))?;

    // Initialize Supervisor
    let model_path = PortablePathManager::models_dir().join(DEFAULT_MODEL_FILENAME);
    let supervisor = SupervisorHandle::new_with_pool_and_model(Some(db_pool.clone()), model_path);

    // Store the initialized state
    let mut app_handle = state
        .app_handle
        .lock()
        .map_err(|e| format!("Failed to acquire app_handle lock: {}", e))?;
    *app_handle = Some(InitializedState {
        supervisor,
        pool: db_pool,
        rate_limiter: Mutex::new(RateLimiter::new(20, Duration::from_secs(60))),
    });

    // Mark as initialized
    state.is_initialized.store(true, Ordering::SeqCst);
    info!("\n╔══════════════════════════════════════════════════════╗");
    info!("║  ✅ APPLICATION INITIALIZED SUCCESSFULLY              ║");
    info!("╚══════════════════════════════════════════════════════╝\n");

    Ok(())
}

/// Tauri command to process a chat message. This is the primary endpoint for user interaction.
///
/// It checks for initialization, applies rate limiting, and then passes the request
/// to the `SupervisorHandle` for processing.
///
/// # Arguments
///
/// * `session_id` - The ID of the session for the message.
/// * `message` - The user's message content.
/// * `window` - The Tauri window object, used for emitting events.
/// * `state` - The managed `AppState`.
///
/// # Returns
///
/// A `Result` containing the full, final response from the assistant, or an error string.
#[tracing::instrument(skip(window, state))]
#[tauri::command]
async fn debug_chat(
    session_id: Option<String>,
    message: String,
    window: tauri::Window,
    state: State<'_, AppState>,
) -> Result<String, String> {
    if !state.is_initialized.load(Ordering::SeqCst) {
        return Err("Application is not initialized yet.".to_string());
    }

    let current_session = session_id.unwrap_or_else(|| "default-session".to_string());
    info!("\n┌─────────────────────────────────────────────────────┐");
    info!("│ 💬 CHAT MESSAGE RECEIVED                            │");
    info!("├─────────────────────────────────────────────────────┤");
    info!("│ Session: {}", current_session);
    info!("│ Message: {} chars", message.len());
    info!("└─────────────────────────────────────────────────────┘");

    let (pool, supervisor) = check_rate_limit_and_get_resources(&state, &current_session)
        .inspect_err(|e| {
            if e.contains("Rate limit") {
                error!("Rate limit exceeded for session: {}", current_session);
            }
        })?;

    // Ensure session exists
    if database::get_session(&pool, &current_session)
        .await
        .is_err()
    {
        info!("Session {} not found, creating it...", current_session);
        let model_config = models::ModelConfig {
            model_id: DEFAULT_MODEL_FILENAME.to_string(),
            temperature: 0.7,
            system_prompt: String::new(),
        };
        if let Err(e) = database::create_session_with_id(
            &pool,
            &current_session,
            "New Chat".to_string(),
            model_config,
        )
        .await
        {
            error!("Failed to auto-create session {}: {}", current_session, e);
            return Err(format!("Failed to create session: {}", e));
        }
    }

    supervisor
        .process_message(current_session, message, Some(&window))
        .await
        .map_err(|e| e.to_string())
}

/// Tauri command to create a new chat session.
///
/// # Arguments
///
/// * `title` - The initial title for the session.
/// * `language` - The UI language code (e.g., "fr", "en") to set the model's response language.
/// * `state` - The managed `AppState`.
///
/// # Returns
///
/// A `Result` containing the new session's ID, or an error string.
#[tracing::instrument(skip(state))]
#[tauri::command]
async fn create_session(
    title: String,
    language: Option<String>,
    system_prompt: Option<String>,
    temperature: Option<f32>,
    state: State<'_, AppState>,
) -> Result<String, String> {
    if !state.is_initialized.load(Ordering::SeqCst) {
        return Err("Application is not initialized yet.".to_string());
    }

    let pool = get_pool(&state)?;

    // Set system prompt based on language or use provided one
    let final_system_prompt = if let Some(prompt) = system_prompt {
        prompt
    } else {
        match language.as_deref() {
            Some("fr") => "Tu es un assistant IA utile et amical. Réponds toujours en français de manière claire et concise.".to_string(),
            _ => "You are a helpful and friendly AI assistant. Always respond clearly and concisely.".to_string(),
        }
    };

    let model_config = models::ModelConfig {
        model_id: DEFAULT_MODEL_FILENAME.to_string(),
        temperature: temperature.unwrap_or(0.7),
        system_prompt: final_system_prompt,
    };
    let session = database::create_session(&pool, title, model_config)
        .await
        .map_err(|e| e.to_string())?;
    Ok(session.id)
}

/// Tauri command to list all existing chat sessions.
///
/// # Arguments
///
/// * `state` - The managed `AppState`.
///
/// # Returns
///
/// A `Result` containing a vector of `Session` objects, or an error string.
#[tracing::instrument(skip(state))]
#[tauri::command]
async fn list_sessions(state: State<'_, AppState>) -> Result<Vec<crate::models::Session>, String> {
    if !state.is_initialized.load(Ordering::SeqCst) {
        return Err("Application is not initialized yet.".to_string());
    }

    let pool = get_pool(&state)?;

    let sessions = database::list_sessions(&pool)
        .await
        .map_err(|e| e.to_string())?;
    Ok(sessions)
}

#[tracing::instrument(skip(state))]
#[tauri::command]
async fn get_session_messages(
    session_id: String,
    state: State<'_, AppState>,
) -> Result<Vec<crate::models::Message>, String> {
    if !state.is_initialized.load(Ordering::SeqCst) {
        return Err("Application is not initialized yet.".to_string());
    }

    let pool = get_pool(&state)?;

    let messages = database::get_session_messages(&pool, &session_id)
        .await
        .map_err(|e| e.to_string())?;
    Ok(messages)
}

#[tracing::instrument(skip(state))]
#[tauri::command]
async fn get_session_files(
    session_id: String,
    state: State<'_, AppState>,
) -> Result<Vec<crate::models::SessionFile>, String> {
    if !state.is_initialized.load(Ordering::SeqCst) {
        return Err("Application is not initialized yet.".to_string());
    }

    let pool = get_pool(&state)?;

    let files = database::get_session_files(&pool, &session_id)
        .await
        .map_err(|e| e.to_string())?;
    Ok(files)
}

#[tracing::instrument(skip(state))]
#[tauri::command]
async fn update_session(
    session_id: String,
    title: Option<String>,
    model_config: Option<models::ModelConfig>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    if !state.is_initialized.load(Ordering::SeqCst) {
        return Err("Application is not initialized yet.".to_string());
    }

    let pool = get_pool(&state)?;

    database::update_session(&pool, &session_id, title, model_config)
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tracing::instrument(skip(state))]
#[tauri::command]
async fn toggle_session_favorite(
    session_id: String,
    state: State<'_, AppState>,
) -> Result<bool, String> {
    if !state.is_initialized.load(Ordering::SeqCst) {
        return Err("Application is not initialized yet.".to_string());
    }

    let pool = get_pool(&state)?;

    database::toggle_session_favorite(&pool, &session_id)
        .await
        .map_err(|e| e.to_string())
}

#[tracing::instrument(skip(state))]
#[tauri::command]
async fn delete_session(session_id: String, state: State<'_, AppState>) -> Result<(), String> {
    if !state.is_initialized.load(Ordering::SeqCst) {
        return Err("Application is not initialized yet.".to_string());
    }

    let pool = get_pool(&state)?;

    database::delete_session(&pool, &session_id)
        .await
        .map_err(|e| e.to_string())
}

#[tracing::instrument(skip(state))]
#[tauri::command]
async fn create_folder(
    name: String,
    color: Option<String>,
    folder_type: Option<String>,
    state: State<'_, AppState>,
) -> Result<models::Folder, String> {
    if !state.is_initialized.load(Ordering::SeqCst) {
        return Err("Application is not initialized yet.".to_string());
    }

    let pool = get_pool(&state)?;

    database::create_folder(&pool, name, color, folder_type)
        .await
        .map_err(|e| e.to_string())
}

#[tracing::instrument(skip(state))]
#[tauri::command]
async fn list_folders(state: State<'_, AppState>) -> Result<Vec<models::Folder>, String> {
    if !state.is_initialized.load(Ordering::SeqCst) {
        return Err("Application is not initialized yet.".to_string());
    }

    let pool = get_pool(&state)?;

    database::list_folders(&pool)
        .await
        .map_err(|e| e.to_string())
}

#[tracing::instrument(skip(state))]
#[tauri::command]
async fn delete_folder(folder_id: String, state: State<'_, AppState>) -> Result<(), String> {
    if !state.is_initialized.load(Ordering::SeqCst) {
        return Err("Application is not initialized yet.".to_string());
    }

    let pool = get_pool(&state)?;

    database::delete_folder(&pool, &folder_id)
        .await
        .map_err(|e| e.to_string())
}

#[tracing::instrument(skip(state))]
#[tauri::command]
async fn move_session_to_folder(
    session_id: String,
    folder_id: Option<String>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    if !state.is_initialized.load(Ordering::SeqCst) {
        return Err("Application is not initialized yet.".to_string());
    }

    let pool = get_pool(&state)?;

    database::move_session_to_folder(&pool, &session_id, folder_id.as_deref())
        .await
        .map_err(|e| e.to_string())
}

#[tracing::instrument(skip(state))]
#[tauri::command]
async fn move_file_to_folder(
    file_id: String,
    folder_id: Option<String>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    if !state.is_initialized.load(Ordering::SeqCst) {
        return Err("Application is not initialized yet.".to_string());
    }

    let pool = get_pool(&state)?;

    database::move_file_to_folder(&pool, &file_id, folder_id.as_deref())
        .await
        .map_err(|e| e.to_string())
}

#[tracing::instrument(skip(state))]
#[tauri::command]
async fn delete_file(file_id: String, state: State<'_, AppState>) -> Result<(), String> {
    if !state.is_initialized.load(Ordering::SeqCst) {
        return Err("Application is not initialized yet.".to_string());
    }

    let pool = get_pool(&state)?;

    // 1. Remove from DB and get path
    let file_path_str = database::delete_library_file(&pool, &file_id)
        .await
        .map_err(|e| e.to_string())?;

    // 2. Remove from disk
    let path = std::path::Path::new(&file_path_str);
    if path.exists() {
        std::fs::remove_file(path)
            .map_err(|e| format!("Failed to delete file from disk: {}", e))?;
        info!("Deleted file from disk: {:?}", path);
    } else {
        warn!("File not found on disk, skipping deletion: {:?}", path);
    }

    Ok(())
}

#[tracing::instrument(skip(state))]
#[tauri::command]
async fn reindex_library(state: State<'_, AppState>) -> Result<String, String> {
    if !state.is_initialized.load(Ordering::SeqCst) {
        return Err("Application is not initialized yet.".to_string());
    }

    info!("Starting library reindexing...");

    let (pool, supervisor) = get_pool_and_supervisor(&state)?;

    let files = database::list_library_files(&pool)
        .await
        .map_err(|e| e.to_string())?;

    let total_files = files.len();
    let mut success_count = 0;
    let mut error_count = 0;

    for file in files {
        let path = std::path::Path::new(&file.path);
        if !path.exists() {
            warn!("File not found during reindex: {:?}", path);
            error_count += 1;
            continue;
        }

        match tokio::fs::read_to_string(path).await {
            Ok(content) => match supervisor.reindex_file(file.id.clone(), content).await {
                Ok(_) => {
                    info!("Reindexed file: {}", file.name);
                    success_count += 1;
                }
                Err(e) => {
                    error!("Failed to reindex file {}: {}", file.name, e);
                    error_count += 1;
                }
            },
            Err(e) => {
                error!("Failed to read file {}: {}", file.name, e);
                error_count += 1;
            }
        }
    }

    let result_msg = format!(
        "Reindexing complete. Processed {} files. Success: {}, Errors: {}",
        total_files, success_count, error_count
    );
    info!("{}", result_msg);
    Ok(result_msg)
}

#[tracing::instrument(skip(state))]
#[tauri::command]
async fn list_library_files(
    state: State<'_, AppState>,
) -> Result<Vec<crate::models::LibraryFile>, String> {
    if !state.is_initialized.load(Ordering::SeqCst) {
        return Err("Application is not initialized yet.".to_string());
    }

    let pool = get_pool(&state)?;

    database::list_library_files(&pool)
        .await
        .map_err(|e| e.to_string())
}

#[tracing::instrument(skip(state))]
#[tauri::command]
async fn save_generated_file(
    session_id: String,
    file_name: String,
    content: String,
    state: State<'_, AppState>,
) -> Result<String, String> {
    if !state.is_initialized.load(Ordering::SeqCst) {
        return Err("Application is not initialized yet.".to_string());
    }

    info!("\n┌─────────────────────────────────────────────────────┐");
    info!("│ 💾 SAVE GENERATED FILE                              │");
    info!("├─────────────────────────────────────────────────────┤");
    info!("│ Session: {}", session_id);
    info!("│ File: {}", file_name);
    info!("└─────────────────────────────────────────────────────┘");

    // Get pool and supervisor
    let (pool, supervisor) = get_pool_and_supervisor(&state)?;

    // 1. Save file to disk
    let file_uuid = uuid::Uuid::new_v4().to_string();
    let files_dir = PortablePathManager::data_dir().join("files");
    std::fs::create_dir_all(&files_dir)
        .map_err(|e| format!("Failed to create files directory: {}", e))?;

    // Basic sanitization
    let safe_name = file_name.replace(
        |c: char| !c.is_alphanumeric() && c != '.' && c != '-' && c != '_',
        "_",
    );
    let extension = std::path::Path::new(&safe_name)
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("txt");

    let stored_filename = format!("{}.{}", file_uuid, extension);
    let file_path = files_dir.join(&stored_filename);

    std::fs::write(&file_path, &content).map_err(|e| format!("Failed to save file: {}", e))?;

    // 2. Add to DB
    let file_type = match extension {
        "md" => "text/markdown",
        "csv" => "text/csv",
        "json" => "application/json",
        "js" | "jsx" | "ts" | "tsx" => "application/javascript",
        "py" => "text/x-python",
        "rs" => "text/rust",
        "html" => "text/html",
        "css" => "text/css",
        _ => "text/plain",
    };

    let _library_file = database::add_library_file(
        &pool,
        &file_uuid,
        &safe_name,
        &file_path.to_string_lossy(),
        file_type,
        content.len() as i64,
    )
    .await
    .map_err(|e| format!("Failed to add file to library: {}", e))?;

    // 3. Link to session
    database::link_file_to_session(&pool, &session_id, &file_uuid)
        .await
        .map_err(|e| format!("Failed to link file to session: {}", e))?;

    // 4. Ingest
    supervisor
        .ingest_content(content, Some(format!("file:{}", file_uuid)))
        .await
        .map_err(|e| e.to_string())?;

    Ok(file_uuid)
}

#[tracing::instrument(skip(file_data, state))]
#[tauri::command]
async fn upload_file_for_session(
    session_id: String,
    file_name: String,
    file_data: Vec<u8>,
    state: State<'_, AppState>,
) -> Result<String, String> {
    if !state.is_initialized.load(Ordering::SeqCst) {
        return Err("Application is not initialized yet.".to_string());
    }
    // Lock moved to where it is needed to avoid holding it across await points

    info!("\n┌─────────────────────────────────────────────────────┐");
    info!("│ 📁 FILE UPLOAD                                      │");
    info!("├─────────────────────────────────────────────────────┤");
    info!("│ Session: {}", session_id);
    info!("│ File: {}", file_name);
    info!("│ Size: {} bytes", file_data.len());
    info!("└─────────────────────────────────────────────────────┘");

    const MAX_FILE_SIZE: usize = 10 * 1024 * 1024;
    if file_data.len() > MAX_FILE_SIZE {
        return Err("File size exceeds 10MB limit".to_string());
    }

    // Check file extension
    let extension = std::path::Path::new(&file_name)
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|s| s.to_lowercase())
        .unwrap_or_default();

    let allowed_extensions = ["txt", "md", "csv", "json", "pdf", "docx", "doc"];
    if !allowed_extensions.contains(&extension.as_str()) {
        return Err(format!("File extension '.{}' is not supported.", extension));
    }

    // Binary file check (PDF and DOCX are binary but allowed)
    let binary_extensions = ["pdf", "docx", "doc"];
    let is_binary_allowed = binary_extensions.contains(&extension.as_str());

    if file_data.contains(&0u8) && !is_binary_allowed {
        return Err("Binary files are not supported".to_string());
    }

    // Extract text content using our text extraction module
    let content = text_extract::extract_text_from_file(&file_name, &file_data)?;

    if content.trim().is_empty() {
        return Err("No text content could be extracted from the file".to_string());
    }

    info!("   ✓ Extracted {} characters from file", content.len());

    // Get pool and supervisor from state
    let (pool, supervisor) = get_pool_and_supervisor(&state)?;

    // Ensure session exists (Fix for FOREIGN KEY constraint failed)
    if database::get_session(&pool, &session_id).await.is_err() {
        info!(
            "Session {} not found during upload, creating it...",
            session_id
        );
        let model_config = models::ModelConfig {
            model_id: DEFAULT_MODEL_FILENAME.to_string(),
            temperature: 0.7,
            system_prompt: String::new(),
        };
        if let Err(e) = database::create_session_with_id(
            &pool,
            &session_id,
            "New Chat".to_string(),
            model_config,
        )
        .await
        {
            return Err(format!("Failed to auto-create session: {}", e));
        }
    }

    // 1. Save file to disk (Global Library Storage)
    let file_uuid = uuid::Uuid::new_v4().to_string();
    let files_dir = PortablePathManager::data_dir().join("files");
    std::fs::create_dir_all(&files_dir)
        .map_err(|e| format!("Failed to create files directory: {}", e))?;

    let extension = std::path::Path::new(&file_name)
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("txt");

    let stored_filename = format!("{}.{}", file_uuid, extension);
    let file_path = files_dir.join(&stored_filename);

    std::fs::write(&file_path, &file_data).map_err(|e| format!("Failed to save file: {}", e))?;
    info!("   ✓ File saved to {:?}", file_path);

    // 2. Add file record to database
    let file_type = match extension {
        "md" => "text/markdown",
        "csv" => "text/csv",
        "json" => "application/json",
        _ => "text/plain",
    };

    // Add to library
    let _library_file = database::add_library_file(
        &pool,
        &file_uuid,
        &file_name,
        &file_path.to_string_lossy(),
        file_type,
        file_data.len() as i64,
    )
    .await
    .map_err(|e| format!("Failed to add file to library: {}", e))?;

    // Link to session
    database::link_file_to_session(&pool, &session_id, &file_uuid)
        .await
        .map_err(|e| format!("Failed to link file to session: {}", e))?;

    info!("   ✓ File record added to database and linked to session");
    info!("   ✓ Ingesting content into RAG...");

    // 3. Ingest content into RAG
    // Use file:{uuid} as metadata so we can filter by file later
    supervisor
        .ingest_content(content, Some(format!("file:{}", file_uuid)))
        .await
        .map_err(|e| e.to_string())
}

/// Links an existing library file to a session (without re-uploading)
#[tauri::command]
async fn link_library_file_to_session(
    session_id: String,
    file_id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    if !state.is_initialized.load(Ordering::SeqCst) {
        return Err("Application is not initialized yet.".to_string());
    }

    info!("\n┌─────────────────────────────────────────────────────┐");
    info!("│ 🔗 LINK FILE TO SESSION                             │");
    info!("├─────────────────────────────────────────────────────┤");
    info!("│ Session: {}", session_id);
    info!("│ File ID: {}", file_id);
    info!("└─────────────────────────────────────────────────────┘");

    let pool = get_pool(&state)?;

    // Verify file exists in library
    let _file = database::get_library_file(&pool, &file_id)
        .await
        .map_err(|e| format!("File not found in library: {}", e))?;

    // Link file to session
    database::link_file_to_session(&pool, &session_id, &file_id)
        .await
        .map_err(|e| format!("Failed to link file to session: {}", e))?;

    info!("   ✓ File linked to session successfully");
    Ok(())
}

/// Downloads a file with resume support.
/// If the file already exists partially, it will resume from where it left off.
async fn download_file<P: AsRef<std::path::Path>>(
    url: &str,
    path: P,
    window: &tauri::Window,
    progress_base: u64,
    progress_scale: u64,
) -> Result<(), String> {
    let path_ref = path.as_ref();
    info!("Starting download from '{}' to '{:?}'", url, path_ref);

    // Check if file already exists (partial download)
    let existing_size = match std::fs::metadata(path_ref) {
        Ok(meta) => meta.len(),
        Err(_) => 0,
    };

    let client = reqwest::Client::builder()
        .user_agent("WhytChat/1.0")
        .tcp_keepalive(Duration::from_secs(30))
        .redirect(reqwest::redirect::Policy::limited(10)) // Follow up to 10 redirects
        .build()
        .map_err(|e| format!("Failed to build client: {}", e))?;

    // Get total size: try HEAD first, fallback to GET with Range: bytes=0-0
    // HuggingFace uses 302 redirects that might not preserve Content-Length on HEAD
    let total_size = {
        // Try HEAD first
        let head_res = client.head(url).send().await;

        match head_res {
            Ok(res) if res.status().is_success() => res.content_length().unwrap_or(0),
            _ => 0,
        }
    };

    // If HEAD didn't give us size, try a range request to get it
    let total_size = if total_size == 0 {
        info!("HEAD request didn't return content-length, trying GET with Range header");
        let range_res = client
            .get(url)
            .header("Range", "bytes=0-0")
            .send()
            .await
            .map_err(|e| format!("Failed to GET range from '{}': {}", url, e))?;

        // Content-Range header format: "bytes 0-0/TOTAL_SIZE"
        if let Some(content_range) = range_res.headers().get("content-range") {
            if let Ok(range_str) = content_range.to_str() {
                // Parse "bytes 0-0/12345678" to extract 12345678
                if let Some(total_str) = range_str.split('/').next_back() {
                    total_str.parse::<u64>().unwrap_or(0)
                } else {
                    0
                }
            } else {
                0
            }
        } else {
            // Last resort: use content-length from response headers if available
            range_res.content_length().unwrap_or(0)
        }
    } else {
        total_size
    };

    info!(
        "Total file size: {} bytes, already downloaded: {} bytes",
        total_size, existing_size
    );

    // If file is already complete, skip download
    if total_size > 0 && existing_size >= total_size {
        info!("File already fully downloaded, skipping");
        window
            .emit("download-progress", progress_base + progress_scale)
            .ok();
        return Ok(());
    }

    // Also, if total_size is 0, it's an error.
    if total_size == 0 {
        return Err(format!(
            "Failed to get valid content length from '{}', size was 0.",
            url
        ));
    }

    // Build request with Range header for resume
    let mut request = client.get(url);
    let mut downloaded: u64 = existing_size;

    if existing_size > 0 {
        info!("Resuming download from byte {}", existing_size);
        request = request.header("Range", format!("bytes={}-", existing_size));
    }

    let res = request
        .send()
        .await
        .map_err(|e| format!("Failed to GET from '{}': {}", url, e))?;

    let status = res.status();

    // 206 Partial Content = resume successful, 200 OK = server doesn't support resume
    if !status.is_success() && status != reqwest::StatusCode::PARTIAL_CONTENT {
        let err_msg = format!("Download failed. Status: {}. URL: {}", status, url);
        error!("{}", err_msg);
        return Err(err_msg);
    }

    // If server returned 200 instead of 206, it doesn't support resume - start from scratch
    if status == reqwest::StatusCode::OK && existing_size > 0 {
        info!("Server doesn't support resume, starting from scratch");
        downloaded = 0;
    }

    // Open file in append mode if resuming, create mode if starting fresh
    let file = if downloaded > 0 {
        tokio::fs::OpenOptions::new()
            .append(true)
            .open(path_ref)
            .await
            .map_err(|e| format!("Failed to open file for append at '{:?}': {}", path_ref, e))?
    } else {
        tokio::fs::File::create(path_ref)
            .await
            .map_err(|e| format!("Failed to create file at '{:?}': {}", path_ref, e))?
    };

    // Use a massive buffer (8MB) to ensure disk I/O is never the bottleneck
    let mut writer = tokio::io::BufWriter::with_capacity(8 * 1024 * 1024, file);
    let mut stream = res.bytes_stream();
    let mut last_emit = std::time::Instant::now();

    // Emit initial progress (especially important for resume)
    let initial_percentage = progress_base + (downloaded * progress_scale / total_size);
    window.emit("download-progress", initial_percentage).ok();

    while let Some(item) = stream.next().await {
        let chunk = item.map_err(|e| format!("Error while downloading file: {}", e))?;
        writer
            .write_all(&chunk)
            .await
            .map_err(|e| format!("Error while writing to file: {}", e))?;
        downloaded += chunk.len() as u64;

        // Throttle progress events to avoid flooding the frontend
        if last_emit.elapsed() >= Duration::from_millis(100) || downloaded >= total_size {
            let percentage = progress_base + (downloaded * progress_scale / total_size);
            // Make emit non-fatal (log error but continue)
            if let Err(e) = window.emit("download-progress", percentage) {
                error!("Failed to emit progress event: {}", e);
            }
            last_emit = std::time::Instant::now();
        }
    }
    writer
        .flush()
        .await
        .map_err(|e| format!("Error flushing file: {}", e))?;
    info!("Download complete: {:?} ({} bytes)", path_ref, downloaded);
    Ok(())
}

fn extract_zip<P: AsRef<std::path::Path>>(zip_path: P, extract_to: P) -> Result<(), String> {
    let file = File::open(zip_path).map_err(|e| format!("Failed to open zip file: {}", e))?;
    let mut archive =
        ZipArchive::new(file).map_err(|e| format!("Failed to read zip archive: {}", e))?;
    archive
        .extract(extract_to)
        .map_err(|e| format!("Failed to extract zip archive: {}", e))?;
    Ok(())
}

/// Validates that llama-server installation is complete with all required files.
/// Returns Ok(()) if valid, Err with details if something is missing.
fn validate_llama_installation(llama_dir: &std::path::Path) -> Result<(), String> {
    // Critical files that MUST exist for llama-server to work
    let required_files = if cfg!(windows) {
        vec!["llama-server.exe", "llama.dll", "ggml.dll"]
    } else {
        vec!["llama-server"]
    };

    let mut missing = Vec::new();

    for file in &required_files {
        let file_path = llama_dir.join(file);
        if !file_path.exists() {
            missing.push(file.to_string());
        }
    }

    if !missing.is_empty() {
        return Err(format!(
            "Incomplete llama-server installation. Missing files: {}. \
             Try deleting {:?} and restarting the app.",
            missing.join(", "),
            llama_dir
        ));
    }

    // Check that llama-server.exe is not 0 bytes (corrupted download)
    let server_exe = if cfg!(windows) {
        "llama-server.exe"
    } else {
        "llama-server"
    };
    let server_path = llama_dir.join(server_exe);

    if let Ok(meta) = std::fs::metadata(&server_path) {
        if meta.len() < 1024 * 1024 {
            // Less than 1 MB is definitely wrong
            return Err(format!(
                "llama-server binary appears corrupted (size: {} bytes). \
                 Delete {:?} and restart the app.",
                meta.len(),
                llama_dir
            ));
        }
    }

    Ok(())
}

/// Validates that the model file is complete and not corrupted.
fn validate_model_file(model_path: &std::path::Path) -> Result<(), String> {
    if !model_path.exists() {
        return Err(format!("Model file not found at {:?}", model_path));
    }

    let meta =
        std::fs::metadata(model_path).map_err(|e| format!("Cannot read model file: {}", e))?;

    let size = meta.len();

    if size < MIN_MODEL_SIZE_BYTES {
        return Err(format!(
            "Model file incomplete: {} bytes (minimum {} bytes required). \
             Delete the file and restart the download.",
            size, MIN_MODEL_SIZE_BYTES
        ));
    }

    // Quick GGUF header check: first 4 bytes should be "GGUF" magic
    let file =
        std::fs::File::open(model_path).map_err(|e| format!("Cannot open model file: {}", e))?;

    let mut reader = std::io::BufReader::new(file);
    let mut magic = [0u8; 4];

    use std::io::Read;
    reader
        .read_exact(&mut magic)
        .map_err(|e| format!("Cannot read model header: {}", e))?;

    // GGUF magic is "GGUF" in little-endian
    if &magic != b"GGUF" {
        return Err(format!(
            "Model file appears corrupted (invalid GGUF header). \
             Expected 'GGUF', got {:?}. Delete and re-download.",
            magic
        ));
    }

    info!(
        "✓ Model file validated: {:.2} GB, valid GGUF format",
        size as f64 / 1024.0 / 1024.0 / 1024.0
    );
    Ok(())
}

/// Minimum expected model size in bytes (3 GB).
/// The Qwen2.5-Coder-7B-Instruct-Q4_K_M model is ~4.7 GB.
/// A partial download will be smaller, so we reject anything under 3 GB.
const MIN_MODEL_SIZE_BYTES: u64 = 3 * 1024 * 1024 * 1024; // 3 GB

/// Tauri command to check if the model file exists AND is complete.
/// Used by frontend to determine if onboarding should be shown.
/// Returns true only if the model file exists and is at least 3 GB.
#[tauri::command]
fn check_model_exists() -> bool {
    let model_path = PortablePathManager::models_dir().join(DEFAULT_MODEL_FILENAME);

    match std::fs::metadata(&model_path) {
        Ok(metadata) => {
            let size = metadata.len();
            let is_complete = size >= MIN_MODEL_SIZE_BYTES;
            info!(
                "Model check: {:?} size = {} bytes, min required = {} bytes, complete = {}",
                model_path, size, MIN_MODEL_SIZE_BYTES, is_complete
            );
            is_complete
        }
        Err(_) => {
            info!("Model check: {:?} does not exist", model_path);
            false
        }
    }
}

/// Tauri command to run a quick preflight check (no startup tests).
/// This is faster and just checks file existence.
#[tauri::command]
fn run_quick_preflight_check() -> preflight::PreflightReport {
    info!("Running quick preflight check...");
    preflight::quick_preflight_check()
}

/// Tauri command to run diagnostic tests for a specific category.
#[tauri::command]
async fn run_diagnostic_category(
    category: String,
    window: tauri::Window,
) -> Result<Vec<diagnostics::TestResult>, String> {
    info!("Running diagnostic tests for category: {}", category);

    let results = diagnostics::run_category_tests(&category).await;

    // Emit each result
    for result in &results {
        let _ = window.emit("diagnostic-test-result", result);
    }

    Ok(results)
}

#[tracing::instrument(skip(window))]
#[tauri::command]
async fn download_model(window: tauri::Window) -> Result<(), String> {
    info!("Starting model download process...");

    // Emit initial progress
    window
        .emit("download-progress", 0)
        .map_err(|e| e.to_string())?;

    // 1. Check and Download llama-server
    let tools_dir = PortablePathManager::tools_dir();
    let llama_dir = tools_dir.join("llama");
    let server_exe_name = if cfg!(windows) {
        "llama-server.exe"
    } else {
        "llama-server"
    };

    // Check if installation exists AND is valid
    let server_needs_install = if llama_dir.exists() && llama_dir.join(server_exe_name).exists() {
        match validate_llama_installation(&llama_dir) {
            Ok(_) => {
                info!("llama-server installation verified. Skipping download.");
                false
            }
            Err(e) => {
                warn!("Existing llama-server is invalid: {}. Will re-download.", e);
                // Delete corrupted installation
                if let Err(del_err) = std::fs::remove_dir_all(&llama_dir) {
                    warn!("Failed to delete corrupted llama-server: {}", del_err);
                }
                true
            }
        }
    } else {
        true
    };

    if server_needs_install {
        if !llama_dir.exists() {
            std::fs::create_dir_all(&llama_dir).map_err(|e| e.to_string())?;
        }

        let zip_path = llama_dir.join("llama-server.zip");
        info!("Downloading llama-server to {:?}", zip_path);

        // Download server (0-20%)
        download_file(LLAMA_SERVER_URL, &zip_path, &window, 0, 20).await?;

        info!("Extracting llama-server...");
        let zip_path_clone = zip_path.clone();
        let llama_dir_clone = llama_dir.clone();

        tokio::task::spawn_blocking(move || extract_zip(&zip_path_clone, &llama_dir_clone))
            .await
            .map_err(|e| format!("Task join error: {}", e))??;

        // Validate extraction: check that critical files exist
        validate_llama_installation(&llama_dir)?;
        info!("✓ llama-server installation validated");
    }
    window
        .emit("download-progress", 20)
        .map_err(|e| e.to_string())?;

    // 2. Check and Download Model
    let models_dir = PortablePathManager::models_dir();
    if !models_dir.exists() {
        std::fs::create_dir_all(&models_dir).map_err(|e| e.to_string())?;
    }
    let model_path = models_dir.join(DEFAULT_MODEL_FILENAME);

    // Check if model exists AND is valid
    let model_needs_download = if model_path.exists() {
        match validate_model_file(&model_path) {
            Ok(_) => {
                info!("Model already exists and validated. Skipping download.");
                false
            }
            Err(e) => {
                warn!("Existing model is invalid: {}. Will re-download.", e);
                // Delete corrupted file to allow fresh download
                if let Err(del_err) = std::fs::remove_file(&model_path) {
                    warn!("Failed to delete corrupted model: {}", del_err);
                }
                true
            }
        }
    } else {
        true
    };

    if !model_needs_download {
        window
            .emit("download-progress", 50)
            .map_err(|e| e.to_string())?;
        tokio::time::sleep(Duration::from_millis(200)).await;
        window
            .emit("download-progress", 90)
            .map_err(|e| e.to_string())?;
        tokio::time::sleep(Duration::from_millis(200)).await;
        window
            .emit("download-progress", 100)
            .map_err(|e| e.to_string())?;
        return Ok(());
    }

    info!("Downloading model to {:?}", model_path);

    // Download model (20-80%)
    download_file(MODEL_URL, &model_path, &window, 20, 60).await?;

    // Validate downloaded model
    validate_model_file(&model_path)?;
    info!("✓ Model download complete and validated.");
    window
        .emit("download-progress", 80)
        .map_err(|e| e.to_string())?;

    // 3. Initialize FastEmbed model (downloads ONNX model if needed) (80-95%)
    info!("┌─────────────────────────────────────────────────────┐");
    info!("│ 🧠 INITIALIZING EMBEDDING MODEL                     │");
    info!("└─────────────────────────────────────────────────────┘");

    // This will download AllMiniLML6V2 ONNX model (~25MB) on first run
    // Store in data/models/embeddings/ to keep everything portable
    let embeddings_dir = PortablePathManager::models_dir().join("embeddings");
    std::fs::create_dir_all(&embeddings_dir).ok();

    let embedding_result = tokio::task::spawn_blocking(move || {
        use fastembed::{EmbeddingModel, InitOptions, TextEmbedding};
        let mut options = InitOptions::new(EmbeddingModel::AllMiniLML6V2);
        options.show_download_progress = true;
        options.cache_dir = embeddings_dir;
        TextEmbedding::try_new(options)
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))?;

    match embedding_result {
        Ok(_) => {
            info!("   ✓ Embedding model initialized successfully");
            window
                .emit("download-progress", 95)
                .map_err(|e| e.to_string())?;
        }
        Err(e) => {
            error!("   ✗ Failed to initialize embedding model: {}", e);
            // Non-fatal - continue anyway, RAG will try again later
        }
    }

    window
        .emit("download-progress", 100)
        .map_err(|e| e.to_string())?;
    info!("╔══════════════════════════════════════════════════════╗");
    info!("║  ✅ ALL MODELS DOWNLOADED SUCCESSFULLY               ║");
    info!("╚══════════════════════════════════════════════════════╝");
    Ok(())
}

/// Initialize tracing subscribers for structured logging.
fn init_tracing() {
    // Default filter: info for our crate, warn for noisy dependencies
    let default_filter = "whytchat_core=info,\
                          fastembed=warn,\
                          ort=warn,\
                          reqwest=warn,\
                          hyper=warn,\
                          h2=warn,\
                          tungstenite=warn,\
                          tokio=warn,\
                          sqlx=warn,\
                          info";

    let env_filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(default_filter));
    let formatting_layer = BunyanFormattingLayer::new("whytchat_core".into(), std::io::stdout);
    let subscriber = Registry::default()
        .with(env_filter)
        .with(JsonStorageLayer)
        .with(formatting_layer);
    set_global_default(subscriber).expect("Failed to set global default subscriber");
}

fn main() {
    dotenv::dotenv().ok();
    init_tracing();

    // Ensure LLAMA_AUTH_TOKEN is set globally for all components (Actors, Diagnostics)
    if std::env::var("LLAMA_AUTH_TOKEN").is_err() {
        let token = uuid::Uuid::new_v4().to_string();
        std::env::set_var("LLAMA_AUTH_TOKEN", &token);
        tracing::info!("Generated global LLAMA_AUTH_TOKEN: {}", token);
    }

    let app = tauri::Builder::default()
        .manage(AppState::new())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            initialize_app,
            debug_chat,
            upload_file_for_session,
            link_library_file_to_session,
            create_session,
            list_sessions,
            get_session_messages,
            get_session_files,
            update_session,
            toggle_session_favorite,
            delete_session,
            create_folder,
            list_folders,
            delete_folder,
            move_session_to_folder,
            move_file_to_folder,
            delete_file,
            reindex_library,
            list_library_files,
            save_generated_file,
            download_model,
            check_model_exists,
            run_quick_preflight_check,
            run_diagnostic_category
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application");

    app.run(|_app_handle, event| {
        match event {
            RunEvent::Exit => {
                info!("Application exit event received. Cleaning up...");
                // Force kill any remaining llama-server processes on Windows
                #[cfg(windows)]
                {
                    let _ = std::process::Command::new("taskkill")
                        .args(["/F", "/IM", "llama-server.exe"])
                        .output();
                    info!("Cleanup: killed any remaining llama-server.exe processes");
                }
            }
            RunEvent::WindowEvent {
                label,
                event: WindowEvent::CloseRequested { .. },
                ..
            } => {
                info!("Window {} close requested", label);
            }
            _ => {}
        }
    });
}
