//! Text Complexity Scoring.
//!
//! Analyzes text complexity using various metrics inspired by readability formulas.
//! Provides a normalized score between 0.0 (simple) and 1.0 (complex).

use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Technical terms that indicate complexity
const TECHNICAL_TERMS: &[&str] = &[
    // Programming concepts
    "algorithm",
    "function",
    "variable",
    "parameter",
    "argument",
    "class",
    "object",
    "interface",
    "abstract",
    "polymorphism",
    "inheritance",
    "encapsulation",
    "recursion",
    "iteration",
    "asynchronous",
    "synchronous",
    "concurrent",
    "thread",
    "mutex",
    "semaphore",
    "deadlock",
    "race condition",
    // Data structures
    "array",
    "vector",
    "hashmap",
    "tree",
    "graph",
    "queue",
    "stack",
    "heap",
    "linked list",
    "binary tree",
    "btree",
    "trie",
    // Architecture
    "microservice",
    "monolith",
    "api",
    "rest",
    "graphql",
    "websocket",
    "database",
    "cache",
    "index",
    "query",
    "transaction",
    "schema",
    // DevOps
    "container",
    "kubernetes",
    "docker",
    "ci/cd",
    "pipeline",
    "deployment",
    "infrastructure",
    "orchestration",
    "monitoring",
    "logging",
    // AI/ML
    "neural network",
    "machine learning",
    "deep learning",
    "model",
    "training",
    "inference",
    "embedding",
    "tokenizer",
    "transformer",
    "llm",
    // French equivalents
    "algorithme",
    "fonction",
    "paramètre",
    "classe",
    "objet",
    "interface",
    "récursion",
    "itération",
    "asynchrone",
    "synchrone",
    "concurrent",
    "microservice",
    "base de données",
    "requête",
    "transaction",
    "conteneur",
    "déploiement",
    "infrastructure",
    "orchestration",
    "réseau neuronal",
    "apprentissage automatique",
    "modèle",
    "entraînement",
];

/// Metrics about text complexity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplexityMetrics {
    /// Total word count
    pub word_count: usize,
    /// Average word length in characters
    pub avg_word_length: f32,
    /// Number of sentences (approximated)
    pub sentence_count: usize,
    /// Number of unique words
    pub unique_words: usize,
    /// Lexical diversity (unique/total words)
    pub lexical_diversity: f32,
    /// Count of detected technical terms
    pub technical_terms_count: usize,
    /// List of detected technical terms
    pub technical_terms: Vec<String>,
    /// Normalized complexity score (0.0 - 1.0)
    pub score: f32,
}

/// Complexity scorer for text analysis
pub struct ComplexityScorer {
    technical_terms: HashSet<String>,
}

impl Default for ComplexityScorer {
    fn default() -> Self {
        Self::new()
    }
}

impl ComplexityScorer {
    /// Create a new complexity scorer
    pub fn new() -> Self {
        let technical_terms: HashSet<String> =
            TECHNICAL_TERMS.iter().map(|s| s.to_lowercase()).collect();

        Self { technical_terms }
    }

    /// Count sentences in text (approximation based on punctuation)
    fn count_sentences(&self, text: &str) -> usize {
        let count = text
            .chars()
            .filter(|c| *c == '.' || *c == '!' || *c == '?' || *c == '\n')
            .count();

        // At least 1 sentence if there's any text
        count.max(1)
    }

    /// Tokenize text into words
    fn tokenize(&self, text: &str) -> Vec<String> {
        text.to_lowercase()
            .split(|c: char| !c.is_alphanumeric() && c != '-' && c != '_')
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
            .collect()
    }

    /// Find technical terms in the text
    fn find_technical_terms(&self, words: &[String]) -> Vec<String> {
        let mut found = Vec::new();
        let text_lower = words.join(" ");

        for term in &self.technical_terms {
            if text_lower.contains(term) && !found.contains(term) {
                found.push(term.clone());
            }
        }

        // Also check individual words
        for word in words {
            if self.technical_terms.contains(word) && !found.contains(word) {
                found.push(word.clone());
            }
        }

        found
    }

    /// Calculate complexity score (0.0 - 1.0)
    fn calculate_score(&self, metrics: &ComplexityMetrics) -> f32 {
        // Weight factors for different metrics
        const WORD_COUNT_WEIGHT: f32 = 0.25;
        const AVG_LENGTH_WEIGHT: f32 = 0.20;
        const DIVERSITY_WEIGHT: f32 = 0.20;
        const TECHNICAL_WEIGHT: f32 = 0.35;

        // Normalize word count (100 words = max contribution)
        let word_score = (metrics.word_count as f32 / 100.0).min(1.0);

        // Normalize average word length (8 chars = max contribution)
        let length_score = (metrics.avg_word_length / 8.0).min(1.0);

        // Inverse lexical diversity (low diversity = high complexity)
        // High diversity means many unique words = potentially simpler individual concepts
        let diversity_score = 1.0 - metrics.lexical_diversity;

        // Normalize technical terms (10 terms = max contribution)
        let technical_score = (metrics.technical_terms_count as f32 / 10.0).min(1.0);

        // Weighted sum
        let score = word_score * WORD_COUNT_WEIGHT
            + length_score * AVG_LENGTH_WEIGHT
            + diversity_score * DIVERSITY_WEIGHT
            + technical_score * TECHNICAL_WEIGHT;

        // Clamp to [0, 1]
        score.clamp(0.0, 1.0)
    }

    /// Analyze text and return complexity metrics
    pub fn analyze(&self, text: &str) -> ComplexityMetrics {
        let words = self.tokenize(text);

        if words.is_empty() {
            return ComplexityMetrics {
                word_count: 0,
                avg_word_length: 0.0,
                sentence_count: 0,
                unique_words: 0,
                lexical_diversity: 0.0,
                technical_terms_count: 0,
                technical_terms: vec![],
                score: 0.0,
            };
        }

        let word_count = words.len();
        let sentence_count = self.count_sentences(text);

        // Calculate average word length
        let total_chars: usize = words.iter().map(|w| w.len()).sum();
        let avg_word_length = total_chars as f32 / word_count as f32;

        // Calculate unique words
        let unique: HashSet<&String> = words.iter().collect();
        let unique_words = unique.len();

        // Calculate lexical diversity
        let lexical_diversity = unique_words as f32 / word_count as f32;

        // Find technical terms
        let technical_terms = self.find_technical_terms(&words);
        let technical_terms_count = technical_terms.len();

        // Create metrics without score first
        let mut metrics = ComplexityMetrics {
            word_count,
            avg_word_length,
            sentence_count,
            unique_words,
            lexical_diversity,
            technical_terms_count,
            technical_terms,
            score: 0.0,
        };

        // Calculate final score
        metrics.score = self.calculate_score(&metrics);

        metrics
    }

    /// Get just the complexity score
    #[allow(dead_code)]
    pub fn score(&self, text: &str) -> f32 {
        self.analyze(text).score
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_text() {
        let scorer = ComplexityScorer::new();

        let metrics = scorer.analyze("Hello world");
        assert!(metrics.score < 0.3);
        assert_eq!(metrics.word_count, 2);
    }

    #[test]
    fn test_complex_text() {
        let scorer = ComplexityScorer::new();

        let text = "Implement an asynchronous microservice architecture using Kubernetes \
                    for container orchestration, with a PostgreSQL database and Redis cache. \
                    The system should handle concurrent requests using thread-safe mutex locks \
                    and implement proper transaction isolation for the database queries.";

        let metrics = scorer.analyze(text);
        assert!(metrics.score > 0.5);
        assert!(metrics.technical_terms_count > 5);
    }

    #[test]
    fn test_code_related() {
        let scorer = ComplexityScorer::new();

        let text = "Create a recursive function that implements the binary tree algorithm";

        let metrics = scorer.analyze(text);
        assert!(metrics.technical_terms_count >= 3);
    }

    #[test]
    fn test_french_technical() {
        let scorer = ComplexityScorer::new();

        let text = "Implémenter une fonction récursive avec une base de données et un algorithme";

        let metrics = scorer.analyze(text);
        assert!(metrics.technical_terms_count >= 2);
    }

    #[test]
    fn test_empty_text() {
        let scorer = ComplexityScorer::new();

        let metrics = scorer.analyze("");
        assert_eq!(metrics.score, 0.0);
        assert_eq!(metrics.word_count, 0);
    }

    #[test]
    fn test_lexical_diversity() {
        let scorer = ComplexityScorer::new();

        // High diversity (all unique words)
        let high_div = scorer.analyze("one two three four five six seven eight");
        assert!(high_div.lexical_diversity > 0.9);

        // Low diversity (repeated words)
        let low_div = scorer.analyze("the the the the same same same word word word");
        assert!(low_div.lexical_diversity < 0.5);
    }
}
