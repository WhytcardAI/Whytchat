//! Semantic Intent Classification using FastEmbed embeddings.
//!
//! Uses the existing AllMiniLML6V2 model to classify intents
//! via cosine similarity with intent descriptions.
//! NO additional model download required!

use crate::brain::intent::Intent;
use fastembed::{EmbeddingModel, InitOptions, TextEmbedding};
use std::sync::Arc;
use tracing::{info, warn};

/// Intent description for semantic matching
struct IntentTemplate {
    intent: Intent,
    descriptions: &'static [&'static str],
}

/// Pre-computed intent templates with descriptions in FR and EN
const INTENT_TEMPLATES: &[IntentTemplate] = &[
    IntentTemplate {
        intent: Intent::Question,
        descriptions: &[
            "asking a question",
            "request for information",
            "inquiry about something",
            "poser une question",
            "demande d'information",
            "what is how does why when where who",
        ],
    },
    IntentTemplate {
        intent: Intent::Command,
        descriptions: &[
            "give an order or instruction",
            "command to do something",
            "imperative request",
            "donner un ordre",
            "commande impérative",
            "do this make that create execute run",
            "parle réponds dis montre fais",
        ],
    },
    IntentTemplate {
        intent: Intent::CodeRequest,
        descriptions: &[
            "programming code request",
            "write code function script",
            "debug fix code error",
            "demande de code programmation",
            "écrire du code fonction script",
            "javascript python rust typescript",
        ],
    },
    IntentTemplate {
        intent: Intent::Creative,
        descriptions: &[
            "creative writing request",
            "write story poem song lyrics",
            "imagine fiction narrative",
            "écriture créative",
            "écris histoire poème chanson",
            "imagine invente raconte",
        ],
    },
    IntentTemplate {
        intent: Intent::Greeting,
        descriptions: &[
            "greeting hello hi",
            "saying hello salutation",
            "bonjour salut coucou",
            "dire bonjour salutation",
        ],
    },
    IntentTemplate {
        intent: Intent::Farewell,
        descriptions: &[
            "goodbye bye farewell",
            "ending conversation",
            "au revoir à bientôt",
            "fin de conversation",
        ],
    },
    IntentTemplate {
        intent: Intent::Analysis,
        descriptions: &[
            "analyze compare summarize",
            "evaluation assessment review",
            "analyser comparer résumer",
            "évaluation examen synthèse",
        ],
    },
    IntentTemplate {
        intent: Intent::Translation,
        descriptions: &[
            "translate to another language",
            "translation request",
            "traduire dans une autre langue",
            "demande de traduction",
            "en français en anglais",
        ],
    },
    IntentTemplate {
        intent: Intent::Explanation,
        descriptions: &[
            "explain clarify elaborate",
            "explanation of concept",
            "expliquer clarifier détailler",
            "explication d'un concept",
        ],
    },
    IntentTemplate {
        intent: Intent::Help,
        descriptions: &[
            "help assistance support",
            "need help with something",
            "aide assistance support",
            "besoin d'aide",
        ],
    },
];

/// Semantic intent classifier using embeddings
pub struct SemanticIntentClassifier {
    model: Arc<TextEmbedding>,
    intent_embeddings: Vec<(Intent, Vec<f32>)>,
}

impl SemanticIntentClassifier {
    /// Create a new semantic classifier
    /// Uses the existing FastEmbed model from data/models/embeddings/
    pub fn new() -> Option<Self> {
        let embeddings_dir =
            crate::fs_manager::PortablePathManager::models_dir().join("embeddings");
        let mut options = InitOptions::new(EmbeddingModel::AllMiniLML6V2);
        options.show_download_progress = false;
        options.cache_dir = embeddings_dir;

        match TextEmbedding::try_new(options) {
            Ok(model) => {
                let model = Arc::new(model);
                let mut classifier = Self {
                    model: model.clone(),
                    intent_embeddings: Vec::new(),
                };

                // Pre-compute intent embeddings
                classifier.precompute_intent_embeddings();
                Some(classifier)
            }
            Err(e) => {
                warn!(
                    "Failed to load embedding model for intent classification: {}",
                    e
                );
                None
            }
        }
    }

    /// Pre-compute embeddings for all intent descriptions
    fn precompute_intent_embeddings(&mut self) {
        info!("Pre-computing intent embeddings...");

        for template in INTENT_TEMPLATES {
            // Combine all descriptions for this intent
            let combined_text = template.descriptions.join(" ");

            match self.model.embed(vec![combined_text], None) {
                Ok(embeddings) if !embeddings.is_empty() => {
                    self.intent_embeddings
                        .push((template.intent, embeddings[0].clone()));
                }
                Ok(_) => warn!("Empty embedding for intent {:?}", template.intent),
                Err(e) => warn!("Failed to embed intent {:?}: {}", template.intent, e),
            }
        }

        info!(
            "Pre-computed {} intent embeddings",
            self.intent_embeddings.len()
        );
    }

    /// Classify a query using semantic similarity
    pub fn classify(&self, query: &str) -> (Intent, f32) {
        // Embed the query
        let query_embedding = match self.model.embed(vec![query.to_string()], None) {
            Ok(embeddings) if !embeddings.is_empty() => embeddings[0].clone(),
            _ => return (Intent::Unknown, 0.0),
        };

        // Find the most similar intent
        let mut best_intent = Intent::Unknown;
        let mut best_similarity = -1.0f32;

        for (intent, intent_embedding) in &self.intent_embeddings {
            let similarity = cosine_similarity(&query_embedding, intent_embedding);
            if similarity > best_similarity {
                best_similarity = similarity;
                best_intent = *intent;
            }
        }

        // Threshold: if similarity is too low, return Unknown
        if best_similarity < 0.25 {
            return (Intent::Unknown, best_similarity);
        }

        (best_intent, best_similarity)
    }

    /// Classify with top-k results
    #[allow(dead_code)]
    pub fn classify_top_k(&self, query: &str, k: usize) -> Vec<(Intent, f32)> {
        let query_embedding = match self.model.embed(vec![query.to_string()], None) {
            Ok(embeddings) if !embeddings.is_empty() => embeddings[0].clone(),
            _ => return vec![(Intent::Unknown, 0.0)],
        };

        let mut results: Vec<(Intent, f32)> = self
            .intent_embeddings
            .iter()
            .map(|(intent, emb)| (*intent, cosine_similarity(&query_embedding, emb)))
            .collect();

        results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        results.truncate(k);
        results
    }
}

/// Calculate cosine similarity between two vectors
fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() || a.is_empty() {
        return 0.0;
    }

    let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let magnitude_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let magnitude_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

    if magnitude_a == 0.0 || magnitude_b == 0.0 {
        return 0.0;
    }

    dot_product / (magnitude_a * magnitude_b)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cosine_similarity() {
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![1.0, 0.0, 0.0];
        assert!((cosine_similarity(&a, &b) - 1.0).abs() < 0.001);

        let c = vec![0.0, 1.0, 0.0];
        assert!((cosine_similarity(&a, &c) - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_semantic_classifier_creation() {
        // This test requires the model to be downloaded
        // In CI, this might be skipped
        if let Some(classifier) = SemanticIntentClassifier::new() {
            assert!(!classifier.intent_embeddings.is_empty());
        }
    }

    #[test]
    fn test_semantic_classification() {
        if let Some(classifier) = SemanticIntentClassifier::new() {
            // Test French command
            let (intent, confidence) = classifier.classify("parle en français");
            println!("'parle en français' -> {:?} ({:.2})", intent, confidence);
            assert!(confidence > 0.2);

            // Test English question
            let (intent, _) = classifier.classify("how do I create a function?");
            println!("'how do I create a function?' -> {:?}", intent);

            // Test greeting
            let (intent, _) = classifier.classify("bonjour");
            println!("'bonjour' -> {:?}", intent);
            assert_eq!(intent, Intent::Greeting);

            // Test code request
            let (intent, _) = classifier.classify("écris moi une fonction python");
            println!("'écris moi une fonction python' -> {:?}", intent);
        }
    }
}
