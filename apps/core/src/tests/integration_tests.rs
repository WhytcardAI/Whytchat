//! Integration Tests
//!
//! End-to-end tests that verify complete workflows across multiple components.

use crate::actors::messages::{AppError, SearchResult};
use crate::actors::supervisor::SupervisorHandle;
use crate::actors::traits::{LlmActor, RagActor};
use crate::brain::BrainAnalyzer;
use crate::database;
use crate::models::ModelConfig;
use async_trait::async_trait;
use sqlx::sqlite::SqlitePool;
use std::sync::Arc;
use tempfile::tempdir;
use tokio::sync::mpsc;

// ============================================================================
// Test Fixtures
// ============================================================================

/// Create a test database pool
async fn create_test_pool() -> SqlitePool {
    let dir = tempdir().expect("Failed to create temp dir");
    let db_path = dir.path().join("integration_test.sqlite");
    let db_url = format!("sqlite://{}?mode=rwc", db_path.display());

    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await
        .expect("Failed to create test pool");

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    pool
}

/// Mock LLM for integration tests
struct IntegrationMockLlm {
    response_fn: Box<dyn Fn(&str) -> String + Send + Sync>,
}

impl IntegrationMockLlm {
    fn new<F>(f: F) -> Self
    where
        F: Fn(&str) -> String + Send + Sync + 'static,
    {
        Self {
            response_fn: Box::new(f),
        }
    }
}

#[async_trait]
impl LlmActor for IntegrationMockLlm {
    async fn generate_with_params(
        &self,
        prompt: String,
        _system_prompt: Option<String>,
        _temperature: Option<f32>,
    ) -> Result<String, AppError> {
        Ok((self.response_fn)(&prompt))
    }

    async fn stream_generate_with_params(
        &self,
        prompt: String,
        _system_prompt: Option<String>,
        _temperature: Option<f32>,
        chunk_sender: mpsc::Sender<Result<String, AppError>>,
    ) -> Result<(), AppError> {
        let response = (self.response_fn)(&prompt);
        for word in response.split_whitespace() {
            let _ = chunk_sender.send(Ok(format!("{} ", word))).await;
        }
        Ok(())
    }
}

/// Mock RAG for integration tests
struct IntegrationMockRag {
    documents: Vec<(String, String)>, // (content, file_id)
}

impl IntegrationMockRag {
    fn new() -> Self {
        Self { documents: vec![] }
    }

    fn with_documents(docs: Vec<(&str, &str)>) -> Self {
        Self {
            documents: docs
                .into_iter()
                .map(|(c, f)| (c.to_string(), f.to_string()))
                .collect(),
        }
    }
}

#[async_trait]
impl RagActor for IntegrationMockRag {
    async fn ingest(
        &self,
        _content: String,
        _metadata: Option<String>,
    ) -> Result<String, AppError> {
        Ok("Ingested".to_string())
    }

    async fn search_with_filters(
        &self,
        query: String,
        file_ids: Vec<String>,
    ) -> Result<Vec<SearchResult>, AppError> {
        let query_lower = query.to_lowercase();
        let results: Vec<SearchResult> = self
            .documents
            .iter()
            .filter(|(content, file_id)| {
                content.to_lowercase().contains(&query_lower)
                    && (file_ids.is_empty() || file_ids.contains(file_id))
            })
            .map(|(content, file_id)| SearchResult {
                content: content.clone(),
                metadata: Some(format!("file:{}", file_id)),
                score: 0.9,
            })
            .collect();
        Ok(results)
    }

    async fn delete_for_file(&self, _file_id: String) -> Result<(), AppError> {
        Ok(())
    }
}

// ============================================================================
// Workflow Integration Tests
// ============================================================================

#[cfg(test)]
mod full_workflow_tests {
    use super::*;

    #[tokio::test]
    async fn test_session_creation_and_messaging_workflow() {
        let pool = create_test_pool().await;

        // 1. Create a new session
        let config = ModelConfig {
            temperature: 0.7,
            system_prompt: "You are a helpful assistant.".to_string(),
            ..Default::default()
        };

        let session = database::create_session(&pool, "Test Chat".to_string(), config)
            .await
            .expect("Session creation failed");

        assert!(!session.id.is_empty());

        // 2. Add user message
        let user_msg = database::add_message(&pool, &session.id, "user", "Hello, AI!")
            .await
            .expect("Failed to add user message");

        assert_eq!(user_msg.role, "user");

        // 3. Add assistant response
        let assistant_msg =
            database::add_message(&pool, &session.id, "assistant", "Hello! How can I help?")
                .await
                .expect("Failed to add assistant message");

        assert_eq!(assistant_msg.role, "assistant");

        // 4. Verify message history
        let messages = database::get_session_messages(&pool, &session.id)
            .await
            .expect("Failed to get messages");

        assert_eq!(messages.len(), 2);
        assert_eq!(messages[0].role, "user");
        assert_eq!(messages[1].role, "assistant");
    }

    #[tokio::test]
    async fn test_file_upload_and_session_linking_workflow() {
        let pool = create_test_pool().await;

        // 1. Create a session
        let session = database::create_session(
            &pool,
            "Document Chat".to_string(),
            ModelConfig::default(),
        )
        .await
        .expect("Session creation failed");

        // 2. Add a file to the library
        let file = database::add_library_file(
            &pool,
            "research.pdf",
            "application/pdf",
            "files/research.pdf",
            "Research content about AI",
        )
        .await
        .expect("Failed to add file");

        // 3. Link file to session
        database::link_file_to_session(&pool, &file.id, &session.id)
            .await
            .expect("Failed to link file");

        // 4. Verify file is linked
        let session_files = database::get_session_files(&pool, &session.id)
            .await
            .expect("Failed to get session files");

        assert_eq!(session_files.len(), 1);
        assert_eq!(session_files[0].id, file.id);

        // 5. Unlink file
        database::unlink_file_from_session(&pool, &file.id, &session.id)
            .await
            .expect("Failed to unlink file");

        let files_after = database::get_session_files(&pool, &session.id)
            .await
            .expect("Failed to get session files");

        assert_eq!(files_after.len(), 0);
    }

    #[tokio::test]
    async fn test_folder_organization_workflow() {
        let pool = create_test_pool().await;

        // 1. Create folders
        let work_folder = database::create_folder(&pool, "Work".to_string())
            .await
            .expect("Failed to create folder");

        let personal_folder = database::create_folder(&pool, "Personal".to_string())
            .await
            .expect("Failed to create folder");

        // 2. Create sessions
        let work_session = database::create_session(
            &pool,
            "Work Project".to_string(),
            ModelConfig::default(),
        )
        .await
        .expect("Failed to create session");

        let personal_session = database::create_session(
            &pool,
            "Personal Notes".to_string(),
            ModelConfig::default(),
        )
        .await
        .expect("Failed to create session");

        // 3. Organize sessions into folders
        database::move_session_to_folder(&pool, &work_session.id, Some(&work_folder.id))
            .await
            .expect("Failed to move session");

        database::move_session_to_folder(&pool, &personal_session.id, Some(&personal_folder.id))
            .await
            .expect("Failed to move session");

        // 4. Verify organization
        let work_updated = database::get_session(&pool, &work_session.id)
            .await
            .expect("Failed to get session");

        assert_eq!(work_updated.folder_id, Some(work_folder.id.clone()));

        // 5. List all folders
        let folders = database::list_folders(&pool)
            .await
            .expect("Failed to list folders");

        assert_eq!(folders.len(), 2);
    }
}

#[cfg(test)]
mod brain_integration_tests {
    use super::*;

    #[tokio::test]
    async fn test_brain_to_rag_decision_flow() {
        let analyzer = BrainAnalyzer::new();

        // Test cases with expected RAG decision
        let test_cases = vec![
            ("Hello!", false),                                         // Greeting - no RAG
            ("What does the document say about climate?", true),       // Search - RAG
            ("Summarize the uploaded PDF", true),                      // Summarization - RAG
            ("Write a hello world in Python", false),                  // Code gen - no RAG
            ("Find information about machine learning", true),         // Search - RAG
        ];

        for (input, expected_rag) in test_cases {
            let result = analyzer.analyze(input);
            assert_eq!(
                result.should_use_rag, expected_rag,
                "For '{}', expected should_use_rag={}",
                input, expected_rag
            );
        }
    }

    #[tokio::test]
    async fn test_brain_analysis_in_chat_context() {
        let pool = create_test_pool().await;
        let analyzer = BrainAnalyzer::new();

        // Create a session
        let session = database::create_session(
            &pool,
            "Brain Test".to_string(),
            ModelConfig::default(),
        )
        .await
        .expect("Failed to create session");

        // Simulate conversation
        let messages = vec![
            ("user", "Hello!"),
            ("assistant", "Hello! How can I help you today?"),
            ("user", "Can you explain quantum computing?"),
        ];

        for (role, content) in &messages {
            database::add_message(&pool, &session.id, role, content)
                .await
                .expect("Failed to add message");
        }

        // Analyze the last user message
        let last_analysis = analyzer.analyze("Can you explain quantum computing?");

        assert_eq!(last_analysis.intent.intent, crate::brain::Intent::Explanation);
    }
}

#[cfg(test)]
mod rag_integration_tests {
    use super::*;

    #[tokio::test]
    async fn test_rag_search_filters_by_session_files() {
        let rag = IntegrationMockRag::with_documents(vec![
            ("Rust programming is safe and fast", "file-1"),
            ("Python is great for data science", "file-2"),
            ("JavaScript runs in the browser", "file-3"),
        ]);

        // Search with file filter
        let results = rag
            .search_with_filters("programming".to_string(), vec!["file-1".to_string()])
            .await
            .expect("Search failed");

        assert_eq!(results.len(), 1);
        assert!(results[0].content.contains("Rust"));

        // Search without filter
        let all_results = rag
            .search_with_filters("programming".to_string(), vec![])
            .await
            .expect("Search failed");

        // Should find Rust (contains "programming")
        assert!(all_results.len() >= 1);
    }

    #[tokio::test]
    async fn test_rag_context_building() {
        let rag = IntegrationMockRag::with_documents(vec![
            ("The secret code is 42", "secret-doc"),
            ("Regular information here", "public-doc"),
        ]);

        // Search for secret
        let results = rag
            .search_with_filters("secret code".to_string(), vec!["secret-doc".to_string()])
            .await
            .expect("Search failed");

        assert_eq!(results.len(), 1);
        assert!(results[0].content.contains("42"));

        // Build context string
        let context: String = results
            .iter()
            .map(|r| {
                let source = r.metadata.as_deref().unwrap_or("unknown");
                format!("[Source: {}]\n{}", source, r.content)
            })
            .collect::<Vec<_>>()
            .join("\n\n");

        assert!(context.contains("file:secret-doc"));
        assert!(context.contains("42"));
    }
}

#[cfg(test)]
mod supervisor_integration_tests {
    use super::*;

    #[tokio::test]
    async fn test_supervisor_handles_concurrent_requests() {
        let llm = Arc::new(IntegrationMockLlm::new(|_| {
            "Concurrent response".to_string()
        }));
        let rag = Arc::new(IntegrationMockRag::new());

        let supervisor = SupervisorHandle::new_with_actors(llm, rag, None);

        let mut handles = vec![];

        for i in 0..10 {
            let sup = supervisor.clone();
            handles.push(tokio::spawn(async move {
                sup.ingest_content(format!("Content {}", i), None).await
            }));
        }

        let mut success = 0;
        for handle in handles {
            if handle.await.unwrap().is_ok() {
                success += 1;
            }
        }

        assert_eq!(success, 10);
    }
}

// ============================================================================
// Error Recovery Tests
// ============================================================================

#[cfg(test)]
mod error_recovery_tests {
    use super::*;

    #[tokio::test]
    async fn test_database_connection_recovery() {
        let pool = create_test_pool().await;

        // Create session
        let session = database::create_session(
            &pool,
            "Recovery Test".to_string(),
            ModelConfig::default(),
        )
        .await
        .expect("Failed to create session");

        // Simulate heavy load
        let mut handles = vec![];
        for i in 0..50 {
            let pool_clone = pool.clone();
            let session_id = session.id.clone();
            handles.push(tokio::spawn(async move {
                database::add_message(&pool_clone, &session_id, "user", &format!("Message {}", i))
                    .await
            }));
        }

        let mut success = 0;
        for handle in handles {
            if handle.await.unwrap().is_ok() {
                success += 1;
            }
        }

        // All operations should succeed
        assert_eq!(success, 50);
    }

    #[tokio::test]
    async fn test_graceful_handling_of_missing_session() {
        let pool = create_test_pool().await;

        // Try to get non-existent session
        let result = database::get_session(&pool, "non-existent-id").await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_graceful_handling_of_missing_file() {
        let pool = create_test_pool().await;

        // Try to get non-existent file
        let result = database::get_library_file(&pool, "non-existent-file").await;

        assert!(result.is_err());
    }
}

// ============================================================================
// Performance Tests
// ============================================================================

#[cfg(test)]
mod performance_tests {
    use super::*;
    use std::time::Instant;

    #[tokio::test]
    async fn test_session_listing_performance() {
        let pool = create_test_pool().await;

        // Create many sessions
        for i in 0..100 {
            let _ = database::create_session(
                &pool,
                format!("Session {}", i),
                ModelConfig::default(),
            )
            .await;
        }

        let start = Instant::now();
        let sessions = database::list_sessions(&pool)
            .await
            .expect("Failed to list sessions");
        let elapsed = start.elapsed();

        assert_eq!(sessions.len(), 100);
        assert!(
            elapsed.as_millis() < 500,
            "Listing 100 sessions should be fast: {:?}",
            elapsed
        );
    }

    #[tokio::test]
    async fn test_message_retrieval_performance() {
        let pool = create_test_pool().await;

        let session = database::create_session(
            &pool,
            "Performance Test".to_string(),
            ModelConfig::default(),
        )
        .await
        .expect("Failed to create session");

        // Add many messages
        for i in 0..200 {
            let _ = database::add_message(
                &pool,
                &session.id,
                if i % 2 == 0 { "user" } else { "assistant" },
                &format!("Message content number {}", i),
            )
            .await;
        }

        let start = Instant::now();
        let messages = database::get_session_messages(&pool, &session.id)
            .await
            .expect("Failed to get messages");
        let elapsed = start.elapsed();

        assert_eq!(messages.len(), 200);
        assert!(
            elapsed.as_millis() < 200,
            "Retrieving 200 messages should be fast: {:?}",
            elapsed
        );
    }

    #[tokio::test]
    async fn test_brain_analysis_performance() {
        let analyzer = BrainAnalyzer::new();

        let inputs = vec![
            "Hello there!",
            "What is machine learning and how does it work?",
            "Write a function to calculate fibonacci numbers in Rust",
            "Summarize the key points from the uploaded document",
            "Search for information about climate change impacts",
        ];

        let start = Instant::now();

        for _ in 0..100 {
            for input in &inputs {
                let _ = analyzer.analyze(input);
            }
        }

        let elapsed = start.elapsed();

        // 500 analyses should complete quickly
        assert!(
            elapsed.as_millis() < 2000,
            "500 brain analyses should complete in under 2 seconds: {:?}",
            elapsed
        );
    }
}
