use crate::llama_install::{download_model, install_server};
use tauri::Emitter;

/// Auto-installation au premier lancement
pub async fn auto_install_on_first_run(app: &tauri::AppHandle) -> Result<(), String> {
    let models_dir = crate::llama_install::app_data_models_dir(app).map_err(|e| e.to_string())?;
    let model_file = models_dir.join("qwen2.5-3b-instruct-q4_k_m.gguf");

    let server_dir = crate::llama_install::app_llama_server_dir(app).map_err(|e| e.to_string())?;
    let server_exe = server_dir.join("llama-server.exe");

    let mut needs_install = false;

    // Vérifier modèle
    if !model_file.exists() {
        needs_install = true;
        let _ = app.emit("setup/status", "Téléchargement du modèle IA...");

        match download_model(app.clone(), None).await {
            Ok(path) => {
                let _ = app.emit("setup/status", format!("Modèle téléchargé: {}", path));
            }
            Err(e) => {
                let _ = app.emit(
                    "setup/error",
                    format!("Erreur téléchargement modèle: {}", e),
                );
                return Err(e);
            }
        }
    }

    // Vérifier serveur
    if !server_exe.exists() {
        needs_install = true;
        let _ = app.emit("setup/status", "Installation du serveur Llama...");

        match install_server(app.clone(), None).await {
            Ok(path) => {
                let _ = app.emit("setup/status", format!("Serveur installé: {}", path));
            }
            Err(e) => {
                let _ = app.emit("setup/error", format!("Erreur installation serveur: {}", e));
                return Err(e);
            }
        }
    }

    if needs_install {
        let _ = app.emit("setup/complete", "Installation terminée !");
    } else {
        let _ = app.emit("setup/ready", "Déjà installé");
    }

    Ok(())
}
