use crate::actors::messages::{ActorError, LlmMessage};
use log::{error, info};
use std::time::Duration;
use tokio::sync::{mpsc, oneshot};
use tokio::process::Command;
use std::process::Stdio;
use reqwest::Client;
use futures::StreamExt;

// --- Actor Handle (Public API) ---
#[derive(Clone)]
pub struct LlmActorHandle {
    sender: mpsc::Sender<LlmMessage>,
}

impl LlmActorHandle {
    pub fn new(model_path: std::path::PathBuf) -> Self {
        let (sender, receiver) = mpsc::channel(32);
        let actor = LlmActorRunner::new(receiver, model_path);
        tokio::spawn(async move { actor.run().await });
        Self { sender }
    }

    #[allow(dead_code)]
    pub async fn generate(&self, prompt: String) -> Result<String, ActorError> {
        self.generate_with_params(prompt, None, None).await
    }

    pub async fn generate_with_params(&self, prompt: String, system_prompt: Option<String>, temperature: Option<f32>) -> Result<String, ActorError> {
        let (send, recv) = oneshot::channel();
        let msg = LlmMessage::Generate {
            prompt,
            system_prompt,
            temperature,
            responder: send,
        };

        self.sender
            .send(msg)
            .await
            .map_err(|_| ActorError::Internal("LLM Actor closed".to_string()))?;
        recv.await
            .map_err(|_| ActorError::Internal("LLM Actor failed to respond".to_string()))?
    }

    #[allow(dead_code)]
    pub async fn stream_generate(
        &self,
        prompt: String,
        chunk_sender: mpsc::Sender<Result<String, ActorError>>,
    ) -> Result<(), ActorError> {
        self.stream_generate_with_params(prompt, None, None, chunk_sender).await
    }

    pub async fn stream_generate_with_params(
        &self,
        prompt: String,
        system_prompt: Option<String>,
        temperature: Option<f32>,
        chunk_sender: mpsc::Sender<Result<String, ActorError>>,
    ) -> Result<(), ActorError> {
        let (send, recv) = oneshot::channel();
        let msg = LlmMessage::StreamGenerate {
            prompt,
            system_prompt,
            temperature,
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
    child: Option<tokio::process::Child>,
    server_url: String,
    model_path: std::path::PathBuf,
    client: Client,
}

impl LlmActorRunner {
    fn new(receiver: mpsc::Receiver<LlmMessage>, model_path: std::path::PathBuf) -> Self {
        Self {
            receiver,
            child: None,
            server_url: "http://localhost:8080".to_string(),
            model_path,
            client: Client::new(),
        }
    }

    async fn run(mut self) {
        info!("LlmActor started");

        // Start the llama-server
        if let Err(e) = self.start_server().await {
            error!("Failed to start llama-server: {}", e);
            return;
        }

        while let Some(msg) = self.receiver.recv().await {
            self.handle_message(msg).await;
        }

        info!("LlmActor stopped");
    }

    async fn start_server(&mut self) -> Result<(), ActorError> {
        info!("Starting llama-server with model: {:?}", self.model_path);

        let child = Command::new("llama-server")
            .arg("-m")
            .arg(&self.model_path)
            .arg("--host")
            .arg("127.0.0.1")
            .arg("--port")
            .arg("8080")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| ActorError::Internal(format!("Failed to spawn llama-server: {}", e)))?;

        self.child = Some(child);

        // Wait a bit for server to start
        tokio::time::sleep(Duration::from_secs(2)).await;

        Ok(())
    }

    async fn handle_message(&mut self, msg: LlmMessage) {
        match msg {
            LlmMessage::Generate { prompt, system_prompt, temperature, responder } => {
                let result = self.generate_completion(prompt, system_prompt, temperature).await;
                let _ = responder.send(result);
            }
            LlmMessage::StreamGenerate {
                prompt,
                system_prompt,
                temperature,
                chunk_sender,
                responder,
            } => {
                let result = self.stream_completion(prompt, system_prompt, temperature, chunk_sender).await;
                let _ = responder.send(result);
            }
        }
    }

    async fn generate_completion(&self, prompt: String, system_prompt: Option<String>, temperature: Option<f32>) -> Result<String, ActorError> {
        info!("LLM Generating for prompt: {}", prompt);

        let mut payload = serde_json::json!({
            "prompt": prompt,
            "stream": false,
            "n_predict": 100
        });

        // Add system prompt if provided
        if let Some(system) = system_prompt {
            payload["system_prompt"] = serde_json::Value::String(system);
        }

        // Add temperature if provided
        if let Some(temp) = temperature {
            payload["temperature"] = serde_json::Value::Number(serde_json::Number::from_f64(temp as f64).unwrap());
        }

        let res = self.client
            .post(format!("{}/completion", self.server_url))
            .json(&payload)
            .send()
            .await
            .map_err(|e| ActorError::Internal(e.to_string()))?;

        let json: serde_json::Value = res.json().await.map_err(|e| ActorError::Internal(e.to_string()))?;

        Ok(json["content"].as_str().unwrap_or("").to_string())
    }

    async fn stream_completion(
        &self,
        prompt: String,
        system_prompt: Option<String>,
        temperature: Option<f32>,
        chunk_sender: mpsc::Sender<Result<String, ActorError>>,
    ) -> Result<(), ActorError> {
        info!("LLM Streaming for prompt: {}", prompt);

        let mut payload = serde_json::json!({
            "prompt": prompt,
            "stream": true,
            "n_predict": 100
        });

        // Add system prompt if provided
        if let Some(system) = system_prompt {
            payload["system_prompt"] = serde_json::Value::String(system);
        }

        // Add temperature if provided
        if let Some(temp) = temperature {
            payload["temperature"] = serde_json::Value::Number(serde_json::Number::from_f64(temp as f64).unwrap());
        }

        let res = self.client
            .post(format!("{}/completion", self.server_url))
            .json(&payload)
            .send()
            .await
            .map_err(|e| ActorError::Internal(e.to_string()))?;

        let mut stream = res.bytes_stream();

        while let Some(chunk) = stream.next().await {
            let chunk = chunk.map_err(|e| ActorError::Internal(e.to_string()))?;
            let text = String::from_utf8_lossy(&chunk);
            // Parse SSE
            for line in text.lines() {
                if let Some(data) = line.strip_prefix("data: ") {
                    if data == "[DONE]" {
                        break;
                    }
                    if let Ok(json) = serde_json::from_str::<serde_json::Value>(data) {
                        if let Some(content) = json["content"].as_str() {
                            let _ = chunk_sender.send(Ok(content.to_string())).await;
                        }
                    }
                }
            }
        }

        Ok(())
    }
}
