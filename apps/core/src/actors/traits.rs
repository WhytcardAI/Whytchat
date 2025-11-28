use crate::actors::messages::{AppError, SearchResult};
use async_trait::async_trait;
use tokio::sync::mpsc;

/// Defines the public interface for an LLM (Large Language Model) actor.
///
/// This trait abstracts the specific implementation of the LLM, allowing for different
/// backends (e.g., local llama.cpp, remote API) to be used interchangeably.
#[async_trait]
pub trait LlmActor: Send + Sync + 'static {
    /// Generates a complete text response based on a prompt and optional parameters.
    async fn generate_with_params(
        &self,
        prompt: String,
        system_prompt: Option<String>,
        temperature: Option<f32>,
    ) -> Result<String, AppError>;

    /// Generates a streaming response, sending chunks of text as they are produced.
    async fn stream_generate_with_params(
        &self,
        prompt: String,
        system_prompt: Option<String>,
        temperature: Option<f32>,
        chunk_sender: mpsc::Sender<Result<String, AppError>>,
    ) -> Result<(), AppError>;
}

/// Defines the public interface for a RAG (Retrieval-Augmented Generation) actor.
///
/// This trait abstracts the logic for managing and querying a knowledge base.
#[async_trait]
pub trait RagActor: Send + Sync + 'static {
    /// Ingests new content into the knowledge base.
    async fn ingest(&self, content: String, metadata: Option<String>) -> Result<String, AppError>;

    /// Searches the knowledge base for content relevant to a query.
    async fn search_with_filters(
        &self,
        query: String,
        file_ids: Vec<String>,
    ) -> Result<Vec<SearchResult>, AppError>;

    /// Deletes all vectors associated with a specific file.
    async fn delete_for_file(&self, file_id: String) -> Result<(), AppError>;
}

#[cfg(test)]
pub mod mocks {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;
    use tokio::sync::Mutex;

    /// Mock LLM Actor for testing
    pub struct MockLlmActor {
        pub response: Arc<Mutex<String>>,
        pub call_count: AtomicUsize,
        pub last_prompt: Arc<Mutex<Option<String>>>,
        pub last_system_prompt: Arc<Mutex<Option<String>>>,
        pub should_fail: std::sync::atomic::AtomicBool,
    }

    impl MockLlmActor {
        pub fn new(response: &str) -> Self {
            Self {
                response: Arc::new(Mutex::new(response.to_string())),
                call_count: AtomicUsize::new(0),
                last_prompt: Arc::new(Mutex::new(None)),
                last_system_prompt: Arc::new(Mutex::new(None)),
                should_fail: std::sync::atomic::AtomicBool::new(false),
            }
        }

        pub fn with_failure() -> Self {
            let mock = Self::new("");
            mock.should_fail.store(true, Ordering::SeqCst);
            mock
        }

        #[allow(dead_code)]
        pub async fn set_response(&self, response: &str) {
            *self.response.lock().await = response.to_string();
        }

        pub fn get_call_count(&self) -> usize {
            self.call_count.load(Ordering::SeqCst)
        }
    }

    #[async_trait]
    impl LlmActor for MockLlmActor {
        async fn generate_with_params(
            &self,
            prompt: String,
            system_prompt: Option<String>,
            _temperature: Option<f32>,
        ) -> Result<String, AppError> {
            self.call_count.fetch_add(1, Ordering::SeqCst);
            *self.last_prompt.lock().await = Some(prompt);
            *self.last_system_prompt.lock().await = system_prompt;

            if self.should_fail.load(Ordering::SeqCst) {
                return Err(AppError::Internal("Mock LLM failure".to_string()));
            }

            Ok(self.response.lock().await.clone())
        }

        async fn stream_generate_with_params(
            &self,
            prompt: String,
            system_prompt: Option<String>,
            _temperature: Option<f32>,
            chunk_sender: mpsc::Sender<Result<String, AppError>>,
        ) -> Result<(), AppError> {
            self.call_count.fetch_add(1, Ordering::SeqCst);
            *self.last_prompt.lock().await = Some(prompt);
            *self.last_system_prompt.lock().await = system_prompt;

            if self.should_fail.load(Ordering::SeqCst) {
                let _ = chunk_sender
                    .send(Err(AppError::Internal("Mock LLM failure".to_string())))
                    .await;
                return Err(AppError::Internal("Mock LLM failure".to_string()));
            }

            // Send response as chunks
            let response = self.response.lock().await.clone();
            for word in response.split_whitespace() {
                let _ = chunk_sender.send(Ok(format!("{} ", word))).await;
            }

            Ok(())
        }
    }

    /// Mock RAG Actor for testing
    pub struct MockRagActor {
        pub search_results: Arc<Mutex<Vec<SearchResult>>>,
        pub ingest_count: AtomicUsize,
        pub search_count: AtomicUsize,
        pub delete_count: AtomicUsize,
        pub last_query: Arc<Mutex<Option<String>>>,
        pub last_ingested: Arc<Mutex<Option<String>>>,
        pub should_fail: std::sync::atomic::AtomicBool,
    }

    impl MockRagActor {
        pub fn new() -> Self {
            Self {
                search_results: Arc::new(Mutex::new(vec![])),
                ingest_count: AtomicUsize::new(0),
                search_count: AtomicUsize::new(0),
                delete_count: AtomicUsize::new(0),
                last_query: Arc::new(Mutex::new(None)),
                last_ingested: Arc::new(Mutex::new(None)),
                should_fail: std::sync::atomic::AtomicBool::new(false),
            }
        }

        pub async fn with_results(results: Vec<SearchResult>) -> Self {
            let mock = Self::new();
            *mock.search_results.lock().await = results;
            mock
        }

        #[allow(dead_code)]
        pub fn with_failure() -> Self {
            let mock = Self::new();
            mock.should_fail.store(true, Ordering::SeqCst);
            mock
        }
    }

    impl Default for MockRagActor {
        fn default() -> Self {
            Self::new()
        }
    }

    #[async_trait]
    impl RagActor for MockRagActor {
        async fn ingest(
            &self,
            content: String,
            _metadata: Option<String>,
        ) -> Result<String, AppError> {
            self.ingest_count.fetch_add(1, Ordering::SeqCst);
            *self.last_ingested.lock().await = Some(content);

            if self.should_fail.load(Ordering::SeqCst) {
                return Err(AppError::Internal("Mock RAG ingest failure".to_string()));
            }

            Ok("Ingested successfully".to_string())
        }

        async fn search_with_filters(
            &self,
            query: String,
            _file_ids: Vec<String>,
        ) -> Result<Vec<SearchResult>, AppError> {
            self.search_count.fetch_add(1, Ordering::SeqCst);
            *self.last_query.lock().await = Some(query);

            if self.should_fail.load(Ordering::SeqCst) {
                return Err(AppError::Internal("Mock RAG search failure".to_string()));
            }

            Ok(self.search_results.lock().await.clone())
        }

        async fn delete_for_file(&self, _file_id: String) -> Result<(), AppError> {
            self.delete_count.fetch_add(1, Ordering::SeqCst);

            if self.should_fail.load(Ordering::SeqCst) {
                return Err(AppError::Internal("Mock RAG delete failure".to_string()));
            }

            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::mocks::*;
    use super::*;

    #[tokio::test]
    async fn test_mock_llm_actor_generate() {
        let mock = MockLlmActor::new("Hello, I am an AI assistant!");

        let result = mock
            .generate_with_params("Hello".to_string(), None, None)
            .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Hello, I am an AI assistant!");
        assert_eq!(mock.get_call_count(), 1);
    }

    #[tokio::test]
    async fn test_mock_llm_actor_stream() {
        let mock = MockLlmActor::new("Hello world");
        let (tx, mut rx) = mpsc::channel(32);

        let result = mock
            .stream_generate_with_params("Hello".to_string(), None, None, tx)
            .await;

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
    async fn test_mock_llm_actor_failure() {
        let mock = MockLlmActor::with_failure();

        let result = mock
            .generate_with_params("Hello".to_string(), None, None)
            .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_mock_rag_actor_search() {
        let results = vec![SearchResult {
            content: "Test content".to_string(),
            metadata: Some("file:test.txt".to_string()),
            score: 0.9,
        }];

        let mock = MockRagActor::with_results(results).await;

        let result = mock
            .search_with_filters("test query".to_string(), vec![])
            .await;

        assert!(result.is_ok());
        let results = result.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].content, "Test content");
    }

    #[tokio::test]
    async fn test_mock_rag_actor_ingest() {
        let mock = MockRagActor::new();

        let result = mock.ingest("New content".to_string(), None).await;

        assert!(result.is_ok());
        assert_eq!(mock.ingest_count.load(std::sync::atomic::Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn test_mock_rag_actor_delete() {
        let mock = MockRagActor::new();

        let result = mock.delete_for_file("file-123".to_string()).await;

        assert!(result.is_ok());
        assert_eq!(mock.delete_count.load(std::sync::atomic::Ordering::SeqCst), 1);
    }
}
