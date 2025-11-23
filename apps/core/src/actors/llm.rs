use crate::actors::messages::{ActorError, LlmMessage};
use log::{error, info};
use std::time::Duration;
use tokio::sync::{mpsc, oneshot};

// --- Actor Handle (Public API) ---
#[derive(Clone)]
pub struct LlmActorHandle {
    sender: mpsc::Sender<LlmMessage>,
}

impl LlmActorHandle {
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::channel(32);
        let actor = LlmActorRunner::new(receiver);
        tokio::spawn(async move { actor.run().await });
        Self { sender }
    }

    pub async fn generate(&self, prompt: String) -> Result<String, ActorError> {
        let (send, recv) = oneshot::channel();
        let msg = LlmMessage::Generate {
            prompt,
            responder: send,
        };

        self.sender
            .send(msg)
            .await
            .map_err(|_| ActorError::Internal("LLM Actor closed".to_string()))?;
        recv.await
            .map_err(|_| ActorError::Internal("LLM Actor failed to respond".to_string()))?
    }

    pub async fn stream_generate(
        &self,
        prompt: String,
        chunk_sender: mpsc::Sender<Result<String, ActorError>>,
    ) -> Result<(), ActorError> {
        let (send, recv) = oneshot::channel();
        let msg = LlmMessage::StreamGenerate {
            prompt,
            chunk_sender,
            responder: send,
        };

        self.sender
            .send(msg)
            .await
            .map_err(|_| ActorError::Internal("LLM Actor closed".to_string()))?;
        recv.await
            .map_err(|_| ActorError::Internal("LLM Actor failed to respond".to_string()))?
    }
}

// --- Actor Runner (Internal Logic) ---
struct LlmActorRunner {
    receiver: mpsc::Receiver<LlmMessage>,
}

impl LlmActorRunner {
    fn new(receiver: mpsc::Receiver<LlmMessage>) -> Self {
        Self {
            receiver,
        }
    }

    async fn run(mut self) {
        info!("LlmActor started");

        while let Some(msg) = self.receiver.recv().await {
            self.handle_message(msg).await;
        }

        info!("LlmActor stopped");
    }

    async fn handle_message(&mut self, msg: LlmMessage) {
        match msg {
            LlmMessage::Generate { prompt, responder } => {
                let result = self.generate_completion(prompt).await;
                let _ = responder.send(result);
            }
            LlmMessage::StreamGenerate {
                prompt,
                chunk_sender,
                responder,
            } => {
                let result = self.stream_completion(prompt, chunk_sender).await;
                let _ = responder.send(result);
            }
        }
    }

    async fn generate_completion(&self, prompt: String) -> Result<String, ActorError> {
        info!("LLM Generating for prompt: {}", prompt);

        // Mock response for now
        Ok(format!("Mock response to: {}", prompt))
    }

    async fn stream_completion(
        &self,
        prompt: String,
        chunk_sender: mpsc::Sender<Result<String, ActorError>>,
    ) -> Result<(), ActorError> {
        info!("LLM Streaming for prompt: {}", prompt);

        // Mock streaming response
        let response = format!("Réponse simulée à : {}", prompt);
        let words: Vec<&str> = response.split_whitespace().collect();
        
        for word in words {
            tokio::time::sleep(Duration::from_millis(100)).await;
            let _ = chunk_sender.send(Ok(format!("{} ", word))).await;
        }

        Ok(())
    }
}
