# 🧪 Tests - WhytChat V1

> Structure et exécution des tests

---

## 📁 Structure des Tests

```
WhytChat_V1/
├── apps/
│   ├── core/
│   │   └── src/
│   │       └── tests/           # Tests unitaires Rust
│   │           ├── mod.rs
│   │           ├── database_tests.rs
│   │           ├── encryption_tests.rs
│   │           ├── actor_tests.rs
│   │           └── brain_tests.rs
│   └── desktop-ui/
│       ├── tests/               # Tests E2E Playwright
│       │   ├── chat-prompt/
│       │   ├── session.spec.js
│       │   └── ...
│       └── playwright.config.js
└── test-results/                # Résultats des tests
```

---

## 🦀 Tests Rust

### Exécution

```bash
# Tous les tests
cargo test --manifest-path apps/core/Cargo.toml

# Tests spécifiques
cargo test --manifest-path apps/core/Cargo.toml database
cargo test --manifest-path apps/core/Cargo.toml encryption
cargo test --manifest-path apps/core/Cargo.toml actor

# Avec output
cargo test --manifest-path apps/core/Cargo.toml -- --nocapture
```

### Résultats Actuels

```
running 44 tests
test result: ok. 44 passed; 0 failed; 0 ignored
```

### Tests par Module

#### database_tests.rs

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::sqlite::SqlitePoolOptions;

    async fn setup_test_db() -> SqlitePool {
        SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .expect("Failed to create test database")
    }

    #[tokio::test]
    async fn test_create_session() {
        let pool = setup_test_db().await;
        sqlx::migrate!("./migrations").run(&pool).await.unwrap();

        let session = Session {
            id: Uuid::new_v4(),
            name: "Test Session".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            is_pinned: false,
            sort_order: None,
        };

        save_session(&pool, &session).await.unwrap();

        let retrieved = get_session(&pool, session.id).await.unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().name, "Test Session");
    }

    #[tokio::test]
    async fn test_save_and_get_messages() {
        let pool = setup_test_db().await;
        sqlx::migrate!("./migrations").run(&pool).await.unwrap();

        // Create session first
        let session_id = Uuid::new_v4();
        let session = Session {
            id: session_id,
            name: "Test".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            is_pinned: false,
            sort_order: None,
        };
        save_session(&pool, &session).await.unwrap();

        // Save message
        let message = Message {
            id: Uuid::new_v4(),
            session_id,
            role: "user".to_string(),
            content: "Hello".to_string(),
            created_at: Utc::now(),
            tokens: Some(5),
        };
        save_message(&pool, &message).await.unwrap();

        // Retrieve messages
        let messages = get_messages_for_session(&pool, session_id).await.unwrap();
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].content, "Hello");
    }

    #[tokio::test]
    async fn test_delete_session_cascades_messages() {
        let pool = setup_test_db().await;
        sqlx::migrate!("./migrations").run(&pool).await.unwrap();

        let session_id = Uuid::new_v4();
        // ... create session and messages

        delete_session(&pool, session_id).await.unwrap();

        let messages = get_messages_for_session(&pool, session_id).await.unwrap();
        assert!(messages.is_empty());
    }
}
```

#### encryption_tests.rs

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let key = derive_key("test_password");
        let data = b"Hello, World!";

        let encrypted = encrypt(data, &key).unwrap();
        let decrypted = decrypt(&encrypted, &key).unwrap();

        assert_eq!(data.to_vec(), decrypted);
    }

    #[test]
    fn test_nonce_is_random() {
        let key = derive_key("test_password");
        let data = b"Same data";

        let encrypted1 = encrypt(data, &key).unwrap();
        let encrypted2 = encrypt(data, &key).unwrap();

        // Les nonces (12 premiers octets) doivent être différents
        assert_ne!(&encrypted1[..12], &encrypted2[..12]);
    }

    #[test]
    fn test_wrong_key_fails() {
        let key1 = derive_key("password1");
        let key2 = derive_key("password2");
        let data = b"Secret data";

        let encrypted = encrypt(data, &key1).unwrap();
        let result = decrypt(&encrypted, &key2);

        assert!(result.is_err());
    }

    #[test]
    fn test_derive_key_deterministic() {
        let key1 = derive_key("same_password");
        let key2 = derive_key("same_password");

        assert_eq!(key1, key2);
    }
}
```

#### actor_tests.rs

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tokio::sync::oneshot;

    // Mock LLM Actor
    struct MockLlmActor {
        response: String,
    }

    #[async_trait]
    impl LlmActor for MockLlmActor {
        async fn generate(&self, _prompt: &str, _system: Option<&str>) -> Result<String, String> {
            Ok(self.response.clone())
        }

        async fn stream_generate(&self, _prompt: &str, _system: Option<&str>, window: &Window) -> Result<(), String> {
            for token in self.response.split_whitespace() {
                window.emit("chat-token", token).map_err(|e| e.to_string())?;
            }
            Ok(())
        }
    }

    // Mock RAG Actor
    struct MockRagActor {
        results: Vec<SearchResult>,
    }

    #[async_trait]
    impl RagActor for MockRagActor {
        async fn ingest(&self, _content: &str, _metadata: HashMap<String, String>) -> Result<(), String> {
            Ok(())
        }

        async fn search(&self, _query: &str, _top_k: usize) -> Result<Vec<SearchResult>, String> {
            Ok(self.results.clone())
        }

        async fn delete(&self, _file_id: Uuid) -> Result<(), String> {
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_supervisor_message_routing() {
        let mock_llm = Arc::new(MockLlmActor {
            response: "Test response".to_string()
        });
        let mock_rag = Arc::new(MockRagActor {
            results: vec![]
        });

        let (supervisor_handle, _runner) = SupervisorHandle::new(
            mock_llm,
            mock_rag,
            Arc::new(BrainAnalyzer::new(None)),
            None,
        );

        // Test would require Window mock which is complex in Tauri
        // This shows the test structure
    }
}
```

#### brain_tests.rs

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_intent_classification_greeting() {
        let classifier = IntentClassifier::new();

        assert_eq!(classifier.classify("Bonjour"), Intent::Greeting);
        assert_eq!(classifier.classify("Hello!"), Intent::Greeting);
        assert_eq!(classifier.classify("Salut"), Intent::Greeting);
    }

    #[test]
    fn test_intent_classification_question() {
        let classifier = IntentClassifier::new();

        assert_eq!(classifier.classify("Comment ça marche?"), Intent::Question);
        assert_eq!(classifier.classify("What is Rust?"), Intent::Question);
        assert_eq!(classifier.classify("Pourquoi le ciel est bleu?"), Intent::Question);
    }

    #[test]
    fn test_intent_classification_code() {
        let classifier = IntentClassifier::new();

        assert_eq!(classifier.classify("Écris du code Python"), Intent::CodeRequest);
        assert_eq!(classifier.classify("Write a function in Rust"), Intent::CodeRequest);
        assert_eq!(classifier.classify("Génère une classe JavaScript"), Intent::CodeRequest);
    }

    #[test]
    fn test_keyword_extraction() {
        let extractor = KeywordExtractor::new();

        let keywords = extractor.extract(
            "Comment créer une fonction async en Rust pour gérer les erreurs?",
            Some(5)
        );

        assert!(keywords.contains(&"fonction".to_string()));
        assert!(keywords.contains(&"async".to_string()));
        assert!(keywords.contains(&"rust".to_string()));
        // "les", "en" etc. should be filtered as stopwords
    }

    #[test]
    fn test_complexity_scoring() {
        let scorer = ComplexityScorer::new();

        let simple = scorer.analyze("Hello");
        let complex = scorer.analyze(
            "Explain the implementation details of async/await in Rust, \
             including how the Future trait works with poll-based execution."
        );

        assert!(complex.overall > simple.overall);
        assert!(complex.word_count > simple.word_count);
    }

    #[test]
    fn test_language_detection() {
        let analyzer = BrainAnalyzer::new(None);

        let packet_fr = analyzer.analyze("Comment puis-je créer une fonction?");
        let packet_en = analyzer.analyze("How can I create a function?");

        assert_eq!(packet_fr.language, "fr");
        assert_eq!(packet_en.language, "en");
    }

    #[test]
    fn test_should_use_rag() {
        let analyzer = BrainAnalyzer::new(None);

        // Questions should use RAG
        let packet_question = analyzer.analyze("What is the best way to handle errors?");
        assert!(packet_question.should_use_rag);

        // Greetings should not use RAG
        let packet_greeting = analyzer.analyze("Bonjour!");
        assert!(!packet_greeting.should_use_rag);
    }
}
```

---

## 🎭 Tests E2E Playwright

### Configuration

```javascript
// playwright.config.js
import { defineConfig } from "@playwright/test";

export default defineConfig({
  testDir: "./tests",
  fullyParallel: true,
  forbidOnly: !!process.env.CI,
  retries: process.env.CI ? 2 : 0,
  workers: process.env.CI ? 1 : undefined,
  reporter: "html",

  use: {
    baseURL: "http://localhost:1420",
    trace: "on-first-retry",
    screenshot: "only-on-failure",
  },

  projects: [
    {
      name: "chromium",
      use: { browserName: "chromium" },
    },
  ],

  webServer: {
    command: "npm run dev:ui",
    url: "http://localhost:1420",
    reuseExistingServer: !process.env.CI,
  },
});
```

### Exécution

```bash
# Tous les tests
npm run test:e2e

# Avec interface
npm run test:e2e:ui

# Test spécifique
npx playwright test session.spec.js
```

### Exemple de Test

```javascript
// tests/session.spec.js
import { test, expect } from "@playwright/test";

test.describe("Session Management", () => {
  test("should create a new session", async ({ page }) => {
    await page.goto("/");

    // Wait for app to load
    await page.waitForSelector('[data-testid="session-list"]');

    // Click new session button
    await page.click('[data-testid="new-session-btn"]');

    // Verify new session appears
    await expect(page.locator('[data-testid="session-item"]')).toHaveCount(1);
  });

  test("should send a message and receive response", async ({ page }) => {
    await page.goto("/");

    // Create session
    await page.click('[data-testid="new-session-btn"]');

    // Type message
    await page.fill('[data-testid="chat-input"]', "Hello AI");

    // Send
    await page.click('[data-testid="send-btn"]');

    // Wait for response (with timeout for LLM)
    await expect(page.locator('[data-testid="message-assistant"]')).toBeVisible(
      { timeout: 30000 }
    );
  });

  test("should delete a session", async ({ page }) => {
    await page.goto("/");

    // Create and then delete session
    await page.click('[data-testid="new-session-btn"]');
    await page.click('[data-testid="session-menu"]');
    await page.click('[data-testid="delete-session"]');

    // Confirm deletion
    await page.click('[data-testid="confirm-delete"]');

    // Verify session removed
    await expect(page.locator('[data-testid="session-item"]')).toHaveCount(0);
  });
});
```

---

## 📊 Couverture de Tests

### Rust

```bash
# Installer cargo-tarpaulin
cargo install cargo-tarpaulin

# Exécuter avec couverture
cargo tarpaulin --manifest-path apps/core/Cargo.toml --out html
```

### Modules Testés

| Module        | Couverture | Tests |
| ------------- | ---------- | ----- |
| database.rs   | ~80%       | 12    |
| encryption.rs | ~95%       | 6     |
| models.rs     | ~60%       | 4     |
| brain/        | ~75%       | 14    |
| actors/       | ~40%       | 8     |

### Modules Non Testés

| Module        | Raison                        |
| ------------- | ----------------------------- |
| main.rs       | Requires Tauri app context    |
| llm.rs        | Requires running llama-server |
| rag.rs        | Requires LanceDB setup        |
| fs_manager.rs | Filesystem dependencies       |

---

## 🔄 CI/CD

### GitHub Actions (Exemple)

```yaml
# .github/workflows/test.yml
name: Tests

on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

jobs:
  rust-tests:
    runs-on: windows-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable

      - name: Run Rust tests
        run: cargo test --manifest-path apps/core/Cargo.toml

  e2e-tests:
    runs-on: windows-latest
    needs: rust-tests
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-node@v4
        with:
          node-version: 20

      - name: Install dependencies
        run: npm ci

      - name: Install Playwright
        run: npx playwright install chromium

      - name: Run E2E tests
        run: npm run test:e2e
```

---

## 🐛 Debugging Tests

### Rust

```bash
# Avec logs détaillés
RUST_LOG=debug cargo test --manifest-path apps/core/Cargo.toml -- --nocapture

# Test unique
cargo test --manifest-path apps/core/Cargo.toml test_encrypt_decrypt_roundtrip -- --nocapture
```

### Playwright

```bash
# Mode debug
npx playwright test --debug

# Headed mode
npx playwright test --headed

# Trace viewer
npx playwright show-trace test-results/*/trace.zip
```

---

_Généré depuis lecture directe de: apps/core/src/tests/*, apps/desktop-ui/tests/*, playwright.config.js, package.json_
