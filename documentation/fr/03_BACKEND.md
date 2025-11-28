# ⚙️ Backend Rust - WhytChat V1

> Détail de tous les modules Rust dans `apps/core/src/`

---

## 📁 Structure des Fichiers

```
apps/core/src/
├── main.rs           # Point d'entrée, 22 commandes Tauri
├── database.rs       # CRUD SQLite (sessions, messages, folders, files)
├── encryption.rs     # AES-256-GCM encrypt/decrypt
├── error.rs          # AppError centralisé
├── fs_manager.rs     # PortablePathManager
├── models.rs         # Structs de données
├── rate_limiter.rs   # Limite 20 req/min
├── text_extract.rs   # Extraction texte (PDF, DOCX, TXT, CSV, JSON)
├── preflight.rs      # Vérifications au démarrage
├── diagnostics.rs    # Tests runtime catégorisés
├── actors/
│   ├── mod.rs        # Exports publics
│   ├── supervisor.rs # Orchestrateur principal
│   ├── llm.rs        # Communication llama-server
│   ├── rag.rs        # LanceDB + FastEmbed
│   ├── messages.rs   # Enums de messages
│   └── traits.rs     # Traits LlmActor, RagActor
└── brain/
    ├── mod.rs        # Exports publics
    ├── analyzer.rs   # BrainAnalyzer orchestrateur
    ├── intent.rs     # Classification regex
    ├── semantic_intent.rs # Classification embeddings
    ├── keywords.rs   # Extraction TF-IDF
    ├── complexity.rs # Score de complexité
    └── context_packet.rs # Struct de sortie
```

---

## 🗄️ database.rs - Couche Persistance

### Sessions CRUD

```rust
pub async fn save_session(pool: &SqlitePool, session: &Session) -> Result<(), AppError>
pub async fn get_session(pool: &SqlitePool, session_id: Uuid) -> Result<Option<Session>, AppError>
pub async fn get_all_sessions(pool: &SqlitePool) -> Result<Vec<Session>, AppError>
pub async fn delete_session(pool: &SqlitePool, session_id: Uuid) -> Result<(), AppError>
```

### Messages CRUD

```rust
pub async fn save_message(pool: &SqlitePool, message: &Message) -> Result<(), AppError>
pub async fn get_messages_for_session(pool: &SqlitePool, session_id: Uuid) -> Result<Vec<Message>, AppError>
pub async fn get_message_count(pool: &SqlitePool, session_id: Uuid) -> Result<i64, AppError>
pub async fn delete_message(pool: &SqlitePool, message_id: Uuid) -> Result<(), AppError>
```

### Folders (Dossiers)

```rust
pub async fn create_folder(pool: &SqlitePool, folder: &Folder) -> Result<(), AppError>
pub async fn get_all_folders(pool: &SqlitePool) -> Result<Vec<Folder>, AppError>
pub async fn get_folder(pool: &SqlitePool, folder_id: Uuid) -> Result<Option<Folder>, AppError>
pub async fn update_folder_name(pool: &SqlitePool, folder_id: Uuid, name: &str) -> Result<(), AppError>
pub async fn delete_folder(pool: &SqlitePool, folder_id: Uuid) -> Result<(), AppError>
```

### Library Files (Bibliothèque globale)

```rust
pub async fn save_library_file(pool: &SqlitePool, file: &LibraryFile) -> Result<(), AppError>
pub async fn get_library_files(pool: &SqlitePool, folder_id: Option<Uuid>) -> Result<Vec<LibraryFile>, AppError>
pub async fn delete_library_file(pool: &SqlitePool, file_id: Uuid) -> Result<(), AppError>
```

### Session Files (Fichiers liés à une session)

```rust
pub async fn save_session_file(pool: &SqlitePool, session_file: &SessionFile) -> Result<(), AppError>
pub async fn get_session_files(pool: &SqlitePool, session_id: Uuid) -> Result<Vec<SessionFile>, AppError>
pub async fn delete_session_file(pool: &SqlitePool, session_id: Uuid, file_id: Uuid) -> Result<(), AppError>
```

### Model Config (Chiffré)

```rust
pub async fn save_model_config(pool: &SqlitePool, config: &ModelConfig, encryption_key: &[u8]) -> Result<(), AppError>
pub async fn get_model_config(pool: &SqlitePool, encryption_key: &[u8]) -> Result<Option<ModelConfig>, AppError>
```

---

## 🔐 encryption.rs - Chiffrement AES-256-GCM

### Fonctions Principales

```rust
/// Chiffre des données avec AES-256-GCM
/// Nonce aléatoire généré par rand::thread_rng()
pub fn encrypt(data: &[u8], key: &[u8]) -> Result<Vec<u8>, AppError>

/// Déchiffre des données AES-256-GCM
/// Extrait le nonce des 12 premiers octets
pub fn decrypt(encrypted_data: &[u8], key: &[u8]) -> Result<Vec<u8>, AppError>

/// Génère une clé de chiffrement à partir d'un mot de passe
/// Utilise PBKDF2 avec sel fixe (TODO: sel dynamique)
pub fn derive_key(password: &str) -> [u8; 32]
```

### Sécurité Vérifiée

```rust
// Nonce ALÉATOIRE (PAS fixe comme indiqué dans l'ancienne doc)
let mut nonce_bytes = [0u8; 12];
rand::thread_rng().fill(&mut nonce_bytes); // ← Correct!
let nonce = Nonce::from_slice(&nonce_bytes);
```

---

## ❌ error.rs - Gestion des Erreurs

### AppError Enum

```rust
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("LLM error: {0}")]
    Llm(String),

    #[error("RAG error: {0}")]
    Rag(String),

    #[error("Encryption error: {0}")]
    Encryption(String),

    #[error("Actor timeout: {0}")]
    ActorTimeout(String),

    #[error("Configuration error: {0}")]
    Config(String),
}
```

### Conversion pour Tauri

Les commandes Tauri convertissent AppError en String :

```rust
#[tauri::command]
async fn my_command(state: State<'_, AppState>) -> Result<String, String> {
    some_operation().await.map_err(|e| e.to_string())
}
```

---

## 📂 fs_manager.rs - Gestion Fichiers Portables

### PortablePathManager

```rust
pub struct PortablePathManager {
    data_dir: PathBuf,  // data/
}

impl PortablePathManager {
    pub fn new() -> Result<Self, AppError>

    // Chemins de base
    pub fn data_dir(&self) -> &Path
    pub fn db_path(&self) -> PathBuf          // data/db/whytchat.sqlite
    pub fn vectors_path(&self) -> PathBuf     // data/vectors/knowledge_base.lance
    pub fn models_dir(&self) -> PathBuf       // data/models/
    pub fn embeddings_dir(&self) -> PathBuf   // data/models/embeddings/
    pub fn files_dir(&self) -> PathBuf        // data/files/

    // Gestion fichiers
    pub fn file_path(&self, file_id: Uuid, ext: &str) -> PathBuf
    pub async fn save_file(&self, file_id: Uuid, ext: &str, content: &[u8]) -> Result<PathBuf, AppError>
    pub async fn read_file(&self, file_id: Uuid, ext: &str) -> Result<Vec<u8>, AppError>
    pub async fn delete_file(&self, file_id: Uuid, ext: &str) -> Result<(), AppError>
}
```

---

## 📊 models.rs - Structures de Données

### Session

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: Uuid,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub is_pinned: bool,         // Épinglée en haut
    pub sort_order: Option<i64>, // Ordre personnalisé
}
```

### Message

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: Uuid,
    pub session_id: Uuid,
    pub role: String,       // "user" | "assistant"
    pub content: String,
    pub created_at: DateTime<Utc>,
    pub tokens: Option<i64>,
}
```

### Folder

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Folder {
    pub id: Uuid,
    pub name: String,
    pub created_at: DateTime<Utc>,
}
```

### LibraryFile (Bibliothèque Globale)

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LibraryFile {
    pub id: Uuid,
    pub folder_id: Option<Uuid>, // Null = racine
    pub name: String,
    pub file_type: String,       // "pdf", "txt", etc.
    pub size_bytes: i64,
    pub created_at: DateTime<Utc>,
    pub is_indexed: bool,        // Indexé dans RAG?
}
```

### SessionFile (Fichier lié à une session)

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionFile {
    pub session_id: Uuid,
    pub file_id: Uuid,
    pub added_at: DateTime<Utc>,
}
```

### ModelConfig (Chiffré en DB)

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelConfig {
    pub model_path: String,
    pub n_ctx: u32,           // Context length (4096 par défaut)
    pub n_gpu_layers: i32,    // Layers GPU (-1 = auto)
    pub temperature: f32,     // 0.7 par défaut
}
```

---

## ⏱️ rate_limiter.rs - Limitation de Débit

```rust
pub struct RateLimiter {
    requests: HashMap<Uuid, VecDeque<Instant>>,
    max_requests: usize,  // 20 par défaut
    window_secs: u64,     // 60 secondes
}

impl RateLimiter {
    pub fn new(max_requests: usize, window_secs: u64) -> Self

    /// Vérifie si une requête est autorisée
    /// Retourne true si OK, false si limite atteinte
    pub fn check(&mut self, session_id: Uuid) -> bool

    /// Nettoie les requêtes expirées
    pub fn cleanup(&mut self)
}
```

**Configuration :** 20 requêtes par minute par session.

---

## 📄 text_extract.rs - Extraction de Texte

### Formats Supportés

| Extension | Méthode                  |
| --------- | ------------------------ |
| `.txt`    | Lecture directe UTF-8    |
| `.md`     | Lecture directe UTF-8    |
| `.json`   | Sérialisation pretty     |
| `.csv`    | Parsing CSV + tabulation |
| `.pdf`    | `pdf-extract` crate      |
| `.docx`   | `docx-rs` crate          |

### Fonction Principale

```rust
pub fn extract_text(file_path: &Path) -> Result<String, AppError> {
    let ext = file_path.extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    match ext.as_str() {
        "txt" | "md" => fs::read_to_string(file_path)?,
        "json" => extract_json(file_path)?,
        "csv" => extract_csv(file_path)?,
        "pdf" => extract_pdf(file_path)?,
        "docx" => extract_docx(file_path)?,
        _ => return Err(AppError::Config(format!("Unsupported format: {}", ext))),
    }
}
```

---

## 🎭 actors/ - Système d'Acteurs

### mod.rs - Exports

```rust
pub mod llm;
pub mod messages;
pub mod rag;
pub mod supervisor;
pub mod traits;

pub use llm::*;
pub use messages::*;
pub use rag::*;
pub use supervisor::*;
pub use traits::*;
```

### messages.rs - Types de Messages

```rust
// Messages pour le Supervisor
pub enum SupervisorMessage {
    ProcessUserMessage {
        session_id: Uuid,
        content: String,
        window: tauri::Window,
        responder: oneshot::Sender<Result<String, String>>,
    },
    IngestContent {
        content: String,
        metadata: HashMap<String, String>,
        responder: oneshot::Sender<Result<(), String>>,
    },
    ReindexFile {
        file_id: Uuid,
        content: String,
        responder: oneshot::Sender<Result<(), String>>,
    },
    Shutdown,
}

// Messages pour le LLM Actor
pub enum LlmMessage {
    Generate {
        prompt: String,
        system_prompt: Option<String>,
        responder: oneshot::Sender<Result<String, String>>,
    },
    StreamGenerate {
        prompt: String,
        system_prompt: Option<String>,
        window: tauri::Window,
        responder: oneshot::Sender<Result<(), String>>,
    },
    Shutdown,
}

// Messages pour le RAG Actor
pub enum RagMessage {
    Ingest {
        content: String,
        metadata: HashMap<String, String>,
        responder: oneshot::Sender<Result<(), String>>,
    },
    Search {
        query: String,
        top_k: usize,
        responder: oneshot::Sender<Result<Vec<SearchResult>, String>>,
    },
    Delete {
        file_id: Uuid,
        responder: oneshot::Sender<Result<(), String>>,
    },
    Shutdown,
}

// Résultat de recherche RAG
pub struct SearchResult {
    pub content: String,
    pub score: f32,
    pub metadata: HashMap<String, String>,
}
```

### traits.rs - Traits Abstraits

```rust
#[async_trait]
pub trait LlmActor: Send + Sync {
    async fn generate(&self, prompt: &str, system: Option<&str>) -> Result<String, String>;
    async fn stream_generate(&self, prompt: &str, system: Option<&str>, window: &Window) -> Result<(), String>;
}

#[async_trait]
pub trait RagActor: Send + Sync {
    async fn ingest(&self, content: &str, metadata: HashMap<String, String>) -> Result<(), String>;
    async fn search(&self, query: &str, top_k: usize) -> Result<Vec<SearchResult>, String>;
    async fn delete(&self, file_id: Uuid) -> Result<(), String>;
}
```

---

## 🤖 actors/llm.rs - LLM Actor

### LlmActorHandle

```rust
pub struct LlmActorHandle {
    sender: mpsc::Sender<LlmMessage>,
}

impl LlmActorHandle {
    pub async fn generate(&self, prompt: &str, system: Option<&str>) -> Result<String, String>
    pub async fn stream_generate(&self, prompt: &str, system: Option<&str>, window: &Window) -> Result<(), String>
}
```

### LlmActorRunner (Interne)

```rust
struct LlmActorRunner {
    receiver: mpsc::Receiver<LlmMessage>,
    server_url: String,         // http://127.0.0.1:8765
    auth_token: String,         // Généré au runtime
    circuit_breaker: CircuitBreaker,
}
```

### Communication avec llama-server

```rust
// Format ChatML pour les prompts
async fn build_chat_request(&self, prompt: &str, system: Option<&str>) -> serde_json::Value {
    json!({
        "messages": [
            {"role": "system", "content": system.unwrap_or("You are a helpful assistant.")},
            {"role": "user", "content": prompt}
        ],
        "stream": true,
        "temperature": 0.7,
        "max_tokens": 4096
    })
}

// Streaming via SSE (Server-Sent Events)
async fn stream_response(&self, window: &Window) -> Result<(), String> {
    while let Some(chunk) = response.chunk().await? {
        let event = parse_sse_event(&chunk)?;
        if let Some(token) = event.get_token() {
            window.emit("chat-token", token)?;
        }
    }
}
```

### Circuit Breaker

```rust
struct CircuitBreaker {
    failures: AtomicU32,
    state: AtomicU8,       // 0=Closed, 1=Open, 2=HalfOpen
    last_failure: AtomicU64,
    threshold: u32,        // 5 échecs max
    reset_timeout: Duration, // 30 secondes
}
```

---

## 🔍 actors/rag.rs - RAG Actor

### RagActorHandle

```rust
pub struct RagActorHandle {
    sender: mpsc::Sender<RagMessage>,
}

impl RagActorHandle {
    pub async fn ingest(&self, content: &str, metadata: HashMap<String, String>) -> Result<(), String>
    pub async fn search(&self, query: &str, top_k: usize) -> Result<Vec<SearchResult>, String>
    pub async fn delete(&self, file_id: Uuid) -> Result<(), String>
}
```

### RagActorRunner (Interne)

```rust
struct RagActorRunner {
    receiver: mpsc::Receiver<RagMessage>,
    db: Arc<Mutex<Connection>>,         // LanceDB connection
    embedder: Arc<TextEmbedding>,       // FastEmbed
    table_name: String,                 // "knowledge_base"
}
```

### Embedding et Indexation

```rust
// Configuration FastEmbed
let model = EmbeddingModel::AllMiniLML6V2; // 384 dimensions
let embedder = TextEmbedding::try_new(InitOptions {
    model_name: model,
    cache_dir: embeddings_dir,
    ..Default::default()
})?;

// Chunking avec overlap
fn chunk_content(content: &str, chunk_size: usize, overlap: usize) -> Vec<String> {
    let words: Vec<&str> = content.split_whitespace().collect();
    let mut chunks = Vec::new();
    let mut start = 0;

    while start < words.len() {
        let end = (start + chunk_size).min(words.len());
        chunks.push(words[start..end].join(" "));
        start += chunk_size - overlap;
    }

    chunks
}

// Paramètres par défaut
const CHUNK_SIZE: usize = 512;   // mots
const CHUNK_OVERLAP: usize = 64; // mots
const TOP_K_DEFAULT: usize = 5;
```

### Recherche Vectorielle

```rust
async fn search(&self, query: &str, top_k: usize) -> Result<Vec<SearchResult>, String> {
    // 1. Embed la requête
    let query_embedding = self.embedder.embed(vec![query], None)?[0].clone();

    // 2. Recherche dans LanceDB
    let results = self.table
        .search(&query_embedding)
        .limit(top_k)
        .execute()
        .await?;

    // 3. Convertir en SearchResult
    results.iter().map(|r| SearchResult {
        content: r.get("content"),
        score: r.get("_distance"),
        metadata: r.get("metadata"),
    }).collect()
}
```

---

## 🎛️ actors/supervisor.rs - Orchestrateur

### SupervisorHandle

```rust
pub struct SupervisorHandle {
    sender: mpsc::Sender<SupervisorMessage>,
}

impl SupervisorHandle {
    /// Process un message utilisateur avec 3 paramètres
    pub async fn process_message(
        &self,
        session_id: Uuid,
        content: &str,
        window: tauri::Window,
    ) -> Result<String, String>

    /// Ingère du contenu dans le RAG
    pub async fn ingest_content(
        &self,
        content: &str,
        metadata: HashMap<String, String>,
    ) -> Result<(), String>

    /// Ré-indexe un fichier
    pub async fn reindex_file(
        &self,
        file_id: Uuid,
        content: &str,
    ) -> Result<(), String>
}
```

### SupervisorRunner (Interne)

```rust
struct SupervisorRunner<L, R>
where
    L: LlmActor + 'static,
    R: RagActor + 'static,
{
    receiver: mpsc::Receiver<SupervisorMessage>,
    llm_actor: Arc<L>,
    rag_actor: Arc<R>,
    brain_analyzer: Arc<BrainAnalyzer>,
    db_pool: Option<SqlitePool>,
}
```

### Flux de Traitement

```rust
async fn handle_process_message(
    &self,
    session_id: Uuid,
    content: String,
    window: Window,
) -> Result<String, String> {
    // 1. Analyse Brain
    window.emit("thinking-step", "Analyzing your message...")?;
    let context = self.brain_analyzer.analyze(&content);
    window.emit("brain-analysis", &context)?;

    // 2. RAG si nécessaire
    let rag_context = if context.should_use_rag {
        window.emit("thinking-step", "Searching knowledge base...")?;
        let results = self.rag_actor.search(&content, 5).await?;
        format_rag_context(&results)
    } else {
        String::new()
    };

    // 3. Construction du prompt
    let system_prompt = build_system_prompt(&context, &rag_context);

    // 4. Génération LLM en streaming
    window.emit("thinking-step", "Generating response...")?;
    self.llm_actor.stream_generate(&content, Some(&system_prompt), &window).await?;

    Ok("Stream completed".to_string())
}
```

---

## 🧠 brain/ - Module d'Analyse

Voir [04_BRAIN_MODULE.md](04_BRAIN_MODULE.md) pour la documentation détaillée.

---

_Généré depuis lecture directe de tous les fichiers dans apps/core/src/_
