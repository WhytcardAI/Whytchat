//! # Brain Module
//!
//! Fast, non-LLM analysis system for WhytChat.
//! Analyzes user input BEFORE calling the LLM to enrich context.
//!
//! ## Components
//! - `intent`: Intent classification using regex patterns (fast path)
//! - `semantic_intent`: Semantic classification using embeddings (fallback)
//! - `keywords`: TF-IDF keyword extraction
//! - `complexity`: Text complexity scoring
//! - `context_packet`: Output data structure
//! - `analyzer`: Main orchestrator

pub mod analyzer;
pub mod complexity;
pub mod context_packet;
pub mod intent;
pub mod keywords;
pub mod semantic_intent;

// Re-export main types for convenience
// Note: Many types are exported for future use in Brain Module integration
#[allow(unused_imports)]
pub use analyzer::BrainAnalyzer;
#[allow(unused_imports)]
pub use complexity::{ComplexityMetrics, ComplexityScorer};
#[allow(unused_imports)]
pub use context_packet::{ContextPacket, Language, Strategy};
#[allow(unused_imports)]
pub use intent::{Intent, IntentClassifier, IntentResult};
#[allow(unused_imports)]
pub use keywords::{KeywordExtractor, KeywordResult};
#[allow(unused_imports)]
pub use semantic_intent::SemanticIntentClassifier;
