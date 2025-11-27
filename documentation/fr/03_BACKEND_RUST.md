# 🦀 Backend Rust - WhytChat V1

> Documentation détaillée de tous les modules `apps/core/src/`

---

## 📑 Table des Matières

1. [Fondations](#1-fondations)
2. [Actor System](#2-actor-system)
3. [Brain System](#3-brain-system)
4. [RAG System](#4-rag-system)
5. [LLM Actor](#5-llm-actor)
6. [Database](#6-database)
7. [Utilitaires](#7-utilitaires)

---

## 1. Fondations

### 1.1 models.rs (~130 lignes)

**But** : Définition des structures de données centrales.

```rust
/// Configuration du modèle LLM
pub struct ModelConfig {
    pub model_id: String,       // Default: "default-model.gguf"
    pub temperature: f32,       // Default: 0.7, Range: 0.0-2.0
    pub system_prompt: String,  // Max 2000 chars
}

/// Session de chat
pub struct Session {
    pub id: String,             // UUID v4
    pub title: String,
    pub model_config: ModelConfig, // CHIFFRÉ en DB
    pub folder_id: Option<String>,
    pub is_favorite: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Message dans une session
pub struct Message {
    pub id: String,
    pub session_id: String,
    pub role: String,           // "user" | "assistant" | "system"
    pub content: String,
    pub created_at: DateTime<Utc>,
}
```

### 1.2 error.rs (~100 lignes)

**But** : Gestion centralisée des erreurs avec `thiserror`.

```rust
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Actor error: {0}")]
    Actor(ActorError),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Rate limited")]
    RateLimited,
    // ... autres variantes
}
```

---

## 2. Actor System

### 2.1 supervisor.rs (~400 lignes)

**Responsabilités** :
- Recevoir les messages utilisateur via `SupervisorHandle`
- Orchestrer Brain → RAG → LLM
- Émettre les events Tauri vers le frontend
- Sauvegarder les messages en DB

**Pattern Actor** :

```rust
pub struct SupervisorHandle {
    sender: mpsc::Sender<SupervisorMessage>,
}

impl SupervisorHandle {
    pub async fn process_message(
        &self,
        session_id: String,
        message: String,
        window: Option<Window>,
    ) -> Result<String, AppError> {
        let (tx, rx) = oneshot::channel();
        self.sender.send(SupervisorMessage::ProcessUserMessage {
            session_id, message, window, response_tx: tx
        }).await?;
        rx.await?
    }
}
```

### 2.2 messages.rs (~80 lignes)

**Types de messages inter-acteurs** :

```rust
pub enum LlmMessage {
    Generate { prompt, system_prompt, temperature, response_tx },
    StreamGenerate { prompt, system_prompt, temperature, token_tx },
    HealthCheck { response_tx },
    Shutdown,
}

pub enum RagMessage {
    Ingest { content, metadata, response_tx },
    Search { query, top_k, response_tx },
    SearchWithFilters { query, file_ids, response_tx },
}
```

---

## 3. Brain System

### 3.1 Architecture Two-Tier

Le Brain utilise une architecture à deux niveaux pour maximiser performance et précision.

```
Query Input
    │
    ▼
┌────────────────────┐
│ Fast Path (Regex)  │ < 1ms
│ confidence > 0.5?  │
└────────────────────┘
    │ Non
    ▼
┌────────────────────┐
│ Semantic Fallback  │ ~10ms
│ (Embeddings)       │
└────────────────────┘
    │
    ▼
ContextPacket
```

### 3.2 Composants

| Module | Fonction | Sortie |
|--------|----------|--------|
| `intent.rs` | Classification intention | Intent enum + confidence |
| `keywords.rs` | Extraction TF-IDF | Vec<(String, f32)> |
| `complexity.rs` | Score de complexité | f32 (0.0-1.0) |
| `semantic_intent.rs` | Fallback embeddings | Intent enum |
| `context_packet.rs` | Structure de sortie | ContextPacket |

### 3.3 Intents Supportés (11)

```rust
pub enum Intent {
    Question,       // Question directe
    Command,        // Instruction à exécuter
    CodeRequest,    // Demande de code
    Creative,       // Génération créative
    Greeting,       // Salutation
    Farewell,       // Au revoir
    Analysis,       // Demande d'analyse
    Translation,    // Traduction
    Explanation,    // Demande d'explication
    Help,           // Demande d'aide
    Unknown,        // Non classifié
}
```

### 3.4 Output : ContextPacket

```rust
pub struct ContextPacket {
    pub intent: Intent,
    pub confidence: f32,
    pub keywords: Vec<(String, f32)>,
    pub complexity: f32,
    pub language: Language,         // FR | EN | Unknown
    pub should_use_rag: bool,
    pub suggested_strategies: Vec<String>,
}
```

---

## 4. RAG System

### 4.1 rag.rs (~350 lignes)

**Chunking Strategy** :

```rust
const TARGET_CHUNK_SIZE: usize = 512;    // chars
const CHUNK_OVERLAP: usize = 50;         // chars
const MIN_CHUNK_SIZE: usize = 20;        // chars
```

**Schema LanceDB** :

```rust
struct KnowledgeChunk {
    id: String,
    content: String,
    metadata: String,           // JSON serialized
    vector: FixedSizeList<384>, // FastEmbed dimension
}
```

**Pipeline d'Ingestion** :

```
Document
    │
    ▼ chunk_text()
Chunks (512 chars, 50 overlap)
    │
    ▼ FastEmbed
Embeddings (384-dim)
    │
    ▼ LanceDB
Stored + Indexed
```

### 4.2 Recherche Vectorielle

```rust
pub async fn search(
    &self,
    query: String,
    top_k: usize,
) -> Result<Vec<SearchResult>, AppError> {
    // 1. Génère embedding du query
    let query_vec = self.embed(&query).await?;
    
    // 2. Recherche cosine similarity
    let results = self.table
        .search(&query_vec)
        .limit(top_k)
        .execute()
        .await?;
    
    Ok(results)
}
```

---

## 5. LLM Actor

### 5.1 llm.rs (~600 lignes)

**Prompt Format** : ChatML

```
<|im_start|>system
{system_prompt}
<|im_end|>
<|im_start|>user
{user_message}
<|im_end|>
<|im_start|>assistant
```

**Circuit Breaker** :

| Paramètre | Valeur |
|-----------|--------|
| Max restarts | 3 en 60s |
| Auto-shutdown | 5 min inactivité |
| GPU layers | 99 (`-ngl 99`) |

**Timeouts** :

| Opération | Timeout |
|-----------|---------|
| Completion | 120s |
| Stream chunk | 30s |
| Health check | 5s |

### 5.2 Streaming

```rust
pub async fn stream_generate(
    &self,
    prompt: String,
    token_tx: mpsc::Sender<String>,
) -> Result<(), AppError> {
    // POST to /completion with stream: true
    let response = client
        .post("http://localhost:8080/completion")
        .json(&json!({
            "prompt": prompt,
            "stream": true,
            "temperature": 0.7,
        }))
        .send()
        .await?;
    
    // Parse SSE events
    let mut stream = response.bytes_stream();
    while let Some(chunk) = stream.next().await {
        // Parse "data: {...}" and extract token
        token_tx.send(token).await?;
    }
}
```

---

## 6. Database

### 6.1 database.rs (~480 lignes)

**Tables** :

| Table | Description |
|-------|-------------|
| `sessions` | Métadonnées + model_config chiffré |
| `messages` | Historique des messages |
| `folders` | Organisation |
| `library_files` | Fichiers uploadés |
| `session_files` | Liens session ↔ fichiers |

**Encryption Pattern** :

```rust
// En écriture
let encrypted = encrypt(&serde_json::to_vec(&model_config)?)?;
sqlx::query!("INSERT INTO sessions ... VALUES (..., ?)", encrypted)

// En lecture
let row = sqlx::query_as!(SessionRow, "SELECT ...")
let config = decrypt_model_config(&row.model_config)?;
```

### 6.2 encryption.rs (~155 lignes)

**Algorithme** : AES-256-GCM

**Gestion des Clés** :

1. Clé cachée en mémoire (`OnceLock`)
2. Variable d'environnement `ENCRYPTION_KEY`
3. Fichier `.encryption_key` dans data/
4. Génération nouvelle clé

**Format** : `Base64(nonce[12] + ciphertext)`

---

## 7. Utilitaires

### 7.1 fs_manager.rs (~140 lignes)

**PortablePathManager** : Gestion des chemins pour portabilité.

```rust
pub struct PortablePathManager {
    base_path: PathBuf,  // Résolu au runtime
}

impl PortablePathManager {
    pub fn get_db_path(&self) -> PathBuf
    pub fn get_models_path(&self) -> PathBuf
    pub fn get_vectors_path(&self) -> PathBuf
    pub fn get_files_path(&self) -> PathBuf
}
```

### 7.2 rate_limiter.rs (~80 lignes)

**Sliding Window Rate Limiting** :

```rust
pub struct RateLimiter {
    window_size: Duration,      // 60s
    max_requests: usize,        // 60 requests
    clients: HashMap<String, VecDeque<Instant>>,
}
```

### 7.3 text_extract.rs (~160 lignes)

**Formats Supportés** :

| Extension | Bibliothèque | Status |
|-----------|--------------|--------|
| `.pdf` | `pdf-extract` | ✅ |
| `.docx` | `docx-rs` | ✅ |
| `.txt` | Built-in | ✅ |
| `.md` | Built-in | ✅ |
| `.json` | Built-in | ✅ |
| `.doc` | - | ⚠️ Non supporté |

### 7.4 diagnostics.rs (~1000 lignes)

**28 Tests Diagnostics** :

| Catégorie | Tests | Description |
|-----------|-------|-------------|
| database | 7 | Connexion, CRUD, encryption |
| rag | 4 | Embeddings, ingestion, search |
| brain | 6 | Intent, keywords, complexity |
| llm | 6 | Startup, health, completion |
| filesystem | 4 | Directories, model, server |
| integration | 1 | Flow complet |

### 7.5 preflight.rs (~380 lignes)

**Vérifications Pré-Démarrage** :

1. Existence des dossiers data/
2. Présence du modèle GGUF
3. Permissions d'écriture
4. Validité de la configuration
5. Test llama-server (port 18080)

---

## 📚 Index des Fichiers

| # | Fichier | Lignes | Description |
|---|---------|--------|-------------|
| 1 | `models.rs` | ~130 | Structures de données |
| 2 | `error.rs` | ~100 | Gestion erreurs |
| 3 | `main.rs` | ~1500 | Entry point, 22 commandes IPC |
| 4 | `actors/supervisor.rs` | ~400 | Orchestrateur |
| 5 | `actors/messages.rs` | ~80 | Messages inter-acteurs |
| 6 | `actors/rag.rs` | ~350 | RAG system |
| 7 | `actors/llm.rs` | ~600 | LLM actor |
| 8 | `brain/analyzer.rs` | ~150 | Orchestrateur Brain |
| 9 | `brain/intent.rs` | ~200 | Classification intention |
| 10 | `brain/keywords.rs` | ~150 | Extraction TF-IDF |
| 11 | `brain/complexity.rs` | ~100 | Score complexité |
| 12 | `database.rs` | ~480 | CRUD SQLite |
| 13 | `encryption.rs` | ~155 | AES-256-GCM |
| 14 | `fs_manager.rs` | ~140 | Chemins portables |
| 15 | `rate_limiter.rs` | ~80 | Rate limiting |
| 16 | `text_extract.rs` | ~160 | Extraction texte |
| 17 | `diagnostics.rs` | ~1000 | Tests diagnostics |
| 18 | `preflight.rs` | ~380 | Vérifications |

---

## 📚 Voir Aussi

- [02_ARCHITECTURE.md](02_ARCHITECTURE.md) - Architecture globale
- [04_FRONTEND_REACT.md](04_FRONTEND_REACT.md) - Frontend React
- [05_FLUX_DONNEES.md](05_FLUX_DONNEES.md) - Flux complets

---

_Document généré le 27 novembre 2025_
