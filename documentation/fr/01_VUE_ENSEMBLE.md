# 🔭 Vue d'Ensemble - WhytChat V1

> Application de chat IA locale avec RAG, construite avec Tauri 2.0

---

## 📋 Informations Projet

| Champ       | Valeur                        |
| ----------- | ----------------------------- |
| Nom         | whytchat-core                 |
| Version     | 1.0.0                         |
| Identifiant | com.whytcard.whytchat-v1      |
| Rust        | 1.80.0+ (rust-toolchain.toml) |
| Tauri       | 2.0.0-rc                      |

---

## 🏗️ Structure Monorepo

```
WhytChat_V1/
├── apps/
│   ├── core/                    # Backend Rust (Tauri)
│   │   ├── src/
│   │   │   ├── main.rs          # Point d'entrée, 22 commandes Tauri
│   │   │   ├── actors/          # Système d'acteurs (LLM, RAG, Supervisor)
│   │   │   ├── brain/           # Analyse pré-LLM (intent, keywords, complexity)
│   │   │   ├── database.rs      # CRUD SQLite
│   │   │   ├── encryption.rs    # AES-256-GCM
│   │   │   ├── error.rs         # AppError centralisé
│   │   │   ├── fs_manager.rs    # PortablePathManager
│   │   │   ├── models.rs        # Structs (Session, Message, etc.)
│   │   │   ├── rate_limiter.rs  # Limite 20 req/min par session
│   │   │   └── text_extract.rs  # PDF, DOCX, TXT, CSV, JSON
│   │   ├── migrations/          # Schéma SQLite
│   │   └── Cargo.toml
│   │
│   └── desktop-ui/              # Frontend React
│       ├── src/
│       │   ├── App.jsx          # Point d'entrée
│       │   ├── components/      # Composants React
│       │   ├── hooks/           # useChatStream, etc.
│       │   ├── store/           # Zustand (appStore.js)
│       │   └── lib/             # Utilitaires
│       └── package.json
│
├── data/                        # Données runtime
│   ├── db/                      # whytchat.sqlite
│   ├── files/                   # Fichiers uploadés
│   ├── models/                  # Modèles GGUF + embeddings
│   └── vectors/                 # LanceDB (knowledge_base.lance)
│
└── package.json                 # Monorepo root
```

---

## 🔧 Stack Technique

### Backend (apps/core)

| Dépendance  | Version  | Rôle                       |
| ----------- | -------- | -------------------------- |
| tauri       | 2.0.0-rc | Framework desktop          |
| sqlx        | 0.8      | SQLite async               |
| lancedb     | 0.10     | Base vectorielle           |
| fastembed   | 4        | Embeddings AllMiniLML6V2   |
| aes-gcm     | 0.10.3   | Encryption configurations  |
| reqwest     | 0.12     | HTTP client (llama-server) |
| tokio       | 1        | Runtime async              |
| tracing     | 0.1      | Logging structuré          |
| pdf-extract | 0.7      | Extraction PDF             |
| docx-rs     | 0.4      | Extraction DOCX            |

### Frontend (apps/desktop-ui)

| Dépendance      | Version | Rôle                 |
| --------------- | ------- | -------------------- |
| react           | 18.3.1  | UI Framework         |
| vite            | 5.4.1   | Build tool           |
| zustand         | 5.0.0   | State management     |
| tailwindcss     | 3.4.10  | CSS utility          |
| @tauri-apps/api | 2.0.0   | Bridge Tauri         |
| i18next         | 25.6.3  | Internationalisation |
| lucide-react    | 0.454.0 | Icons                |
| react-hot-toast | 2.6.0   | Notifications        |

---

## 📊 Constantes du Projet

```rust
// main.rs
const DEFAULT_MODEL_FILENAME: &str = "default-model.gguf";
const LLAMA_SERVER_URL: &str = "https://github.com/ggml-org/llama.cpp/releases/download/b4154/llama-b4154-bin-win-avx2-x64.zip";
const MODEL_URL: &str = "https://huggingface.co/Qwen/Qwen2.5-Coder-7B-Instruct-GGUF/resolve/main/qwen2.5-coder-7b-instruct-q4_k_m.gguf";
const MIN_MODEL_SIZE_BYTES: u64 = 3 * 1024 * 1024 * 1024; // 3 GB minimum
```

---

## 🚀 Modèle LLM

| Propriété    | Valeur                                   |
| ------------ | ---------------------------------------- |
| Modèle       | Qwen2.5-Coder-7B-Instruct                |
| Quantization | Q4_K_M                                   |
| Taille       | ~4.7 GB                                  |
| Format       | GGUF                                     |
| Contexte     | 8192 tokens                              |
| Serveur      | llama-server (llama.cpp b4154)           |
| Port         | 8080                                     |
| Template     | ChatML (`<\|im_start\|>...<\|im_end\|>`) |

---

## 🔐 Sécurité

- **Encryption** : AES-256-GCM pour `model_config` des sessions
- **Nonce** : Aléatoire 12 bytes par encryption (rand::thread_rng)
- **Clé** : 32 bytes stockée dans `data/.encryption_key`
- **Auth** : `LLAMA_AUTH_TOKEN` généré au démarrage (UUID)
- **Rate Limit** : 20 requêtes/minute par session

---

## 📝 Tests

```bash
# 44 tests unitaires Rust
cargo test --manifest-path apps/core/Cargo.toml

# Tests passés:
# - brain::* (14 tests) - Intent, keywords, complexity
# - encryption::* (1 test)
# - rate_limiter::* (2 tests)
# - text_extract::* (8 tests)
```

---

_Généré depuis lecture directe de: main.rs, Cargo.toml, package.json, tauri.conf.json_
