# 🔌 Commandes Tauri - WhytChat V1

> Liste complète des 22 commandes IPC entre frontend et backend

---

## 📋 Vue d'Ensemble

Les commandes Tauri sont définies dans `apps/core/src/main.rs` dans le bloc `generate_handler!`.

```rust
// main.rs - ligne ~1500
.invoke_handler(tauri::generate_handler![
    run_preflight,
    download_model,
    create_session,
    get_all_sessions,
    get_session,
    delete_session,
    rename_session,
    update_session_order,
    get_messages,
    delete_message,
    debug_chat,
    ingest_file,
    upload_library_file,
    get_library_files,
    get_file_content,
    delete_file,
    create_folder,
    get_all_folders,
    delete_folder,
    link_file_to_session,
    get_session_files,
    run_diagnostic_test,
])
```

---

## 🚀 Commandes Système

### run_preflight

Exécute les vérifications au démarrage de l'application.

```typescript
// Frontend
await invoke("run_preflight");
```

```rust
// Backend
#[tauri::command]
async fn run_preflight(
    state: State<'_, AppState>,
    app_handle: AppHandle,
) -> Result<(), String>
```

**Vérifie :**

- Présence du modèle LLM
- Intégrité de la base de données
- Permissions fichiers
- État du llama-server

---

### download_model

Télécharge le modèle LLM depuis HuggingFace.

```typescript
// Frontend
await invoke("download_model");

// Écouter la progression
await listen("download-progress", (event) => {
  console.log(`Progress: ${event.payload}%`);
});

await listen("download-status", (event) => {
  console.log(`Status: ${event.payload.step} - ${event.payload.detail}`);
});
```

```rust
// Backend
#[tauri::command]
async fn download_model(
    state: State<'_, AppState>,
    window: Window,
) -> Result<(), String>
```

**Events émis :**

- `download-progress` : `u64` (0-100)
- `download-status` : `{ step: String, detail: String }`

---

### run_diagnostic_test

Exécute un test de diagnostic spécifique.

```typescript
// Frontend
const result = await invoke("run_diagnostic_test", {
  testId: "llm_connection",
  category: "system",
});
```

```rust
// Backend
#[tauri::command]
async fn run_diagnostic_test(
    state: State<'_, AppState>,
    test_id: String,
    category: String,
) -> Result<DiagnosticResult, String>
```

---

## 📁 Commandes Sessions

### create_session

Crée une nouvelle session de chat.

```typescript
// Frontend
const session = await invoke("create_session", { name: "New Chat" });
```

```rust
// Backend
#[tauri::command]
async fn create_session(
    state: State<'_, AppState>,
    name: String,
) -> Result<Session, String>
```

**Retourne :**

```json
{
  "id": "uuid",
  "name": "New Chat",
  "created_at": "2024-11-28T00:00:00Z",
  "updated_at": "2024-11-28T00:00:00Z",
  "is_pinned": false,
  "sort_order": null
}
```

---

### get_all_sessions

Récupère toutes les sessions.

```typescript
// Frontend
const sessions = await invoke("get_all_sessions");
```

```rust
// Backend
#[tauri::command]
async fn get_all_sessions(
    state: State<'_, AppState>,
) -> Result<Vec<Session>, String>
```

---

### get_session

Récupère une session par ID.

```typescript
// Frontend
const session = await invoke("get_session", { sessionId: "uuid" });
```

```rust
// Backend
#[tauri::command]
async fn get_session(
    state: State<'_, AppState>,
    session_id: String,
) -> Result<Option<Session>, String>
```

---

### delete_session

Supprime une session et tous ses messages.

```typescript
// Frontend
await invoke("delete_session", { sessionId: "uuid" });
```

```rust
// Backend
#[tauri::command]
async fn delete_session(
    state: State<'_, AppState>,
    session_id: String,
) -> Result<(), String>
```

---

### rename_session

Renomme une session.

```typescript
// Frontend
await invoke("rename_session", {
  sessionId: "uuid",
  newName: "Renamed Chat",
});
```

```rust
// Backend
#[tauri::command]
async fn rename_session(
    state: State<'_, AppState>,
    session_id: String,
    new_name: String,
) -> Result<(), String>
```

---

### update_session_order

Met à jour l'ordre de tri d'une session.

```typescript
// Frontend
await invoke("update_session_order", {
  sessionId: "uuid",
  sortOrder: 1,
  isPinned: true,
});
```

```rust
// Backend
#[tauri::command]
async fn update_session_order(
    state: State<'_, AppState>,
    session_id: String,
    sort_order: Option<i64>,
    is_pinned: bool,
) -> Result<(), String>
```

---

## 💬 Commandes Messages

### get_messages

Récupère tous les messages d'une session.

```typescript
// Frontend
const messages = await invoke("get_messages", { sessionId: "uuid" });
```

```rust
// Backend
#[tauri::command]
async fn get_messages(
    state: State<'_, AppState>,
    session_id: String,
) -> Result<Vec<Message>, String>
```

**Retourne :**

```json
[
  {
    "id": "uuid",
    "session_id": "uuid",
    "role": "user",
    "content": "Hello!",
    "created_at": "2024-11-28T00:00:00Z",
    "tokens": null
  }
]
```

---

### delete_message

Supprime un message spécifique.

```typescript
// Frontend
await invoke("delete_message", { messageId: "uuid" });
```

```rust
// Backend
#[tauri::command]
async fn delete_message(
    state: State<'_, AppState>,
    message_id: String,
) -> Result<(), String>
```

---

### debug_chat

Commande principale de chat avec streaming.

```typescript
// Frontend
await invoke("debug_chat", {
  sessionId: "uuid",
  content: "Hello AI!",
});

// Écouter les tokens
await listen("chat-token", (event) => {
  appendToken(event.payload);
});

// Écouter les étapes de réflexion
await listen("thinking-step", (event) => {
  showThinkingStep(event.payload);
});
```

```rust
// Backend
#[tauri::command]
async fn debug_chat(
    state: State<'_, AppState>,
    window: Window,
    session_id: String,
    content: String,
) -> Result<String, String>
```

**Events émis :**

- `chat-token` : `String` (chaque token du LLM)
- `thinking-step` : `String` (étapes d'analyse)
- `brain-analysis` : `ContextPacket` (analyse Brain complète)

---

## 📄 Commandes Fichiers (Bibliothèque)

### upload_library_file

Upload un fichier dans la bibliothèque globale.

```typescript
// Frontend
const file = await invoke("upload_library_file", {
  name: "document.pdf",
  fileType: "pdf",
  content: base64Content,
  folderId: "uuid", // optionnel
});
```

```rust
// Backend
#[tauri::command]
async fn upload_library_file(
    state: State<'_, AppState>,
    name: String,
    file_type: String,
    content: Vec<u8>,
    folder_id: Option<String>,
) -> Result<LibraryFile, String>
```

---

### get_library_files

Récupère les fichiers de la bibliothèque.

```typescript
// Frontend
const files = await invoke("get_library_files", {
  folderId: "uuid", // optionnel, null = racine
});
```

```rust
// Backend
#[tauri::command]
async fn get_library_files(
    state: State<'_, AppState>,
    folder_id: Option<String>,
) -> Result<Vec<LibraryFile>, String>
```

---

### get_file_content

Récupère le contenu d'un fichier.

```typescript
// Frontend
const content = await invoke("get_file_content", { fileId: "uuid" });
```

```rust
// Backend
#[tauri::command]
async fn get_file_content(
    state: State<'_, AppState>,
    file_id: String,
) -> Result<String, String>
```

---

### delete_file

Supprime un fichier de la bibliothèque.

```typescript
// Frontend
await invoke("delete_file", { fileId: "uuid" });
```

```rust
// Backend
#[tauri::command]
async fn delete_file(
    state: State<'_, AppState>,
    file_id: String,
) -> Result<(), String>
```

---

### ingest_file

Indexe un fichier dans le système RAG.

```typescript
// Frontend
await invoke("ingest_file", {
  fileId: "uuid",
  content: "File text content...",
});
```

```rust
// Backend
#[tauri::command]
async fn ingest_file(
    state: State<'_, AppState>,
    file_id: String,
    content: String,
) -> Result<(), String>
```

---

## 📂 Commandes Dossiers

### create_folder

Crée un nouveau dossier dans la bibliothèque.

```typescript
// Frontend
const folder = await invoke("create_folder", { name: "Documents" });
```

```rust
// Backend
#[tauri::command]
async fn create_folder(
    state: State<'_, AppState>,
    name: String,
) -> Result<Folder, String>
```

---

### get_all_folders

Récupère tous les dossiers.

```typescript
// Frontend
const folders = await invoke("get_all_folders");
```

```rust
// Backend
#[tauri::command]
async fn get_all_folders(
    state: State<'_, AppState>,
) -> Result<Vec<Folder>, String>
```

---

### delete_folder

Supprime un dossier.

```typescript
// Frontend
await invoke("delete_folder", { folderId: "uuid" });
```

```rust
// Backend
#[tauri::command]
async fn delete_folder(
    state: State<'_, AppState>,
    folder_id: String,
) -> Result<(), String>
```

---

## 🔗 Commandes Liaison Session-Fichier

### link_file_to_session

Lie un fichier de la bibliothèque à une session.

```typescript
// Frontend
await invoke("link_file_to_session", {
  sessionId: "uuid",
  fileId: "uuid",
});
```

```rust
// Backend
#[tauri::command]
async fn link_file_to_session(
    state: State<'_, AppState>,
    session_id: String,
    file_id: String,
) -> Result<(), String>
```

---

### get_session_files

Récupère les fichiers liés à une session.

```typescript
// Frontend
const sessionFiles = await invoke("get_session_files", {
  sessionId: "uuid",
});
```

```rust
// Backend
#[tauri::command]
async fn get_session_files(
    state: State<'_, AppState>,
    session_id: String,
) -> Result<Vec<SessionFile>, String>
```

---

## ⚠️ Gestion des Erreurs

Toutes les commandes retournent `Result<T, String>` côté Rust, ce qui se traduit par une Promise qui peut rejeter avec un message d'erreur.

### Pattern d'Appel Recommandé

```typescript
async function callTauriCommand<T>(
  command: string,
  args?: Record<string, unknown>
): Promise<T> {
  try {
    return await invoke<T>(command, args);
  } catch (error) {
    // L'erreur est une string côté Rust
    console.error(`Command ${command} failed:`, error);
    throw new Error(String(error));
  }
}

// Utilisation
const sessions = await callTauriCommand<Session[]>("get_all_sessions");
```

---

## 🔒 Rate Limiting

Les commandes de chat (`debug_chat`) sont soumises au rate limiting :

- **Limite :** 20 requêtes par minute par session
- **Réponse si dépassé :** `"Rate limit exceeded. Please wait before sending more messages."`

---

## 📊 Tableau Récapitulatif

| Commande               | Catégorie | Rate Limited | Events                                       |
| ---------------------- | --------- | ------------ | -------------------------------------------- |
| `run_preflight`        | Système   | ❌           | ❌                                           |
| `download_model`       | Système   | ❌           | ✅ progress, status                          |
| `run_diagnostic_test`  | Système   | ❌           | ❌                                           |
| `create_session`       | Session   | ❌           | ❌                                           |
| `get_all_sessions`     | Session   | ❌           | ❌                                           |
| `get_session`          | Session   | ❌           | ❌                                           |
| `delete_session`       | Session   | ❌           | ❌                                           |
| `rename_session`       | Session   | ❌           | ❌                                           |
| `update_session_order` | Session   | ❌           | ❌                                           |
| `get_messages`         | Message   | ❌           | ❌                                           |
| `delete_message`       | Message   | ❌           | ❌                                           |
| `debug_chat`           | Chat      | ✅           | ✅ chat-token, thinking-step, brain-analysis |
| `upload_library_file`  | Fichier   | ❌           | ❌                                           |
| `get_library_files`    | Fichier   | ❌           | ❌                                           |
| `get_file_content`     | Fichier   | ❌           | ❌                                           |
| `delete_file`          | Fichier   | ❌           | ❌                                           |
| `ingest_file`          | RAG       | ❌           | ❌                                           |
| `create_folder`        | Dossier   | ❌           | ❌                                           |
| `get_all_folders`      | Dossier   | ❌           | ❌                                           |
| `delete_folder`        | Dossier   | ❌           | ❌                                           |
| `link_file_to_session` | Liaison   | ❌           | ❌                                           |
| `get_session_files`    | Liaison   | ❌           | ❌                                           |

---

_Généré depuis lecture directe de: main.rs (generate_handler!, toutes les fonctions #[tauri::command])_
