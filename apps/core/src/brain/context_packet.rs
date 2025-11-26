//! Context Packet - Output structure for Brain analysis.
//!
//! Contains all the enriched context extracted from user input.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::complexity::ComplexityMetrics;
use super::intent::IntentResult;
use super::keywords::KeywordResult;

/// Detected language
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Language {
    French,
    English,
    Mixed,
    Unknown,
}

impl Language {
    /// Returns the language code
    #[allow(dead_code)]
    pub fn code(&self) -> &'static str {
        match self {
            Language::French => "fr",
            Language::English => "en",
            Language::Mixed => "mixed",
            Language::Unknown => "unknown",
        }
    }
}

/// Suggested strategy for handling the request
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Strategy {
    /// Search in RAG documents first
    UseRag,
    /// Answer directly without RAG
    DirectAnswer,
    /// Ask for clarification
    AskClarification,
    /// Generate code
    CodeGeneration,
    /// Creative writing mode (higher temperature)
    CreativeMode,
    /// Analysis/comparison mode
    AnalysisMode,
    /// Simple greeting response
    SimpleResponse,
    /// Translation task
    TranslationMode,
}

/// Result from RAG search
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RagResult {
    /// Content of the document chunk
    pub content: String,
    /// Relevance score (0.0 - 1.0)
    pub relevance_score: f32,
    /// Source file path (if available)
    pub source: Option<String>,
    /// Unique chunk identifier
    pub chunk_id: String,
}

/// Complete context packet from Brain analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextPacket {
    /// Original user query
    pub query: String,

    /// Detected intent with confidence
    pub intent: IntentResult,

    /// Extracted keywords with TF-IDF scores
    pub keywords: Vec<KeywordResult>,

    /// Text complexity metrics
    pub complexity: ComplexityMetrics,

    /// Detected language
    pub language: Language,

    /// RAG search results (if performed)
    pub rag_results: Vec<RagResult>,

    /// Suggested strategies for handling
    pub suggested_strategies: Vec<Strategy>,

    /// Whether RAG search is recommended
    pub should_use_rag: bool,

    /// Processing time in milliseconds
    pub processing_time_ms: u64,

    /// Timestamp of analysis
    pub timestamp: DateTime<Utc>,
}

impl ContextPacket {
    /// Create a new empty context packet
    pub fn new(query: String) -> Self {
        Self {
            query,
            intent: IntentResult {
                intent: super::intent::Intent::Unknown,
                confidence: 0.0,
                matched_patterns: vec![],
            },
            keywords: vec![],
            complexity: ComplexityMetrics {
                word_count: 0,
                avg_word_length: 0.0,
                sentence_count: 0,
                unique_words: 0,
                lexical_diversity: 0.0,
                technical_terms_count: 0,
                technical_terms: vec![],
                score: 0.0,
            },
            language: Language::Unknown,
            rag_results: vec![],
            suggested_strategies: vec![],
            should_use_rag: false,
            processing_time_ms: 0,
            timestamp: Utc::now(),
        }
    }

    /// Get the primary strategy
    #[allow(dead_code)]
    pub fn primary_strategy(&self) -> Option<&Strategy> {
        self.suggested_strategies.first()
    }

    /// Check if the query is complex
    #[allow(dead_code)]
    pub fn is_complex(&self) -> bool {
        self.complexity.score > 0.5
    }

    /// Check if the query is code-related
    #[allow(dead_code)]
    pub fn is_code_related(&self) -> bool {
        matches!(self.intent.intent, super::intent::Intent::CodeRequest)
            || self.suggested_strategies.contains(&Strategy::CodeGeneration)
    }

    /// Get a summary for logging
    #[allow(dead_code)]
    pub fn summary(&self) -> String {
        format!(
            "Intent: {:?} ({:.0}%), Keywords: {}, Complexity: {:.2}, Language: {:?}, RAG: {}",
            self.intent.intent,
            self.intent.confidence * 100.0,
            self.keywords.len(),
            self.complexity.score,
            self.language,
            if self.should_use_rag { "yes" } else { "no" }
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_packet_creation() {
        let packet = ContextPacket::new("test query".to_string());

        assert_eq!(packet.query, "test query");
        assert_eq!(packet.language, Language::Unknown);
        assert!(packet.keywords.is_empty());
        assert!(packet.rag_results.is_empty());
    }

    #[test]
    fn test_language_codes() {
        assert_eq!(Language::French.code(), "fr");
        assert_eq!(Language::English.code(), "en");
        assert_eq!(Language::Mixed.code(), "mixed");
        assert_eq!(Language::Unknown.code(), "unknown");
    }

    #[test]
    fn test_summary() {
        let packet = ContextPacket::new("test".to_string());
        let summary = packet.summary();

        assert!(summary.contains("Intent:"));
        assert!(summary.contains("Keywords:"));
        assert!(summary.contains("Complexity:"));
    }
}
