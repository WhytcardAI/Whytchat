use tracing::info;
use std::fs;
use std::path::PathBuf;

/// Manages application paths to ensure portability between development and release builds.
///
/// This struct provides a centralized way to access key directories, abstracting away the
/// differences in file structure between a debug environment (workspace root) and a
/// release build (executable's directory).
pub struct PortablePathManager;

impl PortablePathManager {
    /// Retrieves the application's root directory.
    ///
    /// In debug builds, this typically points to the `apps/core` crate root within the workspace.
    /// In release builds, it points to the directory containing the executable.
    /// This ensures that relative paths to assets (`data`, `models`, etc.) work consistently.
    ///
    /// # Returns
    ///
    /// A `PathBuf` to the calculated root directory.
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
            path
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

    /// Returns the path to the main data directory (`<root>/data`).
    pub fn data_dir() -> PathBuf {
        Self::root_dir().join("data")
    }

    /// Returns the path to the database directory (`<root>/data/db`).
    pub fn db_dir() -> PathBuf {
        Self::data_dir().join("db")
    }

    /// Returns the path to the AI models directory (`<root>/data/models`).
    pub fn models_dir() -> PathBuf {
        Self::data_dir().join("models")
    }

    /// Returns the path to the vector storage directory (`<root>/data/vectors`).
    pub fn vectors_dir() -> PathBuf {
        Self::data_dir().join("vectors")
    }

    /// Returns the path to the directory for a specific session's files.
    /// (`<root>/data/files/session_{session_id}`).
    #[allow(dead_code)]
    pub fn session_files_dir(session_id: &str) -> PathBuf {
        Self::data_dir()
            .join("files")
            .join(format!("session_{}", session_id))
    }

    /// Initializes the necessary application directories.
    ///
    /// This function ensures that the `data`, `db`, `models`, and `vectors` directories
    /// exist, creating them if they don't.
    ///
    /// # Returns
    ///
    /// A `Result` which is `Ok(())` on success, or an `std::io::Error` on failure.
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
