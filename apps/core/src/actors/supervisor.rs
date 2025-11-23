use crate::actors::llm::LlmActorHandle;
use crate::actors::messages::{AppError, SupervisorMessage};
use crate::actors::rag::RagActorHandle;
use crate::actors::traits::{LlmActor, RagActor};
use crate::database;
use crate::fs_manager::PortablePathManager;
use sqlx::sqlite::SqlitePool;
use std::sync::Arc;
use tauri::{Emitter, Window};
use tokio::sync::{mpsc, oneshot};
use tokio::time::{timeout, Duration};
use tracing::{error, info, instrument};

/// A handle to the `SupervisorActor`.
///
/// This is the primary entry point for all business logic in the application. It orchestrates
/// the `LlmActor` and `RagActor` to process user requests.
#[derive(Clone)]
pub struct SupervisorHandle {
    sender: mpsc::Sender<SupervisorMessage>,
}

impl SupervisorHandle {
    /// Creates a new `SupervisorActor` with a default configuration and returns a handle to it.
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self::new_with_pool_and_model(None, PortablePathManager::models_dir().join("model.gguf"))
    }

    /// Creates a new `SupervisorActor` with a specified database pool and returns a handle.
    #[allow(dead_code)]
    pub fn new_with_pool(db_pool: Option<SqlitePool>) -> Self {
        Self::new_with_pool_and_model(db_pool, PortablePathManager::models_dir().join("model.gguf"))
    }

    /// Creates a new `SupervisorActor` with a specified database pool and model path.
    ///
    /// This is the main constructor that spawns the actor and its children (`LlmActor`, `RagActor`).
    ///
    /// # Arguments
    ///
    /// * `db_pool` - An optional `SqlitePool` for database access.
    /// * `model_path` - The path to the GGUF model file to be used by the `LlmActor`.
    pub fn new_with_pool_and_model(db_pool: Option<SqlitePool>, model_path: std::path::PathBuf) -> Self {
        let (sender, receiver) = mpsc::channel(32);
        let actor = new_production_runner(receiver, db_pool, model_path);
        tokio::spawn(async move { actor.run().await });
        Self { sender }
    }

    /// Processes a user message from a specific session.
    ///
    /// This is the core logic loop:
    /// 1. Stores the user message in the database.
    /// 2. Performs a RAG search for relevant context.
    /// 3. Builds a final prompt with history and context.
    /// 4. Streams the LLM response back to the frontend via events.
    /// 5. Stores the assistant's final response in the database.
    ///
    /// # Arguments
    ///
    /// * `session_id` - The ID of the session the message belongs to.
    /// * `content` - The user's message content.
    /// * `window` - The Tauri window to emit events to (e.g., `chat-token`, `thinking-step`).
    ///
    /// # Returns
    ///
    /// A `Result` containing the final, complete response from the assistant.
    #[instrument(skip(self, window))]
    pub async fn process_message(
        &self,
        session_id: String,
        content: String,
        window: &Window,
    ) -> Result<String, AppError> {
        let (send, recv) = oneshot::channel();
        let msg = SupervisorMessage::ProcessUserMessage {
            session_id,
            content,
            window: Some(window.clone()),
            responder: send,
        };
        self.sender
            .send(msg)
            .await
            .map_err(|e| AppError::Actor(e.to_string()))?;
        timeout(Duration::from_secs(30), recv)
            .await?
            .map_err(|e| AppError::Actor(e.to_string()))?
    }

    /// Ingests content into the knowledge base.
    ///
    /// This method delegates the request to the `RagActor`.
    ///
    /// # Arguments
    ///
    /// * `content` - The text content to ingest.
    /// * `metadata` - Optional JSON string metadata to associate with the content.
    ///
    /// # Returns
    ///
    /// A `Result` containing a confirmation message on success.
    #[instrument(skip(self))]
    pub async fn ingest_content(
        &self,
        content: String,
        metadata: Option<String>,
    ) -> Result<String, AppError> {
        let (send, recv) = oneshot::channel();
        let msg = SupervisorMessage::IngestContent {
            content,
            metadata,
            responder: send,
        };
        self.sender
            .send(msg)
            .await
            .map_err(|e| AppError::Actor(e.to_string()))?;
        timeout(Duration::from_secs(60), recv) // Ingest can be long
            .await?
            .map_err(|e| AppError::Actor(e.to_string()))?
    }
}

// --- Actor Runner ---
struct SupervisorRunner<L, R>
where
    L: LlmActor + Send + Sync + 'static,
    R: RagActor + Send + Sync + 'static,
{
    receiver: mpsc::Receiver<SupervisorMessage>,
    llm_actor: Arc<L>,
    rag_actor: Arc<R>,
    db_pool: Option<SqlitePool>,
}

fn new_production_runner(
    receiver: mpsc::Receiver<SupervisorMessage>,
    db_pool: Option<SqlitePool>,
    model_path: std::path.PathBuf,
) -> SupervisorRunner<LlmActorHandle, RagActorHandle> {
    SupervisorRunner {
        receiver,
        llm_actor: Arc::new(LlmActorHandle::new(model_path)),
        rag_actor: Arc::new(RagActorHandle::new_with_options(None, db_pool.clone())),
        db_pool,
    }
}

impl<L, R> SupervisorRunner<L, R>
where
    L: LlmActor + Send + Sync + 'static,
    R: RagActor + Send + Sync + 'static,
{
    #[allow(dead_code)]
    fn new(
        receiver: mpsc::Receiver<SupervisorMessage>,
        llm_actor: Arc<L>,
        rag_actor: Arc<R>,
        db_pool: Option<SqlitePool>,
    ) -> Self {
        Self {
            receiver,
            llm_actor,
            rag_actor,
            db_pool,
        }
    }
    
    async fn run(mut self) {
        info!("Supervisor started");
        while let Some(msg) = self.receiver.recv().await {
            if let Err(e) = self.handle_message(msg).await {
                error!("Error handling message: {:?}", e);
            }
        }
        info!("Supervisor stopped");
    }

    #[instrument(skip(self, msg), fields(message_type = %std::any::type_name::<SupervisorMessage>()))]
    async fn handle_message(&mut self, msg: SupervisorMessage) -> Result<(), AppError> {
        match msg {
            SupervisorMessage::ProcessUserMessage {
                session_id,
                content,
                window,
                responder,
            } => {
                let result = self
                    .handle_user_message(session_id, content, window)
                    .await;
                if let Err(e) = &result {
                    error!("Error processing user message: {:?}", e);
                }
                let _ = responder.send(result);
            }
            SupervisorMessage::IngestContent {
                content,
                metadata,
                responder,
            } => {
                info!("Supervisor orchestrating ingestion...");
                let result = self.rag_actor.ingest(content, metadata).await;
                if let Err(e) = &result {
                    error!("Error ingesting content: {:?}", e);
                }
                let _ = responder.send(result);
            }
            SupervisorMessage::Shutdown => {
                info!("Supervisor shutting down...");
            }
        }
        Ok(())
    }

    #[instrument(skip(self, window))]
    async fn handle_user_message(
        &mut self,
        session_id: String,
        content: String,
        window: Option<Window>,
    ) -> Result<String, AppError> {
        info!("Supervisor received: {}", content);

        let pool = self.db_pool.as_ref().ok_or(AppError::Config(
            "Database not initialized".to_string(),
        ))?;

        // --- Database Operations ---
        let session = database::get_session(pool, &session_id).await?;
        database::add_message(pool, &session_id, "user", &content).await?;
        let session_messages = database::get_session_messages(pool, &session_id).await?;

        // --- Configuration ---
        let config = session.model_config;
        let system_prompt = Some(config.system_prompt.clone());
        let temperature = Some(config.temperature);

        // --- Thinking Steps & Analysis ---
        self.emit_thinking(&window, "thinking.analyzing").await;
        let analysis_prompt =
            format!("Analyze this request and summarize the intent (max 10 words). Request: {}", content);
        let analysis = self
            .llm_actor
            .generate_with_params(analysis_prompt, system_prompt.clone(), temperature)
            .await?;
        self.emit_thinking(&window, &format!("thinking.intent|{}", analysis.trim()))
            .await;

        // --- Context Search ---
        self.emit_thinking(&window, "thinking.searching_context")
            .await;
        let search_results = self
            .rag_actor
            .search_with_session(content.clone(), Some(session_id.clone()))
            .await?;
        let context_str = if !search_results.is_empty() {
            self.emit_thinking(
                &window,
                &format!("thinking.documents_found|{}", search_results.len()),
            )
            .await;
            search_results.join("\n\n")
        } else {
            self.emit_thinking(&window, "thinking.no_documents")
                .await;
            String::new()
        };

        // --- Generation ---
        self.emit_thinking(&window, "thinking.generating_response")
            .await;
        let conversation_history = session_messages
            .iter()
            .map(|msg| format!("{}: {}", msg.role, msg.content))
            .collect::<Vec<String>>()
            .join("\n");

        let final_prompt = build_final_prompt(&conversation_history, &context_str, &content);

        // --- Streaming Response ---
        let (chunk_tx, mut chunk_rx) = mpsc::channel(32);
        self.llm_actor
            .stream_generate_with_params(final_prompt, system_prompt, temperature, chunk_tx)
            .await?;

        let mut full_response = String::new();
        while let Some(result) = chunk_rx.recv().await {
            match result {
                Ok(token) => {
                    full_response.push_str(&token);
                    if let Some(win) = &window {
                        let _ = win.emit("chat-token", &token);
                    }
                }
                Err(e) => {
                    error!("Streaming error: {}", e);
                    // Decide if we should continue or abort
                }
            }
        }

        database::add_message(pool, &session_id, "assistant", &full_response).await?;
        Ok(full_response)
    }

    async fn emit_thinking(&self, window: &Option<Window>, step: &str) {
        if let Some(win) = window {
            let _ = win.emit("thinking-step", step);
        }
    }
}

fn build_final_prompt(conversation_history: &str, context_str: &str, content: &str) -> String {
    let mut prompt_parts = Vec::new();
    if !conversation_history.is_empty() {
        prompt_parts.push(format!("Conversation History:\n{}", conversation_history));
    }
    if !context_str.is_empty() {
        prompt_parts.push(format!("Context:\n{}", context_str));
    }
    prompt_parts.push(format!("User Request: {}", content));
    prompt_parts.join("\n\n")
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::actors::traits::{LlmActor, RagActor};
    use crate::database::init_db;
    use async_trait::async_trait;
    use std::sync::Mutex;
    use tokio::sync::mpsc;

    // --- Mock Actors ---

    #[derive(Clone)]
    struct MockLlmActor {
        generate_response: Arc<Mutex<Result<String, AppError>>>,
    }

    impl MockLlmActor {
        fn new(generate_response: Result<String, AppError>) -> Self {
            Self {
                generate_response: Arc::new(Mutex::new(generate_response)),
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
            self.generate_response.lock().unwrap().clone()
        }

        async fn stream_generate_with_params(
            &self,
            _prompt: String,
            _system_prompt: Option<String>,
            _temperature: Option<f32>,
            chunk_sender: mpsc::Sender<Result<String, AppError>>,
        ) -> Result<(), AppError> {
            let response = self.generate_response.lock().unwrap().clone();
            match response {
                Ok(res) => {
                    chunk_sender.send(Ok(res)).await.unwrap();
                    Ok(())
                }
                Err(e) => {
                    chunk_sender.send(Err(e.clone())).await.unwrap();
                    Err(e)
                }
            }
        }
    }

    struct MockRagActor {
        search_response: Result<Vec<String>, AppError>,
    }

    impl MockRagActor {
        fn new(search_response: Result<Vec<String>, AppError>) -> Self {
            Self { search_response }
        }
    }

    #[async_trait]
    impl RagActor for MockRagActor {
        async fn ingest(
            &self,
            _content: String,
            _metadata: Option<String>,
        ) -> Result<String, AppError> {
            Ok("Ingested".to_string())
        }

        async fn search_with_session(
            &self,
            _query: String,
            _session_id: Option<String>,
        ) -> Result<Vec<String>, AppError> {
            self.search_response.clone()
        }
    }

    // --- Test Setup ---

    async fn setup_supervisor_with_mocks(
        llm_response: Result<String, AppError>,
        rag_response: Result<Vec<String>, AppError>,
    ) -> (SupervisorHandle, SqlitePool) {
        let (sender, receiver) = mpsc::channel(32);
        let db_pool = init_db(Some(":memory:")).await.unwrap();
        
        // Create schema for tests
        sqlx::query("CREATE TABLE IF NOT EXISTS sessions (id TEXT PRIMARY KEY, name TEXT, model_config_json TEXT);")
            .execute(&db_pool).await.unwrap();
        sqlx::query("CREATE TABLE IF NOT EXISTS messages (id INTEGER PRIMARY KEY AUTOINCREMENT, session_id TEXT, role TEXT, content TEXT, timestamp DATETIME DEFAULT CURRENT_TIMESTAMP);")
            .execute(&db_pool).await.unwrap();


        let llm_actor = Arc::new(MockLlmActor::new(llm_response));
        let rag_actor = Arc::new(MockRagActor::new(rag_response));

        let supervisor_runner = SupervisorRunner::new(receiver, llm_actor, rag_actor, Some(db_pool.clone()));
        tokio::spawn(async move { supervisor_runner.run().await });

        (SupervisorHandle { sender }, db_pool)
    }

    // --- Tests ---

    #[tokio::test]
    async fn test_supervisor_process_message_nominal() {
        // 1. Arrange
        let llm_response = Ok("LLM response".to_string());
        let rag_response = Ok(vec!["RAG context".to_string()]);
        let (handle, db_pool) = setup_supervisor_with_mocks(llm_response, rag_response).await;
        
        let session_id = "test_session_nominal";
        let config = crate::models::ModelConfig::default();
        sqlx::query("INSERT INTO sessions (id, name, model_config_json) VALUES (?, ?, ?)")
            .bind(session_id)
            .bind("Test Session")
            .bind(serde_json::to_string(&config).unwrap())
            .execute(&db_pool).await.unwrap();

        // 2. Act
        let window = Window::default(); // Dummy window
        let result = handle.process_message(session_id.to_string(), "Hello".to_string(), &window).await;

        // 3. Assert
        assert!(result.is_ok());
        let final_response = result.unwrap();
        assert_eq!(final_response, "LLM response");

        // Verify that messages were saved to the database
        let messages: Vec<(String, String)> = sqlx::query_as("SELECT role, content FROM messages WHERE session_id = ? ORDER BY timestamp")
            .bind(session_id)
            .fetch_all(&db_pool).await.unwrap();
        
        assert_eq!(messages.len(), 2);
        assert_eq!(messages[0].0, "user");
        assert_eq!(messages[0].1, "Hello");
        assert_eq!(messages[1].0, "assistant");
        assert_eq!(messages[1].1, "LLM response");
    }
}

    #[tokio::test]
    async fn test_supervisor_llm_error_propagates() {
        // 1. Arrange
        let llm_response = Err(AppError::Actor("LLM simulation error".to_string()));
        let rag_response = Ok(vec![]);
        let (handle, db_pool) = setup_supervisor_with_mocks(llm_response, rag_response).await;

        let session_id = "test_session_llm_error";
        let config = crate::models::ModelConfig::default();
        sqlx::query("INSERT INTO sessions (id, name, model_config_json) VALUES (?, ?, ?)")
            .bind(session_id)
            .bind("Test Session")
            .bind(serde_json::to_string(&config).unwrap())
            .execute(&db_pool).await.unwrap();

        // 2. Act
        let window = Window::default();
        let result = handle.process_message(session_id.to_string(), "Hello".to_string(), &window).await;

        // 3. Assert
        assert!(result.is_err());
        if let Err(AppError::Actor(err_msg)) = result {
            assert!(err_msg.contains("LLM simulation error"));
        } else {
            panic!("Expected AppError::Actor due to LLM failure.");
        }
    }