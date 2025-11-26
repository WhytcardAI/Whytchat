use crate::actors::messages::{ActorError, AppError, LlmMessage};
use crate::actors::traits::LlmActor;
use async_trait::async_trait;
use tracing::{error, info, warn};
use reqwest::header::{HeaderMap, AUTHORIZATION};
use std::env;
use std::time::{Duration, Instant};
use tokio::sync::{mpsc, oneshot};
use uuid::Uuid;
use tokio::time::timeout;
use tokio::process::Command;
use std::process::Stdio;
use reqwest::Client;
use futures::StreamExt;
use crate::fs_manager::PortablePathManager;

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
            .map_err(|e| AppError::Actor(crate::actors::messages::ActorError::Internal(e.to_string())))?;
        timeout(Duration::from_secs(60), recv)
            .await?
            .map_err(|e| AppError::Actor(crate::actors::messages::ActorError::Internal(e.to_string())))?
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
            .map_err(|e| AppError::Actor(crate::actors::messages::ActorError::Internal(e.to_string())))?;
        timeout(Duration::from_secs(300), recv) // Longer timeout for streaming
            .await?
            .map_err(|e| AppError::Actor(crate::actors::messages::ActorError::Internal(e.to_string())))?
    }
}

// --- Constants ---
const COMPLETION_TIMEOUT: Duration = Duration::from_secs(120);
const STREAM_CHUNK_TIMEOUT: Duration = Duration::from_secs(30);
const MAX_RESTART_ATTEMPTS: u32 = 3;
const RESET_TIMEOUT: Duration = Duration::from_secs(60);

// --- Actor Runner (Internal Logic) ---
struct LlmActorRunner {
    receiver: mpsc::Receiver<LlmMessage>,
    child: Option<tokio::process::Child>,
    server_url: String,
    model_path: std::path::PathBuf,
    client: Client,
    auth_token: Option<String>,
    // Circuit Breaker State
    restart_attempts: u32,
    last_restart: Instant,
}

impl Drop for LlmActorRunner {
    fn drop(&mut self) {
        self.stop_server_sync();
    }
}

impl LlmActorRunner {
    fn new(receiver: mpsc::Receiver<LlmMessage>, model_path: std::path::PathBuf) -> Self {
        // Prioritize env var, fallback to generated UUID
        let auth_token = env::var("LLAMA_AUTH_TOKEN").ok().or_else(|| {
            let token = Uuid::new_v4().to_string();
            warn!("LLAMA_AUTH_TOKEN not set. Generated temporary token for this session: {}", token);
            Some(token)
        });

        Self {
            receiver,
            child: None,
            server_url: "http://localhost:8080".to_string(),
            model_path,
            client: Client::new(),
            auth_token,
            restart_attempts: 0,
            last_restart: Instant::now(),
        }
    }

    async fn run(mut self) {
        info!("LlmActor started");
        // Note: Server is NOT started automatically here. It's lazy-loaded.

        loop {
            tokio::select! {
                msg = self.receiver.recv() => {
                    match msg {
                        Some(msg) => {
                            self.handle_message(msg).await;
                        }
                        None => break, // Channel closed
                    }
                }
                // Inactivity timeout (e.g., 5 minutes)
                // Only active if the child process exists (server is running)
                _ = tokio::time::sleep(Duration::from_secs(300)), if self.child.is_some() => {
                    info!("LLM Server inactivity timeout. Stopping server.");
                    self.stop_server();
                }
            }
        }

        info!("LlmActor stopped");
    }

    fn stop_server(&mut self) {
        if let Some(mut child) = self.child.take() {
            // Try to kill the child process - cross-platform approach
            match child.start_kill() {
                Ok(_) => info!("llama-server process termination initiated"),
                Err(e) => error!("Failed to kill llama-server process: {}", e),
            }
        }
    }

    /// Synchronous version of stop_server for use in Drop
    /// Uses platform-specific process killing for reliability
    fn stop_server_sync(&mut self) {
        if let Some(child) = self.child.take() {
            let pid = match child.id() {
                Some(pid) => pid,
                None => {
                    info!("llama-server process already exited");
                    return;
                }
            };

            info!("Stopping llama-server process (PID: {})", pid);

            // Platform-specific process killing
            #[cfg(windows)]
            {
                // On Windows, use taskkill for reliable process termination
                let _ = std::process::Command::new("taskkill")
                    .args(["/F", "/PID", &pid.to_string()])
                    .output();
                info!("llama-server process killed via taskkill (PID: {})", pid);
            }

            #[cfg(not(windows))]
            {
                // On Unix, use SIGKILL for immediate termination
                use std::os::unix::process::CommandExt;
                let _ = std::process::Command::new("kill")
                    .args(["-9", &pid.to_string()])
                    .output();
                info!("llama-server process killed via SIGKILL (PID: {})", pid);
            }
        }
    }

    async fn start_server(&mut self) -> Result<(), AppError> {
        if self.child.is_some() {
            return Ok(());
        }

        // Circuit Breaker Logic
        if self.last_restart.elapsed() > RESET_TIMEOUT {
            if self.restart_attempts > 0 {
                info!("Circuit breaker reset after timeout. Resetting attempts.");
                self.restart_attempts = 0;
            }
        } else if self.restart_attempts >= MAX_RESTART_ATTEMPTS {
            let msg = format!(
                "Circuit breaker OPEN. Too many restart attempts ({}) in the last {} seconds.",
                self.restart_attempts,
                RESET_TIMEOUT.as_secs()
            );
            error!("{}", msg);
            return Err(AppError::Actor(ActorError::Internal(msg)));
        }

        self.restart_attempts += 1;
        self.last_restart = Instant::now();
        info!(
            "Starting llama-server with model: {:?} (Attempt {}/{})",
            self.model_path, self.restart_attempts, MAX_RESTART_ATTEMPTS
        );

        // Determine llama-server executable path
        let server_bin_name = if cfg!(windows) { "llama-server.exe" } else { "llama-server" };
        let mut server_path = std::path::PathBuf::from(server_bin_name);

        // Check if in PATH
        if which::which(server_bin_name).is_err() {
            // Not in PATH, check tools dir
            let tools_dir = PortablePathManager::tools_dir();
            let local_server_path = tools_dir.join("llama").join(server_bin_name);

            if local_server_path.exists() {
                server_path = local_server_path;
                info!("Found llama-server at {:?}", server_path);
            } else {
                 return Err(AppError::Config(
                    "llama-server binary not found in PATH or tools directory. Please ensure llama.cpp is installed.".to_string()
                ));
            }
        }

        let mut cmd = Command::new(server_path);
        cmd.arg("-m")
            .arg(&self.model_path)
            .arg("--host")
            .arg("127.0.0.1")
            .arg("--port")
            .arg("8080")
            .arg("-c")
            .arg("8192")  // Context size: 8K tokens
            .arg("-np")
            .arg("2");    // Support 2 parallel requests

        // Enable GPU acceleration if available (will be ignored if no GPU)
        cmd.arg("-ngl").arg("99");

        if let Some(token) = &self.auth_token {
            cmd.arg("--api-key").arg(token);
        }

        let child = cmd
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(AppError::Io)?;

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

        Err(AppError::Actor(ActorError::Internal(format!(
            "llama-server failed to become ready after {} seconds",
            max_retries
        ))))
    }

    fn build_request(&self, endpoint: &str, payload: &serde_json::Value) -> Result<reqwest::RequestBuilder, AppError> {
        let mut headers = HeaderMap::new();
        if let Some(token) = &self.auth_token {
            let auth_value = format!("Bearer {}", token);
            headers.insert(
                AUTHORIZATION,
                auth_value.parse().map_err(|e| {
                    AppError::Actor(ActorError::Internal(format!("Failed to parse auth token: {}", e)))
                })?,
            );
        }

        Ok(self.client
            .post(format!("{}/{}", self.server_url, endpoint))
            .headers(headers)
            .json(payload))
    }

    async fn handle_message(&mut self, msg: LlmMessage) {
        // Ensure server is running before processing message
        if let Err(e) = self.start_server().await {
            error!("Failed to start LLM server on demand: {}", e);
            self.respond_with_error(msg, e);
            return;
        }

        match msg {
            LlmMessage::Generate { prompt, system_prompt, temperature, responder } => {
                let result = self.generate_completion(prompt, system_prompt, temperature).await;
                if responder.send(result).is_err() {
                    warn!("Failed to send generate response (channel closed)");
                }
            }
            LlmMessage::GenerateWithParams { prompt, system_prompt, temperature, responder } => {
                let result = self.generate_completion(prompt, system_prompt, temperature).await;
                if responder.send(result).is_err() {
                    warn!("Failed to send generate_with_params response (channel closed)");
                }
            }
            LlmMessage::StreamGenerate {
                prompt,
                system_prompt,
                temperature,
                chunk_sender,
                responder,
            } => {
                let result = self.stream_completion(prompt, system_prompt, temperature, chunk_sender).await;
                if responder.send(result).is_err() {
                    warn!("Failed to send stream_generate response (channel closed)");
                }
            }
            LlmMessage::StreamGenerateWithParams {
                prompt,
                system_prompt,
                temperature,
                chunk_sender,
                responder,
            } => {
                let result = self.stream_completion(prompt, system_prompt, temperature, chunk_sender).await;
                if responder.send(result).is_err() {
                    warn!("Failed to send stream_generate_with_params response (channel closed)");
                }
            }
        }
    }

    fn respond_with_error(&self, msg: LlmMessage, error: AppError) {
        match msg {
            LlmMessage::Generate { responder, .. } |
            LlmMessage::GenerateWithParams { responder, .. } => {
                if responder.send(Err(error)).is_err() {
                    warn!("Failed to send error response to supervisor (channel closed)");
                }
            }
            LlmMessage::StreamGenerate { responder, .. } |
            LlmMessage::StreamGenerateWithParams { responder, .. } => {
                 if responder.send(Err(error)).is_err() {
                     warn!("Failed to send error response to supervisor (channel closed)");
                 }
            }
        }
    }

    async fn generate_completion(&self, prompt: String, system_prompt: Option<String>, temperature: Option<f32>) -> Result<String, AppError> {
        info!("LLM Generating for prompt: {}", prompt);

        // Build the full prompt using ChatML format for Qwen2.5
        let full_prompt = if let Some(system) = &system_prompt {
            if !system.is_empty() {
                format!(
                    "<|im_start|>system\n{}<|im_end|>\n<|im_start|>user\n{}<|im_end|>\n<|im_start|>assistant\n",
                    system, prompt
                )
            } else {
                format!(
                    "<|im_start|>user\n{}<|im_end|>\n<|im_start|>assistant\n",
                    prompt
                )
            }
        } else {
            format!(
                "<|im_start|>user\n{}<|im_end|>\n<|im_start|>assistant\n",
                prompt
            )
        };

        let mut payload = serde_json::json!({
            "prompt": full_prompt,
            "stream": false,
            "n_predict": 2048,
            "top_k": 40,
            "top_p": 0.95,
            "min_p": 0.05,
            "repeat_penalty": 1.1,
            "repeat_last_n": 64,
            "stop": ["<|im_end|>", "<|im_start|>"]
        });

        // Add temperature if provided, otherwise use 0.7 as default
        let temp = temperature.unwrap_or(0.7);
        payload["temperature"] = serde_json::Value::Number(
            serde_json::Number::from_f64(temp as f64).unwrap_or_else(|| {
                warn!("Invalid temperature value: {}. Using default 0.7.", temp);
                // NOTE: from_f64(0.7) is guaranteed to succeed since 0.7 is a valid float
                serde_json::Number::from_f64(0.7).expect("0.7 is a valid float constant")
            }),
        );

        let request_future = self.build_request("completion", &payload)?.send();

        let res = timeout(COMPLETION_TIMEOUT, request_future)
            .await??;

        let status = res.status();

        if !status.is_success() {
            let body = res.text().await.unwrap_or_default();
            return Err(AppError::Actor(ActorError::LlmError(format!(
                "Completion request failed with status {}: {}",
                status,
                body
            ))));
        }

        let json: serde_json::Value = res
            .json()
            .await
            .map_err(|e| AppError::Actor(ActorError::Internal(e.to_string())))?;

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

        // Build the full prompt using ChatML format for Qwen2.5
        let full_prompt = if let Some(system) = &system_prompt {
            if !system.is_empty() {
                format!(
                    "<|im_start|>system\n{}<|im_end|>\n<|im_start|>user\n{}<|im_end|>\n<|im_start|>assistant\n",
                    system, prompt
                )
            } else {
                format!(
                    "<|im_start|>user\n{}<|im_end|>\n<|im_start|>assistant\n",
                    prompt
                )
            }
        } else {
            format!(
                "<|im_start|>user\n{}<|im_end|>\n<|im_start|>assistant\n",
                prompt
            )
        };

        let mut payload = serde_json::json!({
            "prompt": full_prompt,
            "stream": true,
            "n_predict": 2048,
            "top_k": 40,
            "top_p": 0.95,
            "min_p": 0.05,
            "repeat_penalty": 1.1,
            "repeat_last_n": 64,
            "stop": ["<|im_end|>", "<|im_start|>"]
        });

        // Add temperature if provided, otherwise use 0.7 as default
        let temp = temperature.unwrap_or(0.7);
        payload["temperature"] = serde_json::Value::Number(
            serde_json::Number::from_f64(temp as f64).unwrap_or_else(|| {
                warn!("Invalid temperature value: {}. Using default 0.7.", temp);
                // NOTE: from_f64(0.7) is guaranteed to succeed since 0.7 is a valid float
                serde_json::Number::from_f64(0.7).expect("0.7 is a valid float constant")
            }),
        );

        let request_future = self.build_request("completion", &payload)?.send();

        let res = timeout(COMPLETION_TIMEOUT, request_future)
            .await??;

        let mut stream = res.bytes_stream();

        loop {
            match timeout(STREAM_CHUNK_TIMEOUT, stream.next()).await {
                Ok(Some(chunk_result)) => {
                    let chunk = chunk_result.map_err(|e| AppError::Actor(ActorError::Internal(format!("Stream chunk error: {}", e))))?;

                    let text = String::from_utf8_lossy(&chunk);
                    // Parse SSE
                    for line in text.lines() {
                        if let Some(data) = line.strip_prefix("data: ") {
                            if data == "[DONE]" {
                                return Ok(());
                            }
                            if let Ok(json) = serde_json::from_str::<serde_json::Value>(data) {
                                // Support both raw 'content' (legacy/completion) and OpenAI 'choices[0].delta.content' formats
                                let content_opt = json["content"].as_str()
                                    .or_else(|| json["choices"].get(0)
                                        .and_then(|c| c.get("delta"))
                                        .and_then(|d| d.get("content"))
                                        .and_then(|c| c.as_str()));

                                if let Some(content) = content_opt {
                                    if chunk_sender.send(Ok(content.to_string())).await.is_err() {
                                        warn!("Stream receiver dropped, stopping stream");
                                        return Ok(());
                                    }
                                }
                            }
                        }
                    }
                }
                Ok(None) => break, // End of stream
                Err(e) => return Err(AppError::Actor(ActorError::Internal(format!("Stream chunk timeout: {}", e)))),
            }
        }

        Ok(())
    }
}

