use serde::Serialize;
use tauri::Manager; // Importation du trait Manager
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
async fn start_chat(
    state: tauri::State<'_, ProcessManagerState>,
    app: tauri::AppHandle,
    messages: Vec<llama::Message>,
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
    let result = llama::chat(state, llama::ChatInput { messages }).await?;

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
                if let Err(e) = setup::auto_install_on_first_run(&handle).await {
                    eprintln!("Auto-install error: {}", e);
                }
            });
            Ok(())
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
            start_chat,
            rag::ingest_file,
            rag::query_rag,
            search::search_web // Commande Web Search
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
