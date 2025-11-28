# ⚙️ Configuration - WhytChat V1

> Fichiers de configuration du projet

---

## 📁 Fichiers de Configuration

```
WhytChat_V1/
├── Cargo.toml                    # Workspace Cargo (racine)
├── package.json                  # Scripts npm (racine)
├── apps/
│   ├── core/
│   │   ├── Cargo.toml           # Dépendances Rust
│   │   ├── tauri.conf.json      # Configuration Tauri
│   │   └── rust-toolchain.toml  # Version Rust
│   └── desktop-ui/
│       ├── package.json         # Dépendances React
│       ├── vite.config.js       # Configuration Vite
│       ├── tailwind.config.js   # Configuration Tailwind
│       └── postcss.config.js    # Configuration PostCSS
```

---

## 🦀 Backend Rust

### Cargo.toml (Workspace Root)

```toml
[workspace]
members = ["apps/core"]
resolver = "2"

[profile.release]
lto = true
codegen-units = 1
opt-level = 3
```

### apps/core/Cargo.toml

```toml
[package]
name = "whytchat-core"
version = "0.1.0"
edition = "2021"
rust-version = "1.80.0"
authors = ["WhytChat Team"]
description = "Local-first AI chat application"

[build-dependencies]
tauri-build = { version = "2.0.0-rc", features = [] }

[dependencies]
# === Tauri ===
tauri = { version = "2.0.0-rc", features = ["protocol-asset"] }
tauri-plugin-shell = "2.0.0-rc"

# === Async Runtime ===
tokio = { version = "1.40", features = ["full", "sync", "time", "macros", "rt-multi-thread"] }

# === Database ===
sqlx = { version = "0.8", features = ["runtime-tokio", "sqlite", "uuid", "chrono"] }

# === Serialization ===
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# === UUID & Time ===
uuid = { version = "1.10", features = ["v4", "serde"] }
chrono = { version = "0.4", features = ["serde"] }

# === HTTP Client ===
reqwest = { version = "0.12", features = ["json", "stream"] }
futures-util = "0.3"

# === Vector Database ===
lancedb = "0.10"
arrow-array = "52"
arrow-schema = "52"

# === Embeddings ===
fastembed = "4"

# === Encryption ===
aes-gcm = "0.10"
rand = "0.8"
pbkdf2 = { version = "0.12", features = ["simple"] }
sha2 = "0.10"

# === Regex ===
regex = "1.10"

# === Logging ===
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }

# === Text Extraction ===
pdf-extract = "0.7"
docx-rs = "0.4"
csv = "1.3"

# === Error Handling ===
thiserror = "1.0"
anyhow = "1.0"

# === Async Traits ===
async-trait = "0.1"

[features]
default = ["custom-protocol"]
custom-protocol = ["tauri/custom-protocol"]
```

### rust-toolchain.toml

```toml
[toolchain]
channel = "stable"
components = ["rustfmt", "clippy"]
targets = ["x86_64-pc-windows-msvc"]
```

---

## 🔧 tauri.conf.json

```json
{
  "$schema": "https://schema.tauri.app/config/2.0.0-rc",
  "productName": "WhytChat",
  "version": "0.1.0",
  "identifier": "com.whytchat.app",
  "build": {
    "beforeDevCommand": "npm run dev:ui",
    "devUrl": "http://localhost:1420",
    "beforeBuildCommand": "npm run build:ui",
    "frontendDist": "../desktop-ui/dist"
  },
  "app": {
    "windows": [
      {
        "title": "WhytChat",
        "width": 1200,
        "height": 800,
        "minWidth": 800,
        "minHeight": 600,
        "resizable": true,
        "fullscreen": false,
        "center": true
      }
    ],
    "security": {
      "csp": "default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; img-src 'self' data: asset: https://asset.localhost"
    }
  },
  "bundle": {
    "active": true,
    "icon": [
      "icons/32x32.png",
      "icons/128x128.png",
      "icons/128x128@2x.png",
      "icons/icon.icns",
      "icons/icon.ico"
    ],
    "targets": ["nsis", "msi"],
    "windows": {
      "certificateThumbprint": null,
      "digestAlgorithm": "sha256",
      "timestampUrl": ""
    }
  },
  "plugins": {
    "shell": {
      "open": true
    }
  }
}
```

### Configuration Clé

| Paramètre            | Valeur                | Description                    |
| -------------------- | --------------------- | ------------------------------ |
| `identifier`         | com.whytchat.app      | ID unique de l'application     |
| `devUrl`             | http://localhost:1420 | URL du serveur Vite            |
| `frontendDist`       | ../desktop-ui/dist    | Dossier de build frontend      |
| `minWidth/minHeight` | 800x600               | Taille minimale de la fenêtre  |
| `targets`            | nsis, msi             | Formats d'installateur Windows |

---

## ⚛️ Frontend React

### apps/desktop-ui/package.json

```json
{
  "name": "@whytchat/desktop-ui",
  "private": true,
  "version": "0.1.0",
  "type": "module",
  "scripts": {
    "dev": "vite",
    "build": "vite build",
    "preview": "vite preview",
    "lint": "eslint . --ext js,jsx",
    "lint:fix": "eslint . --ext js,jsx --fix",
    "test": "playwright test",
    "test:ui": "playwright test --ui"
  },
  "dependencies": {
    "@tauri-apps/api": "^2.0.0",
    "@tauri-apps/plugin-shell": "^2.0.0",
    "react": "^18.3.1",
    "react-dom": "^18.3.1",
    "react-markdown": "^9.0.1",
    "react-syntax-highlighter": "^15.5.0",
    "zustand": "^4.5.4",
    "i18next": "^23.14.0",
    "react-i18next": "^14.1.3",
    "i18next-browser-languagedetector": "^8.0.0",
    "lucide-react": "^0.439.0",
    "clsx": "^2.1.1",
    "tailwind-merge": "^2.5.2"
  },
  "devDependencies": {
    "@vitejs/plugin-react": "^4.3.1",
    "vite": "^5.4.1",
    "tailwindcss": "^3.4.10",
    "@tailwindcss/typography": "^0.5.14",
    "postcss": "^8.4.41",
    "autoprefixer": "^10.4.20",
    "eslint": "^8.57.0",
    "eslint-plugin-react": "^7.35.0",
    "eslint-plugin-react-hooks": "^4.6.2",
    "@playwright/test": "^1.46.1"
  }
}
```

### vite.config.js

```javascript
import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import path from "path";

export default defineConfig({
  plugins: [react()],

  resolve: {
    alias: {
      "@": path.resolve(__dirname, "./src"),
    },
  },

  server: {
    port: 1420,
    strictPort: true,
    host: true,
  },

  build: {
    outDir: "dist",
    sourcemap: true,
    rollupOptions: {
      output: {
        manualChunks: {
          vendor: ["react", "react-dom"],
          ui: ["lucide-react", "react-markdown"],
        },
      },
    },
  },

  // Ne pas masquer les erreurs Rust
  clearScreen: false,

  // Variables d'environnement Tauri
  envPrefix: ["VITE_", "TAURI_"],
});
```

### tailwind.config.js

```javascript
/** @type {import('tailwindcss').Config} */
export default {
  darkMode: "class",

  content: ["./index.html", "./src/**/*.{js,jsx,ts,tsx}"],

  theme: {
    extend: {
      colors: {
        background: "hsl(var(--background))",
        foreground: "hsl(var(--foreground))",
        primary: {
          DEFAULT: "hsl(var(--primary))",
          foreground: "hsl(var(--primary-foreground))",
        },
        secondary: {
          DEFAULT: "hsl(var(--secondary))",
          foreground: "hsl(var(--secondary-foreground))",
        },
        muted: {
          DEFAULT: "hsl(var(--muted))",
          foreground: "hsl(var(--muted-foreground))",
        },
        accent: {
          DEFAULT: "hsl(var(--accent))",
          foreground: "hsl(var(--accent-foreground))",
        },
        destructive: {
          DEFAULT: "hsl(var(--destructive))",
          foreground: "hsl(var(--destructive-foreground))",
        },
        border: "hsl(var(--border))",
        input: "hsl(var(--input))",
        ring: "hsl(var(--ring))",
      },
      borderRadius: {
        lg: "var(--radius)",
        md: "calc(var(--radius) - 2px)",
        sm: "calc(var(--radius) - 4px)",
      },
      fontFamily: {
        sans: ["Inter", "system-ui", "sans-serif"],
        mono: ["JetBrains Mono", "monospace"],
      },
    },
  },

  plugins: [require("@tailwindcss/typography")],
};
```

### postcss.config.js

```javascript
export default {
  plugins: {
    tailwindcss: {},
    autoprefixer: {},
  },
};
```

---

## 📜 Scripts npm (Racine)

### package.json (Root)

```json
{
  "name": "whytchat",
  "private": true,
  "version": "0.1.0",
  "scripts": {
    "dev": "npm run tauri dev",
    "dev:ui": "npm run --prefix apps/desktop-ui dev",
    "build": "npm run tauri build",
    "build:ui": "npm run --prefix apps/desktop-ui build",
    "tauri": "tauri",
    "lint": "npm run lint:rust && npm run lint:js",
    "lint:rust": "cd apps/core && cargo clippy -- -D warnings",
    "lint:js": "npm run --prefix apps/desktop-ui lint",
    "test": "npm run test:rust && npm run test:e2e",
    "test:rust": "cargo test --manifest-path apps/core/Cargo.toml",
    "test:e2e": "npm run --prefix apps/desktop-ui test"
  },
  "devDependencies": {
    "@tauri-apps/cli": "^2.0.0-rc"
  }
}
```

### Commandes Principales

| Commande        | Description                                |
| --------------- | ------------------------------------------ |
| `npm run dev`   | Démarrer en mode développement             |
| `npm run build` | Construire l'application (release)         |
| `npm run lint`  | Linter Rust (Clippy) + JavaScript (ESLint) |
| `npm run test`  | Tests Rust + Tests E2E Playwright          |

---

## 🔐 Capabilities Tauri

### apps/core/capabilities/default.json

```json
{
  "$schema": "https://schemas.tauri.app/capability/2.0.0-rc",
  "identifier": "default",
  "description": "Default capabilities for WhytChat",
  "windows": ["main"],
  "permissions": ["core:default", "shell:allow-open", "shell:allow-execute"]
}
```

---

## 🗄️ Variables d'Environnement

### Développement

```env
# Non requis - tout est local
# Les chemins sont relatifs au dossier data/
```

### Runtime

| Variable           | Définie Par  | Usage                            |
| ------------------ | ------------ | -------------------------------- |
| `LLAMA_AUTH_TOKEN` | Runtime Rust | Token auth llama-server          |
| `DATABASE_URL`     | fs_manager   | sqlite://data/db/whytchat.sqlite |

---

## 📊 Schéma Base de Données

### migrations/20251123140220_initial_schema.sql

```sql
CREATE TABLE IF NOT EXISTS sessions (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS messages (
    id TEXT PRIMARY KEY NOT NULL,
    session_id TEXT NOT NULL,
    role TEXT NOT NULL,
    content TEXT NOT NULL,
    created_at TEXT NOT NULL,
    tokens INTEGER,
    FOREIGN KEY (session_id) REFERENCES sessions(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS config (
    key TEXT PRIMARY KEY NOT NULL,
    value BLOB NOT NULL
);
```

### migrations/20251125210000_add_session_sorting.sql

```sql
ALTER TABLE sessions ADD COLUMN is_pinned INTEGER NOT NULL DEFAULT 0;
ALTER TABLE sessions ADD COLUMN sort_order INTEGER;
```

### migrations/20251126100000_global_library.sql

```sql
CREATE TABLE IF NOT EXISTS library_files (
    id TEXT PRIMARY KEY NOT NULL,
    folder_id TEXT,
    name TEXT NOT NULL,
    file_type TEXT NOT NULL,
    size_bytes INTEGER NOT NULL,
    created_at TEXT NOT NULL,
    is_indexed INTEGER NOT NULL DEFAULT 0,
    FOREIGN KEY (folder_id) REFERENCES folders(id) ON DELETE SET NULL
);

CREATE TABLE IF NOT EXISTS session_files (
    session_id TEXT NOT NULL,
    file_id TEXT NOT NULL,
    added_at TEXT NOT NULL,
    PRIMARY KEY (session_id, file_id),
    FOREIGN KEY (session_id) REFERENCES sessions(id) ON DELETE CASCADE,
    FOREIGN KEY (file_id) REFERENCES library_files(id) ON DELETE CASCADE
);
```

### migrations/20251126130000_add_document_folders.sql

```sql
CREATE TABLE IF NOT EXISTS folders (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL,
    created_at TEXT NOT NULL
);
```

---

## 🎯 Modèle LLM

### Configuration par Défaut

| Paramètre      | Valeur                                |
| -------------- | ------------------------------------- |
| Modèle         | Qwen2.5-Coder-7B-Instruct-Q4_K_M.gguf |
| Taille         | ~4.7 GB                               |
| Context Length | 4096 tokens                           |
| Temperature    | 0.7                                   |
| GPU Layers     | -1 (auto)                             |
| Server Port    | 8765                                  |

### Chemin du Modèle

```
data/models/default-model.gguf
```

### Embeddings

| Paramètre  | Valeur                  |
| ---------- | ----------------------- |
| Modèle     | AllMiniLML6V2           |
| Dimensions | 384                     |
| Cache      | data/models/embeddings/ |

---

_Généré depuis lecture directe de: Cargo.toml (workspace et core), package.json (root et desktop-ui), tauri.conf.json, vite.config.js, tailwind.config.js, migrations/\*.sql_
