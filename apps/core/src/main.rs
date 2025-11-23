// WhytChat V1 Backend Entry Point
// "The Brain" - Orchestrator of Cognitive Actors

mod actors;
mod database;
mod fs_manager;
mod models;

use actors::supervisor::SupervisorHandle;
use fs_manager::PortablePathManager;
use log::{error, info};
use std::fs;
use tauri::State;
use url::Url;
use uuid::Uuid;

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

// Default model URL - Qwen 2.5 7B Instruct
const DEFAULT_MODEL_URL: &str = "https://huggingface.co/Qwen/Qwen2.5-7B-Instruct-GGUF/resolve/main/qwen2.5-7b-instruct-q4_k_m.gguf";
const DEFAULT_MODEL_FILENAME: &str = "qwen2.5-7b-instruct-q4_k_m.gguf";

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
async fn upload_file_for_session(
    session_id: String,
    file_name: String,
    file_data: Vec<u8>,
    state: State<'_, AppState>,
) -> Result<String, String> {
    info!(
        "Command received: upload_file_for_session({}, {})",
        session_id, file_name
    );

    // Get database pool
    let pool = state.pool.as_ref().ok_or("Database not initialized")?;

    // Create session files directory
    let session_dir = PortablePathManager::session_files_dir(&session_id);
    fs::create_dir_all(&session_dir).map_err(|e| format!("Failed to create directory: {}", e))?;

    // Generate unique filename to avoid conflicts
    let file_extension = std::path::Path::new(&file_name)
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("");
    let unique_filename = format!("{}_{}.{}", Uuid::new_v4(), file_name, file_extension);
    let file_path = session_dir.join(&unique_filename);

    // Security: Check file size limit (10MB)
    const MAX_FILE_SIZE: usize = 10 * 1024 * 1024;
    if file_data.len() > MAX_FILE_SIZE {
        return Err("File size exceeds maximum limit of 10MB".to_string());
    }

    // Security: Validate file type using MIME detection
    let allowed_text_types = ["text/", "application/json", "application/xml", "application/javascript"];
    let is_text_file = if let Some(kind) = infer::get(&file_data) {
        let mime_type = kind.mime_type();
        allowed_text_types.iter().any(|&t| mime_type.starts_with(t))
    } else {
        // If MIME type cannot be detected, check extension against allowlist
        matches!(
            file_extension.to_lowercase().as_str(),
            "txt" | "md" | "json" | "xml" | "csv" | "log" | "yml" | "yaml" | "toml" | "ini" | "conf" | "cfg"
        )
    };

    // Write file
    fs::write(&file_path, file_data).map_err(|e| format!("Failed to write file: {}", e))?;

    // Determine file type
    let file_type = if file_extension.is_empty() {
        "unknown"
    } else {
        file_extension
    };

    // Save to database
    let relative_path = format!("session_{}/{}", session_id, unique_filename);
    database::add_session_file(pool, &session_id, &relative_path, file_type)
        .await
        .map_err(|e| format!("Failed to save to database: {}", e))?;

    // Trigger RAG Ingestion
    let file_content_bytes = fs::read(&file_path).map_err(|e| format!("Failed to read file: {}", e))?;
    
    // Only ingest if it's a validated text-based file
    if is_text_file {
        let file_content_str = String::from_utf8_lossy(&file_content_bytes).to_string();
        info!("Ingesting file content into RAG...");
        let metadata = serde_json::json!({
            "source": unique_filename,
            "session_id": session_id,
            "file_path": relative_path
        }).to_string();

        match state.supervisor.ingest_content(file_content_str, Some(metadata)).await {
            Ok(_) => {
                info!("File content ingested successfully.");
                info!("File uploaded and ingested successfully: {}", relative_path);
                Ok(format!("File '{}' uploaded and ingested successfully", file_name))
            }
            Err(e) => {
                error!("Failed to ingest file content: {}", e);
                Err(format!("File uploaded but RAG ingestion failed: {}", e))
            }
        }
    } else {
        info!("File uploaded successfully (binary file, RAG skipped): {}", relative_path);
        Ok(format!("File '{}' uploaded successfully", file_name))
    }
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
    let supervisor = if let Ok(ref pool) = db_pool {
        SupervisorHandle::new_with_pool(Some(pool.clone()))
    } else {
        SupervisorHandle::new()
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
            upload_file_for_session
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
