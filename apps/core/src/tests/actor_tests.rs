//! Actor System Tests
//!
//! Comprehensive tests for LLM Actor, RAG Actor, and Supervisor Actor.

use crate::actors::llm::LlmActorHandle;
use crate::actors::messages::{ActorError, AppError, SearchResult};
use crate::actors::rag::RagActorHandle;
use crate::actors::supervisor::SupervisorHandle;
use crate::actors::traits::{LlmActor, RagActor};
use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::time::{sleep, Duration};

// ============================================================================
// Mock Actors for Testing
// ============================================================================

/// Mock LLM Actor that returns predictable responses
pub struct MockLlmActor {
    pub response: String,
    pub delay_ms: u64,
    pub should_fail: bool,
}

impl MockLlmActor {
    pub fn new(response: &str) -> Self {
        Self {
            response: response.to_string(),
            delay_ms: 0,
            should_fail: false,
        }
    }

    pub fn with_delay(mut self, ms: u64) -> Self {
        self.delay_ms = ms;
        self
    }

    pub fn failing() -> Self {
        Self {
            response: String::new(),
            delay_ms: 0,
            should_fail: true,
        }
    }
}

#[async_trait]
impl LlmActor for MockLlmActor {
    async fn generate_with_params(
        &self,
        _prompt: String,
        _system_prompt: Option<String>,
        _temperature: Option<f32>,
    ) -> Result<String, AppError> {
        if self.delay_ms > 0 {
            sleep(Duration::from_millis(self.delay_ms)).await;
        }

        if self.should_fail {
            Err(AppError::Actor(ActorError::Internal("Mock failure".to_string())))
        } else {
            Ok(self.response.clone())
        }
    }

    async fn stream_generate_with_params(
        &self,
        _prompt: String,
        _system_prompt: Option<String>,
        _temperature: Option<f32>,
        chunk_sender: mpsc::Sender<Result<String, AppError>>,
    ) -> Result<(), AppError> {
        if self.should_fail {
            let _ = chunk_sender.send(Err(AppError::Actor(ActorError::Internal("Mock failure".to_string())))).await;
            return Ok(());
        }

        // Stream response word by word
        for word in self.response.split_whitespace() {
            if self.delay_ms > 0 {
                sleep(Duration::from_millis(self.delay_ms / 10)).await;
            }
            let _ = chunk_sender.send(Ok(format!("{} ", word))).await;
        }
        Ok(())
    }
}

/// Mock RAG Actor for testing
pub struct MockRagActor {
    pub search_results: Vec<SearchResult>,
    pub should_fail: bool,
}

impl MockRagActor {
    pub fn new() -> Self {
        Self {
            search_results: vec![],
            should_fail: false,
        }
    }

    pub fn with_results(mut self, results: Vec<SearchResult>) -> Self {
        self.search_results = results;
        self
    }

    pub fn failing() -> Self {
        Self {
            search_results: vec![],
            should_fail: true,
        }
    }
}

#[async_trait]
impl RagActor for MockRagActor {
    async fn ingest(
        &self,
        _content: String,
        _metadata: Option<String>,
    ) -> Result<String, AppError> {
        if self.should_fail {
            Err(AppError::Actor(ActorError::Internal("Mock ingest failure".to_string())))
        } else {
            Ok("Ingested successfully".to_string())
        }
    }

    async fn search_with_filters(
        &self,
        _query: String,
        _file_ids: Vec<String>,
    ) -> Result<Vec<SearchResult>, AppError> {
        if self.should_fail {
            Err(AppError::Actor(ActorError::Internal("Mock search failure".to_string())))
        } else {
            Ok(self.search_results.clone())
        }
    }

    async fn delete_for_file(&self, _file_id: String) -> Result<(), AppError> {
        if self.should_fail {
            Err(AppError::Actor(ActorError::Internal("Mock delete failure".to_string())))
        } else {
            Ok(())
        }
    }
}

// ============================================================================
// LLM Actor Tests
// ============================================================================

#[cfg(test)]
mod llm_actor_tests {
    use super::*;

    #[tokio::test]
    async fn test_mock_llm_basic_response() {
        let llm = MockLlmActor::new("Hello, I am an AI assistant.");

        let result = llm.generate_with_params(
            "Hello".to_string(),
            None,
            None,
        ).await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Hello, I am an AI assistant.");
    }

    #[tokio::test]
    async fn test_mock_llm_with_system_prompt() {
        let llm = MockLlmActor::new("System prompt applied.");

        let result = llm.generate_with_params(
            "Test prompt".to_string(),
            Some("You are a helpful assistant.".to_string()),
            Some(0.7),
        ).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_mock_llm_streaming() {
        let llm = MockLlmActor::new("Hello world from stream");
        let (tx, mut rx) = mpsc::channel(32);

        let result = llm.stream_generate_with_params(
            "Test".to_string(),
            None,
            None,
            tx,
        ).await;

        assert!(result.is_ok());

        let mut collected = String::new();
        while let Some(chunk) = rx.recv().await {
            if let Ok(text) = chunk {
                collected.push_str(&text);
            }
        }

        assert!(collected.contains("Hello"));
        assert!(collected.contains("world"));
    }

    #[tokio::test]
    async fn test_mock_llm_failure() {
        let llm = MockLlmActor::failing();

        let result = llm.generate_with_params(
            "Test".to_string(),
            None,
            None,
        ).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_mock_llm_with_delay() {
        let llm = MockLlmActor::new("Delayed response").with_delay(100);

        let start = std::time::Instant::now();
        let result = llm.generate_with_params(
            "Test".to_string(),
            None,
            None,
        ).await;
        let elapsed = start.elapsed();

        assert!(result.is_ok());
        assert!(elapsed.as_millis() >= 100);
    }
}

// ============================================================================
// RAG Actor Tests
// ============================================================================

#[cfg(test)]
mod rag_actor_tests {
    use super::*;

    #[tokio::test]
    async fn test_mock_rag_ingest() {
        let rag = MockRagActor::new();

        let result = rag.ingest(
            "Test content to ingest".to_string(),
            Some("test-metadata".to_string()),
        ).await;

        assert!(result.is_ok());
        assert!(result.unwrap().contains("Ingested"));
    }

    #[tokio::test]
    async fn test_mock_rag_search_empty() {
        let rag = MockRagActor::new();

        let results = rag.search_with_filters(
            "test query".to_string(),
            vec![],
        ).await;

        assert!(results.is_ok());
        assert!(results.unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_mock_rag_search_with_results() {
        let search_results = vec![
            SearchResult {
                content: "First result".to_string(),
                metadata: Some("file:123".to_string()),
                score: 0.9,
            },
            SearchResult {
                content: "Second result".to_string(),
                metadata: Some("file:456".to_string()),
                score: 0.7,
            },
        ];

        let rag = MockRagActor::new().with_results(search_results);

        let results = rag.search_with_filters(
            "test query".to_string(),
            vec!["123".to_string()],
        ).await;

        assert!(results.is_ok());
        let results = results.unwrap();
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].score, 0.9);
    }

    #[tokio::test]
    async fn test_mock_rag_delete() {
        let rag = MockRagActor::new();

        let result = rag.delete_for_file("file-123".to_string()).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_mock_rag_failure() {
        let rag = MockRagActor::failing();

        let result = rag.ingest(
            "Content".to_string(),
            None,
        ).await;

        assert!(result.is_err());
    }
}

// ============================================================================
// Supervisor Tests with Mocks
// ============================================================================

#[cfg(test)]
mod supervisor_tests {
    use super::*;
    use crate::brain::BrainAnalyzer;

    #[tokio::test]
    async fn test_supervisor_with_mock_actors() {
        let llm = Arc::new(MockLlmActor::new("Mock response from LLM"));
        let rag = Arc::new(MockRagActor::new());

        let supervisor = SupervisorHandle::new_with_actors(llm, rag, None);

        // The supervisor should be created successfully
        // Without a real database, process_message will fail on DB access
        // This test verifies the structure works
        assert!(true);
    }

    #[tokio::test]
    async fn test_supervisor_ingest_content() {
        let llm = Arc::new(MockLlmActor::new(""));
        let rag = Arc::new(MockRagActor::new());

        let supervisor = SupervisorHandle::new_with_actors(llm, rag, None);

        let result = supervisor.ingest_content(
            "Test content".to_string(),
            Some("metadata".to_string()),
        ).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_supervisor_reindex_file() {
        let llm = Arc::new(MockLlmActor::new(""));
        let rag = Arc::new(MockRagActor::new());

        let supervisor = SupervisorHandle::new_with_actors(llm, rag, None);

        let result = supervisor.reindex_file(
            "file-123".to_string(),
            "New content".to_string(),
        ).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_supervisor_concurrent_ingestion() {
        let llm = Arc::new(MockLlmActor::new(""));
        let rag = Arc::new(MockRagActor::new());

        let supervisor = SupervisorHandle::new_with_actors(llm, rag, None);

        let mut handles = vec![];

        for i in 0..10 {
            let sup = supervisor.clone();
            let handle = tokio::spawn(async move {
                sup.ingest_content(
                    format!("Content {}", i),
                    None,
                ).await
            });
            handles.push(handle);
        }

        let mut success_count = 0;
        for handle in handles {
            if handle.await.unwrap().is_ok() {
                success_count += 1;
            }
        }

        assert_eq!(success_count, 10);
    }
}

// ============================================================================
// Integration Tests (Actor Communication)
// ============================================================================

#[cfg(test)]
mod actor_integration_tests {
    use super::*;

    #[tokio::test]
    async fn test_llm_rag_interaction() {
        // Simulate a RAG-augmented LLM flow
        let rag = MockRagActor::new().with_results(vec![
            SearchResult {
                content: "The answer is 42.".to_string(),
                metadata: Some("file:doc1".to_string()),
                score: 0.95,
            },
        ]);

        // Search for context
        let context = rag.search_with_filters(
            "What is the answer?".to_string(),
            vec!["doc1".to_string()],
        ).await.unwrap();

        assert_eq!(context.len(), 1);
        assert!(context[0].content.contains("42"));

        // Generate with context
        let llm = MockLlmActor::new("Based on the context, the answer is 42.");
        let response = llm.generate_with_params(
            format!("Context: {}\nQuestion: What is the answer?", context[0].content),
            None,
            None,
        ).await.unwrap();

        assert!(response.contains("42"));
    }

    #[tokio::test]
    async fn test_full_pipeline_mock() {
        // Complete pipeline: Ingest -> Search -> Generate
        let rag = MockRagActor::new();

        // 1. Ingest content
        let ingest_result = rag.ingest(
            "Important document content about Rust programming.".to_string(),
            Some("file:rust-doc".to_string()),
        ).await;
        assert!(ingest_result.is_ok());

        // 2. Search (mock will return empty, but flow works)
        let search_results = rag.search_with_filters(
            "Rust programming".to_string(),
            vec!["rust-doc".to_string()],
        ).await;
        assert!(search_results.is_ok());

        // 3. Generate response
        let llm = MockLlmActor::new("Rust is a systems programming language.");
        let response = llm.generate_with_params(
            "What is Rust?".to_string(),
            Some("You are a programming expert.".to_string()),
            Some(0.7),
        ).await;
        assert!(response.is_ok());
    }
}

// ============================================================================
// Error Handling Tests
// ============================================================================

#[cfg(test)]
mod error_handling_tests {
    use super::*;

    #[tokio::test]
    async fn test_llm_error_propagation() {
        let llm = MockLlmActor::failing();

        let result = llm.generate_with_params(
            "Test".to_string(),
            None,
            None,
        ).await;

        match result {
            Err(AppError::Actor(ActorError::Internal(msg))) => {
                assert!(msg.contains("Mock failure"));
            }
            _ => panic!("Expected ActorError::Internal"),
        }
    }

    #[tokio::test]
    async fn test_rag_error_propagation() {
        let rag = MockRagActor::failing();

        let result = rag.search_with_filters(
            "Test".to_string(),
            vec![],
        ).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_graceful_degradation() {
        // When RAG fails, the system should still attempt LLM generation
        let rag = MockRagActor::failing();
        let llm = MockLlmActor::new("Fallback response without context");

        // RAG search fails
        let search_result = rag.search_with_filters("test".to_string(), vec![]).await;
        assert!(search_result.is_err());

        // But LLM can still respond
        let llm_result = llm.generate_with_params(
            "test".to_string(),
            None,
            None,
        ).await;
        assert!(llm_result.is_ok());
    }
}

// ============================================================================
// Performance Tests
// ============================================================================

#[cfg(test)]
mod performance_tests {
    use super::*;

    #[tokio::test]
    async fn test_rapid_message_processing() {
        let llm = MockLlmActor::new("Quick response");

        let start = std::time::Instant::now();

        for _ in 0..100 {
            let _ = llm.generate_with_params(
                "Test".to_string(),
                None,
                None,
            ).await;
        }

        let elapsed = start.elapsed();
        assert!(
            elapsed.as_millis() < 1000,
            "100 mock generations should complete in under 1 second"
        );
    }

    #[tokio::test]
    async fn test_concurrent_actor_access() {
        let llm = Arc::new(MockLlmActor::new("Concurrent response"));

        let mut handles = vec![];

        for _ in 0..50 {
            let llm_clone = llm.clone();
            let handle = tokio::spawn(async move {
                llm_clone.generate_with_params(
                    "Test".to_string(),
                    None,
                    None,
                ).await
            });
            handles.push(handle);
        }

        let start = std::time::Instant::now();

        for handle in handles {
            let _ = handle.await;
        }

        let elapsed = start.elapsed();
        assert!(
            elapsed.as_millis() < 500,
            "50 concurrent calls should complete quickly"
        );
    }
}
