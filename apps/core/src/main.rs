// WhytChat V1 Backend Entry Point
// "The Brain" - Orchestrator of Cognitive Actors

mod actors;
mod database;
mod error;
mod fs_manager;
mod models;
mod rate_limiter;

use actors::supervisor::SupervisorHandle;
use fs_manager::PortablePathManager;
use rate_limiter::RateLimiter;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};
mod encryption;
use std::time::Duration;
use tauri::State;
use tracing::{error, info, subscriber::set_global_default};
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_subscriber::{layer::SubscriberExt, EnvFilter, Registry};

// --- Constants ---
const DEFAULT_MODEL_FILENAME: &str = "default-model.bin";

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
    let mut app_handle = state.app_handle.lock().unwrap();
    *app_handle = Some(InitializedState {
        supervisor,
        pool: db_pool,
        rate_limiter: Mutex::new(RateLimiter::new(20, Duration::from_secs(60))),
    });

    // Mark as initialized
    state.is_initialized.store(true, Ordering::SeqCst);
    info!("Application initialized successfully.");

    Ok(())
}

#[tracing::instrument(skip(window, state))]
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
    let handle = state.app_handle.lock().unwrap();
    let initialized_state = handle.as_ref().ok_or("Application state not found")?;

    let current_session = session_id.unwrap_or_else(|| "default-session".to_string());
    info!("Command received: debug_chat({}, {})", current_session, message);

    {
        let mut limiter = initialized_state.rate_limiter.lock().unwrap();
        if !limiter.check(&current_session) {
            error!("Rate limit exceeded for session: {}", current_session);
            return Err(error::AppError::RateLimited.to_string());
        }
    }

    initialized_state
        .supervisor
        .process_message(current_session, message, &window)
        .await
        .map_err(|e| e.to_string())
}

/// Tauri command to create a new chat session.
///
/// # Arguments
///
/// * `title` - The initial title for the session.
/// * `state` - The managed `AppState`.
///
/// # Returns
///
/// A `Result` containing the new session's ID, or an error string.
#[tracing::instrument(skip(state))]
#[tauri::command]
async fn create_session(title: String, state: State<'_, AppState>) -> Result<String, String> {
    if !state.is_initialized.load(Ordering::SeqCst) {
        return Err("Application is not initialized yet.".to_string());
    }
    let handle = state.app_handle.lock().unwrap();
    let initialized_state = handle.as_ref().ok_or("Application state not found")?;

    let model_config = models::ModelConfig {
        model_id: DEFAULT_MODEL_FILENAME.to_string(),
        temperature: 0.7,
        system_prompt: String::new(),
    };
    let session = database::create_session(&initialized_state.pool, title, model_config)
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
    let handle = state.app_handle.lock().unwrap();
    let initialized_state = handle.as_ref().ok_or("Application state not found")?;
    let sessions = database::list_sessions(&initialized_state.pool)
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
    let handle = state.app_handle.lock().unwrap();
    let initialized_state = handle.as_ref().ok_or("Application state not found")?;
    let messages = database::get_session_messages(&initialized_state.pool, &session_id)
        .await
        .map_err(|e| e.to_string())?;
    Ok(messages)
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
    let handle = state.app_handle.lock().unwrap();
    let initialized_state = handle.as_ref().ok_or("Application state not found")?;
    database::update_session(&initialized_state.pool, &session_id, title, model_config)
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
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
    let handle = state.app_handle.lock().unwrap();
    let initialized_state = handle.as_ref().ok_or("Application state not found")?;

    info!("Command received: upload_file_for_session({}, {}, {} bytes)", session_id, file_name, file_data.len());

    const MAX_FILE_SIZE: usize = 10 * 1024 * 1024;
    if file_data.len() > MAX_FILE_SIZE {
        return Err("File size exceeds 10MB limit".to_string());
    }

    if file_data.contains(&0u8) {
        return Err("Binary files are not supported".to_string());
    }

    let allowed_types = ["text/plain", "text/markdown", "text/csv", "application/json"];
    if let Some(kind) = infer::get(&file_data) {
        if !allowed_types.contains(&kind.mime_type()) {
            return Err(format!("File type '{}' is not supported.", kind.mime_type()));
        }
    }

    let allowed_extensions = ["txt", "md", "csv", "json"];
    let extension = std::path::Path::new(&file_name)
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("");

    if !allowed_extensions.contains(&extension) {
        return Err(format!("File extension '.{}' is not supported.", extension));
    }

    let content = String::from_utf8(file_data).map_err(|e| format!("Invalid UTF-8 content: {}", e))?;

    initialized_state
        .supervisor
        .ingest_content(content, Some(format!("session:{}", session_id)))
        .await
        .map_err(|e| e.to_string())
}

/// Initialize tracing subscribers for structured logging.
fn init_tracing() {
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    let formatting_layer = BunyanFormattingLayer::new("whytchat_core".into(), std::io::stdout);
    let subscriber = Registry::default()
        .with(env_filter)
        .with(JsonStorageLayer)
        .with(formatting_layer);
    set_global_default(subscriber).expect("Failed to set global default subscriber");
}

fn main() {
    init_tracing();

    tauri::Builder::default()
        .manage(AppState::new())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            initialize_app,
            debug_chat,
            upload_file_for_session,
            create_session,
            list_sessions,
            get_session_messages,
            update_session
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
