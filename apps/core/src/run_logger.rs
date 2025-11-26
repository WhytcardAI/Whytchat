//! Run Logger Module
//!
//! Provides automatic logging of application runs to a `run.log` file.
//! The logger tracks each application run with timestamps, status, and any errors.
//! It maintains only the last 10 runs to keep the log file manageable.
//!
//! ## Usage
//!
//! ```rust,ignore
//! use run_logger::RunLogger;
//!
//! // Start a new run at application startup
//! let mut logger = RunLogger::start_run();
//!
//! // Log errors during execution
//! logger.log_error("Something went wrong");
//!
//! // Complete the run when application exits (success or failure)
//! logger.complete_run(true); // or false for failure
//! ```

use crate::fs_manager::PortablePathManager;
use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::sync::Mutex;
use tracing::{error, info, warn};

/// Maximum number of runs to keep in the log file
const MAX_RUNS: usize = 10;

/// Name of the run log file
const RUN_LOG_FILENAME: &str = "run.log";

/// Represents a single application run entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunEntry {
    /// Unique identifier for the run (timestamp-based)
    pub run_id: String,
    /// Start time of the run
    pub start_time: DateTime<Local>,
    /// End time of the run (None if still running)
    pub end_time: Option<DateTime<Local>>,
    /// Status of the run: "running", "success", "failure"
    pub status: String,
    /// List of errors encountered during the run
    pub errors: Vec<ErrorEntry>,
    /// Additional info about the run
    pub info: Vec<String>,
}

/// Represents an error that occurred during a run
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorEntry {
    /// Time when the error occurred
    pub timestamp: DateTime<Local>,
    /// Error message
    pub message: String,
    /// Optional context or location of the error
    pub context: Option<String>,
}

/// Global run logger instance
static GLOBAL_LOGGER: Mutex<Option<RunLogger>> = Mutex::new(None);

/// Logger for tracking application runs
pub struct RunLogger {
    /// The current run entry being tracked
    current_run: RunEntry,
    /// Path to the run.log file
    log_path: PathBuf,
}

impl RunLogger {
    /// Creates a new RunLogger and starts tracking a new run.
    ///
    /// This initializes the logger, creates the run.log file if it doesn't exist,
    /// and adds a new run entry with status "running".
    pub fn start_run() -> Self {
        let now = Local::now();
        let run_id = format!("run_{}", now.format("%Y%m%d_%H%M%S"));

        let current_run = RunEntry {
            run_id: run_id.clone(),
            start_time: now,
            end_time: None,
            status: "running".to_string(),
            errors: Vec::new(),
            info: Vec::new(),
        };

        // Ensure logs directory exists
        let logs_dir = PortablePathManager::logs_dir();
        if let Err(e) = fs::create_dir_all(&logs_dir) {
            error!("Failed to create logs directory: {}", e);
        }

        let log_path = logs_dir.join(RUN_LOG_FILENAME);

        let mut logger = Self {
            current_run,
            log_path,
        };

        // Log the start of the run
        logger.log_info("Application started");
        logger.write_to_file();

        info!(
            "📝 Run logger started. Run ID: {}, Log file: {:?}",
            run_id, logger.log_path
        );

        logger
    }

    /// Initializes the global run logger.
    /// Call this at application startup.
    pub fn init_global() {
        let mut global = GLOBAL_LOGGER.lock().unwrap();
        if global.is_none() {
            *global = Some(Self::start_run());
        }
    }

    /// Gets the path to the run.log file.
    pub fn log_path(&self) -> &PathBuf {
        &self.log_path
    }

    /// Logs an informational message for the current run.
    pub fn log_info(&mut self, message: &str) {
        self.current_run
            .info
            .push(format!("[{}] {}", Local::now().format("%H:%M:%S"), message));
    }

    /// Logs an error for the current run.
    ///
    /// # Arguments
    ///
    /// * `message` - The error message
    pub fn log_error(&mut self, message: &str) {
        self.log_error_with_context(message, None);
    }

    /// Logs an error with optional context for the current run.
    ///
    /// # Arguments
    ///
    /// * `message` - The error message
    /// * `context` - Optional context or location of the error
    pub fn log_error_with_context(&mut self, message: &str, context: Option<&str>) {
        let error_entry = ErrorEntry {
            timestamp: Local::now(),
            message: message.to_string(),
            context: context.map(|s| s.to_string()),
        };
        self.current_run.errors.push(error_entry);
        self.write_to_file();
    }

    /// Logs an error to the global logger.
    /// Safe to call even if the logger isn't initialized.
    pub fn global_log_error(message: &str) {
        Self::global_log_error_with_context(message, None);
    }

    /// Logs an error with context to the global logger.
    pub fn global_log_error_with_context(message: &str, context: Option<&str>) {
        if let Ok(mut global) = GLOBAL_LOGGER.lock() {
            if let Some(logger) = global.as_mut() {
                logger.log_error_with_context(message, context);
            }
        }
    }

    /// Logs info to the global logger.
    pub fn global_log_info(message: &str) {
        if let Ok(mut global) = GLOBAL_LOGGER.lock() {
            if let Some(logger) = global.as_mut() {
                logger.log_info(message);
                logger.write_to_file();
            }
        }
    }

    /// Completes the current run and updates its status.
    ///
    /// # Arguments
    ///
    /// * `success` - Whether the run completed successfully
    pub fn complete_run(&mut self, success: bool) {
        self.current_run.end_time = Some(Local::now());
        self.current_run.status = if success {
            "success".to_string()
        } else {
            "failure".to_string()
        };

        let duration = self
            .current_run
            .end_time
            .unwrap()
            .signed_duration_since(self.current_run.start_time);

        self.log_info(&format!(
            "Application {} (duration: {}s)",
            if success { "exited normally" } else { "failed" },
            duration.num_seconds()
        ));

        self.write_to_file();

        info!(
            "📝 Run {} completed with status: {}",
            self.current_run.run_id, self.current_run.status
        );
    }

    /// Completes the global run logger.
    pub fn complete_global(success: bool) {
        if let Ok(mut global) = GLOBAL_LOGGER.lock() {
            if let Some(logger) = global.as_mut() {
                logger.complete_run(success);
            }
        }
    }

    /// Writes the current run to the log file, maintaining only the last MAX_RUNS entries.
    fn write_to_file(&self) {
        // Read existing runs
        let mut runs = self.read_existing_runs();

        // Find and update or add the current run
        let mut found = false;
        for run in &mut runs {
            if run.run_id == self.current_run.run_id {
                *run = self.current_run.clone();
                found = true;
                break;
            }
        }
        if !found {
            runs.push(self.current_run.clone());
        }

        // Keep only the last MAX_RUNS entries
        while runs.len() > MAX_RUNS {
            runs.remove(0);
        }

        // Write back to file
        if let Err(e) = self.write_runs_to_file(&runs) {
            error!("Failed to write run log: {}", e);
        }
    }

    /// Reads existing run entries from the log file.
    fn read_existing_runs(&self) -> Vec<RunEntry> {
        if !self.log_path.exists() {
            return Vec::new();
        }

        match fs::File::open(&self.log_path) {
            Ok(file) => {
                let reader = BufReader::new(file);
                let mut runs = Vec::new();

                for line in reader.lines() {
                    if let Ok(line) = line {
                        let trimmed = line.trim();
                        if trimmed.is_empty() || trimmed.starts_with('#') {
                            continue;
                        }
                        match serde_json::from_str::<RunEntry>(trimmed) {
                            Ok(entry) => runs.push(entry),
                            Err(e) => {
                                warn!("Failed to parse run entry: {} - line: {}", e, trimmed);
                            }
                        }
                    }
                }

                runs
            }
            Err(e) => {
                warn!("Failed to open run log file: {}", e);
                Vec::new()
            }
        }
    }

    /// Writes all run entries to the log file.
    fn write_runs_to_file(&self, runs: &[RunEntry]) -> std::io::Result<()> {
        let mut file = fs::File::create(&self.log_path)?;

        // Write header
        writeln!(
            file,
            "# WhytChat Run Log - Last {} runs",
            runs.len().min(MAX_RUNS)
        )?;
        writeln!(
            file,
            "# Generated: {}",
            Local::now().format("%Y-%m-%d %H:%M:%S")
        )?;
        writeln!(file, "# Each line is a JSON object representing one run")?;
        writeln!(file)?;

        // Write each run as a JSON line
        for run in runs {
            match serde_json::to_string(run) {
                Ok(json) => {
                    writeln!(file, "{}", json)?;
                }
                Err(e) => {
                    error!("Failed to serialize run entry: {}", e);
                }
            }
        }

        Ok(())
    }

    /// Returns the current run entry (for inspection/testing).
    pub fn current_run(&self) -> &RunEntry {
        &self.current_run
    }

    /// Reads and returns all run entries from the log file.
    /// Useful for displaying run history.
    pub fn get_run_history(&self) -> Vec<RunEntry> {
        self.read_existing_runs()
    }

    /// Gets the number of runs currently stored in the log.
    pub fn run_count(&self) -> usize {
        self.read_existing_runs().len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;
    use tempfile::TempDir;

    /// Helper to create a test logger with a custom path
    fn create_test_logger(temp_dir: &Path) -> RunLogger {
        let now = Local::now();
        let run_id = format!("test_run_{}", now.format("%Y%m%d_%H%M%S_%f"));

        let current_run = RunEntry {
            run_id,
            start_time: now,
            end_time: None,
            status: "running".to_string(),
            errors: Vec::new(),
            info: Vec::new(),
        };

        let log_path = temp_dir.join(RUN_LOG_FILENAME);

        RunLogger {
            current_run,
            log_path,
        }
    }

    #[test]
    fn test_create_run_entry() {
        let temp_dir = TempDir::new().unwrap();
        let logger = create_test_logger(temp_dir.path());

        assert!(logger.current_run.run_id.starts_with("test_run_"));
        assert_eq!(logger.current_run.status, "running");
        assert!(logger.current_run.errors.is_empty());
        assert!(logger.current_run.end_time.is_none());
    }

    #[test]
    fn test_log_error() {
        let temp_dir = TempDir::new().unwrap();
        let mut logger = create_test_logger(temp_dir.path());

        logger.log_error("Test error message");

        assert_eq!(logger.current_run.errors.len(), 1);
        assert_eq!(logger.current_run.errors[0].message, "Test error message");
        assert!(logger.current_run.errors[0].context.is_none());
    }

    #[test]
    fn test_log_error_with_context() {
        let temp_dir = TempDir::new().unwrap();
        let mut logger = create_test_logger(temp_dir.path());

        logger.log_error_with_context("Test error", Some("main.rs:42"));

        assert_eq!(logger.current_run.errors.len(), 1);
        assert_eq!(
            logger.current_run.errors[0].context,
            Some("main.rs:42".to_string())
        );
    }

    #[test]
    fn test_log_info() {
        let temp_dir = TempDir::new().unwrap();
        let mut logger = create_test_logger(temp_dir.path());

        logger.log_info("Test info message");

        assert_eq!(logger.current_run.info.len(), 1);
        assert!(logger.current_run.info[0].contains("Test info message"));
    }

    #[test]
    fn test_complete_run_success() {
        let temp_dir = TempDir::new().unwrap();
        let mut logger = create_test_logger(temp_dir.path());

        logger.complete_run(true);

        assert_eq!(logger.current_run.status, "success");
        assert!(logger.current_run.end_time.is_some());
    }

    #[test]
    fn test_complete_run_failure() {
        let temp_dir = TempDir::new().unwrap();
        let mut logger = create_test_logger(temp_dir.path());

        logger.log_error("Critical failure");
        logger.complete_run(false);

        assert_eq!(logger.current_run.status, "failure");
        assert!(logger.current_run.end_time.is_some());
        assert_eq!(logger.current_run.errors.len(), 1);
    }

    #[test]
    fn test_write_and_read_runs() {
        let temp_dir = TempDir::new().unwrap();
        let mut logger = create_test_logger(temp_dir.path());

        logger.log_info("Starting test");
        logger.log_error("Test error");
        logger.complete_run(true);

        // Re-read the file
        let runs = logger.read_existing_runs();

        assert_eq!(runs.len(), 1);
        assert_eq!(runs[0].status, "success");
        assert_eq!(runs[0].errors.len(), 1);
    }

    #[test]
    fn test_run_rotation() {
        let temp_dir = TempDir::new().unwrap();
        let log_path = temp_dir.path().join(RUN_LOG_FILENAME);

        // Create MAX_RUNS + 5 runs to test rotation
        for i in 0..(MAX_RUNS + 5) {
            let run_id = format!("run_{:03}", i);
            let run = RunEntry {
                run_id,
                start_time: Local::now(),
                end_time: Some(Local::now()),
                status: "success".to_string(),
                errors: Vec::new(),
                info: vec![format!("Run number {}", i)],
            };

            // Manually write runs to simulate multiple application starts
            let mut existing_runs = if log_path.exists() {
                let file = fs::File::open(&log_path).unwrap();
                let reader = BufReader::new(file);
                let mut runs = Vec::new();
                for line in reader.lines() {
                    if let Ok(line) = line {
                        let trimmed = line.trim();
                        if trimmed.is_empty() || trimmed.starts_with('#') {
                            continue;
                        }
                        if let Ok(entry) = serde_json::from_str::<RunEntry>(trimmed) {
                            runs.push(entry);
                        }
                    }
                }
                runs
            } else {
                Vec::new()
            };

            existing_runs.push(run);

            // Keep only last MAX_RUNS
            while existing_runs.len() > MAX_RUNS {
                existing_runs.remove(0);
            }

            // Write back
            let mut file = fs::File::create(&log_path).unwrap();
            writeln!(file, "# Test run log").unwrap();
            writeln!(file).unwrap();
            for run in &existing_runs {
                writeln!(file, "{}", serde_json::to_string(run).unwrap()).unwrap();
            }
        }

        // Read and verify
        let file = fs::File::open(&log_path).unwrap();
        let reader = BufReader::new(file);
        let mut runs = Vec::new();
        for line in reader.lines() {
            if let Ok(line) = line {
                let trimmed = line.trim();
                if trimmed.is_empty() || trimmed.starts_with('#') {
                    continue;
                }
                if let Ok(entry) = serde_json::from_str::<RunEntry>(trimmed) {
                    runs.push(entry);
                }
            }
        }

        assert_eq!(runs.len(), MAX_RUNS);
        // First run should be run_005 (since we added 15 runs and kept last 10)
        assert_eq!(runs[0].run_id, "run_005");
        // Last run should be run_014
        assert_eq!(runs[MAX_RUNS - 1].run_id, "run_014");
    }

    #[test]
    fn test_file_creation() {
        let temp_dir = TempDir::new().unwrap();
        let mut logger = create_test_logger(temp_dir.path());

        logger.write_to_file();

        assert!(logger.log_path.exists());
    }

    #[test]
    fn test_multiple_errors() {
        let temp_dir = TempDir::new().unwrap();
        let mut logger = create_test_logger(temp_dir.path());

        logger.log_error("Error 1");
        logger.log_error("Error 2");
        logger.log_error_with_context("Error 3", Some("context"));

        assert_eq!(logger.current_run.errors.len(), 3);
    }

    #[test]
    fn test_serialization_round_trip() {
        let entry = RunEntry {
            run_id: "test_run".to_string(),
            start_time: Local::now(),
            end_time: Some(Local::now()),
            status: "success".to_string(),
            errors: vec![ErrorEntry {
                timestamp: Local::now(),
                message: "Test error".to_string(),
                context: Some("test context".to_string()),
            }],
            info: vec!["Test info".to_string()],
        };

        let json = serde_json::to_string(&entry).unwrap();
        let parsed: RunEntry = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.run_id, entry.run_id);
        assert_eq!(parsed.status, entry.status);
        assert_eq!(parsed.errors.len(), 1);
        assert_eq!(parsed.info.len(), 1);
    }
}
