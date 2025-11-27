# 🔄 Flux de Données - WhytChat V1

> Documentation complète des flux de données dans l'application

---

## 📑 Table des Matières

1. [Flux Principal : Message Chat](#1-flux-principal--message-chat)
2. [Flux Secondaires](#2-flux-secondaires)
3. [Diagrammes de Séquence](#3-diagrammes-de-séquence)

---

## 1. Flux Principal : Message Chat

### Vue d'Ensemble

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                         FLUX D'UN MESSAGE CHAT                               │
└─────────────────────────────────────────────────────────────────────────────┘

USER INPUT (ChatInput.jsx)
    │
    ▼ onSend(text)
┌───────────────────────────────────────────────────────────────────────────┐
│ ChatInterface.jsx                                                          │
│   - Crée session si null (createSession)                                   │
│   - Appelle sendMessage(text, sessionId)                                   │
└───────────────────────────────────────────────────────────────────────────┘
    │
    ▼ useChatStream.sendMessage()
┌───────────────────────────────────────────────────────────────────────────┐
│ useChatStream.js                                                           │
│   1. Ajoute user msg localement (setMessages)                              │
│   2. setThinking(true), clearThinkingSteps()                               │
│   3. invoke('debug_chat', {session_id, message})                           │
│   4. Écoute événements 'chat-token' et 'thinking-step'                     │
└───────────────────────────────────────────────────────────────────────────┘
    │
    ▼ Tauri IPC
┌───────────────────────────────────────────────────────────────────────────┐
│ main.rs :: debug_chat                                                      │
│   1. check_rate_limit_and_get_resources()                                  │
│   2. supervisor.process_message(session_id, message, Some(window))         │
└───────────────────────────────────────────────────────────────────────────┘
    │
    ▼ SupervisorHandle.process_message()
┌───────────────────────────────────────────────────────────────────────────┐
│ supervisor.rs                                                              │
│   1. Sauvegarde user msg en DB (database::add_message)                     │
│   2. Brain Analysis (BrainAnalyzer::analyze)                               │
│      └─ Intent, Keywords, Complexity, Language, RAG decision               │
│   3. emit("thinking-step") + emit("brain-analysis")                        │
│   4. SI should_use_rag → RAG search (rag.search_with_filters)              │
│   5. Build ChatML prompt avec contexte                                     │
│   6. LLM streaming (llm.stream_generate_with_params)                       │
│      └─ Pour chaque token: emit("chat-token")                              │
│   7. Sauvegarde assistant msg en DB                                        │
└───────────────────────────────────────────────────────────────────────────┘
```

---

### Étape 1 : Input Utilisateur

**Fichier** : `ChatInput.jsx`

```jsx
const handleSubmit = () => {
  if (!text.trim()) return;
  onSend(text.trim());  // → ChatInterface.handleSend()
  setText('');
};
```

---

### Étape 2 : Préparation Frontend

**Fichier** : `useChatStream.js`

```javascript
const sendMessage = async (text, sessionId) => {
  // 1. Optimistic update
  setMessages((prev) => [...prev, {
    id: crypto.randomUUID(),
    role: 'user',
    content: text,
    created_at: new Date().toISOString(),
  }]);

  // 2. Setup thinking state
  setThinking(true);
  clearThinkingSteps();

  // 3. Setup response handlers
  let assistantContent = '';
  messageHandler = (payload) => {
    assistantContent += payload.content;
    updateAssistantMessage(assistantContent);
  };
  
  thinkingHandler = (payload) => {
    addThinkingStep(payload);
  };

  // 4. Invoke backend
  await invoke('debug_chat', {
    session_id: sessionId,
    message: text,
  });
};
```

---

### Étape 3 : Commande Tauri

**Fichier** : `main.rs`

```rust
#[tauri::command]
async fn debug_chat(
    session_id: String,
    message: String,
    window: Window,
    state: State<'_, AppState>,
) -> Result<String, String> {
    // 1. Rate limiting
    let (supervisor, _pool) = check_rate_limit_and_get_resources(&state)
        .await
        .map_err(|e| e.to_string())?;

    // 2. Process message
    supervisor
        .process_message(session_id, message, Some(window))
        .await
        .map_err(|e| e.to_string())
}
```

---

### Étape 4 : Orchestration Supervisor

**Fichier** : `supervisor.rs`

```rust
async fn handle_process_message(
    &mut self,
    session_id: String,
    message: String,
    window: Option<Window>,
) -> Result<String, AppError> {
    // 1. Save user message
    database::add_message(&self.pool, &session_id, "user", &message).await?;

    // 2. Brain analysis
    let context = self.brain.analyze(&message).await;
    emit_thinking(&window, "Analyzing query...", &format!("{:?}", context));
    emit_brain_analysis(&window, &context);

    // 3. RAG search (conditional)
    let rag_context = if context.should_use_rag {
        emit_thinking(&window, "Searching knowledge base...", "");
        let results = self.rag.search(&message, 5).await?;
        format_rag_results(&results)
    } else {
        String::new()
    };

    // 4. Build prompt
    let prompt = build_chatml_prompt(&message, &rag_context, &context);

    // 5. Stream LLM response
    emit_thinking(&window, "Generating response...", "");
    let response = self.stream_llm(&prompt, &window).await?;

    // 6. Save assistant message
    database::add_message(&self.pool, &session_id, "assistant", &response).await?;

    Ok(response)
}
```

---

### Étape 5 : Brain Analysis

**Fichier** : `brain/analyzer.rs`

```rust
pub async fn analyze(&self, query: &str) -> ContextPacket {
    // 1. Intent classification (fast path)
    let (intent, confidence) = self.intent_classifier.classify(query);
    
    // 2. Semantic fallback if low confidence
    let final_intent = if confidence < 0.5 {
        self.semantic_intent.classify(query).await
    } else {
        intent
    };

    // 3. Keyword extraction
    let keywords = self.keyword_extractor.extract(query);

    // 4. Complexity scoring
    let complexity = self.complexity_scorer.score(query);

    // 5. Language detection
    let language = detect_language(query);

    // 6. RAG decision
    let should_use_rag = self.should_use_rag(&final_intent, &keywords);

    ContextPacket {
        intent: final_intent,
        confidence,
        keywords,
        complexity,
        language,
        should_use_rag,
        suggested_strategies: vec![],
    }
}
```

---

### Étape 6 : RAG Search

**Fichier** : `actors/rag.rs`

```rust
pub async fn search(
    &self,
    query: &str,
    top_k: usize,
) -> Result<Vec<SearchResult>, AppError> {
    // 1. Generate query embedding
    let query_vec = self.embed(query).await?;

    // 2. Vector search
    let results = self.table
        .search(&query_vec)
        .limit(top_k)
        .execute()
        .await?;

    // 3. Convert to SearchResult
    let search_results = results
        .iter()
        .map(|r| SearchResult {
            content: r.content.clone(),
            metadata: serde_json::from_str(&r.metadata).ok(),
            score: r.score,
        })
        .collect();

    Ok(search_results)
}
```

---

### Étape 7 : LLM Streaming

**Fichier** : `actors/llm.rs`

```rust
pub async fn stream_generate(
    &self,
    prompt: &str,
    window: &Option<Window>,
) -> Result<String, AppError> {
    // 1. Ensure llama-server is running
    self.ensure_server_running().await?;

    // 2. HTTP request with streaming
    let response = self.client
        .post("http://localhost:8080/completion")
        .header("Authorization", format!("Bearer {}", self.auth_token))
        .json(&json!({
            "prompt": prompt,
            "stream": true,
            "temperature": 0.7,
            "max_tokens": 4096,
        }))
        .send()
        .await?;

    // 3. Parse SSE stream
    let mut full_response = String::new();
    let mut stream = response.bytes_stream();

    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        let text = String::from_utf8_lossy(&chunk);
        
        for line in text.lines() {
            if line.starts_with("data: ") {
                let data = &line[6..];
                if let Ok(json) = serde_json::from_str::<Value>(data) {
                    if let Some(content) = json["content"].as_str() {
                        full_response.push_str(content);
                        
                        // Emit token to frontend
                        if let Some(w) = window {
                            w.emit("chat-token", json!({ "content": content }))?;
                        }
                    }
                }
            }
        }
    }

    Ok(full_response)
}
```

---

## 2. Flux Secondaires

### 2.1 Création de Session

```
Frontend                          Backend
   │                                 │
   │ invoke('create_session', {      │
   │   title, language,              │
   │   system_prompt, temperature    │
   │ })                              │
   ├────────────────────────────────►│
   │                                 │ 1. Validate inputs
   │                                 │ 2. Generate UUID
   │                                 │ 3. Encrypt model_config
   │                                 │ 4. INSERT INTO sessions
   │◄────────────────────────────────┤
   │ session_id                      │
```

### 2.2 Upload de Fichier

```
Frontend                          Backend
   │                                 │
   │ invoke('upload_file_for_session', {
   │   session_id, file_path         │
   │ })                              │
   ├────────────────────────────────►│
   │                                 │ 1. Copy file to data/files/
   │                                 │ 2. Extract text (PDF/DOCX)
   │                                 │ 3. Generate embeddings
   │                                 │ 4. Store in LanceDB
   │                                 │ 5. INSERT INTO library_files
   │                                 │ 6. INSERT INTO session_files
   │◄────────────────────────────────┤
   │ file_id                         │
```

### 2.3 Recherche RAG Manuelle

```
Frontend                          Backend
   │                                 │
   │ invoke('search_knowledge', {    │
   │   query, top_k                  │
   │ })                              │
   ├────────────────────────────────►│
   │                                 │ 1. Generate query embedding
   │                                 │ 2. Vector search LanceDB
   │                                 │ 3. Format results
   │◄────────────────────────────────┤
   │ Vec<SearchResult>               │
```

---

## 3. Diagrammes de Séquence

### 3.1 Initialisation Application

```
┌─────────┐ ┌─────────┐ ┌──────────┐ ┌─────┐ ┌─────┐ ┌────────┐
│ App.jsx │ │main.rs  │ │preflight │ │ DB  │ │ RAG │ │  LLM   │
└────┬────┘ └────┬────┘ └────┬─────┘ └──┬──┘ └──┬──┘ └───┬────┘
     │           │            │          │       │        │
     │ mount     │            │          │       │        │
     ├──────────►│            │          │       │        │
     │           │            │          │       │        │
     │           │ preflight_check()     │       │        │
     │           ├───────────►│          │       │        │
     │           │            │ check_dirs        │        │
     │           │            ├─────────►│       │        │
     │           │            │ check_model       │        │
     │           │            │◄─────────┤       │        │
     │           │◄───────────┤          │       │        │
     │           │            │          │       │        │
     │           │ initialize_app()      │       │        │
     │           ├───────────────────────┼──────►│        │
     │           │                       │       │ init   │
     │           │                       │       ├───────►│
     │           │                       │       │◄───────┤
     │           │                       │◄──────┤        │
     │◄──────────┤                       │       │        │
     │ ready     │                       │       │        │
```

### 3.2 Message Chat Complet

```
┌──────────┐┌──────────┐┌─────────┐┌──────────┐┌─────┐┌─────┐┌─────┐
│ChatInput ││useChatSt.││ main.rs ││Supervisor││Brain││ RAG ││ LLM │
└────┬─────┘└────┬─────┘└────┬────┘└────┬─────┘└──┬──┘└──┬──┘└──┬──┘
     │           │           │          │         │      │      │
     │ submit    │           │          │         │      │      │
     ├──────────►│           │          │         │      │      │
     │           │           │          │         │      │      │
     │           │ invoke('debug_chat') │         │      │      │
     │           ├──────────►│          │         │      │      │
     │           │           │          │         │      │      │
     │           │           │ process_message()  │      │      │
     │           │           ├─────────►│         │      │      │
     │           │           │          │         │      │      │
     │           │           │          │ analyze()       │      │
     │           │           │          ├────────►│      │      │
     │           │           │          │◄────────┤      │      │
     │           │           │          │         │      │      │
     │           │ emit('thinking-step')│         │      │      │
     │           │◄──────────┼──────────┤         │      │      │
     │           │           │          │         │      │      │
     │           │           │          │ search()│      │      │
     │           │           │          ├─────────┼─────►│      │
     │           │           │          │◄────────┼──────┤      │
     │           │           │          │         │      │      │
     │           │           │          │ stream_generate()     │
     │           │           │          ├─────────┼──────┼─────►│
     │           │           │          │         │      │      │
     │           │ emit('chat-token') ◄─┼─────────┼──────┼──────┤
     │           │◄──────────┼──────────┤         │      │      │
     │           │           │          │         │      │      │
     │           │ [repeat per token]   │         │      │      │
     │           │           │          │         │      │      │
     │           │◄──────────┼──────────┤ done    │      │      │
     │◄──────────┤           │          │         │      │      │
     │ display   │           │          │         │      │      │
```

---

## 📊 Récapitulatif des Événements

| Événement | Direction | Payload | Quand |
|-----------|-----------|---------|-------|
| `chat-token` | Backend → Frontend | `{ content: string }` | Chaque token LLM |
| `thinking-step` | Backend → Frontend | `{ step: string, details: string }` | Chaque étape |
| `brain-analysis` | Backend → Frontend | `ContextPacket` | Après analyse Brain |

---

## 📚 Voir Aussi

- [02_ARCHITECTURE.md](02_ARCHITECTURE.md) - Architecture globale
- [03_BACKEND_RUST.md](03_BACKEND_RUST.md) - Backend Rust
- [04_FRONTEND_REACT.md](04_FRONTEND_REACT.md) - Frontend React

---

_Document généré le 27 novembre 2025_
