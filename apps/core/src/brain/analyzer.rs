//! Brain Analyzer - Main orchestrator for the Brain module.
//!
//! Coordinates intent classification, keyword extraction, complexity analysis,
//! and strategy suggestion.
//!
//! Uses a two-tier intent classification:
//! 1. Fast regex patterns (< 1ms)
//! 2. Semantic embeddings fallback if regex confidence is low (< 10ms)

use chrono::Utc;
use std::sync::OnceLock;
use std::time::Instant;
use tracing::info;

use super::complexity::ComplexityScorer;
use super::context_packet::{ContextPacket, Language, RagResult, Strategy};
use super::intent::{Intent, IntentClassifier, IntentResult};
use super::keywords::KeywordExtractor;
use super::semantic_intent::SemanticIntentClassifier;

/// Lazy-initialized semantic classifier (expensive to create)
static SEMANTIC_CLASSIFIER: OnceLock<Option<SemanticIntentClassifier>> = OnceLock::new();

/// Get or initialize the semantic classifier
fn get_semantic_classifier() -> Option<&'static SemanticIntentClassifier> {
    SEMANTIC_CLASSIFIER
        .get_or_init(|| {
            info!("Initializing semantic intent classifier...");
            SemanticIntentClassifier::new()
        })
        .as_ref()
}

/// Main Brain analyzer that orchestrates all analysis components
pub struct BrainAnalyzer {
    intent_classifier: IntentClassifier,
    keyword_extractor: KeywordExtractor,
    complexity_scorer: ComplexityScorer,
}

impl Default for BrainAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl BrainAnalyzer {
    /// Create a new Brain analyzer with default settings
    pub fn new() -> Self {
        Self {
            intent_classifier: IntentClassifier::new(),
            keyword_extractor: KeywordExtractor::new(),
            complexity_scorer: ComplexityScorer::new(),
        }
    }

    /// Detect language based on character patterns and common words
    fn detect_language(&self, text: &str) -> Language {
        let text_lower = text.to_lowercase();

        // French indicators
        let french_chars = text.chars().filter(|c| {
            matches!(*c, 'é' | 'è' | 'ê' | 'ë' | 'à' | 'â' | 'ù' | 'û' | 'ô' | 'î' | 'ï' | 'ç' | 'œ')
        }).count();

        let french_words = [
            "le", "la", "les", "un", "une", "des", "du", "de", "et", "ou", "mais",
            "je", "tu", "il", "elle", "nous", "vous", "ils", "elles",
            "est", "sont", "être", "avoir", "faire", "pour", "dans", "sur", "avec",
            "comment", "pourquoi", "quand", "qui", "que", "quoi",
            "bonjour", "salut", "merci", "s'il",
        ];

        let english_words = [
            "the", "a", "an", "is", "are", "was", "were", "be", "been",
            "have", "has", "had", "do", "does", "did",
            "i", "you", "he", "she", "it", "we", "they",
            "and", "or", "but", "for", "with", "from", "to", "in", "on",
            "what", "why", "how", "when", "where", "who",
            "hello", "hi", "please", "thank",
        ];

        let french_word_count = french_words
            .iter()
            .filter(|w| {
                text_lower.split_whitespace().any(|word| {
                    word.trim_matches(|c: char| !c.is_alphanumeric()) == **w
                })
            })
            .count();

        let english_word_count = english_words
            .iter()
            .filter(|w| {
                text_lower.split_whitespace().any(|word| {
                    word.trim_matches(|c: char| !c.is_alphanumeric()) == **w
                })
            })
            .count();

        // Consider French accented characters
        let french_score = french_word_count + french_chars;
        let english_score = english_word_count;

        if french_score > 0 && english_score > 0 {
            if french_score > english_score * 2 {
                Language::French
            } else if english_score > french_score * 2 {
                Language::English
            } else {
                Language::Mixed
            }
        } else if french_score > 0 {
            Language::French
        } else if english_score > 0 {
            Language::English
        } else {
            Language::Unknown
        }
    }

    /// Suggest strategies based on intent and complexity
    fn suggest_strategies(&self, packet: &ContextPacket) -> Vec<Strategy> {
        let mut strategies = Vec::new();

        match packet.intent.intent {
            Intent::Greeting | Intent::Farewell => {
                strategies.push(Strategy::SimpleResponse);
            }
            Intent::CodeRequest => {
                strategies.push(Strategy::CodeGeneration);
                if packet.complexity.score > 0.3 {
                    strategies.push(Strategy::UseRag);
                }
            }
            Intent::Creative => {
                strategies.push(Strategy::CreativeMode);
            }
            Intent::Analysis => {
                strategies.push(Strategy::AnalysisMode);
                strategies.push(Strategy::UseRag);
            }
            Intent::Translation => {
                strategies.push(Strategy::TranslationMode);
            }
            Intent::Question | Intent::Explanation | Intent::Help => {
                // Questions often benefit from RAG
                if packet.complexity.score > 0.2 || packet.complexity.technical_terms_count > 0 {
                    strategies.push(Strategy::UseRag);
                }
                strategies.push(Strategy::DirectAnswer);
            }
            Intent::Command => {
                strategies.push(Strategy::DirectAnswer);
                if packet.complexity.technical_terms_count > 2 {
                    strategies.push(Strategy::UseRag);
                }
            }
            Intent::Unknown => {
                // Default to RAG for unknown intents if complex
                if packet.complexity.score > 0.3 {
                    strategies.push(Strategy::UseRag);
                }
                strategies.push(Strategy::DirectAnswer);
            }
        }

        strategies
    }

    /// Determine if RAG should be used
    fn should_use_rag(&self, packet: &ContextPacket) -> bool {
        // Use RAG if:
        // 1. Complexity is high
        // 2. Technical terms are present
        // 3. Intent suggests information lookup
        // 4. Strategy includes UseRag

        packet.suggested_strategies.contains(&Strategy::UseRag)
            || packet.complexity.score > 0.4
            || packet.complexity.technical_terms_count >= 2
            || matches!(
                packet.intent.intent,
                Intent::Question | Intent::Analysis | Intent::Explanation
            )
    }

    /// Analyze a query and produce a context packet
    pub fn analyze(&self, query: &str) -> ContextPacket {
        let start = Instant::now();

        // Create base packet
        let mut packet = ContextPacket::new(query.to_string());

        // 1. Classify intent (two-tier system)
        packet.intent = self.classify_intent_smart(query);

        // 2. Extract keywords
        packet.keywords = self.keyword_extractor.extract(query, Some(10));

        // 3. Analyze complexity
        packet.complexity = self.complexity_scorer.analyze(query);

        // 4. Detect language
        packet.language = self.detect_language(query);

        // 5. Suggest strategies (needs intent and complexity first)
        packet.suggested_strategies = self.suggest_strategies(&packet);

        // 6. Determine RAG usage
        packet.should_use_rag = self.should_use_rag(&packet);

        // 7. Set timing
        packet.processing_time_ms = start.elapsed().as_millis() as u64;
        packet.timestamp = Utc::now();

        packet
    }

    /// Smart intent classification with fallback to semantic
    fn classify_intent_smart(&self, query: &str) -> IntentResult {
        // Step 1: Try fast regex patterns
        let regex_result = self.intent_classifier.classify(query);

        // If regex found a clear intent with good confidence, use it
        if regex_result.intent != Intent::Unknown && regex_result.confidence >= 0.5 {
            return regex_result;
        }

        // Step 2: Fallback to semantic classification
        if let Some(semantic) = get_semantic_classifier() {
            let (semantic_intent, semantic_confidence) = semantic.classify(query);

            // If semantic has better confidence, use it
            if semantic_confidence > regex_result.confidence && semantic_intent != Intent::Unknown {
                info!(
                    "Using semantic intent: {:?} ({:.2}) over regex: {:?} ({:.2})",
                    semantic_intent, semantic_confidence,
                    regex_result.intent, regex_result.confidence
                );
                return IntentResult {
                    intent: semantic_intent,
                    confidence: semantic_confidence,
                    matched_patterns: vec!["semantic_embedding".to_string()],
                };
            }
        }

        // Return regex result (even if Unknown)
        regex_result
    }

    /// Analyze with RAG results (for full context)
    #[allow(dead_code)]
    pub fn analyze_with_rag(&self, query: &str, rag_results: Vec<RagResult>) -> ContextPacket {
        let mut packet = self.analyze(query);
        packet.rag_results = rag_results;
        packet
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_analysis() {
        let brain = BrainAnalyzer::new();

        let packet = brain.analyze("Hello, how are you?");

        assert_eq!(packet.query, "Hello, how are you?");
        // First call may load semantic model (~500ms), subsequent calls are fast
        // Just verify it completes reasonably
        assert!(packet.processing_time_ms < 5000);
    }

    #[test]
    fn test_greeting_detection() {
        let brain = BrainAnalyzer::new();

        let packet = brain.analyze("Bonjour!");
        assert_eq!(packet.intent.intent, Intent::Greeting);
        assert!(packet.suggested_strategies.contains(&Strategy::SimpleResponse));
        assert!(!packet.should_use_rag);
    }

    #[test]
    fn test_code_request() {
        let brain = BrainAnalyzer::new();

        let packet = brain.analyze("Write a Python function to sort an array");
        assert_eq!(packet.intent.intent, Intent::CodeRequest);
        assert!(packet.suggested_strategies.contains(&Strategy::CodeGeneration));
    }

    #[test]
    fn test_french_detection() {
        let brain = BrainAnalyzer::new();

        let packet = brain.analyze("Comment créer une fonction en Rust?");
        assert_eq!(packet.language, Language::French);
    }

    #[test]
    fn test_english_detection() {
        let brain = BrainAnalyzer::new();

        let packet = brain.analyze("How do I create a function in Rust?");
        assert_eq!(packet.language, Language::English);
    }

    #[test]
    fn test_complex_query_uses_rag() {
        let brain = BrainAnalyzer::new();

        let packet = brain.analyze(
            "Explain the difference between microservices and monolith architecture \
             in terms of database transactions and container orchestration"
        );

        assert!(packet.should_use_rag);
        assert!(packet.complexity.score > 0.4);
    }

    #[test]
    fn test_keyword_extraction() {
        let brain = BrainAnalyzer::new();

        let packet = brain.analyze("Create a Rust function for database queries");

        assert!(!packet.keywords.is_empty());
        let keyword_strings: Vec<&str> = packet.keywords.iter().map(|k| k.keyword.as_str()).collect();
        assert!(keyword_strings.contains(&"rust") || keyword_strings.contains(&"function") || keyword_strings.contains(&"database"));
    }

    #[test]
    fn test_analysis_with_rag() {
        let brain = BrainAnalyzer::new();

        let rag_results = vec![
            RagResult {
                content: "Test content".to_string(),
                relevance_score: 0.9,
                source: Some("test.rs".to_string()),
                chunk_id: "chunk1".to_string(),
            }
        ];

        let packet = brain.analyze_with_rag("test query", rag_results);

        assert_eq!(packet.rag_results.len(), 1);
        assert_eq!(packet.rag_results[0].content, "Test content");
    }

    #[test]
    fn test_performance() {
        let brain = BrainAnalyzer::new();

        // First call initializes semantic model (can be slow)
        brain.analyze("warmup query");

        // Subsequent calls should be fast
        let start = Instant::now();
        for _ in 0..10 {
            brain.analyze("This is a complex query about microservices and databases");
        }
        let elapsed = start.elapsed();

        // 10 iterations should complete in under 5 seconds with semantic fallback
        // (semantic embedding takes ~10-50ms per query)
        assert!(elapsed.as_secs() < 5, "Performance test took {:?}", elapsed);
    }
}
