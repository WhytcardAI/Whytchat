use crate::actors::llm::LlmActorHandle;
use crate::actors::messages::{ActorError, SupervisorMessage};
use crate::actors::rag::RagActorHandle;
use crate::database;
use log::{error, info};
use sqlx::sqlite::SqlitePool;
use tauri::{Emitter, Window};
use tokio::sync::{mpsc, oneshot};

// --- Actor Handle ---
#[derive(Clone)]
pub struct SupervisorHandle {
    sender: mpsc::Sender<SupervisorMessage>,
}

impl SupervisorHandle {
    pub fn new() -> Self {
        Self::new_with_pool(None)
    }

    pub fn new_with_pool(db_pool: Option<SqlitePool>) -> Self {
        let (sender, receiver) = mpsc::channel(32);
        let actor = SupervisorRunner::new(receiver, db_pool);
        tokio::spawn(async move { actor.run().await });
        Self { sender }
    }

    pub async fn process_message(
        &self,
        session_id: String,
        content: String,
        window: &Window,
    ) -> Result<String, ActorError> {
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
            .map_err(|_| ActorError::Internal("Supervisor closed".to_string()))?;
        recv.await
            .map_err(|_| ActorError::Internal("Supervisor failed to respond".to_string()))?
    }
}

// --- Actor Runner ---
struct SupervisorRunner {
    receiver: mpsc::Receiver<SupervisorMessage>,
    llm_actor: LlmActorHandle,
    rag_actor: RagActorHandle,
    db_pool: Option<SqlitePool>,
}

impl SupervisorRunner {
    fn new(receiver: mpsc::Receiver<SupervisorMessage>, db_pool: Option<SqlitePool>) -> Self {
        Self {
            receiver,
            llm_actor: LlmActorHandle::new(),
            rag_actor: RagActorHandle::new_with_options(None, db_pool.clone()),
            db_pool,
        }
    }

    async fn run(mut self) {
        info!("Supervisor started");
        while let Some(msg) = self.receiver.recv().await {
            self.handle_message(msg).await;
        }
        info!("Supervisor stopped");
    }

    async fn handle_message(&mut self, msg: SupervisorMessage) {
        match msg {
            SupervisorMessage::ProcessUserMessage {
                session_id,
                content,
                window,
                responder,
            } => {
                info!("Supervisor received: {}", content);

                // Save user message to database
                if let Some(ref pool) = self.db_pool {
                    if let Err(e) = database::add_message(pool, &session_id, "user", &content).await
                    {
                        error!("Failed to save user message: {}", e);
                    }
                }

                // 1. Emit: Analysis
                self.emit_thinking(&window, "Analyse de la demande...")
                    .await;

                // 2. Agent 1: Analyzer (LLM Call)
                let analysis_prompt = format!("Analyze this request and summarize the intent in French (max 10 words). Request: {}", content);
                let analysis = match self.llm_actor.generate(analysis_prompt).await {
                    Ok(res) => res.trim().to_string(),
                    Err(_) => "Analyse complexe...".to_string(),
                };

                self.emit_thinking(&window, &format!("Intention : {}", analysis))
                    .await;

                // 3. Emit: Context Search
                self.emit_thinking(&window, "Vérification des connaissances locales (RAG)...")
                    .await;

                let mut context_str = String::new();
                match self
                    .rag_actor
                    .search_with_session(content.clone(), Some(session_id.clone()))
                    .await
                {
                    Ok(results) => {
                        if !results.is_empty() {
                            self.emit_thinking(
                                &window,
                                &format!("{} documents pertinents trouvés.", results.len()),
                            )
                            .await;
                            context_str = results.join("\n\n");
                        } else {
                            self.emit_thinking(&window, "Aucun document pertinent trouvé.")
                                .await;
                        }
                    }
                    Err(e) => {
                        error!("RAG Search failed: {}", e);
                        self.emit_thinking(&window, "Erreur lors de la recherche documentaire.")
                            .await;
                    }
                }

                // 4. Emit: Generation
                self.emit_thinking(&window, "Formulation de la réponse...")
                    .await;

                // Load session history
                let session_messages = if let Some(ref pool) = self.db_pool {
                    match database::get_session_messages(pool, &session_id).await {
                        Ok(messages) => messages,
                        Err(e) => {
                            error!("Failed to load session messages: {}", e);
                            Vec::new()
                        }
                    }
                } else {
                    Vec::new()
                };

                // Build conversation history
                let mut conversation_history = String::new();
                for message in &session_messages {
                    if message.role == "user" {
                        conversation_history.push_str(&format!("User: {}\n", message.content));
                    } else if message.role == "assistant" {
                        conversation_history.push_str(&format!("Assistant: {}\n", message.content));
                    }
                }

                // 5. Agent 2: Responder (LLM Call with Streaming)
                let (chunk_tx, mut chunk_rx) = mpsc::channel(32);

                // Construct prompt with context and history
                let final_prompt = if !context_str.is_empty() {
                    if !conversation_history.is_empty() {
                        format!(
                            "Conversation History:\n{}\n\nContext:\n{}\n\nUser Request: {}",
                            conversation_history, context_str, content
                        )
                    } else {
                        format!("Context:\n{}\n\nUser Request: {}", context_str, content)
                    }
                } else if !conversation_history.is_empty() {
                    format!(
                        "Conversation History:\n{}\n\nUser Request: {}",
                        conversation_history, content
                    )
                } else {
                    content.clone()
                };

                // Spawn the streaming task
                let llm_handle = self.llm_actor.clone();
                tokio::spawn(async move {
                    let _ = llm_handle.stream_generate(final_prompt, chunk_tx).await;
                });

                // Consume chunks and emit to frontend
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
                        }
                    }
                }

                // Save assistant response to database
                if let Some(ref pool) = self.db_pool {
                    if let Err(e) =
                        database::add_message(pool, &session_id, "assistant", &full_response).await
                    {
                        error!("Failed to save assistant message: {}", e);
                    }
                }

                let _ = responder.send(Ok(full_response));
            }
            SupervisorMessage::Shutdown => {
                info!("Supervisor shutting down...");
            }
        }
    }

    async fn emit_thinking(&self, window: &Option<Window>, step: &str) {
        if let Some(win) = window {
            let _ = win.emit("thinking-step", step);
        }
    }
}
