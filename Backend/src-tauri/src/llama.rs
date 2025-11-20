use serde::{Deserialize, Serialize};
use std::{
    net::{SocketAddr, TcpStream},
    path::PathBuf,
    time::Duration,
};
use tauri::{Emitter, State};

use crate::llama_install::{app_data_models_dir, find_llama_server};
use crate::process_manager::{start_llama_server, stop_llama_server, ProcessManagerState};

fn model_path(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    let dir = app_data_models_dir(app).map_err(|e| e.to_string())?;
    Ok(dir.join("qwen2.5-3b-instruct-q4_k_m.gguf"))
}

#[tauri::command]
pub async fn start_server(
    app: tauri::AppHandle,
    state: State<'_, ProcessManagerState>,
) -> Result<u16, String> {
    let model = model_path(&app)?;
    if !model.exists() {
        return Err("Model not found. Download it first.".into());
    }
    let Some(server_bin) = find_llama_server(&app) else {
        return Err("llama-server.exe not found. Install it first.".into());
    };

    // Utilisation de "default" comme ID pour le serveur principal pour compatibilité V1
    // Dans le futur, le frontend pourra passer un ID spécifique
    start_llama_server(&app, &state, "default".to_string(), server_bin, model)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn stop_server(
    app: tauri::AppHandle,
    state: State<'_, ProcessManagerState>,
) -> Result<(), String> {
    stop_llama_server(&app, &state, "default".to_string())
        .await
        .map_err(|e| e.to_string())
}

#[derive(Deserialize, Serialize, Clone)]
pub struct Message {
    pub role: String,
    pub content: String,
}

#[derive(Deserialize)]
pub struct ChatInput {
    pub messages: Vec<Message>,
}

#[derive(Serialize)]
pub struct ChatOutput {
    pub reply: String,
}

#[derive(Deserialize)]
pub struct AgentConfig {
    pub name: String,
    pub system_prompt: String,
}

#[derive(Deserialize)]
pub struct MultiChatInput {
    pub agents: Vec<AgentConfig>,
    pub history: Vec<Message>,
    pub synth_system_prompt: String,
}

#[derive(Deserialize)]
pub struct FusionChatInput {
    pub history_a: Vec<Message>,
    pub history_b: Vec<Message>,
    pub common_prompt: String,
}

#[derive(Deserialize)]
pub struct ContextualGenerationInput {
    pub context_history: Vec<Message>,
    pub user_prompt: String,
    pub agent_name: String, // Pour le log/event
}

async fn send_chat_request(port: u16, messages: Vec<Message>) -> Result<String, String> {
    let url = format!("http://127.0.0.1:{}/v1/chat/completions", port);
    let body = serde_json::json!({
        "model": "local",
        "messages": messages,
        "temperature": 0.7,
        "max_tokens": 4096,
        "top_p": 0.9,
        "top_k": 40,
        "min_p": 0.05,
        "repeat_penalty": 1.05,
        "presence_penalty": 0.0,
        "frequency_penalty": 0.0,
        "stop": ["<|im_end|>", "<|endoftext|>"]
    });
    let client = reqwest::Client::new();
    let resp = client
        .post(url)
        .json(&body)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !resp.status().is_success() {
        let err_text = resp.text().await.unwrap_or_default();
        println!("Chat error: {}", err_text);
        return Err(format!("Server error: {}", err_text));
    }

    let val: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
    let reply = val["choices"][0]["message"]["content"]
        .as_str()
        .unwrap_or("")
        .to_string();
    Ok(reply)
}

#[tauri::command]
pub async fn chat(
    state: State<'_, ProcessManagerState>,
    input: ChatInput,
) -> Result<ChatOutput, String> {
    let port = {
        let manager = state.0.lock().unwrap();
        manager.get_server_port("default").ok_or("Server not running")?
    };
    let reply = send_chat_request(port, input.messages).await?;
    Ok(ChatOutput { reply })
}

#[tauri::command]
pub async fn chat_multi(
    app: tauri::AppHandle,
    state: State<'_, ProcessManagerState>,
    input: MultiChatInput,
) -> Result<ChatOutput, String> {
    // Assurer que le serveur est prêt (auto-start si nécessaire)
    // Note: server_ready vérifie maintenant si le port est alloué ET accessible
    if !server_ready(state.clone()).await? {
        start_server(app.clone(), state.clone()).await?;
        // Le start_server attend déjà que le serveur soit prêt, donc pas besoin de boucle ici
    }
    
    let port = {
        let manager = state.0.lock().unwrap();
        manager.get_server_port("default").ok_or("Server not running")?
    };

    let mut reports = String::new();

    // 1. Phase Experts (Séquentiel pour V1)
    for agent in input.agents {
        let mut agent_messages = Vec::new();
        // Injecter le system prompt de l'agent
        agent_messages.push(Message {
            role: "system".into(),
            content: agent.system_prompt,
        });
        // Ajouter l'historique utilisateur
        for msg in &input.history {
            if msg.role != "system" {
                agent_messages.push(msg.clone());
            }
        }

        // Notify start of thinking
        let _ = app.emit("agent-thinking-start", serde_json::json!({
            "agent": agent.name
        }));

        let response = send_chat_request(port, agent_messages).await?;
        
        // Notify thought complete
        let _ = app.emit("agent-thought", serde_json::json!({
            "agent": agent.name,
            "content": response,
            "timestamp": chrono::Utc::now().to_rfc3339()
        }));

        reports.push_str(&format!(
            "\n--- Rapport Expert: {} ---\n{}\n",
            agent.name, response
        ));
    }

    // 2. Phase Synthèse
    let mut synth_messages = Vec::new();
    synth_messages.push(Message {
        role: "system".into(),
        content: input.synth_system_prompt,
    });

    // On remet l'historique pour le contexte
    for msg in &input.history {
        if msg.role != "system" {
            synth_messages.push(msg.clone());
        }
    }

    // On injecte les rapports comme un message "user" ou "system" supplémentaire avant la réponse finale
    // Pour forcer la prise en compte, on le met en dernier message User explicite
    synth_messages.push(Message {
        role: "user".into(),
        content: format!("Voici les analyses des experts consultés. Synthétise une réponse finale cohérente en prenant en compte ces points de vue :\n{}", reports)
    });

    let reply = send_chat_request(port, synth_messages).await?;
    Ok(ChatOutput { reply })
}

#[tauri::command]
pub async fn chat_fusion(
    app: tauri::AppHandle,
    state: State<'_, ProcessManagerState>,
    input: FusionChatInput,
) -> Result<ChatOutput, String> {
    // Ensure server is ready
    if !server_ready(state.clone()).await? {
        start_server(app.clone(), state.clone()).await?;
    }

    let port = {
        let manager = state.0.lock().unwrap();
        manager.get_server_port("default").ok_or("Server not running")?
    };

    // Notify Fusion Start
    let _ = app.emit("agent-thinking-start", serde_json::json!({
        "agent": "Fusion Context A"
    }));
    let _ = app.emit("agent-thinking-start", serde_json::json!({
        "agent": "Fusion Context B"
    }));

    // 1. Context A Response
    let mut messages_a = Vec::new();
    // Filter system messages to avoid conflict, but keep user/assistant flow
    for msg in &input.history_a {
        if msg.role != "system" {
            messages_a.push(msg.clone());
        }
    }
    messages_a.push(Message {
        role: "system".into(),
        content: "You are an expert discussing based on the previous conversation context. Answer the user's new prompt specifically from the perspective of this conversation history.".into()
    });
    messages_a.push(Message {
        role: "user".into(),
        content: input.common_prompt.clone(),
    });

    let response_a = send_chat_request(port, messages_a).await?;

    let _ = app.emit("agent-thought", serde_json::json!({
        "agent": "Fusion Context A",
        "content": response_a,
        "timestamp": chrono::Utc::now().to_rfc3339()
    }));

    // 2. Context B Response (Aware of A's response? Or parallel? User asked for "discute l'une et l'autre")
    // Let's make B aware of A for a debate/collaboration feel
    let mut messages_b = Vec::new();
    for msg in &input.history_b {
        if msg.role != "system" {
            messages_b.push(msg.clone());
        }
    }
    messages_b.push(Message {
        role: "system".into(),
        content: format!("You are an expert discussing based on your own conversation history. You will also see the perspective of another entity (Context A). Discuss, compare, or collaborate with it.").into()
    });
    messages_b.push(Message {
        role: "user".into(),
        content: format!("Question: {}\n\n[Perspective from Context A]:\n{}", input.common_prompt, response_a),
    });

    let response_b = send_chat_request(port, messages_b).await?;

    let _ = app.emit("agent-thought", serde_json::json!({
        "agent": "Fusion Context B",
        "content": response_b,
        "timestamp": chrono::Utc::now().to_rfc3339()
    }));

    // 3. Synthesis
    let synth_prompt = format!(
        "Synthétise les deux perspectives suivantes en réponse à la question : '{}'.\n\n--- Perspective A ---\n{}\n\n--- Perspective B ---\n{}",
        input.common_prompt, response_a, response_b
    );
    
    let synth_messages = vec![
        Message { role: "user".into(), content: synth_prompt }
    ];

    let final_reply = send_chat_request(port, synth_messages).await?;

    Ok(ChatOutput { reply: final_reply })
}

#[tauri::command]
pub async fn generate_response_with_context(
    app: tauri::AppHandle,
    state: State<'_, ProcessManagerState>,
    input: ContextualGenerationInput,
) -> Result<ChatOutput, String> {
    if !server_ready(state.clone()).await? {
        start_server(app.clone(), state.clone()).await?;
    }

    let port = {
        let manager = state.0.lock().unwrap();
        manager.get_server_port("default").ok_or("Server not running")?
    };

    // Construction du prompt contextuel
    let mut messages = Vec::new();
    
    // 1. Injecter l'historique personnel comme contexte (System/Few-shot)
    // On filtre pour ne garder que le contenu pertinent, et on le présente comme "Mémoire"
    let mut memory_context = String::new();
    for msg in &input.context_history {
        if msg.role != "system" {
             memory_context.push_str(&format!("{}: {}\n", msg.role.to_uppercase(), msg.content));
        }
    }

    messages.push(Message {
        role: "system".into(),
        content: format!("You are an expert AI agent participating in a meeting. \n\nYOUR BACKGROUND MEMORY:\n{}\n\nINSTRUCTION: Respond to the user's input based strictly on your background memory and perspective. Be concise.", memory_context)
    });

    // 2. Le prompt actuel (question de l'utilisateur ou tour de parole)
    messages.push(Message {
        role: "user".into(),
        content: input.user_prompt,
    });

    // Notify start
    let _ = app.emit("agent-thinking-start", serde_json::json!({
        "agent": input.agent_name
    }));

    let reply = send_chat_request(port, messages).await?;

    // Notify end
    let _ = app.emit("agent-thought", serde_json::json!({
        "agent": input.agent_name,
        "content": reply,
        "timestamp": chrono::Utc::now().to_rfc3339()
    }));

    Ok(ChatOutput { reply })
}

#[tauri::command]
pub async fn server_ready(state: State<'_, ProcessManagerState>) -> Result<bool, String> {
    let port_opt = {
        let manager = state.0.lock().unwrap();
        manager.get_server_port("default")
    };

    if let Some(port) = port_opt {
        let addr: SocketAddr = match format!("127.0.0.1:{}", port).parse() {
            Ok(a) => a,
            Err(e) => return Err(e.to_string()),
        };
        if TcpStream::connect_timeout(&addr, Duration::from_millis(200)).is_err() {
            return Ok(false);
        }
        let client = reqwest::Client::new();
        let base = format!("http://127.0.0.1:{}", port);
        let endpoints = ["/health", "/healthz", "/"]; // consider reachable if any responds
        for ep in endpoints {
            if let Ok(resp) = client.get(format!("{}{}", base, ep)).send().await {
                if resp.status().as_u16() < 500 {
                    return Ok(true);
                }
            }
        }
        Ok(false) // Port exists but no service
    } else {
        Ok(false) // No server running
    }
}
