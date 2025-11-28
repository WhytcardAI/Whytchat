# 🔐 Sécurité - WhytChat V1

> Chiffrement, gestion des secrets et protection des données

---

## 🎯 Vue d'Ensemble

WhytChat implémente plusieurs couches de sécurité :

1. **Chiffrement AES-256-GCM** - Configuration sensible chiffrée en DB
2. **Clé persistante sécurisée** - Générée une fois, stockée localement
3. **Rate Limiting** - Protection contre les abus (20 req/min)
4. **Isolation locale** - Aucune donnée envoyée vers l'extérieur

---

## 🔒 Chiffrement AES-256-GCM

### Algorithme

| Paramètre    | Valeur                         |
| ------------ | ------------------------------ |
| Algorithme   | AES-256-GCM                    |
| Taille clé   | 256 bits (32 octets)           |
| Taille nonce | 96 bits (12 octets)            |
| Mode         | Authenticated Encryption (GCM) |
| Bibliothèque | `aes-gcm` 0.10                 |

### Génération du Nonce

```rust
// ✅ CORRECT - Nonce aléatoire à chaque chiffrement
let mut rng = rand::thread_rng();
let mut nonce_bytes = [0u8; NONCE_SIZE];  // NONCE_SIZE = 12
rng.fill(&mut nonce_bytes);
let nonce = Nonce::from_slice(&nonce_bytes);
```

> **Note importante** : Le nonce est généré aléatoirement à chaque appel de `encrypt()`. Ceci garantit qu'un même message chiffré deux fois produira deux ciphertexts différents.

### Format des Données Chiffrées

```
┌──────────────────┬─────────────────────────────┐
│  Nonce (12 bytes)│     Ciphertext (variable)   │
└──────────────────┴─────────────────────────────┘
                    │
                    ▼
           Base64 encode
                    │
                    ▼
           String stockée
```

### Fonctions de Chiffrement

```rust
/// Chiffre des données avec AES-256-GCM
/// Nonce aléatoire généré automatiquement
pub fn encrypt(data: &[u8]) -> Result<String, String>

/// Déchiffre des données AES-256-GCM
/// Extrait le nonce des 12 premiers octets
pub fn decrypt(encrypted_base64: &str) -> Result<Vec<u8>, String>
```

### Exemple d'Utilisation

```rust
// Chiffrement
let secret_data = b"API key or sensitive config";
let encrypted = encrypt(secret_data)?;
// Résultat: "base64EncodedNonceAndCiphertext..."

// Déchiffrement
let decrypted = decrypt(&encrypted)?;
assert_eq!(decrypted, secret_data);
```

---

## 🔑 Gestion des Clés

### Stratégie de Clé

```rust
/// Ordre de priorité pour la clé :
/// 1. Cache mémoire (OnceLock)
/// 2. Variable d'environnement ENCRYPTION_KEY (tests/CI)
/// 3. Fichier local data/.encryption_key
/// 4. Génération nouvelle clé + sauvegarde
```

### Génération de Clé

```rust
fn generate_secure_key() -> [u8; 32] {
    let mut key = [0u8; 32];
    rand::thread_rng().fill(&mut key);  // CSPRNG
    key
}
```

### Stockage de la Clé

```
data/
└── .encryption_key    ← Fichier clé (Base64, 600 perms sur Unix)
```

| Attribut         | Valeur                     |
| ---------------- | -------------------------- |
| Emplacement      | `data/.encryption_key`     |
| Format           | Base64 (44 caractères)     |
| Permissions Unix | `0o600` (rw-------)        |
| Permissions Win  | Héritage du dossier parent |

### Cache Mémoire

```rust
static ENCRYPTION_KEY: OnceLock<[u8; 32]> = OnceLock::new();
```

La clé est chargée **une seule fois** au démarrage et cachée en mémoire pour éviter les lectures fichier répétées.

---

## ⏱️ Rate Limiting

### Configuration

| Paramètre   | Valeur         |
| ----------- | -------------- |
| Limite      | 20 requêtes    |
| Fenêtre     | 60 secondes    |
| Algorithme  | Sliding Window |
| Granularité | Par session ID |

### Implémentation

```rust
pub struct RateLimiter {
    requests: HashMap<String, Vec<Instant>>,
    limit: usize,      // 20
    window: Duration,  // 60 secondes
}

impl RateLimiter {
    pub fn check(&mut self, id: &str) -> bool {
        let now = Instant::now();
        let window_start = now - self.window;

        let client_requests = self.requests
            .entry(id.to_string())
            .or_default();

        // Nettoyer les anciennes requêtes
        client_requests.retain(|&ts| ts > window_start);

        if client_requests.len() < self.limit {
            client_requests.push(now);
            true  // Requête autorisée
        } else {
            false // Limite atteinte
        }
    }
}
```

### Utilisation dans main.rs

```rust
async fn check_rate_limit_and_get_resources(
    state: &State<'_, AppState>,
    session_id: Uuid,
) -> Result<(SqlitePool, SupervisorHandle), String> {
    let init_state = get_initialized_state(state)?;

    // Vérifier rate limit
    let mut rate_limiter = init_state.rate_limiter.lock().await;
    if !rate_limiter.check(&session_id.to_string()) {
        return Err("Rate limit exceeded. Please wait before sending more messages.".to_string());
    }

    Ok((init_state.pool.clone(), init_state.supervisor.clone()))
}
```

---

## 🛡️ Données Protégées

### Ce qui est chiffré

| Donnée      | Chiffré | Raison                            |
| ----------- | ------- | --------------------------------- |
| ModelConfig | ✅      | Chemins et paramètres sensibles   |
| Messages    | ❌      | Stockés en local uniquement       |
| Sessions    | ❌      | Métadonnées non sensibles         |
| Embeddings  | ❌      | Données dérivées, non reversibles |

### ModelConfig (Chiffré)

```rust
pub struct ModelConfig {
    pub model_path: String,     // Chemin vers le modèle GGUF
    pub n_ctx: u32,             // Context length
    pub n_gpu_layers: i32,      // Layers GPU
    pub temperature: f32,       // Température LLM
}
```

```rust
// Sauvegarde chiffrée
pub async fn save_model_config(
    pool: &SqlitePool,
    config: &ModelConfig
) -> Result<(), AppError> {
    let json = serde_json::to_vec(config)?;
    let encrypted = encrypt(&json)?;

    sqlx::query!(
        "INSERT OR REPLACE INTO config (key, value) VALUES ('model_config', ?)",
        encrypted
    )
    .execute(pool)
    .await?;

    Ok(())
}

// Récupération déchiffrée
pub async fn get_model_config(
    pool: &SqlitePool
) -> Result<Option<ModelConfig>, AppError> {
    let row = sqlx::query!(
        "SELECT value FROM config WHERE key = 'model_config'"
    )
    .fetch_optional(pool)
    .await?;

    if let Some(row) = row {
        let decrypted = decrypt(&row.value)?;
        let config = serde_json::from_slice(&decrypted)?;
        Ok(Some(config))
    } else {
        Ok(None)
    }
}
```

---

## 🌐 Isolation Réseau

### Politique Locale

WhytChat fonctionne **100% localement** :

```
┌──────────────────────────────────────────────────────────────┐
│                        WhytChat                              │
│                                                              │
│  ┌─────────┐    ┌─────────────┐    ┌──────────────────┐     │
│  │ Frontend│◄──►│   Backend   │◄──►│  llama-server    │     │
│  │ (React) │    │   (Rust)    │    │ (127.0.0.1:8765) │     │
│  └─────────┘    └─────────────┘    └──────────────────┘     │
│        │              │                    │                 │
│        ▼              ▼                    ▼                 │
│  ┌─────────┐    ┌─────────────┐    ┌──────────────────┐     │
│  │localStorage│ │ SQLite DB   │    │   Modèle GGUF    │     │
│  └─────────┘    └─────────────┘    └──────────────────┘     │
│                                                              │
└──────────────────────────────────────────────────────────────┘
                         │
                         ▼
                   ❌ Aucune connexion
                      vers Internet
```

### Exceptions Contrôlées

| Action                | Connexion Externe | Contrôle Utilisateur |
| --------------------- | ----------------- | -------------------- |
| Téléchargement modèle | ✅ HuggingFace    | Explicite (bouton)   |
| Chat avec LLM         | ❌                | -                    |
| Recherche RAG         | ❌                | -                    |
| Stockage messages     | ❌                | -                    |
| Embeddings            | ❌                | -                    |

---

## 🔐 Sécurité Tauri

### Content Security Policy (CSP)

```json
// tauri.conf.json
{
  "app": {
    "security": {
      "csp": "default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; img-src 'self' data: asset: https://asset.localhost"
    }
  }
}
```

### Capabilities

```json
// capabilities/default.json
{
  "identifier": "default",
  "permissions": ["core:default", "shell:allow-open", "shell:allow-execute"]
}
```

### Permissions Minimales

- ✅ `shell:allow-open` - Ouvrir liens externes
- ✅ `shell:allow-execute` - Exécuter llama-server
- ❌ Pas d'accès réseau arbitraire
- ❌ Pas d'accès fichiers arbitraires

---

## ⚠️ Points d'Attention Sécurité

### TODO Identifiés

1. **Sel PBKDF2 fixe** (non utilisé actuellement)
   - La fonction `derive_key` n'est pas utilisée en production
   - Clé générée directement avec CSPRNG

2. **Permissions Windows**
   - Le fichier `.encryption_key` n'a pas de protection spéciale sur Windows
   - L'utilisateur doit sécuriser le dossier `data/`

3. **Clé en mémoire**
   - La clé reste en mémoire pendant l'exécution
   - Acceptable pour une application desktop

### Recommandations

```markdown
1. ✅ Ne jamais commiter data/.encryption_key
2. ✅ Ajouter data/ au .gitignore
3. ✅ Sauvegarder .encryption_key séparément pour récupération
4. ⚠️ Sur Windows, vérifier les permissions du dossier data/
```

---

## 🧪 Tests de Sécurité

### Test Chiffrement

```rust
#[test]
fn test_encryption_decryption() {
    temp_env::with_var(
        "ENCRYPTION_KEY",
        Some("01234567890123456789012345678901"),
        || {
            let data = b"Sensitive Data";
            let encrypted = encrypt(data).expect("Encryption failed");
            let decrypted = decrypt(&encrypted).expect("Decryption failed");
            assert_eq!(data, &decrypted[..]);
        },
    );
}
```

### Test Rate Limiter

```rust
#[test]
fn test_rate_limiter_allows_requests_within_limit() {
    let mut limiter = RateLimiter::new(5, Duration::from_secs(1));
    for _ in 0..5 {
        assert!(limiter.check("client1"));
    }
    assert!(!limiter.check("client1"));  // 6ème refusée
}

#[test]
fn test_rate_limiter_resets_after_window() {
    let mut limiter = RateLimiter::new(2, Duration::from_millis(50));
    assert!(limiter.check("client2"));
    assert!(limiter.check("client2"));
    assert!(!limiter.check("client2"));  // Refusée

    thread::sleep(Duration::from_millis(60));  // Attendre fin fenêtre

    assert!(limiter.check("client2"));  // Acceptée à nouveau
}
```

---

## 📋 Checklist Sécurité

| Élément                     | Status | Notes                |
| --------------------------- | ------ | -------------------- |
| Chiffrement AES-256-GCM     | ✅     | Nonce aléatoire      |
| Clé 256 bits CSPRNG         | ✅     | rand::thread_rng     |
| Stockage clé sécurisé       | ✅     | 600 perms Unix       |
| Rate limiting               | ✅     | 20 req/min/session   |
| CSP configuré               | ✅     | Strict               |
| Pas de secrets hardcodés    | ✅     | Clé générée/fichier  |
| Isolation locale            | ✅     | 127.0.0.1 uniquement |
| Logs sans données sensibles | ✅     | Pas de tokens loggés |

---

_Généré depuis lecture directe de: encryption.rs, rate_limiter.rs, main.rs, tauri.conf.json, capabilities/default.json_
