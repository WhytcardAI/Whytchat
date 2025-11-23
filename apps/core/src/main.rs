// WhytChat V1 Backend Entry Point
// "The Brain" - Orchestrator of Cognitive Actors

mod actors;
mod database;
mod fs_manager;
mod models;

use actors::supervisor::SupervisorHandle;
use fs_manager::PortablePathManager;
use log::{error, info};
use tauri::State;
use url::Url;

// --- State Management ---
struct AppState {
    supervisor: SupervisorHandle,
    pool: Option<sqlx::sqlite::SqlitePool>,
}

// --- Tauri Commands ---
#[tauri::command]
async fn debug_chat(
    session_id: Option<String>,
    message: String,
    window: tauri::Window,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let current_session = session_id.unwrap_or_else(|| "default-session".to_string());
    info!("Command received: debug_chat({}, {})", current_session, message);

    state
        .supervisor
        .process_message(current_session, message, &window)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn create_session(
    title: Option<String>,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let pool = state.pool.as_ref().ok_or("Database not initialized")?;
    
    let session_title = title.unwrap_or_else(|| DEFAULT_SESSION_TITLE.to_string());
    let model_config = models::ModelConfig {
        model_id: DEFAULT_MODEL_FILENAME.to_string(),
        temperature: 0.7,
        system_prompt: DEFAULT_SYSTEM_PROMPT.to_string(),
    };
    
    let session = database::create_session(pool, session_title, model_config)
        .await
        .map_err(|e| e.to_string())?;
    
    Ok(session.id)
}

#[tauri::command]
async fn get_all_sessions(state: State<'_, AppState>) -> Result<Vec<models::Session>, String> {
    let pool = state.pool.as_ref().ok_or("Database not initialized")?;
    
    database::get_all_sessions(pool)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_session_messages(
    session_id: String,
    state: State<'_, AppState>,
) -> Result<Vec<models::Message>, String> {
    let pool = state.pool.as_ref().ok_or("Database not initialized")?;
    
    database::get_session_messages(pool, &session_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn process_user_message(
    session_id: String,
    content: String,
    window: tauri::Window,
    state: State<'_, AppState>,
) -> Result<String, String> {
    info!("Command received: process_user_message({}, {})", session_id, content);

    state
        .supervisor
        .process_message(session_id, content, &window)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn update_session(
    session_id: String,
    title: Option<String>,
    model_config: Option<models::ModelConfig>,
    state: State<'_, AppState>,
) -> Result<models::Session, String> {
    let pool = state.pool.as_ref().ok_or("Database not initialized")?;
    
    database::update_session(pool, &session_id, title, model_config)
        .await
        .map_err(|e| e.to_string())
}

// Default model URL - Qwen 2.5 7B Instruct
const DEFAULT_MODEL_URL: &str = "https://huggingface.co/Qwen/Qwen2.5-7B-Instruct-GGUF/resolve/main/qwen2.5-7b-instruct-q4_k_m.gguf";
const DEFAULT_MODEL_FILENAME: &str = "qwen2.5-7b-instruct-q4_k_m.gguf";
const DEFAULT_SESSION_TITLE: &str = "New session";
const DEFAULT_SYSTEM_PROMPT: &str = "You are a helpful AI assistant.";

#[tauri::command]
async fn download_model(window: tauri::Window, url: Option<String>) -> Result<String, String> {
    use futures::StreamExt;
    use std::io::Write;
    use tauri::Emitter;

    let model_url = url.unwrap_or_else(|| DEFAULT_MODEL_URL.to_string());
    let models_dir = PortablePathManager::models_dir();
    let model_filename = if model_url == DEFAULT_MODEL_URL {
        DEFAULT_MODEL_FILENAME.to_string()
    } else {
        // Proper URL parsing to extract filename
        match Url::parse(&model_url) {
            Ok(parsed_url) => {
                parsed_url
                    .path_segments()
                    .and_then(|segments| segments.last())
                    .filter(|s| !s.is_empty())
                    .unwrap_or(DEFAULT_MODEL_FILENAME)
                    .to_string()
            }
            Err(_) => DEFAULT_MODEL_FILENAME.to_string(),
        }
    };
    let model_path = models_dir.join(&model_filename);
    if model_path.exists() {
        return Ok("Model already exists".to_string());
    }

    info!("Starting download from {}", model_url);
    let _ = window.emit("download-progress", 0);

    let client = reqwest::Client::new();
    let res = client
        .get(model_url)
        .send()
        .await
        .map_err(|e| e.to_string())?;
    let total_size = res.content_length().unwrap_or(0);

    let mut file = std::fs::File::create(&model_path).map_err(|e| e.to_string())?;
    let mut downloaded: u64 = 0;
    let mut stream = res.bytes_stream();

    while let Some(item) = stream.next().await {
        let chunk = item.map_err(|e| e.to_string())?;
        file.write_all(&chunk).map_err(|e| e.to_string())?;
        downloaded += chunk.len() as u64;

        if total_size > 0 {
            let percent = (downloaded as f64 / total_size as f64) * 100.0;
            let _ = window.emit("download-progress", percent as u8);
        }
    }

    let _ = window.emit("download-progress", 100);
    Ok("Download complete".to_string())
}

#[tauri::command]
async fn create_session(state: State<'_, AppState>) -> Result<String, String> {
    let pool = state.pool.as_ref().ok_or("Database not initialized")?;
    let model_config = models::ModelConfig {
        model_id: DEFAULT_MODEL_FILENAME.to_string(),
        temperature: 0.7,
        system_prompt: String::new(), // Empty, will be set by frontend
    };
    let session = database::create_session(pool, String::new(), model_config).await.map_err(|e| e.to_string())?; // Empty title, will be set by frontend
    Ok(session.id)
}

#[tauri::command]
async fn list_sessions(state: State<'_, AppState>) -> Result<Vec<crate::models::Session>, String> {
    let pool = state.pool.as_ref().ok_or("Database not initialized")?;
    let sessions = database::get_all_sessions(pool).await.map_err(|e| e.to_string())?;
    Ok(sessions)
}

#[tauri::command]
async fn get_session_messages(session_id: String, state: State<'_, AppState>) -> Result<Vec<crate::models::Message>, String> {
    let pool = state.pool.as_ref().ok_or("Database not initialized")?;
    let messages = database::get_session_messages(pool, &session_id).await.map_err(|e| e.to_string())?;
    Ok(messages)
}

#[tauri::command]
async fn upload_file_for_session(
    session_id: String,
    file_name: String,
    file_data: Vec<u8>,
    state: State<'_, AppState>,
) -> Result<String, String> {
    info!("Command received: upload_file_for_session({}, {}, {} bytes)", session_id, file_name, file_data.len());

    // Security check: file size limit (10MB)
    const MAX_FILE_SIZE: usize = 10 * 1024 * 1024;
    if file_data.len() > MAX_FILE_SIZE {
        return Err("File size exceeds 10MB limit".to_string());
    }

    // Security check: detect binary files before UTF-8 conversion
    if file_data.contains(&0u8) {
        return Err("Binary files are not supported".to_string());
    }

    // Security check: MIME type validation
    let allowed_types = ["text/plain", "text/markdown", "text/csv", "application/json"];
    if let Some(kind) = infer::get(&file_data) {
        if !allowed_types.contains(&kind.mime_type()) {
            return Err(format!("File type '{}' is not supported. Only text-based files are allowed.", kind.mime_type()));
        }
    }

    // Security check: file extension allowlist
    let allowed_extensions = ["txt", "md", "csv", "json"];
    let extension = std::path::Path::new(&file_name)
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("");
    
    if !allowed_extensions.contains(&extension) {
        return Err(format!("File extension '.{}' is not supported. Allowed extensions: {}", extension, allowed_extensions.join(", ")));
    }

    // Convert bytes to string content
    let content = String::from_utf8(file_data).map_err(|e| format!("Invalid UTF-8 content: {}", e))?;

    // Ingest the content via supervisor
    state
        .supervisor
        .ingest_content(content, Some(format!("session:{}", session_id)))
        .await
        .map_err(|e| e.to_string())
}

fn main() {
    env_logger::init(); // Initialize logging

    // Initialize File System (Portable)
    if let Err(e) = PortablePathManager::init() {
        error!("Failed to initialize portable file system: {}", e);
        // On pourrait décider de paniquer ici si le FS est critique
        // panic!("FS Init failed: {}", e);
    }

    // Initialize Database
    let db_pool = tauri::async_runtime::block_on(async { database::init_db().await });

    if let Err(ref e) = db_pool {
        error!("Failed to initialize database: {}", e);
    }

    let mut builder = tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init());

    // Initialize the Actor System (Supervisor) after DB
    let model_path = PortablePathManager::models_dir().join(DEFAULT_MODEL_FILENAME);
    let supervisor = if let Ok(ref pool) = db_pool {
        SupervisorHandle::new_with_pool_and_model(Some(pool.clone()), model_path)
    } else {
        SupervisorHandle::new_with_pool_and_model(None, model_path)
    };

    let pool_clone = if let Ok(ref pool) = db_pool {
        Some(pool.clone())
    } else {
        None
    };

    builder = builder.manage(AppState {
        supervisor,
        pool: pool_clone,
    });

    if let Ok(pool) = db_pool {
        builder = builder.manage(pool);
    }

    builder
        .invoke_handler(tauri::generate_handler![
            debug_chat,
            download_model,
            upload_file_for_session,
            create_session,
            list_sessions,
            get_session_messages
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
