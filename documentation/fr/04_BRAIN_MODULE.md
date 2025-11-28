# 🧠 Module Brain - WhytChat V1

> Analyse pré-LLM pour optimiser les réponses

---

## 🎯 Objectif

Le module **Brain** analyse chaque message utilisateur AVANT d'appeler le LLM :

1. **Classification d'intent** - Comprendre ce que l'utilisateur veut
2. **Extraction de keywords** - Identifier les termes importants
3. **Score de complexité** - Évaluer la difficulté de la requête
4. **Détection de langue** - Français ou anglais
5. **Décision RAG** - Faut-il chercher dans la base de connaissances ?

---

## 📁 Structure des Fichiers

```
apps/core/src/brain/
├── mod.rs              # Exports publics
├── analyzer.rs         # BrainAnalyzer - orchestrateur principal
├── intent.rs           # Classification regex rapide
├── semantic_intent.rs  # Classification par embeddings (fallback)
├── keywords.rs         # Extraction TF-IDF
├── complexity.rs       # Score de complexité texte
└── context_packet.rs   # Struct de sortie
```

---

## 🔄 Flux d'Analyse

```
[Message Utilisateur]
         │
         ▼
┌─────────────────────┐
│   BrainAnalyzer     │
│     analyze()       │
└──────────┬──────────┘
           │
     ┌─────┴─────┬─────────────┬──────────────┐
     │           │             │              │
     ▼           ▼             ▼              ▼
┌─────────┐ ┌─────────┐ ┌───────────┐ ┌────────────┐
│ Intent  │ │Keywords │ │Complexity │ │ Language   │
│Classifier│ │Extractor│ │  Scorer   │ │ Detector   │
└────┬────┘ └────┬────┘ └─────┬─────┘ └──────┬─────┘
     │           │             │              │
     └─────┬─────┴─────────────┴──────────────┘
           │
           ▼
┌─────────────────────┐
│   ContextPacket     │
│  (résultat final)   │
└─────────────────────┘
```

---

## 📦 analyzer.rs - Orchestrateur Principal

### BrainAnalyzer Struct

```rust
pub struct BrainAnalyzer {
    intent_classifier: IntentClassifier,
    semantic_classifier: Option<SemanticIntentClassifier>,
    keyword_extractor: KeywordExtractor,
    complexity_scorer: ComplexityScorer,
}

impl BrainAnalyzer {
    pub fn new(embedder: Option<Arc<TextEmbedding>>) -> Self {
        Self {
            intent_classifier: IntentClassifier::new(),
            semantic_classifier: embedder.map(|e| SemanticIntentClassifier::new(e)),
            keyword_extractor: KeywordExtractor::new(),
            complexity_scorer: ComplexityScorer::new(),
        }
    }
}
```

### Méthode analyze()

```rust
pub fn analyze(&self, query: &str) -> ContextPacket {
    let mut packet = ContextPacket::default();

    // 1. Classification intent (two-tier)
    packet.intent = self.classify_intent_smart(query);

    // 2. Extraction keywords TF-IDF
    packet.keywords = self.keyword_extractor.extract(query, Some(10));

    // 3. Score complexité
    packet.complexity = self.complexity_scorer.analyze(query);

    // 4. Détection langue
    packet.language = self.detect_language(query);

    // 5. Stratégies suggérées
    packet.suggested_strategies = self.suggest_strategies(&packet);

    // 6. Décision RAG
    packet.should_use_rag = self.should_use_rag(&packet);

    packet
}
```

### Classification Two-Tier

```rust
/// Tier 1: Regex rapide
/// Tier 2: Semantic (si regex retourne Unknown)
fn classify_intent_smart(&self, query: &str) -> Intent {
    // Essai regex d'abord (rapide)
    let intent = self.intent_classifier.classify(query);

    // Si Unknown et semantic disponible, fallback
    if intent == Intent::Unknown {
        if let Some(semantic) = &self.semantic_classifier {
            return semantic.classify(query);
        }
    }

    intent
}
```

---

## 🎯 intent.rs - Classification Regex

### Enum Intent

```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Intent {
    Greeting,      // "Bonjour", "Hello", "Salut"
    Farewell,      // "Au revoir", "Bye", "À plus"
    Question,      // "Comment...", "What is...", "Pourquoi..."
    Command,       // "Fais...", "Create...", "Génère..."
    CodeRequest,   // "Écris du code", "Write a function"
    Explanation,   // "Explique...", "Explain...", "C'est quoi"
    Translation,   // "Traduis...", "Translate...", "En anglais"
    Analysis,      // "Analyse...", "Analyze...", "Examine"
    Creative,      // "Imagine...", "Raconte...", "Write a story"
    Help,          // "Aide...", "Help...", "Besoin d'aide"
    Unknown,       // Fallback
}
```

### IntentClassifier

```rust
pub struct IntentClassifier {
    patterns: HashMap<Intent, Vec<Regex>>,
}

impl IntentClassifier {
    pub fn new() -> Self {
        let mut patterns = HashMap::new();

        // Greeting patterns (FR + EN)
        patterns.insert(Intent::Greeting, vec![
            Regex::new(r"(?i)^(bonjour|salut|hello|hi|hey|coucou)").unwrap(),
            Regex::new(r"(?i)^(bonsoir|good\s*(morning|afternoon|evening))").unwrap(),
        ]);

        // Question patterns
        patterns.insert(Intent::Question, vec![
            Regex::new(r"(?i)^(comment|pourquoi|qu['']est[- ]ce|c['']est quoi)").unwrap(),
            Regex::new(r"(?i)^(what|why|how|when|where|who|which)").unwrap(),
            Regex::new(r"\?$").unwrap(), // Termine par ?
        ]);

        // Code patterns
        patterns.insert(Intent::CodeRequest, vec![
            Regex::new(r"(?i)(écris|génère|crée|write|generate|create)\s*(du|un|une|a|the)?\s*code").unwrap(),
            Regex::new(r"(?i)(fonction|function|class|méthode|method)").unwrap(),
            Regex::new(r"(?i)(javascript|python|rust|typescript|java|c\+\+)").unwrap(),
        ]);

        // ... autres patterns

        Self { patterns }
    }

    pub fn classify(&self, query: &str) -> Intent {
        for (intent, regexes) in &self.patterns {
            for regex in regexes {
                if regex.is_match(query) {
                    return intent.clone();
                }
            }
        }
        Intent::Unknown
    }
}
```

---

## 🔮 semantic_intent.rs - Classification Embeddings

### SemanticIntentClassifier

```rust
pub struct SemanticIntentClassifier {
    embedder: Arc<TextEmbedding>,
    intent_embeddings: HashMap<Intent, Vec<f32>>,
}

impl SemanticIntentClassifier {
    pub fn new(embedder: Arc<TextEmbedding>) -> Self {
        let mut intent_embeddings = HashMap::new();

        // Pré-calculer les embeddings pour chaque intent
        let intent_examples = vec![
            (Intent::Greeting, "hello hi bonjour salut greeting"),
            (Intent::Question, "what why how question ask explain"),
            (Intent::CodeRequest, "code function write program script"),
            (Intent::Explanation, "explain describe what is tell me about"),
            // ... autres intents
        ];

        for (intent, text) in intent_examples {
            if let Ok(embedding) = embedder.embed(vec![text], None) {
                intent_embeddings.insert(intent, embedding[0].clone());
            }
        }

        Self { embedder, intent_embeddings }
    }

    pub fn classify(&self, query: &str) -> Intent {
        // Embed la requête
        let query_embedding = match self.embedder.embed(vec![query], None) {
            Ok(emb) => emb[0].clone(),
            Err(_) => return Intent::Unknown,
        };

        // Trouver l'intent le plus proche (cosine similarity)
        let mut best_intent = Intent::Unknown;
        let mut best_score = f32::MIN;

        for (intent, embedding) in &self.intent_embeddings {
            let score = cosine_similarity(&query_embedding, embedding);
            if score > best_score && score > 0.5 { // Seuil minimum
                best_score = score;
                best_intent = intent.clone();
            }
        }

        best_intent
    }
}
```

---

## 🔑 keywords.rs - Extraction TF-IDF

### KeywordExtractor

```rust
pub struct KeywordExtractor {
    stopwords: HashSet<String>,
}

impl KeywordExtractor {
    pub fn new() -> Self {
        let stopwords = Self::load_stopwords();
        Self { stopwords }
    }

    /// Extrait les N keywords les plus importants
    pub fn extract(&self, text: &str, max_keywords: Option<usize>) -> Vec<String> {
        let max = max_keywords.unwrap_or(10);

        // 1. Tokenisation
        let tokens = self.tokenize(text);

        // 2. Filtrer stopwords
        let filtered: Vec<&str> = tokens.iter()
            .filter(|t| !self.stopwords.contains(*t))
            .filter(|t| t.len() > 2) // Min 3 caractères
            .copied()
            .collect();

        // 3. Calculer TF (Term Frequency)
        let mut tf: HashMap<&str, f32> = HashMap::new();
        for token in &filtered {
            *tf.entry(token).or_insert(0.0) += 1.0;
        }

        // 4. Normaliser et trier
        let total = filtered.len() as f32;
        let mut scored: Vec<(&str, f32)> = tf.iter()
            .map(|(word, count)| (*word, count / total))
            .collect();

        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

        // 5. Retourner top N
        scored.iter()
            .take(max)
            .map(|(word, _)| word.to_string())
            .collect()
    }

    fn tokenize(&self, text: &str) -> Vec<&str> {
        text.split(|c: char| !c.is_alphanumeric())
            .filter(|s| !s.is_empty())
            .map(|s| s.to_lowercase())
            .collect()
    }

    fn load_stopwords() -> HashSet<String> {
        // Stopwords FR + EN
        let words = vec![
            // Français
            "le", "la", "les", "un", "une", "des", "de", "du", "au", "aux",
            "ce", "cette", "ces", "mon", "ma", "mes", "ton", "ta", "tes",
            "je", "tu", "il", "elle", "nous", "vous", "ils", "elles",
            "et", "ou", "mais", "donc", "car", "ni", "que", "qui",
            "est", "sont", "être", "avoir", "fait", "faire",
            "pour", "dans", "sur", "avec", "sans", "par",
            // Anglais
            "the", "a", "an", "and", "or", "but", "is", "are", "was", "were",
            "be", "been", "being", "have", "has", "had", "do", "does", "did",
            "will", "would", "could", "should", "may", "might", "must",
            "i", "you", "he", "she", "it", "we", "they",
            "this", "that", "these", "those",
            "in", "on", "at", "to", "for", "of", "with", "by",
        ];
        words.into_iter().map(String::from).collect()
    }
}
```

---

## 📊 complexity.rs - Score de Complexité

### ComplexityScorer

```rust
pub struct ComplexityScorer;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplexityScore {
    pub overall: f32,        // 0.0 - 1.0
    pub word_count: usize,
    pub sentence_count: usize,
    pub avg_word_length: f32,
    pub technical_terms: usize,
    pub nested_clauses: usize,
}

impl ComplexityScorer {
    pub fn new() -> Self { Self }

    pub fn analyze(&self, text: &str) -> ComplexityScore {
        let words = self.count_words(text);
        let sentences = self.count_sentences(text);
        let avg_word_len = self.avg_word_length(text);
        let technical = self.count_technical_terms(text);
        let nested = self.count_nested_clauses(text);

        // Score pondéré
        let overall = (
            (words as f32 / 100.0).min(1.0) * 0.2 +
            (avg_word_len / 10.0).min(1.0) * 0.2 +
            (technical as f32 / 5.0).min(1.0) * 0.3 +
            (nested as f32 / 3.0).min(1.0) * 0.3
        ).min(1.0);

        ComplexityScore {
            overall,
            word_count: words,
            sentence_count: sentences,
            avg_word_length: avg_word_len,
            technical_terms: technical,
            nested_clauses: nested,
        }
    }

    fn count_technical_terms(&self, text: &str) -> usize {
        let technical_patterns = vec![
            r"(?i)(algorithm|fonction|function|api|database|server)",
            r"(?i)(machine\s*learning|neural|network|model)",
            r"(?i)(async|await|thread|mutex|channel)",
            r"(?i)(struct|enum|trait|interface|class)",
        ];

        technical_patterns.iter()
            .filter_map(|p| Regex::new(p).ok())
            .map(|r| r.find_iter(text).count())
            .sum()
    }
}
```

---

## 📤 context_packet.rs - Structure de Sortie

### ContextPacket

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextPacket {
    /// Intent classifié
    pub intent: Intent,

    /// Keywords extraits (max 10)
    pub keywords: Vec<String>,

    /// Score de complexité
    pub complexity: ComplexityScore,

    /// Langue détectée ("fr" ou "en")
    pub language: String,

    /// Stratégies suggérées pour le LLM
    pub suggested_strategies: Vec<String>,

    /// Faut-il utiliser le RAG ?
    pub should_use_rag: bool,

    /// Timestamp de l'analyse
    pub analyzed_at: DateTime<Utc>,
}

impl Default for ContextPacket {
    fn default() -> Self {
        Self {
            intent: Intent::Unknown,
            keywords: Vec::new(),
            complexity: ComplexityScore::default(),
            language: "en".to_string(),
            suggested_strategies: Vec::new(),
            should_use_rag: false,
            analyzed_at: Utc::now(),
        }
    }
}
```

---

## 🎛️ Logique de Décision RAG

```rust
fn should_use_rag(&self, packet: &ContextPacket) -> bool {
    // Toujours RAG pour ces intents
    if matches!(packet.intent, Intent::Question | Intent::Analysis | Intent::Explanation) {
        return true;
    }

    // RAG si complexité élevée
    if packet.complexity.overall > 0.6 {
        return true;
    }

    // RAG si keywords techniques
    let technical_keywords = ["code", "function", "api", "data", "file", "document"];
    if packet.keywords.iter().any(|k| technical_keywords.contains(&k.as_str())) {
        return true;
    }

    // Pas de RAG pour salutations simples
    if matches!(packet.intent, Intent::Greeting | Intent::Farewell) {
        return false;
    }

    false
}
```

---

## 🌍 Détection de Langue

```rust
fn detect_language(&self, query: &str) -> String {
    let query_lower = query.to_lowercase();

    // Mots français fréquents
    let french_markers = vec![
        "je", "tu", "il", "elle", "nous", "vous", "ils", "elles",
        "le", "la", "les", "un", "une", "des",
        "est", "sont", "être", "avoir", "fait",
        "que", "qui", "quoi", "comment", "pourquoi",
        "bonjour", "salut", "merci", "s'il",
    ];

    // Compter les marqueurs français
    let french_count = french_markers.iter()
        .filter(|marker| {
            let pattern = format!(r"\b{}\b", regex::escape(marker));
            Regex::new(&pattern).map(|r| r.is_match(&query_lower)).unwrap_or(false)
        })
        .count();

    if french_count >= 2 {
        "fr".to_string()
    } else {
        "en".to_string()
    }
}
```

---

## 📈 Exemple d'Utilisation

### Entrée

```
"Comment puis-je créer une fonction async en Rust qui gère les erreurs ?"
```

### Sortie (ContextPacket)

```json
{
  "intent": "CodeRequest",
  "keywords": ["fonction", "async", "rust", "erreurs", "créer", "gère"],
  "complexity": {
    "overall": 0.65,
    "word_count": 12,
    "sentence_count": 1,
    "avg_word_length": 5.2,
    "technical_terms": 3,
    "nested_clauses": 1
  },
  "language": "fr",
  "suggested_strategies": [
    "provide_code_example",
    "explain_error_handling",
    "use_rust_idioms"
  ],
  "should_use_rag": true,
  "analyzed_at": "2024-11-28T00:30:00Z"
}
```

---

_Généré depuis lecture directe de: brain/mod.rs, brain/analyzer.rs, brain/intent.rs, brain/semantic_intent.rs, brain/keywords.rs, brain/complexity.rs, brain/context_packet.rs_
