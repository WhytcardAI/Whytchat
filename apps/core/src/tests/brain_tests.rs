//! Brain Module Tests
//!
//! Comprehensive tests for intent classification, keyword extraction,
//! complexity scoring, and the Brain analyzer orchestrator.

use crate::brain::{
    BrainAnalyzer, ComplexityMetrics, ComplexityScorer, ContextPacket, Intent, IntentClassifier,
    IntentResult, KeywordExtractor, KeywordResult, Language, SemanticIntentClassifier, Strategy,
};

#[cfg(test)]
mod intent_classifier_tests {
    use super::*;

    #[test]
    fn test_greeting_intent_french() {
        let classifier = IntentClassifier::new();

        let greetings = vec![
            "Bonjour",
            "Salut",
            "Coucou",
            "Bonsoir",
            "Hello",
            "bonjour comment vas-tu",
        ];

        for greeting in greetings {
            let result = classifier.classify(greeting);
            assert_eq!(
                result.intent,
                Intent::Greeting,
                "Expected Greeting for '{}'",
                greeting
            );
            assert!(
                result.confidence >= 0.8,
                "Expected high confidence for '{}'",
                greeting
            );
        }
    }

    #[test]
    fn test_greeting_intent_english() {
        let classifier = IntentClassifier::new();

        let greetings = vec!["Hello", "Hi", "Hey", "Good morning", "Good evening"];

        for greeting in greetings {
            let result = classifier.classify(greeting);
            assert_eq!(
                result.intent,
                Intent::Greeting,
                "Expected Greeting for '{}'",
                greeting
            );
        }
    }

    #[test]
    fn test_question_intent() {
        let classifier = IntentClassifier::new();

        let questions = vec![
            "What is the meaning of life?",
            "How do I install Rust?",
            "Why is the sky blue?",
            "When was Python created?",
            "Where can I find documentation?",
            "Qu'est-ce que le machine learning?",
            "Comment fonctionne un moteur?",
            "Pourquoi utiliser Rust?",
        ];

        for question in questions {
            let result = classifier.classify(question);
            assert_eq!(
                result.intent,
                Intent::Question,
                "Expected Question for '{}'",
                question
            );
        }
    }

    #[test]
    fn test_code_generation_intent() {
        let classifier = IntentClassifier::new();

        let code_requests = vec![
            "Write a function to sort an array",
            "Generate code for a REST API",
            "Create a Python class for user management",
            "Implement a binary search algorithm",
            "Code a React component",
            "Écris une fonction pour calculer la factorielle",
            "Génère du code pour parser du JSON",
        ];

        for request in code_requests {
            let result = classifier.classify(request);
            assert_eq!(
                result.intent,
                Intent::CodeGeneration,
                "Expected CodeGeneration for '{}'",
                request
            );
        }
    }

    #[test]
    fn test_explanation_intent() {
        let classifier = IntentClassifier::new();

        let explanations = vec![
            "Explain how neural networks work",
            "Describe the process of photosynthesis",
            "Tell me about quantum computing",
            "Explique-moi les bases de données",
            "Décris le fonctionnement d'un compilateur",
        ];

        for request in explanations {
            let result = classifier.classify(request);
            assert_eq!(
                result.intent,
                Intent::Explanation,
                "Expected Explanation for '{}'",
                request
            );
        }
    }

    #[test]
    fn test_summarization_intent() {
        let classifier = IntentClassifier::new();

        let summaries = vec![
            "Summarize this article",
            "Give me a summary of the document",
            "TL;DR this text",
            "Résume ce texte",
            "Fais un résumé de l'article",
        ];

        for request in summaries {
            let result = classifier.classify(request);
            assert_eq!(
                result.intent,
                Intent::Summarization,
                "Expected Summarization for '{}'",
                request
            );
        }
    }

    #[test]
    fn test_search_intent() {
        let classifier = IntentClassifier::new();

        let searches = vec![
            "Search for information about climate change",
            "Find documents about machine learning",
            "Look up the definition of entropy",
            "Cherche des informations sur le réchauffement",
            "Trouve des articles sur l'IA",
        ];

        for request in searches {
            let result = classifier.classify(request);
            assert_eq!(
                result.intent,
                Intent::Search,
                "Expected Search for '{}'",
                request
            );
        }
    }

    #[test]
    fn test_unknown_intent_fallback() {
        let classifier = IntentClassifier::new();

        let ambiguous = vec!["xyz abc 123", "random gibberish here", "!@#$%^&*"];

        for text in ambiguous {
            let result = classifier.classify(text);
            // Should either be Unknown or have low confidence
            if result.intent != Intent::Unknown {
                assert!(
                    result.confidence < 0.5,
                    "Expected low confidence for ambiguous text: '{}'",
                    text
                );
            }
        }
    }

    #[test]
    fn test_intent_confidence_range() {
        let classifier = IntentClassifier::new();

        let inputs = vec![
            "Hello world",
            "What is Rust?",
            "Write code",
            "Summarize",
            "Random text here",
        ];

        for input in inputs {
            let result = classifier.classify(input);
            assert!(
                result.confidence >= 0.0 && result.confidence <= 1.0,
                "Confidence should be between 0 and 1 for '{}'",
                input
            );
        }
    }
}

#[cfg(test)]
mod keyword_extractor_tests {
    use super::*;

    #[test]
    fn test_keyword_extraction_basic() {
        let extractor = KeywordExtractor::new();

        let text = "Rust programming language is fast and memory-safe";
        let result = extractor.extract(text);

        assert!(
            result.keywords.len() > 0,
            "Should extract at least one keyword"
        );
    }

    #[test]
    fn test_keyword_extraction_technical() {
        let extractor = KeywordExtractor::new();

        let text = "Machine learning neural networks deep learning artificial intelligence";
        let result = extractor.extract(text);

        let keywords_lower: Vec<String> = result
            .keywords
            .iter()
            .map(|k| k.to_lowercase())
            .collect();

        // Should contain relevant technical terms
        assert!(
            keywords_lower
                .iter()
                .any(|k| k.contains("learning") || k.contains("neural") || k.contains("intelligence")),
            "Should extract technical keywords"
        );
    }

    #[test]
    fn test_keyword_extraction_french() {
        let extractor = KeywordExtractor::new();

        let text = "L'apprentissage automatique utilise des réseaux de neurones pour l'analyse de données";
        let result = extractor.extract(text);

        assert!(
            result.keywords.len() > 0,
            "Should extract keywords from French text"
        );
    }

    #[test]
    fn test_keyword_score_range() {
        let extractor = KeywordExtractor::new();

        let text = "Programming languages like Rust, Python, and JavaScript are popular";
        let result = extractor.extract(text);

        for (_, score) in &result.scores {
            assert!(
                *score >= 0.0,
                "Keyword score should be non-negative"
            );
        }
    }

    #[test]
    fn test_empty_input() {
        let extractor = KeywordExtractor::new();

        let result = extractor.extract("");
        // Should handle empty input gracefully
        assert!(result.keywords.len() == 0 || result.keywords.is_empty());
    }

    #[test]
    fn test_stopword_filtering() {
        let extractor = KeywordExtractor::new();

        let text = "the a an is are was were be been being";
        let result = extractor.extract(text);

        // Should filter out most stopwords
        assert!(
            result.keywords.len() <= 2,
            "Stopwords should be filtered"
        );
    }
}

#[cfg(test)]
mod complexity_scorer_tests {
    use super::*;

    #[test]
    fn test_simple_text_low_complexity() {
        let scorer = ComplexityScorer::new();

        let simple_text = "Hello. How are you?";
        let result = scorer.score(simple_text);

        assert!(
            result.score < 0.4,
            "Simple text should have low complexity score: {}",
            result.score
        );
    }

    #[test]
    fn test_complex_text_high_complexity() {
        let scorer = ComplexityScorer::new();

        let complex_text = "Please analyze the architectural patterns in microservices, \
                           considering event-driven design, CQRS, and saga patterns for \
                           distributed transaction management with proper error handling \
                           and eventual consistency guarantees.";
        let result = scorer.score(complex_text);

        assert!(
            result.score > 0.5,
            "Complex text should have higher complexity score: {}",
            result.score
        );
    }

    #[test]
    fn test_complexity_increases_with_length() {
        let scorer = ComplexityScorer::new();

        let short = "Hello";
        let medium = "Hello, how are you doing today?";
        let long = "Hello, how are you doing today? I wanted to discuss the implementation \
                   of our new feature that involves multiple microservices and databases.";

        let short_score = scorer.score(short);
        let medium_score = scorer.score(medium);
        let long_score = scorer.score(long);

        assert!(
            medium_score.word_count >= short_score.word_count,
            "Medium text should have more words"
        );
        assert!(
            long_score.word_count >= medium_score.word_count,
            "Long text should have more words"
        );
    }

    #[test]
    fn test_complexity_metrics_word_count() {
        let scorer = ComplexityScorer::new();

        let text = "One two three four five";
        let result = scorer.score(text);

        assert_eq!(result.word_count, 5, "Should count 5 words");
    }

    #[test]
    fn test_complexity_metrics_sentence_count() {
        let scorer = ComplexityScorer::new();

        let text = "First sentence. Second sentence. Third sentence!";
        let result = scorer.score(text);

        assert!(
            result.sentence_count >= 2,
            "Should count at least 2 sentences"
        );
    }

    #[test]
    fn test_complexity_score_range() {
        let scorer = ComplexityScorer::new();

        let texts = vec![
            "Hi",
            "How are you?",
            "This is a longer sentence with more words.",
            "Complex technical documentation with multiple paragraphs and detailed explanations.",
        ];

        for text in texts {
            let result = scorer.score(text);
            assert!(
                result.score >= 0.0 && result.score <= 1.0,
                "Complexity score should be between 0 and 1 for '{}'",
                text
            );
        }
    }
}

#[cfg(test)]
mod brain_analyzer_tests {
    use super::*;

    #[test]
    fn test_analyzer_creation() {
        let analyzer = BrainAnalyzer::new();
        // Should create without panic
        assert!(true);
    }

    #[test]
    fn test_analyzer_default() {
        let analyzer = BrainAnalyzer::default();
        // Should create using Default trait
        assert!(true);
    }

    #[test]
    fn test_full_analysis_greeting() {
        let analyzer = BrainAnalyzer::new();

        let result = analyzer.analyze("Bonjour, comment ça va?");

        assert_eq!(result.intent.intent, Intent::Greeting);
        assert!(!result.should_use_rag, "Greeting should not use RAG");
        assert_eq!(result.language, Language::French);
    }

    #[test]
    fn test_full_analysis_question() {
        let analyzer = BrainAnalyzer::new();

        let result = analyzer.analyze("What is the capital of France?");

        assert_eq!(result.intent.intent, Intent::Question);
        assert_eq!(result.language, Language::English);
    }

    #[test]
    fn test_full_analysis_code_request() {
        let analyzer = BrainAnalyzer::new();

        let result = analyzer.analyze("Write a Rust function to calculate fibonacci");

        assert_eq!(result.intent.intent, Intent::CodeGeneration);
        assert!(
            result.keywords.len() > 0,
            "Should extract keywords for code requests"
        );
    }

    #[test]
    fn test_language_detection_french() {
        let analyzer = BrainAnalyzer::new();

        let french_texts = vec![
            "Bonjour, je suis français",
            "Comment ça va aujourd'hui?",
            "Je voudrais comprendre le machine learning",
            "Qu'est-ce que la programmation?",
        ];

        for text in french_texts {
            let result = analyzer.analyze(text);
            assert_eq!(
                result.language,
                Language::French,
                "Should detect French for '{}'",
                text
            );
        }
    }

    #[test]
    fn test_language_detection_english() {
        let analyzer = BrainAnalyzer::new();

        let english_texts = vec![
            "Hello, I am learning Rust",
            "What is machine learning?",
            "Please explain the concept",
            "How do databases work?",
        ];

        for text in english_texts {
            let result = analyzer.analyze(text);
            assert_eq!(
                result.language,
                Language::English,
                "Should detect English for '{}'",
                text
            );
        }
    }

    #[test]
    fn test_strategy_selection() {
        let analyzer = BrainAnalyzer::new();

        // Simple greeting - should use fast strategy
        let greeting = analyzer.analyze("Hello!");
        assert_eq!(
            greeting.suggested_strategy,
            Strategy::Direct,
            "Greeting should use Direct strategy"
        );

        // Complex question - should use RAG
        let question = analyzer.analyze(
            "Explain the differences between microservices and monolithic architecture \
             with examples from our documentation",
        );
        assert!(
            question.should_use_rag,
            "Complex question should suggest RAG"
        );
    }

    #[test]
    fn test_context_packet_completeness() {
        let analyzer = BrainAnalyzer::new();

        let result = analyzer.analyze("Write a Python function for data analysis");

        // Verify all fields are populated
        assert!(result.intent.confidence > 0.0);
        assert!(result.complexity.score >= 0.0);
        assert!(!result.raw_input.is_empty());
        assert!(result.timestamp > 0);
    }

    #[test]
    fn test_rag_decision_for_search() {
        let analyzer = BrainAnalyzer::new();

        let result = analyzer.analyze("Search in my documents for information about Rust");

        assert!(
            result.should_use_rag,
            "Search intent should trigger RAG usage"
        );
    }

    #[test]
    fn test_rag_decision_for_greeting() {
        let analyzer = BrainAnalyzer::new();

        let result = analyzer.analyze("Hi there!");

        assert!(
            !result.should_use_rag,
            "Greeting should not trigger RAG usage"
        );
    }

    #[test]
    fn test_processing_time_reasonable() {
        let analyzer = BrainAnalyzer::new();
        let start = std::time::Instant::now();

        for _ in 0..100 {
            let _ = analyzer.analyze("What is the meaning of life?");
        }

        let elapsed = start.elapsed();
        assert!(
            elapsed.as_millis() < 1000,
            "100 analyses should complete in under 1 second: {:?}",
            elapsed
        );
    }
}

#[cfg(test)]
mod context_packet_tests {
    use super::*;

    #[test]
    fn test_language_enum_display() {
        assert_eq!(format!("{:?}", Language::French), "French");
        assert_eq!(format!("{:?}", Language::English), "English");
        assert_eq!(format!("{:?}", Language::Unknown), "Unknown");
    }

    #[test]
    fn test_strategy_enum_display() {
        assert_eq!(format!("{:?}", Strategy::Direct), "Direct");
        assert_eq!(format!("{:?}", Strategy::Rag), "Rag");
        assert_eq!(format!("{:?}", Strategy::Hybrid), "Hybrid");
    }

    #[test]
    fn test_intent_enum_completeness() {
        let intents = vec![
            Intent::Greeting,
            Intent::Question,
            Intent::CodeGeneration,
            Intent::Explanation,
            Intent::Summarization,
            Intent::Search,
            Intent::Unknown,
        ];

        for intent in intents {
            // All intents should be debuggable
            let _ = format!("{:?}", intent);
        }
    }
}
