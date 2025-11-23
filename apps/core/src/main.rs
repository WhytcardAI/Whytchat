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
use uuid::Uuid;

// --- State Management ---
struct AppState {
    supervisor: SupervisorHandle,
    pool: Option<sqlx::sqlite::SqlitePool>,
}

// --- Tauri Commands ---
#[tauri::command]
async fn debug_chat(
    message: String,
    window: tauri::Window,
    state: State<'_, AppState>,
) -> Result<String, String> {
    info!("Command received: debug_chat({})", message);

    // For now, use a default session ID. In a real implementation,
    // this would come from the frontend or be managed per conversation.
    let session_id = "default-session".to_string();

    state
        .supervisor
        .process_message(session_id, message, &window)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn download_model(window: tauri::Window) -> Result<String, String> {
    use futures::StreamExt;
    use std::io::Write;
    use tauri::Emitter;

    let model_url = "https://huggingface.co/Qwen/Qwen2.5-7B-Instruct-GGUF/resolve/main/qwen2.5-7b-instruct-q4_k_m.gguf";
    let models_dir = PortablePathManager::models_dir();
    let model_path = models_dir.join("qwen2.5-7b-instruct-q4_k_m.gguf");

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

    info!("File uploaded successfully: {}", relative_path);
    Ok(format!("File uploaded: {}", file_name))
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
