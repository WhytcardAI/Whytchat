use serde::Serialize;
use tauri::Manager; // Importation du trait Manager
mod constants;
mod llama;
mod llama_install;
mod process_manager;
mod rag;
mod search; // Module recherche Web
mod setup;

use process_manager::ProcessManagerState;
use rag::RagState;

#[derive(Serialize)]
struct Health {
    status: &'static str,
    version: &'static str,
}

#[tauri::command]
fn health() -> Health {
    Health {
        status: "ok",
        version: env!("CARGO_PKG_VERSION"),
    }
}

#[tauri::command]
fn minimize_window(window: tauri::Window) {
    if let Err(e) = window.minimize() {
        eprintln!("Failed to minimize window: {}", e);
    }
}

#[tauri::command]
fn maximize_window(window: tauri::Window) {
    match window.is_maximized() {
        Ok(true) => {
            if let Err(e) = window.unmaximize() {
                eprintln!("Failed to unmaximize window: {}", e);
            }
        }
        Ok(false) => {
            if let Err(e) = window.maximize() {
                eprintln!("Failed to maximize window: {}", e);
            }
        }
        Err(e) => eprintln!("Failed to get window maximization state: {}", e),
    }
}

#[tauri::command]
fn close_window(window: tauri::Window) {
    if let Err(e) = window.close() {
        eprintln!("Failed to close window: {}", e);
    }
}

#[tauri::command]
async fn start_chat(
    state: tauri::State<'_, ProcessManagerState>,
    app: tauri::AppHandle,
    messages: Vec<llama::Message>,
    options: Option<llama::GenerationOptions>,
) -> Result<String, String> {
    // Vérifier si le serveur est prêt
    let ready = llama::server_ready(state.clone()).await?;

    if !ready {
        // Démarrer le serveur automatiquement
        llama::start_server(app.clone(), state.clone()).await?;

        // Attendre que le serveur soit prêt (max 10 secondes)
        // Note: start_server attend déjà un peu, mais on peut garder une sécurité supplémentaire
        // si start_server retourne dès que le process est spawn mais pas encore HTTP ready
        // Cependant, notre implémentation de start_server dans process_manager fait déjà un check_health
        // Donc on peut simplifier, mais pour la compatibilité avec la logique existante :
        for _ in 0..10 {
            if llama::server_ready(state.clone()).await? {
                break;
            }
            tokio::time::sleep(std::time::Duration::from_millis(200)).await;
        }
    }

    // Appeler la commande chat
    let result = llama::chat(app.clone(), state, llama::ChatInput { messages, options }).await?;

    Ok(result.reply)
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(ProcessManagerState::new())
        .setup(|app| {
            // Initialisation RAG
            match RagState::new(app.handle()) {
                Ok(state) => {
                    app.manage(state);
                }
                Err(e) => eprintln!("Failed to initialize RAG state: {}", e),
            }

            let handle = app.handle().clone();

            tauri::async_runtime::spawn(async move {
                // 1. Auto-install check
                if let Err(e) = setup::auto_install_on_first_run(&handle).await {
                    eprintln!("Auto-install error: {}", e);
                    return;
                }

                // 2. Auto-start server (Warm-up)
                // We try to start the server immediately so it's ready when the user types
                println!("Initiating server warm-up...");
                let state = handle.state::<ProcessManagerState>();
                if let Err(e) = llama::start_server(handle.clone(), state).await {
                    eprintln!("Server warm-up failed (non-critical if manual start works): {}", e);
                } else {
                    println!("Server warm-up successful.");
                }
            });
            Ok(())
        })
        .on_window_event(|window, event| match event {
            tauri::WindowEvent::CloseRequested { .. } => {
                println!("Application close requested, stopping all Llama servers...");
                let app_handle = window.app_handle();
                let state = app_handle.state::<ProcessManagerState>();

                tauri::async_runtime::spawn(async move {
                    // Get all server IDs and stop them
                    let server_ids: Vec<String> = {
                        let manager = state.0.lock().unwrap();
                        manager.servers.keys().cloned().collect()
                    };

                    for id in server_ids {
                        if let Err(e) = process_manager::stop_llama_server(&app_handle, &state, id.clone()).await {
                            eprintln!("Failed to stop server {}: {}", id, e);
                        }
                    }
                    println!("All Llama servers stopped.");
                });
            }
            _ => {}
        })
        .invoke_handler(tauri::generate_handler![
            health,
            llama_install::download_model,
            llama_install::install_server,
            llama::start_server,
            llama::stop_server,
            llama::chat,
            llama::chat_multi,
            llama::chat_fusion,
            llama::generate_response_with_context,
            llama::server_ready,
            llama::check_server_health,
            start_chat,
            rag::ingest_file,
            rag::query_rag,
            search::search_web, // Commande Web Search
            minimize_window,
            maximize_window,
            close_window
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}