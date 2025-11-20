use anyhow::{Context, Result};
use fastembed::{EmbeddingModel, InitOptions, TextEmbedding};
use serde::{Deserialize, Serialize};
use std::{
    fs::{self, File},
    io::{BufReader, BufWriter},
    path::PathBuf,
    sync::{Arc, Mutex},
};
use tauri::{Manager, State};

// Structure de données pour un segment de texte indexé
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DocumentChunk {
    pub id: String,
    pub source: String,
    pub content: String,
    pub embedding: Vec<f32>,
}

// État global du RAG
pub struct RagState {
    pub model: Arc<Mutex<TextEmbedding>>,
    pub chunks: Arc<Mutex<Vec<DocumentChunk>>>,
    pub data_dir: PathBuf,
}

impl RagState {
    pub fn new(app_handle: &tauri::AppHandle) -> Result<Self> {
        let data_dir = app_handle
            .path()
            .app_data_dir()
            .context("Failed to get app data dir")?
            .join("rag_index");

        if !data_dir.exists() {
            fs::create_dir_all(&data_dir)?;
        }

        // Initialiser le modèle d'embedding (télécharge automatiquement all-MiniLM-L6-v2 ~80MB)
        let model = TextEmbedding::try_new(InitOptions::new(EmbeddingModel::AllMiniLML6V2).with_show_download_progress(true))?;

        let state = Self {
            model: Arc::new(Mutex::new(model)),
            chunks: Arc::new(Mutex::new(Vec::new())),
            data_dir,
        };

        state.load_index()?;
        Ok(state)
    }

    fn load_index(&self) -> Result<()> {
        let index_path = self.data_dir.join("index.json");
        if index_path.exists() {
            let file = File::open(index_path)?;
            let reader = BufReader::new(file);
            let chunks: Vec<DocumentChunk> = serde_json::from_reader(reader)?;
            *self.chunks.lock().unwrap() = chunks;
        }
        Ok(())
    }

    fn save_index(&self) -> Result<()> {
        let index_path = self.data_dir.join("index.json");
        let file = File::create(index_path)?;
        let writer = BufWriter::new(file);
        let chunks = self.chunks.lock().unwrap();
        serde_json::to_writer(writer, &*chunks)?;
        Ok(())
    }
}

#[tauri::command]
pub async fn ingest_file(
    state: State<'_, RagState>,
    path: String,
) -> Result<String, String> {
    let path_buf = PathBuf::from(&path);
    let filename = path_buf
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();

    // 1. Extraction du texte
    let content = if path.to_lowercase().ends_with(".pdf") {
        pdf_extract::extract_text(&path).map_err(|e| e.to_string())?
    } else {
        fs::read_to_string(&path).map_err(|e| e.to_string())?
    };

    // 2. Chunking (Découpage)
    let splitter = text_splitter::TextSplitter::new(text_splitter::ChunkConfig::new(500).with_trim(true));
    let chunks: Vec<&str> = splitter.chunks(&content).collect(); // ~500 caractères par chunk

    // 3. Embedding
    let model = state.model.lock().unwrap();
    let embeddings = model.embed(chunks.clone(), None).map_err(|e| e.to_string())?;

    // 4. Stockage
    let mut new_docs = Vec::new();
    for (i, chunk) in chunks.iter().enumerate() {
        new_docs.push(DocumentChunk {
            id: format!("{}::{}", filename, i),
            source: filename.clone(),
            content: chunk.to_string(),
            embedding: embeddings[i].clone(),
        });
    }

    {
        let mut store = state.chunks.lock().unwrap();
        store.extend(new_docs);
    }

    state.save_index().map_err(|e| e.to_string())?;

    Ok(format!("Indexed {} chunks from {}", chunks.len(), filename))
}

#[tauri::command]
pub async fn query_rag(
    state: State<'_, RagState>,
    query: String,
    limit: usize,
) -> Result<Vec<DocumentChunk>, String> {
    let model = state.model.lock().unwrap();
    let query_embedding = model
        .embed(vec![&query], None)
        .map_err(|e| e.to_string())?
        .pop()
        .ok_or("Failed to embed query")?;

    let chunks = state.chunks.lock().unwrap();
    
    // Recherche Vectorielle (Cosine Similarity)
    let mut scored_chunks: Vec<(f32, &DocumentChunk)> = chunks
        .iter()
        .map(|chunk| {
            let score = cosine_similarity(&query_embedding, &chunk.embedding);
            (score, chunk)
        })
        .collect();

    // Tri par score décroissant
    scored_chunks.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));

    // Retourne les meilleurs résultats sans l'embedding (trop lourd)
    Ok(scored_chunks
        .into_iter()
        .take(limit)
        .map(|(_score, chunk)| DocumentChunk {
            id: chunk.id.clone(),
            source: chunk.source.clone(),
            content: chunk.content.clone(),
            embedding: vec![], // On vide l'embedding pour le retour frontend pour économiser la bande passante
        })
        .collect())
}

fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    let dot_product: f32 = a.iter().zip(b).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    
    if norm_a == 0.0 || norm_b == 0.0 {
        return 0.0;
    }
    
    dot_product / (norm_a * norm_b)
}