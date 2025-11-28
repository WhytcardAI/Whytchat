# 🔄 Flux de Données - WhytChat V1

> Parcours complet d'un message utilisateur du frontend à la réponse LLM

---

## 📊 Vue d'Ensemble

```
┌──────────────────────────────────────────────────────────────────────────┐
│                              FRONTEND                                     │
│                                                                          │
│  [User Input] → ChatInterface → invoke('debug_chat') → listen(events)    │
│                                                                          │
└──────────────────────────────────┬───────────────────────────────────────┘
                                   │ IPC
                                   ▼
┌──────────────────────────────────────────────────────────────────────────┐
│                              BACKEND                                      │
│                                                                          │
│  1. Rate Limiter → 2. Supervisor → 3. Brain → 4. RAG → 5. LLM → Events  │
│                                                                          │
└──────────────────────────────────────────────────────────────────────────┘
```

---

## 🎯 Étape par Étape

### 1️⃣ Frontend - Envoi du Message

```jsx
// ChatInterface.jsx
async function handleSend() {
  // 1. Validation
  if (!input.trim() || isStreaming) return;

  // 2. Ajouter message utilisateur au store
  addMessage({ role: "user", content: input });

  // 3. Préparer placeholder assistant
  setIsStreaming(true);
  addMessage({ role: "assistant", content: "" });

  // 4. Appeler Tauri
  await invoke("debug_chat", {
    sessionId: currentSessionId,
    content: input,
  });
}
```

---

### 2️⃣ Backend - Réception (main.rs)

```rust
#[tauri::command]
async fn debug_chat(
    state: State<'_, AppState>,
    window: Window,
    session_id: String,
    content: String,
) -> Result<String, String> {
    // 1. Vérifier initialisation
    if !state.is_initialized.load(Ordering::SeqCst) {
        return Err("Application not initialized".to_string());
    }

    // 2. Parser UUID
    let session_uuid = Uuid::parse_str(&session_id)
        .map_err(|e| format!("Invalid session ID: {}", e))?;

    // 3. Rate Limiting + Récupérer resources
    let (pool, supervisor) = check_rate_limit_and_get_resources(
        &state,
        session_uuid
    ).await?;

    // 4. Sauvegarder message utilisateur en DB
    let user_message = Message {
        id: Uuid::new_v4(),
        session_id: session_uuid,
        role: "user".to_string(),
        content: content.clone(),
        created_at: Utc::now(),
        tokens: None,
    };
    save_message(&pool, &user_message).await
        .map_err(|e| e.to_string())?;

    // 5. Déléguer au Supervisor
    supervisor.process_message(session_uuid, &content, window).await
}
```

---

### 3️⃣ Rate Limiter (rate_limiter.rs)

```rust
pub fn check(&mut self, session_id: Uuid) -> bool {
    let now = Instant::now();

    // Nettoyer les entrées expirées
    self.cleanup();

    // Récupérer ou créer l'historique de la session
    let requests = self.requests
        .entry(session_id)
        .or_insert_with(VecDeque::new);

    // Supprimer les requêtes hors fenêtre
    while let Some(front) = requests.front() {
        if now.duration_since(*front) > Duration::from_secs(self.window_secs) {
            requests.pop_front();
        } else {
            break;
        }
    }

    // Vérifier la limite
    if requests.len() >= self.max_requests {
        return false; // Limite atteinte
    }

    // Ajouter la requête
    requests.push_back(now);
    true
}
```

**Configuration :** 20 requêtes / 60 secondes / session

---

### 4️⃣ Supervisor - Orchestration (supervisor.rs)

```rust
async fn handle_process_message(
    &self,
    session_id: Uuid,
    content: String,
    window: Window,
) -> Result<String, String> {

    // ═══════════════════════════════════════════════════════
    // ÉTAPE 1 : Analyse Brain
    // ═══════════════════════════════════════════════════════
    window.emit("thinking-step", "🧠 Analyzing your message...")
        .map_err(|e| e.to_string())?;

    let context_packet = self.brain_analyzer.analyze(&content);

    // Émettre l'analyse complète
    window.emit("brain-analysis", &context_packet)
        .map_err(|e| e.to_string())?;

    info!(
        "Brain analysis: intent={:?}, keywords={:?}, complexity={:.2}, use_rag={}",
        context_packet.intent,
        context_packet.keywords,
        context_packet.complexity.overall,
        context_packet.should_use_rag
    );

    // ═══════════════════════════════════════════════════════
    // ÉTAPE 2 : Recherche RAG (si nécessaire)
    // ═══════════════════════════════════════════════════════
    let rag_context = if context_packet.should_use_rag {
        window.emit("thinking-step", "🔍 Searching knowledge base...")
            .map_err(|e| e.to_string())?;

        match self.rag_actor.search(&content, 5).await {
            Ok(results) if !results.is_empty() => {
                info!("Found {} RAG results", results.len());
                format_rag_context(&results)
            }
            Ok(_) => {
                info!("No RAG results found");
                String::new()
            }
            Err(e) => {
                warn!("RAG search failed: {}", e);
                String::new()
            }
        }
    } else {
        String::new()
    };

    // ═══════════════════════════════════════════════════════
    // ÉTAPE 3 : Construction du Prompt Système
    // ═══════════════════════════════════════════════════════
    let system_prompt = build_system_prompt(&context_packet, &rag_context);

    // ═══════════════════════════════════════════════════════
    // ÉTAPE 4 : Génération LLM en Streaming
    // ═══════════════════════════════════════════════════════
    window.emit("thinking-step", "✨ Generating response...")
        .map_err(|e| e.to_string())?;

    self.llm_actor
        .stream_generate(&content, Some(&system_prompt), &window)
        .await?;

    Ok("Stream completed".to_string())
}

fn format_rag_context(results: &[SearchResult]) -> String {
    if results.is_empty() {
        return String::new();
    }

    let mut context = String::from("\n\n### Relevant Context:\n");
    for (i, result) in results.iter().enumerate() {
        context.push_str(&format!(
            "\n[{}] (score: {:.2})\n{}\n",
            i + 1,
            result.score,
            result.content
        ));
    }
    context
}

fn build_system_prompt(packet: &ContextPacket, rag_context: &str) -> String {
    let mut prompt = String::from("You are a helpful AI assistant.");

    // Adapter selon l'intent
    match packet.intent {
        Intent::CodeRequest => {
            prompt.push_str(" When writing code, follow best practices and include comments.");
        }
        Intent::Explanation => {
            prompt.push_str(" Provide clear, detailed explanations with examples when helpful.");
        }
        Intent::Translation => {
            prompt.push_str(" Translate accurately while preserving the original meaning and tone.");
        }
        _ => {}
    }

    // Adapter selon la langue
    if packet.language == "fr" {
        prompt.push_str(" Respond in French unless asked otherwise.");
    }

    // Ajouter le contexte RAG
    if !rag_context.is_empty() {
        prompt.push_str(&format!(
            "\n\nUse the following context to inform your response:{}",
            rag_context
        ));
    }

    prompt
}
```

---

### 5️⃣ Brain Analyzer (brain/analyzer.rs)

```rust
pub fn analyze(&self, query: &str) -> ContextPacket {
    let mut packet = ContextPacket::default();

    // 1. Classification intent two-tier
    //    - Tier 1: Regex rapide
    //    - Tier 2: Semantic embeddings (fallback)
    packet.intent = self.classify_intent_smart(query);

    // 2. Extraction keywords TF-IDF
    packet.keywords = self.keyword_extractor.extract(query, Some(10));

    // 3. Score complexité
    packet.complexity = self.complexity_scorer.analyze(query);

    // 4. Détection langue (fr/en)
    packet.language = self.detect_language(query);

    // 5. Stratégies suggérées
    packet.suggested_strategies = self.suggest_strategies(&packet);

    // 6. Décision RAG
    packet.should_use_rag = self.should_use_rag(&packet);

    packet.analyzed_at = Utc::now();
    packet
}
```

**Sortie exemple :**

```json
{
  "intent": "CodeRequest",
  "keywords": ["rust", "async", "function", "error"],
  "complexity": { "overall": 0.65, "word_count": 12 },
  "language": "fr",
  "suggested_strategies": ["provide_code", "explain_concepts"],
  "should_use_rag": true
}
```

---

### 6️⃣ RAG Actor (actors/rag.rs)

```rust
async fn search(&self, query: &str, top_k: usize) -> Result<Vec<SearchResult>, String> {
    // 1. Générer l'embedding de la requête
    let query_embedding = self.embedder
        .embed(vec![query], None)
        .map_err(|e| format!("Embedding failed: {}", e))?
        .remove(0);

    // 2. Recherche vectorielle dans LanceDB
    let table = self.db.lock().await
        .open_table("knowledge_base")
        .await
        .map_err(|e| format!("Table open failed: {}", e))?;

    let results = table
        .search(&query_embedding)
        .limit(top_k)
        .execute()
        .await
        .map_err(|e| format!("Search failed: {}", e))?;

    // 3. Convertir en SearchResult
    Ok(results.iter().map(|r| SearchResult {
        content: r.get_string("content"),
        score: 1.0 - r.get_f32("_distance"), // distance → similarity
        metadata: r.get_map("metadata"),
    }).collect())
}
```

**Modèle d'embedding :** AllMiniLML6V2 (384 dimensions)

---

### 7️⃣ LLM Actor (actors/llm.rs)

```rust
async fn stream_generate(
    &self,
    prompt: &str,
    system: Option<&str>,
    window: &Window,
) -> Result<(), String> {
    // 1. Vérifier circuit breaker
    if self.circuit_breaker.is_open() {
        return Err("LLM service temporarily unavailable".to_string());
    }

    // 2. Construire la requête ChatML
    let request = json!({
        "messages": [
            {
                "role": "system",
                "content": system.unwrap_or("You are a helpful assistant.")
            },
            {
                "role": "user",
                "content": prompt
            }
        ],
        "stream": true,
        "temperature": 0.7,
        "max_tokens": 4096
    });

    // 3. Envoyer à llama-server
    let response = self.client
        .post(&format!("{}/v1/chat/completions", self.server_url))
        .header("Authorization", format!("Bearer {}", self.auth_token))
        .json(&request)
        .send()
        .await
        .map_err(|e| {
            self.circuit_breaker.record_failure();
            format!("Request failed: {}", e)
        })?;

    // 4. Traiter le stream SSE
    let mut stream = response.bytes_stream();
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| format!("Stream error: {}", e))?;

        // Parser les événements SSE
        for line in String::from_utf8_lossy(&chunk).lines() {
            if line.starts_with("data: ") {
                let data = &line[6..];
                if data == "[DONE]" {
                    break;
                }

                if let Ok(json) = serde_json::from_str::<Value>(data) {
                    if let Some(content) = json["choices"][0]["delta"]["content"].as_str() {
                        // 5. Émettre le token vers le frontend
                        window.emit("chat-token", content)
                            .map_err(|e| format!("Emit failed: {}", e))?;
                    }
                }
            }
        }
    }

    self.circuit_breaker.record_success();
    Ok(())
}
```

---

### 8️⃣ Frontend - Réception des Events

```javascript
// useChatStream.js
useEffect(() => {
  async function setupListeners() {
    // Listener tokens LLM
    const unlistenToken = await listen("chat-token", (event) => {
      onToken?.(event.payload); // Append au message assistant
    });

    // Listener étapes de réflexion
    const unlistenThinking = await listen("thinking-step", (event) => {
      onThinkingStep?.(event.payload); // Afficher dans ThinkingBubble
    });

    // Listener analyse Brain
    const unlistenBrain = await listen("brain-analysis", (event) => {
      console.log("Brain:", event.payload);
    });

    unlistenersRef.current = [unlistenToken, unlistenThinking, unlistenBrain];
  }

  setupListeners();
  return () => unlistenersRef.current.forEach((fn) => fn());
}, [onToken, onThinkingStep]);
```

```jsx
// ChatInterface.jsx - Mise à jour du message
function updateLastMessage(token) {
  setMessages((prev) => {
    const updated = [...prev];
    const last = updated[updated.length - 1];
    if (last && last.role === "assistant") {
      last.content += token;
    }
    return updated;
  });
}
```

---

## 📈 Diagramme de Séquence Complet

```
┌────────┐     ┌─────────┐    ┌──────────┐    ┌───────┐    ┌─────┐    ┌─────┐
│Frontend│     │ main.rs │    │Supervisor│    │ Brain │    │ RAG │    │ LLM │
└───┬────┘     └────┬────┘    └────┬─────┘    └───┬───┘    └──┬──┘    └──┬──┘
    │               │              │              │           │          │
    │ invoke()      │              │              │           │          │
    │──────────────>│              │              │           │          │
    │               │              │              │           │          │
    │               │ rate_limit() │              │           │          │
    │               │──────────────│              │           │          │
    │               │              │              │           │          │
    │               │ process_msg()│              │           │          │
    │               │─────────────>│              │           │          │
    │               │              │              │           │          │
    │ thinking-step │              │ analyze()    │           │          │
    │<──────────────│──────────────│─────────────>│           │          │
    │               │              │              │           │          │
    │ brain-analysis│              │<─────────────│           │          │
    │<──────────────│──────────────│              │           │          │
    │               │              │              │           │          │
    │ thinking-step │              │ search()     │           │          │
    │<──────────────│──────────────│──────────────│──────────>│          │
    │               │              │              │           │          │
    │               │              │<─────────────│───────────│          │
    │               │              │              │           │          │
    │ thinking-step │              │ stream()     │           │          │
    │<──────────────│──────────────│──────────────│───────────│─────────>│
    │               │              │              │           │          │
    │ chat-token    │              │              │           │          │
    │<──────────────│──────────────│──────────────│───────────│──────────│
    │ chat-token    │              │              │           │          │
    │<──────────────│──────────────│──────────────│───────────│──────────│
    │ ... (N tokens)│              │              │           │          │
    │               │              │              │           │          │
    │ Result<Ok>    │              │              │           │          │
    │<──────────────│              │              │           │          │
```

---

## ⏱️ Timeline Typique

| Étape                  | Durée Estimée  | Cumul    |
| ---------------------- | -------------- | -------- |
| Frontend → Backend IPC | ~1ms           | 1ms      |
| Rate Limiter check     | <1ms           | 1ms      |
| Brain Analysis         | ~5-20ms        | 20ms     |
| RAG Search (si actif)  | ~50-200ms      | 220ms    |
| LLM First Token        | ~200-500ms     | 720ms    |
| LLM Streaming (token)  | ~20-50ms/token | Variable |
| Total (sans RAG)       | ~300-600ms     | -        |
| Total (avec RAG)       | ~400-900ms     | -        |

---

## 🔄 Gestion des Erreurs

### Points de Défaillance

1. **Rate Limit** → Erreur immédiate, aucun traitement
2. **Brain Analysis** → Fallback vers intent "Unknown"
3. **RAG Search** → Continue sans contexte
4. **LLM Generation** → Circuit breaker + retry

### Circuit Breaker (LLM)

```rust
struct CircuitBreaker {
    failures: AtomicU32,      // Compteur d'échecs
    state: AtomicU8,          // 0=Closed, 1=Open, 2=HalfOpen
    threshold: u32,           // 5 échecs avant ouverture
    reset_timeout: Duration,  // 30s avant HalfOpen
}
```

**États :**

- **Closed** : Normal, requêtes passent
- **Open** : Bloque toutes les requêtes (30s)
- **HalfOpen** : Teste une requête, si OK → Closed

---

_Généré depuis lecture directe de: main.rs, supervisor.rs, analyzer.rs, rag.rs, llm.rs, useChatStream.js, ChatInterface.jsx_
