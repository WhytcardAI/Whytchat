use crate::actors::messages::{ActorError, LlmMessage};
use crate::fs_manager::PortablePathManager;
use futures::StreamExt;
use log::{error, info};
use std::io::Write;
use std::path::PathBuf;
use std::process::{Child, Command};
use std::time::Duration;
use tokio::sync::{mpsc, oneshot};

// --- Actor Handle (Public API) ---
#[derive(Clone)]
pub struct LlmActorHandle {
    sender: mpsc::Sender<LlmMessage>,
}

impl LlmActorHandle {
    pub fn new() -> Self {
        let models_dir = PortablePathManager::models_dir();
        let default_model_path = models_dir.join("qwen2.5-7b-instruct-q4_k_m.gguf");
        Self::new_with_model_path(default_model_path)
    }

    pub fn new_with_model_path(model_path: PathBuf) -> Self {
        let (sender, receiver) = mpsc::channel(32);
        let actor = LlmActorRunner::new(receiver, model_path);
        tokio::spawn(async move { actor.run().await });
        Self { sender }
    }

    pub async fn generate(&self, prompt: String) -> Result<String, ActorError> {
        self.generate_with_params(prompt, None, None).await
    }

    pub async fn generate_with_params(
        &self,
        prompt: String,
        system_prompt: Option<String>,
        temperature: Option<f32>,
    ) -> Result<String, ActorError> {
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
            .map_err(|_| ActorError::Internal("LLM Actor closed".to_string()))?;
        recv.await
            .map_err(|_| ActorError::Internal("LLM Actor failed to respond".to_string()))?
    }

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
            .map_err(|_| ActorError::Internal("LLM Actor closed".to_string()))?;
        recv.await
            .map_err(|_| ActorError::Internal("LLM Actor failed to respond".to_string()))?
    }
}

// --- Actor Runner (Internal Logic) ---
struct LlmActorRunner {
    receiver: mpsc::Receiver<LlmMessage>,
    client: reqwest::Client,
    server_process: Option<Child>,
    model_path: PathBuf,
}

impl LlmActorRunner {
    fn new(receiver: mpsc::Receiver<LlmMessage>, model_path: PathBuf) -> Self {
        Self {
            receiver,
            client: reqwest::Client::new(),
            server_process: None,
            model_path,
        }
    }

    async fn run(mut self) {
        info!("LlmActor started");

        // Start the llama-server
        match self.start_server().await {
            Ok(_) => {
                // Wait for server to be ready with health check
                info!("Waiting for llama-server to initialize...");
                if let Err(e) = self.wait_for_server_ready().await {
                    error!("llama-server failed to become ready: {}", e);
                    // We continue anyway, maybe it's a temporary glitch or already running
                } else {
                    info!("llama-server is ready!");
                }
            }
            Err(e) => {
                error!("Failed to start llama-server: {}", e);
            }
        }

        while let Some(msg) = self.receiver.recv().await {
            self.handle_message(msg).await;
        }

        // Cleanup
        if let Some(mut child) = self.server_process {
            info!("Stopping llama-server...");
            let _ = child.kill();
        }
        info!("LlmActor stopped");
    }

    async fn ensure_model_exists(&self) -> Result<PathBuf, String> {
        let models_dir = PortablePathManager::models_dir();
        let model_filename = "qwen2.5-7b-instruct-q4_k_m.gguf";
        let model_path = models_dir.join(model_filename);
        let model_url = "https://huggingface.co/Qwen/Qwen2.5-7B-Instruct-GGUF/resolve/main/qwen2.5-7b-instruct-q4_k_m.gguf";

        if model_path.exists() {
            return Ok(model_path);
        }

        info!(
            "Model not found at {:?}. Starting download from {}...",
            model_path, model_url
        );

        let res = self
            .client
            .get(model_url)
            .send()
            .await
            .map_err(|e| format!("Failed to start download: {}", e))?;

        if !res.status().is_success() {
            return Err(format!("Download failed with status: {}", res.status()));
        }

        let total_size = res.content_length().unwrap_or(0);
        let mut file = std::fs::File::create(&model_path)
            .map_err(|e| format!("Failed to create model file: {}", e))?;

        let mut downloaded: u64 = 0;
        let mut stream = res.bytes_stream();
        let mut last_log_percent = 0;

        while let Some(item) = stream.next().await {
            let chunk = item.map_err(|e| format!("Download stream error: {}", e))?;
            file.write_all(&chunk)
                .map_err(|e| format!("Write error: {}", e))?;
            downloaded += chunk.len() as u64;

            if total_size > 0 {
                let percent = (downloaded as f64 / total_size as f64) * 100.0;
                if (percent as u64) >= last_log_percent + 10 {
                    info!("Downloading model: {:.0}%", percent);
                    last_log_percent = percent as u64;
                }
            }
        }

        info!("Model download complete.");
        Ok(model_path)
    }

    async fn start_server(&mut self) -> Result<(), String> {
        let models_dir = PortablePathManager::models_dir();
        
        // Determine executable name based on OS
        #[cfg(target_os = "windows")]
        let exe_name = "llama-server.exe";
        #[cfg(not(target_os = "windows"))]
        let exe_name = "llama-server";

        let exe_path = models_dir.join(exe_name);

        // Determine which model path to use
        let model_to_use = if self.model_path.exists() {
            self.model_path.clone()
        } else {
            info!("Model not found at {:?}, attempting to download default model...", self.model_path);
            self.ensure_model_exists().await?
        };

        if !exe_path.exists() {
            // In a real production scenario, we should download the appropriate binary
            // or instruct the user. For now, we'll just error out with a more helpful message.
            return Err(format!(
                "{} not found at {:?}. Please ensure the backend server binary is present for your platform.\n\
For instructions on obtaining the correct binary, please refer to the project documentation.",
                exe_name, exe_path
            ));
        }

        info!("Starting llama-server with model: {:?}", model_to_use);

        let child = Command::new(exe_path)
            .arg("-m")
            .arg(&model_to_use)
            .arg("-c")
            .arg("4096") // Context size
            .arg("--port")
            .arg("8080")
            .arg("--n-gpu-layers")
            .arg("99") // Try to use GPU if available
            .spawn()
            .map_err(|e| format!("Failed to spawn server: {}", e))?;

        self.server_process = Some(child);
        Ok(())
    }

    async fn wait_for_server_ready(&self) -> Result<(), String> {
        let start = std::time::Instant::now();
        let timeout = Duration::from_secs(60);

        while start.elapsed() < timeout {
            match self.client.get("http://localhost:8080/health").send().await {
                Ok(res) => {
                    if res.status().is_success() {
                        return Ok(());
                    }
                }
                Err(_) => {
                    // Server not ready yet, ignore error and retry
                }
            }
            tokio::time::sleep(Duration::from_millis(500)).await;
        }

        Err("Timeout waiting for llama-server to be ready".to_string())
    }

    async fn handle_message(&mut self, msg: LlmMessage) {
        match msg {
            LlmMessage::Generate { prompt, responder } => {
                let result = self.generate_completion(prompt, None, None).await;
                let _ = responder.send(result);
            }
            LlmMessage::GenerateWithParams {
                prompt,
                system_prompt,
                temperature,
                responder,
            } => {
                let result = self.generate_completion(prompt, system_prompt, temperature).await;
                let _ = responder.send(result);
            }
            LlmMessage::StreamGenerate {
                prompt,
                chunk_sender,
                responder,
            } => {
                let result = self.stream_completion(prompt, None, None, chunk_sender).await;
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

    async fn generate_completion(
        &self,
        prompt: String,
        system_prompt: Option<String>,
        temperature: Option<f32>,
    ) -> Result<String, ActorError> {
        info!("LLM Generating for prompt: {}", prompt);

        // Use provided system_prompt or default
        let system_msg = system_prompt.unwrap_or_else(|| "You are WhytChat, a helpful local assistant.".to_string());
        
        // Construct the prompt format for Qwen/ChatML
        // <|im_start|>system\n...<|im_end|>\n<|im_start|>user\n...<|im_end|>\n<|im_start|>assistant\n
        let formatted_prompt = format!(
            "<|im_start|>system\n{}\n<|im_end|>\n<|im_start|>user\n{}\n<|im_end|>\n<|im_start|>assistant\n",
            system_msg, prompt
        );

        // Build JSON payload with optional parameters
        let mut payload = serde_json::json!({
            "prompt": formatted_prompt,
            "n_predict": 512,
            "stop": ["<|im_end|>"]
        });
        
        if let Some(temp) = temperature {
            payload["temperature"] = serde_json::json!(temp);
        } else {
            payload["temperature"] = serde_json::json!(0.7);
        }

        let res = self
            .client
            .post("http://localhost:8080/completion")
            .json(&payload)
            .send()
            .await
            .map_err(|e| ActorError::LlmError(format!("Request failed: {}", e)))?;

        if !res.status().is_success() {
            return Err(ActorError::LlmError(format!(
                "Server returned status: {}",
                res.status()
            )));
        }

        let body: serde_json::Value = res
            .json()
            .await
            .map_err(|e| ActorError::LlmError(format!("Parse error: {}", e)))?;

        let content = body["content"]
            .as_str()
            .ok_or(ActorError::LlmError("No content in response".to_string()))?;

        Ok(content.to_string())
    }

    async fn stream_completion(
        &self,
        prompt: String,
        system_prompt: Option<String>,
        temperature: Option<f32>,
        chunk_sender: mpsc::Sender<Result<String, ActorError>>,
    ) -> Result<(), ActorError> {
        use futures::StreamExt;

        info!("LLM Streaming for prompt: {}", prompt);

        // Use provided system_prompt or default
        let system_msg = system_prompt.unwrap_or_else(|| "You are WhytChat, a helpful local assistant.".to_string());

        let formatted_prompt = format!(
            "<|im_start|>system\n{}\n<|im_end|>\n<|im_start|>user\n{}\n<|im_end|>\n<|im_start|>assistant\n",
            system_msg, prompt
        );

        // Build JSON payload with optional parameters
        let mut payload = serde_json::json!({
            "prompt": formatted_prompt,
            "n_predict": 512,
            "stop": ["<|im_end|>"],
            "stream": true
        });
        
        if let Some(temp) = temperature {
            payload["temperature"] = serde_json::json!(temp);
        } else {
            payload["temperature"] = serde_json::json!(0.7);
        }

        let res = self
            .client
            .post("http://localhost:8080/completion")
            .json(&payload)
            .send()
            .await
            .map_err(|e| ActorError::LlmError(format!("Request failed: {}", e)))?;

        if !res.status().is_success() {
            return Err(ActorError::LlmError(format!(
                "Server returned status: {}",
                res.status()
            )));
        }

        let mut stream = res.bytes_stream();

        while let Some(item) = stream.next().await {
            match item {
                Ok(bytes) => {
                    let chunk_str = String::from_utf8_lossy(&bytes);
                    // llama-server sends "data: {json}\n\n"
                    for line in chunk_str.lines() {
                        if line.starts_with("data: ") {
                            let json_str = &line[6..];
                            if let Ok(json) = serde_json::from_str::<serde_json::Value>(json_str) {
                                if let Some(content) = json["content"].as_str() {
                                    if !content.is_empty() {
                                        if let Err(_) =
                                            chunk_sender.send(Ok(content.to_string())).await
                                        {
                                            return Ok(()); // Receiver dropped
                                        }
                                    }
                                }
                                if let Some(stop) = json["stop"].as_bool() {
                                    if stop {
                                        return Ok(());
                                    }
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    let _ = chunk_sender
                        .send(Err(ActorError::LlmError(format!("Stream error: {}", e))))
                        .await;
                }
            }
        }

        Ok(())
    }
}
