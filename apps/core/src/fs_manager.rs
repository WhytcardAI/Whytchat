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

    /// Check if running in portable mode (marker file exists next to executable)
    /// Only used in release builds - in debug mode we always use workspace root
    #[cfg(not(debug_assertions))]
    fn is_portable_mode() -> bool {
        if let Ok(exe_path) = std::env::current_exe() {
            let marker = exe_path.parent().map(|p| p.join("portable.marker"));
            marker.map(|m| m.exists()).unwrap_or(false)
        } else {
            false
        }
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
    /// 2. Portable mode: executable directory (if `portable.marker` exists)
    /// 3. Default: User's local app data folder
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
                // In the monorepo structure, we return the workspace root.
                // This ensures that 'data' and 'tools' directories are created at the workspace root,
                // OUTSIDE of 'apps/core'.
                // This is critical because 'apps/core' is watched by Tauri/Cargo in dev mode.
                // Writing large files (models) inside a watched directory triggers a rebuild/restart,
                // causing the download to fail and the app to loop.
                return path;
            }

            // Fallback: maybe we are already in apps/core (if target was local)
            path
        }

        #[cfg(not(debug_assertions))]
        {
            use tracing::error;

            // Priority 1: Portable mode (marker file next to executable)
            if Self::is_portable_mode() {
                if let Some(exe_dir) = Self::exe_dir() {
                    info!("Running in PORTABLE mode from: {:?}", exe_dir);
                    return exe_dir;
                }
            }

            // Priority 2: User's local app data (default for installed apps)
            #[cfg(target_os = "windows")]
            if let Ok(local_app_data) = std::env::var("LOCALAPPDATA") {
                let path = PathBuf::from(local_app_data).join("WhytChat");
                if !path.exists() {
                    let _ = std::fs::create_dir_all(&path);
                }
                info!("Using LOCALAPPDATA path: {:?}", path);
                return path;
            }

            #[cfg(not(target_os = "windows"))]
            if let Ok(home) = std::env::var("HOME") {
                let path = PathBuf::from(home)
                    .join(".local")
                    .join("share")
                    .join("whytchat");
                if !path.exists() {
                    let _ = std::fs::create_dir_all(&path);
                }
                return path;
            }

            // Fallback to executable directory
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

    /// Returns the path to the logs directory (`<root>/data/logs`).
    pub fn logs_dir() -> PathBuf {
        Self::data_dir().join("logs")
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
        let logs_path = Self::logs_dir();

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

        if !logs_path.exists() {
            info!("Creating logs directory: {:?}", logs_path);
            fs::create_dir_all(&logs_path)?;
        }

        Ok(())
    }
}
