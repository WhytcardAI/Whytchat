//! Intent Classification using regex patterns.
//!
//! Fast pattern-based intent detection for FR and EN languages.
//! No ML model required - pure Rust regex matching.

use regex::Regex;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::sync::LazyLock;

/// Detected intent type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Intent {
    /// Question (contains ?, how, why, what, etc.)
    Question,
    /// Command/Imperative (do, create, make, etc.)
    Command,
    /// Code-related request (code, function, debug, etc.)
    CodeRequest,
    /// Creative writing (write, compose, imagine, etc.)
    Creative,
    /// Greeting (hello, hi, bonjour, etc.)
    Greeting,
    /// Farewell (goodbye, bye, au revoir, etc.)
    Farewell,
    /// Analysis request (analyze, compare, summarize, etc.)
    Analysis,
    /// Translation request (translate, traduis, etc.)
    Translation,
    /// Explanation request (explain, clarify, etc.)
    Explanation,
    /// Help/Assistance (help, aide, etc.)
    Help,
    /// Unknown/Default
    Unknown,
}

impl fmt::Display for Intent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.label())
    }
}

impl Intent {
    /// Returns a human-readable label for the intent
    pub fn label(&self) -> &'static str {
        match self {
            Intent::Question => "question",
            Intent::Command => "command",
            Intent::CodeRequest => "code_request",
            Intent::Creative => "creative",
            Intent::Greeting => "greeting",
            Intent::Farewell => "farewell",
            Intent::Analysis => "analysis",
            Intent::Translation => "translation",
            Intent::Explanation => "explanation",
            Intent::Help => "help",
            Intent::Unknown => "conversation",
        }
    }
}

/// Result of intent classification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntentResult {
    /// Detected intent
    pub intent: Intent,
    /// Confidence score (0.0 - 1.0)
    pub confidence: f32,
    /// Patterns that matched
    pub matched_patterns: Vec<String>,
}

/// Pattern definition for intent matching
struct IntentPattern {
    intent: Intent,
    patterns: Vec<Regex>,
    weight: f32,
}

/// Intent classifier using regex patterns
pub struct IntentClassifier {
    patterns: Vec<IntentPattern>,
}

// Compile patterns once at startup
// NOTE: expect() is acceptable here per STANDARDS.md: "lazy_static peut paniquer si l'état est irrécupérable"
static QUESTION_PATTERNS: LazyLock<Vec<Regex>> = LazyLock::new(|| {
    vec![
        // Universal
        Regex::new(r"\?").expect("Invalid regex: question mark pattern"),
        // French
        Regex::new(r"(?i)^(qu('|')|que\s)").expect("Invalid regex: French qu pattern"),
        Regex::new(r"(?i)\b(comment|pourquoi|quand|où|qui|quel|quelle|quels|quelles|combien)\b").expect("Invalid regex: French question words"),
        Regex::new(r"(?i)\b(est-ce que|c'est quoi|qu'est-ce)\b").expect("Invalid regex: French question phrases"),
        Regex::new(r"(?i)\b(peux-tu|pourrais-tu|sais-tu|connais-tu)\b").expect("Invalid regex: French can-you patterns"),
        // English
        Regex::new(r"(?i)^(what|why|how|when|where|who|which|whose)\b").expect("Invalid regex: English question words"),
        Regex::new(r"(?i)\b(can you|could you|do you|would you|is it|are there)\b").expect("Invalid regex: English can-you patterns"),
        Regex::new(r"(?i)\b(what's|what is|how do|how can|how to)\b").expect("Invalid regex: English what-how patterns"),
    ]
});

static COMMAND_PATTERNS: LazyLock<Vec<Regex>> = LazyLock::new(|| {
    vec![
        // French imperatives
        Regex::new(r"(?i)^(fais|fait|crée|créer|génère|génèr|exécute|lance|démarre|arrête|supprime|ajoute|modifie|change)\b").expect("Invalid regex: French command verbs 1"),
        Regex::new(r"(?i)^(mets|met|envoie|sauvegarde|ouvre|ferme|installe|configure)\b").expect("Invalid regex: French command verbs 2"),
        Regex::new(r"(?i)^(parle|dis|réponds|raconte|décris|montre|donne|trouve|cherche)\b").expect("Invalid regex: French command verbs 3"),
        // English imperatives
        Regex::new(r"(?i)^(do|make|create|generate|run|execute|start|stop|delete|add|modify|change)\b").expect("Invalid regex: English command verbs 1"),
        Regex::new(r"(?i)^(put|send|save|open|close|install|configure|set|build|deploy)\b").expect("Invalid regex: English command verbs 2"),
        Regex::new(r"(?i)^(speak|talk|tell|say|respond|describe|show|give|find|search)\b").expect("Invalid regex: English command verbs 3"),
        // Action verbs with "please"
        Regex::new(r"(?i)\b(s'il te plaît|s'il vous plaît|please|pls)\b").expect("Invalid regex: please patterns"),
    ]
});

static CODE_PATTERNS: LazyLock<Vec<Regex>> = LazyLock::new(|| {
    vec![
        // Code keywords
        Regex::new(r"(?i)\b(code|coder|programmer|script|fonction|function|class|classe|method|méthode)\b").expect("Invalid regex: code keywords"),
        Regex::new(r"(?i)\b(debug|debugger|bug|error|erreur|fix|fixer|corriger)\b").expect("Invalid regex: debug keywords"),
        Regex::new(r"(?i)\b(compile|compiler|build|test|tester|refactor|refactorer)\b").expect("Invalid regex: build keywords"),
        Regex::new(r"(?i)\b(variable|array|tableau|loop|boucle|condition|if|else)\b").expect("Invalid regex: programming constructs"),
        // Programming languages
        Regex::new(r"(?i)\b(rust|python|javascript|typescript|java|c\+\+|go|ruby|php|sql)\b").expect("Invalid regex: programming languages"),
        Regex::new(r"(?i)\b(react|vue|angular|node|express|django|flask|rails)\b").expect("Invalid regex: frameworks"),
        // Code artifacts
        Regex::new(r"```").expect("Invalid regex: code block marker"),
        Regex::new(r"(?i)\b(api|endpoint|route|module|package|crate|library)\b").expect("Invalid regex: code artifacts"),
    ]
});

static CREATIVE_PATTERNS: LazyLock<Vec<Regex>> = LazyLock::new(|| {
    vec![
        // French
        Regex::new(r"(?i)^(écris|écrire|rédige|rédiger|compose|composer)\b").expect("Invalid regex: French creative verbs"),
        Regex::new(r"(?i)\b(imagine|imaginer|invente|inventer|crée une histoire|raconte)\b").expect("Invalid regex: French imagination verbs"),
        Regex::new(r"(?i)\b(poème|poésie|chanson|histoire|conte|récit|roman)\b").expect("Invalid regex: French creative nouns"),
        // English
        Regex::new(r"(?i)^(write|compose|draft|create a story)\b").expect("Invalid regex: English creative verbs"),
        Regex::new(r"(?i)\b(imagine|invent|story|tale|poem|poetry|song|lyrics)\b").expect("Invalid regex: English imagination words"),
        Regex::new(r"(?i)\b(creative|fiction|narrative|essay|article)\b").expect("Invalid regex: English creative nouns"),
    ]
});

static GREETING_PATTERNS: LazyLock<Vec<Regex>> = LazyLock::new(|| {
    vec![
        // French
        Regex::new(r"(?i)^(bonjour|bonsoir|salut|coucou|hey|yo)\b").expect("Invalid regex: French greetings"),
        Regex::new(r"(?i)^(bienvenue|rebonjour)\b").expect("Invalid regex: French welcome"),
        // English
        Regex::new(r"(?i)^(hello|hi|hey|greetings|good morning|good afternoon|good evening)\b").expect("Invalid regex: English greetings"),
        Regex::new(r"(?i)^(howdy|what's up|sup)\b").expect("Invalid regex: English informal greetings"),
    ]
});

static FAREWELL_PATTERNS: LazyLock<Vec<Regex>> = LazyLock::new(|| {
    vec![
        // French - Note: "salut" removed as it's more commonly a greeting
        Regex::new(r"(?i)\b(au revoir|à bientôt|à plus|bye|ciao|adieu|bonne nuit)\b").expect("Invalid regex: French farewells"),
        // English
        Regex::new(r"(?i)\b(goodbye|bye|farewell|see you|take care|good night)\b").expect("Invalid regex: English farewells"),
    ]
});

static ANALYSIS_PATTERNS: LazyLock<Vec<Regex>> = LazyLock::new(|| {
    vec![
        // French
        Regex::new(r"(?i)\b(analyse|analyser|compare|comparer|évalue|évaluer|examine|examiner)\b").expect("Invalid regex: French analysis verbs"),
        Regex::new(r"(?i)\b(résume|résumer|synthèse|synthétise|récapitule)\b").expect("Invalid regex: French summary verbs"),
        Regex::new(r"(?i)\b(différence|avantage|inconvénient|pour et contre)\b").expect("Invalid regex: French comparison nouns"),
        // English
        Regex::new(r"(?i)\b(analyze|analyse|compare|evaluate|examine|assess|review)\b").expect("Invalid regex: English analysis verbs"),
        Regex::new(r"(?i)\b(summarize|summarise|synthesize|recap|overview)\b").expect("Invalid regex: English summary verbs"),
        Regex::new(r"(?i)\b(difference|pros and cons|advantages|disadvantages)\b").expect("Invalid regex: English comparison nouns"),
    ]
});

static TRANSLATION_PATTERNS: LazyLock<Vec<Regex>> = LazyLock::new(|| {
    vec![
        // French
        Regex::new(r"(?i)\b(traduis|traduire|traduction|traduisez)\b").expect("Invalid regex: French translation verbs"),
        Regex::new(r"(?i)\b(en français|en anglais|en espagnol|en allemand)\b").expect("Invalid regex: French language targets"),
        // English
        Regex::new(r"(?i)\b(translate|translation)\b").expect("Invalid regex: English translation verbs"),
        Regex::new(r"(?i)\b(to french|to english|to spanish|to german|into)\b").expect("Invalid regex: English language targets"),
    ]
});

static EXPLANATION_PATTERNS: LazyLock<Vec<Regex>> = LazyLock::new(|| {
    vec![
        // French
        Regex::new(r"(?i)\b(explique|expliquer|clarifie|clarifier|détaille|détailler)\b").expect("Invalid regex: French explanation verbs"),
        Regex::new(r"(?i)\b(c'est quoi|définition|signifie|veut dire)\b").expect("Invalid regex: French definition phrases"),
        // English
        Regex::new(r"(?i)\b(explain|clarify|elaborate|detail)\b").expect("Invalid regex: English explanation verbs"),
        Regex::new(r"(?i)\b(what does.*mean|definition|meaning of)\b").expect("Invalid regex: English definition phrases"),
    ]
});

static HELP_PATTERNS: LazyLock<Vec<Regex>> = LazyLock::new(|| {
    vec![
        // French
        Regex::new(r"(?i)^(aide|aidez|help)\b").expect("Invalid regex: French help words"),
        Regex::new(r"(?i)\b(j'ai besoin d'aide|peux-tu m'aider|besoin d'assistance)\b").expect("Invalid regex: French help phrases"),
        // English
        Regex::new(r"(?i)\b(help me|i need help|assist|assistance|support)\b").expect("Invalid regex: English help words"),
        Regex::new(r"(?i)\b(can you help|could you help)\b").expect("Invalid regex: English help phrases"),
    ]
});

impl Default for IntentClassifier {
    fn default() -> Self {
        Self::new()
    }
}

impl IntentClassifier {
    /// Create a new intent classifier with all patterns
    pub fn new() -> Self {
        let patterns = vec![
            IntentPattern {
                intent: Intent::Greeting,
                patterns: GREETING_PATTERNS.clone(),
                weight: 1.0, // High priority for greetings
            },
            IntentPattern {
                intent: Intent::Farewell,
                patterns: FAREWELL_PATTERNS.clone(),
                weight: 1.0,
            },
            IntentPattern {
                intent: Intent::CodeRequest,
                patterns: CODE_PATTERNS.clone(),
                weight: 0.9,
            },
            IntentPattern {
                intent: Intent::Translation,
                patterns: TRANSLATION_PATTERNS.clone(),
                weight: 0.85,
            },
            IntentPattern {
                intent: Intent::Analysis,
                patterns: ANALYSIS_PATTERNS.clone(),
                weight: 0.8,
            },
            IntentPattern {
                intent: Intent::Creative,
                patterns: CREATIVE_PATTERNS.clone(),
                weight: 0.8,
            },
            IntentPattern {
                intent: Intent::Explanation,
                patterns: EXPLANATION_PATTERNS.clone(),
                weight: 0.75,
            },
            IntentPattern {
                intent: Intent::Help,
                patterns: HELP_PATTERNS.clone(),
                weight: 0.7,
            },
            IntentPattern {
                intent: Intent::Command,
                patterns: COMMAND_PATTERNS.clone(),
                weight: 0.7,
            },
            IntentPattern {
                intent: Intent::Question,
                patterns: QUESTION_PATTERNS.clone(),
                weight: 0.6, // Lower priority - questions are common
            },
        ];

        Self { patterns }
    }

    /// Classify the intent of a text
    pub fn classify(&self, text: &str) -> IntentResult {
        let text = text.trim();

        if text.is_empty() {
            return IntentResult {
                intent: Intent::Unknown,
                confidence: 0.0,
                matched_patterns: vec![],
            };
        }

        let mut best_intent = Intent::Unknown;
        let mut best_score: f32 = 0.0;
        let mut matched_patterns = Vec::new();

        for pattern_group in &self.patterns {
            let mut match_count = 0;
            let mut group_patterns = Vec::new();

            for pattern in &pattern_group.patterns {
                if pattern.is_match(text) {
                    match_count += 1;
                    if let Some(m) = pattern.find(text) {
                        group_patterns.push(m.as_str().to_string());
                    }
                }
            }

            if match_count > 0 {
                // Score based on match count and pattern weight
                let pattern_count = pattern_group.patterns.len() as f32;
                let match_ratio = match_count as f32 / pattern_count;
                let score = match_ratio * pattern_group.weight;

                if score > best_score {
                    best_score = score;
                    best_intent = pattern_group.intent;
                    matched_patterns = group_patterns;
                }
            }
        }

        // Normalize confidence to 0.0-1.0
        let confidence = (best_score * 1.2).min(1.0);

        IntentResult {
            intent: best_intent,
            confidence,
            matched_patterns,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_question_detection() {
        let classifier = IntentClassifier::new();

        let result = classifier.classify("Comment ça marche ?");
        assert_eq!(result.intent, Intent::Question);

        let result = classifier.classify("What is Rust?");
        assert_eq!(result.intent, Intent::Question);

        let result = classifier.classify("Pourquoi utiliser ce framework?");
        assert_eq!(result.intent, Intent::Question);
    }

    #[test]
    fn test_greeting_detection() {
        let classifier = IntentClassifier::new();

        let result = classifier.classify("Bonjour!");
        assert_eq!(result.intent, Intent::Greeting);

        let result = classifier.classify("Hello there");
        assert_eq!(result.intent, Intent::Greeting);

        let result = classifier.classify("Salut, comment vas-tu?");
        assert_eq!(result.intent, Intent::Greeting);
    }

    #[test]
    fn test_code_detection() {
        let classifier = IntentClassifier::new();

        let result = classifier.classify("Écris une fonction Python");
        assert_eq!(result.intent, Intent::CodeRequest);

        let result = classifier.classify("Debug this JavaScript code");
        assert_eq!(result.intent, Intent::CodeRequest);

        let result = classifier.classify("Create a Rust module for parsing");
        assert_eq!(result.intent, Intent::CodeRequest);
    }

    #[test]
    fn test_command_detection() {
        let classifier = IntentClassifier::new();

        let result = classifier.classify("Fais un résumé");
        assert_eq!(result.intent, Intent::Command);

        let result = classifier.classify("Create a new file");
        assert_eq!(result.intent, Intent::Command);
    }

    #[test]
    fn test_translation_detection() {
        let classifier = IntentClassifier::new();

        let result = classifier.classify("Traduis en anglais");
        assert_eq!(result.intent, Intent::Translation);

        let result = classifier.classify("Translate to French");
        assert_eq!(result.intent, Intent::Translation);
    }

    #[test]
    fn test_unknown_detection() {
        let classifier = IntentClassifier::new();

        let result = classifier.classify("");
        assert_eq!(result.intent, Intent::Unknown);

        let result = classifier.classify("   ");
        assert_eq!(result.intent, Intent::Unknown);
    }
}
