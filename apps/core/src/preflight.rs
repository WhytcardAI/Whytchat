//! Preflight Check System
//!
//! This module performs comprehensive health checks on all system components
//! before the application starts. No assumptions - everything is verified.

use crate::fs_manager::PortablePathManager;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tracing::{info, warn};

// --- Constants ---
const MIN_MODEL_SIZE_BYTES: u64 = 3 * 1024 * 1024 * 1024; // 3 GB minimum for GGUF
const MIN_EMBEDDINGS_SIZE_BYTES: u64 = 20 * 1024 * 1024;  // 20 MB minimum for ONNX
const LLAMA_SERVER_STARTUP_TIMEOUT: Duration = Duration::from_secs(30);
const LLAMA_SERVER_HEALTH_TIMEOUT: Duration = Duration::from_secs(5);

/// Result of a single check
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckResult {
    pub name: String,
    pub passed: bool,
    pub message: String,
    pub details: Option<String>,
}

impl CheckResult {
    fn pass(name: &str, message: &str) -> Self {
        Self {
            name: name.to_string(),
            passed: true,
            message: message.to_string(),
            details: None,
        }
    }

    fn fail(name: &str, message: &str, details: Option<String>) -> Self {
        Self {
            name: name.to_string(),
            passed: false,
            message: message.to_string(),
            details,
        }
    }
}

/// Complete preflight check report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreflightReport {
    pub all_passed: bool,
    pub checks: Vec<CheckResult>,
    pub ready_to_start: bool,
    pub needs_onboarding: bool,
    pub summary: String,
}

/// Performs all preflight checks and returns a comprehensive report
pub async fn run_preflight_checks() -> PreflightReport {
    info!("╔══════════════════════════════════════════════════════╗");
    info!("║  🔍 RUNNING PREFLIGHT CHECKS                         ║");
    info!("╚══════════════════════════════════════════════════════╝");

    let mut checks = Vec::new();

    // 1. Check directories
    checks.push(check_directories());

    // 2. Check model file
    let model_check = check_model_file();
    let model_exists = model_check.passed;
    checks.push(model_check);

    // 3. Check llama-server binary
    let server_check = check_llama_server_binary();
    let server_exists = server_check.passed;
    checks.push(server_check);

    // 4. Check embeddings
    let embeddings_check = check_embeddings();
    let embeddings_exist = embeddings_check.passed;
    checks.push(embeddings_check);

    // 5. Check database
    checks.push(check_database().await);

    // 6. Check LanceDB vectors directory
    checks.push(check_vectors_dir());

    // 7. Test llama-server can start (only if binary exists and model exists)
    if server_exists && model_exists {
        checks.push(check_llama_server_starts().await);
    } else {
        checks.push(CheckResult::fail(
            "llama_server_startup",
            "Skipped - missing binary or model",
            None,
        ));
    }

    // 8. Test embeddings can load (only if embeddings exist)
    if embeddings_exist {
        checks.push(check_embeddings_load().await);
    } else {
        checks.push(CheckResult::fail(
            "embeddings_load",
            "Skipped - embeddings not downloaded",
            None,
        ));
    }

    // Calculate results
    let all_passed = checks.iter().all(|c| c.passed);
    let critical_passed = checks.iter()
        .filter(|c| is_critical_check(&c.name))
        .all(|c| c.passed);

    // Determine if we need onboarding (missing model or server)
    let needs_onboarding = !model_exists || !server_exists;

    let summary = if all_passed {
        "All checks passed. System ready.".to_string()
    } else if needs_onboarding {
        "Model or server missing. Onboarding required.".to_string()
    } else if critical_passed {
        "Some non-critical checks failed. System can start with warnings.".to_string()
    } else {
        "Critical checks failed. System cannot start.".to_string()
    };

    // Log results
    for check in &checks {
        if check.passed {
            info!("  ✅ {}: {}", check.name, check.message);
        } else {
            warn!("  ❌ {}: {}", check.name, check.message);
            if let Some(details) = &check.details {
                warn!("      Details: {}", details);
            }
        }
    }

    info!("Summary: {}", summary);

    PreflightReport {
        all_passed,
        checks,
        ready_to_start: critical_passed && !needs_onboarding,
        needs_onboarding,
        summary,
    }
}

fn is_critical_check(name: &str) -> bool {
    matches!(name, "directories" | "model_file" | "llama_server_binary" | "database")
}

// --- Individual Checks ---

fn check_directories() -> CheckResult {
    let dirs = [
        ("data", PortablePathManager::data_dir()),
        ("models", PortablePathManager::models_dir()),
        ("db", PortablePathManager::db_dir()),
        ("vectors", PortablePathManager::vectors_dir()),
        ("tools", PortablePathManager::tools_dir()),
    ];

    let mut missing = Vec::new();
    let mut created = Vec::new();

    for (name, path) in &dirs {
        if !path.exists() {
            match std::fs::create_dir_all(path) {
                Ok(_) => created.push(*name),
                Err(e) => missing.push(format!("{}: {}", name, e)),
            }
        }
    }

    if missing.is_empty() {
        if created.is_empty() {
            CheckResult::pass("directories", "All directories exist")
        } else {
            CheckResult::pass(
                "directories",
                &format!("Created missing directories: {}", created.join(", ")),
            )
        }
    } else {
        CheckResult::fail(
            "directories",
            "Failed to create directories",
            Some(missing.join(", ")),
        )
    }
}

fn check_model_file() -> CheckResult {
    let model_path = PortablePathManager::models_dir().join("default-model.gguf");

    if !model_path.exists() {
        return CheckResult::fail(
            "model_file",
            "Model file not found",
            Some(format!("Expected at: {:?}", model_path)),
        );
    }

    match std::fs::metadata(&model_path) {
        Ok(meta) => {
            let size = meta.len();
            if size >= MIN_MODEL_SIZE_BYTES {
                CheckResult::pass(
                    "model_file",
                    &format!("Model OK ({:.2} GB)", size as f64 / 1024.0 / 1024.0 / 1024.0),
                )
            } else {
                CheckResult::fail(
                    "model_file",
                    "Model file incomplete (partial download)",
                    Some(format!(
                        "Size: {} bytes, minimum required: {} bytes",
                        size, MIN_MODEL_SIZE_BYTES
                    )),
                )
            }
        }
        Err(e) => CheckResult::fail(
            "model_file",
            "Cannot read model file",
            Some(e.to_string()),
        ),
    }
}

fn check_llama_server_binary() -> CheckResult {
    let server_name = if cfg!(windows) {
        "llama-server.exe"
    } else {
        "llama-server"
    };

    // Check in tools directory first
    let tools_path = PortablePathManager::tools_dir()
        .join("llama")
        .join(server_name);

    if tools_path.exists() {
        return CheckResult::pass(
            "llama_server_binary",
            &format!("Found at {:?}", tools_path),
        );
    }

    // Check in PATH
    if which::which(server_name).is_ok() {
        return CheckResult::pass("llama_server_binary", "Found in PATH");
    }

    CheckResult::fail(
        "llama_server_binary",
        "llama-server binary not found",
        Some(format!(
            "Expected at {:?} or in PATH",
            tools_path
        )),
    )
}

fn check_embeddings() -> CheckResult {
    let embeddings_dir = PortablePathManager::models_dir().join("embeddings");

    if !embeddings_dir.exists() {
        return CheckResult::fail(
            "embeddings",
            "Embeddings directory not found",
            Some(format!("Expected at: {:?}", embeddings_dir)),
        );
    }

    // Check for ONNX model files
    let mut total_size: u64 = 0;
    let mut onnx_found = false;

    if let Ok(entries) = std::fs::read_dir(&embeddings_dir) {
        for entry in entries.flatten() {
            if let Ok(meta) = entry.metadata() {
                total_size += meta.len();
                if entry.path().extension().map(|e| e == "onnx").unwrap_or(false) {
                    onnx_found = true;
                }
            }
            // Check subdirectories too
            if entry.path().is_dir() {
                if let Ok(sub_entries) = std::fs::read_dir(entry.path()) {
                    for sub_entry in sub_entries.flatten() {
                        if let Ok(meta) = sub_entry.metadata() {
                            total_size += meta.len();
                            if sub_entry.path().extension().map(|e| e == "onnx").unwrap_or(false) {
                                onnx_found = true;
                            }
                        }
                    }
                }
            }
        }
    }

    if total_size >= MIN_EMBEDDINGS_SIZE_BYTES && onnx_found {
        CheckResult::pass(
            "embeddings",
            &format!("Embeddings OK ({:.2} MB)", total_size as f64 / 1024.0 / 1024.0),
        )
    } else if total_size > 0 {
        CheckResult::fail(
            "embeddings",
            "Embeddings incomplete",
            Some(format!(
                "Size: {} bytes, ONNX found: {}",
                total_size, onnx_found
            )),
        )
    } else {
        CheckResult::fail(
            "embeddings",
            "Embeddings directory empty",
            None,
        )
    }
}

async fn check_database() -> CheckResult {
    let db_path = PortablePathManager::db_dir().join("whytchat.sqlite");

    if !db_path.exists() {
        // Database will be created on first init - this is OK
        return CheckResult::pass(
            "database",
            "Database will be created on first start",
        );
    }

    // Try to open and query the database
    let db_url = format!("sqlite://{}", db_path.to_string_lossy());

    match sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(1)
        .connect(&db_url)
        .await
    {
        Ok(pool) => {
            // Check tables exist
            let tables_result = sqlx::query_scalar::<_, String>(
                "SELECT name FROM sqlite_master WHERE type='table' ORDER BY name"
            )
            .fetch_all(&pool)
            .await;

            match tables_result {
                Ok(tables) => {
                    let required_tables = ["sessions", "messages", "folders", "session_files"];
                    let missing: Vec<&str> = required_tables
                        .iter()
                        .filter(|t| !tables.contains(&t.to_string()))
                        .copied()
                        .collect();

                    if missing.is_empty() {
                        CheckResult::pass(
                            "database",
                            &format!("Database OK ({} tables)", tables.len()),
                        )
                    } else {
                        CheckResult::fail(
                            "database",
                            "Missing tables (migrations needed)",
                            Some(format!("Missing: {}", missing.join(", "))),
                        )
                    }
                }
                Err(e) => CheckResult::fail(
                    "database",
                    "Cannot query database",
                    Some(e.to_string()),
                ),
            }
        }
        Err(e) => CheckResult::fail(
            "database",
            "Cannot connect to database",
            Some(e.to_string()),
        ),
    }
}

fn check_vectors_dir() -> CheckResult {
    let vectors_dir = PortablePathManager::vectors_dir();

    if !vectors_dir.exists() {
        return CheckResult::pass(
            "vectors_dir",
            "Vectors directory will be created when needed",
        );
    }

    // Check if LanceDB files exist
    let has_lance_files = std::fs::read_dir(&vectors_dir)
        .map(|entries| {
            entries.flatten().any(|e| {
                e.path().is_dir() || e.path().extension().map(|ext| ext == "lance").unwrap_or(false)
            })
        })
        .unwrap_or(false);

    if has_lance_files {
        CheckResult::pass("vectors_dir", "LanceDB data found")
    } else {
        CheckResult::pass("vectors_dir", "Vectors directory empty (OK for new install)")
    }
}

async fn check_llama_server_starts() -> CheckResult {
    let server_name = if cfg!(windows) {
        "llama-server.exe"
    } else {
        "llama-server"
    };

    let server_path = PortablePathManager::tools_dir()
        .join("llama")
        .join(server_name);

    let model_path = PortablePathManager::models_dir().join("default-model.gguf");

    // Use a different port to avoid conflicts
    let test_port = 18080;

    info!("Testing llama-server startup on port {}...", test_port);

    let mut cmd = tokio::process::Command::new(&server_path);
    cmd.arg("-m")
        .arg(&model_path)
        .arg("--host")
        .arg("127.0.0.1")
        .arg("--port")
        .arg(test_port.to_string())
        .arg("-c")
        .arg("512")  // Small context for quick test
        .arg("-ngl")
        .arg("0")    // CPU only for quick test
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null());

    let child = match cmd.spawn() {
        Ok(c) => c,
        Err(e) => {
            return CheckResult::fail(
                "llama_server_startup",
                "Cannot spawn llama-server",
                Some(e.to_string()),
            );
        }
    };

    let pid = child.id();

    // Wait for health check
    let client = reqwest::Client::new();
    let health_url = format!("http://127.0.0.1:{}/health", test_port);

    let start = std::time::Instant::now();
    let mut server_ready = false;

    while start.elapsed() < LLAMA_SERVER_STARTUP_TIMEOUT {
        tokio::time::sleep(Duration::from_millis(500)).await;

        match tokio::time::timeout(
            LLAMA_SERVER_HEALTH_TIMEOUT,
            client.get(&health_url).send(),
        )
        .await
        {
            Ok(Ok(response)) if response.status().is_success() => {
                server_ready = true;
                break;
            }
            _ => continue,
        }
    }

    // Kill the test server
    if let Some(pid) = pid {
        #[cfg(windows)]
        {
            let _ = std::process::Command::new("taskkill")
                .args(["/F", "/PID", &pid.to_string()])
                .output();
        }
        #[cfg(not(windows))]
        {
            let _ = std::process::Command::new("kill")
                .args(["-9", &pid.to_string()])
                .output();
        }
    }

    if server_ready {
        CheckResult::pass(
            "llama_server_startup",
            &format!("Server started successfully in {:?}", start.elapsed()),
        )
    } else {
        CheckResult::fail(
            "llama_server_startup",
            "Server failed to start within timeout",
            Some(format!("Timeout: {:?}", LLAMA_SERVER_STARTUP_TIMEOUT)),
        )
    }
}

async fn check_embeddings_load() -> CheckResult {
    let embeddings_dir = PortablePathManager::models_dir().join("embeddings");

    info!("Testing FastEmbed model loading...");

    let result = tokio::task::spawn_blocking(move || {
        use fastembed::{EmbeddingModel, InitOptions, TextEmbedding};

        let start = std::time::Instant::now();

        let mut options = InitOptions::new(EmbeddingModel::AllMiniLML6V2);
        options.show_download_progress = false;
        options.cache_dir = embeddings_dir;

        match TextEmbedding::try_new(options) {
            Ok(model) => {
                // Try to embed a test sentence
                match model.embed(vec!["test".to_string()], None) {
                    Ok(embeddings) => {
                        if embeddings.len() == 1 && embeddings[0].len() == 384 {
                            Ok(start.elapsed())
                        } else {
                            Err(format!(
                                "Unexpected embedding shape: {} vectors, {} dimensions",
                                embeddings.len(),
                                embeddings.first().map(|v| v.len()).unwrap_or(0)
                            ))
                        }
                    }
                    Err(e) => Err(format!("Embedding failed: {}", e)),
                }
            }
            Err(e) => Err(format!("Model load failed: {}", e)),
        }
    })
    .await;

    match result {
        Ok(Ok(duration)) => CheckResult::pass(
            "embeddings_load",
            &format!("FastEmbed loaded in {:?}", duration),
        ),
        Ok(Err(e)) => CheckResult::fail("embeddings_load", "FastEmbed failed", Some(e)),
        Err(e) => CheckResult::fail(
            "embeddings_load",
            "Task panicked",
            Some(e.to_string()),
        ),
    }
}

/// Quick check - just verifies files exist, no startup tests
pub fn quick_preflight_check() -> PreflightReport {
    let mut checks = Vec::new();

    checks.push(check_directories());
    checks.push(check_model_file());
    checks.push(check_llama_server_binary());
    checks.push(check_embeddings());
    checks.push(check_vectors_dir());

    let model_ok = checks.iter().find(|c| c.name == "model_file").map(|c| c.passed).unwrap_or(false);
    let server_ok = checks.iter().find(|c| c.name == "llama_server_binary").map(|c| c.passed).unwrap_or(false);

    let all_passed = checks.iter().all(|c| c.passed);
    let needs_onboarding = !model_ok || !server_ok;

    let summary = if all_passed {
        "Quick check passed".to_string()
    } else if needs_onboarding {
        "Onboarding required".to_string()
    } else {
        "Some checks failed".to_string()
    };

    PreflightReport {
        all_passed,
        checks,
        ready_to_start: !needs_onboarding,
        needs_onboarding,
        summary,
    }
}
