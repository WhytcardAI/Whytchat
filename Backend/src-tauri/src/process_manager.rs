use std::{
    collections::HashMap,
    path::PathBuf,
    process::Stdio,
    sync::{Arc, Mutex},
    time::Duration,
};
use tauri::{AppHandle, Emitter};
use thiserror::Error;
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    process::{Child, Command},
    time::sleep,
};

#[derive(Error, Debug)]
pub enum ProcessError {
    #[error("Executable not found at path: {0}")]
    ExecutableNotFound(PathBuf),
    #[error("Model file not found at path: {0}")]
    ModelNotFound(PathBuf),
    #[error("Failed to spawn process: {0}")]
    SpawnError(String),
    #[error("Server failed to start within timeout")]
    StartupTimeout,
    #[error("Process with ID {0} not found")]
    ProcessNotFound(String),
}

pub struct RunningServer {
    pub child: Child,
    pub port: u16,
}

pub struct ProcessManager {
    servers: HashMap<String, RunningServer>,
    next_port: u16,
}

impl ProcessManager {
    pub fn new() -> Self {
        Self {
            servers: HashMap::new(),
            next_port: 8080,
        }
    }

    fn allocate_port(&mut self) -> u16 {
        let port = self.next_port;
        self.next_port += 1;
        port
    }

    pub fn get_server_port(&self, id: &str) -> Option<u16> {
        self.servers.get(id).map(|s| s.port)
    }
}

// Thread-safe wrapper for Tauri state
pub struct ProcessManagerState(pub Arc<Mutex<ProcessManager>>);

impl ProcessManagerState {
    pub fn new() -> Self {
        Self(Arc::new(Mutex::new(ProcessManager::new())))
    }
}

// Async implementation separated to handle Tauri AppHandle for events
pub async fn start_llama_server(
    app: &AppHandle,
    state: &ProcessManagerState,
    id: String,
    server_bin: PathBuf,
    model_path: PathBuf,
) -> Result<u16, ProcessError> {
    if !server_bin.exists() {
        return Err(ProcessError::ExecutableNotFound(server_bin));
    }
    if !model_path.exists() {
        return Err(ProcessError::ModelNotFound(model_path));
    }

    let port;
    {
        let mut manager = state.0.lock().map_err(|_| ProcessError::SpawnError("Mutex poisoned".into()))?;
        if manager.servers.contains_key(&id) {
            // If already running, return existing port
            return Ok(manager.servers.get(&id).unwrap().port);
        }
        port = manager.allocate_port();
    }

    // Notify UI: Starting
    let _ = app.emit("server-event", serde_json::json!({
        "type": "starting",
        "id": id,
        "port": port
    }));

    println!("Starting llama-server for {} on port {}", id, port);

    // Securely spawn process bound to localhost
    let mut child_command = Command::new(server_bin);
    child_command
        .arg("-m")
        .arg(&model_path)
        .arg("--host")
        .arg("127.0.0.1") // NETWORK CONFINEMENT: IMPERATIVE
        .arg("--port")
        .arg(port.to_string())
        .arg("-c")
        .arg("8192") // Increased context size to prevent KV cache errors
        .arg("-ngl")
        .arg("33") // GPU layers
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    // Windows-specific: Hide console window
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        child_command.creation_flags(CREATE_NO_WINDOW);
    }

    let child_result = child_command.spawn();

    match child_result {
        Ok(mut child) => {
            // Capture stdout
            if let Some(stdout) = child.stdout.take() {
                let id_clone = id.clone();
                tokio::spawn(async move {
                    let reader = BufReader::new(stdout);
                    let mut lines = reader.lines();
                    while let Ok(Some(line)) = lines.next_line().await {
                        println!("[llama-server:{}][STDOUT] {}", id_clone, line);
                    }
                });
            }

            // Capture stderr
            if let Some(stderr) = child.stderr.take() {
                let id_clone = id.clone();
                tokio::spawn(async move {
                    let reader = BufReader::new(stderr);
                    let mut lines = reader.lines();
                    while let Ok(Some(line)) = lines.next_line().await {
                        println!("[llama-server:{}][STDERR] {}", id_clone, line);
                    }
                });
            }

            {
                let mut manager = state.0.lock().map_err(|_| ProcessError::SpawnError("Mutex poisoned".into()))?;
                manager.servers.insert(
                    id.clone(),
                    RunningServer {
                        child,
                        port,
                    },
                );
            }

            // Wait for server to be ready (simple sleep for now, could be health check)
            // In a real robust system, we would read stdout for "HTTP server listening"
            // For now, we implement a simple health check loop
            let mut ready = false;
            for _ in 0..20 { // 10 seconds timeout
                sleep(Duration::from_millis(500)).await;
                if check_health(port).await {
                    ready = true;
                    break;
                }
            }

            if !ready {
                 // Cleanup if failed
                 let _ = stop_llama_server(app, state, id.clone()).await;
                 return Err(ProcessError::StartupTimeout);
            }

            // Notify UI: Ready
            let _ = app.emit("server-event", serde_json::json!({
                "type": "ready",
                "id": id,
                "port": port
            }));

            Ok(port)
        }
        Err(e) => Err(ProcessError::SpawnError(e.to_string())),
    }
}

pub async fn stop_llama_server(
    app: &AppHandle,
    state: &ProcessManagerState,
    id: String,
) -> Result<(), ProcessError> {
    let mut child_opt = None;

    {
        let mut manager = state.0.lock().map_err(|_| ProcessError::SpawnError("Mutex poisoned".into()))?;
        if let Some(server) = manager.servers.remove(&id) {
            child_opt = Some(server.child);
        }
    }

    if let Some(mut child) = child_opt {
        let _ = child.kill().await;
        println!("Stopped llama-server {}", id);
        
        // Notify UI: Stopped
        let _ = app.emit("server-event", serde_json::json!({
            "type": "stopped",
            "id": id
        }));
        
        Ok(())
    } else {
        Err(ProcessError::ProcessNotFound(id))
    }
}

async fn check_health(port: u16) -> bool {
    let client = reqwest::Client::new();
    let url = format!("http://127.0.0.1:{}/health", port);
    match client.get(&url).send().await {
        Ok(resp) => resp.status().is_success(),
        Err(_) => false,
    }
}