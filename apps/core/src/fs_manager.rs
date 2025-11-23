use log::{error, info};
use std::fs;
use std::path::PathBuf;

pub struct PortablePathManager;

impl PortablePathManager {
    /// Récupère le répertoire racine de l'application (là où se trouve l'exécutable).
    pub fn root_dir() -> PathBuf {
        #[cfg(debug_assertions)]
        {
            // In development (Debug), we want to point to apps/core
            // The executable is usually in target/debug at the workspace root
            let mut path = std::env::current_exe().expect("Failed to get current exe");
            path.pop(); // remove exe name
            path.pop(); // remove debug
            path.pop(); // remove target

            // Check if we are at workspace root and apps/core exists
            let core_path = path.join("apps").join("core");
            if core_path.exists() {
                return core_path;
            }

            // Fallback: maybe we are already in apps/core (if target was local)
            return path;
        }

        #[cfg(not(debug_assertions))]
        match std::env::current_exe() {
            Ok(mut path) => {
                path.pop(); // Enlève le nom de l'exécutable pour garder le dossier
                path
            }
            Err(e) => {
                error!(
                    "Failed to get current exe path: {}. Falling back to current_dir.",
                    e
                );
                std::env::current_dir().expect("Failed to get current directory")
            }
        }
    }

    /// Récupère le répertoire de données principal (./data).
    pub fn data_dir() -> PathBuf {
        Self::root_dir().join("data")
    }

    /// Récupère le répertoire de la base de données (./data/db).
    pub fn db_dir() -> PathBuf {
        Self::data_dir().join("db")
    }

    /// Récupère le répertoire des modèles (./data/models).
    pub fn models_dir() -> PathBuf {
        Self::data_dir().join("models")
    }

    /// Récupère le répertoire des vecteurs (./data/vectors).
    pub fn vectors_dir() -> PathBuf {
        Self::data_dir().join("vectors")
    }

    /// Récupère le répertoire des fichiers de session (./data/files/session_{session_id}).
    pub fn session_files_dir(session_id: &str) -> PathBuf {
        Self::data_dir()
            .join("files")
            .join(format!("session_{}", session_id))
    }

    /// Initialise l'arborescence des fichiers.
    /// Crée les dossiers data, db, models et vectors s'ils n'existent pas.
    pub fn init() -> Result<(), std::io::Error> {
        let data_path = Self::data_dir();
        let db_path = Self::db_dir();
        let models_path = Self::models_dir();
        let vectors_path = Self::vectors_dir();

        if !data_path.exists() {
            info!("Creating data directory: {:?}", data_path);
            fs::create_dir_all(&data_path)?;
        }

        if !db_path.exists() {
            info!("Creating db directory: {:?}", db_path);
            fs::create_dir_all(&db_path)?;
        }

        if !models_path.exists() {
            info!("Creating models directory: {:?}", models_path);
            fs::create_dir_all(&models_path)?;
        }

        if !vectors_path.exists() {
            info!("Creating vectors directory: {:?}", vectors_path);
            fs::create_dir_all(&vectors_path)?;
        }

        Ok(())
    }
}
