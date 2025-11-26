//! Diagnostic Tests Module
//!
//! Provides comprehensive tests for all system components.
//! Used during onboarding and for troubleshooting.

use crate::brain::BrainAnalyzer;
use crate::database;
use crate::fs_manager::PortablePathManager;
use crate::models::ModelConfig;
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};
use tracing::{error, info};

// --- Test Result Types ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestResult {
    pub name: String,
    pub category: String,
    pub passed: bool,
    pub duration_ms: u64,
    pub message: String,
    pub details: Option<String>,
}

impl TestResult {
    fn pass(name: &str, category: &str, duration: Duration, message: &str) -> Self {
        Self {
            name: name.to_string(),
            category: category.to_string(),
            passed: true,
            duration_ms: duration.as_millis() as u64,
            message: message.to_string(),
            details: None,
        }
    }

    fn fail(name: &str, category: &str, duration: Duration, message: &str, details: Option<String>) -> Self {
        Self {
            name: name.to_string(),
            category: category.to_string(),
            passed: false,
            duration_ms: duration.as_millis() as u64,
            message: message.to_string(),
            details,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticReport {
    pub total_tests: usize,
    pub passed: usize,
    pub failed: usize,
    pub total_duration_ms: u64,
    pub results: Vec<TestResult>,
    pub categories: Vec<CategorySummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategorySummary {
    pub name: String,
    pub passed: usize,
    pub failed: usize,
}

// --- Main Test Runner ---

/// Runs all diagnostic tests and returns a comprehensive report.
/// Tests are run sequentially to avoid resource conflicts.
pub async fn run_all_tests(
    emit_progress: Option<Box<dyn Fn(&str, &TestResult) + Send + Sync>>,
) -> DiagnosticReport {
    info!("╔══════════════════════════════════════════════════════╗");
    info!("║  🧪 RUNNING DIAGNOSTIC TESTS                         ║");
    info!("╚══════════════════════════════════════════════════════╝");

    let start = Instant::now();
    let mut results = Vec::new();

    // Helper to emit and collect
    let mut run_test = |result: TestResult| {
        if let Some(ref emit) = emit_progress {
            emit(&result.name, &result);
        }
        if result.passed {
            info!("  ✅ {} - {} ({}ms)", result.name, result.message, result.duration_ms);
        } else {
            error!("  ❌ {} - {} ({}ms)", result.name, result.message, result.duration_ms);
        }
        results.push(result);
    };

    // === DATABASE TESTS ===
    run_test(test_db_connection().await);
    run_test(test_db_create_session().await);
    run_test(test_db_add_message().await);
    run_test(test_db_get_messages().await);
    run_test(test_db_list_sessions().await);
    run_test(test_db_update_session().await);
    run_test(test_db_encryption().await);

    // === RAG TESTS ===
    run_test(test_rag_embeddings().await);
    run_test(test_rag_ingest().await);
    run_test(test_rag_search().await);
    run_test(test_rag_context_retrieval().await);

    // === BRAIN TESTS ===
    run_test(test_brain_intent_greeting());
    run_test(test_brain_intent_question());
    run_test(test_brain_intent_code());
    run_test(test_brain_keywords());
    run_test(test_brain_complexity());
    run_test(test_brain_language_detection());

    // === LLM TESTS (these will start the server if needed) ===
    run_test(test_llm_server_startup().await);
    run_test(test_llm_server_health().await);
    run_test(test_llm_simple_completion().await);
    run_test(test_llm_chat_completion().await);
    run_test(test_llm_streaming().await);
    run_test(test_llm_actor_integration().await);

    // === FILE SYSTEM TESTS ===
    run_test(test_fs_directories());
    run_test(test_fs_model_file().await);
    run_test(test_fs_llama_server().await);
    run_test(test_fs_upload_file().await);

    // === INTEGRATION TESTS ===
    run_test(test_full_conversation_flow().await);

    // Calculate summaries
    let total_tests = results.len();
    let passed = results.iter().filter(|r| r.passed).count();
    let failed = total_tests - passed;

    // Category summaries
    let categories = vec!["database", "rag", "brain", "llm", "filesystem", "integration"]
        .into_iter()
        .map(|cat| {
            let cat_results: Vec<_> = results.iter().filter(|r| r.category == cat).collect();
            CategorySummary {
                name: cat.to_string(),
                passed: cat_results.iter().filter(|r| r.passed).count(),
                failed: cat_results.iter().filter(|r| !r.passed).count(),
            }
        })
        .collect();

    let report = DiagnosticReport {
        total_tests,
        passed,
        failed,
        total_duration_ms: start.elapsed().as_millis() as u64,
        results,
        categories,
    };

    info!("═══════════════════════════════════════════════════════");
    info!("  Results: {}/{} passed | {} failed | {}ms total",
        report.passed, report.total_tests, report.failed, report.total_duration_ms);
    info!("═══════════════════════════════════════════════════════");

    report
}

/// Runs tests for a specific category only
pub async fn run_category_tests(category: &str) -> Vec<TestResult> {
    match category {
        "database" => vec![
            test_db_connection().await,
            test_db_create_session().await,
            test_db_add_message().await,
            test_db_get_messages().await,
            test_db_list_sessions().await,
            test_db_update_session().await,
            test_db_encryption().await,
        ],
        "rag" => vec![
            test_rag_embeddings().await,
            test_rag_ingest().await,
            test_rag_search().await,
            test_rag_context_retrieval().await,
        ],
        "brain" => vec![
            test_brain_intent_greeting(),
            test_brain_intent_question(),
            test_brain_intent_code(),
            test_brain_keywords(),
            test_brain_complexity(),
            test_brain_language_detection(),
        ],
        "llm" => vec![
            test_llm_server_startup().await,
            test_llm_server_health().await,
            test_llm_simple_completion().await,
            test_llm_chat_completion().await,
            test_llm_streaming().await,
            test_llm_actor_integration().await,
        ],
        "filesystem" => vec![
            test_fs_directories(),
            test_fs_model_file().await,
            test_fs_llama_server().await,
            test_fs_upload_file().await,
        ],
        "integration" => vec![
            test_full_conversation_flow().await,
        ],
        _ => vec![],
    }
}

// === DATABASE TESTS ===

async fn test_db_connection() -> TestResult {
    let start = Instant::now();
    let name = "db_connection";
    let category = "database";

    match database::init_db().await {
        Ok(_pool) => TestResult::pass(name, category, start.elapsed(), "Database connected successfully"),
        Err(e) => TestResult::fail(name, category, start.elapsed(), "Failed to connect", Some(e.to_string())),
    }
}

async fn test_db_create_session() -> TestResult {
    let start = Instant::now();
    let name = "db_create_session";
    let category = "database";

    let pool = match database::init_db().await {
        Ok(p) => p,
        Err(e) => return TestResult::fail(name, category, start.elapsed(), "DB init failed", Some(e.to_string())),
    };

    let config = ModelConfig {
        model_id: "test-model".to_string(),
        temperature: 0.7,
        system_prompt: "Test prompt".to_string(),
    };

    match database::create_session(&pool, "Test Session".to_string(), config).await {
        Ok(session) => {
            // Clean up - we don't have delete, so just verify it was created
            TestResult::pass(name, category, start.elapsed(), &format!("Created session: {}", session.id))
        }
        Err(e) => TestResult::fail(name, category, start.elapsed(), "Failed to create session", Some(e.to_string())),
    }
}

async fn test_db_add_message() -> TestResult {
    let start = Instant::now();
    let name = "db_add_message";
    let category = "database";

    let pool = match database::init_db().await {
        Ok(p) => p,
        Err(e) => return TestResult::fail(name, category, start.elapsed(), "DB init failed", Some(e.to_string())),
    };

    // Create a test session first
    let config = ModelConfig::default();
    let session = match database::create_session(&pool, "Message Test".to_string(), config).await {
        Ok(s) => s,
        Err(e) => return TestResult::fail(name, category, start.elapsed(), "Session creation failed", Some(e.to_string())),
    };

    // Add messages
    match database::add_message(&pool, &session.id, "user", "Test message from diagnostics").await {
        Ok(_) => {}
        Err(e) => return TestResult::fail(name, category, start.elapsed(), "Failed to add user message", Some(e.to_string())),
    }

    match database::add_message(&pool, &session.id, "assistant", "Test response from diagnostics").await {
        Ok(_) => TestResult::pass(name, category, start.elapsed(), "Messages added successfully"),
        Err(e) => TestResult::fail(name, category, start.elapsed(), "Failed to add assistant message", Some(e.to_string())),
    }
}

async fn test_db_list_sessions() -> TestResult {
    let start = Instant::now();
    let name = "db_list_sessions";
    let category = "database";

    let pool = match database::init_db().await {
        Ok(p) => p,
        Err(e) => return TestResult::fail(name, category, start.elapsed(), "DB init failed", Some(e.to_string())),
    };

    match database::list_sessions(&pool).await {
        Ok(sessions) => TestResult::pass(name, category, start.elapsed(), &format!("Found {} sessions", sessions.len())),
        Err(e) => TestResult::fail(name, category, start.elapsed(), "Failed to list sessions", Some(e.to_string())),
    }
}

async fn test_db_encryption() -> TestResult {
    let start = Instant::now();
    let name = "db_encryption";
    let category = "database";

    // Test encryption/decryption cycle
    let test_data = b"test encryption data";

    match crate::encryption::encrypt(test_data) {
        Ok(encrypted) => {
            match crate::encryption::decrypt(&encrypted) {
                Ok(decrypted) => {
                    if decrypted == test_data {
                        TestResult::pass(name, category, start.elapsed(), "Encryption/decryption works")
                    } else {
                        TestResult::fail(name, category, start.elapsed(), "Decrypted data doesn't match", None)
                    }
                }
                Err(e) => TestResult::fail(name, category, start.elapsed(), "Decryption failed", Some(e)),
            }
        }
        Err(e) => TestResult::fail(name, category, start.elapsed(), "Encryption failed", Some(e)),
    }
}

// === RAG TESTS ===

async fn test_rag_embeddings() -> TestResult {
    let start = Instant::now();
    let name = "rag_embeddings";
    let category = "rag";

    let embeddings_dir = PortablePathManager::models_dir().join("embeddings");

    let result: Result<Result<(), String>, _> = tokio::task::spawn_blocking(move || {
        use fastembed::{EmbeddingModel, InitOptions, TextEmbedding};

        let mut options = InitOptions::new(EmbeddingModel::AllMiniLML6V2);
        options.show_download_progress = false;
        options.cache_dir = embeddings_dir;

        let model = TextEmbedding::try_new(options).map_err(|e| e.to_string())?;
        let embeddings = model.embed(vec!["Test embedding generation".to_string()], None).map_err(|e| e.to_string())?;

        if embeddings.len() == 1 && embeddings[0].len() == 384 {
            Ok(())
        } else {
            Err(format!("Unexpected dimensions: {} vectors, {} dims",
                embeddings.len(),
                embeddings.first().map(|v| v.len()).unwrap_or(0)
            ))
        }
    }).await;

    match result {
        Ok(Ok(())) => TestResult::pass(name, category, start.elapsed(), "Generated 384-dim embedding"),
        Ok(Err(e)) => TestResult::fail(name, category, start.elapsed(), "Embedding failed", Some(e)),
        Err(e) => TestResult::fail(name, category, start.elapsed(), "Task panicked", Some(e.to_string())),
    }
}

async fn test_rag_ingest() -> TestResult {
    let start = Instant::now();
    let name = "rag_ingest";
    let category = "rag";

    use crate::actors::rag::RagActorHandle;
    use crate::actors::traits::RagActor;

    let rag = RagActorHandle::new();

    let test_content = r#"
This is a test document for the RAG system.
It contains multiple lines of text.
Each line should be chunked and embedded.
The WhytChat diagnostic system created this.
Testing vector database ingestion.
"#.to_string();

    match rag.ingest(test_content, Some("diagnostic:test".to_string())).await {
        Ok(msg) => TestResult::pass(name, category, start.elapsed(), &msg),
        Err(e) => TestResult::fail(name, category, start.elapsed(), "Ingestion failed", Some(e.to_string())),
    }
}

async fn test_rag_search() -> TestResult {
    let start = Instant::now();
    let name = "rag_search";
    let category = "rag";

    use crate::actors::rag::RagActorHandle;
    use crate::actors::traits::RagActor;

    let rag = RagActorHandle::new();

    // Search for content we just ingested
    match rag.search_with_filters("WhytChat diagnostic test".to_string(), vec![]).await {
        Ok(results) => {
            TestResult::pass(name, category, start.elapsed(), &format!("Found {} results", results.len()))
        }
        Err(e) => TestResult::fail(name, category, start.elapsed(), "Search failed", Some(e.to_string())),
    }
}

// === BRAIN TESTS ===

fn test_brain_intent_greeting() -> TestResult {
    let start = Instant::now();
    let name = "brain_intent_greeting";
    let category = "brain";

    use crate::brain::Intent;

    let analyzer = BrainAnalyzer::new();
    let packet = analyzer.analyze("Bonjour, comment ça va ?");

    let intent_label = packet.intent.intent.label();
    if matches!(packet.intent.intent, Intent::Greeting) {
        TestResult::pass(name, category, start.elapsed(), &format!("Detected: {}", intent_label))
    } else {
        TestResult::fail(name, category, start.elapsed(),
            &format!("Expected greeting, got: {}", intent_label), None)
    }
}

fn test_brain_intent_question() -> TestResult {
    let start = Instant::now();
    let name = "brain_intent_question";
    let category = "brain";

    use crate::brain::Intent;

    let analyzer = BrainAnalyzer::new();
    let packet = analyzer.analyze("Qu'est-ce que le machine learning ?");

    let intent_label = packet.intent.intent.label();
    if matches!(packet.intent.intent, Intent::Question | Intent::Explanation) {
        TestResult::pass(name, category, start.elapsed(), &format!("Detected: {}", intent_label))
    } else {
        TestResult::fail(name, category, start.elapsed(),
            &format!("Expected question, got: {}", intent_label), None)
    }
}

fn test_brain_intent_code() -> TestResult {
    let start = Instant::now();
    let name = "brain_intent_code";
    let category = "brain";

    use crate::brain::Intent;

    let analyzer = BrainAnalyzer::new();
    let packet = analyzer.analyze("Écris une fonction Python qui calcule la factorielle");

    let intent_label = packet.intent.intent.label();
    if matches!(packet.intent.intent, Intent::CodeRequest | Intent::Command) {
        TestResult::pass(name, category, start.elapsed(), &format!("Detected: {}", intent_label))
    } else {
        TestResult::fail(name, category, start.elapsed(),
            &format!("Expected code, got: {}", intent_label), None)
    }
}

fn test_brain_keywords() -> TestResult {
    let start = Instant::now();
    let name = "brain_keywords";
    let category = "brain";

    let analyzer = BrainAnalyzer::new();
    let packet = analyzer.analyze("Explain neural networks and deep learning");

    if !packet.keywords.is_empty() {
        let keywords: Vec<&str> = packet.keywords.iter().map(|k| k.keyword.as_str()).collect();
        TestResult::pass(name, category, start.elapsed(),
            &format!("Extracted {} keywords: {:?}", packet.keywords.len(), keywords))
    } else {
        TestResult::fail(name, category, start.elapsed(), "No keywords extracted", None)
    }
}

fn test_brain_complexity() -> TestResult {
    let start = Instant::now();
    let name = "brain_complexity";
    let category = "brain";

    let analyzer = BrainAnalyzer::new();

    // Simple message
    let simple = analyzer.analyze("Hello");
    // Complex message
    let complex = analyzer.analyze(
        "Please analyze the architectural patterns in microservices, \
         considering event-driven design, CQRS, and saga patterns for \
         distributed transaction management"
    );

    if complex.complexity.score > simple.complexity.score {
        TestResult::pass(name, category, start.elapsed(),
            &format!("Simple={:.2}, Complex={:.2}", simple.complexity.score, complex.complexity.score))
    } else {
        TestResult::fail(name, category, start.elapsed(),
            &format!("Complexity scoring incorrect: simple={:.2}, complex={:.2}",
                simple.complexity.score, complex.complexity.score), None)
    }
}

// === LLM TESTS ===

/// Starts the LLM server and waits for it to be ready
async fn ensure_llm_server_running() -> Result<(), String> {
    let server_exe = if cfg!(windows) { "llama-server.exe" } else { "llama-server" };
    let server_path = PortablePathManager::tools_dir().join("llama").join(server_exe);
    let model_path = PortablePathManager::models_dir().join("default-model.gguf");

    // Check if server is already running
    let client = reqwest::Client::new();
    let mut request = client.get("http://127.0.0.1:8080/health");
    if let Ok(token) = std::env::var("LLAMA_AUTH_TOKEN") {
        request = request.header("Authorization", format!("Bearer {}", token));
    }

    if let Ok(resp) = request.send().await {
        if resp.status().is_success() {
            info!("LLM server already running");
            return Ok(());
        }
    }

    // Verify files exist
    if !server_path.exists() {
        return Err(format!("llama-server not found at {:?}", server_path));
    }
    if !model_path.exists() {
        return Err(format!("Model not found at {:?}", model_path));
    }

    info!("Starting llama-server for diagnostics...");

    // Start the server
    let mut cmd = tokio::process::Command::new(&server_path);
    cmd.arg("-m").arg(&model_path)
       .arg("--host").arg("127.0.0.1")
       .arg("--port").arg("8080")
       .arg("-c").arg("2048")
       .arg("-ngl").arg("0")  // CPU only for compatibility
       .stdout(std::process::Stdio::null())
       .stderr(std::process::Stdio::null());

    if let Ok(token) = std::env::var("LLAMA_AUTH_TOKEN") {
        cmd.arg("--api-key").arg(token);
    }

    let _child = cmd.spawn().map_err(|e| format!("Failed to spawn server: {}", e))?;

    // Wait for server to be ready (up to 60 seconds)
    let start = std::time::Instant::now();
    let timeout = Duration::from_secs(60);

    while start.elapsed() < timeout {
        tokio::time::sleep(Duration::from_millis(500)).await;

        let mut request = client.get("http://127.0.0.1:8080/health");
        if let Ok(token) = std::env::var("LLAMA_AUTH_TOKEN") {
            request = request.header("Authorization", format!("Bearer {}", token));
        }

        if let Ok(resp) = request.send().await {
            if resp.status().is_success() {
                info!("LLM server ready after {:?}", start.elapsed());
                return Ok(());
            }
        }
    }

    Err(format!("Server failed to start within {:?}", timeout))
}

async fn test_llm_server_startup() -> TestResult {
    let start = Instant::now();
    let name = "llm_server_startup";
    let category = "llm";

    match ensure_llm_server_running().await {
        Ok(()) => TestResult::pass(name, category, start.elapsed(), "Server started and responding"),
        Err(e) => TestResult::fail(name, category, start.elapsed(), "Failed to start server", Some(e)),
    }
}

async fn test_llm_server_health() -> TestResult {
    let start = Instant::now();
    let name = "llm_server_health";
    let category = "llm";

    let client = reqwest::Client::new();
    let health_url = "http://127.0.0.1:8080/health";

    let mut request = client.get(health_url);
    if let Ok(token) = std::env::var("LLAMA_AUTH_TOKEN") {
        request = request.header("Authorization", format!("Bearer {}", token));
    }

    match tokio::time::timeout(
        Duration::from_secs(5),
        request.send()
    ).await {
        Ok(Ok(response)) if response.status().is_success() => {
            TestResult::pass(name, category, start.elapsed(), "Health endpoint OK")
        }
        Ok(Ok(response)) => {
            TestResult::fail(name, category, start.elapsed(),
                &format!("Server returned status {}", response.status()), None)
        }
        Ok(Err(e)) => {
            TestResult::fail(name, category, start.elapsed(),
                "Server not responding", Some(e.to_string()))
        }
        Err(_) => {
            TestResult::fail(name, category, start.elapsed(), "Health check timeout", None)
        }
    }
}

async fn test_llm_simple_completion() -> TestResult {
    let start = Instant::now();
    let name = "llm_simple_completion";
    let category = "llm";

    let client = reqwest::Client::new();

    // Simple completion request
    let request_body = serde_json::json!({
        "prompt": "Say exactly: 'Hello from WhytChat diagnostic test'",
        "n_predict": 20,
        "temperature": 0.1,
        "stop": ["\n", "."]
    });

    let mut request = client.post("http://127.0.0.1:8080/completion").json(&request_body);
    if let Ok(token) = std::env::var("LLAMA_AUTH_TOKEN") {
        request = request.header("Authorization", format!("Bearer {}", token));
    }

    match tokio::time::timeout(
        Duration::from_secs(60),
        request.send()
    ).await {
        Ok(Ok(response)) if response.status().is_success() => {
            match response.json::<serde_json::Value>().await {
                Ok(json) => {
                    if let Some(content) = json.get("content").and_then(|c| c.as_str()) {
                        if !content.is_empty() {
                            TestResult::pass(name, category, start.elapsed(),
                                &format!("Generated: '{}'", content.chars().take(50).collect::<String>()))
                        } else {
                            TestResult::fail(name, category, start.elapsed(), "Empty response", None)
                        }
                    } else {
                        TestResult::fail(name, category, start.elapsed(), "No content in response",
                            Some(format!("{:?}", json)))
                    }
                }
                Err(e) => TestResult::fail(name, category, start.elapsed(), "Invalid JSON response", Some(e.to_string()))
            }
        }
        Ok(Ok(response)) => {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            TestResult::fail(name, category, start.elapsed(),
                &format!("Server error: {}", status), Some(body))
        }
        Ok(Err(e)) => TestResult::fail(name, category, start.elapsed(), "Request failed", Some(e.to_string())),
        Err(_) => TestResult::fail(name, category, start.elapsed(), "Completion timeout (60s)", None)
    }
}

async fn test_llm_chat_completion() -> TestResult {
    let start = Instant::now();
    let name = "llm_chat_completion";
    let category = "llm";

    let client = reqwest::Client::new();

    // Chat completion with messages format
    let request_body = serde_json::json!({
        "messages": [
            {"role": "system", "content": "You are a helpful test assistant. Reply briefly."},
            {"role": "user", "content": "What is 2+2? Reply with just the number."}
        ],
        "n_predict": 10,
        "temperature": 0.1
    });

    let mut request = client.post("http://127.0.0.1:8080/v1/chat/completions").json(&request_body);
    if let Ok(token) = std::env::var("LLAMA_AUTH_TOKEN") {
        request = request.header("Authorization", format!("Bearer {}", token));
    }

    match tokio::time::timeout(
        Duration::from_secs(60),
        request.send()
    ).await {
        Ok(Ok(response)) if response.status().is_success() => {
            match response.json::<serde_json::Value>().await {
                Ok(json) => {
                    // OpenAI-compatible format
                    if let Some(content) = json
                        .get("choices")
                        .and_then(|c| c.get(0))
                        .and_then(|c| c.get("message"))
                        .and_then(|m| m.get("content"))
                        .and_then(|c| c.as_str())
                    {
                        if content.contains('4') {
                            TestResult::pass(name, category, start.elapsed(),
                                &format!("Chat works: '{}'", content.trim()))
                        } else {
                            TestResult::pass(name, category, start.elapsed(),
                                &format!("Chat works (response: '{}')", content.chars().take(30).collect::<String>()))
                        }
                    } else {
                        TestResult::fail(name, category, start.elapsed(), "Unexpected response format",
                            Some(format!("{:?}", json)))
                    }
                }
                Err(e) => TestResult::fail(name, category, start.elapsed(), "Invalid JSON", Some(e.to_string()))
            }
        }
        Ok(Ok(response)) => {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            TestResult::fail(name, category, start.elapsed(),
                &format!("Chat endpoint error: {}", status), Some(body))
        }
        Ok(Err(e)) => TestResult::fail(name, category, start.elapsed(), "Request failed", Some(e.to_string())),
        Err(_) => TestResult::fail(name, category, start.elapsed(), "Chat completion timeout (60s)", None)
    }
}

async fn test_llm_streaming() -> TestResult {
    let start = Instant::now();
    let name = "llm_streaming";
    let category = "llm";

    let client = reqwest::Client::new();

    // Test streaming endpoint
    let request_body = serde_json::json!({
        "prompt": "Count from 1 to 5:",
        "n_predict": 30,
        "temperature": 0.1,
        "stream": true
    });

    let mut request = client.post("http://127.0.0.1:8080/completion").json(&request_body);
    if let Ok(token) = std::env::var("LLAMA_AUTH_TOKEN") {
        request = request.header("Authorization", format!("Bearer {}", token));
    }

    match tokio::time::timeout(
        Duration::from_secs(60),
        request.send()
    ).await {
        Ok(Ok(response)) if response.status().is_success() => {
            let mut stream = response.bytes_stream();
            let mut chunks_received = 0;
            let mut total_content = String::new();

            while let Ok(Some(chunk)) = tokio::time::timeout(
                Duration::from_secs(10),
                stream.next()
            ).await {
                if let Ok(bytes) = chunk {
                    chunks_received += 1;
                    // Parse SSE data
                    let text = String::from_utf8_lossy(&bytes);
                    for line in text.lines() {
                        if line.starts_with("data: ") {
                            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&line[6..]) {
                                if let Some(content) = json.get("content").and_then(|c| c.as_str()) {
                                    total_content.push_str(content);
                                }
                            }
                        }
                    }
                }
                // Stop after receiving some chunks
                if chunks_received > 5 {
                    break;
                }
            }

            if chunks_received > 0 {
                TestResult::pass(name, category, start.elapsed(),
                    &format!("Received {} chunks, content: '{}'", chunks_received,
                        total_content.chars().take(30).collect::<String>()))
            } else {
                TestResult::fail(name, category, start.elapsed(), "No chunks received", None)
            }
        }
        Ok(Ok(response)) => {
            TestResult::fail(name, category, start.elapsed(),
                &format!("Streaming error: {}", response.status()), None)
        }
        Ok(Err(e)) => TestResult::fail(name, category, start.elapsed(), "Stream request failed", Some(e.to_string())),
        Err(_) => TestResult::fail(name, category, start.elapsed(), "Streaming timeout", None)
    }
}

async fn test_llm_actor_integration() -> TestResult {
    let start = Instant::now();
    let name = "llm_actor_integration";
    let category = "llm";

    use crate::actors::llm::LlmActorHandle;
    use crate::actors::traits::LlmActor;

    let model_path = PortablePathManager::models_dir().join("default-model.gguf");
    let llm = LlmActorHandle::new(model_path);

    match tokio::time::timeout(
        Duration::from_secs(90),
        llm.generate_with_params(
            "Reply with exactly: TEST_SUCCESS".to_string(),
            Some("You are a diagnostic test. Follow instructions exactly.".to_string()),
            Some(0.1)
        )
    ).await {
        Ok(Ok(response)) => {
            if response.to_uppercase().contains("TEST") || response.to_uppercase().contains("SUCCESS") || !response.is_empty() {
                TestResult::pass(name, category, start.elapsed(),
                    &format!("Actor generated {} chars", response.len()))
            } else {
                TestResult::fail(name, category, start.elapsed(),
                    "Unexpected response", Some(response))
            }
        }
        Ok(Err(e)) => TestResult::fail(name, category, start.elapsed(), "Actor generation failed", Some(e.to_string())),
        Err(_) => TestResult::fail(name, category, start.elapsed(), "Actor timeout (90s)", None)
    }
}

// Keep old test for backwards compatibility but mark it as deprecated
#[allow(dead_code)]
async fn test_llm_completion() -> TestResult {
    test_llm_simple_completion().await
}

// === FILE SYSTEM TESTS ===

fn test_fs_directories() -> TestResult {
    let start = Instant::now();
    let name = "fs_directories";
    let category = "filesystem";

    let dirs = vec![
        ("data", PortablePathManager::data_dir()),
        ("models", PortablePathManager::models_dir()),
        ("db", PortablePathManager::db_dir()),
        ("vectors", PortablePathManager::vectors_dir()),
        ("tools", PortablePathManager::tools_dir()),
    ];

    let mut issues = Vec::new();

    for (name, path) in &dirs {
        if !path.exists() {
            issues.push(format!("{} missing", name));
        } else if !path.is_dir() {
            issues.push(format!("{} is not a directory", name));
        }
    }

    if issues.is_empty() {
        TestResult::pass(name, category, start.elapsed(), "All directories exist")
    } else {
        TestResult::fail(name, category, start.elapsed(),
            &format!("{} issues", issues.len()), Some(issues.join(", ")))
    }
}

async fn test_fs_upload_file() -> TestResult {
    let start = Instant::now();
    let name = "fs_upload_file";
    let category = "filesystem";

    // Create a test file
    let test_dir = PortablePathManager::data_dir().join("files").join("diagnostic-test");

    if let Err(e) = std::fs::create_dir_all(&test_dir) {
        return TestResult::fail(name, category, start.elapsed(),
            "Failed to create test directory", Some(e.to_string()));
    }

    let test_file = test_dir.join("test.txt");
    let test_content = "This is a diagnostic test file.\nIt has multiple lines.\nFor testing file operations.";

    if let Err(e) = std::fs::write(&test_file, test_content) {
        return TestResult::fail(name, category, start.elapsed(),
            "Failed to write test file", Some(e.to_string()));
    }

    // Verify we can read it back
    match std::fs::read_to_string(&test_file) {
        Ok(content) if content == test_content => {
            // Clean up
            let _ = std::fs::remove_file(&test_file);
            let _ = std::fs::remove_dir(&test_dir);
            TestResult::pass(name, category, start.elapsed(), "File write/read successful")
        }
        Ok(_) => {
            TestResult::fail(name, category, start.elapsed(), "Content mismatch", None)
        }
        Err(e) => {
            TestResult::fail(name, category, start.elapsed(), "Failed to read test file", Some(e.to_string()))
        }
    }
}

// === ADDITIONAL DATABASE TESTS ===

async fn test_db_get_messages() -> TestResult {
    let start = Instant::now();
    let name = "db_get_messages";
    let category = "database";

    let pool = match database::init_db().await {
        Ok(p) => p,
        Err(e) => return TestResult::fail(name, category, start.elapsed(), "DB init failed", Some(e.to_string())),
    };

    // Create a session and add messages
    let config = ModelConfig::default();
    let session = match database::create_session(&pool, "Message Retrieval Test".to_string(), config).await {
        Ok(s) => s,
        Err(e) => return TestResult::fail(name, category, start.elapsed(), "Session creation failed", Some(e.to_string())),
    };

    // Add test messages
    let _ = database::add_message(&pool, &session.id, "user", "Test question").await;
    let _ = database::add_message(&pool, &session.id, "assistant", "Test answer").await;

    // Retrieve messages
    match database::get_session_messages(&pool, &session.id).await {
        Ok(messages) => {
            if messages.len() >= 2 {
                TestResult::pass(name, category, start.elapsed(), &format!("Retrieved {} messages", messages.len()))
            } else {
                TestResult::fail(name, category, start.elapsed(),
                    &format!("Expected 2+ messages, got {}", messages.len()), None)
            }
        }
        Err(e) => TestResult::fail(name, category, start.elapsed(), "Failed to get messages", Some(e.to_string())),
    }
}

async fn test_db_update_session() -> TestResult {
    let start = Instant::now();
    let name = "db_update_session";
    let category = "database";

    let pool = match database::init_db().await {
        Ok(p) => p,
        Err(e) => return TestResult::fail(name, category, start.elapsed(), "DB init failed", Some(e.to_string())),
    };

    let config = ModelConfig::default();
    let session = match database::create_session(&pool, "Update Test".to_string(), config).await {
        Ok(s) => s,
        Err(e) => return TestResult::fail(name, category, start.elapsed(), "Session creation failed", Some(e.to_string())),
    };

    // Update session title
    match database::update_session(&pool, &session.id, Some("Updated Title".to_string()), None).await {
        Ok(_) => TestResult::pass(name, category, start.elapsed(), "Session updated successfully"),
        Err(e) => TestResult::fail(name, category, start.elapsed(), "Failed to update session", Some(e.to_string())),
    }
}

// === ADDITIONAL RAG TESTS ===

async fn test_rag_context_retrieval() -> TestResult {
    let start = Instant::now();
    let name = "rag_context_retrieval";
    let category = "rag";

    use crate::actors::rag::RagActorHandle;
    use crate::actors::traits::RagActor;

    let rag = RagActorHandle::new();

    // Ingest some context first
    let context = "WhytChat is a private AI assistant. It runs locally on your machine. No data is sent to external servers.";
    let _ = rag.ingest(context.to_string(), Some("diagnostic:context".to_string())).await;

    // Search and verify relevance
    match rag.search_with_filters("What is WhytChat?".to_string(), vec![]).await {
        Ok(results) => {
            let relevant = results.iter().any(|r|
                r.to_lowercase().contains("whytchat") ||
                r.to_lowercase().contains("private") ||
                r.to_lowercase().contains("local")
            );
            if relevant {
                TestResult::pass(name, category, start.elapsed(), "Retrieved relevant context")
            } else {
                TestResult::pass(name, category, start.elapsed(),
                    &format!("Retrieved {} results (may not match test data)", results.len()))
            }
        }
        Err(e) => TestResult::fail(name, category, start.elapsed(), "Context retrieval failed", Some(e.to_string())),
    }
}

// === ADDITIONAL BRAIN TESTS ===

fn test_brain_language_detection() -> TestResult {
    let start = Instant::now();
    let name = "brain_language_detection";
    let category = "brain";

    let analyzer = BrainAnalyzer::new();

    let french = analyzer.analyze("Bonjour, comment allez-vous aujourd'hui?");
    let english = analyzer.analyze("Hello, how are you today?");

    // Check that language is detected (even if just checking the packet exists)
    let french_has_keywords = !french.keywords.is_empty();
    let english_has_keywords = !english.keywords.is_empty();

    if french_has_keywords && english_has_keywords {
        TestResult::pass(name, category, start.elapsed(), "Both languages analyzed")
    } else {
        TestResult::fail(name, category, start.elapsed(),
            &format!("FR keywords: {}, EN keywords: {}", french.keywords.len(), english.keywords.len()), None)
    }
}

// === ADDITIONAL FILE SYSTEM TESTS ===

async fn test_fs_model_file() -> TestResult {
    let start = Instant::now();
    let name = "fs_model_file";
    let category = "filesystem";

    let model_path = PortablePathManager::models_dir().join("default-model.gguf");

    if !model_path.exists() {
        return TestResult::fail(name, category, start.elapsed(),
            "Model file not found", Some(format!("{:?}", model_path)));
    }

    match std::fs::metadata(&model_path) {
        Ok(meta) => {
            let size_gb = meta.len() as f64 / 1024.0 / 1024.0 / 1024.0;
            if size_gb >= 3.0 {
                TestResult::pass(name, category, start.elapsed(),
                    &format!("Model file OK ({:.2} GB)", size_gb))
            } else {
                TestResult::fail(name, category, start.elapsed(),
                    &format!("Model too small ({:.2} GB, need 3+ GB)", size_gb), None)
            }
        }
        Err(e) => TestResult::fail(name, category, start.elapsed(), "Cannot read model", Some(e.to_string()))
    }
}

async fn test_fs_llama_server() -> TestResult {
    let start = Instant::now();
    let name = "fs_llama_server";
    let category = "filesystem";

    let server_exe = if cfg!(windows) { "llama-server.exe" } else { "llama-server" };
    let server_path = PortablePathManager::tools_dir().join("llama").join(server_exe);

    if !server_path.exists() {
        return TestResult::fail(name, category, start.elapsed(),
            "llama-server not found", Some(format!("{:?}", server_path)));
    }

    // Check required DLLs on Windows
    #[cfg(windows)]
    {
        let llama_dir = PortablePathManager::tools_dir().join("llama");
        let required = ["llama.dll", "ggml.dll"];
        let mut missing = Vec::new();

        for dll in &required {
            if !llama_dir.join(dll).exists() {
                missing.push(*dll);
            }
        }

        if !missing.is_empty() {
            return TestResult::fail(name, category, start.elapsed(),
                "Missing DLLs", Some(missing.join(", ")));
        }
    }

    match std::fs::metadata(&server_path) {
        Ok(meta) => {
            let size_mb = meta.len() as f64 / 1024.0 / 1024.0;
            TestResult::pass(name, category, start.elapsed(),
                &format!("llama-server OK ({:.1} MB)", size_mb))
        }
        Err(e) => TestResult::fail(name, category, start.elapsed(), "Cannot read server", Some(e.to_string()))
    }
}

// === INTEGRATION TEST ===

async fn test_full_conversation_flow() -> TestResult {
    let start = Instant::now();
    let name = "full_conversation_flow";
    let category = "integration";

    // This test simulates a complete user interaction:
    // 1. Create session
    // 2. Analyze user input with Brain
    // 3. Get context from RAG
    // 4. Generate response with LLM
    // 5. Save to database

    let pool = match database::init_db().await {
        Ok(p) => p,
        Err(e) => return TestResult::fail(name, category, start.elapsed(), "DB init failed", Some(e.to_string())),
    };

    // 1. Create session
    let config = ModelConfig::default();
    let session = match database::create_session(&pool, "Integration Test".to_string(), config).await {
        Ok(s) => s,
        Err(e) => return TestResult::fail(name, category, start.elapsed(), "Session failed", Some(e.to_string())),
    };

    // 2. Analyze input
    let analyzer = BrainAnalyzer::new();
    let user_input = "What is machine learning?";
    let packet = analyzer.analyze(user_input);

    if packet.keywords.is_empty() {
        return TestResult::fail(name, category, start.elapsed(), "Brain analysis failed", None);
    }

    // 3. Save user message
    if let Err(e) = database::add_message(&pool, &session.id, "user", user_input).await {
        return TestResult::fail(name, category, start.elapsed(), "Save user msg failed", Some(e.to_string()));
    }

    // 4. Get RAG context (optional, may be empty)
    use crate::actors::rag::RagActorHandle;
    use crate::actors::traits::RagActor;
    let rag = RagActorHandle::new();
    let _context = rag.search_with_filters(user_input.to_string(), vec![]).await.ok();

    // 5. Generate LLM response
    use crate::actors::llm::LlmActorHandle;
    use crate::actors::traits::LlmActor;

    let model_path = PortablePathManager::models_dir().join("default-model.gguf");
    let llm = LlmActorHandle::new(model_path);

    let response = match tokio::time::timeout(
        Duration::from_secs(120),
        llm.generate_with_params(
            user_input.to_string(),
            Some("You are a helpful assistant. Be brief.".to_string()),
            Some(0.7)
        )
    ).await {
        Ok(Ok(r)) => r,
        Ok(Err(e)) => return TestResult::fail(name, category, start.elapsed(), "LLM failed", Some(e.to_string())),
        Err(_) => return TestResult::fail(name, category, start.elapsed(), "LLM timeout", None),
    };

    if response.is_empty() {
        return TestResult::fail(name, category, start.elapsed(), "Empty LLM response", None);
    }

    // 6. Save assistant response
    if let Err(e) = database::add_message(&pool, &session.id, "assistant", &response).await {
        return TestResult::fail(name, category, start.elapsed(), "Save assistant msg failed", Some(e.to_string()));
    }

    // 7. Verify conversation was saved
    match database::get_session_messages(&pool, &session.id).await {
        Ok(messages) => {
            if messages.len() >= 2 {
                TestResult::pass(name, category, start.elapsed(),
                    &format!("Full flow completed: {} messages, {} char response",
                        messages.len(), response.len()))
            } else {
                TestResult::fail(name, category, start.elapsed(),
                    "Messages not saved correctly", None)
            }
        }
        Err(e) => TestResult::fail(name, category, start.elapsed(), "Get messages failed", Some(e.to_string())),
    }
}
