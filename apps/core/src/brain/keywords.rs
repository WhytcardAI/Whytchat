//! Keyword Extraction using TF-IDF.
//!
//! Extracts the most relevant keywords from text using Term Frequency-Inverse Document Frequency.
//! Includes stopword filtering for French and English.

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// Stopwords for French language
const STOPWORDS_FR: &[&str] = &[
    "le",
    "la",
    "les",
    "un",
    "une",
    "des",
    "du",
    "de",
    "d",
    "l",
    "et",
    "ou",
    "où",
    "mais",
    "donc",
    "or",
    "ni",
    "car",
    "je",
    "tu",
    "il",
    "elle",
    "on",
    "nous",
    "vous",
    "ils",
    "elles",
    "me",
    "te",
    "se",
    "lui",
    "leur",
    "y",
    "en",
    "mon",
    "ton",
    "son",
    "ma",
    "ta",
    "sa",
    "mes",
    "tes",
    "ses",
    "notre",
    "votre",
    "nos",
    "vos",
    "leur",
    "leurs",
    "ce",
    "cet",
    "cette",
    "ces",
    "ça",
    "ceci",
    "cela",
    "qui",
    "que",
    "quoi",
    "dont",
    "lequel",
    "laquelle",
    "ne",
    "pas",
    "plus",
    "moins",
    "très",
    "trop",
    "peu",
    "bien",
    "mal",
    "être",
    "est",
    "sont",
    "était",
    "étaient",
    "sera",
    "seront",
    "avoir",
    "ai",
    "as",
    "a",
    "avons",
    "avez",
    "ont",
    "avait",
    "avaient",
    "faire",
    "fait",
    "fais",
    "font",
    "faisait",
    "aller",
    "va",
    "vais",
    "vont",
    "allait",
    "pouvoir",
    "peut",
    "peux",
    "peuvent",
    "pouvait",
    "vouloir",
    "veut",
    "veux",
    "veulent",
    "voulait",
    "devoir",
    "doit",
    "dois",
    "doivent",
    "devait",
    "dans",
    "sur",
    "sous",
    "avec",
    "sans",
    "pour",
    "par",
    "entre",
    "avant",
    "après",
    "pendant",
    "depuis",
    "jusqu",
    "jusque",
    "ici",
    "là",
    "voici",
    "voilà",
    "quand",
    "comment",
    "pourquoi",
    "combien",
    "tout",
    "tous",
    "toute",
    "toutes",
    "autre",
    "autres",
    "même",
    "mêmes",
    "aussi",
    "encore",
    "déjà",
    "toujours",
    "jamais",
    "si",
    "alors",
    "ainsi",
    "comme",
    "parce",
    "puisque",
    "lorsque",
    "oui",
    "non",
    "peut-être",
    "c",
    "n",
    "s",
    "t",
    "qu",
    "j",
    "m",
];

/// Stopwords for English language
const STOPWORDS_EN: &[&str] = &[
    "the", "a", "an", "and", "or", "but", "nor", "for", "yet", "so", "i", "you", "he", "she", "it",
    "we", "they", "me", "him", "her", "us", "them", "my", "your", "his", "her", "its", "our",
    "their", "mine", "yours", "hers", "ours", "theirs", "this", "that", "these", "those", "who",
    "whom", "which", "what", "whose", "is", "am", "are", "was", "were", "be", "been", "being",
    "have", "has", "had", "having", "do", "does", "did", "doing", "will", "would", "shall",
    "should", "can", "could", "may", "might", "must", "in", "on", "at", "to", "from", "by", "with",
    "about", "against", "between", "into", "through", "during", "before", "after", "above",
    "below", "up", "down", "out", "off", "over", "under", "again", "further", "here", "there",
    "where", "when", "why", "how", "all", "each", "every", "both", "few", "more", "most", "other",
    "some", "any", "no", "not", "only", "own", "same", "than", "too", "very", "just", "also",
    "now", "then", "once", "always", "never", "if", "because", "as", "until", "while", "although",
    "though", "yes", "no", "maybe", "s", "t", "ve", "re", "ll", "d", "m",
];

/// Result of keyword extraction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeywordResult {
    /// The keyword
    pub keyword: String,
    /// TF-IDF score
    pub score: f32,
    /// Raw frequency in the text
    pub frequency: usize,
}

/// Keyword extractor using TF-IDF
pub struct KeywordExtractor {
    stopwords_fr: HashSet<String>,
    stopwords_en: HashSet<String>,
    min_word_length: usize,
    max_keywords: usize,
    /// IDF approximation based on common word frequencies
    idf_weights: HashMap<&'static str, f32>,
}

impl Default for KeywordExtractor {
    fn default() -> Self {
        Self::new()
    }
}

impl KeywordExtractor {
    /// Create a new keyword extractor with default settings
    pub fn new() -> Self {
        Self::with_config(3, 10)
    }

    /// Create a keyword extractor with custom configuration
    pub fn with_config(min_word_length: usize, max_keywords: usize) -> Self {
        let stopwords_fr: HashSet<String> = STOPWORDS_FR.iter().map(|s| s.to_lowercase()).collect();
        let stopwords_en: HashSet<String> = STOPWORDS_EN.iter().map(|s| s.to_lowercase()).collect();

        // Pre-computed IDF weights for common technical terms
        // Higher weight = more important/specific term
        let mut idf_weights = HashMap::new();

        // Very specific terms (high IDF)
        for term in [
            "algorithm",
            "database",
            "function",
            "module",
            "interface",
            "struct",
            "enum",
        ] {
            idf_weights.insert(term, 2.5);
        }

        // Technical terms (medium-high IDF)
        for term in [
            "code", "file", "error", "data", "system", "user", "api", "config",
        ] {
            idf_weights.insert(term, 2.0);
        }

        // Common terms (medium IDF)
        for term in ["create", "update", "delete", "get", "set", "add", "remove"] {
            idf_weights.insert(term, 1.5);
        }

        Self {
            stopwords_fr,
            stopwords_en,
            min_word_length,
            max_keywords,
            idf_weights,
        }
    }

    /// Check if a word is a stopword in either language
    fn is_stopword(&self, word: &str) -> bool {
        let lower = word.to_lowercase();
        self.stopwords_fr.contains(&lower) || self.stopwords_en.contains(&lower)
    }

    /// Tokenize text into words, filtering out non-words
    fn tokenize(&self, text: &str) -> Vec<String> {
        text.to_lowercase()
            .split(|c: char| !c.is_alphanumeric() && c != '_' && c != '-')
            .filter(|word| {
                let w = word.trim();
                w.len() >= self.min_word_length
                    && !self.is_stopword(w)
                    && !w.chars().all(|c| c.is_numeric())
            })
            .map(|s| s.to_string())
            .collect()
    }

    /// Calculate term frequency for each word
    fn calculate_tf(&self, words: &[String]) -> HashMap<String, f32> {
        let total_words = words.len() as f32;
        if total_words == 0.0 {
            return HashMap::new();
        }

        let mut freq: HashMap<String, usize> = HashMap::new();
        for word in words {
            *freq.entry(word.clone()).or_insert(0) += 1;
        }

        freq.into_iter()
            .map(|(word, count)| (word, count as f32 / total_words))
            .collect()
    }

    /// Get IDF weight for a word (approximated)
    fn get_idf(&self, word: &str) -> f32 {
        // Check if we have a pre-computed weight
        if let Some(&weight) = self.idf_weights.get(word) {
            return weight;
        }

        // Heuristic IDF based on word characteristics
        let len = word.len();

        // Longer words tend to be more specific
        let length_factor = (len as f32 / 6.0).min(1.5);

        // Words with underscores or hyphens are often technical
        let special_char_bonus = if word.contains('_') || word.contains('-') {
            0.5
        } else {
            0.0
        };

        // Base IDF + adjustments
        1.0 + length_factor + special_char_bonus
    }

    /// Extract the top N keywords from text
    pub fn extract(&self, text: &str, top_k: Option<usize>) -> Vec<KeywordResult> {
        let max_results = top_k.unwrap_or(self.max_keywords);
        let words = self.tokenize(text);

        if words.is_empty() {
            return vec![];
        }

        // Calculate TF
        let tf_scores = self.calculate_tf(&words);

        // Count frequencies for the result
        let mut freq: HashMap<String, usize> = HashMap::new();
        for word in &words {
            *freq.entry(word.clone()).or_insert(0) += 1;
        }

        // Calculate TF-IDF
        let mut tfidf_scores: Vec<KeywordResult> = tf_scores
            .into_iter()
            .map(|(word, tf)| {
                let idf = self.get_idf(&word);
                let score = tf * idf;
                let frequency = freq.get(&word).copied().unwrap_or(0);
                KeywordResult {
                    keyword: word,
                    score,
                    frequency,
                }
            })
            .collect();

        // Sort by score descending
        tfidf_scores.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // Return top N
        tfidf_scores.into_iter().take(max_results).collect()
    }

    /// Extract keywords and return just the strings
    #[allow(dead_code)]
    pub fn extract_keywords(&self, text: &str, top_k: Option<usize>) -> Vec<String> {
        self.extract(text, top_k)
            .into_iter()
            .map(|k| k.keyword)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keyword_extraction() {
        let extractor = KeywordExtractor::new();

        let text = "Create a Rust function to parse JSON data from the database";
        let keywords = extractor.extract(text, Some(5));

        assert!(!keywords.is_empty());

        // Should contain important terms
        let keyword_strings: Vec<&str> = keywords.iter().map(|k| k.keyword.as_str()).collect();
        assert!(
            keyword_strings.contains(&"rust")
                || keyword_strings.contains(&"function")
                || keyword_strings.contains(&"database")
        );
    }

    #[test]
    fn test_stopword_filtering() {
        let extractor = KeywordExtractor::new();

        let text = "the a an is are was were";
        let keywords = extractor.extract(text, Some(5));

        // All stopwords, should return empty
        assert!(keywords.is_empty());
    }

    #[test]
    fn test_french_extraction() {
        let extractor = KeywordExtractor::new();

        let text = "Créer une fonction pour analyser les données de la base de données";
        let keywords = extractor.extract(text, Some(5));

        assert!(!keywords.is_empty());

        let keyword_strings: Vec<&str> = keywords.iter().map(|k| k.keyword.as_str()).collect();
        assert!(
            keyword_strings.contains(&"fonction")
                || keyword_strings.contains(&"données")
                || keyword_strings.contains(&"analyser")
        );
    }

    #[test]
    fn test_mixed_language() {
        let extractor = KeywordExtractor::new();

        let text = "Comment créer une function en Rust pour parser du JSON?";
        let keywords = extractor.extract(text, Some(5));

        assert!(!keywords.is_empty());
    }

    #[test]
    fn test_empty_text() {
        let extractor = KeywordExtractor::new();

        let keywords = extractor.extract("", Some(5));
        assert!(keywords.is_empty());

        let keywords = extractor.extract("   ", Some(5));
        assert!(keywords.is_empty());
    }

    #[test]
    fn test_short_words_filtered() {
        let extractor = KeywordExtractor::new();

        let text = "a b c d e f g h i j";
        let keywords = extractor.extract(text, Some(5));

        // All words too short, should return empty
        assert!(keywords.is_empty());
    }
}
