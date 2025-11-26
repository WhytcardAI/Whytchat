use std::fs;
use std::path::PathBuf;
use std::sync::OnceLock;
use tracing::info;

/// Global storage for custom data path (set during onboarding or from config)
static CUSTOM_DATA_PATH: OnceLock<PathBuf> = OnceLock::new();

/// Manages application paths to ensure portability between development and release builds.
///
/// This struct provides a centralized way to access key directories, abstracting away the
/// differences in file structure between a debug environment (workspace root) and a
/// release build (executable's directory).
///
/// ## Portable Mode
/// In release builds, the app checks for a `portable.marker` file next to the executable.
/// If found, all data is stored relative to the executable (USB/portable mode).
/// Otherwise, data goes to the user's local app data folder.
pub struct PortablePathManager;

impl PortablePathManager {
    /// Sets a custom data path (called during onboarding if user chooses custom location)
    /// This can only be set once per application run.
    #[allow(dead_code)]
    pub fn set_custom_path(path: PathBuf) -> Result<(), PathBuf> {
        CUSTOM_DATA_PATH.set(path)
    }

    /// Gets the custom data path if set
    #[allow(dead_code)]
    pub fn get_custom_path() -> Option<&'static PathBuf> {
        CUSTOM_DATA_PATH.get()
    }

    /// Gets the executable's directory
    /// Only used in release builds - in debug mode we always use workspace root
    #[cfg(not(debug_assertions))]
    fn exe_dir() -> Option<PathBuf> {
        std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|p| p.to_path_buf()))
    }

    /// Retrieves the application's root directory.
    ///
    /// Priority order in Release mode:
    /// 1. Custom path set via `set_custom_path()` (user choice during onboarding)
    /// 2. ALWAYS use the executable directory (Portable/Self-contained mode)
    ///
    /// # Returns
    ///
    /// A `PathBuf` to the calculated root directory.
    pub fn root_dir() -> PathBuf {
        // Check for custom path first (highest priority)
        if let Some(custom) = CUSTOM_DATA_PATH.get() {
            return custom.clone();
        }

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
                return path;
            }
            path
        }

        #[cfg(not(debug_assertions))]
        {
            use tracing::error;

            // In Release mode, we ALWAYS use the executable directory.
            // This ensures all data (models, db, vectors) stays within the installation folder.
            // NOTE: If installed to Program Files, the app must be run as Admin to write data.
            if let Some(exe_dir) = Self::exe_dir() {
                info!("Using Installation Directory as Root: {:?}", exe_dir);
                return exe_dir;
            }

            // Fallback (should rarely happen)
            match std::env::current_exe() {
                Ok(mut path) => {
                    path.pop(); // Remove executable name
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

    /// Returns the path to the tools directory (`<root>/tools`).
    pub fn tools_dir() -> PathBuf {
        Self::root_dir().join("tools")
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
        let tools_path = Self::tools_dir();

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

        if !tools_path.exists() {
            info!("Creating tools directory: {:?}", tools_path);
            fs::create_dir_all(&tools_path)?;
        }

        Ok(())
    }
}
