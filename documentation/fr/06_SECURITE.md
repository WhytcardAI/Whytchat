# 🔐 Analyse de Sécurité - WhytChat V1

> Évaluation des mécanismes de sécurité et recommandations

---

## 📑 Table des Matières

1. [Vue d'Ensemble](#1-vue-densemble)
2. [Chiffrement des Données](#2-chiffrement-des-données)
3. [Authentification et Autorisation](#3-authentification-et-autorisation)
4. [Sécurité Réseau](#4-sécurité-réseau)
5. [Sécurité du Code](#5-sécurité-du-code)
6. [Vulnérabilités Identifiées](#6-vulnérabilités-identifiées)
7. [Recommandations](#7-recommandations)

---

## 1. Vue d'Ensemble

### Architecture de Sécurité

```
┌─────────────────────────────────────────────────────────────┐
│                     APPLICATION LOCALE                       │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  ┌─────────────┐    ┌─────────────┐    ┌─────────────┐     │
│  │  Frontend   │    │   Tauri     │    │   Backend   │     │
│  │  (React)    │◄──►│    IPC      │◄──►│   (Rust)    │     │
│  └─────────────┘    └─────────────┘    └─────────────┘     │
│                                               │              │
│                                               ▼              │
│  ┌─────────────────────────────────────────────────────┐   │
│  │                    DONNÉES LOCALES                   │   │
│  │  ┌───────────┐  ┌───────────┐  ┌───────────┐       │   │
│  │  │  SQLite   │  │  LanceDB  │  │   Files   │       │   │
│  │  │(Chiffré)  │  │(Vecteurs) │  │ (Bruts)   │       │   │
│  │  └───────────┘  └───────────┘  └───────────┘       │   │
│  └─────────────────────────────────────────────────────┘   │
│                                                              │
│  ┌─────────────────────────────────────────────────────┐   │
│  │               SERVICES LOCAUX                         │   │
│  │  ┌───────────┐                                       │   │
│  │  │llama-srv  │ ◄── Auth Token                        │   │
│  │  │:8080      │                                       │   │
│  │  └───────────┘                                       │   │
│  └─────────────────────────────────────────────────────┘   │
│                                                              │
└─────────────────────────────────────────────────────────────┘
              │
              │ AUCUNE connexion externe
              ▼
         🚫 Internet
```

### Principes de Sécurité

| Principe | Implémentation | Status |
|----------|----------------|--------|
| **Local-First** | Toutes données locales | ✅ |
| **Chiffrement au repos** | AES-256-GCM pour config | ✅ |
| **Auth interne** | Token llama-server | ✅ |
| **CSP** | Content Security Policy | ✅ |
| **Pas de telemetry** | Aucun tracking | ✅ |

---

## 2. Chiffrement des Données

### 2.1 Algorithme Utilisé

**AES-256-GCM** (Galois/Counter Mode)

| Paramètre | Valeur |
|-----------|--------|
| Algorithme | AES |
| Taille clé | 256 bits |
| Mode | GCM |
| Taille nonce | 96 bits (12 bytes) |
| Tag authentification | 128 bits |

### 2.2 Gestion des Clés

```rust
// Ordre de priorité pour obtenir la clé
fn get_encryption_key() -> Key {
    // 1. Cache mémoire (OnceLock)
    if let Some(key) = CACHED_KEY.get() {
        return key.clone();
    }
    
    // 2. Variable d'environnement
    if let Ok(key_hex) = env::var("ENCRYPTION_KEY") {
        return parse_hex_key(&key_hex);
    }
    
    // 3. Fichier .encryption_key
    if let Ok(key) = read_key_file() {
        return key;
    }
    
    // 4. Génération nouvelle clé
    let key = generate_random_key();
    save_key_file(&key);
    key
}
```

### 2.3 Données Chiffrées

| Donnée | Chiffrée | Localisation |
|--------|----------|--------------|
| `ModelConfig` | ✅ | `sessions.model_config` |
| Messages | ❌ | `messages.content` |
| Fichiers uploadés | ❌ | `data/files/` |
| Vecteurs | ❌ | `data/vectors/` |

### 2.4 Format de Stockage

```
Base64(nonce[12 bytes] || ciphertext || tag[16 bytes])
```

---

## 3. Authentification et Autorisation

### 3.1 Token llama-server

**Génération** :
```rust
let token = Uuid::new_v4().to_string();
env::set_var("LLAMA_AUTH_TOKEN", &token);
```

**Utilisation** :
```rust
client.post("http://localhost:8080/completion")
    .header("Authorization", format!("Bearer {}", token))
```

### 3.2 Rate Limiting

```rust
pub struct RateLimiter {
    window_size: Duration,   // 60 secondes
    max_requests: usize,     // 60 requêtes
    clients: HashMap<String, VecDeque<Instant>>,
}

impl RateLimiter {
    pub fn check_rate_limit(&mut self, client_id: &str) -> bool {
        let now = Instant::now();
        let window_start = now - self.window_size;
        
        let requests = self.clients
            .entry(client_id.to_string())
            .or_insert_with(VecDeque::new);
        
        // Purge old requests
        while let Some(&oldest) = requests.front() {
            if oldest < window_start {
                requests.pop_front();
            } else {
                break;
            }
        }
        
        if requests.len() >= self.max_requests {
            return false;  // Rate limited
        }
        
        requests.push_back(now);
        true
    }
}
```

---

## 4. Sécurité Réseau

### 4.1 Ports Exposés

| Port | Service | Binding | Accessible de |
|------|---------|---------|---------------|
| 1420 | Vite (dev) | localhost | Local seulement |
| 8080 | llama-server | localhost | Local seulement |

### 4.2 Content Security Policy

**Configuration Tauri** (`tauri.conf.json`) :

```json
{
  "app": {
    "security": {
      "csp": "default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'"
    }
  }
}
```

### 4.3 Permissions Tauri

```json
{
  "plugins": {
    "fs": {
      "scope": ["$APP/*", "$DATA/*"]
    },
    "shell": {
      "open": false,
      "execute": false
    }
  }
}
```

---

## 5. Sécurité du Code

### 5.1 Analyse des Dépendances

**Rust (Cargo.toml)** :

| Crate | Version | Audit |
|-------|---------|-------|
| `aes-gcm` | 0.10.3 | ✅ No known CVE |
| `sqlx` | 0.8 | ✅ No known CVE |
| `reqwest` | 0.12 | ✅ No known CVE |
| `tokio` | 1.x | ✅ No known CVE |

**JavaScript (package.json)** :

| Package | Audit |
|---------|-------|
| `react` | ✅ |
| `zustand` | ✅ |
| `@tauri-apps/*` | ✅ |

### 5.2 Blocs Unsafe

```rust
// rag.rs - Création NonZeroUsize
unsafe {
    NonZeroUsize::new_unchecked(num_threads)
}
// SAFETY: num_threads est toujours >= 1 (vérifié avant)
```

⚠️ **Recommandation** : Ajouter des commentaires `// SAFETY:` explicites.

### 5.3 Validation des Entrées

```rust
// models.rs
#[derive(Validate)]
pub struct ModelConfig {
    #[validate(length(min = 1))]
    pub model_id: String,

    #[validate(range(min = 0.0, max = 2.0))]
    pub temperature: f32,

    #[validate(length(max = 2000))]
    pub system_prompt: String,
}

// Usage
let config: ModelConfig = serde_json::from_str(&input)?;
config.validate()?;  // Retourne ValidationErrors si invalide
```

---

## 6. Vulnérabilités Identifiées

### 6.1 Haute Sévérité

| # | Vulnérabilité | Impact | Fichier |
|---|---------------|--------|---------|
| - | *Aucune identifiée* | - | - |

### 6.2 Moyenne Sévérité

| # | Vulnérabilité | Impact | Fichier |
|---|---------------|--------|---------|
| S-001 | Nonce potentiellement prévisible | Risque crypto | `encryption.rs` |
| S-002 | Messages non chiffrés | Données lisibles | `database.rs` |
| S-003 | Fichiers bruts sur disque | Données exposées | `fs_manager.rs` |

### 6.3 Basse Sévérité

| # | Vulnérabilité | Impact | Fichier |
|---|---------------|--------|---------|
| S-004 | Pas de purge rate_limiter | Memory leak | `rate_limiter.rs` |
| S-005 | Logs potentiellement sensibles | Fuite info | Multiple |

---

## 7. Recommandations

### 7.1 Priorité Haute 🔴

#### R-001 : Améliorer la génération du nonce

**Actuel** :
```rust
let nonce = Nonce::from_slice(&[0u8; 12]);  // ⚠️ Fixe!
```

**Recommandé** :
```rust
use rand::RngCore;

let mut nonce_bytes = [0u8; 12];
rand::thread_rng().fill_bytes(&mut nonce_bytes);
let nonce = Nonce::from_slice(&nonce_bytes);

// Stocker nonce avec ciphertext
let mut output = nonce_bytes.to_vec();
output.extend_from_slice(&ciphertext);
```

#### R-002 : Chiffrer les messages

**Recommandé** : Étendre le chiffrement aux messages sensibles.

```rust
// Option 1: Chiffrement sélectif
if message.is_sensitive() {
    let encrypted = encrypt(&message.content)?;
    // Store encrypted
}

// Option 2: Chiffrement systématique
let encrypted = encrypt(&message.content)?;
```

### 7.2 Priorité Moyenne ⚠️

#### R-003 : Chiffrer les fichiers uploadés

```rust
pub async fn save_file(content: &[u8], filename: &str) -> Result<PathBuf> {
    let encrypted = encrypt(content)?;
    let path = get_files_path().join(filename);
    fs::write(&path, encrypted)?;
    Ok(path)
}
```

#### R-004 : Purge automatique rate_limiter

```rust
impl RateLimiter {
    pub fn cleanup_stale_clients(&mut self) {
        let cutoff = Instant::now() - Duration::from_secs(300);
        self.clients.retain(|_, requests| {
            requests.back().map_or(false, |&t| t > cutoff)
        });
    }
}
```

### 7.3 Priorité Basse ℹ️

#### R-005 : Audit des logs

```rust
// Éviter
info!("Processing message: {}", message.content);

// Préférer
info!("Processing message for session: {}", session_id);
debug!("Message length: {}", message.content.len());
```

#### R-006 : Rotation des clés

Implémenter une rotation périodique de la clé de chiffrement avec migration des données.

---

## 📊 Matrice de Risques

| Risque | Probabilité | Impact | Score |
|--------|-------------|--------|-------|
| Clé exposée | Faible | Critique | 🟡 Moyen |
| Messages lisibles | Moyenne | Moyen | 🟡 Moyen |
| Fichiers exposés | Moyenne | Moyen | 🟡 Moyen |
| Rate limit DoS | Faible | Faible | 🟢 Faible |

---

## 📚 Voir Aussi

- [03_BACKEND_RUST.md](03_BACKEND_RUST.md) - Détails encryption.rs
- [07_IRREGULARITES.md](07_IRREGULARITES.md) - Problèmes identifiés
- [08_RECOMMANDATIONS.md](08_RECOMMANDATIONS.md) - Actions complètes

---

_Document généré le 27 novembre 2025_
