# 💡 Recommandations - WhytChat V1

> Actions suggérées pour améliorer le codebase

---

## 📑 Table des Matières

1. [Priorité Haute](#1-priorité-haute)
2. [Priorité Moyenne](#2-priorité-moyenne)
3. [Priorité Basse](#3-priorité-basse)
4. [Roadmap Suggérée](#4-roadmap-suggérée)

---

## 1. Priorité Haute 🔴

### R-001 : Corriger les Tests Cassés

**Problème** : 4 fichiers de tests ne compilent pas (IRR-001 à IRR-004).

**Actions** :

1. **Exposer le module supervisor** :
```rust
// apps/core/src/actors/mod.rs
pub mod supervisor;
```

2. **Mettre à jour les signatures de test** :
```rust
// Avant
supervisor.process_message(session_id, message).await;

// Après
supervisor.process_message(session_id, message, None).await;
```

3. **Aligner les tests avec ContextPacket réel** :
```rust
// Supprimer les assertions sur champs inexistants
// assert!(packet.suggestions.is_empty()); // ❌
```

**Effort** : 2-4 heures

---

### R-002 : Implémenter DI pour SupervisorHandle

**Problème** : Impossible de tester le Supervisor avec des mocks.

**Solution** :

```rust
impl SupervisorHandle {
    // Constructeur existant
    pub fn new(pool: SqlitePool) -> Self { ... }
    
    // Nouveau constructeur pour tests
    pub fn new_with_actors(
        pool: SqlitePool,
        brain: Box<dyn BrainAnalyzer>,
        rag: Box<dyn RagActor>,
        llm: Box<dyn LlmActor>,
    ) -> Self { ... }
}
```

**Effort** : 4-8 heures

---

### R-003 : Améliorer la Sécurité du Nonce

**Problème** : Nonce fixe dans `encryption.rs` (IRR-008).

**Solution** :

```rust
use rand::RngCore;

pub fn encrypt(plaintext: &[u8]) -> Result<Vec<u8>, AppError> {
    let key = get_encryption_key()?;
    let cipher = Aes256Gcm::new(&key);
    
    // Générer nonce aléatoire
    let mut nonce_bytes = [0u8; 12];
    rand::thread_rng().fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);
    
    let ciphertext = cipher.encrypt(nonce, plaintext)?;
    
    // Stocker nonce + ciphertext
    let mut output = nonce_bytes.to_vec();
    output.extend_from_slice(&ciphertext);
    Ok(output)
}

pub fn decrypt(data: &[u8]) -> Result<Vec<u8>, AppError> {
    if data.len() < 12 {
        return Err(AppError::Crypto("Data too short".into()));
    }
    
    let (nonce_bytes, ciphertext) = data.split_at(12);
    let nonce = Nonce::from_slice(nonce_bytes);
    
    let key = get_encryption_key()?;
    let cipher = Aes256Gcm::new(&key);
    
    cipher.decrypt(nonce, ciphertext)
        .map_err(|e| AppError::Crypto(e.to_string()))
}
```

**Effort** : 2-3 heures

---

### R-004 : Supporter .doc ou le Retirer

**Problème** : `.doc` listé mais non supporté (IRR-005).

**Option A - Ajouter le support** :
```toml
# Cargo.toml
[dependencies]
cfb = "0.7"  # Pour parser les fichiers OLE
```

**Option B - Retirer proprement** :
```rust
match extension.as_str() {
    "doc" => {
        return Err(AppError::Validation(
            "Old .doc format not supported. Please convert to .docx".into()
        ));
    }
    // ...
}
```

**Effort** : 1-4 heures selon option

---

## 2. Priorité Moyenne ⚠️

### R-005 : Normaliser les Paramètres Tauri

**Problème** : Double params snake_case/camelCase (IRR-006).

**Solution** : Choisir UN style et s'y tenir.

```javascript
// ❌ Avant
await invoke('create_session', {
  session_id: id,
  sessionId: id,  // Doublon
});

// ✅ Après (snake_case côté Rust)
await invoke('create_session', {
  session_id: id,
});
```

**Effort** : 4-6 heures (refactoring global)

---

### R-006 : Réactiver ThinkingBubble

**Problème** : Feature désactivée (IRR-007).

**Solution** :

```jsx
// ChatInterface.jsx
const { isThinking, thinkingSteps } = useAppStore();

return (
  <div>
    <MessageList messages={messages} />
    
    {/* Réactiver avec setting */}
    {isThinking && showThinking && (
      <ThinkingBubble steps={thinkingSteps} />
    )}
    
    <ChatInput onSend={handleSend} />
  </div>
);
```

**Bonus** : Ajouter un toggle dans les settings.

**Effort** : 1-2 heures

---

### R-007 : Implémenter les Filtres RAG

**Problème** : `search_with_filters` ignore les filtres (IRR-010).

**Solution** :

```rust
pub async fn search_with_filters(
    &self,
    query: String,
    file_ids: Vec<String>,
) -> Result<Vec<SearchResult>, AppError> {
    let query_vec = self.embed(&query).await?;
    
    // Construire le filtre LanceDB
    let filter = if file_ids.is_empty() {
        None
    } else {
        Some(format!(
            "file_id IN ({})",
            file_ids.iter()
                .map(|id| format!("'{}'", id))
                .collect::<Vec<_>>()
                .join(", ")
        ))
    };
    
    let mut query_builder = self.table.search(&query_vec);
    
    if let Some(f) = filter {
        query_builder = query_builder.filter(f);
    }
    
    query_builder.limit(5).execute().await
}
```

**Effort** : 2-4 heures

---

### R-008 : Améliorer le Parser Markdown

**Problème** : Parser custom limité (pas de tables, images, liens).

**Option A - Utiliser react-markdown** :
```jsx
import ReactMarkdown from 'react-markdown';
import remarkGfm from 'remark-gfm';

<ReactMarkdown remarkPlugins={[remarkGfm]}>
  {content}
</ReactMarkdown>
```

**Option B - Étendre le parser custom** pour garder le contrôle.

**Effort** : 2-6 heures selon option

---

## 3. Priorité Basse ℹ️

### R-009 : Cleanup Rate Limiter

**Problème** : Pas de purge des clients inactifs (IRR-012).

```rust
impl RateLimiter {
    pub fn cleanup_stale(&mut self) {
        let cutoff = Instant::now() - Duration::from_secs(300);
        self.clients.retain(|_, reqs| {
            reqs.back().map_or(false, |&t| t > cutoff)
        });
    }
}

// Appeler périodiquement
tokio::spawn(async move {
    loop {
        tokio::time::sleep(Duration::from_secs(60)).await;
        rate_limiter.lock().await.cleanup_stale();
    }
});
```

**Effort** : 1 heure

---

### R-010 : Standardiser les Messages d'Erreur

**Problème** : Mix FR/EN (IRR-018).

**Solution** : Utiliser i18n côté backend ou standardiser en anglais.

```rust
// Fichier errors.rs
pub mod messages {
    pub const NOT_INITIALIZED: &str = "Application not initialized";
    pub const DB_ERROR: &str = "Database error";
    // ...
}
```

**Effort** : 2-3 heures

---

### R-011 : Unifier le Style JavaScript

**Problème** : Mix `function()` et arrow functions.

**Solution** : Configurer ESLint pour enforcer arrow functions.

```json
// .eslintrc
{
  "rules": {
    "prefer-arrow-callback": "error",
    "arrow-body-style": ["error", "as-needed"]
  }
}
```

**Effort** : 1-2 heures

---

### R-012 : Documenter les Blocs Unsafe

**Problème** : Unsafe sans commentaires SAFETY (IRR-009).

```rust
// ✅ Bon
// SAFETY: num_threads est toujours >= 1 car vérifié par
// std::thread::available_parallelism() qui retourne au minimum 1
unsafe {
    NonZeroUsize::new_unchecked(num_threads)
}
```

**Effort** : 1 heure

---

## 4. Roadmap Suggérée

### Sprint 1 (Semaine 1) - Critique

| # | Tâche | Effort | Assigné |
|---|-------|--------|---------|
| 1 | R-001 : Corriger tests | 4h | |
| 2 | R-003 : Fix nonce | 3h | |
| 3 | R-004 : Support .doc | 2h | |

### Sprint 2 (Semaine 2) - Important

| # | Tâche | Effort | Assigné |
|---|-------|--------|---------|
| 4 | R-002 : DI Supervisor | 8h | |
| 5 | R-005 : Normaliser params | 6h | |
| 6 | R-006 : ThinkingBubble | 2h | |

### Sprint 3 (Semaine 3) - Amélioration

| # | Tâche | Effort | Assigné |
|---|-------|--------|---------|
| 7 | R-007 : Filtres RAG | 4h | |
| 8 | R-008 : Markdown | 4h | |
| 9 | R-010 : Messages erreur | 3h | |

### Sprint 4 (Semaine 4) - Qualité

| # | Tâche | Effort | Assigné |
|---|-------|--------|---------|
| 10 | R-009 : Rate limiter | 1h | |
| 11 | R-011 : Style JS | 2h | |
| 12 | R-012 : Doc unsafe | 1h | |

---

## 📊 Estimation Totale

| Priorité | Tâches | Heures |
|----------|--------|--------|
| 🔴 Haute | 4 | ~15h |
| ⚠️ Moyenne | 4 | ~16h |
| ℹ️ Basse | 4 | ~7h |
| **Total** | **12** | **~38h** |

---

## 📚 Voir Aussi

- [07_IRREGULARITES.md](07_IRREGULARITES.md) - Détails des problèmes
- [06_SECURITE.md](06_SECURITE.md) - Recommandations sécurité

---

_Document généré le 27 novembre 2025_
