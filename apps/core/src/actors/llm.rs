use crate::actors::messages::{ActorError, AppError, LlmMessage};
use crate::actors::traits::LlmActor;
use async_trait::async_trait;
use tracing::{error, info};
use reqwest::header::{HeaderMap, AUTHORIZATION};
use std::env;
use std::time::Duration;
use tokio::sync::{mpsc, oneshot};
use tokio::time::timeout;
use tokio::process::Command;
use std::process::Stdio;
use reqwest::Client;
use futures::StreamExt;

/// A handle to the `LlmActor`.
///
/// This struct provides a public, cloneable interface for sending messages to the
/// running LLM actor. It abstracts away the `mpsc::Sender`.
#[derive(Clone)]
pub struct LlmActorHandle {
    sender: mpsc::Sender<LlmMessage>,
}

impl LlmActorHandle {
    /// Creates a new `LlmActor` and returns a handle to it.
    ///
    /// This will spawn the `LlmActorRunner` in a new Tokio task.
    ///
    /// # Arguments
    ///
    /// * `model_path` - The path to the GGUF model file for `llama-server`.
    pub fn new(model_path: std::path::PathBuf) -> Self {
        let (sender, receiver) = mpsc::channel(32);
        let actor = LlmActorRunner::new(receiver, model_path);
        tokio::spawn(async move { actor.run().await });
        Self { sender }
    }

    /// A convenience method for generating text with default parameters.
    #[allow(dead_code)]
    pub async fn generate(&self, prompt: String) -> Result<String, AppError> {
        self.generate_with_params(prompt, None, None).await
    }
}

#[async_trait]
impl LlmActor for LlmActorHandle {
    async fn generate_with_params(&self, prompt: String, system_prompt: Option<String>, temperature: Option<f32>) -> Result<String, AppError> {
        let (send, recv) = oneshot::channel();
        let msg = LlmMessage::GenerateWithParams {
            prompt,
            system_prompt,
            temperature,
            responder: send,
        };

        self.sender
            .send(msg)
            .await
            .map_err(|e| AppError::Actor(e.to_string()))?;
        timeout(Duration::from_secs(60), recv)
            .await?
            .map_err(|e| AppError::Actor(e.to_string()))?
    }

    async fn stream_generate_with_params(
        &self,
        prompt: String,
        system_prompt: Option<String>,
        temperature: Option<f32>,
        chunk_sender: mpsc::Sender<Result<String, AppError>>,
    ) -> Result<(), AppError> {
        let (send, recv) = oneshot::channel();
        let msg = LlmMessage::StreamGenerateWithParams {
            prompt,
            system_prompt,
            temperature,
            chunk_sender,
            responder: send,
        };

        self.sender
            .send(msg)
            .await
            .map_err(|e| AppError::Actor(e.to_string()))?;
        timeout(Duration::from_secs(300), recv) // Longer timeout for streaming
            .await?
            .map_err(|e| AppError::Actor(e.to_string()))?
    }
}

// --- Constants ---
const COMPLETION_TIMEOUT: Duration = Duration::from_secs(120);
const STREAM_CHUNK_TIMEOUT: Duration = Duration::from_secs(30);

// --- Actor Runner (Internal Logic) ---
struct LlmActorRunner {
    receiver: mpsc::Receiver<LlmMessage>,
    child: Option<tokio::process::Child>,
    server_url: String,
    model_path: std::path::PathBuf,
    client: Client,
    auth_token: Option<String>,
}

impl Drop for LlmActorRunner {
    fn drop(&mut self) {
        if let Some(mut child) = self.child.take() {
            // Try to kill the child process - cross-platform approach
            // We use start_kill() which is non-blocking and doesn't require async
            match child.start_kill() {
                Ok(_) => info!("llama-server process termination initiated"),
                Err(e) => error!("Failed to kill llama-server process: {}", e),
            }
        }
    }
}

impl LlmActorRunner {
    fn new(receiver: mpsc::Receiver<LlmMessage>, model_path: std::path::PathBuf) -> Self {
        let auth_token = env::var("LLAMA_AUTH_TOKEN").ok();

        Self {
            receiver,
            child: None,
            server_url: "http://localhost:8080".to_string(),
            model_path,
            client: Client::new(),
            auth_token,
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

    async fn start_server(&mut self) -> Result<(), AppError> {
        // Enforce auth token presence.
        if self.auth_token.is_none() {
            let error_msg = "Security alert: LLAMA_AUTH_TOKEN is not set. The application cannot proceed without a token to secure communication with the external llama-server process.";
            error!("{}", error_msg);
            return Err(AppError::Config(error_msg.to_string()));
        }

        info!("Starting llama-server with model: {:?}", self.model_path);

        // Check if llama-server binary exists in PATH
        if which::which("llama-server").is_err() {
            return Err(AppError::Config(
                "llama-server binary not found in PATH. Please ensure llama.cpp is installed and llama-server is in your PATH.".to_string()
            ));
        }

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
            .map_err(|e| AppError::Io(e.into()))?;

        self.child = Some(child);

        // Health check with retries
        let max_retries = 30;
        let retry_interval = Duration::from_secs(1);
        let health_endpoint = format!("{}/health", self.server_url);

        for attempt in 1..=max_retries {
            tokio::time::sleep(retry_interval).await;

            match self.client.get(&health_endpoint).send().await {
                Ok(response) if response.status().is_success() => {
                    info!("llama-server is ready after {} attempts", attempt);
                    return Ok(());
                }
                Ok(response) => {
                    info!("Server responded with status {} on attempt {}", response.status(), attempt);
                }
                Err(e) => {
                    info!("Health check attempt {} failed: {}", attempt, e);
                }
            }
        }

        Err(AppError::Actor(format!(
            "llama-server failed to become ready after {} seconds",
            max_retries
        )))
    }

    fn build_request(&self, endpoint: &str, payload: &serde_json::Value) -> reqwest::RequestBuilder {
        let mut headers = HeaderMap::new();
        if let Some(token) = &self.auth_token {
            let auth_value = format!("Bearer {}", token);
            headers.insert(
                AUTHORIZATION,
                auth_value.parse().expect("Failed to parse auth token"),
            );
        }

        self.client
            .post(format!("{}/{}", self.server_url, endpoint))
            .headers(headers)
            .json(payload)
    }

    async fn handle_message(&mut self, msg: LlmMessage) {
        match msg {
            LlmMessage::Generate { prompt, system_prompt, temperature, responder } => {
                let result = self.generate_completion(prompt, system_prompt, temperature).await;
                let _ = responder.send(result);
            }
            LlmMessage::GenerateWithParams { prompt, system_prompt, temperature, responder } => {
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
            LlmMessage::StreamGenerateWithParams {
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

    async fn generate_completion(&self, prompt: String, system_prompt: Option<String>, temperature: Option<f32>) -> Result<String, AppError> {
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

        let request_future = self.build_request("completion", &payload).send();

        let res = timeout(COMPLETION_TIMEOUT, request_future)
            .await??;
        
        let status = res.status();

        if !status.is_success() {
            let body = res.text().await.unwrap_or_default();
            return Err(AppError::Actor(format!(
                "Completion request failed with status {}: {}",
                status,
                body
            )));
        }

        let json: serde_json::Value = res
            .json()
            .await
            .map_err(|e| AppError::Actor(e.to_string()))?;

        Ok(json["content"].as_str().unwrap_or("").to_string())
    }

    async fn stream_completion(
        &self,
        prompt: String,
        system_prompt: Option<String>,
        temperature: Option<f32>,
        chunk_sender: mpsc::Sender<Result<String, AppError>>,
    ) -> Result<(), AppError> {
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

        let request_future = self.build_request("completion", &payload).send();

        let res = timeout(COMPLETION_TIMEOUT, request_future)
            .await??;

        let mut stream = res.bytes_stream();

        while let Some(chunk_result) = timeout(STREAM_CHUNK_TIMEOUT, stream.next()).await {
            let chunk = chunk_result.transpose().map_err(|e| AppError::Actor(format!("Stream chunk timeout: {}", e)))?.flatten();
            
            if chunk.is_none() {
                break; // End of stream
            }
            let chunk = chunk.unwrap();
            let chunk = chunk.map_err(|e| AppError::Actor(e.to_string()))?;
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
#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};
    use serde_json::json;
    use std::path::PathBuf;

    async fn setup_test_actor(server_url: String) -> LlmActorHandle {
        let (sender, receiver) = mpsc::channel(32);
        
        // Use a dummy model path for tests as we are not starting the real server.
        let model_path = PathBuf::from("dummy/model.gguf");
        
        let mut actor = LlmActorRunner::new(receiver, model_path);
        
        // Override the server_url to point to our mock server
        actor.server_url = server_url;
        
        // We don't call `start_server` here, so the real process is not launched.
        // We manually spawn the actor runner.
        tokio::spawn(async move {
            info!("Mock LlmActor started");
            while let Some(msg) = actor.receiver.recv().await {
                actor.handle_message(msg).await;
            }
            info!("Mock LlmActor stopped");
        });
        
        LlmActorHandle { sender }
    }

    #[tokio::test]
    async fn test_llm_generate_completion_success() {
        // 1. Arrange
        let mock_server = MockServer::start().await;
        let handle = setup_test_actor(mock_server.uri()).await;

        let expected_response = json!({
            "content": "This is a test response.",
            "model": "dummy_model",
            "prompt": "Hello",
            "stop": true
        });

        Mock::given(method("POST"))
            .and(path("/completion"))
            .respond_with(ResponseTemplate::new(200).set_body_json(expected_response))
            .mount(&mock_server)
            .await;

        // 2. Act
        let result = handle.generate("Hello".to_string()).await;

        // 3. Assert
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "This is a test response.");
    }

    #[tokio::test]
    async fn test_llm_generate_completion_server_error() {
        // 1. Arrange
        let mock_server = MockServer::start().await;
        let handle = setup_test_actor(mock_server.uri()).await;

        Mock::given(method("POST"))
            .and(path("/completion"))
            .respond_with(ResponseTemplate::new(500).set_body_string("Internal Server Error"))
            .mount(&mock_server)
            .await;

        // 2. Act
        let result = handle.generate("Hello".to_string()).await;
        
        // 3. Assert
        assert!(result.is_err());
        if let Err(AppError::Actor(err_msg)) = result {
            assert!(err_msg.contains("Completion request failed with status 500"));
            assert!(err_msg.contains("Internal Server Error"));
        } else {
            panic!("Expected AppError::Actor, got something else.");
        }
    }
}
